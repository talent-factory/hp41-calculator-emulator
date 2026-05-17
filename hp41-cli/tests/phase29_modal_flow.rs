//! Phase 29 / Plan 03 — Modal-prompt routing integration tests for Math Pac I workflows.
//!
//! Tests CLI-05 modal-routing for MATRIX / SOLVE / POLY / INTG / DIFEQ / FOUR / TRANS.
//! Each test verifies one of the six contracts from plan 29-03.
//!
//! Test scaffolding mirrors `phase25_xeq_by_name.rs` (make_app, key, raw_key).

#![allow(clippy::unwrap_used)]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use hp41_cli::app::{App, PendingInput, XeqByNameMode};
use hp41_core::ops::math1::modal::{MatrixInputStep, ModalProgram, SolveInputStep};
use hp41_core::ops::Op;
use hp41_core::state::CalcState;

// ── Test scaffolding ─────────────────────────────────────────────────────────

fn key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn raw_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn make_app() -> (App, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir creation must succeed");
    let state_path = tmp.path().join("phase29-modal-flow-test-state.json");
    let app = App::new(CalcState::new(), state_path, None);
    (app, tmp)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Catches: R/S not flushing entry_buf before submit_modal, or submit_modal not advancing
/// the modal state from OrderPrompt to ElementPrompt(0,0).
///
/// Contract: R/S keystroke (F5) in Matrix(OrderPrompt) with entry_buf="3"
/// advances modal to Matrix(ElementPrompt(0,0)) with prompt "A1,1=?".
#[test]
fn matrix_workflow_order_prompt_advances_on_r_s() {
    let (mut app, _tmp) = make_app();

    // Dispatch Op::MatrixWorkflow — opens ORDER=? modal
    app.call_dispatch(Op::MatrixWorkflow);

    // Verify modal is open at OrderPrompt
    assert_eq!(
        app.state.modal_program,
        Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt)),
        "MatrixWorkflow must open modal at OrderPrompt"
    );
    assert_eq!(
        app.state.modal_prompt,
        Some("ORDER=?".to_string()),
        "modal_prompt must be 'ORDER=?' at OrderPrompt"
    );
    // No pending_input should be auto-opened (Matrix doesn't need a label)
    assert!(
        app.pending_input.is_none(),
        "Matrix modal should not open XeqByName after dispatch"
    );

    // Simulate typing '2' into entry_buf (the order)
    app.handle_key(key('2'));
    assert_eq!(app.state.entry_buf, "2", "entry_buf must capture digit '2'");

    // Press R/S (F5) — should call submit_modal which flushes entry_buf and advances modal
    app.handle_key(raw_key(KeyCode::F(5)));

    // Modal should advance to ElementPrompt(0,0) — first element row=0, col=0
    assert_eq!(
        app.state.modal_program,
        Some(ModalProgram::Matrix(MatrixInputStep::ElementPrompt(0, 0))),
        "R/S must advance Matrix modal from OrderPrompt to ElementPrompt(0,0)"
    );
    assert_eq!(
        app.state.modal_prompt,
        Some("A1,1=?".to_string()),
        "modal_prompt must be 'A1,1=?' after OrderPrompt submit"
    );
    // entry_buf must be cleared after flush
    assert!(
        app.state.entry_buf.is_empty(),
        "entry_buf must be cleared after submit_modal flushes it"
    );
}

/// Catches: auto-open hook not firing after Op::Solve dispatch, or
/// CollectForModal mode not set when modal requires alpha label.
///
/// Contract: dispatch(Op::Solve) when !is_running opens modal at
/// Solve(FunctionNamePrompt) AND auto-opens XeqByName{CollectForModal}.
#[test]
fn solve_workflow_auto_opens_collect_for_modal() {
    let (mut app, _tmp) = make_app();

    // Ensure not running
    app.state.is_running = false;

    // Dispatch Op::Solve — should open modal at FunctionNamePrompt
    app.call_dispatch(Op::Solve);

    // Verify modal is open at FunctionNamePrompt
    assert_eq!(
        app.state.modal_program,
        Some(ModalProgram::Solve(SolveInputStep::FunctionNamePrompt)),
        "Op::Solve must open modal at FunctionNamePrompt when !is_running"
    );
    assert_eq!(
        app.state.modal_prompt,
        Some("FUNCTION NAME?".to_string()),
        "modal_prompt must be 'FUNCTION NAME?' at FunctionNamePrompt"
    );

    // Auto-open hook: pending_input must be XeqByName{CollectForModal}
    match &app.pending_input {
        Some(PendingInput::XeqByName { acc, mode }) => {
            assert_eq!(acc, "", "CollectForModal accumulator must start empty");
            assert_eq!(
                *mode,
                XeqByNameMode::CollectForModal,
                "auto-open hook must set mode to CollectForModal"
            );
        }
        other => panic!(
            "expected XeqByName{{CollectForModal}} pending after Solve dispatch; got {other:?}"
        ),
    }
}

/// Catches: submit_modal_with_label not advancing Solve modal from FunctionNamePrompt to
/// Guess1Prompt, or alpha_reg not being set correctly.
///
/// Contract: typing 'F' + ENTER in CollectForModal mode calls submit_modal_with_label
/// which sets alpha_reg="F", advances modal to Guess1Prompt, clears pending_input.
#[test]
fn solve_workflow_label_submission_advances_to_guess1() {
    let (mut app, _tmp) = make_app();

    // Set up Solve modal at FunctionNamePrompt with CollectForModal pending
    app.state.modal_program = Some(ModalProgram::Solve(SolveInputStep::FunctionNamePrompt));
    app.state.modal_prompt = Some("FUNCTION NAME?".to_string());
    app.pending_input = Some(PendingInput::XeqByName {
        acc: String::new(),
        mode: XeqByNameMode::CollectForModal,
    });

    // Type 'F' into the accumulator
    app.handle_key(key('F'));
    match &app.pending_input {
        Some(PendingInput::XeqByName { acc, mode }) => {
            assert_eq!(acc, "F", "typed 'F' must appear in accumulator");
            assert_eq!(*mode, XeqByNameMode::CollectForModal);
        }
        other => panic!("expected XeqByName open after typing 'F'; got {other:?}"),
    }

    // Press Enter — should call submit_modal_with_label("F")
    app.handle_key(raw_key(KeyCode::Enter));

    // pending_input must be cleared
    assert!(
        app.pending_input.is_none(),
        "Enter in CollectForModal must clear pending_input"
    );

    // alpha_reg must be set to "F"
    assert_eq!(
        app.state.alpha_reg, "F",
        "submit_modal_with_label must write label to alpha_reg"
    );

    // Modal must advance to Guess1Prompt
    assert_eq!(
        app.state.modal_program,
        Some(ModalProgram::Solve(SolveInputStep::Guess1Prompt)),
        "label submission must advance Solve modal from FunctionNamePrompt to Guess1Prompt"
    );
    assert_eq!(
        app.state.modal_prompt,
        Some("GUESS 1=?".to_string()),
        "modal_prompt must be 'GUESS 1=?' after FunctionNamePrompt advance"
    );
}

/// Catches: cancel_modal leaving modal_prompt set, or stack being modified on cancel.
///
/// Contract: Esc when modal is open calls cancel_modal which clears modal_program,
/// modal_prompt, entry_buf, sets message="Cancelled"; stack untouched.
#[test]
fn esc_cancels_open_modal() {
    let (mut app, _tmp) = make_app();

    // Set a sentinel value in X register
    app.state.stack.x = hp41_core::num::HpNum::from(42i32);

    // Open Matrix modal
    app.call_dispatch(Op::MatrixWorkflow);
    assert!(
        app.state.modal_program.is_some(),
        "modal must be open after MatrixWorkflow dispatch"
    );
    assert!(
        app.pending_input.is_none(),
        "Matrix modal should not auto-open CollectForModal"
    );

    // Set some entry_buf to verify it's cleared on cancel
    app.state.entry_buf = "3".to_string();

    // Press Esc — should cancel modal
    app.handle_key(raw_key(KeyCode::Esc));

    // modal_program and modal_prompt must be cleared
    assert!(
        app.state.modal_program.is_none(),
        "Esc must clear modal_program via cancel_modal"
    );
    assert!(
        app.state.modal_prompt.is_none(),
        "Esc must clear modal_prompt via cancel_modal"
    );
    // entry_buf must be cleared
    assert!(
        app.state.entry_buf.is_empty(),
        "cancel_modal must clear entry_buf"
    );
    // message must be "Cancelled"
    assert_eq!(
        app.message,
        Some("Cancelled".to_string()),
        "cancel_modal must set message to 'Cancelled'"
    );
    // Stack must be untouched — X still holds sentinel value 42
    assert_eq!(
        app.state.stack.x,
        hp41_core::num::HpNum::from(42i32),
        "cancel_modal must not modify the stack"
    );
}

/// Catches: Esc with shift_armed=true accidentally cancelling modal instead of
/// consuming the shift prefix (§7.2 two-step Esc convention violation).
///
/// Contract: with shift_armed=true AND modal open, pressing Esc clears shift_armed
/// but leaves modal_program intact (two-step Esc convention per §7.2).
#[test]
fn esc_shift_armed_takes_precedence_over_modal_cancel() {
    let (mut app, _tmp) = make_app();

    // Open Matrix modal
    app.call_dispatch(Op::MatrixWorkflow);
    assert!(
        app.state.modal_program.is_some(),
        "Matrix modal must be open"
    );

    // Arm shift
    app.shift_armed = true;

    // Press Esc — shift_armed Esc should consume the shift prefix, NOT cancel the modal
    app.handle_key(raw_key(KeyCode::Esc));

    // shift_armed must be cleared
    assert!(
        !app.shift_armed,
        "Esc with shift_armed must clear shift_armed"
    );

    // Modal must STILL be open (§7.2 two-step convention: first Esc clears shift, second cancels)
    assert!(
        app.state.modal_program.is_some(),
        "modal must remain open after shift-Esc (§7.2 two-step convention)"
    );
}
