//! Integration tests for CORE-02: All operations declare correct stack-lift effects.

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

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
    // X=2 is in the valid domain for all these ops; all are fully implemented.
    let unary_ops = vec![Op::Recip, Op::Sq, Op::Ln, Op::Log, Op::Exp, Op::TenPow];
    for op in unary_ops {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(2);
        state.stack.lift_enabled = false;
        dispatch(&mut state, op.clone()).unwrap();
        assert!(state.stack.lift_enabled, "{op:?} must enable lift");
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

// ═══════════════════════════════════════════════════════════════════════════
// Phase 6 lift semantics — Stats and HMS ops
// ═══════════════════════════════════════════════════════════════════════════

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    state.stack.lift_enabled = true;
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

fn accumulate_two_points(s: &mut CalcState) {
    // Add two (X, Y) data points so n=2 (required for Sdev, LR, Yhat, Corr)
    s.stack.x = HpNum::from(1);
    s.stack.y = HpNum::from(2);
    dispatch(s, Op::SigmaPlus).unwrap();
    s.stack.x = HpNum::from(3);
    s.stack.y = HpNum::from(4);
    dispatch(s, Op::SigmaPlus).unwrap();
}

// ── Stats ops (Enable) ───────────────────────────────────────────────────

#[test]
fn test_sigma_plus_enables_lift() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1);
    s.stack.y = HpNum::from(2);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    assert!(s.stack.lift_enabled, "SigmaPlus must enable lift");
}

#[test]
fn test_sigma_minus_enables_lift() {
    let mut s = CalcState::new();
    // Accumulate one point first so SigmaMinus has data to remove
    s.stack.x = HpNum::from(1);
    s.stack.y = HpNum::from(2);
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::SigmaMinus).unwrap();
    assert!(s.stack.lift_enabled, "SigmaMinus must enable lift");
}

#[test]
fn test_mean_enables_lift() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1);
    s.stack.y = HpNum::from(2);
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Mean).unwrap();
    assert!(s.stack.lift_enabled, "Mean must enable lift");
}

#[test]
fn test_sdev_enables_lift() {
    let mut s = CalcState::new();
    accumulate_two_points(&mut s);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Sdev).unwrap();
    assert!(s.stack.lift_enabled, "Sdev must enable lift");
}

#[test]
fn test_lr_enables_lift() {
    let mut s = CalcState::new();
    accumulate_two_points(&mut s);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::LR).unwrap();
    assert!(s.stack.lift_enabled, "LR must enable lift");
}

#[test]
fn test_yhat_enables_lift() {
    let mut s = CalcState::new();
    accumulate_two_points(&mut s);
    s.stack.x = HpNum::from(2); // X value to predict Y for
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Yhat).unwrap();
    assert!(s.stack.lift_enabled, "Yhat must enable lift");
}

#[test]
fn test_corr_enables_lift() {
    let mut s = CalcState::new();
    accumulate_two_points(&mut s);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Corr).unwrap();
    assert!(s.stack.lift_enabled, "Corr must enable lift");
}

#[test]
fn test_clsigmastat_neutral_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::ClSigmaStat).unwrap();
    assert!(!s.stack.lift_enabled, "ClSigmaStat must be Neutral lift");
}

// ── HMS ops (all Enable) ─────────────────────────────────────────────────

#[test]
fn test_hms_to_h_enables_lift() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045"); // valid H.MMSS
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::HmsToH).unwrap();
    assert!(s.stack.lift_enabled, "HmsToH must enable lift");
}

#[test]
fn test_h_to_hms_enables_lift() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.5125"); // decimal hours
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::HToHms).unwrap();
    assert!(s.stack.lift_enabled, "HToHms must enable lift");
}

#[test]
fn test_hms_add_enables_lift() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.0000");
    push_dec(&mut s, "0.3000");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::HmsAdd).unwrap();
    assert!(s.stack.lift_enabled, "HmsAdd must enable lift");
}

#[test]
fn test_hms_sub_enables_lift() {
    let mut s = CalcState::new();
    push_dec(&mut s, "2.0000");
    push_dec(&mut s, "0.3000");
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::HmsSub).unwrap();
    assert!(s.stack.lift_enabled, "HmsSub must enable lift");
}
