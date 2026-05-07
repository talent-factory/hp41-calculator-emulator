//! Integration tests for PRGM mode recording gate in dispatch() and
//! prgm_mode routing in flush_entry_buf().
//!
//! Plan 03-04: Phase 3 wave 2 — prgm_mode gate in dispatch() + flush routing.

use hp41_core::{CalcState, HpNum};
use hp41_core::ops::{dispatch, Op};

// ═══════════════════════════════════════════════════════════════════════════
// Task 1 tests: flush_entry_buf() prgm_mode routing
// ═══════════════════════════════════════════════════════════════════════════

/// When prgm_mode is false, flush_entry_buf pushes to stack (existing behaviour).
#[test]
fn test_flush_execute_mode_pushes_to_stack() {
    let mut s = CalcState::new();
    s.prgm_mode = false;
    s.entry_buf = "42".to_string();
    // Dispatch any op that triggers flush then does not consume X
    dispatch(&mut s, Op::SetDeg).unwrap();
    // Number must be on X register
    assert_eq!(s.stack.x, HpNum::from(42));
    // Program must remain empty — nothing recorded
    assert!(s.program.is_empty(), "execute mode must not record to program");
    // entry_buf must be cleared
    assert!(s.entry_buf.is_empty());
}

/// When prgm_mode is true, flush_entry_buf appends Op::PushNum to state.program.
#[test]
fn test_flush_prgm_mode_records_pushnum_to_program() {
    let mut s = CalcState::new();
    s.prgm_mode = true;
    s.entry_buf = "7".to_string();
    // Dispatch a recording-mode-compatible op (Add is recorded, not executed)
    dispatch(&mut s, Op::Add).unwrap();
    // PushNum(7) must be the first entry in the program
    assert!(!s.program.is_empty(), "program must contain PushNum after flush in prgm_mode");
    assert!(
        matches!(s.program[0], Op::PushNum(_)),
        "first program entry must be Op::PushNum"
    );
    // entry_buf must be cleared
    assert!(s.entry_buf.is_empty());
}

/// When prgm_mode is true and flush records PushNum, lift_enabled must NOT change.
/// (Recording mode does not affect execution state.)
#[test]
fn test_flush_prgm_mode_does_not_change_lift_enabled() {
    let mut s = CalcState::new();
    s.prgm_mode = true;
    s.entry_buf = "3".to_string();
    let lift_before = s.stack.lift_enabled; // false at startup
    dispatch(&mut s, Op::Add).unwrap();
    assert_eq!(
        s.stack.lift_enabled, lift_before,
        "lift_enabled must not change during prgm_mode recording flush"
    );
}

/// When prgm_mode is true and flush records PushNum, the stack X register must NOT change.
#[test]
fn test_flush_prgm_mode_does_not_push_to_stack() {
    let mut s = CalcState::new();
    s.prgm_mode = true;
    let x_before = s.stack.x.clone();
    s.entry_buf = "99".to_string();
    dispatch(&mut s, Op::Add).unwrap();
    assert_eq!(
        s.stack.x, x_before,
        "stack X must not change during prgm_mode recording flush"
    );
}

/// When entry_buf is empty, flush_entry_buf is a no-op regardless of prgm_mode.
#[test]
fn test_flush_empty_buf_noop_in_prgm_mode() {
    let mut s = CalcState::new();
    s.prgm_mode = true;
    // entry_buf is already empty — dispatch an op that would be recorded
    dispatch(&mut s, Op::Add).unwrap();
    // Program should have one entry: the Add op itself (not a PushNum from flush)
    assert_eq!(s.program.len(), 1, "only the Add op should be recorded, no PushNum from empty flush");
    assert_eq!(s.program[0], Op::Add);
}

// ═══════════════════════════════════════════════════════════════════════════
// Task 2 tests: dispatch() prgm_mode recording gate
// ═══════════════════════════════════════════════════════════════════════════

/// In prgm_mode, dispatch() records an op and returns Ok without executing it.
#[test]
fn test_dispatch_prgm_mode_records_op() {
    let mut s = CalcState::new();
    s.prgm_mode = true;
    // Push known values so we can verify they are unchanged
    s.stack.x = HpNum::from(3);
    s.stack.y = HpNum::from(5);
    dispatch(&mut s, Op::Add).unwrap();
    // Add must be recorded — not executed
    assert!(s.program.contains(&Op::Add), "dispatch in prgm_mode must record Add");
    // Stack must be unchanged — Add was not executed
    assert_eq!(s.stack.x, HpNum::from(3), "X must be unchanged — Add was recorded not executed");
    assert_eq!(s.stack.y, HpNum::from(5), "Y must be unchanged — Add was recorded not executed");
}

/// dispatch() with prgm_mode=false executes normally — existing behaviour unchanged.
#[test]
fn test_dispatch_execute_mode_unchanged() {
    let mut s = CalcState::new();
    s.prgm_mode = false;
    s.stack.x = HpNum::from(3);
    s.stack.y = HpNum::from(5);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Add).unwrap();
    assert_eq!(s.stack.x, HpNum::from(8), "Add must execute in non-prgm mode");
    assert!(s.program.is_empty(), "program must stay empty in execute mode");
}

/// In prgm_mode, dispatching Op::PrgmMode exits recording immediately (toggle, Pitfall 6).
/// The PrgmMode op itself must NOT be recorded.
#[test]
fn test_dispatch_prgm_mode_toggle_exits_recording() {
    let mut s = CalcState::new();
    s.prgm_mode = true;
    dispatch(&mut s, Op::PrgmMode).unwrap();
    assert!(!s.prgm_mode, "PrgmMode dispatch while recording must exit prgm_mode");
    assert!(
        !s.program.contains(&Op::PrgmMode),
        "PrgmMode op must NOT be recorded — it executes immediately"
    );
}

/// After entering prgm_mode, subsequent ops are recorded; after exiting, they execute.
#[test]
fn test_dispatch_record_then_exit_then_execute() {
    let mut s = CalcState::new();
    // Enter prgm_mode by setting the flag directly (PrgmMode op impl comes in 03-06)
    s.prgm_mode = true;
    // Record a couple of ops
    dispatch(&mut s, Op::Add).unwrap();
    dispatch(&mut s, Op::Sub).unwrap();
    // Exit prgm_mode via PrgmMode toggle
    dispatch(&mut s, Op::PrgmMode).unwrap();
    // Program should contain exactly Add and Sub — not PrgmMode
    assert_eq!(s.program.len(), 2);
    assert_eq!(s.program[0], Op::Add);
    assert_eq!(s.program[1], Op::Sub);
    // Now dispatch something that actually executes
    s.stack.x = HpNum::from(4);
    s.stack.y = HpNum::from(6);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Add).unwrap();
    assert_eq!(s.stack.x, HpNum::from(10), "After exiting prgm_mode, Add must execute");
    // Program grows by zero — the execute-mode Add is not recorded
    assert_eq!(s.program.len(), 2);
}

/// Verify that multiple sequential ops are all appended to program in order.
#[test]
fn test_dispatch_prgm_mode_records_multiple_ops_in_order() {
    let mut s = CalcState::new();
    s.prgm_mode = true;
    dispatch(&mut s, Op::Add).unwrap();
    dispatch(&mut s, Op::Sub).unwrap();
    dispatch(&mut s, Op::Mul).unwrap();
    assert_eq!(s.program.len(), 3);
    assert_eq!(s.program[0], Op::Add);
    assert_eq!(s.program[1], Op::Sub);
    assert_eq!(s.program[2], Op::Mul);
}
