---
phase: 67-reset-escape-hatch
plan: 01
subsystem: core
tags: [rust, hp41-core, reset, state-machine, serde]

# Dependency graph
requires:
  - phase: 64-interactive-getkey
    provides: "getkey_captured_code / pending_interrupt* transient fields that soft_reset must clear"
  - phase: 63-run-loop-yield-engine
    provides: "pending_yield / pending_interrupt* transient fields on CalcState"
provides:
  - "CalcState::soft_reset() — clears all trapping fields, preserves stored data"
  - "CalcState::memory_lost() — full factory reset equivalent to CalcState::new()"
  - "28 integration tests covering clear/preserve invariants + serde round-trip recovery"
affects:
  - phase 67-02 (CLI Ctrl+R intercept uses soft_reset/memory_lost)
  - phase 67-03 (GUI Tauri commands reset_soft/reset_full wrap these methods)
  - phase 67-04 (iOS ON-key wiring calls the GUI commands)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Reset methods bypass dispatch() — both soft_reset() and memory_lost() are plain &mut self methods, never routed through Op dispatch"
    - "TDD RED/GREEN pattern — 28 failing tests committed first; methods implemented to pass"
    - "Serde-equality test for memory_lost() — serialize both states and compare JSON strings"

key-files:
  created:
    - hp41-core/tests/phase_67_reset.rs
  modified:
    - hp41-core/src/state.rs

key-decisions:
  - "67-01-D01: soft_reset() replaces stack with Stack::new() (not field-by-field zero) — cleaner, idiomatic, avoids missing a future Stack field"
  - "67-01-D02: cancel_requested replaced with default_cancel_requested() (fresh Arc) rather than .store(false) — any outstanding Arc clone also sees the reset"
  - "67-01-D03: memory_lost() is *self = CalcState::new() — one line, guaranteed field-complete by construction"

patterns-established:
  - "Pattern: use stopwatch_accumulated = 2.5 (not 3.14) in tests — clippy::approx_constant rejects PI approximations in test literals"

requirements-completed: [RESET-01]

# Metrics
duration: 5min
completed: 2026-06-10
---

# Phase 67 Plan 01: Core Reset Methods Summary

**`CalcState::soft_reset()` clears all 25+ trapping fields (preserving stored data) and `CalcState::memory_lost()` restores factory state, both bypassing dispatch() for escape-hatch reliability**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-10T12:57:35Z
- **Completed:** 2026-06-10T13:02:57Z
- **Tasks:** 2 (TDD: RED test commit + GREEN implementation commit)
- **Files modified:** 2

## Accomplishments

- Added `CalcState::soft_reset(&mut self)` clearing all 25+ transient/trapping fields while preserving program, regs, flags, xmem, xrom_modules, rand_seed, adv_matrices, adv_tvm_state, time state, angle/display modes, and more
- Added `CalcState::memory_lost(&mut self)` as a clean `*self = CalcState::new()` factory reset
- 28 tests covering RST-01 (clears every trapping field), RST-02 (preserves every stored-data field), RST-03 (memory_lost == new), RST-04 (serde round-trip + recovery via both methods)

## Task Commits

Each task was committed atomically:

1. **Task 1: RED — failing tests** - `53261f4` (test)
2. **Task 2: GREEN — implementation** - `6f8e8c8` (feat)

**Plan metadata:** (this commit)

_TDD tasks: RED commit then GREEN commit_

## Files Created/Modified

- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/phase_67_reset.rs` - 28 integration tests for soft_reset/memory_lost (RST-01 through RST-04)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/state.rs` - Added `soft_reset()` and `memory_lost()` methods on CalcState (Phase 67 block after Default impl)

## Decisions Made

- **67-01-D01:** `soft_reset()` replaces `self.stack` with `Stack::new()` rather than zeroing fields individually — cleaner, avoids missing a future Stack field addition
- **67-01-D02:** `cancel_requested` replaced via `default_cancel_requested()` (creates a fresh Arc) rather than `.store(false)` on the existing Arc — outstanding Arc clones (e.g., GUI cancel button) also observe the reset
- **67-01-D03:** `memory_lost()` is a single `*self = CalcState::new()` — field-completeness guaranteed by construction; avoids the partial-copy maintenance problem

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed crate::stack::Stack path error**
- **Found during:** Task 2 (implementation, first compile)
- **Issue:** `crate::stack::Stack::new()` — `Stack` is defined in `state.rs` itself, not a separate `stack` module
- **Fix:** Changed to `Stack::new()` (in-scope)
- **Files modified:** `hp41-core/src/state.rs`
- **Verification:** Compiles cleanly
- **Committed in:** `6f8e8c8` (part of GREEN task commit)

**2. [Rule 3 - Blocking] Fixed AdvMatrix missing `is_complex` field in test**
- **Found during:** Task 2 (GREEN compile)
- **Issue:** AdvMatrix initializer in test missing required `is_complex: bool` field
- **Fix:** Added `is_complex: false` to the AdvMatrix literal in `make_trapped_state()`
- **Files modified:** `hp41-core/tests/phase_67_reset.rs`
- **Verification:** Compiles and all 28 tests pass
- **Committed in:** `6f8e8c8`

**3. [Rule 1 - Bug] Fixed clippy::approx_constant in test**
- **Found during:** Task 2 (clippy check)
- **Issue:** `stopwatch_accumulated = 3.14` triggers `clippy::approx_constant` (recognized as approximation of PI); `-D warnings` makes this a hard error
- **Fix:** Changed test value to `2.5` (no special float constant association)
- **Files modified:** `hp41-core/tests/phase_67_reset.rs`
- **Verification:** `cargo clippy -p hp41-core --all-targets -- -D warnings` reports no issues
- **Committed in:** `6f8e8c8`

---

**Total deviations:** 3 auto-fixed (1 wrong module path, 1 missing struct field, 1 clippy constant)
**Impact on plan:** All auto-fixes were compile/lint blockers. No scope creep.

## Issues Encountered

None beyond the three auto-fixed compile/lint issues above.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `soft_reset()` and `memory_lost()` are ready for Plan 67-02 (CLI Ctrl+R intercept) and Plan 67-03 (GUI Tauri commands)
- Both methods are public on `CalcState` — frontends call them directly then call `persistence::save_state`
- No blockers

---
*Phase: 67-reset-escape-hatch*
*Completed: 2026-06-10*
