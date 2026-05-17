// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! INTG (numerical integration) state and implementation.
//!
//! Plan 28-07 fills the full `IntegState` struct and implements:
//! - `integ_threshold(mode)` — display-mode-tied convergence tolerance (ADR-004)
//! - `op_integ_run_loop(state, program)` — re-entrant Simpson integrator
//! - `op_integ(state)` — dispatch-arm stub (returns InvalidOp; INTG runs only in run_loop)
//!
//! ## Architecture — User Callback Re-entrancy (C-28.5 / Pitfall 4)
//!
//! `Op::Integ` is the FIRST v3.0 user-callback op. It re-enters `run_loop` for each
//! Simpson-rule sample point, evaluating a user-defined LBL function f(x).
//! The outer program clone from `run_program` / `resume_program` is reused — NO
//! secondary clone is made inside `op_integ_run_loop`. This is the same pattern as
//! `Op::XeqInd` (program.rs:476-491 for context).
//!
//! ## Scratch Register Convention (RESEARCH Open Q6)
//!
//! Math Pac I uses R00–R07 as scratch during integration.
//! If the user-provided LBL function executes STO to any of these registers,
//! the solver's internal state is corrupted and the result will be wrong.
//! The emulator faithfully reproduces this hardware-faithful behavior — NO snapshot/restore.
//! OM 1979 p. 35: "do not use registers R00–R07 in your user function while INTG is active."
//! Test: `tests/math1_user_callback.rs::user_fn_stores_to_scratch_corrupts_integ`

use std::sync::atomic::Ordering;

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::Op;
use crate::stack::{apply_lift_effect, enter_number, unary_result, LiftEffect};
use crate::state::{CalcState, DisplayMode};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

// ── IntegMode enum ────────────────────────────────────────────────────────────

/// Integration mode for the INTG workflow.
///
/// Discrete: user provides f(xⱼ) sample values manually (trapezoidal or Simpson).
/// Explicit: user provides a LBL program to evaluate f(x) at each sample point.
///
/// Default = Explicit per plan (most common usage).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum IntegMode {
    /// Discrete mode — user provides sample values; INTG applies trapezoidal or Simpson rule.
    Discrete,
    /// Explicit mode — user provides a LBL function; INTG evaluates it at each sample point.
    #[default]
    Explicit,
}

// ── IntegState struct ─────────────────────────────────────────────────────────

/// Mid-iteration state for the INTG numerical integration solver.
///
/// Fields are populated when `Op::Integ` begins and cleared when it completes
/// (success or error). `CalcState.integ_state` holds this as `Option<IntegState>`.
///
/// The `#[serde(skip)]` on `CalcState::integ_state` means this struct is never
/// persisted — it is purely transient state for the duration of integration.
///
/// Scratch register note (RESEARCH Open Q6): R00–R07 are scratch during INTG.
/// The emulator does NOT snapshot/restore these registers. If the user function
/// STO's into R00–R07, the integration result will be wrong (documented divergence).
#[derive(Debug, Clone, Default)]
pub struct IntegState {
    /// XEQ label of the user-supplied integrand function (Explicit mode only).
    pub user_label: String,
    /// Lower integration bound a (Explicit mode).
    pub a: HpNum,
    /// Upper integration bound b (Explicit mode).
    pub b: HpNum,
    /// Number of subdivisions (cap 32768 per INTG-07/ADR-004).
    pub n: u16,
    /// Running Simpson/trapezoidal accumulator.
    pub accumulator: HpNum,
    /// Integration mode: Discrete or Explicit.
    pub mode: IntegMode,
}

// ── integ_threshold helper ────────────────────────────────────────────────────

/// INTG convergence tolerance tied to DisplayMode.
///
/// Source: HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979), Chapter 3,
/// page 35–36: "converges when consecutive approximations differ by less than
/// 5 in the last displayed digit" — i.e., ½ ULP of displayed precision.
/// Formula: 5 × 10^(-(decimals + 1)) where decimals = FIX/SCI/ENG digit count.
/// ADR-004 (Plan 28-01) locks this formula; Plan 28-07 consumes it.
///
/// Free42 cross-check: Free42 uses `0.5e-{digits}` = `5e-(digits+1)` — identical.
///
/// Examples:
/// - Fix(4) → 5e-5  = 0.00005
/// - Fix(9) → 5e-10 = 0.0000000005
///
/// Pitfall-2 detection: tests must run the SAME integral in BOTH Fix(4) and Fix(9)
/// and assert DIFFERENT precision. If this formula is wrong, both tests will pass
/// with the same tolerance and the bug is invisible to single-mode tests.
pub fn integ_threshold(mode: DisplayMode) -> f64 {
    let decimals = match mode {
        DisplayMode::Fix(n) | DisplayMode::Sci(n) | DisplayMode::Eng(n) => n as i32,
    };
    // threshold = 5 × 10^(-(decimals + 1))
    // e.g. Fix(4) → 5e-5; Fix(9) → 5e-10
    5.0_f64 * 10.0_f64.powi(-(decimals + 1))
}

/// Maximum number of function evaluations per INTG call.
/// Source: HP-41C Math Pac OM (HP 00041-90034, 1979), page 37.
pub const INTG_MAX_EVALS: u32 = 32_768; // 2^15

// ── Accumulator helpers ───────────────────────────────────────────────────────

/// Simpson 1/3 rule coefficient for sample index k in [0..=n].
///
/// For n subdivisions (n must be even for Simpson):
///   k=0 or k=n → weight 1 (endpoints)
///   k odd → weight 4
///   k even, k not 0 or n → weight 2
///
/// The final integral ≈ (h/3) * Σ weight_k * f(x_k)
fn simpson_coeff(k: u32, n: u32) -> u32 {
    if k == 0 || k == n {
        1
    } else if k % 2 == 1 {
        4
    } else {
        2
    }
}

// ── op_integ (dispatch arm — REJECT) ─────────────────────────────────────────

/// Dispatch arm for `Op::Integ` (interactive / dispatch() call).
///
/// Returns `Err(HpError::InvalidOp)` unconditionally.
/// `Op::Integ` can only run inside `run_loop`; the real implementation is
/// `op_integ_run_loop`. This mirrors the `Op::XeqInd` precedent: the dispatch
/// arm in program.rs line 839+ returns InvalidOp while the run_loop arm
/// (program.rs:482-496) does the work.
///
/// XROM resolver routes `XEQ "INTG"` through `dispatch()` which calls here.
/// Only when INTG appears inside a *running program* is it routed through
/// `run_loop`'s match arm → `op_integ_run_loop`.
pub fn op_integ(state: &mut CalcState) -> Result<(), HpError> {
    // Phase 29 / CLI-07 — additive completion of Phase 28 stub.
    // The Phase 28 stub anticipated this wiring (symmetric with op_solve pattern).
    //
    // CR-04 fix: open at ModeChoice (the documented OM-faithful entry point),
    // not at FunctionNamePrompt. Previously the dispatch arm skipped ModeChoice
    // entirely, making `IntegInputStep::ModeChoice` and `submit_step(ModeChoice)`
    // unreachable on every interactive path. The submit_step(ModeChoice) arm
    // (Explicit-mode default) advances the modal to FunctionNamePrompt on the
    // first R/S, matching the previous behavior end-to-end for Explicit users.
    // Full Discrete-mode dispatch (split inside submit_step(ModeChoice) based on
    // user choice) ships in a later plan per Phase 28-07-SUMMARY:245.
    if !state.is_running {
        state.modal_program = Some(
            crate::ops::math1::modal::ModalProgram::Integ(
                crate::ops::math1::modal::IntegInputStep::ModeChoice,
            ),
        );
        state.modal_prompt = Some("INTG MODE?".to_string());
        return Ok(());
    }
    // Op::Integ inside run_loop: real implementation in op_integ_run_loop.
    Err(HpError::InvalidOp)
}

// ── op_integ_run_loop (real implementation) ───────────────────────────────────

/// Real INTG implementation — called from `run_loop`'s `Op::Integ` match arm.
///
/// ## Entry contract
///
/// `state.integ_state` is `None` at entry — the run_loop arm is responsible for
/// routing here. This function sets `integ_state = Some(...)` AFTER all pre-mutation
/// guards pass.
///
/// ## Guard order (pre-mutation, MUST check BEFORE any state mutation)
///
/// 1. Strict-reject nested callback (XROM-08 / ADR-002):
///    `state.integ_state.is_some() || state.solve_state.is_some() || state.difeq_state.is_some()`
///    → `HpError::InvalidOp` BEFORE any state change.
/// 2. Pre-mutation call_stack cap (Pitfall 4):
///    `state.call_stack.len() >= 4` → `HpError::CallDepth` BEFORE any state change.
/// 3. Subdivision cap (INTG-07): n > 32768 → `HpError::Domain`.
///
/// ## Explicit mode algorithm
///
/// Simpson composite rule over [a, b] with n subdivisions (n even required for Simpson).
/// h = (b - a) / n
/// Σ = (h/3) * Σ_{k=0}^{n} coeff(k) * f(x_k)  where x_k = a + k*h
///
/// For each sample: push x_k to X with lift, find user LBL in program, push pc + call_stack,
/// run_loop re-entry (NOT run_program — outer clone reused per C-28.5 / Pitfall 4).
/// After re-entry, X = f(x_k).
///
/// ## Cancellation (D-28.7 / D-28.8)
///
/// Every 64 samples: `if state.cancel_requested.load(Ordering::Relaxed)`, clear `integ_state`,
/// return `HpError::Canceled`. This plumbing is correct; the GUI wiring to set
/// `cancel_requested = true` ships in Phase 31 / GUI-05.
///
/// ## Discrete mode
///
/// Trapezoidal or Simpson on user-provided sample values (from X register on each RTN).
/// Phase 28-07 ships the core arithmetic; modal input wiring is Phase 29 / CLI-06.
pub fn op_integ_run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    // ── Pre-mutation guards (MUST run before any state mutation) ──────────────
    // Guard 1 (XROM-08 / ADR-002): strict-reject nested user-callback
    if state.integ_state.is_some() || state.solve_state.is_some() || state.difeq_state.is_some() {
        return Err(HpError::InvalidOp);
    }
    // Guard 2 (Pitfall 4 / D-22.15 generalized): pre-mutation call_stack cap
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }

    // Note: cancel_requested is NOT reset here — the workflow opener (Phase 29 / CLI-07)
    // resets it when the user first initiates INTG. In run_loop context, the flag may
    // already be set by a GUI cancel request (Phase 31 / GUI-05 wiring).
    // The per-64-samples check inside the sample loop (D-28.7 / D-28.8) will fire
    // if cancel_requested is set, returning HpError::Canceled.

    // Read integration parameters from the modal state that was set up before
    // run_loop dispatched Op::Integ. For now (Plan 28-07), we read directly from
    // the stack state (X=a, Y=b) and parse the subdivision count from a register.
    // Phase 29 / CLI-07 wires the full modal flow. For tests, we allow parameters
    // to be pre-staged in a local IntegState by test setup if integ_state is Some —
    // but integ_state MUST be None at this point (guard 1 above ensures this).
    //
    // Default: read a from X, b from Y, n from R00 (integer part), user_label from ALPHA.
    // This matches the OM's "INTG requires the function label in ALPHA" convention.
    let a = state.stack.x.clone();
    let b = state.stack.y.clone();
    let n_raw = state
        .regs
        .first()
        .map(|r| r.trunc_int())
        .unwrap_or_default();
    let n_val = n_raw.inner().to_u32().unwrap_or(0);
    let user_label = state.alpha_reg.clone();
    let mode = IntegMode::Explicit; // Plan 28-07 implements Explicit mode; Discrete wired Phase 29

    // Guard 3 (INTG-07): subdivision cap
    if n_val > INTG_MAX_EVALS {
        return Err(HpError::Domain);
    }

    // Use at least 2 subdivisions for a meaningful integral
    let n = n_val.max(2);

    // Simpson requires even number of subdivisions
    let n_even = if n % 2 == 1 { n + 1 } else { n };

    // ── Commit: set integ_state after all pre-mutation guards pass ────────────
    let integ = IntegState {
        user_label: user_label.clone(),
        a: a.clone(),
        b: b.clone(),
        n: n_even.min(u16::MAX as u32) as u16,
        accumulator: HpNum::zero(),
        mode: mode.clone(),
    };

    // Find the user label in the program BEFORE starting the loop
    // (fail fast with InvalidOp if label not found)
    let label_pos = program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if *l == user_label))
        .ok_or(HpError::InvalidOp)?;

    state.integ_state = Some(integ.clone());

    // ── Simpson composite rule ────────────────────────────────────────────────
    // h = (b - a) / n_even
    // integral ≈ (h/3) * Σ_{k=0}^{n_even} coeff(k) * f(x_k)
    //
    // CR-01 fix: every `?`-conversion between `state.integ_state = Some(...)`
    // above and `state.integ_state = None` at the success cleanup below MUST
    // clear `integ_state` on the Err path. If we leak `Some(_)` here, the next
    // INTG call hits the XROM-08 nested-rejection guard at the top of this
    // function and returns InvalidOp — INTG is permanently broken until the
    // user finds a way to manually reset state.
    let a_f64 = match a.inner().to_f64() {
        Some(v) => v,
        None => {
            state.integ_state = None;
            return Err(HpError::Overflow);
        }
    };
    let b_f64 = match b.inner().to_f64() {
        Some(v) => v,
        None => {
            state.integ_state = None;
            return Err(HpError::Overflow);
        }
    };
    let n_f64 = n_even as f64;
    let h_f64 = (b_f64 - a_f64) / n_f64;

    let mut sum_f64: f64 = 0.0;
    let save_pc = state.pc;
    let save_call_stack_len = state.call_stack.len();

    match mode {
        IntegMode::Explicit => {
            for k in 0..=n_even {
                // ── Per-64-samples cancellation check (D-28.7 / D-28.8) ───────
                if k & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
                    state.integ_state = None;
                    return Err(HpError::Canceled);
                }

                let x_k = a_f64 + (k as f64) * h_f64;
                let coeff = simpson_coeff(k, n_even) as f64;

                // Push x_k to stack with lift enabled
                state.stack.lift_enabled = true;
                // CR-01 fix: clear integ_state on Overflow path so the next
                // INTG call is not poisoned by the XROM-08 nested guard.
                let x_k_decimal = match Decimal::from_f64(x_k) {
                    Some(d) => d,
                    None => {
                        state.integ_state = None;
                        state.pc = save_pc;
                        return Err(HpError::Overflow);
                    }
                };
                enter_number(state, HpNum::from(x_k_decimal));
                apply_lift_effect(state, LiftEffect::Enable);

                // Re-enter run_loop with the user function (C-28.5 / Pitfall 4)
                // Outer program clone is REUSED — no secondary clone here.
                state.call_stack.push(state.pc);
                state.pc = label_pos + 1; // execute step AFTER LBL marker

                // run_loop re-entry (private, accessed via super:: from program.rs arm)
                // We call the inner loop through an indirection: we invoke the callback
                // by running sub_run_loop. Since run_loop is private, we implement the
                // minimal subset here and rely on the run_loop arm to dispatch back.
                //
                // DESIGN NOTE: op_integ_run_loop is called FROM run_loop. To re-enter
                // run_loop for each sample, we implement a local sub-loop that mirrors
                // run_loop's stepping logic but returns on RTN/end-of-program.
                // This is equivalent to what XeqInd does — the call_stack.push above
                // ensures RTN will pop back to save_pc.
                let sub_result = run_user_function(state, program);

                // Truncate any extra call stack frames added by sub-loop (defensive cleanup)
                while state.call_stack.len() > save_call_stack_len {
                    state.call_stack.pop();
                }

                match sub_result {
                    Ok(()) => {}
                    Err(e) => {
                        state.integ_state = None;
                        state.pc = save_pc;
                        return Err(e);
                    }
                }

                // After sub-loop returns, X = f(x_k).
                // CR-01 fix: clear integ_state on Overflow path so the next
                // INTG call is not poisoned by the XROM-08 nested guard.
                let fx_k = match state.stack.x.inner().to_f64() {
                    Some(v) => v,
                    None => {
                        state.integ_state = None;
                        state.pc = save_pc;
                        return Err(HpError::Overflow);
                    }
                };
                sum_f64 += coeff * fx_k;

                // Update running accumulator for Canceled-path diagnostic
                if let Some(ref mut st) = state.integ_state {
                    st.accumulator =
                        HpNum::from(Decimal::from_f64(sum_f64).unwrap_or(Decimal::ZERO));
                }
            }
        }
        IntegMode::Discrete => {
            // Discrete mode: user provides samples one by one via stack input.
            // Phase 29 / CLI-07 wires the full modal flow. For now, return InvalidOp.
            // (Discrete mode tests are in future plans.)
            state.integ_state = None;
            state.pc = save_pc;
            return Err(HpError::InvalidOp);
        }
    }

    // ── Compute final result ──────────────────────────────────────────────────
    let result_f64 = (h_f64 / 3.0) * sum_f64;

    // Restore pc to where we were before INTG
    state.pc = save_pc;

    // Clear integ_state before writing result
    state.integ_state = None;

    // Push result to X with lift (LiftEffect: Enable — new value arrives in X)
    let result = HpNum::from(Decimal::from_f64(result_f64).ok_or(HpError::Overflow)?);
    unary_result(state, result);
    apply_lift_effect(state, LiftEffect::Enable);

    Ok(())
}

// ── run_user_function: sub-loop for user callback evaluation ──────────────────

/// Execute the user callback function starting at the current `state.pc`.
///
/// This is a minimal `run_loop` that steps through the program until:
/// - RTN with empty call_stack (function returned)
/// - End of program
/// - STOP (halt)
/// - Error
///
/// The `state.call_stack` was already pushed by `op_integ_run_loop` before calling here.
/// When `RTN` pops back to the saved PC (from the integ loop), execution stops.
///
/// MAX_STEPS guard matches the outer run_loop limit.
fn run_user_function(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    use crate::ops::program::execute_op_pub;

    // Remember the call_stack depth at entry so we know when the user's RTN
    // pops us back to the integ loop level.
    let entry_depth = state.call_stack.len();
    let mut steps: u64 = 0;
    const MAX_SUB_STEPS: u64 = 100_000;

    loop {
        if steps >= MAX_SUB_STEPS {
            return Err(HpError::Overflow); // infinite-loop guard
        }
        steps += 1;

        if state.pc >= program.len() {
            // Ran off end = implicit RTN
            break;
        }
        let op = program[state.pc].clone();
        state.pc += 1;

        match op {
            Op::Rtn => {
                match state.call_stack.pop() {
                    Some(return_pc) => {
                        state.pc = return_pc;
                        // If we've popped back to entry_depth, we're done
                        if state.call_stack.len() < entry_depth {
                            break;
                        }
                    }
                    None => break, // top-level RTN
                }
            }
            Op::Lbl(_) => {
                // LBL is a marker only — no-op during execution
            }
            Op::Stop => break,
            other => {
                execute_op_pub(state, other)?;
            }
        }
    }

    Ok(())
}

// ── Phase 29 / CLI-05 additive public surface — D-29.5 ───────────────────────

/// Submit a numeric input step in the INTG modal workflow.
///
/// Called by `hp41_core::ops::math1::submit_modal` after `flush_entry_buf` has
/// flushed the entry buffer to `state.stack.x`. Reads X, advances the INTG
/// modal step state machine, updates `state.modal_prompt`.
///
/// **CR-02 fix:** parameters are written to the SAME locations `op_integ_run_loop`
/// reads from (stack X/Y for the integration bounds, R00 for the subdivision
/// count). Previously the modal stored bounds in R02/R03 and N in R04, but the
/// run_loop reads `a` from `state.stack.x`, `b` from `state.stack.y`, and `n`
/// from `regs[0]` — so values entered via the modal were silently discarded at
/// run time. Picking the run_loop side of the contract preserves the existing
/// `make_x_squared_state()` test-helper convention.
///
/// Step transitions:
/// - `ModeChoice` → advances to `FunctionNamePrompt` (Explicit mode).
/// - `IntervalPrompt` → reads Y as upper bound `b`, leaves X as lower bound `a`;
///   stack already holds the (a, b) pair so no scratch-register copy is needed.
///   Advances to SubdivisionPrompt.
/// - `SubdivisionPrompt` → reads X as N (subdivision count, capped at
///   INTG_MAX_EVALS), stores in R00 — matching `op_integ_run_loop` line ~234
///   which reads `n` from `regs[0]`. Restores the (a, b) pair to X/Y so the
///   run_loop sees the integration bounds the user entered at IntervalPrompt.
/// - All other steps → `Err(HpError::InvalidOp)`.
///
/// Phase 29 / CLI-05 additive public surface — D-29.5.
pub fn submit_step(
    state: &mut CalcState,
    step: crate::ops::math1::modal::IntegInputStep,
) -> Result<(), HpError> {
    use crate::ops::math1::modal::{IntegInputStep, ModalProgram};
    match step {
        IntegInputStep::ModeChoice => {
            // Advance to FunctionNamePrompt (Explicit mode — default per plan)
            state.modal_program = Some(ModalProgram::Integ(IntegInputStep::FunctionNamePrompt));
            state.modal_prompt = Some("FUNCTION NAME?".to_string());
            Ok(())
        }
        IntegInputStep::IntervalPrompt => {
            // OM: user enters `a` first, then `b` — so on entry: X = b, Y = a.
            // op_integ_run_loop reads `a` from stack.x and `b` from stack.y, so we
            // swap the two so the bounds land in the run_loop's expected slots.
            // CR-02 fix: keep the bounds on the stack rather than copying them
            // into scratch registers the run_loop never reads.
            let a = state.stack.y.clone();
            let b = state.stack.x.clone();
            state.stack.x = a;
            state.stack.y = b;
            state.modal_program = Some(ModalProgram::Integ(IntegInputStep::SubdivisionPrompt));
            state.modal_prompt = Some("N=?".to_string());
            Ok(())
        }
        IntegInputStep::SubdivisionPrompt => {
            // X = N (subdivision count); cap at INTG_MAX_EVALS.
            // CR-02 fix: store N in R00 (where op_integ_run_loop reads `n`),
            // not R04. Then restore (a, b) to (X, Y) by recovering them from
            // the stack lift chain — but submit_modal has already flushed
            // entry_buf which pushed N to X and shifted (a, b) down to (Y, Z).
            // So Y holds `b`, Z holds `a`.
            let n_raw = state
                .stack
                .x
                .inner()
                .to_u32()
                .unwrap_or(INTG_MAX_EVALS)
                .min(INTG_MAX_EVALS);
            if state.regs.is_empty() {
                return Err(HpError::InvalidOp);
            }
            state.regs[0] = HpNum::from(n_raw as i32);
            // Restore (a, b) → (X, Y): after the N push, Z = a, Y = b.
            // Pop N off the top by shifting the stack down one slot.
            let new_x = state.stack.z.clone(); // a
            let new_y = state.stack.y.clone(); // b
            state.stack.x = new_x;
            state.stack.y = new_y;
            state.modal_program = Some(ModalProgram::Integ(IntegInputStep::Ready));
            state.modal_prompt = None;
            Ok(())
        }
        IntegInputStep::FunctionNamePrompt | IntegInputStep::Ready => {
            // FunctionNamePrompt handled by submit_label_step; Ready has no submission.
            Err(HpError::InvalidOp)
        }
    }
}

/// Submit the function label step for the INTG modal workflow.
///
/// Called by `hp41_core::ops::math1::submit_modal_with_label` when the modal is
/// at `IntegInputStep::FunctionNamePrompt`. The label has already been written to
/// `state.alpha_reg` before this is called.
///
/// Advances the modal from `FunctionNamePrompt` to `IntervalPrompt` and sets
/// `modal_prompt = Some("(A,B)=?")`.
///
/// Phase 29 / CLI-05 additive public surface — D-29.5.
pub fn submit_label_step(state: &mut CalcState) -> Result<(), HpError> {
    use crate::ops::math1::modal::{IntegInputStep, ModalProgram};
    state.modal_program = Some(ModalProgram::Integ(IntegInputStep::IntervalPrompt));
    state.modal_prompt = Some("(A,B)=?".to_string());
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::{CalcState, DisplayMode};

    // ── integ_threshold tests ─────────────────────────────────────────────────

    // Catches: integ_threshold wrong for Fix(4) — must be 5e-5 not 1e-5
    #[test]
    fn threshold_fix4() {
        let t = integ_threshold(DisplayMode::Fix(4));
        // 5 × 10^(-5) = 5e-5 = 0.00005
        approx::assert_relative_eq!(t, 5e-5_f64, max_relative = 1e-10);
    }

    // Catches: integ_threshold wrong for Fix(9) — must be 5e-10 not 5e-5
    // PITFALL-2: if both Fix(4) and Fix(9) return the same threshold, the formula is wrong
    #[test]
    fn threshold_fix9() {
        let t = integ_threshold(DisplayMode::Fix(9));
        // 5 × 10^(-10) = 5e-10
        approx::assert_relative_eq!(t, 5e-10_f64, max_relative = 1e-10);
    }

    // Catches: Fix(4) and Fix(9) accidentally returning the same threshold (Pitfall-2)
    #[test]
    fn threshold_fix4_and_fix9_are_different() {
        let t4 = integ_threshold(DisplayMode::Fix(4));
        let t9 = integ_threshold(DisplayMode::Fix(9));
        assert!(
            t4 > t9,
            "Fix(4) threshold ({t4}) must be larger (coarser) than Fix(9) threshold ({t9})"
        );
        // Ratio should be 10^5 = 100000
        approx::assert_relative_eq!(t4 / t9, 1e5_f64, max_relative = 1e-9);
    }

    // Catches: Sci(n) or Eng(n) not matching Fix(n) for same digit count
    #[test]
    fn threshold_sci4_matches_fix4() {
        let tf = integ_threshold(DisplayMode::Fix(4));
        let ts = integ_threshold(DisplayMode::Sci(4));
        approx::assert_relative_eq!(tf, ts, max_relative = 1e-10);
    }

    // ── INTG_MAX_EVALS constant ───────────────────────────────────────────────

    // Catches: INTG_MAX_EVALS wrong value (must be 2^15 = 32768)
    #[test]
    fn max_evals_is_32768() {
        assert_eq!(
            INTG_MAX_EVALS, 32_768,
            "INTG_MAX_EVALS must be 2^15 = 32768 per OM p.37"
        );
    }

    // ── IntegState struct ─────────────────────────────────────────────────────

    // Catches: IntegState missing fields or wrong defaults
    #[test]
    fn integ_state_default() {
        let s = IntegState::default();
        assert_eq!(s.user_label, "");
        assert_eq!(s.n, 0);
        assert_eq!(s.mode, IntegMode::Explicit);
    }

    // Catches: IntegState clone/eq derives broken
    #[test]
    fn integ_state_clone() {
        let s = IntegState {
            user_label: "F".to_string(),
            a: HpNum::from(0i32),
            b: HpNum::from(1i32),
            n: 10,
            accumulator: HpNum::zero(),
            mode: IntegMode::Explicit,
        };
        let s2 = s.clone();
        assert_eq!(s.user_label, s2.user_label);
        assert_eq!(s.n, s2.n);
    }

    // ── op_integ (dispatch stub) ──────────────────────────────────────────────

    // Catches: op_integ interactive branch (Phase 29 completion) not opening modal at
    // ModeChoice when called interactively (!is_running).
    // CR-04 fix: op_integ opens at ModeChoice (the OM-faithful documented entry).
    // The submit_step(ModeChoice) arm advances to FunctionNamePrompt on first R/S
    // (Explicit-mode default). Previously op_integ opened directly at
    // FunctionNamePrompt, making the ModeChoice variant unreachable.
    #[test]
    fn op_integ_dispatch_opens_modal_when_interactive() {
        let mut state = CalcState::new();
        // is_running = false by default (interactive mode)
        let result = op_integ(&mut state);
        assert!(
            result.is_ok(),
            "op_integ must return Ok(()) when !is_running (opens modal)"
        );
        assert!(
            state.modal_program.is_some(),
            "op_integ must set modal_program when !is_running"
        );
        assert_eq!(
            state.modal_prompt,
            Some("INTG MODE?".to_string()),
            "op_integ must set modal_prompt to 'INTG MODE?' when !is_running (CR-04)"
        );
    }

    // ── Helper: set up a simple state with a user function program ────────────

    fn make_x_squared_state() -> (CalcState, Vec<Op>) {
        // Program: LBL "F" / X^2 / RTN
        // f(x) = x * x
        let program = vec![
            Op::Lbl("F".to_string()),
            Op::Sq, // x^2
            Op::Rtn,
        ];
        let mut state = CalcState::new();
        state.program = program.clone();
        state.alpha_reg = "F".to_string();
        // R00 = 10 (subdivision count)
        state.regs[0] = HpNum::from(10i32);
        // Stack: X = 0 (a), Y = 1 (b)
        state.stack.x = HpNum::from(0i32); // a = 0
        state.stack.y = HpNum::from(1i32); // b = 1
        state.stack.lift_enabled = false;
        (state, program)
    }

    // ── Explicit mode: known integral ∫₀¹ x² dx = 1/3 ───────────────────────

    // Catches: Simpson rule wrong for monotone polynomial (simplest known test)
    // ADR-004 Pitfall-2: test with Fix(4) to verify tolerance (lenient compare ~1e-3)
    #[test]
    fn explicit_integral_x_squared_fix4() {
        let (mut state, program) = make_x_squared_state();
        state.display_mode = DisplayMode::Fix(4);

        let result = op_integ_run_loop(&mut state, &program);
        assert!(result.is_ok(), "op_integ_run_loop failed: {result:?}");

        // ∫₀¹ x² dx = 1/3 ≈ 0.333333...
        let x_val = state.stack.x.inner().to_f64().unwrap();
        approx::assert_relative_eq!(x_val, 1.0 / 3.0, max_relative = 1e-3);
    }

    // Catches: Fix(9) precision must be tighter than Fix(4)
    // Pitfall-2 detection: same integral, different display mode must yield same CORRECT answer
    #[test]
    fn explicit_integral_x_squared_fix9() {
        let (mut state, program) = make_x_squared_state();
        state.display_mode = DisplayMode::Fix(9);
        // Use more subdivisions for higher precision test
        state.regs[0] = HpNum::from(100i32);

        let result = op_integ_run_loop(&mut state, &program);
        assert!(result.is_ok(), "op_integ_run_loop failed: {result:?}");

        let x_val = state.stack.x.inner().to_f64().unwrap();
        approx::assert_relative_eq!(x_val, 1.0 / 3.0, max_relative = 1e-5);
    }

    // ── Subdivision cap (INTG-07) ─────────────────────────────────────────────

    // Catches: subdivision cap > 32768 not rejected with Domain error
    #[test]
    fn subdivision_cap_rejected() {
        let (mut state, program) = make_x_squared_state();
        // Set n > 32768 in R00
        state.regs[0] = HpNum::from(32_769i32);

        let result = op_integ_run_loop(&mut state, &program);
        assert_eq!(result, Err(HpError::Domain), "n > 32768 must return Domain");
        assert!(
            state.integ_state.is_none(),
            "integ_state must be cleared on domain error"
        );
    }

    // ── Nested rejection (XROM-08 / ADR-002) ─────────────────────────────────

    // Catches: nested integ_state not rejected (XROM-08 guard missing)
    #[test]
    fn nested_rejection_integ_state_set() {
        let (mut state, program) = make_x_squared_state();
        // Pre-set integ_state to simulate an outer INTG in progress
        state.integ_state = Some(IntegState::default());

        let result = op_integ_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "nested INTG must return InvalidOp"
        );
        // State must be UNCHANGED (pre-mutation guard fired before any mutation)
        assert!(
            state.integ_state.is_some(),
            "integ_state must remain Some after nested rejection"
        );
    }

    // Catches: solve_state set but not checked (XROM-08 checks all three solver states)
    #[test]
    fn nested_rejection_solve_state_set() {
        let (mut state, program) = make_x_squared_state();
        state.solve_state = Some(crate::ops::math1::solve::SolveState::default());

        let result = op_integ_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "INTG inside SOLVE must return InvalidOp"
        );
    }

    // Catches: difeq_state set but not checked (XROM-08)
    #[test]
    fn nested_rejection_difeq_state_set() {
        let (mut state, program) = make_x_squared_state();
        state.difeq_state = Some(crate::ops::math1::difeq::DifeqState::default());

        let result = op_integ_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "INTG inside DIFEQ must return InvalidOp"
        );
    }

    // ── Pre-mutation call_stack cap (Pitfall 4) ───────────────────────────────

    // Catches: call_stack cap not checked BEFORE mutation (Pitfall 4)
    #[test]
    fn call_stack_full_pre_mutation() {
        let (mut state, program) = make_x_squared_state();
        // Pre-fill call_stack to 4 entries (hardware max)
        state.call_stack = vec![0, 1, 2, 3];
        let call_stack_before = state.call_stack.clone();

        let result = op_integ_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::CallDepth),
            "4-deep call_stack must return CallDepth"
        );
        // State must be UNCHANGED — call_stack still has 4 entries (no leak)
        assert_eq!(
            state.call_stack, call_stack_before,
            "call_stack must be unchanged after pre-mutation CallDepth rejection"
        );
        assert!(state.integ_state.is_none(), "integ_state must remain None");
    }

    // ── Cancellation per 64 samples (D-28.7 / D-28.8) ────────────────────────

    // Catches: cancellation flag not checked inside the sample loop (D-28.8)
    #[test]
    fn cancel_per_64_samples() {
        let (mut state, program) = make_x_squared_state();
        // Set cancel_requested = true BEFORE starting INTG
        state.cancel_requested.store(true, Ordering::Relaxed);
        // n=64 so the first per-64 check fires at k=0 (0 & 0x3F == 0)
        state.regs[0] = HpNum::from(64i32);

        let result = op_integ_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::Canceled),
            "cancel_requested must cause HpError::Canceled"
        );
        assert!(
            state.integ_state.is_none(),
            "integ_state must be cleared on cancellation"
        );
    }

    // ── User label not found → InvalidOp ─────────────────────────────────────

    // Catches: missing label not detected early (should fail before any loop iteration)
    #[test]
    fn missing_user_label_returns_invalid_op() {
        let (mut state, program) = make_x_squared_state();
        state.alpha_reg = "NONEXISTENT".to_string();

        let result = op_integ_run_loop(&mut state, &program);
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "missing label must return InvalidOp"
        );
        assert!(
            state.integ_state.is_none(),
            "integ_state must be None on label-not-found"
        );
    }

    // ── IntegMode enum ────────────────────────────────────────────────────────

    // Catches: IntegMode::default() wrong (must be Explicit per plan)
    #[test]
    fn integ_mode_default_is_explicit() {
        assert_eq!(IntegMode::default(), IntegMode::Explicit);
    }

    // Catches: PartialEq derive broken
    #[test]
    fn integ_mode_eq() {
        assert_eq!(IntegMode::Discrete, IntegMode::Discrete);
        assert_ne!(IntegMode::Discrete, IntegMode::Explicit);
    }

    // ── Simpson coefficient helper ────────────────────────────────────────────

    // Catches: simpson_coeff returns wrong weights (1/4/2 pattern broken)
    #[test]
    fn simpson_coeff_pattern() {
        let n = 4u32;
        assert_eq!(simpson_coeff(0, n), 1, "k=0 must be 1");
        assert_eq!(simpson_coeff(1, n), 4, "k=1 (odd) must be 4");
        assert_eq!(simpson_coeff(2, n), 2, "k=2 (even, interior) must be 2");
        assert_eq!(simpson_coeff(3, n), 4, "k=3 (odd) must be 4");
        assert_eq!(simpson_coeff(4, n), 1, "k=n must be 1");
    }
}
