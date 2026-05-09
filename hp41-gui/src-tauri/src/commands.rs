//! Phase 14 IPC Layer — Tauri command handlers.
//!
//! Two commands registered in lib.rs:
//! - `dispatch_op(key_id, state)` — resolves key_id to Op, dispatches against CalcState,
//!   drains print_buffer, returns CalcStateView.
//! - `get_state(state)` — drains print_buffer and returns current CalcStateView (no dispatch).
//!
//! Decisions: D-08 (stateless IPC — no PendingModal), D-10 (Result<View, Error>),
//! and Claude's Discretion on poisoned-lock recovery (.unwrap_or_else(|e| e.into_inner())).
//!
//! Test strategy (RESEARCH.md Validation Architecture):
//! Tauri's `State<'_, AppState>` extractor cannot be constructed in unit tests without
//! the WebView mock harness. Therefore the IPC LOGIC is factored into two private helpers
//! `handle_op(&mut CalcState, &str)` and `handle_get_state(&mut CalcState)`. The
//! `#[tauri::command]` thunks are 2-line glue (lock + call helper). Tests target the
//! helpers and cover SC-2 (unknown key → GuiError) and SC-3 (print_buffer drain).
//!
//! Wave 0 status: signatures + RED tests. Bodies are `unimplemented!()` until Wave 1.

use crate::types::{CalcStateView, GuiError};
use crate::AppState;
use hp41_core::CalcState;
use tauri::State;

/// Tauri command: dispatch an op identified by a string key ID.
#[tauri::command]
pub fn dispatch_op(
    _key_id: &str,
    _state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    unimplemented!("Wave 1 (Plan 02): lock state, call handle_op, return result")
}

/// Tauri command: get current state without executing any op.
#[tauri::command]
pub fn get_state(_state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    unimplemented!("Wave 1 (Plan 02): lock state, call handle_get_state, return result")
}

/// Pure-Rust helper containing the dispatch_op logic — called by the Tauri thunk above
/// and exercised directly by unit tests. No Tauri State<> extractor needed.
pub fn handle_op(_calc: &mut CalcState, _key_id: &str) -> Result<CalcStateView, GuiError> {
    unimplemented!(
        "Wave 1 (Plan 02): digit-key short-circuit (\"0\"-\"9\", \".\", \"e\"), \
         then key_map::resolve, then hp41_core::ops::dispatch, then drain print_buffer, \
         then CalcStateView::from_state"
    )
}

/// Pure-Rust helper for get_state — drains print_buffer and builds a CalcStateView.
pub fn handle_get_state(_calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    unimplemented!("Wave 1 (Plan 02): drain print_buffer, build CalcStateView")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::ops::{dispatch, Op};
    use hp41_core::HpNum;

    #[test]
    fn test_dispatch_op_unknown_key() {
        // SC-2: unknown key_id returns GuiError, NEVER panics, NEVER silently discards.
        let mut calc = CalcState::new();
        let result = handle_op(&mut calc, "totally_unknown_xyz");
        let err = result.expect_err("unknown key must produce Err(GuiError)");
        assert!(
            err.message.contains("unknown key"),
            "expected 'unknown key' in error message, got: {}",
            err.message
        );
    }

    #[test]
    fn test_print_buffer_drained() {
        // SC-3: After a command that produces print output, the print_buffer is empty
        // AND the returned view.print_lines contains the produced lines.
        let mut calc = CalcState::new();
        calc.stack.x = HpNum::from(42);
        // Ensure the buffer has at least one line via direct dispatch (sanity).
        dispatch(&mut calc, Op::PRX).unwrap();
        assert!(!calc.print_buffer.is_empty(), "PRX should populate print_buffer");
        // Now exercise handle_op via the "prx" key — Wave 1 will produce another line
        // (or we'd have to reset buffer; we instead test the drain via handle_get_state).
        calc.print_buffer.clear();
        calc.stack.x = HpNum::from(7);
        dispatch(&mut calc, Op::PRX).unwrap();
        let pre_drain_lines = calc.print_buffer.len();
        assert_eq!(pre_drain_lines, 1, "second PRX should add 1 line");

        // Wave 1: handle_get_state must drain the buffer and put lines in the view.
        let view = handle_get_state(&mut calc).unwrap();
        assert!(
            calc.print_buffer.is_empty(),
            "print_buffer must be empty after handle_get_state drain"
        );
        assert_eq!(
            view.print_lines.len(),
            1,
            "view.print_lines should contain the drained line"
        );
    }
}
