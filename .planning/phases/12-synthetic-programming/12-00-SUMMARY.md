---
phase: 12-synthetic-programming
plan: "00"
subsystem: testing
tags: [rust, hp41-core, synthetic-programming, tdd, red-tests, getkey, null, hidden-registers]

# Dependency graph
requires:
  - phase: 11-print-emulation
    provides: "Established CalcState field patterns, serde(default) usage, and run_program test patterns used as analogs"
provides:
  - "Wave 0 RED test scaffold in hp41-core/tests/synthetic_tests.rs (21 failing tests)"
  - "Executable behavioral contract for Wave 1 (Plan 12-01) to satisfy"
  - "Test coverage definitions for SYNT-01 (GETKEY), SYNT-02 (NULL), SYNT-03 (hidden regs M/N/O), SYNT-04 (SyntheticByte)"
affects:
  - 12-01-synthetic-core
  - 12-02-synthetic-cli

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wave 0 RED scaffold: test file references symbols that do not yet exist, causing compile errors until Wave 1 ships"
    - "TDD RED state: no #[ignore] attributes — all 21 tests are active RED checks"
    - "serde round-trip tests: JSON serialize/deserialize CalcState and Op variants with #[serde(default)] backward compat"

key-files:
  created:
    - hp41-core/tests/synthetic_tests.rs
  modified: []

key-decisions:
  - "Wave 0 RED-only plan: tests reference non-existent symbols (Op::GetKey, Op::Null, StoM/N/O, RclM/N/O, SyntheticByte(u8), last_key_code, reg_m/n/o, synthetic_byte_to_op()) to establish contract before implementation"
  - "21 tests written (exceeds 18-test minimum): 4 SYNT-01 + 3 SYNT-02 + 7 SYNT-03 + 5 SYNT-04 + 2 bonus"
  - "No #[ignore] or todo!() macros — RED state comes from missing symbols, not stub macros"
  - "serde_json already present in hp41-core dev-dependencies — no manifest changes needed"

patterns-established:
  - "Wave 0 TDD RED scaffold: place test file before implementation; compile errors ARE the failing tests"
  - "run_program integration tests: use label/op/Rtn program pattern for in-program coverage"
  - "Backward compat serde test: strip new fields from JSON via serde_json::Value, verify defaults load"

requirements-completed:
  - SYNT-01
  - SYNT-02
  - SYNT-03
  - SYNT-04

# Metrics
duration: 7min
completed: 2026-05-09
---

# Phase 12 Plan 00: Synthetic Programming Wave 0 Test Scaffold Summary

**21-test RED scaffold in hp41-core/tests/synthetic_tests.rs covering GETKEY, NULL, hidden registers M/N/O, and SyntheticByte(u8) — compile errors are the failing tests until Plan 12-01 ships Op variants and CalcState fields**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-05-09T06:50:00Z
- **Completed:** 2026-05-09T06:57:13Z
- **Tasks:** 1 of 1
- **Files modified:** 1 created

## Accomplishments

- Created `hp41-core/tests/synthetic_tests.rs` with 21 failing tests (RED state) covering all four SYNT requirements
- Tests reference 9 new Op variants, 4 new CalcState fields, and `synthetic_byte_to_op()` — none of which exist yet
- Verified `just build` (library build) still passes — the test scaffold does not break the library build
- Confirmed RED state: `cargo test -p hp41-core --test synthetic_tests` produces 43 compile errors referencing missing symbols

## Task Commits

1. **Task 1: Create synthetic_tests.rs scaffold with failing tests** - `dc55132` (test)

**Plan metadata:** committed below as part of docs commit

## Files Created/Modified

- `hp41-core/tests/synthetic_tests.rs` — 21-test RED scaffold for SYNT-01 through SYNT-04

## Decisions Made

- Used exact content structure from PLAN.md task action, which precisely mirrors the register_tests.rs and print_tests.rs analog patterns
- serde_json was already present in hp41-core `[dev-dependencies]` — no Cargo.toml modification needed
- All 21 tests exceed the 18-test minimum: added 2 bonus tests (lift semantics tests and independence test for hidden regs)

## Deviations from Plan

None - plan executed exactly as written. The test file content matched the plan specification exactly.

## Issues Encountered

None. The test file was written from the plan specification directly. Library build passes; test compilation fails as expected (RED state verified via `cargo test -p hp41-core --test synthetic_tests`).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 12-01 (Wave 1 core implementation) can now proceed: it must implement all referenced symbols to turn these 21 RED tests GREEN
- The test file is the executable contract: `Op::GetKey`, `Op::Null`, `Op::StoM/N/O`, `Op::RclM/N/O`, `Op::SyntheticByte(u8)`, `CalcState.last_key_code`, `CalcState.reg_m/n/o`, and `synthetic_byte_to_op()` must all be added by Plan 12-01
- No blockers — library build is clean

## Self-Check

- [x] `hp41-core/tests/synthetic_tests.rs` exists
- [x] `grep -c "^#[test]" ...` returns 21 (>= 18 required)
- [x] All 9 Op variants referenced
- [x] All 4 CalcState fields referenced
- [x] `synthetic_byte_to_op(0xCF)` reference present
- [x] `hp41_core::run_program` referenced (3 program tests)
- [x] No `#[ignore]` attributes
- [x] No `todo!()` macros
- [x] `serde_json` in `[dev-dependencies]` (pre-existing)
- [x] `just build` passes (library build clean)
- [x] RED state confirmed (43 compile errors)
- [x] Commit `dc55132` exists

## Self-Check: PASSED

---
*Phase: 12-synthetic-programming*
*Completed: 2026-05-09*
