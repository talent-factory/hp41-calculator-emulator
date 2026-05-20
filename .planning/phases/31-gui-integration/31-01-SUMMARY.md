---
phase: 31-gui-integration
plan: "01"
subsystem: gui-test
tags: [sc4-invariant, regression-test, prgm_display, math-pac-i, file-text-scan]
dependency_graph:
  requires: [28-10]
  provides: [GUI-01-regression-gate]
  affects: [hp41-gui/src-tauri/tests/]
tech_stack:
  added: []
  patterns:
    - "std::process::Command grep-based SC-4 gate test"
    - "include_str! compile-time file-text scan for exhaustive match verification"
key_files:
  created:
    - hp41-gui/src-tauri/tests/sc4_invariant.rs
    - hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs
  modified: []
decisions:
  - "File-text scan (include_str!) chosen over Rust API calls because prgm_display is a private mod — zero source edits in hp41-gui/src-tauri/src/"
  - "Hard-coded MATH1_VARIANT_IDS slice used instead of MATH_1.ops iteration to avoid coupling test to xrom.rs visibility (MATH_1.ops entries have duplicate ASCII/Unicode alias entries; slice collapses to 44 unique Op variant identifiers)"
  - "SC-4 test uses std::process::Command grep invocation (exact CLAUDE.md pattern); test passes when grep exits 1 (no matches) and fails when grep exits 0 (matches found)"
metrics:
  duration_minutes: 8
  completed_date: "2026-05-17"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 0
---

# Phase 31 Plan 01: SC-4 Gate + prgm_display Regression Tests Summary

Two regression tests that convert Phase 28's compile-time guarantees into runtime-asserted
CI gates: SC-4 grep invariant and Math Pac I op_display_name arm coverage.

## What Was Built

### Task 1: SC-4 Invariant Integration Test
File: `hp41-gui/src-tauri/tests/sc4_invariant.rs` (commit `ccc4894`)

Single test `sc4_grep_returns_no_matches()` that invokes `grep -rn -E` with the stricter
SC-4 alternation pattern against `hp41-gui/src-tauri/src/`. The test asserts grep exits
with code 1 (no matches). If a future commit introduces a forbidden math function name
(`op_add`, `op_sin`, `flush_entry`, `format_hpnum`, etc.) in the GUI Rust source, the
test immediately fails with a clear diagnostic — converting the CLAUDE.md narrative
reminder into an enforced runtime gate.

### Task 2: Math Pac I prgm_display File-Text Regression Test
File: `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` (commit `4ed6614`)

Single test `every_math1_op_appears_in_prgm_display()` that:
1. Loads `prgm_display.rs` via `include_str!("../src/prgm_display.rs")` at compile time
2. Asserts each of the 44 unique Phase 28 Math Pac I Op variant identifiers appears as
   an `Op::<Id>` substring in the loaded source
3. Asserts no `_ =>` or `_=>` wildcard catch-all exists in/after the `op_display_name`
   function declaration (enforces the 4-exhaustive-match invariant at test time)

The test is a pure file-text scan — zero `hp41_gui_lib::prgm_display::*` imports, zero
calls to `op_display_name`. This preserves Plan 31-01's "zero source edits in
hp41-gui/src-tauri/src/" pledge since `prgm_display` stays a private `mod`.

## Verification Results

- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test sc4_invariant`: PASSED
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test prgm_display_math1_arms`: PASSED
- `grep -rn -E "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)\(" hp41-gui/src-tauri/src/`: exits 1 (no matches — SC-4 PRESERVED)

## SC-4 Invariant Status

SC-4 grep returns nothing. Confirmed:
- The only `fn op_*` function in `hp41-gui/src-tauri/src/` is `fn op_display_name(...)` in
  `prgm_display.rs` — the documented exception (display formatter, not calculator math logic)
- The stricter pattern (`add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum`) correctly
  excludes `op_display_name` and returns zero matches

## Math Pac I Op Variant Coverage

44 unique Phase 28 Math Pac I Op variant identifiers verified by file-text scan:

| Plan | Variants |
|------|---------|
| 28-02 Hyperbolics | Sinh, Cosh, Tanh, Asinh, Acosh, Atanh (6) |
| 28-03 Complex Arith | CPlus, CMinus, CTimes, CDiv, Real (5) |
| 28-04 Complex Funcs | Magz, Cinv, ZpowN, Zpow1N, ExpZ, LnZ, SinZ, CosZ, TanZ, ApowZ, LogZ, ZpowW (12) |
| 28-05 Poly/Roots | PolyWorkflow, Roots (2) |
| 28-06 Matrix | MatrixWorkflow, MatSize, MatVmat, MatEdit, MatDet, MatInv, MatSimeq, MatVcol (8) |
| 28-07 INTG | Integ (1) |
| 28-08 Solve | Solve, Sol (2) |
| 28-09 DIFEQ | Difeq (1) |
| 28-10 FOUR/Tri/Trans | Four, TriSss, TriAsa, TriSaa, TriSas, TriSsa, Trans2d, Trans3d (8) |

All 44 variants confirmed present. No `_ =>` wildcard catch-all found.

## Invariants Preserved

- **Zero source edits in `hp41-gui/src-tauri/src/`**: `prgm_display` stays a private `mod`;
  no `pub use` widening; no `pub mod prgm_display` change
- **SC-4 invariant**: stricter grep returns nothing (test now enforces it at runtime)
- **4-exhaustive-match invariant**: `op_display_name` confirmed to have no wildcard arm
- **MSRV 1.88 unchanged**: no new dependencies added; `std::process::Command` + `include_str!`
  are stable features well within MSRV bounds

## Deviations from Plan

None — plan executed exactly as written.

## Threat Mitigation Status

| Threat ID | Status |
|-----------|--------|
| T-31-01-01 (Tampering — `_ =>` catch-all) | Mitigated — Task 2 asserts no wildcard exists |
| T-31-01-02 (Tampering — SC-4 violation) | Mitigated — Task 1 SC-4 grep test runs as CI gate |

## Known Stubs

None — both test files are complete gate implementations with no placeholder behavior.

## Self-Check: PASSED

- `hp41-gui/src-tauri/tests/sc4_invariant.rs` exists: FOUND
- `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` exists: FOUND
- Commit `ccc4894` (Task 1): FOUND
- Commit `4ed6614` (Task 2): FOUND
