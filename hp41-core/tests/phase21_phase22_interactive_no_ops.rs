#![allow(clippy::unwrap_used)]

//! Phase 21 / Phase 22 interactive-dispatch no-op invariants.
//!
//! Phase 21 and Phase 22 introduced ops that are intentionally NO-OPS in
//! the interactive (non-program-context) dispatch path: `Op::FlagTest`,
//! `Op::Stop`, `Op::Pse`, `Op::Prompt`, `Op::GtoInd`, `Op::XeqInd`,
//! `Op::IsgInd`, `Op::DseInd`, `Op::FlagTestInd`. The design invariant is:
//! pc unchanged, flags unchanged (FlagTest is read-only), lift_enabled
//! Neutral. This file documents and locks that invariant.
//!
//! **Bug class caught:** accidental conversion of an interactive no-op
//! into a state-mutating dispatch (e.g. an interactive Pse that pushes
//! "PAUSE 1000" twice — once in the interactive arm and once in the
//! run_loop arm).
//!
//! Risk-weighted Priority 3 per RESEARCH §Risk-Weighted Uncovered-Line
//! Inventory: targets `ops/mod.rs` lines 804/822/831/811/839/911/912/915.

use hp41_core::ops::{dispatch, FlagTestKind, Op};
use hp41_core::{CalcState, HpError};

/// Build a fresh CalcState and capture the fields that any "Neutral lift,
/// pc unchanged" interactive no-op must preserve.
fn neutral_capture() -> (CalcState, usize, u64, bool) {
    let s = CalcState::new();
    let pc = s.pc;
    let flags = s.flags;
    let lift = s.stack.lift_enabled;
    (s, pc, flags, lift)
}

// ── Op::FlagTest interactive no-op (all 4 kinds) ────────────────────────────
// See also: phase21_flags.rs::test_op_flag_test_interactive_dispatch_is_no_op
// for the IsSet baseline — this file expands to all 4 kinds.

#[test]
fn op_flag_test_isset_interactive_neutral() {
    // Catches: accidental skip / mutation on interactive FS? — covers
    // mod.rs:804 Op::FlagTest { .. } => Neutral no-op arm.
    let (mut s, pc, flags, _) = neutral_capture();
    dispatch(
        &mut s,
        Op::FlagTest {
            kind: FlagTestKind::IsSet,
            flag: 5,
        },
    )
    .unwrap();
    assert_eq!(s.pc, pc);
    assert_eq!(s.flags, flags);
}

#[test]
fn op_flag_test_isclear_interactive_neutral() {
    // Catches: accidental skip / mutation on interactive FC?.
    let (mut s, pc, flags, _) = neutral_capture();
    dispatch(
        &mut s,
        Op::FlagTest {
            kind: FlagTestKind::IsClear,
            flag: 5,
        },
    )
    .unwrap();
    assert_eq!(s.pc, pc);
    assert_eq!(s.flags, flags);
}

#[test]
fn op_flag_test_issetclear_interactive_neutral() {
    // Catches: accidental ALWAYS-CLEAR side effect on interactive FS?C.
    // The always-clear semantic lives in run_loop ONLY (mod.rs:804); a
    // future regression that wires the clear into the interactive arm
    // would break this test.
    let (mut s, pc, _, _) = neutral_capture();
    s.flags = 1u64 << 5;
    dispatch(
        &mut s,
        Op::FlagTest {
            kind: FlagTestKind::IsSetThenClear,
            flag: 5,
        },
    )
    .unwrap();
    assert_eq!(s.pc, pc);
    // CRITICAL: flag 5 must STILL be set — always-clear is run_loop-only.
    assert_ne!(
        s.flags & (1u64 << 5),
        0,
        "Interactive FS?C must NOT clear the flag (Pitfall 3)"
    );
}

#[test]
fn op_flag_test_isclearclear_interactive_neutral() {
    // Catches: accidental ALWAYS-CLEAR side effect on interactive FC?C.
    let (mut s, pc, _, _) = neutral_capture();
    s.flags = 1u64 << 5;
    dispatch(
        &mut s,
        Op::FlagTest {
            kind: FlagTestKind::IsClearThenClear,
            flag: 5,
        },
    )
    .unwrap();
    assert_eq!(s.pc, pc);
    // CRITICAL: flag 5 must STILL be set — always-clear is run_loop-only.
    assert_ne!(s.flags & (1u64 << 5), 0);
}

// ── Op::Stop / Op::Pse / Op::Prompt / Op::AView interactive arms ────────────

#[test]
fn op_stop_interactive_neutral() {
    // Catches: accidental side effect on interactive STOP — covers mod.rs:822.
    // STOP is a run_loop-only break primitive; interactively it is a Neutral
    // no-op (D-22.5).
    let (mut s, pc, flags, lift) = neutral_capture();
    dispatch(&mut s, Op::Stop).unwrap();
    assert_eq!(s.pc, pc);
    assert_eq!(s.flags, flags);
    assert_eq!(s.stack.lift_enabled, lift);
    // STOP does NOT write display_override (unlike PROMPT).
    assert!(s.display_override.is_none());
}

#[test]
fn op_pse_interactive_writes_pause_to_display_override() {
    // Catches: missing PSE side effect in interactive dispatch — covers
    // mod.rs:829-834. Pse DOES write display_override + event_buffer
    // interactively (NOT a pure no-op); the run_loop break behavior is
    // a separate semantic. This test pins the interactive side effect.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Pse).unwrap();
    assert!(
        s.display_override.is_some(),
        "PSE must write display_override interactively"
    );
    assert!(
        s.event_buffer.iter().any(|e| e.contains("PAUSE 1000")),
        "PSE must push PAUSE 1000 to event_buffer"
    );
}

#[test]
fn op_prompt_interactive_arm_exists() {
    // Catches: regression on Op::Prompt interactive arm — covers mod.rs:811.
    // Interactively, PROMPT writes ALPHA to display_override and does NOT
    // panic. The run_loop break behavior is a separate semantic.
    let mut s = CalcState::new();
    s.alpha_reg = "HI".to_string();
    dispatch(&mut s, Op::Prompt).unwrap();
    assert_eq!(s.display_override.as_deref(), Some("HI"));
}

// ── Op::GtoInd / Op::XeqInd interactive arms (return InvalidOp) ─────────────

#[test]
fn op_gto_ind_interactive_returns_invalid_op() {
    // Catches: regression on Op::GtoInd interactive arm — covers
    // mod.rs:839. GTO IND requires run_loop state-machine context; outside
    // it, the op returns InvalidOp.
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::GtoInd(5));
    assert!(matches!(r, Err(HpError::InvalidOp)));
}

#[test]
fn op_xeq_ind_interactive_returns_invalid_op() {
    // Catches: regression on Op::XeqInd interactive arm — covers
    // mod.rs:839 (shared with GtoInd via `|`-pattern).
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::XeqInd(5));
    assert!(matches!(r, Err(HpError::InvalidOp)));
}

// ── Op::IsgInd / Op::DseInd interactive arms (discard skip bool) ────────────

#[test]
fn op_isg_ind_interactive_discards_skip_signal() {
    // Catches: accidental pc advance on interactive IsgInd — covers
    // mod.rs:911. The `.map(|_| ())` arm discards the skip-bool; pc must
    // not advance even if op_isg_ind returned true (counter exit).
    let mut s = CalcState::new();
    s.regs[5] = hp41_core::HpNum::from(7i32);
    // Set regs[7] to a counter at the boundary so op_isg_ind would
    // return true (skip) — but interactively that signal must be ignored.
    s.regs[7] = hp41_core::HpNum::rounded(rust_decimal::Decimal::from_str_exact("5.005").unwrap());
    let pc_before = s.pc;
    dispatch(&mut s, Op::IsgInd(5)).unwrap();
    assert_eq!(s.pc, pc_before, "interactive IsgInd must not advance pc");
}

#[test]
fn op_dse_ind_interactive_discards_skip_signal() {
    // Catches: accidental pc advance on interactive DseInd — covers
    // mod.rs:912.
    let mut s = CalcState::new();
    s.regs[5] = hp41_core::HpNum::from(7i32);
    s.regs[7] = hp41_core::HpNum::rounded(rust_decimal::Decimal::from_str_exact("1.000").unwrap());
    let pc_before = s.pc;
    dispatch(&mut s, Op::DseInd(5)).unwrap();
    assert_eq!(s.pc, pc_before, "interactive DseInd must not advance pc");
}

// ── Op::FlagTestInd interactive arm (Neutral no-op) ─────────────────────────

#[test]
fn op_flag_test_ind_interactive_neutral() {
    // Catches: accidental skip / always-clear side effect on interactive
    // FS?C IND — covers mod.rs:915. Mirrors the Op::FlagTest interactive
    // arm; ind_reg pointer is irrelevant because the arm is a pure no-op.
    let mut s = CalcState::new();
    s.flags = 1u64 << 5;
    s.regs[7] = hp41_core::HpNum::from(5i32);
    let pc_before = s.pc;
    let flags_before = s.flags;
    dispatch(
        &mut s,
        Op::FlagTestInd {
            kind: FlagTestKind::IsSetThenClear,
            ind_reg: 7,
        },
    )
    .unwrap();
    assert_eq!(s.pc, pc_before);
    // Interactive arm is Neutral — flags MUST be unchanged.
    assert_eq!(s.flags, flags_before);
}
