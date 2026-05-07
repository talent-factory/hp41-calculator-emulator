//! Integration tests for SCI-02: HMS/H time-angle conversion operations.
//! Plan 02: real test bodies replacing Wave 0 stubs.

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

// ── HMS→ (op_hms_to_h) tests ─────────────────────────────────────────────────

#[test]
fn test_hms_to_h_canonical_round_trip() {
    // ROADMAP Phase 6 success criterion: 1.3045 → 1.5125
    // 1h 30m 45s = 1 + 30/60 + 45/3600 = 1.5125 exactly
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected = Decimal::from_str("1.5125").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "1.3045 HMS must convert to 1.5125 decimal hours");
}

#[test]
fn test_hms_to_h_whole_hours() {
    // 2.0000 = 2h 0m 0s → 2.0 decimal hours
    let mut s = CalcState::new();
    push_dec(&mut s, "2.0000");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected = Decimal::from(2);
    assert_eq!(s.stack.x.inner(), expected, "2.0000 HMS must convert to 2.0 decimal hours");
}

#[test]
fn test_hms_to_h_invalid_minutes_60() {
    // 1.6000 = 60 minutes → HpError::InvalidInput (D-06)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.6000");
    assert_eq!(dispatch(&mut s, Op::HmsToH), Err(HpError::InvalidInput));
}

#[test]
fn test_hms_to_h_invalid_seconds_60() {
    // 1.0060 = 60 seconds → HpError::InvalidInput (D-06)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.0060");
    assert_eq!(dispatch(&mut s, Op::HmsToH), Err(HpError::InvalidInput));
}

#[test]
fn test_hms_to_h_negative_value() {
    // -1.3045 → -1.5125 (D-08: sign applies to whole value)
    let mut s = CalcState::new();
    push_dec(&mut s, "-1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected = Decimal::from_str("-1.5125").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "-1.3045 HMS must convert to -1.5125");
}

// ── →HMS (op_h_to_hms) tests ─────────────────────────────────────────────────

#[test]
fn test_h_to_hms_canonical_round_trip() {
    // 1.5125 → 1.3045 (reverse of the canonical round-trip)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.5125");
    dispatch(&mut s, Op::HToHms).unwrap();
    let expected = Decimal::from_str("1.3045").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "1.5125 decimal hours must convert to 1.3045 HMS");
}

#[test]
fn test_h_to_hms_negative_value() {
    // -1.5125 → -1.3045 (D-08)
    let mut s = CalcState::new();
    push_dec(&mut s, "-1.5125");
    dispatch(&mut s, Op::HToHms).unwrap();
    let expected = Decimal::from_str("-1.3045").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "-1.5125 must convert to -1.3045 HMS");
}

// ── HMS+ (op_hms_add) tests ──────────────────────────────────────────────────

#[test]
fn test_hms_add_with_base60_carry() {
    // Y=1.4500 (1h 45m), X=0.2000 (0h 20m) → 2.0500 (2h 5m) with carry
    // 1h45m + 0h20m = 2h5m (base-60 carry from minutes)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.4500");
    push_dec(&mut s, "0.2000");
    dispatch(&mut s, Op::HmsAdd).unwrap();
    let expected = Decimal::from_str("2.0500").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "1.4500 + 0.2000 HMS must be 2.0500");
}

#[test]
fn test_hms_add_no_carry() {
    // Y=1.1000 (1h 10m), X=0.2000 (0h 20m) → 1.3000 (1h 30m), no carry
    let mut s = CalcState::new();
    push_dec(&mut s, "1.1000");
    push_dec(&mut s, "0.2000");
    dispatch(&mut s, Op::HmsAdd).unwrap();
    let expected = Decimal::from_str("1.3000").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "1.1000 + 0.2000 HMS must be 1.3000");
}

// ── HMS− (op_hms_sub) tests ──────────────────────────────────────────────────

#[test]
fn test_hms_sub_with_borrow() {
    // Y=2.0500 (2h 5m), X=1.4500 (1h 45m) → 0.2000 (0h 20m) with borrow
    let mut s = CalcState::new();
    push_dec(&mut s, "2.0500");
    push_dec(&mut s, "1.4500");
    dispatch(&mut s, Op::HmsSub).unwrap();
    let expected = Decimal::from_str("0.2000").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "2.0500 - 1.4500 HMS must be 0.2000");
}

#[test]
fn test_hms_sub_invalid_seconds() {
    // Input with 60 seconds → HpError::InvalidInput (D-06)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.0060");
    push_dec(&mut s, "0.3000");
    assert_eq!(dispatch(&mut s, Op::HmsSub), Err(HpError::InvalidInput));
}
