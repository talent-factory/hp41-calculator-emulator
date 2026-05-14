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
/// XEQ-by-name fallback before erroring. This is the
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

// ── Phase 22: Program editing primitives (D-22.7, D-22.8, D-22.9, D-22.10) ──
// CLP / DEL / INS are PRGM-mode editing primitives — they mutate state.program
// directly and NEVER get recorded into the program buffer. The dispatch gate
// in mod.rs special-cases them so they execute immediately while prgm_mode == true.
//
// Stubs land here in task 22-02-01 so the workspace compiles after the variants
// are added. Real bodies fill in over the next two tasks (22-02-02 / 22-02-03).

/// Phase 22 D-22.7 (FN-PROG-03). Clear program from `Op::Lbl("label")` to the
/// next `Op::Lbl(_)` (or end-of-Vec if no further LBL).
///
/// Cursor reposition (Pitfall 6): after the drain, `state.pc` is set to `start`
/// (clamped to the post-drain `state.program.len()`) so the cursor lands at the
/// start of whatever block was deleted. Missing label → `HpError::InvalidOp`.
/// PRGM-mode only (D-22.10) — interactive dispatch with `prgm_mode == false`
/// returns InvalidOp via the defense-in-depth guard.
///
/// Documented divergence: HP-41 hardware uses END/.END. markers; we use
/// next-LBL boundaries because the flat-Vec program model has no explicit
/// END marker. (RESEARCH §1 D-22.7 row, OQ resolved as Option B.)
pub fn op_clp(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.prgm_mode {
        return Err(HpError::InvalidOp);
    }
    let start = state
        .program
        .iter()
        .position(|op| matches!(op, Op::Lbl(n) if n == label))
        .ok_or(HpError::InvalidOp)?;
    let end = state
        .program
        .iter()
        .skip(start + 1)
        .position(|op| matches!(op, Op::Lbl(_)))
        .map(|i| start + 1 + i)
        .unwrap_or(state.program.len());
    state.program.drain(start..end);
    // Pitfall 6: cursor lands at start of deleted block, clamped to new len
    // (protects against the rare case where start == post-drain program.len()).
    state.pc = start.min(state.program.len());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Phase 22 D-22.9 (FN-PROG-04). Delete `nnn` program steps starting at
/// `state.pc`. `nnn` silently clamps to `min(nnn, program.len() - pc)`;
/// `nnn == 0` OR `pc == program.len()` → no-op. PRGM-mode only (D-22.10).
///
/// `state.pc` is UNCHANGED — drain shifts the trailing tail down to fill the
/// gap, so the cursor naturally points at the next surviving step.
pub fn op_del(state: &mut CalcState, nnn: u8) -> Result<(), HpError> {
    if !state.prgm_mode {
        return Err(HpError::InvalidOp);
    }
    // D-22.9 clamping: saturating_sub guards against the pathological pc > len
    // (shouldn't happen, but keeps the helper bounds-safe under any state).
    let n = (nnn as usize).min(state.program.len().saturating_sub(state.pc));
    if n == 0 {
        // No-op for nnn == 0 OR pc == program.len()
        apply_lift_effect(state, LiftEffect::Neutral);
        return Ok(());
    }
    state.program.drain(state.pc..state.pc + n);
    // state.pc deliberately unchanged: drain shifts the tail down so pc
    // naturally falls at the same index (which is the post-drain position).
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Phase 22 D-22.8 (FN-PROG-05). Insert `Op::Null` (no-op placeholder from
/// Phase 12) at `state.pc`. PRGM-mode only (D-22.10).
///
/// `state.pc` is UNCHANGED — the cursor still points at the freshly inserted
/// Null. This matches HP-41 hardware "INS lands a blank step at cursor" behavior.
pub fn op_ins(state: &mut CalcState) -> Result<(), HpError> {
    if !state.prgm_mode {
        return Err(HpError::InvalidOp);
    }
    state.program.insert(state.pc, Op::Null);
    // state.pc deliberately unchanged — cursor still on the new Null.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Phase 22: Memory management (D-22.11) ────────────────────────────────────

/// Phase 22 D-22.11 / FN-MEM-01. Resize `state.regs` to `nnn` slots.
///
/// OQ-2 (AMENDED 2026-05-14): `nnn == 0` silently clamps to 1 (documented
/// divergence from real HP-41 which accepts `SIZE 000`). `nnn > 319`
/// returns `HpError::InvalidOp`. Otherwise `state.regs.resize(target,
/// HpNum::zero())`: shrinking truncates the tail (hardware-faithful
/// "MEM LOST"); growing zero-fills the new slots. Preserves values where
/// the old and new ranges overlap.
///
/// SAFETY: every legacy register access (op_sto/op_rcl/op_sto_arith/op_view/
/// op_clreg/Σ-family) was audited in 22-03-01..03 to honor `state.regs.len()`
/// dynamically, so shrinking via SIZE will NOT panic. The Σ-family
/// additionally fails closed when `state.regs.len() < 7` (Pitfall 5).
///
/// LiftEffect: Neutral.
pub fn op_size(state: &mut CalcState, nnn: u16) -> Result<(), HpError> {
    if nnn > 319 {
        return Err(HpError::InvalidOp);
    }
    let target = nnn.max(1) as usize; // OQ-2: SIZE 0 → silently clamp to 1
    state.regs.resize(target, crate::num::HpNum::zero());
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
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
    // fallback for the four Card Reader ops. User labels always take
    // precedence — fallback only fires on a true miss.
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

/// Resume a halted program from `state.pc`.
///
/// Mirror of [`run_program`] but skips the entry-label search — `state.pc` is
/// the resume point. Used after `Op::Stop` (D-22.1) breaks `run_loop`, when the
/// user hits R/S to continue. Does NOT clear `state.call_stack`: pending XEQ
/// frames must survive a STOP/resume cycle so `RTN` behaves correctly
/// (D-22.2; planner PATTERNS.md §"resume_program()").
///
/// CRITICAL — Pitfall 2: do NOT use `?` to propagate the `run_loop` error.
/// Capture into `let result`, reset `is_running = false`, then return `result`.
/// The naive `run_loop(...)?` short-circuits before the cleanup and leaves
/// `state.is_running == true`. (RESEARCH §2 Pitfall 2.)
///
/// Phase 22 (FN-PROG-01).
pub fn resume_program(state: &mut CalcState) -> Result<(), HpError> {
    if state.pc >= state.program.len() {
        return Err(HpError::InvalidOp); // nothing to resume
    }
    let program = state.program.clone();
    state.is_running = true;
    let result = run_loop(state, &program);
    state.is_running = false; // ALWAYS reset, even on Err (Pitfall 2)
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
            // ── Phase 22 (D-22.15, FN-PROG-06): GTO IND nn ────────────────
            // Inline indirect resolver. Phase 24 will extract this into a
            // shared `resolve_indirect()` helper for ~15 IND variants.
            //
            // 1. Read register (bounds-safe via .get() — D-22.23 zero-panic).
            // 2. Truncate to integer; reject non-integer pointers (FN-IND-02).
            // 3. Stringify the integer and reuse find_in_program (mirrors Op::Gto).
            Op::GtoInd(reg) => {
                let pointer = state
                    .regs
                    .get(reg as usize)
                    .ok_or(HpError::InvalidOp)?
                    .clone();
                let int_part = pointer.trunc_int();
                if int_part != pointer {
                    return Err(HpError::InvalidOp);
                }
                let label_str = int_part.inner().to_string();
                let target = find_in_program(program, &label_str)?;
                state.pc = target + 1; // mirrors Op::Gto: pc → step AFTER LBL marker
            }
            // ── Phase 22 (D-22.15, FN-PROG-07): XEQ IND nn ────────────────
            // Same inline indirect resolver as Op::GtoInd, but performs a
            // subroutine call: push pc onto call_stack BEFORE redirecting.
            //
            // CRITICAL: the 4-deep call_stack guard is PRE-mutation (D-13 /
            // D-14 precedent of Op::Xeq at line 206). The check fires BEFORE
            // reading the register, so an over-deep call returns CallDepth
            // without partially mutating any state.
            //
            // No builtin_card_op fallback — indirect labels are numeric
            // strings only (the integer pointer route never resolves a
            // textual function name).
            Op::XeqInd(reg) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth); // pre-mutation atomicity
                }
                let pointer = state
                    .regs
                    .get(reg as usize)
                    .ok_or(HpError::InvalidOp)?
                    .clone();
                let int_part = pointer.trunc_int();
                if int_part != pointer {
                    return Err(HpError::InvalidOp);
                }
                let label_str = int_part.inner().to_string();
                let target = find_in_program(program, &label_str)?;
                state.call_stack.push(state.pc);
                state.pc = target + 1;
            }
            Op::Xeq(label) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth); // D-13/D-14: error before mutation
                }
                // User-label lookup first; on miss fall back to the four
                // Card Reader built-ins. Built-in dispatch does NOT push
                // the call stack — it's a single op, not a subroutine
                // call, so pc just advances.
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
            // ── Phase 21: Flag tests (skip next step pattern, mirrors Op::Test) ──
            // FS?/FC? skip the next step when the flag is in the "false" state.
            // FS?C/FC?C ALWAYS clear the flag as a side effect (RESEARCH A4), THEN
            // decide the skip based on the PRE-clear state.
            Op::FlagTest { kind, flag } => {
                use crate::ops::flags::{flag_clear, flag_get};
                use crate::ops::FlagTestKind;
                let is_set = flag_get(state.flags, flag);
                let should_skip = match kind {
                    FlagTestKind::IsSet => !is_set,
                    FlagTestKind::IsClear => is_set,
                    FlagTestKind::IsSetThenClear => {
                        state.flags = flag_clear(state.flags, flag);
                        !is_set
                    }
                    FlagTestKind::IsClearThenClear => {
                        state.flags = flag_clear(state.flags, flag);
                        is_set
                    }
                };
                if should_skip {
                    state.pc += 1;
                }
            }
            // ── Phase 22 D-22.1 / Pitfall 1: STOP breaks run_loop only — NO display_override write
            // (unlike Op::Prompt below). The previous step's display persists.
            // state.pc is already advanced past the STOP step by the top-of-iteration
            // `state.pc += 1` (line 189). FN-PROG-01.
            Op::Stop => break,
            // ── Phase 21: PROMPT — write ALPHA to display_override + break run_loop.
            // Full STOP/resume semantics deferred to Phase 22 (RESEARCH A5).
            Op::Prompt => {
                state.display_override = Some(state.alpha_reg.chars().take(24).collect::<String>());
                break;
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
        op_abs, op_acos, op_asin, op_atan, op_cos, op_exp, op_fact, op_frc, op_int, op_ln, op_log,
        op_mod, op_pct_change, op_pi, op_polar_to_rect, op_recip, op_rect_to_polar, op_rnd,
        op_set_deg, op_set_grad, op_set_rad, op_sign, op_sin, op_sq, op_sqrt, op_tan, op_tenpow,
        op_ypow,
    };
    use crate::ops::registers::{op_clreg, op_rcl, op_sto, op_sto_arith, op_sto_arith_stack};
    use crate::ops::stack_ops::{op_chs, op_clx, op_enter, op_lastx, op_r_up, op_rdn, op_xy_swap};
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
        Op::Rup => op_r_up(state),
        Op::XySwap => op_xy_swap(state),
        Op::Lastx => op_lastx(state),
        Op::Pi => op_pi(state),
        Op::PushNum(v) => {
            enter_number(state, v);
            // PushNum inside a program enables lift so subsequent PushNums lift the stack
            // (mirrors flush_entry_buf execute-mode behavior: enter_number + LiftEffect::Enable)
            apply_lift_effect(state, LiftEffect::Enable);
            Ok(())
        }
        // Phase 2 math/trig/angle
        Op::Int => op_int(state),
        // ── Phase 20 additions ──────────────────────────────────────────────
        Op::Rnd => op_rnd(state),
        Op::Frc => op_frc(state),
        Op::Abs => op_abs(state),
        Op::Sign => op_sign(state),
        Op::Fact => op_fact(state),
        Op::Recip => op_recip(state),
        Op::Sqrt => op_sqrt(state),
        Op::Sq => op_sq(state),
        Op::YPow => op_ypow(state),
        Op::Mod => op_mod(state),
        Op::PctChange => op_pct_change(state),
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
        Op::PolarToRect => op_polar_to_rect(state),
        Op::RectToPolar => op_rect_to_polar(state),
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
        // ── Phase 21: Flag operations ──────────────────────────────────────────
        Op::SfFlag(n) => super::flags::op_sf(state, n),
        Op::CfFlag(n) => super::flags::op_cf(state, n),
        // ── Phase 21: Display Control ─────────────────────────────────────────
        // Op::Prompt is intentionally omitted — it is handled by run_loop directly
        // (with the `break` side effect) and listed in the catch-all below so any
        // accidental reach into execute_op returns InvalidOp.
        Op::View(r) => super::display_ops::op_view(state, r),
        Op::AView => super::display_ops::op_aview(state),
        Op::Aon => super::display_ops::op_aon(state),
        Op::Aoff => super::display_ops::op_aoff(state),
        Op::Cld => super::display_ops::op_cld(state),
        // ── Phase 21: Sound ───────────────────────────────────────────────────
        Op::Beep => super::sound::op_beep(state),
        Op::Tone(n) => super::sound::op_tone(state, n),
        // ── Phase 22: PSE — pause display (D-22.4, FN-PROG-02, Pitfall 3) ────
        // Writes both channels: display_override (visible value) + event_buffer
        // ("PAUSE 1000" marker for frontend timing). run_loop does NOT break;
        // execution continues to the next step. display_override survives
        // subsequent run_loop iterations because run_loop calls execute_op
        // directly (NOT dispatch), so the dispatch-top clear at mod.rs:410
        // does not fire between iterations. The NEXT interactive dispatch
        // clears it — matches HP-41 "value visible until next key" semantic.
        // Pitfall 10: do NOT add flush_entry_buf here — dispatch already
        // called it; execute_op inside run_loop never sees stale entry_buf.
        Op::Pse => {
            let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode);
            state.display_override = Some(formatted);
            state.event_buffer.push("PAUSE 1000".to_string());
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // ── Phase 22: Memory management (D-22.11..13, FN-MEM-01..02) ──────────
        // SIZE executes fine inside run_loop — it is a regular dispatch op,
        // not a control-flow primitive. Does NOT join the programming-ops
        // catch-all below.
        Op::Size(n) => op_size(state, n),
        // D-22.13: Op::Cla delegates to op_alpha_clear (hardware-faithful
        // "CLA" listing). Op::AlphaClear (legacy v1.0) stays separate.
        Op::Cla => super::alpha::op_alpha_clear(state),
        // Programming ops handled by run_loop directly — must not reach here
        Op::Lbl(_)
        | Op::Gto(_)
        | Op::Xeq(_)
        | Op::Rtn
        | Op::PrgmMode
        | Op::Test(_)
        | Op::Isg(_)
        | Op::Dse(_)
        | Op::FlagTest { .. }
        | Op::Prompt
        | Op::Stop                              // Phase 22: STOP handled by run_loop break
        | Op::GtoInd(_)                         // Phase 22: GTO IND has run_loop arm
        | Op::XeqInd(_)                         // Phase 22: XEQ IND has run_loop arm
        | Op::Clp(_)                            // Phase 22: CLP is a PRGM-mode editing primitive
        | Op::Del(_)                            // Phase 22: DEL is a PRGM-mode editing primitive
        | Op::Ins => Err(HpError::InvalidOp),   // Phase 22: INS is a PRGM-mode editing primitive
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
/// Used as the label-miss fallback in `op_xeq`, `run_program`, and the
/// `Op::Xeq` arm of `run_loop`. User `LBL "name"` matches take precedence,
/// matching real HP-41 `XEQ "name"` resolution order.
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
