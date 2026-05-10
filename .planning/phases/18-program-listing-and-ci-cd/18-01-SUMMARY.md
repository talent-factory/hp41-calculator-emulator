---
phase: 18-program-listing-and-ci-cd
plan: "01"
subsystem: hp41-gui/src-tauri
tags: [tdd, wave-0, red-stubs, tauri, rust]
dependency_graph:
  requires: []
  provides: [red-stubs-handle-sst, red-stubs-handle-bst, red-stub-phase18-fields]
  affects: [hp41-gui/src-tauri/src/commands.rs, hp41-gui/src-tauri/src/types.rs]
tech_stack:
  added: []
  patterns: [wave-0-red-stubs, tdd-inline-mod-tests]
key_files:
  created: []
  modified:
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/types.rs
decisions:
  - "Wave 0 RED stubs reference handle_sst/handle_bst/program_steps/pc which do not exist yet — intentional compile-fail state"
metrics:
  duration: 110s
  completed: "2026-05-10T15:47:59Z"
  tasks_completed: 2
  files_modified: 2
---

# Phase 18 Plan 01: Wave 0 RED Test Stubs Summary

Wave 0 TDD scaffold — five failing test stubs written before any Phase 18 implementation exists. `cargo check --tests` exits 101 confirming RED state for both files.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add RED test stubs to commands.rs for handle_sst and handle_bst | 9e94538 | hp41-gui/src-tauri/src/commands.rs |
| 2 | Add RED test stub to types.rs for Phase 18 CalcStateView fields | 33d7b11 | hp41-gui/src-tauri/src/types.rs |

## What Was Done

**Task 1 — commands.rs:** Added four RED test stubs inside the existing `#[cfg(test)] #[allow(clippy::unwrap_used)] mod tests` block:
- `test_handle_sst_advances_pc` — verifies `handle_sst` increments pc from 0 to 1
- `test_handle_sst_clamps_at_end` — verifies `handle_sst` does not advance past `program.len()`
- `test_handle_bst_decrements_pc` — verifies `handle_bst` decrements pc from 1 to 0
- `test_handle_bst_clamps_at_zero` — verifies `handle_bst` saturates at 0 (no underflow)

**Task 2 — types.rs:** Added one RED test stub inside the existing `#[cfg(test)] mod tests` block:
- `test_phase18_fields_exist` — verifies `CalcStateView` has `program_steps: Vec<String>` and `pc: usize` fields (references `view.program_steps` and `view.pc` which don't exist yet)

## Verification

```
cargo check --tests: EXIT_CODE 101 (NON-ZERO)

Errors confirmed:
- commands.rs: cannot find function `handle_sst` (×2), cannot find function `handle_bst` (×2)
- types.rs: no field `program_steps` on CalcStateView, no field `pc` on CalcStateView

Plan verification grep:
- commands.rs: 2 matches (test_handle_sst_advances_pc, test_handle_bst_clamps_at_zero)
- types.rs: 1 match (test_phase18_fields_exist)
```

Wave 0 RED state confirmed. Plan 02 will implement `handle_sst`, `handle_bst`, and the `CalcStateView` fields to make these tests GREEN.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

By design, this entire plan IS the stub layer. The test functions themselves are intentionally incomplete (referencing undefined symbols) — this is the correct Wave 0 state. Plan 02 resolves them.

## Threat Surface Scan

No new production code introduced. Test-only changes in `#[cfg(test)]` blocks — no threat surface modifications.

## Self-Check: PASSED

- [x] hp41-gui/src-tauri/src/commands.rs modified: 4 test stubs present
- [x] hp41-gui/src-tauri/src/types.rs modified: 1 test stub present
- [x] Commit 9e94538 exists (Task 1)
- [x] Commit 33d7b11 exists (Task 2)
- [x] cargo check --tests exits NON-ZERO (RED state confirmed)
