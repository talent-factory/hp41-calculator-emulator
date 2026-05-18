// Algorithm independently re-derived from HP-41C Owner's Manual; Free42 source consulted only as sanity-check oracle, not copied.
//! Plan 32-08 gap-closure for `hp41-core/src/ops/program.rs` — closes the 86.42 %
//! per-file coverage gap by exercising error branches in:
//!   - interactive op_xeq (line 83): is_running=false + unknown label
//!   - PRGM-mode guards for op_clp / op_del / op_ins (lines 159, 188, 211)
//!   - SIZE n>319 guard (line 271)
//!   - CATALOG n=0 or n>=5 guard (line 303)
//!   - XEQ in run_loop: missing label + no builtin + no xrom (line 550)
//!   - resume_program pc>=len guard (line 450)
//!   - FmtFix / FmtSci / FmtEng n>9 guards (lines 760-778)
//!   - SyntheticByte invalid-byte arm in execute_op (line 831)
//!
//! Each test follows the D-27.1 risk-weighted discipline with a `// Catches:`
//! doc comment naming the bug class. The lint `lint_math1_assertions.rs` scopes
//! only `tests/math1_*.rs` — this file is OUT of scope but follows the same
//! assertion discipline (no `assert_eq!(decimal, decimal)`, use approx if needed).

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, program::run_program, Op};
use hp41_core::{CalcState, HpError};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fresh_state() -> CalcState {
    CalcState::new()
}

fn state_with_program(ops: Vec<Op>) -> CalcState {
    CalcState {
        program: ops,
        ..Default::default()
    }
}

// ── Test 1: op_xeq interactive with unknown label (line 83) ──────────────────

#[test]
fn op_xeq_interactive_unknown_label_returns_invalid_op() {
    // Catches: op_xeq() fallthrough when is_running=false, no builtin_card_op
    // match, and no xrom_resolve match — the final Err(HpError::InvalidOp) at
    // program.rs line 83. Regression: if the resolver chain silently swallows
    // unknown names this returns Ok(()) and the display stays stale.
    let mut state = fresh_state();
    // "UNKNOWNLABEL" is not in builtin_card_op (WPRGM/RDPRGM/WDTA/RDTA/conditional tests)
    // and not in the XROM Math Pac I module.
    let result = dispatch(&mut state, Op::Xeq("UNKNOWNLABEL".to_string()));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "op_xeq with unknown label (is_running=false) must return InvalidOp"
    );
}

// ── Test 2: op_clp without prgm_mode (line 159) ──────────────────────────────

#[test]
fn op_clp_without_prgm_mode_returns_invalid_op() {
    // Catches: op_clp() prgm_mode guard — calling CLP while NOT in programming
    // mode must return InvalidOp. Regression: if the guard is removed, CLP
    // deletes user program sections during interactive operation.
    let mut state = state_with_program(vec![Op::Lbl("T".to_string()), Op::Rtn]);
    state.prgm_mode = false; // NOT in programming mode
    let result = dispatch(&mut state, Op::Clp("T".to_string()));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "CLP without prgm_mode must return InvalidOp"
    );
}

// ── Test 3: op_clp with prgm_mode but missing label (line 165) ───────────────

#[test]
fn op_clp_prgm_mode_missing_label_returns_invalid_op() {
    // Catches: op_clp() .ok_or(HpError::InvalidOp)? when label not found in
    // state.program — program.rs line 165. This is a separate path from the
    // prgm_mode guard: the guard passes but the label scan fails.
    let mut state = state_with_program(vec![Op::Lbl("A".to_string()), Op::Rtn]);
    state.prgm_mode = true;
    // "MISSING" does not exist in the program
    let result = dispatch(&mut state, Op::Clp("MISSING".to_string()));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "CLP with missing label must return InvalidOp"
    );
}

// ── Test 4: op_del without prgm_mode (line 188) ──────────────────────────────

#[test]
fn op_del_without_prgm_mode_returns_invalid_op() {
    // Catches: op_del() prgm_mode guard — program.rs line 188. DEL is a
    // PRGM-mode editing primitive (D-22.9); calling it interactively is a bug.
    let mut state = fresh_state();
    state.prgm_mode = false;
    let result = dispatch(&mut state, Op::Del(3));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "DEL without prgm_mode must return InvalidOp"
    );
}

// ── Test 5: op_ins without prgm_mode (line 211) ──────────────────────────────

#[test]
fn op_ins_without_prgm_mode_returns_invalid_op() {
    // Catches: op_ins() prgm_mode guard — program.rs line 211. INS is a
    // PRGM-mode editing primitive (D-22.8); calling it interactively is a bug.
    let mut state = fresh_state();
    state.prgm_mode = false;
    let result = dispatch(&mut state, Op::Ins);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "INS without prgm_mode must return InvalidOp"
    );
}

// ── Test 6: op_size n > 319 (line 271) ───────────────────────────────────────

#[test]
fn size_n_above_319_returns_invalid_op() {
    // Catches: op_size() upper-bound guard (n > 319 → InvalidOp) at program.rs
    // line 271. Regression: without the guard, Vec::resize to an arbitrary
    // size could exhaust memory or corrupt the register index space.
    let mut state = fresh_state();
    let result = dispatch(&mut state, Op::Size(320));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "SIZE 320 must return InvalidOp (max is 319)"
    );
}

#[test]
fn size_n_max_valid_succeeds() {
    // Catches: off-by-one adjacent to the SIZE 319 boundary — verify that 319
    // itself is accepted (upper-bound guard is strictly n > 319).
    let mut state = fresh_state();
    dispatch(&mut state, Op::Size(319)).unwrap();
    assert_eq!(state.regs.len(), 319);
}

// ── Test 7: op_catalog n=0 (line 303) ────────────────────────────────────────

#[test]
fn catalog_n_zero_returns_invalid_op() {
    // Catches: op_catalog() lower-bound guard (n == 0 → InvalidOp) at
    // program.rs line 303. CAT 0 is undefined on real HP-41 hardware.
    let mut state = fresh_state();
    let result = dispatch(&mut state, Op::Catalog(0));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "CAT 0 must return InvalidOp"
    );
}

// ── Test 8: op_catalog n >= 5 (line 303) ─────────────────────────────────────

#[test]
fn catalog_n_above_4_returns_invalid_op() {
    // Catches: op_catalog() upper-bound guard (n >= 5 → InvalidOp) at
    // program.rs line 303. CAT 5 is above the documented HP-41 range (1..4).
    let mut state = fresh_state();
    let result = dispatch(&mut state, Op::Catalog(5));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "CAT 5 must return InvalidOp"
    );
}

// ── Test 9: XEQ unknown label in run_loop (line 550) ─────────────────────────

#[test]
fn xeq_missing_label_in_run_loop_returns_invalid_op() {
    // Catches: Op::Xeq arm in run_loop when find_in_program fails, builtin_card_op
    // returns None, and xrom_resolve returns None — the final Err(HpError::InvalidOp)
    // at program.rs line 550. Regression: if the resolver chain silently swallows
    // the miss, the program advances past the bad XEQ and produces wrong output.
    //
    // Uses a label name that is not in the program, not a builtin card op name
    // (WPRGM/RDPRGM/WDTA/RDTA), and not a Math Pac I XROM entry.
    let mut state = state_with_program(vec![
        Op::Lbl("START".to_string()),
        Op::Xeq("NOSUCHLABEL".to_string()),
        Op::Rtn,
    ]);
    let result = run_program(&mut state, "START");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "XEQ to missing label in run_loop must return InvalidOp"
    );
}

// ── Test 10: resume_program pc >= len (line 450) ─────────────────────────────

#[test]
fn resume_program_pc_at_end_returns_invalid_op() {
    // Catches: resume_program() "nothing to resume" guard (pc >= program.len()
    // → InvalidOp) at program.rs line 450. Regression: without the guard, a
    // stale pc after a finished program would re-enter run_loop from an out-of-
    // bounds offset and immediately fall to the "ran off end" break, returning
    // Ok(()) — silently ignoring the stale state.
    use hp41_core::ops::program::resume_program;
    let mut state = state_with_program(vec![Op::Lbl("T".to_string()), Op::Rtn]);
    // Set pc to exactly program.len() (past the last step)
    state.pc = state.program.len();
    let result = resume_program(&mut state);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "resume_program with pc >= program.len() must return InvalidOp"
    );
}

#[test]
fn resume_program_pc_beyond_len_also_returns_invalid_op() {
    // Catches: same guard but with pc > len (can happen after SIZE shrinks the
    // program or after a serialized CalcState is loaded with a stale pc).
    use hp41_core::ops::program::resume_program;
    let mut state = state_with_program(vec![Op::Lbl("T".to_string()), Op::Rtn]);
    state.pc = 999; // far past end
    let result = resume_program(&mut state);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "resume_program with pc >> program.len() must return InvalidOp"
    );
}

// ── Test 11: FmtFix n > 9 in execute_op (line ~760-761) ─────────────────────

#[test]
fn fmt_fix_above_9_returns_invalid_op() {
    // Catches: FmtFix n > 9 guard in execute_op (program.rs line ~761). This
    // path is only reachable via execute_op (inside run_loop), not interactive
    // dispatch — interactive dispatch also checks this, but the execute_op arm
    // is a separate, independently-uncovered arm.
    let mut state = state_with_program(vec![
        Op::Lbl("T".to_string()),
        Op::FmtFix(10), // 10 > 9 — invalid
        Op::Rtn,
    ]);
    let result = run_program(&mut state, "T");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "FMT FIX 10 inside run_program must return InvalidOp"
    );
}

// ── Test 12: FmtSci n > 9 in execute_op (line ~769) ─────────────────────────

#[test]
fn fmt_sci_above_9_returns_invalid_op() {
    // Catches: FmtSci n > 9 guard in execute_op (program.rs line ~769). Same
    // separation rationale as fmt_fix — the execute_op arm is independently
    // uncovered from the interactive dispatch arm.
    let mut state = state_with_program(vec![
        Op::Lbl("T".to_string()),
        Op::FmtSci(10), // invalid
        Op::Rtn,
    ]);
    let result = run_program(&mut state, "T");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "FMT SCI 10 inside run_program must return InvalidOp"
    );
}

// ── Test 13: FmtEng n > 9 in execute_op (line ~777) ─────────────────────────

#[test]
fn fmt_eng_above_9_returns_invalid_op() {
    // Catches: FmtEng n > 9 guard in execute_op (program.rs line ~777). Third
    // member of the Fmt* error-branch triad; all three must be independently
    // tested because they are three distinct match arms.
    let mut state = state_with_program(vec![
        Op::Lbl("T".to_string()),
        Op::FmtEng(10), // invalid
        Op::Rtn,
    ]);
    let result = run_program(&mut state, "T");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "FMT ENG 10 inside run_program must return InvalidOp"
    );
}

// ── Test 14: SyntheticByte invalid byte in execute_op (line ~831) ────────────

#[test]
fn synthetic_byte_invalid_in_execute_op_returns_invalid_op() {
    // Catches: Op::SyntheticByte(b) arm in execute_op when synthetic_byte_to_op
    // returns None — program.rs line ~831. Byte 0x00 is not in the safe subset
    // (the subset only contains ~23 entries per CLAUDE.md §synthetic programming).
    // Regression: if the None arm silently returns Ok(()), synthetic bytes outside
    // the safe subset would execute as no-ops instead of surfacing the error.
    let mut state = state_with_program(vec![
        Op::Lbl("T".to_string()),
        Op::SyntheticByte(0x00), // 0x00 is NOT in the safe subset
        Op::Rtn,
    ]);
    let result = run_program(&mut state, "T");
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "SyntheticByte(0x00) with invalid byte must return InvalidOp"
    );
}

#[test]
fn synthetic_byte_valid_in_execute_op_succeeds() {
    // Catches: SyntheticByte(0x40) == Op::Add — the happy path adjacent to the
    // error path above. Verifies the safe subset is wired end-to-end through
    // execute_op (not just dispatch). 3 + 4 = 7.
    use hp41_core::HpNum;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let mut state = state_with_program(vec![
        Op::Lbl("T".to_string()),
        Op::PushNum(HpNum::from(Decimal::from_str("3").unwrap())),
        Op::PushNum(HpNum::from(Decimal::from_str("4").unwrap())),
        Op::SyntheticByte(0x40), // 0x40 = Op::Add
        Op::Rtn,
    ]);
    run_program(&mut state, "T").unwrap();
    assert_eq!(
        state.stack.x.inner(),
        Decimal::from_str("7").unwrap(),
        "SyntheticByte(0x40) must execute Op::Add: 3+4=7"
    );
}
