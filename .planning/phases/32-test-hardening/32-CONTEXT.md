# Phase 32: Test Hardening — Context

**Gathered:** 2026-05-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 32 closes v3.0 by locking the quality gates — the milestone's final ship. Five concrete deliverables across 3 plans:

1. **Coverage gate held at v2.2 level.** `just coverage` reports `hp41-core` line coverage ≥ 95.0 % / regions ≥ 93.0 %. NO atomic raise this milestone per D-27.2 lessons-learned. The 13 `hp41-core/src/ops/math1/*.rs` source files each reach ≥ 90 % via the 12 existing `tests/math1_*.rs` files plus targeted gap-closure cases where surfaced.
2. **`numerical_accuracy.rs` extension from 566 → ~700+ cases** with risk-weighted distribution across the 11 Math Pac I families. Each new case carries an OM page-and-example citation in a `// Source: HP 00041-90034 p.<n>, ex.<m>` doc comment per the D-27.7 pattern (mirrors the 27 v2.2 citations). `approx 0.5.1` lands as the only new dev-dep; `assert_relative_eq!(actual, expected, max_relative = 1e-7)` is the default tolerance discipline (Math Pac I floor; Pitfall 14 cross-platform drift mitigation).
3. **Per-Op test count ≥ 5 enforcement** (`tests/math1_op_test_count.rs` — currently vacuous from Plan 28-01) graduates to non-vacuous. The gate iterates every Phase-28 `Op` variant in `ops/mod.rs` and counts `#[test]` mentions across `tests/math1_*.rs`; assertion fails if any variant has < 5. Pitfall 16 mitigation against mid-milestone coverage drop.
4. **E2E smoke spec extension** in `hp41-gui/e2e/smoke.spec.js` — adds two new test functions: `sinh(1)→1.1752` (XEQ-by-name resolution path) AND `MATRIX 2x2 DET→-2.0000` (modal pipeline path). Both ship as a deliberate expansion of literal ROADMAP scope per D-32.4. Runs ONLY on Ubuntu in `e2e-linux` job (matches D-27.15 AMENDED). The existing `2 ENTER 3 +` smoke test stays unchanged. File extension stays `.js` per D-32.1 (ROADMAP `.ts` is a noop drift).
5. **Free42 GPL-contamination guard ships.** `scripts/check-free42-contamination.sh` runs in BOTH `just ci` AND a separate visible `ci.yml` step ("License audit"). The 12-symbol tight-allowlist grep policy (D-32.7) catches realistic copy-paste threats while allowlisting the disclaim header line every math1 file already carries. The README v3.0 line graduates from soft-claim to the OM-cited hard claim ("v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034") in the final ship commit (D-32.5/D-32.6), with parallel PROJECT.md and CLAUDE.md `### v3.0 additions` block updates.

**In scope:**
- `hp41-core/tests/` extensions: `numerical_accuracy.rs` 566 → ~700+ cases; `math1_op_test_count.rs` graduates from vacuous to non-vacuous; new `tests/lint_math1_assertions.rs` lint
- `hp41-core/Cargo.toml` `[dev-dependencies]` gains `approx = "0.5.1"` (only new dep; no runtime deps added)
- `scripts/check-free42-contamination.sh` (new — ~30 lines of shell with 12-symbol grep + allowlist)
- `.github/workflows/ci.yml` gains a visible "License audit" step running the script
- `justfile` `ci` recipe gains a line invoking the script
- `hp41-gui/e2e/smoke.spec.js` gains two new test functions inside the existing describe block
- README hard-claim graduation (final ship commit)
- PROJECT.md "Active" line update + `v3.0 additions` Phase 32 subsection
- CLAUDE.md `### v3.0 additions` Phase 32 subsection (final population, replacing `(in progress)` stub from Phase 30)

**Out of scope (explicit):**
- Any `hp41-core/src/` or `hp41-cli/src/` or `hp41-gui/src-tauri/src/` source changes — Phase 32 is test/CI/docs only. Per-Op tests in `tests/math1_*.rs` already shipped across Plans 28-02..28-10.
- Any `hp41-gui/src/` source changes — Phase 31 shipped the GUI surface complete
- Coverage gate raise above 95% — explicitly deferred per D-27.2 lessons-learned (raise requires risk-weighted tests, not coverage padding)
- Conversion of `smoke.spec.js` to `.ts` (D-32.1 — noop drift; future test legitimately needing types can drive)
- New ADRs — D-32.5/D-32.6/D-32.7 are catalogued in `docs/hp41-math1-divergences.md` as Phase 32 behavioral policies if needed; no new `docs/adr/v3.0-NNN-*.md` files this phase
- Stat 1 / Time / Advantage pacs — v3.1+
- HP-copyrighted ROM-image redistribution — permanent exclusion
- Cross-platform CI matrix expansion (adding ARM macOS / Linux aarch64 jobs) — deferred; current 3-OS matrix + 1e-7 tolerance discipline is sufficient

**Mandated by ROADMAP cross-cutting constraints (lines 35–45 of `.planning/ROADMAP.md`):**
- **SC-4 invariant**: stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` must return nothing. Phase 32 touches `hp41-core/tests/`, `hp41-gui/e2e/`, `scripts/`, `.github/`, `justfile`, root `*.md` only — SC-4 trivially preserved.
- **`#![deny(clippy::unwrap_used)]`** continues to apply in `hp41-core`. New test files carry `#![allow(clippy::unwrap_used)]` at file scope per the established Phase 1+ pattern.
- **4-exhaustive-match invariant**: no new `Op` variants in Phase 32. The `prgm_display.rs` copies and `dispatch()` / `execute_op()` are untouched.
- **CLI ↔ GUI parity** (D-25.6): Phase 32 doesn't alter the surface; trivially preserved.
- **Save-file backward compat**: no new `CalcState` fields. v1.0–v2.2 + v3.0 save files all continue to load.
- **MSRV 1.88 unchanged.** `approx 0.5.1` MSRV is 1.0+; trivially compatible. Zero new runtime deps.
- **No HP-copyrighted ROM bytes** — Phase 32 actively guards against contamination via the new CI script.

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / STATE.md / 27-CONTEXT.md / 28–31-CONTEXT.md (carried forward — NOT re-decided here)

- **D-27.2 (lessons-learned):** Coverage gate held at v2.2 level (≥ 95 % lines / ≥ 93 % regions). Atomic raises require risk-weighted tests, not coverage padding. Phase 32 explicitly does NOT raise the gate per this discipline.
- **D-27.3:** Per-Op test count ≥ 5 (Pitfall 16) — risk-weighted, NOT coverage padding. Documented `// Catches: <bug class>` comments on each new test per D-27.1.
- **D-27.5/D-27.7:** Numerical accuracy cases carry OM page-and-example citations per case. 27 v2.2 citations are the precedent; Phase 32 adds ~134 more across 11 Math Pac I families.
- **D-27.6:** `baseline_passes >= 498` floor on the v1.x 503-case subset is independently asserted alongside the combined ≥ 98 % gate. Phase 32 preserves the assertion.
- **D-27.15 AMENDED:** WebdriverIO + tauri-driver on Ubuntu only (`e2e-linux` job in `ci-gui.yml`). macOS/Windows matrix jobs unchanged. Phase 32 extends the spec but not the matrix.
- **D-28.6:** Math Pac I uses XEQ-by-name only — no dedicated key bindings. The sinh(1) E2E case clicks `XEQ S I N H Enter 1 Enter` (alpha-collection path).
- **D-29.1:** `docs/hp41-math1-functions.json` was authored in Phase 29 (pulled forward from Phase 30 / DOC-01). Phase 32's parity tests continue to consume it through the merged `help_entries_all()` accessor.
- **D-30.9:** Hard claim deferred to Phase 32 conditional on coverage gate ≥ 95 %. This phase's D-32.5 graduates the soft-claim.
- **ADR-001 / ADR-002 / ADR-005:** all three locked across Phase 28/30. Phase 32 cites ADR-002 in the disclaim header (already on every math1 file).
- **Pitfall 14:** Cross-platform drift between x86 and ARM. Math Pac I floor is 6 of HP-41's 10 digits → `max_relative = 1e-7` is the default tolerance; deliberate exceptions documented per-case.
- **Pitfall 16:** Per-Op test count ≥ 5 prevents mid-milestone coverage drop. `math1_op_test_count.rs` graduates from vacuous to non-vacuous in Plan 32-01.
- **Pitfall 17:** No `assert_eq!(decimal, decimal)` on iterated results. `lint_math1_assertions.rs` enforces.
- **Pitfall 19:** Free42 GPL contamination. Per-file disclaim header already in place on all 13 math1/*.rs files (confirmed by codebase scout). Phase 32's `scripts/check-free42-contamination.sh` is the CI tripwire.

### Discussed and decided in this session (D-32.1 — D-32.10)

#### E2E smoke workflow choice

- **D-32.1: smoke.spec.js retained — ROADMAP .ts wording is a noop drift.** The file landed in Phase 27 as `.js` and works correctly through `wdio.conf.cjs` (Mocha runner, retries 1). Converting to `.ts` requires `@wdio/typescript-support` (new devDep), a tsconfig that doesn't conflict with `hp41-gui/tsconfig.json`, and re-validating the entire Ubuntu CI path (`webkit2gtk-driver` + `xvfb` apt deps + cargo-bin cache). Zero functional benefit — the smoke uses raw WebDriver selectors `[data-key-id]` and `[data-testid]`, no IPC type imports to benefit from TS. Future conversion is non-breaking and lands when a test legitimately needs strong typing.
  - **Why:** smallest blast radius; matches D-27.15 AMENDED spirit-vs-letter precedent (the ROADMAP wording can drift from disk reality without functional impact).

- **D-32.2: Two new test functions land — sinh(1) AND MATRIX 2x2 DET.** Both as separate `it()` blocks inside the existing describe block in `smoke.spec.js`. The sinh(1) case (`XEQ S I N H Enter 1 Enter` → LCD reads `1.1752`) exercises XEQ-by-name resolution, alpha-collection, `xrom_resolve`, and display update. The MATRIX DET case (`XEQ M A T R I X Enter 2 R/S 1 R/S 2 R/S 3 R/S 4 R/S XEQ D E T Enter` → LCD reads `-2.0000`) exercises modal_program lifecycle, ORDER=? prompt routing, R/S submit semantics, and the cross-section integration of Phases 28-31. Deliberate expansion of literal ROADMAP "OR" scope.
  - **Why:** the two paths exercise structurally distinct surfaces. The MATRIX flow is the single best regression-catcher per second of CI runtime (touches the modal pipeline); sinh(1) is the cheapest possible XEQ-by-name proof. Both run on `e2e-linux` Ubuntu only; total added CI time ~40s.

- **D-32.3: MATRIX DET test does NOT add an Esc-cancel verification step at the end.** The natural workflow completion (modal_program clears when DET fires) is sufficient. Esc-cancel is exercised by Vitest unit tests in `hp41-gui/src/`.
  - **Why:** smaller test; lower CI runtime; cancel semantics are tested more deeply at the unit-test layer; D-27.13 ("the SOLE E2E spec" precedent) bounds E2E scope.

- **D-32.4: Existing `2 ENTER 3 +` smoke test unchanged.** Phase 27 FN-QUAL-05 literal ROADMAP scope preserved bit-for-bit. The two new tests are additions, not replacements.

#### README hard-claim graduation

- **D-32.5: README v3.0 line graduates from soft-claim to hard claim.** Final ship commit (after all 3 plans complete and `just coverage` confirms ≥ 95 % line coverage) replaces the current under-`## Features` line:
  ```
  - Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry
    points, documented divergences)
  ```
  with the OM-cited hard claim:
  ```
  - v3.0 ships Math Pac I behavioral emulation, feature-complete per
    Owner's Manual 00041-90034
  ```
  The "documented divergences" reference moves into a follow-up line linking `docs/hp41-math1-divergences.md`. Rejected the conservative "stay soft-claim" option because Phase 32 IS the gate-verification moment per D-30.9. Rejected the buffer-milestone option because deferring the claim to v3.0.1 would split the narrative across milestones.
  - **Why:** the OM citation is the authoritative spec; grounding the hard claim in HP 00041-90034 makes it the strongest possible statement. The behavioral-emulation scope (not bit-exact) is implicit in "per Owner's Manual" — readers who know HP-41 culture parse this correctly; the divergence link covers the rest.

- **D-32.6: Final ship commit is gate-verified.** The final commit message embeds the output of `just coverage` confirming ≥ 95 % line coverage. Parallel updates to PROJECT.md and CLAUDE.md `### v3.0 additions` block land in the same commit. Rejected the Plan-32-03-closing option because that plan is the documentation-adjacent plan and would conflate the gate-verification with the E2E + Free42 work. Rejected the separate-doc-only-commit option because separating reduces ceremony at the cost of a less-obvious provenance link.
  - **Why:** strongest provenance link between gate and claim. A reviewer reading the commit can see both the test results and the README change in one diff. Mirrors v2.2 ship pattern where the v2.2 tag landed with the coverage-gate-raised commit.

#### Free42 contamination guard

- **D-32.7: Tight 12-symbol grep policy with allowlist.** `scripts/check-free42-contamination.sh` greps for the following distinctive Free42 identifiers + copyright/license markers across `hp41-core/src/ops/math1/`:
  ```bash
  grep -rn -E 'phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License' \
      hp41-core/src/ops/math1/ \
      | grep -v 'Free42 source consulted only as sanity-check oracle'
  ```
  The 12 distinctive symbols come from Free42's Intel BID decimal library, the decNumber library, and Free42 internal types (`vartype`, `arg_struct`, `prgm_lines`, `bcd_t`). The allowlist preserves the single legitimate mention of "Free42" in the disclaim header on every math1/*.rs file. Rejected Option B (aggressive 24+ symbols + function-name patterns) because our own naming (`op_c_plus`, `op_c_minus`, `parse_counter`) is close enough that drift would force false-positive triage. Rejected Option C (minimal 4 symbols) because it misses `decNumber`/`decContext` types and GPL/AGPL copyright markers. Rejected Option D (headers-only) because it's a hygiene gate misnamed.
  - **Why:** hits the realistic threat model — a well-intentioned contributor copy-pasting a Free42 algorithm to verify behavior. Distinctive identifiers survive a quick paste. Allowlist preserves our own disclaim. ~10 lines of shell; low maintenance; composes with ADR-002 + PR review.

- **D-32.8: Guard runs in BOTH `just ci` AND a separate `ci.yml` step.** `just ci` invokes the script during local development for fast feedback. A separate "License audit" step in `.github/workflows/ci.yml` makes the gate's purpose self-describing in the GitHub PR checks panel. Mirrors the `scripts/check-tauri-permissions.sh` pattern from v2.0. Both invocations point at the same script — functionally identical, cosmetically clearer audit trail.
  - **Why:** belt + suspenders. The local-dev path catches contamination before push; the CI path makes the gate visible to reviewers.

#### Numerical-accuracy case-selection priorities

- **D-32.9: Risk-weighted case distribution across 11 Math Pac I families.** ~134 new cases distribute as:
  - POLY ~25 (multiplicity-as-cluster per Pitfall 5; degree 2-5 OM examples)
  - CMPLX ~20 (Euler identities, `Cinv`/`LnZ` (0,0) boundary per Pitfall 6, exp/ln/sin/cos round-trips)
  - MAT ~18 (DET via partial-pivot LU, SIMEQ with EPSILON threshold per ADR-003, small-order 2×2/3×3/4×4 cases)
  - INTG ~15 (Simpson convergence per ADR-004; threshold = 10^(-decimals-1) tied to DisplayMode)
  - SOLVE ~15 (modified secant; multiple-roots → guess-dependent selection; non-convergence path)
  - DIFEQ ~12 (RK4 stability for stiff systems; 1st-order + 2nd-order ODE reduction)
  - HYP ~10 (domain guards: `Acosh(x<1)`, `Atanh(|x|≥1)`; identities: `cosh²-sinh²=1`)
  - TRI ~8 (SSA ambiguous case = 0/1/2 solutions; SSS/ASA/SAA/SAS single-solution sanity)
  - FOUR ~6 (orthogonality; rect↔polar conversion)
  - TRANS ~3 (2D/3D translate-rotate round-trip identity)
  - REAL ~2 (`XEQ "REAL"` flag-flip only — minimal coverage sufficient)

  Total: ~134 cases. Combined with v2.2's 566 baseline → ~700 total. Each case carries a `// Source: HP 00041-90034 p.<n>, ex.<m>` doc comment per D-27.7 (where the OM citation exists; emulator-extension cases like `XEQ "REAL"` carry `// Source: D-28.3 emulator extension`).
  - **Why:** mirrors D-27.3 risk-weighted Phase 27 pattern. The weighting reflects bug-class density surfaced in Phase 28-31 research: POLY's multiplicity-as-cluster (Pitfall 5) and CMPLX's `(0,0)` boundary (Pitfall 6) are the most likely to surface algorithm drift. TRANS/REAL get small allocations because their failure modes are simpler.

- **D-32.10: POLY multiplicity-as-cluster assertion strictness — centroid within 1e-4 + max-imag < 1e-3.** For `(x-1)^5`: assert `mean(roots).re ≈ 1.0 ± 1e-4` AND `max(|roots[i].imag|) < 1e-3`. Hardware-faithful — the OM 1979 example output shows the cluster spread; per-root tolerance would risk false negatives, bit-exact would risk x86/ARM FPU drift (Pitfall 14). Deliberate exception to the default `max_relative = 1e-7` tolerance, documented per-case with a `// POLY cluster assertion per D-32.10` comment.
  - **Why:** the cluster pattern IS the OM-documented behavior; asserting it preserves hardware fidelity while staying within cross-platform-safe bounds. Per-root within-1e-2 would be looser than the algorithm actually produces (false negatives unhelpful); bit-exact would be tighter than FPU determinism allows.

- **D-32.11: INTG and SOLVE error paths get ~3 cases each in `numerical_accuracy.rs`.** INTG: 3 cases hitting the 2^15 = 32768 subdivision cap (e.g., highly oscillatory `sin(1/x)` near 0); SOLVE: 3 cases hitting non-convergence (e.g., `tan(x) = 10` with bad guesses far from the actual root). Each asserts `Err(HpError::Domain("DATA ERROR"))`. The math1_user_callback.rs and math1_solve_paths.rs files already cover the success/edge paths; these 6 cases close the error-branch gap in the accuracy suite.
  - **Why:** error branches receive little organic test exposure. 3 cases per program is enough to assert the error message and the trigger condition without bloating the suite.

### Claude's Discretion

- **Exact distribution of the ~134 cases within each family.** D-32.9 gives counts per family; planner picks specific OM examples within each. Constraint: every case cites an OM page+example, an emulator-extension marker (D-28.3 / D-28.4 / D-28.5), or a Phase 28-31 decision ID. No uncited cases.
- **Per-case `// Catches: <bug class>` doc comment.** Phase 27 added these to risk-weighted tests; Phase 32 should mirror the discipline but the planner picks per-case wording. Categories: "POLY multiplicity-as-cluster", "INTG non-convergence", "CMPLX (0,0) boundary", "MATRIX small-order DET", etc.
- **`lint_math1_assertions.rs` scope.** Discussion didn't lock the scope. Planner picks between (a) lint only `tests/math1_*.rs` (narrow, surgical, matches D-32.9 case-selection scope), (b) lint all `tests/` (broader, catches future drift). Recommendation: option (a) for now; v3.1+ can widen if a gap surfaces. Constraint: must run as a normal cargo test (not a separate CI step) so it gates `just test` as well as `just ci`.
- **`tests/lint_math1_assertions.rs` strictness on tolerance-free patterns.** The lint targets `assert_eq!(decimal, decimal)` on iterated results. Whether to also flag `assert!((a-b).abs() < EPSILON)` patterns (forcing `approx::assert_relative_eq!` everywhere) is planner's choice. Recommendation: flag the manual-tolerance pattern too — single-source-of-truth for tolerance discipline is cleaner.
- **Coverage-gap-closure plan.** If `just coverage` surfaces math1 files below 90 % after the ~134 new cases land, planner picks per-file gap-closure cases. Constraint: every new test carries a `// Catches: <bug class>` comment per D-27.1 — no coverage padding.
- **`scripts/check-free42-contamination.sh` exit-code semantics.** Exit 0 on no matches; exit 1 on any match outside the allowlist. Planner picks whether to also print line context (`-n` flag is already in the grep) for easy review, and whether the script should be `set -euo pipefail` (recommended).
- **Final ship commit message format.** D-32.6 says the commit embeds `just coverage` output. Planner picks the exact format — likely a fenced code block in the commit body with the coverage report's "Lines: 95.X%" + "Regions: 93.X%" lines.
- **PROJECT.md and CLAUDE.md update wording for the `v3.0 additions` Phase 32 subsection.** Mirrors the Phase 27/28/29/30 subsection structure. Planner picks the exact prose, constrained by the existing v3.0 additions block conventions. The Phase 32 subsection replaces the `(in progress)` stub left by Phase 30.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.0 milestone scope, target feature areas, build sequence, key decisions ledger (ADR-001..ADR-005 entries present); Phase 32 updates the "Active" line and graduates to "Shipped" entry post-ship
- `.planning/REQUIREMENTS.md` — Phase 32 maps to QUAL-01..08 (8 requirements; full text in §"Active v3.0" lines 175-182)
- `.planning/ROADMAP.md` — Phase 32 section (lines 187-216) — 5 success criteria, 3 plans, notable risks/decisions; cross-cutting constraints lines 35-45
- `.planning/STATE.md` — Phase 31 complete (5/5), Phase 32 ready to plan; updates on workflow end
- `CLAUDE.md` (repo root) — `### v2.2 additions (Test Hardening, Phase 27)` block is the structural template for Phase 32's subsection update inside the `### v3.0 additions` block; Phase 32 also locks down the FN-QUAL-01..05 + D-27.* invariants referenced extensively in the v2.2 block

### Phase 27 (the v2.2 test-hardening precedent Phase 32 mirrors)

- `.planning/phases/27-test-hardening/27-CONTEXT.md` — D-27.1 (`// Catches: <bug class>` doc comments), D-27.2 (atomic coverage raise lessons-learned), D-27.3 (per-Op test count rationale), D-27.5/D-27.6 (numerical accuracy with OM citations), D-27.7 (citation discipline), D-27.13 (literal ROADMAP scope = sole E2E spec), D-27.15 AMENDED (WebdriverIO + Ubuntu only) — every one of these decisions flows directly into Phase 32 design
- `hp41-core/tests/numerical_accuracy.rs` — 566-case v2.2 baseline; Phase 32 extends to ~700+ in-place
- `hp41-gui/e2e/smoke.spec.js` — Phase 27 `2 ENTER 3 +` smoke test; Phase 32 adds two new `it()` blocks in the same file
- `hp41-gui/wdio.conf.cjs` — Phase 27 WebdriverIO config (framework: mocha, retries: 1); Phase 32 makes no changes
- `.github/workflows/ci-gui.yml::e2e-linux` — Phase 27 Ubuntu-only e2e job; Phase 32 inherits unchanged

### Phase 28-31 (the v3.0 surface Phase 32 hardens)

- `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md` — XROM framework decisions; the 5 ADRs locked here; D-28.7 (cancellation plumbing in core, wired in 31); ADR-002 disclaim sentence (verbatim Free42 reference for the per-file header)
- `.planning/phases/29-cli-integration/29-CONTEXT.md` — CLI integration; `submit_modal` / `cancel_modal` / `requires_alpha_label` API; `PendingInput::XeqByName` with `XeqByNameMode`; modal-prompt routing through `state.modal_prompt`
- `.planning/phases/30-documentation-adrs/30-CONTEXT.md` — ADRs + divergence catalog; D-30.9 (hard claim conditional on Phase 32 gate); the `### v3.0 additions` block template
- `.planning/phases/31-gui-integration/31-CONTEXT.md` — GUI surface; LCD-alternation routing (D-31.5); `request_cancel` Tauri command; CAT 2 implementation; the e2e smoke is the verification surface Phase 32 extends

### ADRs (locked decisions Phase 32 builds upon)

- `docs/adr/v3.0-001-op-strategy.md` — Op-strategy A locked (1 Op variant per function); rationale for the 4-way exhaustive-match invariant
- `docs/adr/v3.0-002-user-callback-policy.md` — Strict-reject nested INTG/SOLVE/DIFEQ; **the verbatim Free42 disclaim sentence ("Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979); Free42 source consulted only as sanity-check oracle, not copied.") that `scripts/check-free42-contamination.sh` allowlists**
- `docs/adr/v3.0-003-inv-epsilon.md` — INV singularity threshold (matrix tests reference this)
- `docs/adr/v3.0-004-intg-threshold.md` — INTG convergence threshold tied to DisplayMode (INTG tests reference this)
- `docs/adr/v3.0-005-json-pipeline.md` — Separate-file JSON pipeline (`function_matrix_parity.rs` tests reference this)

### Divergence catalog (cases Phase 32 cites in `// Source:` comments)

- `docs/hp41-math1-divergences.md` — OM divergences (D-30-01..04), emulator extensions (D-30-05 `XEQ "REAL"`), behavioral policies (D-30-06..07 strict-reject + R/S submit)

### hp41-core public surface (what Phase 32's tests exercise)

- `hp41-core/src/ops/math1/xrom.rs` — `xrom_resolve(name, modules)`, `MATH_1: XromModule`; `tests/xrom_shadowing.rs` (currently 3.2K) hardens the no-shadow invariant
- `hp41-core/src/ops/math1/modal.rs` — `ModalProgram` enum + per-program step states; `submit_modal`/`cancel_modal` per Phase 29
- `hp41-core/src/ops/math1/hyperbolics.rs` — HYP-01..06; tests in `math1_hyperbolics.rs` (11.1K)
- `hp41-core/src/ops/math1/complex.rs` — CMPLX-01..18 (incl. XEQ "REAL"); tests in `math1_complex.rs` (11.9K) + `math1_complex_edge_cases.rs` (7.1K) + `math1_complex_functions.rs` (21.4K)
- `hp41-core/src/ops/math1/poly.rs` — POLY-01..07; tests in `math1_poly.rs` (9.3K)
- `hp41-core/src/ops/math1/matrix.rs` — MAT-01..11; tests in `math1_matrix.rs` (17K) + `math1_matrix_flow.rs` (7.8K)
- `hp41-core/src/ops/math1/integ.rs` — INTG-01..08; tests in `math1_integ.rs` (16.7K)
- `hp41-core/src/ops/math1/solve.rs` — SOLV-01..08; tests in `math1_solve.rs` (11.8K) + `math1_solve_paths.rs` (11.6K)
- `hp41-core/src/ops/math1/difeq.rs` — DIFEQ-01..05; tests in `math1_difeq.rs` (7.4K)
- `hp41-core/src/ops/math1/four.rs` + `tri.rs` + `trans.rs` — FOUR-01..06, TRI-01..05, TRANS-01..05; combined test file `math1_four_tri_trans.rs` (23.2K)
- `hp41-core/src/state.rs` — `cancel_requested: Arc<AtomicBool>` (Phase 28 D-28.7); `tests/cancel_flag_reset_on_open.rs` already exercises lifecycle

### Test infrastructure (what Phase 32 extends)

- `hp41-core/tests/math1_op_test_count.rs` (6.3K) — currently vacuous from Plan 28-01; Phase 32 graduates to non-vacuous (Plan 32-01)
- `hp41-core/tests/math1_user_callback.rs` (20K) — 5 regression tests for user-callback re-entrancy (already implements QUAL-08)
- `hp41-core/tests/xrom_shadowing.rs` (3.2K) — graduates from vacuous to active gate as `MATH_1.ops` populates
- `hp41-core/tests/xrom_chain_order.rs` (4.8K) — resolver chain ordering tests
- `hp41-core/tests/op_catalog_xrom.rs` (4.6K) — CATALOG 2 tests
- `hp41-core/tests/v3_save_compat.rs` (4.6K) — save-file backward compat tests
- `hp41-core/tests/numerical_accuracy.rs` (187K, 60 `#[test]` fns, 50+ `case!()` macros visible in v2.2-baseline range) — Phase 32 grows to ~700+ cases
- `hp41-core/tests/program_execution_coverage.rs` (23K) — Phase 32 may extend per coverage-gap-closure (Claude's discretion)
- `hp41-core/tests/lint_math1_assertions.rs` — NEW file in Phase 32 (Plan 32-01)
- `scripts/check-free42-contamination.sh` — NEW file in Phase 32 (Plan 32-03)
- `scripts/check-tauri-permissions.sh` (1.2K, v2.0) — template for the new contamination guard's CI integration pattern

### v2.2 baseline (pattern reservoir)

- `hp41-core/tests/phase22_stats_size_shrink.rs` (7.4K) — Pitfall-5 SIZE-shrink sentinels pattern; Phase 32 mirrors the `// Catches:` discipline
- `hp41-core/tests/format_eng_edges.rs` (11.8K) — Phase 27 risk-weighted test precedent
- `hp41-cli/tests/function_matrix_parity.rs` — bidirectional Op-enum ↔ JSON parity; Phase 32 makes no changes (Phase 29 already extended)
- `hp41-cli/tests/key_coverage.rs` — D-25.18 key-coverage closure; Phase 32 makes no changes

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`case!(name, op_sequence, expected)` macro in `numerical_accuracy.rs`** — already established case format; Phase 32's ~134 new cases use it bit-for-bit. Each case carries a leading `// Source: HP 00041-90034 p.<n>, ex.<m>` comment per D-27.7 + a `// Catches: <bug class>` comment per D-27.1.
- **`approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)`** — default tolerance for `tests/math1_*.rs`; Phase 32 standardizes via `lint_math1_assertions.rs`.
- **`include_str!` + parse-Op-variants pattern in `math1_op_test_count.rs`** — the vacuous version already shipped; Phase 32 just removes the "no Math Pac I Op variants exist" guard and lets the gate become non-trivial (Phase 28-30 already grew the Op enum and the math1_*.rs test files).
- **`scripts/check-tauri-permissions.sh` (v2.0 pattern)** — bash + grep + exit-1-on-match shape; Phase 32's contamination guard mirrors this exactly (different patterns, identical script architecture).
- **Per-file Free42 disclaim header on every math1/*.rs file** — already in place (confirmed by codebase scout: 13/13 files have the header); Phase 32's CI script greps with an allowlist for this exact line, so the disclaim doesn't trip the gate.

### Established Patterns

- **`#![allow(clippy::unwrap_used)]` at file scope in test modules** — every `tests/*.rs` file in `hp41-core/tests/` carries this allowlist line; new Phase 32 test files (`lint_math1_assertions.rs`) follow the same pattern.
- **Test file naming: `phase<N>_*.rs` for cross-cutting v2.2 features; `math1_<area>.rs` for v3.0 Math Pac I tests** — Phase 32 follows the second convention: `lint_math1_assertions.rs`.
- **`just ci` is the sole task runner entry point** — no bare `cargo` commands; Phase 32's contamination guard is invoked via a justfile recipe line.
- **`gsd-sdk query commit` for documentation commits** — `commit_docs: true` from init means Phase 32's CONTEXT.md + DISCUSSION-LOG.md commit goes through this path.

### Integration Points

- **`tests/math1_op_test_count.rs` reads `hp41-core/src/ops/mod.rs` at test time via `include_str!`** — Phase 32 graduates this from vacuous to non-vacuous; integration point is the parsing logic that detects which Op variants are Math Pac I (matched by Plan 28-02..28-10 introduction date — see file comment for the criterion).
- **`hp41-gui/e2e/smoke.spec.js` uses raw WebDriver selectors `[data-key-id]` and `[data-testid="lcd-display"]`** — both already in place from Phase 27 (Keyboard.tsx + Display14Seg.tsx). Phase 32's two new tests use the same selectors; no frontend changes needed.
- **`docs/hp41-math1-functions.json` provides the function inventory** — Phase 32's `numerical_accuracy.rs` cases reference function names that appear in this JSON. `tests/function_matrix_parity.rs` already asserts bidirectional Op-enum ↔ JSON parity (Phase 29).

</code_context>

<specifics>
## Specific Ideas

- **OM-cited hard claim**: "v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034" — the OM citation grounds the claim in the authoritative spec; "behavioral emulation" is implicit hedging against bit-exact ROM-image reproduction (which we deliberately don't do per the permanent-exclusion policy on HP-copyrighted ROM bytes).
- **POLY cluster centroid + max-imag tolerance pair** — `mean(roots).re ≈ 1.0 ± 1e-4` AND `max(|roots[i].imag|) < 1e-3` for `(x-1)^5`. Two-part assertion; one for the average behavior, one for the spread bound.
- **MATRIX 2x2 DET E2E case** — explicit values: matrix `[[1,2],[3,4]]` has det = `1·4 − 2·3 = −2`. Display reads `-2.0000` (FIX 4 default).
- **sinh(1) E2E case** — `sinh(1) = (e − 1/e)/2 ≈ 1.17520119364...` ; FIX 4 default → LCD reads `1.1752`.

</specifics>

<deferred>
## Deferred Ideas

- **Smoke spec `.js` → `.ts` conversion** — deferred until a future test legitimately needs strong typing (e.g., importing `CalcStateView` from the IPC layer). Documented in D-32.1 as a noop drift from ROADMAP wording.
- **Esc-cancel verification in MATRIX DET E2E** — Vitest unit tests in `hp41-gui/src/` exercise cancel semantics; E2E layer doesn't need to duplicate. Documented in D-32.3.
- **Coverage gate raise above 95 %** — explicitly deferred per D-27.2 lessons-learned. Future milestones (v3.1+) may raise atomically when risk-weighted tests justify.
- **Cross-platform CI matrix expansion** — current 3-OS matrix (Win/macOS/Ubuntu) + 1e-7 tolerance discipline is sufficient for v3.0. ARM macOS / Linux aarch64 jobs are a v3.x+ consideration if Apple-Silicon FPU divergence surfaces.
- **`lint_math1_assertions.rs` widening to all `tests/`** — Phase 32 keeps the lint scoped to `tests/math1_*.rs` only. Future widening when a gap surfaces in non-math1 tests.
- **Headers-only audit gate as a separate script** — D-32.7 covers contamination via the symbol grep; verifying every math1 file has the disclaim header is a hygiene check that could ship as a sibling script (e.g., `scripts/check-disclaim-header.sh`). Deferred — current confidence from per-file headers is high enough.
- **PROJECT.md graduation to "Shipped v3.0" entry** — happens at Phase 32 ship-time; technically not deferred (it's part of the final commit), but logged here as a follow-up checklist item the planner shouldn't forget.

</deferred>

---

*Phase: 32-test-hardening*
*Context gathered: 2026-05-18*
