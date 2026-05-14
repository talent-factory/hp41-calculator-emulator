//! Phase 21 display control operations: VIEW, AVIEW, PROMPT, AON, AOFF, CLD.
//!
//! All ops have LiftEffect::Neutral. Output goes to `state.display_override`
//! (Option<String>); the frontend (CLI/GUI) renders it. PROMPT additionally
//! exits run_loop — see `ops/program.rs`.

use crate::error::HpError;
use crate::format::format_hpnum;
use crate::ops::flags::{flag_clear, flag_set};
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

/// VIEW n — show the formatted value of storage register n on the display.
/// Writes `Some(format_hpnum(...))` to `state.display_override`; stack untouched.
/// LiftEffect: Neutral. Returns `InvalidOp` for reg >= state.regs.len()
/// (Phase 22 D-22.11.1 — honors current SIZE; was hardcoded 100).
pub fn op_view(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let val = state
        .regs
        .get(reg as usize)
        .ok_or(HpError::InvalidOp)?
        .clone();
    state.display_override = Some(format_hpnum(&val, &state.display_mode));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AVIEW — show the ALPHA register (first 24 chars) on the display.
/// LiftEffect: Neutral.
pub fn op_aview(state: &mut CalcState) -> Result<(), HpError> {
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    state.display_override = Some(alpha);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// PROMPT — interactive dispatch: write ALPHA (first 24 chars) to display_override.
/// The PAUSE-program semantic is handled by `run_loop` in `ops/program.rs`, which
/// detects `Op::Prompt` directly and writes-then-breaks. Inside `run_loop` this
/// function is NOT called (the run_loop arm has its own inline body).
/// LiftEffect: Neutral.
pub fn op_prompt(state: &mut CalcState) -> Result<(), HpError> {
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    state.display_override = Some(alpha);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AON — enable ALPHA auto-display by setting system flag 48 (HP-42S compat).
/// Phase 21 only stores the bit; the user-visible "ALPHA after every op" effect
/// is a Phase 25/26 frontend concern.
/// LiftEffect: Neutral.
pub fn op_aon(state: &mut CalcState) -> Result<(), HpError> {
    state.flags = flag_set(state.flags, 48);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AOFF — disable ALPHA auto-display by clearing system flag 48.
/// LiftEffect: Neutral.
pub fn op_aoff(state: &mut CalcState) -> Result<(), HpError> {
    state.flags = flag_clear(state.flags, 48);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLD — explicit clear of `display_override`. Mostly redundant with the
/// dispatch-top reset, but provides a programmable way to clear the override
/// without dispatching another op. LiftEffect: Neutral.
pub fn op_cld(state: &mut CalcState) -> Result<(), HpError> {
    state.display_override = None;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_op_view_register_out_of_range_returns_invalid_op() {
        let mut state = CalcState::new();
        let r = op_view(&mut state, 100);
        assert!(matches!(r, Err(HpError::InvalidOp)));
        assert!(
            state.display_override.is_none(),
            "display_override unchanged on error"
        );
    }

    #[test]
    fn test_op_aview_truncates_to_24_chars() {
        let mut state = CalcState::new();
        state.alpha_reg = "X".repeat(30);
        op_aview(&mut state).unwrap();
        assert_eq!(
            state.display_override.as_deref().unwrap().chars().count(),
            24,
            "AVIEW must truncate alpha_reg to 24 chars"
        );
    }

    #[test]
    fn test_op_cld_clears_only_override() {
        use crate::num::HpNum;
        let mut state = CalcState::new();
        state.display_override = Some("STALE".to_string());
        state.alpha_reg = "Y".to_string();
        state.stack.x = HpNum::from(42i32);
        op_cld(&mut state).unwrap();
        assert!(state.display_override.is_none());
        assert_eq!(state.alpha_reg, "Y", "CLD must not touch alpha_reg");
        assert_eq!(state.stack.x, HpNum::from(42i32), "CLD must not touch X");
    }

    #[test]
    fn test_op_aon_aoff_toggle_flag_48() {
        use crate::ops::flags::flag_get;
        let mut state = CalcState::new();
        op_aon(&mut state).unwrap();
        assert!(flag_get(state.flags, 48));
        op_aoff(&mut state).unwrap();
        assert!(!flag_get(state.flags, 48));
    }

    #[test]
    fn test_op_prompt_interactive_writes_alpha_to_override() {
        let mut state = CalcState::new();
        state.alpha_reg = "READY?".to_string();
        op_prompt(&mut state).unwrap();
        assert_eq!(state.display_override.as_deref(), Some("READY?"));
    }
}
