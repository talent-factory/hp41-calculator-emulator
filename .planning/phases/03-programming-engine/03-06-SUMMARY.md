---
phase: 03-programming-engine
plan: "06"
subsystem: programming-engine
tags: [rust, hp41, dispatch, ops-mod, lib-rs, run_program, TestKind, StoArithKind, wiring, ci-gate]

# Dependency graph
requires:
  - phase: 03-programming-engine
    plan: "04"
    provides: prgm_mode gate in dispatch() + Op::PrgmMode arm (partial)
  - phase: 03-programming-engine
    plan: "05"
    provides: all op_* dispatch functions in ops/program.rs; run_program entry point

provides:
  - ops/mod.rs: complete dispatch() with all 8 Phase 3 Op variant arms (no catch-all)
  - lib.rs: pub use ops::program::run_program (public API)
  - lib.rs: pub use ops::{StoArithKind, TestKind} (test accessibility)

affects: [hp41-cli dispatch integration, consumers using hp41_core::run_program]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Isg/Dse dispatch arms discard Result<bool> via .map(|_| ()) — skip signal only meaningful in run_loop"
    - "pub use ops::program::run_program exposes interpreter entry point at crate root"
    - "pub use ops::{StoArithKind, TestKind} exposes enums at crate root for integration test access"

key-files:
  created: []
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/lib.rs

key-decisions:
  - "Isg/Dse in interactive dispatch discard bool skip signal (.map(|_| ())) — correct HP-41 semantics: ISG/DSE skip is only meaningful inside the interpreter loop, not interactive key presses"
  - "TestKind and StoArithKind added to lib.rs pub use — required for integration tests to import them as hp41_core::TestKind"

requirements-completed: [PROG-01, PROG-02]

# Metrics
duration: ~5min
completed: 2026-05-07T08:35:43Z
---

# Phase 03 Plan 06: Phase 3 Dispatch Wiring + CI Gate Summary

**All 8 Phase 3 Op variants wired in dispatch(); run_program and TestKind exported from lib.rs; just ci green (281 tests, 81.62% coverage)**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-07T08:30:00Z
- **Completed:** 2026-05-07T08:35:43Z
- **Tasks:** 2 (Task 1: wire dispatch arms + lib.rs exports; Task 2: just ci gate)
- **Files created:** 0
- **Files modified:** 2 (ops/mod.rs, lib.rs)

## Accomplishments

- `ops/mod.rs` `dispatch()` updated:
  - Removed catch-all `_ => Err(HpError::InvalidOp)` arm
  - Added explicit arms for all 8 Phase 3 Op variants:
    - `Op::Lbl(_)` → `program::op_lbl(state)`
    - `Op::Gto(s)` → `program::op_gto(state, &s)`
    - `Op::Xeq(s)` → `program::op_xeq(state, &s)`
    - `Op::Rtn` → `program::op_rtn(state)`
    - `Op::PrgmMode` → `program::op_prgm_mode(state)` (was already present from 03-05 deviation)
    - `Op::Test(kind)` → `program::op_test(state, kind)`
    - `Op::Isg(reg)` → `program::op_isg(state, reg).map(|_| ())`
    - `Op::Dse(reg)` → `program::op_dse(state, reg).map(|_| ())`
- `lib.rs` updated with three new public re-exports:
  - `pub use ops::program::run_program` — interpreter entry point at crate root
  - `pub use ops::{StoArithKind, TestKind}` — enum types at crate root
- `just ci` passes: clippy clean, 280 tests green, 81.62% coverage (above 80% gate)

## Task Commits

| # | Task | Commit | Type | Files |
|---|------|--------|------|-------|
| 1 | Wire Phase 3 dispatch arms + export run_program | `2d1e4eb` | feat | ops/mod.rs, lib.rs |
| 2 | Run just ci — full quality gate | (no files changed) | — | — |

## Files Created/Modified

- `hp41-core/src/ops/mod.rs` — replaced 7-line catch-all block with 14-line explicit Phase 3 arms
- `hp41-core/src/lib.rs` — added 2 pub use lines (run_program, StoArithKind+TestKind)

## Decisions Made

- `Op::PrgmMode` was already wired in plan 03-05 as a Rule 3 deviation; left in place (not duplicated). The new match block organizes all Phase 3 arms together for clarity.
- `Isg`/`Dse` return `Result<bool>` internally but dispatch only returns `Result<()>` — discard the bool skip signal via `.map(|_| ())`. Skip semantics are only meaningful in `run_loop`.

## Deviations from Plan

None — plan executed exactly as written.

The `Op::PrgmMode` arm was already present from plan 03-05's Rule 3 deviation. Plan 03-06 consolidates all Phase 3 arms in one coherent block, which implicitly subsumes the 03-05 partial wiring. No behavior changed.

## Known Stubs

None — all Phase 3 dispatch arms are fully implemented and calling real functions.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. T-03-06-01 (GTO label string) and T-03-06-02 (ISG/DSE discard) both accepted per the plan's threat register — no mitigations needed.

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `hp41-core/src/ops/mod.rs` — `program::op_lbl` appears exactly 1 time | FOUND |
| `hp41-core/src/ops/mod.rs` — `Op::Lbl` appears 2 times (doc + arm) | FOUND |
| `hp41-core/src/lib.rs` — `pub use ops::program::run_program` appears 1 time | FOUND |
| Commit `2d1e4eb` (Task 1) exists | FOUND |
| `cargo test -p hp41-core` exits 0 with 280 tests | PASSED |
| `just ci` exits 0, lint clean, 81.62% coverage | PASSED |
