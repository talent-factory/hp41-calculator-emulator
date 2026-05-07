//! Phase 6 statistics operations: Σ+, Σ−, MEAN, SDEV, L.R., YHAT, CORR, CLΣSTAT.
//!
//! Σ register layout in state.regs (0-indexed Vec<HpNum>, 100 slots):
//!   R01 = regs[1] = Σx²
//!   R02 = regs[2] = Σx
//!   R03 = regs[3] = n (count)
//!   R04 = regs[4] = Σy²
//!   R05 = regs[5] = Σy
//!   R06 = regs[6] = Σxy
//!
//! IMPORTANT: Σ+/Σ− do NOT call binary_result() — binary_result() drops Y and saves
//! LASTX, but HP-41 Σ ops do neither. Use enter_number() + apply_lift_effect() directly.

use crate::error::HpError;
use crate::num::HpNum;
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};

/// Σ+: accumulate X and Y into Σ registers R01–R06, then push count n into X.
/// Y, Z, T are NOT dropped (unlike binary ops). LASTX is NOT saved. LiftEffect: Enable.
/// D-03: R01=Σx², R02=Σx, R03=n, R04=Σy², R05=Σy, R06=Σxy.
pub fn op_sigma_plus(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();

    // Accumulate — compute each term atomically before writing (Pitfall guard)
    let new_r1 = state.regs[1].checked_add(&x.checked_sq()?)?;    // Σx² += x²
    let new_r2 = state.regs[2].checked_add(&x)?;                    // Σx  += x
    let new_r3 = state.regs[3].checked_add(&HpNum::from(1i32))?;   // n   += 1
    let new_r4 = state.regs[4].checked_add(&y.checked_sq()?)?;    // Σy² += y²
    let new_r5 = state.regs[5].checked_add(&y)?;                    // Σy  += y
    let new_r6 = state.regs[6].checked_add(&x.checked_mul(&y)?)?;  // Σxy += x·y

    // Write all atomically after all computations succeed
    state.regs[1] = new_r1;
    state.regs[2] = new_r2;
    state.regs[3] = new_r3.clone();
    state.regs[4] = new_r4;
    state.regs[5] = new_r5;
    state.regs[6] = new_r6;

    // Push count n into X (HP-41 hardware: Σ+ pushes new n to X)
    // Use op_rcl pattern: force lift_enabled = true before enter_number
    state.stack.lift_enabled = true;
    enter_number(state, new_r3);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// Σ−: remove X and Y from Σ registers R01–R06, then push count n into X.
/// Reverses the effect of the most recently added (X,Y) data point.
/// LiftEffect: Enable.
pub fn op_sigma_minus(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();

    let new_r1 = state.regs[1].checked_sub(&x.checked_sq()?)?;
    let new_r2 = state.regs[2].checked_sub(&x)?;
    let new_r3 = state.regs[3].checked_sub(&HpNum::from(1i32))?;
    let new_r4 = state.regs[4].checked_sub(&y.checked_sq()?)?;
    let new_r5 = state.regs[5].checked_sub(&y)?;
    let new_r6 = state.regs[6].checked_sub(&x.checked_mul(&y)?)?;

    state.regs[1] = new_r1;
    state.regs[2] = new_r2;
    state.regs[3] = new_r3.clone();
    state.regs[4] = new_r4;
    state.regs[5] = new_r5;
    state.regs[6] = new_r6;

    state.stack.lift_enabled = true;
    enter_number(state, new_r3);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// MEAN: push x̄ into X and ȳ into Y from Σ registers.
/// n=0: returns HpError::InvalidOp. LiftEffect: Enable.
pub fn op_mean(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.regs[3].clone();
    if n.is_zero() {
        return Err(HpError::InvalidOp);
    }
    let x_mean = state.regs[2].checked_div(&n)?; // x̄ = Σx / n
    let y_mean = state.regs[5].checked_div(&n)?; // ȳ = Σy / n

    // Push ȳ first (will become Y after next push), then x̄ (lands in X)
    state.stack.lift_enabled = true;
    enter_number(state, y_mean);
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, x_mean);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// SDEV: push sample σx into X and σy into Y (n-1 denominator per HP-41 hardware).
/// n < 2: returns HpError::InvalidOp. Zero denominator from n·Σx²−(Σx)²: propagates as
/// HpError::Domain (from checked_sqrt on negative) or HpError::DivideByZero.
/// LiftEffect: Enable.
pub fn op_sdev(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.regs[3].clone();
    let n_minus_1 = n.checked_sub(&HpNum::from(1i32))?;
    if n_minus_1.is_zero() || n.is_zero() {
        return Err(HpError::InvalidOp); // need at least 2 data points
    }
    let n_times_n_minus_1 = n.checked_mul(&n_minus_1)?;

    // σx = sqrt((n·Σx² − (Σx)²) / (n·(n−1)))
    let sum_x2 = &state.regs[1];
    let sum_x = &state.regs[2];
    let denom_x = n.checked_mul(sum_x2)?.checked_sub(&sum_x.checked_sq()?)?;
    let sx = denom_x.checked_div(&n_times_n_minus_1)?.checked_sqrt()?;

    // σy = sqrt((n·Σy² − (Σy)²) / (n·(n−1)))
    let sum_y2 = &state.regs[4];
    let sum_y = &state.regs[5];
    let denom_y = n.checked_mul(sum_y2)?.checked_sub(&sum_y.checked_sq()?)?;
    let sy = denom_y.checked_div(&n_times_n_minus_1)?.checked_sqrt()?;

    // Push σy first, then σx (σx lands in X)
    state.stack.lift_enabled = true;
    enter_number(state, sy);
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, sx);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// L.R.: linear regression. Pushes slope m into Y and intercept b into X (D-05).
/// n < 2 or all x-values identical (denominator = 0): returns HpError::InvalidOp.
/// LiftEffect: Enable.
pub fn op_lr(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.regs[3].clone();
    if n.is_zero() {
        return Err(HpError::InvalidOp);
    }

    let sum_x = &state.regs[2];
    let sum_y = &state.regs[5];
    let sum_x2 = &state.regs[1];
    let sum_xy = &state.regs[6];

    // denominator = n·Σx² − (Σx)²
    let denom = n.checked_mul(sum_x2)?.checked_sub(&sum_x.checked_sq()?)?;
    if denom.is_zero() {
        return Err(HpError::InvalidOp); // all x values identical
    }

    // numerator = n·Σxy − Σx·Σy
    let numer = n.checked_mul(sum_xy)?.checked_sub(&sum_x.checked_mul(sum_y)?)?;

    // m = numerator / denominator
    let m = numer.checked_div(&denom)?;

    // b = ȳ − m·x̄  where ȳ = Σy/n, x̄ = Σx/n
    let x_mean = sum_x.checked_div(&n)?;
    let y_mean = sum_y.checked_div(&n)?;
    let b = y_mean.checked_sub(&m.checked_mul(&x_mean)?)?;

    // Push m first (goes to Y), then b (lands in X)
    state.stack.lift_enabled = true;
    enter_number(state, m);
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, b);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// YHAT: ŷ prediction. Reads x from X, computes ŷ = m·x + b via L.R. formulas.
/// Returns ŷ in X via unary_result (saves LASTX). LiftEffect: Enable (implicit via unary_result).
/// n=0 or singular denominator: returns HpError::InvalidOp.
pub fn op_yhat(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.regs[3].clone();
    if n.is_zero() {
        return Err(HpError::InvalidOp);
    }

    let x_val = state.stack.x.clone();
    let sum_x = &state.regs[2];
    let sum_y = &state.regs[5];
    let sum_x2 = &state.regs[1];
    let sum_xy = &state.regs[6];

    let denom = n.checked_mul(sum_x2)?.checked_sub(&sum_x.checked_sq()?)?;
    if denom.is_zero() {
        return Err(HpError::InvalidOp);
    }

    let numer = n.checked_mul(sum_xy)?.checked_sub(&sum_x.checked_mul(sum_y)?)?;
    let m = numer.checked_div(&denom)?;
    let x_mean = sum_x.checked_div(&n)?;
    let y_mean = sum_y.checked_div(&n)?;
    let b = y_mean.checked_sub(&m.checked_mul(&x_mean)?)?;

    let y_hat = m.checked_mul(&x_val)?.checked_add(&b)?;
    crate::stack::unary_result(state, y_hat);
    Ok(())
}

/// CORR: correlation coefficient r in X.
/// r = (n·Σxy − Σx·Σy) / sqrt((n·Σx² − (Σx)²) · (n·Σy² − (Σy)²))
/// Zero or negative product under sqrt: HpError::Domain. n=0: HpError::InvalidOp.
/// Returns r via unary_result (saves LASTX). LiftEffect: Enable.
pub fn op_corr(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.regs[3].clone();
    if n.is_zero() {
        return Err(HpError::InvalidOp);
    }

    let sum_x = &state.regs[2];
    let sum_y = &state.regs[5];
    let sum_x2 = &state.regs[1];
    let sum_y2 = &state.regs[4];
    let sum_xy = &state.regs[6];

    let numer = n.checked_mul(sum_xy)?.checked_sub(&sum_x.checked_mul(sum_y)?)?;
    let denom_x = n.checked_mul(sum_x2)?.checked_sub(&sum_x.checked_sq()?)?;
    let denom_y = n.checked_mul(sum_y2)?.checked_sub(&sum_y.checked_sq()?)?;
    // checked_sqrt returns HpError::Domain if argument is negative
    let denom = denom_x.checked_mul(&denom_y)?.checked_sqrt()?;
    if denom.is_zero() {
        return Err(HpError::InvalidOp);
    }

    let r = numer.checked_div(&denom)?;
    crate::stack::unary_result(state, r);
    Ok(())
}

/// CLΣSTAT: zero Σ registers R01–R06. LiftEffect: Neutral.
pub fn op_cl_sigma_stat(state: &mut CalcState) -> Result<(), HpError> {
    for i in 1..=6 {
        state.regs[i] = HpNum::zero();
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
