---
phase: 66-verification-divergence-doc-updates-quality-gates
plan: "01"
subsystem: hp41-core/error
tags: [error-handling, printer, unc-02, thiserror]
dependency_graph:
  requires: []
  provides: [HpError::NonExistent]
  affects: [hp41-core/src/error.rs]
tech_stack:
  added: []
  patterns: [thiserror-error-attribute]
key_files:
  created: []
  modified:
    - hp41-core/src/error.rs
decisions:
  - "Display string is lowercase 'nonexistent' matching OM p.57-58 NONEXISTENT error-message class (not 'printer not connected' or any other paraphrase)"
  - "Variant placed after NoRoom to keep X-MEM variants grouped and printer variant at end"
metrics:
  duration: "1m 33s"
  completed: "2026-06-10T07:16:54Z"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 1
---

# Phase 66 Plan 01: Add HpError::NonExistent Variant Summary

**One-liner:** Added `HpError::NonExistent` with `#[error("nonexistent")]` for HP-41C OM p.57-58 NONEXISTENT printer-not-present error class (UNC-02 blocker, unblocks Plan 66-02).

## What Was Done

Added one new enum variant to `hp41-core/src/error.rs`:

```rust
/// Printer not present: PRX/PRA/PRSTK executed when neither flag 55
/// (Printer Existence) nor flag 21 (Printer Enable) is set. Matches the
/// HP-41C OM p.57-58 NONEXISTENT error-message class: "An attempt was made
/// to execute a specific print function when the printer was not connected
/// to the system." (UNC-02, Phase 66).
#[error("nonexistent")]
NonExistent,
```

Added two unit tests:
- `non_existent_display` — asserts `HpError::NonExistent.to_string() == "nonexistent"`
- `non_existent_distinct_from_invalid_op` — variant distinctness guard

## Verification

- `cargo check -p hp41-core` — clean (0 errors, 0 warnings)
- `cargo test -p hp41-core -- error::tests` — 8 passed (6 pre-existing + 2 new)
- `cargo +1.88 clippy -p hp41-core -- -D warnings` — clean (MSRV compliance confirmed)

## Deviations from Plan

None — plan executed exactly as written.

## Commits

| Task | Description | Hash | Files |
|------|-------------|------|-------|
| 1 | Add HpError::NonExistent variant | 9723a20 | hp41-core/src/error.rs |

## Self-Check: PASSED

- hp41-core/src/error.rs modified with NonExistent variant: FOUND
- Commit 9723a20 present: FOUND
- `cargo check -p hp41-core` passes: CONFIRMED
- 8 error module tests pass: CONFIRMED
