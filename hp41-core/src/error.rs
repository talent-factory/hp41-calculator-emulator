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
}
