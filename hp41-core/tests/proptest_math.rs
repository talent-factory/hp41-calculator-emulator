#![allow(clippy::unwrap_used)]
#![allow(clippy::approx_constant)]

//! Property-based tests for FN-QUAL-02 shape invariants.
//!
//! Complements the hand-curated `numerical_accuracy.rs` extension
//! (Plan 27-01) by asserting general algebraic truths the hand cases
//! sample only at specific points. Iteration counts per D-27.11: 256
//! cases per block (math involves `rust_decimal` arithmetic — slower
//! than flag bit-twiddling).
//!
//! Shape invariants per D-27.5:
//! 1. FRC + INT round-trip: FRC(x) + INT(x) ≈ x
//! 2. MOD sign-follows-Y (HP-41 hardware semantics, NOT Rust % semantics)
//! 3. FACT(n+1) ≈ FACT(n) × (n+1) for n in 0..=68
//! 4. P→R/R→P round-trip in DEG mode within tolerance
//! 5. RND idempotency: RND(RND(x)) = RND(x) across FIX/SCI/ENG modes

use hp41_core::ops::{dispatch, Op};
use hp41_core::state::DisplayMode;
use hp41_core::{CalcState, HpNum};
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

// ─── Helpers ───────────────────────────────────────────────────────────────────

/// Generate a Decimal within HP-41's representable range.
///
/// Strategy: random sign × mantissa (1..10^10, 10 hardware digits) × exponent.
/// The effective exponent range is clamped to ±18 because `10i64.pow(n)` would
/// overflow for n > 18 — `Decimal` cannot represent values outside this range
/// regardless. The HP-41 hardware advertises -99..+99 but the relevant proptest
/// surface is the algorithmic correctness of FRC/INT/MOD/FACT/RND/P↔R, which
/// the ±18 effective range covers thoroughly.
fn arb_hp_decimal() -> impl Strategy<Value = Decimal> {
    (any::<bool>(), 1u64..10_000_000_000u64, -18i32..=18i32).prop_map(|(neg, mantissa, exp)| {
        let mut d = Decimal::from(mantissa);
        if neg {
            d.set_sign_negative(true);
        }
        if exp >= 0 {
            d * Decimal::from(10i64.pow(exp as u32))
        } else {
            d / Decimal::from(10i64.pow((-exp) as u32))
        }
    })
}

/// Tolerance-based comparison helper (mirrors numerical_accuracy.rs:58).
fn passes_with_tol(actual: f64, expected: f64, tol: f64) -> bool {
    if actual.is_nan() || expected.is_nan() {
        return false;
    }
    if expected == 0.0 {
        actual.abs() <= tol
    } else {
        ((actual - expected) / expected).abs() <= tol
    }
}

// ─── Property 1: FRC + INT round-trip ─────────────────────────────────────────
//
// `Op::Int` (truncate toward zero) and `Op::Frc` (fractional part, sign-
// preserving) together decompose any x: x = INT(x) + FRC(x). The proptest
// asserts this fundamental shape invariant across the full HP-41 range.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Catches: FRC or INT regression that breaks the fundamental
    // decomposition x = INT(x) + FRC(x). E.g. if FRC returns the
    // wrong sign on negative inputs (returns |FRC(x)| instead of
    // sign-preserving), this property fires immediately.
    #[test]
    fn frc_plus_int_equals_x(d in arb_hp_decimal()) {
        let mut s_frc = CalcState::new();
        s_frc.stack.x = HpNum::from(d);
        dispatch(&mut s_frc, Op::Frc).unwrap();
        let frc_part = s_frc.stack.x.inner().to_f64().unwrap_or(f64::NAN);

        let mut s_int = CalcState::new();
        s_int.stack.x = HpNum::from(d);
        dispatch(&mut s_int, Op::Int).unwrap();
        let int_part = s_int.stack.x.inner().to_f64().unwrap_or(f64::NAN);

        let original = d.to_f64().unwrap_or(f64::NAN);
        prop_assert!(
            passes_with_tol(frc_part + int_part, original, 1e-9),
            "FRC({}) + INT({}) = {} ≠ {} (within 1e-9)",
            d, d, frc_part + int_part, original
        );
    }
}

// ─── Property 2: MOD sign-follows-Y (HP-41 hardware) ──────────────────────────
//
// HP-41 MOD differs from Rust's `%`: the result's sign follows Y, not X.
// E.g. MOD(7, -3) = 1 on HP-41 (matches Free42); Rust `7 % -3 = 1` but
// `-7 % 3 = -1`. The proptest asserts the sign invariant across random
// (Y, X) magnitude × sign combinations.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Catches: accidental Rust `%` semantics (sign-follows-X) in
    // op_mod. HP-41 hardware uses sign-follows-Y (cross-referenced
    // against Free42 source ops_math.cc::do_mod — HP-41C Owner's
    // Manual p.234).
    #[test]
    fn mod_sign_follows_y(
        y_mag in 1i64..1_000_000,
        x_mag in 1i64..1000,
        y_neg in any::<bool>(),
        x_neg in any::<bool>(),
    ) {
        let y_val = if y_neg { -y_mag } else { y_mag };
        let x_val = if x_neg { -x_mag } else { x_mag };
        let mut s = CalcState::new();
        s.stack.y = HpNum::from(Decimal::from(y_val));
        s.stack.x = HpNum::from(Decimal::from(x_val));
        dispatch(&mut s, Op::Mod).unwrap();
        let result = s.stack.x.inner();
        if !result.is_zero() {
            // Sign of result must match sign of original Y.
            prop_assert_eq!(
                result.is_sign_negative(), y_neg,
                "MOD({}, {}) = {} — sign should follow Y (y_neg={})",
                y_val, x_val, result, y_neg
            );
        }
        // exact-divisible case (result == 0) is sign-agnostic — skip the check.
    }
}

// ─── Property 3: FACT recursive invariant FACT(n+1) ≈ FACT(n) × (n+1) ─────────
//
// FACT(0) = 1, FACT(n+1) = FACT(n) × (n+1). HP-41 hardware-spec OutOfRange
// fires at X > 69 (Owner's Manual p.234), but the practical Decimal
// representable-range wall is X ≤ 27 (op_fact D-05 comment in math.rs
// — Decimal::from_f64 returns Overflow for f64 factorials of 28..=69).
// To keep both FACT(n) and FACT(n+1) representable, the proptest range
// is n in 0..=26 (so n+1 ≤ 27, the safe interior).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Catches: off-by-one or sign regression in op_fact's inner
    // multiplication. Range deviation from PLAN.md: range narrowed
    // from 0..=68 to 0..=26 because op_fact returns Overflow for
    // n in 28..=69 (per D-05 / math.rs). Hand-curated tests
    // (Plan 27-01) exercise the boundary individually.
    #[test]
    fn fact_recursive_invariant(n in 0i32..=26i32) {
        let mut s_n = CalcState::new();
        s_n.stack.x = HpNum::from(n);
        dispatch(&mut s_n, Op::Fact).unwrap();
        let fact_n = s_n.stack.x.inner().to_f64().unwrap_or(f64::NAN);

        let mut s_n1 = CalcState::new();
        s_n1.stack.x = HpNum::from(n + 1);
        dispatch(&mut s_n1, Op::Fact).unwrap();
        let fact_n1 = s_n1.stack.x.inner().to_f64().unwrap_or(f64::NAN);

        // FACT(n+1) ≈ FACT(n) × (n+1) within HP-41 10-digit tolerance.
        // Tolerance widens to 1e-8 to absorb 10-digit BCD rounding compounding
        // across the recursive multiplication chain.
        prop_assert!(
            passes_with_tol(fact_n1, fact_n * (n + 1) as f64, 1e-8),
            "FACT({}) = {}, FACT({}) = {}, expected ≈ {}",
            n, fact_n, n + 1, fact_n1, fact_n * (n + 1) as f64
        );
    }
}

// ─── Property 4: P→R/R→P round-trip in DEG mode ───────────────────────────────
//
// R→P (rect to polar) then P→R (polar to rect) must return the original
// (x, y) within tolerance. Four trig calls compound 10-digit BCD rounding;
// the WIDE_TOL (1e-6) absorbs the compounding.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Catches: P↔R conversion regressions — the round-trip identity
    // P→R(R→P(x,y)) ≈ (x,y) is a fundamental shape invariant. Sampled
    // in DEG mode (the default). Tolerance accounts for HP-41 10-digit
    // rounding compounding across 4 trig calls.
    #[test]
    fn polar_rect_round_trip_in_deg_mode(
        x in -1000i32..=1000i32,
        y in -1000i32..=1000i32,
    ) {
        prop_assume!(!(x == 0 && y == 0)); // degenerate origin
        let mut s = CalcState::new();
        dispatch(&mut s, Op::SetDeg).unwrap();
        // HP-41 P→R/R→P convention: Y holds first input, X holds second.
        // Set up the (Y=y, X=x) pair, then apply R→P → P→R, then read back.
        s.stack.y = HpNum::from(y);
        s.stack.x = HpNum::from(x);
        dispatch(&mut s, Op::RectToPolar).unwrap();
        dispatch(&mut s, Op::PolarToRect).unwrap();
        let x_back = s.stack.x.inner().to_f64().unwrap_or(f64::NAN);
        let y_back = s.stack.y.inner().to_f64().unwrap_or(f64::NAN);
        prop_assert!(
            passes_with_tol(x_back, x as f64, 1e-6),
            "P→R(R→P(x={}, y={})) X-back = {} ≠ {} (within 1e-6)",
            x, y, x_back, x
        );
        prop_assert!(
            passes_with_tol(y_back, y as f64, 1e-6),
            "P→R(R→P(x={}, y={})) Y-back = {} ≠ {} (within 1e-6)",
            x, y, y_back, y
        );
    }
}

// ─── Property 5: RND idempotency across FIX/SCI/ENG modes ─────────────────────
//
// RND(x) rounds X to the current display-mode precision. Applying RND a
// second time must produce the same result (the first call already
// truncated to the precision). RESEARCH Pitfall 8: test on the underlying
// value, not the display string — display-mode 0-digit formatting has
// special cases the underlying value doesn't share.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Catches: RND not actually rounding (no-op) or RND that produces
    // different output on the second call (e.g. trailing-digit drift
    // from BCD->f64->BCD conversion). RESEARCH Pitfall 8: test on the
    // value, not the display string — display-mode 0-digit formatting
    // has special cases the underlying value doesn't share.
    #[test]
    fn rnd_is_idempotent_in_all_display_modes(
        d in arb_hp_decimal(),
        digits in 0u8..=9,
        mode in prop_oneof![Just(0u8), Just(1u8), Just(2u8)],
    ) {
        let mut s = CalcState::new();
        s.display_mode = match mode {
            0 => DisplayMode::Fix(digits),
            1 => DisplayMode::Sci(digits),
            _ => DisplayMode::Eng(digits),
        };
        s.stack.x = HpNum::from(d);
        dispatch(&mut s, Op::Rnd).unwrap();
        let after_first = s.stack.x.clone();
        dispatch(&mut s, Op::Rnd).unwrap();
        prop_assert_eq!(after_first.inner(), s.stack.x.inner());
    }
}
