use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::{CalcState, DisplayMode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod alpha;
pub mod arithmetic;
pub mod cardreader_ops;
pub mod display_ops;
pub mod flags;
pub mod hms;
pub mod math;
pub mod print;
pub mod program;
pub mod registers;
pub mod sound;
pub mod stack_ops;
pub mod stats;

use alpha::{op_alpha_append, op_alpha_backspace, op_alpha_clear, op_alpha_toggle};
use arithmetic::{op_add, op_div, op_mul, op_sub};
use cardreader_ops::{op_rdprgm, op_rdta, op_wdta, op_wprgm};
use math::{
    op_abs, op_acos, op_asin, op_atan, op_cos, op_exp, op_fact, op_frc, op_int, op_ln, op_log,
    op_mod, op_pct_change, op_pi, op_polar_to_rect, op_recip, op_rect_to_polar, op_rnd,
    op_set_deg, op_set_grad, op_set_rad, op_sign, op_sin, op_sq, op_sqrt, op_tan, op_tenpow,
    op_ypow,
};
use registers::{
    op_clreg, op_getkey, op_rcl, op_rcl_m, op_rcl_n, op_rcl_o, op_sto, op_sto_arith,
    op_sto_arith_stack, op_sto_m, op_sto_n, op_sto_o,
};
use stack_ops::{op_chs, op_clx, op_enter, op_lastx, op_r_up, op_rdn, op_xy_swap};

/// STO arithmetic operation kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoArithKind {
    Add,
    Sub,
    Mul,
    Div,
}

/// Stack register target for STO arithmetic operations (STOA-03).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StackReg {
    Y,
    Z,
    T,
    LastX,
}

/// HP-41 flag-test kind — 4 total. Used in `Op::FlagTest { kind, flag }`.
///
/// Mirrors the `TestKind` / `StoArithKind` sub-enum precedent. The `?C` variants
/// (IsSetThenClear / IsClearThenClear) ALWAYS clear the flag as a side effect,
/// regardless of the test outcome (RESEARCH A4 — strict reading of FS?C / FC?C).
/// Phase 21 (FN-FLAG-02).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlagTestKind {
    /// FS? — skip next step when flag is NOT set (condition: "is set"; skip-if-false).
    IsSet,
    /// FC? — skip next step when flag is NOT clear.
    IsClear,
    /// FS?C — skip if NOT set; ALWAYS clear the flag afterward (RESEARCH A4).
    IsSetThenClear,
    /// FC?C — skip if NOT clear; ALWAYS clear the flag afterward.
    IsClearThenClear,
}

/// HP-41 conditional test kind — 12 total. Used in Op::Test(TestKind).
/// D-07: single enum covers all HP-41 conditionals (symmetric with StoArithKind pattern).
/// D-08: exact variant names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestKind {
    XEqZero,
    XNeZero,
    XLtZero,
    XGtZero,
    XLeZero,
    XGeZero,
    XEqY,
    XNeY,
    XLtY,
    XGtY,
    XLeY,
    XGeY,
}

/// HP-41 calculator operations.
///
/// Phase 1: Add, Sub, Mul, Div, Enter, Clx, Chs, Rdn, XySwap, Lastx, PushNum
/// Phase 2: Recip, Sqrt, Sq, YPow, PctChange, Ln, Log, Exp, TenPow, Sin, Cos, Tan,
///          Asin, Acos, Atan, SetDeg, SetRad, SetGrad,
///          FmtFix, FmtSci, FmtEng, StoReg, RclReg, StoArith, Clreg,
///          AlphaToggle, AlphaAppend, AlphaClear
/// Phase 3: Lbl, Gto, Xeq, Rtn, PrgmMode, Test, Isg, Dse
/// Phase 6: SigmaPlus, SigmaMinus, Mean, Sdev, LR, Yhat, Corr, ClSigmaStat,
///          HmsToH, HToHms, HmsAdd, HmsSub
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Op {
    // ── Arithmetic (Phase 1) ──────────────────────────────────────────
    Add,
    Sub,
    Mul,
    Div,
    // ── Stack operations (Phase 1) ───────────────────────────────────
    Enter,
    Clx,
    Chs,
    Rdn,
    /// R↑ — roll stack up (mirror of Rdn): X←T, T←Z, Z←Y, Y←X.
    /// LiftEffect: Neutral. Does NOT update LASTX (Phase 20, D-19/D-20).
    Rup,
    XySwap,
    Lastx,
    /// PI — push the constant π (3.141592654, 10-digit rounded HP-41 hardware value).
    /// LiftEffect: Enable (Phase 20, D-08/D-10).
    Pi,
    /// Push a numeric literal onto the stack (e.g., from keyboard digit entry).
    PushNum(HpNum),
    // ── Unary math (Phase 2) ─────────────────────────────────────────
    /// INT — truncate X toward zero (integer part). LiftEffect: Enable.
    Int,
    /// RND — round X to the precision of the current display mode (FIX/SCI/ENG n).
    /// LiftEffect: Enable (via unary_result). (Phase 20, D-01/D-02/D-03)
    Rnd,
    /// FRC — fractional part of X (sign-preserving, complement of INT).
    /// LiftEffect: Enable. (Phase 20, D-15)
    Frc,
    /// ABS — absolute value of X. LiftEffect: Enable. (Phase 20, D-16)
    Abs,
    /// SIGN — sign of X: -1 / 0 / +1. LiftEffect: Enable.
    /// (Phase 20 always returns numeric; SIGN-on-ALPHA divergence is documented
    /// in Phase 25 docs per D-18.)
    Sign,
    /// FACT — factorial of integer X. Domain error for non-integer or negative X.
    /// OutOfRange for X > 69 (hardware spec, D-06).
    /// Overflow for X ≥ 28 (Decimal range cap, D-05). LiftEffect: Enable.
    Fact,
    /// 1/x — reciprocal. LiftEffect: Enable.
    Recip,
    /// √x — square root. LiftEffect: Enable.
    Sqrt,
    /// x² — square. LiftEffect: Enable.
    Sq,
    /// Y^X — Y raised to power X (binary). LiftEffect: Enable.
    YPow,
    /// MOD — Y mod X with HP-41 trunc-toward-zero convention (D-14):
    /// result = Y − X · trunc(Y/X). Sign follows Y (matches HP-41C Owner's
    /// Manual + Free42 source). Examples: `7 MOD -3 = 1`, `-7 MOD 3 = -1`.
    /// Domain error if X = 0. LiftEffect: Enable (via binary_result).
    /// (Phase 20, D-14, FN-MATH-06)
    Mod,
    /// %CH — percent change ((X−Y)/Y)×100. Stack effect: unary (Y preserved, LASTX←X). LiftEffect: Enable.
    PctChange,
    /// LN — natural logarithm. LiftEffect: Enable.
    Ln,
    /// LOG — log base 10. LiftEffect: Enable.
    Log,
    /// e^x — natural exponential. LiftEffect: Enable.
    Exp,
    /// 10^x — base-10 exponential. LiftEffect: Enable.
    TenPow,
    // ── Trig (Phase 2) ───────────────────────────────────────────────
    /// SIN — sine in current angle_mode. LiftEffect: Enable.
    Sin,
    /// COS — cosine in current angle_mode. LiftEffect: Enable.
    Cos,
    /// TAN — tangent in current angle_mode. LiftEffect: Enable.
    Tan,
    /// ASIN — arcsine, result in current angle_mode. LiftEffect: Enable.
    Asin,
    /// ACOS — arccosine, result in current angle_mode. LiftEffect: Enable.
    Acos,
    /// ATAN — arctangent, result in current angle_mode. LiftEffect: Enable.
    Atan,
    /// P→R — convert polar (Y = magnitude, X = angle in current angle_mode)
    /// to rectangular (Y = x-coord, X = y-coord). LiftEffect: Enable.
    /// Direct stack assignment; LASTX ← consumed X. (Phase 20, D-11/D-12/D-13)
    PolarToRect,
    /// R→P — convert rectangular (Y = x-coord, X = y-coord) to polar
    /// (Y = magnitude, X = angle in current angle_mode). LiftEffect: Enable.
    /// Direct stack assignment; LASTX ← consumed X. (Phase 20, D-11/D-12/D-13)
    RectToPolar,
    // ── Angle mode (Phase 2) ─────────────────────────────────────────
    /// Set angle mode to DEG. LiftEffect: Neutral.
    SetDeg,
    /// Set angle mode to RAD. LiftEffect: Neutral.
    SetRad,
    /// Set angle mode to GRAD. LiftEffect: Neutral.
    SetGrad,
    // ── Display mode (Phase 2) ───────────────────────────────────────
    /// FIX n — fixed decimal display (n = 0–9). LiftEffect: Neutral.
    FmtFix(u8),
    /// SCI n — scientific notation display (n = 0–9). LiftEffect: Neutral.
    FmtSci(u8),
    /// ENG n — engineering notation display (n = 0–9). LiftEffect: Neutral.
    FmtEng(u8),
    // ── Storage registers (Phase 2) ──────────────────────────────────
    /// STO n — store X into register n (0–99). LiftEffect: Neutral.
    StoReg(u8),
    /// RCL n — recall register n into X (0–99). LiftEffect: Enable.
    RclReg(u8),
    /// STO+/−/×/÷ n — arithmetic on register n using X. LiftEffect: Neutral.
    StoArith {
        reg: u8,
        kind: StoArithKind,
    },
    /// STO+/−/×/÷ stack-reg — arithmetic on a stack register using X. LiftEffect: Neutral.
    StoArithStack {
        kind: StoArithKind,
        stack_reg: StackReg,
    },
    /// CLREG — clear all storage registers to zero. LiftEffect: Neutral.
    Clreg,
    // ── ALPHA mode (Phase 2) ─────────────────────────────────────────
    /// ALPHA — toggle alpha_mode flag. LiftEffect: Neutral.
    AlphaToggle,
    /// Append char to alpha_reg (max 24). LiftEffect: Neutral.
    AlphaAppend(char),
    /// CLRALPHA — clear alpha_reg. LiftEffect: Neutral.
    AlphaClear,
    // ── Programming (Phase 3) ────────────────────────────────────────────
    /// LBL "name" — program label marker. No-op during execution. LiftEffect: Neutral.
    /// D-01: labels are Op::Lbl markers stored in the flat program Vec.
    Lbl(String),
    /// GTO "name" — unconditional branch to label. LiftEffect: Neutral.
    Gto(String),
    /// XEQ "name" — subroutine call (max 4 deep, D-14). LiftEffect: Neutral.
    Xeq(String),
    /// RTN — return from subroutine; terminates run if call_stack is empty. LiftEffect: Neutral.
    Rtn,
    /// PRGM — toggle prgm_mode recording flag (D-03). LiftEffect: Neutral.
    PrgmMode,
    /// Conditional test — skip next step if condition is false (D-09). LiftEffect: Neutral.
    Test(TestKind),
    /// ISG n — increment register n by step, skip next if new_current > final (D-11). LiftEffect: Neutral.
    Isg(u8),
    /// DSE n — decrement register n by step, skip next if new_current <= final (D-11). LiftEffect: Neutral.
    Dse(u8),
    // ── USER mode (Phase 5) ──────────────────────────────────────────────
    /// USER mode toggle: flip state.user_mode. LiftEffect: Neutral.
    UserMode,
    // ── ALPHA backspace (Phase 5) ────────────────────────────────────────
    /// ALPHA backspace: remove last char from alpha_reg (HP-41 ← key). LiftEffect: Neutral.
    AlphaBackspace,
    // ── Science & Engineering (Phase 6) ─────────────────────────────────
    /// Σ+ — accumulate X and Y into Σ registers R01–R06; push count n into X. LiftEffect: Enable.
    SigmaPlus,
    /// Σ− — remove X and Y from Σ registers R01–R06; push count n into X. LiftEffect: Enable.
    SigmaMinus,
    /// MEAN — push x̄ to X and ȳ to Y from Σ registers. LiftEffect: Enable.
    Mean,
    /// SDEV — push sample σx to X and σy to Y (n-1 denominator). LiftEffect: Enable.
    Sdev,
    /// L.R. — linear regression: push slope m to Y and intercept b to X. LiftEffect: Enable.
    LR,
    /// YHAT — ŷ prediction: read x from X, push ŷ into X. LiftEffect: Enable.
    Yhat,
    /// CORR — correlation coefficient r in X. LiftEffect: Enable.
    Corr,
    /// CLΣSTAT — zero Σ registers R01–R06. LiftEffect: Neutral.
    ClSigmaStat,
    /// HMS→ — convert H.MMSS to decimal hours in X. LiftEffect: Enable.
    HmsToH,
    /// →HMS — convert decimal hours in X to H.MMSS. LiftEffect: Enable.
    HToHms,
    /// HMS+ — add two H.MMSS values (Y + X), result in X with stack drop. LiftEffect: Enable.
    HmsAdd,
    /// HMS− — subtract H.MMSS values (Y − X), result in X with stack drop. LiftEffect: Enable.
    HmsSub,
    // ── Print operations (Phase 11) ─────────────────────────────────────────────
    /// PRX — print X register in current display format, right-aligned to 24 chars. LiftEffect: Neutral.
    PRX,
    /// PRA — print ALPHA register, left-aligned to 24 chars. LiftEffect: Neutral.
    PRA,
    /// PRSTK — print full stack T/Z/Y/X/LASTX/ALPHA, 6 lines, 24 chars each. LiftEffect: Neutral.
    PRSTK,
    // ── Synthetic Programming (Phase 12) ────────────────────────────────────
    /// GETKEY — push last key code (HP-41 row×10+col) to X. LiftEffect: Enable.
    GetKey,
    /// NULL — true no-op; does not modify any state. LiftEffect: Neutral.
    Null,
    /// STO M — store X into hidden register M. LiftEffect: Neutral.
    StoM,
    /// STO N — store X into hidden register N. LiftEffect: Neutral.
    StoN,
    /// STO O — store X into hidden register O. LiftEffect: Neutral.
    StoO,
    /// RCL M — recall hidden register M into X. LiftEffect: Enable.
    RclM,
    /// RCL N — recall hidden register N into X. LiftEffect: Enable.
    RclN,
    /// RCL O — recall hidden register O into X. LiftEffect: Enable.
    RclO,
    /// SyntheticByte(u8) — synthetic op inserted via hex modal. At execution time,
    /// dispatches to the corresponding Op via `synthetic_byte_to_op()` lookup.
    /// LiftEffect: varies (delegates to the mapped op).
    SyntheticByte(u8),
    // ── Card Reader ─────────────────────────────────────────────────────────
    /// WDTA — write data registers R00..R(SIZE-1) to the card named in the
    /// ALPHA register. Stages a `CardOpRequest::WriteData` for the frontend
    /// to drain. Empty ALPHA → `HpError::AlphaData`. LiftEffect: Neutral.
    Wdta,
    /// RDTA — read a data card named in ALPHA and replace data registers.
    /// Stages a `CardOpRequest::ReadData`. LiftEffect: Neutral.
    Rdta,
    /// WPRGM — write current program to the `.raw` card named in ALPHA.
    /// Stages a `CardOpRequest::WriteProgram`. LiftEffect: Neutral.
    Wprgm,
    /// RDPRGM — read a `.raw` card named in ALPHA and insert its ops
    /// (replace if program empty, else insert after PC). LiftEffect: Neutral.
    Rdprgm,
    // ── Phase 21: Flags ──────────────────────────────────────────────────────
    /// SF n — set flag n (0..=55). LiftEffect: Neutral.
    SfFlag(u8),
    /// CF n — clear flag n (0..=55). LiftEffect: Neutral.
    CfFlag(u8),
    /// FS? / FC? / FS?C / FC?C n — conditional flag test (run_loop skips next
    /// step on false; `?C` variants also always-clear). LiftEffect: Neutral.
    /// Interactive dispatch is a no-op (mirrors `Op::Test` precedent — no next
    /// program step at the keyboard). Phase 21 (FN-FLAG-02).
    FlagTest {
        kind: FlagTestKind,
        flag: u8,
    },
    // ── Phase 21: Display Control ────────────────────────────────────────────
    /// VIEW n — show formatted value of register n (0..=99). LiftEffect: Neutral.
    /// Phase 21 (FN-DISP-01).
    View(u8),
    /// AVIEW — show ALPHA register on display (24-char truncate). LiftEffect: Neutral.
    /// Phase 21 (FN-DISP-02).
    AView,
    /// PROMPT — show ALPHA on display; inside `run_loop`, also `break` execution.
    /// LiftEffect: Neutral. Phase 21 (FN-DISP-03). Full STOP/resume deferred to Phase 22.
    Prompt,
    /// AON — enable ALPHA auto-display (set system flag 48, HP-42S compat).
    /// LiftEffect: Neutral. Phase 21 (FN-DISP-04).
    Aon,
    /// AOFF — disable ALPHA auto-display (clear system flag 48).
    /// LiftEffect: Neutral. Phase 21 (FN-DISP-04).
    Aoff,
    /// CLD — explicit clear of `display_override`. LiftEffect: Neutral.
    /// Phase 21 (FN-DISP-05).
    Cld,
    // ── Phase 21: Sound ──────────────────────────────────────────────────────
    /// BEEP — push `BEEP` event to event_buffer. LiftEffect: Neutral.
    /// Phase 21 (FN-SOUND-01).
    Beep,
    /// TONE n — push `TONE n` event to event_buffer (n=0..=9).
    /// LiftEffect: Neutral. Phase 21 (FN-SOUND-02).
    Tone(u8),
}

/// Flush the number entry buffer to the stack.
///
/// If entry_buf is non-empty, parse it as a Decimal and push onto the stack
/// via enter_number (respecting lift_enabled). Then set lift_enabled = true.
///
/// This MUST be called at the start of every dispatch() invocation so that
/// pending digit entry is committed before any operation consumes the stack.
///
/// Returns Err(HpError::InvalidOp) only if entry_buf contains unparseable content
/// (defensive guard; well-formed CLI input produces valid Decimal strings).
pub fn flush_entry_buf(state: &mut CalcState) -> Result<(), HpError> {
    if state.entry_buf.is_empty() {
        return Ok(());
    }
    // Clone entry_buf for parsing — do NOT clear yet. Clearing happens only on success
    // so that a parse error preserves the user's in-progress input (WR-02).
    let mut s = state.entry_buf.clone();
    // D-09 (Phase 9): trailing 'e' with no exponent digits is HP-41 hardware-faithful
    // shorthand for "exponent 00". Normalize by appending "00" so from_scientific accepts it.
    // Also handles trailing "e-" (CHS pressed with no exponent digits yet) → "e-00" = exponent 0.
    // We check both 'e'/'E' variants for safety even though entry_buf is always lowercase.
    if s.ends_with("e-") || s.ends_with("E-") || s.ends_with('e') || s.ends_with('E') {
        s.push_str("00");
    }
    let d = Decimal::from_str(&s)
        .or_else(|_| Decimal::from_scientific(&s))
        .map_err(|_| HpError::InvalidOp)?;
    // Parse succeeded — now clear the entry buffer.
    state.entry_buf.clear();
    let n = HpNum::rounded(d);
    if state.prgm_mode {
        // Recording mode: PushNum goes to program Vec, not stack (D-03/D-04).
        // lift_enabled is NOT changed — recording does not affect execution state.
        state.program.push(Op::PushNum(n));
    } else {
        // Execute mode: existing behaviour unchanged.
        crate::stack::enter_number(state, n);
        crate::stack::apply_lift_effect(state, LiftEffect::Enable);
    }
    Ok(())
}

/// Dispatch an operation to its implementation function.
///
/// This is the single entry point for all calculator operations in hp41-core.
/// Callers (hp41-cli, tests) call dispatch(state, op) and handle the Result.
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    flush_entry_buf(state)?; // commit any pending digit entry before executing op
                             // ── Phase 21 Pitfall-5: clear stale display override before op runs.
                             // VIEW/AVIEW/PROMPT write AFTER this line and so survive their own
                             // dispatch; the NEXT op's dispatch clears the override again — matches
                             // HP-41 hardware "VIEW shows until next key" behavior.
    state.display_override = None;
    // ── Phase 3: PRGM mode recording gate (D-03) ────────────────────────────
    if state.prgm_mode {
        // PrgmMode op while recording = exit recording immediately (toggle, Pitfall 6).
        // This op is NOT recorded — it executes immediately to restore normal dispatch.
        if matches!(op, Op::PrgmMode) {
            state.prgm_mode = false;
            apply_lift_effect(state, LiftEffect::Neutral);
            return Ok(());
        }
        // All other ops: append to program Vec; do NOT execute. Stack unmodified.
        state.program.push(op);
        return Ok(());
    }
    match op {
        // ── Phase 1 ops ──────────────────────────────────────────────────
        Op::Add => op_add(state),
        Op::Sub => op_sub(state),
        Op::Mul => op_mul(state),
        Op::Div => op_div(state),
        Op::Enter => op_enter(state),
        Op::Clx => op_clx(state),
        Op::Chs => op_chs(state),
        Op::Rdn => op_rdn(state),
        Op::Rup => op_r_up(state),
        Op::XySwap => op_xy_swap(state),
        Op::Lastx => op_lastx(state),
        Op::Pi => op_pi(state),
        Op::PushNum(v) => {
            crate::stack::enter_number(state, v);
            Ok(())
        }
        // ── Phase 2 math/trig/angle ops (Plan 04) ───────────────────────
        Op::Int => op_int(state),
        // ── Phase 20 unary math additions (D-15/D-16/D-17/D-04..D-07) ───
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
        // ── Phase 3: Programming ops ─────────────────────────────────────────
        // Note: PrgmMode exit (prgm_mode=true + Op::PrgmMode) is handled by the gate above.
        // PrgmMode entry (prgm_mode=false) reaches here and sets prgm_mode=true.
        Op::Lbl(_) => program::op_lbl(state),
        Op::Gto(s) => program::op_gto(state, &s),
        Op::Xeq(s) => program::op_xeq(state, &s),
        Op::Rtn => program::op_rtn(state),
        Op::PrgmMode => program::op_prgm_mode(state),
        Op::Test(kind) => program::op_test(state, kind),
        Op::Isg(reg) => {
            // op_isg returns Result<bool>; dispatch() returns Result<()>.
            // Discard the bool skip signal — skip semantics only apply inside run_loop.
            program::op_isg(state, reg).map(|_| ())
        }
        Op::Dse(reg) => {
            // Same as Isg: discard the bool skip signal — skip only applies inside run_loop.
            program::op_dse(state, reg).map(|_| ())
        }
        // ── Phase 6: Science & Engineering ───────────────────────────────────────
        Op::SigmaPlus => stats::op_sigma_plus(state),
        Op::SigmaMinus => stats::op_sigma_minus(state),
        Op::Mean => stats::op_mean(state),
        Op::Sdev => stats::op_sdev(state),
        Op::LR => stats::op_lr(state),
        Op::Yhat => stats::op_yhat(state),
        Op::Corr => stats::op_corr(state),
        Op::ClSigmaStat => stats::op_cl_sigma_stat(state),
        Op::HmsToH => hms::op_hms_to_h(state),
        Op::HToHms => hms::op_h_to_hms(state),
        Op::HmsAdd => hms::op_hms_add(state),
        Op::HmsSub => hms::op_hms_sub(state),
        // ── Phase 11: Print operations ───────────────────────────────────────────────
        Op::PRX => print::op_prx(state),
        Op::PRA => print::op_pra(state),
        Op::PRSTK => print::op_prstk(state),
        // ── Phase 12: Synthetic Programming ─────────────────────────────────
        Op::GetKey => op_getkey(state),
        Op::Null => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoM => op_sto_m(state),
        Op::StoN => op_sto_n(state),
        Op::StoO => op_sto_o(state),
        Op::RclM => op_rcl_m(state),
        Op::RclN => op_rcl_n(state),
        Op::RclO => op_rcl_o(state),
        Op::SyntheticByte(b) => {
            if let Some(op) = synthetic_byte_to_op(b) {
                // Recursive dispatch — safe: synthetic_byte_to_op never returns
                // Some(Op::SyntheticByte(_)), so recursion depth is exactly 1.
                dispatch(state, op)
            } else {
                Err(HpError::InvalidOp)
            }
        }
        // ── Card Reader ───────────────────────────────────────────────────
        Op::Wdta => op_wdta(state),
        Op::Rdta => op_rdta(state),
        Op::Wprgm => op_wprgm(state),
        Op::Rdprgm => op_rdprgm(state),
        // ── Phase 21: Flags ───────────────────────────────────────────────
        Op::SfFlag(n) => flags::op_sf(state, n),
        Op::CfFlag(n) => flags::op_cf(state, n),
        // Interactive FlagTest is a no-op (mirrors Op::Test). The skip-next-step
        // and always-clear semantics live in run_loop. At the keyboard there is
        // no "next program step" to skip; pc and flags are untouched.
        Op::FlagTest { .. } => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // ── Phase 21: Display Control ─────────────────────────────────────
        Op::View(r) => display_ops::op_view(state, r),
        Op::AView => display_ops::op_aview(state),
        Op::Prompt => display_ops::op_prompt(state),
        Op::Aon => display_ops::op_aon(state),
        Op::Aoff => display_ops::op_aoff(state),
        Op::Cld => display_ops::op_cld(state),
        // ── Phase 21: Sound ───────────────────────────────────────────────
        Op::Beep => sound::op_beep(state),
        Op::Tone(n) => sound::op_tone(state, n),
    }
}

// ── Phase 12: Synthetic Byte Subset (D-11, D-12) ─────────────────────────────
//
// Maps HP-41 NUT/FOCAL byte codes to already-implemented Op variants.
// This is a CONSERVATIVE initial table — covers ~15 well-known single-byte
// codes. Codes outside this table are rejected at the hex modal entry point
// (D-13: app.message = "INVALID"). Expandable in v2+ as part of SYNT-05.
//
// CRITICAL INVARIANT: this function MUST NOT return Some(Op::SyntheticByte(_))
// — that would cause infinite recursion in dispatch() / execute_op().
//
// [ASSUMED] — exact NUT byte codes from secondary sources. Cross-verify
// against HP-41 FOCAL reference if precision is needed for a specific code.

/// Map an HP-41 byte code to the corresponding Op, if it is in the safe subset.
/// Returns `None` for codes outside the curated subset.
pub fn synthetic_byte_to_op(byte: u8) -> Option<Op> {
    match byte {
        // Arithmetic (HP-41 single-byte FOCAL codes — [ASSUMED])
        0x40 => Some(Op::Add),
        0x41 => Some(Op::Sub),
        0x42 => Some(Op::Mul),
        0x43 => Some(Op::Div),
        // Stack ops
        0x4F => Some(Op::Chs),
        0x73 => Some(Op::Clx),
        0x74 => Some(Op::Rdn),
        0x71 => Some(Op::XySwap),
        // Math
        0x52 => Some(Op::Sqrt),
        0x53 => Some(Op::Sq),
        0x57 => Some(Op::Log),
        0x67 => Some(Op::Ln),
        0x60 => Some(Op::Recip),
        // Trig
        0x59 => Some(Op::Sin),
        0x5A => Some(Op::Cos),
        0x5B => Some(Op::Tan),
        // Synthetic primitives — primary purpose of the hex modal
        0xCF => Some(Op::Null),
        0xCE => Some(Op::GetKey),
        // Hidden register access — synthetic byte path mirrors the new Op variants
        0xB0 => Some(Op::StoM),
        0xB1 => Some(Op::StoN),
        0xB2 => Some(Op::StoO),
        0x90 => Some(Op::RclM),
        0x91 => Some(Op::RclN),
        0x92 => Some(Op::RclO),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod flush_eex_tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn make_state_with_entry(s: &str) -> CalcState {
        let mut state = CalcState::new();
        state.entry_buf = s.to_string();
        state
    }

    #[test]
    fn test_flush_scientific_lowercase_e() {
        let mut state = make_state_with_entry("1.5e3");
        flush_entry_buf(&mut state).unwrap();
        assert_eq!(state.stack.x.0, Decimal::from(1500));
    }

    #[test]
    fn test_flush_scientific_uppercase_e() {
        let mut state = make_state_with_entry("2.5E-2");
        flush_entry_buf(&mut state).unwrap();
        assert_eq!(state.stack.x.0, Decimal::from_str("0.025").unwrap());
    }

    #[test]
    fn test_flush_plain_decimal_still_works() {
        let mut state = make_state_with_entry("1500");
        flush_entry_buf(&mut state).unwrap();
        assert_eq!(state.stack.x.0, Decimal::from(1500));
    }

    #[test]
    fn test_flush_invalid_returns_err() {
        let mut state = make_state_with_entry("notanumber");
        assert!(flush_entry_buf(&mut state).is_err());
    }

    #[test]
    fn test_flush_trailing_e_without_exponent_commits_zero_exponent() {
        // HP-41 hardware: trailing 'e' with no exponent digits commits as exponent 00.
        // "1.5e" + ENTER pushes 1.5 to the stack (exponent treated as 00), not a parse error.
        // Per D-09 (Phase 9 CONTEXT): flush_entry_buf normalizes by appending "00" before parsing.
        let mut state = make_state_with_entry("1.5e");
        let result = flush_entry_buf(&mut state);
        assert!(
            result.is_ok(),
            "trailing 'e' with no exponent must commit as exponent 00, not Err"
        );
        assert_eq!(
            state.stack.x.0,
            Decimal::from_str("1.5").unwrap(),
            "1.5e must commit as 1.5 (exponent 00)"
        );
        assert!(
            state.entry_buf.is_empty(),
            "entry_buf must be cleared after successful commit"
        );
    }

    #[test]
    fn test_flush_implicit_one_with_trailing_e_commits_one() {
        // HP-41 hardware: empty-buffer EEX inserts implicit "1" mantissa (D-07 in app.rs).
        // After app.rs sets entry_buf = "1e", flush must commit as 1.0 (1 * 10^0).
        let mut state = make_state_with_entry("1e");
        let result = flush_entry_buf(&mut state);
        assert!(result.is_ok(), "1e must commit successfully");
        assert_eq!(
            state.stack.x.0,
            Decimal::from(1),
            "1e must commit as 1 (1 * 10^0)"
        );
    }

    #[test]
    fn test_flush_trailing_e_minus_parses_as_one() {
        // HP-41 hardware: "1e-" + ENTER commits as 1.0 (exponent -00 = 0).
        // flush_entry_buf normalizes "1e-" → "1e-00" before parsing.
        let mut state = make_state_with_entry("1e-");
        flush_entry_buf(&mut state).unwrap();
        assert!(state.entry_buf.is_empty());
        // "1e-00" == 1.0
        use crate::num::HpNum;
        assert_eq!(state.stack.x, HpNum::from(1i32));
    }

    #[test]
    fn test_flush_entry_buf_negative_exponent() {
        // "1e-2" is a complete negative exponent — parses directly as 0.01.
        let mut state = make_state_with_entry("1e-2");
        flush_entry_buf(&mut state).unwrap();
        assert!(state.entry_buf.is_empty());
        // 1e-2 == 0.01
        assert_eq!(state.stack.x.0, Decimal::from_str("0.01").unwrap());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;

    #[test]
    fn test_user_mode_toggle() {
        let mut state = CalcState::new();
        assert!(!state.user_mode);
        dispatch(&mut state, Op::UserMode).unwrap();
        assert!(state.user_mode, "UserMode must flip to true");
        dispatch(&mut state, Op::UserMode).unwrap();
        assert!(!state.user_mode, "second UserMode must flip back to false");
    }

    #[test]
    fn test_op_serde_round_trip() {
        // Verify Op::Add serializes as JSON string, not a complex structure
        let json = serde_json::to_string(&Op::Add).unwrap();
        let back: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(Op::Add, back);
    }

    #[test]
    fn test_pushnum_serde_round_trip() {
        use crate::num::HpNum;
        let op = Op::PushNum(HpNum::from(42i32));
        let json = serde_json::to_string(&op).unwrap();
        let back: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, back);
    }
}
