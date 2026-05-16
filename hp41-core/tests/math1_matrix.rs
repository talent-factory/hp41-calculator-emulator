// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration tests for MATRIX Op variants (Plan 28-06).
//!
//! These tests explicitly mention each Op variant name to satisfy the Pitfall 16
//! meta-test gate (≥5 mentions per variant in math1_*.rs files).
//!
//! Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3.

#![allow(clippy::unwrap_used)]

use hp41_core::num::HpNum;
use hp41_core::ops::dispatch;
use hp41_core::ops::math1::modal::{MatrixInputStep, ModalProgram};
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

/// Set up an n×n matrix in state for integration tests.
fn mat_setup(state: &mut CalcState, n: u8, elements: &[f64]) {
    assert_eq!(elements.len(), (n as usize) * (n as usize));
    state.matrix_dim = Some((n, n));
    state.matrix_active_reg = Some(15);
    state.regs[14] = HpNum::from(n as i32);
    let required = 15 + (n as usize) * (n as usize) + n as usize + 1;
    if state.regs.len() < required {
        state.regs.resize(required, HpNum::zero());
    }
    for c in 0..(n as usize) {
        for r in 0..(n as usize) {
            let idx = 15 + c * n as usize + r;
            let v = elements[r * n as usize + c];
            let d = Decimal::from_f64(v).expect("finite f64");
            state.regs[idx] = HpNum::rounded(d);
        }
    }
}

// ── Op::MatrixWorkflow tests (≥5 mentions) ────────────────────────────────────

// Catches: MatrixWorkflow dispatch failing
#[test]
fn matrix_workflow_dispatch_succeeds() {
    let mut state = CalcState::new();
    assert!(dispatch(&mut state, Op::MatrixWorkflow).is_ok());
}

// Catches: MatrixWorkflow not setting modal to OrderPrompt
#[test]
fn matrix_workflow_sets_order_prompt_modal() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::MatrixWorkflow).unwrap();
    assert!(matches!(
        state.modal_program,
        Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt))
    ));
}

// Catches: MatrixWorkflow not setting modal_prompt text
#[test]
fn matrix_workflow_sets_modal_prompt_text() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::MatrixWorkflow).unwrap();
    assert_eq!(state.modal_prompt, Some("ORDER=?".to_string()));
}

// Catches: MatrixWorkflow not setting matrix_active_reg
#[test]
fn matrix_workflow_sets_active_reg() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::MatrixWorkflow).unwrap();
    assert_eq!(state.matrix_active_reg, Some(15));
}

// Catches: MatrixWorkflow lift_enabled (must be Neutral)
#[test]
fn matrix_workflow_is_neutral_lift() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::MatrixWorkflow).unwrap();
    assert!(!state.stack.lift_enabled, "MatrixWorkflow must not modify lift_enabled");
}

// Catches: MatrixWorkflow not writing to print_buffer (must be empty)
#[test]
fn matrix_workflow_does_not_write_print_buffer() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::MatrixWorkflow).unwrap();
    assert!(state.print_buffer.is_empty(), "MatrixWorkflow must not write to print_buffer");
}

// ── Op::MatSize tests (≥5 mentions) ──────────────────────────────────────────

// Catches: MatSize dispatch failing
#[test]
fn mat_size_dispatch_succeeds_with_r14_set() {
    let mut state = CalcState::new();
    state.regs[14] = HpNum::from(3i32);
    assert!(dispatch(&mut state, Op::MatSize).is_ok());
}

// Catches: MatSize returning wrong value
#[test]
fn mat_size_returns_order_from_r14() {
    let mut state = CalcState::new();
    state.regs[14] = HpNum::from(5i32);
    dispatch(&mut state, Op::MatSize).unwrap();
    assert_eq!(state.stack.x, HpNum::from(5i32));
}

// Catches: MatSize LiftEffect not Enable
#[test]
fn mat_size_enables_lift() {
    let mut state = CalcState::new();
    state.regs[14] = HpNum::from(2i32);
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::MatSize).unwrap();
    assert!(state.stack.lift_enabled, "MatSize must set lift_enabled = true");
}

// Catches: MatSize not updating LASTX
#[test]
fn mat_size_updates_lastx() {
    let mut state = CalcState::new();
    state.regs[14] = HpNum::from(4i32);
    state.stack.x = HpNum::from(99i32);
    dispatch(&mut state, Op::MatSize).unwrap();
    assert_eq!(state.stack.lastx, HpNum::from(99i32), "MatSize must save X to LASTX");
}

// Catches: MatSize with R14=0 (edge case)
#[test]
fn mat_size_returns_zero_when_r14_is_zero() {
    let mut state = CalcState::new();
    state.regs[14] = HpNum::zero();
    dispatch(&mut state, Op::MatSize).unwrap();
    assert_eq!(state.stack.x, HpNum::zero(), "MatSize returns R14 verbatim (0)");
}

// ── Op::MatVmat tests (≥5 mentions) ──────────────────────────────────────────

// Catches: MatVmat dispatch failing when matrix active
#[test]
fn mat_vmat_dispatch_succeeds() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);
    assert!(dispatch(&mut state, Op::MatVmat).is_ok());
}

// Catches: MatVmat not writing to print_buffer
#[test]
fn mat_vmat_writes_to_print_buffer() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);
    dispatch(&mut state, Op::MatVmat).unwrap();
    assert!(!state.print_buffer.is_empty(), "MatVmat must write to print_buffer");
}

// Catches: MatVmat wrong number of output lines for n×n
#[test]
fn mat_vmat_produces_n_squared_lines() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 3, &[1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0]);
    dispatch(&mut state, Op::MatVmat).unwrap();
    assert_eq!(state.print_buffer.len(), 9, "MatVmat on 3×3 must produce 9 lines");
}

// Catches: MatVmat Ai,j prefix missing
#[test]
fn mat_vmat_lines_start_with_a_prefix() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut state, Op::MatVmat).unwrap();
    for line in &state.print_buffer {
        assert!(line.starts_with('A'), "MatVmat lines must start with 'A'");
    }
}

// Catches: MatVmat LiftEffect not Neutral
#[test]
fn mat_vmat_is_neutral_lift() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::MatVmat).unwrap();
    assert!(!state.stack.lift_enabled, "MatVmat must not modify lift_enabled");
}

// ── Op::MatEdit tests (≥5 mentions) ──────────────────────────────────────────

// Catches: MatEdit dispatch failing when matrix active
#[test]
fn mat_edit_dispatch_succeeds() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    assert!(dispatch(&mut state, Op::MatEdit).is_ok());
}

// Catches: MatEdit not setting EditPrompt modal
#[test]
fn mat_edit_opens_edit_prompt() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut state, Op::MatEdit).unwrap();
    assert!(matches!(
        state.modal_program,
        Some(ModalProgram::Matrix(MatrixInputStep::EditPrompt))
    ));
}

// Catches: MatEdit modal_prompt text wrong
#[test]
fn mat_edit_sets_row_col_prompt() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut state, Op::MatEdit).unwrap();
    assert_eq!(
        state.modal_prompt,
        Some("ROW\u{2191}COL=?".to_string()),
        "MatEdit must set ROW↑COL=? prompt"
    );
}

// Catches: MatEdit lift_enabled change (must be Neutral)
#[test]
fn mat_edit_is_neutral_lift() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::MatEdit).unwrap();
    assert!(state.stack.lift_enabled, "MatEdit must not modify lift_enabled");
}

// Catches: MatEdit without active matrix returning InvalidOp
#[test]
fn mat_edit_without_matrix_returns_invalid_op() {
    let mut state = CalcState::new();
    let result = dispatch(&mut state, Op::MatEdit);
    assert!(result.is_err(), "MatEdit without active matrix must return error");
}

// ── Op::MatDet tests (≥5 mentions) ────────────────────────────────────────────

// Catches: MatDet dispatch failing
#[test]
fn mat_det_dispatch_succeeds() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    assert!(dispatch(&mut state, Op::MatDet).is_ok());
}

// Catches: MatDet wrong result for identity matrix
#[test]
fn mat_det_identity_2x2_is_one() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut state, Op::MatDet).unwrap();
    let det = state.stack.x.inner().to_f64().unwrap();
    assert!((det - 1.0).abs() < 1e-9, "det(I₂) must be 1.0, got {det}");
}

// Catches: MatDet LiftEffect not Enable
#[test]
fn mat_det_enables_lift() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[2.0, 0.0, 0.0, 2.0]);
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::MatDet).unwrap();
    assert!(state.stack.lift_enabled, "MatDet must set lift_enabled = true");
}

// Catches: MatDet near-zero result for nearly-singular matrix
#[test]
fn mat_det_singular_returns_near_zero() {
    let mut state = CalcState::new();
    // [[1,2],[2,4]] — det=0
    mat_setup(&mut state, 2, &[1.0, 2.0, 2.0, 4.0]);
    dispatch(&mut state, Op::MatDet).unwrap();
    let det = state.stack.x.inner().to_f64().unwrap();
    assert!(det.abs() < 1e-7, "det of singular matrix must be near zero, got {det}");
}

// Catches: MatDet ORDER cap not enforced (MAT-09)
#[test]
fn mat_det_order_15_returns_error() {
    let mut state = CalcState::new();
    state.matrix_dim = Some((15, 15));
    state.matrix_active_reg = Some(15);
    let result = dispatch(&mut state, Op::MatDet);
    assert!(result.is_err(), "MatDet with ORDER=15 must return error (cap is 14)");
}

// ── Op::MatInv tests (≥5 mentions) ────────────────────────────────────────────

// Catches: MatInv dispatch failing
#[test]
fn mat_inv_dispatch_succeeds() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    assert!(dispatch(&mut state, Op::MatInv).is_ok());
}

// Catches: MatInv not inverting identity correctly
#[test]
fn mat_inv_identity_gives_identity() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut state, Op::MatInv).unwrap();
    // inv(I₂) = I₂ — A(0,0) should still be 1
    let a00 = state.regs[15].inner().to_f64().unwrap();
    assert!((a00 - 1.0).abs() < 1e-9, "inv(I₂)(0,0) must be 1.0, got {a00}");
}

// Catches: MatInv singular detection (MAT-07)
#[test]
fn mat_inv_singular_no_solution() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 2.0, 2.0, 4.0]);
    dispatch(&mut state, Op::MatInv).unwrap();
    assert_eq!(state.modal_prompt, Some("NO SOLUTION".to_string()));
}

// Catches: MatInv lift_enabled (must be Neutral)
#[test]
fn mat_inv_is_neutral_lift() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::MatInv).unwrap();
    assert!(!state.stack.lift_enabled, "MatInv must not modify lift_enabled");
}

// Catches: MatInv not returning Ok for well-conditioned matrix
#[test]
fn mat_inv_well_conditioned_no_error() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[3.0, 1.0, 2.0, 4.0]);
    let result = dispatch(&mut state, Op::MatInv);
    assert!(result.is_ok(), "MatInv on well-conditioned matrix must return Ok");
    assert!(state.modal_prompt.is_none(), "No NO SOLUTION for well-conditioned matrix");
}

// ── Op::MatSimeq tests (≥5 mentions) ─────────────────────────────────────────

// Catches: MatSimeq dispatch failing
#[test]
fn mat_simeq_dispatch_succeeds() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.regs[19] = HpNum::from(1i32);
    state.regs[20] = HpNum::from(2i32);
    assert!(dispatch(&mut state, Op::MatSimeq).is_ok());
}

// Catches: MatSimeq not solving correctly
#[test]
fn mat_simeq_solves_identity_system() {
    let mut state = CalcState::new();
    // I·[x,y] = [7,3] → x=7, y=3
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.regs[19] = HpNum::from(7i32);
    state.regs[20] = HpNum::from(3i32);
    dispatch(&mut state, Op::MatSimeq).unwrap();
    let x_sol = state.regs[19].inner().to_f64().unwrap();
    let y_sol = state.regs[20].inner().to_f64().unwrap();
    assert!((x_sol - 7.0).abs() < 1e-9, "MatSimeq: x must be 7.0, got {x_sol}");
    assert!((y_sol - 3.0).abs() < 1e-9, "MatSimeq: y must be 3.0, got {y_sol}");
}

// Catches: MatSimeq not setting flag 5 (MAT-11)
#[test]
fn mat_simeq_sets_flag_5_on_success() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.regs[19] = HpNum::from(1i32);
    state.regs[20] = HpNum::from(1i32);
    dispatch(&mut state, Op::MatSimeq).unwrap();
    assert!(
        hp41_core::ops::flags::flag_get(state.flags, 5),
        "MatSimeq must set flag 5 after success"
    );
}

// Catches: MatSimeq singular not yielding NO SOLUTION
#[test]
fn mat_simeq_singular_no_solution() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 2.0, 2.0, 4.0]);
    state.regs[19] = HpNum::from(1i32);
    state.regs[20] = HpNum::from(2i32);
    dispatch(&mut state, Op::MatSimeq).unwrap();
    assert_eq!(state.modal_prompt, Some("NO SOLUTION".to_string()));
}

// Catches: MatSimeq lift_enabled (must be Neutral)
#[test]
fn mat_simeq_is_neutral_lift() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.regs[19] = HpNum::from(1i32);
    state.regs[20] = HpNum::from(1i32);
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::MatSimeq).unwrap();
    assert!(!state.stack.lift_enabled, "MatSimeq must not modify lift_enabled");
}

// ── Op::MatVcol tests (≥5 mentions) ──────────────────────────────────────────

// Catches: MatVcol dispatch failing
#[test]
fn mat_vcol_dispatch_succeeds() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    assert!(dispatch(&mut state, Op::MatVcol).is_ok());
}

// Catches: MatVcol not writing to print_buffer
#[test]
fn mat_vcol_writes_to_print_buffer() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.regs[19] = HpNum::from(42i32);
    state.regs[20] = HpNum::from(7i32);
    dispatch(&mut state, Op::MatVcol).unwrap();
    assert_eq!(state.print_buffer.len(), 2, "MatVcol on 2×2 must produce 2 lines (B1, B2)");
}

// Catches: MatVcol B prefix missing
#[test]
fn mat_vcol_lines_start_with_b_prefix() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.regs[19] = HpNum::from(1i32);
    state.regs[20] = HpNum::from(2i32);
    dispatch(&mut state, Op::MatVcol).unwrap();
    assert!(state.print_buffer[0].starts_with("B1="), "First line must be B1=...");
    assert!(state.print_buffer[1].starts_with("B2="), "Second line must be B2=...");
}

// Catches: MatVcol lift_enabled (must be Neutral)
#[test]
fn mat_vcol_is_neutral_lift() {
    let mut state = CalcState::new();
    mat_setup(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    state.regs[19] = HpNum::from(1i32);
    state.regs[20] = HpNum::from(2i32);
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::MatVcol).unwrap();
    assert!(!state.stack.lift_enabled, "MatVcol must not modify lift_enabled");
}

// Catches: MatVcol without matrix returning error
#[test]
fn mat_vcol_without_matrix_returns_error() {
    let mut state = CalcState::new();
    let result = dispatch(&mut state, Op::MatVcol);
    assert!(result.is_err(), "MatVcol without active matrix must return error");
}
