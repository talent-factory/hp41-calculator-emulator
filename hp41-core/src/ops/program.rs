//! Phase 3 programming engine: LBL, GTO, XEQ, RTN, PRGM, Test, ISG, DSE, run_program.
//!
//! All programming ops have LiftEffect: Neutral (they do not modify lift_enabled).
//! run_program() is the public interpreter entry point exported via lib.rs.
//!
//! Key design constraints:
//!   - run_program() clones state.program to avoid Rust borrow conflict (D-06, RESEARCH Pitfall 1)
//!   - execute_op() is a private helper that does NOT call flush_entry_buf (RESEARCH Pitfall 2)
//!   - ISG/DSE parse counters by string-split, never floor()/fmod() (ADR-001, D-10)

use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::{Op, TestKind};
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};

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
/// Interactive GTO (not running, not recording) → InvalidOp (Claude's Discretion / Pitfall 7).
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
/// Interactive XEQ (not running) → InvalidOp; XEQ inside a running program is
/// handled by run_loop directly (not this function). Phase 4 TUI can add
/// interactive subroutine-run support via run_program().
/// LiftEffect: Neutral.
pub fn op_xeq(state: &mut CalcState, _label: &str) -> Result<(), HpError> {
    if !state.is_running {
        return Err(HpError::InvalidOp);
    }
    // run_loop handles Op::Xeq directly (with call-depth check and label search).
    // This arm is only reached if someone calls op_xeq() outside run_loop,
    // which should not happen — return InvalidOp as a safe guard.
    Err(HpError::InvalidOp)
}

/// RTN: return from subroutine. If call_stack is empty, terminates run (top-level RTN).
/// Interactive RTN when not running: no-op (Claude's Discretion).
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

    // Linear scan for entry label (D-02)
    let start = program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
        .ok_or(HpError::InvalidOp)?;

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
                // find target before pushing to call_stack (error-before-mutation)
                let target = find_in_program(program, &label)?;
                state.call_stack.push(state.pc);
                state.pc = target + 1;
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
    use crate::ops::arithmetic::{op_add, op_sub, op_mul, op_div};
    use crate::ops::stack_ops::{op_enter, op_clx, op_chs, op_rdn, op_xy_swap, op_lastx};
    use crate::ops::math::{
        op_recip, op_sqrt, op_sq, op_ypow, op_ln, op_log, op_exp, op_tenpow,
        op_sin, op_cos, op_tan, op_asin, op_acos, op_atan,
        op_set_deg, op_set_rad, op_set_grad,
    };
    use crate::ops::registers::{op_sto, op_rcl, op_sto_arith, op_clreg};
    use crate::ops::alpha::{op_alpha_toggle, op_alpha_append, op_alpha_clear};
    use crate::state::DisplayMode;

    match op {
        // Phase 1 arithmetic
        Op::Add    => op_add(state),
        Op::Sub    => op_sub(state),
        Op::Mul    => op_mul(state),
        Op::Div    => op_div(state),
        // Phase 1 stack ops
        Op::Enter  => op_enter(state),
        Op::Clx    => op_clx(state),
        Op::Chs    => op_chs(state),
        Op::Rdn    => op_rdn(state),
        Op::XySwap => op_xy_swap(state),
        Op::Lastx  => op_lastx(state),
        Op::PushNum(v) => {
            enter_number(state, v);
            // PushNum inside a program enables lift so subsequent PushNums lift the stack
            // (mirrors flush_entry_buf execute-mode behavior: enter_number + LiftEffect::Enable)
            apply_lift_effect(state, LiftEffect::Enable);
            Ok(())
        }
        // Phase 2 math/trig/angle
        Op::Recip  => op_recip(state),
        Op::Sqrt   => op_sqrt(state),
        Op::Sq     => op_sq(state),
        Op::YPow   => op_ypow(state),
        Op::Ln     => op_ln(state),
        Op::Log    => op_log(state),
        Op::Exp    => op_exp(state),
        Op::TenPow => op_tenpow(state),
        Op::Sin    => op_sin(state),
        Op::Cos    => op_cos(state),
        Op::Tan    => op_tan(state),
        Op::Asin   => op_asin(state),
        Op::Acos   => op_acos(state),
        Op::Atan   => op_atan(state),
        Op::SetDeg => op_set_deg(state),
        Op::SetRad => op_set_rad(state),
        Op::SetGrad => op_set_grad(state),
        Op::FmtFix(n) => {
            if n > 9 { return Err(HpError::InvalidOp); }
            state.display_mode = DisplayMode::Fix(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtSci(n) => {
            if n > 9 { return Err(HpError::InvalidOp); }
            state.display_mode = DisplayMode::Sci(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtEng(n) => {
            if n > 9 { return Err(HpError::InvalidOp); }
            state.display_mode = DisplayMode::Eng(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoReg(r)              => op_sto(state, r),
        Op::RclReg(r)              => op_rcl(state, r),
        Op::StoArith { reg, kind } => op_sto_arith(state, reg, kind),
        Op::Clreg                  => op_clreg(state),
        Op::AlphaToggle            => op_alpha_toggle(state),
        Op::AlphaAppend(ch)        => op_alpha_append(state, ch),
        Op::AlphaClear             => op_alpha_clear(state),
        // Programming ops handled by run_loop directly — must not reach here
        Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode
        | Op::Test(_) | Op::Isg(_) | Op::Dse(_) => Err(HpError::InvalidOp),
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
        TestKind::XEqY    => x == y,
        TestKind::XNeY    => x != y,
        TestKind::XLtY    => x < y,
        TestKind::XGtY    => x > y,
        TestKind::XLeY    => x <= y,
        TestKind::XGeY    => x >= y,
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
    let frac_padded = format!("{:0<5}", frac_part);
    // Truncate if somehow longer than 5 (defensive; should not occur with valid HP-41 counters)
    let frac_padded = if frac_padded.len() > 5 { frac_padded[..5].to_string() } else { frac_padded };
    let final_val: i64 = frac_padded[..3].parse().map_err(|_| HpError::InvalidOp)?;
    let step_raw: i64  = frac_padded[3..5].parse().map_err(|_| HpError::InvalidOp)?;
    let step = if step_raw == 0 { 1 } else { step_raw }; // step 00 → 1 (D-10)
    Ok((current, final_val, step, frac_padded))
}

/// Reconstruct counter HpNum from updated current and the preserved frac_padded string.
/// Preserves the FFFDD fields exactly (only CCCCC changes, D-12).
fn build_counter(current: i64, frac_padded: &str) -> Result<HpNum, HpError> {
    let s = format!("{}.{}", current, frac_padded);
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
