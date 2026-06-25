//! State persistence for hp41-gui: save/load CalcState to/from JSON.
//!
//! D-01 (GUI): Default path: ~/.hp41/autosave.json — shared with hp41-cli for SC-4 interop.
//! D-03: Load failures start fresh — NEVER panic; caller handles Err.
//! D-06: StateFile version wrapper { "version": 1, "state": {...} }
//! D-07: HpNum serializes as string via rust_decimal::serde::str (hp41-core Plan 01)
//! Security: D-03 — serde_json::from_reader returns Err on malformed JSON; never unwrap.

#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};

pub use hp41_app::PersistenceError;
use hp41_core::CalcState;

/// Resolve the default state file path: ~/.hp41/autosave.json
/// Fallback: ./.hp41/autosave.json if home_dir() returns None (D-01, RESEARCH Pitfall 6).
pub fn default_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("autosave.json")
}

/// AppHandle-aware path resolver. On mobile (iOS) uses app_local_data_dir()
/// → Library/Application Support/<bundle_id>/autosave.json.
/// On desktop delegates to default_state_path() — desktop behavior unchanged.
///
/// Phase 54 PERSIST-01: iOS sandbox container path; desktop `~/.hp41/` unchanged.
/// Pitfall 2: unwrap_or_else handles Tauri #12552 "Permission Denied" gracefully.
/// Pitfall 3: fallback uses Library/Application Support, NOT .hp41 (container-root dot-dir).
#[allow(unused_variables)] // `handle` is used only in #[cfg(mobile)] branch; intentional on desktop
pub fn state_path_for_app(handle: &tauri::AppHandle) -> PathBuf {
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
            .join("autosave.json")
    }
    #[cfg(not(mobile))]
    {
        default_state_path()
    }
}

/// Save CalcState to path as pretty-printed JSON with version wrapper.
/// Creates the parent directory if it does not exist (D-01).
/// Returns Err on I/O failure; caller shows error in status bar (D-03).
pub fn save_state(path: &Path, state: &CalcState) -> Result<(), PersistenceError> {
    hp41_app::save_state(path, state)
}

/// Load CalcState from a state file.
/// Returns Err on missing file or parse failure — NEVER panics (D-03, ASVS V5).
/// ALWAYS resets is_running = false on load (RESEARCH Pitfall 4 — corrupt state guard).
pub fn load_state(path: &Path) -> Result<CalcState, PersistenceError> {
    hp41_app::load_state(path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    /// Phase 54 PERSIST-01: desktop path unchanged after adding state_path_for_app().
    /// Ensures the new resolver does not break the existing default_state_path() output.
    #[test]
    fn test_default_state_path_ends_with_dot_hp41_autosave() {
        let path = default_state_path();
        assert!(
            path.ends_with(".hp41/autosave.json"),
            "default_state_path() must end with .hp41/autosave.json, got: {}",
            path.display()
        );
    }

    /// Phase 54 PERSIST-01: mobile fallback path construction test.
    /// The #[cfg(mobile)] live branch is compile-gated to iOS targets; on the host
    /// we test the fallback PathBuf construction directly (RESEARCH Validation Architecture).
    /// The fallback must use Library/Application Support/ch.talent-factory.hp41, not .hp41.
    #[test]
    fn test_mobile_fallback_path_construction() {
        // Simulate the fallback chain from state_path_for_app's #[cfg(mobile)] branch.
        // Mirrors Pattern 1 / Pitfall 3: fallback must NOT use .hp41 (container-root dot-dir).
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let fallback = base
            .join("Library")
            .join("Application Support")
            .join("ch.talent-factory.hp41")
            .join("autosave.json");
        let s = fallback.to_string_lossy();
        assert!(
            s.contains("Application Support"),
            "mobile fallback path must contain 'Application Support', got: {s}"
        );
        assert!(
            s.contains("ch.talent-factory.hp41"),
            "mobile fallback path must contain bundle id 'ch.talent-factory.hp41', got: {s}"
        );
        assert!(
            s.ends_with("autosave.json"),
            "mobile fallback path must end with autosave.json, got: {s}"
        );
        // Verify the fallback does NOT land at the container-root .hp41/ location (Pitfall 3).
        let wrong = base.join(".hp41").join("autosave.json");
        assert_ne!(
            fallback, wrong,
            "mobile fallback must NOT use .hp41/ (container-root dot-dir)"
        );
    }

    /// PR #5 review (pr-test-analyzer) flagged that no test exercised the
    /// CLI→GUI interop path: a state file produced by hp41-cli v1.0 must
    /// load in hp41-gui v2.0 (shared ~/.hp41/autosave.json). All v1.1-
    /// introduced fields carry #[serde(default)] so missing fields must
    /// default to zero values without error.
    #[test]
    fn test_loads_v1_format_state_file() {
        use hp41_core::num::HpNum;

        let path = temp_path("v1_compat");
        let fresh = CalcState::new();
        save_state(&path, &fresh).unwrap();

        // Strip v1.1-introduced fields from the serialized JSON to simulate
        // a v1.0 save file. (print_buffer is #[serde(default, skip)] so it
        // never appears in JSON in the first place.)
        let raw = fs::read_to_string(&path).unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let state_obj = value["state"]
            .as_object_mut()
            .expect("state must be a JSON object");
        for key in ["last_key_code", "reg_m", "reg_n", "reg_o"] {
            state_obj.remove(key);
        }
        fs::write(&path, value.to_string()).unwrap();

        let loaded = load_state(&path).expect("v1.0-format save must load via serde(default)");
        assert_eq!(loaded.last_key_code, 0);
        assert_eq!(loaded.reg_m, HpNum::zero());
        assert_eq!(loaded.reg_n, HpNum::zero());
        assert_eq!(loaded.reg_o, HpNum::zero());
        assert!(loaded.print_buffer.is_empty());
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// Phase 54 PERSIST-03 (compat half): a committed v4.0-era fixture deserializes
    /// without error via load_state() + migrate_after_load().
    ///
    /// Verifies:
    /// - StateFile wrapper (`{"version":1,"state":{…}}`) is deserialized correctly.
    /// - is_running == false after load (Pitfall 4 / D-54.2b guard).
    /// - xrom_modules == 0b0001_1111 (all 5 XROM modules present and preserved).
    /// - xmem_files is non-empty (v4.0 X-MEM state survives round-trip, D-51.0a).
    /// - Two serde-exception fields (rand_seed non-zero, adv_tvm_state present) survive.
    ///
    /// The fixture at tests/fixtures/v40-autosave.json is committed source; serde path
    /// is byte-for-byte identical across all targets — a passing host/CI test proves
    /// iOS compatibility without device injection (D-54.4 / D-54.4b).
    #[test]
    fn test_loads_v40_autosave_fixture() {
        use hp41_core::num::HpNum;

        let fixture = include_str!("../tests/fixtures/v40-autosave.json");
        let path = temp_path("v40_compat");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, fixture.as_bytes()).unwrap();
        let loaded = load_state(&path).expect("v4.0-format save must load");
        assert!(
            !loaded.is_running,
            "is_running must be false after load (Pitfall 4)"
        );
        assert_eq!(
            loaded.xrom_modules, 0b0001_1111u8,
            "v4.0 xrom_modules (all 5 modules) must be preserved"
        );
        // v4.0 X-MEM state: at least one xmem_files entry (DATFILE)
        assert!(
            !loaded.xmem_files.is_empty(),
            "xmem_files must be non-empty in v4.0 fixture"
        );
        assert_eq!(
            loaded.xmem_active_file.as_deref(),
            Some("DATFILE"),
            "xmem_active_file must be 'DATFILE'"
        );
        // Two serde-exception fields (rand_seed, adv_tvm_state) survive round-trip (Pitfall 20).
        assert_ne!(
            loaded.rand_seed,
            HpNum::zero(),
            "rand_seed (serde-exception field) must survive round-trip"
        );
        assert!(
            loaded.adv_tvm_state.is_some(),
            "adv_tvm_state (serde-exception field) must survive round-trip"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// Phase 67-03 round-trip: persisted-trap recovery via soft_reset.
    ///
    /// Proves the reset escape-hatch contract:
    ///   1. A trapped CalcState (is_running + prgm_mode + display_override) is
    ///      saved to the autosave path and reloads as trapped (the bug premise).
    ///   2. Calling soft_reset() then re-saving to the SAME path overwrites the
    ///      autosave with a non-trapped state.
    ///   3. Reloading from that path yields an input-accepting state — recovery
    ///      survives a relaunch.
    ///
    /// This verifies the ordering invariant that the `reset_soft` command relies
    /// on: persist INSIDE the mutex lock so the auto-save thread cannot write
    /// back the pre-reset snapshot.
    #[test]
    fn test_reset_soft_overwrites_persisted_trap() {
        let path = temp_path("reset_soft_overwrite");

        // Step 1: build a trapped state and persist it.
        // Use fields that ARE serialized (no #[serde(skip)]).
        // display_override has #[serde(default, skip)] so it is not persisted — use
        // prgm_mode and user_mode as the persisted trapping indicators instead.
        let mut trapped = CalcState::new();
        trapped.prgm_mode = true;
        trapped.user_mode = true;
        save_state(&path, &trapped).unwrap();

        // Step 2: reload — must still be trapped (proves the bug premise: persisted
        // prgm_mode survives a round-trip and would block normal key dispatch).
        // Note: load_state() resets is_running=false per D-04 / Pitfall 4.
        let reloaded = load_state(&path).unwrap();
        assert!(
            reloaded.prgm_mode,
            "prgm_mode must survive reload (trap persists)"
        );
        assert!(reloaded.user_mode, "user_mode must survive reload");

        // Step 3: apply soft_reset(), re-save to the SAME path (overwrite), reload.
        // This is the exact sequence that the reset_soft Tauri command performs
        // while holding the AppState mutex (T-67-06 ordering invariant).
        trapped.soft_reset();
        save_state(&path, &trapped).unwrap();
        let recovered = load_state(&path).unwrap();

        assert!(!recovered.prgm_mode, "soft_reset must clear prgm_mode");
        assert!(!recovered.user_mode, "soft_reset must clear user_mode");
        assert!(!recovered.is_running, "soft_reset must clear is_running");

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// Phase 67-03 round-trip: persisted-state recovery via memory_lost (full reset).
    ///
    /// Proves that after `memory_lost()` + overwrite + reload, the reloaded state
    /// matches a freshly-constructed `CalcState::new()` for key fields.
    ///
    /// Complements `test_reset_soft_overwrites_persisted_trap` — covers the full-reset
    /// (MEMORY LOST) path with its own overwrite-survives-restart check.
    #[test]
    fn test_reset_full_overwrites_persisted_state() {
        let path = temp_path("reset_full_overwrite");

        // Build a non-trivial state with user data, persist it.
        // Use serialized fields only (display_override has #[serde(skip)]).
        let mut state = CalcState::new();
        state.prgm_mode = true;
        state.user_mode = true;
        state.key_assignments.insert('a', "MY_LABEL".to_string());
        save_state(&path, &state).unwrap();

        // Apply full reset, re-save, reload.
        state.memory_lost();
        save_state(&path, &state).unwrap();
        let recovered = load_state(&path).unwrap();

        // Factory state assertions — memory_lost() == CalcState::new()
        let factory = CalcState::new();
        assert!(!recovered.prgm_mode, "memory_lost must clear prgm_mode");
        assert!(!recovered.user_mode, "memory_lost must clear user_mode");
        assert!(
            recovered.key_assignments.is_empty(),
            "memory_lost must clear key_assignments"
        );
        assert_eq!(
            recovered.program, factory.program,
            "memory_lost must reset program to factory state"
        );
        assert_eq!(
            recovered.regs.len(),
            factory.regs.len(),
            "memory_lost must reset regs to factory length"
        );

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
