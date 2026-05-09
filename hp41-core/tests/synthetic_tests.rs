//! Integration tests for Phase 12 Synthetic Programming.
//! SYNT-01 (GETKEY), SYNT-02 (NULL), SYNT-03 (hidden regs M/N/O), SYNT-04 (SyntheticByte).
//!
//! Wave 0: all tests are RED (compile errors) until Plan 12-01 ships:
//! - CalcState fields: last_key_code, reg_m, reg_n, reg_o
//! - Op variants: GetKey, Null, StoM/StoN/StoO, RclM/RclN/RclO, SyntheticByte(u8)
//! - hp41_core::ops::synthetic_byte_to_op(u8) -> Option<Op>

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, synthetic_byte_to_op, Op};
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

// ── SYNT-01: GETKEY ──────────────────────────────────────────────────────────

/// GETKEY pushes 0 to X when no key has been pressed (last_key_code default = 0).
#[test]
fn test_getkey_zero_when_no_key_pressed() {
    let mut s = CalcState::new();
    // last_key_code = 0 (default) → GetKey must push 0 to X
    dispatch(&mut s, Op::GetKey).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(0), "GetKey with no prior key must push 0 to X");
}

/// GETKEY pushes the stored last_key_code to X.
#[test]
fn test_getkey_pushes_last_key_code() {
    let mut s = CalcState::new();
    s.last_key_code = 62; // row 6 col 2 = '5' key per HP-41 layout
    dispatch(&mut s, Op::GetKey).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(62), "GetKey must push last_key_code to X");
}

/// GETKEY lifts the stack (LiftEffect::Enable) — previous X moves to Y.
#[test]
fn test_getkey_lifts_stack() {
    let mut s = CalcState::new();
    push(&mut s, 7);
    s.last_key_code = 81; // row 8 col 1 = '0' key
    dispatch(&mut s, Op::GetKey).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(81), "GetKey must place key code in X");
    assert_eq!(s.stack.y.inner(), Decimal::from(7), "GetKey must lift previous X to Y");
}

/// GETKEY inside a running program also pushes the key code (execute_op arm exists).
#[test]
fn test_getkey_in_program() {
    let mut s = CalcState::new();
    s.last_key_code = 73; // row 7 col 3 = '3' key
    s.program = vec![Op::Lbl("G".to_string()), Op::GetKey, Op::Rtn];
    hp41_core::run_program(&mut s, "G").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(73), "GetKey in program must push key code to X");
}

// ── SYNT-02: NULL ────────────────────────────────────────────────────────────

/// NULL does not modify any stack register.
#[test]
fn test_null_does_not_modify_stack() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(Decimal::from(42));
    s.stack.y = HpNum::from(Decimal::from(7));
    s.stack.z = HpNum::from(Decimal::from(3));
    s.stack.t = HpNum::from(Decimal::from(1));
    dispatch(&mut s, Op::Null).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(42), "NULL must not change X");
    assert_eq!(s.stack.y.inner(), Decimal::from(7), "NULL must not change Y");
    assert_eq!(s.stack.z.inner(), Decimal::from(3), "NULL must not change Z");
    assert_eq!(s.stack.t.inner(), Decimal::from(1), "NULL must not change T");
}

/// NULL does not modify the lift flag.
#[test]
fn test_null_neutral_lift_effect() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false; // simulate post-Enter state
    dispatch(&mut s, Op::Null).unwrap();
    assert!(!s.stack.lift_enabled, "NULL must keep lift_enabled false (Neutral effect)");

    let mut s2 = CalcState::new();
    s2.stack.lift_enabled = true;
    dispatch(&mut s2, Op::Null).unwrap();
    assert!(s2.stack.lift_enabled, "NULL must keep lift_enabled true (Neutral effect)");
}

/// NULL does not modify any numbered register.
#[test]
fn test_null_does_not_modify_regs() {
    let mut s = CalcState::new();
    push(&mut s, 99);
    dispatch(&mut s, Op::StoReg(7)).unwrap();
    let reg7_before = s.regs[7].clone();
    dispatch(&mut s, Op::Null).unwrap();
    assert_eq!(s.regs[7], reg7_before, "NULL must not modify register 7");
}

// ── SYNT-03: Hidden registers M/N/O ──────────────────────────────────────────

/// STO M / RCL M round-trip preserves the value.
#[test]
fn test_sto_m_rcl_m_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 99);
    dispatch(&mut s, Op::StoM).unwrap();
    s.stack.x = HpNum::zero();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclM).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(99), "RCL M must restore stored value");
}

/// STO N / RCL N round-trip preserves the value.
#[test]
fn test_sto_n_rcl_n_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 17);
    dispatch(&mut s, Op::StoN).unwrap();
    s.stack.x = HpNum::zero();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclN).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(17), "RCL N must restore stored value");
}

/// STO O / RCL O round-trip preserves the value.
#[test]
fn test_sto_o_rcl_o_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 256);
    dispatch(&mut s, Op::StoO).unwrap();
    s.stack.x = HpNum::zero();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclO).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(256), "RCL O must restore stored value");
}

/// M, N, O are independent — storing to M does not affect N or O.
#[test]
fn test_hidden_regs_are_independent() {
    let mut s = CalcState::new();
    push(&mut s, 11);
    dispatch(&mut s, Op::StoM).unwrap();
    push(&mut s, 22);
    dispatch(&mut s, Op::StoN).unwrap();
    push(&mut s, 33);
    dispatch(&mut s, Op::StoO).unwrap();
    assert_eq!(s.reg_m.inner(), Decimal::from(11), "reg_m must hold first stored value");
    assert_eq!(s.reg_n.inner(), Decimal::from(22), "reg_n must hold second stored value");
    assert_eq!(s.reg_o.inner(), Decimal::from(33), "reg_o must hold third stored value");
}

/// Hidden register values survive a JSON serde round-trip (#[serde(default)] + persistent).
#[test]
fn test_hidden_regs_serde_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 5);
    dispatch(&mut s, Op::StoM).unwrap();
    push(&mut s, 6);
    dispatch(&mut s, Op::StoN).unwrap();
    push(&mut s, 7);
    dispatch(&mut s, Op::StoO).unwrap();
    let json = serde_json::to_string(&s).unwrap();
    let s2: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(s2.reg_m.inner(), Decimal::from(5), "reg_m must round-trip through JSON");
    assert_eq!(s2.reg_n.inner(), Decimal::from(6), "reg_n must round-trip through JSON");
    assert_eq!(s2.reg_o.inner(), Decimal::from(7), "reg_o must round-trip through JSON");
}

/// last_key_code survives a JSON serde round-trip (#[serde(default)] + persistent).
#[test]
fn test_last_key_code_serde_round_trip() {
    let mut s = CalcState::new();
    s.last_key_code = 42;
    let json = serde_json::to_string(&s).unwrap();
    let s2: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(s2.last_key_code, 42, "last_key_code must round-trip through JSON");
}

/// Loading a v1.0-style JSON without the new fields succeeds (#[serde(default)] backward compat).
/// The minimal JSON below contains only the absolutely required fields a v1.0 save would have.
#[test]
fn test_calcstate_loads_without_new_fields() {
    // v1.0 save files would not contain last_key_code, reg_m, reg_n, reg_o.
    // Use serde_json::Value to construct a stripped-down JSON, then deserialize.
    // If #[serde(default)] is missing on any new field, this test fails to deserialize.
    let full = serde_json::to_value(CalcState::new()).unwrap();
    let mut obj = full.as_object().unwrap().clone();
    obj.remove("last_key_code");
    obj.remove("reg_m");
    obj.remove("reg_n");
    obj.remove("reg_o");
    let stripped = serde_json::Value::Object(obj);
    let s: CalcState = serde_json::from_value(stripped).unwrap();
    assert_eq!(s.last_key_code, 0, "missing last_key_code must default to 0");
    assert_eq!(s.reg_m.inner(), Decimal::from(0), "missing reg_m must default to zero");
    assert_eq!(s.reg_n.inner(), Decimal::from(0), "missing reg_n must default to zero");
    assert_eq!(s.reg_o.inner(), Decimal::from(0), "missing reg_o must default to zero");
}

/// STO M / RCL M work inside a running program.
#[test]
fn test_hidden_reg_in_program() {
    let mut s = CalcState::new();
    push(&mut s, 88);
    s.program = vec![
        Op::Lbl("H".to_string()),
        Op::StoM,
        Op::PushNum(HpNum::zero()),
        Op::RclM,
        Op::Rtn,
    ];
    hp41_core::run_program(&mut s, "H").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(88), "RCL M in program must restore stored value");
}

// ── SYNT-04: SyntheticByte ───────────────────────────────────────────────────

/// Op::SyntheticByte(u8) survives a JSON serde round-trip.
#[test]
fn test_synthetic_byte_serde_round_trip() {
    let op = Op::SyntheticByte(0xCF);
    let json = serde_json::to_string(&op).unwrap();
    let op2: Op = serde_json::from_str(&json).unwrap();
    assert_eq!(op, op2, "Op::SyntheticByte(u8) must round-trip through JSON");
}

/// synthetic_byte_to_op returns Some for codes in the safe subset.
/// Initial table must include 0xCF → Op::Null (per RESEARCH.md Pattern 5).
#[test]
fn test_synthetic_byte_to_op_includes_null() {
    let op = synthetic_byte_to_op(0xCF);
    assert_eq!(op, Some(Op::Null), "0xCF must map to Op::Null in the safe subset");
}

/// synthetic_byte_to_op returns None for codes NOT in the safe subset.
/// 0x00 and 0xFF are reserved/unused — must always return None.
#[test]
fn test_synthetic_byte_to_op_rejects_unknown() {
    assert_eq!(synthetic_byte_to_op(0x00), None, "0x00 must not be in the safe subset");
    assert_eq!(synthetic_byte_to_op(0xFF), None, "0xFF must not be in the safe subset");
}

/// Op::SyntheticByte(b) at runtime delegates to the mapped op.
/// 0xCF maps to Op::Null — executing SyntheticByte(0xCF) must be a no-op.
#[test]
fn test_synthetic_byte_executes_as_null() {
    let mut s = CalcState::new();
    push(&mut s, 42);
    let x_before = s.stack.x.clone();
    let lift_before = s.stack.lift_enabled;
    dispatch(&mut s, Op::SyntheticByte(0xCF)).unwrap();
    assert_eq!(s.stack.x, x_before, "SyntheticByte(0xCF) → Op::Null must not change X");
    assert_eq!(s.stack.lift_enabled, lift_before, "SyntheticByte(0xCF) → Op::Null must not change lift flag");
}

/// Op::SyntheticByte with an unmapped byte returns InvalidOp at runtime.
/// Defensive: insertion path validates first, but execute_op must still error on bad data.
#[test]
fn test_synthetic_byte_unmapped_returns_error() {
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::SyntheticByte(0x00));
    assert!(result.is_err(), "SyntheticByte(0x00) must return error (not in safe subset)");
}

/// Op::SyntheticByte runs correctly inside a program (execute_op arm exists).
#[test]
fn test_synthetic_byte_in_program() {
    let mut s = CalcState::new();
    push(&mut s, 5);
    s.program = vec![
        Op::Lbl("S".to_string()),
        Op::SyntheticByte(0xCF), // Op::Null — no-op
        Op::Rtn,
    ];
    hp41_core::run_program(&mut s, "S").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(5), "SyntheticByte(0xCF) → Null must preserve X in program");
}
