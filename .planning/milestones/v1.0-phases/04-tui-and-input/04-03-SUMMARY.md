---
phase: 04-tui-and-input
plan: 03
subsystem: keys-and-prgm-display
tags: [crossterm, ratatui, key-mapping, prgm-mode, tdd, rust]

# Dependency graph
requires:
  - phase: 04-tui-and-input
    plan: 01
    provides: App struct, module stubs for keys.rs and prgm_display.rs
  - phase: 03-programming-engine
    provides: CalcState, Op enum (35 variants), dispatch(), run_program()

provides:
  - key_to_op() mapping function with 25 documented key → Op bindings
  - KEY_REF_TABLE: 33-entry self-documenting key reference for TUI right panel
  - format_step() rendering "{pc:03} {op_name}" for any program step
  - op_display_name() covering all 35 Op variants exhaustively

affects: [04-02-ui-layout, 05-persistence]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN: write failing tests first, commit, then implement to pass"
    - "key_to_op() returns None for keys handled in app.handle_key() (digits, quit, F5/F7/F8)"
    - "op_display_name() returns String (not &'static str) — required for dynamic Op data"
    - "Unicode symbols in KEY_REF_TABLE and op_display_name (√, ×, ÷, R↓, X⟷Y, x²)"

key-files:
  created:
    - hp41-cli/src/tests/mod.rs
    - hp41-cli/src/tests/keys_tests.rs
    - hp41-cli/src/tests/prgm_display_tests.rs
  modified:
    - hp41-cli/src/keys.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-cli/src/main.rs

key-decisions:
  - "KEY_REF_TABLE uses unicode escape sequences for special chars (√ = \\u{221a}, × = \\u{00D7}) to avoid source encoding issues"
  - "op_display_name() returns String not &'static str — Op::PushNum(HpNum), Op::Lbl(String), etc. require dynamic formatting"
  - "a/c/k key bindings for Asin/Acos/Atan added (D-09 extension) — c is safe because app.handle_key checks CONTROL modifier first"
  - "Tests use dispatch() with prgm_mode=true to record ops, then set prgm_mode=false — exercises real program recording code path"
  - "Test module wired via #[cfg(test)] mod tests; in main.rs + tests/mod.rs (hp41-core pattern)"

# Metrics
duration: 4min
completed: 2026-05-07
---

# Phase 4 Plan 03: Key Mapping + PRGM Display Summary

**TDD implementation of key_to_op() with 33-entry KEY_REF_TABLE and exhaustive op_display_name() covering all 35 Op variants**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-07T09:31:03Z
- **Completed:** 2026-05-07T09:34:57Z
- **Tasks:** 2 (TDD: RED commit + GREEN commit)
- **Files modified:** 6 (3 created, 3 modified)

## Accomplishments

- `key_to_op()` maps 25 documented key bindings to Op variants, including a/c/k → Asin/Acos/Atan (D-09)
- `KEY_REF_TABLE` has exactly 33 entries — self-documenting right panel for TUI (INPUT-01)
- `format_step()` renders "{pc:03} END" for empty/overrun program and "{pc:03} {op_name}" for any Op
- `op_display_name()` covers all 35 Op variants exhaustively (no non-exhaustive patterns compiler warning)
- 16 unit tests pass across two test modules (keys_tests: 10, prgm_display_tests: 6)
- TDD RED → GREEN cycle followed: failing tests committed first, implementation committed after

## Task Commits

Each task was committed atomically in TDD RED/GREEN order:

1. **TDD RED — test(04-03):** add failing tests for key_to_op() and format_step() - `27edfff`
2. **TDD GREEN — feat(04-03):** implement key_to_op() + KEY_REF_TABLE + format_step() - `afbe2df`

## TDD Gate Compliance

- RED gate: commit `27edfff` — `test(04-03): add failing tests...` (confirmed 8/10 keys tests failed with stub)
- GREEN gate: commit `afbe2df` — `feat(04-03): implement...` (all 16 tests pass)
- REFACTOR gate: Not needed — implementation was clean on first pass

## Files Created/Modified

- `hp41-cli/src/main.rs` — added `#[cfg(test)] mod tests;` declaration
- `hp41-cli/src/tests/mod.rs` — test module root with `#[cfg(test)] mod keys_tests; mod prgm_display_tests;`
- `hp41-cli/src/tests/keys_tests.rs` — 10 tests: Enter, Backspace, arithmetic, stack ops, inverse trig (a/c/k), trig uppercase, F-keys, unmapped keys, KEY_REF_TABLE length
- `hp41-cli/src/tests/prgm_display_tests.rs` — 6 tests: empty program END, Add/Sin/Lbl step display, zero-padded pc, pc-beyond-program END
- `hp41-cli/src/keys.rs` — replaced stub: full key_to_op() match + 33-entry KEY_REF_TABLE
- `hp41-cli/src/prgm_display.rs` — replaced stub: format_step() + exhaustive op_display_name()

## Decisions Made

- `op_display_name()` returns `String` (not `&'static str`) because `Op::PushNum(HpNum)`, `Op::Lbl(String)`, `Op::FmtFix(u8)` and similar variants require dynamic string construction.
- `a/c/k` key bindings for ASIN/ACOS/ATAN are safe despite `c` overlapping with Ctrl+C because `app.handle_key()` checks the CONTROL modifier before routing to `key_to_op()`.
- Test data for `prgm_display_tests` is built via `dispatch()` with `prgm_mode=true`, which exercises the real program-recording code path (no direct `state.program.push()` bypassing dispatch semantics).
- Unicode symbols are written as `\u{NNNN}` escape sequences throughout to keep source files ASCII-clean.

## Deviations from Plan

None — plan executed exactly as written. The test for inverse trig (`inverse_trig_lowercase`) was added beyond the plan's minimum test list as correctness coverage for the three new D-09 bindings (Rule 2: missing test coverage for critical bindings).

## Known Stubs

None — all 35 Op variants are covered in `op_display_name()`. All key bindings documented in D-08/D-09 are wired in `key_to_op()`.

## Threat Surface Scan

No new network endpoints, auth paths, file access, or schema changes introduced. The threat model is unchanged:
- `key_to_op()` returns `None` for all unmapped keys — no state mutation for unknown input
- `format_step()` is read-only on `CalcState` — no execution side-effects

## Self-Check: PASSED

- `hp41-cli/src/keys.rs` — EXISTS, contains `KEY_REF_TABLE` (33 entries), `fn key_to_op`
- `hp41-cli/src/prgm_display.rs` — EXISTS, contains `fn format_step`, `fn op_display_name`
- `hp41-cli/src/tests/keys_tests.rs` — EXISTS
- `hp41-cli/src/tests/prgm_display_tests.rs` — EXISTS
- Commit `27edfff` — RED phase test commit, EXISTS in git log
- Commit `afbe2df` — GREEN phase implementation commit, EXISTS in git log
- `cargo test -p hp41-cli` — exits 0, 16 passed, 0 failed
