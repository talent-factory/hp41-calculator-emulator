//! Phase 2 storage register operations: STO, RCL, STO+/-/×/÷, CLREG.
//!
//! Register addresses are 0–99 (0-indexed). Addresses ≥ 100 return InvalidOp.
//! STO and STO-arith: Neutral lift (do not modify lift_enabled).
//! RCL: Enable lift (like PushNum — places a value on the stack).

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::{StackReg, StoArithKind};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

/// STO n: copy X register into storage register n. Stack unchanged.
/// LiftEffect: Neutral. LASTX: not saved (STO is not an arithmetic operation).
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    state.regs[reg as usize] = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCL n: recall register n into X (with stack lift if lift_enabled).
/// LiftEffect: Enable. LASTX: not saved.
pub fn op_rcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    let val = state.regs[reg as usize].clone();
    // Force lift_enabled = true so enter_number performs the stack lift.
    // This matches HP-41 hardware: RCL always lifts regardless of prior state.
    state.stack.lift_enabled = true;
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// STO+/−/×/÷ n: apply arithmetic to register n using X.
/// R[n] ← R[n] OP X. Stack and X are unchanged.
/// LiftEffect: Neutral. LASTX: not saved.
///
/// IMPORTANT: compute new value FIRST, write ONLY on success (atomicity guarantee).
pub fn op_sto_arith(state: &mut CalcState, reg: u8, kind: StoArithKind) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    // Compute first — do NOT write to state.regs[reg] until we know the op succeeds.
    let new_val = match kind {
        StoArithKind::Add => state.regs[reg as usize].checked_add(&state.stack.x)?,
        StoArithKind::Sub => state.regs[reg as usize].checked_sub(&state.stack.x)?,
        StoArithKind::Mul => state.regs[reg as usize].checked_mul(&state.stack.x)?,
        StoArithKind::Div => state.regs[reg as usize].checked_div(&state.stack.x)?,
    };
    // Write only after successful computation (Pitfall 6 guard)
    state.regs[reg as usize] = new_val;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// STO+/−/×/÷ stack-reg: apply arithmetic to a stack register using X.
/// stack_reg ← stack_reg OP X. Stack and X are unchanged (only target reg written).
/// LiftEffect: Neutral. LASTX: not saved.
///
/// IMPORTANT: compute new value FIRST, write ONLY on success (atomicity guarantee).
pub fn op_sto_arith_stack(
    state: &mut CalcState,
    stack_reg: StackReg,
    kind: StoArithKind,
) -> Result<(), HpError> {
    // Snapshot current value of target register (before any write).
    let current = match stack_reg {
        StackReg::Y => state.stack.y.clone(),
        StackReg::Z => state.stack.z.clone(),
        StackReg::T => state.stack.t.clone(),
        StackReg::LastX => state.stack.lastx.clone(),
    };
    // Compute first — do NOT write until we know the op succeeds.
    let new_val = match kind {
        StoArithKind::Add => current.checked_add(&state.stack.x)?,
        StoArithKind::Sub => current.checked_sub(&state.stack.x)?,
        StoArithKind::Mul => current.checked_mul(&state.stack.x)?,
        StoArithKind::Div => current.checked_div(&state.stack.x)?,
    };
    // Write only after successful computation.
    match stack_reg {
        StackReg::Y => state.stack.y = new_val,
        StackReg::Z => state.stack.z = new_val,
        StackReg::T => state.stack.t = new_val,
        StackReg::LastX => state.stack.lastx = new_val,
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLREG: clear all storage registers to zero.
/// LiftEffect: Neutral.
pub fn op_clreg(state: &mut CalcState) -> Result<(), HpError> {
    state.regs = vec![crate::num::HpNum::zero(); 100];
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Phase 12: Synthetic Programming ──────────────────────────────────────────

/// GETKEY — push the last HP-41 row-column key code to X. LiftEffect::Enable.
/// Reads `state.last_key_code` (u8) — default 0 when no key has been pressed yet.
pub fn op_getkey(state: &mut CalcState) -> Result<(), HpError> {
    let code = HpNum::from(state.last_key_code as i32);
    state.stack.lift_enabled = true; // GETKEY always lifts (produces a new value)
    enter_number(state, code);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// STO M — store X into hidden register M. LiftEffect::Neutral.
pub fn op_sto_m(state: &mut CalcState) -> Result<(), HpError> {
    state.reg_m = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// STO N — store X into hidden register N. LiftEffect::Neutral.
pub fn op_sto_n(state: &mut CalcState) -> Result<(), HpError> {
    state.reg_n = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// STO O — store X into hidden register O. LiftEffect::Neutral.
pub fn op_sto_o(state: &mut CalcState) -> Result<(), HpError> {
    state.reg_o = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCL M — recall hidden register M into X. LiftEffect::Enable.
/// Forces lift_enabled = true before enter_number, matching op_rcl pattern.
pub fn op_rcl_m(state: &mut CalcState) -> Result<(), HpError> {
    let val = state.reg_m.clone();
    state.stack.lift_enabled = true;
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// RCL N — recall hidden register N into X. LiftEffect::Enable.
pub fn op_rcl_n(state: &mut CalcState) -> Result<(), HpError> {
    let val = state.reg_n.clone();
    state.stack.lift_enabled = true;
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// RCL O — recall hidden register O into X. LiftEffect::Enable.
pub fn op_rcl_o(state: &mut CalcState) -> Result<(), HpError> {
    let val = state.reg_o.clone();
    state.stack.lift_enabled = true;
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod stack_arith_tests {
    use super::*;
    use crate::num::HpNum;
    use crate::ops::StackReg;
    use crate::state::CalcState;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).expect("test literal")
    }

    fn make_state(x: Decimal, y: Decimal) -> CalcState {
        let mut s = CalcState::default();
        s.stack.x = HpNum::from(x);
        s.stack.y = HpNum::from(y);
        s
    }

    #[test]
    fn sto_arith_stack_add_y() {
        let mut s = make_state(d("3"), d("10"));
        op_sto_arith_stack(&mut s, StackReg::Y, StoArithKind::Add).unwrap();
        assert_eq!(s.stack.y, HpNum::from(d("13")));
        assert_eq!(s.stack.x, HpNum::from(d("3"))); // X unchanged
    }

    #[test]
    fn sto_arith_stack_sub_lastx() {
        let mut s = CalcState::default();
        s.stack.x = HpNum::from(d("4"));
        s.stack.lastx = HpNum::from(d("10"));
        op_sto_arith_stack(&mut s, StackReg::LastX, StoArithKind::Sub).unwrap();
        assert_eq!(s.stack.lastx, HpNum::from(d("6")));
    }

    #[test]
    fn sto_arith_stack_div_by_zero_returns_err() {
        let mut s = make_state(d("0"), d("5"));
        let result = op_sto_arith_stack(&mut s, StackReg::Y, StoArithKind::Div);
        assert!(result.is_err());
        // Y must be unchanged on error (atomicity)
        assert_eq!(s.stack.y, HpNum::from(d("5")));
    }

    /// PR #5 review (pr-test-analyzer) — only Y (Add) and LastX (Sub) were
    /// unit-tested; Z and T were exercised only through CLI integration paths.
    /// A future refactor that lost the Z or T arm in op_sto_arith_stack would
    /// be silently green. Cover the remaining stack-reg variants here.
    #[test]
    fn sto_arith_stack_mul_z() {
        let mut s = CalcState::default();
        s.stack.x = HpNum::from(d("3"));
        s.stack.z = HpNum::from(d("4"));
        op_sto_arith_stack(&mut s, StackReg::Z, StoArithKind::Mul).unwrap();
        assert_eq!(
            s.stack.z,
            HpNum::from(d("12")),
            "STO×Z must write 3×4=12 into Z"
        );
        assert_eq!(s.stack.x, HpNum::from(d("3")), "X must be unchanged");
    }

    #[test]
    fn sto_arith_stack_div_t() {
        let mut s = CalcState::default();
        s.stack.x = HpNum::from(d("2"));
        s.stack.t = HpNum::from(d("10"));
        op_sto_arith_stack(&mut s, StackReg::T, StoArithKind::Div).unwrap();
        assert_eq!(
            s.stack.t,
            HpNum::from(d("5")),
            "STO÷T must write 10÷2=5 into T"
        );
        assert_eq!(s.stack.x, HpNum::from(d("2")), "X must be unchanged");
    }
}
