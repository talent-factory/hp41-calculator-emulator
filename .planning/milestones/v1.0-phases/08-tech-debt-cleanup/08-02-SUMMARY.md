---
phase: 08-tech-debt-cleanup
plan: "02"
subsystem: hp41-cli
tags: [key-bindings, entry-validation, tdd, sin, clreg, alpha-clear, eex]
dependency_graph:
  requires: []
  provides: [q->SIN binding, g->CLREG binding, Delete-in-ALPHA, entry_buf guards]
  affects: [hp41-cli/src/keys.rs, hp41-cli/src/app.rs]
tech_stack:
  added: []
  patterns: [TDD RED/GREEN/REFACTOR, entry_buf input validation guards]
key_files:
  created: []
  modified:
    - hp41-cli/src/keys.rs
    - hp41-cli/src/app.rs
    - hp41-cli/src/tests/keys_tests.rs
decisions:
  - "Inline tests in keys.rs only for KEY_REF_TABLE checks (no App dependency); key_to_op tests go to tests/keys_tests.rs (existing pattern)"
  - "Replaced test_q_quits_when_no_overlay_open with test_ctrl_c_still_quits to document the new sole-quit behavior"
  - "Delete arm placed before wildcard in handle_alpha_mode_key() to match existing pattern"
metrics:
  duration_seconds: 631
  completed_date: "2026-05-08"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
---

# Phase 8 Plan 02: Key Bindings and Entry_buf Guards Summary

Wire three keyboard bindings (q->SIN, g->CLREG, Delete-in-ALPHA) and four entry_buf guards blocking malformed EEX/decimal input sequences in hp41-cli.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 RED | Failing tests for q->SIN, g->CLREG, KEY_REF_TABLE | 9d5cd8a | keys.rs, tests/keys_tests.rs |
| 1 GREEN | Add q->SIN and g->CLREG bindings in key_to_op | b43ca07 | keys.rs |
| 2 RED | Failing tests for q-not-quit, Delete-in-ALPHA, entry_buf guards | 13725df | app.rs |
| 2 GREEN | Remove q-quit guard, add Delete-in-ALPHA, add entry_buf guards | ec2a2b7 | app.rs |

## What Was Built

**keys.rs changes:**
- `KeyCode::Char('q') => Some(Op::Sin)` — q reassigned from quit to SIN (D-01)
- `KeyCode::Char('g') => Some(Op::Clreg)` — g free key assigned to CLREG (D-02)
- KEY_REF_TABLE: renamed "q/^C" quit to "^C" quit, added "q"->SIN and "g"->CLREG entries (52 → 54 entries)
- Updated module-level doc comments to remove q from quit key references

**app.rs changes:**
- Removed 9-line 'q' quit guard block from `handle_key()` — Ctrl+C is now the sole quit key
- Added `KeyCode::Delete` arm in `handle_alpha_mode_key()` dispatching `Op::AlphaClear`
- Replaced flat `if c.is_ascii_digit() || c == '.' || c == 'e'` guard with three separate guards:
  - `.` blocked if entry_buf contains `.` or `e` (prevents "1..5" and "1.5e2.3")
  - `e` blocked if entry_buf is empty or already contains `e` (prevents "e3" and "1.5ee3")
- Updated STO arithmetic dead_code comment to say "deferred to v1.1"
- Replaced `test_q_quits_when_no_overlay_open` with `test_ctrl_c_still_quits`

**tests/keys_tests.rs changes:**
- Added `q_maps_to_sin` and `g_maps_to_clreg` tests
- Updated `key_ref_table_has_33_entries` to expect 54 entries (was 52)

## Test Results

- hp41-cli: 61 passed (was 49 before this plan — 12 new tests added)
- All tests in workspace: zero failures
- `just lint` (clippy): zero errors

## Deviations from Plan

**1. [Rule 1 - Bug] Inline test module in keys.rs not compiled for test binary**
- Found during: Task 1 RED phase
- Issue: `mod phase8_key_tests` added inline to `keys.rs` was not registered in the test binary. Root cause: tests using `crate::app::App` in `keys.rs` create a dependency cycle that Rust silently drops rather than failing.
- Fix: Added KEY_REF_TABLE tests to the existing `mod tests` block in `keys.rs` (with `use super::KEY_REF_TABLE;`); added `key_to_op` tests to `tests/keys_tests.rs` following the existing project pattern.
- Files modified: keys.rs, tests/keys_tests.rs
- Commit: 9d5cd8a (RED), b43ca07 (GREEN)

## TDD Gate Compliance

- RED gate commit: `test(08-02): add failing tests for q->SIN, g->CLREG, KEY_REF_TABLE (RED)` [9d5cd8a]
- GREEN gate commit: `feat(08-02): add q->SIN and g->CLREG bindings in key_to_op (GREEN)` [b43ca07]
- RED gate commit: `test(08-02): add failing tests for q-not-quit, Delete-in-ALPHA, entry_buf guards (RED)` [13725df]
- GREEN gate commit: `feat(08-02): remove q-quit guard, add Delete-in-ALPHA, add entry_buf guards (GREEN)` [ec2a2b7]

## Self-Check
