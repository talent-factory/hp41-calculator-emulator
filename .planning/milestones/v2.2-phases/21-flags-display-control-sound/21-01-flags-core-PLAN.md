---
phase: 21
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - justfile
  - hp41-core/src/state.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/flags.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase21_flags.rs
  - hp41-core/tests/fixtures/v20-autosave.json
autonomous: true
requirements:
  - FN-FLAG-01
tags:
  - hp41
  - rust
  - flags
  - state
  - persistence
  - backward-compat

must_haves:
  truths:
    - "CalcState carries a new `flags: u64` field with `#[serde(default)]` so a v2.0 autosave.json that lacks the field loads cleanly with flags == 0 (FN-FLAG-01, SC-5)"
    - "Op::SfFlag(n) for n in 0..=55 sets bit n of state.flags; flag indices > 55 return HpError::InvalidOp without mutating state (FN-FLAG-01, T-21-W1-01)"
    - "Op::CfFlag(n) for n in 0..=55 clears bit n of state.flags; flag indices > 55 return HpError::InvalidOp without mutating state (FN-FLAG-01, T-21-W1-01)"
    - "Round-trip property: for every n in 0..=55, after Op::SfFlag(n) followed by serde_json round-trip on CalcState, bit n is still set; flag_get(state.flags, n) returns true"
    - "Round-trip property: for every n in 0..=55, after Op::CfFlag(n) on an all-ones flag word, bit n is the only cleared bit; flag_get(state.flags, n) returns false"
    - "The fixture file `hp41-core/tests/fixtures/v20-autosave.json` (a literal v2.0-era JSON dump captured before flags existed) deserializes into CalcState via serde_json without error; the resulting state has flags == 0, display_override == None, event_buffer empty (Wave-0-shared fixture for plans 21-03 and 21-04)"
    - "A new `test-core` recipe in `justfile` accepts `*args` passthrough so executors can run `just test-core --test phase21_flags`, `just test-core --lib ops::flags::tests`, etc. — replaces direct `cargo test -p hp41-core ...` calls per CLAUDE.md (just is the sole task runner)"
    - "All 3 new Op variants (SfFlag, CfFlag, and a unit FlagTest placeholder NOT introduced here — see 21-02) land in 4 places: enum + dispatch + execute_op + BOTH prgm_display.rs copies; compile-time exhaustive matches confirm coverage"
    - "`#![deny(clippy::unwrap_used)]` gate at hp41-core/src/lib.rs is preserved — every new production line uses `?`-propagation or `.expect(\"reason\")`; test modules carry `#[allow(clippy::unwrap_used)]`"
    - "Phase 11 print_tests and Phase 20 phase20_math suites stay green — no regression in pre-existing state"

  artifacts:
    - path: "justfile"
      provides: "New `test-core *args:` recipe (Wave-0 prerequisite) forwarding to `cargo test -p hp41-core {{args}}` — enables `just test-core --test <name>` / `just test-core --lib <path>` invocations used by Plans 21-01..04 verify blocks"
      contains: "test-core *args:"
    - path: "hp41-core/src/state.rs"
      provides: "New `flags: u64` field on CalcState with #[serde(default)]; initialized to 0 in CalcState::new() (mirrors Phase 12 last_key_code precedent at state.rs:97-99)"
      contains: "pub flags: u64"
    - path: "hp41-core/src/ops/flags.rs"
      provides: "NEW module — bit helpers `flag_get`/`flag_set`/`flag_clear` (free fns taking u64, returning u64/bool), plus op layer `op_sf`/`op_cf` (mirrors registers.rs::op_sto guard+lift+Ok shape)"
      contains: "pub fn op_sf"
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Module declaration `pub mod flags;` (alphabetical insertion); 2 new Op variants (`SfFlag(u8)`, `CfFlag(u8)`); 2 new dispatch() arms forwarding to flags::op_sf / flags::op_cf"
      contains: "Op::SfFlag"
    - path: "hp41-core/src/ops/program.rs"
      provides: "Extend the `use crate::ops::...` import block in execute_op to include flags::op_sf, flags::op_cf; 2 new execute_op() arms mirroring dispatch"
      contains: "Op::SfFlag"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "2 new op_display_name arms: `Op::SfFlag(n) => format!(\"SF {n:02}\")`, `Op::CfFlag(n) => format!(\"CF {n:02}\")`"
      contains: "Op::SfFlag"
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same 2 op_display_name arms (SC-4 spirit exception — display formatter only)"
      contains: "Op::SfFlag"
    - path: "hp41-core/tests/phase21_flags.rs"
      provides: "Integration tests covering FN-FLAG-01: SF/CF round-trip 0..=55, out-of-range rejection, serde-default backward compat, fixture load, free-fn unit coverage. Plan 21-02 EXTENDS this same file with the conditional-skip tests."
      contains: "fn test_sf_sets_bit"
      min_tests: 8
    - path: "hp41-core/tests/fixtures/v20-autosave.json"
      provides: "Wave-0 backward-compat fixture: a real v2.0-era CalcState serialization captured by running the v2.0 binary OR by hand-rolling a JSON object that omits `flags`, `display_override`, `event_buffer`. Used by 21-01 (this plan), 21-03, and 21-04."
      contains: "\"stack\""

  key_links:
    - from: "hp41-core/src/ops/flags.rs::op_sf"
      to: "hp41-core/src/state.rs::CalcState::flags + hp41-core/src/ops/flags.rs::flag_set"
      via: "Range guard (`if n > 55 return InvalidOp`) followed by `state.flags = flag_set(state.flags, n)` and `apply_lift_effect(state, LiftEffect::Neutral)`. Pattern from registers.rs::op_sto (lines 15-22)."
      pattern: "state\\.flags = flag_set\\("
    - from: "hp41-core/src/ops/flags.rs::op_cf"
      to: "hp41-core/src/state.rs::CalcState::flags + hp41-core/src/ops/flags.rs::flag_clear"
      via: "Range guard followed by `state.flags = flag_clear(state.flags, n)` and Neutral lift. Mirror of op_sf."
      pattern: "state\\.flags = flag_clear\\("
    - from: "hp41-core/src/state.rs::CalcState::flags"
      to: "serde_json::from_str — fixture v20-autosave.json"
      via: "#[serde(default)] attribute lets older JSON deserialize with flags = 0"
      pattern: "#\\[serde\\(default\\)\\]\\s+pub flags: u64"
    - from: "hp41-core/src/ops/mod.rs::dispatch"
      to: "hp41-core/src/ops/program.rs::execute_op AND hp41-cli/src/prgm_display.rs AND hp41-gui/src-tauri/src/prgm_display.rs"
      via: "4-place Op-variant rule per CLAUDE.md; exhaustive matches enforce SC-5 at compile time"
      pattern: "Op::SfFlag|Op::CfFlag"
    - from: "justfile::test-core"
      to: "cargo test -p hp41-core"
      via: "*args passthrough recipe — `just test-core --test phase21_flags` expands to `cargo test -p hp41-core --test phase21_flags`. CLAUDE.md mandates `just` as sole task runner."
      pattern: "test-core \\*args:"
---

<objective>
Land the foundational flag storage subsystem for Phase 21 in `hp41-core` only. Three deliverables: (1) a new `flags: u64` field on `CalcState` with `#[serde(default)]` for save-file backward compatibility (FN-FLAG-01), (2) a new `hp41-core/src/ops/flags.rs` module carrying bit helpers (`flag_get`/`flag_set`/`flag_clear`) and the `op_sf`/`op_cf` op layer, (3) two new `Op` variants (`SfFlag(u8)`, `CfFlag(u8)`) routed through all four required landing sites per the CLAUDE.md 4-place rule.

This plan also lands the **Wave-0 backward-compatibility fixture** `hp41-core/tests/fixtures/v20-autosave.json`, which is shared by plans 21-03 (display_override) and 21-04 (event_buffer). The fixture demonstrates SC-5: a v2.0-era save file (without the three new fields) loads cleanly into v2.2.

This plan ALSO lands a Wave-0 `test-core *args:` recipe in `justfile` so all subsequent plans can invoke `just test-core --test <name>` / `just test-core --lib <path>` instead of bare `cargo test -p hp41-core ...`. CLAUDE.md mandates `just` as the sole task runner — Plans 21-01..04 verify blocks reference this new recipe.

This plan does NOT add conditional flag tests (`FS?`/`FC?`/`FS?C`/`FC?C`) — those land in plan 21-02 which builds on the `flags: u64` field defined here.

Purpose: Restore HP-41CV flag storage parity. The flag word is the foundation that every subsequent HP-41CV operation (AON/AOFF via flag 48, indirect addressing via FS?/FC? in Phase 24, etc.) builds on. CLI keyboard wiring and GUI key_map un-stubbing happen in Phase 25/26 — Phase 21 ships only the core engine.

Output: 5 modified files + 4 new files (justfile recipe addition, the flags module, the test file, the fixture). Net new line count ≈ 210 LOC including tests + recipe. All CI gates (`just lint`, `just test`, `just coverage`, `just gui-ci`) stay green. Coverage stays ≥ 92.5% (v2.1/Phase 20 baseline; v2.2 final target is 95% at Phase 27).

## Scheduling note

Although Plans 21-01, 21-03, 21-04 are semantically independent within Wave 1, they share five files (`hp41-core/src/state.rs`, `hp41-core/src/ops/mod.rs`, `hp41-core/src/ops/program.rs`, `hp41-cli/src/prgm_display.rs`, `hp41-gui/src-tauri/src/prgm_display.rs`). The execute-phase orchestrator MUST serialize them at the merge layer. The `depends_on` chain (21-03 → 21-01, 21-04 → 21-01) forces 21-01 first; the orchestrator should additionally serialize 21-03 then 21-04. Plan 21-02 is in Wave 2 (depends on 21-01) and runs after all three Wave-1 plans have merged. This plan (21-01) MUST land FIRST because (a) it adds the `justfile::test-core` recipe used by every other plan's verify block, and (b) it creates the Wave-0 fixture file consumed by Plans 21-03 and 21-04.
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
@CLAUDE.md

# Source files the executor will modify or extend
@justfile
@hp41-core/src/state.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/ops/program.rs
@hp41-core/src/ops/registers.rs
@hp41-core/src/ops/print.rs
@hp41-cli/src/prgm_display.rs
@hp41-gui/src-tauri/src/prgm_display.rs
@hp41-core/tests/phase20_math.rs

<interfaces>
<!-- Key types and contracts the executor needs. Extracted from the codebase on 2026-05-14 by direct Read. -->
<!-- Use these directly — no additional codebase exploration needed. -->

From `hp41-core/src/state.rs` (lines 52-145, condensed):
- `pub struct CalcState { pub stack: Stack, pub regs: Vec<HpNum>, pub alpha_reg: String, pub alpha_mode: bool, pub angle_mode: AngleMode, pub display_mode: DisplayMode, pub entry_buf: String, pub program: Vec<Op>, pub prgm_mode: bool, pub pc: usize, pub call_stack: Vec<usize>, pub is_running: bool, pub user_mode: bool, pub key_assignments: BTreeMap<char, String>, #[serde(default, skip)] pub print_buffer: Vec<String>, #[serde(default)] pub last_key_code: u8, #[serde(default)] pub reg_m: HpNum, #[serde(default)] pub reg_n: HpNum, #[serde(default)] pub reg_o: HpNum, #[serde(default, skip)] pub pending_card_op: Option<CardOpRequest> }`
- The precedent `#[serde(default)]` on `last_key_code: u8` (lines 97-99) is the EXACT idiom this plan reuses for the new `flags: u64` field.
- `CalcState::new()` at lines 121-145 initializes every field literally. Add `flags: 0,` immediately after `pending_card_op: None,` (or in any deterministic position — the order does not matter for serde, only for readability).

From `hp41-core/src/error.rs`:
- `pub enum HpError { Overflow, DivideByZero, InvalidOp, Domain, CallDepth, InvalidInput, AlphaData, CardData(String), OutOfRange }` — Phase 20 added `OutOfRange`. Phase 21 reuses `InvalidOp` for the flag-out-of-range guard (per 21-RESEARCH.md line 622: "Phase 21 needs no new variant, reuses InvalidOp").

From `hp41-core/src/stack.rs`:
- `pub enum LiftEffect { Enable, Disable, Neutral }`
- `pub fn apply_lift_effect(state: &mut CalcState, effect: LiftEffect)` — every Phase 21 op uses `LiftEffect::Neutral` (flags do not push to stack).

From `hp41-core/src/ops/registers.rs` (lines 13-22 — analog for op_sf/op_cf shape):
- `pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError>` follows the exact pattern this plan reuses: 1) range guard (`if reg >= 100 return InvalidOp`), 2) mutate state field, 3) `apply_lift_effect(state, LiftEffect::Neutral)`, 4) `Ok(())`. The 21-01 ops differ only in the field (`state.flags` instead of `state.regs[reg]`) and the limit (`> 55` instead of `>= 100`).

From `hp41-core/src/ops/mod.rs` (lines 1-35, condensed):
- `pub mod alpha; pub mod arithmetic; pub mod cardreader_ops; pub mod hms; pub mod math; pub mod print; pub mod program; pub mod registers; pub mod stack_ops; pub mod stats;` — declarations are alphabetical. Insert `pub mod flags;` BETWEEN `cardreader_ops` and `hms` to preserve order.
- `pub enum StoArithKind { Add, Sub, Mul, Div }` at lines 36-42 — model for sub-enum declaration (21-02 will use the same shape for `FlagTestKind`).
- `pub enum TestKind { XEqZero, XNeZero, ... }` at lines 56-70 — companion-enum model.
- `pub enum Op { ... }` starts around line 73, currently ~65 variants. Insert the 2 new Phase 21 variants AFTER the Phase 12 synthetic block (around line 282) and BEFORE the Card Reader block (line 284). Plan 21-02 / 21-03 / 21-04 will add more variants in the same section.
- `pub fn flush_entry_buf(state: &mut CalcState) -> Result<(), HpError>` at line 309 — unchanged; do NOT touch.
- `pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError>` at line 345 — the giant `match op { ... }` block. Phase 20 additions land around lines 380-403. Phase 21 additions go AFTER the Card Reader arms (around line 540, before the closing `}` of dispatch).

From `hp41-core/src/ops/program.rs` (lines 262-418 — execute_op):
- `fn execute_op(state: &mut CalcState, op: Op) -> Result<(), HpError>` at line 262. The `use crate::ops::...` import block is lines 263-273. The match arms run lines 275-417 with a final catch-all `Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_) | Op::Isg(_) | Op::Dse(_) => Err(HpError::InvalidOp)` at lines 410-417. Phase 21 adds 2 new arms BEFORE the catch-all.

From `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs`:
- Both files declare `fn op_display_name(op: &Op) -> String` with an exhaustive `match op { ... }`. The Phase 20 arms for `Op::Pi`, `Op::Rnd`, etc. live at CLI lines 42-51 and GUI parallel lines. Phase 21 arms go AFTER the Card Reader arms (CLI ~line 150, GUI ~line 195) in both copies. The 2 plans must produce byte-identical arms.

From `hp41-core/tests/phase20_math.rs` (precedent for the integration test shape):
- `#![allow(clippy::unwrap_used)]` at file head
- `use hp41_core::ops::{dispatch, Op}; use hp41_core::{CalcState, HpError, HpNum}; use rust_decimal::Decimal; use std::str::FromStr;`
- Tests instantiate `let mut state = CalcState::new();` and call `dispatch(&mut state, Op::XYZ).unwrap();` (unwrap allowed under the file-level allow).

From `hp41-core/Cargo.toml` and `[dev-dependencies]`:
- `serde_json` is already in the dev-dependency tree (used by persistence_tests). The fixture-load test in this plan calls `serde_json::from_str::<CalcState>(...)`.

From `just` recipes (current `justfile` — confirmed by direct Read on 2026-05-14):
- `just test` runs `cargo test --workspace` (workspace-wide; broad scope)
- `just lint` runs `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `just build` runs `cargo build --workspace`
- `just ci` runs `lint test coverage` (lint → test → coverage)
- `just gui-ci` runs the hp41-gui-specific pipeline (npm install + tsc --noEmit + cargo test + cargo build --release for the Tauri crate)
- `just coverage` runs `cargo llvm-cov clean --workspace` then `cargo llvm-cov --fail-under-lines 80 -p hp41-core`
- **NO `test-core` recipe currently exists** — Task 0 of this plan adds it as a Wave-0 prerequisite. Once added, `just test-core --test phase21_flags` expands to `cargo test -p hp41-core --test phase21_flags` via the `*args` passthrough.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 0 (Wave-0): Add `test-core *args:` recipe to `justfile` so subsequent plans can invoke `just test-core --test <name>` / `just test-core --lib <path>`</name>
  <files>justfile</files>
  <read_first>
    - justfile (entire file — 84 lines as of 2026-05-14; note the existing recipe style, indentation, and the `*args` convention used by similar pattern recipes)
    - CLAUDE.md §Tech Stack / Quality Gates — "`just` — sole task runner; all build/test/lint/run/ci targets are `just` recipes. Never call `cargo` directly in CI or docs."
    - .planning/phases/21-flags-display-control-sound/21-VALIDATION.md (note the `just test-core --test <name>` invocations referenced throughout)
  </read_first>
  <action>
    Append a new recipe to `justfile`. Place it AFTER the existing `test:` recipe (around line 19-21) and BEFORE the `lint:` recipe (line 23) — keeps the test-related recipes grouped together. The recipe is:

    ```
    # Run hp41-core tests with optional filter args (e.g. `just test-core --test phase21_flags`)
    test-core *args:
    	cargo test -p hp41-core {{args}}
    ```

    **Indentation convention:** The existing recipes use a TAB character for the command line (see `cargo test --workspace` on line 20 — that line begins with a literal tab, not spaces). Match that exactly. The leading `*args:` parameter syntax is `just`'s native variadic-args passthrough — anything after `just test-core` is forwarded into `{{args}}` in the command body. This matches the rationale documented in the just manual under "Recipe parameters".

    Examples of what the new recipe enables (these are the invocations Plans 21-01..04 use in their `<verify>` blocks):
    - `just test-core --test phase21_flags` → `cargo test -p hp41-core --test phase21_flags`
    - `just test-core --test phase21_display` → `cargo test -p hp41-core --test phase21_display`
    - `just test-core --test phase21_sound` → `cargo test -p hp41-core --test phase21_sound`
    - `just test-core --lib ops::flags::tests` → `cargo test -p hp41-core --lib ops::flags::tests`
    - `just test-core --lib ops::display_ops::tests` → `cargo test -p hp41-core --lib ops::display_ops::tests`
    - `just test-core` (no args) → `cargo test -p hp41-core` (runs all hp41-core tests)

    Do NOT modify any other recipe in the file. Do NOT change the existing `test:` recipe (which runs `cargo test --workspace` — a different scope).

    After saving, run `just --list` once to confirm the recipe is registered. The output should now include `test-core *args` in the list of recipes.

    This task is a Wave-0 prerequisite: it must merge BEFORE Tasks 1-5 of this plan (and before any Wave-1 plan that calls `just test-core ...`). The execute-phase orchestrator should treat this as the first commit of the phase.
  </action>
  <acceptance_criteria>
    - File assertion: `grep -nE '^test-core \*args:' justfile` returns exactly 1 line.
    - Source assertion: `grep -A1 'test-core \*args:' justfile | grep -c 'cargo test -p hp41-core'` returns 1 (the recipe body forwards to cargo).
    - Source assertion: the recipe body uses `{{args}}` passthrough — `grep -c '{{args}}' justfile` returns ≥ 1.
    - Tooling assertion: `just --list 2>&1 | grep -cE '^\s+test-core'` returns ≥ 1 (just recognizes the recipe).
    - Behavior assertion: `just test-core --list` (or any non-error subcommand to cargo test) exits 0; alternatively `just test-core --help 2>&1 | head -1` does not error out from just itself (cargo's own error is acceptable — we are checking that just dispatches).
    - No regression: `grep -c '^test:' justfile` still returns 1 (the original workspace-wide test recipe is untouched).
  </acceptance_criteria>
  <verify>
    <automated>grep -nE '^test-core \*args:' justfile &amp;&amp; just --list 2>&amp;1 | grep -qE '^\s+test-core'</automated>
  </verify>
  <done>justfile contains a `test-core *args:` recipe forwarding to `cargo test -p hp41-core {{args}}`; `just --list` shows it; the recipe enables all `just test-core --test <name>` / `just test-core --lib <path>` invocations used by Plans 21-01..04 verify blocks.</done>
</task>

<task type="auto">
  <name>Task 1 (Wave-0): Capture the v2.0-era backward-compat fixture v20-autosave.json</name>
  <files>hp41-core/tests/fixtures/v20-autosave.json</files>
  <read_first>
    - hp41-core/src/state.rs lines 52-145 (CalcState fields currently — note which carry #[serde(default)] vs. #[serde(default, skip)] vs. plain)
    - hp41-core/src/persistence.rs (if it exists in hp41-cli — search for `save_state` / `load_state`); otherwise inspect how `serde_json::to_string_pretty(&state)` is used in any persistence test
    - hp41-core/tests/numerical_accuracy.rs first 30 lines (precedent for `#![allow(clippy::unwrap_used)]` file head)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md Pitfall 3 reference and §Pattern 3 (serde-default decision matrix)
    - .planning/phases/21-flags-display-control-sound/21-VALIDATION.md §Wave 0 Requirements line "hp41-core/tests/fixtures/v20-autosave.json"
    - CLAUDE.md §Settled Architecture Decisions / v1.1 additions — the `#[serde(default)]` clause for backward compat
  </read_first>
  <action>
    Create the directory `hp41-core/tests/fixtures/` (use the executor's `mkdir -p` via Bash) and add `v20-autosave.json`. The file is a minimal JSON object that represents a CalcState serialization captured BEFORE the Phase 21 fields (`flags`, `display_override`, `event_buffer`) were added. Two construction options — pick whichever produces the smallest, most deterministic file:

    - **Option A (recommended, deterministic):** Hand-roll the JSON object literal in the editor. Include every CalcState field that exists in the current source EXCEPT the three new Phase 21 fields. Use defaults that exercise the persistence path: a non-zero X register so the load round-trip is verifiable. Suggested fields and values (refer to state.rs:52-118 for the canonical schema):
      - `stack`: object with `x`, `y`, `z`, `t`, `lastx` (each is `HpNum` serialized as `Decimal` in the project's compact form — match the format produced by `serde_json::to_string(&HpNum::from(42))`), and `lift_enabled: true`. Use X = 42, Y = 0, Z = 0, T = 0, LASTX = 0.
      - `regs`: array of 100 zero HpNums (use the compact zero serialization — verify by serializing `vec![HpNum::zero(); 100]` once during development and pasting the result; do NOT hand-roll 100 entries from memory).
      - `alpha_reg`: ""
      - `alpha_mode`: false
      - `angle_mode`: "Deg"
      - `display_mode`: `{"Fix": 4}`
      - `entry_buf`: ""
      - `program`: []
      - `prgm_mode`: false
      - `pc`: 0
      - `call_stack`: []
      - `is_running`: false
      - `user_mode`: false
      - `key_assignments`: {}
      - `last_key_code`: 0
      - `reg_m`, `reg_n`, `reg_o`: each the compact zero HpNum
      - Do NOT include `print_buffer` or `pending_card_op` (they are `#[serde(default, skip)]` — never serialized).
      - Do NOT include `flags`, `display_override`, `event_buffer` — that is the whole point of the fixture.

    - **Option B (validation-by-construction):** Briefly check out the v2.0 tag (`git show v2.0:hp41-core/src/state.rs` — or whatever the v2.0 release commit is per ROADMAP.md milestones table) and serialize a CalcState there. Copy the resulting JSON. SKIP this option unless Option A produces a file that fails to round-trip — Option A is preferred because it is reproducible without touching tags.

    Place the file at `hp41-core/tests/fixtures/v20-autosave.json`. Do NOT include a `_comment` field inside the JSON — `CalcState` does not have a `_comment` field, so serde will REJECT the file during deserialization with "unknown field". JSON natively does not support comments. If the executor wants to document the fixture's provenance, add a separate sidecar `hp41-core/tests/fixtures/README.md` (a plain markdown file) — that is acceptable and does not affect serde. The fixture itself must be valid serde input — every key maps to a known CalcState field.

    Add the fixture to git tracking via `git add hp41-core/tests/fixtures/v20-autosave.json` (and the optional `README.md` if added). Do NOT commit yet — Task 4 of this plan emits the commit via `/git-workflow:commit --with-skills` after all artifacts are in place.

    No `cargo` or `just` build is needed in this task — the fixture is only consumed by tests in Task 4. The deliverable is just the file's existence and well-formedness. JSON validity is verified at runtime by the Task 4 test (`test_load_v20_save_no_flags_field`) which parses the file via `serde_json::from_str::<CalcState>(...)`. Successful test exit proves both that the file is valid JSON AND that it deserializes into the current schema.
  </action>
  <acceptance_criteria>
    - File assertion: `test -f hp41-core/tests/fixtures/v20-autosave.json` succeeds (exit 0).
    - Source assertion: `grep -c '"flags"' hp41-core/tests/fixtures/v20-autosave.json` returns 0 (the fixture must NOT contain a flags key — that is its purpose).
    - Source assertion: `grep -c '"display_override"' hp41-core/tests/fixtures/v20-autosave.json` returns 0.
    - Source assertion: `grep -c '"event_buffer"' hp41-core/tests/fixtures/v20-autosave.json` returns 0.
    - Source assertion: `grep -c '"stack"' hp41-core/tests/fixtures/v20-autosave.json` returns 1.
    - Source assertion: `grep -c '"last_key_code"' hp41-core/tests/fixtures/v20-autosave.json` returns 1 (sanity: the file represents the v2.0 schema which already includes last_key_code from Phase 12).
    - Source assertion (no foreign field): `grep -c '"_comment"' hp41-core/tests/fixtures/v20-autosave.json` returns 0 (no unknown fields — serde would reject the load).
    - JSON validity + schema validity: the fixture-load test added in Task 4 (`test_load_v20_save_no_flags_field`) deserializes this file via `serde_json::from_str::<CalcState>(...)` without error. That test is the authoritative JSON+schema check — if it passes, the fixture is well-formed.
  </acceptance_criteria>
  <verify>
    <automated>test -f hp41-core/tests/fixtures/v20-autosave.json &amp;&amp; ! grep -qE '"(flags|display_override|event_buffer|_comment)"' hp41-core/tests/fixtures/v20-autosave.json &amp;&amp; grep -q '"stack"' hp41-core/tests/fixtures/v20-autosave.json</automated>
  </verify>
  <done>hp41-core/tests/fixtures/v20-autosave.json exists; omits the three Phase 21 fields; contains no `_comment` or other unknown fields; includes a complete pre-Phase-21 CalcState schema (stack/regs/alpha_reg/.../last_key_code/reg_m/reg_n/reg_o) so the Task 4 fixture-load test can deserialize it cleanly. (JSON+schema validity is authoritatively proven by Task 4's `test_load_v20_save_no_flags_field`.)</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2 (Wave-1): Add flags: u64 to CalcState and create hp41-core/src/ops/flags.rs with bit helpers + op_sf / op_cf</name>
  <files>hp41-core/src/state.rs, hp41-core/src/ops/flags.rs, hp41-core/src/ops/mod.rs</files>
  <read_first>
    - hp41-core/src/state.rs (entire file — note the exact attribute idiom on `last_key_code: u8` at lines 97-99 which this task mirrors for `flags: u64`)
    - hp41-core/src/ops/registers.rs lines 13-22 (`op_sto` — the exact analog shape: range-guard + state mutation + apply_lift_effect(Neutral) + Ok(()))
    - hp41-core/src/ops/print.rs (the I/O-free imports pattern — copy use statement style for the new flags.rs module)
    - hp41-core/src/ops/mod.rs lines 9-18 (the alphabetical `pub mod ...;` block — insert `pub mod flags;` between `cardreader_ops` and `hms`)
    - hp41-core/src/error.rs (confirm `HpError::InvalidOp` exists; reuse it — no new variant per RESEARCH.md line 622)
    - hp41-core/src/stack.rs (confirm `LiftEffect::Neutral` and `apply_lift_effect` exports)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/src/ops/flags.rs" (lines 32-101 — the exact analog walkthrough; reuse fraction 0.80 from op_sto)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §Code Example 1 (lines 304-342 — proposed function signatures)
    - CLAUDE.md §Settled Architecture Decisions / v1.1 additions — `#[serde(default)]` for backward compat; `#![deny(clippy::unwrap_used)]`; the 4-place Op-variant rule
  </read_first>
  <behavior>
    Behavior expectations (the integration tests in Task 4 will assert these RED→GREEN):

    - Test 1 `test_flags_field_defaults_to_zero`: `let state = CalcState::new();` → `state.flags == 0u64`.
    - Test 2 `test_load_v20_save_no_flags_field`: reading the fixture from Task 1 via `serde_json::from_str::<CalcState>(&fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap()).unwrap()` succeeds; the resulting `state.flags == 0`.
    - Test 3 `test_serde_round_trip_with_flags_set`: a CalcState with `state.flags = 0b1010_1100_u64` round-trips through `serde_json::to_string` → `from_str` and recovers the same bits.
    - Test 4 `test_flag_get_set_clear_helpers`: pure-function unit coverage of `flag_get` / `flag_set` / `flag_clear` over the n ∈ {0, 1, 31, 55} boundary set. `flag_set(0, 0)` returns `1`. `flag_set(0, 55)` returns `1u64 << 55`. `flag_clear(u64::MAX, 0)` returns `u64::MAX & !1`. `flag_get(0b101, 0)` is true, `flag_get(0b101, 1)` is false, `flag_get(0b101, 2)` is true.
    - Test 5 `test_flag_helpers_out_of_range_no_panic`: `flag_get(0, 56)` returns false (defensive). `flag_set(0, 56)` returns 0 (no-op). `flag_clear(u64::MAX, 56)` returns u64::MAX (no-op). The helpers MUST NOT panic — they are defensive free fns.
    - Test 6 `test_op_sf_sets_bit`: `dispatch(&mut state, Op::SfFlag(05)).unwrap()` → `flag_get(state.flags, 5) == true`. Stack X unchanged.
    - Test 7 `test_op_cf_clears_bit`: pre-set state.flags to all-ones, dispatch `Op::CfFlag(05)` → bit 5 is the only cleared bit.
    - Test 8 `test_op_sf_out_of_range_returns_invalid_op`: `dispatch(&mut state, Op::SfFlag(56))` returns `Err(HpError::InvalidOp)`. state.flags is unchanged (the guard runs BEFORE any mutation).
    - Test 9 `test_op_cf_out_of_range_returns_invalid_op`: symmetric.

    These tests live in `hp41-core/tests/phase21_flags.rs` and are written in Task 4. This task's job is to make tests 1–9 pass by adding the field, the module, the two Op variants, and the dispatch + execute_op + prgm_display arms.
  </behavior>
  <action>
    **Step 1 — add `pub flags: u64` to CalcState** in `hp41-core/src/state.rs`. Mirror the EXACT idiom from `last_key_code` (lines 97-99): use the doc-comment + `#[serde(default)]` attribute + `pub flags: u64` triple. Place the field after the `reg_o` line (after line 112), in a new `// ── Phase 21: Flags ─────────────────────────────────────────` section comment. The doc comment must read: `/// HP-41 flags (user flags 0-29 + system flags 30-55) packed into a single u64. Bit n = flag n. Default: 0 (all clear). Use ops::flags helpers for safe access. Phase 21 (FN-FLAG-01).`

    Then initialize `flags: 0,` in `CalcState::new()` (state.rs:121-145). Place it after `reg_o: HpNum::zero(),` and before `pending_card_op: None,` for readability.

    Do NOT touch any other CalcState field, the `Default` impl, or the `Stack` struct.

    **Step 2 — create `hp41-core/src/ops/flags.rs`** as a new module file. Header comment:
    - `//! Phase 21 flag operations: SF (set), CF (clear), and the bit-twiddling free helpers.`
    - `//!`
    - `//! Both ops have LiftEffect::Neutral. Flag indices are 0..=55; indices > 55 return InvalidOp.`

    Imports (mirror print.rs imports style — see read_first):
    - `use crate::error::HpError;`
    - `use crate::stack::{apply_lift_effect, LiftEffect};`
    - `use crate::state::CalcState;`

    Implement three `#[inline] pub fn` free helpers — each takes a `u64` flag word + `u8` index, returns the updated `u64` (or `bool` for `flag_get`):
    - `pub fn flag_get(flags: u64, n: u8) -> bool` — out-of-range (n > 55) returns false; otherwise returns `(flags & (1u64 << n)) != 0`.
    - `pub fn flag_set(flags: u64, n: u8) -> u64` — out-of-range returns `flags` unchanged; otherwise returns `flags | (1u64 << n)`.
    - `pub fn flag_clear(flags: u64, n: u8) -> u64` — out-of-range returns `flags` unchanged; otherwise returns `flags & !(1u64 << n)`.

    Implement two `pub fn` op-layer functions — both follow the exact `op_sto` shape from registers.rs:15-22:
    - `pub fn op_sf(state: &mut CalcState, n: u8) -> Result<(), HpError>` — guard `if n > 55 { return Err(HpError::InvalidOp); }`, then `state.flags = flag_set(state.flags, n);`, then `apply_lift_effect(state, LiftEffect::Neutral);`, then `Ok(())`.
    - `pub fn op_cf(state: &mut CalcState, n: u8) -> Result<(), HpError>` — same shape with `flag_clear`.

    Add a `#[cfg(test)] mod tests` at the bottom with `#[allow(clippy::unwrap_used)]` at the module head. Inline unit tests covering Test 4 and Test 5 from `<behavior>` above (free-fn coverage). Op-layer tests (Tests 6-9) live in `hp41-core/tests/phase21_flags.rs` (Task 4). The inline tests are SHORT — keep under ~30 LOC.

    **Step 3 — register the module** in `hp41-core/src/ops/mod.rs`:
    - Add `pub mod flags;` to the alphabetical block at lines 9-18 (between `cardreader_ops` and `hms`).
    - Add 2 new `Op` variants inside `pub enum Op { ... }`. Place them in a NEW section after the Card Reader block, with a leading section comment `// ── Phase 21: Flags ──────────────────────────────────────────────────────`. The variants are:
      - `SfFlag(u8)` with doc `/// SF n — set flag n (0..=55). LiftEffect: Neutral.`
      - `CfFlag(u8)` with doc `/// CF n — clear flag n (0..=55). LiftEffect: Neutral.`
    - Add 2 new dispatch arms inside `pub fn dispatch(...)` at the end of the giant match (after the Card Reader arms, before the closing brace of dispatch). Use the path form to avoid polluting the use-block. The arms are:
      - `Op::SfFlag(n) => flags::op_sf(state, n),`
      - `Op::CfFlag(n) => flags::op_cf(state, n),`

    Do NOT touch `flush_entry_buf`. Do NOT touch the PRGM-mode gate. Do NOT touch `synthetic_byte_to_op` — none of the Phase 21 ops appear in the safe synthetic-byte subset.

    Make sure `just build` succeeds after this step (or at minimum `cargo build -p hp41-core` for fast feedback). If exhaustive-match enforcement reports an error for `execute_op` in `program.rs`, that is EXPECTED — Task 3 fixes it. Compile-only intermediate state is allowed within this Task 2/3/5 chain.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -n "pub flags: u64" hp41-core/src/state.rs` returns exactly 1 line.
    - Source assertion: `grep -B1 "pub flags: u64" hp41-core/src/state.rs | grep -c '#\[serde(default)\]'` returns 1 (the attribute is on the line before the field).
    - Source assertion: `grep -c "flags: 0," hp41-core/src/state.rs` returns 1 (CalcState::new() init line).
    - File assertion: `test -f hp41-core/src/ops/flags.rs` succeeds.
    - Source assertion: `grep -cE "^pub fn (flag_get|flag_set|flag_clear|op_sf|op_cf)" hp41-core/src/ops/flags.rs` returns 5.
    - Source assertion: `grep -n "pub mod flags;" hp41-core/src/ops/mod.rs` returns exactly 1 line.
    - Source assertion: `grep -E "^\s+(SfFlag|CfFlag)\(u8\)," hp41-core/src/ops/mod.rs` returns 2 lines.
    - Source assertion: `grep -E "Op::(SfFlag|CfFlag)\(n\) =>" hp41-core/src/ops/mod.rs` returns at least 2 lines (the dispatch arms).
    - Source assertion (idiom compliance, no .unwrap()): `grep -nE '\.unwrap\(\)' hp41-core/src/ops/flags.rs` returns 0 production lines (test modules under `#[allow(clippy::unwrap_used)]` are OK; verify via `grep -B5 '\.unwrap\(\)' hp41-core/src/ops/flags.rs | grep -c 'allow(clippy::unwrap_used)'`).
    - Test command: `just build` (or `cargo build -p hp41-core`) may fail at this exact point because `execute_op` in program.rs has not yet had its arms added (Task 3) — that is acceptable. After Task 3 runs, `just build` MUST exit 0.
    - Test command: `just test-core --lib ops::flags::tests` exits 0 — the inline tests for flag_get/flag_set/flag_clear pass. This is the only test that can run in isolation after Task 2 because it does not need the dispatch path.
    - Lint command (deferred to end of Task 5): `just lint` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>just test-core --lib ops::flags::tests</automated>
  </verify>
  <done>state.rs carries `pub flags: u64` with `#[serde(default)]`; CalcState::new() initializes flags to 0; hp41-core/src/ops/flags.rs exists with the 3 helpers + 2 op fns + inline tests for the helpers; ops/mod.rs declares the new module, adds the 2 Op variants, adds the 2 dispatch arms. Inline tests pass. The crate may have an exhaustive-match compile error on execute_op until Task 3 lands — that is expected and resolved in Task 3.</done>
</task>

<task type="auto">
  <name>Task 3 (Wave-1): Add execute_op arms in program.rs for Op::SfFlag and Op::CfFlag (4-place rule, places 2)</name>
  <files>hp41-core/src/ops/program.rs</files>
  <read_first>
    - hp41-core/src/ops/program.rs lines 262-418 (the entire execute_op function — the use-import block at 263-273 and the match arms at 275-417)
    - hp41-core/src/ops/mod.rs (the dispatch arms added in Task 2 — execute_op mirrors them)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/src/ops/program.rs" lines 396-492 (the analog walkthrough — including the catch-all "programming-only" block at program.rs:410-417 which 21-02 will modify but 21-01 leaves alone)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §Pitfall 2 (the 4-place rule trap)
    - CLAUDE.md §Critical Implementation Traps (the 4-place Op-variant rule, the program.rs catch-all sentinel)
  </read_first>
  <action>
    Extend the `use crate::ops::...` import block at lines 263-273 to bring `flags::op_sf` and `flags::op_cf` into scope. Two acceptable shapes — pick the smaller diff:
    - Shape A: Add `use crate::ops::flags::{op_sf as flags_op_sf, op_cf as flags_op_cf};` to avoid shadowing any other `op_*` names. Then the arms read `Op::SfFlag(n) => flags_op_sf(state, n)`. (Aliases prevent any future grep ambiguity.)
    - Shape B (preferred — consistent with how dispatch() in mod.rs refers to the same fns): Use the qualified path inside the arms: `Op::SfFlag(n) => super::flags::op_sf(state, n)` — no new `use` line needed. Mirror this exact form used in 21-PATTERNS.md §"hp41-core/src/ops/program.rs" line 463.

    Insert 2 new arms in the `match op { ... }` block, in a Phase 21 section AFTER the Card Reader arms at lines 405-408 and BEFORE the catch-all `Op::Lbl(_) | ... => Err(HpError::InvalidOp)` block at 410-417. Add a section comment `// ── Phase 21: Flag operations ──────────────────────────────────────────`. The arms:
    - `Op::SfFlag(n) => super::flags::op_sf(state, n),`
    - `Op::CfFlag(n) => super::flags::op_cf(state, n),`

    Do NOT modify the catch-all "programming-only" block at lines 410-417. 21-02 modifies it to add `Op::FlagTest { .. }` and `Op::Prompt`; 21-01 leaves it alone.

    Do NOT touch `run_loop` at lines 177-254. The conditional skip path is added in 21-02.

    Do NOT touch `evaluate_test` at line 427.

    The compile-time exhaustive match on `Op` in execute_op will pass once these two arms are added. The compile gate from Task 2 (which deferred to here) now closes.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -E "Op::(SfFlag|CfFlag)\(n\) =>" hp41-core/src/ops/program.rs` returns at least 2 lines.
    - Source assertion (no regression to the catch-all): `grep -A 12 "// Programming ops handled by run_loop directly" hp41-core/src/ops/program.rs | grep -cE "SfFlag|CfFlag"` returns 0 (the new flag ops are NOT in the catch-all — they have proper execute_op arms).
    - Test command: `just build` exits 0 — the second compile-time gate of the 4-place rule passes.
    - Test command: `just test-core --test program_tests` exits 0 — existing programming-engine tests still pass (no regression from Phase 3 program semantics).
    - Lint command: `just lint` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>just build &amp;&amp; just lint &amp;&amp; just test-core --test program_tests</automated>
  </verify>
  <done>2 new execute_op() arms exist in hp41-core/src/ops/program.rs after the Card Reader arms and before the catch-all; compile-time exhaustive match on Op passes; existing program_tests stay green; clippy is green; the crate builds cleanly.</done>
</task>

<task type="auto">
  <name>Task 4 (Wave-1): Create hp41-core/tests/phase21_flags.rs with FN-FLAG-01 integration tests + fixture-load test</name>
  <files>hp41-core/tests/phase21_flags.rs</files>
  <read_first>
    - hp41-core/tests/phase20_math.rs (precedent for `#![allow(clippy::unwrap_used)]` at file head and the integration-test shape using `dispatch(state, Op::XYZ)`)
    - hp41-core/tests/numerical_accuracy.rs first 30 lines (precedent for serde-related test file layout if any persistence tests are referenced)
    - hp41-core/src/state.rs (CalcState fields — flags field added in Task 2)
    - hp41-core/src/ops/flags.rs (the 3 helpers + 2 op fns added in Task 2)
    - hp41-core/src/ops/mod.rs (the 2 new Op variants added in Task 2)
    - hp41-core/tests/fixtures/v20-autosave.json (the fixture added in Task 1 — Task 4 deserializes it)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §"Phase Requirements → Test Map" lines 500-527 (test list for FN-FLAG-01)
    - .planning/phases/21-flags-display-control-sound/21-VALIDATION.md §"Per-Task Verification Map" rows 21-01-01 and 21-01-02
  </read_first>
  <action>
    Create `hp41-core/tests/phase21_flags.rs` with the file head:
    - `//! Integration tests for Phase 21 Plan 01 (Flag storage + SF/CF ops).`
    - `//!`
    - `//! Covers FN-FLAG-01 (flag storage with serde-default backward compat) and the SF/CF`
    - `//! happy/error paths. Conditional-skip behavior (FN-FLAG-02) lives in Plan 21-02 and`
    - `//! is added to this same file or a parallel phase21_skip.rs — Plan 21-02 decides.`
    - `#![allow(clippy::unwrap_used)]`
    - `use hp41_core::ops::{dispatch, flags::{flag_get, flag_set, flag_clear}, Op};`
    - `use hp41_core::{CalcState, HpError};`

    Implement the 9 tests listed in Task 2's `<behavior>` block. Each test is a single `#[test] fn test_<name>()`. Key test specifics (the integration counterparts of the Task 2 unit tests):

    - `test_flags_field_defaults_to_zero` — `let s = CalcState::new(); assert_eq!(s.flags, 0u64);`
    - `test_load_v20_save_no_flags_field` — `let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap(); let s: CalcState = serde_json::from_str(&json).unwrap(); assert_eq!(s.flags, 0u64);`. ALSO assert that the loaded state's `last_key_code` and `reg_m` round-tripped (sanity: confirm the fixture is well-formed beyond the omitted-field case). This single test is the AUTHORITATIVE check that the Task-1 fixture is both valid JSON AND a valid CalcState — if it passes, the fixture is correct.
    - `test_serde_round_trip_with_flags_set` — round-trip a state with `state.flags = 0xDEADBEEFu64` (a deterministic non-trivial pattern). Use `serde_json::to_string(&state)` → `from_str` → compare. Assert the round-trip produces `0xDEADBEEFu64`.
    - `test_flag_get_set_clear_helpers_unit` — paramertize via inline loops over n ∈ {0, 1, 31, 55}; for each n, `flag_set(0, n) == 1u64 << n` and `flag_get(1u64 << n, n) == true` and `flag_clear(u64::MAX, n) == u64::MAX & !(1u64 << n)`.
    - `test_flag_helpers_out_of_range_defensive` — `flag_get(0, 56) == false`; `flag_set(42, 56) == 42`; `flag_clear(42, 100) == 42` (any n > 55 is a no-op).
    - `test_op_sf_sets_bit` — fresh state, `dispatch(&mut s, Op::SfFlag(5)).unwrap();` → assert `flag_get(s.flags, 5)` is true; assert other bits are 0.
    - `test_op_cf_clears_bit` — pre-set `s.flags = u64::MAX;` then `dispatch(&mut s, Op::CfFlag(5)).unwrap();` → assert `flag_get(s.flags, 5)` is false; assert all other bits 0..=55 are still set.
    - `test_op_sf_out_of_range_returns_invalid_op` — `let r = dispatch(&mut s, Op::SfFlag(56)); assert!(matches!(r, Err(HpError::InvalidOp))); assert_eq!(s.flags, 0u64);` (state unchanged).
    - `test_op_cf_out_of_range_returns_invalid_op` — symmetric.

    Use `serde_json::from_str::<CalcState>(&json)` for the fixture load. The `serde_json` crate is already in `[dev-dependencies]` for hp41-core — verify by reading `hp41-core/Cargo.toml`. If for some reason it is NOT, add it under `[dev-dependencies]` as `serde_json = "1"`; this is a no-risk change because the same crate is used in workspace consumers.

    The file must respect `#![deny(clippy::unwrap_used)]` indirectly via the file-level `#![allow(clippy::unwrap_used)]` (a test file standard, mirrored from phase20_math.rs).

    Do NOT exceed ~120 LOC. Keep tests focused and deterministic.

    Do NOT add the conditional-skip tests here — those land in Plan 21-02 (which decides whether to extend this file or create a sibling).
  </action>
  <acceptance_criteria>
    - File assertion: `test -f hp41-core/tests/phase21_flags.rs` succeeds.
    - Source assertion: `grep -c '^#\[test\]' hp41-core/tests/phase21_flags.rs` returns ≥ 9 (the 9 tests listed above).
    - Source assertion: file contains all 9 specific test names: `test_flags_field_defaults_to_zero`, `test_load_v20_save_no_flags_field`, `test_serde_round_trip_with_flags_set`, `test_flag_get_set_clear_helpers_unit`, `test_flag_helpers_out_of_range_defensive`, `test_op_sf_sets_bit`, `test_op_cf_clears_bit`, `test_op_sf_out_of_range_returns_invalid_op`, `test_op_cf_out_of_range_returns_invalid_op` — verify each via `grep -c "fn <name>" hp41-core/tests/phase21_flags.rs` returns 1 per name.
    - Test command: `just test-core --test phase21_flags` exits 0 — all 9 tests pass GREEN.
    - Test command: `just test-core` exits 0 — full hp41-core test suite stays green (Phase 11 print_tests, Phase 12 synthetic_tests, Phase 20 phase20_math, etc.).
    - Behavior assertion: Test 2 (`test_load_v20_save_no_flags_field`) actually deserializes the Task-1 fixture; the assertion `s.flags == 0` is reachable. If the fixture is malformed, the test fails with a serde error — investigate and fix the fixture (not the test). This test is also the JSON+schema validity gate for the fixture (replaces the need for any external JSON validator).
  </acceptance_criteria>
  <verify>
    <automated>just test-core --test phase21_flags &amp;&amp; just test-core</automated>
  </verify>
  <done>hp41-core/tests/phase21_flags.rs exists with the 9 named tests; all pass GREEN; fixture-load test confirms SC-5 (v2.0 autosave loads cleanly with flags = 0) AND simultaneously proves the fixture is valid JSON and a valid CalcState; existing hp41-core suites stay green.</done>
</task>

<task type="auto">
  <name>Task 5 (Wave-1): Add 2 op_display_name arms to BOTH prgm_display.rs copies (4-place rule, places 3 and 4)</name>
  <files>hp41-cli/src/prgm_display.rs, hp41-gui/src-tauri/src/prgm_display.rs</files>
  <read_first>
    - hp41-cli/src/prgm_display.rs (entire file — read the Phase 20 arms at lines 42-51 and the parameterized-arm pattern at lines 76-90 / 96-100 for `FmtFix(n)` / `StoArith { reg, kind }`)
    - hp41-gui/src-tauri/src/prgm_display.rs (entire file — must remain a byte-identical mirror for the new arms, per CLAUDE.md SC-4 spirit exception)
    - hp41-core/src/ops/mod.rs (the 2 new Op variants added in Task 2 — `SfFlag(u8)` and `CfFlag(u8)`)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-cli/src/prgm_display.rs" lines 496-535 (the analog walkthrough)
    - CLAUDE.md §Settled Architecture Decisions — the SC-4 invariant and the documented op_display_name duplication exception
  </read_first>
  <action>
    Add 2 new arms to the `match op { ... }` block inside `fn op_display_name(op: &Op) -> String` in BOTH files. The arms must be byte-identical across both copies (the duplication is intentional per CLAUDE.md SC-4 invariant note).

    Place the arms after the Card Reader arms in each file, in a new section with the comment `// Phase 21: Flags`. The two arms:
    - `Op::SfFlag(n) => format!("SF {n:02}"),`
    - `Op::CfFlag(n) => format!("CF {n:02}"),`

    The mnemonic spelling matches the HP-41 hardware: `SF` followed by a 2-digit flag number (`SF 05`, `SF 55`, etc.). The `n:02` width specifier is consistent with the existing `StoReg(r) => format!("STO {r:02}")` arm in both files.

    Do NOT touch `format_step()` in either file — its logic is generic over `op_display_name` so it picks up the new arms automatically.
    Do NOT touch `format_all_steps()` in the GUI copy — same reason.
    Do NOT add any new use statements — `Op` is already in scope.

    **SC-4 invariant gate:** The only addition to `hp41-gui/src-tauri/src/prgm_display.rs` is these 2 arms inside the existing `op_display_name` function. Do NOT add any new `fn op_*`, `fn flush_entry_*`, or `fn format_hpnum` body anywhere in `hp41-gui/src-tauri/src/`. Verify with the stricter grep documented in CLAUDE.md.

    Optionally, extend the existing `#[cfg(test)] mod tests` in each file with one tiny new assertion:
    - In `hp41-cli/src/prgm_display.rs`: add a test `test_display_phase21_flag_labels` that asserts `op_display_name(&Op::SfFlag(5)) == "SF 05"` and `op_display_name(&Op::CfFlag(12)) == "CF 12"`.
    - Add an equivalent test in `hp41-gui/src-tauri/src/prgm_display.rs` (under its existing test module).

    This optional inline test extension is recommended but not required — the integration tests in Task 4 already cover the runtime path; this inline test specifically guards the display-string output which has no other coverage. Keep the inline test under 8 LOC per file.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -E "Op::(SfFlag|CfFlag)\(n\) =>" hp41-cli/src/prgm_display.rs` returns exactly 2 lines.
    - Source assertion: `grep -E "Op::(SfFlag|CfFlag)\(n\) =>" hp41-gui/src-tauri/src/prgm_display.rs` returns exactly 2 lines.
    - Behavior assertion (byte-identical): `diff <(grep -E "Op::(SfFlag|CfFlag)\(n\) =>" hp41-cli/src/prgm_display.rs) <(grep -E "Op::(SfFlag|CfFlag)\(n\) =>" hp41-gui/src-tauri/src/prgm_display.rs)` returns empty output (both copies have the identical pair of arms).
    - SC-4 invariant assertion: `grep -rnE 'fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches.
    - Test command: `just build` exits 0 — the third compile-time gate of the 4-place rule passes (hp41-cli compiles as part of the workspace build).
    - Test command: `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0 — the fourth gate passes (hp41-gui is a nested standalone workspace, not part of the root workspace, so `just build` does not cover it; this direct cargo call is the only available option until a `gui-build-check` recipe is added).
    - Test command: `cargo test -p hp41-cli` exits 0; if `test_display_phase21_flag_labels` was added, it passes. (No `just` recipe currently scopes to a single crate's test run; `just test` runs the entire workspace.)
    - Test command: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0. (Same rationale — hp41-gui is a nested standalone workspace.)
    - Test command: `just ci` exits 0 — full workspace lint + test + coverage gate green.
    - Test command: `just gui-ci` exits 0 — hp41-gui Tauri build + frontend type-check stay green (the prgm_display.rs change compiles cleanly there too).
    - Coverage assertion (non-regression from Phase 20 baseline): `cargo llvm-cov clean --workspace && cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0 — hp41-core line coverage ≥ 92.5%. (Direct cargo-llvm-cov call retained: the existing `just coverage` recipe uses a fail-under threshold of 80 by default; for a strict Phase 21 non-regression check at 92.5, use the direct invocation. Future work could parameterize `just coverage` with a threshold arg.)
  </acceptance_criteria>
  <verify>
    <automated>just build &amp;&amp; cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml &amp;&amp; cargo test -p hp41-cli &amp;&amp; cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml &amp;&amp; just ci &amp;&amp; just gui-ci</automated>
  </verify>
  <done>2 op_display_name arms exist in BOTH prgm_display.rs copies (byte-identical mnemonics SF nn / CF nn); SC-4 invariant grep returns nothing; both hp41-cli and hp41-gui/src-tauri compile and test green; just ci AND just gui-ci both green; hp41-core coverage ≥ 92.5%.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

Plan 21-01 touches `hp41-core` and the two `prgm_display.rs` copies. No new I/O surface, no parser, no network. The only mutable state crossing a boundary is `CalcState`, already a hp41-core-internal value.

| Boundary | Description |
|----------|-------------|
| Programmer-supplied `n: u8` flag index in `Op::SfFlag(n)` / `Op::CfFlag(n)` | Bounded numeric input; range guard `if n > 55 return InvalidOp` runs BEFORE any state mutation. The free-fn helpers also defensively no-op out-of-range. |
| JSON deserialization of CalcState from a v2.0-era file | serde rejects type mismatches (e.g., a non-u64 `flags` field) with a structured error; `#[serde(default)]` handles the missing-field case only. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-W1-01 | Tampering | `op_sf` / `op_cf` flag-index validation | mitigate | Two-layer guard: (1) op-layer `if n > 55 return InvalidOp` BEFORE mutation; (2) free-fn helpers (`flag_set`/`flag_clear`/`flag_get`) defensively no-op for n > 55. The op layer is the primary gate; the free-fn layer is defense-in-depth. |
| T-21-W1-02 | Tampering | Save-file deserialization with malformed `flags` field | mitigate | serde returns Err on type mismatch — propagated through normal load-error path. `#[serde(default)]` only fires on missing-field, not type-mismatch. No panic path. |
| T-21-W1-03 | DoS | event-buffer overflow via programmatic SF/CF loop | accept | Phase 21-01 ops touch only `state.flags: u64` — no buffer growth. MAX_STEPS (1,000,000) guard in run_loop applies to plan 21-02's skip path; out of scope here. |
| T-21-W1-04 | Information Disclosure | None | n/a | Plan 21-01 operates on a single u64 + file I/O for the fixture (a hand-rolled JSON literal). No PII, no secrets. |
| T-21-W1-05 | Spoofing / Repudiation / EoP | None | n/a | hp41-core is single-user, single-process, no auth surface. |

**No new attack surface introduced.** All failure modes are bounded by `HpError::InvalidOp`; zero-panic policy preserved via `#![deny(clippy::unwrap_used)]`.
</threat_model>

<verification>
## Plan-Level Verification

| Gate | Check |
|------|-------|
| FN-FLAG-01 (CalcState.flags exists with serde(default)) | `grep -B1 'pub flags: u64' hp41-core/src/state.rs | grep -c '#\[serde(default)\]'` returns 1 |
| FN-FLAG-01 (backward compat per SC-5) | `just test-core --test phase21_flags test_load_v20_save_no_flags_field` exits 0 |
| 4-place rule (place 1 — enum + dispatch) | `just build` exits 0 |
| 4-place rule (place 2 — execute_op) | covered by `just build` after Task 3 |
| 4-place rule (places 3 + 4 — both prgm_display.rs) | `just build` AND `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` both exit 0 |
| Zero-panic gate | `just lint` exits 0 |
| SC-4 invariant | `grep -rnE 'fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches |
| Coverage non-regression | `cargo llvm-cov clean --workspace && cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0 |
| Full CI gate | `just ci` exits 0 |
| GUI CI gate | `just gui-ci` exits 0 |
| New test-core recipe registered | `just --list` includes `test-core *args` |
| No production .unwrap() in new code | `grep -nE '\.unwrap\(\)' hp41-core/src/ops/flags.rs` returns 0 production lines (anything under `#[allow(clippy::unwrap_used)]` test mod is fine) |

## Cross-Cutting Constraints (from ROADMAP + CLAUDE.md)

- LiftEffect declared per RESEARCH.md — `op_sf` and `op_cf` both `LiftEffect::Neutral` via the trailing `apply_lift_effect(state, LiftEffect::Neutral)` call.
- No `println!` / `eprintln!` introduced in hp41-core (Plan 21-01 adds zero I/O).
- No new HpError variant (reuses InvalidOp per RESEARCH.md line 622).
- The two `prgm_display.rs` arm sets are byte-identical (SC-4 spirit exception confirmed via `diff`).
- The wave-0 fixture file is committed as part of Plan 21-01 so plans 21-03 and 21-04 can consume it without coordination.
- The new `test-core *args:` justfile recipe is committed as part of Plan 21-01 Task 0 so Plans 21-01..04 verify blocks can invoke `just test-core --test <name>` per CLAUDE.md "just is sole task runner" mandate.
</verification>

<success_criteria>
Plan 21-01 is complete when ALL of the following are true:

1. **`test-core *args:` recipe exists** in `justfile`, forwarding to `cargo test -p hp41-core {{args}}`. `just --list` shows it. Plans 21-01..04 verify blocks can now invoke `just test-core --test <name>`.
2. **`flags: u64` field exists** on `CalcState` with `#[serde(default)]` (compile-time + grep check).
3. **`hp41-core/src/ops/flags.rs` exists** with the 3 free-fn helpers + 2 op functions + inline tests for the helpers.
4. **2 new `Op` variants** (`SfFlag(u8)`, `CfFlag(u8)`) exist in `hp41-core/src/ops/mod.rs::Op`.
5. **2 new dispatch() arms** exist in `hp41-core/src/ops/mod.rs::dispatch`.
6. **2 new execute_op() arms** exist in `hp41-core/src/ops/program.rs::execute_op` (the second compile-time gate of the 4-place rule).
7. **2 new op_display_name arms** exist in BOTH `hp41-cli/src/prgm_display.rs` AND `hp41-gui/src-tauri/src/prgm_display.rs` (the third and fourth gates).
8. **`hp41-core/tests/fixtures/v20-autosave.json` exists** as valid JSON omitting the three Phase 21 fields. Validity proven authoritatively by Task 4's `test_load_v20_save_no_flags_field`.
9. **`hp41-core/tests/phase21_flags.rs` exists** with ≥ 9 named tests covering FN-FLAG-01 + serde backward compat; all pass GREEN.
10. **`just ci` passes** — lint, test, coverage all green; hp41-core coverage ≥ 92.5% (non-regression from Phase 20).
11. **`just gui-ci` passes** — Tauri Rust build + frontend type-check stay green.
12. **SC-4 invariant grep** under `hp41-gui/src-tauri/src/` returns zero `fn op_(add|sub|...|format_hpnum)` matches.
13. **No new `.unwrap()` in production code** under hp41-core (verified by clippy under `#![deny(clippy::unwrap_used)]`).
14. **The plan SUMMARY** (`21-01-SUMMARY.md`) is committed — see `<output>`.
</success_criteria>

<output>
After completion, create `.planning/phases/21-flags-display-control-sound/21-01-SUMMARY.md` covering:

- **Plan:** 21-01 (Flags core — FN-FLAG-01, foundation for 21-02 conditional skip; ALSO lands the Wave-0 `test-core *args:` justfile recipe + v20-autosave.json fixture shared by 21-03 and 21-04)
- **Status:** Complete | Partial | Blocked
- **Files touched:** the 9 in `files_modified` (justfile, state.rs, mod.rs, program.rs, the new flags.rs, both prgm_display.rs copies, the new phase21_flags.rs, the new fixtures/v20-autosave.json)
- **What landed:** `test-core *args:` justfile recipe; `flags: u64` field with serde(default); flags.rs module with 3 helpers + op_sf/op_cf; 2 new Op variants in 4 places; Wave-0 backward-compat fixture; 9 integration tests
- **Test results:** count of new tests, pass/fail breakdown of `just ci` and `just gui-ci`, coverage % (target ≥ 92.5%)
- **Followups for plans 21-02 / 21-03 / 21-04:** 21-02 builds on the `flags: u64` field; 21-03 and 21-04 will reuse the `tests/fixtures/v20-autosave.json` fixture for their own backward-compat tests (no need to recreate it); 21-01..04 verify blocks rely on the `just test-core` recipe from Task 0
- **Followups for Phases 25 / 26:** the 2 new Op variants (`SfFlag`, `CfFlag`) are awaiting `key_to_op` (Phase 25) and `key_map::resolve` + `KEY_DEFS` un-stubbing (Phase 26) wiring — flag this in the SUMMARY so the next phase pickup is unambiguous.

Use `/git-workflow:commit --with-skills` to commit the changes (German Emoji Conventional Commits, English-only message body).
</output>
</content>
</invoke>