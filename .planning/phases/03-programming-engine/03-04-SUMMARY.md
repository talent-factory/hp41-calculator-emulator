---
phase: 03-programming-engine
plan: "04"
subsystem: programming-engine
tags: [rust, ops, prgm_mode, dispatch, flush_entry_buf, recording-gate, tdd, hp41-core]

# Dependency graph
requires:
  - phase: 03-programming-engine
    plan: "03"
    provides: Phase 3 Op variants (Lbl/Gto/Xeq/Rtn/PrgmMode/Test/Isg/Dse) + TestKind in ops/mod.rs
  - phase: 03-programming-engine
    plan: "01"
    provides: CalcState.program Vec<Op> + prgm_mode bool field in state.rs
provides:
  - prgm_mode gate in dispatch() — records ops to program when prgm_mode=true
  - prgm_mode routing in flush_entry_buf() — routes PushNum to program when prgm_mode=true
  - Integration test suite prgm_mode_tests.rs (10 tests) for recording gate behavior
affects: [03-05, 03-06, program_tests.rs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "prgm_mode gate pattern: check flag early in dispatch(), before match op, return immediately"
    - "flush_entry_buf prgm_mode branch: PushNum to program Vec, no execution-state change"
    - "PrgmMode toggle: immediate exit from recording without recording the toggle op itself"
    - "catch-all _ arm in dispatch() match: Phase 3 ops handled by gate above, not individual arms"

key-files:
  created:
    - hp41-core/tests/prgm_mode_tests.rs
  modified:
    - hp41-core/src/ops/mod.rs

key-decisions:
  - "prgm_mode gate placed between flush_entry_buf() call and match op {} block — ensures flush routes PushNum before the gate intercepts the op"
  - "PrgmMode toggle op exits recording immediately without self-recording (HP-41 Pitfall 6)"
  - "Phase 3 stub arms replaced by single catch-all _ => Err(HpError::InvalidOp) — gate above makes individual stubs redundant while prgm_mode=false path still needs them as placeholders for 03-06"
  - "lift_enabled unchanged during recording flush — recording does not affect execution state (D-03/D-04)"

# Metrics
duration: ~10min
completed: 2026-05-07
---

# Phase 03 Plan 04: PRGM Mode Recording Gate Summary

**prgm_mode gate added to dispatch() and flush_entry_buf() extended to route PushNum to program Vec when recording — 257 tests pass**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-07T08:05:00Z
- **Completed:** 2026-05-07T08:14:23Z
- **Tasks:** 2 (TDD: RED + GREEN each)
- **Files modified:** 1 (ops/mod.rs)
- **Files created:** 1 (prgm_mode_tests.rs)

## Accomplishments

- `flush_entry_buf()` extended with prgm_mode branch:
  - `prgm_mode=true` → append `Op::PushNum(n)` to `state.program`; `lift_enabled` unchanged
  - `prgm_mode=false` → existing `enter_number` + `apply_lift_effect(Enable)` unchanged
- `dispatch()` gains prgm_mode recording gate between `flush_entry_buf(state)?` and `match op`:
  - `prgm_mode=true` + `op=PrgmMode` → set `prgm_mode=false`, apply Neutral lift, return Ok; op NOT recorded
  - `prgm_mode=true` + any other op → push op to `state.program`, return Ok; op NOT executed
  - `prgm_mode=false` → fall through to existing match arms unchanged
- Individual Phase 3 stub arms replaced by single catch-all `_ => Err(HpError::InvalidOp)`
- 10 integration tests in `prgm_mode_tests.rs` cover all behavioral assertions
- Test count: 247 (baseline) → 257 (10 new tests, all pass)

## Task Commits

Each task committed atomically via TDD flow:

| # | Task | Commit | Type | Files |
|---|------|--------|------|-------|
| RED | Add failing tests for prgm_mode gate | `e65a59e` | test | prgm_mode_tests.rs |
| 1 GREEN | extend flush_entry_buf() to route PushNum to program in prgm_mode | `a67fa75` | feat | ops/mod.rs |
| 2 GREEN | add prgm_mode recording gate to dispatch() | `4e9cdf9` | feat | ops/mod.rs |

## Decisions Made

- prgm_mode gate inserted between `flush_entry_buf()` call and `match op {}` — this ordering ensures the pending digit entry (PushNum) is flushed into `state.program` BEFORE the gate intercepts the triggering op. If the gate came first, entry_buf would not be flushed in recording mode.
- `Op::PrgmMode` while recording exits immediately without self-recording (HP-41 toggle behavior, Pitfall 6 from 03-CONTEXT.md)
- `lift_enabled` not modified during prgm_mode flush — recording is a non-executing path that must not alter execution state
- The 8 individual Phase 3 stub arms collapsed to one `_ => Err(HpError::InvalidOp)` because the prgm_mode gate intercepts all Phase 3 ops during recording; in execute mode, they still return InvalidOp (correct stub behavior until plan 03-06)

## Deviations from Plan

None — plan executed exactly as written. The plan's code examples matched the existing file structure precisely. No Rule 1/2/3 auto-fixes were needed.

## Issues Encountered

None. All verification checks passed on first attempt.

## Known Stubs

None — all functionality specified in the plan is fully implemented. Phase 3 op execution (Lbl/Gto/Xeq/Rtn/Test/Isg/Dse) remains stubbed at `Err(HpError::InvalidOp)` in execute mode, but that is intentional and tracked in plan 03-06, not this plan's scope.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The `state.program.push(op)` path accepts Op values from the caller — the threat model notes this as T-03-04-01 (accept, caller is hp41-cli validated in Phase 4). T-03-04-02 (unbounded Vec growth) is deferred to Phase 4 as documented in the plan's threat register.

## User Setup Required

None.

## Next Phase Readiness

- Plan 03-05 can implement `run_program()` using `state.program`, `state.pc`, `state.call_stack`, and `state.is_running`
- Plan 03-06 can replace the `_ => Err(HpError::InvalidOp)` catch-all with specific arms for Lbl/Gto/Xeq/Rtn/Test/Isg/Dse
- `prgm_mode_tests.rs` provides a regression suite for any future changes to dispatch routing

---
*Phase: 03-programming-engine*
*Completed: 2026-05-07*
