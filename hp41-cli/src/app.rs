//! App — top-level state container and event loop for hp41-cli.
//!
//! App owns CalcState and acts as the controller between crossterm key events
//! and hp41-core's dispatch() entry point.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use hp41_core::ops::Op;
use hp41_core::{CalcState, AngleMode, DisplayMode};

use crate::{keys, ui};

/// Top-level application state. Flat struct — no state machine required for Phase 4.
pub struct App {
    pub state: CalcState,
    /// One-line status / error message shown in the TUI status bar. None = no message.
    pub message: Option<String>,
    /// Set to true to exit the event loop and return from run().
    pub exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            state: CalcState::new(),
            message: None,
            exit: false,
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
            // Phase 5: auto-save timer check goes here (PERS-02 — 30s auto-save)
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

        // Quit: 'q' or Ctrl+C
        if key.code == KeyCode::Char('q') {
            self.exit = true;
            return;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit = true;
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
