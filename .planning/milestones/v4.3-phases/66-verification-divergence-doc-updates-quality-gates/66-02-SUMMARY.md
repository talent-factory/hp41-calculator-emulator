---
phase: 66-verification-divergence-doc-updates-quality-gates
plan: 02
subsystem: core-ops-print
tags: [printer-guard, flags, unc-02, unc-01, unc-03, re-entrancy, tdd]

requires:
  - phase: 66-01
    provides: HpError::NonExistent variant in hp41-core/src/error.rs

provides:
  - Printer-presence guard on op_prx/op_pra/op_prstk (flags 21/55, UNC-02)
  - New test test_prx_returns_nonexistent_when_no_printer_flag
  - Reworked 18 print tests (setup_with_printer sets flag 55)
  - UNC-01 regression test pinning back-arrow clears error message (OM p.15)
  - Accurate OM-cited op_size comment replacing misleading "MEM LOST" text (UNC-03)
  - 7-PITFALLS → 12-test coverage mapping header in phase_63_interrupting_alarms.rs (D-08)

affects: [VERIFY-01, divergence-doc-sweep, quality-gates]

tech-stack:
  added: []
  patterns:
    - "Printer-presence guard: require_printer() reads flag 55 || flag 21 before any print op"
    - "TDD RED/GREEN: new failing test committed before implementation"
    - "Blast-radius fix pattern: CLI print-modal tests updated to set flag 55"

key-files:
  created: []
  modified:
    - hp41-core/src/ops/print.rs
    - hp41-core/tests/print_tests.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/phase_63_interrupting_alarms.rs
    - hp41-cli/src/app.rs

key-decisions:
  - "UNC-02 fixed: require_printer() helper checks flag 55 (Printer Existence) || flag 21 (Printer Enable); returns Err(NonExistent) when both clear — flags 21/55 only, NOT flag 25"
  - "UNC-01 already-correct: regression test added to pin behavior (no production code change)"
  - "UNC-03 already-correct: op_size comment replaced with OM-accurate text citing p.19/p.57 — no display_override added"
  - "Re-entrancy: 7-PITFALLS → 12-test mapping documented as header comment; zero gaps confirmed; no new test added (D-08)"
  - "Blast radius: two CLI print-modal tests (test_print_modal_prx_sets_message, test_print_log_file_append) fixed to set flag 55 in setup"

patterns-established:
  - "Print-guard pattern: inline require_printer() helper at top of each print fn — one function, three call sites"
  - "Test blast-radius audit: after adding an error guard to a production fn, grep all test files for callers and add flag setup"

requirements-completed: [VERIFY-01]

duration: 25min
completed: 2026-06-10
---

# Phase 66 Plan 02: UNC-02/01/03 Code Fixes + Re-entrancy Matrix Summary

**PRX/PRA/PRSTK now error with NonExistent when no printer flag is set (flag 55/21); UNC-01 and UNC-03 confirmed already-correct with regression test and accurate comment; 7-PITFALLS re-entrancy coverage documented as zero-gap.**

## Completed Tasks

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| T1 RED | Failing test for no-printer guard | af58cc1 | print_tests.rs |
| T1 GREEN | Printer-presence guard on op_prx/pra/prstk | 2eddf83 | print.rs |
| T2 | UNC-03 comment fix + UNC-01 regression test + blast-radius CLI fix | 9bad095 | program.rs, app.rs |
| T3 | 7-PITFALLS → 12-test mapping header | e7d06b8 | phase_63_interrupting_alarms.rs |

## Implementation Summary

### UNC-02: PRX/PRA/PRSTK Printer-Presence Guard

Added `require_printer(state: &CalcState) -> Result<(), HpError>` to `hp41-core/src/ops/print.rs`. The helper reads `flag_get(state.flags, 55) || flag_get(state.flags, 21)`; if both are clear, returns `Err(HpError::NonExistent)`. All three print ops call this as their first statement.

Flag semantics per OM p.53-54: flag 55 = "Printer Existence" (hardware-detected), flag 21 = "Printer Enable" (user-settable assumption). Flag 25 = "Error Ignore" — explicitly NOT a printer flag.

### UNC-01: Back-Arrow Clears Error Message

Already correct at `app.rs` lines 974-975. A new regression test `test_backspace_clears_error_message` asserts `app.message == None` after back-arrow with an active error message.

### UNC-03: op_size Comment Fix

Replaced "hardware-faithful 'MEM LOST'" with accurate OM-cited text: SIZE reduction is a silent truncation per OM p.19; MEMORY LOST is a power-event / Continuous-Memory-clear display per OM p.57. No logic change, no display_override added.

### Re-entrancy Matrix (D-08)

Added file-level header comment to `phase_63_interrupting_alarms.rs` mapping all 7 PITFALLS scenarios to their 12 covering test functions. Explicitly states zero gaps — no new test required.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] CLI print-modal test blast radius from UNC-02 guard**
- **Found during:** Task 2 (running `cargo test -p hp41-cli`)
- **Issue:** Two CLI tests (`test_print_modal_prx_sets_message`, `test_print_log_file_append`) called PRX on a default-flags state; the new `require_printer()` guard returned NonExistent instead of printing
- **Fix:** Added `app.state.flags = flag_set(app.state.flags, 55)` to both test setups
- **Files modified:** `hp41-cli/src/app.rs`
- **Commit:** 9bad095

## Known Stubs

None — all changes are fully wired. The printer flag guard is live in production code; tests verify both success (flag set) and failure (flags clear) paths.

## Threat Flags

None — offline calculator behavior change, no new network/auth/serialization surface.

## Verification

- `cargo test -p hp41-core --test print_tests`: 19 passed (18 reworked + 1 new)
- `cargo test -p hp41-core --test phase22_catalog`: 9 passed (CATALOG unaffected)
- `cargo test -p hp41-cli`: 523 passed, 4 ignored
- `cargo test -p hp41-core --test phase_63_interrupting_alarms`: 12 passed
- `cargo test -p hp41-core`: 3143 passed, 2 ignored
- `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings`: clean (0 warnings)
- `print.rs`: uses flags 21/55 only; no println!/eprintln!; no display_override

## Self-Check: PASSED

- hp41-core/src/ops/print.rs: FOUND (contains flag_get(state.flags, 55) and Err(HpError::NonExistent))
- hp41-core/tests/print_tests.rs: FOUND (contains test_prx_returns_nonexistent_when_no_printer_flag)
- hp41-core/tests/phase_63_interrupting_alarms.rs: FOUND (contains PITFALLS mapping header)
- hp41-core/src/ops/program.rs: FOUND (OM p.19 citation in op_size comment)
- hp41-cli/src/app.rs: FOUND (test_backspace_clears_error_message)
- Commit af58cc1: FOUND (RED test)
- Commit 2eddf83: FOUND (GREEN guard implementation)
- Commit 9bad095: FOUND (UNC-03 + UNC-01 + blast radius)
- Commit e7d06b8: FOUND (re-entrancy mapping)
