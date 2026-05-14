//! Integration tests for Phase 21 Plan 01 (Flag storage + SF/CF ops).
//!
//! Covers FN-FLAG-01 (flag storage with serde-default backward compat) and the SF/CF
//! happy/error paths. Conditional-skip behavior (FN-FLAG-02) lives in Plan 21-02 and
//! is appended to this file by that plan.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{
    dispatch,
    flags::{flag_clear, flag_get, flag_set},
    Op,
};
use hp41_core::{CalcState, HpError};

#[test]
fn test_flags_field_defaults_to_zero() {
    let s = CalcState::new();
    assert_eq!(s.flags, 0u64);
}

#[test]
fn test_load_v20_save_no_flags_field() {
    let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap();
    let s: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(s.flags, 0u64, "v2.0 fixture must load with flags = 0");
    assert_eq!(
        s.last_key_code, 0,
        "fixture sanity: last_key_code round-tripped"
    );
    // reg_m default round-trip — proves fixture is well-formed beyond the omitted-field case.
    assert!(s.reg_m.is_zero(), "fixture sanity: reg_m round-tripped");
}

#[test]
fn test_serde_round_trip_with_flags_set() {
    let mut s = CalcState::new();
    s.flags = 0xDEAD_BEEFu64;
    let json = serde_json::to_string(&s).unwrap();
    let back: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.flags, 0xDEAD_BEEFu64);
}

#[test]
fn test_flag_get_set_clear_helpers_unit() {
    for n in [0u8, 1, 31, 55] {
        let f = flag_set(0, n);
        assert_eq!(f, 1u64 << n, "flag_set wrong for n={n}");
        assert!(flag_get(f, n), "flag_get wrong for n={n}");
        let c = flag_clear(u64::MAX, n);
        assert_eq!(c, u64::MAX & !(1u64 << n), "flag_clear wrong for n={n}");
    }
}

#[test]
fn test_flag_helpers_out_of_range_defensive() {
    assert!(!flag_get(0, 56));
    assert!(!flag_get(u64::MAX, 100));
    assert_eq!(flag_set(42, 56), 42);
    assert_eq!(flag_clear(42, 100), 42);
}

#[test]
fn test_op_sf_sets_bit() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::SfFlag(5)).unwrap();
    assert!(flag_get(s.flags, 5));
    // Only bit 5 must be set.
    assert_eq!(s.flags, 1u64 << 5);
}

#[test]
fn test_op_cf_clears_bit() {
    let mut s = CalcState::new();
    s.flags = u64::MAX;
    dispatch(&mut s, Op::CfFlag(5)).unwrap();
    assert!(!flag_get(s.flags, 5));
    // All other 0..=55 bits must still be set.
    for n in 0u8..=55 {
        if n != 5 {
            assert!(flag_get(s.flags, n), "bit {n} must still be set");
        }
    }
}

#[test]
fn test_op_sf_out_of_range_returns_invalid_op() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::SfFlag(56));
    assert!(matches!(r, Err(HpError::InvalidOp)));
    assert_eq!(s.flags, 0u64, "state must not mutate on InvalidOp");
}

#[test]
fn test_op_cf_out_of_range_returns_invalid_op() {
    let mut s = CalcState::new();
    s.flags = u64::MAX;
    let r = dispatch(&mut s, Op::CfFlag(56));
    assert!(matches!(r, Err(HpError::InvalidOp)));
    assert_eq!(s.flags, u64::MAX, "state must not mutate on InvalidOp");
}
