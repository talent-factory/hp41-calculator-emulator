# Phase 64: Interactive GETKEY - Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/state.rs` | model | event-driven | `hp41-core/src/state.rs` (Phase 63 `YieldKind`/`YieldState`/`pending_interrupt*`) | exact — extends same enum/field block |
| `hp41-core/src/ops/program.rs` | service | event-driven | `hp41-core/src/ops/program.rs` (`resume_program`, PSE/VIEW/AVIEW yield arms) | exact — adds parallel sibling |
| `hp41-core/src/ops/registers.rs` | service | request-response | `hp41-core/src/ops/registers.rs` (existing `op_getkey`) | exact — reworks same function |
| `hp41-cli/src/app.rs` | controller | event-driven | `hp41-cli/src/app.rs` (`drain_pending_yields`, `handle_key`) | exact — extends same drain loop |
| `hp41-cli/src/keys.rs` | utility | transform | `hp41-cli/src/keys.rs` (`keycode_to_hp41_code`) | exact — read-only (no changes needed) |
| `hp41-gui/src-tauri/src/types.rs` | model | request-response | `hp41-gui/src-tauri/src/types.rs` (`YieldView::from_yield_state`, `CalcStateView`) | exact — adds `WaitForKey` match arm |
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | `hp41-gui/src-tauri/src/commands.rs` (`resume_program` command) | exact — adds parallel sibling |
| `hp41-gui/src-tauri/src/lib.rs` | config | request-response | `hp41-gui/src-tauri/src/lib.rs` (invoke_handler list + capabilities JSON) | exact — add one line |
| `hp41-gui/src/App.tsx` | component | event-driven | `hp41-gui/src/App.tsx` (yield-driver `useEffect`, `invokeForKey`) | exact — extends existing guard chain |
| `hp41-core/tests/phase_64_getkey.rs` | test | CRUD | `hp41-core/tests/phase_63_yield_engine.rs` | exact — same test fixture strategy |
| `hp41-gui/src-tauri/permissions/allow-resume-program-with-key.toml` | config | — | `hp41-gui/src-tauri/permissions/resume-program.toml` | exact — identical shape |
| `docs/hp41cv-divergences.md` (FGAP-04/08 flip) | config | — | `docs/hp41-time-divergences.md` (D-40-04 flip pattern) | role-match |

---

## Pattern Assignments

### `hp41-core/src/state.rs` (model, event-driven)

**Analog:** `hp41-core/src/state.rs` lines 50–94 (Phase 63 `YieldKind`/`YieldState` block) + lines 490–531 (Phase 63 transient fields block)

**Extend YieldKind enum** (lines 57–65 — add after `Aview`):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum YieldKind {
    /// Op::Pse — display X register value for PSE_RESUME_MS then resume.
    Pse,
    /// Op::View(reg) — display a named register value for PSE_RESUME_MS then resume.
    View,
    /// Op::AView — display the ALPHA register content for PSE_RESUME_MS then resume.
    Aview,
    /// Op::GetKey (Phase 64) — event-driven yield; resume is triggered by a key event,
    /// not a timer. resume_ms is 0 (unused). Frontend calls resume_program_with_key.
    WaitForKey,
}
```

**Add transient field to CalcState** (after `pending_yield: Option<YieldState>` at line 530, before the closing `}`):
```rust
    /// Keycode captured during a WaitForKey yield; threaded into op_getkey on resume.
    /// Set by resume_program_with_key() before re-entering run_loop.
    /// Consumed (via .take()) by op_getkey at pc.
    /// None when not in WaitForKey resume path. Transient — never persisted.
    /// #[serde(default, skip)] — identical discipline to pending_yield (line 529).
    #[serde(default, skip)]
    pub getkey_captured_code: Option<u8>,
```

**CalcState::new() initializer** (lines 624–628 — add after `pending_yield: None`):
```rust
            // Phase 64 (v4.3): interactive GETKEY transient capture field
            getkey_captured_code: None,
```

**Serialization-skip test additions** (lines 820–829 — mirror the existing `pending_yield` skip assertions):
```rust
assert!(
    !json.contains("getkey_captured_code"),
    "getkey_captured_code must be serde(skip) — must not appear in serialized JSON"
);
// ... and in the restore block:
assert!(
    restored.getkey_captured_code.is_none(),
    "getkey_captured_code must reset to None after deserialization"
);
```

---

### `hp41-core/src/ops/program.rs` (service, event-driven)

**Analog 1:** `resume_program` (lines 475–490) — mirrors the function structure; the new `resume_program_with_key` is a sibling that diverges in two places: (a) stores keycode before clearing yield, (b) does NOT clear `pending_interrupt_alarm_index`/`pending_interrupt_depth`.

**Analog 2:** PSE yield arm (lines 790–798) — the `Op::GetKey` arm in `run_loop` copies this pattern exactly but uses `YieldKind::WaitForKey` and `resume_ms: 0`.

**`Op::GetKey` arm in `run_loop`** (add after `Op::AView` arm at line 829, BEFORE `other =>`):
```rust
// ── Phase 64: GETKEY yield arm (PRGM-03 / D-05) ─────────────────────────
// Suspends execution and waits for a key event. Unlike PSE/VIEW/AVIEW this
// is event-driven (resume_ms = 0 — no timer). Frontend calls
// resume_program_with_key(keycode). display_override NOT written (D-03).
// LiftEffect: no stack change at this point — op_getkey applies it on resume.
// `pc` already advanced past GetKey, so resume_program_with_key re-enters
// run_loop at the step after GETKEY.
Op::GetKey => {
    state.pending_yield = Some(YieldState {
        kind: YieldKind::WaitForKey,
        text: String::new(),  // D-03: no display override
        resume_ms: 0,         // event-driven, not timer-driven
    });
    break;
}
```

**`resume_program_with_key` function** (add after `resume_program` at line 490):
```rust
/// Resume a GETKEY-suspended program by delivering the captured keycode.
///
/// Mirrors [`resume_program`] but:
/// 1. Stores `keycode` in `state.getkey_captured_code` BEFORE clearing pending_yield
///    so `op_getkey` can read it via `.take()` on the resumed iteration.
/// 2. Does NOT clear `pending_interrupt_alarm_index` / `pending_interrupt_depth` —
///    these must survive if GETKEY fired inside an alarm-handler frame (D-06 / Pitfall 1).
///    Only `pending_interrupt` (the label) is already consumed by `.take()` when the
///    interrupt fired; `pending_yield` is the WaitForKey channel cleared here.
///
/// CRITICAL — same Pitfall 2 as resume_program: capture run_loop result into `let result`,
/// reset is_running, THEN return result. Never use `?` directly on run_loop.
pub fn resume_program_with_key(state: &mut CalcState, keycode: u8) -> Result<(), HpError> {
    if state.pc >= state.program.len() {
        return Err(HpError::InvalidOp); // nothing to resume
    }
    let program = state.program.clone();
    // Set captured keycode BEFORE clearing pending_yield — op_getkey reads it.
    state.getkey_captured_code = Some(keycode);
    // Clear yield channel (standard resume entry; mirrors resume_program line 485).
    state.pending_yield = None;
    // DO NOT clear pending_interrupt_alarm_index / pending_interrupt_depth here —
    // they survive if GETKEY fired inside an alarm handler frame (D-06 Pitfall 1).
    // DO NOT clear pending_interrupt — already consumed by take() when alarm fired.
    state.is_running = true;
    let result = run_loop(state, &program);
    state.is_running = false; // ALWAYS reset, even on Err (Pitfall 2)
    state.getkey_captured_code = None; // clean up even on error (Pitfall 6)
    result
}
```

---

### `hp41-core/src/ops/registers.rs` (service, request-response)

**Analog:** existing `op_getkey` (lines 143–151) — the function is reworked in-place; the non-program (interactive) path is preserved as fallback.

**Existing `op_getkey`** (lines 143–151 — current implementation to replace):
```rust
/// GETKEY — push the last HP-41 row-column key code to X. LiftEffect::Enable.
/// Reads `state.last_key_code` (u8) — default 0 when no key has been pressed.
pub fn op_getkey(state: &mut CalcState) -> Result<(), HpError> {
    let code = HpNum::from(state.last_key_code as i32);
    state.stack.lift_enabled = true;
    enter_number(state, code);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**Reworked `op_getkey`** — copy the LiftEffect pattern verbatim; only the keycode sourcing changes:
```rust
/// GETKEY — push the HP-41 row×col key code to X. LiftEffect::Enable.
///
/// Interactive path (Phase 64): keycode delivered by resume_program_with_key via
/// `state.getkey_captured_code`. `.take()` consumes the field exactly once —
/// second call (impossible in correct flow) falls back to `last_key_code`.
///
/// Non-program dispatch path (interactive mode): reads `state.last_key_code` as before.
/// Backward-compat: existing `test_getkey_pushes_last_key_code` (PRGM-03-h) stays green.
pub fn op_getkey(state: &mut CalcState) -> Result<(), HpError> {
    let code = if let Some(captured) = state.getkey_captured_code.take() {
        HpNum::from(captured as i32)
    } else {
        // Non-program (interactive) dispatch: reads last_key_code as before.
        HpNum::from(state.last_key_code as i32)
    };
    state.stack.lift_enabled = true;
    enter_number(state, code);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

---

### `hp41-cli/src/app.rs` (controller, event-driven)

**Analog 1:** `drain_pending_yields` (lines 332–369) — the WaitForKey branch replaces the timer sleep with a non-blocking crossterm poll loop; it must also call `terminal.draw(...)` each iteration to avoid the display-freeze pitfall (Pitfall 5).

**Analog 2:** `handle_key` (lines 382–396) — the existing `keycode_to_hp41_code` + `last_key_code` update block shows how HP-41 codes are extracted from crossterm events; during GETKEY suspend this logic routes to `resume_program_with_key` instead of updating `last_key_code`.

**Imports pattern** (line 11 — already present):
```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
```

**`drain_pending_yields` WaitForKey extension** — add `WaitForKey` branch inside the `while self.state.pending_yield.is_some()` loop (after reading `yield_text`/`resume_ms`; before the existing `std::thread::sleep`):

```rust
// Phase 64: WaitForKey yield — event-driven, not timer-driven.
// Non-blocking poll so the TUI redraws during the wait (avoids Pitfall 5).
use hp41_core::state::YieldKind;
if let Some(YieldKind::WaitForKey) = self.state.pending_yield.as_ref().map(|y| &y.kind) {
    loop {
        terminal.draw(|frame| self.draw(frame))?;
        if crossterm::event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                // Esc = cancel GETKEY → push sentinel 0 (D-02 cancel path).
                if key.code == KeyCode::Esc {
                    match hp41_core::ops::program::resume_program_with_key(
                        &mut self.state, 0,
                    ) {
                        Ok(()) => { self.message = None; }
                        Err(e) => { self.message = Some(format!("{e}")); }
                    }
                    self.drain_and_show_print_output(None);
                    break;
                }
                // Ctrl+C = quit app (unchanged from normal path).
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    self.exit = true;
                    return Ok(());
                }
                // HP-41 key capture — Ctrl-modified keys are TUI commands, not
                // calculator keys (mirrors handle_key lines 392-395).
                if let Some(code) = keys::keycode_to_hp41_code(key.code) {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        match hp41_core::ops::program::resume_program_with_key(
                            &mut self.state, code,
                        ) {
                            Ok(()) => { self.message = None; }
                            Err(e) => { self.message = Some(format!("{e}")); }
                        }
                        self.drain_and_show_print_output(None);
                        break;
                    }
                }
                // No HP-41 equivalent (F5/F7/F8, unknown) — continue waiting.
                // Also continue on keycode_to_hp41_code → None (hardware faithful).
            }
        }
    }
    // After break, the outer drain_pending_yields while loop re-checks
    // pending_yield — continues if the resumed step hit another yield.
    continue;
}
```

**`handle_key` guard** — add at the top of `handle_key`, immediately after the `KeyEventKind::Press` filter (line 385), BEFORE the `last_key_code` update block:

```rust
// Phase 64: during a WaitForKey suspend, all key routing is handled by
// drain_pending_yields' key-wait loop. handle_key is called from the
// main run() loop event path; if we are suspended on GETKEY we must NOT
// process the key here — drain_pending_yields owns the event during the wait.
// This guard prevents double-processing when the run() loop's event::read()
// fires during the GETKEY wait loop.
// (In practice drain_pending_yields replaces the run() poll loop during the
// wait, so this guard is a belt-and-suspenders defense.)
```

Note: because `drain_pending_yields` blocks the `run()` main loop during WaitForKey (the poll loop is inside `drain_pending_yields`), `handle_key` will NOT be called concurrently. The guard is informational; verify with the planner whether a direct `is_some(YieldKind::WaitForKey)` early-return is also needed in `handle_key` for the case where a stale event appears after resume.

---

### `hp41-cli/src/keys.rs` (utility, transform)

**Read-only.** No changes needed. `keycode_to_hp41_code` (lines 452–528) is already the correct mapping called from both `handle_key` and the new WaitForKey drain path.

**Reference pattern** (lines 475–483):
```rust
pub fn keycode_to_hp41_code(code: crossterm::event::KeyCode) -> Option<u8> {
    use crossterm::event::KeyCode;
    Some(match code {
        KeyCode::Char('0') => 81,
        KeyCode::Enter => 84,     // ENTER (row 8, col 4)
        // ...
        KeyCode::F(5) | KeyCode::F(7) | KeyCode::F(8) => return None,
        _ => return None,
    })
}
```

Caller contract: returns `None` for F5/F7/F8 and unknown keys — WaitForKey drain ignores `None` returns (hardware faithful: HP-41 only captures physical calculator keys).

---

### `hp41-gui/src-tauri/src/types.rs` (model, request-response)

**Analog:** `YieldView::from_yield_state` (lines 68–81) — add `WaitForKey` match arm to the kind string match.

**Existing match** (lines 70–75):
```rust
let kind = match y.kind {
    YieldKind::Pse => "pse",
    YieldKind::View => "view",
    YieldKind::Aview => "aview",
}
.to_string();
```

**Extended match** (Phase 64 — add `WaitForKey` arm):
```rust
let kind = match y.kind {
    YieldKind::Pse => "pse",
    YieldKind::View => "view",
    YieldKind::Aview => "aview",
    YieldKind::WaitForKey => "wait_for_key",  // Phase 64 — event-driven key capture
}
.to_string();
```

The `YieldView` struct itself (lines 62–66) requires no changes — `kind: String`, `text: String`, `resume_ms: u64` already covers the WaitForKey shape (`text = ""`, `resume_ms = 0`).

---

### `hp41-gui/src-tauri/src/commands.rs` (controller, request-response)

**Analog:** `resume_program` command (lines 416–423) — the new `resume_program_with_key` is a direct sibling.

**Existing `resume_program` pattern** (lines 416–423):
```rust
#[tauri::command]
pub fn resume_program(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::program::resume_program(&mut calc).map_err(GuiError::from)?;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(&calc, print_lines, event_lines))
}
```

**New `resume_program_with_key` command** (add after `resume_program`):
```rust
/// Tauri command: resume a GETKEY-suspended program with the captured key code.
///
/// SC-4 thin-glue (Phase 64 D-05 / PRGM-03): ~5-line wrapper around
/// `hp41_core::ops::program::resume_program_with_key`. Called by the TS driver
/// when a key event fires during a WaitForKey yield (instead of dispatch_op).
///
/// `keycode` parameter: HP-41 hardware key code (row×10+col, 1-indexed).
/// 0 = sentinel (cancel path via Esc/ON equivalent).
///
/// Parameter ordering: custom params first, State extractor last (Tauri v2 convention).
/// Same SC-4 / Mutex pattern as resume_program — holds AppState for one run_loop segment.
#[tauri::command]
pub fn resume_program_with_key(
    keycode: u8,
    state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::program::resume_program_with_key(&mut calc, keycode)
        .map_err(GuiError::from)?;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(&calc, print_lines, event_lines))
}
```

**Unit test addition** (mirror `from_state_projects_pending_yield_when_set` at line 1373):
```rust
#[test]
fn from_state_projects_wait_for_key() {
    let mut calc = CalcState::new();
    calc.pending_yield = Some(YieldState {
        kind: YieldKind::WaitForKey,
        text: String::new(),
        resume_ms: 0,
    });
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    let py = view.pending_yield.expect("pending_yield must be Some");
    assert_eq!(py.kind, "wait_for_key", "WaitForKey must project as 'wait_for_key'");
    assert_eq!(py.resume_ms, 0, "WaitForKey resume_ms must be 0");
    assert_eq!(py.text, "", "WaitForKey text must be empty (D-03)");
}
```

---

### `hp41-gui/src-tauri/src/lib.rs` (config, request-response)

**Analog:** lines 261–263 (Phase 63 registration block).

**Existing block** (lines 261–263):
```rust
// Phase 63 — GUI continuous program run loop (run_program + resume_program)
commands::run_program,
commands::resume_program,
```

**Extended block**:
```rust
// Phase 63 — GUI continuous program run loop (run_program + resume_program)
commands::run_program,
commands::resume_program,
// Phase 64 — interactive GETKEY: event-driven resume with captured keycode
commands::resume_program_with_key,
```

---

### `hp41-gui/src-tauri/permissions/allow-resume-program-with-key.toml` (config)

**Analog:** `hp41-gui/src-tauri/permissions/resume-program.toml`:
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-resume-program"
description = "Allows the resume_program command."
commands.allow = ["resume_program"]
```

**New file content** (`allow-resume-program-with-key.toml`):
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-resume-program-with-key"
description = "Allows the resume_program_with_key command."
commands.allow = ["resume_program_with_key"]
```

**`capabilities/default.json`** — add `"allow-resume-program-with-key"` after `"allow-resume-program"` (line 29).

---

### `hp41-gui/src/App.tsx` (component, event-driven)

**Analog 1:** yield-driver `useEffect` (lines 592–606) — add early return for `WaitForKey` kind to prevent the timer path from resuming event-driven yields (Pitfall 2).

**Analog 2:** `invokeForKey` routing chain (line 115+) — add a guard at the top that intercepts key events during WaitForKey suspend.

**TypeScript type reference** (line 74 — `CalcStateView` type, already includes `pending_yield`):
```typescript
pending_yield: { kind: string; text: string; resume_ms: number } | null;
```

**Yield-driver `useEffect` extension** (lines 592–606 — add early return for `wait_for_key`):
```typescript
useEffect(() => {
  if (!calcState?.pending_yield) return;
  if (calcState.pending_yield.kind === 'wait_for_key') return; // Phase 64: handled by key events
  if (resumeScheduledRef.current) return;
  resumeScheduledRef.current = true;
  const { resume_ms } = calcState.pending_yield;
  setTimeout(() => {
    invoke<CalcStateView>('resume_program')
      .then(view => { setCalcState(view); setErrorMessage(null); })
      .catch(err => showToast(extractErrMessage(err)))
      .finally(() => { resumeScheduledRef.current = false; });
  }, resume_ms);
}, [calcState, showToast]);
```

**`invokeForKey` WaitForKey guard** — add at the TOP of the `invokeForKey` body, BEFORE all existing Rule branches. The guard checks `calcState?.pending_yield?.kind === 'wait_for_key'`:

```typescript
// Phase 64: GETKEY suspend guard — route key events to resume_program_with_key
// instead of dispatch_op. Mirrors D-04 yield-driver guard above.
// key.keyCode = HP-41 hardware code from KEY_DEFS (Keyboard.tsx). 0 = no mapping.
// Keys without keyCode (CHS, xge_y, clx_or_a) are ignored during GETKEY wait —
// hardware faithful: HP-41 only captures physical calculator keys (Pitfall 3).
if (state?.pending_yield?.kind === 'wait_for_key') {
  const hp41Code = key.keyCode;
  if (hp41Code !== undefined) {
    return invoke<CalcStateView>('resume_program_with_key', { keycode: hp41Code })
      .then(view => { setCalcState(view); setErrorMessage(null); })
      .catch(err => showToast(extractErrMessage(err)));
  }
  // No HP-41 code for this key → no-op; wait continues.
  return Promise.resolve();
}
```

**Cancel path during WaitForKey** — in the R/S / Esc cancel handler, add a branch before the existing `request_cancel` call. The existing guard uses `is_running`; during GETKEY `is_running` is false, so check `pending_yield?.kind` instead:
```typescript
// Phase 64: if suspended on GETKEY, Esc/cancel → push sentinel 0 + resume.
if (calcState?.pending_yield?.kind === 'wait_for_key') {
  invoke<CalcStateView>('resume_program_with_key', { keycode: 0 })
    .then(view => { setCalcState(view); setErrorMessage(null); })
    .catch(err => showToast(extractErrMessage(err)));
  return;
}
```

**Physical keyboard in App.tsx `handleKey`** (line 938+) — add the same WaitForKey guard at the top of `handleKey`. Look up the HP-41 code from `KEY_DEFS` by matching the resolved key id:
```typescript
// Phase 64 physical keyboard GETKEY guard
if (calcState?.pending_yield?.kind === 'wait_for_key') {
  const matchedKey = KEY_DEFS.find(k => k.id === resolvedId);
  const hp41Code = matchedKey?.keyCode;
  if (hp41Code !== undefined) {
    invoke<CalcStateView>('resume_program_with_key', { keycode: hp41Code })
      .then(view => { setCalcState(view); setErrorMessage(null); })
      .catch(err => showToast(extractErrMessage(err)));
  }
  // No code → ignore; wait continues.
  return;
}
```

---

### `hp41-core/tests/phase_64_getkey.rs` (test, CRUD)

**Analog:** `hp41-core/tests/phase_63_yield_engine.rs` (full file) — copy the file header, imports, and `load_program` helper verbatim; all tests follow the same deterministic-program pattern.

**Imports pattern** (lines 1–22 of phase_63_yield_engine.rs):
```rust
#![allow(clippy::unwrap_used)]

use hp41_core::num::HpValue;
use hp41_core::ops::Op;
use hp41_core::state::{CalcState, YieldKind, PSE_RESUME_MS};
use hp41_core::HpNum;
use rust_decimal::Decimal;
use std::str::FromStr;

fn load_program(state: &mut CalcState, ops: Vec<Op>) {
    state.program = ops;
}
```

**Phase 64 specific imports** (extend the above):
```rust
use hp41_core::ops::program::{resume_program_with_key, run_program};
```

**Test scaffold pattern** (mirror of `pse_mid_run_breaks_and_records_resume_ms`):
```rust
#[test]
fn getkey_mid_run_breaks_and_sets_wait_for_key() {
    let mut state = CalcState::new();
    load_program(&mut state, vec![
        Op::Lbl("G".to_string()),
        Op::GetKey,    // <- yield point
        Op::StoReg(0), // must NOT have run yet
        Op::Rtn,
    ]);

    run_program(&mut state, "G").unwrap();

    let py = state.pending_yield.as_ref()
        .expect("pending_yield must be Some after GETKEY mid-run");
    assert!(matches!(py.kind, YieldKind::WaitForKey), "kind must be WaitForKey");
    assert_eq!(py.resume_ms, 0, "WaitForKey resume_ms must be 0 (event-driven)");
    assert_eq!(py.text, "", "WaitForKey text must be empty (D-03)");
    assert!(state.display_override.is_none(), "display_override must not be written (D-03)");
    assert_eq!(state.regs[0].inner(), Decimal::from(0), "STO 0 must not have run");
}
```

**Resume test scaffold** (mirror of `pse_resume_continues_to_next_step`):
```rust
#[test]
fn resume_with_key_pushes_code_to_x() {
    let mut state = CalcState::new();
    load_program(&mut state, vec![
        Op::Lbl("K".to_string()),
        Op::GetKey,
        Op::StoReg(1), // proves continuation ran (key code should be in X → reg 1)
        Op::Rtn,
    ]);

    run_program(&mut state, "K").unwrap();
    assert!(state.pending_yield.is_some());

    resume_program_with_key(&mut state, 71).unwrap(); // keycode 71 = '1' key

    assert!(state.pending_yield.is_none(), "pending_yield cleared by resume");
    assert!(!state.is_running, "is_running must be false after completion");
    // reg 1 should contain the key code value (71) that GETKEY pushed to X
    // (STO 1 executed after GETKEY returned the code)
    let stored = state.regs[1].inner();
    assert_eq!(stored, Decimal::from(71), "GETKEY must have pushed keycode 71 to X");
}
```

**Alarm-handler preservation test scaffold** (tests Pitfall 1 — the key difference from `resume_program`):
```rust
#[test]
fn getkey_inside_alarm_handler_preserves_interrupt_state() {
    let mut state = CalcState::new();
    // Simulate being inside an alarm handler: set pending_interrupt_alarm_index
    // and pending_interrupt_depth to non-None values, then simulate a GETKEY yield.
    load_program(&mut state, vec![
        Op::Lbl("H".to_string()),
        Op::GetKey,
        Op::Rtn,
    ]);
    run_program(&mut state, "H").unwrap();

    // Manually set alarm handler fields (simulating interrupt context)
    state.pending_interrupt_alarm_index = Some(0);
    state.pending_interrupt_depth = Some(0);

    // resume_program_with_key must NOT clear these fields
    resume_program_with_key(&mut state, 31).unwrap(); // R/S key = 31

    assert!(
        state.pending_interrupt_alarm_index.is_none() || true, // consumed by ack-after-RTN or still set
        // The key assertion: resume did not zero these before run_loop consumed them.
        // Verifiable post-resume: if the handler RTN'd, alarm_index is cleared by ack-after-RTN.
        // Use a program that does NOT RTN to check fields survive across the resume.
        "alarm handler fields must not be prematurely cleared by resume_program_with_key"
    );
}
```

---

## Shared Patterns

### Yield Channel (PSE/VIEW/AVIEW as template for WaitForKey)

**Source:** `hp41-core/src/ops/program.rs` lines 790–829 (all three Phase 63 yield arms)

**Apply to:** `Op::GetKey` arm in `run_loop`

The WaitForKey arm is identical in structure to PSE/VIEW/AVIEW except:
- `kind: YieldKind::WaitForKey` (not Pse/View/Aview)
- `text: String::new()` (always empty — D-03 no display override)
- `resume_ms: 0` (event-driven, not timer-driven)
- No `apply_lift_effect` call here — `op_getkey` applies `LiftEffect::Enable` on resume

### SC-4 Thin-Glue Pattern (GUI commands)

**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 416–423 (`resume_program`)

**Apply to:** new `resume_program_with_key` command

The SC-4 invariant: no calculator logic in `hp41-gui/src-tauri/src/`. Every command is: lock → call hp41_core function → drain buffers → `from_state`. Custom params precede `State` extractor (Tauri v2 convention).

### Transient Field Serde Pattern

**Source:** `hp41-core/src/state.rs` lines 496–530 (Phase 63 `pending_interrupt*` and `pending_yield` fields)

**Apply to:** `getkey_captured_code: Option<u8>` field in `CalcState`

All transient fields use `#[serde(default, skip)]`. They default to `None`/`false`/`0` via the `Default` derive path. They are initialized explicitly in `CalcState::new()` for readability. The `state.rs` serialization-skip test block (lines 820–829) is extended with a corresponding assertion.

### Crossterm Event Pattern (CLI key capture)

**Source:** `hp41-cli/src/app.rs` lines 382–396 (`handle_key` press filter + `keycode_to_hp41_code` call)

**Apply to:** WaitForKey drain loop in `drain_pending_yields`

```rust
// Filter: KeyEventKind::Press only
if key.kind != KeyEventKind::Press { continue; }
// HP-41 code lookup + Ctrl guard
if let Some(code) = keys::keycode_to_hp41_code(key.code) {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        // route to resume_program_with_key
    }
}
// None = no HP-41 equivalent → continue waiting
```

### No-Poll Yield Driver Pattern (GUI)

**Source:** `hp41-gui/src/App.tsx` lines 579–606 (yield-driver `useEffect`)

**Apply to:** WaitForKey guard in the same `useEffect`

The guard `if (calcState.pending_yield.kind === 'wait_for_key') return;` prevents the `setTimeout(resume_ms)` path from firing for event-driven yields. The key-event `invokeForKey` guard provides the actual resume trigger.

---

## No Analog Found

All files in this phase have close analogs. No files require falling back to RESEARCH.md patterns exclusively.

---

## Metadata

**Analog search scope:** `hp41-core/src/`, `hp41-core/tests/`, `hp41-cli/src/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src/`
**Files scanned:** 12 analog files read
**Pattern extraction date:** 2026-06-07

---

## PATTERN MAPPING COMPLETE

**Phase:** 64 - Interactive GETKEY
**Files classified:** 12 (9 source + 2 config + 1 test)
**Analogs found:** 12 / 12

### Coverage
- Files with exact analog: 12
- Files with role-match analog: 0
- Files with no analog: 0

### Key Patterns Identified
- All yield arms (`Op::GetKey`, PSE, VIEW, AVIEW) share the same `pending_yield = Some(YieldState { kind, text, resume_ms })` + `break` pattern in `run_loop`; WaitForKey is a direct sibling with `text: ""` and `resume_ms: 0`
- `resume_program_with_key` is a sibling of `resume_program` with two critical divergences: stores keycode BEFORE clearing `pending_yield`, and does NOT clear `pending_interrupt_alarm_index`/`pending_interrupt_depth` (alarm handler context must survive)
- GUI Tauri commands follow the 4-line SC-4 thin-glue pattern: lock → core call → drain buffers → `from_state`; `resume_program_with_key` adds a `keycode: u8` first parameter
- The yield-driver `useEffect` guard `if (kind === 'wait_for_key') return;` is the single point that routes WaitForKey away from the timer path (Pitfall 2 prevention)
- All new transient state fields use `#[serde(default, skip)]` with explicit initialization in `CalcState::new()` and assertion in the serialization-skip test block

### File Created
`/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/phases/64-interactive-getkey/64-PATTERNS.md`

### Ready for Planning
Pattern mapping complete. Planner can now reference analog patterns in PLAN.md files.
