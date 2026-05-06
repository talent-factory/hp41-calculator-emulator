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
}
