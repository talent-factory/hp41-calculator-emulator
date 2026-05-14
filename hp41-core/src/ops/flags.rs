//! Phase 21 flag operations: SF (set), CF (clear), and the bit-twiddling free helpers.
//!
//! Both ops have LiftEffect::Neutral. Flag indices are 0..=55; indices > 55 return InvalidOp.

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

/// Read flag `n` (0..=55) from the packed `flags` word.
/// Out-of-range indices (n > 55) return false defensively.
#[inline]
pub fn flag_get(flags: u64, n: u8) -> bool {
    if n > 55 {
        return false;
    }
    (flags & (1u64 << n)) != 0
}

/// Set flag `n` (0..=55) in the packed `flags` word; out-of-range indices are no-ops.
#[inline]
pub fn flag_set(flags: u64, n: u8) -> u64 {
    if n > 55 {
        return flags;
    }
    flags | (1u64 << n)
}

/// Clear flag `n` (0..=55) in the packed `flags` word; out-of-range indices are no-ops.
#[inline]
pub fn flag_clear(flags: u64, n: u8) -> u64 {
    if n > 55 {
        return flags;
    }
    flags & !(1u64 << n)
}

/// SF n — set flag n (0..=55). LiftEffect: Neutral.
/// Returns `HpError::InvalidOp` for n > 55 without mutating state (range guard runs first).
pub fn op_sf(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 55 {
        return Err(HpError::InvalidOp);
    }
    state.flags = flag_set(state.flags, n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CF n — clear flag n (0..=55). LiftEffect: Neutral.
/// Returns `HpError::InvalidOp` for n > 55 without mutating state.
pub fn op_cf(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 55 {
        return Err(HpError::InvalidOp);
    }
    state.flags = flag_clear(state.flags, n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn flag_set_get_boundaries() {
        for n in [0u8, 1, 31, 55] {
            let f = flag_set(0, n);
            assert_eq!(f, 1u64 << n, "flag_set({n}) must return 1<<{n}");
            assert!(flag_get(f, n), "flag_get must observe the set bit at {n}");
        }
    }

    #[test]
    fn flag_clear_boundaries() {
        for n in [0u8, 1, 31, 55] {
            let f = flag_clear(u64::MAX, n);
            assert_eq!(f, u64::MAX & !(1u64 << n), "flag_clear({n}) wrong");
            assert!(!flag_get(f, n), "flag_get must observe the cleared bit");
        }
    }

    #[test]
    fn flag_helpers_out_of_range_are_no_ops() {
        assert!(!flag_get(u64::MAX, 56));
        assert_eq!(flag_set(0, 56), 0);
        assert_eq!(flag_set(42, 100), 42);
        assert_eq!(flag_clear(u64::MAX, 56), u64::MAX);
        assert_eq!(flag_clear(42, 200), 42);
    }
}
