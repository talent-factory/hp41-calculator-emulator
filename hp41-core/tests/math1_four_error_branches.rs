// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Plan 32-05 surgical gap-closure for `hp41-core/src/ops/math1/four.rs`.
//!
//! Closes the 81.29 % per-file coverage gap by exercising:
//! - `compute_dft` empty-samples guard (line 139 -> `Err(HpError::Domain)`)
//! - `op_four_eval_at_t` no-valid-period guard (line 285 -> `Err(HpError::Domain)`)
//! - Five `submit_step` register-count guards (lines 349, 365, 375, 386, 408
//!   -> `Err(HpError::InvalidOp)`)
//! - `submit_step Ready` arm (line 423 -> `Err(HpError::InvalidOp)`)
//! - The `submit_step SamplePrompt` success path advancing to `Ready` (all samples entered)
//! - The full `submit_step` chain `NumSamplesPrompt → NumFreqPrompt → FirstCoeffPrompt
//!   → RectTogglePrompt → SamplePrompt(0) → Ready`
//!
//! All tests carry `// Catches:` doc comments per D-27.1.

#![allow(clippy::unwrap_used)]

use approx::assert_relative_eq;
use hp41_core::error::HpError;
use hp41_core::ops::math1::four::{
    compute_dft, op_four_eval_at_t, submit_step as four_submit_step, SAMPLE_OFFSET,
};
use hp41_core::ops::math1::modal::{FourInputStep, ModalProgram};
use hp41_core::state::CalcState;
use hp41_core::{HpNum};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

fn f64_hpnum(v: f64) -> HpNum {
    HpNum::rounded(Decimal::from_f64(v).unwrap())
}

fn make_state() -> CalcState {
    CalcState::new()
}

fn set_x(state: &mut CalcState, v: f64) {
    state.stack.x = f64_hpnum(v);
}

// ── compute_dft error guards ──────────────────────────────────────────────────

// Catches: compute_dft not returning Domain for empty sample slice (line 139)
#[test]
fn compute_dft_empty_samples_returns_domain() {
    // Source: HP Math Pac I OM FOUR program — N must be >= 1.
    // compute_dft with empty slice must return Err(HpError::Domain).
    let result = compute_dft(&[], 2);
    assert!(
        matches!(result, Err(HpError::Domain)),
        "compute_dft([]) must return Err(HpError::Domain), got {result:?}"
    );
}

// ── op_four_eval_at_t no-valid-period guard ───────────────────────────────────

// Catches: op_four_eval_at_t returning garbage result when both period and N are 0
#[test]
fn four_eval_at_t_no_valid_period_returns_domain() {
    // Source: op_four_eval_at_t (line 285): if period <= 0 AND N (R23) == 0,
    // returns Err(HpError::Domain) — no valid period available.
    let state = make_state();
    // R23 = 0 (no sample count stored — default zero from CalcState::new())
    // period = 0 (HpNum::zero())
    let t = f64_hpnum(1.0);
    let period = HpNum::zero();
    let result = op_four_eval_at_t(&state, t, period);
    assert!(
        matches!(result, Err(HpError::Domain)),
        "op_four_eval_at_t with no valid period must return Err(HpError::Domain), got {result:?}"
    );
}

// Catches: op_four_eval_at_t returning garbage result when period is negative and N is 0
#[test]
fn four_eval_at_t_negative_period_and_zero_n_returns_domain() {
    // Source: period guard: negative period is treated same as zero (period_f64 > 0 check).
    // Both explicit negative period and R23=0 → no valid period → Domain.
    let state = make_state();
    // R23 defaults to 0; pass negative period
    let t = f64_hpnum(2.0);
    let period = f64_hpnum(-5.0); // negative period
    let result = op_four_eval_at_t(&state, t, period);
    assert!(
        matches!(result, Err(HpError::Domain)),
        "op_four_eval_at_t with negative period and N=0 must return Domain, got {result:?}"
    );
}

// ── submit_step register-count guard arms ────────────────────────────────────

// Catches: NumSamplesPrompt arm (line 349) not guarding against insufficient registers
#[test]
fn submit_step_arm_349_insufficient_regs_returns_invalid_op() {
    // Source: submit_step NumSamplesPrompt requires regs.len() >= 25 (lines 348-349).
    let mut state = make_state();
    state.regs.truncate(24); // force regs.len() == 24 < 25
    set_x(&mut state, 4.0); // N=4 samples
    let result = four_submit_step(&mut state, FourInputStep::NumSamplesPrompt);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "NumSamplesPrompt with regs.len()<25 must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: NumFreqPrompt arm (line 365) not guarding against insufficient registers
#[test]
fn submit_step_arm_365_insufficient_regs_returns_invalid_op() {
    // Source: submit_step NumFreqPrompt requires regs.len() >= 26 (lines 364-365).
    let mut state = make_state();
    state.regs.truncate(25); // force regs.len() == 25 < 26
    set_x(&mut state, 3.0); // L=3 frequencies
    let result = four_submit_step(&mut state, FourInputStep::NumFreqPrompt);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "NumFreqPrompt with regs.len()<26 must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: FirstCoeffPrompt arm (line 375) not guarding against insufficient registers
#[test]
fn submit_step_arm_375_insufficient_regs_returns_invalid_op() {
    // Source: submit_step FirstCoeffPrompt requires regs.len() >= 27 (lines 374-375).
    let mut state = make_state();
    state.regs.truncate(26); // force regs.len() == 26 < 27
    set_x(&mut state, 1.0); // start_idx=1
    let result = four_submit_step(&mut state, FourInputStep::FirstCoeffPrompt);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "FirstCoeffPrompt with regs.len()<27 must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: RectTogglePrompt arm (line 386) not guarding against insufficient registers
#[test]
fn submit_step_arm_386_insufficient_regs_returns_invalid_op() {
    // Source: submit_step RectTogglePrompt requires regs.len() >= 27 (lines 385-386).
    let mut state = make_state();
    state.regs.truncate(26); // force regs.len() == 26 < 27
    set_x(&mut state, 1.0); // rect=1 (rectangular output)
    let result = four_submit_step(&mut state, FourInputStep::RectTogglePrompt);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "RectTogglePrompt with regs.len()<27 must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: SamplePrompt arm (line 408) not returning InvalidOp when target index out of bounds
#[test]
fn submit_step_arm_408_sample_out_of_bounds_returns_invalid_op() {
    // Source: SamplePrompt(idx) stores sample at R{SAMPLE_OFFSET + idx}.
    // If target >= regs.len(), returns Err(HpError::InvalidOp) (line 408).
    // SAMPLE_OFFSET = 27, so for a state with exactly 27 regs, SamplePrompt(0)
    // targets R27 which is out of bounds.
    let mut state = make_state();
    // R23 must be set to N=1 so the step reads a valid sample count
    state.regs[23] = f64_hpnum(1.0);
    state.regs.truncate(27); // force regs.len() == 27; target = 27 + 0 = 27 >= 27
    set_x(&mut state, 5.0);
    let result = four_submit_step(&mut state, FourInputStep::SamplePrompt(0));
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "SamplePrompt(0) with target>=regs.len() must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: FourInputStep::Ready arm (line 423) not returning InvalidOp
#[test]
fn submit_step_ready_returns_invalid_op() {
    // Source: submit_step Ready arm returns Err(HpError::InvalidOp) unconditionally (line 423).
    // This guards against the CLI/GUI submitting a value when no prompt is active.
    let mut state = make_state();
    let result = four_submit_step(&mut state, FourInputStep::Ready);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "FourInputStep::Ready must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// ── submit_step success paths ─────────────────────────────────────────────────

// Catches: NumSamplesPrompt success path not advancing to NumFreqPrompt
#[test]
fn submit_step_num_samples_prompt_success() {
    // Source: HP Math Pac I OM FOUR program — step 1 stores N in R23, advances to NumFreqPrompt.
    let mut state = make_state();
    set_x(&mut state, 4.0); // N=4
    let result = four_submit_step(&mut state, FourInputStep::NumSamplesPrompt);
    assert!(result.is_ok(), "NumSamplesPrompt success must return Ok, got {result:?}");
    // R23 should store N=4
    let n = state.regs[23].inner().to_f64().unwrap();
    assert_relative_eq!(n, 4.0, max_relative = 1e-7);
    // Modal advances to NumFreqPrompt
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Four(FourInputStep::NumFreqPrompt))
    );
}

// Catches: NumFreqPrompt success path not capping at MAX_FOURIER_PAIRS and advancing
#[test]
fn submit_step_num_freq_prompt_success_with_cap() {
    // Source: HP Math Pac I OM FOUR-04: L capped at MAX_FOURIER_PAIRS (10).
    // Requesting 15 frequencies must be capped at 10.
    let mut state = make_state();
    set_x(&mut state, 15.0); // L=15 (above cap)
    let result = four_submit_step(&mut state, FourInputStep::NumFreqPrompt);
    assert!(result.is_ok(), "NumFreqPrompt success must return Ok, got {result:?}");
    let l = state.regs[24].inner().to_f64().unwrap();
    assert_relative_eq!(l, 10.0, max_relative = 1e-7); // capped at MAX_FOURIER_PAIRS
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Four(FourInputStep::FirstCoeffPrompt))
    );
}

// Catches: RectTogglePrompt success path not storing choice in R26
#[test]
fn submit_step_rect_toggle_prompt_success() {
    // Source: HP Math Pac I OM FOUR-03: RECT? toggle — non-zero = rectangular.
    // Stores the X value in R26, advances to SamplePrompt(0).
    let mut state = make_state();
    set_x(&mut state, 1.0); // rectangular output (non-zero)
    let result = four_submit_step(&mut state, FourInputStep::RectTogglePrompt);
    assert!(result.is_ok(), "RectTogglePrompt success must return Ok, got {result:?}");
    let rect_flag = state.regs[26].inner().to_f64().unwrap();
    assert_relative_eq!(rect_flag, 1.0, max_relative = 1e-7);
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Four(FourInputStep::SamplePrompt(0)))
    );
}

// Catches: SamplePrompt last-sample path not advancing to Ready
#[test]
fn submit_step_last_sample_advances_to_ready() {
    // Source: HP Math Pac I OM FOUR program — after all N samples are entered, state = Ready.
    // Setup: N=2 (R23), submit SamplePrompt(0) then SamplePrompt(1); second should advance to Ready.
    let mut state = make_state();
    state.regs[23] = f64_hpnum(2.0); // N=2

    // First sample (idx=0): should advance to SamplePrompt(1)
    set_x(&mut state, 3.0);
    let r0 = four_submit_step(&mut state, FourInputStep::SamplePrompt(0));
    assert!(r0.is_ok(), "SamplePrompt(0) must return Ok, got {r0:?}");
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Four(FourInputStep::SamplePrompt(1)))
    );
    // Sample 0 stored at R{SAMPLE_OFFSET + 0}
    let s0 = state.regs[SAMPLE_OFFSET].inner().to_f64().unwrap();
    assert_relative_eq!(s0, 3.0, max_relative = 1e-7);

    // Second sample (idx=1, which == N-1): should advance to Ready
    set_x(&mut state, 7.0);
    let r1 = four_submit_step(&mut state, FourInputStep::SamplePrompt(1));
    assert!(r1.is_ok(), "SamplePrompt(1) must return Ok, got {r1:?}");
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Four(FourInputStep::Ready))
    );
    assert!(state.modal_prompt.is_none(), "Ready state must have no prompt");
    // Sample 1 stored at R{SAMPLE_OFFSET + 1}
    let s1 = state.regs[SAMPLE_OFFSET + 1].inner().to_f64().unwrap();
    assert_relative_eq!(s1, 7.0, max_relative = 1e-7);
}

// Catches: op_four_eval_at_t not reading N from R23 when period = 0
#[test]
fn four_eval_at_t_uses_n_from_r23_when_period_zero() {
    // Source: op_four_eval_at_t (line 282): if period <= 0, fall back to N from R23.
    // Setup: a₀=0, a₁=1, b₁=0, N=8 from R23, L=1. Explicit period=0 → uses N=8.
    // f(0) = a₀/2 + a₁·cos(0) = 1.0.
    let mut state = make_state();
    state.regs[0] = HpNum::zero();
    state.regs[1] = f64_hpnum(1.0);
    state.regs[2] = HpNum::zero();
    state.regs[23] = f64_hpnum(8.0); // N=8 fallback period
    state.regs[24] = f64_hpnum(1.0); // L=1
    let t = HpNum::zero();
    let period = HpNum::zero(); // triggers N fallback
    let result = op_four_eval_at_t(&state, t, period).unwrap();
    let val = result.inner().to_f64().unwrap();
    assert_relative_eq!(val, 1.0, max_relative = 1e-6);
}
