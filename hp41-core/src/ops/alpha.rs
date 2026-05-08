//! Phase 2 ALPHA mode operations: toggle, character append, and clear.
//!
//! ALPHA register: a String in CalcState, max 24 characters.
//! All operations have Neutral lift effect (do not modify lift_enabled).

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

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

/// AlphaBackspace: remove the last character from alpha_reg.
/// HP-41 hardware ← (backspace) key behavior in ALPHA mode.
/// No-op if alpha_reg is already empty — String::pop() handles this safely.
/// LiftEffect: Neutral.
pub fn op_alpha_backspace(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_reg.pop(); // no-op on empty string — correct HP-41 behavior
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;

    #[test]
    fn test_alpha_backspace_removes_last_char() {
        let mut state = CalcState::new();
        state.alpha_reg = "AB".to_string();
        op_alpha_backspace(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "A");
    }

    #[test]
    fn test_alpha_backspace_on_empty_is_noop() {
        let mut state = CalcState::new();
        assert!(state.alpha_reg.is_empty());
        op_alpha_backspace(&mut state).unwrap(); // must not panic
        assert!(state.alpha_reg.is_empty());
    }
}
