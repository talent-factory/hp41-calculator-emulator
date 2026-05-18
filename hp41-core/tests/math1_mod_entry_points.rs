// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Plan 32-01 task 06 surgical gap-closure for `hp41-core/src/ops/math1/mod.rs`
//! (the only math1 source file at 0 % coverage per the Plan 32-01 reconnaissance
//! `just coverage --html` run). The file exposes three Phase 29 / CLI-05
//! additive public-surface helpers:
//!
//! - `submit_modal` (D-29.5) — flushes `entry_buf`, dispatches numeric R/S to
//!   the per-program submit_step.
//! - `cancel_modal` (D-29.6) — clears `modal_program`, `modal_prompt`,
//!   `entry_buf`; never errors.
//! - `submit_modal_with_label` (D-29.7) — uppercases an ALPHA label,
//!   writes to `alpha_reg`, dispatches to the per-program
//!   `submit_label_step` for the three user-callback programs.
//!
//! These helpers are exercised indirectly via the CLI integration tests in
//! Plan 29, but Plan 32-01's per-file ≥ 90 % gate requires direct test
//! exercise. The four #[test]s here pin the error-path contracts (WR-01
//! flush-only-after-guard, WR-02 alpha-reg-only-after-guard, no-modal-active
//! short-circuit) per D-27.1 risk-weighted discipline.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::ops::math1::modal::{IntegInputStep, MatrixInputStep, ModalProgram};
use hp41_core::ops::math1::{cancel_modal, submit_modal, submit_modal_with_label};
use hp41_core::state::CalcState;

// Catches: WR-01 — submit_modal flushing entry_buf BEFORE the no-modal-active
// guard would silently push the user's digits onto the stack with lift,
// leaving the stack mutated even though the function returns InvalidOp.
#[test]
fn submit_modal_no_modal_active_returns_invalid_op_without_flushing() {
    let mut state = CalcState::new();
    // Stage entry_buf with a value that WOULD flush to stack.x if the guard
    // didn't fire first. modal_program stays None — no modal active.
    state.entry_buf = "42".to_string();
    let stack_x_before = state.stack.x.clone();

    let result = submit_modal(&mut state);

    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_modal must return InvalidOp when no modal is active"
    );
    // WR-01 contract: entry_buf was NOT flushed (state.stack.x unchanged)
    assert_eq!(
        state.stack.x, stack_x_before,
        "WR-01: submit_modal must NOT flush entry_buf when no modal active"
    );
}

// Catches: cancel_modal does not clear stack/regs/integ_state (only modal
// fields). This pins the contract per D-29.6.
#[test]
fn cancel_modal_clears_modal_fields_only() {
    let mut state = CalcState::new();
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    state.modal_prompt = Some("ORDER=?".to_string());
    state.entry_buf = "3".to_string();
    state.alpha_reg = "PRESERVED".to_string();
    let stack_before = state.stack.clone();

    cancel_modal(&mut state);

    assert!(
        state.modal_program.is_none(),
        "cancel_modal must clear modal_program"
    );
    assert!(
        state.modal_prompt.is_none(),
        "cancel_modal must clear modal_prompt"
    );
    assert!(
        state.entry_buf.is_empty(),
        "cancel_modal must clear entry_buf"
    );
    assert_eq!(
        state.alpha_reg, "PRESERVED",
        "cancel_modal must NOT clear alpha_reg"
    );
    assert_eq!(
        state.stack.x, stack_before.x,
        "cancel_modal must NOT mutate stack"
    );
}

// Catches: WR-02 — submit_modal_with_label clobbering alpha_reg BEFORE the
// no-modal-active guard fires would silently destroy the user's existing
// ALPHA register contents.
#[test]
fn submit_modal_with_label_no_modal_active_preserves_alpha_reg() {
    let mut state = CalcState::new();
    state.alpha_reg = "ORIGINAL".to_string();
    // modal_program stays None — no modal active.

    let result = submit_modal_with_label(&mut state, "newlabel");

    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_modal_with_label must return InvalidOp when no modal active"
    );
    // WR-02 contract: alpha_reg was NOT overwritten
    assert_eq!(
        state.alpha_reg, "ORIGINAL",
        "WR-02: submit_modal_with_label must NOT clobber alpha_reg pre-guard"
    );
}

// Catches: WR-02 (extension) — submit_modal_with_label on a numeric-input
// step (NOT requires_alpha_label) clobbering alpha_reg silently.
#[test]
fn submit_modal_with_label_wrong_step_preserves_alpha_reg() {
    let mut state = CalcState::new();
    state.alpha_reg = "ORIGINAL".to_string();
    // Matrix OrderPrompt is a numeric step — does NOT require_alpha_label
    state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));

    let result = submit_modal_with_label(&mut state, "newlabel");

    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "submit_modal_with_label must return InvalidOp on non-FunctionNamePrompt step"
    );
    assert_eq!(
        state.alpha_reg, "ORIGINAL",
        "WR-02: submit_modal_with_label must NOT clobber alpha_reg when step rejects"
    );
}

// Catches: submit_modal_with_label accepting a FunctionNamePrompt step and
// uppercasing the label. Happy-path coverage of the INTG FunctionNamePrompt arm.
#[test]
fn submit_modal_with_label_integ_function_name_prompt_uppercases() {
    let mut state = CalcState::new();
    state.alpha_reg = "OLD".to_string();
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::FunctionNamePrompt));

    // The label is trimmed + uppercased before dispatch. submit_label_step
    // for INTG advances the modal to the IntervalPrompt step (or similar
    // — exact next-step is the integ.rs contract). We only assert that the
    // label was written and that the call returned without panicking.
    let _ = submit_modal_with_label(&mut state, "  myfn  ");
    assert_eq!(
        state.alpha_reg, "MYFN",
        "submit_modal_with_label must uppercase + trim the label"
    );
}
