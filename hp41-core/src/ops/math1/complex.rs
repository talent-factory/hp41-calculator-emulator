// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Complex-stack arithmetic operations for Math Pac I (Plan 28-03).
//!
//! ## Complex Stack Overlay Model (D-28.1)
//!
//! The HP-41 complex stack is an OVERLAY on the normal 4-register stack:
//! - ζ = X + iY  (X is the real part, Y is the imaginary part)
//! - τ = Z + iT  (Z is the real part, T is the imaginary part)
//!
//! Zero new HpNum storage fields are added to `CalcState` — only the
//! `complex_mode: bool` flag (which landed in Plan 28-01) is toggled.
//!
//! ## Binary Op Stack Effect
//!
//! Binary complex ops (C+/C-/C×/C÷) consume ζ AND τ (all 4 stack registers),
//! write the result to ζ (X and Y), and apply T-replicate semantics for τ:
//! the new Z and T both receive the old T value (HP-41 hardware T-replicate
//! pattern: when the stack drops, T is "replicated" rather than consumed).
//!
//! ## auto-on Policy (D-28.2)
//!
//! Every binary complex op sets `state.complex_mode = true` BEFORE the
//! computation. Op::Real (CMPLX-18) sets `state.complex_mode = false`.
//!
//! ## complex_atan2 Helper
//!
//! `complex_atan2(im, re)` computes atan2(im, re) via the f64 bridge.
//! The (0,0) case returns HpNum::zero() immediately (Pitfall 6 mitigation).

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use std::f64::consts::PI;

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::{AngleMode, CalcState};

// ── Helper: complex_atan2 (Pitfall 6 gate) ───────────────────────────────────

/// atan2(im, re) for the complex stack — angle in radians.
///
/// First arm: `(im.is_zero() && re.is_zero())` returns `HpNum::zero()` (NOT NaN,
/// NOT DataError) — Pitfall 6 mitigation. All other cases use the f64 bridge.
///
/// Returns `HpNum::zero()` for the catastrophic case where `Decimal::from_f64`
/// cannot represent the result — safe because atan2 output is bounded in [-π, π].
pub(super) fn complex_atan2(im: HpNum, re: HpNum) -> HpNum {
    if im.is_zero() && re.is_zero() {
        // Pitfall 6: (0,0) → 0, not NaN or Domain error
        return HpNum::zero();
    }
    let im_f = im.inner().to_f64().unwrap_or(0.0);
    let re_f = re.inner().to_f64().unwrap_or(0.0);
    let result_f64 = im_f.atan2(re_f);
    // safe: atan2 result is bounded in [-π, π]; Decimal::from_f64 always succeeds for finite results
    Decimal::from_f64(result_f64)
        .map(HpNum::rounded)
        .unwrap_or_else(HpNum::zero)
}

// ── Op implementations ────────────────────────────────────────────────────────

/// C+ — complex addition: ζ' = ζ + τ
///
/// Stack effect: X' = X+Z (real), Y' = Y+T (imag).
/// T-replicate: Z' = old_T, T' = old_T (HP-41 hardware T-replicate pattern).
/// LiftEffect: Enable. Sets complex_mode = true (D-28.2 auto-on).
///
/// CMPLX-02 / HP 00041-90034.
pub fn op_c_plus(state: &mut CalcState) -> Result<(), HpError> {
    // D-28.2: auto-on before computation
    state.complex_mode = true;

    // Snapshot all 4 stack registers before mutation
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();
    let z = state.stack.z.clone();
    let t = state.stack.t.clone();

    // Compute ζ' = ζ + τ
    let new_x = x.checked_add(&z)?; // real:  X + Z
    let new_y = y.checked_add(&t)?; // imag:  Y + T

    // Save LASTX (HP-41 binary ops save X to LASTX)
    state.stack.lastx = x;

    // Write result to ζ (X and Y)
    state.stack.x = new_x;
    state.stack.y = new_y;

    // T-replicate: new Z and T both get old T
    state.stack.z = t.clone();
    state.stack.t = t;

    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// C- — complex subtraction: ζ' = ζ - τ
///
/// Stack effect: X' = X-Z (real), Y' = Y-T (imag).
/// T-replicate: Z' = old_T, T' = old_T.
/// LiftEffect: Enable. Sets complex_mode = true (D-28.2 auto-on).
///
/// CMPLX-03 / HP 00041-90034.
pub fn op_c_minus(state: &mut CalcState) -> Result<(), HpError> {
    // D-28.2: auto-on before computation
    state.complex_mode = true;

    let x = state.stack.x.clone();
    let y = state.stack.y.clone();
    let z = state.stack.z.clone();
    let t = state.stack.t.clone();

    // Compute ζ' = ζ - τ
    let new_x = x.checked_sub(&z)?; // real:  X - Z
    let new_y = y.checked_sub(&t)?; // imag:  Y - T

    state.stack.lastx = x;
    state.stack.x = new_x;
    state.stack.y = new_y;
    state.stack.z = t.clone();
    state.stack.t = t;

    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// C× — complex multiplication: ζ' = ζ · τ
///
/// Formula: (X+iY)(Z+iT) = (XZ - YT) + i(XT + YZ)
/// Uses 4 multiplications + 2 add/sub operations.
///
/// T-replicate: Z' = old_T, T' = old_T.
/// LiftEffect: Enable. Sets complex_mode = true (D-28.2 auto-on).
///
/// CMPLX-04 / HP 00041-90034.
pub fn op_c_times(state: &mut CalcState) -> Result<(), HpError> {
    // D-28.2: auto-on before computation
    state.complex_mode = true;

    let x = state.stack.x.clone();
    let y = state.stack.y.clone();
    let z = state.stack.z.clone();
    let t = state.stack.t.clone();

    // Real part: XZ - YT
    let xz = x.checked_mul(&z)?;
    let yt = y.checked_mul(&t)?;
    let new_x = xz.checked_sub(&yt)?;

    // Imaginary part: XT + YZ
    let xt = x.checked_mul(&t)?;
    let yz = y.checked_mul(&z)?;
    let new_y = xt.checked_add(&yz)?;

    state.stack.lastx = x;
    state.stack.x = new_x;
    state.stack.y = new_y;
    state.stack.z = t.clone();
    state.stack.t = t;

    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// C÷ — complex division: ζ' = ζ / τ
///
/// Formula: (X+iY)/(Z+iT) = ((XZ+YT) + i(YZ-XT)) / (Z² + T²)
///
/// **Zero-divisor check FIRST** (CMPLX-05 / Pitfall 6 mitigation):
/// if Z.is_zero() && T.is_zero() → `HpError::DivideByZero` BEFORE any stack mutation.
///
/// T-replicate: Z' = old_T, T' = old_T.
/// LiftEffect: Enable. Sets complex_mode = true (D-28.2 auto-on).
///
/// CMPLX-05 / HP 00041-90034.
pub fn op_c_div(state: &mut CalcState) -> Result<(), HpError> {
    // Pitfall 6 / CMPLX-05: zero-divisor check BEFORE any state mutation
    if state.stack.z.is_zero() && state.stack.t.is_zero() {
        return Err(HpError::DivideByZero);
    }

    // D-28.2: auto-on only after the guard (mutation comes after the guard)
    state.complex_mode = true;

    let x = state.stack.x.clone();
    let y = state.stack.y.clone();
    let z = state.stack.z.clone();
    let t = state.stack.t.clone();

    // Denominator: Z² + T²
    let z_sq = z.checked_mul(&z)?;
    let t_sq = t.checked_mul(&t)?;
    let denom = z_sq.checked_add(&t_sq)?;
    // denom cannot be zero here — the guard above already checked that

    // Numerator real: XZ + YT
    let xz = x.checked_mul(&z)?;
    let yt = y.checked_mul(&t)?;
    let num_re = xz.checked_add(&yt)?;

    // Numerator imag: YZ - XT
    let yz = y.checked_mul(&z)?;
    let xt = x.checked_mul(&t)?;
    let num_im = yz.checked_sub(&xt)?;

    // Divide
    let new_x = num_re.checked_div(&denom)?;
    let new_y = num_im.checked_div(&denom)?;

    state.stack.lastx = x;
    state.stack.x = new_x;
    state.stack.y = new_y;
    state.stack.z = t.clone();
    state.stack.t = t;

    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// REAL — deactivate complex mode (CMPLX-18 / D-28.3).
///
/// Sets `state.complex_mode = false`. Stack is NOT modified.
/// LiftEffect: Neutral (mode-change only, no stack operation).
///
/// Note: This op is a UX extension — NOT in Math Pac I OM 1979. Documented
/// divergence per D-28.3; recorded in `docs/hp41-math1-divergences.md` (Phase 30/DOC-04).
///
/// CMPLX-18 / D-28.3.
pub fn op_real(state: &mut CalcState) -> Result<(), HpError> {
    state.complex_mode = false;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Angle-mode conversion helper (module-local) ──────────────────────────────

/// Convert a radian value (f64) to the current angle_mode unit.
/// Mirrors `f64_from_radians` from `ops/math.rs` but scoped to math1.
fn f64_from_radians(rad: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Rad => rad,
        AngleMode::Deg => rad * (180.0 / PI),
        AngleMode::Grad => rad * (200.0 / PI),
    }
}

// ── Plan 28-04: Unary complex transcendental functions ────────────────────────

/// MAGZ — complex magnitude: |ζ| = sqrt(X² + Y²).
///
/// Writes magnitude to X; Y is left unchanged (per OM convention: magnitude to X,
/// imaginary part stays in Y for visualization).
/// LiftEffect: Disable. Sets complex_mode = true (D-28.2 auto-on). OM p.~25.
///
/// // rust_decimal has no sqrt on the complex magnitude directly; use f64 bridge
/// CMPLX-06 / HP 00041-90034 ~p.25.
pub fn op_magz(state: &mut CalcState) -> Result<(), HpError> {
    state.complex_mode = true;

    let x_f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y_f = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;

    // rust_decimal has no direct sqrt for complex magnitude; use f64 bridge
    let mag_f = (x_f * x_f + y_f * y_f).sqrt();

    let mag = Decimal::from_f64(mag_f)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    // Magnitude goes to X; Y stays unchanged (OM convention)
    state.stack.x = mag;
    // Y (imaginary part) is left as-is

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

/// CINV — complex inverse: 1/(X+iY) = (X-iY)/(X²+Y²).
///
/// Guard: (X=0 AND Y=0) → HpError::DivideByZero (symmetric with op_c_div's guard).
/// Guard fires BEFORE any state mutation (Pitfall 6).
/// LiftEffect: Disable. Sets complex_mode = true (D-28.2 auto-on). OM p.~25.
///
/// CMPLX-07 / HP 00041-90034 ~p.25.
pub fn op_cinv(state: &mut CalcState) -> Result<(), HpError> {
    // Pitfall 6: zero-divisor guard BEFORE any mutation
    if state.stack.x.is_zero() && state.stack.y.is_zero() {
        return Err(HpError::DivideByZero);
    }

    state.complex_mode = true;

    let x = state.stack.x.clone();
    let y = state.stack.y.clone();

    // Denominator: X² + Y²
    let x_sq = x.checked_mul(&x)?;
    let y_sq = y.checked_mul(&y)?;
    let denom = x_sq.checked_add(&y_sq)?;
    // denom is guaranteed non-zero (guard above confirmed X≠0 or Y≠0)

    // Result: (X - iY) / denom
    let new_x = x.checked_div(&denom)?;
    let neg_y = y.negate();
    let new_y = neg_y.checked_div(&denom)?;

    state.stack.x = new_x;
    state.stack.y = new_y;

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

/// E↑Z — complex exponential: e^(X+iY) = e^X · (cos(Y) + i·sin(Y)).
///
/// Uses f64 bridge for exp, cos, sin (rust_decimal has no complex exponential).
/// No domain restriction. LiftEffect: Disable. Sets complex_mode = true. OM p.~25.
///
/// CMPLX-10 / HP 00041-90034 ~p.25.
pub fn op_exp_z(state: &mut CalcState) -> Result<(), HpError> {
    state.complex_mode = true;

    let x_f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y_f = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;

    // e^(x+iy) = e^x · (cos(y) + i·sin(y))
    let exp_x = x_f.exp();
    let new_re = exp_x * y_f.cos();
    let new_im = exp_x * y_f.sin();

    let new_x = Decimal::from_f64(new_re)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    let new_y = Decimal::from_f64(new_im)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    state.stack.x = new_x;
    state.stack.y = new_y;

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

/// LNZ — complex natural logarithm: ln(X+iY) = ln|ζ| + i·arg(ζ).
///
/// arg(ζ) = complex_atan2(Y, X) — converted from radians to current angle_mode.
/// **Guard (X=0 AND Y=0) → HpError::Domain** (CMPLX-11 / Pitfall 6).
/// LiftEffect: Disable. Sets complex_mode = true. OM p.~26.
///
/// CMPLX-11 / HP 00041-90034 ~p.26.
pub fn op_ln_z(state: &mut CalcState) -> Result<(), HpError> {
    // Pitfall 6 / CMPLX-11: Domain guard BEFORE any mutation
    if state.stack.x.is_zero() && state.stack.y.is_zero() {
        return Err(HpError::Domain);
    }

    state.complex_mode = true;

    let x_f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y_f = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;

    // Magnitude: sqrt(X² + Y²) — guaranteed > 0 (zero check above)
    let mag_f = (x_f * x_f + y_f * y_f).sqrt();
    let ln_mag = mag_f.ln(); // ln of a positive real — always finite

    // Argument: atan2(Y, X) in radians, then convert to current angle_mode
    let theta_rad = y_f.atan2(x_f);
    let theta = f64_from_radians(theta_rad, state.angle_mode);

    let new_x = Decimal::from_f64(ln_mag)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    let new_y = Decimal::from_f64(theta)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    state.stack.x = HpNum::rounded(new_x.inner());
    state.stack.y = HpNum::rounded(new_y.inner());

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

/// LOGZ — complex base-10 logarithm: LNZ(ζ) / ln(10).
///
/// Inherits LNZ's Domain guard: (0+0i) → HpError::Domain.
/// LiftEffect: Disable. Sets complex_mode = true. OM p.~26.
///
/// CMPLX-12 / HP 00041-90034 ~p.26.
pub fn op_log_z(state: &mut CalcState) -> Result<(), HpError> {
    // Pitfall 6: Domain guard BEFORE any mutation (same as LNZ)
    if state.stack.x.is_zero() && state.stack.y.is_zero() {
        return Err(HpError::Domain);
    }

    state.complex_mode = true;

    let x_f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y_f = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;

    let mag_f = (x_f * x_f + y_f * y_f).sqrt();
    let ln_mag = mag_f.ln();
    let theta_rad = y_f.atan2(x_f);
    let theta = f64_from_radians(theta_rad, state.angle_mode);

    // Divide both real and imaginary parts by ln(10) ≈ 2.302585093
    let ln_10 = std::f64::consts::LN_10;
    let new_re = ln_mag / ln_10;
    let new_im = theta / ln_10;

    let new_x = Decimal::from_f64(new_re)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    let new_y = Decimal::from_f64(new_im)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    state.stack.x = HpNum::rounded(new_x.inner());
    state.stack.y = HpNum::rounded(new_y.inner());

    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    /// Build a CalcState with X, Y, Z, T pre-loaded from &str values.
    fn make_state(x: &str, y: &str, z: &str, t: &str) -> CalcState {
        let mut state = CalcState::new();
        state.stack.x = parse_hpnum(x);
        state.stack.y = parse_hpnum(y);
        state.stack.z = parse_hpnum(z);
        state.stack.t = parse_hpnum(t);
        state
    }

    fn parse_hpnum(s: &str) -> HpNum {
        let d = rust_decimal::Decimal::from_str_exact(s)
            .or_else(|_| rust_decimal::Decimal::from_scientific(s))
            .unwrap();
        HpNum::rounded(d)
    }

    fn get_x_f64(state: &CalcState) -> f64 {
        state.stack.x.inner().to_f64().unwrap()
    }

    fn get_y_f64(state: &CalcState) -> f64 {
        state.stack.y.inner().to_f64().unwrap()
    }

    fn get_z_f64(state: &CalcState) -> f64 {
        state.stack.z.inner().to_f64().unwrap()
    }

    fn get_t_f64(state: &CalcState) -> f64 {
        state.stack.t.inner().to_f64().unwrap()
    }

    // ── complex_atan2 tests (Pitfall 6 gate) ─────────────────────────────────

    /// Catches: unhandled (0,0) case returning NaN or Domain instead of 0.
    #[test]
    fn complex_atan2_zero_zero_returns_zero() {
        let result = complex_atan2(HpNum::zero(), HpNum::zero());
        assert!(result.is_zero(), "atan2(0,0) must return HpNum::zero()");
    }

    /// Catches: wrong quadrant for pure imaginary positive axis.
    /// atan2(1, 0) = π/2 ≈ 1.5707963268
    #[test]
    fn complex_atan2_im_one_re_zero_is_pi_over_2() {
        let result = complex_atan2(parse_hpnum("1"), HpNum::zero());
        assert_relative_eq!(
            result.inner().to_f64().unwrap(),
            std::f64::consts::FRAC_PI_2,
            max_relative = 1e-7
        );
    }

    /// Catches: wrong result for pure real positive axis.
    /// atan2(0, 1) = 0
    #[test]
    fn complex_atan2_im_zero_re_one_is_zero() {
        let result = complex_atan2(HpNum::zero(), parse_hpnum("1"));
        assert_relative_eq!(result.inner().to_f64().unwrap(), 0.0, max_relative = 1e-7);
    }

    /// Catches: wrong result for first quadrant.
    /// atan2(1, 1) = π/4 ≈ 0.7853981634
    #[test]
    fn complex_atan2_both_one_is_pi_over_4() {
        let result = complex_atan2(parse_hpnum("1"), parse_hpnum("1"));
        assert_relative_eq!(
            result.inner().to_f64().unwrap(),
            std::f64::consts::FRAC_PI_4,
            max_relative = 1e-7
        );
    }

    /// Catches: wrong sign for negative imaginary axis.
    /// atan2(-1, 0) = -π/2
    #[test]
    fn complex_atan2_neg_im_is_neg_pi_over_2() {
        let result = complex_atan2(parse_hpnum("-1"), HpNum::zero());
        assert_relative_eq!(
            result.inner().to_f64().unwrap(),
            -std::f64::consts::FRAC_PI_2,
            max_relative = 1e-7
        );
    }

    // ── Op::CPlus tests (≥ 5) ────────────────────────────────────────────────

    /// Catches: wrong real addition formula.
    /// (1+2i) + (3+4i) = 4+6i
    /// Source: HP 00041-90034 p.24, complex addition example.
    #[test]
    fn c_plus_positive_case() {
        // ζ = X+iY = 1+2i; τ = Z+iT = 3+4i
        let mut s = make_state("1", "2", "3", "4");
        op_c_plus(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 4.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 6.0, max_relative = 1e-7);
    }

    /// Catches: T-replicate not happening (Z should equal old T after op).
    /// Source: HP 00041-90034 §"Stack Operations" — T-replicate on stack drop.
    #[test]
    fn c_plus_t_replicate_after_op() {
        let mut s = make_state("1", "2", "3", "4");
        op_c_plus(&mut s).unwrap();
        // old T was 4.0; new Z and T must both be 4.0
        assert_relative_eq!(get_z_f64(&s), 4.0, max_relative = 1e-7);
        assert_relative_eq!(get_t_f64(&s), 4.0, max_relative = 1e-7);
    }

    /// Catches: complex_mode not set to true by C+.
    #[test]
    fn c_plus_sets_complex_mode() {
        let mut s = make_state("1", "2", "3", "4");
        assert!(!s.complex_mode, "complex_mode must start false");
        op_c_plus(&mut s).unwrap();
        assert!(s.complex_mode, "C+ must set complex_mode = true (D-28.2 auto-on)");
    }

    /// Catches: lift_enabled not set to true by C+ (binary op must Enable).
    #[test]
    fn c_plus_enables_lift() {
        let mut s = make_state("1", "2", "3", "4");
        s.stack.lift_enabled = false; // start disabled
        op_c_plus(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "C+ must enable stack lift (LiftEffect::Enable)");
    }

    /// Catches: zero-zero identity failing.
    #[test]
    fn c_plus_zero_zero_identity() {
        let mut s = make_state("0", "0", "0", "0");
        op_c_plus(&mut s).unwrap();
        assert!(s.stack.x.is_zero());
        assert!(s.stack.y.is_zero());
    }

    /// Catches: negative imaginary part miscalculated.
    /// (-1+(-2)i) + (1+2i) = 0+0i
    #[test]
    fn c_plus_cancels_to_zero() {
        let mut s = make_state("-1", "-2", "1", "2");
        op_c_plus(&mut s).unwrap();
        assert!(s.stack.x.is_zero(), "real part must be 0");
        assert!(s.stack.y.is_zero(), "imag part must be 0");
    }

    // ── Op::CMinus tests (≥ 5) ───────────────────────────────────────────────

    /// Catches: wrong real subtraction formula.
    /// (3+4i) - (1+2i) = 2+2i
    /// Source: HP 00041-90034 p.24, complex subtraction.
    #[test]
    fn c_minus_positive_case() {
        let mut s = make_state("3", "4", "1", "2");
        op_c_minus(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 2.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 2.0, max_relative = 1e-7);
    }

    /// Catches: sign error in imaginary subtraction.
    /// (1+1i) - (1+2i) = 0+(-1)i
    #[test]
    fn c_minus_negative_imag_result() {
        let mut s = make_state("1", "1", "1", "2");
        op_c_minus(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), -1.0, max_relative = 1e-7);
    }

    /// Catches: T-replicate not applied after C-.
    #[test]
    fn c_minus_t_replicate() {
        let mut s = make_state("3", "4", "1", "2");
        op_c_minus(&mut s).unwrap();
        assert_relative_eq!(get_z_f64(&s), 2.0, max_relative = 1e-7); // old T = 2
        assert_relative_eq!(get_t_f64(&s), 2.0, max_relative = 1e-7);
    }

    /// Catches: complex_mode not set by C-.
    #[test]
    fn c_minus_sets_complex_mode() {
        let mut s = make_state("3", "4", "1", "2");
        op_c_minus(&mut s).unwrap();
        assert!(s.complex_mode, "C- must set complex_mode = true");
    }

    /// Catches: lift_enabled not enabled by C-.
    #[test]
    fn c_minus_enables_lift() {
        let mut s = make_state("3", "4", "1", "2");
        s.stack.lift_enabled = false;
        op_c_minus(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "C- must enable stack lift");
    }

    /// Catches: self-subtraction not producing zero.
    #[test]
    fn c_minus_self_is_zero() {
        let mut s = make_state("5", "7", "5", "7");
        op_c_minus(&mut s).unwrap();
        assert!(s.stack.x.is_zero());
        assert!(s.stack.y.is_zero());
    }

    // ── Op::CTimes tests (≥ 5) ───────────────────────────────────────────────

    /// Catches: wrong multiplication formula (cross terms mixed up).
    /// (1+1i) * (1+1i) = (1-1) + i(1+1) = 0+2i
    /// Source: HP 00041-90034 p.25, complex multiplication.
    #[test]
    fn c_times_unit_complex_squared() {
        let mut s = make_state("1", "1", "1", "1");
        op_c_times(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 2.0, max_relative = 1e-7);
    }

    /// Catches: pure-real path not working (imaginary parts both zero).
    /// (3+0i) * (2+0i) = 6+0i
    #[test]
    fn c_times_pure_real() {
        let mut s = make_state("3", "0", "2", "0");
        op_c_times(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 6.0, max_relative = 1e-7);
        assert!(s.stack.y.is_zero());
    }

    /// Catches: i^2 = -1 not computed correctly.
    /// (0+1i) * (0+1i) = -1+0i  (since i*i = -1)
    #[test]
    fn c_times_i_squared_is_negative_one() {
        let mut s = make_state("0", "1", "0", "1");
        op_c_times(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), -1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: complex_mode not set by C×.
    #[test]
    fn c_times_sets_complex_mode() {
        let mut s = make_state("1", "1", "1", "1");
        op_c_times(&mut s).unwrap();
        assert!(s.complex_mode, "C× must set complex_mode = true");
    }

    /// Catches: lift_enabled not enabled by C×.
    #[test]
    fn c_times_enables_lift() {
        let mut s = make_state("1", "1", "1", "1");
        s.stack.lift_enabled = false;
        op_c_times(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "C× must enable stack lift");
    }

    /// Catches: T-replicate not applied after C×.
    #[test]
    fn c_times_t_replicate() {
        let mut s = make_state("1", "1", "1", "1");
        op_c_times(&mut s).unwrap();
        // old T = 1.0; new Z and T must both be 1.0
        assert_relative_eq!(get_z_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_t_f64(&s), 1.0, max_relative = 1e-7);
    }

    // ── Op::CDiv tests (≥ 5 including zero-divisor guard) ────────────────────

    /// Catches: zero-divisor not caught BEFORE division (Pitfall 6 / CMPLX-05).
    /// Stack: ζ = (1+1i), τ = (0+0i); must return DivideByZero WITHOUT mutating stack.
    /// Source: HP 00041-90034 C÷ algorithm — magnitude check required.
    #[test]
    fn c_div_zero_divisor_returns_divide_by_zero() {
        let mut s = make_state("1", "1", "0", "0");
        let x_before = s.stack.x.clone();
        let y_before = s.stack.y.clone();
        let result = op_c_div(&mut s);
        assert!(
            matches!(result, Err(HpError::DivideByZero)),
            "C÷ with (0+0i) divisor must return DivideByZero"
        );
        // Stack must be UNCHANGED (guard fires BEFORE any mutation)
        assert_eq!(s.stack.x, x_before, "X must be unchanged on DivideByZero");
        assert_eq!(s.stack.y, y_before, "Y must be unchanged on DivideByZero");
    }

    /// Catches: wrong division formula.
    /// (1+0i) / (0+1i) = 0 + (-1)i  (since 1/i = -i)
    /// Source: HP 00041-90034 p.25, complex division.
    /// Free42 v3.0.5: re=0, im=-1.0
    #[test]
    fn c_div_one_over_i() {
        let mut s = make_state("1", "0", "0", "1");
        op_c_div(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), -1.0, max_relative = 1e-7);
    }

    /// Catches: pure-real division broken.
    /// (4+0i) / (2+0i) = 2+0i
    #[test]
    fn c_div_pure_real() {
        let mut s = make_state("4", "0", "2", "0");
        op_c_div(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 2.0, max_relative = 1e-7);
        assert!(s.stack.y.is_zero());
    }

    /// Catches: complex_mode not set by C÷.
    #[test]
    fn c_div_sets_complex_mode() {
        let mut s = make_state("4", "0", "2", "0");
        op_c_div(&mut s).unwrap();
        assert!(s.complex_mode, "C÷ must set complex_mode = true");
    }

    /// Catches: lift_enabled not enabled by C÷.
    #[test]
    fn c_div_enables_lift() {
        let mut s = make_state("4", "0", "2", "0");
        s.stack.lift_enabled = false;
        op_c_div(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "C÷ must enable stack lift");
    }

    /// Catches: T-replicate not applied after C÷.
    #[test]
    fn c_div_t_replicate() {
        // ζ = (4+0i), τ = (2+3i)  → result depends on formula; T was 3.0
        let mut s = make_state("4", "0", "2", "3");
        op_c_div(&mut s).unwrap();
        // old T = 3.0; new Z and T must both be 3.0
        assert_relative_eq!(get_z_f64(&s), 3.0, max_relative = 1e-7);
        assert_relative_eq!(get_t_f64(&s), 3.0, max_relative = 1e-7);
    }

    // ── Op::Real tests (≥ 5) ─────────────────────────────────────────────────

    /// Catches: Op::Real not clearing complex_mode.
    #[test]
    fn real_clears_complex_mode() {
        let mut s = CalcState::new();
        s.complex_mode = true;
        op_real(&mut s).unwrap();
        assert!(!s.complex_mode, "Op::Real must set complex_mode = false");
    }

    /// Catches: Op::Real with complex_mode already false causing any issue.
    #[test]
    fn real_idempotent_when_already_false() {
        let mut s = CalcState::new();
        s.complex_mode = false;
        op_real(&mut s).unwrap();
        assert!(!s.complex_mode, "Op::Real when already false must stay false");
    }

    /// Catches: Op::Real modifying stack X.
    #[test]
    fn real_does_not_modify_stack_x() {
        let mut s = CalcState::new();
        s.complex_mode = true;
        s.stack.x = parse_hpnum("42");
        op_real(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 42.0, max_relative = 1e-7);
    }

    /// Catches: Op::Real modifying stack Y.
    #[test]
    fn real_does_not_modify_stack_y() {
        let mut s = CalcState::new();
        s.complex_mode = true;
        s.stack.y = parse_hpnum("99");
        op_real(&mut s).unwrap();
        assert_relative_eq!(get_y_f64(&s), 99.0, max_relative = 1e-7);
    }

    /// Catches: Op::Real changing lift_enabled (should be Neutral).
    #[test]
    fn real_does_not_change_lift_enabled() {
        let mut s = CalcState::new();
        s.complex_mode = true;

        // Test with lift_enabled = true
        s.stack.lift_enabled = true;
        op_real(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "Op::Real must leave lift_enabled true when it was true");

        // Test with lift_enabled = false
        s.complex_mode = true;
        s.stack.lift_enabled = false;
        op_real(&mut s).unwrap();
        assert!(!s.stack.lift_enabled, "Op::Real must leave lift_enabled false when it was false");
    }

    // ── Op::Magz tests (≥ 5) ─────────────────────────────────────────────────

    /// Catches: wrong magnitude formula (missing cross-term or wrong root).
    /// Pythagorean triple: (3, 4) → 5.
    /// Source: HP 00041-90034 ~p.25, MAGZ example.
    #[test]
    fn magz_pythagorean_triple() {
        let mut s = make_state("3", "4", "0", "0");
        op_magz(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 5.0, max_relative = 1e-7);
    }

    /// Catches: magnitude of (0, 0) not returning 0.
    #[test]
    fn magz_zero_zero_returns_zero() {
        let mut s = make_state("0", "0", "0", "0");
        op_magz(&mut s).unwrap();
        assert!(s.stack.x.is_zero(), "MAGZ(0,0) must be 0");
    }

    /// Catches: unit complex number not having magnitude 1.
    /// |(1+1i)| = √2 ≈ 1.41421356.
    /// Free42 v3.0.5: 1.4142135624.
    #[test]
    fn magz_unit_complex() {
        let mut s = make_state("1", "1", "0", "0");
        op_magz(&mut s).unwrap();
        assert_relative_eq!(
            get_x_f64(&s),
            std::f64::consts::SQRT_2,
            max_relative = 1e-7
        );
    }

    /// Catches: magnitude of negative components not producing positive result.
    /// |(-3)+(-4)i| = 5.
    #[test]
    fn magz_negative_components() {
        let mut s = make_state("-3", "-4", "0", "0");
        op_magz(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 5.0, max_relative = 1e-7);
    }

    /// Catches: complex_mode not set by MAGZ.
    #[test]
    fn magz_sets_complex_mode() {
        let mut s = make_state("3", "4", "0", "0");
        assert!(!s.complex_mode);
        op_magz(&mut s).unwrap();
        assert!(s.complex_mode, "MAGZ must set complex_mode = true");
    }

    /// Catches: lift_enabled not set to Disable by MAGZ.
    #[test]
    fn magz_disables_lift() {
        let mut s = make_state("3", "4", "0", "0");
        s.stack.lift_enabled = true;
        op_magz(&mut s).unwrap();
        assert!(!s.stack.lift_enabled, "MAGZ must disable stack lift (LiftEffect::Disable)");
    }

    /// Catches: Y register being modified by MAGZ (must stay unchanged).
    #[test]
    fn magz_leaves_y_unchanged() {
        let mut s = make_state("3", "4", "5", "6");
        op_magz(&mut s).unwrap();
        // Y must still be 4 (the imaginary part)
        assert_relative_eq!(get_y_f64(&s), 4.0, max_relative = 1e-7);
    }

    // ── Op::Cinv tests (≥ 5 including zero-divisor guard) ────────────────────

    /// Catches: 1/1 = 1 not computed correctly.
    /// CINV(1+0i) = 1+0i.
    #[test]
    fn cinv_one_over_one() {
        let mut s = make_state("1", "0", "0", "0");
        op_cinv(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: 1/i = -i not computed correctly.
    /// CINV(0+1i) = 0 - 1i.
    /// Source: HP 00041-90034 ~p.25, complex inverse example.
    /// Free42 v3.0.5: re=0, im=-1.
    #[test]
    fn cinv_one_over_i() {
        let mut s = make_state("0", "1", "0", "0");
        op_cinv(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), -1.0, max_relative = 1e-7);
    }

    /// Catches: formula wrong for mixed complex.
    /// CINV(1+1i) = 1/(1+i) = (1-i)/2 = 0.5 - 0.5i.
    /// Free42 v3.0.5: re=0.5, im=-0.5.
    #[test]
    fn cinv_unit_complex() {
        let mut s = make_state("1", "1", "0", "0");
        op_cinv(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.5, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), -0.5, max_relative = 1e-7);
    }

    /// Catches: zero divisor not caught BEFORE mutation (Pitfall 6 / CMPLX-07).
    /// CINV(0+0i) must return DivideByZero WITHOUT stack mutation.
    #[test]
    fn cinv_zero_returns_divide_by_zero() {
        let mut s = make_state("0", "0", "0", "0");
        let result = op_cinv(&mut s);
        assert!(
            matches!(result, Err(HpError::DivideByZero)),
            "CINV(0,0) must return DivideByZero"
        );
        // complex_mode must NOT have been set (guard fires before mutation)
        assert!(!s.complex_mode, "complex_mode must not be set on DivideByZero");
    }

    /// Catches: complex_mode not set by CINV.
    #[test]
    fn cinv_sets_complex_mode() {
        let mut s = make_state("1", "1", "0", "0");
        op_cinv(&mut s).unwrap();
        assert!(s.complex_mode, "CINV must set complex_mode = true");
    }

    /// Catches: lift disabled (LiftEffect::Disable) not applied by CINV.
    #[test]
    fn cinv_disables_lift() {
        let mut s = make_state("1", "1", "0", "0");
        s.stack.lift_enabled = true;
        op_cinv(&mut s).unwrap();
        assert!(!s.stack.lift_enabled, "CINV must disable stack lift");
    }

    // ── Op::ExpZ tests (≥ 5) ─────────────────────────────────────────────────

    /// Catches: e^(0+0i) not returning 1+0i.
    #[test]
    fn exp_z_zero_is_one() {
        let mut s = make_state("0", "0", "0", "0");
        op_exp_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-6);
    }

    /// Catches: Euler's formula e^(iπ) = -1 not computed correctly.
    /// e^(0 + iπ) → real ≈ -1, imaginary ≈ 0.
    /// Source: HP 00041-90034 ~p.25, complex exponential.
    /// Free42 v3.0.5: re=-1.0, im=~0 (small floating-point rounding artifact ~1e-9).
    #[test]
    fn exp_z_euler_formula() {
        let pi_str = "3.141592653589793";
        let mut s = make_state("0", pi_str, "0", "0");
        op_exp_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), -1.0, max_relative = 1e-6);
        // Imaginary part is a floating-point rounding artifact (should be 0 mathematically)
        // The HP-41 10-digit BCD would round this to 0; we allow up to 1e-6 absolute tolerance
        assert!(
            get_y_f64(&s).abs() < 1e-6,
            "imaginary part of e^(i*pi) must be ~0 (within 1e-6), got {}",
            get_y_f64(&s)
        );
    }

    /// Catches: e^(1+0i) not returning (e, 0).
    /// Free42 v3.0.5: re=2.7182818285, im=0.
    #[test]
    fn exp_z_pure_real() {
        let mut s = make_state("1", "0", "0", "0");
        op_exp_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), std::f64::consts::E, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-10);
    }

    /// Catches: complex_mode not set by ExpZ.
    #[test]
    fn exp_z_sets_complex_mode() {
        let mut s = make_state("0", "0", "0", "0");
        op_exp_z(&mut s).unwrap();
        assert!(s.complex_mode, "E↑Z must set complex_mode = true");
    }

    /// Catches: lift_enabled not disabled by ExpZ (LiftEffect::Disable).
    #[test]
    fn exp_z_disables_lift() {
        let mut s = make_state("0", "0", "0", "0");
        s.stack.lift_enabled = true;
        op_exp_z(&mut s).unwrap();
        assert!(!s.stack.lift_enabled, "E↑Z must disable stack lift");
    }

    /// Catches: e^(1+1i) wrong formula (cross-component error).
    /// e^(1+i) = e·cos(1) + i·e·sin(1) ≈ 1.4686939399 + 2.2873552872i.
    /// Free42 v3.0.5: re≈1.4686939399, im≈2.2873552872.
    #[test]
    fn exp_z_complex_result() {
        let mut s = make_state("1", "1", "0", "0");
        op_exp_z(&mut s).unwrap();
        let e = std::f64::consts::E;
        let expected_re = e * 1.0_f64.cos();
        let expected_im = e * 1.0_f64.sin();
        assert_relative_eq!(get_x_f64(&s), expected_re, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), expected_im, max_relative = 1e-7);
    }

    // ── Op::LnZ tests (≥ 5 including Domain guard) ───────────────────────────

    /// Catches: ln(1+0i) not returning (0+0i).
    /// Source: HP 00041-90034 ~p.26, LNZ example: ln(1) = 0.
    #[test]
    fn ln_z_one_is_zero() {
        let mut s = make_state("1", "0", "0", "0");
        op_ln_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: ln(e+0i) not returning (1+0i).
    /// Free42 v3.0.5: re=1.0, im=0.
    #[test]
    fn ln_z_e_is_one() {
        let e_str = "2.718281828459045";
        let mut s = make_state(e_str, "0", "0", "0");
        op_ln_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-6);
    }

    /// Catches: ln(0+1i) not returning (0 + π/2·i) in radians mode.
    /// ln(i) = 0 + i·π/2.
    /// Free42 v3.0.5 (RAD mode): re=0, im=1.5707963268.
    #[test]
    fn ln_z_pure_imaginary() {
        let mut s = make_state("0", "1", "0", "0");
        s.angle_mode = crate::state::AngleMode::Rad;
        op_ln_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        assert_relative_eq!(
            get_y_f64(&s),
            std::f64::consts::FRAC_PI_2,
            max_relative = 1e-7
        );
    }

    /// Catches: ln(-1+0i) not returning (0 + π·i) in radians mode.
    /// ln(-1) = i·π — principal branch.
    /// Free42 v3.0.5 (RAD mode): re=0, im=π≈3.1415926536.
    #[test]
    fn ln_z_neg_one_is_i_pi() {
        let mut s = make_state("-1", "0", "0", "0");
        s.angle_mode = crate::state::AngleMode::Rad;
        op_ln_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        assert_relative_eq!(
            get_y_f64(&s),
            std::f64::consts::PI,
            max_relative = 1e-7
        );
    }

    /// Catches: LNZ(0+0i) not returning Domain (CMPLX-11 / Pitfall 6).
    #[test]
    fn ln_z_zero_is_domain() {
        let mut s = make_state("0", "0", "0", "0");
        let result = op_ln_z(&mut s);
        assert!(
            matches!(result, Err(HpError::Domain)),
            "LNZ(0+0i) must return Domain (CMPLX-11)"
        );
        assert!(!s.complex_mode, "complex_mode must not be set on Domain");
    }

    /// Catches: complex_mode not set by LnZ.
    #[test]
    fn ln_z_sets_complex_mode() {
        let mut s = make_state("1", "0", "0", "0");
        op_ln_z(&mut s).unwrap();
        assert!(s.complex_mode, "LNZ must set complex_mode = true");
    }

    // ── Op::LogZ tests (≥ 5 including Domain guard) ──────────────────────────

    /// Catches: log10(10+0i) not returning (1+0i).
    /// Source: HP 00041-90034 ~p.26, LOGZ example.
    /// Free42 v3.0.5: re=1.0, im=0.
    #[test]
    fn log_z_ten_is_one() {
        let mut s = make_state("10", "0", "0", "0");
        op_log_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-6);
    }

    /// Catches: log10(100+0i) not returning (2+0i).
    /// Free42 v3.0.5: re=2.0, im=0.
    #[test]
    fn log_z_hundred_is_two() {
        let mut s = make_state("100", "0", "0", "0");
        op_log_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 2.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 0.0, max_relative = 1e-6);
    }

    /// Catches: LOGZ(0+0i) not returning Domain.
    #[test]
    fn log_z_zero_is_domain() {
        let mut s = make_state("0", "0", "0", "0");
        let result = op_log_z(&mut s);
        assert!(
            matches!(result, Err(HpError::Domain)),
            "LOGZ(0+0i) must return Domain"
        );
        assert!(!s.complex_mode, "complex_mode must not be set on Domain");
    }

    /// Catches: complex_mode not set by LOGZ.
    #[test]
    fn log_z_sets_complex_mode() {
        let mut s = make_state("10", "0", "0", "0");
        op_log_z(&mut s).unwrap();
        assert!(s.complex_mode, "LOGZ must set complex_mode = true");
    }

    /// Catches: lift not disabled by LOGZ.
    #[test]
    fn log_z_disables_lift() {
        let mut s = make_state("10", "0", "0", "0");
        s.stack.lift_enabled = true;
        op_log_z(&mut s).unwrap();
        assert!(!s.stack.lift_enabled, "LOGZ must disable stack lift");
    }

    /// Catches: log10(-1+0i) in radians not returning (0 + π/ln(10)·i).
    /// log10(-1) = i·π/ln(10) ≈ 1.3643763538i (principal branch, RAD mode).
    /// Free42 v3.0.5 (RAD): re=0, im≈1.3643763538.
    #[test]
    fn log_z_neg_one_complex() {
        let mut s = make_state("-1", "0", "0", "0");
        s.angle_mode = crate::state::AngleMode::Rad;
        op_log_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-7);
        // π / ln(10)
        let expected_im = std::f64::consts::PI / std::f64::consts::LN_10;
        assert_relative_eq!(get_y_f64(&s), expected_im, max_relative = 1e-7);
    }

    // ── complex_mode lifecycle tests ──────────────────────────────────────────

    /// Catches: complex_mode not auto-activating then de-activating correctly.
    #[test]
    fn complex_mode_auto_on_off_lifecycle() {
        let mut s = make_state("1", "2", "3", "4");
        assert!(!s.complex_mode, "starts false");
        op_c_plus(&mut s).unwrap();
        assert!(s.complex_mode, "true after C+");
        op_real(&mut s).unwrap();
        assert!(!s.complex_mode, "false after Real");
        // Another complex op re-activates
        s.stack.x = parse_hpnum("1");
        s.stack.y = parse_hpnum("2");
        s.stack.z = parse_hpnum("3");
        s.stack.t = parse_hpnum("4");
        op_c_plus(&mut s).unwrap();
        assert!(s.complex_mode, "true again after second C+");
    }
}
