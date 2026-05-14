//! Integration tests for Phase 22 Plan 02 (program editing: CLP / DEL / INS).
//!
//! Covers FN-PROG-03 / FN-PROG-04 / FN-PROG-05 plus the two RESEARCH.md §2
//! pitfall sentinels and the D-22.10 prgm_mode-gate enforcement:
//! - Pitfall 4: CLP on the last labelled block must drain to end-of-Vec.
//! - Pitfall 6: CLP must reposition state.pc to the start of the deleted
//!   block (clamped to the new program.len()), so the cursor lands on a
//!   sensible step after editing.
//! - D-22.10: all three ops MUST gate on `state.prgm_mode == true`;
//!   `prgm_mode == false` → HpError::InvalidOp without mutating state.program.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpError, HpNum};

// Small helper: build the canonical 3-block test program
//   [Lbl A, PushNum 1, Lbl B, PushNum 2, Lbl C, PushNum 3]
// Indices: 0    1            2    3            4    5
fn make_three_block_program() -> Vec<Op> {
    vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(1i32)),
        Op::Lbl("B".to_string()),
        Op::PushNum(HpNum::from(2i32)),
        Op::Lbl("C".to_string()),
        Op::PushNum(HpNum::from(3i32)),
    ]
}

// ── FN-PROG-03: CLP boundary (LBL → next LBL) ───────────────────────────────

#[test]
fn test_clp_boundary() {
    let mut state = CalcState::new();
    state.program = make_three_block_program();
    state.prgm_mode = true;

    dispatch(&mut state, Op::Clp("B".to_string())).unwrap();

    // LBL B and its PushNum should be drained; LBL A/C blocks preserved.
    assert_eq!(state.program.len(), 4);
    assert!(matches!(&state.program[0], Op::Lbl(n) if n == "A"));
    assert!(matches!(&state.program[1], Op::PushNum(_)));
    assert!(matches!(&state.program[2], Op::Lbl(n) if n == "C"));
    assert!(matches!(&state.program[3], Op::PushNum(_)));
}

// ── Pitfall 4 sentinel: CLP on last labelled block drains to end-of-Vec ─────

#[test]
fn test_clp_last_block_drains_to_end() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(1i32)),
        Op::Lbl("B".to_string()),
        Op::PushNum(HpNum::from(2i32)),
    ];
    state.prgm_mode = true;

    dispatch(&mut state, Op::Clp("B".to_string())).unwrap();

    // Last labelled block: drain to end of Vec. Only LBL A block survives.
    assert_eq!(state.program.len(), 2);
    assert!(matches!(&state.program[0], Op::Lbl(n) if n == "A"));
    assert!(matches!(&state.program[1], Op::PushNum(_)));
}

// ── Pitfall 6 sentinel: pc repositions to start of deleted block ────────────

#[test]
fn test_clp_pc_repositioned_to_start() {
    let mut state = CalcState::new();
    state.program = make_three_block_program();
    state.prgm_mode = true;
    state.pc = 5; // cursor was past the block we're about to delete

    dispatch(&mut state, Op::Clp("B".to_string())).unwrap();

    // After CLP "B": LBL B was at index 2; deleted block was [2..4).
    // pc must reposition to start (2), the post-drain location of what
    // used to be LBL C. Without the fix, pc would still be 5 (dangling
    // past the new program length 4).
    assert_eq!(
        state.pc, 2,
        "Pitfall 6: pc must land at start of deleted block, got {}",
        state.pc
    );
}

// ── Pitfall 6 edge: pc clamps to program.len() when start == new len ────────

#[test]
fn test_clp_pc_clamps_to_program_len() {
    // Pathological setup: the deleted block reaches end-of-Vec AND
    // start == post-drain program.len() (i.e., we delete from index N
    // through the end, leaving exactly N steps; new len == N == start).
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(1i32)),
        Op::Lbl("B".to_string()),
        Op::PushNum(HpNum::from(2i32)),
    ];
    state.prgm_mode = true;
    state.pc = 0; // arbitrary; the clamp is on start, not pc-pre

    dispatch(&mut state, Op::Clp("B".to_string())).unwrap();

    // LBL B was at index 2. After drain, program.len() == 2.
    // start.min(program.len()) == 2.min(2) == 2 (valid one-past-the-end).
    assert_eq!(state.program.len(), 2);
    assert_eq!(state.pc, 2);
}

// ── D-22.7: missing label → InvalidOp ───────────────────────────────────────

#[test]
fn test_clp_missing_label_rejects() {
    let mut state = CalcState::new();
    state.program = make_three_block_program();
    let len_before = state.program.len();
    state.prgm_mode = true;

    let result = dispatch(&mut state, Op::Clp("NONEXISTENT".to_string()));

    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "missing label must reject with InvalidOp; got {:?}",
        result
    );
    // Program must remain unchanged on the error path.
    assert_eq!(state.program.len(), len_before);
}

// ── D-22.10: prgm_mode == false → InvalidOp, program unchanged ──────────────

#[test]
fn test_clp_prgm_mode_false_rejects() {
    let mut state = CalcState::new();
    state.program = make_three_block_program();
    state.prgm_mode = false; // CLP must refuse outside PRGM mode

    let result = dispatch(&mut state, Op::Clp("A".to_string()));

    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "prgm_mode == false must reject CLP; got {:?}",
        result
    );
    // Program must remain unchanged.
    assert_eq!(state.program.len(), 6);
    assert!(matches!(&state.program[0], Op::Lbl(n) if n == "A"));
}

// ── FN-PROG-04: DEL clamping ────────────────────────────────────────────────

#[test]
fn test_del_clamping() {
    let mut state = CalcState::new();
    // 5-step program.
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
        Op::PushNum(HpNum::from(3i32)),
        Op::PushNum(HpNum::from(4i32)),
        Op::PushNum(HpNum::from(5i32)),
    ];
    state.pc = 2; // remaining = 5 - 2 = 3
    state.prgm_mode = true;

    // Request 100 deletions — must clamp silently to 3.
    dispatch(&mut state, Op::Del(100u8)).unwrap();

    assert_eq!(state.program.len(), 2, "DEL 100 from pc=2 in a 5-step program must clamp to 3 deletions");
    // pc unchanged.
    assert_eq!(state.pc, 2);
}

#[test]
fn test_del_zero_is_noop() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
        Op::PushNum(HpNum::from(3i32)),
    ];
    state.pc = 1;
    state.prgm_mode = true;

    dispatch(&mut state, Op::Del(0u8)).unwrap();

    assert_eq!(state.program.len(), 3, "DEL 0 must be a no-op");
    assert_eq!(state.pc, 1);
}

#[test]
fn test_del_pc_at_end_is_noop() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    state.pc = state.program.len(); // pc == len
    state.prgm_mode = true;

    dispatch(&mut state, Op::Del(5u8)).unwrap();

    assert_eq!(state.program.len(), 2, "DEL at pc==len must be a no-op");
    assert_eq!(state.pc, 2);
}

#[test]
fn test_del_prgm_mode_false_rejects() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
        Op::PushNum(HpNum::from(3i32)),
    ];
    state.pc = 0;
    state.prgm_mode = false;

    let result = dispatch(&mut state, Op::Del(2u8));

    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "prgm_mode == false must reject DEL; got {:?}",
        result
    );
    // Program must remain unchanged.
    assert_eq!(state.program.len(), 3);
}

// ── FN-PROG-05: INS inserts Op::Null at pc; pc unchanged ────────────────────

#[test]
fn test_ins_inserts_null_at_pc() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
        Op::PushNum(HpNum::from(3i32)),
    ];
    state.pc = 1;
    state.prgm_mode = true;

    dispatch(&mut state, Op::Ins).unwrap();

    assert_eq!(state.program.len(), 4, "INS must grow the program by 1");
    assert!(
        matches!(state.program[1], Op::Null),
        "INS at pc=1 must place Op::Null at index 1; got {:?}",
        state.program[1]
    );
    // pc UNCHANGED — cursor still points at the freshly inserted Null.
    assert_eq!(state.pc, 1, "INS must NOT modify state.pc");
}

#[test]
fn test_ins_prgm_mode_false_rejects() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    state.pc = 0;
    state.prgm_mode = false;

    let result = dispatch(&mut state, Op::Ins);

    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "prgm_mode == false must reject INS; got {:?}",
        result
    );
    // Program must remain unchanged.
    assert_eq!(state.program.len(), 2);
}

// ── Bonus: edit ops are NOT recorded even in PRGM mode (D-22.10) ────────────
//
// The PRGM-mode recording gate in dispatch() special-cases Clp/Del/Ins so
// they execute immediately rather than being appended to state.program.
// If the gate were missing or buggy, the test program would gain spurious
// Op::Ins entries on each call instead of having Op::Null inserted at pc.

#[test]
fn test_ins_is_not_self_recorded_in_prgm_mode() {
    let mut state = CalcState::new();
    state.program = vec![Op::PushNum(HpNum::from(1i32))];
    state.pc = 0;
    state.prgm_mode = true;

    dispatch(&mut state, Op::Ins).unwrap();

    // Expect: [Null, PushNum 1]. If INS were self-recorded, we'd get
    // [PushNum 1, Ins] instead (and Op::Null would never appear).
    assert_eq!(state.program.len(), 2);
    assert!(
        matches!(state.program[0], Op::Null),
        "INS must execute as edit primitive (insert Null at pc), not self-record; got {:?}",
        state.program
    );
    assert!(matches!(state.program[1], Op::PushNum(_)));
}

// ── Regression: D-22.23 zero-panic invariant under corrupted-load scenario ──
//
// `Vec::insert` panics when `index > len()`. Under normal Phase 22 control
// flow CLP/DEL/run_loop maintain `state.pc <= state.program.len()`, but a
// corrupted or malicious `~/.hp41/autosave.json` could deserialize a
// `CalcState` with `pc > program.len()` — `CalcState` has no field-level
// validation at load time. `op_ins` clamps `state.pc` to
// `state.program.len()` before `Vec::insert` so hp41-core stays
// panic-free even in that pathological state. Mirrors the existing
// `op_del` `saturating_sub` / `.min(...)` neutralization. Targets the
// Phase 22 review Warning at `program.rs:208`.

#[test]
fn test_ins_at_pc_past_len_does_not_panic() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    // Simulate a corrupted-load state: pc beyond program.len().
    state.pc = 99;
    state.prgm_mode = true;

    // Must NOT panic. Under the clamp, Op::Null is appended at the end
    // (idx = program.len() = 2), leaving program.len() == 3.
    dispatch(&mut state, Op::Ins).unwrap();

    assert_eq!(
        state.program.len(),
        3,
        "INS must grow the program by 1 even when pc > len()"
    );
    assert!(
        matches!(state.program[2], Op::Null),
        "Op::Null must land at the clamped index (program.len()); got {:?}",
        state.program[2]
    );
    // pc UNCHANGED — same contract as the normal-path INS.
    assert_eq!(state.pc, 99, "INS must NOT modify state.pc even on clamp");
}

#[test]
fn test_ins_at_pc_equals_len_appends_null() {
    // Legitimate append-at-end case (the "STOP at end-of-program" edit
    // scenario the Phase 22 review noted as untested). pc == program.len()
    // is in-range for Vec::insert (which accepts index == len), so this
    // path works regardless of the clamp — but the regression test pins
    // it explicitly.
    let mut state = CalcState::new();
    state.program = vec![
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    state.pc = state.program.len();
    state.prgm_mode = true;

    dispatch(&mut state, Op::Ins).unwrap();

    assert_eq!(state.program.len(), 3);
    assert!(
        matches!(state.program[2], Op::Null),
        "Op::Null must be appended at pc when pc == program.len()"
    );
    assert_eq!(state.pc, 2, "INS must NOT modify state.pc");
}
