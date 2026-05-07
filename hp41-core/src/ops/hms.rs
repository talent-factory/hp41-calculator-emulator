// placeholder — implemented in Plan 02
use crate::error::HpError;
use crate::state::CalcState;

pub fn op_hms_to_h(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_h_to_hms(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_hms_add(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_hms_sub(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
