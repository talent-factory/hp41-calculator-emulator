// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 integration test: XROM resolver fires LAST in the chain (C-28.4 / Pitfall 1).
//!
//! Asserts:
//! 1. Unknown XEQ name → `HpError::InvalidOp` (unchanged from v2.2).
//! 2. Built-in name (WPRGM) wins over any hypothetical XROM alias — builtin fires BEFORE xrom.
//! 3. At Plan 28-01 stage (no Math 1 Op variants): any name (including future Math Pac I
//!    mnemonics) falls through to `Err(InvalidOp)` because `math1_resolve` returns `None`.
//!
//! The positive xrom-fires-LAST case (a real Math 1 mnemonic dispatching correctly) is
//! wired in Plan 28-02 once `Op::Sinh` exists and `MATH_1.ops` is populated.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::program::{op_xeq, run_program};
use hp41_core::ops::Op;
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;

// ── Helpers ────────────────────────────────────────────────────────────────

fn fresh_state() -> CalcState {
    CalcState::new()
}

fn state_with_program(ops: Vec<Op>) -> CalcState {
    let mut state = fresh_state();
    state.program = ops;
    state
}

// ── Tests ──────────────────────────────────────────────────────────────────

/// Catches: XROM resolver failing to propagate InvalidOp for unknown names
/// (regression — v2.2 behavior must be preserved).
#[test]
fn unknown_xeq_returns_invalid_op() {
    let mut state = fresh_state();
    let result = op_xeq(&mut state, "XYZZY_COMPLETELY_UNKNOWN");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "XEQ of an unknown name must return InvalidOp — both v2.2 and v3.0 behavior"
    );
}

/// Catches: XROM resolver incorrectly shadowing a builtin (Pitfall 1 positive case).
/// WPRGM is a builtin_card_op entry; it must NOT fall through to xrom even if
/// a Math Pac I op happened to be named "WPRGM" (it doesn't, but the chain order
/// must be verified structurally).
#[test]
fn builtin_wprgm_wins_over_xrom() {
    let mut state = fresh_state();
    // WPRGM with empty alpha_reg → HpError::AlphaData (not InvalidOp).
    // If it returned InvalidOp, it would mean builtin_card_op was bypassed.
    let result = op_xeq(&mut state, "WPRGM");
    assert_ne!(
        result,
        Err(HpError::InvalidOp),
        "WPRGM must resolve via builtin_card_op (returns AlphaData), \
         not fall through to InvalidOp (which would mean xrom shadowed it)"
    );
    assert_eq!(
        result,
        Err(HpError::AlphaData),
        "WPRGM with empty alpha_reg must return AlphaData (hardware-faithful)"
    );
}

/// Catches: xrom resolver not firing for a registered Math Pac I mnemonic.
/// Plan 28-02: SINH is now registered in math1_resolve; op_xeq("SINH") must
/// dispatch via xrom_resolve → Op::Sinh → op_sinh rather than InvalidOp.
/// CalcState::new() defaults xrom_modules = 0b0000_0001 (Math 1 pre-loaded).
#[test]
fn math1_sinh_resolves_via_xrom() {
    let mut state = fresh_state();
    // Push X = 0 so SINH(0) = 0 (success, no domain error)
    state.stack.x = HpNum::from(Decimal::ZERO);
    state.stack.lift_enabled = true;
    // xrom_modules defaults to 0b0000_0001 (Math 1 loaded) in fresh CalcState
    let result = op_xeq(&mut state, "SINH");
    assert!(
        result.is_ok(),
        "XEQ 'SINH' with Math 1 loaded must dispatch successfully (not InvalidOp): {:?}",
        result
    );
}

/// Catches: programmatic XEQ in run_loop using the same resolver chain.
/// Op::Xeq("UNKNOWN") inside a program must return HpError::InvalidOp.
#[test]
fn programmatic_xeq_unknown_returns_invalid_op() {
    let program = vec![
        Op::Lbl("MAIN".to_string()),
        Op::Xeq("XYZZY_COMPLETELY_UNKNOWN".to_string()),
    ];
    let mut state = state_with_program(program);
    let result = run_program(&mut state, "MAIN");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "Programmatic Op::Xeq of an unknown name must return InvalidOp via run_loop"
    );
}

/// Catches: programmatic WPRGM via run_loop — builtin chain fires in run_loop too.
#[test]
fn programmatic_xeq_wprgm_via_run_loop() {
    let program = vec![Op::Lbl("MAIN".to_string()), Op::Xeq("WPRGM".to_string())];
    let mut state = state_with_program(program);
    let result = run_program(&mut state, "MAIN");
    // WPRGM with empty alpha_reg → AlphaData, not InvalidOp
    assert_eq!(
        result,
        Err(HpError::AlphaData),
        "Programmatic Op::Xeq('WPRGM') must dispatch via builtin_card_op in run_loop too"
    );
}
