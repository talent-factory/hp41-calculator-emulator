// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! FOUR — Fourier Analysis program implementation.
//!
//! ## Overview
//!
//! The FOUR program computes Discrete Fourier Transform (DFT) coefficients from
//! N user-supplied time-domain samples, producing up to L = `MAX_FOURIER_PAIRS` = 10
//! pairs of coefficients in either rectangular (aₙ, bₙ) or polar (cₙ, φₙ) form.
//!
//! ## Algorithm (FOUR-01..04 / HP-41C Math Pac I OM)
//!
//! Given N samples Y₁..YN and L frequency pairs requested:
//!
//! ```text
//! a₀ = (2/N) · Σ_{k=1..N} Yₖ  (DC component × 2)
//! aₙ = (2/N) · Σ_{k=1..N} Yₖ · cos(2π·n·k/N)  for n=1..L
//! bₙ = (2/N) · Σ_{k=1..N} Yₖ · sin(2π·n·k/N)  for n=1..L
//! ```
//!
//! ## Register layout (FOUR-05, scratch R00–R26)
//!
//! ```text
//! R00 = a₀   (DC, coefficient for n=0)
//! R01 = a₁   R02 = b₁  (frequency 1)
//! R03 = a₂   R04 = b₂  (frequency 2)
//! ...
//! R{2n-1} = aₙ  R{2n} = bₙ  (frequency n, for n=1..L, max L=10)
//! R21 = a₁₀  R22 = b₁₀  (last pair when L=10 → indices 21..22)
//! R23 = N (sample count, stored for USER-mode eval)
//! R24 = L (frequency count, stored for USER-mode eval)
//! R25 = (reserved)
//! R26 = (reserved)
//! ```
//!
//! ## FOUR-06 USER-mode E-key Evaluator
//!
//! After FOUR coefficient computation, the USER-mode E-key evaluates the Fourier series:
//! ```text
//! f(t) = a₀/2 + Σ_{n=1..L} (aₙ·cos(2π·n·t/T) + bₙ·sin(2π·n·t/T))
//! ```
//! where T = period = N (sample count, interpreting sample index k as time step k).
//! Reads (aₙ, bₙ) from R00..R{2L}, N from R23, L from R24.
//!
//! ## Phase boundary
//!
//! Plan 28-10 ships the evaluator FUNCTION (`op_four_eval_at_t`); CLI/GUI E-key dispatch
//! routing lands in Phases 29/31 per the v2.1 USER-mode E-key precedent.

use std::f64::consts::PI;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::math1::modal::{FourInputStep, ModalProgram};
use crate::state::CalcState;

/// Maximum number of Fourier coefficient pairs (frequency harmonics) per FOUR-04 / OM.
/// Requests for more than 10 frequencies are capped at this value.
pub const MAX_FOURIER_PAIRS: u8 = 10;

/// Scratch register range used by FOUR per FOUR-05 (R00–R26, inclusive).
/// R00 = a₀; R01..R22 = (aₙ, bₙ) pairs n=1..10; R23 = N; R24 = L.
pub const SCRATCH_RANGE: std::ops::Range<usize> = 0..27;

/// Convert a float to HpNum, returning Domain on NaN/infinity.
fn f64_to_hpnum(v: f64) -> Result<HpNum, HpError> {
    Decimal::from_f64(v)
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)
}

// ── Master entry ──────────────────────────────────────────────────────────────

/// FOUR — master entry. Opens the Fourier-analysis modal workflow.
///
/// Sets `state.modal_program = Some(ModalProgram::Four(NumSamplesPrompt))` and
/// `state.modal_prompt = Some("NO. SAMPLES=?")`.
///
/// LiftEffect: Neutral (opens modal; no stack interaction at entry).
/// FOUR-01 / HP-41C Math Pac I OM.
///
/// The actual DFT computation and sample-input loop run inside the modal routing
/// (Phases 29 CLI / Phase 31 GUI). Plan 28-10 ships `compute_dft` and
/// `op_four_eval_at_t` for unit-testable access; the modal conductor (Phases 29/31)
/// calls `compute_dft` after all samples are collected.
pub fn op_four(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Four(FourInputStep::NumSamplesPrompt));
    state.modal_prompt = Some("NO. SAMPLES=?".to_string());
    Ok(())
}

// ── DFT computation ───────────────────────────────────────────────────────────

/// Compute Discrete Fourier Transform coefficient pairs from N samples.
///
/// ## Arguments
/// - `samples`: slice of N time-domain sample values Y₁..YN
/// - `num_freq`: number of frequency pairs L to compute (capped at `MAX_FOURIER_PAIRS`)
///
/// ## Returns
/// Vector of (aₙ, bₙ) pairs:
/// - Index 0: (a₀, 0.0) — DC component (b₀ = 0 by definition)
/// - Index n: (aₙ, bₙ) for frequency harmonic n = 1..L
///
/// ## Algorithm
/// ```text
/// aₙ = (2/N) · Σ_{k=1..N} Yₖ · cos(2π·n·k/N)
/// bₙ = (2/N) · Σ_{k=1..N} Yₖ · sin(2π·n·k/N)
/// ```
/// (For n=0: a₀ = (2/N) · Σ Yₖ; b₀ = 0.)
///
/// ## Register layout (stored after computation, FOUR-05)
/// After calling this function, the caller should store pairs in scratch registers:
/// R00 = a₀, R01 = a₁, R02 = b₁, R03 = a₂, R04 = b₂, ...,
/// R{2n-1} = aₙ, R{2n} = bₙ (n ≥ 1). R23 = N, R24 = L.
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), FOUR program, DFT algorithm.
pub fn compute_dft(samples: &[HpNum], num_freq: u8) -> Result<Vec<(HpNum, HpNum)>, HpError> {
    let n = samples.len();
    if n == 0 {
        return Err(HpError::Domain);
    }
    // Cap num_freq at MAX_FOURIER_PAIRS per FOUR-04.
    let l = (num_freq.min(MAX_FOURIER_PAIRS)) as usize;
    let n_f = n as f64;
    let two_over_n = 2.0 / n_f;
    let two_pi_over_n = 2.0 * PI / n_f;

    // Convert samples to f64 for efficient computation.
    let samples_f64: Vec<f64> = samples
        .iter()
        .map(|s| s.inner().to_f64().ok_or(HpError::Overflow))
        .collect::<Result<Vec<f64>, HpError>>()?;

    let mut pairs = Vec::with_capacity(l + 1);

    for freq in 0..=l {
        let mut a_sum = 0.0_f64;
        let mut b_sum = 0.0_f64;

        for (k_idx, &yk) in samples_f64.iter().enumerate() {
            let k = (k_idx + 1) as f64; // samples are 1-indexed per OM
            let angle = two_pi_over_n * (freq as f64) * k;
            a_sum += yk * angle.cos();
            b_sum += yk * angle.sin();
        }

        let an = f64_to_hpnum(two_over_n * a_sum)?;
        let bn = if freq == 0 {
            // b₀ = 0 by definition (sin(0) = 0 for all k)
            HpNum::zero()
        } else {
            f64_to_hpnum(two_over_n * b_sum)?
        };
        pairs.push((an, bn));
    }

    Ok(pairs)
}

/// Store DFT coefficient pairs in CalcState scratch registers per FOUR-05.
///
/// Register layout:
/// - R00 = a₀  (DC component)
/// - R{2n-1} = aₙ,  R{2n} = bₙ  for n = 1..L
/// - R23 = N (sample count, needed for USER-mode eval)
/// - R24 = L (frequency count, needed for USER-mode eval)
///
/// Precondition: pairs.len() == L + 1 (index 0 is DC, indices 1..L are harmonics).
pub fn store_dft_to_registers(state: &mut CalcState, pairs: &[(HpNum, HpNum)], n_samples: usize) {
    // R00 = a₀ (DC)
    if let Some((a0, _)) = pairs.first() {
        if state.regs.len() > 0 {
            state.regs[0] = a0.clone();
        }
    }
    // R{2n-1} = aₙ, R{2n} = bₙ for n = 1..L
    for (n, (an, bn)) in pairs.iter().enumerate().skip(1) {
        let idx_a = 2 * n - 1;
        let idx_b = 2 * n;
        if idx_b < SCRATCH_RANGE.end && idx_b < state.regs.len() {
            state.regs[idx_a] = an.clone();
            state.regs[idx_b] = bn.clone();
        }
    }
    // R23 = N, R24 = L
    let l = pairs.len().saturating_sub(1);
    if state.regs.len() > 24 {
        state.regs[23] = HpNum::rounded(Decimal::from(n_samples as u64));
        state.regs[24] = HpNum::rounded(Decimal::from(l as u64));
    }
}

// ── RECT? toggle ──────────────────────────────────────────────────────────────

/// Convert (aₙ, bₙ) rectangular pairs to polar (cₙ, φₙ) form per FOUR-03.
///
/// ```text
/// cₙ = √(aₙ² + bₙ²)   (magnitude)
/// φₙ = atan2(bₙ, aₙ)  (phase angle, in radians)
/// ```
///
/// Index 0 (DC pair, b₀=0): c₀ = |a₀|, φ₀ = 0 (or π if a₀ < 0).
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), FOUR RECT? toggle.
pub fn convert_to_polar(pairs: &[(HpNum, HpNum)]) -> Result<Vec<(HpNum, HpNum)>, HpError> {
    pairs
        .iter()
        .map(|(an, bn)| {
            let a = an.inner().to_f64().ok_or(HpError::Overflow)?;
            let b = bn.inner().to_f64().ok_or(HpError::Overflow)?;
            let c = (a * a + b * b).sqrt();
            let phi = b.atan2(a);
            Ok((f64_to_hpnum(c)?, f64_to_hpnum(phi)?))
        })
        .collect()
}

// ── USER-mode E-key evaluator (FOUR-06) ──────────────────────────────────────

/// Evaluate the Fourier series at time t (FOUR-06 USER-mode E-key).
///
/// Reads coefficient data from CalcState scratch registers:
/// - R00 = a₀ (DC)
/// - R{2n-1} = aₙ, R{2n} = bₙ for n = 1..L
/// - R23 = N (period T = N time steps, if period == 0)
/// - R24 = L (number of frequency pairs)
///
/// ## Formula
/// ```text
/// f(t) = a₀/2 + Σ_{n=1..L} (aₙ·cos(2π·n·t/T) + bₙ·sin(2π·n·t/T))
/// ```
///
/// ## Arguments
/// - `state`: CalcState with coefficient scratch registers populated by a prior FOUR computation
/// - `t`: time value at which to evaluate (X register value prior to E-key press)
/// - `period`: the period T; if zero or negative, falls back to N from R23
///
/// ## Returns
/// The Fourier series sum f(t) as HpNum.
///
/// ## Phase boundary (FOUR-06)
/// Plan 28-10 ships this function. CLI/GUI E-key routing (Phases 29/31) will call it when:
/// `state.user_mode == true` AND `state.modal_program == Some(ModalProgram::Four(Ready))`.
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), FOUR program, series evaluation.
pub fn op_four_eval_at_t(state: &CalcState, t: HpNum, period: HpNum) -> Result<HpNum, HpError> {
    // Read N (sample count) from R23 and L (freq count) from R24.
    let n_from_reg = state
        .regs
        .get(23)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let l_from_reg = state
        .regs
        .get(24)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);

    // Determine period T: use provided period if positive, else fall back to N from R23.
    let period_f64 = period.inner().to_f64().unwrap_or(0.0);
    let t_period = if period_f64 > 0.0 {
        period_f64
    } else if n_from_reg > 0.0 {
        n_from_reg
    } else {
        return Err(HpError::Domain); // no valid period
    };

    let l = l_from_reg.round() as usize;
    let t_val = t.inner().to_f64().ok_or(HpError::Overflow)?;
    let two_pi_over_t = 2.0 * PI / t_period;

    // a₀/2 (DC component)
    let a0 = state
        .regs
        .first()
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let mut sum = a0 / 2.0;

    // Σ_{n=1..L} (aₙ·cos(2π·n·t/T) + bₙ·sin(2π·n·t/T))
    for n in 1..=l {
        let idx_a = 2 * n - 1;
        let idx_b = 2 * n;
        let an = state
            .regs
            .get(idx_a)
            .map(|v| v.inner().to_f64().unwrap_or(0.0))
            .unwrap_or(0.0);
        let bn = state
            .regs
            .get(idx_b)
            .map(|v| v.inner().to_f64().unwrap_or(0.0))
            .unwrap_or(0.0);
        let angle = two_pi_over_t * (n as f64) * t_val;
        sum += an * angle.cos() + bn * angle.sin();
    }

    f64_to_hpnum(sum)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;

    const TOLERANCE: f64 = 1e-6;

    /// Check that two HpNum values are within `tolerance` of each other.
    fn approx_eq(a: &HpNum, b: f64, tolerance: f64) -> bool {
        let av = a.inner().to_f64().unwrap();
        (av - b).abs() < tolerance
    }

    // Catches: Op::Four master entry not setting modal_program correctly
    #[test]
    fn master_op_opens_modal() {
        use crate::ops::math1::modal::FourInputStep;
        let mut state = CalcState::new();
        op_four(&mut state).unwrap();
        assert_eq!(
            state.modal_program,
            Some(ModalProgram::Four(FourInputStep::NumSamplesPrompt))
        );
        assert_eq!(state.modal_prompt, Some("NO. SAMPLES=?".to_string()));
    }

    // Catches: DFT DC component wrong for constant signal
    // Constant signal Y=5: a₀ = (2/N)·N·5 = 10; all other coeffs = 0.
    // Source: Basic Fourier series property — constant signal has only DC component.
    #[test]
    fn dft_constant_signal() {
        let n = 4;
        let samples: Vec<HpNum> = (0..n).map(|_| HpNum::rounded(Decimal::from(5))).collect();
        let pairs = compute_dft(&samples, 2).unwrap();
        // a₀ = 10.0 (DC component = 2 × mean = 2 × 5 = 10)
        assert!(approx_eq(&pairs[0].0, 10.0, TOLERANCE), "a₀ should be 10");
        assert!(approx_eq(&pairs[0].1, 0.0, TOLERANCE), "b₀ should be 0");
        // All higher harmonics should be ≈ 0
        assert!(approx_eq(&pairs[1].0, 0.0, TOLERANCE), "a₁ should be ≈ 0");
        assert!(approx_eq(&pairs[1].1, 0.0, TOLERANCE), "b₁ should be ≈ 0");
    }

    // Catches: DFT sine signal wrong (b₁ should be 1, all others ≈ 0)
    // N=8 samples of sin(2π·k/8) for k=1..8.
    // Source: Fourier series identity: DFT of sin(2πk/N) = b₁=1, others=0.
    #[test]
    fn dft_pure_sine() {
        let n = 8usize;
        let samples: Vec<HpNum> = (1..=n)
            .map(|k| {
                let v = (2.0 * PI * k as f64 / n as f64).sin();
                HpNum::rounded(Decimal::from_f64(v).unwrap())
            })
            .collect();
        let pairs = compute_dft(&samples, 3).unwrap();
        // a₀ ≈ 0, b₀ = 0 (by definition)
        assert!(approx_eq(&pairs[0].0, 0.0, TOLERANCE), "a₀ should be ≈ 0");
        // a₁ ≈ 0, b₁ ≈ 1.0 (pure sine at frequency 1)
        assert!(approx_eq(&pairs[1].0, 0.0, TOLERANCE), "a₁ should be ≈ 0");
        assert!(approx_eq(&pairs[1].1, 1.0, TOLERANCE), "b₁ should be ≈ 1");
        // Higher harmonics ≈ 0
        assert!(approx_eq(&pairs[2].0, 0.0, TOLERANCE), "a₂ should be ≈ 0");
        assert!(approx_eq(&pairs[2].1, 0.0, TOLERANCE), "b₂ should be ≈ 0");
    }

    // Catches: DFT cosine signal wrong (a₁ should be 1, all others ≈ 0)
    // N=8 samples of cos(2π·k/8) for k=1..8.
    // Source: Fourier series identity: DFT of cos(2πk/N) = a₁=1, others=0.
    #[test]
    fn dft_pure_cosine() {
        let n = 8usize;
        let samples: Vec<HpNum> = (1..=n)
            .map(|k| {
                let v = (2.0 * PI * k as f64 / n as f64).cos();
                HpNum::rounded(Decimal::from_f64(v).unwrap())
            })
            .collect();
        let pairs = compute_dft(&samples, 3).unwrap();
        // a₁ ≈ 1.0, b₁ ≈ 0
        assert!(approx_eq(&pairs[1].0, 1.0, TOLERANCE), "a₁ should be ≈ 1");
        assert!(approx_eq(&pairs[1].1, 0.0, TOLERANCE), "b₁ should be ≈ 0");
        // Higher harmonics ≈ 0
        assert!(approx_eq(&pairs[2].0, 0.0, TOLERANCE), "a₂ should be ≈ 0");
        assert!(approx_eq(&pairs[2].1, 0.0, TOLERANCE), "b₂ should be ≈ 0");
    }

    // Catches: RECT? toggle rectangular form not reading back correctly
    #[test]
    fn rect_toggle_rectangular_form() {
        let pairs = vec![(
            HpNum::rounded(Decimal::from(3)),
            HpNum::rounded(Decimal::from(4)),
        )];
        // In rectangular form, the pairs are unchanged.
        assert!(approx_eq(&pairs[0].0, 3.0, TOLERANCE));
        assert!(approx_eq(&pairs[0].1, 4.0, TOLERANCE));
    }

    // Catches: RECT? toggle polar form conversion wrong (FOUR-03)
    // (a, b) = (3, 4) → c = 5.0, φ = atan2(4, 3) ≈ 0.927295
    #[test]
    fn rect_toggle_polar_form() {
        let pairs = vec![(
            HpNum::rounded(Decimal::from(3)),
            HpNum::rounded(Decimal::from(4)),
        )];
        let polar = convert_to_polar(&pairs).unwrap();
        assert!(
            approx_eq(&polar[0].0, 5.0, TOLERANCE),
            "magnitude c should be 5"
        );
        let expected_phi = (4.0f64).atan2(3.0);
        assert!(
            approx_eq(&polar[0].1, expected_phi, TOLERANCE),
            "phase φ should be atan2(4,3)"
        );
    }

    // Catches: MAX_FOURIER_PAIRS cap not enforced (FOUR-04)
    // Requesting 11 frequencies should cap at 10 (MAX_FOURIER_PAIRS).
    #[test]
    fn four_pairs_cap() {
        let n = 8usize;
        let samples: Vec<HpNum> = (0..n).map(|_| HpNum::rounded(Decimal::from(1))).collect();
        // Request 11 frequencies (above cap)
        let pairs = compute_dft(&samples, 11).unwrap();
        // Should produce MAX_FOURIER_PAIRS + 1 entries (index 0 = DC, 1..10 = harmonics)
        assert_eq!(
            pairs.len(),
            (MAX_FOURIER_PAIRS as usize) + 1,
            "pairs.len() should be capped at MAX_FOURIER_PAIRS+1 = 11"
        );
    }

    // Catches: scratch register layout wrong (FOUR-05)
    // After store_dft_to_registers: R00=a₀, R01=a₁, R02=b₁, R23=N, R24=L.
    #[test]
    fn scratch_registers() {
        let mut state = CalcState::new();
        let samples: Vec<HpNum> = (0..4usize)
            .map(|_| HpNum::rounded(Decimal::from(2)))
            .collect();
        let pairs = compute_dft(&samples, 2).unwrap();
        store_dft_to_registers(&mut state, &pairs, 4);
        // R00 = a₀ = 4.0 (constant 2 → a₀ = (2/4)·4·2 = 4.0)
        let r00 = state.regs[0].inner().to_f64().unwrap();
        assert!((r00 - 4.0).abs() < TOLERANCE, "R00 = a₀ ≈ 4.0, got {r00}");
        // R23 = N = 4
        let r23 = state.regs[23].inner().to_f64().unwrap();
        assert!((r23 - 4.0).abs() < TOLERANCE, "R23 = N = 4, got {r23}");
        // R24 = L = 2
        let r24 = state.regs[24].inner().to_f64().unwrap();
        assert!((r24 - 2.0).abs() < TOLERANCE, "R24 = L = 2, got {r24}");
        // R01 = a₁ ≈ 0 (constant signal has no harmonics)
        let r01 = state.regs[1].inner().to_f64().unwrap();
        assert!(
            r01.abs() < TOLERANCE,
            "R01 = a₁ ≈ 0 for constant signal, got {r01}"
        );
    }

    // Catches: USER-mode E-key evaluator wrong at t=0 for unit cosine signal (FOUR-06)
    // Pre-stage: a₀=0, a₁=1, b₁=0, N=8, L=1.
    // f(0) = a₀/2 + a₁·cos(0) + b₁·sin(0) = 0 + 1·1 + 0 = 1.0
    #[test]
    fn user_mode_eval_at_t_zero() {
        let mut state = CalcState::new();
        // a₀ = 0
        state.regs[0] = HpNum::zero();
        // a₁ = 1, b₁ = 0 (pure cosine at frequency 1)
        state.regs[1] = HpNum::rounded(Decimal::from(1));
        state.regs[2] = HpNum::zero();
        // N = 8, L = 1
        state.regs[23] = HpNum::rounded(Decimal::from(8));
        state.regs[24] = HpNum::rounded(Decimal::from(1));

        let t = HpNum::zero(); // t = 0
        let period = HpNum::zero(); // use N from R23
        let result = op_four_eval_at_t(&state, t, period).unwrap();
        // f(0) = 0/2 + 1·cos(0) + 0·sin(0) = 1.0
        assert!(
            approx_eq(&result, 1.0, TOLERANCE),
            "f(0) should be 1.0, got {:?}",
            result
        );
    }

    // Catches: USER-mode E-key evaluator wrong at t=2 for unit cosine signal
    // f(2) = a₁·cos(2π·1·2/8) = cos(π/2) = 0
    #[test]
    fn user_mode_eval_at_t_quarter() {
        let mut state = CalcState::new();
        state.regs[0] = HpNum::zero();
        state.regs[1] = HpNum::rounded(Decimal::from(1));
        state.regs[2] = HpNum::zero();
        state.regs[23] = HpNum::rounded(Decimal::from(8));
        state.regs[24] = HpNum::rounded(Decimal::from(1));

        let t = HpNum::rounded(Decimal::from(2)); // t = 2 (quarter period)
        let period = HpNum::zero();
        let result = op_four_eval_at_t(&state, t, period).unwrap();
        // f(2) = cos(2π·1·2/8) = cos(π/2) ≈ 0
        assert!(
            approx_eq(&result, 0.0, 1e-5),
            "f(2) = cos(π/2) ≈ 0, got {:?}",
            result
        );
    }

    // Catches: USER-mode E-key evaluator wrong at t=4 for unit cosine signal
    // f(4) = a₁·cos(2π·1·4/8) = cos(π) = -1
    #[test]
    fn user_mode_eval_at_t_half() {
        let mut state = CalcState::new();
        state.regs[0] = HpNum::zero();
        state.regs[1] = HpNum::rounded(Decimal::from(1));
        state.regs[2] = HpNum::zero();
        state.regs[23] = HpNum::rounded(Decimal::from(8));
        state.regs[24] = HpNum::rounded(Decimal::from(1));

        let t = HpNum::rounded(Decimal::from(4)); // t = 4 (half period)
        let period = HpNum::zero();
        let result = op_four_eval_at_t(&state, t, period).unwrap();
        // f(4) = cos(2π·1·4/8) = cos(π) = -1
        assert!(
            approx_eq(&result, -1.0, 1e-5),
            "f(4) = cos(π) = -1, got {:?}",
            result
        );
    }

    // Catches: USER-mode E-key DC component wrong (a₀/2 contribution)
    // Pre-stage: a₀=4, no harmonics (L=0). f(t) = 4/2 = 2 for all t.
    #[test]
    fn user_mode_eval_dc_only() {
        let mut state = CalcState::new();
        state.regs[0] = HpNum::rounded(Decimal::from(4));
        state.regs[23] = HpNum::rounded(Decimal::from(8));
        state.regs[24] = HpNum::zero(); // L = 0 (only DC)

        let t = HpNum::rounded(Decimal::from(3));
        let period = HpNum::zero();
        let result = op_four_eval_at_t(&state, t, period).unwrap();
        // f(t) = a₀/2 = 4/2 = 2.0
        assert!(
            approx_eq(&result, 2.0, TOLERANCE),
            "DC-only f(t) = a₀/2 = 2, got {:?}",
            result
        );
    }

    // Catches: USER-mode E-key period override wrong (explicit period > 0)
    // f(t) = a₁·cos(2π·t/T) with T=4, a₁=1, t=1 → cos(π/2) ≈ 0
    #[test]
    fn user_mode_eval_explicit_period() {
        let mut state = CalcState::new();
        state.regs[0] = HpNum::zero();
        state.regs[1] = HpNum::rounded(Decimal::from(1));
        state.regs[2] = HpNum::zero();
        state.regs[24] = HpNum::rounded(Decimal::from(1)); // L = 1

        let t = HpNum::rounded(Decimal::from(1));
        let period = HpNum::rounded(Decimal::from(4)); // explicit period = 4
        let result = op_four_eval_at_t(&state, t, period).unwrap();
        // f(1) = cos(2π·1·1/4) = cos(π/2) ≈ 0
        assert!(
            approx_eq(&result, 0.0, 1e-5),
            "f(1) with T=4: cos(π/2) ≈ 0, got {:?}",
            result
        );
    }
}
