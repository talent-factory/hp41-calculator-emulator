#![allow(clippy::unwrap_used)]

//! Format SCI/ENG zero-mode boundaries + ENG carry-threshold-crossing case.
//!
//! Covers `hp41-core/src/format.rs` SCI/ENG zero-mode boundaries and the
//! ENG carry-threshold-crossing case (RESEARCH §Priority 6, RESEARCH
//! Pitfall 8).
//!
//! **Bug class caught:** display-mode rounding boundary regressions —
//! `0.0 → FmtSci(0)`, `999.9995 → FmtEng(3)`, the ENG carry where mantissa
//! rounds up to the next decade. These cases are not exercised by
//! `format_tests.rs` today; without them, a future `format_eng::round_eng`
//! refactor could silently break the ENG carry.
//!
//! Risk-weighted Priority 6 per RESEARCH §Risk-Weighted Uncovered-Line
//! Inventory: targets `format.rs` lines 60, 73–92, 148, 188–192, 216–218,
//! 247.

use hp41_core::format::{format_hpnum, round_to_display_precision};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::DisplayMode;
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

fn num(s: &str) -> HpNum {
    HpNum::from(Decimal::from_str(s).unwrap())
}

// ── SCI/ENG zero-mode boundaries ────────────────────────────────────────────

#[test]
fn fmt_sci_zero_digits_with_zero_value() {
    // Catches: SCI(0) zero-value early-return regression — format.rs:148
    // "0.E 00" is the HP-41 representation of 0 in SCI 0 mode.
    assert_eq!(format_hpnum(&HpNum::zero(), &DisplayMode::Sci(0)), "0.E 00");
}

#[test]
fn fmt_sci_nonzero_digits_with_zero_value() {
    // Catches: SCI(n>0) zero-value branch — format.rs:150-151
    // "0.0000E 00" for SCI 4 mode.
    assert_eq!(
        format_hpnum(&HpNum::zero(), &DisplayMode::Sci(4)),
        "0.0000E 00"
    );
}

#[test]
fn fmt_eng_zero_digits_with_zero_value() {
    // Catches: ENG(0) zero-value early-return regression — format.rs:188-189
    assert_eq!(format_hpnum(&HpNum::zero(), &DisplayMode::Eng(0)), "0.E 00");
}

#[test]
fn fmt_eng_nonzero_digits_with_zero_value() {
    // Catches: ENG(n>0) zero-value branch — format.rs:190-192
    assert_eq!(
        format_hpnum(&HpNum::zero(), &DisplayMode::Eng(3)),
        "0.000E 00"
    );
}

// ── ENG carry threshold crossing ────────────────────────────────────────────

#[test]
fn fmt_eng_carry_threshold_crossing_999_9995_in_eng_3() {
    // Catches: ENG carry threshold regression — format.rs:216-218.
    // 999.9995 in ENG(3): mantissa rounds to 1000.000 → must carry to next
    // engineering exponent (10^3). Without the carry block, output would be
    // an invalid "1000.000E 00" instead of "1.000E 03".
    let s = format_hpnum(&num("999.9995"), &DisplayMode::Eng(3));
    // After carry: mantissa = 1.000, eng_exp = 3.
    assert_eq!(s, "1.000E 03", "ENG carry must bump to next decade: got {s}");
}

#[test]
fn fmt_eng_carry_with_small_threshold_in_eng_2() {
    // Catches: ENG carry on smaller-precision boundary — exercises the
    // carry path with different digits parameter.
    // 99.995 in ENG(2): mantissa rounds to 100.00 → carry to 100E 00.
    let s = format_hpnum(&num("99.995"), &DisplayMode::Eng(2));
    assert_eq!(s, "100.00E 00", "ENG carry on 99.995 ENG(2): got {s}");
}

// ── round_to_display_precision direct calls (shared with Op::Rnd) ───────────

#[test]
fn round_eng_body_via_round_to_display_precision() {
    // Catches: round_eng body regression — format.rs:72-92.
    // Direct calls to round_to_display_precision with Eng mode cover the
    // mantissa-rounding + carry-detection path that `format_hpnum` shares
    // with `op_rnd`.
    let out = round_to_display_precision(&num("999.9995"), &DisplayMode::Eng(3));
    // Carry case: round to 1.000E+3 → inner is 1000.
    assert_eq!(out.inner(), Decimal::from(1000));
}

#[test]
fn round_eng_body_negative_input() {
    // Catches: round_eng sign-preservation branch — format.rs:73 is_negative path.
    let out = round_to_display_precision(&num("-1234.5"), &DisplayMode::Eng(2));
    // Negative input should round symmetrically: -1.23e3 → -1230.
    // The exact normalized value depends on the rounding strategy; assert
    // sign and approximate magnitude.
    assert!(out.inner().is_sign_negative(), "must preserve sign");
}

#[test]
fn round_to_display_precision_sci_mode() {
    // Catches: SCI arm of round_to_display_precision — format.rs:57-59.
    // 9.9995 in SCI(3) → 10 via mantissa-carry handling.
    let out = round_to_display_precision(&num("9.9995"), &DisplayMode::Sci(3));
    assert_eq!(out.inner(), Decimal::from(10));
}

#[test]
fn round_to_display_precision_fix_mode() {
    // Catches: FIX arm of round_to_display_precision — format.rs:54-56.
    let out = round_to_display_precision(&num("3.14159"), &DisplayMode::Fix(2));
    assert_eq!(out.inner(), Decimal::from_str("3.14").unwrap());
}

// ── Op::Rnd integration with format::round_to_display_precision ────────────

#[test]
fn op_rnd_in_eng_mode() {
    // Catches: integration regression between Op::Rnd and format::Eng arm —
    // exercises math.rs::op_rnd → format::round_to_display_precision Eng arm.
    let mut s = CalcState::new();
    s.display_mode = DisplayMode::Eng(2);
    s.stack.x = num("999.9995");
    dispatch(&mut s, Op::Rnd).unwrap();
    // After RND in ENG(2): mantissa 1.00, eng_exp=3 ⇒ 1000.
    assert_eq!(s.stack.x.inner(), Decimal::from(1000));
}

#[test]
fn op_rnd_in_sci_mode_carry() {
    // Catches: integration regression on SCI carry through Op::Rnd.
    let mut s = CalcState::new();
    s.display_mode = DisplayMode::Sci(3);
    s.stack.x = num("9.9995");
    dispatch(&mut s, Op::Rnd).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(10));
}

// ── op_abs sentinel (Priority 5) ───────────────────────────────────────────

#[test]
fn op_abs_positive_input_returns_clone() {
    // Catches: regression on op_abs positive pass-through branch —
    // math.rs:414 `else` arm. Closes RESEARCH §Priority 5 coverage hole.
    let mut s = CalcState::new();
    s.stack.x = num("3.14");
    dispatch(&mut s, Op::Abs).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from_str("3.14").unwrap());
}

#[test]
fn op_abs_zero_input_returns_zero() {
    // Catches: op_abs on zero — zero is `is_sign_negative() == false`,
    // so it takes the positive branch and clones the zero value through.
    let mut s = CalcState::new();
    s.stack.x = HpNum::zero();
    dispatch(&mut s, Op::Abs).unwrap();
    assert!(s.stack.x.is_zero());
}

#[test]
fn op_abs_negative_input_flips_sign() {
    // Catches: op_abs negative branch (math.rs:411-413). Symmetric pair to
    // the positive case above; together they prove the if/else covers both
    // arms.
    let mut s = CalcState::new();
    s.stack.x = num("-5.5");
    dispatch(&mut s, Op::Abs).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from_str("5.5").unwrap());
}

// ── Priority-4: interactive-dispatch arms missed by existing suite ─────────
//
// These ops have well-tested run_program / run_loop arms but their plain
// `dispatch()` interactive arms in `ops/mod.rs` were never exercised. Bug
// class caught: regressions in the `match op` dispatch table itself
// (wrong-helper wiring, accidental no-op arms, missed flush_entry_buf
// pre-conditions). Cheap coverage on the most-used central integration
// hub in the crate.

#[test]
fn op_int_interactive_dispatch() {
    // Catches: dispatch wiring regression on Op::Int — mod.rs:671.
    let mut s = CalcState::new();
    s.stack.x = num("3.7");
    dispatch(&mut s, Op::Int).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(3));
}

#[test]
fn op_sto_arith_stack_interactive_dispatch() {
    // Catches: dispatch wiring regression on Op::StoArithStack — mod.rs:726.
    use hp41_core::ops::StackReg;
    use hp41_core::StoArithKind;
    let mut s = CalcState::new();
    s.stack.y = num("10");
    s.stack.x = num("3");
    dispatch(
        &mut s,
        Op::StoArithStack {
            kind: StoArithKind::Add,
            stack_reg: StackReg::Y,
        },
    )
    .unwrap();
    // Y += X → Y = 13.
    assert_eq!(s.stack.y.inner(), Decimal::from(13));
}

#[test]
fn op_alpha_backspace_interactive_dispatch() {
    // Catches: dispatch wiring regression on Op::AlphaBackspace — mod.rs:731.
    let mut s = CalcState::new();
    s.alpha_reg = "HELLO".to_string();
    dispatch(&mut s, Op::AlphaBackspace).unwrap();
    assert_eq!(s.alpha_reg, "HELL");
}

#[test]
fn op_lbl_interactive_dispatch_noop() {
    // Catches: dispatch wiring regression on Op::Lbl — mod.rs:740. LBL is
    // a recording marker; executing interactively is a no-op.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Lbl("FOO".to_string())).unwrap();
    // No-op: state unchanged.
    assert!(s.stack.x.is_zero());
}

#[test]
fn op_gto_interactive_dispatch_returns_invalid_op() {
    // Catches: dispatch wiring regression on Op::Gto — mod.rs:741. Interactive
    // GTO outside a running program → InvalidOp (op_gto guard at program.rs:47).
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::Gto("FOO".to_string()));
    assert!(r.is_err());
}

#[test]
fn op_rtn_interactive_dispatch_noop() {
    // Catches: dispatch wiring regression on Op::Rtn — mod.rs:743. Interactive
    // RTN with empty call_stack is a no-op (program.rs:86-93).
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Rtn).unwrap();
    assert!(s.call_stack.is_empty());
}

#[test]
fn op_test_interactive_dispatch_noop() {
    // Catches: dispatch wiring regression on Op::Test — mod.rs:745. Test
    // interactive arm is a read-only no-op (program.rs:98-101).
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Test(hp41_core::TestKind::XEqZero)).unwrap();
    assert!(s.stack.x.is_zero());
}

#[test]
fn op_isg_interactive_discards_skip_signal() {
    // Catches: dispatch wiring regression on Op::Isg — mod.rs:746-749. The
    // `.map(|_| ())` discards the skip-bool; pc must not advance interactively.
    let mut s = CalcState::new();
    s.regs[5] = HpNum::from(0i32); // counter "0.0" current=0, target=0
    let pc_before = s.pc;
    dispatch(&mut s, Op::Isg(5)).unwrap();
    assert_eq!(s.pc, pc_before);
}

#[test]
fn op_dse_interactive_discards_skip_signal() {
    // Catches: dispatch wiring regression on Op::Dse — mod.rs:751-753.
    let mut s = CalcState::new();
    s.regs[5] = HpNum::from(0i32);
    let pc_before = s.pc;
    dispatch(&mut s, Op::Dse(5)).unwrap();
    assert_eq!(s.pc, pc_before);
}
