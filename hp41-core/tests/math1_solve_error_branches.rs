// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Error-branch coverage for `ops/math1/solve.rs` (gap-closure plan 32-06).
//!
//! Current coverage at plan start: 85.77 % lines.
//! Targeted branches: op_solve is_running arm (line 137), op_sol unconditional return
//! (line 149), run_secant_loop label-not-found arm (line 337), eval_fn error propagation
//! (line 367), cancellation path (line 394), submit_step FunctionNamePrompt/Ready arms
//! (lines 593-595).
//!
//! All tests are REACHABLE arms per Task 1 reconnaissance. The iteration-cap
//! "NO ROOT FOUND" path (lines 478-482) is already covered by
//! `math1_solve_paths.rs::solve_no_root_found` — not duplicated here.
//!
//! Source: HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979), Chapter 6.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::math1::modal::SolveInputStep;
use hp41_core::ops::math1::solve::{op_sol_run_loop, op_solve_run_loop, submit_step};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a SOLVE-ready CalcState with a trivial user function f(x) = x (root at 0).
/// Mirrors `math1_user_callback.rs::make_identity_fn_state` per gap-closure scope.
/// Update both if the helper changes.
fn make_solve_state(label: &str, x1: f64, x2: f64) -> (CalcState, Vec<Op>) {
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;

    let program = vec![Op::Lbl(label.to_string()), Op::Rtn];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = label.to_string();
    state.regs[0] = HpNum::from(Decimal::from_f64(x1).unwrap_or(Decimal::ZERO));
    state.regs[1] = HpNum::from(Decimal::from_f64(x2).unwrap_or(Decimal::ZERO));
    (state, program)
}

// ── Test 1: op_solve dispatch arm returns InvalidOp when is_running=true ─────

/// Catches: op_solve is_running arm (solve.rs:137) not reached.
///
/// When `Op::Solve` is dispatched while `state.is_running = true`, the dispatch
/// arm must return `Err(HpError::InvalidOp)` — the real implementation is in
/// `op_solve_run_loop`, not the dispatch arm.
///
/// Source: HP 00041-90034 (1979), Chapter 6 — SOLVE must be inside a running program.
#[test]
fn op_solve_dispatch_arm_invalid_op_when_running() {
    // Catches: op_solve is_running branch (line 137) not covered
    let mut state = CalcState::new();
    state.is_running = true; // simulate being inside a running program
    let result = dispatch(&mut state, Op::Solve);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Solve when is_running must return Err(InvalidOp) — run_loop handles it"
    );
}

// ── Test 2: op_sol dispatch arm returns InvalidOp unconditionally ─────────────

/// Catches: op_sol unconditional return path (solve.rs:149) not reached.
///
/// `Op::Sol` dispatched interactively always returns `Err(HpError::InvalidOp)`.
/// It has no interactive modal — it can only execute inside a running program.
///
/// Source: HP 00041-90034 (1979), Chapter 6 — SOL sub-entry is program-only.
#[test]
fn op_sol_dispatch_arm_always_invalid_op() {
    // Catches: op_sol unconditional InvalidOp (line 149) not covered
    let mut state = CalcState::new();
    // Interactive mode (is_running=false) — always InvalidOp
    let result = dispatch(&mut state, Op::Sol);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Sol must always return Err(InvalidOp) when dispatched interactively"
    );
}

/// Catches: op_sol unconditional InvalidOp (line 149) not covered — is_running variant
#[test]
fn op_sol_dispatch_arm_invalid_op_when_running_too() {
    // Catches: op_sol dispatch arm with is_running=true (line 149) — unconditional
    let mut state = CalcState::new();
    state.is_running = true;
    let result = dispatch(&mut state, Op::Sol);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Sol must return Err(InvalidOp) even when is_running (run_loop arm handles it)"
    );
}

// ── Test 3: run_secant_loop — label not found returns InvalidOp ───────────────

/// Catches: run_secant_loop label-not-found arm (solve.rs:337) not reached.
///
/// When alpha_reg names a label that does not exist in the program slice, the
/// secant loop must return `Err(HpError::InvalidOp)` and clear solve_state.
///
/// Source: HP 00041-90034 (1979), Chapter 6 — function label must exist.
#[test]
fn solve_run_loop_label_not_found_returns_invalid_op() {
    // Catches: label-not-found arm in run_secant_loop (line 337)
    let program = vec![Op::Lbl("REAL_LABEL".to_string()), Op::Rtn];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "MISSING_LABEL".to_string(); // label not in program
    state.regs[0] = HpNum::from(-1i32);
    state.regs[1] = HpNum::from(1i32);

    let result = op_solve_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "SOLVE with missing label must return Err(InvalidOp)"
    );
    assert!(
        state.solve_state.is_none(),
        "solve_state must be None after label-not-found failure"
    );
}

// ── Test 4: cancellation propagates through secant loop ──────────────────────

/// Catches: cancel path (solve.rs:394) not reached — Canceled not returned.
///
/// Setting `cancel_requested = true` before the secant loop causes the per-64-iteration
/// check (iter=0 → 0 & 0x3F == 0) to fire immediately and return `Err(HpError::Canceled)`.
///
/// Source: D-28.7 / D-28.8 — cancellation plumbing per ADR-002.
#[test]
fn solve_cancel_propagates() {
    // Catches: cancellation check (line 394) not reached
    use std::sync::atomic::Ordering;
    let (mut state, program) = make_solve_state("FN", -5.0, 5.0);
    state.cancel_requested.store(true, Ordering::Relaxed);

    let result = op_solve_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::Canceled),
        "SOLVE must return Canceled when cancel_requested is set before loop"
    );
    assert!(
        state.solve_state.is_none(),
        "solve_state must be None after cancellation"
    );
}

// ── Test 5: user function error propagates through eval_fn ───────────────────

/// Catches: eval_fn error propagation arm (solve.rs:367) not reached.
///
/// When the user function returns an error (here: Domain error from sqrt(-1)), the
/// secant loop must propagate that error and clear solve_state.
///
/// Source: HP 00041-90034 (1979), Chapter 6 — SOLVE propagates user function errors.
#[test]
fn solve_run_loop_propagates_user_fn_error() {
    // Catches: eval_fn error arm (line 367) — error from user function propagates
    // User function: f(x) = sqrt(x-10) — domain error for x < 10 (our guesses are < 10)
    let program = vec![
        Op::Lbl("SQRT_FN".to_string()),
        Op::PushNum(HpNum::from(10i32)),
        Op::Sub,  // x - 10 (negative for x < 10)
        Op::Sqrt, // sqrt of negative → Domain error
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "SQRT_FN".to_string();
    state.regs[0] = HpNum::from(-2i32); // x1 = -2 (domain error)
    state.regs[1] = HpNum::from(-1i32); // x2 = -1 (domain error)

    let result = op_solve_run_loop(&mut state, &program);
    // The error from sqrt(-ve) should propagate out of the secant loop
    assert!(
        result.is_err(),
        "SOLVE must propagate error from user function, got Ok"
    );
    assert!(
        state.solve_state.is_none(),
        "solve_state must be None after user function error"
    );
}

// ── Test 6: submit_step FunctionNamePrompt returns InvalidOp ─────────────────

/// Catches: submit_step FunctionNamePrompt arm (solve.rs:592-593) not reached.
///
/// `submit_step` with `SolveInputStep::FunctionNamePrompt` must return `Err(HpError::InvalidOp)`
/// because this step is handled by `submit_label_step`, not `submit_step`.
///
/// Source: Phase 29 / CLI-05 — D-29.5 step-transition contract.
#[test]
fn submit_step_function_name_prompt_returns_invalid_op() {
    // Catches: submit_step FunctionNamePrompt arm (lines 592-595)
    let mut state = CalcState::new();
    let result = submit_step(&mut state, SolveInputStep::FunctionNamePrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(FunctionNamePrompt) must return Err(InvalidOp) — handled by submit_label_step"
    );
}

// ── Test 7: submit_step Ready returns InvalidOp ───────────────────────────────

/// Catches: submit_step Ready arm (solve.rs:594) not reached.
///
/// `submit_step` with `SolveInputStep::Ready` must return `Err(HpError::InvalidOp)` —
/// the Ready state has no numeric submission.
///
/// Source: Phase 29 / CLI-05 — D-29.5 step-transition contract.
#[test]
fn submit_step_ready_returns_invalid_op() {
    // Catches: submit_step Ready arm (lines 592-595)
    let mut state = CalcState::new();
    let result = submit_step(&mut state, SolveInputStep::Ready);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Ready) must return Err(InvalidOp) — Ready has no numeric submission"
    );
}

// ── Test 8: submit_step Guess1Prompt with empty regs returns InvalidOp ────────

/// Catches: submit_step Guess1Prompt regs-empty guard (solve.rs:573-574) not reached.
///
/// If `state.regs` is empty, `submit_step(Guess1Prompt)` must return `Err(HpError::InvalidOp)`
/// because there is no R00 to write the first guess into.
///
/// This exercises the `if state.regs.is_empty()` guard on line 573.
#[test]
fn submit_step_guess1_empty_regs_returns_invalid_op() {
    // Catches: submit_step Guess1Prompt regs-empty guard (line 573)
    let mut state = CalcState::new();
    state.regs.clear(); // force empty regs
    let result = submit_step(&mut state, SolveInputStep::Guess1Prompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Guess1Prompt) with empty regs must return Err(InvalidOp)"
    );
}

// ── Test 9: submit_step Guess2Prompt with len<2 regs returns InvalidOp ────────

/// Catches: submit_step Guess2Prompt regs-len<2 guard (solve.rs:583-584) not reached.
///
/// If `state.regs.len() < 2`, `submit_step(Guess2Prompt)` must return `Err(HpError::InvalidOp)`.
#[test]
fn submit_step_guess2_short_regs_returns_invalid_op() {
    // Catches: submit_step Guess2Prompt regs-len<2 guard (line 583)
    let mut state = CalcState::new();
    state.regs.truncate(1); // only R00, no R01
    let result = submit_step(&mut state, SolveInputStep::Guess2Prompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Guess2Prompt) with regs.len() < 2 must return Err(InvalidOp)"
    );
}

// ── Test 10: op_sol_run_loop with empty alpha_reg returns InvalidOp ───────────

/// Catches: op_sol_run_loop empty-alpha_reg guard (solve.rs:271) not reached.
///
/// `op_sol_run_loop` requires `alpha_reg` to be non-empty (it holds the user label).
/// Empty `alpha_reg` means no prior SOLVE setup — returns `Err(HpError::InvalidOp)`.
///
/// Source: SOLV-02 — Op::Sol requires prior function label in ALPHA.
#[test]
fn op_sol_run_loop_empty_alpha_reg_returns_invalid_op() {
    // Catches: op_sol_run_loop empty-alpha_reg guard (line 271)
    let program = vec![Op::Lbl("FN".to_string()), Op::Rtn];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = String::new(); // empty — no prior SOLVE setup

    let result = op_sol_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "op_sol_run_loop with empty alpha_reg must return Err(InvalidOp) per SOLV-02"
    );
}
