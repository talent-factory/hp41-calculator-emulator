# Phase 18: Program Listing & CI/CD - Pattern Map

**Mapped:** 2026-05-10
**Files analyzed:** 11 new/modified files
**Analogs found:** 11 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/src/prgm_display.rs` | utility | transform | `hp41-cli/src/prgm_display.rs` | exact copy |
| `hp41-gui/src-tauri/src/types.rs` | model/DTO | request-response | `hp41-gui/src-tauri/src/types.rs` (self, prior phase) | self-extension |
| `hp41-gui/src-tauri/src/commands.rs` | service | request-response | `hp41-gui/src-tauri/src/commands.rs` `handle_get_state` | exact role-match |
| `hp41-gui/src-tauri/src/lib.rs` | config | request-response | `hp41-gui/src-tauri/src/lib.rs` (self, prior phase) | self-extension |
| `hp41-gui/src-tauri/permissions/sst-step.toml` | config | — | `hp41-gui/src-tauri/permissions/dispatch-op.toml` | exact copy |
| `hp41-gui/src-tauri/permissions/bst-step.toml` | config | — | `hp41-gui/src-tauri/permissions/get-state.toml` | exact copy |
| `hp41-gui/src-tauri/capabilities/default.json` | config | — | `hp41-gui/src-tauri/capabilities/default.json` (self) | self-extension |
| `hp41-gui/src-tauri/tauri.conf.json` | config | — | `hp41-gui/src-tauri/tauri.conf.json` (self) | self-extension |
| `hp41-gui/src/App.tsx` | component | event-driven | `hp41-gui/src/App.tsx` (self — print panel, handleClick) | self-extension |
| `hp41-gui/src/Keyboard.tsx` | component | event-driven | `hp41-gui/src/Keyboard.tsx` (self — KEY_DEFS entries) | self-extension |
| `hp41-gui/src/App.css` | config | — | `hp41-gui/src/App.css` `.print-panel` block (lines 89-139) | exact role-match |
| `.github/workflows/ci-gui.yml` | config | — | `.github/workflows/ci.yml` matrix strategy | role-match |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/prgm_display.rs` (utility, transform) — NEW FILE

**Analog:** `hp41-cli/src/prgm_display.rs` — copy verbatim, then add one public function.

**Imports pattern** (lines 7-8 of analog):
```rust
use hp41_core::ops::{Op, StackReg, StoArithKind};
use hp41_core::CalcState;
```
These imports compile identically in `hp41-gui/src-tauri` because `hp41-core` is already a path dependency. No adaptation needed.

**Core pattern — `op_display_name` private function** (lines 27-139 of analog):
```rust
fn op_display_name(op: &Op) -> String {
    match op {
        Op::Add => "+ ".to_string(),
        Op::Sub => "- ".to_string(),
        // ... all 35+ Op variants exhaustively covered ...
        Op::SyntheticByte(b) => format!("SYN {:02X}", b),
    }
}
```
Keep `op_display_name` private — only the new `format_all_steps` crosses the module boundary.

**New public function to add** (does NOT exist in analog — add to GUI copy only):
```rust
/// Format all program steps as pre-rendered strings for the React frontend.
/// Returns ["000 END"] for empty program (always at least one row).
/// Index 0 = step 000; index N = step N.
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

**Anti-pattern warning:** Do NOT call `format_step(state)` in a loop — it reads `state.pc` (current pc), not the loop index, so it returns the same step N times. Use `format_all_steps` above instead.

**Test pattern** (lines 141-161 of analog, keep or adapt):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_all_steps_empty_program() {
        let state = hp41_core::CalcState::new();
        let steps = format_all_steps(&state);
        assert_eq!(steps, vec!["000 END"]);
    }

    #[test]
    fn test_format_all_steps_nonempty() {
        use hp41_core::ops::Op;
        let mut state = hp41_core::CalcState::new();
        state.program = vec![Op::Add, Op::Enter];
        let steps = format_all_steps(&state);
        assert_eq!(steps[0], "000 + ");
        assert_eq!(steps[1], "001 ENTER");
    }
}
```

---

### `hp41-gui/src-tauri/src/types.rs` (model/DTO, request-response) — MODIFIED

**Analog:** `hp41-gui/src-tauri/src/types.rs` (self — existing Phase 14/15 fields).

**Struct extension pattern** — follow how Phase 15 added Y/Z/T/LASTX fields (lines 29-34):
```rust
// Existing fields (lines 25-35) — do NOT remove
#[derive(Debug, Serialize)]
pub struct CalcStateView {
    pub display_str: String,
    pub x_str: String,
    pub y_str: String,
    pub z_str: String,
    pub t_str: String,
    pub lastx_str: String,
    pub in_eex_mode: bool,
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>,
    // Phase 18 additions — append after existing fields:
    pub program_steps: Vec<String>,
    pub pc: usize,
}
```

**Import to add** at top of file (after existing `use hp41_core::...` line):
```rust
use crate::prgm_display;
```

**`from_state()` extension pattern** — follow how Phase 15 added computed fields (lines 57-66). Add before the final `CalcStateView { ... }` initializer:
```rust
// Phase 18 D-01/D-02: program listing for PRGM panel
let program_steps = prgm_display::format_all_steps(state);
let pc = state.pc;
```

Then add to the `CalcStateView { ... }` initializer block (lines 76-87), after `print_lines`:
```rust
CalcStateView {
    display_str,
    x_str,
    y_str,
    z_str,
    t_str,
    lastx_str,
    in_eex_mode,
    annunciators,
    print_lines,
    program_steps,  // Phase 18
    pc,             // Phase 18
}
```

**Test pattern — payload size assertion** (lines 112-123). After adding `program_steps` and `pc`, verify the test still passes. With an empty `CalcState::new()`, `program_steps = ["000 END"]` adds ~35 bytes. If the assertion at `≤ 300` fails, relax to `≤ 350` and update the comment:
```rust
#[test]
fn test_dispatch_op_payload_size() {
    // SC-1: CalcStateView JSON for CalcState::new() (empty program — program_steps = ["000 END"]).
    // This assertion covers the empty-program baseline only; real programs grow program_steps.
    let state = CalcState::new();
    let view = CalcStateView::from_state(&state, vec![]);
    let json = serde_json::to_string(&view).unwrap();
    assert!(
        json.len() <= 350,
        "CalcStateView JSON (empty program) must be ≤350 bytes, got {} bytes: {}",
        json.len(),
        json
    );
}
```

**New test stub for Phase 18 fields** (follow `test_phase15_stack_fields_exist` pattern, lines 154-168):
```rust
#[test]
fn test_phase18_fields_exist() {
    // Wave 0 RED: CalcStateView must have program_steps and pc after Phase 18 update.
    let state = CalcState::new();
    let view = CalcStateView::from_state(&state, vec![]);
    assert_eq!(view.program_steps, vec!["000 END"], "empty program → [\"000 END\"]");
    assert_eq!(view.pc, 0, "fresh state pc must be 0");
}
```

---

### `hp41-gui/src-tauri/src/commands.rs` (service, request-response) — MODIFIED

**Analog:** `hp41-gui/src-tauri/src/commands.rs` — `handle_get_state` / `get_state` pair (lines 39-46 and 132-135).

**Tauri command thunk pattern** (copy of `get_state` at lines 40-46):
```rust
/// Tauri command: step the program counter forward by 1 (SST).
/// Locks AppState (with poisoned-lock recovery) and delegates to `handle_sst`.
#[tauri::command]
pub fn sst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_sst(&mut calc)
}

/// Tauri command: step the program counter backward by 1 (BST).
#[tauri::command]
pub fn bst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_bst(&mut calc)
}
```

**Pure-Rust helper pattern** (copy of `handle_get_state` at lines 132-135):
```rust
/// Pure-Rust helper for sst_step — unit-testable without Tauri runtime.
/// Advances pc by 1, capped at program.len() (no wrap-around — matches HP-41 hardware).
pub fn handle_sst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    if calc.pc < calc.program.len() {
        calc.pc += 1;
    }
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}

/// Pure-Rust helper for bst_step — decrements pc, saturates at 0.
pub fn handle_bst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.pc = calc.pc.saturating_sub(1);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}
```

**Test pattern** (follow `test_dispatch_op_unknown_key` and `test_print_buffer_drained` at lines 144-185):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    // ... existing tests stay unchanged ...

    #[test]
    fn test_handle_sst_advances_pc() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add, Op::Enter];
        calc.pc = 0;
        handle_sst(&mut calc).unwrap();
        assert_eq!(calc.pc, 1);
    }

    #[test]
    fn test_handle_sst_clamps_at_end() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add];
        calc.pc = 1; // already at end
        handle_sst(&mut calc).unwrap();
        assert_eq!(calc.pc, 1, "SST must not advance past program.len()");
    }

    #[test]
    fn test_handle_bst_decrements_pc() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add];
        calc.pc = 1;
        handle_bst(&mut calc).unwrap();
        assert_eq!(calc.pc, 0);
    }

    #[test]
    fn test_handle_bst_clamps_at_zero() {
        let mut calc = CalcState::new();
        calc.pc = 0;
        handle_bst(&mut calc).unwrap();
        assert_eq!(calc.pc, 0, "BST must not underflow below 0");
    }
}
```

---

### `hp41-gui/src-tauri/src/lib.rs` (config, request-response) — MODIFIED

**Analog:** `hp41-gui/src-tauri/src/lib.rs` (self — existing module declarations and `invoke_handler`).

**Module declaration pattern** (lines 6-9 — add `prgm_display` after existing modules):
```rust
mod commands;
mod key_map;
mod persistence;
mod prgm_display;  // Phase 18 — added
mod types;
```

**`invoke_handler` extension pattern** (lines 43-46 — add two new commands):
```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    commands::sst_step,   // Phase 18
    commands::bst_step,   // Phase 18
])
```

No other changes to `lib.rs`.

---

### `hp41-gui/src-tauri/permissions/sst-step.toml` (config) — NEW FILE

**Analog:** `hp41-gui/src-tauri/permissions/dispatch-op.toml` (lines 1-6) — exact structural copy, change identifiers only.

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-sst-step"
description = "Allows the sst_step command."
commands.allow = ["sst_step"]
```

---

### `hp41-gui/src-tauri/permissions/bst-step.toml` (config) — NEW FILE

**Analog:** `hp41-gui/src-tauri/permissions/get-state.toml` (lines 1-6) — exact structural copy, change identifiers only.

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-bst-step"
description = "Allows the bst_step command."
commands.allow = ["bst_step"]
```

---

### `hp41-gui/src-tauri/capabilities/default.json` (config) — MODIFIED

**Analog:** `hp41-gui/src-tauri/capabilities/default.json` (self — lines 1-10). Add two new permission identifiers to the `permissions` array:

```json
{
  "identifier": "default",
  "description": "Default capability for hp41-gui — core + Phase 14 IPC commands",
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

---

### `hp41-gui/src-tauri/tauri.conf.json` (config) — MODIFIED

**Analog:** `hp41-gui/src-tauri/tauri.conf.json` (self — line 17). Change `height` from `700` to `900` in `app.windows[0]`:

```json
"windows": [
  {
    "title": "HP-41 Calculator",
    "width": 400,
    "height": 900,
    "resizable": false,
    "decorations": true
  }
]
```

All other fields stay unchanged.

---

### `hp41-gui/src/App.tsx` (component, event-driven) — MODIFIED

**Analog:** `hp41-gui/src/App.tsx` (self — print panel pattern lines 98-110, 147-160; `handleClick` lines 82-89; `resolveKeyId` lines 26-51).

**TypeScript interface extension** — add to `CalcStateView` interface (after `print_lines: string[]` on line 24):
```typescript
interface CalcStateView {
  display_str: string;
  x_str: string;
  y_str: string;
  z_str: string;
  t_str: string;
  lastx_str: string;
  in_eex_mode: boolean;
  annunciators: Annunciators;
  print_lines: string[];
  program_steps: string[];  // Phase 18 D-01
  pc: number;               // Phase 18 D-01
}
```

**New `useRef` for auto-scroll** — add alongside existing `printEndRef` (line 58):
```typescript
const activeStepRef = useRef<HTMLDivElement>(null);
```

**Auto-scroll `useEffect` pattern** — follow the `printEndRef` scroll pattern (lines 107-110), add after it:
```typescript
// Auto-scroll active program step into view when pc changes (D-09)
useEffect(() => {
  activeStepRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}, [calcState?.pc]);
```

**`resolveKeyId` extension** — add F7/F8 before the `MAP` lookup (after line 34, before `const MAP`):
```typescript
// Phase 18 D-07: F7/F8 → SST/BST keyboard bindings
if (e.key === 'F7') return 'sst';
if (e.key === 'F8') return 'bst';
```

**`handleClick` extension** — add SST/BST routing before the default `dispatch_op` call. Replace existing `handleClick` (lines 82-89) with:
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

**Note on `handleKey`:** The existing `handleKey` (lines 68-80) calls `invoke('dispatch_op', { keyId })` directly after `resolveKeyId`. Since F7/F8 now resolve to `'sst'`/`'bst'`, `handleKey` must also be updated to route through `handleClick` instead of calling `dispatch_op` directly. Replace the `invoke` call inside `handleKey` with `handleClick(keyId)`:
```typescript
const handleKey = useCallback((e: KeyboardEvent) => {
  if (e.repeat) return;
  if (busyRef.current) return;
  const keyId = resolveKeyId(e, calcState);
  if (keyId === null) return;
  e.preventDefault();
  handleClick(keyId);  // delegates to handleClick which routes sst/bst correctly
}, [calcState, handleClick]);
```

**Program listing panel JSX** — add after `<Keyboard onKey={handleClick} busyRef={busyRef} />` (after line 146), following the print panel pattern (lines 147-160):
```tsx
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

**TypeScript strictness warning:** `tsconfig.json` has `noUnusedLocals: true`. `activeStepRef` must be referenced in JSX (via the `ref={...}` prop above) or the TypeScript check will fail. The conditional `ref` assignment ensures it is always syntactically used.

---

### `hp41-gui/src/Keyboard.tsx` (component, event-driven) — MODIFIED

**Analog:** `hp41-gui/src/Keyboard.tsx` `KEY_DEFS` array (lines 18-72) — change `id: ''` to real IDs at two positions.

**SST key** — row 3, col 2 (line 54 of analog):
```typescript
// Before:
{ id: '',     label: 'SST',  row: 3, col: 2 },
// After:
{ id: 'sst',  label: 'SST',  row: 3, col: 2 },
```

**BST key** — row 4, col 8 (line 71 of analog):
```typescript
// Before:
{ id: '',      label: 'BST',  row: 4, col: 8 },
// After:
{ id: 'bst',   label: 'BST',  row: 4, col: 8 },
```

No other changes to `Keyboard.tsx`. The `handleKeyClick` guard `if (!keyId) return;` (line 97) already handles empty IDs, so adding real IDs makes these keys clickable automatically.

---

### `hp41-gui/src/App.css` (config) — MODIFIED

**Analog:** `.print-panel` block in `hp41-gui/src/App.css` (lines 89-139) — reuse the same dark aesthetic, monospace font, border-top separator pattern.

**Styles to append** (after the existing `.print-line` block at line 139):
```css
/* ── Program Listing Panel (Phase 18: D-08, D-09, D-11) ───────────────── */

.prgm-panel {
  width: 100%;
  background: #1a1a1a;
  border-top: 1px solid #3a3a3a;
  border-radius: 0 0 8px 8px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 11px;
  overflow: hidden;
}

.prgm-panel-header {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  background: #252525;
  border-bottom: 1px solid #3a3a3a;
  color: #888;
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.prgm-panel-content {
  max-height: 160px;
  overflow-y: auto;
  padding: 4px 8px;
}

.step-row {
  color: #c8c8c8;
  padding: 1px 0;
  white-space: pre;
  line-height: 1.4;
}

.step-active {
  background: #2a3a2a;
  color: #c8e6c9;
  border-radius: 2px;
  padding-left: 4px;
}
```

Note: `prgm-panel-content` uses `max-height: 160px` with `overflow-y: auto` (D-09), unlike `print-panel-content` which uses a fixed `height: 130px`. This allows the panel to be shorter when the program has few steps.

---

### `.github/workflows/ci-gui.yml` (config) — NEW FILE

**Analog:** `.github/workflows/ci.yml` — copy the matrix strategy (lines 30-47), `env` block (lines 9-11), checkout+toolchain+cache steps, and `fail-fast: false` pattern. Key differences: add path filter, add Node setup, add Linux WebKit deps, use `cargo build --release` in `hp41-gui/src-tauri`, no `just` task runner.

**Full file pattern** (derived from `ci.yml` `test` job structure):
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

**Key differences from `ci.yml`:**
- Path filter on `push` and `pull_request` (D-13) — `ci.yml` has no path filter
- `actions/setup-node@v4` step (not present in `ci.yml`)
- Linux `apt-get` step (not present in `ci.yml` — CLI has no WebKit dependency)
- `working-directory: hp41-gui/src-tauri` for Cargo (not `just build-release`)
- No `just` task runner required — `cargo build --release` directly
- No lint, coverage, or MSRV jobs — only the build job (SC-4 scope)

---

## Shared Patterns

### Mutex Lock with Poisoned-Lock Recovery

**Source:** `hp41-gui/src-tauri/src/commands.rs` line 35
**Apply to:** All new Tauri command thunks (`sst_step`, `bst_step`)

```rust
let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
```

Required by `#![deny(clippy::unwrap_used)]` in `lib.rs` line 1. Never use `.unwrap()` on a Mutex lock.

### Print Buffer Drain

**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 76-77, 124-125, 133
**Apply to:** `handle_sst`, `handle_bst`

```rust
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
Ok(CalcStateView::from_state(calc, print_lines))
```

The drain must happen before calling `from_state` because `from_state` takes `&CalcState` (immutable borrow) after the mutable drain.

### `busyRef` Debounce Guard

**Source:** `hp41-gui/src/App.tsx` lines 83, 71
**Apply to:** `handleClick` (already present; extended with SST/BST routing)

```typescript
if (busyRef.current) return;
busyRef.current = true;
// ... invoke ...
.finally(() => { busyRef.current = false; });
```

SST/BST clicks follow the same guard — the `handleClick` extension preserves this pattern.

### Tauri Permission TOML Schema Reference

**Source:** `hp41-gui/src-tauri/permissions/dispatch-op.toml` line 1
**Apply to:** `sst-step.toml`, `bst-step.toml`

```toml
"$schema" = "../gen/schemas/desktop-schema.json"
```

The `../gen/schemas/desktop-schema.json` path is relative to the `permissions/` directory and references already-generated Tauri schema. Do not change this path.

### `#[allow(clippy::unwrap_used)]` in Test Modules

**Source:** `hp41-gui/src-tauri/src/commands.rs` line 138
**Apply to:** All new `#[cfg(test)]` blocks in `commands.rs` and `prgm_display.rs`

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    // ...
}
```

### Vanilla CSS Only (No Inline Styles or CSS Modules)

**Source:** Phase 15 D-10; `hp41-gui/src/App.css` throughout
**Apply to:** `App.css` additions for `.prgm-panel`, `.step-row`, `.step-active`

All styling goes in `App.css`. Use `className` strings in JSX. No `style={{}}` inline props, no CSS-in-JS.

---

## No Analog Found

All 11 files have clear analogs. No file requires RESEARCH.md patterns as a fallback.

---

## Metadata

**Analog search scope:** `hp41-gui/src-tauri/src/`, `hp41-gui/src/`, `hp41-cli/src/`, `.github/workflows/`, `hp41-gui/src-tauri/permissions/`, `hp41-gui/src-tauri/capabilities/`
**Files scanned:** 14 source files read directly
**Pattern extraction date:** 2026-05-10
