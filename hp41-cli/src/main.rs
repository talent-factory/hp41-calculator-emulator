//! hp41 — HP-41 calculator emulator CLI (Phase 4: TUI & Input)
//!
//! Entry point: parses CLI args, initialises ratatui terminal, runs App event loop,
//! then restores terminal.
//!
//! All TUI rendering: ui.rs
//! Key mapping:       keys.rs
//! PRGM step display: prgm_display.rs
//! App event loop:    app.rs

mod app;
mod ui;
mod keys;
mod prgm_display;

#[cfg(test)]
mod tests;

use clap::Parser;
use app::App;

/// HP-41 Calculator Emulator — faithful HP-41C/CV/CX behavioral emulation in the terminal.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // Phase 5: add --state <path> to load a persisted CalcState from a JSON file (PERS-01).
}

fn main() -> std::io::Result<()> {
    // Parse CLI args — exits with help/version if requested; otherwise continues.
    let _cli = Cli::parse();

    // ratatui::init() does all terminal setup:
    //   1. Creates CrosstermBackend writing to stdout
    //   2. Enables raw mode and enters alternate screen
    //   3. Installs a panic hook that calls ratatui::restore() before printing the panic
    //
    // D-05 / D-17: ratatui::init() is mandatory (never Terminal::new()).
    // The returned DefaultTerminal is NOT RestoreTerminalGuard — that type does not exist
    // in ratatui 0.30. Hold this value alive; do NOT drop it before ratatui::restore().
    let terminal = ratatui::init();

    let result = App::new().run(terminal);

    // D-17: explicit restore must be called after the event loop returns (both Ok and Err).
    // The panic hook also fires restore() on unhandled panics, but normal return bypasses
    // the hook — explicit call here is mandatory.
    ratatui::restore();

    result
}
