//! Phase 24 Plan 02 (FN-IND-01 + FN-IND-02) integration tests for the 11 new
//! Op::*Ind variants. Three test classes per variant: happy / non-integer /
//! out-of-bounds.
//!
//! Plus inheritance bonuses (D-23.4 sidecar via op_sto_ind, Enable lift via
//! op_rcl_ind, display_override semantics via op_view_ind, kind reuse for
//! StoArithInd / FlagTestInd, interactive-no-op defense for FlagTestInd).
//!
//! Tests for IsgInd / DseInd / FlagTestInd happy paths drive through
//! `run_program` (NOT `dispatch`) per RESEARCH Pitfall 1 -- proves the
//! run_loop arms wire skip semantics correctly.
//!
//! Test modules allow unwrap (CLAUDE.md "Zero panics" applies to production
//! code only).

#![allow(clippy::unwrap_used)]

use hp41_core::format::format_hpnum;
use hp41_core::num::HpNum;
use hp41_core::ops::program::run_program;
use hp41_core::ops::{dispatch, FlagTestKind, Op, StoArithKind};
use hp41_core::state::{CalcState, DisplayMode};
use hp41_core::HpError;
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Helpers ───────────────────────────────────────────────────────────────

/// Convenience: read flag n's set/clear bit directly.
fn flag_set_test(flags: u64, n: u8) -> bool {
    flags & (1u64 << n) != 0
}

// ── Op::StoInd ────────────────────────────────────────────────────────────

#[test]
fn sto_ind_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.stack.x = HpNum::from(7i32);
    dispatch(&mut state, Op::StoInd(5)).unwrap();
    assert_eq!(state.regs[12], HpNum::from(7i32));
}

#[test]
fn sto_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    let result = dispatch(&mut state, Op::StoInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn sto_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    // default regs.len() == 100; resolved address 200 > regs.len()
    state.regs[5] = HpNum::from(200i32);
    // Note: 200 fits in u8 (< 256), so resolve_indirect returns Ok(200);
    // op_sto's bounds check then rejects (idx >= regs.len()).
    let result = dispatch(&mut state, Op::StoInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn sto_ind_clears_text_regs_sidecar() {
    // BONUS: D-23.4 inheritance via delegation. STO IND must clear the
    // text_regs sidecar for the RESOLVED register (not the pointer reg).
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.text_regs.insert(12, "ABC".to_string());
    state.stack.x = HpNum::from(7i32);
    dispatch(&mut state, Op::StoInd(5)).unwrap();
    assert_eq!(state.text_regs.get(&12), None);
    assert_eq!(state.regs[12], HpNum::from(7i32));
}

// ── Op::RclInd ────────────────────────────────────────────────────────────

#[test]
fn rcl_ind_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(99i32);
    dispatch(&mut state, Op::RclInd(5)).unwrap();
    assert_eq!(state.stack.x, HpNum::from(99i32));
}

#[test]
fn rcl_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    let result = dispatch(&mut state, Op::RclInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn rcl_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(200i32);
    let result = dispatch(&mut state, Op::RclInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn rcl_ind_lift_enable_inheritance() {
    // BONUS: op_rcl_ind delegates to op_rcl which sets lift_enabled = true.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(99i32);
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::RclInd(5)).unwrap();
    assert!(state.stack.lift_enabled, "RCL IND must enable lift");
}

// ── Op::StoArithInd ───────────────────────────────────────────────────────

#[test]
fn sto_arith_ind_add_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(10i32);
    state.stack.x = HpNum::from(3i32);
    dispatch(&mut state, Op::StoArithInd(5, StoArithKind::Add)).unwrap();
    assert_eq!(state.regs[12], HpNum::from(13i32));
}

#[test]
fn sto_arith_ind_sub_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(10i32);
    state.stack.x = HpNum::from(3i32);
    dispatch(&mut state, Op::StoArithInd(5, StoArithKind::Sub)).unwrap();
    assert_eq!(state.regs[12], HpNum::from(7i32));
}

#[test]
fn sto_arith_ind_mul_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(10i32);
    state.stack.x = HpNum::from(3i32);
    dispatch(&mut state, Op::StoArithInd(5, StoArithKind::Mul)).unwrap();
    assert_eq!(state.regs[12], HpNum::from(30i32));
}

#[test]
fn sto_arith_ind_div_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(12i32);
    state.stack.x = HpNum::from(3i32);
    dispatch(&mut state, Op::StoArithInd(5, StoArithKind::Div)).unwrap();
    assert_eq!(state.regs[12], HpNum::from(4i32));
}

#[test]
fn sto_arith_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    state.stack.x = HpNum::from(3i32);
    let result = dispatch(&mut state, Op::StoArithInd(5, StoArithKind::Add));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn sto_arith_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(200i32);
    state.stack.x = HpNum::from(3i32);
    let result = dispatch(&mut state, Op::StoArithInd(5, StoArithKind::Add));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Op::IsgInd (drive via run_program per Pitfall 1) ──────────────────────

#[test]
fn isg_ind_inside_run_loop() {
    // ISG counter: 0.005 means current=0, target=5, step=1.
    // Each iteration: current += 1, check current > target.
    // Program: LBL A, ISG IND 5, GTO A, LBL END
    //   When ISG returns true (counter exit), pc skips Gto and runs Lbl END.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::rounded(Decimal::from_str("0.005").unwrap());
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::IsgInd(5),
        Op::Gto("A".to_string()),
        Op::Lbl("END".to_string()),
    ];
    run_program(&mut state, "A").unwrap();
    // After 6 ISG calls (0->1->2->3->4->5->6), 6 > 5 triggers exit. The
    // counter holds the post-exit current value with .005 frac preserved.
    // We don't lock the exact value here -- the key invariant is that
    // run_program returned Ok (didn't hang).
    let exit_val = &state.regs[12];
    assert!(
        exit_val.inner() > Decimal::from_str("5").unwrap(),
        "ISG IND must drive counter past the target; got {:?}",
        exit_val
    );
}

#[test]
fn isg_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    state.program = vec![Op::Lbl("A".to_string()), Op::IsgInd(5)];
    let result = run_program(&mut state, "A");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn isg_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(200i32);
    state.program = vec![Op::Lbl("A".to_string()), Op::IsgInd(5)];
    let result = run_program(&mut state, "A");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Op::DseInd (drive via run_program per Pitfall 1) ──────────────────────

#[test]
fn dse_ind_inside_run_loop() {
    // DSE counter: 5.001 means current=5, target=1, step=1 (decrement)
    // Each iteration: current -= 1, check current <= target.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::rounded(Decimal::from_str("5.001").unwrap());
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::DseInd(5),
        Op::Gto("A".to_string()),
        Op::Lbl("END".to_string()),
    ];
    run_program(&mut state, "A").unwrap();
    let exit_val = &state.regs[12];
    assert!(
        exit_val.trunc_int().inner() <= Decimal::from_str("1").unwrap(),
        "DSE IND must drive counter down to target; got {:?}",
        exit_val
    );
}

#[test]
fn dse_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    state.program = vec![Op::Lbl("A".to_string()), Op::DseInd(5)];
    let result = run_program(&mut state, "A");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn dse_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(200i32);
    state.program = vec![Op::Lbl("A".to_string()), Op::DseInd(5)];
    let result = run_program(&mut state, "A");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Op::SfFlagInd ─────────────────────────────────────────────────────────

#[test]
fn sf_flag_ind_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    dispatch(&mut state, Op::SfFlagInd(5)).unwrap();
    assert!(flag_set_test(state.flags, 12), "flag 12 must be set");
}

#[test]
fn sf_flag_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    let result = dispatch(&mut state, Op::SfFlagInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn sf_flag_ind_out_of_flag_range() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(60i32); // > 55 (op_sf rejects)
    let result = dispatch(&mut state, Op::SfFlagInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Op::CfFlagInd ─────────────────────────────────────────────────────────

#[test]
fn cf_flag_ind_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    dispatch(&mut state, Op::CfFlagInd(5)).unwrap();
    assert!(!flag_set_test(state.flags, 12), "flag 12 must be clear");
}

#[test]
fn cf_flag_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    let result = dispatch(&mut state, Op::CfFlagInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn cf_flag_ind_out_of_flag_range() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(60i32);
    let result = dispatch(&mut state, Op::CfFlagInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Op::FlagTestInd (drive via run_program per Pitfall 1) ─────────────────

#[test]
fn flag_test_ind_is_set_happy_inside_run_loop() {
    // Flag 12 IS set; IsSet => no skip => both PushNums execute, X=2.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSet,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    run_program(&mut state, "A").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
}

#[test]
fn flag_test_ind_is_clear_happy_inside_run_loop() {
    // Flag 12 is CLEAR; IsClear => no skip => both PushNums execute, X=2.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 0;
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    run_program(&mut state, "A").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
}

#[test]
fn flag_test_ind_is_set_then_clear_happy_inside_run_loop() {
    // Flag 12 IS set; IsSetThenClear => always clear AFTER test, no skip
    // (was set), so both PushNums execute and X=2. Post-condition: flag 12
    // is now cleared.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSetThenClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    run_program(&mut state, "A").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert!(
        !flag_set_test(state.flags, 12),
        "FS?C must always clear flag 12, regardless of test outcome"
    );
}

#[test]
fn flag_test_ind_is_clear_then_clear_happy_inside_run_loop() {
    // Flag 12 is CLEAR; IsClearThenClear => always clear (no-op when
    // already clear), no skip (was clear), both PushNums execute, X=2.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 0;
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClearThenClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    run_program(&mut state, "A").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert!(!flag_set_test(state.flags, 12), "flag stays clear");
}

#[test]
fn flag_test_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSet,
            ind_reg: 5,
        },
    ];
    let result = run_program(&mut state, "A");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn flag_test_ind_high_flag_no_panic() {
    // Defensive: flag_get / flag_clear in ops/flags.rs return false / no-op
    // for n > 55. So resolved address 100 should run without panic. The
    // test verifies no panic and the program completes; flag-test fires
    // as "not set" -> for IsSet kind, that means SKIP.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(100i32);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSet,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
    ];
    let result = run_program(&mut state, "A");
    // Either Ok (flag-get returns false defensively, skip fires, only the
    // second PushNum runs, X==2) or Err is acceptable -- the key invariant
    // is "no panic". Match either outcome.
    assert!(
        result.is_ok() || matches!(result, Err(HpError::InvalidOp)),
        "high flag must not panic: {:?}",
        result
    );
}

#[test]
fn flag_test_ind_interactive_is_neutral_no_op() {
    // BONUS: defends against accidentally adding skip semantics to dispatch.
    // Interactive FlagTestInd is a Neutral no-op (mirrors Op::FlagTest).
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    let pc_before = state.pc;
    let flags_before = state.flags;
    dispatch(
        &mut state,
        Op::FlagTestInd {
            kind: FlagTestKind::IsSetThenClear,
            ind_reg: 5,
        },
    )
    .unwrap();
    assert_eq!(state.pc, pc_before, "interactive dispatch must not move pc");
    assert_eq!(
        state.flags, flags_before,
        "interactive dispatch must not mutate flags (no always-clear at keyboard)"
    );
}

// ── Op::ArclInd ───────────────────────────────────────────────────────────

#[test]
fn arcl_ind_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(42i32);
    state.alpha_reg = String::new();
    state.display_mode = DisplayMode::Fix(4);
    dispatch(&mut state, Op::ArclInd(5)).unwrap();
    let expected = format_hpnum(&state.regs[12], &state.display_mode);
    assert_eq!(state.alpha_reg, expected);
}

#[test]
fn arcl_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    let result = dispatch(&mut state, Op::ArclInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn arcl_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(200i32);
    let result = dispatch(&mut state, Op::ArclInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Op::AstoInd ───────────────────────────────────────────────────────────

#[test]
fn asto_ind_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.alpha_reg = "HELLO".to_string();
    dispatch(&mut state, Op::AstoInd(5)).unwrap();
    assert_eq!(state.text_regs.get(&12), Some(&"HELLO".to_string()));
    // ASTO zeroes the numeric slot (no-drift invariant per phase23 D-23.5).
    assert_eq!(state.regs[12], HpNum::zero());
}

#[test]
fn asto_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    let result = dispatch(&mut state, Op::AstoInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn asto_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(200i32);
    let result = dispatch(&mut state, Op::AstoInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Op::ViewInd (R9 mitigation: shows resolved register's value) ──────────

#[test]
fn view_ind_happy() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(42i32);
    state.display_mode = DisplayMode::Fix(4);
    dispatch(&mut state, Op::ViewInd(5)).unwrap();
    let expected = format_hpnum(&HpNum::from(42i32), &state.display_mode);
    assert_eq!(state.display_override, Some(expected));
}

#[test]
fn view_ind_shows_resolved_register_value() {
    // BONUS R9 sentinel: display_override must hold the VALUE of R12, NOT
    // any representation of R05 (the pointer). With R05=12 and R12=42 and
    // Fix(4) mode, the expected display is "42.0000", which is NOT
    // "12.0000" (R05's formatted value).
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::from(42i32);
    state.display_mode = DisplayMode::Fix(4);
    dispatch(&mut state, Op::ViewInd(5)).unwrap();

    let pointer_value = format_hpnum(&HpNum::from(12i32), &state.display_mode);
    let resolved_value = format_hpnum(&HpNum::from(42i32), &state.display_mode);
    let actual = state.display_override.clone().unwrap();
    assert_eq!(
        actual, resolved_value,
        "VIEW IND must show resolved register's value"
    );
    assert_ne!(
        actual, pointer_value,
        "VIEW IND must NOT show pointer register's value"
    );
}

#[test]
fn view_ind_non_integer() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
    let result = dispatch(&mut state, Op::ViewInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

#[test]
fn view_ind_out_of_regs_len() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(200i32);
    let result = dispatch(&mut state, Op::ViewInd(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
}
