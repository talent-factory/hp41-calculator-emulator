---
phase: 05-persistence-and-ux
plan: 09
subsystem: testing
tags: [rust, hp41-core, programs, prime-test, trial-division, integer-truncation]

requires:
  - phase: 05-08
    provides: CI quality gate, all prior plans passing

provides:
  - Corrected prime_test_ops: XySwap removed, loop exit fixed (XGtY→XLtY), exact integer modulo via Op::Int
  - New Op::Int operation (HP-41 INT function) in hp41-core for exact integer truncation
  - Replaced mean_sdev_ops with correct 4-value stack mean (no indirect addressing needed)
  - Fixed quadratic_ops comment: "c in X, b in Y, a in Z" (matches actual StoReg sequence)
  - Behavioural tests: test_prime_test_correctness and test_stack_mean_correctness
  - SC-5 gap closure: 7 programs::tests pass, prime(2,3,13)=1 and prime(4,9)=0 verified

affects:
  - 05-10 (help overlay 'q' routing)
  - future plans using hp41-core ops (Op::Int now available for integer arithmetic programs)

tech-stack:
  added: []
  patterns:
    - "Op::Int (integer truncation) added to hp41-core as standard HP-41 operation"
    - "Integer modulo n mod d = n - d * int(n/d) pattern using Op::Int for exact BCD results"
    - "HP-41 Test semantics: TRUE=execute next, FALSE=skip next (XLtY for loop exit when d²>n)"

key-files:
  created: []
  modified:
    - hp41-cli/src/programs.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/math.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/num.rs
    - hp41-cli/src/prgm_display.rs

key-decisions:
  - "Op::Int added to hp41-core (missing standard HP-41 function needed for correct integer modulo)"
  - "Integer modulo via n - d*int(n/d) using Op::Int instead of n - d*(n/d) which gives 0 for any terminating decimal quotient"
  - "Loop exit test changed from XGtY to XLtY: at Test point X=n, Y=d²; n<d² (TRUE) means prime"

patterns-established:
  - "BCD modulo correctness: always use Op::Int to truncate quotient before multiplication to avoid terminating-decimal false zeros"
  - "HP-41 conditional pattern: TRUE=execute next, FALSE=skip next (opposite of skip-if-true)"

requirements-completed: [UX-03]

duration: 28min
completed: 2026-05-07
---

# Phase 5 Plan 9: Prime Test Bug Fix and Behavioural Tests Summary

**Three bugs in sample programs fixed: spurious XySwap removed from prime_test_ops, Op::Int added to hp41-core for exact integer modulo, loop exit condition corrected (XLtY), mean_sdev_ops replaced with correct 4-value stack mean, quadratic comment fixed — all 7 programs::tests now pass**

## Performance

- **Duration:** 28 min
- **Started:** 2026-05-07T16:56:00Z
- **Completed:** 2026-05-07T17:24:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Removed spurious `Op::XySwap` from prime_test_ops early-exit path (plan-identified bug)
- Added `Op::Int` (HP-41 INT function) to hp41-core; fixed modulo computation from `n-d*(n/d)` (wrong for terminating decimals like 3/2=1.5) to `n-d*int(n/d)` (exact integer truncation); fixed loop exit test from `XGtY` to `XLtY`
- Replaced broken mean_sdev_ops (unconditional RclReg(0)) with a correct 7-op 4-value stack mean
- Fixed quadratic_ops comment to accurately document stack entry order (c in X, b in Y, a in Z)
- Added `test_prime_test_correctness` and `test_stack_mean_correctness` behavioural tests; all 7 programs::tests pass

## Task Commits

1. **Task 1: Fix prime_test_ops — remove spurious XySwap** - `dcb59eb` (fix)
2. **Task 2: Replace mean_sdev_ops; fix quadratic_ops comment** - `0e9ea40` (fix)
3. **Task 3: Add Op::Int, fix prime_test_ops loop bugs, add behavioural tests** - `1793268` (feat)

## Files Created/Modified

- `hp41-cli/src/programs.rs` — prime_test_ops corrected (XySwap removed, loop exit XLtY, Op::Int modulo); mean_sdev_ops replaced (7-op stack mean); quadratic_ops comment fixed; 2 new behavioural tests added
- `hp41-core/src/ops/mod.rs` — Op::Int variant added to enum; op_int wired in dispatch()
- `hp41-core/src/ops/math.rs` — op_int() implemented using Decimal::trunc() via HpNum::trunc_int()
- `hp41-core/src/ops/program.rs` — Op::Int added to execute_op() match for program execution
- `hp41-core/src/num.rs` — trunc_int() method added to HpNum (wraps Decimal::trunc())
- `hp41-cli/src/prgm_display.rs` — Op::Int display name "INT" added to exhaustive match

## Decisions Made

- Added Op::Int to hp41-core rather than working around the missing operation. The HP-41 INT function is a standard calculator operation; its absence was a gap causing incorrect modulo computation for non-divisible integer pairs.
- Changed loop exit test from `XGtY` to `XLtY`: at the Test point, stack is X=n, Y=d². The correct prime condition is d²>n (we've checked all divisors through √n), which in HP-41 terms means n<d² (X<Y). `XLtY` evaluates TRUE when n<d², causing Gto("P") to execute (prime route). `XGtY` was inverting this logic.
- Used HpNum::trunc_int() wrapping Decimal::trunc() (truncates toward zero) rather than floor/round — matches HP-41 INT behavior for positive integers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed prime_test_ops loop exit test XGtY→XLtY**
- **Found during:** Task 3 (behavioural tests — prime(3) returned 0 instead of 1)
- **Issue:** After removing the early-exit XySwap, prime(3) entered the divisor loop. Loop exit `Test(XGtY)` with X=n=3, Y=d²=4: 3>4 is FALSE → skip Gto("P") → fall through to modulo check. divisor d=3 then gives n mod d=0 (3/3=1, 1*3=3, 3-3=0), incorrectly classifying 3 as composite.
- **Fix:** Changed `TestKind::XGtY` to `TestKind::XLtY` at loop exit. Now n<d² → TRUE → execute Gto("P") (prime). Verified: prime(3) with d=2, d²=4: 3<4=TRUE → prime ✓
- **Files modified:** hp41-cli/src/programs.rs
- **Verification:** test_prime_test_correctness asserts prime(3)=1; tests pass
- **Committed in:** 1793268 (Task 3 commit)

**2. [Rule 1 - Bug] Fixed prime_test_ops modulo via n-d*(n/d) giving 0 for terminating decimals**
- **Found during:** Task 3 (analysis of prime(13) failure risk)
- **Issue:** In BCD arithmetic, `n - d*(n/d)` gives 0 whenever n/d produces a terminating decimal (e.g., 13/2=6.5, 2*6.5=13.0, 13-13=0). This would cause prime(13) to appear divisible by 2. The old code only accidentally avoided this because the inverted XGtY check jumped to prime before the modulo ran.
- **Fix:** Added `Op::Int` (new HP-41 INT operation) to hp41-core. Changed modulo sequence to use `int(n/d)` (truncated quotient) instead of `n/d` (exact quotient). Now: 13/2=6.5, int(6.5)=6, 6*2=12, 13-12=1 (non-zero → not divisible) ✓
- **Files modified:** hp41-core/src/num.rs, hp41-core/src/ops/math.rs, hp41-core/src/ops/mod.rs, hp41-core/src/ops/program.rs, hp41-cli/src/prgm_display.rs, hp41-cli/src/programs.rs
- **Verification:** test_prime_test_correctness asserts prime(2,3,13)=1 and prime(4,9)=0; 288 hp41-core tests pass; clippy: no warnings
- **Committed in:** 1793268 (Task 3 commit)

**3. [Rule 2 - Missing Critical] Added Op::Int to hp41-core Op enum**
- **Found during:** Task 3 (root cause analysis of modulo bug)
- **Issue:** Op::Int (HP-41 INT function — truncate toward zero) was absent from the Op enum. Without it, programs relying on integer division are forced to use inexact floating-point modulo, causing false divisibility for terminating-decimal quotients.
- **Fix:** Added `Op::Int` variant to Op enum, `trunc_int()` method to HpNum using Decimal::trunc(), `op_int()` in math.rs, wired in dispatch() (mod.rs) and execute_op() (program.rs). Display name "INT" added to prgm_display.rs.
- **Files modified:** hp41-core/src/ops/mod.rs, hp41-core/src/ops/math.rs, hp41-core/src/ops/program.rs, hp41-core/src/num.rs, hp41-cli/src/prgm_display.rs
- **Verification:** cargo build passes; 288 hp41-core tests pass; zero clippy warnings
- **Committed in:** 1793268 (Task 3 commit)

---

**Total deviations:** 3 auto-fixed (2 Rule 1 bugs, 1 Rule 2 missing critical)
**Impact on plan:** All auto-fixes required for correctness. Bugs 1 and 2 were masked in the original code by the primary bug (XySwap inverting early exit) — removing that bug exposed two deeper algorithmic flaws. Op::Int addition is a one-file standard HP-41 operation with no architectural impact.

## Issues Encountered

The VERIFICATION.md identified the primary bug as "XySwap before Test(XLeY)" causing prime to always return 1 for n≥2. After removing the XySwap, the tests revealed two additional bugs: the loop exit test direction was inverted (XGtY should be XLtY), and the modulo algorithm using `n - d*(n/d)` fails for any n/d that produces a terminating decimal (e.g., 3/2=1.5, giving a false zero remainder). Both were fixed via deviation rules.

## Next Phase Readiness

- SC-5 gap fully closed: prime_test_ops, mean_sdev_ops, quadratic_ops all corrected
- Op::Int now available for future programs requiring integer arithmetic
- 05-10 (help overlay 'q' routing) is independent and can proceed
- All 7 programs::tests pass, 288 hp41-core tests pass

## Self-Check

Verified files exist:
- hp41-cli/src/programs.rs: FOUND (modified, committed in dcb59eb, 0e9ea40, 1793268)
- hp41-core/src/ops/mod.rs: FOUND (modified, committed in 1793268)
- hp41-core/src/ops/math.rs: FOUND (modified, committed in 1793268)
- hp41-core/src/num.rs: FOUND (modified, committed in 1793268)
- .planning/phases/05-persistence-and-ux/05-09-SUMMARY.md: this file

Verified commits exist:
- dcb59eb: fix(05-09): remove spurious XySwap from prime_test_ops early-exit path
- 0e9ea40: fix(05-09): replace mean_sdev_ops with 4-value stack mean; fix quadratic comment
- 1793268: feat(05-09): add Op::Int, fix prime_test_ops loop bugs, add behavioural tests

All 7 programs::tests pass. 288 hp41-core tests pass. Zero clippy warnings.

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
