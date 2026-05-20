---
phase: 28-xrom-framework-math-pac-i-core-ops
plan: "04"
subsystem: hp41-core
tags:
  - complex-functions
  - math-pac-i
  - xrom
  - transcendental
dependency_graph:
  requires:
    - 28-01 (XROM framework + CalcState complex_mode field)
    - 28-02 (hyperbolics — used for SinZ/CosZ/TanZ decomposition)
    - 28-03 (complex_atan2 helper + complex_mode lifecycle + T-replicate pattern)
  provides:
    - Op::Magz, Op::Cinv, Op::ZpowN, Op::Zpow1N, Op::ExpZ, Op::LnZ (unary complex functions)
    - Op::SinZ, Op::CosZ, Op::TanZ (trig via hyperbolic-identity decomposition)
    - Op::ApowZ, Op::LogZ, Op::ZpowW (power/log variants; binary ops: A↑Z, Z↑W)
    - CMPLX-06..17 satisfied (12 requirements closed)
    - math1_complex_edge_cases.rs fully filled (4/4 cases — Plan 28-01 scaffold complete)
    - MATH_1.ops: 30 entries (6 hyperbolic + 7 complex-arith + 17 complex-fn with aliases)
    - Plan 28-05 POLY can now build on full complex stack + functions
  affects:
    - hp41-core/src/ops/math1/complex.rs (12 new op functions)
    - hp41-core/src/ops/mod.rs (12 Op variants + 12 dispatch arms)
    - hp41-core/src/ops/program.rs (12 execute_op arms)
    - hp41-core/src/ops/math1/xrom.rs (30 entries + 12 match arms)
    - hp41-core/tests/math1_complex_edge_cases.rs (2 #[ignore] cases filled)
    - hp41-core/tests/numerical_accuracy.rs (36 new complex-function cases)
    - hp41-core/tests/math1_complex_functions.rs (new — 60 integration tests)
    - hp41-cli/src/prgm_display.rs (12 new arms)
    - hp41-gui/src-tauri/src/prgm_display.rs (12 new arms)
tech_stack:
  added: []
  patterns:
    - f64-bridge transcendental pattern for complex ops (exp, ln, cos, sin, cosh, sinh, atan2, sqrt)
    - Hyperbolic-identity decomposition: SinZ/CosZ/TanZ reuse f64 hyperbolic ops
    - Pitfall-6 branch-cut policy: Domain guard BEFORE complex_mode=true for LnZ, LogZ, ApowZ, ZpowW(0,Re≤0)
    - DivideByZero guard BEFORE mutation for Cinv (symmetric with Plan 28-03 CDiv pattern)
    - TanZ singularity detection: denom |cos(z)|² < 1e-18 threshold (covers BCD-rounded pi/2)
    - Binary complex ops (ApowZ, ZpowW): T-replicate + LiftEffect::Enable (established in Plan 28-03)
    - Unary complex ops: LiftEffect::Disable (consistent with D-28.2 complex unary rule)
    - ZpowN: repeated complex multiply (no f64 bridge per plan spec); N=X, base=Y+iZ
    - Zpow1N: polar-form r^(1/N)·cis(θ/N); (0,0)→(0,0) zero-first-arm per RESEARCH
key_files:
  created:
    - hp41-core/tests/math1_complex_functions.rs
  modified:
    - hp41-core/src/ops/math1/complex.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/tests/math1_complex_edge_cases.rs
    - hp41-core/tests/numerical_accuracy.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
decisions:
  - "complex_atan2 (pub(super) from Plan 28-03) left unused by Plan 28-04 — LnZ/LogZ/ApowZ/ZpowW use direct f64 atan2 via the f64 bridge for clarity; complex_atan2 will be needed by Plan 28-05 POLY"
  - "TanZ singularity threshold set to 1e-18 (not 1e-28) because rust_decimal rounds pi/2 string to 1.570796327 giving cos ≈ -2.05e-10 and denom ≈ 4.2e-20; 1e-18 covers this while staying far from any legitimate non-singular argument"
  - "ZpowN stack convention: N=X (real integer), complex base=Y+iZ (second complex pair) — matches Free42 and OM convention; N is consumed from X"
  - "Zpow1N stack convention: N=X, base=Y+iZ — symmetric with ZpowN; (0,0)^(1/N)=0 per RESEARCH zero-first-arm"
  - "ApowZ and ZpowW are binary ops using τ as the base/exponent: a=Z+iT, z=X+iY for ApowZ; z=X+iY, w=Z+iT for ZpowW — consistent with HP-41 stack orientation"
  - "ZpowW: (0+0i)^w with Re(w)>0 returns (0,0) (zero to positive power); Re(w)≤0 returns Domain"
  - "LnZ angle output respects angle_mode (DEG/RAD/GRAD) — complex LN's imaginary part is the principal argument converted to current mode"
  - "MATH_1.ops: 30 entries (6+7+17); math1_resolve uses Unicode ↑ (U+2191) as primary with ASCII ^ aliases"
metrics:
  duration: "~65 minutes"
  completed: "2026-05-16"
  tasks_completed: 4
  tasks_total: 4
  files_created: 1
  files_modified: 8
---

# Phase 28 Plan 04: Complex Transcendental Functions Summary

**One-liner:** 12 complex transcendental Op variants (Magz through ZpowW) landed via f64-bridge with Pitfall-6-safe branch-cut guards, hyperbolic-identity trig decomposition, and repeated-multiply ZpowN — closing CMPLX-06..17 and completing the Plan 28-01 scaffold.

## What Was Built

### Task 1: 5 Unary Complex Functions (Magz, Cinv, ExpZ, LnZ, LogZ)

Added to `hp41-core/src/ops/math1/complex.rs`:

**op_magz** — `|ζ| = sqrt(X²+Y²)` via f64 bridge. Writes to X; Y unchanged per OM convention. Domain: none. LiftEffect::Disable.

**op_cinv** — `1/(X+iY) = (X-iY)/(X²+Y²)`. DivideByZero guard BEFORE mutation (symmetric with Plan 28-03's CDiv). LiftEffect::Disable.

**op_exp_z** — `e^(X+iY) = e^X·(cos(Y)+i·sin(Y))` via f64 bridge. No domain. LiftEffect::Disable.

**op_ln_z** — `ln|ζ| + i·arg(ζ)`. **Domain guard on (0+0i) (CMPLX-11 / Pitfall 6) fires BEFORE complex_mode=true.** Angle output respects angle_mode. LiftEffect::Disable.

**op_log_z** — `LNZ/ln(10)`. Inherits LnZ's Domain guard on (0+0i) (CMPLX-12). LiftEffect::Disable.

Also added: `f64_from_radians` module-local helper (mirrors `ops/math.rs` private helper; needed for LnZ/LogZ angle conversion).

Inline tests: 31 new unit tests (≥5 per op); all pass.

### Task 2: 7 Complex Trig + Power Functions (SinZ, CosZ, TanZ, ZpowN, Zpow1N, ApowZ, ZpowW)

Added to `hp41-core/src/ops/math1/complex.rs`:

**op_sin_z** — `sin(X+iY) = sin(X)·cosh(Y) + i·cos(X)·sinh(Y)`. Hyperbolic-identity decomposition. LiftEffect::Disable.

**op_cos_z** — `cos(X+iY) = cos(X)·cosh(Y) - i·sin(X)·sinh(Y)`. Hyperbolic-identity decomposition. LiftEffect::Disable.

**op_tan_z** — `tan(z) = sin(z)/cos(z)`. **Domain guard: denom |cos(z)|² < 1e-18 → Domain (CMPLX-13).** Threshold covers BCD-rounded π/2 case (cos ≈ -2.05e-10, denom ≈ 4.2e-20). LiftEffect::Disable.

**op_z_pow_n** — `ζ^N` via repeated complex multiply (no f64 bridge). N=X (integer), base=Y+iZ. z^0=1; negative N computes inverse via CINV logic. LiftEffect::Disable.

**op_z_pow_1_n** — `r^(1/N)·cis(θ/N)` via polar form. (0+0i)^(1/N)→(0+0i) zero-first-arm. LiftEffect::Disable.

**op_a_pow_z** — `exp(z·ln(a))`. **Domain on a=(0+0i) (CMPLX-16) fires BEFORE complex_mode=true.** Binary: T-replicate; LiftEffect::Enable.

**op_z_pow_w** — `exp(w·ln(z))`. **Domain on (0+0i)^w with Re(w)≤0 (CMPLX-17 / Pitfall 6).** (0+0i)^w with Re(w)>0 → (0+0i). Binary: T-replicate; LiftEffect::Enable.

Inline tests: 39 new unit tests (≥5 per op); all pass. All 105 complex unit tests pass.

### Task 3: Wire 12 Op Variants Through Dispatch Chain + Fill Tests

**Op enum** (`ops/mod.rs`): 12 new variants with doc comments (CMPLX-06..17 cross-references).

**dispatch()** (`ops/mod.rs`): 12 new arms + import of all 12 op functions.

**execute_op()** (`ops/program.rs`): 12 new arms using fully-qualified paths.

**MATH_1.ops** (`ops/math1/xrom.rs`): grew from 13 to 30 entries:
- 17 new entries with Unicode ↑ (U+2191) primary + ASCII ^ aliases
- math1_resolve() extended with 12 new match arms

**math1_complex_edge_cases.rs**: Removed `#[ignore]` from 2 remaining placeholders:
- `ln_z_zero_returns_domain`: dispatch Op::LnZ on (0+0i) → Err(Domain); stack unchanged; complex_mode not set
- `z_pow_w_zero_neg_exp_returns_domain`: dispatch Op::ZpowW on (0+0i)^(-1+0i) → Err(Domain); also (0+0i)^(0+0i) → Domain
- All 4 edge cases now pass (Plan 28-01 scaffold complete)

**numerical_accuracy.rs**: +36 cases (3 per op × 12 ops) with HP 00041-90034 ~pp.24-26 + Free42 v3.0.5 cross-check citations.

**math1_complex_functions.rs** (new): 60 dispatch-path integration tests (5 per variant), satisfying math1_op_test_count Pitfall 16 gate.

### Task 4: prgm_display.rs Exhaustive Match Arms (Both Copies)

Added 12 arms under `// ── Phase 28: Complex Functions (Plan 28-04)` section:
- `Op::Magz => "MAGZ"`, `Op::Cinv => "CINV"`, `Op::ZpowN => "Z↑N"`, `Op::Zpow1N => "Z↑1/N"`
- `Op::ExpZ => "E↑Z"`, `Op::LnZ => "LNZ"`, `Op::SinZ => "SINZ"`, `Op::CosZ => "COSZ"`
- `Op::TanZ => "TANZ"`, `Op::ApowZ => "A↑Z"`, `Op::LogZ => "LOGZ"`, `Op::ZpowW => "Z↑W"`

Both `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs` updated identically.

`cargo build --workspace` confirms exhaustive match in both copies (Rust compiler enforces no missing Op variant).

## Test Results

| Gate | Result |
|------|--------|
| `cargo build -p hp41-core` | PASS (1 dead_code warning — complex_atan2 pub(super), used by Plan 28-05) |
| `cargo build --workspace` | PASS |
| `cargo test -p hp41-core` | 1228 passed, 5 ignored |
| `--lib math1::complex::tests` | 105 passed (35 Plan-28-03 + 31 Task-1 + 39 Task-2) |
| `--test math1_complex_edge_cases` | 4 passed (all 4 cases, NO #[ignore] remaining) |
| `--test math1_complex_functions` | 60 passed (5 per variant × 12 variants) |
| `--test math1_op_test_count` | 1 passed (Pitfall 16 gate: ≥5 test mentions per variant) |
| `--test xrom_shadowing` | 2 passed (30 mnemonics confirmed non-shadowing) |
| `--test numerical_accuracy` | 8 passed (36 new complex-function cases embedded in suite) |

## Branch-Cut Policy (CMPLX-11/17 / Pitfall 6)

| Op | Guard | Error Type | Fires Before Mutation |
|----|-------|------------|----------------------|
| `Cinv` | X=0 AND Y=0 | DivideByZero | Yes |
| `LnZ` | X=0 AND Y=0 | Domain | Yes |
| `LogZ` | X=0 AND Y=0 | Domain | Yes |
| `ApowZ` | Z=0 AND T=0 (a=0) | Domain | Yes |
| `ZpowW` | X=0 AND Y=0 AND Re(w)≤0 | Domain | Yes |
| `TanZ` | denom < 1e-18 | Domain | Yes (fires before complex_mode=true) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] TanZ singularity threshold calibration**
- **Found during:** Task 2
- **Issue:** The plan specified "cos(x)=0 AND sinh(y)=0" as the singularity condition. Initial implementation used EPS=1e-15 threshold, but rust_decimal rounds the π/2 string "1.5707963267948966" to `1.570796327` (10 sig figs), giving `cos = -2.05e-10` and `denom ≈ 4.2e-20`. Thresholds of 1e-28 and 1e-20 both failed to catch this.
- **Fix:** Set threshold to 1e-18 — catches the BCD-rounded pi/2 case (denom ~4.2e-20) while remaining far above any legitimate non-singular cos² value.
- **Files modified:** `hp41-core/src/ops/math1/complex.rs` (op_tan_z)
- **Verification:** Test `tan_z_singularity_is_domain` now passes.

**2. [Rule 1 - Bug] numerical_accuracy.rs missing FromPrimitive import**
- **Found during:** Task 3
- **Issue:** Added `Decimal::from_f64()` calls to numerical_accuracy.rs without adding the required `FromPrimitive` trait import. Compilation error.
- **Fix:** Added `use rust_decimal::prelude::{FromPrimitive, ToPrimitive};` (was `ToPrimitive` only).
- **Files modified:** `hp41-core/tests/numerical_accuracy.rs`

**3. [Rule 1 - Bug] AngleMode path wrong in numerical_accuracy.rs**
- **Found during:** Task 3
- **Issue:** Used `hp41_core::ops::AngleMode::Rad` but `AngleMode` is re-exported at `hp41_core::AngleMode`, not in the `ops` submodule.
- **Fix:** Changed to `hp41_core::AngleMode::Rad` (correct re-export path from lib.rs).
- **Files modified:** `hp41-core/tests/numerical_accuracy.rs`

**4. [Rule 2 - Missing Functionality] math1_complex_functions.rs integration test file**
- **Found during:** Task 3 (math1_op_test_count meta-test failure)
- **Issue:** The Pitfall 16 gate (`math1_op_test_count.rs`) scans `hp41-core/tests/math1_*.rs` files for variant name mentions. The new variants had ≥5 tests in the inline test module (`complex.rs`) but the meta-test only scans `math1_*.rs` files in the `tests/` directory.
- **Fix:** Created `hp41-core/tests/math1_complex_functions.rs` with 60 dispatch-path integration tests (5 per variant), exactly as Plan 28-02 created `math1_hyperbolics.rs` and Plan 28-03 created `math1_complex.rs`.
- **Files created:** `hp41-core/tests/math1_complex_functions.rs`

## Known Stubs

None. All 12 Op variants are fully implemented. `complex_atan2` is `pub(super)` and unused by Plan 28-04 (all functions use direct f64 `atan2` for clarity), but it IS available for Plan 28-05.

## Plan 28-05 Inheritance

Plan 28-05 (POLY — polynomial evaluation with complex coefficients) can now rely on:
- Full complex function suite (12 transcendental ops + 5 arithmetic ops)
- `complex_atan2(pub(super))` helper available if needed for POLY internal computations
- Branch-cut policy locked (Domain on zero inputs for log-class ops)
- MATH_1.ops has 30 entries; 23 unique mnemonics registered in math1_resolve

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `hp41-core/src/ops/math1/complex.rs` functions exist (12 new) | FOUND |
| `hp41-core/tests/math1_complex_functions.rs` exists | FOUND |
| Commit 094cae8 exists (Task 1) | FOUND |
| Commit dcf68cc exists (Task 2) | FOUND |
| Commit 10e9b31 exists (Task 3) | FOUND |
| Commit 8d8d131 exists (Task 4) | FOUND |
| 12 Op variants in hp41-cli prgm_display.rs | FOUND (12 lines) |
| 12 Op variants in hp41-gui prgm_display.rs | FOUND (12 lines) |
| math1_complex_edge_cases.rs: 4/4 cases active (no #[ignore]) | CONFIRMED |
| MATH_1.ops.len() == 30 | CONFIRMED (test passes) |
| xrom_shadowing: 30 mnemonics non-shadowing | CONFIRMED (2 tests pass) |
| cargo test -p hp41-core: 1228 passed, 5 ignored | CONFIRMED |
| cargo build --workspace: clean (1 dead_code warning) | CONFIRMED |
| STATE.md and ROADMAP.md unmodified | CONFIRMED |
