//! UI-neutral application services shared by the Tauri compatibility shell
//! and the native Swift bridge.

mod cards;
mod control;
mod dispatch;
mod files;
mod key_map;
mod persistence;
mod prgm_display;
mod request;
mod view;

pub use cards::{
    apply_card_read_result, cards_dir, cards_dir_required, drain_pending_card_op,
    execute_prepared_card_op, prepare_pending_card_op, sanitize_card_name, CardReadResult,
    PreparedCardOp,
};
pub use control::{
    bst_step, cancel_modal, resume_program, resume_program_with_key, run_program, run_stop,
    sst_step, submit_modal, submit_modal_with_label, tick_time,
};
pub use dispatch::{dispatch_key, DispatchError, DispatchOutcome};
pub use files::{
    export_data, export_data_file, export_raw, export_raw_file, import_data, import_data_file,
    import_raw, import_raw_file, inspect_raw, inspect_raw_file, FileTransferError, RawProgramInfo,
};
pub use key_map::{resolve, ResolveError};
pub use persistence::{
    full_reset, load_state, save_state, soft_reset, PersistenceError, StateFile, STATE_FILE_VERSION,
};
pub use prgm_display::{format_all_steps, format_step};
pub use request::{
    dispatch_request_key, execute_request, AppRequest, AppRequestOutcome, AppResponse, AppResult,
    AppStatus,
};
pub use view::{drain_state_view, Annunciators, CalcStateView, GuiError, YieldView};
