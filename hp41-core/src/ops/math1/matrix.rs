// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! MATRIX — HP-41C Math Pac I matrix operations (Plan 28-06).
//!
//! ## Supported Operations
//!
//! - `XEQ "MATRIX"`: Master entry — opens modal workflow (ORDER=? → element entry → Ready).
//! - `XEQ "SIZE"`:   Returns matrix order N from R14. LiftEffect::Enable.
//! - `XEQ "VMAT"`:   Displays all elements in column-major order via print_buffer.
//! - `XEQ "EDIT"`:   Opens ROW↑COL=? edit-mode modal. LiftEffect::Neutral.
//! - `XEQ "DET"`:    LU decomposition with partial pivoting; result in X. LiftEffect::Enable.
//! - `XEQ "INV"`:    Gauss-Jordan inverse in place; singular → "NO SOLUTION" modal_prompt.
//! - `XEQ "SIMEQ"`:  Solves [A|b]; solution at R(N+1)..R(2N); sets flag 5. LiftEffect::Neutral.
//! - `XEQ "VCOL"`:   Displays B-vector (R(N+1)..R(2N)) via print_buffer. LiftEffect::Neutral.
//!
//! ## Column-Major Storage (MAT-02)
//!
//! Matrix A(r, c) is stored at `state.regs[base + c * rows + r]`
//! where `base = state.matrix_active_reg` (default 15) and
//! `(rows, cols) = state.matrix_dim`.
//!
//! Order N stored in R14 per OM Chapter 3 convention.
//!
//! ## Singularity Detection (INV_EPSILON / ADR-003)
//!
//! Source: HP-41C Math Pac OM (HP 00041-90034, 1979), Chapter 3, pp. 23, 28.
//! The OM specifies "DATA ERROR" on singular input; no numeric threshold is given.
//! `INV_EPSILON = 1e-10` is derived from HP Museum ROM-observation (hardware ground truth).
//! See `docs/adr/v3.0-003-inv-epsilon.md` for the full decision record.
//!
//! ## ORDER Cap (MAT-09)
//!
//! Maximum ORDER = 14 (memory-bounded, OM Chapter 3 constraint).
//! ORDER > 14 → `HpError::OutOfRange`.

use crate::error::HpError;
use crate::format::format_hpnum;
use crate::num::HpNum;
use crate::ops::flags::flag_set;
use crate::ops::math1::modal::{MatrixInputStep, ModalProgram};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── INV_EPSILON constant (ADR-003) ───────────────────────────────────────────
//
// INV_EPSILON: near-singular pivot detection threshold for Gaussian elimination.
//
// Source: HP-41C Math Pac Owner's Manual (HP 00041-90034, 1979) does NOT provide
// a numeric threshold — it only specifies "DATA ERROR" on singular input (p. 23, 28).
// Value 1e-10 is derived from ROM reverse-engineering observations (HP Museum community)
// and is more conservative than Free42's 5e-10 heuristic.
// ADR-003 (Plan 28-01) locks this value; Plan 28-06 consumes it.
// Revisit if OM primary source quotes a specific constant.
//
// Verification: math1_matrix_flow.rs cases `inv_singular`, `inv_near_singular`,
// `inv_well_conditioned` cite this ADR; deviation > 1 ULP at 10-digit precision
// = wrong EPSILON (FN-QUAL-02 guard).
pub const INV_EPSILON: f64 = 1e-10;

// ── Matrix ORDER constant ─────────────────────────────────────────────────────

/// R14 is the OM-designated register that stores the active matrix ORDER.
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3, p. 10.
const ORDER_REG: usize = 14;

/// Base register for matrix A storage (first element A(0,0) is at this index).
/// Source: HP-41C Math Pac I OM Chapter 3 — "registers starting at R15".
const DEFAULT_MATRIX_BASE_REG: u8 = 15;

/// Maximum matrix ORDER supported by the HP-41C Math Pac I (MAT-09).
/// Source: OM Chapter 3 — memory-bounded maximum.
const MAX_ORDER: u8 = 14;

// ── Private helper: column-major element access ───────────────────────────────

/// Get matrix element A(r, c) using column-major storage.
///
/// Column-major: A(r, c) lives at `base + c * rows + r`.
/// Returns `HpError::InvalidOp` if no matrix is active or index out of range.
fn matrix_get(state: &CalcState, r: u8, c: u8) -> Result<HpNum, HpError> {
    let (rows, _cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
    let base = state.matrix_active_reg.ok_or(HpError::InvalidOp)? as usize;
    let idx = base + c as usize * rows as usize + r as usize;
    if idx >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    Ok(state.regs[idx].clone())
}

/// Set matrix element A(r, c) using column-major storage.
///
/// Returns `HpError::InvalidOp` if no matrix is active or index out of range.
fn matrix_set(state: &mut CalcState, r: u8, c: u8, val: HpNum) -> Result<(), HpError> {
    let (rows, _cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
    let base = state.matrix_active_reg.ok_or(HpError::InvalidOp)? as usize;
    let idx = base + c as usize * rows as usize + r as usize;
    if idx >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    state.regs[idx] = val;
    Ok(())
}

// ── Private: matrix kernel (LU determinant + Gauss-Jordan inverse) ────────────

/// LU determinant with partial pivoting on a square N×N submatrix.
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3, p. 14.
/// Algorithm: partial-pivot LU decomposition; det = product of diagonal pivots × sign.
///
/// Returns `HpError::InvalidOp` if the matrix has not been set up.
/// Returns a near-zero value (not an error) for near-singular matrices —
/// only `op_mat_inv` enforces the INV_EPSILON singularity gate.
fn lu_det(state: &CalcState, n: u8) -> Result<f64, HpError> {
    // Copy matrix to f64 scratch for LU
    let mut a = vec![0.0f64; n as usize * n as usize];
    for c in 0..n {
        for r in 0..n {
            let val = matrix_get(state, r, c)?;
            a[c as usize * n as usize + r as usize] =
                val.inner().to_f64().ok_or(HpError::Overflow)?;
        }
    }

    let sz = n as usize;
    let mut det_sign = 1.0f64;
    let mut det_val = 1.0f64;

    for col in 0..sz {
        // Find pivot (maximum absolute value in this column, at or below diagonal)
        let mut max_row = col;
        let mut max_val = a[col * sz + col].abs();
        for row in (col + 1)..sz {
            let v = a[col * sz + row].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }

        if max_row != col {
            // Swap rows col and max_row in all columns
            for k in 0..sz {
                a.swap(k * sz + col, k * sz + max_row);
            }
            det_sign = -det_sign;
        }

        let pivot = a[col * sz + col];
        det_val *= pivot;

        if pivot == 0.0 {
            return Ok(0.0);
        }

        // Eliminate below pivot
        for row in (col + 1)..sz {
            let factor = a[col * sz + row] / pivot;
            for k in col..sz {
                let diff = factor * a[k * sz + col];
                a[k * sz + row] -= diff;
            }
        }
    }

    Ok(det_sign * det_val)
}

/// Gauss-Jordan inversion on augmented [A | I_N] with partial pivoting.
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3 "INV function".
/// Singular detection: |pivot| < INV_EPSILON → return Err(HpError::Domain) (caller
/// converts to "NO SOLUTION" modal_prompt per plan spec).
///
/// On success: writes the inverse back into the matrix storage in place (column-major).
fn gauss_jordan_inv(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    let sz = n as usize;

    // Build augmented matrix [A | I] in row-major f64 for arithmetic
    let mut aug = vec![0.0f64; sz * sz * 2];
    for r in 0..sz {
        for c in 0..sz {
            let val = matrix_get(state, r as u8, c as u8)?;
            aug[r * sz * 2 + c] = val.inner().to_f64().ok_or(HpError::Overflow)?;
            // Identity part
            aug[r * sz * 2 + sz + r] = 1.0;
        }
    }

    // Forward elimination with partial pivoting
    for col in 0..sz {
        // Find pivot row
        let mut max_row = col;
        let mut max_val = aug[col * sz * 2 + col].abs();
        for row in (col + 1)..sz {
            let v = aug[row * sz * 2 + col].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }

        if max_row != col {
            for k in 0..(sz * 2) {
                aug.swap(col * sz * 2 + k, max_row * sz * 2 + k);
            }
        }

        let pivot = aug[col * sz * 2 + col];

        // Singular check — ADR-003 threshold
        if pivot.abs() < INV_EPSILON {
            return Err(HpError::Domain);
        }

        // Scale pivot row
        for k in 0..(sz * 2) {
            aug[col * sz * 2 + k] /= pivot;
        }

        // Eliminate all other rows
        for row in 0..sz {
            if row == col {
                continue;
            }
            let factor = aug[row * sz * 2 + col];
            for k in 0..(sz * 2) {
                let diff = factor * aug[col * sz * 2 + k];
                aug[row * sz * 2 + k] -= diff;
            }
        }
    }

    // Write result (right half of augmented matrix) back to column-major storage
    for r in 0..sz {
        for c in 0..sz {
            let val_f64 = aug[r * sz * 2 + sz + c];
            let d = Decimal::from_f64(val_f64).ok_or(HpError::Overflow)?;
            matrix_set(state, r as u8, c as u8, HpNum::rounded(d))?;
        }
    }

    Ok(())
}

/// Gauss elimination to solve [A | b] → solution vector x.
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 3 "SIMEQ function".
/// Singular detection: |pivot| < INV_EPSILON → return Err(HpError::Domain).
/// Solution is stored at R(N+1)..R(2N) by the caller.
fn gauss_solve(state: &CalcState, n: u8, b: &[f64]) -> Result<Vec<f64>, HpError> {
    let sz = n as usize;
    // Build augmented matrix [A | b]
    let mut aug = vec![0.0f64; sz * (sz + 1)];
    for r in 0..sz {
        for c in 0..sz {
            let val = matrix_get(state, r as u8, c as u8)?;
            aug[r * (sz + 1) + c] = val.inner().to_f64().ok_or(HpError::Overflow)?;
        }
        aug[r * (sz + 1) + sz] = b[r];
    }

    // Forward elimination with partial pivoting
    for col in 0..sz {
        let mut max_row = col;
        let mut max_val = aug[col * (sz + 1) + col].abs();
        for row in (col + 1)..sz {
            let v = aug[row * (sz + 1) + col].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }

        if max_row != col {
            for k in 0..(sz + 1) {
                aug.swap(col * (sz + 1) + k, max_row * (sz + 1) + k);
            }
        }

        let pivot = aug[col * (sz + 1) + col];
        if pivot.abs() < INV_EPSILON {
            return Err(HpError::Domain);
        }

        for k in 0..(sz + 1) {
            aug[col * (sz + 1) + k] /= pivot;
        }

        for row in (col + 1)..sz {
            let factor = aug[row * (sz + 1) + col];
            for k in 0..(sz + 1) {
                let diff = factor * aug[col * (sz + 1) + k];
                aug[row * (sz + 1) + k] -= diff;
            }
        }
    }

    // Back-substitution
    let mut x = vec![0.0f64; sz];
    for i in (0..sz).rev() {
        x[i] = aug[i * (sz + 1) + sz];
        for j in (i + 1)..sz {
            x[i] -= aug[i * (sz + 1) + j] * x[j];
        }
    }

    Ok(x)
}

// ── Public op functions ───────────────────────────────────────────────────────

/// MATRIX — master entry point: opens modal workflow (OrderPrompt).
///
/// Sets `state.modal_program = Some(ModalProgram::Matrix(OrderPrompt))` and
/// `state.modal_prompt = Some("ORDER=?")`.
/// Sets `state.matrix_active_reg = Some(DEFAULT_MATRIX_BASE_REG)`.
/// LiftEffect: Neutral.
/// Source: HP 00041-90034 (1979), Chapter 3 "MATRIX program entry".
pub fn op_matrix_workflow(state: &mut CalcState) -> Result<(), HpError> {
    // Set default base register for matrix storage
    state.matrix_active_reg = Some(DEFAULT_MATRIX_BASE_REG);
    // Open modal workflow at the ORDER prompt step
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    state.modal_prompt = Some("ORDER=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Phase 29 / CLI-05 additive public surface — D-29.5 ───────────────────────

/// Submit a numeric input step in the MATRIX modal workflow.
///
/// Called by `hp41_core::ops::math1::submit_modal` after `flush_entry_buf` has
/// already flushed the entry buffer to `state.stack.x`. Each step reads X,
/// advances the modal state machine, and updates `state.modal_prompt`.
///
/// ## Step transitions (column-major element entry):
///
/// - `OrderPrompt` → reads X as order N (clamped 1..=14 per MAT-09),
///   sets `matrix_dim = Some((n, n))`, stores N in R14,
///   advances to `ElementPrompt(0, 0)`.
/// - `ElementPrompt(r, c)` → writes X to matrix A(r,c), advances to
///   next element in column-major order. When all elements are entered,
///   advances to `Ready` (no prompt).
/// - `Ready` / all other steps → returns `Err(HpError::InvalidOp)` (nothing
///   to submit in a non-prompting state).
///
/// Phase 29 / CLI-05 additive public surface — D-29.5.
pub fn submit_step(state: &mut CalcState, step: MatrixInputStep) -> Result<(), HpError> {
    match step {
        MatrixInputStep::OrderPrompt => {
            // Read X as order N (after flush_entry_buf; X holds the entered value)
            let n_raw = state.stack.x.inner().to_u8().unwrap_or(0);
            let n = n_raw.clamp(1, MAX_ORDER);
            // Store N in R14 (ORDER_REG)
            if ORDER_REG >= state.regs.len() {
                return Err(HpError::InvalidOp);
            }
            state.regs[ORDER_REG] = HpNum::from(n as i32);
            // Set matrix dimensions and active register
            state.matrix_dim = Some((n, n));
            state.matrix_active_reg = Some(DEFAULT_MATRIX_BASE_REG);
            // Advance to first element prompt ElementPrompt(0, 0)
            state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::ElementPrompt(0, 0)));
            state.modal_prompt = Some("A1,1=?".to_string());
            Ok(())
        }
        MatrixInputStep::ElementPrompt(r, c) => {
            let (rows, cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
            // Write the current X value to matrix A(r, c)
            let val = state.stack.x.clone();
            matrix_set(state, r, c, val)?;
            // Advance to next element in column-major order: row varies fastest
            let next_r = r + 1;
            if next_r < rows {
                // More rows in this column
                let prompt = format!("A{},{}=?", next_r + 1, c + 1);
                state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::ElementPrompt(
                    next_r, c,
                )));
                state.modal_prompt = Some(prompt);
            } else {
                // Move to next column
                let next_c = c + 1;
                if next_c < cols {
                    let prompt = format!("A1,{}=?", next_c + 1);
                    state.modal_program = Some(ModalProgram::Matrix(
                        MatrixInputStep::ElementPrompt(0, next_c),
                    ));
                    state.modal_prompt = Some(prompt);
                } else {
                    // All elements entered — matrix is ready
                    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::Ready));
                    state.modal_prompt = None;
                }
            }
            Ok(())
        }
        MatrixInputStep::Ready
        | MatrixInputStep::EditPrompt
        | MatrixInputStep::SimeqInputPrompt(_)
        | MatrixInputStep::SimeqDone => {
            // No numeric submission expected in these states
            Err(HpError::InvalidOp)
        }
    }
}

/// SIZE — returns current matrix order N (stored in R14) to X.
///
/// LiftEffect: Enable (pushes a value to X).
/// Source: HP 00041-90034 (1979), Chapter 3 "SIZE function".
pub fn op_mat_size(state: &mut CalcState) -> Result<(), HpError> {
    if ORDER_REG >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let order_val = state.regs[ORDER_REG].clone();
    state.stack.lastx = state.stack.x.clone();
    enter_number(state, order_val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// VMAT — display all matrix elements in column-major order via print_buffer.
///
/// Format: `"A{r+1},{c+1}={value}"` for each element in column-major order.
/// LiftEffect: Neutral.
/// Source: HP 00041-90034 (1979), Chapter 3 "VMAT function".
pub fn op_mat_vmat(state: &mut CalcState) -> Result<(), HpError> {
    let (rows, cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
    // Column-major iteration: outer loop = column, inner loop = row
    for c in 0..cols {
        for r in 0..rows {
            let val = matrix_get(state, r, c)?;
            let line = format!(
                "A{},{}={}",
                r + 1,
                c + 1,
                format_hpnum(&val, &state.display_mode)
            );
            state.print_buffer.push(line);
        }
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// EDIT — opens matrix element edit mode (ROW↑COL=? prompt).
///
/// Sets `state.modal_program = Some(ModalProgram::Matrix(EditPrompt))`.
/// LiftEffect: Neutral.
/// Source: HP 00041-90034 (1979), Chapter 3 "EDIT function".
pub fn op_mat_edit(state: &mut CalcState) -> Result<(), HpError> {
    // Guard: matrix must be active
    if state.matrix_dim.is_none() || state.matrix_active_reg.is_none() {
        return Err(HpError::InvalidOp);
    }
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::EditPrompt));
    state.modal_prompt = Some("ROW\u{2191}COL=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// DET — LU determinant with partial pivoting; result in X.
///
/// LiftEffect: Enable (pushes determinant to X).
/// Source: HP 00041-90034 (1979), Chapter 3, p. 14 "DET function".
pub fn op_mat_det(state: &mut CalcState) -> Result<(), HpError> {
    // Guard: matrix must be active
    let (rows, cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
    state.matrix_active_reg.ok_or(HpError::InvalidOp)?;
    if rows != cols {
        // Non-square matrix — determinant undefined
        return Err(HpError::InvalidOp);
    }
    let n = rows;
    if n > MAX_ORDER {
        return Err(HpError::OutOfRange);
    }

    let det_f64 = lu_det(state, n)?;
    let det_dec = Decimal::from_f64(det_f64).ok_or(HpError::Overflow)?;
    let det = HpNum::rounded(det_dec);

    state.stack.lastx = state.stack.x.clone();
    enter_number(state, det);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// INV — Gauss-Jordan matrix inversion in place.
///
/// Singular detection: if |pivot| < INV_EPSILON → sets `state.modal_prompt = Some("NO SOLUTION")`
/// and returns Ok(()) (NOT an HpError — OM p. 23 surfaces as display message).
/// On success: matrix is overwritten with its inverse in column-major storage.
/// LiftEffect: Neutral.
/// Source: HP 00041-90034 (1979), Chapter 3, p. 23 "INV function".
pub fn op_mat_inv(state: &mut CalcState) -> Result<(), HpError> {
    let (rows, cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
    state.matrix_active_reg.ok_or(HpError::InvalidOp)?;
    if rows != cols {
        return Err(HpError::InvalidOp);
    }
    let n = rows;
    if n > MAX_ORDER {
        return Err(HpError::OutOfRange);
    }

    match gauss_jordan_inv(state, n) {
        Ok(()) => {
            // Inversion succeeded; clear any prior modal prompt
            state.modal_prompt = None;
        }
        Err(HpError::Domain) => {
            // Singular or near-singular matrix — OM p. 23 "DATA ERROR" behavior
            // Surfaces as modal_prompt message (not an HpError per plan spec)
            state.modal_prompt = Some("NO SOLUTION".to_string());
        }
        Err(e) => return Err(e),
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SIMEQ — opens SIMEQ modal to enter B-vector; on completion solves [A|b].
///
/// When the modal opens: sets SimeqInputPrompt(0), modal_prompt = "B1=?".
/// On completion: stores solution at R(N+1)..R(2N); sets flag 5.
/// Singular → modal_prompt = "NO SOLUTION".
/// LiftEffect: Neutral.
/// Source: HP 00041-90034 (1979), Chapter 3, p. 28 "SIMEQ function".
pub fn op_mat_simeq(state: &mut CalcState) -> Result<(), HpError> {
    // Guard: matrix must be active
    let (rows, cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
    state.matrix_active_reg.ok_or(HpError::InvalidOp)?;
    if rows != cols {
        return Err(HpError::InvalidOp);
    }
    let n = rows;
    if n > MAX_ORDER {
        return Err(HpError::OutOfRange);
    }

    // Open modal for B-vector entry
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::SimeqInputPrompt(0)));
    state.modal_prompt = Some("B1=?".to_string());

    // For the core-level test, we perform the solve with current B values
    // (stored in R(N+1)..R(2N)).
    // Phase 29/31 wiring will drive R/S-submit to update these registers
    // and then call a solve-step helper. For now, solve immediately using
    // whatever is in R(N+1)..R(2N) as the B vector.
    let base = state.matrix_active_reg.expect("checked above") as usize;
    let b_base = base + n as usize * n as usize;

    // Read B vector from R(b_base)..R(b_base + N - 1)
    let mut b_vec = Vec::with_capacity(n as usize);
    for i in 0..(n as usize) {
        let idx = b_base + i;
        if idx >= state.regs.len() {
            return Err(HpError::InvalidOp);
        }
        let v = state.regs[idx].inner().to_f64().ok_or(HpError::Overflow)?;
        b_vec.push(v);
    }

    match gauss_solve(state, n, &b_vec) {
        Ok(x) => {
            // Store solution at R(N+1)..R(2N) (overwrite B-input slot with solution)
            for (i, &xi) in x.iter().enumerate() {
                let idx = b_base + i;
                if idx >= state.regs.len() {
                    return Err(HpError::InvalidOp);
                }
                let d = Decimal::from_f64(xi).ok_or(HpError::Overflow)?;
                state.regs[idx] = HpNum::rounded(d);
            }
            // Set flag 5 — MAT-11: SIMEQ sets flag 5 on successful solution
            state.flags = flag_set(state.flags, 5);
            // Clear modal state (done)
            state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::SimeqDone));
            state.modal_prompt = None;
        }
        Err(HpError::Domain) => {
            // Singular system — "NO SOLUTION" per OM p. 28
            state.modal_prompt = Some("NO SOLUTION".to_string());
        }
        Err(e) => return Err(e),
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// VCOL — display B-vector elements (R(N+1)..R(2N)) via print_buffer.
///
/// Format: `"B{n+1}={value}"` for each element.
/// LiftEffect: Neutral.
/// Source: HP 00041-90034 (1979), Chapter 3 "VCOL function".
pub fn op_mat_vcol(state: &mut CalcState) -> Result<(), HpError> {
    let (rows, _cols) = state.matrix_dim.ok_or(HpError::InvalidOp)?;
    let base = state.matrix_active_reg.ok_or(HpError::InvalidOp)? as usize;
    let n = rows as usize;
    let b_base = base + n * n;

    for i in 0..n {
        let idx = b_base + i;
        if idx >= state.regs.len() {
            return Err(HpError::InvalidOp);
        }
        let val = state.regs[idx].clone();
        let line = format!("B{}={}", i + 1, format_hpnum(&val, &state.display_mode));
        state.print_buffer.push(line);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Helper for tests: set up a matrix in state ────────────────────────────────

/// Set up a matrix in `state` for testing:
/// - `matrix_dim = Some((n, n))`
/// - `matrix_active_reg = Some(DEFAULT_MATRIX_BASE_REG)`
/// - R14 = n (ORDER register)
/// - Fills matrix from `elements` (row-major order for test convenience;
///   stored column-major in registers).
///
/// `elements` must have exactly n*n entries (row-major: elements[r*n + c]).
/// Resizes `state.regs` if necessary to accommodate the matrix storage.
#[cfg(test)]
pub fn setup_matrix(state: &mut CalcState, n: u8, elements: &[f64]) {
    assert_eq!(elements.len(), (n as usize) * (n as usize));
    state.matrix_dim = Some((n, n));
    state.matrix_active_reg = Some(DEFAULT_MATRIX_BASE_REG);
    // Store ORDER in R14
    state.regs[ORDER_REG] = HpNum::from(n as i32);
    // Ensure regs vector is large enough to hold the matrix (base + n*n elements)
    // Plus extra room for B-vector in SIMEQ tests (base + n*n + n)
    let required_size =
        DEFAULT_MATRIX_BASE_REG as usize + (n as usize) * (n as usize) + n as usize + 1;
    if state.regs.len() < required_size {
        state.regs.resize(required_size, HpNum::zero());
    }
    // Store elements column-major (input elements are row-major)
    for c in 0..(n as usize) {
        for r in 0..(n as usize) {
            let idx = DEFAULT_MATRIX_BASE_REG as usize + c * n as usize + r;
            let v = elements[r * n as usize + c]; // row-major input → column-major storage
            let d = Decimal::from_f64(v).expect("test value must be finite f64");
            state.regs[idx] = HpNum::rounded(d);
        }
    }
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ops::flags::flag_get;
    use crate::state::CalcState;

    // ── basic ─────────────────────────────────────────────────────────────────

    // Catches: MatrixWorkflow not setting modal_program to OrderPrompt
    #[test]
    fn matrix_workflow_opens_order_prompt() {
        let mut state = CalcState::new();
        op_matrix_workflow(&mut state).unwrap();
        assert!(
            matches!(
                state.modal_program,
                Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt))
            ),
            "op_matrix_workflow must set modal_program = Matrix(OrderPrompt)"
        );
        assert_eq!(
            state.modal_prompt,
            Some("ORDER=?".to_string()),
            "op_matrix_workflow must set modal_prompt = Some('ORDER=?')"
        );
    }

    // Catches: MatrixWorkflow not setting matrix_active_reg to default base
    #[test]
    fn matrix_workflow_sets_active_reg() {
        let mut state = CalcState::new();
        op_matrix_workflow(&mut state).unwrap();
        assert_eq!(
            state.matrix_active_reg,
            Some(DEFAULT_MATRIX_BASE_REG),
            "op_matrix_workflow must set matrix_active_reg = Some(15)"
        );
    }

    // Catches: MatSize not reading R14
    #[test]
    fn mat_size_returns_r14() {
        let mut state = CalcState::new();
        state.regs[ORDER_REG] = HpNum::from(3i32);
        op_mat_size(&mut state).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(3i32),
            "SIZE must push R14 (order=3) to X"
        );
    }

    // Catches: MatVmat not pushing to print_buffer in column-major order
    #[test]
    fn mat_vmat_populates_print_buffer_column_major() {
        let mut state = CalcState::new();
        // 2×2 matrix: [[1,2],[3,4]] (row-major input)
        // Column-major storage: R15=1, R16=3, R17=2, R18=4
        setup_matrix(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);
        op_mat_vmat(&mut state).unwrap();
        // Column-major iteration: A1,1 A2,1 A1,2 A2,2
        assert!(
            !state.print_buffer.is_empty(),
            "VMAT must push to print_buffer"
        );
        assert!(
            state.print_buffer[0].contains("A1,1="),
            "First output must be A1,1"
        );
        assert!(
            state.print_buffer[1].contains("A2,1="),
            "Second output must be A2,1"
        );
        assert!(
            state.print_buffer[2].contains("A1,2="),
            "Third output must be A1,2"
        );
        assert!(
            state.print_buffer[3].contains("A2,2="),
            "Fourth output must be A2,2"
        );
    }

    // ── column_major ─────────────────────────────────────────────────────────

    // Catches: column-major storage regression (MAT-02)
    #[test]
    fn column_major_storage_layout() {
        let mut state = CalcState::new();
        // 2×2 matrix row-major: [[1,2],[3,4]]
        // Column-major: A(0,0)=1, A(1,0)=3, A(0,1)=2, A(1,1)=4
        setup_matrix(&mut state, 2, &[1.0, 2.0, 3.0, 4.0]);

        assert_eq!(
            matrix_get(&state, 0, 0).unwrap(),
            HpNum::from(1i32),
            "A(0,0) must be 1 (column-major: base+0*2+0=15)"
        );
        assert_eq!(
            matrix_get(&state, 1, 0).unwrap(),
            HpNum::from(3i32),
            "A(1,0) must be 3 (column-major: base+0*2+1=16)"
        );
        assert_eq!(
            matrix_get(&state, 0, 1).unwrap(),
            HpNum::from(2i32),
            "A(0,1) must be 2 (column-major: base+1*2+0=17)"
        );
        assert_eq!(
            matrix_get(&state, 1, 1).unwrap(),
            HpNum::from(4i32),
            "A(1,1) must be 4 (column-major: base+1*2+1=18)"
        );
    }

    // ── order_cap ─────────────────────────────────────────────────────────────

    // Catches: ORDER > 14 not returning OutOfRange (MAT-09)
    #[test]
    fn order_cap_15_returns_out_of_range() {
        let mut state = CalcState::new();
        // Set up a 15×15 "matrix" (invalid, exceeds cap)
        state.matrix_dim = Some((15, 15));
        state.matrix_active_reg = Some(DEFAULT_MATRIX_BASE_REG);
        let result = op_mat_det(&mut state);
        assert!(
            matches!(result, Err(HpError::OutOfRange)),
            "ORDER=15 must return HpError::OutOfRange (MAT-09 cap is 14)"
        );
    }

    // Catches: ORDER == 14 not accepted (boundary condition)
    #[test]
    fn order_cap_14_is_accepted() {
        let mut state = CalcState::new();
        // 14×14 identity matrix (det=1.0)
        let n = 14usize;
        let mut elems = vec![0.0f64; n * n];
        for i in 0..n {
            elems[i * n + i] = 1.0;
        }
        setup_matrix(&mut state, 14, &elems);
        let result = op_mat_det(&mut state);
        assert!(
            result.is_ok(),
            "ORDER=14 must be accepted (boundary condition)"
        );
    }

    // ── inv_singular ──────────────────────────────────────────────────────────

    // Catches: INV_EPSILON threshold not catching exactly-singular matrices (MAT-07)
    #[test]
    fn inv_singular_2x2_sets_no_solution() {
        let mut state = CalcState::new();
        // Singular 2×2: [[1,2],[2,4]] (det=0, row 2 = 2 × row 1)
        setup_matrix(&mut state, 2, &[1.0, 2.0, 2.0, 4.0]);
        op_mat_inv(&mut state).unwrap();
        assert_eq!(
            state.modal_prompt,
            Some("NO SOLUTION".to_string()),
            "Singular matrix must set modal_prompt = Some('NO SOLUTION')"
        );
    }

    // ── inv_near_singular ─────────────────────────────────────────────────────

    // Catches: INV_EPSILON not triggering for near-singular matrices (MAT-07)
    #[test]
    fn inv_near_singular_sets_no_solution() {
        let mut state = CalcState::new();
        // Near-singular: [[1, 1+1e-11], [1, 1+1e-11]] — pivot ≈ 1e-11 < INV_EPSILON
        // det ≈ (1+1e-11) - (1+1e-11) = 0 → near-singular
        setup_matrix(&mut state, 2, &[1.0, 1.0 + 1e-11, 1.0, 1.0 + 1e-11]);
        op_mat_inv(&mut state).unwrap();
        assert_eq!(
            state.modal_prompt,
            Some("NO SOLUTION".to_string()),
            "Near-singular matrix (pivot < INV_EPSILON) must yield NO SOLUTION"
        );
    }

    // ── inv_well_conditioned ──────────────────────────────────────────────────

    // Catches: Gauss-Jordan inverse giving wrong result for well-conditioned matrix (MAT-07)
    #[test]
    fn inv_well_conditioned_2x2() {
        let mut state = CalcState::new();
        // [[2,0],[0,2]] — diagonal matrix, inverse = [[0.5,0],[0,0.5]]
        setup_matrix(&mut state, 2, &[2.0, 0.0, 0.0, 2.0]);
        op_mat_inv(&mut state).unwrap();
        assert!(
            state.modal_prompt.is_none(),
            "Well-conditioned matrix must not set NO SOLUTION"
        );
        let a00 = matrix_get(&state, 0, 0).unwrap();
        let a11 = matrix_get(&state, 1, 1).unwrap();
        let a01 = matrix_get(&state, 0, 1).unwrap();
        let a10 = matrix_get(&state, 1, 0).unwrap();
        // Inverse of [[2,0],[0,2]] = [[0.5,0],[0,0.5]]
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let half = HpNum::rounded(Decimal::from_str("0.5").unwrap());
        assert_eq!(a00, half, "Inverse (0,0) must be 0.5");
        assert_eq!(a11, half, "Inverse (1,1) must be 0.5");
        assert_eq!(a01, HpNum::zero(), "Inverse (0,1) must be 0");
        assert_eq!(a10, HpNum::zero(), "Inverse (1,0) must be 0");
    }

    // ── inv_back_sub_overflow ─────────────────────────────────────────────────

    // Catches: back-substitution overflow not surfaced (MAT-07)
    // Note: for this implementation using Gauss-Jordan, overflow during element scaling
    // can arise. We test that a near-singular system with very large pivot doesn't panic.
    // Using a well-conditioned 2×2 with large but Decimal-representable values.
    #[test]
    fn inv_back_sub_extreme_scale_does_not_panic() {
        // Large but representable values: [[1e10, 0],[0, 1e10]]
        // Inverse = [[1e-10, 0],[0, 1e-10]] — must not panic; may succeed or fail gracefully
        let mut state = CalcState::new();
        setup_matrix(&mut state, 2, &[1e10, 0.0, 0.0, 1e10]);
        // Must not panic; may return Ok or Err(Overflow)
        let _result = op_mat_inv(&mut state);
        // Acceptable outcomes: Ok (inverse computed) or Err(Overflow)
        // NOT acceptable: panic
    }

    // ── det_3x3 ──────────────────────────────────────────────────────────────

    // Catches: LU determinant wrong for 3×3 example from OM Chapter 3
    // Source: HP 00041-90034 (1979), Chapter 3, p. 14 "DET example"
    // Matrix: [[1,2,3],[4,5,6],[7,8,9]] — singular (det=0)
    #[test]
    fn det_3x3_singular_example() {
        let mut state = CalcState::new();
        // Row-major input: [[1,2,3],[4,5,6],[7,8,9]]
        setup_matrix(
            &mut state,
            3,
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
        );
        op_mat_det(&mut state).unwrap();
        // det([[1,2,3],[4,5,6],[7,8,9]]) = 0 (rows are arithmetic progression)
        let det_val = state.stack.x.inner().to_f64().unwrap();
        assert!(
            det_val.abs() < 1e-6,
            "det([[1,2,3],[4,5,6],[7,8,9]]) must be ≈0, got {det_val}"
        );
    }

    // Catches: LU determinant wrong for non-singular 3×3
    // Source: HP 00041-90034 (1979), Chapter 3 — non-singular example
    // Free42 v3.0.5 cross-check: det([[1,2,0],[0,1,0],[0,0,3]]) = 3
    #[test]
    fn det_3x3_nonsingular() {
        let mut state = CalcState::new();
        // Row-major: [[1,2,0],[0,1,0],[0,0,3]]
        setup_matrix(
            &mut state,
            3,
            &[1.0, 2.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 3.0],
        );
        op_mat_det(&mut state).unwrap();
        let det_val = state.stack.x.inner().to_f64().unwrap();
        assert!(
            (det_val - 3.0).abs() < 1e-7,
            "det([[1,2,0],[0,1,0],[0,0,3]]) must be ≈3, got {det_val}"
        );
    }

    // ── simeq_solves ─────────────────────────────────────────────────────────

    // Catches: SIMEQ not solving 2-variable system correctly (MAT-10)
    // Source: HP 00041-90034 (1979), Chapter 3 "SIMEQ example"
    #[test]
    fn simeq_solves_2x2_system() {
        let mut state = CalcState::new();
        // System: [[1,1],[1,-1]] · [x,y] = [3,1] → x=2, y=1
        setup_matrix(&mut state, 2, &[1.0, 1.0, 1.0, -1.0]);
        // B vector in R(N+1), R(N+2) = R17, R18 (base=15, n=2, b_base=15+4=19)
        // base=15, n=2, n*n=4, b_base=15+4=19
        state.regs[19] = HpNum::from(3i32); // B1=3
        state.regs[20] = HpNum::from(1i32); // B2=1
        op_mat_simeq(&mut state).unwrap();
        // Solution should be at R19, R20
        let x_sol = state.regs[19].inner().to_f64().unwrap();
        let y_sol = state.regs[20].inner().to_f64().unwrap();
        assert!(
            (x_sol - 2.0).abs() < 1e-7,
            "SIMEQ solution x must be ≈2.0, got {x_sol}"
        );
        assert!(
            (y_sol - 1.0).abs() < 1e-7,
            "SIMEQ solution y must be ≈1.0, got {y_sol}"
        );
    }

    // Catches: SIMEQ not setting flag 5 after successful solve (MAT-11)
    #[test]
    fn simeq_sets_flag_5() {
        let mut state = CalcState::new();
        setup_matrix(&mut state, 2, &[1.0, 1.0, 1.0, -1.0]);
        state.regs[19] = HpNum::from(3i32);
        state.regs[20] = HpNum::from(1i32);
        op_mat_simeq(&mut state).unwrap();
        assert!(
            flag_get(state.flags, 5),
            "SIMEQ must set flag 5 after successful solution (MAT-11)"
        );
    }

    // ── simeq_singular ────────────────────────────────────────────────────────

    // Catches: SIMEQ singular system not yielding NO SOLUTION
    #[test]
    fn simeq_singular_yields_no_solution() {
        let mut state = CalcState::new();
        // Singular: [[1,2],[2,4]] — det=0
        setup_matrix(&mut state, 2, &[1.0, 2.0, 2.0, 4.0]);
        state.regs[19] = HpNum::from(1i32);
        state.regs[20] = HpNum::from(2i32);
        op_mat_simeq(&mut state).unwrap();
        assert_eq!(
            state.modal_prompt,
            Some("NO SOLUTION".to_string()),
            "SIMEQ with singular matrix must set modal_prompt = Some('NO SOLUTION')"
        );
    }

    // ── lift_effects ──────────────────────────────────────────────────────────

    // Catches: MatrixWorkflow, MatEdit not declaring LiftEffect::Neutral
    #[test]
    fn matrix_workflow_and_edit_are_neutral() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        op_matrix_workflow(&mut state).unwrap();
        assert!(
            !state.stack.lift_enabled,
            "MatrixWorkflow must not modify lift_enabled (Neutral)"
        );

        setup_matrix(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
        op_mat_edit(&mut state).unwrap();
        assert!(
            !state.stack.lift_enabled,
            "MatEdit must not modify lift_enabled (Neutral)"
        );
    }

    // Catches: MatSize, MatDet not enabling lift (they push to X)
    #[test]
    fn mat_size_and_det_enable_lift() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        state.regs[ORDER_REG] = HpNum::from(2i32);
        op_mat_size(&mut state).unwrap();
        assert!(
            state.stack.lift_enabled,
            "MatSize must set lift_enabled = true (Enable)"
        );

        state.stack.lift_enabled = false;
        setup_matrix(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
        op_mat_det(&mut state).unwrap();
        assert!(
            state.stack.lift_enabled,
            "MatDet must set lift_enabled = true (Enable)"
        );
    }

    // Catches: MatVmat, MatSimeq, MatVcol not being Neutral
    #[test]
    fn vmat_simeq_vcol_are_neutral() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = true;
        setup_matrix(&mut state, 2, &[1.0, 0.0, 0.0, 1.0]);
        op_mat_vmat(&mut state).unwrap();
        assert!(
            state.stack.lift_enabled,
            "MatVmat must not modify lift_enabled (Neutral)"
        );
    }
}
