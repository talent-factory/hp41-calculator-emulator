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

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

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
