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

use crate::prgm_display;
use hp41_core::{format_alpha, format_hpnum, AngleMode, CalcState, HpError};
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
    pub y_str: String,     // Phase 15 D-01: stack Y register
    pub z_str: String,     // Phase 15 D-01: stack Z register
    pub t_str: String,     // Phase 15 D-01: stack T register
    pub lastx_str: String, // Phase 15 D-01: LASTX register
    pub in_eex_mode: bool, // Phase 15 D-02: entry_buf.contains('e')
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>,
    pub program_steps: Vec<String>, // Phase 18 D-01: pre-formatted step strings
    pub pc: usize,                  // Phase 18 D-01: current program counter
}

impl CalcStateView {
    /// Build a CalcStateView from CalcState + already-drained print lines.
    ///
    /// IMPORTANT: caller MUST drain `state.print_buffer` BEFORE calling this function
    /// (drain takes &mut, then this function takes &). See RESEARCH.md Pitfall 1.
    pub fn from_state(state: &CalcState, print_lines: Vec<String>) -> Self {
        // display_str priority chain (D-01 + Claude's Discretion):
        //   1. entry_buf (when user is typing)
        //   2. alpha_reg via format_alpha (when alpha_mode is on)
        //   3. format_hpnum(stack.x, display_mode) (default)
        let display_str = if !state.entry_buf.is_empty() {
            state.entry_buf.clone()
        } else if state.alpha_mode {
            format_alpha(&state.alpha_reg)
        } else {
            format_hpnum(&state.stack.x, &state.display_mode)
        };

        // x_str is always the formatted X register — independent of entry/alpha mode.
        // Phase 15 stack panel will use this directly without re-formatting.
        let x_str = format_hpnum(&state.stack.x, &state.display_mode);

        // Phase 15 D-01: populate Y/Z/T/LASTX stack register strings for the stack panel.
        let y_str = format_hpnum(&state.stack.y, &state.display_mode);
        let z_str = format_hpnum(&state.stack.z, &state.display_mode);
        let t_str = format_hpnum(&state.stack.t, &state.display_mode);
        let lastx_str = format_hpnum(&state.stack.lastx, &state.display_mode);

        // Phase 15 D-02: in_eex_mode — true when entry_buf contains 'e' (EEX entry active).
        let in_eex_mode = state.entry_buf.contains('e');

        let annunciators = Annunciators {
            user: state.user_mode,
            prgm: state.prgm_mode,
            alpha: state.alpha_mode,
            rad: state.angle_mode == AngleMode::Rad,
            grad: state.angle_mode == AngleMode::Grad,
        };

        // Phase 18 D-01: populate program_steps and pc for the program listing panel.
        let program_steps = prgm_display::format_all_steps(state);
        let pc = state.pc;

        CalcStateView {
            display_str,
            x_str,
            y_str,
            z_str,
            t_str,
            lastx_str,
            in_eex_mode,
            annunciators,
            print_lines,
            program_steps,
            pc,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GuiError {
    pub message: String,
}

impl From<HpError> for GuiError {
    fn from(e: HpError) -> Self {
        // HpError uses #[derive(thiserror::Error)] with #[error("...")] attrs.
        // .to_string() yields the literal message ("overflow", "divide by zero", etc.).
        GuiError {
            message: e.to_string(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::CalcState;

    #[test]
    fn test_dispatch_op_payload_size() {
        // SC-1: empty program baseline — program_steps adds ["000 END"] (~35 bytes). Real programs grow beyond this limit.
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![]);
        let json = serde_json::to_string(&view).unwrap();
        assert!(
            json.len() <= 400,
            "CalcStateView JSON (empty program) must be ≤400 bytes, got {} bytes: {}",
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

    #[test]
    fn test_phase15_stack_fields_exist() {
        // Wave 0 RED test: CalcStateView must have y_str, z_str, t_str, lastx_str,
        // and in_eex_mode after Phase 15 types.rs is updated.
        // This test compiles only after Wave 1 adds these fields to the struct.
        let mut state = CalcState::new();
        state.entry_buf = "1e2".to_string();
        let view = CalcStateView::from_state(&state, vec![]);
        // y/z/t/lastx start as "0.0000000000" (format_hpnum of zero stack)
        assert!(!view.y_str.is_empty(), "y_str must be populated");
        assert!(!view.z_str.is_empty(), "z_str must be populated");
        assert!(!view.t_str.is_empty(), "t_str must be populated");
        assert!(!view.lastx_str.is_empty(), "lastx_str must be populated");
        // in_eex_mode: entry_buf "1e2" contains 'e' → true
        assert!(
            view.in_eex_mode,
            "in_eex_mode must be true when entry_buf contains 'e'"
        );
    }

    #[test]
    fn test_in_eex_mode_false_without_e() {
        let mut state = CalcState::new();
        state.entry_buf = "42".to_string();
        let view = CalcStateView::from_state(&state, vec![]);
        assert!(
            !view.in_eex_mode,
            "in_eex_mode must be false when entry_buf has no 'e'"
        );
    }

    #[test]
    fn test_phase18_fields_exist() {
        // Wave 0 RED: CalcStateView must have program_steps: Vec<String> and pc: usize
        // after Phase 18 Plan 02 adds these fields. This test will fail until Plan 02 runs.
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![]);
        assert_eq!(
            view.program_steps,
            vec!["000 END"],
            "empty program must produce program_steps = [\"000 END\"]"
        );
        assert_eq!(view.pc, 0, "fresh CalcState pc must be 0");
    }
}
