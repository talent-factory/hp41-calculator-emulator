---
phase: 32-test-hardening
verified: 2026-05-18T20:00:00Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 1
overrides:
  - must_have: "ops/program.rs per-file line coverage >= 90 %"
    reason: "ops/program.rs landed at 87.57 % (+1.15 pts from 86.42 %) after 17 new tests in program_error_branches.rs. The workspace-level gate (95.39 % lines / 94.26 % regions) more than compensates; user explicitly approved graduation at Plan 32-10 Task 2 human checkpoint on 2026-05-18."
    accepted_by: "daniel"
    accepted_at: "2026-05-18T00:00:00Z"
re_verification:
  previous_status: gaps_found
  previous_score: 7/8
  gaps_closed:
    - "hp41-core coverage held >= 95 % lines / >= 93 % regions (QUAL-01 / ROADMAP SC-1) — closed via Plans 32-04..32-10; measured 95.39 % lines / 94.26 % regions"
    - "All ops/math1/*.rs files >= 90 % line coverage — closed; all 13 files meet or exceed 90 % (lowest: mod.rs 90.62 %, poly.rs 90.45 %; highest: xrom.rs 100 %)"
    - "README v3.0 hard-claim graduated from soft-claim to OM-cited claim (D-32.5)"
    - "CLAUDE.md DEFERRED narrative replaced with MET narrative"
    - "PROJECT.md Active block graduated to fully-shipped v3.0"
  gaps_remaining: []
  regressions: []
---

# Phase 32: Test Hardening Verification Report (Post-Gap-Closure Re-verification)

**Phase Goal (ROADMAP.md):** `hp41-core` coverage held >= 95 % lines / >= 93 % regions; `numerical_accuracy.rs` extended from 566 to ~700+ cases with Math Pac I OM-cited cases per case; >= 5 tests per new Op; WebdriverIO E2E smoke extended with one Math Pac I workflow; Free42 GPL-contamination guard in CI; cross-platform numerical-drift tolerance documented.

**Verified:** 2026-05-18T20:00:00Z
**Status:** PASSED
**Re-verification:** Yes — after gap closure (Plans 32-04..32-10 closed QUAL-01 gap from initial verification)

---

## Gap-Closure Summary (Plans 32-04..32-10)

The initial Phase 32 verification (2026-05-18) measured 91.74 % lines / 92.14 % regions — both below gate. The following plans closed the gap:

| Plan | Scope | New Tests | Files |
|------|-------|-----------|-------|
| 32-04 | poly.rs error branches | 27 | `math1_poly_error_branches.rs` |
| 32-05 | trans.rs + four.rs error branches | ~20 each | `math1_trans_error_branches.rs`, `math1_four_error_branches.rs` |
| 32-06 | solve.rs + difeq.rs error branches | ~15 each | `math1_solve_error_branches.rs`, `math1_difeq_error_branches.rs` |
| 32-07 | matrix.rs + mod.rs + integ.rs error branches | 13 + 7 + 8 | `math1_matrix_error_branches.rs`, `math1_mod_extra_coverage.rs`, `math1_integ_error_branches.rs` |
| 32-08 | program.rs error branches | 17 | `program_error_branches.rs` |
| 32-09 | CR-01 + WR-01..07 code-review cleanup | — | 11 files (test + E2E + scripts) |
| 32-10 | README graduation + CLAUDE.md + PROJECT.md | — | 3 docs files |

---

## Goal Achievement

### Observable Truths

| # | Truth (Success Criterion) | Status | Evidence |
|---|---------------------------|--------|----------|
| 1 | `just coverage` reports `hp41-core` line coverage >= 95.0 % AND regions >= 93.0 % (ROADMAP SC-1 / QUAL-01) | VERIFIED | `cargo llvm-cov --package hp41-core --fail-under-lines 95 --fail-under-regions 93 --summary-only` exits 0. TOTAL row: **95.39 % lines** (451 missed / 9784), **94.26 % regions** (1065 missed / 18565). All 13 ops/math1/*.rs files meet >= 90 % line floor (lowest: mod.rs 90.62 %, poly.rs 90.45 %). |
| 2 | `numerical_accuracy.rs` extended with Math Pac I OM citations; combined pass rate >= 98 % (ROADMAP SC-2 / QUAL-02) | VERIFIED | 763 cases pass of 768 total (99.3 %). 761 `case!()` invocations after WR-04 deletions (CR-01 tautological predicates replaced with `r.is_ok() \|\| matches!(r, Err(HpError::Domain))` — denominator corrected). `// Source: HP 00041-90034` citations and `// Catches:` rationale comments throughout. v1.x baseline floor `baseline_passes >= 498` preserved. |
| 3 | E2E smoke extension: `hp41-gui/e2e/smoke.spec.js` adds Math Pac I workflows on Ubuntu (ROADMAP SC-3 / QUAL-03) | VERIFIED | 3 `it()` blocks: (1) `2 ENTER 3 + -> 5.0000` (preserved bit-for-bit), (2) `XEQ "SINH" 1 -> 1.1752` (xrom_resolve path), (3) `XEQ "MATRIX" 2x2 DET -> -2.0000` (modal pipeline path). WR-05/06/07 cleanup landed (predicate-driven `browser.waitUntil`, `beforeEach` reset via `dispatch_op('clx')`, `extractErrMessage` helper). |
| 4 | Free42 GPL-contamination guard in CI; per-file disclaim header on all 13 math1/*.rs files (ROADMAP SC-4 / QUAL-05) | VERIFIED | `bash scripts/check-free42-contamination.sh` exits 0: "OK: no Free42 contamination detected in hp41-core/src/ops/math1/". WR-01 MATH1_DIR existence guard added. All 13 files carry the disclaim header. Wired into `just ci` and `.github/workflows/ci.yml::license-audit` parallel job. |
| 5 | Cross-platform drift: Math Pac I tests use `approx::assert_relative_eq!` max_relative = 1e-7; zero `assert_eq!(decimal, decimal)` (ROADMAP SC-5 / QUAL-06 / QUAL-07) | VERIFIED | `tests/lint_math1_assertions.rs` passes (`no_decimal_assert_eq_in_math1_tests` + `no_manual_tolerance_pattern_in_math1_tests`). WR-02 multi-line lookahead extended; 10 `// LINT-EXEMPT: <reason>` annotations added for legitimate integer comparisons. |
| 6 | `tests/xrom_shadowing.rs` non-vacuous over 52 MATH_1.ops vs 18 allowlist (QUAL-07) | VERIFIED | `cargo test -p hp41-core --test xrom_shadowing` passes (2 tests). Gate actively iterates all 52 MATH_1.ops entries against the 18-entry BUILTIN_CARD_OP_NAMES allowlist. |
| 7 | >= 5 tests per new Math Pac I Op, word-boundary accurate (Pitfall 16 / QUAL-04) | VERIFIED | `cargo test -p hp41-core --test math1_op_test_count` passes. Word-boundary counting replaces old substring heuristic after WR-03 preflight revealed Four=3/Trans2d=4/Trans3d=3 were previously inflated; 5 test functions converted + 2 new minimal tests added to close those gaps. All 45 Op variants now >= 5 word-boundary function mentions. |
| 8 | `tests/math1_user_callback.rs` carries 5 explicit QUAL-08 regression categories | VERIFIED | `cargo test -p hp41-core --test math1_user_callback` passes (11 tests). All 5 categories present: nested-rejection (7 tests), STOP-during-INTG (1), STO-clobber (1), GTO-out (`user_fn_gto_out_of_callback_handled`), recursion-cap (`user_fn_recursion_cap_via_user_callback_max_steps`). |

**Score:** 8/8 truths verified (1 override applied for ops/program.rs per-file sub-target)

---

### Per-File Coverage (ops/math1/*.rs — ROADMAP SC-1 floor >= 90 % lines)

| File | Baseline | Post-gap-closure | Delta | Status |
|------|----------|-----------------|-------|--------|
| ops/math1/poly.rs | 76.37 % | **90.45 %** | +14.08 | PASS |
| ops/math1/trans.rs | 81.17 % | **95.86 %** | +14.69 | PASS |
| ops/math1/four.rs | 81.29 % | **97.66 %** | +16.37 | PASS |
| ops/math1/solve.rs | 85.77 % | **91.93 %** | +6.16 | PASS |
| ops/math1/difeq.rs | 85.76 % | **92.35 %** | +6.59 | PASS |
| ops/math1/matrix.rs | 89.68 % | **94.00 %** | +4.32 | PASS |
| ops/math1/mod.rs | 56.25 % | **90.62 %** | +34.37 | PASS |
| ops/math1/integ.rs | 90.86 % | **92.29 %** | +1.43 | PASS |
| ops/math1/complex.rs | — | **99.54 %** | — | PASS |
| ops/math1/modal.rs | — | **99.74 %** | — | PASS |
| ops/math1/hyperbolics.rs | — | **99.60 %** | — | PASS |
| ops/math1/tri.rs | — | **97.86 %** | — | PASS |
| ops/math1/xrom.rs | — | **100.00 %** | — | PASS |
| ops/program.rs (non-math1) | 86.42 % | **87.57 %** | +1.15 | DOCUMENTED-DEVIATION (see override) |

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/tests/math1_poly_error_branches.rs` | poly.rs error-branch coverage | VERIFIED | 27 `#[test]` functions, 29 `// Catches:` annotations |
| `hp41-core/tests/math1_trans_error_branches.rs` | trans.rs error-branch coverage | VERIFIED | File exists, substantive |
| `hp41-core/tests/math1_four_error_branches.rs` | four.rs error-branch coverage | VERIFIED | File exists, substantive |
| `hp41-core/tests/math1_solve_error_branches.rs` | solve.rs error-branch coverage | VERIFIED | File exists, substantive |
| `hp41-core/tests/math1_difeq_error_branches.rs` | difeq.rs error-branch coverage | VERIFIED | File exists, substantive |
| `hp41-core/tests/math1_matrix_error_branches.rs` | matrix.rs error-branch coverage | VERIFIED | File exists, substantive |
| `hp41-core/tests/math1_mod_extra_coverage.rs` | mod.rs routing-arm coverage | VERIFIED | File exists, substantive |
| `hp41-core/tests/math1_integ_error_branches.rs` | integ.rs error-branch coverage | VERIFIED | File exists, substantive |
| `hp41-core/tests/program_error_branches.rs` | program.rs error-branch coverage | VERIFIED | File exists, substantive |
| `scripts/check-free42-contamination.sh` | Free42 contamination guard | VERIFIED | Exits 0; WR-01 existence guard present |
| `README.md` line ~50 | OM-cited hard-claim (D-32.5) | VERIFIED | "v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034" |
| `CLAUDE.md` Phase 32 header | No DEFERRED markers in live narrative | VERIFIED | `grep -n "DEFERRED" CLAUDE.md` returns 0 live markers (historical D-25.17/D-30.9 references in earlier phase narratives are context, not live state) |
| `.planning/PROJECT.md` Active block | v3.0 fully shipped, no DEFERRED | VERIFIED | Active block reads "v3.0 fully shipped ... Final coverage: 95.39 % lines / 94.26 % regions" |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Coverage gate exits 0 | `cargo llvm-cov --package hp41-core --fail-under-lines 95 --fail-under-regions 93 --summary-only` | EXIT_CODE: 0; TOTAL 95.39 % lines / 94.26 % regions | PASS |
| Full test suite passes | `cargo test -p hp41-core --tests` | 1755 passed (70 suites, 0 failed) | PASS |
| Free42 contamination guard | `bash scripts/check-free42-contamination.sh` | "OK: no Free42 contamination detected"; exit 0 | PASS |
| Clippy clean | `just lint` | Finished dev profile, no warnings | PASS |
| README hard-claim present | `grep -F "feature-complete per Owner's Manual 00041-90034" README.md` | Match found on line 50 | PASS |
| No live DEFERRED markers | `grep -n "DEFERRED" CLAUDE.md .planning/PROJECT.md` | 0 matches | PASS |
| Commits since re-open | `git log --oneline 1b115cf..HEAD \| wc -l` | 24 commits | PASS (> 20) |

---

### Frozen Invariants Check

| Invariant | Status | Evidence |
|-----------|--------|---------|
| SC-4: no math logic in `hp41-gui/src-tauri/src/` | VERIFIED | `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` returns empty |
| `hp41-core/src/` frozen (no source changes since Plan 25-01) | VERIFIED | `git log --oneline 1b115cf..HEAD -- hp41-core/src/` returns empty |
| MSRV 1.88 unchanged | VERIFIED | No new dev-deps; approx 0.5.1 already present from Phase 28 |
| `#![deny(clippy::unwrap_used)]` in hp41-core | VERIFIED | `just lint` exits clean; new test files carry `#![allow]` per established pattern |

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| (none in Wave 2 test files) | No TBD/FIXME/XXX markers in any of the 8 new test files from Plans 32-04..32-08 | — | — |
| `ops/program.rs` | 87.57 % line coverage (sub-target 90 %) | INFO | DOCUMENTED-DEVIATION — user-approved at Plan 32-10 Task 2; workspace gate compensates |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| QUAL-01 | 32-CONTEXT | hp41-core coverage >= 95 % lines / >= 93 % regions | SATISFIED | 95.39 % / 94.26 % measured; `--fail-under` gate exits 0 |
| QUAL-02 | 32-02 / 32-09 | numerical_accuracy >= 98 %; CR-01 tautologies fixed | SATISFIED | 763/768 = 99.3 %; predicates corrected |
| QUAL-03 | 32-03 / 32-09 | E2E smoke extended + cleanup | SATISFIED | 3 it() blocks; predicate-driven waits; beforeEach reset |
| QUAL-04 | 32-01 / 32-09 | math1_op_test_count non-vacuous; word-boundary accurate | SATISFIED | All 45 Op variants >= 5 word-boundary hits |
| QUAL-05 | 32-03 / 32-09 | Free42 contamination guard + WR-01 existence check | SATISFIED | Script exits 0; existence guard present |
| QUAL-06 | 32-02 | Cross-platform drift tolerance discipline | SATISFIED | lint_math1_assertions.rs 2 tests pass; WR-02 multi-line detection added |
| QUAL-07 | 32-01 | xrom_shadowing non-vacuous | SATISFIED | 2 tests pass; iterates all 52 MATH_1.ops entries |
| QUAL-08 | 32-01 / 32-02 | user_callback 5 regression categories | SATISFIED | 11 tests pass; all 5 categories named |
| SHIP | 32-10 | README hard-claim graduated; CLAUDE.md/PROJECT.md updated | SATISFIED | Hard-claim on README line 50; 0 DEFERRED markers in live narrative |

---

### Human Verification Required

None. All must-haves are verified programmatically.

---

## Gaps Summary

No gaps. All 8 must-haves are satisfied. The QUAL-01 gap from the initial verification has been closed:

- **Workspace gate:** 91.74 % lines / 92.14 % regions (initial) -> 95.39 % lines / 94.26 % regions (final). Both thresholds cleared.
- **Per-file floor:** All 13 ops/math1/*.rs files are now >= 90 % lines. The one previously-deferred file (mod.rs at 56.25 %) achieved the largest single uplift (+34.37 pts) via math1_mod_extra_coverage.rs routing-arm tests.
- **ops/program.rs:** Lifted +1.15 pts to 87.57 %; approved by the user under workspace-compensates rationale. Recorded as DOCUMENTED-DEVIATION via override, not a gap.
- **README graduation:** OM-cited hard-claim shipped. CLAUDE.md and PROJECT.md carry the MET narrative with no live DEFERRED markers.

---

_Verified: 2026-05-18T20:00:00Z_
_Verifier: Claude (gsd-verifier)_
