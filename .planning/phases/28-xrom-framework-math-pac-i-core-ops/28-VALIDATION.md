---
phase: 28
slug: xrom-framework-math-pac-i-core-ops
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-16
---

# Phase 28 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `28-RESEARCH.md` §Validation Architecture (lines 779–844).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (workspace) + `criterion` (benches) + `approx 0.5.1` (dev-dep, relative-tolerance assertions) |
| **Config file** | `Cargo.toml` `[dev-dependencies]` + `[[bench]]` sections; per-test attributes inline |
| **Quick run command** | `cargo test -p hp41-core --lib --test xrom_shadowing --test math1_user_callback` |
| **Full suite command** | `just test` (workspace-wide) |
| **Estimated runtime** | ~30 seconds (full); ~10 seconds (quick) |
| **Coverage gate** | `just coverage` — ≥ 95% lines on `hp41-core` (preserves v2.2 Phase 27 FN-QUAL-01) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core --lib` (~5–10 s)
- **After every plan wave:** Run `just test` (full workspace; ~30 s)
- **Before `/gsd:verify-work`:** Full suite + `just coverage` ≥ 95% + `cargo bench --bench dispatch_overhead` within budget
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

Phase 28 ships 47 REQ-IDs across 10 plans. Each plan's task list will fill row-level detail; the table below is the phase-level entry contract.

| REQ-ID | Plan | Wave | Behavior | Test Type | Automated Command | File Exists |
|--------|------|------|----------|-----------|-------------------|-------------|
| XROM-01..03 | 28-01 | 1 | `XromModule` struct + `MATH_1` const + `xrom_resolve` API | unit | `cargo test --lib xrom::tests` | ❌ W0 |
| XROM-04 | 28-01 | 1 | `xrom_resolve("SINH", 0b1)` → `Some(Op::Sinh)` | unit | `cargo test --lib xrom::tests::resolve_known` | ❌ W0 |
| XROM-05 | 28-01 | 1 | Resolver fires LAST — no Math Pac I name shadows v2.2 mnemonics | integration | `cargo test --test xrom_shadowing` | ❌ W0 |
| XROM-06 | 28-01 | 1 | 7 new `CalcState` fields with `#[serde(default)] / #[serde(skip)]` | unit | `cargo test --lib state::tests::serde_roundtrip` | ❌ W0 |
| XROM-07 | 28-01 | 1 | Resolver-chain extension at `op_xeq` / `execute_op` / `xeq_by_name_local_resolve` | integration | `cargo test --test xrom_chain_order` | ❌ W0 |
| XROM-08 | 28-07 | 4 | Nested INTG-in-SOLVE rejected with `HpError::InvalidOp` | integration | `cargo test --test math1_user_callback nested_integ_inside_solve_rejected` | ❌ W0 |
| XROM-09 | 28-01 | 1 | `ModalProgram` enum + `current_prompt()` writes to `modal_prompt` channel | unit | `cargo test --lib math1::modal::tests` | ❌ W0 |
| HYP-01..06 | 28-02 | 2 | 6 hyperbolic Ops + domain errors (Acosh<1, Atanh≥1) | unit, ≥ 5 cases each (Pitfall 16) | `cargo test --lib math1::hyperbolics::tests` | ❌ W0 |
| CMPLX-01..04 | 28-03 | 3 | `Op::{CPlus, CMinus, CTimes, CDiv}` over (ζ, τ) | unit | `cargo test --lib math1::complex::tests::arith` | ❌ W0 |
| CMPLX-05 | 28-03 | 3 | `op_c_div(0+0i)` returns `DivideByZero` BEFORE division (Pitfall 6) | unit | `cargo test --lib math1::complex::tests::cdiv_zero_divisor` | ❌ W0 |
| CMPLX-06..17 | 28-04 | 3 | 13 complex functions + branch cuts + (0,0) handling | integration | `cargo test --test math1_complex_edge_cases` | ❌ W0 |
| POLY-01..03 | 28-05 | 4 | `Op::{PolyWorkflow, Roots}` + `PolyInputStep` modal | unit | `cargo test --lib math1::poly::tests::flow` | ❌ W0 |
| POLY-04 | 28-05 | 4 | `U=u`/`V=v`/`U=u`/`-V=-v` output format (Pitfall 5) | unit + OM cite | `cargo test --lib math1::poly::tests::output_format` | ❌ W0 |
| POLY-05..07 | 28-05 | 4 | Multiplicity-as-cluster, non-convergence DATA ERROR | unit | `cargo test --lib math1::poly::tests::edges` | ❌ W0 |
| MAT-01..06 | 28-06 | 4 | `Op::{MatrixWorkflow, MatSize, MatVmat, MatEdit, MatDet, MatInv}` | unit | `cargo test --lib math1::matrix::tests::basic` | ❌ W0 |
| MAT-07 | 28-06 | 4 | `INV` singular detection at `INV_EPSILON` (Pitfall 7, OM-transcribed in 28-01) | unit, 3 cases | `cargo test --lib math1::matrix::tests::inv_*` | ❌ W0 |
| MAT-08..11 | 28-06 | 4 | `Op::{MatSimeq, MatVcol}`, ORDER ≤ 14, flag 4/5, `NO SOLUTION` | integration | `cargo test --test math1_matrix_flow` | ❌ W0 |
| INTG-01..07 | 28-07 | 4 | Discrete (A=h, B=f(xⱼ), C/D = trapezoidal/Simpson) + explicit mode | unit | `cargo test --lib math1::integ::tests` | ❌ W0 |
| INTG-08 | 28-07 | 4 | Convergence threshold tied to `DisplayMode` (Pitfall 2, OM-transcribed in 28-01) | unit, 2 cases | `cargo test --lib math1::integ::tests::threshold_*` | ❌ W0 |
| SOLV-01..03 | 28-08 | 5 | `Op::{Solve, Sol}` + modified secant + 100-iter cap | unit | `cargo test --lib math1::solve::tests::secant` | ❌ W0 |
| SOLV-04 | 28-08 | 5 | Three termination paths (`NO ROOT FOUND` / `ROOT IS` / `ROOT IS BETWEEN`) | integration, ≥ 3 cases | `cargo test --test math1_solve_paths` | ❌ W0 |
| SOLV-05..08 | 28-08 | 5 | User-callback re-entrancy via `run_loop`, nested rejection per XROM-08 | integration | `cargo test --test math1_user_callback solve_*` | ❌ W0 |
| DIFEQ-01..05 | 28-09 | 5 | `Op::Difeq` + RK4 + `ORDER=?` 1-vs-2 + step output | integration | `cargo test --lib math1::difeq::tests` | ❌ W0 |
| FOUR-01..06 | 28-10 | 5 | `Op::Four` + DFT + USER-mode E-key evaluation | integration | `cargo test --lib math1::four::tests` | ❌ W0 |
| TRI-01..05 | 28-10 | 5 | 5 triangle solvers (Law of Sines/Cosines + SSA ambiguous) | unit, ≥ 5 cases each | `cargo test --lib math1::tri::tests` | ❌ W0 |
| TRANS-01..05 | 28-10 | 5 | `Op::{Trans2d, Trans3d}` + Rodrigues rotation | unit | `cargo test --lib math1::trans::tests` | ❌ W0 |
| **All Math Pac I Ops** | meta | post | ≥ 5 tests per new Op (Pitfall 16) | meta-test | `cargo test --test math1_op_test_count` | ❌ W0 |
| **Enum bloat** | bench | post | `dispatch_overhead < 200 ns/op` (Pitfall 10; v2.2 baseline 65 ns) | criterion bench (advisory) | `cargo bench --bench dispatch_overhead` | ✅ extends v2.2 |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

The following test files must be created in Plan 28-01 (Wave 1) BEFORE Plan 28-02 implementation begins (Wave 2 entry). They scaffold the validation harness so subsequent plans land into existing CI gates.

- [ ] `hp41-core/tests/xrom_shadowing.rs` — covers XROM-05 / Pitfall 1 (iterates `MATH_1.ops`, asserts no v2.2 name collision)
- [ ] `hp41-core/tests/math1_user_callback.rs` — covers XROM-08 / Pitfall 4 (5 regression cases scaffolded; INTG/SOLVE/DIFEQ branches filled in by Plans 28-07/08/09)
- [ ] `hp41-core/tests/math1_complex_edge_cases.rs` — covers Pitfall 6 ((0,0) handling, zero-divisor branch cuts; filled by Plan 28-03/04)
- [ ] `hp41-core/tests/math1_op_test_count.rs` — grep meta-test enforcing ≥ 5 tests per new Op (Pitfall 16 CI gate)
- [ ] `hp41-core/tests/xrom_chain_order.rs` — verifies xrom-fires-LAST at all 3 resolver call sites
- [ ] Per-file header comments — every Plan 28-02..28-10 source file ships with the Free42-contamination disclaimer (`// Algorithm derived from HP-41C Math Pac I Owner's Manual 00041-90034. Free42 used as consult-only sanity oracle per Pitfall 19 / QUAL-05.`)

*`scripts/check-free42-contamination.sh` lands in Phase 32 / QUAL-05; per-file disclaimers ship from Plan 28-01 onward.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| OM page-and-paragraph transcription of INV-EPSILON | MAT-07 (Pitfall 7) | OM PDF text extraction is unreliable; human OCR + Free42 cross-check required | Plan 28-01 Task 1: open `hp41-pac-math-en.pdf` page (TBD), quote the exact INV-EPSILON constant into `docs/adr/v3.0-003-inv-epsilon.md`, verify against Free42 output for a known-singular 3×3 matrix |
| OM transcription of INTG convergence threshold formula | INTG-08 (Pitfall 2) | Same as above | Plan 28-01 Task 2: quote OM-page (TBD) into `docs/adr/v3.0-004-intg-threshold.md`, verify against Free42 for a Fix(4) vs Fix(9) of `∫₀¹ x² dx` |
| POLY complex-root cluster output format fidelity | POLY-04 (Pitfall 5) | Output is multi-line `print_buffer` text; visual diff against OM page-XX example | Plan 28-05 reviewer compares `print_buffer.join("\n")` for `x²+1` against OM-page exemplar `U=0`/`V=1`/`U=0`/`-V=-1` |

---

## Pitfall-Specific Validation Strategies

| Pitfall | Detection mechanism | CI gate |
|---------|---------------------|---------|
| **Pitfall 1** (xrom name shadowing) | `tests/xrom_shadowing.rs` iterates `MATH_1.ops`; asserts `xeq_by_name_local_resolve(name).is_none() && builtin_card_op(name).is_none()` for every entry | `cargo test --test xrom_shadowing` — REQUIRED |
| **Pitfall 2** (INTG threshold) | Constants exported as `pub const` from `ops/math1/integ.rs`; tests in `numerical_accuracy.rs` import by name; docstring carries `// Source: HP 00041-90034 p.<n>` + Free42 cross-check | `cargo test -p hp41-core --test numerical_accuracy` |
| **Pitfall 4** (call_stack overflow) | Property test: random user-callback program length 1–8 × nesting depth 0–6; assert `call_stack.len() ≤ 4` AND `≥ 4` entry returns `HpError::CallDepth` | `cargo test --test math1_user_callback` — REQUIRED |
| **Pitfall 5** (POLY cluster format) | Unit test asserts exact `print_buffer` text after `XEQ "ROOTS"` on `x²+1`: `["U=0.0000", "V=1.0000", "U=0.0000", "-V=-1.0000"]` | `cargo test --lib math1::poly::tests::output_format` |
| **Pitfall 6** (complex branch cuts) | `tests/math1_complex_edge_cases.rs`: `LnZ(0,0)`, `ZpowW(0,_)`, `complex_atan2(0,0) = 0`, `CDiv(_, 0+0i) → DivideByZero` | `cargo test --test math1_complex_edge_cases` — REQUIRED |
| **Pitfall 7** (INV singular EPSILON) | 3 unit cases in `math1::matrix::tests`: known singular (det = 0), near-singular (det ≈ INV_EPSILON), well-conditioned (det >> EPSILON) | `cargo test --lib math1::matrix::tests::inv_*` |
| **Pitfall 10** (enum bloat) | `criterion bench/dispatch_overhead.rs` extension: bench `dispatch(Op::Sinh, _)`, `dispatch(Op::CPlus, _)`, `dispatch(Op::MatInv, _)`, etc.; assert per-op median < 200 ns | `cargo bench --bench dispatch_overhead` — ADVISORY (PR description prints) |
| **Pitfall 11** (cancellation timing) | `state.cancel_requested: Arc<AtomicBool>` field lands Plan 28-01; per-64-samples check stubs ship in INTG/SOLVE/DIFEQ implementations; wiring lands Phase 31 | `cargo test --lib state::tests::cancel_field_present` (compile-time + serde-skip verify) |
| **Pitfall 16** (per-Op test count) | `tests/math1_op_test_count.rs` grep meta-test: for each `Op::*` variant in Plans 28-02..28-10, count `#[test]` functions in `hp41-core/tests/math1_*.rs` mentioning that variant by name; assert ≥ 5 | `cargo test --test math1_op_test_count` — REQUIRED |

---

## Free42 Cross-Check Protocol

For each Math Pac I numerical case landing in `numerical_accuracy.rs` (Phase 32 territory, but baseline lands per-plan from 28-02 onward):

1. Run the OM-cited input through Free42 (consult-only — see Pitfall 19 / QUAL-05).
2. Record Free42's output in the test docstring: `// Free42 v3.0.5: 1.1752 — agrees with OM p.XX`
3. The test asserts the emulator's output matches the OM-quoted value within `approx::assert_relative_eq!(max_relative = 1e-7)` (Pitfall 14).

NO Free42 source code is copied — algorithm derives from OM. Free42 is a **sanity-check oracle** only.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`xrom_shadowing.rs`, `math1_user_callback.rs`, `math1_complex_edge_cases.rs`, `math1_op_test_count.rs`, `xrom_chain_order.rs`)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter (flip after Plan 28-01 W0 files committed)

**Approval:** pending — flips to approved YYYY-MM-DD after gsd-plan-checker confirms every plan task references this VALIDATION.md
