//! Phase 29 / Plan 03 — `pending_prompt` widened-signature unit tests.
//!
//! Verifies the precedence rules in the widened `pending_prompt` signature:
//!   `pub fn pending_prompt(pending: Option<&PendingInput>, modal_prompt: Option<&str>) -> String`
//!
//! Three unit tests per plan spec:
//! 1. `pending_prompt(None, Some("ORDER=?"))` returns `"ORDER=?"`
//! 2. `pending_prompt(Some(&XeqByName{Normal}), None)` returns XEQ display
//! 3. `pending_prompt(Some(&XeqByName{CollectForModal}), Some("FUNCTION NAME?"))` — modal wins

#![allow(clippy::unwrap_used)]

use hp41_cli::app::{PendingInput, XeqByNameMode};
use hp41_cli::ui::pending_prompt;

// Catches: pending_prompt regression — modal_prompt not rendered when no pending input
#[test]
fn pending_prompt_renders_modal_prompt_when_no_pending() {
    let result = pending_prompt(None, Some("ORDER=?"));
    assert_eq!(
        result, "ORDER=?",
        "pending_prompt(None, Some('ORDER=?')) must return 'ORDER=?'"
    );
}

// Catches: pending_prompt regression — existing XeqByName{Normal} rendering broken
#[test]
fn pending_prompt_renders_pending_when_no_modal() {
    let p = PendingInput::XeqByName {
        acc: "F".to_string(),
        mode: XeqByNameMode::Normal,
    };
    let result = pending_prompt(Some(&p), None);
    assert!(
        result.contains("XEQ"),
        "Normal mode XeqByName must render as XEQ ...; got: {result:?}"
    );
    assert!(
        result.contains('F'),
        "accumulated 'F' must appear in the prompt; got: {result:?}"
    );
}

// Catches: pending_prompt regression — modal_prompt must win over CollectForModal
// display to show the user the expected workflow prompt, not the XEQ UI mode label
#[test]
fn pending_prompt_modal_wins_when_both_active() {
    let p = PendingInput::XeqByName {
        acc: "F".to_string(),
        mode: XeqByNameMode::CollectForModal,
    };
    let result = pending_prompt(Some(&p), Some("FUNCTION NAME?"));
    assert!(
        result.contains("FUNCTION NAME"),
        "modal_prompt must win over CollectForModal display; got: {result:?}"
    );
}
