//! End-to-end Card Reader integration tests for hp41-cli.
//! Spec: docs/superpowers/specs/2026-05-13-card-reader-manual-verification-design.md

#![allow(clippy::unwrap_used)]

use std::fs;

use hp41_cli::cards::drain_pending_card_op;
use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::{dispatch, Op};
use hp41_core::run_program;
use hp41_core::state::CalcState;
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// A small program whose ops are all in the `.raw` codec's supported subset.
/// Mirrors the pattern from hp41-core/tests/cardreader_tests.rs.
fn make_state_with_simple_program() -> CalcState {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("QUAD".to_string()),
        Op::Sq,
        Op::Add,
        Op::Sqrt,
        Op::StoReg(1),
        Op::RclReg(1),
        Op::Rtn,
    ];
    state
}

#[test]
fn roundtrip_program_via_tempdir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = make_state_with_simple_program();
    state.alpha_reg = "QUAD".to_string();
    let original_program = state.program.clone();

    // 1. Stage WPRGM via the "WPRGM" builtin fallback in run_program, then drain.
    run_program(&mut state, "WPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let raw_path = tmp.path().join("QUAD.raw");
    assert!(raw_path.exists(), "QUAD.raw must exist after WPRGM + drain");
    let hash_a = sha256_hex(&fs::read(&raw_path).unwrap());

    // 2. Clear the program.
    state.program.clear();
    state.pc = 0;

    // 3. Stage RDPRGM + drain — should restore the program.
    state.alpha_reg = "QUAD".to_string();
    run_program(&mut state, "RDPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert_eq!(
        state.program, original_program,
        "program after RDPRGM + drain must equal the original"
    );
    assert_eq!(
        state.pc, 0,
        "RDPRGM into an empty program must reset pc to 0 (cardreader::insert_program_ops contract)"
    );

    // 4. Re-save and compare hashes — encoding must be byte-stable.
    state.alpha_reg = "QUAD".to_string();
    run_program(&mut state, "WPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let hash_b = sha256_hex(&fs::read(&raw_path).unwrap());
    assert_eq!(
        hash_a, hash_b,
        "SHA-256 of QUAD.raw must be byte-stable across save → load → re-save"
    );
}

#[test]
fn roundtrip_data_via_tempdir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    state.regs[0] = HpNum::from(42i32);
    state.regs[50] = HpNum::from(314i32);
    state.regs[99] = HpNum::from(-1i32);

    // Save.
    state.alpha_reg = "BACKUP".to_string();
    run_program(&mut state, "WDTA").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let path = tmp.path().join("BACKUP.card.json");
    assert!(
        path.exists(),
        "BACKUP.card.json must exist after WDTA + drain"
    );
    let hash_c = sha256_hex(&fs::read(&path).unwrap());

    // Clear registers.
    for r in &mut state.regs {
        *r = HpNum::zero();
    }
    assert_eq!(
        state.regs[0],
        HpNum::zero(),
        "registers must be zeroed before reload"
    );

    // Load.
    state.alpha_reg = "BACKUP".to_string();
    run_program(&mut state, "RDTA").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert_eq!(state.regs[0], HpNum::from(42i32), "R00 must round-trip");
    assert_eq!(state.regs[50], HpNum::from(314i32), "R50 must round-trip");
    assert_eq!(state.regs[99], HpNum::from(-1i32), "R99 must round-trip");
    assert!(
        state.regs.len() >= 100,
        "load_data_card must keep register count >= 100"
    );

    // Re-save → hash stability.
    state.alpha_reg = "BACKUP".to_string();
    run_program(&mut state, "WDTA").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let hash_d = sha256_hex(&fs::read(&path).unwrap());
    assert_eq!(
        hash_c, hash_d,
        "SHA-256 of BACKUP.card.json must be byte-stable across save → load → re-save"
    );
}

#[test]
fn empty_alpha_yields_alpha_data_error() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    // alpha_reg defaults to "" — empty.
    let err = run_program(&mut state, "WPRGM").unwrap_err();
    assert!(
        matches!(err, HpError::AlphaData),
        "empty ALPHA + WPRGM must surface AlphaData, got {err:?}",
    );
    // Nothing staged → drain is a no-op.
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
}

#[test]
fn missing_file_yields_card_data_error() {
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    state.alpha_reg = "NOPE".to_string();
    // Stage the read; the file does not exist on disk.
    run_program(&mut state, "RDPRGM").unwrap();
    let err = drain_pending_card_op(&mut state, tmp.path()).unwrap_err();
    assert!(
        matches!(err, HpError::CardData(_)),
        "missing .raw file must yield CardData, got {err:?}",
    );
}

#[test]
fn corrupt_data_json_yields_card_data_error() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("BAD.card.json"), b"this is not json").unwrap();
    let mut state = CalcState::new();
    state.alpha_reg = "BAD".to_string();
    run_program(&mut state, "RDTA").unwrap();
    let err = drain_pending_card_op(&mut state, tmp.path()).unwrap_err();
    assert!(
        matches!(err, HpError::CardData(_)),
        "corrupt .card.json must yield CardData, got {err:?}",
    );
}

#[test]
fn dispatch_op_xeq_then_drain_works_for_gui_path() {
    // Mirrors the GUI path: dispatch(Op::Xeq("WPRGM")) instead of run_program.
    let tmp = tempfile::tempdir().unwrap();
    let mut state = make_state_with_simple_program();
    state.alpha_reg = "FROMGUI".to_string();
    dispatch(&mut state, Op::Xeq("WPRGM".to_string())).unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert!(
        tmp.path().join("FROMGUI.raw").exists(),
        "FROMGUI.raw must exist after dispatch(Op::Xeq(\"WPRGM\")) + drain"
    );
}

/// Reviewer-flagged coverage gap: the only existing end-to-end RDPRGM test
/// reads into a deliberately-cleared program. The `insert_program_ops`
/// helper has a second branch — insert-after-pc when the program is
/// non-empty — that was unit-tested in `hp41-core` but never integration-
/// tested through the CLI's drain path. A regression that swapped the two
/// branches, or that reset `pc` to 0 unconditionally, would slip past CI.
#[test]
fn rdprgm_into_non_empty_program_inserts_and_preserves_pc() {
    let tmp = tempfile::tempdir().unwrap();

    // Stage 1: write a card whose contents differ from what we'll load it into.
    let mut writer = CalcState::new();
    writer.program = vec![Op::Lbl("ADDED".to_string()), Op::Add, Op::Rtn];
    writer.alpha_reg = "INSERTME".to_string();
    run_program(&mut writer, "WPRGM").unwrap();
    drain_pending_card_op(&mut writer, tmp.path()).unwrap();

    // Stage 2: read it back into a state that already has a different program
    // and a non-zero pc.
    let mut reader = CalcState::new();
    reader.program = vec![
        Op::Lbl("HOST".to_string()),
        Op::Sub,
        Op::Mul,
        Op::Div,
        Op::Rtn,
    ];
    reader.pc = 1; // mid-program, NOT at start
    let len_before = reader.program.len();

    reader.alpha_reg = "INSERTME".to_string();
    run_program(&mut reader, "RDPRGM").unwrap();
    drain_pending_card_op(&mut reader, tmp.path()).unwrap();

    assert!(
        reader.program.len() > len_before,
        "non-empty RDPRGM must extend the program, not replace it; \
         was {len_before} steps, became {}",
        reader.program.len()
    );
    assert!(
        reader
            .program
            .iter()
            .any(|op| matches!(op, Op::Lbl(s) if s == "HOST")),
        "host LBL must still be present after the insertion",
    );
    assert!(
        reader
            .program
            .iter()
            .any(|op| matches!(op, Op::Lbl(s) if s == "ADDED")),
        "card LBL must be inserted into the host program",
    );
    assert_ne!(
        reader.pc, 0,
        "RDPRGM into a non-empty program must NOT reset pc to 0 — \
         the insert-after-pc branch must preserve the user's position",
    );
}
