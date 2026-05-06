use crate::error::HpError;
use crate::state::CalcState;
use crate::stack::binary_result;

/// Add: Y + X → X
/// LiftEffect: Enable (via binary_result)
pub fn op_add(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_add(&state.stack.x)?;
    binary_result(state, result);
    Ok(())
}

/// Subtract: Y - X → X
/// LiftEffect: Enable (via binary_result)
/// HP-41 convention: Y is the minuend, X the subtrahend.
pub fn op_sub(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_sub(&state.stack.x)?;
    binary_result(state, result);
    Ok(())
}

/// Multiply: Y × X → X
/// LiftEffect: Enable (via binary_result)
pub fn op_mul(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_mul(&state.stack.x)?;
    binary_result(state, result);
    Ok(())
}

/// Divide: Y ÷ X → X
/// LiftEffect: Enable (via binary_result)
/// Returns Err(HpError::DivideByZero) if X is zero (checked before rust_decimal is called).
pub fn op_div(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_div(&state.stack.x)?;
    binary_result(state, result);
    Ok(())
}
