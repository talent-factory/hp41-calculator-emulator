use hp41_core::cardreader::{
    capture_data_card, decode_all_programs, decode_data, encode_data, encode_program,
    insert_program_ops, load_data_card, picker_label,
};
use hp41_core::{CalcState, HpError};
use serde::Serialize;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// Picker metadata for one program in a multi-program RAW archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct RawProgramInfo {
    pub label: String,
    pub index: usize,
    pub byte_len: usize,
    pub step_count: usize,
}

#[derive(Debug)]
pub enum FileTransferError {
    InvalidPath(String),
    Io(io::Error),
    Core(HpError),
    IndexOutOfRange { index: usize, count: usize },
}

impl fmt::Display for FileTransferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "io: {error}"),
            Self::Core(error) => error.fmt(formatter),
            Self::IndexOutOfRange { index, count } => {
                write!(
                    formatter,
                    "import index out of range: {index} (file has {count} programs)"
                )
            }
        }
    }
}

impl std::error::Error for FileTransferError {}

impl From<io::Error> for FileTransferError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<HpError> for FileTransferError {
    fn from(error: HpError) -> Self {
        Self::Core(error)
    }
}

pub fn inspect_raw(bytes: &[u8]) -> Result<Vec<RawProgramInfo>, FileTransferError> {
    let programs = decode_all_programs(bytes)?;
    Ok(programs
        .iter()
        .enumerate()
        .map(|(index, program)| RawProgramInfo {
            label: picker_label(index, &program.ops, program.byte_len),
            index,
            byte_len: program.byte_len,
            step_count: program.ops.len(),
        })
        .collect())
}

/// Imports the selected archive programs in order. All indices are validated
/// before the calculator is mutated, so a stale picker selection is atomic.
pub fn import_raw(
    state: &mut CalcState,
    bytes: &[u8],
    indices: &[usize],
) -> Result<Vec<RawProgramInfo>, FileTransferError> {
    let programs = decode_all_programs(bytes)?;
    if let Some(&index) = indices.iter().find(|&&index| index >= programs.len()) {
        return Err(FileTransferError::IndexOutOfRange {
            index,
            count: programs.len(),
        });
    }

    let imported = indices
        .iter()
        .map(|&index| {
            let program = &programs[index];
            RawProgramInfo {
                label: picker_label(index, &program.ops, program.byte_len),
                index,
                byte_len: program.byte_len,
                step_count: program.ops.len(),
            }
        })
        .collect::<Vec<_>>();
    for &index in indices {
        insert_program_ops(state, programs[index].ops.clone());
    }
    Ok(imported)
}

pub fn export_raw(state: &CalcState) -> Result<Vec<u8>, FileTransferError> {
    encode_program(&state.program).map_err(Into::into)
}

pub fn import_data(state: &mut CalcState, bytes: &[u8]) -> Result<(), FileTransferError> {
    load_data_card(state, decode_data(bytes)?);
    Ok(())
}

pub fn export_data(state: &CalcState) -> Result<Vec<u8>, FileTransferError> {
    encode_data(&capture_data_card(state)).map_err(Into::into)
}

pub fn inspect_raw_file(path: &Path) -> Result<Vec<RawProgramInfo>, FileTransferError> {
    inspect_raw(&read_regular_file(path)?)
}

pub fn import_raw_file(
    state: &mut CalcState,
    path: &Path,
    indices: &[usize],
) -> Result<Vec<RawProgramInfo>, FileTransferError> {
    import_raw(state, &read_regular_file(path)?, indices)
}

pub fn export_raw_file(state: &CalcState, path: &Path) -> Result<(), FileTransferError> {
    require_absolute(path)?;
    fs::write(path, export_raw(state)?)?;
    Ok(())
}

pub fn import_data_file(state: &mut CalcState, path: &Path) -> Result<(), FileTransferError> {
    import_data(state, &read_regular_file(path)?)
}

pub fn export_data_file(state: &CalcState, path: &Path) -> Result<(), FileTransferError> {
    require_absolute(path)?;
    fs::write(path, export_data(state)?)?;
    Ok(())
}

fn read_regular_file(path: &Path) -> Result<Vec<u8>, FileTransferError> {
    require_absolute(path)?;
    if !path.is_file() {
        return Err(FileTransferError::InvalidPath(format!(
            "not a regular file: {}",
            path.display()
        )));
    }
    fs::read(path).map_err(Into::into)
}

fn require_absolute(path: &Path) -> Result<(), FileTransferError> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(FileTransferError::InvalidPath(format!(
            "expected an absolute path, got: {}",
            path.display()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hp41_core::ops::Op;
    use hp41_core::{HpNum, HpValue};

    fn archive() -> Vec<u8> {
        let mut bytes =
            encode_program(&[Op::Lbl("A".into()), Op::Add]).expect("first program should encode");
        bytes.extend(
            encode_program(&[Op::Lbl("B".into()), Op::Sub]).expect("second program should encode"),
        );
        bytes
    }

    #[test]
    fn raw_inspection_describes_every_program_without_mutation() {
        let info = inspect_raw(&archive()).expect("archive should inspect");
        assert_eq!(info.len(), 2);
        assert!(info[0].label.starts_with("A ("));
        assert_eq!(info[1].index, 1);
        assert_eq!(info[1].step_count, 2);
    }

    #[test]
    fn invalid_selection_does_not_partially_import() {
        let mut state = CalcState::new();
        state.program = vec![Op::Add];
        let before = state.program.clone();
        let error =
            import_raw(&mut state, &archive(), &[0, 9]).expect_err("stale selection must fail");
        assert!(matches!(
            error,
            FileTransferError::IndexOutOfRange { index: 9, count: 2 }
        ));
        assert_eq!(state.program, before);
    }

    #[test]
    fn selected_programs_are_imported_in_requested_order() {
        let mut state = CalcState::new();
        let info = import_raw(&mut state, &archive(), &[1, 0]).expect("import should succeed");
        assert_eq!(
            info.iter().map(|item| item.index).collect::<Vec<_>>(),
            [1, 0]
        );
        assert!(matches!(&state.program[0], Op::Lbl(label) if label == "B"));
        assert!(state
            .program
            .iter()
            .any(|op| matches!(op, Op::Lbl(label) if label == "A")));
    }

    #[test]
    fn data_card_bytes_round_trip_registers() {
        let mut source = CalcState::new();
        source.regs[7] = HpValue::Numeric(HpNum::from(41));
        let bytes = export_data(&source).expect("data should encode");
        let mut target = CalcState::new();
        import_data(&mut target, &bytes).expect("data should decode");
        assert_eq!(target.regs[7], HpValue::Numeric(HpNum::from(41)));
    }
}
