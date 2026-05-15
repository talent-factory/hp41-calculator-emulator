#![allow(clippy::unwrap_used)]

//! Property-based tests for FN-QUAL-03 (flag semantics across all 56 user flags).
//!
//! Covers ROADMAP-mandated 3 invariants (SF→FS?, CF→FC?, SF→FS?C→FC?) per
//! D-27.9 item 1 + four user-selected extensions per D-27.9 items 2–5
//! (independence, idempotency, save-load roundtrip, IND-resolved) + the
//! conditional-skip semantics sentinel per D-27.10. Iteration counts per
//! D-27.11: 1024 cases per block (flag bit-twiddling is fast).
//!
//! Complements `phase21_flags.rs` (example tests); does NOT duplicate.
//! The IND-flag property (Property 5) lives here per D-27.12 — example tests
//! for `_IND` ops live in `phase24_ind_variants.rs` / `indirect_addressing.rs`.

use hp41_core::ops::{
    dispatch, flags::flag_get, program::run_program, FlagTestKind, Op,
};
use hp41_core::{CalcState, HpNum};
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rust_decimal::Decimal;

// ─── Property 1a: ROADMAP-3 invariant — SF(n) ⇒ FS?(n) = true ──────────────────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: incorrect bit-set in op_sf — a regression that mis-shifts
    // (e.g. flag_set(n) sets bit (n-1) instead of bit n) would fail FS?
    // immediately. Tests every flag n in 0..56 across 1024 random orderings.
    #[test]
    fn sf_then_fs_q_is_true(n in 0u8..56) {
        let mut s = CalcState::new();
        dispatch(&mut s, Op::SfFlag(n)).unwrap();
        prop_assert!(flag_get(s.flags, n));
    }
}

// ─── Property 1b: ROADMAP-3 invariant — CF(n) ⇒ FC?(n) = true ──────────────────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: incorrect bit-clear mask in op_cf — wrong mask would leave
    // the flag set after CF(n). The SF→CF→FC? sequence is non-trivial
    // because the initial state has all flags clear (so CF alone is a no-op
    // confirmation); SF first ensures the flag was actually set before clear.
    #[test]
    fn sf_then_cf_then_fc_q_is_true(n in 0u8..56) {
        let mut s = CalcState::new();
        dispatch(&mut s, Op::SfFlag(n)).unwrap();
        dispatch(&mut s, Op::CfFlag(n)).unwrap();
        prop_assert!(!flag_get(s.flags, n));
    }
}

// ─── Property 1c: ROADMAP-3 invariant — SF(n) → FS?C(n) → FC?(n) = true ────────
//
// FS?C's clear-after-test side effect only fires inside run_loop (the
// interactive Op::FlagTest dispatch arm at ops/mod.rs:804 is a Neutral
// no-op, per D-21.x and verified by reading the source — phase21_flags.rs
// test_fs_q_c_clears_flag_after_test:197 uses run_program for the same
// reason). The property MUST go through run_program to exercise the
// always-clear side effect.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: FS?C's clear-after-test side effect missing or applied
    // before the test (which would mean the test always returns false).
    // Asserts the ROADMAP-3 sequence SF(n) → FS?C(n) → FC?(n) = true,
    // i.e. after running SF then FS?C, the flag is clear (so FC? would
    // return true).
    #[test]
    fn sf_then_fs_q_c_clears_flag(n in 0u8..56) {
        let mut s = CalcState::new();
        s.program = vec![
            Op::Lbl("T".to_string()),
            Op::SfFlag(n),
            Op::FlagTest {
                kind: FlagTestKind::IsSetThenClear,
                flag: n,
            },
            Op::Rtn,
        ];
        run_program(&mut s, "T").unwrap();
        prop_assert!(!flag_get(s.flags, n));
    }
}

// ─── Property 2a: Independence — SF(m) leaves FS?(n) unchanged for m ≠ n ──────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: bit-field overflow / mask bugs — if op_sf wrote a
    // multi-bit mask (e.g. flag_set sets both n and n+1), this property
    // fails immediately on the n+1 read. The n_initial bool randomizes
    // whether flag n was set before SF(m), exercising both "set→set" and
    // "clear→clear" preservation.
    #[test]
    fn sf_leaves_other_flags_unchanged(
        m in 0u8..56,
        n in 0u8..56,
        n_initial in any::<bool>(),
    ) {
        prop_assume!(m != n);
        let mut s = CalcState::new();
        if n_initial {
            dispatch(&mut s, Op::SfFlag(n)).unwrap();
        }
        let before = flag_get(s.flags, n);
        dispatch(&mut s, Op::SfFlag(m)).unwrap();
        let after = flag_get(s.flags, n);
        prop_assert_eq!(before, after);
    }
}

// ─── Property 2b: Independence — CF(m) leaves FS?(n) unchanged for m ≠ n ──────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: bit-field overflow / mask bugs in op_cf. Mirrors 2a but
    // for the clear operation.
    #[test]
    fn cf_leaves_other_flags_unchanged(
        m in 0u8..56,
        n in 0u8..56,
        n_initial in any::<bool>(),
    ) {
        prop_assume!(m != n);
        let mut s = CalcState::new();
        if n_initial {
            dispatch(&mut s, Op::SfFlag(n)).unwrap();
        }
        let before = flag_get(s.flags, n);
        dispatch(&mut s, Op::CfFlag(m)).unwrap();
        let after = flag_get(s.flags, n);
        prop_assert_eq!(before, after);
    }
}

// ─── Property 3a: Idempotency — SF(n); SF(n) ≡ SF(n) ──────────────────────────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: off-by-one toggle bug — if op_sf were XOR instead of OR,
    // the second SF(n) would TOGGLE rather than re-set, failing this.
    #[test]
    fn sf_is_idempotent(n in 0u8..56) {
        let mut s1 = CalcState::new();
        dispatch(&mut s1, Op::SfFlag(n)).unwrap();
        let flags_after_one = s1.flags;

        let mut s2 = CalcState::new();
        dispatch(&mut s2, Op::SfFlag(n)).unwrap();
        dispatch(&mut s2, Op::SfFlag(n)).unwrap();
        prop_assert_eq!(s2.flags, flags_after_one);
    }
}

// ─── Property 3b: Idempotency — CF(n); CF(n) ≡ CF(n) ──────────────────────────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: off-by-one toggle bug in op_cf. Precondition: SF(n) first
    // so there's something to clear (an initial CalcState has all flags
    // clear; CF alone would be a trivial no-op idempotent).
    #[test]
    fn cf_is_idempotent(n in 0u8..56) {
        let mut s1 = CalcState::new();
        dispatch(&mut s1, Op::SfFlag(n)).unwrap();
        dispatch(&mut s1, Op::CfFlag(n)).unwrap();
        let flags_after_one = s1.flags;

        let mut s2 = CalcState::new();
        dispatch(&mut s2, Op::SfFlag(n)).unwrap();
        dispatch(&mut s2, Op::CfFlag(n)).unwrap();
        dispatch(&mut s2, Op::CfFlag(n)).unwrap();
        prop_assert_eq!(s2.flags, flags_after_one);
    }
}

// ─── Property 4: Save-load roundtrip ──────────────────────────────────────────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: regression in #[serde(default)] on CalcState.flags — a
    // missing #[serde(default)] would cause from_str to fail on old
    // save files. This property empirically asserts the invariant under
    // random patterns (the v1.x baseline case is the single-pattern
    // existing test in phase21_flags.rs::test_load_v20_save_no_flags_field).
    // print_buffer / event_buffer are #[serde(skip)] — do NOT assert on
    // them; restored values are always empty Vec.
    #[test]
    fn flag_state_round_trips_through_serde(flag_pattern: u64) {
        let mut s = CalcState::new();
        s.flags = flag_pattern;
        let json = serde_json::to_string(&s).unwrap();
        let restored: CalcState = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(restored.flags, flag_pattern);
    }
}

// ─── Property 5a: IND-resolved flag semantics — SF_IND(r) ≡ SF(n) ─────────────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: IND resolution divergence in op_sf_flag_ind vs op_sf — the
    // two paths must compute identical post-state. Lives in
    // proptest_flags.rs per D-27.12 (property paradigm); the happy-path
    // example tests for SfFlagInd live in indirect_addressing.rs.
    #[test]
    fn sf_ind_equiv_to_sf_when_resolved(
        n in 0u8..56,
        r in 0u8..100,
    ) {
        // r is the register pointer; regs[r] holds n as a Decimal integer.
        let mut s_direct = CalcState::new();
        s_direct.regs[r as usize] = HpNum::from(n as i32);
        dispatch(&mut s_direct, Op::SfFlag(n)).unwrap();
        let direct_flags = s_direct.flags;

        let mut s_ind = CalcState::new();
        s_ind.regs[r as usize] = HpNum::from(n as i32);
        dispatch(&mut s_ind, Op::SfFlagInd(r)).unwrap();
        prop_assert_eq!(s_ind.flags, direct_flags);
    }
}

// ─── Property 5b: IND-resolved flag semantics — CF_IND(r) ≡ CF(n) ─────────────
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: IND resolution divergence in op_cf_flag_ind vs op_cf.
    // Precondition: SF first (both direct and IND paths) so CF actually
    // has something to clear; the post-state must match across the two
    // resolution paths.
    #[test]
    fn cf_ind_equiv_to_cf_when_resolved(
        n in 0u8..56,
        r in 0u8..100,
    ) {
        let mut s_direct = CalcState::new();
        s_direct.regs[r as usize] = HpNum::from(n as i32);
        dispatch(&mut s_direct, Op::SfFlag(n)).unwrap();
        dispatch(&mut s_direct, Op::CfFlag(n)).unwrap();
        let direct_flags = s_direct.flags;

        let mut s_ind = CalcState::new();
        s_ind.regs[r as usize] = HpNum::from(n as i32);
        dispatch(&mut s_ind, Op::SfFlag(n)).unwrap();
        dispatch(&mut s_ind, Op::CfFlagInd(r)).unwrap();
        prop_assert_eq!(s_ind.flags, direct_flags);
    }
}

// ─── Property 6a: Conditional-skip sentinel — FS? skip semantics ──────────────
//
// Truth table for FS? n: skip the next program step iff flag n is CLEAR
// (i.e. condition "is set" evaluated false → skip). Program shape is
// [LBL, set_or_clear_flag, FS?, push(a), push(b), RTN] — when flag is
// SET, both pushes run (X=b, Y=a); when flag is CLEAR, push(a) is
// skipped (X=b, Y=0 initial).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: run_loop conditional-skip arm regression. The truth
    // table for FS? n: skip the next program step iff flag n is CLEAR.
    // Random (n, flag_set, a, b) cross-product exhaustively explores
    // the (flag-state × program-shape × value) space the example tests
    // in phase21_flags.rs only sample by hand.
    #[test]
    fn fs_q_skip_semantics_match_truth_table(
        n in 0u8..56,
        flag_set in any::<bool>(),
        a in 1i32..100,
        b in 1i32..100,
    ) {
        let mut s = CalcState::new();
        if flag_set {
            dispatch(&mut s, Op::SfFlag(n)).unwrap();
        } else {
            dispatch(&mut s, Op::CfFlag(n)).unwrap();
        }
        s.program = vec![
            Op::Lbl("T".to_string()),
            Op::FlagTest { kind: FlagTestKind::IsSet, flag: n },
            Op::PushNum(HpNum::from(a)),  // executed iff flag SET
            Op::PushNum(HpNum::from(b)),  // always executed
            Op::Rtn,
        ];
        run_program(&mut s, "T").unwrap();
        prop_assert_eq!(s.stack.x.inner(), Decimal::from(b));
        let expected_y = if flag_set { Decimal::from(a) } else { Decimal::ZERO };
        prop_assert_eq!(s.stack.y.inner(), expected_y);
    }
}

// ─── Property 6b: Conditional-skip sentinel — FC? skip semantics ──────────────
//
// FC? skips iff flag is SET (condition "is clear" false → skip). Inverse of FS?.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: FC? skip-arm regression in run_loop — the inverse of FS?.
    // Truth table: skip next step iff flag n is SET.
    #[test]
    fn fc_q_skip_semantics_match_truth_table(
        n in 0u8..56,
        flag_set in any::<bool>(),
        a in 1i32..100,
        b in 1i32..100,
    ) {
        let mut s = CalcState::new();
        if flag_set {
            dispatch(&mut s, Op::SfFlag(n)).unwrap();
        } else {
            dispatch(&mut s, Op::CfFlag(n)).unwrap();
        }
        s.program = vec![
            Op::Lbl("T".to_string()),
            Op::FlagTest { kind: FlagTestKind::IsClear, flag: n },
            Op::PushNum(HpNum::from(a)),  // executed iff flag CLEAR
            Op::PushNum(HpNum::from(b)),  // always executed
            Op::Rtn,
        ];
        run_program(&mut s, "T").unwrap();
        prop_assert_eq!(s.stack.x.inner(), Decimal::from(b));
        let expected_y = if flag_set { Decimal::ZERO } else { Decimal::from(a) };
        prop_assert_eq!(s.stack.y.inner(), expected_y);
    }
}

// ─── Property 6c: Conditional-skip sentinel — FS?C skip-AND-clear semantics ───
//
// FS?C: skip iff flag CLEAR; ALWAYS clear the flag after the test
// regardless of test outcome (RESEARCH A4, ops/mod.rs:60).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: FS?C side-effect omission — the always-clear semantic
    // (regardless of test outcome) MUST fire. Random cross-product
    // covers both "was set → skip = false → clear" and "was clear →
    // skip = true → clear (idempotent)" arms.
    #[test]
    fn fs_q_c_skip_and_clear_match_truth_table(
        n in 0u8..56,
        flag_set in any::<bool>(),
        a in 1i32..100,
        b in 1i32..100,
    ) {
        let mut s = CalcState::new();
        if flag_set {
            dispatch(&mut s, Op::SfFlag(n)).unwrap();
        } else {
            dispatch(&mut s, Op::CfFlag(n)).unwrap();
        }
        s.program = vec![
            Op::Lbl("T".to_string()),
            Op::FlagTest { kind: FlagTestKind::IsSetThenClear, flag: n },
            Op::PushNum(HpNum::from(a)),  // executed iff flag was SET
            Op::PushNum(HpNum::from(b)),  // always executed
            Op::Rtn,
        ];
        run_program(&mut s, "T").unwrap();
        prop_assert_eq!(s.stack.x.inner(), Decimal::from(b));
        let expected_y = if flag_set { Decimal::from(a) } else { Decimal::ZERO };
        prop_assert_eq!(s.stack.y.inner(), expected_y);
        // Always-clear side effect: flag is CLEAR after the test regardless
        // of starting state.
        prop_assert!(!flag_get(s.flags, n), "FS?C must always clear flag");
    }
}

// ─── Property 6d: Conditional-skip sentinel — FC?C skip-AND-clear semantics ───
//
// FC?C: skip iff flag SET; ALWAYS clear the flag after the test.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    // Catches: FC?C side-effect omission — the inverse of FS?C with the
    // same always-clear semantic.
    #[test]
    fn fc_q_c_skip_and_clear_match_truth_table(
        n in 0u8..56,
        flag_set in any::<bool>(),
        a in 1i32..100,
        b in 1i32..100,
    ) {
        let mut s = CalcState::new();
        if flag_set {
            dispatch(&mut s, Op::SfFlag(n)).unwrap();
        } else {
            dispatch(&mut s, Op::CfFlag(n)).unwrap();
        }
        s.program = vec![
            Op::Lbl("T".to_string()),
            Op::FlagTest { kind: FlagTestKind::IsClearThenClear, flag: n },
            Op::PushNum(HpNum::from(a)),  // executed iff flag was CLEAR
            Op::PushNum(HpNum::from(b)),  // always executed
            Op::Rtn,
        ];
        run_program(&mut s, "T").unwrap();
        prop_assert_eq!(s.stack.x.inner(), Decimal::from(b));
        let expected_y = if flag_set { Decimal::ZERO } else { Decimal::from(a) };
        prop_assert_eq!(s.stack.y.inner(), expected_y);
        // Always-clear: flag is CLEAR after the test regardless of state.
        prop_assert!(!flag_get(s.flags, n), "FC?C must always clear flag");
    }
}
