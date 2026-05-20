# Technology Stack — v3.0 Math 1 Pac

**Researched:** 2026-05-16
**Scope:** NEW dependencies (or zero additions, justified) needed to add Math 1 Pac behavioral emulation. The existing v1.0 → v2.2 stack is validated and unchanged.

> **NOTE (2026-05-16):** This document was drafted under the assumption that v3.0 Math 1 covers matrix ops (`M+`, `MAT*`, `INV`, `TRANS`, `DET`), discrete-op complex arithmetic (`CADD`, `CMUL`, `CDIV`, `CABS`, `CARG`, `CCHS`, `CCONJ`), polynomial root-finder `PROOT`, and vector ops (`V+`, `V-`, `VDOT`). After the FEATURES.md research, this inventory turned out to be the **Advanced Matrix Pac + Advantage Pac**, not the actual HP-41 Math Pac I. The true Math Pac I is prompt-driven user-code (MATRIX, SOLVE, POLY, INTG, DIFEQ, FOUR, hyperbolics, triangles, TRANS coord-transform). The crate-level conclusions below (zero runtime deps, hand-coded numerics, optional `approx` dev-dep) remain valid for EITHER scope — Math Pac I shares the same f64-bridge / rust_decimal-everywhere discipline. The PHASE PLAN section below is invalidated and will be reshaped after the scope decision (see REQUIREMENTS.md).

---

## Summary

**Recommendation: ZERO new runtime dependencies in `hp41-core` for v3.0.**

Every Math 1 op (whether the user-intended Advanced-Matrix-/Advantage-Pac function set OR the actually-documented Math Pac I prompt-driven programs) is implementable inside `hp41-core` using:
- The existing `rust_decimal 1.42` `HpNum` discipline (BCD with 10-digit rounding) for every user-visible quantity
- An internal-only f64 bridge (the established pattern from `num.rs::checked_asin/_acos/_atan`) where rust_decimal lacks the primitive (e.g. `atan2` for complex `CARG`, LU-pivot search inside matrix `INV`, iterative root refinement)
- Hand-coded textbook algorithms (≤ 50 LOC each) for the small bounded numerical surface

Every realistic third-party candidate (`num-complex`, `nalgebra`, `faer`, `ndarray`, `roots`, `peroxide`, `gauss-quad`, `argmin`) either imposes a `T: Float` trait bound that excludes `rust_decimal::Decimal`, panics on bad input violating our `#![deny(clippy::unwrap_used)]` invariant, or adds binary-size cost far in excess of what it replaces. The hand-coded path keeps the type-discipline (`HpNum` everywhere except internal f64 bridges) that already underwrites the 566-case numerical_accuracy harness.

One **dev-dependency** addition is recommended for test ergonomics: `approx 0.5.1` for matrix/complex relative-tolerance assertion macros. Nothing else.

---

## Verified Versions (crates.io, 2026-05-16)

| Crate | Latest | MSRV | Status |
|-------|--------|------|--------|
| `rust_decimal` | 1.42.x stable (2.0.0-alpha.0 on master, MSRV 1.85 — do NOT track) | 1.85 | Already used — keep `1.42` workspace pin |
| `num-complex` | 0.4.6 | 1.60 | **Reject** — `Float` bound on `sqrt`/`exp`/`ln` excludes `HpNum` |
| `nalgebra` | 0.34.2 | 1.87 | **Reject** — `Field`-trait + f64-centric, overkill for ≤ 14×14 |
| `faer` | 0.24.0 | 1.84 | **Reject** — production LAPACK, ≤ 14×14 matrices don't justify it |
| `ndarray` | 0.17.2 | unknown (≥ 1.64) | **Reject** — N-D generic, we need 2-D |
| `roots` | 0.0.8 (last release 2022-12) | unspecified | **Reject** — abandoned, generic over `FloatType` (Float-only) |
| `peroxide` | 0.41.2 | edition 2018 | **Reject** — comprehensive science library (ML/plotting/dataframes), footprint mismatched |
| `gauss-quad` | 0.3.1 | unknown | **Reject** — fixed Gauss nodes, no adaptive refinement; INTG needs adaptive |
| `argmin` | 0.11.0 | recent | **Reject** — heavy multi-method optimization framework, not single-variable secant |
| `approx` | 0.5.1 | 1.36+ | **Accept as DEV-dep only** for matrix/complex relative-tolerance assertions |

---

## The Four Decisions Driving v3.0 Stack

### Decision 1: HpNum is the unit of currency for ALL Math 1 ops

Every Math 1 op signature is `fn op_xxx(state: &mut CalcState) -> Result<(), HpError>`, consuming and producing `HpNum` exactly like the existing ~130 ops. **No exposed f64.**

**Internal f64 fallback is permitted in three documented cases:**

1. **Complex `sqrt` / `arg`** (for `CABS`/`MAGZ`, `CARG`, polar form). `rust_decimal::MathematicalOps::sqrt` already exists; `atan2` does not. Use the existing f64-bridge pattern from `checked_asin` (num.rs:184–192): `to_f64() → f64::atan2() → from_f64() → HpNum::rounded()`. The 15.9 → 10 digit drop is the same QUAL-06-acceptable trade we made in v1.0.

2. **Matrix LU pivot selection** (for MATRIX `DET`, `SOLVE`-via-matrix). Pivoting needs `|x| > eps` comparisons; rust_decimal handles these natively, but the pivot-row swap and Gauss elimination steps benefit from a single `Vec<f64>` working copy when matrix ≥ 5×5 (perf, not correctness). Result rounded back to `HpNum` per-cell before storing.

3. **Polynomial root iteration** (`POLY` / `PROOT`). Polynomial-root iteration is intrinsically iterative — running it in f64 then `HpNum::rounded()`-ing the final root matches HP-41 hardware behavior where the original 10-digit display masks intermediate-step error.

**Rationale:** The HP-41 Math Pac itself ran on a Nut CPU with 56-bit BCD floats — every Math Pac ROM routine internally used the same BCD that user-visible registers stored. Our `HpNum`/`rust_decimal` model mirrors that. Switching to f64 across the board would (a) violate the unbroken type-discipline from v1.0 (`HpNum` everywhere except inverse-trig + one statistics edge case), (b) break round-trip determinism of saved register files, and (c) break the 566-case numerical_accuracy harness's tolerance model.

### Decision 2: ComplexHp — roll our own, do NOT add `num-complex`

Define `ComplexHp { re: HpNum, im: HpNum }` in a new module `hp41-core/src/ops/complex.rs`.

**Why not `num-complex 0.4.6`:**

- `Complex<T>` arithmetic requires `T: Clone + Num`. `rust_decimal::Decimal` does NOT impl `num_traits::Num`. Even if we adapter-impl `Num` for a `HpNum` newtype, transcendentals (`Complex::sqrt`, `Complex::exp`, `Complex::ln`, `Complex::powc`) ALL require `T: Float`. `Decimal` cannot satisfy `Float` (no `NaN`, no `INFINITY`, no `EPSILON`, no IEEE-754 layout). Verified via docs.rs/num-complex/0.4.6 trait-bound inspection.
- We would end up forced to use `Complex<f64>`. The HP-41 Math Pac stores complex numbers in stack-register pairs (X/Y or the dedicated ζ/τ overlay per FEATURES.md). Wrapping them in `Complex<f64>` at every entry/exit boundary trashes 5+ digits of precision unnecessarily and inverts our type-discipline.
- A hand-coded `ComplexHp` lives inside `#![deny(clippy::unwrap_used)]` cleanly — every op returns `Result<ComplexHp, HpError>`, propagating overflow/domain.

**Integration point:** `hp41-core/src/ops/complex.rs` (new). Pattern: mirror `ops/hms.rs` (small self-contained module, no cross-module dispatch needed).

### Decision 3: Matrices — hand-coded `MatrixView` over user register blocks

Define `MatrixView<'a>` in a new module `hp41-core/src/ops/matrix.rs`. The HP-41 Math Pac I stores matrices as contiguous register blocks starting at R15, with order N in R14 (per FEATURES.md). MatrixView borrows the slice, exposes `get(r,c) -> HpNum` / `set(r,c, HpNum)` / `dim() -> (usize, usize)`.

**Why not `nalgebra 0.34` / `faer 0.24` / `ndarray 0.17`:**

- **Float-only generic bounds:** All three rest on `T: Float` / `T: Scalar + Field`. `rust_decimal::Decimal` does not satisfy. We'd be forced to use f64 matrices, lose precision, and write conversion shims at every register I/O boundary.
- **Binary size:** nalgebra brings 10+ transitive deps (matrixmultiply, simba, num-traits, num-complex, approx, rand, typenum) — ~150 KB compiled, ~1.5–2 MB in the Tauri bundle. For ≤ 14×14 matrices, the BLAS path is never even hit.
- **Panic-on-error:** All three panic on dimension mismatch / singular matrix. Our `Result<(), HpError>` discipline requires we surface every error as `HpError::Domain`. Defensive try-blocks at every call site defeat the convenience.
- **MSRV:** nalgebra 0.34.2 declares `rust-version = "1.87.0"` AND `edition = "2024"` — the latter requires 1.85+ but the toolchain semantics are fresh and untested in our workspace.

**Algorithms to hand-code (all textbook, ≤ 50 LOC each):**

- **Multiplication:** triple-loop `O(n³)`. For n ≤ 14, ≤ 2744 ops total — negligible vs. dispatch overhead (~65 ns/op baseline).
- **Determinant:** LU decomposition with partial pivoting; det = product of diagonal × pivot-sign. ~30 LOC.
- **Inverse:** Gauss-Jordan on augmented matrix `[A | I]`. ~40 LOC. Returns `HpError::Domain` on singular (pivot below 1e-10 threshold). Hardware-faithful: Math Pac I surfaces `NO SOLUTION`.
- **Transpose, addition, subtraction:** trivial.

**Integration point:** `hp41-core/src/ops/matrix.rs` (new module). Tests use `approx::assert_relative_eq!` (dev-dep) for matrix-product / inverse-product-identity assertions with tolerance ~1e-9.

### Decision 4: Integration and root-solver — hand-coded Romberg/Simpson and secant

Hand-code `INTG` (Math Pac I uses Simpson per FEATURES.md OM citation) and `SOLVE` (modified secant per OM) directly in `hp41-core/src/ops/numerical.rs` (new). Both take a user-program label callback through `run_loop()` (existing infrastructure from v1.0 — no new IPC needed). The user-callback re-entrancy decision is detailed in ARCHITECTURE.md.

**Why hand-coded, not `gauss-quad` / `quadrature` / `argmin`:**

- **`gauss-quad 0.3.1`:** fixed-order Gauss-Legendre nodes. Math Pac I INTG is adaptive (refines per display setting). Wrong algorithmic shape.
- **`quadrature 0.1.2`:** Gauss-Kronrod and double-exponential, all f64. Last release 2017 — abandoned.
- **`argmin 0.11`:** multi-method optimization framework (BFGS, trust regions, conjugate gradient). SOLVE is one-dim secant with bracket — argmin is 100× the surface and panics on trait-mismatch.
- HP-41 Math Pac I's actual SOLVE/INTG algorithms are well-documented in the 1981 Owner's Manual. We have a **behavioral spec** — re-implementing the exact algorithms makes our outputs match hardware bit-for-bit (within 10-digit rounding). Using a different crate's algorithm guarantees subtle divergence.

**Integration point:** `hp41-core/src/ops/numerical.rs` (new). Connects to existing `ops/program.rs::run_loop()` for the user-program callback path; lift effect per-op (mostly `Neutral`).

---

## Module-Loading / XROM Framework: ZERO new deps

Static-link the Math Pac I op set into `hp41-core` directly. The "XROM framework" is a **dispatch pattern**, not a runtime loader.

```rust
// hp41-core/src/ops/xrom.rs (new)
pub struct XromModule {
    pub id: u8,
    pub name: &'static str,
    pub ops: &'static [(&'static str, Op)],
}
pub const MATH_1: XromModule = XromModule {
    id: 7,   // real Math Pac is XROM 7 — honor it
    name: "MATH 1A",
    ops: &[
        ("MATRIX", Op::MatrixWorkflow),
        ("SOLVE", Op::SolveWorkflow),
        ("POLY", Op::PolyWorkflow),
        ("INTG", Op::IntgWorkflow),
        ("DIFEQ", Op::DifeqWorkflow),
        ("FOUR", Op::FourWorkflow),
        ("MAGZ", Op::Magz),
        // ... 55 named entry points
    ],
};
pub fn resolve_xrom(name: &str) -> Option<Op> {
    MATH_1.ops.iter().find(|(n, _)| *n == name).map(|(_, op)| *op)
}
```

`hp41-cli` and `hp41-gui` already have the XEQ-by-name infrastructure (`builtin_card_op` resolver pattern from v2.1, extended in v2.2 Plan 25-03 from 4 → 12 names). Math Pac I ops slot into the same resolver chain.

**Why not dynamic `.mod` file parsing:** PROJECT.md:160 locks "HP-copyrighted ROM bytes NEVER redistributed." Dynamic .mod loading invites users to drop HP's copyrighted ROM images into a runtime directory — exactly the legal hazard we excluded permanently in v2.2. Behavioral emulation = compiled Rust, not parsed bytes.

**Future-proofing for v3.1+:** When Stat 1 lands, it becomes another `pub const STAT_1: XromModule = …` constant alongside `MATH_1`. The resolver just searches a longer list. No new deps ever.

---

## Test Infrastructure: One dev-dep addition

### `approx 0.5.1` as dev-dependency (NOT runtime)

```toml
[dev-dependencies]
approx = "0.5"
```

**Purpose:** Matrix and complex-number assertions in test files need relative-tolerance comparators. Today the 566-case `numerical_accuracy.rs` harness implements `passes_with_tol` manually (line 58–67 of the existing file) — that works for scalars. For matrix-inverse-product round-trips and complex-arithmetic identity tests (`e^(iπ) + 1 ≈ 0`), `approx::assert_relative_eq!(actual, expected, max_relative = 1e-9)` is more readable than 10+ hand-rolled per-cell comparisons.

**MSRV impact:** approx 0.5.1 MSRV is 1.36 — well under our 1.88.
**Zero-panic compatibility:** approx is dev-only, applies only inside `#[cfg(test)]` modules that already carry `#![allow(clippy::unwrap_used)]`.

**Integration points:**
- `hp41-core/tests/numerical_accuracy.rs` — extend with new sections
- `hp41-core/tests/matrix_ops.rs` (new) — dedicated matrix-ops suite
- `hp41-core/tests/complex_ops.rs` (new) — dedicated complex-arithmetic suite

---

## Migration vs Additions

### Runtime additions to `hp41-core` (PRODUCTION dependencies): ZERO

Cargo.toml `[dependencies]` block is **unchanged** from v2.2.

### Dev-dependency additions: ONE

```toml
[dev-dependencies]
approx = "0.5"   # NEW — matrix / complex relative-tolerance assertions
```

### CLI / GUI additions: ZERO

The `hp41-cli` and `hp41-gui` workspaces are pure adapter layers. New Math Pac I ops surface through the existing `Op` enum dispatch in `hp41-cli/src/keys.rs::key_to_op` + `xeq_by_name_local_resolve` and `hp41-gui/src-tauri/src/key_map.rs::resolve`. No new crates.

### New modules in `hp41-core/src/` (approximate LOC, depends on final FEATURES scope)

| File | Purpose |
|------|---------|
| `ops/xrom.rs` | XromModule struct + MATH_1 const + resolver |
| `ops/complex.rs` | ComplexHp struct + complex ops |
| `ops/matrix.rs` | MatrixView + matrix ops |
| `ops/numerical.rs` | INTG (Simpson/Romberg) + SOLVE (secant) + POLY |
| `ops/hyperbolic.rs` | SINH/COSH/TANH/ASINH/ACOSH/ATANH |

---

## What NOT to Add

| Crate | Reason to Exclude |
|-------|-------------------|
| `nalgebra` | Float-only `Scalar` bound rejects `HpNum`; +10 transitive deps; +1.5–2 MB binary; ≤ 14×14 matrices never trip the optimised paths. |
| `faer` | Same Float bound; production LAPACK is overkill for ≤ 14×14; binary-size cost ditto. |
| `ndarray` | N-dim generic for 2-D-only HP-41 matrices; same Float constraint. |
| `num-complex` | `Complex::sqrt`/`exp`/`ln`/`powc` require `T: Float`; `rust_decimal::Decimal` cannot satisfy `Float` (no NaN/INFINITY/EPSILON layout). |
| `roots` | Abandoned (last release 2022-12); Float-only generic. |
| `peroxide` | Comprehensive science library — linear algebra + ML + statistics + plotting + dataframes. Footprint catastrophically larger than our need. Edition 2018, suggests stagnation. |
| `gauss-quad` | Fixed-order Gauss quadrature; HP-41 INTG is adaptive — wrong algorithmic shape. |
| `quadrature` | Last release 2017, abandoned; f64-only; not adaptive. |
| `argmin` | Multi-method optimization framework; SOLVE is one-dim secant — 100× our surface. |
| `scilib` | Broad scientific library; mixes physics, special functions, statistics; same overkill story as peroxide. |
| `libm` | We already get `libm` transitively through `rust_decimal::MathematicalOps`. Direct addition is duplicate work. |
| Custom BCD for matrices/complex | Already evaluated and rejected at v1.0 (key decision row in PROJECT.md). `rust_decimal` was the right answer then. |
| `tauri-plugin-*` | All GUI integration goes through existing `dispatch_op` / `get_state` / `sst_step` / `bst_step` / `run_stop` Tauri commands. Math Pac I ops add string IDs to `key_map.rs`, not new commands. |
| Dynamic-loading frameworks (`libloading`, `bevy_reflect`) | Static-link only — legal constraint (HP-copyrighted ROM bytes never redistributed) makes dynamic loading actively undesirable. |

---

## MSRV Impact: NONE

- `rust_decimal` 1.42: MSRV 1.85 (already on workspace pin)
- `approx` 0.5.1 (dev-dep): MSRV 1.36
- All hand-coded modules use stable Rust 1.88 features only.

Workspace MSRV 1.88 is unchanged.

---

## Zero-Panic Compatibility Audit

`#![deny(clippy::unwrap_used)]` in `hp41-core/src/lib.rs` continues to apply. Every new module's API surface returns `Result<_, HpError>`. Per-module panic-risk audit:

| Module | Panic risk | Mitigation |
|--------|------------|------------|
| `ops/xrom.rs` | None — pure lookup over `&'static [..]` | n/a |
| `ops/complex.rs` | `checked_div` on `re² + im²` zero (CDIV by zero) | Returns `HpError::DivideByZero` |
| `ops/matrix.rs` | Index-out-of-bounds on bad register descriptor; singular matrix in `INV` | `get_checked`/`set_checked` returning `Result`; pivot threshold returns `HpError::Domain` on `|piv| < 1e-10` |
| `ops/numerical.rs` | Iteration overflow on non-convergent SOLVE/INTG/POLY | Iteration cap (Math 1 OM-cited) → `HpError::Domain` on cap exceeded |
| `ops/hyperbolic.rs` | Domain errors on `acosh(x<1)`, `atanh(|x|≥1)` | Returns `HpError::Domain` |
| **`approx` (dev-only)** | `assert_relative_eq!` panics on inequality — that's its purpose | Confined to `#[cfg(test)]` modules with `#![allow(clippy::unwrap_used)]` |

No runtime dependency introduces a new panic class.

---

## Coverage Implications

The v2.2 gate `hp41-core` ≥ 95 % lines / ≥ 93 % regions (D-27.2) must be MAINTAINED for v3.0. The new ~1000–1500 LOC across the new modules will need ~60–80 test cases to reach 95 % — that's the dedicated `matrix_ops.rs` / `complex_ops.rs` / `numerical_ops.rs` / `hyperbolic_ops.rs` suites + ~150 new cases in `numerical_accuracy.rs`. Plus normalcy: the new `Op` variants need exhaustive-match coverage in `ops/program.rs::execute_op`, `prgm_display.rs::op_display_name` (BOTH copies — hp41-cli + hp41-gui — per the v2.0 invariant), and JSON canonical-source extension.

---

## Sources

- `rust_decimal`, `num-complex`, `nalgebra`, `faer`, `ndarray`, `roots`, `approx`, `peroxide`, `gauss-quad`, `argmin` — crate metadata verified via crates.io API and github.com manifest inspection on 2026-05-16
- `num-complex` Float bound on transcendentals: docs.rs/num-complex/0.4.6/num_complex/struct.Complex.html — confirmed `impl<T: Float> Complex<T>` for `sqrt`/`exp`/`ln`/`powc`
- `Decimal` does NOT impl `Float` (no IEEE-754 layout): direct inspection of `rust_decimal::Decimal` API
- HP-41C Math Pac I Owner's Manual (00041-90034, 1981) — public-domain behavioral spec (NOT ROM bytes)
- `hp41-core/Cargo.toml` — current `[dependencies]` block (4 entries: rust_decimal, thiserror, serde, serde_json)
- `hp41-core/src/num.rs:184–211` — established f64-bridge pattern for `checked_asin`/`checked_acos`/`checked_atan`
- `hp41-core/src/ops/mod.rs:1–22` — module declaration block
- `hp41-core/tests/numerical_accuracy.rs:1–120` — existing harness structure
- `.planning/PROJECT.md:7–22` — v3.0 scope; `.planning/PROJECT.md:155–157` — Math 1 Pac scope locked
- `CLAUDE.md:46–47` — settled BCD/f64 decision; `CLAUDE.md:65` — `#![deny(clippy::unwrap_used)]` invariant
