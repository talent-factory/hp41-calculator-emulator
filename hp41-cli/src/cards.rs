//! Card Reader frontend integration for hp41-cli.
//!
//! Owns:
//! - `cards_dir()` — `~/.hp41/cards/` resolution.
//! - `sanitize_name()` — rejects path separators and dot-paths.
//! - `drain_pending_card_op()` — performs the staged disk I/O after dispatch.
//!
//! SC-4 invariant: this module calls only the public `hp41_core::cardreader::*`
//! API for encoding/decoding. No codec logic lives here.
//!
//! SYNC-NOTE: keep this file in step with `hp41-gui/src/cards.rs`.
//! Diff the two files after every CardOpRequest variant addition or
//! sanitize_name change — they share the same staging-drain contract
//! and must agree on behaviour.

use std::fs;
use std::path::{Path, PathBuf};

use hp41_core::cardreader::{
    capture_data_card, decode_data, decode_program, encode_data, encode_program,
    insert_program_ops, load_data_card, CardOpRequest,
};
use hp41_core::error::HpError;
use hp41_core::state::CalcState;

/// Default cards directory: `~/.hp41/cards/`. Shared with hp41-gui.
///
/// Returns `None` if `dirs::home_dir()` is unavailable (rare; CI / containers
/// with no $HOME). Callers should treat that as a fatal startup error since
/// any card op would fail.
pub fn cards_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".hp41").join("cards"))
}

/// Reject card names that would escape `cards_dir` or otherwise be unsafe.
///
/// Sanitisation, not normalisation: surface `HpError::CardData` rather than
/// silently mangling the user's input.
pub fn sanitize_name(name: &str) -> Result<&str, HpError> {
    if name.is_empty() {
        // Defensive: op handlers should have already returned AlphaData,
        // but guard anyway so this helper is safe to call standalone.
        return Err(HpError::AlphaData);
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err(HpError::CardData(format!("invalid card name: {name:?}")));
    }
    if name.starts_with('.') {
        // Rejects ".", "..", and any hidden-file-style name like ".hidden".
        return Err(HpError::CardData(format!("invalid card name: {name:?}")));
    }
    Ok(name)
}

/// Drain `state.pending_card_op` and perform the staged disk I/O against `cards_dir`.
///
/// No-op if no request is pending. Errors are surfaced as `HpError::CardData(msg)`
/// so the CLI display can show "CARD DATA" with a useful suffix.
///
/// `cards_dir` is a parameter rather than computed inside — keeps integration
/// tests sandboxable via `tempfile::tempdir()`.
pub fn drain_pending_card_op(state: &mut CalcState, cards_dir: &Path) -> Result<(), HpError> {
    let Some(req) = state.pending_card_op.take() else {
        return Ok(());
    };

    fs::create_dir_all(cards_dir).map_err(|e| {
        HpError::CardData(format!("io: cannot create {}: {e}", cards_dir.display()))
    })?;

    match req {
        CardOpRequest::WriteProgram { name } => {
            let safe = sanitize_name(&name)?;
            let bytes = encode_program(&state.program)
                .map_err(|e| HpError::CardData(format!("encode: {e}")))?;
            let path = cards_dir.join(safe).with_extension("raw");
            fs::write(&path, &bytes)
                .map_err(|e| HpError::CardData(format!("io: write {}: {e}", path.display())))?;
            Ok(())
        }
        CardOpRequest::WriteData { name } => {
            let safe = sanitize_name(&name)?;
            let card = capture_data_card(state);
            let bytes = encode_data(&card)
                .map_err(|e| HpError::CardData(format!("encode-json: {e}")))?;
            let path = cards_dir.join(safe).with_extension("card.json");
            fs::write(&path, &bytes)
                .map_err(|e| HpError::CardData(format!("io: write {}: {e}", path.display())))?;
            Ok(())
        }
        CardOpRequest::ReadProgram { name } => {
            let safe = sanitize_name(&name)?;
            let path = cards_dir.join(safe).with_extension("raw");
            let bytes = fs::read(&path)
                .map_err(|e| HpError::CardData(format!("io: read {}: {e}", path.display())))?;
            let ops = decode_program(&bytes)
                .map_err(|e| HpError::CardData(format!("decode: {e}")))?;
            insert_program_ops(state, ops);
            Ok(())
        }
        CardOpRequest::ReadData { name } => {
            let safe = sanitize_name(&name)?;
            let path = cards_dir.join(safe).with_extension("card.json");
            let bytes = fs::read(&path)
                .map_err(|e| HpError::CardData(format!("io: read {}: {e}", path.display())))?;
            let card = decode_data(&bytes)
                .map_err(|e| HpError::CardData(format!("decode-json: {e}")))?;
            load_data_card(state, card);
            Ok(())
        }
    }
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
        assert!(matches!(sanitize_name(".hidden"), Err(HpError::CardData(_))));
        assert_eq!(sanitize_name("QUAD"), Ok("QUAD"));
        assert_eq!(sanitize_name("BACKUP-1"), Ok("BACKUP-1"));
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
}
