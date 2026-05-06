---
phase: 02-core-math
verified: 2026-05-06T12:00:00Z
status: gaps_found
score: 4/5
overrides_applied: 0
gaps:
  - truth: "User can cycle FIX 4 → SCI 2 → ENG 3 and the same number renders in the correct notation each time"
    status: partial
    reason: "Two correctness bugs in format.rs are not covered by any test: CR-01 (SCI/ENG mantissa rounding carry — 9.9995 in Sci(3) formats as '10.000E 00' instead of '1.000E 01') and CR-03 (FmtFix/FmtSci/FmtEng accept digit counts 0–255, not 0–9; FmtFix(11) produces garbage output with no error). Format works for typical inputs but is incorrect for edge cases that valid HP-41 inputs can produce."
    artifacts:
      - path: "hp41-core/src/format.rs"
        issue: "CR-01: format_sci and format_eng do not check whether the rounded mantissa has reached >= 10; if it does, sci_exp must be incremented by 1 and mantissa divided by 10 before formatting"
      - path: "hp41-core/src/ops/mod.rs"
        issue: "CR-03: FmtFix/FmtSci/FmtEng dispatch arms have no guard rejecting n > 9; HP-41 hardware only supports digit counts 0–9"
    missing:
      - "In format_sci and format_eng: after computing mantissa_rounded, check if it >= 10; if so, increment exponent by 1 and divide mantissa by 10 (re-clamp to multiple-of-3 for ENG)"
      - "In dispatch FmtFix/FmtSci/FmtEng arms: add 'if n > 9 { return Err(HpError::InvalidOp); }' before setting display_mode"
      - "Add test test_sci3_mantissa_carry: Sci(3) of 9.9995 must produce '1.000E 01' not '10.000E 00'"
      - "Add test test_fmt_fix_out_of_range: FmtFix(10) must return Err(HpError::InvalidOp)"
---

# Phase 2: Core Math — Verification Report

**Phase Goal:** Users can perform the complete HP-41 arithmetic, trigonometric, and formatting operation set, store and recall values in R00–R99 registers, and enter alphanumeric strings in ALPHA mode — all with HP-41-accurate 10-digit results.
**Verified:** 2026-05-06T12:00:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can compute `+ − × ÷`, `1/x`, `√x`, `x²`, `Y^X`, `LN`, `LOG`, `e^x`, `10^x` and see a 10-digit-accurate result | VERIFIED | All ops in math.rs, HpNum methods in num.rs; math_tests.rs 18 tests pass; LN(2) asserted equal to 0.6931471806; 92.63% coverage gate passes |
| 2 | User can press SIN/COS/TAN and inverses after switching DEG/RAD/GRAD; SIN(90°) = exactly 1 | VERIFIED | op_sin/cos/tan/asin/acos/atan in math.rs; trig_tests.rs 14 tests pass including `test_sin_90_deg_is_1` asserts `Decimal::from(1)` exactly; all three modes tested |
| 3 | User can cycle FIX 4 → SCI 2 → ENG 3 and same number renders in correct notation | PARTIAL | format.rs exists with format_hpnum(); format_tests.rs 13 tests pass for normal inputs; BUT CR-01: mantissa rounding carry bug (9.999999999 in Sci(4) produces "10.0000E 00" instead of "1.0000E 01") is live and untested; CR-03: FmtFix(11) produces garbage with no error, no guard in dispatch |
| 4 | User can STO into R00–R99, RCL, and STO+/−/×/÷ — all matching HP-41 hardware | VERIFIED | registers.rs fully implemented; register_tests.rs 15 tests pass; bounds-check at reg >= 100; STO-arith atomic; correct lift semantics (STO=Neutral, RCL=Enable) |
| 5 | User can activate ALPHA mode, type a 24-char string, confirm stored in ALPHA register | VERIFIED | alpha.rs implemented; alpha_tests.rs 6 tests pass; 24-char limit tested with silent discard; toggle, append, clear all verified |

**Score:** 4/5 truths fully verified (SC-3 is partial due to CR-01 and CR-03)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/Cargo.toml` | rust_decimal maths feature enabled | VERIFIED | `features = ["maths"]` present |
| `hp41-core/src/state.rs` | CalcState with regs[100], alpha_reg, alpha_mode, angle_mode, display_mode, entry_buf | VERIFIED | All fields present; AngleMode(Deg/Rad/Grad), DisplayMode(Fix/Sci/Eng) enums defined |
| `hp41-core/src/num.rs` | 14 HpNum math methods | VERIFIED | checked_recip, checked_sqrt, checked_sq, checked_ln, checked_log10, checked_exp, checked_exp10, checked_sin, checked_cos, checked_tan, checked_asin, checked_acos, checked_atan, checked_powd — all present and use HpNum::rounded() |
| `hp41-core/src/ops/mod.rs` | All Phase 2 Op variants + dispatch + flush_entry_buf | VERIFIED | All 27 Op variants present; flush_entry_buf called at dispatch start; FmtFix/Sci/Eng lack digit range guard (CR-03) |
| `hp41-core/src/ops/math.rs` | All 17 math/trig/angle ops | VERIFIED | op_recip through op_set_grad all implemented using unary_result/binary_result; f64 bridge for forward trig and inverse trig |
| `hp41-core/src/ops/registers.rs` | op_sto, op_rcl, op_sto_arith, op_clreg | VERIFIED | Bounds-check, atomic write for STO-arith, correct lift semantics |
| `hp41-core/src/ops/alpha.rs` | op_alpha_toggle, op_alpha_append, op_alpha_clear | VERIFIED | 24-char guard uses .chars().count(); all Neutral lift |
| `hp41-core/src/format.rs` | format_hpnum() with FIX/SCI/ENG modes | PARTIAL | Implemented; normal inputs correct; CR-01 mantissa carry bug; fallback for FIX overflow present |
| `hp41-core/tests/math_tests.rs` | MATH-01 test suite | VERIFIED | 18 tests, 80+ lines, covers all unary ops, LASTX, lift |
| `hp41-core/tests/trig_tests.rs` | MATH-02 test suite | VERIFIED | 14 tests, covers DEG/RAD/GRAD, ASIN/ACOS/ATAN, SIN(90°)=1 |
| `hp41-core/tests/format_tests.rs` | MATH-03 test suite | PARTIAL | 13 tests pass; missing: mantissa carry edge case, out-of-range digit count |
| `hp41-core/tests/register_tests.rs` | REGS-01 test suite | VERIFIED | 15 tests pass |
| `hp41-core/tests/alpha_tests.rs` | ALPH-01 test suite | VERIFIED | 6 tests pass |
| `hp41-core/tests/entry_buf_tests.rs` | Entry buffer flush integration | VERIFIED | 10 tests pass; flush on math op, ENTER, STO; invalid buf error |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ops/mod.rs` | `ops/math.rs` | `math::op_sin(state)` etc. | WIRED | `pub mod math;` declared; all 17 math variants dispatched |
| `ops/mod.rs` | `ops/registers.rs` | `registers::op_sto(state, r)` | WIRED | `pub mod registers;` declared; StoReg/RclReg/StoArith/Clreg dispatched |
| `ops/mod.rs` | `ops/alpha.rs` | `alpha::op_alpha_append(state, ch)` | WIRED | `pub mod alpha;` declared; AlphaToggle/AlphaAppend/AlphaClear dispatched |
| `ops/math.rs` | `num.rs` | `state.stack.x.checked_ln()` | WIRED | All 14 HpNum methods called from corresponding op functions |
| `ops/math.rs` | `stack.rs` | `unary_result(state, result)` | WIRED | All unary ops use unary_result; YPow uses binary_result |
| `ops/registers.rs` | `state.rs` | `state.regs[reg as usize]` | WIRED | Direct array access with bounds check |
| `ops/alpha.rs` | `state.rs` | `state.alpha_reg.push(ch)` | WIRED | Direct field mutation |
| `ops/mod.rs` | `state.rs` | `flush_entry_buf` clears `state.entry_buf` | WIRED | Called at dispatch() start |
| `lib.rs` | `format.rs` | `pub mod format; pub use format::format_hpnum` | WIRED | Re-exported at crate root |
| `format_tests.rs` | `format.rs` | `hp41_core::format::format_hpnum()` | WIRED | Import resolves; 13 tests compile and pass |

### Data-Flow Trace (Level 4)

Not applicable — this phase produces library logic (pure functions), not rendering components. All state flows through `CalcState` passed by mutable reference.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 238 tests pass | `cargo test -p hp41-core` | 238 passed, 0 failed | PASS |
| Clippy clean | `cargo clippy -- -D warnings` | No warnings | PASS |
| Coverage gate ≥80% | `cargo llvm-cov --fail-under-lines 80 -p hp41-core` | 94.51% line coverage | PASS |
| SIN(90°) = 1 exactly | `test_sin_90_deg_is_1` | `Decimal::from(1)` | PASS |
| LN(2) = 0.6931471806 | `test_ln_2_accuracy_10_digits` | exact match | PASS |
| STO/RCL round-trip | `test_sto_rcl_round_trip` | passes | PASS |
| ALPHA 24-char limit | `test_alpha_24_char_limit_enforced` | 24 chars stored, 25th discarded | PASS |
| CR-01: 9.9995 in Sci(3) | No test exists | Untested — bug live | FAIL |
| CR-03: FmtFix(10) → InvalidOp | No test exists | Untested — no guard | FAIL |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| MATH-01 | 02-01, 02-02, 02-03, 02-04 | Core arithmetic + math ops with 10-digit accuracy | SATISFIED | All ops in math.rs; HpNum methods in num.rs; math_tests.rs passes; LN(2) exact to 10 digits |
| MATH-02 | 02-01, 02-02, 02-03, 02-04 | Trig in DEG/RAD/GRAD with inverses | SATISFIED | ops implemented; trig_tests.rs passes; SIN(90°)=1 exactly verified |
| MATH-03 | 02-01, 02-03, 02-05 | FIX/SCI/ENG display modes n=0-9 | PARTIALLY SATISFIED | format.rs exists; normal inputs correct; CR-01 mantissa carry bug; CR-03 missing digit range guard |
| REGS-01 | 02-01, 02-03, 02-05 | R00-R99 with STO/RCL/STO-arith | SATISFIED | registers.rs; register_tests.rs passes; all edge cases covered |
| ALPH-01 | 02-01, 02-03, 02-06 | 24-char ALPHA register | SATISFIED | alpha.rs; alpha_tests.rs passes; 24-char limit enforced |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `format.rs` | 88–100, 127–133 | Mantissa rounded after exponent computed — no carry normalization (CR-01) | BLOCKER | Numbers like 9.9995 in SCI(3) produce "10.000E 00" instead of "1.000E 01" — display incorrect for valid HP-41 values near powers of 10 |
| `ops/mod.rs` | 182–195 | FmtFix/FmtSci/FmtEng dispatch arms accept n=0–255, no guard (CR-03) | BLOCKER | FmtFix(11) causes FIX overflow fallback to trigger for all numbers ≥1; FmtFix(200) produces 200-digit mantissa — both are user-visible correctness violations |
| `num.rs` | 130 | `checked_powd` returns `HpError::Domain` for overflow conditions (CR-02) | WARNING | Callers that distinguish Domain vs Overflow for display messaging receive wrong information; does not affect arithmetic correctness for normal HP-41 inputs |
| `ops/math.rs` | 25, 28 | `.unwrap()` on `Decimal::from_str` for hardcoded constants in `pi_over_180`/`pi_over_200` (WR-01) | WARNING | Technically violates zero-panic invariant; currently dead code; safe in practice since literals are valid |
| `num.rs` | 141, 148, 155 | `checked_sin/cos/tan` map `None` to `HpError::Domain` instead of `HpError::Overflow` (WR-02) | WARNING | Wrong error type for very large angular inputs; does not affect accuracy for inputs in valid range |
| `trig_tests.rs` | 178–183 | `test_trig_ops_enable_lift` is vacuous — checks `if s.stack.lift_enabled { assert!(s.stack.lift_enabled) }` (WR-06) | WARNING | Test will always pass regardless of whether trig ops correctly enable lift; provides false coverage assurance |
| `lift_tests.rs` | entire file | Op::Sqrt, Op::YPow, Op::Sin, Op::Cos, Op::Tan, Op::Asin, Op::Acos, Op::Atan absent (WR-07) | WARNING | 8 Enable-lift ops have no canonical lift regression test in lift_tests.rs |

### Human Verification Required

None — all phase-2 behaviors are programmatically verifiable.

### Gaps Summary

**Two blocking correctness bugs in format.rs** prevent the phase goal from being fully achieved for MATH-03:

**Gap 1 — CR-01: SCI/ENG mantissa carry bug**
`format_sci` and `format_eng` compute the scientific exponent first, then round the mantissa. When rounding carries the mantissa from 9.999... to 10.0000, the code formats it as "10.0000E 00" instead of normalizing to "1.0000E 01". This is a silent wrong output for valid HP-41 register values. The fix requires a post-rounding normalization step in both `format_sci` and `format_eng`.

**Gap 2 — CR-03: FmtFix/FmtSci/FmtEng accept out-of-range digit counts**
The HP-41 hardware supports digit counts 0–9 only. The dispatch arms for FmtFix/FmtSci/FmtEng accept any u8 (0–255) without validation. FmtFix(11) causes the overflow threshold to be 1, making every number ≥1 fall back to SCI mode. FmtFix(200) produces a 200-digit mantissa. Both are correctness violations with no error returned. The fix is a 3-line guard added to each dispatch arm.

**Neither bug has test coverage** — format_tests.rs tests normal inputs only.

The three remaining requirements (MATH-01, MATH-02, REGS-01, ALPH-01) are fully satisfied with substantive implementations and passing tests. The entry buffer flush (Plan 07) correctly wires digit entry to the dispatch pipeline. The full CI gate (lint + 238 tests + 92.63% line coverage) passes.

---

_Verified: 2026-05-06T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
