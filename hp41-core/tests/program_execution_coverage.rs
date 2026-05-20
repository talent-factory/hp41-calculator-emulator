#![allow(clippy::unwrap_used)]

//! Program-context execution coverage for Phase 20–24 ops.
//!
//! Phase 20–24 ops are exercised in `hp41-core/src/ops/mod.rs::dispatch` via
//! interactive tests (`phase20_math.rs`, `phase21_*.rs`, `phase22_*.rs`,
//! `phase23_*.rs`, `phase24_*.rs`). They are NOT exercised through
//! `hp41-core/src/ops/program.rs::execute_op` — the `run_program` execution
//! context. This file closes that gap.
//!
//! **Bug class caught:** divergence between interactive and program-context
//! execution (lift effects, side-channel writes to `print_buffer` /
//! `event_buffer` / `display_override`, `pc` advancement). Phase 22 Pitfall 3
//! (PSE `display_override` survival across run_loop iterations) is exactly
//! this class — without these tests, a future regression on a single
//! `execute_op` arm would only surface in a real running program, not in
//! any existing test.
//!
//! Risk-weighted Priority 1 per RESEARCH §Risk-Weighted Uncovered-Line
//! Inventory: targets `ops/program.rs::execute_op` (lines 647–851).
//!
//! Cross-reference: full Phase 24 IND surface lives in `phase24_ind_variants.rs`
//! and Plan 27-03's `indirect_addressing.rs`; this file only probes the
//! `run_loop` arms.

use hp41_core::ops::{dispatch, program::run_program, FlagTestKind, Op, StoArithKind};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Build a single-op program body: `LBL "T", <op>, RTN`. Reused by every test.
fn build_single_op_program(op: Op) -> Vec<Op> {
    vec![Op::Lbl("T".into()), op, Op::Rtn]
}

/// Install the program and run it from label "T". Reused by every test.
fn run_op_in_program(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    state.program = build_single_op_program(op);
    run_program(state, "T")
}

/// Push a Decimal literal onto X with stack-lift enabled. Reused by setup code.
fn push(state: &mut CalcState, s: &str) {
    state.stack.lift_enabled = true;
    dispatch(
        state,
        Op::PushNum(HpNum::from(Decimal::from_str(s).unwrap())),
    )
    .unwrap();
}

// ── Phase 20: math / conversion ops in program context ─────────────────────

#[test]
fn op_pi_in_run_program() {
    // Catches: program-context divergence on Op::Pi — execute_op arm at program.rs
    let mut state = CalcState::new();
    run_op_in_program(&mut state, Op::Pi).unwrap();
    // PI: HpNum::rounded(3.141592653589793) → 3.141592654 (10 sig digits).
    let expected = HpNum::rounded(Decimal::from_str("3.141592654").unwrap());
    assert_eq!(state.stack.x, expected);
}

#[test]
fn op_rnd_in_run_program() {
    // Catches: program-context divergence on Op::Rnd — execute_op arm at program.rs
    let mut state = CalcState::new();
    state.display_mode = hp41_core::DisplayMode::Fix(2);
    push(&mut state, "3.14159");
    run_op_in_program(&mut state, Op::Rnd).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from_str("3.14").unwrap());
}

#[test]
fn op_frc_in_run_program() {
    // Catches: program-context divergence on Op::Frc — execute_op arm at program.rs
    let mut state = CalcState::new();
    push(&mut state, "3.14");
    run_op_in_program(&mut state, Op::Frc).unwrap();
    // FRC(3.14) = 0.14 — sign-preserving.
    let actual = state.stack.x.inner();
    let expected = Decimal::from_str("0.14").unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn op_abs_negative_in_run_program() {
    // Catches: program-context divergence on Op::Abs (negative branch) —
    // execute_op arm at program.rs. Negative input flips sign; closes
    // RESEARCH §Priority 5 op_abs branch coverage.
    let mut state = CalcState::new();
    push(&mut state, "-7.5");
    run_op_in_program(&mut state, Op::Abs).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from_str("7.5").unwrap());
}

#[test]
fn op_abs_positive_in_run_program() {
    // Catches: program-context divergence on Op::Abs (positive pass-through
    // branch) — closes the symmetric pair of the negative case above.
    let mut state = CalcState::new();
    push(&mut state, "3.5");
    run_op_in_program(&mut state, Op::Abs).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from_str("3.5").unwrap());
}

#[test]
fn op_sign_in_run_program() {
    // Catches: program-context divergence on Op::Sign — execute_op arm at program.rs
    let mut state = CalcState::new();
    push(&mut state, "-42");
    run_op_in_program(&mut state, Op::Sign).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(-1));
}

#[test]
fn op_fact_in_run_program() {
    // Catches: program-context divergence on Op::Fact — execute_op arm at program.rs
    let mut state = CalcState::new();
    push(&mut state, "5");
    run_op_in_program(&mut state, Op::Fact).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(120));
}

#[test]
fn op_mod_in_run_program() {
    // Catches: program-context divergence on Op::Mod (HP-41 sign-follows-Y
    // convention) — execute_op arm at program.rs
    let mut state = CalcState::new();
    push(&mut state, "7");
    push(&mut state, "-3");
    run_op_in_program(&mut state, Op::Mod).unwrap();
    // HP-41: 7 MOD -3 = 1 (sign follows Y; NOT Rust % semantics).
    assert_eq!(state.stack.x.inner(), Decimal::from(1));
}

#[test]
fn op_polar_to_rect_in_run_program() {
    // Catches: program-context divergence on Op::PolarToRect — execute_op arm at program.rs
    let mut state = CalcState::new();
    dispatch(&mut state, Op::SetDeg).unwrap();
    push(&mut state, "5"); // r
    push(&mut state, "0"); // theta = 0 degrees
    run_op_in_program(&mut state, Op::PolarToRect).unwrap();
    // r=5, θ=0° → x=5, y=0 (X-reg holds y-coord per FN-MATH-03).
    let x = state.stack.x.inner();
    let y = state.stack.y.inner();
    assert!(x.abs() < Decimal::from_str("1e-9").unwrap(), "x={x}");
    assert_eq!(y, Decimal::from(5));
}

#[test]
fn op_rect_to_polar_in_run_program() {
    // Catches: program-context divergence on Op::RectToPolar — execute_op arm at program.rs
    let mut state = CalcState::new();
    dispatch(&mut state, Op::SetDeg).unwrap();
    push(&mut state, "3"); // x-coord in Y
    push(&mut state, "4"); // y-coord in X
    run_op_in_program(&mut state, Op::RectToPolar).unwrap();
    // 3-4-5 triangle: r=5, θ=atan2(3,4)≈36.87°. X holds angle, Y holds r.
    assert_eq!(state.stack.y.inner(), Decimal::from(5));
}

// ── Phase 21: flag / display / sound ops in program context ────────────────

#[test]
fn op_sf_flag_in_run_program() {
    // Catches: program-context divergence on Op::SfFlag — execute_op arm at program.rs
    let mut state = CalcState::new();
    run_op_in_program(&mut state, Op::SfFlag(5)).unwrap();
    assert_eq!(state.flags & (1u64 << 5), 1u64 << 5);
}

#[test]
fn op_cf_flag_in_run_program() {
    // Catches: program-context divergence on Op::CfFlag — execute_op arm at program.rs
    let mut state = CalcState::new();
    state.flags = u64::MAX;
    run_op_in_program(&mut state, Op::CfFlag(5)).unwrap();
    assert_eq!(state.flags & (1u64 << 5), 0);
}

#[test]
fn op_flag_test_skip_in_run_program() {
    // Catches: run_loop conditional-skip arm for Op::FlagTest (program.rs).
    // FS? on a CLEAR flag → skip next step. Body: [LBL T, FS?(5), PushNum(1), PushNum(2), RTN].
    // With flag 5 CLEAR, FS? IsSet ⇒ should_skip = true. PushNum(1) skipped.
    // X then receives PushNum(2) = 2.
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("T".into()),
        Op::FlagTest {
            kind: FlagTestKind::IsSet,
            flag: 5,
        },
        Op::PushNum(HpNum::from(1i32)),
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    // X = 2 (PushNum(1) was skipped).
    assert_eq!(state.stack.x.inner(), Decimal::from(2));
}

#[test]
fn op_flag_test_set_then_clear_in_run_program() {
    // Catches: run_loop FS?C always-clear side effect (program.rs).
    // Set flag 5 → FS?C(5) should not skip AND should clear the flag.
    let mut state = CalcState::new();
    state.flags = 1u64 << 5;
    state.program = vec![
        Op::Lbl("T".into()),
        Op::FlagTest {
            kind: FlagTestKind::IsSetThenClear,
            flag: 5,
        },
        Op::PushNum(HpNum::from(42i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    // Side effect: flag 5 always cleared.
    assert_eq!(state.flags & (1u64 << 5), 0);
    // Skip not taken (flag was set ⇒ skip-if-not-set = false): PushNum executed.
    assert_eq!(state.stack.x.inner(), Decimal::from(42));
}

#[test]
fn op_aon_in_run_program() {
    // Catches: program-context divergence on Op::Aon — execute_op arm at program.rs
    let mut state = CalcState::new();
    run_op_in_program(&mut state, Op::Aon).unwrap();
    // Aon sets system flag 48.
    assert_ne!(state.flags & (1u64 << 48), 0);
}

#[test]
fn op_aoff_in_run_program() {
    // Catches: program-context divergence on Op::Aoff — execute_op arm at program.rs
    let mut state = CalcState::new();
    state.flags = 1u64 << 48;
    run_op_in_program(&mut state, Op::Aoff).unwrap();
    assert_eq!(state.flags & (1u64 << 48), 0);
}

#[test]
fn op_cld_in_run_program() {
    // Catches: program-context divergence on Op::Cld — execute_op arm at program.rs
    let mut state = CalcState::new();
    state.display_override = Some("STALE".to_string());
    run_op_in_program(&mut state, Op::Cld).unwrap();
    assert!(state.display_override.is_none());
}

#[test]
fn op_tone_in_run_program() {
    // Catches: program-context divergence on Op::Tone — execute_op arm at program.rs.
    // Tone pushes "TONE n" into event_buffer.
    let mut state = CalcState::new();
    run_op_in_program(&mut state, Op::Tone(3)).unwrap();
    assert!(
        state.event_buffer.iter().any(|e| e.contains("3")),
        "event_buffer must contain TONE 3 marker: {:?}",
        state.event_buffer
    );
}

#[test]
fn op_pse_in_run_program() {
    // Catches: program-context divergence on Op::Pse + Pitfall-3 invariant
    // (display_override + event_buffer "PAUSE 1000" both written; run_loop
    // does NOT break). execute_op arm at program.rs.
    let mut state = CalcState::new();
    push(&mut state, "42");
    state.program = vec![
        Op::Lbl("T".into()),
        Op::Pse,
        Op::PushNum(HpNum::from(99i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    // Pse writes display_override AND event_buffer; does not break run_loop.
    assert!(
        state.display_override.is_some(),
        "PSE must set display_override"
    );
    assert!(
        state.event_buffer.iter().any(|e| e.contains("PAUSE 1000")),
        "event_buffer must contain PAUSE 1000 marker"
    );
    // Subsequent op did execute (X = 99).
    assert_eq!(state.stack.x.inner(), Decimal::from(99));
}

#[test]
fn op_beep_in_run_program() {
    // Catches: program-context divergence on Op::Beep — execute_op arm at program.rs
    let mut state = CalcState::new();
    run_op_in_program(&mut state, Op::Beep).unwrap();
    assert!(
        state.event_buffer.iter().any(|e| e.contains("BEEP")),
        "event_buffer must contain BEEP marker: {:?}",
        state.event_buffer
    );
}

// ── Phase 22: program-control / memory / catalog / ASN ops ─────────────────

#[test]
fn op_cla_in_run_program() {
    // Catches: program-context divergence on Op::Cla — execute_op arm at program.rs.
    // Cla is the hardware-faithful "CLA" alias (vs legacy AlphaClear).
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    run_op_in_program(&mut state, Op::Cla).unwrap();
    assert!(state.alpha_reg.is_empty());
}

#[test]
fn op_clst_in_run_program() {
    // Catches: program-context divergence on Op::Clst — execute_op arm at program.rs.
    // D-22.14 invariant: CLST zeros X/Y/Z/T while preserving LASTX and lift_enabled.
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(1i32);
    state.stack.y = HpNum::from(2i32);
    state.stack.z = HpNum::from(3i32);
    state.stack.t = HpNum::from(4i32);
    state.stack.lastx = HpNum::from(99i32);
    run_op_in_program(&mut state, Op::Clst).unwrap();
    assert!(state.stack.x.is_zero());
    assert!(state.stack.y.is_zero());
    assert!(state.stack.z.is_zero());
    assert!(state.stack.t.is_zero());
    // LASTX preserved per D-22.14.
    assert_eq!(state.stack.lastx.inner(), Decimal::from(99));
}

#[test]
fn op_pack_in_run_program() {
    // Catches: program-context divergence on Op::Pack — execute_op arm in program.rs.
    // PACK is a documented no-op on the flat-Vec program model (D-22.12).
    // The assertion must witness a non-trivial state surface to catch a
    // regressing implementation that mutates regs / flags / lift instead
    // of returning Ok(()): set X=42 + flag 5 + a register, run PACK, then
    // require all three are unchanged.
    let mut state = CalcState::new();
    push(&mut state, "42");
    state.regs[7] = HpNum::rounded(Decimal::from(99));
    state.flags = hp41_core::ops::flags::flag_set(state.flags, 5);
    let x_before = state.stack.x.clone();
    let reg7_before = state.regs[7].clone();
    let flags_before = state.flags;
    let lift_before = state.stack.lift_enabled;
    run_op_in_program(&mut state, Op::Pack).unwrap();
    assert_eq!(state.stack.x, x_before, "PACK must not mutate X");
    assert_eq!(state.regs[7], reg7_before, "PACK must not mutate registers");
    assert_eq!(state.flags, flags_before, "PACK must not mutate flags");
    assert_eq!(
        state.stack.lift_enabled, lift_before,
        "PACK must not mutate lift"
    );
}

#[test]
fn op_size_in_run_program() {
    // Catches: program-context divergence on Op::Size — execute_op arm at program.rs.
    let mut state = CalcState::new();
    run_op_in_program(&mut state, Op::Size(50)).unwrap();
    assert_eq!(state.regs.len(), 50);
}

#[test]
fn op_catalog_in_run_program() {
    // Catches: program-context divergence on Op::Catalog — execute_op arm at program.rs.
    // CAT 2 enumerates the Math Pac I XROM module when bit 0 of xrom_modules is set
    // (default state from Phase 31-04); otherwise it emits "NO XROM".
    let mut state = CalcState::new();
    run_op_in_program(&mut state, Op::Catalog(2)).unwrap();
    assert!(
        state
            .print_buffer
            .iter()
            .any(|l| l.contains("XROM") || l.contains("NO XROM")),
        "CAT 2 must emit XROM enumeration or NO XROM: {:?}",
        state.print_buffer
    );
}

#[test]
fn op_asn_in_run_program() {
    // Catches: program-context divergence on Op::Asn — execute_op arm at program.rs.
    let mut state = CalcState::new();
    run_op_in_program(
        &mut state,
        Op::Asn {
            name: "SIN".to_string(),
            key_code: 11,
        },
    )
    .unwrap();
    assert_eq!(state.assignments.get(&11), Some(&"SIN".to_string()));
}

#[test]
fn op_view_in_run_program() {
    // Catches: program-context divergence on Op::View — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(42i32);
    state.display_mode = hp41_core::DisplayMode::Fix(2);
    run_op_in_program(&mut state, Op::View(3)).unwrap();
    assert!(state.display_override.is_some());
}

#[test]
fn op_aview_in_run_program() {
    // Catches: program-context divergence on Op::AView — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.alpha_reg = "WORLD".to_string();
    run_op_in_program(&mut state, Op::AView).unwrap();
    assert_eq!(state.display_override.as_deref(), Some("WORLD"));
}

#[test]
fn op_stop_in_run_program() {
    // Catches: run_loop Stop break semantic (program.rs).
    // Op::Stop breaks run_loop without writing display_override (unlike Prompt).
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("T".into()),
        Op::PushNum(HpNum::from(7i32)),
        Op::Stop,
        Op::PushNum(HpNum::from(99i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    // Stop broke the loop after PushNum(7); PushNum(99) was NOT executed.
    assert_eq!(state.stack.x.inner(), Decimal::from(7));
}

// ── Phase 23: ALPHA ops in program context ─────────────────────────────────

#[test]
fn op_arcl_in_run_program() {
    // Catches: program-context divergence on Op::Arcl — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(42i32);
    state.display_mode = hp41_core::DisplayMode::Fix(0);
    run_op_in_program(&mut state, Op::Arcl(3)).unwrap();
    // Arcl appends the formatted register value to alpha_reg.
    assert!(state.alpha_reg.contains("42"));
}

#[test]
fn op_asto_in_run_program() {
    // Catches: program-context divergence on Op::Asto — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.alpha_reg = "ABC".to_string();
    run_op_in_program(&mut state, Op::Asto(3)).unwrap();
    // Asto packs alpha into text_regs[3] and zeroes regs[3].
    assert!(state.text_regs.contains_key(&3));
    assert!(state.regs[3].is_zero());
}

#[test]
fn op_atox_in_run_program() {
    // Catches: program-context divergence on Op::Atox — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.alpha_reg = "ABC".to_string();
    run_op_in_program(&mut state, Op::Atox).unwrap();
    // Atox pops 'A' (Unicode 65) and pushes it into X.
    assert_eq!(state.stack.x.inner(), Decimal::from(65));
    // Alpha now has the leading 'A' removed.
    assert_eq!(state.alpha_reg, "BC");
}

#[test]
fn op_xtoa_in_run_program() {
    // Catches: program-context divergence on Op::Xtoa — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(66i32); // 'B'
    run_op_in_program(&mut state, Op::Xtoa).unwrap();
    assert!(state.alpha_reg.contains('B'));
}

#[test]
fn op_arot_in_run_program() {
    // Catches: program-context divergence on Op::Arot — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.alpha_reg = "ABCDE".to_string();
    state.stack.x = HpNum::from(1i32);
    run_op_in_program(&mut state, Op::Arot).unwrap();
    // AROT(1) rotates left by 1: ABCDE → BCDEA.
    assert_eq!(state.alpha_reg, "BCDEA");
}

#[test]
fn op_posa_in_run_program() {
    // Catches: program-context divergence on Op::Posa — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    state.stack.x = HpNum::from(76i32); // 'L' (ASCII 76)
    run_op_in_program(&mut state, Op::Posa).unwrap();
    // First 'L' in "HELLO" is at index 2.
    assert_eq!(state.stack.x.inner(), Decimal::from(2));
}

// ── Phase 24: indirect-addressing ops in program context ───────────────────
//
// Plan 27-03 will ship the full happy/sad-path IND surface in
// `tests/indirect_addressing.rs` per D-27.12. This file probes the
// `run_loop` arms only — ensures every `*Ind` variant compiles and routes
// through `execute_op` (or run_loop for skip-semantic variants).

#[test]
fn op_sto_ind_in_run_program() {
    // Catches: program-context divergence on Op::StoInd — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(7i32); // pointer
    state.stack.x = HpNum::from(99i32); // value
    run_op_in_program(&mut state, Op::StoInd(5)).unwrap();
    assert_eq!(state.regs[7].inner(), Decimal::from(99));
}

#[test]
fn op_rcl_ind_in_run_program() {
    // Catches: program-context divergence on Op::RclInd — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(7i32); // pointer
    state.regs[7] = HpNum::from(123i32); // target value
    run_op_in_program(&mut state, Op::RclInd(5)).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(123));
}

#[test]
fn op_sto_arith_ind_in_run_program() {
    // Catches: program-context divergence on Op::StoArithInd — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(7i32);
    state.regs[7] = HpNum::from(10i32);
    state.stack.x = HpNum::from(3i32);
    run_op_in_program(&mut state, Op::StoArithInd(5, StoArithKind::Add)).unwrap();
    // regs[7] = 10 + 3 = 13
    assert_eq!(state.regs[7].inner(), Decimal::from(13));
}

#[test]
fn op_sf_flag_ind_in_run_program() {
    // Catches: program-context divergence on Op::SfFlagInd — execute_op arm at program.rs.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(10i32); // flag number = 10
    run_op_in_program(&mut state, Op::SfFlagInd(5)).unwrap();
    assert_ne!(state.flags & (1u64 << 10), 0);
}

#[test]
fn op_isg_ind_skip_in_run_program() {
    // Catches: run_loop IsgInd skip semantic (program.rs). Counter exits on
    // first iteration when initial current > final. With pointer regs[5]=12 and
    // regs[12]="5.005" (current=5, target=5, step=1): new_current=6, 6 > 5 ⇒ skip.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::rounded(Decimal::from_str("5.005").unwrap());
    state.program = vec![
        Op::Lbl("T".into()),
        Op::IsgInd(5),
        Op::PushNum(HpNum::from(1i32)), // SKIPPED on counter exit
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    // X = 2 (PushNum(1) was skipped on counter exit).
    assert_eq!(state.stack.x.inner(), Decimal::from(2));
}

#[test]
fn op_dse_ind_skip_in_run_program() {
    // Catches: run_loop DseInd skip semantic (program.rs). Counter exits
    // (new_current <= final). Pointer regs[5]=12; regs[12]="1.000" current=1,
    // target=0, step=1; new_current=0, 0 <= 0 ⇒ skip.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(12i32);
    state.regs[12] = HpNum::rounded(Decimal::from_str("1.000").unwrap());
    state.program = vec![
        Op::Lbl("T".into()),
        Op::DseInd(5),
        Op::PushNum(HpNum::from(1i32)), // SKIPPED on counter exit
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(2));
}

#[test]
fn op_flag_test_ind_skip_in_run_program() {
    // Catches: run_loop FlagTestInd skip + always-clear semantic (program.rs).
    // regs[5] = 7 (flag number 7). Flag 7 is SET; FS?C(IND 5) ⇒ should_skip = false,
    // but always clears flag 7.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(7i32);
    state.flags = 1u64 << 7;
    state.program = vec![
        Op::Lbl("T".into()),
        Op::FlagTestInd {
            kind: FlagTestKind::IsSetThenClear,
            ind_reg: 5,
        },
        Op::PushNum(HpNum::from(42i32)),
        Op::Rtn,
    ];
    run_program(&mut state, "T").unwrap();
    // Always-clear side effect: flag 7 cleared.
    assert_eq!(state.flags & (1u64 << 7), 0);
    // Skip not taken (flag was set ⇒ skip-if-not-set = false): PushNum executed.
    assert_eq!(state.stack.x.inner(), Decimal::from(42));
}
