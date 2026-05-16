---
phase: 27-test-hardening
plan: 03
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-core/tests/indirect_addressing.rs
autonomous: true
requirements:
  - FN-QUAL-04
tags:
  - integration-tests
  - indirect-addressing
  - happy-path
  - non-integer-rejection
  - phase24-complement

must_haves:
  truths:
    - "`hp41-core/tests/indirect_addressing.rs` exists with happy-path + non-integer-rejection tests for EVERY `_IND` op named in FN-QUAL-04: STO_IND, RCL_IND, ISG_IND, DSE_IND, SF_IND, CF_IND, FS?_IND, FC?_IND, FS?C_IND, FC?C_IND, STO+_IND, STO-_IND, STO×_IND, STO÷_IND, ARCL_IND, ASTO_IND, VIEW_IND (~17 ops)."
    - "Each test has a one-line `// Catches: <bug class>` rationale per D-27.1 (e.g. 'Catches: non-integer rejection regression in resolve_indirect when register holds Decimal::from_str(\"12.5\")')."
    - "Happy-path pattern: register holds a valid integer pointer (e.g. regs[5] = 12); op dispatches; resolved target register / flag / display reflects the correct effect."
    - "Non-integer-rejection pattern: register holds a non-integer Decimal (e.g. 12.5); op MUST return `Err(HpError::InvalidOp)`; state MUST be unchanged (the resolve_indirect guard is fail-closed per Phase 24 D-24.x design)."
    - "Skip-semantic IND ops (ISG_IND, DSE_IND, and the four FS?_IND / FC?_IND / FS?C_IND / FC?C_IND test-variants) drive through `run_program` per the Phase 24 precedent in `phase24_ind_variants.rs::isg_ind_inside_run_loop` (line 178). Plain dispatch is insufficient — the run_loop arm is where the skip signal is acted on."
    - "The IND-resolved flag PROPERTY (D-27.9 item 5 — `SF_IND(r) ≡ SF(n)` when regs[r]=n) lives in Plan 27-02 `proptest_flags.rs`, NOT here. This plan ships ONLY example tests. The paradigm split per D-27.12 is mandatory."
    - "`indirect_addressing.rs` COMPLEMENTS `phase24_ind_variants.rs` (existing 20.5K file); does NOT duplicate. The Phase 24 file has happy + non-integer + out-of-regs-len + sidecar/inheritance bonuses; this Phase 27 file consolidates the happy + non-integer surface per FN-QUAL-04's exact wording in REQUIREMENTS.md line 106."
    - "No `hp41-core/src/` source changes (FROZEN); no `hp41-gui/src-tauri/` source changes (SC-4 invariant); `#![allow(clippy::unwrap_used)]` at file scope."
  artifacts:
    - path: "hp41-core/tests/indirect_addressing.rs"
      provides: "NEW — Phase 27 FN-QUAL-04 IND integration suite; ≥ 34 tests (17 ops × 2 paths each, happy + reject); skip-semantic ops use run_program"
      contains: "ind_happy_and_reject"
      contains_2: "HpError::InvalidOp"
      contains_3: "// Catches"
      min_lines: 250
  key_links:
    - from: "hp41-core/tests/indirect_addressing.rs"
      to: "hp41-core/src/ops/program.rs::resolve_indirect (the Phase 24 two-tier helper)"
      via: "Every dispatch through Op::*Ind ends up calling resolve_indirect on regs[r]; the test asserts both branches (Ok-integer → target effect; Err-non-integer → InvalidOp + state unchanged)"
      pattern: "resolve_indirect|HpError::InvalidOp"
    - from: "hp41-core/tests/indirect_addressing.rs (skip-semantic ops)"
      to: "hp41-core/src/ops/program.rs::run_loop (the FlagTestInd + IsgInd + DseInd arms)"
      via: "state.program = [<flag-test-or-counter>, <step A>, <step B>, Rtn]; run_program('T'); assert post-state X/Y indicates which branch executed"
      pattern: "run_program"
    - from: "hp41-core/tests/indirect_addressing.rs (FlagTestInd cases)"
      to: "Plan 27-02 hp41-core/tests/proptest_flags.rs::sf_ind_equiv_to_sf_when_resolved (the IND-flag PROPERTY)"
      via: "Documentary cross-reference in a module doc comment — example tests here, property tests in 27-02"
      pattern: "proptest_flags|D-27.12"
---

# Plan 27-03: IND integration suite — every `_IND` op happy + non-integer rejection

**Goal:** Land FN-QUAL-04 via a single new test file `hp41-core/tests/indirect_addressing.rs` covering happy-path + non-integer-rejection for every `_IND` op named in REQUIREMENTS.md line 106. Skip-semantic variants (ISG/DSE/FS?/FC?/FS?C/FC?C) drive through `run_program` per the Phase 24 precedent.

**Requirement IDs:** FN-QUAL-04
**Touches:** `hp41-core/tests/` (1 new file)
**Plan depends on:** none — independent of 27-01/27-02/27-04 (different test surfaces). Documentary cross-reference to Plan 27-02's `proptest_flags.rs` IND-flag property (D-27.12 paradigm split).

<objective>
Ship `hp41-core/tests/indirect_addressing.rs` as the dedicated Phase 27 FN-QUAL-04 example-test suite. Every `_IND` op specified in REQUIREMENTS.md line 106 + ROADMAP.md SC-4 line 199 gets a happy-path test (register holds a valid integer pointer, op dispatches, target effect is observed) and a non-integer-rejection test (register holds e.g. `Decimal::from_str("12.5")`, op MUST return `HpError::InvalidOp`, state MUST be unchanged).

Purpose: FN-QUAL-04 is explicitly named in the requirements as a separate integration suite from the Phase 24 example tests. Per D-27.12, the planner deliberately split this file from `phase24_ind_variants.rs` (existing 20.5K, Phase 24 era) so the Phase 27 hardening surface is self-contained and named per the ROADMAP. The IND-resolved flag PROPERTY lives in `proptest_flags.rs` (Plan 27-02) — keeping example tests here and properties there preserves the paradigm split D-27.12 requires.

Output: One new test file. ≥ 34 tests (17 ops × 2 paths each, plus a small number of skip-semantic-specific run_program tests).

Out of scope (explicit):
- Out-of-regs-len rejection tests — already covered exhaustively in `phase24_ind_variants.rs::*_out_of_regs_len` for every IND op. FN-QUAL-04 names only happy + non-integer; out-of-bounds is Phase 24 surface.
- Sidecar / lift-inheritance / display_override semantics — already covered in `phase24_ind_variants.rs` bonus tests; not in scope for FN-QUAL-04.
- The IND-flag PROPERTY → Plan 27-02 (`proptest_flags.rs` per D-27.12).
- Coverage gate work → Plan 27-01.
- Proptest math properties → Plan 27-02.
- Playwright / Vitest / CI work → Plan 27-04.
- `hp41-core/src/` changes (frozen).
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

# Phase 27 inputs
@.planning/phases/27-test-hardening/27-CONTEXT.md
@.planning/phases/27-test-hardening/27-RESEARCH.md

# Phase 24 context — the IND mechanism this plan tests
@.planning/phases/24-indirect-addressing/

# Style precedent — read FIRST (the 20.5K Phase 24 file this plan complements)
@hp41-core/tests/phase24_ind_variants.rs

# Source — read-only
@hp41-core/src/ops/program.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/ops/registers.rs
@hp41-core/src/ops/flags.rs

<interfaces>
<!-- Key contracts the executor needs. Extracted from the codebase so no
     scavenger hunt is required. -->

# IND ops surface (17 ops named in REQUIREMENTS.md line 106 / ROADMAP SC-4):
#   1. Op::StoInd(u8)                              — store X to regs[regs[r]]
#   2. Op::RclInd(u8)                              — recall regs[regs[r]] to X
#   3. Op::IsgInd(u8)                              — counter inc + skip (run_loop)
#   4. Op::DseInd(u8)                              — counter dec + skip (run_loop)
#   5. Op::SfFlagInd(u8)                           — set flag regs[r]
#   6. Op::CfFlagInd(u8)                           — clear flag regs[r]
#   7. Op::FlagTestInd { kind: FlagTestKind::IsSet, flag: r }       — FS?_IND
#   8. Op::FlagTestInd { kind: FlagTestKind::IsClear, flag: r }     — FC?_IND
#   9. Op::FlagTestInd { kind: FlagTestKind::IsSetClear, flag: r }  — FS?C_IND
#  10. Op::FlagTestInd { kind: FlagTestKind::IsClearClear, flag: r } — FC?C_IND
#  11. Op::StoArithInd { kind: StoArithKind::Add, n: r }            — STO+_IND
#  12. Op::StoArithInd { kind: StoArithKind::Sub, n: r }            — STO-_IND
#  13. Op::StoArithInd { kind: StoArithKind::Mul, n: r }            — STO×_IND
#  14. Op::StoArithInd { kind: StoArithKind::Div, n: r }            — STO÷_IND
#  15. Op::ArclInd(u8)                             — append regs[regs[r]] to ALPHA
#  16. Op::AstoInd(u8)                             — store ALPHA chars to regs[regs[r]]
#  17. Op::ViewInd(u8)                             — display regs[regs[r]] via display_override
#
# Verify exact variant names + struct-vs-tuple shapes during read_first by
# grepping hp41-core/src/ops/mod.rs for "Ind(" and "Ind {". The Phase 24
# precedent in tests/phase24_ind_variants.rs (lines 117–166 for StoArithInd
# variants) is the canonical reference.

# Pattern A — plain dispatch (for ops with no skip-semantic, no run_loop arm):
#   StoInd, RclInd, SfFlagInd, CfFlagInd, StoArithInd (4 kinds), ArclInd,
#   AstoInd, ViewInd. (11 of the 17 ops; 22 tests.)
#
#   Happy: regs[5] = HpNum::from(12); dispatch Op::StoInd(5) with X=7;
#          assert regs[12] == HpNum::from(7).
#   Reject: regs[5] = HpNum::from_str("12.5"); dispatch Op::StoInd(5);
#           assert matches!(result, Err(HpError::InvalidOp)) AND regs[12]
#           is unchanged (still default 0).

# Pattern B — run_program (for skip-semantic ops: IsgInd, DseInd, FlagTestInd
# 4-variants). Reference: phase24_ind_variants.rs::isg_ind_inside_run_loop
# at line 178.
#
#   Happy:
#     state.regs[5] = HpNum::from(12);
#     state.regs[12] = HpNum::from(<initial-counter>);   // ISG/DSE
#     OR state.flags = flag_set_at_index_12_via_op_sf;   // FlagTestInd
#     state.program = vec![
#       Op::Lbl("T".into()),
#       <Op::IsgInd(5) | Op::DseInd(5) | Op::FlagTestInd { kind, flag: 5 }>,
#       Op::PushNum(HpNum::from(99i32)),   // skipped if test skips
#       Op::PushNum(HpNum::from(11i32)),   // always executed
#       Op::Rtn,
#     ];
#     run_program(&mut state, "T").unwrap();
#     // Assert post-state: X == 11; Y == 99 or Y == 0 depending on test result
#
#   Reject:
#     state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
#     state.program = vec![Op::Lbl("T".into()), Op::IsgInd(5), Op::Rtn];
#     let result = run_program(&mut state, "T");
#     assert!(matches!(result, Err(HpError::InvalidOp)));

# Helper macro precedent from RESEARCH §Code Example 3:
#   macro_rules! ind_happy_and_reject {
#       ($name:ident, $op:expr, $setup:expr, $assert_happy:expr) => {
#           #[test] fn $name() { ... }
#       };
#   }
# Define this macro once, invoke per IND op for Pattern-A ops. Pattern-B ops
# write explicit #[test] fns because the program-shape varies per op.

# Existing Phase 24 file (read for style + verify no duplicate test names):
#   sto_ind_happy, sto_ind_non_integer, sto_ind_out_of_regs_len,
#   sto_ind_clears_text_regs_sidecar, rcl_ind_happy, rcl_ind_non_integer,
#   rcl_ind_out_of_regs_len, rcl_ind_lift_enable_inheritance,
#   sto_arith_ind_add_happy, sto_arith_ind_sub_happy, ...
# Phase 27 test names must NOT collide with Phase 24 names. Use a different
# prefix or suffix (e.g. fn_qual_04_sto_ind_happy_path, or just sto_ind_happy
# inside indirect_addressing.rs — Rust integration tests are sibling crates,
# so name collisions ACROSS test files are allowed by the harness BUT
# confusing for grep + readability. Recommend a distinct suffix like
# `_fn_qual_04` OR a doc comment cross-referencing the Phase 24 sibling).

# state.alpha is the ALPHA register (String) used by ArclInd / AstoInd.
# state.display_override is Option<String> used by ViewInd to signal the
# rendered LCD value. Phase 24 file at lines 400–450 has ViewInd examples.

# Helper: flag_set_test (already at the top of phase24_ind_variants.rs):
#   fn flag_set_test(flags: u64, n: u8) -> bool { flags & (1u64 << n) != 0 }
# Copy this helper into indirect_addressing.rs for reuse.

# Op::Lbl signature — verified via phase21_flags.rs line 116–135 + phase24
# usage. Likely Op::Lbl(String) or Op::Lbl(&'static str); confirm during
# read_first.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create indirect_addressing.rs with Pattern-A (plain-dispatch) IND ops</name>

  <files>hp41-core/tests/indirect_addressing.rs</files>

  <read_first>
    - hp41-core/tests/phase24_ind_variants.rs (the entire file — 20.5K, the style precedent. CRITICAL: verify each existing test name so this plan's tests don't collide. Test naming convention: this plan uses the same `<op>_ind_happy_*` / `<op>_ind_reject_*` shape with a distinguishing suffix like `_fn_qual_04` if needed.)
    - hp41-core/src/ops/mod.rs (verify each Op::*Ind variant name + signature — struct-variant vs tuple-variant, exact field names)
    - hp41-core/src/ops/program.rs::resolve_indirect (read the function body to understand the exact error semantics — does it return `Err(HpError::InvalidOp)` directly, or does the caller op return that on top of a more granular `Err` from resolve_indirect?)
    - hp41-core/src/ops/registers.rs (op_sto_arith / op_sto_arith_stack for the StoArithInd path)
    - hp41-core/src/ops/flags.rs (op_sf_ind / op_cf_ind for the flag IND paths)
    - .planning/phases/27-test-hardening/27-CONTEXT.md D-27.12 (paradigm split — example tests here, properties in proptest_flags.rs)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Code Example 3 (the ind_happy_and_reject macro pattern)
    - .planning/REQUIREMENTS.md line 106 (the exact 17-op enumeration: STO/RCL/ISG/DSE/SF/CF/FS?/FC?/FS?C/FC?C/STO+/-/×/÷/ARCL/ASTO/VIEW)
  </read_first>

  <action>
    Create `hp41-core/tests/indirect_addressing.rs`.

    1. File header:
       - `#![allow(clippy::unwrap_used)]` at file top.
       - Module doc:
         ```
         //! Phase 27 FN-QUAL-04 integration suite: every `_IND` op resolves
         //! correctly and rejects non-integer with HpError::InvalidOp.
         //!
         //! Complements `phase24_ind_variants.rs` (Phase 24 example tests with
         //! out-of-regs-len + sidecar/inheritance bonuses). This file is the
         //! Phase 27 hardening surface — happy-path + non-integer-rejection
         //! ONLY, as named in REQUIREMENTS.md FN-QUAL-04 and ROADMAP SC-4.
         //!
         //! The IND-resolved flag PROPERTY (D-27.9 item 5 — SF_IND(r) ≡ SF(n))
         //! lives in `proptest_flags.rs` (Plan 27-02) per D-27.12 paradigm
         //! split. This file ships example tests; that file ships properties.
         //!
         //! All 17 IND ops named in FN-QUAL-04 are covered:
         //!   Pattern A (plain dispatch, 11 ops, 22 tests): StoInd, RclInd,
         //!     SfFlagInd, CfFlagInd, StoArithInd × 4, ArclInd, AstoInd,
         //!     ViewInd.
         //!   Pattern B (run_program, 6 ops, ~14 tests): IsgInd, DseInd,
         //!     FlagTestInd × 4 (FS?, FC?, FS?C, FC?C).
         ```

    2. Imports (verify exact paths during read_first):
       ```
       use hp41_core::ops::{dispatch, Op, StoArithKind, FlagTestKind};
       use hp41_core::ops::program::run_program;
       use hp41_core::{CalcState, HpError, HpNum};
       use rust_decimal::Decimal;
       use std::str::FromStr;
       ```

    3. Helper `fn non_integer_register() -> HpNum`: returns `HpNum::rounded(Decimal::from_str("12.5").unwrap())`. Documented as: "12.5 is the canonical non-integer pointer per phase24_ind_variants.rs precedent. Catches: resolve_indirect missing the `frac != 0` rejection branch."

    4. Helper `fn flag_set_test(flags: u64, n: u8) -> bool`: copy verbatim from phase24_ind_variants.rs:30.

    5. **Pattern-A macro** (RESEARCH Example 3 verbatim):
       ```
       /// Pattern-A test pair: happy-path + non-integer-rejection via plain dispatch.
       /// Use for IND ops with no run_loop skip semantic (StoInd, RclInd, flag-set/clear,
       /// StoArithInd × 4, ArclInd, AstoInd, ViewInd).
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
                   // HpError::InvalidOp. State unchanged invariant.
                   let mut state = CalcState::new();
                   state.regs[5] = non_integer_register();
                   $setup(&mut state);
                   let result = dispatch(&mut state, $op);
                   assert!(matches!(result, Err(HpError::InvalidOp)),
                       "expected InvalidOp on non-integer pointer, got {:?}", result);
               }
           };
       }
       ```

    6. **Section A.1 — STO_IND** (2 tests):
       ```
       ind_happy_and_reject!(
           sto_ind_fn_qual_04_happy,
           sto_ind_fn_qual_04_reject,
           Op::StoInd(5),
           |s: &mut CalcState| s.stack.x = HpNum::from(7i32),
           |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(7i32))
       );
       ```

    7. **Section A.2 — RCL_IND** (2 tests):
       ```
       ind_happy_and_reject!(
           rcl_ind_fn_qual_04_happy,
           rcl_ind_fn_qual_04_reject,
           Op::RclInd(5),
           |s: &mut CalcState| s.regs[12] = HpNum::from(99i32),
           |s: &CalcState| assert_eq!(s.stack.x, HpNum::from(99i32))
       );
       ```

    8. **Section A.3 — SF_IND** (2 tests) — flag pointer:
       ```
       ind_happy_and_reject!(
           sf_flag_ind_fn_qual_04_happy,
           sf_flag_ind_fn_qual_04_reject,
           Op::SfFlagInd(5),
           |_s: &mut CalcState| {},
           |s: &CalcState| assert!(flag_set_test(s.flags, 12))
       );
       ```
       NOTE: in the happy path, regs[5] = 12 means flag 12 is set after dispatch. Verify flag 12 is in the user-range (0..56 per CLAUDE.md flag invariant). Adjust the test pointer (e.g. regs[5] = 30) if flag 12 has system-reserved semantics; verify against hp41-core/src/ops/flags.rs flag-range constraints.

    9. **Section A.4 — CF_IND** (2 tests):
       ```
       ind_happy_and_reject!(
           cf_flag_ind_fn_qual_04_happy,
           cf_flag_ind_fn_qual_04_reject,
           Op::CfFlagInd(5),
           |s: &mut CalcState| { dispatch(s, Op::SfFlag(12)).unwrap(); },   // precondition: flag 12 set
           |s: &CalcState| assert!(!flag_set_test(s.flags, 12))
       );
       ```

    10. **Section A.5 — STO+_IND, STO-_IND, STO×_IND, STO÷_IND** (8 tests, 4 macro invocations):
        ```
        ind_happy_and_reject!(
            sto_add_ind_fn_qual_04_happy,
            sto_add_ind_fn_qual_04_reject,
            Op::StoArithInd { kind: StoArithKind::Add, n: 5 },
            |s: &mut CalcState| {
                s.regs[12] = HpNum::from(3i32);
                s.stack.x = HpNum::from(7i32);
            },
            |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(10i32))
        );
        // ... three more for Sub, Mul, Div with appropriate setup + assert_happy
        ```
        Verify the StoArithInd variant constructor shape (struct vs tuple) during read_first; the example above uses struct form per RESEARCH §Interfaces.

    11. **Section A.6 — ARCL_IND** (2 tests):
        ```
        ind_happy_and_reject!(
            arcl_ind_fn_qual_04_happy,
            arcl_ind_fn_qual_04_reject,
            Op::ArclInd(5),
            |s: &mut CalcState| s.regs[12] = HpNum::from(42i32),
            |s: &CalcState| {
                // ARCL appends regs[12] (formatted) to the ALPHA register.
                // The exact format depends on the current DisplayMode; verify
                // the expected appended substring during read_first.
                assert!(s.alpha.contains("42"));
            }
        );
        ```
        Verify `s.alpha` field name + how `op_arcl` formats numeric values into ALPHA during read_first. The Phase 23 file `hp41-core/tests/phase23_arcl_asto.rs` is the precedent.

    12. **Section A.7 — ASTO_IND** (2 tests):
        ```
        ind_happy_and_reject!(
            asto_ind_fn_qual_04_happy,
            asto_ind_fn_qual_04_reject,
            Op::AstoInd(5),
            |s: &mut CalcState| s.alpha = "HELLO".into(),
            |s: &CalcState| {
                // ASTO stores up to 6 ALPHA chars as packed integer in regs[12].
                // Exact packed format from op_asto — verify during read_first.
                assert!(s.regs[12] != HpNum::from(0i32),
                    "regs[12] should be the packed encoding of 'HELLO'");
            }
        );
        ```
        Refine the assert_happy once op_asto's encoding is confirmed; the precise check may require checking text_regs[12] OR a specific Decimal value. Use phase23_arcl_asto.rs as the precedent.

    13. **Section A.8 — VIEW_IND** (2 tests):
        ```
        ind_happy_and_reject!(
            view_ind_fn_qual_04_happy,
            view_ind_fn_qual_04_reject,
            Op::ViewInd(5),
            |s: &mut CalcState| s.regs[12] = HpNum::from(42i32),
            |s: &CalcState| {
                // VIEW writes regs[12]'s formatted value to display_override.
                assert!(s.display_override.is_some());
                let display = s.display_override.as_ref().unwrap();
                assert!(display.contains("42"), "got {:?}", display);
            }
        );
        ```

    14. **Subtotal after Pattern A:** 22 tests (11 ops × 2 paths). Verify count via `grep -c "^#\[test\]\|^fn.*_fn_qual_04_" indirect_addressing.rs` (macro-generated fns may not produce `^#[test]` headers in cargo-expand; counting macro invocations × 2 is the auditable number).

    Self-check after Task 1:
    - `cargo test -p hp41-core --test indirect_addressing 2>&1 | grep -E "test result|passed|failed"` shows ≥ 22 passes.
    - `grep -c "ind_happy_and_reject!" hp41-core/tests/indirect_addressing.rs` returns ≥ 11.
    - `cargo clippy -p hp41-core --tests -- -D warnings` clean.
    - No test name collision with `phase24_ind_variants.rs` (run `cargo test -p hp41-core 2>&1 | grep ind_ | sort | uniq -d` — must return empty).
  </action>

  <verify>
    <automated>cargo test -p hp41-core --test indirect_addressing 2>&1 | tail -5</automated>
  </verify>

  <done>
    Pattern-A section ships with ≥ 22 tests covering STO_IND, RCL_IND, SF_IND, CF_IND, STO+/-/×/÷ IND, ARCL_IND, ASTO_IND, VIEW_IND (happy + reject each); all pass; macro produces consistent test names; no collision with `phase24_ind_variants.rs`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Pattern-B run_program tests for skip-semantic IND ops (ISG, DSE, FlagTestInd ×4)</name>

  <files>hp41-core/tests/indirect_addressing.rs</files>

  <read_first>
    - hp41-core/tests/phase24_ind_variants.rs::isg_ind_inside_run_loop (lines 178–204) — the canonical run_program-driven IND test pattern
    - hp41-core/tests/phase24_ind_variants.rs::dse_ind_inside_run_loop (lines 226–246)
    - hp41-core/src/ops/program.rs::run_loop (find the IsgInd / DseInd / FlagTestInd arms — the skip signal is consumed here)
    - hp41-core/src/ops/program.rs (verify the ISG/DSE counter format — the parse_counter helper at the file head, mentioned in CLAUDE.md as never `floor()`/`fmod()` on f64)
    - hp41-core/src/ops/mod.rs (verify Op::FlagTestInd struct-variant shape — `kind: FlagTestKind, flag: u8`)
  </read_first>

  <action>
    Append to `hp41-core/tests/indirect_addressing.rs` (continuing the file from Task 1).

    1. Add a section header comment:
       ```
       // ── Pattern B: run_program-driven IND ops (skip semantic) ────────────
       // Per the Phase 24 precedent in phase24_ind_variants.rs::isg_ind_inside_run_loop,
       // ISG / DSE / FlagTestInd happy paths MUST drive through run_program —
       // plain dispatch interactively returns Neutral / discards the skip signal
       // (cf. ops/mod.rs:911 IsgInd .map(|_| ()) arm).
       ```

    2. **ISG_IND happy + reject** (3 tests — happy for skip + happy for execute + reject):
       The ISG counter format is HP-41-specific: `ccccccc.fffii` where ccc is the current count, fff is the final value, ii is the increment. Per CLAUDE.md, parse_counter splits on the decimal point, never floor()/fmod(). Read the canonical test in phase24 lines 178–204 for the exact counter setup.

       Test A — ISG executes next step when counter has NOT reached final:
       ```
       #[test]
       fn isg_ind_fn_qual_04_executes_next_step_when_counter_under_final() {
           // Catches: ISG_IND skip signal misrouted in run_loop — if the arm
           // skips when it should NOT, the post-state pushes are reversed.
           let mut state = CalcState::new();
           state.regs[5] = HpNum::from(12i32);
           // Set counter at regs[12] to "1.005" (cur=1, final=5, inc=1 — not at end).
           state.regs[12] = HpNum::rounded(Decimal::from_str("1.005").unwrap());
           state.program = vec![
               Op::Lbl("T".into()),
               Op::IsgInd(5),
               Op::PushNum(HpNum::from(99i32)),  // EXECUTED — ISG doesn't skip when cur < final
               Op::PushNum(HpNum::from(11i32)),  // always executed
               Op::Rtn,
           ];
           run_program(&mut state, "T").unwrap();
           assert_eq!(state.stack.x.inner(), Decimal::from(11));
           assert_eq!(state.stack.y.inner(), Decimal::from(99));
       }
       ```

       Test B — ISG skips next step when counter has reached final:
       ```
       #[test]
       fn isg_ind_fn_qual_04_skips_next_step_when_counter_at_final() {
           // Catches: ISG_IND not skipping when it should — counter at final
           // means "increment past, skip next step".
           let mut state = CalcState::new();
           state.regs[5] = HpNum::from(12i32);
           state.regs[12] = HpNum::rounded(Decimal::from_str("5.005").unwrap());   // cur=5 = final=5
           state.program = vec![
               Op::Lbl("T".into()),
               Op::IsgInd(5),
               Op::PushNum(HpNum::from(99i32)),  // SKIPPED
               Op::PushNum(HpNum::from(11i32)),  // always executed
               Op::Rtn,
           ];
           run_program(&mut state, "T").unwrap();
           assert_eq!(state.stack.x.inner(), Decimal::from(11));
           assert_eq!(state.stack.y.inner(), Decimal::ZERO);   // 99 was skipped
       }
       ```

       Test C — ISG_IND non-integer rejection (run_program path):
       ```
       #[test]
       fn isg_ind_fn_qual_04_rejects_non_integer_pointer() {
           // Catches: ISG_IND's pre-resolve guard missing — even though the
           // counter value can be a decimal (.fffii suffix), the POINTER
           // register must be an integer.
           let mut state = CalcState::new();
           state.regs[5] = non_integer_register();   // pointer is 12.5 — invalid
           state.program = vec![Op::Lbl("T".into()), Op::IsgInd(5), Op::Rtn];
           let result = run_program(&mut state, "T");
           assert!(matches!(result, Err(HpError::InvalidOp)));
       }
       ```

    3. **DSE_IND happy + reject** (3 tests — symmetric to ISG):
       - `dse_ind_fn_qual_04_executes_next_step_when_counter_above_final` (cur > final, no skip)
       - `dse_ind_fn_qual_04_skips_next_step_when_counter_at_or_below_final` (cur ≤ final, skip)
       - `dse_ind_fn_qual_04_rejects_non_integer_pointer` (pointer 12.5 → InvalidOp)

    4. **FlagTestInd: FS?_IND (4 tests — 2 happy + 1 reject)**:
       ```
       #[test]
       fn fs_q_ind_fn_qual_04_executes_next_step_when_flag_set() {
           // Catches: FS?_IND inverted truth table.
           let mut state = CalcState::new();
           state.regs[5] = HpNum::from(12i32);
           dispatch(&mut state, Op::SfFlag(12)).unwrap();
           state.program = vec![
               Op::Lbl("T".into()),
               Op::FlagTestInd { kind: FlagTestKind::IsSet, flag: 5 },
               Op::PushNum(HpNum::from(99i32)),  // executed iff flag SET
               Op::PushNum(HpNum::from(11i32)),
               Op::Rtn,
           ];
           run_program(&mut state, "T").unwrap();
           assert_eq!(state.stack.x.inner(), Decimal::from(11));
           assert_eq!(state.stack.y.inner(), Decimal::from(99));
       }

       #[test]
       fn fs_q_ind_fn_qual_04_skips_next_step_when_flag_clear() {
           // Catches: FS?_IND not skipping when flag clear.
           let mut state = CalcState::new();
           state.regs[5] = HpNum::from(12i32);
           // flag 12 is clear by default
           state.program = vec![
               Op::Lbl("T".into()),
               Op::FlagTestInd { kind: FlagTestKind::IsSet, flag: 5 },
               Op::PushNum(HpNum::from(99i32)),  // SKIPPED
               Op::PushNum(HpNum::from(11i32)),
               Op::Rtn,
           ];
           run_program(&mut state, "T").unwrap();
           assert_eq!(state.stack.x.inner(), Decimal::from(11));
           assert_eq!(state.stack.y.inner(), Decimal::ZERO);
       }

       #[test]
       fn fs_q_ind_fn_qual_04_rejects_non_integer_pointer() {
           let mut state = CalcState::new();
           state.regs[5] = non_integer_register();
           state.program = vec![
               Op::Lbl("T".into()),
               Op::FlagTestInd { kind: FlagTestKind::IsSet, flag: 5 },
               Op::Rtn,
           ];
           let result = run_program(&mut state, "T");
           assert!(matches!(result, Err(HpError::InvalidOp)));
       }
       ```

    5. **FlagTestInd: FC?_IND (3 tests)** — symmetric to FS?: skip iff flag SET (inverted truth table).
       - `fc_q_ind_fn_qual_04_executes_when_flag_clear`
       - `fc_q_ind_fn_qual_04_skips_when_flag_set`
       - `fc_q_ind_fn_qual_04_rejects_non_integer_pointer`

    6. **FlagTestInd: FS?C_IND (4 tests — happy x2 + reject + post-test flag-state assertion)**:
       - `fs_q_c_ind_fn_qual_04_executes_and_clears_when_flag_set` — set flag 12 first; assert program executes the not-skipped branch AND flag 12 is CLEARED after.
       - `fs_q_c_ind_fn_qual_04_skips_when_flag_clear`
       - `fs_q_c_ind_fn_qual_04_clears_flag_regardless_of_skip` (or fold into the first test's assertion — verify against phase21_flags.rs::test_fs_q_c_clears_flag_after_test pattern)
       - `fs_q_c_ind_fn_qual_04_rejects_non_integer_pointer`

    7. **FlagTestInd: FC?C_IND (4 tests)** — symmetric to FS?C:
       - `fc_q_c_ind_fn_qual_04_executes_and_clears_when_flag_clear`
       - `fc_q_c_ind_fn_qual_04_skips_when_flag_set` (and clears flag 12 after)
       - `fc_q_c_ind_fn_qual_04_clears_flag_regardless_of_skip` (or fold into the prior)
       - `fc_q_c_ind_fn_qual_04_rejects_non_integer_pointer`

    8. **Total Pattern-B tests:** ISG (3) + DSE (3) + FS? (3) + FC? (3) + FS?C (4) + FC?C (4) = 20 tests. Total file ≥ 22 + 20 = 42 tests.

    9. Each Pattern-B test has a `// Catches: <bug class>` doc comment per D-27.1.

    Self-check after Task 2:
    - `cargo test -p hp41-core --test indirect_addressing 2>&1 | grep -E "test result|passed"` shows ≥ 42 passes.
    - `grep -c "// Catches:" hp41-core/tests/indirect_addressing.rs` returns ≥ 30 (Pattern A has macro-side rationale comments inside the macro body; Pattern B tests have explicit comments — so 30+ is a reasonable floor).
    - `grep -c "run_program" hp41-core/tests/indirect_addressing.rs` returns ≥ 20.
    - `cargo clippy -p hp41-core --tests -- -D warnings` clean.
    - No name collision with `phase24_ind_variants.rs`: `cargo test -p hp41-core 2>&1 | grep "test " | awk '{print $2}' | sort | uniq -d` returns empty.
  </action>

  <verify>
    <automated>cargo test -p hp41-core --test indirect_addressing 2>&1 | tail -5</automated>
  </verify>

  <done>
    Pattern-B section adds ≥ 20 run_program-driven tests; file total ≥ 42 tests covering all 17 IND ops × {happy, reject} pairs per FN-QUAL-04; all pass; no name collision with phase24_ind_variants.rs; module doc comment cross-references Plan 27-02's IND-flag property per D-27.12.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| resolve_indirect parse | The non-integer-rejection path is the explicit fail-closed boundary. Tests assert the boundary rejects the canonical 12.5 case. No untrusted input — all test data is author-controlled. |
| run_program execution | Short generated programs (≤ 5 ops) drive ISG / DSE / FlagTestInd skip arms. Test author controls the program shape; no external input. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-27-03-01 | Tampering | resolve_indirect fail-closed guard | mitigate | Every IND op gets a `_reject` test asserting `Err(HpError::InvalidOp)` on 12.5 pointer. If a future refactor accidentally removes the `frac != 0` rejection, all 17 reject tests fire immediately. |
| T-27-03-02 | Denial of Service | run_program runtime budget | mitigate | Pattern-B programs are ≤ 5 ops each; run_program loop limits prevent infinite execution. Per-test runtime <1 ms. |
| T-27-03-03 | Information Disclosure | test panic output | accept | Failing assertions print HpNum / Decimal values only — no secrets. |
</threat_model>

<verification>
## Phase-level checks (run after both tasks land)

- `cargo test -p hp41-core --test indirect_addressing` exits 0; reports ≥ 42 passes.
- Every IND op named in REQUIREMENTS.md FN-QUAL-04 has at least one `_happy` and one `_reject` test. Audit by grep:
  `for op in sto rcl isg dse sf cf fs_q fc_q fs_q_c fc_q_c sto_add sto_sub sto_mul sto_div arcl asto view; do grep -c "${op}_ind_fn_qual_04" hp41-core/tests/indirect_addressing.rs; done` — each line returns ≥ 1.
- `grep -c "phase24_ind_variants\|D-27.12\|proptest_flags" hp41-core/tests/indirect_addressing.rs` returns ≥ 1 (the documentary cross-reference).
- `cargo test -p hp41-core 2>&1 | grep "test " | awk '{print $2}' | sort | uniq -d` returns empty (no name collision with phase24_ind_variants.rs).
- No `hp41-core/src/` changes: `git diff --stat HEAD~ HEAD -- hp41-core/src/` is empty.
- `cargo clippy -p hp41-core --tests -- -D warnings` clean.

## Nyquist verification dimensions (record in plan SUMMARY)

- **Behavioral:** All 17 IND ops resolve correctly on integer pointers and reject 12.5 with InvalidOp. Verified by `cargo test`.
- **Functional:** Every FN-QUAL-04 enumerated op has exactly 2 dispatched tests (happy + reject) for Pattern-A; skip-semantic ops add additional run_program tests for both skip and execute branches.
- **Regression:** A future regression in `resolve_indirect` (e.g. accidentally accepting 12.5) fires 17 reject tests; a regression in run_loop's IsgInd / DseInd / FlagTestInd arms fires the relevant Pattern-B test.
</verification>

<success_criteria>
- [x] FN-QUAL-04 satisfied: `hp41-core/tests/indirect_addressing.rs` exists with happy + reject tests for all 17 IND ops named in REQUIREMENTS.md
- [x] D-27.12 paradigm split honored: example tests here, IND-flag property in `proptest_flags.rs` (Plan 27-02); module doc cross-references the split
- [x] D-27.1 rationale: every test has a `// Catches:` doc comment (Pattern-A via macro body; Pattern-B explicit)
- [x] No collision with `phase24_ind_variants.rs` test names
- [x] No `hp41-core/src/` source changes (frozen)
- [x] No `hp41-gui/src-tauri/` source changes (SC-4)
- [x] MSRV 1.88 unchanged
- [x] `#![allow(clippy::unwrap_used)]` at file scope; no production code changes
</success_criteria>

<output>
After completion, create `.planning/phases/27-test-hardening/27-03-SUMMARY.md` recording:
- Test count per IND op (audit table — 17 rows, 2+ tests each)
- Total file line count + test count
- Confirmation of cross-reference to Plan 27-02's IND-flag property (D-27.12 paradigm split)
- Any IND op for which the test pattern needed adjustment from the planned shape (e.g. ASTO_IND's packed encoding assertion)
- Coverage uplift attributable to this plan (informational; the gate raise lives in Plan 27-01)
</output>

<failure_modes>
## Failure Modes & Mitigations

- **`Op::StoArithInd` field name mismatch:** the variant may use `n` or `reg` or another field name for the pointer; verify against `ops/mod.rs` during read_first and the Phase 24 file at lines 117–166. RESEARCH §Interfaces names the struct form `Op::StoArithInd { kind, n: r }` — adjust if reading reveals otherwise.
- **ARCL_IND / ASTO_IND assertion shape uncertain:** the exact encoding of an HpNum → ALPHA append (ARCL) and ALPHA → HpNum packed integer (ASTO) is in `hp41-core/src/ops/alpha.rs` (per Phase 23). The plan recommends checking phase23_arcl_asto.rs for the canonical assertion shape before finalizing the assert_happy closures. If the encoding is implementation-specific, the assert can be relaxed to "alpha mutated" / "regs[12] != default" with a doc comment explaining the looser assertion.
- **Flag 12 has system semantics:** flags 0–29 are user-flags; 30+ may be system-reserved (DEG/RAD mode, etc.) per HP-41 conventions. If flag 12 turns out to be a system flag, swap to flag 25 (or any clearly-user-range flag) and update the macro invocations. Verify against `hp41-core/src/ops/flags.rs` flag-range documentation during read_first.
- **Test name collision with `phase24_ind_variants.rs`:** the macro names (`sto_ind_fn_qual_04_happy`) include the `_fn_qual_04_` infix specifically to avoid collision. Cargo test discovery treats integration-test files as separate crates so technically collisions wouldn't compile-fail, but the unique infix keeps grep + readability clean.
- **ISG/DSE counter format misunderstood:** the format `ccccc.fffii` is HP-41-specific. CLAUDE.md mandates parse_counter splits on the decimal point. The canonical test in phase24_ind_variants.rs::isg_ind_inside_run_loop (lines 178–204) IS the reference — copy its counter literal verbatim (e.g. `Decimal::from_str("1.005")` for cur=1 / final=5 / inc=1) and adjust per branch.
- **A test discovers a real bug in resolve_indirect or run_loop:** failure is the point per RESEARCH Assumption A7. Escalate via the SUMMARY — do NOT silently widen the assertion.

## Out of Scope (explicit)
- Out-of-regs-len rejection tests → already in `phase24_ind_variants.rs::*_out_of_regs_len` (FN-QUAL-04 requires only happy + non-integer-reject per REQUIREMENTS.md wording)
- Sidecar / inheritance bonus tests → already in `phase24_ind_variants.rs`
- IND-flag PROPERTY → Plan 27-02 `proptest_flags.rs` (D-27.12)
- Coverage push, accuracy extension → Plan 27-01
- Proptest math → Plan 27-02
- Playwright / Vitest / CI → Plan 27-04
- `hp41-core/src/` edits (frozen)

## References
- 27-CONTEXT.md D-27.12 (IND-test layout — example tests here, properties in proptest_flags.rs)
- 27-RESEARCH.md §Code Example 3 (ind_happy_and_reject macro pattern)
- REQUIREMENTS.md line 106 (FN-QUAL-04 enumeration of 17 IND ops)
- ROADMAP.md SC-4 line 199 (the explicit `indirect_addressing.rs` filename)
- hp41-core/tests/phase24_ind_variants.rs (style precedent, name-collision avoidance reference)
- hp41-core/tests/phase21_flags.rs (FS?C / FC?C clear-flag-after-test pattern)
- CLAUDE.md v2.2 additions section (Phase 24 indirect_addressing design)
</failure_modes>
