---
phase: 05-persistence-and-ux
plan: 08
subsystem: quality-gate
tags: [rust, serde, testing, clippy, coverage, ci]

# Dependency graph
requires:
  - phase: 05-persistence-and-ux
    plan: 07
    provides: USER mode runtime dispatch, full Phase 5 feature set

provides:
  - test_calc_state_serde_roundtrip in hp41-core/src/tests.rs — full CalcState JSON round-trip
  - Phase 5 requirement smoke tests in hp41-cli/src/tests/mod.rs
  - Zero clippy warnings in hp41-core and hp41-cli
  - just ci green: 82.51% coverage (gate: ≥80%), 288 + 42 tests all passing

affects:
  - All Phase 5 plans — this is the quality gate confirming all deliverables are correct

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "#[allow(clippy::const_is_empty)] on test assertions for const slice emptiness checks"
    - "Move test mod after all production code to satisfy clippy::items_after_test_module"
    - "Op::StoReg as function item instead of |reg| Op::StoReg(reg) closure (redundant_closure)"
    - "io::Error::other(e) instead of io::Error::new(ErrorKind::Other, e)"

key-files:
  created: []
  modified:
    - hp41-core/src/tests.rs (added serde_tests module with test_calc_state_serde_roundtrip)
    - hp41-cli/src/tests/mod.rs (added 4 Phase 5 smoke tests)
    - hp41-cli/src/app.rs (removed unused scroll fields; #[allow(dead_code)] on StoArith variants; redundant_closure fix)
    - hp41-cli/src/persistence.rs (io::Error::other idiom fix)
    - hp41-cli/src/ui.rs (removed unused TableState import; removed unused super::*; moved test mod to end)

key-decisions:
  - "test_calc_state_serde_roundtrip placed in serde_tests module (not tests:: module) to group serde contract tests separately from general unit tests"
  - "help_scroll and programs_scroll fields removed from App — scroll state is tracked exclusively via RefCell<TableState>; the usize fields were initialized but never read"
  - "#[allow(dead_code)] on StoAdd/StoSub/StoMul/StoDiv PendingInput variants — they are handled in match arms but not yet constructed (key binding wiring deferred to Phase 7)"
  - "#[allow(clippy::const_is_empty)] on HELP_DATA test assertions — the const is compile-time provably non-empty, making is_empty() trivially false; the assert verifies the const was not accidentally emptied during refactoring"

requirements-completed: [PERS-01, PERS-02, UX-01, UX-02, UX-03]

# Metrics
duration: ~25min
completed: 2026-05-07
---

# Phase 5 Plan 08: CI Quality Gate Summary

**Full CI gate passes: zero clippy warnings, 288 hp41-core + 42 hp41-cli tests green, 82.51% coverage; CalcState serde round-trip test and Phase 5 requirement smoke tests added**

## Performance

- **Duration:** ~25 min
- **Completed:** 2026-05-07
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `test_calc_state_serde_roundtrip` to `hp41-core/src/tests.rs` — comprehensive CalcState JSON round-trip verifying: X register, regs[], user_mode, key_assignments (BTreeMap), program (Vec<Op>), and is_running field all survive serde_json round-trip intact
- Added 4 Phase 5 smoke tests to `hp41-cli/src/tests/mod.rs`: PERS-01 canary (default_state_path), UX-01 canary (HELP_DATA non-empty), UX-03 canary (≥10 sample programs), and integration canary (`test_phase5_requirements`)
- Resolved all 10 clippy warnings / errors that blocked `just ci`:
  - `app.rs`: removed vestigial `help_scroll`/`programs_scroll` fields, `#[allow(dead_code)]` on StoArith variants, `redundant_closure` fix for `Op::StoReg`/`Op::RclReg`
  - `persistence.rs`: `io::Error::other(e)` idiom
  - `ui.rs`: removed unused `TableState` import, removed unused `super::*` from test module, moved test module to after all production functions
  - `tests/mod.rs`: `#[allow(clippy::const_is_empty)]` on HELP_DATA assertions
- `just ci` passes: 82.51% line coverage in hp41-core (gate: ≥80%), zero warnings, all tests green

## Task Commits

1. **Task 1: Add CalcState serde round-trip test; verify all VALIDATION.md tests green** - `d661b3c` (test)
2. **Task 2: Add Phase 5 smoke tests; fix all clippy warnings; just ci passes** - `3e2b43c` (feat)

## Files Created/Modified

- `hp41-core/src/tests.rs` — added `mod serde_tests` with `test_calc_state_serde_roundtrip`
- `hp41-cli/src/tests/mod.rs` — added 4 Phase 5 requirement smoke tests
- `hp41-cli/src/app.rs` — removed `help_scroll`/`programs_scroll` fields; `#[allow(dead_code)]` on StoArith variants; `Op::StoReg`/`Op::RclReg` as function items
- `hp41-cli/src/persistence.rs` — `io::Error::other(e)` idiom
- `hp41-cli/src/ui.rs` — removed unused imports; moved test module to end of file

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] 10 clippy warnings blocked just ci before Task 2 could complete**
- **Found during:** Task 2 (running cargo clippy -D warnings)
- **Issue:** 10 clippy warnings in hp41-cli: unused imports (TableState, super::*), unused struct fields (help_scroll, programs_scroll), uncreated enum variants (StoAdd/StoSub/StoMul/StoDiv), redundant closures, io_other_error idiom, items-after-test-module
- **Fix:** Applied all fixes described in Accomplishments section
- **Files modified:** hp41-cli/src/app.rs, hp41-cli/src/persistence.rs, hp41-cli/src/ui.rs
- **Commit:** 3e2b43c (Task 2 commit)

**2. [Rule 1 - Bug] Initial Task 1 commit landed on develop branch instead of worktree branch**
- **Found during:** Task 1 post-commit verification
- **Issue:** git add/commit used `cd /path/to/main-repo && git add` — operated on main repo's develop branch instead of worktree
- **Fix:** Reset develop to d8579f6; re-applied change to worktree's absolute path; recommitted from worktree
- **Files modified:** hp41-core/src/tests.rs (correctly in worktree)
- **Commit:** d661b3c (correct worktree branch commit)

---

**Total deviations:** 2 auto-fixed (1 missing-critical clippy block, 1 worktree path bug)
**Impact on plan:** All plan objectives met. Zero regressions. just ci green.

## VALIDATION.md Test Coverage (all 15 rows green)

| Req ID | Test | Status |
|--------|------|--------|
| PERS-01 | `cargo test -p hp41-cli -- persistence::tests` (6 tests) | Green |
| PERS-01 | `cargo test -p hp41-core -- num::tests::test_hpnum_serde` (2 tests) | Green |
| PERS-01 | `cargo test -p hp41-cli -- persistence::tests::test_user_mode_roundtrip` | Green |
| PERS-01 | `test_calc_state_serde_roundtrip` (added in this plan) | Green |
| UX-01 | `cargo test -p hp41-cli -- help_data::tests` (3 tests) | Green |
| UX-01 | `cargo test -p hp41-cli -- ui::tests::test_help_scroll` | Green |
| UX-02 | `cargo test -p hp41-core -- ops::tests::test_user_mode_toggle` | Green |
| UX-02 | `cargo test -p hp41-cli -- keys::tests::test_user_mode_dispatch` | Green |
| UX-03 | `cargo test -p hp41-cli -- programs::tests` (5 tests) | Green |

## CI Results

```
just ci final run:
  cargo clippy -p hp41-core -p hp41-cli --all-targets -- -D warnings → 0 errors, 0 warnings
  cargo test -p hp41-core → 288 passed (14 suites)
  cargo test -p hp41-cli  → 42 passed (1 suite)
  cargo llvm-cov hp41-core → 82.51% line coverage (gate: ≥80%) PASSED
```

## Known Stubs

None — all Phase 5 deliverables are fully implemented. No placeholder data flows to UI.

## Threat Flags

None — this plan adds tests and fixes lints only; no new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check

### Files exist

- `hp41-core/src/tests.rs` — exists, contains `test_calc_state_serde_roundtrip`
- `hp41-cli/src/tests/mod.rs` — exists, contains `test_phase5_requirements`

### Commits exist

- d661b3c — Task 1 (test: CalcState serde round-trip)
- 3e2b43c — Task 2 (feat: smoke tests + clippy fixes)

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
