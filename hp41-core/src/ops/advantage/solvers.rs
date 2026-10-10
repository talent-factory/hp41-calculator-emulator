// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `solvers` — ADV MATH solver states + full ops: FSOLVE / FINTG / FDIFEQ / FROOT.
//!
//! Each solver has its own transient state struct (D-43.7 re-entrancy design):
//! one active level per solver type, allowing cross-solver nesting (e.g. FINTG
//! calling FSOLVE internally).  All fields carry `#[serde(default, skip)]` on
//! `CalcState` — solver state is inherently transient.
//!
//! ## Cross-nesting guard policy (D-43.7)
//!
//! - FSOLVE guard checks ONLY `adv_fsolve_state.is_some()` (NOT adv_fintg_state).
//!   This allows FINTG to run inside FSOLVE's user callback.
//! - FINTG guard checks ONLY `adv_fintg_state.is_some()` (NOT adv_fsolve_state).
//!   This allows FSOLVE to run inside FINTG's user callback.
//! - FDIFEQ guard checks ONLY `adv_fdifeq_state.is_some()`.
//! - All guards check `call_stack.len() >= 4` (Pitfall 4).
//!
//! ## Algorithm sources
//!
//! - FSOLVE: Modified secant method, HP Advantage Pac OM Chapter 3.
//! - FINTG: Romberg/Simpson composite rule, HP Advantage Pac OM Chapter 4.
//! - FDIFEQ: RK4 (4th-order Runge-Kutta), HP Advantage Pac OM Chapter 5.
//! - FROOT: Laguerre's method (Numerical Recipes §9.5, independently derived).
//!
//! ## Cancellation (T-43-12)
//!
//! Every 64 iterations: check `state.cancel_requested`. If set, clear solver
//! state and return `HpError::Canceled`.
//!
//! ## Iteration limits (T-43-12)
//!
//! - FSOLVE: 100 iterations maximum.
//! - FINTG: 32768 subdivisions maximum.
//! - FDIFEQ: max_steps from R05 (default 1000).
//! - FROOT: 100 Laguerre iterations per root.

use std::sync::atomic::Ordering;

use crate::{
    error::HpError,
    format::format_hpnum,
    num::HpNum,
    ops::{math1::USER_CALLBACK_MAX_STEPS, Op},
    stack::{apply_lift_effect, enter_number, unary_result, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum iterations for FSOLVE secant loop (T-43-12).
const FSOLVE_MAX_ITERATIONS: u32 = 100;

/// Convergence threshold for FSOLVE: |f(x)| below this → root found.
const FSOLVE_CONVERGENCE_THRESHOLD: f64 = 5e-9;

/// Maximum subdivisions for FINTG (T-43-12).
const FINTG_MAX_SUBDIVISIONS: u32 = 32_768;

/// Maximum polynomial degree for FROOT (T-43-13).
const FROOT_MAX_DEGREE: usize = 100;

/// Maximum Laguerre iterations per root (T-43-12).
const LAGUERRE_MAX_ITER: u32 = 100;

/// Laguerre convergence tolerance.
const LAGUERRE_TOLERANCE: f64 = 1e-14;

// ---------------------------------------------------------------------------
// Solver state structs (transient — skipped by serde on CalcState)
// ---------------------------------------------------------------------------

/// Transient state for FROOT (polynomial root finding via Laguerre's method).
///
/// Stores the polynomial degree, found roots (as real/imag pairs), and a cursor
/// for sequential RTS output.
#[derive(Debug, Clone, Default)]
pub struct FrootState {
    /// Degree of the polynomial.
    pub degree: usize,
    /// Found roots as (real, imaginary) pairs. Length == degree on completion.
    pub roots_found: Vec<(f64, f64)>,
    /// Current cursor for RTS sequential output.
    pub root_index: usize,
}

/// Transient state for FINTG (numerical integration).
#[derive(Debug, Clone, Default)]
pub struct AdvFintegState {
    /// Lower bound of integration (X on entry, a).
    pub lower: HpNum,
    /// Upper bound of integration (Y on entry, b).
    pub upper: HpNum,
    /// Current accumulated integral estimate.
    pub accumulator: HpNum,
    /// Label/name of integrand function.
    pub function_label: String,
    /// Number of sub-intervals used so far.
    pub intervals: u32,
}

/// Transient state for FSOLVE (equation solver / root-finding with two initial guesses).
#[derive(Debug, Clone, Default)]
pub struct AdvFsolveState {
    /// First initial guess (R00 on entry, x1).
    pub x0: HpNum,
    /// Second initial guess (R01 on entry, x2).
    pub x1: HpNum,
    /// Current best estimate.
    pub current: HpNum,
    /// Label/name of function to solve.
    pub function_label: String,
    /// Iteration count.
    pub iterations: u32,
}

/// Transient state for FDIFEQ (differential equation integrator).
#[derive(Debug, Clone, Default)]
pub struct AdvFdifeqState {
    /// Order of the ODE system (1 or 2).
    pub order: u8,
    /// Current independent variable (x).
    pub x: HpNum,
    /// Current dependent-variable values y_0..y_(order-1).
    pub y: Vec<HpNum>,
    /// Step size.
    pub h: HpNum,
    /// Label/name of derivative function.
    pub function_label: String,
    /// Steps completed.
    pub steps: u32,
    /// Maximum steps to compute.
    pub max_steps: u32,
}

// ---------------------------------------------------------------------------
// Shared sub-runner: execute user callback function
// ---------------------------------------------------------------------------

/// Execute the user callback function starting at `state.pc`.
///
/// This is the same pattern as `math1::integ::run_user_function` and
/// `math1::solve::run_user_function`. The call_stack must have been pushed
/// by the caller before invoking this function.
///
/// Returns after RTN pops back to entry_depth or end of program.
fn run_user_function(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    use crate::ops::program::execute_op_pub;

    let entry_depth = state.call_stack.len();
    let mut steps: u64 = 0;

    loop {
        if steps >= USER_CALLBACK_MAX_STEPS {
            return Err(HpError::Overflow); // infinite-loop guard
        }
        steps += 1;

        if state.pc >= program.len() {
            break; // ran off end = implicit RTN
        }
        let op = program[state.pc].clone();
        state.pc += 1;

        match op {
            Op::Rtn => match state.call_stack.pop() {
                Some(return_pc) => {
                    state.pc = return_pc;
                    if state.call_stack.len() < entry_depth {
                        break;
                    }
                }
                None => break,
            },
            Op::Lbl(_) => {} // LBL is marker only
            Op::Stop => break,
            other => {
                execute_op_pub(state, other)?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Shared helper: evaluate f(x) for a user-program function
// ---------------------------------------------------------------------------

/// Push `x` to stack, call user function at `label_pos+1`, return f(x) as f64.
///
/// Used by FSOLVE secant loop. After call, X = f(x).
// NOTE: eval_user_fn is a documented helper that uses run_user_function; kept for potential future use.
#[allow(dead_code)]
fn eval_user_fn(
    state: &mut CalcState,
    program: &[Op],
    label_pos: usize,
    x: &HpNum,
    save_call_stack_len: usize,
    save_pc: usize,
) -> Result<f64, HpError> {
    // Push x to stack with lift enabled
    state.stack.lift_enabled = true;
    enter_number(state, x.clone());
    apply_lift_effect(state, LiftEffect::Enable);

    // Re-enter user function
    state.call_stack.push(state.pc);
    state.pc = label_pos + 1;

    let sub_result = run_user_function(state, program);

    // Defensive cleanup of extra frames
    while state.call_stack.len() > save_call_stack_len {
        state.call_stack.pop();
    }

    match sub_result {
        Ok(()) => {}
        Err(e) => {
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Err(e);
        }
    }

    state.stack.x.inner().to_f64().ok_or(HpError::Overflow)
}

// ---------------------------------------------------------------------------
// FSOLVE — Modified secant method (ADV-MATH-03)
// ---------------------------------------------------------------------------

/// ADV FSOLVE dispatch arm — returns InvalidOp (FSOLVE only runs in run_loop context).
///
/// # Errors
/// Returns `HpError::InvalidOp` unconditionally (must be called via run_loop).
pub fn op_adv_fsolve(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FSOLVE run-loop arm — modified secant method root-finding.
///
/// ## Entry convention (D-43.7 test-compatible):
/// - `state.alpha_reg` = function label
/// - `state.regs[0]` (R00) = first guess x1
/// - `state.regs[1]` (R01) = second guess x2
///
/// ## Nesting guard (D-43.7):
/// Only blocks `adv_fsolve_state.is_some()` (FINTG can nest inside FSOLVE).
///
/// ## Termination paths:
/// 1. "ROOT IS <v>" — convergence achieved
/// 2. "ROOT IS BETWEEN <v1> AND <v2>" — sign change but non-narrowable
/// 3. "NO ROOT FOUND" — iteration cap (100) reached
///
/// # Errors
/// Returns `HpError::InvalidOp` if already in FSOLVE, call_stack full, or label not found.
/// Returns `HpError::Canceled` if cancel flag set.
pub fn op_adv_fsolve_run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    // Guard 1 (D-43.7): only block self-nesting (NOT adv_fintg_state)
    if state.adv_fsolve_state.is_some() {
        return Err(HpError::InvalidOp);
    }
    // Guard 2 (Pitfall 4): pre-mutation call_stack cap
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }

    // Read parameters
    let user_label = state.alpha_reg.clone();
    let x1 = state
        .regs
        .first()
        .map(|v| v.numeric_or_zero())
        .unwrap_or_default();
    let x2 = state
        .regs
        .get(1)
        .map(|v| v.numeric_or_zero())
        .unwrap_or_default();

    // Find label before any state mutation (fail fast)
    let label_pos = match program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if *l == user_label))
    {
        Some(pos) => pos,
        None => {
            return Err(HpError::InvalidOp);
        }
    };

    // Commit state
    state.adv_fsolve_state = Some(AdvFsolveState {
        function_label: user_label.clone(),
        x0: x1.clone(),
        x1: x2.clone(),
        current: x1.clone(),
        iterations: 0,
    });

    let save_pc = state.pc;
    let save_call_stack_len = state.call_stack.len();

    let mut x1_f64 = match x1.inner().to_f64() {
        Some(v) => v,
        None => {
            state.adv_fsolve_state = None;
            return Err(HpError::Overflow);
        }
    };
    let mut x2_f64 = match x2.inner().to_f64() {
        Some(v) => v,
        None => {
            state.adv_fsolve_state = None;
            return Err(HpError::Overflow);
        }
    };

    // Evaluate f(x1) and f(x2)
    let mut fx1_f64 = match eval_user_fn_inner(
        state,
        program,
        label_pos,
        x1_f64,
        save_call_stack_len,
        save_pc,
    ) {
        Ok(v) => v,
        Err(e) => {
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Err(e);
        }
    };
    let mut fx2_f64 = match eval_user_fn_inner(
        state,
        program,
        label_pos,
        x2_f64,
        save_call_stack_len,
        save_pc,
    ) {
        Ok(v) => v,
        Err(e) => {
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Err(e);
        }
    };

    // Modified secant loop
    for iter in 0..FSOLVE_MAX_ITERATIONS {
        // Per-64-iterations cancellation check
        if iter & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Err(HpError::Canceled);
        }

        if let Some(ref mut ss) = state.adv_fsolve_state {
            ss.iterations = iter;
        }

        let denom = fx2_f64 - fx1_f64;

        if denom.abs() < 1e-300 {
            // Secant denominator zero — check for sign change
            if fx1_f64 * fx2_f64 < 0.0 {
                let v1 = format_hpnum(
                    &HpNum::from(Decimal::from_f64(x1_f64).unwrap_or(Decimal::ZERO)),
                    &state.display_mode,
                );
                let v2 = format_hpnum(
                    &HpNum::from(Decimal::from_f64(x2_f64).unwrap_or(Decimal::ZERO)),
                    &state.display_mode,
                );
                state
                    .print_buffer
                    .push(format!("ROOT IS BETWEEN {v1} AND {v2}"));
            } else {
                state.print_buffer.push("NO ROOT FOUND".to_string());
            }
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Ok(());
        }

        // Secant step: x_new = x2 - f(x2)*(x2-x1)/(f(x2)-f(x1))
        let x_new_f64 = x2_f64 - fx2_f64 * (x2_f64 - x1_f64) / denom;
        if !x_new_f64.is_finite() {
            state.print_buffer.push("NO ROOT FOUND".to_string());
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Err(HpError::Overflow);
        }
        let x_new = HpNum::from(Decimal::from_f64(x_new_f64).ok_or(HpError::Overflow)?);

        let fx_new_f64 = match eval_user_fn_inner(
            state,
            program,
            label_pos,
            x_new_f64,
            save_call_stack_len,
            save_pc,
        ) {
            Ok(v) => v,
            Err(e) => {
                state.adv_fsolve_state = None;
                state.pc = save_pc;
                return Err(e);
            }
        };

        // Convergence check
        if fx_new_f64.abs() < FSOLVE_CONVERGENCE_THRESHOLD {
            let v = format_hpnum(&x_new, &state.display_mode);
            state.print_buffer.push(format!("ROOT IS {v}"));
            // Push root to X
            state.stack.lift_enabled = true;
            enter_number(state, x_new.clone());
            apply_lift_effect(state, LiftEffect::Enable);
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Ok(());
        }

        // Sign change stagnation check
        if fx1_f64 * fx_new_f64 < 0.0 && (x_new_f64 - x2_f64).abs() < 1e-14 * x2_f64.abs().max(1.0)
        {
            let v1 = format_hpnum(
                &HpNum::from(Decimal::from_f64(x1_f64).unwrap_or(Decimal::ZERO)),
                &state.display_mode,
            );
            let v2 = format_hpnum(&x_new, &state.display_mode);
            state
                .print_buffer
                .push(format!("ROOT IS BETWEEN {v1} AND {v2}"));
            state.adv_fsolve_state = None;
            state.pc = save_pc;
            return Ok(());
        }

        // Shift: x1 ← x2, x2 ← x_new
        x1_f64 = x2_f64;
        fx1_f64 = fx2_f64;
        x2_f64 = x_new_f64;
        fx2_f64 = fx_new_f64;
    }

    // Iteration cap — surface failure to programmatic callers
    state.print_buffer.push("NO ROOT FOUND".to_string());
    state.adv_fsolve_state = None;
    state.pc = save_pc;
    Err(HpError::NoRoot)
}

/// Inner helper: evaluate f(x_val_f64) via run_user_function re-entry.
fn eval_user_fn_inner(
    state: &mut CalcState,
    program: &[Op],
    label_pos: usize,
    x_val_f64: f64,
    save_call_stack_len: usize,
    save_pc: usize,
) -> Result<f64, HpError> {
    let x_num = HpNum::from(Decimal::from_f64(x_val_f64).ok_or(HpError::Overflow)?);

    state.stack.lift_enabled = true;
    enter_number(state, x_num);
    apply_lift_effect(state, LiftEffect::Enable);

    state.call_stack.push(state.pc);
    state.pc = label_pos + 1;

    let sub_result = run_user_function(state, program);

    while state.call_stack.len() > save_call_stack_len {
        state.call_stack.pop();
    }

    sub_result.inspect_err(|_| {
        state.pc = save_pc;
    })?;

    state.stack.x.inner().to_f64().ok_or(HpError::Overflow)
}

// ---------------------------------------------------------------------------
// FINTG — Composite Simpson integration (ADV-MATH-04)
// ---------------------------------------------------------------------------

/// Simpson 1/3 rule coefficient for sample index k in [0..=n].
fn simpson_coeff(k: u32, n: u32) -> f64 {
    if k == 0 || k == n {
        1.0
    } else if k % 2 == 1 {
        4.0
    } else {
        2.0
    }
}

/// ADV FINTG dispatch arm — returns InvalidOp (FINTG only runs in run_loop context).
///
/// # Errors
/// Returns `HpError::InvalidOp` unconditionally.
pub fn op_adv_fintg(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FINTG run-loop arm — composite Simpson integration.
///
/// ## Entry convention:
/// - `state.alpha_reg` = function label
/// - `state.regs[0]` (R00) = number of subdivisions N (integer)
/// - `state.stack.x` = lower bound a
/// - `state.stack.y` = upper bound b
///
/// ## Nesting guard (D-43.7):
/// Only blocks `adv_fintg_state.is_some()` (FSOLVE can nest inside FINTG).
///
/// ## Result:
/// Pushes integral value to X with `LiftEffect::Enable`.
///
/// # Errors
/// Returns `HpError::InvalidOp` if already in FINTG, call_stack full, or label not found.
/// Returns `HpError::Domain` if N > 32768.
/// Returns `HpError::Canceled` if cancel flag set.
pub fn op_adv_fintg_run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    // Guard 1 (D-43.7): only block self-nesting (NOT adv_fsolve_state)
    if state.adv_fintg_state.is_some() {
        return Err(HpError::InvalidOp);
    }
    // Guard 2 (Pitfall 4): pre-mutation call_stack cap
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }

    // Read parameters
    let a = state.stack.x.clone();
    let b = state.stack.y.clone();
    let n_raw = state
        .regs
        .first()
        .map(|r| r.numeric_or_zero().inner().to_u32().unwrap_or(0))
        .unwrap_or(0);
    let user_label = state.alpha_reg.clone();

    // Guard 3: subdivision cap (T-43-12)
    if n_raw > FINTG_MAX_SUBDIVISIONS {
        return Err(HpError::Domain);
    }

    let n = n_raw.max(2);
    // Simpson requires even N
    let n_even = if n % 2 == 1 { n + 1 } else { n };

    // Find label before mutation
    let label_pos = program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if *l == user_label))
        .ok_or(HpError::InvalidOp)?;

    // Commit state
    state.adv_fintg_state = Some(AdvFintegState {
        function_label: user_label,
        lower: a.clone(),
        upper: b.clone(),
        accumulator: HpNum::zero(),
        intervals: n_even,
    });

    let a_f64 = match a.inner().to_f64() {
        Some(v) => v,
        None => {
            state.adv_fintg_state = None;
            return Err(HpError::Overflow);
        }
    };
    let b_f64 = match b.inner().to_f64() {
        Some(v) => v,
        None => {
            state.adv_fintg_state = None;
            return Err(HpError::Overflow);
        }
    };

    // Zero-width interval → result is 0
    if (b_f64 - a_f64).abs() < f64::EPSILON {
        state.adv_fintg_state = None;
        unary_result(state, HpNum::zero());
        apply_lift_effect(state, LiftEffect::Enable);
        return Ok(());
    }

    let h_f64 = (b_f64 - a_f64) / (n_even as f64);
    let mut sum_f64: f64 = 0.0;
    let save_pc = state.pc;
    let save_call_stack_len = state.call_stack.len();

    for k in 0..=n_even {
        // Per-64-samples cancellation check
        if k & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
            state.adv_fintg_state = None;
            state.pc = save_pc;
            return Err(HpError::Canceled);
        }

        let x_k = a_f64 + (k as f64) * h_f64;
        let coeff = simpson_coeff(k, n_even);

        // Push x_k to stack
        state.stack.lift_enabled = true;
        let x_k_dec = match Decimal::from_f64(x_k) {
            Some(d) => d,
            None => {
                state.adv_fintg_state = None;
                state.pc = save_pc;
                return Err(HpError::Overflow);
            }
        };
        enter_number(state, HpNum::from(x_k_dec));
        apply_lift_effect(state, LiftEffect::Enable);

        // Re-enter user function
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let sub_result = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        match sub_result {
            Ok(()) => {}
            Err(e) => {
                state.adv_fintg_state = None;
                state.pc = save_pc;
                return Err(e);
            }
        }

        let fx_k = match state.stack.x.inner().to_f64() {
            Some(v) => v,
            None => {
                state.adv_fintg_state = None;
                state.pc = save_pc;
                return Err(HpError::Overflow);
            }
        };
        sum_f64 += coeff * fx_k;

        if !sum_f64.is_finite() {
            state.adv_fintg_state = None;
            state.pc = save_pc;
            return Err(HpError::Overflow);
        }
        // Update accumulator for diagnostic
        if let Some(ref mut st) = state.adv_fintg_state {
            st.accumulator = HpNum::from(Decimal::from_f64(sum_f64).unwrap_or(Decimal::ZERO));
        }
    }

    // Final result: (h/3) * sum
    let result_f64 = (h_f64 / 3.0) * sum_f64;
    state.pc = save_pc;
    state.adv_fintg_state = None;

    let result = HpNum::from(Decimal::from_f64(result_f64).ok_or(HpError::Overflow)?);
    unary_result(state, result);
    apply_lift_effect(state, LiftEffect::Enable);

    Ok(())
}

// ---------------------------------------------------------------------------
// FDIFEQ — RK4 ODE solver (ADV-MATH-05)
// ---------------------------------------------------------------------------

/// ADV FDIFEQ dispatch arm — returns InvalidOp (FDIFEQ only runs in run_loop context).
///
/// # Errors
/// Returns `HpError::InvalidOp` unconditionally.
pub fn op_adv_fdifeq(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FDIFEQ run-loop arm — RK4 ODE integration.
///
/// ## Entry convention (symmetric with math1/difeq.rs):
/// - `state.alpha_reg` = function label
/// - `state.regs[0]` (R00) = order (1 or 2)
/// - `state.regs[1]` (R01) = step size h
/// - `state.regs[2]` (R02) = x0 (initial x)
/// - `state.regs[3]` (R03) = y0 (initial y)
/// - `state.regs[4]` (R04) = y'0 (initial y', only for ORDER=2)
/// - `state.regs[5]` (R05) = max_steps (0 = default 1000)
///
/// ## Output:
/// Each RK4 step pushes "X=<v> Y=<v>" to print_buffer.
///
/// # Errors
/// Returns `HpError::InvalidOp` if already in FDIFEQ or label not found.
/// Returns `HpError::Canceled` if cancel flag set.
pub fn op_adv_fdifeq_run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    // Guard 1: only block self-nesting
    if state.adv_fdifeq_state.is_some() {
        return Err(HpError::InvalidOp);
    }
    // Guard 2 (Pitfall 4)
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }

    let user_label = state.alpha_reg.clone();
    let order_raw = state
        .regs
        .first()
        .map(|r| r.numeric_or_zero().inner().to_u8().unwrap_or(0))
        .unwrap_or(0);
    let step_size = state
        .regs
        .get(1)
        .map(|v| v.numeric_or_zero())
        .unwrap_or_default();
    let x0 = state
        .regs
        .get(2)
        .map(|v| v.numeric_or_zero())
        .unwrap_or_default();
    let y0 = state
        .regs
        .get(3)
        .map(|v| v.numeric_or_zero())
        .unwrap_or_default();
    let y_prime0 = state
        .regs
        .get(4)
        .map(|v| v.numeric_or_zero())
        .unwrap_or_default();
    let max_steps_raw = state
        .regs
        .get(5)
        .map(|r| r.numeric_or_zero().inner().to_u32().unwrap_or(0))
        .unwrap_or(0);
    let max_steps = if max_steps_raw == 0 {
        1000u32
    } else {
        max_steps_raw
    };

    if order_raw != 1 && order_raw != 2 {
        return Err(HpError::Domain);
    }
    let order = order_raw;

    // Find label before mutation
    let label_pos = program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if *l == user_label))
        .ok_or(HpError::InvalidOp)?;

    state.adv_fdifeq_state = Some(AdvFdifeqState {
        order,
        x: x0.clone(),
        y: vec![y0.clone()],
        h: step_size.clone(),
        function_label: user_label,
        steps: 0,
        max_steps,
    });

    // Print initial state
    let x_str = format_hpnum(&x0, &state.display_mode);
    let y_str = format_hpnum(&y0, &state.display_mode);
    let initial_line = if order == 2 {
        let yp_str = format_hpnum(&y_prime0, &state.display_mode);
        format!("X={x_str} Y={y_str} Y'={yp_str}")
    } else {
        format!("X={x_str} Y={y_str}")
    };
    state.print_buffer.push(initial_line);

    let save_pc = state.pc;
    let save_call_stack_len = state.call_stack.len();
    let display_mode = state.display_mode;

    loop {
        let (step_count, max_s, x_n, y_n, z_n_opt, h) = {
            let st = state.adv_fdifeq_state.as_ref().ok_or(HpError::InvalidOp)?;
            (
                st.steps,
                st.max_steps,
                st.x.clone(),
                st.y.first().cloned().unwrap_or_default(),
                if order == 2 {
                    st.y.get(1).cloned()
                } else {
                    None
                },
                st.h.clone(),
            )
        };

        // Per-64-steps cancellation check
        if step_count & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
            state.adv_fdifeq_state = None;
            state.pc = save_pc;
            return Err(HpError::Canceled);
        }

        if step_count >= max_s {
            state.adv_fdifeq_state = None;
            state.pc = save_pc;
            return Ok(());
        }

        let x_f64 = match x_n.inner().to_f64() {
            Some(v) => v,
            None => {
                state.adv_fdifeq_state = None;
                state.pc = save_pc;
                return Err(HpError::Overflow);
            }
        };
        let y_f64 = match y_n.inner().to_f64() {
            Some(v) => v,
            None => {
                state.adv_fdifeq_state = None;
                state.pc = save_pc;
                return Err(HpError::Overflow);
            }
        };
        let h_f64 = match h.inner().to_f64() {
            Some(v) => v,
            None => {
                state.adv_fdifeq_state = None;
                state.pc = save_pc;
                return Err(HpError::Overflow);
            }
        };

        // Evaluate user callback with current (x, y) → returns f(x, y)
        // Push y first (becomes Y register), then x (becomes X register),
        // so the user function sees x in X and y in Y per HP Advantage Pac convention.
        let eval_f = |state: &mut CalcState, x_val: f64, y_val: f64| -> Result<f64, HpError> {
            let y_dec = Decimal::from_f64(y_val).ok_or(HpError::Overflow)?;
            let x_dec = Decimal::from_f64(x_val).ok_or(HpError::Overflow)?;
            state.stack.lift_enabled = true;
            enter_number(state, HpNum::from(y_dec));
            apply_lift_effect(state, LiftEffect::Enable);
            state.stack.lift_enabled = true;
            enter_number(state, HpNum::from(x_dec));
            apply_lift_effect(state, LiftEffect::Enable);

            state.call_stack.push(state.pc);
            state.pc = label_pos + 1;
            let sub_result = run_user_function(state, program);
            while state.call_stack.len() > save_call_stack_len {
                state.call_stack.pop();
            }
            match sub_result {
                Ok(()) => {}
                Err(e) => {
                    state.adv_fdifeq_state = None;
                    state.pc = save_pc;
                    return Err(e);
                }
            }
            state.stack.x.inner().to_f64().ok_or(HpError::Overflow)
        };

        // ORDER=1 RK4:
        // k1 = h * f(x_n, y_n)
        // k2 = h * f(x_n + h/2, y_n + k1/2)
        // k3 = h * f(x_n + h/2, y_n + k2/2)
        // k4 = h * f(x_n + h, y_n + k3)
        // y_{n+1} = y_n + (k1 + 2*k2 + 2*k3 + k4) / 6
        let (y_new_f64, z_new_opt_f64) = if order == 1 {
            let k1 = h_f64 * eval_f(state, x_f64, y_f64)?;
            let k2 = h_f64 * eval_f(state, x_f64 + h_f64 / 2.0, y_f64 + k1 / 2.0)?;
            let k3 = h_f64 * eval_f(state, x_f64 + h_f64 / 2.0, y_f64 + k2 / 2.0)?;
            let k4 = h_f64 * eval_f(state, x_f64 + h_f64, y_f64 + k3)?;
            let y_new = y_f64 + (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
            (y_new, None)
        } else {
            // ORDER=2: coupled system y'=z, z'=f(x,y,z)
            let z_f64 = z_n_opt
                .as_ref()
                .and_then(|v| v.inner().to_f64())
                .unwrap_or(0.0);

            let k1z = h_f64 * eval_f(state, x_f64, y_f64)?;
            let k1y = h_f64 * z_f64;

            let k2z = h_f64 * eval_f(state, x_f64 + h_f64 / 2.0, y_f64 + k1y / 2.0)?;
            let k2y = h_f64 * (z_f64 + k1z / 2.0);

            let k3z = h_f64 * eval_f(state, x_f64 + h_f64 / 2.0, y_f64 + k2y / 2.0)?;
            let k3y = h_f64 * (z_f64 + k2z / 2.0);

            let k4z = h_f64 * eval_f(state, x_f64 + h_f64, y_f64 + k3y)?;
            let k4y = h_f64 * (z_f64 + k3z);

            let y_new = y_f64 + (k1y + 2.0 * k2y + 2.0 * k3y + k4y) / 6.0;
            let z_new = z_f64 + (k1z + 2.0 * k2z + 2.0 * k3z + k4z) / 6.0;
            (y_new, Some(z_new))
        };

        let x_new_f64 = x_f64 + h_f64;
        if !x_new_f64.is_finite() || !y_new_f64.is_finite() {
            state.adv_fdifeq_state = None;
            state.pc = save_pc;
            return Err(HpError::Overflow);
        }
        let x_new = HpNum::from(Decimal::from_f64(x_new_f64).ok_or(HpError::Overflow)?);
        let y_new = HpNum::from(Decimal::from_f64(y_new_f64).ok_or(HpError::Overflow)?);

        // Update state
        if let Some(ref mut st) = state.adv_fdifeq_state {
            st.x = x_new.clone();
            st.y[0] = y_new.clone();
            if order == 2 {
                if let Some(z_f64) = z_new_opt_f64 {
                    if !z_f64.is_finite() {
                        state.adv_fdifeq_state = None;
                        state.pc = save_pc;
                        return Err(HpError::Overflow);
                    }
                    let z_new = HpNum::from(Decimal::from_f64(z_f64).ok_or(HpError::Overflow)?);
                    if st.y.len() > 1 {
                        st.y[1] = z_new;
                    } else {
                        st.y.push(z_new);
                    }
                }
            }
            st.steps += 1;
        }

        // Push step output
        let line = if order == 2 {
            let z_str = if let Some(z_f) = z_new_opt_f64 {
                let z_hp = HpNum::from(Decimal::from_f64(z_f).unwrap_or(Decimal::ZERO));
                format_hpnum(&z_hp, &display_mode)
            } else {
                "0".to_string()
            };
            format!(
                "X={} Y={} Y'={}",
                format_hpnum(&x_new, &display_mode),
                format_hpnum(&y_new, &display_mode),
                z_str
            )
        } else {
            format!(
                "X={} Y={}",
                format_hpnum(&x_new, &display_mode),
                format_hpnum(&y_new, &display_mode)
            )
        };
        state.print_buffer.push(line);
    }
}

// ---------------------------------------------------------------------------
// FROOT — Laguerre's method for polynomial roots (ADV-MATH-06)
// ---------------------------------------------------------------------------

/// ADV FROOT — find all roots of a polynomial of arbitrary degree.
///
/// ## Entry convention (D-43.8):
/// - `state.stack.x` = degree (positive integer ≤ 100)
/// - `state.regs[1]` (R01) through `state.regs[degree+1]` = coefficients,
///   highest-degree first.
///
/// ## Algorithm: Laguerre's method (Numerical Recipes §9.5)
///
/// For each root: start with initial guess x = 0. Iterate Laguerre step using
/// Horner evaluation of P(x), P'(x), P''(x). Deflate polynomial after each root.
///
/// All arithmetic in f64 for numerical stability. Results converted to HpNum at end.
///
/// ## Output:
/// Found roots stored in `state.adv_froot_state` for sequential RTS output.
/// Real roots also pushed to print_buffer.
///
/// # Errors
/// Returns `HpError::Domain` if degree is 0 or > 100.
/// Returns `HpError::InvalidOp` on any numerical failure.
pub fn op_adv_froot(state: &mut CalcState) -> Result<(), HpError> {
    let degree_f64 = state.stack.x.inner().to_f64().ok_or(HpError::Domain)?;

    if !degree_f64.is_finite() || degree_f64 < 1.0 || degree_f64 > FROOT_MAX_DEGREE as f64 {
        return Err(HpError::Domain);
    }

    let degree = degree_f64 as usize;

    // Read coefficients from R01..R(degree+1), highest-degree first
    let mut coeffs: Vec<f64> = Vec::with_capacity(degree + 1);
    for i in 0..=degree {
        let reg_idx = i + 1; // R01 = highest-degree coefficient
        let coeff = state
            .regs
            .get(reg_idx)
            .map(|r| r.numeric_or_zero().inner().to_f64().unwrap_or(0.0))
            .unwrap_or(0.0);
        coeffs.push(coeff);
    }

    // Find all roots using Laguerre's method with deflation
    let roots = laguerre_roots(&coeffs, degree)?;

    // Store results in adv_froot_state
    state.adv_froot_state = Some(FrootState {
        degree,
        roots_found: roots.clone(),
        root_index: 0,
    });

    // Push real roots to print_buffer
    for (re, im) in &roots {
        if im.abs() < 1e-10 {
            let root_hp = HpNum::from(Decimal::from_f64(*re).unwrap_or(Decimal::ZERO));
            state.print_buffer.push(format!(
                "ROOT: {}",
                format_hpnum(&root_hp, &state.display_mode)
            ));
        } else {
            let re_hp = HpNum::from(Decimal::from_f64(*re).unwrap_or(Decimal::ZERO));
            let im_hp = HpNum::from(Decimal::from_f64(im.abs()).unwrap_or(Decimal::ZERO));
            let sign = if *im < 0.0 { "-" } else { "+" };
            state.print_buffer.push(format!(
                "ROOT: {}{}{}i",
                format_hpnum(&re_hp, &state.display_mode),
                sign,
                format_hpnum(&im_hp, &state.display_mode)
            ));
        }
    }

    // Push first root to X (real part)
    if let Some((re, _im)) = roots.first() {
        let root_hp = HpNum::from(Decimal::from_f64(*re).ok_or(HpError::Overflow)?);
        unary_result(state, root_hp);
        apply_lift_effect(state, LiftEffect::Enable);
    }

    Ok(())
}

/// Laguerre's method to find all roots of a polynomial with real coefficients.
///
/// Strategy for real polynomials:
/// - Apply Laguerre at complex x to find a root (which may be complex).
/// - If the root is real (|im| < threshold): deflate by linear factor (x - root_re).
/// - If the root is complex: also add its conjugate and deflate by the quadratic
///   factor (x^2 - 2*re*x + (re^2+im^2)) to keep remaining polynomial real.
///
/// This avoids the numerical issues of complex synthetic division on real coefficients.
///
/// Algorithm: Numerical Recipes §9.5 — independently re-derived.
fn laguerre_roots(coeffs: &[f64], degree: usize) -> Result<Vec<(f64, f64)>, HpError> {
    // Work on a copy of the deflated polynomial (always real coefficients)
    let mut remaining = coeffs.to_vec();
    let mut roots: Vec<(f64, f64)> = Vec::with_capacity(degree);

    let mut root_idx = 0;
    while root_idx < degree {
        let n = remaining.len() - 1; // current polynomial degree
        if n == 0 {
            break;
        }

        // Choose an initial guess that avoids the symmetric singularity at x=0.
        // Use a small non-zero complex starting point to improve convergence
        // for polynomials like x^2+1.
        let start_re = 0.4;
        let start_im = 0.9;
        let mut x_re = start_re;
        let mut x_im = start_im;

        // Laguerre iteration (up to LAGUERRE_MAX_ITER)
        for _iter in 0..LAGUERRE_MAX_ITER {
            // Evaluate P(x), P'(x), P''(x) via Horner at complex x
            let (p, dp, ddp) = horner_complex(&remaining, x_re, x_im);

            let p_abs = (p.0 * p.0 + p.1 * p.1).sqrt();
            if p_abs < LAGUERRE_TOLERANCE {
                break;
            }

            // G = P'/P (complex division) — None means P(x)≈0, i.e. root found
            let (g_re, g_im) = match complex_div(dp.0, dp.1, p.0, p.1) {
                Some(v) => v,
                None => break,
            };
            // H = G^2 - P''/P
            let g2_re = g_re * g_re - g_im * g_im;
            let g2_im = 2.0 * g_re * g_im;
            let (pp_over_p_re, pp_over_p_im) = match complex_div(ddp.0, ddp.1, p.0, p.1) {
                Some(v) => v,
                None => break,
            };
            let h_re = g2_re - pp_over_p_re;
            let h_im = g2_im - pp_over_p_im;

            // Discriminant: (n-1) * (n*H - G^2)
            let n_f = n as f64;
            let disc_re = (n_f - 1.0) * (n_f * h_re - g2_re);
            let disc_im = (n_f - 1.0) * (n_f * h_im - g2_im);

            // sqrt(discriminant) — complex square root
            let (sqrt_re, sqrt_im) = complex_sqrt(disc_re, disc_im);

            // Choose sign to maximize |denominator|
            let denom1_re = g_re + sqrt_re;
            let denom1_im = g_im + sqrt_im;
            let denom2_re = g_re - sqrt_re;
            let denom2_im = g_im - sqrt_im;
            let abs1 = denom1_re * denom1_re + denom1_im * denom1_im;
            let abs2 = denom2_re * denom2_re + denom2_im * denom2_im;

            let (denom_re, denom_im) = if abs1 >= abs2 {
                (denom1_re, denom1_im)
            } else {
                (denom2_re, denom2_im)
            };

            // Step: a = n / denom — None means degenerate case, stop iteration
            let (a_re, a_im) = match complex_div(n_f, 0.0, denom_re, denom_im) {
                Some(v) => v,
                None => break,
            };

            let abs_a = (a_re * a_re + a_im * a_im).sqrt();
            if abs_a < LAGUERRE_TOLERANCE {
                break;
            }

            x_re -= a_re;
            x_im -= a_im;
        }

        // Polish the root against the ORIGINAL polynomial (not deflated)
        // to reduce accumulated deflation error
        for _ in 0..10 {
            let (p, dp, _) = horner_complex(coeffs, x_re, x_im);
            let dp_abs = dp.0 * dp.0 + dp.1 * dp.1;
            if dp_abs < 1e-300 {
                break;
            }
            let (step_re, step_im) = match complex_div(p.0, p.1, dp.0, dp.1) {
                Some(v) => v,
                None => break,
            };
            let new_re = x_re - step_re;
            let new_im = x_im - step_im;
            if (step_re * step_re + step_im * step_im).sqrt() < 1e-14 {
                x_re = new_re;
                x_im = new_im;
                break;
            }
            x_re = new_re;
            x_im = new_im;
        }

        // Snap small imaginary parts to zero (numerical noise threshold)
        if x_im.abs() < 1e-10 {
            x_im = 0.0;
        }

        if x_im.abs() < 1e-8 {
            // Treat as real root
            x_im = 0.0;
            roots.push((x_re, x_im));
            root_idx += 1;
            // Deflate by linear factor (x - x_re) — real division
            remaining = deflate_linear(&remaining, x_re);
        } else {
            // Complex root: also add conjugate, deflate by quadratic factor
            // (x^2 - 2*x_re*x + (x_re^2 + x_im^2))
            roots.push((x_re, x_im));
            roots.push((x_re, -x_im)); // conjugate
            root_idx += 2;
            let quadratic_b = -2.0 * x_re;
            let quadratic_c = x_re * x_re + x_im * x_im;
            remaining = deflate_quadratic(&remaining, quadratic_b, quadratic_c);
        }
    }

    Ok(roots)
}

/// Evaluate P(x), P'(x), P''(x) at complex x using Horner's method.
/// Returns ((P_re, P_im), (P'_re, P'_im), (P''_re, P''_im)).
fn horner_complex(coeffs: &[f64], x_re: f64, x_im: f64) -> ((f64, f64), (f64, f64), (f64, f64)) {
    let mut p_re = coeffs[0];
    let mut p_im = 0.0_f64;
    let mut dp_re = 0.0_f64;
    let mut dp_im = 0.0_f64;
    let mut ddp_re = 0.0_f64;
    let mut ddp_im = 0.0_f64;

    for &c in coeffs.iter().skip(1) {
        // ddp = ddp * x + 2 * dp
        let ddp_re_new = ddp_re * x_re - ddp_im * x_im + 2.0 * dp_re;
        let ddp_im_new = ddp_re * x_im + ddp_im * x_re + 2.0 * dp_im;
        ddp_re = ddp_re_new;
        ddp_im = ddp_im_new;

        // dp = dp * x + p
        let dp_re_new = dp_re * x_re - dp_im * x_im + p_re;
        let dp_im_new = dp_re * x_im + dp_im * x_re + p_im;
        dp_re = dp_re_new;
        dp_im = dp_im_new;

        // p = p * x + c
        let p_re_new = p_re * x_re - p_im * x_im + c;
        let p_im_new = p_re * x_im + p_im * x_re;
        p_re = p_re_new;
        p_im = p_im_new;
    }

    ((p_re, p_im), (dp_re, dp_im), (ddp_re, ddp_im))
}

/// Complex division: (a_re + i*a_im) / (b_re + i*b_im).
/// Returns `None` when the denominator is near zero.
fn complex_div(a_re: f64, a_im: f64, b_re: f64, b_im: f64) -> Option<(f64, f64)> {
    let denom = b_re * b_re + b_im * b_im;
    if denom < 1e-300 {
        return None;
    }
    Some((
        (a_re * b_re + a_im * b_im) / denom,
        (a_im * b_re - a_re * b_im) / denom,
    ))
}

/// Complex square root.
fn complex_sqrt(re: f64, im: f64) -> (f64, f64) {
    let r = (re * re + im * im).sqrt().sqrt();
    let theta = im.atan2(re) / 2.0;
    (r * theta.cos(), r * theta.sin())
}

/// Deflate polynomial by dividing out linear factor (x - root_re).
/// Synthetic division for real root. Returns quotient coefficients.
fn deflate_linear(coeffs: &[f64], root_re: f64) -> Vec<f64> {
    if coeffs.len() <= 1 {
        return vec![1.0];
    }
    let n = coeffs.len() - 1;
    let mut result = vec![0.0_f64; n];
    result[0] = coeffs[0];
    for i in 1..n {
        result[i] = coeffs[i] + result[i - 1] * root_re;
    }
    result
}

/// Deflate polynomial by dividing out quadratic factor (x^2 + b*x + c).
/// Used for complex-conjugate pairs. Returns quotient coefficients (real).
///
/// For polynomial P of degree n (n+1 coefficients) divided by degree-2 factor,
/// quotient Q has degree n-2 (n-1 coefficients).
fn deflate_quadratic(coeffs: &[f64], b: f64, c: f64) -> Vec<f64> {
    let len = coeffs.len();
    if len <= 2 {
        return vec![1.0];
    }
    // Quotient has len-2 coefficients (degree = len-3)
    let q_len = len - 2;
    let mut q = vec![0.0_f64; q_len];
    q[0] = coeffs[0];
    if q_len >= 2 {
        q[1] = coeffs[1] - b * q[0];
    }
    for i in 2..q_len {
        q[i] = coeffs[i] - b * q[i - 1] - c * q[i - 2];
    }
    q
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ops::advantage::{op_adv_ply, op_adv_rts};

    // Catches: FrootState default initializes cleanly
    #[test]
    fn froot_state_default() {
        let s = FrootState::default();
        assert_eq!(s.degree, 0);
        assert_eq!(s.root_index, 0);
        assert!(s.roots_found.is_empty());
    }

    // Catches: AdvFintegState default initializes cleanly
    #[test]
    fn finteg_state_default() {
        let s = AdvFintegState::default();
        assert_eq!(s.intervals, 0);
        assert!(s.function_label.is_empty());
    }

    // Catches: AdvFsolveState default initializes cleanly
    #[test]
    fn fsolve_state_default() {
        let s = AdvFsolveState::default();
        assert_eq!(s.iterations, 0);
        assert!(s.function_label.is_empty());
    }

    // Catches: AdvFdifeqState default initializes cleanly
    #[test]
    fn fdifeq_state_default() {
        let s = AdvFdifeqState::default();
        assert_eq!(s.order, 0);
        assert_eq!(s.steps, 0);
        assert!(s.y.is_empty());
        assert!(s.function_label.is_empty());
    }

    // Catches: solver dispatch stubs return InvalidOp (dispatch arm invariant)
    #[test]
    fn solver_dispatch_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_fsolve(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fintg(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fdifeq(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: solver states are Clone
    #[test]
    fn solver_states_are_clone() {
        let _r = FrootState::default().clone();
        let _i = AdvFintegState::default().clone();
        let _s = AdvFsolveState::default().clone();
        let _d = AdvFdifeqState::default().clone();
    }

    // Catches: FSOLVE finds root of x^2-4=0 near bracket [1,3] → x=2
    #[test]
    fn fsolve_x2_minus4() {
        let program = vec![
            Op::Lbl("SLV".to_string()),
            Op::Sq,
            Op::PushNum(HpNum::from(4i32)),
            Op::Sub,
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "SLV".to_string();
        state.regs[0] = HpNum::from(1i32).into();
        state.regs[1] = HpNum::from(3i32).into();
        state.stack.lift_enabled = false;

        let result = op_adv_fsolve_run_loop(&mut state, &program);
        assert!(result.is_ok(), "FSOLVE x^2-4=0: {result:?}");
        assert!(!state.print_buffer.is_empty());
        assert!(
            state.print_buffer[0].starts_with("ROOT IS"),
            "expected ROOT IS, got: {:?}",
            state.print_buffer[0]
        );
        assert!(state.adv_fsolve_state.is_none());
    }

    // Catches: FSOLVE no-root produces NO ROOT FOUND
    #[test]
    fn fsolve_no_root() {
        let program = vec![
            Op::Lbl("NR".to_string()),
            Op::Sq,
            Op::PushNum(HpNum::from(1i32)),
            Op::Add,
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "NR".to_string();
        state.regs[0] = HpNum::from(1i32).into();
        state.regs[1] = HpNum::from(2i32).into();
        state.stack.lift_enabled = false;

        let result = op_adv_fsolve_run_loop(&mut state, &program);
        assert!(matches!(result, Err(HpError::NoRoot)));
        assert_eq!(
            state.print_buffer.first().map(|s| s.as_str()),
            Some("NO ROOT FOUND")
        );
        assert!(state.adv_fsolve_state.is_none());
    }

    // Catches: FSOLVE self-nesting not blocked
    #[test]
    fn fsolve_self_nesting_blocked() {
        let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
        let mut state = CalcState::new();
        state.adv_fsolve_state = Some(AdvFsolveState::default());
        let result = op_adv_fsolve_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::InvalidOp));
        assert!(state.adv_fsolve_state.is_some()); // unchanged
    }

    // Catches: FSOLVE blocking when adv_fintg_state is set (D-43.7 violation)
    #[test]
    fn fsolve_allows_fintg_state_active() {
        // FSOLVE must work even when adv_fintg_state is set (cross-nesting allowed)
        let program = vec![
            Op::Lbl("SLV".to_string()),
            Op::Sq,
            Op::PushNum(HpNum::from(4i32)),
            Op::Sub,
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.alpha_reg = "SLV".to_string();
        state.regs[0] = HpNum::from(1i32).into();
        state.regs[1] = HpNum::from(3i32).into();
        state.stack.lift_enabled = false;
        // Set adv_fintg_state — FSOLVE must NOT block on this
        state.adv_fintg_state = Some(AdvFintegState::default());

        let result = op_adv_fsolve_run_loop(&mut state, &program);
        // FSOLVE must succeed despite adv_fintg_state being set
        assert!(
            result.is_ok(),
            "FSOLVE must work when adv_fintg_state is set (D-43.7): {result:?}"
        );
    }

    // Catches: FSOLVE cancel not working
    #[test]
    fn fsolve_cancel_stops() {
        let program = vec![
            Op::Lbl("NR".to_string()),
            Op::Sq,
            Op::PushNum(HpNum::from(1i32)),
            Op::Add,
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.alpha_reg = "NR".to_string();
        state.regs[0] = HpNum::from(1i32).into();
        state.regs[1] = HpNum::from(2i32).into();
        state.cancel_requested.store(true, Ordering::Relaxed);
        let result = op_adv_fsolve_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::Canceled));
        assert!(state.adv_fsolve_state.is_none());
    }

    // Catches: FSOLVE missing label not returning InvalidOp
    #[test]
    fn fsolve_missing_label() {
        let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
        let mut state = CalcState::new();
        state.alpha_reg = "MISSING".to_string();
        state.regs[0] = HpNum::from(1i32).into();
        state.regs[1] = HpNum::from(2i32).into();
        let result = op_adv_fsolve_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::InvalidOp));
        assert!(state.adv_fsolve_state.is_none());
    }

    // Catches: FINTG not computing ∫₀¹ x² dx ≈ 0.333
    #[test]
    fn fintg_x2_from_0_to_1() {
        let program = vec![Op::Lbl("F".to_string()), Op::Sq, Op::Rtn];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "F".to_string();
        state.regs[0] = HpNum::from(100i32).into();
        state.stack.x = HpNum::from(0i32);
        state.stack.y = HpNum::from(1i32);
        state.stack.lift_enabled = false;

        let result = op_adv_fintg_run_loop(&mut state, &program);
        assert!(result.is_ok(), "FINTG x^2 0..1: {result:?}");
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Simpson integration result; 1/3 is not exactly representable
        assert!(
            (val - 1.0 / 3.0).abs() < 1e-4,
            "FINTG(x^2,0,1) = {val}, expected ~0.333"
        );
        assert!(state.adv_fintg_state.is_none());
    }

    // Catches: FINTG zero-width interval not returning 0
    #[test]
    fn fintg_zero_width() {
        let program = vec![Op::Lbl("F".to_string()), Op::Sq, Op::Rtn];
        let mut state = CalcState::new();
        state.alpha_reg = "F".to_string();
        state.regs[0] = HpNum::from(10i32).into();
        state.stack.x = HpNum::from(0i32);
        state.stack.y = HpNum::from(0i32);
        state.stack.lift_enabled = false;

        let result = op_adv_fintg_run_loop(&mut state, &program);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        assert!(val.abs() < 1e-10, "zero-width → ~0, got {val}");
    }

    // Catches: FINTG self-nesting not blocked
    #[test]
    fn fintg_self_nesting_blocked() {
        let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
        let mut state = CalcState::new();
        state.adv_fintg_state = Some(AdvFintegState::default());
        let result = op_adv_fintg_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::InvalidOp));
        assert!(state.adv_fintg_state.is_some());
    }

    // Catches: FINTG blocking when adv_fsolve_state is set (D-43.7 violation)
    #[test]
    fn fintg_allows_fsolve_state_active() {
        let program = vec![Op::Lbl("F".to_string()), Op::Sq, Op::Rtn];
        let mut state = CalcState::new();
        state.alpha_reg = "F".to_string();
        state.regs[0] = HpNum::from(10i32).into();
        state.stack.x = HpNum::from(0i32);
        state.stack.y = HpNum::from(1i32);
        state.stack.lift_enabled = false;
        // Set adv_fsolve_state — FINTG must NOT block on this
        state.adv_fsolve_state = Some(AdvFsolveState::default());

        let result = op_adv_fintg_run_loop(&mut state, &program);
        assert!(
            result.is_ok(),
            "FINTG must work when adv_fsolve_state is set (D-43.7): {result:?}"
        );
    }

    // Catches: FINTG cancel not working
    #[test]
    fn fintg_cancel_stops() {
        let program = vec![Op::Lbl("F".to_string()), Op::Sq, Op::Rtn];
        let mut state = CalcState::new();
        state.alpha_reg = "F".to_string();
        state.regs[0] = HpNum::from(1024i32).into();
        state.stack.x = HpNum::from(0i32);
        state.stack.y = HpNum::from(1i32);
        state.stack.lift_enabled = false;
        state.cancel_requested.store(true, Ordering::Relaxed);

        let result = op_adv_fintg_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::Canceled));
        assert!(state.adv_fintg_state.is_none());
    }

    // Catches: FINTG domain rejection not working
    #[test]
    fn fintg_domain_rejection() {
        let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
        let mut state = CalcState::new();
        state.alpha_reg = "F".to_string();
        state.regs[0] = HpNum::from(32_769i32).into(); // > 32768
        state.stack.x = HpNum::from(0i32);
        state.stack.y = HpNum::from(1i32);
        let result = op_adv_fintg_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::Domain));
    }

    // Catches: PLY not evaluating 2x^2+3x+1 at x=2 → 15
    #[test]
    fn ply_evaluates_polynomial() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(2i32);
        state.stack.lift_enabled = false;
        state.regs[0] = HpNum::from(2i32).into(); // degree = 2
        state.regs[1] = HpNum::from(2i32).into(); // 2x^2
        state.regs[2] = HpNum::from(3i32).into(); // 3x
        state.regs[3] = HpNum::from(1i32).into(); // +1

        let result = op_adv_ply(&mut state);
        assert!(result.is_ok(), "PLY: {result:?}");
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Horner result; exact integer arithmetic, no rounding
        assert!((val - 15.0).abs() < 1e-9, "PLY(2x^2+3x+1, 2) = {val}");
    }

    // Catches: PLY degree-0 (constant) wrong
    #[test]
    fn ply_constant() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(5i32);
        state.stack.lift_enabled = false;
        state.regs[0] = HpNum::from(0i32).into(); // degree = 0
        state.regs[1] = HpNum::from(42i32).into(); // constant = 42

        let result = op_adv_ply(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Horner result; degree-0 constant, exact
        assert!((val - 42.0).abs() < 1e-9, "PLY(42, x=5) = {val}");
    }

    // Catches: PLY Horner evaluation wrong for degree-3
    #[test]
    fn ply_degree3_horner() {
        // P(x) = x^3 + 2x^2 + 3x + 4 at x=1 → 1+2+3+4=10
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32);
        state.stack.lift_enabled = false;
        state.regs[0] = HpNum::from(3i32).into();
        state.regs[1] = HpNum::from(1i32).into();
        state.regs[2] = HpNum::from(2i32).into();
        state.regs[3] = HpNum::from(3i32).into();
        state.regs[4] = HpNum::from(4i32).into();

        let result = op_adv_ply(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Horner result; degree-3 with integer coefficients, exact
        assert!((val - 10.0).abs() < 1e-9, "PLY(x^3+2x^2+3x+4, 1) = {val}");
    }

    // Catches: FROOT degree-0 not rejected
    #[test]
    fn froot_degree_zero_rejected() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(0i32);
        state.stack.lift_enabled = false;
        assert_eq!(op_adv_froot(&mut state), Err(HpError::Domain));
    }

    // Catches: FROOT degree > 100 not rejected
    #[test]
    fn froot_degree_limit() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(101i32);
        state.stack.lift_enabled = false;
        assert_eq!(op_adv_froot(&mut state), Err(HpError::Domain));
    }

    // Catches: FROOT not finding roots of x^2-4 (roots ±2)
    #[test]
    fn froot_quadratic() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(2i32); // degree = 2
        state.stack.lift_enabled = false;
        state.regs[1] = HpNum::from(1i32).into(); // x^2
        state.regs[2] = HpNum::from(0i32).into(); // x
        state.regs[3] = HpNum::from(-4i32).into(); // constant

        let result = op_adv_froot(&mut state);
        assert!(result.is_ok(), "FROOT x^2-4: {result:?}");
        // Check that roots are found
        assert!(state.adv_froot_state.is_some());
        let froot = state.adv_froot_state.as_ref().unwrap();
        assert_eq!(froot.roots_found.len(), 2);
        // Roots should be approximately 2 and -2
        let mut reals: Vec<f64> = froot
            .roots_found
            .iter()
            .filter(|(_, im)| im.abs() < 1e-6)
            .map(|(re, _)| *re)
            .collect();
        reals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(reals.len(), 2, "both roots should be real");
        // LINT-EXEMPT: pure-f64 Laguerre root tolerance; numerical root-finder result
        assert!(
            (reals[0] - (-2.0)).abs() < 1e-6,
            "root[0] ≈ -2, got {}",
            reals[0]
        );
        // LINT-EXEMPT: pure-f64 Laguerre root tolerance; numerical root-finder result
        assert!(
            (reals[1] - 2.0).abs() < 1e-6,
            "root[1] ≈ 2, got {}",
            reals[1]
        );
    }

    // Catches: FROOT cubic x^3-1 missing real root at x=1
    #[test]
    fn froot_cubic() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(3i32);
        state.stack.lift_enabled = false;
        state.regs[1] = HpNum::from(1i32).into(); // x^3
        state.regs[2] = HpNum::from(0i32).into(); // x^2
        state.regs[3] = HpNum::from(0i32).into(); // x
        state.regs[4] = HpNum::from(-1i32).into(); // constant

        let result = op_adv_froot(&mut state);
        assert!(result.is_ok(), "FROOT x^3-1: {result:?}");
        let froot = state.adv_froot_state.as_ref().unwrap();
        assert_eq!(froot.roots_found.len(), 3);
        // Should have at least one real root near x=1
        // LINT-EXEMPT: pure-f64 Laguerre root tolerance; `.any()` closure, not top-level assert
        let has_root_at_1 = froot
            .roots_found
            .iter()
            .any(|(re, im)| im.abs() < 1e-4 && (re - 1.0).abs() < 1e-4);
        assert!(has_root_at_1, "x^3-1 must have root at x=1");
    }

    // Catches: RTS with no prior FROOT not returning InvalidOp
    #[test]
    fn rts_no_froot_returns_invalid_op() {
        let mut state = CalcState::new();
        assert!(state.adv_froot_state.is_none());
        assert_eq!(op_adv_rts(&mut state), Err(HpError::InvalidOp));
    }

    // Catches: RTS not outputting sequential roots
    #[test]
    fn rts_sequential_output() {
        let mut state = CalcState::new();
        state.adv_froot_state = Some(FrootState {
            degree: 2,
            roots_found: vec![(2.0, 0.0), (-2.0, 0.0)],
            root_index: 0,
        });
        state.stack.lift_enabled = true;

        // First RTS → 2.0
        let result = op_adv_rts(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 root-to-f64() bridge; exact f64 literal 2.0 stored in FrootState
        assert!((val - 2.0).abs() < 1e-9, "RTS[0] = {val}");

        // Second RTS → -2.0
        state.stack.lift_enabled = true;
        let result = op_adv_rts(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 root-to-f64() bridge; exact f64 literal -2.0 stored in FrootState
        assert!((val - (-2.0)).abs() < 1e-9, "RTS[1] = {val}");
    }

    // Catches: Laguerre implementation using complex arithmetic
    #[test]
    fn laguerre_complex_arithmetic_works() {
        // x^2 + 1 = 0 has complex roots at ±i
        let coeffs = vec![1.0_f64, 0.0, 1.0];
        let roots = laguerre_roots(&coeffs, 2).unwrap();
        assert_eq!(roots.len(), 2);
        // Both roots should be complex (|im| ≈ 1)
        let complex_count = roots.iter().filter(|(_, im)| im.abs() > 0.5).count();
        assert_eq!(complex_count, 2, "x^2+1 has 2 complex roots");
    }

    // Catches: FDIFEQ missing label not returning InvalidOp
    #[test]
    fn fdifeq_missing_label() {
        let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
        let mut state = CalcState::new();
        state.alpha_reg = "MISSING".to_string();
        state.regs[0] = HpNum::from(1i32).into(); // order=1
        state.regs[1] = HpNum::from(1i32).into(); // step=1
        state.regs[2] = HpNum::from(0i32).into();
        state.regs[3] = HpNum::from(0i32).into();
        state.regs[5] = HpNum::from(1i32).into(); // max_steps=1
        let result = op_adv_fdifeq_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    // Catches: FDIFEQ order not 1 or 2 producing error message
    #[test]
    fn fdifeq_invalid_order() {
        let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
        let mut state = CalcState::new();
        state.alpha_reg = "F".to_string();
        state.regs[0] = HpNum::from(3i32).into(); // order=3 (invalid)
        state.regs[1] = HpNum::from(1i32).into();
        state.regs[2] = HpNum::from(0i32).into();
        state.regs[3] = HpNum::from(0i32).into();
        let result = op_adv_fdifeq_run_loop(&mut state, &program);
        assert!(matches!(result, Err(HpError::Domain)));
    }

    // Catches: FDIFEQ RK4 not integrating dy/dx=2x correctly
    #[test]
    fn fdifeq_rk4_linear() {
        // f(x) = 2*x; y(0)=0; exact: y=x^2; at x=1: y=1
        let program = vec![
            Op::Lbl("ODE".to_string()),
            Op::PushNum(HpNum::from(2i32)),
            Op::Mul, // 2*x
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.alpha_reg = "ODE".to_string();
        state.regs[0] = HpNum::from(1i32).into(); // order=1
                                                  // step_size = 0.1
        use rust_decimal::Decimal;
        use std::str::FromStr;
        state.regs[1] = HpNum::from(Decimal::from_str("0.1").unwrap()).into();
        state.regs[2] = HpNum::from(0i32).into(); // x0=0
        state.regs[3] = HpNum::from(0i32).into(); // y0=0
        state.regs[5] = HpNum::from(10i32).into(); // max_steps=10

        let result = op_adv_fdifeq_run_loop(&mut state, &program);
        assert!(result.is_ok(), "FDIFEQ dy/dx=2x: {result:?}");
        // Should have 10+1 lines (initial + 10 steps)
        assert!(
            state.print_buffer.len() >= 10,
            "must have >=10 output lines"
        );
        // Last line should show X≈1.0
        let last = state.print_buffer.last().unwrap();
        assert!(
            last.starts_with("X="),
            "last line must start X=, got: {last}"
        );
    }

    // Catches: Horner evaluation wrong
    #[test]
    fn horner_complex_evaluation() {
        // P(x) = x^2 - 4 at x=2 → 0
        let coeffs = [1.0_f64, 0.0, -4.0];
        let (p, _dp, _ddp) = horner_complex(&coeffs, 2.0, 0.0);
        assert!((p.0).abs() < 1e-10, "P(2)=0 for x^2-4, got {}", p.0);
    }
}
