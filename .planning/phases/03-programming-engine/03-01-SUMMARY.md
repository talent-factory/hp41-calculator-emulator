---
phase: 03-programming-engine
plan: "01"
subsystem: core
tags: [rust, state, calcstate, programming-engine, vec, bool, usize]

# Dependency graph
requires:
  - phase: 02-core-math
    provides: CalcState, HpNum, Op enum, dispatch() entry point, flush_entry_buf
provides:
  - CalcState with five Phase 3 fields (program, prgm_mode, pc, call_stack, is_running)
  - Zero-initialised programming engine state in CalcState::new()
affects:
  - 03-02: PRGM mode recording gate in dispatch() reads prgm_mode and writes to program
  - 03-03: Op variants already added by parallel agent; TestKind enum in ops/mod.rs
  - 03-04: run_program() reads/writes pc, call_stack, is_running
  - 03-05: ISG/DSE reads registers; does not directly use Phase 3 state fields added here

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Vec<crate::ops::Op> full path avoids circular dependency (state.rs must not import ops)"
    - "TDD RED/GREEN cycle: failing tests committed before implementation"

key-files:
  created: []
  modified:
    - hp41-core/src/state.rs
    - hp41-core/src/tests.rs

key-decisions:
  - "Used Vec<crate::ops::Op> full path in state.rs — adding 'use crate::ops::Op' would create state↔ops circular dependency"
  - "call_stack uses Vec<usize> not a fixed-size array — enforcement of max-4 depth deferred to run_program() in plan 03-04"

patterns-established:
  - "Phase 3 fields added after entry_buf as a clearly labelled block with D-number references"
  - "TDD: RED commit (test(03-01)) precedes GREEN commit (feat(03-01)) for traceability"

requirements-completed:
  - PROG-01

# Metrics
duration: 8min
completed: 2026-05-07
---

# Phase 3 Plan 01: CalcState Programming Engine Fields Summary

**Five Phase 3 fields added to CalcState — program Vec, prgm_mode flag, pc counter, call_stack, and is_running guard — all zero-initialised in new()**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-07T08:00:00Z
- **Completed:** 2026-05-07T08:08:17Z
- **Tasks:** 1 (TDD: RED + GREEN)
- **Files modified:** 2

## Accomplishments

- Added `pub program: Vec<crate::ops::Op>` to CalcState (D-01: flat program memory)
- Added `pub prgm_mode: bool` to CalcState (D-03: PRGM recording gate)
- Added `pub pc: usize` to CalcState (D-05: program counter, 0 at startup)
- Added `pub call_stack: Vec<usize>` to CalcState (D-14: return stack, enforcement in plan 03-04)
- Added `pub is_running: bool` to CalcState (D-06: re-entrancy guard)
- All five initialised to zero/false/empty in `CalcState::new()`
- 245 hp41-core tests pass; cargo check exits 0

## Task Commits

1. **RED — failing tests for Phase 3 CalcState fields** - `a65ad99` (test)
2. **GREEN — add 5 Phase 3 fields to CalcState** - `8f47750` (feat)

_TDD cycle: test commit precedes implementation commit_

## Files Created/Modified

- `hp41-core/src/state.rs` — 5 Phase 3 fields + 5 initialisers in `CalcState::new()`
- `hp41-core/src/tests.rs` — `phase3_state_tests` module with 5 RED→GREEN tests

## Decisions Made

- `Vec<crate::ops::Op>` full path used for the `program` field type. Adding `use crate::ops::Op` at the top of `state.rs` would create a state↔ops circular dependency (ops/mod.rs already imports CalcState from state.rs). Rust resolves the full crate path cleanly at link time without creating the cycle.
- `call_stack: Vec<usize>` rather than a fixed-size array. The 4-entry maximum is an invariant enforced at runtime by `run_program()` (plan 03-04), not at the type level. Using Vec simplifies the struct and keeps enforcement co-located with the logic.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None. The worktree's isolated working directory contained the clean Phase 2 baseline at HEAD `914a516`. The `Vec<crate::ops::Op>` full path resolved without issue because Rust does not require an explicit `use` for types referenced via their full crate path in a struct field.

## Known Stubs

None. This plan adds struct fields only; no UI rendering or data flow paths are introduced.

## Threat Flags

None. Pure in-memory struct extension with no I/O, no external input, no network surface. T-03-01-01 (heap allocation via `Vec::new()`) is accepted per plan threat model — bounded by HP-41 999-step limit enforced in later plans.

## Next Phase Readiness

- Phase 3 Plan 02 (PRGM mode gate in dispatch()) can read `state.prgm_mode` and write to `state.program` — both fields are present
- Phase 3 Plan 03 (Op variants / TestKind) may already be partially complete from parallel agent work; executor should check for existing `TestKind` before adding it
- Phase 3 Plan 04 (run_program()) can read/write `state.pc`, `state.call_stack`, `state.is_running`

## Self-Check

- [x] `hp41-core/src/state.rs` modified: `pub program: Vec<crate::ops::Op>` present
- [x] `hp41-core/src/state.rs` modified: `pub prgm_mode: bool` present
- [x] `hp41-core/src/state.rs` modified: `pub pc: usize` present
- [x] `hp41-core/src/state.rs` modified: `pub call_stack: Vec<usize>` present
- [x] `hp41-core/src/state.rs` modified: `pub is_running: bool` present
- [x] RED commit exists: `a65ad99`
- [x] GREEN commit exists: `8f47750`
- [x] `cargo check -p hp41-core` exits 0
- [x] 245 tests pass

## Self-Check: PASSED

---
*Phase: 03-programming-engine*
*Completed: 2026-05-07*
