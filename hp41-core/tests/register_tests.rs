//! Integration tests for REGS-01: storage registers R00–R99, STO/RCL, STO-arith.

use hp41_core::ops::{dispatch, Op, StoArithKind};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

// ── STO / RCL round-trip ─────────────────────────────────────────────────

#[test]
fn test_sto_rcl_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 42);
    dispatch(&mut s, Op::StoReg(5)).unwrap();
    // Clear X to confirm RCL restores the value
    s.stack.x = HpNum::zero();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclReg(5)).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(42),
        "RCL must restore STO'd value"
    );
}

#[test]
fn test_sto_does_not_change_x() {
    let mut s = CalcState::new();
    push(&mut s, 7);
    dispatch(&mut s, Op::StoReg(0)).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(7), "STO must not change X");
}

#[test]
fn test_rcl_pushes_to_stack() {
    let mut s = CalcState::new();
    push(&mut s, 10);
    dispatch(&mut s, Op::StoReg(3)).unwrap();
    push(&mut s, 99);
    s.stack.lift_enabled = true; // force lift for RCL
    dispatch(&mut s, Op::RclReg(3)).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(10),
        "RCL must become new X"
    );
    assert_eq!(
        s.stack.y.inner(),
        Decimal::from(99),
        "previous X must lift to Y"
    );
}

// ── STO-arith ────────────────────────────────────────────────────────────

#[test]
fn test_sto_add_updates_register() {
    let mut s = CalcState::new();
    s.regs[5] = HpNum::from(10);
    push(&mut s, 3);
    dispatch(
        &mut s,
        Op::StoArith {
            reg: 5,
            kind: StoArithKind::Add,
        },
    )
    .unwrap();
    assert_eq!(s.regs[5].inner(), Decimal::from(13));
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(3),
        "X must be unchanged after STO+"
    );
}

#[test]
fn test_sto_sub_updates_register() {
    let mut s = CalcState::new();
    s.regs[2] = HpNum::from(20);
    push(&mut s, 5);
    dispatch(
        &mut s,
        Op::StoArith {
            reg: 2,
            kind: StoArithKind::Sub,
        },
    )
    .unwrap();
    assert_eq!(s.regs[2].inner(), Decimal::from(15));
}

#[test]
fn test_sto_mul_updates_register() {
    let mut s = CalcState::new();
    s.regs[0] = HpNum::from(4);
    push(&mut s, 3);
    dispatch(
        &mut s,
        Op::StoArith {
            reg: 0,
            kind: StoArithKind::Mul,
        },
    )
    .unwrap();
    assert_eq!(s.regs[0].inner(), Decimal::from(12));
}

#[test]
fn test_sto_div_updates_register() {
    let mut s = CalcState::new();
    s.regs[1] = HpNum::from(10);
    push(&mut s, 2);
    dispatch(
        &mut s,
        Op::StoArith {
            reg: 1,
            kind: StoArithKind::Div,
        },
    )
    .unwrap();
    assert_eq!(s.regs[1].inner(), Decimal::from(5));
}

// ── CLREG ────────────────────────────────────────────────────────────────

#[test]
fn test_clreg_zeros_all_registers() {
    let mut s = CalcState::new();
    s.regs[0] = HpNum::from(1);
    s.regs[50] = HpNum::from(2);
    s.regs[99] = HpNum::from(3);
    dispatch(&mut s, Op::Clreg).unwrap();
    assert!(s.regs[0].is_zero(), "R00 must be zero after CLREG");
    assert!(s.regs[50].is_zero(), "R50 must be zero after CLREG");
    assert!(s.regs[99].is_zero(), "R99 must be zero after CLREG");
}

// ── Out-of-range register ────────────────────────────────────────────────

#[test]
fn test_sto_out_of_range_returns_invalid_op() {
    let mut s = CalcState::new();
    push(&mut s, 1);
    assert_eq!(dispatch(&mut s, Op::StoReg(100)), Err(HpError::InvalidOp));
}

#[test]
fn test_rcl_out_of_range_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::RclReg(100)), Err(HpError::InvalidOp));
}

// ── Lift semantics ───────────────────────────────────────────────────────

#[test]
fn test_sto_is_neutral_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    push(&mut s, 1);
    dispatch(&mut s, Op::StoReg(0)).unwrap();
    assert!(
        !s.stack.lift_enabled,
        "STO must be Neutral — must not set lift to true"
    );
}

#[test]
fn test_sto_is_neutral_lift_when_lift_already_true() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = true;
    push(&mut s, 1);
    dispatch(&mut s, Op::StoReg(0)).unwrap();
    // Neutral = does not CHANGE lift, so it should remain true
    assert!(
        s.stack.lift_enabled,
        "STO Neutral must not disable lift either"
    );
}

#[test]
fn test_rcl_enables_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclReg(0)).unwrap();
    assert!(s.stack.lift_enabled, "RCL must enable lift");
}

#[test]
fn test_sto_arith_is_neutral_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    push(&mut s, 1);
    dispatch(
        &mut s,
        Op::StoArith {
            reg: 0,
            kind: StoArithKind::Add,
        },
    )
    .unwrap();
    assert!(!s.stack.lift_enabled, "STO+ must be Neutral lift");
}

// ── All registers initialized to zero ────────────────────────────────────

#[test]
fn test_registers_initialized_to_zero() {
    let s = CalcState::new();
    assert!(s.regs[0].is_zero(), "R00 must be zero on startup");
    assert!(s.regs[99].is_zero(), "R99 must be zero on startup");
}
