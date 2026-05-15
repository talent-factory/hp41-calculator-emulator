#![allow(clippy::unwrap_used)]

//! Pitfall-5 regression sentinel for the Phase 22 SIZE-shrink stats guards.
//!
//! The stats ops (`Σ-`, `MEAN`, `SDEV`, `LR`, `YHAT`, `CORR`, `CLΣ`) each
//! carry a fail-closed SIZE-shrink guard: if `state.regs.len() < 7` (the Σ
//! register block start, R01..R06), the op returns `HpError::InvalidOp`
//! instead of indexing OOB. This file exercises EVERY guard. Without these
//! tests, a future SIZE-shrink-related regression (Phase 22 Pitfall 5
//! class) would only surface in a real user-driven SIZE-down-shrink
//! workflow.
//!
//! Risk-weighted Priority 2 per RESEARCH §Risk-Weighted Uncovered-Line
//! Inventory: targets `ops/stats.rs` lines 62/93/120/158/206/245/279
//! (the SIZE-shrink guards) plus the `op_lr` / `op_yhat` denom-zero and
//! n=0 guards (lines 173/210/221/249).

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Build a fresh CalcState with `state.regs.truncate(3)` — SIZE 003, below
/// the Σ-register block start (R01..R06 require regs.len() >= 7).
fn shrunken_state() -> CalcState {
    let mut s = CalcState::new();
    s.regs.truncate(3);
    s
}

/// Snapshot of mutable fields that must survive a fail-closed guard call.
struct Snapshot {
    regs_len: usize,
    x: HpNum,
    lift_enabled: bool,
}

impl Snapshot {
    fn of(state: &CalcState) -> Self {
        Self {
            regs_len: state.regs.len(),
            x: state.stack.x.clone(),
            lift_enabled: state.stack.lift_enabled,
        }
    }
    fn assert_unchanged(&self, state: &CalcState) {
        assert_eq!(state.regs.len(), self.regs_len, "regs.len must not change");
        assert_eq!(state.stack.x, self.x, "X must not change");
        assert_eq!(
            state.stack.lift_enabled, self.lift_enabled,
            "lift_enabled must not change"
        );
    }
}

// ── SIZE-shrink guards (one test per Σ op) ──────────────────────────────────

#[test]
fn op_sigma_minus_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::SigmaMinus — guard at stats.rs
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::SigmaMinus);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

#[test]
fn op_sigma_plus_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::SigmaPlus — guard at stats.rs.
    // Σ+ shares the same guard; including it gives the matched pair for the
    // accumulator (Σ+ adds, Σ- removes) so the failure-class is fully bracketed.
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::SigmaPlus);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

#[test]
fn op_mean_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::Mean — guard at stats.rs
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::Mean);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

#[test]
fn op_sdev_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::Sdev — guard at stats.rs
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::Sdev);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

#[test]
fn op_lr_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::LR — guard at stats.rs
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::LR);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

#[test]
fn op_yhat_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::Yhat — guard at stats.rs
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::Yhat);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

#[test]
fn op_corr_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::Corr — guard at stats.rs
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::Corr);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

#[test]
fn op_cl_sigma_stat_shrunken_returns_invalid_op() {
    // Catches: SIZE-shrink-induced panic on Op::ClSigmaStat — guard at stats.rs
    let mut s = shrunken_state();
    let snap = Snapshot::of(&s);
    let r = dispatch(&mut s, Op::ClSigmaStat);
    assert!(matches!(r, Err(HpError::InvalidOp)));
    snap.assert_unchanged(&s);
}

// ── Denom-zero / n=0 guards (deeper Pitfall-5 sentinels) ────────────────────

/// Add a single (y, x) data point via Σ+. Used to set up small-sample states.
fn add_point(state: &mut CalcState, y_val: &str, x_val: &str) {
    state.stack.lift_enabled = true;
    dispatch(
        state,
        Op::PushNum(HpNum::from(Decimal::from_str(y_val).unwrap())),
    )
    .unwrap();
    state.stack.lift_enabled = true;
    dispatch(
        state,
        Op::PushNum(HpNum::from(Decimal::from_str(x_val).unwrap())),
    )
    .unwrap();
    dispatch(state, Op::SigmaPlus).unwrap();
}

#[test]
fn op_lr_denom_zero_two_points_with_identical_x_returns_invalid_op() {
    // Catches: divide-by-zero in linear-regression slope when all x_i are
    // equal — denom = n·Σx² − (Σx)² = 0 (stats.rs).
    let mut s = CalcState::new();
    add_point(&mut s, "5", "3"); // y=5, x=3
    add_point(&mut s, "7", "3"); // y=7, x=3 (same x!)
    let r = dispatch(&mut s, Op::LR);
    assert!(matches!(r, Err(HpError::InvalidOp)));
}

#[test]
fn op_yhat_n_zero_empty_sigma_returns_invalid_op() {
    // Catches: YHAT on empty Σ block — n == 0 guard at stats.rs.
    let mut s = CalcState::new();
    // Fresh state: regs[3] = n = 0 by default.
    let r = dispatch(&mut s, Op::Yhat);
    assert!(matches!(r, Err(HpError::InvalidOp)));
}

#[test]
fn op_yhat_denom_zero_returns_invalid_op() {
    // Catches: divide-by-zero in YHAT linear-regression-slope step when all
    // x_i are equal — denom guard at stats.rs.
    let mut s = CalcState::new();
    add_point(&mut s, "1", "5");
    add_point(&mut s, "2", "5"); // identical x → denom == 0
    let r = dispatch(&mut s, Op::Yhat);
    assert!(matches!(r, Err(HpError::InvalidOp)));
}

#[test]
fn op_corr_n_zero_empty_sigma_returns_invalid_op() {
    // Catches: CORR on empty Σ block — n == 0 guard at stats.rs.
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::Corr);
    assert!(matches!(r, Err(HpError::InvalidOp)));
}

#[test]
fn op_mean_n_zero_empty_sigma_returns_invalid_op() {
    // Catches: MEAN on empty Σ block — n == 0 guard at stats.rs.
    // Symmetric to corr/yhat n=0 cases; closes the n=0 sentinel cluster.
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::Mean);
    assert!(matches!(r, Err(HpError::InvalidOp)));
}

#[test]
fn op_sdev_n_lt_2_returns_invalid_op() {
    // Catches: SDEV needs >= 2 data points — n_minus_1 == 0 guard at
    // stats.rs. Single data point ⇒ n=1 ⇒ n_minus_1=0.
    let mut s = CalcState::new();
    add_point(&mut s, "5", "3");
    let r = dispatch(&mut s, Op::Sdev);
    assert!(matches!(r, Err(HpError::InvalidOp)));
}
