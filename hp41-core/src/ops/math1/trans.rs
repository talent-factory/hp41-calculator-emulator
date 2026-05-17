// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! TRANS — Coordinate Transformations (2D rotation+translation + 3D Rodrigues rotation).
//!
//! ## Source
//! HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter on Coordinate Transformations.
//!
//! ## Programs
//!
//! ### TRANS (Op::Trans2d — TRANS-01..02)
//!
//! 2D coordinate transformation: rotation by θ followed by translation to (x₀, y₀).
//!
//! Initialization (A-entry, TRANS-01): stores (x₀, y₀, θ) in R00/R01/R02.
//! Forward transform (C-entry, TRANS-02): given input (x, y), computes:
//! ```text
//! x' = (x - x₀)·cos(θ) + (y - y₀)·sin(θ)
//! y' = -(x - x₀)·sin(θ) + (y - y₀)·cos(θ)
//! ```
//! Inverse transform (E-entry, TRANS-02): given (x', y'), computes:
//! ```text
//! x = x₀ + x'·cos(θ) - y'·sin(θ)
//! y = y₀ + x'·sin(θ) + y'·cos(θ)
//! ```
//!
//! Scratch registers: R00=x₀, R01=y₀, R02=θ (TRANS_2D_SCRATCH_RANGE = 0..3).
//!
//! ### T3D (Op::Trans3d — TRANS-03..04)
//!
//! 3D coordinate transformation via Rodrigues' rotation formula.
//!
//! Initialization A-entry (TRANS-03): origin (x₀, y₀, z₀) stored in R00/R01/R02.
//! Initialization B-entry (TRANS-03): rotation axis (a, b, c) and angle θ stored in R03..R06.
//! Forward transform (C-entry, TRANS-04): given input (x, y, z):
//! ```text
//! v = (x-x₀, y-y₀, z-z₀)
//! k = unit-normalize(a, b, c)
//! v' = v·cos(θ) + (k×v)·sin(θ) + k·(k·v)·(1-cos(θ))   ← Rodrigues' formula
//! output = v' + origin
//! ```
//! Inverse transform (E-entry, TRANS-04): rotate by -θ using Rodrigues.
//!
//! Scratch registers: R00..R06 (origin x₀/y₀/z₀, axis a/b/c, θ).
//! Full range per TRANS-05: TRANS_3D_SCRATCH_RANGE = 0..25.
//!
//! ## Rodrigues' Rotation Formula (TRANS-04)
//!
//! Given a 3D vector **v**, a unit rotation axis **k**, and angle θ:
//! ```text
//! v_rotated = v·cos(θ) + (k × v)·sin(θ) + k·(k·v)·(1 - cos(θ))
//! ```
//! Where `k·v` is the dot product (scalar) and `k × v` is the cross product (vector).
//! This is Rodrigues' rotation formula — rotates v around axis k by angle θ.
//! Inverse rotation uses -θ: cos(-θ) = cos(θ), sin(-θ) = -sin(θ).

use std::f64::consts::PI;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::math1::modal::{ModalProgram, TransInputStep};
use crate::state::{AngleMode, CalcState};

/// Scratch register range for 2D transforms (R00–R02: x₀, y₀, θ).
pub const TRANS_2D_SCRATCH_RANGE: std::ops::Range<usize> = 0..3;

/// Scratch register range for 3D transforms per TRANS-05 (R00–R24).
pub const TRANS_3D_SCRATCH_RANGE: std::ops::Range<usize> = 0..25;

// ── Angle conversion helpers ──────────────────────────────────────────────────

fn to_radians(angle: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Rad => angle,
        AngleMode::Deg => angle.to_radians(),
        AngleMode::Grad => angle * (PI / 200.0),
    }
}

fn f64_to_hpnum(v: f64) -> Result<HpNum, HpError> {
    Decimal::from_f64(v)
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)
}

// ── 3D vector helpers ─────────────────────────────────────────────────────────

/// Cross product of two 3D vectors: a × b.
///
/// Standard algebraic cross product formula:
/// `(a₁, a₂, a₃) × (b₁, b₂, b₃) = (a₂b₃ - a₃b₂, a₃b₁ - a₁b₃, a₁b₂ - a₂b₁)`
fn cross_product_3d(a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
    (
        a.1 * b.2 - a.2 * b.1,
        a.2 * b.0 - a.0 * b.2,
        a.0 * b.1 - a.1 * b.0,
    )
}

/// Dot product of two 3D vectors: a · b.
fn dot_product_3d(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

/// Unit-normalize a 3D vector (a, b, c).
///
/// Returns `Err(HpError::Domain)` if the vector has zero length (degenerate axis).
/// The threshold 1e-14 guards against floating-point underflow creating a phantom
/// unit vector from a near-zero axis.
fn normalize_3d(a: f64, b: f64, c: f64) -> Result<(f64, f64, f64), HpError> {
    let len = (a * a + b * b + c * c).sqrt();
    if len < 1e-14 {
        return Err(HpError::Domain); // zero-length axis: cannot normalize
    }
    Ok((a / len, b / len, c / len))
}

/// Apply Rodrigues' rotation formula to rotate vector v around unit axis k by angle θ.
///
/// ```text
/// v_rotated = v·cos(θ) + (k × v)·sin(θ) + k·(k·v)·(1 - cos(θ))
/// ```
///
/// Inverse rotation: pass `theta = -original_theta` (cos is even, sin is odd).
fn rodrigues_rotate(v: (f64, f64, f64), k: (f64, f64, f64), theta: f64) -> (f64, f64, f64) {
    let cos_t = theta.cos();
    let sin_t = theta.sin();
    let k_dot_v = dot_product_3d(k, v);
    let k_cross_v = cross_product_3d(k, v);

    (
        v.0 * cos_t + k_cross_v.0 * sin_t + k.0 * k_dot_v * (1.0 - cos_t),
        v.1 * cos_t + k_cross_v.1 * sin_t + k.1 * k_dot_v * (1.0 - cos_t),
        v.2 * cos_t + k_cross_v.2 * sin_t + k.2 * k_dot_v * (1.0 - cos_t),
    )
}

// ── 2D transform sub-operations ───────────────────────────────────────────────

/// Execute the 2D forward transform using parameters from CalcState scratch R00..R02.
///
/// Parameters from scratch:
/// - R00 = x₀ (translation origin x)
/// - R01 = y₀ (translation origin y)
/// - R02 = θ (rotation angle in the angle_mode used at init time — stored as raw value)
///
/// Input from stack: X = x, Y = y (the point to transform).
/// Output to stack: X = x', Y = y' (the transformed point).
///
/// Forward transform:
/// ```text
/// dx = x - x₀,  dy = y - y₀
/// x' = dx·cos(θ) + dy·sin(θ)
/// y' = -dx·sin(θ) + dy·cos(θ)
/// ```
/// LiftEffect: Enable (pushes transformed coordinates to stack).
pub fn do_trans2d_forward(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;

    let x0 = state
        .regs
        .first()
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let y0 = state
        .regs
        .get(1)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    // θ stored as raw value in current angle_mode
    let theta_raw = state
        .regs
        .get(2)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let theta = to_radians(theta_raw, state.angle_mode);

    let dx = x - x0;
    let dy = y - y0;
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    let x_prime = dx * cos_t + dy * sin_t;
    let y_prime = -dx * sin_t + dy * cos_t;

    state.stack.x = f64_to_hpnum(x_prime)?;
    state.stack.y = f64_to_hpnum(y_prime)?;
    // LiftEffect: Enable (stack already holds result; future enters will lift)
    state.stack.lift_enabled = true;
    Ok(())
}

/// Execute the 2D inverse transform using parameters from CalcState scratch R00..R02.
///
/// Input from stack: X = x', Y = y' (transformed point).
/// Output to stack: X = x, Y = y (original point).
///
/// Inverse transform:
/// ```text
/// x = x₀ + x'·cos(θ) - y'·sin(θ)
/// y = y₀ + x'·sin(θ) + y'·cos(θ)
/// ```
/// LiftEffect: Enable.
pub fn do_trans2d_inverse(state: &mut CalcState) -> Result<(), HpError> {
    let x_prime = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y_prime = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;

    let x0 = state
        .regs
        .first()
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let y0 = state
        .regs
        .get(1)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let theta_raw = state
        .regs
        .get(2)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let theta = to_radians(theta_raw, state.angle_mode);

    let cos_t = theta.cos();
    let sin_t = theta.sin();

    let x = x0 + x_prime * cos_t - y_prime * sin_t;
    let y = y0 + x_prime * sin_t + y_prime * cos_t;

    state.stack.x = f64_to_hpnum(x)?;
    state.stack.y = f64_to_hpnum(y)?;
    state.stack.lift_enabled = true;
    Ok(())
}

// ── 3D transform sub-operations ───────────────────────────────────────────────

/// Execute the 3D forward transform using Rodrigues' formula with scratch R00..R06.
///
/// Scratch layout (set during initialization):
/// - R00=x₀, R01=y₀, R02=z₀ (origin)
/// - R03=a,  R04=b,  R05=c  (rotation axis direction, need not be unit)
/// - R06=θ (rotation angle, raw value in current angle_mode)
///
/// Stack input: X=x, Y=y, Z=z (point to transform).
/// Stack output: X=x'', Y=y'', Z=z'' (transformed point).
///
/// Algorithm:
/// ```text
/// v = (x-x₀, y-y₀, z-z₀)
/// k = normalize(a, b, c)
/// v_rot = rodrigues(v, k, θ)
/// output = v_rot   [origin subtracted at start, no re-add for world→local transform]
/// ```
/// LiftEffect: Enable.
pub fn do_trans3d_forward(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;
    let z = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?;

    let x0 = state
        .regs
        .first()
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let y0 = state
        .regs
        .get(1)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let z0 = state
        .regs
        .get(2)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let axis_a = state
        .regs
        .get(3)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let axis_b = state
        .regs
        .get(4)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let axis_c = state
        .regs
        .get(5)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let theta_raw = state
        .regs
        .get(6)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let theta = to_radians(theta_raw, state.angle_mode);

    let k = normalize_3d(axis_a, axis_b, axis_c)?;
    let v = (x - x0, y - y0, z - z0);
    let v_rot = rodrigues_rotate(v, k, theta);

    state.stack.x = f64_to_hpnum(v_rot.0)?;
    state.stack.y = f64_to_hpnum(v_rot.1)?;
    state.stack.z = f64_to_hpnum(v_rot.2)?;
    state.stack.lift_enabled = true;
    Ok(())
}

/// Execute the 3D inverse transform (rotate by -θ) using Rodrigues' formula.
///
/// Inverse rotation: same as forward but with negative angle (θ → -θ).
/// Stack input: X=x'', Y=y'', Z=z'' (rotated point).
/// Stack output: X=x, Y=y, Z=z (original point, up to floating-point rounding).
/// LiftEffect: Enable.
pub fn do_trans3d_inverse(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
    let y = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?;
    let z = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?;

    let x0 = state
        .regs
        .first()
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let y0 = state
        .regs
        .get(1)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let z0 = state
        .regs
        .get(2)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let axis_a = state
        .regs
        .get(3)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let axis_b = state
        .regs
        .get(4)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let axis_c = state
        .regs
        .get(5)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let theta_raw = state
        .regs
        .get(6)
        .map(|v| v.inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);
    let theta = to_radians(theta_raw, state.angle_mode);

    let k = normalize_3d(axis_a, axis_b, axis_c)?;
    let v = (x, y, z);
    // Inverse: rotate by -θ (Rodrigues formula with negated angle)
    let v_back = rodrigues_rotate(v, k, -theta);

    // Re-add origin to get back to world coordinates
    state.stack.x = f64_to_hpnum(v_back.0 + x0)?;
    state.stack.y = f64_to_hpnum(v_back.1 + y0)?;
    state.stack.z = f64_to_hpnum(v_back.2 + z0)?;
    state.stack.lift_enabled = true;
    Ok(())
}

// ── Master entry ops ──────────────────────────────────────────────────────────

/// TRANS — 2D coordinate transform master entry (TRANS-01).
///
/// Opens the TRANS modal workflow with `Init2dPrompt`.
/// The actual transform parameters (x₀, y₀, θ) are entered via modal prompting
/// (Phases 29 CLI / Phase 31 GUI); plan 28-10 ships the helper functions.
///
/// LiftEffect: Neutral (modal opener; no stack interaction at entry).
pub fn op_trans2d(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Trans(TransInputStep::Init2dPrompt));
    state.modal_prompt = Some("X0,Y0,\u{03B8}?".to_string());
    Ok(())
}

/// T3D — 3D coordinate transform master entry (TRANS-03).
///
/// Opens the T3D modal workflow with `Init3dOriginPrompt`.
/// Parameters (origin, axis, θ) entered via modal prompting (Phases 29/31).
///
/// LiftEffect: Neutral (modal opener; no stack interaction at entry).
pub fn op_trans3d(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Trans(TransInputStep::Init3dOriginPrompt));
    state.modal_prompt = Some("ORIGIN?".to_string());
    Ok(())
}

/// Store 2D transform parameters in scratch registers (helper for tests and CLI/GUI modal).
///
/// Layout: R00=x₀, R01=y₀, R02=θ_raw (in user angle_mode units).
pub fn store_trans2d_params(state: &mut CalcState, x0: f64, y0: f64, theta_raw: f64) {
    if state.regs.len() > 2 {
        state.regs[0] = f64_to_hpnum(x0).unwrap_or_else(|_| HpNum::zero());
        state.regs[1] = f64_to_hpnum(y0).unwrap_or_else(|_| HpNum::zero());
        state.regs[2] = f64_to_hpnum(theta_raw).unwrap_or_else(|_| HpNum::zero());
    }
}

/// Store 3D transform parameters in scratch registers (helper for tests and CLI/GUI modal).
///
/// Layout: R00=x₀, R01=y₀, R02=z₀, R03=a, R04=b, R05=c, R06=θ_raw.
pub fn store_trans3d_params(
    state: &mut CalcState,
    origin: (f64, f64, f64),
    axis: (f64, f64, f64),
    theta_raw: f64,
) {
    if state.regs.len() > 6 {
        state.regs[0] = f64_to_hpnum(origin.0).unwrap_or_else(|_| HpNum::zero());
        state.regs[1] = f64_to_hpnum(origin.1).unwrap_or_else(|_| HpNum::zero());
        state.regs[2] = f64_to_hpnum(origin.2).unwrap_or_else(|_| HpNum::zero());
        state.regs[3] = f64_to_hpnum(axis.0).unwrap_or_else(|_| HpNum::zero());
        state.regs[4] = f64_to_hpnum(axis.1).unwrap_or_else(|_| HpNum::zero());
        state.regs[5] = f64_to_hpnum(axis.2).unwrap_or_else(|_| HpNum::zero());
        state.regs[6] = f64_to_hpnum(theta_raw).unwrap_or_else(|_| HpNum::zero());
    }
}

// ── Phase 29 / CLI-05 additive public surface — D-29.5 ───────────────────────

/// Submit a numeric input step in the TRANS modal workflow.
///
/// Called by `hp41_core::ops::math1::submit_modal` after `flush_entry_buf` has
/// flushed the entry buffer to `state.stack.x`. Reads X, advances the TRANS
/// modal step state machine, updates `state.modal_prompt`.
///
/// Step transitions:
/// - `Init2dPrompt` → reads X as x₀ (first of 3 params: x₀, y₀, θ entered sequentially),
///   stores in R00. Advances to Init2dPrompt... For simplicity in Phase 29 CLI implementation:
///   accepts a single R/S submit that reads the current X value and stores x₀,
///   then prompts for the next parameter in sequence. Full 2D/3D transform
///   initialization requires 3 values; each R/S submits one value in sequence.
///   This implementation uses a simplified single-value read per step.
/// - `Init3dOriginPrompt` → reads X as origin x₀, stores in R00,
///   advances to Init3dAxisPrompt.
/// - `Init3dAxisPrompt` → stores X in R03, advances to Ready.
/// - `ForwardPrompt` → reads X (input point), runs forward transform, pushes result;
///   stays in ForwardPrompt for repeated use.
/// - `InversePrompt` → reads X, runs inverse transform; stays in InversePrompt.
/// - `Ready` → `Err(HpError::InvalidOp)`.
///
/// Phase 29 / CLI-05 additive public surface — D-29.5.
pub fn submit_step(
    state: &mut CalcState,
    step: TransInputStep,
) -> Result<(), HpError> {
    match step {
        TransInputStep::Init2dPrompt => {
            // Phase 29 simplified: read x₀ from X, advance to ForwardPrompt.
            // Full multi-step 2D init (x₀, y₀, θ) deferred to Phase 31 enhanced CLI.
            if state.regs.len() < 3 {
                return Err(HpError::InvalidOp);
            }
            state.regs[0] = state.stack.x.clone();
            state.modal_program = Some(ModalProgram::Trans(TransInputStep::ForwardPrompt));
            state.modal_prompt = Some("FWD?".to_string());
            Ok(())
        }
        TransInputStep::Init3dOriginPrompt => {
            if state.regs.len() < 4 {
                return Err(HpError::InvalidOp);
            }
            state.regs[0] = state.stack.x.clone();
            state.modal_program = Some(ModalProgram::Trans(TransInputStep::Init3dAxisPrompt));
            state.modal_prompt = Some("AXIS+\u{03B8}?".to_string());
            Ok(())
        }
        TransInputStep::Init3dAxisPrompt => {
            if state.regs.len() < 7 {
                return Err(HpError::InvalidOp);
            }
            state.regs[3] = state.stack.x.clone();
            state.modal_program = Some(ModalProgram::Trans(TransInputStep::ForwardPrompt));
            state.modal_prompt = Some("FWD?".to_string());
            Ok(())
        }
        TransInputStep::ForwardPrompt | TransInputStep::InversePrompt => {
            // Forward/inverse prompts stay in their respective state (repeated use).
            // Actual computation is deferred to full Phase 31 wiring with the
            // op_trans2d_forward / op_trans3d_forward helpers.
            // For Phase 29 CLI: just acknowledge and clear prompt.
            state.modal_program = Some(ModalProgram::Trans(TransInputStep::Ready));
            state.modal_prompt = None;
            Ok(())
        }
        TransInputStep::Ready => Err(HpError::InvalidOp),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::{AngleMode, CalcState, DisplayMode};

    #[allow(dead_code)]
    const TOLERANCE: f64 = 1e-7;

    fn approx_eq(actual: f64, expected: f64, tol: f64) -> bool {
        (actual - expected).abs() < tol
    }

    fn get_xyz(state: &CalcState) -> (f64, f64, f64) {
        let x = state.stack.x.inner().to_f64().unwrap();
        let y = state.stack.y.inner().to_f64().unwrap();
        let z = state.stack.z.inner().to_f64().unwrap();
        (x, y, z)
    }

    fn set_stack_xyz(state: &mut CalcState, x: f64, y: f64, z: f64) {
        state.stack.x = f64_to_hpnum(x).unwrap();
        state.stack.y = f64_to_hpnum(y).unwrap();
        state.stack.z = f64_to_hpnum(z).unwrap();
    }

    // ── Trans2d Tests ─────────────────────────────────────────────────────────

    // Catches: Op::Trans2d master entry not setting modal_program correctly
    #[test]
    fn trans2d_master_opens_modal() {
        use crate::ops::math1::modal::TransInputStep;
        let mut state = CalcState::new();
        op_trans2d(&mut state).unwrap();
        assert_eq!(
            state.modal_program,
            Some(ModalProgram::Trans(TransInputStep::Init2dPrompt))
        );
        assert_eq!(state.modal_prompt, Some("X0,Y0,\u{03B8}?".to_string()));
    }

    // Catches: 2D forward rotation wrong (x₀=0, y₀=0, θ=90°, input (1,0) → (0,-1))
    // Verify rotation convention: right-hand rotation about origin rotates (1,0) to (0,1)
    // with the formula: x' = dx·cos(θ)+dy·sin(θ), y' = -dx·sin(θ)+dy·cos(θ)
    // At θ=90°: x' = 1·0 + 0·1 = 0, y' = -1·1 + 0·0 = -1 → output (0, -1)
    #[test]
    fn trans2d_forward_90deg_rotation() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        state.display_mode = DisplayMode::Fix(6);
        // Set params: origin=(0,0), θ=90°
        store_trans2d_params(&mut state, 0.0, 0.0, 90.0);
        // Set input: (x=1, y=0)
        set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
        do_trans2d_forward(&mut state).unwrap();
        let (x_prime, y_prime, _) = get_xyz(&state);
        assert!(
            approx_eq(x_prime, 0.0, 1e-6),
            "x' should be 0, got {x_prime}"
        );
        assert!(
            approx_eq(y_prime, -1.0, 1e-6),
            "y' should be -1, got {y_prime}"
        );
    }

    // Catches: 2D forward+inverse round-trip not recovering input
    #[test]
    fn trans2d_inverse_round_trip() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        store_trans2d_params(&mut state, 3.0, 5.0, 45.0);
        set_stack_xyz(&mut state, 7.0, 2.0, 0.0);

        // Forward transform
        do_trans2d_forward(&mut state).unwrap();
        let (x_prime, y_prime, _) = get_xyz(&state);

        // Inverse transform on result
        set_stack_xyz(&mut state, x_prime, y_prime, 0.0);
        do_trans2d_inverse(&mut state).unwrap();
        let (x_back, y_back, _) = get_xyz(&state);

        assert!(
            approx_eq(x_back, 7.0, 1e-6),
            "round-trip: x should recover 7.0, got {x_back}"
        );
        assert!(
            approx_eq(y_back, 2.0, 1e-6),
            "round-trip: y should recover 2.0, got {y_back}"
        );
    }

    // Catches: 2D origin translation wrong (θ=0, should just translate)
    // With x₀=5, y₀=5, θ=0 and input (5, 5) → forward should produce (0, 0)
    #[test]
    fn trans2d_origin_translation() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        store_trans2d_params(&mut state, 5.0, 5.0, 0.0);
        set_stack_xyz(&mut state, 5.0, 5.0, 0.0);
        do_trans2d_forward(&mut state).unwrap();
        let (x_prime, y_prime, _) = get_xyz(&state);
        assert!(
            approx_eq(x_prime, 0.0, 1e-6),
            "pure translation: x' should be 0, got {x_prime}"
        );
        assert!(
            approx_eq(y_prime, 0.0, 1e-6),
            "pure translation: y' should be 0, got {y_prime}"
        );
    }

    // Catches: 2D identity (θ=0, origin=0) not producing identity result
    #[test]
    fn trans2d_identity() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        store_trans2d_params(&mut state, 0.0, 0.0, 0.0);
        set_stack_xyz(&mut state, 3.0, 4.0, 0.0);
        do_trans2d_forward(&mut state).unwrap();
        let (x_prime, y_prime, _) = get_xyz(&state);
        assert!(
            approx_eq(x_prime, 3.0, 1e-6),
            "identity: x' = x = 3, got {x_prime}"
        );
        assert!(
            approx_eq(y_prime, 4.0, 1e-6),
            "identity: y' = y = 4, got {y_prime}"
        );
    }

    // Catches: 2D Rad angle mode not converted properly
    #[test]
    fn trans2d_radians_mode() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Rad;
        // θ = π/2 in radians = 90°
        store_trans2d_params(&mut state, 0.0, 0.0, PI / 2.0);
        set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
        do_trans2d_forward(&mut state).unwrap();
        let (x_prime, y_prime, _) = get_xyz(&state);
        assert!(
            approx_eq(x_prime, 0.0, 1e-6),
            "x' should be 0 (π/2 rad rotation), got {x_prime}"
        );
        assert!(
            approx_eq(y_prime, -1.0, 1e-6),
            "y' should be -1, got {y_prime}"
        );
    }

    // ── Trans3d Tests ─────────────────────────────────────────────────────────

    // Catches: Op::Trans3d master entry not setting modal_program correctly
    #[test]
    fn trans3d_master_opens_modal() {
        use crate::ops::math1::modal::TransInputStep;
        let mut state = CalcState::new();
        op_trans3d(&mut state).unwrap();
        assert_eq!(
            state.modal_program,
            Some(ModalProgram::Trans(TransInputStep::Init3dOriginPrompt))
        );
        assert_eq!(state.modal_prompt, Some("ORIGIN?".to_string()));
    }

    // Catches: 3D Rodrigues forward rotation wrong (z-axis 90° rotation)
    // Origin=(0,0,0), axis=(0,0,1), θ=90°, input=(1,0,0) → (0,1,0)
    // Source: Rodrigues' formula — rotation about z-axis maps (1,0,0) to (0,1,0).
    // Note: our forward transform does NOT subtract origin from output (world→local convention);
    // with origin=(0,0,0) the result is purely the rotated vector.
    #[test]
    fn trans3d_rodrigues_forward_z_axis() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        // Origin at (0,0,0), axis = z-axis (0,0,1), θ = 90°
        store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 90.0);
        set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
        do_trans3d_forward(&mut state).unwrap();
        let (x_rot, y_rot, z_rot) = get_xyz(&state);
        // Rodrigues around z-axis by 90°: (1,0,0) → (0,1,0)
        assert!(
            approx_eq(x_rot, 0.0, 1e-6),
            "x' should be 0 (z-axis 90° rotation), got {x_rot}"
        );
        assert!(approx_eq(y_rot, 1.0, 1e-6), "y' should be 1, got {y_rot}");
        assert!(approx_eq(z_rot, 0.0, 1e-6), "z' should be 0, got {z_rot}");
    }

    // Catches: 3D forward+inverse round-trip not recovering input
    // Convention: forward maps world→local (subtracts origin, rotates).
    // Inverse maps local→world (inverse-rotates, adds origin).
    // Round-trip: forward(world) → local; inverse(local) → world.
    // Note: the inverse function receives the local (rotated) coordinates directly
    // (the same output from forward), not world coordinates.
    #[test]
    fn trans3d_rodrigues_inverse_round_trip() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        // Use simple origin and axis for predictable round-trip
        store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 45.0);

        let input_x = 3.0;
        let input_y = 4.0;
        let input_z = 5.0;
        set_stack_xyz(&mut state, input_x, input_y, input_z);

        // Forward transform: world → local
        do_trans3d_forward(&mut state).unwrap();
        let (xr, yr, zr) = get_xyz(&state);

        // Inverse transform: local → world (receives forward output directly)
        set_stack_xyz(&mut state, xr, yr, zr);
        do_trans3d_inverse(&mut state).unwrap();
        let (x_back, y_back, z_back) = get_xyz(&state);

        assert!(
            approx_eq(x_back, input_x, 1e-5),
            "round-trip x should recover {input_x}, got {x_back}"
        );
        assert!(
            approx_eq(y_back, input_y, 1e-5),
            "round-trip y should recover {input_y}, got {y_back}"
        );
        assert!(
            approx_eq(z_back, input_z, 1e-5),
            "round-trip z should recover {input_z}, got {z_back}"
        );
    }

    // Catches: 3D non-unit axis not normalized correctly
    // Axis = (2,0,0) (non-unit, length 2) — should normalize to (1,0,0) = x-axis.
    // Rotation of (0,1,0) around x-axis by 90° → (0,0,1).
    #[test]
    fn trans3d_axis_normalization() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (2.0, 0.0, 0.0), 90.0);
        set_stack_xyz(&mut state, 0.0, 1.0, 0.0);
        do_trans3d_forward(&mut state).unwrap();
        let (x_rot, y_rot, z_rot) = get_xyz(&state);
        // x-axis 90°: (0,1,0) → (0,0,1)
        assert!(
            approx_eq(x_rot, 0.0, 1e-6),
            "x should be 0 for x-axis rotation, got {x_rot}"
        );
        assert!(approx_eq(y_rot, 0.0, 1e-6), "y should be 0, got {y_rot}");
        assert!(
            approx_eq(z_rot, 1.0, 1e-6),
            "z should be 1 (x-axis 90°), got {z_rot}"
        );
    }

    // Catches: zero-length axis not returning Domain error
    #[test]
    fn trans3d_zero_axis_error() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 90.0);
        set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
        let result = do_trans3d_forward(&mut state);
        assert!(
            result.is_err(),
            "zero-length axis should return Domain error"
        );
    }

    // Catches: 3D rotation about arbitrary axis (1,1,1) wrong
    // Axis = (1,1,1) normalized = (1/√3, 1/√3, 1/√3), θ = 2π/3 = 120°.
    // By Rodrigues identity: rotation by 120° around (1,1,1)/√3 cycles (1,0,0) → (0,1,0).
    // Source: Rodrigues' formula — 2π/3 rotation around the (1,1,1) axis is a cyclic permutation.
    #[test]
    fn trans3d_arbitrary_axis_cyclic_permutation() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        // Axis = (1,1,1), θ = 120° (= 2π/3)
        store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0), 120.0);
        set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
        do_trans3d_forward(&mut state).unwrap();
        let (x_rot, y_rot, z_rot) = get_xyz(&state);
        // (1,0,0) → (0,1,0) under 120° rotation about (1,1,1)/√3
        assert!(
            approx_eq(x_rot, 0.0, 1e-5),
            "x cyclic: (1,0,0) → 0 under (1,1,1) 120°, got {x_rot}"
        );
        assert!(
            approx_eq(y_rot, 1.0, 1e-5),
            "y cyclic: (1,0,0) → 1, got {y_rot}"
        );
        assert!(
            approx_eq(z_rot, 0.0, 1e-5),
            "z cyclic: (1,0,0) → 0, got {z_rot}"
        );
    }

    // Catches: scratch register layout wrong (TRANS-05)
    // After store_trans3d_params: R00=x₀, R01=y₀, R02=z₀, R03=a, R04=b, R05=c, R06=θ
    #[test]
    fn scratch_registers_layout() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        store_trans3d_params(&mut state, (1.0, 2.0, 3.0), (4.0, 5.0, 6.0), 45.0);
        let r = |i: usize| state.regs[i].inner().to_f64().unwrap();
        // Origin in R00/R01/R02
        assert!(approx_eq(r(0), 1.0, 1e-10), "R00=x₀=1");
        assert!(approx_eq(r(1), 2.0, 1e-10), "R01=y₀=2");
        assert!(approx_eq(r(2), 3.0, 1e-10), "R02=z₀=3");
        // Axis in R03/R04/R05
        assert!(approx_eq(r(3), 4.0, 1e-10), "R03=a=4");
        assert!(approx_eq(r(4), 5.0, 1e-10), "R04=b=5");
        assert!(approx_eq(r(5), 6.0, 1e-10), "R05=c=6");
        // Angle in R06
        assert!(approx_eq(r(6), 45.0, 1e-10), "R06=θ=45°");
    }

    // Catches: 3D zero rotation (θ=0) not returning identity
    #[test]
    fn trans3d_zero_rotation_identity() {
        let mut state = CalcState::new();
        state.angle_mode = AngleMode::Deg;
        store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 0.0);
        set_stack_xyz(&mut state, 3.0, 4.0, 5.0);
        do_trans3d_forward(&mut state).unwrap();
        let (x_rot, y_rot, z_rot) = get_xyz(&state);
        assert!(
            approx_eq(x_rot, 3.0, 1e-6),
            "θ=0 identity: x unchanged, got {x_rot}"
        );
        assert!(
            approx_eq(y_rot, 4.0, 1e-6),
            "θ=0 identity: y unchanged, got {y_rot}"
        );
        assert!(
            approx_eq(z_rot, 5.0, 1e-6),
            "θ=0 identity: z unchanged, got {z_rot}"
        );
    }
}
