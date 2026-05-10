#![deny(clippy::unwrap_used)]

use std::sync::Mutex;
use tauri::Manager;

mod commands;
mod key_map;
mod persistence;
mod prgm_display;   // Phase 18 D-03
mod types;

pub type AppState = Mutex<hp41_core::CalcState>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // D-03: attempt to load ~/.hp41/autosave.json; fall back to fresh state on any error.
            // D-04: load_state() always resets is_running = false (Pitfall 4 guard).
            let save_path = persistence::default_state_path();
            let initial_state = persistence::load_state(&save_path)
                .unwrap_or_else(|_| hp41_core::CalcState::new());
            app.manage(Mutex::new(initial_state));

            // D-01: spawn auto-save background thread — 30s sleep, then lock, then save.
            // D-02: save failures are logged to stderr; no UI notification.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let thread_save_path = persistence::default_state_path();
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    // Clone state under lock, then drop guard before disk I/O (CR-01).
                    let state = handle.state::<AppState>();
                    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
                    drop(state);
                    if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
                        eprintln!("auto-save failed: {e}");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::dispatch_op,
            commands::get_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
