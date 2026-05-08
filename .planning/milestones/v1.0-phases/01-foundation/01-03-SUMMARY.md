---
phase: 01-foundation
plan: 03
subsystem: ops
tags: [rust, hp41-core, ops, arithmetic, stack-ops, dispatch, tdd, lift-effect]

# Dependency graph
requires:
  - 01-02 (HpError, HpNum, CalcState/Stack, LiftEffect, binary_result, enter_number, apply_lift_effect)
provides:
  - Op enum (Add/Sub/Mul/Div/Enter/Clx/Chs/Rdn/XySwap/Lastx/PushNum variants)
  - dispatch(state, op) single entry point for all Phase 1 operations
  - op_add/sub/mul/div arithmetic functions (use binary_result; Y op X → X)
  - op_enter/clx/chs/rdn/xy_swap/lastx stack operation functions
  - Correct HP-41 lift semantics per operation (Enable/Disable/Neutral)
affects: [04-tests, all subsequent phases that add operations]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - TDD RED/GREEN cycle for all op implementations
    - dispatch() as single-entry-point chokepoint for all calculator operations
    - Every op explicitly declares its LiftEffect (Enable/Disable/Neutral)
    - Arithmetic ops delegate stack bookkeeping entirely to binary_result
    - ENTER always lifts unconditionally regardless of lift_enabled (HP-41 Pitfall 4)

key-files:
  created:
    - hp41-core/src/ops/arithmetic.rs
    - hp41-core/src/ops/stack_ops.rs
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/tests.rs

key-decisions:
  - "dispatch() uses plain match over Op enum variants — no dyn Trait, no dynamic dispatch; enum is exhaustive, serializable, and fast"
  - "op_lastx sets lift_enabled = true before calling enter_number to ensure the stack always lifts when recalling LASTX (regardless of prior lift state)"
  - "All 22 new tests follow the same pattern as plan 02 tests: construct fresh CalcState per test, no shared mutable state"

patterns-established:
  - "All ops return Result<(), HpError> — zero panic! or unwrap() in ops/ directory"
  - "Arithmetic ops are thin wrappers: compute result via checked_* then call binary_result"
  - "Stack ops call apply_lift_effect() as last action to declare their lift semantic"

requirements-completed: [CORE-01, CORE-02]

# Metrics
duration: 8min
completed: 2026-05-06
---

# Phase 1 Plan 03: Op Enum, Dispatch, and Operations Summary

**Op enum with dispatch(), op_add/sub/mul/div via binary_result, and op_enter/clx/chs/rdn/xy_swap/lastx with correct HP-41 lift semantics — all 10 Phase 1 operations implemented, 43 tests pass**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-06
- **Completed:** 2026-05-06
- **Tasks:** 2 (TDD: RED commit + GREEN commit covering both tasks)
- **Files modified:** 4

## Accomplishments

- All 10 Phase 1 operations compile and are reachable via `Op` enum + `dispatch()`
- Lift effects match HP-41 specification: Add/Sub/Mul/Div/Lastx=Enable, Enter/Clx=Disable, Chs/Rdn/XySwap=Neutral
- `binary_result` called for all arithmetic ops — lastx capture and stack rotation happen correctly
- Zero `unwrap()` or `panic!` in ops/ directory
- `cargo check -p hp41-core` exits 0; `just build` compiles full workspace
- `cargo test -p hp41-core` exits 0 with 43 tests passing

## Task Commits

1. **RED: Failing tests for arithmetic ops, stack ops, and dispatch** — `e0d3e6f` (test)
2. **GREEN: Implement Op enum, dispatch, arithmetic ops, and stack ops** — `531b443` (feat)

## Files Created/Modified

- `hp41-core/src/ops/arithmetic.rs` — op_add/sub/mul/div; each checks operands via HpNum::checked_*, calls binary_result, returns Ok(())
- `hp41-core/src/ops/stack_ops.rs` — op_enter (unconditional lift, disable), op_clx (zero X, disable), op_chs (negate X, neutral), op_rdn (rotate Y→X/Z→Y/T→Z/X→T, neutral), op_xy_swap (exchange X/Y, neutral), op_lastx (push lastx into X with lift, enable)
- `hp41-core/src/ops/mod.rs` — Op enum (11 variants), dispatch() match expression routing to implementation functions
- `hp41-core/src/tests.rs` — 22 new tests across arithmetic_tests, dispatch_tests, stack_ops_dispatch_tests modules

## TDD Gate Compliance

- RED gate: `test(01-03)` commit `e0d3e6f` — 22 tests written before any ops implementation, confirmed failing (compile errors on missing modules)
- GREEN gate: `feat(01-03)` commit `531b443` — all 43 tests pass (21 from plan 02 preserved + 22 new)
- REFACTOR gate: Not needed — implementation matched plan spec exactly

## Decisions Made

- `op_lastx` forces `lift_enabled = true` before calling `enter_number` to guarantee the stack always lifts when recalling LASTX, then calls `apply_lift_effect(Enable)` to leave lift enabled for the next entry — this matches HP-41 hardware behavior
- Both tasks share one GREEN commit because `stack_ops.rs` requires the Op enum in `mod.rs` to satisfy the import in `tests.rs`; splitting would require a second intermediate failing state
- `PushNum(HpNum)` dispatch arm calls `enter_number` directly (not a separate function) because PushNum is not a named HP-41 key — it is the internal representation of digit entry from the UI layer

## Deviations from Plan

None — plan executed exactly as written. Both TDD phases (RED/GREEN) committed separately. All acceptance criteria verified.

## Threat Model Coverage

| Threat ID | Mitigation | Location |
|-----------|-----------|---------|
| T-01-06 | op_div delegates to `HpNum::checked_div` which has `if rhs.0.is_zero() { return Err(DivideByZero); }` before any rust_decimal call; dispatch propagates the error | `ops/arithmetic.rs:op_div` → `num.rs:checked_div` |
| T-01-07 | ENTER's unconditional lift (ignoring lift_enabled) is HP-41 specified behavior; documented in op_enter docstring and Pitfall 4 of RESEARCH.md | `ops/stack_ops.rs:op_enter` |
| T-01-08 | No new threat surface; Op::PushNum(HpNum) stores an already-rounded HpNum (10-digit cap enforced at HpNum creation); no allocation growth possible | `ops/mod.rs:dispatch` |

## Known Stubs

None — all 10 Phase 1 operations are fully implemented and tested.

## Threat Flags

None — this plan adds no network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. All code is pure local arithmetic with no I/O.

---

## Self-Check: PASSED

Files verified:
- FOUND: hp41-core/src/ops/arithmetic.rs
- FOUND: hp41-core/src/ops/stack_ops.rs
- FOUND: hp41-core/src/ops/mod.rs (updated from stub)
- FOUND: hp41-core/src/tests.rs (updated with 22 new tests)

Commits verified:
- FOUND: e0d3e6f (test — RED phase)
- FOUND: 531b443 (feat — GREEN phase)

---
*Phase: 01-foundation*
*Completed: 2026-05-06*
