// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! INTG (numerical integration) state and implementation.
//!
//! `IntegState` is a placeholder stub. Plan 28-07 fills the fields:
//! `user_label`, `a`, `b`, `n`, `accumulator`, `mode`.
//! ADR-004 (`docs/adr/v3.0-004-intg-threshold.md`) locks the convergence formula.

/// Mid-iteration state for the INTG numerical integration solver.
///
/// Placeholder — Plan 28-07 expands with:
/// - `user_label: String` — XEQ label of the user-supplied integrand function
/// - `a: HpNum` — lower integration bound
/// - `b: HpNum` — upper integration bound
/// - `n: u32` — current subdivision count
/// - `accumulator: HpNum` — running Simpson sum
/// - `mode: DisplayMode` — display mode at INTG invocation (convergence threshold)
///
/// `#[serde(skip)]` on `CalcState::integ_state` — never persisted.
#[derive(Debug, Clone, Default)]
pub struct IntegState;
