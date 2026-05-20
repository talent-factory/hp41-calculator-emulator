# Phase 28: XROM Framework + Math Pac I Core Ops — Pattern Map

**Mapped:** 2026-05-16
**Files analyzed:** 16 new + 7 modified
**Analogs found:** 20 / 23 (3 files: novel — Simpson/RK4 kernels, MatrixView lifetime helper, op-test-count meta-test)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `hp41-core/src/ops/math1/mod.rs` (NEW) | module root | request-response | `hp41-core/src/ops/mod.rs:9-23` (mod decls) | role |
| `hp41-core/src/ops/math1/xrom.rs` (NEW) | resolver / const registry | request-response | `hp41-core/src/ops/program.rs:987-1007` (`builtin_card_op`) + `ops/mod.rs:940-974` (`synthetic_byte_to_op`) | exact |
| `hp41-core/src/ops/math1/modal.rs` (NEW) | modal state machine | event-driven | `hp41-cli/src/app.rs:43-114` (`PendingInput`) + `keys.rs:32-67` (`FlagPromptKind`/`RegisterOpKind`) | role |
| `hp41-core/src/ops/math1/hyperbolics.rs` (NEW) | unary pure-fn ops | transform | `hp41-core/src/ops/math.rs:160-247` (`op_sin`/`op_acos`) | exact |
| `hp41-core/src/ops/math1/complex.rs` (NEW) | binary + unary + domain guards | transform | `math.rs:220-247` (`op_asin` domain) + `ops/arithmetic.rs` (binary stack) | role |
| `hp41-core/src/ops/math1/poly.rs` (NEW) | modal workflow + print output | event-driven | modal opener pattern + `ops/print.rs` push to `print_buffer` | partial |
| `hp41-core/src/ops/math1/matrix.rs` (NEW) | modal + register-array storage | CRUD | `ops/registers.rs` + `op_isg` bounds-check at `program.rs:106-115` | partial |
| `hp41-core/src/ops/math1/integ.rs` (NEW) | user-callback re-entrant op | event-driven | `hp41-core/src/ops/program.rs:476-491` (`Op::XeqInd` cap + re-entry) | exact |
| `hp41-core/src/ops/math1/solve.rs` (NEW) | user-callback re-entrant op | event-driven | same as `integ.rs` | exact |
| `hp41-core/src/ops/math1/difeq.rs` (NEW) | user-callback re-entrant op | event-driven | same as `integ.rs` | exact |
| `hp41-core/src/ops/math1/four.rs`, `tri.rs`, `trans.rs` (NEW) | mixed unary + modal | mixed | hyperbolics + POLY patterns | role |
| `hp41-core/tests/xrom_shadowing.rs` (NEW) | CI gate integration test | request-response | `hp41-core/tests/phase20_math.rs:1-21` (scaffold) | exact |
| `hp41-core/tests/math1_user_callback.rs` (NEW) | integration test | event-driven | same scaffold | exact |
| `hp41-core/tests/xrom_chain_order.rs` (NEW) | integration test | request-response | same scaffold | exact |
| `hp41-core/tests/math1_op_test_count.rs` (NEW) | grep meta-test | request-response | (novel — no analog) | none |
| `hp41-core/tests/math1_complex_edge_cases.rs` (NEW) | integration test | transform | `tests/numerical_accuracy.rs` scaffold | exact |
| `hp41-core/src/state.rs` (MODIFY) | 7 new fields | — | `state.rs:103-157` (`#[serde(default)]` + `(default, skip)` precedents) | exact |
| `hp41-core/src/error.rs` (MODIFY) | `Canceled` variant | — | `error.rs:1-33` (existing thiserror variants) | exact |
| `hp41-core/src/ops/mod.rs` (MODIFY) | `Op` enum + `dispatch()` arms | — | `ops/mod.rs:104-201` (Op enum) + `:652-694` (dispatch arms) | exact |
| `hp41-core/src/ops/program.rs` (MODIFY) | resolver chain + run_loop arms | — | `program.rs:492-516` (`Op::Xeq` label-miss fallback) | exact |
| `hp41-core/src/lib.rs` (MODIFY) | mod declaration | — | `lib.rs:6-12` (`pub mod ops;`) | exact |
| `hp41-cli/src/prgm_display.rs` (Phase 29) | display formatter | — | `prgm_display.rs:27-90` (`op_display_name`) | exact |
| `hp41-gui/src-tauri/src/prgm_display.rs` (Phase 31) | display formatter | — | mirror of CLI | exact |

Phase 28 is `hp41-core`-only (CONTEXT.md §domain). The two `prgm_display.rs` rows ship in Phase 29 / 31 — captured here as the contract.

---

## Pattern Assignments

### Plan 28-01 — Framework + state + error variant + resolver chain

#### `hp41-core/src/ops/math1/xrom.rs` (NEW) — resolver + `MATH_1` const

**Analog:** `hp41-core/src/ops/program.rs:987-1007` (`builtin_card_op`).

```rust
// hp41-core/src/ops/program.rs:987-1007 (REFERENCE — copy this shape)
pub(super) fn builtin_card_op(name: &str) -> Option<Op> {
    match name {
        "WPRGM" => Some(Op::Wprgm),
        "RDPRGM" => Some(Op::Rdprgm),
        "WDTA" => Some(Op::Wdta),
        "RDTA" => Some(Op::Rdta),
        "X<>Y?" | "X\u{2260}Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY)),
        "X<Y?" => Some(Op::Test(TestKind::XLtY)),
        "X>=Y?" | "X\u{2265}Y?" => Some(Op::Test(TestKind::XGeY)),
        // ... 8 more conditional spellings
        _ => None,
    }
}
```

**Notes:**
- Copy `match name { "FOO" => Some(Op::Foo), ..., _ => None }` shape.
- `xrom_resolve` is `pub fn` (not `pub(super)`) — 3 callers reach it (`op_xeq`, `run_loop::Op::Xeq`, `xeq_by_name_local_resolve` in Phase 29).
- Accept Unicode aliases (`C×`, `C÷`, `Z↑N`, `E↑Z`) the way `builtin_card_op` accepts `X\u{2260}Y?`.
- `pub const MATH_1: XromModule = XromModule { id: 7, name: "MATH 1A", ops: &[("SINH", Op::Sinh), ...] };` co-located at bottom of file — mirrors `synthetic_byte_to_op` table placement at `ops/mod.rs:940-974`.
- SC-4 trivially preserved — no `op_*` math functions defined here.

---

#### `hp41-core/src/state.rs` (MODIFY) — 7 new fields at struct tail

**Analog:** `hp41-core/src/state.rs:103-157` (every v2.x field addition).

```rust
// state.rs:111-128 — persistent #[serde(default)] precedent
#[serde(default)]
pub last_key_code: u8,
#[serde(default)]
pub reg_m: HpNum,

// state.rs:107-110 — transient #[serde(default, skip)] precedent
#[serde(default, skip)]
pub print_buffer: Vec<String>,
```

**Notes (RESEARCH §State-Field Changes lines 656-725):**
- **Persistent fields** (`#[serde(default)]`): `xrom_modules: u8` (custom default fn returns `0b0000_0001`), `complex_mode: bool`, `matrix_dim: Option<(u8,u8)>`, `matrix_active_reg: Option<u8>`.
- **Transient fields** (`#[serde(default, skip)]`): `modal_program: Option<ModalProgram>`, `modal_prompt: Option<String>`, `integ_state: Option<IntegState>`, `solve_state: Option<SolveState>`, `difeq_state: Option<DifeqState>` (per RESEARCH Open Q2 (a) — land in 28-01).
- **`cancel_requested: Arc<AtomicBool>`** — `Arc<AtomicBool>` has no `Default` impl → use `#[serde(default = "default_cancel_requested", skip)]` with `fn default_cancel_requested() -> Arc<AtomicBool> { Arc::new(AtomicBool::new(false)) }`. Mirrors the `default_xrom_modules` shape.
- **Append fields at END of struct** to preserve serde field-order for v1.0–v2.2 save files.
- **`CalcState::new()` (`state.rs:161-189`) updated in lock-step** — every new field needs an explicit initializer.

---

#### `hp41-core/src/error.rs` (MODIFY) — add `Canceled` variant

**Analog:** `hp41-core/src/error.rs:1-33`.

```rust
// error.rs:1-33 (REFERENCE — pattern for the new variant)
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum HpError {
    #[error("overflow")]
    Overflow,
    // ...
    #[error("out of range")]
    OutOfRange,
    #[error("try again")]
    CallDepth,
    // ... add NEW here:
    /// User-initiated cancellation of long-running solver (INTG/SOLVE/DIFEQ).
    /// Distinct from `Domain`. Surfaces as "CANCELED" in GUI/CLI.
    /// D-28.7 / D-28.8 / D-28.9; wiring in Phase 31 / GUI-05.
    #[error("canceled")]
    Canceled,
}
```

**Notes:**
- thiserror's `#[error("<msg>")]` provides `Display` automatically — NO manual `impl Display` needed.
- `HpError` is never serialized — save-file forward-compat unaffected.
- **Audit:** grep `match.*HpError::` for any non-exhaustive consumer before merge.

---

#### `hp41-core/src/lib.rs` (MODIFY) — declare `math1` submodule

**Analog:** `hp41-core/src/lib.rs:6-12`.

```rust
// lib.rs:1-12 (REFERENCE)
#![deny(clippy::unwrap_used)]
pub mod cardreader;
pub mod error;
pub mod format;
pub mod num;
pub mod ops;        // ← math1 is a submodule of ops, NOT a peer
pub mod stack;
pub mod state;
```

**Notes:**
- `math1` is added inside `hp41-core/src/ops/mod.rs:9-22` alongside `alpha`, `arithmetic`, `cardreader_ops`, ...
  ```rust
  pub mod math1;        // Phase 28 — Math Pac I XROM emulation
  ```
- Optional: re-export `MATH_1` / `XromModule` at crate root for CLI ergonomics in Phase 29.

---

#### `hp41-core/src/ops/mod.rs` (MODIFY) — `Op` enum + dispatch arms

**Analog:** `ops/mod.rs:104-201` (Op enum) + `:652-694` (dispatch arms).

```rust
// ops/mod.rs:144-167 — Op enum variant doc-comment pattern
/// 1/x — reciprocal. LiftEffect: Enable.
Recip,
/// √x — square root. LiftEffect: Enable.
Sqrt,
// ...
/// SIN — sine in current angle_mode. LiftEffect: Enable.
Sin,

// ops/mod.rs:678-697 — dispatch arm shape
Op::Recip => op_recip(state),
Op::Sqrt => op_sqrt(state),
Op::Sin => op_sin(state),
Op::Cos => op_cos(state),
Op::Asin => op_asin(state),
```

**Notes:**
- Mirror exactly. Group block header: `// ── Phase 28: Hyperbolics (Plan 28-02) ────────`, then `Op::Sinh => math1::hyperbolics::op_sinh(state),`.
- **For Integ/Solve/Difeq:** dispatch arm returns `Err(HpError::InvalidOp)` — actual implementation in `run_loop` match-arm. Mirrors `Op::XeqInd` precedent at `program.rs:839+` (RESEARCH §"`execute_op` vs `run_loop` split" line 770).
- **Modal-workflow openers** (`Op::MatrixWorkflow`, `Op::Solve`, `Op::PolyWorkflow`): dispatch arm sets `state.modal_program = Some(...)`, `state.modal_prompt = Some("ORDER=?".into())`, returns `Ok(())`. State transition inside the dispatch arm, NOT in run_loop.
- `use math1::{hyperbolics::*, complex::*, ...};` at the top — mirror existing `use alpha::{...};` pattern at `ops/mod.rs:24-36`.

---

#### `hp41-core/src/ops/program.rs` (MODIFY) — resolver chain + run_loop re-entry

**Analog:** `program.rs:476-516` (`Op::XeqInd` cap, `Op::Xeq` label-miss chain).

```rust
// program.rs:476-491 — Op::XeqInd pre-mutation cap (RESEARCH line 288 calls this THE precedent)
Op::XeqInd(reg) => {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);    // pre-mutation guard (D-22.15)
    }
    let i = crate::ops::indirect::resolve_indirect_decimal(state, reg)?;
    let label_str = i.to_string();
    let target = find_in_program(program, &label_str)?;
    state.call_stack.push(state.pc);
    state.pc = target + 1;
}

// program.rs:492-516 — Op::Xeq label-miss fallback chain
Op::Xeq(label) => {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }
    match find_in_program(program, &label) {
        Ok(target) => {
            state.call_stack.push(state.pc);
            state.pc = target + 1;
        }
        Err(_) => {
            if let Some(card_op) = builtin_card_op(&label) {
                crate::ops::dispatch(state, card_op)?;
            } else {
                return Err(HpError::InvalidOp);
            }
        }
    }
}
```

**Phase 28 derived edit (C-28.4 — xrom_resolve fires LAST):**

```rust
            Err(_) => {
                if let Some(card_op) = builtin_card_op(&label) {
                    crate::ops::dispatch(state, card_op)?;
                } else if let Some(xrom_op) =                                       // NEW
                    crate::ops::math1::xrom::xrom_resolve(&label, state.xrom_modules)
                {
                    crate::ops::dispatch(state, xrom_op)?;
                } else {
                    return Err(HpError::InvalidOp);
                }
            }
```

**Notes:**
- **Three insertion sites** for `xrom_resolve` (RESEARCH §"XROM Resolver Chain"):
  1. `op_xeq` at `program.rs:67-81` (interactive path)
  2. `run_loop::Op::Xeq` at `program.rs:506` (programmatic path)
  3. `xeq_by_name_local_resolve` at `hp41-cli/src/keys.rs:347-370` — **Phase 29, NOT Phase 28**
- **Pre-mutation cap** in Plan 28-07/08/09's `op_integ` / `op_solve` / `op_difeq`: mirror `Op::XeqInd:479` EXACTLY — check `state.call_stack.len() >= 4` AND the strict-reject `state.integ_state.is_some() || state.solve_state.is_some() || state.difeq_state.is_some()` BEFORE any mutation.

---

#### `hp41-core/src/ops/math1/modal.rs` (NEW) — `ModalProgram` enum + step states

**Analog:** `hp41-cli/src/app.rs:79-93` (`PendingInput::FlagPrompt { kind, ind, acc }`) + `hp41-cli/src/keys.rs:37-42` (`FlagPromptKind`).

```rust
// app.rs:79-93 — Hybrid struct-variant pattern (D-25.11)
FlagPrompt {
    kind: FlagPromptKind,
    ind: bool,
    acc: String,
},
RegisterPrompt {
    op: RegisterOpKind,
    ind: bool,
    acc: String,
},

// keys.rs:37-42 — Discriminator that WRAPS (not duplicates) a core enum (D-25.13)
#[derive(Debug, Clone, PartialEq)]
pub enum FlagPromptKind {
    SetFlag,
    ClearFlag,
    Test(FlagTestKind),    // ← wraps hp41_core::ops::FlagTestKind
}
```

**Notes:**
- **`ModalProgram` is orthogonal to `PendingInput`** — `PendingInput` lives in `hp41-cli/src/app.rs` (CLI-local); `ModalProgram` lives in `hp41-core/src/state.rs` (state field) + `hp41-core/src/ops/math1/modal.rs` (enum def). Phase 28 does NOT touch CLI.
- **Carrier + sub-enum consolidation**: `enum ModalProgram { Matrix(MatrixInputStep), Solve(SolveInputStep), Poly(PolyInputStep), Integ(IntegInputStep), Difeq(DifeqInputStep), Four(FourInputStep), Trans(TransInputStep) }` — 7 named variants collapse the workflow space.
- **Wrap-not-duplicate**: e.g. `MatrixInputStep::ElementPrompt(u8, u8)` (row, col), `SolveInputStep::FunctionNamePrompt | Guess1Prompt | Guess2Prompt`. Per-step state per OM-cited workflow.
- **`Default::default()`-constructible constraint** for `IntegState` / `SolveState` / `DifeqState` (RESEARCH §Open Q2): `#[derive(Default)]` so `CalcState::new()` doesn't break.
- **`ModalProgram::current_prompt() -> Option<&str>`** accessor (RESEARCH §Open Q4): pure data, consumed by Phase 29's CLI `pending_prompt()` and Phase 31's GUI overlay banner. No CLI dependency in hp41-core.
- **Exhaustive-match invariant** (FN-CLI-04 from Phase 25): Phase 29's `pending_prompt()` MUST stay exhaustive — NO `_ =>` arm.

---

#### `hp41-core/tests/xrom_shadowing.rs` (NEW) — CI gate

**Analog:** `hp41-core/tests/phase20_math.rs:1-21`.

```rust
// phase20_math.rs:1-21 — integration test scaffold (REFERENCE)
//! Integration tests for Phase 20 (Core Math & Conversions).
//! ...

#![allow(clippy::unwrap_used)]    // file-scope, NOT inner mod

use hp41_core::ops::{dispatch, Op};
use hp41_core::{AngleMode, CalcState, DisplayMode, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Test body (from RESEARCH lines 129-147):**

```rust
#[test]
fn math1_names_do_not_shadow_builtins() {
    use hp41_core::ops::math1::xrom::MATH_1;
    for (name, _op) in MATH_1.ops {
        assert!(
            hp41_core::ops::program::builtin_card_op(name).is_none(),
            "Math Pac I name {name:?} shadows builtin_card_op"
        );
    }
}
```

**Notes:**
- `#![allow(clippy::unwrap_used)]` at **file scope** (NOT inside `#[cfg(test)] mod foo`). Every integration test in `hp41-core/tests/` follows this Phase 1+ convention.
- `builtin_card_op` is `pub(super) fn` at `program.rs:987` — Phase 28 must NOT widen this. Either re-export under `#[cfg(test)]` shim or accept the test-only widening. Recommendation: re-export.
- CLI's `xeq_by_name_local_resolve` shadowing assertion lands in `hp41-cli/tests/xrom_shadowing_cli.rs` in **Phase 29**, not here.

---

### Plan 28-02 — Hyperbolics

#### `hp41-core/src/ops/math1/hyperbolics.rs` (NEW) — 6 unary pure-fn ops

**Analog:** `hp41-core/src/ops/math.rs:160-247`.

```rust
// math.rs:158-168 — op_sin (f64-bridge unary)
pub fn op_sin(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let rad = to_radians_f64(v, state.angle_mode);
    let result = Decimal::from_f64(rad.sin())
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)?;
    unary_result(state, result);
    Ok(())
}

// math.rs:218-232 — op_asin with domain guard (REFERENCE for ACOSH/ATANH)
pub fn op_asin(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    if !(-1.0..=1.0).contains(&v) {
        return Err(HpError::Domain);
    }
    let rad = v.asin();
    let angle = f64_from_radians(rad, state.angle_mode);
    let result = Decimal::from_f64(angle)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}
```

**Notes:**
- Hyperbolics are **angle-mode-INDEPENDENT**. Strip the `to_radians_f64` line. SINH: `Decimal::from_f64(v.sinh())`.
- Domain guards (RESEARCH §Hyperbolics table line 477):
  - `Op::Acosh`: `X < 1` → `HpError::Domain`
  - `Op::Atanh`: `|X| >= 1` → `HpError::Domain`
- `rust_decimal` has NO `sinh`/`cosh`/`tanh` — f64 bridge is mandatory.
- **NO keyboard binding** (D-28.6) — Phase 28 only adds `Op::Sinh` etc. + `xrom_resolve` registration. Phase 29 does NOT extend `keys.rs`.
- **≥ 5 tests per Op** (Pitfall 16). Inline `#[cfg(test)] mod tests` at bottom of `hyperbolics.rs` — mirror per-file unit-test pattern in `ops/math.rs`.
- `unary_result(state, result)` applies `LiftEffect::Enable` — re-export from `crate::ops::math::unary_result` (if `pub(super)`) or factor out to `ops/helpers.rs` in Plan 28-01.

---

### Plan 28-03 + 28-04 — Complex stack + functions

#### `hp41-core/src/ops/math1/complex.rs` (NEW)

**Analog:** `math.rs:220-247` (domain-guarded unary) + `ops/arithmetic.rs` (binary stack-acting).

```rust
// Pattern (derived from math.rs:220-232 + RESEARCH lines 426-437)

/// CDIV — complex division. DivideByZero check FIRST (Pitfall 6 / CMPLX-05).
pub fn op_c_div(state: &mut CalcState) -> Result<(), HpError> {
    let c = state.stack.x.inner();    // X = re(divisor)
    let d = state.stack.y.inner();    // Y = im(divisor)
    if c.is_zero() && d.is_zero() {
        return Err(HpError::DivideByZero);    // BEFORE division
    }
    // ... (a+bi)/(c+di) on Z/T → write ζ; T-replicate fills τ
    Ok(())
}

/// complex_atan2 with (0,0) → 0 first arm (NOT NaN, NOT DATA ERROR).
fn complex_atan2(im: HpNum, re: HpNum) -> HpNum {
    if im.is_zero() && re.is_zero() { return HpNum::ZERO; }
    let result_f64 = im.to_f64().atan2(re.to_f64());
    HpNum::from_f64_rounded(result_f64)
}
```

**Notes:**
- `complex_atan2` lives **in `complex.rs`** (NOT in `num.rs`) per RESEARCH §Open Q5. First arm `(0,0) → 0` — Pitfall 6 mitigation.
- **(0,0) policy** (RESEARCH lines 440-444):
  - `LnZ(0+0i)` → `HpError::Domain`
  - `ZpowW(0+0i, w)` with `Re(w) ≤ 0` → `HpError::Domain`
  - `CDiv(_, 0+0i)` → `HpError::DivideByZero` BEFORE division
- **Stack-lift semantics** (RESEARCH lines 446-451):
  - Binary `C+/C-/C×/C÷`: consume ζ + τ (4 levels), write ζ, T-replicate from old T. `LiftEffect::Enable`.
  - Unary `MAGZ`/`CINV`/`E↑Z`/`LNZ`: consume ζ, write ζ, τ unchanged. `LiftEffect::Disable`.
- **`Op::Real` (D-28.3 / CMPLX-18):** trivial — `state.complex_mode = false; apply_lift_effect(state, LiftEffect::Neutral); Ok(())`.
- **`complex_mode` auto-on (D-28.2):** every binary/unary complex op sets `state.complex_mode = true` BEFORE the computation. Mirrors v2.2's `shift_armed` implicit-state-machine pattern.
- **`#[deny(clippy::unwrap_used)]` compliance**: `Decimal::from_f64(...).ok_or(HpError::Domain)?` — never `.unwrap()`.

---

### Plan 28-05 — POLY / ROOTS

#### `hp41-core/src/ops/math1/poly.rs` (NEW)

**Analog:** modal-opener pattern (Plan 28-01 modal.rs) + `op_prx` in `ops/print.rs` for `print_buffer` push.

```rust
// Pattern: dispatch arm opens the modal, sets first prompt, returns.
// (Derived from D-28.4 lifecycle in RESEARCH lines 273-277.)
pub fn op_poly_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Poly(PolyInputStep::DegreePrompt));
    state.modal_prompt = Some("DEGREE=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Notes:**
- **Output format gate (Pitfall 5):** roots displayed exactly as `U=u` / `V=v` / `U=u` / `-V=-v` for complex pairs. Write to `state.print_buffer` via `push` calls — NEVER `println!` (hp41-core forbids).
- **Multiplicity-as-cluster (POLY-06):** `(x-1)^5` returns 5 roots with small imaginary parts (~10⁻³). NO snap-to-zero. Document in `docs/hp41-math1-divergences.md` (Phase 30 / DOC-04).
- **`PolyInputStep`** lives in `modal.rs`: `DegreePrompt`, `CoefficientPrompt(degree, current_idx)`, `Ready`.
- **Tests use `approx::assert_relative_eq!(max_relative = 1e-7)`** per Pitfall 14 — add `approx = "0.5.1"` to `hp41-core/Cargo.toml` `[dev-dependencies]` in Plan 28-01.

---

### Plan 28-06 — MATRIX

#### `hp41-core/src/ops/math1/matrix.rs` (NEW)

**Analog:** `ops/registers.rs` (regs access) + `op_isg` bounds-check (`program.rs:106-115`).

```rust
// Bounds-check + column-major access pattern (derived from program.rs:106-115)
fn matrix_get(state: &CalcState, base: u8, rows: u8, _cols: u8, r: u8, c: u8)
    -> Result<HpNum, HpError>
{
    let idx = base as usize + (c as usize * rows as usize + r as usize);    // column-major
    if idx >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    Ok(state.regs[idx].clone())
}
```

**Notes:**
- **`pub const INV_EPSILON: HpNum`** — exact value transcribed in **Plan 28-01 Task 1** from OM HP-00041-90034 Chapter 4 (p.~14). Document verbatim OM quote + page citation in the doc-comment.
- **`MatrixView<'a>`** (RESEARCH Open Q3): module-local to `matrix.rs` for v3.0. Borrows `&'a [HpNum]` or `&'a mut`. Exposes `get(r,c)` / `set(r,c,HpNum)` / `det()` / `invert()` / `simeq()`.
- **Column-major storage (MAT-02):** elements at `R(base + c*rows + r)`. Typical: `state.matrix_active_reg = Some(15)`, `state.matrix_dim = Some((rows, cols))`.
- **Singular detection:** `|pivot| < INV_EPSILON` → `state.modal_prompt = Some("NO SOLUTION".into())` (NOT an `HpError`).
- **Flag 5 side effect** after `MatSimeq` column storage — use `crate::ops::flags::set_flag(state, 5)`.

---

### Plan 28-07 — INTG (user-callback infrastructure)

#### `hp41-core/src/ops/math1/integ.rs` (NEW)

**Analog:** `hp41-core/src/ops/program.rs:476-491` (`Op::XeqInd`) — THE precedent for user-callback re-entrancy (RESEARCH line 288).

```rust
// program.rs:476-491 — Op::XeqInd pre-mutation cap (REFERENCE)
Op::XeqInd(reg) => {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);    // pre-mutation guard (D-22.15)
    }
    let i = crate::ops::indirect::resolve_indirect_decimal(state, reg)?;
    let label_str = i.to_string();
    let target = find_in_program(program, &label_str)?;
    state.call_stack.push(state.pc);
    state.pc = target + 1;
}
```

**Phase 28-07 derived pattern (RESEARCH lines 293-330):**

```rust
pub fn op_integ(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    // 1. Strict-reject nested (XROM-08 / D-28.7 / ADR-002)
    if state.integ_state.is_some() || state.solve_state.is_some() {
        return Err(HpError::InvalidOp);
    }
    // 2. Pre-mutation call_stack cap (mirrors Op::XeqInd at program.rs:479)
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }
    // 3. Reset cancel flag at op entry (D-28.7)
    state.cancel_requested.store(false, Ordering::Relaxed);
    state.integ_state = Some(IntegState::new(/* ... */));

    for sample_idx in 0..n_samples {
        // 4. Per-64-samples cancel check (D-28.8)
        if sample_idx & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
            state.integ_state = None;
            return Err(HpError::Canceled);
        }
        // ... compute x_k, push to stack, find user_label, run_loop re-entry
    }
    state.integ_state = None;
    Ok(())
}
```

**Notes:**
- **Strict-reject BEFORE call_stack check** — both must be pre-mutation (D-22.15 invariant generalized).
- **`run_loop` re-entry (NOT `run_program`)** per C-28.5 (RESEARCH line 289). `run_program` clones; 30 KB × 1000 samples = 30 MB catastrophe. `run_loop` reuses outer clone.
- **`IntegState` struct shape** per CONTEXT.md Claude's-Discretion. RESEARCH lines 361-369 suggest: `{ user_label: String, a: HpNum, b: HpNum, n: u16, accumulator: HpNum, mode: IntegMode }`. `#[derive(Default)]` mandatory.
- **`#[serde(default, skip)]` on `state.integ_state`** — transient.
- **`dispatch` arm returns `InvalidOp`** for `Op::Integ` (mirrors `Op::XeqInd` precedent at `program.rs:839+`). Real implementation runs ONLY in the `run_loop` match-arm. Plan 28-07 ships BOTH the dispatch stub AND the run_loop body.

---

### Plan 28-08 / 28-09 — SOLVE / DIFEQ

#### `hp41-core/src/ops/math1/solve.rs` and `difeq.rs` (NEW)

**Analog:** `integ.rs` (Plan 28-07) — identical structure.

**Notes:**
- **Strict-reject guard grows in Plan 28-09:**
  ```rust
  if state.integ_state.is_some() || state.solve_state.is_some() || state.difeq_state.is_some()
  ```
- **Plan 28-01 ships `difeq_state` field** alongside the others (RESEARCH Open Q2 (a) — early commitment for symmetric save-file backward-compat).
- **`SolveState`** (RESEARCH lines 371-379): `{ user_label, x1, x2, fx1, fx2, iteration: u8 }`. Iteration cap 100 per SOLV-07.
- **Three termination paths** (Pitfall 3 / RESEARCH line 568): `NO ROOT FOUND`, `ROOT IS <v>`, `ROOT IS BETWEEN <v1> AND <v2>` — write to `state.print_buffer` (NOT `modal_prompt` — these are results, not prompts).

---

### Plan 28-10 — FOUR + Triangles + TRANS

#### `four.rs`, `tri.rs`, `trans.rs` (NEW)

**Analog:** hyperbolics (Plan 28-02) for unary triangle ops; POLY (Plan 28-05) for FOUR + TRANS modal workflows.

**Notes:**
- **TRI-05 ambiguous SSA case:** pattern-match from OM worked examples; output mirrors OM display sequence (CONTEXT.md Claude's Discretion).
- **FOUR USER-mode `E`-key (FOUR-06):** post-coefficient-computation evaluation at time `t`. Depends on user-callback infrastructure from Plan 28-07.
- **Keep as one plan** unless plan-checker flags oversized (RESEARCH §"Revision suggestion" line 866).

---

## Shared Patterns

### Pattern 1 — `#![deny(clippy::unwrap_used)]` crate boundary

**Source:** `hp41-core/src/lib.rs:1`

```rust
#![deny(clippy::unwrap_used)]
```

**Apply to:** every `hp41-core/src/ops/math1/*.rs` file (NEW) — inherited at crate root, no per-file declaration needed.

**Test files** in `hp41-core/tests/math1_*.rs` carry `#![allow(clippy::unwrap_used)]` at file scope (mirror `tests/phase20_math.rs:15`).

### Pattern 2 — SC-4 invariant (no calc/math logic in hp41-gui)

**Source:** CLAUDE.md §"v2.0 additions" SC-4 paragraph.
**Apply to:** all of Phase 28 (trivially preserved — `hp41-gui` untouched).

Audit (stricter form per CLAUDE.md):
```bash
grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" \
  hp41-gui/src-tauri/src/
```

### Pattern 3 — 4-place exhaustive-match for every new `Op` variant

**Source:** CLAUDE.md §"v2.0 additions" — "Op variants land before TUI code".

| Place | File | Phase |
|-------|------|-------|
| 1 | `hp41-core/src/ops/mod.rs::Op` enum | 28 |
| 2 | `hp41-core/src/ops/mod.rs::dispatch()` arm | 28 |
| 3 | `hp41-cli/src/prgm_display.rs::op_display_name` | **29** |
| 4 | `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` | **31** |

Phase 28's compile boundary = (1) + (2). Places (3) + (4) refuse to compile in Phase 29/31 unless arms are added — structural enforcement.

**Excerpt — `hp41-cli/src/prgm_display.rs:27-66` (template for Phase 29 arms):**

```rust
fn op_display_name(op: &Op) -> String {
    match op {
        Op::Add => "+ ".to_string(),
        Op::Sub => "- ".to_string(),
        // ...
        Op::Sin => "SIN".to_string(),
        Op::Cos => "COS".to_string(),
        // ...
    }
}
```

Phase 29 mirror: `Op::Sinh => "SINH".to_string(),`, `Op::MatrixWorkflow => "MATRIX".to_string(),`, `Op::Real => "REAL".to_string(),`, etc.

The `hp41-gui/src-tauri/src/prgm_display.rs` copy is identical structurally (CLAUDE.md notes the duplication as the documented exception to SC-4 spirit-vs-literal).

### Pattern 4 — `criterion bench` enum-bloat regression guard

**Source:** `hp41-core/benches/dispatch_bench.rs:17-50`.

```rust
// dispatch_bench.rs:17-50 (REFERENCE)
fn bench_dispatch_mixed(c: &mut Criterion) {
    let ops: Vec<Op> = vec![
        Op::PushNum(HpNum::from(3i32)),
        Op::Enter,
        // ... 18 more
        Op::Sin, Op::Cos, Op::Tan, Op::Asin,
        Op::StoReg(0), Op::RclReg(0),
        // ...
    ];
    c.bench_function("dispatch_mixed_20ops", |b| {
        b.iter(|| {
            let mut state = CalcState::default();
            for op in &ops { let _ = dispatch(&mut state, op.clone()); }
        });
    });
}
```

**Notes:**
- File is `dispatch_bench.rs` in v2.2 (NOT `dispatch_overhead.rs` as RESEARCH text says — verify the file name in Plan 28-01).
- Plan 28-01 adds `bench_dispatch_math1` group with `Op::Sinh`, `Op::MatInv`, `Op::Integ`, `Op::PolyWorkflow` samples — one per plan family.
- **Advisory only — NOT CI-gated.** Result reported in PR description.
- Baseline 65 ns/op (v2.2); floor < 200 ns/op (Pitfall 10).

### Pattern 5 — `print_buffer` vs `modal_prompt` channel separation (D-28.4)

| Channel | Lifecycle | Used for |
|---------|-----------|----------|
| `state.print_buffer: Vec<String>` (existing) | Drained on every dispatch | PRX/PRA/PRSTK + ROOTS U=u/V=v + DIFEQ step output + SOLVE result lines |
| `state.modal_prompt: Option<String>` (NEW) | Set on prompt-open; cleared on prompt-resolve | `"ORDER=?"`, `"A1,1=?"`, `"FUNCTION NAME?"`, `"GUESS 1=?"`, `"NO SOLUTION"` |

NEVER use `print_buffer` for prompts (rejected by D-28.4). NEVER use `modal_prompt` for solver-result output.

---

## No Analog Found

| File | Role | Plan | Why |
|------|------|------|-----|
| `hp41-core/tests/math1_op_test_count.rs` | grep meta-test (Pitfall 16) | 28-01 | Novel CI gate; closest is v2.2 `function_matrix_parity.rs` but count-mentions-by-name approach is new. |
| `MatrixView<'a>` borrow-helper | matrix.rs | 28-06 | No v2.2 lifetime-bearing slice view; closest is `state.regs` direct indexing in `op_isg` (`program.rs:106-115`). |
| Simpson / RK4 / secant numerical kernels | integ/solve/difeq | 28-07/08/09 | v2.2 has no numerical-method kernels. RESEARCH §"Per-Program Implementation Notes" + OM transcription (Plan 28-01 Task 1) are the only sources. |
| `complex_atan2` + complex newtype | complex.rs | 28-03 | No complex-number primitives in v2.2. The f64-bridge precedent applies to per-axis arithmetic; (0,0)→0 first arm + branch-cut policy is new. |

---

## Metadata

**Analog search scope:**
- `hp41-core/src/{lib.rs, error.rs, state.rs, num.rs, stack.rs, format.rs}`
- `hp41-core/src/ops/{mod.rs, program.rs, math.rs, registers.rs, arithmetic.rs, print.rs, flags.rs, indirect.rs}`
- `hp41-core/tests/{phase20_math.rs, numerical_accuracy.rs}` (scaffold inspection)
- `hp41-core/benches/dispatch_bench.rs`
- `hp41-cli/src/{app.rs, keys.rs, prgm_display.rs}`
- `hp41-gui/src-tauri/src/prgm_display.rs`

**Files scanned:** ~20
**Pattern extraction date:** 2026-05-16
**Phase boundary:** `hp41-core`-only (per CONTEXT.md §domain line 21)
**SC-4 invariant:** trivially preserved — zero `hp41-gui/src-tauri/src/` source touched

---

## PATTERN MAPPING COMPLETE

**Phase:** 28 — XROM Framework + Math Pac I Core Ops
**Files classified:** 23 (16 new + 7 modified)
**Analogs found:** 20 / 23 (4 novel patterns documented under "No Analog Found")

### Coverage
- Exact-match analog: 14 (resolver chain, dispatch arms, serde fields, error variant, test scaffolds, unary ops, prgm_display, bench, `Op::XeqInd` re-entrancy)
- Role-match analog: 5 (modal state machine, complex ops, POLY, FOUR/TRI/TRANS, matrix register-access)
- No analog: 4 (Simpson/RK4 kernels, complex_atan2, op_test_count meta-test, MatrixView lifetime helper)

### Key Patterns Identified
- `xrom_resolve` mirrors `builtin_card_op` (`program.rs:987-1007`) — string→Op resolver returning regular `Op::*`; never bypasses the exhaustive match.
- `Op::XeqInd` (`program.rs:476-491`) is THE precedent for user-callback re-entrancy: pre-mutation `call_stack >= 4` cap → push → re-enter `run_loop`. Plans 28-07/08/09 mirror exactly.
- `PendingInput::FlagPrompt { kind, ind, acc }` + `FlagPromptKind` (wrap, don't duplicate) is the hybrid-struct-variant precedent for `ModalProgram::Matrix(MatrixInputStep)` etc.
- `#[serde(default)]` vs `#[serde(default, skip)]` precedents in `state.rs` (`last_key_code`, `print_buffer`, `display_override`) cover every Phase 28 field-addition shape.
- 4-place exhaustive-match invariant: Phase 28 owns places 1+2 (Op enum + dispatch); Phase 29+31 own places 3+4 (two `prgm_display.rs` copies).

### File Created
`/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-PATTERNS.md`

### Ready for Planning
Planner can now reference exact v2.2 analog file paths + line numbers in every Plan 28-01..28-10 action section.
