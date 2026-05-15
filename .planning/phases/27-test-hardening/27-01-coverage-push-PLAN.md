---
phase: 27-test-hardening
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-core/tests/program_execution_coverage.rs
  - hp41-core/tests/phase22_stats_size_shrink.rs
  - hp41-core/tests/phase21_phase22_interactive_no_ops.rs
  - hp41-core/tests/format_eng_edges.rs
  - hp41-core/tests/numerical_accuracy.rs
  - justfile
  - CLAUDE.md
autonomous: true
requirements:
  - FN-QUAL-01
  - FN-QUAL-02
tags:
  - coverage
  - tests
  - risk-weighted
  - atomic-gate-raise
  - numerical-accuracy

must_haves:
  truths:
    - "`just coverage` reports `hp41-core` line coverage ≥ 95.0 % after the final commit of this plan (D-27.2 atomic raise). If the realistic ceiling lands at 94.x %, the plan ships with the achieved-and-defensible threshold (D-27.3 fallback) AND CLAUDE.md records the deviation."
    - "The 80 → 95 raise in `justfile coverage:` lands in the SAME commit as the final batch of test additions — never as a standalone gate-only commit (D-27.2, RESEARCH Pitfall 7)."
    - "Every new test in this plan carries a `// Catches: <bug class>` rationale doc comment per D-27.1. No coverage-padding tests (D-27.3: tests that only exercise `Debug`, `Clone`, or unreachable defensive `_ => InvalidOp` arms are NOT acceptable)."
    - "`hp41-core/tests/program_execution_coverage.rs` runs the v2.2 (Phase 20–24) ops through `run_program` and asserts post-state matches the interactive-`dispatch` baseline — catches the Phase 22 Pitfall-3 bug class (PSE display_override survival, lift-effect divergence between interactive and program context)."
    - "`hp41-core/tests/phase22_stats_size_shrink.rs` exercises every SIZE-shrink guard in `ops/stats.rs` (`op_sigma_minus`, `op_mean`, `op_sdev`, `op_lr`, `op_yhat`, `op_corr`, `op_cl_sigma_stat`) plus the `op_lr` / `op_yhat` denom-zero and n=0 guards — Pitfall 5 regression sentinels per RESEARCH §Priority 2."
    - "`hp41-core/tests/phase21_phase22_interactive_no_ops.rs` covers the Phase 21/22 interactive no-op arms in `ops/mod.rs` (`Op::FlagTest`, `Op::Stop`, `Op::Pse`, `Op::Prompt`, `Op::GtoInd`/`Op::XeqInd`, `Op::IsgInd`/`Op::DseInd`) — documents the design invariant that these are Neutral lift, pc unchanged in the interactive path."
    - "`hp41-core/tests/format_eng_edges.rs` covers `format.rs` SCI/ENG zero-mode boundaries and the ENG carry-threshold-crossing case (~15–20 covered lines per RESEARCH §Priority 6)."
    - "`hp41-core/tests/numerical_accuracy.rs` extended with ~70–105 hand-curated v2.2 cases covering PI, P→R, R→P, RND, FRC, MOD, FACT (D-27.5, ~10–15 per op). Existing 500-case baseline still passes at 500/500. Combined ≥ 98 % pass rate maintained per D-27.6 / ROADMAP SC-2."
    - "Each quirky hand-curated case carries a `// Cross-checked against Free42 …` or `// HP-41C Owner's Manual p.XXX` comment per D-27.7. No Free42 binary added to the repo."
    - "FACT(70) returns `HpError::OutOfRange` (HP-41C upper limit per Owner's Manual p.234, cross-checked Free42 `core_math.cc`); FACT(0) returns 1; MOD(7, -3) = 1 (sign follows Y, NOT Rust `%` semantics). These three are the headline HP-41-quirk cases."
    - "No source changes to `hp41-core/src/` (FROZEN per CLAUDE.md v2.2 additions). No source changes to `hp41-gui/src-tauri/` (SC-4 invariant)."
    - "`#![deny(clippy::unwrap_used)]` invariant preserved — every new test file carries `#![allow(clippy::unwrap_used)]` at the file scope."
  artifacts:
    - path: "hp41-core/tests/program_execution_coverage.rs"
      provides: "NEW — exercises Phase 20–24 ops in run_program context to cover the execute_op arms at hp41-core/src/ops/program.rs:647–851 (RESEARCH §Priority 1, ~80–100 covered lines)"
      contains: "run_program"
      contains_2: "// Catches"
    - path: "hp41-core/tests/phase22_stats_size_shrink.rs"
      provides: "NEW — Pitfall-5 regression sentinels for stats SIZE-shrink + denom-zero + n=0 guards (RESEARCH §Priority 2, ~11 covered lines)"
      contains: "SIZE"
    - path: "hp41-core/tests/phase21_phase22_interactive_no_ops.rs"
      provides: "NEW — interactive-dispatch no-op invariants (RESEARCH §Priority 3, ~20 covered lines)"
      contains: "FlagTest"
    - path: "hp41-core/tests/format_eng_edges.rs"
      provides: "NEW — format.rs SCI/ENG zero-mode + ENG carry boundary (RESEARCH §Priority 6, ~15–20 covered lines)"
      contains: "format_eng"
    - path: "hp41-core/tests/numerical_accuracy.rs"
      provides: "EXTENDED — ~70–105 new v2.2 cases for PI, P→R, R→P, RND, FRC, MOD, FACT (D-27.5, FN-QUAL-02). Existing helpers (`new_deg_state`, `push`, `get_x`, `passes_with_tol`) reused verbatim."
      contains: "FACT_0_returns_1"
      contains_2: "Cross-checked against Free42"
    - path: "justfile"
      provides: "EDITED — `coverage:` recipe `--fail-under-lines 80` → `95` (D-27.2 atomic raise — LAST commit of this plan)"
      contains: "--fail-under-lines 95"
    - path: "CLAUDE.md"
      provides: "EDITED — Quality Gates table updated to ≥ 95 % coverage; defensible-skip lines recorded per D-27.3 if applicable"
      contains: "≥ 95%"
  key_links:
    - from: "hp41-core/tests/program_execution_coverage.rs"
      to: "hp41-core/src/ops/program.rs::execute_op (lines 647–851)"
      via: "run_program(&mut state, label) drives the Phase 20–24 op arms"
      pattern: "run_program"
    - from: "hp41-core/tests/numerical_accuracy.rs (v2.2 extension block)"
      to: "hp41-core/src/ops/{program.rs,math.rs} (Op::Pi, Op::Rnd, Op::Frc, Op::Fact, Op::Mod, Op::PolarToRect, Op::RectToPolar)"
      via: "case! macro + dispatch + passes_with_tol"
      pattern: "case!\\("
    - from: "justfile coverage:"
      to: "FN-QUAL-01 gate"
      via: "--fail-under-lines 95 — atomic with final test additions per D-27.2"
      pattern: "--fail-under-lines 95"
---

# Plan 27-01: Coverage push + atomic 80 → 95 gate raise + accuracy suite extension

**Goal:** Close FN-QUAL-01 (line coverage ≥ 95 % on `hp41-core`) and FN-QUAL-02 (numerical-accuracy extension for the 7 v2.2 math/conversion ops) in a single plan, with the gate raise atomic to the final test-addition commit per D-27.2.

**Requirement IDs:** FN-QUAL-01, FN-QUAL-02
**Touches:** `hp41-core/tests/` (4 new files + 1 extended), `justfile`, `CLAUDE.md`
**Plan depends on:** none — independent of 27-02/27-03/27-04 (different test surfaces)

<objective>
Close the 1.41-percentage-point gap from the 93.59 % baseline (measured 2026-05-15 by RESEARCH) to the 95.0 % FN-QUAL-01 gate by adding **risk-weighted tests that catch real bug classes**, NOT padding. Land the v2.2 numerical-accuracy extension for PI / P→R / R→P / RND / FRC / MOD / FACT in the same plan to keep the math test work atomic. The final commit raises the gate from 80 to 95 per D-27.2 — gate-and-test atomicity is the entire point of D-27.2 (RESEARCH Pitfall 7).

Purpose: FN-QUAL-01 has been a deferred liability since v1.1 (coverage slipped from the v1.0 high-water mark of 94.87 % to 92.5–93.6 % across the Phase 11/12 synthetic-dispatch additions). Phase 27 is the last opportunity to close it before v2.2 ships. The risk-weighted approach (Priority 1–6 per RESEARCH §Risk-Weighted Uncovered-Line Inventory) is the inverse of the failure mode D-27.3 warns about: each new test is justified by the bug class it catches, not by the lines it lifts.

Output: 4 new test files in `hp41-core/tests/`, an extension block in `numerical_accuracy.rs`, the atomic `justfile` 80→95 raise in the final commit, and a CLAUDE.md update reflecting the new gate and any documented defensible-skip lines.

Out of scope (explicit):
- Any `hp41-core/src/` changes (FROZEN since Plan 25-01 per CLAUDE.md v2.2 additions). If a coverage hole genuinely requires a `#[allow(dead_code)]` annotation on `ops/math.rs::to_radians_hpnum` or `ops/program.rs::find_label_in_state`, that is a code-side fix and OUT of Phase 27. Document the achievable ceiling in CLAUDE.md instead.
- Proptest additions (those live in Plan 27-02 — keeps paradigms separate).
- IND happy/sad-path integration tests (those live in Plan 27-03).
- Playwright / Vitest / `gui-ci` changes (those live in Plan 27-04).
- Free42 fixture-generation harness (D-27.7 — citation-only).
- GUI coverage gating (D-27.4 — measure-only, recorded in 27-04 SUMMARY).
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

# Phase 27 inputs (locked decisions + research — required reading)
@.planning/phases/27-test-hardening/27-CONTEXT.md
@.planning/phases/27-test-hardening/27-RESEARCH.md

# Risk-weighted uncovered-line inventory targets — read RESEARCH §Risk-Weighted Uncovered-Line Inventory FIRST
@hp41-core/src/ops/program.rs
@hp41-core/src/ops/stats.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/format.rs
@hp41-core/src/state.rs

# Style precedents — mirror, do not re-invent
@hp41-core/tests/numerical_accuracy.rs
@hp41-core/tests/phase21_flags.rs
@hp41-core/tests/phase22_program_control.rs
@hp41-core/tests/phase22_program_edit.rs

# Justfile (target of the atomic gate raise)
@justfile

<interfaces>
<!-- Key contracts the executor needs. Extracted from the codebase so no
     scavenger hunt is required. -->

# numerical_accuracy.rs helpers (existing, reuse verbatim — lines 27–80):
#   const TOLERANCE: f64 = 1e-9;
#   const WIDE_TOL: f64 = 1e-6;
#   fn dec(s: &str) -> Decimal;
#   fn push(state: &mut CalcState, s: &str);
#   fn get_x(state: &CalcState) -> f64;
#   fn passes_with_tol(actual: f64, expected: f64, tol: f64) -> bool;
#   fn new_deg_state() -> CalcState;
#   fn new_rad_state() -> CalcState;
#
# The existing 500-case suite uses a per-domain harness pattern (e.g. Phase 2
# trig cases collect into a Vec<AccuracyCase> and assert ≥98% pass at the end).
# v2.2 extension MUST integrate into the same harness — append cases to the
# same collector or add a parallel collector with the same pass-rate gate.
# Read lines 80–200 of numerical_accuracy.rs to see the harness shape.

# Op variants for the v2.2 extension (all already in hp41-core/src/ops/mod.rs):
#   Op::Pi                                — math.rs::op_pi
#   Op::Rnd                               — math.rs::op_rnd  (idempotent per display mode)
#   Op::Frc                               — math.rs::op_frc  (FRC(x) + INT(x) ≈ x)
#   Op::Fact                              — math.rs::op_fact (FACT(0)=1, FACT(69) max valid, FACT(70) → OutOfRange)
#   Op::Mod                               — math.rs::op_mod  (sign-follows-Y per HP-41 hardware)
#   Op::PolarToRect, Op::RectToPolar      — math.rs (R/θ ↔ X/Y conversions; mode-aware DEG/RAD/GRAD)

# Op variants for program_execution_coverage.rs (Phase 20–24 surface):
#   Phase 20: Pi, Rnd, Frc, Abs, Sign, Fact, Mod, PolarToRect, RectToPolar
#   Phase 21: SfFlag(n), CfFlag(n), FlagTest { kind, flag }, Aon, Aoff, Cld, Tone(n), Pse, Beep
#   Phase 22: Cla, Clst, Pack, Size, Catalog(n), Asn(_), View(n), AView, Prompt, Stop, GtoInd, XeqInd
#   Phase 23: Arcl(_), Asto(_), Atox, Xtoa, Arot, Posa
#   Phase 24: StoInd, RclInd, StoArithInd, SfFlagInd, CfFlagInd, ArclInd, AstoInd, ViewInd, IsgInd, DseInd, FlagTestInd
#
# Pattern (per RESEARCH §Priority 1 recommendation):
#   1. Build a 1-op program: [Op::Lbl("T"), <target Op>, Op::Rtn].
#   2. Set up any required preconditions (X register, regs, flags).
#   3. Call run_program(&mut state, "T").
#   4. Assert post-state matches the same op driven by interactive dispatch().
# This catches divergence between the two execution contexts (Phase 22
# Pitfall 3 class: PSE display_override leaks across run_loop iterations).

# Stats SIZE-shrink guard pattern (existing precedent in
# hp41-core/tests/stats_tests.rs — read for style):
#   let mut state = CalcState::new();
#   state.regs.truncate(3);  // shrink to SIZE 003 (less than the Σ-register
#                              // block start, currently 7)
#   assert!(matches!(dispatch(&mut state, Op::SigmaMinus), Err(HpError::InvalidOp)));
#
# Op::Size is in ops/mod.rs; for SIZE manipulation in tests, prefer direct
# regs.truncate / regs.resize for determinism (rather than dispatching Op::Size).
# Read hp41-core/src/ops/stats.rs lines 50–280 to confirm guard shapes.

# Free42 cite format for D-27.7 — verbatim style precedent for the comments:
#   // Cross-checked against Free42 source ops_math.cc::do_mod —
#   // Free42 returns 1 for MOD(7, -3), matching HP-41C Owner's Manual p.234.
# Place the comment IMMEDIATELY above the relevant case! invocation OR
# inside the #[test] fn body, on the line above the assert.

# Defensible-skip lines per D-27.3 (DO NOT write tests for these — document
# in CLAUDE.md as the achievable-ceiling explanation if 95% is missed):
#   - hp41-core/src/ops/program.rs:49–53, 77, 80, 88, 869, 950–954
#     (op_gto / op_xeq / op_rtn interactive-running guards + find_label_in_state)
#   - hp41-core/src/ops/math.rs:25–34, 41–44
#     (pi_over_180 / pi_over_200 / to_radians_hpnum — gated by #[allow(dead_code)])
#   - hp41-core/src/ops/registers.rs uncovered regions (1.66% — already at 98.34%)
# Total defensible-skip surface ≈ 25 lines.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Pre-implementation baseline + Priority-1 program-execution coverage</name>

  <files>hp41-core/tests/program_execution_coverage.rs</files>

  <read_first>
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Risk-Weighted Uncovered-Line Inventory (lines 345–434)
    - hp41-core/src/ops/program.rs lines 600–860 (the execute_op arms — see specifically the Phase 20–24 arms enumerated in RESEARCH §Priority 1 lines 647–851)
    - hp41-core/tests/phase22_program_control.rs (style precedent for run_program-driven tests)
    - hp41-core/tests/phase21_flags.rs lines 100–300 (per-flag-op run_program tests — closest analog)
    - hp41-core/tests/phase24_ind_variants.rs::isg_ind_inside_run_loop (lines 178–204 — run_program pattern with explicit pc assertion)
  </read_first>

  <action>
    Pre-implementation baseline measurement:

    1. Run `just coverage` once and record the current line coverage % in your scratch buffer. This is the baseline this plan must move; record it for the final CLAUDE.md update. Expected ~93.59 % per RESEARCH 2026-05-15 measurement.

    Create `hp41-core/tests/program_execution_coverage.rs`:

    2. File header (verbatim shape):
       - `#![allow(clippy::unwrap_used)]` at the top (per CLAUDE.md test-mod pattern).
       - Module doc comment explaining the bug class caught: "Phase 20–24 ops are exercised in `hp41-core/src/ops/mod.rs::dispatch` via interactive tests (`phase20_math.rs`, `phase21_*.rs`, `phase22_*.rs`, `phase23_*.rs`, `phase24_*.rs`). They are NOT exercised through `hp41-core/src/ops/program.rs::execute_op` — the `run_program` execution context. This file closes that gap. **Bug class caught:** divergence between interactive and program-context execution (lift effects, side-channel writes to `print_buffer`/`event_buffer`/`display_override`, `pc` advancement). Phase 22 Pitfall 3 (PSE display_override survival across run_loop iterations) is exactly this class — without these tests, a future regression on a single execute_op arm would only surface in a real running program, not in any existing test."
       - Standard imports: `use hp41_core::ops::{dispatch, Op, FlagTestKind, StoArithKind}; use hp41_core::ops::program::run_program; use hp41_core::{CalcState, HpError, HpNum};` plus `rust_decimal::Decimal` and `std::str::FromStr` as needed.

    3. Helper function `fn build_single_op_program(op: Op) -> Vec<Op>`: returns `vec![Op::Lbl("T".into()), op, Op::Rtn]`. Reused by every test in this file. Add a 2-line doc comment.

    4. Helper function `fn run_op_in_program(state: &mut CalcState, op: Op) -> Result<(), HpError>`: sets `state.program = build_single_op_program(op);` then calls `run_program(state, "T")`. Reused by every test. Add a 2-line doc comment.

    5. **Phase 20 math/conversion ops** (one #[test] per op; ~9 tests). For each: set up the required X/Y register state, call run_op_in_program with the op, assert the post-state matches what `dispatch(&mut <fresh state>, op)` produces on the same initial state. Cover: `Op::Pi`, `Op::Rnd`, `Op::Frc`, `Op::Abs` (with negative input — closes RESEARCH §Priority 5 `op_abs` positive-branch via the symmetric pair), `Op::Sign`, `Op::Fact`, `Op::Mod`, `Op::PolarToRect`, `Op::RectToPolar`. Each test gets a `// Catches: program-context divergence on Op::<name> — execute_op arm at program.rs:<approx line>` comment.

    6. **Phase 21 flag / display / sound ops** (~10 tests). Cover: `Op::SfFlag(5)`, `Op::CfFlag(5)`, `Op::FlagTest { kind: FlagTestKind::IsSet, flag: 5 }` (drive run_program with a 3-step body to exercise the run_loop conditional-skip arm), `Op::Aon`, `Op::Aoff`, `Op::Cld`, `Op::Tone(3)`, `Op::Pse`, `Op::Beep`. For Pse, additionally assert `state.print_buffer` and `state.display_override` per the Pitfall-3 invariant. Each test gets the `// Catches:` doc comment.

    7. **Phase 22 program-control / memory / catalog / ASN ops** (~8 tests). Cover: `Op::Cla`, `Op::Clst`, `Op::Pack`, `Op::Catalog(2)`, `Op::View(5)`, `Op::AView`, `Op::Stop`. Skip `Op::Prompt` (interactive-only, no run_loop arm). Skip `Op::GtoInd`/`Op::XeqInd` in this file — they have run_loop arms but the test surface is better suited to Plan 27-03 (IND integration suite).

    8. **Phase 23 ALPHA ops** (~5 tests). Cover: `Op::Arcl(_)`, `Op::Asto(_)`, `Op::Atox`, `Op::Xtoa`, `Op::Arot`, `Op::Posa`. Set up `state.alpha` as needed.

    9. **Phase 24 indirect-addressing ops** (~5 representative tests; full coverage of the 17-op IND surface lives in Plan 27-03, but program-context arms must be exercised here). Cover: `Op::StoInd(5)`, `Op::RclInd(5)`, `Op::SfFlagInd(5)`, `Op::IsgInd(5)` (run_program 3-step body to hit the run_loop skip arm), `Op::DseInd(5)`. For each, set `regs[5] = HpNum::from(<target>)` and dispatch through run_program. Cross-reference Plan 27-03 in a doc comment ("FN-QUAL-04 full IND surface covered by `tests/indirect_addressing.rs`; this file probes the run_loop arms only").

    10. **Coverage measurement after Task 1:** run `cargo llvm-cov clean --workspace && cargo llvm-cov -p hp41-core --text 2>&1 | tail -10` and record the new line coverage in your scratch buffer. Expected ≥ 95.3 % after this task (per RESEARCH §Priority 1 estimate ~+80–100 lines from a 247-uncovered-line baseline). If the number lands ≥ 95.0 %, the rest of the priorities are headroom — proceed anyway to land Priority 2/3/5/6 in Tasks 2 and 3 to give comfortable margin against future drift.

    Self-check after Task 1:
    - `grep -c "// Catches:" hp41-core/tests/program_execution_coverage.rs` returns ≥ 25 (one per #[test] + the per-op rationale comments).
    - `grep -nE "^#\[test\]" hp41-core/tests/program_execution_coverage.rs | wc -l` returns ≥ 25.
    - `cargo test -p hp41-core --test program_execution_coverage` passes.
    - `cargo clippy -p hp41-core --tests -- -D warnings` clean.
  </action>

  <verify>
    <automated>cargo test -p hp41-core --test program_execution_coverage 2>&1 | tail -5</automated>
  </verify>

  <done>
    `program_execution_coverage.rs` exists; ≥ 25 tests pass; per-test `// Catches:` doc comments record the bug class per D-27.1; `just coverage` shows measurable line-coverage uplift toward the 95.0 % gate (record the post-Task-1 number for Task 4's CLAUDE.md update).
  </done>
</task>

<task type="auto">
  <name>Task 2: Priority-2 stats SIZE-shrink guards + Priority-3 interactive no-op arms</name>

  <files>hp41-core/tests/phase22_stats_size_shrink.rs, hp41-core/tests/phase21_phase22_interactive_no_ops.rs</files>

  <read_first>
    - hp41-core/src/ops/stats.rs lines 50–280 (every SIZE-shrink guard and denom-zero / n=0 guard — RESEARCH §Priority 2 enumerates the exact uncovered lines)
    - hp41-core/src/ops/mod.rs lines 671–839, 911–912 (Phase 21/22 interactive no-op arms — RESEARCH §Priority 3)
    - hp41-core/tests/stats_tests.rs (style precedent for stats tests; verify the existing SIZE-aware harness shape — does NOT cover the shrink-fail-closed path)
    - hp41-core/tests/phase21_flags.rs::test_op_flag_test_interactive_dispatch_is_no_op (line 295 — the closest analog for the interactive-no-op test pattern)
  </read_first>

  <action>
    Create two new test files in a single task (both are small enough to land together; both target Priority-2/3 hot spots and share no code).

    **File 1: `hp41-core/tests/phase22_stats_size_shrink.rs`**

    1. File header: `#![allow(clippy::unwrap_used)]` + module doc explaining the bug class — "Pitfall 5 regression sentinel. The stats ops (`Σ-`, `MEAN`, `SDEV`, `LR`, `YHAT`, `CORR`, `CLΣ`) each carry a fail-closed SIZE-shrink guard: if `state.regs.len() < 7` (the Σ-register block start), the op returns `HpError::InvalidOp` instead of indexing OOB. This file exercises EVERY guard; without these tests, a future SIZE-shrink-related regression (Phase 22 Pitfall 5 class) would only surface in a real user-driven SIZE down-shrink workflow."

    2. Helper `fn shrunken_state() -> CalcState`: returns a fresh `CalcState::new()` with `state.regs.truncate(3)` (SIZE 003 — below the Σ block start). Used by every test.

    3. One #[test] per guard (target line numbers from RESEARCH §Priority 2):
       - `op_sigma_minus_shrunken_returns_invalid_op` (line 62 in stats.rs)
       - `op_mean_shrunken_returns_invalid_op` (line 93)
       - `op_sdev_shrunken_returns_invalid_op` (line 120)
       - `op_lr_shrunken_returns_invalid_op` (line 158)
       - `op_yhat_shrunken_returns_invalid_op` (line 206)
       - `op_corr_shrunken_returns_invalid_op` (line 245)
       - `op_cl_sigma_stat_shrunken_returns_invalid_op` (line 279)
       - `op_lr_denom_zero_two_points_with_identical_x_returns_invalid_op` (line 173)
       - `op_yhat_n_zero_empty_sigma_returns_invalid_op` (line 210)
       - `op_yhat_denom_zero_returns_invalid_op` (line 221)
       - `op_corr_n_zero_empty_sigma_returns_invalid_op` (line 249)

    4. Each test: assert `matches!(dispatch(&mut state, Op::<name>), Err(HpError::InvalidOp))` AND assert `state` is unchanged from the precondition (regs.len(), stack.x, lift_enabled — sample 2-3 fields, don't write an exhaustive eq check). Each test gets a `// Catches: SIZE-shrink-shrink panic on Op::<name> — guard at stats.rs:<line>` doc comment.

    5. For the denom-zero / n=0 variants, set up the precondition explicitly (e.g. for op_lr_denom_zero: `op_sigma_plus` twice with the same X value → both data points have identical x → linear regression denom = 0). Cite the bug class: "Catches: divide-by-zero in linear-regression slope when all x_i are equal".

    **File 2: `hp41-core/tests/phase21_phase22_interactive_no_ops.rs`**

    6. File header: `#![allow(clippy::unwrap_used)]` + module doc — "Phase 21 and Phase 22 introduced ops that are intentionally NO-OPS in the interactive (non-program-context) dispatch path: `Op::FlagTest`, `Op::Stop`, `Op::Pse`, `Op::Prompt`, `Op::GtoInd`, `Op::XeqInd`, `Op::IsgInd`, `Op::DseInd`. The design invariant is: pc unchanged, flags unchanged (FlagTest is read-only), lift_enabled Neutral. This file documents and locks that invariant. **Bug class caught:** accidental conversion of an interactive no-op into a state-mutating dispatch (e.g. an interactive Pse that pushes 'PAUSE 1000' twice — once in the interactive arm and once in the run_loop arm)."

    7. Helper `fn assert_neutral_interactive_dispatch(op: Op)`: builds a fresh CalcState, captures the pre-state pc/flags/lift_enabled, dispatches the op, and asserts those three fields are unchanged. For ops that DO produce side effects (Pse's display_override, Stop's no-op, AView's display_override), the test additionally asserts the documented side effect — write a separate per-op test (don't shoehorn into one helper).

    8. Per-op tests (~12 tests):
       - `op_flag_test_isset_interactive_neutral` (covers ops/mod.rs:804)
       - `op_flag_test_isclear_interactive_neutral`
       - `op_flag_test_issetclear_interactive_neutral`
       - `op_flag_test_isclearclear_interactive_neutral` (all four FlagTestKind variants)
       - `op_stop_interactive_neutral` (mod.rs:740 / 822)
       - `op_pse_interactive_writes_pause_to_display_override` (mod.rs:743, 831 — covers Pse's interactive side effect)
       - `op_prompt_interactive_arm_exists` (mod.rs:811 — one test that exercises Op::Prompt via dispatch; assert it doesn't panic and produces the expected `state.display_override`)
       - `op_gto_ind_interactive_returns_invalid_op` (mod.rs:746)
       - `op_xeq_ind_interactive_returns_invalid_op` (mod.rs:749)
       - `op_isg_ind_interactive_discards_skip_signal` (mod.rs:911 — assert that dispatching IsgInd interactively does NOT advance pc; the `.map(|_| ())` arm discards the bool)
       - `op_dse_ind_interactive_discards_skip_signal` (mod.rs:912)
       - `op_flag_test_ind_interactive_neutral` (mod.rs:915)

    9. Each test gets the `// Catches: <bug class>` doc comment. Add cross-references to existing phase21/phase22 example tests in a `// See also: ` line so the reader can locate the in-program counterparts.

    Self-check after Task 2:
    - `cargo test -p hp41-core --test phase22_stats_size_shrink` passes (≥ 11 tests).
    - `cargo test -p hp41-core --test phase21_phase22_interactive_no_ops` passes (≥ 12 tests).
    - `grep -c "// Catches:" hp41-core/tests/phase22_stats_size_shrink.rs` returns ≥ 11.
    - `grep -c "// Catches:" hp41-core/tests/phase21_phase22_interactive_no_ops.rs` returns ≥ 12.
    - `cargo clippy -p hp41-core --tests -- -D warnings` clean.
    - Re-run `just coverage` (the gate is still at 80 at this point; just observe the number). Expected ≥ 95.5 % after Tasks 1+2.
  </action>

  <verify>
    <automated>cargo test -p hp41-core --test phase22_stats_size_shrink --test phase21_phase22_interactive_no_ops 2>&1 | tail -5</automated>
  </verify>

  <done>
    Both new test files exist; ≥ 23 total tests pass across the two files; each test has a `// Catches:` rationale per D-27.1; coverage continues climbing (record the cumulative number for Task 4 CLAUDE.md).
  </done>
</task>

<task type="auto">
  <name>Task 3: Priority-6 format SCI/ENG edges + Priority-5 op_abs positive branch + v2.2 numerical-accuracy extension (FN-QUAL-02)</name>

  <files>hp41-core/tests/format_eng_edges.rs, hp41-core/tests/numerical_accuracy.rs</files>

  <read_first>
    - hp41-core/src/format.rs lines 60–250 (RESEARCH §Priority 6 names lines 60, 73–92, 148, 188–192, 216–218, 247)
    - hp41-core/src/ops/math.rs lines 380–420 (the op_abs arms — RESEARCH §Priority 5)
    - hp41-core/tests/format_tests.rs (existing format test style precedent)
    - hp41-core/tests/numerical_accuracy.rs lines 1–200 (harness shape — `AccuracyCase`, `case!` macro if one exists, per-domain Vec<AccuracyCase> collectors, final ≥ 98% gate)
    - hp41-core/tests/phase20_math.rs (existing v2.2 math test surface — verify which cases are already covered vs new in this plan)
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Code Examples (Examples 1, 2, 4 — the FACT / MOD / RND patterns)
  </read_first>

  <action>
    **File 1: `hp41-core/tests/format_eng_edges.rs`**

    1. File header: `#![allow(clippy::unwrap_used)]` + module doc — "Covers `hp41-core/src/format.rs` SCI/ENG zero-mode boundaries and the ENG carry-threshold-crossing case (RESEARCH §Priority 6, RESEARCH Pitfall 8). **Bug class caught:** display-mode rounding boundary regressions — `0.0 → FmtSci(0)`, `999.9995 → FmtEng(3)`, the ENG carry where mantissa rounds up to the next decade. These cases are not exercised by `format_tests.rs` today; without them, a future `format_eng::round_eng` refactor could silently break the ENG carry."

    2. Imports: `use hp41_core::{CalcState, HpNum}; use hp41_core::ops::{dispatch, Op}; use hp41_core::state::DisplayMode; use hp41_core::format::{format_hpnum, round_to_display_precision, decimal_pow10}; use rust_decimal::Decimal; use std::str::FromStr;` — adjust based on actual format.rs exports.

    3. Tests (~10):
       - `fmt_sci_zero_digits_with_zero_value` (line 148: digits == 0 early return on Sci)
       - `fmt_eng_zero_digits_with_zero_value` (lines 188–192: digits == 0 early return on Eng)
       - `fmt_eng_carry_threshold_crossing_999_9995_in_eng_3` (lines 216–218: mantissa rounded ≥ carry_threshold → new eng_exp) — expected output e.g. `1.000E+3` or equivalent per HP-41 display string format
       - `round_to_display_precision_eng_mode` (line 60: Eng arm in the precision dispatcher)
       - `round_eng_body_direct_call` (lines 73–92: at least 3 direct calls to round_eng with various digits/values to cover the body)
       - `decimal_pow10_exp_zero_returns_one` (line 247: exp == 0 → Decimal::ONE early return) — direct call to decimal_pow10(0); assert eq Decimal::ONE
       - `decimal_pow10_positive_exp_then_negative_exp_round_trip` (covers the symmetric paths; smoke)
       - `rnd_op_in_eng_mode` (drives Op::Rnd while DisplayMode::Eng(2) is active — covers the integration between format::round_to_display_precision Eng arm and Op::Rnd)
       - `op_abs_positive_input_returns_clone` (Priority 5: math.rs:414 — input X = positive Decimal; dispatch Op::Abs; assert X unchanged. ALSO add the negative-input symmetric case for completeness even though it's already covered elsewhere.)

    4. Each test gets `// Catches: <bug class>` doc comment.

    **File 2: extend `hp41-core/tests/numerical_accuracy.rs`** (FN-QUAL-02, D-27.5)

    5. Read the existing harness shape carefully BEFORE writing. The file has 80K of cases organized per-domain (Phase 1, 2, 3, ... — exact organization in lines 80–2700). Each domain collects `AccuracyCase`s into a Vec and asserts ≥ 98 % pass rate at the end of the domain section. The v2.2 extension lands as a new `// === v2.2 extension (Phase 20–24 ops) ===` section at the END of the file, with its own collector and gate. DO NOT modify the existing harness's per-domain collectors — the 500-case baseline must stay at 500/500.

    6. Add ~70–105 hand-curated cases distributed across 7 ops (~10–15 cases per op per D-27.5). Per-op breakdown:

       **Op::Pi** (~3 cases, smallest budget — single-value op):
       - PI returns 3.141592654 (HP-41 10-digit rounding of π)
       - PI in DEG mode → unchanged value
       - PI followed by SIN → 0.0 (within tolerance — sin(π) edge case, validates the trig pipeline doesn't choke on the Pi-pushed value)

       **Op::Fact** (~12 cases — HP-41-specific overflow + zero):
       - FACT(0) = 1 (D-27.5 headline case; HP-41C Owner's Manual p.234, cross-check Free42 `core_math.cc::docmd_fact`)
       - FACT(1) = 1
       - FACT(5) = 120
       - FACT(10) = 3628800
       - FACT(20) = 2.432902008e18
       - FACT(50) = 3.041409320e64
       - FACT(69) = 1.711224524e98 (max valid value — RESEARCH Example 4 reference)
       - FACT(70) → HpError::OutOfRange (HP-41 hardware ceiling) — this is a separate `#[test]` (not a case!) because it asserts an error, not a value
       - FACT of non-integer (e.g. 3.5) → HpError::InvalidOp (HP-41 hardware rejects)
       - FACT of negative → HpError::InvalidOp
       - FACT(2), FACT(7), FACT(13) — coverage for the inner loop
       - Each case has a doc comment citing HP-41C Owner's Manual page or Free42 source line

       **Op::Mod** (~12 cases — sign-follows-Y per HP-41 hardware, RESEARCH Example 2):
       - MOD(7, 3) = 1 (positive Y, positive X — control case)
       - MOD(7, -3) = 1 (positive Y, negative X — HP-41-specific: sign follows Y, NOT Rust `%`)
       - MOD(-7, 3) = -1 (negative Y — sign follows Y)
       - MOD(-7, -3) = -1
       - MOD(0, 5) = 0
       - MOD(5, 5) = 0 (exact-divisible edge)
       - MOD(7.5, 2) = 1.5 (non-integer dividend)
       - MOD(7, 0) → HpError::DivideByZero (test, not case!)
       - Each case carries the citation: `// Cross-checked against Free42 source ops_math.cc::do_mod — HP-41 sign follows Y, per Owner's Manual p.234.`

       **Op::Rnd** (~10 cases — display-mode rounding + idempotency):
       - RND(3.14159, FIX(2)) = 3.14
       - RND(3.14159, FIX(4)) = 3.1416
       - RND(0.1 + 0.2, FIX(5)) — verifies BCD doesn't carry f64 imprecision
       - RND(1234.5678, SCI(2)) = 1.235e3 (rounds to 3 sig figs in SCI 2 — see format.rs round_to_display_precision Sci arm)
       - RND(1234.5678, ENG(2)) — ENG 2 means 3 sig figs in engineering notation, exponent multiple of 3
       - RND idempotent: RND(x) twice = RND(x) once for a sample value in each mode (proptest in 27-02 covers this exhaustively; here 3 hand cases pin specific values)

       **Op::Frc** (~8 cases — FRC + INT round-trip):
       - FRC(3.14) = 0.14
       - FRC(-3.14) = -0.14 (sign follows input — HP-41 convention)
       - FRC(0) = 0
       - FRC(integer) = 0 (e.g. FRC(5) = 0)
       - FRC(very small) = the value (e.g. FRC(0.0001) = 0.0001)
       - Each case validates the HP-41 round-trip invariant manually (proptest in 27-02 covers programmatically)

       **Op::PolarToRect** (~10 cases — R/θ → X/Y conversions, mode-aware):
       - PR(R=5, θ=0°) = (5, 0) in DEG mode
       - PR(R=5, θ=90°) = (0, 5) — verify trig 10-digit rounding artifacts (sin(90°) should be exactly 1, but the BCD path may yield 1.0000000000 ± LSB)
       - PR(R=5, θ=180°) = (-5, 0)
       - PR(R=5, θ=270°) = (0, -5)
       - PR(R=10, θ=45°) = (7.071067812, 7.071067812) — irrational sqrt(50)/sqrt(2) value, exercises the 10-digit rounding boundary
       - PR with negative R: PR(R=-5, θ=0°) = (-5, 0) — sign carries through
       - Same set in RAD mode (R=5, θ=π/2 in radians) = (0, 5) — exercises mode switch
       - Each case has a HP-41C Owner's Manual citation (Chapter 3, polar/rectangular conversions)

       **Op::RectToPolar** (~10 cases — X/Y → R/θ conversions, mode-aware):
       - RP(X=5, Y=0) = (R=5, θ=0°)
       - RP(X=0, Y=5) = (R=5, θ=90°)
       - RP(X=3, Y=4) = (R=5, θ=53.13010235°) — the classic 3-4-5 triangle (HP-41C Owner's Manual reference)
       - RP(X=-3, Y=4) = (R=5, θ=126.8698976°) — second quadrant
       - RP(X=0, Y=0) = (R=0, θ=0°) — degenerate case
       - Round-trip: PR(RP(x,y)) ≈ (x,y) for a sample point — pins the inverse-pair invariant by example (proptest covers it generally)
       - Same set in RAD mode for a sample

    7. Integrate the new cases into a v2.2 collector at the end of `numerical_accuracy.rs`. Use the existing `AccuracyCase` struct + the existing per-domain pass-rate harness pattern. Final gate per D-27.6: combined (existing 500 + new ~70–105) pass rate ≥ 98 %. The existing 500 must stay at 500/500 (any regression there is a hard fail — assert this independently).

    8. Each "Quirky" case (FACT(70), MOD(7,-3), PR(R=5,θ=45°)) gets a comment block per D-27.7 citing source:
       ```
       // Cross-checked against Free42 source ops_math.cc::docmd_fact:
       //   Free42 returns ERR_OUT_OF_RANGE for n > 69, matching the HP-41C
       //   ROM behavior documented in the Owner's Manual p.234.
       ```

    Self-check after Task 3:
    - `cargo test -p hp41-core --test format_eng_edges` passes (≥ 10 tests).
    - `cargo test -p hp41-core --test numerical_accuracy` passes; existing 500 cases at 500/500 (no regression); new v2.2 cases ≥ 98 % pass rate per D-27.6.
    - `grep -c "Cross-checked against Free42\|HP-41C Owner's Manual" hp41-core/tests/numerical_accuracy.rs` returns ≥ 15 (one per quirky case per D-27.7).
    - `cargo clippy -p hp41-core --tests -- -D warnings` clean.
    - Re-run `just coverage` (gate still at 80). Expected ≥ 95.7 % after Tasks 1+2+3. Record the number.

    **If the cumulative coverage post-Task-3 is < 95.0 %:** the realistic ceiling per D-27.3 has been hit. Do NOT add padding tests. Instead, identify the residual gap from `cargo llvm-cov --html`, classify it as defensible-skip per D-27.3 (`ops/program.rs::find_label_in_state`, `ops/math.rs::to_radians_hpnum`, `ops/registers.rs` 1.66% region residue, etc.), and prepare Task 4 to ship the achieved threshold (e.g. 94.5 %) with a CLAUDE.md deviation note. The achieved-and-defensible threshold per D-27.3 is the gate this plan ships.
  </action>

  <verify>
    <automated>cargo test -p hp41-core --test format_eng_edges --test numerical_accuracy 2>&1 | tail -10</automated>
  </verify>

  <done>
    `format_eng_edges.rs` ships with ≥ 10 tests; `numerical_accuracy.rs` extended with ~70–105 v2.2 cases; combined ≥ 98 % pass rate per D-27.6; quirky cases carry Free42 / Owner's Manual citations per D-27.7; coverage ≥ 95.0 % OR D-27.3 ceiling-fallback path identified.
  </done>
</task>

<task type="auto">
  <name>Task 4: Atomic 80 → 95 gate raise + CLAUDE.md update (D-27.2, FINAL commit of this plan)</name>

  <files>justfile, CLAUDE.md</files>

  <read_first>
    - justfile lines 34–37 (the existing coverage: recipe)
    - CLAUDE.md "## Quality Gates" table section (locate via grep — the table has "hp41-core coverage" row at "≥ 80%" with the v1.1/v2.0 column at "92.5% lines / 89.9% regions")
    - CLAUDE.md "## Settled Architecture Decisions" v2.2 section (the last block) — the Phase 27 note goes after the v2.2 additions section
    - .planning/phases/27-test-hardening/27-RESEARCH.md §Pitfalls #7 (coverage gate raise atomicity — D-27.2)
    - Your scratch buffer with the post-Task-3 cumulative coverage number
  </read_first>

  <action>
    **This task ships the atomic gate raise per D-27.2. EVERYTHING in this task lands in a SINGLE commit. If the gate raise and CLAUDE.md update are committed separately and any prior test additions are reverted, the gate diverges from reality — that is the failure mode D-27.2 / Pitfall 7 prevents.**

    Decision branch (use the post-Task-3 coverage number from your scratch buffer):

    **Branch A — coverage ≥ 95.0 %:**

    1. Edit `justfile` line 34 (the doc comment) AND line 37 (the recipe):
       - Line 34 comment: replace `≥80%` with `≥95%`.
       - Line 37 command: replace `--fail-under-lines 80` with `--fail-under-lines 95`.
       - Keep `cargo llvm-cov clean --workspace` (line 36) unchanged — the ff39017 worktree fix stays.

    2. Edit CLAUDE.md "## Quality Gates" table:
       - Update the "hp41-core coverage" row's "Target" column from `≥ 80%` to `≥ 95%`.
       - Update the v1.1/v2.0 column from `92.5% lines / 89.9% regions (slipped slightly...)` to `95.X% lines / 9X.X% regions (Phase 27 FN-QUAL-01 closure, atomic 80→95 ratchet per D-27.2)` — using the exact post-Task-3 numbers.
       - Verify the v1.0 column ("94.87%") stays unchanged.

    3. Add a Phase 27 settled-architecture note to CLAUDE.md, placed AFTER the existing "### v2.2 additions (HP-41CV Feature Completeness, Phases 20–25)" block. The exact heading is `### v2.2 additions (Test Hardening, Phase 27)`. Content (2–3 bullets):
       - "**Coverage gate raise (FN-QUAL-01):** `just coverage` enforces ≥ 95 % line coverage on `hp41-core` (raised atomically from 80 % per D-27.2). The 5 new test files (`program_execution_coverage.rs`, `phase22_stats_size_shrink.rs`, `phase21_phase22_interactive_no_ops.rs`, `format_eng_edges.rs`, `numerical_accuracy.rs` v2.2 extension) close the gap with risk-weighted tests catching real bug classes per D-27.1, not coverage padding per D-27.3."
       - "**Numerical accuracy ≥ 98 % gate extended (FN-QUAL-02):** the 500-case baseline grew to ~570–605 cases covering PI / P→R / R→P / RND / FRC / MOD / FACT per D-27.5. Quirky cases (FACT(70) → OutOfRange, MOD(7,-3) = 1 sign-follows-Y, FACT(0) = 1) carry Free42 / Owner's Manual citations per D-27.7. The 500-case baseline stays at 500/500 — no regression."
       - "**Frozen invariants preserved:** no `hp41-core/src/` source changes (frozen since Plan 25-01); no `hp41-gui/src-tauri/` source changes (SC-4 invariant); MSRV 1.88 unchanged; `#![deny(clippy::unwrap_used)]` continues to apply (new test files carry `#![allow]` at file scope)."

    **Branch B — coverage 94.0 % ≤ post-Task-3 < 95.0 %:**

    Per D-27.3 ceiling-fallback: ship the achieved-and-defensible threshold instead of padding.

    1. Edit `justfile` line 37 to `--fail-under-lines NN.N` where NN.N is the floor of the post-Task-3 number rounded down to one decimal (e.g. 94.7 → `--fail-under-lines 94`).

    2. Edit CLAUDE.md Quality Gates table: update the "hp41-core coverage" Target column to `≥ NN.N% (D-27.3 ceiling)`.

    3. Add the v2.2 / Phase 27 settled-architecture note as in Branch A, BUT add a fourth bullet explaining the D-27.3 deviation:
       - "**D-27.3 ceiling fallback applied:** the practical maximum after the risk-weighted push (Priorities 1–6 per RESEARCH §Risk-Weighted Uncovered-Line Inventory) is NN.N % lines. The residual ~M uncovered lines are defensible-skip per D-27.3 — they live in `ops/program.rs` interactive-running dead branches (lines 49–53, 77, 80, 88, 869, 950–954), `ops/math.rs` `#[allow(dead_code)]`-gated trig constants (lines 25–34, 41–44), and `ops/registers.rs` multi-arm match expression residue (1.66 % region-only). No padding tests added per D-27.3."
       - Document the specific defensible-skip line ranges with file:line citations so a future reader can audit.

    **Branch C — coverage < 94.0 %:**

    Halt and surface as a blocker. The risk-weighted plan was not sufficient. Update CLAUDE.md ONLY with the measured number, leave the gate at 80, and emit `## PLAN BLOCKED — coverage shortfall` in the SUMMARY so the orchestrator can route to gap-closure mode. (This branch should be unreachable per RESEARCH §Gap-closure summary's 95.7–97.6 % estimate, but the safety net is documented.)

    **For all branches, the final commit message MUST mention the atomicity** — e.g. "test(27-01): land risk-weighted coverage push + 80→95 gate raise atomically (D-27.2)". The commit includes ALL files from Tasks 1–3 PLUS the justfile edit PLUS the CLAUDE.md edit.

    Self-check after Task 4:
    - `grep -n "fail-under-lines" justfile` shows the new threshold (95 in branch A; NN in branch B).
    - `grep -n "≥ 95%\|≥ 80%" CLAUDE.md` shows the new threshold (≥ 95% expected in branch A).
    - `just coverage` PASSES (this is the actual gate verification — if it fails, the commit cannot land per D-27.2).
    - `grep -n "Phase 27" CLAUDE.md` shows the new settled-architecture block.
  </action>

  <verify>
    <automated>just coverage 2>&1 | tail -10</automated>
  </verify>

  <done>
    `justfile` and `CLAUDE.md` updated atomically with the achieved threshold (95.0% in branch A; D-27.3 ceiling in branch B); `just coverage` passes; Phase 27 settled-architecture note records the gate change and any defensible-skip deviations; the final commit (this task) is the SAME commit as Task 3's numerical_accuracy.rs extension landing — gate-and-test atomicity per D-27.2 preserved.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| serde JSON parse | Save-load roundtrip (covered by Plan 27-02 proptest_flags.rs item 4); this plan only exercises the deterministic test paths. No new boundary introduced. |
| Decimal::from_str parse | numerical_accuracy.rs case constructors parse string literals. Inputs are test-author-controlled — no untrusted input crosses this boundary. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-27-01-01 | Information Disclosure | numerical_accuracy.rs test outputs | accept | Failing cases print diagnostics with HP-41 reference values only (no secrets in test data; calculator emulation has no sensitive surface). |
| T-27-01-02 | Denial of Service | hp41-core test suite runtime | mitigate | Hand-curated cases only (no random generation in this plan — proptest in 27-02). 70–105 new cases add ~50 ms to `cargo test` runtime per RESEARCH measurement, well under the existing 10s budget. |
| T-27-01-03 | Tampering | justfile coverage gate value | mitigate | D-27.2 atomicity — gate raise and final tests in SAME commit (Task 4 enforces). A revert of either is a single revert. |
</threat_model>

<verification>
## Phase-level checks (run after Task 4 lands)

- `just coverage` exits 0 and reports line coverage ≥ 95.0 % (branch A) or ≥ achieved-and-defensible threshold (branch B).
- `cargo test -p hp41-core` (all 32+ test files) exits 0; no regressions on existing tests.
- `cargo clippy -p hp41-core --tests -- -D warnings` clean.
- `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` — SC-4 invariant preserved (no new matches beyond the existing `op_display_name` formatter).
- `grep -c '// Catches:' hp41-core/tests/program_execution_coverage.rs hp41-core/tests/phase22_stats_size_shrink.rs hp41-core/tests/phase21_phase22_interactive_no_ops.rs hp41-core/tests/format_eng_edges.rs` returns a positive number on each (per D-27.1).
- `grep -c 'Cross-checked against Free42\|HP-41C Owner.s Manual' hp41-core/tests/numerical_accuracy.rs` returns ≥ 15 (per D-27.7).
- `git diff HEAD~1 HEAD -- justfile` shows the `--fail-under-lines` change in the SAME commit as the final test additions (D-27.2 atomicity audit).

## Nyquist verification dimensions (record in plan SUMMARY)

- **Behavioral:** `just coverage` reports ≥ 95.0 % AND `cargo test -p hp41-core` ≥ 98 % pass on numerical_accuracy combined suite. Both are automated commands; no manual verification required.
- **Functional:** Every Phase 20–24 op has at least one run_program-context test in `program_execution_coverage.rs`. Cross-checked by enumerating Op variants vs. `#[test]` fn names.
- **Regression:** Existing 500-case numerical_accuracy baseline stays at 500/500. Pitfall-5 SIZE-shrink regressions caught by new sentinels.
</verification>

<success_criteria>
- [x] FN-QUAL-01 closed: `just coverage` reports ≥ 95.0 % line coverage on hp41-core (or D-27.3 ceiling, documented)
- [x] FN-QUAL-02 closed: numerical_accuracy.rs ≥ 98 % pass rate on the combined ~570–605 cases per D-27.6
- [x] D-27.2 atomicity preserved: gate raise and final test additions in the same commit
- [x] D-27.1 rationale: every new test carries a `// Catches:` doc comment
- [x] D-27.7 cite-trail: every quirky hand-curated case cites Free42 or Owner's Manual
- [x] No `hp41-core/src/` source changes (frozen invariant preserved)
- [x] No `hp41-gui/src-tauri/` source changes (SC-4 invariant preserved)
- [x] MSRV 1.88 unchanged
- [x] CLAUDE.md Quality Gates table reflects the new threshold; v2.2 Phase 27 settled-architecture note added
</success_criteria>

<output>
After completion, create `.planning/phases/27-test-hardening/27-01-SUMMARY.md` recording:
- Pre-Task-1 baseline coverage number (from your scratch buffer)
- Cumulative coverage after each task
- Final gate threshold shipped (95.0 % expected; D-27.3 fallback documented if applicable)
- New test file paths + #[test] counts
- The specific defensible-skip lines documented (if branch B applied)
- Confirmation that the final commit includes both the justfile edit and the last test-addition (D-27.2 atomicity audit)
</output>

<failure_modes>
## Failure Modes & Mitigations

- **Padding-only tests sneak in:** every test MUST have a `// Catches:` rationale. Reviewer / verifier greps for the comment; missing comment = test rejected per D-27.1.
- **Gate raise lands without tests (D-27.2 violation):** Task 4 is the SAME commit as Task 3's final additions. Pre-commit hook OR reviewer checks `git diff HEAD~1 HEAD -- justfile hp41-core/tests/numerical_accuracy.rs` and rejects if either is empty.
- **numerical_accuracy.rs baseline regresses below 500/500:** independently asserted alongside the v2.2 extension's ≥ 98 % gate. A drop in the 500 baseline is a hard fail even if the combined ratio stays ≥ 98 %.
- **Coverage falls short of 95 % AND between 94.0–94.9 %:** D-27.3 ceiling fallback (Branch B in Task 4); ship the achieved threshold with documented defensible-skip lines. NO padding tests.
- **Coverage falls below 94.0 %:** Branch C; halt and surface as `## PLAN BLOCKED`. (Should be unreachable per RESEARCH estimate.)
- **Stale .profraw data from worktree runs:** the existing `cargo llvm-cov clean --workspace` line in `coverage:` (commit ff39017) prevents this. Do NOT delete that line during the edit.
- **A new test happens to also exercise `Debug` / `Clone` paths and lifts coverage cosmetically:** acceptable iff the test was added for bug-catching value per D-27.1; the coverage uplift is a side effect, not the purpose.

## Out of Scope (explicit)
- Proptest additions → Plan 27-02
- IND happy/sad-path integration tests → Plan 27-03
- Playwright / Vitest / `gui-ci` changes → Plan 27-04
- Free42 fixture-generation harness (D-27.7 — citation-only)
- GUI coverage gating (D-27.4 — measure-only)
- `#[cfg(not(coverage))]` or `#[allow(dead_code)]` annotations on `hp41-core/src/` (would require src changes; frozen per CLAUDE.md v2.2 additions)

## References
- 27-CONTEXT.md D-27.1 (risk-weighted tests), D-27.2 (atomic gate raise), D-27.3 (ceiling fallback), D-27.5 (hybrid accuracy extension), D-27.6 (≥ 98 % gate), D-27.7 (Free42 cite-trail)
- 27-RESEARCH.md §Risk-Weighted Uncovered-Line Inventory (Priorities 1–6 enumerated with line numbers)
- 27-RESEARCH.md §Code Examples 1, 2, 4 (RND idempotency, MOD sign, FACT cases)
- 27-RESEARCH.md §Pitfalls 1, 2, 7, 8 (proptest seeds, region-vs-line, atomicity, ENG zero-mode)
- Plan 25-04 precedent (justfile recipe additions + CLAUDE.md settled-architecture block)
- hp41-core/tests/proptest_stack.rs (style precedent for test mod allow attrs)
</failure_modes>
