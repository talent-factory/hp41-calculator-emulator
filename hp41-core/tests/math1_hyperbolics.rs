// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration tests for Math Pac I hyperbolic operations (Plan 28-02).
//!
//! These tests exercise the full dispatch chain:
//! dispatch() → op_sinh/cosh/tanh/asinh/acosh/atanh → unary_result
//!
//! Complement the unit tests in hp41-core/src/ops/math1/hyperbolics.rs::tests.
//! Required by math1_op_test_count.rs Pitfall 16 gate (≥ 5 mentions per variant
//! in math1_*.rs test files).

#![allow(clippy::unwrap_used)]

use approx::assert_relative_eq;
use hp41_core::ops::{dispatch, Op};
use hp41_core::CalcState;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::str::FromStr;

fn make_state_with_x(x_str: &str) -> CalcState {
    let mut state = CalcState::new();
    let d = Decimal::from_str(x_str)
        .or_else(|_| Decimal::from_scientific(x_str))
        .expect("valid decimal in integration test");
    state.stack.x = hp41_core::HpNum::rounded(d);
    state
}

fn get_x(state: &CalcState) -> f64 {
    state.stack.x.inner().to_f64().unwrap()
}

// ── Op::Sinh — integration tests via dispatch() ─────────────────────────────

/// Catches: Op::Sinh variant missing from dispatch() match block (compile-time + runtime).
#[test]
fn dispatch_sinh_zero_identity() {
    let mut s = make_state_with_x("0");
    dispatch(&mut s, Op::Sinh).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
}

/// Catches: Op::Sinh dispatch not reaching op_sinh (wrong function called).
/// Source: HP 00041-90034 p.44, ex.1 — sinh(1) = 1.175201194
/// Free42 v3.0.5: 1.1752011936 — agrees with OM
#[test]
fn dispatch_sinh_reference_value() {
    let mut s = make_state_with_x("1");
    dispatch(&mut s, Op::Sinh).unwrap();
    assert_relative_eq!(get_x(&s), 1.175_201_193_6, max_relative = 1e-7);
}

/// Catches: Op::Sinh LASTX not being saved by unary_result.
#[test]
fn dispatch_sinh_saves_lastx() {
    let mut s = make_state_with_x("2");
    let orig = s.stack.x.clone();
    dispatch(&mut s, Op::Sinh).unwrap();
    assert_eq!(s.stack.lastx, orig, "Op::Sinh must save X to LASTX");
}

/// Catches: Op::Sinh not enabling stack lift after dispatch.
#[test]
fn dispatch_sinh_enables_lift() {
    let mut s = make_state_with_x("1");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Sinh).unwrap();
    assert!(s.stack.lift_enabled, "Op::Sinh must enable stack lift");
}

/// Catches: Op::Sinh overflow not propagated from dispatch layer.
#[test]
fn dispatch_sinh_overflow() {
    let mut s = make_state_with_x("1000");
    let result = dispatch(&mut s, Op::Sinh);
    assert_eq!(result, Err(hp41_core::HpError::Overflow));
}

// ── Op::Cosh — integration tests via dispatch() ─────────────────────────────

/// Catches: Op::Cosh variant missing from dispatch() match block.
#[test]
fn dispatch_cosh_zero_is_one() {
    let mut s = make_state_with_x("0");
    dispatch(&mut s, Op::Cosh).unwrap();
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
}

/// Catches: Op::Cosh dispatch not reaching op_cosh.
/// Source: HP 00041-90034 p.44, ex.3 — cosh(1) = 1.543080635
/// Free42 v3.0.5: 1.5430806348 — agrees with OM
#[test]
fn dispatch_cosh_reference_value() {
    let mut s = make_state_with_x("1");
    dispatch(&mut s, Op::Cosh).unwrap();
    assert_relative_eq!(get_x(&s), 1.543_080_634_8, max_relative = 1e-7);
}

/// Catches: Op::Cosh LASTX not saved.
#[test]
fn dispatch_cosh_saves_lastx() {
    let mut s = make_state_with_x("1");
    let orig = s.stack.x.clone();
    dispatch(&mut s, Op::Cosh).unwrap();
    assert_eq!(s.stack.lastx, orig, "Op::Cosh must save X to LASTX");
}

/// Catches: Op::Cosh not enabling stack lift.
#[test]
fn dispatch_cosh_enables_lift() {
    let mut s = make_state_with_x("0");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Cosh).unwrap();
    assert!(s.stack.lift_enabled, "Op::Cosh must enable stack lift");
}

/// Catches: Op::Cosh overflow not propagated.
#[test]
fn dispatch_cosh_overflow() {
    let mut s = make_state_with_x("1000");
    let result = dispatch(&mut s, Op::Cosh);
    assert_eq!(result, Err(hp41_core::HpError::Overflow));
}

// ── Op::Tanh — integration tests via dispatch() ─────────────────────────────

/// Catches: Op::Tanh variant missing from dispatch() match block.
#[test]
fn dispatch_tanh_zero_identity() {
    let mut s = make_state_with_x("0");
    dispatch(&mut s, Op::Tanh).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
}

/// Catches: Op::Tanh dispatch not reaching op_tanh.
/// Source: HP 00041-90034 p.44, ex.5 — tanh(1) = 0.761594156
/// Free42 v3.0.5: 0.7615941560 — agrees with OM
#[test]
fn dispatch_tanh_reference_value() {
    let mut s = make_state_with_x("1");
    dispatch(&mut s, Op::Tanh).unwrap();
    assert_relative_eq!(get_x(&s), 0.761_594_156_0, max_relative = 1e-7);
}

/// Catches: Op::Tanh LASTX not saved.
#[test]
fn dispatch_tanh_saves_lastx() {
    let mut s = make_state_with_x("1");
    let orig = s.stack.x.clone();
    dispatch(&mut s, Op::Tanh).unwrap();
    assert_eq!(s.stack.lastx, orig, "Op::Tanh must save X to LASTX");
}

/// Catches: Op::Tanh not enabling stack lift.
#[test]
fn dispatch_tanh_enables_lift() {
    let mut s = make_state_with_x("1");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Tanh).unwrap();
    assert!(s.stack.lift_enabled, "Op::Tanh must enable stack lift");
}

/// Catches: Op::Tanh incorrectly failing at saturation magnitudes (should not overflow).
#[test]
fn dispatch_tanh_saturates_not_overflows() {
    let mut s = make_state_with_x("100");
    let result = dispatch(&mut s, Op::Tanh);
    assert!(
        result.is_ok(),
        "Op::Tanh must saturate to ±1 for large |X|, not return error"
    );
    assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
}

// ── Op::Asinh — integration tests via dispatch() ────────────────────────────

/// Catches: Op::Asinh variant missing from dispatch() match block.
#[test]
fn dispatch_asinh_zero_identity() {
    let mut s = make_state_with_x("0");
    dispatch(&mut s, Op::Asinh).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
}

/// Catches: Op::Asinh dispatch not reaching op_asinh.
/// Source: HP 00041-90034 p.45, ex.7 — asinh(1) = 0.881373587
/// Free42 v3.0.5: 0.8813735870 — agrees with OM
#[test]
fn dispatch_asinh_reference_value() {
    let mut s = make_state_with_x("1");
    dispatch(&mut s, Op::Asinh).unwrap();
    assert_relative_eq!(get_x(&s), 0.881_373_587_0, max_relative = 1e-7);
}

/// Catches: Op::Asinh LASTX not saved.
#[test]
fn dispatch_asinh_saves_lastx() {
    let mut s = make_state_with_x("1");
    let orig = s.stack.x.clone();
    dispatch(&mut s, Op::Asinh).unwrap();
    assert_eq!(s.stack.lastx, orig, "Op::Asinh must save X to LASTX");
}

/// Catches: Op::Asinh not enabling stack lift.
#[test]
fn dispatch_asinh_enables_lift() {
    let mut s = make_state_with_x("0");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Asinh).unwrap();
    assert!(s.stack.lift_enabled, "Op::Asinh must enable stack lift");
}

/// Catches: Op::Asinh incorrectly returning Domain error (no domain restriction).
#[test]
fn dispatch_asinh_large_argument_no_error() {
    // asinh(500) ≈ 6.9078 — no domain restriction, should succeed
    let mut s = make_state_with_x("500");
    let result = dispatch(&mut s, Op::Asinh);
    assert!(result.is_ok(), "Op::Asinh must accept all real arguments");
}

// ── Op::Acosh — integration tests via dispatch() ────────────────────────────

/// Catches: Op::Acosh variant missing from dispatch() match block.
/// Source: HP 00041-90034 p.45 — acosh(1) = 0
/// Free42 v3.0.5: 0 — agrees with OM
#[test]
fn dispatch_acosh_one_is_zero() {
    let mut s = make_state_with_x("1");
    dispatch(&mut s, Op::Acosh).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
}

/// Catches: Op::Acosh dispatch not reaching op_acosh.
/// Source: HP 00041-90034 p.45, ex.9 — acosh(2) = 1.316957897
/// Free42 v3.0.5: 1.3169578970 — agrees with OM
#[test]
fn dispatch_acosh_reference_value() {
    let mut s = make_state_with_x("2");
    dispatch(&mut s, Op::Acosh).unwrap();
    assert_relative_eq!(get_x(&s), 1.316_957_897_0, max_relative = 1e-7);
}

/// Catches: Op::Acosh domain guard missing for X < 1.
/// Source: HP 00041-90034 p.45 — acosh(0.5) = Domain error
/// Free42 v3.0.5: returns error — agrees with OM domain restriction
#[test]
fn dispatch_acosh_below_one_returns_domain() {
    let mut s = make_state_with_x("0.5");
    let result = dispatch(&mut s, Op::Acosh);
    assert_eq!(result, Err(hp41_core::HpError::Domain));
}

/// Catches: Op::Acosh LASTX not saved.
#[test]
fn dispatch_acosh_saves_lastx() {
    let mut s = make_state_with_x("2");
    let orig = s.stack.x.clone();
    dispatch(&mut s, Op::Acosh).unwrap();
    assert_eq!(s.stack.lastx, orig, "Op::Acosh must save X to LASTX");
}

/// Catches: Op::Acosh not enabling stack lift.
#[test]
fn dispatch_acosh_enables_lift() {
    let mut s = make_state_with_x("2");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Acosh).unwrap();
    assert!(s.stack.lift_enabled, "Op::Acosh must enable stack lift");
}

// ── Op::Atanh — integration tests via dispatch() ────────────────────────────

/// Catches: Op::Atanh variant missing from dispatch() match block.
#[test]
fn dispatch_atanh_zero_identity() {
    let mut s = make_state_with_x("0");
    dispatch(&mut s, Op::Atanh).unwrap();
    assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
}

/// Catches: Op::Atanh dispatch not reaching op_atanh.
/// Source: HP 00041-90034 p.45, ex.11 — atanh(0.5) = 0.549306144
/// Free42 v3.0.5: 0.5493061443 — agrees with OM
#[test]
fn dispatch_atanh_reference_value() {
    let mut s = make_state_with_x("0.5");
    dispatch(&mut s, Op::Atanh).unwrap();
    assert_relative_eq!(get_x(&s), 0.549_306_144_3, max_relative = 1e-7);
}

/// Catches: Op::Atanh domain guard missing for X = 1.
/// Source: HP 00041-90034 p.45 — atanh(1) = Domain error
/// Free42 v3.0.5: returns error — agrees with OM domain restriction
#[test]
fn dispatch_atanh_one_returns_domain() {
    let mut s = make_state_with_x("1");
    let result = dispatch(&mut s, Op::Atanh);
    assert_eq!(result, Err(hp41_core::HpError::Domain));
}

/// Catches: Op::Atanh LASTX not saved.
#[test]
fn dispatch_atanh_saves_lastx() {
    let mut s = make_state_with_x("0.5");
    let orig = s.stack.x.clone();
    dispatch(&mut s, Op::Atanh).unwrap();
    assert_eq!(s.stack.lastx, orig, "Op::Atanh must save X to LASTX");
}

/// Catches: Op::Atanh not enabling stack lift.
#[test]
fn dispatch_atanh_enables_lift() {
    let mut s = make_state_with_x("0.5");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Atanh).unwrap();
    assert!(s.stack.lift_enabled, "Op::Atanh must enable stack lift");
}
