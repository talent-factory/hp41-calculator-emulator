//! Integration tests for SCI-02: HMS/H conversion operations HMS→, →HMS, HMS+, HMS−.
//!
//! Canonical test: 1.3045 (1h 30m 45s) → HMS→ → 1.5125 decimal hours.
//! Verified: 1 + 30/60 + 45/3600 = 1 + 0.5 + 0.0125 = 1.5125 exactly.

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Push a decimal string onto the stack with lift enabled.
/// CRITICAL: Op::PushNum does NOT set lift_enabled itself; caller must set it first.
fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    state.stack.lift_enabled = true;
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

// ── HMS→ tests ────────────────────────────────────────────────────────────────

#[test]
fn test_hms_to_h_canonical_1_3045() {
    // ROADMAP Phase 6 success criterion: 1.3045 → 1.5125
    // 1h 30m 45s = 1 + 30/60 + 45/3600 = 1.5125 exactly
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected = Decimal::from_str("1.5125").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "1.3045 H.MMSS must convert to 1.5125 decimal hours"
    );
}

#[test]
fn test_hms_to_h_whole_hours() {
    // 2.0000 H.MMSS = 2 hours exactly
    let mut s = CalcState::new();
    push_dec(&mut s, "2.0000");
    dispatch(&mut s, Op::HmsToH).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(2),
        "2.0000 H.MMSS must be 2.0 hours"
    );
}

#[test]
fn test_hms_to_h_saves_lastx() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected_lastx = Decimal::from_str("1.3045").unwrap();
    assert_eq!(
        s.stack.lastx.inner(),
        expected_lastx,
        "HMS→ must save X into LASTX before converting (unary_result semantics)"
    );
}

// ── →HMS tests ────────────────────────────────────────────────────────────────

#[test]
fn test_h_to_hms_canonical_1_5125() {
    // ROADMAP Phase 6 success criterion: 1.5125 → 1.3045
    let mut s = CalcState::new();
    push_dec(&mut s, "1.5125");
    dispatch(&mut s, Op::HToHms).unwrap();
    let expected = Decimal::from_str("1.3045").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "1.5125 decimal must convert to 1.3045 H.MMSS"
    );
}

#[test]
fn test_h_to_hms_round_trip() {
    // Convert 1.3045 → decimal → back to H.MMSS; must recover 1.3045
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap(); // → 1.5125
    dispatch(&mut s, Op::HToHms).unwrap(); // → 1.3045
    let expected = Decimal::from_str("1.3045").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "round-trip must recover original H.MMSS"
    );
}

// ── Invalid HMS validation tests ──────────────────────────────────────────────

#[test]
fn test_hms_to_h_invalid_minutes_60_returns_invalid_input() {
    // 1.6000 = 1h 60m 00s → minutes >= 60 → HpError::InvalidInput (D-06)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.6000");
    assert_eq!(
        dispatch(&mut s, Op::HmsToH),
        Err(HpError::InvalidInput),
        "60 minutes must return HpError::InvalidInput"
    );
}

#[test]
fn test_hms_to_h_invalid_seconds_60_returns_invalid_input() {
    // 1.0060 = 1h 00m 60s → seconds >= 60 → HpError::InvalidInput (D-06)
    let mut s = CalcState::new();
    push_dec(&mut s, "1.0060");
    assert_eq!(
        dispatch(&mut s, Op::HmsToH),
        Err(HpError::InvalidInput),
        "60 seconds must return HpError::InvalidInput"
    );
}

#[test]
fn test_hms_to_h_boundary_59_valid() {
    // 0.5959 = 0h 59m 59s → valid (59 < 60); must not error
    let mut s = CalcState::new();
    push_dec(&mut s, "0.5959");
    assert!(
        dispatch(&mut s, Op::HmsToH).is_ok(),
        "59m 59s must be valid HMS"
    );
}

// ── Negative HMS tests ────────────────────────────────────────────────────────

#[test]
fn test_hms_to_h_negative_value() {
    // -1.3045 H.MMSS should convert to -1.5125 decimal hours (D-08)
    let mut s = CalcState::new();
    push_dec(&mut s, "-1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected = Decimal::from_str("-1.5125").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "negative H.MMSS must produce negative decimal hours"
    );
}

#[test]
fn test_h_to_hms_negative_value() {
    // -1.5125 decimal should convert to -1.3045 H.MMSS (D-08)
    let mut s = CalcState::new();
    push_dec(&mut s, "-1.5125");
    dispatch(&mut s, Op::HToHms).unwrap();
    let expected = Decimal::from_str("-1.3045").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "negative decimal hours must produce negative H.MMSS"
    );
}

// ── HMS+ tests ────────────────────────────────────────────────────────────────

#[test]
fn test_hms_add_with_carry() {
    // Y=1.4500 (1h 45m 00s) + X=0.2000 (0h 20m 00s) = 2.0500 (2h 05m 00s)
    // 45m + 20m = 65m = 1h 5m → carry into hours
    let mut s = CalcState::new();
    push_dec(&mut s, "1.4500"); // pushed first, becomes Y after next push
    push_dec(&mut s, "0.2000"); // X
    dispatch(&mut s, Op::HmsAdd).unwrap();
    let expected = Decimal::from_str("2.0500").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "1.4500 + 0.2000 H.MMSS must produce 2.0500 (base-60 carry)"
    );
}

#[test]
fn test_hms_add_no_carry() {
    // Y=1.1000 (1h 10m) + X=0.2000 (0h 20m) = 1.3000 (1h 30m) — no carry needed
    let mut s = CalcState::new();
    push_dec(&mut s, "1.1000");
    push_dec(&mut s, "0.2000");
    dispatch(&mut s, Op::HmsAdd).unwrap();
    let expected = Decimal::from_str("1.3000").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "simple HMS addition without carry"
    );
}

// ── HMS− tests ────────────────────────────────────────────────────────────────

#[test]
fn test_hms_sub_with_borrow() {
    // Y=2.0500 (2h 05m) − X=1.4500 (1h 45m) = 0.2000 (0h 20m)
    // 65m total − 45m = 20m (inverse of add-carry test)
    let mut s = CalcState::new();
    push_dec(&mut s, "2.0500"); // Y
    push_dec(&mut s, "1.4500"); // X
    dispatch(&mut s, Op::HmsSub).unwrap();
    let expected = Decimal::from_str("0.2000").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "2.0500 − 1.4500 H.MMSS must produce 0.2000 (base-60 borrow)"
    );
}

#[test]
fn test_hms_sub_saves_lastx() {
    // HMS− is binary — it saves LASTX (the old X value) via binary_result()
    let mut s = CalcState::new();
    push_dec(&mut s, "2.0500");
    push_dec(&mut s, "1.4500");
    dispatch(&mut s, Op::HmsSub).unwrap();
    let expected_lastx = Decimal::from_str("1.4500").unwrap();
    assert_eq!(
        s.stack.lastx.inner(),
        expected_lastx,
        "HMS− must save old X into LASTX"
    );
}

#[test]
fn test_hms_add_invalid_operand_returns_invalid_input() {
    // X = 1.6000 (60 minutes) → both operands validated → HpError::InvalidInput
    let mut s = CalcState::new();
    push_dec(&mut s, "1.0000"); // Y (valid)
    push_dec(&mut s, "1.6000"); // X (invalid: 60 minutes)
    assert_eq!(
        dispatch(&mut s, Op::HmsAdd),
        Err(HpError::InvalidInput)
    );
}
