//! macOS menu-bar (status-item) mode for the HP-41 GUI.
//!
//! Everything here is macOS-only. On other platforms the app keeps its normal
//! decorated window (see `apply_menu_bar_mode` callers in lib.rs). The frontend
//! and IPC contract are unchanged; this module only manages the window's
//! presentation (Accessory policy, decorations off, always-on-top, hidden start)
//! and a tray icon whose left-click toggles a borderless popover and whose
//! right-click opens a small menu.

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;

use crate::tray_helpers::{
    compute_fallback_position, compute_popover_position, fit_inner_height, should_show_after_hide,
};

/// Shared state for the popover toggle. Managed via `app.manage`.
/// - `last_hidden`: when the window was last hidden (flicker-guard, see tray_helpers).
/// - `suppress_hide`: true while a native file dialog is open, so the blur it
///   causes does not hide the popover (set/cleared in commands.rs).
/// - `menu_bar_active`: true only when the app actually entered menu-bar (popover)
///   mode at startup. The blur handler auto-hides the window ONLY when this is set,
///   so the "window" launch mode keeps a normal window that stays visible on blur.
/// - `last_tray_rect`: the tray icon's physical rect (icon_x, icon_w, icon_bottom)
///   captured on the most recent tray click. The global hotkey and single-instance
///   re-launch reuse it to position the popover under the icon; before the first
///   click it is `None` and callers fall back to a top-right anchor.
#[derive(Default)]
pub struct PopoverState {
    pub last_hidden: Mutex<Option<Instant>>,
    pub suppress_hide: AtomicBool,
    pub menu_bar_active: AtomicBool,
    pub last_tray_rect: Mutex<Option<(i32, i32, i32)>>,
}

const DEBOUNCE: Duration = Duration::from_millis(250);
// DESIGN_WIDTH/HEIGHT mirror the frontend layout size in `../../src/scale.ts`
// (DESIGN_WIDTH/DESIGN_HEIGHT) and the window size in `tauri.conf.json`. These
// three are a frozen triple — change all of them together or the popover window
// will desync from the React layout's letterboxing.
const DESIGN_HEIGHT: f64 = 1020.0;
const DESIGN_WIDTH: f64 = 440.0;
const MAX_SCREEN_FRACTION: f64 = 0.92;
/// Approximate macOS menu-bar height (logical px) used only for the fallback
/// popover anchor when no real tray-icon rect has been captured yet.
const MENU_BAR_LOGICAL: f64 = 24.0;

/// Resize the popover to fit the current monitor height, position it centered
/// under the tray icon, then show + focus it.
fn show_popover(window: &WebviewWindow, icon_x: i32, icon_w: i32, icon_bottom: i32) {
    // Clamp height to the monitor so a 1020px layout fits short laptop screens.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = monitor.scale_factor();
        let screen_h_logical = monitor.size().height as f64 / scale;
        let h = fit_inner_height(DESIGN_HEIGHT, screen_h_logical, MAX_SCREEN_FRACTION);
        let _ = window.set_size(LogicalSize::new(DESIGN_WIDTH, h));
    }

    // Position using physical pixels (icon rect is physical).
    let win_w_physical = window
        .outer_size()
        .map(|s| s.width as i32)
        .unwrap_or(DESIGN_WIDTH as i32);
    let pos = compute_popover_position(icon_x, icon_w, icon_bottom, win_w_physical);
    let _ = window.set_position(PhysicalPosition::new(pos.x, pos.y));

    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_popover(app: &AppHandle, window: &WebviewWindow) {
    let _ = window.hide();
    // `last_hidden` is also written by the blur handler in lib.rs. The two writes
    // both feed the 250ms `should_show_after_hide` debounce, which suppresses the
    // tray click that immediately follows a blur-hide (the click-to-close gesture).
    // KNOWN LIMITATION: a "click away → click tray to reopen" within 250ms can be
    // wrongly suppressed. Whether this is perceptible depends on macOS AppKit event
    // ordering and must be confirmed in the interactive smoke; if it misbehaves,
    // arming the debounce only from the blur handler (not here) is the first fix to try.
    if let Some(state) = app.try_state::<PopoverState>() {
        if let Ok(mut g) = state.last_hidden.lock() {
            *g = Some(Instant::now());
        }
    }
}

/// Show the popover using the last captured tray-icon rect, or — if the tray has
/// never been clicked this session — a top-right fallback anchor near the menu bar.
/// Shared by the tray click, the global hotkey, and single-instance re-launch.
fn show_popover_at_last_rect(app: &AppHandle, window: &WebviewWindow) {
    let last_rect = app
        .try_state::<PopoverState>()
        .and_then(|s| s.last_tray_rect.lock().ok().map(|g| *g))
        .flatten();

    if let Some((icon_x, icon_w, icon_bottom)) = last_rect {
        show_popover(window, icon_x, icon_w, icon_bottom);
        return;
    }

    // No tray rect yet: resize to fit, then pin to the top-right corner.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let scale = monitor.scale_factor();
        let screen_h_logical = monitor.size().height as f64 / scale;
        let h = fit_inner_height(DESIGN_HEIGHT, screen_h_logical, MAX_SCREEN_FRACTION);
        let _ = window.set_size(LogicalSize::new(DESIGN_WIDTH, h));

        let win_w = window
            .outer_size()
            .map(|s| s.width as i32)
            .unwrap_or(DESIGN_WIDTH as i32);
        let menu_bar_h = (MENU_BAR_LOGICAL * scale).round() as i32;
        let pos = compute_fallback_position(monitor.size().width as i32, win_w, menu_bar_h);
        let _ = window.set_position(PhysicalPosition::new(pos.x, pos.y));
    }
    let _ = window.show();
    let _ = window.set_focus();
}

/// Toggle the popover: hide it if visible, otherwise show it (flicker-guarded).
/// Entry point for the global hotkey and the tray left-click.
pub fn toggle_popover(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        hide_popover(app, &window);
        return;
    }
    let last_hidden = app
        .try_state::<PopoverState>()
        .and_then(|s| s.last_hidden.lock().ok().map(|g| *g))
        .flatten();
    if !should_show_after_hide(last_hidden, Instant::now(), DEBOUNCE) {
        return; // this trigger is the one that just closed the popover
    }
    show_popover_at_last_rect(app, &window);
}

/// Bring the running instance to the foreground without toggling it off — used by
/// the single-instance plugin when a second launch is attempted. If already
/// visible, just refocus; otherwise show it (no flicker debounce: an explicit
/// re-launch should always surface the window).
pub fn surface_popover(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.set_focus();
    } else {
        show_popover_at_last_rect(app, &window);
    }
}

/// Build the tray icon, its menu, and event handlers. macOS only.
pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let handle = app.handle();

    // ---- right-click menu ----
    let about_i = MenuItemBuilder::with_id("about", "About HP-41 Calculator").build(app)?;
    let start_login_i = CheckMenuItemBuilder::with_id("start_login", "Start at Login")
        .checked(handle.autolaunch().is_enabled().unwrap_or(false))
        .build(app)?;
    let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&about_i)
        .item(&start_login_i)
        .separator()
        .item(&quit_i)
        .build()?;

    // Clonable handle so the on_menu_event closure can update the checkmark after
    // a toggle (CheckMenuItem handles are Arc-backed in Tauri 2).
    let start_login_for_event = start_login_i.clone();

    // Embed the icon in the binary rather than resolving it from the bundle's
    // resource dir: BaseDirectory::Resource is not populated in `tauri dev`
    // (the PNG isn't a declared bundle resource), which made from_path fail with
    // ENOENT and abort menu-bar mode. include_bytes works identically in dev and
    // bundled builds. The `tray-icon` + `image-png` Cargo features decode it.
    let icon_bytes = include_bytes!("../icons/tray-template.png");

    TrayIconBuilder::with_id("hp41-tray")
        .icon(tauri::image::Image::from_bytes(icon_bytes)?)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false) // left click is the popover toggle, not the menu
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "about" => {
                let version = env!("CARGO_PKG_VERSION");
                app.dialog()
                    .message(format!("HP-41 Calculator\nVersion {version}"))
                    .title("About")
                    .blocking_show();
            }
            "start_login" => {
                let mgr = app.autolaunch();
                let now_enabled = mgr.is_enabled().unwrap_or(false);
                let result = if now_enabled {
                    mgr.disable()
                } else {
                    mgr.enable()
                };
                if let Err(e) = result {
                    eprintln!("hp41-gui: failed to toggle Start at Login: {e}");
                }
                // Reflect the effective state in the checkmark (covers both a
                // successful toggle and a failed one that left the old state).
                let effective = mgr.is_enabled().unwrap_or(false);
                if let Err(e) = start_login_for_event.set_checked(effective) {
                    eprintln!("hp41-gui: failed to update Start at Login checkmark: {e}");
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                // Capture the icon rect (physical px) so the global hotkey and a
                // single-instance re-launch can position the popover under the icon.
                // `rect.position` / `rect.size` are `dpi` enums in 2.11; convert via
                // the window scale factor before storing plain integers.
                let scale = app
                    .get_webview_window("main")
                    .and_then(|w| w.scale_factor().ok())
                    .unwrap_or(1.0);
                let pos_phys = rect.position.to_physical::<i32>(scale);
                let size_phys = rect.size.to_physical::<i32>(scale);
                let icon_bottom = pos_phys.y + size_phys.height;
                if let Some(state) = app.try_state::<PopoverState>() {
                    if let Ok(mut g) = state.last_tray_rect.lock() {
                        *g = Some((pos_phys.x, size_phys.width, icon_bottom));
                    }
                }
                // Single shared toggle path (hide if visible, else flicker-guarded show).
                toggle_popover(app);
            }
        })
        .build(app)?;

    Ok(())
}

/// Apply pure menu-bar presentation to the main window: no Dock icon, no
/// decorations, always on top, hidden until the tray toggles it.
pub fn apply_menu_bar_mode(app: &mut App) -> tauri::Result<()> {
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_decorations(false);
        let _ = win.set_always_on_top(true);
        let _ = win.hide();
    }
    setup_tray(app)
}
