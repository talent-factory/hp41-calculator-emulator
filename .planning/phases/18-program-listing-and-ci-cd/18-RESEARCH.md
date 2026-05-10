# Phase 18: Program Listing & CI/CD — Research

**Researched:** 2026-05-10
**Domain:** Tauri v2 Rust command layer, React state + useEffect patterns, GitHub Actions CI
**Confidence:** HIGH — all findings verified against live source files in this repository

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Program Data in CalcStateView**
- D-01: Add `program_steps: Vec<String>` and `pc: usize` to `CalcStateView` unconditionally. The 300-byte size assertion in `types.rs` tests only on `CalcState::new()` (empty program); update the test comment to document this scope rather than adding a total-size assertion that breaks with real programs.
- D-02: `program_steps` is a `Vec<String>` where each entry is the formatted step string (`"{pc:03} {op_name}"` format from `prgm_display.rs`). Index 0 = step 000. The Rust backend formats all steps before sending; React only renders strings.
- D-03: Copy `hp41-cli/src/prgm_display.rs` into `hp41-gui/src-tauri/src/prgm_display.rs` and use it to format `program_steps` in `CalcStateView::from_state()`. The GUI copy may evolve independently.

**SST/BST Dispatch**
- D-04: SST and BST are implemented as two new Tauri commands: `sst_step` and `bst_step`. Each locks `AppState`, directly manipulates `CalcState.pc` (increment by 1 capped at `program.len()`, or `saturating_sub(1)` for BST), drains `print_buffer`, and returns `CalcStateView`. Not routed through `Op` dispatch.
- D-05: Register `sst_step` and `bst_step` in `lib.rs`'s `invoke_handler`. Add pure-Rust helpers `handle_sst()` and `handle_bst()` in `commands.rs`.
- D-06: SST/BST SVG key IDs: set `id: 'sst'` and `id: 'bst'` in `Keyboard.tsx KEY_DEFS`. Click handler calls `invoke<CalcStateView>('sst_step')` / `invoke<CalcStateView>('bst_step')` via `App.tsx`'s `handleClick`.
- D-07: Physical keyboard bindings: add `F7 → sst_step` and `F8 → bst_step` to `resolveKeyId()` in `App.tsx`.

**Program Listing Panel UI**
- D-08: The program listing panel appears below the `<Keyboard />` component inside `.calculator`, auto-shown when `calcState.annunciators.prgm === true`, hidden when `prgm_mode` is false.
- D-09: The panel is a scrollable `<div>` (`max-height: 160px`, `overflow-y: auto`) containing one `<div>` per step. The step at index `calcState.pc` receives a highlight class. Auto-scroll via `useEffect` + `scrollIntoView`.
- D-10: Window height grows from 700 to 900 in `tauri.conf.json` (`windows[0].height: 900`).
- D-11: The panel header shows `"PRGM — N steps"`. No close button — panel disappears automatically when user exits PRGM mode.

**CI Job for Tauri GUI**
- D-12: New file `.github/workflows/ci-gui.yml` — separate from the existing `ci.yml`.
- D-13: Path filter triggers on `hp41-gui/**` and `hp41-core/**` changes on push and pull_request.
- D-14: CI job runs `cargo build --release` of `hp41-gui/src-tauri` on all three platforms plus `npm install && npx tsc --noEmit` for the TypeScript frontend check. No full `tauri build` bundle.
- D-15: Linux CI step installs WebKit system dependencies before Cargo steps.
- D-16: Use `Swatinem/rust-cache@v2` and `actions/setup-node@v4`. Branch filter: `main` and `develop`.

### Claude's Discretion
- Exact CSS for the program listing panel (font size, step row height, highlight color) — consistent with existing dark calculator aesthetic, monospace font for step labels.
- Whether step rows alternate background shading for readability.
- Exact wording of the panel header (`"PRGM"` vs `"PRGM MODE"` vs step count format).
- Whether D-02 pads step numbers — follow `format_step()` behavior (`:03` zero-padding).

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PROG-01 | User can view the current program listing and navigate steps with SST/BST in GUI PRGM mode | Addressed by D-01–D-11: prgm_display.rs copy, CalcStateView enrichment, sst_step/bst_step commands, listing panel UI |
</phase_requirements>

---

## Summary

Phase 18 completes the v2.0 Tauri GUI milestone by adding two independent features: a PRGM-mode program listing panel, and a cross-platform CI job for the GUI. Both follow patterns already established in the v2.0 codebase and require no new architectural decisions.

The program listing panel is a data-plumbing exercise: the `hp41-cli` already has `prgm_display.rs` with exhaustive `op_display_name()` coverage for all Op variants. Copying that module into `hp41-gui/src-tauri/src/` (identical to the Phase 17 `persistence.rs` copy pattern) gives the backend the formatter it needs. `CalcStateView` gains two fields (`program_steps: Vec<String>` and `pc: usize`), populated in `from_state()`. SST and BST are thin Tauri command thunks that directly manipulate `CalcState.pc` — they do not go through `Op` dispatch, matching the CLI's F7/F8 special-case approach. The React side renders a conditionally-visible panel that auto-scrolls to the active step via `useEffect` + `scrollIntoView`.

The CI job is a straightforward copy-and-adapt of the existing `ci.yml` pattern, filtered to the GUI-relevant paths. The key difference is that `tauri build` is not feasible in CI (requires a fully signed environment and OS-specific bundler tooling), so `cargo build --release` for the Rust backend plus `npx tsc --noEmit` for the TypeScript frontend is the correct CI scope. Linux requires WebKit GTK development headers installed before Cargo steps.

**Primary recommendation:** Execute in two waves — Wave 1 (parallel): Rust backend changes (prgm_display copy, CalcStateView enrichment, sst_step/bst_step commands, permissions) and CI YAML. Wave 2: React frontend (program listing panel, handleClick/resolveKeyId routing, App.css, window height). Wave 2 is blocked on Wave 1 compiling.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Program step formatting | API/Backend (Rust) | — | `op_display_name()` is pure Rust; all formatting happens server-side, React only renders pre-formatted strings |
| SST/BST pc manipulation | API/Backend (Rust) | — | `CalcState.pc` is owned by the Rust mutex; direct mutation in Tauri command handlers, no client-side state |
| Program listing render | Browser/React | — | Conditional JSX panel; `useEffect` auto-scroll is browser-side DOM behavior |
| SST/BST keyboard routing | Browser/React | — | `resolveKeyId()` maps F7/F8; `handleClick` detects 'sst'/'bst' and routes to dedicated invoke calls |
| Tauri permission grants | API/Backend (Tauri config) | — | `permissions/<cmd>.toml` + `capabilities/default.json` grant must exist for Tauri v2.11 to allow IPC calls |
| CI build validation | CDN/Static (GitHub Actions) | — | Cross-platform job independent of app runtime; validates compilation and TypeScript type safety |

---

## Standard Stack

### Core (unchanged from existing codebase)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri | 2.11 | Desktop app shell, IPC bridge | Already in use; `#[tauri::command]` decorator handles serialization |
| hp41-core | local path | Calculator engine, CalcState, Op enum | Core invariant: GUI borrows unchanged |
| React | 19.2 | Frontend UI | Already in use since Phase 15 |
| @tauri-apps/api | 2.11 | Frontend `invoke()` bridge | Already in use |

[VERIFIED: hp41-gui/src-tauri/Cargo.toml, hp41-gui/package.json — read directly in this session]

### New in Phase 18

No new dependencies. This phase adds Rust source files and React JSX — all within the existing stack.

---

## Architecture Patterns

### System Architecture Diagram

```
  CalcState.program (Vec<Op>)
          |
          v
  prgm_display::format_all_steps()   ← new pub fn in GUI copy
          |
          v
  CalcStateView { program_steps: Vec<String>, pc: usize, ... }
          |
          | serde_json (Tauri IPC)
          v
  React calcState.program_steps
          |
  ┌───────┴───────────┐
  |                   |
  v                   v
annunciators.prgm?   calcState.pc
  YES → render        → highlight row, scrollIntoView
  NO  → hide panel

SST/BST:
  [F7 / SVG 'sst' click]
       |
  resolveKeyId / handleClick detect 'sst'
       |
  invoke('sst_step')        invoke('bst_step')
       |
  handle_sst(&mut CalcState):     handle_bst(&mut CalcState):
    pc = min(pc+1, program.len())   pc = pc.saturating_sub(1)
    drain print_buffer              drain print_buffer
    CalcStateView::from_state()     CalcStateView::from_state()
       |
  setCalcState(view) → re-render panel with new highlight
```

### Recommended Project Structure

```
hp41-gui/src-tauri/src/
├── commands.rs          # + handle_sst(), handle_bst(), sst_step cmd, bst_step cmd
├── types.rs             # + program_steps: Vec<String>, pc: usize in CalcStateView
├── prgm_display.rs      # NEW: copied from hp41-cli/src/prgm_display.rs
├── lib.rs               # + sst_step, bst_step in invoke_handler![]
├── key_map.rs           # unchanged (SST/BST bypass key_map)
└── permissions/
    ├── dispatch-op.toml # unchanged
    ├── get-state.toml   # unchanged
    ├── sst-step.toml    # NEW
    └── bst-step.toml    # NEW

hp41-gui/src/
├── App.tsx              # + program listing panel JSX, handleClick sst/bst routing
├── App.css              # + prgm-panel, step-row, step-active styles
└── Keyboard.tsx         # + id: 'sst' at row 3 col 2; id: 'bst' at row 4 col 8

hp41-gui/src-tauri/tauri.conf.json   # height: 700 → 900

.github/workflows/
├── ci.yml               # UNCHANGED
└── ci-gui.yml           # NEW: Rust + TS CI for hp41-gui
```

### Pattern 1: Copy-to-GUI Module Adaptation

**What:** Copy `hp41-cli/src/prgm_display.rs` to `hp41-gui/src-tauri/src/prgm_display.rs`. The imports `use hp41_core::ops::{Op, StackReg, StoArithKind}` and `use hp41_core::CalcState` compile identically in the GUI crate because `hp41-core` is already a dependency.

**Critical adaptation:** `op_display_name()` is currently `fn` (private) in the CLI copy. The GUI copy needs to expose it or add a new `pub fn format_all_steps(state: &CalcState) -> Vec<String>` so `types.rs::from_state()` can call it. The cleanest approach is to add:

```rust
// Source: derived from prgm_display.rs op_display_name() + CalcState.program iteration
pub fn format_all_steps(state: &CalcState) -> Vec<String> {
    if state.program.is_empty() {
        vec!["000 END".to_string()]
    } else {
        state
            .program
            .iter()
            .enumerate()
            .map(|(i, op)| format!("{i:03} {}", op_display_name(op)))
            .collect()
    }
}
```

[VERIFIED: prgm_display.rs read directly — op_display_name is private; format_step uses state.pc not an index]

**When to use:** Whenever a module developed for `hp41-cli` has no CLI-specific dependencies and can serve the GUI unchanged.

### Pattern 2: Thin Tauri Thunk + Pure-Rust Helper

**What:** Phase 14 established the pattern: `#[tauri::command] pub fn sst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError>` locks `AppState` and delegates to `handle_sst(&mut CalcState)`. The helper is unit-testable without a Tauri runtime.

```rust
// Source: commands.rs handle_op / handle_get_state pattern (Phase 14)
#[tauri::command]
pub fn sst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_sst(&mut calc)
}

pub fn handle_sst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    // Cap at program.len() — same as HP-41 hardware (pc stays at end, no wrap)
    if calc.pc < calc.program.len() {
        calc.pc += 1;
    }
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}

pub fn handle_bst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.pc = calc.pc.saturating_sub(1);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}
```

[VERIFIED: commands.rs read directly — pattern confirmed]

### Pattern 3: Tauri v2.11 Permission TOML

**What:** Tauri v2.11 does NOT auto-generate `allow-<cmd>` permissions for inline app commands. Each new command needs a TOML file in `src-tauri/permissions/`.

**Template** (from existing `dispatch-op.toml`):
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-sst-step"
description = "Allows the sst_step command."
commands.allow = ["sst_step"]
```

And `capabilities/default.json` must add `"allow-sst-step"` and `"allow-bst-step"` to its `permissions` array.

[VERIFIED: permissions/dispatch-op.toml and get-state.toml read directly; capabilities/default.json verified]

### Pattern 4: React handleClick SST/BST Routing

**What:** The current `handleClick` unconditionally calls `invoke<CalcStateView>('dispatch_op', { keyId })`. SST and BST use dedicated Tauri commands, not `dispatch_op`. Two approaches:

**Option A — Special-case inside handleClick:**
```typescript
const handleClick = useCallback((keyId: string) => {
  if (busyRef.current) return;
  busyRef.current = true;
  let invokePromise: Promise<CalcStateView>;
  if (keyId === 'sst') {
    invokePromise = invoke<CalcStateView>('sst_step');
  } else if (keyId === 'bst') {
    invokePromise = invoke<CalcStateView>('bst_step');
  } else {
    invokePromise = invoke<CalcStateView>('dispatch_op', { keyId });
  }
  invokePromise
    .then(view => setCalcState(view))
    .catch(err => console.error('invoke error:', err))
    .finally(() => { busyRef.current = false; });
}, []);
```

**Option B — Dedicated handlers wired at the resolveKeyId level:** F7/F8 resolve to `'sst'`/`'bst'` in `resolveKeyId()`, then `handleKey` dispatches to a special branch.

Option A is preferred — it keeps all routing inside `handleClick` and avoids adding a third handler function.

[VERIFIED: App.tsx handleClick read directly — confirmed it uses dispatch_op unconditionally; needs extension]

### Pattern 5: Auto-Scroll to Active Step

**What:** `useEffect` watching `calcState?.pc` scrolls the highlighted step into view.

```typescript
const activeStepRef = useRef<HTMLDivElement>(null);

useEffect(() => {
  activeStepRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}, [calcState?.pc]);
```

The active step `<div>` receives the ref: `ref={calcState.pc === i ? activeStepRef : null}`.

[VERIFIED: App.tsx Phase 17 printEndRef scroll pattern confirmed as analog]

### Anti-Patterns to Avoid

- **Routing SST/BST through key_map::resolve():** They are not `Op` variants — there is no `Op::Sst` or `Op::Bst`. They manipulate `pc` directly. Confirmed: `key_map.rs` has no sst/bst entries. [VERIFIED]
- **Using `format_step()` in a loop:** `format_step()` uses `state.pc` (current pc), not an index. Iterating with it would return the same step N times. Use `op_display_name()` via a new `format_all_steps()` helper. [VERIFIED]
- **`tauri build` in CI:** Requires native code signing, OS-specific bundler tools, and notarization on macOS. `cargo build --release` for the Rust layer plus `npx tsc --noEmit` is the correct CI scope as decided in D-14.
- **Modifying `ci.yml`:** Phase 18 adds `ci-gui.yml` only. The existing `ci.yml` is untouched per D-12/SC-5.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Op → display name mapping | Custom match in types.rs | `prgm_display.rs` copy (D-03) | Already exhaustive over all 35+ Op variants including Phase 12 synthetic ops |
| Mutex poisoning recovery | `.unwrap()` | `.unwrap_or_else(|e| e.into_inner())` | Zero-panic policy (`#![deny(clippy::unwrap_used)]`) requires this exact pattern |
| SST/BST as Op dispatch | Add Op::Sst / Op::Bst | Direct pc manipulation in command helpers | HP-41 hardware: SST/BST are navigation keys, not program ops |
| Tauri permissions auto-detection | Rely on auto-generation | Explicit TOML in `permissions/` | Tauri v2.11 does not auto-generate allow-<cmd> for inline app commands |

---

## Common Pitfalls

### Pitfall 1: The 300-Byte CalcStateView Size Assertion

**What goes wrong:** `types.rs` test `test_dispatch_op_payload_size` asserts `json.len() <= 300`. Adding `program_steps: Vec<String>` and `pc: usize` to `CalcStateView` means a real program breaks this limit immediately.

**Why it happens:** The original assertion was written for a lean DTO. With `program_steps`, the JSON payload grows with each step stored.

**How to avoid:** D-01 is explicit: update the test comment to document that the assertion scope is `CalcState::new()` (empty program only). The assertion can be left in place for the empty-program case — with an empty program, `program_steps = ["000 END"]` adds ~35 bytes, which may push the total past 300. Verify after implementation. If it fails, relax the assertion to cover empty-program only (e.g., ≤350 bytes) or replace it with a structural test that checks `program_steps == ["000 END"]` and `pc == 0` for `CalcState::new()`.

**Warning signs:** `test_dispatch_op_payload_size` failing in CI after adding the new fields.

### Pitfall 2: op_display_name is Private

**What goes wrong:** Copying `prgm_display.rs` verbatim and then calling `op_display_name()` from `types.rs::from_state()` produces a compilation error: `function 'op_display_name' is private`.

**Why it happens:** The CLI module has `fn op_display_name()` (private) because the CLI only needs `format_step()` externally. The GUI needs `op_display_name()` to build the full listing.

**How to avoid:** Add `pub fn format_all_steps(state: &CalcState) -> Vec<String>` to the GUI copy of `prgm_display.rs`. Keep `op_display_name()` private — only the new public function crosses the module boundary. `format_all_steps()` handles the empty-program edge case (returns `["000 END"]`).

**Warning signs:** `error[E0603]: function 'op_display_name' is private` during `cargo check`.

### Pitfall 3: Permissions TOML Must Exist Before First Run

**What goes wrong:** Adding `sst_step` and `bst_step` to `invoke_handler![]` without creating their permission TOML files causes the frontend `invoke('sst_step')` calls to receive a permission-denied error at runtime.

**Why it happens:** Tauri v2.11 capability system requires explicit `allow-<cmd>` permission grants in both a TOML file and `capabilities/default.json`.

**How to avoid:** Create `permissions/sst-step.toml` and `permissions/bst-step.toml` using the exact pattern from `dispatch-op.toml`. Add `"allow-sst-step"` and `"allow-bst-step"` to `capabilities/default.json` permissions array. These TOML files reference `../gen/schemas/desktop-schema.json` which is already generated.

**Warning signs:** Frontend console error `"Command sst_step not allowed by capability"` or similar at runtime.

### Pitfall 4: handleClick Only Routes through dispatch_op

**What goes wrong:** Adding `id: 'sst'` and `id: 'bst'` to `KEY_DEFS` causes those keys to call `handleClick('sst')` and `handleClick('bst')`, which then calls `invoke('dispatch_op', { keyId: 'sst' })`. But `key_map::resolve('sst')` returns `Err(GuiError { message: "unknown key: sst" })`.

**Why it happens:** `handleClick` unconditionally calls `dispatch_op`. SST/BST are not routable through `key_map.rs`.

**How to avoid:** Extend `handleClick` with an `if (keyId === 'sst') invoke('sst_step')` / `if (keyId === 'bst') invoke('bst_step')` branch before the default `dispatch_op` call. Similarly, `handleKey` for F7/F8 must call the dedicated commands, not `dispatch_op`.

**Warning signs:** Frontend console error `"unknown key: sst"` when clicking the SST key.

### Pitfall 5: Linux WebKit Missing in CI

**What goes wrong:** `cargo build --release` of `hp41-gui/src-tauri` fails on Ubuntu runners with missing header files: `webkit2gtk-4.1/webkit2gtk.h: No such file or directory`.

**Why it happens:** The Tauri v2 Rust crate requires system WebKit2GTK development libraries on Linux. GitHub-hosted `ubuntu-latest` runners do not include these by default.

**How to avoid:** Include D-15's `apt-get install` step before the Cargo build step:
```yaml
- name: Install Linux system deps
  if: matrix.os == 'ubuntu-latest'
  run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Warning signs:** CI Linux build fails at the Cargo compilation step with linker or header errors.

### Pitfall 6: TypeScript noUnusedLocals / noUnusedParameters

**What goes wrong:** Adding a `activeStepRef` or a `prgmListingRef` that is conditionally assigned but not consistently used triggers `noUnusedLocals` TypeScript errors caught by `npx tsc --noEmit`.

**Why it happens:** `tsconfig.json` has `"noUnusedLocals": true` and `"noUnusedParameters": true` [VERIFIED].

**How to avoid:** Every variable declared must be used. The active step ref pattern must assign the ref in JSX, not leave it dangling. Verify with `npx tsc --noEmit` from `hp41-gui/` before committing.

---

## Code Examples

### CalcStateView with new fields

```rust
// Source: types.rs from_state() — Phase 18 additions
#[derive(Debug, Serialize)]
pub struct CalcStateView {
    // ... existing fields ...
    pub program_steps: Vec<String>,  // Phase 18 D-01
    pub pc: usize,                   // Phase 18 D-01
}

// In from_state():
let program_steps = prgm_display::format_all_steps(state);
let pc = state.pc;

CalcStateView {
    // ... existing fields ...
    program_steps,
    pc,
}
```

### Capability JSON update

```json
{
  "identifier": "default",
  "description": "Default capability for hp41-gui",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "allow-dispatch-op",
    "allow-get-state",
    "allow-sst-step",
    "allow-bst-step"
  ]
}
```

### Program listing panel JSX

```tsx
// Source: derived from Phase 17 print panel pattern (App.tsx lines 147-160)
{calcState.annunciators.prgm && (
  <div className="prgm-panel">
    <div className="prgm-panel-header">
      PRGM — {calcState.program_steps.length} step{calcState.program_steps.length !== 1 ? 's' : ''}
    </div>
    <div className="prgm-panel-content">
      {calcState.program_steps.map((step, i) => (
        <div
          key={i}
          ref={calcState.pc === i ? activeStepRef : null}
          className={`step-row${calcState.pc === i ? ' step-active' : ''}`}
        >
          {step}
        </div>
      ))}
    </div>
  </div>
)}
```

### ci-gui.yml structure (mirror of ci.yml)

```yaml
name: ci-gui

on:
  push:
    branches: [main, develop]
    paths: ['hp41-gui/**', 'hp41-core/**']
  pull_request:
    branches: [main, develop]
    paths: ['hp41-gui/**', 'hp41-core/**']

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  build:
    name: GUI Build (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'
      - name: Install Linux system deps
        if: matrix.os == 'ubuntu-latest'
        run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
      - name: Install npm dependencies
        run: npm install
        working-directory: hp41-gui
      - name: TypeScript type check
        run: npx tsc --noEmit
        working-directory: hp41-gui
      - name: Rust build (release)
        run: cargo build --release
        working-directory: hp41-gui/src-tauri
```

---

## Precise Integration Points

This section is the planner's primary reference for "what changes where."

### hp41-gui/src-tauri/src/prgm_display.rs — NEW FILE

Copy from `hp41-cli/src/prgm_display.rs` verbatim. Adapt:
1. Add `pub fn format_all_steps(state: &CalcState) -> Vec<String>` (returns `["000 END"]` for empty program; otherwise iterates `state.program.iter().enumerate().map(|(i, op)| format!("{i:03} {}", op_display_name(op)))`).
2. Keep `op_display_name()` private.
3. Remove the `#[cfg(test)]` block from the CLI copy (or keep it — it's valid either way; the tests reference `Op` variants that exist in hp41-core).

Module declaration goes in `lib.rs`: `mod prgm_display;`

### hp41-gui/src-tauri/src/types.rs — MODIFIED

1. Add `use crate::prgm_display;` at the top.
2. Add to `CalcStateView` struct: `pub program_steps: Vec<String>,` and `pub pc: usize,`.
3. Add to `from_state()`: `let program_steps = prgm_display::format_all_steps(state);` and `let pc = state.pc;`.
4. Add these fields to the `CalcStateView { ... }` initializer.
5. Update `test_dispatch_op_payload_size`: add a comment documenting scope (empty program only); verify the assertion still passes after adding the two new fields (with `["000 END"]` the JSON grows by ~35 bytes — may need to relax to ≤350).

### hp41-gui/src-tauri/src/commands.rs — MODIFIED

Add `handle_sst()`, `handle_bst()` helpers and `sst_step`, `bst_step` Tauri command thunks. Pattern is identical to `handle_get_state` / `get_state`.

### hp41-gui/src-tauri/src/lib.rs — MODIFIED

1. Add `mod prgm_display;` (if types.rs uses `crate::prgm_display`, the module must be declared in lib.rs root).
2. Add `commands::sst_step, commands::bst_step` to `invoke_handler![]`.

### hp41-gui/src-tauri/permissions/sst-step.toml — NEW FILE

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-sst-step"
description = "Allows the sst_step command."
commands.allow = ["sst_step"]
```

### hp41-gui/src-tauri/permissions/bst-step.toml — NEW FILE

Same structure with `identifier = "allow-bst-step"` and `commands.allow = ["bst_step"]`.

### hp41-gui/src-tauri/capabilities/default.json — MODIFIED

Add `"allow-sst-step"` and `"allow-bst-step"` to the `permissions` array.

### hp41-gui/src-tauri/tauri.conf.json — MODIFIED

Change `windows[0].height` from `700` to `900`.

### hp41-gui/src/Keyboard.tsx — MODIFIED

Row 3, col 2: change `{ id: '', label: 'SST', row: 3, col: 2 }` to `{ id: 'sst', label: 'SST', row: 3, col: 2 }`.
Row 4, col 8: change `{ id: '', label: 'BST', row: 4, col: 8 }` to `{ id: 'bst', label: 'BST', row: 4, col: 8 }`.

[VERIFIED: Keyboard.tsx KEY_DEFS read directly — SST is at row 3 col 2 with id: ''; BST is at row 4 col 8 with id: '']

### hp41-gui/src/App.tsx — MODIFIED

1. Add `program_steps: string[];` and `pc: number;` to `CalcStateView` TypeScript interface.
2. Add `activeStepRef = useRef<HTMLDivElement>(null)`.
3. Add `useEffect` for auto-scroll: watches `calcState?.pc`, calls `activeStepRef.current?.scrollIntoView(...)`.
4. Extend `resolveKeyId()`: add `if (e.key === 'F7') return 'sst';` and `if (e.key === 'F8') return 'bst';` before the MAP lookup.
5. Extend `handleClick` to route `'sst'` → `invoke('sst_step')` and `'bst'` → `invoke('bst_step')`.
6. In `handleKey`: the existing `handleKey` calls `handleClick(keyId)` implicitly via `resolveKeyId` + `invoke`. Since F7/F8 resolve to 'sst'/'bst' and `handleClick` now routes those to dedicated commands, no separate `handleKey` change is needed beyond `resolveKeyId`.
7. Add program listing panel JSX after `<Keyboard />`, conditional on `calcState.annunciators.prgm`.

[VERIFIED: App.tsx read directly — F7/F8 not in MAP; handleClick calls dispatch_op unconditionally]

### hp41-gui/src/App.css — MODIFIED

Add styles for `.prgm-panel`, `.prgm-panel-header`, `.prgm-panel-content`, `.step-row`, `.step-active`. Reuse the aesthetic of `.print-panel` — same dark background, monospace font, border-top separator.

---

## Validation Architecture

nyquist_validation is enabled (from `.planning/config.json`).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust native tests (`#[test]`), TypeScript `npx tsc --noEmit` |
| Config file | None (Rust inline tests in `mod tests {}` blocks) |
| Quick run command | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Full suite command | `just test && cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml && cd hp41-gui && npx tsc --noEmit` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROG-01 | `handle_sst` advances pc by 1, caps at program.len() | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- handle_sst` | Wave 0 |
| PROG-01 | `handle_bst` decrements pc, saturates at 0 | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- handle_bst` | Wave 0 |
| PROG-01 | `format_all_steps` returns `["000 END"]` for empty program | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- format_all_steps` | Wave 0 |
| PROG-01 | `CalcStateView` contains `program_steps` and `pc` fields | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- test_phase18_fields` | Wave 0 |
| PROG-01 | Program listing appears below keyboard in PRGM mode; SST/BST highlighted step scrolls | manual (SC-1–SC-3) | Human verify | manual |
| SC-4 | GUI CI job passes on all 3 platforms | integration | GitHub Actions `ci-gui.yml` | Wave 1 |

### Wave 0 Gaps

- [ ] `hp41-gui/src-tauri/src/commands.rs` — add test stubs `test_handle_sst_advances_pc`, `test_handle_sst_clamps_at_end`, `test_handle_bst_decrements_pc`, `test_handle_bst_clamps_at_zero`
- [ ] `hp41-gui/src-tauri/src/prgm_display.rs` — add test stub `test_format_all_steps_empty_program`, `test_format_all_steps_nonempty`
- [ ] `hp41-gui/src-tauri/src/types.rs` — add test stub `test_phase18_fields_exist` (verifies `program_steps` and `pc` are in `CalcStateView`)

### Sampling Rate

- **Per task commit:** `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Per wave merge:** Full suite + TypeScript check
- **Phase gate:** All Rust tests green + TypeScript clean + human SC-1..SC-5 verification before `/gsd-verify-work`

---

## Security Domain

Security enforcement is enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | yes | Tauri capability system — only `allow-sst-step` / `allow-bst-step` permissions needed; no additional capabilities |
| V5 Input Validation | yes | `handle_sst` / `handle_bst` take no user input; pc arithmetic uses Rust's `saturating_sub` and bounds check — no overflow possible |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unauthorized SST/BST command invocation | Elevation of Privilege | Tauri capability permission grant in `capabilities/default.json` restricts these commands to the `main` window |
| pc out-of-bounds write | Tampering | `handle_sst` uses `if calc.pc < calc.program.len()` (no underflow, no overflow); `handle_bst` uses `saturating_sub(1)` |

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | Cargo build | Yes | 1.89.0 | — |
| Node.js | npm install, tsc | Yes | 22.16.0 | — |
| npm | hp41-gui npm packages | Yes | 10.9.2 | — |
| Tauri CLI (`@tauri-apps/cli`) | `npm run tauri dev` | Yes (in devDeps) | 2.11 | — |
| GitHub Actions ubuntu/macos/windows runners | `ci-gui.yml` | Yes (GitHub-hosted) | — | — |
| WebKit2GTK system libs | Linux Rust build | Not locally | — | CI installs via apt-get (D-15) |

---

## Open Questions

1. **300-byte CalcStateView assertion — exact relaxation needed**
   - What we know: Adding `program_steps: ["000 END"]` and `pc: 0` adds ~35 bytes to the empty-program JSON. Current assertion is ≤300 bytes.
   - What's unclear: Whether the current serialized size is ≤265 bytes (leaving room) or already at ~285 bytes (requiring relaxation).
   - Recommendation: The implementer should run the test after adding the fields and either verify it passes or relax to ≤350 bytes. If the assertion is changed, update the comment to document the scope.

2. **handleKey for F7/F8 vs the existing handleClick routing**
   - What we know: `handleKey` calls `resolveKeyId()` to get a `keyId` string, then calls `invoke('dispatch_op', { keyId })` directly (not via `handleClick`). So extending `handleClick` for sst/bst does NOT automatically cover keyboard F7/F8.
   - What's unclear: Phase 18 D-07 says "add F7/F8 to resolveKeyId". If `handleKey` continues to call `dispatch_op` directly after resolveKeyId, F7 would still hit `dispatch_op` with key_id `'sst'` and fail.
   - Recommendation: Either (a) extract the invoke routing from `handleKey` into `handleClick` so the two share the same SST/BST detection, OR (b) add a parallel F7/F8 check in `handleKey` that calls `invoke('sst_step')` directly before the `dispatch_op` call. Option (a) is cleaner.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | GitHub Actions `ubuntu-latest` runner does not pre-install `libwebkit2gtk-4.1-dev` | Common Pitfalls (Pitfall 5) | If it does ship these, the install step is harmless but unnecessary |
| A2 | `actions/setup-node@v4` with `node-version: 'lts/*'` is compatible with the project's npm 10.9.2 requirement | Code Examples (ci-gui.yml) | If not, pinning to a specific Node LTS version is trivial |

**All other claims in this document are VERIFIED against live source files read in this session.**

---

## Sources

### Primary (HIGH confidence)

- `hp41-cli/src/prgm_display.rs` — read directly; op_display_name coverage, format_step signature, imports confirmed
- `hp41-gui/src-tauri/src/types.rs` — read directly; CalcStateView struct shape, from_state() body, 300-byte test confirmed
- `hp41-gui/src-tauri/src/commands.rs` — read directly; handle_op/handle_get_state patterns, Mutex lock pattern confirmed
- `hp41-gui/src-tauri/src/lib.rs` — read directly; invoke_handler registration, AppState alias confirmed
- `hp41-gui/src-tauri/src/key_map.rs` — read directly; no sst/bst entries confirmed
- `hp41-gui/src/App.tsx` — read directly; handleClick, resolveKeyId, F7/F8 absence, handleKey confirmed
- `hp41-gui/src/Keyboard.tsx` — read directly; KEY_DEFS positions for SST (row 3 col 2, id: '') and BST (row 4 col 8, id: '') confirmed
- `hp41-gui/src/App.css` — read directly; print-panel CSS patterns confirmed for reuse
- `hp41-gui/src-tauri/permissions/dispatch-op.toml` — read directly; permission TOML template confirmed
- `hp41-gui/src-tauri/permissions/get-state.toml` — read directly; confirmed
- `hp41-gui/src-tauri/capabilities/default.json` — read directly; permissions array structure confirmed
- `hp41-gui/src-tauri/tauri.conf.json` — read directly; height: 700 confirmed
- `.github/workflows/ci.yml` — read directly; Swatinem/rust-cache@v2, dtolnay/rust-toolchain@stable, fail-fast: false matrix confirmed
- `hp41-gui/src-tauri/Cargo.toml` — read directly; tauri 2.11, hp41-core path dep confirmed
- `hp41-gui/package.json` — read directly; @tauri-apps/api 2.11, TypeScript 6.0, React 19.2 confirmed
- `hp41-gui/tsconfig.json` — read directly; noUnusedLocals, noUnusedParameters: true confirmed
- `.planning/config.json` — read directly; nyquist_validation: true confirmed
- `hp41-core/src/state.rs` — read directly; `pub program: Vec<Op>`, `pub pc: usize`, `pub prgm_mode: bool` confirmed
- `.planning/STATE.md` — read directly; Tauri v2.11 permission TOML requirement, poisoned-lock pattern confirmed

### Secondary (MEDIUM confidence)

None — all findings verified against source files.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified from Cargo.toml / package.json
- Architecture: HIGH — all integration points verified from live source files
- Pitfalls: HIGH — each pitfall traced to a specific source-verified fact
- CI patterns: HIGH — ci.yml read directly; WebKit apt deps are project-established pattern

**Research date:** 2026-05-10
**Valid until:** 2026-06-10 (stable codebase; Tauri version pinned)
