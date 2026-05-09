#![deny(clippy::unwrap_used)]

use std::sync::Mutex;
use tauri::Manager;

pub type AppState = Mutex<hp41_core::CalcState>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(hp41_core::CalcState::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tauri commands registered here in Phase 14
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
