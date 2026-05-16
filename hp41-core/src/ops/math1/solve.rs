// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! SOLVE (root-finding) state and implementation.
//!
//! `SolveState` is a placeholder stub. Plan 28-08 fills the fields:
//! `user_label`, `x0`, `x1`, `fx0`, `fx1`, `iteration`.

/// Mid-iteration state for the SOLVE root-finding solver (secant method).
///
/// Placeholder — Plan 28-08 expands with:
/// - `user_label: String` — XEQ label of the user-supplied function
/// - `x0: HpNum` — first current secant point
/// - `x1: HpNum` — second current secant point
/// - `fx0: HpNum` — f(x0)
/// - `fx1: HpNum` — f(x1)
/// - `iteration: u32` — iteration counter (bounded convergence guard)
///
/// `#[serde(skip)]` on `CalcState::solve_state` — never persisted.
#[derive(Debug, Clone, Default)]
pub struct SolveState;
