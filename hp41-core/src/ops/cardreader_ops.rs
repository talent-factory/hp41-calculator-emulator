//! Card Reader op handlers — `WDTA`, `RDTA`, `WPRGM`, `RDPRGM`.
//!
//! These handlers do not perform any disk I/O. They validate the ALPHA
//! register (hardware-faithful: empty → `ALPHA DATA` error) and stage a
//! `CardOpRequest` in `state.pending_card_op` for the frontend to drain.
//! The frontend (hp41-cli, hp41-gui) performs the actual file read/write
//! and, for read ops, calls back into the core helpers
//! `cardreader::insert_program_ops` / `cardreader::load_data_card`.
//!
//! Each handler refuses to overwrite an un-drained request — back-to-back
//! card ops (e.g. `WDTA` immediately followed by `WPRGM` inside a program
//! before the host has had a chance to act) surface as `CardData` instead of
//! silently losing the prior request.

use crate::cardreader::CardOpRequest;
use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

fn alpha_name(state: &CalcState) -> Result<String, HpError> {
    if state.alpha_reg.is_empty() {
        return Err(HpError::AlphaData);
    }
    Ok(state.alpha_reg.clone())
}

fn ensure_no_pending(state: &CalcState) -> Result<(), HpError> {
    if state.pending_card_op.is_some() {
        return Err(HpError::CardData(
            "a previous card operation is still pending — frontend must drain it first".into(),
        ));
    }
    Ok(())
}

pub fn op_wdta(state: &mut CalcState) -> Result<(), HpError> {
    ensure_no_pending(state)?;
    let name = alpha_name(state)?;
    state.pending_card_op = Some(CardOpRequest::WriteData { name });
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_rdta(state: &mut CalcState) -> Result<(), HpError> {
    ensure_no_pending(state)?;
    let name = alpha_name(state)?;
    state.pending_card_op = Some(CardOpRequest::ReadData { name });
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_wprgm(state: &mut CalcState) -> Result<(), HpError> {
    ensure_no_pending(state)?;
    let name = alpha_name(state)?;
    state.pending_card_op = Some(CardOpRequest::WriteProgram { name });
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_rdprgm(state: &mut CalcState) -> Result<(), HpError> {
    ensure_no_pending(state)?;
    let name = alpha_name(state)?;
    state.pending_card_op = Some(CardOpRequest::ReadProgram { name });
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn wdta_with_empty_alpha_returns_alpha_data() {
        let mut state = CalcState::new();
        assert_eq!(op_wdta(&mut state), Err(HpError::AlphaData));
        assert!(state.pending_card_op.is_none());
    }

    #[test]
    fn wdta_stages_write_data_request_with_alpha_name() {
        let mut state = CalcState::new();
        state.alpha_reg = "QUAD".to_string();
        op_wdta(&mut state).unwrap();
        assert_eq!(
            state.pending_card_op,
            Some(CardOpRequest::WriteData {
                name: "QUAD".to_string()
            })
        );
    }

    #[test]
    fn rdprgm_stages_read_program_request() {
        let mut state = CalcState::new();
        state.alpha_reg = "LOOP".to_string();
        op_rdprgm(&mut state).unwrap();
        assert_eq!(
            state.pending_card_op,
            Some(CardOpRequest::ReadProgram {
                name: "LOOP".to_string()
            })
        );
    }

    #[test]
    fn all_four_ops_reject_empty_alpha() {
        let mut state = CalcState::new();
        assert_eq!(op_wdta(&mut state), Err(HpError::AlphaData));
        assert_eq!(op_rdta(&mut state), Err(HpError::AlphaData));
        assert_eq!(op_wprgm(&mut state), Err(HpError::AlphaData));
        assert_eq!(op_rdprgm(&mut state), Err(HpError::AlphaData));
    }

    #[test]
    fn lift_is_neutral_so_stack_x_unchanged_for_subsequent_entry() {
        let mut state = CalcState::new();
        state.alpha_reg = "X".to_string();
        state.stack.lift_enabled = false;
        op_wprgm(&mut state).unwrap();
        // Neutral lift effect: flag stays as it was (false).
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn second_card_op_before_drain_returns_card_data() {
        // Regression guard: the first op_wdta stages a request; a follow-up
        // op_wprgm before the frontend drains must NOT silently overwrite —
        // it must surface as CardData so the host or the program sees it.
        let mut state = CalcState::new();
        state.alpha_reg = "A".into();
        op_wdta(&mut state).unwrap();
        let staged_before = state.pending_card_op.clone();
        assert!(staged_before.is_some());

        let err = op_wprgm(&mut state).unwrap_err();
        assert!(
            matches!(&err, HpError::CardData(msg) if msg.contains("pending")),
            "expected pending-overwrite diagnostic, got: {err:?}"
        );
        // The original request must still be intact for the frontend to drain.
        assert_eq!(state.pending_card_op, staged_before);
    }
}
