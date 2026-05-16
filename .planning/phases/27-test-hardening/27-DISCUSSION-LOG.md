# Phase 27: Test Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-15
**Phase:** 27-test-hardening
**Areas discussed:** Coverage strategy, Accuracy suite shape, Flag proptest scope, Playwright E2E scope

**Baseline measurement (taken 2026-05-15 before discussion):** `cargo llvm-cov -p hp41-core` reports 93.59% lines, 91.21% regions, 97.42% functions. Gap to FN-QUAL-01 target ≥ 95.0% lines: ~1.4 pp lines (~100 lines), ~3.8 pp regions.

---

## Coverage strategy

### Q1: Where should the coverage push focus first?

| Option | Description | Selected |
|--------|-------------|----------|
| Risk-weighted hot spots | Target indirect-resolution error branches, run_loop conditional-skip, ops/stats.rs (84%), synthetic dispatch arms. Bug-catching, not coverage padding. | ✓ |
| File-by-file budget | Walk every file in hp41-core/src, raise each to ≥95%. Mechanical but risks low-value tests. | |
| Broad sweep on error paths | Focus exclusively on Err(…) branches across all hp41-core. Boost regions faster than lines. | |

**User's choice:** Risk-weighted hot spots
**Notes:** D-27.1 enumerates the priority order — ops/program.rs first, then ops/stats.rs (lowest at 84%), then ops/mod.rs synthetic arms, ops/registers.rs, error paths last.

### Q2: How should the `just coverage` gate be raised from `--fail-under-lines 80`?

| Option | Description | Selected |
|--------|-------------|----------|
| Single hard flip to 95 | One atomic commit flips gate from 80 → 95 with the final test additions. Matches ROADMAP verbatim. | ✓ |
| Staged ratchet 80→93→95 | Mid-phase 93 safety net, then 95 at end. Catches regressions during push, adds a commit. | |
| Add regions gate too | Set --fail-under-lines 95 AND --fail-under-regions 90. Stricter signal. | |

**User's choice:** Single hard flip to 95
**Notes:** D-27.2 — atomicity wins over staging. Regions gate left as a v3.x candidate.

### Q3: Fallback if 95% lines isn't reachable without padding?

| Option | Description | Selected |
|--------|-------------|----------|
| Adjust ROADMAP target down | Land at e.g. 94.5% with everything meaningful covered, update ROADMAP/REQUIREMENTS as documented deviation. | ✓ |
| Add #[cfg(not(coverage))] excludes | Annotate genuinely-untestable code so excluded from denominator. | |
| Pad to hit 95 exactly | Write trivial tests until number hits. Fast but maintenance weight. | |

**User's choice:** Adjust ROADMAP target down
**Notes:** D-27.3 — intellectual honesty about realistic ceiling. The `cfg(not(coverage))` knob may still be used sparingly after the risk-weighted push reveals what's left.

### Q4: GUI coverage treatment?

| Option | Description | Selected |
|--------|-------------|----------|
| Out of scope, measure only | One-shot Vitest + cargo llvm-cov reports for visibility; no CI gate. | ✓ |
| Vitest --coverage in gui-ci | Add vitest --coverage with 80% threshold on hp41-gui/src. | |
| Both Rust + TS coverage gated | Maximum hardening, real scope expansion beyond ROADMAP. | |

**User's choice:** Out of scope, measure only
**Notes:** D-27.4 — FN-QUAL-01 explicitly targets hp41-core only. GUI coverage gates are v2.3+ candidates.

---

## Accuracy suite shape

### Q1: How to extend numerical_accuracy.rs for v2.2 math ops (PI, P→R, R→P, RND, FRC, MOD, FACT)?

| Option | Description | Selected |
|--------|-------------|----------|
| Hybrid: curated edges + proptest | ~10–15 hand-curated cases per op + proptest shape invariants. Catches known quirks AND unknown regressions. | ✓ |
| Hand-curated only | ~15–20 cases per op (~110 total), all from HP-41 manual / Free42. No randomization. | |
| Proptest-driven | Skip hand-curated additions; rely on proptest invariants. Faster to write, misses hardware-specific quirks. | |

**User's choice:** Hybrid: curated edges + proptest
**Notes:** D-27.5 — list of specific shape invariants captured (FRC+INT roundtrip, MOD sign semantics, FACT recurrence, P→R↔R→P, RND idempotency).

### Q2: Acceptance gate for the extended suite?

| Option | Description | Selected |
|--------|-------------|----------|
| ≥98% on the new total | Old 500/500 + new ~100 → ~600 cases at ≥98%. ~12 failures budget for HP-41 hardware quirks. | ✓ |
| Hard 100% on new, 98% on legacy | New cases must all pass; legacy keeps ≥1% allowance. | |
| Per-op pass-rate buckets | Each new op (PI, P→R, etc.) hits ≥98% individually. More bookkeeping. | |

**User's choice:** ≥98% on the new total
**Notes:** D-27.6 — global gate is honest about realistic HP-41 quirks; each acceptable failure must cite divergence reason in test doc comment.

### Q3: Free42 use for cross-checking?

| Option | Description | Selected |
|--------|-------------|----------|
| Manual cross-check, cite in tests | Planner runs input through Free42, cites result in doc comment. No automated harness. | ✓ |
| Generate fixtures via Free42 harness | Build Python/C harness producing JSON fixtures. Expensive setup. | |
| HP-41 manual only | Stay within Owner's Manual; skip Free42. Avoids licensing/source-attribution ambiguity. | |

**User's choice:** Manual cross-check, cite in tests
**Notes:** D-27.7 — cheap defensible middle ground; auditable; no Free42 build-dependency.

### Q4: Where do the new proptest modules live?

| Option | Description | Selected |
|--------|-------------|----------|
| One file per concern | New proptest_flags.rs + proptest_math.rs alongside existing proptest_stack.rs. | ✓ |
| One omnibus proptest.rs | Single file with all property modules inline. Easier to scan, bigger file. | |
| Inline next to integration tests | Fold flag proptest into phase21_flags.rs, math into phase20_math.rs. Mixes paradigms. | |

**User's choice:** One file per concern
**Notes:** D-27.8 — mirrors existing hp41-core/tests/phase*_*.rs naming, independent file = independent atomic test run.

---

## Flag proptest scope

### Q1: Beyond ROADMAP's 3 invariants, what additional flag invariants? (multiSelect)

| Option | Description | Selected |
|--------|-------------|----------|
| Independence | For (m, n) m≠n: SF(m) leaves FS?(n) unchanged. Cheap, catches bit-field bugs. | ✓ |
| Idempotency | SF(n)+SF(n) ≡ SF(n). Catches off-by-one toggle bugs. | ✓ |
| Save-load roundtrip | Random flag patterns, serialize/deserialize, assert identical. | ✓ |
| IND-resolved flag semantics | SF_IND(r where r=n) leaves flag n equivalent to SF(n). | ✓ |

**User's choice:** ALL FOUR
**Notes:** D-27.9 — comprehensive flag invariant set. ROADMAP-mandated 3 also land. IND-flag property lives in proptest_flags.rs (it's a property), not indirect_addressing.rs (which is example tests).

### Q2: Also cover run_loop conditional-skip semantics?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, separate sentinel test in proptest_flags.rs | Random short programs with flag-test + 2 steps, assert pc lands correctly. | ✓ |
| Not in proptest — use phase21_flags.rs example tests | Skip rule is one line in run_loop; proptest fuzz adds little. | |
| Defer to IND-addressing integration tests | Cover it in indirect_addressing.rs alongside IND skip cases. | |

**User's choice:** Yes, separate sentinel test
**Notes:** D-27.10 — covers FS?/FC?/FS?C/FC?C × flag-state × program-shape cross-product that example tests sample by hand.

### Q3: Proptest iteration count?

| Option | Description | Selected |
|--------|-------------|----------|
| 1024 flag / 256 math | Mirror ROADMAP cross-cutting guidance verbatim. | ✓ |
| 256 across the board | Use proptest default everywhere. Conservative. | |
| Tune per-property in code review | Start with defaults, let CI data drive. Slower iteration. | |

**User's choice:** 1024 flag / 256 math
**Notes:** D-27.11 — configured via proptest::test_runner::Config per proptest! block.

### Q4: IND integration tests layout?

| Option | Description | Selected |
|--------|-------------|----------|
| New indirect_addressing.rs + flag-IND in proptest_flags.rs | Dedicated file per ROADMAP; flag-IND co-located with flag properties. | ✓ |
| Extend phase24_ind_variants.rs | Add happy-path + non-integer-rejection to existing file. Smaller diff. | |
| One indirect_addressing.rs containing both | Fold flag-IND proptest in alongside example tests. Mixes paradigms. | |

**User's choice:** New indirect_addressing.rs + flag-IND in proptest_flags.rs
**Notes:** D-27.12 — ROADMAP names indirect_addressing.rs. Property+example split keeps each file focused on one paradigm.

---

## Playwright E2E scope

### Q1: How broad should the Playwright smoke be?

| Option | Description | Selected |
|--------|-------------|----------|
| Literal ROADMAP smoke only | Boot, click 2 ENTER 3 +, assert display = 5.0000. One spec. | ✓ |
| Smoke + one modal flow | ROADMAP smoke + STO modal interaction (SHIFT+STO, 0, 5, RCL). | |
| Smoke + modal + persistence | Smoke + modal + autosave→reload roundtrip. Highest coverage, higher flake. | |

**User's choice:** Literal ROADMAP smoke only
**Notes:** D-27.13 — Playwright is a canary not a comprehensive E2E suite. Broader flows are v2.3+.

### Q2: Should Vitest be added to gui-ci?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, add to gui-ci | Add npx vitest run to the gui-ci recipe. One-line change. | ✓ |
| Yes, separate gui-ci-vitest job | New job on Ubuntu only. Keeps matrix job lean. | |
| Out of scope, document as TODO | Defer to v2.3+ to keep Phase 27 focused on FN-QUAL requirements. | |

**User's choice:** Yes, add to gui-ci
**Notes:** D-27.14 — closes a quiet hole. The 5 existing Vitest files pass locally; gating them is high-value for negligible CI runtime.

### Q3: Playwright launch mode?

| Option | Description | Selected |
|--------|-------------|----------|
| Production build + webdriver | just gui-build → tauri-driver → Playwright on WebKitGTK Ubuntu. Tests real IPC. | ✓ |
| Dev server via vite | npm run dev → Playwright on Chromium headless. Skips Tauri runtime + IPC. | |
| Tauri WebDriver via Microsoft Edge | Edge WebDriver instead of WebKitGTK. Equivalent target per Tauri docs. | |

**User's choice:** Production build + webdriver
**Notes:** D-27.15 — official Tauri E2E pattern. Slower job (~3–5 min) but tests what users actually get.

### Q4: Failure-mode policy?

| Option | Description | Selected |
|--------|-------------|----------|
| Required, with one retry | Required for merge; retries: 1 in playwright.config.ts. Transient WebKitGTK hiccups self-heal. | ✓ |
| Required, no retries | Strict gate. Any failure blocks. Forces immediate flake debug. | |
| Continue-on-error initially | Non-blocking smoke for 1–2 weeks of soak before making required. | |

**User's choice:** Required, with one retry
**Notes:** D-27.16 — standard Playwright pattern. Infra hiccups self-heal; deterministic regressions still surface.

---

## Claude's Discretion

- Specific uncovered lines to target first within risk-weighted priorities — planner finalizes after fresh `cargo llvm-cov --html` pass during planning
- Specific hand-curated test cases for each math op (~10–15 per op, sourced from HP-41 manual + Free42 cross-check)
- Proptest strategy combinators (`prop_oneof!` shapes, magnitude ranges, shrink strategies)
- `tauri-driver` vs `WebKitWebDriver` apt package selection — planner confirms during planning
- Playwright test file location (likely `hp41-gui/e2e/smoke.spec.ts` separate from Vitest unit tests)
- One-shot GUI coverage measurement tooling (Vitest v8 provider vs `c8` standalone — advisory only)

## Deferred Ideas

- Broader Playwright E2E (modal flows, persistence roundtrip, multi-spec) — v2.3+ if smoke canary stable
- Free42 fixture-generation harness — v3.x if hand-curated edge cases reveal sustained quirk discovery
- GUI coverage CI gates (Vitest --coverage, cargo llvm-cov on hp41-gui/src-tauri) — v2.3+ after baselines measured
- Per-op pass-rate buckets for numerical_accuracy.rs — v3.x if a single op drifts
- TypeScript codegen from Rust enums (FlagTestKind, RegisterOpKind) — Phase 26 deferral; v3.x candidate
- Visual-regression snapshot tests for `<Display14Seg />` — Phase 26 deferral; stays deferred
- README "feature-complete HP-41CV" HARD claim — happens post-Phase-27 verification or in v2.2 milestone-completion commit
- Sentinel CI gate to assert Playwright stays Ubuntu-only — defense-in-depth, not in scope
