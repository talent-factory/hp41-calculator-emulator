//! Integration test stubs for SCI-01: statistics operations.
//! Wave 0: stubs compile but FAIL — implementations created in Plan 02.
//! Full test bodies added in Plan 03.

use hp41_core::{CalcState, HpError};
use hp41_core::ops::{dispatch, Op};

// ── Wave 0 stubs ──────────────────────────────────────────────────────────────
// These tests will FAIL with "no variant or associated item" until Plan 02
// creates stats.rs. They are intentionally failing stubs.

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_sigma_plus_stub() {
    let mut s = CalcState::new();
    // SCI-01: Σ+ accumulates X and Y into R01–R06, pushes count n into X
    let result = dispatch(&mut s, Op::SigmaPlus);
    // Will pass once op_sigma_plus is implemented in Plan 02
    assert!(result.is_ok(), "SigmaPlus must not error on valid input");
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_sigma_minus_stub() {
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::SigmaMinus);
    assert!(result.is_ok(), "SigmaMinus must not error on valid input");
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_mean_empty_returns_invalid_op_stub() {
    let mut s = CalcState::new();
    // MEAN with n=0 must return HpError::InvalidOp
    assert_eq!(dispatch(&mut s, Op::Mean), Err(HpError::InvalidOp));
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_sdev_stub() {
    let mut s = CalcState::new();
    // SDEV with n<2 must return HpError::InvalidOp
    assert_eq!(dispatch(&mut s, Op::Sdev), Err(HpError::InvalidOp));
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_lr_stub() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::LR), Err(HpError::InvalidOp));
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_yhat_stub() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::Yhat), Err(HpError::InvalidOp));
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_corr_stub() {
    let mut s = CalcState::new();
    assert_eq!(dispatch(&mut s, Op::Corr), Err(HpError::InvalidOp));
}

#[test]
#[ignore = "stub: implementation in Plan 02"]
fn test_cl_sigma_stat_stub() {
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::ClSigmaStat);
    assert!(result.is_ok(), "ClSigmaStat must not error");
}
