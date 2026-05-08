---
phase: 11-print-emulation
plan: "00"
subsystem: hp41-core/tests
tags: [tdd, red-state, print, test-scaffold]
dependency_graph:
  requires: []
  provides: [hp41-core/tests/print_tests.rs]
  affects: []
tech_stack:
  added: []
  patterns: [integration-test, tdd-red-state]
key_files:
  created:
    - hp41-core/tests/print_tests.rs
  modified: []
decisions:
  - "Wave 0 RED state: test file references Op::PRX/PRA/PRSTK and print_buffer which do not exist until Plan 11-01"
  - "18 test functions organized by PRNT-01 (PRX, 5 tests), PRNT-02 (PRA, 5 tests), PRNT-03 (PRSTK, 4 tests), program execution (3 tests, 1 test)"
  - "No #[ignore] attributes — all tests are active RED checks per TDD discipline"
metrics:
  duration: "89 seconds"
  completed_date: "2026-05-08"
  tasks_completed: 1
  tasks_total: 1
  files_created: 1
  files_modified: 0
---

# Phase 11 Plan 00: Print Test Scaffold Summary

Wave 0 TDD test scaffold for PRX/PRA/PRSTK print operations — 18 failing integration tests in RED state covering all PRNT-01/02/03 behavioral contracts before any implementation exists.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create print_tests.rs scaffold with failing tests | 93b6513 | hp41-core/tests/print_tests.rs (created, 270 lines) |

## What Was Built

Created `hp41-core/tests/print_tests.rs` with 18 integration test stubs that define the behavioral contracts for the three print operations:

**PRNT-01: PRX (5 tests)**
- `test_prx_pushes_one_line_to_buffer` — buffer gets exactly 1 line
- `test_prx_output_is_24_chars` — line width exactly 24 chars
- `test_prx_output_is_right_aligned` — leading spaces for short values
- `test_prx_respects_display_mode_sci` — SCI mode produces E-notation
- `test_prx_lift_effect_neutral` — stack X unchanged after PRX

**PRNT-02: PRA (5 tests)**
- `test_pra_pushes_one_line_to_buffer` — buffer gets exactly 1 line
- `test_pra_output_is_24_chars` — line width exactly 24 chars
- `test_pra_output_is_left_aligned` — trailing spaces for short values
- `test_pra_empty_alpha_is_24_spaces` — empty alpha_reg → 24 spaces
- `test_pra_truncates_long_alpha_to_24_chars` — 30-char input → 24-char output

**PRNT-03: PRSTK (4 tests)**
- `test_prstk_produces_six_lines` — exactly 6 lines in buffer
- `test_prstk_all_lines_are_24_chars` — all 6 lines are exactly 24 chars
- `test_prstk_line_order_and_labels` — T:/Z:/Y:/X:/LASTX:/ALPHA: order
- `test_prstk_alpha_empty_line_format` — empty alpha_reg formats correctly
- `test_prstk_alpha_nonempty_line_format` — non-empty content appears in line

**Program execution (3 tests)**
- `test_prx_in_program` — PRX inside running program populates buffer
- `test_pra_in_program` — PRA inside running program populates buffer
- `test_prstk_in_program` — PRSTK inside running program pushes 6 lines

## RED State Verification

```
cargo test -p hp41-core --test print_tests
  → error: could not compile `hp41-core` (test "print_tests") due to 40 previous errors
  → no variant or associated item named `PRX` found for enum `Op`
  → no variant or associated item named `PRA` found for enum `Op`
  → no variant or associated item named `PRSTK` found for enum `Op`
  → no field `print_buffer` on type `CalcState`
```

```
cargo build -p hp41-core
  → Finished `dev` profile (library build unaffected by test file)
```

## Deviations from Plan

None — plan executed exactly as written. The test content was taken verbatim from the plan specification.

## Known Stubs

None — this is a test scaffold plan. The tests themselves are not stubs; they are intentionally failing because the implementation does not exist yet (Wave 0 RED state by design).

## Threat Flags

None — Wave 0 creates only test code with synthetic `CalcState` values. No trust boundaries introduced.

## Self-Check: PASSED

- `hp41-core/tests/print_tests.rs` exists: FOUND
- Commit 93b6513 exists: FOUND
- `grep -c "^#\[test\]" hp41-core/tests/print_tests.rs` = 18: PASSED
- No `#[ignore]` attributes: PASSED
- `cargo build -p hp41-core` succeeds: PASSED
- `cargo test --test print_tests` shows compile error referencing Op::PRX: PASSED (RED state confirmed)
