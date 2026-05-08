---
phase: 10-sto-arithmetic-modals
plan: "03"
subsystem: hp41-cli/help
tags: [help-overlay, ux, sto-arithmetic]
dependency_graph:
  requires: []
  provides: [corrected-sto-arith-help-entries]
  affects: [help-overlay-display]
tech_stack:
  added: []
  patterns: [static-help-data]
key_files:
  created: []
  modified:
    - hp41-cli/src/help_data.rs
decisions:
  - "Use actual UTF-8 em dash inline (matches existing file convention) not escape sequence"
  - "Expand single-line entries to multi-line for consistency with surrounding entries"
metrics:
  duration: "3 minutes"
  completed: "2026-05-08T12:03:06Z"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 1
---

# Phase 10 Plan 03: Fix STO Arithmetic Help Entries Summary

**One-liner:** Replaced four stale `Shift+R+/-/*/` placeholder keys with `S +/-/*/` matching the actual 3-step modal, with descriptions citing `nn or Y/Z/T/L` targets.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Correct four STO arithmetic help entries | a2a1440 | hp41-cli/src/help_data.rs |

## What Was Built

The `?` help overlay in `hp41-cli` previously showed four incorrect STO arithmetic entries:
- Key column: `Shift+R+` / `Shift+R-` / `Shift+R*` / `Shift+R/` — these keys do not exist
- Descriptions: incomplete, no mention of stack register targets

The entries now read:
- Key column: `S +` / `S -` / `S *` / `S /` — matching the actual 3-step modal sequence
- Descriptions: "Add X to register nn or stack Y/Z/T/L — press S then +, then nn or Y/Z/T/L" (and analogously for sub/mul/div)

The surrounding `Shift+R` (plain STO) and `g` (CLREG) entries are unchanged.

## Acceptance Criteria Verified

- `grep -c "Shift+R+" hp41-cli/src/help_data.rs` → 0 (placeholder fully removed)
- `grep -c '"S +"' hp41-cli/src/help_data.rs` → 1
- `grep -c '"S -"' hp41-cli/src/help_data.rs` → 1
- `grep -c '"S \*"' hp41-cli/src/help_data.rs` → 1
- `grep -c '"S /"' hp41-cli/src/help_data.rs` → 1
- `grep -c "Y/Z/T/L" hp41-cli/src/help_data.rs` → 4
- `just build` → exit 0
- `just ci` → exit 0 (94.22% coverage, 461 tests passing)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all four entries are complete, accurate descriptions of the implemented modal behavior.

## Threat Flags

None — static string constant, no security-relevant surface.

## Self-Check: PASSED

- hp41-cli/src/help_data.rs: modified with correct entries
- Commit a2a1440 exists: confirmed via git log
- All acceptance criteria verified with grep counts
