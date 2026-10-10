---
phase: 65-standalone-fidelity-fixes
plan: "03"
subsystem: hp41-cli
tags: [chs, entry-buf, mantissa, in-buffer-edit, fidelity, disp-02]
dependency_graph:
  requires: []
  provides: [DISP-02]
  affects: [hp41-cli/src/app.rs]
tech_stack:
  added: []
  patterns: [early-return in-buffer editing (app.rs EEX-CHS analog)]
key_files:
  modified:
    - hp41-cli/src/app.rs
decisions:
  - "65-03-D01: DISP-02 block placed immediately before the EEX-CHS block — the gate `!entry_buf.contains('e')` is mutually exclusive with the EEX branch, so order only matters for clarity (DISP-02 first, then EEX-CHS)."
metrics:
  duration_minutes: 8
  completed_date: "2026-06-07"
  tasks_completed: 2
  files_modified: 1
---

# Phase 65 Plan 03: DISP-02 CHS Mantissa Sign Toggle Summary

CHS during mantissa entry now toggles a leading '-' in the entry buffer in place — no flush, no dispatch, no stack lift. Matches real HP-41 hardware fidelity (DISP-02).

## What Was Built

Added a new early-return block in `hp41-cli/src/app.rs` immediately before the existing EEX-CHS block. The block is gated on `c == 'n' && !entry_buf.is_empty() && !entry_buf.contains('e')` and toggles a leading `'-'` in place using `remove(0)` / `insert(0, '-')`.

14 new integration tests in `disp02_chs_mantissa_toggle_tests` module cover:
- Core toggle: "123" → "-123" → "123" round-trip (no flush, no dispatch)
- Decimal mantissa: "3.14" → "-3.14"
- Edge case: "0" → "-0" → "0" (display verbatim)
- Regression: "1e2" → CHS still toggles exponent sign (EEX-CHS branch unchanged)
- Regression: empty buffer → CHS still dispatches `Op::Chs` and negates X

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | b3d8d86 | feat(65-03): insert DISP-02 CHS mantissa in-buffer sign toggle |
| 2 | 01acbe9 | test(65-03): add DISP-02 CHS mantissa toggle integration tests |

## Deviations from Plan

None - plan executed exactly as written. The DISP-02 code block from `65-PATTERNS.md` was inserted verbatim, with comments added for clarity per project convention.

## Verification

- `cargo test -p hp41-cli`: 515 passed, 4 ignored (501 before + 14 new DISP-02 tests)
- `just test`: all suites green
- "123" → CHS → "-123" → CHS → "123": confirmed in-buffer toggle, no flush/lift
- Empty-buffer CHS: confirmed dispatches `Op::Chs` (X negated)
- EEX-CHS ("1e2" → CHS → "1e-2"): regression confirmed unchanged

## Known Stubs

None.

## Threat Flags

None — frontend-only entry-buffer string edit on a local keystroke; no new network, auth, or parse surface (T-65-04: accept, per plan threat model).

## Self-Check: PASSED

- `hp41-cli/src/app.rs` modified: confirmed
- Commit b3d8d86 exists: confirmed
- Commit 01acbe9 exists: confirmed
- 515 CLI tests pass: confirmed
