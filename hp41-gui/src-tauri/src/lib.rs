#![deny(clippy::unwrap_used)]

use std::sync::Mutex;
use tauri::Manager;

pub mod cards;
mod commands;
mod key_map;
mod persistence;
mod prgm_display; // Phase 18 D-03
mod types;

pub type AppState = Mutex<hp41_core::CalcState>;

/// Separate managed state for the cancellation flag (Phase 31 / GUI-05 / Plan 31-02).
///
/// This MUST be a separate `tauri::State` from `AppState` to avoid deadlock:
/// `request_cancel` must flip the AtomicBool without acquiring the AppState Mutex,
/// because `dispatch_op` holds the AppState Mutex for the entire duration of a
/// long-running op (INTG/SOLVE/DIFEQ). If `request_cancel` tried to lock AppState,
/// it would deadlock (Pitfall 1 / RESEARCH.md §"AppState Mutex + AtomicBool interleaving").
///
/// The Arc inside is the SAME Arc as `CalcState.cancel_requested` — cloned at setup
/// time before the CalcState is wrapped in the Mutex. The solver loops in
/// `op_integ`/`op_solve`/`op_difeq` read it via `state.cancel_requested.load(Relaxed)`.
/// `request_cancel` writes it via `cancel_flag.store(true, Relaxed)`.
pub type CancelFlag = std::sync::Arc<std::sync::atomic::AtomicBool>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // D-03: attempt to load ~/.hp41/autosave.json; fall back to fresh state on any error.
            // D-04: load_state() always resets is_running = false (Pitfall 4 guard).
            //
            // Match the CLI's corrupt-vs-missing split (hp41-cli/src/main.rs:51-62):
            // a missing file is the normal first-run case (silent); an existing-but-
            // unreadable file means the user lost a session, so log to stderr instead
            // of silently dropping it.
            let save_path = persistence::default_state_path();
            let initial_state = match persistence::load_state(&save_path) {
                Ok(state) => state,
                Err(e) if save_path.exists() => {
                    eprintln!(
                        "hp41-gui: state load failed for {} ({e}); starting fresh",
                        save_path.display()
                    );
                    hp41_core::CalcState::new()
                }
                Err(_) => hp41_core::CalcState::new(),
            };
            // Clone the Arc<AtomicBool> out BEFORE wrapping initial_state in the Mutex.
            // This gives us a separate CancelFlag handle that request_cancel can flip
            // WITHOUT acquiring the AppState Mutex — the deadlock-avoidance invariant
            // (Pitfall 1 / RESEARCH.md §"AppState Mutex + AtomicBool interleaving").
            // The Arc is shared: solver loops read via CalcState.cancel_requested;
            // request_cancel writes via this cloned Arc.
            let cancel_flag: CancelFlag =
                std::sync::Arc::clone(&initial_state.cancel_requested);
            app.manage(Mutex::new(initial_state));
            app.manage(cancel_flag);

            // D-01: spawn auto-save background thread — 30s sleep, then lock, then save.
            // D-02: save failures are logged to stderr; no UI notification.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let thread_save_path = persistence::default_state_path();
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    // Clone state under lock, then drop guard before disk I/O (CR-01).
                    // Note: the MutexGuard is released when the .clone() above returns;
                    // `state` here is just a tauri::State reference wrapper, not the guard.
                    let state = handle.state::<AppState>();
                    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
            commands::sst_step,       // Phase 18 D-05
            commands::bst_step,       // Phase 18 D-05
            commands::run_stop,       // Phase 19 (v2.1) — R/S key toggle
            commands::request_cancel, // Phase 31 Plan 31-02 — flip cancel_requested AtomicBool
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
