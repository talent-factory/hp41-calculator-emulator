use hp41_core::CalcState;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

pub const STATE_FILE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct StateFile {
    pub version: u32,
    pub state: CalcState,
}

impl StateFile {
    pub fn current(state: CalcState) -> Self {
        Self {
            version: STATE_FILE_VERSION,
            state,
        }
    }
}

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl Display for PersistenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, formatter),
            Self::Json(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn save_state(path: &Path, state: &CalcState) -> Result<(), PersistenceError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &StateFile::current(state.clone()))?;
    Ok(())
}

pub fn load_state(path: &Path) -> Result<CalcState, PersistenceError> {
    let file = fs::File::open(path)?;
    let wrapper: StateFile = serde_json::from_reader(file)?;
    let mut state = wrapper.state;
    state.is_running = false;
    state.migrate_after_load();
    Ok(state)
}

pub fn soft_reset(state: &mut CalcState) {
    state.soft_reset();
}

pub fn full_reset(state: &mut CalcState) {
    state.memory_lost();
}

#[cfg(test)]
mod tests {
    use super::*;
    use hp41_core::HpNum;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!(
                "hp41_app_persistence_{name}_{}",
                std::process::id()
            ))
            .join("autosave.json")
    }

    #[test]
    fn roundtrip_uses_version_wrapper_and_stops_running_state() {
        let path = temp_path("roundtrip");
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(42);
        state.is_running = true;
        save_state(&path, &state).unwrap();

        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"version\": 1"));
        let loaded = load_state(&path).unwrap();
        assert_eq!(loaded.stack.x, HpNum::from(42));
        assert!(!loaded.is_running);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn missing_and_corrupt_files_return_typed_errors() {
        let missing = temp_path("missing");
        let _ = fs::remove_dir_all(missing.parent().unwrap());
        assert!(matches!(load_state(&missing), Err(PersistenceError::Io(_))));

        let corrupt = temp_path("corrupt");
        fs::create_dir_all(corrupt.parent().unwrap()).unwrap();
        fs::write(&corrupt, "not json").unwrap();
        assert!(matches!(
            load_state(&corrupt),
            Err(PersistenceError::Json(_))
        ));
        let _ = fs::remove_dir_all(corrupt.parent().unwrap());
    }

    #[test]
    fn soft_reset_preserves_memory_while_full_reset_clears_it() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(9);
        state.regs[0] = hp41_core::HpValue::Numeric(HpNum::from(7));
        state.program.push(hp41_core::ops::Op::Enter);
        state.entry_buf = "123".to_string();
        state.user_mode = true;
        soft_reset(&mut state);
        assert!(state.stack.x.is_zero());
        assert_eq!(state.regs[0], hp41_core::HpValue::Numeric(HpNum::from(7)));
        assert_eq!(state.program, [hp41_core::ops::Op::Enter]);
        assert!(state.entry_buf.is_empty());
        assert!(!state.user_mode);

        full_reset(&mut state);
        assert!(state.stack.x.is_zero());
        assert!(state.program.is_empty());
        assert!(!state.user_mode);
        assert_eq!(state.regs.len(), CalcState::new().regs.len());
    }
}
