// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! DIFEQ (differential equation) state and implementation.
//!
//! `DifeqState` is a placeholder stub. Plan 28-09 fills the fields:
//! `user_label`, `order`, `h`, `x`, `y`, `yprime`, `step_count`.

/// Mid-iteration state for the DIFEQ differential-equation solver (RK4).
///
/// Placeholder — Plan 28-09 expands with:
/// - `user_label: String` — XEQ label of the user-supplied ODE right-hand-side
/// - `order: u8` — ODE order (1 or 2)
/// - `h: HpNum` — step size
/// - `x: HpNum` — current x
/// - `y: HpNum` — current y(x)
/// - `yprime: HpNum` — current y'(x) (2nd-order only)
/// - `step_count: u32` — steps taken (bounded output guard)
///
/// `#[serde(skip)]` on `CalcState::difeq_state` — never persisted.
/// RESEARCH Open Q2 recommendation (a): early commitment — field ships now.
#[derive(Debug, Clone, Default)]
pub struct DifeqState;
