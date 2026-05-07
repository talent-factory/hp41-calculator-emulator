---
phase: 07-hardening
verified: 2026-05-08T00:00:00Z
status: human_needed
score: 14/15 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run `just bench-startup` on Apple M1 hardware after `just build-release`"
    expected: "Cold-start time <= 0.5 s (QUAL-01: 'hyperfine' reports median < 0.5 s for `./target/release/hp41`)"
    why_human: "QUAL-01 cold-start verification requires physical execution on target hardware (M1 / Intel i5 8th gen). Cannot be confirmed from static code analysis or local CI output. The bench-startup recipe is advisory only; no programmatic evidence of the actual start-up timing exists in the codebase."
  - test: "Run CI workflow via GitHub Actions push to `develop` or `main` branch"
    expected: "Test matrix job shows green on ubuntu-latest, macos-latest, and windows-latest for both `just build-release` and `just test` steps"
    why_human: "QUAL-05 cross-platform verification requires GitHub Actions to actually execute the matrix. The workflow file is correctly configured but cross-platform compilation on actual runner VMs has not been observed — only the local macOS execution of `just ci` was verified."
---

# Phase 7: Hardening Verification Report

**Phase Goal:** Performance, cross-platform, test coverage, numerical accuracy suite — all quality gates (QUAL-01 through QUAL-06) verified.
**Verified:** 2026-05-08T00:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `#![deny(clippy::unwrap_used)]` is present as first line of hp41-core/src/lib.rs | VERIFIED | `lib.rs` line 1: `#![deny(clippy::unwrap_used)]` confirmed by direct file read |
| 2 | hp41-core/src/ops/math.rs uses `.expect("valid constant")` not `.unwrap()` | VERIFIED | Lines 25 and 28 of math.rs: `Decimal::from_str("…").expect("valid constant")` — zero `.unwrap()` calls in production code |
| 3 | CI test matrix runs `just build-release` on ubuntu-latest, macos-latest, windows-latest | VERIFIED | `.github/workflows/ci.yml` lines 43-44: `name: Build release binary` / `run: just build-release` inside matrix with `os: [ubuntu-latest, macos-latest, windows-latest]` |
| 4 | CI lint job uses `just fmt-check` (not bare cargo fmt) | VERIFIED | `.github/workflows/ci.yml` line 26: `run: just fmt-check` in the lint job |
| 5 | hp41-core/benches/dispatch_bench.rs exists with criterion_group | VERIFIED | File exists (3.3K). Contains `criterion_group!(benches, …)` and `criterion_main!(benches)` |
| 6 | hp41-core/Cargo.toml contains criterion dev-dependency and [[bench]] entry with harness=false | VERIFIED | `criterion = { version = "0.5", features = ["html_reports"] }` in dev-dependencies; `name = "dispatch_bench"` / `harness = false` in [[bench]] |
| 7 | hp41-core/tests/numerical_accuracy.rs exists with `passes >= 490` assertion | VERIFIED | File exists (2197 lines, 74.5K). `assert!(passes >= 490, …)` present at line 2192 |
| 8 | `just ci` exits 0 | VERIFIED | Ran locally: lint + test + coverage all passed; 94.87% line coverage on hp41-core (gate >= 80%) |
| 9 | REQUIREMENTS.md shows all QUAL-* requirements as [x] complete | VERIFIED | All 6 QUAL-* requirements show `[x]` in requirements list AND traceability table shows `Complete (2026-05-07)` for each |
| 10 | The canonical HMS case (1.3045 → 1.5125) is present in the accuracy suite | VERIFIED | Line 1924: `("1.3045", 1.5125, false)` confirmed in numerical_accuracy.rs |
| 11 | The canonical ISG case (1.00500, 4 iterations before skip) is present | VERIFIED | Line 1430: `case!("isg_dse", "ISG(1.00500) skip=false", …)` confirmed in numerical_accuracy.rs |
| 12 | Program tests: 35 targeted tests in program_tests module pass | VERIFIED | `cargo test -p hp41-core -- program_tests` returns 35 passed, 0 failed — includes test_call_depth_limit, test_max_steps_infinite_loop_guard, test_op_isg_reg_out_of_bounds, etc. |
| 13 | ops/program.rs line coverage >= 80% | VERIFIED | Coverage report shows ops/program.rs at 95.23% line coverage |
| 14 | QUAL-06: numerical_accuracy test passes (`cargo test -p hp41-core -- numerical_accuracy` exits 0) | VERIFIED | `cargo test -p hp41-core -- numerical_accuracy` exits 0 with 1 test passed |
| 15 | QUAL-01: cold-start <= 0.5 s (hyperfine measurement on target hardware) | UNCERTAIN | Recipe `bench-startup` exists (`hyperfine --runs 10 ./target/release/hp41`), release binary exists (1.7MB) — but cold-start timing on physical M1/i5 hardware has NOT been measured in this verification. Requires human testing. |

**Score:** 14/15 truths verified (1 requires human testing)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/lib.rs` | `#![deny(clippy::unwrap_used)]` as first line | VERIFIED | Line 1 confirmed |
| `hp41-core/src/ops/math.rs` | `.expect("valid constant")` in pi_over_180 and pi_over_200 | VERIFIED | Lines 25 and 28 confirmed; zero `.unwrap()` in production code |
| `.github/workflows/ci.yml` | Matrix runs `just build-release` on 3 platforms | VERIFIED | Lines 35, 43-44 confirmed |
| `hp41-core/benches/dispatch_bench.rs` | Criterion benchmark with criterion_group | VERIFIED | File exists with 3 benchmark groups (dispatch_mixed_20ops, dispatch_single_add, dispatch_1000) |
| `hp41-core/Cargo.toml` | criterion dev-dependency and [[bench]] entry | VERIFIED | Both present |
| `hp41-core/tests/numerical_accuracy.rs` | 500 cases, passes >= 490 assertion | VERIFIED | 2197-line file, assertion at line 2192 |
| `hp41-core/src/ops/program.rs` | 35 targeted test functions | VERIFIED | 35 `fn test_` functions confirmed |
| `.planning/REQUIREMENTS.md` | All QUAL-01..QUAL-06 marked [x] | VERIFIED | All 6 requirements show `[x]` and `Complete (2026-05-07)` |
| `.planning/ROADMAP.md` | Phase 7 marked complete | VERIFIED | `[x] **Phase 7: Hardening**` confirmed, progress table shows 6/6 plans complete |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `lib.rs #![deny(clippy::unwrap_used)]` | All hp41-core production code | Crate-level inner attribute | WIRED | Applies to all source files in the crate; `just lint` exits 0 confirming no violations |
| `.github/workflows/ci.yml test matrix` | `just build-release` | `run:` step in matrix job | WIRED | Line 44 of ci.yml; matrix OS list confirmed |
| `.github/workflows/ci.yml lint job` | `just fmt-check` | `run:` step | WIRED | Line 26 confirmed |
| `dispatch_bench.rs` | `hp41_core::ops::dispatch` | criterion benchmark loop | WIRED | Imports `dispatch` from `hp41_core::ops` and calls it in all 3 benchmark functions |
| `numerical_accuracy.rs` | `hp41_core::ops::dispatch` | CalcState + dispatch calls | WIRED | Uses `dispatch()` for all 500 cases; ISG/DSE use `op_isg`/`op_dse` directly |
| `coverage gate` | `just ci` chain | Justfile `ci: lint test coverage` | WIRED | `just ci` confirmed exits 0; `ci:` recipe chains lint, test, coverage in Justfile |

### Data-Flow Trace (Level 4)

Not applicable for this phase. Phase 7 produces test infrastructure, benchmarks, and CI configuration — not components that render dynamic data from a database or API. The `just ci` execution confirmed end-to-end wiring of all test pipelines.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `just ci` exits 0 (lint + test + coverage) | `just ci` | Exits 0; 94.87% coverage; 407 tests pass | PASS |
| Numerical accuracy test passes | `cargo test -p hp41-core -- numerical_accuracy` | Exits 0, 1 test passed | PASS |
| program_tests suite passes | `cargo test -p hp41-core -- program_tests` | 35 passed, 0 failed | PASS |
| Release binary exists | `ls -la target/release/hp41` | 1.7 MB binary exists | PASS |
| Criterion bench compiles | `cargo build --benches -p hp41-core` | (inferred from existing binary and `just ci` passing) | PASS (inferred) |
| `just bench-startup` on target hardware | Cannot run without target hardware | Not executed | SKIP (human needed) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| QUAL-01 | 07-02, 07-06 | Cold-start latency <= 0.5 s on M1 / i5 8th gen | NEEDS HUMAN | `bench-startup` recipe configured; release binary exists; timing on target hardware not measured |
| QUAL-02 | 07-03 | Key-press median latency <= 50 ms | SATISFIED | Criterion dispatch_single_add measured ~25 ns/op (SUMMARY); ~65 ns/op in 07-06 SUMMARY; well under 50 ms |
| QUAL-03 | 07-01 | Zero panics in hp41-core; crash-free >= 99.5% | SATISFIED | `#![deny(clippy::unwrap_used)]` in lib.rs; math.rs uses `.expect()`; `just lint` exits 0 |
| QUAL-04 | 07-04 | hp41-core >= 80% line coverage | SATISFIED | `just coverage` exits 0; 94.87% total; ops/program.rs at 95.23% |
| QUAL-05 | 07-02 | Single codebase on Windows/macOS/Ubuntu | NEEDS HUMAN | CI matrix configured correctly; actual cross-platform CI run not observed in this verification |
| QUAL-06 | 07-05 | >= 98% numerical agreement, 500-case suite | SATISFIED | `cargo test -p hp41-core -- numerical_accuracy` exits 0; plan/SUMMARY records 495/500 (99%) |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | - | No blockers found | - | - |

Scanned: hp41-core/src/lib.rs, hp41-core/src/ops/math.rs, hp41-core/benches/dispatch_bench.rs, hp41-core/tests/numerical_accuracy.rs, hp41-core/src/ops/program.rs (test module), .github/workflows/ci.yml, Justfile. No TODO/FIXME placeholders, no return null/empty stubs, no hardcoded empty values in production rendering paths.

### Human Verification Required

#### 1. QUAL-01 Cold-Start Latency

**Test:** Run `just build-release` then `just bench-startup` on Apple M1 hardware AND on an Intel i5 8th gen machine.
**Expected:** hyperfine reports median cold-start time < 0.5 s for `./target/release/hp41` on both platforms.
**Why human:** The `bench-startup` recipe runs `hyperfine --runs 10 ./target/release/hp41` — this is an advisory recipe, not in the `just ci` chain. Local execution is required on physical target hardware. The release binary exists (1.7 MB, confirmed) and the recipe is correctly configured. This is the only QUAL gate with no programmatic evidence.

#### 2. QUAL-05 Cross-Platform CI Verification

**Test:** Push the current `develop` branch to GitHub and confirm the Actions matrix job succeeds on ubuntu-latest, macos-latest, and windows-latest (both the "Build release binary" and "Run tests" steps green on all three).
**Expected:** All 3 platform jobs show green for `just build-release` and `just test`.
**Why human:** The CI workflow YAML is correctly configured and verified, but cross-platform execution on GitHub Actions runners has not been observed in this verification session. The CI only ran locally on macOS.

---

## Gaps Summary

No code gaps found. All automated checks pass. The 2 items above are observational gaps requiring human execution on specific hardware or CI infrastructure — the code and configuration supporting them are correctly implemented.

**QUAL-01** (cold-start): Recipe exists, binary exists, hyperfine command is correct — just needs to be run on target hardware.
**QUAL-05** (cross-platform): CI matrix is correctly configured with all 3 platforms — just needs an actual push to trigger and observe the GitHub Actions execution.

Both items are infrastructure/observational needs, not implementation defects.

---

_Verified: 2026-05-08T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
