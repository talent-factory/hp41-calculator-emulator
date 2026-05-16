// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration tests for MATRIX workflow (Plan 28-06).
//!
//! Tests the full end-to-end dispatch path:
//! `Op::Xeq("MATRIX")` → xrom_resolve → op_matrix_workflow → modal state.
//!
//! These tests verify that the resolver chain (builtin_card_op → xrom_resolve)
//! correctly routes MATRIX mnemonic to the Math Pac I implementation.
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

/// Integration-test helper: set up an n×n matrix in state.
/// Equivalent to the unit-test `setup_matrix` helper in matrix.rs.
fn setup_matrix(state: &mut CalcState, n: u8, elements: &[f64]) {
    assert_eq!(elements.len(), (n as usize) * (n as usize));
    state.matrix_dim = Some((n, n));
    state.matrix_active_reg = Some(15);
    state.regs[14] = HpNum::from(n as i32);
    // Ensure regs is large enough
    let required = 15 + (n as usize) * (n as usize) + n as usize + 1;
    if state.regs.len() < required {
        state.regs.resize(required, HpNum::zero());
    }
    // Store column-major (input is row-major)
    for c in 0..(n as usize) {
        for r in 0..(n as usize) {
            let idx = 15 + c * n as usize + r;
            let v = elements[r * n as usize + c];
            let d = Decimal::from_f64(v).expect("finite f64");
            state.regs[idx] = HpNum::rounded(d);
        }
    }
}

// ── matrix_full_flow_dispatch ─────────────────────────────────────────────────

// Catches: xrom_resolve("MATRIX") not routing to op_matrix_workflow
#[test]
fn matrix_workflow_dispatches_via_xeq() {
    let mut state = CalcState::new();
    // XEQ "MATRIX" — must reach op_matrix_workflow via xrom_resolve
    dispatch(&mut state, Op::Xeq("MATRIX".to_string())).unwrap();
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt))
        ),
        "XEQ MATRIX must open Matrix(OrderPrompt) modal"
    );
    assert_eq!(
        state.modal_prompt,
        Some("ORDER=?".to_string()),
        "XEQ MATRIX must set modal_prompt = 'ORDER=?'"
    );
}

// Catches: xrom_resolve("DET") not routing to op_mat_det
#[test]
fn matrix_det_dispatches_via_xeq() {
    let mut state = CalcState::new();
    // 2×2 identity matrix (det=1)
    setup_matrix(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut state, Op::Xeq("DET".to_string())).unwrap();
    let det = state.stack.x.inner().to_f64().unwrap();
    assert!(
        (det - 1.0).abs() < 1e-7,
        "XEQ DET on identity matrix must return 1.0, got {det}"
    );
}

// Catches: xrom_resolve("INV") not routing to op_mat_inv (MAT-06)
#[test]
fn matrix_inv_dispatches_via_xeq() {
    let mut state = CalcState::new();
    // 2×2 diagonal [[2,0],[0,2]]; inverse = [[0.5,0],[0,0.5]]
    setup_matrix(&mut state, 2, &[2.0, 0.0, 0.0, 2.0]);
    dispatch(&mut state, Op::Xeq("INV".to_string())).unwrap();
    assert!(
        state.modal_prompt.is_none(),
        "XEQ INV on well-conditioned matrix must not set NO SOLUTION"
    );
}

// Catches: xrom_resolve("SIZE") not routing to op_mat_size
#[test]
fn matrix_size_dispatches_via_xeq() {
    let mut state = CalcState::new();
    // Set R14 = 3 (order register)
    state.regs[14] = hp41_core::num::HpNum::from(3i32);
    dispatch(&mut state, Op::Xeq("SIZE".to_string())).unwrap();
    assert_eq!(
        state.stack.x,
        hp41_core::num::HpNum::from(3i32),
        "XEQ SIZE must push R14 (=3) to X"
    );
}

// Catches: xrom_resolve("SIMEQ") not routing to op_mat_simeq; flag 5 not set
#[test]
fn matrix_simeq_flag5() {
    let mut state = CalcState::new();
    // System: [[1,0],[0,1]] · [x,y] = [5,7] → x=5, y=7 (trivial identity case)
    setup_matrix(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    // B vector: base=15, n=2, n*n=4, b_base=19
    state.regs[19] = hp41_core::num::HpNum::from(5i32);
    state.regs[20] = hp41_core::num::HpNum::from(7i32);
    dispatch(&mut state, Op::Xeq("SIMEQ".to_string())).unwrap();
    // Flag 5 must be set after successful SIMEQ
    assert!(
        hp41_core::ops::flags::flag_get(state.flags, 5),
        "XEQ SIMEQ must set flag 5 after successful solution (MAT-11)"
    );
}

// ── matrix_inv_singular_no_solution ──────────────────────────────────────────

// Catches: op_mat_inv not converting singular error to modal_prompt NO SOLUTION
#[test]
fn matrix_inv_singular_no_solution() {
    let mut state = CalcState::new();
    // Singular: [[1,2],[2,4]] (det=0)
    setup_matrix(&mut state, 2, &[1.0, 2.0, 2.0, 4.0]);
    dispatch(&mut state, Op::Xeq("INV".to_string())).unwrap();
    assert_eq!(
        state.modal_prompt,
        Some("NO SOLUTION".to_string()),
        "Singular matrix INV must set modal_prompt = Some('NO SOLUTION')"
    );
}

// ── matrix_vmat_column_major_display ─────────────────────────────────────────

// Catches: VMAT not displaying in column-major order
#[test]
fn matrix_vmat_column_major_display() {
    let mut state = CalcState::new();
    // 2×2 matrix: row-major [[1,2],[3,4]]
    // Column-major output order: A1,1=1 / A2,1=3 / A1,2=2 / A2,2=4
    setup_matrix(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);
    dispatch(&mut state, Op::Xeq("VMAT".to_string())).unwrap();
    assert_eq!(state.print_buffer.len(), 4, "VMAT on 2×2 must push 4 lines");
    assert!(state.print_buffer[0].contains("A1,1="), "First line must be A1,1");
    assert!(state.print_buffer[1].contains("A2,1="), "Second line must be A2,1");
    assert!(state.print_buffer[2].contains("A1,2="), "Third line must be A1,2");
    assert!(state.print_buffer[3].contains("A2,2="), "Fourth line must be A2,2");
}

// ── EDIT modal dispatch ───────────────────────────────────────────────────────

// Catches: EDIT not opening ROW↑COL=? modal
#[test]
fn matrix_edit_opens_modal() {
    let mut state = CalcState::new();
    setup_matrix(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut state, Op::Xeq("EDIT".to_string())).unwrap();
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Matrix(MatrixInputStep::EditPrompt))
        ),
        "XEQ EDIT must open Matrix(EditPrompt) modal"
    );
}

// ── VCOL display ─────────────────────────────────────────────────────────────

// Catches: VCOL not displaying B vector
#[test]
fn matrix_vcol_displays_b_vector() {
    let mut state = CalcState::new();
    setup_matrix(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
    // Set B values at b_base = 15 + 4 = 19
    state.regs[19] = hp41_core::num::HpNum::from(42i32);
    state.regs[20] = hp41_core::num::HpNum::from(7i32);
    dispatch(&mut state, Op::Xeq("VCOL".to_string())).unwrap();
    assert_eq!(state.print_buffer.len(), 2, "VCOL on 2×2 must push 2 lines (B1, B2)");
    assert!(state.print_buffer[0].contains("B1="), "First line must be B1");
    assert!(state.print_buffer[1].contains("B2="), "Second line must be B2");
}
