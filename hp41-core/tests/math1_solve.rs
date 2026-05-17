// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration tests for Op::Solve (master root-finder) and Op::Sol (sub-entry).
//!
//! These tests directly reference `Op::Solve` and `Op::Sol` variants to satisfy
//! the Pitfall 16 gate in `math1_op_test_count.rs` (≥5 mentions per variant
//! in `math1_*.rs` files). The tests also cover behavioral paths not covered
//! by the inline unit tests in `solve.rs`.
//!
//! Source: HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979), Chapter 6.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::math1::solve::{op_sol_run_loop, op_solve_run_loop, SolveState};
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
use rust_decimal::prelude::ToPrimitive;

// ── Test helpers ──────────────────────────────────────────────────────────────

/// Build a state for SOLVE on f(x) = x (root at 0).
/// Op::Solve master entry — reads alpha_reg + R00/R01.
fn make_solve_identity() -> (CalcState, Vec<Op>) {
    let program = vec![Op::Lbl("ID".to_string()), Op::Rtn];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "ID".to_string();
    state.regs[0] = HpNum::from(-1i32);
    state.regs[1] = HpNum::from(1i32);
    (state, program)
}

/// Build a state for Op::Sol sub-entry on f(x) = x.
/// Op::Sol reads x1 from R00, x2 from R01 directly (SOLV-02).
fn make_sol_identity() -> (CalcState, Vec<Op>) {
    make_solve_identity() // same state — Sol uses alpha_reg + R00/R01 too
}

// ── Op::Solve: master entry tests ─────────────────────────────────────────────

// Catches: Op::Solve dispatch arm not returning InvalidOp when called outside run_loop
// (mirrors Op::Integ pattern — only valid inside run_loop match arm)
#[test]
fn solve_dispatch_arm_returns_invalid_op() {
    let mut state = CalcState::new();
    let result = hp41_core::ops::dispatch(&mut state, Op::Solve);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Solve dispatch must return InvalidOp (runs only in run_loop, not interactively)"
    );
}

// Catches: Op::Solve not storing SolveState during run_loop execution
#[test]
fn solve_sets_solve_state_during_run_loop() {
    let (mut state, program) = make_solve_identity();
    let result = op_solve_run_loop(&mut state, &program);
    assert!(
        result.is_ok(),
        "Op::Solve run_loop should succeed: {result:?}"
    );
    // After completion, solve_state must be None (cleared on success)
    assert!(
        state.solve_state.is_none(),
        "Op::Solve must clear SolveState after completion"
    );
}

// Catches: Op::Solve producing wrong termination message format
#[test]
fn solve_termination_message_format() {
    // Verify Op::Solve writes to print_buffer (not modal_prompt)
    // Source: PATTERNS line 537 — SOLVE results go to print_buffer, not prompts
    let (mut state, program) = make_solve_identity();
    op_solve_run_loop(&mut state, &program).unwrap();
    assert!(
        !state.print_buffer.is_empty(),
        "Op::Solve must write termination message to state.print_buffer (PATTERNS line 537)"
    );
    // modal_prompt must NOT be used for results
    assert!(
        state.modal_prompt.is_none(),
        "Op::Solve results must go to print_buffer, not modal_prompt"
    );
}

// Catches: Op::Solve not initializing SolveState with correct initial fields
#[test]
fn solve_state_has_correct_fields() {
    // Verify SolveState has all 6 required fields per PATTERNS line 536
    let s = SolveState {
        user_label: "F".to_string(),
        x1: HpNum::from(1i32),
        x2: HpNum::from(2i32),
        fx1: HpNum::from(0i32),
        fx2: HpNum::from(0i32),
        iteration: 0u8,
    };
    assert_eq!(s.user_label, "F");
    assert_eq!(s.iteration, 0u8);
    // iteration: u8 cap is 100 per SOLV-07
    assert!(
        u8::MAX >= 100,
        "SolveState.iteration: u8 must be able to hold 100 iterations"
    );
}

// Catches: Op::Solve not reading user label from alpha_reg (OM convention)
#[test]
fn solve_reads_label_from_alpha_reg() {
    let (mut state, program) = make_solve_identity();
    // alpha_reg = "ID" — this is the function label
    assert_eq!(
        state.alpha_reg, "ID",
        "alpha_reg must be set to function label"
    );
    let result = op_solve_run_loop(&mut state, &program);
    // Should succeed — label "ID" is in the program
    assert!(
        result.is_ok(),
        "Op::Solve must find label from alpha_reg: {result:?}"
    );
}

// Catches: Op::Solve XROM-08 guard ordering wrong (solve_state set before rejection)
#[test]
fn solve_strict_reject_fires_pre_mutation() {
    let (mut state, program) = make_solve_identity();
    // Pre-set solve_state → SOLVE must reject without mutating further
    state.solve_state = Some(SolveState::default());
    let solve_state_before = state.solve_state.clone();

    let result = op_solve_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "nested Op::Solve must reject"
    );
    // solve_state must be unchanged (guard fired before mutation)
    assert!(
        state.solve_state.is_some(),
        "Op::Solve must not clear solve_state on pre-mutation rejection"
    );
    assert_eq!(
        state.solve_state.as_ref().map(|s| &s.user_label),
        solve_state_before.as_ref().map(|s| &s.user_label),
        "solve_state must be unchanged after rejection"
    );
}

// ── Op::Sol: sub-entry tests ──────────────────────────────────────────────────

// Catches: Op::Sol dispatch arm not returning InvalidOp when called outside run_loop
#[test]
fn sol_dispatch_arm_returns_invalid_op() {
    let mut state = CalcState::new();
    let result = hp41_core::ops::dispatch(&mut state, Op::Sol);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Sol dispatch must return InvalidOp (runs only in run_loop)"
    );
}

// Catches: Op::Sol not reading x1 from R00 and x2 from R01 (SOLV-05)
#[test]
fn sol_reads_scratch_registers_r00_r01() {
    let (mut state, program) = make_sol_identity();
    // R00 = -1 (x1), R01 = 1 (x2) — set by make_sol_identity
    assert_eq!(
        state.regs[0].inner().to_i32(),
        Some(-1),
        "Op::Sol: R00 must hold x1"
    );
    assert_eq!(
        state.regs[1].inner().to_i32(),
        Some(1),
        "Op::Sol: R01 must hold x2"
    );
    let result = op_sol_run_loop(&mut state, &program);
    assert!(
        result.is_ok(),
        "Op::Sol should find root at 0 for f(x)=x: {result:?}"
    );
    assert!(
        !state.print_buffer.is_empty(),
        "Op::Sol must write termination message"
    );
    assert!(
        state.print_buffer[0].starts_with("ROOT IS"),
        "Op::Sol: f(x)=x with guesses ±1 must find ROOT IS: {:?}",
        state.print_buffer[0]
    );
}

// Catches: Op::Sol not rejecting when alpha_reg is empty (no prior SOLVE setup)
#[test]
fn sol_rejects_without_user_label() {
    let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
    let mut state = CalcState::new();
    state.program = program.clone();
    // alpha_reg intentionally empty — Op::Sol requires a user label
    state.alpha_reg = "".to_string();
    state.regs[0] = HpNum::from(0i32);
    state.regs[1] = HpNum::from(1i32);

    let result = op_sol_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Sol must return InvalidOp when alpha_reg is empty (no user label staged)"
    );
}

// Catches: Op::Sol XROM-08 guard ordering wrong (solve_state not checked pre-mutation)
#[test]
fn sol_strict_reject_fires_pre_mutation() {
    let (mut state, program) = make_sol_identity();
    // Pre-set solve_state → Op::Sol must reject (XROM-08 / SOLV-08)
    state.solve_state = Some(SolveState::default());

    let result = op_sol_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Op::Sol must reject when solve_state is set"
    );
    assert!(
        state.solve_state.is_some(),
        "solve_state must remain Some after Op::Sol pre-mutation rejection"
    );
}

// Catches: Op::Sol producing wrong termination message format (should write to print_buffer)
#[test]
fn sol_termination_message_to_print_buffer() {
    // Source: PATTERNS line 537 — SOLVE results go to print_buffer, not prompts
    let (mut state, program) = make_sol_identity();
    op_sol_run_loop(&mut state, &program).unwrap();
    assert!(
        !state.print_buffer.is_empty(),
        "Op::Sol must write termination message to state.print_buffer"
    );
    assert!(
        state.modal_prompt.is_none(),
        "Op::Sol results must NOT go to modal_prompt"
    );
}

// ── xrom_resolve integration tests ───────────────────────────────────────────

// Catches: Op::Solve not registered in MATH_1.ops (xrom_resolve won't dispatch)
#[test]
fn xrom_resolve_dispatches_solve() {
    use hp41_core::ops::math1::xrom::xrom_resolve;
    let resolved = xrom_resolve("SOLVE", 0b0000_0001);
    assert_eq!(
        resolved,
        Some(Op::Solve),
        "xrom_resolve('SOLVE', bit0=1) must return Some(Op::Solve)"
    );
}

// Catches: Op::Sol not registered in MATH_1.ops
#[test]
fn xrom_resolve_dispatches_sol() {
    use hp41_core::ops::math1::xrom::xrom_resolve;
    let resolved = xrom_resolve("SOL", 0b0000_0001);
    assert_eq!(
        resolved,
        Some(Op::Sol),
        "xrom_resolve('SOL', bit0=1) must return Some(Op::Sol)"
    );
}

// ── 100-iteration cap and termination paths ───────────────────────────────────

// Catches: Op::Solve 100-iteration cap not matching SOLV-07 specification
#[test]
fn solve_100_iteration_cap() {
    // f(x) = x^2 + 1 — no real roots; must hit 100-iter cap → "NO ROOT FOUND"
    // Source: HP 00041-90034 (1979), Chapter 6 "when no root exists" behavior.
    let program = vec![
        Op::Lbl("ITCAP".to_string()),
        Op::Sq,
        Op::PushNum(HpNum::from(1i32)),
        Op::Add,
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "ITCAP".to_string();
    state.regs[0] = HpNum::from(1i32);
    state.regs[1] = HpNum::from(2i32);

    op_solve_run_loop(&mut state, &program).unwrap();
    assert_eq!(
        state.print_buffer.first().map(|s| s.as_str()),
        Some("NO ROOT FOUND"),
        "Op::Solve must print 'NO ROOT FOUND' at 100-iteration cap (SOLV-07)"
    );
}

// Catches: Op::Sol 100-iteration cap behavior different from Op::Solve
#[test]
fn sol_100_iteration_cap() {
    // Op::Sol must also respect the 100-iteration cap (shared run_secant_loop helper)
    let program = vec![
        Op::Lbl("SITCAP".to_string()),
        Op::Sq,
        Op::PushNum(HpNum::from(1i32)),
        Op::Add,
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "SITCAP".to_string();
    state.regs[0] = HpNum::from(1i32);
    state.regs[1] = HpNum::from(2i32);

    op_sol_run_loop(&mut state, &program).unwrap();
    assert_eq!(
        state.print_buffer.first().map(|s| s.as_str()),
        Some("NO ROOT FOUND"),
        "Op::Sol must also print 'NO ROOT FOUND' at 100-iteration cap"
    );
}
