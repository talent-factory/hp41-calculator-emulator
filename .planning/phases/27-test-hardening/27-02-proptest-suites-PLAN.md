---
phase: 27-test-hardening
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-core/tests/proptest_flags.rs
  - hp41-core/tests/proptest_math.rs
  - proptest-regressions/.gitkeep
  - .gitignore
autonomous: true
requirements:
  - FN-QUAL-02
  - FN-QUAL-03
tags:
  - proptest
  - flag-semantics
  - math-shape-invariants
  - save-load-roundtrip
  - skip-semantics

must_haves:
  truths:
    - "`hp41-core/tests/proptest_flags.rs` ships ALL FIVE properties from D-27.9 + the conditional-skip sentinel from D-27.10."
    - "Property 1 (ROADMAP-3): for all n in 0..56, `SF(n) → FS?(n) = true`; `CF(n) → FC?(n) = true`; `SF(n) → FS?C(n) → FC?(n) = true`. Three proptest blocks; 1024 cases each (D-27.11)."
    - "Property 2 (Independence): for any pair (m, n) with m ≠ n, `SF(m)` leaves `FS?(n)` unchanged. Catches bit-field overflow / mask bugs."
    - "Property 3 (Idempotency): `SF(n); SF(n)` is equivalent to a single `SF(n)`; same for CF. Catches off-by-one toggle bugs."
    - "Property 4 (Save-load roundtrip): random `u64` flag patterns survive `serde_json::to_string → from_str` unchanged. Empirical assertion of the `#[serde(default)]` backward-compat invariant per D-27.9 item 4. Does NOT assert on `print_buffer` / `event_buffer` (those are `#[serde(skip)]`)."
    - "Property 5 (IND-resolved flag semantics): for `(n, r)` where register r holds n as Decimal, `SF_IND(r)` ≡ `SF(n)`. Lives in `proptest_flags.rs` per D-27.12, NOT in `indirect_addressing.rs` (Plan 27-03)."
    - "Conditional-skip sentinel proptest (D-27.10): generates random short programs `[<flag-test op>, <step A>, <step B>]`, runs them, asserts `state.pc` (or post-state X/Y) lands on the correct step. Covers FS?, FC?, FS?C, FC?C — all 4 conditional-test variants from Phase 21. FS?C / FC?C variants additionally assert post-test flag state."
    - "`hp41-core/tests/proptest_math.rs` ships the 5 math shape invariants from D-27.5: FRC + INT round-trip; MOD sign-follows-Y; FACT(n+1) ≈ FACT(n) × (n+1) for n in 0..68; P→R/R→P round-trip within tolerance; RND idempotency across all display modes."
    - "Proptest iteration counts per D-27.11: 1024 cases per flag-invariants block; 256 cases per math-shape block. Configured via per-block `proptest! { #![proptest_config(ProptestConfig::with_cases(N))] ... }` (RESEARCH Pattern 1)."
    - "Each property carries a `// Catches: <bug class>` doc comment per D-27.1 alignment."
    - "`proptest-regressions/` directory exists in the repo and is NOT in `.gitignore` (RESEARCH Pitfall 1 — persisted failing seeds must replay in CI; an empty `.gitkeep` seeds the directory)."
    - "No `hp41-core/src/` source changes; no `hp41-gui/src-tauri/` source changes; `#![deny(clippy::unwrap_used)]` invariant preserved (test files carry `#![allow]`)."
  artifacts:
    - path: "hp41-core/tests/proptest_flags.rs"
      provides: "NEW — flag invariants (ROADMAP-3 + independence + idempotency + save-load roundtrip + IND-resolved) + conditional-skip sentinel (FN-QUAL-03, D-27.9, D-27.10)"
      contains: "proptest!"
      contains_2: "ProptestConfig::with_cases(1024)"
    - path: "hp41-core/tests/proptest_math.rs"
      provides: "NEW — math shape invariants for FRC/INT, MOD, FACT, P↔R, RND (FN-QUAL-02 shape, D-27.5)"
      contains: "ProptestConfig::with_cases(256)"
      contains_2: "arb_hp_decimal"
    - path: "proptest-regressions/.gitkeep"
      provides: "NEW — seeds proptest's auto-persistence directory in the repo per Pitfall 1; future CI runs replay any failing seed deterministically"
    - path: ".gitignore"
      provides: "EDITED IF NEEDED — confirm `proptest-regressions/` is NOT excluded; add an explicit `!proptest-regressions/` un-ignore if a parent rule excludes it (Open Question 3 mitigation)"
  key_links:
    - from: "hp41-core/tests/proptest_flags.rs"
      to: "hp41-core/src/state.rs::CalcState::flags (u64 bit-field) + hp41-core/src/ops/flags.rs::{flag_get, flag_set, flag_clear, op_sf, op_cf} + hp41-core/src/ops/mod.rs::Op::FlagTest"
      via: "Direct dispatch via Op::SfFlag, Op::CfFlag, Op::FlagTest + serde_json roundtrip"
      pattern: "Op::SfFlag\\|Op::CfFlag\\|Op::FlagTest\\|serde_json::to_string"
    - from: "hp41-core/tests/proptest_math.rs"
      to: "hp41-core/src/ops/math.rs::{op_pi, op_rnd, op_frc, op_fact, op_mod, op_polar_to_rect, op_rect_to_polar}"
      via: "Dispatch through Op::Frc, Op::Fact, Op::Mod, Op::Rnd, Op::PolarToRect, Op::RectToPolar"
      pattern: "Op::(Frc|Fact|Mod|Rnd|PolarToRect|RectToPolar)"
    - from: "proptest-regressions/.gitkeep"
      to: "future CI failing-seed replays"
      via: "auto-persistence directory at proptest crate scope"
      pattern: "proptest-regressions"
---

# Plan 27-02: Proptest suites — flag invariants + math shape invariants

**Goal:** Land FN-QUAL-03 (flag-semantics proptest covering ROADMAP-3 + 4 extensions + conditional-skip sentinel) and FN-QUAL-02 shape-invariant complement (math properties — FRC, MOD, FACT, P↔R, RND idempotency). Two new files; no source edits.

**Requirement IDs:** FN-QUAL-02 (shape complement), FN-QUAL-03 (flag semantics)
**Touches:** `hp41-core/tests/` (2 new files), `proptest-regressions/` (new directory + .gitkeep), `.gitignore` (defensive check)
**Plan depends on:** none — independent of 27-01/27-03/27-04 (different test surfaces)

<objective>
Ship two new test files under `hp41-core/tests/` containing the property-based invariants for Phase 27. `proptest_flags.rs` locks the FN-QUAL-03 invariants (the ROADMAP-mandated three + four user-selected extensions per D-27.9 + the conditional-skip sentinel per D-27.10). `proptest_math.rs` locks the FN-QUAL-02 shape complement per D-27.5 — the proptest half of the hybrid (hand-curated cases ship in Plan 27-01, shape invariants here).

Purpose: Property tests catch unknown regressions in invariant shape that hand-curated cases (Plan 27-01) cannot exhaustively enumerate. The flag bit-field (`CalcState::flags: u64`) is exactly the kind of state where independence + idempotency bugs hide for years until a stress sequence trips them; proptest's randomized exploration with persisted-seed replay (RESEARCH Pitfall 1) is the right tool. Similarly, math shape invariants — FRC + INT round-trip, MOD sign-follows-Y, FACT × (n+1), P↔R round-trip, RND idempotency — are general truths about the calculator's algebra that hand-curated cases prove only at specific points.

Output: `proptest_flags.rs` (≥ 8 properties), `proptest_math.rs` (≥ 5 properties), `proptest-regressions/.gitkeep` (the persistence directory).

Out of scope (explicit):
- Coverage push and accuracy-suite hand-curated extension → Plan 27-01
- IND happy/sad example tests → Plan 27-03 (the IND-flag PROPERTY lives here per D-27.12)
- Playwright / Vitest / `gui-ci` changes → Plan 27-04
- Any `hp41-core/src/` change (FROZEN per CLAUDE.md v2.2 additions)
- `Decimal` strategy that crosses HP-41 representable range (exponent stays in -99..=99 per RESEARCH Pattern 2)
- Float-tolerance comparison primitives — reuse the precedent from `hp41-core/tests/numerical_accuracy.rs::passes_with_tol` if needed; do NOT introduce new tolerance helpers
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@CLAUDE.md

# Phase 27 inputs (locked decisions + research)
@.planning/phases/27-test-hardening/27-CONTEXT.md
@.planning/phases/27-test-hardening/27-RESEARCH.md

# Style precedent — read FIRST (the only proptest file in the workspace today)
@hp41-core/tests/proptest_stack.rs

# Existing flag-semantics example tests (complement, do NOT duplicate)
@hp41-core/tests/phase21_flags.rs

# Math op sources (read-only — no edits)
@hp41-core/src/ops/math.rs
@hp41-core/src/ops/flags.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/state.rs

# Cargo.toml verifies proptest pin
@hp41-core/Cargo.toml

<interfaces>
<!-- Key contracts the executor needs. Extracted from the codebase so no
     scavenger hunt is required. -->

# Flag API (hp41-core/src/ops/flags.rs):
#   pub fn flag_get(flags: u64, n: u8) -> bool;          // line 12
#   pub fn flag_set(flags: u64, n: u8) -> u64;           // line 21
#   pub fn flag_clear(flags: u64, n: u8) -> u64;         // similar
#   pub fn op_sf(state: &mut CalcState, n: u8) -> Result<(), HpError>;
#   pub fn op_cf(state: &mut CalcState, n: u8) -> Result<(), HpError>;
# Out-of-range (n >= 56) returns Err(HpError::InvalidOp) — properties must
# constrain n to 0..56 OR cover the rejection path separately.

# Op variants needed (hp41-core/src/ops/mod.rs):
#   Op::SfFlag(u8)                       — set flag
#   Op::CfFlag(u8)                       — clear flag
#   Op::SfFlagInd(u8)                    — IND set (Plan 27-03 happy/sad; here only the property)
#   Op::CfFlagInd(u8)                    — IND clear
#   Op::FlagTest { kind: FlagTestKind, flag: u8 }
#   Op::FlagTestInd { kind: FlagTestKind, flag: u8 }   // IND wrapper
#   pub enum FlagTestKind { IsSet, IsClear, IsSetClear, IsClearClear }
#     — line 63 of ops/mod.rs; IsSetClear is FS?C (test, then clear);
#     IsClearClear is FC?C (test, then clear regardless of test result).

# Math Op variants (hp41-core/src/ops/math.rs + ops/mod.rs):
#   Op::Pi                                — math.rs::op_pi
#   Op::Rnd                               — math.rs::op_rnd  (idempotent per display mode)
#   Op::Frc                               — math.rs::op_frc  (FRC(x) + INT(x) ≈ x)
#   Op::Int                               — math.rs::op_int  (NOTE: verify exact variant name during read_first)
#   Op::Fact                              — math.rs::op_fact
#   Op::Mod                               — math.rs::op_mod  (sign-follows-Y per HP-41 hardware)
#   Op::PolarToRect, Op::RectToPolar     — mode-aware DEG/RAD/GRAD

# State setup helpers (existing precedents):
#   hp41-core/tests/numerical_accuracy.rs::new_deg_state(), new_rad_state(),
#   push(state, "<decimal>"), get_x(state), passes_with_tol(actual, expected, tol)
# Use these via re-import OR copy-paste minimal helpers into the proptest
# files — proptest files cannot directly `use crate::tests::numerical_accuracy::*`
# because integration tests are sibling crates. Pattern: small local helpers.

# proptest 1.11 API (hp41-core/Cargo.toml line 15: proptest = "1.11"):
#   use proptest::prelude::*;
#   use proptest::test_runner::Config as ProptestConfig;
#   proptest! {
#       #![proptest_config(ProptestConfig::with_cases(1024))]
#       #[test]
#       fn name(arg in 0u8..56) { prop_assert!(...); }
#   }
# Each proptest! block can have its own #![proptest_config(...)] at the top.
# For an HP-41 Decimal strategy, see RESEARCH Pattern 2 — `arb_hp_decimal()`
# helper returning `impl Strategy<Value = Decimal>` with mantissa in
# 1u64..10_000_000_000 and exponent in -99..=99.

# CalcState::flags is `pub flags: u64` (verified in src/state.rs); direct
# read/write is permitted in tests (the proptest can assign
# `state.flags = pattern;` for save-load roundtrip property 4).

# serde_json roundtrip:
#   let json = serde_json::to_string(&state).unwrap();
#   let restored: CalcState = serde_json::from_str(&json).unwrap();
# print_buffer / event_buffer are #[serde(skip)] — restored value is always
# empty Vec. The property MUST compare only persisted fields (e.g. `flags`
# directly, NOT a full eq on CalcState).

# Conditional-skip sentinel pattern (RESEARCH Pattern 3):
#   let mut s = CalcState::new();
#   if flag_set { dispatch(&mut s, Op::SfFlag(n))?; } else { dispatch(&mut s, Op::CfFlag(n))?; }
#   s.program = vec![
#       Op::Lbl("T".into()),
#       Op::FlagTest { kind, flag: n },
#       Op::PushNum(HpNum::from(a)),  // skipped if test fails
#       Op::PushNum(HpNum::from(b)),  // always executed
#       Op::Rtn,
#   ];
#   run_program(&mut s, "T")?;
#   // assert state.stack.x == b; state.stack.y == (if test_passed { a } else { 0 })
# This precise pattern catches the run_loop conditional-skip arm regressions.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create proptest_flags.rs with FN-QUAL-03 properties + conditional-skip sentinel</name>

  <files>hp41-core/tests/proptest_flags.rs</files>

  <read_first>
    - hp41-core/tests/proptest_stack.rs (the entire file — 81 lines, the ONLY style precedent in the workspace)
    - hp41-core/tests/phase21_flags.rs (existing flag-semantics example tests — proptest_flags must COMPLEMENT, not duplicate. Verify the existing FS?C / FC?C tests at lines 197–293 — the property locks the invariant the examples sample by hand.)
    - hp41-core/src/ops/flags.rs lines 1–100 (the full flag API including out-of-range rejection logic)
    - hp41-core/src/ops/mod.rs lines 56–80 (FlagTestKind enum) + lines 800–820 (interactive FlagTest dispatch arm)
    - hp41-core/src/state.rs (verify `pub flags: u64` field visibility + #[serde(default)] attribute presence)
    - hp41-core/src/ops/program.rs::run_program signature + the run_loop FlagTest arm (search for `Op::FlagTest`)
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.9, D-27.10, D-27.11, D-27.12
    - .planning/phases/27-test-hardening/27-RESEARCH.md Patterns 1, 3, 4 (proptest config, conditional-skip sentinel, save-load roundtrip) + Pitfall 1 (regressions directory)
  </read_first>

  <action>
    Create `hp41-core/tests/proptest_flags.rs`.

    1. File header:
       - `#![allow(clippy::unwrap_used)]` at the file top (per the CLAUDE.md test-mod pattern).
       - Module doc comment: "Property-based tests for FN-QUAL-03 (flag semantics across all 56 user flags). Covers ROADMAP-mandated 3 invariants (SF→FS?, CF→FC?, SF→FS?C→FC?) per D-27.9 item 1 + four user-selected extensions per D-27.9 items 2–5 (independence, idempotency, save-load roundtrip, IND-resolved) + the conditional-skip semantics sentinel per D-27.10. Iteration counts per D-27.11: 1024 cases per block (flag bit-twiddling is fast). Complements `phase21_flags.rs` (example tests); does NOT duplicate."
       - Imports: `use hp41_core::ops::{dispatch, Op, FlagTestKind}; use hp41_core::ops::flags::flag_get; use hp41_core::ops::program::run_program; use hp41_core::{CalcState, HpError, HpNum}; use proptest::prelude::*; use rust_decimal::Decimal;` (verify each module path during read_first; `flag_get` exact path is `hp41_core::ops::flags::flag_get`).

    2. **Property 1a — ROADMAP-3 invariant: SF(n) ⇒ FS?(n) = true** (1024 cases):
       ```
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
       ```

    3. **Property 1b — ROADMAP-3 invariant: CF(n) ⇒ FC?(n) = true** (1024 cases):
       Symmetric to 1a: dispatch `Op::CfFlag(n)`, assert `!flag_get(s.flags, n)`. Initial state has all flags clear by default, so this property is non-trivial only when preceded by a `Op::SfFlag(n)` to ensure CF actually clears something. Write it as: `dispatch SF(n); dispatch CF(n); prop_assert!(!flag_get(...))`. Add `// Catches: incorrect bit-clear mask in op_cf — wrong mask would leave the flag set after CF(n)`.

    4. **Property 1c — ROADMAP-3 invariant: SF(n) → FS?C(n) → FC?(n) = true** (1024 cases):
       The FS?C op is `FlagTest { kind: FlagTestKind::IsSetClear, flag: n }`. The dispatch interactive arm at ops/mod.rs:804 is a no-op (read-only test) BUT the side effect (clearing the flag after test) is part of the IsSetClear semantic — verify by reading the op_flag_test_isset_clear implementation in flags.rs.

       The property in interactive context:
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(1024))]

           // Catches: FS?C's clear-after-test side effect missing or applied
           // before the test (which would mean the test always returns false).
           #[test]
           fn sf_then_fs_q_c_then_fc_q_is_true(n in 0u8..56) {
               let mut s = CalcState::new();
               dispatch(&mut s, Op::SfFlag(n)).unwrap();
               // FS?C in interactive context — verify behavior matches the
               // documented invariant. If the FS?C side effect (clear after
               // test) is run_loop-only, this property must use run_program
               // — verify during read_first which path is correct.
               dispatch(&mut s, Op::FlagTest { kind: FlagTestKind::IsSetClear, flag: n }).unwrap();
               prop_assert!(!flag_get(s.flags, n));  // FS?C cleared the flag after testing
           }
       }
       ```

       **NOTE:** during read_first, confirm whether `Op::FlagTest` interactive dispatch in mod.rs:804 IS a Neutral no-op (which would mean FS?C does NOT clear the flag interactively). If interactive is no-op-only, this property must run through `run_program` instead. Adapt the property to whichever path the impl uses. The ROADMAP SC-3 wording ("`SF(n); FS?C(n); FC?(n) = true`") implies SF?C does clear when actually executed — write the property in the path where it DOES execute (run_program if interactive is no-op).

    5. **Property 2 — Independence: SF(m) leaves FS?(n) unchanged for m ≠ n** (1024 cases):
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(1024))]

           // Catches: bit-field overflow / mask bugs — if op_sf wrote a
           // multi-bit mask (e.g. flag_set sets both n and n+1), this property
           // fails immediately on the n+1 read.
           #[test]
           fn sf_leaves_other_flags_unchanged(
               m in 0u8..56,
               n in 0u8..56,
               n_initial in any::<bool>(),
           ) {
               prop_assume!(m != n);
               let mut s = CalcState::new();
               if n_initial { dispatch(&mut s, Op::SfFlag(n)).unwrap(); }
               let before = flag_get(s.flags, n);
               dispatch(&mut s, Op::SfFlag(m)).unwrap();
               let after = flag_get(s.flags, n);
               prop_assert_eq!(before, after);
           }
       }
       ```
       Mirror for CF: `cf_leaves_other_flags_unchanged`.

    6. **Property 3 — Idempotency: SF(n); SF(n) ≡ SF(n)** (1024 cases each, two blocks):
       ```
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
       ```
       Mirror for CF: `cf_is_idempotent` (precondition: SF(n) first, so there's something to clear).

    7. **Property 4 — Save-load roundtrip** (1024 cases) — RESEARCH Pattern 4 verbatim:
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(1024))]

           // Catches: regression in #[serde(default)] on CalcState.flags — a
           // missing `#[serde(default)]` would cause from_str to fail on old
           // save files. This property empirically asserts the invariant under
           // random patterns (the v1.x baseline case is the single-pattern
           // existing test in phase21_flags.rs::test_load_v20_save_no_flags_field).
           #[test]
           fn flag_state_round_trips_through_serde(flag_pattern: u64) {
               let mut s = CalcState::new();
               s.flags = flag_pattern;
               let json = serde_json::to_string(&s).unwrap();
               let restored: CalcState = serde_json::from_str(&json).unwrap();
               prop_assert_eq!(restored.flags, flag_pattern);
               // print_buffer / event_buffer are #[serde(skip)] — do NOT
               // assert on them.
           }
       }
       ```

    8. **Property 5 — IND-resolved flag semantics: SF_IND(r) ≡ SF(n) when regs[r] = n** (1024 cases per D-27.12 item 5):
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(1024))]

           // Catches: IND resolution divergence in op_sf_ind vs op_sf — the
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
       ```

       Repeat for CF / FS? / FC? IND equivalence if Op::*Ind variants exist for those (verify during read_first — RESEARCH names SfFlagInd, CfFlagInd, FS?_Ind / FC?_Ind / FS?C_Ind / FC?C_Ind in the FN-QUAL-04 inventory). Pick at minimum SfFlagInd + CfFlagInd; the IsgInd/DseInd skip-arms are covered in Plan 27-03.

    9. **Property 6 — Conditional-skip sentinel** (1024 cases per D-27.10) — RESEARCH Pattern 3:
       ```
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
                   Op::Lbl("T".into()),
                   Op::FlagTest { kind: FlagTestKind::IsSet, flag: n },
                   Op::PushNum(HpNum::from(a)),  // executed iff flag SET
                   Op::PushNum(HpNum::from(b)),  // always executed
                   Op::Rtn,
               ];
               run_program(&mut s, "T").unwrap();
               // When flag SET: X=b, Y=a; when flag CLEAR: X=b, Y=0 (initial)
               prop_assert_eq!(s.stack.x.inner(), Decimal::from(b));
               let expected_y = if flag_set { Decimal::from(a) } else { Decimal::ZERO };
               prop_assert_eq!(s.stack.y.inner(), expected_y);
           }
       }
       ```
       Repeat for the three remaining variants per D-27.10:
       - `fc_q_skip_semantics_match_truth_table` (FC?: skip iff flag SET)
       - `fs_q_c_skip_and_clear_match_truth_table` (FS?C: skip iff flag CLEAR; ALSO assert flag is cleared post-test regardless)
       - `fc_q_c_skip_and_clear_match_truth_table` (FC?C: skip iff flag SET; ALSO assert flag is cleared post-test regardless)

    10. Verify the exact `Op::Lbl` / `Op::Rtn` constructor signatures during read_first (Op::Lbl takes a String or &str; check by reading existing run_program tests in `phase21_flags.rs::test_fs_q_in_program_executes_next_when_flag_set` lines 116–135).

    11. Final pass: every `proptest!` block has its `#![proptest_config(ProptestConfig::with_cases(1024))]` at the top per D-27.11 (RESEARCH Pitfall 2 says the syntax must be a literal call, not a const reference).

    Self-check after Task 1:
    - `cargo test -p hp41-core --test proptest_flags` passes (≥ 11 properties; runtime <30 s).
    - `grep -c "ProptestConfig::with_cases(1024)" hp41-core/tests/proptest_flags.rs` returns ≥ 8 (one per proptest! block; 8–11 blocks total).
    - `grep -c "// Catches:" hp41-core/tests/proptest_flags.rs` returns ≥ 8.
    - `cargo clippy -p hp41-core --tests -- -D warnings` clean.
    - If any property fails with a seed: the seed is auto-persisted to `proptest-regressions/proptest_flags.txt` — this directory is created in Task 3 and committed. Investigate the failure (it may be a REAL bug per RESEARCH Assumption A7 — failure is the point of these tests; the planner should expect possible discovery).
  </action>

  <verify>
    <automated>cargo test -p hp41-core --test proptest_flags 2>&1 | tail -10</automated>
  </verify>

  <done>
    `proptest_flags.rs` ships ≥ 11 properties covering ROADMAP-3 + independence + idempotency + save-load roundtrip + IND-resolved + conditional-skip sentinel (4 variants); all pass; runtime <30 s; each property carries `// Catches:` rationale.
  </done>
</task>

<task type="auto">
  <name>Task 2: Create proptest_math.rs with FN-QUAL-02 shape invariants</name>

  <files>hp41-core/tests/proptest_math.rs</files>

  <read_first>
    - hp41-core/tests/proptest_stack.rs (style precedent — Strategy + proptest! block shape)
    - hp41-core/src/ops/math.rs (the source — verify exact function names: op_pi, op_rnd, op_frc, op_int (or op_int_part), op_fact, op_mod, op_polar_to_rect, op_rect_to_polar)
    - hp41-core/src/state.rs::DisplayMode enum (Fix / Sci / Eng variants — needed for RND idempotency across modes)
    - hp41-core/tests/numerical_accuracy.rs lines 27–80 (helpers passes_with_tol, new_deg_state, new_rad_state — copy minimal local equivalents into proptest_math.rs)
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.5, D-27.11
    - .planning/phases/27-test-hardening/27-RESEARCH.md Patterns 1, 2 + Code Examples 1, 2 (RND idempotency, MOD sign-follows-Y)
  </read_first>

  <action>
    Create `hp41-core/tests/proptest_math.rs`.

    1. File header:
       - `#![allow(clippy::unwrap_used)]` + `#![allow(clippy::approx_constant)]` (the latter for any HP-41-faithful approximate literals).
       - Module doc: "Property-based tests for FN-QUAL-02 shape invariants. Complements the hand-curated `numerical_accuracy.rs` extension (Plan 27-01) by asserting general algebraic truths the hand cases sample only at specific points. Iteration counts per D-27.11: 256 cases per block (math involves `rust_decimal` arithmetic — slower than flag bit-twiddling)."

    2. Imports: `use hp41_core::ops::{dispatch, Op}; use hp41_core::state::{CalcState, DisplayMode}; use hp41_core::{HpError, HpNum}; use proptest::prelude::*; use rust_decimal::Decimal; use rust_decimal::prelude::ToPrimitive; use std::str::FromStr;` (verify DisplayMode location during read_first; it may be in state.rs or its own module).

    3. **Helper: `arb_hp_decimal()` strategy** (RESEARCH Pattern 2 verbatim):
       ```
       /// Generate a Decimal within HP-41's representable range:
       /// mantissa in 1..10^10, exponent in -99..=99, random sign.
       fn arb_hp_decimal() -> impl Strategy<Value = Decimal> {
           (
               any::<bool>(),
               1u64..10_000_000_000u64,
               -99i32..=99i32,
           )
               .prop_map(|(neg, mantissa, exp)| {
                   let mut d = Decimal::from(mantissa);
                   d.set_sign_negative(neg);
                   if exp >= 0 {
                       d * Decimal::from(10i64.pow(exp.min(18) as u32))
                   } else {
                       d / Decimal::from(10i64.pow((-exp).min(18) as u32))
                   }
               })
       }
       ```

       NOTE: the exp.min(18) clamp avoids `i64::pow` overflow on values outside Decimal's representable range. The HP-41 hardware range (-99..+99) is larger than this strategy's effective range (about -18..+18 due to the clamp); that's acceptable because Decimal cannot represent values outside that anyway. Document this in a doc comment on the helper.

    4. **Helper: `passes_with_tol()`** (copy from numerical_accuracy.rs:58):
       ```
       fn passes_with_tol(actual: f64, expected: f64, tol: f64) -> bool {
           if actual.is_nan() || expected.is_nan() { return false; }
           if expected == 0.0 { actual.abs() <= tol } else { ((actual - expected) / expected).abs() <= tol }
       }
       ```

    5. **Property 1 — FRC + INT round-trip: `FRC(x) + INT(x) ≈ x`** (256 cases per D-27.11):
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(256))]

           // Catches: FRC or INT regression that breaks the fundamental
           // decomposition x = INT(x) + FRC(x). E.g. if FRC returns the
           // wrong sign on negative inputs, this property fires immediately.
           #[test]
           fn frc_plus_int_equals_x(d in arb_hp_decimal()) {
               let mut s_frc = CalcState::new();
               s_frc.stack.x = HpNum::from(d);
               dispatch(&mut s_frc, Op::Frc).unwrap();
               let frc_part = s_frc.stack.x.inner().to_f64().unwrap_or(f64::NAN);

               let mut s_int = CalcState::new();
               s_int.stack.x = HpNum::from(d);
               dispatch(&mut s_int, Op::Int).unwrap();   // verify exact Op variant name during read_first
               let int_part = s_int.stack.x.inner().to_f64().unwrap_or(f64::NAN);

               let original = d.to_f64().unwrap_or(f64::NAN);
               prop_assert!(passes_with_tol(frc_part + int_part, original, 1e-9));
           }
       }
       ```

       If `Op::Int` does not exist as a variant, use the equivalent path (e.g. `Op::IntegerPart` or `Op::IPart` — confirm during read_first). If no such op exists in v2.2, skip this property and document the gap in the SUMMARY (FRC alone can be tested via `FRC(x) = x - INT_computed_externally` but that's contrived; better to confirm the variant exists).

    6. **Property 2 — MOD sign-follows-Y** (256 cases) — RESEARCH Example 2 verbatim:
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(256))]

           // Catches: accidental Rust `%` semantics (sign-follows-X) in
           // op_mod. HP-41 hardware uses sign-follows-Y (cross-referenced
           // against Free42 source ops_math.cc::do_mod — HP-41C Owner's
           // Manual p.234).
           #[test]
           fn mod_sign_follows_y(
               y_mag in 1i64..1_000_000,
               x_mag in 1i64..1000,
               y_neg in any::<bool>(),
               x_neg in any::<bool>(),
           ) {
               let y_val = if y_neg { -y_mag } else { y_mag };
               let x_val = if x_neg { -x_mag } else { x_mag };
               let mut s = CalcState::new();
               s.stack.y = HpNum::from(y_val);
               s.stack.x = HpNum::from(x_val);
               dispatch(&mut s, Op::Mod).unwrap();
               let result = s.stack.x.inner();
               if result.is_zero() {
                   // exact-divisible case is sign-agnostic
               } else {
                   prop_assert_eq!(result.is_sign_negative(), y_neg);
               }
           }
       }
       ```

    7. **Property 3 — FACT(n+1) ≈ FACT(n) × (n+1) for n in 0..68** (256 cases):
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(256))]

           // Catches: off-by-one or sign regression in op_fact's inner
           // multiplication. HP-41 hardware: FACT(0)=1, FACT(70) OutOfRange
           // (Owner's Manual p.234); n in 0..=68 is the safe interior.
           #[test]
           fn fact_recursive_invariant(n in 0i32..=68i32) {
               let mut s_n = CalcState::new();
               s_n.stack.x = HpNum::from(n);
               dispatch(&mut s_n, Op::Fact).unwrap();
               let fact_n = s_n.stack.x.inner().to_f64().unwrap_or(f64::NAN);

               let mut s_n1 = CalcState::new();
               s_n1.stack.x = HpNum::from(n + 1);
               dispatch(&mut s_n1, Op::Fact).unwrap();
               let fact_n1 = s_n1.stack.x.inner().to_f64().unwrap_or(f64::NAN);

               // FACT(n+1) ≈ FACT(n) × (n+1) within HP-41 10-digit tolerance
               prop_assert!(passes_with_tol(fact_n1, fact_n * (n + 1) as f64, 1e-8));
           }
       }
       ```

    8. **Property 4 — P→R/R→P round-trip within tolerance** (256 cases):
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(256))]

           // Catches: P↔R conversion regressions — the round-trip identity
           // P→R(R→P(x,y)) ≈ (x,y) is a fundamental shape invariant. Sampled
           // in DEG mode (the default). Tolerance accounts for HP-41 10-digit
           // rounding compounding across 4 trig calls.
           #[test]
           fn polar_rect_round_trip_in_deg_mode(
               x in -1000i32..=1000i32,
               y in -1000i32..=1000i32,
           ) {
               prop_assume!(!(x == 0 && y == 0));   // degenerate origin
               let mut s = CalcState::new();
               dispatch(&mut s, Op::SetDeg).unwrap();
               s.stack.x = HpNum::from(x);
               s.stack.y = HpNum::from(y);
               dispatch(&mut s, Op::RectToPolar).unwrap();
               dispatch(&mut s, Op::PolarToRect).unwrap();
               let x_back = s.stack.x.inner().to_f64().unwrap_or(f64::NAN);
               let y_back = s.stack.y.inner().to_f64().unwrap_or(f64::NAN);
               prop_assert!(passes_with_tol(x_back, x as f64, 1e-6));
               prop_assert!(passes_with_tol(y_back, y as f64, 1e-6));
           }
       }
       ```

       Use WIDE_TOL (1e-6) because four trig calls compound 10-digit BCD rounding. If the property is too tight and produces false failures, widen to 1e-5 with a doc comment explaining the compounding.

    9. **Property 5 — RND idempotency: `RND(RND(x)) = RND(x)` in every display mode** (256 cases) — RESEARCH Example 1:
       ```
       proptest! {
           #![proptest_config(ProptestConfig::with_cases(256))]

           // Catches: RND not actually rounding (no-op) or RND that produces
           // different output on the second call (e.g. trailing-digit drift
           // from BCD->f64->BCD conversion). RESEARCH Pitfall 8: test on the
           // value, not the display string — display-mode 0-digit formatting
           // has special cases the underlying value doesn't share.
           #[test]
           fn rnd_is_idempotent_in_all_display_modes(
               d in arb_hp_decimal(),
               digits in 0u8..=9,
               mode in prop_oneof![Just(0u8), Just(1u8), Just(2u8)],
           ) {
               let mut s = CalcState::new();
               s.display_mode = match mode {
                   0 => DisplayMode::Fix(digits),
                   1 => DisplayMode::Sci(digits),
                   _ => DisplayMode::Eng(digits),
               };
               s.stack.x = HpNum::from(d);
               dispatch(&mut s, Op::Rnd).unwrap();
               let after_first = s.stack.x.clone();
               dispatch(&mut s, Op::Rnd).unwrap();
               prop_assert_eq!(after_first.inner(), s.stack.x.inner());
           }
       }
       ```

    10. Verify the exact `s.display_mode` field name and `DisplayMode::Fix(_)` variant constructor during read_first.

    Self-check after Task 2:
    - `cargo test -p hp41-core --test proptest_math` passes (≥ 5 properties; runtime <60 s).
    - `grep -c "ProptestConfig::with_cases(256)" hp41-core/tests/proptest_math.rs` returns ≥ 5.
    - `grep -c "// Catches:" hp41-core/tests/proptest_math.rs` returns ≥ 5.
    - `cargo clippy -p hp41-core --tests -- -D warnings` clean.
    - If a property fails: the seed is auto-persisted to `proptest-regressions/proptest_math.txt` — investigate per RESEARCH Assumption A7. Common case: a tolerance too tight; widen with a doc comment OR a real regression (the bug is the point).
  </action>

  <verify>
    <automated>cargo test -p hp41-core --test proptest_math 2>&1 | tail -10</automated>
  </verify>

  <done>
    `proptest_math.rs` ships ≥ 5 shape-invariant properties (FRC+INT round-trip, MOD sign-follows-Y, FACT recursive, P↔R round-trip, RND idempotent); all pass within ~60 s; each property carries `// Catches:` rationale.
  </done>
</task>

<task type="auto">
  <name>Task 3: Seed proptest-regressions/ directory (Pitfall 1 mitigation)</name>

  <files>proptest-regressions/.gitkeep, .gitignore</files>

  <read_first>
    - .gitignore (verify it does NOT exclude `proptest-regressions/` directly OR through a parent pattern — RESEARCH Open Question 3 flags this for confirmation)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Pitfalls #1 (the rationale — auto-persistence of failing seeds is the core proptest CI hygiene)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Open Questions #3
  </read_first>

  <action>
    Establish the `proptest-regressions/` directory as a tracked-but-empty seed-persistence anchor.

    1. Create the directory (if `mkdir -p proptest-regressions` is needed, do so).

    2. Create `proptest-regressions/.gitkeep` as an empty file. This is the standard "tracked-but-empty-directory" pattern and ensures `git add` picks the directory up.

    3. Optionally add a one-line `proptest-regressions/.gitkeep` body comment (in a .md-style sibling file like `proptest-regressions/README.md` OR a leading comment inside .gitkeep itself if a `#` line is harmless) explaining: "Proptest auto-persists failing seeds here (`proptest_<file>.txt`). CI replays persisted seeds before exploring new ones. Do NOT add this directory to `.gitignore` (Phase 27 / 27-02 RESEARCH Pitfall 1 mitigation)." Single .gitkeep file is sufficient; the README is optional.

    4. Audit `.gitignore`:
       - `grep -nE "proptest-regressions|^/?\\*" .gitignore` — verify no pattern excludes the directory.
       - If any pattern (e.g. a broad `target/` rule) does NOT exclude it, no edit needed.
       - If a future-proofing entry is desired, add a positive marker line `# proptest-regressions/ IS tracked — do not gitignore (Phase 27 Pitfall 1)` near the top of `.gitignore`. NOT mandatory; only add if `.gitignore` has many wildcards that could shadow the directory.

    5. Verify the directory exists and is tracked: `git status proptest-regressions/` shows `.gitkeep` as a new file to be committed (not ignored).

    Self-check after Task 3:
    - `test -d proptest-regressions/` exits 0 (directory exists).
    - `git check-ignore -v proptest-regressions/.gitkeep` exits 1 with no output (NOT ignored).
    - The directory is staged in git: `git diff --cached --name-only | grep -E "proptest-regressions"` returns `proptest-regressions/.gitkeep`.
  </action>

  <verify>
    <automated>test -d proptest-regressions && ! git check-ignore proptest-regressions/.gitkeep > /dev/null 2>&1 && echo "OK: directory exists and is tracked"</automated>
  </verify>

  <done>
    `proptest-regressions/` directory exists in the repo with a tracked `.gitkeep` file; `.gitignore` does not exclude it; any future failing-seed file (e.g. `proptest_flags.txt`) auto-persists into this directory and replays in CI per RESEARCH Pitfall 1.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| serde_json parse boundary | The save-load roundtrip property (Property 4 in proptest_flags.rs) exercises this — random u64 patterns serialize and deserialize. Inputs are test-author-controlled; no untrusted input. |
| run_program execution | The conditional-skip sentinel (Property 6) runs short generated programs (≤ 5 ops) through run_loop. Random program shapes stay within the FlagTestKind × n × {a, b} cross-product; no untrusted code execution. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-27-02-01 | Tampering | proptest seed reproducibility | mitigate | `proptest-regressions/` is committed (Task 3) so any failing seed replays deterministically in CI per Pitfall 1. Without persistence, flaky tests would mask real bugs. |
| T-27-02-02 | Denial of Service | hp41-core test suite runtime | mitigate | D-27.11 iteration counts: 1024 for flag (bit-twiddling, ~milliseconds) + 256 for math (decimal arithmetic, ~milliseconds-seconds). Total proptest runtime budget <90 s on a single core, well under existing 2-min ci budget. |
| T-27-02-03 | Information Disclosure | proptest case panic output | accept | proptest's automatic shrink + persistence prints the minimal failing input. Test data is deterministic — no secrets. |
| T-27-02-04 | Tampering | property false positives masking real bugs | mitigate | Per-property `// Catches:` rationale per D-27.1 documents the bug class so a future reader can audit whether the property is actually doing its job. RESEARCH Assumption A7 explicitly authorises the planner to expect potential discovery (a failing property may be a real regression, not a noisy test). |
</threat_model>

<verification>
## Phase-level checks (run after both tasks land)

- `cargo test -p hp41-core --test proptest_flags --test proptest_math` exits 0; total runtime <120 s.
- `grep -c "ProptestConfig::with_cases(1024)" hp41-core/tests/proptest_flags.rs` returns ≥ 8 (D-27.11).
- `grep -c "ProptestConfig::with_cases(256)" hp41-core/tests/proptest_math.rs` returns ≥ 5 (D-27.11).
- `grep -c "// Catches:" hp41-core/tests/proptest_flags.rs hp41-core/tests/proptest_math.rs` returns ≥ 13 (per D-27.1 alignment).
- `test -d proptest-regressions && ls proptest-regressions/.gitkeep` confirms the persistence directory.
- `git check-ignore proptest-regressions/.gitkeep` exits 1 (NOT ignored).
- No `hp41-core/src/` changes: `git diff --stat HEAD~ HEAD -- hp41-core/src/` is empty.
- `cargo clippy -p hp41-core --tests -- -D warnings` clean.

## Nyquist verification dimensions (record in plan SUMMARY)

- **Behavioral:** Both proptest files complete without failure; total runtime acceptable. If any property finds a real bug (Assumption A7), the failing seed in `proptest-regressions/` documents the regression and the bug fix lands in a follow-up plan or is escalated.
- **Functional:** Every D-27.9 item 1–5 + D-27.10 conditional-skip variant has at least one property; every D-27.5 math shape invariant (FRC+INT, MOD sign, FACT recursive, P↔R round-trip, RND idempotent) has at least one property.
- **Regression:** Persisted failing seeds replay on every future CI run per Pitfall 1.
</verification>

<success_criteria>
- [x] FN-QUAL-03 satisfied: proptest_flags.rs covers ROADMAP-3 + 4 extensions (D-27.9) + 4 skip-semantic sentinels (D-27.10)
- [x] FN-QUAL-02 shape complement satisfied: proptest_math.rs covers 5 shape invariants (D-27.5)
- [x] D-27.11 iteration counts: 1024 for flag blocks; 256 for math blocks
- [x] D-27.1 rationale: every property carries `// Catches:` doc comment
- [x] RESEARCH Pitfall 1: `proptest-regressions/` directory exists and is tracked
- [x] No `hp41-core/src/` source changes
- [x] No `hp41-gui/src-tauri/` source changes (SC-4 invariant)
- [x] MSRV 1.88 unchanged
</success_criteria>

<output>
After completion, create `.planning/phases/27-test-hardening/27-02-SUMMARY.md` recording:
- proptest_flags.rs property count + total runtime
- proptest_math.rs property count + total runtime
- Any failing-seed files persisted in `proptest-regressions/` and whether they represent a real bug (Assumption A7 investigation outcome — likely none but the SUMMARY documents the possibility)
- Coverage uplift attributable to this plan (compare `just coverage` numbers before and after — informational only, not a gate; the gate raise lives in Plan 27-01 Task 4)
- Confirmation that Op::Int / Op::IntPart / equivalent variant name was confirmed during execution (Property 1 of proptest_math.rs)
</output>

<failure_modes>
## Failure Modes & Mitigations

- **`Op::Int` variant doesn't exist in v2.2:** RESEARCH names INT(x) as part of the Phase 20 op surface but the exact variant name needs read_first confirmation. If the variant is `Op::IntegerPart` or `Op::IPart` instead, the FRC+INT property uses that name. If no such op exists (FRC ships without an INT counterpart), document the gap in the SUMMARY and skip Property 1 of proptest_math.rs — the other 4 properties are still in scope.
- **A property fails on a real bug (Assumption A7):** the failing seed in `proptest-regressions/` is the bug report. The planner does NOT silently widen tolerance to mask the failure. The decision tree per A7: (a) if the failure is a tolerance issue (e.g. P↔R round-trip with 4× trig compounding needs WIDE_TOL not TOLERANCE), widen with a doc comment explaining; (b) if the failure is a real semantic divergence (e.g. MOD sign-follows-X), escalate as a P0 finding in the SUMMARY and recommend gap-closure for Plan 27-XX. Phase 27 is purely test work — finding a real bug means the bug is genuine. RESEARCH Assumption A7 lists this as the expected outcome.
- **proptest auto-persistence file in `proptest-regressions/` is not committed:** RESEARCH Pitfall 1 — the file lives at `proptest-regressions/proptest_<filename>.txt`. Task 3 seeds the directory; the proptest runner writes the file on first failure. The planner ensures the file is staged and committed alongside its origin proptest file.
- **Per-block `ProptestConfig::with_cases(N)` syntax fails to compile:** RESEARCH Anti-Pattern — only the literal-call form works inside the attribute. `#![proptest_config(ProptestConfig::with_cases(1024))]` is correct. The proptest book confirms this.
- **The conditional-skip sentinel proptest randomly fails the post-test-flag assertion in FS?C/FC?C variants:** verify during read_first whether the FS?C clear-flag side effect runs in interactive dispatch (mod.rs:804) or only in run_loop. The property uses run_program for correctness — adapt to whichever path actually clears the flag per the impl.
- **`proptest_math.rs` P↔R round-trip fails on edge cases (origin, very small magnitudes):** `prop_assume!(!(x == 0 && y == 0))` guards the origin; if other small-magnitude failures surface, widen the tolerance (1e-5 or 1e-4) with a doc comment citing BCD compounding.

## Out of Scope (explicit)
- Coverage push and hand-curated accuracy extension → Plan 27-01
- IND happy/sad-path example tests → Plan 27-03 (the IND-flag PROPERTY lives in proptest_flags.rs per D-27.12)
- Playwright / Vitest / `gui-ci` changes → Plan 27-04
- `hp41-core/src/` edits (frozen)
- `proptest_stack.rs` modifications (existing 81-line precedent stays untouched per RESEARCH §Component Responsibilities)

## References
- 27-CONTEXT.md D-27.5 (shape invariants), D-27.8 (file location), D-27.9 (all 5 flag properties), D-27.10 (skip sentinel), D-27.11 (iteration counts), D-27.12 (IND-flag property goes here)
- 27-RESEARCH.md Patterns 1, 2, 3, 4 (config, decimal strategy, skip sentinel, save-load roundtrip)
- 27-RESEARCH.md Code Examples 1, 2 (RND idempotency, MOD sign)
- 27-RESEARCH.md Pitfall 1 (regressions persistence), Pitfall 2 (region vs line — N/A here), Pitfall 8 (display vs value)
- 27-RESEARCH.md Assumption A7 (expect potential bug discovery)
- 27-RESEARCH.md Open Question 3 (.gitignore audit)
- hp41-core/tests/proptest_stack.rs (style precedent)
- hp41-core/tests/phase21_flags.rs (example-test complement)
</failure_modes>
