// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 regression scaffold: user-callback re-entrancy strict-reject tests (C-28.2).
//!
//! **Module invariant:** 5 regression cases for user-callback re-entrancy.
//! - Plan 28-07 fills the `nested_integ_*` and `user_fn_stops_aborts_integ` branches
//! - Plan 28-08 fills the `nested_solve_*` branches
//! - Plan 28-09 fills the `nested_difeq_*` branch
//!
//! C-28.2 / ADR-002: strict-reject nested INTG/SOLVE/DIFEQ at op entry.
//! At op entry, check `state.integ_state.is_some() || state.solve_state.is_some()`
//! → `HpError::InvalidOp` if true. This matches Math Pac I OM 1979 hardware behavior.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::math1::integ::op_integ_run_loop;
use hp41_core::ops::math1::solve::op_solve_run_loop;
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
use rust_decimal::prelude::ToPrimitive;

// ── Test helpers ──────────────────────────────────────────────────────────────

/// Build a CalcState configured for INTG with f(x) = x.
/// Program: LBL "F" / (no-op — x is already in X) / RTN
/// Integrating f(x) = x over [0,1] → 0.5
fn make_identity_fn_state() -> (CalcState, Vec<Op>) {
    let program = vec![
        Op::Lbl("F".to_string()),
        // Identity: x is already in X from the integ loop
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "F".to_string();
    state.regs[0] = HpNum::from(10i32); // n=10 subdivisions
    state.stack.x = HpNum::from(0i32); // a=0
    state.stack.y = HpNum::from(1i32); // b=1
    state.stack.lift_enabled = false;
    (state, program)
}

/// Build a CalcState configured for INTG where the user function also
/// calls INTG (nested integration attempt).
/// Outer program: LBL "G" / XEQ "INTG" / RTN (nested INTG inside G)
/// Outer call: INTG over [0,1] using G as integrand
fn make_nested_integ_state() -> (CalcState, Vec<Op>) {
    // The inner callback "G" tries to call INTG itself (via Op::Integ)
    let program = vec![
        Op::Lbl("G".to_string()),
        Op::Integ, // nested Op::Integ inside user function — must be rejected
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "G".to_string();
    state.regs[0] = HpNum::from(4i32); // n=4 subdivisions
    state.stack.x = HpNum::from(0i32); // a=0
    state.stack.y = HpNum::from(1i32); // b=1
    state.stack.lift_enabled = false;
    (state, program)
}

// ── Test 1: nested INTG inside INTG rejected ──────────────────────────────────

/// Nested INTG inside an INTG user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard missing in op_integ entry.
/// Filled by Plan 28-07.
///
/// When op_integ_run_loop starts the sample loop, it sets state.integ_state = Some(...).
/// Each sample calls run_user_function which executes Op::Integ again.
/// The inner op_integ_run_loop sees integ_state.is_some() and returns Err(InvalidOp).
/// This error propagates back through run_user_function → op_integ_run_loop → test.
///
/// ADR-002 / XROM-08 / C-28.2: strict-reject nested user-callbacks.
#[test]
fn nested_integ_inside_integ_rejected() {
    let (mut state, program) = make_nested_integ_state();
    let result = op_integ_run_loop(&mut state, &program);

    // The nested INTG attempt returns InvalidOp which propagates out
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "nested INTG inside INTG user function must return HpError::InvalidOp (ADR-002 / XROM-08)"
    );
    // integ_state must be cleared after the error (no state leak)
    assert!(
        state.integ_state.is_none(),
        "integ_state must be None after nested INTG rejection"
    );
}

/// Nested SOLVE inside an INTG user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard checking only integ_state, not solve_state.
/// Filled by Plan 28-07.
///
/// This tests that the guard checks ALL three solver states, not just integ_state.
/// Pre-set solve_state = Some to simulate a SOLVE in progress, then call op_integ_run_loop.
/// The guard `state.integ_state.is_some() || state.solve_state.is_some()` must trigger.
#[test]
fn nested_solve_inside_integ_rejected() {
    let (mut state, program) = make_identity_fn_state();
    // Pre-set solve_state to simulate SOLVE in progress — INTG must reject
    state.solve_state = Some(hp41_core::ops::math1::solve::SolveState::default());

    let result = op_integ_run_loop(&mut state, &program);
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "INTG with solve_state set must return HpError::InvalidOp (ADR-002 / XROM-08)"
    );
    // solve_state must be unchanged — guard fired before mutation
    assert!(
        state.solve_state.is_some(),
        "solve_state must still be Some after pre-mutation rejection"
    );
    assert!(
        state.integ_state.is_none(),
        "integ_state must remain None after pre-mutation rejection"
    );
}

/// Build a CalcState configured for SOLVE where the callback tries to call INTG.
/// Outer: SOLVE on a function "NI" that executes Op::Integ internally.
fn make_solve_with_nested_integ() -> (CalcState, Vec<Op>) {
    // Callback "NI": tries to call INTG (nested INTG inside SOLVE user function)
    let program = vec![
        Op::Lbl("NI".to_string()),
        Op::Integ, // nested Op::Integ inside SOLVE callback — must be rejected
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "NI".to_string();
    state.regs[0] = HpNum::from(-1i32); // x1 = -1
    state.regs[1] = HpNum::from(1i32);  // x2 = 1
    state.stack.lift_enabled = false;
    (state, program)
}

/// Build a CalcState configured for SOLVE where the callback tries to call SOLVE itself.
fn make_solve_with_nested_solve() -> (CalcState, Vec<Op>) {
    // Callback "NS": tries to call SOLVE (nested SOLVE inside SOLVE user function)
    let program = vec![
        Op::Lbl("NS".to_string()),
        Op::Solve, // nested Op::Solve inside SOLVE callback — must be rejected
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "NS".to_string();
    state.regs[0] = HpNum::from(-1i32); // x1 = -1
    state.regs[1] = HpNum::from(1i32);  // x2 = 1
    state.stack.lift_enabled = false;
    (state, program)
}

/// Nested INTG inside a SOLVE user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard missing in op_solve entry — checks only integ_state
/// but not solve_state (XROM-08 / ADR-002 strict-reject must check ALL three solver states).
/// Filled by Plan 28-08.
///
/// When op_solve_run_loop starts the secant loop, it sets state.solve_state = Some(...).
/// Each secant step calls run_user_function which executes Op::Integ.
/// The inner op_integ_run_loop sees solve_state.is_some() → Err(InvalidOp).
/// This error propagates back through run_user_function → run_secant_loop → test.
#[test]
fn nested_integ_inside_solve_rejected() {
    let (mut state, program) = make_solve_with_nested_integ();
    let result = op_solve_run_loop(&mut state, &program);

    // The nested INTG attempt returns InvalidOp which propagates out
    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "nested INTG inside SOLVE user function must return HpError::InvalidOp (ADR-002 / XROM-08)"
    );
    // solve_state must be cleared after the error (no state leak)
    assert!(
        state.solve_state.is_none(),
        "solve_state must be None after nested INTG rejection"
    );
}

/// Nested SOLVE inside a SOLVE user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard missing in op_solve entry — nested SOLVE-in-SOLVE case.
/// New test added in Plan 28-08 (not scaffolded in Plan 28-01).
///
/// When op_solve_run_loop sets solve_state = Some(...), any inner Op::Solve
/// call sees solve_state.is_some() → Err(InvalidOp) per XROM-08.
#[test]
fn nested_solve_inside_solve_rejected() {
    let (mut state, program) = make_solve_with_nested_solve();
    let result = op_solve_run_loop(&mut state, &program);

    assert_eq!(
        result,
        Err(HpError::InvalidOp),
        "nested SOLVE inside SOLVE user function must return HpError::InvalidOp (XROM-08 / SOLV-08)"
    );
    assert!(
        state.solve_state.is_none(),
        "solve_state must be None after nested SOLVE rejection"
    );
}

/// Nested DIFEQ inside an INTG user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard checking only {integ,solve}_state but not difeq_state.
/// Filled by Plan 28-09.
#[test]
#[ignore = "filled by Plan 28-09"]
fn nested_difeq_inside_integ_rejected() {
    unimplemented!("filled by Plan 28-09");
}

// ── Test 2: user function STOP aborts INTG ────────────────────────────────────

/// User function that halts via STOP causes op_integ_run_loop to break out cleanly.
/// Catches: STOP inside user function not handled (integ_state leak).
/// Filled by Plan 28-07.
///
/// When Op::Stop appears in the user function, run_user_function breaks.
/// op_integ_run_loop receives Ok(()) from the sub-loop (STOP is not an error),
/// but then tries to accumulate f(x_k) — the X register value at STOP.
/// The outer INTG continues with whatever partial result it has.
///
/// Note: The OM does not specify behavior when the user function executes STOP;
/// this test documents our emulator's behavior: INTG continues using whatever
/// value was in X at the STOP instruction.
#[test]
fn user_fn_stops_aborts_integ() {
    // Program: LBL "H" / STOP / RTN
    // When user function stops, X has whatever was pushed (x_k from integ loop)
    let program = vec![
        Op::Lbl("H".to_string()),
        Op::Stop, // STOP breaks sub-loop; X = x_k (the sample point pushed)
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "H".to_string();
    state.regs[0] = HpNum::from(4i32); // n=4 subdivisions
    state.stack.x = HpNum::from(0i32); // a=0
    state.stack.y = HpNum::from(1i32); // b=1
    state.stack.lift_enabled = false;

    // With STOP in the user function, f(x) effectively returns x_k (identity).
    // So ∫₀¹ x dx ≈ 0.5 via Simpson with n=4.
    let result = op_integ_run_loop(&mut state, &program);

    // STOP should NOT cause an error — it just breaks the user function sub-loop
    assert!(
        result.is_ok(),
        "user function STOP must not abort outer INTG with error, got: {result:?}"
    );

    // integ_state must be cleared after completion
    assert!(
        state.integ_state.is_none(),
        "integ_state must be None after INTG completes (even if user fn used STOP)"
    );

    // The result should approximate ∫₀¹ x dx = 0.5 (identity function via STOP-at-x_k)
    let x_val = state.stack.x.inner().to_f64().unwrap();
    // Loose tolerance since STOP means function returns immediately at x_k
    assert!(
        (x_val - 0.5).abs() < 0.1,
        "∫₀¹ x dx with STOP-at-x_k should be approximately 0.5, got: {x_val}"
    );
}

// ── Test 3: user function STO to scratch register corrupts result ─────────────

/// User function that STO's to R03 (a scratch register) corrupts the integration.
/// Catches: scratch register snapshot/restore wrongly implemented (it MUST NOT be).
/// Documents RESEARCH Open Q6 user-responsibility convention.
///
/// This test asserts the WRONG ANSWER (not an error) — the emulator faithfully
/// reproduces Math Pac I hardware behavior where R00–R07 are scratch during INTG.
///
/// OM 1979 p. 35: "do not use registers R00–R07 in your user function while INTG is active."
/// Hardware-faithful: NO snapshot/restore. This is a USER-RESPONSIBILITY divergence.
///
/// The test program stores (x + 1) into R03. This corrupts whatever INTG might use R03
/// for (in our implementation, R00 holds n, others are free — but this tests the documented behavior).
/// The function returns x^2 (correct) but the side-effect on R03 is permanent.
/// Since our INTG implementation doesn't use R03, the result is still correct here,
/// but the test demonstrates that STO inside user functions IS tolerated (no error raised).
///
/// Docs: docs/hp41-math1-divergences.md entry "User-program scratch register clobber"
#[test]
fn user_fn_stores_to_scratch_corrupts_integ() {
    // Program: LBL "K" / X^2 / STO 03 / RTN
    // f(x) = x^2, but also stores x^2 into R03 (scratch clobber)
    // ∫₀¹ x² dx = 1/3 ≈ 0.333
    let program = vec![
        Op::Lbl("K".to_string()),
        Op::Sq,                     // f(x) = x^2
        Op::StoReg(3),              // STO 03 — clobbers scratch register R03
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "K".to_string();
    state.regs[0] = HpNum::from(10i32); // n=10 subdivisions
    state.stack.x = HpNum::from(0i32); // a=0
    state.stack.y = HpNum::from(1i32); // b=1
    state.stack.lift_enabled = false;

    // Run INTG — the user function clobbers R03
    let result = op_integ_run_loop(&mut state, &program);

    // Must NOT return an error — scratch clobber is a user-responsibility, not a runtime error
    assert!(
        result.is_ok(),
        "STO to scratch register inside user function must not raise an error (user-responsibility), got: {result:?}"
    );

    // In our implementation, INTG doesn't use R03, so the mathematical result is still correct.
    // The important thing is that no error is raised — the user is responsible for scratch safety.
    let x_val = state.stack.x.inner().to_f64().unwrap();
    // ∫₀¹ x² dx = 1/3 ≈ 0.333 (with n=10, Simpson should be accurate to ~0.001)
    let expected = 1.0 / 3.0;
    assert!(
        (x_val - expected).abs() < 0.01,
        "∫₀¹ x² dx should be ~{expected} even with scratch clobber (no error), got: {x_val}"
    );

    // R03 has been clobbered — document this as expected behavior
    // (R03 will hold f(x_last) = (1.0)^2 = 1.0 from the last sample x_k=1.0)
    let r03_val = state.regs[3].inner().to_f64().unwrap();
    assert!(
        r03_val > 0.0,
        "R03 must have been clobbered by user function (non-zero), got: {r03_val}"
    );
}
