//! Phase 11 print operations: PRX, PRA, PRSTK.
//!
//! All three ops have LiftEffect::Neutral — they read state but do not modify the stack.
//! Output is buffered into state.print_buffer; the CLI drains the buffer after each dispatch.

use crate::error::HpError;
use crate::format::format_hpnum;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

/// PRX — print X register in current display format, right-aligned to 24 chars.
/// Pushes exactly one line to state.print_buffer. LiftEffect: Neutral.
pub fn op_prx(state: &mut CalcState) -> Result<(), HpError> {
    let line = format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode));
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// PRA — print ALPHA register, left-aligned to 24 chars.
/// Pushes exactly one line to state.print_buffer. LiftEffect: Neutral.
/// NOTE: Does NOT use format_alpha() which truncates to 12 chars. Uses 24-char width directly.
pub fn op_pra(state: &mut CalcState) -> Result<(), HpError> {
    // Take at most 24 chars from alpha_reg (HP-41 ALPHA is max 24 chars but guard regardless).
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    let line = format!("{alpha:<24}");
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// PRSTK — print full stack T/Z/Y/X/LASTX/ALPHA, 6 lines of 24 chars each.
/// Pushes 6 lines to state.print_buffer. Line format: left-aligned 7-char label + 17-char value.
/// LiftEffect: Neutral.
pub fn op_prstk(state: &mut CalcState) -> Result<(), HpError> {
    let mode = &state.display_mode.clone();
    // Numeric lines: 7-char label (left) + 17-char formatted value (right) = 24 chars total.
    // format_hpnum output for SCI 9 widest case is "-1.234567890E-99" = 16 chars → fits in :>17.
    let lines: [String; 6] = [
        format!("{:<7}{:>17}", "T:", format_hpnum(&state.stack.t, mode)),
        format!("{:<7}{:>17}", "Z:", format_hpnum(&state.stack.z, mode)),
        format!("{:<7}{:>17}", "Y:", format_hpnum(&state.stack.y, mode)),
        format!("{:<7}{:>17}", "X:", format_hpnum(&state.stack.x, mode)),
        format!(
            "{:<7}{:>17}",
            "LASTX:",
            format_hpnum(&state.stack.lastx, mode)
        ),
        {
            // ALPHA line is left-aligned value (not right-aligned), max 17 chars.
            let alpha = state.alpha_reg.chars().take(17).collect::<String>();
            format!("{:<7}{:<17}", "ALPHA:", alpha)
        },
    ];
    for line in lines {
        state.print_buffer.push(line);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
