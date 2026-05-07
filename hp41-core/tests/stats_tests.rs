//! Integration tests for SCI-01: statistics operations.
//! Plan 02: real test bodies replacing Wave 0 stubs.

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    state.stack.lift_enabled = true;
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

/// Helper: push Y then X so the stack has the intended values.
fn push_xy(state: &mut CalcState, y: &str, x: &str) {
    push_dec(state, y);
    push_dec(state, x);
}

// ── Σ+ tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_sigma_plus_first_point_count_is_1() {
    // Σ+ with X=3, Y=5 → X = 1 (count n)
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    let expected = Decimal::from(1);
    assert_eq!(s.stack.x.inner(), expected, "count must be 1 after first Σ+");
}

#[test]
fn test_sigma_plus_accumulates_into_r02() {
    // Σ+ with X=3, Y=5 → R02 (Σx) = 3
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    let expected = Decimal::from_str("3").unwrap();
    assert_eq!(s.regs[2].inner(), expected, "R02 (Σx) must be 3");
}

#[test]
fn test_sigma_plus_accumulates_into_r03_n() {
    // Σ+ twice → R03 (n) = 2, X = 2
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "2", "7");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::from(2), "R03 (n) must be 2 after two Σ+");
    assert_eq!(s.stack.x.inner(), Decimal::from(2), "X must be count n=2");
}

#[test]
fn test_sigma_minus_decrements_count() {
    // Σ+ then Σ− → n back to 0
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    // Restore X=3, Y=5 for the reverse op
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaMinus).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::ZERO, "R03 (n) must be 0 after Σ−");
}

#[test]
fn test_sigma_plus_enables_lift() {
    // After Σ+, lift_enabled must be true
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    assert!(s.stack.lift_enabled, "Σ+ must set lift_enabled = true");
}

// ── MEAN tests ────────────────────────────────────────────────────────────────

#[test]
fn test_mean_empty_returns_invalid_op() {
    // MEAN with n=0 → HpError::InvalidOp
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::Mean), Err(HpError::InvalidOp));
}

#[test]
fn test_mean_two_points() {
    // After (X=3,Y=5) and (X=7,Y=2): x̄ = 5.0, ȳ = 3.5
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "2", "7");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    dispatch(&mut s, Op::Mean).unwrap();
    let x_mean = Decimal::from_str("5").unwrap();
    let y_mean = Decimal::from_str("3.5").unwrap();
    assert_eq!(s.stack.x.inner(), x_mean, "X must be x̄=5.0");
    assert_eq!(s.stack.y.inner(), y_mean, "Y must be ȳ=3.5");
}

// ── SDEV tests ────────────────────────────────────────────────────────────────

#[test]
fn test_sdev_n_less_than_2_returns_invalid_op() {
    // SDEV with n < 2 → HpError::InvalidOp
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::Sdev), Err(HpError::InvalidOp));
}

#[test]
fn test_sdev_n_equal_1_returns_invalid_op() {
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    assert_eq!(dispatch(&mut s, Op::Sdev), Err(HpError::InvalidOp));
}

#[test]
fn test_sdev_two_points_x() {
    // Two points: (X=3,Y=5) and (X=7,Y=2)
    // σx = sqrt((2·(9+49) − (3+7)²) / (2·1)) = sqrt((116 − 100) / 2) = sqrt(8) = 2√2 ≈ 2.828427125
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "2", "7");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    dispatch(&mut s, Op::Sdev).unwrap();
    // σx = sqrt(8) = 2√2 ≈ 2.828427125 (rounded to 10 sig digits)
    let expected_sx = Decimal::from_str("2.828427125").unwrap();
    assert_eq!(s.stack.x.inner(), expected_sx, "X must be σx=2.828427125");
}

// ── L.R. tests ────────────────────────────────────────────────────────────────

#[test]
fn test_lr_empty_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::LR), Err(HpError::InvalidOp));
}

#[test]
fn test_lr_slope_in_y_intercept_in_x() {
    // Three points for a clean line: (1,2), (2,4), (3,6) → slope m=2, intercept b=0
    let mut s = CalcState::new();
    push_xy(&mut s, "2", "1");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "4", "2");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "6", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    dispatch(&mut s, Op::LR).unwrap();
    let slope = Decimal::from(2);
    let intercept = Decimal::ZERO;
    assert_eq!(s.stack.y.inner(), slope, "Y must be slope m=2 (D-05)");
    assert_eq!(s.stack.x.inner(), intercept, "X must be intercept b=0 (D-05)");
}

#[test]
fn test_lr_all_same_x_returns_invalid_op() {
    // All x-values identical → denominator = 0 → HpError::InvalidOp
    let mut s = CalcState::new();
    push_xy(&mut s, "1", "5");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "2", "5");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    assert_eq!(dispatch(&mut s, Op::LR), Err(HpError::InvalidOp));
}

// ── YHAT tests ────────────────────────────────────────────────────────────────

#[test]
fn test_yhat_empty_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::Yhat), Err(HpError::InvalidOp));
}

#[test]
fn test_yhat_predicts_correctly() {
    // Line: slope=2, intercept=0. YHAT(x=4) = 2*4+0 = 8
    let mut s = CalcState::new();
    push_xy(&mut s, "2", "1");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "4", "2");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "6", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    // Push x=4 for YHAT prediction
    push_dec(&mut s, "4");
    dispatch(&mut s, Op::Yhat).unwrap();
    let expected = Decimal::from(8);
    assert_eq!(s.stack.x.inner(), expected, "YHAT(4) must be 8 on y=2x line");
}

// ── CORR tests ────────────────────────────────────────────────────────────────

#[test]
fn test_corr_empty_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::Corr), Err(HpError::InvalidOp));
}

#[test]
fn test_corr_perfect_line_is_one() {
    // Perfect positive linear relationship → r = 1
    let mut s = CalcState::new();
    push_xy(&mut s, "2", "1");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "4", "2");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "6", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    dispatch(&mut s, Op::Corr).unwrap();
    let expected = Decimal::from(1);
    assert_eq!(s.stack.x.inner(), expected, "CORR must be 1 for perfect positive line");
}

#[test]
fn test_corr_zero_denominator_returns_error() {
    // All x same, y different → CORR denominator contains 0 → error
    let mut s = CalcState::new();
    push_xy(&mut s, "1", "5");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    push_xy(&mut s, "2", "5");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    let result = dispatch(&mut s, Op::Corr);
    assert!(result.is_err(), "CORR must error when denominator is zero");
}

// ── CLΣSTAT tests ─────────────────────────────────────────────────────────────

#[test]
fn test_cl_sigma_stat_zeros_all_registers() {
    // Accumulate some data, then clear, verify R01–R06 are zero
    let mut s = CalcState::new();
    push_xy(&mut s, "5", "3");
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    dispatch(&mut s, Op::ClSigmaStat).unwrap();
    for i in 1..=6 {
        assert!(s.regs[i].is_zero(), "R{i:02} must be zero after CLΣSTAT");
    }
}

#[test]
fn test_cl_sigma_stat_does_not_error() {
    let mut s = CalcState::new();
    assert!(dispatch(&mut s, Op::ClSigmaStat).is_ok());
}
