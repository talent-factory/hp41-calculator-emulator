---
phase: 02-core-math
plan: 02
subsystem: core-model
tags: [rust, hp41-core, hpnum, math, trig, rust_decimal, maths-feature, tdd, f64-bridge]

# Dependency graph
requires:
  - 02-01 (Cargo.toml with maths feature, CalcState skeleton)
  - 01-02 (HpNum newtype, checked_add/sub/mul/div, HpNum::rounded)
provides:
  - HpNum::checked_recip — 1/x, DivideByZero on zero
  - HpNum::checked_sqrt — √x, Domain on x<0 (uses MathematicalOps::sqrt)
  - HpNum::checked_sq — x², via checked_mul (no maths feature)
  - HpNum::checked_ln — ln(x), Domain on x≤0
  - HpNum::checked_log10 — log₁₀(x), Domain on x≤0
  - HpNum::checked_exp — e^x via MathematicalOps::checked_exp
  - HpNum::checked_exp10 — 10^x via Decimal::from(10).checked_powd(self.0)
  - HpNum::checked_powd — y^x, Domain on negative base with fractional exponent
  - HpNum::checked_sin/cos/tan — radians, decimal-native via MathematicalOps
  - HpNum::checked_asin/acos/atan — f64 round-trip bridge; asin/acos domain guarded
affects: [02-03 math.rs — all 14 op functions call these methods directly]

# Tech tracking
tech-stack:
  added:
    - rust_decimal::MathematicalOps (already feature-flagged in 02-01 Cargo.toml)
    - rust_decimal::prelude::ToPrimitive (.to_f64() for f64 bridge)
    - rust_decimal::prelude::FromPrimitive (Decimal::from_f64() for f64 bridge)
  patterns:
    - Domain guards BEFORE calling checked_* to distinguish Domain vs Overflow (ln/sqrt/powd/asin/acos)
    - f64 round-trip bridge for inverse trig (asin/acos/atan): Decimal→f64→libstd→HpNum::rounded()
    - sqrt uses MathematicalOps::sqrt() returning Option<Decimal> (no checked_sqrt in API)
    - All results pass through HpNum::rounded() — enforces 10-sig-digit HP-41 precision

key-files:
  created: []
  modified:
    - hp41-core/src/num.rs
    - hp41-core/src/tests.rs

key-decisions:
  - "MathematicalOps::sqrt() returns Option<Decimal> — not a checked_ prefix method; used directly with domain guard before it"
  - "All 14 methods implemented in the same impl HpNum block together to avoid two separate feature commits"
  - "TDD: scalar math RED test commit separate from GREEN; trig tests added after GREEN (Task 2 executed together with Task 1 in GREEN)"

# Metrics
duration: 3min
completed: 2026-05-06
---

# Phase 2 Plan 02: HpNum Math Methods Summary

**14 HpNum checked math methods added to num.rs: scalar math (recip/sqrt/sq/ln/log10/exp/exp10/powd) and trig (sin/cos/tan via MathematicalOps; asin/acos/atan via f64 round-trip bridge), all enforcing 10-digit HP-41 precision and explicit domain guards**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-06T17:05:53Z
- **Completed:** 2026-05-06T17:08:51Z
- **Tasks:** 2 (Task 1 scalar math, Task 2 trig)
- **Files modified:** 2

## Accomplishments

- 14 new HpNum methods covering the full HP-41 math op set
- All methods use only checked_* variants — zero panics possible
- Domain errors explicitly guarded before checked_* calls (Domain ≠ Overflow distinction preserved)
- f64 round-trip bridge for asin/acos/atan with domain guards and inline comment documenting why
- `cargo check -p hp41-core` exits 0
- `cargo test -p hp41-core` — 159 tests pass (0 FAILED)
- 18 total `checked_*` methods in HpNum impl (4 from Phase 1 + 14 new)
- 10 explicit `HpError::Domain` occurrences (all mitigations from threat model implemented)

## Task Commits

1. **RED: Failing tests for scalar math methods** — `8ac077e` (test)
2. **GREEN: All 14 HpNum math + trig methods** — `2d4d38d` (feat)
3. **Trig tests for Task 2** — `9258c4a` (test)

## Files Created/Modified

- `hp41-core/src/num.rs` — Added imports (MathematicalOps, ToPrimitive, FromPrimitive) + 14 new methods in impl HpNum block; scalar math before negate(), trig after powd()
- `hp41-core/src/tests.rs` — Added `num_scalar_math_tests` (37 tests) and `num_trig_math_tests` (20 tests) modules

## Decisions Made

- `MathematicalOps::sqrt()` (not `checked_sqrt`) — rust_decimal 1.41 exposes sqrt() returning `Option<Decimal>`, which is the "checked" variant. Domain guard for x<0 is applied explicitly before calling it.
- Task 1 and Task 2 methods were implemented together in a single GREEN commit for efficiency. The TDD RED test commit for scalar math was separate; trig tests were added post-GREEN as Task 2 verification.
- `checked_exp10` uses `Decimal::from(10).checked_powd(self.0)` rather than a hypothetical `checked_exp10` method that does not exist in rust_decimal 1.41.

## TDD Gate Compliance

- RED gate (Task 1): `test(02-02)` commit `8ac077e` — scalar math tests written before implementation, confirmed failing (compile errors)
- GREEN gate (Tasks 1+2): `feat(02-02)` commit `2d4d38d` — all 14 methods implemented, 159 tests pass
- Task 2 trig tests added as separate commit `9258c4a` — methods already existed; tests serve as documentation and regression anchors
- Note: Task 2 did not have a separate RED commit before GREEN because both tasks were implemented together. The scalar math RED commit `8ac077e` covers the TDD gate; trig tests validate the same feat commit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `checked_sqrt` not available on `Decimal`**
- **Found during:** Task 1 GREEN implementation
- **Issue:** Plan specified `self.0.checked_sqrt()` but rust_decimal 1.41 MathematicalOps does not expose a `checked_sqrt` method. The method is named `sqrt()` and returns `Option<Decimal>` (which is the checked behavior).
- **Fix:** Used `self.0.sqrt()` instead with explicit comment "rust_decimal MathematicalOps provides sqrt() returning Option<Decimal>". The Option semantics are identical to `checked_sqrt`.
- **Files modified:** `hp41-core/src/num.rs`
- **Commit:** `2d4d38d`

## Threat Model Coverage

All mitigations from the plan's threat model are implemented:

| Threat ID | Mitigation | Location |
|-----------|-----------|---------|
| T-02-04 | `if self.0 <= Decimal::ZERO { return Err(HpError::Domain); }` in checked_ln and checked_sqrt | `num.rs:checked_ln`, `num.rs:checked_sqrt` |
| T-02-05 | rust_decimal checked_sin/cos/tan returns None for huge angles → mapped to Domain | `num.rs:checked_sin/cos/tan` |
| T-02-06 | `(-1.0..=1.0).contains(&v)` guard before v.asin()/v.acos() | `num.rs:checked_asin`, `num.rs:checked_acos` |
| T-02-07 | `self.0.is_sign_negative() && !exp.0.fract().is_zero()` guard in checked_powd | `num.rs:checked_powd` |

## Known Stubs

None — all 14 methods are fully implemented with domain guards and 10-digit rounding.

## Threat Flags

None — this plan adds no new network endpoints, auth paths, file access, or schema changes. All code is pure local arithmetic in hp41-core.

---

## Self-Check: PASSED

Files verified:
- FOUND: hp41-core/src/num.rs
- FOUND: hp41-core/src/tests.rs

Commits verified:
- FOUND: 8ac077e (test — RED phase)
- FOUND: 2d4d38d (feat — GREEN phase)
- FOUND: 9258c4a (test — Task 2 trig tests)

---
*Phase: 02-core-math*
*Completed: 2026-05-06*
