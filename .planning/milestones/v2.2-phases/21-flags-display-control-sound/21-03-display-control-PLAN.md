---
phase: 21
plan: 03
type: execute
wave: 1
depends_on:
  - 21-01
files_modified:
  - hp41-core/src/state.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/display_ops.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase21_display.rs
autonomous: true
requirements:
  - FN-DISP-01
  - FN-DISP-02
  - FN-DISP-03
  - FN-DISP-04
  - FN-DISP-05
tags:
  - hp41
  - rust
  - display
  - alpha
  - program-pause

must_haves:
  truths:
    - "CalcState carries a new `display_override: Option<String>` field with `#[serde(default, skip)]` so it is transient — never persisted, defaults to None (FN-DISP-01..05; matches print_buffer transient idiom)"
    - "`dispatch()` clears `state.display_override = None` at the top BEFORE the prgm_mode gate and BEFORE the giant match (Pitfall 5 — natural place is right after `flush_entry_buf` and before the prgm_mode check). VIEW/AVIEW/PROMPT write AFTER this clear and so survive their own dispatch; the NEXT op's dispatch clears the override again, matching HP-41 hardware 'VIEW shows until next key' behavior"
    - "`Op::View(u8)` (reg 0..=99) writes `Some(format_hpnum(&state.regs[reg], &state.display_mode))` to state.display_override; stack untouched; reg > 99 returns InvalidOp (FN-DISP-01)"
    - "`Op::AView` writes `Some(state.alpha_reg.chars().take(24).collect::<String>())` to state.display_override (FN-DISP-02)"
    - "`Op::Prompt` (interactive dispatch): writes ALPHA-truncated-to-24-chars to display_override (mirror of AView). Inside `run_loop`: write ALPHA to display_override AND `break` out of run_loop (state.is_running is reset to false by run_program's exit path) — full STOP/resume semantics are deferred to Phase 22 (FN-DISP-03)"
    - "`Op::Aon` sets system flag 48 via flags::flag_set; `Op::Aoff` clears flag 48 via flags::flag_clear (FN-DISP-04, RESEARCH §Pattern 6 / Pitfall 6)"
    - "`Op::Cld` sets `state.display_override = None` explicitly (HP-41 behavior: explicit clear; mostly redundant with dispatch-top clear but provides programmable clearing) (FN-DISP-05)"
    - "All 6 new Op variants (View, AView, Prompt, Aon, Aoff, Cld) land in 4 places: enum + dispatch + execute_op (Prompt is in execute_op catch-all instead) + BOTH prgm_display.rs copies"
    - "Phase 11 print_tests, Phase 20 phase20_math, and Plan 21-01 phase21_flags tests stay green — no regression"
    - "`#![deny(clippy::unwrap_used)]` preserved; SC-4 invariant grep returns nothing"

  artifacts:
    - path: "hp41-core/src/state.rs"
      provides: "New `display_override: Option<String>` field with #[serde(default, skip)] (transient, never persisted; cleared at top of dispatch)"
      contains: "pub display_override: Option<String>"
    - path: "hp41-core/src/ops/display_ops.rs"
      provides: "NEW module — op_view(reg), op_aview, op_prompt, op_aon, op_aoff, op_cld functions"
      contains: "pub fn op_view"
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Module declaration `pub mod display_ops;`; clear-at-top in dispatch (`state.display_override = None;` after flush_entry_buf, before prgm_mode gate); 6 new Op variants (View(u8), AView, Prompt, Aon, Aoff, Cld); 6 new dispatch arms"
      contains: "Op::View"
    - path: "hp41-core/src/ops/program.rs"
      provides: "5 new execute_op arms (View, AView, Aon, Aoff, Cld); 1 new run_loop arm for Op::Prompt that writes display_override and `break`s out of the loop; Op::Prompt added to the execute_op catch-all `Op::Lbl(_) | ... => Err(InvalidOp)` block (Prompt belongs only in run_loop, mirrors Op::FlagTest from Plan 21-02)"
      contains: "Op::Prompt =>"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "6 new op_display_name arms: `Op::View(r) => format!(\"VIEW {r:02}\")`, `Op::AView => \"AVIEW\".to_string()`, `Op::Prompt => \"PROMPT\".to_string()`, `Op::Aon => \"AON\".to_string()`, `Op::Aoff => \"AOFF\".to_string()`, `Op::Cld => \"CLD\".to_string()`"
      contains: "Op::AView"
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same 6 op_display_name arms (SC-4 spirit exception)"
      contains: "Op::AView"
    - path: "hp41-core/tests/phase21_display.rs"
      provides: "Integration tests covering FN-DISP-01..05: VIEW writes register, AVIEW writes ALPHA, PROMPT exits run_loop, AON sets flag 48, AOFF clears flag 48, CLD clears override, dispatch-top clear, v2.0-fixture-load backward compat"
      contains: "fn test_view_writes_register_to_override"
      min_tests: 10

  key_links:
    - from: "hp41-core/src/ops/mod.rs::dispatch"
      to: "state.display_override = None reset"
      via: "Clear-at-top inserted AFTER `flush_entry_buf(state)?;` and BEFORE the prgm_mode gate so it runs on every dispatch — VIEW/AVIEW/PROMPT write AFTER this clear so their override survives their own dispatch (RESEARCH Pitfall 5)"
      pattern: "state\\.display_override = None"
    - from: "hp41-core/src/ops/display_ops.rs::op_view"
      to: "hp41-core/src/format.rs::format_hpnum + state.regs[reg]"
      via: "Reg guard (reg >= 100 → InvalidOp) then `state.display_override = Some(format_hpnum(&state.regs[reg as usize], &state.display_mode))`"
      pattern: "state\\.display_override = Some\\(format_hpnum\\("
    - from: "hp41-core/src/ops/display_ops.rs::op_prompt"
      to: "state.alpha_reg (truncated to 24 chars)"
      via: "`state.display_override = Some(state.alpha_reg.chars().take(24).collect::<String>())` (mirror of op_pra and op_aview)"
      pattern: "alpha_reg.chars\\(\\).take\\(24\\)"
    - from: "hp41-core/src/ops/program.rs::run_loop"
      to: "Op::Prompt break"
      via: "Write display_override AND `break` (mirrors top-level RTN break at lines 192-197)"
      pattern: "Op::Prompt =>"
    - from: "hp41-core/src/ops/display_ops.rs::op_aon"
      to: "hp41-core/src/ops/flags.rs::flag_set (Plan 21-01)"
      via: "`state.flags = flag_set(state.flags, 48)` (RESEARCH §Pattern 6 — flag 48 = ALPHA auto-display, HP-42S compat)"
      pattern: "flag_set\\(state\\.flags, 48\\)"
---

<objective>
Land the HP-41 display-control subsystem in `hp41-core`: 6 new ops (`VIEW`, `AVIEW`, `PROMPT`, `AON`, `AOFF`, `CLD`), a new `display_override: Option<String>` field on `CalcState`, and the dispatch-top clear that gives the "VIEW shows until next key" semantic. PROMPT additionally pauses program execution by `break`ing out of `run_loop`.

This plan **depends on Plan 21-01** (`depends_on: ["21-01"]`) because:
1. The Wave-0 fixture file `hp41-core/tests/fixtures/v20-autosave.json` is created by Plan 21-01 Task 1 and consumed by this plan's `test_load_v20_save_no_display_override_field` integration test.
2. The Wave-0 `justfile::test-core *args:` recipe is added by Plan 21-01 Task 0 and used by this plan's verify blocks (`just test-core --test phase21_display`, etc.).
3. The `flag_set` / `flag_clear` helpers (Plan 21-01 deliverable) are imported by `op_aon` / `op_aoff` in this plan's new `display_ops.rs` module.

For compilation correctness alone (3) above would already mandate the dependency: `display_ops.rs` will not compile without `hp41-core/src/ops/flags.rs` from Plan 21-01. The fixture-consumption and justfile-recipe dependencies are additional reasons; collectively they make `21-03 → 21-01` a hard `depends_on` edge.

Purpose: Restore HP-41CV display-control parity for the 6 named ops. Frontends (CLI display rendering, GUI LCD) consume `display_override` in Phase 25/26 — Phase 21 only ships the engine.

Output: 7 modified files (state.rs, mod.rs, program.rs, the new display_ops.rs, both prgm_display.rs copies, the new phase21_display.rs). Net new line count ≈ 200 LOC including tests. All CI gates stay green; coverage stays ≥ 92.5%.

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
@CLAUDE.md

# Source files the executor will modify
@hp41-core/src/state.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/ops/program.rs
@hp41-core/src/ops/print.rs
@hp41-core/src/ops/registers.rs
@hp41-core/src/format.rs
@hp41-cli/src/prgm_display.rs
@hp41-gui/src-tauri/src/prgm_display.rs

<interfaces>
<!-- Key types and contracts the executor needs. Extracted from the codebase on 2026-05-14 by direct Read. -->

From Plan 21-01 (must be merged first per depends_on):
- `pub flags: u64` field on `CalcState` (state.rs) — referenced by `op_aon` / `op_aoff`
- `pub fn flag_set(flags: u64, n: u8) -> u64` (flags.rs) — used by op_aon
- `pub fn flag_clear(flags: u64, n: u8) -> u64` (flags.rs) — used by op_aoff
- `hp41-core/tests/fixtures/v20-autosave.json` — consumed by this plan's fixture-load test
- `justfile::test-core *args:` recipe — used by every verify block in this plan

From `hp41-core/src/state.rs` (lines 89-94 — `print_buffer` is the analog for `display_override`):
- `#[serde(default, skip)] pub print_buffer: Vec<String>` — the IDENTICAL idiom this plan reuses for `display_override: Option<String>`. The doc-comment + attribute + field shape is the model.
- The Phase 21 section in state.rs is where the field is added — same section as `flags: u64` from Plan 21-01 (merged before this plan per depends_on). Order is irrelevant for serde — only for readability.

From `hp41-core/src/format.rs`:
- `pub fn format_hpnum(n: &HpNum, mode: &DisplayMode) -> String` — VIEW uses this to render a register's value into a display string.
- After Phase 20, `pub fn round_to_display_precision(n: &HpNum, mode: &DisplayMode) -> HpNum` also exists but is NOT needed here (VIEW formats for display only; no value mutation).

From `hp41-core/src/ops/print.rs` (lines 22-30 — `op_pra` is the analog for `op_aview` and `op_prompt`):
- `pub fn op_pra(state: &mut CalcState) -> Result<(), HpError>` uses `state.alpha_reg.chars().take(24).collect::<String>()` to grab the first 24 chars (HP-41 ALPHA is max 24). The new ops use the same idiom — only the target is `state.display_override = Some(_)` instead of `state.print_buffer.push(_)`.

From `hp41-core/src/ops/registers.rs` (lines 13-22):
- `pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError>` is the analog for `op_view`: `if reg >= 100 return InvalidOp; <body>; apply_lift_effect(Neutral); Ok(())`.

From `hp41-core/src/ops/flags.rs` (Plan 21-01 deliverable):
- `pub fn flag_set(flags: u64, n: u8) -> u64` — used by `op_aon`
- `pub fn flag_clear(flags: u64, n: u8) -> u64` — used by `op_aoff`
- Module is reachable as `crate::ops::flags::{flag_set, flag_clear}`.

From `hp41-core/src/ops/program.rs` (lines 177-254 — run_loop):
- `Op::Rtn` arm at lines 192-197 — the analog for `Op::Prompt`'s `break` semantic:
  - `Op::Rtn => match state.call_stack.pop() { Some(return_pc) => state.pc = return_pc, None => break }` — `break` exits run_loop; `is_running` is reset to false by run_program's exit path at line 167.
- The `match op { ... }` block accepts new arms BEFORE the `other =>` catch-all at line 246.
- `Op::Prompt` is added to the execute_op catch-all "programming-only" block at lines 410-417 — mirrors how Plan 21-02 adds `Op::FlagTest { .. }`.

From `hp41-core/src/ops/mod.rs` (lines 345-360 — dispatch entry):
- `pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> { flush_entry_buf(state)?; /* ── Phase 3: PRGM mode recording gate (D-03) ── */ if state.prgm_mode { ... } match op { ... } }`
- The clear-at-top of dispatch goes AFTER `flush_entry_buf(state)?;` (line 346) and BEFORE the `if state.prgm_mode` gate (line 348). This makes the clear unconditional — it runs in both program-recording mode and execute mode, but VIEW/AVIEW/PROMPT only run their write path in execute mode (PRGM mode records them, so the override write happens at run-time when the program is replayed).

From `hp41-cli/src/prgm_display.rs` lines 27-100 and the GUI mirror:
- The Phase 20 arms exist (Op::Pi, Op::Rnd, etc.); the Phase 21 SfFlag/CfFlag arms from Plan 21-01 exist. The new section "// Phase 21: Display Control" comes after them.
- Both files use String returns; nullary arms are `"AVIEW".to_string()` shape; parameterized arms are `format!("VIEW {r:02}")`.

From `hp41-core/tests/fixtures/v20-autosave.json` (Plan 21-01 deliverable):
- The fixture omits `display_override` (along with `flags` and `event_buffer`). Plan 21-03's fixture-load test asserts that loading the fixture sets `display_override == None`.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1 (Wave-1): Add display_override field + create display_ops.rs module + 6 new Op variants + dispatch arms + dispatch-top clear</name>
  <files>hp41-core/src/state.rs, hp41-core/src/ops/display_ops.rs, hp41-core/src/ops/mod.rs</files>
  <read_first>
    - hp41-core/src/state.rs lines 89-94 (`print_buffer` serde-skip idiom — the exact analog for display_override) + lines 117-118 (`pending_card_op` skip idiom — also Option-typed and skipped)
    - hp41-core/src/ops/mod.rs lines 9-18 (pub mod block — alphabetical) + lines 280-300 (Phase 21 Op section) + lines 345-360 (dispatch entry — the clear-at-top insertion point)
    - hp41-core/src/ops/print.rs lines 22-30 (op_pra — alpha_reg.chars().take(24) idiom for AVIEW/PROMPT)
    - hp41-core/src/ops/registers.rs lines 13-22 (op_sto — reg-guard pattern for op_view)
    - hp41-core/src/ops/flags.rs (Plan 21-01 deliverable — flag_set / flag_clear used by op_aon / op_aoff)
    - hp41-core/src/format.rs (format_hpnum signature for op_view)
    - hp41-core/src/stack.rs (apply_lift_effect, LiftEffect — every new op uses Neutral)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/src/ops/display_ops.rs" lines 149-219 (analog walkthrough with each op implementation)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §Pattern 6 (AON/AOFF flag 48 decision) + §Pitfall 5 (display_override clear-at-top in dispatch)
  </read_first>
  <behavior>
    Behavior expectations (the integration tests in Task 4 will assert these RED→GREEN):

    - Test 1 `test_display_override_field_defaults_to_none`: `let s = CalcState::new(); assert!(s.display_override.is_none());`
    - Test 2 `test_load_v20_save_no_display_override_field`: `let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap(); let s: CalcState = serde_json::from_str(&json).unwrap(); assert!(s.display_override.is_none());`
    - Test 3 `test_display_override_skipped_on_serialize`: `let mut s = CalcState::new(); s.display_override = Some("TEST".to_string()); let json = serde_json::to_string(&s).unwrap(); assert!(!json.contains("display_override"));` — the field is `#[serde(default, skip)]` so it must NOT appear in serialized output.
    - Test 4 `test_view_writes_register_to_override`: `let mut s = CalcState::new(); s.regs[3] = HpNum::from(42i32); s.display_mode = DisplayMode::Fix(4); dispatch(&mut s, Op::View(3)).unwrap(); assert_eq!(s.display_override.as_deref(), Some("42.0000"));` (or whatever the canonical Fix(4) form for 42 produces — verify against format_hpnum precedent).
    - Test 5 `test_view_preserves_stack`: VIEW does NOT touch stack X/Y/Z/T or LASTX.
    - Test 6 `test_view_out_of_range`: `Op::View(100)` returns `Err(HpError::InvalidOp)`; display_override unchanged.
    - Test 7 `test_aview_writes_alpha_to_override`: `s.alpha_reg = "HELLO".to_string(); dispatch(&mut s, Op::AView).unwrap(); assert_eq!(s.display_override.as_deref(), Some("HELLO"));`
    - Test 8 `test_aon_sets_flag_48`: `dispatch(&mut s, Op::Aon).unwrap(); assert!(flag_get(s.flags, 48));`
    - Test 9 `test_aoff_clears_flag_48`: pre-set flags[48] = 1; `dispatch(&mut s, Op::Aoff).unwrap(); assert!(!flag_get(s.flags, 48));`
    - Test 10 `test_cld_clears_only_override`: pre-set `s.display_override = Some("X".to_string())`, `s.alpha_reg = "Y"`, `s.stack.x = HpNum::from(42)`; `dispatch(&mut s, Op::Cld).unwrap();` → `s.display_override.is_none() && s.alpha_reg == "Y" && s.stack.x.inner() == Decimal::from(42)`.
    - Test 11 `test_dispatch_top_clears_stale_override`: pre-set `s.display_override = Some("STALE".to_string()); dispatch(&mut s, Op::Enter).unwrap(); assert!(s.display_override.is_none());` (any non-display-writing op clears stale state via the dispatch-top reset).
    - Test 12 `test_prompt_exits_run_loop`: program `[Lbl("T"), AlphaAppend('H'), AlphaAppend('I'), Prompt, PushNum(99)]`; run_program returns Ok(()) and `state.stack.x` is whatever was there BEFORE the PushNum(99) — the 99 is never executed because PROMPT broke run_loop. Also `state.display_override.as_deref() == Some("HI")`.
    - Test 13 `test_prompt_inside_program_returns_quickly` (timing — RESEARCH Pitfall 3 warning sign): `let start = std::time::Instant::now(); run_program(&mut state, "T").unwrap(); assert!(start.elapsed() < std::time::Duration::from_millis(100));` — ensures PROMPT does not busy-wait.

    These tests live in `hp41-core/tests/phase21_display.rs` and are written in Task 4. Task 1 implements the field + module + variants so the tests can compile.
  </behavior>
  <action>
    **Step 1 — add `display_override: Option<String>` field to CalcState** in `hp41-core/src/state.rs`. Mirror the `print_buffer` idiom (lines 89-94): `#[serde(default, skip)] pub display_override: Option<String>`. Place it in the same Phase 21 section as `flags: u64` (Plan 21-01 must already be merged per depends_on; place it after `flags`). Doc comment: `/// HP-41 display override channel: VIEW/AVIEW/PROMPT/CLD write to this. None = render normal display. Transient — cleared at top of dispatch and never persisted (#[serde(default, skip)]). Phase 21 (FN-DISP-01..05).`

    Then initialize `display_override: None,` in `CalcState::new()`. Place it after `flags: 0,` (Plan 21-01 already added this init line per depends_on).

    **Step 2 — create `hp41-core/src/ops/display_ops.rs`** as a new module. Header:
    - `//! Phase 21 display control operations: VIEW, AVIEW, PROMPT, AON, AOFF, CLD.`
    - `//!`
    - `//! All ops have LiftEffect::Neutral. Output goes to state.display_override (Option<String>);`
    - `//! the frontend renders it. PROMPT additionally exits run_loop — see ops/program.rs.`

    Imports:
    - `use crate::error::HpError;`
    - `use crate::format::format_hpnum;`
    - `use crate::ops::flags::{flag_clear, flag_set};`
    - `use crate::stack::{apply_lift_effect, LiftEffect};`
    - `use crate::state::CalcState;`

    Implement the 6 op functions. Each ends with `apply_lift_effect(state, LiftEffect::Neutral); Ok(())` (the standard tail from registers.rs:20-21):

    - `pub fn op_view(state: &mut CalcState, reg: u8) -> Result<(), HpError>` — guard `if reg >= 100 return InvalidOp;`. Then `let val = state.regs[reg as usize].clone();` and `state.display_override = Some(format_hpnum(&val, &state.display_mode));`. Mirror the op_sto shape from registers.rs:15-22.

    - `pub fn op_aview(state: &mut CalcState) -> Result<(), HpError>` — `let alpha = state.alpha_reg.chars().take(24).collect::<String>(); state.display_override = Some(alpha);`. Mirror op_pra from print.rs:23-28.

    - `pub fn op_prompt(state: &mut CalcState) -> Result<(), HpError>` — Identical body to op_aview (write ALPHA to display_override). The PAUSE-program semantic is handled by run_loop in Task 3 (NOT by op_prompt). The interactive dispatch path runs op_prompt which writes the override; the run_loop path detects `Op::Prompt` directly and writes-then-breaks (the op_prompt function is NOT called inside run_loop, see Task 3).

    - `pub fn op_aon(state: &mut CalcState) -> Result<(), HpError>` — `state.flags = flag_set(state.flags, 48);`. Per RESEARCH §Pattern 6 (HP-42S compat: flag 48 = ALPHA auto-display). The user-visible effect ("ALPHA auto-display after every op") is a Phase 25/26 frontend concern; Phase 21 only stores the bit.

    - `pub fn op_aoff(state: &mut CalcState) -> Result<(), HpError>` — `state.flags = flag_clear(state.flags, 48);`. Mirror of op_aon.

    - `pub fn op_cld(state: &mut CalcState) -> Result<(), HpError>` — `state.display_override = None;`. Explicit clear (mostly redundant with the dispatch-top clear from Step 3, but provides a programmable way to clear).

    Add a `#[cfg(test)] mod tests` at the bottom with `#[allow(clippy::unwrap_used)]`. Inline unit tests for the 3 "interesting" ops:
    - `test_op_view_register_out_of_range_returns_invalid_op` — `Op::View(100)` returns InvalidOp; display_override unchanged.
    - `test_op_aview_truncates_to_24_chars` — set alpha_reg to a 30-char string; assert display_override contains exactly 24 chars after AView.
    - `test_op_cld_clears_only_override` — set display_override, stack.x, alpha_reg; CLD only touches display_override.

    Keep the inline test module under ~40 LOC. Integration tests for the full behavior live in `tests/phase21_display.rs` (Task 4).

    **Step 3 — register the module + add Op variants + dispatch arms + dispatch-top clear** in `hp41-core/src/ops/mod.rs`:

    a) Add `pub mod display_ops;` to the alphabetical block at lines 9-18. Place it between `cardreader_ops` and `flags` (Plan 21-01 has merged per depends_on; `display_ops` comes alphabetically before `flags`).

    b) Add 6 new `Op` variants in the Phase 21 section, AFTER the SfFlag/CfFlag variants from Plan 21-01 — order is `View(u8)`, `AView`, `Prompt`, `Aon`, `Aoff`, `Cld`. Each gets a `///` doc comment naming LiftEffect: Neutral and the requirement ID.

    c) Add 6 new dispatch arms in `dispatch()`, in a new Phase 21 Display Control section:
    - `Op::View(r) => display_ops::op_view(state, r),`
    - `Op::AView => display_ops::op_aview(state),`
    - `Op::Prompt => display_ops::op_prompt(state),`
    - `Op::Aon => display_ops::op_aon(state),`
    - `Op::Aoff => display_ops::op_aoff(state),`
    - `Op::Cld => display_ops::op_cld(state),`

    d) **Add the dispatch-top clear** — insert `state.display_override = None;` between `flush_entry_buf(state)?;` (line 346) and the prgm_mode gate (line 348). Add a comment: `// ── Phase 21 D-Pitfall-5: clear stale display override before op runs. VIEW/AVIEW/PROMPT write AFTER this line and survive their own dispatch.`

    This ordering is CRITICAL — if the clear runs AFTER the op body, VIEW immediately wipes its own override (HP-41 hardware mismatch).

    Do NOT touch the catch-all in dispatch — none of these 6 ops are programming-only (Prompt is a special case handled by run_loop in Task 3, but its dispatch arm above still routes to display_ops::op_prompt for the interactive case).

    Do NOT touch `flush_entry_buf` or `synthetic_byte_to_op`.

    The crate will fail to compile after Step 3 because (a) execute_op in program.rs is non-exhaustive (missing 5 of 6 new variants — Prompt goes to the catch-all in Task 3), and (b) the prgm_display.rs copies are non-exhaustive. Both fixed in Tasks 3 and 5 below.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -B1 'pub display_override: Option<String>' hp41-core/src/state.rs | grep -c '#\[serde(default, skip)\]'` returns 1.
    - Source assertion: `grep -c "display_override: None," hp41-core/src/state.rs` returns 1.
    - File assertion: `test -f hp41-core/src/ops/display_ops.rs` succeeds.
    - Source assertion: `grep -cE "^pub fn (op_view|op_aview|op_prompt|op_aon|op_aoff|op_cld)" hp41-core/src/ops/display_ops.rs` returns 6.
    - Source assertion: `grep -n "pub mod display_ops;" hp41-core/src/ops/mod.rs` returns exactly 1 line.
    - Source assertion: `grep -cE "^\s+(View\(u8\)|AView|Prompt|Aon|Aoff|Cld),?$" hp41-core/src/ops/mod.rs` returns 6 (the 6 Op variants).
    - Source assertion: `grep -E "Op::(View|AView|Prompt|Aon|Aoff|Cld)\s*(\(r\))?\s*=>" hp41-core/src/ops/mod.rs` returns ≥ 6.
    - Source assertion (dispatch-top clear): `grep -n "state.display_override = None;" hp41-core/src/ops/mod.rs` returns ≥ 2 lines (one in op_cld, one at dispatch top). To distinguish: `grep -B2 "state.display_override = None;" hp41-core/src/ops/mod.rs | grep -c "Pitfall-5"` returns ≥ 1 (the dispatch-top clear is annotated with the Pitfall-5 comment).
    - Source assertion (idiom compliance): `grep -nE '\.unwrap\(\)' hp41-core/src/ops/display_ops.rs | grep -v 'cfg(test)'` returns 0.
    - Test command (per W-8 — augmented with crate-build): `just build` to ensure the crate (and workspace) compiles to the extent possible after Task 1. **EXPECTED partial failure:** since Task 2 has not yet added the `execute_op` arms and Task 3 has not extended `prgm_display.rs`, the workspace build will report non-exhaustive-match errors on `execute_op` (missing 5 of 6 new variants) and on `op_display_name` in both prgm_display.rs files. This is INTENTIONAL — Task 1's verify is intentionally partial. The executor should NOT treat the non-exhaustive errors as failures at this point; they close in Task 2 (execute_op arms) and Task 3 (prgm_display arms).
    - Test command (the actually-passing check for Task 1): `just test-core --lib ops::display_ops::tests` exits 0 — inline unit tests for the 3 "interesting" ops pass. This is the ONLY full-green check that can run in isolation after Task 1 because it does not depend on the dispatch path or the exhaustive matches in execute_op / prgm_display.
    - Lint command (deferred): `just lint` will exit 0 after Tasks 2 and 3.
  </acceptance_criteria>
  <verify>
    <automated>just test-core --lib ops::display_ops::tests &amp;&amp; (just build 2>&amp;1 | tee /tmp/p21_03_t1.log; grep -qE 'non-exhaustive patterns|match arms' /tmp/p21_03_t1.log || just build)</automated>
  </verify>
  <done>display_override field exists with #[serde(default, skip)]; display_ops.rs module exists with 6 ops + inline tests; ops/mod.rs declares the module, adds 6 Op variants, 6 dispatch arms, and the dispatch-top clear at the correct location (after flush, before prgm_mode gate). The crate may have non-exhaustive-match errors elsewhere (on execute_op and on both prgm_display.rs copies); those are resolved in Tasks 2 and 3.</done>
</task>

<task type="auto">
  <name>Task 2 (Wave-1): Add Op::Prompt run_loop arm and 5 execute_op arms in ops/program.rs</name>
  <files>hp41-core/src/ops/program.rs</files>
  <read_first>
    - hp41-core/src/ops/program.rs lines 177-254 (run_loop — the Op::Rtn break precedent at lines 192-197)
    - hp41-core/src/ops/program.rs lines 262-418 (execute_op)
    - hp41-core/src/ops/display_ops.rs (Task 1 deliverable — the 6 op functions)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-core/src/ops/program.rs" lines 396-492 (the analog walkthrough; PROMPT exits via `break` and is added to the execute_op catch-all)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §Pitfall 3 (PROMPT must NOT busy-loop)
  </read_first>
  <action>
    **Step 1 — add 1 new arm in run_loop** for `Op::Prompt`. Place it BEFORE the `other =>` catch-all at line 246, AFTER the existing `Op::FlagTest` arm from Plan 21-02 (if merged) or AFTER the `Op::Dse` arm at line 244 (if 21-02 has not merged). The arm body:

    `Op::Prompt => { state.display_override = Some(state.alpha_reg.chars().take(24).collect::<String>()); break; }`

    - The `break` exits `run_loop` (mirrors the top-level RTN at lines 192-197).
    - `state.is_running` is reset to false by `run_program` at line 167.
    - Resume from PROMPT is deferred to Phase 22 (per RESEARCH A5).

    Section comment: `// ── Phase 21: PROMPT — write ALPHA to display_override + break run_loop. Full STOP/resume semantics deferred to Phase 22 (RESEARCH A5).`

    **Step 2 — add 5 new arms in execute_op** (for the non-PROMPT ops):
    Extend the `use crate::ops::...` import block (lines 263-273) to include `use crate::ops::display_ops::{op_aoff, op_aon, op_aview, op_cld, op_view};` — note `op_prompt` is intentionally NOT imported because Prompt never reaches execute_op (it is handled by run_loop's arm above AND by the execute_op catch-all below).

    Insert the 5 new arms in the match block AFTER the Plan 21-01 SfFlag/CfFlag arms (if merged) and AFTER Plan 21-02's FlagTest entry in the catch-all (if merged), and BEFORE the catch-all "programming-only" block at lines 410-417. Section comment: `// ── Phase 21: Display Control ──────────────────────────`. The arms:
    - `Op::View(r) => super::display_ops::op_view(state, r),`
    - `Op::AView => super::display_ops::op_aview(state),`
    - `Op::Aon => super::display_ops::op_aon(state),`
    - `Op::Aoff => super::display_ops::op_aoff(state),`
    - `Op::Cld => super::display_ops::op_cld(state),`

    Use the `super::display_ops::` qualified path to avoid polluting the use-block — consistent with the pattern Plan 21-02 uses for `super::flags::...`.

    **Step 3 — add `Op::Prompt` to the execute_op catch-all** at lines 410-417. Extend the chain to include `| Op::Prompt` so the catch-all reads (with Plan 21-02 also merged):
    `Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_) | Op::Isg(_) | Op::Dse(_) | Op::FlagTest { .. } | Op::Prompt => Err(HpError::InvalidOp),`

    This honors Pitfall 2 (RESEARCH): Op::Prompt is handled by run_loop directly, never by execute_op. The catch-all entry ensures explicit InvalidOp if PROMPT ever reaches execute_op (defense-in-depth).

    After this task, `just build` MUST exit 0 again (the second compile-gate of the 4-place rule closes for the 6 display ops).
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -n "Op::Prompt =>" hp41-core/src/ops/program.rs | grep -c "break"` returns ≥ 1 (the run_loop arm contains break).
    - Source assertion: `grep -E "Op::(View|AView|Aon|Aoff|Cld)\s*(\(r\))?\s*=>" hp41-core/src/ops/program.rs` returns ≥ 5 (the 5 execute_op arms; Prompt is intentionally in the catch-all).
    - Source assertion (catch-all extension): `grep -E "Op::Prompt" hp41-core/src/ops/program.rs | grep -c "InvalidOp"` returns ≥ 1 (Prompt is in the execute_op catch-all returning InvalidOp).
    - Test command: `just build` exits 0 — the exhaustive-match gate closes.
    - Test command: `just test-core --test program_tests` exits 0 — no regression in programming-engine tests.
    - Lint command: `just lint` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>just build &amp;&amp; just lint &amp;&amp; just test-core --test program_tests</automated>
  </verify>
  <done>1 new run_loop arm for Op::Prompt (writes display_override + break); 5 new execute_op arms for View/AView/Aon/Aoff/Cld; Op::Prompt added to execute_op catch-all; the crate compiles cleanly; program_tests stay green.</done>
</task>

<task type="auto">
  <name>Task 3 (Wave-1): Add 6 op_display_name arms to BOTH prgm_display.rs copies (4-place rule, places 3 + 4)</name>
  <files>hp41-cli/src/prgm_display.rs, hp41-gui/src-tauri/src/prgm_display.rs</files>
  <read_first>
    - hp41-cli/src/prgm_display.rs (the existing Phase 20 + Plan 21-01 SfFlag/CfFlag + Plan 21-02 FlagTest arms — verify the Phase 21 section comment is in place)
    - hp41-gui/src-tauri/src/prgm_display.rs (the parallel file — must mirror the CLI byte-identically)
    - hp41-core/src/ops/mod.rs (the 6 new Op variants from Task 1)
    - .planning/phases/21-flags-display-control-sound/21-PATTERNS.md §"hp41-cli/src/prgm_display.rs" lines 496-535 (Phase 21 arm walkthrough)
  </read_first>
  <action>
    Add 6 new arms to the `match op { ... }` block inside `fn op_display_name(op: &Op) -> String` in BOTH files. Place them after the Plan 21-01 SfFlag/CfFlag arms and after the Plan 21-02 FlagTest arm (if merged), in the same Phase 21 section. Section header comment: `// Phase 21: Display Control`.

    The 6 arms (byte-identical across both files):
    - `Op::View(r) => format!("VIEW {r:02}"),`
    - `Op::AView => "AVIEW".to_string(),`
    - `Op::Prompt => "PROMPT".to_string(),`
    - `Op::Aon => "AON".to_string(),`
    - `Op::Aoff => "AOFF".to_string(),`
    - `Op::Cld => "CLD".to_string(),`

    Do NOT touch `format_step()` in either file.
    Do NOT touch `format_all_steps()` in the GUI copy.
    Do NOT add new use statements — `Op` is already in scope.

    **SC-4 invariant gate:** The only addition to `hp41-gui/src-tauri/src/prgm_display.rs` is these 6 arms inside the existing `op_display_name`. No new `fn op_*` / `flush_entry_*` / `format_hpnum` body — verify with the CLAUDE.md stricter grep.

    Optionally extend the `#[cfg(test)] mod tests` in each file with `test_display_phase21_display_labels`:
    - `op_display_name(&Op::View(5)) == "VIEW 05"`
    - `op_display_name(&Op::AView) == "AVIEW"`
    - `op_display_name(&Op::Prompt) == "PROMPT"`
    - `op_display_name(&Op::Aon) == "AON"`
    - `op_display_name(&Op::Aoff) == "AOFF"`
    - `op_display_name(&Op::Cld) == "CLD"`

    Keep the test under 12 LOC per file.
  </action>
  <acceptance_criteria>
    - Source assertion: `grep -E "Op::(View\(r\)|AView|Prompt|Aon|Aoff|Cld) =>" hp41-cli/src/prgm_display.rs` returns ≥ 6.
    - Source assertion: `grep -E "Op::(View\(r\)|AView|Prompt|Aon|Aoff|Cld) =>" hp41-gui/src-tauri/src/prgm_display.rs` returns ≥ 6.
    - Behavior assertion (byte-identical): `diff <(grep -E "Op::(View|AView|Prompt|Aon|Aoff|Cld)" hp41-cli/src/prgm_display.rs | sort) <(grep -E "Op::(View|AView|Prompt|Aon|Aoff|Cld)" hp41-gui/src-tauri/src/prgm_display.rs | sort)` returns empty (modulo enum/import lines).
    - SC-4 invariant assertion: `grep -rnE 'fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches.
    - Test command: `just build` exits 0 (workspace build covers hp41-cli).
    - Test command: `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0 (hp41-gui is a nested standalone workspace).
    - Test command: `cargo test -p hp41-cli` exits 0.
    - Test command: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0.
  </acceptance_criteria>
  <verify>
    <automated>just build &amp;&amp; cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml &amp;&amp; cargo test -p hp41-cli &amp;&amp; cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml</automated>
  </verify>
  <done>6 op_display_name arms exist in BOTH prgm_display.rs copies (byte-identical); SC-4 invariant preserved; both crates compile and test green.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 4 (Wave-1): Create hp41-core/tests/phase21_display.rs with FN-DISP-01..05 integration tests + dispatch-top clear test + PROMPT-exits-run_loop test</name>
  <files>hp41-core/tests/phase21_display.rs</files>
  <read_first>
    - hp41-core/tests/phase20_math.rs (precedent for integration test file shape)
    - hp41-core/tests/phase21_flags.rs (Plan 21-01 precedent — same test-file shape for Phase 21 tests)
    - hp41-core/src/ops/display_ops.rs (Task 1 deliverable — the op functions under test)
    - hp41-core/src/ops/mod.rs (the dispatch-top clear and 6 new dispatch arms)
    - hp41-core/src/ops/program.rs (the run_loop Op::Prompt arm)
    - hp41-core/tests/fixtures/v20-autosave.json (Plan 21-01 fixture — Task 4 reads it for the backward-compat test)
    - .planning/phases/21-flags-display-control-sound/21-RESEARCH.md §"Phase Requirements → Test Map" lines 513-519 (FN-DISP-01..05 test list)
    - .planning/phases/21-flags-display-control-sound/21-VALIDATION.md row 21-03-01
  </read_first>
  <action>
    Create `hp41-core/tests/phase21_display.rs` with the file head:
    - `//! Integration tests for Phase 21 Plan 03 (Display Control: VIEW/AVIEW/PROMPT/AON/AOFF/CLD).`
    - `//!`
    - `//! Covers FN-DISP-01..05 plus the dispatch-top clear (Pitfall 5), the v2.0 backward-compat`
    - `//! load (SC-5 spillover from Plan 21-01), and the PROMPT-exits-run_loop semantic (Pitfall 3).`
    - `#![allow(clippy::unwrap_used)]`
    - `use hp41_core::ops::{dispatch, flags::flag_get, program::run_program, Op};`
    - `use hp41_core::{CalcState, DisplayMode, HpError, HpNum};`
    - `use rust_decimal::Decimal;`
    - `use std::str::FromStr;`

    Implement the 13 tests listed in Task 1's `<behavior>` block (Tests 1-13). Each test is a single `#[test] fn test_<name>()`. For Test 12 (PROMPT exits run_loop), set `state.program = vec![Op::Lbl("T".to_string()), Op::AlphaAppend('H'), Op::AlphaAppend('I'), Op::Prompt, Op::PushNum(HpNum::from(Decimal::from_str("99").unwrap()))];` and `state.alpha_reg.clear();` (since AlphaAppend appends), then call `run_program(&mut state, "T").unwrap();`. Assert state.display_override.as_deref() == Some("HI") AND state.stack.x.inner() != Decimal::from_str("99").unwrap() (the PushNum(99) is never reached).

    For Test 13 (timing — Pitfall 3), wrap the run_program call with `let start = std::time::Instant::now();` and assert `start.elapsed() < std::time::Duration::from_millis(100)` AFTER the run. This is a sentinel for the busy-wait anti-pattern.

    Include a fixture-load test that demonstrates SC-5 spillover for the display_override field:
    - `test_load_v20_save_no_display_override_field` — reads `hp41-core/tests/fixtures/v20-autosave.json` (Plan 21-01 deliverable) and asserts `state.display_override.is_none()` after deserialization. This complements Plan 21-01's `test_load_v20_save_no_flags_field`.

    Use `serde_json::from_str::<CalcState>(...)`. The crate dependency is already in `[dev-dependencies]` from Plan 21-01.

    Do NOT exceed ~150 LOC. Keep tests focused.

    The 13 tests will go RED on entry (some compile errors because Op::View / Op::AView / etc. do not exist until Tasks 1-3 merge). After Tasks 1, 2, and 3 land, ALL 13 must pass GREEN.
  </action>
  <acceptance_criteria>
    - File assertion: `test -f hp41-core/tests/phase21_display.rs` succeeds.
    - Source assertion: `grep -c '^#\[test\]' hp41-core/tests/phase21_display.rs` returns ≥ 13.
    - Source assertion: file contains all 13 specific test names — verify each via `grep -c "fn test_display_override_field_defaults_to_none\|fn test_load_v20_save_no_display_override_field\|fn test_display_override_skipped_on_serialize\|fn test_view_writes_register_to_override\|fn test_view_preserves_stack\|fn test_view_out_of_range\|fn test_aview_writes_alpha_to_override\|fn test_aon_sets_flag_48\|fn test_aoff_clears_flag_48\|fn test_cld_clears_only_override\|fn test_dispatch_top_clears_stale_override\|fn test_prompt_exits_run_loop\|fn test_prompt_inside_program_returns_quickly" hp41-core/tests/phase21_display.rs` returns ≥ 13.
    - Test command: `just test-core --test phase21_display` exits 0 — all 13 tests pass GREEN.
    - Test command: `just test-core` exits 0 — full hp41-core test suite stays green.
    - Test command: `just ci` exits 0.
    - Test command: `just gui-ci` exits 0.
    - Coverage assertion: `cargo llvm-cov clean --workspace && cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0.
    - Behavior assertion (PROMPT timing — Pitfall 3): the `test_prompt_inside_program_returns_quickly` test asserts elapsed < 100ms; the test's body and assertion are present.
  </acceptance_criteria>
  <verify>
    <automated>just test-core --test phase21_display &amp;&amp; just ci &amp;&amp; just gui-ci</automated>
  </verify>
  <done>hp41-core/tests/phase21_display.rs exists with the 13 named tests; all pass GREEN; PROMPT-exits-run_loop and dispatch-top-clear behaviors verified; v2.0 fixture backward-compat (no display_override) verified; just ci and just gui-ci green; coverage ≥ 92.5%.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

Plan 21-03 extends `hp41-core` with a display channel + 6 ops. The only new input surface is the `reg: u8` operand on `Op::View(reg)`.

| Boundary | Description |
|----------|-------------|
| `reg: u8` in `Op::View(reg)` | Bounded numeric input; range guard `if reg >= 100 return InvalidOp` BEFORE any state read. Mirrors registers.rs::op_sto convention. |
| `state.alpha_reg.chars().take(24)` in `op_aview` / `op_prompt` | Truncates to 24 chars max; HP-41 ALPHA register is documented as 24 chars but defensive truncation protects against larger-than-expected input. |
| run_loop break on Op::Prompt | The `break` returns control to run_program (line 165-167) which clears is_running and returns Ok. No infinite-loop risk; covered by Test 13 timing sentinel. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-21-W1-10 | Tampering | `op_view` register-index validation | mitigate | `if reg >= 100 return Err(HpError::InvalidOp)` BEFORE the index access — defense against panic on out-of-bounds Vec indexing. |
| T-21-W1-11 | Tampering | Malformed alpha_reg longer than 24 chars | mitigate | `.chars().take(24).collect::<String>()` truncates defensively; matches existing op_pra precedent at print.rs:25. |
| T-21-W1-12 | DoS | PROMPT busy-loop in run_loop | mitigate | PROMPT exits via `break` (NOT a wait-loop); covered by Test 13 timing sentinel (asserts elapsed < 100ms). |
| T-21-W1-13 | DoS | Op::AView with massive alpha_reg | mitigate | `chars().take(24)` caps at 24 chars; HpError handling unchanged. |
| T-21-W1-14 | Information Disclosure | None | n/a | All ops operate on local CalcState; no PII, no file I/O. |
| T-21-W1-15 | Spoofing / Repudiation / EoP | None | n/a | hp41-core single-user, no auth surface. |
</threat_model>

<verification>
## Plan-Level Verification

| Gate | Check |
|------|-------|
| FN-DISP-01 (VIEW writes register to display_override) | `just test-core --test phase21_display test_view_writes_register_to_override` exits 0 |
| FN-DISP-01 (VIEW preserves stack) | `test_view_preserves_stack` |
| FN-DISP-02 (AVIEW writes ALPHA) | `test_aview_writes_alpha_to_override` |
| FN-DISP-03 (PROMPT exits run_loop) | `test_prompt_exits_run_loop` AND `test_prompt_inside_program_returns_quickly` (Pitfall 3) |
| FN-DISP-04 (AON sets flag 48 / AOFF clears flag 48) | `test_aon_sets_flag_48` AND `test_aoff_clears_flag_48` |
| FN-DISP-05 (CLD clears override; stack/ALPHA untouched) | `test_cld_clears_only_override` |
| Backward compat (SC-5 spillover) | `test_load_v20_save_no_display_override_field` exits 0 |
| Pitfall 5 (dispatch-top clear) | `test_dispatch_top_clears_stale_override` AND `test_display_override_skipped_on_serialize` |
| 4-place rule (all 4 places) | `just build` + `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` all exit 0 |
| Zero-panic gate | `just lint` exits 0 |
| SC-4 invariant | `grep -rnE 'fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches |
| Coverage non-regression | `cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0 |
| Full CI gate | `just ci` exits 0 |
| GUI CI gate | `just gui-ci` exits 0 |

## Cross-Cutting Constraints

- All 6 new ops have LiftEffect::Neutral; the trailing `apply_lift_effect(state, LiftEffect::Neutral)` is present in every function.
- No `println!` / `eprintln!` introduced in hp41-core.
- No new HpError variant (reuses InvalidOp).
- AON/AOFF target flag 48 per RESEARCH §Pattern 6 (HP-42S compat). Document in CLAUDE.md "v2.2 additions" alongside Plan 21-01 flag storage decisions.
- PROMPT exits run_loop via `break`; full STOP/resume is Phase 22 territory (RESEARCH A5).
- The dispatch-top clear runs BEFORE the prgm_mode gate so it is unconditional; VIEW/AVIEW/PROMPT write AFTER it and survive their own dispatch.
</verification>

<success_criteria>
Plan 21-03 is complete when ALL of the following are true:

1. **`display_override: Option<String>` field exists** on CalcState with `#[serde(default, skip)]`.
2. **`hp41-core/src/ops/display_ops.rs` exists** with the 6 op functions + inline tests.
3. **6 new `Op` variants** exist (View(u8), AView, Prompt, Aon, Aoff, Cld).
4. **6 new dispatch() arms** exist.
5. **Dispatch-top clear** (`state.display_override = None`) is inserted between flush_entry_buf and the prgm_mode gate.
6. **1 new run_loop arm** for `Op::Prompt` writes display_override AND `break`s.
7. **5 new execute_op arms** for View/AView/Aon/Aoff/Cld; `Op::Prompt` is in the execute_op catch-all.
8. **6 new op_display_name arms** in BOTH prgm_display.rs copies, byte-identical.
9. **`hp41-core/tests/phase21_display.rs` exists** with ≥ 13 tests; all pass GREEN.
10. **`just ci` passes**; **`just gui-ci` passes**.
11. **Coverage ≥ 92.5%** on hp41-core.
12. **SC-4 invariant grep** returns nothing.
13. **PROMPT timing sentinel** test asserts run_program returns in < 100ms (Pitfall 3 guard).
14. **The plan SUMMARY** (`21-03-SUMMARY.md`) is committed.
</success_criteria>

<output>
After completion, create `.planning/phases/21-flags-display-control-sound/21-03-SUMMARY.md` covering:

- **Plan:** 21-03 (Display control: VIEW / AVIEW / PROMPT / AON / AOFF / CLD — FN-DISP-01..05)
- **Status:** Complete | Partial | Blocked
- **Files touched:** the 7 in `files_modified` (state.rs, mod.rs, program.rs, the new display_ops.rs, both prgm_display.rs copies, the new phase21_display.rs)
- **What landed:** display_override field with serde-skip; display_ops.rs module with 6 ops; dispatch-top clear; PROMPT run_loop break; 6 new Op variants in 4 places; 13 integration tests including v2.0-fixture backward-compat
- **Test results:** count of new tests, pass/fail breakdown of `just ci` and `just gui-ci`, coverage % (≥ 92.5%)
- **Architectural notes for CLAUDE.md "v2.2 additions":** (1) display_override is core-managed — cleared at top of dispatch, written by VIEW/AVIEW/PROMPT/CLD; (2) AON/AOFF target system flag 48 (HP-42S compat); (3) PROMPT exits run_loop via break (full STOP/resume deferred to Phase 22)
- **Followups for Phases 25 / 26:** the 6 new Op variants are awaiting `key_to_op` (Phase 25) and `key_map::resolve` + KEY_DEFS un-stubbing (Phase 26) wiring. Frontend rendering of display_override (CLI ui.rs and GUI Display14Seg) is Phase 25/26.

Use `/git-workflow:commit --with-skills` to commit (German Emoji Conventional Commits, English-only).
</output>
</content>
</invoke>