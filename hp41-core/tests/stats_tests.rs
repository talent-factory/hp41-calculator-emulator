//! Integration tests for SCI-01: statistics operations Σ+, Σ−, MEAN, SDEV, L.R., YHAT, CORR, CLΣSTAT.
//!
//! All tests dispatch through hp41_core::ops::dispatch() — same path as interactive use.
//! Σ register layout: R01=Σx², R02=Σx, R03=n, R04=Σy², R05=Σy, R06=Σxy (D-03).

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push(state: &mut CalcState, n: i32) {
    state.stack.lift_enabled = true;
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

/// Accumulate a (y, x) data point via Σ+.
/// push_dec(y) then push_dec(x) with lift enabled so Y=y, X=x.
fn add_point(state: &mut CalcState, y: i32, x: i32) {
    push(state, y);
    push(state, x);
    dispatch(state, Op::SigmaPlus).unwrap();
}

// ── Σ+ tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_sigma_plus_count_increments() {
    let mut s = CalcState::new();
    add_point(&mut s, 5, 3); // X=3, Y=5
    assert_eq!(s.regs[3].inner(), Decimal::from(1), "n must be 1 after first Σ+");
    assert_eq!(s.stack.x.inner(), Decimal::from(1), "X must hold n=1 after Σ+");
}

#[test]
fn test_sigma_plus_accumulates_sum_x() {
    let mut s = CalcState::new();
    add_point(&mut s, 5, 3); // X=3, Y=5
    add_point(&mut s, 2, 7); // X=7, Y=2

    assert_eq!(s.regs[3].inner(), Decimal::from(2), "n must be 2 after two Σ+");
    assert_eq!(s.regs[2].inner(), Decimal::from(10), "Σx must be 3+7=10");
    assert_eq!(s.regs[5].inner(), Decimal::from(7), "Σy must be 5+2=7");
    assert_eq!(s.stack.x.inner(), Decimal::from(2), "X must hold n=2 after second Σ+");
}

#[test]
fn test_sigma_plus_does_not_save_lastx() {
    let mut s = CalcState::new();
    // Establish a known LASTX via a unary op
    push(&mut s, 99);
    dispatch(&mut s, Op::Sqrt).unwrap(); // LASTX = 99 after sqrt
    let lastx_before = s.stack.lastx.inner();

    add_point(&mut s, 5, 3);
    // Σ+ must NOT overwrite LASTX
    assert_eq!(
        s.stack.lastx.inner(),
        lastx_before,
        "Σ+ must not save LASTX"
    );
}

#[test]
fn test_sigma_plus_enables_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    add_point(&mut s, 5, 3);
    assert!(s.stack.lift_enabled, "Σ+ must enable lift after pushing n");
}

// ── Σ− tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_sigma_minus_removes_data_point() {
    let mut s = CalcState::new();
    add_point(&mut s, 5, 3);

    // Remove the same point
    push(  &mut s, 5); // Y=5
    push(  &mut s, 3); // X=3
    dispatch(&mut s, Op::SigmaMinus).unwrap();

    assert_eq!(s.regs[3].inner(), Decimal::ZERO, "n must be 0 after Σ+/Σ−");
    assert_eq!(s.regs[2].inner(), Decimal::ZERO, "Σx must be 0 after Σ+/Σ−");
    assert_eq!(s.stack.x.inner(), Decimal::ZERO, "X must hold n=0 after Σ−");
}

// ── MEAN tests ────────────────────────────────────────────────────────────────

#[test]
fn test_mean_with_no_data_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::Mean), Err(HpError::InvalidOp));
}

#[test]
fn test_mean_two_points() {
    // Data: (X=3,Y=5) and (X=7,Y=2)
    // x̄ = (3+7)/2 = 5.0,  ȳ = (5+2)/2 = 3.5
    let mut s = CalcState::new();
    add_point(&mut s, 5, 3);
    add_point(&mut s, 2, 7);

    dispatch(&mut s, Op::Mean).unwrap();

    let expected_x_mean = Decimal::from_str("5").unwrap();
    let expected_y_mean = Decimal::from_str("3.5").unwrap();
    assert_eq!(s.stack.x.inner(), expected_x_mean, "X must be x̄=5.0");
    assert_eq!(s.stack.y.inner(), expected_y_mean, "Y must be ȳ=3.5");
}

// ── SDEV tests ────────────────────────────────────────────────────────────────

#[test]
fn test_sdev_with_one_point_returns_invalid_op() {
    let mut s = CalcState::new();
    add_point(&mut s, 5, 3);
    assert_eq!(dispatch(&mut s, Op::Sdev), Err(HpError::InvalidOp));
}

#[test]
fn test_sdev_sample_two_points() {
    // Data: (X=2,Y=4) and (X=4,Y=8) — Y = 2X relationship
    // n=2, Σx=6, Σx²=20, Σy=12, Σy²=80
    // σx = sqrt((2·20 − 36) / (2·1)) = sqrt(2) ≈ 1.4142135623...
    // σy = sqrt((2·80 − 144) / (2·1)) = sqrt(8) ≈ 2.8284271247...
    // Verify: σx > 0, σy > 0, and σy > σx (Y variance is larger than X variance)
    let mut s = CalcState::new();
    add_point(&mut s, 4, 2);
    add_point(&mut s, 8, 4);

    dispatch(&mut s, Op::Sdev).unwrap();
    let sigma_x = s.stack.x.inner();
    let sigma_y = s.stack.y.inner();
    // σx must be positive (sqrt(2) ≈ 1.414)
    assert!(sigma_x > Decimal::ZERO, "σx must be positive");
    // σy must be positive (sqrt(8) ≈ 2.828)
    assert!(sigma_y > Decimal::ZERO, "σy must be positive");
    // σy must be exactly 2 · σx: verify within 1 ULP at 10 significant digits
    // Expected: σx = 1.414213562, σy = 2.828427125 (rounding of sqrt(8) independently)
    let expected_sx = Decimal::from_str("1.414213562").unwrap();
    let expected_sy = Decimal::from_str("2.828427125").unwrap();
    assert_eq!(sigma_x, expected_sx, "σx must be sqrt(2) rounded to 10 sig digits");
    assert_eq!(sigma_y, expected_sy, "σy must be sqrt(8) rounded to 10 sig digits");
}

// ── L.R. tests ────────────────────────────────────────────────────────────────

#[test]
fn test_lr_with_no_data_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::LR), Err(HpError::InvalidOp));
}

#[test]
fn test_lr_slope_in_y_intercept_in_x() {
    // Data: Y = 2X + 1 (perfect linear fit)
    // Points: (X=1,Y=3), (X=2,Y=5), (X=3,Y=7)
    // slope m = 2, intercept b = 1
    let mut s = CalcState::new();
    for (y, x) in [(3i32, 1i32), (5, 2), (7, 3)] {
        add_point(&mut s, y, x);
    }

    dispatch(&mut s, Op::LR).unwrap();

    let expected_slope = Decimal::from(2);
    let expected_intercept = Decimal::from(1);
    assert_eq!(s.stack.x.inner(), expected_intercept, "X must hold intercept b=1 (D-05)");
    assert_eq!(s.stack.y.inner(), expected_slope, "Y must hold slope m=2 (D-05)");
}

#[test]
fn test_corr_denominator_zero_returns_error() {
    // All x values identical → denominator term n·Σx²−(Σx)² = 0 → Domain or InvalidOp
    let mut s = CalcState::new();
    for y in [1i32, 2, 3] {
        add_point(&mut s, y, 5); // X always 5
    }
    let result = dispatch(&mut s, Op::Corr);
    assert!(result.is_err(), "CORR with identical x values must return an error");
}

// ── YHAT tests ────────────────────────────────────────────────────────────────

#[test]
fn test_yhat_uses_regression() {
    // Same Y=2X+1 data as above; YHAT at X=4 should give ŷ=9
    let mut s = CalcState::new();
    for (y, x) in [(3i32, 1i32), (5, 2), (7, 3)] {
        add_point(&mut s, y, x);
    }

    push(&mut s, 4); // X=4
    dispatch(&mut s, Op::Yhat).unwrap();

    assert_eq!(s.stack.x.inner(), Decimal::from(9), "ŷ must be 9 for x=4 on Y=2X+1");
}

// ── CORR tests ────────────────────────────────────────────────────────────────

#[test]
fn test_corr_perfect_positive_correlation() {
    // Perfect Y=2X+1 fit: r must be exactly 1.0
    let mut s = CalcState::new();
    for (y, x) in [(3i32, 1i32), (5, 2), (7, 3)] {
        add_point(&mut s, y, x);
    }

    dispatch(&mut s, Op::Corr).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(1), "r must be 1.0 for perfect linear data");
}

#[test]
fn test_corr_singular_returns_error() {
    // All x values identical → denominator term n·Σx²−(Σx)² = 0 → Domain or InvalidOp
    let mut s = CalcState::new();
    for y in [1i32, 2, 3] {
        add_point(&mut s, y, 5); // X always 5
    }
    let result = dispatch(&mut s, Op::Corr);
    assert!(result.is_err(), "CORR with identical x values must return an error");
}

// ── CLΣSTAT tests ─────────────────────────────────────────────────────────────

#[test]
fn test_cl_sigma_stat_zeros_r01_to_r06() {
    let mut s = CalcState::new();
    // Accumulate some data to make registers non-zero
    add_point(&mut s, 5, 3);

    dispatch(&mut s, Op::ClSigmaStat).unwrap();

    for i in 1..=6 {
        assert_eq!(
            s.regs[i].inner(),
            Decimal::ZERO,
            "R{:02} must be zero after CLΣSTAT",
            i
        );
    }
}

#[test]
fn test_cl_sigma_stat_lift_is_neutral() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::ClSigmaStat).unwrap();
    assert!(
        !s.stack.lift_enabled,
        "CLΣSTAT must be LiftEffect::Neutral (lift_enabled unchanged)"
    );
}
