//! Integration tests for the Card Reader.
//!
//! Drives the full pipeline a frontend would execute:
//!   1. `dispatch(Op::Wdta)` stages a `pending_card_op`
//!   2. The frontend would drain it, encode via `encode_data`, write to disk
//!   3. On read, decode via `decode_data` and load via `load_data_card`
//!
//! These tests skip the disk hop and chain the in-memory steps directly.

use hp41_core::cardreader::{
    capture_data_card, decode_data, decode_program, encode_data, encode_program,
    insert_program_ops, load_data_card, CardOpRequest,
};
use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::program::run_program;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;

#[test]
fn wdta_full_pipeline_round_trips_data_registers() {
    // Setup: populate some registers, then "write" via WDTA.
    let mut state = CalcState::new();
    state.regs[0] = HpNum::from(42i32);
    state.regs[5] = HpNum::from(-3i32);
    state.regs[99] = HpNum::from(123i32);
    state.alpha_reg = "MYDATA".to_string();

    // Step 1: dispatch WDTA — must stage a WriteData request.
    dispatch(&mut state, Op::Wdta).expect("WDTA must succeed when ALPHA is set");
    let request = state
        .pending_card_op
        .take()
        .expect("pending_card_op must be set");
    assert_eq!(
        request,
        CardOpRequest::WriteData {
            name: "MYDATA".to_string()
        }
    );

    // Step 2: frontend captures and encodes (simulated in-memory).
    let card = capture_data_card(&state);
    let bytes = encode_data(&card).expect("encode_data must succeed");

    // Step 3: a fresh calculator reads the bytes back.
    let card_back = decode_data(&bytes).expect("decode_data must succeed");
    let mut other = CalcState::new();
    load_data_card(&mut other, card_back);

    // Verify registers round-tripped exactly.
    assert_eq!(other.regs[0], HpNum::from(42i32));
    assert_eq!(other.regs[5], HpNum::from(-3i32));
    assert_eq!(other.regs[99], HpNum::from(123i32));
    assert_eq!(other.regs.len(), 100);
}

#[test]
fn wprgm_full_pipeline_round_trips_program() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("QUAD".to_string()),
        Op::Sq,
        Op::Add,
        Op::Sqrt,
        Op::Rtn,
    ];
    state.alpha_reg = "PROG".to_string();

    // Stage the write.
    dispatch(&mut state, Op::Wprgm).expect("WPRGM must succeed");
    let request = state
        .pending_card_op
        .take()
        .expect("pending_card_op must be set");
    assert_eq!(
        request,
        CardOpRequest::WriteProgram {
            name: "PROG".to_string()
        }
    );

    // Simulate the frontend: encode → bytes.
    let bytes = encode_program(&state.program).expect("encode_program must succeed");

    // END marker is always appended on encode.
    assert_eq!(&bytes[bytes.len() - 3..], &[0xC0, 0x00, 0x0D]);

    // Read back into a fresh calculator: empty program → replace.
    let ops_back = decode_program(&bytes).expect("decode_program must succeed");
    let mut other = CalcState::new();
    insert_program_ops(&mut other, ops_back);

    assert_eq!(other.program, state.program);
    assert_eq!(other.pc, 0, "RDPRGM into empty program must reset pc");
}

#[test]
fn rdprgm_into_nonempty_program_inserts_after_pc() {
    // Set up a calculator with a small program and pc mid-stream.
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("A".into()), Op::Add, Op::Rtn];
    state.pc = 1; // currently at Add

    // Simulate reading a card containing [Mul, Sub].
    let card_bytes = encode_program(&[Op::Mul, Op::Sub]).expect("encode_program");
    let ops = decode_program(&card_bytes).expect("decode_program");
    insert_program_ops(&mut state, ops);

    // Expected merged program: LBL A, Add, Mul, Sub, Rtn.
    assert_eq!(
        state.program,
        vec![Op::Lbl("A".into()), Op::Add, Op::Mul, Op::Sub, Op::Rtn]
    );
}

#[test]
fn wdta_with_empty_alpha_is_alpha_data_error() {
    let mut state = CalcState::new();
    // alpha_reg is empty by default.
    let err = dispatch(&mut state, Op::Wdta).unwrap_err();
    assert_eq!(err, HpError::AlphaData);
    assert!(state.pending_card_op.is_none());
}

#[test]
fn encode_program_rejects_unsupported_op() {
    // FmtFix is outside the encoding subset — encode_program rejects it.
    let err = encode_program(&[Op::FmtFix(4)]).unwrap_err();
    assert!(matches!(err, HpError::CardData(_)));
}

#[test]
fn data_card_format_tag_is_stable() {
    // Future tooling will identify our .card.json files by this exact tag.
    let mut state = CalcState::new();
    state.regs[0] = HpNum::from(7i32);
    let card = capture_data_card(&state);
    let bytes = encode_data(&card).unwrap();
    let json = std::str::from_utf8(&bytes).unwrap();
    assert!(
        json.contains("\"format\""),
        "encoded data card must carry a format field"
    );
    assert!(
        json.contains("hp41-data-v1"),
        "encoded data card must carry the hp41-data-v1 magic tag"
    );
}

#[test]
fn pending_card_op_is_not_serialized_to_save_state() {
    // pending_card_op carries `#[serde(skip)]` so that mid-card-operation state
    // never persists across autosave/load (transient, like print_buffer).
    let mut state = CalcState::new();
    state.alpha_reg = "X".into();
    dispatch(&mut state, Op::Wdta).unwrap();
    assert!(state.pending_card_op.is_some());

    let json = serde_json::to_string(&state).unwrap();
    assert!(
        !json.contains("pending_card_op"),
        "pending_card_op must be skipped from serialization"
    );
}

#[test]
fn card_op_recorded_in_program_stages_request_via_run_program() {
    // The four card ops route through execute_op() in the program interpreter,
    // not the interactive dispatch path. Build a program that contains
    // Op::Wdta, run it via run_program(), and verify the request is staged
    // and lift behaviour stayed Neutral.
    let mut state = CalcState::new();
    state.alpha_reg = "BACKUP".into();
    state.program = vec![Op::Lbl("A".into()), Op::Wdta, Op::Rtn];
    state.stack.lift_enabled = false;

    run_program(&mut state, "A").expect("program with WDTA must run cleanly");
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteData {
            name: "BACKUP".into()
        })
    );
    assert!(
        !state.stack.lift_enabled,
        "Neutral lift effect must survive the run_program path too"
    );
}

#[test]
fn back_to_back_card_ops_in_program_surface_card_data_error() {
    // Two card ops in a row, with no chance for the frontend to drain in
    // between, must NOT silently overwrite. The second op surfaces a CardData
    // error so the program halts and the user sees that the first request
    // never completed.
    let mut state = CalcState::new();
    state.alpha_reg = "A".into();
    state.program = vec![Op::Lbl("A".into()), Op::Wdta, Op::Wprgm, Op::Rtn];

    let err = run_program(&mut state, "A").unwrap_err();
    assert!(
        matches!(&err, HpError::CardData(msg) if msg.contains("pending")),
        "expected pending-overwrite diagnostic, got: {err:?}"
    );
    // First request must still be intact.
    assert_eq!(
        state.pending_card_op,
        Some(CardOpRequest::WriteData { name: "A".into() })
    );
}

#[test]
fn unsupported_version_surfaces_card_data_error() {
    // A future schema bump (or a hand-edited file) must be rejected, not
    // silently loaded as v1.
    let bad = br#"{"format":"hp41-data-v1","version":2,"registers":[]}"#;
    let err = decode_data(bad).unwrap_err();
    assert!(
        matches!(&err, HpError::CardData(msg) if msg.contains("version")),
        "expected unsupported-version diagnostic, got: {err:?}"
    );
}
