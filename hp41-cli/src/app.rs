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

        // Phase 5: pending input guard — intercept BEFORE alpha mode and digit entry (D-08, Pitfall 5).
        // S key triggers StoRegister modal; R key triggers RclRegister modal.
        if key.code == KeyCode::Char('S') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::StoRegister(String::new()));
            self.message = None;
            return;
        }
        if key.code == KeyCode::Char('R') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::RclRegister(String::new()));
            self.message = None;
            return;
        }
        // Ctrl+A triggers USER key assignment modal (D-27)
        if key.code == KeyCode::Char('a') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::AssignKey);
            self.message = None;
            return;
        }
        // Route to pending_input handler if modal is active
        if self.pending_input.is_some() {
            self.handle_pending_input(key);
            return;
        }

        // Phase 5: ALPHA mode routing (D-12) — must be BEFORE digit-entry block (RESEARCH Pitfall 5).
        // In ALPHA mode, 'a' must append 'a', not dispatch Asin.
        if self.state.alpha_mode {
            self.handle_alpha_mode_key(key);
            return;
        }

        // Phase 5: overlay navigation — help and program library overlays intercept nav keys.
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.show_help = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_table_state.borrow_mut().select_previous();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_table_state.borrow_mut().select_next();
                }
                _ => {}
            }
            return; // consume all keys when help overlay is open
        }
        if self.show_programs {
            match key.code {
                KeyCode::Esc => {
                    self.show_programs = false;
                    self.pending_input = None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.programs_table_state.borrow_mut().select_previous();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.programs_table_state.borrow_mut().select_next();
                }
                KeyCode::Enter => {
                    // Load selected program (D-22)
                    if let Some(idx) = self.programs_table_state.borrow().selected() {
                        let programs = crate::programs::sample_programs();
                        if idx < programs.len() {
                            if !self.state.program.is_empty() {
                                // Non-empty: request confirmation (D-22)
                                self.pending_input = Some(PendingInput::ConfirmLoad(idx));
                                self.show_programs = false;
                            } else {
                                self.state.program = programs[idx].ops.clone();
                                self.message = Some(format!("Loaded: {}", programs[idx].name));
                                self.show_programs = false;
                            }
                        }
                    }
                }
                _ => {}
            }
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

    /// Handle key events while pending_input is Some(…). (D-08 through D-11, D-22, D-27)
    /// Release filter already applied in handle_key() before this is called.
    fn handle_pending_input(&mut self, key: KeyEvent) {
        // Take pending — we re-set it only if the modal continues.
        let pending = self.pending_input.take();
        match pending {
            Some(PendingInput::StoRegister(ref acc)) => self.handle_reg_modal(
                key, acc.clone(),
                |reg| Op::StoReg(reg),
                PendingInput::StoRegister,
            ),
            Some(PendingInput::RclRegister(ref acc)) => self.handle_reg_modal(
                key, acc.clone(),
                |reg| Op::RclReg(reg),
                PendingInput::RclRegister,
            ),
            Some(PendingInput::StoAdd(ref acc)) => self.handle_reg_modal(
                key, acc.clone(),
                |reg| Op::StoArith { reg, kind: hp41_core::ops::StoArithKind::Add },
                PendingInput::StoAdd,
            ),
            Some(PendingInput::StoSub(ref acc)) => self.handle_reg_modal(
                key, acc.clone(),
                |reg| Op::StoArith { reg, kind: hp41_core::ops::StoArithKind::Sub },
                PendingInput::StoSub,
            ),
            Some(PendingInput::StoMul(ref acc)) => self.handle_reg_modal(
                key, acc.clone(),
                |reg| Op::StoArith { reg, kind: hp41_core::ops::StoArithKind::Mul },
                PendingInput::StoMul,
            ),
            Some(PendingInput::StoDiv(ref acc)) => self.handle_reg_modal(
                key, acc.clone(),
                |reg| Op::StoArith { reg, kind: hp41_core::ops::StoArithKind::Div },
                PendingInput::StoDiv,
            ),
            Some(PendingInput::AssignKey) => {
                // D-27 step 1: waiting for any printable char
                match key.code {
                    KeyCode::Esc => { self.pending_input = None; }
                    KeyCode::Char(c) => {
                        self.pending_input = Some(PendingInput::AssignLabel(c, String::new()));
                    }
                    _ => { self.pending_input = Some(PendingInput::AssignKey); }
                }
            }
            Some(PendingInput::AssignLabel(c, ref acc)) => {
                // D-27 step 2: accumulating label name
                match key.code {
                    KeyCode::Esc => { self.pending_input = None; }
                    KeyCode::Enter => {
                        if !acc.is_empty() {
                            self.state.key_assignments.insert(c, acc.clone());
                            self.message = Some(format!("Assigned '{c}' \u{2192} LBL:{acc}"));
                        }
                        self.pending_input = None;
                    }
                    KeyCode::Backspace => {
                        let mut new_acc = acc.clone();
                        new_acc.pop();
                        self.pending_input = Some(PendingInput::AssignLabel(c, new_acc));
                    }
                    KeyCode::Char(ch) => {
                        let mut new_acc = acc.clone();
                        new_acc.push(ch);
                        self.pending_input = Some(PendingInput::AssignLabel(c, new_acc));
                    }
                    _ => { self.pending_input = Some(PendingInput::AssignLabel(c, acc.clone())); }
                }
            }
            Some(PendingInput::ConfirmLoad(idx)) => {
                // D-22: confirm before overwriting existing program
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        let programs = crate::programs::sample_programs();
                        if idx < programs.len() {
                            self.state.program = programs[idx].ops.clone();
                            self.message = Some(format!("Loaded: {}", programs[idx].name));
                        }
                        self.pending_input = None;
                    }
                    _ => {
                        // Any other key (including 'n', Esc) = cancel
                        self.message = Some("Load cancelled".to_string());
                        self.pending_input = None;
                    }
                }
            }
            None => {} // shouldn't happen (guard checked is_some() before calling)
        }
    }

    /// Generic 2-digit register number accumulator (D-09).
    /// op_fn: given the parsed register number, returns the Op to dispatch.
    /// pending_fn: given the accumulator string, returns the PendingInput variant to continue.
    fn handle_reg_modal(
        &mut self,
        key: KeyEvent,
        acc: String,
        op_fn: impl Fn(u8) -> Op,
        pending_fn: impl Fn(String) -> PendingInput,
    ) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let mut new_acc = acc;
                new_acc.push(c);
                if new_acc.len() == 2 {
                    // Auto-dispatch on second digit (D-09)
                    let reg: u8 = new_acc.parse().unwrap_or(0);
                    if reg < 100 {
                        self.call_dispatch(op_fn(reg));
                    } else {
                        self.message = Some(format!("Invalid register: {new_acc}"));
                    }
                    self.pending_input = None;
                } else {
                    self.pending_input = Some(pending_fn(new_acc));
                }
            }
            KeyCode::Backspace => {
                // D-09: Backspace resets entire accumulator
                self.pending_input = Some(pending_fn(String::new()));
            }
            KeyCode::Esc => {
                // D-09: Esc cancels modal
                self.pending_input = None;
            }
            _ => {
                // Non-digit, non-control key: restore modal (silently ignore)
                self.pending_input = Some(pending_fn(acc));
            }
        }
    }

    /// Handle key events while state.alpha_mode is true (D-12 through D-15).
    /// Printable chars → AlphaAppend. Backspace → AlphaBackspace. Enter/a → AlphaToggle (exit).
    /// Release filter already applied in handle_key() before this is called.
    fn handle_alpha_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                // Exit ALPHA mode (D-13, D-15)
                self.call_dispatch(Op::AlphaToggle);
            }
            KeyCode::Char('a') => {
                // D-15: 'a' exits ALPHA mode (same as Enter)
                self.call_dispatch(Op::AlphaToggle);
            }
            KeyCode::Backspace => {
                // D-13: Backspace in ALPHA mode = AlphaBackspace (remove last char)
                self.call_dispatch(Op::AlphaBackspace);
            }
            KeyCode::Char(c) => {
                // D-12: all printable chars route to AlphaAppend
                self.call_dispatch(Op::AlphaAppend(c));
            }
            _ => {
                // Other keys (arrows, F-keys, etc.) ignored in ALPHA mode
            }
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

    fn make_app() -> App {
        App::new(hp41_core::CalcState::new(), std::path::PathBuf::from("/tmp/test_07.json"))
    }

    /// UX-02: try_user_dispatch() returns true and runs program when user_mode is true
    /// and the pressed key char has an assignment in state.key_assignments.
    #[test]
    fn test_user_mode_dispatch_runs_program() {
        let mut app = make_app();
        // Set up a simple program under label "A"
        app.state.program = vec![
            hp41_core::ops::Op::Lbl("A".to_string()),
            hp41_core::ops::Op::PushNum(hp41_core::HpNum::from(42i32)),
            hp41_core::ops::Op::Rtn,
        ];
        // Assign 'z' → "A"
        app.state.key_assignments.insert('z', "A".to_string());
        app.state.user_mode = true;

        // Simulate pressing 'z' with user_mode on
        let result = app.try_user_dispatch(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result, "try_user_dispatch must return true when key is assigned");
        // Program should have run and pushed 42 onto stack
        assert!(!app.state.stack.x.is_zero(), "program should have pushed 42 to X");
    }

    /// UX-02: try_user_dispatch() returns false when user_mode is false,
    /// even if the key has an assignment.
    #[test]
    fn test_user_mode_dispatch_skipped_when_off() {
        let mut app = make_app();
        app.state.key_assignments.insert('z', "A".to_string());
        app.state.user_mode = false; // USER mode OFF

        let result = app.try_user_dispatch(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(!result, "try_user_dispatch must return false when user_mode is off");
    }

    /// UX-02: try_user_dispatch() returns false when user_mode is true
    /// but the pressed key has no assignment.
    #[test]
    fn test_user_mode_dispatch_no_assignment() {
        let mut app = make_app();
        app.state.user_mode = true;
        // No assignment for 'z'

        let result = app.try_user_dispatch(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(!result, "try_user_dispatch must return false when key has no assignment");
    }

    /// UX-02: try_user_dispatch() shows error in message when run_program returns Err
    /// (label referenced in assignment not found in program).
    #[test]
    fn test_user_mode_dispatch_missing_label_shows_error() {
        let mut app = make_app();
        app.state.user_mode = true;
        // Assign 'z' → "MISSING" but no program with that label exists
        app.state.key_assignments.insert('z', "MISSING".to_string());

        let result = app.try_user_dispatch(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::NONE,
        ));
        // Still returns true (key was consumed) — error shown in status bar
        assert!(result, "try_user_dispatch must return true (consumed) even on Err");
        assert!(app.message.is_some(), "error message must be set when label not found");
    }

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
