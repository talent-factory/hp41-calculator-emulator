//! Integration tests for MATH-01: unary math ops, binary YPow, 10-digit accuracy,
//! LASTX save behavior, stack-lift enable for all math ops, %CH op-level integration
//! (stack mechanics, Y preservation, error atomicity), and %CH PRGM-mode recording
//! and playback.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{run_program, CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}

// ── 1/x ──────────────────────────────────────────────────────────────────

#[test]
fn test_recip_of_4_is_0_25() {
    let mut s = CalcState::new();
    push(&mut s, 4);
    dispatch(&mut s, Op::Recip).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from_str("0.25").unwrap());
}

#[test]
fn test_recip_of_zero_returns_divide_by_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0);
    assert_eq!(dispatch(&mut s, Op::Recip), Err(HpError::DivideByZero));
}

// ── √x ───────────────────────────────────────────────────────────────────

#[test]
fn test_sqrt_of_4_is_2() {
    let mut s = CalcState::new();
    push(&mut s, 4);
    dispatch(&mut s, Op::Sqrt).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
}

#[test]
fn test_sqrt_of_negative_returns_domain() {
    let mut s = CalcState::new();
    push(&mut s, -1);
    assert_eq!(dispatch(&mut s, Op::Sqrt), Err(HpError::Domain));
}

// ── x² ───────────────────────────────────────────────────────────────────

#[test]
fn test_sq_of_5_is_25() {
    let mut s = CalcState::new();
    push(&mut s, 5);
    dispatch(&mut s, Op::Sq).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(25));
}

// ── LN ───────────────────────────────────────────────────────────────────

#[test]
fn test_ln_of_1_is_0() {
    let mut s = CalcState::new();
    push(&mut s, 1);
    dispatch(&mut s, Op::Ln).unwrap();
    assert!(s.stack.x.is_zero(), "LN(1) must be 0");
}

#[test]
fn test_ln_2_accuracy_10_digits() {
    // LN(2) = 0.6931471806 (10 significant digits, HP-41 hardware value)
    let mut s = CalcState::new();
    push(&mut s, 2);
    dispatch(&mut s, Op::Ln).unwrap();
    let expected = Decimal::from_str("0.6931471806").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        expected,
        "LN(2) must equal 0.6931471806 at 10 sig digits"
    );
}

#[test]
fn test_ln_of_zero_returns_domain() {
    let mut s = CalcState::new();
    push(&mut s, 0);
    assert_eq!(dispatch(&mut s, Op::Ln), Err(HpError::Domain));
}

#[test]
fn test_ln_of_negative_returns_domain() {
    let mut s = CalcState::new();
    push(&mut s, -5);
    assert_eq!(dispatch(&mut s, Op::Ln), Err(HpError::Domain));
}

// ── LOG ──────────────────────────────────────────────────────────────────

#[test]
fn test_log_of_100_is_2() {
    let mut s = CalcState::new();
    push(&mut s, 100);
    dispatch(&mut s, Op::Log).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(2));
}

// ── e^x ──────────────────────────────────────────────────────────────────

#[test]
fn test_exp_of_0_is_1() {
    let mut s = CalcState::new();
    push(&mut s, 0);
    dispatch(&mut s, Op::Exp).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(1));
}

// ── 10^x ─────────────────────────────────────────────────────────────────

#[test]
fn test_tenpow_of_2_is_100() {
    let mut s = CalcState::new();
    push(&mut s, 2);
    dispatch(&mut s, Op::TenPow).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(100));
}

// ── Y^X ──────────────────────────────────────────────────────────────────

#[test]
fn test_ypow_2_to_10_is_1024() {
    let mut s = CalcState::new();
    push(&mut s, 2); // Y = 2
    s.stack.lift_enabled = true;
    push(&mut s, 10); // X = 10
    dispatch(&mut s, Op::YPow).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(1024));
}

#[test]
fn test_ypow_2_to_0_5_is_sqrt_2() {
    // 2^0.5 = √2 = 1.414213562 (10 sig digits)
    let mut s = CalcState::new();
    push(&mut s, 2);
    s.stack.lift_enabled = true;
    push_dec(&mut s, "0.5");
    dispatch(&mut s, Op::YPow).unwrap();
    let expected = Decimal::from_str("1.414213562").unwrap();
    assert_eq!(s.stack.x.inner(), expected);
}

// ── LASTX save for all unary math ops ────────────────────────────────────

#[test]
fn test_unary_ops_save_lastx() {
    // For every unary op, X before the op must equal LASTX after the op
    let ops = vec![
        Op::Recip,
        Op::Sqrt,
        Op::Sq,
        Op::Ln,
        Op::Log,
        Op::Exp,
        Op::TenPow,
    ];
    for op in ops {
        let mut s = CalcState::new();
        push(&mut s, 2); // X = 2 (valid domain for all these ops)
        let x_before = s.stack.x.inner();
        dispatch(&mut s, op.clone()).unwrap();
        assert_eq!(
            s.stack.lastx.inner(),
            x_before,
            "LASTX must be saved for {op:?}"
        );
    }
}

#[test]
fn test_ypow_saves_lastx() {
    // YPow is binary — saves X (the exponent) to LASTX
    let mut s = CalcState::new();
    push(&mut s, 2);
    s.stack.lift_enabled = true;
    push(&mut s, 3); // X = 3 (exponent)
    let x_before = s.stack.x.inner();
    dispatch(&mut s, Op::YPow).unwrap();
    assert_eq!(
        s.stack.lastx.inner(),
        x_before,
        "YPow must save X (exponent) to LASTX"
    );
}

// ── Stack-lift enable for all math ops ───────────────────────────────────

#[test]
fn test_math_ops_enable_lift() {
    let ops = vec![
        Op::Recip,
        Op::Sqrt,
        Op::Sq,
        Op::Ln,
        Op::Log,
        Op::Exp,
        Op::TenPow,
    ];
    for op in ops {
        let mut s = CalcState::new();
        push(&mut s, 2);
        s.stack.lift_enabled = false; // force disable before op
        dispatch(&mut s, op.clone()).unwrap();
        assert!(s.stack.lift_enabled, "{op:?} must enable stack lift");
    }
}

// ── Unary ops do NOT change Y, Z, T ──────────────────────────────────────

#[test]
fn test_unary_op_does_not_modify_y_z_t() {
    let mut s = CalcState::new();
    push(&mut s, 3);
    s.stack.lift_enabled = true;
    push(&mut s, 2);
    s.stack.lift_enabled = true;
    push(&mut s, 1); // X=1, Y=2, Z=3, T=0
    let y_before = s.stack.y.inner();
    let z_before = s.stack.z.inner();
    dispatch(&mut s, Op::Sq).unwrap(); // 1² = 1, Y/Z/T unchanged
    assert_eq!(s.stack.y.inner(), y_before, "Sq must not modify Y");
    assert_eq!(s.stack.z.inner(), z_before, "Sq must not modify Z");
}

// ── %CH (percent change) op-level integration tests ───────────────────────────
// These cover the stack mechanics: Y preservation (the defining feature of
// the HP-41 % family), LASTX capture, lift_enabled, and error atomicity.

#[test]
fn test_pct_change_basic_plus_15_percent() {
    // 200 ENTER 230 %CH → X=15
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(15));
}

#[test]
fn test_pct_change_preserves_y() {
    // The DEFINING test for this op. Y must survive intact.
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    s.stack.z = HpNum::from(7i32);
    s.stack.t = HpNum::from(13i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(15), "X must be the result");
    assert_eq!(
        s.stack.y.inner(),
        Decimal::from(200),
        "Y must be preserved (% family)"
    );
    assert_eq!(s.stack.z.inner(), Decimal::from(7), "Z must be untouched");
    assert_eq!(s.stack.t.inner(), Decimal::from(13), "T must be untouched");
}

#[test]
fn test_pct_change_saves_old_x_to_lastx() {
    // LASTX must capture the *old* X (230), not the result (15).
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.lastx.inner(), Decimal::from(230));
}

#[test]
fn test_pct_change_enables_lift() {
    // After %CH, the next number-entry must lift the stack (not overwrite X).
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(100i32);
    s.stack.x = HpNum::from(125i32);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::PctChange).unwrap();
    assert!(
        s.stack.lift_enabled,
        "%CH must enable stack lift after execution"
    );
}

#[test]
fn test_pct_change_divide_by_zero_leaves_stack_untouched() {
    // Atomicity invariant: Err path makes no partial writes.
    let mut s = CalcState::new();
    s.stack.y = HpNum::zero();
    s.stack.x = HpNum::from(42i32);
    s.stack.z = HpNum::from(7i32);
    s.stack.t = HpNum::from(13i32);
    let lastx_before = s.stack.lastx.clone();
    let lift_enabled_before = s.stack.lift_enabled;
    let result = dispatch(&mut s, Op::PctChange);
    assert_eq!(result, Err(HpError::DivideByZero));
    assert_eq!(
        s.stack.y.inner(),
        Decimal::from(0),
        "Y must be untouched on Err"
    );
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(42),
        "X must be untouched on Err"
    );
    assert_eq!(
        s.stack.z.inner(),
        Decimal::from(7),
        "Z must be untouched on Err"
    );
    assert_eq!(
        s.stack.t.inner(),
        Decimal::from(13),
        "T must be untouched on Err"
    );
    assert_eq!(
        s.stack.lastx.inner(),
        lastx_before.inner(),
        "LASTX must be untouched on Err"
    );
    assert_eq!(
        s.stack.lift_enabled, lift_enabled_before,
        "lift_enabled must be untouched on Err"
    );
}

#[test]
fn test_pct_change_lastx_round_trip() {
    // 200 ENTER 230 %CH LASTX  →  X is the *original* 230 (not 15).
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(15)); // sanity
    dispatch(&mut s, Op::Lastx).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(230),
        "LASTX must restore old X"
    );
}

#[test]
fn test_pct_change_chained_invocation() {
    // Y=100, X=125 → %CH → X=25, Y=100 (preserved).
    // Then push 150 via Op::PushNum — lift_enabled is true after %CH, so 150
    // lifts the stack: T←Z, Z←Y(=100), Y←X(=25), X=150.
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(100i32);
    s.stack.x = HpNum::from(125i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(25));
    assert_eq!(s.stack.y.inner(), Decimal::from(100));
    // Push 150 via the public dispatch path (integration tests don't import
    // crate::stack::enter_number — Op::PushNum is the equivalent and honours lift_enabled).
    dispatch(&mut s, Op::PushNum(HpNum::from(150i32))).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(150));
    assert_eq!(
        s.stack.y.inner(),
        Decimal::from(25),
        "Y after lift is prior X (25)"
    );
}

// ── %CH PRGM mode: recording and playback ─────────────────────────────────────

#[test]
fn test_pct_change_recorded_into_program_when_prgm_mode() {
    // In prgm_mode = true, dispatching Op::PctChange must APPEND to state.program
    // (recording) and MUST NOT touch the stack.
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    s.prgm_mode = true;
    let program_len_before = s.program.len();

    dispatch(&mut s, Op::PctChange).unwrap();

    assert_eq!(
        s.program.len(),
        program_len_before + 1,
        "PctChange must be appended to program Vec"
    );
    assert_eq!(
        s.program.last(),
        Some(&Op::PctChange),
        "the appended op must be PctChange"
    );
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(230),
        "X must be untouched in PRGM mode (op was executed instead of recorded)"
    );
    assert_eq!(
        s.stack.y.inner(),
        Decimal::from(200),
        "Y must be untouched in PRGM mode (op was executed instead of recorded)"
    );
}

#[test]
fn test_pct_change_playback_via_run_program() {
    // Build a tiny program: LBL "T", PushNum(200), Enter, PushNum(230), PctChange, Rtn.
    // Run it. Expect X=15, Y=200.
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::PushNum(HpNum::from(200i32)),
        Op::Enter,
        Op::PushNum(HpNum::from(230i32)),
        Op::PctChange,
        Op::Rtn,
    ];
    run_program(&mut s, "T").unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(15),
        "playback result must equal 15"
    );
    assert_eq!(
        s.stack.y.inner(),
        Decimal::from(200),
        "Y preserved after playback"
    );
}
