---
phase: 06-science-and-engineering
plan: "01"
subsystem: hp41-core/ops
tags: [tdd, wave-0, scaffold, error-types, op-enum]
dependency_graph:
  requires: []
  provides: [HpError::InvalidInput, Op::SigmaPlus, Op::SigmaMinus, Op::Mean, Op::Sdev, Op::LR, Op::Yhat, Op::Corr, Op::ClSigmaStat, Op::HmsToH, Op::HToHms, Op::HmsAdd, Op::HmsSub, stats_tests_stub, hms_tests_stub]
  affects: [hp41-core/src/error.rs, hp41-core/src/ops/mod.rs, hp41-core/src/ops/program.rs, hp41-cli/src/prgm_display.rs]
tech_stack:
  added: []
  patterns: [tdd-red-green, wave-0-stub, placeholder-modules]
key_files:
  created:
    - hp41-core/tests/stats_tests.rs
    - hp41-core/tests/hms_tests.rs
    - hp41-core/src/ops/stats.rs
    - hp41-core/src/ops/hms.rs
  modified:
    - hp41-core/src/error.rs
    - hp41-core/src/tests.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
decisions:
  - "Placeholder stats.rs and hms.rs created (not empty) to keep build green; Plan 02 replaces them"
  - "execute_op() in program.rs updated alongside dispatch() to prevent non-exhaustive match compile error"
  - "prgm_display.rs arms added as Rule 3 auto-fix: no wildcard arm means missing variants cause compile error"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-07"
  tasks_completed: 3
  tasks_total: 3
  files_created: 4
  files_modified: 5
---

# Phase 6 Plan 01: Wave 0 Scaffold and Type Foundation Summary

Wave 0 test stubs for SCI-01/SCI-02 plus HpError::InvalidInput variant and 12 new Op enum variants with dispatch arms establish the compilation foundation for Phase 6.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Create Wave 0 test stub files | 6490fc6 | hp41-core/tests/stats_tests.rs, hms_tests.rs |
| 2 | Add HpError::InvalidInput (TDD) | 389728b | hp41-core/src/error.rs, tests.rs |
| 3 | Add 12 Op variants and dispatch arms | 9b485e6 | hp41-core/src/ops/mod.rs, program.rs, stats.rs, hms.rs, prgm_display.rs |

## Verification Results

- `just build` produces zero error lines
- `just test` shows all existing tests passing, 13 stubs ignored (5 hms + 8 stats)
- `HpError::InvalidInput` present in error.rs with display "invalid input"
- Two new hperror tests (`hperror_invalid_input_message`, `hperror_invalid_input_is_partialeq`) pass
- All 12 Phase 6 Op variants present in enum and dispatch() match
- `pub mod stats` and `pub mod hms` declared in ops/mod.rs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Placeholder files with function stubs instead of comment-only files**
- **Found during:** Task 3
- **Issue:** Adding `pub mod stats` and `pub mod hms` to mod.rs + dispatch arms referencing `stats::op_sigma_plus()` etc. requires the functions to exist in the modules. An empty comment-only placeholder file compiles the module declaration but causes "unresolved function" errors on the dispatch arms.
- **Fix:** Created placeholder stats.rs and hms.rs with no-op stub functions returning `Err(HpError::InvalidOp)`. These are replaced wholesale by Plan 02.
- **Files modified:** hp41-core/src/ops/stats.rs, hp41-core/src/ops/hms.rs
- **Commit:** 9b485e6

**2. [Rule 3 - Blocking] Added execute_op() arms in program.rs**
- **Found during:** Task 3
- **Issue:** program.rs `execute_op()` has no wildcard arm — adding 12 new Op variants without matching arms caused a non-exhaustive match compile error.
- **Fix:** Added 12 Phase 6 arms to execute_op() calling `super::stats::*` and `super::hms::*`, per PATTERNS.md guidance.
- **Files modified:** hp41-core/src/ops/program.rs
- **Commit:** 9b485e6

**3. [Rule 3 - Blocking] Added prgm_display.rs arms in hp41-cli**
- **Found during:** Task 3
- **Issue:** prgm_display.rs `op_display_name()` has no wildcard arm — the 12 new Op variants caused a non-exhaustive match compile error in hp41-cli.
- **Fix:** Added 12 Phase 6 arms with Unicode display strings as documented in PATTERNS.md.
- **Files modified:** hp41-cli/src/prgm_display.rs
- **Commit:** 9b485e6

## Known Stubs

| File | Location | Description |
|------|----------|-------------|
| hp41-core/src/ops/stats.rs | all functions | Placeholder implementations returning Err(InvalidOp); replaced in Plan 02 |
| hp41-core/src/ops/hms.rs | all functions | Placeholder implementations returning Err(InvalidOp); replaced in Plan 02 |
| hp41-core/tests/stats_tests.rs | all 8 tests | #[ignore] stubs; un-ignored in Plan 03 after implementations land |
| hp41-core/tests/hms_tests.rs | all 5 tests | #[ignore] stubs; un-ignored in Plan 03 after implementations land |

These stubs are intentional Wave 0 scaffolding — they provide compile-time signal (Op variants exist, function signatures correct) without blocking the existing test suite. Plan 02 replaces the placeholder module files with real implementations.

## Threat Flags

None. All changes are pure type system extensions with no external input surface. HpError::InvalidInput is an internal error value used by the calculator state machine.

## Self-Check: PASSED

- hp41-core/tests/stats_tests.rs: FOUND
- hp41-core/tests/hms_tests.rs: FOUND
- hp41-core/src/error.rs (InvalidInput): FOUND
- hp41-core/src/ops/mod.rs (pub mod stats, pub mod hms): FOUND
- hp41-core/src/ops/stats.rs: FOUND
- hp41-core/src/ops/hms.rs: FOUND
- Commit 6490fc6: FOUND (test stubs)
- Commit 389728b: FOUND (HpError::InvalidInput)
- Commit 9b485e6: FOUND (Op variants + dispatch)
