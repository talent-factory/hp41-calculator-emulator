use crate::error::HpError;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use rust_decimal::RoundingStrategy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HpNum(#[serde(with = "rust_decimal::serde::str")] pub(crate) Decimal);

impl HpNum {
    /// Enforce HP-41 10-significant-digit precision with round-half-away-from-zero.
    /// This matches HP-41 hardware display rounding (NOT Bankers/MidpointNearestEven).
    pub fn rounded(d: Decimal) -> Self {
        HpNum(
            d.round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
                .expect("round_sf_with_strategy(10) must succeed for valid finite Decimal"),
        )
    }

    pub fn zero() -> Self {
        HpNum(Decimal::ZERO)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn checked_add(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.0
            .checked_add(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn checked_sub(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.0
            .checked_sub(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn checked_mul(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.0
            .checked_mul(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn checked_div(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        if rhs.0.is_zero() {
            return Err(HpError::DivideByZero);
        }
        self.0
            .checked_div(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    // ── Scalar math methods ───────────────────────────────────────────────────

    /// 1/x — reciprocal of self.
    /// Returns DivideByZero if self is zero.
    /// LiftEffect declared by caller (Enable for op_recip).
    pub fn checked_recip(&self) -> Result<HpNum, HpError> {
        HpNum::from(1).checked_div(self) // reuses existing DivideByZero guard in checked_div
    }

    /// √x — square root of self.
    /// Returns Domain if self < 0.
    pub fn checked_sqrt(&self) -> Result<HpNum, HpError> {
        if self.0 < Decimal::ZERO {
            return Err(HpError::Domain);
        }
        // rust_decimal MathematicalOps provides sqrt() returning Option<Decimal>
        self.0.sqrt().map(HpNum::rounded).ok_or(HpError::Overflow)
    }

    /// x² — self multiplied by self.
    /// Uses checked_mul — no maths feature dependency.
    pub fn checked_sq(&self) -> Result<HpNum, HpError> {
        self.checked_mul(self)
    }

    /// LN — natural logarithm of self.
    /// Returns Domain if self ≤ 0.
    pub fn checked_ln(&self) -> Result<HpNum, HpError> {
        if self.0 <= Decimal::ZERO {
            return Err(HpError::Domain);
        }
        self.0
            .checked_ln()
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// LOG — log base 10 of self.
    /// Returns Domain if self ≤ 0.
    pub fn checked_log10(&self) -> Result<HpNum, HpError> {
        if self.0 <= Decimal::ZERO {
            return Err(HpError::Domain);
        }
        self.0
            .checked_log10()
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// e^x — natural exponential of self.
    pub fn checked_exp(&self) -> Result<HpNum, HpError> {
        self.0
            .checked_exp()
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// 10^x — base-10 exponential of self.
    /// Computed as 10^self via MathematicalOps::checked_powd.
    pub fn checked_exp10(&self) -> Result<HpNum, HpError> {
        Decimal::from(10)
            .checked_powd(self.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// Y^X — self raised to the power of exp.
    /// Returns Domain if self < 0 and exp has a non-zero fractional part
    /// (complex result — HP-41 returns INVALID DATA in this case).
    pub fn checked_powd(&self, exp: &HpNum) -> Result<HpNum, HpError> {
        if self.0.is_sign_negative() && !exp.0.fract().is_zero() {
            return Err(HpError::Domain);
        }
        self.0
            .checked_powd(exp.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    /// %CH — percent change from self (base, Y) to new_val (the new value, X).
    /// Computes `((new_val − self) / self) × 100`.
    /// Returns `DivideByZero` if self is zero; `Overflow` on intermediate or final overflow.
    /// Sign emerges naturally from the arithmetic — negative bases are not special-cased.
    pub fn checked_pct_change(&self, new_val: &HpNum) -> Result<HpNum, HpError> {
        let delta = new_val.checked_sub(self)?;
        let ratio = delta.checked_div(self)?; // DivideByZero if self == 0
        ratio.checked_mul(&HpNum::from(100i32))
    }

    // ── Trigonometric methods (angle in RADIANS) ──────────────────────────────
    // All trig methods expect/return values in radians.
    // Angle mode conversion (DEG/GRAD ↔ RAD) is the caller's responsibility.

    /// sin(x) — x must be in radians. Uses rust_decimal MathematicalOps (Maclaurin series).
    pub fn checked_sin(&self) -> Result<HpNum, HpError> {
        self.0
            .checked_sin()
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    /// cos(x) — x must be in radians. Uses rust_decimal MathematicalOps.
    pub fn checked_cos(&self) -> Result<HpNum, HpError> {
        self.0
            .checked_cos()
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    /// tan(x) — x must be in radians. Returns Domain at tan(π/2) etc.
    pub fn checked_tan(&self) -> Result<HpNum, HpError> {
        self.0
            .checked_tan()
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    // ── Inverse trig — f64 round-trip bridge ─────────────────────────────────
    // rust_decimal MathematicalOps does not provide asin/acos/atan.
    // f64 has ~15.9 decimal digits of precision; rounding to 10 via HpNum::rounded()
    // is sufficient to meet QUAL-06 (≥98% accuracy at 10 sig digits).

    /// asin(x) — returns result in radians. Domain error if |x| > 1.
    pub fn checked_asin(&self) -> Result<HpNum, HpError> {
        let v = self.0.to_f64().ok_or(HpError::Overflow)?;
        if !(-1.0..=1.0).contains(&v) {
            return Err(HpError::Domain);
        }
        Decimal::from_f64(v.asin())
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// acos(x) — returns result in radians. Domain error if |x| > 1.
    pub fn checked_acos(&self) -> Result<HpNum, HpError> {
        let v = self.0.to_f64().ok_or(HpError::Overflow)?;
        if !(-1.0..=1.0).contains(&v) {
            return Err(HpError::Domain);
        }
        Decimal::from_f64(v.acos())
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// atan(x) — returns result in radians. No domain restriction.
    pub fn checked_atan(&self) -> Result<HpNum, HpError> {
        let v = self.0.to_f64().ok_or(HpError::Overflow)?;
        Decimal::from_f64(v.atan())
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn negate(&self) -> Self {
        HpNum(-self.0)
    }

    pub fn inner(&self) -> Decimal {
        self.0
    }

    /// INT — truncate toward zero (integer part, HP-41 INT function).
    /// Equivalent to Decimal::trunc() which truncates toward zero.
    /// No domain restriction; always succeeds.
    pub fn trunc_int(&self) -> HpNum {
        HpNum(self.0.trunc())
    }
}

impl From<i32> for HpNum {
    fn from(n: i32) -> Self {
        HpNum(Decimal::from(n))
    }
}

impl From<Decimal> for HpNum {
    fn from(d: Decimal) -> Self {
        HpNum::rounded(d)
    }
}

impl std::fmt::Display for HpNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for HpNum {
    fn default() -> Self {
        HpNum::zero()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_hpnum_serde_is_string() {
        let n = HpNum::from(3i32);
        let json = serde_json::to_string(&n).unwrap();
        // Must be a JSON string "3", not a float 3
        assert!(
            json.starts_with('"'),
            "HpNum must serialize as JSON string, got: {json}"
        );
        let back: HpNum = serde_json::from_str(&json).unwrap();
        assert_eq!(back, n, "round-trip must be lossless");
    }

    #[test]
    fn test_hpnum_serde_decimal_precision() {
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let d = Decimal::from_str("3.1415926536").unwrap();
        let n = HpNum(d);
        let json = serde_json::to_string(&n).unwrap();
        let back: HpNum = serde_json::from_str(&json).unwrap();
        assert_eq!(n, back, "10-digit decimal must round-trip exactly");
    }
}
