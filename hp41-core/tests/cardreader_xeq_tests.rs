//! XEQ-by-name fallback tests for the four Card Reader ops.
//! Spec: docs/superpowers/specs/2026-05-13-card-reader-manual-verification-design.md

#![allow(clippy::unwrap_used)]

use hp41_core::cardreader::CardOpRequest;
use hp41_core::ops::Op;
use hp41_core::run_program;
use hp41_core::state::CalcState;

fn state_with_alpha(name: &str) -> CalcState {
    let mut state = CalcState::new();
    state.alpha_reg = name.to_string();
    state
}

#[test]
fn run_program_xeq_wprgm_stages_write_program_request() {
    let mut state = state_with_alpha("QUAD");
    // program is empty — no LBL "WPRGM" anywhere
    run_program(&mut state, "WPRGM").expect("XEQ WPRGM via run_program must succeed");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteProgram { name: "QUAD".to_string() }),
    );
}

#[test]
fn run_program_xeq_rdprgm_stages_read_program_request() {
    let mut state = state_with_alpha("QUAD");
    run_program(&mut state, "RDPRGM").expect("XEQ RDPRGM via run_program must succeed");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::ReadProgram { name: "QUAD".to_string() }),
    );
}

#[test]
fn run_program_xeq_wdta_stages_write_data_request() {
    let mut state = state_with_alpha("BACKUP");
    run_program(&mut state, "WDTA").unwrap();
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteData { name: "BACKUP".to_string() }),
    );
}

#[test]
fn run_program_xeq_rdta_stages_read_data_request() {
    let mut state = state_with_alpha("BACKUP");
    run_program(&mut state, "RDTA").unwrap();
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::ReadData { name: "BACKUP".to_string() }),
    );
}

#[test]
fn run_program_unknown_label_still_errors() {
    let mut state = state_with_alpha("X");
    let err = run_program(&mut state, "TOTALLY_UNKNOWN").unwrap_err();
    // Existing behavior is HpError::InvalidOp on label miss; the fallback
    // must not change that for non-card names.
    assert!(
        matches!(err, hp41_core::error::HpError::InvalidOp),
        "unknown label must still surface InvalidOp, got {err:?}",
    );
}

#[test]
fn user_label_takes_precedence_over_builtin() {
    // If the operator's program has LBL "WPRGM", that label must win.
    // Guards against accidental shadowing of legitimate user code.
    let mut state = state_with_alpha("QUAD");
    state.program = vec![
        Op::Lbl("WPRGM".to_string()),
        Op::Rtn,
    ];
    run_program(&mut state, "WPRGM").expect("user LBL WPRGM must run, not stage a card op");
    assert!(
        state.pending_card_op.is_none(),
        "user LBL must take precedence over builtin fallback — no card op should be staged",
    );
}

#[test]
fn run_loop_xeq_wprgm_inside_program_stages_request() {
    // Program: LBL "MAIN" / XEQ "WPRGM" / RTN
    // Running MAIN should execute the XEQ step, which stages a WriteProgram
    // request and then returns to MAIN's RTN (top-level → terminate).
    let mut state = state_with_alpha("CARD1");
    state.program = vec![
        Op::Lbl("MAIN".to_string()),
        Op::Xeq("WPRGM".to_string()),
        Op::Rtn,
    ];
    run_program(&mut state, "MAIN").expect("MAIN must run cleanly");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteProgram { name: "CARD1".to_string() }),
    );
}

#[test]
fn run_loop_xeq_card_op_does_not_skip_next_instruction() {
    // Regression guard: if the XEQ card-op fallback over-advances pc, the
    // StoReg(1) below would be silently skipped and R01 would stay at 0.
    let mut state = state_with_alpha("CARD1");
    state.stack.x = hp41_core::num::HpNum::from(42i32);
    state.program = vec![
        Op::Lbl("MAIN".to_string()),
        Op::Xeq("WPRGM".to_string()), // stages a card op via the builtin fallback
        Op::StoReg(1),                 // MUST execute — proves pc advanced exactly one step
        Op::Rtn,
    ];
    run_program(&mut state, "MAIN").expect("MAIN must run cleanly");
    assert_eq!(
        state.regs[1],
        hp41_core::num::HpNum::from(42i32),
        "STO 01 after XEQ \"WPRGM\" must execute — proves the fallback did not over-advance pc",
    );
    // And the card op should still be staged.
    assert!(state.pending_card_op.is_some(), "card op must still be staged");
}

#[test]
fn op_xeq_interactive_dispatch_stages_card_request() {
    // Mirrors the GUI path: dispatch(Op::Xeq("WPRGM")) with is_running=false.
    use hp41_core::ops::dispatch;
    let mut state = state_with_alpha("QUAD");
    assert!(!state.is_running);
    dispatch(&mut state, Op::Xeq("WPRGM".to_string())).expect("interactive XEQ WPRGM must succeed");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteProgram { name: "QUAD".to_string() }),
    );
}

#[test]
fn op_xeq_interactive_unknown_name_still_errors() {
    use hp41_core::ops::dispatch;
    let mut state = state_with_alpha("X");
    let err = dispatch(&mut state, Op::Xeq("UNKNOWN_XYZ".to_string())).unwrap_err();
    assert!(
        matches!(err, hp41_core::error::HpError::InvalidOp),
        "interactive XEQ with unknown name must keep returning InvalidOp, got {err:?}",
    );
}
