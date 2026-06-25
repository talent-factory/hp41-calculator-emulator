#![deny(clippy::unwrap_used)]

use std::sync::Mutex;
use tauri::Manager;

mod app_intents;
pub mod cards;
mod commands;
mod persistence;
mod prefs; // Phase 48 — GUI preferences (theme, future onboarding flag) — stored in ~/.hp41/prefs.json (P59/THEME-05)
#[cfg(target_os = "macos")]
mod shortcut; // macOS global hotkey to toggle the menu-bar popover
#[cfg(target_os = "macos")]
mod tray; // macOS menu-bar mode (tray icon + popover + Accessory policy)
mod tray_helpers; // pure geometry/debounce helpers for the macOS menu-bar popover
pub mod types; // pub so integration tests (lcd_alternation_modal_prompt.rs) can access CalcStateView::from_state

pub type AppState = Mutex<hp41_core::CalcState>;

/// Managed state for GUI preferences — separate from AppState (CalcState) per P59/THEME-05.
/// Stored in `~/.hp41/prefs.json`; never touches `autosave.json`.
pub type PrefsState = Mutex<prefs::GuiPrefs>;

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

/// Bring the already-running instance to the foreground when a second launch is
/// attempted (single-instance plugin callback). On macOS in menu-bar mode this
/// surfaces the popover (under the tray icon if it has been clicked before, else a
/// top-right fallback); otherwise it shows and focuses the main window. Desktop-only,
/// matching the single-instance plugin's availability.
#[cfg(desktop)]
fn focus_existing_instance(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        if let Some(state) = app.try_state::<crate::tray::PopoverState>() {
            if state
                .menu_bar_active
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                crate::tray::surface_popover(app);
                return;
            }
        }
    }
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // tauri-plugin-single-instance MUST be the first plugin registered so it runs
    // before any other plugin can interfere (plugin docs). Desktop-only crate. When
    // a second launch is attempted, the ALREADY-running instance receives this
    // callback and surfaces itself; the second process exits immediately (plugin
    // behavior), so no duplicate tray icon is ever created.
    #[cfg(desktop)]
    let builder =
        tauri::Builder::default().plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_existing_instance(app);
        }));
    #[cfg(not(desktop))]
    let builder = tauri::Builder::default();

    let builder = builder.plugin(tauri_plugin_dialog::init()); // Phase 50 — file dialog plugin for .raw/.card.json import/export

    // tauri-plugin-autostart is desktop-only: its `init`/`MacosLauncher` symbols do
    // not exist on the iOS/Android mobile targets, so registering it unconditionally
    // breaks the `aarch64-apple-ios` cross-compile (v4.1 Phase 53 scaffold spike,
    // P-iOS-08 smoke). Gate it to desktop — desktop behavior is unchanged.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_autostart::init(
        tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        None,
    ));

    // macOS-only: global hotkey plugin for the menu-bar popover toggle. Only one
    // hotkey is ever registered (the actual accelerator is set in setup() from
    // prefs via shortcut::install), so the handler toggles unconditionally on
    // ShortcutState::Pressed without matching a specific Shortcut.
    #[cfg(target_os = "macos")]
    let builder = {
        use tauri_plugin_global_shortcut::ShortcutState;
        builder.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        crate::tray::toggle_popover(app);
                    }
                })
                .build(),
        )
    };

    // tauri-plugin-haptics is mobile-only: iOS UIImpactFeedbackGenerator symbols
    // do not exist on desktop targets. Gating with #[cfg(mobile)] mirrors the
    // #[cfg(desktop)] pattern for autostart above. The JS haptic calls are
    // iOS-gated via isIos, so desktop code never reaches the IPC endpoint.
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_haptics::init());

    builder
        .setup(|app| {
            app_intents::install_app_handle(app.handle());

            // D-03: attempt to load ~/.hp41/autosave.json; fall back to fresh state on any error.
            // D-04: load_state() always resets is_running = false (Pitfall 4 guard).
            //
            // Match the CLI's corrupt-vs-missing split (hp41-cli/src/main.rs:51-62):
            // a missing file is the normal first-run case (silent); an existing-but-
            // unreadable file means the user lost a session, so log to stderr instead
            // of silently dropping it.
            // Phase 48: load GUI preferences FIRST (before CalcState) — preferences are
            // completely independent of CalcState (P59/THEME-05). A missing prefs.json
            // is normal (first-run); load_prefs() silently returns GuiPrefs::default().
            let prefs_path = prefs::prefs_path_for_app(app.handle());
            let initial_prefs = prefs::load_prefs(&prefs_path);
            // Capture the macOS launch mode before initial_prefs is moved into the
            // managed Mutex — used by the macOS setup branch below. Unused on non-macOS.
            #[cfg(target_os = "macos")]
            let macos_launch_mode = initial_prefs.macos_launch_mode.clone();
            // Capture the global hotkey accelerator before initial_prefs is moved —
            // registered at the end of the macOS setup branch. Unused on non-macOS.
            #[cfg(target_os = "macos")]
            let macos_global_shortcut = initial_prefs.global_shortcut.clone();
            app.manage(Mutex::new(initial_prefs));

            let save_path = persistence::state_path_for_app(app.handle());
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
            let cancel_flag: CancelFlag = std::sync::Arc::clone(&initial_state.cancel_requested);
            app.manage(Mutex::new(initial_state));
            app.manage(cancel_flag);

            if let Some(w) = app.get_webview_window("main") {
                let version = env!("CARGO_PKG_VERSION");
                let _ = w.set_title(&format!("HP-41 Calculator — v{version}"));
            }

            // D-01: spawn auto-save background thread — 30s sleep, then lock, then save.
            // D-02: save failures are logged to stderr; no UI notification.
            // Phase 54 PERSIST-01: resolve the iOS-aware path BEFORE spawning, then move-capture
            // the PathBuf into the closure (RESEARCH Pattern 2 — AppHandle not reconstructible
            // inside std::thread::spawn without cloning; resolving here is cleaner).
            let handle = app.handle().clone();
            let thread_save_path = persistence::state_path_for_app(&handle);
            std::thread::spawn(move || {
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

            // ── macOS menu-bar mode (Task 4 of the menu-bar plan) ──
            // Set HP41_SHOW_ON_START to opt out (normal visible window) — used by
            // anyone running the E2E suite on macOS locally.
            #[cfg(target_os = "macos")]
            {
                app.manage(crate::tray::PopoverState::default());
                // Precedence: HP41_SHOW_ON_START (E2E backdoor) > "window" pref > menu-bar.
                if std::env::var_os("HP41_SHOW_ON_START").is_some() || macos_launch_mode == "window"
                {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                } else {
                    match crate::tray::apply_menu_bar_mode(app) {
                        Ok(()) => {
                            // Menu-bar (popover) mode is active — enable auto-hide-on-blur.
                            app.state::<crate::tray::PopoverState>()
                                .menu_bar_active
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        Err(e) => {
                            eprintln!(
                                "hp41-gui: failed to enter menu-bar mode: {e}; showing window"
                            );
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                            }
                        }
                    }
                }
                // Register the global hotkey last — it summons the popover in
                // menu-bar mode and the window in "window" mode alike.
                crate::shortcut::install(app.handle(), &macos_global_shortcut);
            }
            // Non-macOS: the window stays a normal decorated window. Because the
            // bundle config will start hidden (visible:false, a later task),
            // show it explicitly.
            #[cfg(not(target_os = "macos"))]
            {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_intents::take_pending_app_intent,
            commands::dispatch_op,
            commands::get_state,
            commands::sst_step,                // Phase 18 D-05
            commands::bst_step,                // Phase 18 D-05
            commands::run_stop,                // Phase 19 (v2.1) — R/S key toggle
            commands::request_cancel, // Phase 31 Plan 31-02 — flip cancel_requested AtomicBool
            commands::submit_modal,   // Phase 31 Plan 31-03 — R/S submit modal step
            commands::cancel_modal,   // Phase 31 Plan 31-03 — Esc cancel modal
            commands::submit_modal_with_label, // Phase 31 Plan 31-03 — XEQ-by-name FUNCTION NAME? step
            commands::tick_time, // Phase 41 D-41.1 — 100ms periodic tick for live display
            commands::get_prefs, // Phase 48 INFRA-01 — read GUI preferences
            commands::set_pref,  // Phase 48 INFRA-02 — write/persist a GUI preference
            commands::restart_app, // macOS launch-mode toggle — offer relaunch after switch
            commands::is_macos,  // macOS launch-mode toggle — gate the Settings control
            commands::is_ios,    // D-55.1 — iOS platform detection (mirrors is_macos)
            commands::save_state, // Phase 49 KBD-02 — on-demand save (Ctrl+S / F5 in GUI)
            // Phase 50 — .raw file I/O via native OS file dialog
            commands::import_raw_dialog,
            commands::export_raw_dialog,
            commands::import_data_dialog,
            commands::export_data_dialog,
            commands::import_selected_programs,
            // Phase 63 — GUI continuous program run loop (run_program + resume_program)
            commands::run_program,
            commands::resume_program,
            // Phase 64 — interactive GETKEY: event-driven resume with captured keycode
            commands::resume_program_with_key,
            // Phase 67 — reset escape hatch (bypasses dispatch; usable when core path is stuck)
            commands::reset_soft,
            commands::reset_full,
        ])
        .on_window_event(|window, event| {
            #[cfg(target_os = "macos")]
            {
                if let tauri::WindowEvent::Focused(false) = event {
                    if window.label() != "main" {
                        return;
                    }
                    let app = window.app_handle();
                    let Some(state) = app.try_state::<crate::tray::PopoverState>() else {
                        return;
                    };
                    // Auto-hide on blur ONLY in menu-bar (popover) mode. In "window"
                    // launch mode the window must stay visible when it loses focus.
                    if !state
                        .menu_bar_active
                        .load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }
                    // Don't hide while a native file dialog is open (it steals focus and
                    // would otherwise dismiss the popover mid-operation).
                    if state
                        .suppress_hide
                        .load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return;
                    }
                    if let Ok(mut g) = state.last_hidden.lock() {
                        *g = Some(std::time::Instant::now());
                    }
                    let _ = window.hide();
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (window, event); // silence unused warnings off-macOS
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
