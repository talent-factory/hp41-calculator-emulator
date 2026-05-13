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
