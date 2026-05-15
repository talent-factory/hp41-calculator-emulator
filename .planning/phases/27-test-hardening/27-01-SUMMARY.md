---
phase: 27-test-hardening
plan: 01
subsystem: testing
tags: [coverage, tests, risk-weighted, atomic-gate-raise, numerical-accuracy, free42-citations]

# Dependency graph
requires:
  - phase: 20-core-math-and-conversions
    provides: Op::Pi / Op::Rnd / Op::Frc / Op::Fact / Op::Mod / Op::PolarToRect / Op::RectToPolar variants tested in accuracy suite extension
  - phase: 21-flags-display-control-and-sound
    provides: Op::SfFlag / Op::CfFlag / Op::FlagTest / display-control / sound ops tested in program-context coverage
  - phase: 22-program-control-memory-catalog-asn
    provides: Op::Stop / Op::Pse / Op::Size / Op::Cla / Op::Clst / Op::Pack / Op::Catalog / Op::Asn + SIZE-shrink stats guards (Pitfall 5)
  - phase: 23-alpha-register-operations
    provides: Op::Arcl / Op::Asto / Op::Atox / Op::Xtoa / Op::Arot / Op::Posa run-loop arms covered
  - phase: 24-indirect-addressing
    provides: 11-variant *Ind family run-loop / interactive coverage probes
provides:
  - FN-QUAL-01 closure: `just coverage` gate raised atomically from 80% to 95% on hp41-core
  - FN-QUAL-02 closure: numerical_accuracy.rs extended 503 → 566 cases with v2.2 math ops + Free42/OM citations per D-27.7
  - 4 new test files closing Priorities 1, 2, 3, 5, 6 of RESEARCH §Risk-Weighted Uncovered-Line Inventory
  - 1 extended test file: hand-curated v2.2 math accuracy cases with baseline non-regression assertion
  - Phase 27 settled-architecture block in CLAUDE.md
affects: [27-02-proptest-suites, 27-03-ind-integration, 27-04-e2e-and-vitest-ci, README v2.2 hard-claim upgrade]

# Tech tracking
tech-stack:
  added: []  # Test-only phase — no new runtime deps; existing hp41-core test infrastructure reused
  patterns:
    - "// Catches: <bug class> rationale doc-comment per test (D-27.1 — every new test names the bug class it guards)"
    - "Cross-checked against Free42 / HP-41C Owner's Manual cite-trail per quirky hand-curated case (D-27.7)"
    - "run_program-driven program-context coverage tests via build_single_op_program + run_op_in_program helpers"
    - "Snapshot-based SIZE-shrink sentinel pattern: capture (regs.len, X, lift_enabled), dispatch failing op, assert unchanged"
    - "Baseline non-regression assertion alongside combined ≥98% gate (D-27.6 dual-floor pattern)"
    - "Atomic gate-and-test ratchet: justfile --fail-under-lines change in SAME commit as final test additions (D-27.2)"

key-files:
  created:
    - hp41-core/tests/program_execution_coverage.rs (Priority-1, 42 #[test]s, 603 lines)
    - hp41-core/tests/phase22_stats_size_shrink.rs (Priority-2, 14 #[test]s, ~220 lines)
    - hp41-core/tests/phase21_phase22_interactive_no_ops.rs (Priority-3, 12 #[test]s, ~230 lines)
    - hp41-core/tests/format_eng_edges.rs (Priorities 4/5/6, 24 #[test]s including 9 dispatch-arm Priority-4 sentinels, ~280 lines)
    - .planning/phases/27-test-hardening/27-01-SUMMARY.md (this file)
  modified:
    - hp41-core/tests/numerical_accuracy.rs (503 → 566 cases via v2.2 hand-curated extension; baseline non-regression floor 498 asserted per D-27.6)
    - justfile (`coverage:` recipe: `--fail-under-lines 80` → `95`, atomic per D-27.2)
    - CLAUDE.md (Quality Gates table extended with v2.2 column; new "v2.2 additions (Test Hardening, Phase 27)" settled-architecture block)

key-decisions:
  - "Branch A applied (D-27.3): achieved coverage 95.25 % lines ≥ 95.0 % gate; no D-27.3 ceiling-fallback documentation needed"
  - "Priority-4 (interactive-dispatch arms) added to format_eng_edges.rs as a small supplementary section after Priority-3 was found insufficient on its own to push past the 95 % gate. Added 9 tests covering Op::Int, Op::StoArithStack, Op::AlphaBackspace, Op::Lbl, Op::Gto, Op::Rtn, Op::Test, Op::Isg, Op::Dse interactive arms in ops/mod.rs"
  - "D-27.6 baseline-floor adjustment: the plan's 'existing 500-case baseline still passes at 500/500' was aspirational — the actual baseline is 498/503 (the original `passes >= 493` floor reflected 5 pre-existing HP-41 hardware-rounding divergences that are part of the historical failure budget). The new assertion preserves the actual existing floor (`baseline_passes >= 498`), NOT an inflated zero-failure floor that would have falsely failed"
  - "MOD divide-by-zero error variant: spec mentioned `HpError::DivideByZero`; actual implementation returns `HpError::Domain`. Test asserts `Domain` (matches code; Free42 returns ERR_DIVIDE_BY_0 — emulator classifies indistinguishably as Domain)"

patterns-established:
  - "Risk-weighted Priority N test files: one file per priority cluster (Priority 1 = program-context; Priority 2 = SIZE-shrink sentinels; Priority 3 = interactive no-ops; Priority 5+6 = math/format edges + dispatch-arm coverage)"
  - "Free42 citation format: `// Cross-checked against Free42 source <file>::<fn>` + `HP-41C Owner's Manual p.<page>` immediately preceding the case! invocation or assertion"
  - "Combined-gate `div_ceil` ceiling threshold: `let threshold = (total * 98).div_ceil(100);` for ≥98% pass-rate gate"

requirements-completed: [FN-QUAL-01, FN-QUAL-02]

# Metrics
duration: ~75min
completed: 2026-05-15
---

# Phase 27 Plan 01: Coverage Push + Atomic 80 → 95 Gate Raise + Accuracy Suite Extension Summary

**Risk-weighted hp41-core coverage uplift 93.59 % → 95.25 % lines, with atomic 80 → 95 gate ratchet and v2.2 numerical-accuracy extension (566 cases, 27 Free42/OM citations).**

## Performance

- **Duration:** ~75 min (4 sequential tasks executed; final commit is atomic Task 3+4 per D-27.2)
- **Started:** 2026-05-15
- **Completed:** 2026-05-15
- **Tasks:** 4 (Tasks 3+4 land in a single atomic commit per D-27.2)
- **Files modified:** 7 (4 new + 1 extended + justfile + CLAUDE.md)

## Accomplishments

- **FN-QUAL-01 closed:** `just coverage` now enforces ≥ 95.0 % line coverage on hp41-core. Achieved 95.25 % lines / 93.75 % regions (vs 93.59 % / 91.21 % baseline measured 2026-05-15 by RESEARCH). Branch A per D-27.3 ceiling decision tree — no defensible-skip-line documentation needed.
- **FN-QUAL-02 closed:** numerical_accuracy.rs grew from 503 to 566 hand-curated cases adding Pi, Fact, Mod, Rnd, Frc, P→R, R→P coverage per D-27.5. 99.1 % combined pass rate (561/566). 27 Free42 / OM citations (≥ 15 floor per D-27.7).
- **D-27.2 atomicity preserved:** the justfile gate raise lands in the SAME commit (`584f7b2`) as the final test additions per the audit trail `git diff HEAD~1 HEAD -- justfile`.
- **D-27.6 dual-floor gate:** v1.x 503-case baseline floor 498 asserted independently of the combined ≥ 98 % gate.
- **D-27.1 rationale:** 80+ `// Catches: <bug class>` comments across the 4 new test files; 27 Free42/OM citations in numerical_accuracy.rs.
- **SC-4 invariant preserved:** zero source changes to hp41-core/src/ or hp41-gui/src-tauri/. MSRV 1.88 unchanged. `#![deny(clippy::unwrap_used)]` continues to apply (new test files carry `#![allow]` at file scope).

## Coverage Trajectory

| Stage                                              | Lines    | Regions  | Notes                                                                                |
| -------------------------------------------------- | -------- | -------- | ------------------------------------------------------------------------------------ |
| Pre-Task-1 baseline (2026-05-15)                   | 93.59 %  | 91.21 %  | Per RESEARCH; matches independent measurement at start of Task 1                     |
| Post-Task-1 (program_execution_coverage.rs)        | n/m      | n/m      | Not measured intermediate; combined post-Task-3 below                                |
| Post-Task-3 (4 test files landed, gate still at 80)| 94.96 %  | 93.30 %  | Just below 95.0 % — triggered Priority-4 supplement                                  |
| Post-Priority-4 supplement                         | **95.25 %** | **93.75 %** | Branch A: gate at 95 PASSES                                                          |

**Biggest single-file uplifts:**

- `ops/stats.rs`: 94.05 % → 100.00 % lines (Pitfall-5 SIZE-shrink + denom-zero / n=0 sentinels covered ALL missed lines)
- `format.rs`: 79.77 % → 97.11 % lines (ENG carry + SCI/ENG zero-mode + round_to_display_precision direct calls)
- `ops/mod.rs`: 91.50 % → 95.10 % lines (interactive-no-op sentinels + Priority-4 dispatch arms)
- `ops/program.rs`: 89.58 % lines (unchanged — region coverage 85.89 % → 88.46 % via run_loop probes)

## Task Commits

Each task was committed atomically per D-27.2 (Task 3+4 combined into a single commit for gate-and-test atomicity):

1. **Task 1: Pre-implementation baseline + Priority-1 program-execution coverage** — `bafbef0` (test)
2. **Task 2: Priority-2 stats SIZE-shrink guards + Priority-3 interactive no-op arms** — `528feb3` (test)
3. **Task 3+4 (atomic per D-27.2): format edges + v2.2 accuracy extension + 80→95 gate raise + CLAUDE.md update** — `584f7b2` (test)

D-27.2 atomicity audit: `git diff HEAD~1 HEAD --name-only` shows CLAUDE.md, Justfile, format_eng_edges.rs, numerical_accuracy.rs all in commit `584f7b2`.

## Files Created/Modified

- `hp41-core/tests/program_execution_coverage.rs` (NEW) — 42 #[test]s, Priority-1: every Phase 20–24 op exercised in `run_program` context via `build_single_op_program` + `run_op_in_program` helpers. Catches divergence between interactive `dispatch()` and program-context `execute_op()` (lift effects, display_override leakage, event_buffer side effects, pc advancement). Pitfall-3 (PSE survival) specifically asserted.
- `hp41-core/tests/phase22_stats_size_shrink.rs` (NEW) — 14 #[test]s, Priority-2: every Σ-op SIZE-shrink guard (Σ+, Σ−, MEAN, SDEV, LR, YHAT, CORR, CLΣ) + denom-zero / n=0 / n<2 guards. `Snapshot::assert_unchanged` confirms fail-closed semantics preserve state.
- `hp41-core/tests/phase21_phase22_interactive_no_ops.rs` (NEW) — 12 #[test]s, Priority-3: design invariant locking for Op::FlagTest×4, Op::Stop, Op::Pse (with documented side effect), Op::Prompt, Op::GtoInd/XeqInd (return InvalidOp), Op::IsgInd/DseInd (discard skip-bool interactively), Op::FlagTestInd.
- `hp41-core/tests/format_eng_edges.rs` (NEW) — 24 #[test]s, Priorities 5/6 + Priority-4 supplement: SCI/ENG zero-mode boundaries, ENG carry threshold (`999.9995 → 1.000E 03`), `round_to_display_precision` direct calls, Op::Rnd integration, op_abs ± symmetric pair, plus 9 Priority-4 interactive dispatch-arm sentinels (Op::Int, Op::StoArithStack, Op::AlphaBackspace, Op::Lbl, Op::Gto, Op::Rtn, Op::Test, Op::Isg, Op::Dse).
- `hp41-core/tests/numerical_accuracy.rs` (EXTENDED) — 503 → 566 cases via the v2.2 extension block (Pi×3, Fact×11, Mod×12, Rnd×9, Frc×8, P→R×10, R→P×10) + 4 error-path #[test]s (Fact 70 OutOfRange, Fact 3.5 Domain, Fact -3 Domain, Mod _,0 Domain). 27 Free42/OM citations. Baseline-non-regression: `baseline_passes >= 498` per D-27.6.
- `justfile` (EDITED) — `coverage:` recipe `--fail-under-lines 80` → `95`. Comment updated. `cargo llvm-cov clean --workspace` line preserved (ff39017 worktree-fix unchanged).
- `CLAUDE.md` (EDITED) — Quality Gates table extended with v2.2 (Phase 27) column showing 95.25 % / 93.75 %; Tech Stack `cargo-llvm-cov` line updated to ≥ 95 %; new "v2.2 additions (Test Hardening, Phase 27)" settled-architecture block (4 bullets: gate raise, accuracy extension, frozen invariants, // Catches: rationale).

## Decisions Made

1. **Branch A (D-27.3) applied:** the risk-weighted Priority-1..6 push reached 94.96 % after the planned 4 test files; a small Priority-4 supplement (9 tests, ~80 lines) lifted to 95.25 % without padding. No defensible-skip-line documentation required.
2. **Priority-4 supplement placement:** added to `format_eng_edges.rs` (rather than a 5th test file) to keep the 4-task plan structure and avoid further increasing file count beyond what 27-PLAN-CHECK already flagged as borderline. The supplement targets the most-used central integration hub (ops/mod.rs::dispatch) with the cheapest bug-catching value (dispatch wiring regressions).
3. **D-27.6 baseline-floor calibration:** the plan's "500/500 baseline" claim was aspirational. Empirical baseline is 498/503 (5 documented HP-41 hardware-rounding divergences acceptable per the historical `passes >= 493` failure budget). The new `baseline_passes >= 498` assertion preserves the ACTUAL existing floor — any drop signals a real regression, not a tightening of an arbitrary threshold the plan never met.
4. **MOD divide-by-zero error variant:** plan referenced `HpError::DivideByZero`; the actual `op_mod` returns `HpError::Domain` (math.rs:496-497). Test asserts `Domain` to match implementation. Free42 returns `ERR_DIVIDE_BY_0`; we classify this case as `Domain` in the HpError taxonomy (functionally indistinguishable to the user).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Baseline assertion calibrated to actual empirical floor**
- **Found during:** Task 3 (numerical_accuracy.rs extension)
- **Issue:** Plan called for asserting "v1.x baseline still passes at 500/500" but the actual pre-extension baseline is 498/503 (5 documented HP-41 hardware-rounding divergences in trig / log / HMS that have been part of the historical `passes >= 493` failure budget since the suite was built).
- **Fix:** Asserted `baseline_passes >= 498` instead of `baseline_passes == 503` (or `== 500`). The new floor preserves the actual existing pass count; any drop below it is a genuine regression. Documented in the assertion message and in the SUMMARY decision log.
- **Files modified:** hp41-core/tests/numerical_accuracy.rs
- **Verification:** `cargo test -p hp41-core --test numerical_accuracy` passes; combined gate 561/566 ≥ 98 %.
- **Committed in:** `584f7b2` (atomic Task 3+4 commit)

**2. [Rule 2 - Missing Critical] Priority-4 supplement (interactive-dispatch arms)**
- **Found during:** Task 3 mid-execution (post-format_eng_edges first pass)
- **Issue:** After landing Priorities 1, 2, 3, 5, 6 the coverage was 94.96 % — just below the 95 % gate D-27.2 mandates landing atomically with this final commit. Without supplemental tests, Branch B (D-27.3 ceiling fallback) would have been required, but the gap was 0.04 % — clearly fillable via genuine bug-catching tests, NOT padding.
- **Fix:** Added 9 small interactive-dispatch arm sentinels in `format_eng_edges.rs` (Op::Int, Op::StoArithStack, Op::AlphaBackspace, Op::Lbl, Op::Gto, Op::Rtn, Op::Test, Op::Isg, Op::Dse). These are Priority-4 per RESEARCH §Risk-Weighted Inventory (mod.rs dispatch wiring regressions) — they catch bugs in the most-used central integration hub. Each carries `// Catches: dispatch wiring regression on Op::X — mod.rs:LLL` per D-27.1.
- **Files modified:** hp41-core/tests/format_eng_edges.rs
- **Verification:** `cargo test -p hp41-core --test format_eng_edges` passes 24 tests; `just coverage` reports 95.25 % lines — Branch A applies.
- **Committed in:** `584f7b2` (atomic Task 3+4 commit)

**3. [Rule 3 - Blocking] `manual_div_ceil` clippy lint on threshold computation**
- **Found during:** Task 3 numerical_accuracy.rs gate update
- **Issue:** Initial implementation used `(total * 98 + 99) / 100` for ceiling division; clippy `-D warnings` rejected as `clippy::manual_div_ceil` (suggested `.div_ceil(100)`).
- **Fix:** Replaced with `(total * 98).div_ceil(100)`.
- **Files modified:** hp41-core/tests/numerical_accuracy.rs
- **Verification:** `cargo clippy -p hp41-core --tests -- -D warnings` clean.
- **Committed in:** `584f7b2`

---

**Total deviations:** 3 auto-fixed (1 bug calibration, 1 missing-critical supplement, 1 blocking lint).
**Impact on plan:** No scope creep. The Priority-4 supplement is in-scope per D-27.1 (catches real bugs, not padding) and per D-27.3 (preferred over Branch B fallback when the realistic ceiling is reachable via bug-catching tests). The baseline-floor calibration is a Rule-1 bug fix: asserting `== 503` against an empirical 498 would have been a false claim, not a tighter check.

## Issues Encountered

- **Filesystem case-sensitivity:** the justfile is tracked as `Justfile` (capital J) but the filesystem case-insensitively maps `justfile` ↔ `Justfile`. `git add justfile` showed staged but `git diff --cached justfile` was empty until restaged as `git add Justfile`. Resolved without affecting plan execution.
- **`Decimal::from_str_exact` usage:** used in `phase21_phase22_interactive_no_ops.rs` to avoid the rounding behavior of `Decimal::from_str` for counter values like `"5.005"`. Worked first try; no version-compat issues.

## Self-Check: PASSED

**Test counts (per task verification):**
- `program_execution_coverage.rs`: 42 #[test]s, 42 `// Catches:` comments, all pass.
- `phase22_stats_size_shrink.rs`: 14 #[test]s, 14 `// Catches:`, all pass.
- `phase21_phase22_interactive_no_ops.rs`: 12 #[test]s, 12 `// Catches:`, all pass.
- `format_eng_edges.rs`: 24 #[test]s, 24 `// Catches:`, all pass.
- `numerical_accuracy.rs`: 5 #[test]s (1 main suite + 4 error-path tests), 27 Free42/OM citations, suite gate passes (561/566 combined, baseline floor 498).

**Final invariants verified:**
- `cargo test -p hp41-core` → 869 tests pass (38 suites).
- `cargo clippy -p hp41-core --tests -- -D warnings` → clean.
- `just coverage` exit 0 with `--fail-under-lines 95`, reports TOTAL 95.25 % lines / 93.75 % regions.
- `git diff HEAD~1 HEAD -- justfile` shows the `--fail-under-lines 80 → 95` change is in commit `584f7b2`, the SAME commit as `hp41-core/tests/numerical_accuracy.rs` (D-27.2 atomicity audit).
- `grep -c "// Catches:" hp41-core/tests/{program_execution_coverage,phase22_stats_size_shrink,phase21_phase22_interactive_no_ops,format_eng_edges}.rs` → 42 + 14 + 12 + 24 = 92 total `// Catches:` comments (≥ 80 floor).
- `grep -c "Cross-checked against Free42\|HP-41C Owner.s Manual" hp41-core/tests/numerical_accuracy.rs` → 27 (≥ 15 floor per D-27.7).
- No source changes to `hp41-core/src/` (`git diff HEAD~3 HEAD -- hp41-core/src/` is empty).
- No source changes to `hp41-gui/src-tauri/` (`git diff HEAD~3 HEAD -- hp41-gui/` is empty).

## Threat Flags

None — Phase 27 Plan 01 is test-only. No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Next Phase Readiness

- **Plan 27-02 (proptest suites)** is unblocked: this plan landed the hand-curated half of the D-27.5 hybrid; Plan 02 lands the shape-property complement (`proptest_flags.rs`, `proptest_math.rs`) without rework or coupling to this plan's case set.
- **Plan 27-03 (IND integration)** is unblocked: the Phase 24 `*Ind` family run-loop arms have been verified to compile and route through `execute_op` correctly via the 8 Phase-24 IND tests in `program_execution_coverage.rs`. Plan 03 ships the dedicated `indirect_addressing.rs` happy/sad-path suite.
- **Plan 27-04 (E2E + Vitest CI)** is unblocked: hp41-gui changes are independent of this plan. The new coverage gate at 95 % stays inside `ci.yml` only (Plan 04 changes are confined to `ci-gui.yml` per D-27.4).
- **README v2.2 hard-claim upgrade (Phase 25 D-25.17 follow-up):** the FN-QUAL-01 ≥ 95 % gate is now closed. After Phase 27 verification passes (post-Plans 02/03/04), the README soft-claim can be upgraded.

---
*Phase: 27-test-hardening*
*Completed: 2026-05-15*
