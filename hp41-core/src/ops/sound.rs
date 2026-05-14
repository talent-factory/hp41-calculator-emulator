//! Phase 21 sound event operations: BEEP, TONE n.
//!
//! Both ops have LiftEffect::Neutral. Output is buffered into `state.event_buffer`;
//! the CLI / GUI drains the buffer after each dispatch (Phase 25/26 wiring).
//! This module preserves the hp41-core zero-I/O invariant (no println!/eprintln!).

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

/// BEEP — push the literal `BEEP` event line into `state.event_buffer`.
/// No I/O; LiftEffect: Neutral.
pub fn op_beep(state: &mut CalcState) -> Result<(), HpError> {
    state.event_buffer.push("BEEP".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// TONE n — push the formatted `TONE n` event line into `state.event_buffer`.
/// n is `0..=9`; out-of-range values return `InvalidOp` WITHOUT pushing
/// (atomicity matches the Phase 11/12 guard-before-mutate pattern).
/// LiftEffect: Neutral.
pub fn op_tone(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 9 {
        return Err(HpError::InvalidOp);
    }
    state.event_buffer.push(format!("TONE {n}"));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_op_beep_pushes_literal() {
        let mut state = CalcState::new();
        op_beep(&mut state).unwrap();
        assert_eq!(state.event_buffer, vec!["BEEP".to_string()]);
    }

    #[test]
    fn test_op_tone_out_of_range() {
        let mut state = CalcState::new();
        let r = op_tone(&mut state, 10);
        assert!(matches!(r, Err(HpError::InvalidOp)));
        assert!(state.event_buffer.is_empty(), "no push on InvalidOp");
    }
}
