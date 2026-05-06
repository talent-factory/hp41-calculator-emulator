use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use crate::error::HpError;

#[derive(Clone, Debug, PartialEq)]
pub struct HpNum(pub(crate) Decimal);

impl HpNum {
    /// Enforce HP-41 10-significant-digit precision with round-half-away-from-zero.
    /// This matches HP-41 hardware display rounding (NOT Bankers/MidpointNearestEven).
    pub fn rounded(d: Decimal) -> Self {
        HpNum(
            d.round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
                .unwrap_or(d)
        )
    }

    pub fn zero() -> Self {
        HpNum(Decimal::ZERO)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn checked_add(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.0.checked_add(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn checked_sub(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.0.checked_sub(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn checked_mul(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.0.checked_mul(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn checked_div(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        if rhs.0.is_zero() {
            return Err(HpError::DivideByZero);
        }
        self.0.checked_div(rhs.0)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn negate(&self) -> Self {
        HpNum(-self.0)
    }

    pub fn inner(&self) -> Decimal {
        self.0
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
