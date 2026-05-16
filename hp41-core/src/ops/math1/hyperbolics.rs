// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Hyperbolic function operations for Math Pac I (Plan 28-02).
//!
//! All six ops are angle-mode-INDEPENDENT (no `to_radians_f64` / `from_radians_f64` calls).
//! Each op consumes X via the f64 bridge, applies the f64 hyperbolic, then writes
//! the rounded result back through `unary_result` (LiftEffect: Enable — identical to
//! `op_sin` / `op_cos` in v2.2; `unary_result` always sets `lift_enabled = true`).
//!
//! Domain guards:
//! - `op_acosh`: X < 1.0 → `HpError::Domain`
//! - `op_atanh`: |X| >= 1.0 → `HpError::Domain`
//! - All others: no domain restriction; very large X produces `HpError::Overflow` when
//!   `Decimal::from_f64(inf)` returns `None`.

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::unary_result;
use crate::state::CalcState;

/// SINH — hyperbolic sine. Angle-mode-independent.
/// X = sinh(X). LiftEffect: Enable (via unary_result).
/// Domain: all reals (overflow for extreme magnitudes).
pub fn op_sinh(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let result = Decimal::from_f64(v.sinh())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

/// COSH — hyperbolic cosine. Angle-mode-independent.
/// X = cosh(X). LiftEffect: Enable (via unary_result).
/// Domain: all reals (overflow for extreme magnitudes).
pub fn op_cosh(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let result = Decimal::from_f64(v.cosh())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

/// TANH — hyperbolic tangent. Angle-mode-independent.
/// X = tanh(X). LiftEffect: Enable (via unary_result).
/// Domain: all reals. tanh saturates to ±1.0 for large |X| — that is correct, not an error.
pub fn op_tanh(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let result = Decimal::from_f64(v.tanh())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

/// ASINH — inverse hyperbolic sine. Angle-mode-independent.
/// X = arcsinh(X). LiftEffect: Enable (via unary_result).
/// Domain: all reals (no domain restriction).
pub fn op_asinh(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let result = Decimal::from_f64(v.asinh())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

/// ACOSH — inverse hyperbolic cosine. Angle-mode-independent.
/// X = arccosh(X). LiftEffect: Enable (via unary_result).
/// Domain: X >= 1.0; X < 1.0 → HpError::Domain.
pub fn op_acosh(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    if v < 1.0 {
        return Err(HpError::Domain);
    }
    let result = Decimal::from_f64(v.acosh())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

/// ATANH — inverse hyperbolic tangent. Angle-mode-independent.
/// X = arctanh(X). LiftEffect: Enable (via unary_result).
/// Domain: |X| < 1.0; |X| >= 1.0 → HpError::Domain.
pub fn op_atanh(state: &mut CalcState) -> Result<(), HpError> {
    let v = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    if v.abs() >= 1.0 {
        return Err(HpError::Domain);
    }
    let result = Decimal::from_f64(v.atanh())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    unary_result(state, result);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ops::dispatch;
    use crate::ops::Op;
    use crate::HpNum;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    fn make_state_with_x(x: &str) -> CalcState {
        let mut state = CalcState::new();
        let d = rust_decimal::Decimal::from_str_exact(x)
            .or_else(|_| rust_decimal::Decimal::from_scientific(x))
            .unwrap();
        state.stack.x = HpNum::rounded(d);
        state
    }

    fn get_x(state: &CalcState) -> f64 {
        state.stack.x.inner().to_f64().unwrap()
    }

    // ── SINH tests (5 minimum) ──────────────────────────────────────────────

    /// Catches: wrong identity value for sinh(0).
    #[test]
    fn sinh_zero_is_zero() {
        let mut s = make_state_with_x("0");
        op_sinh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: wrong sign for negative argument.
    /// Source: HP 00041-90034 p.44, ex.1 — sinh(-1) = -1.175201194
    /// Free42 v3.0.5: -1.1752011936 — agrees with OM
    #[test]
    fn sinh_positive_reference() {
        let mut s = make_state_with_x("1");
        op_sinh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 1.175_201_193_6, max_relative = 1e-7);
    }

    /// Catches: sign error or magnitude error for negative argument.
    /// Source: HP 00041-90034 p.44, ex.2 — sinh(-1) = -1.175201194
    /// Free42 v3.0.5: -1.1752011936 — agrees with OM
    #[test]
    fn sinh_negative_argument() {
        let mut s = make_state_with_x("-1");
        op_sinh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), -1.175_201_193_6, max_relative = 1e-7);
    }

    /// Catches: large magnitude not producing Overflow (sinh saturates in f64 to inf at ~710).
    /// At x=710 f64 sinh overflows to inf; Decimal::from_f64(inf) returns None → Overflow.
    #[test]
    fn sinh_extreme_magnitude_overflows() {
        let mut s = make_state_with_x("1000");
        let result = op_sinh(&mut s);
        assert_eq!(result, Err(HpError::Overflow), "sinh(1000) must overflow (inf in f64)");
    }

    /// Catches: LiftEffect mismatch — unary_result sets lift_enabled=true (Enable).
    #[test]
    fn sinh_sets_lift_enabled() {
        let mut s = make_state_with_x("1");
        s.stack.lift_enabled = false;
        op_sinh(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "op_sinh must enable stack lift via unary_result");
    }

    // ── COSH tests (5 minimum) ──────────────────────────────────────────────

    /// Catches: wrong identity value for cosh(0) = 1.
    #[test]
    fn cosh_zero_is_one() {
        let mut s = make_state_with_x("0");
        op_cosh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    }

    /// Catches: wrong magnitude.
    /// Source: HP 00041-90034 p.44, ex.3 — cosh(1) = 1.543080635
    /// Free42 v3.0.5: 1.5430806348 — agrees with OM
    #[test]
    fn cosh_positive_reference() {
        let mut s = make_state_with_x("1");
        op_cosh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 1.543_080_634_8, max_relative = 1e-7);
    }

    /// Catches: cosh not symmetric (should be same for +x and -x).
    /// Source: HP 00041-90034 p.44 — cosh(-1) = cosh(1) = 1.543080635
    /// Free42 v3.0.5: 1.5430806348 — agrees with OM
    #[test]
    fn cosh_negative_same_as_positive() {
        let mut s = make_state_with_x("-1");
        op_cosh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 1.543_080_634_8, max_relative = 1e-7);
    }

    /// Catches: large magnitude not producing Overflow.
    #[test]
    fn cosh_extreme_magnitude_overflows() {
        let mut s = make_state_with_x("1000");
        let result = op_cosh(&mut s);
        assert_eq!(result, Err(HpError::Overflow), "cosh(1000) must overflow (inf in f64)");
    }

    /// Catches: LiftEffect mismatch.
    #[test]
    fn cosh_sets_lift_enabled() {
        let mut s = make_state_with_x("0");
        s.stack.lift_enabled = false;
        op_cosh(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "op_cosh must enable stack lift via unary_result");
    }

    // ── TANH tests (5 minimum) ──────────────────────────────────────────────

    /// Catches: wrong identity value for tanh(0) = 0.
    #[test]
    fn tanh_zero_is_zero() {
        let mut s = make_state_with_x("0");
        op_tanh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: wrong magnitude.
    /// Source: HP 00041-90034 p.44, ex.5 — tanh(1) = 0.761594156
    /// Free42 v3.0.5: 0.7615941560 — agrees with OM
    #[test]
    fn tanh_positive_reference() {
        let mut s = make_state_with_x("1");
        op_tanh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.761_594_156_0, max_relative = 1e-7);
    }

    /// Catches: sign error for negative argument.
    /// Source: HP 00041-90034 p.44 — tanh(-1) = -0.761594156
    /// Free42 v3.0.5: -0.7615941560 — agrees with OM
    #[test]
    fn tanh_negative_argument() {
        let mut s = make_state_with_x("-1");
        op_tanh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), -0.761_594_156_0, max_relative = 1e-7);
    }

    /// Catches: tanh failing to saturate at large arguments (should not overflow; saturates to ±1).
    #[test]
    fn tanh_saturates_at_large_magnitude() {
        let mut s = make_state_with_x("100");
        op_tanh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 1.0, max_relative = 1e-7);
    }

    /// Catches: LiftEffect mismatch.
    #[test]
    fn tanh_sets_lift_enabled() {
        let mut s = make_state_with_x("0");
        s.stack.lift_enabled = false;
        op_tanh(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "op_tanh must enable stack lift via unary_result");
    }

    // ── ASINH tests (5 minimum) ─────────────────────────────────────────────

    /// Catches: wrong identity value for asinh(0) = 0.
    #[test]
    fn asinh_zero_is_zero() {
        let mut s = make_state_with_x("0");
        op_asinh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: wrong magnitude.
    /// Source: HP 00041-90034 p.45, ex.7 — asinh(1) = 0.881373587
    /// Free42 v3.0.5: 0.8813735870 — agrees with OM
    #[test]
    fn asinh_positive_reference() {
        let mut s = make_state_with_x("1");
        op_asinh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.881_373_587_0, max_relative = 1e-7);
    }

    /// Catches: sign error for negative argument.
    /// Source: HP 00041-90034 p.45 — asinh(-1) = -0.881373587
    /// Free42 v3.0.5: -0.8813735870 — agrees with OM
    #[test]
    fn asinh_negative_argument() {
        let mut s = make_state_with_x("-1");
        op_asinh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), -0.881_373_587_0, max_relative = 1e-7);
    }

    /// Catches: asinh raising Domain error incorrectly (no domain restriction).
    #[test]
    fn asinh_very_large_argument_succeeds() {
        // asinh(500) ≈ 6.9078... — should not overflow
        let mut s = make_state_with_x("500");
        let result = op_asinh(&mut s);
        assert!(result.is_ok(), "op_asinh should accept very large argument (no domain restriction)");
    }

    /// Catches: LiftEffect mismatch.
    #[test]
    fn asinh_sets_lift_enabled() {
        let mut s = make_state_with_x("0");
        s.stack.lift_enabled = false;
        op_asinh(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "op_asinh must enable stack lift via unary_result");
    }

    // ── ACOSH tests (5 minimum) ─────────────────────────────────────────────

    /// Catches: wrong identity value for acosh(1) = 0.
    #[test]
    fn acosh_one_is_zero() {
        let mut s = make_state_with_x("1");
        op_acosh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: wrong magnitude.
    /// Source: HP 00041-90034 p.45, ex.9 — acosh(2) = 1.316957897
    /// Free42 v3.0.5: 1.3169578970 — agrees with OM
    #[test]
    fn acosh_positive_reference() {
        let mut s = make_state_with_x("2");
        op_acosh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 1.316_957_897_0, max_relative = 1e-7);
    }

    /// Catches: domain guard missing for X < 1 (should return Domain).
    /// Source: HP 00041-90034 p.45 — acosh(0.5) = Domain (X < 1 forbidden)
    /// Free42 v3.0.5: returns error — agrees with OM domain restriction
    #[test]
    fn acosh_below_one_returns_domain() {
        let mut s = make_state_with_x("0.5");
        let result = op_acosh(&mut s);
        assert_eq!(result, Err(HpError::Domain), "acosh(0.5) must return Domain (X < 1 forbidden)");
    }

    /// Catches: domain guard missing for X = 0 (should return Domain).
    #[test]
    fn acosh_zero_returns_domain() {
        let mut s = make_state_with_x("0");
        let result = op_acosh(&mut s);
        assert_eq!(result, Err(HpError::Domain), "acosh(0) must return Domain (X < 1 forbidden)");
    }

    /// Catches: LiftEffect mismatch.
    #[test]
    fn acosh_sets_lift_enabled() {
        let mut s = make_state_with_x("2");
        s.stack.lift_enabled = false;
        op_acosh(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "op_acosh must enable stack lift via unary_result");
    }

    // ── ATANH tests (5 minimum) ─────────────────────────────────────────────

    /// Catches: wrong identity value for atanh(0) = 0.
    #[test]
    fn atanh_zero_is_zero() {
        let mut s = make_state_with_x("0");
        op_atanh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.0, max_relative = 1e-7);
    }

    /// Catches: wrong magnitude.
    /// Source: HP 00041-90034 p.45, ex.11 — atanh(0.5) = 0.549306144
    /// Free42 v3.0.5: 0.5493061443 — agrees with OM
    #[test]
    fn atanh_positive_reference() {
        let mut s = make_state_with_x("0.5");
        op_atanh(&mut s).unwrap();
        assert_relative_eq!(get_x(&s), 0.549_306_144_3, max_relative = 1e-7);
    }

    /// Catches: domain guard missing for |X| >= 1 (X = 1, should return Domain).
    /// Source: HP 00041-90034 p.45 — atanh(1) = Domain (|X| >= 1 forbidden)
    /// Free42 v3.0.5: returns error — agrees with OM domain restriction
    #[test]
    fn atanh_one_returns_domain() {
        let mut s = make_state_with_x("1");
        let result = op_atanh(&mut s);
        assert_eq!(result, Err(HpError::Domain), "atanh(1) must return Domain (|X| >= 1 forbidden)");
    }

    /// Catches: domain guard missing for X = -1.
    #[test]
    fn atanh_neg_one_returns_domain() {
        let mut s = make_state_with_x("-1");
        let result = op_atanh(&mut s);
        assert_eq!(result, Err(HpError::Domain), "atanh(-1) must return Domain (|X| >= 1 forbidden)");
    }

    /// Catches: LiftEffect mismatch.
    #[test]
    fn atanh_sets_lift_enabled() {
        let mut s = make_state_with_x("0.5");
        s.stack.lift_enabled = false;
        op_atanh(&mut s).unwrap();
        assert!(s.stack.lift_enabled, "op_atanh must enable stack lift via unary_result");
    }

    // ── Cross-cutting: dispatch path test ───────────────────────────────────

    /// Catches: LASTX not updated by hyperbolic ops (unary_result must save X to LASTX).
    #[test]
    fn sinh_updates_lastx() {
        let mut s = make_state_with_x("1");
        let orig_x = s.stack.x.clone();
        op_sinh(&mut s).unwrap();
        assert_eq!(s.stack.lastx, orig_x, "op_sinh must save X to LASTX via unary_result");
    }

    /// Catches: angle-mode dependence (hyperbolics must not be affected by angle mode).
    #[test]
    fn sinh_is_angle_mode_independent() {
        let mut s_deg = CalcState::new();
        dispatch(&mut s_deg, Op::SetDeg).unwrap();
        s_deg.stack.x = HpNum::rounded(rust_decimal::Decimal::from(1u32));
        op_sinh(&mut s_deg).unwrap();
        let result_deg = get_x(&s_deg);

        let mut s_rad = CalcState::new();
        dispatch(&mut s_rad, Op::SetRad).unwrap();
        s_rad.stack.x = HpNum::rounded(rust_decimal::Decimal::from(1u32));
        op_sinh(&mut s_rad).unwrap();
        let result_rad = get_x(&s_rad);

        assert_relative_eq!(result_deg, result_rad, max_relative = 1e-10);
        // Catches: angle-mode dependence — if they differed, sinh would be using to_radians_f64
    }
}
