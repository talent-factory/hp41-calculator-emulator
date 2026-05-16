// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `math1` — XROM framework and Math Pac I (HP 00041-90034, 1979) operations.
//!
//! Module structure:
//! - `xrom`: XromModule registry, `xrom_resolve()` entry point, `MATH_1` const
//! - `modal`: ModalProgram state-machine enum for prompt-driven workflows
//! - `integ`: IntegState placeholder (Plan 28-07 fills)
//! - `solve`: SolveState placeholder (Plan 28-08 fills)
//! - `difeq`: DifeqState placeholder (Plan 28-09 fills)

pub mod difeq;
pub mod hyperbolics;
pub mod integ;
pub mod modal;
pub mod solve;
pub mod xrom;
