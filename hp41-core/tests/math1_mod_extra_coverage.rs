// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Plan 32-07 gap-closure: additional coverage for `hp41-core/src/ops/math1/mod.rs`.
//!
//! Plan 32-01 (`math1_mod_entry_points.rs`) covered the 5 error-path contracts
//! (WR-01 flush-only-after-guard, WR-02 alpha-reg-only-after-guard, cancel_modal
//! field-clear, no-modal-active short-circuit). The remaining ~14 missed lines are
//! the per-program routing arms in `submit_modal` for Solve/Poly/Difeq/Four/Trans
//! and the `submit_modal_with_label` arms for Solve and Difeq.
//!
//! Each test exercises the routing arm in `mod.rs`; per-program logic depth is
//! intentionally minimal — the goal is to confirm the dispatch fires (state
//! advanced, or Ok returned), NOT to re-verify per-program solver accuracy.

#![allow(clippy::unwrap_used)]

use hp41_core::num::HpNum;
use hp41_core::ops::math1::modal::{
    DifeqInputStep, FourInputStep, ModalProgram, PolyInputStep, SolveInputStep, TransInputStep,
};
use hp41_core::ops::math1::{submit_modal, submit_modal_with_label};
use hp41_core::state::CalcState;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

// ── Helper: set X register to a specific f64 value ───────────────────────────

fn set_x(state: &mut CalcState, v: f64) {
    state.stack.x = HpNum::from(Decimal::from_f64(v).unwrap_or(Decimal::ZERO));
}

// ── submit_modal: Solve routing arm ──────────────────────────────────────────

// Catches: submit_modal Solve routing arm missing — Solve modal never advances
// when submit_modal is called from the CLI/GUI R/S handler.
#[test]
fn submit_modal_dispatches_solve() {
    let mut state = CalcState::new();
    // Set modal at Guess1Prompt (a numeric-input step)
    state.modal_program = Some(ModalProgram::Solve(SolveInputStep::Guess1Prompt));
    state.modal_prompt = Some("GUESS 1=?".to_string());
    set_x(&mut state, 0.5);
    state.stack.lift_enabled = false;

    let result = submit_modal(&mut state);
    // submit_modal should route to solve::submit_step and return Ok or valid Err;
    // the goal is that it does NOT return InvalidOp from the routing layer itself.
    // (solve::submit_step on Guess1Prompt advances to Guess2Prompt)
    assert!(
        result.is_ok(),
        "submit_modal must route to solve::submit_step for Guess1Prompt, got {result:?}"
    );
    // Verify the modal advanced to Guess2Prompt (routing fired, not just returned Ok)
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Solve(SolveInputStep::Guess2Prompt))
        ),
        "submit_modal for Solve/Guess1Prompt must advance to Guess2Prompt"
    );
}

// ── submit_modal: Poly routing arm ───────────────────────────────────────────

// Catches: submit_modal Poly routing arm missing — Poly modal never advances
// when submit_modal is called.
#[test]
fn submit_modal_dispatches_poly() {
    let mut state = CalcState::new();
    // DegreePrompt is the first Poly step (numeric input)
    state.modal_program = Some(ModalProgram::Poly(PolyInputStep::DegreePrompt));
    state.modal_prompt = Some("DEGREE=?".to_string());
    // Poly requires degree 2..=5; set X=3
    set_x(&mut state, 3.0);
    state.stack.lift_enabled = false;

    let result = submit_modal(&mut state);
    assert!(
        result.is_ok(),
        "submit_modal must route to poly::submit_step for DegreePrompt, got {result:?}"
    );
    // After DegreePrompt with degree=3, modal advances to CoefficientPrompt(3, 0)
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Poly(PolyInputStep::CoefficientPrompt(3, 0)))
        ),
        "submit_modal for Poly/DegreePrompt must advance to CoefficientPrompt(3, 0)"
    );
}

// ── submit_modal: Difeq routing arm ──────────────────────────────────────────

// Catches: submit_modal Difeq routing arm missing — Difeq modal never advances
// when submit_modal is called.
#[test]
fn submit_modal_dispatches_difeq() {
    let mut state = CalcState::new();
    // OrderPrompt is a numeric step (ORDER=? → 1 or 2)
    state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::OrderPrompt));
    state.modal_prompt = Some("ORDER=?".to_string());
    set_x(&mut state, 1.0);
    state.stack.lift_enabled = false;

    let result = submit_modal(&mut state);
    assert!(
        result.is_ok(),
        "submit_modal must route to difeq::submit_step for OrderPrompt, got {result:?}"
    );
    // After OrderPrompt with order=1, modal advances to StepSizePrompt
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Difeq(DifeqInputStep::StepSizePrompt))
        ),
        "submit_modal for Difeq/OrderPrompt must advance to StepSizePrompt"
    );
}

// ── submit_modal: Four routing arm ───────────────────────────────────────────

// Catches: submit_modal Four routing arm missing — Four modal never advances
// when submit_modal is called.
#[test]
fn submit_modal_dispatches_four() {
    let mut state = CalcState::new();
    // NumSamplesPrompt is the first FOUR step
    state.modal_program = Some(ModalProgram::Four(FourInputStep::NumSamplesPrompt));
    state.modal_prompt = Some("NO. SAMPLES=?".to_string());
    // FOUR requires 2..=some_max samples; use 4
    set_x(&mut state, 4.0);
    state.stack.lift_enabled = false;

    let result = submit_modal(&mut state);
    assert!(
        result.is_ok(),
        "submit_modal must route to four::submit_step for NumSamplesPrompt, got {result:?}"
    );
    // After NumSamplesPrompt the modal should have advanced (not still at NumSamplesPrompt)
    assert!(
        !matches!(
            state.modal_program,
            Some(ModalProgram::Four(FourInputStep::NumSamplesPrompt))
        ),
        "submit_modal for Four/NumSamplesPrompt must advance the modal state"
    );
}

// ── submit_modal: Trans routing arm ──────────────────────────────────────────

// Catches: submit_modal Trans routing arm missing — Trans modal never advances
// when submit_modal is called.
#[test]
fn submit_modal_dispatches_trans() {
    let mut state = CalcState::new();
    // Init2dPrompt is the first Trans step
    state.modal_program = Some(ModalProgram::Trans(TransInputStep::Init2dPrompt));
    state.modal_prompt = Some("X0,Y0,θ?".to_string());
    // Any numeric value is fine for the init step
    set_x(&mut state, 0.0);
    state.stack.lift_enabled = false;

    // submit_step for Trans/Init2dPrompt may succeed or return a specific error;
    // the contract we are testing is that mod.rs routes to trans::submit_step
    // rather than returning InvalidOp from the routing layer itself.
    let result = submit_modal(&mut state);
    // Result should be Ok or a domain-specific error — NOT the routing-layer InvalidOp
    // (which would only fire if modal_program were None or not a recognised variant)
    let _ = result; // outcome depends on trans internal state; routing is the target

    // The modal_program should no longer be Some(Trans(Init2dPrompt)) if routing fired
    // (either it advanced or was cleared on a domain error)
    assert!(
        !matches!(
            state.modal_program,
            Some(ModalProgram::Trans(TransInputStep::Init2dPrompt))
        ),
        "submit_modal for Trans/Init2dPrompt must route to trans::submit_step (not stay at Init2dPrompt)"
    );
}

// ── submit_modal_with_label: Solve FunctionNamePrompt arm ────────────────────

// Catches: submit_modal_with_label Solve routing arm missing — SOLVE
// never advances past FunctionNamePrompt when the CLI/GUI provides an
// alpha label via Enter-in-XeqByName mode (D-29.7).
#[test]
fn submit_modal_with_label_solve_advances_to_guess1() {
    let mut state = CalcState::new();
    state.alpha_reg = "OLD".to_string();
    state.modal_program = Some(ModalProgram::Solve(SolveInputStep::FunctionNamePrompt));
    state.modal_prompt = Some("FUNCTION NAME?".to_string());

    let result = submit_modal_with_label(&mut state, "  mysolvefn  ");

    assert!(
        result.is_ok(),
        "submit_modal_with_label for Solve/FunctionNamePrompt must return Ok, got {result:?}"
    );
    // Label was uppercased and trimmed
    assert_eq!(
        state.alpha_reg, "MYSOLVEFN",
        "submit_modal_with_label must uppercase + trim the label for Solve"
    );
    // Modal advanced to Guess1Prompt
    assert!(
        matches!(
            state.modal_program,
            Some(ModalProgram::Solve(SolveInputStep::Guess1Prompt))
        ),
        "submit_modal_with_label for Solve/FunctionNamePrompt must advance to Guess1Prompt"
    );
}

// ── submit_modal_with_label: Difeq FunctionNamePrompt arm ────────────────────

// Catches: submit_modal_with_label Difeq routing arm missing — DIFEQ
// never advances past FunctionNamePrompt when the CLI/GUI provides an alpha label.
#[test]
fn submit_modal_with_label_difeq_advances_past_function_name() {
    let mut state = CalcState::new();
    state.alpha_reg = "OLD".to_string();
    state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt));
    state.modal_prompt = Some("FUNCTION NAME?".to_string());

    let result = submit_modal_with_label(&mut state, "difn");

    assert!(
        result.is_ok(),
        "submit_modal_with_label for Difeq/FunctionNamePrompt must return Ok, got {result:?}"
    );
    assert_eq!(
        state.alpha_reg, "DIFN",
        "submit_modal_with_label must uppercase + trim label for Difeq"
    );
    // Modal should have advanced past FunctionNamePrompt
    assert!(
        !matches!(
            state.modal_program,
            Some(ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt))
        ),
        "submit_modal_with_label for Difeq/FunctionNamePrompt must advance the modal state"
    );
}

// ── submit_modal: defensive `_ =>` arm in submit_modal_with_label ────────────
//
// The `_ =>` arm at the end of `submit_modal_with_label` is a defensive guard
// that satisfies Rust's exhaustiveness rule. It is only reachable if a new
// ModalProgram variant is added that passes `requires_alpha_label()` without
// being one of the three explicitly-matched variants (Solve, Integ, Difeq).
//
// Currently unreachable from the state machine because:
//   - `requires_alpha_label()` returns true ONLY for those three variants
//     (see modal.rs::ModalProgram::requires_alpha_label implementation)
//   - The `_ =>` arm is thus a compile-time exhaustiveness sentinel, not a
//     runtime path.
//
// This is a known defensive arm. It is documented here per the 32-04
// `quadratic_zero_leading_returns_domain` pattern as UNREACHABLE. Adding a
// test stub would require constructing a custom ModalProgram variant that
// satisfies `requires_alpha_label()` but is NOT one of the three explicit
// arms — which is impossible without changing the enum itself.
// UNREACHABLE arm: documented, not padded.
