//! App — top-level state container and event loop for hp41-cli.
//!
//! App owns CalcState and acts as the controller between crossterm key events
//! and hp41-core's dispatch() entry point.

use std::cell::RefCell;
use std::io::Write; // for writeln! and BufWriter::flush() in call_dispatch_and_drain
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::TableState;
use ratatui::DefaultTerminal;

use hp41_core::ops::{synthetic_byte_to_op, Op, StackReg, StoArithKind};
use hp41_core::{AngleMode, CalcState, DisplayMode};

use crate::{keys, persistence, ui};

/// Transient UI state for multi-key input (D-08). NOT serialized to disk.
/// Consumed in App::handle_pending_input(). Cleared on Esc or successful dispatch.
#[derive(Debug, Clone)]
pub enum PendingInput {
    StoRegister(String), // accumulating 2-digit register number for STO [nn]
    RclRegister(String), // accumulating 2-digit register number for RCL [nn]
    // STO arithmetic step-3 variants (active in v1.1 modal flow — S → op-key → register).
    StoAdd(String),                    // STO+ [nn or stack-reg]
    StoSub(String),                    // STO- [nn or stack-reg]
    StoMul(String),                    // STO× [nn or stack-reg]
    StoDiv(String),                    // STO÷ [nn or stack-reg]
    AssignKey,                         // D-27 step 1: waiting for key char to assign
    AssignLabel(char, String),         // D-27 step 2: char received; accumulating label name
    ConfirmLoad(usize),                // D-22: awaiting Y/n before overwriting program
    FmtDigits(hp41_core::DisplayMode), // digit-count modal for FIX/SCI/ENG (opened by 'F')
    PrintModal, // Phase 11 D-06: 'P'-prefix modal for print ops (PRX/PRA/PRSTK)
    /// Phase 12: hex-byte insertion modal accumulator. Holds 0, 1, or 2 hex chars.
    /// Triggered by uppercase 'X' in PRGM mode (D-14). On 2nd char, validates
    /// against synthetic_byte_to_op() and either inserts Op::SyntheticByte at
    /// state.pc or sets app.message = "INVALID" (D-13).
    HexModal(String),
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
    /// RefCell: draw(&self) is immutable but render_stateful_widget needs &mut TableState.
    /// Single-threaded, non-reentrant draw — borrow_mut() will never panic at runtime.
    pub help_table_state: RefCell<TableState>,
    pub show_programs: bool,
    pub programs_table_state: RefCell<TableState>,
    /// BufWriter for --print-log, if specified. None = no file logging.
    pub print_log_writer: Option<std::io::BufWriter<std::fs::File>>,
    /// Default `~/.hp41/cards/` (or a relative `.hp41/cards` fallback if
    /// `dirs::home_dir()` is unavailable on this platform). Drain helper
    /// receives this by reference; tests can inject a tempdir at the
    /// module-level `cards::drain_pending_card_op` instead of going through
    /// App.
    cards_dir: std::path::PathBuf,
}

impl App {
    pub fn new(
        state: CalcState,
        state_path: PathBuf,
        print_log: Option<std::path::PathBuf>,
    ) -> Self {
        let (print_log_writer, initial_message) = match print_log {
            None => (None, None),
            Some(path) => {
                match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                {
                    Ok(file) => (Some(std::io::BufWriter::new(file)), None),
                    Err(e) => {
                        // Surface the error in the TUI status bar via app.message.
                        // Also write to stderr as a fallback for contexts where the TUI
                        // may not yet be visible (e.g., early startup errors).
                        eprintln!("Warning: cannot open print log '{}': {e}", path.display());
                        (
                            None,
                            Some(format!(
                                "Warning: cannot open print log '{}': {e}",
                                path.display()
                            )),
                        )
                    }
                }
            }
        };
        App {
            state,
            message: initial_message,
            exit: false,
            last_save: Instant::now(),
            state_path,
            pending_input: None,
            show_help: false,
            help_table_state: RefCell::new(TableState::default()),
            show_programs: false,
            programs_table_state: RefCell::new(TableState::default()),
            print_log_writer,
            cards_dir: crate::cards::cards_dir()
                .unwrap_or_else(|| std::path::PathBuf::from(".hp41/cards")),
        }
    }

    /// Check whether the auto-save interval has elapsed; save if so.
    /// Extracted from run() so it can be unit-tested with a manipulated `last_save`.
    /// Called once per poll iteration from run().
    pub fn check_autosave(&mut self) {
        if self.last_save.elapsed() >= Duration::from_secs(30) {
            if let Err(e) = persistence::save_state(&self.state_path, &self.state) {
                // One-time warning; timer resets so the next 30s tick retries
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
        if let Err(e) = persistence::save_state(&self.state_path, &self.state) {
            eprintln!("Warning: failed to save state on exit: {e}");
        }
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

        // Update last_key_code for physical HP-41 keys only:
        // - None from keycode_to_hp41_code = no HP-41 equivalent (F5/F7/F8, unknown keys)
        // - Ctrl-modified keys are TUI commands (save, quit), not calculator keypresses
        if let Some(code) = keys::keycode_to_hp41_code(key.code) {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                self.state.last_key_code = code;
            }
        }

        // Quit: Ctrl+C only (D-16, D-22). 'q' was reassigned to SIN in Phase 8; quit is Ctrl+C only.
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit = true;
            return;
        }

        // D-04: Ctrl+S — manual save to active state file
        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            match persistence::save_state(&self.state_path, &self.state) {
                Ok(()) => self.message = Some(format!("Saved to {}", self.state_path.display())),
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

        // Phase 5: route to pending_input handler if modal is active — MUST come before
        // the modal-opening interceptors below (CR-02). If any modal is active, 'S', 'R',
        // and Ctrl+A must be handled by the active modal, not silently replaced.
        if self.pending_input.is_some() {
            self.handle_pending_input(key);
            return;
        }

        // Only open new modals when no modal is currently active (D-08, Pitfall 5).
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
        // 'F' (Shift+f) opens the digit-count modal for the current display mode.
        // The modal lets the user set an exact digit count for FIX/SCI/ENG via 0–9.
        if key.code == KeyCode::Char('F') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::FmtDigits(self.state.display_mode));
            self.message = None;
            return;
        }
        // 'P' (Shift+p) opens the PrintModal for PRX/PRA/PRSTK selection (D-06, Phase 11).
        if key.code == KeyCode::Char('P') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::PrintModal);
            self.message = None;
            return;
        }
        // [Phase 12 D-14] 'X' (Shift+X / uppercase) opens hex-byte insertion modal in PRGM mode.
        // Lowercase 'x' is unchanged — it dispatches Op::XySwap via key_to_op() below.
        // Gated on prgm_mode: hex insertion only makes sense while recording a program (D-18).
        // 'X' is also a valid key code 21 (XEQ) — the last_key_code update above already ran.
        if key.code == KeyCode::Char('X') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            if self.state.prgm_mode {
                self.pending_input = Some(PendingInput::HexModal(String::new()));
                self.message = None;
            } else {
                self.message = Some("PRGM mode required for hex insertion (X nn)".to_string());
            }
            return;
        }
        // Ctrl+A triggers USER key assignment modal (D-27)
        if key.code == KeyCode::Char('a') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::AssignKey);
            self.message = None;
            return;
        }

        // Phase 19: Card Reader comfort shortcuts — Ctrl+W/R/D/F dispatch the four card ops
        // directly without typing ALPHA + XEQ. Hardware-faithful path still works in parallel.
        if key.code == KeyCode::Char('w') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.call_dispatch_and_drain(Op::Wprgm);
            return;
        }
        if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.call_dispatch_and_drain(Op::Rdprgm);
            return;
        }
        if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.call_dispatch_and_drain(Op::Wdta);
            return;
        }
        if key.code == KeyCode::Char('f') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.call_dispatch_and_drain(Op::Rdta);
            return;
        }

        // Phase 5: ALPHA mode routing (D-12) — must be BEFORE digit-entry block (RESEARCH Pitfall 5).
        // In ALPHA mode, 'a' must append 'a', not dispatch Asin.
        if self.state.alpha_mode {
            self.handle_alpha_mode_key(key);
            return;
        }

        // Phase 5: USER mode dispatch (D-28) — check key assignments before normal routing.
        // Only active when user_mode == true. Returns true if key was consumed.
        if self.try_user_dispatch(key) {
            return;
        }

        // Phase 5: F1–F4 pre-wired USER keys a/b/c/d (D-28)
        // In USER mode: run the assigned program for a/b/c/d.
        // Outside USER mode: no-op (F1-F4 have no non-USER function in v1.0).
        if self.state.user_mode {
            let user_char = match key.code {
                KeyCode::F(1) => Some('a'),
                KeyCode::F(2) => Some('b'),
                KeyCode::F(3) => Some('c'),
                KeyCode::F(4) => Some('d'),
                _ => None,
            };
            if let Some(c) = user_char {
                if let Some(label) = self.state.key_assignments.get(&c).cloned() {
                    match hp41_core::run_program(&mut self.state, &label) {
                        Ok(()) => {
                            // Clear any stale error message from a previous dispatch, then drain
                            // card op first (may surface CARD DATA), then drain print output.
                            self.message = None;
                            self.drain_pending_card_op();
                            self.drain_and_show_print_output();
                        }
                        Err(e) => self.message = Some(format!("{e}")),
                    }
                }
                return; // consume F1-F4 in USER mode regardless of assignment
            }
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
        // Phase 8 (T-08-03, T-08-04): guards prevent malformed strings reaching flush_entry_buf.
        // Phase 9 (D-05/D-06/D-07/D-08): EEX hardware fidelity — implicit "1" mantissa on
        // empty-buffer EEX, 2-digit exponent cap, double-EEX still blocked.
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_digit() {
                // Phase 9 D-05/D-06: cap exponent entry at 2 digits. If entry_buf contains 'e',
                // count the digits AFTER 'e' (excluding optional leading '-'). If count >= 2,
                // silently ignore the new digit (no message, no beep).
                if let Some(e_pos) = self.state.entry_buf.find('e') {
                    let after_e = &self.state.entry_buf[e_pos + 1..];
                    let exp_digit_count = after_e.chars().filter(|ch| ch.is_ascii_digit()).count();
                    if exp_digit_count >= 2 {
                        return; // silently block 3rd exponent digit
                    }
                }
                self.state.entry_buf.push(c);
                self.message = None;
                return;
            }
            if c == '.' {
                // Block duplicate decimal point, and '.' after 'e' (exponent is integer-only)
                if self.state.entry_buf.contains('.') || self.state.entry_buf.contains('e') {
                    return; // silently ignore malformed input
                }
                self.state.entry_buf.push('.');
                self.message = None;
                return;
            }
            if c == 'e' {
                // Phase 9 D-08: still block double-EEX (entry_buf already contains 'e').
                if self.state.entry_buf.contains('e') {
                    return; // silently ignore second EEX
                }
                // Phase 9 D-07: empty-buffer EEX inserts implicit "1" mantissa.
                // HP-41 hardware shows "1   _" in this state; we set entry_buf = "1e" and
                // let format_entry_buf_display in ui.rs render it as "1E_ _".
                if self.state.entry_buf.is_empty() {
                    self.state.entry_buf.push_str("1e");
                } else {
                    self.state.entry_buf.push('e');
                }
                self.message = None;
                return;
            }
            if c == 'n' && self.state.entry_buf.contains('e') {
                // CHS during EEX entry: toggle exponent sign in-place — no flush, no dispatch.
                // HP-41 hardware behavior: CHS while in EEX mode toggles the exponent sign.
                // Find the 'e' position; everything after it is the (optional signed) exponent.
                if let Some(e_pos) = self.state.entry_buf.find('e') {
                    let after_e = &self.state.entry_buf[e_pos + 1..];
                    if after_e.starts_with('-') {
                        // Remove the minus: "1e-2" → "1e2", "1e-" → "1e"
                        self.state.entry_buf.remove(e_pos + 1);
                    } else {
                        // Insert minus: "1e2" → "1e-2", "1e" → "1e-"
                        self.state.entry_buf.insert(e_pos + 1, '-');
                    }
                }
                self.message = None;
                return;
            }
        }

        // D-10: 'd' cycles angle mode DEG → RAD → GRAD
        if key.code == KeyCode::Char('d') {
            let next_op = match self.state.angle_mode {
                AngleMode::Deg => Op::SetRad,
                AngleMode::Rad => Op::SetGrad,
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
                Ok(()) => {
                    // Clear any stale error message from a previous dispatch, then drain
                    // card op first (may surface CARD DATA), then drain print output.
                    self.message = None;
                    self.drain_pending_card_op();
                    self.drain_and_show_print_output();
                }
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
            Some(PendingInput::StoRegister(ref acc)) => {
                // [Phase 12 D-08] M/N/O dispatch — ONLY valid as FIRST char (acc.is_empty() guard).
                // If user has already typed a digit (e.g., "0"), ignore M/N/O — there's no
                // valid register "0M" and we want plain numbered registers to keep working.
                if acc.is_empty() {
                    match key.code {
                        KeyCode::Char('M') | KeyCode::Char('m') => {
                            self.call_dispatch(Op::StoM);
                            self.pending_input = None;
                            return;
                        }
                        KeyCode::Char('N') | KeyCode::Char('n') => {
                            self.call_dispatch(Op::StoN);
                            self.pending_input = None;
                            return;
                        }
                        KeyCode::Char('O') | KeyCode::Char('o') => {
                            self.call_dispatch(Op::StoO);
                            self.pending_input = None;
                            return;
                        }
                        _ => {} // fall through to existing arithmetic-key + handle_reg_modal logic
                    }
                }
                // Step 2: intercept arithmetic op keys before delegating to digit accumulator.
                match key.code {
                    KeyCode::Char('+') => {
                        self.pending_input = Some(PendingInput::StoAdd(String::new()));
                    }
                    KeyCode::Char('-') => {
                        self.pending_input = Some(PendingInput::StoSub(String::new()));
                    }
                    KeyCode::Char('*') => {
                        self.pending_input = Some(PendingInput::StoMul(String::new()));
                    }
                    KeyCode::Char('/') => {
                        self.pending_input = Some(PendingInput::StoDiv(String::new()));
                    }
                    _ => self.handle_reg_modal(
                        key,
                        acc.clone(),
                        Op::StoReg,
                        PendingInput::StoRegister,
                    ),
                }
            }
            Some(PendingInput::RclRegister(ref acc)) => {
                // [Phase 12 D-08] M/N/O dispatch — first-char only.
                if acc.is_empty() {
                    match key.code {
                        KeyCode::Char('M') | KeyCode::Char('m') => {
                            self.call_dispatch(Op::RclM);
                            self.pending_input = None;
                            return;
                        }
                        KeyCode::Char('N') | KeyCode::Char('n') => {
                            self.call_dispatch(Op::RclN);
                            self.pending_input = None;
                            return;
                        }
                        KeyCode::Char('O') | KeyCode::Char('o') => {
                            self.call_dispatch(Op::RclO);
                            self.pending_input = None;
                            return;
                        }
                        _ => {} // fall through to handle_reg_modal
                    }
                }
                self.handle_reg_modal(key, acc.clone(), Op::RclReg, PendingInput::RclRegister)
            }
            Some(PendingInput::StoAdd(ref acc)) => {
                // Step 3: Y/Z/T/L dispatch to stack registers immediately.
                match key.code {
                    KeyCode::Char('Y') | KeyCode::Char('y') => {
                        self.call_dispatch(Op::StoArithStack {
                            kind: StoArithKind::Add,
                            stack_reg: StackReg::Y,
                        });
                        self.pending_input = None;
                    }
                    KeyCode::Char('Z') | KeyCode::Char('z') => {
                        self.call_dispatch(Op::StoArithStack {
                            kind: StoArithKind::Add,
                            stack_reg: StackReg::Z,
                        });
                        self.pending_input = None;
                    }
                    KeyCode::Char('T') | KeyCode::Char('t') => {
                        self.call_dispatch(Op::StoArithStack {
                            kind: StoArithKind::Add,
                            stack_reg: StackReg::T,
                        });
                        self.pending_input = None;
                    }
                    KeyCode::Char('L') | KeyCode::Char('l') => {
                        self.call_dispatch(Op::StoArithStack {
                            kind: StoArithKind::Add,
                            stack_reg: StackReg::LastX,
                        });
                        self.pending_input = None;
                    }
                    _ => self.handle_reg_modal(
                        key,
                        acc.clone(),
                        |reg| Op::StoArith {
                            reg,
                            kind: StoArithKind::Add,
                        },
                        PendingInput::StoAdd,
                    ),
                }
            }
            Some(PendingInput::StoSub(ref acc)) => match key.code {
                KeyCode::Char('Y') | KeyCode::Char('y') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Sub,
                        stack_reg: StackReg::Y,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('Z') | KeyCode::Char('z') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Sub,
                        stack_reg: StackReg::Z,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('T') | KeyCode::Char('t') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Sub,
                        stack_reg: StackReg::T,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('L') | KeyCode::Char('l') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Sub,
                        stack_reg: StackReg::LastX,
                    });
                    self.pending_input = None;
                }
                _ => self.handle_reg_modal(
                    key,
                    acc.clone(),
                    |reg| Op::StoArith {
                        reg,
                        kind: StoArithKind::Sub,
                    },
                    PendingInput::StoSub,
                ),
            },
            Some(PendingInput::StoMul(ref acc)) => match key.code {
                KeyCode::Char('Y') | KeyCode::Char('y') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Mul,
                        stack_reg: StackReg::Y,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('Z') | KeyCode::Char('z') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Mul,
                        stack_reg: StackReg::Z,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('T') | KeyCode::Char('t') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Mul,
                        stack_reg: StackReg::T,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('L') | KeyCode::Char('l') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Mul,
                        stack_reg: StackReg::LastX,
                    });
                    self.pending_input = None;
                }
                _ => self.handle_reg_modal(
                    key,
                    acc.clone(),
                    |reg| Op::StoArith {
                        reg,
                        kind: StoArithKind::Mul,
                    },
                    PendingInput::StoMul,
                ),
            },
            Some(PendingInput::StoDiv(ref acc)) => match key.code {
                KeyCode::Char('Y') | KeyCode::Char('y') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Div,
                        stack_reg: StackReg::Y,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('Z') | KeyCode::Char('z') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Div,
                        stack_reg: StackReg::Z,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('T') | KeyCode::Char('t') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Div,
                        stack_reg: StackReg::T,
                    });
                    self.pending_input = None;
                }
                KeyCode::Char('L') | KeyCode::Char('l') => {
                    self.call_dispatch(Op::StoArithStack {
                        kind: StoArithKind::Div,
                        stack_reg: StackReg::LastX,
                    });
                    self.pending_input = None;
                }
                _ => self.handle_reg_modal(
                    key,
                    acc.clone(),
                    |reg| Op::StoArith {
                        reg,
                        kind: StoArithKind::Div,
                    },
                    PendingInput::StoDiv,
                ),
            },
            Some(PendingInput::AssignKey) => {
                // D-27 step 1: waiting for any printable char
                match key.code {
                    KeyCode::Esc => {
                        self.pending_input = None;
                    }
                    KeyCode::Char(c) => {
                        self.pending_input = Some(PendingInput::AssignLabel(c, String::new()));
                    }
                    _ => {
                        self.pending_input = Some(PendingInput::AssignKey);
                    }
                }
            }
            Some(PendingInput::AssignLabel(c, ref acc)) => {
                // D-27 step 2: accumulating label name
                match key.code {
                    KeyCode::Esc => {
                        self.pending_input = None;
                    }
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
                    _ => {
                        self.pending_input = Some(PendingInput::AssignLabel(c, acc.clone()));
                    }
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
            Some(PendingInput::FmtDigits(mode)) => {
                // Digit-count modal: 0–9 dispatches FmtFix/Sci/Eng(n), 'f' cycles mode,
                // Esc cancels. Any other key silently restores the modal.
                match key.code {
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let n = c as u8 - b'0';
                        let op = match mode {
                            hp41_core::DisplayMode::Fix(_) => Op::FmtFix(n),
                            hp41_core::DisplayMode::Sci(_) => Op::FmtSci(n),
                            hp41_core::DisplayMode::Eng(_) => Op::FmtEng(n),
                        };
                        self.call_dispatch(op);
                        self.pending_input = None;
                    }
                    KeyCode::Char('f') => {
                        // Cycle the pending format type: Fix → Sci → Eng → Fix.
                        // The digit count in the stored mode is irrelevant — only the variant matters.
                        let new_mode = match mode {
                            hp41_core::DisplayMode::Fix(n) => hp41_core::DisplayMode::Sci(n),
                            hp41_core::DisplayMode::Sci(n) => hp41_core::DisplayMode::Eng(n),
                            hp41_core::DisplayMode::Eng(n) => hp41_core::DisplayMode::Fix(n),
                        };
                        self.pending_input = Some(PendingInput::FmtDigits(new_mode));
                    }
                    KeyCode::Esc => {
                        self.pending_input = None;
                    }
                    _ => {
                        // Restore modal silently for unrecognized keys
                        self.pending_input = Some(PendingInput::FmtDigits(mode));
                    }
                }
            }
            Some(PendingInput::PrintModal) => {
                match key.code {
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        self.call_dispatch_and_drain(Op::PRX);
                        self.pending_input = None;
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.call_dispatch_and_drain(Op::PRA);
                        self.pending_input = None;
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        self.call_dispatch_and_drain(Op::PRSTK);
                        self.pending_input = None;
                    }
                    KeyCode::Esc => {
                        self.pending_input = None;
                    }
                    _ => {
                        // Silently ignore unrecognized keys — keep modal open (existing convention).
                        self.pending_input = Some(PendingInput::PrintModal);
                    }
                }
            }
            Some(PendingInput::HexModal(ref acc)) => {
                match key.code {
                    KeyCode::Char(c) if c.is_ascii_hexdigit() => {
                        // Normalize to uppercase for display consistency.
                        let hex_char = c.to_ascii_uppercase();
                        let mut new_acc = acc.clone();
                        new_acc.push(hex_char);
                        if new_acc.len() == 2 {
                            // Two ASCII hex chars always parse as u8 — invariant guarantees no panic.
                            let byte = u8::from_str_radix(&new_acc, 16)
                                .expect("two ASCII hex digits must parse as u8");
                            match synthetic_byte_to_op(byte) {
                                Some(_) => {
                                    // Clamp pc to program.len(): Vec::insert panics when index > len.
                                    // state.pc can be program.len()+1 after ISG/DSE skip on the last step.
                                    let insert_pos = self.state.pc.min(self.state.program.len());
                                    self.state
                                        .program
                                        .insert(insert_pos, Op::SyntheticByte(byte));
                                    self.state.pc = insert_pos + 1;
                                    self.message = None;
                                }
                                None => {
                                    // [Phase 12 D-13] Rejected: signal to user, leave program Vec untouched.
                                    self.message = Some("INVALID".to_string());
                                }
                            }
                            // [D-13] Modal always closes after the second digit (valid or not).
                            self.pending_input = None;
                        } else {
                            // First digit accumulated — keep modal open for second digit.
                            self.pending_input = Some(PendingInput::HexModal(new_acc));
                        }
                    }
                    KeyCode::Esc => {
                        // Cancel — no side effects (program unchanged, no INVALID message).
                        self.pending_input = None;
                    }
                    _ => {
                        // Non-hex key: keep modal open silently (existing modal convention).
                        self.pending_input = Some(PendingInput::HexModal(acc.clone()));
                    }
                }
            }
            None => unreachable!("handle_pending_input called with no pending input — caller must check is_some() first"),
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
                    // Auto-dispatch on second digit (D-09).
                    // Two ASCII digit chars always parse as u8 in 0–99 — no fallback needed.
                    let reg: u8 = new_acc
                        .parse()
                        .expect("two ASCII digit chars always parse as u8 ≤ 99");
                    self.call_dispatch(op_fn(reg));
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
            KeyCode::Delete => {
                // Phase 8: Delete key in ALPHA mode clears the entire ALPHA register (D-03)
                self.call_dispatch(Op::AlphaClear);
            }
            _ => {
                // Other keys (arrows, F-keys, etc.) ignored in ALPHA mode
            }
        }
    }

    /// USER mode dispatch: if user_mode is active and the key has an assignment,
    /// run the assigned program label and return true (key consumed).
    /// Returns false if user_mode is off or no assignment exists for this key.
    /// D-28: USER mode key dispatch via key_assignments BTreeMap.
    fn try_user_dispatch(&mut self, key: KeyEvent) -> bool {
        if !self.state.user_mode {
            return false;
        }
        if let KeyCode::Char(c) = key.code {
            if let Some(label) = self.state.key_assignments.get(&c).cloned() {
                match hp41_core::run_program(&mut self.state, &label) {
                    Ok(()) => {
                        // Clear any stale error message from a previous dispatch, then drain
                        // card op first (may surface CARD DATA), then drain print output.
                        self.message = None;
                        self.drain_pending_card_op();
                        self.drain_and_show_print_output();
                    }
                    Err(e) => self.message = Some(format!("{e}")),
                }
                return true; // consumed
            }
        }
        false // not consumed — fall through to normal routing
    }

    /// Drain the staged Card Reader request (if any), performing the disk I/O.
    ///
    /// Mirrors `drain_and_show_print_output` — same call sites, surfaces
    /// `HpError::CardData(msg)` into `self.message` so the CLI display shows
    /// "CARD DATA" with a diagnostic suffix.
    fn drain_pending_card_op(&mut self) {
        if let Err(e) = crate::cards::drain_pending_card_op(&mut self.state, &self.cards_dir) {
            self.message = Some(format!("{e}"));
        }
    }

    /// Drain print_buffer after a run_program() Ok(()) return and surface output in the TUI.
    ///
    /// Mirrors the drain branch inside call_dispatch_and_drain but decoupled from dispatch —
    /// called after run_program() has already returned successfully.
    ///
    /// For 1 line (PRX/PRA): sets app.message to the formatted line.
    /// For N > 1 lines (PRSTK or multiple print ops in one program): sets app.message to
    ///   "PRSTK → N lines" summary consistent with D-01.
    /// If print_log_writer is Some, writes each line to the file via
    /// `write_lines_to_print_log()` which disables the writer on first I/O error.
    /// Clears print_buffer via drain(..).
    fn drain_and_show_print_output(&mut self) {
        let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
        if !lines.is_empty() {
            let log_failure = self.write_lines_to_print_log(&lines);
            let summary = if lines.len() > 1 {
                format!("PRSTK \u{2192} {} lines", lines.len())
            } else {
                lines.into_iter().next().unwrap_or_default()
            };
            self.message = Some(match log_failure {
                Some(err) => format!("{summary} ({err})"),
                None => summary,
            });
        }
        // If lines is empty, leave self.message as None (caller already set it to None
        // on the Ok(()) branch before calling this helper).
    }

    /// Write a batch of print lines to `print_log_writer`. On the first I/O error
    /// (write or flush) disables the writer (sets to `None`) and returns a one-shot
    /// `"print log disabled: {err}"` message so the caller can append it to
    /// `self.message`. After this returns `Some(_)` once, subsequent calls are no-ops.
    ///
    /// This replaces the original `let _ = writeln!(...)` pattern that silently
    /// dropped every line after the first failure (PR #5 silent-failure review).
    fn write_lines_to_print_log(&mut self, lines: &[String]) -> Option<String> {
        let failure: Option<String> = {
            let writer = self.print_log_writer.as_mut()?;
            let mut err_msg: Option<String> = None;
            for line in lines {
                if let Err(e) = writeln!(writer, "{line}") {
                    err_msg = Some(format!("print log disabled: {e}"));
                    break;
                }
            }
            if err_msg.is_none() {
                if let Err(e) = writer.flush() {
                    err_msg = Some(format!("print log disabled: {e}"));
                }
            }
            err_msg
        };
        if failure.is_some() {
            self.print_log_writer = None;
        }
        failure
    }

    /// Call hp41_core::ops::dispatch and map any HpError to self.message.
    fn call_dispatch(&mut self, op: Op) {
        match hp41_core::ops::dispatch(&mut self.state, op) {
            Ok(()) => self.message = None,
            Err(e) => self.message = Some(format!("{e}")),
        }
    }

    /// Call hp41_core::ops::dispatch, then drain card op and print_buffer.
    /// For PRX/PRA (1 line): sets app.message to the formatted line (per D-01).
    /// For PRSTK (6 lines): sets app.message to "PRSTK → N lines" summary (per D-01).
    /// If print_log_writer is Some, writes each line to the file (best-effort, never panics).
    pub(crate) fn call_dispatch_and_drain(&mut self, op: Op) {
        match hp41_core::ops::dispatch(&mut self.state, op) {
            Ok(()) => {
                self.drain_pending_card_op();
                let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
                if !lines.is_empty() {
                    let log_failure = self.write_lines_to_print_log(&lines);
                    let summary = if lines.len() > 1 {
                        format!("PRSTK \u{2192} {} lines", lines.len())
                    } else {
                        // lines.len() == 1; into_iter().next() is safe here
                        lines.into_iter().next().unwrap_or_default()
                    };
                    self.message = Some(match log_failure {
                        Some(err) => format!("{summary} ({err})"),
                        None => summary,
                    });
                }
            }
            Err(e) => self.message = Some(format!("{e}")),
        }
    }
}

#[cfg(test)]
impl App {
    /// Test-only constructor: builds a minimal App suitable for handle_key
    /// integration tests. Uses a temporary state path; persistence side effects
    /// are harmless because tests do not call the auto-save path.
    pub fn new_for_test() -> Self {
        App::new(
            CalcState::new(),
            PathBuf::from("/tmp/hp41-cli-test-state.json"),
            None,
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        let mut app = App::new(state, state_path.clone(), None);

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
        let mut app = App::new(state, state_path.clone(), None);

        // last_save is Instant::now() — only 0ms elapsed; should NOT save.
        app.check_autosave();

        assert!(
            !state_path.exists(),
            "check_autosave() must NOT save before 30s have elapsed"
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    fn make_app() -> App {
        App::new(
            hp41_core::CalcState::new(),
            std::path::PathBuf::from("/tmp/hp41_test_app.json"),
            None,
        )
    }

    /// D-28: try_user_dispatch() returns true and runs program when user_mode is active
    /// and the pressed key has an assignment in key_assignments.
    #[test]
    fn test_user_mode_dispatch_runs_program() {
        let mut app = make_app();
        // Set up a simple program under label "A"
        app.state.program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(hp41_core::HpNum::from(42i32)),
            Op::Rtn,
        ];
        // Assign 'z' → "A"
        app.state.key_assignments.insert('z', "A".to_string());
        app.state.user_mode = true;

        // Simulate pressing 'z' with user_mode on
        let result = app.try_user_dispatch(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(
            result,
            "try_user_dispatch must return true when key is assigned"
        );
        // Program should have run and pushed 42 onto stack
        assert!(
            !app.state.stack.x.is_zero(),
            "program should have pushed 42 to X"
        );
    }

    /// D-28: try_user_dispatch() returns false when user_mode is off — normal routing applies.
    #[test]
    fn test_user_mode_dispatch_skipped_when_off() {
        let mut app = make_app();
        app.state.key_assignments.insert('z', "A".to_string());
        app.state.user_mode = false; // USER mode OFF

        let result = app.try_user_dispatch(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(
            !result,
            "try_user_dispatch must return false when user_mode is off"
        );
    }

    /// D-28: try_user_dispatch() returns false when user_mode is on but key has no assignment.
    #[test]
    fn test_user_mode_dispatch_no_assignment() {
        let mut app = make_app();
        app.state.user_mode = true;
        // No key assignments at all

        let result = app.try_user_dispatch(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('z'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(
            !result,
            "try_user_dispatch must return false when key has no assignment"
        );
    }

    // Helper — create a Press key event with no modifiers.
    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_q_does_not_quit_when_help_overlay_open() {
        // SC-3 gap closure: 'q' must close the overlay, NOT set exit=true.
        let mut app = make_app();
        app.show_help = true;
        app.handle_key(make_key(KeyCode::Char('q')));
        assert!(!app.exit, "'q' must not quit when help overlay is open");
        assert!(!app.show_help, "'q' must close the help overlay");
    }

    #[test]
    fn test_ctrl_c_still_quits() {
        // Phase 8: Ctrl+C remains the sole quit key after 'q' reassignment to SIN.
        let mut app = make_app();
        app.handle_key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert!(app.exit, "Ctrl+C must still quit the app");
    }

    #[test]
    fn test_q_does_not_quit_when_programs_overlay_open() {
        // 'q' is not bound to close the programs overlay (only Esc is), but it must
        // not set exit=true either.
        let mut app = make_app();
        app.show_programs = true;
        app.handle_key(make_key(KeyCode::Char('q')));
        assert!(!app.exit, "'q' must not quit when programs overlay is open");
    }

    // Phase 8: new behavior tests (RED — fail until implementation is added)

    #[test]
    fn test_q_no_longer_quits_in_normal_mode() {
        // Phase 8: 'q' is now SIN, not quit. exit must stay false.
        let mut app = make_app();
        app.handle_key(make_key(KeyCode::Char('q')));
        assert!(!app.exit, "'q' must not quit after reassignment to SIN");
    }

    #[test]
    fn test_delete_in_alpha_mode_clears_alpha_register() {
        // Phase 8: Delete in ALPHA mode dispatches Op::AlphaClear
        let mut app = make_app();
        app.state.alpha_mode = true;
        app.state.alpha_reg = "HELLO".to_string();
        app.handle_key(make_key(KeyCode::Delete));
        assert!(
            app.state.alpha_reg.is_empty(),
            "Delete in ALPHA mode must clear alpha_reg"
        );
    }

    #[test]
    fn test_eex_on_empty_entry_buf_inserts_implicit_one() {
        // Phase 9 D-07: pressing 'e' on empty buffer inserts "1e" (implicit mantissa).
        // The old behavior (blocking EEX when empty) was removed in Phase 9.
        let mut app = make_app();
        assert!(app.state.entry_buf.is_empty());
        app.handle_key(make_key(KeyCode::Char('e')));
        assert_eq!(
            app.state.entry_buf, "1e",
            "'e' on empty entry_buf must insert implicit mantissa \"1e\" (D-07)"
        );
    }

    #[test]
    fn test_eex_blocked_when_already_present() {
        let mut app = make_app();
        app.state.entry_buf = "1.5e".to_string();
        app.handle_key(make_key(KeyCode::Char('e')));
        assert_eq!(
            app.state.entry_buf, "1.5e",
            "'e' must not be appended when 'e' already present"
        );
    }

    #[test]
    fn test_decimal_blocked_when_already_present() {
        let mut app = make_app();
        app.state.entry_buf = "1.5".to_string();
        app.handle_key(make_key(KeyCode::Char('.')));
        assert_eq!(
            app.state.entry_buf, "1.5",
            "'.' must not be appended when '.' already present"
        );
    }

    #[test]
    fn test_decimal_blocked_after_eex() {
        let mut app = make_app();
        app.state.entry_buf = "1.5e2".to_string();
        app.handle_key(make_key(KeyCode::Char('.')));
        assert_eq!(
            app.state.entry_buf, "1.5e2",
            "'.' must not be appended after 'e' already present"
        );
    }

    #[test]
    fn test_eex_appended_when_valid() {
        let mut app = make_app();
        app.state.entry_buf = "1.5".to_string();
        app.handle_key(make_key(KeyCode::Char('e')));
        assert_eq!(
            app.state.entry_buf, "1.5e",
            "'e' must append when entry_buf is non-empty and has no 'e'"
        );
    }

    #[test]
    fn test_delete_outside_alpha_mode_is_noop() {
        // Delete is only routed to Op::AlphaClear inside handle_alpha_mode_key.
        // Outside ALPHA mode it must not modify the stack X register.
        let mut app = make_app();
        // alpha_mode is false by default
        app.state.stack.x = hp41_core::HpNum::from(7);
        app.handle_key(make_key(KeyCode::Delete));
        assert_eq!(
            format!("{}", app.state.stack.x),
            "7",
            "Delete outside ALPHA mode must not modify stack X"
        );
    }

    #[test]
    fn test_q_close_help_does_not_dispatch_sin() {
        // 'q' closes the help overlay via an early-return guard. Op::Sin must NOT fire.
        let mut app = make_app();
        app.show_help = true;
        app.state.stack.x = hp41_core::HpNum::from(30);
        app.handle_key(make_key(KeyCode::Char('q')));
        assert!(!app.show_help, "'q' must close the help overlay");
        assert_eq!(
            format!("{}", app.state.stack.x),
            "30",
            "'q' closing help must not dispatch Op::Sin (x must remain 30, not become 0.5)"
        );
    }

    // ── CHS during EEX entry (exponent sign toggle) ──────────────────────────

    #[test]
    fn test_chs_eex_inserts_minus_on_bare_e() {
        let mut app = make_app();
        app.handle_key(make_key(KeyCode::Char('1')));
        app.handle_key(make_key(KeyCode::Char('e')));
        assert_eq!(app.state.entry_buf, "1e");
        app.handle_key(make_key(KeyCode::Char('n')));
        assert_eq!(app.state.entry_buf, "1e-");
        assert!(app.message.is_none());
    }

    #[test]
    fn test_chs_eex_removes_minus_on_e_minus() {
        let mut app = make_app();
        app.handle_key(make_key(KeyCode::Char('1')));
        app.handle_key(make_key(KeyCode::Char('e')));
        app.handle_key(make_key(KeyCode::Char('n'))); // → "1e-"
        app.handle_key(make_key(KeyCode::Char('n'))); // → "1e" (toggle back)
        assert_eq!(app.state.entry_buf, "1e");
    }

    #[test]
    fn test_chs_eex_inserts_minus_with_digits() {
        let mut app = make_app();
        app.handle_key(make_key(KeyCode::Char('1')));
        app.handle_key(make_key(KeyCode::Char('e')));
        app.handle_key(make_key(KeyCode::Char('2')));
        assert_eq!(app.state.entry_buf, "1e2");
        app.handle_key(make_key(KeyCode::Char('n')));
        assert_eq!(app.state.entry_buf, "1e-2");
    }

    #[test]
    fn test_chs_eex_removes_minus_with_digits() {
        let mut app = make_app();
        app.handle_key(make_key(KeyCode::Char('1')));
        app.handle_key(make_key(KeyCode::Char('e')));
        app.handle_key(make_key(KeyCode::Char('n'))); // "1e-"
        app.handle_key(make_key(KeyCode::Char('2'))); // "1e-2"
        app.handle_key(make_key(KeyCode::Char('n'))); // "1e2"
        assert_eq!(app.state.entry_buf, "1e2");
    }

    #[test]
    fn test_chs_eex_toggle_long_mantissa() {
        let mut app = make_app();
        // Build "1.5e-23" then toggle → "1.5e23"
        for c in "1.5e".chars() {
            app.handle_key(make_key(KeyCode::Char(c)));
        }
        app.handle_key(make_key(KeyCode::Char('n'))); // "1.5e-"
        app.handle_key(make_key(KeyCode::Char('2'))); // "1.5e-2"
        app.handle_key(make_key(KeyCode::Char('3'))); // "1.5e-23"
        assert_eq!(app.state.entry_buf, "1.5e-23");
        app.handle_key(make_key(KeyCode::Char('n')));
        assert_eq!(app.state.entry_buf, "1.5e23");
    }

    #[test]
    fn test_integration_1_eex_chs_2_enter_gives_0_01() {
        // Full keystroke sequence: 1, EEX, CHS, 2, Enter → X = 1E-2 = 0.01
        let mut app = make_app();
        app.handle_key(make_key(KeyCode::Char('1'))); // entry_buf = "1"
        app.handle_key(make_key(KeyCode::Char('e'))); // entry_buf = "1e"
        app.handle_key(make_key(KeyCode::Char('n'))); // entry_buf = "1e-"
        app.handle_key(make_key(KeyCode::Char('2'))); // entry_buf = "1e-2"
                                                      // Simulate Enter by dispatching Op::Enter directly
        app.call_dispatch(Op::Enter);
        assert!(
            app.state.entry_buf.is_empty(),
            "entry_buf must flush on Enter"
        );
        // X should be 0.01 = 1E-2. Verify via display format (FIX 4 default).
        let formatted = hp41_core::format_hpnum(&app.state.stack.x, &app.state.display_mode);
        assert_eq!(
            formatted, "0.0100",
            "1 EEX CHS 2 Enter must yield 0.01 (shown as 0.0100 in FIX 4)"
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod eex_integration_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use hp41_core::ops::dispatch;

    fn make_app() -> App {
        // Construct a minimal App with a default CalcState. The persistence path
        // and TUI fields don't matter for these tests — we only exercise handle_key
        // and the cli/core integration via dispatch.
        App::new_for_test()
    }

    fn key(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_eex_trailing_e_then_enter_pushes_mantissa() {
        // Phase 9 success criterion #1: "1.5e" + ENTER pushes 1.5 to X (exponent 00).
        let mut app = make_app();
        app.handle_key(key('1'));
        app.handle_key(key('.'));
        app.handle_key(key('5'));
        app.handle_key(key('e'));
        assert_eq!(app.state.entry_buf, "1.5e");
        // Now dispatch Enter — flush_entry_buf normalizes "1.5e" -> "1.5e00" -> 1.5.
        dispatch(&mut app.state, Op::Enter).expect("Enter must succeed");
        // Use format_hpnum with the state's display mode (Fix(4) by default) to
        // verify the pushed value is 1.5. format_hpnum is re-exported by hp41-core.
        let formatted = hp41_core::format_hpnum(&app.state.stack.x, &app.state.display_mode);
        assert_eq!(
            formatted, "1.5000",
            "EEX trailing-e must push mantissa 1.5 (shown as 1.5000 in FIX 4)"
        );
        assert!(app.state.entry_buf.is_empty());
    }

    #[test]
    fn test_empty_buffer_eex_inserts_implicit_one() {
        // Phase 9 D-07: pressing 'e' on an empty buffer inserts "1e".
        let mut app = make_app();
        assert!(app.state.entry_buf.is_empty());
        app.handle_key(key('e'));
        assert_eq!(app.state.entry_buf, "1e");
    }

    #[test]
    fn test_exponent_digit_cap_blocks_third_digit() {
        // Phase 9 D-05/D-06: 3rd exponent digit silently blocked.
        let mut app = make_app();
        app.handle_key(key('1'));
        app.handle_key(key('.'));
        app.handle_key(key('5'));
        app.handle_key(key('e'));
        app.handle_key(key('2'));
        app.handle_key(key('3'));
        assert_eq!(app.state.entry_buf, "1.5e23");
        // Third digit must be silently ignored.
        app.handle_key(key('4'));
        assert_eq!(app.state.entry_buf, "1.5e23");
        assert!(app.message.is_none(), "silent block — no message set");
    }

    #[test]
    fn test_double_eex_blocked() {
        // Phase 9 D-08: pressing 'e' when entry_buf already contains 'e' is ignored.
        let mut app = make_app();
        app.handle_key(key('1'));
        app.handle_key(key('.'));
        app.handle_key(key('5'));
        app.handle_key(key('e'));
        assert_eq!(app.state.entry_buf, "1.5e");
        app.handle_key(key('e'));
        assert_eq!(app.state.entry_buf, "1.5e");
        assert!(app.message.is_none(), "silent block — no message set");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod print_modal_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use hp41_core::ops::Op;

    #[allow(dead_code)]
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_print_modal_prx_sets_message() {
        let mut app = App::new_for_test();
        // Push a value onto the stack
        hp41_core::ops::dispatch(&mut app.state, Op::PushNum(hp41_core::HpNum::from(42))).unwrap();
        // Simulate 'P' key (opens modal)
        let p_key = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE);
        app.handle_key(p_key);
        assert!(
            matches!(app.pending_input, Some(PendingInput::PrintModal)),
            "Pressing 'P' must set PendingInput::PrintModal"
        );
        // Simulate 'x' key (dispatches PRX)
        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        app.handle_key(x_key);
        assert!(
            app.pending_input.is_none(),
            "After PrintModal 'x', pending_input must be None"
        );
        assert!(
            app.message.is_some(),
            "After PRX, app.message must contain the formatted line"
        );
        let msg = app.message.as_deref().unwrap_or("");
        assert_eq!(msg.len(), 24, "PRX message must be 24 chars, got {msg:?}");
    }

    #[test]
    fn test_print_modal_esc_cancels_without_dispatch() {
        let mut app = App::new_for_test();
        hp41_core::ops::dispatch(&mut app.state, Op::PushNum(hp41_core::HpNum::from(5))).unwrap();
        let x_before = app.state.stack.x.clone();
        // Open modal
        let p_key = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::NONE);
        app.handle_key(p_key);
        // Cancel
        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key(esc_key);
        assert!(app.pending_input.is_none(), "Esc must clear pending_input");
        assert!(
            app.state.print_buffer.is_empty(),
            "Esc must not dispatch any print op"
        );
        assert_eq!(app.state.stack.x, x_before, "Esc must not modify stack");
    }

    #[test]
    fn test_print_log_file_append() {
        let tmp_dir = std::env::temp_dir().join("hp41_print_log_test");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let log_path = tmp_dir.join("print.txt");
        // Remove if leftover from prior run
        let _ = std::fs::remove_file(&log_path);

        let mut app = App::new(
            CalcState::new(),
            PathBuf::from("/tmp/hp41-cli-test-state.json"),
            Some(log_path.clone()),
        );
        hp41_core::ops::dispatch(&mut app.state, Op::PushNum(hp41_core::HpNum::from(99))).unwrap();
        // Trigger PRX via call_dispatch_and_drain directly
        app.call_dispatch_and_drain(Op::PRX);

        // Flush should have been called in call_dispatch_and_drain
        let contents = std::fs::read_to_string(&log_path).unwrap();
        assert!(
            !contents.is_empty(),
            "print log file must have content after PRX"
        );
        assert_eq!(
            contents.lines().count(),
            1,
            "PRX must write exactly 1 line to the log file"
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_print_log_invalid_path_sets_message() {
        // Use a path that cannot be created (directory that does not exist, read-only root)
        let bad_path = std::path::PathBuf::from("/no_such_dir_hp41/print.log");
        let app = App::new(
            CalcState::new(),
            PathBuf::from("/tmp/hp41-cli-test-state.json"),
            Some(bad_path),
        );
        assert!(
            app.print_log_writer.is_none(),
            "App::new with invalid --print-log path must set print_log_writer = None"
        );
        assert!(
            app.message.is_some(),
            "App::new with invalid --print-log path must set app.message to an error string"
        );
        let msg = app.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("Warning") || msg.contains("cannot open") || msg.contains("print log"),
            "Error message must describe the open failure, got: {msg:?}"
        );
    }
}

/// Phase 12 CLI wiring tests: last_key_code, HexModal, and M/N/O modal extensions.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod synthetic_modal_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    fn make_app() -> App {
        App::new_for_test()
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_last_key_code_updated_on_press() {
        let mut app = make_app();
        // Press '5' (KeyCode::Char('5')) — keycode_to_hp41_code maps to 62 (row 6, col 2).
        app.handle_key(press(KeyCode::Char('5')));
        assert_eq!(
            app.state.last_key_code, 62,
            "Press '5' must set last_key_code = 62"
        );
    }

    #[test]
    fn test_hex_modal_opens_only_in_prgm_mode() {
        let mut app = make_app();
        app.state.prgm_mode = false;
        app.handle_key(press(KeyCode::Char('X')));
        assert!(
            !matches!(app.pending_input, Some(PendingInput::HexModal(_))),
            "HexModal must NOT open outside PRGM mode"
        );

        let mut app2 = make_app();
        app2.state.prgm_mode = true;
        app2.handle_key(press(KeyCode::Char('X')));
        assert!(
            matches!(app2.pending_input, Some(PendingInput::HexModal(ref s)) if s.is_empty()),
            "HexModal must open with empty accumulator in PRGM mode"
        );
    }

    #[test]
    fn test_hex_modal_invalid_byte_rejects_with_message() {
        let mut app = make_app();
        app.state.prgm_mode = true;
        app.pending_input = Some(PendingInput::HexModal(String::new()));
        let prog_len_before = app.state.program.len();
        // 0x00 is not in the safe subset → INVALID.
        app.handle_key(press(KeyCode::Char('0')));
        app.handle_key(press(KeyCode::Char('0')));
        assert_eq!(
            app.state.program.len(),
            prog_len_before,
            "Invalid byte must NOT modify the program Vec"
        );
        assert_eq!(
            app.message.as_deref(),
            Some("INVALID"),
            "Invalid byte must set app.message = 'INVALID'"
        );
        assert!(
            app.pending_input.is_none(),
            "HexModal must close after rejection"
        );
    }

    #[test]
    fn test_hex_modal_valid_byte_inserts_synthetic() {
        let mut app = make_app();
        app.state.prgm_mode = true;
        app.pending_input = Some(PendingInput::HexModal(String::new()));
        app.state.pc = 0;
        let pc_before = app.state.pc;
        let prog_len_before = app.state.program.len();
        // 0xCF is NULL — which is in the safe subset (maps to Op::Null).
        // Check synthetic_byte_to_op(0xCF) is Some in Wave 1 implementation.
        // If 0xCF is not in subset, use 0xCE (GetKey) which is confirmed in mod.rs.
        app.handle_key(press(KeyCode::Char('c')));
        app.handle_key(press(KeyCode::Char('e')));
        assert_eq!(
            app.state.program.len(),
            prog_len_before + 1,
            "Valid byte must insert exactly 1 step"
        );
        assert_eq!(
            app.state.program[0],
            hp41_core::ops::Op::SyntheticByte(0xCE),
            "Inserted step must be SyntheticByte(0xCE)"
        );
        assert_eq!(
            app.state.pc,
            pc_before + 1,
            "PC must advance past inserted step"
        );
        assert!(
            app.pending_input.is_none(),
            "HexModal must close after insertion"
        );
    }

    #[test]
    fn test_hex_modal_esc_cancels_cleanly() {
        let mut app = make_app();
        app.state.prgm_mode = true;
        app.pending_input = Some(PendingInput::HexModal(String::new()));
        let prog_len_before = app.state.program.len();
        app.handle_key(press(KeyCode::Esc));
        assert_eq!(
            app.state.program.len(),
            prog_len_before,
            "Esc must not modify program Vec"
        );
        assert!(app.pending_input.is_none(), "Esc must close HexModal");
        assert!(app.message.is_none(), "Esc must not set INVALID message");
    }

    #[test]
    fn test_hex_modal_esc_after_first_digit_cancels() {
        // Esc must also cancel when one hex digit has already been typed
        let mut app = make_app();
        app.state.prgm_mode = true;
        app.pending_input = Some(PendingInput::HexModal(String::new()));
        app.handle_key(press(KeyCode::Char('c'))); // first digit — acc = "c"
        let prog_len = app.state.program.len();
        app.handle_key(press(KeyCode::Esc));
        assert_eq!(
            app.state.program.len(),
            prog_len,
            "Esc must not insert anything"
        );
        assert!(app.pending_input.is_none(), "Esc must close modal");
        assert!(app.message.is_none(), "Esc must not set INVALID");
    }

    #[test]
    fn test_sto_n_via_modal() {
        let mut app = make_app();
        app.handle_key(press(KeyCode::Char('S')));
        app.state.stack.x = hp41_core::HpNum::from(55i32);
        app.handle_key(press(KeyCode::Char('N')));
        assert!(app.pending_input.is_none(), "modal must close after N");
        assert_eq!(
            app.state.reg_n,
            hp41_core::HpNum::from(55i32),
            "STO N must store X into reg_n"
        );
    }

    #[test]
    fn test_rcl_o_via_modal() {
        let mut app = make_app();
        app.state.reg_o = hp41_core::HpNum::from(77i32);
        app.handle_key(press(KeyCode::Char('R')));
        app.handle_key(press(KeyCode::Char('o')));
        assert!(app.pending_input.is_none(), "modal must close after o");
        assert_eq!(
            app.state.stack.x,
            hp41_core::HpNum::from(77i32),
            "RCL O must recall reg_o into X"
        );
    }

    #[test]
    fn test_sto_m_via_modal() {
        let mut app = make_app();
        // Press 'S' to open StoRegister modal
        app.handle_key(press(KeyCode::Char('S')));
        assert!(
            matches!(app.pending_input, Some(PendingInput::StoRegister(_))),
            "Pressing 'S' must open StoRegister modal"
        );
        // Set X to 42 before pressing 'M'
        app.state.stack.x = hp41_core::HpNum::from(42i32);
        // Press 'M' to dispatch StoM
        app.handle_key(press(KeyCode::Char('M')));
        assert!(
            app.pending_input.is_none(),
            "Modal must close after M dispatch"
        );
        assert_eq!(
            app.state.reg_m,
            hp41_core::HpNum::from(42i32),
            "STO M must store X into reg_m"
        );
    }

    #[test]
    fn test_rcl_m_via_modal() {
        let mut app = make_app();
        // Pre-load reg_m with 99
        app.state.reg_m = hp41_core::HpNum::from(99i32);
        // Press 'R' to open RclRegister modal
        app.handle_key(press(KeyCode::Char('R')));
        assert!(
            matches!(app.pending_input, Some(PendingInput::RclRegister(_))),
            "Pressing 'R' must open RclRegister modal"
        );
        // Press 'm' (lowercase) to dispatch RclM
        app.handle_key(press(KeyCode::Char('m')));
        assert!(
            app.pending_input.is_none(),
            "Modal must close after m dispatch"
        );
        assert_eq!(
            app.state.stack.x,
            hp41_core::HpNum::from(99i32),
            "RCL M must recall reg_m into X"
        );
    }

    #[test]
    fn test_mno_guard_only_when_acc_empty() {
        // If user already typed a digit '0' in StoRegister, pressing 'M' should NOT
        // dispatch StoM — it should fall through to handle_reg_modal and be ignored
        // (since '0M' is not a valid register number, handle_reg_modal will ignore 'M').
        let mut app = make_app();
        app.pending_input = Some(PendingInput::StoRegister("0".to_string()));
        app.state.stack.x = hp41_core::HpNum::from(77i32);
        app.handle_key(press(KeyCode::Char('M')));
        // reg_m must NOT be updated because acc was non-empty ("0") when M was pressed.
        assert!(
            app.state.reg_m.is_zero(),
            "STO M must not dispatch when accumulator is non-empty"
        );
    }

    #[test]
    fn test_hex_modal_insert_when_pc_past_end_does_not_panic() {
        // Reproduces ISG/DSE skip-at-end scenario where pc = program.len() + 1.
        // Vec::insert panics when index > len — the clamp in the Some(_) arm prevents this.
        let mut app = make_app();
        app.state.prgm_mode = true;
        app.state.program = vec![hp41_core::ops::Op::Null]; // len = 1
        app.state.pc = 2; // len + 1 — the dangerous value after ISG/DSE skip
        app.pending_input = Some(PendingInput::HexModal(String::new()));
        // 0xCF → Op::Null (valid) — must insert at clamped position without panic
        app.handle_key(press(KeyCode::Char('c')));
        app.handle_key(press(KeyCode::Char('f')));
        assert_eq!(app.state.program.len(), 2, "one step must be inserted");
        assert!(
            app.pending_input.is_none(),
            "HexModal must close after insertion"
        );
    }

    #[test]
    fn test_getkey_end_to_end_keypress_to_x() {
        // UAT-1: keyboard press → last_key_code → GETKEY in program via run_program → X
        // Tests the execute_op path (Op::GetKey arm in program.rs), not just dispatch.
        let mut app = make_app();
        // Press '5' — keycode_to_hp41_code maps it to 62 (row 6 × 10 + col 2)
        app.handle_key(press(KeyCode::Char('5')));
        assert_eq!(
            app.state.last_key_code, 62,
            "pressing '5' must record HP-41 code 62"
        );

        // Load program [LBL A, GetKey] and run via run_program — exercises execute_op
        app.state.program = vec![
            hp41_core::ops::Op::Lbl("A".to_string()),
            hp41_core::ops::Op::GetKey,
        ];
        hp41_core::run_program(&mut app.state, "A").unwrap();

        assert_eq!(
            app.state.stack.x,
            hp41_core::HpNum::from(62i32),
            "GETKEY in program must push last_key_code (62) into X"
        );
    }

    #[test]
    fn test_getkey_via_synthetic_byte_in_program() {
        // SyntheticByte(0xCE) → GetKey path via execute_op (HexModal insertion flow)
        let mut app = make_app();
        app.handle_key(press(KeyCode::Char('7')));
        assert_eq!(
            app.state.last_key_code, 51,
            "pressing '7' must record HP-41 code 51"
        );

        app.state.program = vec![
            hp41_core::ops::Op::Lbl("A".to_string()),
            hp41_core::ops::Op::SyntheticByte(0xCE), // 0xCE → GetKey
        ];
        hp41_core::run_program(&mut app.state, "A").unwrap();

        assert_eq!(
            app.state.stack.x,
            hp41_core::HpNum::from(51i32),
            "SyntheticByte(0xCE) in program must execute as GETKEY and push 51"
        );
    }

    // Helper — create a Ctrl-modified Press key event.
    fn make_ctrl_key(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    /// Phase 19: Ctrl+W dispatches WPRGM (write program to card).
    #[test]
    fn test_ctrl_w_dispatches_wprgm() {
        let mut app = make_app();
        app.handle_key(make_ctrl_key('w'));
        // App must not quit and must not have entered ALPHA mode.
        assert!(!app.exit, "Ctrl+W must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+W must not activate ALPHA mode");
    }

    /// Phase 19: Ctrl+R dispatches RDPRGM (read program from card).
    #[test]
    fn test_ctrl_r_dispatches_rdprgm() {
        let mut app = make_app();
        app.handle_key(make_ctrl_key('r'));
        assert!(!app.exit, "Ctrl+R must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+R must not activate ALPHA mode");
    }

    /// Phase 19: Ctrl+D dispatches WDTA (write data registers to card).
    #[test]
    fn test_ctrl_d_dispatches_wdta() {
        let mut app = make_app();
        app.handle_key(make_ctrl_key('d'));
        assert!(!app.exit, "Ctrl+D must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+D must not activate ALPHA mode");
    }

    /// Phase 19: Ctrl+F dispatches RDTA (read data registers from card).
    #[test]
    fn test_ctrl_f_dispatches_rdta() {
        let mut app = make_app();
        app.handle_key(make_ctrl_key('f'));
        assert!(!app.exit, "Ctrl+F must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+F must not activate ALPHA mode");
    }
}
