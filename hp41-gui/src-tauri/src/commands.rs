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

use crate::key_map;
use crate::types::{CalcStateView, GuiError};
use crate::AppState;
use hp41_core::ops::dispatch;
use hp41_core::CalcState;
use tauri::State;

/// Tauri command: dispatch an op identified by a string key ID.
///
/// Locks AppState (with poisoned-lock recovery) and delegates to `handle_op`. The command
/// MUST stay tiny — all logic lives in the helper so it is unit-testable without a Tauri
/// runtime.
#[tauri::command]
pub fn dispatch_op(
    key_id: &str,
    state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_op(&mut calc, key_id)
}

/// Tauri command: get current state without executing any op.
///
/// Locks AppState (with poisoned-lock recovery) and delegates to `handle_get_state`.
#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_get_state(&mut calc)
}

/// Pure-Rust helper containing the dispatch_op logic — called by the Tauri thunk above
/// and exercised directly by unit tests. No Tauri State<> extractor needed.
///
/// Routing logic mirrors hp41-cli/src/app.rs lines 342–388 (digit entry) and lines
/// 998–1019 (call_dispatch_and_drain). Digit keys append to entry_buf and bypass
/// `dispatch`; named/parameterized keys flow through key_map → dispatch → drain.
pub fn handle_op(calc: &mut CalcState, key_id: &str) -> Result<CalcStateView, GuiError> {
    // ── Digit keys 0..=9 — bypass dispatch, append to entry_buf ───────────────
    if matches!(
        key_id,
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
    ) {
        // INPUT-02 guard (Phase 9): cap exponent entry at 2 digits.
        if let Some(e_pos) = calc.entry_buf.find('e') {
            let after_e = &calc.entry_buf[e_pos + 1..];
            let exp_digits = after_e.chars().filter(|ch| ch.is_ascii_digit()).count();
            if exp_digits >= 2 {
                // Silently block third exponent digit — return current view unchanged.
                let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
                return Ok(CalcStateView::from_state(calc, print_lines));
            }
        }
        // key_id is exactly one ASCII char — safe to take .next().
        let ch = key_id
            .chars()
            .next()
            .expect("digit key_id has at least one char");
        calc.entry_buf.push(ch);
        let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
        return Ok(CalcStateView::from_state(calc, print_lines));
    }

    // ── '.' — block duplicate '.' and '.' after 'e' ──────────────────────────
    if key_id == "." {
        if !calc.entry_buf.contains('.') && !calc.entry_buf.contains('e') {
            calc.entry_buf.push('.');
        }
        let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
        return Ok(CalcStateView::from_state(calc, print_lines));
    }

    // ── 'e' (EEX) — implicit "1" mantissa on empty entry_buf (Phase 9 D-07) ──
    if key_id == "e" {
        if !calc.entry_buf.contains('e') {
            if calc.entry_buf.is_empty() {
                calc.entry_buf.push_str("1e");
            } else {
                calc.entry_buf.push('e');
            }
        }
        let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
        return Ok(CalcStateView::from_state(calc, print_lines));
    }

    // ── Named / parameterized op — resolve and dispatch ──────────────────────
    let op = key_map::resolve(key_id)?;
    dispatch(calc, op).map_err(GuiError::from)?;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}

/// Pure-Rust helper for get_state — drains print_buffer and builds a CalcStateView.
///
/// Returns Result for symmetry with handle_op (and to allow future error paths) even
/// though the current implementation is infallible.
pub fn handle_get_state(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
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
