//! hp41 — HP-41 calculator emulator CLI
//!
//! Entry point: parses CLI args, loads state, initialises ratatui terminal,
//! runs App event loop, then saves state and restores terminal.

mod app;
mod ui;
mod keys;
mod prgm_display;
mod persistence;
mod help_data;
mod programs;

#[cfg(test)]
mod tests;

use clap::Parser;
use app::App;
use hp41_core::CalcState;

/// HP-41 Calculator Emulator — faithful HP-41C/CV/CX behavioral emulation in the terminal.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the state file (JSON). Loaded on startup, saved on exit and every 30s.
    /// Default: ~/.hp41/autosave.json
    #[arg(long, value_name = "FILE")]
    state_file: Option<std::path::PathBuf>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // Resolve the active state file path (D-02: CLI override or default).
    let state_path = cli.state_file
        .unwrap_or_else(persistence::default_state_path);

    // D-03: load existing state or start fresh; NEVER panic on parse failure.
    let (initial_state, load_message) = match persistence::load_state(&state_path) {
        Ok(state) => (state, None),
        Err(e) if state_path.exists() => {
            // File exists but is corrupt — warn and start fresh.
            let msg = format!("State load failed ({e}); starting fresh");
            (CalcState::new(), Some(msg))
        }
        Err(_) => {
            // File missing — normal first-run case; no message needed.
            (CalcState::new(), None)
        }
    };

    let terminal = ratatui::init();

    let mut app = App::new(initial_state, state_path);
    if let Some(msg) = load_message {
        app.message = Some(msg);
    }

    let result = app.run(terminal);

    ratatui::restore();

    result
}
