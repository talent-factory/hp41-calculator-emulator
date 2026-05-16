# Architecture Patterns

**Domain:** HP-41 Calculator Emulator — v2.0 Tauri GUI Integration
**Researched:** 2026-05-09
**Focus:** How `hp41-gui` (Tauri v2 + React + TypeScript) integrates with `hp41-core`

---

## Summary

`hp41-core` is already designed for exactly this use case: a UI-agnostic library with a single mutable state (`CalcState`), a single entry point (`dispatch(op, state)`), and full serde coverage. Tauri v2's managed-state pattern maps cleanly onto this. The integration is shallow by design — `hp41-gui` is a thin adapter crate, not a reimplementation.

**The canonical pattern:**
- `CalcState` lives in `Mutex<CalcState>` managed by Tauri's `AppState`.
- Every key click → `invoke("dispatch_op", { op: "Sin" })` → Tauri command locks mutex, calls `dispatch(op, &mut state)`, returns `CalcStateView` snapshot to frontend.
- Frontend re-renders from the returned snapshot. No push events needed for synchronous ops.
- Key-to-Op mapping lives in Rust (not TypeScript). The frontend sends a key identifier; Rust resolves it to `Op`.

**What does NOT change:** `hp41-core` is untouched. Zero new dependencies are added to it. `hp41-cli` continues to build and run unchanged.

---

## Data Flow

```
SVG Key Click (TypeScript)
  │
  │  invoke("dispatch_op", { keyId: "sin" })
  ▼
Tauri IPC (JSON over webview message channel)
  │
  ▼
dispatch_op command (src-tauri/src/commands.rs)
  │  1. lock Mutex<CalcState>
  │  2. resolve keyId → Op  (key_map.rs)
  │  3. hp41_core::ops::dispatch(&mut state, op)
  │  4. build CalcStateView from state
  │  5. drain print_buffer
  ▼
Return CalcStateView (serialized to JSON by Tauri)
  │
  ▼
React state update (useState / useReducer)
  │
  ▼
Re-render: display panel + annunciators + print output
```

**Key invariants preserved:**
- `flush_entry_buf()` is called inside `dispatch()` — no change needed.
- `is_running` is reset to `false` after `load_state` (same as `hp41-cli`).
- `print_buffer` is drained by the Tauri command layer (analogous to `hp41-cli`'s `call_dispatch_and_drain()`).
- `prgm_mode` is a field in `CalcState`; the frontend reflects it from the returned view.

---

## New Components

### Modified (workspace-level)

| File | Change | Reason |
|------|--------|--------|
| `Cargo.toml` (root) | Add `"hp41-gui"` to `members` | Register new workspace member |
| `Justfile` | Add `gui-dev`, `gui-build`, `gui-test` recipes | Expose Tauri CLI through `just` |

### New crate: `hp41-gui/`

Standard Tauri v2 structure. `src-tauri/` is the Rust crate; `src/` (or `ui/src/`) holds React/TypeScript.

```
hp41-gui/
  src-tauri/
    Cargo.toml                  ← workspace member; depends on hp41-core
    build.rs                    ← tauri_build::build()
    tauri.conf.json             ← app metadata, devUrl, frontendDist, bundle
    capabilities/
      default.json              ← core:default + custom hp41 commands
    src/
      main.rs                   ← desktop entry: calls lib::run()
      lib.rs                    ← #[cfg_attr(mobile, tauri::mobile_entry_point)] pub fn run()
      commands.rs               ← #[tauri::command] functions
      key_map.rs                ← key_id string → Op resolution
      state.rs                  ← CalcStateView (serializable snapshot)
      error.rs                  ← CommandError (implements serde::Serialize)
      persistence.rs            ← reuses save_state/load_state logic from hp41-cli
  ui/                           ← Vite + React + TypeScript frontend
    src/
      main.tsx
      App.tsx
      components/
        Calculator.tsx           ← top-level layout
        Display.tsx              ← 12-char dot-matrix + annunciators
        Keyboard.tsx             ← SVG skin with clickable regions
        PrintOutput.tsx          ← print buffer display
      hooks/
        useCalcState.ts          ← invoke wrapper + local state
      bindings.ts               ← hand-written or tauri-specta generated types
    index.html
    vite.config.ts
    package.json
    tsconfig.json
```

### New files in `hp41-gui/src-tauri/src/`

**`state.rs` — CalcStateView**

`CalcState` is too large and has internal fields (e.g., `program: Vec<Op>`, `call_stack`) that the frontend does not need every frame. Derive a `CalcStateView` as the IPC return type — a serializable subset covering what the display needs:

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalcStateView {
    pub display_x: String,       // format_hpnum(&state.stack.x, &state.display_mode)
    pub display_y: String,
    pub display_z: String,
    pub display_t: String,
    pub display_lastx: String,
    pub alpha_reg: String,
    pub alpha_mode: bool,
    pub prgm_mode: bool,
    pub user_mode: bool,
    pub angle_mode: String,      // "DEG" | "RAD" | "GRAD"
    pub display_mode: String,    // "FIX 4" etc.
    pub is_running: bool,
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>, // drained print_buffer
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Annunciators {
    pub user: bool,
    pub prgm: bool,
    pub alpha: bool,
    pub rad: bool,
    pub grad: bool,
    pub run: bool,    // is_running
}
```

**`error.rs` — CommandError**

`HpError` does not implement `serde::Serialize` (by design — `hp41-core` has no serde feature on errors). Add a thin adapter in `hp41-gui`:

```rust
#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub kind: String,    // HpError::to_string()
}

impl From<hp41_core::HpError> for CommandError {
    fn from(e: hp41_core::HpError) -> Self {
        CommandError { kind: e.to_string() }
    }
}
```

All commands return `Result<CalcStateView, CommandError>`.

**`key_map.rs` — Key ID → Op resolution**

The frontend sends a stable string key identifier (e.g., `"sin"`, `"sto_reg_5"`, `"push_num_3"`). Rust resolves to `Op`. This keeps Op serialization complexity out of the IPC layer:

```rust
pub fn key_id_to_op(id: &str) -> Option<hp41_core::ops::Op> {
    match id {
        "sin"    => Some(Op::Sin),
        "cos"    => Some(Op::Cos),
        "enter"  => Some(Op::Enter),
        "clx"    => Some(Op::Clx),
        // digit keys: "digit_0" .. "digit_9", "dot", "eex", "chs"
        // handled by special path: append to entry_buf, not dispatch
        _ => None,
    }
}
```

Digit keys (`0`–`9`, `.`, `EEX`, `CHS`) are special: instead of dispatching `Op::PushNum`, the Tauri command appends to `CalcState.entry_buf` and returns a view (no `flush_entry_buf` yet, exactly mirroring `hp41-cli` key handling).

**`commands.rs` — Tauri commands**

```rust
#[tauri::command]
pub fn dispatch_op(
    key_id: String,
    state: tauri::State<'_, Mutex<CalcState>>,
) -> Result<CalcStateView, CommandError> {
    let mut calc = state.lock().expect("CalcState mutex poisoned");
    handle_key_id(&key_id, &mut calc)?;
    Ok(build_view(&calc))
}

#[tauri::command]
pub fn load_state_cmd(
    state: tauri::State<'_, Mutex<CalcState>>,
) -> Result<CalcStateView, CommandError> { ... }

#[tauri::command]
pub fn save_state_cmd(
    state: tauri::State<'_, Mutex<CalcState>>,
) -> Result<(), CommandError> { ... }
```

`handle_key_id` is not a Tauri command — it's plain Rust called from within commands. This keeps the command signatures thin.

---

## IPC Design

### Recommendation: Key-ID string API (not Op enum over IPC)

Do NOT serialize `Op` variants over IPC. The `Op` enum contains variants with nested data (e.g., `Op::StoArith { reg: u8, kind: StoArithKind }`, `Op::PushNum(HpNum)`, `Op::Lbl(String)`) that would require complex serde tagging and TypeScript union types to model correctly. This creates a brittle coupling where every new Op variant requires frontend type updates.

Instead, use a **stable key-identifier string API**:

| Frontend sends | Rust resolves |
|---------------|---------------|
| `"sin"` | `Op::Sin` |
| `"sto_reg_05"` | `Op::StoReg(5)` |
| `"fmt_fix_4"` | `Op::FmtFix(4)` |
| `"digit_3"` | append `"3"` to `entry_buf` |
| `"alpha_a"` | `Op::AlphaAppend('a')` |
| `"gto_lbl"` + `label` param | `Op::Gto(label)` |

Multi-step UI flows (e.g., STO arithmetic modal that hp41-cli implements as a 3-step keyboard state machine) are implemented as multi-step React state — the frontend manages which step it is in, then sends the fully-resolved key-ID once complete (e.g., `"sto_arith_add_reg_05"`).

### Single command surface

Use **one primary command**: `dispatch_op(key_id: string) -> CalcStateView`. Additional commands are support only:

| Command | Purpose |
|---------|---------|
| `dispatch_op` | All key inputs |
| `load_state` | Load from `~/.hp41/autosave.json` |
| `save_state` | Manual save |
| `get_state` | Initial hydration on app start |
| `get_program_listing` | Fetch `Vec<String>` for program display panel |

### Return value: always CalcStateView

Every `dispatch_op` call returns a fresh `CalcStateView`. The frontend replaces its entire React state from this view. No partial updates, no delta patching, no events. This is the simplest correct model for a single-user single-window app with ~65 ns/op dispatch.

### No push events for synchronous ops

Tauri events (`app.emit()`) are for asynchronous notifications (e.g., auto-save completion). They are explicitly not suited for low-latency command-response cycles (Tauri docs: "not designed for low latency or high throughput situations"). Do not use events for key dispatch results.

---

## State Management

### CalcState lives in `Mutex<CalcState>` in Tauri AppState

**Use `std::sync::Mutex`, not `tokio::sync::Mutex`.** The hp41-core dispatch is synchronous, fast (~65 ns), and never crosses an await point. `std::sync::Mutex` is appropriate and preferred per Tauri's own documentation.

**No `Arc` needed.** Tauri wraps managed state in its own reference-counted container. Adding `Arc` is redundant.

```rust
// In lib.rs run():
tauri::Builder::default()
    .setup(|app| {
        let state = load_or_new_calc_state();
        app.manage(Mutex::new(state));
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        commands::dispatch_op,
        commands::load_state_cmd,
        commands::save_state_cmd,
        commands::get_state,
        commands::get_program_listing,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application")
```

### No per-command reconstruction

Do NOT reconstruct `CalcState` from scratch per command and do NOT pass the full serialized state from the frontend on each call. The state is server-side (Rust). Frontend holds only `CalcStateView` for rendering.

### Persistence reuse

`hp41-cli`'s `save_state` / `load_state` logic uses only `serde_json` and `std::fs`. Copy (do not share) this module into `hp41-gui/src-tauri/src/persistence.rs` — it is three functions and straightforward to maintain in both crates. Using the same `~/.hp41/autosave.json` path means the CLI and GUI share a save file, which is the correct HP-41 behavior (one calculator, two interfaces). Ensure `is_running = false` is reset on load (already enforced in the existing code).

### Auto-save

Tauri has no built-in interval timer at the app level. Use a background thread spawned in `setup()` that wakes every 30 seconds (matching v1.0 CLI behavior) and calls `save_state`. Access state via `app.state::<Mutex<CalcState>>()` cloned into the thread via `AppHandle::clone()`.

---

## Build Order

The following phase sequence is recommended based on dependencies:

### Phase 1: Workspace skeleton and Tauri shell

Create `hp41-gui/` as a workspace member. Scaffold the Tauri v2 project (`cargo tauri init` or manual). Confirm `just build` and `just gui-dev` work with an empty "Hello World" frontend. No hp41-core integration yet. Success criterion: Tauri window opens.

### Phase 2: CalcState in AppState + get_state command

Wire `hp41-core` dependency into `hp41-gui/src-tauri/Cargo.toml`. Implement `Mutex<CalcState>` in AppState. Add `get_state` command returning `CalcStateView`. Implement `build_view()`. Success criterion: frontend can fetch initial state on startup.

### Phase 3: dispatch_op command + key_map + display rendering

Implement `key_map.rs`, `dispatch_op` command, and digit-entry path. Build React `Display` component consuming `CalcStateView`. Success criterion: digit entry and arithmetic (`+`, `-`, `×`, `÷`, `Enter`) work end to end with correct display update.

### Phase 4: SVG keyboard skin

Build the SVG HP-41C layout with clickable regions. Each key's `onClick` calls `invoke("dispatch_op", { keyId: "..." })`. This phase is pure frontend work against the Phase 3 backend. Success criterion: all 130 ops reachable by click, display updates after every click.

### Phase 5: Persistence, annunciators, print output

Add `load_state_cmd`, `save_state_cmd`, and the 30-second auto-save background thread. Add annunciator rendering (USER, PRGM, RAD/GRAD, ALPHA). Add `PrintOutput` component draining `print_lines` from `CalcStateView`. Success criterion: state survives app restart; annunciators match CLI behavior.

### Phase 6: Program listing panel + PRGM mode UI

Add `get_program_listing` command. Render program listing panel (equivalent to ratatui PRGM panel). PRGM mode entry/exit visible in the GUI. Success criterion: programs can be recorded and run from the GUI.

---

## Justfile Integration

Add these recipes to the root Justfile. All use `cargo tauri` CLI through `npm`/`pnpm` — the standard Tauri v2 toolchain. Tauri's CLI wraps `cargo` internally.

```just
# GUI: install frontend dependencies (run once after clone)
gui-install:
    cd hp41-gui && npm install

# GUI: development mode (hot-reload frontend + Rust watch)
gui-dev:
    cd hp41-gui && npm run tauri dev

# GUI: production build (bundles native app)
gui-build:
    cd hp41-gui && npm run tauri build

# GUI: build only the Rust backend (useful for CI type-checking)
gui-check:
    cargo check -p hp41-gui

# Full CI gate including GUI Rust check (not the full Tauri bundle)
ci-full: lint test coverage gui-check
```

**Rationale:**
- `gui-dev` and `gui-build` must be run from `hp41-gui/` because `tauri.conf.json` and `package.json` are there. The `cd hp41-gui &&` prefix achieves this without breaking the "no bare `cargo`" rule — `npm run tauri` is the entry point, not `cargo` directly.
- `gui-check` is added to `ci-full` (a new recipe) rather than to the existing `ci` recipe. The existing `ci` recipe must remain identical for the CLI build matrix. CI can run `ci` for the CLI job and `ci-full` or `gui-check` separately.
- `just build` (existing) continues to build only `hp41-core` and `hp41-cli` via `cargo build --workspace`. To avoid pulling Tauri's heavy build-time dependencies into the standard workspace build, `hp41-gui/src-tauri` should either be excluded from `cargo build --workspace` by not being a workspace member at the root level (using a nested workspace) or by excluding it conditionally. The recommended approach: make `hp41-gui/src-tauri` a standalone Cargo workspace (not a member of the root workspace) and depend on `hp41-core` via path. This keeps `cargo build --workspace` fast and `hp41-cli` CI unaffected.

**Nested workspace pattern (recommended):**

```
# Root Cargo.toml — unchanged:
[workspace]
members = ["hp41-core", "hp41-cli"]

# hp41-gui/src-tauri/Cargo.toml — standalone:
[package]
name = "hp41-gui"
...

[dependencies]
hp41-core = { path = "../../hp41-core" }
tauri = { version = "2", features = [...] }
```

This is the same pattern Tauri itself recommends (the `src-tauri` folder is described as optionally a workspace member or a standalone crate). It avoids forcing `tauri`, `wry`, and `tao` into every `cargo check` run.

---

## Key Architectural Decisions and Rationale

| Decision | Rationale |
|----------|-----------|
| `std::sync::Mutex<CalcState>` | hp41-core dispatch is sync, fast, never crosses await. Matches Tauri's own recommendation. |
| Key-ID string API over Op enum IPC | Avoids brittle serde tagging of complex enum variants. Stable contract: new Op variants don't break the frontend API. |
| CalcStateView (not full CalcState) | Keeps IPC payload small. `program: Vec<Op>` and `call_stack` are not needed for display. Avoids serializing HpNum (Decimal) to the frontend every frame — format to String in Rust where format logic lives. |
| Key-to-Op mapping in Rust | Op logic belongs in Rust. TypeScript should not need to know about Op variants, StoArithKind, or StackReg. |
| Shared save path `~/.hp41/autosave.json` | One calculator state across both interfaces. Users who use both CLI and GUI stay in sync. |
| Nested workspace (hp41-gui standalone) | Prevents Tauri/wry/tao from polluting `cargo build --workspace` and slowing CLI CI. |
| No auto-generated TypeScript bindings (tauri-specta) | tauri-specta adds build complexity and a new dependency. With a key-ID string API and a hand-written `CalcStateView` type, the TypeScript surface is small and stable. Re-evaluate if the command surface grows significantly. |
| Digit entry via entry_buf, not PushNum dispatch | Matches hp41-cli behavior exactly. The Tauri command layer appends digits to `state.entry_buf` directly; `flush_entry_buf()` is called by the next `dispatch()` invocation. This preserves HP-41 number entry semantics. |

---

## Constraints from hp41-core (must not be violated)

1. `hp41-core` has zero UI/CLI/Tauri dependencies — enforced at compile time. Never add `tauri` to `hp41-core/Cargo.toml`.
2. `HpError` does not implement `serde::Serialize`. Wrap in `CommandError` in `hp41-gui` only.
3. `#![deny(clippy::unwrap_used)]` is active in `hp41-core`. The new `hp41-gui` crate should adopt the same lint but is not required to.
4. `CalcState.print_buffer` must be drained after every dispatch that could produce print output (PRX/PRA/PRSTK, and any run_program path). The Tauri command layer owns this drain, returning lines in `CalcStateView.print_lines`.
5. `is_running = false` must be enforced on state load (already in `load_state`).

---

## Sources

- [Tauri v2 State Management](https://v2.tauri.app/develop/state-management/) — HIGH confidence (official docs)
- [Tauri v2 Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/) — HIGH confidence (official docs)
- [Tauri v2 Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — HIGH confidence (official docs)
- [Tauri v2 Project Structure](https://v2.tauri.app/start/project-structure/) — HIGH confidence (official docs)
- [Tauri v2 Architecture](https://v2.tauri.app/concept/architecture/) — HIGH confidence (official docs)
- [tauri-specta GitHub](https://github.com/specta-rs/tauri-specta) — MEDIUM confidence (community library, actively maintained)
- hp41-core codebase — HIGH confidence (direct inspection of state.rs, ops/mod.rs, error.rs, persistence.rs)
