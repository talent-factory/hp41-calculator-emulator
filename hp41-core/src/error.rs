use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum HpError {
    #[error("overflow")]
    Overflow,
    #[error("divide by zero")]
    DivideByZero,
    #[error("invalid operation")]
    InvalidOp,
    #[error("domain error")]
    Domain,
    /// Hardware-spec out-of-range (Phase 20 / D-06): FACT with X > 69 returns this
    /// before the magnitude `Overflow` path is reached. Preserves the HP-41
    /// "X > 69 → OUT OF RANGE" wording from SC-3 / FN-MATH-08.
    #[error("out of range")]
    OutOfRange,
    /// HP-41 subroutine call-depth exceeded (5th nested XEQ).
    #[error("try again")]
    CallDepth,
    /// HMS field-range validation: minutes >= 60 or seconds >= 60.
    #[error("invalid input")]
    InvalidInput,
    /// Card Reader: WDTA/RDTA/WPRGM/RDPRGM with an empty ALPHA register.
    /// Matches the hardware-faithful "ALPHA DATA" message on real HP-41 card readers.
    #[error("alpha data")]
    AlphaData,
    /// Card Reader: card payload could not be encoded/decoded. Carries a short
    /// diagnostic (serde line/col, "unsupported op", "truncated", etc.) so the
    /// frontend can surface something more useful than a generic "CARD DATA".
    #[error("card data: {0}")]
    CardData(String),
}
