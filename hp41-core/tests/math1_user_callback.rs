// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 regression scaffold: user-callback re-entrancy strict-reject tests (C-28.2).
//!
//! **Module invariant:** 5 regression cases for user-callback re-entrancy.
//! - Plan 28-07 fills the `nested_integ_*` and `user_fn_stops_aborts_integ` branches
//! - Plan 28-08 fills the `nested_solve_*` branches
//! - Plan 28-09 fills the `nested_difeq_*` branch
//!
//! C-28.2 / ADR-002: strict-reject nested INTG/SOLVE/DIFEQ at op entry.
//! At op entry, check `state.integ_state.is_some() || state.solve_state.is_some()`
//! → `HpError::InvalidOp` if true. This matches Math Pac I OM 1979 hardware behavior.

#![allow(clippy::unwrap_used)]

/// Nested INTG inside an INTG user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard missing in op_integ entry.
/// Filled by Plan 28-07.
#[test]
#[ignore = "filled by Plan 28-07"]
fn nested_integ_inside_integ_rejected() {
    unimplemented!("filled by Plan 28-07");
}

/// Nested SOLVE inside an INTG user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard checking only integ_state, not solve_state.
/// Filled by Plan 28-07.
#[test]
#[ignore = "filled by Plan 28-07"]
fn nested_solve_inside_integ_rejected() {
    unimplemented!("filled by Plan 28-07");
}

/// Nested INTG inside a SOLVE user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard missing in op_solve entry.
/// Filled by Plan 28-08.
#[test]
#[ignore = "filled by Plan 28-08"]
fn nested_integ_inside_solve_rejected() {
    unimplemented!("filled by Plan 28-08");
}

/// Nested DIFEQ inside an INTG user function must return HpError::InvalidOp.
/// Catches: re-entrancy guard checking only {integ,solve}_state but not difeq_state.
/// Filled by Plan 28-09.
#[test]
#[ignore = "filled by Plan 28-09"]
fn nested_difeq_inside_integ_rejected() {
    unimplemented!("filled by Plan 28-09");
}

/// User function that sets cancel_requested = true stops INTG and returns HpError::Canceled.
/// Catches: cancellation check missing inside the Simpson inner loop (D-28.7/D-28.8).
/// Filled by Plan 28-07.
#[test]
#[ignore = "filled by Plan 28-07"]
fn user_fn_stops_aborts_integ() {
    unimplemented!("filled by Plan 28-07");
}
