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

    // ── "eex_chs" — toggle exponent sign in entry_buf (Phase 15 D-06) ────────────
    // Source: hp41-cli/src/app.rs lines 389-404 (entry_buf direct mutation, no dispatch).
    // MUST come before key_map::resolve() — no Op::EexChs variant exists.
    if key_id == "eex_chs" {
        if let Some(e_pos) = calc.entry_buf.find('e') {
            let after_e = &calc.entry_buf[e_pos + 1..];
            if after_e.starts_with('-') {
                // Remove minus: "1e-2" → "1e2", "1e-" → "1e"
                calc.entry_buf.remove(e_pos + 1);
            } else {
                // Insert minus: "1e2" → "1e-2", "1e" → "1e-"
                calc.entry_buf.insert(e_pos + 1, '-');
            }
        }
        // No-op if entry_buf has no 'e' — React guards this but Rust is defensive.
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

/// Tauri command: step the program counter forward by 1 (SST — Single Step).
/// Mirrors HP-41 hardware: pc stays at program.len(), no wrap-around.
#[tauri::command]
pub fn sst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_sst(&mut calc)
}

/// Tauri command: step the program counter backward by 1 (BST — Back Step).
/// Mirrors HP-41 hardware: pc stays at 0, no underflow.
#[tauri::command]
pub fn bst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_bst(&mut calc)
}

/// Tauri command: toggle program run/stop state (R/S key).
///
/// Mirrors sst_step/bst_step shape — never goes through dispatch_op because
/// R/S is not a single Op variant (it toggles CalcState.is_running).
/// v2.1 scope: only toggles the flag. Actual stepping is deferred to v2.2
/// once a tick thread lands (today the IPC thread cannot block on a run loop).
#[tauri::command]
pub fn run_stop(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_run_stop(&mut calc)
}

/// Pure-Rust helper for sst_step — unit-testable without a Tauri runtime.
/// Advances pc by 1, capped at program.len() (no wrap-around, matching HP-41 hardware behavior).
pub fn handle_sst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    if calc.pc < calc.program.len() {
        calc.pc += 1;
    }
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}

/// Pure-Rust helper for bst_step — decrements pc via saturating_sub, clamped at 0.
pub fn handle_bst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.pc = calc.pc.saturating_sub(1);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines))
}

/// Pure-Rust helper for run_stop — toggles `is_running`. Unit-testable without a
/// Tauri runtime. v2.1 scope: flag-toggle only; no run loop is spawned here.
pub fn handle_run_stop(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.is_running = !calc.is_running;
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

    /// PR #5 review (pr-test-analyzer) — the existing test_print_buffer_drained
    /// only exercises handle_get_state. The handle_op path (the most-trafficked
    /// dispatch_op IPC route) was never directly asserted to drain print_buffer
    /// AND populate view.print_lines. A regression that skipped the drain in
    /// handle_op would silently break the entire GUI print panel without any
    /// test signal.
    #[test]
    fn test_handle_op_drains_print_buffer_via_dispatch() {
        // PRX
        let mut calc = CalcState::new();
        calc.stack.x = HpNum::from(7);
        let view = handle_op(&mut calc, "prx").expect("handle_op prx must succeed");
        assert!(
            calc.print_buffer.is_empty(),
            "handle_op('prx') must drain print_buffer (was: {} lines)",
            calc.print_buffer.len()
        );
        assert_eq!(
            view.print_lines.len(),
            1,
            "view.print_lines must contain the PRX line after dispatch"
        );

        // PRA — exercises a different op_* helper through the same drain path
        let mut calc2 = CalcState::new();
        calc2.alpha_reg = "HELLO".to_string();
        let view2 = handle_op(&mut calc2, "pra").expect("handle_op pra must succeed");
        assert!(calc2.print_buffer.is_empty());
        assert_eq!(view2.print_lines.len(), 1);

        // PRSTK — drains 6 lines in one call
        let mut calc3 = CalcState::new();
        let view3 = handle_op(&mut calc3, "prstk").expect("handle_op prstk must succeed");
        assert!(calc3.print_buffer.is_empty());
        assert_eq!(
            view3.print_lines.len(),
            6,
            "PRSTK must drain all 6 lines (T/Z/Y/X/LASTX/ALPHA) into the view"
        );
    }

    #[test]
    fn test_eex_chs_toggles_exponent_sign() {
        // Wave 0 RED test: "eex_chs" key_id must toggle the exponent sign in entry_buf.
        // entry_buf "1e2" → "1e-2" on first call → "1e2" on second call.
        let mut calc = CalcState::new();
        calc.entry_buf = "1e2".to_string();
        handle_op(&mut calc, "eex_chs").unwrap();
        assert_eq!(calc.entry_buf, "1e-2", "first eex_chs must insert minus sign");
        handle_op(&mut calc, "eex_chs").unwrap();
        assert_eq!(calc.entry_buf, "1e2", "second eex_chs must remove minus sign");
    }

    #[test]
    fn test_eex_chs_noop_without_e() {
        // Defensive: if no 'e' in entry_buf, eex_chs must return Ok without panic or error.
        let mut calc = CalcState::new();
        calc.entry_buf = "42".to_string();
        let result = handle_op(&mut calc, "eex_chs");
        assert!(result.is_ok(), "eex_chs with no 'e' must not error or panic");
        assert_eq!(calc.entry_buf, "42", "entry_buf must be unchanged when no 'e' present");
    }

    #[test]
    fn test_handle_sst_advances_pc() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add, Op::Enter];
        calc.pc = 0;
        handle_sst(&mut calc).unwrap();
        assert_eq!(calc.pc, 1, "SST must advance pc from 0 to 1");
    }

    #[test]
    fn test_handle_sst_clamps_at_end() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add];
        calc.pc = 1; // already at end (program.len() == 1)
        handle_sst(&mut calc).unwrap();
        assert_eq!(calc.pc, 1, "SST must not advance past program.len()");
    }

    #[test]
    fn test_handle_bst_decrements_pc() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add];
        calc.pc = 1;
        handle_bst(&mut calc).unwrap();
        assert_eq!(calc.pc, 0, "BST must decrement pc from 1 to 0");
    }

    #[test]
    fn test_handle_bst_clamps_at_zero() {
        let mut calc = CalcState::new();
        calc.pc = 0;
        handle_bst(&mut calc).unwrap();
        assert_eq!(calc.pc, 0, "BST must not underflow below 0");
    }

    #[test]
    fn test_handle_run_stop_toggles_is_running() {
        let mut calc = CalcState::new();
        assert!(!calc.is_running, "fresh CalcState must start with is_running == false");
        handle_run_stop(&mut calc).unwrap();
        assert!(calc.is_running, "first run_stop must flip is_running to true");
        handle_run_stop(&mut calc).unwrap();
        assert!(!calc.is_running, "second run_stop must flip is_running back to false");
    }

    /// Smoke test for Op::PctChange through the Tauri command path.
    /// Y=100 (base), X=125 (new value) → %CH = (125-100)/100 * 100 = 25; Y preserved at 100.
    /// Seeds chosen to yield an exact integer result. Parses x_str/y_str back to Decimal
    /// so the comparison is independent of rust_decimal's trailing-zero scale.
    #[test]
    fn handle_op_pct_change_produces_expected_view() {
        use rust_decimal::Decimal;
        let mut state = CalcState::new();
        state.stack.y = HpNum::from(100i32);
        state.stack.x = HpNum::from(125i32);
        let view = handle_op(&mut state, "pct_change")
            .expect("pct_change must dispatch successfully");

        let x_val: Decimal = view
            .x_str
            .parse()
            .expect("x_str must parse as Decimal");
        let y_val: Decimal = view
            .y_str
            .parse()
            .expect("y_str must parse as Decimal");
        assert_eq!(x_val, Decimal::from(25), "%CH(100→125) must be 25");
        assert_eq!(y_val, Decimal::from(100), "Y must be preserved at 100");
    }
}
