//! Phase 23 plan 02 (FN-ALPHA-03..06) integration tests — drive each op
//! through `dispatch(state, &Op::Xxx)` so the dispatch arms in `ops/mod.rs`
//! are covered alongside the function bodies in `ops/alpha.rs`.
//!
//! Covers the plan's documented success criteria and intentional
//! divergences:
//!   #1  ATOX pops first char, pushes ASCII code with lift Enable (SC#3 forward)
//!   #2  XTOA appends ASCII char, X preserved (SC#3 reverse)
//!   #3  ATOX/XTOA round-trip exact for ASCII 0..=127 (SC#3 property)
//!   #4  XTOA upper-ASCII (128..=255) maps to '?' placeholder (D-23.11)
//!   #5  AROT left rotation: AROT 2 of "HELLO" → "LLOHE" (SC#4 forward)
//!   #6  AROT right rotation via rem_euclid: AROT -1 of "HELLO" → "OHELL" (SC#4)
//!   #7  AROT modulo: AROT 7 of "HELLO" → "LLOHE" (same as N=2)
//!   #8  AROT empty ALPHA is a no-op, X preserved (D-23.9 edge case)
//!   #9  POSA finds single char: 'Q' in "THE QUICK BROWN FOX" → X=4 (SC#5)
//!   #10 POSA not-found returns -1 (SC#5 explicit wording — not haystack length)
//!   #11 POSA rejects non-integer X (D-23.7 stricter than AROT)
//!   #12 POSA rejects out-of-range X (ASCII gate 0..=127)
//!   #13 AROT silently truncates non-integer X (D-23.9 — intentional divergence
//!       from POSA's strict rejection; both behaviors pinned mechanically)
//!
//! Test modules allow unwrap (CLAUDE.md "Zero panics" applies to production
//! code only; tests carry the precedent #[allow(clippy::unwrap_used)]).

#![allow(clippy::unwrap_used)]

use hp41_core::num::HpNum;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use hp41_core::HpError;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Test #1 — SC#3 forward. ATOX pops the first ALPHA char and pushes its
/// ASCII code onto X with stack lift Enable. The prior X moves into Y.
#[test]
fn atox_pops_first_char_pushes_ascii_with_lift() {
    let mut state = CalcState::new();
    state.alpha_reg = "A".to_string();
    state.stack.x = HpNum::from(99);
    dispatch(&mut state, Op::Atox).unwrap();
    assert!(state.alpha_reg.is_empty(), "first char dropped → ALPHA empty");
    assert_eq!(state.stack.x, HpNum::from(65), "X = ASCII 'A' = 65");
    assert_eq!(state.stack.y, HpNum::from(99), "prior X lifted to Y");
}

/// Test #2 — SC#3 reverse. XTOA appends the ASCII char of X (mod 256) to
/// ALPHA without consuming X.
#[test]
fn xtoa_appends_ascii_char_preserves_x() {
    let mut state = CalcState::new();
    assert!(state.alpha_reg.is_empty());
    state.stack.x = HpNum::from(66);
    dispatch(&mut state, Op::Xtoa).unwrap();
    assert_eq!(state.alpha_reg, "B");
    assert_eq!(state.stack.x, HpNum::from(66), "X preserved (Neutral lift)");
}

/// Test #3 — SC#3 property: round-trip exact for ASCII 0..=127. Property-
/// style with four sample values (no `proptest` dep needed for a 4-shot
/// suite — the property is structural, not statistical).
#[test]
fn atox_xtoa_round_trip_preserves_ascii_0_to_127() {
    for code in [32_i32, 65, 97, 126] {
        let mut state = CalcState::new();
        state.alpha_reg = (code as u8 as char).to_string();
        dispatch(&mut state, Op::Atox).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(code),
            "ATOX of {code} must push exactly {code}"
        );

        // Clear ALPHA, then XTOA the same code back. X is still `code`
        // (preserved by Atox lift — actually X was overwritten by Atox push;
        // but ATOX above set X = code, and XTOA preserves X).
        state.alpha_reg.clear();
        dispatch(&mut state, Op::Xtoa).unwrap();
        assert_eq!(
            state.alpha_reg,
            (code as u8 as char).to_string(),
            "XTOA of {code} must reproduce the original char"
        );
        assert_eq!(state.stack.x, HpNum::from(code), "X preserved across XTOA");
    }
}

/// Test #4 — D-23.11 divergence. Codes 128..=255 map to '?' (HP-41 upper-
/// ASCII glyphs are not in our String/UTF-8 model).
#[test]
fn xtoa_upper_ascii_maps_to_question_placeholder() {
    for x_value in [128_i32, 200, 255] {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(x_value);
        dispatch(&mut state, Op::Xtoa).unwrap();
        assert!(
            state.alpha_reg.ends_with('?'),
            "X={x_value} → '?' placeholder (got ALPHA={:?})",
            state.alpha_reg
        );
    }
}

/// Test #5 — SC#4 forward. AROT 2 of "HELLO" → "LLOHE" (positive N = left
/// rotation: chars 'H','E' move from front to back).
#[test]
fn arot_left_rotation_two_of_hello_produces_lloghe() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    state.stack.x = HpNum::from(2);
    dispatch(&mut state, Op::Arot).unwrap();
    assert_eq!(state.alpha_reg, "LLOHE", "SC#4 forward wording");
    assert_eq!(state.stack.x, HpNum::from(2), "X preserved (Neutral lift)");
}

/// Test #6 — SC#4 reverse. AROT -1 of "HELLO" → "OHELL" (negative N = right
/// rotation; `rem_euclid(-1, 5) = 4` → "OHELL" via chars[4..] ++ chars[..4]).
#[test]
fn arot_right_rotation_negative_one_of_hello_produces_ohell() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    state.stack.x = HpNum::from(-1);
    dispatch(&mut state, Op::Arot).unwrap();
    assert_eq!(state.alpha_reg, "OHELL", "SC#4 reverse wording");
}

/// Test #7 — `|N| > len` normalises via `rem_euclid`. AROT 7 of "HELLO"
/// (len 5) ≡ AROT 2 → "LLOHE".
#[test]
fn arot_modulo_handles_n_greater_than_len() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    state.stack.x = HpNum::from(7);
    dispatch(&mut state, Op::Arot).unwrap();
    assert_eq!(state.alpha_reg, "LLOHE", "AROT 7 ≡ AROT (7 mod 5) = AROT 2");
}

/// Test #8 — empty ALPHA is a no-op. X is preserved (Neutral lift).
#[test]
fn arot_empty_alpha_is_noop_preserves_x() {
    let mut state = CalcState::new();
    assert!(state.alpha_reg.is_empty());
    state.stack.x = HpNum::from(3);
    dispatch(&mut state, Op::Arot).unwrap();
    assert!(state.alpha_reg.is_empty(), "ALPHA stays empty");
    assert_eq!(state.stack.x, HpNum::from(3), "X preserved");
}

/// Test #9 — SC#5. POSA of `ALPHA="THE QUICK BROWN FOX"` with X=81 ('Q')
/// produces X=4 (0-indexed position of the first 'Q').
#[test]
fn posa_single_char_finds_position_4_for_q_in_the_quick() {
    let mut state = CalcState::new();
    state.alpha_reg = "THE QUICK BROWN FOX".to_string();
    state.stack.x = HpNum::from(81); // ASCII 'Q' = 81
    dispatch(&mut state, Op::Posa).unwrap();
    assert_eq!(state.stack.x, HpNum::from(4), "SC#5: 'Q' is at index 4");
}

/// Test #10 — SC#5 negative path. POSA returns -1 for not-found (NOT
/// haystack length — explicit ROADMAP wording).
#[test]
fn posa_not_found_returns_minus_one() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    state.stack.x = HpNum::from(90); // 'Z' — not in "HELLO"
    dispatch(&mut state, Op::Posa).unwrap();
    assert_eq!(state.stack.x, HpNum::from(-1));
}

/// Test #11 — POSA rejects non-integer X (D-23.7 stricter than AROT's
/// silent-trunc). ALPHA and X are both untouched on the error path.
#[test]
fn posa_rejects_non_integer_x() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    let x_orig = HpNum::from(Decimal::from_str("2.5").unwrap());
    state.stack.x = x_orig.clone();
    let result = dispatch(&mut state, Op::Posa);
    assert_eq!(result, Err(HpError::InvalidOp));
    assert_eq!(state.alpha_reg, "HELLO", "ALPHA unchanged on error");
    assert_eq!(state.stack.x, x_orig, "X unchanged on error");
}

/// Test #12 — POSA rejects out-of-range X (ASCII gate `0..=127`).
#[test]
fn posa_rejects_out_of_range_x() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    state.stack.x = HpNum::from(200); // beyond 127
    let result = dispatch(&mut state, Op::Posa);
    assert_eq!(result, Err(HpError::InvalidOp));
}

/// Test #13 — D-23.9 vs D-23.7 divergence: AROT silently truncates non-
/// integer X (faithful HP-41CV), while POSA rejects it (#11 above). This
/// test mechanically pins the AROT side of the divergence. Both behaviors
/// are intentional; removing either would be a regression.
#[test]
fn arot_silently_truncates_non_integer_x() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    // 2.7 silently truncates toward zero → 2 → AROT 2 → "LLOHE".
    state.stack.x = HpNum::from(Decimal::from_str("2.7").unwrap());
    dispatch(&mut state, Op::Arot).unwrap();
    assert_eq!(
        state.alpha_reg, "LLOHE",
        "AROT silently truncates non-integer X (D-23.9 — faithful HP-41CV)"
    );
}
