use crate::error::HpError;
use crate::num::HpNum;
use crate::state::CalcState;

pub mod arithmetic;
pub mod stack_ops;

use arithmetic::{op_add, op_sub, op_mul, op_div};
use stack_ops::{op_enter, op_clx, op_chs, op_rdn, op_xy_swap, op_lastx};

/// HP-41 calculator operations implemented in Phase 1.
///
/// Every variant maps to a function with signature:
///   fn(state: &mut CalcState) -> Result<(), HpError>
///
/// Phase 2 will add: Arithmetic extended ops (1/x, sqrt, pow, ln, log, exp)
/// Phase 3 will add: Programming ops (LBL, GTO, XEQ, RTN, ISG, DSE, conditional tests)
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    // Stack operations
    Enter,
    Clx,
    Chs,
    Rdn,
    XySwap,
    Lastx,
    /// Push a numeric literal onto the stack (e.g., from keyboard digit entry)
    PushNum(HpNum),
}

/// Dispatch an operation to its implementation function.
///
/// This is the single entry point for all calculator operations in hp41-core.
/// Callers (hp41-cli, tests) call dispatch(state, op) and handle the Result.
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    match op {
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
    }
}
