//! Integration tests for Phase 20 (Core Math & Conversions).
//!
//! Covers the four ROADMAP success criteria (SC-1..SC-4) plus per-op happy
//! and error paths for the 10 new ops: PI, P→R, R→P, RND, FRC, MOD, ABS,
//! FACT, SIGN, R↑.
//!
//! SC-5 (the 4-place Op-variant rule) is enforced at compile time by the
//! exhaustive matches in Tasks 2/3/5 — no runtime test needed here.
//!
//! Decisions honored: D-01 .. D-25. Note that D-14 was UPDATED 2026-05-13
//! to "sign follows Y" via trunc-toward-zero (FN-MATH-06 updated in parallel
//! in REQUIREMENTS.md). The three MOD tests below are deterministic per the
//! corrected formula `Y - X * trunc(Y/X)`.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{AngleMode, CalcState, DisplayMode, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Helpers ────────────────────────────────────────────────────────────────

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).expect("valid decimal literal in test")
}

fn push_x(state: &mut CalcState, s: &str) {
    dispatch(state, Op::PushNum(HpNum::from(dec(s)))).unwrap();
}

fn push_y_then_x(state: &mut CalcState, y: &str, x: &str) {
    dispatch(state, Op::PushNum(HpNum::from(dec(y)))).unwrap();
    dispatch(state, Op::Enter).unwrap();
    dispatch(state, Op::PushNum(HpNum::from(dec(x)))).unwrap();
}

/// Tolerance for f64-bridge ops (PolarToRect / RectToPolar). HpNum::rounded
/// keeps 10 sig digits, so 1e-7 is comfortably looser than the rounding floor.
fn close_enough(actual: Decimal, expected: Decimal) -> bool {
    let diff = (actual - expected).abs();
    diff < dec("0.0000001")
}

// ── SC-1: PI lifts the stack and pushes 3.141592654 ────────────────────────

#[test]
fn test_pi_pushes_ten_digit_rounded_and_lifts_stack() {
    // Covers SC-1, FN-MATH-01, D-08, D-10.
    let mut state = CalcState::new();
    dispatch(&mut state, Op::Pi).unwrap();
    assert_eq!(
        state.stack.x.inner(),
        dec("3.141592654"),
        "PI must push the 10-sig-digit hardware value",
    );
    // Second PI: previous PI value must have rolled into Y (stack lifted).
    dispatch(&mut state, Op::Pi).unwrap();
    assert_eq!(state.stack.x.inner(), dec("3.141592654"));
    assert_eq!(
        state.stack.y.inner(),
        dec("3.141592654"),
        "second PI must lift the prior PI into Y",
    );
}

// ── P→R / R→P (FN-MATH-02, FN-MATH-03, D-11..D-13) ─────────────────────────

#[test]
fn test_polar_to_rect_deg_mode_three_four_at_5313() {
    // Y = 3 (magnitude), X = 53.13010235 (degrees) → Y ≈ 1.8, X ≈ 2.4
    // (3·cos(53.13°) ≈ 1.8, 3·sin(53.13°) ≈ 2.4).
    let mut state = CalcState::new();
    state.angle_mode = AngleMode::Deg;
    push_y_then_x(&mut state, "3", "53.13010235");
    dispatch(&mut state, Op::PolarToRect).unwrap();
    assert!(
        close_enough(state.stack.y.inner(), dec("1.8")),
        "P→R Y (x-coord) ≈ 1.8, got {}",
        state.stack.y.inner(),
    );
    assert!(
        close_enough(state.stack.x.inner(), dec("2.4")),
        "P→R X (y-coord) ≈ 2.4, got {}",
        state.stack.x.inner(),
    );
}

#[test]
fn test_rect_to_polar_deg_mode_three_four_to_5_at_5313() {
    // Y = 3 (x-coord), X = 4 (y-coord) → Y = 5 (magnitude), X ≈ 53.13010235°
    // Covers SC-2 part-a.
    let mut state = CalcState::new();
    state.angle_mode = AngleMode::Deg;
    push_y_then_x(&mut state, "3", "4");
    dispatch(&mut state, Op::RectToPolar).unwrap();
    assert!(
        close_enough(state.stack.y.inner(), dec("5")),
        "R→P Y (magnitude) = 5, got {}",
        state.stack.y.inner(),
    );
    assert!(
        close_enough(state.stack.x.inner(), dec("53.13010235")),
        "R→P X (angle in deg) ≈ 53.13010235, got {}",
        state.stack.x.inner(),
    );
}

#[test]
fn test_rect_to_polar_rad_mode() {
    // Same coords as above but in RAD mode → angle ≈ 0.927295218 rad.
    // Covers SC-2 part-b.
    let mut state = CalcState::new();
    state.angle_mode = AngleMode::Rad;
    push_y_then_x(&mut state, "3", "4");
    dispatch(&mut state, Op::RectToPolar).unwrap();
    assert!(
        close_enough(state.stack.y.inner(), dec("5")),
        "R→P magnitude = 5 in RAD mode, got {}",
        state.stack.y.inner(),
    );
    assert!(
        close_enough(state.stack.x.inner(), dec("0.927295218")),
        "R→P angle ≈ 0.927295218 rad, got {}",
        state.stack.x.inner(),
    );
}

// ── RND (SC-3 part-a, FN-MATH-04, D-01..D-03) ──────────────────────────────

#[test]
fn test_rnd_fix_one_keeps_one_decimal_for_negative_value() {
    // FIX 1 of -5.7 → -5.7 (the input already matches the FIX-1 precision).
    let mut state = CalcState::new();
    state.display_mode = DisplayMode::Fix(1);
    push_x(&mut state, "-5.7");
    dispatch(&mut state, Op::Rnd).unwrap();
    assert_eq!(state.stack.x.inner(), dec("-5.7"));
}

#[test]
fn test_rnd_fix_zero_rounds_to_integer_minus_six() {
    // FIX 0 of -5.7 → -6 (round-half-away-from-zero).
    let mut state = CalcState::new();
    state.display_mode = DisplayMode::Fix(0);
    push_x(&mut state, "-5.7");
    dispatch(&mut state, Op::Rnd).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(-6));
}

// ── FACT — happy path + error matrix (FN-MATH-08, D-04..D-07) ──────────────

#[test]
fn test_fact_five_equals_120() {
    let mut state = CalcState::new();
    push_x(&mut state, "5");
    dispatch(&mut state, Op::Fact).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(120));
    assert_eq!(
        state.stack.lastx.inner(),
        Decimal::from(5),
        "FACT must save the consumed X into LASTX",
    );
}

#[test]
fn test_fact_twenty_seven_succeeds() {
    // 27! ≈ 1.0889e28 — fits inside Decimal range (~7.9e28).
    let mut state = CalcState::new();
    push_x(&mut state, "27");
    let res = dispatch(&mut state, Op::Fact);
    assert_eq!(res, Ok(()));
    assert!(
        !state.stack.x.is_zero(),
        "FACT 27 must produce a non-zero result",
    );
}

#[test]
fn test_fact_twenty_eight_returns_overflow() {
    // 28! ≈ 3.05e29 — exceeds Decimal range, Decimal::from_f64 returns None.
    let mut state = CalcState::new();
    push_x(&mut state, "28");
    let res = dispatch(&mut state, Op::Fact);
    assert_eq!(res, Err(HpError::Overflow));
}

#[test]
fn test_fact_seventy_returns_out_of_range() {
    // SC-3 part-b: hardware-spec pre-flight wins over Overflow.
    let mut state = CalcState::new();
    push_x(&mut state, "70");
    let res = dispatch(&mut state, Op::Fact);
    assert_eq!(res, Err(HpError::OutOfRange));
}

#[test]
fn test_fact_negative_returns_domain() {
    let mut state = CalcState::new();
    push_x(&mut state, "-1");
    let res = dispatch(&mut state, Op::Fact);
    assert_eq!(res, Err(HpError::Domain));
}

#[test]
fn test_fact_non_integer_returns_domain() {
    let mut state = CalcState::new();
    push_x(&mut state, "2.5");
    let res = dispatch(&mut state, Op::Fact);
    assert_eq!(res, Err(HpError::Domain));
}

// ── MOD — sign follows Y per corrected D-14 (FN-MATH-06) ───────────────────

#[test]
fn test_mod_seven_mod_three_is_one() {
    // Trivially positive: q = 2.333…, trunc = 2, result = 7 - 3·2 = 1.
    let mut state = CalcState::new();
    push_y_then_x(&mut state, "7", "3");
    dispatch(&mut state, Op::Mod).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(1));
}

#[test]
fn test_mod_seven_mod_neg_three() {
    // D-14 example: q = -2.333…, trunc = -2, result = 7 - (-3)·(-2) = 1.
    // Sign follows Y (positive Y → positive result).
    let mut state = CalcState::new();
    push_y_then_x(&mut state, "7", "-3");
    dispatch(&mut state, Op::Mod).unwrap();
    assert_eq!(
        state.stack.x.inner(),
        Decimal::from(1),
        "7 MOD -3 = 1 per corrected D-14 (sign follows Y)",
    );
}

#[test]
fn test_mod_neg_seven_mod_three() {
    // D-14 example: q = -2.333…, trunc = -2, result = -7 - 3·(-2) = -1.
    // Sign follows Y (negative Y → negative result).
    let mut state = CalcState::new();
    push_y_then_x(&mut state, "-7", "3");
    dispatch(&mut state, Op::Mod).unwrap();
    assert_eq!(
        state.stack.x.inner(),
        Decimal::from(-1),
        "-7 MOD 3 = -1 per corrected D-14 (sign follows Y)",
    );
}

#[test]
fn test_mod_div_by_zero() {
    let mut state = CalcState::new();
    push_y_then_x(&mut state, "5", "0");
    let res = dispatch(&mut state, Op::Mod);
    assert_eq!(res, Err(HpError::Domain));
}

// ── FRC / ABS / SIGN ───────────────────────────────────────────────────────

#[test]
fn test_frc_negative_three_seven_is_minus_zero_seven() {
    // FN-MATH-05, D-15: FRC(-3.7) = -0.7 (sign-preserving complement of INT).
    let mut state = CalcState::new();
    push_x(&mut state, "-3.7");
    dispatch(&mut state, Op::Frc).unwrap();
    assert_eq!(state.stack.x.inner(), dec("-0.7"));
}

#[test]
fn test_abs_negative_five_is_five() {
    // FN-MATH-07, D-16: ABS(-5) = 5; LASTX captured.
    let mut state = CalcState::new();
    push_x(&mut state, "-5");
    dispatch(&mut state, Op::Abs).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(5));
    assert_eq!(state.stack.lastx.inner(), Decimal::from(-5));
}

#[test]
fn test_sign_negative_zero_positive() {
    // FN-MATH-09, D-17: -1 / 0 / +1.
    // Case (a): negative input.
    let mut state = CalcState::new();
    push_x(&mut state, "-5");
    dispatch(&mut state, Op::Sign).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(-1));

    // Case (b): zero input.
    let mut state = CalcState::new();
    push_x(&mut state, "0");
    dispatch(&mut state, Op::Sign).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::ZERO);

    // Case (c): positive input.
    let mut state = CalcState::new();
    push_x(&mut state, "5");
    dispatch(&mut state, Op::Sign).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(1));
}

// ── SC-4: R↑ mirrors Rdn ────────────────────────────────────────────────────

#[test]
fn test_rup_mirrors_rdn() {
    // SC-4, FN-STACK-01, D-19, D-20: starting from {X=1, Y=2, Z=3, T=4},
    // R↑ produces {X=4, Y=1, Z=2, T=3} and LASTX is unchanged.
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(1);
    state.stack.y = HpNum::from(2);
    state.stack.z = HpNum::from(3);
    state.stack.t = HpNum::from(4);
    let prior_lastx = state.stack.lastx.clone();
    dispatch(&mut state, Op::Rup).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(4));
    assert_eq!(state.stack.y.inner(), Decimal::from(1));
    assert_eq!(state.stack.z.inner(), Decimal::from(2));
    assert_eq!(state.stack.t.inner(), Decimal::from(3));
    assert_eq!(
        state.stack.lastx, prior_lastx,
        "R↑ must NOT update LASTX (D-19)",
    );
}
