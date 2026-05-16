---
phase: 21
plan: 02
type: execute
wave: 2
depends_on:
  - 21-01
files_modified:
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase21_flags.rs
autonomous: true
requirements:
  - FN-FLAG-02
tags:
  - hp41
  - rust
  - flags
  - run-loop
  - conditional-skip

must_haves:
  truths:
    - "A new `FlagTestKind` companion enum exists with 4 variants (IsSet, IsClear, IsSetThenClear, IsClearThenClear) mirroring the StoArithKind / TestKind precedent (FN-FLAG-02)"
    - "A new `Op::FlagTest { kind: FlagTestKind, flag: u8 }` variant exists, with the 4-place rule honored: enum + dispatch (interactive no-op) + run_loop arm + execute_op catch-all listing + BOTH prgm_display.rs copies"
    - "Inside `run_loop` (program execution), `Op::FlagTest { kind: IsSet, flag: n }` skips the NEXT step (state.pc += 1) when flag n is CLEAR; executes the next step when flag n is SET (mirror of Op::Test arm at program.rs:231-235)"
    - "Inside `run_loop`, `Op::FlagTest { kind: IsClear, flag: n }` skips next step when flag n is SET"
    - "Inside `run_loop`, `Op::FlagTest { kind: IsSetThenClear, flag: n }` ALWAYS clears flag n (regardless of test outcome) AND skips next step when flag n was CLEAR before the clear (RESEARCH A4 — strict reading of `FS?C`)"
    - "Inside `run_loop`, `Op::FlagTest { kind: IsClearThenClear, flag: n }` ALWAYS clears flag n AND skips next step when flag n was SET before the clear"
    - "Interactive dispatch (outside run_loop) for any Op::FlagTest is a no-op (mirrors Op::Test interactive behavior at program.rs:98-101) — apply_lift_effect(Neutral) + Ok(()); the PC is not advanced because there is no next program step at the keyboard"
    - "The end-to-end SC-1 scenario passes: program `[Lbl(\"T\"), SfFlag(5), FlagTest{IsSet,5}, PushNum(1), PushNum(2), Rtn]` produces final stack X=2, Y=1 (the FS?5 on a SET flag does NOT skip the `1` push)"
    - "The symmetric SC-1 scenario passes: program `[Lbl(\"T\"), CfFlag(5), FlagTest{IsSet,5}, PushNum(1), PushNum(2), Rtn]` produces final stack X=2 only (the FS?5 on a CLEAR flag SKIPS the `1` push)"
    - "SC-2 passes: a program with `FlagTest{IsSetThenClear, 10}` on a previously-SET flag 10 leaves flag 10 CLEAR after the test runs (verified via flag_get after the program finishes); a program with `FlagTest{IsSetThenClear, 10}` on a CLEAR flag leaves it CLEAR (idempotent — already-clear stays clear)"
    - "All 6 flag-test orderings (4 kinds × {set / clear} starting state) have explicit deterministic test coverage"
    - "Phase 11/12/20 test suites and Plan 21-01 tests stay green — no regression"

  artifacts:
    - path: "hp41-core/src/ops/mod.rs"
      provides: "New `FlagTestKind` enum (4 variants); new `Op::FlagTest { kind, flag }` struct variant; 1 new dispatch arm (interactive no-op delegating to apply_lift_effect + Ok)"
      contains: "pub enum FlagTestKind"
    - path: "hp41-core/src/ops/program.rs"
      provides: "1 new `run_loop` arm at line ~246 (BEFORE the `other =>` catch-all at line 246) implementing the 4-kind match with always-clear side effect for IsSetThenClear/IsClearThenClear; `Op::FlagTest { .. }` added to the catch-all `Op::Lbl(_) | ... => Err(InvalidOp)` block at lines 410-417 so execute_op rejects FlagTest variants (they belong only in run_loop)"
      contains: "Op::FlagTest { kind, flag }"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "1 new op_display_name arm: `Op::FlagTest { kind, flag }` formatted as `\"FS? 05\"` / `\"FC? 05\"` / `\"FS?C 05\"` / `\"FC?C 05\"` per kind"
      contains: "Op::FlagTest"
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same op_display_name arm (SC-4 spirit exception)"
      contains: "Op::FlagTest"
    - path: "hp41-core/tests/phase21_flags.rs"
      provides: "Extended with FN-FLAG-02 integration tests covering the 4 conditional tests × 2 starting states (8 baseline tests) + the side-effect tests for FS?C / FC?C (always-clear semantic) + the interactive-no-op test"
      contains: "fn test_fs_q_in_program_skips_when_flag_clear"
      min_tests: 10

  key_links:
    - from: "hp41-core/src/ops/mod.rs::dispatch"
      to: "Op::FlagTest interactive arm (no-op + Neutral lift)"
      via: "Mirrors the Op::Test no-op interactive pattern (RESEARCH Pitfall 1 — keyboard FS? does not skip because there is no next program step at the keyboard)"
      pattern: "Op::FlagTest \\{ .. \\} =>"
    - from: "hp41-core/src/ops/program.rs::run_loop"
      to: "ops::flags::{flag_get, flag_clear}"
      via: "The 4-kind match: read flag with flag_get; for kinds IsSetThenClear / IsClearThenClear write back via flag_clear BEFORE deciding the skip; advance state.pc += 1 on skip"
      pattern: "flag_get\\(state\\.flags, flag\\)"
    - from: "hp41-core/src/ops/program.rs::execute_op"
      to: "Catch-all `Op::Lbl(_) | ... => Err(HpError::InvalidOp)`"
      via: "Add `| Op::FlagTest { .. }` to the catch-all chain at lines 410-417 — FlagTest never reaches execute_op (run_loop handles it directly)"
      pattern: "\\| Op::FlagTest \\{ \\.\\. \\}"
---

<objective>
Land the conditional flag-test family for Phase 21 — the four ops `FS?` / `FC?` / `FS?C` / `FC?C` — as a single struct-variant `Op::FlagTest { kind: FlagTestKind, flag: u8 }`. Inside `run_loop`, this variant performs the HP-41 "skip next step on false" semantic plus the always-clear side effect for the `?C` variants. Interactive dispatch is a no-op (matches `Op::Test` precedent: keyboard FS? does not skip because there is no next program step at the keyboard).

This plan depends on Plan 21-01: the `flags: u64` field, the `flag_get` / `flag_clear` helpers, AND the `justfile::test-core *args:` recipe must already exist before this plan can compile / be verified.

Purpose: Restore HP-41CV flag-conditional control flow inside running programs. Combined with the SF/CF set/clear ops from 21-01, this satisfies FN-FLAG-02 and is the foundation for indirect-addressed flag operations in Phase 24.

Output: 5 modified files. Net new line count ≈ 150 LOC including tests. All CI gates stay green; coverage stays ≥ 92.5%.
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
@.planning/phases/21-flags-display-control-sound/21-RESEARCH.md
@.planning/phases/21-flags-display-control-sound/21-PATTERNS.md
@.planning/phases/21-flags-display-control-sound/21-VALIDATION.md
@.planning/phases/21-flags-display-control-sound/21-01-flags-core-PLAN.md
@CLAUDE.md

# Source files the executor will modify or extend
@hp41-core/src/ops/mod.rs
@hp41-core/src/ops/program.rs
@hp41-core/src/ops/flags.rs
@hp41-cli/src/prgm_display.rs
@hp41-gui/src-tauri/src/prgm_display.rs
@hp41-core/tests/phase21_flags.rs

<interfaces>
<!-- Key types and contracts the executor needs. Extracted from the codebase on 2026-05-14 by direct Read. -->

From Plan 21-01 (must be merged first):
- `pub flags: u64` field on `CalcState` (state.rs)
- `pub fn flag_get(flags: u64, n: u8) -> bool` (flags.rs)
- `pub fn flag_set(flags: u64, n: u8) -> u64` (flags.rs)
- `pub fn flag_clear(flags: u64, n: u8) -> u64` (flags.rs)
- `Op::SfFlag(u8)`, `Op::CfFlag(u8)` and their 4-place landings
- `justfile::test-core *args:` recipe (Wave-0 from Plan 21-01 Task 0) — forwards to `cargo test -p hp41-core {{args}}`

From `hp41-core/src/ops/mod.rs` (lines 36-70):
- `pub enum StoArithKind { Add, Sub, Mul, Div }` at 36-42 — model for the new `FlagTestKind`.
- `pub enum StackReg { Y, Z, T, LastX }` at 44-51 — model.
- `pub enum TestKind { XEqZero, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero, XEqY, XNeY, XLtY, XGtY, XLeY, XGeY }` at 56-70 — companion-enum model; uses the same `Debug, Clone, PartialEq, Serialize, Deserialize` derive chain.
- `Op::StoArith { reg: u8, kind: StoArithKind }` (struct-variant precedent for `Op::FlagTest { kind, flag }`).
- `Op::Test(TestKind)` (a tuple-variant of an enum that already lives in `run_loop` and dispatch — Op::FlagTest mirrors this precedent for the dispatch-side, but uses a struct-variant for clarity with two fields.)

From `hp41-core/src/ops/program.rs` (lines 177-254 — run_loop):
- The `match op { ... }` block inside `run_loop` lives at lines 191-251.
- `Op::Test(kind)` arm at 231-235:
  - `Op::Test(kind) => { if !evaluate_test(state, &kind) { state.pc += 1; /* skip next step */ } }`
- `Op::Isg(reg)` / `Op::Dse(reg)` arms at 236-244 — bool-returning helper variant.
- The `other =>` catch-all at line 246 forwards to `execute_op(state, other)?`. Phase 21 inserts the new arms BEFORE this catch-all so they bypass `execute_op`.
- The catch-all "programming-only" block in `execute_op` at lines 410-417: `Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_) | Op::Isg(_) | Op::Dse(_) => Err(HpError::InvalidOp)`. Phase 21-02 adds `| Op::FlagTest { .. }` here so execute_op never silently swallows a FlagTest variant (Pitfall 2 from RESEARCH).

From `hp41-core/src/ops/program.rs::op_test` at lines 98-101:
- `pub fn op_test(_state: &mut CalcState, _kind: TestKind) -> Result<(), HpError>` is a NO-OP at the keyboard. The actual skip logic lives in `run_loop`'s `Op::Test(kind)` arm. Phase 21 mirrors this exactly: interactive dispatch for `Op::FlagTest { .. }` is `apply_lift_effect(state, LiftEffect::Neutral); Ok(())`.

From `hp41-core/src/ops/flags.rs` (lines from Plan 21-01):
- `pub fn flag_get(flags: u64, n: u8) -> bool` — read helper
- `pub fn flag_clear(flags: u64, n: u8) -> u64` — clear helper (returns new flags word)

From `hp41-cli/src/prgm_display.rs` and the GUI copy:
- The `Op::StoArith { reg, kind }` arm at CLI lines 82-90 is the struct-variant display precedent. The new `Op::FlagTest { kind, flag }` arm follows the same `match kind` + `format!(...)` shape.

From `hp41-core/tests/phase21_flags.rs` (created in Plan 21-01):
- File-level `#![allow(clippy::unwrap_used)]`
- `use hp41_core::ops::{dispatch, flags::{flag_get, flag_set, flag_clear}, Op};`
- `use hp41_core::{CalcState, HpError};`
- Plan 21-02 extends this file with the FN-FLAG-02 tests (no new file).

From `hp41-core/src/ops/program.rs::run_program` (lines 130-169):
- `pub fn run_program(state: &mut CalcState, label: &str) -> Result<(), HpError>` — invocation entry point. Tests construct a `state.program: Vec<Op>` with a leading `Op::Lbl(label)` and call `run_program(&mut state, label)`. This is the precedent used by `tests/program_tests.rs` and Phase 20 tests.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1 (Wave-2 RED): Extend hp41-core/tests/phase21_flags.rs with FN-FLAG-02 conditional-skip tests (RED tests written FIRST)</name>
  <files>hp41-core/tests/phase21_flags.rs</files>
  <read_first>
    - hp41-core/tests/phase21_flags.rs (the existing file from Plan 21-01 with the 9 FN-FLAG-01 tests; new tests append to this file)
    - hp41-core/tests/program_tests.rs (precedent for run_program-based integration tests — note how `state.program = vec![Op::Lbl("T".to_string()), ...]` is set up and `hp41_core::ops::program::run_program(&mut state, "T").unwrap()` is called)
    - hp41-core/src/ops/program.rs lines 130-260 (run_program + run_loop entry points)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §"Phase Requirements → Test Map" lines 506-512 (the FN-FLAG-02 test list)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/tests/phase21_flags.rs" lines 552-598 (test pattern with `Op::PushNum(HpNum::from(Decimal::from_str("1").unwrap()))`)
  </read_first>
  <behavior>
    Behavior expectations (these tests are RED on entry — Tasks 2/3/4/5 of this plan make them GREEN):

    - Test 10 `test_fs_q_in_program_executes_next_when_flag_set` — program `[Lbl("T"), SfFlag(5), FlagTest{IsSet,5}, PushNum(1), PushNum(2), Rtn]` → final X=2, Y=1 (FS?5 on a SET flag does NOT skip; both pushes execute).
    - Test 11 `test_fs_q_in_program_skips_next_when_flag_clear` — program `[Lbl("T"), CfFlag(5), FlagTest{IsSet,5}, PushNum(1), PushNum(2), Rtn]` → final X=2, Y=0 (FS?5 on a CLEAR flag SKIPS the `1` push; only `2` reaches the stack).
    - Test 12 `test_fc_q_in_program_skips_when_flag_set` — program `[Lbl("T"), SfFlag(5), FlagTest{IsClear,5}, PushNum(1), PushNum(2), Rtn]` → final X=2, Y=0 (FC?5 skip-on-NOT-clear).
    - Test 13 `test_fc_q_in_program_executes_when_flag_clear` — program `[Lbl("T"), CfFlag(5), FlagTest{IsClear,5}, PushNum(1), PushNum(2), Rtn]` → final X=2, Y=1.
    - Test 14 `test_fs_q_c_clears_flag_after_test` — program `[Lbl("T"), SfFlag(10), FlagTest{IsSetThenClear,10}, Rtn]` → after run, `flag_get(state.flags, 10) == false`. The IsSetThenClear variant cleared the flag as its side effect, regardless of the skip outcome (RESEARCH A4 — strict reading).
    - Test 15 `test_fs_q_c_on_clear_flag_idempotent` — program `[Lbl("T"), CfFlag(10), FlagTest{IsSetThenClear,10}, Rtn]` → after run, `flag_get(state.flags, 10) == false` (already-clear stays clear; the always-clear side effect is idempotent for an already-clear flag).
    - Test 16 `test_fc_q_c_clears_flag_after_test` — program `[Lbl("T"), SfFlag(10), FlagTest{IsClearThenClear,10}, Rtn]` → flag 10 is cleared after.
    - Test 17 `test_fs_q_c_skip_branch` — `[Lbl("T"), CfFlag(5), FlagTest{IsSetThenClear,5}, PushNum(99), PushNum(7), Rtn]` → final X=7 only (FS?C on a CLEAR flag: skip outcome = TRUE because !is_set, so the `99` push is skipped). flag_get(state.flags, 5) is still false.
    - Test 18 `test_fc_q_c_skip_branch` — `[Lbl("T"), SfFlag(5), FlagTest{IsClearThenClear,5}, PushNum(99), PushNum(7), Rtn]` → final X=7 only (FC?C on a SET flag: skip outcome = TRUE because is_set, so the `99` push is skipped). flag_get(state.flags, 5) is now false (side effect cleared it).
    - Test 19 `test_op_flag_test_interactive_dispatch_is_no_op` — `let mut s = CalcState::new(); s.flags = 0; let initial_pc = s.pc; dispatch(&mut s, Op::FlagTest { kind: FlagTestKind::IsSet, flag: 5 }).unwrap(); assert_eq!(s.pc, initial_pc); assert_eq!(s.flags, 0); /* no skip, no flag mutation, no stack mutation */`.

    All 10 tests append to `hp41-core/tests/phase21_flags.rs` (the file from Plan 21-01). They depend on `Op::FlagTest`, `FlagTestKind`, and the run_loop arm — all of which are added in Tasks 2/3 of this plan.
  </behavior>
  <action>
    Append the 10 new tests to `hp41-core/tests/phase21_flags.rs`. Extend the existing `use` statements at the top with `FlagTestKind`:
    - Change the existing line `use hp41_core::ops::{dispatch, flags::{flag_get, flag_set, flag_clear}, Op};` to `use hp41_core::ops::{dispatch, flags::{flag_get, flag_set, flag_clear}, FlagTestKind, Op};`
    - Add `use hp41_core::ops::program::run_program;` (or use the fully qualified path `hp41_core::ops::program::run_program` inline).
    - Add `use hp41_core::HpNum;` for the `Op::PushNum(HpNum::from(...))` calls. Also add `use rust_decimal::Decimal; use std::str::FromStr;` if numeric literal pushes use Decimal::from_str (cleaner for arbitrary values than `HpNum::from(i32)`).

    For each test, set up the program via `state.program = vec![Op::Lbl("T".to_string()), ..., Op::Rtn]` and call `run_program(&mut state, "T").unwrap();`. Then assert the final stack via `state.stack.x.inner()` comparisons (e.g., `assert_eq!(state.stack.x.inner(), Decimal::from(2));`).

    The `Op::PushNum(HpNum::from(1i32))` form is the cleanest for integer pushes; `Op::PushNum(HpNum::from(Decimal::from_str("1.5").unwrap()))` for non-integers. The Phase 20 test file `hp41-core/tests/phase20_math.rs` is the precedent — copy its `push_x` helper if it produces a shorter test body.

    Set `state.flags = u64::MAX` only as a precondition for the symmetric tests where the analog requires "all-but-bit-N" coverage. Most tests start from `let mut state = CalcState::new();` and use `SfFlag(n)` / `CfFlag(n)` to establish the pre-test flag state — that exercises the SF/CF dispatch from Plan 21-01 as a bonus.

    Verify the file head still has `#![allow(clippy::unwrap_used)]` (carried over from Plan 21-01).

    Do NOT delete or modify any of the 9 tests from Plan 21-01 — append only.

    These tests will be RED (fail to compile, then fail at runtime) until Tasks 2-5 of this plan land. Run `just test-core --test phase21_flags` to confirm they are currently RED: `cargo` will fail with "no variant named FlagTest" / "no enum named FlagTestKind". That is expected.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -c '^#\[test\]' hp41-core/tests/phase21_flags.rs` returns ≥ 19 (9 from Plan 21-01 + 10 new in this task).
    - Source assertion: file contains all 10 specific new test names — verify each via `grep -c "fn test_fs_q_in_program_executes_next_when_flag_set\|fn test_fs_q_in_program_skips_next_when_flag_clear\|fn test_fc_q_in_program_skips_when_flag_set\|fn test_fc_q_in_program_executes_when_flag_clear\|fn test_fs_q_c_clears_flag_after_test\|fn test_fs_q_c_on_clear_flag_idempotent\|fn test_fc_q_c_clears_flag_after_test\|fn test_fs_q_c_skip_branch\|fn test_fc_q_c_skip_branch\|fn test_op_flag_test_interactive_dispatch_is_no_op" hp41-core/tests/phase21_flags.rs` returns ≥ 10.
    - Source assertion: `grep -c 'FlagTestKind' hp41-core/tests/phase21_flags.rs` returns ≥ 10 (every conditional-skip test references the kind enum).
    - Source assertion: `grep -c 'run_program' hp41-core/tests/phase21_flags.rs` returns ≥ 8 (every program-mode test invokes the entry point).
    - Test command (this task's RED check — STRENGTHENED per W-4): the compile failure MUST mention the missing FlagTest symbols. Run `just test-core --test phase21_flags 2>&1 | tee /tmp/p21t1.log` (this captures both stdout and stderr). Then assert: `grep -qE 'no variant named .FlagTest|cannot find type .FlagTestKind|no associated item' /tmp/p21t1.log` returns 0 (match found). This proves the RED state is caused by the expected missing symbols, NOT by test logic errors or unrelated compile failures. The earlier grep-only check counted lines without verifying they came from cargo's diagnostic stream — this stricter check actually inspects the compiler output.
    - The 9 existing tests from Plan 21-01 are NOT modified or deleted (verify by `grep -c 'fn test_flags_field_defaults_to_zero\|fn test_load_v20_save_no_flags_field\|fn test_serde_round_trip_with_flags_set\|fn test_flag_get_set_clear_helpers_unit\|fn test_flag_helpers_out_of_range_defensive\|fn test_op_sf_sets_bit\|fn test_op_cf_clears_bit\|fn test_op_sf_out_of_range_returns_invalid_op\|fn test_op_cf_out_of_range_returns_invalid_op' hp41-core/tests/phase21_flags.rs` returns ≥ 9).
  </acceptance_criteria>
  <verify>
    <automated>grep -c '^#\[test\]' hp41-core/tests/phase21_flags.rs | awk '{ exit $1 &lt; 19 }' &amp;&amp; (just test-core --test phase21_flags 2>&amp;1 | tee /tmp/p21t1.log; grep -qE 'no variant named .FlagTest|cannot find type .FlagTestKind|no associated item' /tmp/p21t1.log)</automated>
  </verify>
  <done>10 new tests appended to hp41-core/tests/phase21_flags.rs; they reference `FlagTestKind` and `Op::FlagTest`; they currently fail to compile (RED) with diagnostics specifically about the missing FlagTest symbols (not unrelated errors) — Tasks 2-5 below produce them. The 9 tests from Plan 21-01 are untouched.</done>
</task>

<task type="auto">
  <name>Task 2 (Wave-2): Add FlagTestKind enum + Op::FlagTest variant + dispatch interactive no-op arm in ops/mod.rs</name>
  <files>hp41-core/src/ops/mod.rs</files>
  <read_first>
    - hp41-core/src/ops/mod.rs lines 36-70 (the StoArithKind, StackReg, TestKind sub-enums — the analog templates for FlagTestKind)
    - hp41-core/src/ops/mod.rs lines 80-300 (Op enum — find the Phase 21 section added in Plan 21-01 and insert the new variant alongside SfFlag/CfFlag)
    - hp41-core/src/ops/mod.rs lines 345-560 (the dispatch function — the new interactive arm goes near the SfFlag/CfFlag arms added in Plan 21-01)
    - hp41-core/src/ops/program.rs lines 98-101 (op_test no-op precedent for the interactive arm)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/src/ops/mod.rs" lines 280-393 (sub-enum + struct-variant + dispatch arm walkthrough)
    - CLAUDE.md §Critical Implementation Traps (the 4-place rule sentinel)
  </read_first>
  <action>
    **Step 1 — add `FlagTestKind` sub-enum** in `hp41-core/src/ops/mod.rs`, placed AFTER `TestKind` (around line 70) and BEFORE `pub enum Op`. Use the same `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` derive chain. The doc comment must read: `/// HP-41 flag-test kind — 4 total. Used in Op::FlagTest { kind, flag }. Mirrors the TestKind / StoArithKind sub-enum precedent. Phase 21 (FN-FLAG-02).`

    Variants (with brief doc comments inside the enum):
    - `IsSet` — doc: `/// FS? — skip next step when flag is NOT set (i.e. condition is "is set"; skip-if-false semantic).`
    - `IsClear` — doc: `/// FC? — skip next step when flag is NOT clear.`
    - `IsSetThenClear` — doc: `/// FS?C — skip if NOT set; ALWAYS clear the flag afterward (RESEARCH A4 — strict reading).`
    - `IsClearThenClear` — doc: `/// FC?C — skip if NOT clear; ALWAYS clear the flag afterward.`

    **Step 2 — add `Op::FlagTest { kind: FlagTestKind, flag: u8 }`** to the `pub enum Op` block, in the SAME Phase 21 section as `SfFlag(u8)` and `CfFlag(u8)` from Plan 21-01 (right after them). Doc: `/// FS? / FC? / FS?C / FC?C n — conditional flag test (run_loop skips next step on false; ?C variants also always-clear). LiftEffect: Neutral. Phase 21 (FN-FLAG-02).`

    **Step 3 — add 1 new dispatch arm** in `pub fn dispatch(...)` next to the SfFlag/CfFlag arms from Plan 21-01. The arm is an interactive no-op (mirrors `op_test` at program.rs:98-101 — the actual skip logic lives in `run_loop`, added in Task 3 below):
    - `Op::FlagTest { .. } => { apply_lift_effect(state, LiftEffect::Neutral); Ok(()) }`

    Use the `{ .. }` field-pattern to make clear that neither `kind` nor `flag` is read at the keyboard. (This is the same shape as the Op::Test interactive case which forwards to `program::op_test`. A direct inline body is preferred here since the no-op does not warrant a dedicated `pub fn`.)

    Do NOT touch the catch-all "programming-only" block at the end of dispatch — `Op::FlagTest` is handled by its own arm above, not by the catch-all.

    Do NOT touch `flush_entry_buf`, the PRGM-mode gate, or any other Op variant.

    After this task, `just build` will FAIL because execute_op in program.rs is now non-exhaustive (Op::FlagTest is unhandled). Task 3 fixes that. Compile-only intermediate state is allowed across Tasks 2-4 of this plan.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -n "pub enum FlagTestKind" hp41-core/src/ops/mod.rs` returns exactly 1 line.
    - Source assertion: `grep -cE "^\s+(IsSet|IsClear|IsSetThenClear|IsClearThenClear),?" hp41-core/src/ops/mod.rs` returns 4 (the 4 FlagTestKind variants).
    - Source assertion: `grep -n "FlagTest \{ kind: FlagTestKind, flag: u8 \}" hp41-core/src/ops/mod.rs` returns ≥ 1 line (the Op variant).
    - Source assertion: `grep -E "Op::FlagTest \{ \.\. \} =>" hp41-core/src/ops/mod.rs` returns ≥ 1 line (the interactive no-op dispatch arm).
    - Test command: `just build 2>&1 | grep -c 'non-exhaustive patterns'` returns ≥ 1 — this is EXPECTED (execute_op in program.rs has not been extended yet; Task 3 fixes it). DO NOT commit this state alone — Tasks 2-5 should all land before the final commit.
  </acceptance_criteria>
  <verify>
    <automated>grep -n "pub enum FlagTestKind" hp41-core/src/ops/mod.rs &amp;&amp; grep -n "Op::FlagTest \{ \.\. \} =>" hp41-core/src/ops/mod.rs</automated>
  </verify>
  <done>FlagTestKind enum (4 variants) exists in ops/mod.rs; Op::FlagTest { kind, flag } variant exists; 1 new interactive dispatch arm exists with no-op body (`apply_lift_effect(Neutral) + Ok`). The crate may have a non-exhaustive-match compile error on execute_op until Task 3 — expected and resolved in Task 3.</done>
</task>

<task type="auto">
  <name>Task 3 (Wave-2): Add run_loop arm + execute_op catch-all listing for Op::FlagTest in ops/program.rs</name>
  <files>hp41-core/src/ops/program.rs</files>
  <read_first>
    - hp41-core/src/ops/program.rs lines 177-254 (the entire run_loop function; the existing arms for Rtn / Lbl / Gto / Xeq / Test / Isg / Dse / catch-all `other`)
    - hp41-core/src/ops/program.rs lines 405-418 (the execute_op catch-all "programming-only" block listing Op::Lbl/Gto/Xeq/Rtn/PrgmMode/Test/Isg/Dse)
    - hp41-core/src/ops/flags.rs (the `flag_get` and `flag_clear` helpers — both used by the new run_loop arm)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/src/ops/program.rs" lines 396-492 (the exact walkthrough including the always-clear side-effect logic)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §Code Example 2 lines 344-376 (the FlagTestKind match with always-clear behavior, RESEARCH A4 strict reading)
  </read_first>
  <action>
    **Step 1 — add 1 new arm to run_loop** in `hp41-core/src/ops/program.rs`, INSIDE the `match op { ... }` block (lines 191-251), placed BEFORE the `other =>` catch-all at line 246 and AFTER the existing `Op::Dse(reg)` arm at lines 241-244. Add a section comment `// ── Phase 21: Flag tests (skip next step pattern, mirrors Op::Test) ──`.

    The arm has shape `Op::FlagTest { kind, flag } => { ... }`. Inside the block:
    1. Import the helpers at the top of the arm body via `use crate::ops::flags::{flag_clear, flag_get};` OR add the import to the top of the function. Pick whichever produces the smaller diff. The PATTERNS.md walkthrough uses the inline `use` form (line 431) which is fine.
    2. Read the current flag state: `let is_set = flag_get(state.flags, flag);`
    3. Compute `should_skip` via match on `kind`:
       - `FlagTestKind::IsSet => !is_set` — FS?: skip if NOT set
       - `FlagTestKind::IsClear => is_set` — FC?: skip if NOT clear
       - `FlagTestKind::IsSetThenClear => { state.flags = flag_clear(state.flags, flag); !is_set }` — FS?C: ALWAYS clear, skip if was-clear
       - `FlagTestKind::IsClearThenClear => { state.flags = flag_clear(state.flags, flag); is_set }` — FC?C: ALWAYS clear, skip if was-set
    4. If `should_skip` is true: `state.pc += 1;` (this is the HP-41 skip-next-step semantic, identical to the `Op::Test(kind)` arm at lines 231-235).

    The always-clear side effect for IsSetThenClear / IsClearThenClear matches the strict reading of HP-41 documentation per RESEARCH A4. Both branches clear `state.flags` regardless of the prior bit value — for an already-clear flag this is idempotent (clearing a clear bit is a no-op on the bit), so SC-2 ("FS?C 10 on a clear flag leaves it clear") is satisfied.

    Do NOT bind the inner variable names with a different identifier from `kind` / `flag` — they are documented in mod.rs and PATTERNS.md.

    **Step 2 — add `Op::FlagTest { .. }` to the execute_op catch-all** at lines 410-417. Extend the chain from:
    `Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_) | Op::Isg(_) | Op::Dse(_) => Err(HpError::InvalidOp),`
    To:
    `Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_) | Op::Isg(_) | Op::Dse(_) | Op::FlagTest { .. } => Err(HpError::InvalidOp),`

    This honors Pitfall 2 from RESEARCH.md: `Op::FlagTest` is handled by `run_loop` directly (Step 1 above), NEVER by `execute_op`. The catch-all ensures execute_op explicitly errors if it ever sees a FlagTest variant (defense-in-depth against a future bug where the new arm in run_loop is removed by accident).

    Do NOT add an `Op::FlagTest =>` arm anywhere ELSE in execute_op — that would create dead code.

    After this task, `just build` MUST exit 0 again. Verify by running it. If non-exhaustive errors remain, the catch-all extension was missed.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -E "Op::FlagTest \{ kind, flag \} =>" hp41-core/src/ops/program.rs` returns ≥ 1 line (the run_loop arm).
    - Source assertion: `grep -n "IsSetThenClear =>" hp41-core/src/ops/program.rs` returns ≥ 1 line (the always-clear side-effect branch).
    - Source assertion (always-clear semantic): `grep -B2 -A4 "IsSetThenClear" hp41-core/src/ops/program.rs | grep -c "flag_clear(state.flags, flag)"` returns ≥ 1 (the clear happens inside the IsSetThenClear arm, BEFORE the `!is_set` skip-condition expression).
    - Source assertion (catch-all extension): `grep -E "Op::FlagTest \{ \.\. \}" hp41-core/src/ops/program.rs | grep -c "Err(HpError::InvalidOp)"` returns ≥ 1 — FlagTest is in the execute_op catch-all.
    - Test command: `just build` exits 0 — the exhaustive-match gate is satisfied.
    - Test command: `just test-core --test program_tests` exits 0 — existing programming-engine tests still pass.
    - Test command: `just test-core --test phase21_flags` exits 0 — ALL 19 tests (9 from Plan 21-01 + 10 from Task 1 of this plan) pass GREEN. The 10 new conditional-skip tests now succeed because the run_loop arm implements them.
    - Lint command: `just lint` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>just build &amp;&amp; just lint &amp;&amp; just test-core --test program_tests &amp;&amp; just test-core --test phase21_flags</automated>
  </verify>
  <done>1 new run_loop arm exists in hp41-core/src/ops/program.rs implementing the 4-kind skip logic + always-clear side effect for ?C variants; Op::FlagTest { .. } is appended to the execute_op catch-all; the 10 new RED tests from Task 1 now turn GREEN; cargo build green; clippy green; no regression in existing test suites.</done>
</task>

<task type="auto">
  <name>Task 4 (Wave-2): Add 1 op_display_name arm to BOTH prgm_display.rs copies for Op::FlagTest (4-place rule, places 3 + 4)</name>
  <files>hp41-cli/src/prgm_display.rs, hp41-gui/src-tauri/src/prgm_display.rs</files>
  <read_first>
    - hp41-cli/src/prgm_display.rs (entire file — the Phase 21 SfFlag/CfFlag arms from Plan 21-01 and the StoArith struct-variant arm at lines 82-90 as the format-shape precedent)
    - hp41-gui/src-tauri/src/prgm_display.rs (entire file — must mirror the CLI arm byte-identically)
    - hp41-core/src/ops/mod.rs (the new FlagTestKind enum and Op::FlagTest variant added in Task 2 — they must be in scope in prgm_display.rs)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-cli/src/prgm_display.rs" lines 496-535 (the exact arm shape with the inner `match kind` for the mnemonic)
  </read_first>
  <action>
    Extend the `use` statement in BOTH files to bring `FlagTestKind` into scope:
    - In `hp41-cli/src/prgm_display.rs` line 7, change `use hp41_core::ops::{Op, StackReg, StoArithKind};` to `use hp41_core::ops::{FlagTestKind, Op, StackReg, StoArithKind};`.
    - In `hp41-gui/src-tauri/src/prgm_display.rs`, find the equivalent `use hp41_core::ops::...` line and add `FlagTestKind` similarly.

    Add 1 new arm to the `match op { ... }` block inside `fn op_display_name(op: &Op) -> String` in BOTH files. Place it after the `Op::SfFlag(n)` and `Op::CfFlag(n)` arms from Plan 21-01 (the Phase 21 section).

    The arm has shape `Op::FlagTest { kind, flag } => { let mnemonic = match kind { ... }; format!("{mnemonic} {flag:02}") }`. The inner `match kind` returns:
    - `FlagTestKind::IsSet => "FS?"`
    - `FlagTestKind::IsClear => "FC?"`
    - `FlagTestKind::IsSetThenClear => "FS?C"`
    - `FlagTestKind::IsClearThenClear => "FC?C"`

    The output for `Op::FlagTest { kind: IsSet, flag: 5 }` is `"FS? 05"`. For `kind: IsSetThenClear, flag: 12` it is `"FS?C 12"`. The `{flag:02}` width matches the existing `StoReg(r) => format!("STO {r:02}")` arm.

    Both files must produce byte-identical arms.

    **SC-4 invariant gate (reminder):** The only addition to `hp41-gui/src-tauri/src/prgm_display.rs` in this plan is this 1 new arm + the `FlagTestKind` import. Do NOT add any new `fn op_*`, `fn flush_entry_*`, or `fn format_hpnum` body anywhere in `hp41-gui/src-tauri/src/`. Verify with the stricter grep documented in CLAUDE.md.

    Optionally extend the existing `#[cfg(test)] mod tests` in each file with a small test `test_display_phase21_flag_test_labels` asserting:
    - `op_display_name(&Op::FlagTest { kind: FlagTestKind::IsSet, flag: 5 }) == "FS? 05"`
    - `op_display_name(&Op::FlagTest { kind: FlagTestKind::IsSetThenClear, flag: 10 }) == "FS?C 10"`
    - `op_display_name(&Op::FlagTest { kind: FlagTestKind::IsClearThenClear, flag: 0 }) == "FC?C 00"`

    Keep the test under 10 LOC per file.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -cE "Op::FlagTest \{ kind, flag \} =>" hp41-cli/src/prgm_display.rs` returns 1.
    - Source assertion: `grep -cE "Op::FlagTest \{ kind, flag \} =>" hp41-gui/src-tauri/src/prgm_display.rs` returns 1.
    - Source assertion: `grep -c 'FlagTestKind::IsSetThenClear => "FS?C"' hp41-cli/src/prgm_display.rs` returns 1; symmetric for the GUI copy.
    - Behavior assertion (byte-identical arm): `diff <(grep -A8 "Op::FlagTest \{ kind, flag \}" hp41-cli/src/prgm_display.rs) <(grep -A8 "Op::FlagTest \{ kind, flag \}" hp41-gui/src-tauri/src/prgm_display.rs)` returns empty output.
    - SC-4 invariant assertion: `grep -rnE 'fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches.
    - Test command: `just build` exits 0 (hp41-cli compiles as part of the workspace build).
    - Test command: `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0 (hp41-gui is a nested standalone workspace — direct cargo call required).
    - Test command: `cargo test -p hp41-cli` exits 0.
    - Test command: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0.
    - Test command: `just ci` exits 0.
    - Test command: `just gui-ci` exits 0.
    - Coverage assertion (non-regression): `cargo llvm-cov clean --workspace && cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>just build &amp;&amp; cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml &amp;&amp; just ci &amp;&amp; just gui-ci</automated>
  </verify>
  <done>1 new op_display_name arm exists in BOTH prgm_display.rs copies (byte-identical); FlagTestKind imported; SC-4 invariant grep returns nothing; both hp41-cli and hp41-gui/src-tauri compile and test green; just ci AND just gui-ci both green; hp41-core coverage ≥ 92.5%.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

Plan 21-02 extends `hp41-core` with conditional-skip semantics. No new I/O surface; same boundary set as Plan 21-01.

| Boundary | Description |
|----------|-------------|
| Programmer-supplied `flag: u8` index in `Op::FlagTest { flag, .. }` | Bounded numeric input. The `flag_get` and `flag_clear` helpers are defensive — out-of-range returns false / unchanged-flags. No additional op-layer guard is added in run_loop because run_loop receives the variant from the program Vec; the program is constructed by the user via SF/CF prompts (Phase 25 keyboard wiring, not in Phase 21 scope), and any flag > 55 silently becomes a no-skip + flags unchanged. |
| `run_loop` instruction iteration | MAX_STEPS guard (1,000,000) at program.rs:175-181 caps any infinite loop; FlagTest does not bypass this. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-W2-01 | Tampering | `Op::FlagTest { flag }` out-of-range value in a deserialized program | mitigate | The `flag_get` / `flag_clear` helpers defensively no-op for n > 55 (Plan 21-01). The run_loop arm reads `flag_get(state.flags, flag)` which returns false for out-of-range — should_skip becomes false (or true for IsClear/IsClearThenClear, depending on kind) but no panic, no state corruption. |
| T-21-W2-02 | DoS | Infinite-loop program using FlagTest to never advance | mitigate | MAX_STEPS (1,000,000) guard in run_loop applies — FlagTest only mutates flags + pc; the per-iteration step increment ensures eventual termination. |
| T-21-W2-03 | Tampering | execute_op silently accepts a FlagTest variant (Pitfall 2) | mitigate | Op::FlagTest { .. } is added to the execute_op catch-all chain at lines 410-417 — explicit InvalidOp return; never silently dispatched outside run_loop. |
| T-21-W2-04 | Information Disclosure | None | n/a | Pure flag arithmetic; no PII, no file I/O. |
| T-21-W2-05 | Spoofing / Repudiation / EoP | None | n/a | hp41-core single-user, no auth surface. |
</threat_model>

<verification>
## Plan-Level Verification

| Gate | Check |
|------|-------|
| FN-FLAG-02 (4 conditional tests work in run_loop) | `just test-core --test phase21_flags` exits 0 — all 19 tests pass (9 from Plan 21-01 + 10 new) |
| 4-place rule (place 1 — enum + dispatch) | `just build` exits 0 |
| 4-place rule (place 2 — execute_op catch-all + run_loop) | covered by the same build |
| 4-place rule (places 3 + 4 — both prgm_display.rs) | `just build` AND `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` both exit 0 |
| Zero-panic gate | `just lint` exits 0 |
| SC-4 invariant | `grep -rnE 'fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 |
| Always-clear side effect for ?C variants | Tests 14 (FS?C clears flag after SET) + 15 (FS?C on clear flag stays clear) + 16 (FC?C clears flag after SET) pass — these are dedicated assertions in phase21_flags.rs |
| Interactive dispatch is no-op | Test 19 (`test_op_flag_test_interactive_dispatch_is_no_op`) passes — pc unchanged, flags unchanged, stack unchanged |
| Coverage non-regression | `cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0 |
| Full CI gate | `just ci` exits 0 |
| GUI CI gate | `just gui-ci` exits 0 |

## Cross-Cutting Constraints

- All 4 FlagTestKind variants have explicit deterministic test coverage (Tests 10-18 in Task 1 — 8 baseline + 2 always-clear edge cases).
- `Op::FlagTest` LiftEffect is Neutral (the interactive dispatch arm explicitly calls `apply_lift_effect(state, LiftEffect::Neutral)`; the run_loop arm does not alter the stack at all, so the lift_enabled flag is preserved from the prior op).
- The always-clear side effect for IsSetThenClear / IsClearThenClear matches RESEARCH A4 strict reading — verified by Test 14 (FS?C on a SET flag clears it).
- No `println!` / `eprintln!` introduced in hp41-core.
- No new HpError variant.
</verification>

<success_criteria>
Plan 21-02 is complete when ALL of the following are true:

1. **`FlagTestKind` enum exists** in `hp41-core/src/ops/mod.rs` with 4 variants (IsSet, IsClear, IsSetThenClear, IsClearThenClear).
2. **`Op::FlagTest { kind, flag }` variant exists** in `hp41-core/src/ops/mod.rs::Op`.
3. **1 new dispatch arm exists** for interactive no-op behavior.
4. **1 new run_loop arm exists** in `hp41-core/src/ops/program.rs` implementing the 4-kind skip semantic + always-clear side effect.
5. **`Op::FlagTest { .. }` is appended to the execute_op catch-all** so execute_op rejects FlagTest variants explicitly (Pitfall 2 sentinel).
6. **1 new op_display_name arm exists in BOTH** prgm_display.rs copies, byte-identical.
7. **10 new tests appended** to `hp41-core/tests/phase21_flags.rs`; all pass GREEN; total file has ≥ 19 tests.
8. **`just ci` passes**.
9. **`just gui-ci` passes**.
10. **Coverage ≥ 92.5%** on hp41-core (non-regression from Plan 21-01).
11. **SC-4 invariant grep** returns nothing.
12. **The plan SUMMARY** (`21-02-SUMMARY.md`) is committed.
</success_criteria>

<output>
After completion, create `.planning/phases/21-flags-display-control-sound/21-02-SUMMARY.md` covering:

- **Plan:** 21-02 (Conditional flag tests + skip-next-step semantic — FN-FLAG-02)
- **Status:** Complete | Partial | Blocked
- **Files touched:** the 5 in `files_modified` (ops/mod.rs, ops/program.rs, both prgm_display.rs copies, tests/phase21_flags.rs)
- **What landed:** FlagTestKind enum, Op::FlagTest struct variant, run_loop arm with 4-kind match and always-clear side effect, execute_op catch-all extension, prgm_display arms in both copies, 10 new integration tests
- **Test results:** count of new tests, pass/fail breakdown of `just ci` and `just gui-ci`, coverage % (≥ 92.5%)
- **Followups for Phases 25 / 26:** the new Op::FlagTest variant + FlagTestKind are awaiting `key_to_op` keyboard wiring (Phase 25) and `key_map::resolve` + new modal flow in the GUI (Phase 26)
- **Architectural note:** RESEARCH A4 (strict "always-clear" reading of FS?C / FC?C) was honored; document this in CLAUDE.md "v2.2 additions" if not already covered by Plan 21-01

Use `/git-workflow:commit --with-skills` to commit the changes (German Emoji Conventional Commits, English-only message body).
</output>
</content>
</invoke>