# Phase 32: Test Hardening — Research

**Researched:** 2026-05-18
**Domain:** Test hardening for v3.0 Math Pac I (re-application of v2.2 Phase 27 / FN-QUAL-01 playbook)
**Confidence:** HIGH

## Phase Summary

Phase 32 closes v3.0 by **re-running the v2.2 Phase 27 test-hardening playbook** on the Math Pac I surface that landed in Phases 28-31. The work is test-only (no `hp41-core/src/` source edits — FROZEN since Plan 25-01; no `hp41-gui/src-tauri/src/` source edits — SC-4) and decomposes into three plans: (32-01) graduate the existing-but-vacuous `math1_op_test_count.rs` + `xrom_shadowing.rs` gates to non-vacuous, add `lint_math1_assertions.rs`, close any per-file coverage gaps surfaced by `just coverage`; (32-02) extend `numerical_accuracy.rs` from 566 → ~700 cases with OM-cited Math Pac I cases; (32-03) extend the WebdriverIO E2E smoke with two new test functions (sinh(1) + MATRIX DET), ship `scripts/check-free42-contamination.sh` in both `just ci` and `.github/workflows/ci.yml`, and graduate the README v3.0 line from soft-claim to OM-cited hard claim.

**Primary recommendation:** Phase 27 is the verbatim template — every section of this research maps a Phase 27 artifact to its Phase 32 counterpart. Three things stand out as cheaper than expected and one thing stands out as harder than expected: (a) `math1_op_test_count.rs` already exists and the underlying tests already meet the ≥ 5 mentions threshold for all 45 Math Pac I variants (minimum: TriSaa=6, TriSas=6) — the gate just needs its vacuous-early-return removed; (b) every math1 source file already carries the Free42 disclaim header (13/13 confirmed) — the CI script is purely a tripwire, not a fix-it gate; (c) `approx::assert_relative_eq!(..., max_relative = 1e-7)` is already an active dev-dep and pattern in `math1_complex.rs` etc.; (d) the Free42 contamination guard MUST allowlist `Free42` as a bare string because 122 legitimate `Free42 v3.0.5: …` cross-check references exist across `hp41-core/tests/numerical_accuracy.rs` (13) and `hp41-core/src/ops/math1/*.rs` (58) — the 12 D-32.7 grep symbols are tighter and DO NOT match any of these, so the script as locked is correct.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (from 32-CONTEXT.md `<decisions>`)

**Carried forward from v2.2 / Phase 28-31 (NOT re-decided here):**
- **D-27.2 (lessons-learned):** Coverage gate HELD at v2.2 level (≥ 95 % lines / ≥ 93 % regions). NO atomic raise this milestone — atomic raises require risk-weighted tests, not coverage padding.
- **D-27.3:** Per-Op test count ≥ 5 (Pitfall 16) — risk-weighted, NOT coverage padding. Documented `// Catches: <bug class>` comments on each new test per D-27.1.
- **D-27.5/D-27.7:** Numerical accuracy cases carry OM page-and-example citations per case. 27 v2.2 citations are the precedent; Phase 32 adds ~134 more across 11 Math Pac I families.
- **D-27.6:** `baseline_passes >= 498` floor on the v1.x 503-case subset independently asserted alongside the combined ≥ 98 % gate.
- **D-27.15 AMENDED:** WebdriverIO + tauri-driver on Ubuntu only (`e2e-linux` job in `ci-gui.yml`). macOS/Windows matrix jobs unchanged.
- **D-28.6:** Math Pac I uses XEQ-by-name only — no dedicated key bindings.
- **D-29.1:** `docs/hp41-math1-functions.json` shipped in Phase 29.
- **D-30.9:** Hard claim deferred to Phase 32 conditional on coverage gate ≥ 95 %. Phase 32's D-32.5 graduates the soft-claim.
- **ADR-001 / ADR-002 / ADR-005:** all three locked across Phase 28/30.
- **Pitfall 14:** Math Pac I floor is 6 of HP-41's 10 digits → `max_relative = 1e-7` is default; deliberate exceptions documented per-case.
- **Pitfall 16:** Per-Op test count ≥ 5 prevents mid-milestone coverage drop.
- **Pitfall 17:** No `assert_eq!(decimal, decimal)` on iterated results.
- **Pitfall 19:** Free42 GPL contamination guard. Per-file disclaim header already in place on all 13 math1/*.rs files (confirmed).

**Phase 32 session decisions D-32.1 — D-32.11:**

- **D-32.1:** smoke.spec.js retained — ROADMAP .ts wording is a noop drift.
- **D-32.2:** Two new test functions land — sinh(1) AND MATRIX 2x2 DET, separate `it()` blocks inside the existing describe block in `smoke.spec.js`.
- **D-32.3:** MATRIX DET test does NOT add an Esc-cancel verification step at the end.
- **D-32.4:** Existing `2 ENTER 3 +` smoke test unchanged.
- **D-32.5:** README v3.0 line graduates from soft-claim to hard claim: "v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034".
- **D-32.6:** Final ship commit is gate-verified — embeds `just coverage` output; parallel updates to PROJECT.md and CLAUDE.md `### v3.0 additions` block in the same commit.
- **D-32.7:** Tight 12-symbol grep policy: `phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License` with allowlist `grep -v 'Free42 source consulted only as sanity-check oracle'`.
- **D-32.8:** Guard runs in BOTH `just ci` AND a separate visible "License audit" step in `.github/workflows/ci.yml`.
- **D-32.9:** Risk-weighted case distribution across 11 Math Pac I families: POLY ~25, CMPLX ~20, MAT ~18, INTG ~15, SOLVE ~15, DIFEQ ~12, HYP ~10, TRI ~8, FOUR ~6, TRANS ~3, REAL ~2 (~134 total → ~700 combined with the 566 baseline).
- **D-32.10:** POLY multiplicity-as-cluster assertion strictness — centroid within 1e-4 + max-imag < 1e-3.
- **D-32.11:** INTG and SOLVE error paths get ~3 cases each in `numerical_accuracy.rs` (subdivision cap + non-convergence).

### Claude's Discretion

- **Exact distribution of the ~134 cases within each family** (constraint: every case cites an OM page+example, an emulator-extension marker D-28.3/4/5, or a Phase 28-31 decision ID).
- **Per-case `// Catches: <bug class>` doc comment wording.**
- **`lint_math1_assertions.rs` scope.** Recommendation: lint only `tests/math1_*.rs` (option (a) per CONTEXT.md). Must run as a normal `cargo test` so it gates `just test` as well as `just ci`.
- **`lint_math1_assertions.rs` strictness on tolerance-free patterns.** Recommendation: flag the manual-tolerance pattern `(a-b).abs() < EPSILON` too (single-source-of-truth for tolerance discipline).
- **Coverage-gap-closure plan** if `just coverage` surfaces math1 files below 90 %.
- **`scripts/check-free42-contamination.sh` exit-code semantics.** Exit 0 on no matches; exit 1 on any match outside the allowlist. Should be `set -euo pipefail`. May print line context.
- **Final ship commit message format** (likely a fenced code block in the commit body with the coverage report's "Lines: 95.X%" + "Regions: 93.X%" lines).
- **PROJECT.md and CLAUDE.md update wording for the `v3.0 additions` Phase 32 subsection.**

### Deferred Ideas (OUT OF SCOPE — DO NOT RESEARCH)

- Smoke spec `.js` → `.ts` conversion.
- Esc-cancel verification in MATRIX DET E2E.
- Coverage gate raise above 95 %.
- Cross-platform CI matrix expansion (ARM macOS / Linux aarch64).
- `lint_math1_assertions.rs` widening to all `tests/`.
- Headers-only audit gate as a separate script.
- PROJECT.md graduation to "Shipped v3.0" entry (this happens in the final ship commit; not a separate task).
- Any `hp41-core/src/`, `hp41-cli/src/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src/` source changes.
- New ADRs.
- Stat 1 / Time / Advantage pacs.
- HP-copyrighted ROM-image redistribution.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **QUAL-01** | `hp41-core` Coverage ≥ 95 % Lines / ≥ 93 % Regions (HELD, not raised) | §"Coverage Gap Analysis" + Plan 32-01 — `math1_op_test_count.rs` graduates from vacuous; coverage push closes per-file gaps if any surface below 90 % |
| **QUAL-02** | `numerical_accuracy.rs` extended 566 → ~700+ cases with Math Pac I per-program cases + OM citations + ≥ 98 % pass rate | §"Plan 32-02" + risk-weighted distribution (D-32.9) — case! macro reused bit-for-bit; OM citation pattern already established in v3.0 hyperbolic block (line 3289+) |
| **QUAL-03** | E2E smoke extension (one Math Pac I workflow) on `e2e-linux` Ubuntu only | §"Plan 32-03" + §"E2E click-sequence resolution" — two new `it()` blocks in `smoke.spec.js` using existing `[data-key-id]` + `[data-testid="lcd-display"]` selectors; no frontend changes needed |
| **QUAL-04** | Per-Op test count ≥ 5 (Pitfall 16) | §"Plan 32-01" + §"Per-Op Test Count Audit" — all 45 variants already meet the threshold; gate just removes its vacuous-pass early-return |
| **QUAL-05** | Free42 GPL contamination guard + per-file disclaim header | §"Plan 32-03" + §"Free42 Contamination Guard Implementation" — header already on 13/13 math1 files; 12-symbol grep verified zero false positives against current source |
| **QUAL-06** | Cross-platform numerical drift (`approx::assert_relative_eq!` `max_relative = 1e-7`) | §"Plan 32-02" + §"`approx 0.5.1` API Surface" — crate already in `[dev-dependencies]`; pattern in active use in `math1_complex.rs` |
| **QUAL-07** | `tests/xrom_shadowing.rs` CI gate — no Math Pac I name shadows builtin mnemonic | §"Plan 32-01" + §"xrom_shadowing graduation" — gate already exists; becomes non-vacuous automatically because `MATH_1.ops` now has 52 entries |
| **QUAL-08** | `tests/math1_user_callback.rs` 5 regression tests | §"Plan 32-01" + §"math1_user_callback.rs status" — **already shipped with 9 active tests** across Plans 28-07/08/09; QUAL-08 is met as-is. Phase 32 verifies count and adds STO-clobber / GTO-out / recursion-cap tests if the count is below 5 by category. |
</phase_requirements>

## Coverage Gap Analysis

**Status:** Coverage estimate by source-file inspection rather than running `just coverage`. The planner should run `just coverage --html` early in Plan 32-01 to confirm; the gap-closure phase is then a focused per-file pass over whichever files fall below 90 %. The CONTEXT.md (line 12) and ROADMAP success criterion 1 require ≥ 90 % per `ops/math1/*.rs` file individually.

**13 math1 source files** vs **14 math1 test files**. Source-LOC and existing-test-file mapping:

| Source file | LOC | Primary test file(s) | Test fns | Likely uncovered areas | Plan |
|-------------|----:|---------------------|---------:|------------------------|------|
| `complex.rs` | 2030 | `math1_complex.rs` (361), `math1_complex_functions.rs` (637), `math1_complex_edge_cases.rs` (167) | 30+60+4 | Error-arm coverage on rare branch cuts; `LnZ(0,0)` per Pitfall 6 already tested in `math1_complex_edge_cases.rs` | 32-01 gap-close if needed |
| `matrix.rs` | 1046 | `math1_matrix.rs` (543), `math1_matrix_flow.rs` (213) | 41+9 | INV singular-matrix `NO SOLUTION` path; SIMEQ with EPSILON edge; >9×9 matrices | 32-01 gap-close if needed |
| `poly.rs` | 849 | `math1_poly.rs` (259) | 13 | Multiplicity-as-cluster `(x-1)^5` (POLY-06); non-convergence `|imag|>10^9` (POLY-07); degree-2/3/4/5 OM examples | 32-02 (case-driven; POLY=25) |
| `integ.rs` | 929 | `math1_integ.rs` (452), `math1_user_callback.rs` (456, partial) | 16+9 | Subdivision cap 2^15 (INTG-07); cancel branch (D-28.7) | 32-01 + 32-02 (~3 INTG error cases per D-32.11) |
| `solve.rs` | 1129 | `math1_solve.rs` (333), `math1_solve_paths.rs` (261) | 15+3 | Non-convergence path; 100-iter cap; ROOT IS BETWEEN branch | 32-02 (~3 SOLVE error cases per D-32.11) |
| `difeq.rs` | 1370 | `math1_difeq.rs` (171), `math1_user_callback.rs` (partial) | 7 | RK4 stability for stiff; 2nd-order ODE reduction (DIFEQ-04) | 32-02 (DIFEQ=12 cases) |
| `four.rs` | 696 | `math1_four_tri_trans.rs` (775 partial) | ~12 | DFT orthogonality; RECT? toggle (FOUR-03) | 32-02 (FOUR=6 cases) |
| `tri.rs` | 951 | `math1_four_tri_trans.rs` (partial) | ~25 | SSA ambiguous case 0/1/2 solutions (TRI-05); SSS/ASA/SAA/SAS sanity | 32-02 (TRI=8 cases) |
| `trans.rs` | 913 | `math1_four_tri_trans.rs` (partial) | ~10 | 2D/3D round-trip identity (Rodrigues) | 32-02 (TRANS=3 cases) |
| `hyperbolics.rs` | 487 | `math1_hyperbolics.rs` (320) | 30 | Domain errors `Acosh(x<1)`, `Atanh(|x|≥1)` already covered | 32-02 (HYP=10 cases — likely just deepens existing) |
| `modal.rs` | 945 | (multiple via flow tests) | — | `submit_step` / `cancel_step` per-program variants | Likely already strong via integration tests |
| `xrom.rs` | 349 | `xrom_shadowing.rs` (87), `xrom_chain_order.rs` (~120) + inline `#[cfg(test)]` (~120) | 9+~4+~7 | `xrom_resolve` bit-mask off-by-ones; mnemonic-consistency loop | Strong; no action |
| `mod.rs` (math1) | 135 | `cancel_flag_reset_on_open.rs` (~130) | ~5 | `submit_modal` / `cancel_modal` / `submit_modal_with_label` paths | Strong; no action |

**Coverage measurement strategy for Plan 32-01:**

```bash
just coverage  # baseline + gate at ≥ 95 %
cargo llvm-cov --html -p hp41-core  # render per-file breakdown to target/llvm-cov/html/
```

Then for any `ops/math1/*.rs` file below 90 %, surface the uncovered line ranges from `target/llvm-cov/html/index.html` and add targeted `// Catches: <bug class>` tests to the matching `tests/math1_<area>.rs` file. The new test file `tests/lint_math1_assertions.rs` does NOT need its own coverage (it IS test infrastructure).

**Estimate (rough):** the existing 288 `#[test]` functions across 14 math1 test files already exercise most happy paths and the most-cited Pitfalls (5, 6). Coverage is likely already in the 90-95 % per-file range across most math1 sources — the bulk-extension from Plan 32-02 (~134 cases in `numerical_accuracy.rs`) will sweep many of the remaining gaps for free. **Estimated coverage gap-closure work in Plan 32-01: 5-15 new targeted tests**, not 50+. The planner should not over-provision.

## Per-Op Test Count Audit (QUAL-04 / Pitfall 16)

**Pre-computed against the 45-variant Math Pac I `Op` enum** (Plan 28-02..28-10), counting mentions in `hp41-core/tests/math1_*.rs`:

| Op variant | Mentions | OK? | | Op variant | Mentions | OK? |
|------------|---------:|:---:|-|------------|---------:|:---:|
| Sinh | 8 | ✓ | | MatrixWorkflow | 8 | ✓ |
| Cosh | 7 | ✓ | | MatSize | 8 | ✓ |
| Tanh | 8 | ✓ | | MatVmat | 9 | ✓ |
| Asinh | 8 | ✓ | | MatEdit | 8 | ✓ |
| Acosh | 7 | ✓ | | MatDet | 7 | ✓ |
| Atanh | 7 | ✓ | | MatInv | 7 | ✓ |
| CPlus | 8 | ✓ | | MatSimeq | 9 | ✓ |
| CMinus | 8 | ✓ | | MatVcol | 8 | ✓ |
| CTimes | 8 | ✓ | | Integ | 23 | ✓ |
| CDiv | 12 | ✓ | | Solve | 15 | ✓ |
| Real | 11 | ✓ | | Sol | 19 | ✓ |
| Magz | 8 | ✓ | | Difeq | 25 | ✓ |
| Cinv | 8 | ✓ | | Four | 16 | ✓ |
| ZpowN | 7 | ✓ | | TriSss | 9 | ✓ |
| Zpow1N | 8 | ✓ | | TriAsa | 7 | ✓ |
| ExpZ | 8 | ✓ | | **TriSaa** | **6** | ✓ (min) |
| LnZ | 9 | ✓ | | **TriSas** | **6** | ✓ (min) |
| SinZ | 7 | ✓ | | TriSsa | 10 | ✓ |
| CosZ | 7 | ✓ | | Trans2d | 7 | ✓ |
| TanZ | 9 | ✓ | | Trans3d | 7 | ✓ |
| ApowZ | 8 | ✓ | | | | |
| LogZ | 8 | ✓ | | | | |
| ZpowW | 10 | ✓ | | | | |
| PolyWorkflow | 13 | ✓ | | | | |
| Roots | 13 | ✓ | | | | |

**Verdict:** **all 45 Math Pac I `Op` variants already meet the ≥ 5 threshold.** The Phase 32 graduation of `math1_op_test_count.rs` is a one-line edit — remove the `if variants.is_empty() { return; }` early-return (currently at line 125 of the file). Once removed, the gate iterates the 52 entries in `MATH_1.ops`, collapses by Op variant (multiple mnemonics like C× / C* both map to CTimes), and the loop has zero failures.

**Caveat:** the current heuristic counts every line mentioning the variant name (not just `#[test]` lines), so the numbers above are upper bounds. If after graduation a variant comes in below 5 by a stricter count, the planner should either tighten the heuristic OR add explicit `#[test]` functions naming the variant. Recommendation: keep the current heuristic (line-mentions are a proxy for "the test code touches this variant") — it matched the design intent at Plan 28-01.

## Plan-by-Plan Implementation Notes

### Plan 32-01 — Coverage push + lint + meta-gate graduation

**Requirements covered:** QUAL-01, QUAL-04, QUAL-07, QUAL-08

**Files touched:**
- `hp41-core/tests/math1_op_test_count.rs` (graduate from vacuous): delete lines 125-128 (the `if variants.is_empty() { return; }` early-return) and let the gate iterate the 52 entries in `MATH_1.ops`. Add a `// Catches: Pitfall 16 — Op variants with insufficient test coverage` comment near the assertion. Carries `#![allow(clippy::unwrap_used)]` already.
- `hp41-core/tests/xrom_shadowing.rs` (graduates automatically): no source edit needed — the gate already loops `MATH_1.ops` which is no longer empty (52 entries since Plan 28-09). Verify by inspection that no Math Pac I mnemonic shadows the 18-entry `BUILTIN_CARD_OP_NAMES` allow-list at line 33-55. (Quick mental scan: SINH/COSH/TANH/POLY/MATRIX/INTG/SOLVE/DIFEQ/FOUR/SSS/ASA/SAA/SAS/SSA/TRANS/T3D/etc. — none collide with WPRGM/RDPRGM/WDTA/RDTA or any of the 14 conditional-test mnemonics.) The gate becomes a real CI check on every push.
- `hp41-core/tests/lint_math1_assertions.rs` (NEW, ~80-120 lines): include-string-scans `tests/math1_*.rs` files and asserts the file body contains zero `assert_eq!` invocations whose left- and right-hand sides reference `HpNum`/`Decimal`/`f64` (Pitfall 17 — iterated decimals can drift across platforms). Recommendation per Claude's Discretion: also flag the manual-tolerance pattern `assert!((<expr> - <expr>).abs() < <num>)`. Mirrors the parsing approach of `math1_op_test_count.rs` (`include_str!` + `std::fs::read_dir` + `std::fs::read_to_string`). File scope: only `tests/math1_*.rs` (option (a)). The lint runs as a normal `#[test]` and is picked up by `just test` AND `just ci`. Carries `#![allow(clippy::unwrap_used)]` per the established pattern.
- `hp41-core/tests/math1_user_callback.rs`: **already meets QUAL-08 with 9 active tests** (5 nested-rejection + 1 STOP + 1 STO-clobber + 2 nested-DIFEQ variants per Plan 28-09's comment header at line 11-19). Phase 32 verifies the count and adds 1-2 more if the rubric tightens (e.g., explicit "GTO-out-of-callback" or "recursion-cap" tests not currently covered).
- (conditional) Per-file gap-closure tests: if `just coverage --html` shows any `ops/math1/*.rs` file < 90 %, add 1-5 `// Catches: <bug class>` tests to the appropriate `tests/math1_<area>.rs` file. **Likely effort: 5-15 tests total**, not a major surface.

**Dep additions:** none for this plan.

**Verification:** `just coverage` reports `hp41-core` lines ≥ 95 % AND each `ops/math1/*.rs` file ≥ 90 %. `just test` includes `lint_math1_assertions::test_no_decimal_assert_eq` AND `math1_op_test_count::each_math1_op_has_at_least_5_tests` AND `xrom_shadowing::math1_names_do_not_shadow_builtins` all passing.

#### Research priority #5 — Per-Op test-count enforcement strategy

The discovery strategy is already implemented in `hp41-core/tests/math1_op_test_count.rs` and uses neither `strum::IntoEnumIterator` (no such derive is in use in `hp41-core/src/ops/mod.rs` — verified by inspection) nor compile-time enum reflection. Instead:

1. **Compile-time include of `xrom.rs` source as a string** via `include_str!("../src/ops/math1/xrom.rs")` (line 52).
2. **Parse the `math1_resolve` match arms** by scanning for `=> Some(Op::` and extracting the alphanumeric variant name (lines 55-71).
3. **Scan `hp41-core/tests/math1_*.rs`** via `std::fs::read_dir` + `std::fs::read_to_string` (lines 76-109) using `CARGO_MANIFEST_DIR` (set at compile time to `hp41-core/`).
4. **Count non-comment line mentions** of the variant name and assert ≥ 5.

This is a Wave-0 meta-test pattern: no external dep, no `build.rs`, no macro magic. The only thing Phase 32 changes is removing the vacuous-pass early-return at line 125-128. No new strategy required.

**Pseudo-code summary** (the existing implementation):
```rust
let variants: Vec<String> = scan_xrom_match_arms_for_variant_names();
// Plan 32-01: remove the `if variants.is_empty() { return; }` line here.
let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
let failures: Vec<String> = variants.iter()
    .filter_map(|v| {
        let n = count_mentions(v, &tests_dir);
        if n < 5 { Some(format!("Op::{v}: only {n}")) } else { None }
    })
    .collect();
assert!(failures.is_empty(), "Pitfall 16 violations:\n{}", failures.join("\n"));
```

#### Research priority #6 — xrom_shadowing.rs CI gate sketch

**Already implemented** at `hp41-core/tests/xrom_shadowing.rs`. The test iterates `MATH_1.ops` (now 52 entries) and asserts each mnemonic is NOT in the hand-curated `BUILTIN_CARD_OP_NAMES: &[&str]` allowlist of 18 entries (4 card-reader + 14 conditional tests across ASCII + Unicode spellings). At the time `MATH_1.ops` was empty, the test was vacuously true. With Phase 28-09's population to 52 entries, the test now exercises every Math Pac I mnemonic against every v2.2 builtin. **No edits needed in Phase 32** — the test simply graduates from vacuous to active without source changes.

**The hand-curated allowlist in xrom_shadowing.rs MUST stay in sync with `builtin_card_op` in `hp41-core/src/ops/program.rs`.** The doc-comment at line 22-32 of `xrom_shadowing.rs` warns about this. Plan 32-01 should verify that `builtin_card_op` has not gained new entries since v2.2 Plan 25-03 (4→12 conditional-test extension). Quick grep: `grep -n "fn builtin_card_op" hp41-core/src/ops/program.rs` then walk the match arms.

#### Research priority #7 — math1_user_callback.rs status

**Already has 9 active tests** across Plans 28-07/08/09 (see file header comment lines 11-19). The 5 categories from QUAL-08 map as:

| QUAL-08 category | Status | Test name |
|------------------|--------|-----------|
| Nested INTG/SOLVE/DIFEQ rejection | ✓ shipped (6 tests) | `nested_integ_inside_integ_rejected`, `nested_solve_inside_integ_rejected`, `nested_integ_inside_solve_rejected`, `nested_solve_inside_solve_rejected`, `nested_difeq_inside_integ_rejected`, `nested_difeq_inside_solve_rejected`, `nested_difeq_inside_difeq_rejected` |
| STO-clobbering | ✓ shipped | `user_fn_stores_to_scratch_corrupts_integ` (lines 411-456) |
| STOP-during-INTG | ✓ shipped | `user_fn_stops_aborts_integ` (lines 351-390) |
| GTO-out-of-callback | **not explicit** | Plan 32-01 may add `user_fn_gto_out_of_callback_handled` |
| Recursion-cap | **not explicit** | Plan 32-01 may add `user_fn_recursion_cap_via_user_callback_max_steps` (asserts `USER_CALLBACK_MAX_STEPS = 100_000` is the budget) |

**Recommendation:** Plan 32-01 adds 2 new tests (GTO-out + recursion-cap) so all 5 QUAL-08 categories are explicit. The state machine for INTG/SOLVE/DIFEQ uses `state.integ_state` / `state.solve_state` / `state.difeq_state` (the 3-state guard documented as XROM-08 FINAL at line 233) — Plan 32-01 just adds tests that exercise the existing guard paths, no source changes.

---

### Plan 32-02 — `numerical_accuracy.rs` extension 566 → ~700+ cases

**Requirements covered:** QUAL-02, QUAL-06

**Files touched:**
- `hp41-core/tests/numerical_accuracy.rs`: extend in-place with ~134 new `case!(...)` invocations distributed per D-32.9. Each case carries:
  ```rust
  // Source: HP 00041-90034 p.<n>, ex.<m> — <description>
  // Free42 v3.0.5: <value> — agrees with OM    (optional cross-check, when available)
  // Catches: <bug-class wording per planner's discretion>
  ```
  (The pattern is exactly what the v3.0 hyperbolic block at line 3289+ already does — Phase 32 continues the convention for the remaining 10 families.)
- `hp41-core/Cargo.toml`: **`approx = "0.5.1"` is already in `[dev-dependencies]` at line 18.** Plan 32-02 does NOT need to add it — verify via `grep approx hp41-core/Cargo.toml`. The earlier RESEARCH note said "approx 0.5.1 lands as the only new dev-dep" but this is incorrect — it landed earlier (likely Plan 28-03 / 28-04 alongside complex tests). **Plan 32-02 has zero dep changes.**

**Dep additions:** none — `approx 0.5.1` is already there.

**Distribution per D-32.9** (target ~134 cases):
- POLY ~25 cases — degree 2-5 OM examples (use `math1_poly.rs` p.30-32 examples as starting point); multiplicity-as-cluster `(x-1)^5` per D-32.10 with cluster-centroid + max-imag two-part assertion.
- CMPLX ~20 cases — Euler identities (`e^(iπ)+1=0`), `Cinv`/`LnZ` (0,0) boundary per Pitfall 6, exp/ln/sin/cos round-trips. Existing `math1_complex_functions.rs` is the pattern reservoir.
- MAT ~18 cases — DET via partial-pivot LU (small 2×2/3×3/4×4 OM examples on p.10-15), SIMEQ with EPSILON threshold per ADR-003.
- INTG ~15 cases — Simpson convergence per ADR-004 (threshold = 10^(-decimals-1) tied to DisplayMode); ~3 error-path cases per D-32.11 (subdivision cap, asserts `Err(HpError::Domain("DATA ERROR"))`).
- SOLVE ~15 cases — modified secant; multi-root selection; ~3 error-path cases per D-32.11 (non-convergence).
- DIFEQ ~12 cases — RK4 stability for stiff systems; 1st-order + 2nd-order ODE reduction.
- HYP ~10 cases — domain guards + identities; many likely deepen existing 9 cases at line 3287-3510.
- TRI ~8 cases — SSA ambiguous case = 0/1/2 solutions; SSS/ASA/SAA/SAS single-solution sanity.
- FOUR ~6 cases — orthogonality; rect↔polar conversion.
- TRANS ~3 cases — 2D/3D translate-rotate round-trip identity.
- REAL ~2 cases — `XEQ "REAL"` flag-flip only.

**POLY cluster assertion (D-32.10):** for `(x-1)^5` use a non-`case!` test arm (the macro asserts a single scalar pair). Pattern:
```rust
{
    // Source: HP 00041-90034 p.32 — multiplicity-5 cluster
    // POLY cluster assertion per D-32.10
    let mut s = CalcState::new();
    /* ... open POLY modal, enter degree 5, A=1 B=-5 C=10 D=-10 E=5 F=-1, run ... */
    let roots = extract_roots(&s);
    let mean_re = roots.iter().map(|(re, _)| re).sum::<f64>() / 5.0;
    let max_imag = roots.iter().map(|(_, im)| im.abs()).fold(0.0_f64, f64::max);
    assert!((mean_re - 1.0).abs() < 1e-4, "centroid drift: {}", mean_re);
    assert!(max_imag < 1e-3, "imaginary spread: {}", max_imag);
}
```

**Tolerance discipline:** every assertion uses `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` UNLESS a documented exception (POLY cluster, INTG error path, etc.). The `lint_math1_assertions.rs` gate from Plan 32-01 enforces this in `tests/math1_*.rs` — `numerical_accuracy.rs` is OUT of the lint scope (it uses the `case!` macro with its own internal tolerance per `AccuracyCase.tol`).

**v1.x baseline floor:** the existing `numerical_accuracy.rs` already asserts both the combined ≥ 98 % gate AND the independent `baseline_passes >= 498` floor per D-27.6. Plan 32-02 must NOT alter the assertion logic — only append cases.

#### Research priority #2 — `approx 0.5.1` API surface

**Crate already in active use** in `hp41-core/tests/math1_complex.rs` (line 17 `use approx::assert_relative_eq;`, then 30 usages). Pattern:

```rust
use approx::assert_relative_eq;
assert_relative_eq!(get_x(&s), 4.0, max_relative = 1e-7);
```

**Works with `f64`** directly. **Does NOT work with `HpNum` (rust_decimal-backed)** — the macro requires the type to implement `approx::RelativeEq`, which is implemented for `f32`/`f64` only in the base crate. The established pattern in `math1_complex.rs` uses `get_x(&s) -> f64` (an existing helper at line 24 of `numerical_accuracy.rs`):

```rust
fn get_x(state: &CalcState) -> f64 {
    state.stack.x.inner().to_f64().unwrap_or(f64::NAN)
}
```

This Decimal → f64 bridge via `to_f64()` is the established v2.2 pattern and continues to apply. The conversion loses 4 digits of HP-41's 10-digit precision (f64 has ~15 sig figs; the BCD is 10), but `max_relative = 1e-7` (6 of 10 digits) is well within both representations.

**Cite:** [approx 0.5.1 docs.rs](https://docs.rs/approx/0.5.1/approx/) — `assert_relative_eq!` takes `actual, expected, max_relative = <epsilon>` where epsilon is the maximum relative error.

---

### Plan 32-03 — E2E smoke extension + Free42 guard + hard-claim graduation

**Requirements covered:** QUAL-03, QUAL-05

**Files touched:**
- `hp41-gui/e2e/smoke.spec.js`: add TWO new `it()` blocks inside the existing `describe('HP-41 GUI smoke (FN-QUAL-05, D-27.13 literal ROADMAP scope)', ...)` block. **DO NOT** touch the existing `it('2 ENTER 3 + displays 5.0000', ...)` block (D-32.4 preserved). DO NOT touch `wdio.conf.cjs`. DO NOT touch `Display14Seg.tsx` (the `data-testid="lcd-display"` selector is already there from Plan 27-04).
- `scripts/check-free42-contamination.sh` (NEW): bash script implementing D-32.7. ~30 lines.
- `.github/workflows/ci.yml`: add a "License audit" step that invokes `bash scripts/check-free42-contamination.sh`. Recommended placement: right after the `lint:` job, before `test:` (fail-fast for a license-audit failure).
- `justfile` recipe `ci` (line 67): extend from `ci: lint test coverage` to `ci: lint test coverage license-audit` AND add a new recipe `license-audit: bash scripts/check-free42-contamination.sh` (or inline the bash invocation). Recommendation: separate recipe for `just license-audit` clarity.
- `README.md`: graduate the v3.0 line (currently `- Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry points, documented divergences)`) to `- v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034`. Move the "documented divergences" reference to a follow-up bullet linking `docs/hp41-math1-divergences.md`. Per D-32.5/D-32.6, this edit lands in the FINAL ship commit — not in Plan 32-03's primary code-changes commit.
- `PROJECT.md` + `CLAUDE.md`: update `Active` line and `### v3.0 additions` block (replacing the `(in progress)` stubs left by Phase 30). Final commit per D-32.6.

**Dep additions:** none. The Free42-contamination guard is bash + grep; the E2E spec is plain JavaScript using existing WebdriverIO APIs.

#### Research priority #3 — WebdriverIO test syntax (E2E click-sequence)

**Existing `smoke.spec.js`** at `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/e2e/smoke.spec.js` carries:
- One `describe()` block (line 49)
- One `it()` block (line 50): "2 ENTER 3 + displays 5.0000"
- A helper `clickKey(keyId)` (lines 35-47) that dispatches a synthetic `click` MouseEvent on `[data-key-id="${keyId}"]` via `browser.execute()`. This bypasses WebKitGTK's SVG-non-interactability quirk; React's `onClick` handlers on the SVG `<g>` still fire.
- Assertion path: `display.getAttribute('data-text')` against the `[data-testid="lcd-display"]` outer SVG. The 14-segment LCD renders SVG `<path>` only — no `<text>` nodes — so `data-text={text}` is the contract.

**Pattern for the two new `it()` blocks:** two more `it(...)` calls inside the same `describe` block. The user explicitly chose this in D-32.2 ("Both as separate `it()` blocks inside the existing describe block").

**XEQ "SINH" 1 ENTER click sequence:** the on-screen Keyboard.tsx maps XEQ to the `xeq_prompt` key (id, row 3 col 1, alphaChar='K'). Clicking XEQ opens a frontend modal (`{ kind: 'xeq_name', acc: '', dispatchPrefix: 'xeq', mode: 'normal' }` via `App.tsx::MODAL_OPENERS.xeq_prompt`). Letter keys (within the open modal) accumulate into `acc` via `handleModalKey` — they use the keys' `alphaChar` rather than their primary id. From `KEY_DEFS`:

| Letter | Key id | alphaChar | Row/Col |
|--------|--------|-----------|---------|
| S | `8` | S | 5/2 (alpha mapping; row 5 col 2 = SHIFT+8 → CF; alpha mode uses `8` key as 'S') |
| I | `cos` | I | 2/3 |
| N | `enter` | N | 4/0 |
| H | `sin` | H | 2/2 |

**However**, the modal flow inside `App.tsx::handleClick` at lines ~390-400 (per the earlier grep at line 392) shows that for `kind: 'xeq_name'` modals, the click handler reads `key.alphaChar` as the routed key (`routedKey = key.alphaChar`). So clicking `[data-key-id="8"]` after opening XEQ modal accumulates `S`; clicking `[data-key-id="cos"]` accumulates `I`; clicking `[data-key-id="enter"]` (with `alphaChar='N'`) accumulates `N` — and the Enter-to-dispatch path needs a separate trigger.

**Recommendation:** the E2E test should mirror Phase 31's CLI E2E smoke flow (if there is one) OR use the magic-prefix dispatch directly. The cleanest path is:

```javascript
it('XEQ "SINH" 1 displays 1.1752 (Math Pac I via xrom_resolve)', async () => {
    const display = await $('[data-testid="lcd-display"]');
    await display.waitForExist({ timeout: 10000 });
    // Open XEQ-by-name modal
    await clickKey('xeq_prompt');
    // Accumulate "SINH" via alphaChar routing.
    // Per KEY_DEFS: alphaChar for S=key id '8' (row 5 col 2),
    // I='cos', N='enter', H='sin'.
    await clickKey('8');     // alphaChar 'S' → acc = "S"
    await clickKey('cos');   // alphaChar 'I' → acc = "SI"
    await clickKey('enter'); // alphaChar 'N' → acc = "SIN"
    await clickKey('sin');   // alphaChar 'H' → acc = "SINH"
    // Submit the modal's accumulated label. The Enter key in xeq_name modal
    // dispatches `xeq_SINH` to Tauri. Per App.tsx::handleModalKey, Enter is
    // mapped from a specific key — need to identify which key. ALTERNATIVE:
    // call browser.execute() to dispatch the xeq_<label> id directly via
    // the global invoke shim.
    // ...
    // Push 1 onto X and call Op::Sinh via xrom_resolve.
    await clickKey('1');
    await clickKey('enter');
    // Re-trigger the dispatched XEQ "SINH" — the modal already submitted.
    await browser.pause(500);
    const dataText = await display.getAttribute('data-text');
    if (dataText !== '1.1752') {
        throw new Error(`expected 1.1752, got '${dataText}'`);
    }
});
```

**WARNING — discovery gap:** the exact key-click sequence for "submit the XEQ-by-name modal once `acc` reaches the full label" is not fully resolved by reading App.tsx — the `handleModalKey` function for `'xeq_name'` kind needs to be inspected to identify which click submits. The planner should:

1. Read `hp41-gui/src/pending_input.ts` `handleModalKey` for the `'xeq_name'` case (line ~339 per grep), find the "Enter" key handler, and identify whether it's triggered by the `enter` key id or a different key id (likely `enter`, but `enter`'s `alphaChar='N'` complicates things in alpha mode).
2. As a fallback if the click-sequence proves brittle, use `browser.execute()` to directly call `window.__TAURI__.invoke('dispatch_op', { keyId: 'xeq_SINH' })` — this bypasses the modal entirely and exercises only the backend `xrom_resolve` path. This is acceptable per QUAL-03 (the requirement is "one Math Pac I workflow" — using the xeq direct-dispatch IS a workflow).
3. Plan 32-03 should make the click-sequence discovery a sub-task ("Wave 0 / Reconnaissance") before committing to the test body.

**MATRIX DET click sequence:** `XEQ "MATRIX" 2 R/S 1 R/S 2 R/S 3 R/S 4 R/S XEQ "DET" Enter` → LCD reads `-2.0000`. The same key-routing concerns apply; recommendation is the same — verify via reconnaissance before committing the test body.

**Same `describe` block, two new `it()` blocks** (per D-32.2). DO NOT create a separate `describe` block — that would split the spec across two top-level groups in the WebdriverIO reporter output, adding ceremony without value.

**Mocha syntax confirmation:** `framework: 'mocha'`, `ui: 'bdd'` per `wdio.conf.cjs` line 94-96. `describe(...)` and `it(...)` are the canonical Mocha BDD verbs. Per CONTEXT.md `<canonical_refs>` "Phase 27 (the v2.2 test-hardening precedent Phase 32 mirrors)" — pattern preserved bit-for-bit.

#### Research priority #4 — Free42-contamination guard implementation pattern

**Template: `scripts/check-tauri-permissions.sh`** (1.2 K, lines 1-32 of that file). Standard Bash CI gate pattern:
- `#!/usr/bin/env bash`
- `set -euo pipefail`
- Grep + exit-N-on-match convention
- Single-purpose: one CI invariant per script

**Adapting to Free42 contamination guard per D-32.7:**

```bash
#!/usr/bin/env bash
# scripts/check-free42-contamination.sh
# CI gate: hp41-core/src/ops/math1/ must contain no distinctive Free42
# identifiers (Intel BID library, decNumber, Free42 internals, copyright
# markers) outside the allowlisted disclaim header on each math1 file.
#
# D-32.7: 12 distinctive symbols + AGPL/GPL copyright markers.
# D-32.8: invoked from `just ci` and from a separate `ci.yml` "License audit"
# step for visible audit trail.
#
# Rationale: realistic threat is a well-intentioned contributor copy-pasting
# a Free42 algorithm to verify behavior. These 12 symbols are distinctive
# enough that drift would force false-positive triage only in deliberate
# contamination cases. The allowlist preserves the legitimate disclaim header.
set -euo pipefail

MATH1_DIR="hp41-core/src/ops/math1"
DISCLAIM_LINE='Free42 source consulted only as sanity-check oracle'

# Distinctive Free42 / decNumber / Intel BID / Free42-internal types +
# common GPL/AGPL copyright markers.
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License'

if matches=$(grep -rn -E "$PATTERN" "$MATH1_DIR" | grep -v "$DISCLAIM_LINE"); then
    echo "FAIL: Free42 contamination detected in $MATH1_DIR:"
    echo "$matches"
    exit 1
fi

echo "OK: no Free42 contamination detected in $MATH1_DIR/"
exit 0
```

**Verification against current state:** the 12-symbol grep returns ZERO matches against `hp41-core/src/ops/math1/` (tested in research — see §"Verification" below). The allowlist preserves the 13 disclaim header lines. **The script as written is correct for v3.0 ship-time.**

**`numerical_accuracy.rs` scope question:** the script greps only `hp41-core/src/ops/math1/`, NOT `hp41-core/tests/`. The 13 `Free42 v3.0.5: <value>` cross-check references in `numerical_accuracy.rs` are SAFE — they're outside the script's scope. This is the correct boundary per D-32.7 (the contamination threat is in production code, not test cross-check values).

**`hp41-cli/` and `hp41-gui/` scope:** the script does NOT scan these. Math Pac I math logic is confined to `hp41-core/src/ops/math1/` per SC-4. Scanning the GUI/CLI would be both redundant and a maintenance burden when v3.1+ adds Stat 1 Pac.

**Verification of the 12-symbol pattern:**

| Symbol | Source | Risk if appears |
|--------|--------|-----------------|
| `phloat` | Free42 BCD wrapper class | direct code copy |
| `Phloat` | (same, capitalized variant) | direct code copy |
| `bid128_` | Intel BID 128-bit decimal library | direct code copy |
| `decNumber` | decNumber library type | direct code copy |
| `decContext` | decNumber library type | direct code copy |
| `vartype` | Free42 internal variable union | direct code copy |
| `arg_struct` | Free42 internal argument struct | direct code copy |
| `prgm_lines` | Free42 internal program line storage | direct code copy |
| `bcd_t` | BCD typedef (Free42 + others) | direct code copy |
| `Thomas Okken` | Free42 author | copyright comment / header |
| `AGPL` | License marker | GPL contamination |
| `GNU General Public License` | License marker | GPL contamination |

**Justfile integration:**

```just
[group('ci')]
license-audit:
    bash scripts/check-free42-contamination.sh

[group('ci')]
ci: lint test coverage license-audit
```

**ci.yml integration:**

```yaml
  license-audit:
    name: License audit (Free42 contamination)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: bash scripts/check-free42-contamination.sh
```

Recommended placement in `.github/workflows/ci.yml`: as a parallel job, NOT a step inside an existing job. Mirrors the way `lint` / `test` / `coverage` / `msrv` are separate jobs.

---

## Validation Architecture

> Phase 32 IS the validation phase — its "validation architecture" IS the artifact. Each Phase 32 plan installs a permanent CI gate that catches regressions in the v3.0 surface from this point forward.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + cargo-llvm-cov 0.6+ + WebdriverIO 9.x + bash |
| Config file | `Cargo.toml` (workspace + per-crate); `wdio.conf.cjs` (E2E); `justfile` (gate orchestration) |
| Quick run command | `just test-core` (filters: e.g. `just test-core --test math1_op_test_count`) |
| Full suite command | `just ci` (= `lint + test + coverage + license-audit`) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| QUAL-01 | `hp41-core` coverage ≥ 95 % lines / ≥ 93 % regions | coverage gate | `just coverage` | ✅ |
| QUAL-02 | numerical accuracy ≥ 98 % (combined ~700 cases) + baseline 498/503 | suite + floor | `cargo test -p hp41-core --test numerical_accuracy` | ✅ (extension only) |
| QUAL-03 | E2E sinh(1) + MATRIX DET workflow on Ubuntu | E2E smoke | `just gui-e2e` | ✅ (extension only) |
| QUAL-04 | Per-Op test count ≥ 5 across 45 Math Pac I variants | meta-test | `cargo test -p hp41-core --test math1_op_test_count` | ✅ (graduation only) |
| QUAL-05 | Free42 12-symbol grep clean in `hp41-core/src/ops/math1/` | CI script | `just license-audit` | ❌ Plan 32-03 |
| QUAL-06 | `approx::assert_relative_eq!` `max_relative = 1e-7` discipline | lint test | `cargo test -p hp41-core --test lint_math1_assertions` | ❌ Plan 32-01 |
| QUAL-07 | No Math Pac I name shadows v2.2 builtin mnemonic | meta-test | `cargo test -p hp41-core --test xrom_shadowing` | ✅ (graduation only) |
| QUAL-08 | 5 categories of user-callback regression tests | regression tests | `cargo test -p hp41-core --test math1_user_callback` | ✅ (9/5 categories already shipped) |

### Sampling Rate
- **Per task commit:** `just test-core --test <specific-file>` (sub-second per file).
- **Per wave merge:** `just ci` (full lint + test + coverage + license-audit gate; ~3-5 min on M1 / Ubuntu CI).
- **Phase gate:** `just ci` + `just gui-ci` + `just gui-e2e` (Ubuntu only) all green before final ship commit.

### Cross-Platform OS Coverage
| Gate | Linux | macOS | Windows |
|------|:-----:|:-----:|:-------:|
| `lint` | ✓ | — | — |
| `test` | ✓ | ✓ | ✓ |
| `coverage` (≥ 95 %) | ✓ | — | — |
| `msrv` (1.88) | ✓ | — | — |
| `license-audit` (NEW) | ✓ | — | — |
| `gui-ci` | ✓ | ✓ | ✓ |
| `gui-e2e` (NEW: 2nd + 3rd tests) | ✓ | — | — |

### Wave 0 Gaps
- [ ] `scripts/check-free42-contamination.sh` (NEW — Plan 32-03)
- [ ] `hp41-core/tests/lint_math1_assertions.rs` (NEW — Plan 32-01)
- [ ] `.github/workflows/ci.yml::license-audit` job (NEW — Plan 32-03)
- [ ] `justfile::license-audit` recipe + `ci` extension (NEW — Plan 32-03)

### Pass / Fail Thresholds (numeric)
- Coverage: hp41-core lines **≥ 95.0 %** AND regions **≥ 93.0 %**; per-`ops/math1/*.rs` file **≥ 90.0 %**.
- Numerical accuracy: combined **≥ 98.0 %** (≥ ~686 of ~700 cases) AND `baseline_passes ≥ 498` of 503 v1.x cases (independent assertion, D-27.6).
- Per-Op test count: **≥ 5 mentions per Math Pac I variant** in `tests/math1_*.rs`.
- Free42 contamination: **zero matches** of the 12-symbol pattern outside the disclaim allowlist.
- E2E smoke: 3 of 3 `it()` blocks pass with `mochaOpts.retries: 1` budget on Ubuntu.

---

## Open Questions

1. **XEQ-by-name click sequence in `smoke.spec.js`**
   - **What we know:** the E2E spec already uses the synthetic-MouseEvent `clickKey` helper. `xeq_prompt` opens an `xeq_name` modal. Letter keys accumulate into `acc` via their `alphaChar` (e.g., 'S' is on key id '8', 'I' on 'cos', 'N' on 'enter', 'H' on 'sin').
   - **What's unclear:** which key click submits the modal once the label is fully accumulated (likely 'enter', but 'enter' has `alphaChar='N'` which would APPEND to the label first). The `handleModalKey` function for `'xeq_name'` in `pending_input.ts` is the source of truth and needs to be inspected during Plan 32-03 Wave 0.
   - **Recommendation:** Plan 32-03 first sub-task (Wave 0 / Reconnaissance) is to read `pending_input.ts` for the `xeq_name` modal Enter handler. Fallback if the click sequence is brittle: use `browser.execute(() => window.__TAURI__.invoke('dispatch_op', {keyId: 'xeq_SINH'}))` to bypass the modal entirely and exercise only the `xrom_resolve` path. This is acceptable per QUAL-03's "one Math Pac I workflow" wording — the workflow IS xeq_SINH → Op::Sinh → display 1.1752.

2. **`lint_math1_assertions.rs` strictness on `assert_eq!(scalar, scalar)`**
   - **What we know:** D-32 left this as Claude's discretion; CONTEXT.md `### Claude's Discretion` recommends flagging the manual-tolerance pattern too.
   - **What's unclear:** the lint needs to distinguish `assert_eq!(state.modal_prompt, Some("ORDER=?".to_string()))` (string equality — fine) from `assert_eq!(x_val, 0.5)` (decimal/float equality — bad). The existing 111 `assert_eq!` invocations across `tests/math1_*.rs` are mostly the former (string/enum equality, error-variant equality, integer equality). The current 5 manual-tolerance patterns in `math1_matrix.rs` (lines 298, 323, 360, 427, 431) ARE the lint target.
   - **Recommendation:** the lint uses a string-level grep heuristic — flag lines matching either `assert_eq!(.*\.to_f64()` OR `(.+ - .+).abs() < <num>`. Anything else (string/enum equality) is fine. Document the heuristic in the file's doc comment with examples.

3. **Coverage gap-closure scope (Plan 32-01)**
   - **What we know:** existing 288 `#[test]` functions in `math1_*.rs` likely give most files ≥ 90 % coverage. The bulk-extension in Plan 32-02 will sweep many remaining gaps for free.
   - **What's unclear:** the exact gap won't be known until `just coverage --html` runs.
   - **Recommendation:** Plan 32-01 runs `just coverage --html` as its FIRST step and budgets 5-15 targeted tests as a likely upper bound. If the gap turns out to be larger (e.g., a math1 file at 75 %), the planner should consider whether that file shipped with insufficient tests (a Phase 28-31 process gap) and add 1-2 cases per uncovered branch with `// Catches: <bug class>` rationale.

## Sources

### Primary (HIGH confidence)
- `hp41-core/tests/math1_op_test_count.rs` — full implementation; vacuous-early-return at line 125-128
- `hp41-core/tests/xrom_shadowing.rs` — full implementation; gate already exists
- `hp41-core/tests/math1_user_callback.rs` — 457 lines, 9 active tests (5 nested-rejection + STOP + STO-clobber + 2 nested-DIFEQ)
- `hp41-core/src/ops/math1/xrom.rs` — `MATH_1.ops` populated with 52 entries (Plan 28-09)
- `hp41-core/Cargo.toml` — confirms `approx = "0.5.1"` already in `[dev-dependencies]`
- `hp41-core/tests/numerical_accuracy.rs` — 566 cases, `case!` macro at lines 100-123, Math Pac I extension precedent at lines 3287+
- `hp41-gui/e2e/smoke.spec.js` — Phase 27 pattern; `clickKey` helper and assertion style
- `hp41-gui/wdio.conf.cjs` — Mocha runner, `mochaOpts.retries: 1`
- `scripts/check-tauri-permissions.sh` — template for the Free42 contamination guard
- `justfile` — `coverage` recipe, `ci` composition pattern
- `.github/workflows/ci.yml` — job-level CI pattern; license-audit will be a sibling job
- `.planning/phases/32-test-hardening/32-CONTEXT.md` — all D-32.1 — D-32.11 decisions
- `.planning/REQUIREMENTS.md` — QUAL-01..08 (lines 173-182)
- `.planning/ROADMAP.md` — Phase 32 success criteria (lines 187-216)

### Secondary (MEDIUM confidence)
- [approx 0.5.1 on docs.rs](https://docs.rs/approx/0.5.1/approx/) — `assert_relative_eq!` API surface (verified via existing usage in `math1_complex.rs`)
- [WebdriverIO Mocha guide](https://webdriver.io/docs/frameworks/) — `describe`/`it` BDD syntax (matches existing `smoke.spec.js`)

### Tertiary (LOW confidence)
- None — every Phase 32 design decision is grounded in either CONTEXT.md (locked decisions) or existing repository state (codebase inspection).

## Metadata

**Confidence breakdown:**
- Coverage gap analysis: HIGH (288 existing tests + per-variant count audit) — gap is likely small
- `math1_op_test_count` graduation: HIGH (one-line edit; threshold met by all 45 variants)
- `xrom_shadowing` graduation: HIGH (no edit needed; gate goes from vacuous to active automatically)
- `lint_math1_assertions.rs` scope: MEDIUM (heuristic needs tuning to distinguish string/enum equality from decimal/float)
- `numerical_accuracy.rs` extension: HIGH (pattern fully established at line 3287+)
- E2E smoke extension: MEDIUM (click sequence for XEQ-by-name modal needs Wave 0 reconnaissance — fallback path via `browser.execute()` always works)
- Free42 contamination guard: HIGH (12-symbol pattern verified zero matches against current source)
- README hard-claim graduation: HIGH (D-32.5/6 fully specified)

**Research date:** 2026-05-18
**Valid until:** 2026-06-17 (stable — test infrastructure)

## RESEARCH COMPLETE
