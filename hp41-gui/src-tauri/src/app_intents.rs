//! Bridge between iOS App Intents and the running Tauri application.
//!
//! App Intents execute in Swift, outside Tauri's IPC surface. The Swift side
//! atomically writes one pending request into the app-local data directory;
//! the webview consumes it through `take_pending_app_intent` when iOS brings
//! the app to the foreground. Keeping this bridge file-based makes cold starts
//! reliable and avoids exposing Tauri's managed `AppState` through a C ABI.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

const MAILBOX_FILE: &str = "pending-app-intent.json";
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PendingAppIntent {
    ExecuteFunction { value: String },
    RunProgram { value: String },
}

/// Retains the live Tauri handle for the Swift-to-Rust notification callback.
/// A cold-start intent can arrive before setup; its durable mailbox is consumed
/// by the frontend's startup checks. Once setup completes, subsequent intents
/// also emit an event immediately, covering an already-running app.
pub fn install_app_handle(app: &AppHandle) {
    let _ = APP_HANDLE.set(app.clone());
}

/// Called by the shared Swift App Intent source after it atomically writes the
/// mailbox. This symbol is also the link anchor that keeps the Swift intent
/// object code in the macOS Rust executable.
#[no_mangle]
pub extern "C" fn hp41_app_intent_enqueued() {
    let Some(app) = APP_HANDLE.get() else {
        return;
    };

    #[cfg(desktop)]
    crate::focus_existing_instance(app);
    let _ = app.emit("app-intent-enqueued", ());
}

fn mailbox_path(app: &AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;

    app.path()
        .app_local_data_dir()
        .map(|dir| dir.join(MAILBOX_FILE))
        .map_err(|e| format!("unable to resolve App Intent mailbox: {e}"))
}

fn take_from_path(path: &Path) -> Result<Option<PendingAppIntent>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("unable to read App Intent mailbox: {e}")),
    };

    // The Swift writer uses an atomic replacement, so a successful read is a
    // complete request. Remove it before parsing to ensure a malformed request
    // cannot trap the app in an error loop on every foreground transition.
    fs::remove_file(path).map_err(|e| format!("unable to clear App Intent mailbox: {e}"))?;

    let intent: PendingAppIntent =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid App Intent request: {e}"))?;

    let value = match &intent {
        PendingAppIntent::ExecuteFunction { value } | PendingAppIntent::RunProgram { value } => {
            value
        }
    };
    if value.trim().is_empty() || value.len() > 128 {
        return Err("invalid App Intent value".to_string());
    }

    Ok(Some(intent))
}

#[tauri::command]
pub fn take_pending_app_intent(app: AppHandle) -> Result<Option<PendingAppIntent>, String> {
    take_from_path(&mailbox_path(&app)?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn takes_and_removes_valid_request() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(MAILBOX_FILE);
        fs::write(&path, br#"{"kind":"run_program","value":"SOLVE"}"#).unwrap();

        assert_eq!(
            take_from_path(&path).unwrap(),
            Some(PendingAppIntent::RunProgram {
                value: "SOLVE".to_string()
            })
        );
        assert!(!path.exists());
        assert_eq!(take_from_path(&path).unwrap(), None);
    }

    #[test]
    fn malformed_request_is_removed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(MAILBOX_FILE);
        fs::write(&path, b"not json").unwrap();

        assert!(take_from_path(&path).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn rejects_empty_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(MAILBOX_FILE);
        fs::write(&path, br#"{"kind":"execute_function","value":"   "}"#).unwrap();

        assert!(take_from_path(&path).is_err());
        assert!(!path.exists());
    }
}
