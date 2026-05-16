# Phase 26: GUI Integration & Polish — Pattern Map

**Mapped:** 2026-05-15
**Files analyzed:** 11 (5 modified, 3 new TS, 1 new TOML, 2 modified support)
**Analogs found:** 11 / 11 (100% — every target file has a strong existing in-repo analog)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/src/key_map.rs` (modify) | resolver | request-response (string → enum) | self (extend `resolve` + `resolve_parameterized`) | exact |
| `hp41-gui/src-tauri/src/types.rs` (modify) | DTO/projection | transform | self (extend `CalcStateView::from_state`) | exact |
| `hp41-gui/src-tauri/src/commands.rs` (modify, optional) | controller (Tauri command) | request-response | self — `run_stop` is the precedent for any new ASN command | exact |
| `hp41-gui/src-tauri/src/lib.rs` (modify, conditional) | config / handler registry | event-driven | self (`generate_handler!` + `setup`) | exact |
| `hp41-gui/src/App.tsx` (modify) | component / controller | event-driven (modal state machine) | self — `shiftActive` Phase 19 + `handleClick` resolution priority | exact |
| `hp41-gui/src/Keyboard.tsx` (modify) | component (SVG) | event-driven (click → callback) | self — `KeyDef` three-label + `handleKeyClick` | exact |
| `hp41-gui/src/Display14Seg.tsx` (CREATE) | component (SVG) | render-only | `Keyboard.tsx` SVG-grid + `pressedKey` modeling | role-match |
| `hp41-gui/src/HelpOverlay.tsx` (CREATE) | component (modal overlay) | render + filter | `print-panel` / `prgm-panel` overlay div in `App.tsx` lines 310–342 | role-match |
| `hp41-gui/src/help_data.ts` (CREATE) | data loader | file-I/O (vite import) | `hp41-cli/src/help_data.rs` — JSON-import + `HelpEntry` interface | role-match (Rust → TS) |
| `hp41-gui/src/App.css` (modify) | styles | render | self — `.toast` + `.prgm-panel` + `.key-shift-active` precedents | exact |
| `hp41-gui/src-tauri/permissions/<new>.toml` (CREATE, conditional) | config (Tauri permission) | declarative | `permissions/run-stop.toml` (4-line template) | exact |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/key_map.rs` (extend) — resolver, request-response

**Analog:** self (this file is the established pattern; Phase 26 extends it in-place)

**Imports pattern** (lines 10–12):
```rust
use crate::types::GuiError;
use hp41_core::ops::{Op, StackReg};
use hp41_core::StoArithKind;
```
Phase 26 extends with `hp41_core::ops::{FlagTestKind, TestKind}` plus any new enums needed for the named-op resolvers (Tone, Catalog, etc.).

**Bare-op resolver pattern** (lines 18–92, every line is `"<id>" => Ok(Op::<Variant>),`):
```rust
pub fn resolve(key_id: &str) -> Result<Op, GuiError> {
    match key_id {
        // ── Stack ────────────────────────────────────────────────────────────
        "enter" => Ok(Op::Enter),
        "clx" => Ok(Op::Clx),
        // ── Comparison (Test variants — keyboard-accessible without label arg) ───
        "xge_y" => Ok(Op::Test(hp41_core::ops::TestKind::XGeY)),
        // ── Print ────────────────────────────────────────────────────────────
        "prx" => Ok(Op::PRX),
        // ...
```
Phase 26 adds ~80 new arms following this exact format: `"pi" => Ok(Op::Pi)`, `"polar_to_rect" => Ok(Op::PolarToRect)`, `"abs" => Ok(Op::Abs)`, `"arcl" => Ok(Op::Arcl)`, `"asto" => Ok(Op::Asto)`, `"atox" => Ok(Op::Atox)`, etc. Keep the section comment headers (`// ── Section ──`) for navigability.

**Stub-error arm pattern** (lines 93–104) — Phase 26 SHRINKS this:
```rust
// ── Stub-error arm: ids that are clickable in the skin but not yet ──
"pi" | "polar_to_rect" | "rect_to_polar" | "beep" | "asn" | "catalog" | "view"
| "xeq_prompt" | "gto_prompt" | "lbl_prompt" => Err(GuiError {
    message: format!("'{key_id}' is planned for a future phase"),
}),
```
**Phase 26 D-26.5:** the bare ROM ops (`pi`, `polar_to_rect`, `rect_to_polar`, `beep`) move to real `Ok(Op::*)` arms above. The 13 `*_prompt` ids + `asn`/`view`/`catalog` STAY in the stub arm as defense-in-depth — the frontend intercepts them in `handleClick` BEFORE invoking dispatch_op (D-26.5), so the stub only fires on regression. Test `test_modal_prompt_ids_are_stubs_for_now` (lines 322–352) must continue to pass.

**Parameterized-prefix pattern** (lines 113–148):
```rust
fn resolve_parameterized(key_id: &str) -> Result<Op, GuiError> {
    // Single-prefix u8 args
    if let Some(rest) = key_id.strip_prefix("sto_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::StoReg(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("rcl_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::RclReg(n));
        }
    }
    // ...
```
Phase 26 adds new prefixes following this format: `sf_`, `cf_`, `fs_`, `view_`, `arcl_`, `asto_`, `tone_`, `del_`, `clp_` (label as raw string suffix), `sto_ind_`, `rcl_ind_`, `isg_ind_`, `dse_ind_`, `sf_ind_`, `cf_ind_`, `fs_ind_`, `view_ind_`, `arcl_ind_`, `asto_ind_`. The IND-bearing prefixes use `strip_prefix("sto_ind_")` BEFORE `strip_prefix("sto_")` to avoid the more-general arm winning (Pitfall 3 echo from existing `sto_arith_` pattern).

**Multi-segment parse pattern** (lines 152–154 + 182–216) — REUSE for `sto_arith_<op>_ind_<reg>` extension:
```rust
if let Some(rest) = key_id.strip_prefix("sto_arith_") {
    return resolve_sto_arith(rest, key_id);
}
// ...
fn resolve_sto_arith(rest: &str, original: &str) -> Result<Op, GuiError> {
    let (kind_str, reg_str) = rest.rsplit_once('_').ok_or_else(|| GuiError {
        message: format!("unknown key: {original}"),
    })?;
    // ...
}
```
Use `rsplit_once('_')` (NOT `split_once`) for any `<op>_<arg>` form where `<op>` may itself contain underscores. Phase 26 likely needs a similar helper for `sto_arith_plus_ind_05`-style ids if STO-arith IND lands.

**Test pattern** (lines 218–411):
```rust
#[test]
fn test_new_named_op_resolvers() {
    assert_eq!(resolve("sq").unwrap(), Op::Sq);
    assert_eq!(resolve("ypow").unwrap(), Op::YPow);
    // ...
}

#[test]
fn test_stub_error_for_v22_backlog_ops() {
    let stub_ids = ["pi", "polar_to_rect", "rect_to_polar", /* ... */];
    for id in stub_ids {
        let err = resolve(id).unwrap_err();
        assert!(err.message.contains("planned for a future phase"));
    }
}
```
Phase 26 EXTENDS `test_new_named_op_resolvers` with assertions for every newly-wired bare op, and SHRINKS `test_stub_error_for_v22_backlog_ops`'s `stub_ids` array to drop `pi`/`polar_to_rect`/`rect_to_polar`/`beep` (they now have real arms). The `test_modal_prompt_ids_are_stubs_for_now` array stays unchanged — those 13 ids remain backend stubs by design (frontend-intercepted per D-26.5).

---

### `hp41-gui/src-tauri/src/types.rs` (extend) — DTO / projection, transform

**Analog:** self (extend the existing `CalcStateView` + `from_state` pattern)

**Struct field pattern** (lines 25–38):
```rust
#[derive(Debug, Serialize)]
pub struct CalcStateView {
    pub display_str: String,
    pub x_str: String,
    pub y_str: String,     // Phase 15 D-01: stack Y register
    // ...
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>,
    pub program_steps: Vec<String>, // Phase 18 D-01: pre-formatted step strings
    pub pc: usize,                  // Phase 18 D-01: current program counter
}
```
Phase 26 D-26.11 adds:
```rust
    pub user_keymap: Vec<(u8, String)>,    // Phase 26 D-26.11: ASN map for USER-mode relabel
    pub flags: Vec<u8>,                    // OR `pub flags: u64` — planner picks compact repr
    pub display_override: Option<String>,  // Phase 26 D-26.11: surface display_override if not yet exposed
    pub event_buffer: Vec<String>,         // Phase 26 D-26.11: drained per IPC like print_lines
```
Each new field should carry a `// Phase 26 D-26.11:` provenance comment matching the Phase-15 / Phase-18 precedent.

**Constructor pattern** (lines 40–97):
```rust
impl CalcStateView {
    pub fn from_state(state: &CalcState, print_lines: Vec<String>) -> Self {
        let display_str = if !state.entry_buf.is_empty() {
            state.entry_buf.clone()
        } else if state.alpha_mode {
            format_alpha(&state.alpha_reg)
        } else {
            format_hpnum(&state.stack.x, &state.display_mode)
        };
        // ...
        let annunciators = Annunciators { /* ... */ };
        let program_steps = prgm_display::format_all_steps(state);
        let pc = state.pc;
        CalcStateView { display_str, x_str, /* ... */, program_steps, pc }
    }
}
```
**CRITICAL drain-before-call pitfall** (line 44 comment): the print_lines drain happens in the Tauri command BEFORE `from_state` is called (drain takes `&mut`, then `from_state` takes `&`). Phase 26's `event_buffer` MUST follow the same pattern: drain in `handle_op_finalize` / `handle_get_state` (commands.rs) and pass as a parameter, NOT drain inside `from_state`.

**Budget-test pattern** (lines 120–132):
```rust
#[test]
fn test_dispatch_op_payload_size() {
    let state = CalcState::new();
    let view = CalcStateView::from_state(&state, vec![]);
    let json = serde_json::to_string(&view).unwrap();
    assert!(
        json.len() <= 400,
        "CalcStateView JSON (empty program) must be ≤400 bytes, got {} bytes: {}",
        json.len(), json
    );
}
```
Phase 26 RAISES the budget to `≤ 500` per D-26.11 (FN-GUI-05). Update the assertion threshold AND the docstring; consider a second test seeded with ~10 ASN entries to verify the budget holds under realistic load.

---

### `hp41-gui/src-tauri/src/commands.rs` (extend, conditional) — controller, request-response

**Analog:** self — `run_stop` (lines 245–280) is the v2.1 precedent for any new dedicated Tauri command Phase 26 introduces.

**Tauri command + helper pair pattern** (lines 251–280):
```rust
#[tauri::command]
pub fn run_stop(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_run_stop(&mut calc)
}

pub fn handle_run_stop(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.is_running = !calc.is_running;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}
```
**Why two functions:** Tauri's `State<'_, AppState>` extractor is unmockable in unit tests (RESEARCH §Validation Architecture). The pure helper takes `&mut CalcState` and is testable; the `#[tauri::command]` thunk is 2 lines of glue. Phase 26 ASN-flow commands (if any) follow this exact split.

**Lock-poison-recovery pattern:** EVERY Tauri command must use `state.lock().unwrap_or_else(|e| e.into_inner())` — never `.unwrap()`. This is enforced by `#![deny(clippy::unwrap_used)]` in `lib.rs` line 1.

**D-26.12 default assumption:** ASN goes through `dispatch_op` with parameterized id `asn_NN_NAME` — so commands.rs likely needs ZERO changes. Only if planner finds a state-shape that doesn't fit a single Op does a new command appear.

**Test pattern** (lines 449–465 — `test_handle_run_stop_toggles_is_running`):
```rust
#[test]
fn test_handle_run_stop_toggles_is_running() {
    let mut calc = CalcState::new();
    handle_run_stop(&mut calc).unwrap();
    assert!(calc.is_running, "first run_stop must flip is_running to true");
    handle_run_stop(&mut calc).unwrap();
    assert!(!calc.is_running, "second run_stop must flip is_running back to false");
}
```
Mirror this exact structure for any new Phase 26 command helper.

---

### `hp41-gui/src-tauri/src/lib.rs` (extend, conditional) — config, event-driven

**Analog:** self (extend `generate_handler!` registration if new commands land)

**Handler registration pattern** (lines 60–66):
```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    commands::sst_step, // Phase 18 D-05
    commands::bst_step, // Phase 18 D-05
    commands::run_stop, // Phase 19 (v2.1) — R/S key toggle
])
```
Each entry carries a `// Phase XX` provenance comment. Phase 26 additions follow the same convention. Default expectation: NO new entries (ASN routes through `dispatch_op` with parameterized id).

---

### `hp41-gui/src/App.tsx` (extend) — component / controller, event-driven (modal state machine)

**Analog:** self — the `shiftActive` state machine (lines 111–124, 161–206) is the EXACT precedent for the new `pendingInput` state machine.

**Frontend-owned one-shot state pattern** (lines 111–124):
```typescript
// Frontend-owned SHIFT one-shot prefix (no IPC round-trip).
const [shiftActive, setShiftActive] = useState(false);
// Toast overlay for GuiError responses (single-toast policy, 2s auto-dismiss).
// The monotonic `seq` is required because two clicks on the same stubbed
// key produce identical message strings — setting state to the same value
// does not re-run the auto-dismiss effect, so the second toast would be
// dismissed by the first click's still-running timer.
const [toast, setToast] = useState<{ msg: string; seq: number } | null>(null);
const toastSeqRef = useRef(0);
const showToast = useCallback((msg: string) => {
    toastSeqRef.current += 1;
    setToast({ msg, seq: toastSeqRef.current });
}, []);
```
**Phase 26 D-26.1/D-26.4** adds in the same hook block:
```typescript
const [pendingInput, setPendingInput] = useState<PendingInput | null>(null);
```
where `PendingInput` is the discriminated union from D-26.4. NO IPC round-trip until end-of-modal — exactly mirrors the `shiftActive` pattern.

**Resolution-priority click handler pattern** (lines 161–206):
```typescript
const handleClick = useCallback(async (key: KeyDef) => {
    if (busyRef.current) return;
    // SHIFT key itself toggles state, no dispatch (rule 1 above).
    if (key.id === 'shift') {
        setShiftActive(prev => !prev);
        return;
    }
    // Resolve the effective id per rules 2-4.
    const alphaOn = calcState?.annunciators.alpha ?? false;
    let effectiveId: string;
    let consumesShift = false;
    if (alphaOn && key.alphaChar) {
        effectiveId = `alpha_${key.alphaChar}`;
    } else if (shiftActive && key.shifted) {
        effectiveId = key.shifted.id;
        consumesShift = true;
    } else {
        effectiveId = key.id;
    }
    if (!effectiveId) return;
    busyRef.current = true;
    try {
        let view: CalcStateView;
        if (effectiveId === 'clx_or_a') {
            const targetId = alphaOn ? 'alpha_clear' : 'clx';
            view = await invoke<CalcStateView>('dispatch_op', { keyId: targetId });
        } else {
            view = await invokeForKey(effectiveId);
        }
        setCalcState(view);
        setErrorMessage(null);
    } catch (err) {
        showToast(extractErrMessage(err));
    } finally {
        if (consumesShift) setShiftActive(false);
        busyRef.current = false;
    }
}, [calcState, shiftActive]);
```
**Phase 26 D-26.5 extension** — INSERT the modal-opener intercept BEFORE the busyRef block, AFTER effectiveId is computed:
```typescript
// Phase 26 D-26.5: modal-opener intercept — never reaches dispatch_op.
const MODAL_OPENERS: Record<string, (() => PendingInput)> = {
    'sto_prompt': () => ({ kind: 'register', op: 'Sto', ind: false, acc: '' }),
    'rcl_prompt': () => ({ kind: 'register', op: 'Rcl', ind: false, acc: '' }),
    'sf_prompt':  () => ({ kind: 'flag', testKind: 'SF', ind: false, acc: '' }),
    // ... 13 prompt ids + asn/view/catalog
};
if (MODAL_OPENERS[effectiveId]) {
    setPendingInput(MODAL_OPENERS[effectiveId]());
    if (consumesShift) setShiftActive(false);
    return;
}
```
The functional-setState form (`setShiftActive(prev => !prev)`) is REQUIRED to avoid stale-closure pitfall (Phase 19 D-?? Pitfall 4).

**`invokeForKey` helper pattern** (lines 46–53):
```typescript
async function invokeForKey(effectiveId: string): Promise<CalcStateView> {
    if (effectiveId === 'sst') return invoke<CalcStateView>('sst_step');
    if (effectiveId === 'bst') return invoke<CalcStateView>('bst_step');
    if (effectiveId === 'r_s') return invoke<CalcStateView>('run_stop');
    return invoke<CalcStateView>('dispatch_op', { keyId: effectiveId });
}
```
Phase 26: end-of-modal-accumulation calls `invokeForKey(finalParameterizedId)` — e.g. `invokeForKey('sto_05')`, `invokeForKey('sf_ind_12')`, `invokeForKey('clp_MYPRG')`, `invokeForKey('tone_5')`. NO new branches needed in `invokeForKey` (default `dispatch_op` route handles all parameterized ids).

**Error-extraction helper pattern** (lines 34–44):
```typescript
function extractErrMessage(err: unknown): string {
    if (typeof err === 'object' && err !== null) {
        if ('message' in err) return String((err as { message: unknown }).message);
        try { return JSON.stringify(err); } catch { /* fall through */ }
    }
    return String(err);
}
```
Reuse unchanged. End-of-modal dispatch errors flow through `showToast(extractErrMessage(err))`.

**Display-text derivation pattern** (line 289):
```typescript
<div className="display">{calcState.display_str}</div>
```
**Phase 26 D-26.3/D-26.7 swap:**
```typescript
const displayText = pendingInput
    ? renderModalLcd(pendingInput)
    : calcState.display_str;
// ...
<div className="display"><Display14Seg text={displayText} /></div>
```
Existing `.display` CSS (lines 41–52 in App.css) stays unchanged — `<Display14Seg>` renders inside the same div.

**Physical-keyboard MAP table pattern** (lines 82–96):
```typescript
const MAP: Record<string, string> = {
    'Enter': 'enter', 'Backspace': 'clx',
    '+': 'plus', '-': 'minus', '*': 'mul', '/': 'div',
    'r': 'rdn', 'x': 'xy_swap', 'l': 'lastx', 's': 'sqrt', 'p': 'prx',
    // ...
};
```
**Phase 26 D-26.10 swap:**
```typescript
'p': 'prgm_mode',  // was 'prx' — D-26.10 remap
'P': 'prx',        // SHIFT+P (case-detected) routes to PRX
```

**Tab/Esc one-shot consumption pattern** (lines 210–227):
```typescript
const handleKey = useCallback((e: KeyboardEvent) => {
    if (e.repeat) return;
    if (e.key === 'Escape') { setShiftActive(false); return; }
    if (e.key === 'Tab') { e.preventDefault(); setShiftActive(prev => !prev); return; }
    if (busyRef.current) return;
    const keyId = resolveKeyId(e, calcState);
    if (keyId === null) return;
    e.preventDefault();
    dispatchKeyId(keyId);
}, [calcState, dispatchKeyId]);
```
**Phase 26 extension:** Esc must ALSO clear `pendingInput`; physical-keyboard digits must route through `handleModalKey` if `pendingInput !== null`:
```typescript
if (e.key === 'Escape') {
    setShiftActive(false);
    setPendingInput(null);  // D-26.4: cancel any open modal
    return;
}
// After resolveKeyId, before dispatchKeyId:
if (pendingInput) {
    handleModalKey(e, pendingInput);  // routes to discriminated-union handler
    return;
}
```
ALPHA-overrides-SHIFT-overrides-modal precedence carries forward unchanged.

---

### `hp41-gui/src/Keyboard.tsx` (extend) — component (SVG), event-driven

**Analog:** self — `KeyDef` (lines 23–36) and `handleKeyClick` (lines 146–152) are the established patterns Phase 26 extends.

**`KeyDef` three-label model pattern** (lines 23–36):
```typescript
export type KeyDef = {
    id: string;                                  // primary op id (or '' for unwired)
    label: string;                               // primary visible label
    shifted?: { id: string; label: string };    // shifted op + orange label above
    alphaChar?: string;                          // ALPHA-mode character (blue label below)
    row: number;                                 // 0 = top-row band, 1..8 = main grid rows
    col: number;                                 // 0..4 within row
    colSpan?: number;                            // default 1 (ENTER spans 2)
    variant?: 'top' | 'shift' | 'enter';        // styling hint
};
```
**Phase 26 D-26.9 extension** — add optional `keyCode` field:
```typescript
    keyCode?: number;  // HP-41 row×10+col code; presence means USER-mode can relabel
```
Per Claude's Discretion in CONTEXT, planner picks hard-coded `keyCode` literals OR a `keyPosition`-style derivation. Hard-coding is more explicit; the deriving helper at lines 98–126 already computes (x,y,w,h) from (row,col), so a parallel `hp41KeyCode(row,col)` is straightforward.

**`KEY_DEFS` audit candidates** (lines 50–94 — current entries):
The 35-entry main grid covers v2.0 + v2.1 ops. Phase 26 audit MUST verify coverage of every new v2.2 named op that has a keyboard binding. Many will already be present via the `shifted` slot (e.g. `sf_prompt`, `cf_prompt`, `fix_prompt` lines 76–90). New entries needed: any v2.2 ROM op that lacks a current `shifted` slot reference (planner cross-checks against `docs/hp41cv-functions.json` `key_path` field).

**Click handler pattern** (lines 146–152):
```typescript
const handleKeyClick = (key: KeyDef) => {
    if (!key.id) return;                 // ON and other unwired keys
    if (busyRef.current) return;
    setPressedKey(key.id);
    setTimeout(() => setPressedKey(prev => (prev === key.id ? null : prev)), 150);
    onKey(key);
};
```
NO change in Phase 26 — the parent `handleClick` in `App.tsx` is where the modal-opener intercept happens. The 150ms `pressedKey` animation works for modal-opener clicks unchanged.

**USER-mode relabel pattern** (NEW per D-26.9, derive from existing label rendering at lines 259–269):
```typescript
{/* Primary label */}
<text
    x={x + w / 2}
    y={y + h / 2 + 5}
    textAnchor="middle"
    fill={labelColor}
    fontSize={key.variant === 'enter' ? 13 : 14}
    fontWeight="bold"
>
    {key.label}
</text>
```
**Phase 26 extension:** if USER mode is active AND `key.keyCode` is in the `userKeymap` prop, render the ASN'd label INSTEAD of `key.label`:
```typescript
const userLabel = userActive && key.keyCode != null
    ? userKeymap?.find(([code]) => code === key.keyCode)?.[1]
    : null;
// ...
{userLabel ?? key.label}
```
Pass `userKeymap` and `userActive` as new props on `KeyboardProps` (extend lines 136–141).

**Variant + grad pattern** (lines 128–134):
```typescript
function getKeyGrad(key: KeyDef, shiftActive: boolean): string {
    if (key.variant === 'shift') {
        return shiftActive ? 'url(#grad-shift-active)' : 'url(#grad-shift-idle)';
    }
    if (key.variant === 'enter') return 'url(#grad-enter)';
    return 'url(#grad-dark)';
}
```
USER-relabel does NOT need a new variant — keep the existing key cap; only swap the text content.

---

### `hp41-gui/src/Display14Seg.tsx` (CREATE) — component (SVG), render-only

**Analog:** `Keyboard.tsx` SVG-grid construction (lines 154–289) — same SVG `<svg viewBox>` + per-cell `<g>` + nested shape elements idiom.

**SVG-component skeleton pattern** (Keyboard.tsx lines 143–162, 195–289):
```typescript
export function Keyboard({ onKey, busyRef, shiftActive, alphaActive }: KeyboardProps) {
    const [pressedKey, setPressedKey] = useState<string | null>(null);
    return (
        <svg
            width="100%"
            viewBox={`0 0 ${KEYBOARD_W} ${KEYBOARD_H}`}
            xmlns="http://www.w3.org/2000/svg"
            aria-label="HP-41C keyboard"
        >
            <defs>
                <linearGradient id="body-grad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#1a1a1a" />
                    <stop offset="100%" stopColor="#000000" />
                </linearGradient>
                {/* ... more gradients ... */}
            </defs>
            <rect width={KEYBOARD_W} height={KEYBOARD_H} fill="url(#body-grad)" rx={10} />
            {KEY_DEFS.map(key => { /* render per-key */ })}
        </svg>
    );
}
```
**Phase 26 `<Display14Seg text={...} />`** mirrors this shape: a single `<svg viewBox>` containing 12 character cells, each with 14 `<path>` (or `<polygon>`) segments. Recommended structure:
```typescript
const SEGMENT_MAP: Record<string, number[]> = {
    'A': [0, 1, 2, 3, 4, 5, 6, 12],  // segment indices that are LIT for 'A'
    '0': [0, 1, 2, 3, 4, 5],
    // ... A-Z, 0-9, '.', ',', '-', '+', '(', ')', '=', '/', ':', ' ', '_'
};

const SEGMENT_PATHS: string[] = [
    /* 14 SVG path 'd' attributes — one per segment 0..13 */
];

export function Display14Seg({ text }: { text: string }) {
    const chars = text.padEnd(12, ' ').slice(0, 12);
    return (
        <svg viewBox={`0 0 ${CELL_W * 12} ${CELL_H}`} xmlns="...">
            {chars.split('').map((ch, i) => (
                <g key={i} transform={`translate(${i * CELL_W}, 0)`}>
                    {SEGMENT_PATHS.map((d, segIdx) => {
                        const lit = (SEGMENT_MAP[ch.toUpperCase()] ?? []).includes(segIdx);
                        return <path key={segIdx} d={d}
                            fill={lit ? '#a0ffa0' : '#1a3a1a'}
                            opacity={lit ? 1.0 : 0.1}  // D-26.6: dim 'off' segments
                        />;
                    })}
                </g>
            ))}
        </svg>
    );
}
```
**D-26.6 contract:** every cell renders all 14 segments unconditionally; lit/unlit is opacity-based (10% for 'off', 100% for 'on'). 12 cells × 14 segments = 168 paths per render — fine for human-scale refresh rate.

**Color palette to match existing display:** `.display` CSS (App.css lines 41–52) uses `color: #c8e6c9` for lit text. Use the same `#c8e6c9` family or the Phase 26 D-26.6 spec `#a0ffa0`-ish for lit segments.

**Trailing-cursor convention** (D-26.3 modal preview): `STO _5` shows the underscore as the digit-entry cursor — the same convention `format_entry_buf_display` in `hp41-cli/src/ui.rs` uses for EEX. The `text` prop already encodes the underscore; `SEGMENT_MAP['_']` lights only segment 13 (or whichever bottom-bar segment).

---

### `hp41-gui/src/HelpOverlay.tsx` (CREATE) — component (modal overlay), render + filter

**Analog:** `App.tsx` `print-panel` rendering (lines 329–342) and `prgm-panel` rendering (lines 310–328) — established overlay-div pattern.

**Overlay-div pattern** (App.tsx lines 329–342, App.css lines 111–158):
```typescript
{printPanelOpen && (
    <div className="print-panel">
        <div className="print-panel-header">
            <span>PRINT</span>
            <button className="print-panel-close" onClick={() => setPrintPanelOpen(false)}>×</button>
        </div>
        <div className="print-panel-content">
            {printLog.map((line, i) => (
                <div key={i} className="print-line">{line}</div>
            ))}
            <div ref={printEndRef} />
        </div>
    </div>
)}
```
**Phase 26 `<HelpOverlay open={...} onClose={...} />`** mirrors this:
```typescript
export function HelpOverlay({ open, onClose }: { open: boolean; onClose: () => void }) {
    const [query, setQuery] = useState('');
    const entries = helpEntries();  // from help_data.ts
    const filtered = entries.filter(e =>
        e.key_path != null && (
            query === '' ||
            e.display_name.toLowerCase().includes(query.toLowerCase()) ||
            e.description.toLowerCase().includes(query.toLowerCase()) ||
            e.category.toLowerCase().includes(query.toLowerCase())
        )
    );
    if (!open) return null;
    return (
        <div className="help-overlay" role="dialog" aria-label="HP-41 function reference">
            <div className="help-overlay-header">
                <input value={query} onChange={e => setQuery(e.target.value)}
                       placeholder="Search functions..." autoFocus />
                <button onClick={onClose}>×</button>
            </div>
            <div className="help-overlay-content">
                {/* group filtered by category, render rows */}
            </div>
        </div>
    );
}
```
**Filter rule per D-26.8:** drop entries with `key_path == null` (XEQ-by-Name-only ops aren't keyboard shortcuts). Group by `category` field for section headings — categories appear in JSON declaration order (mirror `hp41-cli/src/help_data.rs::help_overlay_rows` lines 95–121).

**Mount/unmount + Esc-close pattern** (App.tsx lines 230–233 useEffect for keyboard listener):
```typescript
useEffect(() => {
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
}, [handleKey]);
```
Phase 26: in `App.tsx`, listen for `?` (unshifted, NOT in ALPHA mode) to toggle `helpOpen`; listen for Esc inside `HelpOverlay` to close. The CLI Phase 25 `?` overlay narrowing (Plan 03 auto-fix) is the precedent: `?` opens overlay UNLESS a modal is currently accepting `?` as input (which the Phase 26 modals don't).

---

### `hp41-gui/src/help_data.ts` (CREATE) — data loader, file-I/O (vite import)

**Analog:** `hp41-cli/src/help_data.rs` (Rust → TS port). Same canonical data source `docs/hp41cv-functions.json`; same `HelpEntry` interface; same lazy-load + cache convention.

**Imports / interface pattern** (Rust analog at help_data.rs lines 22–57):
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HelpEntry {
    pub op_variant: String,
    pub display_name: String,
    pub category: String,
    pub status: String,
    pub phase: Option<String>,
    pub key_path: Option<String>,
    pub description: String,
    #[serde(default)]
    pub divergences: Vec<String>,
}
```
**TypeScript port** (Phase 26 D-26.8 `help_data.ts`):
```typescript
import functions from '../../docs/hp41cv-functions.json';

export interface HelpEntry {
    op_variant: string;
    display_name: string;
    category: string;
    status: 'implemented' | 'deferred-v3' | 'na';
    phase: string | null;
    key_path: string | null;
    description: string;
    divergences?: string[];
}

export function helpEntries(): readonly HelpEntry[] {
    return functions as readonly HelpEntry[];  // type assertion per D-26.8
}
```
**Vite JSON-import semantics** (D-25.16 + D-26.8): vite resolves `import functions from '../../docs/hp41cv-functions.json'` at build time — zero runtime fetch, zero filesystem access in the browser. The path `../../docs/...` is relative from `hp41-gui/src/help_data.ts` to `<repo>/docs/hp41cv-functions.json`.

**Lazy-init pattern** (Rust analog lines 64–77):
```rust
static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries() -> &'static [HelpEntry] {
    HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(FUNCTIONS_JSON)
            .expect("hp41cv-functions.json is malformed — fix the JSON")
    })
}
```
TypeScript equivalent: vite's static import is itself the cache (module evaluation is one-shot). No `OnceLock` needed; the `import` is the lazy-init.

**Hard-build-blocker semantics** (D-25.17 carried into D-26.8): if the JSON is malformed, vite's JSON loader fails the build. This matches the Rust `.expect(...)` semantics — a malformed canonical file is a hard-build-blocker by design.

**Helper functions to mirror from Rust** (lines 95–121):
```typescript
export function helpOverlayRows(): HelpRow[] {
    const entries = helpEntries();
    const categories: string[] = [];
    for (const entry of entries) {
        if (!categories.includes(entry.category)) categories.push(entry.category);
    }
    const rows: HelpRow[] = [];
    for (const cat of categories) {
        rows.push({ key: '', op: '', desc: `=== ${cat} ===` });
        for (const entry of entries.filter(e => e.category === cat)) {
            rows.push({
                key: entry.key_path ?? '',
                op: entry.display_name,
                desc: entry.description,
            });
        }
    }
    return rows;
}
```
Same first-appearance-order grouping as Rust (lines 96–102).

---

### `hp41-gui/src/App.css` (extend) — styles, render

**Analog:** self — `.toast` (lines 217–239) + `.prgm-panel` (lines 163–203) + `.print-panel` (lines 109–158) are the existing overlay-style precedents.

**Drop-in component preserves CSS pattern** (lines 41–52, the `.display` rule):
```css
.display {
  background: #111;
  padding: 6px 10px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 22px;
  text-align: right;
  color: #c8e6c9;
  letter-spacing: 0.05em;
  min-height: 2em;
  border-bottom: 1px solid #222;
  white-space: pre;
}
```
**Phase 26 D-26.7:** this rule stays unchanged. `<Display14Seg>` renders inside `.display` and respects its `min-height: 2em`. Add a child rule `.display svg` if the SVG needs sizing constraints, but keep `.display` itself untouched.

**Overlay-card style pattern** (lines 111–132, `.print-panel` + `.print-panel-header`):
```css
.print-panel {
  width: 100%;
  background: #1a1a1a;
  border-top: 1px solid #3a3a3a;
  border-radius: 0 0 8px 8px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 11px;
  overflow: hidden;
}

.print-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 8px;
  background: #252525;
  border-bottom: 1px solid #3a3a3a;
  color: #888;
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
```
**Phase 26 `.help-overlay`** mirrors this color palette + header structure. Difference: position `absolute` + cover the full `.calculator` (`top: 0; left: 0; right: 0; bottom: 0; z-index: 60` — above the toast's z-index 50).

**SHIFT-armed glow precedent** (lines 211–213) for Phase 26 USER-relabel highlighting:
```css
.key.key-shift-active {
  filter: brightness(1.18) drop-shadow(0 0 4px #f5a423);
}
```
Phase 26 may add `.key.key-user-relabel` with a similar filter if USER-mapped keys need visual distinction (Claude's Discretion — not mandated by D-26.9).

**Toast keyframe precedent** (lines 217–239) for any new modal-preview animation:
```css
@keyframes toast-fade {
  0%   { opacity: 0; transform: translate(-50%, -4px); }
  10%  { opacity: 1; transform: translate(-50%, 0); }
  85%  { opacity: 1; }
  100% { opacity: 0; transform: translate(-50%, -2px); }
}
```
Phase 26 likely doesn't need a new keyframe — modal-preview is just text content swap inside `.display`.

---

### `hp41-gui/src-tauri/permissions/<new-cmd>.toml` (CREATE, conditional) — config (Tauri permission), declarative

**Analog:** `permissions/run-stop.toml` (4 lines, exact template).

**Permission TOML pattern** (full file, 6 lines including blank):
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-run-stop"
description = "Allows the run_stop command."
commands.allow = ["run_stop"]
```
**Naming convention:** filename is the kebab-case command name (`run-stop.toml` for `run_stop` command); identifier is `allow-<kebab-case>`. Each command gets its OWN file — DO NOT bundle multiple commands per file.

**Capability registration** (`capabilities/default.json`):
```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "allow-dispatch-op",
    "allow-get-state",
    "allow-sst-step",
    "allow-bst-step",
    "allow-run-stop"
  ]
}
```
Add the new identifier to the `permissions` array. Run `cargo check` in `hp41-gui/src-tauri/` after creating the TOML so the permission registry regenerates (Tauri v2.11 build-time codegen).

**Default Phase 26 expectation:** ZERO new permission TOMLs (ASN routes through existing `dispatch_op`). Only create new files if planner introduces a dedicated command.

---

## Shared Patterns

### Frontend-owned UI state with backend round-trip on final dispatch

**Source:** `hp41-gui/src/App.tsx` lines 111–124 (`shiftActive` + `toast` + `showToast`)
**Apply to:** `pendingInput` state machine (D-26.1), `helpOpen` overlay state (D-26.8), USER-relabel relies on `calcState.user_keymap` (read-only from backend).

```typescript
const [shiftActive, setShiftActive] = useState(false);
// No corresponding field on CalcStateView — this is purely frontend.
```
**Invariant:** state that mirrors physical user-input intent (SHIFT armed, modal open, help overlay visible) lives in React. Only state that affects the calculator's mathematical/programmatic semantics crosses IPC. D-26.1 explicitly applies this rule to `pendingInput`.

### IPC error → toast pipeline

**Source:** `hp41-gui/src/App.tsx` lines 34–44 (`extractErrMessage`) + lines 119–132 (`toast` + auto-dismiss)
**Apply to:** all Phase 26 invoke calls — modal-end-dispatch, USER-mode toggles, ASN flow.

```typescript
.catch(err => showToast(extractErrMessage(err)))
```
**Critical detail:** the `seq` field in `{ msg, seq }` is REQUIRED for re-firing identical messages. Without it, two clicks on the same stub produce identical state and the auto-dismiss timer doesn't reset.

### Hardware-faithful one-shot bit reuse for IND-toggle

**Source:** `hp41-cli/src/app.rs` lines 1111–1124 (`check_ind_toggle`) — REFERENCE ONLY, port to TypeScript
**Apply to:** `handleModalKey` for `kind: 'flag'` and `kind: 'register'` arms (D-26.2).

```rust
fn check_ind_toggle(&mut self, key: KeyEvent) -> IndToggleAction {
    if !self.shift_armed
        && key.code == KeyCode::Char('f')
        && !key.modifiers.contains(KeyModifiers::CONTROL)
    {
        self.shift_armed = true;
        return IndToggleAction::ArmShift;
    }
    if self.shift_armed && key.code == KeyCode::Char('0') {
        self.shift_armed = false;
        return IndToggleAction::ToggleInd;
    }
    IndToggleAction::Continue
}
```
**TypeScript port (D-26.2):** inside `handleModalKey`, before digit-accumulation:
```typescript
// Mirror hp41-cli/src/app.rs lines 1111-1124 — D-26.2 / Pitfall 10.
if (!shiftActive && key === 'shift_press_intent_id') {
    setShiftActive(true);
    return; // re-store modal unchanged; next-key cycle is the toggle
}
if (shiftActive && key === '0' && (pending.kind === 'flag' || pending.kind === 'register')) {
    setShiftActive(false);
    setPendingInput({ ...pending, ind: !pending.ind });
    return;
}
// fall through to digit accumulation
```
SHIFT bit is REUSED — no separate `shiftPending` field. Bit-for-bit parity with CLI per D-25.6 / D-26.2.

### `#![deny(clippy::unwrap_used)]` enforcement

**Source:** `hp41-gui/src-tauri/src/lib.rs` line 1
**Apply to:** ALL Rust changes in `hp41-gui/src-tauri/`

```rust
#![deny(clippy::unwrap_used)]
```
Test modules can opt out via `#[allow(clippy::unwrap_used)]` (see key_map.rs line 219, types.rs line 115, commands.rs line 283). Production code uses `.expect("reason")`, `?`, or pattern-match. Mutex locks use `.unwrap_or_else(|e| e.into_inner())` (commands.rs lines 53, 68, 81, 233, 242, 253).

### Drain-print-buffer-before-from_state pattern

**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 211–215 (`handle_op_finalize`) and lines 263–264 (`handle_sst`), plus types.rs line 44 contract comment
**Apply to:** any new Phase 26 command helper; Phase 26 `event_buffer` field follows the same drain-then-pass model.

```rust
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
Ok(CalcStateView::from_state(calc, print_lines))
```
**RESEARCH Pitfall 1:** drain takes `&mut`, then `from_state` takes `&` — they cannot interleave. Phase 26's `event_buffer` extension means `from_state` likely takes `(state: &CalcState, print_lines: Vec<String>, event_lines: Vec<String>)` OR each command drains both buffers and passes both via parameters.

### Defense-in-depth stub arm + frontend intercept

**Source:** `hp41-gui/src-tauri/src/key_map.rs` lines 93–104 (stub arm) + `test_modal_prompt_ids_are_stubs_for_now` (lines 322–352)
**Apply to:** D-26.5 modal-opener intercept design

The 13 `*_prompt` ids continue to surface `GuiError` from `key_map::resolve()` (defense). The frontend MUST intercept them in `handleClick` BEFORE invoking `dispatch_op` (primary path). A regression in either layer surfaces as a toast — never silent. D-07 invariant preserved.

---

## No Analog Found

None. Every Phase 26 target file has a strong existing in-repo analog:
- The 14-seg LCD is the only file without a direct "calculator-display" precedent in the repo, but `Keyboard.tsx`'s SVG-grid construction provides the rendering idiom (per-cell `<g>` with nested shape primitives + lookup-table-driven fill).
- The TypeScript-side help_data is a port of `hp41-cli/src/help_data.rs` — same JSON file, same shape, different language.

---

## Metadata

**Analog search scope:**
- `hp41-gui/src/` (App.tsx, Keyboard.tsx, App.css)
- `hp41-gui/src-tauri/src/` (key_map.rs, types.rs, commands.rs, lib.rs, persistence.rs, prgm_display.rs, cards.rs)
- `hp41-gui/src-tauri/permissions/` (run-stop.toml, sst-step.toml, others)
- `hp41-gui/src-tauri/capabilities/default.json`
- `hp41-cli/src/app.rs` (PendingInput hybrid struct-variants — REFERENCE ONLY for TS port)
- `hp41-cli/src/keys.rs` (FlagPromptKind / RegisterOpKind — REFERENCE ONLY)
- `hp41-cli/src/ui.rs` (pending_prompt exhaustive match — REFERENCE ONLY for renderModalLcd)
- `hp41-cli/src/help_data.rs` (JSON pipeline — REFERENCE for help_data.ts)
- `docs/hp41cv-functions.json` (canonical data source — 1395 lines, vite-imported in Phase 26)

**Files scanned:** 16
**Pattern extraction date:** 2026-05-15

## PATTERN MAPPING COMPLETE
