---
phase: 11-print-emulation
plan: "01"
subsystem: core-ops
tags: [rust, hp41-core, print, buffer, ops, serde]

# Dependency graph
requires:
  - phase: 11-00
    provides: RED test suite in hp41-core/tests/print_tests.rs defining all 18 required behaviors
provides:
  - print_buffer: Vec<String> field on CalcState with serde(default) for v1.0 JSON backward compat
  - op_prx: PRX op buffers right-aligned 24-char X register string into print_buffer
  - op_pra: PRA op buffers left-aligned 24-char ALPHA register string into print_buffer
  - op_prstk: PRSTK op buffers 6 lines (T/Z/Y/X/LASTX/ALPHA) of 24 chars each into print_buffer
  - Op::PRX / Op::PRA / Op::PRSTK variants in Op enum with dispatch() and execute_op() arms
affects:
  - 11-02-plan (CLI drain of print_buffer after dispatch)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Print ops are I/O-free: all output goes to CalcState.print_buffer; CLI drains after dispatch"
    - "LiftEffect::Neutral for all print ops — stack is read-only, never modified"
    - "serde(default) on transient fields ensures v1.0 JSON backward compatibility"

key-files:
  created:
    - hp41-core/src/ops/print.rs
  modified:
    - hp41-core/src/state.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs

key-decisions:
  - "print_buffer is Vec<String> on CalcState, not a separate type — simplest shape for CLI drain"
  - "serde(default) on print_buffer is mandatory for v1.0 JSON save file backward compat"
  - "PRX uses format_hpnum + {:>24} right-align; PRA uses direct alpha_reg take(24) + {:<24}"
  - "PRSTK uses 7-char label field + 17-char value field = 24-char invariant for all 6 lines"
  - "prgm_display.rs needed Rule 2 arms for PRX/PRA/PRSTK to fix non-exhaustive match in hp41-cli"

patterns-established:
  - "Print buffer pattern: core ops push to state.print_buffer; UI layer drains after dispatch"
  - "execute_op() arms required alongside dispatch() arms for ops to work inside programs"

requirements-completed: [PRNT-01, PRNT-02, PRNT-03]

# Metrics
duration: 4min
completed: 2026-05-08
---

# Phase 11 Plan 01: Print Emulation Core Summary

**PRX/PRA/PRSTK ops with CalcState.print_buffer using format_hpnum — all 18 RED tests GREEN, zero I/O in hp41-core**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-08T18:24:20Z
- **Completed:** 2026-05-08T18:28:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `print_buffer: Vec<String>` to CalcState with `#[serde(default)]` for v1.0 JSON compatibility
- Created `hp41-core/src/ops/print.rs` with op_prx, op_pra, op_prstk — all LiftEffect::Neutral, zero I/O
- Added Op::PRX / Op::PRA / Op::PRSTK to Op enum, dispatch() in mod.rs, and execute_op() in program.rs
- All 18 print_tests GREEN; full workspace 490 tests pass without regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Add print_buffer to CalcState (state.rs)** - `3331dbd` (feat)
2. **Task 2: Create ops/print.rs module and register in mod.rs + program.rs** - `90d729a` (feat)

## Files Created/Modified

- `hp41-core/src/state.rs` - Added print_buffer: Vec<String> with #[serde(default)] and Vec::new() initializer
- `hp41-core/src/ops/print.rs` - New module: op_prx, op_pra, op_prstk implementations
- `hp41-core/src/ops/mod.rs` - Added pub mod print; Op::PRX/PRA/PRSTK enum variants and dispatch() arms
- `hp41-core/src/ops/program.rs` - Added execute_op() arms for PRX/PRA/PRSTK before catch-all
- `hp41-cli/src/prgm_display.rs` - Added display string arms for PRX/PRA/PRSTK (Rule 2 fix)

## Decisions Made

- PRX right-aligns via `format!("{:>24}", format_hpnum(...))` — matches HP-41 printer right-justified numeric output
- PRA takes alpha_reg chars directly (not via format_alpha which truncates to 12) for full 24-char print width
- PRSTK uses `{:<7}{:>17}` for numeric lines and `{:<7}{:<17}` for ALPHA line (left-aligned content)
- print_buffer is cleared by the CLI caller, not by the ops — ops only push, never drain

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added prgm_display.rs arms for PRX/PRA/PRSTK**
- **Found during:** Task 2 (workspace test after implementing print ops)
- **Issue:** hp41-cli/src/prgm_display.rs has a non-exhaustive match on Op that did not cover the new PRX/PRA/PRSTK variants, causing a compilation error across the workspace
- **Fix:** Added `Op::PRX => "PRX".to_string()`, `Op::PRA => "PRA".to_string()`, `Op::PRSTK => "PRSTK".to_string()` arms to the `op_to_display_string()` function
- **Files modified:** hp41-cli/src/prgm_display.rs
- **Verification:** `cargo test --workspace` exits 0 with 490 tests passing
- **Committed in:** 90d729a (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical — non-exhaustive match in hp41-cli)
**Impact on plan:** Essential fix; new Op variants always require coverage in prgm_display.rs. No scope creep.

## Issues Encountered

None - implementation followed plan exactly. The prgm_display.rs fix is a predictable consequence of adding new Op variants to a codebase with exhaustive match patterns.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 11-02 (CLI drain) can proceed: print_buffer is ready on CalcState, all three ops push to it correctly
- The drain pattern for hp41-cli/src/app.rs: call `state.print_buffer.drain(..)` after each dispatch() call and route lines to the TUI print panel

---
*Phase: 11-print-emulation*
*Completed: 2026-05-08*
