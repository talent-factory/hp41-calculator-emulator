//! Card Reader frontend integration for hp41-gui.
//!
//! Owns:
//! - `cards_dir()` / `cards_dir_required()` — `~/.hp41/cards/` resolution.
//! - `sanitize_name()` — rejects path separators and dot-paths.
//! - `prepare_pending_card_op()` / `execute_prepared_card_op()` /
//!   `apply_card_read_result()` — three-phase drain that lets callers
//!   release shared state between phase 1 (in-lock prep + write-snapshot)
//!   and phase 3 (apply read result). The Tauri thunk in `commands.rs`
//!   uses the split so disk I/O does not hold the AppState mutex.
//! - `drain_pending_card_op()` — single-call convenience wrapper used by
//!   tests and by the single-threaded hp41-cli event loop (mirror).
//!
//! SC-4 invariant: this module calls only the public `hp41_core::cardreader::*`
//! API for encoding/decoding. No codec logic lives here.
//!
//! SYNC-NOTE: keep this file in step with `hp41-cli/src/cards.rs`.
//! Diff the two files after every CardOpRequest variant addition or
//! sanitize_name change — they share the same staging-drain contract
//! and must agree on behaviour.

use std::fs;
use std::path::{Path, PathBuf};

use hp41_core::cardreader::{
    capture_data_card, decode_data, decode_program, encode_data, encode_program,
    insert_program_ops, load_data_card, CardOpRequest, DataCard,
};
use hp41_core::error::HpError;
use hp41_core::ops::Op;
use hp41_core::state::CalcState;

/// Default cards directory: `~/.hp41/cards/`. Shared with hp41-cli.
///
/// Returns `None` if `dirs::home_dir()` is unavailable (rare; CI / containers
/// with no $HOME). Callers should treat that as a fatal startup error since
/// any card op would fail.
pub fn cards_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".hp41").join("cards"))
}

/// Resolve `~/.hp41/cards/` or fail with a `CardData` error that names the
/// real cause. Use this instead of `cards_dir()` whenever a missing $HOME
/// must NOT be silently downgraded to a no-op write.
pub fn cards_dir_required() -> Result<PathBuf, HpError> {
    cards_dir()
        .ok_or_else(|| HpError::CardData("cannot resolve ~/.hp41/cards (no $HOME)".to_string()))
}

/// Reject card names that would escape `cards_dir` or otherwise be unsafe.
///
/// Sanitisation, not normalisation: surface `HpError::CardData` rather than
/// silently mangling the user's input. The diagnostic names the offending
/// rule so the user can correct it without guessing.
pub fn sanitize_name(name: &str) -> Result<&str, HpError> {
    if name.is_empty() {
        // Defensive: op handlers should have already returned AlphaData,
        // but guard anyway so this helper is safe to call standalone.
        return Err(HpError::AlphaData);
    }
    if name.contains('/') || name.contains('\\') {
        return Err(HpError::CardData(format!(
            "invalid card name {name:?}: contains path separator"
        )));
    }
    if name.contains('\0') {
        return Err(HpError::CardData(format!(
            "invalid card name {name:?}: contains NUL byte"
        )));
    }
    if name.starts_with('.') {
        // Rejects ".", "..", and any hidden-file-style name like ".hidden".
        return Err(HpError::CardData(format!(
            "invalid card name {name:?}: leading '.' not allowed"
        )));
    }
    Ok(name)
}

/// Result of the second phase for a read op. Write ops produce `None`.
pub enum CardReadResult {
    Program(Vec<Op>),
    Data(DataCard),
}

/// Phase-1 output: everything needed to perform the disk I/O without
/// further access to `CalcState`. Constructed under the caller's lock and
/// then carried across a lock drop into [`execute_prepared_card_op`].
pub struct PreparedCardOp {
    req: CardOpRequest,
    /// `Some(bytes)` for the two write variants, `None` for the two read variants.
    write_payload: Option<Vec<u8>>,
}

/// Phase 1: take `state.pending_card_op` out and, for write ops, encode
/// the outgoing bytes against the current state. Returns `Ok(None)` when
/// no request is pending. After this returns `Ok(Some(_))`, the caller is
/// free to drop any state lock for the duration of the disk I/O.
pub fn prepare_pending_card_op(state: &mut CalcState) -> Result<Option<PreparedCardOp>, HpError> {
    let Some(req) = state.pending_card_op.take() else {
        return Ok(None);
    };
    let write_payload = match &req {
        CardOpRequest::WriteProgram { .. } => Some(
            encode_program(&state.program)
                .map_err(|e| HpError::CardData(format!("encode: {e}")))?,
        ),
        CardOpRequest::WriteData { .. } => {
            let card = capture_data_card(state);
            Some(encode_data(&card).map_err(|e| HpError::CardData(format!("encode-json: {e}")))?)
        }
        CardOpRequest::ReadProgram { .. } | CardOpRequest::ReadData { .. } => None,
    };
    Ok(Some(PreparedCardOp { req, write_payload }))
}

/// Phase 2: perform the actual filesystem I/O. Touches no `CalcState`.
/// Safe to call without holding any state lock — that's the whole point
/// of the three-phase split.
pub fn execute_prepared_card_op(
    prepared: PreparedCardOp,
    cards_dir: &Path,
) -> Result<Option<CardReadResult>, HpError> {
    fs::create_dir_all(cards_dir).map_err(|e| {
        HpError::CardData(format!("io: cannot create {}: {e}", cards_dir.display()))
    })?;
    let PreparedCardOp { req, write_payload } = prepared;
    match req {
        CardOpRequest::WriteProgram { name } => {
            let safe = sanitize_name(&name)?;
            let bytes =
                write_payload.expect("WriteProgram must carry encoded bytes from prepare phase");
            let path = cards_dir.join(safe).with_extension("raw");
            fs::write(&path, &bytes)
                .map_err(|e| HpError::CardData(format!("io: write {}: {e}", path.display())))?;
            Ok(None)
        }
        CardOpRequest::WriteData { name } => {
            let safe = sanitize_name(&name)?;
            let bytes =
                write_payload.expect("WriteData must carry encoded bytes from prepare phase");
            let path = cards_dir.join(safe).with_extension("card.json");
            fs::write(&path, &bytes)
                .map_err(|e| HpError::CardData(format!("io: write {}: {e}", path.display())))?;
            Ok(None)
        }
        CardOpRequest::ReadProgram { name } => {
            let safe = sanitize_name(&name)?;
            let path = cards_dir.join(safe).with_extension("raw");
            let bytes = fs::read(&path)
                .map_err(|e| HpError::CardData(format!("io: read {}: {e}", path.display())))?;
            let ops =
                decode_program(&bytes).map_err(|e| HpError::CardData(format!("decode: {e}")))?;
            Ok(Some(CardReadResult::Program(ops)))
        }
        CardOpRequest::ReadData { name } => {
            let safe = sanitize_name(&name)?;
            let path = cards_dir.join(safe).with_extension("card.json");
            let bytes = fs::read(&path)
                .map_err(|e| HpError::CardData(format!("io: read {}: {e}", path.display())))?;
            let card =
                decode_data(&bytes).map_err(|e| HpError::CardData(format!("decode-json: {e}")))?;
            Ok(Some(CardReadResult::Data(card)))
        }
    }
}

/// Phase 3: apply the read result back to `CalcState`. No-op for write ops
/// (where the caller will have received `Ok(None)` from phase 2).
pub fn apply_card_read_result(state: &mut CalcState, result: CardReadResult) {
    match result {
        CardReadResult::Program(ops) => insert_program_ops(state, ops),
        CardReadResult::Data(card) => load_data_card(state, card),
    }
}

/// Single-call convenience wrapper that composes all three phases under the
/// caller's exclusive access to `state`. Used by tests and the
/// single-threaded hp41-cli event loop (mirror). The Tauri thunk uses the
/// phases separately so it can release the AppState mutex around the I/O step.
pub fn drain_pending_card_op(state: &mut CalcState, cards_dir: &Path) -> Result<(), HpError> {
    let Some(prepared) = prepare_pending_card_op(state)? else {
        return Ok(());
    };
    if let Some(result) = execute_prepared_card_op(prepared, cards_dir)? {
        apply_card_read_result(state, result);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_rejects_path_separators() {
        assert!(matches!(sanitize_name("../etc"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("a/b"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("a\\b"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("a\0b"), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name("."), Err(HpError::CardData(_))));
        assert!(matches!(sanitize_name(".."), Err(HpError::CardData(_))));
        assert!(matches!(
            sanitize_name(".hidden"),
            Err(HpError::CardData(_))
        ));
        assert_eq!(sanitize_name("QUAD"), Ok("QUAD"));
        assert_eq!(sanitize_name("BACKUP-1"), Ok("BACKUP-1"));
    }

    #[test]
    fn sanitize_diagnostic_names_offending_rule() {
        // Each rule must surface its identifier so the user can correct the name
        // without guessing which check fired.
        let assert_msg_contains = |name: &str, needle: &str| match sanitize_name(name) {
            Err(HpError::CardData(msg)) => assert!(
                msg.contains(needle),
                "expected {needle:?} in error for {name:?}, got: {msg}"
            ),
            other => panic!("expected CardData for {name:?}, got: {other:?}"),
        };
        assert_msg_contains("a/b", "path separator");
        assert_msg_contains("a\\b", "path separator");
        assert_msg_contains("a\0b", "NUL byte");
        assert_msg_contains(".hidden", "leading '.'");
    }

    #[test]
    fn sanitize_empty_yields_alpha_data() {
        assert!(matches!(sanitize_name(""), Err(HpError::AlphaData)));
    }

    #[test]
    fn drain_with_no_request_is_noop() {
        let mut state = CalcState::new();
        let tmp = tempfile::tempdir().unwrap();
        // Pass a subdirectory that does NOT yet exist. The early-return path
        // must skip create_dir_all entirely; otherwise this assertion fails.
        let subdir = tmp.path().join("never_should_be_created");
        assert!(state.pending_card_op.is_none());
        drain_pending_card_op(&mut state, &subdir).unwrap();
        assert!(state.pending_card_op.is_none());
        assert!(
            !subdir.exists(),
            "no-op drain must not call create_dir_all on cards_dir",
        );
    }

    #[test]
    fn prepare_with_no_request_returns_none() {
        let mut state = CalcState::new();
        let prepared = prepare_pending_card_op(&mut state).unwrap();
        assert!(prepared.is_none());
    }

    #[test]
    fn prepare_write_program_captures_bytes_and_clears_request() {
        let mut state = CalcState::new();
        state.program = vec![Op::Add, Op::Sub];
        state.pending_card_op = Some(CardOpRequest::WriteProgram {
            name: "X".to_string(),
        });
        let prepared = prepare_pending_card_op(&mut state).unwrap().unwrap();
        assert!(
            state.pending_card_op.is_none(),
            "prepare must take the request out so a follow-up dispatch does not see it"
        );
        assert!(
            prepared.write_payload.is_some(),
            "WriteProgram must carry encoded bytes into phase 2"
        );
    }

    #[test]
    fn prepare_read_program_takes_request_with_no_payload() {
        let mut state = CalcState::new();
        state.pending_card_op = Some(CardOpRequest::ReadProgram {
            name: "X".to_string(),
        });
        let prepared = prepare_pending_card_op(&mut state).unwrap().unwrap();
        assert!(state.pending_card_op.is_none());
        assert!(
            prepared.write_payload.is_none(),
            "ReadProgram has no write payload"
        );
    }
}
