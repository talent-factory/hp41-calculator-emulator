//! Phase 23 (FN-ALPHA-01 + FN-ALPHA-02) integration tests for ARCL / ASTO.
//!
//! Covers the plan's documented success criteria and invariants:
//!   #1  ARCL respects current display mode (SC#1 — FIX/SCI/ENG)
//!   #2  ASTO + ARCL round-trip via the text_regs sidecar (SC#2)
//!   #3  Numeric STO clears the matching text_regs entry (D-23.4)
//!   #4  CLREG clears BOTH regs and text_regs (D-23.4)
//!   #5  ARCL with out-of-range reg returns InvalidOp without panic
//!   #5b ARCL rejects out-of-range reg even when text_regs has a stale entry
//!       (W-2 strengthening of D-23.3 — threat T-23-01 mitigation)
//!   #6  ASTO's 24-char ALPHA cap is honored on a subsequent ARCL
//!   #7  v1.x save files without `text_regs` load cleanly (#[serde(default)])
//!
//! Test modules allow unwrap (CLAUDE.md "Zero panics" applies to production
//! code only; tests carry the precedent #[allow(clippy::unwrap_used)]).

#![allow(clippy::unwrap_used)]

use hp41_core::format::format_hpnum;
use hp41_core::num::HpNum;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::{CalcState, DisplayMode};
use hp41_core::HpError;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Test #1 — SC#1 verifier: ARCL appends the formatted value of regs[5] using
/// the active display mode. Switching FIX→SCI between two ARCLs of the same
/// register produces a DIFFERENT appended suffix. Reuses `format_hpnum`
/// directly to derive the expected string at test time (no hardcoded mantissa).
#[test]
fn arcl_appends_numeric_register_using_current_display_mode() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(Decimal::from_str("3.14").unwrap());

    // FIX(2) — append format_hpnum(3.14, Fix(2)).
    state.alpha_reg = "HELLO".to_string();
    state.display_mode = DisplayMode::Fix(2);
    dispatch(&mut state, Op::Arcl(5)).unwrap();
    let expected_fix = format!(
        "HELLO{}",
        format_hpnum(&state.regs[5], &DisplayMode::Fix(2))
    );
    assert_eq!(state.alpha_reg, expected_fix);
    let fix_suffix = format_hpnum(&state.regs[5], &DisplayMode::Fix(2));

    // SCI(3) — same register, different appended suffix.
    state.alpha_reg = "HELLO".to_string();
    state.display_mode = DisplayMode::Sci(3);
    dispatch(&mut state, Op::Arcl(5)).unwrap();
    let expected_sci = format!(
        "HELLO{}",
        format_hpnum(&state.regs[5], &DisplayMode::Sci(3))
    );
    assert_eq!(state.alpha_reg, expected_sci);
    let sci_suffix = format_hpnum(&state.regs[5], &DisplayMode::Sci(3));

    assert_ne!(
        fix_suffix, sci_suffix,
        "SC#1: FIX(2) and SCI(3) must format the same value differently"
    );
}

/// Test #2 — SC#2 verifier: ASTO 12 with ALPHA="GOODBYE" stores "GOODBY" (first
/// 6 chars) into text_regs[12] and zeroes regs[12]. A subsequent ARCL 12 (after
/// clearing ALPHA) reproduces "GOODBY".
///
/// NOTE: real HP-41 RCL of a text-shadowed register copies the packed 56 bits
/// back to X; our HpNum = rust_decimal::Decimal model cannot preserve those raw
/// bits. D-23.5 documents the divergence: RCL of register 12 here pushes 0.
/// SC#2 is therefore interpreted as "ARCL round-trips the text" — NOT "RCL".
#[test]
fn asto_arcl_round_trip_reproduces_first_6_chars() {
    let mut state = CalcState::new();
    state.alpha_reg = "GOODBYE".to_string();
    dispatch(&mut state, Op::Asto(12)).unwrap();
    assert_eq!(state.text_regs.get(&12), Some(&"GOODBY".to_string()));
    assert_eq!(
        state.regs[12],
        HpNum::zero(),
        "no-drift invariant: ASTO zeroes the numeric slot"
    );

    // Clear ALPHA via Op::Cla. (Op::AlphaClear would work too — the plan's
    // SC#2 wording is implementation-agnostic on which clear op is used.)
    dispatch(&mut state, Op::Cla).unwrap();
    assert!(state.alpha_reg.is_empty());

    dispatch(&mut state, Op::Arcl(12)).unwrap();
    assert_eq!(state.alpha_reg, "GOODBY");
}

/// Test #3 — D-23.4 invariant: numeric STO clears the matching text_regs entry
/// so ARCL of that register thereafter reads the formatted numeric (NOT the
/// stale text shadow). Pins the no-drift property at the integration layer.
#[test]
fn numeric_sto_clears_text_regs_sidecar_no_drift() {
    let mut state = CalcState::new();

    // 1) ASTO 7 with ALPHA="HELLO" → text_regs[7]="HELLO", regs[7]=0.
    state.alpha_reg = "HELLO".to_string();
    dispatch(&mut state, Op::Asto(7)).unwrap();
    assert_eq!(state.text_regs.get(&7), Some(&"HELLO".to_string()));
    assert_eq!(state.regs[7], HpNum::zero());

    // 2) Put 3.14 in X and STO 7 → text_regs[7] must be CLEARED (D-23.4)
    //    AND regs[7] must hold the new numeric value.
    state.stack.x = HpNum::from(Decimal::from_str("3.14").unwrap());
    dispatch(&mut state, Op::StoReg(7)).unwrap();
    assert_eq!(
        state.text_regs.get(&7),
        None,
        "D-23.4: numeric STO must clear the text_regs sidecar"
    );
    assert_ne!(
        state.regs[7],
        HpNum::zero(),
        "numeric STO must have written 3.14 into regs[7]"
    );

    // 3) ARCL 7 must append the FORMATTED numeric (default Fix(4)) — not
    //    "HELLO".
    state.alpha_reg.clear();
    dispatch(&mut state, Op::Arcl(7)).unwrap();
    let expected = format_hpnum(&state.regs[7], &DisplayMode::Fix(4));
    assert_eq!(state.alpha_reg, expected);
    assert_ne!(state.alpha_reg, "HELLO", "ARCL must not return the stale shadow");
}

/// Test #4 — CLREG clears BOTH regs and text_regs. Without this, a CLREG would
/// leave ghost ARCL output behind after a fresh STO into the same slot.
#[test]
fn clreg_clears_both_regs_and_text_regs() {
    let mut state = CalcState::new();
    state.text_regs.insert(3, "FOO".to_string());
    state.text_regs.insert(8, "BAR".to_string());
    state.regs[3] = HpNum::from(Decimal::from_str("99.99").unwrap());
    state.regs[8] = HpNum::from(Decimal::from_str("12.5").unwrap());

    dispatch(&mut state, Op::Clreg).unwrap();
    assert!(
        state.text_regs.is_empty(),
        "Op::Clreg must empty the text_regs sidecar (D-23.4)"
    );
    for r in &state.regs {
        assert_eq!(r, &HpNum::zero(), "all numeric regs must be zero after CLREG");
    }
}

/// Test #5 — out-of-range reg returns InvalidOp without panic, leaves ALPHA
/// unchanged.
///
/// After the W-2 strengthening of op_arcl (a leading
/// `(reg as usize) >= state.regs.len()` bounds check), Op::Arcl(200) hits the
/// regs.len() guard FIRST — text_regs.get(&200) is never consulted. Pre-W-2
/// this test still passed via the numeric-fallback regs.get(200) but the
/// rejection path is now stricter and symmetric with op_asto.
#[test]
fn arcl_out_of_range_reg_returns_invalid_op_without_panic() {
    let mut state = CalcState::new();
    assert_eq!(state.regs.len(), 100, "default SIZE is 100");
    state.alpha_reg = "BEFORE".to_string();
    let result = dispatch(&mut state, Op::Arcl(200));
    assert_eq!(result, Err(HpError::InvalidOp));
    assert_eq!(
        state.alpha_reg, "BEFORE",
        "ALPHA must be unchanged on the error path"
    );
}

/// Test #5b — W-2 / W-3 demonstrator: a stale `text_regs[200]` entry (e.g. from
/// a hand-edited autosave.json — threat T-23-01) MUST NOT let op_arcl(200) bypass
/// the regs.len() bounds check. Without the W-2 leading bounds check, op_arcl
/// would clone "X" from the sidecar and weaponize the tampered save file.
#[test]
fn arcl_rejects_out_of_range_reg_even_when_text_regs_has_stale_entry() {
    let mut state = CalcState::new();
    assert_eq!(state.regs.len(), 100);
    state.alpha_reg = "BEFORE".to_string();
    // Tamper: insert a sidecar entry for an out-of-range reg index.
    state.text_regs.insert(200, "X".to_string());
    let result = dispatch(&mut state, Op::Arcl(200));
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "W-2: leading regs.len() bounds check must fire BEFORE text_regs lookup"
    );
    assert_eq!(
        state.alpha_reg, "BEFORE",
        "the bogus 'X' must NOT have been appended (T-23-01 mitigation)"
    );
}

/// Test #6 — silent 24-char cap on a subsequent ARCL. With ALPHA at 23 chars,
/// an ARCL of a 5-char shadow appends only the first char before hitting the
/// 24-char ceiling.
#[test]
fn asto_silent_24_char_cap_on_subsequent_arcl() {
    let mut state = CalcState::new();
    state.alpha_reg = "A".repeat(23); // 23 'A's
    state.text_regs.insert(0, "BCDEF".to_string());

    dispatch(&mut state, Op::Arcl(0)).unwrap();
    assert_eq!(state.alpha_reg.chars().count(), 24, "ALPHA caps at 24 chars");
    assert!(
        state.alpha_reg.ends_with('B'),
        "only the first char of 'BCDEF' fits before the cap; rest silently discarded"
    );
}

/// Test #7 — D-23.13 save-file compat: a JSON payload with the v2.1 schema
/// (no `text_regs` field) deserializes to a CalcState whose `text_regs` is the
/// `BTreeMap::new()` default. Mirrors the Phase 12 / Phase 22 `#[serde(default)]`
/// backward-compat pattern.
#[test]
fn serde_default_loads_v21_save_file_without_text_regs_field() {
    // Round-trip approach (test stays self-contained): serialize a default
    // CalcState, strip the `text_regs` key from the JSON, then deserialize.
    // This proves the deserializer accepts a payload lacking the field.
    let state = CalcState::new();
    let serialized = serde_json::to_string(&state).expect("CalcState serializes");
    let mut json: serde_json::Value =
        serde_json::from_str(&serialized).expect("valid JSON round-trip");

    // Remove text_regs from the JSON object — simulates v2.0/v2.1 save files
    // that were written before this field existed.
    let obj = json
        .as_object_mut()
        .expect("CalcState serializes as JSON object");
    let removed = obj.remove("text_regs");
    assert!(
        removed.is_some(),
        "sanity: current CalcState DOES serialize text_regs (we just stripped it)"
    );

    // Deserialize without the field. #[serde(default)] must default it to
    // BTreeMap::new().
    let restored: CalcState =
        serde_json::from_value(json).expect("missing text_regs must deserialize via #[serde(default)]");
    assert!(
        restored.text_regs.is_empty(),
        "text_regs must default to an empty map when missing from the payload"
    );
}

/// Test #8 — WR-01 (D-23.4 leak fix): `op_size` must drop `text_regs` entries
/// whose key now points past the new end-of-regs, so a shrink-then-grow SIZE
/// cycle cannot resurrect a stale text shadow.
///
/// Sequence mirrors the WR-01 finding from 23-REVIEW.md:
///   1. Start at SIZE 100. ASTO 60 with ALPHA="GHOST" → text_regs[60]="GHOST",
///      regs[60]=0.
///   2. SIZE 50 → regs truncated. text_regs[60] MUST be pruned (was the bug).
///   3. SIZE 100 → regs grown, regs[60]=0.
///   4. ARCL 60 must read the numeric fallback (0 in current display mode),
///      NOT the resurrected "GHOST" shadow.
#[test]
fn size_shrink_then_grow_drops_text_regs_no_ghost_resurrection() {
    let mut state = CalcState::new();
    assert_eq!(state.regs.len(), 100, "default SIZE is 100");

    // Step 1: stash "GHOST" into the sidecar at reg 60.
    state.alpha_reg = "GHOST".to_string();
    dispatch(&mut state, Op::Asto(60)).unwrap();
    assert_eq!(state.text_regs.get(&60), Some(&"GHOST".to_string()));
    assert_eq!(state.regs[60], HpNum::zero());

    // Step 2: SIZE 50 must prune text_regs[60] (WR-01 fix).
    dispatch(&mut state, Op::Size(50)).unwrap();
    assert_eq!(state.regs.len(), 50);
    assert_eq!(
        state.text_regs.get(&60),
        None,
        "WR-01: op_size must drop text_regs entries past end-of-regs"
    );

    // Step 3: regrow to 100 — regs[60] is fresh HpNum::zero().
    dispatch(&mut state, Op::Size(100)).unwrap();
    assert_eq!(state.regs.len(), 100);
    assert_eq!(state.regs[60], HpNum::zero());

    // Step 4: ARCL 60 must format the numeric fallback, NOT "GHOST".
    state.alpha_reg.clear();
    dispatch(&mut state, Op::Arcl(60)).unwrap();
    let expected = format_hpnum(&state.regs[60], &state.display_mode);
    assert_eq!(state.alpha_reg, expected);
    assert!(
        !state.alpha_reg.contains("GHOST"),
        "shrink-then-grow must not resurrect the stale text shadow"
    );
}
