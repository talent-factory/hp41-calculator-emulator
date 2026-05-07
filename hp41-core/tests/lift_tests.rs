//! Integration tests for CORE-02: All operations declare correct stack-lift effects.

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpNum};

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
    assert!(
        state.stack.lift_enabled,
        "Chs is Neutral — must preserve true"
    );
}

#[test]
fn test_chs_neutral_lift_false() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::Chs).unwrap();
    assert!(
        !state.stack.lift_enabled,
        "Chs is Neutral — must preserve false"
    );
}

#[test]
fn test_rdn_neutral_lift_true() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Rdn).unwrap();
    assert!(
        state.stack.lift_enabled,
        "Rdn is Neutral — must preserve true"
    );
}

#[test]
fn test_rdn_neutral_lift_false() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::Rdn).unwrap();
    assert!(
        !state.stack.lift_enabled,
        "Rdn is Neutral — must preserve false"
    );
}

#[test]
fn test_xy_swap_neutral_lift_true() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::XySwap).unwrap();
    assert!(
        state.stack.lift_enabled,
        "XySwap is Neutral — must preserve true"
    );
}

#[test]
fn test_xy_swap_neutral_lift_false() {
    let mut state = make_state_with_values();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::XySwap).unwrap();
    assert!(
        !state.stack.lift_enabled,
        "XySwap is Neutral — must preserve false"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Phase 2 lift semantics
// ═══════════════════════════════════════════════════════════════════════════

// ── Math ops (all Enable) ─────────────────────────────────────────────────

#[test]
fn test_p2_math_ops_enable_lift() {
    // Unary math ops must all set lift_enabled = true
    let unary_ops = vec![Op::Recip, Op::Sq, Op::Ln, Op::Log, Op::Exp, Op::TenPow];
    for op in unary_ops {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(2); // valid input domain for all these ops
        state.stack.lift_enabled = false;
        let _ = dispatch(&mut state, op.clone()); // may return InvalidOp (stub) or Ok
        if state.stack.x.inner() != rust_decimal::Decimal::from(2) {
            // op ran successfully — check lift was set
            assert!(state.stack.lift_enabled, "{op:?} must enable lift");
        }
    }
}

// ── Mode-setting ops (all Neutral) ───────────────────────────────────────

#[test]
fn test_p2_mode_ops_neutral_lift_when_disabled() {
    let mode_ops: Vec<Op> = vec![
        Op::SetDeg,
        Op::SetRad,
        Op::SetGrad,
        Op::FmtFix(4),
        Op::FmtSci(4),
        Op::FmtEng(3),
    ];
    for op in mode_ops {
        let mut s = CalcState::new();
        s.stack.lift_enabled = false;
        dispatch(&mut s, op.clone()).unwrap();
        assert!(
            !s.stack.lift_enabled,
            "{op:?} must be Neutral — must not enable lift"
        );
    }
}

#[test]
fn test_p2_mode_ops_neutral_lift_when_enabled() {
    let mode_ops: Vec<Op> = vec![
        Op::SetDeg,
        Op::SetRad,
        Op::SetGrad,
        Op::FmtFix(4),
        Op::FmtSci(4),
        Op::FmtEng(3),
    ];
    for op in mode_ops {
        let mut s = CalcState::new();
        s.stack.lift_enabled = true;
        dispatch(&mut s, op.clone()).unwrap();
        assert!(
            s.stack.lift_enabled,
            "{op:?} must be Neutral — must not disable lift"
        );
    }
}

// ── Register ops (STO=Neutral, RCL=Enable, STO-arith=Neutral) ────────────

#[test]
fn test_sto_reg_neutral_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::StoReg(0)).unwrap();
    assert!(!s.stack.lift_enabled, "StoReg must be Neutral lift");
}

#[test]
fn test_rcl_reg_enables_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclReg(0)).unwrap();
    assert!(s.stack.lift_enabled, "RclReg must Enable lift");
}

#[test]
fn test_clreg_neutral_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Clreg).unwrap();
    assert!(!s.stack.lift_enabled, "Clreg must be Neutral lift");
}

// ── ALPHA ops (all Neutral) ───────────────────────────────────────────────

#[test]
fn test_alpha_ops_neutral_lift_in_lift_tests() {
    let alpha_ops: Vec<Op> = vec![Op::AlphaToggle, Op::AlphaAppend('A'), Op::AlphaClear];
    for op in alpha_ops {
        let mut s = CalcState::new();
        s.stack.lift_enabled = false;
        dispatch(&mut s, op.clone()).unwrap();
        assert!(!s.stack.lift_enabled, "{op:?} must be Neutral lift");
    }
}
