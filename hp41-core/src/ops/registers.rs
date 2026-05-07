//! Phase 2 storage register operations: STO, RCL, STO+/-/×/÷, CLREG.
//!
//! Register addresses are 0–99 (0-indexed). Addresses ≥ 100 return InvalidOp.
//! STO and STO-arith: Neutral lift (do not modify lift_enabled).
//! RCL: Enable lift (like PushNum — places a value on the stack).

use crate::error::HpError;
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::ops::StoArithKind;

/// STO n: copy X register into storage register n. Stack unchanged.
/// LiftEffect: Neutral. LASTX: not saved (STO is not an arithmetic operation).
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    state.regs[reg as usize] = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCL n: recall register n into X (with stack lift if lift_enabled).
/// LiftEffect: Enable. LASTX: not saved.
pub fn op_rcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    let val = state.regs[reg as usize].clone();
    // Force lift_enabled = true so enter_number performs the stack lift.
    // This matches HP-41 hardware: RCL always lifts regardless of prior state.
    state.stack.lift_enabled = true;
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// STO+/−/×/÷ n: apply arithmetic to register n using X.
/// R[n] ← R[n] OP X. Stack and X are unchanged.
/// LiftEffect: Neutral. LASTX: not saved.
///
/// IMPORTANT: compute new value FIRST, write ONLY on success (atomicity guarantee).
pub fn op_sto_arith(state: &mut CalcState, reg: u8, kind: StoArithKind) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    // Compute first — do NOT write to state.regs[reg] until we know the op succeeds.
    let new_val = match kind {
        StoArithKind::Add => state.regs[reg as usize].checked_add(&state.stack.x)?,
        StoArithKind::Sub => state.regs[reg as usize].checked_sub(&state.stack.x)?,
        StoArithKind::Mul => state.regs[reg as usize].checked_mul(&state.stack.x)?,
        StoArithKind::Div => state.regs[reg as usize].checked_div(&state.stack.x)?,
    };
    // Write only after successful computation (Pitfall 6 guard)
    state.regs[reg as usize] = new_val;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLREG: clear all storage registers to zero.
/// LiftEffect: Neutral.
pub fn op_clreg(state: &mut CalcState) -> Result<(), HpError> {
    state.regs = vec![crate::num::HpNum::zero(); 100];
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
