//! App — top-level state container and event loop for hp41-cli.
//!
//! App owns CalcState and acts as the controller between crossterm key events
//! and hp41-core's dispatch() entry point.

use std::cell::RefCell;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::widgets::TableState;

use hp41_core::ops::Op;
use hp41_core::{CalcState, AngleMode, DisplayMode};

use crate::{keys, persistence, ui};

/// Transient UI state for multi-key input (D-08). NOT serialized to disk.
/// Consumed in App::handle_pending_input(). Cleared on Esc or successful dispatch.
#[derive(Debug, Clone)]
pub enum PendingInput {
    StoRegister(String),          // accumulating 2-digit register number for STO [nn]
    RclRegister(String),          // accumulating 2-digit register number for RCL [nn]
    StoAdd(String),               // STO+ [nn]
    StoSub(String),               // STO- [nn]
    StoMul(String),               // STO× [nn]
    StoDiv(String),               // STO÷ [nn]
    AssignKey,                    // D-27 step 1: waiting for key char to assign
    AssignLabel(char, String),    // D-27 step 2: char received; accumulating label name
    ConfirmLoad(usize),           // D-22: awaiting Y/n before overwriting program
}

/// Top-level application state. Flat struct — no state machine required for Phase 4.
pub struct App {
    pub state: CalcState,
    /// One-line status / error message shown in the TUI status bar. None = no message.
    pub message: Option<String>,
    /// Set to true to exit the event loop and return from run().
    pub exit: bool,
    // ── Phase 5: persistence (D-05) ──────────────────────────────────────────
    pub last_save: Instant,
    pub state_path: PathBuf,
    // ── Phase 5: modal input (D-08) ──────────────────────────────────────────
    pub pending_input: Option<PendingInput>,
    // ── Phase 5: overlays (D-16, D-22) ───────────────────────────────────────
    pub show_help: bool,
    pub help_scroll: usize,
    /// RefCell: draw(&self) is immutable but render_stateful_widget needs &mut TableState.
    /// Single-threaded, non-reentrant draw — borrow_mut() will never panic. (RESEARCH Pitfall 1)
    pub help_table_state: RefCell<TableState>,
    pub show_programs: bool,
    pub programs_scroll: usize,
    pub programs_table_state: RefCell<TableState>,
}

impl App {
    pub fn new(state: CalcState, state_path: PathBuf) -> Self {
        App {
            state,
            message: None,
            exit: false,
            last_save: Instant::now(),
            state_path,
            pending_input: None,
            show_help: false,
            help_scroll: 0,
            help_table_state: RefCell::new(TableState::default()),
            show_programs: false,
            programs_scroll: 0,
            programs_table_state: RefCell::new(TableState::default()),
        }
    }

    /// Check whether the auto-save interval has elapsed; save if so.
    /// Extracted from run() so it can be unit-tested with a manipulated `last_save`.
    /// Called once per poll iteration from run().
    pub fn check_autosave(&mut self) {
        if self.last_save.elapsed() >= Duration::from_secs(30) {
            if let Err(e) = persistence::save_state(&self.state_path, &self.state) {
                // One-time warning; retry on next 30s tick (Claude's Discretion)
                self.message = Some(format!("Auto-save failed: {e}"));
            }
            self.last_save = Instant::now(); // reset even on failure
        }
    }

    /// Run the synchronous poll-based event loop until self.exit is true.
    ///
    /// D-04: poll(16ms) — never event::read() directly in the loop; poll() is required
    /// so Phase 5 can inject the auto-save timer check without blocking redraws.
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            // Render first — always draw before blocking on input.
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
            // PERS-02: 30-second auto-save via extracted method (D-05)
            self.check_autosave();
        }
        // D-05: save on graceful exit before ratatui::restore()
        let _ = persistence::save_state(&self.state_path, &self.state);
        Ok(())
    }

    /// Render the TUI. Takes &self (immutable) to avoid borrow conflict with
    /// &mut terminal inside terminal.draw() — see RESEARCH.md Pitfall 4.
    fn draw(&self, frame: &mut ratatui::Frame) {
        ui::render_ui(self, frame);
    }

    /// Handle a single key event. All mutation of CalcState happens here (not in draw).
    fn handle_key(&mut self, key: KeyEvent) {
        // D-06: filter Release immediately — Windows crossterm fires both Press and Release.
        // This MUST be the first check — no other logic before it.
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Quit: 'q' or Ctrl+C
        if key.code == KeyCode::Char('q') {
            self.exit = true;
            return;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit = true;
            return;
        }

        // D-04: Ctrl+S — manual save to active state file
        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            match persistence::save_state(&self.state_path, &self.state) {
                Ok(()) => self.message = Some(format!(
                    "Saved to {}",
                    self.state_path.display()
                )),
                Err(e) => self.message = Some(format!("Save failed: {e}")),
            }
            return;
        }

        // D-16: '?' toggles the help overlay
        if key.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            self.show_programs = false; // close programs overlay if open
            return;
        }

        // D-22: Ctrl+P toggles the program library overlay
        if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.show_programs = !self.show_programs;
            self.show_help = false; // close help overlay if open
            return;
        }

        // D-11 / D-13: digit keys and 'e' (EEX) append directly to entry_buf.
        // dispatch() calls flush_entry_buf() automatically on the next non-digit op.
        // DO NOT call dispatch() here — that would push each digit as a separate PushNum.
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_digit() || c == '.' || c == 'e' {
                self.state.entry_buf.push(c);
                self.message = None;
                return;
            }
        }

        // D-10: 'd' cycles angle mode DEG → RAD → GRAD
        if key.code == KeyCode::Char('d') {
            let next_op = match self.state.angle_mode {
                AngleMode::Deg  => Op::SetRad,
                AngleMode::Rad  => Op::SetGrad,
                AngleMode::Grad => Op::SetDeg,
            };
            self.call_dispatch(next_op);
            return;
        }

        // D-10: 'f' cycles display format FIX 4 → SCI 4 → ENG 4 (digit count stays 4)
        if key.code == KeyCode::Char('f') {
            let next_op = match self.state.display_mode {
                DisplayMode::Fix(_) => Op::FmtSci(4),
                DisplayMode::Sci(_) => Op::FmtEng(4),
                DisplayMode::Eng(_) => Op::FmtFix(4),
            };
            self.call_dispatch(next_op);
            return;
        }

        // D-15: SST (single-step forward) — F7, increments pc without executing
        if key.code == KeyCode::F(7) {
            if self.state.pc < self.state.program.len() {
                self.state.pc += 1;
            }
            return;
        }

        // D-15: BST (back-step) — F8, decrements pc without executing
        if key.code == KeyCode::F(8) {
            self.state.pc = self.state.pc.saturating_sub(1);
            return;
        }

        // D-16: R/S — F5, hardcoded run_program("A") in Phase 4
        // F5 is handled here directly, not routed through key_to_op().
        if key.code == KeyCode::F(5) {
            match hp41_core::run_program(&mut self.state, "A") {
                Ok(()) => self.message = None,
                Err(e) => self.message = Some(format!("{e}")),
            }
            return;
        }

        // All other ops: route through keys.rs → dispatch()
        if let Some(op) = keys::key_to_op(key, self) {
            self.call_dispatch(op);
        }
    }

    /// Call hp41_core::ops::dispatch and map any HpError to self.message.
    fn call_dispatch(&mut self, op: Op) {
        match hp41_core::ops::dispatch(&mut self.state, op) {
            Ok(()) => self.message = None,
            Err(e) => self.message = Some(format!("{e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// PERS-02: verify that check_autosave() saves the file when last_save is old enough.
    /// Manipulates last_save directly to avoid sleeping 30s in tests.
    #[test]
    fn test_autosave_timer_logic() {
        let tmp_dir = std::env::temp_dir().join("hp41_autosave_test");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let state_path = tmp_dir.join("autosave_test.json");

        let state = hp41_core::CalcState::new();
        let mut app = App::new(state, state_path.clone());

        // Wind last_save back 31 seconds — timer should fire immediately.
        app.last_save = Instant::now() - Duration::from_secs(31);

        app.check_autosave();

        assert!(
            state_path.exists(),
            "check_autosave() must create the state file when 30s have elapsed"
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// PERS-02: verify that check_autosave() does NOT save when last_save is recent.
    #[test]
    fn test_autosave_timer_no_premature_save() {
        let tmp_dir = std::env::temp_dir().join("hp41_autosave_premature_test");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let state_path = tmp_dir.join("premature.json");

        let state = hp41_core::CalcState::new();
        let mut app = App::new(state, state_path.clone());

        // last_save is Instant::now() — only 0ms elapsed; should NOT save.
        app.check_autosave();

        assert!(
            !state_path.exists(),
            "check_autosave() must NOT save before 30s have elapsed"
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
