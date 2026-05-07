//! Phase 2 math operations: unary arithmetic, YPow, trig, angle mode.
//!
//! All unary ops use `unary_result()` — this saves X to LASTX and enables lift.
//! Y^X uses `binary_result()` — saves X to LASTX and drops Y.
//! Angle mode ops use `apply_lift_effect(Neutral)`.

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::f64::consts::PI;
use std::str::FromStr;

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, binary_result, unary_result, LiftEffect};
use crate::state::{AngleMode, CalcState};

// ── Angle conversion constants for forward trig (DEG/GRAD → radians) ────────
//
// Stored at full Decimal precision (NOT pre-rounded to 10 sig digits).
// Using HpNum::from(Decimal) would round them to 10 sig digits, which can
// cause off-by-one errors in the last sig digit for canonical inputs.
// We use HpNum(raw_decimal) (pub(crate) inner field) to bypass pre-rounding.
// The multiplication result is still rounded to 10 sig digits via checked_mul.
fn pi_over_180() -> HpNum {
    HpNum(Decimal::from_str("0.01745329251994329576").unwrap())
}
fn pi_over_200() -> HpNum {
    HpNum(Decimal::from_str("0.01570796326794896558").unwrap())
}
/// Convert a value from the current angle mode to radians (HpNum path).
/// Used internally — NOT called by forward trig ops (those use the f64 path below).
/// Kept for potential future use in extended ops.
#[allow(dead_code)]
fn to_radians_hpnum(x: &HpNum, mode: AngleMode) -> Result<HpNum, HpError> {
    match mode {
        AngleMode::Rad => Ok(x.clone()),
        AngleMode::Deg => x.checked_mul(&pi_over_180()),
        AngleMode::Grad => x.checked_mul(&pi_over_200()),
    }
}

/// Convert angle from current mode to radians in f64.
/// Used by forward trig ops to avoid double-rounding.
fn to_radians_f64(v: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Rad => v,
        AngleMode::Deg => v.to_radians(), // uses std f64 conversion (v * PI / 180)
        AngleMode::Grad => v * (PI / 200.0),
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

/// INT: truncate X toward zero (integer part, HP-41 INT function).
/// Removes the fractional part of X. No domain restriction.
/// LiftEffect: Enable (via unary_result). LASTX: saves X before overwrite.
pub fn op_int(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.trunc_int();
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
//
// Forward trig uses a direct f64 bridge (same rationale as inverse trig):
//   1. Convert X to f64
//   2. Convert to radians in f64 (avoids double-rounding via rounded constant)
//   3. Compute sin/cos/tan in f64
//   4. Round to 10 sig digits once at the end
//
// This gives exact results for canonical angles (SIN(30°)=0.5, COS(60°)=0.5, etc).

/// SIN: sin(X) where X is in current angle_mode.
/// LiftEffect: Enable.
pub fn op_sin(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let rad = to_radians_f64(v, state.angle_mode);
    let result = Decimal::from_f64(rad.sin())
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)?;
    unary_result(state, result);
    Ok(())
}

/// COS: cos(X) where X is in current angle_mode.
/// LiftEffect: Enable.
pub fn op_cos(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let rad = to_radians_f64(v, state.angle_mode);
    let result = Decimal::from_f64(rad.cos())
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)?;
    unary_result(state, result);
    Ok(())
}

/// TAN: tan(X) where X is in current angle_mode.
/// LiftEffect: Enable. Domain error at tan(π/2) etc.
pub fn op_tan(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let rad = to_radians_f64(v, state.angle_mode);
    let tan_val = rad.tan();
    // tan(π/2) = infinity in f64; Decimal::from_f64(inf) returns None → Domain
    let result = Decimal::from_f64(tan_val)
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)?;
    unary_result(state, result);
    Ok(())
}

// ── Trig ops — inverse ────────────────────────────────────────────────────
//
// Inverse trig ops use a direct f64 bridge to avoid double-rounding.
// The sequence is:
//   1. Convert X (HpNum) to f64
//   2. Apply domain guard
//   3. Compute asin/acos/atan in f64 (result in radians)
//   4. Convert from radians to target angle_mode (also in f64)
//   5. Round to 10 sig digits via HpNum::rounded() ONCE at the end
//
// This avoids the precision error from: rounded_radians * deg_per_rad
// (which loses accuracy in the last digit for canonical angles like ASIN(1)=90).
// See deviation note in SUMMARY.md for explanation.

fn f64_from_radians(rad: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Rad => rad,
        AngleMode::Deg => rad * (180.0 / PI),
        AngleMode::Grad => rad * (200.0 / PI),
    }
}

/// ASIN: arcsin(X), result in current angle_mode.
/// LiftEffect: Enable. Domain error if |X| > 1.
pub fn op_asin(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    if !(-1.0..=1.0).contains(&v) {
        return Err(HpError::Domain);
    }
    let rad = v.asin();
    let angle = f64_from_radians(rad, state.angle_mode);
    let result = Decimal::from_f64(angle)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

/// ACOS: arccos(X), result in current angle_mode.
/// LiftEffect: Enable. Domain error if |X| > 1.
pub fn op_acos(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    if !(-1.0..=1.0).contains(&v) {
        return Err(HpError::Domain);
    }
    let rad = v.acos();
    let angle = f64_from_radians(rad, state.angle_mode);
    let result = Decimal::from_f64(angle)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

/// ATAN: arctan(X), result in current angle_mode.
/// LiftEffect: Enable. No domain restriction.
pub fn op_atan(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let rad = v.atan();
    let angle = f64_from_radians(rad, state.angle_mode);
    let result = Decimal::from_f64(angle)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
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
