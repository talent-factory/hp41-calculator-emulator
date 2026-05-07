use crate::num::HpNum;
use crate::state::CalcState;

/// Stack-lift effect that every operation must declare.
///
/// Enable  — set lift_enabled = true  (most arithmetic operations, RCL)
/// Disable — set lift_enabled = false (ENTER, CLX)
/// Neutral — leave lift_enabled unchanged (display/mode ops: VIEW, PRGM toggle, etc.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiftEffect {
    Enable,
    Disable,
    Neutral,
}

/// Apply a declared lift effect to the calculator state.
/// This is the single authority for modifying lift_enabled.
pub fn apply_lift_effect(state: &mut CalcState, effect: LiftEffect) {
    match effect {
        LiftEffect::Enable => state.stack.lift_enabled = true,
        LiftEffect::Disable => state.stack.lift_enabled = false,
        LiftEffect::Neutral => { /* intentional no-op */ }
    }
}

/// Enter a numeric value into X, respecting the current lift_enabled flag.
///
/// If lift_enabled: push the stack (T←Z, Z←Y, Y←X) then write X.
/// If not lift_enabled: overwrite X without lifting.
/// NOTE: number entry does NOT modify lift_enabled (that is the op's responsibility).
pub fn enter_number(state: &mut CalcState, value: HpNum) {
    if state.stack.lift_enabled {
        // HP-41 hardware: T is duplicated (old T is lost), not rotated out
        state.stack.t = state.stack.z.clone();
        state.stack.z = state.stack.y.clone();
        state.stack.y = state.stack.x.clone();
    }
    state.stack.x = value;
    // lift_enabled is NOT changed here — the calling operation sets it after entry
}

/// Record the result of a unary (single-operand) operation.
///
/// Saves current X to LASTX BEFORE overwriting X — same as binary_result.
/// Unlike binary_result, Y, Z, and T are NOT modified (unary = no stack drop).
/// Always enables lift after a unary result.
///
/// LiftEffect: Enable (implicit — unary_result always sets lift_enabled = true)
pub fn unary_result(state: &mut CalcState, result: HpNum) {
    // Capture X into LASTX BEFORE writing result — critical ordering
    state.stack.lastx = state.stack.x.clone();
    // Place result in X; Y, Z, T remain unchanged
    state.stack.x = result;
    // Unary results always enable lift (same behavior as binary)
    state.stack.lift_enabled = true;
}

/// Record the result of a binary (two-operand) operation.
///
/// Saves current X to LASTX BEFORE overwriting X.
/// Rotates: X←result, Y←Z, Z←T (T stays unchanged — HP-41 hardware behavior).
/// Always enables lift after a binary result.
pub fn binary_result(state: &mut CalcState, result: HpNum) {
    // Capture X into LASTX BEFORE writing result — critical ordering
    state.stack.lastx = state.stack.x.clone();
    // Place result in X
    state.stack.x = result;
    // Rotate Y and Z up (consuming Y, which held the second operand)
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    // T stays (HP-41 hardware: T is duplicated, not consumed, on stack drop)
    // Binary operations always enable lift
    state.stack.lift_enabled = true;
}
