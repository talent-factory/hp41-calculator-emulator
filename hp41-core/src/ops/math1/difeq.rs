// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! DIFEQ (differential equation) state and implementation.
//!
//! Plan 28-09 fills the full `DifeqState` struct and implements:
//! - `op_difeq(state)` — dispatch arm stub (returns InvalidOp; DIFEQ runs only in run_loop)
//! - `op_difeq_run_loop(state, program)` — real RK4 implementation inside run_loop
//! - `rk4_step_order1(...)` — private helper for ORDER=1 (single-variable ODE)
//! - `rk4_step_order2(...)` — private helper for ORDER=2 (coupled system ODE)
//!
//! ## Architecture — User Callback Re-entrancy (C-28.5 / Pitfall 4)
//!
//! `Op::Difeq` reuses INTG's user-program callback infrastructure verbatim:
//! same run_loop re-entry via `call_stack.push + run_user_function`, same
//! strict-reject (XROM-08 FINAL 3-state form), same pre-mutation call_stack cap.
//! CROSS-REFERENCE: see Plan 28-07 op_integ_run_loop for the symmetric pattern.
//!
//! ## 4th-Order Runge-Kutta Algorithm (DIFEQ-02 / OM Chapter 7)
//!
//! For ORDER=1 ODE y' = f(x, y):
//!   k1 = h · f(x_n, y_n)
//!   k2 = h · f(x_n + h/2, y_n + k1/2)
//!   k3 = h · f(x_n + h/2, y_n + k2/2)
//!   k4 = h · f(x_n + h, y_n + k3)
//!   y_{n+1} = y_n + (k1 + 2·k2 + 2·k3 + k4) / 6
//!
//! For ORDER=2 ODE y'' = f(x, y, y'), reduced to coupled system y'=z, z'=f(x,y,z):
//!   (HP-41C Math Pac I OM, Chapter 7, pp. 43-50 — coupled RK4 for 2nd-order ODEs)
//!   k1y = h · z_n
//!   k1z = h · f(x_n, y_n, z_n)
//!   k2y = h · (z_n + k1z/2)
//!   k2z = h · f(x_n + h/2, y_n + k1y/2, z_n + k1z/2)
//!   k3y = h · (z_n + k2z/2)
//!   k3z = h · f(x_n + h/2, y_n + k2y/2, z_n + k2z/2)
//!   k4y = h · (z_n + k3z)
//!   k4z = h · f(x_n + h, y_n + k3y, z_n + k3z)
//!   y_{n+1} = y_n + (k1y + 2·k2y + 2·k3y + k4y) / 6
//!   z_{n+1} = z_n + (k1z + 2·k2z + 2·k3z + k4z) / 6
//!
//! Each k_z evaluation is one run_loop re-entry on user_label with
//! (X=x_intermediate, Y=y_intermediate, Z=z_intermediate for ORDER=2).
//! k_y evaluations are pure arithmetic (no user-LBL call needed).
//!
//! ## Scratch Register Convention (RESEARCH Open Q6 / DIFEQ-03)
//!
//! BL-03 fix: the documented layout below matches the code path actually used
//! by `op_difeq_run_loop` (see lines 209-225) and `submit_step` (lines 754-827).
//! The previous "R00 (x), R01 (y), R02 (y'), R03 (step_size), R04..R07 (k1..k4)"
//! comment contradicted the implementation — RK4 intermediates k1..k4 live in
//! local Rust variables, NOT in scratch registers.
//!
//! ```text
//! R00 = ODE order (1 or 2; DIFEQ-01)
//! R01 = step size h
//! R02 = x0 (initial x)
//! R03 = y0 (initial y)
//! R04 = y'0 (initial y'; only for ORDER=2)
//! R05 = max_steps (RK4 step budget; defaults to 1000 when R05 is 0)
//! ```
//!
//! User-program-clobber of R00..R05 is documented user-responsibility per
//! RESEARCH Open Q6 (same as INTG/SOLVE). Emulator uses local Rust variables
//! for the RK4 k₁..k₄ intermediates; the OM-documented "user function must
//! not write to scratch" advice applies to R00..R05 inclusive.
//!
//! ## Step-by-Step Output (DIFEQ-05)
//!
//! Each RK4 step pushes formatted lines to state.print_buffer:
//! - ORDER=1: "X=<v> Y=<v>"
//! - ORDER=2: "X=<v> Y=<v> Y'=<v>"
//!
//! ## Cancellation (D-28.7 / D-28.8)
//!
//! Every 64 steps: check state.cancel_requested. If true, clear difeq_state and
//! return HpError::Canceled. GUI wiring ships Phase 31 / GUI-05.

use std::sync::atomic::Ordering;

use crate::error::HpError;
use crate::format::format_hpnum;
use crate::num::HpNum;
use crate::ops::Op;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

// ── DifeqState struct ─────────────────────────────────────────────────────────

/// Mid-iteration state for the DIFEQ differential-equation solver (RK4).
///
/// Fields are populated when `Op::Difeq` begins and cleared when it completes
/// (success or cancellation). `CalcState.difeq_state` holds this as `Option<DifeqState>`.
///
/// The `#[serde(skip)]` on `CalcState::difeq_state` means this struct is never
/// persisted — it is purely transient state for the duration of integration.
///
/// Scratch register note (DIFEQ-03): R00–R07 are scratch during DIFEQ per OM convention.
/// The emulator does NOT snapshot/restore these registers. Hardware-faithful behavior.
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 7.
/// PATTERNS line 583: { user_label, order, step_size, x, y, y_prime, step_count }.
#[derive(Debug, Clone, Default)]
pub struct DifeqState {
    /// XEQ label of the user-supplied ODE right-hand-side function.
    pub user_label: String,
    /// ODE order: 1 (single-variable) or 2 (coupled system). DIFEQ-01.
    pub order: u8,
    /// Step size h (integration step). From STEP SIZE=? prompt.
    pub step_size: HpNum,
    /// Current x value (independent variable). Advances by step_size each RK4 step.
    pub x: HpNum,
    /// Current y value (primary solution). Updated each RK4 step.
    pub y: HpNum,
    /// Current y' value (derivative). `Some(z)` when order=2; `None` when order=1.
    pub y_prime: Option<HpNum>,
    /// Step counter — used for per-64-steps cancellation check (D-28.7 / D-28.8).
    pub step_count: u32,
    /// Maximum number of RK4 steps to take before stopping.
    /// Read from R05 at entry (integer part). If 0 or unset, defaults to 1000.
    /// Phase 29 / CLI-08 wires this to the "N STEPS=?" modal parameter.
    /// Allowing user control prevents runaway loops for ODEs with large x-ranges.
    pub max_steps: u32,
}

// ── op_difeq (dispatch arm — REJECT) ─────────────────────────────────────────

/// Dispatch arm for `Op::Difeq` (interactive / dispatch() call).
///
/// Returns `Err(HpError::InvalidOp)` unconditionally.
/// `Op::Difeq` can only run inside `run_loop`; the real implementation is
/// `op_difeq_run_loop`. This mirrors the `Op::XeqInd` precedent, `Op::Integ`
/// pattern from Plan 28-07, and `Op::Solve` pattern from Plan 28-08.
///
/// XROM resolver routes `XEQ "DIFEQ"` through `dispatch()` which calls here.
/// Only when DIFEQ appears inside a *running program* is it routed through
/// `run_loop`'s match arm → `op_difeq_run_loop`.
pub fn op_difeq(state: &mut CalcState) -> Result<(), HpError> {
    // Phase 29 / CLI-07 — additive completion of Phase 28 stub.
    // The Phase 28 stub anticipated this wiring (symmetric with op_solve/op_integ pattern).
    if !state.is_running {
        // Plan 31-02 surgical hp41-core exception — idempotency invariant (T-31-W1-sticky-cancel):
        // Reset cancel_requested to false at workflow-open time. Symmetric with op_integ / op_solve.
        // The user has just pressed the DIFEQ key interactively — the previous run (if any)
        // is complete or canceled; sticky cancel_requested = true would abort the new run.
        state
            .cancel_requested
            .store(false, std::sync::atomic::Ordering::Relaxed);
        // Interactive: open the DIFEQ modal at FunctionNamePrompt.
        // CLI auto-open hook will fire CollectForModal after this returns (D-29.9).
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Difeq(
            crate::ops::math1::modal::DifeqInputStep::FunctionNamePrompt,
        ));
        state.modal_prompt = Some("FUNCTION NAME?".to_string());
        return Ok(());
    }
    // Op::Difeq inside run_loop: real implementation in op_difeq_run_loop.
    Err(HpError::InvalidOp)
}

// ── op_difeq_run_loop (real implementation) ───────────────────────────────────

/// Real DIFEQ implementation — called from `run_loop`'s `Op::Difeq` match arm.
///
/// ## Entry contract
///
/// `state.difeq_state` is `None` at entry — the run_loop arm is responsible for
/// routing here. This function sets `difeq_state = Some(...)` AFTER all pre-mutation
/// guards pass.
///
/// ## Guard order (pre-mutation, MUST check BEFORE any state mutation)
///
/// 1. Strict-reject nested callback (XROM-08 FINAL form / ADR-002 / D-28.7):
///    `state.integ_state.is_some() || state.solve_state.is_some() || state.difeq_state.is_some()`
///    → `HpError::InvalidOp` BEFORE any state change.
///    This is the canonical 3-state guard cited in D-28.7. Plan 28-09 is the LAST plan
///    to grow this guard; subsequent plans inherit it unchanged (PATTERNS lines 531-534).
/// 2. Pre-mutation call_stack cap (Pitfall 4 / D-22.15 generalized):
///    `state.call_stack.len() >= 4` → `HpError::CallDepth` BEFORE any state change.
///
/// ## Modal parameter convention (symmetric with Plan 28-07 INTG / Plan 28-08 SOLVE)
///
/// For Plan 28-09 tests (Phase 29 / CLI wiring deferred): we read:
/// - user_label from `state.alpha_reg` (OM: function label in ALPHA)
/// - order from `state.regs[0]` integer part (1 or 2; DIFEQ-01)
/// - step_size (h) from `state.regs[1]`
/// - x0 from `state.regs[2]`
/// - y0 from `state.regs[3]`
/// - y_prime0 from `state.regs[4]` (only when order=2)
///
/// ORDER validation: only 1 and 2 accepted. Any other value writes
/// `state.modal_prompt = Some("ORDER MUST BE 1 OR 2")` and returns `Ok(())` (NOT an HpError).
/// This matches the OM behavior of prompting "invalid order" without crashing.
///
/// CROSS-REFERENCE: see Plan 28-07 op_integ_run_loop for the symmetric pattern.
/// Phase 29 / CLI-08 wires the full DifeqInputStep modal flow.
///
/// ## RK4 loop
///
/// The solver runs indefinitely until cancellation (D-28.7) or until an external
/// trigger (planned for Phase 29 / CLI-08 "number of steps" modal parameter).
/// For tests, a MAX_STEPS outer budget of 10_000 applies as a safety cap.
pub fn op_difeq_run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    // ── Pre-mutation guards (MUST run before any state mutation) ──────────────
    // Guard 1 (XROM-08 FINAL form / ADR-002 / D-28.7): strict-reject nested user-callback.
    // This is the FINAL canonical 3-state guard: integ OR solve OR difeq → reject.
    // Plan 28-09 is the last plan to grow this guard; unchanged in subsequent plans.
    if state.integ_state.is_some() || state.solve_state.is_some() || state.difeq_state.is_some() {
        return Err(HpError::InvalidOp);
    }
    // Guard 2 (Pitfall 4 / D-22.15): pre-mutation call_stack cap
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }

    // ── Read modal-staged parameters (Plan 28-07 symmetric convention) ────────
    // Phase 29 / CLI-08 wires the full DifeqInputStep modal flow that stages these
    // into the same registers before calling run_loop. For Plan 28-09 tests, we
    // read directly from the register/alpha convention:
    //   alpha_reg = user function label
    //   R00 = order (integer part: 1 or 2)
    //   R01 = step size h
    //   R02 = x0
    //   R03 = y0
    //   R04 = y_prime0 (only relevant when order=2)
    let user_label = state.alpha_reg.clone();
    let order_raw = state
        .regs
        .first()
        .map(|r| r.inner().to_u8().unwrap_or(0))
        .unwrap_or(0);
    let step_size_val = state.regs.get(1).cloned().unwrap_or_default();
    let x0 = state.regs.get(2).cloned().unwrap_or_default();
    let y0 = state.regs.get(3).cloned().unwrap_or_default();
    let y_prime0 = state.regs.get(4).cloned().unwrap_or_default();
    // R05 = max_steps (integer part; 0 or unset → default 1000)
    // Phase 29 / CLI-08 wires this to the "N STEPS=?" modal parameter.
    let max_steps_raw = state
        .regs
        .get(5)
        .map(|r| r.inner().to_u32().unwrap_or(0))
        .unwrap_or(0);
    let max_steps = if max_steps_raw == 0 {
        1000u32
    } else {
        max_steps_raw
    };

    // ORDER validation: only 1 and 2 are accepted per DIFEQ-01.
    // Any other value writes modal_prompt and returns Ok(()) (NOT an HpError).
    if order_raw != 1 && order_raw != 2 {
        state.modal_prompt = Some("ORDER MUST BE 1 OR 2".to_string());
        return Ok(());
    }
    let order = order_raw;

    // ── Commit: set difeq_state after all pre-mutation guards pass ────────────
    state.difeq_state = Some(DifeqState {
        user_label: user_label.clone(),
        order,
        step_size: step_size_val.clone(),
        x: x0.clone(),
        y: y0.clone(),
        y_prime: if order == 2 {
            Some(y_prime0.clone())
        } else {
            None
        },
        step_count: 0,
        max_steps,
    });

    // Find the user label in the program BEFORE starting the loop
    // (fail fast with InvalidOp if label not found)
    let label_pos = program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if *l == user_label))
        .ok_or_else(|| {
            state.difeq_state = None;
            HpError::InvalidOp
        })?;

    // ── Push initial step to print_buffer (DIFEQ-05) ─────────────────────────
    let x_str = format_hpnum(&x0, &state.display_mode);
    let y_str = format_hpnum(&y0, &state.display_mode);
    let initial_line = if order == 2 {
        let yp_str = format_hpnum(&y_prime0, &state.display_mode);
        format!("X={x_str} Y={y_str} Y'={yp_str}")
    } else {
        format!("X={x_str} Y={y_str}")
    };
    state.print_buffer.push(initial_line);

    // ── RK4 iteration loop ────────────────────────────────────────────────────
    // Runs until cancellation or MAX_STEPS safety cap.
    // The max_steps field in DifeqState controls loop termination.
    // Phase 29 / CLI-08 wires this to a user-configurable "N STEPS=?" parameter.
    let save_pc = state.pc;
    let save_call_stack_len = state.call_stack.len();
    // Capture display_mode before the mutable borrow of state in the loop
    let display_mode = state.display_mode;

    loop {
        let step_count = {
            let st = state.difeq_state.as_ref().expect("set above");
            st.step_count
        };

        // Per-64-steps cancellation check (D-28.7 / D-28.8)
        if step_count & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
            state.difeq_state = None;
            state.pc = save_pc;
            return Err(HpError::Canceled);
        }

        // Stop when max_steps reached (user-configurable via R05 / modal parameter)
        let current_max = state.difeq_state.as_ref().expect("set above").max_steps;
        if step_count >= current_max {
            // Push final state to difeq_state before clearing (leaves result accessible)
            state.difeq_state = None;
            state.pc = save_pc;
            return Ok(());
        }

        // Read current state for this RK4 step
        let (x_n, y_n, z_n_opt, h) = {
            let st = state.difeq_state.as_ref().expect("set above");
            (
                st.x.clone(),
                st.y.clone(),
                st.y_prime.clone(),
                st.step_size.clone(),
            )
        };

        // ── RK4 computation ───────────────────────────────────────────────────
        let result = if order == 1 {
            // ORDER=1: y' = f(x, y) — 4 user-callback re-entries
            // CROSS-REFERENCE: see Plan 28-07 op_integ_run_loop for the symmetric
            // user-callback pattern used here.
            match rk4_step_order1(state, program, label_pos, &x_n, &y_n, &h) {
                Ok(y_new) => {
                    // Advance x by h
                    let x_new = HpNum::from(
                        Decimal::from_f64(
                            x_n.inner().to_f64().ok_or(HpError::Overflow)?
                                + h.inner().to_f64().ok_or(HpError::Overflow)?,
                        )
                        .ok_or(HpError::Overflow)?,
                    );
                    // Update state
                    if let Some(ref mut st) = state.difeq_state {
                        st.x = x_new.clone();
                        st.y = y_new.clone();
                        st.step_count += 1;
                    }
                    // Push step output (DIFEQ-05)
                    let line = format!(
                        "X={} Y={}",
                        format_hpnum(&x_new, &display_mode),
                        format_hpnum(&y_new, &display_mode)
                    );
                    state.print_buffer.push(line);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            // ORDER=2: y'' = f(x, y, y'), reduced to coupled system y'=z, z'=f(x,y,z)
            // CROSS-REFERENCE: see Plan 28-07 op_integ_run_loop for the symmetric
            // user-callback pattern used here.
            let z_n = z_n_opt.clone().unwrap_or_default();
            match rk4_step_order2(state, program, label_pos, &x_n, &y_n, &z_n, &h) {
                Ok((y_new, z_new)) => {
                    let x_new = HpNum::from(
                        Decimal::from_f64(
                            x_n.inner().to_f64().ok_or(HpError::Overflow)?
                                + h.inner().to_f64().ok_or(HpError::Overflow)?,
                        )
                        .ok_or(HpError::Overflow)?,
                    );
                    // Update state
                    if let Some(ref mut st) = state.difeq_state {
                        st.x = x_new.clone();
                        st.y = y_new.clone();
                        st.y_prime = Some(z_new.clone());
                        st.step_count += 1;
                    }
                    // Push step output (DIFEQ-05) with Y' component
                    let line = format!(
                        "X={} Y={} Y'={}",
                        format_hpnum(&x_new, &display_mode),
                        format_hpnum(&y_new, &display_mode),
                        format_hpnum(&z_new, &display_mode)
                    );
                    state.print_buffer.push(line);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        };

        match result {
            Ok(()) => {}
            Err(e) => {
                state.difeq_state = None;
                state.pc = save_pc;
                // Restore call stack
                while state.call_stack.len() > save_call_stack_len {
                    state.call_stack.pop();
                }
                return Err(e);
            }
        }
    }
    // Note: this loop is bounded by MAX_STEPS safety cap above.
    // Phase 29 / CLI-08 will wire a user-configurable step count.
}

// ── rk4_step_order1: ORDER=1 RK4 helper ──────────────────────────────────────

/// Compute one RK4 step for ORDER=1 ODE y' = f(x, y).
///
/// 4 user-callback re-entries (k1..k4 evaluations), each pushing (x, y) to stack
/// and reading f(x, y) from X after run_user_function returns.
///
/// Returns y_{n+1} = y_n + (k1 + 2·k2 + 2·k3 + k4) / 6.
///
/// CROSS-REFERENCE: mirrors op_integ_run_loop user-callback re-entry pattern (Plan 28-07).
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 7, p. 43.
fn rk4_step_order1(
    state: &mut CalcState,
    program: &[Op],
    label_pos: usize,
    x_n: &HpNum,
    y_n: &HpNum,
    h: &HpNum,
) -> Result<HpNum, HpError> {
    let save_call_stack_len = state.call_stack.len();
    let x_f64 = x_n.inner().to_f64().ok_or(HpError::Overflow)?;
    let y_f64 = y_n.inner().to_f64().ok_or(HpError::Overflow)?;
    let h_f64 = h.inner().to_f64().ok_or(HpError::Overflow)?;

    // k1 = h · f(x_n, y_n)
    // CROSS-REFERENCE: see Plan 28-07 op_integ_run_loop for the symmetric re-entry pattern.
    let k1 = {
        push_two_args(state, x_f64, y_f64)?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // k2 = h · f(x_n + h/2, y_n + k1/2)
    let k2 = {
        push_two_args(state, x_f64 + h_f64 / 2.0, y_f64 + k1 / 2.0)?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // k3 = h · f(x_n + h/2, y_n + k2/2)
    let k3 = {
        push_two_args(state, x_f64 + h_f64 / 2.0, y_f64 + k2 / 2.0)?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // k4 = h · f(x_n + h, y_n + k3)
    let k4 = {
        push_two_args(state, x_f64 + h_f64, y_f64 + k3)?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // y_{n+1} = y_n + (k1 + 2·k2 + 2·k3 + k4) / 6
    let y_new_f64 = y_f64 + (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
    Ok(HpNum::from(
        Decimal::from_f64(y_new_f64).ok_or(HpError::Overflow)?,
    ))
}

// ── rk4_step_order2: ORDER=2 RK4 helper ──────────────────────────────────────

/// Compute one RK4 step for ORDER=2 ODE y'' = f(x, y, y') via coupled system.
///
/// Reduction: y' = z, z' = f(x, y, z) — standard system form for 2nd-order ODEs.
/// The coupled RK4 updates both y and z (= y') simultaneously.
///
/// Source: HP-41C Math Pac I OM (HP 00041-90034, 1979), Chapter 7, pp. 43-50
/// (2nd-order coupled RK4 for y'' = f(x, y, y')).
///
/// ## Coupled RK4 slope pairs
///
/// k1y = h · z_n                          (no user call — pure arithmetic)
/// k1z = h · f(x_n, y_n, z_n)             (1 user call)
/// k2y = h · (z_n + k1z/2)               (no user call — pure arithmetic)
/// k2z = h · f(x_n+h/2, y_n+k1y/2, z_n+k1z/2)  (1 user call)
/// k3y = h · (z_n + k2z/2)               (no user call — pure arithmetic)
/// k3z = h · f(x_n+h/2, y_n+k2y/2, z_n+k2z/2)  (1 user call)
/// k4y = h · (z_n + k3z)                 (no user call — pure arithmetic)
/// k4z = h · f(x_n+h, y_n+k3y, z_n+k3z)         (1 user call)
///
/// y_{n+1} = y_n + (k1y + 2·k2y + 2·k3y + k4y) / 6
/// z_{n+1} = z_n + (k1z + 2·k2z + 2·k3z + k4z) / 6
///
/// Each k_z evaluation is one run_loop re-entry on user_label with
/// (X=x_intermediate, Y=y_intermediate, Z=z_intermediate in stack).
/// User LBL returns f(x,y,z) = y'' in X.
///
/// CROSS-REFERENCE: mirrors op_integ_run_loop user-callback re-entry pattern (Plan 28-07).
fn rk4_step_order2(
    state: &mut CalcState,
    program: &[Op],
    label_pos: usize,
    x_n: &HpNum,
    y_n: &HpNum,
    z_n: &HpNum,
    h: &HpNum,
) -> Result<(HpNum, HpNum), HpError> {
    let save_call_stack_len = state.call_stack.len();
    let x_f64 = x_n.inner().to_f64().ok_or(HpError::Overflow)?;
    let y_f64 = y_n.inner().to_f64().ok_or(HpError::Overflow)?;
    let z_f64 = z_n.inner().to_f64().ok_or(HpError::Overflow)?;
    let h_f64 = h.inner().to_f64().ok_or(HpError::Overflow)?;

    // k1y = h · z_n  (pure arithmetic — no user call)
    let k1y = h_f64 * z_f64;

    // k1z = h · f(x_n, y_n, z_n)  (1 user call; user LBL receives x in X, y in Y, z in Z)
    // CROSS-REFERENCE: see Plan 28-07 op_integ_run_loop for the symmetric re-entry pattern.
    let k1z = {
        push_three_args(state, x_f64, y_f64, z_f64)?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // k2y = h · (z_n + k1z/2)  (pure arithmetic)
    let k2y = h_f64 * (z_f64 + k1z / 2.0);

    // k2z = h · f(x_n+h/2, y_n+k1y/2, z_n+k1z/2)  (1 user call)
    let k2z = {
        push_three_args(
            state,
            x_f64 + h_f64 / 2.0,
            y_f64 + k1y / 2.0,
            z_f64 + k1z / 2.0,
        )?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // k3y = h · (z_n + k2z/2)  (pure arithmetic)
    let k3y = h_f64 * (z_f64 + k2z / 2.0);

    // k3z = h · f(x_n+h/2, y_n+k2y/2, z_n+k2z/2)  (1 user call)
    let k3z = {
        push_three_args(
            state,
            x_f64 + h_f64 / 2.0,
            y_f64 + k2y / 2.0,
            z_f64 + k2z / 2.0,
        )?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // k4y = h · (z_n + k3z)  (pure arithmetic)
    let k4y = h_f64 * (z_f64 + k3z);

    // k4z = h · f(x_n+h, y_n+k3y, z_n+k3z)  (1 user call)
    let k4z = {
        push_three_args(state, x_f64 + h_f64, y_f64 + k3y, z_f64 + k3z)?;
        state.call_stack.push(state.pc);
        state.pc = label_pos + 1;
        let r = run_user_function(state, program);
        while state.call_stack.len() > save_call_stack_len {
            state.call_stack.pop();
        }
        r?;
        let f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;
        h_f64 * f
    };

    // y_{n+1} = y_n + (k1y + 2·k2y + 2·k3y + k4y) / 6
    let y_new_f64 = y_f64 + (k1y + 2.0 * k2y + 2.0 * k3y + k4y) / 6.0;
    // z_{n+1} = z_n + (k1z + 2·k2z + 2·k3z + k4z) / 6
    let z_new_f64 = z_f64 + (k1z + 2.0 * k2z + 2.0 * k3z + k4z) / 6.0;

    Ok((
        HpNum::from(Decimal::from_f64(y_new_f64).ok_or(HpError::Overflow)?),
        HpNum::from(Decimal::from_f64(z_new_f64).ok_or(HpError::Overflow)?),
    ))
}

// ── run_user_function: sub-loop for user callback evaluation ──────────────────

/// Execute the user callback function starting at the current `state.pc`.
///
/// This mirrors `run_user_function` from `integ.rs` (Plan 28-07) exactly.
/// A local sub-loop that steps through the program until:
/// - RTN with empty call_stack (function returned)
/// - End of program
/// - STOP (halt)
/// - Error
///
/// The `state.call_stack` was already pushed by the caller before calling here.
/// When `RTN` pops back to the saved PC (from the difeq loop), execution stops.
///
/// CROSS-REFERENCE: see Plan 28-07 integ.rs::run_user_function for the symmetric pattern.
/// Plan 28-08 solve.rs uses the identical local copy pattern.
fn run_user_function(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    use crate::ops::program::execute_op_pub;

    let entry_depth = state.call_stack.len();
    let mut steps: u64 = 0;
    // WR-07: shared cap in crate::ops::math1::USER_CALLBACK_MAX_STEPS
    use crate::ops::math1::USER_CALLBACK_MAX_STEPS;

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
            Op::Rtn => {
                match state.call_stack.pop() {
                    Some(return_pc) => {
                        state.pc = return_pc;
                        if state.call_stack.len() < entry_depth {
                            break;
                        }
                    }
                    None => break, // top-level RTN
                }
            }
            Op::Lbl(_) => {} // LBL is a marker only — no-op during execution
            Op::Stop => break,
            other => {
                execute_op_pub(state, other)?;
            }
        }
    }

    Ok(())
}

// ── Stack helpers ─────────────────────────────────────────────────────────────

/// Push two args (x, y) to HP stack for ORDER=1 user-callback calls.
/// After push: X = x_arg, Y = y_arg (user function reads f(x, y) from X, Y).
fn push_two_args(state: &mut CalcState, x_arg: f64, y_arg: f64) -> Result<(), HpError> {
    // Push y_arg first, then x_arg — HP stack is LIFO: last push ends up in X
    state.stack.lift_enabled = true;
    enter_number(
        state,
        HpNum::from(Decimal::from_f64(y_arg).ok_or(HpError::Overflow)?),
    );
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(
        state,
        HpNum::from(Decimal::from_f64(x_arg).ok_or(HpError::Overflow)?),
    );
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// Push three args (x, y, z) to HP stack for ORDER=2 user-callback calls.
/// After push: X = x_arg, Y = y_arg, Z = z_arg (user function reads f(x,y,z)).
fn push_three_args(
    state: &mut CalcState,
    x_arg: f64,
    y_arg: f64,
    z_arg: f64,
) -> Result<(), HpError> {
    // Push z first, y second, x last — HP stack LIFO: last push ends up in X
    state.stack.lift_enabled = true;
    enter_number(
        state,
        HpNum::from(Decimal::from_f64(z_arg).ok_or(HpError::Overflow)?),
    );
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(
        state,
        HpNum::from(Decimal::from_f64(y_arg).ok_or(HpError::Overflow)?),
    );
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(
        state,
        HpNum::from(Decimal::from_f64(x_arg).ok_or(HpError::Overflow)?),
    );
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── Phase 29 / CLI-05 additive public surface — D-29.5 ───────────────────────

/// Submit a numeric input step in the DIFEQ modal workflow.
///
/// Called by `hp41_core::ops::math1::submit_modal` after `flush_entry_buf` has
/// flushed the entry buffer to `state.stack.x`. Reads X, advances the DIFEQ
/// modal step state machine, updates `state.modal_prompt`.
///
/// Step transitions (DIFEQ prompt sequence per HP-41C Math Pac I OM, Chapter 7):
/// - `OrderPrompt` → reads X as ODE order (1 or 2), stores in R00,
///   advances to StepSizePrompt.
/// - `StepSizePrompt` → reads X as step size h, stores in R01,
///   advances to X0Prompt.
/// - `X0Prompt` → reads X as initial x0, stores in R02, advances to Y0Prompt.
/// - `Y0Prompt` → reads X as initial y0, stores in R03. If order=2, advances to
///   Y1PrimePrompt; else advances to Ready.
/// - `Y1PrimePrompt` → reads X as y'0, stores in R04, advances to Ready.
/// - All other steps → `Err(HpError::InvalidOp)`.
///
/// Phase 29 / CLI-05 additive public surface — D-29.5.
pub fn submit_step(
    state: &mut CalcState,
    step: crate::ops::math1::modal::DifeqInputStep,
) -> Result<(), HpError> {
    use crate::ops::math1::modal::{DifeqInputStep, ModalProgram};
    match step {
        DifeqInputStep::OrderPrompt => {
            // WR-03 fix: do NOT silently clamp the raw value to {1, 2}. Let
            // out-of-range values reach op_difeq_run_loop which reports
            // "ORDER MUST BE 1 OR 2" via modal_prompt — explicit feedback
            // beats silent coercion. A user typing `3` for a 3rd-order ODE
            // previously got a 2nd-order run with no warning; now they get
            // a clear rejection at solver-start.
            let order = state.stack.x.inner().to_u8().unwrap_or(0);
            if state.regs.is_empty() {
                return Err(HpError::InvalidOp);
            }
            state.regs[0] = crate::num::HpNum::from(order as i32);
            state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::StepSizePrompt));
            state.modal_prompt = Some("STEP SIZE=?".to_string());
            Ok(())
        }
        DifeqInputStep::StepSizePrompt => {
            if state.regs.len() < 2 {
                return Err(HpError::InvalidOp);
            }
            state.regs[1] = state.stack.x.clone();
            state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::X0Prompt));
            state.modal_prompt = Some("X0=?".to_string());
            Ok(())
        }
        DifeqInputStep::X0Prompt => {
            if state.regs.len() < 3 {
                return Err(HpError::InvalidOp);
            }
            state.regs[2] = state.stack.x.clone();
            state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::Y0Prompt));
            state.modal_prompt = Some("Y0=?".to_string());
            Ok(())
        }
        DifeqInputStep::Y0Prompt => {
            if state.regs.len() < 4 {
                return Err(HpError::InvalidOp);
            }
            state.regs[3] = state.stack.x.clone();
            // Check order from R00 to decide if Y'0 is needed
            let order = state.regs[0].inner().to_u8().unwrap_or(1);
            if order == 2 {
                state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::Y1PrimePrompt));
                state.modal_prompt = Some("Y'0=?".to_string());
            } else {
                state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::Ready));
                state.modal_prompt = None;
            }
            Ok(())
        }
        DifeqInputStep::Y1PrimePrompt => {
            if state.regs.len() < 5 {
                return Err(HpError::InvalidOp);
            }
            state.regs[4] = state.stack.x.clone();
            state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::Ready));
            state.modal_prompt = None;
            Ok(())
        }
        DifeqInputStep::FunctionNamePrompt | DifeqInputStep::Ready => {
            // FunctionNamePrompt handled by submit_label_step; Ready has no submission.
            Err(HpError::InvalidOp)
        }
    }
}

/// Submit the function label step for the DIFEQ modal workflow.
///
/// Called by `hp41_core::ops::math1::submit_modal_with_label` when the modal is
/// at `DifeqInputStep::FunctionNamePrompt`. The label has already been written to
/// `state.alpha_reg` before this is called.
///
/// Advances the modal from `FunctionNamePrompt` to `OrderPrompt` and sets
/// `modal_prompt = Some("ORDER=?")`.
///
/// Phase 29 / CLI-05 additive public surface — D-29.5.
pub fn submit_label_step(state: &mut CalcState) -> Result<(), HpError> {
    use crate::ops::math1::modal::{DifeqInputStep, ModalProgram};
    state.modal_program = Some(ModalProgram::Difeq(DifeqInputStep::OrderPrompt));
    state.modal_prompt = Some("ORDER=?".to_string());
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;
    use rust_decimal::prelude::ToPrimitive;

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Build a CalcState for ORDER=1 DIFEQ with f(x, y) = y (exponential growth).
    /// dy/dx = y, y0=1, x0=0, h=0.1 → y(1) ≈ e ≈ 2.71828...
    /// Analytical solution: y(x) = e^x
    ///
    /// `n_steps` controls the number of RK4 steps to take (via R05 = max_steps).
    fn make_exp_growth_state(n_steps: u32) -> (CalcState, Vec<Op>) {
        // Program: LBL "EG" / (f(x,y) = y; y is in Y register after push_two_args) / RTN
        // ORDER=1: user LBL receives x in X, y in Y; returns f(x,y)=y in X
        // To return y: swap X and Y, then RTN
        let program = vec![
            Op::Lbl("EG".to_string()),
            Op::XySwap, // bring y to X (since push_two_args puts x in X, y in Y)
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "EG".to_string();
        // R00 = order (1), R01 = h, R02 = x0, R03 = y0, R05 = max_steps
        state.regs[0] = HpNum::from(1i32); // order = 1
        state.regs[1] = HpNum::from(
            rust_decimal::Decimal::from_f64(0.1).unwrap_or(rust_decimal::Decimal::ZERO),
        ); // h = 0.1
        state.regs[2] = HpNum::from(0i32); // x0 = 0
        state.regs[3] = HpNum::from(1i32); // y0 = 1
                                           // R05 = max_steps: controls how many RK4 steps to take before stopping
        state.regs[5] = HpNum::from(n_steps as i32); // number of steps
        (state, program)
    }

    /// Build a CalcState for ORDER=1 DIFEQ with f(x, y) = -y (exponential decay).
    /// dy/dx = -y, y0=1, x0=0, h=0.1 → y(1) ≈ 1/e ≈ 0.36788...
    fn make_exp_decay_state() -> (CalcState, Vec<Op>) {
        // f(x,y) = -y: swap X,Y then CHS (negate), then RTN
        let program = vec![
            Op::Lbl("ED".to_string()),
            Op::XySwap, // y → X
            Op::Chs,    // negate: -y
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "ED".to_string();
        state.regs[0] = HpNum::from(1i32);
        state.regs[1] = HpNum::from(
            rust_decimal::Decimal::from_f64(0.1).unwrap_or(rust_decimal::Decimal::ZERO),
        );
        state.regs[2] = HpNum::from(0i32);
        state.regs[3] = HpNum::from(1i32);
        state.regs[5] = HpNum::from(20i32); // max_steps = 20 (more than 5 needed for test)
        (state, program)
    }

    /// Build a CalcState for ORDER=2 DIFEQ with y'' = -y (harmonic oscillator).
    /// y0=1, y'0=0, h=0.1 → y(1) ≈ cos(1) ≈ 0.5403...
    fn make_harmonic_state() -> (CalcState, Vec<Op>) {
        // ORDER=2: user LBL receives x in X, y in Y, y' in Z; returns f(x,y,y')=y'' in X
        // f = -y → negate y: need to get y from Y register
        // Stack: X=x, Y=y, Z=z. f=-y means we need: XY swap to get y→X, CHS to negate, RTN
        let program = vec![
            Op::Lbl("HO".to_string()),
            Op::XySwap, // y → X (x → Y)
            Op::Chs,    // -y
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "HO".to_string();
        state.regs[0] = HpNum::from(2i32); // order = 2
        state.regs[1] = HpNum::from(
            rust_decimal::Decimal::from_f64(0.1).unwrap_or(rust_decimal::Decimal::ZERO),
        );
        state.regs[2] = HpNum::from(0i32); // x0 = 0
        state.regs[3] = HpNum::from(1i32); // y0 = 1
        state.regs[4] = HpNum::from(0i32); // y'0 = 0
        state.regs[5] = HpNum::from(20i32); // max_steps = 20 (at least 10 for the correctness test)
        (state, program)
    }

    // ── op_difeq dispatch stub test ───────────────────────────────────────────

    // Catches: op_difeq interactive branch (Phase 29 completion) not opening modal at
    // FunctionNamePrompt when called interactively (!is_running).
    // Phase 29 / CLI-08: op_difeq now opens a modal when !is_running, per the documented
    // Phase 28 stub design (symmetric with op_solve and op_integ completion pattern).
    #[test]
    fn op_difeq_dispatch_opens_modal_when_interactive() {
        let mut state = CalcState::new();
        // is_running = false by default (interactive mode)
        let result = op_difeq(&mut state);
        assert!(
            result.is_ok(),
            "op_difeq must return Ok(()) when !is_running (opens modal)"
        );
        assert!(
            state.modal_program.is_some(),
            "op_difeq must set modal_program when !is_running"
        );
        assert_eq!(
            state.modal_prompt,
            Some("FUNCTION NAME?".to_string()),
            "op_difeq must set modal_prompt to 'FUNCTION NAME?' when !is_running"
        );
    }

    // ── DifeqState struct tests ───────────────────────────────────────────────

    // Catches: DifeqState missing fields or wrong defaults
    #[test]
    fn difeq_state_default() {
        let s = DifeqState::default();
        assert_eq!(s.user_label, "");
        assert_eq!(s.order, 0);
        assert!(s.y_prime.is_none());
        assert_eq!(s.step_count, 0);
    }

    // Catches: DifeqState clone derive broken
    #[test]
    fn difeq_state_clone() {
        let s = DifeqState {
            user_label: "F".to_string(),
            order: 1,
            step_size: HpNum::from(1i32),
            x: HpNum::from(0i32),
            y: HpNum::from(1i32),
            y_prime: None,
            step_count: 5,
            max_steps: 100,
        };
        let s2 = s.clone();
        assert_eq!(s.user_label, s2.user_label);
        assert_eq!(s.order, s2.order);
        assert_eq!(s.step_count, s2.step_count);
    }

    // ── Modal flow tests (ORDER routing) ─────────────────────────────────────

    // Catches: ORDER=1 path accidentally visiting Y1PrimePrompt
    // Simulates the 5-prompt modal flow for ORDER=1
    #[test]
    fn modal_order1_flow() {
        use crate::ops::math1::modal::{DifeqInputStep, ModalProgram};
        // Verify the 5 prompts for ORDER=1 flow
        let steps = [
            DifeqInputStep::FunctionNamePrompt,
            DifeqInputStep::OrderPrompt,
            DifeqInputStep::StepSizePrompt,
            DifeqInputStep::X0Prompt,
            DifeqInputStep::Y0Prompt,
            DifeqInputStep::Ready, // Y1PrimePrompt is NOT visited for ORDER=1
        ];
        let prompts: Vec<Option<String>> = steps
            .iter()
            .map(|s| ModalProgram::Difeq(s.clone()).current_prompt())
            .collect();
        assert_eq!(prompts[0], Some("FUNCTION NAME?".to_string()));
        assert_eq!(prompts[1], Some("ORDER=?".to_string()));
        assert_eq!(prompts[2], Some("STEP SIZE=?".to_string()));
        assert_eq!(prompts[3], Some("X0=?".to_string()));
        assert_eq!(prompts[4], Some("Y0=?".to_string()));
        assert_eq!(prompts[5], None); // Ready — Y1PrimePrompt not in ORDER=1 flow
    }

    // Catches: ORDER=2 path not visiting Y1PrimePrompt
    // Simulates the 6-prompt modal flow for ORDER=2
    #[test]
    fn modal_order2_flow() {
        use crate::ops::math1::modal::{DifeqInputStep, ModalProgram};
        let steps = [
            DifeqInputStep::FunctionNamePrompt,
            DifeqInputStep::OrderPrompt,
            DifeqInputStep::StepSizePrompt,
            DifeqInputStep::X0Prompt,
            DifeqInputStep::Y0Prompt,
            DifeqInputStep::Y1PrimePrompt, // visited for ORDER=2
            DifeqInputStep::Ready,
        ];
        let prompts: Vec<Option<String>> = steps
            .iter()
            .map(|s| ModalProgram::Difeq(s.clone()).current_prompt())
            .collect();
        assert_eq!(prompts[4], Some("Y0=?".to_string()));
        assert_eq!(prompts[5], Some("Y'0=?".to_string())); // Y1PrimePrompt IS visited
        assert_eq!(prompts[6], None); // Ready
    }

    // ── ORDER validation test ─────────────────────────────────────────────────

    // Catches: ORDER=3 not triggering "ORDER MUST BE 1 OR 2" modal_prompt
    #[test]
    fn order_validation() {
        let (mut state, program) = make_exp_growth_state(1);
        // Set invalid order
        state.regs[0] = HpNum::from(3i32); // ORDER=3 (invalid)

        let result = op_difeq_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Ok(()),
            "Invalid order must return Ok(()) not HpError (per DIFEQ-01)"
        );
        assert_eq!(
            state.modal_prompt,
            Some("ORDER MUST BE 1 OR 2".to_string()),
            "Invalid order must set modal_prompt to 'ORDER MUST BE 1 OR 2'"
        );
        assert!(
            state.difeq_state.is_none(),
            "difeq_state must be None after invalid order (no mutation)"
        );
    }

    // ── RK4 correctness tests ─────────────────────────────────────────────────

    // Catches: ORDER=1 RK4 algorithm wrong (must converge to e^x for dy/dx=y)
    // Source: OM Chapter 7 worked example; Free42 v3.0.5 oracle: y(1.0) ≈ 2.71828
    // Pitfall-14: RK4 has O(h^4) global error; at h=0.1 over 10 steps, expect ~1e-5 accuracy
    // Use n_steps=12 so print_buffer[10] corresponds to x=1.0 (initial + 10 steps)
    #[test]
    fn rk4_order1_correctness() {
        let (mut state, program) = make_exp_growth_state(12); // R05=12: 12 steps then stop
        let result = op_difeq_run_loop(&mut state, &program);

        // Result should be Ok (no error)
        assert!(result.is_ok(), "ORDER=1 RK4 on dy/dx=y failed: {result:?}");

        // After 10 steps (x=0..1 with h=0.1), y ≈ e ≈ 2.71828
        // print_buffer[0] = initial (x=0, y=1), print_buffer[10] = step 10 (x=1.0)
        assert!(
            state.print_buffer.len() > 10,
            "Expected at least 11 entries in print_buffer, got {}",
            state.print_buffer.len()
        );
        // Parse y from step 10 (index 10)
        let step10 = &state.print_buffer[10];
        assert!(
            step10.starts_with("X="),
            "Step 10 must start with X=, got: {step10}"
        );
        // Extract Y value from "X=... Y=..."
        if let Some(y_part) = step10.split(" Y=").nth(1) {
            let y_str = y_part.split_whitespace().next().unwrap_or("0");
            let y_val: f64 = y_str.parse().unwrap_or(0.0);
            let e = std::f64::consts::E;
            assert!(
                (y_val - e).abs() < 1e-4,
                "ORDER=1 RK4 dy/dx=y at x=1: expected y≈{e:.5}, got {y_val:.5}"
            );
        }
    }

    // Catches: ORDER=2 RK4 algorithm wrong (must converge to cos(x) for y''=-y)
    // Source: OM Chapter 7 worked example; Free42 v3.0.5 oracle: y(1.0) ≈ cos(1) ≈ 0.5403
    #[test]
    fn rk4_order2_correctness() {
        let (mut state, program) = make_harmonic_state();
        let result = op_difeq_run_loop(&mut state, &program);

        assert!(result.is_ok(), "ORDER=2 RK4 on y''=-y failed: {result:?}");

        // After 10 steps (h=0.1), y ≈ cos(1) ≈ 0.5403
        assert!(
            state.print_buffer.len() > 10,
            "Expected at least 11 print_buffer entries, got {}",
            state.print_buffer.len()
        );
        let step10 = &state.print_buffer[10];
        if let Some(y_part) = step10.split(" Y=").nth(1) {
            let y_str = y_part.split(" Y'=").next().unwrap_or("0").trim();
            let y_val: f64 = y_str.parse().unwrap_or(0.0);
            let cos1 = 1.0_f64.cos();
            assert!(
                (y_val - cos1).abs() < 1e-3,
                "ORDER=2 RK4 y''=-y at x=1: expected y≈{cos1:.5}, got {y_val:.5}"
            );
        }
    }

    // ── Print buffer output test (DIFEQ-05) ───────────────────────────────────

    // Catches: step-by-step print_buffer output missing or wrong format
    #[test]
    fn step_by_step_print_buffer() {
        let (mut state, program) = make_exp_decay_state();
        let result = op_difeq_run_loop(&mut state, &program);

        assert!(result.is_ok(), "op_difeq_run_loop failed: {result:?}");

        // Should have initial + at least 5 step lines
        assert!(
            state.print_buffer.len() >= 6,
            "Expected at least 6 print_buffer entries (initial + 5 steps), got {}",
            state.print_buffer.len()
        );
        // Every line must contain "X=" and "Y="
        for (i, line) in state.print_buffer.iter().enumerate() {
            assert!(
                line.contains("X=") && line.contains("Y="),
                "print_buffer[{i}] must contain X= and Y=, got: {line}"
            );
        }
        // ORDER=1 lines must NOT contain "Y'=" (that's for ORDER=2)
        for (i, line) in state.print_buffer.iter().enumerate() {
            assert!(
                !line.contains("Y'="),
                "ORDER=1 print_buffer[{i}] must not contain Y'=, got: {line}"
            );
        }
    }

    // Catches: ORDER=2 print_buffer not including Y'= component
    #[test]
    fn step_by_step_print_buffer_order2() {
        let (mut state, program) = make_harmonic_state();
        let result = op_difeq_run_loop(&mut state, &program);

        assert!(result.is_ok(), "ORDER=2 DIFEQ failed: {result:?}");
        assert!(
            state.print_buffer.len() >= 6,
            "Expected at least 6 print_buffer entries, got {}",
            state.print_buffer.len()
        );
        // ORDER=2 lines must contain "Y'=" (from step 1 onward)
        for (i, line) in state.print_buffer.iter().skip(1).enumerate() {
            assert!(
                line.contains("Y'="),
                "ORDER=2 print_buffer[{}] must contain Y'=, got: {line}",
                i + 1
            );
        }
    }

    // ── Scratch registers test (DIFEQ-03) ─────────────────────────────────────

    // Catches: wrong understanding of scratch register convention
    // The HP-41 OM Chapter 7 assigns R00-R07 as scratch during DIFEQ.
    // Our implementation uses local Rust variables for RK4 intermediates;
    // the register convention is documented but not enforced by the emulator.
    // This test documents the user-responsibility: a function that STO's R01
    // (which holds step_size in our convention) will corrupt the integration.
    //
    // Test: function stores a value into R01. DIFEQ must still complete (no error),
    // but the step_size read for subsequent steps may differ.
    #[test]
    fn scratch_registers() {
        // Scratch register layout (DIFEQ-03 per OM convention):
        // R00 = order, R01 = step size h, R02 = x0, R03 = y0, R04 = y'0
        // This test verifies that a user function that STO's into R04
        // (y'0 slot) does NOT raise an error (hardware-faithful: user-responsibility).
        //
        // Program: LBL "SR" / (return y already in Y) swap / (clobber R04) STO 04 / RTN
        let program = vec![
            Op::Lbl("SR".to_string()),
            Op::XySwap,    // y → X (f(x,y) = y for dy/dx = y)
            Op::StoReg(4), // clobber R04 — scratch register (user-responsibility divergence)
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "SR".to_string();
        state.regs[0] = HpNum::from(1i32);
        state.regs[1] = HpNum::from(
            rust_decimal::Decimal::from_f64(0.1).unwrap_or(rust_decimal::Decimal::ZERO),
        );
        state.regs[2] = HpNum::from(0i32);
        state.regs[3] = HpNum::from(1i32);
        state.regs[5] = HpNum::from(5i32); // max_steps = 5 (enough to see R04 clobber)

        // Must complete without error (user-responsibility, hardware-faithful)
        let result = op_difeq_run_loop(&mut state, &program);
        assert!(
            result.is_ok(),
            "Scratch register clobber in user fn must not raise error (user-responsibility), got: {result:?}"
        );
        // R04 has been clobbered — non-zero from function writes
        let r04_val = state.regs[4].inner().to_f64().unwrap();
        assert!(
            r04_val != 0.0,
            "R04 must have been clobbered by user function STO 04, got: {r04_val}"
        );
    }

    // ── Nested rejection test (XROM-08 FINAL 3-state guard) ──────────────────

    // Catches: XROM-08 guard missing for one of the 3 states (integ/solve/difeq)
    // Tests all 3 pre-set states → each must return InvalidOp (XROM-08 / DIFEQ-01)
    #[test]
    fn nested_rejection() {
        use crate::ops::math1::integ::IntegState;
        use crate::ops::math1::solve::SolveState;

        let (mut state, program) = make_exp_growth_state(1);

        // Test 1: integ_state pre-set → DIFEQ must reject
        state.integ_state = Some(IntegState::default());
        let result = op_difeq_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "DIFEQ with integ_state set must return InvalidOp (XROM-08 FINAL 3-state guard)"
        );
        assert!(
            state.difeq_state.is_none(),
            "difeq_state must remain None after rejection"
        );
        assert!(
            state.integ_state.is_some(),
            "integ_state must be unchanged after pre-mutation rejection"
        );
        state.integ_state = None; // reset

        // Test 2: solve_state pre-set → DIFEQ must reject
        state.solve_state = Some(SolveState::default());
        let result = op_difeq_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "DIFEQ with solve_state set must return InvalidOp (XROM-08 FINAL 3-state guard)"
        );
        assert!(
            state.difeq_state.is_none(),
            "difeq_state must remain None after rejection"
        );
        state.solve_state = None; // reset

        // Test 3: difeq_state pre-set → DIFEQ must reject itself
        state.difeq_state = Some(DifeqState::default());
        let result = op_difeq_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "DIFEQ with difeq_state set must return InvalidOp (XROM-08 FINAL 3-state guard)"
        );
        // difeq_state must be unchanged (pre-mutation rejection)
        assert!(
            state.difeq_state.is_some(),
            "Pre-set difeq_state must be unchanged after pre-mutation rejection"
        );
    }

    // ── Call stack full test (Pitfall 4) ──────────────────────────────────────

    // Catches: call_stack cap not checked before mutation (state leak on CallDepth)
    #[test]
    fn call_stack_full() {
        let (mut state, program) = make_exp_growth_state(1);

        // Pre-fill call_stack with 4 entries
        state.call_stack.push(0);
        state.call_stack.push(1);
        state.call_stack.push(2);
        state.call_stack.push(3);
        assert_eq!(state.call_stack.len(), 4);

        let result = op_difeq_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::CallDepth),
            "op_difeq_run_loop with full call_stack must return CallDepth"
        );
        // call_stack must still have 4 entries (no leak — pre-mutation check)
        assert_eq!(
            state.call_stack.len(),
            4,
            "call_stack must not be modified after CallDepth rejection"
        );
        // difeq_state must remain None (pre-mutation check)
        assert!(
            state.difeq_state.is_none(),
            "difeq_state must be None after CallDepth rejection"
        );
    }

    // ── Cancellation test (D-28.8 per-64-steps check) ────────────────────────

    // Catches: per-64-steps cancellation check missing or checking wrong flag
    #[test]
    fn cancel_per_64_steps() {
        let (mut state, program) = make_exp_growth_state(100);
        // Pre-set cancel_requested = true (would normally come from GUI via Phase 31)
        state
            .cancel_requested
            .store(true, std::sync::atomic::Ordering::Relaxed);

        let result = op_difeq_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::Canceled),
            "DIFEQ with cancel_requested=true must return Canceled on first per-64 check"
        );
        // difeq_state must be cleared after cancellation
        assert!(
            state.difeq_state.is_none(),
            "difeq_state must be None after cancellation (no state leak)"
        );
    }
}
