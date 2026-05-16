---
phase: 21
plan: 04
type: execute
wave: 1
depends_on:
  - 21-01
files_modified:
  - hp41-core/src/state.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/sound.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase21_sound.rs
autonomous: true
requirements:
  - FN-SOUND-01
  - FN-SOUND-02
tags:
  - hp41
  - rust
  - sound
  - event-channel
  - io-free

must_haves:
  truths:
    - "CalcState carries a new `event_buffer: Vec<String>` field with `#[serde(default, skip)]` — transient (like print_buffer); drained by frontend on each dispatch; never persisted (FN-SOUND-01/02; RESEARCH §Pattern 3)"
    - "`Op::Beep` (nullary) pushes literal `BEEP` into state.event_buffer; no I/O; LiftEffect::Neutral (FN-SOUND-01)"
    - "`Op::Tone(u8)` for n in 0..=9 pushes `TONE n` formatted string into state.event_buffer; n > 9 returns HpError::InvalidOp without pushing (FN-SOUND-02)"
    - "No `println!` or `eprintln!` anywhere in hp41-core production code after Plan 21-04 lands — verified by `grep -rn` returning 0 production lines"
    - "Resource cap: `run_loop` MAX_STEPS (1,000,000) at program.rs:175 prevents unbounded event_buffer growth — a program that issues BEEP in an infinite loop is capped at 1M events maximum per run_program invocation"
    - "Both new Op variants (Beep, Tone(u8)) land in 4 places: enum + dispatch + execute_op + BOTH prgm_display.rs copies"
    - "v2.0 fixture loads cleanly with `event_buffer` defaulting to empty Vec — backward compat verified (SC-5 spillover from Plan 21-01)"
    - "Phase 11 print_tests + Phase 20 phase20_math + Plan 21-01 phase21_flags tests stay green — no regression"

  artifacts:
    - path: "hp41-core/src/state.rs"
      provides: "New `event_buffer: Vec<String>` field with #[serde(default, skip)] (transient, mirrors print_buffer exactly)"
      contains: "pub event_buffer: Vec<String>"
    - path: "hp41-core/src/ops/sound.rs"
      provides: "NEW module — op_beep (push BEEP literal) and op_tone(n) (range guard + push TONE n)"
      contains: "pub fn op_beep"
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Module declaration `pub mod sound;`; 2 new Op variants (Beep, Tone(u8)); 2 new dispatch arms"
      contains: "Op::Beep"
    - path: "hp41-core/src/ops/program.rs"
      provides: "2 new execute_op arms forwarding to sound::op_beep / sound::op_tone"
      contains: "Op::Beep"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "2 new op_display_name arms: Op::Beep returns BEEP string, Op::Tone(n) returns TONE n formatted"
      contains: "Op::Beep"
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same 2 op_display_name arms (SC-4 spirit exception)"
      contains: "Op::Beep"
    - path: "hp41-core/tests/phase21_sound.rs"
      provides: "Integration tests covering FN-SOUND-01/02: BEEP push, TONE n push (n=0,5,9), TONE 10 InvalidOp, event_buffer skipped on serialize, v2.0-fixture-load backward compat (asserts event_buffer is empty Vec after load), no-println-in-core grep regression sentinel"
      contains: "fn test_beep_pushes_event"
      min_tests: 7

  key_links:
    - from: "hp41-core/src/ops/sound.rs::op_beep"
      to: "state.event_buffer + apply_lift_effect"
      via: "Push BEEP literal to event_buffer, then apply_lift_effect Neutral — verbatim mirror of op_prx from print.rs:13-18 with print_buffer renamed to event_buffer and the fixed string literal"
      pattern: "state\\.event_buffer\\.push"
    - from: "hp41-core/src/ops/sound.rs::op_tone"
      to: "state.event_buffer (with format!)"
      via: "Range guard then format-and-push"
      pattern: "format!\\(\"TONE \\{n\\}\"\\)"
    - from: "hp41-core/src/state.rs::event_buffer"
      to: "serde idiom #[serde(default, skip)]"
      via: "Identical to print_buffer at state.rs:89-94 — the EXACT analog"
      pattern: "#\\[serde\\(default, skip\\)\\]\\s+pub event_buffer: Vec<String>"
---

<objective>
Land the HP-41 sound subsystem in `hp41-core`: 2 new ops (`BEEP`, `TONE n`) backed by a new `event_buffer: Vec<String>` field on `CalcState`. Both ops push structured event lines into the buffer; the frontend (CLI in Phase 25, GUI in Phase 26) drains the buffer and routes the events to audio output. NO direct I/O in `hp41-core` — the zero-I/O invariant from Phase 11 (`print_buffer`) is honored.

This plan is the smallest of the four Phase 21 plans (~120 LOC). It is the closest analog to Phase 11 — the buffer-channel pattern is reused verbatim.

This plan **depends on Plan 21-01** (`depends_on: ["21-01"]`) because:
1. The Wave-0 fixture file `hp41-core/tests/fixtures/v20-autosave.json` is created by Plan 21-01 Task 1 and consumed by this plan's `test_load_v20_save_no_event_buffer_field` integration test.
2. The Wave-0 `justfile::test-core *args:` recipe is added by Plan 21-01 Task 0 and used by this plan's verify blocks (`just test-core --test phase21_sound`, etc.).

Unlike Plan 21-03, this plan does NOT have a compile-time dependency on Plan 21-01 (no flags helpers are imported by `sound.rs`). The dependency is purely on the test fixture and the justfile recipe. The orchestrator MUST still serialize 21-04 after 21-01 because both touch the same shared files (see Scheduling note below).

Purpose: Restore HP-41CV BEEP/TONE parity. CLI keyboard wiring and GUI audio playback land in Phase 25/26.

Output: 7 modified files. Net new line count ≈ 100 LOC. All CI gates stay green.

## Scheduling note

Although Plans 21-01, 21-03, 21-04 are semantically independent within Wave 1, they share five files (`hp41-core/src/state.rs`, `hp41-core/src/ops/mod.rs`, `hp41-core/src/ops/program.rs`, `hp41-cli/src/prgm_display.rs`, `hp41-gui/src-tauri/src/prgm_display.rs`). The execute-phase orchestrator MUST serialize them at the merge layer. The `depends_on` chain (21-03 → 21-01, 21-04 → 21-01) forces 21-01 first; the orchestrator should additionally serialize 21-03 then 21-04. Plan 21-02 is in Wave 2 (depends on 21-01) and runs after all three Wave-1 plans have merged.
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
@.planning/milestones/v1.1/11-print-emulation/11-PLAN.md
@CLAUDE.md

# Source files the executor will modify
@hp41-core/src/state.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/ops/program.rs
@hp41-core/src/ops/print.rs
@hp41-cli/src/prgm_display.rs
@hp41-gui/src-tauri/src/prgm_display.rs

<interfaces>
<!-- Key types and contracts the executor needs. Extracted from the codebase on 2026-05-14 by direct Read. -->

From Plan 21-01 (must be merged first per depends_on):
- `hp41-core/tests/fixtures/v20-autosave.json` — consumed by this plan's fixture-load test
- `justfile::test-core *args:` recipe — used by every verify block in this plan

From `hp41-core/src/state.rs` lines 89-94 (`print_buffer` — the EXACT analog this plan reuses):
- Doc comment about buffered print lines drained by hp41-cli after each dispatch, never persisted.
- Attribute `#[serde(default, skip)]`.
- Field: `pub print_buffer: Vec<String>`.

Phase 21 application — `event_buffer` is a structural twin with a different name and a different consumer (BEEP/TONE write here, the frontend reads and plays audio).

From `hp41-core/src/ops/print.rs` lines 11-18 (`op_prx` — the verbatim analog for `op_beep`):
- `op_prx` formats X for display, pushes one line to `state.print_buffer`, calls `apply_lift_effect(state, LiftEffect::Neutral)`, returns `Ok(())`.

Phase 21 application — `op_beep` is the same shape with two differences: (1) the line content is the fixed literal `"BEEP".to_string()` (no stack read, no display_mode), and (2) the target buffer is `state.event_buffer`.

From `hp41-core/src/error.rs`:
- `HpError::InvalidOp` is reused for the TONE n > 9 range-guard error. No new variant needed.

From `hp41-core/src/stack.rs`:
- `pub fn apply_lift_effect(state: &mut CalcState, effect: LiftEffect)` — the standard tail call.
- `LiftEffect::Neutral` — used by both new ops.

From `hp41-core/src/ops/mod.rs` lines 9-18:
- The alphabetical `pub mod` block. `sound` goes after `registers` and before `stack_ops` (alphabetically).

From `hp41-core/tests/print_tests.rs`:
- Existing tests assert `state.print_buffer.contains(...)` after dispatching PRX/PRA/PRSTK. The new tests in this plan follow the same shape — assert `state.event_buffer.contains(...)` after dispatching Beep/Tone.

From `hp41-core/tests/fixtures/v20-autosave.json` (Plan 21-01 deliverable):
- Omits `event_buffer`. Plan 21-04's fixture-load test asserts that `state.event_buffer.is_empty()` after deserialization.

From `hp41-core/src/ops/program.rs` lines 175-181 (MAX_STEPS — resource cap for the threat model):
- `const MAX_STEPS: u64 = 1_000_000;` — guards against infinite loops. A program that emits BEEP every step is capped at 1M events maximum per run_program invocation.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1 (Wave-1): Add event_buffer field, create sound.rs module, register 2 new Op variants in all 4 places</name>
  <files>hp41-core/src/state.rs, hp41-core/src/ops/sound.rs, hp41-core/src/ops/mod.rs, hp41-core/src/ops/program.rs, hp41-cli/src/prgm_display.rs, hp41-gui/src-tauri/src/prgm_display.rs</files>
  <read_first>
    - hp41-core/src/state.rs lines 89-94 (print_buffer #[serde(default, skip)] idiom — exact analog) + lines 121-145 (CalcState::new init)
    - hp41-core/src/ops/print.rs (entire file — op_prx is the verbatim shape for op_beep)
    - hp41-core/src/ops/mod.rs lines 9-18 (pub mod block — sound goes after registers alphabetically) + lines 280-300 (Phase 21 Op section) + lines 345-560 (dispatch — Phase 21 arms section)
    - hp41-core/src/ops/program.rs lines 262-418 (execute_op — the use-import block at 263-273 and the match arms)
    - hp41-cli/src/prgm_display.rs (the existing Phase 21 section if Plans 21-01..03 have merged; otherwise the Phase 20 section as the insertion anchor)
    - hp41-gui/src-tauri/src/prgm_display.rs (parallel structure)
    - hp41-core/src/error.rs (HpError::InvalidOp reused for TONE n > 9; no new variant)
    - hp41-core/src/stack.rs (apply_lift_effect, LiftEffect::Neutral)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/src/ops/sound.rs" lines 103-145 (analog walkthrough; reuse fraction 0.90)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §Code Example 4 lines 388-407 + §State of the Art lines 410-419 (event line format)
    - .planning/milestones/v1.1/11-print-emulation/11-PLAN.md (the closest prior plan stylistically — buffer-channel pattern is established here)
  </read_first>
  <behavior>
    Behavior expectations (the integration tests in Task 2 will assert these RED → GREEN):

    - Test 1 `test_event_buffer_field_defaults_to_empty`: `let s = CalcState::new(); assert!(s.event_buffer.is_empty());`
    - Test 2 `test_load_v20_save_no_event_buffer_field`: reads `tests/fixtures/v20-autosave.json` via serde_json::from_str and asserts `s.event_buffer.is_empty()`.
    - Test 3 `test_event_buffer_skipped_on_serialize`: pre-set `s.event_buffer.push("BEEP".to_string());` then serialize via `serde_json::to_string(&s)`; assert the JSON output does NOT contain the substring `event_buffer` (the field has `#[serde(skip)]`).
    - Test 4 `test_beep_pushes_event`: `dispatch(&mut s, Op::Beep).unwrap();` then `assert_eq!(s.event_buffer, vec!["BEEP".to_string()]);`
    - Test 5 `test_beep_preserves_stack`: BEEP does NOT touch stack X/Y/Z/T or LASTX.
    - Test 6 `test_tone_n_pushes_event`: `dispatch(&mut s, Op::Tone(5)).unwrap();` then assert the last entry equals `TONE 5`. Verify for n=0 and n=9 as boundary cases too.
    - Test 7 `test_tone_out_of_range`: `let r = dispatch(&mut s, Op::Tone(10));` then `assert!(matches!(r, Err(HpError::InvalidOp)));` and `assert!(s.event_buffer.is_empty());` — guard runs BEFORE the push.
    - Test 8 `test_no_println_in_hp41_core_after_phase21` (regression sentinel): use `std::process::Command::new("grep")` with args `-rn`, the pattern `println!\|eprintln!`, and the path `hp41-core/src/` then assert the output contains 0 production lines (lines without `// test` are excluded). Acceptance for the zero-I/O invariant; an alternative simpler form is to run the grep at the verify step rather than from inside a Rust test.

    These tests live in `hp41-core/tests/phase21_sound.rs` and are written in Task 2. Task 1 implements the field + module + variants + arms so the tests can compile.
  </behavior>
  <action>
    **Step 1 — add `event_buffer: Vec<String>` field to CalcState** in `hp41-core/src/state.rs`. Mirror the print_buffer idiom (lines 89-94) verbatim: place `#[serde(default, skip)]` on the line before, then `pub event_buffer: Vec<String>` with the same naming convention. Place the field in the Phase 21 section. Doc comment: `/// HP-41 sound event buffer: BEEP and TONE n push structured event lines here. Drained by hp41-cli/hp41-gui after each dispatch — frontend plays audio. Transient, never persisted. Phase 21 (FN-SOUND-01/02).`

    Initialize `event_buffer: Vec::new(),` in `CalcState::new()`. Place it after `display_override: None,` (if Plan 21-03 merged) or in any deterministic position — order is irrelevant for serde.

    **Step 2 — create `hp41-core/src/ops/sound.rs`** as a new module. Header:
    - `//! Phase 21 sound event operations: BEEP, TONE n.`
    - `//!`
    - `//! Both ops have LiftEffect::Neutral. Output is buffered into state.event_buffer;`
    - `//! the CLI / GUI drains the buffer after each dispatch (Phase 25/26 wiring).`
    - `//! This module preserves the hp41-core zero-I/O invariant (no println!/eprintln!).`

    Imports:
    - `use crate::error::HpError;`
    - `use crate::stack::{apply_lift_effect, LiftEffect};`
    - `use crate::state::CalcState;`

    Implement 2 op functions, each named after the HP-41 op:

    - `pub fn op_beep(state: &mut CalcState) -> Result<(), HpError>` — body pushes the literal `"BEEP".to_string()` to `state.event_buffer`, then calls `apply_lift_effect(state, LiftEffect::Neutral)`, then `Ok(())`. Doc: `/// BEEP — push the literal BEEP event line. LiftEffect: Neutral.`

    - `pub fn op_tone(state: &mut CalcState, n: u8) -> Result<(), HpError>` — body: `if n > 9 { return Err(HpError::InvalidOp); }`, then `state.event_buffer.push(format!("TONE {n}"));`, then the standard `apply_lift_effect + Ok` tail. Doc: `/// TONE n — push the formatted TONE n event line. n is 0..=9; out-of-range returns InvalidOp.`

    Add a `#[cfg(test)] mod tests` at the bottom with `#[allow(clippy::unwrap_used)]`. Two inline tests:
    - `test_op_beep_pushes_literal` — fresh state, call `op_beep(&mut state).unwrap()` directly, assert event_buffer contains exactly the BEEP string.
    - `test_op_tone_out_of_range` — `op_tone(&mut state, 10)` returns InvalidOp; event_buffer remains empty.

    Keep the inline test module under ~25 LOC. Integration tests live in `tests/phase21_sound.rs` (Task 2).

    **Step 3 — register the module + add Op variants + dispatch arms + execute_op arms + prgm_display arms** — six edits across four other files:

    a) In `hp41-core/src/ops/mod.rs`:
    - Add `pub mod sound;` to the alphabetical block (between `registers` and `stack_ops`).
    - Add 2 new `Op` variants in the Phase 21 section:
      - `Beep` with doc `/// BEEP — push BEEP event to event_buffer. LiftEffect: Neutral. Phase 21 (FN-SOUND-01).`
      - `Tone(u8)` with doc `/// TONE n — push TONE n event to event_buffer (n=0..=9). LiftEffect: Neutral. Phase 21 (FN-SOUND-02).`
    - Add 2 new dispatch arms (use the qualified path consistent with Plans 21-01..03):
      - `Op::Beep => sound::op_beep(state),`
      - `Op::Tone(n) => sound::op_tone(state, n),`

    b) In `hp41-core/src/ops/program.rs`:
    - Extend the use-import block at lines 263-273 if you prefer a flat import, OR use `super::sound::op_beep` / `super::sound::op_tone` inline (consistent with the pattern Plan 21-02 / 21-03 use). Pick the smaller diff.
    - Add 2 new execute_op arms (placed in the Phase 21 section, BEFORE the catch-all at lines 410-417):
      - `Op::Beep => super::sound::op_beep(state),`
      - `Op::Tone(n) => super::sound::op_tone(state, n),`
    - Do NOT add Beep or Tone to the catch-all — they are normal ops (not programming-control), so they belong in the dispatched arms.

    c) In `hp41-cli/src/prgm_display.rs`:
    - Add 2 new op_display_name arms in the Phase 21 section (after the display-control arms if Plan 21-03 has merged):
      - `Op::Beep => "BEEP".to_string(),`
      - `Op::Tone(n) => format!("TONE {n}"),`

    d) In `hp41-gui/src-tauri/src/prgm_display.rs`:
    - Add the SAME 2 arms (byte-identical). Both copies must match.

    **SC-4 invariant gate (reminder):** Only `op_display_name` arms are added to `hp41-gui/src-tauri/src/`. No new `fn op_*` / `flush_entry_*` / `format_hpnum` body. Verify with the stricter grep documented in CLAUDE.md.

    After this task, `just build` (and `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` for the nested workspace) MUST all exit 0 — all four compile-time gates of the 4-place rule close.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -B1 'pub event_buffer: Vec<String>' hp41-core/src/state.rs | grep -c '#\[serde(default, skip)\]'` returns 1.
    - Source assertion: `grep -c "event_buffer: Vec::new()," hp41-core/src/state.rs` returns 1.
    - File assertion: `test -f hp41-core/src/ops/sound.rs` succeeds.
    - Source assertion: `grep -cE "^pub fn (op_beep|op_tone)" hp41-core/src/ops/sound.rs` returns 2.
    - Source assertion: `grep -n "pub mod sound;" hp41-core/src/ops/mod.rs` returns exactly 1 line.
    - Source assertion: `grep -cE "^\s+(Beep|Tone\(u8\)),?$" hp41-core/src/ops/mod.rs` returns 2.
    - Source assertion: `grep -E "Op::(Beep|Tone\(n\)) =>" hp41-core/src/ops/mod.rs` returns ≥ 2 (dispatch arms).
    - Source assertion: `grep -E "Op::(Beep|Tone\(n\)) =>" hp41-core/src/ops/program.rs` returns ≥ 2 (execute_op arms).
    - Source assertion: `grep -E "Op::(Beep|Tone\(n\)) =>" hp41-cli/src/prgm_display.rs` returns 2.
    - Source assertion: `grep -E "Op::(Beep|Tone\(n\)) =>" hp41-gui/src-tauri/src/prgm_display.rs` returns 2.
    - Source assertion (idiom compliance — strictest check for this plan): `grep -rn 'println!\|eprintln!' hp41-core/src/ | grep -v '^\s*//' | grep -v '#\[cfg(test)\]'` returns 0 lines — the zero-I/O invariant is preserved.
    - SC-4 invariant assertion: `grep -rnE 'fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches.
    - Test command: `just build` exits 0 (workspace build covers hp41-core and hp41-cli).
    - Test command: `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0 (hp41-gui is a nested standalone workspace).
    - Test command: `just test-core --lib ops::sound::tests` exits 0 — inline unit tests pass.
    - Lint command: `just lint` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>just build &amp;&amp; cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml &amp;&amp; just test-core --lib ops::sound::tests &amp;&amp; just lint</automated>
  </verify>
  <done>event_buffer field exists with #[serde(default, skip)]; sound.rs module exists with op_beep + op_tone + inline tests; 2 new Op variants land in all 4 places (enum + dispatch + execute_op + both prgm_display.rs copies); zero-I/O invariant preserved; SC-4 invariant preserved; clippy green; all crates build cleanly.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2 (Wave-1): Create hp41-core/tests/phase21_sound.rs with FN-SOUND-01/02 integration tests + v2.0 fixture load + no-println sentinel</name>
  <files>hp41-core/tests/phase21_sound.rs</files>
  <read_first>
    - hp41-core/tests/phase20_math.rs (precedent for integration test file shape)
    - hp41-core/tests/phase21_flags.rs (Plan 21-01 deliverable — same Phase 21 test-file pattern)
    - hp41-core/tests/print_tests.rs (precedent for `assert!(state.print_buffer.contains(...))` after dispatching PRX/PRA — Plan 21-04 mirrors this for event_buffer)
    - hp41-core/src/ops/sound.rs (Task 1 deliverable — op_beep + op_tone)
    - hp41-core/src/ops/mod.rs (Op::Beep / Op::Tone(u8) variants from Task 1)
    - hp41-core/tests/fixtures/v20-autosave.json (Plan 21-01 fixture)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §"Phase Requirements → Test Map" lines 520-525 (FN-SOUND-01/02 test list)
    - .planning/phases/21-flags-display-control-sound/21-VALIDATION.md row 21-04-01
  </read_first>
  <action>
    Create `hp41-core/tests/phase21_sound.rs` with the file head:
    - `//! Integration tests for Phase 21 Plan 04 (Sound event channel: BEEP / TONE n).`
    - `//!`
    - `//! Covers FN-SOUND-01/02 plus the zero-I/O invariant regression sentinel.`
    - `#![allow(clippy::unwrap_used)]`
    - `use hp41_core::ops::{dispatch, Op};`
    - `use hp41_core::{CalcState, HpError};`

    Implement the 8 tests listed in Task 1's `<behavior>` block (Tests 1-8). Each test is a single `#[test] fn test_<name>()`.

    For Test 2 (`test_load_v20_save_no_event_buffer_field`), use `let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap(); let s: CalcState = serde_json::from_str(&json).unwrap(); assert!(s.event_buffer.is_empty());`. This depends on Plan 21-01's fixture being committed first.

    For Test 3 (`test_event_buffer_skipped_on_serialize`), set `state.event_buffer.push("BEEP".to_string());` then `let json = serde_json::to_string(&state).unwrap();`, then `assert!(!json.contains("event_buffer"));`. The field has `#[serde(skip)]` so it must NEVER appear in serialized output.

    For Test 6 (`test_tone_n_pushes_event`), parameterize across n ∈ {0, 5, 9} — three separate dispatch calls, each followed by an assertion on the last entry of `state.event_buffer`. The expected line text is `format!("TONE {n}")` for each value.

    For Test 7 (`test_tone_out_of_range`), assert BOTH that the dispatch returns `Err(HpError::InvalidOp)` AND that `state.event_buffer.is_empty()` (guard runs BEFORE the push — Phase 11/12 atomicity pattern).

    For Test 8 (`test_no_println_in_hp41_core_after_phase21`), use the following approach: invoke `std::process::Command::new("grep")` with args `-rn`, the pattern `println!|eprintln!`, and the path `hp41-core/src/`. Capture stdout, count lines, and assert the count is 0. Note: this test depends on the working directory being the workspace root when `cargo test` runs (the project's standard convention). If running from a different directory the path needs adjustment — document this in a `// Note: ...` comment in the test body. Alternative simpler form: do this check via a `just` recipe in the verify step rather than from inside a Rust test.

    Use `serde_json::from_str::<CalcState>(...)` and `serde_json::to_string(&state)`. The crate dependency is already in `[dev-dependencies]` from Plan 21-01.

    Do NOT exceed ~100 LOC. Keep tests focused.

    The 8 tests will go RED on entry (compile errors because Op::Beep / Op::Tone do not exist until Task 1 lands). After Task 1 merges, ALL 8 must pass GREEN.
  </action>
  <acceptance_criteria>
    - File assertion: `test -f hp41-core/tests/phase21_sound.rs` succeeds.
    - Source assertion: `grep -c '^#\[test\]' hp41-core/tests/phase21_sound.rs` returns ≥ 7 (Test 8 may be implemented as a verify-step grep rather than a Rust test — that is acceptable; minimum 7 Rust tests).
    - Source assertion: file contains the 7+ specific test names — verify each via `grep -c "fn test_event_buffer_field_defaults_to_empty\|fn test_load_v20_save_no_event_buffer_field\|fn test_event_buffer_skipped_on_serialize\|fn test_beep_pushes_event\|fn test_beep_preserves_stack\|fn test_tone_n_pushes_event\|fn test_tone_out_of_range" hp41-core/tests/phase21_sound.rs` returns ≥ 7.
    - Test command: `just test-core --test phase21_sound` exits 0 — all tests pass GREEN.
    - Test command: `just test-core` exits 0 — full hp41-core test suite stays green.
    - Test command: `just ci` exits 0.
    - Test command: `just gui-ci` exits 0.
    - Source-grep sentinel (zero-I/O invariant — either as Test 8 or as the verify step): `grep -rn 'println!\|eprintln!' hp41-core/src/ | grep -v '^\s*//'` returns 0 production lines.
    - Coverage assertion: `cargo llvm-cov clean --workspace && cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0.
    - Behavior assertion: Test 7 asserts BOTH `Err(HpError::InvalidOp)` AND `event_buffer.is_empty()` after a TONE 10 attempt — the guard atomicity is verified.
  </acceptance_criteria>
  <verify>
    <automated>just test-core --test phase21_sound &amp;&amp; just ci &amp;&amp; just gui-ci &amp;&amp; ! grep -rn 'println!\|eprintln!' hp41-core/src/ | grep -v '^\s*//'</automated>
  </verify>
  <done>hp41-core/tests/phase21_sound.rs exists with ≥ 7 named tests; all pass GREEN; v2.0 fixture backward-compat (no event_buffer) verified; serde-skip verified; range guard atomicity (no push on InvalidOp) verified; zero-I/O invariant grep returns nothing; just ci and just gui-ci green; coverage ≥ 92.5%.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

Plan 21-04 extends `hp41-core` with a buffered event channel + 2 ops. The only new input surface is the `n: u8` operand on `Op::Tone(n)`.

| Boundary | Description |
|----------|-------------|
| `n: u8` in `Op::Tone(n)` | Bounded numeric input; range guard `if n > 9 return InvalidOp` BEFORE any state mutation. The push to event_buffer happens ONLY on the valid branch — Phase 11/12 atomicity pattern. |
| `state.event_buffer` growth in tight loops | Capped by run_loop's MAX_STEPS = 1_000_000 (program.rs:175-181). A program that issues BEEP in a hot loop produces at most 1M strings before run_program returns HpError::Overflow. Memory cost: ~32 bytes per "BEEP" string × 1M ≈ 32 MB worst case; acceptable for a desktop calculator emulator. |
| serde deserialization with a malformed event_buffer field | `#[serde(skip)]` means event_buffer is NEVER read from JSON — the field is always initialized via the type's Default impl (`Vec::new()`). A malicious or malformed JSON cannot inject event lines. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-W1-20 | Tampering | `op_tone` n-range validation | mitigate | Range guard `if n > 9 return Err(HpError::InvalidOp)` BEFORE the push; matches RESEARCH A3 (TONE accepts 0..=9). Test 7 verifies atomicity (no push on InvalidOp). |
| T-21-W1-21 | DoS | Event-buffer growth via programmatic infinite loop | mitigate | MAX_STEPS (1,000,000) guard in run_loop caps event_buffer at 1M entries per run_program invocation. Memory cost bounded at ~32 MB worst case. |
| T-21-W1-22 | Tampering | println! introduction in hp41-core | mitigate | Plan 21-04 explicitly tests for zero `println!` / `eprintln!` in hp41-core production code via grep (Test 8 or the verify step). CI fails the build if introduced. |
| T-21-W1-23 | Information Disclosure | Event lines as side-channel | n/a | Event lines are user-initiated BEEP/TONE outputs — no secrets, no PII. The buffer is drained by the frontend on each dispatch (Phase 25/26). |
| T-21-W1-24 | Spoofing / Repudiation / EoP | None | n/a | hp41-core single-user, no auth surface. |
</threat_model>

<verification>
## Plan-Level Verification

| Gate | Check |
|------|-------|
| FN-SOUND-01 (BEEP pushes BEEP event) | `just test-core --test phase21_sound test_beep_pushes_event` exits 0 |
| FN-SOUND-01 (BEEP preserves stack) | `test_beep_preserves_stack` |
| FN-SOUND-02 (TONE n pushes TONE n) | `test_tone_n_pushes_event` (n=0,5,9) |
| FN-SOUND-02 (TONE > 9 InvalidOp) | `test_tone_out_of_range` (asserts atomicity: no push on error) |
| Backward compat (SC-5 spillover) | `test_load_v20_save_no_event_buffer_field` |
| Serde-skip transient | `test_event_buffer_skipped_on_serialize` |
| Zero-I/O invariant (the headline gate for this plan) | `grep -rn 'println!\|eprintln!' hp41-core/src/ | grep -v '^\s*//'` returns 0 production lines |
| 4-place rule (all 4 places) | `just build` + `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` all exit 0 |
| Zero-panic gate | `just lint` exits 0 |
| SC-4 invariant | `grep -rnE 'fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches |
| Coverage non-regression | `cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0 |
| Full CI gate | `just ci` exits 0 |
| GUI CI gate | `just gui-ci` exits 0 |

## Cross-Cutting Constraints

- Both new ops have LiftEffect::Neutral; trailing `apply_lift_effect(state, LiftEffect::Neutral)` is present.
- No `println!` / `eprintln!` introduced in hp41-core — verified by Test 8 / verify-step grep.
- No new HpError variant (reuses InvalidOp per RESEARCH line 622).
- Event line format is plain text (`"BEEP"`, `"TONE n"`) per RESEARCH §State of the Art lines 410-419 — future structured-events upgrade (JSON-tagged) is non-breaking because event_buffer is `#[serde(skip)]`.
- Pitfall 4 (flag 21 / printer enable) — Plan 21-04 deliberately does NOT wire flag 21 to print operations. The flag is stored data only; existing Phase 11 PRX/PRA/PRSTK behavior is unchanged. Document this in CLAUDE.md "v2.2 additions" alongside the Plan 21-01 / 21-03 flag decisions.
</verification>

<success_criteria>
Plan 21-04 is complete when ALL of the following are true:

1. **`event_buffer: Vec<String>` field exists** on CalcState with `#[serde(default, skip)]`.
2. **`hp41-core/src/ops/sound.rs` exists** with `op_beep` + `op_tone` + inline tests.
3. **2 new `Op` variants** exist (`Beep`, `Tone(u8)`).
4. **2 new dispatch() arms** exist.
5. **2 new execute_op() arms** exist.
6. **2 new op_display_name arms** in BOTH prgm_display.rs copies, byte-identical.
7. **`hp41-core/tests/phase21_sound.rs` exists** with ≥ 7 named tests; all pass GREEN.
8. **Zero `println!` / `eprintln!`** in hp41-core production code (verified by grep).
9. **`just ci` passes**; **`just gui-ci` passes**.
10. **Coverage ≥ 92.5%** on hp41-core.
11. **SC-4 invariant grep** returns nothing.
12. **The plan SUMMARY** (`21-04-SUMMARY.md`) is committed.
</success_criteria>

<output>
After completion, create `.planning/phases/21-flags-display-control-sound/21-04-SUMMARY.md` covering:

- **Plan:** 21-04 (Sound event channel: BEEP / TONE — FN-SOUND-01/02)
- **Status:** Complete | Partial | Blocked
- **Files touched:** the 7 in `files_modified` (state.rs, mod.rs, program.rs, the new sound.rs, both prgm_display.rs copies, the new phase21_sound.rs)
- **What landed:** event_buffer field with serde(default, skip); sound.rs module with op_beep + op_tone; 2 new Op variants in 4 places; 7+ integration tests including v2.0-fixture backward-compat and zero-I/O invariant sentinel
- **Test results:** count of new tests, pass/fail breakdown of `just ci` and `just gui-ci`, coverage % (≥ 92.5%)
- **Architectural notes for CLAUDE.md "v2.2 additions":** (1) event_buffer mirrors print_buffer (transient, serde-skipped); (2) line format is plain text `BEEP` / `TONE n` — future structured-events upgrade is non-breaking; (3) flag 21 (printer enable) is intentionally NOT wired to gate PRX/PRA/PRSTK in this milestone — Phase 11 behavior unchanged; Phase 27 backlog item to revisit.
- **Followups for Phases 25 / 26:** the 2 new Op variants are awaiting `key_to_op` (Phase 25) and `key_map::resolve` + KEY_DEFS un-stubbing (Phase 26). Frontend audio playback (CLI: print event lines or play system bell; GUI: WebAudio TONE n) is Phase 25/26.

Use `/git-workflow:commit --with-skills` to commit (German Emoji Conventional Commits, English-only).
</output>
</content>
</invoke>