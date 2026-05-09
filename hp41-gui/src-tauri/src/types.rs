//! Phase 14 IPC Layer — DTO types sent from Tauri commands to the React frontend.
//!
//! - `CalcStateView` — lean (~200 bytes) snapshot of CalcState (display_str, x_str,
//!   annunciators, drained print_lines). Built via `from_state(&CalcState, Vec<String>)`.
//! - `Annunciators` — five booleans (user, prgm, alpha, rad, grad) derived from CalcState.
//! - `GuiError` — minimal serializable error returned to the frontend; converts from HpError.
//!
//! Decisions: D-01..D-03 (CalcStateView shape), D-10..D-11 (GuiError shape).
//! Phase 14 design: types.rs has zero side effects — only struct definitions and a
//! pure constructor that reads CalcState fields.
//!
//! Wave 0 status: scaffolds + RED tests. `from_state` and `From<HpError>` are
//! `unimplemented!()` so tests fail predictably until Wave 1 fills in real logic.

use hp41_core::HpError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Annunciators {
    pub user: bool,
    pub prgm: bool,
    pub alpha: bool,
    pub rad: bool,
    pub grad: bool,
}

#[derive(Debug, Serialize)]
pub struct CalcStateView {
    pub display_str: String,
    pub x_str: String,
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>,
}

impl CalcStateView {
    /// Build a CalcStateView from CalcState + already-drained print lines.
    ///
    /// IMPORTANT: caller MUST drain `state.print_buffer` BEFORE calling this function
    /// (drain takes &mut, then this function takes &). See RESEARCH.md Pitfall 1.
    pub fn from_state(_state: &hp41_core::CalcState, _print_lines: Vec<String>) -> Self {
        unimplemented!("Wave 1 (Plan 01): build CalcStateView from CalcState fields per D-01")
    }
}

#[derive(Debug, Serialize)]
pub struct GuiError {
    pub message: String,
}

impl From<HpError> for GuiError {
    fn from(_e: HpError) -> Self {
        unimplemented!("Wave 1 (Plan 01): convert HpError to GuiError via to_string()")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::CalcState;

    #[test]
    fn test_dispatch_op_payload_size() {
        // SC-1: CalcStateView JSON serialization must be ≤ 300 bytes.
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![]);
        let json = serde_json::to_string(&view).unwrap();
        assert!(
            json.len() <= 300,
            "CalcStateView JSON must be ≤300 bytes, got {} bytes: {}",
            json.len(),
            json
        );
    }

    #[test]
    fn test_calc_state_view_structure() {
        // entry_buf priority 1: when non-empty, display_str equals entry_buf verbatim.
        let mut state = CalcState::new();
        state.entry_buf = "42".to_string();
        let view = CalcStateView::from_state(&state, vec![]);
        assert_eq!(view.display_str, "42");
    }

    #[test]
    fn test_annunciators_from_state() {
        // Fresh CalcState defaults: angle_mode=Deg, all mode flags false.
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![]);
        assert!(!view.annunciators.user);
        assert!(!view.annunciators.prgm);
        assert!(!view.annunciators.alpha);
        assert!(!view.annunciators.rad);
        assert!(!view.annunciators.grad);
    }

    #[test]
    fn test_gui_error_from_hp_error() {
        // HpError::Overflow has #[error("overflow")] — to_string() yields "overflow".
        let err: GuiError = HpError::Overflow.into();
        assert_eq!(err.message, "overflow");
    }
}
