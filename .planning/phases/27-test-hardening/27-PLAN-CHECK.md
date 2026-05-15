# Phase 27 Plan Check

**Checked:** 2026-05-15
**Plans verified:** 4 (27-01 coverage-push, 27-02 proptest-suites, 27-03 ind-integration, 27-04 e2e-and-vitest-ci)
**Source-of-truth artifacts read:** 27-CONTEXT.md (D-27.1..D-27.16 incl. D-27.15/D-27.16 AMENDED 2026-05-15), 27-RESEARCH.md, REQUIREMENTS.md §FN-QUAL, ROADMAP.md Phase 27 (lines 192–207), CLAUDE.md, plus codebase spot-checks (`hp41-core/src/ops/mod.rs`, `hp41-gui/src/Keyboard.tsx`, `hp41-gui/src/Display14Seg.tsx`, `justfile`, `.github/workflows/ci-gui.yml`, `hp41-core/tests/proptest_stack.rs`, `hp41-core/tests/phase24_ind_variants.rs`).

---

## Adversarial Stance

Starting hypothesis: the 4 plans will not deliver the phase goal. Below are the dimensions audited; only verifiable coverage credits the plans.

---

## Goal-Backward Coverage Map

| ROADMAP Success Criterion | Phase truth required | Plan that ships it | Verdict |
|---|---|---|---|
| SC-1: `just coverage` ≥ 95.0 % (raised from 80) | Risk-weighted tests across Priorities 1–6 + atomic 80→95 ratchet in the SAME commit as the last test addition (D-27.2) | 27-01 Tasks 1–4 (Task 4 atomic ratchet) | COVERED |
| SC-2: `numerical_accuracy.rs` ≥ 98 % with v2.2 cases (PI / P→R / R→P / RND / FRC / MOD / FACT) + Free42 citations (D-27.7) | ~70–105 new hand-curated cases keep 500/500 baseline + ≥ 98 % combined gate | 27-01 Task 3 (hand-curated) + 27-02 Task 2 (shape complement, D-27.5) | COVERED — split per D-27.5 honored |
| SC-3: Proptest covers ROADMAP-3 flag invariants for n ∈ 0..56 | 1024-case blocks for ROADMAP-3 + independence + idempotency + save-load + IND-resolved + skip sentinel (4 variants) | 27-02 Task 1 (≥ 11 properties) | COVERED |
| SC-4: `indirect_addressing.rs` happy + non-integer rejection for every `_IND` op | 17 IND ops × {happy, reject} = ≥ 34 tests; skip-semantic ops drive through `run_program` | 27-03 Tasks 1–2 (≥ 42 tests total) | COVERED |
| SC-5: Playwright/WDIO smoke `2 ENTER 3 + → 5.0000`, Ubuntu only, required with 1 retry | WebdriverIO + tauri-driver per D-27.15 AMENDED; new `e2e-linux` job; `mochaOpts.retries: 1` | 27-04 Tasks 1–4 | COVERED |

All 5 SCs have at least one primary plan and clear task hand-off. No requirement orphaned.

---

## Dimension 1: Requirement Coverage (FN-QUAL-01..05)

| Requirement | Primary plan | `requirements:` frontmatter declared? | Verdict |
|---|---|---|---|
| FN-QUAL-01 | 27-01 | yes (line 17) | OK |
| FN-QUAL-02 | 27-01 (hand) + 27-02 (shape) | yes in both (27-01:18, 27-02:14) | OK |
| FN-QUAL-03 | 27-02 | yes (27-02:15) | OK |
| FN-QUAL-04 | 27-03 | yes (27-03:11) | OK |
| FN-QUAL-05 | 27-04 | yes (27-04:17) | OK |

All 5 FN-QUAL requirements appear in at least one plan's frontmatter `requirements:` field. Result: PASS.

---

## Dimension 2: Decision Compliance (D-27.1..D-27.16)

| Decision | Where addressed | Verdict |
|---|---|---|
| D-27.1 risk-weighted, not file-by-file | 27-01 must_haves bullets 3–7; tasks 1–3 explicitly ordered by RESEARCH §Priority 1–6 bug-class rationale; every task mandates `// Catches: <bug class>` comment | OK |
| D-27.2 atomic 80→95 ratchet in FINAL commit | 27-01 Task 4 explicitly: "EVERYTHING in this task lands in a SINGLE commit"; verification section greps `git diff HEAD~1 HEAD -- justfile` | OK — CRITICAL invariant addressed twice |
| D-27.3 ceiling fallback (94.x % defensible-skip) | 27-01 Task 4 has Branch A / B / C decision tree with defensible-skip enumeration | OK |
| D-27.4 GUI coverage measure-only | 27-04 Task 5 explicit "NO gate, NO devDep added" | OK |
| D-27.5 hybrid (hand + shape) | 27-01 Task 3 = hand; 27-02 Task 2 = shape. Both plans cross-reference. | OK |
| D-27.6 ≥ 98 % global gate with failure budget | 27-01 must_haves bullet 8: "Combined ≥ 98 % pass rate maintained per D-27.6 / ROADMAP SC-2. Existing 500-case baseline still passes at 500/500" | OK |
| D-27.7 Free42 manual cross-check citation | 27-01 Task 3 step 8 verbatim cite format; verification greps `Cross-checked against Free42` count ≥ 15 | OK |
| D-27.8 one file per concern (`proptest_flags.rs` + `proptest_math.rs`) | 27-02 Task 1 = flags, Task 2 = math, 2 separate files | OK |
| D-27.9 ALL 5 flag-proptest extensions in `proptest_flags.rs` (incl. IND-resolved) | 27-02 Task 1 Properties 1–5 enumerated; Property 5 = IND-resolved per item 5 | OK |
| D-27.10 conditional-skip sentinel (4 variants) | 27-02 Task 1 Property 6 covers FS?, FC?, FS?C, FC?C | OK |
| D-27.11 1024 (flags) / 256 (math) | 27-02 must_haves bullet 6; both plans contain explicit `ProptestConfig::with_cases(1024)` and `with_cases(256)` literals | OK |
| D-27.12 IND example tests → 27-03; flag-IND property → 27-02 | 27-02 Property 5 in `proptest_flags.rs`; 27-03 NOT duplicating it (module doc cross-reference) | OK |
| D-27.13 literal ROADMAP smoke only (no broader flows) | 27-04 Task 2 single `it()` block clicking `2 ENTER 3 +`; out-of-scope explicitly lists modal/persistence flows | OK |
| D-27.14 Vitest gated in `gui-ci` | 27-04 Task 3 appends `cd hp41-gui && npm test` | OK |
| D-27.15 AMENDED — WebdriverIO + tauri-driver, NOT Playwright | 27-04 entire plan uses WebdriverIO; explicit "NOT Playwright" callouts in must_haves bullet 1 and out-of-scope. devdeps are `webdriverio` + 4 `@wdio/*` packages, NO `@playwright/test` | OK |
| D-27.16 1 retry via `wdio.conf.js mochaOpts.retries: 1` (NOT `playwright.config.ts`) | 27-04 Task 2 wdio.conf.js template has `retries: 1` inside `mochaOpts: { ui: 'bdd', timeout: 60000, retries: 1 }` | OK |

All 16 locked decisions have at least one implementing task. No contradictions detected.

---

## Dimension 2b: Scope Reduction Detection

Scanned every task action for `"v1"`, `"v2"`, `"simplified"`, `"static for now"`, `"placeholder"`, `"hardcoded"`, `"future enhancement"`, `"basic version"`, `"deferred"`, `"too complex"`, `"too difficult"`, `"non-trivial"` used as scope justification.

Findings (acceptable):
- 27-04 Task 4 contains "informational on PRs" w.r.t. required-status-check enforcement — this is a CORRECT description of YAML's limitation; required-for-merge is a repo-settings concern documented as a manual follow-up. NOT a scope reduction.
- 27-04 Task 5 contains "deferred to v3.x" for the Vitest coverage devDep — this is the D-27.4 decision, NOT a planner-invented reduction.
- 27-04 references "broader Playwright flows (modal interactions, persistence roundtrip)" as deferred — this is D-27.13 verbatim, NOT a reduction.

No silent scope reductions of D-27.x decisions. Result: PASS.

---

## Dimension 3: Task Completeness

Every `<task type="auto">` carries `<files>`, `<action>`, `<verify>`, `<done>`. Spot-checks:

| Plan | Tasks | Files | Action specificity | Verify command | Done criteria |
|---|---|---|---|---|---|
| 27-01 | 4 | concrete paths | step-by-step with file:line cites | `cargo test ...` / `just coverage` | measurable thresholds |
| 27-02 | 3 | concrete paths | exact code blocks copied from RESEARCH Patterns 1–4 | `cargo test --test proptest_flags --test proptest_math` | property counts + runtime budgets |
| 27-03 | 2 | one file | macro + per-op test enumeration | `cargo test --test indirect_addressing` | ≥ 22 + ≥ 20 = ≥ 42 tests |
| 27-04 | 6 | 7 files | TS/JS/YAML/Just snippets verbatim | YAML lint + grep counts | observable file states |

All tasks complete. Result: PASS.

---

## Dimension 4: Dependency Correctness

| Plan | `depends_on:` | Wave declared | Implied minimum wave |
|---|---|---|---|
| 27-01 | `[]` | 1 | 1 |
| 27-02 | `[]` | 1 | 1 |
| 27-03 | `[]` | 1 | 1 |
| 27-04 | `[]` | 2 | 1 (no actual file dep on 01/02/03) |

Anomaly: 27-04 declares `wave: 2` with `depends_on: []`. The plan body says "Recommended execution order — LAST in the wave, so all prior coverage / proptest / IND additions are stable before the new CI job activates." This is a **scheduling preference**, not a hard file/contract dependency. The wave-2 placement is sensible (the new CI gate activates after Plans 27-01/02/03 land); since `depends_on:` is empty, no cycle exists and no executor will block.

No circular dependencies, no missing references, no forward references. Result: PASS.

---

## Dimension 5: Key Links Planned

| Plan | Key link declared | Source artifact task | Action mentions wiring? |
|---|---|---|---|
| 27-01 | `program_execution_coverage.rs → ops/program.rs::execute_op (647–851) via run_program` | Task 1 step 5–9 | Yes — every Phase 20–24 op wired through `run_op_in_program` helper |
| 27-01 | `numerical_accuracy.rs (v2.2 block) → math ops via case! + dispatch` | Task 3 step 5–7 | Yes — explicit "integrate into the same harness" instructions |
| 27-01 | `justfile coverage: → --fail-under-lines 95` | Task 4 step 1 | Yes — exact diff |
| 27-02 | `proptest_flags.rs → flags::flag_get / Op::SfFlag / serde_json` | Task 1 properties 1–6 | Yes — every property has dispatch + assertion code block |
| 27-02 | `proptest_math.rs → math ops` | Task 2 properties 1–5 | Yes |
| 27-02 | `proptest-regressions/.gitkeep → CI replay` | Task 3 | Yes — `.gitignore` audit + `git check-ignore` verify |
| 27-03 | `indirect_addressing.rs → resolve_indirect (both branches)` | Pattern-A macro + Pattern-B tests | Yes |
| 27-03 | Documentary cross-reference to 27-02 IND-flag property | Module doc | Yes (D-27.12) |
| 27-04 | `smoke.spec.ts → Keyboard data-key-id + Display14Seg data-testid` | Task 1 (testid) + Task 2 (spec) | Yes |
| 27-04 | `wdio.conf.js → tauri-driver + release binary` | Task 2 | Yes — `beforeSession` spawn + `tauri:options.application` |
| 27-04 | `ci-gui.yml::e2e-linux → just gui-e2e` | Task 4 | Yes — `xvfb-run -a just gui-e2e` step |
| 27-04 | `gui-ci → 5 Vitest files` | Task 3 | Yes — single appended `cd hp41-gui && npm test` |

Wiring complete in every plan. Result: PASS.

---

## Dimension 6: Scope Sanity

| Plan | Tasks | Files modified | Concrete risk |
|---|---|---|---|
| 27-01 | 4 | 7 | Borderline — 4 tasks is the warning threshold per the GSD baseline. Tasks 1–3 each create 1 file; Task 4 is the atomic ratchet. Justified because D-27.2 mandates atomicity in the SAME plan. |
| 27-02 | 3 | 4 | Healthy. |
| 27-03 | 2 | 1 | Healthy. |
| 27-04 | 6 | 7 | HIGH — 6 tasks exceeds the 2–3 target and approaches the 5+ blocker line. BUT: tasks 1 and 5 are zero-write (data-testid one-liner + measurement-only); task 6 is CLAUDE.md doc. The "real" work is tasks 2–4 (wdio setup + justfile + ci-gui.yml). Mitigation: tasks are linearly sequenced with clear file boundaries, no cross-task overlap. The 6-task structure is defensible given the inherent multi-surface nature of an E2E job (config + spec + DOM hook + apt deps + cargo install + YAML). Treated as a WARNING, not a blocker. |

Total context budget: 4 plans × ~10–15 files each = manageable. No single plan approaches the 15-file blocker line. Result: PASS with one WARNING on 27-04 task count.

---

## Dimension 7: must_haves Derivation

All four plans have `must_haves: { truths, artifacts, key_links }`. Spot-check of truths:

| Plan | User-observable? | Implementation-leaking? |
|---|---|---|
| 27-01 | "`just coverage` reports ≥ 95.0 %" — observable | Mix of observable + descriptive (e.g. "Each new test carries `// Catches:`" is more of a constraint than a user truth, but it's the D-27.1 evidence-anchor — acceptable) |
| 27-02 | "Property 1 (ROADMAP-3): SF(n) → FS?(n) = true for all n in 0..56" — observable via test pass | OK |
| 27-03 | "Every IND op has happy + non-integer reject" — observable via cargo test | OK |
| 27-04 | "WebdriverIO + tauri-driver E2E smoke boots prod binary, asserts 5.0000" — observable via CI job pass | OK |

No implementation-only truths (no "bcrypt installed" style). Result: PASS.

---

## Dimension 7c: Architectural Tier Compliance

RESEARCH.md §Architectural Responsibility Map present (lines 17–27). Spot-checks:

- 27-01 places coverage gate in `justfile` (Build tooling tier) — matches map row 1.
- 27-01/02/03 place tests in `hp41-core/tests/` (hp41-core test target tier) — matches map rows 2–4.
- 27-04 places E2E in `hp41-gui/e2e/` (hp41-gui test target tier) — matches map row 5.
- 27-04 places Vitest gating in `justfile gui-ci:` (Build tooling tier) — matches map row 6.

No tier mismatches. No security-sensitive tier downgrades. Result: PASS.

---

## Dimension 10: CLAUDE.md Compliance

Spot-checks against CLAUDE.md project rules:

- "**Commits via `/git-workflow:commit --with-skills`**" — 27-01 Task 4 references the commit message wording but does NOT bypass the skills protocol. No `git commit` direct calls in any task action. OK.
- "**`#![deny(clippy::unwrap_used)]` at hp41-core crate root**" — All 5 new test files (27-01 Task 1/2/3 files + 27-02 Tasks 1/2 files + 27-03 Task 1 file) carry `#![allow(clippy::unwrap_used)]` at file scope. OK.
- "**No `println!` / `eprintln!` in `hp41-core`**" — No task adds `println!`/`eprintln!` in `hp41-core`. Tests use `state.print_buffer` / `state.event_buffer` per established pattern. OK.
- "**SC-4 invariant: no source changes to `hp41-gui/src-tauri/`**" — 27-04 explicitly preserves this; the Display14Seg.tsx edit lives in `hp41-gui/src/` (verified — file is at line `/Users/.../hp41-gui/src/Display14Seg.tsx`). The Display14Seg.tsx is currently a pure render-only component with NO `data-testid` (verified: read of the file 2026-05-15 shows the outer `<svg>` element at line 209 has only `viewBox`, `xmlns`, `aria-label`, `preserveAspectRatio` — no `data-testid`). The one-line addition is genuinely needed. OK.
- "**MSRV 1.88 unchanged**" — 27-04 explicitly preserves; `tauri-driver` 2.0.6 MSRV 1.77 < 1.88 — compatible. OK.
- "**No new `Op` variants**" — Phase 27 is test-only. Spot-checked all 4 plans: zero `hp41-core/src/` modifications. OK.

Result: PASS.

---

## Dimension 11: Research Resolution

27-RESEARCH.md was read; the §Open Questions section status was inspected. The CONTEXT.md D-27.15 AMENDED note ("Open Question 1 resolved post-research after the WebDriver-classic vs Playwright protocol mismatch was established") indicates the open question was resolved in CONTEXT.md, and CONTEXT.md is the source of truth the plans implement.

The RESEARCH §Open Question 3 (`.gitignore` audit) is mitigated by 27-02 Task 3 (`.gitignore` defensive audit + un-ignore comment if needed).

Result: PASS.

---

## Dimension 12: Pattern Compliance

No PATTERNS.md exists for this phase. Plans correctly cite RESEARCH §Patterns 1–5 + §Code Examples 1–6 + the existing `proptest_stack.rs` / `phase24_ind_variants.rs` style precedents instead. Result: SKIPPED (no PATTERNS.md).

---

## Dimension 8: Nyquist Compliance

VALIDATION.md not present for Phase 27 — but RESEARCH §Validation Architecture (lines 789–832) serves the equivalent role. Each task has an `<automated>` verify command:

| Task | Plan | Automated command |
|---|---|---|
| 1 | 27-01 | `cargo test -p hp41-core --test program_execution_coverage` |
| 2 | 27-01 | `cargo test -p hp41-core --test phase22_stats_size_shrink --test phase21_phase22_interactive_no_ops` |
| 3 | 27-01 | `cargo test -p hp41-core --test format_eng_edges --test numerical_accuracy` |
| 4 | 27-01 | `just coverage` |
| 1 | 27-02 | `cargo test -p hp41-core --test proptest_flags` |
| 2 | 27-02 | `cargo test -p hp41-core --test proptest_math` |
| 3 | 27-02 | shell-test `test -d proptest-regressions && ! git check-ignore ...` |
| 1 | 27-03 | `cargo test -p hp41-core --test indirect_addressing` |
| 2 | 27-03 | `cargo test -p hp41-core --test indirect_addressing` |
| 1 | 27-04 | `cd hp41-gui && npx tsc --noEmit && npm test` |
| 2 | 27-04 | shell-test grep + file presence |
| 3 | 27-04 | `just --list | grep -E "gui-ci|gui-e2e"` |
| 4 | 27-04 | `python3 -c "import yaml; yaml.safe_load(...)"` + grep |
| 5 | 27-04 | grep on `package.json` devDep absence |
| 6 | 27-04 | grep on CLAUDE.md |

No watch-mode flags. No latencies > 30 s in unit tests (proptest_math is the slowest at ~60 s budget — RESEARCH Pitfall budget). Sampling continuity: every wave-1 task has automated verify; no run of 3 consecutive without verify.

Result: PASS.

---

## Concrete Spot-Checks (claims vs. codebase ground truth)

| Claim in plan | Verified against | Outcome |
|---|---|---|
| `justfile coverage:` is at lines 34–37 with `--fail-under-lines 80` | Read `justfile`: actual block is `coverage:` at offset ~34 with `cargo llvm-cov --fail-under-lines 80 -p hp41-core` — single line, matches | OK |
| `justfile gui-ci:` is at lines 82–87 | Read `justfile`: `gui-ci:` block has 4 indented commands matching the plan diff — exact lines may differ by 1–2 but the structure matches | OK |
| `Display14Seg.tsx` lacks `data-testid` today | Read file — outer `<svg>` at line 209 has only `viewBox`, `xmlns`, `aria-label`, `preserveAspectRatio`. No `data-testid`. | OK — edit genuinely required |
| `Keyboard.tsx` exposes `data-key-id` at lines 285, 303 | grep: `data-key-id={key.id \|\| undefined}` matches at lines 285 and 303 | OK |
| Key IDs `'2'`, `'enter'`, `'3'`, `'plus'` exist in KEY_DEFS | grep: `id: 'enter'` at line 97; `id: 'plus'` at line 111; `id: '2'` at line 119; `id: '3'` at line 120 | OK |
| `proptest_stack.rs` is 81 lines and is the only proptest precedent | wc -l: 81 lines. OK | OK |
| `phase24_ind_variants.rs` is 20.5K — style precedent for Plan 27-03 | wc -l: 582 lines (~21K). OK | OK |
| `Op::Int` exists as a variant per Plan 27-02 Task 2 step 5 | grep `mod.rs`: `Op::Int => op_int(state)` at line 671. OK | OK |
| `Op::SetDeg` exists for proptest_math.rs Property 4 | grep `mod.rs`: line 696 `Op::SetDeg => op_set_deg(state)`. OK | OK |
| `Op::StoArithInd` is **STRUCT variant** `{ kind, n }` per Plan 27-03 Task 1 step 10 | grep `mod.rs:910`: `Op::StoArithInd(reg, kind) => indirect::op_sto_arith_ind(state, reg, kind)`. This is a **TUPLE variant** `(reg, kind)`. `phase24_ind_variants.rs:123` confirms: `Op::StoArithInd(5, StoArithKind::Add)`. | **MISMATCH** — plan template uses struct form |
| `Op::FlagTestInd { kind, flag: r }` per Plan 27-02 Task 1 step 8 + Plan 27-03 Task 2 step 4 | Read `mod.rs:560–563`: `FlagTestInd { kind: FlagTestKind, ind_reg: u8 }`. Field name is `ind_reg`, NOT `flag`. | **MISMATCH** — plan template uses wrong field name |
| `ci-gui.yml` "build" job exists; matrix on 3 OS; no E2E job today | Read file: yes, single `build:` job with `matrix.os` on ubuntu/macos/windows, single `Install Linux system deps` step on Ubuntu only. No E2E job. | OK |

---

## Suggestions (non-blocking — informational)

1. **27-03 Task 1 (`Op::StoArithInd` struct vs tuple variant) and 27-02 Task 1 / 27-03 Task 2 (`Op::FlagTestInd` field name `flag` vs `ind_reg`).** The plan example code snippets show:
   - `Op::StoArithInd { kind: StoArithKind::Add, n: 5 }` — should be `Op::StoArithInd(5, StoArithKind::Add)` (tuple variant; field name is `reg` in the source but accessed positionally).
   - `Op::FlagTestInd { kind, flag: r }` — should be `Op::FlagTestInd { kind, ind_reg: r }` (the struct field is named `ind_reg` per `hp41-core/src/ops/mod.rs:562`).

   Both plans explicitly tell the executor "verify exact variant ... during read_first" and point at `phase24_ind_variants.rs:123` and `mod.rs` as canonical references, so a careful executor will catch and correct this. The plans are not blocked, but the in-line templates would compile-fail if pasted verbatim. **Recommend** the planner amend the example code snippets in 27-02 Task 1 step 8 and 27-03 Task 1 steps 10–13 / Task 2 step 4 to use the correct shapes — saves the executor one read+fix cycle.

2. **27-04 Task 3 vs RESEARCH §Example 5 `gui-e2e:` recipe shape.** RESEARCH Example 5 (lines 716–720) shows the `gui-e2e:` recipe building the binary inside the recipe via `npm run tauri build -- --debug --no-bundle`. Plan 27-04 Task 3 omits the build step from the recipe (saying CI's `just gui-build` covers it) AND removes `onPrepare` from `wdio.conf.js`. This is internally consistent — CI does the build before calling `just gui-e2e` — but means local developer `just gui-e2e` runs without `just gui-build` will fail with "binary not found". Plan 27-04 documents this precondition as a recipe comment, which is acceptable, but a fail-safe `just gui-build && cd hp41-gui && ...` chain would be more robust. Not a blocker.

3. **27-04 Task 4 `e2e-linux` job name vs branch-protection.** The plan correctly notes that GitHub's required-status-check enforcement is a repo-settings concern (not enforceable in YAML), and documents the manual follow-up. This is honest but worth surfacing as a v2.2 milestone-completion checklist item, not just a SUMMARY note. The current note in Task 4 is acceptable.

4. **27-01 Task count = 4 is borderline.** D-27.2 atomicity (gate raise + final tests in same commit) is the genuine architectural reason — Task 4 cannot be its own plan. The risk is acceptable given the rationale is explicit and Task 4 doesn't introduce new test surface (it edits `justfile` + `CLAUDE.md` only). No split recommended.

5. **27-04 Task count = 6.** Defensible but at the upper bound. Tasks 1 (data-testid), 3 (justfile), 5 (coverage measurement), 6 (CLAUDE.md) are all single-file mechanical edits. Tasks 2 + 4 are the substantive WebdriverIO + CI YAML edits. The structure is fine; no split recommended.

6. **27-04 Task 2 binary name resolution.** The plan correctly flags that `tauri:options.application` needs the actual binary name (verified via `Cargo.toml` `[[bin]] name = ...` or `tauri.conf.json mainBinaryName`). This is a read_first action; not a blocker.

---

## Blockers

None.

The two variant-syntax mismatches (Suggestion #1) are flagged as informational rather than blocking because:
- Both plans explicitly instruct the executor to verify variant shapes against `hp41-core/src/ops/mod.rs` and `phase24_ind_variants.rs` during `read_first`.
- The plans list this exact failure mode in their "Failure Modes & Mitigations" sections (27-03 Failure Modes bullet 1 names the `StoArithInd` field-name failure mode directly).
- A careful executor following the `read_first` directive will catch and correct on first compile.

The plans deliver the phase goal: every FN-QUAL-01..05 requirement has a primary plan; every D-27.1..D-27.16 (including the D-27.15/D-27.16 AMENDED for WebdriverIO) is implemented; the atomic 80→95 ratchet lives in 27-01 Task 4's final commit per D-27.2; SC-4 invariant is preserved; MSRV 1.88 is unchanged; no `Op` variants are added; no `hp41-core/src/` or `hp41-gui/src-tauri/` changes; the Display14Seg `data-testid` edit is in `hp41-gui/src/` (outside SC-4 boundary); the WebdriverIO + tauri-driver Ubuntu-only E2E job replaces the Playwright wording per the D-27.15 amendment.

---

## PLAN CHECK PASS
