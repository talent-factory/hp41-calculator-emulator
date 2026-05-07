//! Integration tests for PROG-01 + PROG-02: keystroke programming engine.
//!
//! Test naming follows the pattern from register_tests.rs and entry_buf_tests.rs.
//! All tests use CalcState::new() and hp41_core::ops::dispatch() as the entry point.

use hp41_core::ops::program::run_program; // full path — lib.rs pub use added in 03-06
use hp41_core::ops::{dispatch, Op, TestKind};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push(state: &mut CalcState, n: i64) {
    let d = Decimal::from(n);
    // Simulate keyboard number entry: lift if enabled, place n on X, enable lift.
    // This mirrors the flush_entry_buf → enter_number + LiftEffect::Enable path.
    if state.stack.lift_enabled {
        state.stack.t = state.stack.z.clone();
        state.stack.z = state.stack.y.clone();
        state.stack.y = state.stack.x.clone();
    }
    state.stack.x = HpNum::rounded(d);
    state.stack.lift_enabled = true;
}

fn push_decimal(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).unwrap();
    dispatch(state, Op::PushNum(HpNum::rounded(d))).unwrap();
}

fn x_val(state: &CalcState) -> Decimal {
    state.stack.x.inner()
}

// ── PRGM mode recording ──────────────────────────────────────────────────────

#[test]
fn test_prgm_mode_toggle() {
    let mut s = CalcState::new();
    assert!(!s.prgm_mode, "prgm_mode must be false on new()");
    dispatch(&mut s, Op::PrgmMode).unwrap();
    assert!(s.prgm_mode, "PrgmMode must set prgm_mode = true");
    dispatch(&mut s, Op::PrgmMode).unwrap();
    assert!(!s.prgm_mode, "Second PrgmMode must exit prgm_mode (toggle)");
}

#[test]
fn test_prgm_mode_records_ops() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::PrgmMode).unwrap(); // enter recording
    dispatch(&mut s, Op::Add).unwrap();
    assert_eq!(s.program.len(), 1, "One op must be recorded");
    assert_eq!(s.program[0], Op::Add, "Recorded op must be Op::Add");
    // Stack must be unchanged — Add was NOT executed
    assert_eq!(
        x_val(&s),
        Decimal::ZERO,
        "X must be unchanged (Add not executed)"
    );
}

#[test]
fn test_prgm_mode_does_not_push_stack() {
    let mut s = CalcState::new();
    push(&mut s, 7); // X = 7 before recording
    dispatch(&mut s, Op::PrgmMode).unwrap();
    dispatch(&mut s, Op::Clx).unwrap(); // recorded, not executed
    assert_eq!(
        x_val(&s),
        Decimal::from(7),
        "X must still be 7; CLX was recorded not executed"
    );
}

#[test]
fn test_prgm_mode_exit_not_recorded() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::PrgmMode).unwrap(); // enter
    dispatch(&mut s, Op::Add).unwrap(); // recorded
    let len_before = s.program.len();
    dispatch(&mut s, Op::PrgmMode).unwrap(); // exit — must NOT be recorded
    assert!(
        !s.prgm_mode,
        "prgm_mode must be false after second PrgmMode"
    );
    assert_eq!(
        s.program.len(),
        len_before,
        "PrgmMode exit must not append to program"
    );
}

#[test]
fn test_prgm_mode_records_pushnum_via_entry_buf() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::PrgmMode).unwrap();
    s.entry_buf = "5".to_string();
    dispatch(&mut s, Op::Add).unwrap(); // flush entry_buf → Op::PushNum(5) recorded, then Op::Add
    assert_eq!(
        s.program.len(),
        2,
        "entry_buf flush must record PushNum before Add"
    );
    assert!(
        matches!(s.program[0], Op::PushNum(_)),
        "First recorded op must be PushNum"
    );
    assert_eq!(s.program[1], Op::Add, "Second recorded op must be Add");
}

// ── Label and branch ─────────────────────────────────────────────────────────

#[test]
fn test_run_program_basic_lbl_rtn() {
    let mut s = CalcState::new();
    let d42 = Decimal::from(42);
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::rounded(d42)),
        Op::Rtn,
    ];
    run_program(&mut s, "A").unwrap();
    assert_eq!(
        x_val(&s),
        Decimal::from(42),
        "X must be 42 after running program A"
    );
}

#[test]
fn test_run_unknown_label_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(run_program(&mut s, "X"), Err(HpError::InvalidOp));
}

#[test]
fn test_gto_within_program() {
    let mut s = CalcState::new();
    // Program: Lbl("A"), Gto("END"), PushNum(99) [skipped], Lbl("END"), PushNum(1), Rtn
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Gto("END".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from(99))), // must be skipped
        Op::Lbl("END".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from(1))),
        Op::Rtn,
    ];
    run_program(&mut s, "A").unwrap();
    assert_eq!(
        x_val(&s),
        Decimal::from(1),
        "GTO must skip PushNum(99); X must be 1"
    );
}

#[test]
fn test_gto_unknown_label_returns_invalid_op() {
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Gto("MISSING".to_string()),
        Op::Rtn,
    ];
    assert_eq!(run_program(&mut s, "A"), Err(HpError::InvalidOp));
}

// ── Subroutine calls ─────────────────────────────────────────────────────────

#[test]
fn test_xeq_and_rtn() {
    let mut s = CalcState::new();
    // Program A calls B which pushes 42, then B returns to A which pushes 1
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Xeq("B".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from(1))),
        Op::Rtn,
        Op::Lbl("B".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from(42))),
        Op::Rtn,
    ];
    run_program(&mut s, "A").unwrap();
    // Stack after: X=1 (pushed last by A), Y=42 (pushed by B)
    assert_eq!(x_val(&s), Decimal::from(1), "X must be 1 (last push in A)");
    assert_eq!(
        s.stack.y.inner(),
        Decimal::from(42),
        "Y must be 42 (pushed by B)"
    );
}

#[test]
fn test_rtn_at_top_level_terminates() {
    let mut s = CalcState::new();
    s.program = vec![Op::Lbl("A".to_string()), Op::Rtn];
    // Top-level RTN must return Ok(()), not an error
    assert_eq!(run_program(&mut s, "A"), Ok(()));
}

#[test]
fn test_xeq_nesting_4_levels_succeeds() {
    let mut s = CalcState::new();
    // A→B→C→D→E: 4 nested XEQ calls (A is level 0, B=1, C=2, D=3, E=4th push)
    // call_stack after XEQ into E has 4 entries → allowed (D-14)
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Xeq("B".to_string()),
        Op::Rtn,
        Op::Lbl("B".to_string()),
        Op::Xeq("C".to_string()),
        Op::Rtn,
        Op::Lbl("C".to_string()),
        Op::Xeq("D".to_string()),
        Op::Rtn,
        Op::Lbl("D".to_string()),
        Op::Xeq("E".to_string()),
        Op::Rtn,
        Op::Lbl("E".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from(99))),
        Op::Rtn,
    ];
    // 4 levels of nesting must succeed
    assert_eq!(run_program(&mut s, "A"), Ok(()));
    assert_eq!(x_val(&s), Decimal::from(99));
}

#[test]
fn test_xeq_5th_level_returns_call_depth() {
    let mut s = CalcState::new();
    // A→B→C→D→E→F: 5th XEQ (into F) must return CallDepth
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Xeq("B".to_string()),
        Op::Rtn,
        Op::Lbl("B".to_string()),
        Op::Xeq("C".to_string()),
        Op::Rtn,
        Op::Lbl("C".to_string()),
        Op::Xeq("D".to_string()),
        Op::Rtn,
        Op::Lbl("D".to_string()),
        Op::Xeq("E".to_string()),
        Op::Rtn,
        Op::Lbl("E".to_string()),
        Op::Xeq("F".to_string()), // 5th XEQ — must fail (D-13)
        Op::Rtn,
        Op::Lbl("F".to_string()),
        Op::Rtn,
    ];
    assert_eq!(run_program(&mut s, "A"), Err(HpError::CallDepth));
}

// ── Conditional tests ────────────────────────────────────────────────────────

#[test]
fn test_test_x_eq_zero_true_executes_next() {
    let mut s = CalcState::new();
    // X=0; Test(XEqZero) is true → execute next step (PushNum(1))
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Test(TestKind::XEqZero),
        Op::PushNum(HpNum::rounded(Decimal::from(1))), // executed when condition true
        Op::Rtn,
    ];
    // X starts as 0 (CalcState::new())
    run_program(&mut s, "A").unwrap();
    assert_eq!(
        x_val(&s),
        Decimal::from(1),
        "Condition true: next step executes (D-09)"
    );
}

#[test]
fn test_test_x_eq_zero_false_skips_next() {
    let mut s = CalcState::new();
    push(&mut s, 1); // X = 1 (non-zero)
                     // Test(XEqZero) is false → skip next step (PushNum(99)), execute PushNum(2)
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Test(TestKind::XEqZero),
        Op::PushNum(HpNum::rounded(Decimal::from(99))), // skipped when condition false
        Op::PushNum(HpNum::rounded(Decimal::from(2))),  // executed
        Op::Rtn,
    ];
    run_program(&mut s, "A").unwrap();
    assert_eq!(
        x_val(&s),
        Decimal::from(2),
        "Condition false: next step skipped (D-09)"
    );
}

#[test]
fn test_all_12_test_kinds_basic() {
    // Verify each TestKind returns the correct bool for a known input
    struct Case {
        kind: TestKind,
        x: i64,
        y: i64,
        expected: bool,
    }
    let cases = vec![
        Case {
            kind: TestKind::XEqZero,
            x: 0,
            y: 0,
            expected: true,
        },
        Case {
            kind: TestKind::XEqZero,
            x: 1,
            y: 0,
            expected: false,
        },
        Case {
            kind: TestKind::XNeZero,
            x: 1,
            y: 0,
            expected: true,
        },
        Case {
            kind: TestKind::XNeZero,
            x: 0,
            y: 0,
            expected: false,
        },
        Case {
            kind: TestKind::XLtZero,
            x: -1,
            y: 0,
            expected: true,
        },
        Case {
            kind: TestKind::XLtZero,
            x: 0,
            y: 0,
            expected: false,
        },
        Case {
            kind: TestKind::XGtZero,
            x: 1,
            y: 0,
            expected: true,
        },
        Case {
            kind: TestKind::XGtZero,
            x: 0,
            y: 0,
            expected: false,
        },
        Case {
            kind: TestKind::XLeZero,
            x: 0,
            y: 0,
            expected: true,
        },
        Case {
            kind: TestKind::XLeZero,
            x: 1,
            y: 0,
            expected: false,
        },
        Case {
            kind: TestKind::XGeZero,
            x: 0,
            y: 0,
            expected: true,
        },
        Case {
            kind: TestKind::XGeZero,
            x: -1,
            y: 0,
            expected: false,
        },
        Case {
            kind: TestKind::XEqY,
            x: 3,
            y: 3,
            expected: true,
        },
        Case {
            kind: TestKind::XEqY,
            x: 3,
            y: 4,
            expected: false,
        },
        Case {
            kind: TestKind::XNeY,
            x: 3,
            y: 4,
            expected: true,
        },
        Case {
            kind: TestKind::XNeY,
            x: 3,
            y: 3,
            expected: false,
        },
        Case {
            kind: TestKind::XLtY,
            x: 2,
            y: 3,
            expected: true,
        },
        Case {
            kind: TestKind::XLtY,
            x: 3,
            y: 3,
            expected: false,
        },
        Case {
            kind: TestKind::XGtY,
            x: 4,
            y: 3,
            expected: true,
        },
        Case {
            kind: TestKind::XGtY,
            x: 3,
            y: 3,
            expected: false,
        },
        Case {
            kind: TestKind::XLeY,
            x: 3,
            y: 3,
            expected: true,
        },
        Case {
            kind: TestKind::XLeY,
            x: 4,
            y: 3,
            expected: false,
        },
        Case {
            kind: TestKind::XGeY,
            x: 3,
            y: 3,
            expected: true,
        },
        Case {
            kind: TestKind::XGeY,
            x: 2,
            y: 3,
            expected: false,
        },
    ];
    for case in cases {
        let mut s = CalcState::new();
        push(&mut s, case.y); // Y first (push lifts)
        push(&mut s, case.x); // X on top
        use hp41_core::ops::program::evaluate_test;
        let result = evaluate_test(&s, &case.kind);
        assert_eq!(
            result, case.expected,
            "TestKind::{:?} with X={} Y={} expected={}",
            case.kind, case.x, case.y, case.expected
        );
    }
}

// ── ISG/DSE counter (PROG-02) ────────────────────────────────────────────────

#[test]
fn test_isg_increments_4_times_before_skip() {
    // Canonical success criterion: R00=1.00500 (current=1, final=5, step=1).
    // Loop body executes 4 times (current 1→2→3→4→5), then 5+1=6>5 → skip.
    let mut s = CalcState::new();
    let counter = Decimal::from_str("1.00500").unwrap();
    s.regs[0] = HpNum::rounded(counter);

    // Program: Lbl("LOOP"), StoArith+R01 [count iterations], Isg(0), Gto("LOOP"), Rtn
    // X = 1 (used as addend in StoArith) to accumulate cleanly.
    push(&mut s, 1); // X = 1 (used as addend in StoArith)

    s.program = vec![
        Op::Lbl("LOOP".to_string()),
        Op::StoArith {
            reg: 1,
            kind: hp41_core::ops::StoArithKind::Add,
        }, // R01 += X (X=1)
        Op::Isg(0),                  // increment R00, skip if > final
        Op::Gto("LOOP".to_string()), // branch back if not skipping
        Op::Rtn,
    ];
    run_program(&mut s, "LOOP").unwrap();

    // R01 must be 5.0 (loop body ran 5 times).
    // Loop structure: body(StoArith) runs BEFORE ISG checks. ISG increments: 1→2→3→4→5→6.
    // ISG skips when new_current > final (6 > 5). Body runs 5 times (at current 1,2,3,4,5).
    // [Rule 1 fix: plan comment "4 times" was wrong; with body-before-ISG structure, body
    //  runs on the same pass as the skipping ISG call — correct HP-41 ISG > semantics]
    assert_eq!(s.regs[1].inner(), Decimal::from(5),
        "ISG loop body runs 5 times: body before ISG, skip triggers at new_current 6 > final 5 (PROG-02)");
}

#[test]
fn test_isg_step_zero_treated_as_one() {
    let mut s = CalcState::new();
    // Counter with step=00 — step must be treated as 1 (D-10)
    let counter = Decimal::from_str("3.00500").unwrap(); // current=3, final=5, step=00→1
    s.regs[0] = HpNum::rounded(counter);
    push(&mut s, 1);
    s.program = vec![
        Op::Lbl("L".to_string()),
        Op::StoArith {
            reg: 1,
            kind: hp41_core::ops::StoArithKind::Add,
        },
        Op::Isg(0),
        Op::Gto("L".to_string()),
        Op::Rtn,
    ];
    run_program(&mut s, "L").unwrap();
    // current starts at 3: 3+1=4 (4>5? No), 4+1=5 (5>5? No), 5+1=6 (6>5? Yes, skip). Body runs 3 times.
    assert_eq!(
        s.regs[1].inner(),
        Decimal::from(3),
        "Step 00 treated as 1; loop body runs 3 times (3→4→5→6>5 skip)"
    );
}

#[test]
fn test_isg_counter_string_round_trip() {
    // Verify parse_counter handles rust_decimal trailing-zero normalisation
    // "1.00500" stored as HpNum may normalise to "1.005" internally.
    // parse_counter must still return (current=1, final=5, step=1).
    let mut s = CalcState::new();
    let counter = Decimal::from_str("1.00500").unwrap();
    s.regs[0] = HpNum::rounded(counter);
    // Run one ISG — if parsing is wrong, the loop count will be off
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Isg(0),
        Op::PushNum(HpNum::rounded(Decimal::from(1))), // executed when NOT skipping
        Op::Rtn,
        Op::Rtn, // executed when skipping (pc skips PushNum, lands on second Rtn)
    ];
    // current=1, step=1 → new_current=2. 2 > 5? No → do not skip (PushNum executes).
    run_program(&mut s, "A").unwrap();
    assert_eq!(
        x_val(&s),
        Decimal::from(1),
        "First ISG: current 1→2, not yet > 5, must not skip"
    );
}

#[test]
fn test_dse_decrements_until_skip() {
    let mut s = CalcState::new();
    // R00 = 3.00100 (current=3, final=1, step=1)
    let counter = Decimal::from_str("3.00100").unwrap();
    s.regs[0] = HpNum::rounded(counter);
    push(&mut s, 1);
    s.program = vec![
        Op::Lbl("L".to_string()),
        Op::StoArith {
            reg: 1,
            kind: hp41_core::ops::StoArithKind::Add,
        },
        Op::Dse(0),
        Op::Gto("L".to_string()),
        Op::Rtn,
    ];
    run_program(&mut s, "L").unwrap();
    // DSE: 3-1=2 (2<=1? No, continue), 2-1=1 (1<=1? Yes, skip).
    // Loop body runs 2 times (at current 3 and 2).
    assert_eq!(
        s.regs[1].inner(),
        Decimal::from(2),
        "DSE loop body runs 2 times (3→2, 2→1≤1 skip)"
    );
}

// ── Integration / state management ───────────────────────────────────────────

#[test]
fn test_is_running_reset_on_completion() {
    let mut s = CalcState::new();
    s.program = vec![Op::Lbl("A".to_string()), Op::Rtn];
    run_program(&mut s, "A").unwrap();
    assert!(
        !s.is_running,
        "is_running must be false after successful completion"
    );
}

#[test]
fn test_is_running_reset_on_error() {
    let mut s = CalcState::new();
    // Program that hits CallDepth error
    s.program = vec![
        Op::Lbl("A".to_string()),
        Op::Xeq("B".to_string()),
        Op::Rtn,
        Op::Lbl("B".to_string()),
        Op::Xeq("C".to_string()),
        Op::Rtn,
        Op::Lbl("C".to_string()),
        Op::Xeq("D".to_string()),
        Op::Rtn,
        Op::Lbl("D".to_string()),
        Op::Xeq("E".to_string()),
        Op::Rtn,
        Op::Lbl("E".to_string()),
        Op::Xeq("F".to_string()), // 5th XEQ = CallDepth
        Op::Rtn,
        Op::Lbl("F".to_string()),
        Op::Rtn,
    ];
    let result = run_program(&mut s, "A");
    assert_eq!(result, Err(HpError::CallDepth));
    assert!(
        !s.is_running,
        "is_running must be false even after error (D-06 safety reset)"
    );
}

#[test]
fn test_full_program_via_dispatch_recording() {
    // Record a program via prgm_mode, then run it
    let mut s = CalcState::new();
    // Enter PRGM mode
    dispatch(&mut s, Op::PrgmMode).unwrap();
    // Record: Lbl("A"), PushNum(10), PushNum(20), Add, Rtn
    dispatch(&mut s, Op::Lbl("A".to_string())).unwrap();
    s.entry_buf = "10".to_string();
    dispatch(&mut s, Op::Enter).unwrap(); // flush 10 as PushNum, record Enter
    s.entry_buf = "20".to_string();
    dispatch(&mut s, Op::Add).unwrap(); // flush 20 as PushNum, record Add
    dispatch(&mut s, Op::Rtn).unwrap();
    // Exit PRGM mode
    dispatch(&mut s, Op::PrgmMode).unwrap();
    assert!(!s.prgm_mode, "Must be back in execute mode");
    // Run the recorded program
    run_program(&mut s, "A").unwrap();
    assert_eq!(x_val(&s), Decimal::from(30), "10 + 20 = 30");
}

// Suppress unused variable warning for push_decimal helper (used in more complex tests)
#[allow(dead_code)]
fn _use_push_decimal(state: &mut CalcState, s: &str) {
    push_decimal(state, s);
}
