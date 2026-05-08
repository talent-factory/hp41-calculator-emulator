---
phase: 03-programming-engine
plan: "05"
subsystem: programming-engine
tags: [rust, hp41, interpreter, run_program, isg, dse, isg-dse, lbl, gto, xeq, rtn, test, prgm-mode, parse-counter, evaluate-test, tdd, integration-tests]

# Dependency graph
requires:
  - phase: 03-programming-engine
    plan: "01"
    provides: CalcState Phase 3 fields (program, prgm_mode, pc, call_stack, is_running)
  - phase: 03-programming-engine
    plan: "02"
    provides: HpError::CallDepth variant in error.rs
  - phase: 03-programming-engine
    plan: "03"
    provides: Phase 3 Op variants (Lbl/Gto/Xeq/Rtn/PrgmMode/Test/Isg/Dse) + TestKind in ops/mod.rs
  - phase: 03-programming-engine
    plan: "04"
    provides: prgm_mode gate in dispatch() + flush_entry_buf recording routing

provides:
  - ops/program.rs: run_program(), run_loop(), execute_op(), evaluate_test(), parse_counter(), build_counter()
  - op_lbl, op_gto, op_xeq, op_rtn, op_test, op_isg, op_dse public dispatch arms
  - program_tests.rs: 23-test PROG-01 + PROG-02 integration suite (all green)
  - Op::PrgmMode dispatch arm added to ops/mod.rs (entry path for recording mode)
  - PushNum in execute_op now applies LiftEffect::Enable (correct mid-program stack semantics)

affects: [03-06, 03-07, lib.rs-pub-use-run_program]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "run_program() clones state.program to avoid Rust borrow conflict (D-06 / RESEARCH Pitfall 1)"
    - "execute_op() private helper: no flush_entry_buf, no prgm_mode check (RESEARCH Pitfall 2 guard)"
    - "parse_counter() string-split at '.' + {:0<5} right-pad for CCCCC.FFFDD format (ADR-001)"
    - "evaluate_test() read-only stack access via Decimal ZERO sentinel (LiftEffect: Neutral)"
    - "ISG/DSE error-before-mutation: find/compute result before writing to state.regs"
    - "CallDepth enforced at call_stack.len() >= 4 before pushing (D-13/D-14)"
    - "PushNum inside execute_op applies LiftEffect::Enable so subsequent stack pushes lift correctly"

key-files:
  created:
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/program_tests.rs
  modified:
    - hp41-core/src/ops/mod.rs

key-decisions:
  - "PushNum inside execute_op must apply LiftEffect::Enable — without it, sequential PushNums inside a program overwrite X instead of lifting (Rule 1 bug fix)"
  - "ISG loop body runs N times where N includes the iteration where ISG finally skips (body-before-ISG structure); test expectation corrected from 4 to 5 with R00=1.00500 (Rule 1 fix)"
  - "Op::PrgmMode dispatch arm added to ops/mod.rs so test_prgm_mode_toggle can use dispatch() (Rule 3: needed for test suite to pass before plan 03-06)"
  - "push() test helper uses direct stack manipulation instead of dispatch(PushNum) to guarantee correct lift semantics in test setup"

patterns-established:
  - "parse_counter signature: (¿HpNum) -> Result<(i64, i64, i64, String), HpError> — returns frac_padded as 4th element for build_counter reuse"
  - "evaluate_test signature: (¿CalcState, ¿TestKind) -> bool — read-only, no lift mutation"
  - "execute_op(state, op) -> Result<(), HpError>: the private interpreter dispatch without flush/prgm_mode"

requirements-completed: [PROG-01, PROG-02]

# Metrics
duration: ~25min
completed: 2026-05-07
---

# Phase 03 Plan 05: Programming Engine Interpreter Summary

**PC-driven HP-41 interpreter (run_program/run_loop/execute_op) with ISG/DSE string-split counter parsing, 12-kind conditional tests, 4-deep call stack, and 23-test integration suite — all green**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-07T09:00:00Z
- **Completed:** 2026-05-07T09:25:00Z
- **Tasks:** 2 (Task 1: ops/program.rs; Task 2: program_tests.rs)
- **Files created:** 2 (program.rs, program_tests.rs)
- **Files modified:** 1 (ops/mod.rs)

## Accomplishments

- `ops/program.rs` created with complete HP-41 interpreter:
  - `run_program()`: clones program Vec (borrow guard), finds entry label, runs interpreter loop, resets is_running on both success and error paths
  - `run_loop()`: PC-driven loop with native handling of Rtn/Lbl/Gto/Xeq/Test/Isg/Dse
  - `execute_op()`: private dispatch for all non-programming ops without flush_entry_buf or prgm_mode check
  - `evaluate_test()`: 12 HP-41 conditionals (XEqZero through XGeY) — read-only, Neutral lift
  - `parse_counter()`: `CCCCC.FFFDD` string-split with `{:0<5}` right-pad (ADR-001 / D-10)
  - `build_counter()`: reconstructs HpNum preserving FFFDD fields
  - `op_lbl/op_gto/op_xeq/op_rtn/op_test/op_isg/op_dse`: public dispatch arms for plan 03-06
- `program_tests.rs` created with 23 integration tests:
  - PRGM mode recording (5 tests)
  - Label and branch: Lbl/GTO, unknown label error (4 tests)
  - Subroutine calls: XEQ+RTN, 4-level nesting, CallDepth error, top-level RTN (4 tests)
  - Conditional tests: skip-if-false, skip-if-true, all 12 TestKind variants (3 tests)
  - ISG/DSE counter: 4 tests covering step=00 (treated as 1), string round-trip, DSE boundary
  - Integration: is_running reset on success and error, full dispatch-recording-then-run (3 tests)
- Test count: 257 (baseline) → 280 (+23 program tests, all green)

## Task Commits

| # | Task | Commit | Type | Files |
|---|------|--------|------|-------|
| 1 | Create ops/program.rs — full interpreter | `43f35e5` | feat | ops/program.rs, ops/mod.rs (pub mod program) |
| 2 | Create program_tests.rs — 23-test suite | `cb1dd1e` | feat | program_tests.rs, ops/mod.rs (PrgmMode arm), ops/program.rs (PushNum lift fix) |

## Files Created/Modified

- `hp41-core/src/ops/program.rs` (368 lines) — HP-41 programming engine: run_program, run_loop, execute_op, evaluate_test, parse_counter, build_counter, all op_* dispatch functions
- `hp41-core/tests/program_tests.rs` (452 lines) — 23-test PROG-01 + PROG-02 integration suite
- `hp41-core/src/ops/mod.rs` — added `pub mod program;` and `Op::PrgmMode` dispatch arm

## Decisions Made

- `parse_counter` made `pub` (not private) to satisfy plan acceptance criteria; `evaluate_test` remains `pub` for test access via `hp41_core::ops::program::evaluate_test`
- `Op::PrgmMode` dispatch arm added to `ops/mod.rs` execute path — this is technically 03-06 territory but needed for `test_prgm_mode_toggle` to use `dispatch()` rather than setting state directly
- `push()` helper in tests uses direct stack manipulation (avoids dispatch's missing lift enable for PushNum)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] PushNum in execute_op must enable lift after placing value**
- **Found during:** Task 2 (test_xeq_and_rtn failing: Y=0 instead of Y=42)
- **Issue:** `execute_op` for `Op::PushNum` called `enter_number()` but did not call `apply_lift_effect(Enable)`. Subsequent PushNums in the same program overwrote X instead of lifting the stack. The `flush_entry_buf` path applies Enable but `execute_op` did not.
- **Fix:** Added `apply_lift_effect(state, LiftEffect::Enable)` after `enter_number(state, v)` in `execute_op` PushNum arm.
- **Files modified:** `hp41-core/src/ops/program.rs`
- **Verification:** `test_xeq_and_rtn` passes: Y=42 confirmed. All 23 tests green.
- **Committed in:** `cb1dd1e`

**2. [Rule 1 - Bug] test_isg_increments_4_times_before_skip expected R01=4 but correct value is R01=5**
- **Found during:** Task 2 (test_isg_increments_4_times_before_skip failing)
- **Issue:** Plan's test code asserted R01=4, but with the loop structure `Lbl, StoArith(body), Isg, Gto, Rtn` — the body executes BEFORE ISG. ISG with R00=1.00500 (current=1, final=5, step=1) fires 5 times: 1→2→3→4→5→6 (6>5 skips). Body runs on each pass including the final one → R01=5. The plan comment "4 times (1→2→3→4→5)" counted increments not iterations. Confirmed consistent with `test_isg_step_zero_treated_as_one` which expects R01=3 with current=3,final=5 (also body-before-ISG, 3 iterations).
- **Fix:** Changed assertion from `Decimal::from(4)` to `Decimal::from(5)` with corrected comment explaining body-before-ISG semantics.
- **Files modified:** `hp41-core/tests/program_tests.rs`
- **Verification:** `test_isg_increments_4_times_before_skip` passes with R01=5. Both ISG tests consistent.
- **Committed in:** `cb1dd1e`

**3. [Rule 3 - Blocking] Op::PrgmMode needed in dispatch() execute path for test suite**
- **Found during:** Task 2 (test_prgm_mode_toggle failing: `dispatch(PrgmMode)` returns InvalidOp)
- **Issue:** `dispatch()` catch-all `_ => Err(InvalidOp)` covers `Op::PrgmMode` in execute mode. Tests that use `dispatch(Op::PrgmMode)` to enter recording mode (rather than setting `s.prgm_mode=true` directly) need the dispatch arm. Plan 03-06 was designated to wire Phase 3 arms, but the test suite requires it now.
- **Fix:** Added `Op::PrgmMode => program::op_prgm_mode(state)` arm to `dispatch()` match before the catch-all.
- **Files modified:** `hp41-core/src/ops/mod.rs`
- **Verification:** `test_prgm_mode_toggle` passes. `test_full_program_via_dispatch_recording` passes.
- **Committed in:** `cb1dd1e`

---

**Total deviations:** 3 auto-fixed (2 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All fixes necessary for correctness. The PushNum lift fix ensures HP-41-correct mid-program stack behavior. The ISG assertion fix corrects a plan typo. The PrgmMode arm enables dispatch-based test setup (consistent with how dispatch is used throughout the codebase). No scope creep.

## Known Stubs

None — all functionality specified in the plan is fully implemented. The remaining Phase 3 dispatch arms (Lbl/Gto/Xeq/Rtn/Test/Isg/Dse in execute mode when `prgm_mode=false`) are intentionally left as `_ => Err(HpError::InvalidOp)` until plan 03-06 wires them.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Trust boundary T-03-05-02 (parse_counter string slicing) is mitigated: `frac_padded[..3]` and `[3..5]` are protected by the 5-char right-pad guarantee. T-03-05-03 (execute_op catch-all) returns `InvalidOp` — no panic possible. T-03-05-01 (GTO-to-self infinite loop) is accepted as a Phase 4 TUI concern (R/S key).

## Issues Encountered

None beyond the deviations documented above. All verification checks passed after each fix.

## Next Phase Readiness

- Plan 03-06 can wire the remaining Phase 3 dispatch arms (Lbl/Gto/Xeq/Rtn/Test/Isg/Dse) — `op_*` functions are fully implemented and public in `ops/program.rs`
- Plan 03-07 can export `run_program` from `lib.rs` — function is public in `ops/program.rs`
- `program_tests.rs` provides regression coverage for all interpreter behavior; subsequent plans should not break these 23 tests
- `prgm_mode_tests.rs` (10 tests from 03-04) continues to provide recording gate coverage

---
*Phase: 03-programming-engine*
*Completed: 2026-05-07*

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `hp41-core/src/ops/program.rs` exists | FOUND |
| `hp41-core/tests/program_tests.rs` exists | FOUND |
| `03-05-SUMMARY.md` exists | FOUND |
| Commit `43f35e5` (Task 1) exists | FOUND |
| Commit `cb1dd1e` (Task 2) exists | FOUND |
| `cargo check -p hp41-core` exits 0 | PASSED (0 errors) |
| `cargo test -p hp41-core --test program_tests` passes | PASSED (23/23) |
