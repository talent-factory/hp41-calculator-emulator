//! Phase 24 indirect-addressing helpers and op shims.
//!
//! The two-tier resolver (D-24.1):
//!   - `resolve_indirect_decimal` is the SINGLE source of pointer-validation
//!     truth in `hp41-core`. It validates that `state.regs[reg]` is in range,
//!     reads the value, truncates to integer, and rejects non-integer pointers.
//!   - `resolve_indirect` wraps the inner helper for the common case where
//!     callers want a `u8` register / flag address (FN-IND-01 / FN-IND-02).
//!
//! D-24.4: Plan 24-01 ships ONLY the two helpers + inline tests. The 11
//! `op_*_ind` delegation shims (StoInd / RclInd / StoArithInd / IsgInd /
//! DseInd / SfFlagInd / CfFlagInd / FlagTestInd-resolve / ArclInd / AstoInd
//! / ViewInd) land in plan 24-02 — each will be a 2-line delegation: resolve
//! the indirect address, then call the existing direct-form op. The direct
//! ops carry bounds-checking (D-22.11.1), sidecar-clearing (D-23.4),
//! atomicity (compute-then-write), and per-op lift effects — all of which
//! Phase 24 inherits gratis through delegation.

use crate::error::HpError;
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
pub(crate) fn resolve_indirect_decimal(
    _state: &CalcState,
    _reg: u8,
) -> Result<Decimal, HpError> {
    // RED stub — implementation follows in the GREEN commit.
    Err(HpError::InvalidOp)
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
        // 300 fits in i64 but not u8 — must reject via try_from path.
        state.regs[5] = HpNum::from(300i32);
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_pointer_exceeds_i64_range_rejects() {
        let mut state = CalcState::new();
        // 2^64 is well outside i64::MAX — must reject via to_i64 path
        // (the OTHER ?-arm in resolve_indirect, branch coverage).
        state.regs[5] = HpNum::rounded(
            Decimal::from_str("18446744073709551616").unwrap(),
        );
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
