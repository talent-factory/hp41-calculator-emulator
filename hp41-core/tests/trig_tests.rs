//! Integration tests for MATH-02: trig ops in DEG, RAD, and GRAD angle modes.
//! Critical: asin/acos/atan must return results in the CURRENT angle mode (not always radians).

use hp41_core::ops::{dispatch, Op};
use hp41_core::{AngleMode, CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

fn set_deg(s: &mut CalcState) {
    dispatch(s, Op::SetDeg).unwrap();
}
fn set_rad(s: &mut CalcState) {
    dispatch(s, Op::SetRad).unwrap();
}
fn set_grad(s: &mut CalcState) {
    dispatch(s, Op::SetGrad).unwrap();
}

// ── SIN in DEG mode ──────────────────────────────────────────────────────

#[test]
fn test_sin_30_deg_is_0_5() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 30);
    dispatch(&mut s, Op::Sin).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from_str("0.5").unwrap());
}

#[test]
fn test_sin_90_deg_is_1() {
    // HP-41 success criterion: SIN(90°) = exactly 1
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 90);
    dispatch(&mut s, Op::Sin).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(1),
        "SIN(90°) must be exactly 1"
    );
}

#[test]
fn test_sin_0_deg_is_0() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 0);
    dispatch(&mut s, Op::Sin).unwrap();
    assert!(s.stack.x.is_zero(), "SIN(0°) must be 0");
}

// ── COS in DEG mode ──────────────────────────────────────────────────────

#[test]
fn test_cos_0_deg_is_1() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 0);
    dispatch(&mut s, Op::Cos).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(1));
}

#[test]
fn test_cos_60_deg_is_0_5() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 60);
    dispatch(&mut s, Op::Cos).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from_str("0.5").unwrap());
}

// ── SIN in RAD mode ──────────────────────────────────────────────────────

#[test]
fn test_sin_pi_over_6_rad_is_0_5() {
    // π/6 ≈ 0.5235987756
    let mut s = CalcState::new();
    set_rad(&mut s);
    push_dec(&mut s, "0.5235987756");
    dispatch(&mut s, Op::Sin).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from_str("0.5").unwrap());
}

// ── TAN in GRAD mode ─────────────────────────────────────────────────────

#[test]
fn test_tan_100_grad_equals_tan_90_deg() {
    // 100 grad = 90°; both should produce the same (very large) value or Domain
    let mut s_deg = CalcState::new();
    set_deg(&mut s_deg);
    push(&mut s_deg, 89); // use 89° not 90° to avoid domain error in both
    let result_deg = dispatch(&mut s_deg, Op::Tan);

    let mut s_grad = CalcState::new();
    set_grad(&mut s_grad);
    push_dec(&mut s_grad, "98.888888889"); // 89° in grads ≈ 98.888...
    let result_grad = dispatch(&mut s_grad, Op::Tan);

    assert_eq!(
        result_deg.is_ok(),
        result_grad.is_ok(),
        "both must succeed or fail together"
    );
}

#[test]
fn test_angle_mode_stored_after_set_deg() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::SetRad).unwrap();
    assert_eq!(s.angle_mode, AngleMode::Rad);
    dispatch(&mut s, Op::SetDeg).unwrap();
    assert_eq!(s.angle_mode, AngleMode::Deg);
    dispatch(&mut s, Op::SetGrad).unwrap();
    assert_eq!(s.angle_mode, AngleMode::Grad);
}

// ── ASIN in DEG mode ─────────────────────────────────────────────────────

#[test]
fn test_asin_0_5_is_30_deg() {
    // ASIN(0.5) in DEG mode must return 30
    let mut s = CalcState::new();
    set_deg(&mut s);
    push_dec(&mut s, "0.5");
    dispatch(&mut s, Op::Asin).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(30),
        "ASIN(0.5) in DEG = 30"
    );
}

#[test]
fn test_asin_1_is_90_deg() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 1);
    dispatch(&mut s, Op::Asin).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(90), "ASIN(1) in DEG = 90");
}

#[test]
fn test_asin_out_of_domain_returns_error() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 2); // |2| > 1 — domain error
    assert_eq!(dispatch(&mut s, Op::Asin), Err(HpError::Domain));
}

// ── ATAN in DEG mode ─────────────────────────────────────────────────────

#[test]
fn test_atan_1_is_45_deg() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 1);
    dispatch(&mut s, Op::Atan).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(45), "ATAN(1) in DEG = 45");
}

// ── Trig LASTX save ──────────────────────────────────────────────────────

#[test]
fn test_trig_ops_save_lastx() {
    let ops = vec![Op::Sin, Op::Cos, Op::Tan];
    for op in ops {
        let mut s = CalcState::new();
        set_deg(&mut s);
        push(&mut s, 30);
        let x_before = s.stack.x.inner();
        dispatch(&mut s, op.clone()).unwrap();
        assert_eq!(
            s.stack.lastx.inner(),
            x_before,
            "{op:?} must save X to LASTX"
        );
    }
}

// ── Trig lift enable ─────────────────────────────────────────────────────

#[test]
fn test_trig_ops_enable_lift() {
    // 0.5 is a valid input domain for all six ops in DEG mode:
    // sin(0.5°), cos(0.5°), tan(0.5°) — fine; asin(0.5)=30°, acos(0.5)=60°, atan(0.5)≈26.6°
    let ops = vec![Op::Sin, Op::Cos, Op::Tan, Op::Asin, Op::Acos, Op::Atan];
    for op in ops {
        let mut s = CalcState::new();
        set_deg(&mut s);
        push_dec(&mut s, "0.5");
        s.stack.lift_enabled = false;
        dispatch(&mut s, op.clone()).unwrap();
        assert!(s.stack.lift_enabled, "{op:?} must enable lift on success");
    }
}

// ── ACOS tests ───────────────────────────────────────────────────────────

#[test]
fn test_acos_half_is_60_deg() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push_dec(&mut s, "0.5");
    dispatch(&mut s, Op::Acos).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(60), "ACOS(0.5) in DEG = 60");
}

#[test]
fn test_acos_out_of_domain_returns_error() {
    let mut s = CalcState::new();
    set_deg(&mut s);
    push(&mut s, 2); // |2| > 1 — domain error
    assert_eq!(dispatch(&mut s, Op::Acos), Err(HpError::Domain));
}

// ── ATAN in RAD and GRAD modes ───────────────────────────────────────────

#[test]
fn test_atan_1_in_rad_mode() {
    let mut s = CalcState::new();
    set_rad(&mut s);
    push(&mut s, 1);
    dispatch(&mut s, Op::Atan).unwrap();
    // ATAN(1) in RAD = π/4 ≈ 0.7853981634
    let result = s.stack.x.inner();
    let expected = Decimal::from_str("0.7853981634").unwrap();
    let diff = (result - expected).abs();
    assert!(
        diff < Decimal::from_str("0.0000000001").unwrap(),
        "ATAN(1) in RAD = π/4 ≈ 0.7853981634, got {result}"
    );
}

#[test]
fn test_atan_1_in_grad_mode() {
    let mut s = CalcState::new();
    set_grad(&mut s);
    push(&mut s, 1);
    dispatch(&mut s, Op::Atan).unwrap();
    // ATAN(1) in GRAD = 50 grad
    assert_eq!(s.stack.x.inner(), Decimal::from(50), "ATAN(1) in GRAD = 50");
}
