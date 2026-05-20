// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration tests for Math Pac I POLY / ROOTS operations (Plan 28-05).
//!
//! These tests exercise the full dispatch chain:
//! dispatch() → op_poly_workflow / op_roots → modal_program / print_buffer
//!
//! Complement the unit tests in hp41-core/src/ops/math1/poly.rs::tests.
//! Required by math1_op_test_count.rs Pitfall 16 gate (≥ 5 mentions per variant
//! in math1_*.rs test files).
//!
//! Source: HP-41C Math Pac Owner's Manual (HP 00041-90034, 1979), Chapter 7.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::math1::modal::{ModalProgram, PolyInputStep};
use hp41_core::ops::{dispatch, Op};
use hp41_core::CalcState;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

fn make_state() -> CalcState {
    CalcState::new()
}

fn set_reg(state: &mut CalcState, idx: usize, val: f64) {
    let d = Decimal::from_f64(val).unwrap_or(Decimal::ZERO);
    state.regs[idx] = hp41_core::HpNum::rounded(d);
}

// ── Op::PolyWorkflow — integration tests via dispatch() ──────────────────────

/// Catches: Op::PolyWorkflow variant missing from dispatch() match block.
/// Verifies the full dispatch chain: dispatch(PolyWorkflow) → op_poly_workflow.
/// Source: HP 00041-90034 (1979), Chapter 7 "Polynomial Solutions".
#[test]
fn dispatch_poly_workflow_succeeds() {
    let mut s = make_state();
    let result = dispatch(&mut s, Op::PolyWorkflow);
    assert!(
        result.is_ok(),
        "dispatch(Op::PolyWorkflow) must return Ok(())"
    );
}

/// Catches: Op::PolyWorkflow not setting modal_program to DegreePrompt.
/// Source: HP 00041-90034 (1979), Chapter 7 — first prompt is DEGREE=?
#[test]
fn dispatch_poly_workflow_sets_modal_program() {
    let mut s = make_state();
    dispatch(&mut s, Op::PolyWorkflow).unwrap();
    assert!(
        matches!(
            s.modal_program,
            Some(ModalProgram::Poly(PolyInputStep::DegreePrompt))
        ),
        "Op::PolyWorkflow must set modal_program to Poly(DegreePrompt)"
    );
}

/// Catches: Op::PolyWorkflow not setting modal_prompt to "DEGREE=?".
/// Source: HP 00041-90034 (1979), Chapter 7, D-28.4 modal_prompt channel.
#[test]
fn dispatch_poly_workflow_sets_modal_prompt_text() {
    let mut s = make_state();
    dispatch(&mut s, Op::PolyWorkflow).unwrap();
    assert_eq!(
        s.modal_prompt.as_deref(),
        Some("DEGREE=?"),
        "Op::PolyWorkflow must set modal_prompt to 'DEGREE=?'"
    );
}

/// Catches: Op::PolyWorkflow modifying the stack (LiftEffect must be Neutral).
/// Source: HP 00041-90034 (1979), Chapter 7 — POLY opener does not consume stack.
#[test]
fn dispatch_poly_workflow_does_not_modify_stack() {
    let mut s = make_state();
    let x_before = Decimal::from(42i32);
    s.stack.x = hp41_core::HpNum::rounded(x_before);
    dispatch(&mut s, Op::PolyWorkflow).unwrap();
    // LINT-EXEMPT: integer sentinel — x_before is Decimal::from(42i32) which is exact
    // (no f64 bridge, no FPU rounding); comparing .inner() to an integer Decimal
    // is cross-platform-safe per Pitfall 17.
    assert_eq!(
        s.stack.x.inner(),
        x_before,
        "Op::PolyWorkflow must not modify stack X (LiftEffect::Neutral)"
    );
}

/// Catches: Op::PolyWorkflow not idempotent on re-open.
/// Source: HP 00041-90034 (1979), Chapter 7 — XEQ "POLY" twice resets the workflow.
#[test]
fn dispatch_poly_workflow_is_idempotent_re_open() {
    let mut s = make_state();
    dispatch(&mut s, Op::PolyWorkflow).unwrap();
    // Simulate some partial state
    s.modal_program = Some(ModalProgram::Poly(PolyInputStep::CoefficientPrompt(3, 1)));
    // Re-open must reset to DegreePrompt
    dispatch(&mut s, Op::PolyWorkflow).unwrap();
    assert_eq!(
        s.modal_prompt.as_deref(),
        Some("DEGREE=?"),
        "Op::PolyWorkflow re-open must reset modal_prompt to 'DEGREE=?'"
    );
}

/// Catches: Op::PolyWorkflow dispatch going to wrong op (regression guard).
/// Verifies that PolyWorkflow does NOT dispatch to op_roots by checking that
/// print_buffer is empty (op_roots writes to print_buffer; op_poly_workflow does not).
/// Source: HP 00041-90034 (1979), Chapter 7.
#[test]
fn dispatch_poly_workflow_does_not_write_to_print_buffer() {
    let mut s = make_state();
    dispatch(&mut s, Op::PolyWorkflow).unwrap();
    assert!(
        s.print_buffer.is_empty(),
        "Op::PolyWorkflow must not write to print_buffer (only op_roots writes output)"
    );
}

// ── Op::Roots — integration tests via dispatch() ─────────────────────────────

/// Catches: Op::Roots variant missing from dispatch() match block.
/// Uses x² - 1 = 0 (roots ±1) as the simplest possible test case.
/// Source: HP 00041-90034 (1979), Chapter 7.
#[test]
fn dispatch_roots_succeeds_for_simple_quadratic() {
    let mut s = make_state();
    set_reg(&mut s, 0, 1.0); // A=1 (x² term)
    set_reg(&mut s, 1, 0.0); // B=0
    set_reg(&mut s, 2, -1.0); // C=-1 (constant)
    let result = dispatch(&mut s, Op::Roots);
    assert!(
        result.is_ok(),
        "dispatch(Op::Roots) must return Ok(()) for x²-1=0"
    );
}

/// Catches: Op::Roots not writing any output to print_buffer.
/// Source: HP 00041-90034 (1979), Chapter 7 — ROOTS writes U= / V= lines.
#[test]
fn dispatch_roots_writes_to_print_buffer() {
    let mut s = make_state();
    s.display_mode = hp41_core::DisplayMode::Fix(4);
    set_reg(&mut s, 0, 1.0); // A=1
    set_reg(&mut s, 1, 0.0); // B=0
    set_reg(&mut s, 2, -1.0); // C=-1
    dispatch(&mut s, Op::Roots).unwrap();
    assert!(
        !s.print_buffer.is_empty(),
        "Op::Roots must write root values to print_buffer"
    );
}

/// Catches: Op::Roots output format regression (U= prefix missing).
/// Source: HP 00041-90034 (1979), Chapter 7 POLY-04 output format.
#[test]
fn dispatch_roots_output_has_u_prefix() {
    let mut s = make_state();
    s.display_mode = hp41_core::DisplayMode::Fix(4);
    set_reg(&mut s, 0, 1.0); // A=1
    set_reg(&mut s, 1, 0.0); // B=0
    set_reg(&mut s, 2, -1.0); // C=-1
    dispatch(&mut s, Op::Roots).unwrap();
    let has_u_line = s.print_buffer.iter().any(|l| l.starts_with("U="));
    assert!(
        has_u_line,
        "Op::Roots print_buffer must contain at least one 'U=' line"
    );
}

/// Catches: Op::Roots not clearing modal_program on success.
/// Source: D-28.4 — modal state must be cleared when computation completes.
#[test]
fn dispatch_roots_clears_modal_state() {
    let mut s = make_state();
    s.modal_program = Some(ModalProgram::Poly(PolyInputStep::Ready));
    s.modal_prompt = Some("A=?".to_string());
    set_reg(&mut s, 0, 1.0);
    set_reg(&mut s, 1, 0.0);
    set_reg(&mut s, 2, -1.0);
    dispatch(&mut s, Op::Roots).unwrap();
    assert!(
        s.modal_program.is_none(),
        "Op::Roots must set modal_program to None on success"
    );
    assert!(
        s.modal_prompt.is_none(),
        "Op::Roots must set modal_prompt to None on success"
    );
}

/// Catches: Op::Roots modifying stack X (LiftEffect must be Neutral).
/// Source: HP 00041-90034 (1979), Chapter 7 — ROOTS writes to print_buffer, not stack.
#[test]
fn dispatch_roots_lift_effect_neutral() {
    let mut s = make_state();
    let sentinel = Decimal::from(77i32);
    s.stack.x = hp41_core::HpNum::rounded(sentinel);
    s.display_mode = hp41_core::DisplayMode::Fix(4);
    set_reg(&mut s, 0, 1.0);
    set_reg(&mut s, 1, 0.0);
    set_reg(&mut s, 2, -1.0);
    dispatch(&mut s, Op::Roots).unwrap();
    // LINT-EXEMPT: integer sentinel — sentinel is Decimal::from(77i32) which is exact
    // (no f64 bridge, no FPU rounding); comparing .inner() to an integer Decimal
    // is cross-platform-safe per Pitfall 17.
    assert_eq!(
        s.stack.x.inner(),
        sentinel,
        "Op::Roots must not modify stack X (LiftEffect::Neutral)"
    );
}

/// Catches: Op::Roots giving wrong root count for degree-2 real-roots polynomial.
/// x² - 3x + 2 = (x-1)(x-2) → 2 real roots → 2 U= lines.
/// Source: HP 00041-90034 (1979), Chapter 7 worked quadratic example.
/// Free42 v3.0.5: roots 1.0 and 2.0 confirmed.
#[test]
fn dispatch_roots_quadratic_real_root_count() {
    let mut s = make_state();
    s.display_mode = hp41_core::DisplayMode::Fix(4);
    set_reg(&mut s, 0, 1.0); // A=1 (x² term)
    set_reg(&mut s, 1, -3.0); // B=-3 (x term)
    set_reg(&mut s, 2, 2.0); // C=2 (constant)
    dispatch(&mut s, Op::Roots).unwrap();
    let u_count = s
        .print_buffer
        .iter()
        .filter(|l| l.starts_with("U="))
        .count();
    assert_eq!(
        u_count, 2,
        "x²-3x+2 has 2 real roots → exactly 2 U= lines in print_buffer"
    );
}

/// Catches: Op::Roots complex-pair output not following Pitfall 5 four-line format.
/// x² + 1 = 0 → complex roots ±i → EXACTLY 4 lines: U=.., V=.., U=.., -V=-..
/// Source: HP 00041-90034 (1979), Chapter 7 POLY-04 format gate (Pitfall 5).
/// Free42 v3.0.5: confirms 4-line output for complex-conjugate pairs.
#[test]
fn dispatch_roots_complex_pair_four_line_format() {
    let mut s = make_state();
    s.display_mode = hp41_core::DisplayMode::Fix(4);
    set_reg(&mut s, 0, 1.0); // A=1 (x² term)
    set_reg(&mut s, 1, 0.0); // B=0
    set_reg(&mut s, 2, 1.0); // C=1 (constant)
    dispatch(&mut s, Op::Roots).unwrap();
    assert_eq!(
        s.print_buffer.len(),
        4,
        "x²+1=0 has complex pair → exactly 4 print_buffer lines (POLY-04 / Pitfall 5)"
    );
    assert!(s.print_buffer[0].starts_with("U="), "Line 0: U=<u>");
    assert!(s.print_buffer[1].starts_with("V="), "Line 1: V=<v>");
    assert!(
        s.print_buffer[2].starts_with("U="),
        "Line 2: U=<u> (repeated)"
    );
    assert!(s.print_buffer[3].starts_with("-V=-"), "Line 3: -V=-<v>");
}
