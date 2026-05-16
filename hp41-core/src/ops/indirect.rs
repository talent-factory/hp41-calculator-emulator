//! Phase 24 indirect-addressing helpers and op shims.
//!
//! The two-tier resolver (D-24.1):
//!   - `resolve_indirect_decimal` is the SINGLE source of pointer-validation
//!     truth in `hp41-core`. It validates that `state.regs[reg]` is in range,
//!     reads the value, truncates to integer, and rejects non-integer pointers.
//!   - `resolve_indirect` wraps the inner helper for the common case where
//!     callers want a `u8` register / flag address (FN-IND-01 / FN-IND-02).
//!
//! Plan 24-02 appends 10 `op_*_ind` delegation shims (StoInd / RclInd /
//! StoArithInd / IsgInd / DseInd / SfFlagInd / CfFlagInd / ArclInd / AstoInd
//! / ViewInd) — each a 2-line body: resolve the indirect address, then call
//! the existing direct-form op. The direct ops carry bounds-checking
//! (D-22.11.1), sidecar-clearing (D-23.4), atomicity (compute-then-write),
//! and per-op lift effects — all of which Phase 24 inherits gratis through
//! delegation (D-24.4). `Op::FlagTestInd` has NO shim by design — its
//! behavior lives inline in `run_loop` (mirrors `Op::FlagTest` precedent).

use crate::error::HpError;
use crate::ops::alpha::{op_arcl, op_asto};
use crate::ops::display_ops::op_view;
use crate::ops::flags::{op_cf, op_sf};
use crate::ops::program::{op_dse, op_isg};
use crate::ops::registers::{op_rcl, op_sto, op_sto_arith};
use crate::ops::StoArithKind;
use crate::state::CalcState;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Validate that `state.regs[reg]` holds an integer pointer; return its Decimal.
///
/// Failure modes (all return `HpError::InvalidOp`):
/// - `reg` is out of range for `state.regs`
/// - the register holds a non-integer value (fractional part non-zero)
///
/// This is the SINGLE source of pointer-validation truth in `hp41-core`.
/// Both `resolve_indirect` (u8 wrapper, used by Phase 24 IND variants) and
/// the refactored Phase-22 `Op::GtoInd` / `Op::XeqInd` arms in
/// `ops/program.rs::run_loop` call this helper directly.
pub(crate) fn resolve_indirect_decimal(state: &CalcState, reg: u8) -> Result<Decimal, HpError> {
    let pointer = state
        .regs
        .get(reg as usize)
        .ok_or(HpError::InvalidOp)?
        .clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer {
        return Err(HpError::InvalidOp);
    }
    Ok(int_part.inner())
}

/// Resolve register `reg`'s integer-part contents into a `u8` register address.
///
/// Two cascading rejection paths, both `HpError::InvalidOp`:
/// 1. `resolve_indirect_decimal` — non-integer pointer (FN-IND-02).
/// 2. `to_i64` / `u8::try_from` — pointer doesn't fit in `u8` (e.g. R05 = 300).
///
/// Per D-24.3, this helper does NOT check the resolved address against
/// `state.regs.len()` (regs ops) or `< 56` (flag ops). Bounds enforcement
/// is the caller's responsibility — the existing direct-form ops
/// (`op_sto`, `op_sf`, etc.) already do it via `.get().ok_or(InvalidOp)?`.
pub fn resolve_indirect(state: &CalcState, reg: u8) -> Result<u8, HpError> {
    let i = resolve_indirect_decimal(state, reg)?;
    let as_i64 = i.to_i64().ok_or(HpError::InvalidOp)?;
    u8::try_from(as_i64).map_err(|_| HpError::InvalidOp)
}

// -- Phase 24 plan 02: op_*_ind delegation shims (FN-IND-01, FN-IND-02) ------
//
// Each shim is 2 lines of body: resolve the indirect address through
// `resolve_indirect`, then call the existing direct-form op with the
// resolved address. The direct op carries:
//   - bounds-checking (D-22.11.1: `.get().ok_or(InvalidOp)?` or `< 56` for flags)
//   - sidecar-clearing (D-23.4: `text_regs.remove` on numeric writes)
//   - atomicity (compute-then-write; checked_* arithmetic for STO-arith)
//   - per-op lift effect (Neutral for STO/SF/CF/ARCL/ASTO/VIEW/ISG/DSE;
//     Enable for RCL)
//
// Phase 24 inherits ALL of these gratis through delegation (D-24.4). The
// resolver is the only new pointer-validation site in the workspace
// (D-24.1 -- single source of truth, also consumed by Phase-22 GtoInd/XeqInd
// after the 24-01 refactor).
//
// NOTE: there is intentionally NO `op_flag_test_ind` shim -- `Op::FlagTestInd`'s
// behavior lives inline in `run_loop` (mirroring the existing `Op::FlagTest`
// precedent in `ops/program.rs::run_loop`). Interactive dispatch is a Neutral
// no-op (no "next program step" to skip at the keyboard).

/// STO IND nn -- store X into register pointed to by R[nn]'s integer part.
/// Delegates to `op_sto`. Inherits Neutral lift, D-23.4 sidecar-clearing on
/// `text_regs`, and D-22.11.1 bounds check via the direct op.
pub(crate) fn op_sto_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_sto(state, addr)
}

/// RCL IND nn -- recall `regs[R[nn].int_part]` into X.
/// Delegates to `op_rcl`. Inherits Enable lift.
pub(crate) fn op_rcl_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_rcl(state, addr)
}

/// STO+/-/x// IND nn -- arithmetic via R[nn]'s integer-part address.
/// Single-variant kind reuse mirrors the `Op::StoArith` shape exactly.
pub(crate) fn op_sto_arith_ind(
    state: &mut CalcState,
    reg: u8,
    kind: StoArithKind,
) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_sto_arith(state, addr, kind)
}

/// ISG IND nn -- increment register pointed to by R[nn]; returns true on
/// counter exit (skip-next signal). Returns `Result<bool, _>` like `op_isg`
/// so `run_loop` can act on the skip.
pub(crate) fn op_isg_ind(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_isg(state, addr)
}

/// DSE IND nn -- decrement and skip on exit; returns `Result<bool, _>` like
/// `op_dse`.
pub(crate) fn op_dse_ind(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_dse(state, addr)
}

/// SF IND nn -- set flag at R[nn]'s integer part. Inherits the `< 56` bounds
/// check from `op_sf`.
pub(crate) fn op_sf_flag_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let flag = resolve_indirect(state, reg)?;
    op_sf(state, flag)
}

/// CF IND nn -- clear flag at R[nn]. Inherits `< 56` bounds via `op_cf`.
pub(crate) fn op_cf_flag_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let flag = resolve_indirect(state, reg)?;
    op_cf(state, flag)
}

/// ARCL IND nn -- append regs[R[nn].int_part]'s formatted value to ALPHA.
/// Delegates to `op_arcl`; inherits text_regs sidecar handling.
pub(crate) fn op_arcl_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_arcl(state, addr)
}

/// ASTO IND nn -- pack first 6 ALPHA chars into `regs[R[nn]]`. Inherits
/// text_regs sidecar setup and atomicity.
pub(crate) fn op_asto_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_asto(state, addr)
}

/// VIEW IND nn -- display the VALUE of `regs[R[nn].int_part]` (NOT the
/// pointer value). R9 mitigation: this delegates to `op_view(state, addr)`,
/// which formats `state.regs[addr]`, so the display shows the resolved
/// register's contents -- `VIEW IND 05` with R05=12 displays the value of R12.
pub(crate) fn op_view_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_view(state, addr)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn resolve_indirect_happy_integer_pointer() {
        let mut state = CalcState::new();
        state.regs[5] = HpNum::from(42i32);
        assert_eq!(resolve_indirect(&state, 5).unwrap(), 42u8);
    }

    #[test]
    fn resolve_indirect_non_integer_rejects() {
        let mut state = CalcState::new();
        state.regs[5] = HpNum::rounded(Decimal::from_str("12.345").unwrap());
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_reg_out_of_range_rejects() {
        let state = CalcState::new();
        // CalcState::new() ships 100 regs; reg=200 is out-of-range.
        assert!(matches!(
            resolve_indirect(&state, 200),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_pointer_exceeds_u8_range_rejects() {
        let mut state = CalcState::new();
        // 300 fits in i64 but not u8 -- must reject via try_from path.
        state.regs[5] = HpNum::from(300i32);
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_pointer_exceeds_i64_range_rejects() {
        let mut state = CalcState::new();
        // 2^64 is well outside i64::MAX -- must reject via to_i64 path
        // (the OTHER ?-arm in resolve_indirect, branch coverage).
        state.regs[5] = HpNum::rounded(Decimal::from_str("18446744073709551616").unwrap());
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_negative_integer_pointer_rejects_via_u8() {
        let mut state = CalcState::new();
        // -3 is an integer (passes the inner helper) but doesn't fit in u8.
        state.regs[5] = HpNum::from(-3i32);
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_decimal_preserves_sign_for_gto_ind_callers() {
        // Inner helper returns the Decimal as-is; sign preservation is
        // observable to the GtoInd / XeqInd refactor sites.
        let mut state = CalcState::new();
        state.regs[5] = HpNum::from(-3i32);
        let d = resolve_indirect_decimal(&state, 5).unwrap();
        assert_eq!(d.to_string(), "-3");
    }
}
