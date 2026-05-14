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
use crate::stack::{apply_lift_effect, binary_result, enter_number, unary_result, LiftEffect};
use crate::state::{AngleMode, CalcState};

// ── Angle conversion constants for forward trig (DEG/GRAD → radians) ────────
//
// Stored at full Decimal precision (NOT pre-rounded to 10 sig digits).
// Using HpNum::from(Decimal) would round them to 10 sig digits, which can
// cause off-by-one errors in the last sig digit for canonical inputs.
// We use HpNum(raw_decimal) (pub(crate) inner field) to bypass pre-rounding.
// The multiplication result is still rounded to 10 sig digits via checked_mul.
fn pi_over_180() -> HpNum {
    HpNum(
        Decimal::from_str("0.01745329251994329576")
            .expect("pi/180 angle constant must parse as valid Decimal"),
    )
}
fn pi_over_200() -> HpNum {
    HpNum(
        Decimal::from_str("0.01570796326794896558")
            .expect("pi/200 angle constant must parse as valid Decimal"),
    )
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

/// %CH: ((X − Y) / Y) × 100, leaving Y on the stack.
///
/// HP-41 % family — reads Y as base and X as new value, but does NOT
/// consume Y. Stack effect is unary (LASTX←oldX, X←result, Y/Z/T fixed) —
/// we reuse `unary_result()` even though the math is binary; do NOT
/// refactor to `binary_result` or Y will be silently consumed.
/// LiftEffect: Enable (via unary_result).
pub fn op_pct_change(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_pct_change(&state.stack.x)?;
    unary_result(state, result);
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

// ── Phase 20: Constant push ─────────────────────────────────────────────

/// PI — push the constant π (3.141592654, 10-digit rounded HP-41 hardware value).
///
/// Parses the high-precision literal `"3.141592653589793"` once and routes it
/// through `HpNum::rounded(...)` so the value matches what the HP-41 hardware
/// shows (D-08).
///
/// Stack behavior mirrors `op_lastx`: forces `lift_enabled = true`, calls
/// `enter_number`, then re-applies `LiftEffect::Enable` (D-10). LASTX is NOT
/// updated — PI is a constant push, not arithmetic.
pub fn op_pi(state: &mut CalcState) -> Result<(), HpError> {
    let pi_value =
        HpNum::rounded(Decimal::from_str("3.141592653589793").expect("PI literal must parse"));
    // Force stack-lift so PI always lifts X onto Y regardless of prior op (D-10).
    state.stack.lift_enabled = true;
    enter_number(state, pi_value);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── Phase 20: Polar/rectangular conversions ─────────────────────────────

/// P→R — polar to rectangular (D-11/D-12/D-13).
///
/// Reads Y = magnitude (r), X = angle in current `angle_mode`, then writes
/// Y = r·cos(angle) (x-coord) and X = r·sin(angle) (y-coord). LASTX ←
/// consumed X. Z and T unchanged (direct stack assignment — neither
/// `unary_result` nor `binary_result` fits the binary-out shape).
/// LiftEffect: Enable.
pub fn op_polar_to_rect(state: &mut CalcState) -> Result<(), HpError> {
    let r = state
        .stack
        .y
        .inner()
        .to_f64()
        .expect("HpNum is always within f64 range");
    let theta = state
        .stack
        .x
        .inner()
        .to_f64()
        .expect("HpNum is always within f64 range");
    let rad = to_radians_f64(theta, state.angle_mode);
    let new_y = r * rad.cos();
    let new_x = r * rad.sin();
    let new_y_d = Decimal::from_f64(new_y)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    let new_x_d = Decimal::from_f64(new_x)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    state.stack.lastx = state.stack.x.clone();
    state.stack.y = new_y_d;
    state.stack.x = new_x_d;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// R→P — rectangular to polar (D-11/D-12/D-13).
///
/// Reads Y = x-coord, X = y-coord, then writes Y = √(x²+y²) (magnitude) and
/// X = atan2(y, x) in current `angle_mode`. Magnitude uses `f64::hypot` for
/// improved numerical accuracy. LASTX ← consumed X. Z and T unchanged.
/// LiftEffect: Enable.
pub fn op_rect_to_polar(state: &mut CalcState) -> Result<(), HpError> {
    let yc = state
        .stack
        .y
        .inner()
        .to_f64()
        .expect("HpNum is always within f64 range");
    let xc = state
        .stack
        .x
        .inner()
        .to_f64()
        .expect("HpNum is always within f64 range");
    let r = yc.hypot(xc);
    // Standard atan2 takes (y, x). Y register holds x-coord, X register holds
    // y-coord per FN-MATH-03 — so the f64 atan2 call is (X-reg).atan2(Y-reg).
    let rad = xc.atan2(yc);
    let angle = f64_from_radians(rad, state.angle_mode);
    let r_d = Decimal::from_f64(r)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    let angle_d = Decimal::from_f64(angle)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    state.stack.lastx = state.stack.x.clone();
    state.stack.y = r_d;
    state.stack.x = angle_d;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── Phase 20: Unary math ────────────────────────────────────────────────

/// RND — round X to the current display precision (D-01/D-02/D-03).
///
/// Routes through `crate::format::round_to_display_precision`, the single
/// source of truth shared with `format_hpnum`. LiftEffect: Enable (via
/// `unary_result` — LASTX ← previous X).
pub fn op_rnd(state: &mut CalcState) -> Result<(), HpError> {
    let rounded = crate::format::round_to_display_precision(&state.stack.x, &state.display_mode);
    unary_result(state, rounded);
    Ok(())
}

/// FRC — fractional part of X (D-15, sign-preserving complement of INT).
///
/// `FRC(x) = x − trunc(x)`. Sign matches the input: `FRC(-3.7) = -0.7`.
/// LiftEffect: Enable (via `unary_result`).
pub fn op_frc(state: &mut CalcState) -> Result<(), HpError> {
    let int_part = state.stack.x.trunc_int();
    let frac = state.stack.x.checked_sub(&int_part)?;
    unary_result(state, frac);
    Ok(())
}

/// ABS — absolute value of X (D-16).
///
/// Negative inputs flip sign via `HpNum::negate()`; zero and positive inputs
/// pass through. LiftEffect: Enable (via `unary_result`).
pub fn op_abs(state: &mut CalcState) -> Result<(), HpError> {
    let result = if state.stack.x.inner().is_sign_negative() {
        state.stack.x.negate()
    } else {
        state.stack.x.clone()
    };
    unary_result(state, result);
    Ok(())
}

/// SIGN — sign of X: -1 / 0 / +1 (D-17/D-18).
///
/// Phase 20 always returns numeric. HP-41 hardware's SIGN-on-ALPHA-typed-X
/// divergence (which would return 0 when X holds alpha data) is documented
/// as a known divergence — our model has no alpha-typed X register.
/// LiftEffect: Enable (via `unary_result`).
pub fn op_sign(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner();
    let result = if v.is_zero() {
        HpNum::zero()
    } else if v.is_sign_negative() {
        HpNum::from(-1i32)
    } else {
        HpNum::from(1i32)
    };
    unary_result(state, result);
    Ok(())
}

/// FACT — factorial of integer X (D-04/D-05/D-06/D-07).
///
/// Order of checks is strict:
/// 1. Read X as f64 for the magnitude pre-flight.
/// 2. Hardware-spec OutOfRange (D-06): `X > 69 → OutOfRange`. Must run
///    before the integer check so X = 70.5 reports OutOfRange (matching
///    the spirit of SC-3).
/// 3. Integer check (D-07): non-integer X → `Domain`.
/// 4. Sign check (D-07): negative X → `Domain`.
/// 5. Iterative f64 product (D-04); convert via
///    `Decimal::from_f64(...).map(HpNum::rounded).ok_or(HpError::Overflow)`
///    — practical magnitude wall is `X ≤ 27` (D-05); `28..=69` returns
///    `Overflow` from the conversion side.
///
/// LiftEffect: Enable (via `unary_result`).
pub fn op_fact(state: &mut CalcState) -> Result<(), HpError> {
    let v = state
        .stack
        .x
        .inner()
        .to_f64()
        .expect("HpNum is always within f64 range");
    // Step 2: hardware-spec OutOfRange pre-flight (D-06, SC-3).
    if v > 69.0 {
        return Err(HpError::OutOfRange);
    }
    // Step 3: integer check (D-07).
    let int_x = state.stack.x.trunc_int();
    if state.stack.x != int_x {
        return Err(HpError::Domain);
    }
    // Step 4: negative check (D-07). After the integer check `v` is a finite integer.
    if v < 0.0 {
        return Err(HpError::Domain);
    }
    // Step 5: iterative product (D-04). `n` is bounded by 0..=69 from the checks above.
    let n = v as u64;
    let mut acc: f64 = 1.0;
    for k in 1..=n {
        acc *= k as f64;
    }
    let result = Decimal::from_f64(acc)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

// ── Phase 20: Binary math ───────────────────────────────────────────────

/// MOD — Y mod X with HP-41 trunc-toward-zero convention (D-14, UPDATED 2026-05-13).
///
/// Result = `Y − X · trunc(Y/X)`. **Sign follows Y** (matches HP-41C
/// Owner's Manual + Free42 source). Examples:
/// `7 MOD -3 = 1` (sign of Y); `-7 MOD 3 = -1` (sign of Y).
/// Domain error if X = 0. LiftEffect: Enable (via `binary_result`).
pub fn op_mod(state: &mut CalcState) -> Result<(), HpError> {
    if state.stack.x.is_zero() {
        return Err(HpError::Domain);
    }
    let q = state.stack.y.checked_div(&state.stack.x)?;
    let q_trunc = q.trunc_int();
    let product = state.stack.x.checked_mul(&q_trunc)?;
    let result = state.stack.y.checked_sub(&product)?;
    binary_result(state, result);
    Ok(())
}
