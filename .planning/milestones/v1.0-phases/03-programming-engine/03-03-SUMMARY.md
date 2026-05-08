---
phase: 03-programming-engine
plan: "03"
subsystem: programming-engine
tags: [rust, ops, enum, testkind, programming-ops, hp41-core]

# Dependency graph
requires:
  - phase: 02-core-math
    provides: Op enum, dispatch(), StoArithKind pattern, flush_entry_buf
provides:
  - TestKind enum with 12 HP-41 conditional test variants in ops/mod.rs
  - 8 Phase 3 Op variants (Lbl/Gto/Xeq/Rtn/PrgmMode/Test/Isg/Dse) in ops/mod.rs
  - Placeholder dispatch() arms for Phase 3 variants (returns InvalidOp until plan 03-06)
affects: [03-04, 03-05, 03-06, program_tests.rs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TestKind mirrors StoArithKind pattern — newtypes over enum for Op payload"
    - "Placeholder dispatch arms use Err(HpError::InvalidOp) not todo!() to honour zero-panic invariant"

key-files:
  created: []
  modified:
    - hp41-core/src/ops/mod.rs

key-decisions:
  - "Rust non-exhaustive match is an error not a warning — stub dispatch arms required at definition time (Rule 1 auto-fix)"
  - "Stubs return Err(HpError::InvalidOp) not todo!() to preserve zero-panic guarantee in hp41-core"
  - "TestKind follows StoArithKind pattern: all 12 variants in one enum used as Op::Test(TestKind) payload"

patterns-established:
  - "Phase 3 Op variant stubs: Err(HpError::InvalidOp) placeholder — replaced by real impl in plan 03-06"

requirements-completed: [PROG-01, PROG-02]

# Metrics
duration: 10min
completed: 2026-05-07
---

# Phase 03 Plan 03: Op Enum Phase 3 Variants Summary

**TestKind enum (12 HP-41 conditional variants) and 8 Phase 3 Op variants added to ops/mod.rs, enabling Wave 2 program.rs and program_tests.rs to compile**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-07T07:53:00Z
- **Completed:** 2026-05-07T08:03:59Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- `TestKind` enum with exactly 12 variants (XEqZero/XNeZero/XLtZero/XGtZero/XLeZero/XGeZero + XEqY/XNeY/XLtY/XGtY/XLeY/XGeY) inserted after StoArithKind
- 8 Phase 3 `Op` variants (Lbl/Gto/Xeq/Rtn/PrgmMode/Test/Isg/Dse) added to the Op enum
- Placeholder dispatch() arms added for all 8 new variants to satisfy Rust exhaustive-match requirement without panics
- `cargo check -p hp41-core` exits 0 with zero errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add TestKind enum after StoArithKind** - `04bb10d` (feat)
2. **Task 2: Add Phase 3 Op variants to Op enum** - `ad23036` (feat)

## Files Created/Modified
- `hp41-core/src/ops/mod.rs` - Added TestKind enum (9 lines) + Phase 3 Op variants + stub dispatch arms (40 lines total)

## Decisions Made
- Stubs use `Err(HpError::InvalidOp)` not `todo!()` — `todo!()` panics, violating the hp41-core zero-panic invariant from CLAUDE.md
- Doc-comment on Op enum updated to list Phase 3 variants alongside Phase 1 and Phase 2

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added stub dispatch() arms for Phase 3 Op variants**
- **Found during:** Task 2 (Add Phase 3 Op variants to Op enum)
- **Issue:** Plan stated dispatch() would emit a "non-exhaustive patterns warning" — in fact Rust treats non-exhaustive `match` on an enum as a hard **error** (E0004), not a warning. The crate would not compile without arms for the new variants.
- **Fix:** Added 8 placeholder dispatch arms returning `Err(HpError::InvalidOp)` for all Phase 3 variants. These will be replaced with real implementations in plan 03-06.
- **Files modified:** `hp41-core/src/ops/mod.rs`
- **Verification:** `cargo check -p hp41-core` exits 0, zero errors
- **Committed in:** `ad23036` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in plan's assumption about Rust warning vs error)
**Impact on plan:** Auto-fix necessary for compilation. No scope creep — stubs are minimal and will be replaced in plan 03-06 as planned.

## Issues Encountered
- None beyond the auto-fixed deviation above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Wave 2 plans (03-04, 03-05) can now import `TestKind` and all 8 Phase 3 Op variants from `ops/mod.rs`
- `program.rs` and `program_tests.rs` can reference these types without compile errors
- Plan 03-06 must replace the 8 `Err(HpError::InvalidOp)` stub arms with real implementations

---
*Phase: 03-programming-engine*
*Completed: 2026-05-07*
