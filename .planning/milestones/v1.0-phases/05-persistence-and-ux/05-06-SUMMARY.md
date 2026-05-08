---
phase: 05-persistence-and-ux
plan: 06
subsystem: ux-modal-input
tags: [rust, ratatui, tui, modal-input, sto, rcl, alpha-mode, user-mode, overlay-navigation]

# Dependency graph
requires:
  - phase: 05-persistence-and-ux
    plan: 03
    provides: App struct Phase 5 fields (pending_input, show_help, show_programs, help_table_state, programs_table_state, PendingInput enum)
  - phase: 05-persistence-and-ux
    plan: 05
    provides: render_help_overlay() + render_programs_overlay() + pending_prompt() in ui.rs (display layer for this plan's modals)

provides:
  - hp41-cli/src/app.rs handle_pending_input() — dispatches all PendingInput variants (STO/RCL/StoArith/Assign/ConfirmLoad)
  - hp41-cli/src/app.rs handle_reg_modal() — generic 2-digit register accumulator with auto-dispatch on 2nd digit (D-09)
  - hp41-cli/src/app.rs handle_alpha_mode_key() — AlphaAppend/AlphaBackspace/AlphaToggle routing (D-12 through D-15)
  - hp41-cli/src/app.rs handle_key() — S/R/Ctrl+A guards + pending_input + alpha_mode + overlay navigation blocks (routing priority D-08)
  - hp41-cli/src/keys.rs — u=Op::UserMode, S=None (STO modal), R=None (RCL modal), KEY_REF_TABLE extended to 40 entries

affects:
  - 05-07 (USER mode dispatch via key_assignments + try_user_dispatch not yet wired; AssignKey/AssignLabel flow now functional)
  - 05-08 (full integration test: STO/RCL round-trip through CalcState.regs)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Modal state machine via PendingInput enum with take()/re-set pattern — avoid borrow issues
    - handle_reg_modal generic helper: op_fn + pending_fn closures for STO/RCL/StoArith variants
    - Routing priority in handle_key: Release filter → Ctrl-keys → S/R/Ctrl+A triggers → pending_input guard → alpha_mode guard → overlay nav → digit entry → key_to_op
    - Overlay key-consume-all: early return after overlay match block ensures non-nav keys don't fall through to normal dispatch
    - ConfirmLoad(usize) safety: non-empty program.is_empty() check before load triggers confirmation modal (T-05-15)
    - Register range guard in handle_reg_modal: reg < 100 check before dispatch (T-05-13)

key-files:
  created: []
  modified:
    - hp41-cli/src/app.rs (282 → 523 lines: handle_pending_input, handle_reg_modal, handle_alpha_mode_key, routing guards, overlay nav)
    - hp41-cli/src/keys.rs (98 → 150 lines: u/S/R bindings, extended KEY_REF_TABLE, keys::tests module)
    - hp41-cli/src/tests/keys_tests.rs (106 → 113 lines: S/R None assertions, KEY_REF_TABLE count updated to 40)

key-decisions:
  - "S key remapped from Op::Sin to STO modal (returns None from key_to_op, intercepted in handle_key before key_to_op); SIN is now unmapped — trade-off accepted per plan D-10"
  - "handle_reg_modal() as shared generic helper with op_fn/pending_fn closures eliminates 6 duplicate match arms across STO/STO+/-/x/div variants"
  - "Backspace in reg modal resets entire accumulator to empty (not just last digit) — matches HP-41 hardware behavior (D-09)"
  - "ConfirmLoad idx passed as usize (not program data) to avoid cloning Vec<Op> in pending state — programs looked up by index on confirm"

requirements-completed: [UX-01, UX-02, UX-03, PERS-01]

# Metrics
duration: ~12min
completed: 2026-05-07
---

# Phase 5 Plan 06: Modal Input State Machine + Overlay Navigation Summary

**STO/RCL 2-digit register modal, ALPHA mode routing, Ctrl+A key-assign modal, overlay Up/Down/Enter navigation, and u=UserMode binding all wired into app.rs handle_key() with correct routing priority**

## Performance

- **Duration:** ~12 min
- **Completed:** 2026-05-07
- **Tasks:** 2
- **Files created:** 0, **Files modified:** 3

## Accomplishments

- `handle_pending_input()`: full state machine for all 8 PendingInput variants. StoRegister/RclRegister/StoAdd/StoSub/StoMul/StoDiv via shared `handle_reg_modal()` generic helper; AssignKey/AssignLabel for USER key assignment (D-27); ConfirmLoad(idx) for program overwrite confirmation (D-22).
- `handle_reg_modal()`: generic 2-digit accumulator — op_fn closure produces the correct Op (StoReg/RclReg/StoArith), pending_fn closure produces the correct PendingInput variant. Auto-dispatches on 2nd digit, Backspace resets accumulator, Esc cancels (D-09). Range guard: reg >= 100 shows error without dispatching (T-05-13).
- `handle_alpha_mode_key()`: all printable chars → AlphaAppend(c), Backspace → AlphaBackspace, Esc/Enter/'a' → AlphaToggle (exits ALPHA mode). Prevents 'a' from dispatching Asin while in ALPHA mode (D-12 through D-15).
- `handle_key()` routing guards added (in priority order): S/R/Ctrl+A trigger initiators → pending_input.is_some() early-return → alpha_mode early-return → show_help overlay nav (Up/Down/j/k scroll, Esc/q/? close) → show_programs overlay nav (Up/Down/j/k scroll, Esc close, Enter load-or-confirm).
- `keys.rs`: `u` → Op::UserMode (D-26). S and R explicitly return None with comment — modal intercepted upstream. KEY_REF_TABLE extended from 33 to 40 entries with Phase 5 bindings.
- `keys::tests` module added: `test_user_mode_dispatch` verifies Op::UserMode toggles state.user_mode both directions; `test_user_key_assignment_persists` verifies BTreeMap key assignment round-trip.
- Full test suite: **35 tests pass, zero regressions** (33 pre-existing + 2 new keys::tests).

## Task Commits

1. **Task 1: Add S/R/u bindings + KEY_REF_TABLE + tests in keys.rs** - `c5dca08` (feat)
2. **Task 2: Add handle_pending_input/alpha/reg_modal + overlay nav to app.rs** - `349f2e0` (feat)

## Files Created/Modified

- `hp41-cli/src/app.rs` — modified: 3 new methods (handle_pending_input, handle_reg_modal, handle_alpha_mode_key), routing guards and overlay nav blocks in handle_key()
- `hp41-cli/src/keys.rs` — modified: u/S/R bindings, KEY_REF_TABLE extended, keys::tests module
- `hp41-cli/src/tests/keys_tests.rs` — modified: S/R now assert None, KEY_REF_TABLE count updated to 40

## Decisions Made

- S remapped from SIN to STO modal: `Op::Sin` was the prior binding for `S` (Shift+s). Per plan D-10, S must now trigger the STO register modal. SIN is unmapped in v1.0 keyboard layout — this is an accepted trade-off (SIN available via `handle_alpha_mode_key` path is not wired; direct uppercase key lost).
- `handle_reg_modal()` as shared generic: instead of 6 identical match blocks for STO/STO+/-/x/div, one generic function takes `op_fn: impl Fn(u8) -> Op` and `pending_fn: impl Fn(String) -> PendingInput` closures.
- `take()` pattern in `handle_pending_input()`: `self.pending_input.take()` clears the field before the match, then each arm either re-sets it (modal continues) or leaves it None (done/cancelled). Avoids double-borrow issues with `&self.pending_input`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] keys_tests.rs expected S → Op::Sin (old behavior)**
- **Found during:** Task 2 (cargo test -p hp41-cli after Task 1 changes)
- **Issue:** Existing `trig_math_uppercase_shift` test asserted `key_to_op(S) == Some(Op::Sin)`. Plan 06 intentionally remaps S to STO modal (returns None). Test was testing old behavior that the plan changes.
- **Fix:** Updated assertion to `None` with clear comment explaining S is intercepted upstream. Added R=None and u=Op::UserMode assertions. Updated `key_ref_table_has_33_entries` count from 33 to 40.
- **Files modified:** `hp41-cli/src/tests/keys_tests.rs`
- **Verification:** `cargo test -p hp41-cli` 35 tests pass.
- **Committed in:** 349f2e0 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — test update for intentional behavior change)
**Impact on plan:** Test update was necessary — the plan explicitly remaps S from SIN to STO modal. No scope creep.

## Known Stubs

None — all functionality fully wired. Note: `try_user_dispatch()` is described in PATTERNS.md but is out of scope for Plan 06 (it belongs to Plan 07 USER mode dispatch). The `key_assignments` field is functional (AssignKey/AssignLabel wired) but lookup during normal key dispatch is Plan 07's responsibility.

## Threat Model Coverage

| Threat | Status |
|--------|--------|
| T-05-13: Tampering — register number > 99 via modal | Covered: handle_reg_modal() checks `reg < 100`; shows error message, does not dispatch |
| T-05-14: DoS — AssignLabel accumulator unbounded growth | Accepted: HP-41 label names are short; no network/storage amplification; no explicit limit in v1.0 |
| T-05-15: Tampering — ConfirmLoad overwrites program without confirmation | Covered: `if !self.state.program.is_empty()` triggers ConfirmLoad(idx) modal; any non-Y key cancels |

## Self-Check

### Modified files exist

- `hp41-cli/src/app.rs` — exists (523 lines)
- `hp41-cli/src/keys.rs` — exists (150 lines)
- `hp41-cli/src/tests/keys_tests.rs` — exists (113 lines)

### Commits exist

- c5dca08 — Task 1 (keys.rs: S/R/u bindings + KEY_REF_TABLE + keys::tests)
- 349f2e0 — Task 2 (app.rs: handle_pending_input + handle_reg_modal + handle_alpha_mode_key + routing guards)

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
