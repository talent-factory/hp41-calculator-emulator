//! hp41 — HP-41 calculator emulator CLI (Phase 4: TUI & Input)
//!
//! Entry point: initialises ratatui terminal, runs App event loop, restores terminal.
//! All TUI rendering: ui.rs. Key mapping: keys.rs. PRGM display: prgm_display.rs.

mod app;
mod ui;
mod keys;
mod prgm_display;

#[cfg(test)]
mod tests;

use app::App;

fn main() -> std::io::Result<()> {
    // ratatui::init() does:
    //   1. Creates CrosstermBackend writing to stdout
    //   2. Enables raw mode and enters alternate screen
    //   3. Installs a panic hook that calls ratatui::restore() before printing the panic
    //
    // Returns DefaultTerminal (NOT RestoreTerminalGuard — that type does not exist in 0.30).
    // Hold this value alive until ratatui::restore() is called — do NOT drop it early.
    // D-05, D-17: ratatui::init() is mandatory (not Terminal::new()).
    let terminal = ratatui::init();

    let result = App::new().run(terminal);

    // D-17: explicit restore after the event loop exits (both Ok and Err paths).
    // The panic hook also fires restore() on unhandled panics, but main() returning
    // normally bypasses the hook — explicit call is required.
    ratatui::restore();

    result
}
