use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::{CalcState, DisplayMode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod alpha;
pub mod arithmetic;
pub mod hms;
pub mod math;
pub mod program;
pub mod registers;
pub mod stack_ops;
pub mod stats;

use alpha::{op_alpha_append, op_alpha_backspace, op_alpha_clear, op_alpha_toggle};
use arithmetic::{op_add, op_div, op_mul, op_sub};
use math::{
    op_acos, op_asin, op_atan, op_cos, op_exp, op_int, op_ln, op_log, op_recip, op_set_deg,
    op_set_grad, op_set_rad, op_sin, op_sq, op_sqrt, op_tan, op_tenpow, op_ypow,
};
use registers::{op_clreg, op_rcl, op_sto, op_sto_arith};
use stack_ops::{op_chs, op_clx, op_enter, op_lastx, op_rdn, op_xy_swap};

/// STO arithmetic operation kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoArithKind {
    Add,
    Sub,
    Mul,
    Div,
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
/// Phase 2: Recip, Sqrt, Sq, YPow, Ln, Log, Exp, TenPow, Sin, Cos, Tan,
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
    XySwap,
    Lastx,
    /// Push a numeric literal onto the stack (e.g., from keyboard digit entry).
    PushNum(HpNum),
    // ── Unary math (Phase 2) ─────────────────────────────────────────
    /// INT — truncate X toward zero (integer part). LiftEffect: Enable.
    Int,
    /// 1/x — reciprocal. LiftEffect: Enable.
    Recip,
    /// √x — square root. LiftEffect: Enable.
    Sqrt,
    /// x² — square. LiftEffect: Enable.
    Sq,
    /// Y^X — Y raised to power X (binary). LiftEffect: Enable.
    YPow,
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
    let s = state.entry_buf.clone();
    state.entry_buf.clear();
    let d = Decimal::from_str(&s)
        .or_else(|_| Decimal::from_scientific(&s))
        .map_err(|_| HpError::InvalidOp)?;
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
        Op::XySwap => op_xy_swap(state),
        Op::Lastx => op_lastx(state),
        Op::PushNum(v) => {
            crate::stack::enter_number(state, v);
            Ok(())
        }
        // ── Phase 2 math/trig/angle ops (Plan 04) ───────────────────────
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
