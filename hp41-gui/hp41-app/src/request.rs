//! Framework-independent typed application request boundary.

use crate::{
    bst_step, cancel_modal, dispatch_key, drain_pending_card_op, drain_state_view,
    export_data_file, export_raw_file, full_reset, import_data_file, import_raw_file,
    inspect_raw_file, resume_program, resume_program_with_key, run_program, run_stop, save_state,
    soft_reset, sst_step, submit_modal, submit_modal_with_label, tick_time, CalcStateView,
    RawProgramInfo,
};
use hp41_core::CalcState;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AppRequest {
    GetState,
    Dispatch { key_id: String },
    SstStep,
    BstStep,
    RunStop,
    RunProgram { label: String },
    ResumeProgram,
    ResumeProgramWithKey { keycode: u8 },
    RequestCancel,
    SubmitModal,
    CancelModal,
    SubmitModalWithLabel { label: String },
    TickTime,
    SaveState,
    ResetSoft,
    ResetFull,
    InspectRaw { path: String },
    ImportRaw { path: String, indices: Vec<usize> },
    ExportRaw { path: String },
    ImportData { path: String },
    ExportData { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppResult {
    RawPrograms {
        programs: Vec<RawProgramInfo>,
    },
    FileTransfer {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        programs: Option<Vec<RawProgramInfo>>,
    },
}

#[derive(Serialize)]
pub struct AppResponse {
    #[serde(flatten)]
    pub state: CalcStateView,
    pub status: AppStatus,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AppResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppStatus {
    Ok,
    Error,
}

impl AppResponse {
    pub fn drain(state: &mut CalcState, error: Option<String>, result: Option<AppResult>) -> Self {
        let status = if error.is_some() {
            AppStatus::Error
        } else {
            AppStatus::Ok
        };
        Self {
            state: drain_state_view(state),
            status,
            error,
            result,
        }
    }
}

#[derive(Debug, Default)]
pub struct AppRequestOutcome {
    pub error: Option<String>,
    pub result: Option<AppResult>,
}

/// Dispatch a native surface key, including the three control keys that are
/// commands rather than ordinary `Op` variants at the application boundary.
pub fn dispatch_request_key(state: &mut CalcState, key_id: &str) -> Result<(), String> {
    match key_id {
        "sst" => {
            sst_step(state);
            Ok(())
        }
        "bst" => {
            bst_step(state);
            Ok(())
        }
        "r_s" => {
            if state.prgm_mode || state.program.is_empty() {
                return Ok(());
            }
            if state.pc >= state.program.len() {
                state.pc = 0;
            }
            resume_program(state).map_err(|error| error.to_string())
        }
        _ => dispatch_key(state, key_id)
            .map(|_| ())
            .map_err(|error| error.to_string()),
    }
}

/// Execute one application request using adapter-provided persistence and card paths.
/// Platform file selection and preferences remain outside this boundary.
pub fn execute_request(
    state: &mut CalcState,
    request: AppRequest,
    state_path: &Path,
    cards_path: Option<&Path>,
) -> AppRequestOutcome {
    let should_save = !matches!(
        &request,
        AppRequest::GetState
            | AppRequest::RequestCancel
            | AppRequest::TickTime
            | AppRequest::InspectRaw { .. }
            | AppRequest::ExportRaw { .. }
            | AppRequest::ExportData { .. }
    );

    let result: Result<Option<AppResult>, String> = match request {
        AppRequest::GetState => Ok(None),
        AppRequest::Dispatch { key_id } => dispatch_request_key(state, &key_id).map(|_| None),
        AppRequest::SstStep => {
            sst_step(state);
            Ok(None)
        }
        AppRequest::BstStep => {
            bst_step(state);
            Ok(None)
        }
        AppRequest::RunStop => {
            run_stop(state);
            Ok(None)
        }
        AppRequest::RunProgram { label } => run_program(state, &label)
            .map(|_| None)
            .map_err(|error| error.to_string()),
        AppRequest::ResumeProgram => resume_program(state)
            .map(|_| None)
            .map_err(|error| error.to_string()),
        AppRequest::ResumeProgramWithKey { keycode } => resume_program_with_key(state, keycode)
            .map(|_| None)
            .map_err(|error| error.to_string()),
        AppRequest::RequestCancel => {
            state
                .cancel_requested
                .store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(None)
        }
        AppRequest::SubmitModal => submit_modal(state)
            .map(|_| None)
            .map_err(|error| error.to_string()),
        AppRequest::CancelModal => {
            cancel_modal(state);
            Ok(None)
        }
        AppRequest::SubmitModalWithLabel { label } => submit_modal_with_label(state, &label)
            .map(|_| None)
            .map_err(|error| error.to_string()),
        AppRequest::TickTime => {
            tick_time(state);
            Ok(None)
        }
        AppRequest::SaveState => Ok(None),
        AppRequest::ResetSoft => {
            soft_reset(state);
            Ok(None)
        }
        AppRequest::ResetFull => {
            full_reset(state);
            Ok(None)
        }
        AppRequest::InspectRaw { path } => inspect_raw_file(Path::new(&path))
            .map(|programs| Some(AppResult::RawPrograms { programs }))
            .map_err(|error| error.to_string()),
        AppRequest::ImportRaw { path, indices } => {
            import_raw_file(state, Path::new(&path), &indices)
                .map(|programs| {
                    Some(AppResult::FileTransfer {
                        path,
                        programs: Some(programs),
                    })
                })
                .map_err(|error| error.to_string())
        }
        AppRequest::ExportRaw { path } => export_raw_file(state, Path::new(&path))
            .map(|_| {
                Some(AppResult::FileTransfer {
                    path,
                    programs: None,
                })
            })
            .map_err(|error| error.to_string()),
        AppRequest::ImportData { path } => import_data_file(state, Path::new(&path))
            .map(|_| {
                Some(AppResult::FileTransfer {
                    path,
                    programs: None,
                })
            })
            .map_err(|error| error.to_string()),
        AppRequest::ExportData { path } => export_data_file(state, Path::new(&path))
            .map(|_| {
                Some(AppResult::FileTransfer {
                    path,
                    programs: None,
                })
            })
            .map_err(|error| error.to_string()),
    };

    let result = result.and_then(|value| match cards_path {
        Some(path) => drain_pending_card_op(state, path)
            .map(|_| value)
            .map_err(|error| error.to_string()),
        None if state.pending_card_op.is_some() => {
            state.pending_card_op = None;
            Err("card data: cannot resolve ~/.hp41/cards (no $HOME)".to_string())
        }
        None => Ok(value),
    });

    match result {
        Err(message) => AppRequestOutcome {
            error: Some(message),
            result: None,
        },
        Ok(result) if should_save => match save_state(state_path, state) {
            Ok(()) => AppRequestOutcome {
                error: None,
                result,
            },
            Err(error) => AppRequestOutcome {
                error: Some(error.to_string()),
                result: None,
            },
        },
        Ok(result) => AppRequestOutcome {
            error: None,
            result,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn request_schema_round_trips_and_executes_without_ui_framework() {
        let request = AppRequest::Dispatch {
            key_id: "fix_2".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(serde_json::from_str::<AppRequest>(&json).unwrap(), request);

        let root = std::env::temp_dir().join(format!("hp41-app-request-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut state = CalcState::new();
        for key_id in ["2", "chs", "abs", "fix_2"] {
            let outcome = execute_request(
                &mut state,
                AppRequest::Dispatch {
                    key_id: key_id.to_string(),
                },
                &root.join("state.json"),
                Some(&root.join("cards")),
            );
            assert_eq!(outcome.error, None);
        }
        assert_eq!(drain_state_view(&mut state).x_str, "2.00");
        let _ = fs::remove_dir_all(root);
    }
}
