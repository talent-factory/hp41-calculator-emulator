//! Phase 2 ALPHA mode operations: toggle, character append, and clear.
//!
//! ALPHA register: a String in CalcState, max 24 characters.
//! All operations have Neutral lift effect (do not modify lift_enabled).
//!
//! Phase 23 additions (FN-ALPHA-01, FN-ALPHA-02): `op_arcl` / `op_asto`
//! tie ALPHA into the new `state.text_regs` packed-text sidecar map.
//!
//! Phase 23 plan 02 (FN-ALPHA-03..06): `op_atox` / `op_xtoa` / `op_arot` /
//! `op_posa` complete the ALPHA-register expansion. These four ops touch
//! only `state.alpha_reg` and `state.stack.x` — no new persistent state.
//! See `.planning/phases/23-alpha-operations/23-CONTEXT.md` decisions
//! D-23.7 (POSA), D-23.8/9 (AROT), D-23.10 (ATOX), D-23.11 (XTOA), D-23.16
//! (LiftEffect summary).

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
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
        let r = state.regs.get(reg as usize).expect("bounds-checked above");
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

/// ATOX (Phase 23, FN-ALPHA-03, D-23.10): pop the first ALPHA char and push
/// its Unicode codepoint into X.
///
/// Semantics:
/// - Read `state.alpha_reg.chars().next()`. If `Some(c)`, rebuild the ALPHA
///   register with chars[0] dropped (multibyte-safe — never byte-slices),
///   and set the code to `u32::from(c).min(255) as i32` (8-bit cap, D-23.10).
/// - If `None` (empty ALPHA), set code = 0.
/// - Routes through `crate::stack::enter_number` — the single source of truth
///   for stack-lift in `hp41-core` (mirrors `op_pi`'s lift-then-push ordering
///   precedent in math.rs ~line 297). Forcing `lift_enabled = true` ahead of
///   the call guarantees the push lifts X→Y unconditionally; the trailing
///   `apply_lift_effect(Enable)` declares the post-op lift state.
///
/// Documented divergences:
/// - 8-bit cap: multibyte first-char codepoints > 255 are capped to 255
///   (e.g., Σ U+03A3 = decimal 931 → 255). HP-41 hardware glyphs at codes
///   128..=255 are not reverse-mappable from arbitrary Unicode; this is the
///   conservative choice.
/// - Empty ALPHA pushes 0 with the lift still enabled (faithful HP-41
///   behaviour — lift fires regardless of whether ALPHA had a char).
///
/// LiftEffect: Enable.
pub fn op_atox(state: &mut CalcState) -> Result<(), HpError> {
    let code: i32 = match state.alpha_reg.chars().next() {
        Some(c) => {
            // Rebuild via chars() to drop the first char — never byte-slice
            // (multibyte safety, Phase 2 invariant).
            let mut chars: Vec<char> = state.alpha_reg.chars().collect();
            chars.remove(0);
            state.alpha_reg = chars.into_iter().collect();
            // 8-bit cap (D-23.10): codepoints > 255 saturate to 255.
            u32::from(c).min(255) as i32
        }
        None => 0,
    };
    // WR-02: route through enter_number (the canonical stack-lift helper)
    // instead of direct-assigning stack fields. Mirrors op_pi's structural
    // pattern (math.rs ~line 297): force lift, push via enter_number, then
    // declare the post-op lift effect. Future refactors of enter_number
    // (e.g. an x_text shadow channel hook) now reach ATOX automatically.
    state.stack.lift_enabled = true;
    enter_number(state, HpNum::from(code));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// XTOA (Phase 23, FN-ALPHA-04, D-23.11): convert X mod 256 to a character
/// and append it to ALPHA. X is NOT consumed.
///
/// Semantics:
/// - `i_dec = state.stack.x.trunc_int().inner()` — truncate-toward-zero (faithful
///   HP-41CV; matches AROT's silent-trunc per D-23.9).
/// - `i_i64: i64 = i_dec.try_into().map_err(|_| HpError::InvalidOp)?` — rejects
///   Decimal::MAX-class overflow (D-23.14 zero-panic policy).
/// - `code: u32 = i_i64.rem_euclid(256) as u32` — `rem_euclid` (NOT `%`) so
///   negative X normalises into 0..=255 correctly.
/// - `c = if code < 128 { code as u8 as char } else { '?' }` — D-23.11
///   documented divergence: HP-41 upper-ASCII glyphs (Σ, λ, ⊢, etc.) are not
///   in our String/UTF-8 model; we use '?' as a placeholder.
/// - Silent 24-char ALPHA cap (`chars().count() < 24`) mirrors `op_alpha_append`
///   (Phase 2 invariant).
/// - X register is preserved (LiftEffect::Neutral, D-23.16).
///
/// LiftEffect: Neutral.
pub fn op_xtoa(state: &mut CalcState) -> Result<(), HpError> {
    let i_dec = state.stack.x.trunc_int().inner();
    let i_i64: i64 = i_dec.try_into().map_err(|_| HpError::InvalidOp)?;
    let code: u32 = (i_i64.rem_euclid(256)) as u32;
    let c: char = if code < 128 { code as u8 as char } else { '?' };
    if state.alpha_reg.chars().count() < 24 {
        state.alpha_reg.push(c);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// AROT (Phase 23, FN-ALPHA-05, D-23.8 / D-23.9): rotate ALPHA by X chars
/// (positive = left, negative = right). X is NOT consumed.
///
/// Semantics:
/// - `apply_lift_effect(state, LiftEffect::Neutral)` is called EARLY so that
///   the early-return on empty-ALPHA / overflow-error still settles the lift
///   state (Phase 21/22 precedent).
/// - Empty ALPHA → no-op, X preserved.
/// - `n_dec = state.stack.x.trunc_int().inner()` — silent truncation toward
///   zero (D-23.9 faithful HP-41CV; STRICTER `Err(InvalidOp)` for non-integer
///   X belongs to POSA, not AROT — see `op_posa`).
/// - `n = n_i64.rem_euclid(len as i64) as usize` — `rem_euclid` (NOT `%`)
///   normalises negative N: AROT -1 of "HELLO" (len 5) → rem_euclid(-1, 5) = 4
///   → ALPHA = chars[4..] ++ chars[..4] = "OHELL".
/// - Rebuild via `chars().collect::<Vec<char>>()` then re-join (multibyte-safe).
///
/// LiftEffect: Neutral.
pub fn op_arot(state: &mut CalcState) -> Result<(), HpError> {
    // Settle lift first so early-return paths (empty ALPHA, overflow) leave
    // lift_enabled in a Neutral state.
    apply_lift_effect(state, LiftEffect::Neutral);
    let len = state.alpha_reg.chars().count();
    if len == 0 {
        return Ok(()); // empty ALPHA no-op (D-23.9)
    }
    let n_dec = state.stack.x.trunc_int().inner();
    let n_i64: i64 = n_dec.try_into().map_err(|_| HpError::InvalidOp)?;
    // rem_euclid (NOT %) — handles negative N: rem_euclid(-1, 5) = 4.
    let n = n_i64.rem_euclid(len as i64) as usize;
    let chars: Vec<char> = state.alpha_reg.chars().collect();
    state.alpha_reg = chars[n..].iter().chain(chars[..n].iter()).collect();
    Ok(())
}

/// POSA (Phase 23, FN-ALPHA-06, D-23.7): single-char POSA. X must be an
/// integer ASCII codepoint in 0..=127; the result REPLACES X with the
/// 0-indexed position of the first matching char in ALPHA, or `-1` if not
/// found.
///
/// Semantics:
/// - `i = state.stack.x.trunc_int()` then `if i != x { return InvalidOp; }` —
///   STRICTER than AROT's silent-trunc (D-23.7 vs D-23.9). Documented
///   divergence: AROT is faithful HP-41CV (silently truncates non-integer X);
///   POSA is stricter because position-lookup with a fractional codepoint is
///   semantically meaningless.
/// - `code_i64: i64 = i.inner().try_into().map_err(|_| HpError::InvalidOp)?`
///   then `(0..=127).contains(&code_i64)` gate (ASCII range).
/// - `pos = state.alpha_reg.chars().position(|c| c == needle).map(|p| p as i32)
///   .unwrap_or(-1)` — `chars()` is multibyte-safe.
/// - `-1` for not-found is SC#5's explicit wording. Other HP-41 sources return
///   the haystack length; we pick -1 per ROADMAP SC#5.
/// - Multi-char POSA is deferred to v3.x per D-23.6 — would require a typed-
///   stack `x_text: Option<String>` channel that our HpNum=Decimal model
///   cannot preserve.
///
/// LiftEffect: Disable (replaces X with the computed position).
pub fn op_posa(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    let i = x.trunc_int();
    if i != x {
        return Err(HpError::InvalidOp); // non-integer X — stricter than AROT (D-23.7)
    }
    let code_i64: i64 = i.inner().try_into().map_err(|_| HpError::InvalidOp)?;
    if !(0..=127).contains(&code_i64) {
        return Err(HpError::InvalidOp); // ASCII range gate (D-23.7)
    }
    let needle = (code_i64 as u8) as char;
    let pos: i32 = state
        .alpha_reg
        .chars()
        .position(|c| c == needle)
        .map(|p| p as i32)
        .unwrap_or(-1); // SC#5 explicit wording — -1, not haystack length.
    state.stack.x = HpNum::from(pos);
    apply_lift_effect(state, LiftEffect::Disable);
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
        let expected = format!(
            "HELLO{}",
            format_hpnum(&state.regs[5], &DisplayMode::Fix(2))
        );
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
            format_hpnum(
                &HpNum::from(Decimal::from_str("3.14").unwrap()),
                &DisplayMode::Sci(3)
            )
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
        assert!(
            state.alpha_reg.is_empty(),
            "ALPHA must be unchanged on error"
        );
    }

    #[test]
    fn test_arcl_respects_24_char_alpha_cap_silent_discard() {
        let mut state = CalcState::new();
        // 22 'A's then ARCL "BCDEF" — only the first 2 chars fit before the cap.
        state.alpha_reg = "A".repeat(22);
        state.text_regs.insert(0, "BCDEF".to_string());
        op_arcl(&mut state, 0).unwrap();
        assert_eq!(state.alpha_reg.chars().count(), 24);
        assert!(
            state.alpha_reg.ends_with("BC"),
            "first 2 chars of 'BCDEF' fit; rest silently discarded"
        );
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

    // ── Phase 23 plan 02: ATOX / XTOA / AROT / POSA inline unit tests ───────

    #[test]
    fn test_atox_pops_first_char_pushes_ascii_code_with_lift() {
        // SC#3 forward: ALPHA="ABC" → ATOX → X=65 ('A'), Y=prior_X, ALPHA="BC".
        let mut state = CalcState::new();
        state.alpha_reg = "ABC".to_string();
        state.stack.x = HpNum::from(99);
        op_atox(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "BC", "first char dropped");
        assert_eq!(state.stack.x, HpNum::from(65), "X = ASCII 'A'");
        assert_eq!(state.stack.y, HpNum::from(99), "prior X lifted to Y");
    }

    #[test]
    fn test_atox_empty_alpha_pushes_zero_with_lift() {
        let mut state = CalcState::new();
        assert!(state.alpha_reg.is_empty());
        state.stack.x = HpNum::from(7);
        op_atox(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::from(0), "empty ALPHA → X = 0");
        assert_eq!(state.stack.y, HpNum::from(7), "lift still fires on empty");
        assert!(state.alpha_reg.is_empty(), "ALPHA stays empty");
    }

    #[test]
    fn test_atox_multibyte_first_char_capped_at_255() {
        // Σ is U+03A3, decimal 931 — must saturate to 255 (D-23.10 8-bit cap).
        let mut state = CalcState::new();
        state.alpha_reg = "Σabc".to_string();
        op_atox(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::from(255), "Σ codepoint capped to 255");
        assert_eq!(state.alpha_reg, "abc", "first multibyte char dropped");
    }

    #[test]
    fn test_xtoa_appends_ascii_char_x_preserved() {
        // SC#3 reverse: X=66 ('B'), ALPHA="" → XTOA → ALPHA="B", X still 66.
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(66);
        op_xtoa(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "B");
        assert_eq!(state.stack.x, HpNum::from(66), "X preserved (Neutral lift)");
    }

    #[test]
    fn test_xtoa_upper_ascii_maps_to_question_mark() {
        // D-23.11: codes 128..=255 map to '?' placeholder (HP-41 upper-ASCII
        // glyphs are not in our String/UTF-8 model).
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(200);
        op_xtoa(&mut state).unwrap();
        assert!(state.alpha_reg.ends_with('?'), "200 → '?'");
    }

    #[test]
    fn test_xtoa_silent_24_char_cap() {
        // 24-char ALPHA cap: silent discard on append (Phase 2 op_alpha_append).
        let mut state = CalcState::new();
        state.alpha_reg = "A".repeat(24);
        state.stack.x = HpNum::from(66);
        op_xtoa(&mut state).unwrap();
        assert_eq!(state.alpha_reg.chars().count(), 24);
        assert!(
            state.alpha_reg.ends_with('A'),
            "no 'B' appended past the cap"
        );
    }

    #[test]
    fn test_arot_positive_n_left_rotation() {
        // SC#4 forward: AROT 2 of "HELLO" → "LLOHE".
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        state.stack.x = HpNum::from(2);
        op_arot(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "LLOHE");
    }

    #[test]
    fn test_arot_negative_n_right_rotation() {
        // SC#4 reverse: AROT -1 of "HELLO" → "OHELL" (rem_euclid(-1, 5) = 4).
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        state.stack.x = HpNum::from(-1);
        op_arot(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "OHELL");
    }

    #[test]
    fn test_arot_n_greater_than_len_modulo() {
        // AROT 7 of "HELLO" (len 5) → AROT 2 ≡ "LLOHE".
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        state.stack.x = HpNum::from(7);
        op_arot(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "LLOHE");
    }

    #[test]
    fn test_arot_empty_alpha_is_noop() {
        let mut state = CalcState::new();
        assert!(state.alpha_reg.is_empty());
        state.stack.x = HpNum::from(3);
        op_arot(&mut state).unwrap();
        assert!(state.alpha_reg.is_empty(), "empty ALPHA stays empty");
        assert_eq!(state.stack.x, HpNum::from(3), "X preserved");
    }

    #[test]
    fn test_arot_x_preserved_neutral_lift() {
        // D-23.16: AROT is LiftEffect::Neutral — X must be unchanged after rotation.
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        state.stack.x = HpNum::from(2);
        let x_before = state.stack.x.clone();
        op_arot(&mut state).unwrap();
        assert_eq!(state.stack.x, x_before, "X preserved across AROT");
    }

    #[test]
    fn test_posa_finds_single_char() {
        // SC#5: ALPHA="THE QUICK BROWN FOX", X=81 ('Q', position 4) → POSA → X=4.
        let mut state = CalcState::new();
        state.alpha_reg = "THE QUICK BROWN FOX".to_string();
        state.stack.x = HpNum::from(81);
        op_posa(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::from(4), "'Q' at position 4");
    }

    #[test]
    fn test_posa_not_found_returns_minus_one() {
        // SC#5 negative path: -1 (not haystack length — explicit ROADMAP wording).
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        state.stack.x = HpNum::from(90); // 'Z' — not in "HELLO"
        op_posa(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::from(-1));
    }

    #[test]
    fn test_posa_non_integer_x_returns_invalid_op() {
        // D-23.7: stricter than AROT — non-integer X is rejected, not truncated.
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        let x_orig = HpNum::from(Decimal::from_str("2.5").unwrap());
        state.stack.x = x_orig.clone();
        let result = op_posa(&mut state);
        assert_eq!(result, Err(HpError::InvalidOp));
        assert_eq!(state.alpha_reg, "HELLO", "ALPHA unchanged on error");
        assert_eq!(state.stack.x, x_orig, "X unchanged on error");
    }

    #[test]
    fn test_posa_out_of_range_x_returns_invalid_op() {
        // D-23.7: X must be in 0..=127. 200 is out of ASCII range.
        let mut state = CalcState::new();
        state.alpha_reg = "HELLO".to_string();
        state.stack.x = HpNum::from(200);
        let result = op_posa(&mut state);
        assert_eq!(result, Err(HpError::InvalidOp));
    }
}
