---
phase: 01-foundation
plan: 04
subsystem: tests
tags: [rust, hp41-core, integration-tests, proptest, coverage, ci, CORE-01, CORE-02]

# Dependency graph
requires:
  - 01-03 (Op enum, dispatch, all 10 Phase 1 operations implemented)
  - 01-02 (HpNum, CalcState, Stack, LiftEffect, binary_result, enter_number)
provides:
  - hp41-core/tests/stack_tests.rs — 18 CORE-01 integration tests
  - hp41-core/tests/lift_tests.rs — 13 CORE-02 lift-effect integration tests
  - hp41-core/tests/proptest_stack.rs — 3 property tests (zero-panic + lift invariants)
  - just ci passes end-to-end (lint clean, 77 tests, 94.67% coverage)
affects: [all subsequent phases — Phase 1 acceptance gate cleared]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Integration tests in hp41-core/tests/ directory (Rust auto-discovery)
    - proptest strategy with prop_oneof! generating random Op sequences
    - Property tests verify zero-panic invariant and terminal lift state

key-files:
  created:
    - hp41-core/tests/stack_tests.rs
    - hp41-core/tests/lift_tests.rs
    - hp41-core/tests/proptest_stack.rs
  modified: []

key-decisions:
  - "Integration tests placed in hp41-core/tests/ (separate from src/tests.rs unit tests) for clean CORE-01/02 acceptance coverage"
  - "proptest excluded rust_decimal::Decimal import — HpNum::from(i32) is sufficient for random operand generation; no unused import warning"
  - "Coverage gate passes at 94.67% (far above 80% threshold) with no targeted gap-filling needed"

patterns-established:
  - "Integration tests use dispatch() as sole entry point — tests never call op_* functions directly"
  - "Each integration test constructs fresh CalcState — no shared mutable state between tests"

requirements-completed: [CORE-01, CORE-02]

# Metrics
duration: 10min
completed: 2026-05-06
---

# Phase 1 Plan 04: Test Suite and CI Gate Summary

**77-test acceptance suite covering CORE-01 stack behavior and CORE-02 lift semantics; just ci passes with 94.67% line coverage on hp41-core**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-06
- **Completed:** 2026-05-06
- **Tasks:** 2
- **Files modified:** 3 (all created)

## Accomplishments

- 18 CORE-01 integration tests in stack_tests.rs covering push/lift, ENTER, CLX, CHS, LASTX, arithmetic stack state, division-by-zero, RDN, XY-swap
- 13 CORE-02 lift-effect tests in lift_tests.rs covering Enable (Add/Sub/Mul/Div/Lastx), Disable (Enter/Clx), Neutral (Chs/Rdn/XySwap) classifications
- 3 proptest property tests verifying zero-panic invariant across sequences of 0–30 random ops, and terminal lift state after Add/Enter
- `just ci` passes end-to-end: lint clean with zero warnings, 77 tests all pass, 94.67% line coverage
- Phase 1 acceptance gate cleared: all 5 success criteria met

## Task Commits

1. **Task 1: CORE-01 unit tests (stack_tests + lift_tests)** — `caf0253` (test)
2. **Task 2: proptest suite and just ci verification** — `3c211f0` (test)

## Files Created/Modified

- `hp41-core/tests/stack_tests.rs` — 18 tests: push overwrite/lift, T-register duplication, ENTER semantics (twice), CLX, CHS neutral, LASTX capture/recall, arithmetic stack rotation, division error, RDN rotation, XY-swap
- `hp41-core/tests/lift_tests.rs` — 13 tests: all 5 Enable ops, 2 Disable ops, 6 Neutral tests (true/false × 3 ops)
- `hp41-core/tests/proptest_stack.rs` — 3 property tests using prop_oneof! strategy over all Phase 1 Op variants plus small-integer PushNum

## Coverage Results

```
Filename              Lines      Missed Lines     Cover
----------------------------------------------------------
num.rs                  50                 6    88.00%
ops/arithmetic.rs       20                 0   100.00%
ops/mod.rs              16                 0   100.00%
ops/stack_ops.rs        40                 0   100.00%
stack.rs                22                 0   100.00%
state.rs                21                 3    85.71%
----------------------------------------------------------
TOTAL                  169                 9    94.67%
```

The 9 missed lines are in `HpNum::checked_sub`/`checked_mul` (overflow paths, would require large Decimal values to trigger), and `Default` trait impls that forward to `new()` (trivially correct). All are non-critical paths.

## Deviations from Plan

None — plan executed exactly as written. No bugs found in the Phase 1 operations implementations. All 77 tests passed on first run.

## Verification Gate Results

1. `cargo check -p hp41-core` — exits 0, zero UI/CLI deps in tree
2. `cargo test -p hp41-core` — 77 tests pass (43 unit + 13 lift + 3 proptest + 18 stack)
3. `cargo llvm-cov --fail-under-lines 80 -p hp41-core` — exits 0 (94.67% >= 80%)
4. `just --list` — shows all six recipes: build, ci, coverage, default, lint, run, test
5. `just ci` — passes end-to-end on macOS
6. `grep -c "ADR-001" hp41-core/src/state.rs` — returns 1

## Phase 1 Success Criteria — All Met

1. User can push values onto the 4-level stack and LASTX captures correct value — verified by `test_lastx_captures_x_before_add`
2. ENTER, arithmetic result, CLX, CHS each produce correct stack-lift behavior — verified by lift_tests.rs
3. `cargo check -p hp41-core` passes with zero UI/CLI dependencies
4. BCD/f64 decision committed as ADR-001 in state.rs (rust_decimal 1.41 + HpNum newtype)
5. `just --list` shows all six recipes; `just ci` passes on macOS

## Known Stubs

None — all test files are complete and wire to real implementations.

## Threat Flags

None — this plan adds only test files. No new network endpoints, auth paths, file access, or schema changes.

---

## Self-Check: PASSED

Files verified:
- FOUND: hp41-core/tests/stack_tests.rs
- FOUND: hp41-core/tests/lift_tests.rs
- FOUND: hp41-core/tests/proptest_stack.rs

Commits verified:
- FOUND: caf0253 (test — Task 1, CORE-01/02 integration tests)
- FOUND: 3c211f0 (test — Task 2, proptest suite + just ci verification)

---
*Phase: 01-foundation*
*Completed: 2026-05-06*
