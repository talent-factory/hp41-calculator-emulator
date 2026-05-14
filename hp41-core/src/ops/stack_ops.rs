use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

/// ENTER: Duplicate X into Y, lift the stack, disable lift.
///
/// HP-41 ENTER semantics (Pitfall 4 from RESEARCH.md):
///   ENTER always lifts unconditionally (does NOT check lift_enabled).
///   After ENTER: T←Z, Z←Y, Y←X (X is duplicated in Y).
///   Then sets lift_enabled = false so the next digit entry overwrites X
///   (not lifts again).
///
/// LiftEffect: Disable
pub fn op_enter(state: &mut CalcState) -> Result<(), HpError> {
    // Unconditional stack lift — ENTER always pushes regardless of lift_enabled
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = state.stack.x.clone();
    // X is duplicated in Y; X itself is unchanged
    // Disable lift so next digit entry overwrites X
    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

/// CLX: Clear X register to zero, disable lift.
///
/// LiftEffect: Disable
pub fn op_clx(state: &mut CalcState) -> Result<(), HpError> {
    state.stack.x = HpNum::zero();
    apply_lift_effect(state, LiftEffect::Disable);
    Ok(())
}

/// CHS: Change sign of X (negate). Does not modify any other register.
///
/// LiftEffect: Neutral (HP-41 hardware: CHS during number entry appends sign;
///   here we model post-entry CHS which negates the displayed value)
pub fn op_chs(state: &mut CalcState) -> Result<(), HpError> {
    state.stack.x = state.stack.x.negate();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RDN (Roll Down): Rotate the stack so Y→X, Z→Y, T→Z, X→T.
///
/// Does NOT update LASTX (RDN is a stack reorganization, not an arithmetic result).
/// LiftEffect: Neutral
pub fn op_rdn(state: &mut CalcState) -> Result<(), HpError> {
    let old_x = state.stack.x.clone();
    state.stack.x = state.stack.y.clone();
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    state.stack.t = old_x;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// R↑ (Roll Up): Rotate the stack so X←T, T←Z, Z←Y, Y←X (mirror of Rdn).
///
/// Does NOT update LASTX (R↑ is a stack reorganization, not an arithmetic
/// result — same convention as `op_rdn`, D-19). LiftEffect: Neutral (D-20/D-25).
pub fn op_r_up(state: &mut CalcState) -> Result<(), HpError> {
    let old_x = state.stack.x.clone();
    state.stack.x = state.stack.t.clone();
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = old_x;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// X⇆Y (Swap X and Y): Exchange X and Y registers.
///
/// Does NOT update LASTX.
/// LiftEffect: Neutral
pub fn op_xy_swap(state: &mut CalcState) -> Result<(), HpError> {
    let old_x = state.stack.x.clone();
    state.stack.x = state.stack.y.clone();
    state.stack.y = old_x;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// LASTX: Recall the LASTX register value into X (with stack lift).
///
/// LASTX pushes via enter_number (respects lift_enabled for the push),
/// then enables lift so the next number entry lifts.
/// LiftEffect: Enable
pub fn op_lastx(state: &mut CalcState) -> Result<(), HpError> {
    let lastx_val = state.stack.lastx.clone();
    // Force lift enabled so enter_number always lifts the stack before placing LASTX
    state.stack.lift_enabled = true;
    enter_number(state, lastx_val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
