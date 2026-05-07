//! Integration test stubs for SCI-02: HMS/H conversion operations.
//! Wave 0: stubs compile but FAIL — implementations created in Plan 02.
//! Full test bodies added in Plan 03.

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

// ── Wave 0 stubs ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_hms_to_h_stub() {
    // ROADMAP Phase 6 success criterion: 1.3045 → 1.5125
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045");
    let result = dispatch(&mut s, Op::HmsToH);
    assert!(result.is_ok(), "HmsToH must not error on valid H.MMSS");
    let expected = Decimal::from_str("1.5125").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "1.3045 must convert to 1.5125");
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_h_to_hms_stub() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.5125");
    let result = dispatch(&mut s, Op::HToHms);
    assert!(result.is_ok(), "HToHms must not error on valid decimal hours");
    let expected = Decimal::from_str("1.3045").unwrap();
    assert_eq!(s.stack.x.inner(), expected, "1.5125 must convert to 1.3045");
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_hms_to_h_invalid_minutes_stub() {
    // 1.6000 = 60 minutes → HpError::InvalidInput (D-06)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.6000");
    assert_eq!(dispatch(&mut s, Op::HmsToH), Err(HpError::InvalidInput));
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_hms_add_stub() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.4500");
    push_dec(&mut s, "0.2000");
    let result = dispatch(&mut s, Op::HmsAdd);
    assert!(result.is_ok(), "HmsAdd must not error on valid H.MMSS inputs");
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_hms_sub_stub() {
    let mut s = CalcState::new();
    push_dec(&mut s, "2.0500");
    push_dec(&mut s, "1.4500");
    let result = dispatch(&mut s, Op::HmsSub);
    assert!(result.is_ok(), "HmsSub must not error on valid H.MMSS inputs");
}
