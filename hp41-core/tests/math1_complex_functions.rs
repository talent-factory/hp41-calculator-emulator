// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration tests for Math Pac I complex function operations (Plan 28-04).
//!
//! These tests exercise the full dispatch chain:
//! dispatch() → op_magz/cinv/exp_z/ln_z/log_z/sin_z/cos_z/tan_z/z_pow_n/z_pow_1_n/a_pow_z/z_pow_w
//!
//! Complement the unit tests in hp41-core/src/ops/math1/complex.rs::tests.
//! Required by math1_op_test_count.rs Pitfall 16 gate (≥ 5 mentions per variant
//! in math1_*.rs test files).

#![allow(clippy::unwrap_used)]

use approx::assert_relative_eq;
use hp41_core::ops::{dispatch, Op};
use hp41_core::{AngleMode, CalcState};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

fn make_state(x: f64, y: f64, z: f64, t: f64) -> CalcState {
    let mut s = CalcState::new();
    s.stack.x = hp41_core::HpNum::rounded(Decimal::from_f64(x).unwrap());
    s.stack.y = hp41_core::HpNum::rounded(Decimal::from_f64(y).unwrap());
    s.stack.z = hp41_core::HpNum::rounded(Decimal::from_f64(z).unwrap());
    s.stack.t = hp41_core::HpNum::rounded(Decimal::from_f64(t).unwrap());
    s
}

fn get_x(s: &CalcState) -> f64 {
    s.stack.x.inner().to_f64().unwrap()
}
fn get_y(s: &CalcState) -> f64 {
    s.stack.y.inner().to_f64().unwrap()
}

// ── Op::Magz integration tests ───────────────────────────────────────────────

/// Catches: Op::Magz missing from dispatch() match (compile-time enforcement).
/// Magz(3+4i) = 5.0. Source: HP 00041-90034 ~p.25.
#[test]
fn dispatch_magz_pythagorean() {
    let mut s = make_state(3.0, 4.0, 0.0, 0.0);
    dispatch(&mut s, Op::Magz).unwrap();
    assert_relative_eq!(get_x(&s), 5.0, max_relative = 1e-7);
}

/// Catches: Magz sets complex_mode flag (auto-on D-28.2).
#[test]
fn dispatch_magz_sets_complex_mode() {
    let mut s = make_state(1.0, 1.0, 0.0, 0.0);
    assert!(!s.complex_mode);
    dispatch(&mut s, Op::Magz).unwrap();
    assert!(s.complex_mode, "Op::Magz must set complex_mode");
}

/// Catches: Magz xrom_resolve routing (MAGZ mnemonic).
/// Magz(0+0i) = 0.
#[test]
fn dispatch_magz_zero_zero() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::Magz).unwrap();
    assert!(s.stack.x.is_zero(), "Magz(0,0) must be 0");
}

/// Catches: Magz LiftEffect::Disable not applied.
#[test]
fn dispatch_magz_disables_lift() {
    let mut s = make_state(3.0, 4.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::Magz).unwrap();
    assert!(!s.stack.lift_enabled, "Magz must LiftEffect::Disable");
}

/// Catches: Magz formula wrong for pure imaginary (0+5i → 5).
#[test]
fn dispatch_magz_pure_imaginary() {
    let mut s = make_state(0.0, 5.0, 0.0, 0.0);
    dispatch(&mut s, Op::Magz).unwrap();
    assert_relative_eq!(get_x(&s), 5.0, max_relative = 1e-7);
}

// ── Op::Cinv integration tests ───────────────────────────────────────────────

/// Catches: Op::Cinv missing from dispatch() match.
/// Cinv(0+1i) = 0 - 1i. Source: HP 00041-90034 ~p.25.
#[test]
fn dispatch_cinv_one_over_i() {
    let mut s = make_state(0.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::Cinv).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), -1.0, max_relative = 1e-7);
}

/// Catches: Cinv zero-divisor guard not firing.
#[test]
fn dispatch_cinv_zero_is_divide_by_zero() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    let r = dispatch(&mut s, Op::Cinv);
    assert!(
        matches!(r, Err(hp41_core::HpError::DivideByZero)),
        "Cinv(0,0) must DivideByZero"
    );
}

/// Catches: Cinv(1+0i) = 1+0i identity.
#[test]
fn dispatch_cinv_identity() {
    let mut s = make_state(1.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::Cinv).unwrap();
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 0.0, max_relative = 1e-7);
}

/// Catches: Cinv sets complex_mode.
#[test]
fn dispatch_cinv_sets_complex_mode() {
    let mut s = make_state(1.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::Cinv).unwrap();
    assert!(s.complex_mode, "Cinv must set complex_mode");
}

/// Catches: Cinv(2+0i) = 0.5+0i (reciprocal of real).
#[test]
fn dispatch_cinv_real_recip() {
    let mut s = make_state(2.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::Cinv).unwrap();
    assert_relative_eq!(get_x(&s), 0.5, max_relative = 1e-7);
    assert!(s.stack.y.is_zero(), "Cinv(2+0i) imag must be 0");
}

// ── Op::ExpZ integration tests ────────────────────────────────────────────────

/// Catches: Op::ExpZ missing from dispatch() match.
/// e^(0+0i) = 1+0i. Source: HP 00041-90034 ~p.25.
#[test]
fn dispatch_exp_z_zero_is_one() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::ExpZ).unwrap();
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    assert!(get_y(&s).abs() < 1e-9, "ExpZ(0,0) imag must be ~0");
}

/// Catches: ExpZ LiftEffect::Disable not applied.
#[test]
fn dispatch_exp_z_disables_lift() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::ExpZ).unwrap();
    assert!(!s.stack.lift_enabled, "ExpZ must LiftEffect::Disable");
}

/// Catches: ExpZ sets complex_mode.
#[test]
fn dispatch_exp_z_sets_complex_mode() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::ExpZ).unwrap();
    assert!(s.complex_mode, "ExpZ must set complex_mode");
}

/// Catches: ExpZ formula wrong for pure real (e^(1+0i) = e).
#[test]
fn dispatch_exp_z_pure_real() {
    let mut s = make_state(1.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::ExpZ).unwrap();
    assert_relative_eq!(get_x(&s), std::f64::consts::E, max_relative = 1e-7);
    assert!(get_y(&s).abs() < 1e-9);
}

/// Catches: ExpZ Euler formula e^(iπ) ≈ -1.
#[test]
fn dispatch_exp_z_euler_approx() {
    let mut s = make_state(0.0, std::f64::consts::PI, 0.0, 0.0);
    dispatch(&mut s, Op::ExpZ).unwrap();
    assert_relative_eq!(get_x(&s), -1.0, max_relative = 1e-6);
}

// ── Op::LnZ integration tests ─────────────────────────────────────────────────

/// Catches: Op::LnZ missing from dispatch() match.
/// ln(1+0i) = 0+0i. Source: HP 00041-90034 ~p.26.
#[test]
fn dispatch_ln_z_one_is_zero() {
    let mut s = make_state(1.0, 0.0, 0.0, 0.0);
    s.angle_mode = AngleMode::Rad;
    dispatch(&mut s, Op::LnZ).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 0.0, max_relative = 1e-7);
}

/// Catches: LnZ Domain guard on (0+0i).
#[test]
fn dispatch_ln_z_zero_is_domain() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    let r = dispatch(&mut s, Op::LnZ);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "LnZ(0,0) must Domain"
    );
}

/// Catches: LnZ sets complex_mode.
#[test]
fn dispatch_ln_z_sets_complex_mode() {
    let mut s = make_state(1.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::LnZ).unwrap();
    assert!(s.complex_mode, "LnZ must set complex_mode");
}

/// Catches: LnZ angle output in radians mode (ln(-1) = iπ).
#[test]
fn dispatch_ln_z_neg_one_rad() {
    let mut s = make_state(-1.0, 0.0, 0.0, 0.0);
    s.angle_mode = AngleMode::Rad;
    dispatch(&mut s, Op::LnZ).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), std::f64::consts::PI, max_relative = 1e-7);
}

/// Catches: LnZ LiftEffect::Disable not applied.
#[test]
fn dispatch_ln_z_disables_lift() {
    let mut s = make_state(1.0, 0.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::LnZ).unwrap();
    assert!(!s.stack.lift_enabled, "LnZ must LiftEffect::Disable");
}

// ── Op::LogZ integration tests ────────────────────────────────────────────────

/// Catches: Op::LogZ missing from dispatch() match.
/// log10(10+0i) = 1+0i. Source: HP 00041-90034 ~p.26.
#[test]
fn dispatch_log_z_ten_is_one() {
    let mut s = make_state(10.0, 0.0, 0.0, 0.0);
    s.angle_mode = AngleMode::Rad;
    dispatch(&mut s, Op::LogZ).unwrap();
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    assert!(get_y(&s).abs() < 1e-9);
}

/// Catches: LogZ Domain guard on (0+0i).
#[test]
fn dispatch_log_z_zero_is_domain() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    let r = dispatch(&mut s, Op::LogZ);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "LogZ(0,0) must Domain"
    );
}

/// Catches: LogZ sets complex_mode.
#[test]
fn dispatch_log_z_sets_complex_mode() {
    let mut s = make_state(10.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::LogZ).unwrap();
    assert!(s.complex_mode, "LogZ must set complex_mode");
}

/// Catches: LogZ LiftEffect::Disable not applied.
#[test]
fn dispatch_log_z_disables_lift() {
    let mut s = make_state(10.0, 0.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::LogZ).unwrap();
    assert!(!s.stack.lift_enabled, "LogZ must LiftEffect::Disable");
}

/// Catches: LogZ(100+0i) = 2+0i.
#[test]
fn dispatch_log_z_hundred_is_two() {
    let mut s = make_state(100.0, 0.0, 0.0, 0.0);
    s.angle_mode = AngleMode::Rad;
    dispatch(&mut s, Op::LogZ).unwrap();
    assert_relative_eq!(get_x(&s), 2.0, max_relative = 1e-7);
}

// ── Op::SinZ integration tests ────────────────────────────────────────────────

/// Catches: Op::SinZ missing from dispatch() match.
/// sin(0+0i) = 0+0i. Source: HP 00041-90034 ~p.26.
#[test]
fn dispatch_sin_z_zero_is_zero() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::SinZ).unwrap();
    assert!(s.stack.x.is_zero());
    assert!(s.stack.y.is_zero());
}

/// Catches: SinZ sets complex_mode.
#[test]
fn dispatch_sin_z_sets_complex_mode() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::SinZ).unwrap();
    assert!(s.complex_mode, "SinZ must set complex_mode");
}

/// Catches: SinZ(0+1i) = (0, sinh(1)) via hyperbolic identity.
#[test]
fn dispatch_sin_z_pure_imaginary() {
    let mut s = make_state(0.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::SinZ).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 1.0_f64.sinh(), max_relative = 1e-7);
}

/// Catches: SinZ LiftEffect::Disable not applied.
#[test]
fn dispatch_sin_z_disables_lift() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::SinZ).unwrap();
    assert!(!s.stack.lift_enabled, "SinZ must LiftEffect::Disable");
}

/// Catches: SinZ(1+1i) real part wrong.
#[test]
fn dispatch_sin_z_combined() {
    let mut s = make_state(1.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::SinZ).unwrap();
    let expected = 1.0_f64.sin() * 1.0_f64.cosh();
    assert_relative_eq!(get_x(&s), expected, max_relative = 1e-7);
}

// ── Op::CosZ integration tests ────────────────────────────────────────────────

/// Catches: Op::CosZ missing from dispatch() match.
/// cos(0+0i) = 1+0i. Source: HP 00041-90034 ~p.26.
#[test]
fn dispatch_cos_z_zero_is_one() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::CosZ).unwrap();
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    assert!(s.stack.y.is_zero());
}

/// Catches: CosZ sets complex_mode.
#[test]
fn dispatch_cos_z_sets_complex_mode() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::CosZ).unwrap();
    assert!(s.complex_mode, "CosZ must set complex_mode");
}

/// Catches: CosZ(0+1i) = (cosh(1), 0) via hyperbolic identity.
#[test]
fn dispatch_cos_z_pure_imaginary() {
    let mut s = make_state(0.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::CosZ).unwrap();
    assert_relative_eq!(get_x(&s), 1.0_f64.cosh(), max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 0.0, max_relative = 1e-7);
}

/// Catches: CosZ LiftEffect::Disable not applied.
#[test]
fn dispatch_cos_z_disables_lift() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::CosZ).unwrap();
    assert!(!s.stack.lift_enabled, "CosZ must LiftEffect::Disable");
}

/// Catches: CosZ imaginary sign wrong (should be negative for non-zero real + imag).
/// cos(1+1i) im = -sin(1)*sinh(1).
#[test]
fn dispatch_cos_z_combined_im_sign() {
    let mut s = make_state(1.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::CosZ).unwrap();
    let expected_im = -(1.0_f64.sin() * 1.0_f64.sinh());
    assert_relative_eq!(get_y(&s), expected_im, max_relative = 1e-7);
}

// ── Op::TanZ integration tests ────────────────────────────────────────────────

/// Catches: Op::TanZ missing from dispatch() match.
/// tan(0+0i) = 0+0i. Source: HP 00041-90034 ~p.26.
#[test]
fn dispatch_tan_z_zero_is_zero() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::TanZ).unwrap();
    assert!(s.stack.x.is_zero());
    assert!(s.stack.y.is_zero());
}

/// Catches: TanZ Domain guard at singularity (pi/2 + 0i).
#[test]
fn dispatch_tan_z_singularity_is_domain() {
    let mut s = make_state(std::f64::consts::FRAC_PI_2, 0.0, 0.0, 0.0);
    let r = dispatch(&mut s, Op::TanZ);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "TanZ(pi/2,0) must Domain"
    );
}

/// Catches: TanZ sets complex_mode.
#[test]
fn dispatch_tan_z_sets_complex_mode() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::TanZ).unwrap();
    assert!(s.complex_mode, "TanZ must set complex_mode");
}

/// Catches: TanZ LiftEffect::Disable not applied.
#[test]
fn dispatch_tan_z_disables_lift() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::TanZ).unwrap();
    assert!(!s.stack.lift_enabled, "TanZ must LiftEffect::Disable");
}

/// Catches: TanZ(0+1i) im = tanh(1) (pure imaginary identity).
#[test]
fn dispatch_tan_z_pure_imaginary() {
    let mut s = make_state(0.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::TanZ).unwrap();
    assert!(get_x(&s).abs() < 1e-9, "TanZ(0+1i) re must be ~0");
    assert_relative_eq!(get_y(&s), 1.0_f64.tanh(), max_relative = 1e-7);
}

// ── Op::ZpowN integration tests ───────────────────────────────────────────────

/// Catches: Op::ZpowN missing from dispatch() match.
/// (2+0i)^3 = 8+0i. N=X=3, base=Y+iZ=(2,0).
#[test]
fn dispatch_z_pow_n_cube() {
    let mut s = make_state(3.0, 2.0, 0.0, 0.0);
    dispatch(&mut s, Op::ZpowN).unwrap();
    assert_relative_eq!(get_x(&s), 8.0, max_relative = 1e-7);
    assert!(get_y(&s).abs() < 1e-9);
}

/// Catches: ZpowN zero-exponent returns (1, 0).
#[test]
fn dispatch_z_pow_n_zero_exp_is_one() {
    let mut s = make_state(0.0, 5.0, 3.0, 0.0);
    dispatch(&mut s, Op::ZpowN).unwrap();
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    assert!(
        get_y(&s).is_nan() || get_y(&s).abs() < 1e-9,
        "ZpowN(z,0) imag must be ~0"
    );
}

/// Catches: ZpowN sets complex_mode.
#[test]
fn dispatch_z_pow_n_sets_complex_mode() {
    let mut s = make_state(2.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::ZpowN).unwrap();
    assert!(s.complex_mode, "ZpowN must set complex_mode");
}

/// Catches: ZpowN (1+1i)^2 = 0+2i.
#[test]
fn dispatch_z_pow_n_unit_complex_sq() {
    let mut s = make_state(2.0, 1.0, 1.0, 0.0);
    dispatch(&mut s, Op::ZpowN).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 2.0, max_relative = 1e-7);
}

/// Catches: ZpowN negative exponent computing inverse correctly.
#[test]
fn dispatch_z_pow_n_neg_exponent() {
    // (2+0i)^(-1) = (0.5+0i). N=X=-1, base=Y+iZ=(2,0).
    let mut s = make_state(-1.0, 2.0, 0.0, 0.0);
    dispatch(&mut s, Op::ZpowN).unwrap();
    assert_relative_eq!(get_x(&s), 0.5, max_relative = 1e-7);
}

// ── Op::Zpow1N integration tests ──────────────────────────────────────────────

/// Catches: Op::Zpow1N missing from dispatch() match.
/// sqrt(4+0i) = 2+0i. N=X=2, base=Y+iZ=(4,0).
#[test]
fn dispatch_zpow_1n_sqrt_four() {
    let mut s = make_state(2.0, 4.0, 0.0, 0.0);
    dispatch(&mut s, Op::Zpow1N).unwrap();
    assert_relative_eq!(get_x(&s), 2.0, max_relative = 1e-7);
    assert!(get_y(&s).abs() < 1e-9);
}

/// Catches: Zpow1N (0+0i)^(1/N) = 0 (zero-first-arm).
#[test]
fn dispatch_zpow_1n_zero_base_is_zero() {
    let mut s = make_state(5.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::Zpow1N).unwrap();
    assert!(s.stack.x.is_zero(), "Zpow1N(0, 1/5) must be 0");
}

/// Catches: Zpow1N sqrt(-1) = (0, 1) principal branch.
#[test]
fn dispatch_zpow_1n_sqrt_neg_one() {
    let mut s = make_state(2.0, -1.0, 0.0, 0.0);
    dispatch(&mut s, Op::Zpow1N).unwrap();
    assert!(get_x(&s).abs() < 1e-9, "sqrt(-1) re must be ~0");
    assert_relative_eq!(get_y(&s), 1.0, max_relative = 1e-7);
}

/// Catches: Zpow1N sets complex_mode.
#[test]
fn dispatch_zpow_1n_sets_complex_mode() {
    let mut s = make_state(2.0, 1.0, 0.0, 0.0);
    dispatch(&mut s, Op::Zpow1N).unwrap();
    assert!(s.complex_mode, "Zpow1N must set complex_mode");
}

/// Catches: Zpow1N LiftEffect::Disable not applied.
#[test]
fn dispatch_zpow_1n_disables_lift() {
    let mut s = make_state(2.0, 1.0, 0.0, 0.0);
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::Zpow1N).unwrap();
    assert!(!s.stack.lift_enabled, "Zpow1N must LiftEffect::Disable");
}

// ── Op::ApowZ integration tests ───────────────────────────────────────────────

/// Catches: Op::ApowZ missing from dispatch() match.
/// (2+0i)^(3+0i) = 8+0i. a=Z+iT=(2,0), z=X+iY=(3,0).
/// Source: HP 00041-90034 ~p.26.
#[test]
fn dispatch_a_pow_z_pure_real() {
    let mut s = make_state(3.0, 0.0, 2.0, 0.0);
    dispatch(&mut s, Op::ApowZ).unwrap();
    assert_relative_eq!(get_x(&s), 8.0, max_relative = 1e-6);
    assert!(get_y(&s).abs() < 1e-9);
}

/// Catches: ApowZ Domain guard on a=(0+0i).
#[test]
fn dispatch_a_pow_z_zero_base_is_domain() {
    let mut s = make_state(1.0, 0.0, 0.0, 0.0);
    let r = dispatch(&mut s, Op::ApowZ);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "ApowZ with a=0 must Domain"
    );
}

/// Catches: ApowZ sets complex_mode.
#[test]
fn dispatch_a_pow_z_sets_complex_mode() {
    let mut s = make_state(2.0, 0.0, 3.0, 0.0);
    dispatch(&mut s, Op::ApowZ).unwrap();
    assert!(s.complex_mode, "ApowZ must set complex_mode");
}

/// Catches: ApowZ T-replicate not applied (binary op).
#[test]
fn dispatch_a_pow_z_t_replicate() {
    // a=Z+iT=(2,7), z=X+iY=(1,0): result=(2,~0); old T was 7
    let mut s = make_state(1.0, 0.0, 2.0, 7.0);
    dispatch(&mut s, Op::ApowZ).unwrap();
    assert_relative_eq!(
        s.stack.z.inner().to_f64().unwrap(),
        7.0,
        max_relative = 1e-7
    );
    assert_relative_eq!(
        s.stack.t.inner().to_f64().unwrap(),
        7.0,
        max_relative = 1e-7
    );
}

/// Catches: ApowZ LiftEffect::Enable not applied.
#[test]
fn dispatch_a_pow_z_enables_lift() {
    let mut s = make_state(2.0, 0.0, 3.0, 0.0);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::ApowZ).unwrap();
    assert!(
        s.stack.lift_enabled,
        "ApowZ must LiftEffect::Enable (binary)"
    );
}

// ── Op::ZpowW integration tests ───────────────────────────────────────────────

/// Catches: Op::ZpowW missing from dispatch() match.
/// (2+0i)^(3+0i) = 8+0i. z=X+iY=(2,0), w=Z+iT=(3,0).
/// Source: HP 00041-90034 ~p.26.
#[test]
fn dispatch_z_pow_w_pure_real() {
    let mut s = make_state(2.0, 0.0, 3.0, 0.0);
    dispatch(&mut s, Op::ZpowW).unwrap();
    assert_relative_eq!(get_x(&s), 8.0, max_relative = 1e-6);
    assert!(get_y(&s).abs() < 1e-9);
}

/// Catches: ZpowW Domain guard on (0+0i)^(negative).
#[test]
fn dispatch_z_pow_w_zero_neg_exp_is_domain() {
    let mut s = make_state(0.0, 0.0, -1.0, 0.0);
    let r = dispatch(&mut s, Op::ZpowW);
    assert!(
        matches!(r, Err(hp41_core::HpError::Domain)),
        "ZpowW(0,0,-1) must Domain"
    );
}

/// Catches: ZpowW sets complex_mode.
#[test]
fn dispatch_z_pow_w_sets_complex_mode() {
    let mut s = make_state(2.0, 0.0, 3.0, 0.0);
    dispatch(&mut s, Op::ZpowW).unwrap();
    assert!(s.complex_mode, "ZpowW must set complex_mode");
}

/// Catches: ZpowW T-replicate not applied (binary op).
#[test]
fn dispatch_z_pow_w_t_replicate() {
    // z=(2,0), w=(3,7): T was 7
    let mut s = make_state(2.0, 0.0, 3.0, 7.0);
    dispatch(&mut s, Op::ZpowW).unwrap();
    assert_relative_eq!(
        s.stack.z.inner().to_f64().unwrap(),
        7.0,
        max_relative = 1e-7
    );
    assert_relative_eq!(
        s.stack.t.inner().to_f64().unwrap(),
        7.0,
        max_relative = 1e-7
    );
}

/// Catches: ZpowW (0+0i)^(positive) = 0 (not Domain).
#[test]
fn dispatch_z_pow_w_zero_pos_exp_is_zero() {
    let mut s = make_state(0.0, 0.0, 1.0, 0.0);
    dispatch(&mut s, Op::ZpowW).unwrap();
    assert!(s.stack.x.is_zero(), "ZpowW(0,0,1) must be 0");
}
