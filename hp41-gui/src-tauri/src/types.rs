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
//! Phase 31 Plan 05: LCD-alternation routing (D-31.5 / D-31.6).
//! LCD_WIDTH / CONTINUATION / truncate_with_continuation live here so they are
//! co-located with CalcStateView::from_state (the sole caller). The routing
//! lives in from_state as a new 4th branch at the TOP of the display_str priority
//! chain — placed BEFORE the existing `if !state.entry_buf.is_empty()` branch.
//!
//! IMPORTANT: display_override is RESERVED for Phase 21 VIEW/AVIEW/PROMPT/CLD
//! per hp41-core/src/state.rs and is cleared at the top of dispatch. Modal-prompt
//! routing must NOT go through display_override (collision risk). The from_state
//! branch is the correct implementation location.

/// HP-41C LCD character width (12 characters per real hardware display row).
const LCD_WIDTH: usize = 12;

/// HP-41 standard continuation marker (U+2261 IDENTICAL TO) — shown as the
/// 12th character when a prompt is longer than LCD_WIDTH. Hardware-faithful
/// per D-31.6.
const CONTINUATION: char = '\u{2261}'; // ≡

/// Truncate a string to LCD_WIDTH characters. If the string fits, returns it
/// unchanged. If it is longer, returns the first (LCD_WIDTH - 1) characters
/// followed by CONTINUATION (≡).
///
/// Uses Unicode-correct char iteration (NOT byte indexing) so multi-byte
/// characters are handled safely.
fn truncate_with_continuation(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= LCD_WIDTH {
        return s.to_string();
    }
    let mut result: String = chars.iter().take(LCD_WIDTH - 1).collect();
    result.push(CONTINUATION);
    result
}

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
    // Phase 26 D-26.11: HP-41 ASN key assignments for USER-mode relabel.
    // Sorted by key_code for deterministic JSON output; sourced from
    // `state.assignments: BTreeMap<u8, String>`. Plan 26-03 consumes this
    // in `<Keyboard userKeymap={...} />`.
    pub user_keymap: Vec<(u8, String)>,
    // Phase 26 D-26.11: set-flag indices (Vec<u8>) projected from the u64
    // bitfield `state.flags`. Vec representation keeps the JSON budget small
    // for typical "0-3 flags set" workloads (empty: 11 bytes; 3 flags:
    // ~18 bytes vs. raw u64 = 8 bytes serialized as "1234567890" decimal).
    pub flags: Vec<u8>,
    // Phase 26 D-26.11: surface state.display_override (Phase 21) so the
    // frontend can show VIEW / AVIEW / PROMPT messages. None = no override
    // active; render normal display.
    pub display_override: Option<String>,
    // Phase 26 D-26.11: drained sound-event lines (BEEP / TONE n).
    // Caller drains `state.event_buffer` BEFORE calling `from_state`,
    // mirroring the `print_lines` drain pattern (Pitfall 1 from
    // hp41-gui/src-tauri/src/types.rs line 44 — &mut → & cannot interleave).
    pub event_buffer: Vec<String>,
    // Phase 31 Plan 03: modal workflow state fields for D-31.1 / D-31.2 dispatch routing.
    // is_running: mirrors CalcState.is_running for R/S 3-way and Esc-cancel routing.
    pub is_running: bool,
    // modal_program_active: true when CalcState.modal_program.is_some() — avoids
    // sending the full ModalProgram enum over IPC (SC-4 / D-25.6 parity: no GUI-only logic).
    pub modal_program_active: bool,
    // modal_requires_alpha_label: true when the active modal step expects XEQ-by-name input
    // (FUNCTION NAME? prompt). Drives the post-dispatch auto-open of XeqByName{CollectForModal}
    // in App.tsx (D-29.9 mirror).
    pub modal_requires_alpha_label: bool,
    // modal_prompt: cloned from CalcState.modal_prompt for debug + accessibility.
    // Also used by Display14Seg LCD-alternation routing (D-31.5) in Phase 31 Plan 05.
    pub modal_prompt: Option<String>,
}

impl CalcStateView {
    /// Build a CalcStateView from CalcState + already-drained print + event lines.
    ///
    /// IMPORTANT: caller MUST drain `state.print_buffer` AND `state.event_buffer`
    /// BEFORE calling this function (both drains take `&mut`, then `from_state`
    /// takes `&`). See RESEARCH.md Pitfall 1. Phase 26 D-26.11 added the
    /// `event_lines` parameter alongside the pre-existing `print_lines`.
    pub fn from_state(
        state: &CalcState,
        print_lines: Vec<String>,
        event_lines: Vec<String>,
    ) -> Self {
        // display_str priority chain (D-01 + Claude's Discretion + D-31.5):
        //   0. [NEW Phase 31 Plan 05] modal_prompt truncated when modal is active
        //      AND entry_buf is empty AND modal_prompt is set → LCD-alternation
        //      (D-31.5 / D-31.6). Placed BEFORE entry_buf priority so the prompt
        //      shows while waiting for user input; once the user starts typing,
        //      entry_buf is non-empty and that branch wins instead (live feedback).
        //      NOTE: display_override is RESERVED for Phase 21 VIEW/AVIEW/PROMPT/CLD
        //      and must NOT be used here — see module-level doc comment.
        //   1. entry_buf (when user is typing — overrides modal prompt)
        //   2. alpha_reg via format_alpha (when alpha_mode is on)
        //   3. format_hpnum(stack.x, display_mode) (default)
        let display_str = if state.modal_program.is_some()
            && state.entry_buf.is_empty()
            && state.modal_prompt.is_some()
        {
            truncate_with_continuation(
                state
                    .modal_prompt
                    .as_ref()
                    .expect("modal_prompt set in guard above"),
            )
        } else if !state.entry_buf.is_empty() {
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

        // Phase 26 D-26.11: project user_keymap from state.assignments
        // (BTreeMap<u8, String>) into a Vec for serialization. BTreeMap already
        // iterates by sorted key — collect preserves that determinism.
        let user_keymap: Vec<(u8, String)> = state
            .assignments
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        // Phase 26 D-26.11: project state.flags (u64 bitfield) into the set
        // of set-flag indices 0..=55 (HP-41 user flags 0-29 + system flags 30-55).
        // Vec<u8> is smaller than a raw u64 in JSON for the typical "0-3 flags
        // set" workload.
        let flags: Vec<u8> = (0u8..=55)
            .filter(|i| (state.flags >> i) & 1 == 1)
            .collect();

        // Phase 26 D-26.11: surface display_override; clone the Option<String>.
        let display_override = state.display_override.clone();

        // Phase 31 Plan 03: project modal workflow state fields.
        let is_running = state.is_running;
        let modal_program_active = state.modal_program.is_some();
        let modal_requires_alpha_label = state
            .modal_program
            .as_ref()
            .map(|m| m.requires_alpha_label())
            .unwrap_or(false);
        let modal_prompt = state.modal_prompt.clone();

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
            user_keymap,
            flags,
            display_override,
            event_buffer: event_lines,
            is_running,
            modal_program_active,
            modal_requires_alpha_label,
            modal_prompt,
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
        // EXCEPTION: HpError::Canceled Display returns lowercase "canceled" but
        // UI-SPEC requires uppercase "CANCELED" (Phase 31 Plan 03 / Pitfall 4).
        let message = match e {
            HpError::Canceled => "CANCELED".to_string(),
            other => other.to_string(),
        };
        GuiError { message }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::CalcState;

    #[test]
    fn test_dispatch_op_payload_size() {
        // SC-1 + Phase 26 FN-GUI-05: empty program baseline — program_steps adds
        // ["000 END"] (~35 bytes); the 4 new D-26.11 projections add ~60 bytes
        // for empty/None defaults. Budget raised from 400 to 500 bytes per
        // D-26.11. Real programs grow beyond this limit.
        // Phase 31 Plan 03: 4 new modal fields add ~100 bytes for empty/false/None
        // defaults. Per RESEARCH Pitfall 10: 337 + 100 = ~437 bytes, 63-byte headroom.
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        let json = serde_json::to_string(&view).unwrap();
        // Phase 26 measured baseline: 337 bytes. Phase 31 adds ~100 bytes for modal fields.
        // Combined budget: <= 500 bytes (63-byte headroom at ~437 bytes).
        assert!(
            json.len() <= 500,
            "CalcStateView JSON (empty program + empty assignments + no flags) must be ≤500 bytes, got {} bytes: {}",
            json.len(),
            json
        );
    }

    #[test]
    fn test_dispatch_op_payload_size_with_realistic_load() {
        // Phase 26 D-26.11: budget must hold with a realistic load — ~5 ASN
        // assignments + 3 set flags. Verifies the new projections don't blow
        // the budget in real-world usage.
        // Phase 31 Plan 03: 4 new modal fields add ~103 bytes (measured). The
        // realistic-load budget is raised from 500 to 600 bytes to accommodate
        // the new fields. The empty-program budget stays at 500 bytes.
        let mut state = CalcState::new();
        state.assignments.insert(11, "SIN".to_string());
        state.assignments.insert(12, "COS".to_string());
        state.assignments.insert(21, "TEST".to_string());
        state.assignments.insert(22, "MYPRG".to_string());
        state.assignments.insert(33, "SUB".to_string());
        state.flags = (1u64 << 5) | (1u64 << 10) | (1u64 << 22); // flags 5, 10, 22 set
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        let json = serde_json::to_string(&view).unwrap();
        // Phase 26 measured load: 401 bytes; Phase 31 adds ~103 bytes → ~504 bytes.
        // Budget set to 600 bytes with headroom for future fields.
        assert!(
            json.len() <= 600,
            "CalcStateView JSON (realistic ASN+flag load) must be ≤600 bytes, got {} bytes: {}",
            json.len(),
            json
        );
    }

    #[test]
    fn test_calc_state_view_structure() {
        // entry_buf priority 1: when non-empty, display_str equals entry_buf verbatim.
        let mut state = CalcState::new();
        state.entry_buf = "42".to_string();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert_eq!(view.display_str, "42");
    }

    #[test]
    fn test_annunciators_from_state() {
        // Fresh CalcState defaults: angle_mode=Deg, all mode flags false.
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert!(!view.annunciators.user);
        assert!(!view.annunciators.prgm);
        assert!(!view.annunciators.alpha);
        assert!(!view.annunciators.rad);
        assert!(!view.annunciators.grad);
    }

    /// Phase 26 D-26.11: user_keymap projection is deterministic (BTreeMap
    /// order) and surfaces every ASN entry.
    #[test]
    fn test_user_keymap_projection() {
        let mut state = CalcState::new();
        state.assignments.insert(22, "TEST".to_string());
        state.assignments.insert(11, "SIN".to_string());
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        // BTreeMap iterates by sorted key — 11 must come before 22.
        assert_eq!(
            view.user_keymap,
            vec![(11, "SIN".to_string()), (22, "TEST".to_string())]
        );
    }

    /// Phase 26 D-26.11: flags projection extracts set-flag indices from the
    /// u64 bitfield. Empty (flags=0) → empty Vec.
    #[test]
    fn test_flags_projection() {
        let mut state = CalcState::new();
        state.flags = 0;
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert!(view.flags.is_empty(), "flags=0 must project to empty Vec");

        state.flags = (1u64 << 5) | (1u64 << 12) | (1u64 << 30);
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert_eq!(view.flags, vec![5, 12, 30]);
    }

    /// Phase 26 D-26.11: event_buffer comes from the caller-drained Vec,
    /// NOT from state.event_buffer directly (Pitfall 1 — drain-before-from_state).
    #[test]
    fn test_event_buffer_passed_through() {
        let state = CalcState::new();
        let event_lines = vec!["BEEP".to_string(), "TONE 5".to_string()];
        let view = CalcStateView::from_state(&state, vec![], event_lines.clone());
        assert_eq!(view.event_buffer, event_lines);
    }

    #[test]
    fn test_gui_error_from_hp_error() {
        // HpError::Overflow has #[error("overflow")] — to_string() yields "overflow".
        let err: GuiError = HpError::Overflow.into();
        assert_eq!(err.message, "overflow");
    }

    /// Phase 31 Plan 03 / Pitfall 4: HpError::Canceled must map to UPPERCASE "CANCELED"
    /// (not the lowercase "canceled" that .to_string() yields from the #[error] attribute).
    /// UI-SPEC mandates uppercase per the "CANCELED" display requirement.
    #[test]
    fn test_canceled_maps_to_uppercase() {
        let err: GuiError = HpError::Canceled.into();
        assert_eq!(
            err.message,
            "CANCELED",
            "HpError::Canceled must map to 'CANCELED' (uppercase per UI-SPEC)"
        );
    }

    /// Phase 31 Plan 03: CalcStateView now has 4 new modal fields.
    /// Verify they project correctly from a fresh CalcState.
    #[test]
    fn test_modal_fields_default_projection() {
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert!(!view.is_running, "fresh state: is_running must be false");
        assert!(
            !view.modal_program_active,
            "fresh state: modal_program_active must be false"
        );
        assert!(
            !view.modal_requires_alpha_label,
            "fresh state: modal_requires_alpha_label must be false"
        );
        assert!(
            view.modal_prompt.is_none(),
            "fresh state: modal_prompt must be None"
        );
    }

    #[test]
    fn test_phase15_stack_fields_exist() {
        // Wave 0 RED test: CalcStateView must have y_str, z_str, t_str, lastx_str,
        // and in_eex_mode after Phase 15 types.rs is updated.
        // This test compiles only after Wave 1 adds these fields to the struct.
        let mut state = CalcState::new();
        state.entry_buf = "1e2".to_string();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
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
        let view = CalcStateView::from_state(&state, vec![], vec![]);
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
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert_eq!(
            view.program_steps,
            vec!["000 END"],
            "empty program must produce program_steps = [\"000 END\"]"
        );
        assert_eq!(view.pc, 0, "fresh CalcState pc must be 0");
    }
}
