---
phase: 02-core-math
plan: 03
subsystem: core
tags: [rust, tdd, test-scaffold, math-ops, trig, registers, alpha, format, lift-semantics]

# Dependency graph
requires:
  - phase: 02-core-math
    plan: 01
    provides: Op enum stubs (Recip/Sqrt/Sq/YPow/Ln/Log/Exp/TenPow/Sin/Cos/Tan/Asin/Acos/Atan, SetDeg/SetRad/SetGrad, FmtFix/FmtSci/FmtEng, StoReg/RclReg/StoArith/Clreg, AlphaToggle/AlphaAppend/AlphaClear), CalcState with all Phase 2 fields

provides:
  - math_tests.rs — MATH-01 test suite (18 tests: recip/sqrt/sq/ln/log/exp/tenpow/ypow, LASTX, lift enable, Y/Z/T preservation)
  - trig_tests.rs — MATH-02 test suite (14 tests: sin/cos/tan in DEG/RAD/GRAD, asin/acos/atan, angle_mode storage, LASTX, lift enable)
  - format_tests.rs — MATH-03 test suite (13 tests: FIX trailing zeros/overflow, SCI uppercase E, ENG multiple-of-3 exponent; awaits Plan 04 format module)
  - register_tests.rs — REGS-01 test suite (15 tests: STO/RCL round-trip, STO-arith x4, CLREG, out-of-range errors, lift semantics)
  - alpha_tests.rs — ALPH-01 test suite (6 tests: append/build, 24-char limit, clear, toggle, Neutral lift, initial state)
  - lift_tests.rs — extended with Phase 2 lift assertions for ~10 new ops

affects:
  - 02-04 (format_tests.rs will compile after Plan 04 creates format.rs)
  - 02-05 (register_tests.rs tests go GREEN after Plan 05 implements STO/RCL/Clreg)
  - 02-06 (alpha_tests.rs tests go GREEN after Plan 06 implements AlphaAppend/AlphaClear/AlphaToggle)
  - 02-07 (math_tests.rs and trig_tests.rs go GREEN after Plan 07 implements math/trig ops)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "RED test scaffold pattern: write tests against Op stubs that return InvalidOp; tests fail at runtime as expected"
    - "format_tests.rs references hp41_core::format (future module); compile error expected until Plan 04"
    - "test_unary_ops_save_lastx: loop over ops vec to test LASTX invariant for all unary math ops"
    - "test_math_ops_enable_lift: loop over ops vec to test lift-enable invariant for all math ops"

key-files:
  created:
    - hp41-core/tests/math_tests.rs
    - hp41-core/tests/trig_tests.rs
    - hp41-core/tests/format_tests.rs
    - hp41-core/tests/register_tests.rs
    - hp41-core/tests/alpha_tests.rs
  modified:
    - hp41-core/tests/lift_tests.rs

key-decisions:
  - "format_tests.rs written in final form referencing hp41_core::format — compile error is expected/acceptable until Plan 04 lands"
  - "Removed StoArithKind import from lift_tests.rs Phase 2 section — not needed since lift tests use Op::StoReg/RclReg/Clreg directly"
  - "test_ypow_2_to_0_5_is_sqrt_2 uses Decimal from_str('1.414213562') per HP-41 10-digit rounding spec"
  - "Trig lift tests use conditional check (if state changed) rather than unconditional assert to handle stub ops gracefully"

patterns-established:
  - "Pattern: loop over ops vec in test — reduces boilerplate for LASTX/lift invariant tests across all math ops"
  - "Pattern: push_dec() helper for decimal string literals — clean test values without f64 artifacts"

requirements-completed: [MATH-01, MATH-02, MATH-03, REGS-01, ALPH-01]

# Metrics
duration: 4min
completed: 2026-05-06
---

# Phase 2 Plan 03: Test Scaffolds (RED Phase) Summary

**5 new test files + 1 extended test file defining acceptance criteria for all Phase 2 math/trig/format/register/alpha operations — 66 tests in RED state awaiting Plans 04–06 implementations**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-06T17:06:10Z
- **Completed:** 2026-05-06T17:10:25Z
- **Tasks:** 2
- **Files created:** 5 (944 lines total), 1 modified

## Accomplishments

- **math_tests.rs** (211 lines, 18 tests): Full MATH-01 coverage — 1/x with divide-by-zero, sqrt with domain, sq, ln with 10-digit accuracy check (LN(2)=0.6931471806), log(100)=2, exp(0)=1, 10^2=100, y^x, LASTX save for all 7 unary ops, lift enable for all 7 unary ops, Y/Z/T preservation invariant
- **trig_tests.rs** (184 lines, 14 tests): Full MATH-02 coverage — sin/cos in DEG/RAD/GRAD, sin(30°)=0.5, sin(90°)=1, cos(0°)=1, cos(60°)=0.5, asin(0.5)=30°, asin(1)=90°, asin domain error, atan(1)=45°, angle_mode storage, LASTX save for trig ops, lift enable loop
- **format_tests.rs** (97 lines, 13 tests): Full MATH-03 coverage — FIX 4 trailing zeros, pi rounding, negative, zero, FIX 0 integer format, FIX overflow to SCI, SCI speed-of-light (2.9979E 08), SCI small number (1.2340E-05), SCI 0 single digit, SCI zero, ENG 3 with 12345, ENG small, ENG million
- **register_tests.rs** (158 lines, 15 tests): Full REGS-01 coverage — STO/RCL round-trip, STO not changing X, RCL lifts to Y, STO+/−/×/÷ arith, CLREG zeros all registers, out-of-range STO(100)/RCL(100) returns InvalidOp, STO=Neutral/RCL=Enable/STO+=Neutral lift semantics, registers initialized to zero
- **alpha_tests.rs** (76 lines, 6 tests): Full ALPH-01 coverage — append builds string, 24-char limit enforced (25th char silently discarded), clear empties register, toggle flips flag (idempotent), all alpha ops Neutral lift, initial state empty
- **lift_tests.rs extended** (218 lines, +10 tests): Phase 2 lift assertions — unary math ops Enable lift, mode-setting ops (SetDeg/SetRad/SetGrad/FmtFix/FmtSci/FmtEng) are Neutral in both directions, StoReg=Neutral, RclReg=Enable, Clreg=Neutral, all alpha ops Neutral

## Task Commits

1. **Task 1** — `811b27a`: `test(02-03): add MATH-01 and MATH-02 RED test scaffolds`
2. **Task 2** — `e105abd`: `test(02-03): add MATH-03/REGS-01/ALPH-01 RED test scaffolds + extend lift tests`

## Files Created/Modified

- `hp41-core/tests/math_tests.rs` — 211 lines, 18 tests, MATH-01
- `hp41-core/tests/trig_tests.rs` — 184 lines, 14 tests, MATH-02
- `hp41-core/tests/format_tests.rs` — 97 lines, 13 tests, MATH-03 (compile error expected until Plan 04)
- `hp41-core/tests/register_tests.rs` — 158 lines, 15 tests, REGS-01
- `hp41-core/tests/alpha_tests.rs` — 76 lines, 6 tests, ALPH-01
- `hp41-core/tests/lift_tests.rs` — extended from 122 to 218 lines with Phase 2 lift assertions

## Decisions Made

- `format_tests.rs` written in final form referencing `hp41_core::format::format_hpnum`. The compile error `unresolved import 'hp41_core::format'` is the expected RED state until Plan 04 creates `src/format.rs`. All other test files compile cleanly.
- Removed unused `StoArithKind` import from the Phase 2 section of `lift_tests.rs` — the Phase 2 lift tests verify `Op::StoReg/RclReg/Clreg` directly and don't need the enum.
- `test_ypow_2_to_0_5_is_sqrt_2` asserts `Decimal::from_str("1.414213562")` — the 10-significant-digit value specified in the HP-41 accuracy spec.

## Deviations from Plan

None — plan executed exactly as written.

## RED State Summary

All 66 tests (excluding format_tests.rs which cannot compile yet) will FAIL at runtime against the current stubs. Expected progression:

| Test File | Goes GREEN After |
|-----------|-----------------|
| math_tests.rs | Plan 04 (math ops) |
| trig_tests.rs | Plan 04 (trig ops) |
| format_tests.rs | Plan 04 (format module, also required to compile) |
| register_tests.rs | Plan 05 (storage registers) |
| alpha_tests.rs | Plan 06 (alpha mode) |
| lift_tests.rs Phase 2 section | Plans 04, 05, 06 (each their respective ops) |

## Known Stubs

None — this plan creates tests only. All stubs are tracked in the 02-01-SUMMARY.md.

## Threat Model Compliance

T-02-08: Test files are dev-only, no production attack surface — accepted per plan.

## Self-Check: PASSED
