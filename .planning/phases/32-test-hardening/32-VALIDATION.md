---
phase: 32
slug: test-hardening
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-18
---

# Phase 32 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source of truth for thresholds, gates, and OS coverage:
> `32-RESEARCH.md` § "Validation Architecture" (lines 465–517).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + cargo-llvm-cov 0.6+ + WebdriverIO 9.x + bash |
| **Config file** | `Cargo.toml` (workspace + per-crate); `wdio.conf.cjs` (E2E); `justfile` (gate orchestration) |
| **Quick run command** | `just test-core` (filter: `--test math1_op_test_count`, `--test xrom_shadowing`, etc.) |
| **Full suite command** | `just ci` (= `lint + test + coverage + license-audit`) |
| **Estimated runtime** | ~3–5 min on M1 / Ubuntu CI for `just ci`; sub-second per filtered test |

---

## Sampling Rate

- **After every task commit:** Run `just test-core --test <specific-file>` (sub-second per file).
- **After every plan wave:** Run `just ci` (full lint + test + coverage + license-audit gate).
- **Before `/gsd:verify-work`:** `just ci` + `just gui-ci` + `just gui-e2e` (Ubuntu) all green.
- **Max feedback latency:** < 5 seconds for per-task; ~5 min for full-wave gate.

---

## Per-Task Verification Map

> Filled in by `gsd-planner` during plan generation (step 8). Each task in each PLAN.md must reference
> a QUAL requirement and supply an automated verification command. The placeholder rows below show
> the requirement→command mapping; replace with actual `{N}-XX-YY` task IDs once plans land.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 32-01-XX | 01 | 1 | QUAL-01 | — | `hp41-core` line coverage ≥ 95.0 %, regions ≥ 93.0 %; per-`ops/math1/*.rs` file ≥ 90 % | coverage gate | `just coverage` | ✅ | ⬜ pending |
| 32-01-XX | 01 | 1 | QUAL-04 | — | Per-Op test count ≥ 5 across 45 Math Pac I variants | meta-test | `cargo test -p hp41-core --test math1_op_test_count` | ✅ (graduation only) | ⬜ pending |
| 32-01-XX | 01 | 1 | QUAL-07 | — | No Math Pac I name shadows v2.2 builtin mnemonic | meta-test | `cargo test -p hp41-core --test xrom_shadowing` | ✅ (graduation only) | ⬜ pending |
| 32-01-XX | 01 | 1 | QUAL-08 | — | 5 categories of user-callback regression tests (nested-rejection, STO-clobber, STOP-during-INTG, GTO-out, recursion-cap) | regression tests | `cargo test -p hp41-core --test math1_user_callback` | ✅ (9/5 categories shipped — optional add: GTO-out + recursion-cap) | ⬜ pending |
| 32-02-XX | 02 | 1 | QUAL-02 | — | numerical accuracy ≥ 98 % (combined ~700 cases) + baseline_passes ≥ 498/503 | suite + floor | `cargo test -p hp41-core --test numerical_accuracy` | ✅ (extension only) | ⬜ pending |
| 32-02-XX | 02 | 1 | QUAL-06 | — | `approx::assert_relative_eq!` `max_relative = 1e-7` discipline; zero `assert_eq!(decimal, decimal)` on iterated results | lint test | `cargo test -p hp41-core --test lint_math1_assertions` | ❌ W0 (Plan 32-01 or 32-02) | ⬜ pending |
| 32-03-XX | 03 | 1 | QUAL-03 | — | E2E `XEQ "SINH" 1 ENTER` → LCD `1.1752` OR MATRIX DET → LCD `-2.0000` (Ubuntu only) | E2E smoke | `just gui-e2e` | ✅ (extension only) | ⬜ pending |
| 32-03-XX | 03 | 1 | QUAL-05 | — | Free42 12-symbol grep clean in `hp41-core/src/ops/math1/` AND per-file disclaim header present | CI script | `just license-audit` | ❌ W0 (Plan 32-03) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/lint_math1_assertions.rs` — NEW test file (assertion-discipline lint, Plan 32-01)
- [ ] `scripts/check-free42-contamination.sh` — NEW bash CI script (Plan 32-03)
- [ ] `justfile::license-audit` recipe — NEW recipe; extends `just ci` (Plan 32-03)
- [ ] `.github/workflows/ci.yml::license-audit` job — NEW parallel job (Plan 32-03)
- [ ] `data-testid="lcd-display"` continues to surface Math Pac I results — verify in `Display14Seg.tsx` (Plan 32-03 — already present from Phase 27, but Math Pac I output paths must be confirmed reachable)

Existing infrastructure (`approx 0.5.1` already in dev-deps, `math1_op_test_count.rs` already present and graduating from vacuous, `xrom_shadowing.rs` already present and graduating, `math1_user_callback.rs` already present with 9 tests) reduces Wave 0 surface significantly versus a from-scratch phase.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none planned) | — | All Phase 32 gates are automated by design — manual verification is anti-thetical to the phase goal | — |

*All phase behaviors have automated verification.*

---

## Cross-Platform OS Coverage

| Gate | Linux | macOS | Windows |
|------|:-----:|:-----:|:-------:|
| `lint` | ✓ | — | — |
| `test` | ✓ | ✓ | ✓ |
| `coverage` (≥ 95 %) | ✓ | — | — |
| `msrv` (1.88) | ✓ | — | — |
| `license-audit` (NEW) | ✓ | — | — |
| `gui-ci` | ✓ | ✓ | ✓ |
| `gui-e2e` (NEW: 2nd test) | ✓ | — | — |

Rationale (per CONTEXT.md D-27.15 AMENDED 2026-05-15 + Pitfall 14):
- WebdriverIO + `tauri-driver` smoke runs Ubuntu only — `webkit2gtk-driver` + `xvfb` apt deps make Linux the cheapest CI surface; macOS/Windows GUI tests stay at Vitest level.
- Coverage measured on Linux only — `cargo-llvm-cov` profile data is platform-dependent; one canonical OS for the ≥ 95 % gate prevents drift.
- `test` (cargo test) runs on all three OSes — catches x86-vs-ARM numerical drift on the very first PR, per Pitfall 14.

---

## Pass / Fail Thresholds (numeric)

- **Coverage:** `hp41-core` lines **≥ 95.0 %** AND regions **≥ 93.0 %**; per-`ops/math1/*.rs` file **≥ 90.0 %**.
- **Numerical accuracy:** combined **≥ 98.0 %** (≥ ~686 of ~700 cases) AND `baseline_passes ≥ 498` of 503 v1.x cases (independent assertion, D-27.6).
- **Per-Op test count:** **≥ 5 mentions per Math Pac I variant** in `tests/math1_*.rs`.
- **Free42 contamination:** **zero matches** of the 12-symbol pattern outside the disclaim allowlist; **100 % of `hp41-core/src/ops/math1/*.rs` files carry the disclaim header** verbatim.
- **E2E smoke:** **2 of 2** `it()` blocks pass with `mochaOpts.retries: 1` budget on Ubuntu (existing `2 ENTER 3 +` + new Math Pac I workflow).
- **Assertion-discipline lint:** **zero** non-allowlisted `assert_eq!(decimal, decimal)` or hand-rolled `(a - b).abs() < ε` patterns in `tests/math1_*.rs` (replaced with `approx::assert_relative_eq!(..., max_relative = 1e-7)`).

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`lint_math1_assertions.rs`, `check-free42-contamination.sh`, `justfile::license-audit`, `ci.yml::license-audit`)
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s per task / < 5 min per wave
- [ ] `nyquist_compliant: true` set in frontmatter after Wave 0 completes

**Approval:** pending
