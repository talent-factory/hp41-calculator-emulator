//! Integration tests for ALPH-01: ALPHA register, 24-char limit, toggle, clear.

use hp41_core::CalcState;
use hp41_core::ops::{dispatch, Op};

// ── AlphaAppend ──────────────────────────────────────────────────────────

#[test]
fn test_alpha_append_builds_string() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::AlphaAppend('H')).unwrap();
    dispatch(&mut s, Op::AlphaAppend('P')).unwrap();
    dispatch(&mut s, Op::AlphaAppend('-')).unwrap();
    dispatch(&mut s, Op::AlphaAppend('4')).unwrap();
    dispatch(&mut s, Op::AlphaAppend('1')).unwrap();
    assert_eq!(s.alpha_reg, "HP-41");
}

#[test]
fn test_alpha_24_char_limit_enforced() {
    let mut s = CalcState::new();
    // Append 25 characters — only first 24 must be stored
    for c in "ABCDEFGHIJKLMNOPQRSTUVWXY".chars() {
        dispatch(&mut s, Op::AlphaAppend(c)).unwrap();
    }
    assert_eq!(s.alpha_reg.chars().count(), 24, "alpha_reg must stop at 24 chars");
    assert!(!s.alpha_reg.contains('Y'), "25th char 'Y' must be silently discarded");
    assert!(s.alpha_reg.ends_with('X'), "24th char must be 'X'");
}

// ── AlphaClear ───────────────────────────────────────────────────────────

#[test]
fn test_alpha_clear_empties_register() {
    let mut s = CalcState::new();
    s.alpha_reg = "TEST".to_string();
    dispatch(&mut s, Op::AlphaClear).unwrap();
    assert!(s.alpha_reg.is_empty(), "AlphaClear must empty alpha_reg");
}

// ── AlphaToggle ──────────────────────────────────────────────────────────

#[test]
fn test_alpha_toggle_flips_flag() {
    let mut s = CalcState::new();
    assert!(!s.alpha_mode, "alpha_mode must start false");
    dispatch(&mut s, Op::AlphaToggle).unwrap();
    assert!(s.alpha_mode, "AlphaToggle must enable alpha_mode");
    dispatch(&mut s, Op::AlphaToggle).unwrap();
    assert!(!s.alpha_mode, "second AlphaToggle must disable alpha_mode");
}

// ── Lift semantics (Neutral) ─────────────────────────────────────────────

#[test]
fn test_alpha_ops_are_neutral_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::AlphaToggle).unwrap();
    assert!(!s.stack.lift_enabled, "AlphaToggle must be Neutral lift");

    dispatch(&mut s, Op::AlphaAppend('A')).unwrap();
    assert!(!s.stack.lift_enabled, "AlphaAppend must be Neutral lift");

    dispatch(&mut s, Op::AlphaClear).unwrap();
    assert!(!s.stack.lift_enabled, "AlphaClear must be Neutral lift");
}

// ── Initial state ────────────────────────────────────────────────────────

#[test]
fn test_alpha_reg_starts_empty() {
    let s = CalcState::new();
    assert!(s.alpha_reg.is_empty(), "alpha_reg must be empty on startup");
    assert!(!s.alpha_mode, "alpha_mode must be false on startup");
}
