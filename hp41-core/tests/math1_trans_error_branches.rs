// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Plan 32-05 surgical gap-closure for `hp41-core/src/ops/math1/trans.rs`.
//!
//! Closes the 81.17 % per-file coverage gap by exercising:
//! - The Rodrigues zero-axis guard (`normalize_3d` line 117 -> `Err(HpError::Domain)`)
//! - The three `submit_step` register-count guards (lines 469, 484, 503 -> `Err(HpError::InvalidOp)`)
//! - The `submit_step Ready` arm (line 569 -> `Err(HpError::InvalidOp)`)
//! - The `submit_step ForwardPrompt` and `InversePrompt` arms exercising untested
//!   `do_trans2d_inverse` and `do_trans3d_inverse` paths surfaced by Task 1 reconnaissance.
//! - The `submit_step Init2dPrompt` and `Init3dAxisPrompt` success paths (previously
//!   exercised only in CLI integration tests, not in unit test coverage).
//!
//! All tests carry `// Catches:` doc comments per D-27.1.

#![allow(clippy::unwrap_used)]

use approx::assert_relative_eq;
use hp41_core::error::HpError;
use hp41_core::ops::math1::modal::{ModalProgram, TransInputStep};
use hp41_core::ops::math1::trans::{
    do_trans2d_forward, do_trans2d_inverse, do_trans3d_forward, do_trans3d_inverse,
    store_trans2d_params, store_trans3d_params, submit_step as trans_submit_step,
};
use hp41_core::state::CalcState;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
fn f64_hpnum(v: f64) -> hp41_core::HpNum {
    hp41_core::HpNum::rounded(Decimal::from_f64(v).unwrap())
}

fn make_state() -> CalcState {
    CalcState::new()
}

fn set_stack_xyz(state: &mut CalcState, x: f64, y: f64, z: f64) {
    state.stack.x = f64_hpnum(x);
    state.stack.y = f64_hpnum(y);
    state.stack.z = f64_hpnum(z);
}

fn set_stack_xyzt(state: &mut CalcState, x: f64, y: f64, z: f64, t: f64) {
    state.stack.x = f64_hpnum(x);
    state.stack.y = f64_hpnum(y);
    state.stack.z = f64_hpnum(z);
    state.stack.t = f64_hpnum(t);
}

fn get_xyz(state: &CalcState) -> (f64, f64, f64) {
    let x = state.stack.x.inner().to_f64().unwrap();
    let y = state.stack.y.inner().to_f64().unwrap();
    let z = state.stack.z.inner().to_f64().unwrap();
    (x, y, z)
}

// ── Rodrigues zero-axis guard ────────────────────────────────────────────────

// Catches: zero-length axis silently producing garbage rotation instead of Domain error
#[test]
fn rodrigues_zero_axis_returns_domain() {
    // Source: HP Math Pac I OM (HP 00041-90034, 1979), TRANS program — axis must be non-zero.
    // Setup: 3D state with axis = (0, 0, 0) (all-zero), θ = 90°.
    // Expected: do_trans3d_forward propagates Err(HpError::Domain) from normalize_3d (line 117).
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Deg;
    store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 90.0);
    set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
    let result = do_trans3d_forward(&mut state);
    assert!(
        matches!(result, Err(HpError::Domain)),
        "zero-length axis must return Err(HpError::Domain), got {result:?}"
    );
}

// Catches: near-zero axis (below 1e-14 threshold) not caught by the guard
#[test]
fn rodrigues_near_zero_axis_returns_domain() {
    // Source: normalize_3d uses threshold 1e-14 to guard floating-point underflow.
    // A vector with length 1e-20 (far below threshold) must also return Domain.
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Deg;
    // Axis components each 1e-21 → length ≈ sqrt(3) × 1e-21 << 1e-14
    store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (1e-21, 1e-21, 1e-21), 45.0);
    set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
    let result = do_trans3d_forward(&mut state);
    assert!(
        matches!(result, Err(HpError::Domain)),
        "near-zero axis below 1e-14 threshold must return Err(HpError::Domain), got {result:?}"
    );
}

// ── submit_step register-count guard arms ────────────────────────────────────

// Catches: Init2dPrompt arm (line 469) not guarding against insufficient registers
#[test]
fn submit_step_arm_469_insufficient_regs_returns_invalid_op() {
    // Source: submit_step Init2dPrompt requires regs.len() >= 3 (lines 468-469).
    // If state.regs has fewer than 3 entries, returns Err(HpError::InvalidOp).
    let mut state = make_state();
    state.regs.truncate(2); // force regs.len() == 2 < 3
    let result = trans_submit_step(&mut state, TransInputStep::Init2dPrompt);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "Init2dPrompt with regs.len()<3 must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: Init3dOriginPrompt arm (line 484) not guarding against insufficient registers
#[test]
fn submit_step_arm_484_insufficient_regs_returns_invalid_op() {
    // Source: submit_step Init3dOriginPrompt requires regs.len() >= 4 (lines 483-484).
    let mut state = make_state();
    state.regs.truncate(3); // force regs.len() == 3 < 4
    let result = trans_submit_step(&mut state, TransInputStep::Init3dOriginPrompt);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "Init3dOriginPrompt with regs.len()<4 must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: Init3dAxisPrompt arm (line 503) not guarding against insufficient registers
#[test]
fn submit_step_arm_503_insufficient_regs_returns_invalid_op() {
    // Source: submit_step Init3dAxisPrompt requires regs.len() >= 7 (lines 502-503).
    let mut state = make_state();
    state.regs.truncate(6); // force regs.len() == 6 < 7
    let result = trans_submit_step(&mut state, TransInputStep::Init3dAxisPrompt);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "Init3dAxisPrompt with regs.len()<7 must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// Catches: TransInputStep::Ready arm (line 569) not returning InvalidOp
#[test]
fn submit_step_ready_returns_invalid_op() {
    // Source: submit_step Ready arm returns Err(HpError::InvalidOp) unconditionally (line 569).
    // This guards against the CLI/GUI submitting a value when no prompt is active.
    let mut state = make_state();
    let result = trans_submit_step(&mut state, TransInputStep::Ready);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "TransInputStep::Ready must return Err(HpError::InvalidOp), got {result:?}"
    );
}

// ── submit_step success paths (untested function entry points) ────────────────

// Catches: Init2dPrompt success path not storing params in correct register layout
#[test]
fn submit_step_init2d_success_stores_params() {
    // Source: HP Math Pac I OM TRANS-01 A-entry: x₀/y₀/θ entered as Z/Y/X on stack.
    // submit_step Init2dPrompt reads X=θ, Y=y₀, Z=x₀ and stores R00=x₀, R01=y₀, R02=θ.
    // Advances modal to ForwardPrompt.
    let mut state = make_state();
    // Stack: X=45.0 (θ), Y=5.0 (y₀), Z=3.0 (x₀)
    set_stack_xyz(&mut state, 45.0, 5.0, 3.0);
    let result = trans_submit_step(&mut state, TransInputStep::Init2dPrompt);
    assert!(result.is_ok(), "Init2dPrompt success path must return Ok, got {result:?}");
    // Verify register layout: R00=3.0 (x₀), R01=5.0 (y₀), R02=45.0 (θ)
    let r00 = state.regs[0].inner().to_f64().unwrap();
    let r01 = state.regs[1].inner().to_f64().unwrap();
    let r02 = state.regs[2].inner().to_f64().unwrap();
    assert_relative_eq!(r00, 3.0, max_relative = 1e-7);
    assert_relative_eq!(r01, 5.0, max_relative = 1e-7);
    assert_relative_eq!(r02, 45.0, max_relative = 1e-7);
    // Modal advances to ForwardPrompt
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Trans(TransInputStep::ForwardPrompt))
    );
}

// Catches: Init3dOriginPrompt success path not storing origin in correct registers
#[test]
fn submit_step_init3d_origin_success_stores_origin() {
    // Source: HP Math Pac I OM TRANS-03 A-entry: origin (x₀, y₀, z₀) entered as T/Y/X.
    // submit_step Init3dOriginPrompt reads X=z₀, Y=y₀, Z=x₀ and stores R00=x₀, R01=y₀, R02=z₀.
    // Advances modal to Init3dAxisPrompt.
    let mut state = make_state();
    // Stack: X=3.0 (z₀), Y=2.0 (y₀), Z=1.0 (x₀)
    set_stack_xyz(&mut state, 3.0, 2.0, 1.0);
    let result = trans_submit_step(&mut state, TransInputStep::Init3dOriginPrompt);
    assert!(result.is_ok(), "Init3dOriginPrompt success path must return Ok, got {result:?}");
    let r00 = state.regs[0].inner().to_f64().unwrap();
    let r01 = state.regs[1].inner().to_f64().unwrap();
    let r02 = state.regs[2].inner().to_f64().unwrap();
    assert_relative_eq!(r00, 1.0, max_relative = 1e-7);
    assert_relative_eq!(r01, 2.0, max_relative = 1e-7);
    assert_relative_eq!(r02, 3.0, max_relative = 1e-7);
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Trans(TransInputStep::Init3dAxisPrompt))
    );
}

// Catches: Init3dAxisPrompt success path not storing axis+angle in correct registers
#[test]
fn submit_step_init3d_axis_success_stores_axis_and_theta() {
    // Source: HP Math Pac I OM TRANS-03 B-entry: axis (a,b,c) + θ entered as T/Z/Y/X.
    // submit_step Init3dAxisPrompt reads X=θ, Y=c, Z=b, T=a and stores R03=a, R04=b, R05=c, R06=θ.
    // Advances modal to ForwardPrompt.
    let mut state = make_state();
    // Stack: X=90.0 (θ), Y=1.0 (c), Z=0.0 (b), T=0.0 (a)
    set_stack_xyzt(&mut state, 90.0, 1.0, 0.0, 0.0);
    let result = trans_submit_step(&mut state, TransInputStep::Init3dAxisPrompt);
    assert!(result.is_ok(), "Init3dAxisPrompt success path must return Ok, got {result:?}");
    let r03 = state.regs[3].inner().to_f64().unwrap(); // a
    let r04 = state.regs[4].inner().to_f64().unwrap(); // b
    let r05 = state.regs[5].inner().to_f64().unwrap(); // c
    let r06 = state.regs[6].inner().to_f64().unwrap(); // θ
    assert_relative_eq!(r03, 0.0, max_relative = 1e-7);
    assert_relative_eq!(r04, 0.0, max_relative = 1e-7);
    assert_relative_eq!(r05, 1.0, max_relative = 1e-7);
    assert_relative_eq!(r06, 90.0, max_relative = 1e-7);
    assert_eq!(
        state.modal_program,
        Some(ModalProgram::Trans(TransInputStep::ForwardPrompt))
    );
}

// ── submit_step ForwardPrompt and InversePrompt (2D/3D dispatch heuristic) ───

// Catches: submit_step ForwardPrompt 2D dispatch path not running the actual transform
#[test]
fn submit_step_forward_prompt_2d_runs_transform() {
    // Source: HP Math Pac I OM TRANS-02 C-entry: forward 2D transform.
    // Setup: 2D params (x₀=0, y₀=0, θ=90°). Input: X=1.0, Y=0.0.
    // ForwardPrompt heuristic: R06 == 0 && R03 == 0 → 2D path.
    // Expected: X=0.0, Y=-1.0 after 90° rotation of (1,0).
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Deg;
    store_trans2d_params(&mut state, 0.0, 0.0, 90.0);
    set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
    let result = trans_submit_step(&mut state, TransInputStep::ForwardPrompt);
    assert!(result.is_ok(), "ForwardPrompt 2D must return Ok, got {result:?}");
    let (x_prime, y_prime, _) = get_xyz(&state);
    assert_relative_eq!(x_prime, 0.0, max_relative = 1e-6, epsilon = 1e-9);
    assert_relative_eq!(y_prime, -1.0, max_relative = 1e-6);
    // modal_prompt stays as "FWD?" (re-set by the arm for repeated use)
    assert_eq!(state.modal_prompt, Some("FWD?".to_string()));
}

// Catches: submit_step InversePrompt 2D dispatch path not running the actual inverse transform
#[test]
fn submit_step_inverse_prompt_2d_runs_transform() {
    // Source: HP Math Pac I OM TRANS-02 E-entry: inverse 2D transform.
    // Setup: 2D params (x₀=0, y₀=0, θ=90°). Input: X=0.0, Y=-1.0 (output of forward).
    // InversePrompt heuristic: R06 == 0 → 2D path.
    // Expected: X=1.0, Y=0.0 after inverse rotation (round-trip).
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Deg;
    store_trans2d_params(&mut state, 0.0, 0.0, 90.0);
    set_stack_xyz(&mut state, 0.0, -1.0, 0.0);
    let result = trans_submit_step(&mut state, TransInputStep::InversePrompt);
    assert!(result.is_ok(), "InversePrompt 2D must return Ok, got {result:?}");
    let (x_back, y_back, _) = get_xyz(&state);
    assert_relative_eq!(x_back, 1.0, max_relative = 1e-6);
    assert_relative_eq!(y_back, 0.0, max_relative = 1e-6, epsilon = 1e-9);
    // modal_prompt stays as "INV?" (re-set by the arm for repeated use)
    assert_eq!(state.modal_prompt, Some("INV?".to_string()));
}

// Catches: submit_step ForwardPrompt 3D dispatch path not running the 3D transform
#[test]
fn submit_step_forward_prompt_3d_runs_rodrigues() {
    // Source: HP Math Pac I OM TRANS-04 C-entry: forward 3D transform.
    // Setup: 3D params (origin=(0,0,0), axis=(0,0,1), θ=90°). Input: X=1.0, Y=0.0, Z=0.0.
    // ForwardPrompt heuristic: R06 != 0 (θ=90) → 3D path.
    // Expected: Rodrigues rotation about z-axis by 90°: (1,0,0) → (0,1,0).
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Deg;
    store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 90.0);
    set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
    let result = trans_submit_step(&mut state, TransInputStep::ForwardPrompt);
    assert!(result.is_ok(), "ForwardPrompt 3D must return Ok, got {result:?}");
    let (x_rot, y_rot, z_rot) = get_xyz(&state);
    assert_relative_eq!(x_rot, 0.0, max_relative = 1e-6, epsilon = 1e-9);
    assert_relative_eq!(y_rot, 1.0, max_relative = 1e-6);
    assert_relative_eq!(z_rot, 0.0, max_relative = 1e-6, epsilon = 1e-9);
}

// Catches: do_trans3d_inverse round-trip not recovering original coordinates
#[test]
fn do_trans3d_inverse_round_trip_with_origin() {
    // Source: HP Math Pac I OM TRANS-04 E-entry: inverse 3D transform.
    // Tests do_trans3d_inverse directly (previously uncovered by unit tests).
    // Setup: origin=(1,2,3), axis=(0,0,1), θ=90°.
    // Forward then inverse must recover original input.
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Deg;
    store_trans3d_params(&mut state, (1.0, 2.0, 3.0), (0.0, 0.0, 1.0), 90.0);

    // Forward: input (4, 6, 5) → local rotated coords
    set_stack_xyz(&mut state, 4.0, 6.0, 5.0);
    do_trans3d_forward(&mut state).unwrap();
    let (xf, yf, zf) = get_xyz(&state);

    // Inverse: rotated coords → should recover (4, 6, 5)
    set_stack_xyz(&mut state, xf, yf, zf);
    do_trans3d_inverse(&mut state).unwrap();
    let (x_back, y_back, z_back) = get_xyz(&state);

    assert_relative_eq!(x_back, 4.0, max_relative = 1e-5);
    assert_relative_eq!(y_back, 6.0, max_relative = 1e-5);
    assert_relative_eq!(z_back, 5.0, max_relative = 1e-5);
}

// Catches: do_trans2d_inverse not recovering input for non-zero origin
#[test]
fn do_trans2d_inverse_with_nonzero_origin() {
    // Source: HP Math Pac I OM TRANS-02 E-entry: inverse 2D transform.
    // Tests do_trans2d_inverse with a non-trivial origin and rotation.
    // Setup: origin=(3,5), θ=45°. Input: (7, 2). Forward then inverse must recover (7,2).
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Deg;
    store_trans2d_params(&mut state, 3.0, 5.0, 45.0);

    set_stack_xyz(&mut state, 7.0, 2.0, 0.0);
    do_trans2d_forward(&mut state).unwrap();
    let (xf, yf, _) = get_xyz(&state);

    set_stack_xyz(&mut state, xf, yf, 0.0);
    do_trans2d_inverse(&mut state).unwrap();
    let (x_back, y_back, _) = get_xyz(&state);

    assert_relative_eq!(x_back, 7.0, max_relative = 1e-6);
    assert_relative_eq!(y_back, 2.0, max_relative = 1e-6);
}

// Catches: GRAD angle mode not converted properly in 3D transform
#[test]
fn trans3d_grad_angle_mode() {
    // Source: to_radians in trans.rs converts GRAD: angle * (PI / 200.0).
    // 100 gradians = 90 degrees = PI/2 radians.
    // Rotation about z-axis by 100g of (1,0,0) must produce ≈ (0,1,0).
    let mut state = make_state();
    state.angle_mode = hp41_core::state::AngleMode::Grad;
    store_trans3d_params(&mut state, (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 100.0);
    set_stack_xyz(&mut state, 1.0, 0.0, 0.0);
    do_trans3d_forward(&mut state).unwrap();
    let (x_rot, y_rot, z_rot) = get_xyz(&state);
    assert_relative_eq!(x_rot, 0.0, max_relative = 1e-6, epsilon = 1e-9);
    assert_relative_eq!(y_rot, 1.0, max_relative = 1e-6);
    assert_relative_eq!(z_rot, 0.0, max_relative = 1e-6, epsilon = 1e-9);
}
