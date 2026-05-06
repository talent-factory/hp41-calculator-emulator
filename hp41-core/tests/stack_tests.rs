//! Integration tests for CORE-01: 4-level stack and LASTX register behavior.

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

// ── Stack push / lift ────────────────────────────────────────────────────────

#[test]
fn test_push_overwrites_x_when_lift_disabled() {
    let mut state = CalcState::new();
    // lift_enabled starts false — push overwrites X
    push(&mut state, 5);
    assert_eq!(state.stack.x, HpNum::from(5));
    assert_eq!(state.stack.y, HpNum::zero());
}

#[test]
fn test_push_lifts_stack_when_lift_enabled() {
    let mut state = CalcState::new();
    push(&mut state, 3);
    // enable lift then push again
    state.stack.lift_enabled = true;
    push(&mut state, 7);
    assert_eq!(state.stack.x, HpNum::from(7));
    assert_eq!(state.stack.y, HpNum::from(3));
}

#[test]
fn test_t_register_duplicates_on_lift() {
    // HP-41 hardware: T is duplicated (not lost) when lifting past 4 values
    let mut state = CalcState::new();
    push(&mut state, 1);
    state.stack.lift_enabled = true;
    push(&mut state, 2);
    state.stack.lift_enabled = true;
    push(&mut state, 3);
    state.stack.lift_enabled = true;
    push(&mut state, 4);
    // stack: X=4, Y=3, Z=2, T=1
    state.stack.lift_enabled = true;
    push(&mut state, 5);
    // stack: X=5, Y=4, Z=3, T=2  (1 was displaced)
    assert_eq!(state.stack.x, HpNum::from(5));
    assert_eq!(state.stack.t, HpNum::from(2));
}

// ── ENTER ────────────────────────────────────────────────────────────────────

#[test]
fn test_enter_duplicates_x() {
    let mut state = CalcState::new();
    push(&mut state, 3);
    dispatch(&mut state, Op::Enter).unwrap();
    assert_eq!(state.stack.x, HpNum::from(3));
    assert_eq!(state.stack.y, HpNum::from(3));
}

#[test]
fn test_enter_disables_lift() {
    let mut state = CalcState::new();
    push(&mut state, 3);
    dispatch(&mut state, Op::Enter).unwrap();
    assert!(!state.stack.lift_enabled, "ENTER must disable lift");
}

#[test]
fn test_enter_twice_does_not_double_lift() {
    // 3 ENTER ENTER → X=3, Y=3, Z=3 (not garbage in Z)
    let mut state = CalcState::new();
    push(&mut state, 3);
    dispatch(&mut state, Op::Enter).unwrap();
    dispatch(&mut state, Op::Enter).unwrap();
    assert_eq!(state.stack.x, HpNum::from(3));
    assert_eq!(state.stack.y, HpNum::from(3));
    assert_eq!(state.stack.z, HpNum::from(3));
}

// ── CLX ──────────────────────────────────────────────────────────────────────

#[test]
fn test_clx_zeros_x() {
    let mut state = CalcState::new();
    push(&mut state, 42);
    dispatch(&mut state, Op::Clx).unwrap();
    assert_eq!(state.stack.x, HpNum::zero());
}

#[test]
fn test_clx_disables_lift() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Clx).unwrap();
    assert!(!state.stack.lift_enabled, "CLX must disable lift");
}

// ── CHS ──────────────────────────────────────────────────────────────────────

#[test]
fn test_chs_negates_x() {
    let mut state = CalcState::new();
    push(&mut state, 5);
    dispatch(&mut state, Op::Chs).unwrap();
    assert_eq!(state.stack.x, HpNum::from(-5));
}

#[test]
fn test_chs_neutral_lift_when_enabled() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Chs).unwrap();
    assert!(state.stack.lift_enabled, "CHS must be Neutral — must not clear lift");
}

#[test]
fn test_chs_neutral_lift_when_disabled() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::Chs).unwrap();
    assert!(!state.stack.lift_enabled, "CHS must be Neutral — must not set lift");
}

// ── LASTX after binary op ─────────────────────────────────────────────────────

#[test]
fn test_lastx_captures_x_before_add() {
    // 1 ENTER 2 + → X=3, LASTX=2  (X was 2 before the add)
    let mut state = CalcState::new();
    push(&mut state, 1);
    dispatch(&mut state, Op::Enter).unwrap();
    push(&mut state, 2);
    dispatch(&mut state, Op::Add).unwrap();
    assert_eq!(state.stack.x, HpNum::from(3));
    assert_eq!(state.stack.lastx, HpNum::from(2),
        "LASTX should be 2 (X before add), not 3 (result)");
}

#[test]
fn test_lastx_recall() {
    // 1 ENTER 2 + LASTX → X=2 (recalls the pre-add X)
    let mut state = CalcState::new();
    push(&mut state, 1);
    dispatch(&mut state, Op::Enter).unwrap();
    push(&mut state, 2);
    dispatch(&mut state, Op::Add).unwrap();
    dispatch(&mut state, Op::Lastx).unwrap();
    assert_eq!(state.stack.x, HpNum::from(2));
}

// ── Arithmetic result stack state ─────────────────────────────────────────────

#[test]
fn test_add_consumes_y() {
    // After Y+X, Y should contain old Z (not old Y)
    let mut state = CalcState::new();
    push(&mut state, 10); // X=10
    state.stack.lift_enabled = true;
    push(&mut state, 20); // Y=10, X=20
    state.stack.lift_enabled = true;
    push(&mut state, 30); // Z=10, Y=20, X=30
    // Note: T=0 (default)
    dispatch(&mut state, Op::Add).unwrap();
    // X = 20+30 = 50, Y = old Z = 10, Z = old T = 0
    assert_eq!(state.stack.x, HpNum::from(50));
    assert_eq!(state.stack.y, HpNum::from(10));
    assert_eq!(state.stack.z, HpNum::zero());
}

#[test]
fn test_div_by_zero_returns_error() {
    let mut state = CalcState::new();
    push(&mut state, 5);
    state.stack.lift_enabled = true;
    push(&mut state, 0);
    let result = dispatch(&mut state, Op::Div);
    assert_eq!(result, Err(HpError::DivideByZero));
}

#[test]
fn test_sub_order_is_y_minus_x() {
    // HP-41: 10 ENTER 3 - → 7  (Y - X)
    let mut state = CalcState::new();
    push(&mut state, 10);
    dispatch(&mut state, Op::Enter).unwrap();
    push(&mut state, 3);
    dispatch(&mut state, Op::Sub).unwrap();
    assert_eq!(state.stack.x, HpNum::from(7));
}

#[test]
fn test_rdn_rotates_stack() {
    // X=1, Y=2, Z=3, T=4 → RDN → X=2, Y=3, Z=4, T=1
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(1);
    state.stack.y = HpNum::from(2);
    state.stack.z = HpNum::from(3);
    state.stack.t = HpNum::from(4);
    dispatch(&mut state, Op::Rdn).unwrap();
    assert_eq!(state.stack.x, HpNum::from(2));
    assert_eq!(state.stack.y, HpNum::from(3));
    assert_eq!(state.stack.z, HpNum::from(4));
    assert_eq!(state.stack.t, HpNum::from(1));
}

#[test]
fn test_xy_swap() {
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(10);
    state.stack.y = HpNum::from(20);
    dispatch(&mut state, Op::XySwap).unwrap();
    assert_eq!(state.stack.x, HpNum::from(20));
    assert_eq!(state.stack.y, HpNum::from(10));
}
