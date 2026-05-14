//! Integration tests for Phase 21 Plans 01 & 02 (Flag storage + SF/CF ops
//! + conditional-skip flag tests FS?/FC?/FS?C/FC?C).
//!
//! Covers FN-FLAG-01 (flag storage with serde-default backward compat),
//! FN-FLAG-02 (conditional-skip semantics in run_loop with always-clear
//! side effect for ?C variants).

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{
    dispatch,
    flags::{flag_clear, flag_get, flag_set},
    program::run_program,
    FlagTestKind, Op,
};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

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

// ── Plan 21-02: Conditional flag-test integration (FN-FLAG-02) ──────────────

/// Helper: push a literal integer via Op::PushNum.
fn push(n: i64) -> Op {
    Op::PushNum(HpNum::from(Decimal::from_str(&n.to_string()).unwrap()))
}

#[test]
fn test_fs_q_in_program_executes_next_when_flag_set() {
    // SF 5 → FS? 5 (SET → does not skip) → push 1 → push 2 → final X=2, Y=1
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::SfFlag(5),
        Op::FlagTest {
            kind: FlagTestKind::IsSet,
            flag: 5,
        },
        push(1),
        push(2),
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
    assert_eq!(s.stack.y.inner(), Decimal::from(1));
}

#[test]
fn test_fs_q_in_program_skips_next_when_flag_clear() {
    // CF 5 → FS? 5 (CLEAR → skip the next step "push 1") → push 2 → final X=2
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::CfFlag(5),
        Op::FlagTest {
            kind: FlagTestKind::IsSet,
            flag: 5,
        },
        push(1),
        push(2),
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
    // Y is whatever was there before — 0 (initial), since push(1) was skipped.
    assert_eq!(s.stack.y.inner(), Decimal::ZERO);
}

#[test]
fn test_fc_q_in_program_skips_when_flag_set() {
    // SF 5 → FC? 5 (SET → skip — condition "is clear" is false) → push 2 → X=2
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::SfFlag(5),
        Op::FlagTest {
            kind: FlagTestKind::IsClear,
            flag: 5,
        },
        push(1),
        push(2),
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
    assert_eq!(s.stack.y.inner(), Decimal::ZERO);
}

#[test]
fn test_fc_q_in_program_executes_when_flag_clear() {
    // CF 5 → FC? 5 (CLEAR → execute next) → push 1 → push 2 → X=2, Y=1
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::CfFlag(5),
        Op::FlagTest {
            kind: FlagTestKind::IsClear,
            flag: 5,
        },
        push(1),
        push(2),
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
    assert_eq!(s.stack.y.inner(), Decimal::from(1));
}

#[test]
fn test_fs_q_c_clears_flag_after_test() {
    // SF 10 → FS?C 10 → after run, flag 10 must be CLEAR (always-clear side effect).
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::SfFlag(10),
        Op::FlagTest {
            kind: FlagTestKind::IsSetThenClear,
            flag: 10,
        },
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert!(!flag_get(s.flags, 10), "FS?C must clear flag after test");
}

#[test]
fn test_fs_q_c_on_clear_flag_idempotent() {
    // CF 10 → FS?C 10 → flag stays CLEAR (already-clear stays clear, idempotent).
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::CfFlag(10),
        Op::FlagTest {
            kind: FlagTestKind::IsSetThenClear,
            flag: 10,
        },
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert!(
        !flag_get(s.flags, 10),
        "idempotent — already-clear stays clear"
    );
}

#[test]
fn test_fc_q_c_clears_flag_after_test() {
    // SF 10 → FC?C 10 → after run, flag 10 cleared (always-clear).
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::SfFlag(10),
        Op::FlagTest {
            kind: FlagTestKind::IsClearThenClear,
            flag: 10,
        },
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert!(!flag_get(s.flags, 10), "FC?C must clear flag after test");
}

#[test]
fn test_fs_q_c_skip_branch() {
    // CF 5 → FS?C 5 (CLEAR → skip — was-clear before always-clear)
    //   → push 99 SKIPPED → push 7 → final X=7
    //   → flag 5 stays CLEAR (idempotent always-clear)
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::CfFlag(5),
        Op::FlagTest {
            kind: FlagTestKind::IsSetThenClear,
            flag: 5,
        },
        push(99),
        push(7),
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(7));
    assert!(!flag_get(s.flags, 5));
}

#[test]
fn test_fc_q_c_skip_branch() {
    // SF 5 → FC?C 5 (SET → skip — was-set before always-clear)
    //   → push 99 SKIPPED → push 7 → final X=7
    //   → flag 5 is now CLEAR (always-clear side effect fired)
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::SfFlag(5),
        Op::FlagTest {
            kind: FlagTestKind::IsClearThenClear,
            flag: 5,
        },
        push(99),
        push(7),
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(7));
    assert!(!flag_get(s.flags, 5), "FC?C must clear the flag");
}

#[test]
fn test_op_flag_test_interactive_dispatch_is_no_op() {
    // Interactive dispatch of FlagTest: no pc advance, no flag mutation, no stack change.
    let mut s = CalcState::new();
    let initial_pc = s.pc;
    s.flags = 0;
    dispatch(
        &mut s,
        Op::FlagTest {
            kind: FlagTestKind::IsSet,
            flag: 5,
        },
    )
    .unwrap();
    assert_eq!(s.pc, initial_pc);
    assert_eq!(s.flags, 0);
    assert_eq!(s.stack.x, HpNum::zero());
}
