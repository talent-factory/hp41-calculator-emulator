//! TUI widget layout for the HP-41 calculator emulator.
//!
//! Layout (D-01/D-02):
//!   Left 55%: stack panel (T/Z/Y/X/LASTX) + display panel + annunciator bar + status bar
//!   Right 45%: key-reference panel (INPUT-01 discoverability)
//!
//! Minimum terminal size: 80×24 (D-01). Smaller terminals render an error message only.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;

use crate::help_data;
use crate::programs;

use hp41_core::{format_alpha, format_hpnum, AngleMode};

use crate::app::App;
use crate::keys::key_ref_entries;
use crate::prgm_display;

/// Render the full HP-41 TUI into `frame`. Called from App::draw() every frame.
pub fn render_ui(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // D-01: minimum size guard — 80 columns × 24 rows required.
    // If the terminal is too small, render a single error line and return.
    if area.width < 80 || area.height < 24 {
        frame.render_widget(
            Paragraph::new("Terminal too small (need 80×24 minimum)"),
            area,
        );
        return;
    }

    // Split into two columns: left 55%, right 45%.
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area);

    render_left_panel(app, frame, left);
    render_right_panel(app, frame, right);

    // Phase 5 overlays — rendered AFTER main panels for correct z-ordering (draw order = paint order).
    // Overlays cover both columns.
    if app.show_help {
        render_help_overlay(app, frame);
    }
    if app.show_programs {
        render_programs_overlay(app, frame);
    }
}

// ── Left panel ────────────────────────────────────────────────────────────────

fn render_left_panel(app: &App, frame: &mut Frame, area: Rect) {
    // Left column split (D-02):
    //   Row 0: T register     (1 line)
    //   Row 1: Z register     (1 line)
    //   Row 2: Y register     (1 line)
    //   Row 3: X register     (1 line)
    //   Row 4: LASTX          (1 line)
    //   Row 5: blank spacer   (1 line)
    //   Row 6: display panel  (3 lines — the largest/most prominent element)
    //   Row 7: annunciator bar (1 line)
    //   Row 8: status bar     (remainder)
    let [row_t, row_z, row_y, row_x, row_lastx, row_spacer, row_display, row_annunc, row_status] =
        Layout::vertical([
            Constraint::Length(1), // T
            Constraint::Length(1), // Z
            Constraint::Length(1), // Y
            Constraint::Length(1), // X
            Constraint::Length(1), // LASTX
            Constraint::Length(1), // spacer
            Constraint::Length(3), // display (prominent — 3 lines)
            Constraint::Length(1), // annunciators
            Constraint::Min(0),    // status bar (remainder)
        ])
        .areas(area);

    let _ = row_spacer; // intentionally empty

    render_stack(app, frame, row_t, row_z, row_y, row_x, row_lastx);
    render_display(app, frame, row_display);
    render_annunciators(app, frame, row_annunc);
    render_status(app, frame, row_status);
}

fn render_stack(
    app: &App,
    frame: &mut Frame,
    row_t: Rect,
    row_z: Rect,
    row_y: Rect,
    row_x: Rect,
    row_lastx: Rect,
) {
    let st = &app.state;
    let mode = &st.display_mode;

    // D-02: X/Y/Z/T each on their own labeled line, LASTX below T.
    let fmt = |label: &str, val: &hp41_core::HpNum| -> Paragraph<'static> {
        let text = format!("{label}: {}", format_hpnum(val, mode));
        Paragraph::new(text)
    };

    frame.render_widget(fmt("T", &st.stack.t), row_t);
    frame.render_widget(fmt("Z", &st.stack.z), row_z);
    frame.render_widget(fmt("Y", &st.stack.y), row_y);
    frame.render_widget(fmt("X", &st.stack.x), row_x);
    frame.render_widget(fmt("L", &st.stack.lastx), row_lastx);
}

fn render_display(app: &App, frame: &mut Frame, area: Rect) {
    // D-12 / D-14: display priority order:
    //   1. entry_buf content (live digit preview while typing)
    //   2. prgm_display::format_step() when in PRGM recording mode
    //   3. format_alpha() when in ALPHA mode
    //   4. format_hpnum(X, display_mode) — normal calculator display
    let display_str = get_display_string(app);

    let block = Block::bordered().title_top(" Display ");
    frame.render_widget(Paragraph::new(display_str).block(block), area);
}

/// Get the string to show in the HP-41 display area.
/// Priority: entry_buf > prgm step > alpha > formatted X.
fn get_display_string(app: &App) -> String {
    let st = &app.state;
    if !st.entry_buf.is_empty() {
        // Phase 9 D-01..D-04: when entry_buf is in exponent-entry mode, render
        // placeholder slots for unfilled exponent digits. Plain numeric entry
        // (no 'e') is unchanged — return verbatim.
        if st.entry_buf.contains('e') {
            format_entry_buf_display(&st.entry_buf)
        } else {
            st.entry_buf.clone()
        }
    } else if st.prgm_mode {
        // D-14: PRGM mode shows step number + op name.
        prgm_display::format_step(st)
    } else if st.alpha_mode {
        // ALPHA mode: show the ALPHA register (12-char max per format_alpha).
        format_alpha(&st.alpha_reg)
    } else {
        // Normal mode: format X register per current display mode.
        format_hpnum(&st.stack.x, &st.display_mode)
    }
}

/// Format an entry_buf string for display when it contains 'e' (exponent entry mode).
/// Implements D-01..D-04 from Phase 9 CONTEXT:
///   "1.5e"   → "1.5E_ _"   (no exponent digits typed yet)
///   "1.5e2"  → "1.5E2_"    (1 exponent digit typed, 1 underscore for pending slot)
///   "1.5e23" → "1.5E23"    (both slots filled — no underscores)
///   "1e"     → "1E_ _"     (empty-buffer EEX implicit "1" mantissa)
/// A negative exponent sign is preserved before the digits:
///   "1.5e-2"  → "1.5E-2_"
///   "1.5e-23" → "1.5E-23"
/// If the input does not contain 'e', it is returned verbatim (defensive fallback;
/// callers normally pre-check with `entry_buf.contains('e')` before calling).
fn format_entry_buf_display(s: &str) -> String {
    let Some(e_pos) = s.find('e') else {
        return s.to_string();
    };
    let mantissa = &s[..e_pos];
    let after_e = &s[e_pos + 1..];

    // Only '-' (negative exponent) is a possible sign; '+' is never appended to entry_buf.
    let (sign, digits) = if let Some(rest) = after_e.strip_prefix('-') {
        ("-", rest)
    } else {
        ("", after_e)
    };

    // Render exactly 2 slots: each typed digit fills one slot; remaining slots are "_".
    // Slots are space-separated when an underscore is present, per D-01 ("1.5E_ _").
    let typed: Vec<char> = digits.chars().take(2).collect();
    let slot_render = match typed.len() {
        0 => "_ _".to_string(),
        1 => format!("{}_", typed[0]),
        _ => format!("{}{}", typed[0], typed[1]),
    };

    format!("{mantissa}E{sign}{slot_render}")
}

fn render_annunciators(app: &App, frame: &mut Frame, area: Rect) {
    let st = &app.state;

    // Helper: return a styled Span for one annunciator — bold when active, dim when not.
    // Uses String (not &'static str) because format! is needed for the brackets.
    let ann = |label: &str, active: bool| -> Span<'static> {
        let text = format!("[{label}]");
        if active {
            Span::styled(text, Style::new().bold())
        } else {
            Span::styled(text, Style::new().dim())
        }
    };

    // D-02 annunciator bar: USER PRGM ALPHA SHIFT RAD DEG GRAD
    // Phase 25 (D-25.4 / Plan 01): SHIFT reflects `app.shift_armed` so users
    // see the one-shot f-prefix state in the same place GUI v2.1 surfaces
    // `shiftActive` (parity invariant D-25.6).
    let line = Line::from(vec![
        ann("USER", st.user_mode),
        Span::raw(" "),
        ann("PRGM", st.prgm_mode),
        Span::raw(" "),
        ann("ALPHA", st.alpha_mode),
        Span::raw(" "),
        ann("SHIFT", app.shift_armed),
        Span::raw(" "),
        ann("RAD", st.angle_mode == AngleMode::Rad),
        Span::raw(" "),
        ann("DEG", st.angle_mode == AngleMode::Deg),
        Span::raw(" "),
        ann("GRAD", st.angle_mode == AngleMode::Grad),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    // D-11: pending_input prompts override normal status message
    // D-14: ALPHA mode has a standard status message
    // Phase 29 (D-29.3): modal_prompt renders via widened pending_prompt signature.
    let base: String = if app.pending_input.is_some() || app.state.modal_prompt.is_some() {
        pending_prompt(
            app.pending_input.as_ref(),
            app.state.modal_prompt.as_deref(),
        )
    } else if app.state.alpha_mode {
        "ALPHA mode — Enter or A to exit".to_string()
    } else {
        app.message.as_deref().unwrap_or("Ready").to_string()
    };
    // Phase 25 (D-25.4 / Plan 01 / RESEARCH Open Q 5): prepend an "f→"
    // indicator when the prefix is armed AND no modal/ALPHA is active —
    // doubles the SHIFT annunciator with an inline cue right next to the
    // status text so users see the armed-prefix state without scanning
    // the whole UI.
    let text = if app.shift_armed && app.pending_input.is_none() && !app.state.alpha_mode {
        format!("f\u{2192} {base}")
    } else {
        base
    };
    frame.render_widget(Paragraph::new(text), area);
}

/// Format the status bar text for each PendingInput variant (D-11).
/// Uses {:_<2} to show placeholder underscores for accumulator length.
///
/// **FN-CLI-04 hard rule (D-25.14):** the inner match over PendingInput is **exhaustive** —
/// no `_ =>` catch-all, no `unreachable!()`. Adding a new `PendingInput` variant forces
/// the compiler to flag this match at build time.
///
/// **Widened signature (Phase 29 / D-29.3):**
/// - `pending: Option<&PendingInput>` — `None` when no modal is active.
/// - `modal_prompt: Option<&str>` — set by hp41-core when a Math Pac I modal is open.
///
/// **Precedence rules (RESEARCH §3.3 / §7.3):**
/// - If `pending.is_none() && modal_prompt.is_some()` → return `modal_prompt` directly.
/// - If `pending == Some(XeqByName{CollectForModal})` AND `modal_prompt.is_some()` →
///   modal_prompt wins (shows the user the Math Pac I prompt, not the XEQ UI indicator).
/// - Otherwise: render the PendingInput (existing exhaustive match).
/// - If both are None: return empty string (caller falls through to alpha/message/Ready).
///
/// `pub` so integration tests under `hp41-cli/tests/` can verify status-bar formatting.
pub fn pending_prompt(
    pending: Option<&crate::app::PendingInput>,
    modal_prompt: Option<&str>,
) -> String {
    use crate::app::{PendingInput, XeqByNameMode};
    use hp41_core::ops::{FlagTestKind, StoArithKind};

    use crate::keys::{FlagPromptKind, RegisterOpKind};

    // Phase 29 (D-29.3): precedence rules.
    // WR-05 fix: `if let Some(mp)` is structurally identical to the previous
    // `is_some() ... unwrap_or("")` pair but makes the invariant explicit —
    // no defensive `""` fallback that clippy::unwrap_used would flag and that
    // could only be reached if the surrounding guard was wrong.
    if pending.is_none() {
        if let Some(mp) = modal_prompt {
            return mp.to_string();
        }
    }
    // CollectForModal: modal_prompt wins when both are Some
    if let Some(PendingInput::XeqByName {
        mode: XeqByNameMode::CollectForModal,
        ..
    }) = pending
    {
        if let Some(mp) = modal_prompt {
            return mp.to_string();
        }
    }

    // Standard exhaustive match over pending (FN-CLI-04 invariant preserved)
    match pending {
        None => String::new(),
        Some(pending) => match pending {
            PendingInput::StoRegister(acc) => format!("STO [{acc:_<2}]"),
            PendingInput::RclRegister(acc) => format!("RCL [{acc:_<2}]"),
            PendingInput::StoAdd(acc) => format!("STO+ [{acc:_<2}]"),
            PendingInput::StoSub(acc) => format!("STO- [{acc:_<2}]"),
            PendingInput::StoMul(acc) => format!("STO\u{00D7} [{acc:_<2}]"),
            PendingInput::StoDiv(acc) => format!("STO\u{00F7} [{acc:_<2}]"),
            PendingInput::AssignKey => "Assign: press key to assign".to_string(),
            PendingInput::AssignLabel(c, acc) => format!("Assign '{c}' \u{2192} LBL: [{acc}]"),
            PendingInput::ConfirmLoad(idx) => {
                let name = crate::programs::sample_programs()
                    .get(*idx)
                    .map(|p| p.name)
                    .unwrap_or("program");
                format!("Load '{name}'? Current program will be lost. [Y/n]")
            }
            PendingInput::FmtDigits(mode) => {
                let label = match mode {
                    hp41_core::DisplayMode::Fix(_) => "FIX",
                    hp41_core::DisplayMode::Sci(_) => "SCI",
                    hp41_core::DisplayMode::Eng(_) => "ENG",
                };
                format!("{label} [_]  (0\u{2013}9 set digits, f cycles, Esc cancel)")
            }
            PendingInput::PrintModal => "PRNT: _".to_string(),
            PendingInput::HexModal(acc) => {
                if acc.is_empty() {
                    "HEX: __".to_string()
                } else {
                    format!("HEX: {acc}_")
                }
            }
            // ── Phase 25 Plan 02 — Hybrid PendingInput variants (D-25.11) ────
            PendingInput::FlagPrompt { kind, ind, acc } => {
                let mnemonic = match kind {
                    FlagPromptKind::SetFlag => "SF",
                    FlagPromptKind::ClearFlag => "CF",
                    FlagPromptKind::Test(FlagTestKind::IsSet) => "FS?",
                    FlagPromptKind::Test(FlagTestKind::IsClear) => "FC?",
                    FlagPromptKind::Test(FlagTestKind::IsSetThenClear) => "FS?C",
                    FlagPromptKind::Test(FlagTestKind::IsClearThenClear) => "FC?C",
                };
                let ind_str = if *ind { " IND" } else { "" };
                format!("{mnemonic}{ind_str} [{acc:_<2}]")
            }
            PendingInput::RegisterPrompt { op, ind, acc } => {
                let mnemonic = match op {
                    RegisterOpKind::Sto => "STO",
                    RegisterOpKind::Rcl => "RCL",
                    RegisterOpKind::StoArith(StoArithKind::Add) => "STO+",
                    RegisterOpKind::StoArith(StoArithKind::Sub) => "STO-",
                    RegisterOpKind::StoArith(StoArithKind::Mul) => "STO\u{00D7}",
                    RegisterOpKind::StoArith(StoArithKind::Div) => "STO\u{00F7}",
                    RegisterOpKind::View => "VIEW",
                    RegisterOpKind::Arcl => "ARCL",
                    RegisterOpKind::Asto => "ASTO",
                    RegisterOpKind::Isg => "ISG",
                    RegisterOpKind::Dse => "DSE",
                };
                let ind_str = if *ind { " IND" } else { "" };
                format!("{mnemonic}{ind_str} [{acc:_<2}]")
            }
            PendingInput::ClpLabel(acc) => format!("CLP [{acc}]_"),
            PendingInput::DelCount(acc) => format!("DEL [{acc:_<3}]"),
            PendingInput::TonePrompt => "TONE [_]".to_string(),
            // Phase 29 (D-29.8): XeqByName is now a struct variant with XeqByNameMode.
            // Two explicit arms per FN-CLI-04 (no `_ =>`).
            PendingInput::XeqByName {
                acc,
                mode: XeqByNameMode::Normal,
            } => {
                format!("XEQ \"{acc}\"_")
            }
            PendingInput::XeqByName {
                acc,
                mode: XeqByNameMode::CollectForModal,
            } => {
                // CollectForModal: modal_prompt should have won above; this is fallback.
                format!("NAME: {acc}_")
            }
        }, // end Some(pending) => match pending
    } // end match pending (outer)
}

// ── Phase 5 overlays ─────────────────────────────────────────────────────────

/// Render the HP-41 Function Reference overlay (D-17, UX-01).
/// Uses ratatui 0.30 Rect::centered() — no manual calculation needed.
/// RESEARCH Pitfall 1: draw(&self) is immutable; RefCell<TableState> allows borrow_mut here.
fn render_help_overlay(app: &App, frame: &mut Frame) {
    let overlay_area = frame
        .area()
        .centered(Constraint::Percentage(80), Constraint::Percentage(90));

    // D-25.16 / D-25.18: rows derive from docs/hp41cv-functions.json via
    // help_data::help_overlay_rows. Category headers are synthesised by the
    // helper as "=== <category> ===" with empty key/op fields.
    let overlay_rows = help_data::help_overlay_rows();
    let rows: Vec<Row> = overlay_rows
        .iter()
        .map(|row| {
            if row.desc.starts_with("===") {
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(row.desc.clone()),
                ])
                .style(ratatui::style::Style::new().bold())
            } else {
                Row::new(vec![
                    Cell::from(row.key.clone()),
                    Cell::from(row.op.clone()),
                    Cell::from(row.desc.clone()),
                ])
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Min(30),
        ],
    )
    .block(Block::bordered().title_top(" HP-41 Function Reference  [? or Esc to close] "))
    .row_highlight_style(ratatui::style::Style::new().reversed());

    // Clear the underlying widgets (right-panel, stack panel, display) before
    // painting the overlay. Without this `Clear`, ratatui's Table renders
    // cells with transparent backgrounds — the underlying content bleeds
    // through in the gaps between columns (visible as e.g. `XEQ "X<Y?"X<Y?`
    // appearing inside the overlay's Math section). Standard ratatui modal
    // pattern: render `Clear` to the modal area first, then the modal.
    frame.render_widget(Clear, overlay_area);
    // RefCell::borrow_mut() — safe: draw() is single-threaded and non-reentrant.
    frame.render_stateful_widget(table, overlay_area, &mut app.help_table_state.borrow_mut());
}

/// Render the program library overlay (D-22, UX-03).
fn render_programs_overlay(app: &App, frame: &mut Frame) {
    let overlay_area = frame
        .area()
        .centered(Constraint::Percentage(70), Constraint::Percentage(80));

    let progs = programs::sample_programs();
    let rows: Vec<Row> = progs
        .iter()
        .map(|p| Row::new(vec![Cell::from(p.name), Cell::from(p.description)]))
        .collect();

    let table = Table::new(rows, [Constraint::Length(22), Constraint::Min(30)])
        .block(Block::bordered().title_top(" Sample Programs  [Enter=load, Esc=close] "))
        .row_highlight_style(ratatui::style::Style::new().reversed());

    // Same `Clear` rationale as `render_help_overlay` — wipe the underlying
    // widgets before painting the modal to prevent right-panel/stack/display
    // bleed-through in column gaps.
    frame.render_widget(Clear, overlay_area);
    frame.render_stateful_widget(
        table,
        overlay_area,
        &mut app.programs_table_state.borrow_mut(),
    );
}

// ── Right panel ───────────────────────────────────────────────────────────────

fn render_right_panel(_app: &App, frame: &mut Frame, area: Rect) {
    // INPUT-01 / D-03 / D-25.18 (Plan 25-04): key-reference panel —
    // discoverable key labels derived from docs/hp41cv-functions.json via
    // help_data::help_entries() (no hand-curated KEY_REF_TABLE const).
    let block = Block::bordered().title_top(" Keys ");

    let entries = key_ref_entries();
    let lines: Vec<Line> = entries
        .iter()
        .map(|(k, desc)| {
            Line::from(vec![
                Span::styled(format!("{k:<8}"), Style::new().bold()),
                Span::raw(desc.clone()),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

#[cfg(test)]
mod tests {
    use hp41_core::CalcState;
    use std::path::PathBuf;

    fn make_app() -> crate::app::App {
        crate::app::App::new(
            CalcState::new(),
            PathBuf::from("/tmp/hp41_ui_test.json"),
            None,
        )
    }

    /// BLOCKER 1: test_help_scroll — help_table_state.select_next() must not panic.
    #[test]
    fn test_help_scroll() {
        let app = make_app();
        assert!(!app.show_help, "show_help starts false");
        // select_next() on unselected TableState wraps to index 0; must not panic.
        app.help_table_state.borrow_mut().select_next();
        // A second call also must not panic.
        app.help_table_state.borrow_mut().select_next();
        app.help_table_state.borrow_mut().select_previous();
    }

    /// BLOCKER 1: test_programs_scroll — programs_table_state.select_next() must not panic.
    #[test]
    fn test_programs_scroll() {
        let app = make_app();
        assert!(!app.show_programs, "show_programs starts false");
        app.programs_table_state.borrow_mut().select_next();
        app.programs_table_state.borrow_mut().select_next();
        app.programs_table_state.borrow_mut().select_previous();
    }
}

#[cfg(test)]
mod entry_buf_display_tests {
    use super::format_entry_buf_display;

    #[test]
    fn test_d01_trailing_e_no_digits() {
        // D-01: "1.5e" -> "1.5E_ _"
        assert_eq!(format_entry_buf_display("1.5e"), "1.5E_ _");
    }

    #[test]
    fn test_d02_one_exponent_digit() {
        // D-02: "1.5e2" -> "1.5E2_"
        assert_eq!(format_entry_buf_display("1.5e2"), "1.5E2_");
    }

    #[test]
    fn test_d03_two_exponent_digits_no_underscores() {
        // D-03: "1.5e23" -> "1.5E23"
        assert_eq!(format_entry_buf_display("1.5e23"), "1.5E23");
    }

    #[test]
    fn test_d04_implicit_one_mantissa() {
        // D-04: "1e" -> "1E_ _"
        assert_eq!(format_entry_buf_display("1e"), "1E_ _");
    }

    #[test]
    fn test_negative_exponent_one_digit() {
        // Negative exponent + 1 digit: sign preserved, second slot is underscore.
        assert_eq!(format_entry_buf_display("1.5e-2"), "1.5E-2_");
    }

    #[test]
    fn test_negative_exponent_two_digits() {
        // Negative exponent + 2 digits: sign preserved, no underscores.
        assert_eq!(format_entry_buf_display("1.5e-23"), "1.5E-23");
    }

    #[test]
    fn test_no_e_returns_verbatim() {
        // Defensive fallback: no 'e' in input -> verbatim.
        assert_eq!(format_entry_buf_display("1.5"), "1.5");
        assert_eq!(format_entry_buf_display("12"), "12");
    }
}
