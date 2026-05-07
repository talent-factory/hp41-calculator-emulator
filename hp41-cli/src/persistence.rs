//! State persistence for hp41-cli: save/load CalcState to/from JSON.
//!
//! D-01: Default path: ~/.hp41/autosave.json
//! D-03: Load failures start fresh — NEVER panic; caller handles Err.
//! D-06: StateFile version wrapper { "version": 1, "state": {...} }
//! D-07: HpNum serializes as string via rust_decimal::serde::str (hp41-core Plan 01)
//! Security: D-03 — serde_json::from_reader returns Err on malformed JSON; never unwrap.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use hp41_core::CalcState;

/// Version-tagged wrapper for forward-compatible state files.
/// D-06: `version` enables future migration without breaking existing saves.
#[derive(Serialize, Deserialize)]
pub struct StateFile {
    pub version: u32,
    pub state: CalcState,
}

impl StateFile {
    pub fn current(state: CalcState) -> Self {
        StateFile { version: 1, state }
    }
}

/// Resolve the default state file path: ~/.hp41/autosave.json
/// Fallback: ./.hp41/autosave.json if home_dir() returns None (D-01, RESEARCH Pitfall 6).
pub fn default_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("autosave.json")
}

/// Save CalcState to path as pretty-printed JSON with version wrapper.
/// Creates the parent directory if it does not exist (D-01).
/// Returns Err on I/O failure; caller shows error in status bar (D-03).
pub fn save_state(path: &Path, state: &CalcState) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = fs::File::create(path)?;
    let wrapper = StateFile::current(state.clone());
    serde_json::to_writer_pretty(file, &wrapper).map_err(std::io::Error::other)
}

/// Load CalcState from a state file.
/// Returns Err on missing file or parse failure — NEVER panics (D-03, ASVS V5).
/// ALWAYS resets is_running = false on load (RESEARCH Pitfall 4 — corrupt state guard).
pub fn load_state(path: &Path) -> Result<CalcState, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let wrapper: StateFile = serde_json::from_reader(file)?;
    let mut state = wrapper.state;
    // Pitfall 4: never resume mid-execution after a reload.
    // A state file written during program execution could have is_running=true.
    state.is_running = false;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hp41_core::CalcState;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("hp41_test_{name}"))
            .join("state.json")
    }

    #[test]
    fn test_roundtrip_fresh_state() {
        let path = temp_path("roundtrip");
        let state = CalcState::new();
        save_state(&path, &state).unwrap();
        let loaded = load_state(&path).unwrap();
        assert!(loaded.stack.x.is_zero());
        assert!(!loaded.is_running);
        assert_eq!(loaded.regs.len(), 100);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_missing_file_returns_err() {
        let path = PathBuf::from("/nonexistent/path/hp41_no_such_file.json");
        assert!(load_state(&path).is_err(), "missing file must return Err");
    }

    #[test]
    fn test_corrupt_json_returns_err() {
        let path = temp_path("corrupt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"this is not valid json {{ ").unwrap();
        assert!(load_state(&path).is_err(), "corrupt JSON must return Err");
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_is_running_reset_on_load() {
        let path = temp_path("running");
        let mut state = CalcState::new();
        state.is_running = true; // simulate killed-during-execution
        save_state(&path, &state).unwrap();
        let loaded = load_state(&path).unwrap();
        assert!(!loaded.is_running, "is_running must be false after load");
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_user_mode_roundtrip() {
        let path = temp_path("user_mode");
        let mut state = CalcState::new();
        state.user_mode = true;
        state.key_assignments.insert('a', "FIBONACCI".to_string());
        save_state(&path, &state).unwrap();
        let loaded = load_state(&path).unwrap();
        assert!(loaded.user_mode);
        assert_eq!(
            loaded.key_assignments.get(&'a').map(|s| s.as_str()),
            Some("FIBONACCI")
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_version_field_in_json() {
        let path = temp_path("version");
        let state = CalcState::new();
        save_state(&path, &state).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("\"version\""),
            "JSON must contain version field"
        );
        assert!(
            content.contains("\"state\""),
            "JSON must contain state wrapper"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
