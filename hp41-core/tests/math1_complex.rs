// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration tests for Math Pac I complex arithmetic operations (Plan 28-03).
//!
//! These tests exercise the full dispatch chain:
//! dispatch() → op_c_plus/c_minus/c_times/c_div/real
//!
//! Complement the unit tests in hp41-core/src/ops/math1/complex.rs::tests.
//! Required by math1_op_test_count.rs Pitfall 16 gate (≥ 5 mentions per variant
//! in math1_*.rs test files).
//!
//! Complex stack overlay: ζ = X+iY, τ = Z+iT (D-28.1).

#![allow(clippy::unwrap_used)]

use approx::assert_relative_eq;
use hp41_core::ops::{dispatch, Op};
use hp41_core::CalcState;
use rust_decimal::prelude::ToPrimitive;

fn make_state(x: f64, y: f64, z: f64, t: f64) -> CalcState {
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;
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

// ── Op::CPlus — integration tests via dispatch() ─────────────────────────────

/// Catches: CPlus variant missing from dispatch() match (compile-time + runtime).
/// Source: HP 00041-90034 p.24.
#[test]
fn dispatch_c_plus_basic() {
    let mut s = make_state(1.0, 2.0, 3.0, 4.0);
    dispatch(&mut s, Op::CPlus).unwrap();
    assert_relative_eq!(get_x(&s), 4.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 6.0, max_relative = 1e-7);
}

/// Catches: CPlus complex_mode not set via dispatch path.
#[test]
fn dispatch_c_plus_sets_complex_mode() {
    let mut s = make_state(1.0, 0.0, 1.0, 0.0);
    assert!(!s.complex_mode);
    dispatch(&mut s, Op::CPlus).unwrap();
    assert!(s.complex_mode, "CPlus dispatch must set complex_mode");
}

/// Catches: CPlus lift_enabled not set via dispatch path.
#[test]
fn dispatch_c_plus_enables_lift() {
    let mut s = make_state(1.0, 0.0, 1.0, 0.0);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::CPlus).unwrap();
    assert!(s.stack.lift_enabled, "CPlus must enable lift via dispatch");
}

/// Catches: CPlus T-replicate not applied via dispatch path.
#[test]
fn dispatch_c_plus_t_replicate() {
    let mut s = make_state(1.0, 2.0, 3.0, 5.0); // old T = 5.0
    dispatch(&mut s, Op::CPlus).unwrap();
    assert_relative_eq!(s.stack.z.inner().to_f64().unwrap(), 5.0, max_relative = 1e-7);
    assert_relative_eq!(s.stack.t.inner().to_f64().unwrap(), 5.0, max_relative = 1e-7);
}

/// Catches: CPlus LASTX not saved on the dispatch path.
#[test]
fn dispatch_c_plus_saves_lastx() {
    let mut s = make_state(7.0, 2.0, 3.0, 4.0); // X = 7.0 before op
    dispatch(&mut s, Op::CPlus).unwrap();
    assert_relative_eq!(s.stack.lastx.inner().to_f64().unwrap(), 7.0, max_relative = 1e-7);
}

/// Catches: CPlus zero-zero degenerate case failing.
#[test]
fn dispatch_c_plus_zero_plus_zero() {
    let mut s = make_state(0.0, 0.0, 0.0, 0.0);
    dispatch(&mut s, Op::CPlus).unwrap();
    assert!(s.stack.x.is_zero());
    assert!(s.stack.y.is_zero());
}

// ── Op::CMinus — integration tests via dispatch() ────────────────────────────

/// Catches: CMinus variant missing from dispatch() match (compile-time + runtime).
/// Source: HP 00041-90034 p.24.
#[test]
fn dispatch_c_minus_basic() {
    let mut s = make_state(5.0, 3.0, 2.0, 1.0);
    dispatch(&mut s, Op::CMinus).unwrap();
    assert_relative_eq!(get_x(&s), 3.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 2.0, max_relative = 1e-7);
}

/// Catches: CMinus complex_mode not set via dispatch path.
#[test]
fn dispatch_c_minus_sets_complex_mode() {
    let mut s = make_state(5.0, 3.0, 2.0, 1.0);
    dispatch(&mut s, Op::CMinus).unwrap();
    assert!(s.complex_mode, "CMinus dispatch must set complex_mode");
}

/// Catches: CMinus lift_enabled not set via dispatch path.
#[test]
fn dispatch_c_minus_enables_lift() {
    let mut s = make_state(5.0, 3.0, 2.0, 1.0);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::CMinus).unwrap();
    assert!(s.stack.lift_enabled, "CMinus must enable lift via dispatch");
}

/// Catches: CMinus T-replicate not applied via dispatch path.
#[test]
fn dispatch_c_minus_t_replicate() {
    let mut s = make_state(5.0, 3.0, 2.0, 9.0); // old T = 9.0
    dispatch(&mut s, Op::CMinus).unwrap();
    assert_relative_eq!(s.stack.t.inner().to_f64().unwrap(), 9.0, max_relative = 1e-7);
}

/// Catches: CMinus sign error in subtraction.
/// (0+0i) - (1+1i) = -1-1i
#[test]
fn dispatch_c_minus_negative_result() {
    let mut s = make_state(0.0, 0.0, 1.0, 1.0);
    dispatch(&mut s, Op::CMinus).unwrap();
    assert_relative_eq!(get_x(&s), -1.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), -1.0, max_relative = 1e-7);
}

/// Catches: CMinus self-subtraction not producing zero.
#[test]
fn dispatch_c_minus_self_is_zero() {
    let mut s = make_state(3.0, 4.0, 3.0, 4.0);
    dispatch(&mut s, Op::CMinus).unwrap();
    assert!(s.stack.x.is_zero());
    assert!(s.stack.y.is_zero());
}

// ── Op::CTimes — integration tests via dispatch() ────────────────────────────

/// Catches: CTimes variant missing from dispatch() match (compile-time + runtime).
/// (1+1i)(1+1i) = 0+2i. Source: HP 00041-90034 p.25.
#[test]
fn dispatch_c_times_basic() {
    let mut s = make_state(1.0, 1.0, 1.0, 1.0);
    dispatch(&mut s, Op::CTimes).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 2.0, max_relative = 1e-7);
}

/// Catches: CTimes complex_mode not set via dispatch path.
#[test]
fn dispatch_c_times_sets_complex_mode() {
    let mut s = make_state(2.0, 0.0, 3.0, 0.0);
    dispatch(&mut s, Op::CTimes).unwrap();
    assert!(s.complex_mode, "CTimes dispatch must set complex_mode");
}

/// Catches: CTimes lift_enabled not set via dispatch path.
#[test]
fn dispatch_c_times_enables_lift() {
    let mut s = make_state(2.0, 0.0, 3.0, 0.0);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::CTimes).unwrap();
    assert!(s.stack.lift_enabled, "CTimes must enable lift via dispatch");
}

/// Catches: CTimes T-replicate not applied via dispatch path.
#[test]
fn dispatch_c_times_t_replicate() {
    let mut s = make_state(1.0, 0.0, 2.0, 7.0); // old T = 7.0
    dispatch(&mut s, Op::CTimes).unwrap();
    assert_relative_eq!(s.stack.t.inner().to_f64().unwrap(), 7.0, max_relative = 1e-7);
}

/// Catches: CTimes cross-term mixing.
/// (0+1i)(0-1i) = -i*i = -(i^2) = 1+0i
#[test]
fn dispatch_c_times_conjugate_multiply() {
    let mut s = make_state(0.0, 1.0, 0.0, -1.0);
    dispatch(&mut s, Op::CTimes).unwrap();
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 0.0, max_relative = 1e-7);
}

/// Catches: CTimes real-by-imaginary cross product.
/// (2+0i)(0+3i) = 0+6i
#[test]
fn dispatch_c_times_real_times_imaginary() {
    let mut s = make_state(2.0, 0.0, 0.0, 3.0);
    dispatch(&mut s, Op::CTimes).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 6.0, max_relative = 1e-7);
}

// ── Op::CDiv — integration tests via dispatch() ──────────────────────────────

/// Catches: CDiv variant missing from dispatch() match (compile-time + runtime).
/// (4+0i)/(2+0i) = 2+0i. Source: HP 00041-90034 p.25.
#[test]
fn dispatch_c_div_pure_real() {
    let mut s = make_state(4.0, 0.0, 2.0, 0.0);
    dispatch(&mut s, Op::CDiv).unwrap();
    assert_relative_eq!(get_x(&s), 2.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 0.0, max_relative = 1e-7);
}

/// Catches: CDiv zero-divisor guard missing (CMPLX-05 / Pitfall 6).
/// (1+1i)/(0+0i) must return DivideByZero.
#[test]
fn dispatch_c_div_zero_divisor_returns_error() {
    let mut s = make_state(1.0, 1.0, 0.0, 0.0);
    let result = dispatch(&mut s, Op::CDiv);
    assert!(
        matches!(result, Err(hp41_core::HpError::DivideByZero)),
        "CDiv zero-divisor must return DivideByZero via dispatch"
    );
}

/// Catches: CDiv complex_mode not set via dispatch path.
#[test]
fn dispatch_c_div_sets_complex_mode() {
    let mut s = make_state(4.0, 0.0, 2.0, 0.0);
    dispatch(&mut s, Op::CDiv).unwrap();
    assert!(s.complex_mode, "CDiv dispatch must set complex_mode");
}

/// Catches: CDiv lift_enabled not set via dispatch path.
#[test]
fn dispatch_c_div_enables_lift() {
    let mut s = make_state(4.0, 0.0, 2.0, 0.0);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::CDiv).unwrap();
    assert!(s.stack.lift_enabled, "CDiv must enable lift via dispatch");
}

/// Catches: CDiv formula error for complex/complex case.
/// (1+0i)/(0+1i) = 0+(-1)i (since 1/i = -i).
/// Free42 v3.0.5: re=0, im=-1.
#[test]
fn dispatch_c_div_one_over_i() {
    let mut s = make_state(1.0, 0.0, 0.0, 1.0);
    dispatch(&mut s, Op::CDiv).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), -1.0, max_relative = 1e-7);
}

/// Catches: CDiv T-replicate not applied via dispatch path.
#[test]
fn dispatch_c_div_t_replicate() {
    let mut s = make_state(4.0, 0.0, 2.0, 8.0); // old T = 8.0
    dispatch(&mut s, Op::CDiv).unwrap();
    assert_relative_eq!(s.stack.t.inner().to_f64().unwrap(), 8.0, max_relative = 1e-7);
}

// ── Op::Real — integration tests via dispatch() ──────────────────────────────

/// Catches: Real variant missing from dispatch() match (compile-time + runtime).
#[test]
fn dispatch_real_clears_complex_mode() {
    let mut s = CalcState::new();
    s.complex_mode = true;
    dispatch(&mut s, Op::Real).unwrap();
    assert!(!s.complex_mode, "Real dispatch must clear complex_mode");
}

/// Catches: Real leaving stack modified via dispatch path.
#[test]
fn dispatch_real_does_not_touch_stack_x() {
    use hp41_core::HpNum;
    let mut s = CalcState::new();
    s.complex_mode = true;
    s.stack.x = HpNum::rounded(rust_decimal::Decimal::from(42i32));
    dispatch(&mut s, Op::Real).unwrap();
    assert_relative_eq!(get_x(&s), 42.0, max_relative = 1e-7);
}

/// Catches: Real modifying lift_enabled (must be Neutral).
#[test]
fn dispatch_real_lift_neutral_when_true() {
    let mut s = CalcState::new();
    s.complex_mode = true;
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::Real).unwrap();
    assert!(s.stack.lift_enabled, "Real is Neutral — must not disable lift when it was true");
}

/// Catches: Real modifying lift_enabled (must be Neutral) — false branch.
#[test]
fn dispatch_real_lift_neutral_when_false() {
    let mut s = CalcState::new();
    s.complex_mode = true;
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Real).unwrap();
    assert!(!s.stack.lift_enabled, "Real is Neutral — must not enable lift when it was false");
}

/// Catches: Real not idempotent (second call should not fail or change state).
#[test]
fn dispatch_real_idempotent() {
    let mut s = CalcState::new();
    s.complex_mode = false;
    dispatch(&mut s, Op::Real).unwrap();
    assert!(!s.complex_mode, "Real when complex_mode already false must stay false");
}

/// Catches: Real not returning Ok (should never error).
#[test]
fn dispatch_real_always_returns_ok() {
    let mut s = CalcState::new();
    s.complex_mode = true;
    let result = dispatch(&mut s, Op::Real);
    assert!(result.is_ok(), "Real must always return Ok(())");
}
