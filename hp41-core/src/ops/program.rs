//! Phase 3 programming engine: LBL, GTO, XEQ, RTN, PRGM, Test, ISG, DSE, run_program.
//!
//! All programming ops have LiftEffect: Neutral (they do not modify lift_enabled).
//! run_program() is the public interpreter entry point exported via lib.rs.
//!
//! Key design constraints:
//!   - run_program() clones state.program to avoid a borrow conflict: dispatch() needs &mut CalcState
//!     while iterating over state.program (D-06)
//!   - execute_op() is a private helper that does NOT call flush_entry_buf — the entry buffer
//!     must not be reset mid-execution
//!   - ISG/DSE parse counters by string-split, never floor()/fmod() (ADR-001, D-10)

use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::{Op, TestKind};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

// ── Public op dispatch functions ─────────────────────────────────────────────
// Called from dispatch() match arms (added in plan 03-06).

/// LBL: no-op during interactive execution — a label is a marker only.
/// LiftEffect: Neutral.
pub fn op_lbl(_state: &mut CalcState) -> Result<(), HpError> {
    // LBL is a recording marker; executing it interactively is a no-op.
    // Inside run_loop, Lbl arms are handled directly (also no-op).
    Ok(())
}

/// PrgmMode: enter PRGM recording mode (toggle enter path).
/// The exit path (prgm_mode=true + Op::PrgmMode) is handled in dispatch() gate directly.
/// LiftEffect: Neutral.
pub fn op_prgm_mode(state: &mut CalcState) -> Result<(), HpError> {
    state.prgm_mode = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// GTO: unconditional branch to label. Only meaningful when is_running.
/// Interactive GTO outside a running program → InvalidOp; HP-41 supports interactive GTO,
/// but this emulator keeps the interactive dispatch path simple by not implementing it.
/// LiftEffect: Neutral.
pub fn op_gto(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        return Err(HpError::InvalidOp);
    }
    let target = find_label_in_state(state, label)?;
    state.pc = target + 1; // execute step AFTER the Lbl marker (Pitfall 4)
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// XEQ: subroutine call. Enforces 4-level call stack limit (D-14).
///
/// Interactive XEQ (not running): tries the four-entry Card Reader
/// XEQ-by-name fallback (Phase 19 spec) before erroring. This is the
/// path the GUI uses — `dispatch(Op::Xeq("WPRGM"))` with `is_running=false`.
///
/// Programmatic XEQ inside `run_loop` is handled there directly (with
/// call-depth check + user-label scan + same builtin fallback) — this
/// function is never reached during program execution.
///
/// LiftEffect: Neutral.
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        // Built-in XEQ-by-name fallback for the four Card Reader ops.
        // No user-label scan here: user-program XEQ goes through run_loop,
        // not op_xeq. If a user wants to call their own LBL interactively
        // they use run_program(state, label) directly, not dispatch.
        if let Some(card_op) = builtin_card_op(label) {
            return crate::ops::dispatch(state, card_op);
        }
        return Err(HpError::InvalidOp);
    }
    // run_loop handles Op::Xeq directly. Reaching here while running is a
    // logic bug elsewhere — return InvalidOp as a safe guard.
    Err(HpError::InvalidOp)
}

/// RTN: return from subroutine. If call_stack is empty, terminates run (top-level RTN).
/// Interactive RTN when not running: no-op (call_stack is empty, nothing to pop).
/// LiftEffect: Neutral.
pub fn op_rtn(state: &mut CalcState) -> Result<(), HpError> {
    if let Some(return_pc) = state.call_stack.pop() {
        state.pc = return_pc;
    }
    // Empty call_stack = top-level RTN — run_loop breaks on next iteration.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Test: interactive dispatch arm — no-op (read-only conditional; result only meaningful inside run_loop).
/// Inside run_loop, Test is handled directly using evaluate_test().
/// LiftEffect: Neutral.
pub fn op_test(state: &mut CalcState, _kind: TestKind) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ISG n: increment register n by step; return true if loop should skip (new_current > final).
/// Uses string-split parsing per ADR-001 — never floor()/fmod() on f64.
/// LiftEffect: Neutral.
pub fn op_isg(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    if reg as usize >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let (current, final_val, step, frac_padded) = parse_counter(&state.regs[reg as usize])?;
    let new_current = current + step;
    state.regs[reg as usize] = build_counter(new_current, &frac_padded)?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(new_current > final_val) // true = skip next (loop exits, D-11)
}

/// DSE n: decrement register n by step; return true if loop should skip (new_current <= final).
/// LiftEffect: Neutral.
pub fn op_dse(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    if reg as usize >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let (current, final_val, step, frac_padded) = parse_counter(&state.regs[reg as usize])?;
    let new_current = current - step;
    state.regs[reg as usize] = build_counter(new_current, &frac_padded)?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(new_current <= final_val) // true = skip next (loop exits, D-11)
}

// ── Public interpreter entry point ───────────────────────────────────────────

/// Execute a recorded program starting at the given label.
///
/// Clones state.program to avoid Rust borrow conflict: cannot hold &program[pc]
/// and &mut state simultaneously (standard Rust ownership constraint).
/// HP-41 programs are at most 999 steps; the clone is negligible. (RESEARCH Pitfall 1)
///
/// D-06: sets is_running = true, resets to false even on error path.
pub fn run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError> {
    // Clone program — borrow conflict guard (D-06, RESEARCH Pitfall 1)
    let program = state.program.clone();

    // Linear scan for entry label (D-02). On miss, try the XEQ-by-name
    // fallback for the four Card Reader ops (Phase 19 spec). User labels
    // always take precedence — fallback only fires on a true miss.
    let start = match program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
    {
        Some(idx) => idx,
        None => {
            if let Some(op) = builtin_card_op(entry_label) {
                // Dispatch the built-in once and return — no program to run.
                // is_running stays false; we never enter run_loop.
                return crate::ops::dispatch(state, op);
            }
            return Err(HpError::InvalidOp);
        }
    };

    state.pc = start + 1; // execute step AFTER the Lbl marker (Pitfall 4)
    state.call_stack.clear();
    state.is_running = true;

    let result = run_loop(state, &program);

    state.is_running = false; // always reset, even on error (is_running safety reset pattern)
    result
}

// ── Private interpreter loop ──────────────────────────────────────────────────

/// Maximum steps per run_program execution — guards against infinite loops.
/// HP-41 programs are at most 999 steps; 1 000 000 allows generous loop counts.
const MAX_STEPS: u64 = 1_000_000;

fn run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    let mut steps: u64 = 0;
    loop {
        if steps >= MAX_STEPS {
            return Err(HpError::Overflow); // infinite-loop guard (CR-01)
        }
        steps += 1;
        if state.pc >= program.len() {
            // Ran off end of program = implicit top-level RTN
            break;
        }
        let op = program[state.pc].clone();
        state.pc += 1;

        match op {
            Op::Rtn => {
                match state.call_stack.pop() {
                    Some(return_pc) => state.pc = return_pc,
                    None => break, // top-level RTN = normal termination
                }
            }
            Op::Lbl(_) => {
                // No-op during execution — LBL is only a search target
            }
            Op::Gto(label) => {
                let target = find_in_program(program, &label)?;
                state.pc = target + 1;
            }
            Op::Xeq(label) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth); // D-13/D-14: error before mutation
                }
                // User-label lookup first; on miss fall back to the four
                // Card Reader built-ins (Phase 19 spec). Built-in dispatch
                // does NOT push the call stack — it's a single op, not a
                // subroutine call, so pc just advances.
                match find_in_program(program, &label) {
                    Ok(target) => {
                        state.call_stack.push(state.pc);
                        state.pc = target + 1;
                    }
                    Err(_) => {
                        if let Some(card_op) = builtin_card_op(&label) {
                            // No pc adjustment — the main run_loop advance at the top of
                            // this iteration already moved pc past the XEQ. A card op is a
                            // single instruction (not a control-flow change), so pc resumes
                            // at the step that follows the XEQ.
                            crate::ops::dispatch(state, card_op)?;
                        } else {
                            return Err(HpError::InvalidOp);
                        }
                    }
                }
            }
            Op::Test(kind) => {
                if !evaluate_test(state, &kind) {
                    state.pc += 1; // skip next step (D-09: skip-if-false)
                }
            }
            Op::Isg(reg) => {
                if op_isg(state, reg)? {
                    state.pc += 1; // loop exit: skip next
                }
            }
            Op::Dse(reg) => {
                if op_dse(state, reg)? {
                    state.pc += 1; // loop exit: skip next
                }
            }
            other => {
                // All other ops execute without flush_entry_buf (no digit entry mid-program)
                // and without prgm_mode check (RESEARCH Pitfall 2)
                execute_op(state, other)?;
            }
        }
    }
    Ok(())
}

// ── Private execute_op (no flush, no prgm_mode) ──────────────────────────────

/// Execute a non-programming op inside the interpreter loop.
///
/// MUST NOT call flush_entry_buf (no digit entry mid-program, RESEARCH Pitfall 2).
/// MUST NOT check prgm_mode (always false when is_running = true).
fn execute_op(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    use crate::ops::alpha::{op_alpha_append, op_alpha_backspace, op_alpha_clear, op_alpha_toggle};
    use crate::ops::arithmetic::{op_add, op_div, op_mul, op_sub};
    use crate::ops::math::{
        op_acos, op_asin, op_atan, op_cos, op_exp, op_int, op_ln, op_log, op_recip, op_set_deg,
        op_set_grad, op_set_rad, op_sin, op_sq, op_sqrt, op_tan, op_tenpow, op_ypow,
    };
    use crate::ops::registers::{op_clreg, op_rcl, op_sto, op_sto_arith, op_sto_arith_stack};
    use crate::ops::stack_ops::{op_chs, op_clx, op_enter, op_lastx, op_rdn, op_xy_swap};
    use crate::state::DisplayMode;

    match op {
        // Phase 1 arithmetic
        Op::Add => op_add(state),
        Op::Sub => op_sub(state),
        Op::Mul => op_mul(state),
        Op::Div => op_div(state),
        // Phase 1 stack ops
        Op::Enter => op_enter(state),
        Op::Clx => op_clx(state),
        Op::Chs => op_chs(state),
        Op::Rdn => op_rdn(state),
        Op::XySwap => op_xy_swap(state),
        Op::Lastx => op_lastx(state),
        Op::PushNum(v) => {
            enter_number(state, v);
            // PushNum inside a program enables lift so subsequent PushNums lift the stack
            // (mirrors flush_entry_buf execute-mode behavior: enter_number + LiftEffect::Enable)
            apply_lift_effect(state, LiftEffect::Enable);
            Ok(())
        }
        // Phase 2 math/trig/angle
        Op::Int => op_int(state),
        Op::Recip => op_recip(state),
        Op::Sqrt => op_sqrt(state),
        Op::Sq => op_sq(state),
        Op::YPow => op_ypow(state),
        Op::Ln => op_ln(state),
        Op::Log => op_log(state),
        Op::Exp => op_exp(state),
        Op::TenPow => op_tenpow(state),
        Op::Sin => op_sin(state),
        Op::Cos => op_cos(state),
        Op::Tan => op_tan(state),
        Op::Asin => op_asin(state),
        Op::Acos => op_acos(state),
        Op::Atan => op_atan(state),
        Op::SetDeg => op_set_deg(state),
        Op::SetRad => op_set_rad(state),
        Op::SetGrad => op_set_grad(state),
        Op::FmtFix(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Fix(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtSci(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Sci(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtEng(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Eng(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoReg(r) => op_sto(state, r),
        Op::RclReg(r) => op_rcl(state, r),
        Op::StoArith { reg, kind } => op_sto_arith(state, reg, kind),
        Op::StoArithStack { kind, stack_reg } => op_sto_arith_stack(state, stack_reg, kind),
        Op::Clreg => op_clreg(state),
        Op::AlphaToggle => op_alpha_toggle(state),
        Op::AlphaAppend(ch) => op_alpha_append(state, ch),
        Op::AlphaClear => op_alpha_clear(state),
        Op::AlphaBackspace => op_alpha_backspace(state),
        Op::UserMode => {
            state.user_mode = !state.user_mode;
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // ── Phase 6: Science & Engineering ───────────────────────────────────────
        Op::SigmaPlus => super::stats::op_sigma_plus(state),
        Op::SigmaMinus => super::stats::op_sigma_minus(state),
        Op::Mean => super::stats::op_mean(state),
        Op::Sdev => super::stats::op_sdev(state),
        Op::LR => super::stats::op_lr(state),
        Op::Yhat => super::stats::op_yhat(state),
        Op::Corr => super::stats::op_corr(state),
        Op::ClSigmaStat => super::stats::op_cl_sigma_stat(state),
        Op::HmsToH => super::hms::op_hms_to_h(state),
        Op::HToHms => super::hms::op_h_to_hms(state),
        Op::HmsAdd => super::hms::op_hms_add(state),
        Op::HmsSub => super::hms::op_hms_sub(state),
        // ── Phase 11: Print operations ───────────────────────────────────────────────
        Op::PRX => super::print::op_prx(state),
        Op::PRA => super::print::op_pra(state),
        Op::PRSTK => super::print::op_prstk(state),
        // ── Phase 12: Synthetic Programming ─────────────────────────────────
        Op::GetKey => super::registers::op_getkey(state),
        Op::Null => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoM => super::registers::op_sto_m(state),
        Op::StoN => super::registers::op_sto_n(state),
        Op::StoO => super::registers::op_sto_o(state),
        Op::RclM => super::registers::op_rcl_m(state),
        Op::RclN => super::registers::op_rcl_n(state),
        Op::RclO => super::registers::op_rcl_o(state),
        Op::SyntheticByte(b) => {
            if let Some(op) = super::synthetic_byte_to_op(b) {
                // Recursive — safe per the same invariant as in dispatch().
                execute_op(state, op)
            } else {
                Err(HpError::InvalidOp)
            }
        }
        // Inside a running program, card ops stage a request just like in
        // interactive dispatch. Back-to-back card ops without a frontend
        // drain in between surface as `HpError::CardData` rather than
        // silently dropping the prior request.
        Op::Wdta => super::cardreader_ops::op_wdta(state),
        Op::Rdta => super::cardreader_ops::op_rdta(state),
        Op::Wprgm => super::cardreader_ops::op_wprgm(state),
        Op::Rdprgm => super::cardreader_ops::op_rdprgm(state),
        // Programming ops handled by run_loop directly — must not reach here
        Op::Lbl(_)
        | Op::Gto(_)
        | Op::Xeq(_)
        | Op::Rtn
        | Op::PrgmMode
        | Op::Test(_)
        | Op::Isg(_)
        | Op::Dse(_) => Err(HpError::InvalidOp),
    }
}

// ── Public conditional test evaluator ────────────────────────────────────────

/// Evaluate a conditional test against the current stack.
/// Returns true if condition is TRUE (execute next step).
/// Returns false if condition is FALSE (skip next step, D-09).
/// Stack is NOT modified (LiftEffect: Neutral — read-only access to X and Y).
pub fn evaluate_test(state: &CalcState, kind: &TestKind) -> bool {
    let x = state.stack.x.inner();
    let y = state.stack.y.inner();
    let zero = Decimal::ZERO;
    match kind {
        TestKind::XEqZero => x == zero,
        TestKind::XNeZero => x != zero,
        TestKind::XLtZero => x < zero,
        TestKind::XGtZero => x > zero,
        TestKind::XLeZero => x <= zero,
        TestKind::XGeZero => x >= zero,
        TestKind::XEqY => x == y,
        TestKind::XNeY => x != y,
        TestKind::XLtY => x < y,
        TestKind::XGtY => x > y,
        TestKind::XLeY => x <= y,
        TestKind::XGeY => x >= y,
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Parse CCCCC.FFFDD counter format by string-splitting at '.'.
/// Returns (current, final, step, frac_padded_5_chars).
///
/// ADR-001: never use floor()/fmod() on f64.
/// D-10: left of decimal = current (i64); right padded to 5 chars;
///       first 3 = final count, last 2 = step (00 → 1).
///
/// CRITICAL: format!("{:0<5}", frac_part) pads RIGHT with zeros (left-align).
/// Do NOT use "{:0>5}" (pads LEFT = wrong field extraction).
pub fn parse_counter(n: &HpNum) -> Result<(i64, i64, i64, String), HpError> {
    let s = n.inner().to_string(); // rust_decimal normalises trailing zeros (e.g. 1.00500 → "1.005")
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let current: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // Pad RIGHT with zeros to exactly 5 chars (trailing-zero normalisation fix, RESEARCH Pitfall 3)
    let frac_padded = format!("{frac_part:0<5}");
    // Truncate if somehow longer than 5 (defensive; should not occur with valid HP-41 counters)
    let frac_padded = if frac_padded.len() > 5 {
        frac_padded[..5].to_string()
    } else {
        frac_padded
    };
    let final_val: i64 = frac_padded[..3].parse().map_err(|_| HpError::InvalidOp)?;
    let step_raw: i64 = frac_padded[3..5].parse().map_err(|_| HpError::InvalidOp)?;
    let step = if step_raw == 0 { 1 } else { step_raw }; // step 00 → 1 (D-10)
    Ok((current, final_val, step, frac_padded))
}

/// Reconstruct counter HpNum from updated current and the preserved frac_padded string.
/// Preserves the FFFDD fields exactly (only CCCCC changes, D-12).
fn build_counter(current: i64, frac_padded: &str) -> Result<HpNum, HpError> {
    let s = format!("{current}.{frac_padded}");
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::rounded(d))
}

/// Linear scan for a label in the cloned program slice (run_loop helper).
fn find_in_program(program: &[Op], label: &str) -> Result<usize, HpError> {
    program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == label))
        .ok_or(HpError::InvalidOp)
}

/// Linear scan for a label in state.program (interactive dispatch helpers op_gto/op_xeq).
fn find_label_in_state(state: &CalcState, label: &str) -> Result<usize, HpError> {
    state
        .program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == label))
        .ok_or(HpError::InvalidOp)
}

/// XEQ-by-name fallback: resolves the four Card Reader op names to their
/// `Op` variants. Returns `None` for anything else — including unknown
/// names, lowercase variants, and any built-in not in the Card Reader set.
///
/// Intended as the label-miss fallback in `run_program`, `run_loop` (the
/// `Op::Xeq` arm), and `op_xeq` — wired up in subsequent commits. User
/// `LBL "name"` matches will take precedence, matching real HP-41
/// `XEQ "name"` resolution order.
///
/// Deliberately *not* a general built-in dispatcher — Spec §"Out of Scope".
pub(super) fn builtin_card_op(name: &str) -> Option<Op> {
    match name {
        "WPRGM" => Some(Op::Wprgm),
        "RDPRGM" => Some(Op::Rdprgm),
        "WDTA" => Some(Op::Wdta),
        "RDTA" => Some(Op::Rdta),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod program_tests {
    use crate::error::HpError;
    use crate::num::HpNum;
    use crate::ops::program::{
        evaluate_test, op_dse, op_gto, op_isg, op_lbl, op_prgm_mode, op_rtn, op_test, op_xeq,
        parse_counter,
    };
    use crate::ops::{Op, TestKind};
    use crate::state::CalcState;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn state_with_program(ops: Vec<Op>) -> CalcState {
        CalcState {
            program: ops,
            ..Default::default()
        }
    }

    // ── Original 12 targeted tests for error paths ────────────────────────────

    #[test]
    fn test_run_program_label_not_found() {
        let mut state = CalcState::default();
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_run_program_is_running_reset_on_error() {
        let mut state = CalcState::default();
        // Error result intentionally discarded — this test only checks the is_running side-effect.
        let _ = crate::ops::program::run_program(&mut state, "A");
        assert!(
            !state.is_running,
            "is_running must be false after run_program error"
        );
    }

    #[test]
    fn test_call_depth_limit() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::Xeq("B".to_string()),
            Op::Rtn,
            Op::Lbl("B".to_string()),
            Op::Xeq("C".to_string()),
            Op::Rtn,
            Op::Lbl("C".to_string()),
            Op::Xeq("D".to_string()),
            Op::Rtn,
            Op::Lbl("D".to_string()),
            Op::Xeq("E".to_string()),
            Op::Rtn,
            Op::Lbl("E".to_string()),
            Op::Xeq("F".to_string()),
            Op::Rtn,
            Op::Lbl("F".to_string()),
            Op::Rtn,
        ];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(
            result,
            Err(HpError::CallDepth),
            "5th XEQ must exceed 4-level limit"
        );
    }

    #[test]
    fn test_max_steps_infinite_loop_guard() {
        let program = vec![Op::Lbl("A".to_string()), Op::Gto("A".to_string())];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(
            result,
            Err(HpError::Overflow),
            "Infinite loop must be caught by MAX_STEPS"
        );
    }

    #[test]
    fn test_op_isg_reg_out_of_bounds() {
        let mut state = CalcState::default();
        let result = op_isg(&mut state, 100);
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_op_dse_reg_out_of_bounds() {
        let mut state = CalcState::default();
        let result = op_dse(&mut state, 100);
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_op_gto_interactive_invalid() {
        let mut state = CalcState::default();
        let result = op_gto(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_op_xeq_interactive_invalid() {
        let mut state = CalcState::default();
        let result = op_xeq(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_gto_label_not_found_during_run() {
        let program = vec![Op::Lbl("A".to_string()), Op::Gto("MISSING".to_string())];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_parse_counter_canonical_phase3_example() {
        let n = HpNum(Decimal::from_str("1.005").unwrap());
        let (current, final_val, step, frac_padded) = parse_counter(&n).unwrap();
        assert_eq!(current, 1);
        assert_eq!(final_val, 5);
        assert_eq!(step, 1);
        assert_eq!(&frac_padded, "00500");
    }

    #[test]
    fn test_parse_counter_integer_only_register() {
        // A register with no decimal part (e.g. initialised to 5 without ISG setup):
        // frac = "" → padded = "00000" → final=0, step 00 → 1
        let n = HpNum(Decimal::from_str("5").unwrap());
        let (current, final_val, step, frac_padded) = parse_counter(&n).unwrap();
        assert_eq!(current, 5);
        assert_eq!(final_val, 0, "no decimal → final=0");
        assert_eq!(step, 1, "no decimal → step 00 → 1");
        assert_eq!(&frac_padded, "00000");
    }

    #[test]
    fn test_parse_counter_step_99_max_step() {
        // counter = 1.00099 → current=1, final=000=0, step=99
        let n = HpNum(Decimal::from_str("1.00099").unwrap());
        let (current, final_val, step, frac_padded) = parse_counter(&n).unwrap();
        assert_eq!(current, 1);
        assert_eq!(final_val, 0);
        assert_eq!(step, 99, "step field '99' must parse as 99");
        assert_eq!(&frac_padded, "00099");
    }

    #[test]
    fn test_isg_increments_and_then_skips() {
        let mut state = CalcState::default();
        state.regs[0] = HpNum(Decimal::from_str("4.005").unwrap());
        let result1 = op_isg(&mut state, 0).unwrap();
        assert!(
            !result1,
            "isg at current=4 (new=5 not > final=5): must NOT skip"
        );
        let result2 = op_isg(&mut state, 0).unwrap();
        assert!(result2, "isg at current=5 (new=6 > final=5): must skip");
    }

    #[test]
    fn test_rtn_interactive_noop() {
        let mut state = CalcState::default();
        assert!(state.call_stack.is_empty());
        let result = op_rtn(&mut state);
        assert!(result.is_ok());
        assert!(state.call_stack.is_empty());
    }

    // ── execute_op and run_loop coverage tests ────────────────────────────────

    #[test]
    fn test_program_arithmetic_add() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("3").unwrap())),
            Op::PushNum(HpNum(Decimal::from_str("4").unwrap())),
            Op::Add,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("7").unwrap()));
    }

    #[test]
    fn test_program_sub_mul_div() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("10").unwrap())),
            Op::PushNum(HpNum(Decimal::from_str("2").unwrap())),
            Op::Sub,
            Op::PushNum(HpNum(Decimal::from_str("3").unwrap())),
            Op::Mul,
            Op::PushNum(HpNum(Decimal::from_str("4").unwrap())),
            Op::Div,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("6").unwrap()));
    }

    #[test]
    fn test_program_stack_ops() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("5").unwrap())),
            Op::Enter,
            Op::Clx,
            Op::PushNum(HpNum(Decimal::from_str("3").unwrap())),
            Op::Chs,
            Op::PushNum(HpNum(Decimal::from_str("7").unwrap())),
            Op::XySwap,
            Op::Rdn,
            Op::Lastx,
        ];
        let mut state = state_with_program(program);
        assert!(crate::ops::program::run_program(&mut state, "A").is_ok());
    }

    #[test]
    fn test_program_sto_rcl_clreg() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("42").unwrap())),
            Op::StoReg(5),
            Op::Clreg,
            Op::RclReg(5),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    #[test]
    fn test_program_fmt_ops() {
        use crate::state::DisplayMode;
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::FmtFix(2),
            Op::FmtSci(3),
            Op::FmtEng(4),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.display_mode, DisplayMode::Eng(4));
    }

    #[test]
    fn test_program_alpha_ops() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::AlphaToggle,
            Op::AlphaAppend('H'),
            Op::AlphaAppend('I'),
            Op::AlphaBackspace,
            Op::AlphaClear,
            Op::AlphaToggle,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert!(state.alpha_reg.is_empty());
    }

    #[test]
    fn test_program_math_ops() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("4").unwrap())),
            Op::Sqrt,
            Op::Sq,
            Op::Int,
            Op::Recip,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("0.25").unwrap()));
    }

    #[test]
    fn test_program_runs_off_end() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("1").unwrap())),
        ];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert!(result.is_ok());
        assert!(!state.is_running);
    }

    #[test]
    fn test_program_lbl_noop_in_execution() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::Lbl("B".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("9").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("9").unwrap()));
    }

    #[test]
    fn test_program_test_op_skip() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("0").unwrap())),
            Op::Test(TestKind::XNeZero),
            Op::PushNum(HpNum(Decimal::from_str("99").unwrap())),
            Op::PushNum(HpNum(Decimal::from_str("7").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("7").unwrap()));
    }

    #[test]
    fn test_program_test_op_no_skip() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("0").unwrap())),
            Op::Test(TestKind::XEqZero),
            Op::PushNum(HpNum(Decimal::from_str("42").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("42").unwrap()));
    }

    #[test]
    fn test_program_user_mode_toggle() {
        let program = vec![Op::Lbl("A".to_string()), Op::UserMode];
        let mut state = state_with_program(program);
        assert!(!state.user_mode);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert!(state.user_mode);
    }

    #[test]
    fn test_program_isg_inside_program() {
        // counter 0.00103 → current=0, final=1, step=3; 0+3=3 > 1 → skip
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("0.00103").unwrap())),
            Op::StoReg(0),
            Op::Isg(0),
            Op::Gto("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("5").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("5").unwrap()));
    }

    #[test]
    fn test_program_dse_inside_program() {
        // counter 3.00103 → current=3, final=1, step=3; 3-3=0 <= 1 → skip
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("3.00103").unwrap())),
            Op::StoReg(0),
            Op::Dse(0),
            Op::Gto("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("8").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("8").unwrap()));
    }

    #[test]
    fn test_program_xeq_subroutine_returns() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("1").unwrap())),
            Op::Xeq("B".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("2").unwrap())),
            Op::Rtn,
            Op::Lbl("B".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("10").unwrap())),
            Op::Rtn,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("2").unwrap()));
    }

    #[test]
    fn test_evaluate_test_relational_variants() {
        let mut state = CalcState::default();
        state.stack.x = HpNum(Decimal::from_str("-3").unwrap());
        state.stack.y = HpNum(Decimal::from_str("5").unwrap());

        assert!(evaluate_test(&state, &TestKind::XLtZero));
        assert!(!evaluate_test(&state, &TestKind::XGtZero));
        assert!(evaluate_test(&state, &TestKind::XLeZero));
        assert!(!evaluate_test(&state, &TestKind::XGeZero));
        assert!(!evaluate_test(&state, &TestKind::XEqY));
        assert!(evaluate_test(&state, &TestKind::XNeY));
        assert!(evaluate_test(&state, &TestKind::XLtY));
        assert!(!evaluate_test(&state, &TestKind::XGtY));
        assert!(evaluate_test(&state, &TestKind::XLeY));
        assert!(!evaluate_test(&state, &TestKind::XGeY));
    }

    #[test]
    fn test_op_prgm_mode_sets_flag() {
        let mut state = CalcState::default();
        assert!(!state.prgm_mode);
        op_prgm_mode(&mut state).unwrap();
        assert!(state.prgm_mode);
    }

    #[test]
    fn test_op_lbl_interactive_noop() {
        let mut state = CalcState::default();
        assert!(op_lbl(&mut state).is_ok());
    }

    #[test]
    fn test_op_test_interactive_noop() {
        let mut state = CalcState::default();
        assert!(op_test(&mut state, TestKind::XEqZero).is_ok());
    }

    #[test]
    fn test_program_trig_and_exp_ops() {
        // Cover Op::Ln, Op::Log, Op::Exp, Op::TenPow, Op::YPow,
        //       Op::SetDeg, Op::SetRad, Op::SetGrad
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("1").unwrap())),
            Op::Exp,
            Op::Ln,
            Op::SetRad,
            Op::SetGrad,
            Op::SetDeg,
            Op::PushNum(HpNum(Decimal::from_str("100").unwrap())),
            Op::Log,
            Op::TenPow,
            Op::PushNum(HpNum(Decimal::from_str("2").unwrap())),
            Op::YPow,
        ];
        let mut state = state_with_program(program);
        assert!(crate::ops::program::run_program(&mut state, "A").is_ok());
    }

    #[test]
    fn test_program_trig_sin_cos_tan() {
        // Cover Op::Sin, Op::Cos, Op::Tan, Op::Asin, Op::Acos, Op::Atan
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("30").unwrap())),
            Op::Sin,
            Op::Asin,
            Op::PushNum(HpNum(Decimal::from_str("60").unwrap())),
            Op::Cos,
            Op::Acos,
            Op::PushNum(HpNum(Decimal::from_str("45").unwrap())),
            Op::Tan,
            Op::Atan,
        ];
        let mut state = state_with_program(program);
        assert!(crate::ops::program::run_program(&mut state, "A").is_ok());
    }

    #[test]
    fn test_program_fmt_invalid_n_errors() {
        // Cover FmtFix/FmtSci/FmtEng > 9 error paths inside execute_op
        let program_fix = vec![
            Op::Lbl("A".to_string()),
            Op::FmtFix(10), // n > 9 → InvalidOp
        ];
        let mut state = state_with_program(program_fix);
        assert_eq!(
            crate::ops::program::run_program(&mut state, "A"),
            Err(HpError::InvalidOp)
        );

        let program_sci = vec![Op::Lbl("A".to_string()), Op::FmtSci(10)];
        let mut state2 = state_with_program(program_sci);
        assert_eq!(
            crate::ops::program::run_program(&mut state2, "A"),
            Err(HpError::InvalidOp)
        );

        let program_eng = vec![Op::Lbl("A".to_string()), Op::FmtEng(10)];
        let mut state3 = state_with_program(program_eng);
        assert_eq!(
            crate::ops::program::run_program(&mut state3, "A"),
            Err(HpError::InvalidOp)
        );
    }

    #[test]
    fn test_program_sto_arith() {
        // Cover Op::StoArith inside execute_op
        use crate::ops::StoArithKind;
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("10").unwrap())),
            Op::StoReg(0),
            Op::PushNum(HpNum(Decimal::from_str("5").unwrap())),
            Op::StoArith {
                reg: 0,
                kind: StoArithKind::Add,
            },
            Op::RclReg(0),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("15").unwrap()));
    }

    #[test]
    fn builtin_card_op_resolves_four_names() {
        use crate::ops::program::builtin_card_op;
        use crate::ops::Op;
        assert_eq!(builtin_card_op("WPRGM"), Some(Op::Wprgm));
        assert_eq!(builtin_card_op("RDPRGM"), Some(Op::Rdprgm));
        assert_eq!(builtin_card_op("WDTA"), Some(Op::Wdta));
        assert_eq!(builtin_card_op("RDTA"), Some(Op::Rdta));
        assert_eq!(
            builtin_card_op("wprgm"),
            None,
            "case-sensitive — HP-41 names are uppercase"
        );
        assert_eq!(builtin_card_op("UNKNOWN"), None);
        assert_eq!(builtin_card_op(""), None);
    }
}
