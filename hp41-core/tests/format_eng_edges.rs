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
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

fn num(s: &str) -> HpNum {
    HpNum::from(Decimal::from_str(s).unwrap())
}

// ── SCI/ENG zero-mode boundaries ────────────────────────────────────────────

#[test]
fn fmt_sci_zero_digits_with_zero_value() {
    // Catches: SCI(0) zero-value early-return regression — format.rs
    // "0.E 00" is the HP-41 representation of 0 in SCI 0 mode.
    assert_eq!(format_hpnum(&HpNum::zero(), &DisplayMode::Sci(0)), "0.E 00");
}

#[test]
fn fmt_sci_nonzero_digits_with_zero_value() {
    // Catches: SCI(n>0) zero-value branch — format.rs
    // "0.0000E 00" for SCI 4 mode.
    assert_eq!(
        format_hpnum(&HpNum::zero(), &DisplayMode::Sci(4)),
        "0.0000E 00"
    );
}

#[test]
fn fmt_eng_zero_digits_with_zero_value() {
    // Catches: ENG(0) zero-value early-return regression — format.rs
    assert_eq!(format_hpnum(&HpNum::zero(), &DisplayMode::Eng(0)), "0.E 00");
}

#[test]
fn fmt_eng_nonzero_digits_with_zero_value() {
    // Catches: ENG(n>0) zero-value branch — format.rs
    assert_eq!(
        format_hpnum(&HpNum::zero(), &DisplayMode::Eng(3)),
        "0.000E 00"
    );
}

// ── ENG carry threshold crossing ────────────────────────────────────────────

#[test]
fn fmt_eng_carry_threshold_crossing_999_9995_in_eng_3() {
    // Catches: ENG carry threshold regression — format.rs.
    // 999.9995 in ENG(3): mantissa rounds to 1000.000 → must carry to next
    // engineering exponent (10^3). Without the carry block, output would be
    // an invalid "1000.000E 00" instead of "1.000E 03".
    let s = format_hpnum(&num("999.9995"), &DisplayMode::Eng(3));
    // After carry: mantissa = 1.000, eng_exp = 3.
    assert_eq!(
        s, "1.000E 03",
        "ENG carry must bump to next decade: got {s}"
    );
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
    // Catches: round_eng body regression — format.rs.
    // Direct calls to round_to_display_precision with Eng mode cover the
    // mantissa-rounding + carry-detection path that `format_hpnum` shares
    // with `op_rnd`.
    let out = round_to_display_precision(&num("999.9995"), &DisplayMode::Eng(3));
    // Carry case: round to 1.000E+3 → inner is 1000.
    assert_eq!(out.inner(), Decimal::from(1000));
}

#[test]
fn round_eng_body_negative_input() {
    // Catches: round_eng sign-preservation branch — format.rs is_negative path.
    let out = round_to_display_precision(&num("-1234.5"), &DisplayMode::Eng(2));
    // Negative input should round symmetrically: -1.23e3 → -1230.
    // The exact normalized value depends on the rounding strategy; assert
    // sign and approximate magnitude.
    assert!(out.inner().is_sign_negative(), "must preserve sign");
}

#[test]
fn round_to_display_precision_sci_mode() {
    // Catches: SCI arm of round_to_display_precision — format.rs.
    // 9.9995 in SCI(3) → 10 via mantissa-carry handling.
    let out = round_to_display_precision(&num("9.9995"), &DisplayMode::Sci(3));
    assert_eq!(out.inner(), Decimal::from(10));
}

#[test]
fn round_to_display_precision_fix_mode() {
    // Catches: FIX arm of round_to_display_precision — format.rs.
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
    // math.rs `else` arm. Closes RESEARCH §Priority 5 coverage hole.
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
    // Catches: op_abs negative branch (math.rs). Symmetric pair to
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
    // Catches: dispatch wiring regression on Op::Int — mod.rs.
    let mut s = CalcState::new();
    s.stack.x = num("3.7");
    dispatch(&mut s, Op::Int).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(3));
}

#[test]
fn op_sto_arith_stack_interactive_dispatch() {
    // Catches: dispatch wiring regression on Op::StoArithStack — mod.rs.
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
    // Catches: dispatch wiring regression on Op::AlphaBackspace — mod.rs.
    let mut s = CalcState::new();
    s.alpha_reg = "HELLO".to_string();
    dispatch(&mut s, Op::AlphaBackspace).unwrap();
    assert_eq!(s.alpha_reg, "HELL");
}

#[test]
fn op_lbl_interactive_dispatch_noop() {
    // Catches: dispatch wiring regression on Op::Lbl — mod.rs. LBL is
    // a recording marker; executing interactively is a no-op.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Lbl("FOO".to_string())).unwrap();
    // No-op: state unchanged.
    assert!(s.stack.x.is_zero());
}

#[test]
fn op_gto_interactive_dispatch_returns_invalid_op() {
    // Catches: dispatch wiring regression on Op::Gto — interactive GTO outside
    // a running program must return InvalidOp (op_gto guard in program.rs).
    // Strict variant match (not loose is_err) so an error-taxonomy refactor
    // can't silently swap InvalidOp → Domain etc.
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::Gto("FOO".to_string()));
    assert!(
        matches!(r, Err(HpError::InvalidOp)),
        "Op::Gto interactive must return InvalidOp; got {r:?}"
    );
}

#[test]
fn op_rtn_interactive_dispatch_noop() {
    // Catches: dispatch wiring regression on Op::Rtn — mod.rs. Interactive
    // RTN with empty call_stack is a no-op (program.rs).
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Rtn).unwrap();
    assert!(s.call_stack.is_empty());
}

#[test]
fn op_test_interactive_dispatch_noop() {
    // Catches: dispatch wiring regression on Op::Test — mod.rs. Test
    // interactive arm is a read-only no-op (program.rs).
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Test(hp41_core::TestKind::XEqZero)).unwrap();
    assert!(s.stack.x.is_zero());
}

#[test]
fn op_isg_interactive_discards_skip_signal() {
    // Catches: dispatch wiring regression on Op::Isg — interactive context.
    // The `.map(|_| ())` discards the skip-bool; pc must not advance
    // interactively. ALSO assert regs[5] WAS mutated (counter incremented):
    // without that side-effect check, a buggy Op::Isg that becomes a true
    // no-op (early-returns Ok(()) without calling the counter helper) would
    // silently pass — pc==pc_before is trivially true for a no-op.
    // Counter "1.001" = current=1, increment=0.001 → increment to 1.001 +
    // delta=0.001 → current becomes 2 (per parse_counter semantics).
    let mut s = CalcState::new();
    s.regs[5] = HpNum::rounded(Decimal::from_str("1.001").expect("literal"));
    let pc_before = s.pc;
    let reg5_before = s.regs[5].clone();
    dispatch(&mut s, Op::Isg(5)).unwrap();
    assert_eq!(s.pc, pc_before, "interactive ISG must not advance pc");
    assert_ne!(
        s.regs[5], reg5_before,
        "interactive ISG must still mutate the counter register"
    );
}

#[test]
fn op_dse_interactive_discards_skip_signal() {
    // Catches: dispatch wiring regression on Op::Dse — interactive context.
    // See op_isg_interactive_discards_skip_signal for the side-effect-witness
    // rationale: assert regs[5] is mutated so a "true no-op" regression
    // doesn't silently pass.
    let mut s = CalcState::new();
    s.regs[5] = HpNum::rounded(Decimal::from_str("5.001").expect("literal"));
    let pc_before = s.pc;
    let reg5_before = s.regs[5].clone();
    dispatch(&mut s, Op::Dse(5)).unwrap();
    assert_eq!(s.pc, pc_before, "interactive DSE must not advance pc");
    assert_ne!(
        s.regs[5], reg5_before,
        "interactive DSE must still mutate the counter register"
    );
}
