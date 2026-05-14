//! Phase 2 ALPHA mode operations: toggle, character append, and clear.
//!
//! ALPHA register: a String in CalcState, max 24 characters.
//! All operations have Neutral lift effect (do not modify lift_enabled).
//!
//! Phase 23 additions (FN-ALPHA-01, FN-ALPHA-02): `op_arcl` / `op_asto`
//! tie ALPHA into the new `state.text_regs` packed-text sidecar map.

use crate::error::HpError;
use crate::num::HpNum;
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

/// ARCL nn (Phase 23, FN-ALPHA-01, D-23.3): append register `reg`'s formatted
/// value to the ALPHA register.
///
/// Lookup order:
/// 1. **Leading bounds check** (W-2 strengthening of D-23.3): out-of-range
///    `reg` returns `HpError::InvalidOp` BEFORE consulting `text_regs`. A
///    hand-edited autosave.json with an out-of-range `text_regs` key (threat
///    T-23-01) must NOT be able to bypass the regs-bounds check by injecting
///    a sidecar entry — the leading check makes the op symmetric with
///    `op_asto`.
/// 2. If `state.text_regs[reg]` is present → clone that packed-text shadow.
/// 3. Else format the numeric register via `format_hpnum(.., display_mode)`,
///    respecting the current FIX/SCI/ENG setting (SC#1).
///
/// The resulting text is appended char-by-char with the 24-char silent
/// discard cap that `op_alpha_append` established (Phase 2 precedent).
///
/// LiftEffect: Neutral. The ALPHA register is the only mutated state.
pub fn op_arcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    // W-2: leading bounds check — symmetric with op_asto, prevents a
    // tampered autosave from sneaking an out-of-range text_regs entry
    // through. Threat T-23-01 mitigation.
    if (reg as usize) >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let text: String = if let Some(t) = state.text_regs.get(&reg) {
        t.clone()
    } else {
        // .expect rather than .unwrap to satisfy clippy::unwrap_used; the
        // index is guaranteed valid by the leading bounds check above.
        let r = state
            .regs
            .get(reg as usize)
            .expect("bounds-checked above");
        crate::format::format_hpnum(r, &state.display_mode)
    };
    // 24-char silent discard cap — multibyte-safe (Phase 2 ALPHA invariant).
    for c in text.chars() {
        if state.alpha_reg.chars().count() >= 24 {
            break;
        }
        state.alpha_reg.push(c);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ASTO nn (Phase 23, FN-ALPHA-02, D-23.2): pack the first 6 chars of the
/// ALPHA register into register `reg`'s packed-text shadow.
///
/// Semantics:
/// - First 6 chars of `alpha_reg` via `chars().take(6)` (multibyte-safe).
/// - Sidecar write goes into `state.text_regs.insert(reg, text)`.
/// - The numeric slot `regs[reg]` is zeroed — combined with D-23.4 sidecar
///   clearing on numeric STO, this enforces the "at most one representation
///   is non-default" invariant (no drift between numeric and text shadows).
/// - The ALPHA register itself is NOT modified by ASTO.
/// - Out-of-range `reg` returns `HpError::InvalidOp` BEFORE writing the
///   sidecar (atomicity: a failing ASTO leaves both reps untouched).
///
/// LiftEffect: Neutral.
pub fn op_asto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    // Bounds check first — atomicity guard: a failing op must not write
    // to the sidecar (matches op_sto Pitfall 6 precedent).
    if (reg as usize) >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    // chars().take(6) is multibyte-safe; never byte-slices the String.
    let text: String = state.alpha_reg.chars().take(6).collect();
    state.text_regs.insert(reg, text);
    // Zero the numeric slot so RCL of an ASTO'd register pushes 0 (D-23.5,
    // documented divergence from real HP-41 which would copy raw 56 bits).
    // `if let Some` is defense-in-depth — the slot is guaranteed to exist
    // by the leading bounds check.
    if let Some(slot) = state.regs.get_mut(reg as usize) {
        *slot = HpNum::zero();
    }
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

    // ── Phase 23: ARCL / ASTO inline unit tests ──────────────────────────────

    use crate::format::format_hpnum;
    use crate::state::DisplayMode;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_arcl_appends_numeric_register_via_format_hpnum_in_fix_mode() {
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        state.regs[5] = HpNum::from(Decimal::from_str("3.14").unwrap());
        state.display_mode = DisplayMode::Fix(2);
        op_arcl(&mut state, 5).unwrap();
        let expected =
            format!("HELLO{}", format_hpnum(&state.regs[5], &DisplayMode::Fix(2)));
        assert_eq!(state.alpha_reg, expected);
    }

    #[test]
    fn test_arcl_appends_numeric_register_in_sci_mode_differs_from_fix() {
        // SC#1 verifier: switching FIX→SCI between two ARCLs of the same
        // register must produce a DIFFERENT appended suffix.
        let mut state = CalcState::new();
        state.regs[5] = HpNum::from(Decimal::from_str("3.14").unwrap());

        state.alpha_reg.clear();
        state.display_mode = DisplayMode::Fix(2);
        op_arcl(&mut state, 5).unwrap();
        let fix_output = state.alpha_reg.clone();

        state.alpha_reg.clear();
        state.display_mode = DisplayMode::Sci(3);
        op_arcl(&mut state, 5).unwrap();
        let sci_output = state.alpha_reg.clone();

        assert_ne!(
            fix_output, sci_output,
            "FIX(2) and SCI(3) must format the same numeric register differently"
        );
        assert_eq!(
            sci_output,
            format_hpnum(&HpNum::from(Decimal::from_str("3.14").unwrap()), &DisplayMode::Sci(3))
        );
    }

    #[test]
    fn test_arcl_prefers_text_regs_over_numeric_regs_when_both_set() {
        // By D-23.4 this combined state cannot arise from public ops (every
        // numeric STO clears the sidecar). Direct manipulation is fine for
        // the invariant test — we pin "text wins on lookup" so that even if
        // a future refactor regressed the sidecar clear, ARCL's lookup
        // priority itself remains correct.
        let mut state = CalcState::new();
        state.regs[5] = HpNum::from(Decimal::from_str("99.99").unwrap());
        state.text_regs.insert(5, "TEXT".to_string());
        state.alpha_reg.clear();
        op_arcl(&mut state, 5).unwrap();
        assert_eq!(state.alpha_reg, "TEXT");
    }

    #[test]
    fn test_arcl_out_of_range_reg_returns_invalid_op() {
        let mut state = CalcState::new();
        assert_eq!(state.regs.len(), 100);
        let result = op_arcl(&mut state, 200);
        assert_eq!(result, Err(HpError::InvalidOp));
        assert!(state.alpha_reg.is_empty(), "ALPHA must be unchanged on error");
    }

    #[test]
    fn test_arcl_respects_24_char_alpha_cap_silent_discard() {
        let mut state = CalcState::new();
        // 22 'A's then ARCL "BCDEF" — only the first 2 chars fit before the cap.
        state.alpha_reg = "A".repeat(22);
        state.text_regs.insert(0, "BCDEF".to_string());
        op_arcl(&mut state, 0).unwrap();
        assert_eq!(state.alpha_reg.chars().count(), 24);
        assert!(state.alpha_reg.ends_with("BC"), "first 2 chars of 'BCDEF' fit; rest silently discarded");
    }

    #[test]
    fn test_asto_packs_first_6_chars_into_text_regs() {
        let mut state = CalcState::new();
        state.alpha_reg = "GOODBYE".to_string();
        op_asto(&mut state, 12).unwrap();
        assert_eq!(state.text_regs.get(&12), Some(&"GOODBY".to_string()));
    }

    #[test]
    fn test_asto_zeroes_numeric_slot_after_packing() {
        // No-drift invariant (D-23.4): after ASTO, the numeric slot is 0.
        let mut state = CalcState::new();
        state.regs[7] = HpNum::from(Decimal::from_str("42.0").unwrap());
        state.alpha_reg = "HELLO".to_string();
        op_asto(&mut state, 7).unwrap();
        assert_eq!(state.regs[7], HpNum::zero(), "numeric slot must be zeroed");
        assert_eq!(state.text_regs.get(&7), Some(&"HELLO".to_string()));
    }

    #[test]
    fn test_asto_out_of_range_reg_returns_invalid_op() {
        let mut state = CalcState::new();
        state.alpha_reg = "OOPS".to_string();
        let result = op_asto(&mut state, 200);
        assert_eq!(result, Err(HpError::InvalidOp));
        // Atomicity: sidecar must NOT have been inserted on the error path.
        assert_eq!(
            state.text_regs.get(&200),
            None,
            "failing op_asto must not write to text_regs"
        );
    }

    #[test]
    fn test_asto_multibyte_first_6_chars_via_chars_take_6() {
        // `chars().take(6)` is the multibyte-safe slicing (Phase 2 invariant).
        // "café ré" — first 6 chars (é is 1 char, multi-byte in UTF-8).
        let mut state = CalcState::new();
        state.alpha_reg = "café résumé".to_string();
        op_asto(&mut state, 3).unwrap();
        let stored = state.text_regs.get(&3).expect("inserted");
        assert_eq!(stored.chars().count(), 6, "exactly 6 chars stored");
        assert_eq!(stored, "café r");
    }
}
