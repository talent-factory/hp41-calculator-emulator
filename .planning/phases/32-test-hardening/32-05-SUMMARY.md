---
phase: 32-test-hardening
plan: "32-05"
subsystem: testing
tags: [coverage, gap-closure, trans, four, error-branches, math1]

# Dependency graph
requires:
  - phase: 32-01..32-04
    provides: math1 test infrastructure, lint gate, coverage baseline
provides:
  - "hp41-core/tests/math1_trans_error_branches.rs: 15 tests closing trans.rs 81.17%->>=90% gap"
  - "hp41-core/tests/math1_four_error_branches.rs: 14 tests closing four.rs 81.29%->>=90% gap"
affects: [32-06, 32-07, 32-08, 32-09, QUAL-01]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "state.regs.truncate(N) to trigger register-count guard arms in submit_step"
    - "matches!(result, Err(HpError::Domain)) for error-branch assertions without decimal compare"
    - "assert_relative_eq!(x, 0.0, max_relative=1e-6, epsilon=1e-9) for near-zero float assertions"

key-files:
  created:
    - hp41-core/tests/math1_trans_error_branches.rs
    - hp41-core/tests/math1_four_error_branches.rs
  modified: []

key-decisions:
  - "Reconned submit_step ForwardPrompt/InversePrompt arms: they set modal_prompt but NOT modal_program — corrected test assertions accordingly (Rule 1 fix during Task 2)"
  - "trans.rs line 87 f64_to_hpnum Domain path classified UNREACHABLE from tests (intermediate f64 arithmetic on finite inputs cannot produce NaN in practice)"
  - "four.rs line 87 f64_to_hpnum Domain path classified UNREACHABLE from tests (same rationale as trans.rs)"
  - "Pre-existing lint_math1_assertions failure (math1_matrix/integ_error_branches.rs) confirmed out-of-scope via git stash verification — logged to deferred-items"

patterns-established:
  - "ForwardPrompt/InversePrompt arms: assert modal_prompt re-set, NOT modal_program (which stays None when called directly)"

requirements-completed: [QUAL-01]

# Metrics
duration: 35min
completed: 2026-05-18
---

# Phase 32 Plan 05: trans.rs + four.rs Error-Branch Gap Closure Summary

**15-test trans.rs suite + 14-test four.rs suite close both 81% coverage gaps to >=90% by exercising all REACHABLE submit_step register-count guards, Rodrigues zero-axis Domain error, and no-valid-period Domain path**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-18T16:50:00Z
- **Completed:** 2026-05-18T17:25:00Z
- **Tasks:** 4 (Task 1 reconnaissance read-only; Tasks 2-3 authored files; Task 4 coverage measurement)
- **Files created:** 2

## Accomplishments

- `math1_trans_error_branches.rs`: 15 tests, 16 `// Catches:` comments — covers Rodrigues zero-axis guard (line 117), three register-count InvalidOp arms (lines 469/484/503), Ready arm (line 569), ForwardPrompt/InversePrompt 2D+3D dispatch, Init2dPrompt/Init3dOriginPrompt/Init3dAxisPrompt success paths, do_trans3d_inverse round-trip, GRAD angle mode
- `math1_four_error_branches.rs`: 14 tests, 15 `// Catches:` comments — covers compute_dft empty-samples Domain (line 139), op_four_eval_at_t no-valid-period Domain (line 285, two cases), five register-count InvalidOp arms (lines 349/365/375/386/408), Ready arm (line 423), full workflow chain NumSamplesPrompt→Ready
- Coverage confirmed: trans.rs 92.20% lines (target ≥90%), four.rs 93.05% lines (target ≥90%)

## Arm Reachability Classification

### trans.rs

| Source line | Branch | Classification | Test name |
|-------------|--------|---------------|-----------|
| 87 | f64_to_hpnum Domain (intermediate NaN) | UNREACHABLE — finite f64 arithmetic in to_radians/trig cannot produce NaN for valid HP-41 inputs | documented |
| 117 | normalize_3d zero-axis Domain | REACHABLE | `rodrigues_zero_axis_returns_domain`, `rodrigues_near_zero_axis_returns_domain` |
| 469 | Init2dPrompt regs.len()<3 InvalidOp | REACHABLE | `submit_step_arm_469_insufficient_regs_returns_invalid_op` |
| 484 | Init3dOriginPrompt regs.len()<4 InvalidOp | REACHABLE | `submit_step_arm_484_insufficient_regs_returns_invalid_op` |
| 503 | Init3dAxisPrompt regs.len()<7 InvalidOp | REACHABLE | `submit_step_arm_503_insufficient_regs_returns_invalid_op` |
| 569 | Ready InvalidOp | REACHABLE | `submit_step_ready_returns_invalid_op` |

Reach-confirmed test count: **6 error-arm tests** (arms covered) + **9 additional success/function-entry tests** = 15 total.

### four.rs

| Source line | Branch | Classification | Test name |
|-------------|--------|---------------|-----------|
| 87 | f64_to_hpnum Domain (intermediate NaN) | UNREACHABLE — same rationale as trans.rs | documented |
| 139 | compute_dft empty-samples Domain | REACHABLE | `compute_dft_empty_samples_returns_domain` |
| 285 | op_four_eval_at_t no-valid-period Domain | REACHABLE | `four_eval_at_t_no_valid_period_returns_domain`, `four_eval_at_t_negative_period_and_zero_n_returns_domain` |
| 349 | NumSamplesPrompt regs.len()<25 InvalidOp | REACHABLE | `submit_step_arm_349_insufficient_regs_returns_invalid_op` |
| 365 | NumFreqPrompt regs.len()<26 InvalidOp | REACHABLE | `submit_step_arm_365_insufficient_regs_returns_invalid_op` |
| 375 | FirstCoeffPrompt regs.len()<27 InvalidOp | REACHABLE | `submit_step_arm_375_insufficient_regs_returns_invalid_op` |
| 386 | RectTogglePrompt regs.len()<27 InvalidOp | REACHABLE | `submit_step_arm_386_insufficient_regs_returns_invalid_op` |
| 408 | SamplePrompt target>=regs.len() InvalidOp | REACHABLE | `submit_step_arm_408_sample_out_of_bounds_returns_invalid_op` |
| 423 | Ready InvalidOp | REACHABLE | `submit_step_ready_returns_invalid_op` |

Reach-confirmed test count: **9 error-arm tests** + **5 success-path tests** = 14 total.

## Task Commits

1. **Task 1: Reconnaissance** — read-only, no commit
2. **Task 2: math1_trans_error_branches.rs** — `520a661` (test)
3. **Task 3: math1_four_error_branches.rs** — `275e42c` (test)
4. **Task 4: Coverage measurement** — no file writes

## Files Created

- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/math1_trans_error_branches.rs` — 15 tests, ADR-002 header, 16 `// Catches:` comments
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/math1_four_error_branches.rs` — 14 tests, ADR-002 header, 15 `// Catches:` comments

## Decisions Made

- ForwardPrompt/InversePrompt arms in submit_step do NOT set `modal_program` (only `modal_prompt`) — test assertions corrected to reflect actual API contract (Rule 1 bug in initial test draft)
- Both line-87 `f64_to_hpnum` Domain paths classified UNREACHABLE: finite HP-41 stack values converted to f64 via `to_f64()` cannot produce NaN; trig functions on finite f64 are always finite; intermediate results cannot overflow to infinity within the tested transform ranges
- Pre-existing `lint_math1_assertions` test failure (Pitfall 14 violation in `math1_matrix_error_branches.rs:249` and `math1_integ_error_branches.rs:241/245/308/315/319`) confirmed pre-existing via `git stash` verification — out of scope for Plan 32-05

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ForwardPrompt/InversePrompt test assertions corrected**
- **Found during:** Task 2 (`submit_step_forward_prompt_2d_runs_transform` and `submit_step_inverse_prompt_2d_runs_transform`)
- **Issue:** Initial tests asserted `state.modal_program == Some(ModalProgram::Trans(ForwardPrompt))` after calling `submit_step(ForwardPrompt)` directly. The source code only sets `modal_prompt` in those arms, not `modal_program`.
- **Fix:** Changed assertions to check `state.modal_prompt == Some("FWD?")` / `"INV?"` respectively
- **Files modified:** `hp41-core/tests/math1_trans_error_branches.rs`
- **Verification:** 15/15 tests pass
- **Committed in:** `520a661` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in test assertions)
**Impact on plan:** Minimal — corrected test contract to match actual API. No scope change.

## Pre-existing Issues (Out of Scope)

Logged to `deferred-items.md` per deviation scope rules:
- `math1_integ_error_branches.rs`: compile errors (`Op::RollUp` not found) and lint violations (manual `.abs()` tolerance patterns)
- `math1_matrix_error_branches.rs`: lint violation (manual `.abs()` tolerance at line 249)
- `cargo llvm-cov --package hp41-core` full-suite run: SIGKILL on lib tests (OOM during instrumented binary execution on Apple M-series) — per-file coverage confirmed via targeted `--test` invocations showing trans.rs 92.20% and four.rs 93.05%

## Coverage Results

| File | Before | After | Target | Status |
|------|--------|-------|--------|--------|
| ops/math1/trans.rs | 81.17% | 92.20% | ≥90% | PASSED |
| ops/math1/four.rs | 81.29% | 93.05% | ≥90% | PASSED |

Measurement method: `cargo llvm-cov --package hp41-core --summary-only --test math1_trans_error_branches --test math1_four_error_branches --test math1_four_tri_trans`

## Self-Check

- [x] `hp41-core/tests/math1_trans_error_branches.rs` exists: FOUND
- [x] `hp41-core/tests/math1_four_error_branches.rs` exists: FOUND
- [x] Commit `520a661` exists: FOUND
- [x] Commit `275e42c` exists: FOUND
- [x] `cargo test -p hp41-core --test math1_trans_error_branches`: 15 passed
- [x] `cargo test -p hp41-core --test math1_four_error_branches`: 14 passed
- [x] trans.rs line coverage ≥ 90%: 92.20% — PASSED
- [x] four.rs line coverage ≥ 90%: 93.05% — PASSED
- [x] ADR-002 disclaim header lines 1-2: both files verified
- [x] `// Catches:` count: 16 in trans file, 15 in four file

## Self-Check: PASSED

## Next Phase Readiness

- Plans 32-06 (difeq) and 32-07 (solve/integ) can proceed independently
- The pre-existing `lint_math1_assertions` failure and `math1_integ_error_branches.rs` compile errors should be fixed before running `cargo test -p hp41-core` cleanly — see deferred-items

---
*Phase: 32-test-hardening*
*Completed: 2026-05-18*
