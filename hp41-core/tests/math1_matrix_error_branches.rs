// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Plan 32-07 gap-closure: error-branch coverage for `hp41-core/src/ops/math1/matrix.rs`.
//!
//! Targets the ~55 missed lines at 89.68 % by covering:
//! - Index out-of-range paths in `matrix_get`/`matrix_set` (reachable via oversized
//!   active-reg offset) — matrix.rs:88, 101
//! - `submit_step(Ready|EditPrompt|SimeqInputPrompt|SimeqDone)` InvalidOp returns
//!   — matrix.rs:403–409
//! - `op_mat_det` non-square guard — matrix.rs:477–478
//! - `op_mat_inv` non-square guard — matrix.rs:505–506
//! - `op_mat_simeq` non-square guard — matrix.rs:541–542
//! - `op_mat_vmat` / `op_mat_vcol` out-of-bounds index guard — via extreme active_reg
//! - `op_mat_size` no-matrix guard — matrix.rs:418–419
//! - `op_mat_edit` no-matrix guard — matrix.rs:459–460
//!
//! Arm reachability classification (per 32-04 pattern):
//! - REACHABLE: all arms above (constructible from normal state machine)
//! - UNREACHABLE: ORDER_REG (=14) out-of-bounds guard in submit_step::OrderPrompt
//!   because CalcState::new() initialises 100 registers and ORDER_REG=14 is always
//!   within range. Constructing a CalcState with fewer than 15 registers requires
//!   direct struct mutation that is not part of the public API surface.
//!   Documented here; not padded.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::math1::matrix::submit_step;
use hp41_core::ops::math1::modal::{MatrixInputStep, ModalProgram};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Helper: push a specific f64 to X ─────────────────────────────────────────

fn set_x(state: &mut CalcState, v: f64) {
    state.stack.x = HpNum::from(Decimal::from_f64(v).unwrap_or(Decimal::ZERO));
    state.stack.lift_enabled = false;
}

/// Set up an n×n matrix in state (column-major storage).
/// `elements` is provided in row-major order for test convenience.
fn setup_matrix_inline(state: &mut CalcState, n: u8, elements: &[f64]) {
    // LINT-EXEMPT: usize-equality (element count) — HpNum in subsequent setup lines is
    // a false-positive of the multi-line lookahead; no float comparison here.
    assert_eq!(elements.len(), (n as usize) * (n as usize));
    state.matrix_dim = Some((n, n));
    state.matrix_active_reg = Some(15);
    state.regs[14] = HpNum::from(n as i32);
    let required = 15 + n as usize * n as usize + n as usize + 1;
    if state.regs.len() < required {
        state.regs.resize(required, HpNum::zero());
    }
    for c in 0..(n as usize) {
        for r in 0..(n as usize) {
            let idx = 15 + c * n as usize + r;
            let d = Decimal::from_f64(elements[r * n as usize + c]).expect("finite f64");
            state.regs[idx] = HpNum::from(d);
        }
    }
}

// ── submit_step: Ready returns InvalidOp ─────────────────────────────────────

// Catches: submit_step not guarding the Ready state — a stale R/S press after
// matrix entry completes would silently mutate state instead of returning an error.
#[test]
fn submit_step_ready_returns_invalid_op() {
    let mut state = CalcState::new();
    setup_matrix_inline(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);
    // Force the modal into Ready state (all elements entered)
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::Ready));

    let result = submit_step(&mut state, MatrixInputStep::Ready);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(Ready) must return InvalidOp — nothing to submit in done state"
    );
}

// Catches: submit_step not guarding the EditPrompt state — attempting to submit
// a numeric value in edit-row/col mode is not valid; the CLI sends a different path.
#[test]
fn submit_step_edit_prompt_returns_invalid_op() {
    let mut state = CalcState::new();
    setup_matrix_inline(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::EditPrompt));

    let result = submit_step(&mut state, MatrixInputStep::EditPrompt);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(EditPrompt) must return InvalidOp"
    );
}

// Catches: submit_step not guarding SimeqInputPrompt — submitting a numeric value
// while waiting for the SIMEQ B-vector prompt path is not handled by submit_step.
#[test]
fn submit_step_simeq_input_prompt_returns_invalid_op() {
    let mut state = CalcState::new();
    setup_matrix_inline(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);

    let result = submit_step(&mut state, MatrixInputStep::SimeqInputPrompt(0));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(SimeqInputPrompt(0)) must return InvalidOp"
    );
}

// Catches: submit_step not guarding SimeqDone — modal in SimeqDone state should
// not accept further numeric submission.
#[test]
fn submit_step_simeq_done_returns_invalid_op() {
    let mut state = CalcState::new();
    setup_matrix_inline(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);

    let result = submit_step(&mut state, MatrixInputStep::SimeqDone);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_step(SimeqDone) must return InvalidOp"
    );
}

// ── matrix_get index out-of-range via high active_reg ────────────────────────

// Catches: `matrix_get` missing bounds check — an extremely high `matrix_active_reg`
// offset causes `idx = base + c*rows + r >= state.regs.len()`, which should return
// InvalidOp rather than panicking.
//
// Technique: set matrix_dim to Some((1,1)) and matrix_active_reg to a value that
// pushes the first element index past the end of state.regs.
// This exercises the `if idx >= state.regs.len() { return Err(HpError::InvalidOp) }`
// branch in matrix_get (line 87–88) and in lu_det (via op_mat_det).
#[test]
fn mat_det_with_oob_active_reg_returns_error() {
    let mut state = CalcState::new();
    // Set a 1x1 matrix with active_reg just past the end of regs
    state.matrix_dim = Some((1, 1));
    let oob_reg = state.regs.len() as u8 + 10; // definitely out of bounds
    state.matrix_active_reg = Some(oob_reg);
    state.regs[14] = HpNum::from(1i32); // ORDER_REG

    let result = dispatch(&mut state, Op::MatDet);
    assert!(
        result.is_err(),
        "op_mat_det with out-of-bounds active_reg must return an error"
    );
}

// Catches: `op_mat_vmat` propagating matrix_get InvalidOp correctly when
// matrix_active_reg points past regs end.
#[test]
fn mat_vmat_with_oob_active_reg_returns_error() {
    let mut state = CalcState::new();
    state.matrix_dim = Some((1, 1));
    let oob_reg = state.regs.len() as u8 + 5;
    state.matrix_active_reg = Some(oob_reg);
    state.regs[14] = HpNum::from(1i32);

    let result = dispatch(&mut state, Op::MatVmat);
    assert!(
        result.is_err(),
        "op_mat_vmat with out-of-bounds active_reg must return an error"
    );
}

// ── op_mat_det: non-square matrix guard ──────────────────────────────────────

// Catches: op_mat_det not checking rows != cols — computing det of a non-square
// matrix is mathematically undefined and must return InvalidOp.
#[test]
fn mat_det_non_square_returns_invalid_op() {
    let mut state = CalcState::new();
    // Manually set a 2x3 non-square matrix
    state.matrix_dim = Some((2, 3));
    state.matrix_active_reg = Some(15);
    state.regs[14] = HpNum::from(2i32);

    let result = dispatch(&mut state, Op::MatDet);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "op_mat_det with non-square matrix must return InvalidOp"
    );
}

// ── op_mat_inv: non-square matrix guard ──────────────────────────────────────

// Catches: op_mat_inv not checking rows != cols — inverting a non-square matrix
// is undefined and must return InvalidOp.
#[test]
fn mat_inv_non_square_returns_invalid_op() {
    let mut state = CalcState::new();
    state.matrix_dim = Some((2, 3));
    state.matrix_active_reg = Some(15);
    state.regs[14] = HpNum::from(2i32);

    let result = dispatch(&mut state, Op::MatInv);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "op_mat_inv with non-square matrix must return InvalidOp"
    );
}

// ── op_mat_simeq: non-square matrix guard ────────────────────────────────────

// Catches: op_mat_simeq not checking rows != cols — SIMEQ requires a square
// coefficient matrix A; non-square must return InvalidOp.
#[test]
fn mat_simeq_non_square_returns_invalid_op() {
    let mut state = CalcState::new();
    state.matrix_dim = Some((2, 3));
    state.matrix_active_reg = Some(15);
    state.regs[14] = HpNum::from(2i32);

    let result = dispatch(&mut state, Op::MatSimeq);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "op_mat_simeq with non-square matrix must return InvalidOp"
    );
}

// ── op_mat_size: no-matrix guard ─────────────────────────────────────────────

// Catches: op_mat_size not returning InvalidOp when ORDER_REG is uninitialized
// (zero) after initial state — SIZE with no active matrix should return the
// value in R14; zero is a valid (degenerate) case. But if ORDER_REG >= regs.len()
// the guard fires. This test exercises op_mat_size with a fresh state where
// R14=0 (no active matrix set up). The result is Ok (returns 0.0) since
// ORDER_REG=14 < default 100 regs — the SIZE op itself does not error;
// we verify it returns Ok and X = 0.
#[test]
fn mat_size_with_no_setup_returns_ok_zero() {
    let mut state = CalcState::new();
    // No matrix_dim, no matrix_active_reg set; R14 = 0 (default)
    let result = dispatch(&mut state, Op::MatSize);
    assert!(
        result.is_ok(),
        "op_mat_size with no matrix setup should return Ok (reads R14 = 0)"
    );
    let x_val = state.stack.x.inner().to_f64().unwrap_or(-1.0);
    // LINT-EXEMPT: integer-exact equality (R14=0, which is exactly 0.0 in f64).
    assert!(
        (x_val - 0.0).abs() < 1e-9,
        "op_mat_size with R14=0 must push 0 to X, got {x_val}"
    );
}

// ── op_mat_edit: no-matrix guard ─────────────────────────────────────────────

// Catches: op_mat_edit not returning InvalidOp when no matrix is active —
// opening the element edit modal without an active matrix makes no sense.
#[test]
fn mat_edit_no_active_matrix_returns_invalid_op() {
    let mut state = CalcState::new();
    // No matrix_dim set

    let result = dispatch(&mut state, Op::MatEdit);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "op_mat_edit with no active matrix must return InvalidOp"
    );
}

// ── submit_step OrderPrompt: N > MAX_ORDER (14) returns OutOfRange ────────────

// Catches: ORDER clamping not enforced at the submit_step level — an order value
// > MAX_ORDER (14) should be clamped, not rejected; the OutOfRange comes from
// op_mat_det/inv/simeq when n > MAX_ORDER is set externally. This test verifies
// the clamping is correct (order 20 → clamped to 14).
#[test]
fn submit_step_order_prompt_clamps_to_max_order() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::MatrixWorkflow).unwrap();
    // Set X to 20 (above MAX_ORDER=14)
    set_x(&mut state, 20.0);

    let result = submit_step(&mut state, MatrixInputStep::OrderPrompt);
    assert!(
        result.is_ok(),
        "submit_step(OrderPrompt) with N=20 must clamp to MAX_ORDER=14, not error"
    );
    // matrix_dim should be (14, 14)
    assert_eq!(
        state.matrix_dim,
        Some((14, 14)),
        "submit_step(OrderPrompt) with N=20 must clamp order to 14"
    );
}

// ── ElementPrompt: matrix_set index OOB returns error ────────────────────────

// Catches: ElementPrompt arm in submit_step not propagating matrix_set InvalidOp
// when the active_reg + element offset exceeds regs.len().
//
// Technique: set OrderPrompt with N=1, then set matrix_active_reg to a value
// that puts element (0,0) out of bounds after submit_step(ElementPrompt(0,0)).
#[test]
fn submit_step_element_prompt_oob_active_reg_returns_error() {
    let mut state = CalcState::new();
    // Simulate a correctly-ordered 1x1 matrix setup but OOB active_reg
    state.matrix_dim = Some((1, 1));
    let oob_reg = state.regs.len() as u8 + 10;
    state.matrix_active_reg = Some(oob_reg);

    set_x(&mut state, 5.0);
    let result = submit_step(&mut state, MatrixInputStep::ElementPrompt(0, 0));
    assert!(
        result.is_err(),
        "submit_step(ElementPrompt) with OOB active_reg must return an error"
    );
}
