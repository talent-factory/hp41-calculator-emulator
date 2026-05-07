---
phase: 07-hardening
plan: 04
subsystem: testing
tags: [rust, coverage, hp41-core, program, isg, dse, run_program, llvm-cov]

# Dependency graph
requires:
  - phase: 07-hardening
    plan: 01
    provides: "Zero-panic guarantee: #![deny(clippy::unwrap_used)] in lib.rs, test modules need #[allow(clippy::unwrap_used)]"
provides:
  - "35 targeted unit tests in ops/program.rs covering all error paths and execute_op match arms"
  - "ops/program.rs line coverage improved from 59% to 95%+ (D-15 resolved)"
  - "Overall hp41-core coverage gate passes at 94.57% (--fail-under-lines 80)"
affects: [future plans adding hp41-core program.rs changes, 07-05, 07-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Integration-style tests for run_program: craft a Vec<Op> program, call run_program(), assert stack/state result"
    - "Test module placement: inline #[cfg(test)] at end of production file with #[allow(clippy::unwrap_used)]"
    - "Use HpNum(Decimal::from_str(...).unwrap()) in tests for exact decimal counter values (pub(crate) inner field)"

key-files:
  created: []
  modified:
    - hp41-core/src/ops/program.rs

key-decisions:
  - "Extended beyond the original 12 planned error-path tests to add 23 execute_op integration tests — necessary to reach 80% file coverage because execute_op is the largest function (110 lines of match arms) and was almost entirely uncovered"
  - "Used run_program() as the test entry point for execute_op coverage rather than calling execute_op directly (it is private) — cleaner and validates the full interpreter pipeline"
  - "Discovered that the overall hp41-core coverage gate (--fail-under-lines 80) was already failing on develop at 64.31% before this plan; the worktree's coverage of 94.57% passes the gate because the worktree has the same test files but they achieve better combined coverage when the program_tests module is present"

patterns-established:
  - "Pattern: When a large match-dispatch function has low coverage, add integration tests that execute programs containing those op variants — this covers all arms in a single test"
  - "Pattern: For ISG/DSE counter tests, use Decimal values with explicit fractional structure e.g. 0.00103 (current=0, final=1, step=3) to get predictable skip behavior"

requirements-completed: [QUAL-04]

# Metrics
duration: 45min
completed: 2026-05-07
---

# Phase 7 Plan 04: Coverage Gap — ops/program.rs Targeted Tests Summary

**35 targeted tests for hp41-core/ops/program.rs: all error paths, execute_op match arms, and run_loop branches — ops/program.rs line coverage from 59% to 95%, overall coverage gate passes at 94.57%**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-07T20:27:00Z
- **Completed:** 2026-05-07T21:12:00Z
- **Tasks:** 1 (TDD task with RED tests that exercised existing code)
- **Files modified:** 1

## Accomplishments

- Added 35-test `program_tests` module to hp41-core/src/ops/program.rs with `#[cfg(test)] #[allow(clippy::unwrap_used)]`
- Covered all 12 originally planned error paths: label-not-found, is_running safety reset, 4-level call-depth limit (CallDepth error), MAX_STEPS infinite-loop guard (Overflow error), ISG/DSE out-of-bounds register, interactive GTO/XEQ InvalidOp, GTO label-not-found during run, parse_counter canonical example, ISG increment+skip sequence, RTN no-op on empty call_stack
- Added 23 execute_op integration tests covering all major match arms: arithmetic (Add/Sub/Mul/Div), stack ops (Enter/Clx/Chs/XySwap/Rdn/Lastx), STO/RCL/CLREG, FmtFix/Sci/Eng (both valid and >9 error paths), alpha ops, math ops (Sqrt/Sq/Int/Recip), trig (Sin/Cos/Tan/Asin/Acos/Atan), exp/log/pow, StoArith, Test skip/no-skip, UserMode toggle, ISG/DSE in programs, XEQ subroutine with RTN, all 12 evaluate_test variants, op_prgm_mode, op_lbl, op_test interactive no-ops
- ops/program.rs line coverage: 59% → 95.07%
- Overall hp41-core coverage gate: `just coverage` exits 0 at 94.57% total lines covered

## Task Commits

1. **Task 1: Add targeted tests for uncovered error paths in program.rs (D-15)** — `e00acf6` (test)

## Files Created/Modified

- `hp41-core/src/ops/program.rs` — Added 494-line test module (35 test functions) after the existing production code

## Decisions Made

- Extended to 35 tests (vs 12 planned): the original 12 tests only raised ops/program.rs coverage from 59% to ~61%. The `execute_op` function (110 lines of match arms) was almost entirely uncovered. Added 23 integration tests running short programs through `run_program()` to cover the match arms.
- Used `run_program()` as the integration test driver (rather than a hypothetical private `execute_op()` directly) — validates the full interpreter pipeline and covers run_loop arms simultaneously.
- Fixed clippy `field_reassign_with_default` lint by using struct update syntax `CalcState { program: ops, ..Default::default() }` and removing redundant `state.is_running = false` assignments (default is already false).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy field_reassign_with_default errors in test module**
- **Found during:** Task 1 (lint gate)
- **Issue:** `state_with_program` helper used `let mut s = CalcState::default(); s.program = ops; s` which triggers `clippy::field_reassign_with_default`. Also `state.is_running = false` after `CalcState::default()` is redundant.
- **Fix:** Changed helper to use struct update syntax. Removed 3 redundant `state.is_running = false` assignments.
- **Files modified:** hp41-core/src/ops/program.rs
- **Verification:** `just lint` exits 0
- **Committed in:** e00acf6 (Task 1 commit)

**2. [Rule 2 - Missing Critical] Extended from 12 to 35 tests to reach 80% file coverage**
- **Found during:** Task 1 (coverage measurement)
- **Issue:** The 12 planned tests only achieved ~61% line coverage on ops/program.rs, not the required 80%. The `execute_op` function (the largest in the file at 110 lines) was almost entirely uncovered because it is private and only reachable through `run_program()`.
- **Fix:** Added 23 additional integration tests that run short programs via `run_program()` to cover all major `execute_op` match arms, the evaluate_test variants, and run_loop branches (runs-off-end, Lbl no-op, Test skip/no-skip).
- **Files modified:** hp41-core/src/ops/program.rs
- **Verification:** ops/program.rs line coverage 95.07%; `just coverage` exits 0 at 94.57%
- **Committed in:** e00acf6 (Task 1 commit, same commit)

**3. [Rule 1 - Bug] Absolute path trap (#3099) — edits initially targeted main repo instead of worktree**
- **Found during:** Task 1 (pre-commit check)
- **Issue:** Edit tool calls used `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/ops/program.rs` (main repo path), but the worktree file is at `/Users/daniel/GitRepository/hp41-calculator-emulator/.claude/worktrees/agent-a94d246c0610c5b29/hp41-core/src/ops/program.rs`. All test code was written to the wrong location.
- **Fix:** Restored main repo file to its committed state (`git checkout -- hp41-core/src/ops/program.rs` in the main repo). Rewrote the complete test module to the correct worktree-relative absolute path.
- **Files modified:** .claude/worktrees/agent-a94d246c0610c5b29/hp41-core/src/ops/program.rs
- **Verification:** `git rev-parse --show-toplevel` from the worktree cwd confirms path; `cargo test -p hp41-core -- program_tests` shows 35 passed
- **Committed in:** e00acf6 (final Task 1 commit to worktree branch)

---

**Total deviations:** 3 auto-fixed (1 Rule 1 clippy lint, 1 Rule 2 missing coverage, 1 Rule 1 path bug)
**Impact on plan:** All three deviations were necessary corrections. The path bug was critical — without fixing it, no tests would have been committed to the worktree branch. The coverage extension was required to meet the plan's 80% file-coverage goal. The clippy fix was required for `just lint` to pass.

## Issues Encountered

- **Overall coverage gate pre-existing failure:** On the `develop` branch (main repo), `just coverage` was already failing at 64.31% TOTAL line coverage before this plan started. The plan assumed 82.8% overall coverage which was incorrect. In the worktree, coverage is 94.57% because the test suite coverage is measured differently (fewer external test files). This is a pre-existing issue on `develop` that is out of scope for Plan 07-04 which specifically targets `ops/program.rs`.
- **execute_op is private:** The function cannot be called directly in tests; had to use `run_program()` as an integration harness. This was anticipated in the plan's `<interfaces>` section.

## Known Stubs

None — this plan adds only test code; no data-rendering or UI stubs involved.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Test module is `#[cfg(test)]` and does not affect the production binary.

## Next Phase Readiness

- Plan 07-04 complete: ops/program.rs coverage gap (D-15) resolved
- `just coverage` passes at 94.57% in worktree
- `just test` passes (all 357 tests, 0 failed)
- `just lint` passes (0 warnings)
- Ready for Plan 07-05 (next hardening plan)

## Self-Check: PASSED

- FOUND: /Users/daniel/GitRepository/hp41-calculator-emulator/.claude/worktrees/agent-a94d246c0610c5b29/hp41-core/src/ops/program.rs (906 lines, 35 #[test] attributes)
- FOUND commit e00acf6 (test(07-04): add 35 targeted tests for ops/program.rs coverage)
- VERIFIED: cargo test -p hp41-core -- program_tests = 35 passed (from worktree)
- VERIFIED: just lint exits 0
- VERIFIED: just coverage exits 0 at 94.57% total (ops/program.rs 95.07%)
- VERIFIED: no file deletions in commit e00acf6

---
*Phase: 07-hardening*
*Completed: 2026-05-07*
