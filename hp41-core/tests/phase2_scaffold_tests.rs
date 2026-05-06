//! Phase 2 Plan 01 — RED phase tests for structural scaffolding.
//!
//! These tests cover:
//! - CalcState Phase 2 fields (regs, alpha_reg, alpha_mode, angle_mode, display_mode, entry_buf)
//! - AngleMode and DisplayMode enums
//! - HpNum Default implementation
//! - unary_result() helper in stack.rs
//! - Op enum Phase 2 variants and StoArithKind enum
//!
//! All tests in this file should FAIL before the Phase 2 scaffolding is in place.

use hp41_core::{CalcState, HpNum};
use hp41_core::state::{AngleMode, DisplayMode};
use hp41_core::stack::unary_result;
use hp41_core::ops::{Op, StoArithKind, dispatch};

// ── AngleMode enum ────────────────────────────────────────────────────────────

#[test]
fn test_angle_mode_has_three_variants() {
    let deg = AngleMode::Deg;
    let rad = AngleMode::Rad;
    let grad = AngleMode::Grad;
    assert_ne!(deg, rad);
    assert_ne!(deg, grad);
    assert_ne!(rad, grad);
    // Copy + Clone
    let _deg2 = deg;
    let _deg3 = deg; // still usable = Copy
}

#[test]
fn test_angle_mode_debug() {
    // Debug derive must work
    let _ = format!("{:?}", AngleMode::Deg);
    let _ = format!("{:?}", AngleMode::Rad);
    let _ = format!("{:?}", AngleMode::Grad);
}

// ── DisplayMode enum ──────────────────────────────────────────────────────────

#[test]
fn test_display_mode_has_three_variants() {
    let fix4 = DisplayMode::Fix(4);
    let sci4 = DisplayMode::Sci(4);
    let eng3 = DisplayMode::Eng(3);
    assert_ne!(fix4, sci4);
    assert_ne!(fix4, eng3);
    assert_ne!(sci4, eng3);
    // Fix(4) != Fix(5)
    assert_ne!(DisplayMode::Fix(4), DisplayMode::Fix(5));
}

#[test]
fn test_display_mode_debug() {
    let _ = format!("{:?}", DisplayMode::Fix(4));
    let _ = format!("{:?}", DisplayMode::Sci(2));
    let _ = format!("{:?}", DisplayMode::Eng(3));
}

// ── CalcState Phase 2 fields ──────────────────────────────────────────────────

#[test]
fn test_calcstate_has_regs_field_100_elements() {
    let state = CalcState::new();
    // Must have exactly 100 elements, all zero
    assert_eq!(state.regs.len(), 100);
    for (i, reg) in state.regs.iter().enumerate() {
        assert!(reg.is_zero(), "register {} should be zero on init", i);
    }
}

#[test]
fn test_calcstate_has_alpha_reg_field() {
    let state = CalcState::new();
    assert!(state.alpha_reg.is_empty(), "alpha_reg should be empty on init");
}

#[test]
fn test_calcstate_has_alpha_mode_field() {
    let state = CalcState::new();
    assert!(!state.alpha_mode, "alpha_mode should be false on init");
}

#[test]
fn test_calcstate_angle_mode_defaults_to_deg() {
    let state = CalcState::new();
    assert_eq!(state.angle_mode, AngleMode::Deg, "angle_mode must default to DEG");
}

#[test]
fn test_calcstate_display_mode_defaults_to_fix4() {
    let state = CalcState::new();
    assert_eq!(state.display_mode, DisplayMode::Fix(4), "display_mode must default to Fix(4)");
}

#[test]
fn test_calcstate_entry_buf_field() {
    let state = CalcState::new();
    assert!(state.entry_buf.is_empty(), "entry_buf should be empty on init");
}

#[test]
fn test_calcstate_regs_are_writable() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(42);
    assert_eq!(state.regs[5], HpNum::from(42));
    assert!(state.regs[0].is_zero());
    assert!(state.regs[99].is_zero());
}

// ── HpNum Default implementation ─────────────────────────────────────────────

#[test]
fn test_hpnum_default_returns_zero() {
    let n: HpNum = Default::default();
    assert!(n.is_zero(), "HpNum::default() must return zero");
}

#[test]
fn test_hpnum_default_eq_zero() {
    assert_eq!(HpNum::default(), HpNum::zero());
}

// ── unary_result() helper ─────────────────────────────────────────────────────

#[test]
fn test_unary_result_saves_x_to_lastx() {
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(5);
    state.stack.y = HpNum::from(10);
    unary_result(&mut state, HpNum::from(99));
    // lastx must be old X (5), not the result (99)
    assert_eq!(state.stack.lastx, HpNum::from(5), "LASTX must be saved before overwrite");
}

#[test]
fn test_unary_result_sets_x_to_result() {
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(5);
    unary_result(&mut state, HpNum::from(42));
    assert_eq!(state.stack.x, HpNum::from(42));
}

#[test]
fn test_unary_result_does_not_modify_y_z_t() {
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(1);
    state.stack.y = HpNum::from(2);
    state.stack.z = HpNum::from(3);
    state.stack.t = HpNum::from(4);
    unary_result(&mut state, HpNum::from(99));
    // Y, Z, T must be unchanged
    assert_eq!(state.stack.y, HpNum::from(2), "Y must not be modified by unary_result");
    assert_eq!(state.stack.z, HpNum::from(3), "Z must not be modified by unary_result");
    assert_eq!(state.stack.t, HpNum::from(4), "T must not be modified by unary_result");
}

#[test]
fn test_unary_result_enables_lift() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = false;
    unary_result(&mut state, HpNum::from(1));
    assert!(state.stack.lift_enabled, "unary_result must enable lift");
}

#[test]
fn test_unary_result_enables_lift_when_already_true() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = true;
    unary_result(&mut state, HpNum::from(1));
    assert!(state.stack.lift_enabled, "unary_result must keep lift enabled");
}

// ── StoArithKind enum ─────────────────────────────────────────────────────────

#[test]
fn test_sto_arith_kind_has_four_variants() {
    let a = StoArithKind::Add;
    let s = StoArithKind::Sub;
    let m = StoArithKind::Mul;
    let d = StoArithKind::Div;
    assert_ne!(a, s);
    assert_ne!(a, m);
    assert_ne!(a, d);
    assert_ne!(s, m);
}

#[test]
fn test_sto_arith_kind_debug_and_clone() {
    let k = StoArithKind::Add;
    let _ = format!("{:?}", k);
    let k2 = k.clone();
    assert_eq!(k, k2);
}

// ── Op enum Phase 2 variants ──────────────────────────────────────────────────

#[test]
fn test_op_enum_has_recip_variant() {
    let op = Op::Recip;
    let _ = format!("{:?}", op);
    assert_eq!(op.clone(), Op::Recip);
}

#[test]
fn test_op_enum_has_sqrt_variant() {
    let op = Op::Sqrt;
    assert_eq!(op.clone(), Op::Sqrt);
    assert_ne!(Op::Sqrt, Op::Recip);
}

#[test]
fn test_op_enum_has_sq_variant() {
    let op = Op::Sq;
    assert_eq!(op.clone(), Op::Sq);
}

#[test]
fn test_op_enum_has_ypow_variant() {
    let op = Op::YPow;
    assert_eq!(op.clone(), Op::YPow);
}

#[test]
fn test_op_enum_has_ln_variant() {
    let op = Op::Ln;
    assert_eq!(op.clone(), Op::Ln);
}

#[test]
fn test_op_enum_has_log_variant() {
    let op = Op::Log;
    assert_eq!(op.clone(), Op::Log);
}

#[test]
fn test_op_enum_has_exp_variant() {
    let op = Op::Exp;
    assert_eq!(op.clone(), Op::Exp);
}

#[test]
fn test_op_enum_has_tenpow_variant() {
    let op = Op::TenPow;
    assert_eq!(op.clone(), Op::TenPow);
}

#[test]
fn test_op_enum_has_trig_variants() {
    let _sin = Op::Sin;
    let _cos = Op::Cos;
    let _tan = Op::Tan;
    let _asin = Op::Asin;
    let _acos = Op::Acos;
    let _atan = Op::Atan;
    assert_ne!(Op::Sin, Op::Cos);
    assert_ne!(Op::Asin, Op::Sin);
}

#[test]
fn test_op_enum_has_angle_mode_variants() {
    let _deg = Op::SetDeg;
    let _rad = Op::SetRad;
    let _grad = Op::SetGrad;
    assert_ne!(Op::SetDeg, Op::SetRad);
    assert_ne!(Op::SetRad, Op::SetGrad);
}

#[test]
fn test_op_enum_has_format_variants() {
    let fix = Op::FmtFix(4);
    let sci = Op::FmtSci(4);
    let eng = Op::FmtEng(3);
    assert_ne!(fix, sci);
    assert_ne!(fix, eng);
    // FmtFix(4) != FmtFix(5)
    assert_ne!(Op::FmtFix(4), Op::FmtFix(5));
}

#[test]
fn test_op_enum_has_register_variants() {
    let sto = Op::StoReg(5);
    let rcl = Op::RclReg(5);
    assert_ne!(sto, rcl);
    assert_ne!(Op::StoReg(0), Op::StoReg(1));
    // StoArith
    let arith = Op::StoArith { reg: 5, kind: StoArithKind::Add };
    let _ = format!("{:?}", arith);
}

#[test]
fn test_op_enum_has_clreg_variant() {
    let op = Op::Clreg;
    assert_eq!(op.clone(), Op::Clreg);
}

#[test]
fn test_op_enum_has_alpha_variants() {
    let _toggle = Op::AlphaToggle;
    let _append = Op::AlphaAppend('A');
    let _clear = Op::AlphaClear;
    assert_ne!(Op::AlphaToggle, Op::AlphaClear);
    assert_ne!(Op::AlphaAppend('A'), Op::AlphaAppend('B'));
}

// ── Phase 1 regression: existing tests still compile and pass ─────────────────

#[test]
fn test_phase1_add_still_works() {
    use hp41_core::ops::{dispatch, Op};
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(3);
    state.stack.y = HpNum::from(4);
    dispatch(&mut state, Op::Add).unwrap();
    assert_eq!(state.stack.x, HpNum::from(7));
}

#[test]
fn test_phase1_calcstate_still_has_stack() {
    let state = CalcState::new();
    assert!(state.stack.x.is_zero());
    assert!(!state.stack.lift_enabled);
}
