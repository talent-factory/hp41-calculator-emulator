//! GUI card-IO integration tests — exercise the encode/decode path through the
//! GUI's drain helper and prove round-trip stability.
//! Spec: docs/superpowers/specs/2026-05-13-card-reader-manual-verification-design.md

#![allow(clippy::unwrap_used)]

use std::fs;

use hp41_core::ops::{dispatch, Op};
use hp41_core::run_program;
use hp41_core::state::CalcState;
use hp41_gui_lib::cards::drain_pending_card_op;

#[test]
fn gui_drain_writes_raw_file() {
    // Lower-bound test: prove the GUI's drain helper works in isolation.
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("X".to_string()), Op::Rtn];
    state.alpha_reg = "TESTGUI".to_string();

    dispatch(&mut state, Op::Xeq("WPRGM".to_string())).unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert!(tmp.path().join("TESTGUI.raw").exists());
}

#[test]
fn gui_program_round_trip_via_run_program() {
    // Validates the full save → clear → load → re-save → hash-stable
    // cycle through the GUI's drain helper. Mirror of the CLI test in
    // hp41-cli/tests/card_io_tests.rs.
    let tmp = tempfile::tempdir().unwrap();
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
    state.alpha_reg = "QUAD".to_string();
    let original_program = state.program.clone();

    // Save.
    run_program(&mut state, "WPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let raw_path = tmp.path().join("QUAD.raw");
    assert!(raw_path.exists());
    let bytes_a = fs::read(&raw_path).unwrap();

    // Clear.
    state.program.clear();

    // Load.
    state.alpha_reg = "QUAD".to_string();
    run_program(&mut state, "RDPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    assert_eq!(state.program, original_program);
    assert_eq!(state.pc, 0, "RDPRGM into empty program must reset pc to 0");

    // Re-save and assert byte stability.
    state.alpha_reg = "QUAD".to_string();
    run_program(&mut state, "WPRGM").unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let bytes_b = fs::read(&raw_path).unwrap();
    assert_eq!(
        bytes_a, bytes_b,
        "QUAD.raw must be byte-stable across save→load→save"
    );
}

#[test]
fn gui_wprgm_produces_non_trivial_raw_file() {
    // Smoke test for the encode path through the GUI drain: a small program
    // must produce more than a few bytes when encoded (END marker alone is
    // 3 bytes; a real program is more). True CLI/GUI byte identity is
    // verified by the manual procedure in docs/verifying-card-reader.md
    // (sha256 of the same card from both UIs).
    let tmp = tempfile::tempdir().unwrap();
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("SAME".to_string()), Op::Add, Op::Sub, Op::Rtn];
    state.alpha_reg = "SAME".to_string();

    dispatch(&mut state, Op::Xeq("WPRGM".to_string())).unwrap();
    drain_pending_card_op(&mut state, tmp.path()).unwrap();
    let bytes = fs::read(tmp.path().join("SAME.raw")).unwrap();
    assert!(
        bytes.len() > 5,
        "encoded program must contain at least the END marker bytes"
    );
}
