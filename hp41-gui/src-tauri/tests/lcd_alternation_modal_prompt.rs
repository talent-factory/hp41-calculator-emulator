// Phase 31 Plan 05 — LCD-alternation routing regression tests.
//
// Verifies that CalcStateView::from_state correctly implements the D-31.5 priority:
//   0. [TOP branch] modal_program.is_some() && entry_buf.is_empty() && modal_prompt.is_some()
//      → display_str = truncate_with_continuation(modal_prompt)
//   1. entry_buf non-empty → display_str = entry_buf (overrides modal prompt)
//   2. fallthrough to existing v2.2 logic (not exercised here).
//
// ROUTING NOTE: The LCD-alternation branch lives in `types.rs::CalcStateView::from_state`
// (NOT in `commands.rs::handle_get_state` and NOT via `display_override`).
// display_override is RESERVED for Phase 21 VIEW/AVIEW/PROMPT/CLD per state.rs.
//
// All 5 tests construct a fresh CalcState, set fields by direct field assignment
// (no op dispatch needed), and call CalcStateView::from_state directly.

#![allow(clippy::unwrap_used)]

use hp41_core::{
    ops::math1::modal::{MatrixInputStep, ModalProgram},
    CalcState,
};
use hp41_gui_lib::types::CalcStateView;

/// (a) Short prompt (under 12 chars) renders verbatim — no truncation.
///
/// Catches: LCD-alternation branch missing or not firing for modal_program.is_some()
#[test]
fn prompt_under_12_chars_no_truncation() {
    let mut calc = CalcState::new();
    // Set modal_program (any variant is fine — Matrix::OrderPrompt is simplest)
    calc.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    calc.modal_prompt = Some("ORDER=?".to_string()); // 7 chars — fits in 12
    // entry_buf is empty (default)
    assert!(calc.entry_buf.is_empty());

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "ORDER=?",
        "Short prompt must render verbatim (no truncation marker added)"
    );
}

/// (b) Prompt at exactly 12 chars renders verbatim — boundary: no truncation.
///
/// Catches: off-by-one in LCD_WIDTH boundary check (≤ vs <)
#[test]
fn prompt_at_12_chars_no_truncation() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    let prompt_12 = "DEGREE=?ABCD"; // exactly 12 chars
    assert_eq!(prompt_12.chars().count(), 12, "test fixture: must be exactly 12 chars");
    calc.modal_prompt = Some(prompt_12.to_string());
    assert!(calc.entry_buf.is_empty());

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "DEGREE=?ABCD",
        "12-char prompt must render verbatim (boundary: no truncation)"
    );
}

/// (c) Prompt at 13 chars truncates: first 11 chars + ≡ (U+2261) marker.
///
/// Catches: truncate_with_continuation not applied, wrong width, wrong continuation char
#[test]
fn prompt_at_13_chars_truncated_with_marker() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    let prompt_13 = "NO. SAMPLES=?"; // 13 chars
    assert_eq!(prompt_13.chars().count(), 13, "test fixture: must be exactly 13 chars");
    calc.modal_prompt = Some(prompt_13.to_string());
    assert!(calc.entry_buf.is_empty());

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    // Expected: first 11 chars + CONTINUATION marker (≡, U+2261)
    let expected: String = prompt_13.chars().take(11).collect::<String>() + "\u{2261}";
    assert_eq!(expected.chars().count(), 12, "expected must be 12 chars");
    assert_eq!(
        view.display_str, expected,
        "13-char prompt must be truncated to 11 chars + ≡ continuation marker"
    );
}

/// (d) Prompt at 14 chars truncates: first 11 chars ("FUNCTION NA") + ≡ marker.
///
/// Catches: truncation fires at wrong threshold, or ≡ not used
#[test]
fn prompt_at_14_chars_truncated() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    let prompt_14 = "FUNCTION NAME?"; // 14 chars
    assert_eq!(prompt_14.chars().count(), 14, "test fixture: must be exactly 14 chars");
    calc.modal_prompt = Some(prompt_14.to_string());
    assert!(calc.entry_buf.is_empty());

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    // "FUNCTION NAME?" chars: F(1)U(2)N(3)C(4)T(5)I(6)O(7)N(8) (9)N(10)A(11)M(12)E(13)?(14)
    // LCD_WIDTH=12, take(LCD_WIDTH-1)=take(11) → "FUNCTION NA" (11) + ≡ = "FUNCTION NA≡"
    assert_eq!(
        view.display_str, "FUNCTION NA\u{2261}",
        "14-char 'FUNCTION NAME?' must render as 'FUNCTION NA≡' (first 11 chars + ≡)"
    );
    assert_eq!(view.display_str.chars().count(), 12, "truncated result must be 12 chars");
}

/// (e) When entry_buf is non-empty, it overrides the modal prompt (live input feedback).
///
/// Catches: LCD-alternation branch incorrectly ignoring non-empty entry_buf
#[test]
fn entry_buf_nonempty_overrides_prompt() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt));
    calc.modal_prompt = Some("ORDER=?".to_string());
    // Simulate the user typing "3" (entering the matrix order)
    calc.entry_buf = "3".to_string();

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "3",
        "Non-empty entry_buf must override modal_prompt (live entry feedback per D-29.4)"
    );
}
