---
phase: 02-core-math
plan: 05
subsystem: core-model
tags: [rust, hp41-core, registers, format, sto, rcl, fix, sci, eng, display, rust_decimal, tdd]

# Dependency graph
requires:
  - 02-01 (CalcState with regs[100], DisplayMode enum, AngleMode)
  - 02-02 (HpNum checked_add/sub/mul/div for STO-arith)
provides:
  - op_sto: copy X into storage register R00–R99
  - op_rcl: recall register into X with stack lift (Enable)
  - op_sto_arith: STO+/−/×/÷ with atomicity guarantee
  - op_clreg: clear all 100 storage registers to zero
  - format_hpnum: FIX/SCI/ENG display formatting per HP-41 spec
  - format_alpha: ALPHA register truncation to 12 chars
  - FmtFix/FmtSci/FmtEng dispatch arms: update state.display_mode
affects: [02-06 (alpha ops use same dispatch pattern), 02-07 (trig integration tests use format_hpnum), hp41-cli TUI display]

# Tech tracking
tech-stack:
  added:
    - rust_decimal::RoundingStrategy::MidpointAwayFromZero (for FIX rounding — HP-41 hardware behavior)
  patterns:
    - decimal_pow10(exp) helper: builds 10^n as Decimal without E notation (rust_decimal from_str rejects "1E-8")
    - STO-arith atomicity: compute new_val first via checked_*?, write state.regs[reg] only on success
    - RCL lift: set lift_enabled = true BEFORE enter_number(), then apply_lift_effect(Enable) after
    - FIX overflow: if abs >= 10^(10-digits), fall back to format_sci(d, 9) — matches HP-41 hardware

key-files:
  created:
    - hp41-core/src/ops/registers.rs
    - (hp41-core/src/format.rs replaced stub with real implementation)
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/lib.rs
    - hp41-core/tests/format_tests.rs

key-decisions:
  - "rust_decimal Decimal::from_str() rejects E notation ('1E-8' fails) — use decimal_pow10() with string-built decimal literals"
  - "FIX rounding: use round_dp_with_strategy(MidpointAwayFromZero) before format! — Decimal's default Display uses Banker's rounding"
  - "Test fix: test_fix4_overflow_to_sci used num('1E15') which panics on rust_decimal parse; replaced with decimal notation '1000000000000000'"

requirements-completed: [MATH-03, REGS-01]

# Metrics
duration: 5min
completed: 2026-05-06
---

# Phase 2 Plan 05: Storage Registers + Display Formatting Summary

**STO/RCL/STO-arith/CLREG register operations with bounds-check and atomic write, plus FIX/SCI/ENG HP-41 display formatting with MidpointAwayFromZero rounding and decimal_pow10 scaling**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-06T17:22:52Z
- **Completed:** 2026-05-06T17:28:38Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- 4 register ops (op_sto, op_rcl, op_sto_arith, op_clreg) with HP-41-correct lift semantics
- Bounds check `reg >= 100 → InvalidOp` in all 3 bounded ops (T-02-13 mitigation)
- STO-arith atomicity: compute new_val first, write only on success (T-02-14 mitigation, Pitfall 6)
- RCL sets lift_enabled=true BEFORE enter_number — HP-41 hardware behavior
- format_hpnum() with FIX (trailing zeros + overflow fallback), SCI (HP-41 space/minus exponent format), ENG (multiple-of-3 exponent)
- Removed all 13 `#[ignore]` attributes from format_tests.rs; all tests GREEN
- All 15 register_tests and 13 format_tests pass; math_tests (18) and stack_tests (18) still GREEN

## Task Commits

1. **Task 1: ops/registers.rs + dispatch wiring** — `26353a8` (feat)
2. **Task 2: format.rs + lib.rs + FmtFix/Sci/Eng dispatch** — `d7df723` (feat)

## Files Created/Modified

- `hp41-core/src/ops/registers.rs` — New: op_sto, op_rcl, op_sto_arith, op_clreg with bounds checks and correct lift effects
- `hp41-core/src/format.rs` — Replaced stub: format_hpnum (FIX/SCI/ENG), format_alpha, decimal_pow10 helper
- `hp41-core/src/ops/mod.rs` — Uncommented `pub mod registers;`, added imports, wired StoReg/RclReg/StoArith/Clreg/FmtFix/FmtSci/FmtEng dispatch arms
- `hp41-core/src/lib.rs` — Added `pub use format::{format_hpnum, format_alpha};`
- `hp41-core/tests/format_tests.rs` — Removed all 13 `#[ignore]` attributes; fixed test_fix4_overflow_to_sci to use parseable decimal notation

## Decisions Made

- **rust_decimal from_str rejects E notation**: `Decimal::from_str("1E-8")` returns `Err("Invalid decimal: unknown character")`. Used `decimal_pow10(exp)` helper that builds decimal strings like `"0.00000001"` (for negative exponents) and `"100000000"` (for positive). This is safe and accurate for all exponents used in format.rs.

- **FIX rounding requires explicit round_dp_with_strategy**: Rust's `format!("{:.4}", decimal)` uses Decimal's Display impl which applies Banker's rounding — `3.14159265359` formatted to 4 places gives `"3.1415"` instead of `"3.1416"`. Added explicit `d.round_dp_with_strategy(digits, MidpointAwayFromZero)` before format!, matching HP-41 hardware rounding behavior.

- **test_fix4_overflow_to_sci test fix (Rule 1 - Bug)**: The test used `num("1E15")` which calls `Decimal::from_str("1E15").expect(...)` — this panics because rust_decimal rejects E notation. Changed to `num("1000000000000000")` (same mathematical value, decimal notation). Test intent preserved — it checks `result.contains('E')` for overflow detection.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rust_decimal Decimal::from_str rejects E notation**
- **Found during:** Task 2 (format.rs implementation)
- **Issue:** Plan specified using `Decimal::from_str(&format!("1E{exp}"))` for scale factors in SCI/ENG formatting. rust_decimal 1.41 from_str rejects scientific notation like "1E-8", returning `Err("Invalid decimal: unknown character")`.
- **Fix:** Created `decimal_pow10(exp: i32) -> Decimal` helper that builds scale factors as decimal strings ("100000000" for 10^8, "0.00000001" for 10^-8). Applied same fix to the FIX overflow threshold calculation.
- **Files modified:** `hp41-core/src/format.rs`
- **Committed in:** `d7df723`

**2. [Rule 1 - Bug] Decimal Display uses Banker's rounding, not HP-41 rounding**
- **Found during:** Task 2 (format_tests verification)
- **Issue:** `format!("{:.4}", Decimal::from_str("3.14159265359").unwrap())` produces `"3.1415"` not `"3.1416"`. Decimal's Display trait uses MidpointNearestEven (Banker's rounding), which diverges from HP-41 hardware (MidpointAwayFromZero).
- **Fix:** Added `d.round_dp_with_strategy(digits as u32, RoundingStrategy::MidpointAwayFromZero)` before the format! call in `format_fix()` and before mantissa formatting in `format_sci()`/`format_eng()`.
- **Files modified:** `hp41-core/src/format.rs`
- **Committed in:** `d7df723`

**3. [Rule 1 - Bug] test_fix4_overflow_to_sci used unparseable E notation in test helper**
- **Found during:** Task 2 (format_tests run)
- **Issue:** The test called `num("1E15")` but `Decimal::from_str("1E15")` panics in the `num()` helper. This is a bug in the test infrastructure — the test can never reach its assertion.
- **Fix:** Changed `"1E15"` to `"1000000000000000"` (equivalent large number in decimal notation parseable by rust_decimal).
- **Files modified:** `hp41-core/tests/format_tests.rs`
- **Committed in:** `d7df723`

---

**Total deviations:** 3 auto-fixed (all Rule 1 - Bug)
**Impact on plan:** All fixes necessary for correctness. The E notation issue is a rust_decimal API constraint not documented in the plan's context. No scope creep.

## Threat Model Coverage

All mitigations from the plan's threat model are implemented:

| Threat ID | Mitigation | Location |
|-----------|-----------|---------|
| T-02-13 | `if reg >= 100 { return Err(HpError::InvalidOp) }` in op_sto, op_rcl, op_sto_arith | `registers.rs` |
| T-02-14 | Compute `new_val = ...?` first, write `state.regs[reg] = new_val` only after success | `registers.rs:op_sto_arith` |
| T-02-15 | Accepted — HpNum::rounded enforces 10-sig-digit cap; format_sci handles any valid HpNum | (no code change) |
| T-02-16 | Accepted — Phase 7 can add clamping at u8 max=9 | (no code change) |

## Known Stubs

None — all required implementations are complete. Alpha ops (AlphaToggle/AlphaAppend/AlphaClear) remain stubbed as `Err(HpError::InvalidOp)` in dispatch, but those are Plan 02-06's responsibility.

## Threat Flags

None — this plan adds no new network endpoints, auth paths, file access, or schema changes. All code is pure local arithmetic and string formatting in hp41-core.

---

## Self-Check: PASSED

Files verified:
- FOUND: hp41-core/src/ops/registers.rs
- FOUND: hp41-core/src/format.rs
- FOUND: hp41-core/src/lib.rs (modified)
- FOUND: hp41-core/src/ops/mod.rs (modified)
- FOUND: hp41-core/tests/format_tests.rs (modified)

Commits verified:
- FOUND: 26353a8 (Task 1 — storage register operations)
- FOUND: d7df723 (Task 2 — format.rs + dispatch wiring)

---
*Phase: 02-core-math*
*Completed: 2026-05-06*
