// placeholder — implemented in Plan 02
use crate::error::HpError;
use crate::state::CalcState;

pub fn op_sigma_plus(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_sigma_minus(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_mean(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_sdev(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_lr(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_yhat(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_corr(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
pub fn op_cl_sigma_stat(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }
