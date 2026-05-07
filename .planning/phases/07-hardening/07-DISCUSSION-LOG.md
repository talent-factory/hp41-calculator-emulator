# Phase 7: Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 7-hardening
**Areas discussed:** CI Matrix, Numerical Accuracy Suite, Performance Measurement, Panic Audit

---

## CI Matrix

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub Actions | Free for public repos, native matrix support, runners for all 3 platforms | ✓ |
| No CI for now | Local-only (just ci), manual step before release | |
| Other | GitLab CI, CircleCI, etc. | |

**User's choice:** GitHub Actions

---

| Option | Description | Selected |
|--------|-------------|----------|
| build + test only | cargo build --release + cargo test on all 3 platforms; coverage stays local | ✓ |
| Full just ci on all 3 | lint + test + coverage on every platform | |
| Build only | Just verifies compilation; no test runs | |

**User's choice:** build + test only

---

| Option | Description | Selected |
|--------|-------------|----------|
| Push to main + PRs | Gates every PR and every push to develop/main | ✓ |
| Manual trigger only | workflow_dispatch only, run before release | |
| Push to any branch | CI fires on every push including feature branches | |

**User's choice:** Push to main + PRs

---

| Option | Description | Selected |
|--------|-------------|----------|
| Linux only, separate job | Dedicated coverage job on ubuntu-latest alongside matrix | ✓ |
| No coverage in CI | Coverage stays local gate only | |
| All 3 platforms | cargo-llvm-cov on every platform | |

**User's choice:** Linux only, separate job

---

## Numerical Accuracy Suite

| Option | Description | Selected |
|--------|-------------|----------|
| Document-derived | HP-41 Owner's Handbook formulas + known constants | ✓ |
| Hardware-measured | Physical HP-41 unit as ground truth | |
| Hybrid | Document-derived bulk + hardware-verified edge cases | |

**User's choice:** Document-derived

---

| Option | Description | Selected |
|--------|-------------|----------|
| Rust test module in hp41-core | tests/numerical_accuracy.rs with Rust structs | ✓ |
| External data file (CSV/JSON) | Cases in a data file, harness reads them | |
| Property-based tests with proptest | Random inputs vs reference formulas | |

**User's choice:** Rust test module in hp41-core

---

| Option | Description | Selected |
|--------|-------------|----------|
| Weighted by ROADMAP success criteria | Trig-heavy distribution matching QUAL-05/06 | ✓ |
| Equal coverage across all ops | ~4-5 cases per operation | |
| You decide the distribution | Claude designs distribution based on numerical risk | |

**User's choice:** Weighted by ROADMAP success criteria (arithmetic 100, trig 150, logs 100, ISG/DSE 50, transcendental 50, HMS 30, stats 20)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Count-based threshold test | Single #[test] asserts passes >= 490; failing cases printed | ✓ |
| All-or-nothing (every case must pass) | Each case is its own #[test] | |
| Advisory only (no gate) | Cases run but don't fail the build | |

**User's choice:** Count-based threshold test

---

## Performance Measurement

| Option | Description | Selected |
|--------|-------------|----------|
| hyperfine + just recipe + documented manual step | Non-CI gate, pre-release checklist | ✓ |
| CI benchmark job (automated) | Advisory hyperfine step in CI, non-blocking | |
| Manual only (no just recipe) | Pure manual measurement before v1.0 tag | |

**User's choice:** hyperfine + just bench-startup recipe + documented manual step

---

| Option | Description | Selected |
|--------|-------------|----------|
| Benchmark core dispatch loop (criterion.rs) | dispatch() 1000x, statistical median | ✓ |
| Instrument TUI with timestamps | --bench-mode flag, requires TUI startup | |
| Manual stopwatch / documented untestable | Accept key latency can't be auto-measured in CI | |

**User's choice:** Benchmark core dispatch loop directly with criterion.rs

---

| Option | Description | Selected |
|--------|-------------|----------|
| Advisory only — benchmarks don't fail CI | Too much CI VM variance for absolute timing gates | ✓ |
| Hard gate in CI | CI fails if dispatch median > 50ms | |
| No CI benchmark at all | Benchmarks local only, release checklist | |

**User's choice:** Advisory only

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, add criterion.rs | Industry-standard, produces statistical median + HTML reports | ✓ |
| Use std::time::Instant in a test | Simpler, zero new dependencies | |
| No benchmarks in code | hyperfine-only for all performance validation | |

**User's choice:** criterion.rs as dev-dependency in hp41-core

---

## Panic Audit

| Option | Description | Selected |
|--------|-------------|----------|
| Strict: fix all unwrap() in production code | Convert pi_over_180/pi_over_200 to OnceLock | ✓ |
| Pragmatic: annotate as safe with comment | #[allow(clippy::unwrap_used)] | |
| No change needed | Tested indirectly, ship as-is | |

**User's choice:** Strict — fix all unwrap() in production code

---

| Option | Description | Selected |
|--------|-------------|----------|
| Targeted tests for uncovered paths | Read uncovered lines, write focused tests for gaps | ✓ |
| Accept 59% — TUI-driven paths untestable | Some paths only exercisable through TUI | |
| You decide coverage approach | Claude reviews and decides what to test | |

**User's choice:** Targeted tests for uncovered paths in ops/program.rs

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, deny(clippy::unwrap_used) in lib.rs | Structural enforcement at crate root | ✓ |
| No — too strict | clippy::unwrap_used is too broad | |
| Advisory warning only | warn() not deny() | |

**User's choice:** #![deny(clippy::unwrap_used)] in hp41-core/src/lib.rs

---

| Option | Description | Selected |
|--------|-------------|----------|
| Verify in Phase 7 opening step | Run gcd/stack_stats tests, confirm or fix | ✓ |
| Treat Phase 5 as complete — trust SUMMARY.md | SUMMARY.md documents the fix | |
| Explicitly out of Phase 7 scope | Fix separately before Phase 7 starts | |

**User's choice:** Verify in Phase 7 opening step

---

## Claude's Discretion

- Exact file path for accuracy suite: `tests/numerical_accuracy.rs` vs `src/tests/accuracy_suite.rs` — match existing test structure
- Tolerance epsilon: `≤ 1e-10` (consistent with 10-digit HpNum rounding)
- Whether `just ci` needs update vs new `bench` recipe alongside it
- GitHub Actions YAML specifics (cache, toolchain version pin, OS strings)

## Deferred Ideas

None — discussion stayed within phase scope.
