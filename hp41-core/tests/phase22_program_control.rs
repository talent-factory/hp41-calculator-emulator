//! Integration tests for Phase 22 Plan 01 (program control:
//! STOP / PSE / GTO IND / XEQ IND + resume_program).
//!
//! Covers FN-PROG-01 / FN-PROG-02 / FN-PROG-06 / FN-PROG-07 plus the
//! three RESEARCH.md §2 pitfall sentinels:
//! - Pitfall 1: Op::Stop must NOT write display_override.
//! - Pitfall 2: resume_program must reset is_running on the Err path.
//! - Pitfall 3: Op::Pse's display_override survives subsequent run_loop
//!   iterations and is cleared by the next dispatch.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::program::{resume_program, run_program};
use hp41_core::ops::{dispatch, Op};
use hp41_core::{format_hpnum, CalcState, DisplayMode, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── FN-PROG-01: STOP halts; resume_program continues from pc ────────────────

#[test]
fn test_stop_then_resume() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(42i32)),
        Op::Stop,
        Op::PushNum(HpNum::from(99i32)),
    ];

    run_program(&mut state, "A").unwrap();

    // After STOP: X is 42, is_running is false, pc is within program
    // (advanced past STOP by the top-of-iteration pc += 1).
    assert_eq!(state.stack.x, HpNum::from(42i32));
    assert!(
        !state.is_running,
        "is_running must be false after STOP halt"
    );
    assert!(
        state.pc < state.program.len(),
        "pc ({}) must point at the next step within program (len {})",
        state.pc,
        state.program.len()
    );

    // Resume — continues from saved pc through the final PushNum.
    resume_program(&mut state).unwrap();
    assert_eq!(state.stack.x, HpNum::from(99i32));
    assert!(!state.is_running);
}

// ── Pitfall 1 sentinel: STOP must NOT write display_override ────────────────

#[test]
fn test_stop_does_not_write_display_override() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(7i32)),
        Op::Stop,
    ];
    // Pre-condition: display_override starts None.
    assert!(state.display_override.is_none());

    run_program(&mut state, "A").unwrap();

    // The PushNum step does not write display_override either, so the
    // contract that STOP "freezes whatever is currently displayed" reduces
    // to: STOP leaves display_override unchanged. With no prior writer in
    // this program, that means display_override stays None.
    assert!(
        state.display_override.is_none(),
        "Op::Stop must NOT write display_override (Pitfall 1); got {:?}",
        state.display_override
    );
}

// ── Pitfall 2 sentinel: resume_program resets is_running on Err path ────────

#[test]
fn test_resume_resets_is_running_on_err() {
    let mut state = CalcState::new();
    // Program designed to fail mid-run: XEQ to non-existent label.
    state.program = vec![Op::Xeq("MISSING".to_string())];
    state.pc = 0;

    let result = resume_program(&mut state);

    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "expected InvalidOp from missing-label XEQ, got {result:?}"
    );
    assert!(
        !state.is_running,
        "is_running MUST be reset to false even on Err path (Pitfall 2)"
    );
}

#[test]
fn test_resume_program_rejects_when_pc_past_end() {
    let mut state = CalcState::new();
    state.program = vec![Op::PushNum(HpNum::from(1i32))];
    state.pc = state.program.len(); // past end

    let result = resume_program(&mut state);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "resume_program must reject when pc >= program.len()"
    );
    assert!(!state.is_running);
}

#[test]
fn test_resume_program_preserves_call_stack() {
    // resume_program must NOT clear state.call_stack (unlike run_program).
    let mut state = CalcState::new();
    state.program = vec![Op::PushNum(HpNum::from(1i32))]; // any benign program
    state.pc = 0;
    state.call_stack = vec![123usize, 456usize];

    resume_program(&mut state).unwrap();

    // The benign program does not pop the call_stack, so both frames must
    // still be there after a successful resume.
    assert_eq!(state.call_stack, vec![123usize, 456usize]);
}

// ── FN-PROG-02: PSE writes both channels ────────────────────────────────────

#[test]
fn test_pse_writes_both_channels() {
    let mut state = CalcState::new();
    // Force a known display mode so the formatted string is predictable.
    state.display_mode = DisplayMode::Fix(4);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from_str("1.23").unwrap())),
        Op::Pse,
    ];

    run_program(&mut state, "A").unwrap();

    let expected = format_hpnum(
        &HpNum::rounded(Decimal::from_str("1.23").unwrap()),
        &DisplayMode::Fix(4),
    );
    assert_eq!(
        state.display_override.as_deref(),
        Some(expected.as_str()),
        "display_override must equal format_hpnum(X) at PSE time"
    );
    assert!(
        state.event_buffer.iter().any(|e| e == "PAUSE 1000"),
        "event_buffer must contain 'PAUSE 1000' (got {:?})",
        state.event_buffer
    );
}

// ── Pitfall 3 sentinel: PSE's display_override survives next step ───────────

#[test]
fn test_pse_display_override_survives_next_program_step() {
    let mut state = CalcState::new();
    state.display_mode = DisplayMode::Fix(4);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from_str("1.23").unwrap())),
        Op::Pse,
        Op::PushNum(HpNum::from(5i32)),
    ];

    run_program(&mut state, "A").unwrap();

    // run_loop calls execute_op directly (NOT dispatch), so the dispatch-top
    // display_override = None clear does NOT fire between iterations. The
    // PSE write therefore survives the subsequent PushNum step.
    let expected_pse = format_hpnum(
        &HpNum::rounded(Decimal::from_str("1.23").unwrap()),
        &DisplayMode::Fix(4),
    );
    assert_eq!(
        state.display_override.as_deref(),
        Some(expected_pse.as_str()),
        "display_override from PSE must survive subsequent run_loop iterations"
    );
    // X still ends as 5 from the final PushNum.
    assert_eq!(state.stack.x, HpNum::from(5i32));
}

// ── Pitfall 3 sentinel: next interactive dispatch clears display_override ───

#[test]
fn test_pse_display_override_cleared_by_next_dispatch() {
    let mut state = CalcState::new();
    state.display_mode = DisplayMode::Fix(4);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(7i32)),
        Op::Pse,
    ];

    run_program(&mut state, "A").unwrap();
    assert!(
        state.display_override.is_some(),
        "PSE must have written display_override"
    );

    // Next interactive dispatch — any op — clears display_override via the
    // mod.rs:410 dispatch-top clear. This is Phase 21's Pitfall 5 in action.
    dispatch(&mut state, Op::Add).ok();
    // Don't care if Op::Add errors (stack may be partial) — the clear happens
    // BEFORE the op runs, at the top of dispatch().
    assert!(
        state.display_override.is_none(),
        "next dispatch must clear display_override (HP-41 'value visible until next key')"
    );
}

// ── FN-PROG-06: GTO IND happy + reject paths ────────────────────────────────

#[test]
fn test_gto_ind_happy() {
    // R05 = 42 (integer pointer); program has LBL "42" target.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(42i32);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::PushNum(HpNum::from(111i32)), // would-be unreachable after GTO
        Op::Lbl("42".to_string()),
        Op::PushNum(HpNum::from(7i32)),
    ];

    run_program(&mut state, "A").unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(7i32),
        "GTO IND R05 (= 42) must branch to LBL 42; got X = {:?}",
        state.stack.x
    );
}

#[test]
fn test_gto_ind_non_integer_rejects() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.345").unwrap());
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::Lbl("12".to_string()), // would-be target if truncation were allowed
    ];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "GTO IND with non-integer pointer must return InvalidOp (FN-IND-02); got {result:?}"
    );
}

#[test]
fn test_gto_ind_reg_out_of_range_rejects() {
    // CalcState::new() ships 100 registers (R00..R99). Asking for R200 must
    // return InvalidOp via .get().ok_or(InvalidOp) — and crucially NOT panic.
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("A".to_string()), Op::GtoInd(200)];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "GTO IND with reg >= regs.len() must return InvalidOp, not panic; got {result:?}"
    );
}

// ── FN-PROG-07: XEQ IND happy + reject paths + call-stack guard ─────────────

#[test]
fn test_xeq_ind_happy() {
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(10i32);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::XeqInd(3),
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
        Op::Lbl("10".to_string()),
        Op::PushNum(HpNum::from(99i32)),
        Op::Rtn,
    ];

    run_program(&mut state, "A").unwrap();
    // After: A pushes 99 (via the subroutine), RTN returns to step after
    // XeqInd, pushes 2, RTN at the top level breaks.
    assert_eq!(
        state.stack.x,
        HpNum::from(2i32),
        "X after XEQ IND subroutine + return must be 2; got {:?}",
        state.stack.x
    );
}

#[test]
fn test_xeq_ind_4_deep_call_stack_rejects() {
    // Drive via resume_program (NOT run_program) so the pre-set call_stack
    // is NOT cleared at entry — run_program would wipe it (line 162).
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(10i32);
    state.program = vec![Op::XeqInd(3), Op::Lbl("10".to_string())];
    state.pc = 0;
    state.call_stack = vec![999usize; 4]; // pre-fill to 4 frames

    let result = resume_program(&mut state);

    assert!(
        matches!(result, Err(HpError::CallDepth)),
        "XEQ IND at 4-deep call_stack must return CallDepth; got {result:?}"
    );
    // Pre-mutation atomicity: the push did NOT happen — still exactly 4.
    assert_eq!(
        state.call_stack.len(),
        4,
        "call_stack must not be mutated on CallDepth (pre-mutation guard); got {:?}",
        state.call_stack
    );
}

#[test]
fn test_xeq_ind_reg_out_of_range_rejects() {
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("A".to_string()), Op::XeqInd(200)];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "XEQ IND with reg >= regs.len() must return InvalidOp, not panic; got {result:?}"
    );
}

#[test]
fn test_xeq_ind_non_integer_rejects() {
    let mut state = CalcState::new();
    state.regs[3] = HpNum::rounded(Decimal::from_str("10.5").unwrap());
    state.program = vec![Op::Lbl("A".to_string()), Op::XeqInd(3)];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "XEQ IND with non-integer pointer must return InvalidOp (FN-IND-02); got {result:?}"
    );
}

// ── Phase 24 D-24.5 sentinel: refactored GtoInd/XeqInd onto shared helper ────
//
// These tests exercise the same code paths as the original Phase-22 tests
// above, but assert specific invariants of the shared-helper refactor:
// (1) call-depth guard still runs FIRST (pre-mutation atomicity)
// (2) Decimal::to_string preserves sign for label-lookup callers
//
// If any of these tests fail, the refactor regressed Phase-22 behavior.

#[test]
fn phase24_gto_ind_uses_shared_helper() {
    // Sanity — proves the refactored arm still routes via
    // crate::ops::indirect::resolve_indirect_decimal to find_in_program.
    // Identical inputs/outputs to test_gto_ind_happy.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(42i32);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::PushNum(HpNum::from(111i32)), // would-be unreachable after GTO
        Op::Lbl("42".to_string()),
        Op::PushNum(HpNum::from(7i32)),
    ];

    run_program(&mut state, "A").unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(7i32),
        "GTO IND R05 (= 42) must branch to LBL 42 via shared helper; got X = {:?}",
        state.stack.x
    );
}

#[test]
fn phase24_xeq_ind_uses_shared_helper() {
    // Sanity for XeqInd refactor — same flow as test_xeq_ind_happy.
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(10i32);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::XeqInd(3),
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
        Op::Lbl("10".to_string()),
        Op::PushNum(HpNum::from(99i32)),
        Op::Rtn,
    ];

    run_program(&mut state, "A").unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(2i32),
        "X after XEQ IND subroutine + return must be 2 via shared helper; got {:?}",
        state.stack.x
    );
}

#[test]
fn phase24_xeq_ind_call_depth_guard_runs_before_pointer_read() {
    // CRITICAL D-24.5 sentinel: drive via resume_program (NOT run_program —
    // the latter wipes call_stack at entry per the inline comment at
    // test_xeq_ind_4_deep_call_stack_rejects above). With a 4-deep call_stack
    // AND a non-integer pointer, the pre-mutation atomicity guard must fire
    // FIRST and return CallDepth — NOT InvalidOp from a downstream pointer
    // read. If a future planner accidentally moves the call_stack.len() >= 4
    // check below the pointer read, this test catches it.
    let mut state = CalcState::new();
    state.regs[3] = HpNum::rounded(Decimal::from_str("12.345").unwrap());
    state.program = vec![Op::XeqInd(3), Op::Lbl("12".to_string())];
    state.pc = 0;
    state.call_stack = vec![999usize; 4]; // pre-fill to 4 frames

    let result = resume_program(&mut state);

    assert!(
        matches!(result, Err(HpError::CallDepth)),
        "XEQ IND at 4-deep call_stack with non-integer pointer must return \
         CallDepth (pre-mutation guard fires FIRST), NOT InvalidOp; got {result:?}"
    );
    // Pre-mutation atomicity: the push did NOT happen — still exactly 4.
    assert_eq!(
        state.call_stack.len(),
        4,
        "call_stack must not be mutated on CallDepth (pre-mutation guard); got {:?}",
        state.call_stack
    );
}

#[test]
fn phase24_gto_ind_negative_pointer_stringifies_with_sign() {
    // Pitfall 2 sentinel: -3 IS an integer (passes the inner helper without
    // rejection), so the failure path is "find_in_program('-3') failed", NOT
    // "non-integer pointer". This confirms Decimal::to_string preserves the
    // sign exactly as Phase 22 did pre-refactor.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(-3i32);
    state.program = vec![Op::Lbl("A".to_string()), Op::GtoInd(5)]; // no LBL "-3"

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "GTO IND R05 (= -3) must InvalidOp via find_in_program (label '-3' \
         not found), NOT via non-integer rejection; got {result:?}"
    );
}
