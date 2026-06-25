//! Tauri compatibility re-exports for the shared card-reader workflow.

pub use hp41_app::{
    apply_card_read_result, cards_dir_required, drain_pending_card_op, execute_prepared_card_op,
    prepare_pending_card_op, CardReadResult, PreparedCardOp,
};
