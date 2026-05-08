---
phase: 06-science-and-engineering
plan: "02"
subsystem: hp41-core/ops
tags: [tdd, wave-2, stats, hms, science-engineering]
dependency_graph:
  requires: [HpError::InvalidInput, Op::SigmaPlus, Op::SigmaMinus, Op::Mean, Op::Sdev, Op::LR, Op::Yhat, Op::Corr, Op::ClSigmaStat, Op::HmsToH, Op::HToHms, Op::HmsAdd, Op::HmsSub]
  provides: [op_sigma_plus, op_sigma_minus, op_mean, op_sdev, op_lr, op_yhat, op_corr, op_cl_sigma_stat, op_hms_to_h, op_h_to_hms, op_hms_add, op_hms_sub]
  affects: [hp41-core/src/ops/stats.rs, hp41-core/src/ops/hms.rs, hp41-core/tests/stats_tests.rs, hp41-core/tests/hms_tests.rs]
tech_stack:
  added: []
  patterns: [tdd-red-green, atomic-register-write, integer-seconds-arithmetic, string-split-field-extraction, enter_number-lift-pattern]
key_files:
  created: []
  modified:
    - hp41-core/src/ops/stats.rs
    - hp41-core/src/ops/hms.rs
    - hp41-core/tests/stats_tests.rs
    - hp41-core/tests/hms_tests.rs
decisions:
  - "HMS+/HMS- use integer seconds arithmetic to avoid 10-digit rounding precision loss (0.3333... * 3600 != 1200)"
  - "push_dec test helper sets lift_enabled=true before Op::PushNum — Op::PushNum in dispatch() does not call apply_lift_effect unlike flush_entry_buf"
  - "op_sigma_plus/minus compute all register updates atomically before any write (guard against partial failure)"
  - "execute_op and prgm_display arms already complete from Plan 01 Rule 3 auto-fixes — no changes needed in Task 3"
metrics:
  duration: "~30 minutes"
  completed: "2026-05-07"
  tasks_completed: 3
  tasks_total: 3
  files_created: 0
  files_modified: 4
---

# Phase 6 Plan 02: Stats and HMS Implementation Summary

8 statistics operations (Σ+, Σ−, MEAN, SDEV, L.R., YHAT, CORR, CLΣSTAT) and 4 HMS conversion operations (HMS→, →HMS, HMS+, HMS−) fully implemented with TDD, replacing Plan 01 stubs; execute_op and prgm_display arms were already complete from Plan 01.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| RED | TDD test gate — stats and HMS failing tests | 3c0586d | hp41-core/tests/stats_tests.rs, hms_tests.rs |
| 1+2 GREEN | stats.rs and hms.rs implementations | 05fc2aa | hp41-core/src/ops/stats.rs, hms.rs |
| 3 | execute_op + prgm_display arms | 9b485e6 (Plan 01) | Already complete — no changes needed |

## Verification Results

- `just build` produces zero error lines
- `grep -c "^pub fn op_" hp41-core/src/ops/stats.rs` = 8
- `grep -c "^pub fn op_" hp41-core/src/ops/hms.rs` = 4
- `grep "SigmaPlus" hp41-core/src/ops/program.rs` matches (execute_op arm)
- `grep "SigmaPlus" hp41-cli/src/prgm_display.rs` matches (prgm_display arm)
- `just test` = 370 tests pass, 0 failed, 0 ignored
- No `unwrap()` or `expect()` in stats.rs or hms.rs non-test code
- `HpError::InvalidInput` returned by validate_hms for minutes >= 60 or seconds >= 60

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] test push_dec helper needed lift_enabled=true**
- **Found during:** Task 1 GREEN (stats tests failing 4 of 20)
- **Issue:** `Op::PushNum(v)` in `dispatch()` does NOT call `apply_lift_effect(Enable)` — unlike `flush_entry_buf()` which does. Consecutive `push_dec()` calls without `lift_enabled=true` all overwrite X (stack lift never fires). Tests for binary ops like MEAN and LR were using Y=0 instead of the intended Y value.
- **Fix:** Added `state.stack.lift_enabled = true;` before `dispatch(state, Op::PushNum(...))` in both `push_dec()` helpers in stats_tests.rs and hms_tests.rs. This matches the pattern in math_tests.rs (line 203: `push(&mut s, 3); s.stack.lift_enabled = true;`).
- **Files modified:** hp41-core/tests/stats_tests.rs, hp41-core/tests/hms_tests.rs
- **Commit:** 05fc2aa

**2. [Rule 1 - Bug] HMS+/HMS− precision loss via decimal intermediate**
- **Found during:** Task 2 GREEN (2 of 11 HMS tests failing)
- **Issue:** Original `op_hms_add`/`op_hms_sub` converted H.MMSS to decimal hours (e.g., 20min → 20/60 = 0.3333333333...), added, then converted back. `HpNum::rounded` truncates to 10 significant digits, so `0.3333333333 * 3600 = 1199.9999988` which truncates to 1199 total seconds instead of 1200. Result: `1.4500 + 0.2000` produced `2.0459` instead of `2.0500`.
- **Fix:** Replaced decimal intermediate with integer seconds arithmetic. `hms_to_total_secs()` converts H.MMSS fields + sign to signed total seconds (integer, no float). `total_secs_to_hms_fields()` converts back. No rounding error possible for any valid H.MMSS values (integer field arithmetic throughout).
- **Files modified:** hp41-core/src/ops/hms.rs
- **Commit:** 05fc2aa

**3. [Observation] Task 3 already complete from Plan 01**
- **Found during:** Task 3 verification
- **Issue:** Plan 02 Task 3 specifies adding `execute_op()` arms in program.rs and `op_display_name()` arms in prgm_display.rs. These were already added in Plan 01 commit 9b485e6 as Rule 3 auto-fixes (non-exhaustive match compile errors). All 12 arms exist and are correct.
- **Action:** No changes made. Task 3 verified as complete via grep.

## Known Stubs

None — all 12 operations are fully implemented with real logic. Plan 01 stubs replaced in this plan.

## TDD Gate Compliance

RED gate: commit 3c0586d — test stubs replaced with 20 stats tests + 11 HMS tests, all failing against stub implementations.
GREEN gate: commit 05fc2aa — implementations pass all 31 new tests; 370 total tests pass.
REFACTOR: not needed — code clean, no duplication.

## Threat Flags

None. The T-06-03 (DoS via parse_hms overflow) threat was mitigated: `to_i64()` returns `None` on overflow → mapped to `HpError::Overflow` in `decimal_to_hms_fields`. No panic path.

## Self-Check: PASSED

- hp41-core/src/ops/stats.rs: FOUND
- hp41-core/src/ops/hms.rs: FOUND
- hp41-core/tests/stats_tests.rs: FOUND
- hp41-core/tests/hms_tests.rs: FOUND
- Commit 3c0586d (RED test gate): FOUND
- Commit 05fc2aa (GREEN implementations): FOUND
- just build: 0 errors
- just test: 0 failures
