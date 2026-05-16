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
use hp41_core::{CalcState, HpError};

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

/// Catches: xrom resolver returning Some() for a name not yet registered.
/// At Plan 28-01 stage (no Math 1 Op variants), all names including future
/// Math Pac I mnemonics must fall through to InvalidOp.
#[test]
fn future_math1_mnemonic_returns_invalid_op_at_plan_28_01() {
    let mut state = fresh_state();
    // "SINH" will be a Math Pac I mnemonic in Plan 28-02. At Plan 28-01 stage,
    // it must still return InvalidOp because math1_resolve returns None for it.
    let result = op_xeq(&mut state, "SINH");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "SINH must return InvalidOp at Plan 28-01 stage (no Math Pac I Op variants yet). \
         Plan 28-02 changes this test expectation."
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
    let program = vec![
        Op::Lbl("MAIN".to_string()),
        Op::Xeq("WPRGM".to_string()),
    ];
    let mut state = state_with_program(program);
    let result = run_program(&mut state, "MAIN");
    // WPRGM with empty alpha_reg → AlphaData, not InvalidOp
    assert_eq!(
        result,
        Err(HpError::AlphaData),
        "Programmatic Op::Xeq('WPRGM') must dispatch via builtin_card_op in run_loop too"
    );
}
