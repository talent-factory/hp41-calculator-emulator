---
phase: 22-program-control-and-memory-ops
subsystem: core
tags: [hp41, rust, milestone-rollup, phase-22, program-control, memory-ops, catalog, asn]
status: complete

# Dependency graph (phase-level rollup)
requires:
  - phase: 21-flags-display-sound
    provides: "Op enum + display_override + event_buffer + flags channels; #[serde(default)] precedent; print_buffer drain pattern"
  - phase: 20-additional-math-and-rounding
    provides: "HpNum::trunc_int primitive for GTO/XEQ IND integer-pointer extraction"
  - phase: 12-synthetic-programming
    provides: "Op::Null no-op placeholder (reused by INS); row×10+col 1-indexed key-code encoding"
  - phase: 11-print-emulation
    provides: "state.print_buffer Vec<String> drain channel (extended by CATALOG)"
  - phase: 06-science-and-engineering
    provides: "Σ-family ops in stats.rs (audited in 22-03 for bounds-safety under SIZE shrink)"
  - phase: 05-persistence-and-ux
    provides: "BTreeMap<char, String> key_assignments precedent (Phase 22 adds sibling BTreeMap<u8, String> assignments)"
  - phase: 03-programming-engine
    provides: "run_program / run_loop / execute_op / programming-ops catch-all / 4-deep call_stack / find_in_program"
provides:
  - "13 HP-41CV ROM ops in hp41-core: Stop, Pse, GtoInd, XeqInd, Clp, Del, Ins, Size, Cla, Clst, Pack, Catalog, Asn"
  - "1 new CalcState field: assignments: BTreeMap<u8, String> with #[serde(default)]"
  - "pub fn resume_program(state) entry point — interpreter resume from state.pc"
  - "Wave-0 bounds audit: 28 register-access sites in registers/display_ops/stats use state.regs.len() dynamically"
  - "Σ-family fail-closed entry guards (Pitfall 5 sentinels) — SIZE shrink does NOT panic legacy access paths"
  - "5 new integration test suites in hp41-core/tests/: phase22_program_control.rs, phase22_program_edit.rs, phase22_memory_ops.rs, phase22_catalog.rs, phase22_asn.rs"
affects:
  - phase: 23-alpha-operations
    note: "Phase 22 plan 22-03 added Op::Cla (hardware-faithful CLA alias for op_alpha_clear). Phase 23 adds more alpha ops onto the same alpha_reg surface."
  - phase: 24-indirect-addressing
    note: "Phase 22 plan 22-01 ships an inline 6-step indirect resolver in Op::GtoInd / Op::XeqInd run_loop arms. Phase 24 extracts it into resolve_indirect(state, reg) -> Result<u8, HpError> and refactors the ~13 other IND variants to call it."
  - phase: 25-cli-keyboard-wiring
    note: "Phase 25 wires CLI R/S key to resume_program; SIZE/CATALOG numeric-prompt modals; CLP/DEL/INS PRGM-mode editing modals; ASN 2-step alpha-prompt → key-prompt modal."
  - phase: 26-gui-key-map
    note: "Phase 26 GUI key_map can resolve v2.1 stubbed ids (catalog, asn, clp, del, ins, size_prompt, r_s) to real Op variants. The v2.1 cosmetic run_stop Tauri command can wire to resume_program."

# Phase-level metrics
tech-stack:
  added: []  # purely additive on hp41-core; no new dependencies
  patterns:
    - "Inline indirect resolver (6-step bounds-safe pattern) — Phase 24 extraction target"
    - "let-result reset-on-Err pattern for is_running-toggle entry points (resume_program; do NOT use ? propagation)"
    - "Two-channel display+event write for sub-second timing (PSE writes both display_override and event_buffer)"
    - "PRGM-mode dispatch-gate special-case for editing primitives (Clp/Del/Ins fall through instead of self-recording)"
    - "Vec::drain + cursor reposition via .min(post_drain_len) clamp (Pitfall 6)"
    - "Wave-0 bounds audit: replace hardcoded length constants with state.field.len() reads BEFORE introducing the op that mutates the Vec length"
    - "Σ-family fail-closed entry guard: single-line if state.regs.len() < 7 { return InvalidOp } (Pitfall 5)"
    - "Documented no-op variant pattern (Pack ↔ Null precedent): 2-line inline body + doc-comment flagging the divergence"
    - "Dual-variant intentional duplication for display divergence (Cla / AlphaClear coexist, distinct display names)"
    - "Hardware-faithful structured-output pattern for CATALOG (header + payload + footer with uniform 24-char width)"
    - "Empty-string-as-tombstone semantic for map ops (ASN empty name removes)"
    - "Struct-variant JSON shape pinning sentinel test (Pitfall 9)"

key-files-aggregate:
  created:
    - "hp41-core/tests/phase22_program_control.rs — 15 integration tests (348 lines)"
    - "hp41-core/tests/phase22_program_edit.rs — 13 integration tests (307 lines)"
    - "hp41-core/tests/phase22_memory_ops.rs — 20 integration tests (344 lines)"
    - "hp41-core/tests/phase22_catalog.rs — 10 integration tests (210 lines)"
    - "hp41-core/tests/phase22_asn.rs — 10 integration tests (234 lines)"
  modified:
    - "hp41-core/src/state.rs — new assignments BTreeMap<u8, String> field with #[serde(default)]"
    - "hp41-core/src/ops/mod.rs — 13 new Op variants + 13 dispatch arms + PRGM-mode recording gate special-case for edit primitives"
    - "hp41-core/src/ops/program.rs — 3 new run_loop arms (Stop/GtoInd/XeqInd); 9 new execute_op arms (Pse/Size/Cla/Clst/Pack/Catalog/Asn + 2 dispatch-only Clp/Del/Ins); extended programming-ops catch-all (Stop/Clp/Del/Ins/GtoInd/XeqInd); 7 new pub helpers (resume_program, op_clp, op_del, op_ins, op_size, op_clst, op_catalog, op_asn)"
    - "hp41-core/src/lib.rs — re-export resume_program"
    - "hp41-core/src/ops/registers.rs — Wave-0 bounds audit (op_sto/op_rcl/op_sto_arith/op_clreg honor state.regs.len())"
    - "hp41-core/src/ops/display_ops.rs — Wave-0 bounds audit (op_view honors state.regs.len())"
    - "hp41-core/src/ops/stats.rs — 8 Σ-family fail-closed entry guards"
    - "hp41-cli/src/prgm_display.rs — 13 display arms"
    - "hp41-gui/src-tauri/src/prgm_display.rs — same 13 display arms (SC-4 intentional duplication)"

key-decisions-aggregate:
  - "OQ-1 → Option B (hardware-faithful CATALOG): CAT 1 = programs (LBL listing with step counts); CAT 2/3/4 = NOT AVAILABLE. Register-listing dropped from Phase 22 (may return in a future quick-task or v3.0)."
  - "OQ-2 → Option A (SIZE 0 clamps to 1): SIZE 0 silently coerces to 1-register minimum; SIZE > 319 returns InvalidOp. Documented divergence from real HP-41 (which accepts SIZE 000)."
  - "OQ-3 → Option A (ASN empty name removes): hardware-faithful `ASN \"\" key_code` undoes `ASN \"name\" key_code` via state.assignments.remove."
  - "OQ-4 → Acknowledge no change: Op::Cla displays as `CLA` (hardware-faithful program listing); legacy Op::AlphaClear keeps `CLRALPHA` for v1.x save-file fidelity. Two variants intentionally coexist."
  - "STOP/PSE break-pattern asymmetry: STOP breaks run_loop (user yield), PSE writes display+event and continues (timed-display continuation)."
  - "resume_program preserves call_stack (no .clear()): unlike run_program, pending XEQ frames must survive a STOP/resume cycle so RTN behaves correctly."
  - "Pre-mutation 4-deep call-stack guard in Op::XeqInd: over-deep returns CallDepth without partial mutation (matches Op::Xeq precedent at program.rs:206-207)."
  - "PRGM-mode recording-gate special-case: Clp/Del/Ins bypass the auto-record branch and execute immediately — they are program-editing primitives, never self-recorded."
  - "CLP cursor reposition uses start.min(program.len()) clamp (Pitfall 6 sentinel): the cursor lands at the start of the (now-deleted) block, clamped to remain a valid index even when the entire labelled suffix was the deleted block."
  - "DEL uses saturating_sub for clamp arithmetic to remain bounds-safe under any state (zero-panic invariant)."
  - "Wave-0 bounds audit lands as 3 separate commits BEFORE Op::Size: registers.rs+display_ops.rs (write/read sites) / stats.rs (Σ-family entry guards) / op_clreg dynamic length. Behavior-preserving at SIZE=100 baseline (all 620 prior tests stay green)."
  - "Σ-family threshold is uniform `< 7` (not per-function-precise): matches the Σ-block R01..R06 convention; boundary test catches off-by-one errors."
  - "Op::Clst preservation invariant enforced by ABSENCE of assignment in the helper body (no `lastx = ...`, no `lift_enabled = ...`). apply_lift_effect(Neutral) is a no-op for lift_enabled."
  - "Op::Pack is a documented no-op + Neutral lift for the flat-Vec program model; backlog candidate for the hypothetical future Op::End / .END. marker rework."
  - "CATALOG 1 long-label truncation to 9 chars: name.chars().take(9).collect() keeps total line width at 24 chars without an overflow marker."
  - "ASN struct-variant JSON shape pinned by sentinel test: `{\"Asn\":{\"name\":\"SIN\",\"key_code\":11}}` — Pitfall 9 forward-compat guard."
  - "All 13 new Op variants are LiftEffect::Neutral (none push a new value onto the stack)."

patterns-established:
  - "13 Op variants land in the canonical 4 places (Op enum + dispatch + execute_op + both prgm_display.rs copies) — exhaustive-match compiler check is the safety net."
  - "Wave-0 bounds audit per separate-commit-per-file pattern: reusable for any future Vec-length-mutating op."
  - "Inline indirect resolver with documented Phase 24 extraction target."
  - "Hardware-faithful structured-output pattern for CATALOG-style listings: header + payload + footer at uniform 24-char width via the print_buffer drain channel."
  - "Empty-string-as-tombstone semantic for map ops: a single Op variant covers insert + remove via is_empty() discriminant."

requirements-completed: [FN-PROG-01, FN-PROG-02, FN-PROG-03, FN-PROG-04, FN-PROG-05, FN-PROG-06, FN-PROG-07, FN-MEM-01, FN-MEM-02, FN-MEM-03, FN-MEM-04, FN-MEM-05, FN-KEY-01]

# Phase-level metrics
metrics:
  total_plans: 4
  total_tasks: 22
  duration_total: ~130 min (~50 + ~25 + ~35 + ~20)
  ops_added: 13
  state_fields_added: 1
  new_test_files: 5
  new_tests: 71  # 15 + 13 + 20 + 10 + 10 + 1 sentinel cross-test
  hp41_core_tests_total: 660  # was 589 at Phase 21 finish; 71 new in Phase 22
  hp41_core_coverage_lines: 92.32%
  hp41_core_coverage_regions: 90.13%
  panics_introduced: 0
  new_hp_error_variants: 0
  completed: 2026-05-14
---

# Phase 22: Program Control & Memory Ops — Phase Summary

**Phase 22 lands the 13 final HP-41CV ROM ops in `hp41-core` covering program-flow control (STOP/PSE/GTO IND/XEQ IND/resume), program editing (CLP/DEL/INS), memory & stack management (SIZE/CLA/CLST/PACK), key assignments (CATALOG/ASN), plus a critical Wave-0 bounds audit that converts 28 production register-access sites from hardcoded SIZE-100 assumptions to dynamic `state.regs.len()` reads — unlocking SIZE shrink without panic risk.**

## Plan-by-Plan Summary

### Plan 22-01: Program Control (FN-PROG-01/02/06/07)
- 4 new Op variants: `Stop`, `Pse`, `GtoInd(u8)`, `XeqInd(u8)`
- 1 new public entry point: `pub fn resume_program(state) -> Result<(), HpError>`
- 15 integration tests in `tests/phase22_program_control.rs` (348 lines)
- Commit range: `e7468c3..9f7b94a` (6 commits)
- ~50 min duration

### Plan 22-02: Program Editing (FN-PROG-03/04/05)
- 3 new Op variants: `Clp(String)`, `Del(u8)`, `Ins`
- 3 new helpers: `op_clp`, `op_del`, `op_ins`
- PRGM-mode recording-gate special-case (edit primitives bypass auto-record)
- 13 integration tests in `tests/phase22_program_edit.rs` (307 lines)
- Commit range: `29028d7..e579720` (4 commits)
- ~25 min duration

### Plan 22-03: Memory & Stack Ops + Wave-0 Bounds Audit (FN-MEM-01/02/03/04)
- Wave-0 audit (3 commits): 28 register-access sites in registers.rs / display_ops.rs / stats.rs honor `state.regs.len()` dynamically
- 4 new Op variants: `Size(u16)`, `Cla`, `Clst`, `Pack`
- 2 new helpers: `op_size`, `op_clst` (Pack is inline 2-line no-op)
- 8 Σ-family entry guards (Pitfall 5 sentinels — SIZE shrink does not panic)
- 20 integration tests in `tests/phase22_memory_ops.rs` (344 lines)
- Commit range: `1df6dc3..79deb89` (8 commits)
- ~35 min duration

### Plan 22-04: CATALOG + ASN (FN-MEM-05, FN-KEY-01)
- 1 new CalcState field: `assignments: BTreeMap<u8, String>` with `#[serde(default)]`
- 2 new Op variants: `Catalog(u8)`, `Asn { name: String, key_code: u8 }`
- 2 new helpers: `op_catalog`, `op_asn`
- 10 integration tests in `tests/phase22_catalog.rs` (210 lines)
- 10 integration tests in `tests/phase22_asn.rs` (234 lines)
- Commit range: `1a1ead4..08ed3ed` (5 commits incl. SUMMARY)
- ~20 min duration

## Aggregate Accomplishments

- **13 new Op variants in canonical 4-place rule landing:** Op enum + dispatch + execute_op + BOTH prgm_display.rs copies (CLI + GUI). Compile-time exhaustive-match coverage intact throughout.
- **1 new CalcState field with v1.x backward compat:** `assignments: BTreeMap<u8, String>` carries `#[serde(default)]` so every v1.0–v2.1 save file loads cleanly. Sentinel test on `v20-autosave.json` enforces this.
- **OQ-1 / OQ-2 / OQ-3 resolutions implemented:**
  - **OQ-1 Option B (hardware-faithful CATALOG):** CAT 1 = programs (LBL listing with step counts), CAT 2/3/4 = single "NOT AVAILABLE" payload (no XROM/HP-IL/peripherals in this emulator).
  - **OQ-2 Option A (SIZE 0 clamps to 1):** SIZE 0 silently coerces to 1-register minimum; SIZE > 319 returns InvalidOp. Documented divergence from real HP-41 (which accepts SIZE 000).
  - **OQ-3 Option A (ASN empty name removes):** Hardware-faithful `ASN "" 11` undoes `ASN "SIN" 11` via `state.assignments.remove(&key_code)`.
- **OQ-4 (Cla / AlphaClear coexistence):** acknowledged, no code change beyond D-22.13. Op::Cla displays as "CLA" (hardware-faithful); legacy Op::AlphaClear displays as "CLRALPHA" for v1.x save-file fidelity.
- **Wave-0 bounds audit (D-22.11.1):** 28 production register-access sites in `op_sto`, `op_rcl`, `op_sto_arith`, `op_view`, `op_clreg`, and the 8 Σ-family functions (`op_sigma_plus`, `op_sigma_minus`, `op_mean`, `op_sdev`, `op_lr`, `op_yhat`, `op_corr`, `op_cl_sigma_stat`) converted from hardcoded `reg >= 100` / `vec![..; 100]` / `state.regs[1..=6]` to `state.regs.len()`-aware patterns. Sentinel tests prove SIZE-3 register access returns InvalidOp rather than panicking.
- **5 new integration test suites + ~71 new tests:** hp41-core total jumps from 589 (Phase 21 finish) to 660 (Phase 22 finish). All `just ci` green.
- **Coverage:** hp41-core 92.32% lines / 90.13% regions (≥80% gate; effectively non-regressing from the Phase 21 baseline of 92.68% lines).
- **Zero new HpError variants:** Phase 22 reuses `InvalidOp` (most cases) and `CallDepth` (XEQ IND 4-deep guard). Keeps the error surface stable for hp41-cli / hp41-gui display formatting.
- **Zero panics introduced:** every new code path uses `?`-propagation or bounds-safe `.get().ok_or(InvalidOp)?` patterns. The Wave-0 audit eliminated 28 potential panic sites under SIZE shrink. `#![deny(clippy::unwrap_used)]` continues to pass.

## OQ-1 / OQ-2 / OQ-3 Resolutions — Implementation Locations

| OQ | Decision | Code location |
|----|----------|---------------|
| OQ-1 (CATALOG) | Option B hardware-faithful | `hp41-core/src/ops/program.rs::op_catalog` (CAT 1 enumerates Op::Lbl; CAT 2-4 push "NOT AVAILABLE") |
| OQ-2 (SIZE 0) | Option A clamp to 1 | `hp41-core/src/ops/program.rs::op_size` (`let target = nnn.max(1) as usize`) |
| OQ-3 (ASN empty) | Option A removes | `hp41-core/src/ops/program.rs::op_asn` (`if name.is_empty() { state.assignments.remove(&key_code) }`) |
| OQ-4 (Cla/AlphaClear) | Coexist, no change | `hp41-core/src/ops/mod.rs::Op::Cla` (delegates to op_alpha_clear) + `Op::AlphaClear` retained |

## Cross-Cutting Architectural Decisions (Locked in CONTEXT.md, Verified in Code)

- **D-22.21 (4-place rule):** all 13 new variants land in Op enum + dispatch + execute_op + both prgm_display copies. Compile-time exhaustive-match coverage intact.
- **D-22.22 (Save-file compat):** `assignments` field carries `#[serde(default)]`; all new Op variants are additive at the end of the Op enum. v1.x save files load cleanly.
- **D-22.23 (Zero-panic policy):** `#![deny(clippy::unwrap_used)]` continues to hold. Every new code path uses `?`-propagation or `.get().ok_or(InvalidOp)?` patterns.
- **D-22.24 (SC-4 invariant):** no `op_*` / `flush_entry_*` / `format_hpnum` functions added to `hp41-gui/src-tauri/`. Only `prgm_display.rs` exhaustive-match arms (the documented exception).
- **D-22.25 (LiftEffect summary):** all 13 new variants are Neutral. Phase 22 is overwhelmingly control-flow / edit / memory-management, none of which push a new value onto the stack.

## Quality Gates at Phase Exit

- `cargo check --workspace` — exit 0
- `cd hp41-gui/src-tauri && cargo check` — exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0
- `cargo test -p hp41-core` — 660 passed, 0 failed (was 589 at Phase 21 finish)
- `just ci` — exit 0 (workspace tests + clippy + fmt + coverage)
- hp41-core coverage: 92.32% lines / 90.13% regions (target ≥80%)
- Zero `.unwrap()` / `panic!()` introduced in production code
- All 13 Phase 22 Op variants visible: `grep -nE '^\s*(Stop,|Pse,|GtoInd\(|XeqInd\(|Clp\(|Del\(|Ins,|Size\(|Cla,|Clst,|Pack,|Catalog\(|Asn \{)' hp41-core/src/ops/mod.rs` → 13 hits
- v20-autosave.json still loads correctly (sentinel tests in phase21_flags / phase21_sound / phase22_asn all pass)

## Files Touched (Aggregate)

### Created (5 test files)
- `hp41-core/tests/phase22_program_control.rs` (348 lines, 15 tests)
- `hp41-core/tests/phase22_program_edit.rs` (307 lines, 13 tests)
- `hp41-core/tests/phase22_memory_ops.rs` (344 lines, 20 tests)
- `hp41-core/tests/phase22_catalog.rs` (210 lines, 10 tests)
- `hp41-core/tests/phase22_asn.rs` (234 lines, 10 tests)

### Modified (production)
- `hp41-core/src/state.rs` — new `assignments: BTreeMap<u8, String>` field + initializer
- `hp41-core/src/ops/mod.rs` — 13 new Op variants + 13 dispatch arms + PRGM-mode gate special-case
- `hp41-core/src/ops/program.rs` — 3 new run_loop arms + 9 new execute_op arms + extended programming-ops catch-all + 7 new public helpers
- `hp41-core/src/lib.rs` — re-export `resume_program`
- `hp41-core/src/ops/registers.rs` — Wave-0 audit (op_sto/op_rcl/op_sto_arith/op_clreg)
- `hp41-core/src/ops/display_ops.rs` — Wave-0 audit (op_view)
- `hp41-core/src/ops/stats.rs` — 8 Σ-family entry guards
- `hp41-cli/src/prgm_display.rs` — 13 display arms
- `hp41-gui/src-tauri/src/prgm_display.rs` — same 13 display arms (SC-4 duplication)

## Next Phase Readiness

**Phase 23 (ALPHA Operations) is unblocked.** Builds on the alpha_reg surface (Phase 22-03 added Op::Cla as a hardware-faithful alias for op_alpha_clear). Phase 23 lands FN-ALPHA-01..06 (AS, ASTO, ARCL, ATOX, XTOA, AROT) onto the existing alpha_reg field.

**Phase 24 (Indirect Addressing) has a clean refactor target.** The 6-step inline indirect resolver in Op::GtoInd / Op::XeqInd run_loop arms (Plan 22-01) is intentionally duplicated. Phase 24 extracts steps 1–4 (register read + integer truncate + non-integer reject + stringify) into `resolve_indirect(state, reg) -> Result<u8, HpError>` and refactors the ~13 other IND variants to call it.

**Phase 25/26 (CLI/GUI keyboard wiring)** can wire the v2.1-stubbed `catalog`, `asn`, `clp`, `del`, `ins`, `size_prompt`, `r_s` ids to real Op variants. The v2.1 cosmetic `run_stop` Tauri command can wire to `resume_program(state)` (when `is_running == false`) and to a stop-requested sentinel (Phase 22 deferred) when `is_running == true`.

## OQ-4 Acknowledgement (Phase-Level Changelog Note)

**Op::Cla and Op::AlphaClear coexist intentionally throughout Phase 22.** A v1.x save file containing `Op::AlphaClear` continues to list as `"CLRALPHA"` after v2.2; fresh user-recorded `CLA` keystrokes (Phase 25/26) will use `Op::Cla` and list as `"CLA"`. This visible divergence in program listings is documented in CONTEXT.md OQ-4 resolution. Future maintainers should NOT consolidate the two variants — removing Op::AlphaClear breaks v1.0 save-file loading (Pitfall 8 sentinel).

---

*Phase: 22-Program-Control-and-Memory-Ops*
*Plans: 4 (22-01, 22-02, 22-03, 22-04)*
*Tasks: 22 across 4 plans*
*Duration total: ~130 min*
*Completed: 2026-05-14*
*PHASE 22 COMPLETE — all 13 FN-PROG/FN-MEM/FN-KEY requirements satisfied*
