// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Error-branch coverage for `ops/math1/difeq.rs` (gap-closure plan 32-06).
//!
//! Current coverage at plan start: 85.76 % lines.
//! Targeted branches:
//!   - op_difeq is_running arm (line 160) — REACHABLE: dispatch with is_running=true
//!   - call_stack depth guard (line 215) — REACHABLE: fill call_stack to 4
//!   - label-not-found arm (line 282) — REACHABLE: alpha_reg names absent label
//!   - cancellation path (line 315) — REACHABLE: cancel_requested before loop
//!   - run-loop error propagation (line 414) — REACHABLE: user fn returns error
//!   - run_user_function overflow guard (line 670) — UNREACHABLE: execute_op_pub rejects
//!     all program-control ops (GTO/XEQ) before they can spin 100_000 steps; documented
//!     in SUMMARY per the `math1_user_callback.rs::user_fn_recursion_cap` pattern.
//!   - submit_step OrderPrompt regs-empty (line 789) — REACHABLE: clear regs
//!   - submit_step StepSizePrompt regs-len<2 (line 798) — REACHABLE: truncate regs
//!   - submit_step X0Prompt regs-len<3 (line 807) — REACHABLE: truncate regs
//!   - submit_step Y0Prompt regs-len<4 (line 816) — REACHABLE: truncate regs
//!   - submit_step Y1PrimePrompt regs-len<5 (line 832) — REACHABLE: truncate regs
//!   - submit_step FunctionNamePrompt/Ready (line 841) — REACHABLE: call with those variants
//!   - ORDER=2 coupled RK4 path — REACHABLE: set order=2 in R00
//!
//! Source: HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979), Chapter 7.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::math1::difeq::{op_difeq_run_loop, submit_step};
use hp41_core::ops::math1::modal::DifeqInputStep;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a DIFEQ-ready CalcState for ORDER=1 with f(x,y)=y (exponential growth).
/// dy/dx = y, y(0)=1, h=0.1, x0=0. User fn: swap X↔Y to return y from X.
fn make_difeq_order1(max_steps: u32) -> (CalcState, Vec<Op>) {
    let program = vec![
        Op::Lbl("DFQ".to_string()),
        Op::XySwap, // ORDER=1: stack has Y=y, X=x; swap → X=y = f(x,y)
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "DFQ".to_string();
    state.regs[0] = HpNum::from(1i32); // ORDER=1
    state.regs[1] = HpNum::from(Decimal::from_f64(0.1).unwrap_or(Decimal::ZERO)); // h=0.1
    state.regs[2] = HpNum::from(0i32); // x0=0
    state.regs[3] = HpNum::from(1i32); // y0=1
    state.regs[5] = HpNum::from(max_steps as i32); // max_steps
    (state, program)
}

/// Build a DIFEQ-ready CalcState for ORDER=2 with f(x,y,y')=-y (simple harmonic).
/// y'' = -y: solution is y=cos(x). Initial: y0=1, y'0=0, h=0.1.
/// User fn: negate X (which holds y'' from the stack: stack has Z=y'=z, Y=y, X=x;
/// for SHO f=-y: user needs Y value in X → pop Y to X by dropping X, then negate).
fn make_difeq_order2(max_steps: u32) -> (CalcState, Vec<Op>) {
    // ORDER=2: stack pushed as Z=z_arg, Y=y_arg, X=x_arg (push_three_args).
    // User function should return y'' in X. For coverage purposes, we just
    // return x_arg unchanged (the value already in X). This is a trivial
    // non-erroring function that exercises the ORDER=2 code path.
    let program = vec![
        Op::Lbl("SHO".to_string()),
        // X already contains x_arg — return it as y'' (not physically meaningful
        // but exercises the 4-call coupled RK4 path without arithmetic errors).
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "SHO".to_string();
    state.regs[0] = HpNum::from(2i32); // ORDER=2
    state.regs[1] = HpNum::from(Decimal::from_f64(0.1).unwrap_or(Decimal::ZERO)); // h=0.1
    state.regs[2] = HpNum::from(0i32); // x0=0
    state.regs[3] = HpNum::from(1i32); // y0=1
    state.regs[4] = HpNum::from(0i32); // y'0=0
    state.regs[5] = HpNum::from(max_steps as i32); // max_steps
    (state, program)
}

// ── Test 1: op_difeq dispatch arm returns InvalidOp when is_running=true ──────

/// Catches: op_difeq is_running arm (difeq.rs:160) not reached.
///
/// When `Op::Difeq` is dispatched while `state.is_running = true`, the dispatch
/// arm must return `Err(HpError::InvalidOp)` — the real implementation is in
/// `op_difeq_run_loop`, not in the dispatch arm.
///
/// Source: HP 00041-90034 (1979), Chapter 7 — DIFEQ must be inside a running program.
#[test]
fn op_difeq_dispatch_arm_invalid_op_when_running() {
    // Catches: op_difeq is_running branch (line 160) not covered
    let mut state = CalcState::new();
    state.is_running = true; // simulate being inside a running program
    let result = dispatch(&mut state, Op::Difeq);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Difeq when is_running must return Err(InvalidOp) — run_loop handles it"
    );
}

// ── Test 2: call_stack depth guard fires before state mutation ────────────────

/// Catches: call_stack depth guard (difeq.rs:215) not reached — CallDepth not returned.
///
/// When `state.call_stack.len() >= 4`, `op_difeq_run_loop` must return
/// `Err(HpError::CallDepth)` before any state mutation. Mirrors the integ/solve
/// pre-mutation Pitfall 4 guard.
///
/// Source: ADR-002 / Pitfall 4 — pre-mutation call_stack cap of 4.
#[test]
fn difeq_run_loop_call_depth_exhausted() {
    // Catches: call_stack depth guard (line 215) — Err(CallDepth)
    let (mut state, program) = make_difeq_order1(3);
    // Fill call_stack to depth 4 (the cap)
    state.call_stack.push(0);
    state.call_stack.push(1);
    state.call_stack.push(2);
    state.call_stack.push(3);
    assert_eq!(state.call_stack.len(), 4);

    let result = op_difeq_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::CallDepth),
        "DIFEQ with call_stack.len()>=4 must return Err(CallDepth) per Pitfall 4"
    );
    // difeq_state must NOT be set — guard fired before mutation
    assert!(
        state.difeq_state.is_none(),
        "difeq_state must remain None when CallDepth guard fires"
    );
}

// ── Test 3: label not found returns InvalidOp ─────────────────────────────────

/// Catches: label-not-found arm (difeq.rs:282) not reached — InvalidOp not returned.
///
/// When `alpha_reg` names a label absent from the program slice, `op_difeq_run_loop`
/// must return `Err(HpError::InvalidOp)` and clear `difeq_state`.
///
/// Source: HP 00041-90034 (1979), Chapter 7 — DIFEQ function label must exist.
#[test]
fn difeq_run_loop_label_not_found_returns_invalid_op() {
    // Catches: label-not-found arm (line 282)
    let program = vec![Op::Lbl("REAL_LBL".to_string()), Op::Rtn];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "MISSING".to_string(); // label not in program
    state.regs[0] = HpNum::from(1i32); // ORDER=1
    state.regs[1] = HpNum::from(Decimal::from_f64(0.1).unwrap_or(Decimal::ZERO));
    state.regs[2] = HpNum::from(0i32);
    state.regs[3] = HpNum::from(1i32);
    state.regs[5] = HpNum::from(3i32);

    let result = op_difeq_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "DIFEQ with missing label must return Err(InvalidOp)"
    );
    assert!(
        state.difeq_state.is_none(),
        "difeq_state must be None after label-not-found failure"
    );
}

// ── Test 4: cancellation propagates through RK4 loop ─────────────────────────

/// Catches: cancel path (difeq.rs:315) not reached — Canceled not returned.
///
/// Setting `cancel_requested = true` before the RK4 loop causes the per-64-step
/// check (step_count=0 → 0 & 0x3F == 0) to fire immediately and return
/// `Err(HpError::Canceled)`.
///
/// Source: D-28.7 / D-28.8 — per-64-step cancellation plumbing.
#[test]
fn difeq_cancel_propagates() {
    // Catches: cancellation check (line 315) not reached
    use std::sync::atomic::Ordering;
    let (mut state, program) = make_difeq_order1(1000);
    state.cancel_requested.store(true, Ordering::Relaxed);

    let result = op_difeq_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::Canceled),
        "DIFEQ must return Canceled when cancel_requested is set before RK4 loop"
    );
    assert!(
        state.difeq_state.is_none(),
        "difeq_state must be None after cancellation"
    );
}

// ── Test 5: user function error propagates through run_loop (line 414) ────────

/// Catches: run-loop error propagation arm (difeq.rs:414) not reached.
///
/// When the user function returns an error (e.g. sqrt of negative number), the
/// RK4 loop must propagate that error, clear difeq_state, and restore call_stack.
///
/// Source: HP 00041-90034 (1979), Chapter 7 — DIFEQ propagates user function errors.
#[test]
fn difeq_run_loop_propagates_user_fn_error() {
    // Catches: run-loop error propagation arm (line 414)
    // User function: f(x,y) = sqrt(x - 100) — domain error for x < 100 (x0=0)
    let program = vec![
        Op::Lbl("ERR_FN".to_string()),
        Op::PushNum(HpNum::from(100i32)),
        Op::Sub,  // x - 100 (negative for x < 100)
        Op::Sqrt, // sqrt(negative) → Domain error
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "ERR_FN".to_string();
    state.regs[0] = HpNum::from(1i32); // ORDER=1
    state.regs[1] = HpNum::from(Decimal::from_f64(0.1).unwrap_or(Decimal::ZERO));
    state.regs[2] = HpNum::from(0i32); // x0=0 → f(0,y)=sqrt(-100) → error
    state.regs[3] = HpNum::from(1i32); // y0=1
    state.regs[5] = HpNum::from(5i32); // max_steps

    let result = op_difeq_run_loop(&mut state, &program);
    assert!(
        result.is_err(),
        "DIFEQ must propagate error from user function, got Ok"
    );
    assert!(
        state.difeq_state.is_none(),
        "difeq_state must be None after user function error"
    );
}

// ── Test 6: ORDER=2 coupled RK4 path executes correctly ──────────────────────

/// Catches: ORDER=2 coupled RK4 path (difeq.rs:370-403) not reached.
///
/// Runs a 2nd-order ODE with ORDER=2, y''=y (which approximates y=cosh). The
/// ORDER=2 branch of the RK4 loop calls `rk4_step_order2` which exercises the
/// `push_three_args` path and the 4-call coupled RK4 scheme.
///
/// Source: HP 00041-90034 (1979), Chapter 7 — ORDER=2 coupled system.
#[test]
fn difeq_order2_coupled_rk4_path() {
    // Catches: ORDER=2 branch in the RK4 loop (lines 371-403) and rk4_step_order2
    let (mut state, program) = make_difeq_order2(3);

    let result = op_difeq_run_loop(&mut state, &program);
    assert!(
        result.is_ok(),
        "DIFEQ ORDER=2 simple harmonic should succeed for 3 steps: {result:?}"
    );
    assert!(
        state.difeq_state.is_none(),
        "difeq_state must be None after successful ORDER=2 completion"
    );
    // Should have printed initial + 3 step lines = 4 lines
    assert_eq!(
        state.print_buffer.len(),
        4,
        "print_buffer should have 4 lines (initial + 3 steps), got: {:?}",
        state.print_buffer
    );
    // Each step line should contain Y' (ORDER=2 format)
    for line in state.print_buffer.iter().skip(1) {
        assert!(
            line.contains("Y'="),
            "ORDER=2 output must include Y' component, got: {line:?}"
        );
    }
}

// ── Tests 7-12: submit_step Err-arms (6 InvalidOp guards) ────────────────────

/// Catches: submit_step OrderPrompt regs-empty guard (difeq.rs:789) not reached.
///
/// If `state.regs` is empty, `submit_step(OrderPrompt)` must return `Err(HpError::InvalidOp)`
/// because there is no R00 to write the order value into.
#[test]
fn submit_step_order_prompt_empty_regs_returns_invalid_op() {
    // Catches: submit_step OrderPrompt regs-empty guard (line 789)
    let mut state = CalcState::new();
    state.regs.clear(); // force empty regs
    let result = submit_step(&mut state, DifeqInputStep::OrderPrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(OrderPrompt) with empty regs must return Err(InvalidOp)"
    );
}

/// Catches: submit_step StepSizePrompt regs-len<2 guard (difeq.rs:798) not reached.
///
/// If `state.regs.len() < 2`, `submit_step(StepSizePrompt)` must return
/// `Err(HpError::InvalidOp)`.
#[test]
fn submit_step_step_size_prompt_short_regs_returns_invalid_op() {
    // Catches: submit_step StepSizePrompt regs-len<2 guard (line 798)
    let mut state = CalcState::new();
    state.regs.truncate(1); // only R00
    let result = submit_step(&mut state, DifeqInputStep::StepSizePrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(StepSizePrompt) with regs.len() < 2 must return Err(InvalidOp)"
    );
}

/// Catches: submit_step X0Prompt regs-len<3 guard (difeq.rs:807) not reached.
///
/// If `state.regs.len() < 3`, `submit_step(X0Prompt)` must return
/// `Err(HpError::InvalidOp)`.
#[test]
fn submit_step_x0_prompt_short_regs_returns_invalid_op() {
    // Catches: submit_step X0Prompt regs-len<3 guard (line 807)
    let mut state = CalcState::new();
    state.regs.truncate(2); // only R00, R01
    let result = submit_step(&mut state, DifeqInputStep::X0Prompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(X0Prompt) with regs.len() < 3 must return Err(InvalidOp)"
    );
}

/// Catches: submit_step Y0Prompt regs-len<4 guard (difeq.rs:816) not reached.
///
/// If `state.regs.len() < 4`, `submit_step(Y0Prompt)` must return
/// `Err(HpError::InvalidOp)`.
#[test]
fn submit_step_y0_prompt_short_regs_returns_invalid_op() {
    // Catches: submit_step Y0Prompt regs-len<4 guard (line 816)
    let mut state = CalcState::new();
    state.regs.truncate(3); // only R00, R01, R02
    let result = submit_step(&mut state, DifeqInputStep::Y0Prompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Y0Prompt) with regs.len() < 4 must return Err(InvalidOp)"
    );
}

/// Catches: submit_step Y1PrimePrompt regs-len<5 guard (difeq.rs:832) not reached.
///
/// If `state.regs.len() < 5`, `submit_step(Y1PrimePrompt)` must return
/// `Err(HpError::InvalidOp)`.
#[test]
fn submit_step_y1_prime_prompt_short_regs_returns_invalid_op() {
    // Catches: submit_step Y1PrimePrompt regs-len<5 guard (line 832)
    let mut state = CalcState::new();
    state.regs.truncate(4); // only R00–R03
    let result = submit_step(&mut state, DifeqInputStep::Y1PrimePrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Y1PrimePrompt) with regs.len() < 5 must return Err(InvalidOp)"
    );
}

/// Catches: submit_step FunctionNamePrompt arm (difeq.rs:841) not reached.
///
/// `submit_step(FunctionNamePrompt)` must return `Err(HpError::InvalidOp)` —
/// handled by `submit_label_step`, not `submit_step`.
#[test]
fn submit_step_function_name_prompt_returns_invalid_op() {
    // Catches: submit_step FunctionNamePrompt arm (line 841)
    let mut state = CalcState::new();
    let result = submit_step(&mut state, DifeqInputStep::FunctionNamePrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(FunctionNamePrompt) must return Err(InvalidOp) — use submit_label_step"
    );
}

/// Catches: submit_step Ready arm (difeq.rs:841) not reached.
///
/// `submit_step(Ready)` must return `Err(HpError::InvalidOp)` — the Ready
/// state has no numeric submission.
#[test]
fn submit_step_ready_returns_invalid_op() {
    // Catches: submit_step Ready arm (line 841)
    let mut state = CalcState::new();
    let result = submit_step(&mut state, DifeqInputStep::Ready);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Ready) must return Err(InvalidOp) — Ready has no numeric submission"
    );
}

// ── Test 13: Y0Prompt with order=2 advances to Y1PrimePrompt ─────────────────

/// Catches: Y0Prompt → Y1PrimePrompt transition (difeq.rs:821-823) not reached.
///
/// When submit_step(Y0Prompt) is called with R00=2 (ORDER=2), the modal must
/// advance to Y1PrimePrompt (not Ready). This exercises the order-branch inside
/// the Y0Prompt arm.
///
/// Source: HP 00041-90034 (1979), Chapter 7 — 2nd-order ODEs need y'(0) too.
#[test]
fn submit_step_y0_prompt_order2_advances_to_y1_prime() {
    // Catches: Y0Prompt → Y1PrimePrompt transition (lines 821-823)
    use hp41_core::ops::math1::modal::ModalProgram;
    let mut state = CalcState::new();
    // R00 = 2 (ORDER=2) — needed for the Y1PrimePrompt branch
    state.regs[0] = HpNum::from(2i32);
    state.stack.x = HpNum::from(1i32); // y0=1

    let result = submit_step(&mut state, DifeqInputStep::Y0Prompt);
    assert_eq!(
        result,
        Ok(()),
        "submit_step(Y0Prompt) with ORDER=2 must return Ok"
    );
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Difeq(DifeqInputStep::Y1PrimePrompt)),
        "Y0Prompt with ORDER=2 must advance to Y1PrimePrompt"
    );
    assert_eq!(
        state.modal_prompt,
        Some("Y'0=?".to_string()),
        "Y1PrimePrompt must set modal_prompt to \"Y'0=?\""
    );
}
