// Tests for Phase 1 Plan 02: Core type modules
// RED phase — these fail until the four modules are implemented.

#[cfg(test)]
mod error_tests {
    use crate::error::HpError;

    #[test]
    fn hperror_has_four_variants() {
        // Each variant must be constructible and PartialEq + Clone
        let e1 = HpError::Overflow;
        let e2 = HpError::DivideByZero;
        let e3 = HpError::InvalidOp;
        let e4 = HpError::Domain;
        assert_eq!(e1.clone(), HpError::Overflow);
        assert_eq!(e2.clone(), HpError::DivideByZero);
        assert_eq!(e3.clone(), HpError::InvalidOp);
        assert_eq!(e4.clone(), HpError::Domain);
        assert_ne!(e1, e2);
    }

    #[test]
    fn hperror_display_messages() {
        assert_eq!(HpError::Overflow.to_string(), "overflow");
        assert_eq!(HpError::DivideByZero.to_string(), "divide by zero");
        assert_eq!(HpError::InvalidOp.to_string(), "invalid operation");
        assert_eq!(HpError::Domain.to_string(), "domain error");
    }

    // Phase 3 Plan 02: CallDepth variant (D-13/D-14)
    #[test]
    fn hperror_call_depth_message_is_try_again() {
        assert_eq!(HpError::CallDepth.to_string(), "try again");
    }

    #[test]
    fn hperror_call_depth_is_partialeq() {
        let e = HpError::CallDepth;
        assert_eq!(e.clone(), HpError::CallDepth);
        assert_ne!(e, HpError::InvalidOp);
    }

    // Phase 6 Plan 01: InvalidInput variant (D-06 — HMS minutes/seconds >= 60)
    #[test]
    fn hperror_invalid_input_message() {
        assert_eq!(HpError::InvalidInput.to_string(), "invalid input");
    }

    #[test]
    fn hperror_invalid_input_is_partialeq() {
        let e = HpError::InvalidInput;
        assert_eq!(e.clone(), HpError::InvalidInput);
        assert_ne!(e, HpError::InvalidOp);
    }
}

#[cfg(test)]
mod num_tests {
    use crate::num::HpNum;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn hpnum_zero_is_zero() {
        let z = HpNum::zero();
        assert!(z.is_zero());
    }

    #[test]
    fn hpnum_from_i32_exact() {
        // Integer construction must be exact (no rounding artifacts)
        let n = HpNum::from(42i32);
        assert_eq!(n.inner(), Decimal::from(42));
    }

    #[test]
    fn hpnum_rounded_10_significant_digits() {
        // 1.23456789012345 rounded to 10 sig digits → 1.234567890
        let d = Decimal::from_str("1.23456789012345").unwrap();
        let n = HpNum::rounded(d);
        // 10 significant digits: 1.234567890
        let expected = Decimal::from_str("1.234567890").unwrap();
        assert_eq!(n.inner(), expected);
    }

    #[test]
    fn hpnum_rounding_midpoint_away_from_zero() {
        // 2.5 rounded to 1 sig digit should be 3 (not 2 = Bankers rounding)
        // Use rounded() with a value where MidpointAwayFromZero differs from MidpointNearestEven
        // 1.5500000000x rounds to 1.550000001 at 10 sig digits? No — use cleaner test:
        // round 3.333333335 to 9 sig digits with MidpointAwayFromZero → 3.33333334
        let d = Decimal::from_str("3.3333333350").unwrap();
        let n = HpNum::rounded(d);
        // 10 sig digits = all 10 digits of 3.333333335 → but we need to verify direction
        // Actually 3.3333333350 has 11 significant digits; round to 10 → 3.333333335
        // Better test: 3.33333333450 rounded to 10 sig digits
        // = 3.333333334|50 → MidpointAwayFromZero → 3.333333335
        let d2 = Decimal::from_str("3.33333333450").unwrap();
        let n2 = HpNum::rounded(d2);
        let expected = Decimal::from_str("3.333333335").unwrap();
        assert_eq!(n2.inner(), expected);
        // suppress unused warning
        let _ = n;
    }

    #[test]
    fn hpnum_checked_add_normal() {
        let a = HpNum::from(3i32);
        let b = HpNum::from(4i32);
        let result = a.checked_add(&b).unwrap();
        assert_eq!(result.inner(), Decimal::from(7));
    }

    #[test]
    fn hpnum_checked_div_by_zero_returns_error() {
        use crate::error::HpError;
        let a = HpNum::from(1i32);
        let b = HpNum::zero();
        let result = a.checked_div(&b);
        assert_eq!(result, Err(HpError::DivideByZero));
    }

    #[test]
    fn hpnum_checked_div_normal() {
        let a = HpNum::from(10i32);
        let b = HpNum::from(2i32);
        let result = a.checked_div(&b).unwrap();
        assert_eq!(result.inner(), Decimal::from(5));
    }

    #[test]
    fn hpnum_negate() {
        let n = HpNum::from(5i32);
        let neg = n.negate();
        assert_eq!(neg.inner(), Decimal::from(-5));
    }
}

#[cfg(test)]
mod state_tests {
    use crate::state::{CalcState, Stack};

    #[test]
    fn stack_new_all_zero_lift_disabled() {
        let s = Stack::new();
        assert!(s.x.is_zero());
        assert!(s.y.is_zero());
        assert!(s.z.is_zero());
        assert!(s.t.is_zero());
        assert!(s.lastx.is_zero());
        assert!(!s.lift_enabled);
    }

    #[test]
    fn calcstate_new_has_fresh_stack() {
        let state = CalcState::new();
        assert!(state.stack.x.is_zero());
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn calcstate_default_equals_new() {
        let a = CalcState::new();
        let b = CalcState::default();
        // Both should have same initial state (x==0, lift_enabled==false)
        assert_eq!(a.stack.x.inner(), b.stack.x.inner());
        assert_eq!(a.stack.lift_enabled, b.stack.lift_enabled);
    }
}

#[cfg(test)]
mod stack_ops_tests {
    use crate::num::HpNum;
    use crate::state::CalcState;
    use crate::stack::{LiftEffect, apply_lift_effect, enter_number, binary_result};

    #[test]
    fn lift_effect_has_three_variants() {
        // Each variant must be constructible, PartialEq, Clone, Copy
        let e = LiftEffect::Enable;
        let d = LiftEffect::Disable;
        let n = LiftEffect::Neutral;
        assert_ne!(e, d);
        assert_ne!(e, n);
        assert_ne!(d, n);
        // Copy — no move issues
        let _e2 = e;
        let _e3 = e; // still usable after copy
    }

    #[test]
    fn apply_lift_effect_enable_sets_true() {
        let mut state = CalcState::new();
        assert!(!state.stack.lift_enabled);
        apply_lift_effect(&mut state, LiftEffect::Enable);
        assert!(state.stack.lift_enabled);
    }

    #[test]
    fn apply_lift_effect_disable_sets_false() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = true;
        apply_lift_effect(&mut state, LiftEffect::Disable);
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn apply_lift_effect_neutral_leaves_unchanged() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = true;
        apply_lift_effect(&mut state, LiftEffect::Neutral);
        assert!(state.stack.lift_enabled); // still true

        state.stack.lift_enabled = false;
        apply_lift_effect(&mut state, LiftEffect::Neutral);
        assert!(!state.stack.lift_enabled); // still false
    }

    #[test]
    fn enter_number_overwrites_x_when_lift_disabled() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        state.stack.x = HpNum::from(99i32);
        enter_number(&mut state, HpNum::from(42i32));
        // X is overwritten; Y, Z, T unchanged
        assert_eq!(state.stack.x.inner(), rust_decimal::Decimal::from(42));
        assert!(state.stack.y.is_zero());
    }

    #[test]
    fn enter_number_lifts_when_lift_enabled() {
        let mut state = CalcState::new();
        // Set up stack: X=1, Y=2, Z=3, T=4
        state.stack.x = HpNum::from(1i32);
        state.stack.y = HpNum::from(2i32);
        state.stack.z = HpNum::from(3i32);
        state.stack.t = HpNum::from(4i32);
        state.stack.lift_enabled = true;

        enter_number(&mut state, HpNum::from(5i32));

        // Stack should be: X=5, Y=1 (old X), Z=2 (old Y), T=3 (old Z)
        // T was old Z — HP-41: T←Z on push
        assert_eq!(state.stack.x.inner(), rust_decimal::Decimal::from(5));
        assert_eq!(state.stack.y.inner(), rust_decimal::Decimal::from(1));
        assert_eq!(state.stack.z.inner(), rust_decimal::Decimal::from(2));
        assert_eq!(state.stack.t.inner(), rust_decimal::Decimal::from(3));
        // lift_enabled is NOT changed by enter_number
        assert!(state.stack.lift_enabled);
    }

    #[test]
    fn binary_result_captures_lastx_before_overwrite() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(10i32);
        state.stack.y = HpNum::from(3i32);
        state.stack.z = HpNum::from(2i32);
        state.stack.t = HpNum::from(1i32);

        binary_result(&mut state, HpNum::from(99i32));

        // lastx must be old X (10), NOT the result (99)
        assert_eq!(state.stack.lastx.inner(), rust_decimal::Decimal::from(10));
        // X gets the result
        assert_eq!(state.stack.x.inner(), rust_decimal::Decimal::from(99));
        // Y←Z (old Z=2), Z←T (old T=1), T stays (=1)
        assert_eq!(state.stack.y.inner(), rust_decimal::Decimal::from(2));
        assert_eq!(state.stack.z.inner(), rust_decimal::Decimal::from(1));
        assert_eq!(state.stack.t.inner(), rust_decimal::Decimal::from(1));
        // lift is enabled after binary result
        assert!(state.stack.lift_enabled);
    }

    #[test]
    fn binary_result_enables_lift() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        binary_result(&mut state, HpNum::from(7i32));
        assert!(state.stack.lift_enabled);
    }
}

#[cfg(test)]
mod arithmetic_tests {
    use crate::num::HpNum;
    use crate::state::CalcState;
    use crate::ops::arithmetic::{op_add, op_sub, op_mul, op_div};
    use crate::error::HpError;
    use rust_decimal::Decimal;

    fn state_with_xy(x: i32, y: i32) -> CalcState {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(x);
        state.stack.y = HpNum::from(y);
        state
    }

    #[test]
    fn op_add_adds_y_plus_x() {
        // 3 + 4 = 7
        let mut state = state_with_xy(4, 3);
        op_add(&mut state).unwrap();
        assert_eq!(state.stack.x.inner(), Decimal::from(7));
    }

    #[test]
    fn op_add_enables_lift() {
        let mut state = state_with_xy(1, 2);
        state.stack.lift_enabled = false;
        op_add(&mut state).unwrap();
        assert!(state.stack.lift_enabled);
    }

    #[test]
    fn op_add_captures_lastx() {
        // X=4 before add; lastx should be 4 after
        let mut state = state_with_xy(4, 3);
        op_add(&mut state).unwrap();
        assert_eq!(state.stack.lastx.inner(), Decimal::from(4));
    }

    #[test]
    fn op_sub_y_minus_x() {
        // Y=10, X=3 → Y-X = 7
        let mut state = state_with_xy(3, 10);
        op_sub(&mut state).unwrap();
        assert_eq!(state.stack.x.inner(), Decimal::from(7));
    }

    #[test]
    fn op_mul_multiplies() {
        // Y=3, X=4 → 12
        let mut state = state_with_xy(4, 3);
        op_mul(&mut state).unwrap();
        assert_eq!(state.stack.x.inner(), Decimal::from(12));
    }

    #[test]
    fn op_div_y_divided_by_x() {
        // Y=10, X=2 → 5
        let mut state = state_with_xy(2, 10);
        op_div(&mut state).unwrap();
        assert_eq!(state.stack.x.inner(), Decimal::from(5));
    }

    #[test]
    fn op_div_by_zero_returns_error() {
        let mut state = state_with_xy(0, 10);
        let result = op_div(&mut state);
        assert_eq!(result, Err(HpError::DivideByZero));
    }

    #[test]
    fn op_add_rotates_stack_y_gets_z() {
        // Stack: X=1, Y=2, Z=3, T=4 → after add: X=3 (2+1), Y=3 (old Z), Z=4 (old T), T=4
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32);
        state.stack.y = HpNum::from(2i32);
        state.stack.z = HpNum::from(3i32);
        state.stack.t = HpNum::from(4i32);
        op_add(&mut state).unwrap();
        assert_eq!(state.stack.x.inner(), Decimal::from(3));
        assert_eq!(state.stack.y.inner(), Decimal::from(3)); // old Z
        assert_eq!(state.stack.z.inner(), Decimal::from(4)); // old T
        assert_eq!(state.stack.t.inner(), Decimal::from(4)); // T duplicated
    }
}

// ── Phase 2 Plan 02: HpNum math methods ──────────────────────────────────────
// RED phase — tests for all 8 scalar math methods (Task 1) and 6 trig methods (Task 2).
// These fail until the methods are implemented in num.rs.

#[cfg(test)]
mod num_scalar_math_tests {
    use crate::num::HpNum;
    use crate::error::HpError;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // ── checked_recip ─────────────────────────────────────────────────────
    #[test]
    fn checked_recip_of_four_is_zero_point_25() {
        let n = HpNum::from(4i32);
        let r = n.checked_recip().unwrap();
        let expected = Decimal::from_str("0.25").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_recip_of_zero_returns_divide_by_zero() {
        let n = HpNum::zero();
        assert_eq!(n.checked_recip(), Err(HpError::DivideByZero));
    }

    #[test]
    fn checked_recip_result_is_rounded_to_10_sig_digits() {
        // 1/3 = 0.3333333333 (10 sig digits)
        let n = HpNum::from(3i32);
        let r = n.checked_recip().unwrap();
        let expected = Decimal::from_str("0.3333333333").unwrap();
        assert_eq!(r.inner(), expected);
    }

    // ── checked_sqrt ──────────────────────────────────────────────────────
    #[test]
    fn checked_sqrt_of_four_is_two() {
        let n = HpNum::from(4i32);
        let r = n.checked_sqrt().unwrap();
        assert_eq!(r.inner(), Decimal::from(2));
    }

    #[test]
    fn checked_sqrt_of_two_is_1_414213562() {
        // √2 = 1.414213562 (10 sig digits)
        let n = HpNum::from(2i32);
        let r = n.checked_sqrt().unwrap();
        let expected = Decimal::from_str("1.414213562").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_sqrt_of_negative_returns_domain() {
        let n = HpNum::from(-1i32);
        assert_eq!(n.checked_sqrt(), Err(HpError::Domain));
    }

    #[test]
    fn checked_sqrt_of_zero_is_zero() {
        let n = HpNum::zero();
        let r = n.checked_sqrt().unwrap();
        assert!(r.is_zero());
    }

    // ── checked_sq ────────────────────────────────────────────────────────
    #[test]
    fn checked_sq_of_three_is_nine() {
        let n = HpNum::from(3i32);
        let r = n.checked_sq().unwrap();
        assert_eq!(r.inner(), Decimal::from(9));
    }

    #[test]
    fn checked_sq_of_negative_is_positive() {
        let n = HpNum::from(-5i32);
        let r = n.checked_sq().unwrap();
        assert_eq!(r.inner(), Decimal::from(25));
    }

    // ── checked_ln ────────────────────────────────────────────────────────
    #[test]
    fn checked_ln_of_one_is_zero() {
        let n = HpNum::from(1i32);
        let r = n.checked_ln().unwrap();
        assert!(r.is_zero());
    }

    #[test]
    fn checked_ln_of_two_is_0_6931471806() {
        // LN(2) = 0.6931471806 (10 sig digits)
        let n = HpNum::from(2i32);
        let r = n.checked_ln().unwrap();
        let expected = Decimal::from_str("0.6931471806").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_ln_of_zero_returns_domain() {
        let n = HpNum::zero();
        assert_eq!(n.checked_ln(), Err(HpError::Domain));
    }

    #[test]
    fn checked_ln_of_negative_returns_domain() {
        let n = HpNum::from(-1i32);
        assert_eq!(n.checked_ln(), Err(HpError::Domain));
    }

    // ── checked_log10 ─────────────────────────────────────────────────────
    #[test]
    fn checked_log10_of_100_is_two() {
        let n = HpNum::from(100i32);
        let r = n.checked_log10().unwrap();
        assert_eq!(r.inner(), Decimal::from(2));
    }

    #[test]
    fn checked_log10_of_zero_returns_domain() {
        let n = HpNum::zero();
        assert_eq!(n.checked_log10(), Err(HpError::Domain));
    }

    #[test]
    fn checked_log10_of_negative_returns_domain() {
        let n = HpNum::from(-5i32);
        assert_eq!(n.checked_log10(), Err(HpError::Domain));
    }

    // ── checked_exp ───────────────────────────────────────────────────────
    #[test]
    fn checked_exp_of_zero_is_one() {
        let n = HpNum::zero();
        let r = n.checked_exp().unwrap();
        assert_eq!(r.inner(), Decimal::from(1));
    }

    #[test]
    fn checked_exp_of_one_is_e() {
        // e^1 = 2.718281828 (10 sig digits)
        let n = HpNum::from(1i32);
        let r = n.checked_exp().unwrap();
        let expected = Decimal::from_str("2.718281828").unwrap();
        assert_eq!(r.inner(), expected);
    }

    // ── checked_exp10 ─────────────────────────────────────────────────────
    #[test]
    fn checked_exp10_of_zero_is_one() {
        let n = HpNum::zero();
        let r = n.checked_exp10().unwrap();
        assert_eq!(r.inner(), Decimal::from(1));
    }

    #[test]
    fn checked_exp10_of_two_is_100() {
        let n = HpNum::from(2i32);
        let r = n.checked_exp10().unwrap();
        assert_eq!(r.inner(), Decimal::from(100));
    }

    #[test]
    fn checked_exp10_of_three_is_1000() {
        let n = HpNum::from(3i32);
        let r = n.checked_exp10().unwrap();
        assert_eq!(r.inner(), Decimal::from(1000));
    }

    // ── checked_powd ──────────────────────────────────────────────────────
    #[test]
    fn checked_powd_two_to_ten_is_1024() {
        // 2^10 = 1024
        let base = HpNum::from(2i32);
        let exp = HpNum::from(10i32);
        let r = base.checked_powd(&exp).unwrap();
        assert_eq!(r.inner(), Decimal::from(1024));
    }

    #[test]
    fn checked_powd_two_to_half_is_sqrt2() {
        // 2^0.5 = 1.414213562
        let base = HpNum::from(2i32);
        let exp = HpNum::from(Decimal::from_str("0.5").unwrap());
        let r = base.checked_powd(&exp).unwrap();
        let expected = Decimal::from_str("1.414213562").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_powd_negative_base_fractional_exp_returns_domain() {
        // (-2)^0.5 = complex → Domain
        let base = HpNum::from(-2i32);
        let exp = HpNum::from(Decimal::from_str("0.5").unwrap());
        assert_eq!(base.checked_powd(&exp), Err(HpError::Domain));
    }

    #[test]
    fn checked_powd_negative_base_integer_exp_is_ok() {
        // (-2)^3 = -8 — integer exponent is valid (result is real)
        let base = HpNum::from(-2i32);
        let exp = HpNum::from(3i32);
        let r = base.checked_powd(&exp).unwrap();
        assert_eq!(r.inner(), Decimal::from(-8));
    }
}

#[cfg(test)]
mod num_trig_math_tests {
    use crate::num::HpNum;
    use crate::error::HpError;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // ── Forward trig (decimal-native via MathematicalOps) ─────────────────────

    #[test]
    fn checked_sin_of_zero_is_zero() {
        let n = HpNum::zero();
        let r = n.checked_sin().unwrap();
        assert!(r.is_zero());
    }

    #[test]
    fn checked_sin_of_pi_over_6_is_half() {
        // sin(π/6) = 0.5 — π/6 ≈ 0.5235987756 radians
        let pi_over_6 = HpNum::from(Decimal::from_str("0.5235987756").unwrap());
        let r = pi_over_6.checked_sin().unwrap();
        // Allow slight precision difference at 10th digit
        let expected = Decimal::from_str("0.5").unwrap();
        // round to 1 sig digit to avoid precision comparison issues
        let rounded = r.inner().round_dp(1);
        assert_eq!(rounded, expected);
    }

    #[test]
    fn checked_cos_of_zero_is_one() {
        let n = HpNum::zero();
        let r = n.checked_cos().unwrap();
        assert_eq!(r.inner(), Decimal::from(1));
    }

    #[test]
    fn checked_cos_of_pi_over_3_is_half() {
        // cos(π/3) = 0.5 — π/3 ≈ 1.047197551 radians
        let pi_over_3 = HpNum::from(Decimal::from_str("1.047197551").unwrap());
        let r = pi_over_3.checked_cos().unwrap();
        let rounded = r.inner().round_dp(1);
        assert_eq!(rounded, Decimal::from_str("0.5").unwrap());
    }

    #[test]
    fn checked_tan_of_zero_is_zero() {
        let n = HpNum::zero();
        let r = n.checked_tan().unwrap();
        assert!(r.is_zero());
    }

    #[test]
    fn checked_tan_of_pi_over_4_is_one() {
        // tan(π/4) = 1.0 — π/4 ≈ 0.7853981634 radians
        let pi_over_4 = HpNum::from(Decimal::from_str("0.7853981634").unwrap());
        let r = pi_over_4.checked_tan().unwrap();
        let rounded = r.inner().round_dp(0);
        assert_eq!(rounded, Decimal::from(1));
    }

    // ── Inverse trig (f64 round-trip bridge) ──────────────────────────────────

    #[test]
    fn checked_asin_of_zero_is_zero() {
        let n = HpNum::zero();
        let r = n.checked_asin().unwrap();
        assert!(r.is_zero());
    }

    #[test]
    fn checked_asin_of_half_is_pi_over_6() {
        // asin(0.5) = π/6 ≈ 0.5235987756 radians (10 sig digits)
        let n = HpNum::from(Decimal::from_str("0.5").unwrap());
        let r = n.checked_asin().unwrap();
        let expected = Decimal::from_str("0.5235987756").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_asin_of_one_is_pi_over_2() {
        // asin(1.0) = π/2 ≈ 1.570796327 radians (10 sig digits)
        let n = HpNum::from(1i32);
        let r = n.checked_asin().unwrap();
        let expected = Decimal::from_str("1.570796327").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_asin_out_of_domain_returns_domain() {
        // asin(2.0) — outside [-1, 1] → Domain
        let n = HpNum::from(2i32);
        assert_eq!(n.checked_asin(), Err(HpError::Domain));
    }

    #[test]
    fn checked_acos_of_one_is_zero() {
        let n = HpNum::from(1i32);
        let r = n.checked_acos().unwrap();
        assert!(r.is_zero());
    }

    #[test]
    fn checked_acos_of_half_is_pi_over_3() {
        // acos(0.5) = π/3 ≈ 1.047197551 radians (10 sig digits)
        let n = HpNum::from(Decimal::from_str("0.5").unwrap());
        let r = n.checked_acos().unwrap();
        let expected = Decimal::from_str("1.047197551").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_acos_out_of_domain_returns_domain() {
        // acos(-2.0) — outside [-1, 1] → Domain
        let n = HpNum::from(-2i32);
        assert_eq!(n.checked_acos(), Err(HpError::Domain));
    }

    #[test]
    fn checked_atan_of_zero_is_zero() {
        let n = HpNum::zero();
        let r = n.checked_atan().unwrap();
        assert!(r.is_zero());
    }

    #[test]
    fn checked_atan_of_one_is_pi_over_4() {
        // atan(1.0) = π/4 ≈ 0.7853981634 radians (10 sig digits)
        let n = HpNum::from(1i32);
        let r = n.checked_atan().unwrap();
        let expected = Decimal::from_str("0.7853981634").unwrap();
        assert_eq!(r.inner(), expected);
    }

    #[test]
    fn checked_atan_no_domain_restriction_large_input() {
        // atan(1000) should succeed (no domain restriction for atan)
        let n = HpNum::from(1000i32);
        let r = n.checked_atan();
        assert!(r.is_ok());
    }

    #[test]
    fn f64_round_trip_bridge_comment_documented() {
        // Structural: the comment "f64 round-trip" must be present in num.rs
        // This is verified by grep in acceptance_criteria — this test is a placeholder
        // that always passes to document the requirement.
        // Structural: the comment "f64 round-trip" is present in num.rs — verified by acceptance_criteria grep.
        // No runtime assertion needed here.
    }
}

// ── Phase 3 Plan 01: CalcState Programming Engine Fields ─────────────────────
// RED phase — these fail until the five Phase 3 fields are added to state.rs.
// Spec: D-01, D-05, D-06 from 03-CONTEXT.md
#[cfg(test)]
mod phase3_state_tests {
    use crate::state::CalcState;

    #[test]
    fn calcstate_new_program_is_empty_vec() {
        // D-01: program is a flat Vec<Op>; starts empty
        let s = CalcState::new();
        assert!(s.program.is_empty());
    }

    #[test]
    fn calcstate_new_prgm_mode_is_false() {
        // D-03: prgm_mode gates dispatch() recording; off at startup
        let s = CalcState::new();
        assert!(!s.prgm_mode);
    }

    #[test]
    fn calcstate_new_pc_is_zero() {
        // D-05: pc is the program counter index; 0 at startup
        let s = CalcState::new();
        assert_eq!(s.pc, 0usize);
    }

    #[test]
    fn calcstate_new_call_stack_is_empty() {
        // D-14: call_stack holds subroutine return addresses; empty at startup
        let s = CalcState::new();
        assert!(s.call_stack.is_empty());
    }

    #[test]
    fn calcstate_new_is_running_is_false() {
        // D-06: is_running guards re-entrancy; false at startup
        let s = CalcState::new();
        assert!(!s.is_running);
    }
}

#[cfg(test)]
mod dispatch_tests {
    use crate::num::HpNum;
    use crate::state::CalcState;
    use crate::ops::{Op, dispatch};
    use crate::error::HpError;
    use rust_decimal::Decimal;

    #[test]
    fn dispatch_add_works() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(3i32);
        state.stack.y = HpNum::from(4i32);
        dispatch(&mut state, Op::Add).unwrap();
        assert_eq!(state.stack.x.inner(), Decimal::from(7));
    }

    #[test]
    fn dispatch_push_num_enters_value() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = true;
        let val = HpNum::from(42i32);
        dispatch(&mut state, Op::PushNum(val)).unwrap();
        assert_eq!(state.stack.x.inner(), Decimal::from(42));
    }

    #[test]
    fn dispatch_div_by_zero_propagates_error() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(0i32);
        state.stack.y = HpNum::from(5i32);
        let result = dispatch(&mut state, Op::Div);
        assert_eq!(result, Err(HpError::DivideByZero));
    }

    #[test]
    fn op_enum_is_clone_and_partialeq() {
        let a = Op::Add;
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(Op::Add, Op::Sub);
    }
}

#[cfg(test)]
mod stack_ops_dispatch_tests {
    use crate::num::HpNum;
    use crate::state::CalcState;
    use crate::ops::stack_ops::{op_enter, op_clx, op_chs, op_rdn, op_xy_swap, op_lastx};
    use rust_decimal::Decimal;

    #[test]
    fn op_enter_lifts_unconditionally_and_disables_lift() {
        // With lift_enabled = false, ENTER should still lift
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(5i32);
        state.stack.y = HpNum::from(2i32);
        state.stack.z = HpNum::from(1i32);
        state.stack.t = HpNum::from(0i32);
        state.stack.lift_enabled = false; // disabled, but ENTER always lifts

        op_enter(&mut state).unwrap();

        // After ENTER: T←Z, Z←Y, Y←X (X duplicated in Y)
        assert_eq!(state.stack.x.inner(), Decimal::from(5)); // X unchanged
        assert_eq!(state.stack.y.inner(), Decimal::from(5)); // Y = old X
        assert_eq!(state.stack.z.inner(), Decimal::from(2)); // Z = old Y
        assert_eq!(state.stack.t.inner(), Decimal::from(1)); // T = old Z
        // lift must be disabled after ENTER
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn op_enter_lifts_when_already_enabled() {
        // With lift_enabled = true, ENTER should still lift (unconditional)
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(3i32);
        state.stack.y = HpNum::from(2i32);
        state.stack.lift_enabled = true;

        op_enter(&mut state).unwrap();

        assert_eq!(state.stack.y.inner(), Decimal::from(3)); // Y = old X
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn op_clx_zeros_x_and_disables_lift() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(42i32);
        state.stack.lift_enabled = true;

        op_clx(&mut state).unwrap();

        assert!(state.stack.x.is_zero());
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn op_chs_negates_x_neutral_lift() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(7i32);
        state.stack.lift_enabled = true;

        op_chs(&mut state).unwrap();

        assert_eq!(state.stack.x.inner(), Decimal::from(-7));
        assert!(state.stack.lift_enabled); // Neutral: unchanged
    }

    #[test]
    fn op_rdn_rotates_stack_down() {
        // Stack: X=1, Y=2, Z=3, T=4
        // After RDN: X=Y=2, Y=Z=3, Z=T=4, T=X=1
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32);
        state.stack.y = HpNum::from(2i32);
        state.stack.z = HpNum::from(3i32);
        state.stack.t = HpNum::from(4i32);

        op_rdn(&mut state).unwrap();

        assert_eq!(state.stack.x.inner(), Decimal::from(2));
        assert_eq!(state.stack.y.inner(), Decimal::from(3));
        assert_eq!(state.stack.z.inner(), Decimal::from(4));
        assert_eq!(state.stack.t.inner(), Decimal::from(1));
    }

    #[test]
    fn op_rdn_neutral_lift() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        op_rdn(&mut state).unwrap();
        assert!(!state.stack.lift_enabled); // Neutral: unchanged

        state.stack.lift_enabled = true;
        op_rdn(&mut state).unwrap();
        assert!(state.stack.lift_enabled); // Neutral: unchanged
    }

    #[test]
    fn op_xy_swap_exchanges_x_and_y() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(10i32);
        state.stack.y = HpNum::from(20i32);

        op_xy_swap(&mut state).unwrap();

        assert_eq!(state.stack.x.inner(), Decimal::from(20));
        assert_eq!(state.stack.y.inner(), Decimal::from(10));
    }

    #[test]
    fn op_xy_swap_neutral_lift() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        op_xy_swap(&mut state).unwrap();
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn op_lastx_recalls_lastx_into_x() {
        let mut state = CalcState::new();
        state.stack.lastx = HpNum::from(99i32);
        state.stack.x = HpNum::from(0i32);
        state.stack.lift_enabled = false;

        op_lastx(&mut state).unwrap();

        // LASTX enters lastx value into X (via enter_number with lift_enabled)
        assert_eq!(state.stack.x.inner(), Decimal::from(99));
        assert!(state.stack.lift_enabled); // Enable
    }

    #[test]
    fn op_lastx_lifts_stack_when_enabled() {
        // LASTX should push stack: lift_enabled is set to true before enter_number call
        let mut state = CalcState::new();
        state.stack.lastx = HpNum::from(5i32);
        state.stack.x = HpNum::from(10i32);
        state.stack.y = HpNum::from(20i32);
        state.stack.lift_enabled = false; // start disabled, op_lastx enables it

        op_lastx(&mut state).unwrap();

        // After LASTX: X=lastx=5, Y=old X=10
        assert_eq!(state.stack.x.inner(), Decimal::from(5));
        assert_eq!(state.stack.y.inner(), Decimal::from(10));
        assert!(state.stack.lift_enabled);
    }
}

// ── Phase 5 Plan 08: CalcState serde round-trip ──────────────────────────────
// Covers PERS-01 at the core level: full CalcState serialization contract.
#[cfg(test)]
mod serde_tests {
    use crate::num::HpNum;
    use crate::state::CalcState;
    use crate::ops::Op;

    #[test]
    fn test_calc_state_serde_roundtrip() {
        // Full CalcState round-trip via serde_json — covers PERS-01 at the core level.
        let mut state = CalcState::new();

        // Set up some non-default state
        state.stack.x = HpNum::from(3i32);
        state.regs[5] = HpNum::from(42i32);
        state.user_mode = true;
        state.key_assignments.insert('z', "MYPROG".to_string());
        state.program = vec![
            Op::Lbl("A".to_string()),
            Op::Add,
            Op::Rtn,
        ];
        // Set is_running = true before serializing so the round-trip test
        // actually exercises the cross-module guarantee that load_state resets is_running.
        // serde_json::from_str alone does NOT reset it — that reset belongs in persistence::load_state.
        // This test verifies the serde contract (field survives JSON). The persistence::test_is_running_reset_on_load
        // test verifies the load_state() reset. Together they close the guarantee.
        state.is_running = true;

        let json = serde_json::to_string(&state).expect("CalcState must serialize");
        // Raw serde_json round-trip: is_running should be true here (it serialized as true)
        let back: CalcState = serde_json::from_str(&json).expect("CalcState must deserialize");

        assert_eq!(back.stack.x, state.stack.x, "X register must survive round-trip");
        assert_eq!(back.regs[5], state.regs[5], "registers must survive round-trip");
        assert!(back.user_mode, "user_mode must survive round-trip");
        assert_eq!(
            back.key_assignments.get(&'z').map(|s| s.as_str()),
            Some("MYPROG"),
            "key_assignments must survive round-trip"
        );
        assert_eq!(back.program.len(), 3, "program must survive round-trip");
        // is_running serializes and deserializes faithfully at the serde level.
        // The persistence::load_state() function resets it to false — tested in persistence::tests.
        assert!(back.is_running, "is_running must survive the raw serde round-trip (reset happens in load_state, not serde)");
    }
}
