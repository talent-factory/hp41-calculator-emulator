//! Integration tests for CORE-02: All operations declare correct stack-lift effects.

use hp41_core::{CalcState, HpNum};
use hp41_core::ops::{dispatch, Op};

fn make_state_with_values() -> CalcState {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(2);
    s.stack.y = HpNum::from(3);
    s.stack.z = HpNum::from(4);
    s.stack.t = HpNum::from(5);
    s.stack.lift_enabled = false; // start with lift disabled
    s
}

// ── Enable (after op, lift_enabled must be true) ────────────────────────────

#[test]
fn test_add_enables_lift() {
    let mut state = make_state_with_values();
    dispatch(&mut state, Op::Add).unwrap();
    assert!(state.stack.lift_enabled, "Add must enable lift");
}

#[test]
fn test_sub_enables_lift() {
    let mut state = make_state_with_values();
    dispatch(&mut state, Op::Sub).unwrap();
    assert!(state.stack.lift_enabled, "Sub must enable lift");
}

#[test]
fn test_mul_enables_lift() {
    let mut state = make_state_with_values();
    dispatch(&mut state, Op::Mul).unwrap();
    assert!(state.stack.lift_enabled, "Mul must enable lift");
}

#[test]
fn test_div_enables_lift() {
    let mut state = make_state_with_values();
    state.stack.x = HpNum::from(1); // avoid div-by-zero
    dispatch(&mut state, Op::Div).unwrap();
    assert!(state.stack.lift_enabled, "Div must enable lift");
}

#[test]
fn test_lastx_enables_lift() {
    let mut state = make_state_with_values();
    state.stack.lastx = HpNum::from(7);
    dispatch(&mut state, Op::Lastx).unwrap();
    assert!(state.stack.lift_enabled, "Lastx must enable lift");
}

// ── Disable (after op, lift_enabled must be false) ──────────────────────────

#[test]
fn test_enter_disables_lift() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = true; // start enabled
    dispatch(&mut state, Op::Enter).unwrap();
    assert!(!state.stack.lift_enabled, "Enter must disable lift");
}

#[test]
fn test_clx_disables_lift() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Clx).unwrap();
    assert!(!state.stack.lift_enabled, "Clx must disable lift");
}

// ── Neutral (lift_enabled must be unchanged) ─────────────────────────────────

#[test]
fn test_chs_neutral_lift_true() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Chs).unwrap();
    assert!(state.stack.lift_enabled, "Chs is Neutral — must preserve true");
}

#[test]
fn test_chs_neutral_lift_false() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::Chs).unwrap();
    assert!(!state.stack.lift_enabled, "Chs is Neutral — must preserve false");
}

#[test]
fn test_rdn_neutral_lift_true() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Rdn).unwrap();
    assert!(state.stack.lift_enabled, "Rdn is Neutral — must preserve true");
}

#[test]
fn test_rdn_neutral_lift_false() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::Rdn).unwrap();
    assert!(!state.stack.lift_enabled, "Rdn is Neutral — must preserve false");
}

#[test]
fn test_xy_swap_neutral_lift_true() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::XySwap).unwrap();
    assert!(state.stack.lift_enabled, "XySwap is Neutral — must preserve true");
}

#[test]
fn test_xy_swap_neutral_lift_false() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::XySwap).unwrap();
    assert!(!state.stack.lift_enabled, "XySwap is Neutral — must preserve false");
}
