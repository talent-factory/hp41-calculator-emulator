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
    // Phase 3 addition — HP-41 call-depth exceeded (5th subroutine level, D-13/D-14)
    #[error("try again")]
    CallDepth,
    // Phase 6 addition — HMS field-range validation: minutes >= 60 or seconds >= 60 (D-06)
    #[error("invalid input")]
    InvalidInput,
    // Phase 19 addition — Card Reader: WDTA/WPRGM with empty ALPHA register, or RDPRGM
    // attempting to insert when the encoded form contains unsupported ops (hardware-faithful
    // "ALPHA DATA" message shown on real HP-41 card reader).
    #[error("alpha data")]
    AlphaData,
    // Phase 19 addition — Card Reader: card payload could not be encoded/decoded (corrupt
    // V41 .raw bytes, malformed .card.json, or program contains ops not representable in V41).
    #[error("card data")]
    CardData,
}
