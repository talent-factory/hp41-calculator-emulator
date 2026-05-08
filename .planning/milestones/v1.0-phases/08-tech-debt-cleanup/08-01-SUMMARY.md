---
phase: 08-tech-debt-cleanup
plan: "01"
subsystem: core
tags: [rust-decimal, flush_entry_buf, eex, scientific-notation, tdd]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: flush_entry_buf function, CalcState, HpNum newtype
  - phase: 02-core-math
    provides: rust_decimal dependency with from_scientific API
provides:
  - flush_entry_buf parses scientific notation strings (e.g. "1.5e3", "2.5E-2")
  - EEX entry buffer values correctly flushed to stack without InvalidOp error
affects:
  - 08-02 (entry_buf guards in app.rs — depends on flush working correctly for EEX)
  - future plans using dispatch() with scientific notation input

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "from_str().or_else(from_scientific) chained parse — parse with primary path, fall through to scientific notation fallback only on failure"

key-files:
  created: []
  modified:
    - hp41-core/src/ops/mod.rs

key-decisions:
  - "Use or_else(|_| Decimal::from_scientific(&s)) fallback rather than pre-checking for 'e' in string — keeps parse logic minimal and correct; both parsers return Err on invalid input so InvalidOp mapping remains safe"

patterns-established:
  - "TDD RED/GREEN: add failing tests as separate commit before implementation commit"

requirements-completed:
  - INPUT-01

# Metrics
duration: 10min
completed: 2026-05-08
---

# Phase 8 Plan 01: EEX flush_entry_buf Fix Summary

**Decimal::from_scientific() fallback added to flush_entry_buf so EEX scientific notation strings like "1.5e3" parse to stack values instead of returning InvalidOp**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-08T00:00:00Z
- **Completed:** 2026-05-08T00:10:00Z
- **Tasks:** 1 (TDD: 2 commits — RED then GREEN)
- **Files modified:** 1

## Accomplishments

- Scientific notation entry buffer strings ("1.5e3", "2.5E-2") now correctly flush to Decimal stack values
- Added `flush_eex_tests` module with 4 tests covering lowercase e, uppercase E, plain decimal, and invalid strings
- Full test suite remains at zero failures after fix

## Task Commits

TDD task with RED/GREEN commit split:

1. **RED — failing tests** - `7b6a6af` (test)
2. **GREEN — from_scientific fallback** - `0de607c` (fix)

## Files Created/Modified

- `hp41-core/src/ops/mod.rs` — Added `.or_else(|_| Decimal::from_scientific(&s))` fallback on line 213-215; added `flush_eex_tests` module with 4 tests

## Decisions Made

- `or_else` chaining chosen over pre-scanning `entry_buf` for 'e': simpler, correct, no false positives. Both parsers return `Err` on malformed input so the final `.map_err(|_| HpError::InvalidOp)?` still handles all error cases correctly.
- Tests access `state.stack.x.0` directly (the `pub(crate)` Decimal field on HpNum) since tests live inside the crate — no new public API needed.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- EEX core fix is complete; Plan 02 (entry_buf guards in hp41-cli/src/app.rs) can now proceed knowing flush_entry_buf handles scientific notation correctly
- No blockers

---
*Phase: 08-tech-debt-cleanup*
*Completed: 2026-05-08*
