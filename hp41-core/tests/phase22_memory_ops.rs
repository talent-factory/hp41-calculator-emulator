//! Integration tests for Phase 22 Plan 03 (Memory & stack ops: SIZE / CLA /
//! CLST / PACK + Wave-0 bounds-audit sentinels).
//!
//! Covers FN-MEM-01 / FN-MEM-02 / FN-MEM-03 / FN-MEM-04 plus the critical
//! sentinel tests for Pitfall 4 (op_sto post-shrink → InvalidOp not panic),
//! Pitfall 5 (Σ+ post-shrink → InvalidOp not panic), OQ-2 (SIZE 0 clamps
//! to 1), and D-22.14 (CLST preserves LASTX + lift_enabled).

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpError, HpNum};

// ─── SIZE: FN-MEM-01 + OQ-2 + Pitfall 4 + Pitfall 5 ─────────────────────────

#[test]
fn test_size_basic() {
    // Op::Size(50) → state.regs.len() == 50, all zero.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Size(50)).unwrap();
    assert_eq!(s.regs.len(), 50);
    for r in &s.regs {
        assert_eq!(*r, HpNum::zero());
    }
}

#[test]
fn test_size_zero_clamps_to_one() {
    // OQ-2 sentinel: Op::Size(0) → Ok AND state.regs.len() == 1.
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::Size(0));
    assert!(
        result.is_ok(),
        "SIZE 0 must NOT be InvalidOp (OQ-2 Option A); got {result:?}"
    );
    assert_eq!(s.regs.len(), 1, "SIZE 0 must clamp to 1 register, not 0");
}

#[test]
fn test_size_over_319_rejects() {
    // SIZE 320 must return InvalidOp without mutating regs.
    let mut s = CalcState::new();
    let before_len = s.regs.len();
    let result = dispatch(&mut s, Op::Size(320));
    assert!(matches!(result, Err(HpError::InvalidOp)));
    assert_eq!(
        s.regs.len(),
        before_len,
        "regs must be unchanged on InvalidOp"
    );
}

#[test]
fn test_size_319_accepted() {
    // SIZE 319 is the documented hardware maximum and must succeed.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Size(319)).unwrap();
    assert_eq!(s.regs.len(), 319);
}

#[test]
fn test_size_shrink_truncates_tail() {
    // Fill regs with non-zero values, shrink, confirm tail discarded but
    // surviving cells preserved.
    let mut s = CalcState::new();
    for r in s.regs.iter_mut() {
        *r = HpNum::from(7i32);
    }
    dispatch(&mut s, Op::Size(10)).unwrap();
    assert_eq!(s.regs.len(), 10);
    for r in &s.regs {
        assert_eq!(
            *r,
            HpNum::from(7i32),
            "shrink must preserve surviving values"
        );
    }
}

#[test]
fn test_size_grow_zero_fills() {
    // Shrink to 5 with values, grow to 20, confirm new cells zeroed and
    // surviving cells preserved.
    let mut s = CalcState::new();
    for r in s.regs.iter_mut() {
        *r = HpNum::from(3i32);
    }
    dispatch(&mut s, Op::Size(5)).unwrap();
    dispatch(&mut s, Op::Size(20)).unwrap();
    assert_eq!(s.regs.len(), 20);
    // First 5 retained their value
    for r in &s.regs[..5] {
        assert_eq!(*r, HpNum::from(3i32), "preserved cells");
    }
    // Cells 5..20 are zero-filled
    for r in &s.regs[5..20] {
        assert_eq!(*r, HpNum::zero(), "grown cells must be zero");
    }
}

#[test]
fn test_sto_out_of_range_after_shrink_returns_invalid_op_not_panic() {
    // Pitfall 4 sentinel: Op::Size(5) followed by Op::StoReg(50) returns
    // Err(InvalidOp) and MUST NOT PANIC. The standard test harness reports
    // panics as failures, so a successful Err match is sufficient evidence
    // that the audit (22-03-01) is in place.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Size(5)).unwrap();
    let result = dispatch(&mut s, Op::StoReg(50));
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "STO 50 on SIZE-5 state must be InvalidOp, got {result:?}"
    );
}

#[test]
fn test_rcl_out_of_range_after_shrink_returns_invalid_op_not_panic() {
    // Companion to test_sto_out_of_range_after_shrink: verifies op_rcl
    // bounds-safety after SIZE shrink (audited in 22-03-01).
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Size(5)).unwrap();
    let result = dispatch(&mut s, Op::RclReg(50));
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "RCL 50 on SIZE-5 state must be InvalidOp, got {result:?}"
    );
}

#[test]
fn test_sigma_plus_on_shrunk_size_rejects_not_panic() {
    // Pitfall 5 sentinel: Op::Size(3) (regs.len() < 7) followed by
    // Op::SigmaPlus returns Err(InvalidOp) and MUST NOT PANIC. Verifies
    // the entry guard added in 22-03-02 protects the Σ-block R01..R06
    // access from underflow.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Size(3)).unwrap();
    let result = dispatch(&mut s, Op::SigmaPlus);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "Σ+ on SIZE-3 state must be InvalidOp, got {result:?}"
    );
}

#[test]
fn test_sigma_plus_on_size_7_succeeds() {
    // Boundary case: regs.len() == 7 is exactly enough for R01..R06 access.
    // Σ+ must succeed (not be over-eagerly rejected).
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Size(7)).unwrap();
    let result = dispatch(&mut s, Op::SigmaPlus);
    assert!(
        result.is_ok(),
        "Σ+ on SIZE-7 state must succeed (boundary), got {result:?}"
    );
}

#[test]
fn test_clreg_after_size_honors_current_size() {
    // Bounds-audit verification (22-03-03): CLREG after SIZE shrink must
    // produce regs.len() == new SIZE, NOT silently re-grow to 100.
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Size(20)).unwrap();
    // Stamp values then call Clreg
    for r in s.regs.iter_mut() {
        *r = HpNum::from(99i32);
    }
    dispatch(&mut s, Op::Clreg).unwrap();
    assert_eq!(s.regs.len(), 20, "CLREG must honor current SIZE (was 20)");
    for r in &s.regs {
        assert_eq!(*r, HpNum::zero(), "CLREG must zero each surviving cell");
    }
}

// ─── CLA: FN-MEM-02 + D-22.13 ───────────────────────────────────────────────

#[test]
fn test_cla_clears_alpha() {
    let mut s = CalcState::new();
    s.alpha_reg = "HELLO".to_string();
    dispatch(&mut s, Op::Cla).unwrap();
    assert!(
        s.alpha_reg.is_empty(),
        "CLA must clear alpha_reg; got: {:?}",
        s.alpha_reg
    );
}

#[test]
fn test_cla_equivalent_to_alpha_clear() {
    // D-22.13 sentinel: Op::Cla and Op::AlphaClear must have identical
    // effect on state.alpha_reg. The duplication is intentional (different
    // prgm_display names); the underlying op_alpha_clear is shared.
    let mut a = CalcState::new();
    a.alpha_reg = "WORLD".to_string();
    let mut b = CalcState::new();
    b.alpha_reg = "WORLD".to_string();

    dispatch(&mut a, Op::AlphaClear).unwrap();
    dispatch(&mut b, Op::Cla).unwrap();

    assert_eq!(
        a.alpha_reg, b.alpha_reg,
        "Op::Cla must produce same alpha_reg as Op::AlphaClear"
    );
    assert!(a.alpha_reg.is_empty());
    assert!(b.alpha_reg.is_empty());
}

#[test]
fn test_cla_preserves_stack() {
    // CLA should only touch alpha_reg; stack registers untouched.
    let mut s = CalcState::new();
    s.alpha_reg = "ANYTHING".to_string();
    s.stack.x = HpNum::from(11i32);
    s.stack.y = HpNum::from(22i32);
    s.stack.z = HpNum::from(33i32);
    s.stack.t = HpNum::from(44i32);
    s.stack.lastx = HpNum::from(55i32);

    dispatch(&mut s, Op::Cla).unwrap();

    assert_eq!(s.stack.x, HpNum::from(11i32));
    assert_eq!(s.stack.y, HpNum::from(22i32));
    assert_eq!(s.stack.z, HpNum::from(33i32));
    assert_eq!(s.stack.t, HpNum::from(44i32));
    assert_eq!(s.stack.lastx, HpNum::from(55i32));
}

// ─── CLST: FN-MEM-03 + D-22.14 ──────────────────────────────────────────────

#[test]
fn test_clst_zeros_xyzt() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1i32);
    s.stack.y = HpNum::from(2i32);
    s.stack.z = HpNum::from(3i32);
    s.stack.t = HpNum::from(4i32);

    dispatch(&mut s, Op::Clst).unwrap();

    assert_eq!(s.stack.x, HpNum::zero(), "X must be 0");
    assert_eq!(s.stack.y, HpNum::zero(), "Y must be 0");
    assert_eq!(s.stack.z, HpNum::zero(), "Z must be 0");
    assert_eq!(s.stack.t, HpNum::zero(), "T must be 0");
}

#[test]
fn test_clst_preserves_lastx_and_lift_enabled() {
    // D-22.14 sentinel: CLST zeros X/Y/Z/T but PRESERVES lastx and
    // lift_enabled. This is the critical divergence from a hypothetical
    // "reset everything stack-related" op.
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1i32);
    s.stack.y = HpNum::from(2i32);
    s.stack.z = HpNum::from(3i32);
    s.stack.t = HpNum::from(4i32);
    s.stack.lastx = HpNum::from(42i32);
    s.stack.lift_enabled = false;

    dispatch(&mut s, Op::Clst).unwrap();

    // X/Y/Z/T zeroed
    assert_eq!(s.stack.x, HpNum::zero());
    assert_eq!(s.stack.y, HpNum::zero());
    assert_eq!(s.stack.z, HpNum::zero());
    assert_eq!(s.stack.t, HpNum::zero());
    // LASTX preserved
    assert_eq!(
        s.stack.lastx,
        HpNum::from(42i32),
        "CLST must NOT touch LASTX"
    );
    // lift_enabled preserved
    assert!(
        !s.stack.lift_enabled,
        "CLST must NOT touch lift_enabled (was false)"
    );
}

#[test]
fn test_clst_preserves_lift_enabled_when_true() {
    // Complement to the above: also confirm the true case is preserved.
    let mut s = CalcState::new();
    s.stack.lift_enabled = true;
    s.stack.lastx = HpNum::from(7i32);

    dispatch(&mut s, Op::Clst).unwrap();

    assert!(
        s.stack.lift_enabled,
        "CLST must NOT touch lift_enabled (was true)"
    );
    assert_eq!(s.stack.lastx, HpNum::from(7i32));
}

#[test]
fn test_clst_preserves_regs() {
    // CLST should only touch the 4-level stack; regs untouched.
    let mut s = CalcState::new();
    for (i, r) in s.regs.iter_mut().enumerate() {
        *r = HpNum::from(i as i32);
    }
    let regs_before = s.regs.clone();

    dispatch(&mut s, Op::Clst).unwrap();

    assert_eq!(s.regs, regs_before, "CLST must NOT touch regs");
}

// ─── PACK: FN-MEM-04 + D-22.12 ──────────────────────────────────────────────

#[test]
fn test_pack_returns_ok() {
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::Pack);
    assert!(result.is_ok(), "PACK must return Ok(()); got {result:?}");
}

#[test]
fn test_pack_is_noop() {
    // PACK should not alter ANY observable state (D-22.12 documented no-op).
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1i32);
    s.stack.y = HpNum::from(2i32);
    s.stack.z = HpNum::from(3i32);
    s.stack.t = HpNum::from(4i32);
    s.stack.lastx = HpNum::from(5i32);
    s.stack.lift_enabled = true;
    s.alpha_reg = "STAYS".to_string();
    s.program = vec![Op::Add, Op::Sub];
    for (i, r) in s.regs.iter_mut().enumerate() {
        *r = HpNum::from((i + 10) as i32);
    }
    let regs_before = s.regs.clone();
    let program_before = s.program.clone();

    dispatch(&mut s, Op::Pack).unwrap();

    assert_eq!(s.stack.x, HpNum::from(1i32));
    assert_eq!(s.stack.y, HpNum::from(2i32));
    assert_eq!(s.stack.z, HpNum::from(3i32));
    assert_eq!(s.stack.t, HpNum::from(4i32));
    assert_eq!(s.stack.lastx, HpNum::from(5i32));
    assert!(s.stack.lift_enabled);
    assert_eq!(s.alpha_reg, "STAYS");
    assert_eq!(s.regs, regs_before);
    assert_eq!(s.program, program_before);
}
