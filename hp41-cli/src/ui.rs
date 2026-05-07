//! TUI widget layout for the HP-41 calculator emulator.
//!
//! Layout (D-01/D-02):
//!   Left 55%: stack panel (T/Z/Y/X/LASTX) + display panel + annunciator bar + status bar
//!   Right 45%: key-reference panel (INPUT-01 discoverability)
//!
//! Minimum terminal size: 80×24 (D-01). Smaller terminals render an error message only.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use hp41_core::{AngleMode, format_hpnum, format_alpha};

use crate::app::App;
use crate::prgm_display;
use crate::keys::KEY_REF_TABLE;

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
    let [left, right] = Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ]).areas(area);

    render_left_panel(app, frame, left);
    render_right_panel(app, frame, right);
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
    let [row_t, row_z, row_y, row_x, row_lastx, row_spacer,
         row_display, row_annunc, row_status] = Layout::vertical([
        Constraint::Length(1),   // T
        Constraint::Length(1),   // Z
        Constraint::Length(1),   // Y
        Constraint::Length(1),   // X
        Constraint::Length(1),   // LASTX
        Constraint::Length(1),   // spacer
        Constraint::Length(3),   // display (prominent — 3 lines)
        Constraint::Length(1),   // annunciators
        Constraint::Min(0),      // status bar (remainder)
    ]).areas(area);

    let _ = row_spacer; // intentionally empty

    render_stack(app, frame, row_t, row_z, row_y, row_x, row_lastx);
    render_display(app, frame, row_display);
    render_annunciators(app, frame, row_annunc);
    render_status(app, frame, row_status);
}

fn render_stack(
    app: &App, frame: &mut Frame,
    row_t: Rect, row_z: Rect, row_y: Rect, row_x: Rect, row_lastx: Rect,
) {
    let st = &app.state;
    let mode = &st.display_mode;

    // D-02: X/Y/Z/T each on their own labeled line, LASTX below T.
    let fmt = |label: &str, val: &hp41_core::HpNum| -> Paragraph<'static> {
        let text = format!("{label}: {}", format_hpnum(val, mode));
        Paragraph::new(text)
    };

    frame.render_widget(fmt("T", &st.stack.t),      row_t);
    frame.render_widget(fmt("Z", &st.stack.z),      row_z);
    frame.render_widget(fmt("Y", &st.stack.y),      row_y);
    frame.render_widget(fmt("X", &st.stack.x),      row_x);
    frame.render_widget(fmt("L", &st.stack.lastx),  row_lastx);
}

fn render_display(app: &App, frame: &mut Frame, area: Rect) {
    // D-12 / D-14: display priority order:
    //   1. entry_buf content (live digit preview while typing)
    //   2. prgm_display::format_step() when in PRGM recording mode
    //   3. format_alpha() when in ALPHA mode
    //   4. format_hpnum(X, display_mode) — normal calculator display
    let display_str = get_display_string(app);

    let block = Block::bordered().title_top(" Display ");
    frame.render_widget(
        Paragraph::new(display_str).block(block),
        area,
    );
}

/// Get the string to show in the HP-41 display area.
/// Priority: entry_buf > prgm step > alpha > formatted X.
fn get_display_string(app: &App) -> String {
    let st = &app.state;
    if !st.entry_buf.is_empty() {
        // Live digit preview while user is typing a number.
        st.entry_buf.clone()
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
    // USER and SHIFT are always dim in Phase 4 (USER mode is Phase 5).
    let line = Line::from(vec![
        ann("USER",  st.user_mode),
        Span::raw(" "),
        ann("PRGM",  st.prgm_mode),
        Span::raw(" "),
        ann("ALPHA", st.alpha_mode),
        Span::raw(" "),
        ann("SHIFT", false),
        Span::raw(" "),
        ann("RAD",   st.angle_mode == AngleMode::Rad),
        Span::raw(" "),
        ann("DEG",   st.angle_mode == AngleMode::Deg),
        Span::raw(" "),
        ann("GRAD",  st.angle_mode == AngleMode::Grad),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    // D-11: pending_input prompts override normal status message
    // D-14: ALPHA mode has a standard status message
    let text: String = if let Some(ref pending) = app.pending_input {
        pending_prompt(pending)
    } else if app.state.alpha_mode {
        "ALPHA mode — Enter or A to exit".to_string()
    } else {
        app.message.as_deref().unwrap_or("Ready").to_string()
    };
    frame.render_widget(Paragraph::new(text), area);
}

/// Format the status bar text for each PendingInput variant (D-11).
/// Uses {:_<2} to show placeholder underscores for accumulator length.
fn pending_prompt(pending: &crate::app::PendingInput) -> String {
    use crate::app::PendingInput;
    match pending {
        PendingInput::StoRegister(acc)      => format!("STO [{:_<2}]", acc),
        PendingInput::RclRegister(acc)      => format!("RCL [{:_<2}]", acc),
        PendingInput::StoAdd(acc)           => format!("STO+ [{:_<2}]", acc),
        PendingInput::StoSub(acc)           => format!("STO- [{:_<2}]", acc),
        PendingInput::StoMul(acc)           => format!("STO\u{00D7} [{:_<2}]", acc),
        PendingInput::StoDiv(acc)           => format!("STO\u{00F7} [{:_<2}]", acc),
        PendingInput::AssignKey             => "Assign: press key to assign".to_string(),
        PendingInput::AssignLabel(c, acc)   => format!("Assign '{c}' \u{2192} LBL: [{acc}]"),
        PendingInput::ConfirmLoad(idx)      => {
            let name = crate::programs::sample_programs()
                .get(*idx)
                .map(|p| p.name)
                .unwrap_or("program");
            format!("Load '{name}'? Current program will be lost. [Y/n]")
        }
    }
}

// ── Right panel ───────────────────────────────────────────────────────────────

fn render_right_panel(_app: &App, frame: &mut Frame, area: Rect) {
    // INPUT-01 / D-03: key-reference panel — discoverable key labels.
    // Built from the same KEY_REF_TABLE constant in keys.rs that drives key_to_op().
    let block = Block::bordered().title_top(" Keys ");

    let lines: Vec<Line> = KEY_REF_TABLE.iter()
        .map(|(k, desc)| {
            Line::from(vec![
                Span::styled(format!("{k:<8}"), Style::new().bold()),
                Span::raw(*desc),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).block(block), area);
}
