//! Phase 2 math operations: unary arithmetic, YPow, trig, angle mode.
//!
//! All unary ops use `unary_result()` — this saves X to LASTX and enables lift.
//! Y^X uses `binary_result()` — saves X to LASTX and drops Y.
//! Angle mode ops use `apply_lift_effect(Neutral)`.

use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::HpError;
use crate::num::HpNum;
use crate::state::{AngleMode, CalcState};
use crate::stack::{apply_lift_effect, binary_result, unary_result, LiftEffect};

// ── Angle conversion constants ────────────────────────────────────────────
//
// These are computed as high-precision Decimal string literals.
// They are module-private (not pub) — used only by to_radians/from_radians.
fn pi_over_180() -> HpNum {
    HpNum::from(Decimal::from_str("0.01745329251994329576").unwrap())
}
fn deg_per_rad() -> HpNum {
    HpNum::from(Decimal::from_str("57.29577951308232522583").unwrap())
}
fn pi_over_200() -> HpNum {
    HpNum::from(Decimal::from_str("0.01570796326794896558").unwrap())
}
fn grad_per_rad() -> HpNum {
    HpNum::from(Decimal::from_str("63.66197723675813430755").unwrap())
}

/// Convert a value from the current angle mode to radians.
/// Used as INPUT conversion for SIN/COS/TAN (forward trig).
fn to_radians(x: &HpNum, mode: AngleMode) -> Result<HpNum, HpError> {
    match mode {
        AngleMode::Rad => Ok(x.clone()),
        AngleMode::Deg => x.checked_mul(&pi_over_180()),
        AngleMode::Grad => x.checked_mul(&pi_over_200()),
    }
}

/// Convert a value from radians to the current angle mode.
/// Used as OUTPUT conversion for ASIN/ACOS/ATAN (inverse trig).
fn from_radians(x: &HpNum, mode: AngleMode) -> Result<HpNum, HpError> {
    match mode {
        AngleMode::Rad => Ok(x.clone()),
        AngleMode::Deg => x.checked_mul(&deg_per_rad()),
        AngleMode::Grad => x.checked_mul(&grad_per_rad()),
    }
}

// ── Unary arithmetic ops ──────────────────────────────────────────────────

/// 1/x: reciprocal of X.
/// LiftEffect: Enable (via unary_result). LASTX: saves X before overwrite.
pub fn op_recip(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_recip()?;
    unary_result(state, result);
    Ok(())
}

/// √x: square root of X.
/// LiftEffect: Enable. Domain error if X < 0.
pub fn op_sqrt(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_sqrt()?;
    unary_result(state, result);
    Ok(())
}

/// x²: X multiplied by X.
/// LiftEffect: Enable.
pub fn op_sq(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_sq()?;
    unary_result(state, result);
    Ok(())
}

/// LN: natural logarithm of X.
/// LiftEffect: Enable. Domain error if X ≤ 0.
pub fn op_ln(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_ln()?;
    unary_result(state, result);
    Ok(())
}

/// LOG: log base 10 of X.
/// LiftEffect: Enable. Domain error if X ≤ 0.
pub fn op_log(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_log10()?;
    unary_result(state, result);
    Ok(())
}

/// e^x: natural exponential of X.
/// LiftEffect: Enable.
pub fn op_exp(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_exp()?;
    unary_result(state, result);
    Ok(())
}

/// 10^x: base-10 exponential of X.
/// LiftEffect: Enable.
pub fn op_tenpow(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_exp10()?;
    unary_result(state, result);
    Ok(())
}

// ── Binary arithmetic op ──────────────────────────────────────────────────

/// Y^X: Y raised to the power of X.
/// LiftEffect: Enable (via binary_result — drops Y). LASTX: saves X (exponent).
pub fn op_ypow(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_powd(&state.stack.x)?;
    binary_result(state, result);
    Ok(())
}

// ── Trig ops — forward ────────────────────────────────────────────────────

/// SIN: sin(X) where X is in current angle_mode.
/// LiftEffect: Enable. Converts X to radians BEFORE calling checked_sin.
pub fn op_sin(state: &mut CalcState) -> Result<(), HpError> {
    let radians = to_radians(&state.stack.x, state.angle_mode)?;
    let result = radians.checked_sin()?;
    unary_result(state, result);
    Ok(())
}

/// COS: cos(X) where X is in current angle_mode.
/// LiftEffect: Enable.
pub fn op_cos(state: &mut CalcState) -> Result<(), HpError> {
    let radians = to_radians(&state.stack.x, state.angle_mode)?;
    let result = radians.checked_cos()?;
    unary_result(state, result);
    Ok(())
}

/// TAN: tan(X) where X is in current angle_mode.
/// LiftEffect: Enable. Domain error at tan(π/2) etc.
pub fn op_tan(state: &mut CalcState) -> Result<(), HpError> {
    let radians = to_radians(&state.stack.x, state.angle_mode)?;
    let result = radians.checked_tan()?;
    unary_result(state, result);
    Ok(())
}

// ── Trig ops — inverse ────────────────────────────────────────────────────
//
// CRITICAL: checked_asin/acos/atan return radians.
// from_radians() MUST be applied to convert to the current angle_mode.
// This is the opposite direction from forward trig (where to_radians converts INPUT).

/// ASIN: arcsin(X), result in current angle_mode.
/// LiftEffect: Enable. Domain error if |X| > 1.
pub fn op_asin(state: &mut CalcState) -> Result<(), HpError> {
    let result_rad = state.stack.x.checked_asin()?; // returns radians
    let result = from_radians(&result_rad, state.angle_mode)?; // → angle_mode units
    unary_result(state, result);
    Ok(())
}

/// ACOS: arccos(X), result in current angle_mode.
/// LiftEffect: Enable. Domain error if |X| > 1.
pub fn op_acos(state: &mut CalcState) -> Result<(), HpError> {
    let result_rad = state.stack.x.checked_acos()?;
    let result = from_radians(&result_rad, state.angle_mode)?;
    unary_result(state, result);
    Ok(())
}

/// ATAN: arctan(X), result in current angle_mode.
/// LiftEffect: Enable. No domain restriction.
pub fn op_atan(state: &mut CalcState) -> Result<(), HpError> {
    let result_rad = state.stack.x.checked_atan()?;
    let result = from_radians(&result_rad, state.angle_mode)?;
    unary_result(state, result);
    Ok(())
}

// ── Angle mode ops ────────────────────────────────────────────────────────

/// DEG: set angle mode to degrees. LiftEffect: Neutral.
pub fn op_set_deg(state: &mut CalcState) -> Result<(), HpError> {
    state.angle_mode = AngleMode::Deg;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RAD: set angle mode to radians. LiftEffect: Neutral.
pub fn op_set_rad(state: &mut CalcState) -> Result<(), HpError> {
    state.angle_mode = AngleMode::Rad;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// GRAD: set angle mode to gradians. LiftEffect: Neutral.
pub fn op_set_grad(state: &mut CalcState) -> Result<(), HpError> {
    state.angle_mode = AngleMode::Grad;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
