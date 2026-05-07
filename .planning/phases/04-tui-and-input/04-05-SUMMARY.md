---
phase: 04-tui-and-input
plan: 05
subsystem: testing
tags: [clippy, ci, coverage, cargo-llvm-cov, rust, quality-gate]

# Dependency graph
requires:
  - phase: 04-tui-and-input
    plan: 04
    provides: Complete main.rs entry point; end-to-end TUI smoke-tested

provides:
  - Green just ci gate across full workspace (lint + tests + coverage)
  - Phase 4 fully CI-clean — zero clippy errors, all hp41-core and hp41-cli tests pass

affects: [05-persistence, 06-science-engineering, 07-hardening]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Do not use .is_empty() on const slices in tests — clippy::const_is_empty fires (always false)"
    - "Redundant not-empty test subsumed by exact-length assertion — keep only the 33-entry count test"

key-files:
  modified:
    - hp41-cli/src/tests/keys_tests.rs

key-decisions:
  - "Removed key_ref_table_not_empty test — clippy::const_is_empty rejects !const.is_empty(); key_ref_table_has_33_entries subsumes it"

requirements-completed:
  - DISP-01
  - DISP-02
  - INPUT-01

# Metrics
duration: ~5min
completed: 2026-05-07
---

# Phase 4 Plan 05: CI Gate Summary

**`just ci` green: clippy clean, all tests pass, hp41-core coverage 84.47% (>= 80% gate)**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-07T11:10:24Z
- **Completed:** 2026-05-07
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- `just ci` exits 0 across the full workspace
- Clippy: zero errors after removing redundant `key_ref_table_not_empty` test that used `is_empty()` on a `const` slice (triggers `clippy::const_is_empty`)
- All hp41-cli unit tests pass: 32 tests in `keys_tests` (1 removed as redundant) + 6 in `prgm_display_tests`
- All hp41-core tests pass: 100+ tests across arithmetic, math, trig, stack, registers, programming engine
- Coverage gate: hp41-core line coverage 84.47% (>= 80% threshold — PASS)

## Task Commits

1. **Task 1: Fix clippy lint and confirm just ci green** — `3d716e2` (fix(04-05))

## Files Created/Modified

- `hp41-cli/src/tests/keys_tests.rs` — Removed `key_ref_table_not_empty` test (clippy::const_is_empty violation); `key_ref_table_has_33_entries` already asserts non-empty

## Decisions Made

- Removed `key_ref_table_not_empty` test: clippy flags `!const_slice.is_empty()` as `clippy::const_is_empty` because a const slice with 33 entries can never be empty at compile time. The `key_ref_table_has_33_entries` test (`assert_eq!(len, 33)`) fully subsumes the removed test.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed test using is_empty() on const slice triggering clippy::const_is_empty**
- **Found during:** Task 1 (first just ci run)
- **Issue:** `assert!(!crate::keys::KEY_REF_TABLE.is_empty(), ...)` triggers `clippy::const_is_empty` because clippy correctly detects that a `const` slice with 33 literal entries can never be empty — the assertion is vacuously true and the negation (`!`) always evaluates to false.
- **Fix:** Removed the `key_ref_table_not_empty` test entirely. The subsequent `key_ref_table_has_33_entries` test (`assert_eq!(KEY_REF_TABLE.len(), 33)`) provides strictly stronger coverage.
- **Files modified:** `hp41-cli/src/tests/keys_tests.rs`
- **Verification:** `just ci` exits 0, no clippy errors, tests pass
- **Committed in:** `3d716e2` (task commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in test assertion)
**Impact on plan:** Minimal. One test removed, coverage strengthened by the retained 33-entry count assertion. No scope creep.

## Issues Encountered

Single clippy error on first `just ci` run: `clippy::const_is_empty` on `KEY_REF_TABLE.is_empty()` inside a test. Fixed by removing the redundant test. No other issues.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 4 (TUI & Input) is fully complete and CI-clean
- All five plans (04-01 through 04-05) committed and passing
- Ready for Phase 5: Persistence & UX (serde_json state save/load, 30s auto-save timer)

---
*Phase: 04-tui-and-input*
*Completed: 2026-05-07*
