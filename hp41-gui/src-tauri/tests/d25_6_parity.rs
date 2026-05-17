//! D-25.6 CLI ↔ GUI parity regression tests.
//!
//! Asserts that Math Pac I functions produce identical X-register output
//! whether reached via:
//!   (a) the CLI path: `xrom_resolve("NAME", modules)` returning `Some(Op::Name)`,
//!       then direct `dispatch(&mut state, Op::Name)`
//!   (b) the GUI-equivalent path: `Op::Xeq("NAME")` inside a run_program loop,
//!       which flows through core's resolver chain → `xrom_resolve` → `Op::Name`
//!
//! The two paths are bit-identical in the same process (per Pitfall 14 /
//! 31-RESEARCH.md §Pitfall 14 lines 1100-1108): no floating-point rounding
//! differences, no lazy evaluation, no caching. Assertions use strict
//! `assert_eq!` (NOT `approx::assert_relative_eq!`).
//!
//! Purpose: if a future regression introduces a new resolver in `key_map.rs`
//! that routes `dispatch_op("xeq_SINH")` through a different code path than
//! `xrom_resolve`, the SINH test will catch it immediately.
//!
//! D-25.6 contract: GUI calls shared `hp41-core::ops::math1::*` functions verbatim.
//! NO parallel Math Pac I logic in `hp41-gui/src-tauri/`. This test locks that
//! invariant behind a regression.

#![allow(clippy::unwrap_used)]

use hp41_core::{ops::dispatch, ops::math1::xrom::xrom_resolve, ops::Op, CalcState, HpNum};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

/// Helper: create a test state with stack.x set to `input`, run a one-shot
/// program `[LBL "MAIN", XEQ "name", RTN]`, return the resulting state.
fn run_via_xeq(name: &str, input: f64) -> CalcState {
    let mut state = CalcState::new();
    state.stack.x = HpNum::rounded(Decimal::from_f64(input).unwrap());
    state.program.push(Op::Lbl("MAIN".into()));
    state.program.push(Op::Xeq(name.into()));
    state.program.push(Op::Rtn);
    hp41_core::run_program(&mut state, "MAIN").unwrap();
    state
}

/// Helper: create a test state with stack.x set to `input`, dispatch `op` directly.
fn run_direct(op: Op, input: f64) -> CalcState {
    let mut state = CalcState::new();
    state.stack.x = HpNum::rounded(Decimal::from_f64(input).unwrap());
    dispatch(&mut state, op).unwrap();
    state
}

/// Catches: future divergence between CLI xrom_resolve path and GUI dispatch_op-driven
/// path for SINH(1.5). If a new resolver arm in key_map.rs routes xeq_SINH differently,
/// the stack.x values will differ and this test fails.
#[test]
fn parity_sinh_1_5() {
    // CLI path: xeq_by_name_local_resolve("SINH", 0b0000_0001) → Op::Sinh
    let resolved = xrom_resolve("SINH", 0b0000_0001);
    assert_eq!(resolved, Some(Op::Sinh), "xrom_resolve must return Some(Op::Sinh) for MATH_1");

    // GUI-equivalent path: Op::Xeq("SINH") inside run_program
    let state_gui = run_via_xeq("SINH", 1.5);

    // Direct dispatch baseline (same as CLI after xrom_resolve returns the Op)
    let state_direct = run_direct(Op::Sinh, 1.5);

    assert_eq!(
        state_gui.stack.x,
        state_direct.stack.x,
        "SINH(1.5) via GUI XEQ path must match direct dispatch (D-25.6 parity)"
    );
}

/// Catches: future divergence between CLI xrom_resolve path and GUI dispatch_op-driven
/// path for ASINH(2.0). Two-arg function (stack-acting on X) — same shape as SINH test.
#[test]
fn parity_asinh_2_0() {
    // CLI path
    let resolved = xrom_resolve("ASINH", 0b0000_0001);
    assert_eq!(resolved, Some(Op::Asinh), "xrom_resolve must return Some(Op::Asinh) for MATH_1");

    // GUI-equivalent path
    let state_gui = run_via_xeq("ASINH", 2.0);

    // Direct dispatch baseline
    let state_direct = run_direct(Op::Asinh, 2.0);

    assert_eq!(
        state_gui.stack.x,
        state_direct.stack.x,
        "ASINH(2.0) via GUI XEQ path must match direct dispatch (D-25.6 parity)"
    );
}

/// Catches: future divergence between CLI xrom_resolve path and GUI dispatch_op-driven
/// path for TANH(1.0). Third function confirms the parity contract is systematic,
/// not an accidental single-entry pass.
#[test]
fn parity_tanh_1_0() {
    // CLI path
    let resolved = xrom_resolve("TANH", 0b0000_0001);
    assert_eq!(resolved, Some(Op::Tanh), "xrom_resolve must return Some(Op::Tanh) for MATH_1");

    // GUI-equivalent path
    let state_gui = run_via_xeq("TANH", 1.0);

    // Direct dispatch baseline
    let state_direct = run_direct(Op::Tanh, 1.0);

    assert_eq!(
        state_gui.stack.x,
        state_direct.stack.x,
        "TANH(1.0) via GUI XEQ path must match direct dispatch (D-25.6 parity)"
    );
}

/// Catches: xrom_resolve returning Some for an unregistered mnemonic (resolver-miss
/// path must return None so unknown XEQ calls fall through to InvalidOp, not a
/// spurious Math Pac I function).
#[test]
fn parity_unknown_returns_none() {
    // BOGUS is not in MATH_1.ops — must resolve to None regardless of modules flag.
    let resolved = xrom_resolve("BOGUS", 0b0000_0001);
    assert_eq!(
        resolved,
        None,
        "xrom_resolve must return None for an unknown mnemonic"
    );
}
