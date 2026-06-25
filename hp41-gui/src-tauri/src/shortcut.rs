//! Global-hotkey registration for the macOS menu-bar app.
//!
//! macOS-only: the hotkey toggles the menu-bar popover (and works in "window"
//! launch mode too, as a summon shortcut). Registration goes through
//! `tauri-plugin-global-shortcut`, which uses Carbon `RegisterEventHotKey` on
//! macOS — so it needs **no** Accessibility/Input-Monitoring permission.
//!
//! The accelerator string is stored in `GuiPrefs.global_shortcut` (see `prefs.rs`)
//! and parsed here with the plugin's `Shortcut` `FromStr`. Tokens are the standard
//! Tauri accelerator names joined with `+`, e.g. `"Control+Alt+Command+H"`
//! (modifiers: `Control`/`Ctrl`, `Alt`/`Option`, `Shift`, `Command`/`Cmd`/`Super`;
//! keys: single letters/digits, `Space`, `F1`..`F12`, arrows). The frontend
//! `ShortcutRecorder` emits strings in exactly this format.
//!
//! Only ONE hotkey is ever registered, so the plugin handler in `lib.rs` can toggle
//! unconditionally on `Pressed` without matching a specific `Shortcut`.

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Factory default: `⌃⌥⌘H` (Control + Option + Command + H, "H" for HP-41).
/// Chosen for being practically collision-free on macOS. Re-exported from `prefs`
/// so the accelerator literal has a single source of truth.
pub use crate::prefs::DEFAULT_GLOBAL_SHORTCUT as DEFAULT_ACCEL;

/// Parse an accelerator string into a `Shortcut`.
fn parse(accel: &str) -> Result<Shortcut, String> {
    accel
        .parse::<Shortcut>()
        .map_err(|e| format!("unparseable accelerator {accel:?}: {e}"))
}

/// Register `accel` as the sole global hotkey at startup. Best-effort: on a parse
/// or registration failure it logs and falls back to [`DEFAULT_ACCEL`] so the app
/// still has a working hotkey even if `prefs.json` holds a stale/garbage value.
pub fn install(app: &AppHandle, accel: &str) {
    if try_register(app, accel).is_ok() {
        return;
    }
    if accel != DEFAULT_ACCEL {
        eprintln!("hp41-gui: hotkey {accel:?} failed; falling back to default {DEFAULT_ACCEL:?}");
        let _ = try_register(app, DEFAULT_ACCEL);
    }
}

/// Replace the active hotkey at runtime (called from `set_pref` when the user
/// records a new combo). `previous` is the currently-registered accelerator, used
/// to roll back if the new one cannot be registered. Returns `Err` with a
/// human-readable reason if the new accelerator is invalid or cannot be
/// registered, so the command can reject it (and keep `previous` live).
pub fn reregister(app: &AppHandle, accel: &str, previous: &str) -> Result<(), String> {
    // Validate first so an unparseable value never touches the live registration.
    parse(accel)?;
    if let Err(e) = try_register(app, accel) {
        // `try_register` ran `unregister_all` before the failed `register`, so the
        // old hotkey is already gone. Restore `previous` best-effort — a failed
        // rebind must never leave the app with no working hotkey at all.
        let _ = try_register(app, previous);
        return Err(e);
    }
    Ok(())
}

/// Unregister everything, then register `accel`. Parsing happens before the
/// unregister so an invalid value leaves the previous hotkey intact. The
/// `unregister_all` failure is now propagated (was previously swallowed): if the
/// old bindings cannot be cleared, the caller should know rather than silently
/// stack a second hotkey.
fn try_register(app: &AppHandle, accel: &str) -> Result<(), String> {
    let shortcut = parse(accel)?;
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| {
        format!("failed to clear existing hotkey before registering {accel:?}: {e}")
    })?;
    gs.register(shortcut)
        .map_err(|e| format!("failed to register {accel:?}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_accelerator_parses() {
        assert!(
            parse(DEFAULT_ACCEL).is_ok(),
            "DEFAULT_ACCEL must be a valid accelerator string"
        );
    }

    #[test]
    fn rejects_garbage_accelerator() {
        assert!(parse("not a shortcut").is_err());
    }

    #[test]
    fn parses_a_simple_combo() {
        assert!(parse("CommandOrControl+Shift+C").is_ok());
    }
}
