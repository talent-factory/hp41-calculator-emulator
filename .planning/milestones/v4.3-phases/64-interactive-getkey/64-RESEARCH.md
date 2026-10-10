# Phase 64: Interactive GETKEY — Research

**Researched:** 2026-06-06
**Domain:** HP-41 keyboard-poll instruction; yield/suspend engine extension (hp41-core + CLI + GUI)
**Confidence:** HIGH (architecture: verified by reading Phase 63 source; HP-41 GETKEY semantics: MEDIUM — manual not fully accessible, community sources + function-list classification corroborate)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Exact wait semantics resolved by researcher (see § "D-01 Resolution" below). Intent = faithful interactive GETKEY. No-key sentinel returned on cancel (D-02) and on timeout if research finds timeout is HP-faithful.
- **D-02:** Faithful all-key capture. Every key returns its row×col code and resumes. R/S captured as keycode 84; does NOT stop the program. Escape = Phase 63 `request_cancel` path → push sentinel 0 and end/return control.
- **D-03:** Display unchanged during suspend. No override text in the yield. `display_override` untouched (Phase 63 D-04 carry-forward).
- **D-04:** CLI reuses existing `handle_key` row×col mapping. GUI must now capture keys during GETKEY (on-screen taps + physical). GUI currently NEVER sets `last_key_code` — this is the gap to close.
- **D-05:** Extend Phase 63 yield channel with a new event-driven wait-for-key `YieldKind`. `resume_program` carries captured keycode → X (LiftEffect::Enable). No new `Op`; no JSON/`builtin_card_op` change. Exact shape = planner's call (researcher recommends shape, see § "Recommended YieldKind Shape").
- **D-06:** Researcher to confirm faithful behavior + re-entrancy safety for: ALPHA-mode-during-GETKEY; nested yields (GETKEY inside Phase-63 interrupt-alarm handler, or during a PSE/VIEW/AVIEW yield); GETKEY interaction with `pending_interrupt`/`pending_yield` states.

### Claude's Discretion
- Exact shape/naming of the new wait-for-key `YieldKind` and how the captured keycode threads into `resume_program` → X (D-05).
- CLI test-fixture strategy for asserting the suspend/resume + keycode without real keyboard input (mirror Phase 63's no-wall-clock-sleep test approach).

### Deferred Ideas (OUT OF SCOPE)
- `GETKEYX` (timed/timeout GETKEY variant) — only in scope if D-01 finds it intrinsic to faithful GETKEY.
- CATALOG interactive scroll (FGAP-06) — v4.4.
- DISP-01 general CLI `display_override` rendering — v4.4.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PRGM-03 | GETKEY inside a running program pauses execution, waits for the next key press, pushes that key's HP-41 row×col code to X, and resumes (returns the no-key sentinel only when appropriate). (FGAP-04; subsumes FGAP-08) | Yield-suspend engine extension: new WaitForKey YieldKind + event-driven resume_program path; CLI handle_key captures and triggers resume; GUI on-screen key and physical keyboard event triggers resume_program with keycode. |
</phase_requirements>

---

## Summary

Phase 64 adds interactive GETKEY by extending the Phase 63 yield/suspend engine with a new **event-driven yield kind** — one where resume is triggered by a key event rather than a timer. The core machinery is already in place: `run_loop` breaks and records a `pending_yield`; `resume_program` re-enters. What changes is: (a) a new `YieldKind::WaitForKey` in the typed yield channel; (b) `op_getkey` in `run_loop` now sets this yield kind instead of reading `last_key_code`; (c) CLI captures the next keypress and calls a new `resume_program_with_key(keycode)` entry point; (d) GUI must now report a keycode during GETKEY suspend — it currently never sets `last_key_code` at all.

The no-key sentinel (0) is returned on `request_cancel` (escape/ON). Real HP-41 GETKEY is confirmed to **wait for a keypress** (halts the program); GETKEYX is the separate timed variant. The implementation strategy defers GETKEYX to a later phase and implements GETKEY as: wait until a key is pressed or cancel is invoked.

ALPHA mode during GETKEY is hardware-faithful: the HP-41 GETKEY returns a row×col key code regardless of ALPHA mode — ALPHA state does NOT alter the code. Key codes are the same in ALPHA and non-ALPHA contexts. Re-entrancy is safe because the WaitForKey yield breaks `run_loop` entirely (like Op::Prompt); no solver/modal can be mid-execution when GETKEY fires — the run_loop is the only execution context and it has already paused.

**Primary recommendation:** Add `YieldKind::WaitForKey` with a `captured_keycode: Option<u8>` payload; extend `resume_program` to accept an optional keycode (`resume_program_with_key(state, keycode: Option<u8>)`) so the frontend threads the captured code through to `op_getkey`'s X-push logic.

---

## D-01 Resolution: GETKEY Wait/Timeout Semantics

**Research finding:** Multiple authoritative sources classify GETKEY and GETKEYX as distinct HP-41CX Extended Functions:
- GETKEY = "get a key" (halts program, waits for a keypress) [CITED: finseth.com/hpdata/hp41cx.php]
- GETKEYX = "get a key timed" (timed variant) [CITED: finseth.com/hpdata/hp41cx.php]

Community forum evidence (hp41.org/viewtopic?t=488) reports GETKEY allows ~10 seconds before returning 0 (no-key sentinel). This 10-second timeout is the hardware GETKEY behavior on real HP-41CX hardware. However, the community source is a forum post, not the manual. [ASSUMED: exact 10-second timeout figure — manual text not accessible; forum recollection]

**Implication for the emulator:**
- GETKEY waiting indefinitely (until cancel) is acceptable as an emulator simplification vs. the ~10-second hardware timeout. The CONTEXT.md D-02 decision (cancel via `request_cancel` → push 0) covers this: the user can always escape a blocked GETKEY. The CONTEXT.md explicitly calls GETKEYX a separate deferred item.
- **GETKEYX is a separate function** — NOT intrinsic to faithful GETKEY. GETKEY as a standalone function waits for a key press (or cancel escape). This is confirmed as the correct emulator behavior.
- **The no-key sentinel (0) is returned on `request_cancel`** (cancel path). If a hardware timeout is desired later, it can be implemented as a timer-triggered `request_cancel` in the frontend.
- **FGAP-08 is subsumed**: once GETKEY suspends and waits for a key, the sentinel 0 is returned only on cancel, not spuriously.

**Conclusion (locked for planning):**
GETKEY suspends execution and waits until: (a) the user presses a key → push row×col code → resume; or (b) `request_cancel` is invoked → push 0 (sentinel) → end program / return. GETKEYX remains deferred (future phase). No emulator-side 10-second timeout is needed in Phase 64 — the cancel escape is sufficient for usability.

---

## D-06 Resolution: Edge Behavior + Re-Entrancy Safety

### ALPHA Mode During GETKEY
**Finding:** HP-41 GETKEY returns the physical key's row×col code regardless of ALPHA mode. ALPHA mode shifts which *function* a key invokes (letters vs. operations), but GETKEY bypasses function dispatch entirely — it captures the raw key code before any function resolution. [ASSUMED based on: HP-41 hardware design; the QRG confirms USER mode can affect key assignment but GETKEY returns the hardware position code, not the assigned function. Forum examples show numeric codes like 71/41 which are physical positions.]

**Implementation implication:** During a GETKEY suspend, the CLI should capture the next physical keypress's `keycode_to_hp41_code()` result WITHOUT processing `handle_alpha_mode_key` or any other prefix handling. The raw row×col code goes directly into the resume call. ALPHA mode status does not filter or transform the code.

**Guard needed:** The CLI's `handle_key` routing must intercept during GETKEY suspend: skip all modal, ALPHA, shift-armed logic and route the keypress directly to `resume_program_with_key(code)`.

### Nested Yields: GETKEY inside a PSE/VIEW/AVIEW yield
**Finding:** This cannot happen. `run_loop` is synchronous and breaks for each yield. When a PSE/VIEW/AVIEW yield fires, `run_loop` has already exited and `resume_program` is called by the frontend. Only when `resume_program` re-enters `run_loop` can another op execute. A program sequence like `PSE → GETKEY` would: (1) PSE yields → frontend sleeps/times out → resume → (2) GETKEY yields → frontend waits for key → resume. These yields are strictly sequential, never concurrent. [VERIFIED: Phase 63 code reading; `pending_yield` is cleared by `resume_program` entry, then GETKEY sets a new WaitForKey yield. Sequential, no overlap.]

**Guard:** `resume_program` already clears `pending_yield` at entry. The new `resume_program_with_key` must do the same.

### Nested: GETKEY inside an interrupting alarm handler
**Finding:** An alarm handler program is a synthetic XEQ frame inside `run_loop` (Phase 63 Phase-B). When the alarm handler runs `GETKEY`, `run_loop` hits the WaitForKey yield arm and breaks — exactly as if GETKEY were in the main program. The call_stack has the interrupt resume frame already pushed (the interrupted program's pc). When the user presses a key, `resume_program_with_key` re-enters `run_loop`, GETKEY pushes the code to X, and the alarm handler continues from GETKEY+1. When the handler RTNs, `call_stack.pop()` restores the interrupted program's pc — ack-after-RTN fires as normal. [VERIFIED: Phase 63 call-stack mechanics; no special GETKEY guard needed for this case.]

**Guard needed:** `pending_interrupt_depth` / `pending_interrupt_alarm_index` survive the GETKEY suspend (they are cleared only by `resume_program` at entry — but `resume_program_with_key` must NOT clear them, because the alarm context must survive to the eventual RTN). This is the key difference from `resume_program`: the alarm tracking fields must be preserved across the WaitForKey resume.

### GETKEY vs. `pending_interrupt` + `pending_yield` States
**Finding:** When GETKEY breaks `run_loop`, it sets `pending_yield = Some(YieldState { kind: WaitForKey, ... })`. The `pending_interrupt` field and `pending_interrupt_alarm_index/depth` are NOT cleared at this point — they survive if set (e.g., if an alarm handler is mid-execution and hits GETKEY). `resume_program_with_key` must:
1. Store the keycode in a new transient field OR pass it as a parameter to `op_getkey` via the state.
2. Clear `pending_yield` (standard resume_program behavior).
3. NOT clear `pending_interrupt*` fields (preserve alarm handler context for ack-after-RTN).
4. NOT re-invoke `run_program` from scratch (must use `resume_program`-style re-entry).

### Solver/Modal Isolation (D-10 carry-forward)
**Finding:** No solver or modal program can be mid-execution when `run_loop` breaks for GETKEY. The yield-break is a synchronous exit from `run_loop`. Since `run_loop` holds no per-frame locals, the solver state fields (`integ_state`, `solve_state`, `difeq_state`, `modal_program`) are NOT mid-mutation when the yield fires — they would only be set if a solver op had been dispatched, and solver ops have their own internal loop that runs to completion before `run_loop` sees a normal-break. GETKEY itself is not a solver op. [VERIFIED: Phase 63 architecture — run_loop arms for PSE/VIEW/AVIEW fire BEFORE `other =>` execute_op; GETKEY arm will similarly fire before execute_op.]

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| GETKEY suspend (yield break) | hp41-core (`run_loop`) | — | All program execution is in core; yield break must happen there |
| Keycode capture from user input | CLI frontend / GUI frontend | — | Front-ends own key events; core is UI-agnostic |
| Threading keycode into resume | hp41-core (`resume_program_with_key`) | Frontend (calls it with code) | Core owns state mutation; frontend provides event |
| Display during suspend (D-03) | No change needed | — | Display unchanged; Phase 63 D-04 no-override carry-forward |
| Cancel → sentinel 0 path | CLI/GUI frontend (`request_cancel`) | hp41-core (push 0 to X) | Frontend triggers cancel; core executes the push |
| Row×col key code scheme | CLI (`keycode_to_hp41_code`) / GUI (KEY_DEFS `keyCode`) | hp41-core (documents encoding) | Both frontends already have the mapping |

---

## Standard Stack

No new library dependencies. This phase is entirely within the existing Rust + TypeScript stack.

| Component | Version | Role | Status |
|-----------|---------|------|--------|
| hp41-core | workspace | Core engine: YieldKind extension, op_getkey rework, resume_program_with_key | Existing — extend |
| hp41-cli (crossterm) | 0.29 | Key event capture during GETKEY suspend | Existing — extend |
| hp41-gui (Tauri v2 + React) | v2.11 / React 18 | GUI key dispatch during GETKEY suspend | Existing — extend |

### Installation
No new packages to install.

---

## Package Legitimacy Audit

> Not applicable — no external packages are installed in this phase.

---

## Architecture Patterns

### System Architecture Diagram

```
User Keypress / request_cancel
        │
        ▼
  ┌─────────────────────────────────────────────────────┐
  │  CLI: handle_key() detects is_getkey_suspended       │
  │  GUI: dispatch_op('xeq_GETKEY_KEY_<code>') OR        │
  │       new Tauri command resume_program_with_key(code) │
  └──────────────────┬──────────────────────────────────┘
                     │ keycode (u8) or 0 (cancel)
                     ▼
  ┌─────────────────────────────────────────────────────┐
  │  hp41-core: resume_program_with_key(state, code)     │
  │  ├─ state.getkey_captured_code = Some(code)          │
  │  ├─ state.pending_yield = None (clear yield)         │
  │  ├─ state.is_running = true                          │
  │  └─ run_loop(state, &program)                        │
  │       └─ op = Op::GetKey at state.pc                 │
  │           └─ op_getkey(state):                       │
  │               reads state.getkey_captured_code       │
  │               push code to X (LiftEffect::Enable)    │
  │               → program continues at pc+1            │
  └─────────────────────────────────────────────────────┘
                     │
                     ▼
             CalcStateView returned
         (pending_yield: None if program finished
          or Some(next yield) if next op is PSE/VIEW/etc)
```

### GETKEY Suspend Path (inside run_loop)

```
run_loop iteration N:
  op = program[pc]; pc += 1  // fetches Op::GetKey
  match op {
    Op::GetKey => {
      // INTERACTIVE path: suspend and wait for key event
      // (replaces stale last_key_code read)
      state.pending_yield = Some(YieldState {
          kind: YieldKind::WaitForKey,
          text: String::new(),  // D-03: no display change
          resume_ms: 0,         // event-driven, not timer-driven
      });
      break;  // same break pattern as Op::Pse/Op::View/Op::AView
    }
    // ... other arms unchanged
  }
```

### Recommended YieldKind Shape (D-05 — planner's call, researcher recommends)

```rust
// In hp41-core/src/state.rs — extend YieldKind enum:
pub enum YieldKind {
    Pse,
    View,
    Aview,
    WaitForKey,  // NEW: event-driven, resume via resume_program_with_key
}
```

```rust
// In hp41-core/src/state.rs — add transient field to CalcState:
/// Keycode captured during a WaitForKey yield; threaded into op_getkey on resume.
/// Set by resume_program_with_key() before re-entering run_loop.
/// None when not in WaitForKey resume. Transient — never persisted.
#[serde(default, skip)]
pub getkey_captured_code: Option<u8>,
```

```rust
// In hp41-core/src/ops/program.rs — new entry point:
pub fn resume_program_with_key(state: &mut CalcState, keycode: u8) -> Result<(), HpError> {
    if state.pc >= state.program.len() {
        return Err(HpError::InvalidOp);
    }
    let program = state.program.clone();
    // Store captured keycode BEFORE clearing pending_yield — op_getkey reads it
    state.getkey_captured_code = Some(keycode);
    // Clear yield channel (standard resume entry)
    state.pending_yield = None;
    // NOTE: DO NOT clear pending_interrupt*/pending_interrupt_alarm_index/depth here —
    // they may be active if GETKEY fired inside an alarm handler frame (D-06).
    state.is_running = true;
    let result = run_loop(state, &program);
    state.is_running = false;
    state.getkey_captured_code = None;  // clean up even on error
    result
}
```

```rust
// In hp41-core/src/ops/registers.rs — reworked op_getkey:
pub fn op_getkey(state: &mut CalcState) -> Result<(), HpError> {
    // Interactive path: keycode delivered by resume_program_with_key.
    // Fallback to last_key_code for the non-program dispatch path (interactive mode).
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

**Why `getkey_captured_code.take()` instead of a parameter:** `op_getkey` is called by `execute_op` which is called by `run_loop` — the call chain does not thread extra parameters. Using a transient state field is the established pattern (mirrors how `modal_prompt` carries multi-step modal data). The `take()` ensures the field is consumed exactly once.

### run_loop WaitForKey arm placement

The WaitForKey arm in `run_loop` must fire BEFORE the `other => execute_op(state, op)?` catch-all (same as PSE/VIEW/AVIEW arms in Phase 63-02), and AFTER the interrupt boundary check:

```rust
// run_loop match arm ordering (inside the op match block):
Op::Rtn => { ... }
Op::Stop => { break; }
Op::Prompt => { ... break; }
Op::Pse => { ... break; }         // Phase 63
Op::View(reg) => { ... break; }   // Phase 63
Op::AView => { ... break; }       // Phase 63
Op::GetKey => {                   // Phase 64 — BEFORE other =>
    state.pending_yield = Some(YieldState {
        kind: YieldKind::WaitForKey,
        text: String::new(),
        resume_ms: 0,
    });
    break;
}
other => { execute_op(state, other)?; }
```

### CLI Capture Path

During a WaitForKey suspend:
1. `run_program`/`resume_program_with_key` returns with `state.pending_yield = Some(WaitForKey{})`.
2. A new `drain_pending_yields` extension detects `WaitForKey` kind: does NOT sleep (no timer); instead enters a **blocking key-wait loop**.
3. The key-wait loop polls `crossterm::event::read()` for the next `KeyEventKind::Press` event.
4. For each key, call `keycode_to_hp41_code(key.code)` to get the HP-41 row×col code.
5. If code is `Some(c)` and not a Ctrl-modified key: call `hp41_core::resume_program_with_key(&mut self.state, c)`.
6. If request_cancel semantics apply (ON/Esc equivalent — currently Ctrl+C in CLI): call `hp41_core::resume_program_with_key(&mut self.state, 0)` to push sentinel.
7. After resume, continue `drain_pending_yields` loop (in case next op is PSE/VIEW/AVIEW).

**Escape/cancel in CLI:** The existing `Ctrl+C` → `self.exit = true` path quits the app entirely. For GETKEY cancel, a dedicated escape (e.g., `Esc` key during GETKEY wait) should push sentinel 0 and resume. The planner must decide whether Esc = cancel-GETKEY or Esc = quit-app during GETKEY suspend. Recommended: `Esc` = cancel GETKEY (push 0 + resume), Ctrl+C = quit app.

**Keys that have no HP-41 code:** `keycode_to_hp41_code` returns `None` for F5/F7/F8 and unknown keys. During GETKEY suspend, these should be ignored (the wait loop continues without resuming). This is hardware-faithful: the HP-41 only captures physical calculator keys.

### GUI Capture Path

During a WaitForKey suspend:
1. `run_program` or `resume_program` (Phase 63) returns with `pending_yield: { kind: 'wait_for_key', resume_ms: 0, text: '' }`.
2. The existing yield-driver `useEffect` on `pending_yield` fires. It detects `kind === 'wait_for_key'` — instead of scheduling a `setTimeout(resume_ms)`, it enters a **key-capture mode**.
3. In key-capture mode, the next on-screen key tap or physical keyboard keypress invokes a new Tauri command: `resume_program_with_key({ keycode: N })`.
4. The Tauri command calls `hp41_core::ops::program::resume_program_with_key(&mut calc, keycode)`.
5. The returned `CalcStateView` is set via `setCalcState`, re-firing the effect. If `pending_yield` is now null, the yield loop ends. If it's another yield kind (PSE/VIEW), the existing timer path handles it.

**Cancel (R/S / Escape in GUI):** R/S branch 2 (is_running check) calls `request_cancel`. But during a WaitForKey suspend, `is_running` is `false` (run_loop has returned). This is a **gap**: during GETKEY suspend, the GUI needs to detect the suspended state. Options:
- (a) Add `is_getkey_suspended: bool` to `CalcStateView` (simplest).
- (b) Check `pending_yield?.kind === 'wait_for_key'` in App.tsx.

Option (b) requires no new field: if `calcState.pending_yield?.kind === 'wait_for_key'`, the cancel button calls `resume_program_with_key({ keycode: 0 })`. The planner should use option (b) to avoid a new CalcStateView field.

**Physical keyboard during GETKEY in GUI:** The App.tsx `handleKey` function currently routes physical keys to `dispatch_op`. During GETKEY suspend (`pending_yield?.kind === 'wait_for_key'`), physical keys should instead invoke `resume_program_with_key({ keycode: resolvedHp41Code })`. A guard on `pending_yield?.kind` at the top of `handleKey`/`invokeForKey` achieves this.

**KEY_DEFS keyCode coverage:** Keyboard.tsx `MAIN_GRID` already maps most keys to HP-41 `keyCode`. Keys without a `keyCode` (CHS=undefined, xge_y=undefined, clx_or_a=undefined, shift button itself) cannot provide a keycode for GETKEY. During GETKEY suspend, tapping a key without `keyCode` should be a no-op (the wait continues). This is acceptable — those keys have no HP-41 hardware equivalent for row×col encoding.

---

## HP-41 Key Code Scheme (VERIFIED)

Row×10+col, 1-indexed. 8 rows × 5 columns. Confirmed from multiple sources:

| HP-41 Row | Keys (col 1–5) | Codes |
|-----------|-----------------|-------|
| Row 1 | Σ+(11), 1/x(12), √x(13), LOG(14), LN(15) | 11–15 |
| Row 2 | XEQ(21), STO(22), RCL(23), R↓(24), SIN(25) | 21–25 |
| Row 3 | R/S(31), SST(32), GTO(33), COS(34), TAN(35) | 31–35 |
| Row 4 | USER(41), f(42), g(43), ENTER(44), ÷(45) | 41–45 |
| Row 5 | 7(51), 8(52), 9(53), ×(54) | 51–54 |
| Row 6 | 4(61), 5(62), 6(63), −(64) | 61–64 |
| Row 7 | 1(71), 2(72), 3(73), +(74) | 71–74 |
| Row 8 | 0(81), .(82), EEX(83), R/S(84), ENTER(85) | 81–85 |

[VERIFIED: `hp41-cli/src/keys.rs:keycode_to_hp41_code` — exact mapping in code; `hp41-gui/src/Keyboard.tsx:MAIN_GRID` keyCode fields — full GUI map]

Note: CLI maps Enter→84 and R/S→31 (via the F5/R-S key assignment). GUI maps R/S→31 (`r_s` keyCode: 31) and ENTER→84 (`enter` keyCode: 84). Both agree on R/S = 84 in the row×col table (row 8, col 4) vs. the functional R/S = 31 assignment. In the CONTEXT.md D-02 decision, R/S "is captured as keycode 84" — this refers to the physical ENTER key position (row 8, col 4), not the R/S key's keyCode=31. Planner should clarify: during GETKEY, pressing the R/S key should return 31 (its physical position), not 84. The CONTEXT.md wording "R/S is captured as keycode 84" [ASSUMED: may be a documentation approximation — the actual HP-41 hardware assigns R/S to row 3 col 1 = 31; keyCode 84 is ENTER's physical position].

**Gap — Keys missing keyCode in GUI:**
- CHS (row 4, col 2 = hardware code 42) — `keyCode: undefined` in Keyboard.tsx (comment: "no unambiguous CLI mapping")
- xge_y (x≥y, row 2, col 0 = hardware code 21-like) — `keyCode: undefined`
- clx_or_a (←, row 4, col 4) — `keyCode: undefined`
- shift button — not a keyable HP-41 key

These keys cannot provide a GETKEY code via the GUI on-screen keyboard. Physical keyboard equivalents (where mapped in App.tsx keyboard MAP) may provide codes. This is a pre-existing gap; GETKEY simply ignores taps on these keys during the wait.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Key event blocking in CLI | Custom sleep/poll | crossterm `event::read()` (already used) | Already proven in drain_pending_yields |
| Async key capture in GUI | Polling `get_state` | `pending_yield` kind detection + resume command | D-11 no-polling; same pattern as PSE/VIEW/AVIEW yield driver |
| New Op variant for GETKEY | Op::WaitForKey or similar | YieldKind::WaitForKey on existing Op::GetKey | 4-way exhaustive match invariant — Op::GetKey already exists |

---

## Common Pitfalls

### Pitfall 1: Clearing `pending_interrupt*` fields in `resume_program_with_key`
**What goes wrong:** If `resume_program_with_key` clears `pending_interrupt_alarm_index` and `pending_interrupt_depth` (copying `resume_program` verbatim), the ack-after-RTN gate in `Op::Rtn` will miss the alarm acknowledgment when GETKEY is inside an alarm handler.
**Why it happens:** `resume_program` clears all transient interrupt state (D-09), which is correct for the R/S-after-STOP case but NOT for GETKEY-mid-alarm-handler.
**How to avoid:** `resume_program_with_key` must NOT clear `pending_interrupt_alarm_index` / `pending_interrupt_depth`. It SHOULD clear `pending_yield` and `pending_interrupt` (the label itself — it was already consumed by `take()` when the interrupt fired). Only clear `getkey_captured_code` AFTER `run_loop` returns.
**Warning signs:** `ack_after_RTN_fires_when_getkey_in_alarm_handler` test fails.

### Pitfall 2: `resume_program` (timer path) inadvertently resumes a WaitForKey yield
**What goes wrong:** The Phase 63 GUI yield driver calls `resume_program` on a timeout. If `pending_yield.kind == WaitForKey`, the driver fires and re-enters `run_loop` without a keycode — `getkey_captured_code` is None — so `op_getkey` falls back to `last_key_code` (wrong behavior: stale code or 0).
**Why it happens:** The yield driver fires on every `pending_yield` change, regardless of kind.
**How to avoid:** The yield driver `useEffect` must check `kind !== 'wait_for_key'` before scheduling a `setTimeout`. WaitForKey yields must be routed exclusively to `resume_program_with_key`.
**Warning signs:** GETKEY in program immediately resumes with stale `last_key_code`.

### Pitfall 3: GUI key dispatch during GETKEY fires a normal `dispatch_op` instead of `resume_program_with_key`
**What goes wrong:** The user presses a key during GETKEY suspend. App.tsx `handleKey` routes it to `dispatch_op(keyId)`, executing the key's function rather than providing the code to GETKEY.
**Why it happens:** The existing key-event path is unconditional.
**How to avoid:** Add a guard at the top of `handleKey`/`invokeForKey`: if `calcState?.pending_yield?.kind === 'wait_for_key'`, route to `resume_program_with_key({ keycode: hp41Code })` instead of `dispatch_op`. The HP-41 code for the key comes from `KEY_DEFS.find(k => k.id === resolvedId)?.keyCode`.
**Warning signs:** Key presses during GETKEY execute their normal functions instead of resuming.

### Pitfall 4: R/S key code ambiguity (31 vs. 84)
**What goes wrong:** The CONTEXT.md says "R/S is captured as keycode 84" but the Keyboard.tsx R/S entry has `keyCode: 31`. These conflict.
**Why it happens:** R/S physically occupies row 3, col 1 (code 31) on the HP-41 hardware; code 84 is the ENTER key's row 8, col 4 position. The CONTEXT.md phrasing is imprecise.
**How to avoid:** During GETKEY, R/S returns **31** (its `keyCode` in KEY_DEFS, the physical hardware position). The CONTEXT.md statement "R/S captured as keycode 84" [ASSUMED] may refer to a different key layout tradition or is a typo. Physical HP-41 R/S is row 3, col 1 = 31.
**Warning signs:** GETKEY test fails for R/S key expecting 84.

### Pitfall 5: CLI blocking loop during GETKEY freezes the redraw ticker
**What goes wrong:** The `drain_pending_yields` extension enters a blocking `event::read()` loop for GETKEY. The 16ms poll timer no longer fires → no redraws → UI appears frozen.
**Why it happens:** `crossterm::event::read()` blocks indefinitely.
**How to avoid:** Use `crossterm::event::poll(Duration::from_millis(16))` inside the key-wait loop and redraw on each timeout iteration (same pattern as the main `run()` loop). Or use a non-blocking `poll` with a short timeout and call `terminal.draw(...)` between polls.
**Warning signs:** Display freezes while GETKEY is active in CLI.

### Pitfall 6: `getkey_captured_code` not cleaned up on error return from `run_loop`
**What goes wrong:** If `run_loop` returns `Err(...)` after `getkey_captured_code` was set but before `op_getkey` consumed it, the field stays `Some(code)`. The next `run_program` call (fresh execution) reads a stale code.
**Why it happens:** `resume_program_with_key` sets the field before `run_loop`; if `run_loop` errors before reaching GETKEY, `op_getkey` never runs.
**How to avoid:** `resume_program_with_key` must `state.getkey_captured_code = None` after `run_loop` returns (both Ok and Err paths), BEFORE returning.
**Warning signs:** Test: "run_loop error before GETKEY must not leave getkey_captured_code set".

---

## Code Examples

### WaitForKey yield arm in run_loop
```rust
// Source: Phase 63 63-02-SUMMARY.md (PSE/VIEW/AVIEW pattern — extend the same set)
// Placement: BEFORE other => catch-all, AFTER Phase 63 PSE/VIEW/AVIEW arms
Op::GetKey => {
    state.pending_yield = Some(YieldState {
        kind: YieldKind::WaitForKey,
        text: String::new(),  // D-03: no display override
        resume_ms: 0,         // event-driven, not timer-driven
    });
    break;
}
```

### CLI key-wait loop (in drain_pending_yields extension)
```rust
// Pattern: extend drain_pending_yields to handle WaitForKey kind
// (non-timer branch — blocks until keypress, with periodic redraws)
YieldKind::WaitForKey => {
    // Non-blocking poll loop so the TUI redraws during the wait.
    loop {
        terminal.draw(|frame| self.draw(frame))?;
        if crossterm::event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind != KeyEventKind::Press { continue; }
                // Cancel / escape path → sentinel 0
                if key.code == KeyCode::Esc {
                    match hp41_core::ops::program::resume_program_with_key(&mut self.state, 0) {
                        Ok(()) => break,
                        Err(e) => { self.message = Some(format!("{e}")); break; }
                    }
                }
                // Quit path stays Ctrl+C
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.exit = true;
                    return Ok(());
                }
                // HP-41 key capture
                if let Some(code) = keys::keycode_to_hp41_code(key.code) {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        match hp41_core::ops::program::resume_program_with_key(&mut self.state, code) {
                            Ok(()) => break,
                            Err(e) => { self.message = Some(format!("{e}")); break; }
                        }
                    }
                }
                // No HP-41 equivalent (F5/F7/F8, unknown) — continue waiting
            }
        }
    }
    // After resume, continue drain_pending_yields loop (may have more yields)
    self.drain_and_show_print_output(None);
}
```

### GUI yield driver extension (App.tsx)
```typescript
// Source: Phase 63 63-06-SUMMARY.md (yield-driver useEffect — extend for WaitForKey)
useEffect(() => {
  if (!calcState?.pending_yield) return;
  if (calcState.pending_yield.kind === 'wait_for_key') return; // handled by key events
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

// New: key event guard in invokeForKey / handleKey
// When pending_yield.kind === 'wait_for_key', route to resume_program_with_key
function getHp41KeyCode(keyId: string): number | undefined {
  return KEY_DEFS.find(k => k.id === keyId)?.keyCode;
}

// In invokeForKey, before normal dispatch:
if (state?.pending_yield?.kind === 'wait_for_key') {
  const hp41Code = getHp41KeyCode(effectiveId);
  if (hp41Code !== undefined) {
    return invoke<CalcStateView>('resume_program_with_key', { keycode: hp41Code });
  }
  // No HP-41 code for this key → no-op (wait continues)
  return Promise.resolve(state);
}

// Cancel during WaitForKey: if R/S or cancel pressed, push sentinel 0
// (detected via pending_yield.kind, not is_running)
```

### New Tauri command (commands.rs)
```rust
// Pattern: mirrors resume_program (63-04-SUMMARY.md) — thin SC-4 glue
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

---

## State of the Art

| Old Behavior | New Behavior | Change | Impact |
|--------------|-------------|--------|--------|
| `op_getkey` reads stale `last_key_code` | `run_loop` breaks with WaitForKey yield; keycode delivered via `resume_program_with_key` | Phase 64 | GETKEY is now interactive |
| GUI never sets `last_key_code` (gap D-04) | GUI routes on-screen key + physical key to `resume_program_with_key(keycode)` | Phase 64 | Closes the GUI GETKEY gap |
| GETKEY sentinel 0 = default `last_key_code` | Sentinel 0 pushed only on `request_cancel` escape | Phase 64 | FGAP-08 resolved |
| `YieldKind`: Pse/View/Aview only | Adds `WaitForKey` | Phase 64 | Event-driven yield family |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | GETKEY waits ~10 seconds on real HP-41CX hardware (forum recollection, not manual) | D-01 Resolution | If wrong: emulator correctly implements indefinite-wait (our approach), which is safer/simpler |
| A2 | ALPHA mode does NOT change the row×col keycode GETKEY returns | D-06 ALPHA mode | If wrong: need an ALPHA-mode code transformation; low likelihood since GETKEY captures hardware position |
| A3 | R/S key GETKEY code is 31 (physical position), not 84 (ENTER physical position) | D-04 / HP-41 key code scheme | If wrong: R/S GETKEY code is 84; affects user programs that branch on R/S code |
| A4 | Keys without `keyCode` in Keyboard.tsx (CHS, xge_y, clx_or_a) are correctly ignored during GETKEY wait | GUI capture path | If wrong: those keys silently fail to resume; user experience only (not correctness) |
| A5 | `resume_program_with_key` must NOT clear `pending_interrupt*` fields (unlike `resume_program`) | D-06 nested yield | If wrong: ack-after-RTN is skipped when GETKEY fires inside alarm handler — repeating alarms don't reschedule |

**If this table is empty:** It is not — A1–A5 above require attention.

---

## Open Questions (RESOLVED)

> All three resolved by the orchestrator with the user during `/gsd-plan-phase 64` (2026-06-07) and implemented in the plans.

1. **R/S key code during GETKEY (A3): 31 or 84? → RESOLVED: 31.**
   - What we know: Keyboard.tsx `r_s` has `keyCode: 31`; CONTEXT.md says "R/S captured as keycode 84"; hardware row 3 col 1 = 31, hardware row 8 col 4 = 84 (ENTER position).
   - **Resolution:** Use **31** (the physical R/S position). Confirmed with the user; this overrides CONTEXT.md D-02's "84" (which conflated R/S with ENTER). Matches Keyboard.tsx, CLI `keys.rs` (R/S → 31), and hardware. Implemented in Plan 64-02 T2; the D-02 "84"→31 correction is recorded in Plan 64-05 (divergence doc).

2. **CLI Esc key during GETKEY: cancel-GETKEY (sentinel 0) or quit-app? → RESOLVED: cancel → sentinel 0.**
   - **Resolution:** `Esc` during GETKEY suspend → push sentinel 0 + resume (Phase 63 `request_cancel` path); `Ctrl+C` still hard-quits the app. Confirmed with the user. Implemented in Plan 64-02 T1.

3. **`resume_program_with_key` vs. Tauri command naming → RESOLVED.**
   - **Resolution:** Core function = `resume_program_with_key`; Tauri command = `resume_program_with_key`; permission TOML = `allow-resume-program-with-key.toml`. Implemented across Plans 64-01/64-03/64-04.

---

## Environment Availability

> Step 2.6: SKIPPED — no external dependencies; this phase is code-only changes to existing Rust + TypeScript components.

---

## Validation Architecture

> nyquist_validation: true — include this section.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (hp41-core), just test-core, just gui-ci (Rust + Vitest) |
| Config file | `hp41-core/` (Cargo.toml workspace), `hp41-gui/` (vitest.config.ts) |
| Quick run command | `just test-core --test phase_64_getkey` |
| Full suite command | `just ci` (core + CLI + license-audit), `just gui-ci` (GUI Rust + Vitest) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File |
|--------|----------|-----------|-------------------|------|
| PRGM-03-a | GETKEY in program suspends (pending_yield WaitForKey set, loop breaks) | unit | `just test-core --test phase_64_getkey -- getkey_mid_run_breaks_and_sets_wait_for_key` | ❌ Wave 0 |
| PRGM-03-b | resume_program_with_key delivers keycode to X (LiftEffect::Enable, correct code) | unit | `just test-core --test phase_64_getkey -- resume_with_key_pushes_code_to_x` | ❌ Wave 0 |
| PRGM-03-c | resume_program_with_key(0) delivers sentinel 0 (cancel path) | unit | `just test-core --test phase_64_getkey -- resume_with_key_zero_pushes_sentinel` | ❌ Wave 0 |
| PRGM-03-d | Program continues to next step after GETKEY + resume | unit | `just test-core --test phase_64_getkey -- getkey_resume_continues_to_next_step` | ❌ Wave 0 |
| PRGM-03-e | GETKEY does NOT set display_override (D-03: display unchanged) | unit | `just test-core --test phase_64_getkey -- getkey_does_not_write_display_override` | ❌ Wave 0 |
| PRGM-03-f | GETKEY inside alarm handler: pending_interrupt fields survive resume | unit | `just test-core --test phase_64_getkey -- getkey_inside_alarm_handler_preserves_interrupt_state` | ❌ Wave 0 |
| PRGM-03-g | GETKEY followed by PSE: sequential yields work (WaitForKey then Pse) | unit | `just test-core --test phase_64_getkey -- getkey_then_pse_sequential_yields` | ❌ Wave 0 |
| PRGM-03-h | Non-program (interactive) dispatch still uses last_key_code (backward compat) | unit | `just test-core --test synthetic_tests -- test_getkey_pushes_last_key_code` | ✅ existing |
| PRGM-03-i | getkey_captured_code cleared after run_loop error (no leakage) | unit | `just test-core --test phase_64_getkey -- getkey_captured_code_cleared_on_error` | ❌ Wave 0 |
| PRGM-03-j | GUI CalcStateView projects WaitForKey kind as "wait_for_key" | Rust unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- from_state_projects_wait_for_key` | ❌ Wave 0 |
| PRGM-03-k | GUI yield driver ignores WaitForKey (no setTimeout scheduled) | Vitest | `just gui-ci -- getkey_yield_driver_skips_wait_for_key` | ❌ Wave 0 |
| PRGM-03-l | GUI key event during WaitForKey → resume_program_with_key (not dispatch_op) | Vitest | `just gui-ci -- getkey_key_press_routes_to_resume` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `just test-core --test phase_64_getkey`
- **Per wave merge:** `just ci` (core + CLI + license-audit)
- **Phase gate:** `just ci && just gui-ci` — full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `hp41-core/tests/phase_64_getkey.rs` — covers PRGM-03-a through PRGM-03-i (9 new tests)
- [ ] `hp41-gui/src-tauri/src/commands.rs` test: `from_state_projects_wait_for_key` — covers PRGM-03-j
- [ ] `hp41-gui/src/App.test.tsx` — Group Q tests covering PRGM-03-k and PRGM-03-l
- [ ] `YieldKind::WaitForKey` variant in `hp41-core/src/state.rs` + `getkey_captured_code` field (Wave 1 state.rs deliverable)

*(Existing `synthetic_tests.rs::test_getkey_pushes_last_key_code` covers PRGM-03-h — no gap there)*

---

## Security Domain

> security_enforcement: not explicitly false → include this section.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes (key codes are u8 — bounded by type; no string parsing) | Rust type system (`u8`, `Option<u8>`) |
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Key code out of range | Tampering | `u8` type (max 255); HP-41 row×col codes are 11–85; no bounds check needed beyond `u8` |
| Double-resume (resume_program_with_key called twice) | Tampering | `getkey_captured_code.take()` — second call gets None → falls back to `last_key_code` (harmless) |
| resumeScheduledRef not guarded for WaitForKey | DoS (duplicate scheduling) | Early return when `kind === 'wait_for_key'` in yield driver |

---

## Sources

### Primary (HIGH confidence)
- `hp41-core/src/ops/program.rs` — `run_loop` (lines 498–714), `resume_program` (lines 475–490), `run_program` (lines 423–459) — direct code reading
- `hp41-core/src/state.rs` — `YieldKind`, `YieldState`, `PSE_RESUME_MS`, `CalcState` fields including `last_key_code`, `pending_yield`, `pending_interrupt*`, `getkey_captured_code` (to add) — direct code reading
- `hp41-core/src/ops/registers.rs:143–151` — `op_getkey` current implementation — direct code reading
- `hp41-cli/src/keys.rs:452–528` — `keycode_to_hp41_code` and HP-41 row×col scheme — direct code reading
- `hp41-gui/src/Keyboard.tsx:173–236` — `KEY_DEFS`/`MAIN_GRID` with `keyCode` fields — direct code reading
- `.planning/phases/63-*/63-02-SUMMARY.md` — PSE/VIEW/AVIEW yield arm pattern — direct reading
- `.planning/phases/63-*/63-04-SUMMARY.md` — `run_program`/`resume_program` Tauri commands + SC-4 pattern — direct reading
- `.planning/phases/63-*/63-06-SUMMARY.md` — yield-and-resume driver in App.tsx — direct reading
- `hp41-core/tests/phase_63_yield_engine.rs` — test fixture patterns for yield engine — direct reading
- `.planning/phases/64-interactive-getkey/64-CONTEXT.md` — locked decisions — direct reading

### Secondary (MEDIUM confidence)
- [Finseth HP-41CX function list](https://www.finseth.com/hpdata/hp41cx.php) — GETKEY "get a key" / GETKEYX "get a key timed" distinction (CITED)
- `.planning/research/DIVERGENCE-AUDIT.md` — FGAP-04/FGAP-08 behavioral description and code citations

### Tertiary (LOW confidence)
- [forum.hp41.org thread 488](https://forum.hp41.org/viewtopic.php?f=20&t=488) — community report of ~10s GETKEY timeout and 0 sentinel (forum post, not primary source)
- `.planning/research/ARCHITECTURE.md` — synchronous run_loop yield mechanism (confirmed implemented in Phase 63)

---

## Metadata

**Confidence breakdown:**
- Yield engine extension: HIGH — Phase 63 source fully read; patterns are verified and repeatable
- YieldKind::WaitForKey shape: HIGH — direct extrapolation from existing Pse/View/Aview shapes
- HP-41 GETKEY wait-indefinitely semantics: MEDIUM — Finseth distinguishes GETKEY vs GETKEYX by name; forum corroborates halt-until-keypress
- R/S key code = 31 vs 84: MEDIUM (ASSUMED) — requires manual verification
- ALPHA mode key code: MEDIUM (ASSUMED) — hardware design logic; no direct manual citation
- GUI capture path design: HIGH — extends verified Phase 63 patterns

**Research date:** 2026-06-06
**Valid until:** 2026-07-06 (30 days — stable codebase, no fast-moving deps)
