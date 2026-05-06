use crate::error::HpError;
use crate::num::HpNum;
use crate::state::CalcState;

pub mod arithmetic;
pub mod stack_ops;
pub mod math;
// Phase 2 modules — uncommented when their files are created:
pub mod registers;
// pub mod alpha;

use arithmetic::{op_add, op_sub, op_mul, op_div};
use stack_ops::{op_enter, op_clx, op_chs, op_rdn, op_xy_swap, op_lastx};
use math::{
    op_recip, op_sqrt, op_sq, op_ypow, op_ln, op_log, op_exp, op_tenpow,
    op_sin, op_cos, op_tan, op_asin, op_acos, op_atan,
    op_set_deg, op_set_rad, op_set_grad,
};
use registers::{op_sto, op_rcl, op_sto_arith, op_clreg};

/// STO arithmetic operation kind.
#[derive(Debug, Clone, PartialEq)]
pub enum StoArithKind {
    Add,
    Sub,
    Mul,
    Div,
}

/// HP-41 calculator operations.
///
/// Phase 1: Add, Sub, Mul, Div, Enter, Clx, Chs, Rdn, XySwap, Lastx, PushNum
/// Phase 2: Recip, Sqrt, Sq, YPow, Ln, Log, Exp, TenPow, Sin, Cos, Tan,
///          Asin, Acos, Atan, SetDeg, SetRad, SetGrad,
///          FmtFix, FmtSci, FmtEng, StoReg, RclReg, StoArith, Clreg,
///          AlphaToggle, AlphaAppend, AlphaClear
#[derive(Debug, Clone, PartialEq)]
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
    StoArith { reg: u8, kind: StoArithKind },
    /// CLREG — clear all storage registers to zero. LiftEffect: Neutral.
    Clreg,
    // ── ALPHA mode (Phase 2) ─────────────────────────────────────────
    /// ALPHA — toggle alpha_mode flag. LiftEffect: Neutral.
    AlphaToggle,
    /// Append char to alpha_reg (max 24). LiftEffect: Neutral.
    AlphaAppend(char),
    /// CLRALPHA — clear alpha_reg. LiftEffect: Neutral.
    AlphaClear,
}

/// Dispatch an operation to its implementation function.
///
/// This is the single entry point for all calculator operations in hp41-core.
/// Callers (hp41-cli, tests) call dispatch(state, op) and handle the Result.
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    match op {
        // ── Phase 1 ops ──────────────────────────────────────────────────
        Op::Add        => op_add(state),
        Op::Sub        => op_sub(state),
        Op::Mul        => op_mul(state),
        Op::Div        => op_div(state),
        Op::Enter      => op_enter(state),
        Op::Clx        => op_clx(state),
        Op::Chs        => op_chs(state),
        Op::Rdn        => op_rdn(state),
        Op::XySwap     => op_xy_swap(state),
        Op::Lastx      => op_lastx(state),
        Op::PushNum(v) => {
            crate::stack::enter_number(state, v);
            Ok(())
        }
        // ── Phase 2 math/trig/angle ops (Plan 04) ───────────────────────
        Op::Recip      => op_recip(state),
        Op::Sqrt       => op_sqrt(state),
        Op::Sq         => op_sq(state),
        Op::YPow       => op_ypow(state),
        Op::Ln         => op_ln(state),
        Op::Log        => op_log(state),
        Op::Exp        => op_exp(state),
        Op::TenPow     => op_tenpow(state),
        Op::Sin        => op_sin(state),
        Op::Cos        => op_cos(state),
        Op::Tan        => op_tan(state),
        Op::Asin       => op_asin(state),
        Op::Acos       => op_acos(state),
        Op::Atan       => op_atan(state),
        Op::SetDeg     => op_set_deg(state),
        Op::SetRad     => op_set_rad(state),
        Op::SetGrad    => op_set_grad(state),
        Op::FmtFix(_)      => Err(HpError::InvalidOp),
        Op::FmtSci(_)      => Err(HpError::InvalidOp),
        Op::FmtEng(_)      => Err(HpError::InvalidOp),
        Op::StoReg(r)              => op_sto(state, r),
        Op::RclReg(r)              => op_rcl(state, r),
        Op::StoArith { reg, kind } => op_sto_arith(state, reg, kind),
        Op::Clreg                  => op_clreg(state),
        Op::AlphaToggle    => Err(HpError::InvalidOp),
        Op::AlphaAppend(_) => Err(HpError::InvalidOp),
        Op::AlphaClear     => Err(HpError::InvalidOp),
    }
}
