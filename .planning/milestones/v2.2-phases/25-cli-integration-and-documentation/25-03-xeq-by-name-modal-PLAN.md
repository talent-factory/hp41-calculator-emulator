---
phase: 25-cli-integration-and-documentation
plan: 03
type: execute
wave: 3
depends_on: [01, 02]
files_modified:
  - hp41-core/src/ops/program.rs
  - hp41-cli/src/keys.rs
  - hp41-cli/src/app.rs
  - hp41-cli/tests/phase25_xeq_by_name.rs
autonomous: true
requirements:
  - FN-TEST-01
user_setup: []
tags:
  - cli
  - xeq-by-name
  - hp41-core-surgical

must_haves:
  truths:
    - "hp41-core::ops::program::builtin_card_op resolves 12 names (4 v2.1 card-reader + 8 conditional-test mnemonics)"
    - "Each of the 8 non-keyboard conditional tests (X≠Y, X<Y, X≥Y, X≠0, X<0, X>0, X≤0, X≥0) is reachable from the CLI via the XEQ-by-Name modal as a keystroke sequence: open the modal (X outside PRGM), type the mnemonic, press Enter (per D-25.9)"
    - "Both ASCII-pure spellings (X<>Y?, X#Y?, X#0?, X<=0?, X>=0?) and Unicode-symbol spellings (X≠Y?, X≠0?, X≤0?, X≥0?) resolve to the correct Op::Test(TestKind::*) variant (RESEARCH §Conditional tests)"
    - "Programmatic XEQ (`Op::Xeq(\"X<>Y?\")` constructed by run_program / op_xeq / run_loop) ALSO resolves the 8 conditional-test mnemonics — symmetry between keyboard and programmatic paths is preserved per HP-41 hardware semantics"
    - "The pre-existing 4-name builtin_card_op resolution (WPRGM/RDPRGM/WDTA/RDTA) continues to work — no regression"
    - "Unknown XEQ names fall through to HpError::InvalidOp (existing Pitfall 9 behavior — documented, NOT a hint with 'did you mean...?' until Phase 26)"
    - "Op::Xeq mnemonic asymmetry (W4 documented, NOT a defect): the 4 keyboard-reachable conditional tests (X=Y, X≤Y, X>Y, X=0) are reached ONLY via the f-arith keystroke path from Plan 01 — they are intentionally NOT registered in builtin_card_op as XEQ-by-Name mnemonics. The 8 non-keyboard conditional tests (X≠Y, X<Y, X≥Y, X≠0, X<0, X>0, X≤0, X≥0) ARE registered in builtin_card_op so they reach Op::Test via XEQ. This 4 vs 12 asymmetry is hardware-faithful (HP-41CV ROM does not name X=Y? / X<=Y? / X>Y? / X=0? as XEQ targets — they are exclusively keyboard ops); a user who types `XEQ \"X=Y?\"` gets HpError::InvalidOp by design."
    - "All 12 conditional tests are now keyboard-reachable per D-25.9: 4 via f-arith (Plan 01) + 8 via XEQ-by-Name modal (this plan) = FN-TEST-01 CLOSED"
  artifacts:
    - path: "hp41-core/src/ops/program.rs"
      provides: "builtin_card_op extended 4→12 name entries (surgical hp41-core exception cleared by user per Open Question 2 in RESEARCH; D-25.8 implementation)"
      contains: "Op::Test(TestKind::XNeY)"
    - path: "hp41-cli/src/keys.rs"
      provides: "Free function xeq_by_name_local_resolve(&str) -> Option<Op> — CLI-side resolver that accepts both ASCII and Unicode mnemonic spellings; routes to Op::Test(TestKind::*) directly"
      contains: "xeq_by_name_local_resolve"
    - path: "hp41-cli/src/app.rs"
      provides: "XeqByName Enter-arm wired to call xeq_by_name_local_resolve first, falling through to Op::Xeq() (which now resolves the same 12 mnemonics via the extended builtin_card_op) — guarantees keyboard + programmatic symmetry"
      contains: "xeq_by_name_local_resolve"
    - path: "hp41-core/src/ops/program.rs (inline #[cfg(test)] mod phase25_builtin_card_op_tests)"
      provides: "5 inline tests: 8 mnemonic-resolution (ASCII+Unicode), 4-name regression, unknown-name None, case-sensitivity, programmatic XEQ symmetry — placed inline to avoid widening builtin_card_op's pub(super) visibility (W1 fix)"
      contains: "resolves_8_conditional_test_mnemonics"
    - path: "hp41-cli/tests/phase25_xeq_by_name.rs"
      provides: "End-to-end keystroke-sequence tests — open XEQ modal, type each mnemonic, press Enter, assert correct Op::Test dispatch; covers both ASCII and Unicode forms"
      contains: "xeq_by_name_resolves_x_ne_y"
  key_links:
    - from: "hp41-core/src/ops/program.rs::builtin_card_op"
      to: "hp41-core::ops::Op::Test(TestKind::XNeY..XGeZero)"
      via: "Direct match-arm returns for 8 new mnemonic strings"
      pattern: "Op::Test\\(TestKind::"
    - from: "hp41-cli/src/app.rs::handle_pending_input (XeqByName arm)"
      to: "hp41-cli/src/keys.rs::xeq_by_name_local_resolve"
      via: "Enter triggers first-pass CLI-local resolve; falls through to Op::Xeq(name) → core builtin_card_op"
      pattern: "xeq_by_name_local_resolve"
---

<objective>
Close FN-TEST-01 by making all 12 HP-41CV conditional tests reachable from the CLI keyboard: extend hp41-core's `builtin_card_op` from 4 names to 12 (surgical exception to the "hp41-core FROZEN" rule per RESEARCH Recommendation Path 2, cleared by the user) AND wire the CLI-local `xeq_by_name_local_resolve` to dispatch the 8 non-keyboard conditional-test mnemonics directly through the XEQ-by-Name modal scaffold introduced in Plan 02.

Purpose: D-25.8 demands that the 8 non-keyboard conditional tests be reachable "via the XEQ-by-Name palette" — but the palette didn't exist in hp41-cli prior to Plan 02 (RESEARCH Assumption A3 surfaced this CONTEXT inaccuracy). With Plan 02's XeqByName modal scaffold in place, this plan supplies the resolver behind the Enter key. Per RESEARCH Recommendation Path 2 (cleared by user via planning_context note: "extending `builtin_card_op` in hp41-core/src/ops/program.rs from 4 to 12 mnemonics so the new CLI XEQ-by-Name modal AND programmatic XEQ resolve the 8 non-keyboard conditional tests through a single source of truth"), the cleanest implementation extends the existing 4-entry match arm to 12 entries — no new Op variants, no new state, no new error variants, just an enlarged name table.

Output: builtin_card_op resolves 12 names (4 + 8 conditional tests); xeq_by_name_local_resolve mirrors the same 12 mappings in hp41-cli for ASCII+Unicode form acceptance; XEQ-by-Name modal Enter-arm dispatches the resolved Op::Test directly; comprehensive integration tests for keyboard and programmatic paths.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md
@.planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md
@.planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md
@.planning/phases/25-cli-integration-and-documentation/25-01-SUMMARY.md
@.planning/phases/25-cli-integration-and-documentation/25-02-SUMMARY.md
@CLAUDE.md

<interfaces>
<!-- Key types and contracts for this plan. -->

From hp41-core/src/ops/program.rs (lines 960–974 — CURRENT shape; EXTEND match arms):

  pub(super) fn builtin_card_op(name: &str) -> Option<Op> {
      match name {
          "WPRGM" => Some(Op::Wprgm),
          "RDPRGM" => Some(Op::Rdprgm),
          "WDTA" => Some(Op::Wdta),
          "RDTA" => Some(Op::Rdta),
          _ => None,
      }
  }

  Existing call sites (READ ONLY — no changes needed):
  - hp41-core/src/ops/program.rs (around line 73, 389, 506) — op_xeq, run_program, run_loop fall back to builtin_card_op after user-LBL search fails

  Existing test mirror (preserve regression):
  - hp41-core/src/ops/program.rs (lines 1495–1510) — `builtin_card_op_resolves_four_names` continues to PASS unchanged

From hp41-core/src/ops/mod.rs (TestKind enum — Phase 3, 12 variants — DO NOT MODIFY):
  pub enum TestKind {
      XEqY, XNeY, XLtY, XGtY, XLeY, XGeY,
      XEqZero, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero,
  }

  Of these:
  - 4 mapped to f-arith keys in Plan 01: XEqY (f-), XLeY (f+), XGtY (f*), XEqZero (f/)
  - 8 reachable ONLY via XEQ-by-Name in this plan: XNeY, XLtY, XGeY, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero

From hp41-cli/src/keys.rs (Plan 01+02 output):
  - `pub fn shifted_key_to_op(key: KeyEvent, app: &mut App) -> Option<Op>` already exists
  - This plan ADDS a new sibling free function `pub fn xeq_by_name_local_resolve(name: &str) -> Option<Op>`

From hp41-cli/src/app.rs (Plan 02 output):
  - `Some(PendingInput::XeqByName(ref acc))` arm in handle_pending_input exists; on Enter it currently dispatches `Op::Xeq(acc.clone())`. This plan PREPENDS the xeq_by_name_local_resolve fast-path.

ROM mnemonic spellings (from RESEARCH §"Conditional tests — keyboard vs XEQ-by-Name" — accept BOTH forms):
  - X<>Y? | X≠Y? | X#Y?     → Op::Test(TestKind::XNeY)
  - X<Y?                     → Op::Test(TestKind::XLtY)
  - X>=Y? | X≥Y?             → Op::Test(TestKind::XGeY)
  - X#0? | X≠0?              → Op::Test(TestKind::XNeZero)
  - X<0?                     → Op::Test(TestKind::XLtZero)
  - X>0?                     → Op::Test(TestKind::XGtZero)
  - X<=0? | X≤0?             → Op::Test(TestKind::XLeZero)
  - X>=0? | X≥0?             → Op::Test(TestKind::XGeZero)

</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend hp41-core::ops::program::builtin_card_op from 4 to 12 mnemonic entries</name>
  <files>hp41-core/src/ops/program.rs</files>
  <read_first>
    - hp41-core/src/ops/program.rs (lines 955–975 — current builtin_card_op definition + its doc comment; lines 1495–1510 — existing 4-name regression test inside `program_tests` module)
    - hp41-core/src/ops/mod.rs (locate `pub enum TestKind` to confirm 12 variant names match the mnemonic table in <interfaces>)
    - hp41-core/src/ops/program.rs (lines around 73, 389, 506 — call sites of builtin_card_op for op_xeq / run_program / run_loop — READ ONLY to confirm call signatures unchanged)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"Code Examples → Extending the XEQ-by-Name resolver" (Recommendation Path 2, cleared by user)
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (hp41-core builtin_card_op 4→12 extension section)
    - CLAUDE.md ("hp41-core FROZEN" rule — this is the documented surgical exception cleared by the user)
  </read_first>
  <behavior>
    - builtin_card_op accepts the 4 v2.1 card-reader names UNCHANGED (regression: existing test `builtin_card_op_resolves_four_names` MUST continue to pass).
    - builtin_card_op accepts each of the 8 conditional-test mnemonics in BOTH ASCII-pure and Unicode-symbol form per <interfaces>: e.g. `"X<>Y?" | "X≠Y?" | "X#Y?"` → Some(Op::Test(TestKind::XNeY)); etc.
    - Case-sensitive — `"x<>y?"` returns None (HP-41 ROM names are uppercase).
    - Unknown name returns None.
    - All 3 existing call sites (op_xeq line 73, run_program line 389, run_loop line 506) work without modification — the function signature is unchanged.
    - Lift/skip semantics for `Op::Test` already implemented in run_loop; this plan adds NO new dispatch logic — only the name resolver grows.
  </behavior>
  <action>
    Step 1 — Extend builtin_card_op: edit hp41-core/src/ops/program.rs around line 966–974. ADD a `use crate::ops::TestKind;` line at the top of the function body (or at the file head if not already imported). REPLACE the existing match-arm block with one that keeps the 4 v2.1 names AND adds the 8 conditional-test mnemonics per <interfaces>. Use Rust's or-pattern syntax (e.g. `"X<>Y?" | "X≠Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY))`) for the mnemonics that accept multiple spellings. Preserve the trailing `_ => None` arm. Update the function doc-comment to document the 12 names and call out the Phase 25 D-25.8 origin.

    Step 2 — Update inline regression test: the existing `builtin_card_op_resolves_four_names` test at lines 1495–1510 stays as-is (regression coverage for the 4 v2.1 names). Do NOT modify it.

    Step 3 — Inline test module in `hp41-core/src/ops/program.rs`: do NOT widen `builtin_card_op`'s visibility (it stays `pub(super) fn` per W1 fix). Instead, place the new tests INSIDE `hp41-core/src/ops/program.rs` as a `#[cfg(test)] mod phase25_builtin_card_op_tests { … }` block adjacent to the existing inline `program_tests` module (lines ~1495+). The block carries `#![allow(clippy::unwrap_used)]` (mirroring existing test-module convention) and `use super::*;` / `use crate::ops::TestKind;` so `builtin_card_op` is reachable via `super::` without changing its visibility. No new file under `hp41-core/tests/` is created in this plan — the inline module gives the same coverage with zero API-surface impact.

    Tests inside `mod phase25_builtin_card_op_tests`:
    a. `resolves_8_conditional_test_mnemonics` — 16 assertions (8 ASCII + 8 Unicode), one per spelling. e.g. `assert_eq!(builtin_card_op("X<>Y?"), Some(Op::Test(TestKind::XNeY)));` and `assert_eq!(builtin_card_op("X≠Y?"), Some(Op::Test(TestKind::XNeY)));` etc.
    b. `preserves_4_card_reader_names` — 4 assertions on WPRGM/RDPRGM/WDTA/RDTA (regression mirror of the inline test at lines 1495–1510; do NOT remove or duplicate that test — the new test is an additional independent regression bound to the new module).
    c. `unknown_name_returns_none` — assert builtin_card_op("foobar") == None; builtin_card_op("") == None; builtin_card_op("FOOBAR") == None (uppercase but unknown).
    d. `case_sensitive_lowercase_rejected` — assert builtin_card_op("wprgm") == None; builtin_card_op("x<>y?") == None.
    e. `programmatic_xeq_dispatches_x_ne_y` — Build a small program `[Op::Lbl("TEST".into()), Op::Xeq("X<>Y?".into()), Op::Rtn]`. Initialize `CalcState` with `stack.y = HpNum::from(5); stack.x = HpNum::from(7)`. Call `run_program(&mut state, "TEST")`. Assert `state.skip_next_step` toggled correctly (XNeY: 5≠7 → test passes → no skip). Confirms keyboard + programmatic symmetry (one of the success criteria from <objective>).

    Use `.expect("reason")` outside test bodies; `.unwrap()` permitted inside `#[test]` bodies (the new inline module carries `#![allow(clippy::unwrap_used)]`).

    Note (W1 fix, 2026-05-14): The plan-checker initially flagged a conditional `pub(super) → pub` visibility widening on `builtin_card_op`. Resolution: keep `pub(super) fn` UNCHANGED and place the tests inline in the same file. No new `D-25.19` decision is required because the API surface is unchanged.

    Use plain English in commit messages.
  </action>
  <verify>
    <automated>cargo test -p hp41-core --lib builtin_card_op_12 && cargo test -p hp41-core --lib builtin_card_op_resolves_four_names</automated>
  </verify>
  <acceptance_criteria>
    - hp41-core/src/ops/program.rs builtin_card_op match arms count: `grep -nE "Some\\(Op::(Wprgm|Rdprgm|Wdta|Rdta|Test\\(TestKind))" hp41-core/src/ops/program.rs | wc -l` ≥ 12 distinct mappings (4 card-reader Some-returns + at least 8 distinct TestKind variants)
    - All 8 TestKind variants targeted by the 8 mnemonics appear in builtin_card_op: `grep -oE "TestKind::(XNeY|XLtY|XGeY|XNeZero|XLtZero|XGtZero|XLeZero|XGeZero)" hp41-core/src/ops/program.rs | sort -u | wc -l` == 8
    - Inside hp41-core/src/ops/program.rs the new module exists: `grep -n "mod phase25_builtin_card_op_tests" hp41-core/src/ops/program.rs` returns 1 line
    - The new module has ≥5 `#[test]` functions: count via `awk '/mod phase25_builtin_card_op_tests/,/^}/' hp41-core/src/ops/program.rs | grep -c "^[[:space:]]*#\[test\]"` ≥ 5
    - `cargo test -p hp41-core builtin_card_op_12` exits 0 (runs the inline test `cargo test -p hp41-core builtin_card_op_12` per W1 fix — no visibility change to API surface)
    - `builtin_card_op` keeps `pub(super) fn` visibility: `grep -n "pub(super) fn builtin_card_op" hp41-core/src/ops/program.rs` returns 1 line; `grep -nE "^pub fn builtin_card_op|^[[:space:]]+pub fn builtin_card_op" hp41-core/src/ops/program.rs` returns 0 lines
    - The inline `builtin_card_op_resolves_four_names` regression test PASSES: `cargo test -p hp41-core --lib builtin_card_op_resolves_four_names` exits 0
    - `cargo clippy -p hp41-core --tests -- -D warnings` passes
    - `cargo build -p hp41-core` clean
  </acceptance_criteria>
  <done>
    hp41-core::builtin_card_op resolves 12 names; the 4 v2.1 names regression is intact; 5+ new inline tests in `mod phase25_builtin_card_op_tests` inside hp41-core/src/ops/program.rs all GREEN including the programmatic-symmetry test that exercises run_program → builtin_card_op → Op::Test dispatch end-to-end. `builtin_card_op` visibility is UNCHANGED (still `pub(super) fn`) per W1 fix.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add CLI-local xeq_by_name_local_resolve + wire XeqByName Enter-arm</name>
  <files>hp41-cli/src/keys.rs, hp41-cli/src/app.rs, hp41-cli/tests/phase25_xeq_by_name.rs</files>
  <read_first>
    - hp41-cli/src/keys.rs (Plan 01+02 output — shifted_key_to_op signature and import block)
    - hp41-cli/src/app.rs (Plan 02 output — XeqByName arm in handle_pending_input)
    - hp41-cli/tests/phase25_pending_input.rs (Plan 02 output — `test_xeq_by_name_modal_scaffold` test; this plan REPLACES the scaffold-only assertion with full conditional-test resolution coverage)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"XEQ-by-Name CLI Modal" (xeq_by_name_local_resolve template)
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (xeq_by_name_local_resolve section)
  </read_first>
  <behavior>
    - xeq_by_name_local_resolve(name) returns Some(Op) for each of the 8 conditional-test mnemonics in BOTH ASCII and Unicode forms (mapping table from <interfaces>). Returns None for the 4 v2.1 card-reader names (those fall through to Op::Xeq → builtin_card_op chain). Returns None for unknown names.
    - The XeqByName modal Enter-arm in hp41-cli/src/app.rs::handle_pending_input first calls `xeq_by_name_local_resolve(&acc)`; if Some(op) → call_dispatch(op); else `self.call_dispatch(Op::Xeq(acc.clone()))` (existing fallback). Same Esc/Backspace/Char behavior as Plan 02 — only the Enter path changes.
    - The CLI-local resolver is SEMANTICALLY EQUIVALENT to the extended hp41-core::builtin_card_op for the 8 conditional tests — this gives the test suite two independent paths that should agree (cross-check test below).
    - Why both? Path 1 (CLI-local) gives immediate dispatch without constructing Op::Xeq+run_program; Path 2 (hp41-core extension) ensures programmatic XEQ inside a saved program also resolves. Both are needed for full FN-TEST-01 coverage.
  </behavior>
  <action>
    Step 1 — Add xeq_by_name_local_resolve to hp41-cli/src/keys.rs: free function at module level (not inside an impl). Signature: `pub fn xeq_by_name_local_resolve(name: &str) -> Option<hp41_core::ops::Op>`. Body: import `use hp41_core::ops::{Op, TestKind};` if not already in scope; match arms per <interfaces> mnemonic table (8 arms with or-pattern syntax for ASCII+Unicode). Default arm `_ => None`. Add doc-comment explaining: "Resolves the 8 non-keyboard conditional-test mnemonics ahead of the hp41-core::builtin_card_op fallback. Both ASCII and Unicode spellings accepted per D-25.10 + RESEARCH §Conditional tests."

    Step 2 — Wire the resolver into hp41-cli/src/app.rs::handle_pending_input's XeqByName Enter-arm. Locate the existing arm from Plan 02 (`Some(PendingInput::XeqByName(ref acc)) =>`). On `KeyCode::Enter`, BEFORE the fallback `self.call_dispatch(Op::Xeq(acc.clone()))`, add: `if let Some(op) = keys::xeq_by_name_local_resolve(acc) { self.call_dispatch(op); self.pending_input = None; return; }`. Then preserve the existing Op::Xeq fallback for the 4 card-reader names + user-defined labels.

    Step 3 — Create hp41-cli/tests/phase25_xeq_by_name.rs with `#![allow(clippy::unwrap_used)]`. Imports mirror phase25_pending_input.rs from Plan 02 (KeyEvent helper, App construction). Tests:

    a. `xeq_by_name_resolves_x_ne_y` — open XeqByName modal, type `X` `<` `>` `Y` `?` (5 char keys), press Enter; assert dispatched op was Op::Test(TestKind::XNeY) AND skip_next_step set correctly (set state.stack.y=5, stack.x=7 before the test; XNeY: 5≠7 → no skip).
    b. `xeq_by_name_resolves_x_lt_y` — same pattern for `X<Y?` → XLtY.
    c. `xeq_by_name_resolves_x_ge_y` — `X>=Y?` AND `X≥Y?` (two sub-cases) → XGeY.
    d. `xeq_by_name_resolves_x_ne_zero` — `X#0?` AND `X≠0?` → XNeZero.
    e. `xeq_by_name_resolves_x_lt_zero` — `X<0?` → XLtZero.
    f. `xeq_by_name_resolves_x_gt_zero` — `X>0?` → XGtZero.
    g. `xeq_by_name_resolves_x_le_zero` — `X<=0?` AND `X≤0?` → XLeZero.
    h. `xeq_by_name_resolves_x_ge_zero` — `X>=0?` AND `X≥0?` → XGeZero.
    i. `xeq_by_name_unicode_form_works` — explicit unicode-only test: type `X` `≠` `Y` `?` (using `KeyEvent::new(KeyCode::Char('≠'), …)`); assert dispatch is XNeY.
    j. `xeq_by_name_falls_through_to_card_reader` — type `W` `P` `R` `G` `M` and press Enter; assert dispatched op was Op::Wprgm (not Op::Xeq("WPRGM") — the resolver chain found the v2.1 card-reader name via the core fallback).
    k. `xeq_by_name_unknown_returns_invalid_op` — type `F` `O` `O` `B` `A` `R` and press Enter; assert app.message contains an InvalidOp indicator (Pitfall 9 — documented behavior, no "did you mean…?" hint until Phase 26).
    l. `all_12_conditional_tests_reachable` — comprehensive coverage: for each of the 12 TestKind variants, dispatch via either the keyboard path (4) or the XEQ-by-Name path (8), and assert the correct Op was dispatched. This is the FN-TEST-01 closure test.
    m. `cli_resolver_matches_core_resolver` — cross-check: for each of the 8 conditional-test mnemonics (ASCII spelling), assert `keys::xeq_by_name_local_resolve(name) == hp41_core::ops::program::builtin_card_op(name)`. Catches drift between the two resolvers.

    Use `.expect("reason")` outside test bodies; `.unwrap()` permitted inside test bodies.
  </action>
  <verify>
    <automated>cargo test -p hp41-cli --test phase25_xeq_by_name</automated>
  </verify>
  <acceptance_criteria>
    - `grep -n "pub fn xeq_by_name_local_resolve" hp41-cli/src/keys.rs` returns exactly 1 line
    - xeq_by_name_local_resolve has at least 8 distinct TestKind targets: `grep -oE "TestKind::(XNeY|XLtY|XGeY|XNeZero|XLtZero|XGtZero|XLeZero|XGeZero)" hp41-cli/src/keys.rs | sort -u | wc -l` == 8
    - `grep -n "xeq_by_name_local_resolve" hp41-cli/src/app.rs` matches in the handle_pending_input XeqByName Enter-arm (verify by reading lines around the existing arm)
    - File hp41-cli/tests/phase25_xeq_by_name.rs exists with `#![allow(clippy::unwrap_used)]` and ≥12 `#[test]` functions: `grep -c "^#\\[test\\]" hp41-cli/tests/phase25_xeq_by_name.rs` ≥ 12
    - All tests pass: `cargo test -p hp41-cli --test phase25_xeq_by_name` exits 0
    - Test `all_12_conditional_tests_reachable` PASSES — closes FN-TEST-01
    - Test `cli_resolver_matches_core_resolver` PASSES — confirms no drift between CLI and core resolvers
    - `cargo clippy -p hp41-cli --tests -- -D warnings` passes
  </acceptance_criteria>
  <done>
    xeq_by_name_local_resolve in keys.rs mirrors the hp41-core builtin_card_op extension for the 8 conditional tests; XeqByName Enter-arm uses the CLI-local fast-path with core fallback; all 12 conditional tests now keyboard-reachable per D-25.9 (4 via f-arith + 8 via XEQ-by-Name); FN-TEST-01 CLOSED; cross-resolver drift test in place.
  </done>
</task>

</tasks>

<verification>
- `cargo test -p hp41-core --lib builtin_card_op_12` exits 0 with ≥5 inline tests GREEN (inline `mod phase25_builtin_card_op_tests` per W1 fix — no external test file).
- `cargo test -p hp41-cli --test phase25_xeq_by_name` exits 0 with ≥12 tests GREEN.
- `cargo test -p hp41-core --lib builtin_card_op_resolves_four_names` exits 0 (no regression in v2.1 4-name behavior).
- `cargo test -p hp41-cli --test phase25_keyboard` (Plan 01 regression) and `cargo test -p hp41-cli --test phase25_pending_input` (Plan 02 regression) BOTH exit 0.
- `just ci` GREEN (full workspace including hp41-core 800+ tests + hp41-cli tests).
- `cargo clippy --workspace -- -D warnings` passes.
- Manual smoke: `just run-cli`; press `X` (outside PRGM mode) → XEQ modal opens; type `X<>Y?` and press Enter → conditional test dispatches against y/x stack values; repeat for unicode `X≠Y?`; type unknown name `FOO` → error message.
</verification>

<success_criteria>
- builtin_card_op in hp41-core extended from 4 to 12 names; the 4 v2.1 names regression intact.
- xeq_by_name_local_resolve in hp41-cli mirrors the same 8 conditional-test mnemonic mappings; both accept ASCII and Unicode spellings.
- XeqByName modal Enter-arm uses the CLI-local fast-path; falls through to core for everything else.
- All 12 HP-41CV conditional tests are reachable from the CLI via keyboard or modal keystroke sequence per D-25.9.
- FN-TEST-01 is CLOSED.
- Programmatic + keyboard symmetry: `Op::Xeq("X<>Y?")` inside a saved program resolves identically to typing it in the XEQ modal.
- Cross-resolver drift test prevents future divergence between the two resolvers.
- All Wave-0 tests GREEN.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| User-typed XEQ name → builtin_card_op or xeq_by_name_local_resolve | Unbounded string input (capped at 24 chars by the modal — Plan 02 cap) |
| Saved program → run_program/op_xeq/run_loop → builtin_card_op | Untrusted program file contents (HP-41 .h41 / autosave.json) |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-25-09 | Tampering | Resolver drift between hp41-core builtin_card_op and hp41-cli xeq_by_name_local_resolve | mitigate | `cli_resolver_matches_core_resolver` cross-check test in phase25_xeq_by_name.rs runs on every CI build |
| T-25-10 | Denial of Service | Saved program containing unknown XEQ name causes panic during run_program | accept | Existing fallback chain returns `HpError::InvalidOp`; no panic path. Documented per Pitfall 9. |
| T-25-11 | Information Disclosure | Case-insensitive match could allow Mojibake bypass of HP-41 ROM name semantics | mitigate | Case-sensitive match in builtin_card_op verified by `case_sensitive_lowercase_rejected` test |
| T-25-12 | Tampering | Future Op variant added without matching builtin_card_op entry → mnemonic dispatch fails silently | accept | Documented: builtin_card_op covers ONLY 4 + 8 = 12 hardware ROM names; user-defined LBLs and `Op::Xeq(label)` programmatic dispatch remain primary path. Phase 26+ may extend if a new ROM op needs mnemonic dispatch. |
</threat_model>

<output>
After completion, create `.planning/phases/25-cli-integration-and-documentation/25-03-SUMMARY.md` per execute-plan template. Record: that `builtin_card_op` visibility was UNCHANGED (`pub(super) fn` — W1 fix from 2026-05-14 plan revision); the final 12-name table for D-25.16 JSON ingestion in Plan 04; the cross-resolver drift test as a guard against future Phase 26+ resolver divergence; closure of FN-TEST-01.
</output>
