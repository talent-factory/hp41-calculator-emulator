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
