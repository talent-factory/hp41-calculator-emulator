---
phase: 07-hardening
status: findings
files_reviewed: 12
findings:
  critical: 1
  warning: 1
  info: 2
  total: 4
---

# Phase 07 Hardening — Code Review

Reviewed against CLAUDE.md project guidelines at standard depth. All 12 files read in full.

## Files Reviewed

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | CI pipeline definition |
| `Justfile` | Task runner recipes |
| `hp41-core/Cargo.toml` | Crate manifest |
| `hp41-core/benches/dispatch_bench.rs` | Criterion benchmark |
| `hp41-core/src/lib.rs` | Crate root + deny attribute |
| `hp41-core/src/num.rs` | HpNum type + inline tests |
| `hp41-core/src/ops/alpha.rs` | ALPHA mode operations |
| `hp41-core/src/ops/math.rs` | Trig + math operations |
| `hp41-core/src/ops/mod.rs` | Op enum + dispatch() |
| `hp41-core/src/ops/program.rs` | Programming engine + tests |
| `hp41-core/src/tests.rs` | Integration-level unit tests |
| `hp41-core/tests/numerical_accuracy.rs` | 500-case accuracy suite |

---

## Critical

### CRIT-01: CI Workflow Calls `cargo fmt` Directly — Violates CLAUDE.md Tooling Contract

**Confidence: 95** | **File:** `.github/workflows/ci.yml`, line 26

```yaml
- run: cargo fmt --all -- --check
```

CLAUDE.md is explicit: "`just` — sole task runner; all build/test/lint/run/ci targets are `just` recipes. Never call `cargo` directly in CI or docs." The `Justfile` provides `fmt-check` for this purpose. Every other CI step (`just lint`, `just build-release`, `just test`, `just coverage`) routes through `just`; this one step does not.

**Fix:** `- run: just fmt-check`

---

## Warning

### WARN-01: Tolerance Doc Comment Contradicts Actual Constant — Off by 10×

**Confidence: 90** | **File:** `hp41-core/tests/numerical_accuracy.rs`, line 15 vs. line 27

Doc comment says `<= 1e-10` but the actual constant is `1e-9` (one order of magnitude looser). A result differing in the 10th significant digit passes `1e-9` but would fail `1e-10`. Specification drift misleads future contributors.

**Fix:** Update doc comment to `<= 1e-9` to match the constant.

---

## Info

### INFO-01: Dead Helper `get_y` Suppressed With `#[allow(dead_code)]`

**Confidence: 85** | **File:** `hp41-core/tests/numerical_accuracy.rs`, lines 53–56

`get_y` is declared but never called. Stats ops like `MEAN` and `L.R.` push results to both X and Y registers — ȳ and slope in Y are not verified. Either add Y-register checks or remove the helper.

### INFO-02: Dead HpNum-Path Angle Functions (Pre-existing)

**Confidence: 80** | **File:** `hp41-core/src/ops/math.rs`, lines 24–40

`pi_over_180()`, `pi_over_200()`, `to_radians_hpnum()` are dead code from Phase 2, carried forward. Identified in prior review, deferred. Not blocking; recorded for completeness.

---

## Overall Assessment

Phase 7 hardening deliverables are substantively correct. The `#![deny(clippy::unwrap_used)]` gate is properly placed, the 500-case accuracy suite is arithmetically verified, ISG/DSE skip-signal assertions are correct, and the criterion benchmark is advisory-only as intended. CRIT-01 must be fixed to make the CI pipeline CLAUDE.md-compliant.
