//! Phase 27 FN-QUAL-04 integration suite: every `_IND` op resolves correctly
//! and rejects non-integer pointers with `HpError::InvalidOp`.
//!
//! Complements `phase24_ind_variants.rs` (Phase 24 example tests with
//! out-of-regs-len + sidecar / inheritance bonuses). This file is the
//! Phase 27 hardening surface — happy-path + non-integer-rejection ONLY,
//! as named in REQUIREMENTS.md FN-QUAL-04 and ROADMAP SC-4.
//!
//! The IND-resolved flag PROPERTY (D-27.9 item 5 — `SF_IND(r) ≡ SF(n)` when
//! regs[r]=n) lives in `proptest_flags.rs` (Plan 27-02) per the D-27.12
//! paradigm split. This file ships example tests; that file ships properties.
//!
//! All 17 IND ops named in FN-QUAL-04 are covered:
//!   Pattern A (plain dispatch, 11 ops × 2 = 22 tests):
//!     StoInd, RclInd, SfFlagInd, CfFlagInd, StoArithInd × 4 (Add/Sub/Mul/Div),
//!     ArclInd, AstoInd, ViewInd.
//!   Pattern B (run_program, 6 ops, ~20 tests including skip/no-skip branches):
//!     IsgInd, DseInd, FlagTestInd × 4 (IsSet, IsClear, IsSetThenClear,
//!     IsClearThenClear).
//!
//! Pattern-B skip-semantic ops MUST drive through `run_program` per the
//! Phase 24 precedent in `phase24_ind_variants.rs::isg_ind_inside_run_loop`
//! (line ~178). Plain `dispatch` discards the skip signal for IsgInd / DseInd
//! (the `.map(|_| ())` arm in ops/mod.rs) and is a Neutral no-op for
//! FlagTestInd (interactive defense — phase24 line 448).
//!
//! Test names carry the `_fn_qual_04_` infix to avoid grep-collision with
//! `phase24_ind_variants.rs`. Integration-test files are sibling crates so
//! Rust's harness allows name collisions across files, but unique names keep
//! `cargo test ind_` output unambiguous.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::program::run_program;
use hp41_core::ops::{dispatch, FlagTestKind, Op, StoArithKind};
use hp41_core::state::{CalcState, DisplayMode};
use hp41_core::{format_hpnum, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Helpers ───────────────────────────────────────────────────────────────

/// Canonical non-integer pointer used by every `_reject` test in this file.
/// 12.5 is the same literal used by `phase24_ind_variants.rs` (line 48 et al.)
/// so the two suites stay in lockstep against any future change to the
/// resolve_indirect non-integer rejection contract.
///
/// Catches: resolve_indirect missing the `frac != 0` rejection branch.
fn non_integer_register() -> HpNum {
    HpNum::rounded(Decimal::from_str("12.5").unwrap())
}

/// Copy of the helper at `phase24_ind_variants.rs` — read flag n's bit.
fn flag_set_test(flags: u64, n: u8) -> bool {
    flags & (1u64 << n) != 0
}

// ── Pattern A: plain-dispatch IND ops ─────────────────────────────────────
//
// Pattern A covers IND ops with NO run_loop skip semantic. Each macro
// invocation generates two #[test] functions: `<name>_happy` and `<name>_reject`.
// The macro is the FN-QUAL-04 evidence-anchor for the 11 plain-dispatch ops.

/// Pattern-A test pair: happy-path + non-integer-rejection via plain dispatch.
///
/// Layout for every happy test:
///   1. Place an integer pointer (12) in regs[5].
///   2. Run caller-supplied `$setup` (e.g. preload regs[12], set X, etc.).
///   3. dispatch($op).unwrap().
///   4. Run caller-supplied `$assert_happy(&state)` to assert the resolved
///      target register / flag / display reflects the correct effect.
///
/// Layout for every reject test:
///   1. Place a non-integer (12.5) in regs[5].
///   2. Run caller-supplied `$setup` (so any state precondition is identical
///      to the happy case — only the pointer differs).
///   3. dispatch($op) -> must return Err(HpError::InvalidOp).
///
/// Catches:
///   * resolve_indirect Ok-integer branch missing or computing wrong target.
///   * resolve_indirect Err-non-integer branch missing — the fail-closed
///     guard at frac != 0 must reject 12.5 with HpError::InvalidOp.
macro_rules! ind_happy_and_reject {
    ($happy_name:ident, $reject_name:ident, $op:expr, $setup:expr, $assert_happy:expr) => {
        #[test]
        fn $happy_name() {
            // Catches: resolve_indirect Ok-integer branch missing or
            // computing the wrong target register / flag / display.
            let mut state = CalcState::new();
            state.regs[5] = HpNum::from(12i32);
            $setup(&mut state);
            dispatch(&mut state, $op).unwrap();
            $assert_happy(&state);
        }

        #[test]
        fn $reject_name() {
            // Catches: resolve_indirect Err-non-integer branch missing —
            // the fail-closed guard at frac != 0 must reject 12.5 with
            // HpError::InvalidOp.
            let mut state = CalcState::new();
            state.regs[5] = non_integer_register();
            $setup(&mut state);
            let result = dispatch(&mut state, $op);
            assert!(
                matches!(result, Err(HpError::InvalidOp)),
                "expected InvalidOp on non-integer pointer, got {:?}",
                result
            );
        }
    };
}

// ── A.1 — STO_IND ─────────────────────────────────────────────────────────
// Catches: STO_IND writing to the POINTER register instead of the RESOLVED
// register (a classic indirection bug class — would store X into regs[5]
// instead of regs[12]).

ind_happy_and_reject!(
    sto_ind_fn_qual_04_happy,
    sto_ind_fn_qual_04_reject,
    Op::StoInd(5),
    |s: &mut CalcState| {
        s.stack.x = HpNum::from(7i32);
    },
    |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(7i32))
);

// ── A.2 — RCL_IND ─────────────────────────────────────────────────────────
// Catches: RCL_IND pushing the POINTER value (12) instead of the RESOLVED
// register's value (99). Symmetric inverse of the STO_IND failure mode.

ind_happy_and_reject!(
    rcl_ind_fn_qual_04_happy,
    rcl_ind_fn_qual_04_reject,
    Op::RclInd(5),
    |s: &mut CalcState| {
        s.regs[12] = HpNum::from(99i32);
    },
    |s: &CalcState| assert_eq!(s.stack.x, HpNum::from(99i32))
);

// ── A.3 — SF_IND ──────────────────────────────────────────────────────────
// flag 12 is a user-range flag (0..29 user, 30..55 system, per op_sf bounds).
// Catches: SF_IND setting bit `5` (the pointer reg number) instead of bit 12
// (the resolved flag index) — would manifest as `flags & (1<<5)` non-zero.

ind_happy_and_reject!(
    sf_flag_ind_fn_qual_04_happy,
    sf_flag_ind_fn_qual_04_reject,
    Op::SfFlagInd(5),
    |_s: &mut CalcState| {},
    |s: &CalcState| assert!(
        flag_set_test(s.flags, 12),
        "SF IND must set flag 12 (regs[5]=12); flags=0b{:b}",
        s.flags
    )
);

// ── A.4 — CF_IND ──────────────────────────────────────────────────────────
// Catches: CF_IND clearing bit 5 (pointer reg) instead of bit 12 (resolved
// flag) — would leave flag 12 untouched.

ind_happy_and_reject!(
    cf_flag_ind_fn_qual_04_happy,
    cf_flag_ind_fn_qual_04_reject,
    Op::CfFlagInd(5),
    |s: &mut CalcState| {
        // Precondition: flag 12 starts SET so the CF effect is observable.
        s.flags = 1u64 << 12;
    },
    |s: &CalcState| assert!(
        !flag_set_test(s.flags, 12),
        "CF IND must clear flag 12; flags=0b{:b}",
        s.flags
    )
);

// ── A.5 — STO+_IND, STO-_IND, STO×_IND, STO÷_IND ──────────────────────────
// `Op::StoArithInd` is a TUPLE variant `(u8, StoArithKind)` per mod.rs —
// NOT a struct variant. Plan template error (PLAN-CHECK Suggestion #1)
// corrected here.

// Catches (STO+): STO_ARITH_IND_ADD computing on the POINTER register's
// numeric value (12) instead of the RESOLVED register's value (3) — would
// produce regs[12] = 12 + 7 = 19 instead of 3 + 7 = 10.
ind_happy_and_reject!(
    sto_add_ind_fn_qual_04_happy,
    sto_add_ind_fn_qual_04_reject,
    Op::StoArithInd(5, StoArithKind::Add),
    |s: &mut CalcState| {
        s.regs[12] = HpNum::from(3i32);
        s.stack.x = HpNum::from(7i32);
    },
    |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(10i32))
);

// Catches (STO-): STO_ARITH_IND_SUB orientation reversed — must compute
// regs[12] = regs[12] - X, NOT X - regs[12].
ind_happy_and_reject!(
    sto_sub_ind_fn_qual_04_happy,
    sto_sub_ind_fn_qual_04_reject,
    Op::StoArithInd(5, StoArithKind::Sub),
    |s: &mut CalcState| {
        s.regs[12] = HpNum::from(10i32);
        s.stack.x = HpNum::from(3i32);
    },
    // STO- semantics: regs[12] = regs[12] - X = 10 - 3 = 7.
    |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(7i32))
);

// Catches (STO×): STO_ARITH_IND_MUL routing the multiplication through the
// pointer reg's value (12 × 3 = 36) instead of the resolved register (4 × 3 = 12).
ind_happy_and_reject!(
    sto_mul_ind_fn_qual_04_happy,
    sto_mul_ind_fn_qual_04_reject,
    Op::StoArithInd(5, StoArithKind::Mul),
    |s: &mut CalcState| {
        s.regs[12] = HpNum::from(4i32);
        s.stack.x = HpNum::from(3i32);
    },
    |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(12i32))
);

// Catches (STO÷): STO_ARITH_IND_DIV orientation reversed — must compute
// regs[12] = regs[12] / X, NOT X / regs[12]. Also catches divide-by-zero
// guard regression (X=3 is safe; a regression that uses pointer-side as
// divisor would still pass here but the orientation assertion catches it).
ind_happy_and_reject!(
    sto_div_ind_fn_qual_04_happy,
    sto_div_ind_fn_qual_04_reject,
    Op::StoArithInd(5, StoArithKind::Div),
    |s: &mut CalcState| {
        s.regs[12] = HpNum::from(12i32);
        s.stack.x = HpNum::from(3i32);
    },
    // STO/ semantics: regs[12] = regs[12] / X = 12 / 3 = 4.
    |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(4i32))
);

// ── A.6 — ARCL_IND ────────────────────────────────────────────────────────
// op_arcl appends regs[resolved_addr]'s formatted value to alpha_reg
// (24-char cap via `chars().count()`). With alpha_reg empty, append == replace.
// Catches: ARCL_IND appending the formatted value of the POINTER register
// (12.0000) instead of the RESOLVED register (42.0000).

ind_happy_and_reject!(
    arcl_ind_fn_qual_04_happy,
    arcl_ind_fn_qual_04_reject,
    Op::ArclInd(5),
    |s: &mut CalcState| {
        s.regs[12] = HpNum::from(42i32);
        s.alpha_reg = String::new();
        s.display_mode = DisplayMode::Fix(4);
    },
    |s: &CalcState| {
        let expected = format_hpnum(&HpNum::from(42i32), &s.display_mode);
        assert_eq!(
            s.alpha_reg, expected,
            "ARCL IND must append regs[12] formatted value to alpha_reg"
        );
    }
);

// ── A.7 — ASTO_IND ────────────────────────────────────────────────────────
// op_asto packs first 6 chars of alpha_reg into state.text_regs (sidecar)
// and zeros state.regs[resolved_addr]. Mirrors phase24_ind_variants.rs.
// Catches: ASTO_IND writing the packed text to text_regs[5] (the pointer
// register) instead of text_regs[12] (the resolved register).

ind_happy_and_reject!(
    asto_ind_fn_qual_04_happy,
    asto_ind_fn_qual_04_reject,
    Op::AstoInd(5),
    |s: &mut CalcState| {
        s.alpha_reg = "HELLO".to_string();
    },
    |s: &CalcState| {
        assert_eq!(
            s.text_regs.get(&12),
            Some(&"HELLO".to_string()),
            "ASTO IND must place packed text in text_regs[12]"
        );
        assert_eq!(
            s.regs[12],
            HpNum::zero(),
            "ASTO IND must zero the numeric slot (no-drift invariant)"
        );
    }
);

// ── A.8 — VIEW_IND (R9 sentinel: shows resolved register's value) ─────────
// Catches: VIEW_IND displaying the POINTER register's formatted value
// (12.0000) instead of the RESOLVED register's value (42.0000) — Phase 24
// R9 mitigation regression.

ind_happy_and_reject!(
    view_ind_fn_qual_04_happy,
    view_ind_fn_qual_04_reject,
    Op::ViewInd(5),
    |s: &mut CalcState| {
        s.regs[12] = HpNum::from(42i32);
        s.display_mode = DisplayMode::Fix(4);
    },
    |s: &CalcState| {
        let expected = format_hpnum(&HpNum::from(42i32), &s.display_mode);
        assert_eq!(
            s.display_override.as_deref(),
            Some(expected.as_str()),
            "VIEW IND must write resolved regs[12]'s formatted value to display_override"
        );
    }
);

// ── Pattern B: run_program-driven IND ops (skip semantic) ────────────────
//
// Per `phase24_ind_variants.rs::isg_ind_inside_run_loop` (line ~178), ISG /
// DSE / FlagTestInd happy paths MUST drive through run_program — plain
// dispatch interactively returns Neutral / discards the skip signal
// (`Op::IsgInd(reg) => indirect::op_isg_ind(state, reg).map(|_| ())` at
// ops/mod.rs; FlagTestInd is `() = { /* interactive no-op */ }` at
// ops/mod.rs).
//
// Test shape:
//   regs[5] = 12;                              // pointer
//   regs[12] = <counter>  OR  flags |= 1<<12;  // resolved target
//   program = [Lbl("T"), <op>, PushNum(1), PushNum(2), Rtn]
//   run_program("T")
//   * if op did NOT skip: PushNum(1) then PushNum(2) execute => X=2, Y=1
//   * if op DID skip:     PushNum(1) is skipped, PushNum(2) executes => X=2, Y=0
//
// Disambiguation: every test asserts on Y as well as X — X==2 is identical
// across skip/no-skip branches because PushNum(2) is the second-after-skip
// step. Y==1 (no skip) vs Y==0 (skip). The Y assertion is the actual
// disambiguator.

// ── B.1 — ISG_IND ─────────────────────────────────────────────────────────
// ISG counter format: ccccc.fffii (current . final + increment).
// `0.005` => current=0, final=5, inc=1 (the increment defaults to 1 when fff
// is left undefined per HP-41 spec). After ISG: current=1 (< 5) ⇒ no skip.
// `5.005` => current=5, final=5, inc=1. ISG returns true at counter exit ⇒ skip.
// Counter convention verified against parse_counter in ops/program.rs and the
// phase24 precedent at line 185.

#[test]
fn isg_ind_fn_qual_04_executes_next_step_when_counter_under_final() {
    // Catches: ISG_IND skip signal mis-routed in run_loop — if the arm
    // skips when current < final, Y would be 0 instead of 1.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::rounded(Decimal::from_str("0.005").unwrap());
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::IsgInd(5),
        Op::PushNum(HpNum::from(1i32)), // executed: ISG didn't skip (post=1, < final=5)
        Op::PushNum(HpNum::from(2i32)), // always executed
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::from(1i32),
        "ISG IND must NOT skip when post-increment counter < final"
    );
}

#[test]
fn isg_ind_fn_qual_04_skips_next_step_when_counter_at_final() {
    // Catches: ISG_IND not skipping when post-increment counter > final.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    // current=5 / final=5 / inc=1 → after ISG current=6 > 5 ⇒ exit, skip.
    state.regs[12] = HpNum::rounded(Decimal::from_str("5.005").unwrap());
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::IsgInd(5),
        Op::PushNum(HpNum::from(1i32)), // SKIPPED
        Op::PushNum(HpNum::from(2i32)), // always executed
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::zero(),
        "ISG IND must skip PushNum(1) when post-increment counter > final"
    );
}

#[test]
fn isg_ind_fn_qual_04_rejects_non_integer_pointer() {
    // Catches: ISG_IND's pre-resolve guard missing — the counter value can be
    // a decimal (.fffii suffix is fractional by construction), but the
    // POINTER register must still be an integer.
    let mut state = CalcState::new();
    state.regs[5] = non_integer_register();
    state.program = vec![Op::Lbl("T".to_string()), Op::IsgInd(5), Op::Rtn];
    let result = run_program(&mut state, "T");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── B.2 — DSE_IND ─────────────────────────────────────────────────────────
// DSE counter: same ccccc.fffii format. After DSE: current decremented.
// Counter exit when current <= final. `5.001` => cur=5,final=1,inc=1 → after
// DSE cur=4 > 1 ⇒ no skip. `1.005` => cur=1,final=5,inc=1 → after DSE cur=0,
// and 0 <= 5 ⇒ skip.

#[test]
fn dse_ind_fn_qual_04_executes_next_step_when_counter_above_final() {
    // Catches: DSE_IND skip mis-routed — must NOT skip when post-decrement
    // counter > final.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::rounded(Decimal::from_str("5.001").unwrap());
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::DseInd(5),
        Op::PushNum(HpNum::from(1i32)), // executed: post-dec=4 > final=1
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::from(1i32),
        "DSE IND must NOT skip when post-decrement counter > final"
    );
}

#[test]
fn dse_ind_fn_qual_04_skips_next_step_when_counter_at_or_below_final() {
    // Catches: DSE_IND not skipping when counter reaches final.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    // cur=1,final=5,inc=1 → after DSE cur=0; 0 <= 5 ⇒ exit, skip.
    state.regs[12] = HpNum::rounded(Decimal::from_str("1.005").unwrap());
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::DseInd(5),
        Op::PushNum(HpNum::from(1i32)), // SKIPPED
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::zero(),
        "DSE IND must skip PushNum(1) when post-decrement counter <= final"
    );
}

#[test]
fn dse_ind_fn_qual_04_rejects_non_integer_pointer() {
    // Catches: DSE_IND's pre-resolve guard missing (symmetric with ISG_IND).
    let mut state = CalcState::new();
    state.regs[5] = non_integer_register();
    state.program = vec![Op::Lbl("T".to_string()), Op::DseInd(5), Op::Rtn];
    let result = run_program(&mut state, "T");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── B.3 — FlagTestInd: FS?_IND (IsSet) ────────────────────────────────────
// `Op::FlagTestInd { kind, ind_reg }` is a STRUCT variant with field name
// `ind_reg` per mod.rs — NOT `flag`. Plan template error
// (PLAN-CHECK Suggestion #1) corrected here.
//
// HP-41 semantics: FS? skips next step when flag is NOT set (test fires "skip
// if false"). So with flag SET → test true → no skip → PushNum(1) executes.

#[test]
fn fs_q_ind_fn_qual_04_executes_next_step_when_flag_set() {
    // Catches: FS?_IND inverted truth table — when flag is SET, test is
    // TRUE, no skip should fire.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSet,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // executed iff flag SET (test true)
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::from(1i32),
        "FS?_IND must NOT skip when flag 12 IS set"
    );
}

#[test]
fn fs_q_ind_fn_qual_04_skips_next_step_when_flag_clear() {
    // Catches: FS?_IND not skipping when flag is clear — the test is FALSE,
    // and HP-41 "skip if false" means PushNum(1) is skipped.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    // flag 12 is clear (default)
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSet,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // SKIPPED
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::zero(),
        "FS?_IND must skip when flag 12 is clear"
    );
}

#[test]
fn fs_q_ind_fn_qual_04_rejects_non_integer_pointer() {
    // Catches: FS?_IND missing pre-resolve guard — pointer in regs[5] must
    // be an integer even though flag IND is "indirect via integer part".
    let mut state = CalcState::new();
    state.regs[5] = non_integer_register();
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSet,
            ind_reg: 5,
        },
        Op::Rtn,
    ];
    let result = run_program(&mut state, "T");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── B.4 — FlagTestInd: FC?_IND (IsClear) ──────────────────────────────────
// Symmetric to FS?_IND with inverted truth table: test true when flag CLEAR.

#[test]
fn fc_q_ind_fn_qual_04_executes_next_step_when_flag_clear() {
    // Catches: FC?_IND inverted truth table — when flag is CLEAR, test is
    // TRUE, no skip should fire.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    // flag 12 is clear (default)
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // executed iff flag CLEAR
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::from(1i32),
        "FC?_IND must NOT skip when flag 12 is clear"
    );
}

#[test]
fn fc_q_ind_fn_qual_04_skips_next_step_when_flag_set() {
    // Catches: FC?_IND not skipping when flag SET — test false ⇒ skip.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // SKIPPED
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::zero(),
        "FC?_IND must skip when flag 12 IS set"
    );
}

#[test]
fn fc_q_ind_fn_qual_04_rejects_non_integer_pointer() {
    // Catches: FC?_IND missing pre-resolve guard.
    let mut state = CalcState::new();
    state.regs[5] = non_integer_register();
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClear,
            ind_reg: 5,
        },
        Op::Rtn,
    ];
    let result = run_program(&mut state, "T");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── B.5 — FlagTestInd: FS?C_IND (IsSetThenClear) ──────────────────────────
// Same truth table as FS?_IND but ALWAYS clears the flag afterward
// (regardless of skip outcome, per RESEARCH A4 strict reading).

#[test]
fn fs_q_c_ind_fn_qual_04_executes_and_clears_when_flag_set() {
    // Catches: FS?C_IND not clearing flag after a "set" test. Per RESEARCH
    // A4: flag is ALWAYS cleared, whether or not skip fires.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSetThenClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // executed: flag was set, no skip
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::from(1i32),
        "FS?C_IND must NOT skip when flag was set"
    );
    assert!(
        !flag_set_test(state.flags, 12),
        "FS?C_IND must ALWAYS clear flag 12 after the test"
    );
}

#[test]
fn fs_q_c_ind_fn_qual_04_skips_and_keeps_clear_when_flag_clear() {
    // Catches: FS?C_IND skip path mis-routed; flag-clear case must still
    // result in flag CLEARED (no-op on already-clear) AND skip fires.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    // flag 12 starts CLEAR
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSetThenClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // SKIPPED (test false)
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::zero(),
        "FS?C_IND must skip when flag was clear"
    );
    assert!(
        !flag_set_test(state.flags, 12),
        "FS?C_IND must keep flag 12 clear after the test"
    );
}

#[test]
fn fs_q_c_ind_fn_qual_04_rejects_non_integer_pointer() {
    // Catches: FS?C_IND missing pre-resolve guard.
    let mut state = CalcState::new();
    state.regs[5] = non_integer_register();
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSetThenClear,
            ind_reg: 5,
        },
        Op::Rtn,
    ];
    let result = run_program(&mut state, "T");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── B.6 — FlagTestInd: FC?C_IND (IsClearThenClear) ────────────────────────
// Same truth table as FC?_IND but ALWAYS clears the flag afterward.

#[test]
fn fc_q_c_ind_fn_qual_04_executes_and_clears_when_flag_clear() {
    // Catches: FC?C_IND not clearing flag after a "clear" test (the no-op
    // path — flag was already clear, still must end clear).
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    // flag 12 starts CLEAR
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClearThenClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // executed: flag was clear, no skip
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::from(1i32),
        "FC?C_IND must NOT skip when flag was clear"
    );
    assert!(
        !flag_set_test(state.flags, 12),
        "FC?C_IND must keep flag 12 clear after the test"
    );
}

#[test]
fn fc_q_c_ind_fn_qual_04_skips_and_clears_when_flag_set() {
    // Catches: FC?C_IND not clearing flag after a SET test (skip path).
    // The "ALWAYS clear" rule applies even when skip fires.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.flags = 1u64 << 12;
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClearThenClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(1i32)), // SKIPPED (test false)
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
    assert_eq!(
        state.stack.y,
        HpNum::zero(),
        "FC?C_IND must skip when flag was set"
    );
    assert!(
        !flag_set_test(state.flags, 12),
        "FC?C_IND must ALWAYS clear flag 12 (even after skip)"
    );
}

#[test]
fn fc_q_c_ind_fn_qual_04_rejects_non_integer_pointer() {
    // Catches: FC?C_IND missing pre-resolve guard.
    let mut state = CalcState::new();
    state.regs[5] = non_integer_register();
    state.program = vec![
        Op::Lbl("T".to_string()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsClearThenClear,
            ind_reg: 5,
        },
        Op::Rtn,
    ];
    let result = run_program(&mut state, "T");
    assert!(matches!(result, Err(HpError::InvalidOp)));
}

// ── Cross-cut: documentary anchors for D-27.12 paradigm split ─────────────
//
// These example tests document the EQUIVALENCE between IND-resolved flag
// ops and their direct-form counterparts. The corresponding PROPERTY tests
// (proptest over n ∈ 0..56) live in `proptest_flags.rs` per D-27.12. These
// examples are deliberately concrete pinpoints — if proptest_flags.rs is
// ever rebuilt or removed, the equivalence at n=12 is still asserted here.

#[test]
fn sf_ind_fn_qual_04_equiv_to_sf_when_resolved_n_12() {
    // Catches: SF_IND drifting from SF — D-27.12 example anchor for the
    // PROPERTY `SF_IND(r) ≡ SF(n) when regs[r]=n` in proptest_flags.rs.
    let mut state_direct = CalcState::new();
    let mut state_indirect = CalcState::new();
    state_indirect.regs[5] = HpNum::from(12i32);

    dispatch(&mut state_direct, Op::SfFlag(12)).unwrap();
    dispatch(&mut state_indirect, Op::SfFlagInd(5)).unwrap();

    assert_eq!(
        state_direct.flags, state_indirect.flags,
        "SF_IND(5) must be equivalent to SF(12) when regs[5]=12 (D-27.12)"
    );
}

#[test]
fn rcl_ind_fn_qual_04_equiv_to_rcl_when_resolved_n_12() {
    // Catches: RCL_IND drifting from RCL — paired with the SF_IND
    // documentary cross-reference. Validates that the resolve_indirect →
    // direct-op delegation produces identical state mutations.
    let mut state_direct = CalcState::new();
    let mut state_indirect = CalcState::new();
    state_indirect.regs[5] = HpNum::from(12i32);
    state_direct.regs[12] = HpNum::from(77i32);
    state_indirect.regs[12] = HpNum::from(77i32);

    dispatch(&mut state_direct, Op::RclReg(12)).unwrap();
    dispatch(&mut state_indirect, Op::RclInd(5)).unwrap();

    assert_eq!(
        state_direct.stack.x, state_indirect.stack.x,
        "RCL_IND(5) must be equivalent to RCL(12) when regs[5]=12 (D-27.12)"
    );
    assert_eq!(
        state_direct.stack.lift_enabled, state_indirect.stack.lift_enabled,
        "RCL_IND must inherit RCL's LiftEffect::Enable (D-24.4 inheritance)"
    );
}
