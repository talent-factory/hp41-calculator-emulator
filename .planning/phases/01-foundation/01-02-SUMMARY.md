---
phase: 01-foundation
plan: 02
subsystem: core-model
tags: [rust, hp41-core, hpnum, hperror, calcstate, stack, liftefect, rust_decimal, thiserror, tdd]

# Dependency graph
requires:
  - 01-01 (Cargo workspace scaffold with stubs in error.rs, num.rs, state.rs, stack.rs)
provides:
  - HpError enum (4 variants, PartialEq + Clone, thiserror derive)
  - HpNum newtype (rust_decimal, 10-digit MidpointAwayFromZero rounding, checked_* arithmetic)
  - CalcState and Stack structs (5 HpNum fields + lift_enabled: bool)
  - LiftEffect enum (Enable/Disable/Neutral) with apply_lift_effect, enter_number, binary_result helpers
  - ADR-001 comment in state.rs documenting rust_decimal numeric representation decision
  - pub use re-exports in lib.rs for all four types
affects: [03-ops, 04-tests, all subsequent phases — these are the contracts every operation implements against]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - TDD RED/GREEN cycle for core type module implementation
    - HpNum newtype enforces 10-sig-digit MidpointAwayFromZero rounding at all arithmetic boundaries
    - binary_result saves lastx BEFORE overwriting X (ordering enforced by code structure, not convention)
    - LiftEffect as sole authority for lift_enabled mutation — never set lift_enabled directly outside apply_lift_effect or binary_result
    - ADR comment in state.rs locks the rust_decimal vs BCD decision with full rationale

key-files:
  created:
    - hp41-core/src/tests.rs
  modified:
    - hp41-core/src/error.rs
    - hp41-core/src/num.rs
    - hp41-core/src/state.rs
    - hp41-core/src/stack.rs
    - hp41-core/src/lib.rs

key-decisions:
  - "rust_decimal 1.41 with MidpointAwayFromZero rounding strategy (not Bankers) matches HP-41 hardware display rounding — documented in ADR-001 in state.rs"
  - "HpNum inner field is pub(crate) Decimal to allow test assertions via inner() without exposing raw Decimal to consumers"
  - "binary_result captures lastx as first line before any X overwrite — structural enforcement of the LASTX timing contract (T-01-05)"
  - "enter_number does NOT modify lift_enabled — callers are responsible for setting it after entry"

patterns-established:
  - "All HP-41 register arithmetic flows through HpNum checked_* methods — no raw Decimal operations in hp41-core"
  - "Tests live in hp41-core/src/tests.rs as an inline module included in lib.rs"

requirements-completed: [CORE-01, CORE-02]

# Metrics
duration: 2min
completed: 2026-05-06
---

# Phase 1 Plan 02: Core Type Modules Summary

**HpError/HpNum/CalcState/Stack/LiftEffect implemented with rust_decimal MidpointAwayFromZero rounding, LASTX-before-overwrite ordering, and ADR-001 rust_decimal decision documented in state.rs**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-05-06T14:48:19Z
- **Completed:** 2026-05-06T14:51:00Z
- **Tasks:** 1 (TDD: RED commit + GREEN commit)
- **Files modified:** 5

## Accomplishments

- All four stub files filled with production implementations
- `cargo check -p hp41-core` exits 0 with zero errors
- `cargo tree -p hp41-core` shows no ratatui, crossterm, clap, or tokio dependencies
- 21 unit tests pass across error_tests, num_tests, state_tests, and stack_ops_tests
- ADR-001 comment in state.rs documents rust_decimal decision with rationale and consequences
- Threat model mitigations T-01-03, T-01-04, T-01-05 implemented and structurally enforced

## Task Commits

1. **RED: Failing tests for all four type modules** — `e9406a4` (test)
2. **GREEN: Implement HpError, HpNum, CalcState/Stack, LiftEffect** — `fee2c02` (feat)

## Files Created/Modified

- `hp41-core/src/error.rs` — HpError enum: Overflow, DivideByZero, InvalidOp, Domain; thiserror derive; PartialEq + Clone
- `hp41-core/src/num.rs` — HpNum newtype over rust_decimal::Decimal; rounded() with MidpointAwayFromZero; checked_add/sub/mul/div; negate/inner/zero/is_zero; From<i32> and From<Decimal> impls; Display
- `hp41-core/src/state.rs` — ADR-001 comment; CalcState { stack: Stack }; Stack { x/y/z/t/lastx: HpNum, lift_enabled: bool }; Default impls for both
- `hp41-core/src/stack.rs` — LiftEffect { Enable, Disable, Neutral }; apply_lift_effect (sole authority); enter_number (lift/overwrite with T←Z,Z←Y,Y←X); binary_result (lastx before X, Y←Z, Z←T, T duplicated)
- `hp41-core/src/lib.rs` — Updated pub use re-exports: HpError, HpNum, CalcState, Stack, LiftEffect; added `mod tests`
- `hp41-core/src/tests.rs` — 21 unit tests covering all behaviors specified in plan

## Decisions Made

- Used `pub(crate) Decimal` inner field on HpNum so tests can call `.inner()` for assertions without exposing raw Decimal to external consumers
- Tests are in `src/tests.rs` as a module included from `lib.rs` rather than a separate `tests/` directory — keeps tests co-located with the types they test, consistent with plan 01 structure
- `binary_result` directly sets `lift_enabled = true` (not via `apply_lift_effect`) because binary_result owns the full state rotation contract; this is intentional and documented in the function

## TDD Gate Compliance

- RED gate: `test(01-02)` commit `e9406a4` — tests written before any implementation, confirmed failing (compile error)
- GREEN gate: `feat(01-02)` commit `fee2c02` — all 21 tests pass after implementation
- REFACTOR gate: Not needed — implementation matched plan spec exactly

## Deviations from Plan

None - plan executed exactly as written.

## Threat Model Coverage

All three mitigations from the plan's threat model are implemented:

| Threat ID | Mitigation | Location |
|-----------|-----------|---------|
| T-01-03 | Explicit `if rhs.0.is_zero() { return Err(HpError::DivideByZero); }` before rust_decimal division | `num.rs:checked_div` |
| T-01-04 | All four arithmetic methods use `.checked_*()` variants; None maps to `Err(HpError::Overflow)` | `num.rs:checked_add/sub/mul/div` |
| T-01-05 | `state.stack.lastx = state.stack.x.clone()` is first line of `binary_result` — before any X overwrite | `stack.rs:binary_result` |

## Known Stubs

None — all four modules are fully implemented.

## Threat Flags

None — this plan adds no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. All code is pure local arithmetic with no I/O.

---

## Self-Check: PASSED

Files verified:
- FOUND: hp41-core/src/error.rs
- FOUND: hp41-core/src/num.rs
- FOUND: hp41-core/src/state.rs
- FOUND: hp41-core/src/stack.rs
- FOUND: hp41-core/src/tests.rs
- FOUND: hp41-core/src/lib.rs

Commits verified:
- FOUND: e9406a4 (test — RED phase)
- FOUND: fee2c02 (feat — GREEN phase)

---
*Phase: 01-foundation*
*Completed: 2026-05-06*
