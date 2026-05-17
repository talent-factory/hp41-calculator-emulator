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
use hp41_core::{AngleMode, CalcState};

use crate::keys::{FlagPromptKind, RegisterOpKind};
use crate::{keys, persistence, ui};

/// Result of the shared IND-toggle detection (D-25.12 / RESEARCH Pitfall 10)
/// for FlagPrompt / RegisterPrompt arms. See `App::check_ind_toggle`.
enum IndToggleAction {
    /// `f` pressed inside a modal with `shift_armed == false` — arm the
    /// shift bit (already done by the helper) and re-store the modal
    /// unchanged. The next-key cycle is the actual toggle.
    ArmShift,
    /// `0` pressed with `shift_armed == true` — flip the modal's `ind`
    /// field. The helper already cleared `shift_armed` to false.
    ToggleInd,
    /// Standard key — fall through to digit / Esc / Backspace handling.
    Continue,
}

/// Discriminator for `PendingInput::XeqByName` — distinguishes the normal
/// XEQ-by-Name flow from the CollectForModal auto-open hook (D-29.8 / D-29.9).
///
/// `Normal`: the user explicitly opened XEQ-by-Name via `f-N` (keys.rs:319).
/// `CollectForModal`: auto-opened by `maybe_auto_open_collect_for_modal` after
///   a Math Pac I op opened a modal at FunctionNamePrompt. Enter dispatches
///   `submit_modal_with_label` instead of `Op::Xeq`.
///
/// Phase 29 / CLI-05 additive surface — D-29.8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XeqByNameMode {
    /// Standard XEQ-by-Name: Enter dispatches `Op::Xeq(acc)` or the local resolver.
    Normal,
    /// CollectForModal: Enter dispatches `submit_modal_with_label(state, &acc)`.
    CollectForModal,
}

/// Transient UI state for multi-key input (D-08). NOT serialized to disk.
/// Consumed in App::handle_pending_input(). Cleared on Esc or successful dispatch.
//
// Plan 02 grew this enum from 12 to 18 variants by adding the 6 Hybrid
// variants (FlagPrompt, RegisterPrompt, ClpLabel, DelCount, TonePrompt,
// XeqByName) per D-25.11. Task 1 landed the variants + exhaustive
// `pending_prompt()`; Task 2 wired the modal openers and dispatch.
// Plan 29-03 migrated XeqByName from a tuple variant to a struct variant
// with a `mode: XeqByNameMode` discriminator per D-29.8.
#[derive(Debug, Clone)]
pub enum PendingInput {
    /// Legacy v1.1 STO-register modal. Plan 02 routes `S` to
    /// `RegisterPrompt { Sto }` instead, so this variant is no longer
    /// constructed by the live keyboard handler — preserved for the
    /// already-shipped Plan 04 deprecation path and for tests that still
    /// exercise the v1.1 dispatch arm.
    #[allow(dead_code)]
    StoRegister(String), // accumulating 2-digit register number for STO [nn]
    /// Legacy v1.1 RCL-register modal. See `StoRegister` note above.
    #[allow(dead_code)]
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
    // ── Phase 25 Plan 02 — Hybrid PendingInput variants (D-25.11) ────────
    /// SF/CF/FS?/FC?/FS?C/FC?C × {direct, IND} — 12 logical flag ops collapsed
    /// into one group variant per D-25.11. The `kind` discriminator reuses
    /// `hp41_core::ops::FlagTestKind` via `FlagPromptKind::Test(_)` per D-25.13.
    /// `acc` is a 2-digit numeric accumulator. `ind` is toggled via the
    /// hardware-faithful shift-0 keystroke (RESEARCH Pitfall 10 / D-25.12),
    /// reusing `App.shift_armed` from Plan 01 — no separate `shift_pending`
    /// field (W2 fix). End-of-accumulation dispatch picks `Op::SfFlag(n)` vs
    /// `Op::SfFlagInd(n)` (and similarly for CF / FlagTest) at a single
    /// decision point per D-25.12.
    FlagPrompt {
        kind: FlagPromptKind,
        ind: bool,
        acc: String,
    },
    /// STO/RCL/STO+-×÷/VIEW/ARCL/ASTO/ISG/DSE × {direct, IND} — 22 logical
    /// register ops collapsed into one group variant per D-25.11. The `op`
    /// discriminator reuses `hp41_core::ops::StoArithKind` via
    /// `RegisterOpKind::StoArith(_)` per D-25.13. Same 2-digit + shift-0
    /// IND-toggle scaffold as `FlagPrompt`.
    RegisterPrompt {
        op: RegisterOpKind,
        ind: bool,
        acc: String,
    },
    /// CLP "name" — text-input modal for clearing a labelled program block.
    /// Accumulator capped at 7 chars (HP-41 LBL hardware limit per RESEARCH
    /// §Security V5). Enter dispatches `Op::Clp(acc)`; Esc cancels; Backspace
    /// pops the last char.
    ClpLabel(String),
    /// DEL nnn — 3-digit numeric accumulator for the program-step delete op.
    /// Final parse uses `.parse::<u8>().unwrap_or(u8::MAX)` so user input
    /// `999` silently clamps to 255 (T-25-06 mitigation).
    DelCount(String),
    /// TONE n — single-digit accumulator (0–9). First digit auto-dispatches
    /// `Op::Tone(n)`; non-digit cancels. Even simpler than the 2-digit
    /// scaffold.
    TonePrompt,
    /// XEQ "NAME" — text-input modal scaffold for the XEQ-by-Name flow.
    /// Plan 02 ships the scaffold; Plan 03 wires the resolver (`Op::Xeq`
    /// already falls back to `builtin_card_op` for the 4 card-reader names —
    /// the 8 conditional-test mnemonics land via Plan 03's
    /// `builtin_card_op` extension). Accumulator capped at 24 chars
    /// (HP-41 ALPHA register width per RESEARCH §Security V5).
    /// Plan 29-03 (D-29.8): migrated from tuple variant to struct variant
    /// with `mode: XeqByNameMode` discriminator for the CollectForModal flow.
    XeqByName { acc: String, mode: XeqByNameMode },
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
    // ── Phase 25: one-shot HP-41CV f-prefix arm state (D-25.1 / D-25.4) ──────
    /// True for exactly one key-press cycle after `f` is pressed; consumed
    /// by the next op key (which is then resolved via `keys::shifted_key_to_op`)
    /// OR cleared by Esc. Cleared UNCONDITIONALLY at the end of the consumed
    /// branch — see Pitfall 5 in RESEARCH.md. Frontend-only, never persisted
    /// in `CalcState`, never crosses IPC — mirrors hp41-gui v2.1's `shiftActive`
    /// per D-25.6 (CLI ↔ GUI parity invariant).
    pub shift_armed: bool,
    // ── Phase 5: overlays (D-16, D-22) ───────────────────────────────────────
    pub show_help: bool,
    /// RefCell: draw(&self) is immutable but render_stateful_widget needs &mut TableState.
    /// Single-threaded, non-reentrant draw — borrow_mut() will never panic at runtime.
    pub help_table_state: RefCell<TableState>,
    pub show_programs: bool,
    pub programs_table_state: RefCell<TableState>,
    /// BufWriter for --print-log, if specified. None = no file logging.
    pub print_log_writer: Option<std::io::BufWriter<std::fs::File>>,
    /// `~/.hp41/cards/`. `None` when `dirs::home_dir()` is unavailable (rare;
    /// CI / containers with no $HOME) — in that case `drain_pending_card_op`
    /// surfaces a "cannot resolve" diagnostic to `self.message` and clears
    /// `pending_card_op` so the modal does not get stuck. We deliberately
    /// do NOT fall back to a relative `.hp41/cards` path: a relative path
    /// would scatter cards across whatever directory the user happened to
    /// launch the binary from, and would NOT be readable by the GUI that
    /// expects `~/.hp41/cards/` — breaking the documented "shared dir"
    /// invariant in CLAUDE.md.
    cards_dir: Option<std::path::PathBuf>,
}

impl App {
    pub fn new(
        state: CalcState,
        state_path: PathBuf,
        print_log: Option<std::path::PathBuf>,
    ) -> Self {
        let (print_log_writer, mut initial_message) = match print_log {
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
        let cards_dir = crate::cards::cards_dir();
        if cards_dir.is_none() {
            // Non-fatal: card ops will report the same diagnostic when invoked,
            // but a startup warning saves the user a confused round-trip.
            // Combine with any pre-existing warning (e.g. failed --print-log
            // open) instead of suppressing either — both are independently
            // actionable and `initial_message` has only one slot.
            let card_warn = "Warning: cannot resolve ~/.hp41/cards (no $HOME) — card ops disabled";
            initial_message = Some(match initial_message {
                Some(existing) => format!("{existing}; {card_warn}"),
                None => card_warn.to_string(),
            });
        }
        App {
            state,
            message: initial_message,
            exit: false,
            last_save: Instant::now(),
            state_path,
            pending_input: None,
            shift_armed: false,
            show_help: false,
            help_table_state: RefCell::new(TableState::default()),
            show_programs: false,
            programs_table_state: RefCell::new(TableState::default()),
            print_log_writer,
            cards_dir,
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
    ///
    /// `pub` (Phase 25) so integration tests under `tests/` can drive the full
    /// f-prefix state machine end-to-end without resorting to `#[cfg(test)]`
    /// in-crate hacks. The CLI binary calls it from the `run()` event loop.
    pub fn handle_key(&mut self, key: KeyEvent) {
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

        // D-16: '?' toggles the help overlay.
        //
        // Phase 25 / Plan 03 (Rule 1 — fixes blocker for FN-TEST-01): when a
        // text-input modal (`XeqByName` or `ClpLabel`) is active, the user
        // needs to type `?` as the trailing character of HP-41CV mnemonics
        // like `X<>Y?`. Let the `?` flow through to the modal handler in
        // that case. For non-text-input modals (Sto/Rcl reg modals, Flag
        // prompts, etc.) `?` retains its help-toggle behavior because those
        // modals reject non-digit characters anyway.
        if key.code == KeyCode::Char('?')
            && !matches!(
                self.pending_input,
                Some(PendingInput::XeqByName { .. }) | Some(PendingInput::ClpLabel(_))
            )
        {
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
        // Plan 02 (D-25.11): S/R open the new `RegisterPrompt` hybrid variants
        // (op=Sto/Rcl, ind:false). The new arm preserves the legacy v1.1
        // STO-arithmetic chain (`S → +/-/×/÷ → register`) by intercepting the
        // arithmetic keys when `op == Sto/Rcl && acc.is_empty()`, falling
        // back to the existing `PendingInput::StoAdd/Sub/Mul/Div` modals.
        // M/N/O hidden-register dispatch is preserved the same way.
        if key.code == KeyCode::Char('S') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::RegisterPrompt {
                op: RegisterOpKind::Sto,
                ind: false,
                acc: String::new(),
            });
            self.message = None;
            return;
        }
        if key.code == KeyCode::Char('R') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            self.pending_input = Some(PendingInput::RegisterPrompt {
                op: RegisterOpKind::Rcl,
                ind: false,
                acc: String::new(),
            });
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

        // Card Reader comfort shortcuts — Ctrl+W/R/D/F dispatch the four card ops
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

        // ── Phase 25: HP-41CV one-shot f-prefix state machine (D-25.1 / D-25.4) ──
        // Ordering rules (non-negotiable):
        //   • AFTER the pending_input route — an active modal MUST swallow `f`
        //     (Pitfall 4 — INTENDED). The route at line ~228 already returned.
        //   • AFTER the ALPHA-mode block — `f` in ALPHA mode types the letter F
        //     (D-25.5 / Pitfall 2). The block above already returned.
        //   • BEFORE the v1.x `f` FmtDigits cycle which is REMOVED in this commit
        //     (D-25.3 / Pitfall 3) — no other `f` handler may stand between this
        //     point and the end of handle_key.
        //
        // Arm on plain `f` (no Ctrl modifier; Ctrl+F is RDTA, handled above):
        if !self.shift_armed
            && key.code == KeyCode::Char('f')
            && !key.modifiers.contains(KeyModifiers::CONTROL)
        {
            self.shift_armed = true;
            self.message = None;
            return;
        }
        // Consume on the next key cycle (one-shot lifetime, D-25.4):
        if self.shift_armed {
            // Esc cancels without dispatching.
            if key.code == KeyCode::Esc {
                self.shift_armed = false;
                return;
            }
            // Try the shifted resolver; on miss the prefix is consumed silently.
            if let Some(op) = keys::shifted_key_to_op(key, self) {
                self.call_dispatch(op);
            }
            // ALWAYS clear after consumption — Pitfall 5: the one-shot lifetime
            // is "next key cycle", NOT "next CONSUMED key". Bleed across handle_key
            // invocations is what `test_shift_armed_pitfall5_bleed` guards against.
            self.shift_armed = false;
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
                            // Clear stale message, drain card op (capturing any error
                            // so the print drain cannot overwrite it), then drain
                            // print output with the card error threaded in.
                            self.message = None;
                            let card_err = self.drain_pending_card_op();
                            self.drain_and_show_print_output(card_err);
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

        // Phase 25 (D-25.3 / Pitfall 3): the v1.x `f` direct-cycle binding
        // (FIX/SCI/ENG cycle) was REMOVED here. The HP-41CV `f` key is now the
        // ONE yellow prefix shift (D-25.2), armed above. The FIX/SCI/ENG modal
        // remains reachable via `F` (uppercase) which opens `PendingInput::FmtDigits`;
        // Plan 02 / Plan 04 reposition that modal to its real HP-41CV f-shifted
        // keyboard position once the JSON-derived key table lands (D-25.18).

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

        // Phase 29 (D-29.5): R/S submits modal numeric input.
        // MUST be below the pending_input.is_some() return at line ~327 (D-07 invariant)
        // and below the shift_armed block (§7.2). The pending_input.is_none() defensive
        // guard prevents future code-shape changes from breaking D-07.
        if key.code == KeyCode::F(5)
            && self.state.modal_program.is_some()
            && self.pending_input.is_none()
        {
            match hp41_core::ops::math1::submit_modal(&mut self.state) {
                Ok(()) => {
                    self.message = None;
                    self.drain_and_show_print_output(None);
                }
                Err(e) => self.message = Some(format!("{e}")),
            }
            return;
        }

        // Phase 29 (D-29.6): Esc cancels open math1 modal (no pending_input active).
        // MUST be below the shift_armed Esc block (§7.2 two-step convention): the
        // shift_armed block at ~line 438-451 fires first when shift_armed=true, clears
        // shift_armed, and returns. Only when shift_armed=false does this block fire.
        // The defensive && self.pending_input.is_none() guard preserves D-07 even if
        // future refactors change the ordering above.
        if key.code == KeyCode::Esc
            && self.state.modal_program.is_some()
            && self.pending_input.is_none()
        {
            hp41_core::ops::math1::cancel_modal(&mut self.state);
            self.message = Some("Cancelled".to_string());
            return;
        }

        // D-16: R/S — F5, hardcoded run_program("A") in Phase 4
        // F5 is handled here directly, not routed through key_to_op().
        if key.code == KeyCode::F(5) {
            match hp41_core::run_program(&mut self.state, "A") {
                Ok(()) => {
                    self.message = None;
                    let card_err = self.drain_pending_card_op();
                    self.drain_and_show_print_output(card_err);
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
            // ── Phase 25 Plan 02 — Hybrid PendingInput arms ──────────────
            // Task 1 lands the variants and the exhaustive pending_prompt()
            // arm. Task 2 wires the real accumulator + IND-toggle + dispatch
            // logic. To keep this compiling between commits, the 6 stubs
            // below close the modal on any key — Task 2 replaces them
            // wholesale with the correct scaffold.
            Some(PendingInput::FlagPrompt { kind, ind, acc }) => {
                self.handle_flag_prompt(key, kind, ind, acc);
            }
            Some(PendingInput::RegisterPrompt { op, ind, acc }) => {
                self.handle_register_prompt(key, op, ind, acc);
            }
            Some(PendingInput::ClpLabel(acc)) => {
                self.handle_clp_label(key, acc);
            }
            Some(PendingInput::DelCount(acc)) => {
                self.handle_del_count(key, acc);
            }
            Some(PendingInput::TonePrompt) => {
                self.handle_tone_prompt(key);
            }
            Some(PendingInput::XeqByName { acc, mode }) => {
                self.handle_xeq_by_name(key, acc, mode);
            }
            None => unreachable!("handle_pending_input called with no pending input — caller must check is_some() first"),
        }
    }

    // ── Phase 25 Plan 02 — Hybrid PendingInput arm bodies (D-25.11/12) ───
    //
    // Each handler implements one of the 6 new PendingInput variants:
    //   • FlagPrompt / RegisterPrompt: 2-digit numeric accumulator with the
    //     hardware-faithful shift-0 IND-toggle per D-25.12 + Pitfall 10.
    //     The IND-toggle REUSES App.shift_armed (W2 fix — no parallel
    //     `shift_pending` field). Final dispatch picks Op::*Ind(n) vs
    //     Op::*(n) at a single decision point per D-25.12.
    //   • ClpLabel: text-input modal, cap at 7 chars (HP-41 LBL hardware
    //     limit, T-25-05 mitigation), Enter → Op::Clp.
    //   • DelCount: 3-digit accumulator, silent-clamp to u8::MAX
    //     (T-25-06 mitigation), Enter or 3rd digit → Op::Del.
    //   • TonePrompt: single-digit (0–9) auto-dispatch → Op::Tone.
    //   • XeqByName: text-input modal, cap at 24 chars (T-25-05 mitigation),
    //     Enter → Op::Xeq(acc). Plan 03 extends the resolver chain to handle
    //     the 8 conditional-test mnemonics via builtin_card_op 4→12.

    /// Shared IND-toggle detection for FlagPrompt / RegisterPrompt.
    ///
    /// Returns:
    ///   • `IndToggleAction::ArmShift` when the key is plain `f` (no Ctrl)
    ///     AND `self.shift_armed` is currently false → caller stores the
    ///     pending state unchanged and the caller MUST set `self.shift_armed
    ///     = true` (we do it inline to centralise the rule).
    ///   • `IndToggleAction::ToggleInd` when `self.shift_armed` is true AND
    ///     the key is `0` → caller flips `ind` and the helper clears
    ///     `self.shift_armed`.
    ///   • `IndToggleAction::Continue` otherwise; caller falls through to
    ///     the standard accumulator/Esc/Backspace logic.
    ///
    /// This is the SINGLE place that mutates `self.shift_armed` from inside
    /// a modal — same one-shot bit Plan 01 introduced, no parallel field.
    fn check_ind_toggle(&mut self, key: KeyEvent) -> IndToggleAction {
        if !self.shift_armed
            && key.code == KeyCode::Char('f')
            && !key.modifiers.contains(KeyModifiers::CONTROL)
        {
            self.shift_armed = true;
            return IndToggleAction::ArmShift;
        }
        if self.shift_armed && key.code == KeyCode::Char('0') {
            self.shift_armed = false;
            return IndToggleAction::ToggleInd;
        }
        IndToggleAction::Continue
    }

    fn handle_flag_prompt(&mut self, key: KeyEvent, kind: FlagPromptKind, ind: bool, acc: String) {
        match self.check_ind_toggle(key) {
            IndToggleAction::ArmShift => {
                // Re-store the modal unchanged; the next-key cycle is the toggle.
                self.pending_input = Some(PendingInput::FlagPrompt { kind, ind, acc });
                return;
            }
            IndToggleAction::ToggleInd => {
                self.pending_input = Some(PendingInput::FlagPrompt {
                    kind,
                    ind: !ind,
                    acc,
                });
                return;
            }
            IndToggleAction::Continue => {}
        }

        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let mut new_acc = acc;
                new_acc.push(c);
                if new_acc.len() == 2 {
                    // "two ASCII digits always parse as u8 ≤ 99" — same invariant
                    // as the legacy handle_reg_modal helper.
                    let n: u8 = new_acc
                        .parse()
                        .expect("two ASCII digit chars always parse as u8 ≤ 99");
                    let op = match (kind, ind) {
                        (FlagPromptKind::SetFlag, false) => Op::SfFlag(n),
                        (FlagPromptKind::SetFlag, true) => Op::SfFlagInd(n),
                        (FlagPromptKind::ClearFlag, false) => Op::CfFlag(n),
                        (FlagPromptKind::ClearFlag, true) => Op::CfFlagInd(n),
                        (FlagPromptKind::Test(k), false) => Op::FlagTest { kind: k, flag: n },
                        (FlagPromptKind::Test(k), true) => Op::FlagTestInd {
                            kind: k,
                            ind_reg: n,
                        },
                    };
                    self.call_dispatch(op);
                    self.pending_input = None;
                } else {
                    self.pending_input = Some(PendingInput::FlagPrompt {
                        kind,
                        ind,
                        acc: new_acc,
                    });
                }
            }
            KeyCode::Backspace => {
                self.pending_input = Some(PendingInput::FlagPrompt {
                    kind,
                    ind,
                    acc: String::new(),
                });
            }
            KeyCode::Esc => {
                // T-25-07 mitigation: also clear shift_armed on Esc so a
                // half-armed prefix does not leak past the cancelled modal.
                self.shift_armed = false;
                self.pending_input = None;
            }
            _ => {
                // Silently restore the modal for unrecognised keys.
                self.pending_input = Some(PendingInput::FlagPrompt { kind, ind, acc });
            }
        }
    }

    fn handle_register_prompt(
        &mut self,
        key: KeyEvent,
        op: RegisterOpKind,
        ind: bool,
        acc: String,
    ) {
        match self.check_ind_toggle(key) {
            IndToggleAction::ArmShift => {
                self.pending_input = Some(PendingInput::RegisterPrompt { op, ind, acc });
                return;
            }
            IndToggleAction::ToggleInd => {
                self.pending_input = Some(PendingInput::RegisterPrompt { op, ind: !ind, acc });
                return;
            }
            IndToggleAction::Continue => {}
        }

        // v1.1 STO-arithmetic chain and M/N/O dispatch — preserved per Plan 02
        // must_have truth #8 ("STO-arith stays on legacy S→op→reg chain"). Only
        // fires for direct STO/RCL with an empty accumulator; once digits have
        // landed or IND is armed, the user is in the numeric phase and the
        // chain shortcuts do not apply.
        if matches!(op, RegisterOpKind::Sto | RegisterOpKind::Rcl) && !ind && acc.is_empty() {
            match key.code {
                // STO+/-/×/÷ chain transition (only valid in op == Sto case).
                KeyCode::Char('+') if matches!(op, RegisterOpKind::Sto) => {
                    self.pending_input = Some(PendingInput::StoAdd(String::new()));
                    return;
                }
                KeyCode::Char('-') if matches!(op, RegisterOpKind::Sto) => {
                    self.pending_input = Some(PendingInput::StoSub(String::new()));
                    return;
                }
                KeyCode::Char('*') if matches!(op, RegisterOpKind::Sto) => {
                    self.pending_input = Some(PendingInput::StoMul(String::new()));
                    return;
                }
                KeyCode::Char('/') if matches!(op, RegisterOpKind::Sto) => {
                    self.pending_input = Some(PendingInput::StoDiv(String::new()));
                    return;
                }
                // M/N/O hidden registers — dispatch immediately (Phase 12 D-08).
                KeyCode::Char('M') | KeyCode::Char('m') => {
                    let mno_op = match op {
                        RegisterOpKind::Sto => Op::StoM,
                        RegisterOpKind::Rcl => Op::RclM,
                        _ => unreachable!("guarded by Sto|Rcl match above"),
                    };
                    self.call_dispatch(mno_op);
                    self.pending_input = None;
                    return;
                }
                KeyCode::Char('N') | KeyCode::Char('n') => {
                    let mno_op = match op {
                        RegisterOpKind::Sto => Op::StoN,
                        RegisterOpKind::Rcl => Op::RclN,
                        _ => unreachable!("guarded by Sto|Rcl match above"),
                    };
                    self.call_dispatch(mno_op);
                    self.pending_input = None;
                    return;
                }
                KeyCode::Char('O') | KeyCode::Char('o') => {
                    let mno_op = match op {
                        RegisterOpKind::Sto => Op::StoO,
                        RegisterOpKind::Rcl => Op::RclO,
                        _ => unreachable!("guarded by Sto|Rcl match above"),
                    };
                    self.call_dispatch(mno_op);
                    self.pending_input = None;
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let mut new_acc = acc;
                new_acc.push(c);
                if new_acc.len() == 2 {
                    let n: u8 = new_acc
                        .parse()
                        .expect("two ASCII digit chars always parse as u8 ≤ 99");
                    let final_op = match (op, ind) {
                        (RegisterOpKind::Sto, false) => Op::StoReg(n),
                        (RegisterOpKind::Sto, true) => Op::StoInd(n),
                        (RegisterOpKind::Rcl, false) => Op::RclReg(n),
                        (RegisterOpKind::Rcl, true) => Op::RclInd(n),
                        (RegisterOpKind::StoArith(k), false) => Op::StoArith { reg: n, kind: k },
                        (RegisterOpKind::StoArith(k), true) => Op::StoArithInd(n, k),
                        (RegisterOpKind::View, false) => Op::View(n),
                        (RegisterOpKind::View, true) => Op::ViewInd(n),
                        (RegisterOpKind::Arcl, false) => Op::Arcl(n),
                        (RegisterOpKind::Arcl, true) => Op::ArclInd(n),
                        (RegisterOpKind::Asto, false) => Op::Asto(n),
                        (RegisterOpKind::Asto, true) => Op::AstoInd(n),
                        (RegisterOpKind::Isg, false) => Op::Isg(n),
                        (RegisterOpKind::Isg, true) => Op::IsgInd(n),
                        (RegisterOpKind::Dse, false) => Op::Dse(n),
                        (RegisterOpKind::Dse, true) => Op::DseInd(n),
                    };
                    self.call_dispatch(final_op);
                    self.pending_input = None;
                } else {
                    self.pending_input = Some(PendingInput::RegisterPrompt {
                        op,
                        ind,
                        acc: new_acc,
                    });
                }
            }
            KeyCode::Backspace => {
                self.pending_input = Some(PendingInput::RegisterPrompt {
                    op,
                    ind,
                    acc: String::new(),
                });
            }
            KeyCode::Esc => {
                self.shift_armed = false;
                self.pending_input = None;
            }
            _ => {
                self.pending_input = Some(PendingInput::RegisterPrompt { op, ind, acc });
            }
        }
    }

    /// HP-41 LBL hardware limit — labels are at most 7 characters
    /// (single ALPHA register row holds 6 packed chars + 1 sentinel).
    const CLP_LABEL_CAP: usize = 7;

    fn handle_clp_label(&mut self, key: KeyEvent, acc: String) {
        match key.code {
            KeyCode::Esc => {
                self.pending_input = None;
            }
            KeyCode::Enter => {
                if !acc.is_empty() {
                    self.call_dispatch(Op::Clp(acc));
                }
                self.pending_input = None;
            }
            KeyCode::Backspace => {
                let mut new_acc = acc;
                new_acc.pop();
                self.pending_input = Some(PendingInput::ClpLabel(new_acc));
            }
            KeyCode::Char(ch) => {
                let mut new_acc = acc;
                if new_acc.len() < Self::CLP_LABEL_CAP {
                    new_acc.push(ch);
                }
                self.pending_input = Some(PendingInput::ClpLabel(new_acc));
            }
            _ => {
                self.pending_input = Some(PendingInput::ClpLabel(acc));
            }
        }
    }

    fn handle_del_count(&mut self, key: KeyEvent, acc: String) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let mut new_acc = acc;
                new_acc.push(c);
                if new_acc.len() == 3 {
                    // T-25-06 mitigation: silent-clamp on overflow. 999 > u8::MAX
                    // (255) — `.parse::<u8>().unwrap_or(u8::MAX)` is the documented
                    // pattern.
                    let n: u8 = new_acc.parse::<u8>().unwrap_or(u8::MAX);
                    self.call_dispatch(Op::Del(n));
                    self.pending_input = None;
                } else {
                    self.pending_input = Some(PendingInput::DelCount(new_acc));
                }
            }
            KeyCode::Backspace => {
                self.pending_input = Some(PendingInput::DelCount(String::new()));
            }
            KeyCode::Esc => {
                self.pending_input = None;
            }
            _ => {
                self.pending_input = Some(PendingInput::DelCount(acc));
            }
        }
    }

    fn handle_tone_prompt(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let n: u8 = c as u8 - b'0';
                self.call_dispatch(Op::Tone(n));
                self.pending_input = None;
            }
            KeyCode::Esc => {
                self.pending_input = None;
            }
            _ => {
                self.pending_input = Some(PendingInput::TonePrompt);
            }
        }
    }

    /// HP-41 ALPHA register width — XEQ-by-Name names cap at 24 chars
    /// (T-25-05 mitigation; matches the ALPHA register size).
    const XEQ_NAME_CAP: usize = 24;

    fn handle_xeq_by_name(&mut self, key: KeyEvent, acc: String, mode: XeqByNameMode) {
        match key.code {
            KeyCode::Esc => {
                self.pending_input = None;
            }
            KeyCode::Enter => {
                if !acc.is_empty() {
                    match mode {
                        XeqByNameMode::CollectForModal => {
                            // Phase 29 (D-29.8): CollectForModal Enter → submit_modal_with_label
                            match hp41_core::ops::math1::submit_modal_with_label(
                                &mut self.state,
                                &acc,
                            ) {
                                Ok(()) => self.message = None,
                                Err(e) => self.message = Some(format!("{e}")),
                            }
                        }
                        XeqByNameMode::Normal => {
                            // Plan 03 (D-25.8 + D-25.9): CLI-local fast-path first —
                            // resolves the 8 non-keyboard conditional-test mnemonics
                            // directly to Op::Test(TestKind::*) without round-tripping
                            // through Op::Xeq + run_program. Falls through to
                            // Op::Xeq(acc) for the 4 v2.1 card-reader names
                            // (WPRGM/RDPRGM/WDTA/RDTA — resolved by hp41-core::
                            // builtin_card_op) and for user LBLs. Unknown names
                            // surface as HpError::InvalidOp via Op::Xeq (Pitfall 9 —
                            // no "did you mean…?" hint until Phase 26).
                            if let Some(op) =
                                keys::xeq_by_name_local_resolve(&acc, self.state.xrom_modules)
                            {
                                self.call_dispatch(op);
                            } else {
                                self.call_dispatch(Op::Xeq(acc));
                            }
                        }
                    }
                }
                self.pending_input = None;
            }
            KeyCode::Backspace => {
                let mut new_acc = acc;
                new_acc.pop();
                self.pending_input = Some(PendingInput::XeqByName { acc: new_acc, mode });
            }
            KeyCode::Char(ch) => {
                let mut new_acc = acc;
                if new_acc.len() < Self::XEQ_NAME_CAP {
                    new_acc.push(ch);
                }
                self.pending_input = Some(PendingInput::XeqByName { acc: new_acc, mode });
            }
            _ => {
                self.pending_input = Some(PendingInput::XeqByName { acc, mode });
            }
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
                        self.message = None;
                        let card_err = self.drain_pending_card_op();
                        self.drain_and_show_print_output(card_err);
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
    /// Returns `Some(msg)` on failure (caller is responsible for routing it
    /// into `self.message` while preserving any subsequent print-output
    /// summary). `None` on success or no-op. Also returns `Some(msg)` —
    /// without touching state — when `cards_dir` is unresolved.
    ///
    /// Returning rather than writing `self.message` directly is what fixes
    /// the message-clobber bug: a CARD DATA error inside a program that
    /// also prints (PRX) would otherwise be replaced by the print summary.
    fn drain_pending_card_op(&mut self) -> Option<String> {
        self.state.pending_card_op.as_ref()?;
        let Some(dir) = self.cards_dir.as_ref() else {
            // Clear the staged request — leaving it pending would lock the
            // user out of every subsequent card op via `ensure_no_pending`.
            self.state.pending_card_op = None;
            return Some("card op failed: cannot resolve ~/.hp41/cards (no $HOME)".to_string());
        };
        match crate::cards::drain_pending_card_op(&mut self.state, dir) {
            Ok(()) => None,
            Err(e) => Some(format!("{e}")),
        }
    }

    /// Drain print_buffer after a run_program() Ok(()) return and surface output in the TUI.
    ///
    /// Mirrors the drain branch inside call_dispatch_and_drain but decoupled from dispatch —
    /// called after run_program() has already returned successfully.
    ///
    /// `card_error` is the result of `drain_pending_card_op` for the same
    /// call site, threaded in so the card diagnostic is not clobbered by the
    /// print summary. When both are present they are combined into a single
    /// status line so the user sees both signals at once.
    ///
    /// For 1 line (PRX/PRA): sets app.message to the formatted line.
    /// For N > 1 lines (PRSTK or multiple print ops in one program): sets app.message to
    ///   "PRSTK → N lines" summary consistent with D-01.
    /// If print_log_writer is Some, writes each line to the file via
    /// `write_lines_to_print_log()` which disables the writer on first I/O error.
    /// Clears print_buffer via drain(..).
    fn drain_and_show_print_output(&mut self, card_error: Option<String>) {
        let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
        if !lines.is_empty() {
            let log_failure = self.write_lines_to_print_log(&lines);
            let summary = if lines.len() > 1 {
                format!("PRSTK \u{2192} {} lines", lines.len())
            } else {
                lines.into_iter().next().unwrap_or_default()
            };
            let print_msg = match log_failure {
                Some(err) => format!("{summary} ({err})"),
                None => summary,
            };
            self.message = Some(match card_error {
                Some(err) => format!("{err}; {print_msg}"),
                None => print_msg,
            });
        } else if let Some(err) = card_error {
            // No print output to summarise — surface the card error directly.
            self.message = Some(err);
        }
        // Empty lines AND no card error: leave self.message as the caller set it.
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
    pub fn call_dispatch(&mut self, op: Op) {
        match hp41_core::ops::dispatch(&mut self.state, op) {
            Ok(()) => self.message = None,
            Err(e) => self.message = Some(format!("{e}")),
        }
        // Phase 29 (D-29.9): post-dispatch auto-open CollectForModal modal
        // when modal needs alpha label.
        self.maybe_auto_open_collect_for_modal();
    }

    /// Call hp41_core::ops::dispatch, then drain card op and print_buffer.
    /// For PRX/PRA (1 line): sets app.message to the formatted line (per D-01).
    /// For PRSTK (6 lines): sets app.message to "PRSTK → N lines" summary (per D-01).
    /// If print_log_writer is Some, writes each line to the file (best-effort, never panics).
    pub(crate) fn call_dispatch_and_drain(&mut self, op: Op) {
        match hp41_core::ops::dispatch(&mut self.state, op) {
            Ok(()) => {
                let card_err = self.drain_pending_card_op();
                let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
                if !lines.is_empty() {
                    let log_failure = self.write_lines_to_print_log(&lines);
                    let summary = if lines.len() > 1 {
                        format!("PRSTK \u{2192} {} lines", lines.len())
                    } else {
                        // lines.len() == 1; into_iter().next() is safe here
                        lines.into_iter().next().unwrap_or_default()
                    };
                    let print_msg = match log_failure {
                        Some(err) => format!("{summary} ({err})"),
                        None => summary,
                    };
                    self.message = Some(match card_err {
                        Some(err) => format!("{err}; {print_msg}"),
                        None => print_msg,
                    });
                } else if let Some(err) = card_err {
                    // Pure card op (no print output) — surface the diagnostic.
                    self.message = Some(err);
                }
                // No card error and no print output: leave self.message alone;
                // call_dispatch (used by non-print ops) is the canonical place
                // that clears stale messages on Ok.
            }
            Err(e) => self.message = Some(format!("{e}")),
        }
        // Phase 29 (D-29.9): post-dispatch auto-open CollectForModal modal
        // when modal needs alpha label.
        self.maybe_auto_open_collect_for_modal();
    }

    /// Phase 29 (D-29.9): post-dispatch auto-open CollectForModal modal.
    ///
    /// Called at the tail of `call_dispatch` and `call_dispatch_and_drain` after
    /// every hp41-core dispatch. If the current `modal_program` requires an alpha
    /// label (i.e., it's at a FunctionNamePrompt step for Integ, Solve, or Difeq),
    /// auto-opens `PendingInput::XeqByName { mode: CollectForModal }` so the CLI
    /// immediately prompts the user to type the function label name.
    ///
    /// Depth bound: `requires_alpha_label()` returns false for ALL steps except
    /// the three FunctionNamePrompt variants (per D-29.9 / RESEARCH §7.8). After
    /// `submit_modal_with_label` advances the step, the next dispatch will NOT
    /// re-trigger this hook. Bound = 1.
    fn maybe_auto_open_collect_for_modal(&mut self) {
        if self.pending_input.is_some() {
            return;
        }
        let Some(ref mp) = self.state.modal_program else {
            return;
        };
        if mp.requires_alpha_label() {
            self.pending_input = Some(PendingInput::XeqByName {
                acc: String::new(),
                mode: XeqByNameMode::CollectForModal,
            });
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
        // Phase 25 Plan 02: `S` now opens `RegisterPrompt { Sto }` instead
        // of the legacy `StoRegister` modal. The M/N/O hidden-register
        // dispatch is preserved by `handle_register_prompt` (the new arm
        // intercepts M/N/O when `op == Sto/Rcl && acc.is_empty()`).
        use crate::keys::RegisterOpKind;
        let mut app = make_app();
        app.handle_key(press(KeyCode::Char('S')));
        assert!(
            matches!(
                app.pending_input,
                Some(PendingInput::RegisterPrompt {
                    op: RegisterOpKind::Sto,
                    ind: false,
                    ..
                })
            ),
            "Pressing 'S' must open RegisterPrompt {{ Sto }} (Plan 02 truth #7)"
        );
        app.state.stack.x = hp41_core::HpNum::from(42i32);
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
        // Phase 25 Plan 02: `R` now opens `RegisterPrompt { Rcl }` — M/N/O
        // shortcut preserved by the new arm.
        use crate::keys::RegisterOpKind;
        let mut app = make_app();
        app.state.reg_m = hp41_core::HpNum::from(99i32);
        app.handle_key(press(KeyCode::Char('R')));
        assert!(
            matches!(
                app.pending_input,
                Some(PendingInput::RegisterPrompt {
                    op: RegisterOpKind::Rcl,
                    ind: false,
                    ..
                })
            ),
            "Pressing 'R' must open RegisterPrompt {{ Rcl }} (Plan 02 truth #7)"
        );
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

    /// Ctrl+W dispatches WPRGM (write program to card).
    /// Sandboxed: injects a tempdir as cards_dir so no real ~/.hp41/cards/ is touched.
    /// Proves the correct op was dispatched: WPRGM writes TESTCARD.raw; a W↔R or
    /// W↔D mapping swap would produce a different file extension (or an error message).
    #[test]
    fn test_ctrl_w_dispatches_wprgm() {
        let tmp = tempfile::tempdir().unwrap();
        let mut app = make_app();
        app.cards_dir = Some(tmp.path().to_path_buf());
        app.state.alpha_reg = "TESTCARD".to_string();
        // Give the program something to encode (avoid empty-program edge cases).
        app.state.program = vec![Op::Lbl("X".to_string()), Op::Rtn];

        app.handle_key(make_ctrl_key('w'));

        assert!(!app.exit, "Ctrl+W must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+W must not activate ALPHA mode");
        let raw_path = tmp.path().join("TESTCARD.raw");
        assert!(
            raw_path.exists(),
            "Ctrl+W must write TESTCARD.raw via WPRGM"
        );
    }

    /// Ctrl+R dispatches RDPRGM (read program from card).
    /// Sandboxed: injects a tempdir as cards_dir (no MISSING.raw present).
    /// Proves the correct op was dispatched: RDPRGM on a missing file surfaces
    /// "card data" in app.message; a R↔W swap would write a file instead of erroring.
    #[test]
    fn test_ctrl_r_dispatches_rdprgm() {
        let tmp = tempfile::tempdir().unwrap();
        let mut app = make_app();
        app.cards_dir = Some(tmp.path().to_path_buf());
        app.state.alpha_reg = "MISSING".to_string();

        app.handle_key(make_ctrl_key('r'));

        assert!(!app.exit, "Ctrl+R must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+R must not activate ALPHA mode");
        // RDPRGM on a missing file → HpError::CardData → app.message contains "card data".
        let msg = app.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("card data") || msg.contains("CARD DATA"),
            "Ctrl+R on missing file must surface CARD DATA via app.message; got {msg:?}",
        );
    }

    /// Ctrl+D dispatches WDTA (write data registers to card).
    /// Sandboxed: injects a tempdir as cards_dir so no real ~/.hp41/cards/ is touched.
    /// Proves the correct op was dispatched: WDTA writes TESTCARD.card.json; a D↔F or
    /// D↔W mapping swap would produce a different file extension (or an error message).
    #[test]
    fn test_ctrl_d_dispatches_wdta() {
        let tmp = tempfile::tempdir().unwrap();
        let mut app = make_app();
        app.cards_dir = Some(tmp.path().to_path_buf());
        app.state.alpha_reg = "TESTCARD".to_string();

        app.handle_key(make_ctrl_key('d'));

        assert!(!app.exit, "Ctrl+D must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+D must not activate ALPHA mode");
        let json_path = tmp.path().join("TESTCARD.card.json");
        assert!(
            json_path.exists(),
            "Ctrl+D must write TESTCARD.card.json via WDTA"
        );
    }

    /// Ctrl+F dispatches RDTA (read data registers from card).
    /// Sandboxed: injects a tempdir as cards_dir (no MISSING.card.json present).
    /// Proves the correct op was dispatched: RDTA on a missing file surfaces
    /// "card data" in app.message; an F↔D swap would write a file instead of erroring.
    #[test]
    fn test_ctrl_f_dispatches_rdta() {
        let tmp = tempfile::tempdir().unwrap();
        let mut app = make_app();
        app.cards_dir = Some(tmp.path().to_path_buf());
        app.state.alpha_reg = "MISSING".to_string();

        app.handle_key(make_ctrl_key('f'));

        assert!(!app.exit, "Ctrl+F must not quit the app");
        assert!(!app.state.alpha_mode, "Ctrl+F must not activate ALPHA mode");
        // RDTA on a missing file → HpError::CardData → app.message contains "card data".
        let msg = app.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("card data") || msg.contains("CARD DATA"),
            "Ctrl+F on missing file must surface CARD DATA via app.message; got {msg:?}",
        );
    }

    /// Card error must NOT be clobbered by a print-output summary fired in
    /// the same dispatch tick. Before the I3 fix, the print drain wrote
    /// straight into `self.message` and overwrote whatever the card drain
    /// had recorded — so a user running a program that mixed `PRX` with a
    /// failed card op only saw the print line and never knew the card op
    /// failed.
    #[test]
    fn card_error_combined_with_print_output() {
        use hp41_core::cardreader::CardOpRequest;
        let tmp = tempfile::tempdir().unwrap();
        let mut app = make_app();
        app.cards_dir = Some(tmp.path().to_path_buf());
        // Stage a card op whose name will fail sanitize so the drain returns
        // a CardData diagnostic without needing a real fs failure.
        app.state.pending_card_op = Some(CardOpRequest::WriteProgram {
            name: "BAD/SEP".to_string(),
        });
        // Stage a print line so the print drain's summary path also fires.
        app.state.print_buffer.push("PRX line".to_string());

        let card_err = app.drain_pending_card_op();
        assert!(card_err.is_some(), "bad name must yield a card error");
        app.drain_and_show_print_output(card_err);

        let msg = app.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("path separator"),
            "card diagnostic must survive print drain; got {msg:?}",
        );
        assert!(
            msg.contains("PRX line"),
            "print line must still appear; got {msg:?}",
        );
    }

    /// $HOME unavailable: card op must surface a clear diagnostic AND clear
    /// `pending_card_op` so the user is not locked out of subsequent ops
    /// via `ensure_no_pending`.
    #[test]
    fn card_op_with_no_cards_dir_surfaces_diagnostic() {
        use hp41_core::cardreader::CardOpRequest;
        let mut app = make_app();
        app.cards_dir = None; // simulate no $HOME
        app.state.pending_card_op = Some(CardOpRequest::WriteProgram {
            name: "OK".to_string(),
        });

        let err = app.drain_pending_card_op();
        let msg = err.expect("missing cards_dir must produce an error");
        assert!(
            msg.contains("no $HOME") || msg.contains("cannot resolve"),
            "diagnostic must name the real cause; got {msg:?}",
        );
        assert!(
            app.state.pending_card_op.is_none(),
            "request must be cleared so user is not locked out of next card op",
        );
    }
}
