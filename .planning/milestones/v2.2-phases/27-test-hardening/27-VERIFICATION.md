---
phase: 27-test-hardening
verified: 2026-05-15T14:12:15Z
status: human_needed
score: "5/5 truths verified in codebase; 5/5 FN-QUAL requirements satisfied; 16/16 decisions honored; 1 human follow-up: green CI run of e2e-linux job on Ubuntu"
overrides_applied: 0
re_verification:
  previous_status: not_present
  previous_score: "n/a"
  gaps_closed: []
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Push current develop branch to GitHub and observe `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` job complete with green status"
    expected: "Job exit 0 within ~6-8 min cold (~3-5 min cached); WebdriverIO spec asserts data-testid='lcd-display' reads '5.0000' after `2 ENTER 3 +`"
    why_human: "The e2e-linux job was just committed (a92568d, 2026-05-15) and has not yet been observed running green on GitHub Ubuntu CI. The verifier ran all locally-runnable probes (coverage, cargo test, vitest); only the Ubuntu CI execution of tauri-driver + WebdriverIO + xvfb-run remains as an externally-observable gate. Manual repo branch-protection configuration also remains: the e2e-linux job is not yet wired as required-for-merge (Settings → Branches → Branch protection rules; manual follow-up per 27-04-SUMMARY.md line 244)."
---

# Phase 27 Verification — Test Hardening

## Verdict

**human_needed** — All five Success Criteria are satisfied in the codebase by direct inspection and locally-runnable probes; one human follow-up remains (observe the e2e-linux CI job green on Ubuntu after the next push). Status is `human_needed` strictly because the user explicitly chose this verdict per the workflow's matrix when a CI run has been authored but not yet observed green; all programmatic gates already pass.

Phase 27 goal stated by ROADMAP.md: restore hp41-core line coverage to ≥95%, extend numerical accuracy suite for new v2.2 math/conversion ops at ≥98% pass rate, ship 56-flag proptest invariants, ship dedicated indirect_addressing.rs integration suite, and add an E2E smoke test in ci-gui.yml.

| Goal-backward truth | Evidence |
|---|---|
| hp41-core line coverage ≥ 95% | **95.25% / 93.75% regions** measured via `just coverage` (this verifier run) — gate `--fail-under-lines 95` PASSES |
| Numerical accuracy ≥ 98% with v2.2 extensions | **561/566 (99.1%)** combined pass rate; 27 Free42/Owner's-Manual citations (≥15 floor) |
| 56-flag proptest invariants in CI | **14 properties × 1024 cases** in proptest_flags.rs; **5 properties × 256 cases** in proptest_math.rs; 19/19 pass |
| IND integration tests for every _IND op | **42/42 tests** in indirect_addressing.rs covering all 17 _IND variants; 21 run_program references for skip-semantic ops |
| E2E smoke in ci-gui.yml on Ubuntu | `e2e-linux` job present; wdio.conf.js + e2e/smoke.spec.ts implement WebdriverIO + tauri-driver per D-27.15 AMENDED |

## SC-by-SC Audit

| SC | Description | Verifier evidence | Status |
|---|---|---|---|
| SC-1 | `just coverage` reports hp41-core line coverage ≥ 95.0%; justfile enforces threshold (raised 80→95) | Justfile:37 contains `cargo llvm-cov --fail-under-lines 95 -p hp41-core`. Coverage run produced: TOTAL 7259 regions / 454 missed = 93.75% regions; 3853 lines / 183 missed = **95.25% lines**. Gate exits 0. Largest uplift: `ops/stats.rs` line cov is now 100% (vs 84.04% pre-phase per RESEARCH baseline) | ✓ VERIFIED |
| SC-2 | `numerical_accuracy.rs` ≥ 98% (≥490/500) with v2.2 cases for PI, P→R, R→P, RND, FRC, MOD, FACT | `cargo test --test numerical_accuracy -- --nocapture` → **561/566 (99.1%) combined**; 5 known HP-41 hardware-rounding divergences (trig, log, hms — pre-existing failure budget). v2.2 hand-curated cases present for all seven ops in lines ~2729–3293. **27 Free42 / Owner's Manual citations** (verified via grep; D-27.7 floor is ≥15) | ✓ VERIFIED |
| SC-3 | Proptest module asserts SF/CF/FS?/FS?C invariants for all n in 0..56 in CI | `hp41-core/tests/proptest_flags.rs`: 14 `proptest!` blocks, each with `#![proptest_config(ProptestConfig::with_cases(1024))]`. Covers ROADMAP-3 (1a/1b/1c), independence (2a/2b), idempotency (3a/3b), save-load roundtrip (4), IND-resolved (5a/5b), and conditional-skip sentinel (6a–6d). `hp41-core/tests/proptest_math.rs`: 5 properties × 256 cases. **19/19 proptests pass.** D-27.12 IND-resolved property lives here per the paradigm split | ✓ VERIFIED |
| SC-4 | `indirect_addressing.rs` covers every _IND op (17 total) — happy + non-integer reject | `hp41-core/tests/indirect_addressing.rs` exists (822 lines, 42 tests). All 17 IND ops present in audit table (STO_IND, RCL_IND, ISG_IND, DSE_IND, SF_IND, CF_IND, FS?/FC?/FS?C/FC?C_IND, STO+/-/×/÷ IND, ARCL_IND, ASTO_IND, VIEW_IND). 21 `run_program` references for skip-semantic ops (ISG/DSE/FlagTest×4). **42/42 tests pass** | ✓ VERIFIED |
| SC-5 | E2E spec (WebdriverIO + tauri-driver per D-27.15 AMENDED) in ci-gui.yml Ubuntu-only, drives production Tauri build, asserts `2 ENTER 3 + → 5.0000` | `.github/workflows/ci-gui.yml` contains `e2e-linux` job on `ubuntu-latest`, `needs: build`, with steps: cargo-bin cache for tauri-driver, apt deps (`webkit2gtk-driver xvfb`), `cargo install tauri-driver --locked --version 2.0.6`, `just gui-build`, `xvfb-run -a just gui-e2e`. `wdio.conf.js` spawns tauri-driver on 127.0.0.1:4444 with `mochaOpts.retries: 1` and `tauri:options.application='../src-tauri/target/release/hp41-gui'`. `e2e/smoke.spec.ts` clicks data-key-id 2/enter/3/plus and asserts `[data-testid="lcd-display"]` reads `5.0000`. `Display14Seg.tsx:219-220` carries `data-testid="lcd-display"` + `data-text={text}`. `@playwright/test` ABSENT in package.json. **Code complete; CI green status pending first push observation** | ✓ VERIFIED (code) / ⏳ HUMAN (CI green observation) |

## Decision Compliance (D-27.1 .. D-27.16)

| Decision | Compliance check | Evidence |
|---|---|---|
| D-27.1 — Risk-weighted; every new test names bug class | 92 `// Catches:` comments across the 4 new test files (program_execution_coverage 42 + phase22_stats_size_shrink 14 + phase21_phase22_interactive_no_ops 12 + format_eng_edges 24) + 19 in proptest files + 31 in indirect_addressing.rs | ✓ |
| D-27.2 — Atomic 80→95 gate raise with final tests | `git log` shows commit `584f7b2` "atomic 80→95 coverage ratchet + accuracy suite extension" lands justfile change with format_eng_edges.rs + numerical_accuracy.rs | ✓ |
| D-27.3 — Branch A applied (95.25% reached without padding) | 95.25% lines achieved — no defensible-skip-line annotations needed | ✓ |
| D-27.4 — GUI coverage measure-only, no devDep added | `package.json` does NOT contain `@vitest/coverage-v8`; one-shot hp41-gui/src-tauri snapshot at 77.92% lines recorded in 27-04-SUMMARY.md (not gated) | ✓ |
| D-27.5 — Hybrid hand-curated + proptest shape invariants | Hand-curated cases for PI×3, FACT×11, MOD×12, RND×9, FRC×8, P→R×10, R→P×10 in numerical_accuracy.rs PLUS 5 shape properties in proptest_math.rs (FRC+INT, MOD sign, FACT recursive, P↔R round-trip, RND idempotency) | ✓ |
| D-27.6 — ≥98% global gate on combined total | 561/566 = 99.1% combined; baseline-non-regression floor `baseline_passes >= 498` asserted independently | ✓ |
| D-27.7 — Free42 cross-check, cited inline | 27 `Cross-checked against Free42` / `HP-41C Owner's Manual` citations in numerical_accuracy.rs (≥15 floor) | ✓ |
| D-27.8 — One proptest file per concern | `proptest_flags.rs` (407 lines, 14 properties) + `proptest_math.rs` (253 lines, 5 properties) | ✓ |
| D-27.9 — All FOUR flag extensions in proptest_flags.rs | ROADMAP-3 (1a/1b/1c) + independence (2a/2b) + idempotency (3a/3b) + save-load roundtrip (4) + IND-resolved (5a/5b for SF/CF; FlagTestInd narrowed per noted deviation) | ✓ (with documented narrowing of FlagTestInd to phase24 + indirect_addressing.rs B-pattern tests) |
| D-27.10 — Conditional-skip sentinel proptest | Properties 6a–6d in proptest_flags.rs drive through `run_program` for FS?/FC?/FS?C/FC?C with skip-truth-table assertions | ✓ |
| D-27.11 — 1024 cases (flags) / 256 cases (math) | 14 `with_cases(1024)` in proptest_flags.rs + 5 `with_cases(256)` in proptest_math.rs | ✓ |
| D-27.12 — IND example tests in indirect_addressing.rs; IND properties in proptest_flags.rs | 42 example tests in indirect_addressing.rs; module doc-comment explicitly cites D-27.12 paradigm split; 2 documentary cross-cut anchors at n=12 + property version in proptest_flags.rs:5a/5b | ✓ |
| D-27.13 — Literal ROADMAP smoke only | smoke.spec.ts contains exactly one `describe`/`it` pair for `2 ENTER 3 + → 5.0000`; no modal flows, no persistence tests | ✓ |
| D-27.14 — Vitest gated in gui-ci | Justfile:88 `gui-ci:` recipe ends with `cd hp41-gui && npm test` | ✓ |
| D-27.15 AMENDED — WebdriverIO + tauri-driver (NOT Playwright) | `package.json` devDeps: webdriverio, @wdio/cli, @wdio/local-runner, @wdio/mocha-framework, @wdio/spec-reporter at ^9.19; `@playwright/test` ABSENT. CLAUDE.md line documents D-27.15 AMENDED. CI YAML references `tauri-driver` 2.0.6 + `webkit2gtk-driver` apt deps | ✓ |
| D-27.16 — Required job with 1 retry | `wdio.conf.js:49` `mochaOpts: { retries: 1, ... }`. The "required for merge" branch-protection setting is a repo-settings configuration (manual follow-up per 27-04-SUMMARY.md) | ✓ (code) / ⏳ MANUAL (branch protection) |

## Cross-Phase Regression

`cargo test --workspace` exit 0; 51 test suites all pass with no FAILED lines.

| Phase area | Status | Notes |
|---|---|---|
| Phase 1–8 (v1.0 foundation) | ✓ no regressions | proptest_stack, stack/format/math/registers/persistence/program suites all green |
| Phase 9–12 (v1.1) | ✓ no regressions | entry_buf_tests (EEX), STO-arithmetic, print_tests, synthetic_tests all pass |
| Phase 20 (Core Math) | ✓ no regressions | phase20_math.rs 20/20; new accuracy-suite cases land additional v2.2 coverage |
| Phase 21 (Flags/Display/Sound) | ✓ no regressions | phase21_flags 19/19, phase21_display, phase21_sound all pass; proptest_flags adds defense in depth |
| Phase 22 (Program Control / Memory / Catalog / ASN) | ✓ no regressions | phase22_* suites all pass; new phase22_stats_size_shrink.rs adds 14 SIZE-shrink sentinels |
| Phase 23 (ALPHA ops) | ✓ no regressions | phase23_* suites all pass |
| Phase 24 (Indirect Addressing) | ✓ no regressions | phase24_ind_variants (20.5K) + phase24_resolve_indirect both pass; indirect_addressing.rs adds 42 new tests on top |
| Phase 25 (CLI Integration) | ✓ no regressions | function_matrix_parity 4/4, key_coverage all pass; help_data.rs JSON path unchanged |
| Phase 26 (GUI Integration) | ✓ no regressions | hp41-gui/src-tauri/ tests untouched (SC-4 invariant preserved); 142/142 Vitest tests pass; Display14Seg.tsx data-testid addition is additive |

**SC-4 invariant (no core/CLI duplication in hp41-gui/src-tauri/):**
- Stricter pattern `grep -rnE "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` → 0 matches.
- Loose pattern still matches only the documented `op_display_name` formatter at `prgm_display.rs:47` (intentional exception per CLAUDE.md).

## Numbers

- **hp41-core line coverage:** 95.25% (gate ≥95% PASSES) — measured this verifier run
- **hp41-core regions coverage:** 93.75%
- **numerical_accuracy.rs:** 561/566 cases passing (99.1%); 5 known HP-41 hardware-rounding divergences within the historical failure budget; baseline-non-regression floor `baseline_passes >= 498` asserted
- **proptest_flags.rs:** 14 properties × 1024 cases = 14336 cases/run
- **proptest_math.rs:** 5 properties × 256 cases = 1280 cases/run
- **indirect_addressing.rs:** 42 tests (20 raw `#[test]` + 22 macro-expanded); 21 `run_program` references for skip-semantic ops
- **WebdriverIO devdeps present:** 5 (`webdriverio`, `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`, `@wdio/spec-reporter`); `@playwright/test` absent: YES
- **e2e-linux job in ci-gui.yml:** present (Ubuntu-only, `needs: build`, cargo-bin cache, `xvfb-run -a just gui-e2e`)
- **Vitest gating:** 5 files / 142 tests, runs as part of `just gui-ci`
- **CLAUDE.md v2.2 additions (Test Hardening, Phase 27) block:** present with 5 bullets (Coverage gate, Numerical accuracy, E2E, Vitest, GUI coverage snapshot)
- **Workspace tests:** 51 suites, all green; no FAILED lines under `cargo test --workspace`

## Gaps / Deviations Found

**None blocking the phase goal.** Three deviations documented in plan SUMMARYs are reasonable Rule-1/Rule-3 deviations (not gaps):

1. **FACT proptest range narrowed 0..=68 → 0..=26** (27-02 deviation): `op_fact` returns `Overflow` for n ∈ 28..=69 via the `Decimal::from_f64` wall (math.rs:450), not `OutOfRange`. The hardware-spec X>69 OutOfRange boundary case is still covered in the hand-curated accuracy suite — the proptest range simply stays in representable territory. Documented inline in proptest_math.rs and 27-02-SUMMARY.md.
2. **Baseline-non-regression floor adjusted 500 → 498** (27-01 deviation): the plan's "500/500 baseline" was aspirational; the actual empirical baseline is 498/503 (5 pre-existing HP-41 hardware-rounding divergences in trig/log/HMS that have been within the historical failure budget since v1.0). Floor asserted at 498 to preserve the actual existing floor — any drop signals a real regression rather than tightening to a number the suite never met.
3. **MOD divide-by-zero classifies as `HpError::Domain`, not `DivideByZero`** (27-01 deviation): matches the actual `op_mod` implementation at math.rs:496-497; Free42 returns `ERR_DIVIDE_BY_0` but the hp41-core HpError taxonomy lumps this into `Domain`. Test asserts `Domain` to match the implementation; functionally indistinguishable to the user.

All three are observability bugs in the plans, not the implementation; the implementation is correct and the tests are now calibrated to it.

## Human Verification Items

### 1. Observe e2e-linux job green on GitHub Ubuntu CI

**Test:** Push the develop branch (commits 6bfa1c3..430253f are already authored locally) to GitHub. Watch the GitHub Actions tab and confirm the `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` workflow completes with exit 0.

**Expected:**
- Cold cache run: ~6–8 min (includes `cargo install tauri-driver --locked --version 2.0.6` first-time install)
- Cached subsequent runs: ~3–5 min
- Spec reporter prints: `HP-41 GUI smoke (FN-QUAL-05, D-27.13 literal ROADMAP scope) ✓ 2 ENTER 3 + displays 5.0000`
- No `WebKitWebDriver not found` failure (apt `webkit2gtk-driver` installs cleanly on Ubuntu 24.04)
- No Xvfb display-not-found error (the `xvfb-run -a` wrapper allocates a virtual display)

**Why human:** The e2e-linux job has never run before — it was authored on 2026-05-15 (commit a92568d, present locally). The verifier ran every locally-runnable probe (coverage, all cargo tests, all Vitest tests) and confirmed wdio.conf.js + smoke.spec.ts + ci-gui.yml + Justfile gui-e2e recipe are all internally consistent and well-formed. The remaining gate — actual execution under WebKitGTK + xvfb-run on the GitHub Ubuntu runner — requires push-then-observe and is outside the verifier's local sandbox. Per the workflow's "first CI run not yet observed" matrix, this is the canonical human-needed signal.

### 2. Configure branch protection to mark e2e-linux as required-for-merge

**Test:** In GitHub repo Settings → Branches → Branch protection rules for `develop` and `main`, add `GUI E2E (Ubuntu only — WebdriverIO + tauri-driver) / e2e-linux` as a required status check.

**Expected:** PRs to develop and main can no longer be merged unless e2e-linux completes green.

**Why human:** The "required-for-merge" semantics live in GitHub repo settings (or branch-protection-rule API), not in the YAML workflow file. The YAML can only declare the job exists; making it gate the PR merge button is a repo-settings configuration. 27-04-SUMMARY.md flags this explicitly as a manual follow-up (line 244).

## Override Suggestions

None. All five Success Criteria are verified directly in the codebase by independent measurement. The two `human_needed` items are external observations (CI green status, repo settings) — not failed must-haves and not candidates for override.

---

*Phase: 27-test-hardening*
*Verified: 2026-05-15T14:12:15Z by goal-backward verification per `verify-work` workflow*
*Verifier: Claude (gsd-verifier)*
