//! Shared card-reader filesystem workflow used by native and compatibility UIs.

use hp41_core::cardreader::{
    capture_data_card, decode_data, decode_program, encode_data, encode_program,
    insert_program_ops, load_data_card, CardOpRequest, DataCard,
};
use hp41_core::ops::Op;
use hp41_core::{CalcState, HpError};
use std::fs;
use std::path::{Path, PathBuf};

/// Default cards directory shared with the CLI. `HP41_CARDS_PATH` provides a
/// deterministic override for tests and sandboxed native distributions.
pub fn cards_dir() -> Option<PathBuf> {
    std::env::var_os("HP41_CARDS_PATH")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".hp41").join("cards")))
}

pub fn cards_dir_required() -> Result<PathBuf, HpError> {
    cards_dir()
        .ok_or_else(|| HpError::CardData("cannot resolve ~/.hp41/cards (no $HOME)".to_string()))
}

pub fn sanitize_card_name(name: &str) -> Result<&str, HpError> {
    if name.is_empty() {
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
        return Err(HpError::CardData(format!(
            "invalid card name {name:?}: leading '.' not allowed"
        )));
    }
    Ok(name)
}

pub enum CardReadResult {
    Program(Vec<Op>),
    Data(DataCard),
}

pub struct PreparedCardOp {
    request: CardOpRequest,
    write_payload: Option<Vec<u8>>,
}

pub fn prepare_pending_card_op(state: &mut CalcState) -> Result<Option<PreparedCardOp>, HpError> {
    let Some(request) = state.pending_card_op.take() else {
        return Ok(None);
    };
    let write_payload = match &request {
        CardOpRequest::WriteProgram { .. } => Some(
            encode_program(&state.program)
                .map_err(|error| HpError::CardData(format!("encode: {error}")))?,
        ),
        CardOpRequest::WriteData { .. } => Some(
            encode_data(&capture_data_card(state))
                .map_err(|error| HpError::CardData(format!("encode-json: {error}")))?,
        ),
        CardOpRequest::ReadProgram { .. } | CardOpRequest::ReadData { .. } => None,
    };
    Ok(Some(PreparedCardOp {
        request,
        write_payload,
    }))
}

pub fn execute_prepared_card_op(
    prepared: PreparedCardOp,
    directory: &Path,
) -> Result<Option<CardReadResult>, HpError> {
    fs::create_dir_all(directory).map_err(|error| {
        HpError::CardData(format!(
            "io: cannot create {}: {error}",
            directory.display()
        ))
    })?;
    let PreparedCardOp {
        request,
        write_payload,
    } = prepared;
    match request {
        CardOpRequest::WriteProgram { name } => {
            let path = directory
                .join(sanitize_card_name(&name)?)
                .with_extension("raw");
            let bytes = write_payload.expect("prepared program write must contain bytes");
            fs::write(&path, bytes).map_err(|error| {
                HpError::CardData(format!("io: write {}: {error}", path.display()))
            })?;
            Ok(None)
        }
        CardOpRequest::WriteData { name } => {
            let path = directory
                .join(sanitize_card_name(&name)?)
                .with_extension("card.json");
            let bytes = write_payload.expect("prepared data write must contain bytes");
            fs::write(&path, bytes).map_err(|error| {
                HpError::CardData(format!("io: write {}: {error}", path.display()))
            })?;
            Ok(None)
        }
        CardOpRequest::ReadProgram { name } => {
            let path = directory
                .join(sanitize_card_name(&name)?)
                .with_extension("raw");
            let bytes = fs::read(&path).map_err(|error| {
                HpError::CardData(format!("io: read {}: {error}", path.display()))
            })?;
            let operations = decode_program(&bytes)
                .map_err(|error| HpError::CardData(format!("decode: {error}")))?;
            Ok(Some(CardReadResult::Program(operations)))
        }
        CardOpRequest::ReadData { name } => {
            let path = directory
                .join(sanitize_card_name(&name)?)
                .with_extension("card.json");
            let bytes = fs::read(&path).map_err(|error| {
                HpError::CardData(format!("io: read {}: {error}", path.display()))
            })?;
            let card = decode_data(&bytes)
                .map_err(|error| HpError::CardData(format!("decode-json: {error}")))?;
            Ok(Some(CardReadResult::Data(card)))
        }
    }
}

pub fn apply_card_read_result(state: &mut CalcState, result: CardReadResult) {
    match result {
        CardReadResult::Program(operations) => insert_program_ops(state, operations),
        CardReadResult::Data(card) => load_data_card(state, card),
    }
}

pub fn drain_pending_card_op(state: &mut CalcState, directory: &Path) -> Result<(), HpError> {
    let Some(prepared) = prepare_pending_card_op(state)? else {
        return Ok(());
    };
    if let Some(result) = execute_prepared_card_op(prepared, directory)? {
        apply_card_read_result(state, result);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_names_are_rejected_with_specific_diagnostics() {
        for (name, diagnostic) in [
            ("../etc", "path separator"),
            ("a\\b", "path separator"),
            ("a\0b", "NUL byte"),
            (".hidden", "leading '.'"),
        ] {
            let error = sanitize_card_name(name).expect_err("unsafe name must fail");
            assert!(error.to_string().contains(diagnostic));
        }
        assert_eq!(sanitize_card_name("BACKUP-1"), Ok("BACKUP-1"));
        assert!(matches!(sanitize_card_name(""), Err(HpError::AlphaData)));
    }

    #[test]
    fn no_pending_request_does_not_create_directory() {
        let directory =
            std::env::temp_dir().join(format!("hp41-no-card-op-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        let mut state = CalcState::new();
        drain_pending_card_op(&mut state, &directory).expect("empty drain should succeed");
        assert!(!directory.exists());
    }

    #[test]
    fn program_and_data_cards_round_trip_through_directory() {
        let directory =
            std::env::temp_dir().join(format!("hp41-card-roundtrip-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        let mut state = CalcState::new();
        state.program = vec![Op::Add, Op::Sub];
        state.pending_card_op = Some(CardOpRequest::WriteProgram { name: "P".into() });
        drain_pending_card_op(&mut state, &directory).expect("program write should succeed");
        assert!(directory.join("P.raw").is_file());

        state.program.clear();
        state.pending_card_op = Some(CardOpRequest::ReadProgram { name: "P".into() });
        drain_pending_card_op(&mut state, &directory).expect("program read should succeed");
        assert_eq!(state.program, [Op::Add, Op::Sub]);

        state.pending_card_op = Some(CardOpRequest::WriteData { name: "D".into() });
        drain_pending_card_op(&mut state, &directory).expect("data write should succeed");
        assert!(directory.join("D.card.json").is_file());
        state.pending_card_op = Some(CardOpRequest::ReadData { name: "D".into() });
        drain_pending_card_op(&mut state, &directory).expect("data read should succeed");
        let _ = fs::remove_dir_all(directory);
    }
}
