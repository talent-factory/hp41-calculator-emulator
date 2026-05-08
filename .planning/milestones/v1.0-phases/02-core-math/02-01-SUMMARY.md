---
phase: 02-core-math
plan: 01
subsystem: core
tags: [rust, rust_decimal, maths, calcstate, op-enum, tdd]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: HpNum newtype, Stack, CalcState, Op enum, dispatch(), HpError, LiftEffect

provides:
  - rust_decimal maths feature enabled in hp41-core
  - AngleMode enum (Deg/Rad/Grad) in state.rs
  - DisplayMode enum (Fix/Sci/Eng) in state.rs
  - CalcState with all 6 Phase 2 fields: regs [HpNum;100], alpha_reg, alpha_mode, angle_mode, display_mode, entry_buf
  - HpNum Default impl returning zero
  - unary_result() helper in stack.rs
  - StoArithKind enum (Add/Sub/Mul/Div) in ops/mod.rs
  - Op enum with all 27 Phase 2 variants as stubs
  - dispatch() with exhaustive match arms for all new variants (stubs return InvalidOp)

affects:
  - 02-02 (HpNum math methods — depends on maths feature and CalcState.angle_mode)
  - 02-03 (math ops — depends on Op variants and unary_result)
  - 02-04 (display formatting — depends on DisplayMode)
  - 02-05 (storage registers — depends on regs, StoArithKind, StoReg/RclReg/StoArith/Clreg variants)
  - 02-06 (alpha mode — depends on alpha_reg, alpha_mode, AlphaAppend/AlphaClear/AlphaToggle variants)
  - 02-07 (entry buf — depends on entry_buf)

# Tech tracking
tech-stack:
  added:
    - rust_decimal maths feature (ln, exp, sqrt, sin, cos, tan, pow)
  patterns:
    - unary_result() pattern: saves LASTX, sets X, enables lift, Y/Z/T unchanged
    - StoArithKind enum for STO arithmetic operation type
    - Stub dispatch arms returning InvalidOp until real implementation in downstream plans

key-files:
  created:
    - hp41-core/tests/phase2_scaffold_tests.rs
  modified:
    - hp41-core/Cargo.toml
    - hp41-core/src/state.rs
    - hp41-core/src/num.rs
    - hp41-core/src/lib.rs
    - hp41-core/src/stack.rs
    - hp41-core/src/ops/mod.rs

key-decisions:
  - "rust_decimal maths feature enabled for ln/exp/pow/trig per ADR-001 locked decision"
  - "CalcState gains 6 new Phase 2 fields initialized to HP-41 hardware cold-start defaults"
  - "unary_result() helper follows binary_result() pattern but does not drop Y/Z/T"
  - "StoArithKind defined in ops/mod.rs (not registers.rs) to avoid forward-dependency before Plan 05"
  - "Phase 2 module declarations (math/registers/alpha) commented out until their files exist"

patterns-established:
  - "Pattern: unary_result(state, result) — saves LASTX before overwriting X, preserves Y/Z/T"
  - "Pattern: StoArithKind enum in ops/mod.rs for typed STO arithmetic dispatch"
  - "Pattern: stub dispatch arms returning InvalidOp mark future implementation targets"

requirements-completed: [MATH-01, MATH-02, MATH-03, REGS-01, ALPH-01]

# Metrics
duration: 3min
completed: 2026-05-06
---

# Phase 2 Plan 01: Structural Scaffolding Summary

**rust_decimal maths feature enabled, CalcState expanded with 6 Phase 2 fields, unary_result() helper added, Op enum gains all 27 Phase 2 variants as stubs — all downstream Phase 2 plans can now target real types**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-06T16:58:46Z
- **Completed:** 2026-05-06T17:01:46Z
- **Tasks:** 2 (executed together as 1 TDD cycle)
- **Files modified:** 7 (1 created, 6 modified)

## Accomplishments

- CalcState now carries all 6 Phase 2 fields: `regs [HpNum; 100]`, `alpha_reg`, `alpha_mode`, `angle_mode`, `display_mode`, `entry_buf` — all initialized to HP-41 hardware cold-start defaults
- `AngleMode` (Deg/Rad/Grad) and `DisplayMode` (Fix/Sci/Eng) enums added to `state.rs` and re-exported from `lib.rs`
- `unary_result()` helper in `stack.rs` — the canonical pattern for all unary ops: saves LASTX, sets X, enables lift, leaves Y/Z/T untouched
- `Op` enum extended from 11 to 38 variants covering all Phase 2 operations; `StoArithKind` enum defines STO arithmetic types
- `dispatch()` now has exhaustive match arms for all 38 variants — compiler enforces coverage
- All 117 tests pass (77 Phase 1 + 40 Phase 2 scaffold tests)

## Task Commits

TDD cycle:

1. **RED — failing scaffold tests** - `1c36b80` (test)
2. **GREEN — structural scaffolding implementation** - `fe1980d` (feat)

## Files Created/Modified

- `hp41-core/Cargo.toml` — added `features = ["maths"]` to rust_decimal dependency
- `hp41-core/src/state.rs` — added AngleMode, DisplayMode enums; expanded CalcState with 6 Phase 2 fields
- `hp41-core/src/num.rs` — added `impl Default for HpNum` returning zero
- `hp41-core/src/lib.rs` — added AngleMode, DisplayMode to public re-exports
- `hp41-core/src/stack.rs` — added `unary_result()` function
- `hp41-core/src/ops/mod.rs` — replaced with StoArithKind enum, 27 new Op variants, exhaustive dispatch() stubs
- `hp41-core/tests/phase2_scaffold_tests.rs` — 40 RED phase tests covering all new types and behaviors

## Decisions Made

- `StoArithKind` defined directly in `ops/mod.rs` rather than a new `ops/registers.rs` — avoids forward-dependency before Plan 05 creates that file
- Phase 2 module declarations (`pub mod math; pub mod registers; pub mod alpha;`) left as comments — uncommenting them before the files exist would cause compile errors
- `std::array::from_fn(|_| HpNum::zero())` used for `[HpNum; 100]` initialization — does not require `HpNum: Copy` (Decimal may or may not implement Copy; `from_fn` is always safe)

## Deviations from Plan

None — plan executed exactly as written.

## Threat Model Compliance

All three STRIDE threats from the plan correctly handled:
- **T-02-01** (StoReg bounds check): Stub returns `InvalidOp`; real bounds check `if reg >= 100` added in Plan 05
- **T-02-02** (array init DoS): `std::array::from_fn(|_| HpNum::zero())` is infallible — no panic path
- **T-02-03** (AlphaAppend unbounded): Stub returns `InvalidOp`; 24-char cap enforced in Plan 06

## Known Stubs

The following are intentional stubs per plan design, tracked for completion in downstream plans:

| Stub | File | Downstream Plan |
|------|------|-----------------|
| `Op::Recip`, `Op::Sqrt`, `Op::Sq`, `Op::YPow`, `Op::Ln`, `Op::Log`, `Op::Exp`, `Op::TenPow` | ops/mod.rs | Plan 03 (math ops) |
| `Op::Sin`, `Op::Cos`, `Op::Tan`, `Op::Asin`, `Op::Acos`, `Op::Atan` | ops/mod.rs | Plan 03 (trig ops) |
| `Op::SetDeg`, `Op::SetRad`, `Op::SetGrad` | ops/mod.rs | Plan 03 (angle mode) |
| `Op::FmtFix`, `Op::FmtSci`, `Op::FmtEng` | ops/mod.rs | Plan 04 (display formatting) |
| `Op::StoReg`, `Op::RclReg`, `Op::StoArith`, `Op::Clreg` | ops/mod.rs | Plan 05 (storage registers) |
| `Op::AlphaToggle`, `Op::AlphaAppend`, `Op::AlphaClear` | ops/mod.rs | Plan 06 (alpha mode) |

All stubs return `Err(HpError::InvalidOp)` — safe, predictable behavior until replaced.

## TDD Gate Compliance

- RED gate: `test(02-01)` commit `1c36b80` — 40 failing tests confirmed compile errors before implementation
- GREEN gate: `feat(02-01)` commit `fe1980d` — all 117 tests pass after implementation

## Issues Encountered

None.

## Next Phase Readiness

All downstream Phase 2 plans (02-02 through 02-07) can now:
- Use `AngleMode` and `DisplayMode` in dispatch and implementations
- Access `CalcState.regs`, `CalcState.alpha_reg`, etc.
- Call `unary_result()` for unary math implementations
- Reference `StoArithKind` variants for register arithmetic
- Add `use` statements against the real types without forward-reference errors

No blockers for parallel execution of wave 2 plans.

---
*Phase: 02-core-math*
*Completed: 2026-05-06*
