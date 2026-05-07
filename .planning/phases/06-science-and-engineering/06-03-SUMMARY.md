---
phase: 06-science-and-engineering
plan: "03"
subsystem: hp41-core/tests, hp41-cli/src
tags: [tdd, wave-3, integration-tests, key-bindings, help-data, ci-gate]
dependency_graph:
  requires: [op_sigma_plus, op_sigma_minus, op_mean, op_sdev, op_lr, op_yhat, op_corr, op_cl_sigma_stat, op_hms_to_h, op_h_to_hms, op_hms_add, op_hms_sub]
  provides: [SCI-01 integration tests, SCI-02 integration tests, 12 key bindings, Science & Engineering help category]
  affects: [hp41-core/tests/stats_tests.rs, hp41-core/tests/hms_tests.rs, hp41-cli/src/keys.rs, hp41-cli/src/help_data.rs, hp41-cli/src/tests/keys_tests.rs]
tech_stack:
  added: []
  patterns: [integration-test-via-dispatch, push-with-lift-enabled, unicode-help-entries]
key_files:
  created: []
  modified:
    - hp41-core/tests/stats_tests.rs
    - hp41-core/tests/hms_tests.rs
    - hp41-cli/src/keys.rs
    - hp41-cli/src/help_data.rs
    - hp41-cli/src/tests/keys_tests.rs
decisions:
  - "Corrected push() helper to set lift_enabled=true before Op::PushNum — plan template used set_y() direct mutation which fails after first Σ+ leaves lift_enabled=true"
  - "Replaced set_y()+push() pattern with add_point(y,x) helper using two sequential pushes with lift_enabled=true — same pattern as Plan 02 push_xy"
  - "Used exact Decimal values (1.414213562 and 2.828427125) for SDEV test instead of 2·σx multiplication — avoids last-digit rounding divergence from independent sqrt calculations"
  - "Updated key_ref_table_has_33_entries test: 40 → 52 entries (added 12 Phase 6 bindings)"
  - "KEY_REF_TABLE conflict-free bindings per RESEARCH.md: D for SDEV (d intercepted), F for →HMS (f intercepted), b for L.R. (l=Lastx, R=RCL modal)"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-07"
  tasks_completed: 3
  tasks_total: 3
  files_created: 0
  files_modified: 5
---

# Phase 6 Plan 03: Integration Tests, Key Bindings, and CI Gate Summary

Complete integration test suites for SCI-01 (17 stats tests) and SCI-02 (15 HMS tests) with dispatch-through-core validation, plus 12 new TUI key bindings and Science & Engineering help category — `just ci` passes with 86% line coverage.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Create stats_tests.rs — SCI-01 integration tests | d56db56 | hp41-core/tests/stats_tests.rs |
| 2 | Create hms_tests.rs — SCI-02 integration tests | e8a45f3 | hp41-core/tests/hms_tests.rs |
| 3 | Wire TUI key bindings, help text, run just ci | 228abf9 | hp41-cli/src/keys.rs, help_data.rs, tests/keys_tests.rs |

## Verification Results

- `just ci` exits with code 0 (lint + test + coverage all green)
- Line coverage: 86.00% (gate ≥ 80% — PASSED)
- `test test_hms_to_h_canonical_1_3045 ... ok` — ROADMAP Phase 6 success criterion verified
- `test test_lr_slope_in_y_intercept_in_x ... ok` — D-05 slope m in Y, intercept b in X verified
- 17 stats tests in stats_tests.rs (minimum 12 required)
- 15 HMS tests in hms_tests.rs (minimum 12 required)
- Zero FAILED tests across full workspace
- `grep Op::Sdev keys.rs` → `KeyCode::Char('D') => Some(Op::Sdev)` — 'd' conflict avoided
- `grep Op::HToHms keys.rs` → `KeyCode::Char('F') => Some(Op::HToHms)` — 'f' conflict avoided
- `grep "Science & Engineering" help_data.rs` — category present with 12 op entries
- `test_all_ten_categories_present` still passes (uses `any()` matching, not count)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Plan template's set_y()+push() helper fails after first Σ+**
- **Found during:** Task 1 — test_sigma_plus_accumulates_sum_x analysis
- **Issue:** Plan 03 template used `set_y(&mut s, 2); push(&mut s, 7)` for the second data point. After the first Σ+ sets `lift_enabled=true`, calling `push(7)` triggers a stack lift before setting X, causing Y to receive the count (1) instead of the intended value (2). This would make all multi-point stats tests fail.
- **Fix:** Replaced `set_y()+push()` with `add_point(y, x)` helper that calls `push(y)` then `push(x)`, both with `lift_enabled=true` set first. Two sequential lifts correctly place y in Y and x in X.
- **Files modified:** hp41-core/tests/stats_tests.rs
- **Commit:** d56db56

**2. [Rule 1 - Bug] SDEV test 2·σx comparison fails due to rounding divergence**
- **Found during:** Task 1 — test_sdev_sample_two_points**
- **Issue:** Plan template used `sigma_y == 2 * sigma_x` to verify σy=2·σx for Y=2X data. σx=sqrt(2) rounded to 10 sig digits = 1.414213562; 2·σx = 2.828427124. But σy=sqrt(8) computed independently = 2.828427125 (last digit differs due to intermediate rounding). Assertion failed.
- **Fix:** Replaced with exact expected values: `sigma_x == 1.414213562` and `sigma_y == 2.828427125`. Both are independently correct rounded values.
- **Files modified:** hp41-core/tests/stats_tests.rs
- **Commit:** d56db56

**3. [Rule 3 - Blocking] key_ref_table_has_33_entries test checks exact count**
- **Found during:** Task 3 — `just test` after adding 12 KEY_REF_TABLE entries
- **Issue:** `hp41-cli/src/tests/keys_tests.rs` asserts `KEY_REF_TABLE.len() == 40`. Adding 12 Phase 6 entries would make it 52, failing the test.
- **Fix:** Updated assertion to `== 52` with comment explaining Phase 6 addition.
- **Files modified:** hp41-cli/src/tests/keys_tests.rs
- **Commit:** 228abf9

## Known Stubs

None — all 12 operations fully implemented (Plan 02). All stubs from Plan 01 replaced.

## Threat Flags

None. T-06-05 (key binding tampering) mitigated per threat register: 'd'/'f' conflict analysis verified in app.rs (lines 291-310), using 'D'/'F' per RESEARCH.md Pitfall 1 findings. Existing bindings unmodified per D-09.

## Self-Check: PASSED

- hp41-core/tests/stats_tests.rs: FOUND (17 tests)
- hp41-core/tests/hms_tests.rs: FOUND (15 tests)
- hp41-cli/src/keys.rs (12 new bindings): FOUND
- hp41-cli/src/help_data.rs (Science & Engineering): FOUND
- Commit d56db56 (stats_tests.rs): FOUND
- Commit e8a45f3 (hms_tests.rs): FOUND
- Commit 228abf9 (keys.rs + help_data.rs): FOUND
- just ci: exit code 0
- test_hms_to_h_canonical_1_3045: PASSED
- test_lr_slope_in_y_intercept_in_x: PASSED
