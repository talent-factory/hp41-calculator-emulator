# Phase 31: GUI Integration — Pattern Map

**Mapped:** 2026-05-17
**Files analyzed:** 23 (new + modified across 5 plans)
**Analogs found:** 23 / 23 (every concrete file change has a verified in-tree analog)

> Phase 31 is **wiring, not authoring** (per 31-RESEARCH.md §400-411). Every behavior already has a `hp41-core`/`hp41-cli`/`hp41-gui` reference; this PATTERNS.md tells the planner *exactly which file to copy from* for each new/modified file.

---

## File Classification

### Plan 31-01 — `prgm_display.rs` verification + SC-4 grep

| File | Status | Role | Data Flow | Closest Analog | Match Quality |
|------|--------|------|-----------|-----------------|----------------|
| `hp41-gui/src-tauri/src/prgm_display.rs` | VERIFY-ONLY (already shipped Phase 28) | display formatter (SC-4 documented exception) | transform | self (lines 259-313 — Phase 28 arms already present) | exact |
| `hp41-gui/src-tauri/tests/sc4_invariant.rs` | NEW | test | request-response | `hp41-gui/src-tauri/tests/` (any existing integration test as TOML+structure analog) — also CLAUDE.md "SC-4 invariant" grep doc | role-match (new shape) |

### Plan 31-02 — Cancellation channel

| File | Status | Role | Data Flow | Closest Analog | Match Quality |
|------|--------|------|-----------|-----------------|----------------|
| `hp41-gui/src-tauri/src/commands.rs` (+`request_cancel`) | MODIFY | controller / Tauri thunk | request-response | `hp41-gui/src-tauri/src/commands.rs::run_stop` lines 250-260 + `handle_run_stop` 281-288 | exact |
| `hp41-gui/src-tauri/src/lib.rs` | MODIFY (registry + `CancelFlag` State + setup() Arc::clone) | config / app bootstrap | event-driven | `hp41-gui/src-tauri/src/lib.rs::run()` lines 16-69 (existing `app.manage` + `generate_handler!` lines) | exact |
| `hp41-gui/src-tauri/permissions/request-cancel.toml` | NEW | config / permission | request-response | `hp41-gui/src-tauri/permissions/run-stop.toml` | exact |
| `hp41-gui/src-tauri/capabilities/default.json` | MODIFY (+ `"allow-request-cancel"`) | config | n/a | self (current 6-permission list) | exact |
| `scripts/check-tauri-permissions.sh` | NEW (per Open Q1 — script doesn't exist) | utility / CI gate | batch | none in tree — RESEARCH §Open Q1 sketches the shape | no analog (author from sketch) |
| `hp41-gui/src-tauri/tests/cancel_request_integ.rs` | NEW | test (integration) | event-driven | `hp41-gui/src-tauri/src/commands.rs` `tests` module (lines 290-563) for in-crate Mutex-based test patterns | role-match |

### Plan 31-03 — XEQ-by-name parity + 3 modal Tauri thunks + CalcStateView projection

| File | Status | Role | Data Flow | Closest Analog | Match Quality |
|------|--------|------|-----------|-----------------|----------------|
| `hp41-gui/src-tauri/src/commands.rs` (+`submit_modal`, +`cancel_modal`, +`submit_modal_with_label`) | MODIFY | controller / Tauri thunk | request-response | `hp41-gui/src-tauri/src/commands.rs::run_stop` (no-arg) + `dispatch_op` (with `key_id: &str` arg) for the label-taking thunk | exact |
| `hp41-gui/src-tauri/permissions/submit-modal.toml` | NEW | config / permission | request-response | `hp41-gui/src-tauri/permissions/run-stop.toml` | exact |
| `hp41-gui/src-tauri/permissions/cancel-modal.toml` | NEW | config / permission | request-response | `hp41-gui/src-tauri/permissions/run-stop.toml` | exact |
| `hp41-gui/src-tauri/permissions/submit-modal-with-label.toml` | NEW | config / permission | request-response | `hp41-gui/src-tauri/permissions/dispatch-op.toml` (has a label-taking analog if `submit_modal_with_label` takes `label: &str`) — else `run-stop.toml` | role-match |
| `hp41-gui/src-tauri/src/types.rs` (extend `CalcStateView`) | MODIFY (+4 fields: `is_running`, `modal_program_active`, `modal_requires_alpha_label`, `modal_prompt`) | model / DTO | transform | `hp41-gui/src-tauri/src/types.rs` lines 25-57 (existing CalcStateView fields) + `from_state` lines 66-145 | exact |
| `hp41-gui/src/App.tsx` (`CalcStateView` TS interface mirror) | MODIFY | model / DTO TS-mirror | transform | `hp41-gui/src/App.tsx` lines 23-40 (existing TS mirror) | exact |
| `hp41-gui/src-tauri/tests/d25_6_parity.rs` | NEW | test (integration) | transform | 31-RESEARCH.md §"Code Examples / Example 9" lines 781-810 (full body provided) | exact (template ready) |
| `hp41-gui/src-tauri/tests/calc_state_view_payload_size.rs` (or extend `types.rs::tests`) | MODIFY | test | transform | `hp41-gui/src-tauri/src/types.rs::tests::test_dispatch_op_payload_size` lines 169-208 | exact |

### Plan 31-04 — `?`-overlay parallel-load + `Op::Catalog(2)` surgical extension

| File | Status | Role | Data Flow | Closest Analog | Match Quality |
|------|--------|------|-----------|-----------------|----------------|
| `hp41-gui/src/help_data.ts` | MODIFY (+ `import math1Json` + `helpEntriesMath1` + `helpEntriesAll` + extend `HelpEntry.xrom?`) | model / data accessor | transform | `hp41-cli/src/help_data.rs` lines 100-135 (Phase 29 D-29.2 parallel-load + merged accessor — same shape in TypeScript) + self lines 13, 19-44 | exact (Rust parallel) |
| `hp41-gui/src/HelpOverlay.tsx` | MODIFY (two-section wrapper around existing per-category rendering) | component | transform | self lines 22-98 (existing one-section structure — wrap inside two-section grouping by `entry.xrom?.module === 'Math 1'` discriminator) | exact (extend) |
| `hp41-gui/src/App.css` (+`.help-overlay-section-heading`, `.help-overlay-section-body`) | MODIFY | config (style) | n/a | self lines 325-335 (existing `.help-overlay-category-heading` selector — copy/step-up 13px→14px + add bold) | exact |
| `hp41-core/src/ops/program.rs::op_catalog` (n==2 arm) | MODIFY (SURGICAL `hp41-core` exception — Plan 25-03 precedent) | service / catalog enumeration | streaming (push-to-buffer) | self lines 300-345 (`op_catalog` body; replace lines 335-339 `2..=4 NOT AVAILABLE` stub with XROM enumeration loop) + `hp41-core/src/ops/math1/xrom.rs::MATH_1` lines 43-46 (iteration source) | exact (surgical extension pattern from v2.2 Plan 25-03) |
| `hp41-core/tests/op_catalog_xrom.rs` (or extend an existing tests file) | NEW (test) | test | streaming | `hp41-core/src/ops/program.rs::tests` arms already existing for CAT 1 | role-match |

### Plan 31-05 — R/S 3-way + Esc cascade + LCD-alternation + post-dispatch auto-open + `≡` glyph + `pending_input.ts` extension

| File | Status | Role | Data Flow | Closest Analog | Match Quality |
|------|--------|------|-----------|-----------------|----------------|
| `hp41-gui/src-tauri/src/commands.rs::handle_get_state` (LCD-alternation routing) | MODIFY | controller / state projection | transform | self lines 228-232 (`handle_get_state` body — extend with prompt-routing block BEFORE the drain) + `types.rs::from_state` lines 75-81 (existing display_str priority chain) | exact (parallel branch) |
| `hp41-gui/src/App.tsx` (R/S 3-way, Esc cascade, post-dispatch auto-open `useEffect`, `invokeForKey` extension) | MODIFY | component / event handler | event-driven | `hp41-cli/src/app.rs` lines 660-698 (R/S submit + Esc cancel CLI ground truth) + `hp41-cli/src/app.rs::maybe_auto_open_collect_for_modal` lines 1782-1795 (post-dispatch auto-open CLI ground truth) + self lines 62-67 (`invokeForKey` v2.1 shape) + self lines 414-486 (handleKey Esc precedence) | exact (CLI mirror) |
| `hp41-gui/src/pending_input.ts` (extend `xeq_name` with `mode?: 'normal' \| 'collect-for-modal'`; switch-case for CollectForModal Enter arm) | MODIFY | model / state machine | event-driven | `hp41-cli/src/app.rs::XeqByNameMode` lines 44-50 + `PendingInput::XeqByName { acc, mode }` lines 134-138 (Rust source) + self lines 58 (existing variant) and lines 335-363 (existing xeq_name switch arm) | exact (Rust↔TS port) |
| `hp41-gui/src/Display14Seg.tsx` (+ `'\u{2261}': [0, 6, 7, 3]` SEGMENT_MAP entry) | MODIFY | component / glyph table | transform | self lines 131-184 (SEGMENT_MAP; insert one new entry near `'='` at line 177 since `≡` is a triple-bar variant of `=`) | exact (one-line table entry) |
| `hp41-gui/src/Display14Seg.test.tsx` (+ render-snapshot test for `≡`) | MODIFY (test) | test | transform | self (existing Display14Seg snapshot tests — add a new case asserting `"FUNCTION NAM\u{2261}"` renders) | exact |
| `hp41-gui/src-tauri/src/types.rs::From<HpError> for GuiError` (CANCELED uppercase override) | MODIFY | model / error mapping | transform | self lines 153-161 (existing From impl — extend with match arm per RESEARCH Pitfall 4 §504-516) | exact |

---

## Pattern Assignments

### Plan 31-02 — `hp41-gui/src-tauri/src/commands.rs` (controller, request-response)

**Analog:** `hp41-gui/src-tauri/src/commands.rs::run_stop` (v2.1) + `handle_run_stop`

**Tauri command thunk pattern** (lines 250-260):

```rust
/// Tauri command: toggle program run/stop state (R/S key).
///
/// Mirrors sst_step/bst_step shape — never goes through dispatch_op because
/// R/S is not a single Op variant (it toggles CalcState.is_running).
/// Toggles the flag only; no run loop is spawned here (the IPC thread cannot
/// block on a run loop, so actual stepping requires a separate tick thread).
#[tauri::command]
pub fn run_stop(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_run_stop(&mut calc)
}
```

**Helper-function pattern** (lines 283-288):

```rust
/// Pure-Rust helper for run_stop — toggles `is_running`. Unit-testable
/// without a Tauri runtime. Flag-toggle only; no run loop spawned here.
pub fn handle_run_stop(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.is_running = !calc.is_running;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

**Phase 31 deviation (Pitfall 1):** `request_cancel` MUST NOT take `State<'_, AppState>` — it would deadlock waiting for the Mutex held by `dispatch_op(Op::Integ)`. Use a SECOND `tauri::State<'_, CancelFlag>` (a separately-managed `Arc<AtomicBool>` cloned at setup time). See 31-RESEARCH.md §Pitfall 1 lines 432-456:

```rust
// hp41-gui/src-tauri/src/lib.rs (Plan 31-02 setup() extension)
use std::sync::{Arc, atomic::AtomicBool};
pub type CancelFlag = Arc<AtomicBool>;

// In setup():
let app_state = Mutex::new(initial_state);
let cancel_flag = Arc::clone(&app_state.lock().unwrap_or_else(|e| e.into_inner()).cancel_requested);
app.manage(app_state);
app.manage(cancel_flag);  // second managed state — lock-free reachable
```

```rust
// hp41-gui/src-tauri/src/commands.rs (Plan 31-02 new thunk)
#[tauri::command]
pub fn request_cancel(cancel: State<'_, CancelFlag>) -> Result<(), GuiError> {
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
```

**Auto-save thread lock-release precedent** (`lib.rs` lines 42-56) confirms the pattern is established:

```rust
let handle = app.handle().clone();
std::thread::spawn(move || {
    let thread_save_path = persistence::default_state_path();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        let state = handle.state::<AppState>();
        let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();  // guard dropped here
        if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
            eprintln!("auto-save failed: {e}");
        }
    }
});
```

---

### Plan 31-02 — `hp41-gui/src-tauri/permissions/request-cancel.toml` (config)

**Analog:** `hp41-gui/src-tauri/permissions/run-stop.toml` (verbatim shape):

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-run-stop"
description = "Allows the run_stop command."
commands.allow = ["run_stop"]
```

**Phase 31 target (Plan 31-02):**

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-request-cancel"
description = "Allows the request_cancel command."
commands.allow = ["request_cancel"]
```

Plans 31-03 produces three more permission files with identical shape: `submit-modal.toml`, `cancel-modal.toml`, `submit-modal-with-label.toml`.

---

### Plan 31-02 — `hp41-gui/src-tauri/src/lib.rs` (config / app bootstrap)

**Analog:** self lines 60-66 (existing `generate_handler!` block):

```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    commands::sst_step, // Phase 18 D-05
    commands::bst_step, // Phase 18 D-05
    commands::run_stop, // Phase 19 (v2.1) — R/S key toggle
])
```

**Phase 31 extension (Plans 31-02 + 31-03):** add four entries — `request_cancel`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`. The four entries MUST each have a matching `permissions/<kebab>.toml` (Tauri v2.11 inline-command lifecycle, CLAUDE.md "v2.0 additions" line on permissions).

**Capabilities registry (also modify `capabilities/default.json`):**

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
    "allow-bst-step",
    "allow-run-stop",
    "allow-request-cancel",       // Plan 31-02
    "allow-submit-modal",          // Plan 31-03
    "allow-cancel-modal",          // Plan 31-03
    "allow-submit-modal-with-label" // Plan 31-03
  ]
}
```

---

### Plan 31-02 — `scripts/check-tauri-permissions.sh` (utility / CI gate)

**Analog:** none in tree (per RESEARCH Open Q1 — script doesn't exist; planner authors from sketch). The contract is well-defined:

```bash
#!/usr/bin/env bash
# scripts/check-tauri-permissions.sh — verify every generate_handler! command has a TOML
set -euo pipefail
HANDLER_FILE="hp41-gui/src-tauri/src/lib.rs"
PERMS_DIR="hp41-gui/src-tauri/permissions"
commands=$(grep -oE 'commands::[a-z_]+' "$HANDLER_FILE" | sed 's/commands:://')
missing=0
for cmd in $commands; do
    kebab=$(echo "$cmd" | sed 's/_/-/g')
    if [[ ! -f "$PERMS_DIR/$kebab.toml" ]]; then
        echo "MISSING: $PERMS_DIR/$kebab.toml (for command $cmd)"
        missing=1
    fi
done
exit $missing
```

Call site: `justfile`'s `gui-ci` recipe (planner extends).

---

### Plan 31-03 — `hp41-gui/src-tauri/src/commands.rs` (modal thunks, request-response)

**Analog:** `hp41-gui/src-tauri/src/commands.rs::run_stop` (no-arg) + `dispatch_op` (with `&str` arg).

**Phase 31 targets (verbatim per RESEARCH §Pitfall 9 lines 568-589):**

```rust
#[tauri::command]
pub fn submit_modal(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::submit_modal(&mut calc).map_err(GuiError::from)?;
    handle_get_state(&mut calc)
}

#[tauri::command]
pub fn cancel_modal(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::cancel_modal(&mut calc); // no Result — always succeeds
    handle_get_state(&mut calc)
}

#[tauri::command]
pub fn submit_modal_with_label(label: &str, state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::submit_modal_with_label(&mut calc, label).map_err(GuiError::from)?;
    handle_get_state(&mut calc)
}
```

**Shared core entry points** (FROZEN since Phase 28/29):
- `hp41_core::ops::math1::submit_modal` (`hp41-core/src/ops/math1/mod.rs` lines 54-81)
- `hp41_core::ops::math1::cancel_modal` (lines 92-96)
- `hp41_core::ops::math1::submit_modal_with_label` (lines 109-135)

D-25.6 parity invariant: GUI thunks are 4-line glue around shared core functions — NO Math Pac I logic in `hp41-gui/src-tauri/`.

---

### Plan 31-03 — `hp41-gui/src-tauri/src/types.rs` (model / DTO extensions)

**Analog:** self lines 25-57 (existing 14-field struct) + lines 66-145 (`from_state` constructor).

**Phase 31 additions** (per RESEARCH Pitfall 10 lines 601-613):

```rust
// Append to CalcStateView struct (lines 25-57):
pub is_running: bool,                    // = state.is_running (currently NOT projected)
pub modal_program_active: bool,          // = state.modal_program.is_some()
pub modal_requires_alpha_label: bool,    // = state.modal_program.as_ref().map(|m| m.requires_alpha_label()).unwrap_or(false)
pub modal_prompt: Option<String>,        // = state.modal_prompt.clone() — for debug + accessibility
```

**`from_state` extensions** (in lines 96+ of types.rs):

```rust
let is_running = state.is_running;
let modal_program_active = state.modal_program.is_some();
let modal_requires_alpha_label = state
    .modal_program
    .as_ref()
    .map(|m| m.requires_alpha_label())
    .unwrap_or(false);
let modal_prompt = state.modal_prompt.clone();
```

**Payload-size budget regression** (extend existing `test_dispatch_op_payload_size` lines 169-208) — budget is 500 bytes, baseline 337 + 4 new fields (~100 bytes) → ~437 bytes (63-byte headroom).

**`HpError::Canceled` → "CANCELED" override** (per RESEARCH Pitfall 4 lines 504-516):

```rust
// hp41-gui/src-tauri/src/types.rs lines 153-161 — extend From impl
impl From<HpError> for GuiError {
    fn from(e: HpError) -> Self {
        let message = match e {
            HpError::Canceled => "CANCELED".to_string(), // uppercase per UI-SPEC
            other => other.to_string(),
        };
        GuiError { message }
    }
}
```

---

### Plan 31-03 — `hp41-gui/src-tauri/tests/d25_6_parity.rs` (test, transform)

**Analog:** RESEARCH §"Code Examples / Example 9" lines 781-810 supplies the verbatim template:

```rust
// hp41-gui/src-tauri/tests/d25_6_parity.rs (new test file)
//
// Drive identical input through xrom_resolve from both CLI's resolver and
// GUI's dispatch_op path. Both must produce identical X-register output.
#[test]
fn parity_sinh_1_5() {
    use hp41_core::{CalcState, HpNum, ops::dispatch, ops::Op};
    use hp41_core::ops::math1::xrom::xrom_resolve;

    // CLI path: xeq_by_name_local_resolve("SINH", state.xrom_modules) → Op::Sinh
    let resolved_cli = xrom_resolve("SINH", 0b0000_0001);
    assert_eq!(resolved_cli, Some(Op::Sinh));

    // GUI path: dispatch_op("xeq_SINH") → key_map → Op::Xeq("SINH") → run_program → xrom_resolve → Op::Sinh
    let mut state_gui = CalcState::new();
    state_gui.stack.x = HpNum::from_f64(1.5).unwrap();
    state_gui.program.push(Op::Lbl("MAIN".into()));
    state_gui.program.push(Op::Xeq("SINH".into()));
    state_gui.program.push(Op::Rtn);
    hp41_core::run_program(&mut state_gui, "MAIN").unwrap();

    // Direct dispatch baseline
    let mut state_direct = CalcState::new();
    state_direct.stack.x = HpNum::from_f64(1.5).unwrap();
    dispatch(&mut state_direct, Op::Sinh).unwrap();

    assert_eq!(state_gui.stack.x, state_direct.stack.x, "GUI path must match direct dispatch");
}
```

---

### Plan 31-04 — `hp41-gui/src/help_data.ts` (model / data accessor, transform)

**Analog 1 (TS shape):** self lines 13-44 (existing `import hp41cvFunctions` + `HelpEntry` interface + `helpEntries()`).

**Analog 2 (semantic mirror — D-29.2 Rust pattern to port):** `hp41-cli/src/help_data.rs` lines 102-135:

```rust
// Rust pattern (Phase 29 D-29.2) — port verbatim shape to TypeScript:
const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");
static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_math1() -> &'static [HelpEntry] {
    MATH1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(MATH1_FUNCTIONS_JSON)
            .expect("hp41-math1-functions.json is malformed — fix the JSON")
    })
}

pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries().iter().chain(help_entries_math1().iter())
}
```

**Phase 31 TypeScript port:**

```typescript
// hp41-gui/src/help_data.ts (Plan 31-04 extensions)
import functions from '../../docs/hp41cv-functions.json';           // existing line 13
import math1Functions from '../../docs/hp41-math1-functions.json';  // NEW Phase 31

/// Append optional XROM descriptor (C-28.3 / Phase 29 D-29.1 schema):
export interface XromEntry {
    module: string;       // "Math 1"
    module_id: number;    // 7
    function_id: number;  // 1-based index in MATH_1.ops
}

export interface HelpEntry {
    op_variant: string;
    display_name: string;
    category: string;
    status: 'implemented' | 'deferred-v3' | 'na';
    phase: string | null;
    key_path: string | null;
    description: string;
    divergences?: string[];
    xrom?: XromEntry;     // NEW Phase 31 — present for Math Pac I entries only
}

// Narrow accessors (one per pool):
export function helpEntries(): readonly HelpEntry[] {
    return functions as readonly HelpEntry[];
}
export function helpEntriesMath1(): readonly HelpEntry[] {
    return math1Functions as readonly HelpEntry[];
}
// Merged accessor (UI render paths use this):
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1()];
}
```

**Vite cross-directory note:** RESEARCH §Pitfall 7 lines 542-548 confirms the `../../docs/...` relative path works because `vite.config.ts` line 18-19 already sets `fs.allow: [path.resolve(__dirname, '..')]` (Phase 26 W8). **No `vite.config.ts` changes needed.**

**Hard-build-blocker:** Vite static JSON-import fails build on malformed JSON (mirror of CLI's `.expect("...malformed")` — D-25.17 GUI parallel).

---

### Plan 31-04 — `hp41-gui/src/HelpOverlay.tsx` (component / two-section wrapper)

**Analog:** self lines 22-98 (existing one-section structure).

**Existing one-section grouping pattern** (lines 28-39):

```typescript
const grouped = useMemo(() => {
    const groups = new Map<string, HelpEntry[]>();
    for (const entry of filtered) {
        const arr = groups.get(entry.category);
        if (arr) {
            arr.push(entry);
        } else {
            groups.set(entry.category, [entry]);
        }
    }
    return Array.from(groups.entries());
}, [filtered]);
```

**Phase 31 extension — wrap inside two-section grouping by `entry.xrom?.module === 'Math 1'`:**

```typescript
// Plan 31-04 target structure:
const SECTIONS: Array<{ id: 'hp41cv' | 'math1'; heading: string; predicate: (e: HelpEntry) => boolean }> = [
    { id: 'hp41cv', heading: 'HP-41CV (built-in)', predicate: e => !e.xrom },
    { id: 'math1',  heading: 'Math 1 Pac (XROM 7)', predicate: e => e.xrom?.module === 'Math 1' },
];

// Render structure:
// .help-overlay-content
//   .help-overlay-section{2}
//     .help-overlay-section-heading  (collapsible, 14px bold #f5a423, focusable <button>)
//     .help-overlay-section-body
//       .help-overlay-category-heading{N}  (existing 13px #f5a423)
//       .help-overlay-row{M}               (existing)
```

**Per UI-SPEC §Component Inventory (line 167) verbatim DOM shape:**

```
.help-overlay-content > .help-overlay-section{2}
  > .help-overlay-section-heading + .help-overlay-section-body
    > .help-overlay-category-heading{N} + .help-overlay-row{M}
```

Section collapse state: local `useState<{ hp41cv: boolean; math1: boolean }>({hp41cv: true, math1: true})` — both expanded on open per D-31.8; no localStorage per Claude's discretion (CONTEXT line 158).

Accessibility (UI-SPEC §Accessibility line 313): section headers are `<button>` with `aria-expanded`.

---

### Plan 31-04 — `hp41-gui/src/App.css` (style)

**Analog:** self lines 325-335 (existing `.help-overlay-category-heading`):

```css
.help-overlay-category-heading {
  margin: 12px 0 6px 0;
  color: #f5a423;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  border-bottom: 1px solid #3a3a3a;
  padding-bottom: 4px;
}
```

**Phase 31 additions** (one new selector pair per UI-SPEC Typography line 78):

```css
/* Plan 31-04 — D-31.8 top-level section heading (one step up from category-heading). */
.help-overlay-section-heading {
  margin: 16px 0 8px 0;
  color: #f5a423;
  font-size: 14px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  border-bottom: 1px solid #3a3a3a;
  padding-bottom: 4px;
  background: transparent;
  border-left: none;
  border-right: none;
  border-top: none;
  cursor: pointer;
  width: 100%;
  text-align: left;
}

.help-overlay-section-body {
  /* No additional indentation — keep visual rhythm with v2.2 category rows. */
}
```

UI-SPEC §Color line 101 reserves `#f5a423` accent for the section heading addition (item (f) in the 8-item accent-reserved list).

---

### Plan 31-04 — `hp41-core/src/ops/program.rs::op_catalog` (surgical CAT 2 extension, streaming)

**Analog 1 (extension pattern precedent):** v2.2 Plan 25-03 `builtin_card_op` 4→12 extension (CLAUDE.md v2.2 section: "the surgical 4→12 `builtin_card_op` extension in Plan 25-03 is the documented exception"). Same shape: visibility unchanged, no API widening, single-arm extension.

**Analog 2 (line-by-line pattern):** self lines 307-339 (existing CAT 1 body inside `op_catalog`).

**Existing stub** (lines 335-339 — to REPLACE):

```rust
2..=4 => {
    // CATALOG 2 (XROM modules) / 3 (HP-IL) / 4 (peripherals) — none
    // in this emulator → single payload line.
    state.print_buffer.push(format!("{:<24}", "NOT AVAILABLE"));
}
```

**Phase 31 surgical replacement** (per RESEARCH Pitfall 2 lines 466-484):

```rust
2 => {
    // CATALOG 2: XROM modules (Phase 31 surgical hp41-core exception per Plan 25-03 precedent).
    // For v3.0, only Math 1 is loaded (xrom_modules bit 0).
    if state.xrom_modules & 0b0000_0001 != 0 {
        use crate::ops::math1::xrom::MATH_1;
        state.print_buffer.push(format!("{:<24}", format!("XROM {} {}", MATH_1.id, MATH_1.name)));
        for (name, _op) in MATH_1.ops {
            state.print_buffer.push(format!("{:<24}", name));
        }
    } else {
        state.print_buffer.push(format!("{:<24}", "NO XROM"));
    }
}
3..=4 => {
    // CAT 3 (HP-IL) + CAT 4 (peripherals) — still NOT AVAILABLE.
    state.print_buffer.push(format!("{:<24}", "NOT AVAILABLE"));
}
```

**Visibility:** `op_catalog` is already `pub fn` (line 300). No API widening. SC-4 invariant trivially preserved (the change lives in `hp41-core`, not `hp41-gui/src-tauri/`).

**CAT 2 PSE semantics (per RESEARCH Pitfall 11 lines 619-628):** v2.2 CAT 1 is synchronous push-to-buffer (no PSE-step). Phase 31 CAT 2 mirrors this — all lines arrive in the next `get_state` response. The CONTEXT.md D-31.12/D-31.14 references to "~500ms PSE delay" describe the UX intention; the underlying mechanism is "instant scroll, frontend appends to print panel". Defer per-line PSE to v3.1 polish.

---

### Plan 31-05 — `hp41-gui/src-tauri/src/commands.rs::handle_get_state` (LCD-alternation routing, transform)

**Analog:** self lines 228-232 + `types.rs::from_state` lines 75-81 (existing display_str priority chain).

**Existing handle_get_state body** (lines 228-232 — to extend):

```rust
pub fn handle_get_state(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

**Existing display_str priority chain inside `from_state`** (`types.rs` lines 75-81):

```rust
// display_str priority chain (D-01 + Claude's Discretion):
//   1. entry_buf (when user is typing)
//   2. alpha_reg via format_alpha (when alpha_mode is on)
//   3. format_hpnum(stack.x, display_mode) (default)
let display_str = if !state.entry_buf.is_empty() {
    state.entry_buf.clone()
} else if state.alpha_mode {
    format_alpha(&state.alpha_reg)
} else {
    format_hpnum(&state.stack.x, &state.display_mode)
};
```

**Phase 31 extension target — Claude's Discretion recommends `handle_get_state` (RESEARCH §76; CONTEXT line 147-151):**

```rust
// Plan 31-05: add helper next to handle_get_state.
const LCD_WIDTH: usize = 12;
const CONTINUATION: char = '\u{2261}'; // HP-41 ≡ truncation marker (D-31.6)

fn truncate_with_continuation(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= LCD_WIDTH {
        return s.to_string();
    }
    let mut result: String = chars.iter().take(LCD_WIDTH - 1).collect();
    result.push(CONTINUATION);
    result
}

// Patch handle_get_state to route modal_prompt → display BEFORE the drain.
pub fn handle_get_state(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    // D-31.5: LCD-alternation — when a modal is open AND entry_buf is empty,
    // override the display string with the truncated modal prompt.
    if calc.modal_program.is_some() && calc.entry_buf.is_empty() {
        if let Some(ref prompt) = calc.modal_prompt {
            // Stash into display_override (existing Phase 26 D-26.11 channel).
            // The from_state precedence chain will surface it.
            calc.display_override = Some(truncate_with_continuation(prompt));
        }
    }
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

**Alternative routing:** Add a 4th priority arm to `from_state`'s display_str chain (between entry_buf and alpha_reg). Both placements are acceptable; planner picks one and documents.

**Edge cases tested per UI-SPEC §State & Interaction line 245:**
- 12-char prompt: no truncation
- 13-char: truncated, marker at position 12
- 14-char (`FUNCTION NAME?` → `FUNCTION NAM≡`)
- Unicode-rich prompt (per-char not per-byte truncation)
- Empty `modal_prompt` (renders empty 12-char display = LCD blank)

---

### Plan 31-05 — `hp41-gui/src/App.tsx` (R/S 3-way + Esc cascade + post-dispatch auto-open, event-driven)

**Analog 1 (CLI ground truth — R/S submit):** `hp41-cli/src/app.rs` lines 664-683:

```rust
if key.code == KeyCode::F(5)
    && self.state.modal_program.is_some()
    && self.pending_input.is_none()
{
    match hp41_core::ops::math1::submit_modal(&mut self.state) {
        Ok(()) => {
            self.message = None;
            self.drain_and_show_print_output(None);
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
    self.state.last_key_code = 0;
    return;
}
```

**Analog 2 (CLI ground truth — Esc cancel):** `hp41-cli/src/app.rs` lines 685-698:

```rust
if key.code == KeyCode::Esc
    && self.state.modal_program.is_some()
    && self.pending_input.is_none()
{
    hp41_core::ops::math1::cancel_modal(&mut self.state);
    self.message = Some("Cancelled".to_string());
    return;
}
```

**Analog 3 (existing GUI invokeForKey to extend):** self lines 62-67:

```typescript
async function invokeForKey(effectiveId: string): Promise<CalcStateView> {
  if (effectiveId === 'sst') return invoke<CalcStateView>('sst_step');
  if (effectiveId === 'bst') return invoke<CalcStateView>('bst_step');
  if (effectiveId === 'r_s') return invoke<CalcStateView>('run_stop');
  return invoke<CalcStateView>('dispatch_op', { keyId: effectiveId });
}
```

**Phase 31 — extend `invokeForKey` with state-aware routing:**

```typescript
// Plan 31-05 target — invokeForKey is no longer purely id-based; it accepts state.
async function invokeForKey(effectiveId: string, state: CalcStateView | null): Promise<CalcStateView> {
  if (effectiveId === 'sst') return invoke<CalcStateView>('sst_step');
  if (effectiveId === 'bst') return invoke<CalcStateView>('bst_step');
  if (effectiveId === 'r_s') {
    // D-31.1: 3-way state-routed.
    //   1. modal_program.is_some() → submit_modal
    //   2. is_running               → request_cancel
    //   3. else                     → run_stop (existing v2.1)
    if (state?.modal_program_active) return invoke<CalcStateView>('submit_modal');
    if (state?.is_running) {
      await invoke<void>('request_cancel');
      return invoke<CalcStateView>('get_state');  // refresh after async cancel
    }
    return invoke<CalcStateView>('run_stop');
  }
  return invoke<CalcStateView>('dispatch_op', { keyId: effectiveId });
}
```

**Existing GUI Esc precedence to extend** (lines 436-448):

```typescript
if (e.key === 'Escape') {
  if (helpOpen) { setHelpOpen(false); return; }
  if (pendingInput !== null) { setPendingInput(null); setShiftActive(false); return; }
  setShiftActive(false);
  return;
}
```

**Phase 31 Esc extension — D-31.2 priority chain (insert between pendingInput branch and shiftActive branch):**

```typescript
if (e.key === 'Escape') {
  if (helpOpen) { setHelpOpen(false); return; }
  if (pendingInput !== null) { setPendingInput(null); setShiftActive(false); return; }
  // D-31.2 — Esc universal cancel.
  //   1. modal_program.is_some() → cancel_modal (calls shared core)
  //   2. is_running               → request_cancel
  //   3. shiftActive              → clear (existing v2.1)
  //   4. else no-op.
  if (calcState?.modal_program_active) {
    void invoke<CalcStateView>('cancel_modal').then(view => setCalcState(view))
      .catch(err => showToast(extractErrMessage(err)));
    return;
  }
  if (calcState?.is_running) {
    void invoke<void>('request_cancel').then(() => invoke<CalcStateView>('get_state'))
      .then(view => view && setCalcState(view))
      .catch(err => showToast(extractErrMessage(err)));
    return;
  }
  setShiftActive(false);
  return;
}
```

**Post-dispatch auto-open `useEffect` — D-29.9 GUI mirror** (per RESEARCH §"Code Examples / Example 6" lines 728-740 — verbatim template ready):

```typescript
// Plan 31-05 target — new useEffect in App.tsx
useEffect(() => {
  if (!calcState) return;
  if (pendingInput !== null) return;
  if (!calcState.modal_program_active) return;
  if (!calcState.modal_requires_alpha_label) return;
  setPendingInput({
    kind: 'xeq_name',
    dispatchPrefix: 'xeq',
    acc: '',
    mode: 'collect-for-modal',
  });
}, [calcState, pendingInput]);
```

**Pitfall 5 (RESEARCH lines 525-528) — R/S 3-way insertion site:** the new branch in `handleClick` MUST be placed AFTER the existing `pending_input !== null` block at line 312-372 and BEFORE the MODAL_OPENERS intercept at line 374. Preserves D-07 never-discard invariant.

---

### Plan 31-05 — `hp41-gui/src/pending_input.ts` (model / state machine extension)

**Analog 1 (Rust source to port):** `hp41-cli/src/app.rs` lines 44-50 (XeqByNameMode) + lines 134-138 (XeqByName struct variant):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XeqByNameMode {
    Normal,
    CollectForModal,
}

XeqByName {
    acc: String,
    mode: XeqByNameMode,
}
```

**Analog 2 (existing TS variant to extend):** self line 58:

```typescript
| { kind: 'xeq_name'; acc: string; dispatchPrefix: 'xeq' | 'gto' | 'lbl' }
```

**Phase 31 TypeScript port — D-29.8 mirror (per RESEARCH §Pattern 4 lines 354-363):**

```typescript
// Recommendation per RESEARCH: extend existing variant with optional mode
// (default 'normal') so existing call sites continue working.
| { kind: 'xeq_name'; acc: string; dispatchPrefix: 'xeq' | 'gto' | 'lbl'; mode?: 'normal' | 'collect-for-modal' }
```

**Existing `xeq_name` handleModalKey arm to extend** (self lines 335-363):

```typescript
case 'xeq_name': {
  if (key === 'Enter') {
    if (pending.acc.length === 0) {
      return { nextPending: pending, dispatchId: null, consumesShift: false };
    }
    // PHASE 31 EXTENSION — CollectForModal branch:
    if (pending.mode === 'collect-for-modal') {
      // Dispatch via submit_modal_with_label Tauri thunk instead of xeq_<acc>.
      return {
        nextPending: null,
        dispatchId: `__submit_modal_with_label__${pending.acc}`,  // magic prefix for App.tsx route
        consumesShift: false,
      };
    }
    // Existing Normal-mode behavior:
    return {
      nextPending: null,
      dispatchId: `${pending.dispatchPrefix}_${pending.acc}`,
      consumesShift: false,
    };
  }
  // ... existing Backspace + printable-char arms unchanged
}
```

**App.tsx complementary route** (extend `invokeForKey` or `applyModalResult`):

```typescript
// Plan 31-05 — recognise the magic prefix in App.tsx's applyModalResult
if (result.dispatchId?.startsWith('__submit_modal_with_label__')) {
  const label = result.dispatchId.slice('__submit_modal_with_label__'.length);
  const view = await invoke<CalcStateView>('submit_modal_with_label', { label });
  setCalcState(view);
  return true;
}
```

(Magic-prefix pattern precedent: existing `makeKeyCodeMagic` in `pending_input.ts` line 508 for ASN flow.)

---

### Plan 31-05 — `hp41-gui/src/Display14Seg.tsx` (glyph table extension)

**Analog:** self lines 131-184 (SEGMENT_MAP one-line entries; `'='` at line 177 is the closest sibling).

**Existing entry pattern** (line 177):

```typescript
'=': [6, 7, 3],            // middle bar + bottom (two horizontals)
```

**Phase 31 addition** (per RESEARCH §Pitfall 6 lines 530-538 + Assumption A4 line 837):

```typescript
// Plan 31-05: add ≡ (U+2261) — HP-41 ALPHA-truncation marker (D-31.6).
// Renders as a three-bar shape: top horizontal + middle bar + bottom horizontal.
// Recommended segment combination: [0, 6, 7, 3] (a + g1 + g2 + d).
'\u{2261}': [0, 6, 7, 3],
```

**Vitest snapshot to add** (Display14Seg.test.tsx — existing test file):

```typescript
test('renders ≡ (U+2261) HP-41 continuation marker as three-bar', () => {
  const { container } = render(<Display14Seg text="FUNCTION NAM\u{2261}" />);
  // Cell index 11 (the ≡) should have segments 0, 6, 7, 3 lit.
  // Assertion shape mirrors existing SEGMENT_MAP tests.
});
```

---

## Shared Patterns

### Pattern S1 — Tauri command thunk shape (Plans 31-02 + 31-03)

**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 250-260 (`run_stop`) + 281-288 (`handle_run_stop`)

**Applies to:** all 4 new Tauri commands (`request_cancel`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`)

**Shape:**
1. `#[tauri::command]` thunk acquires `State<'_, AppState>` with `.lock().unwrap_or_else(|e| e.into_inner())` poisoned-lock recovery
2. Delegates to pure-Rust `handle_<name>(&mut CalcState)` helper for unit-testability
3. Helper drains `print_buffer` + `event_buffer` BEFORE `CalcStateView::from_state` (Pitfall 1)
4. Returns `Result<CalcStateView, GuiError>` (or `Result<(), GuiError>` for `request_cancel` — silent)

**Exception (request_cancel only):** takes `State<'_, CancelFlag>` (separately-managed Arc) NOT `State<'_, AppState>` — see RESEARCH Pitfall 1.

---

### Pattern S2 — Tauri permission TOML (Plans 31-02 + 31-03)

**Source:** `hp41-gui/src-tauri/permissions/run-stop.toml`

**Applies to:** `request-cancel.toml`, `submit-modal.toml`, `cancel-modal.toml`, `submit-modal-with-label.toml`

**Shape (4 files, each):**

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-<kebab-name>"
description = "Allows the <snake_name> command."
commands.allow = ["<snake_name>"]
```

Also register in `capabilities/default.json` `permissions` array.

CI gate: `scripts/check-tauri-permissions.sh` (NEW in Plan 31-02 per RESEARCH Open Q1).

---

### Pattern S3 — D-25.6 CLI ↔ GUI parity (all plans)

**Source:** CLAUDE.md "v2.2 additions / D-25.6" + 31-CONTEXT.md line 49 + RESEARCH lines 263-264

**Applies to:** Plans 31-02, 31-03, 31-05 (every plan that adds a GUI behavior with a Math Pac I or modal path)

**Contract:**
- GUI calls shared `hp41-core::ops::math1::*` functions verbatim — NO parallel implementations
- `key_map.rs` gets ZERO new arms in Phase 31 (Math Pac I = XEQ-by-name only, D-28.6)
- Behaviors that mirror CLI (R/S submit, Esc cancel, post-dispatch auto-open) port the CLI logic 1:1
- Parity test in Plan 31-03 (`d25_6_parity.rs`) drives identical input through both paths and asserts identical X-register output

---

### Pattern S4 — Exhaustive match invariant (Plan 31-05)

**Source:** CLAUDE.md "FN-CLI-04 compile-time guarantee" + 31-CONTEXT.md line 52 "FN-GUI-04 exhaustive match" + `pending_input.ts` existing switch over 14 variants (no `default:`)

**Applies to:** `pending_input.ts` `handleModalKey` / `renderModalLcd` switch statements when adding the `mode` discriminator to `xeq_name`

**Contract:** every `switch (pending.kind)` exhausts all variants; every `switch (pending.mode)` (if planner extracts a sub-switch) exhausts `'normal' | 'collect-for-modal'`. NO `default:` catch-all anywhere.

---

### Pattern S5 — Vite static JSON-import as hard-build-blocker (Plan 31-04)

**Source:** `hp41-gui/src/help_data.ts` line 13 (existing) + `vite.config.ts` lines 18-19 (Phase 26 W8 fs.allow)

**Applies to:** `hp41-gui/src/help_data.ts` parallel-load of `hp41-math1-functions.json`

**Contract:**
- Static `import` at module top — resolved at BUILD TIME
- Malformed JSON fails Vite build (parallel to Rust's `.expect("...malformed")`)
- No async fetch, no Tauri IPC, no `vite.config.ts` changes (the existing fs.allow covers it)
- The merged accessor `helpEntriesAll()` is the single source of truth for the `?`-overlay's two-section render

---

### Pattern S6 — Surgical `hp41-core` extension exception (Plan 31-04)

**Source:** CLAUDE.md v2.2 section: "the surgical 4→12 `builtin_card_op` extension in Plan 25-03 is the documented exception" + 31-CONTEXT.md line 152 + RESEARCH Pitfall 2 lines 466-484

**Applies to:** `hp41-core/src/ops/program.rs::op_catalog` n==2 arm (replace `NOT AVAILABLE` stub with XROM enumeration)

**Contract:**
- Visibility unchanged (`op_catalog` already `pub fn`)
- No API widening (signature stays `fn op_catalog(state: &mut CalcState, n: u8) -> Result<(), HpError>`)
- Documented in plan SUMMARY.md as a Phase 31 exception with rationale
- SC-4 trivially preserved (change in `hp41-core/`, not `hp41-gui/src-tauri/`)
- `tests/xrom_shadowing.rs` continues to pass (no new MATH_1.ops entries)

---

### Pattern S7 — Lock-release-during-async pattern (Plan 31-02)

**Source:** `hp41-gui/src-tauri/src/lib.rs` lines 49-55 (auto-save commit ff39017) + `commands.rs` lines 49-69 (`dispatch_op` three-phase prepare/execute/finalize)

**Applies to:** `request_cancel` Tauri thunk + understanding that `op_integ`/`op_solve`/`op_difeq` MUST release the AppState Mutex between sample batches (per ROADMAP Phase 31 SC-4 line 168)

**Contract:**
- Long ops clone the `Arc<AtomicBool>` from `state.cancel_requested` at op entry
- The Arc is reachable lock-free from anywhere (the AtomicBool itself doesn't need the Mutex)
- `request_cancel` Tauri thunk acquires `State<'_, CancelFlag>` — NOT `State<'_, AppState>` — so it cannot deadlock waiting for the dispatch lock (RESEARCH Pitfall 1)
- The cancel reset happens at `op_integ`/`op_solve`/`op_difeq` workflow-opener time (per RESEARCH Open Q6 — Plan 31-02 audit task verifies this is wired; if not, adds the one-line `cancel_requested.store(false, Relaxed)` at op-entry)

---

### Pattern S8 — Toast inheritance for `HpError::Canceled` (Plan 31-02)

**Source:** `hp41-gui/src/App.tsx` lines 201-214 (existing `toastMsg` state + `showToast` + 2s auto-dismiss `useEffect`) + `hp41-gui/src-tauri/src/types.rs` lines 153-161 (existing `From<HpError> for GuiError`)

**Applies to:** cancellation feedback (D-31.3) — zero new render code

**Contract:**
- `HpError::Canceled` (already exists in `hp41-core/src/error.rs` line 37; `Display` returns lowercase `"canceled"`)
- `From<HpError> for GuiError` overrides to UPPERCASE `"CANCELED"` per UI-SPEC (RESEARCH Pitfall 4)
- Flows through existing toast pathway — no new state, no new CSS
- 2s auto-dismiss inherits unchanged

---

## No Analog Found

| File | Role | Data Flow | Reason / Mitigation |
|------|------|-----------|---------------------|
| `scripts/check-tauri-permissions.sh` | utility / CI gate | batch | Per RESEARCH Open Q1 lines 851-872 — the script is referenced in CLAUDE.md/CONTEXT/Code Insights as a Pitfall-21 gate but `find scripts -name "*.sh"` returns zero results. Planner authors from the sketch provided in RESEARCH §"Open Questions / 1" lines 855-872 (12 lines of bash). Wire into `justfile`'s `gui-ci` recipe. |

(Every other file in the 5 plans has a concrete in-tree analog.)

---

## Metadata

**Analog search scope:**
- `hp41-gui/src/` (all .tsx, .ts, .css files)
- `hp41-gui/src-tauri/src/` (all .rs files)
- `hp41-gui/src-tauri/permissions/` (5 existing TOMLs)
- `hp41-gui/src-tauri/capabilities/default.json`
- `hp41-gui/vite.config.ts`
- `hp41-cli/src/app.rs` (Phase 29 R/S submit + Esc cancel + auto-open ground truth)
- `hp41-cli/src/help_data.rs` (Phase 29 D-29.2 parallel-load merged accessor)
- `hp41-cli/src/keys.rs` (xeq_by_name_local_resolve)
- `hp41-cli/src/prgm_display.rs` (Phase 29 ~40 Math Pac I arms — sibling of GUI shipped arms)
- `hp41-core/src/ops/program.rs` (op_catalog stub; xrom_resolve fallthrough; run_program)
- `hp41-core/src/ops/math1/mod.rs` (submit_modal / cancel_modal / submit_modal_with_label)
- `hp41-core/src/ops/math1/modal.rs` (ModalProgram + requires_alpha_label)
- `hp41-core/src/ops/math1/xrom.rs` (MATH_1 registry)
- `hp41-core/src/state.rs` (is_running, cancel_requested, modal_program, modal_prompt fields)
- `hp41-core/src/error.rs` (HpError::Canceled)
- `docs/hp41-math1-functions.json` (~55 entries authored Phase 29 D-29.1)
- `docs/hp41cv-functions.json` (130 v2.2 entries — sibling)
- `.planning/phases/31-gui-integration/31-{CONTEXT,RESEARCH,UI-SPEC}.md`
- `.planning/ROADMAP.md` Phase 31 section (lines 154-185)
- `CLAUDE.md` v2.2 / v3.0 sections

**Files scanned:** 24 read with targeted offsets; 12 grep audits.

**Pattern extraction date:** 2026-05-17

**Quality indicators:**
- Concrete, not abstract: every excerpt cites file path + line numbers + verbatim code
- Accurate classification: role + data flow per file matches the file's actual purpose
- Best analog selected: closest by role + data flow + recency (v2.1 / v2.2 / Phase 28-29 patterns preferred over v2.0 baseline where both exist)
- Actionable for planner: planner can copy excerpts directly into plan action sections
