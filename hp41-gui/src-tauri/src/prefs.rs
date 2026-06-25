//! GUI preference persistence for hp41-gui.
//!
//! Stores user-facing preferences in `~/.hp41/prefs.json` — completely separate from
//! `~/.hp41/autosave.json` which holds the calculator state. This separation is a hard
//! constraint (P59 / THEME-05): theme preference MUST NEVER appear in the autosave file.
//!
//! Design mirrors `persistence.rs` but with two key differences:
//! 1. No `StateFile` version wrapper — prefs.json is simpler (just the struct, pretty-printed).
//! 2. `load_prefs` returns `GuiPrefs` directly (not `Result`) — a missing file is the normal
//!    first-run case, not an error. Corrupt JSON also returns `GuiPrefs::default()`.
//!
//! Phase 49 added `pub onboarding_done: bool` with `#[serde(default)]` to this struct.
//! The `#[serde(default)]` on each field ensures forward/backward compatibility: a prefs.json
//! written by an older version will load cleanly into a newer struct with new fields defaulted.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// User-facing GUI preferences. Stored in `~/.hp41/prefs.json`.
///
/// THEME-05: This struct must NEVER be embedded in the autosave file or shared with
/// the core calculator state — preferences are fully orthogonal to calculator memory.
///
/// # Field notes
/// - `theme`: one of "dark" | "light" | "classic-beige" | "high-contrast" (D-48.8).
///   Defaults to "dark" on missing prefs.json (first-run default).
/// - `onboarding_done`: set to `true` once the user dismisses the first-run quick-start
///   guide (D-49.4 / ONBOARD-05). Never appears in autosave.json (P59).
/// - `macos_launch_mode`: "menu-bar" (status-item accessory) or "window" (normal decorated
///   window). Ignored on Windows/Linux. Defaults to "menu-bar" so existing prefs.json files
///   (no field) keep the current menu-bar behavior (ADR-v4.1-001).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub onboarding_done: bool,
    /// macOS-only launch mode: "menu-bar" (status-item accessory) or "window"
    /// (normal decorated window). Ignored on Windows/Linux. Defaults to "menu-bar"
    /// so existing prefs.json files (no field) keep the current menu-bar behavior.
    #[serde(default = "default_launch_mode")]
    pub macos_launch_mode: String,
    /// macOS-only global hotkey accelerator that toggles the menu-bar popover
    /// (e.g. "Control+Alt+Command+H"). Ignored on Windows/Linux. Defaults to
    /// `DEFAULT_GLOBAL_SHORTCUT` so existing prefs.json files (no field) keep
    /// the established hotkey. The accepted accelerator grammar lives in
    /// `shortcut.rs`; the macOS registration falls back to the default for any
    /// non-empty-but-unparseable value.
    #[serde(default = "default_global_shortcut")]
    pub global_shortcut: String,
}

/// Default theme value — "dark" per D-48.8.
fn default_theme() -> String {
    "dark".to_string()
}

/// Default launch mode — "menu-bar" (the established macOS behavior, ADR-v4.1-001).
fn default_launch_mode() -> String {
    "menu-bar".to_string()
}

/// Factory-default global hotkey accelerator — `⌃⌥⌘H`. Single source of truth,
/// shared by `GuiPrefs::default()` and the macOS `shortcut` module.
pub const DEFAULT_GLOBAL_SHORTCUT: &str = "Control+Alt+Command+H";

/// Default global hotkey — [`DEFAULT_GLOBAL_SHORTCUT`].
fn default_global_shortcut() -> String {
    DEFAULT_GLOBAL_SHORTCUT.to_string()
}

impl Default for GuiPrefs {
    fn default() -> Self {
        GuiPrefs {
            theme: default_theme(),
            onboarding_done: false,
            macos_launch_mode: default_launch_mode(),
            global_shortcut: default_global_shortcut(),
        }
    }
}

/// Resolve the default preferences file path: `~/.hp41/prefs.json`.
///
/// Fallback: `./.hp41/prefs.json` if `home_dir()` returns None (mirrors `persistence.rs`
/// `default_state_path()` — same pattern, different target file).
pub fn default_prefs_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("prefs.json")
}

/// AppHandle-aware preferences path resolver. On mobile (iOS) uses app_local_data_dir()
/// → Library/Application Support/<bundle_id>/prefs.json.
/// On desktop delegates to default_prefs_path() — desktop behavior unchanged.
///
/// Phase 54 PERSIST-01 / D-54.3: mirrors persistence::state_path_for_app(), filename only differs.
/// Pitfall 2: unwrap_or_else handles Tauri #12552 "Permission Denied" gracefully.
/// Pitfall 3: fallback uses Library/Application Support, NOT .hp41 (container-root dot-dir).
#[allow(unused_variables)] // `handle` is used only in #[cfg(mobile)] branch; intentional on desktop
pub fn prefs_path_for_app(handle: &tauri::AppHandle) -> PathBuf {
    #[cfg(mobile)]
    {
        use tauri::Manager; // `.path()` is a Manager-trait method; scoped to mobile to avoid a desktop unused-import warning
        handle
            .path()
            .app_local_data_dir()
            .unwrap_or_else(|e| {
                eprintln!("hp41: app_local_data_dir failed ({e}), falling back to HOME");
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Library")
                    .join("Application Support")
                    .join("ch.talent-factory.hp41")
            })
            .join("prefs.json")
    }
    #[cfg(not(mobile))]
    {
        default_prefs_path()
    }
}

/// Persist `prefs` to `path` as pretty-printed JSON.
///
/// Creates the parent directory if it does not exist (mirrors `save_state`).
/// Returns `Err` on I/O failure; the caller should log and continue.
pub fn save_prefs(path: &Path, prefs: &GuiPrefs) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, prefs).map_err(std::io::Error::other)
}

/// Load preferences from `path`, returning `GuiPrefs::default()` on any failure.
///
/// Unlike `load_state`, this function never returns `Err`:
/// - Missing file → `GuiPrefs::default()` (normal first-run case, no log).
/// - Corrupt JSON → `GuiPrefs::default()` (safe fallback) plus a `stderr` warning
///   so a silent reset-to-default is debuggable.
///
/// P59 / THEME-05: this function is fully isolated from the calculator state and autosave.json.
pub const VALID_THEMES: &[&str] = &["dark", "light", "classic-beige", "high-contrast"];
pub const VALID_LAUNCH_MODES: &[&str] = &["menu-bar", "window"];

pub fn load_prefs(path: &Path) -> GuiPrefs {
    let mut prefs: GuiPrefs = match fs::File::open(path) {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(e) => {
                // Cosmetic prefs only (isolated from autosave.json), so a parse
                // failure is non-fatal — but log it instead of silently reverting.
                eprintln!("hp41: ignoring corrupt prefs at {}: {e}", path.display());
                GuiPrefs::default()
            }
        },
        Err(_) => GuiPrefs::default(), // missing file → normal first-run
    };
    if !VALID_THEMES.contains(&prefs.theme.as_str()) {
        prefs.theme = default_theme();
    }
    if !VALID_LAUNCH_MODES.contains(&prefs.macos_launch_mode.as_str()) {
        prefs.macos_launch_mode = default_launch_mode();
    }
    // Only an emptiness sanity-check here: full accelerator parsing requires the
    // macOS-only `shortcut::Shortcut` type (prefs.rs compiles on all platforms,
    // including iOS). The macOS `shortcut::install` already falls back to the
    // default for any non-empty-but-unparseable value, so an empty string is the
    // only case we must repair to keep a usable hotkey.
    if prefs.global_shortcut.trim().is_empty() {
        prefs.global_shortcut = default_global_shortcut();
    }
    prefs
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("hp41_test_{name}"))
            .join("prefs.json")
    }

    #[test]
    fn test_roundtrip() {
        let path = temp_path("prefs_roundtrip");
        let prefs = GuiPrefs {
            theme: "light".to_string(),
            onboarding_done: false,
            macos_launch_mode: "menu-bar".to_string(),
            global_shortcut: default_global_shortcut(),
        };
        save_prefs(&path, &prefs).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(loaded.theme, "light", "roundtrip must preserve theme value");
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_missing_file_returns_default() {
        let path = temp_path("prefs_missing_should_not_exist_xyz987");
        // Ensure the path does not exist
        let _ = fs::remove_dir_all(path.parent().unwrap());
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.theme, "dark",
            "missing prefs.json must return default theme 'dark'"
        );
    }

    #[test]
    fn test_corrupt_json_returns_default() {
        let path = temp_path("prefs_corrupt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"this is not valid json {{ garbage").unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.theme, "dark",
            "corrupt prefs.json must return default theme 'dark'"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_default_theme_is_dark() {
        let prefs = GuiPrefs::default();
        assert_eq!(
            prefs.theme, "dark",
            "GuiPrefs::default() must have theme 'dark'"
        );
    }

    #[test]
    fn test_unknown_theme_falls_back_to_default() {
        let path = temp_path("prefs_unknown_theme");
        let prefs = GuiPrefs {
            theme: "neon-pink".to_string(),
            onboarding_done: false,
            macos_launch_mode: "menu-bar".to_string(),
            global_shortcut: default_global_shortcut(),
        };
        save_prefs(&path, &prefs).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.theme, "dark",
            "unknown theme in prefs.json must fall back to 'dark'"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_launch_mode_roundtrip() {
        let path = temp_path("prefs_launch_mode_roundtrip");
        let prefs = GuiPrefs {
            theme: "dark".to_string(),
            onboarding_done: false,
            macos_launch_mode: "window".to_string(),
            global_shortcut: default_global_shortcut(),
        };
        save_prefs(&path, &prefs).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.macos_launch_mode, "window",
            "roundtrip must preserve macos_launch_mode"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// Backward compat: a prefs.json without `macos_launch_mode` (written before this
    /// feature) must load with the default "menu-bar" via #[serde(default)]. This is the
    /// load-bearing guarantee that existing macOS users stay in menu-bar mode (D-2).
    #[test]
    fn test_launch_mode_serde_default() {
        let path = temp_path("prefs_launch_mode_default");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, br#"{"theme":"dark","onboarding_done":true}"#).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.macos_launch_mode, "menu-bar",
            "missing macos_launch_mode must default to 'menu-bar' (backward compat)"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_unknown_launch_mode_falls_back_to_default() {
        let path = temp_path("prefs_unknown_launch_mode");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            br#"{"theme":"dark","onboarding_done":false,"macos_launch_mode":"hologram"}"#,
        )
        .unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.macos_launch_mode, "menu-bar",
            "unknown macos_launch_mode must fall back to 'menu-bar'"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_global_shortcut_roundtrip() {
        let path = temp_path("prefs_global_shortcut_roundtrip");
        let prefs = GuiPrefs {
            theme: "dark".to_string(),
            onboarding_done: false,
            macos_launch_mode: "menu-bar".to_string(),
            global_shortcut: "Control+Shift+K".to_string(),
        };
        save_prefs(&path, &prefs).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.global_shortcut, "Control+Shift+K",
            "roundtrip must preserve global_shortcut"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// Backward compat: a prefs.json without `global_shortcut` (written before this
    /// feature) must load with the default accelerator via `#[serde(default)]`.
    #[test]
    fn test_global_shortcut_serde_default() {
        let path = temp_path("prefs_global_shortcut_default");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, br#"{"theme":"dark","onboarding_done":true}"#).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.global_shortcut, DEFAULT_GLOBAL_SHORTCUT,
            "missing global_shortcut must default to DEFAULT_GLOBAL_SHORTCUT (backward compat)"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_empty_global_shortcut_falls_back_to_default() {
        let path = temp_path("prefs_global_shortcut_empty");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            br#"{"theme":"dark","onboarding_done":false,"macos_launch_mode":"menu-bar","global_shortcut":"  "}"#,
        )
        .unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.global_shortcut, DEFAULT_GLOBAL_SHORTCUT,
            "blank global_shortcut must fall back to the default accelerator"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// D-49.4 / ONBOARD-05: `onboarding_done = true` must survive a save/load roundtrip.
    #[test]
    fn test_onboarding_done_roundtrip() {
        let path = temp_path("prefs_onboarding_roundtrip");
        let prefs = GuiPrefs {
            theme: "dark".to_string(),
            onboarding_done: true,
            macos_launch_mode: "menu-bar".to_string(),
            global_shortcut: default_global_shortcut(),
        };
        save_prefs(&path, &prefs).unwrap();
        let loaded = load_prefs(&path);
        assert!(
            loaded.onboarding_done,
            "onboarding_done=true must survive a prefs save/load roundtrip"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// Backward compat: a prefs.json that does not contain `onboarding_done` (written by
    /// Phase 48 or earlier) must load with `onboarding_done == false` via `#[serde(default)]`.
    #[test]
    fn test_onboarding_done_serde_default() {
        let path = temp_path("prefs_onboarding_default");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        // Write JSON without onboarding_done field (old format).
        fs::write(&path, br#"{"theme":"dark"}"#).unwrap();
        let loaded = load_prefs(&path);
        assert!(
            !loaded.onboarding_done,
            "missing onboarding_done field must default to false (backward compat)"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_default_prefs_path_ends_with_prefs_json() {
        let path = default_prefs_path();
        assert!(
            path.ends_with(".hp41/prefs.json"),
            "default_prefs_path() must end with .hp41/prefs.json, got: {}",
            path.display()
        );
        assert!(
            !path.to_string_lossy().contains("autosave"),
            "default_prefs_path() must NOT reference autosave.json (THEME-05)"
        );
    }
}
