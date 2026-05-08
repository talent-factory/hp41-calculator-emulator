---
phase: 08-tech-debt-cleanup
plan: "03"
subsystem: hp41-cli
tags: [help-data, key-bindings, sin, clreg, documentation]
dependency_graph:
  requires: [08-02]
  provides: [corrected HELP_DATA with accurate q->SIN and g->CLREG entries]
  affects: [hp41-cli/src/help_data.rs]
tech_stack:
  added: []
  patterns: [single-source-of-truth correction]
key_files:
  created: []
  modified:
    - hp41-cli/src/help_data.rs
decisions:
  - "Also removed stale ('q', 'QUIT') entry from Quit section and fixed Help close text 'Esc/q/?' -> 'Esc/?' as Rule 2 auto-fixes (stale bindings mislead users)"
metrics:
  duration_seconds: 120
  completed_date: "2026-05-08"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 1
---

# Phase 8 Plan 03: help_data.rs Corrections Summary

Corrected two stale key-binding entries in HELP_DATA: SIN entry now shows key 'q' (not 'S'), and a new CLREG entry documents the 'g' binding added in Plan 02.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Fix SIN key binding and add CLREG entry in HELP_DATA | 7e43c95 | hp41-cli/src/help_data.rs |

## What Was Built

**help_data.rs changes:**
- `("S", "SIN", ...)` corrected to `("q", "SIN", "Sine of X (in current angle mode)")` in Trig section (line 32)
- `("g", "CLREG", "Clear all storage registers R00-R99 to zero")` added in Registers section (line 77)
- Removed stale `("q", "QUIT", "Quit (saves state first)")` from Quit section (Rule 2 auto-fix)
- Fixed Help close entry from `"Esc/q/?"` to `"Esc/?"` since 'q' no longer closes the overlay (Rule 2 auto-fix)

## Test Results

- hp41-cli: 61 passed, 0 failed
- All existing help_data tests (minimum_entries, ten_categories, no_empty_key_or_op): GREEN
- Full workspace: 0 failures across all test suites

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Fix] Removed stale ("q", "QUIT") entry from Quit section**
- Found during: Task 1, while editing the file
- Issue: The Quit section still had `("q", "QUIT", "Quit (saves state first)")`. Since Plan 02 reassigned 'q' to Op::Sin, this entry was actively misleading — a user reading the help overlay would press 'q' expecting to quit but would instead compute sine.
- Fix: Removed the `("q", "QUIT", ...)` line from the Quit section. Ctrl+C remains documented as the sole quit key.
- Files modified: hp41-cli/src/help_data.rs
- Commit: 7e43c95

**2. [Rule 2 - Missing Critical Fix] Fixed Help close label "Esc/q/?" -> "Esc/?"**
- Found during: Task 1, while editing the file
- Issue: The Help section had `("Esc/q/?", "HELP close", "Close this overlay (Esc, q, or ? again)")`. Since 'q' is now SIN, pressing 'q' in the help overlay triggers SIN computation, not close.
- Fix: Updated key binding label to `"Esc/?"` and description to `"Close this overlay (Esc or ? again)"`.
- Files modified: hp41-cli/src/help_data.rs
- Commit: 7e43c95

## Threat Mitigations Applied

| Threat ID | Mitigation |
|-----------|-----------|
| T-08-06 | Fixed "S"->SIN to "q"->SIN; stale binding no longer misleads user |
| T-08-07 | Added "g"->CLREG entry; function is now discoverable from the help overlay |

## Self-Check: PASSED

- [x] `grep -n '"q", "SIN"' hp41-cli/src/help_data.rs` returns 1 result (line 32)
- [x] `grep -c '"S", "SIN"' hp41-cli/src/help_data.rs` returns 0
- [x] `grep -n '"g", "CLREG"' hp41-cli/src/help_data.rs` returns 1 result (line 77)
- [x] `grep -c '"q", "QUIT"' hp41-cli/src/help_data.rs` returns 0
- [x] Commit 7e43c95 exists in git log
- [x] just test: 0 failures across full workspace
