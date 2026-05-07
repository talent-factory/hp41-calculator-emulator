---
phase: 05-persistence-and-ux
plan: 11
subsystem: testing
tags: [rust, programs, gcd, stack-stats, behavioral-tests]

requires:
  - phase: 05-persistence-and-ux
    provides: programs.rs with bundled sample programs (base for fixes)
  - phase: 05-09
    provides: Op::Int fix in prime_test_ops (template for CR-02 fix)

provides:
  - Correct gcd_ops with Op::Int after Op::Div (exact integer modulo via BCD truncation)
  - Correct stack_stats_ops with register-save pattern accessing all 4 stack values
  - test_gcd_correctness: gcd(12,8)=4, gcd(7,3)=1, gcd(15,5)=5
  - test_stack_stats_correctness: X=1 (min), Y=5 (max) for inputs [3,1,4,5]

affects: [phase-6, any phase consuming bundled program correctness]

tech-stack:
  added: []
  patterns:
    - "Register-save pattern (StoReg R00-R03 via Rdn cycling) for correct 4-value stack comparisons"
    - "BCD integer modulo: Op::Div, Op::Int, Mul, Sub — matches prime_test_ops pattern"

key-files:
  created: []
  modified:
    - hp41-cli/src/programs.rs

key-decisions:
  - "stack_stats_ops rewritten with R00-R03 save pattern: Enter+Rdn loop never reached Z (minimum was in Z for test inputs); only register-save guarantees all 4 values are compared"
  - "gcd_ops fix mirrors prime_test_ops fix exactly: Op::Int truncates BCD quotient before multiply-back"

patterns-established:
  - "Op::Int after Op::Div: required for exact integer arithmetic on HP-41 BCD values"
  - "RclReg pairwise comparison for stack min/max: saves T/Z/Y/X to R00-R03 first, then RclReg(N) brings each value for comparison with running min/max in Y"

requirements-completed: [UX-03]

duration: 15min
completed: 2026-05-07
---

# Plan 05-11: gcd_ops CR-02 and stack_stats_ops CR-03 Summary

**gcd_ops fixed with Op::Int for exact BCD modulo; stack_stats_ops rewritten with R00-R03 register-save pattern covering all 4 stack values; two behavioral tests added — SC-5 fully satisfied**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-07T19:00:00Z
- **Completed:** 2026-05-07T19:15:00Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Fixed gcd_ops CR-02: inserted `Op::Int` after `Op::Div` in the modulo step; BCD division of 7/3 yields 2.333…; without Int, 2.333…×3=6.999… never reaches zero; with Int, 2×3=6 and 7-6=1, correct Euclidean remainder
- Fixed stack_stats_ops CR-03: rewritten with R00-R03 register-save pattern. The Enter+Rdn approach (original and first-pass fix) only compared X and Y from the initial stack; Z was never directly reached. The new algorithm saves all 4 values first, then RclReg pairwise comparisons with XLtY (max) and XGtY (min) cover all four registers
- Added test_gcd_correctness and test_stack_stats_correctness; hp41-cli test suite grows from 47 to 49 passing tests; all 10 bundled programs produce documented outputs (SC-5 verified)

## Task Commits

All three tasks combined in one atomic commit:

1. **Task 1: Fix gcd_ops Op::Int** — `7029962` (fix)
2. **Task 2: Rewrite stack_stats_ops** — `7029962` (fix)
3. **Task 3: Add behavioral tests** — `7029962` (test)

## Files Created/Modified

- `hp41-cli/src/programs.rs` — gcd_ops Op::Int fix, stack_stats_ops register-save rewrite, two new behavioral tests

## Decisions Made

- **stack_stats_ops algorithm rewritten (not just test-condition inversion):** Plan CR-03 specified inverting 6 Test conditions, but tracing revealed the Enter+Rdn loop compares X against Y repeatedly and never surfaces Z to X for comparison. The minimum (1) was in Z for the test inputs [3,1,4,5]; simply inverting conditions left R04=4 (wrong). The register-save approach (StoReg R00-R03 via three Rdn ops, then RclReg pairwise comparisons) is the correct fix. Key_links are fully satisfied: max section uses XLtY, min section uses XGtY.

## Deviations from Plan

### Auto-fixed Issues

**1. [Algorithmic deviation] stack_stats_ops requires register-save, not condition inversion alone**
- **Found during:** Task 2 (stack_stats_ops fix)
- **Issue:** Plan specified inverting 6 TestKind conditions in the Enter+Rdn loop. Traced execution shows the loop only ever compares original X vs original Y (the Enter duplicate and the original Y); the third Rdn+RclReg step repeats the same X-vs-Y comparison. For test input T=3,Z=1,Y=4,X=5, R04 was set to 4 (first XGtY swap), and 1 (the true min in Z) was never brought to X for comparison
- **Fix:** Rewrote stack_stats_ops to save all 4 values to R00-R03 via Rdn cycling first, then use RclReg(0..3) with pairwise Test+XySwap — guarantees all 4 registers are compared. Final algorithm is longer but correct for all stack configurations
- **Files modified:** hp41-cli/src/programs.rs
- **Verification:** test_stack_stats_correctness passes (X=1 min, Y=5 max); cargo test -p hp41-cli: 49 passed
- **Committed in:** 7029962

---

**Total deviations:** 1 auto-fixed (algorithm deviation — register-save required instead of condition-only inversion)
**Impact on plan:** Fix is strictly additive, uses no new ops or APIs, satisfies all plan key_links (XLtY for max, XGtY for min), and passes all verification criteria.

## Issues Encountered

None beyond the algorithmic deviation documented above.

## Next Phase Readiness

- All 10 bundled sample programs produce documented outputs (SC-5 verified)
- hp41-cli: 49 tests passing, 0 failed, clippy clean
- Phase 5 gap closure complete; Phase 6 (Science & Engineering) ready to plan

## Self-Check: PASSED

- [x] gcd_ops contains Op::Int immediately after Op::Div in modulo step
- [x] All three max-section Tests in stack_stats_ops use TestKind::XLtY
- [x] All three min-section Tests in stack_stats_ops use TestKind::XGtY
- [x] test_gcd_correctness passes: gcd(12,8)=4, gcd(7,3)=1, gcd(15,5)=5
- [x] test_stack_stats_correctness passes: X=1 (min), Y=5 (max) for inputs [3,1,4,5]
- [x] cargo test -p hp41-cli: 49 passed, 0 failed
- [x] cargo clippy -p hp41-cli --all-targets -- -D warnings: no issues

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
