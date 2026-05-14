//! Integration tests for Phase 21 Plan 03 (Display Control: VIEW/AVIEW/PROMPT/AON/AOFF/CLD).
//!
//! Covers FN-DISP-01..05 plus the dispatch-top clear (Pitfall 5), the v2.0
//! backward-compat load (SC-5 spillover from Plan 21-01), and the
//! PROMPT-exits-run_loop semantic (Pitfall 3).

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, flags::flag_get, program::run_program, Op};
use hp41_core::{CalcState, DisplayMode, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_display_override_field_defaults_to_none() {
    let s = CalcState::new();
    assert!(s.display_override.is_none());
}

#[test]
fn test_load_v20_save_no_display_override_field() {
    let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap();
    let s: CalcState = serde_json::from_str(&json).unwrap();
    assert!(s.display_override.is_none());
}

#[test]
fn test_display_override_skipped_on_serialize() {
    let mut s = CalcState::new();
    s.display_override = Some("TEST".to_string());
    let json = serde_json::to_string(&s).unwrap();
    assert!(
        !json.contains("display_override"),
        "display_override must be #[serde(skip)] — JSON: {json}"
    );
}

#[test]
fn test_view_writes_register_to_override() {
    let mut s = CalcState::new();
    s.regs[3] = HpNum::from(42i32);
    s.display_mode = DisplayMode::Fix(4);
    dispatch(&mut s, Op::View(3)).unwrap();
    let out = s.display_override.as_deref().unwrap();
    // format_hpnum on Fix(4) for 42 → "42.0000"
    assert_eq!(out, "42.0000", "actual: {out}");
}

#[test]
fn test_view_preserves_stack() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1i32);
    s.stack.y = HpNum::from(2i32);
    s.stack.z = HpNum::from(3i32);
    s.stack.t = HpNum::from(4i32);
    s.stack.lastx = HpNum::from(5i32);
    s.regs[7] = HpNum::from(99i32);
    dispatch(&mut s, Op::View(7)).unwrap();
    assert_eq!(s.stack.x, HpNum::from(1i32));
    assert_eq!(s.stack.y, HpNum::from(2i32));
    assert_eq!(s.stack.z, HpNum::from(3i32));
    assert_eq!(s.stack.t, HpNum::from(4i32));
    assert_eq!(s.stack.lastx, HpNum::from(5i32));
}

#[test]
fn test_view_out_of_range() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::View(100));
    assert!(matches!(r, Err(HpError::InvalidOp)));
    assert!(s.display_override.is_none());
}

#[test]
fn test_aview_writes_alpha_to_override() {
    let mut s = CalcState::new();
    s.alpha_reg = "HELLO".to_string();
    dispatch(&mut s, Op::AView).unwrap();
    assert_eq!(s.display_override.as_deref(), Some("HELLO"));
}

#[test]
fn test_aon_sets_flag_48() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Aon).unwrap();
    assert!(flag_get(s.flags, 48));
}

#[test]
fn test_aoff_clears_flag_48() {
    let mut s = CalcState::new();
    s.flags = 1u64 << 48;
    dispatch(&mut s, Op::Aoff).unwrap();
    assert!(!flag_get(s.flags, 48));
}

#[test]
fn test_cld_clears_only_override() {
    let mut s = CalcState::new();
    s.display_override = Some("X".to_string());
    s.alpha_reg = "Y".to_string();
    s.stack.x = HpNum::from(42i32);
    dispatch(&mut s, Op::Cld).unwrap();
    assert!(s.display_override.is_none());
    assert_eq!(s.alpha_reg, "Y");
    assert_eq!(s.stack.x.inner(), Decimal::from(42));
}

#[test]
fn test_dispatch_top_clears_stale_override() {
    // Pre-set a stale override; dispatching any non-display-writing op must clear it
    // via the dispatch-top reset.
    let mut s = CalcState::new();
    s.display_override = Some("STALE".to_string());
    dispatch(&mut s, Op::Enter).unwrap();
    assert!(
        s.display_override.is_none(),
        "dispatch-top must clear stale display_override"
    );
}

#[test]
fn test_prompt_exits_run_loop() {
    // Program: load ALPHA with "HI", PROMPT (break expected), PushNum(99) must NOT execute.
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::AlphaAppend('H'),
        Op::AlphaAppend('I'),
        Op::Prompt,
        Op::PushNum(HpNum::from(Decimal::from_str("99").unwrap())),
    ];
    let initial_x = s.stack.x.clone();
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.display_override.as_deref(), Some("HI"));
    // The PushNum(99) is unreachable — X must still be the original value (zero).
    assert_eq!(s.stack.x, initial_x, "PROMPT must break run_loop before PushNum(99)");
}

#[test]
fn test_prompt_inside_program_returns_quickly() {
    // Pitfall 3 sentinel: PROMPT must not busy-wait. Even on a slow runner,
    // a single-step run_program should return in well under 100 ms.
    let mut s = CalcState::new();
    s.program = vec![Op::Lbl("T".to_string()), Op::Prompt];
    let start = std::time::Instant::now();
    run_program(&mut s, "T").unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "PROMPT busy-wait sentinel: elapsed = {elapsed:?}"
    );
}
