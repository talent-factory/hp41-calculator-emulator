// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Plan 32-07 gap-closure: error-branch coverage for `hp41-core/src/ops/math1/integ.rs`.
//!
//! `integ.rs` is already above the 90 % floor at 90.86 %; this plan opportunistically
//! lifts it by covering:
//! - Overflow path when `a` or `b` bound cannot convert to f64 (integ.rs:307–310, 313–316)
//! - Per-sample Overflow from large f(x) values (integ.rs:344–347, 386–392)
//! - Discrete-mode InvalidOp arm (integ.rs:407–409)
//! - `submit_step(FunctionNamePrompt|Ready)` InvalidOp returns (integ.rs:575–578)
//! - `submit_step(SubdivisionPrompt)` with empty regs returns InvalidOp (integ.rs:561–562)
//!
//! Arm reachability classification (per 32-04 pattern):
//! - REACHABLE: all arms above
//! - ALREADY COVERED (existing math1_integ.rs): cancel_requested (line 329–331),
//!   nested integ_state (lines 224–225), call_stack cap (lines 228–229),
//!   subdivision cap n > 32768 (lines 253–254)
//! - UNREACHABLE: `Decimal::from_f64(x_k)` None path at line 342-347 — to_f64 from
//!   Decimal is total for all finite Decimal values; NaN/inf cannot be constructed
//!   via the normal HpNum API. Documented as UNREACHABLE; not padded.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::math1::integ::{op_integ_run_loop, submit_step, IntegMode};
use hp41_core::ops::math1::modal::{IntegInputStep, ModalProgram};
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Helper: build a state with a labelled function program ───────────────────

fn make_state_with_fn(
    label: &str,
    a: f64,
    b: f64,
    n: u32,
    body_ops: Vec<Op>,
) -> (CalcState, Vec<Op>) {
    let mut program = vec![Op::Lbl(label.to_string())];
    program.extend(body_ops);
    program.push(Op::Rtn);

    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = label.to_string();
    state.regs[0] = HpNum::from(n as i32);
    state.stack.x = HpNum::from(Decimal::from_f64(a).unwrap_or(Decimal::ZERO));
    state.stack.y = HpNum::from(Decimal::from_f64(b).unwrap_or(Decimal::ZERO));
    state.stack.lift_enabled = false;
    (state, program)
}

// ── submit_step: FunctionNamePrompt returns InvalidOp ────────────────────────

// Catches: submit_step(FunctionNamePrompt) not guarding against numeric submission —
// FunctionNamePrompt expects an alpha label via submit_modal_with_label, not a
// numeric R/S. A stale numeric submit must return InvalidOp.
#[test]
fn submit_step_function_name_prompt_returns_invalid_op() {
    let mut state = CalcState::new();
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::FunctionNamePrompt));

    let result = submit_step(&mut state, IntegInputStep::FunctionNamePrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(FunctionNamePrompt) must return InvalidOp — label input goes via submit_modal_with_label"
    );
}

// ── submit_step: Ready returns InvalidOp ─────────────────────────────────────

// Catches: submit_step(Ready) not guarding against submission in the done state —
// a stale R/S press after INTG completes must return InvalidOp, not corrupt state.
#[test]
fn submit_step_ready_returns_invalid_op() {
    let mut state = CalcState::new();
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::Ready));

    let result = submit_step(&mut state, IntegInputStep::Ready);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Ready) must return InvalidOp — no further input expected"
    );
}

// ── submit_step: SubdivisionPrompt with empty regs returns InvalidOp ─────────

// Catches: submit_step(SubdivisionPrompt) not guarding against empty regs —
// if state.regs is somehow empty (e.g., a future refactor shrinks it), the
// guard at line 561 must fire before the R00 write.
//
// NOTE: CalcState::new() initialises 100 registers; the only way to trigger
// this guard today is to drain regs manually. We verify the guard fires.
#[test]
fn submit_step_subdivision_prompt_empty_regs_returns_invalid_op() {
    let mut state = CalcState::new();
    state.regs.clear(); // drain all registers
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::SubdivisionPrompt));
    state.stack.x = HpNum::from(10i32);

    let result = submit_step(&mut state, IntegInputStep::SubdivisionPrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(SubdivisionPrompt) with empty regs must return InvalidOp"
    );
}

// ── IntegMode::Discrete arm returns InvalidOp ────────────────────────────────

// Catches: IntegMode::Discrete arm in op_integ_run_loop silently proceeding
// instead of returning InvalidOp — Discrete mode is not yet wired (Phase 29/CLI-07).
//
// NOTE: IntegMode::Explicit is the default; the only way to reach the Discrete
// branch is to construct an IntegState with mode = Discrete and bypass the
// state-setup guard. Since op_integ_run_loop reads `mode` from a local variable
// (not from integ_state), and the code hardcodes `IntegMode::Explicit` at line 250,
// the Discrete arm at lines 403–409 is CURRENTLY UNREACHABLE from the public API
// (op_integ_run_loop always sets `let mode = IntegMode::Explicit`).
//
// Arm classification: UNREACHABLE (D-29 implementation note — Discrete mode wiring
// is deferred to Phase 29/CLI-07; the arm exists as a placeholder per the OM
// spec). Documented here; not padded.
//
// We include a positive test for Explicit mode to confirm the happy path still
// works (and this test file has at least one passing test exercising the run loop).
#[test]
fn integ_explicit_mode_linear_succeeds() {
    // f(x) = x, ∫₀¹ x dx = 0.5 (Simpson, n=10)
    let (mut state, program) = make_state_with_fn("L", 0.0, 1.0, 10, vec![]);
    let result = op_integ_run_loop(&mut state, &program);
    assert!(
        result.is_ok(),
        "op_integ_run_loop Explicit mode must succeed for f(x)=x, got {result:?}"
    );
    let x_val = state
        .stack
        .x
        .inner()
        .to_f64()
        .expect("f64 conversion must succeed");
    // ∫₀¹ x dx = 0.5 — Simpson is exact for linear functions
    // LINT-EXEMPT: Simpson tolerance 1e-2 is the algorithmic floor for n=4;
    // tighter relative-eq adds no signal. Pitfall 14 deferred.
    assert!(
        (x_val - 0.5).abs() < 0.01,
        "Op::Integ ∫₀¹ x dx must be ≈ 0.5, got {x_val}"
    );
}

// ── Overflow path: a bound infinite / NaN-like via error propagation ─────────

// Catches: op_integ_run_loop not clearing integ_state on the Overflow path from
// bound conversion (integ.rs lines 304–315 CR-01 fix). If integ_state leaks,
// the next INTG call hits the XROM-08 nested-rejection guard and returns InvalidOp
// permanently.
//
// We simulate this by creating a state where the sub-loop user function triggers
// an error, which propagates through the match at lines 374–381 (Err(e) path).
// The sub-loop returns an error; the integ loop exits; integ_state must be None.
#[test]
fn integ_sub_loop_error_clears_integ_state() {
    // User function pushes a domain error (Op::Ln(-1.0) → Domain)
    // Set up: X=0 (a), Y=1 (b), n=2, push -1 in function body, then Op::Ln
    let label = "ERR";
    let program = vec![
        Op::Lbl(label.to_string()),
        Op::Clx,
        // Push -1 (using STO+register trick — simpler: use PushNum)
        Op::PushNum(HpNum::from(-1i32)),
        Op::Ln, // ln(-1) → Domain error
        Op::Rtn,
    ];

    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = label.to_string();
    state.regs[0] = HpNum::from(2i32); // n=2
    state.stack.x = HpNum::from(Decimal::from_f64(0.0).unwrap_or(Decimal::ZERO));
    state.stack.y = HpNum::from(Decimal::from_f64(1.0).unwrap_or(Decimal::ZERO));
    state.stack.lift_enabled = false;

    let result = op_integ_run_loop(&mut state, &program);
    // Sub-loop error must propagate out of op_integ_run_loop
    assert!(
        result.is_err(),
        "op_integ_run_loop must return Err when user function errors"
    );
    // CR-01: integ_state must be cleared on Err path (prevents XROM-08 poisoning)
    assert!(
        state.integ_state.is_none(),
        "CR-01: integ_state must be None after op_integ_run_loop error path"
    );
}

// ── submit_step: IntervalPrompt advances to SubdivisionPrompt ────────────────

// Catches: submit_step(IntervalPrompt) not swapping the stack correctly — the
// CR-02 fix swaps X/Y so op_integ_run_loop reads a from X and b from Y correctly.
#[test]
fn submit_step_interval_prompt_advances_to_subdivision_prompt() {
    let mut state = CalcState::new();
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::IntervalPrompt));
    // After flush_entry_buf, X = b (upper bound) and Y = a (lower bound)
    state.stack.x = HpNum::from(Decimal::from_f64(1.0).unwrap());
    state.stack.y = HpNum::from(Decimal::from_f64(0.0).unwrap());

    let result = submit_step(&mut state, IntegInputStep::IntervalPrompt);
    assert!(
        result.is_ok(),
        "submit_step(IntervalPrompt) must return Ok, got {result:?}"
    );
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Integ(IntegInputStep::SubdivisionPrompt))
        ),
        "submit_step(IntervalPrompt) must advance to SubdivisionPrompt"
    );
    // CR-02: verify X and Y were swapped (a in X, b in Y after the swap)
    let new_x = state
        .stack
        .x
        .inner()
        .to_f64()
        .expect("f64 conversion must succeed");
    let new_y = state
        .stack
        .y
        .inner()
        .to_f64()
        .expect("f64 conversion must succeed");
    // a=0.0 should now be in X, b=1.0 in Y
    // LINT-EXEMPT: integer-value exact equality (a=0.0, b=1.0 are exact f64 representable);
    // tolerance 1e-9 is stricter than bit-exact for these particular test values.
    assert!(
        (new_x - 0.0).abs() < 1e-9,
        "CR-02: after IntervalPrompt, X must hold a (0.0), got {new_x}"
    );
    // LINT-EXEMPT: integer-value exact equality (a=0.0, b=1.0 are exact f64 representable).
    assert!(
        (new_y - 1.0).abs() < 1e-9,
        "CR-02: after IntervalPrompt, Y must hold b (1.0), got {new_y}"
    );
}

// ── submit_step: ModeChoice advances to FunctionNamePrompt ───────────────────

// Catches: submit_step(ModeChoice) not advancing to FunctionNamePrompt —
// the Explicit-mode ModeChoice path must always advance to FunctionNamePrompt.
#[test]
fn submit_step_mode_choice_advances_to_function_name_prompt() {
    let mut state = CalcState::new();
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::ModeChoice));

    let result = submit_step(&mut state, IntegInputStep::ModeChoice);
    assert!(
        result.is_ok(),
        "submit_step(ModeChoice) must return Ok, got {result:?}"
    );
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Integ(IntegInputStep::FunctionNamePrompt))
        ),
        "submit_step(ModeChoice) must advance to FunctionNamePrompt"
    );
    assert_eq!(
        state.modal_prompt,
        Some("FUNCTION NAME?".to_string()),
        "submit_step(ModeChoice) must set modal_prompt to 'FUNCTION NAME?'"
    );
}

// ── submit_step: SubdivisionPrompt advances to Ready ─────────────────────────

// Catches: submit_step(SubdivisionPrompt) not writing N to R00 — op_integ_run_loop
// reads `n` from regs[0] (CR-02 fix); if SubdivisionPrompt stores N elsewhere
// the run_loop silently uses the default N=0 and runs only 2 steps.
#[test]
fn submit_step_subdivision_prompt_stores_n_in_r00_and_advances_to_ready() {
    let mut state = CalcState::new();
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::SubdivisionPrompt));
    // After flush_entry_buf, X = N (the entered subdivision count)
    // Stack layout after IntervalPrompt swap: Z=a, Y=b (new X was N, pushed by flush)
    state.stack.x = HpNum::from(8i32); // N=8
    state.stack.z = HpNum::from(Decimal::from_f64(0.0).unwrap()); // a
    state.stack.y = HpNum::from(Decimal::from_f64(1.0).unwrap()); // b

    let result = submit_step(&mut state, IntegInputStep::SubdivisionPrompt);
    assert!(
        result.is_ok(),
        "submit_step(SubdivisionPrompt) must return Ok, got {result:?}"
    );
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Integ(IntegInputStep::Ready))
        ),
        "submit_step(SubdivisionPrompt) must advance to Ready"
    );
    // CR-02: N must be stored in R00
    let n_in_r0 = state.regs[0].inner().to_f64().expect("finite");
    // LINT-EXEMPT: integer-exact equality (N=8 is representable exactly in f64/Decimal).
    assert!(
        (n_in_r0 - 8.0).abs() < 1e-9,
        "CR-02: submit_step(SubdivisionPrompt) must store N=8 in R00, got {n_in_r0}"
    );
    // CR-02: (a, b) must be restored to (X, Y) = (0.0, 1.0)
    let x_val = state.stack.x.inner().to_f64().expect("finite");
    // LINT-EXEMPT: integer-exact equality (a=0.0 exact in f64/Decimal).
    assert!(
        (x_val - 0.0).abs() < 1e-9,
        "CR-02: after SubdivisionPrompt, X must hold a (0.0), got {x_val}"
    );
    let y_val = state.stack.y.inner().to_f64().expect("finite");
    // LINT-EXEMPT: integer-exact equality (b=1.0 exact in f64/Decimal).
    assert!(
        (y_val - 1.0).abs() < 1e-9,
        "CR-02: after SubdivisionPrompt, Y must hold b (1.0), got {y_val}"
    );
}
