//! Phase 2 ALPHA mode operations: toggle, character append, and clear.
//!
//! ALPHA register: a String in CalcState, max 24 characters.
//! All operations have Neutral lift effect (do not modify lift_enabled).

use crate::error::HpError;
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, LiftEffect};

/// ALPHA toggle: flip alpha_mode flag.
/// When alpha_mode = true, the CLI routes keyboard chars to AlphaAppend.
/// LiftEffect: Neutral.
pub fn op_alpha_toggle(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_mode = !state.alpha_mode;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AlphaAppend: append a character to alpha_reg if under the 24-char limit.
/// HP-41 hardware silently discards characters when alpha_reg is full (no error).
/// LiftEffect: Neutral.
pub fn op_alpha_append(state: &mut CalcState, ch: char) -> Result<(), HpError> {
    // Use .chars().count() not .len() — correct for multibyte characters.
    if state.alpha_reg.chars().count() < 24 {
        state.alpha_reg.push(ch);
    }
    // Excess characters are silently discarded — HP-41 hardware behavior (see ADR/RESEARCH.md A2)
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AlphaClear: clear the alpha_reg string.
/// LiftEffect: Neutral.
pub fn op_alpha_clear(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_reg.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
