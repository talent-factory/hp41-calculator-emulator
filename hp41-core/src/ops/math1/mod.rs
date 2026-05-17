// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `math1` — XROM framework and Math Pac I (HP 00041-90034, 1979) operations.
//!
//! Module structure:
//! - `xrom`: XromModule registry, `xrom_resolve()` entry point, `MATH_1` const
//! - `modal`: ModalProgram state-machine enum for prompt-driven workflows
//! - `integ`: IntegState placeholder (Plan 28-07 fills)
//! - `solve`: SolveState placeholder (Plan 28-08 fills)
//! - `difeq`: DifeqState placeholder (Plan 28-09 fills)

pub mod complex;
pub mod difeq;
pub mod four;
pub mod hyperbolics;
pub mod integ;
pub mod matrix;
pub mod modal;
pub mod poly;
pub mod solve;
pub mod trans;
pub mod tri;
pub mod xrom;

pub use modal::ModalProgram;

use crate::error::HpError;
use crate::state::CalcState;

// ── Phase 29 / CLI-05 additive public surface — D-29.5 / D-29.6 / D-29.7 ────

/// Submit a numeric R/S input to the currently active modal workflow.
///
/// Flushes `state.entry_buf` (via `flush_entry_buf`) first, then dispatches to
/// the per-program `submit_step` based on `state.modal_program`. Returns
/// `Err(HpError::InvalidOp)` if no modal is active.
///
/// Used by the CLI R/S interceptor (hp41-cli `handle_key` F5 path when modal
/// is active — D-29.5) and will be reused identically by hp41-gui Phase 31
/// (D-25.6 CLI ↔ GUI parity).
///
/// Phase 29 / CLI-05 additive public surface — D-29.5.
pub fn submit_modal(state: &mut CalcState) -> Result<(), HpError> {
    use modal::ModalProgram;

    // WR-01 fix: verify a modal is active BEFORE mutating state via
    // flush_entry_buf. Previously the flush ran unconditionally — if a
    // non-CLI caller (e.g., a future GUI binding) invoked submit_modal
    // without an open modal, the user's typed digits got pushed onto the
    // stack with lift enabled and only THEN did the function return
    // InvalidOp, leaving the stack visibly mutated for no semantic gain.
    let modal = match state.modal_program.clone() {
        Some(m) => m,
        None => return Err(HpError::InvalidOp),
    };

    // Flush entry_buf to X register so submit_step reads the numeric input.
    // Only executed after the modal-active guard above passes.
    crate::ops::flush_entry_buf(state)?;

    match modal {
        ModalProgram::Matrix(step) => matrix::submit_step(state, step),
        ModalProgram::Solve(step) => solve::submit_step(state, step),
        ModalProgram::Poly(step) => poly::submit_step(state, step),
        ModalProgram::Integ(step) => integ::submit_step(state, step),
        ModalProgram::Difeq(step) => difeq::submit_step(state, step),
        ModalProgram::Four(step) => four::submit_step(state, step),
        ModalProgram::Trans(step) => trans::submit_step(state, step),
    }
}

/// Cancel the currently active modal workflow.
///
/// Clears `modal_program`, `modal_prompt`, and `entry_buf`. Leaves the stack and
/// all matrix/solver state untouched. No error path (always succeeds).
///
/// Used by the CLI Esc interceptor (D-29.6) and will be reused identically by
/// hp41-gui Phase 31 (D-25.6 CLI ↔ GUI parity).
///
/// Phase 29 / CLI-05 additive public surface — D-29.6.
pub fn cancel_modal(state: &mut CalcState) {
    state.modal_program = None;
    state.modal_prompt = None;
    state.entry_buf.clear();
}

/// Submit an alpha label to the currently active modal workflow (for FunctionNamePrompt steps).
///
/// Trims and uppercases the label, writes it to `state.alpha_reg`, then dispatches to the
/// per-program `submit_label_step` for the three user-callback programs (Integ, Solve, Difeq).
/// Returns `Err(HpError::InvalidOp)` if no modal is active or if the current step is not a
/// FunctionNamePrompt (doc comment: "unreachable in well-formed flow per D-29.9 gate").
///
/// Used by the CLI XeqByName{CollectForModal} Enter arm (D-29.7 / D-29.8) and will be
/// reused identically by hp41-gui Phase 31 (D-25.6 CLI ↔ GUI parity).
///
/// Phase 29 / CLI-05 additive public surface — D-29.7.
pub fn submit_modal_with_label(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    use modal::{DifeqInputStep, IntegInputStep, ModalProgram, SolveInputStep};

    // WR-02 fix: verify the modal is in a label-accepting state BEFORE
    // mutating state.alpha_reg. Previously alpha_reg was overwritten with
    // `upper` unconditionally, even when no modal was active or when the
    // current step was not one of the three FunctionNamePrompt variants —
    // silently clobbering the user's ALPHA register contents.
    let modal = match state.modal_program.clone() {
        Some(m) if m.requires_alpha_label() => m,
        Some(_) => return Err(HpError::InvalidOp),
        None => return Err(HpError::InvalidOp),
    };

    let upper = label.trim().to_ascii_uppercase();
    state.alpha_reg = upper;

    match modal {
        ModalProgram::Solve(SolveInputStep::FunctionNamePrompt) => {
            solve::submit_label_step(state)
        }
        ModalProgram::Integ(IntegInputStep::FunctionNamePrompt) => {
            integ::submit_label_step(state)
        }
        ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt) => {
            difeq::submit_label_step(state)
        }
        // Unreachable: requires_alpha_label() guarantees one of the three
        // FunctionNamePrompt variants above (D-29.7 / D-29.9). Defensive arm
        // preserved to satisfy match exhaustiveness without `_ =>` weakening.
        _ => Err(HpError::InvalidOp),
    }
}
