---
phase: 02-core-math
plan: 04
subsystem: core-model
tags: [rust, hp41-core, ops, math, trig, angle-mode, f64-bridge, rust_decimal, tdd]

# Dependency graph
requires:
  - 02-02 (HpNum checked_* math methods: checked_recip/sqrt/sq/ln/log10/exp/exp10/powd/sin/cos/tan/asin/acos/atan)
  - 02-03 (RED test scaffolds: math_tests.rs and trig_tests.rs with failing integration tests)
provides:
  - ops/math.rs with all 17 Phase 2 math/trig/angle-mode operations
  - dispatch() wired for Recip, Sqrt, Sq, YPow, Ln, Log, Exp, TenPow, Sin, Cos, Tan, Asin, Acos, Atan, SetDeg, SetRad, SetGrad
  - All math_tests (18) and trig_tests (14) passing GREEN
affects: [02-05 registers/format ops, 02-06 alpha ops, all future plans calling dispatch()]

# Tech tracking
tech-stack:
  added:
    - std::f64::consts::PI (for f64-native angle conversion in trig ops)
    - rust_decimal::prelude::ToPrimitive (for HpNum inner → f64 conversion in trig)
    - rust_decimal::prelude::FromPrimitive (for f64 → Decimal in trig result)
  patterns:
    - All ops dispatch through dispatch() → math:: function names
    - Unary math ops: checked_* → unary_result() (saves LASTX, enables lift, no stack drop)
    - Binary math op (YPow): checked_powd → binary_result() (saves LASTX, drops Y)
    - Angle mode ops: set field → apply_lift_effect(Neutral)
    - Trig (forward + inverse): full f64 bridge with single HpNum::rounded() at end (avoids double-rounding)

key-files:
  created:
    - hp41-core/src/ops/math.rs
  modified:
    - hp41-core/src/ops/mod.rs

key-decisions:
  - "All trig ops use direct f64 bridge (HpNum → f64 → trig/angle → Decimal::from_f64 → HpNum::rounded) to avoid double-rounding precision errors in canonical angles"
  - "Constants pi_over_180/pi_over_200 stored as raw HpNum(Decimal) bypassing HpNum::from() rounding — required for correct DEG/GRAD conversion precision"
  - "Forward and inverse trig both use f64-native angle conversion (to_radians_f64 / f64_from_radians) rather than Decimal multiplication"

patterns-established:
  - "Pattern: Trig ops convert angle in f64 space, compute trig in f64, then call HpNum::rounded() once — never chain two rounded HpNum operations in the trig path"
  - "Pattern: to_radians_hpnum kept dead_code for potential non-trig angle conversion uses in future plans"

requirements-completed: [MATH-01, MATH-02]

# Metrics
duration: 15min
completed: 2026-05-06
---

# Phase 2 Plan 04: Math Operations GREEN Summary

**ops/math.rs with 17 HP-41 math/trig/angle ops wired into dispatch() using f64-bridge trig for 10-digit precision accuracy on canonical angles**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-06T17:30:00Z
- **Completed:** 2026-05-06T17:45:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Created `hp41-core/src/ops/math.rs` with 17 public op functions (14 unary, 1 binary, 3 angle mode)
- Wired all 17 op variants in `dispatch()` (replaced `InvalidOp` stubs)
- All 18 math_tests pass (recip, sqrt, sq, ln, log, exp, tenpow, ypow, LASTX, lift)
- All 14 trig_tests pass (sin/cos/tan in DEG/RAD/GRAD, asin/acos/atan, angle mode state, LASTX, lift)
- Phase 1 stack_tests (18) still pass — no regressions

## Task Commits

1. **Task 1: Create ops/math.rs** — `24d8b31` (feat)
2. **Task 2: Wire math module into dispatch(); all tests GREEN** — `94e4df7` (feat)

## Files Created/Modified

- `hp41-core/src/ops/math.rs` — New file: 17 op functions, private trig helpers, f64-bridge angle conversion
- `hp41-core/src/ops/mod.rs` — Uncommented `pub mod math;`, added import block, replaced 17 stub dispatch arms

## Decisions Made

- **f64 bridge for all trig ops:** Both forward (sin/cos/tan) and inverse (asin/acos/atan) ops use a complete f64 computation path (HpNum → f64 → compute → Decimal::from_f64 → HpNum::rounded). This avoids the "double rounding" error where: (1) `HpNum::rounded()` truncates the intermediate radian value to 10 sig digits, then (2) multiplying by a deg_per_rad constant loses another digit. For canonical angles like COS(60°)=0.5 and ASIN(1)=90°, the f64 path produces exact results.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] All trig ops redesigned to use f64 bridge to avoid double-rounding**
- **Found during:** Task 2 (trig_tests run)
- **Issue:** Plan specified `to_radians(x, mode)` → `checked_sin/cos/tan` and `checked_asin/acos/atan` → `from_radians(result, mode)` using HpNum multiplication with Decimal constants. This caused double-rounding: `checked_mul` rounds the intermediate radian value to 10 sig digits, then `from_radians` multiply adds more rounding error. Result: `COS(60°) = 0.5000000002` (not 0.5), `ASIN(1) = 90.00000001°` (not 90).
- **Root cause:** `1.570796327` (pi/2 rounded to 10 sig digits) × `57.29577951308232...` = `90.00000001...` — exactly 10 sig digits, so `HpNum::rounded()` cannot fix it.
- **Fix:** All 6 trig ops (sin/cos/tan/asin/acos/atan) now: convert HpNum X to f64 → compute angle conversion and trig in f64 → `Decimal::from_f64(result) → HpNum::rounded()` once. This matches how `checked_asin/acos/atan` already worked in num.rs (f64 round-trip bridge) and extends it to forward trig and angle conversion.
- **Files modified:** `hp41-core/src/ops/math.rs`
- **Verification:** `cargo test -p hp41-core --test trig_tests` — 14/14 passed (was 12/14 before fix)
- **Committed in:** `94e4df7` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - precision bug in trig angle conversion)
**Impact on plan:** Fix required for test correctness. No scope changes. The f64 bridge pattern was already established by `checked_asin/acos/atan` in Plan 02 — this simply extends it consistently to all trig ops.

## Issues Encountered

- `HpNum::from(Decimal)` calls `HpNum::rounded()` which pre-rounds constants to 10 sig digits. Used `HpNum(raw_decimal)` (the `pub(crate)` inner field) for the angle conversion constants to keep them at full precision.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 17 math/trig/angle-mode ops are live in dispatch()
- Plans 05 (format.rs, registers.rs) and 06 (alpha.rs) can now implement their ops using the same pattern
- `lift_tests` partial failures (6/20 failing) are pre-existing stubs for Plans 05-06 ops — not regressions

## Known Stubs

None — all 17 ops are fully implemented with correct behavior.

## Threat Flags

None — this plan adds no new network endpoints, auth paths, file access patterns, or schema changes. All code is pure local arithmetic in hp41-core.

---
*Phase: 02-core-math*
*Completed: 2026-05-06*

## Self-Check: PASSED

Files verified:
- FOUND: hp41-core/src/ops/math.rs
- FOUND: hp41-core/src/ops/mod.rs

Commits verified:
- FOUND: 24d8b31 (feat — Task 1: create ops/math.rs)
- FOUND: 94e4df7 (feat — Task 2: wire dispatch, tests GREEN)
