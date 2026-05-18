---
phase: 32-test-hardening
verified: 2026-05-18T00:00:00Z
status: gaps_found
score: 7/8 requirements verified (QUAL-01 FAIL; QUAL-02 PASS-with-caveat)
overrides_applied: 0
gaps:
  - truth: "hp41-core coverage held ≥ 95 % lines / ≥ 93 % regions (QUAL-01 / ROADMAP SC-1)"
    status: failed
    reason: "Measured 91.74 % lines / 92.14 % regions on the post-merge codebase — both metrics below the 95 % / 93 % gate. Seven ops/math1/*.rs files sit below the 90 % per-file floor mandated by ROADMAP SC-1 (poly.rs 76.37 %, mod.rs 56.25 %, trans.rs 81.17 %, four.rs 81.29 %, solve.rs 85.77 %, difeq.rs 85.76 %, matrix.rs 89.68 %); integ.rs sits at 90.86 % marginal pass. Plus ops/program.rs 86.42 % and ops/math.rs 91.65 % drag the workspace total. This is the documented Rule-4 deferral surfaced by Plan 32-03 (32-03-SUMMARY.md `Rule 4 — Architectural Decision`)."
    artifacts:
      - path: "hp41-core/src/ops/math1/poly.rs"
        issue: "76.37 % line coverage — below 90 % per-file floor; error-branch arms (POLY-07 reject path, Bairstow non-convergence) not reached by Plan 32-02's OM-cited happy-path numerical_accuracy cases"
      - path: "hp41-core/src/ops/math1/mod.rs"
        issue: "56.25 % line coverage — Wave 1 surgical Plan 32-01 lifted 0 % → 56 % via 5 risk-weighted submit_modal / cancel_modal tests; remaining error branches uncovered"
      - path: "hp41-core/src/ops/math1/trans.rs"
        issue: "81.17 % line coverage; Rodrigues 3D edge cases / TRANS-04 axis-normalization paths uncovered"
      - path: "hp41-core/src/ops/math1/four.rs"
        issue: "81.29 % line coverage; FOUR-04 pair-count cap / non-DC harmonic paths uncovered"
      - path: "hp41-core/src/ops/math1/solve.rs"
        issue: "85.77 % line coverage; SOLV-04 non-convergence / SOLV-07 iteration cap branches partially covered"
      - path: "hp41-core/src/ops/math1/difeq.rs"
        issue: "85.76 % line coverage; ORDER=2 coupled RK4 / max_steps overflow branches partially covered"
      - path: "hp41-core/src/ops/math1/matrix.rs"
        issue: "89.68 % line coverage; marginally below 90 % per-file floor"
      - path: "hp41-core/src/ops/program.rs"
        issue: "86.42 % line coverage; non-math1 file dragging workspace total"
    missing:
      - "Targeted error-branch test pass on 7 below-90 % math1 files (5–15 tests per file with `// Catches: <bug class>` rationale per D-27.1)"
      - "Either close gap to ≥ 90 % per file + ≥ 95 % workspace lines, OR document acceptable per-file thresholds in an ADR raising the gap to the OM-divergence catalog (e.g., POLY non-convergence branches documented OM-divergent per Pitfall 5)"
      - "v3.0.1 follow-up milestone owns this scope per 32-03-SUMMARY.md `Rule 4 — Architectural Decision`"
deferred:
  - truth: "README v3.0 hard-claim graduation from soft-claim to OM-cited 'feature-complete per Owner's Manual 00041-90034'"
    addressed_in: "v3.0.1 follow-up milestone (planned, not yet roadmapped)"
    evidence: "Per D-30.9 / D-32.5 / D-32.6 gate-conditional discipline mirroring v2.2 HP-41CV pattern. 32-03-SUMMARY.md Rule 4 section: 'The README v3.0 soft-claim line stays as-is. CLAUDE.md + PROJECT.md updates DO reflect Phase 32 ship (the test/CI infrastructure shipped green; Wave 1+2 ALL delivered). The README OM-cited hard claim waits for a follow-up v3.0.1 milestone'. CLAUDE.md `### v3.0 additions` header line explicitly reads `Phases 28–32 — README hard-claim DEFERRED pending coverage gate` to make disposition transparent."
---

# Phase 32: Test Hardening Verification Report

**Phase Goal (ROADMAP.md):** `hp41-core` coverage held ≥ 95 % lines / ≥ 93 % regions; `numerical_accuracy.rs` extended from 566 → ~700+ cases with Math Pac I cases per program (OM-cited per case per D-27.1); ≥ 5 tests per new `Op` (per Pitfall 16 to avoid mid-milestone coverage drop); WebdriverIO E2E smoke extended with one Math Pac I workflow; Free42 GPL-contamination guard in CI; cross-platform numerical-drift tolerance documented.

**Verified:** 2026-05-18
**Status:** gaps_found
**Re-verification:** No — initial verification

---

## Goal Achievement Summary

The phase ships **most** of its goal: meta-gate graduation, lint discipline, ≥ 5 tests per new Op, ~134 new numerical-accuracy cases, E2E Math Pac I workflows, Free42 contamination guard, and full QUAL-08 user-callback rubric — all green. **But the primary ROADMAP success criterion — `hp41-core` coverage ≥ 95 % lines / ≥ 93 % regions — is NOT met.** Measured 91.74 % / 92.14 %.

The team **correctly recognized** the gap during Plan 32-03 (Rule 4 — Architectural Decision) and **deferred the README hard-claim graduation** to a v3.0.1 follow-up milestone, preserving the gate-conditional graduation discipline established for v2.2 HP-41CV. This is the right call — but it leaves QUAL-01 / ROADMAP SC-1 as a real, documented, unresolved gap. The verification status is **`gaps_found`** because the success criterion was not achieved in the codebase, regardless of how cleanly the deferral was documented.

Additionally, code review **CR-01** (32-REVIEW.md) flagged 15 tautological `r.is_ok() || r.is_err()` cases at `numerical_accuracy.rs:5867,5885,5903,5921,5944,5962,5980,5999,6018,6041,6060,6078,6097,6116,6266` — confirmed by grep. The QUAL-02 99.3 % pass-rate gate is met, but ~15 of those passing cases are guaranteed-passes that inflate the denominator without contributing regression signal. This is recorded as a **partial** for QUAL-02 with a documented caveat.

---

## Observable Truths

| # | Truth (Success Criterion) | Status | Evidence |
|---|---------------------------|--------|----------|
| 1 | `just coverage` reports `hp41-core` line coverage ≥ 95.0 % AND regions ≥ 93.0 % (ROADMAP SC-1 / QUAL-01) | ✗ FAILED | Measured 91.74 % lines / 92.14 % regions via `cargo llvm-cov --package hp41-core --summary-only` (TOTAL row: 18565 lines, 1533 missed). 7 ops/math1/*.rs files below 90 % per-file floor. |
| 2 | `numerical_accuracy.rs` extended from 566 → ~700+ cases with Math Pac I OM citations per case; combined pass rate ≥ 98 % (ROADMAP SC-2 / QUAL-02 / QUAL-06) | ⚠ PARTIAL | 574 `case!()` matches total (gate met per spirit of D-32.9 ~134 cases; planner's 700 projection conflated `case!` invocations with case-IDs); 188 `// Source: HP 00041-90034` citations; 181 `// Catches:` markers; 19 `HpError::Domain` assertions; POLY cluster sentinel present (1 instance). 763/768 = 99.3 % pass rate. **CAVEAT:** CR-01 flags 15 tautological `r.is_ok() || r.is_err()` cases inflating denominator. |
| 3 | E2E smoke extension: `hp41-gui/e2e/smoke.spec.js` adds Math Pac I workflow on Ubuntu (ROADMAP SC-3 / QUAL-03) | ✓ VERIFIED | 3 `it()` blocks at lines 96, 147, 195: `2 ENTER 3 + → 5.0000` (preserved bit-for-bit), `XEQ "SINH" 1 → 1.1752`, `XEQ "MATRIX" 2x2 DET → -2.0000`. Click strategy documented in leading comments per T-32-03. |
| 4 | Free42 GPL-contamination guard in CI; per-file disclaim header on all 13 math1/*.rs files (ROADMAP SC-4 / QUAL-05) | ✓ VERIFIED | `scripts/check-free42-contamination.sh` exists, executable, line 1 `#!/usr/bin/env bash`, `set -euo pipefail` present, all 12 D-32.7 symbols present, allowlist references disclaim sentence. `bash scripts/check-free42-contamination.sh` exits 0 with "OK: no Free42 contamination detected". `grep -L 'Free42 source consulted only as sanity-check oracle' hp41-core/src/ops/math1/*.rs` returns empty (all 13 files carry header). Wired into `Justfile` (ci recipe ends `ci: lint test coverage license-audit`) AND `.github/workflows/ci.yml::license-audit` parallel job. |
| 5 | Cross-platform drift: every Math Pac I numerical test uses `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)`; zero `assert_eq!(decimal, decimal)` (ROADMAP SC-5 / QUAL-06 / QUAL-07) | ✓ VERIFIED | `tests/lint_math1_assertions.rs` ships with 2 `#[test]` functions (`no_decimal_assert_eq_in_math1_tests`, `no_manual_tolerance_pattern_in_math1_tests`), both pass. Free42 disclaim sentence on lines 1–2 per ADR-002. WR-02 documents a heuristic gap (multi-line `assert_eq!` not detected) — caveat noted but doesn't invalidate the gate. |
| 6 | `tests/xrom_shadowing.rs` gates against name collision; non-vacuous over 52 MATH_1.ops vs 18 allowlist (ROADMAP SC-5 / QUAL-07) | ✓ VERIFIED | `cargo test -p hp41-core --test xrom_shadowing` → 2 passed; gate now actively iterates all 52 `MATH_1.ops` entries against the 18-entry `BUILTIN_CARD_OP_NAMES` allowlist per Pitfall 1 (graduated from vacuous in Plan 32-01). WR-04 flags hand-curated allowlist drift risk — info-level, doesn't fail the gate. |
| 7 | ≥ 5 tests per new Math Pac I Op (Pitfall 16 / QUAL-04) | ✓ VERIFIED | `cargo test -p hp41-core --test math1_op_test_count` → 1 passed; gate graduated from vacuous to non-vacuous, iterates all 45 Math Pac I `Op` variants against the 14 `tests/math1_*.rs` files with TriSaa=6, TriSas=6 baseline in doc-comment per T-32-04. WR-03 flags substring-match heuristic (`Sol` substring of `Solve`) — info-level for v3.1 tightening; gate is one-sided so cannot HIDE shortfalls below 5 in current codebase. |
| 8 | `tests/math1_user_callback.rs` carries 5 explicit QUAL-08 regression categories (QUAL-08) | ✓ VERIFIED | `cargo test -p hp41-core --test math1_user_callback` → 11 passed (9 pre-existing + 2 new). All 5 categories named in code: nested-rejection (7 tests), STOP-during-INTG (1), STO-clobber (1), GTO-out (`user_fn_gto_out_of_callback_handled` at line 480, NEW), recursion-cap (`user_fn_recursion_cap_via_user_callback_max_steps` at line 533, NEW). |

**Score:** 6 PASS / 1 PARTIAL / 1 FAIL = 7/8 truths in goal-state

---

## Requirements Coverage

All 8 phase requirement IDs scored against actual codebase evidence:

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| QUAL-01 | 32-01 | `hp41-core` Coverage ≥ 95 % Lines / ≥ 93 % Regions (held from v2.2) | ✗ **FAIL** | 91.74 % lines / 92.14 % regions — BELOW gate. Documented Rule-4 deferral in 32-03-SUMMARY.md is honest about this but does not change the gate outcome. |
| QUAL-02 | 32-02 | `numerical_accuracy.rs` extended 566 → ~700+ with OM citations; pass rate ≥ 98 % | ⚠ **PARTIAL** | 763/768 = 99.3 % above gate. But CR-01 documents 15 tautological cases at lines 5867–6266 inflating the denominator. The gate passes cleanly but ~15 passes are vacuous. |
| QUAL-03 | 32-03 | GUI E2E smoke extended with Math Pac I workflow on Ubuntu | ✓ **PASS** | 3 `it()` blocks in smoke.spec.js incl. SINH (1.1752) and MATRIX DET (-2.0000). WR-05 flags `browser.pause()` time-based wait pattern — info-level flake risk, not a gate fail. CI runtime validation pending on next push. |
| QUAL-04 | 32-01 | Per-Op test count ≥ 5 (Pitfall 16) | ✓ **PASS** | `math1_op_test_count.rs` graduated non-vacuous; all 45 Math Pac I Op variants ≥ 5 mentions; gate passes. WR-03 documents substring-match heuristic — for future tightening but current corpus all clean. |
| QUAL-05 | 32-03 | Free42-GPL-Contamination-Guard CI script + per-file disclaim header | ✓ **PASS** | Script exists, executable, exits 0, exercises 12 D-32.7 symbols. All 13 math1/*.rs files carry verbatim disclaim header. WR-01 documents `MATH1_DIR` non-existence silent-pass bug — warning-level, doesn't fail current gate since MATH1_DIR exists today. |
| QUAL-06 | 32-02 | Cross-Platform Numerical Drift: `approx::assert_relative_eq!` with max_relative = 1e-7 | ✓ **PASS** | `lint_math1_assertions.rs` 2 gates pass; matrix.rs / matrix_flow.rs / four_tri_trans.rs offenders refactored to `approx::assert_relative_eq!`; LINT-EXEMPT annotations for documented exceptions (Simpson floor, SSA rounding, Rodrigues round-trip). |
| QUAL-07 | 32-01 | `tests/xrom_shadowing.rs` CI gate: no Math Pac I shadows existing built-in mnemonic | ✓ **PASS** | Gate graduated from vacuous; actively cross-checks 52 MATH_1.ops mnemonics × 18-entry BUILTIN_CARD_OP_NAMES allowlist; 2 tests pass; no shadowing detected. |
| QUAL-08 | 32-01 | 5 regression tests for user-callback re-entrancy (nested INTG/SOLVE reject, STO clobber, STOP-during-INTG, GTO-out, recursion-cap) | ✓ **PASS** | All 5 categories present in `math1_user_callback.rs`: 11 tests pass; 2 new tests added in Plan 32-01 for the previously-implicit GTO-out and recursion-cap categories. |

**Coverage:** 6 SATISFIED / 1 PARTIAL / 1 BLOCKED = 7/8

**No orphaned requirements:** all 8 IDs from REQUIREMENTS.md mapped to plan frontmatter (`32-01-PLAN.md` covers QUAL-01/04/07/08; `32-02-PLAN.md` covers QUAL-02/06; `32-03-PLAN.md` covers QUAL-03/05). Verified via `grep -A5 "^requirements_addressed:" *-PLAN.md`.

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/tests/lint_math1_assertions.rs` | NEW Pitfall 14 + 17 lint | ✓ VERIFIED | 282 lines; 2 `#[test]` fns at lines 232, 260; Free42 disclaim on lines 1–2; `#![allow(clippy::unwrap_used)]` at file scope; both gates pass |
| `hp41-core/tests/math1_mod_entry_points.rs` | NEW WR-01/WR-02/step-precedence pins | ✓ VERIFIED | 152 lines; 5 risk-weighted tests for `submit_modal`/`cancel_modal`/`submit_modal_with_label`; surgically lifted `ops/math1/mod.rs` 0 % → 56.25 % |
| `hp41-core/tests/math1_op_test_count.rs` | MODIFY: delete vacuous early-return | ✓ VERIFIED | `grep -c 'if variants.is_empty()'` returns 0; `grep -c 'passes vacuously'` returns 0; doc-comment carries TriSaa=6 baseline; `// Catches:` comment on assert |
| `hp41-core/tests/xrom_shadowing.rs` | Auto-graduate on populated MATH_1.ops | ✓ VERIFIED | 87 lines; 2 tests pass; gate now iterates 52 MATH_1.ops × 18 BUILTIN_CARD_OP_NAMES |
| `hp41-core/tests/math1_user_callback.rs` | +2 explicit QUAL-08 tests | ✓ VERIFIED | 11 tests pass; new `user_fn_gto_out_of_callback_handled` line 480 + `user_fn_recursion_cap_via_user_callback_max_steps` line 533 |
| `hp41-core/tests/numerical_accuracy.rs` | +~134 risk-weighted cases with OM citations | ⚠ PARTIAL (per-CR-01) | 574 `case!()` matches total (planner projection 700+ counted case-IDs, not invocations); 188 OM citations, 181 `// Catches:`, 19 HpError::Domain assertions, 1 POLY cluster marker. D-27.6 baseline floor preserved at lines 7024–7040. **CR-01 flags 15 tautological `r.is_ok()||r.is_err()` cases.** |
| `hp41-gui/e2e/smoke.spec.js` | +2 Math Pac I `it()` blocks | ✓ VERIFIED | 3 `it()` blocks at lines 96, 147, 195; SINH + MATRIX DET added; WR-05/WR-06/WR-07 documented (info-level) |
| `scripts/check-free42-contamination.sh` | NEW Bash CI guard | ✓ VERIFIED | 27 lines; executable; line 1 `#!/usr/bin/env bash`; `set -euo pipefail`; 12 D-32.7 symbols present; `bash scripts/check-free42-contamination.sh` → exit 0. **WR-01:** silent-pass bug if `MATH1_DIR` is missing — warning-level. |
| `Justfile` | +`license-audit` recipe + extend `ci` | ✓ VERIFIED | `grep -E '^ci:'` matches `ci: lint test coverage license-audit`; license-audit recipe in `[group('ci')]` with TAB indentation |
| `.github/workflows/ci.yml` | +`license-audit` parallel job | ✓ VERIFIED | `license-audit:` job at line 100; `name: License audit (Free42 contamination)`; `runs-on: ubuntu-latest`; no `needs:` clause (parallel sibling) |
| `CLAUDE.md` | Replace Phase 32 (in progress) stub with shipped subsection | ✓ VERIFIED | `grep -c 'Phase 32.*in progress'` returns 0; new shipped subsection at line 155; cites D-32.5/6/7 + Pitfall 19 (4 mentions). Header line transparently reads `Phases 28–32 — README hard-claim DEFERRED pending coverage gate`. |
| `.planning/PROJECT.md` | Add Phase 32 to Shipped block | ✓ VERIFIED | 4 `Phase 32` mentions; Phase 32 in `Shipped milestones` block at line 47 with explicit gate-deferral note; Active block updated to mark v3.0 as 26/26 plans complete |
| `README.md` | Graduate v3.0 line from soft-claim to hard claim per D-32.5 | ⚠ DEFERRED (intentional) | Line 50: `Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry points, documented divergences)` — soft-claim PRESERVED per Rule 4 deferral. NOT a failure — the deferral is the *correct* response to QUAL-01 fail. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `Justfile` `ci` recipe | `scripts/check-free42-contamination.sh` | `license-audit` recipe → `bash scripts/check-free42-contamination.sh` | ✓ WIRED | Verified by `just license-audit` exits 0 |
| `.github/workflows/ci.yml::license-audit` job | `scripts/check-free42-contamination.sh` | `just license-audit` (via `taiki-e/install-action@v2 tool: just`) | ✓ WIRED | Job present at line 100; runs in parallel (no `needs:`) per D-32.8 belt+suspenders |
| `lint_math1_assertions.rs` | `tests/math1_*.rs` corpus | Directory scan `starts_with("math1_") && extension == "rs"` | ✓ WIRED | 2 gates pass against current corpus; LINT-EXEMPT annotations recognized inline and via preceding-block heuristic |
| `math1_op_test_count.rs` | `tests/math1_*.rs` corpus + `MATH_1.ops` | `collect_math1_variant_names()` + `count_test_mentions()` | ✓ WIRED | Test passes; all 45 Math Pac I variants ≥ 5 mentions |
| `xrom_shadowing.rs` | `MATH_1.ops` (52 entries) | Hand-curated `BUILTIN_CARD_OP_NAMES` (18 entries) — drift risk per IN-04 | ✓ WIRED | 2 tests pass; no shadowing; WR-04 info-level drift-risk |
| `smoke.spec.js` test 2 (SINH) | `dispatch_op` Tauri command | `window.__TAURI_INTERNALS__.invoke('dispatch_op', { keyId: 'xeq_SINH' })` browser.execute fallback | ✓ WIRED (fallback) | Per documented click-strategy decision (modal alphaChar/Enter collision) |
| `smoke.spec.js` test 3 (MATRIX) | modal pipeline + `dispatch_op` | Hybrid: real-clicks for digits/R-S + `__TAURI_INTERNALS__.invoke` for xeq_MATRIX/xeq_DET | ✓ WIRED (hybrid) | Exercises D-31.1 R/S 3-way routing + column-major iteration |
| `CLAUDE.md` Phase 32 subsection | All 5 D-32.X locked decisions + Pitfall 19 | Citations in subsection body | ✓ WIRED | 4 distinct mentions of D-32.5/D-32.6/D-32.7/Pitfall 19 in subsection at line 155 |

---

## Anti-Patterns Found

| File | Line(s) | Pattern | Severity | Impact |
|------|---------|---------|----------|--------|
| `hp41-core/tests/numerical_accuracy.rs` | 5867, 5885, 5903, 5921, 5944, 5962, 5980, 5999, 6018, 6041, 6060, 6078, 6097, 6116, 6266 | `if r.is_ok() \|\| r.is_err() { 1.0 } else { 0.0 }` tautology | 🛑 BLOCKER (per CR-01) | 15 cases inflate QUAL-02 denominator with guaranteed passes; no regression signal. Each case carries a `// Catches:` claim that is unverified by the case body. Exactly the T-32-04 meta-test-gaming threat. Recorded as **partial** for QUAL-02 not silently passed. |
| `scripts/check-free42-contamination.sh` | 11, 20 | Missing `[[ ! -d "$MATH1_DIR" ]]` existence check | ⚠ WARNING (per WR-01) | Script silently exits 0 if `MATH1_DIR` is renamed/missing — exactly when contamination is most likely to slip in (refactor). Doesn't fail current gate. |
| `hp41-core/tests/lint_math1_assertions.rs` | 161–183 | Multi-line `assert_eq!(decimal, decimal)` not detected | ⚠ WARNING (per WR-02) | Single-line heuristic fails on idiomatic multi-line Rust. Documented blind spot — doesn't fail Pitfall 17 in current corpus. |
| `hp41-core/tests/math1_op_test_count.rs` | 77–110 | `contains(variant_name)` is substring match — `Sol` matches `Solve`, `Sinh` matches `Asinh` | ⚠ WARNING (per WR-03) | One-sided gate (only fails on < 5) so substring inflation cannot HIDE shortfalls in current corpus. But the docstring claim "≥ 5 test functions" is technically unmet (mentions, not fns). |
| `hp41-core/tests/numerical_accuracy.rs` | 4344–4346, 4361–4363, 4962–4964, 4980–4982, 5680–5682, 6180–6182, 6803–6805 | `case!("name", "desc", 1.0, 1.0)` redundant sentinels after real `assert!` | ⚠ WARNING (per WR-04) | 7 sentinels inflate 98% gate denominator. Less severe than CR-01 because real `assert!` is up-front. |
| `hp41-gui/e2e/smoke.spec.js` | 110, 159, 202, 209, 215, 218, 221, 224, 230 | `browser.pause(N)` time-based wait instead of predicate-driven `waitUntil` | ⚠ WARNING (per WR-05) | Canonical flaky-E2E anti-pattern. Mitigated by `mochaOpts.retries: 1`. |
| `hp41-gui/e2e/smoke.spec.js` | 96–243 | No `beforeEach` state reset between `it()` blocks | ⚠ WARNING (per WR-06) | Cross-test state leak risk — future regressions could be masked. |
| `hp41-gui/e2e/smoke.spec.js` | 84 | `String(err && err.message ? err.message : err)` mishandles falsy non-Error | ℹ INFO (per WR-07) | Edge case unlikely to trigger in practice. |

**Debt-marker scan:** No `TBD`, `FIXME`, `XXX` debt markers in files modified by this phase. (Verified via grep on modified files.)

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| hp41-core test suite passes | `cargo test -p hp41-core --tests` | 1627 passed, 0 failed | ✓ PASS |
| `math1_op_test_count` non-vacuous | `cargo test -p hp41-core --test math1_op_test_count` | 1 passed | ✓ PASS |
| `xrom_shadowing` non-vacuous | `cargo test -p hp41-core --test xrom_shadowing` | 2 passed | ✓ PASS |
| `lint_math1_assertions` gates pass | `cargo test -p hp41-core --test lint_math1_assertions` | 2 passed (no_decimal_assert_eq, no_manual_tolerance) | ✓ PASS |
| `math1_user_callback` ≥ 11 tests | `cargo test -p hp41-core --test math1_user_callback` | 11 passed (9 pre-existing + 2 new QUAL-08 categories) | ✓ PASS |
| `numerical_accuracy` suite passes | `cargo test -p hp41-core --test numerical_accuracy` | 41 passed (incl. `test_numerical_accuracy_suite`); internal 763/768 = 99.3 % | ✓ PASS |
| Free42 contamination guard | `bash scripts/check-free42-contamination.sh` | exit 0; "OK: no Free42 contamination detected" | ✓ PASS |
| All 13 math1/*.rs files carry disclaim header | `grep -L 'Free42 source consulted only as sanity-check oracle' hp41-core/src/ops/math1/*.rs` | (empty) | ✓ PASS |
| SC-4 invariant (strict grep) | `grep -rn "fn op_(add\|sub\|mul\|...)" hp41-gui/src-tauri/src/` | (no matches, exit 1) | ✓ PASS |
| `Justfile` ci recipe extended | `grep -E '^ci:' Justfile` | `ci: lint test coverage license-audit` | ✓ PASS |
| ci.yml license-audit parallel job | `grep -A 3 'license-audit:' .github/workflows/ci.yml` | Job present, `ubuntu-latest`, no `needs:` | ✓ PASS |
| **`hp41-core` coverage gate** | `cargo llvm-cov --package hp41-core --summary-only` | **91.74 % lines / 92.14 % regions** | ✗ **FAIL** |
| `hp41-gui/e2e/smoke.spec.js` 3 it() blocks | `grep -nE '^[[:space:]]+it\(' smoke.spec.js` | Lines 96, 147, 195 (3 blocks) | ✓ PASS |
| `just gui-e2e` runtime validation on Ubuntu | (Not run — requires xvfb + webkit2gtk-driver; deferred to CI on next push per 32-03-SUMMARY.md) | ? SKIP | ? UNCERTAIN — see Human Verification |

---

## Probe Execution

This phase does not declare conventional `scripts/*/tests/probe-*.sh` probes. The probe-equivalent checks are the `cargo test` gates above and `bash scripts/check-free42-contamination.sh`, all of which pass.

---

## Human Verification Required

### 1. WebdriverIO E2E smoke runs green on Ubuntu CI

**Test:** Push a commit to a PR branch and verify `ci-gui.yml::e2e-linux` runs all 3 `it()` blocks (existing `2 ENTER 3 + → 5.0000` + new `XEQ "SINH" 1 → 1.1752` + new `XEQ "MATRIX" 2x2 DET → -2.0000`) under the `mochaOpts.retries: 1` budget.

**Expected:** All 3 specs pass on `ubuntu-latest`; no flakes; total runtime ≤ ~60s. The two new tests should pass cleanly because they exercise verified code paths (`xrom_resolve`, `submit_modal`, column-major matrix iteration).

**Why human:** Plan 32-03 acknowledges in SUMMARY (32-03-SUMMARY.md "Quality Gates" section): "`just gui-e2e` was NOT run locally in this session (the worktree does not have webkit2gtk-driver / xvfb installed locally; CI runs the smoke on Ubuntu via `ci-gui.yml::e2e-linux`)." The E2E runtime is uniquely a CI-only validation surface for this phase.

### 2. v3.0.1 follow-up scope review

**Test:** Review the proposed v3.0.1 follow-up scope (in 32-03-SUMMARY.md `Rule 4 — Architectural Decision` section) and decide whether (a) targeted error-branch test pass on 7 below-90 % math1 files OR (b) ADR documenting acceptable per-file thresholds is the right path forward.

**Expected:** A decision on v3.0.1 scope; either a new milestone is created with the error-branch test pass plan, or an ADR is authored documenting why specific math1 files have lower coverage floors (e.g., Bairstow non-convergence branches are by-design POLY-07 reject paths).

**Why human:** Architectural choice — depends on whether the team wants to close the gap empirically (more tests) or via formal exception (ADR). Both are valid; both unblock the README hard-claim graduation.

### 3. CR-01 remediation decision (15 tautological cases)

**Test:** Decide whether to (a) replace the 15 `r.is_ok() || r.is_err()` tautologies in `numerical_accuracy.rs` with meaningful regression sentinels (per CR-01 fix recommendation), (b) delete them outright, or (c) accept the documented caveat and keep them as-is.

**Expected:** A decision on whether QUAL-02's 99.3 % gate quality is "good enough" with 15 inflated denominators, or whether to remediate before v3.0.1 ship.

**Why human:** Quality bar judgment — the gate passes either way, but the spirit of T-32-04 (meta-test-gaming threat model) is contradicted by the current cases. The team's own lint (`lint_math1_assertions.rs`) catches this class in `math1_*.rs` but explicitly excludes `numerical_accuracy.rs`.

---

## Gaps Summary

**One real gap, one intentional deferral, two documented caveats.**

1. **GAP — QUAL-01 / ROADMAP SC-1 NOT MET:** `hp41-core` coverage is 91.74 % lines / 92.14 % regions, below the 95 % / 93 % gate. Seven `ops/math1/*.rs` files are below the 90 % per-file floor. This is the **primary phase goal** and it is not achieved. The closure path is the v3.0.1 follow-up milestone (per Plan 32-03 Rule 4 — Architectural Decision), which must either (a) close the gap via targeted error-branch tests, or (b) document acceptable thresholds in an ADR.

2. **DEFERRED (intentional) — README v3.0 hard-claim graduation:** Per the gate-conditional discipline established for v2.2 HP-41CV, the README v3.0 line stays at soft-claim until QUAL-01 closes. This is the **correct** response to gap #1; the deferral is documented in CLAUDE.md (`### v3.0 additions` header reads `28–32 — README hard-claim DEFERRED pending coverage gate`), PROJECT.md (Active block notes coverage gap), and Plan 32-03 SUMMARY.

3. **CAVEAT — QUAL-02 partial (CR-01):** 15 of the 763 passing `numerical_accuracy.rs` cases are tautological `r.is_ok() || r.is_err()` placeholders that contribute zero regression signal but count toward the 98 % gate denominator. The gate passes at 99.3 % even excluding all 15 (748/753 = 99.34 %), so QUAL-02 is not silently failing — but the cases violate the T-32-04 meta-test-gaming threat model that this same phase introduced.

4. **CAVEAT — Code review warnings (WR-01 through WR-07):** 7 warning-level issues from 32-REVIEW.md, none of which fail current gates but all of which represent technical debt: missing `MATH1_DIR` existence check in contamination guard (WR-01), multi-line `assert_eq!` blind spot in lint (WR-02), substring-match in op-test-count heuristic (WR-03), 7 redundant sentinel cases (WR-04), 9 hard-coded `browser.pause()` calls (WR-05), no `beforeEach` state reset in E2E (WR-06), falsy error handling (WR-07).

**Frozen invariants preserved:** SC-4 verified (strict grep returns 0); MSRV 1.88 unchanged; `#![deny(clippy::unwrap_used)]` continues to apply; save-file backward compat preserved (no new `CalcState` fields); no `hp41-core/src/` source changes; no `hp41-gui/src-tauri/src/` source changes.

**Test infrastructure ships green:** All 1627 hp41-core tests pass, all 6 phase-specific gates pass, contamination guard exits 0, SC-4 clean, no debt markers. The Phase 32 *test/CI infrastructure* is fully delivered. Only the *coverage gate* — the primary phase goal per ROADMAP — remains unmet.

---

_Verified: 2026-05-18_
_Verifier: Claude (gsd-verifier)_
