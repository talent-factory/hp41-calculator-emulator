//! Shared calculator state projection for all HP-41 application shells.
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
use hp41_core::ops::time::clock::get_clock_display_str;
use hp41_core::ops::time::stopwatch::get_stopwatch_display_str;
use hp41_core::state::{YieldKind, YieldState};
use hp41_core::{format_alpha, format_hpnum, format_hpnum_lcd, AngleMode, CalcState, HpError};
use serde::Serialize;

/// Serializable snapshot of `YieldState` sent to the TS layer when the core
/// run_loop breaks at a PSE / VIEW / AVIEW yield.
///
/// `kind` is a lowercase string so the TS layer needs no Rust enum knowledge:
/// `"pse"` / `"view"` / `"aview"`.
/// `resume_ms` is the milliseconds the frontend must wait before calling
/// `resume_program` (single-source-of-truth: `hp41_core::state::PSE_RESUME_MS`).
#[derive(Debug, Serialize)]
pub struct YieldView {
    pub kind: String,
    pub text: String,
    pub resume_ms: u64,
}

impl YieldView {
    fn from_yield_state(y: &YieldState) -> Self {
        let kind = match y.kind {
            YieldKind::Pse => "pse",
            YieldKind::View => "view",
            YieldKind::Aview => "aview",
            YieldKind::WaitForKey => "wait_for_key", // Phase 64 — event-driven key capture
        }
        .to_string();
        YieldView {
            kind,
            text: y.text.clone(),
            resume_ms: y.resume_ms,
        }
    }
}

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
    // this module's `from_state` contract — &mut → & cannot interleave).
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
    // Phase 41 D-41.3: live-display trigger fields projected from CalcState transient booleans.
    // Frontend starts setInterval(100ms) when clock_active || stopwatch_keyboard_mode is true,
    // clears the interval when both are false (D-41.8).
    // Both fields are #[serde(default, skip)] in CalcState (transient — not persisted).
    pub clock_active: bool,
    pub stopwatch_keyboard_mode: bool,
    // stopwatch_running: true when stopwatch_mode == Running; frontend uses this to
    // decide Space → RUNSW (start) vs STOPSW (stop) in stopwatch keyboard mode.
    pub stopwatch_running: bool,
    // Phase 63 D-04 / PRGM-01/PRGM-02: yield channel from the run_loop.
    // Some when run_loop broke at a PSE/VIEW/AVIEW yield; None on normal stop/end/error.
    // Carries kind (lowercase "pse"/"view"/"aview"), pre-formatted text, and resume_ms
    // so the TS driver can render the yield display and schedule resume_program.
    // display_override is NOT written by yield paths (D-04); DISP-01 resolved in Phase 65.
    pub pending_yield: Option<YieldView>,
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
        // display_str priority chain (D-01 + D-31.5 + D-39.1):
        //   -1. clock/stopwatch live display (highest priority, mirrors CLI ui.rs)
        //   0. modal_prompt LCD-alternation (D-31.5 / D-31.6) — GUI-only, gated disjointly
        //   1. entry_buf (when user is typing)
        //   2. prgm_mode: current program step via prgm_display::format_step (D-14)
        //      Parity with CLI hp41-cli/src/ui.rs get_display_string (D-25.6):
        //      clock -> stopwatch -> [modal_prompt] -> entry_buf -> prgm_mode -> alpha -> X
        //   3. alpha_reg via format_alpha
        //   4. format_hpnum(stack.x, display_mode) (default)
        let display_str = if let Some(s) = get_clock_display_str(state) {
            s
        } else if let Some(s) = get_stopwatch_display_str(state) {
            s
        } else if let Some(prompt) = state
            .modal_prompt
            .as_ref()
            .filter(|_| state.modal_program.is_some() && state.entry_buf.is_empty())
        {
            truncate_with_continuation(prompt)
        } else if !state.entry_buf.is_empty() {
            state.entry_buf.clone()
        } else if state.prgm_mode {
            prgm_display::format_step(state)
        } else if state.alpha_mode {
            format_alpha(&state.alpha_reg)
        } else if state.flags & (1u64 << 48) != 0 {
            // DISP-03 (Phase 65): AON (flag 48) — at rest, show the ALPHA register
            // instead of X. Uses raw state.flags u64 (not the projected Vec<u8>).
            // Branch sits AFTER alpha_mode so active ALPHA entry takes priority (D-10).
            format_alpha(&state.alpha_reg)
        } else {
            // Phase 65 (ADR v4.3-006): the main 14-segment display has only 12
            // cells, so large-exponent values use the authentic no-"E" LCD form.
            // The stack panel (x_str below) keeps the wide format_hpnum "E" form.
            format_hpnum_lcd(&state.stack.x, &state.display_mode)
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
        let flags: Vec<u8> = (0u8..=55).filter(|i| (state.flags >> i) & 1 == 1).collect();

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

        // Phase 41 D-41.3: live-display trigger booleans projected from transient CalcState fields.
        // Frontend uses these to start/stop the 100ms setInterval for clock/stopwatch display.
        let clock_active = state.clock_active;
        let stopwatch_keyboard_mode = state.stopwatch_keyboard_mode;
        let stopwatch_running =
            state.stopwatch_mode == hp41_core::ops::time::stopwatch::StopwatchMode::Running;

        // Phase 63 D-04 / PRGM-01/PRGM-02: project pending_yield from the run_loop yield channel.
        // None when the program ended normally / stopped / errored — the TS driver only auto-resumes
        // when Some (schedules resume_program via setInterval after resume_ms ms).
        // display_override is NOT touched here per D-04 (DISP-01 deferred).
        let pending_yield = state
            .pending_yield
            .as_ref()
            .map(YieldView::from_yield_state);

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
            clock_active,
            stopwatch_keyboard_mode,
            stopwatch_running,
            pending_yield,
        }
    }
}

/// Drain transient output channels and project the canonical application view.
pub fn drain_state_view(state: &mut CalcState) -> CalcStateView {
    let print_lines = state.print_buffer.drain(..).collect();
    let event_lines = state.event_buffer.drain(..).collect();
    CalcStateView::from_state(state, print_lines, event_lines)
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
        // Phase 41 adds ~75 bytes for clock_active + stopwatch_keyboard_mode + stopwatch_running.
        // Phase 63 adds ~20 bytes for pending_yield: null (JSON null, small budget impact).
        // Combined budget: <= 560 bytes (headroom maintained).
        assert!(
            json.len() <= 560,
            "CalcStateView JSON (empty program + empty assignments + no flags) must be ≤560 bytes, got {} bytes: {}",
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
        // Phase 41 adds ~75 bytes for clock_active + stopwatch_keyboard_mode + stopwatch_running.
        // Phase 63 adds ~20 bytes for pending_yield: null.
        // Budget set to 650 bytes with headroom for future fields.
        assert!(
            json.len() <= 650,
            "CalcStateView JSON (realistic ASN+flag load) must be ≤650 bytes, got {} bytes: {}",
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
            err.message, "CANCELED",
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

    /// quick-task 260603-o2e: prgm_mode branch added to display_str priority chain.
    /// When prgm_mode is true and entry_buf is empty, display_str must equal format_step output.
    /// When entry_buf is also non-empty (program entry in progress), entry_buf wins (mirrors CLI).
    #[test]
    fn test_prgm_mode_display_str() {
        // prgm_mode=true, entry_buf empty → display_str = format_step = "000 END"
        let mut state = CalcState::new();
        state.prgm_mode = true;
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert_eq!(
            view.display_str, "000 END",
            "prgm_mode=true + empty entry_buf must show format_step (e.g. '000 END')"
        );

        // prgm_mode=true but entry_buf non-empty → entry_buf wins (D-25.6 / CLI parity)
        state.entry_buf = "42".to_string();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert_eq!(
            view.display_str, "42",
            "prgm_mode=true + non-empty entry_buf: entry_buf must win"
        );

        // prgm_mode=false → normal X-register display
        state.prgm_mode = false;
        state.entry_buf = String::new();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        assert!(
            !view.display_str.starts_with("000"),
            "prgm_mode=false must NOT show a program step"
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

    /// Phase 64 PRGM-03-j: WaitForKey yield must project as kind "wait_for_key"
    /// with empty text (D-03: no display override during GETKEY suspend) and
    /// resume_ms == 0 (event-driven — no timer fires for this kind).
    #[test]
    fn from_state_projects_wait_for_key() {
        let mut calc = CalcState::new();
        calc.pending_yield = Some(YieldState {
            kind: YieldKind::WaitForKey,
            text: String::new(),
            resume_ms: 0,
        });
        let view = CalcStateView::from_state(&calc, vec![], vec![]);
        let py = view.pending_yield.expect("pending_yield must be Some");
        assert_eq!(
            py.kind, "wait_for_key",
            "WaitForKey must project as 'wait_for_key'"
        );
        assert_eq!(
            py.resume_ms, 0,
            "WaitForKey resume_ms must be 0 (event-driven, no timer)"
        );
        assert_eq!(
            py.text, "",
            "WaitForKey text must be empty (D-03: no display override)"
        );
    }

    /// DISP-03 (Phase 65): AON (flag 48) — at rest, GUI from_state display_str shows ALPHA register.
    /// AOFF (flag 48 cleared) reverts display_str to X format (D-09, D-10, D-25.6).
    #[test]
    fn test_aon_flag48_gui_shows_alpha_reg() {
        let mut state = CalcState::new();
        // Set flag 48 (AON) and put text in alpha_reg.
        state.flags |= 1u64 << 48;
        state.alpha_reg = "HP41".to_string();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        let expected = format_alpha(&state.alpha_reg);
        assert_eq!(
            view.display_str, expected,
            "AON (flag 48 set): from_state display_str must show alpha_reg"
        );
    }

    /// DISP-03 (Phase 65): AOFF (flag 48 cleared) — GUI from_state display_str reverts to X.
    #[test]
    fn test_aoff_gui_reverts_to_x_register() {
        let mut state = CalcState::new();
        // Set then clear flag 48.
        state.flags |= 1u64 << 48;
        state.flags &= !(1u64 << 48);
        state.alpha_reg = "HP41".to_string();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        let expected = format_hpnum(&state.stack.x, &state.display_mode);
        assert_eq!(
            view.display_str, expected,
            "AOFF (flag 48 cleared): from_state display_str must show X register"
        );
    }

    /// DISP-03 (Phase 65): alpha_mode takes precedence over AON (D-10 — alpha_mode branch first).
    #[test]
    fn test_alpha_mode_wins_over_aon_gui() {
        let mut state = CalcState::new();
        state.flags |= 1u64 << 48;
        state.alpha_mode = true;
        state.alpha_reg = "TEST".to_string();
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        let expected = format_alpha(&state.alpha_reg);
        // alpha_mode branch fires before flag-48 branch; result is the same format_alpha
        // output but the code path proves ordering is correct.
        assert_eq!(
            view.display_str, expected,
            "alpha_mode must take priority over AON flag-48 branch (D-10)"
        );
    }

    /// Phase 65 (Option A / ADR v4.3-006): a large-exponent X value must render in
    /// the GUI's 12-cell 14-segment display without an "E" and without truncation.
    /// Before the fix, display_str was `1.088886945E 28` (14 cells) and the GUI
    /// truncated the exponent.
    #[test]
    fn test_large_exponent_x_fits_12_cell_lcd() {
        use hp41_core::HpNum;
        let mut state = CalcState::new();
        // 27! = 1.088886945E28 — the value the UAT screenshot showed truncated.
        state.stack.x = HpNum::from_f64(1.088_886_945e28).expect("representable");
        let view = CalcStateView::from_state(&state, vec![], vec![]);
        let cells = view.display_str.chars().filter(|&c| c != '.').count();
        assert!(
            cells <= 12,
            "display_str must fit 12 cells, got [{}] ({cells} cells)",
            view.display_str
        );
        assert!(
            !view.display_str.contains('E'),
            "authentic HP-41 LCD has no 'E': [{}]",
            view.display_str
        );
        assert_eq!(view.display_str, "1.08888694528");
    }
}
