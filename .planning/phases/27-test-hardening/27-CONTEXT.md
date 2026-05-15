# Phase 27: Test Hardening — Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 27 is the final phase of milestone v2.2. It restores `hp41-core` line coverage to the v1.0 high-water-mark target of **≥ 95.0%** (currently 93.59% lines / 91.21% regions per `cargo llvm-cov -p hp41-core` baseline measured 2026-05-15), extends the existing 500-case `numerical_accuracy.rs` suite with cases for every new v2.2 math/conversion op, lands proptest-driven flag-semantics invariants across the 56 user flags, ships a focused `hp41-core/tests/indirect_addressing.rs` integration suite, and adds a literal-ROADMAP-smoke Playwright E2E job to `ci-gui.yml`. As a quietly-relevant CI hygiene improvement that surfaced during discussion, this phase also gates the existing 5 Vitest test files (App, Display14Seg, HelpOverlay, Keyboard, pending_input) in `gui-ci` — they exist, pass locally, and are NOT run on CI today.

**In scope:**
- FN-QUAL-01: `hp41-core` line coverage ≥ 95% gate (`just coverage` raised from `--fail-under-lines 80` to `--fail-under-lines 95`)
- FN-QUAL-02: numerical accuracy extension for `PI`, `P→R`, `R→P`, `RND`, `FRC`, `MOD`, `FACT` — hybrid hand-curated edges + proptest shape invariants, ≥ 98% pass rate on the new total
- FN-QUAL-03: flag-semantics proptest covering ROADMAP 3 invariants + independence + idempotency + save-load roundtrip + IND-resolved-flag semantics + run_loop skip-next-step sentinel
- FN-QUAL-04: `hp41-core/tests/indirect_addressing.rs` happy-path + non-integer-rejection for every `_IND` op
- FN-QUAL-05: Playwright E2E in `ci-gui.yml` — literal ROADMAP smoke (`2 ENTER 3 + → 5.0000`), production build + `tauri-driver`, Ubuntu only, required job with 1 retry
- CI hygiene: Vitest `npm test` added to the `gui-ci` recipe in `justfile`

**Out of scope (explicit):**
- Any `hp41-core` source changes — Phase 27 is purely test + gate + CI work. No new `Op` variants. No new `CalcState` fields.
- Any `hp41-cli` / `hp41-gui` feature changes — Phase 26 closed the GUI integration loop; Phase 27 only adds tests on top.
- GUI coverage as an enforced gate — Vitest coverage and `cargo llvm-cov` on `hp41-gui/src-tauri/` are measure-only this phase, NOT CI-gated (FN-QUAL-01 explicitly targets `hp41-core` only).
- Free42-driven fixture generation harness — manual cross-check with citation in test doc comments only (avoids licensing/dependency complexity).
- Broader Playwright flows (modal interactions, autosave persistence roundtrip) — explicit deferral; only the ROADMAP literal smoke ships in this phase.
- Module-Pac emulation (Math 1 / Stat 1 / Time / Advantage) — permanent v3.x boundary per REQUIREMENTS.md.

**Mandated by ROADMAP cross-cutting constraints (lines 200–207 of ROADMAP.md):**
- Coverage gate raise (80% → 95%) is atomic with test additions, OR CI fails. The 5 success criteria are independently observable.
- Proptest cases ≤ 256 iterations per case for slow properties; 1024 OK for fast flag invariants.
- Playwright runs **only on Ubuntu** in `ci-gui.yml`. macOS/Windows runners are slow and headless support is weaker. Document the Linux-only scope in CLAUDE.md.
- No new `Op` variants in this phase.
- `#![deny(clippy::unwrap_used)]` continues to apply. Test modules carry `#[allow(clippy::unwrap_used)]` at the test mod level (existing pattern).

</domain>

<decisions>
## Implementation Decisions

### Coverage strategy (D-27.1 — D-27.4)

- **D-27.1: Risk-weighted hot spots first.** Target the highest-risk uncovered paths in order of bug-catching value, NOT a file-by-file mechanical sweep. Concrete priorities measured against the 2026-05-15 `cargo llvm-cov -p hp41-core` baseline:
  1. `ops/program.rs` — `resolve_indirect()` error branches (non-integer rejection, out-of-range register), `run_loop` conditional-skip arms, synthetic dispatch arms in `execute_op()`
  2. `ops/stats.rs` — currently the lowest-coverage module at 84.04% lines; statistics ops have stack-lift edge cases (Σ+ on empty registers, MEAN with zero count) that haven't been exhaustively tested
  3. `ops/mod.rs` — synthetic dispatch arms (HexModal byte sequences, GETKEY edge paths) the v1.1 baseline never fully covered
  4. `ops/registers.rs` — STO arithmetic stack-register variants on uninitialized/poisoned state
  5. Error-path coverage across the crate — every `HpError::*` variant should have at least one test that produces it. Region coverage (91.21%) lags lines because of unexercised match arms.
  - **Why:** the user explicitly chose "tests that catch real bugs, not coverage padding." HP-41 hardware-error semantics are a known regression-risk area. The risk-weighted ordering is also the ordering most likely to surface NEW bugs before the next release rather than just decorating the metric.

- **D-27.2: Single hard flip 80 → 95 in `justfile`, atomic with final test additions.** No mid-phase staged ratchet (e.g. 80 → 93 → 95). The final coverage commit raises `cargo llvm-cov --fail-under-lines 80` to `--fail-under-lines 95` AND lands the last batch of tests in one commit. If `just coverage` doesn't hit 95% locally after the last commit, the phase isn't done.
  - **Why:** matches ROADMAP success criterion verbatim. Eliminates "did we forget to raise the gate?" ambiguity. Atomicity means a revert in either direction is a single revert.

- **D-27.3: Ceiling fallback — adjust ROADMAP target down with a documented deviation if 95% is genuinely unreachable without trivial padding tests on cosmetic code (e.g. `Debug` impls, `panic-on-impossible` branches the type system already forbids).** Concretely: the planner adds risk-weighted tests until either (a) `just coverage` reports ≥ 95.0% lines, or (b) the planner identifies a defensible ceiling at e.g. 94.5% with every meaningful path covered. In case (b), Phase 27 ships with an updated ROADMAP/REQUIREMENTS gate at the achieved-and-defensible number AND a documented deviation note. Padding-only tests are NOT acceptable.
  - **Why:** prevents the worst failure mode — writing low-signal tests to game a number. Honest about the realistic ceiling of a Rust crate with `#[derive(Debug)]` on lots of types. The user explicitly chose this over either "pad to hit 95" or "annotate with `#[cfg(not(coverage))]`" — the latter is a knob worth using sparingly but only after the risk-weighted push reveals what's left.

- **D-27.4: GUI coverage is out of scope for Phase 27 — measure only, no CI gate.** As a one-time snapshot during planning the planner may run `vitest --coverage` on `hp41-gui/src` and `cargo llvm-cov --manifest-path hp41-gui/src-tauri/Cargo.toml` and record the numbers in the SUMMARY for future reference. NO threshold is added to `gui-ci`. NO coverage provider dependency is added to `hp41-gui/package.json`.
  - **Why:** FN-QUAL-01 explicitly targets `hp41-core`. Phase 27 is already adding a Playwright job + Vitest CI gate — adding GUI coverage gates is scope creep that doesn't align with the locked requirement. The planner has discretion to record the measured numbers in CLAUDE.md as a v3.x-roadmap signal, but the CI surface stays focused on `hp41-core` ≥ 95%.

### Accuracy suite shape (D-27.5 — D-27.8)

- **D-27.5: Hybrid extension — hand-curated edges + proptest shape invariants.** Add ~10–15 hand-curated test cases per v2.2 math op (PI, P→R, R→P, RND, FRC, MOD, FACT — 7 ops) sourced from the HP-41C Owner's Manual + Free42 cross-check, totaling ~70–105 new cases in `numerical_accuracy.rs`. PLUS a new `proptest_math.rs` module that asserts shape invariants without specific numeric outputs:
  - `FRC(x) + Integer-part(x) ≈ x` (FRC + INT round-trip)
  - `MOD(y, x) sign follows sign of y` (HP-41 sign semantics)
  - `FACT(n+1) ≈ FACT(n) * (n+1)` for n in 0..68
  - `P→R(magnitude=R, angle=θ)` then `R→P` returns `(R, θ)` within finite-precision tolerance
  - `RND` is idempotent (RND ∘ RND = RND) for any display-mode setting
  - **Why:** the user picked "hybrid" over both extremes. Hand-curated catches known HP-41-specific quirks (10-digit rounding artifacts, MOD(7, -3) = 1 not -2, FACT(0) = 1); proptest catches unknown regressions in shape invariants that a 105-case suite can never fully enumerate.

- **D-27.6: Acceptance gate ≥ 98% on the new total.** The existing 500/500 v2.1-baseline cases stay at 500/500 (any regression there is a hard fail). The new ~100 cases combine with the existing 500 into a ~600-case total; the gate at ≥ 98% allows up to ~12 acceptable failures across the combined suite (1% over 600 ≈ 6 cases; ROADMAP says ≥ 98%). HP-41 hardware quirks that diverge from f64-ideal expectations (e.g. transcendental-function-rounding boundaries) are the expected failure budget. Each acceptable failure must be cited in a test-doc comment with the divergence reason — silent failures are NOT acceptable.
  - **Why:** the user picked the global gate over either "100% on new cases" (too strict given known HP-41 quirks in trig) or "per-op buckets" (more bookkeeping than the bug-catching value justifies for 7 ops).

- **D-27.7: Manual Free42 cross-check, cite source location in test doc comments.** When deriving a quirky hand-curated case the planner runs the input through Free42 (or reads its source — Free42 is open-source at https://thomasokken.com/free42/) and adds a comment to the relevant test like `// Cross-checked against Free42 source ops_math.cc::do_mod — Free42 returns 1 for MOD(7, -3), matching HP-41C Owner's Manual p.234`. NO automated Free42-fixture-generation harness is built. NO Free42 binaries are added to the repo.
  - **Why:** the user picked the cheap-defensible option over either expensive fixture generation or strict "manual only, no Free42". Cited cross-references are auditable. Avoids license/distribution complexity.

- **D-27.8: Proptest module location — one file per concern.** Extend the existing 1-file precedent (`hp41-core/tests/proptest_stack.rs`) with TWO new files:
  - `hp41-core/tests/proptest_flags.rs` — FN-QUAL-03 invariants (see D-27.9, D-27.10)
  - `hp41-core/tests/proptest_math.rs` — shape invariants from D-27.5
  - **Why:** the user picked granularity over either "one omnibus proptest.rs" or "inline in phase21_flags.rs / phase20_math.rs". Matches the existing `hp41-core/tests/phase*_*.rs` naming convention. Independent file = independent atomic test run = easier failure attribution.

### Flag proptest scope (D-27.9 — D-27.12)

- **D-27.9: Flag invariants beyond ROADMAP — ALL FOUR extensions land in `proptest_flags.rs`.**
  1. **ROADMAP 3 (mandatory):** `SF(n) ⇒ FS?(n) = true`; `CF(n) ⇒ FC?(n) = true`; `SF(n) → FS?C(n) → FC?(n) = true` for all n in 0..56
  2. **Independence:** for any pair `(m, n)` with `m ≠ n`, `SF(m)` leaves `FS?(n)` unchanged. Catches accidental bit-field overflow / mask bugs.
  3. **Idempotency:** `SF(n) → SF(n)` is equivalent to a single `SF(n)`; same for CF. Catches off-by-one toggle bugs.
  4. **Save-load roundtrip:** generate random flag patterns, `serde_json::to_string(&state) → from_str(...)`, assert all flag states identical. Catches the `#[serde(default)]` backward-compat invariant under random patterns rather than just the v1.x baseline case.
  5. **IND-resolved flag semantics:** generate `(n, r)` where register r holds the integer n as Decimal; assert `SF_IND(r)` leaves flag n in the same state as `SF(n)`. Lives in `proptest_flags.rs` (not in `indirect_addressing.rs`) because it's a property assertion, not a happy-path example test.
  - **Why:** the user explicitly selected all four extensions plus the ROADMAP-mandated three. Each catches a distinct bug class. Independence + idempotency are cheap-to-write but high-value invariants the example tests in `phase21_flags.rs` don't exhaustively cover. Save-load roundtrip is the kind of property where a single failing seed is a real bug.

- **D-27.10: Conditional-skip semantics sentinel proptest.** Add a `proptest_flags.rs` property that generates random short programs of shape `[<flag-test op>, <step A>, <step B>]`, sets/clears the relevant flag, runs the program, and asserts `state.pc` lands on the correct step (A if test passed, B if test failed). Covers `FS?`, `FC?`, `FS?C`, `FC?C` — all four conditional-test variants from Phase 21. The `FS?C` / `FC?C` variants additionally assert post-test flag state.
  - **Why:** the user picked the sentinel proptest over "rely on existing phase21_flags.rs example tests" or "defer to IND integration tests". The skip rule is a single-line decision in `run_loop` but its interaction with the flag-test result + post-test side effect is where conditional-test bugs hide. Proptest randomization explores the (flag-state × test-variant × program-shape) cross-product the example tests sample by hand.

- **D-27.11: Proptest iteration counts — 1024 for flag invariants, 256 for math properties.** Mirrors ROADMAP cross-cutting guidance verbatim. Flag invariants are bit-twiddling on `flags: u64` — 1024 cases cost milliseconds. Math properties involve `rust_decimal` arithmetic and shape-invariant computation — 256 cases keep CI runtime under control. Configure via `proptest::test_runner::Config { cases: 1024, .. Config::default() }` per `proptest!` block.
  - **Why:** the user picked the ROADMAP-aligned split over either "256 across the board" (conservative but leaves rare flag-interaction patterns less likely to be caught) or "tune per-property in code review" (slower iteration). The split is mechanical and obvious from the test-file structure.

- **D-27.12: IND-test layout — NEW `hp41-core/tests/indirect_addressing.rs` + flag-IND proptest in `proptest_flags.rs`.** Per ROADMAP success criterion 4: a dedicated `indirect_addressing.rs` integration suite covers happy-path + non-integer rejection for every `_IND` op (STO, RCL, ISG, DSE, SF, CF, FS?, FC?, FS?C, FC?C, STO+, STO-, STO×, STO÷, ARCL, ASTO, VIEW — total ~17 ops). The flag-specific IND-resolved-semantics property (D-27.9 item 5) goes in `proptest_flags.rs` alongside the other flag properties — it's a property assertion, not an example test, and co-locating it with the flag invariants keeps related tests next to each other. `phase24_ind_variants.rs` (20.5K, exists) stays as the example-test suite for Phase 24; `indirect_addressing.rs` is the Phase 27 hardening complement.
  - **Why:** the user picked the dedicated file over "extend phase24_ind_variants.rs" or "one indirect_addressing.rs containing both paradigms". The ROADMAP explicitly names `indirect_addressing.rs` as the test file. Splitting example tests from properties keeps each test file focused on one paradigm.

### Playwright E2E scope (D-27.13 — D-27.16)

- **D-27.13: Literal ROADMAP smoke only — one Playwright spec.** The spec boots the Tauri production build, clicks `2 ENTER 3 +` on the SVG keyboard, asserts the LCD display reads `5.0000` (or the current display-mode equivalent). NO modal interactions, NO autosave persistence roundtrip, NO multi-spec coverage. Proves the Tauri ↔ React ↔ `hp41-core` dispatch chain works end-to-end on the Ubuntu CI runner.
  - **Why:** the user picked the literal ROADMAP scope over "smoke + one modal flow" or "smoke + modal + persistence". Phase 27 already adds a lot of test surface (coverage push, proptests, IND integration, Vitest gating); the Playwright job's job is to be a tiny canary, not a comprehensive E2E suite. Broader Playwright flows are deferred to v2.3+ if the canary proves stable.

- **D-27.14: Add Vitest tests to `gui-ci`.** The existing `gui-ci` recipe in `justfile` runs `npx tsc --noEmit` + `cargo test`. Phase 27 appends `npm test` (which maps to `vitest run` per `hp41-gui/package.json`). Gates the 5 existing Vitest files (`App.test.tsx`, `Display14Seg.test.tsx`, `HelpOverlay.test.tsx`, `Keyboard.test.tsx`, `pending_input.test.ts`) on every CI run. These tests already pass locally — not gating them is a quiet hole that surfaced during discussion.
  - **Why:** the user picked adding to `gui-ci` over either a separate `gui-ci-vitest` job (more YAML, marginal benefit since they pass quickly) or deferring it as a TODO. The existing tests are a sunk cost — gating them is a one-line change with immediate regression-detection value.

- **D-27.15: Playwright launch mode — production build + `tauri-driver` (WebKitGTK on Ubuntu).** Build the Tauri app with `just gui-build` (release binary), then drive it via `tauri-driver` connected through Playwright's webdriver protocol. Matches the Tauri team's official E2E pattern (https://tauri.app/v2/develop/tests/webdriver/). Slower job (~3–5 min on the GitHub Ubuntu runner) but tests the actual Tauri runtime + IPC layer + React frontend, NOT a mocked Vite dev server with stubbed `invoke()` calls.
  - **Why:** the user picked the production-build-driven approach over either "dev server via Vite" (skips the IPC layer that's the whole point of an E2E test) or "Microsoft Edge WebDriver" (works on Tauri but WebKitGTK on Ubuntu is the more conventional Linux-runner setup).

- **D-27.16: Required job with 1 retry.** Mark the Playwright job required for merge in `ci-gui.yml`. Configure `retries: 1` in `playwright.config.ts` so transient WebKitGTK / X server startup hiccups don't fail the PR, but a real regression (e.g. broken dispatch path) still does (retry would also fail). Standard Playwright pattern.
  - **Why:** the user picked the balanced policy over either "required, no retries" (strict but flake-sensitive) or "continue-on-error initially" (soak period — slower path to confidence). With 1 retry, transient infra issues self-heal; deterministic regressions still surface.

### Claude's Discretion

- **Specific uncovered lines to target first within the risk-weighted priorities.** D-27.1 names the priority files in order; planner finalizes the specific functions and branches based on a fresh `cargo llvm-cov --html` pass during planning. The planner may also choose to start with `ops/stats.rs` (currently 84%, easiest big win) if early data shows the bigger files would need more bookkeeping. **What matters:** the rationale for each chosen target is recorded in the plan (catches bug class X, not "lifts coverage of file Y").
- **Specific hand-curated test cases for each math op.** D-27.5 sets the per-op budget at ~10–15 cases. Planner picks the specific inputs (likely sourcing from HP-41C Owner's Manual Chapter 3 — Math, Free42's `core_math.cc` test fixtures if any are publicly shipped, and the existing `numerical_accuracy.rs` taste-of-style).
- **Proptest strategy combinators.** Planner picks `prop_oneof!` shapes for ops, magnitudes for math inputs (within HP-41-realistic ranges, e.g. exponents -99..+99), and shrink strategies. The existing `proptest_stack.rs::arb_simple_op()` is the style precedent.
- **`tauri-driver` vs `WebKitWebDriver` package selection.** Tauri 2.11 ships `tauri-driver` as a crate; the Ubuntu runner needs the `webkit2gtk-driver` apt package. Planner confirms the exact apt package name vs `xvfb-run` wrapper requirement during planning. The existing `ci-gui.yml` already installs `libwebkit2gtk-4.1-dev` — driver should be a small additional dependency.
- **Playwright test file location.** Likely `hp41-gui/e2e/smoke.spec.ts` (separate from Vitest unit tests in `hp41-gui/src/*.test.tsx`). Planner finalizes the directory and `playwright.config.ts` location.
- **One-shot GUI coverage measurement methodology.** D-27.4 allows the planner to record GUI coverage numbers in the SUMMARY/CLAUDE.md as a measure-only snapshot. Tooling choice (Vitest's built-in v8 provider via `vitest run --coverage` vs `c8` standalone) is at planner discretion — the numbers are advisory, not gated.

### Cross-cutting invariants (carried forward, NOT re-decided)

- **`#![deny(clippy::unwrap_used)]` at the `hp41-core` crate root.** All new test files carry `#[allow(clippy::unwrap_used)]` at the test mod level (existing pattern from Phase 1 onward).
- **`hp41-core` has no `println!` / `eprintln!`.** Tests that need I/O assertions use the `print_buffer` / `event_buffer` channels established in Phase 11 / Phase 21.
- **SC-4 invariant unchanged.** Phase 27 touches `hp41-core/tests/`, `hp41-cli/tests/`, `hp41-gui/src/` (Vitest already there), `hp41-gui/e2e/` (new), `.github/workflows/ci-gui.yml`, `justfile`, and `CLAUDE.md`. NO source changes to `hp41-core/src/` (test-discovery patterns may need `#[allow]` annotations but no logic edits). NO source changes to `hp41-gui/src-tauri/`.
- **Save-file backward compatibility preserved.** D-27.9 item 4 (save-load roundtrip proptest) explicitly tests this invariant; no `CalcState` field changes are made in this phase.
- **MSRV 1.88 unchanged.** Phase 27 may need to add `proptest` features or `playwright`/`@playwright/test` versions; planner verifies compatibility but the MSRV stays put.
- **CI scope unchanged on `ci.yml`** (CLI/core matrix on Win+macOS+Ubuntu). Playwright lands ONLY in `ci-gui.yml` and ONLY on Ubuntu per ROADMAP cross-cutting constraint.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level
- `.planning/PROJECT.md` — Project goals, build sequence, v2.x scope boundary
- `.planning/REQUIREMENTS.md` §FN-QUAL-01..05 (lines 103–107) — the five locked Phase 27 requirements
- `.planning/ROADMAP.md` Phase 27 section (lines 200–207) — phase goal, 5 success criteria, cross-cutting constraints (coverage gate atomicity, proptest iteration limits, Ubuntu-only Playwright)
- `.planning/STATE.md` — current milestone state (v2.2, Phases 20–26 shipped)
- `CLAUDE.md` — settled architecture decisions (especially v1.0 zero-panics, v1.1 BCD/f64, v2.0 SC-4 invariant, v2.2 Phase 21–25 sections — all relevant context for risk-weighted test targeting)

### Prior phase context (carry-forward)
- `.planning/phases/26-gui-integration-and-polish/26-CONTEXT.md` — D-26.1..D-26.12 (frontend-owned modals, CalcStateView budget, integration-test layout pattern App.test.tsx)
- `.planning/phases/26-gui-integration-and-polish/26-VERIFICATION.md` — recent gap-closure precedent (CR-01..CR-05) showing the kind of integration regression tests Phase 27 should also exercise
- `.planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md` — D-25.6 (CLI ↔ GUI parity), D-25.11..D-25.14 (PendingInput hybrid struct-variants for which Phase 27 tests should exercise round-trips), D-25.16 (JSON pipeline — touched by `function_matrix_parity.rs`)
- `.planning/phases/24-indirect-addressing/24-CONTEXT.md` — D-24.1..D-24.9 (indirect resolution mechanism — the FN-QUAL-04 test surface)
- `.planning/phases/21-flags-display-control-and-sound/21-*-CONTEXT.md` (or PLAN files) — flag-storage shape, FlagTestKind, run_loop skip semantics, display_override / event_buffer fields — all primary targets for Phase 27 properties
- `.planning/phases/20-core-math-and-conversions/20-CONTEXT.md` — math-op semantics for the v2.2 accuracy-suite extensions

### Codebase files (key test targets — risk-weighted priority order from D-27.1)
- `hp41-core/src/ops/program.rs` — `resolve_indirect()`, `run_loop` conditional-skip, `execute_op` synthetic dispatch arms
- `hp41-core/src/ops/stats.rs` — currently lowest coverage at 84.04% lines
- `hp41-core/src/ops/mod.rs` — `dispatch()` synthetic byte conversions, GETKEY edge paths
- `hp41-core/src/ops/registers.rs` — STO arithmetic stack-register variants
- `hp41-core/src/state.rs` — `CalcState` field interactions, serde defaults

### Codebase files (test files to create or extend)
- `hp41-core/tests/numerical_accuracy.rs` — extend with ~70–105 hand-curated cases (D-27.5)
- `hp41-core/tests/proptest_stack.rs` — REFERENCE for style (existing, unchanged in Phase 27)
- `hp41-core/tests/proptest_flags.rs` — NEW (D-27.8, D-27.9, D-27.10) — flag invariants + skip-semantics sentinel + IND-flag property
- `hp41-core/tests/proptest_math.rs` — NEW (D-27.8) — math shape invariants
- `hp41-core/tests/indirect_addressing.rs` — NEW (D-27.12, FN-QUAL-04) — happy-path + non-integer-rejection per `_IND` op
- `hp41-gui/e2e/smoke.spec.ts` — NEW (D-27.13, FN-QUAL-05) — Playwright spec for `2 ENTER 3 + → 5.0000`
- `hp41-gui/playwright.config.ts` — NEW — Playwright config with `retries: 1` (D-27.16)
- `hp41-gui/package.json` — add `@playwright/test`, `tauri-driver` (or equivalent) to `devDependencies`
- `justfile` — `coverage:` recipe raises `--fail-under-lines` from 80 to 95 (D-27.2); `gui-ci:` recipe appends `npm test` (D-27.14); possibly new `gui-e2e:` recipe for local Playwright runs
- `.github/workflows/ci-gui.yml` — add Playwright job (Ubuntu only, required, 1 retry) per D-27.15 and D-27.16
- `CLAUDE.md` — update "Quality Gates" table to ≥ 95% coverage; add Phase 27 settled-architecture note about Playwright scope (Ubuntu only) and Vitest CI gating

### HP-41CV reference material (external — planner sources only for accuracy-suite cases)
- HP-41C Owner's Manual Chapter 3 (Math) — primary source for hand-curated cases per D-27.7
- Free42 source code (https://thomasokken.com/free42/ — open source) — cross-check reference per D-27.7
- HP-41C/CV Quick Reference Guide — secondary source

### Tauri / Playwright official references
- https://tauri.app/v2/develop/tests/webdriver/ — official Tauri E2E webdriver guide (the basis for D-27.15)
- https://playwright.dev/docs/test-retries — retries config for D-27.16

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`hp41-core/tests/proptest_stack.rs`** (Phase 1, 81 lines, 3 properties) — established style precedent: `arb_simple_op()` strategy, `proptest!` block with single `prop_assume!` + assertion. New `proptest_flags.rs` and `proptest_math.rs` mirror this shape.
- **`hp41-core/tests/numerical_accuracy.rs`** (80.4K, 500 cases) — existing format pattern: each case is a `(input, expected, tolerance)` triple, dispatched through `CalcState`, assertion compares with HP-41 10-digit rounding. New v2.2 cases follow the same per-op section structure.
- **`hp41-core/tests/phase21_flags.rs`** (8.9K) — existing example-test coverage of `SF`/`CF`/`FS?`/`FC?`/`FS?C`/`FC?C` + run_loop skip semantics. `proptest_flags.rs` complements (does not duplicate) — properties augment examples.
- **`hp41-core/tests/phase24_ind_variants.rs`** (20.5K) — existing example tests for every `_IND` op. New `indirect_addressing.rs` adds happy-path + non-integer-rejection sentinels missing from this file (per FN-QUAL-04 reading).
- **`hp41-cli/tests/function_matrix_parity.rs`** (Phase 25) — pattern for "test that asserts an external artifact matches the codebase". Useful precedent if Phase 27 wants a similar test for help_data / KEY_REF parity (NOT in scope but a future-phase hook).
- **`hp41-gui/src/App.test.tsx`** (Phase 26 gap-closure, 14.7K, 13 tests) — first repo example of `vi.mock('@tauri-apps/api/core')` pattern. The Playwright E2E job exercises the real `invoke` path; Vitest stays mocked.
- **`justfile` `coverage:` recipe** — already calls `cargo llvm-cov clean --workspace` before measuring (commit ff39017 fix for worktree-stale-profraw). The 80 → 95 ratchet is the single existing line to change.

### Established Patterns
- **`#[deny(clippy::unwrap_used)]` at crate root + `#[allow]` at test mod level** — the established pattern for production-vs-test discipline. Phase 27 test files follow this verbatim.
- **`Op` variants land in 4 places** (D-22.21, D-23.12) — REFERENCED for risk-weighted coverage targeting. Tests can assert dispatch / execute_op / prgm_display parity at a meta level.
- **`#[serde(default)]` for new `CalcState` fields** — REFERENCED by D-27.9 item 4 (save-load roundtrip proptest as an empirical assertion of this invariant).
- **CI scope split: `ci.yml` (3-OS hp41-core/cli matrix) + `ci-gui.yml` (3-OS hp41-gui matrix)** — Phase 27 adds Playwright as a new job (or step) ONLY in `ci-gui.yml` and ONLY on Ubuntu.
- **Vitest as the GUI test framework, Vitest mocks `@tauri-apps/api/core`** — Phase 27 does NOT change Vitest's mocking pattern. Playwright is a separate paradigm: it exercises the un-mocked stack.

### Integration Points
- **`hp41-core/src/ops/program.rs::resolve_indirect`** — primary FN-QUAL-04 test target. The two-tier helper (Phase 24) returns `HpError::InvalidOp` on non-integer; new `indirect_addressing.rs` exercises both branches per `_IND` op.
- **`hp41-core/src/state.rs::CalcState::flags`** — primary FN-QUAL-03 test target. `proptest_flags.rs` properties operate on the underlying `flags` storage; `independence` and `idempotency` properties read/write specific bits across the 56-flag range.
- **`hp41-gui/src-tauri/src/commands.rs::dispatch_op`** — primary FN-QUAL-05 test target. Playwright spec exercises the JS-side `invoke('dispatch_op', { key_id: 'plus' })` path through the real Tauri runtime.
- **`justfile` `coverage:` recipe + `gui-ci:` recipe** — primary FN-QUAL-01 + FN-QUAL-05-side gate points.

</code_context>

<specifics>
## Specific Ideas

- **"Tests that catch real bugs, not coverage padding."** The user explicitly stated this preference for D-27.1. Every test added in Phase 27 must justify its existence in terms of bug class caught, not lines lifted. The planner records the bug-catching rationale alongside each test.

- **Manual Free42 cross-check, cited in test doc comments.** D-27.7 picks the auditable middle ground. When the planner reads Free42 source to validate a quirky case, the test gets a comment like `// Cross-checked against Free42 source ops_math.cc::do_mod — Free42 returns 1 for MOD(7, -3), matching HP-41C Owner's Manual p.234`. This makes the test's HP-41 fidelity claim auditable without taking on Free42 as a build dependency.

- **Honest about the realistic coverage ceiling.** D-27.3 explicitly allows adjusting the ROADMAP gate DOWN if 95% requires trivial padding. The user values intellectual honesty over hitting an arbitrary number. The planner records the actual measured ceiling in `CLAUDE.md` as a v3.x signal.

- **Vitest CI gating is a quiet hole worth closing.** D-27.14 surfaced during discussion (not in ROADMAP). The 5 existing Vitest files pass locally but aren't run on CI — a 1-line justfile change closes the hole.

- **Playwright is a canary, not a comprehensive E2E suite.** D-27.13 picks the literal ROADMAP smoke; broader flows (modal interactions, persistence roundtrip) are explicitly deferred. The job's value is "does the Tauri ↔ React ↔ core chain still work?", not "does feature X behave correctly end-to-end?".

</specifics>

<deferred>
## Deferred Ideas

- **Broader Playwright E2E coverage (modal flows, autosave persistence roundtrip, multi-spec battery)** — Phase 27 D-27.13 deferred. Candidates for v2.3+ if the literal-smoke canary proves stable on the Ubuntu runner.
- **Free42 fixture-generation harness** — Phase 27 D-27.7 deferred. If the v2.2 hand-curated approach reveals lots of edge-case divergence and we keep finding HP-41 quirks, a small Python wrapper around Free42 to generate JSON fixture files might be worth building in v3.x.
- **GUI coverage gates (Vitest --coverage + cargo llvm-cov on hp41-gui/src-tauri)** — Phase 27 D-27.4 deferred. Measure-only this phase; v2.3+ may add CI gates once we have baseline numbers.
- **Per-op pass-rate buckets for numerical_accuracy.rs** — Phase 27 D-27.6 deferred. Global ≥ 98% gate this phase; per-op buckets if a single op drifts in v3.x.
- **TypeScript codegen from Rust enums (`FlagTestKind`, `RegisterOpKind`)** — carried forward from Phase 26 D-26.4 Claude's Discretion. The Phase 27 Vitest gate now catches drift faster, reducing the urgency. v3.x candidate if hand-typed unions become a maintenance burden.
- **Visual-regression snapshot tests for `<Display14Seg />`** — Phase 26 D-26.6 had this as a Phase 27 candidate; not picked up in this discussion. Stays deferred (low value vs the existing Phase 26 unit tests + the new Playwright smoke).
- **README "feature-complete HP-41CV" HARD claim** — carried forward from Phase 25 D-25.17. After Phase 27 verification passes and coverage hits ≥ 95%, the README soft-claim can be upgraded to a hard claim. NOT part of Phase 27 itself; happens in the post-verification commit or the v2.2 milestone-completion commit.
- **CI gate to assert Playwright runs only on Ubuntu** — Phase 27 D-27.15 documents the scope in CLAUDE.md; a sentinel job that asserts the Playwright job doesn't appear in the macOS/Windows matrix would be defense-in-depth but isn't in scope.

</deferred>

---

*Phase: 27-test-hardening*
*Context gathered: 2026-05-15*
