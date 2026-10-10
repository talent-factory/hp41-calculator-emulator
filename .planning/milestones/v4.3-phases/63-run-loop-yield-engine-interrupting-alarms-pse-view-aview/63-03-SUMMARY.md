---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
plan: "03"
subsystem: hp41-cli
tags: [yield-engine, pse, view, aview, alarm-missing, run-loop, cli-driver, d-25-6-parity]
dependency_graph:
  requires:
    - pending_yield field on CalcState with YieldState{kind, text, resume_ms} (63-01)
    - resume_program exported from hp41_core (63-01)
    - PSE/VIEW/AVIEW yield arms in run_loop setting pending_yield + break (63-02)
    - alarm:missing:{label} event emitted by run_loop interrupt boundary on missing label (63-02)
  provides:
    - drain_pending_yields(terminal): render yield text, sleep resume_ms, call resume_program, drain outputs
    - yield drain loop called from run() after handle_key — has terminal access for redraw
    - alarm:missing:{label} arm in drain_event_buffer → self.message (D-08 surface, D-07 never-swallow)
    - Removed old alarm:interrupting silent-ignore comment (dead code since 63-01)
    - Targeted test drain_event_alarm_missing asserting status-line surface
  affects:
    - hp41-cli/src/app.rs (drain_pending_yields + run() yield loop + drain_event_buffer alarm:missing arm)
    - Phase 63-04 (GUI Rust) — parallel plan landing CLI half of D-25.6 in same phase
    - Phase 63-06 (GUI TS) — builds on the same yield-and-resume concept for the App.tsx driver
tech_stack:
  added: []
  patterns:
    - "drain_pending_yields(terminal) helper: while pending_yield.is_some() → entry_buf temp display → draw → sleep → resume_program → drain"
    - "entry_buf as temporary display carrier during yield: priority 3 in get_display_string, always empty during program execution"
    - "alarm:missing arm in drain_event_buffer: strip_prefix + format! → self.message (same pattern as alarm:message arm)"
key_files:
  created: []
  modified:
    - hp41-cli/src/app.rs
key-decisions:
  - "63-03-D01: Yield loop lives in run() not handle_key — terminal borrow only available in run(); handle_key does not carry terminal ref"
  - "63-03-D02: Use entry_buf as temporary display carrier for yield text — priority 3 in get_display_string, always empty during program execution, no ui.rs modification needed (file isolation constraint)"
  - "63-03-D03: Single commit for both tasks — they share the same file (app.rs); splitting would require revert+re-apply which is error-prone with no benefit"
patterns-established:
  - "drain_pending_yields pattern: terminal-aware helper that renders then sleeps then resumes — models for future yield consumers"

requirements-completed: [ALARM-02, ALARM-03, PRGM-01, PRGM-02]

duration: ~20min
completed: "2026-06-06"
---

# Phase 63 Plan 03: CLI Yield Render + Sleep + Resume Loop + alarm:missing Surface Summary

**CLI yield drain loop (PSE/VIEW/AVIEW render-sleep-resume via entry_buf + terminal.draw) + alarm:missing status-line surface; 489 hp41-cli tests pass, clippy clean.**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-06-06T16:08:00Z
- **Completed:** 2026-06-06T16:28:47Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- `drain_pending_yields(terminal)` helper loops while `state.pending_yield.is_some()`: shows yield text via `entry_buf` (display priority 3), redraws terminal so value is visible, sleeps `resume_ms` ms, calls `resume_program`, drains print/card output (I-07)
- Yield loop called from `run()` immediately after `handle_key` returns — `terminal` is available there; handles consecutive yields (PSE → VIEW) in one loop
- `alarm:missing:{label}` arm added to `drain_event_buffer` → `self.message = "Alarm XEQ {label}: label not found"` (D-08 / D-07)
- Old silent-ignore arm for the now-defunct `alarm:interrupting:` string removed; doc comment updated to Phase 63 routing
- Targeted test `drain_event_alarm_missing` verifies the status-line arm in isolation

## Task Commits

Both tasks target the same file (`hp41-cli/src/app.rs`) and were committed together:

1. **Task 1 + Task 2: Yield render loop + alarm:missing arm** — `3bf055f` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `hp41-cli/src/app.rs` — Added `drain_pending_yields(terminal)` helper, yield loop in `run()`, `alarm:missing:` arm in `drain_event_buffer`, targeted test

## Decisions Made

- **63-03-D01:** Yield loop lives in `run()`, not `handle_key` — `terminal: DefaultTerminal` borrow is only available in `run()`; `handle_key` receives no terminal reference and extracting it would require API changes out of scope for this plan.
- **63-03-D02:** Use `state.entry_buf` as temporary display carrier during yield — `entry_buf` has priority 3 in `get_display_string` (above X register), is always empty during program execution, and allows the yield text to be shown without modifying `ui.rs` (which would violate the file isolation constraint of wave 3 parallelism).
- **63-03-D03:** Tasks 1 and 2 share the same file; single commit is correct (splitting would require staged-hunk surgery with no benefit).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Pre-existing] "silently ignored" comment at line 2655 in EEX digit test**
- **Found during:** Task 2 verification (`grep -n "silently ignored" hp41-cli/src/app.rs`)
- **Issue:** The plan's done criterion checks `grep -n "silently ignored\|alarm:interrupting"` returns empty. Line 2655 has "Third digit must be silently ignored." in an EEX input test — a pre-existing, unrelated comment about digit capping behavior, not alarm routing.
- **Fix:** No change needed — the comment is correct and unrelated to alarms. The alarm-related silent-ignore at line 1797 was removed. The pre-existing test comment is out of scope per the scope boundary rule.
- **Files modified:** none (pre-existing condition documented)
- **Committed in:** N/A

---

**Total deviations:** 1 pre-existing out-of-scope (documented, not fixed)
**Impact on plan:** No scope creep. The alarm silent-ignore that the plan intended to remove was removed. The unrelated test comment is correct behavior.

## Issues Encountered

None — implementation was straightforward given the 63-01/63-02 infrastructure.

## Known Stubs

None — `drain_pending_yields` is a real implementation. The yield text display uses `entry_buf` temporarily (cleared before `resume_program`). All arms of `drain_event_buffer` are wired.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes. T-63-06 (yield resume loop DoS) is correctly bounded: the loop terminates when `pending_yield` is None after `resume_program`, which happens when the program ends or errors. `MAX_STEPS` in the core engine is the ultimate bound.

## Self-Check

- `hp41-cli/src/app.rs` — FOUND and modified
- `drain_pending_yields` method — FOUND in file
- `alarm:missing:` arm in `drain_event_buffer` — FOUND at line ~1861
- Old `alarm:interrupting:` silent-ignore comment — REMOVED
- `drain_event_alarm_missing` test — FOUND in test module
- `cargo test -p hp41-cli` — 489 passed, 0 failed
- `cargo +1.88 clippy -p hp41-cli --all-targets -- -D warnings` — clean (no warnings)
- `just ci` — EXIT 0 (coverage 95.09% >= 95% gate, license-audit clean)
- `git diff HEAD -- hp41-core/` — EMPTY (only app.rs modified)
- `git diff HEAD -- hp41-gui/` — EMPTY (Info.plist pre-existing dirty, NOT staged)

## Self-Check: PASSED

---

*Phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview*
*Completed: 2026-06-06*
