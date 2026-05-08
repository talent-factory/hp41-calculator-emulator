//! Integration tests for MATH-01: unary math ops, binary YPow, 10-digit accuracy,
//! LASTX save behavior, and stack-lift enable for all math ops.

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

// ── 1/x ──────────────────────────────────────────────────────────────────

#[test]
fn test_recip_of_4_is_0_25() {
    let mut s = CalcState::new();
    push(&mut s, 4);
    dispatch(&mut s, Op::Recip).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from_str("0.25").unwrap());
}

#[test]
fn test_recip_of_zero_returns_divide_by_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0);
    assert_eq!(dispatch(&mut s, Op::Recip), Err(HpError::DivideByZero));
}

// ── √x ───────────────────────────────────────────────────────────────────

#[test]
fn test_sqrt_of_4_is_2() {
    let mut s = CalcState::new();
    push(&mut s, 4);
    dispatch(&mut s, Op::Sqrt).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
}

#[test]
fn test_sqrt_of_negative_returns_domain() {
    let mut s = CalcState::new();
    push(&mut s, -1);
    assert_eq!(dispatch(&mut s, Op::Sqrt), Err(HpError::Domain));
}

// ── x² ───────────────────────────────────────────────────────────────────

#[test]
fn test_sq_of_5_is_25() {
    let mut s = CalcState::new();
    push(&mut s, 5);
    dispatch(&mut s, Op::Sq).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(25));
}

// ── LN ───────────────────────────────────────────────────────────────────

#[test]
fn test_ln_of_1_is_0() {
    let mut s = CalcState::new();
    push(&mut s, 1);
    dispatch(&mut s, Op::Ln).unwrap();
    assert!(s.stack.x.is_zero(), "LN(1) must be 0");
}

#[test]
fn test_ln_2_accuracy_10_digits() {
    // LN(2) = 0.6931471806 (10 significant digits, HP-41 hardware value)
    let mut s = CalcState::new();
    push(&mut s, 2);
    dispatch(&mut s, Op::Ln).unwrap();
    let expected = Decimal::from_str("0.6931471806").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "LN(2) must equal 0.6931471806 at 10 sig digits"
    );
}

#[test]
fn test_ln_of_zero_returns_domain() {
    let mut s = CalcState::new();
    push(&mut s, 0);
    assert_eq!(dispatch(&mut s, Op::Ln), Err(HpError::Domain));
}

#[test]
fn test_ln_of_negative_returns_domain() {
    let mut s = CalcState::new();
    push(&mut s, -5);
    assert_eq!(dispatch(&mut s, Op::Ln), Err(HpError::Domain));
}

// ── LOG ──────────────────────────────────────────────────────────────────

#[test]
fn test_log_of_100_is_2() {
    let mut s = CalcState::new();
    push(&mut s, 100);
    dispatch(&mut s, Op::Log).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
}

// ── e^x ──────────────────────────────────────────────────────────────────

#[test]
fn test_exp_of_0_is_1() {
    let mut s = CalcState::new();
    push(&mut s, 0);
    dispatch(&mut s, Op::Exp).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(1));
}

// ── 10^x ─────────────────────────────────────────────────────────────────

#[test]
fn test_tenpow_of_2_is_100() {
    let mut s = CalcState::new();
    push(&mut s, 2);
    dispatch(&mut s, Op::TenPow).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(100));
}

// ── Y^X ──────────────────────────────────────────────────────────────────

#[test]
fn test_ypow_2_to_10_is_1024() {
    let mut s = CalcState::new();
    push(&mut s, 2); // Y = 2
    s.stack.lift_enabled = true;
    push(&mut s, 10); // X = 10
    dispatch(&mut s, Op::YPow).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(1024));
}

#[test]
fn test_ypow_2_to_0_5_is_sqrt_2() {
    // 2^0.5 = √2 = 1.414213562 (10 sig digits)
    let mut s = CalcState::new();
    push(&mut s, 2);
    s.stack.lift_enabled = true;
    push_dec(&mut s, "0.5");
    dispatch(&mut s, Op::YPow).unwrap();
    let expected = Decimal::from_str("1.414213562").unwrap();
    assert_eq!(s.stack.x.inner(), expected);
}

// ── LASTX save for all unary math ops ────────────────────────────────────

#[test]
fn test_unary_ops_save_lastx() {
    // For every unary op, X before the op must equal LASTX after the op
    let ops = vec![
        Op::Recip,
        Op::Sqrt,
        Op::Sq,
        Op::Ln,
        Op::Log,
        Op::Exp,
        Op::TenPow,
    ];
    for op in ops {
        let mut s = CalcState::new();
        push(&mut s, 2); // X = 2 (valid domain for all these ops)
        let x_before = s.stack.x.inner();
        dispatch(&mut s, op.clone()).unwrap();
        assert_eq!(
            s.stack.lastx.inner(),
            x_before,
            "LASTX must be saved for {op:?}"
        );
    }
}

#[test]
fn test_ypow_saves_lastx() {
    // YPow is binary — saves X (the exponent) to LASTX
    let mut s = CalcState::new();
    push(&mut s, 2);
    s.stack.lift_enabled = true;
    push(&mut s, 3); // X = 3 (exponent)
    let x_before = s.stack.x.inner();
    dispatch(&mut s, Op::YPow).unwrap();
    assert_eq!(
        s.stack.lastx.inner(),
        x_before,
        "YPow must save X (exponent) to LASTX"
    );
}

// ── Stack-lift enable for all math ops ───────────────────────────────────

#[test]
fn test_math_ops_enable_lift() {
    let ops = vec![
        Op::Recip,
        Op::Sqrt,
        Op::Sq,
        Op::Ln,
        Op::Log,
        Op::Exp,
        Op::TenPow,
    ];
    for op in ops {
        let mut s = CalcState::new();
        push(&mut s, 2);
        s.stack.lift_enabled = false; // force disable before op
        dispatch(&mut s, op.clone()).unwrap();
        assert!(s.stack.lift_enabled, "{op:?} must enable stack lift");
    }
}

// ── Unary ops do NOT change Y, Z, T ──────────────────────────────────────

#[test]
fn test_unary_op_does_not_modify_y_z_t() {
    let mut s = CalcState::new();
    push(&mut s, 3);
    s.stack.lift_enabled = true;
    push(&mut s, 2);
    s.stack.lift_enabled = true;
    push(&mut s, 1); // X=1, Y=2, Z=3, T=0
    let y_before = s.stack.y.inner();
    let z_before = s.stack.z.inner();
    dispatch(&mut s, Op::Sq).unwrap(); // 1² = 1, Y/Z/T unchanged
    assert_eq!(s.stack.y.inner(), y_before, "Sq must not modify Y");
    assert_eq!(s.stack.z.inner(), z_before, "Sq must not modify Z");
}
