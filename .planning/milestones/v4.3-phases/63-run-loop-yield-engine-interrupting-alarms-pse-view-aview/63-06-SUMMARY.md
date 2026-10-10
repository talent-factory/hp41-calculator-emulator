---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
plan: "06"
subsystem: hp41-gui/src
tags: [gui-run-loop, yield-driver, pending-yield, pse-view-aview, alarm-missing, r-s-routing, d-25.6, d-11, no-polling]

dependency_graph:
  requires:
    - phase: 63-04
      provides: run_program / resume_program Tauri commands + pending_yield CalcStateView projection (YieldView)
    - phase: 63-02
      provides: run_loop yield arms (PSE/VIEW/AVIEW) + Phase-C alarm check + ack-after-RTN
  provides:
    - R/S 4-way state-routed dispatch: modal > cancel > run-loop-stop > run-loop-start (run_program)
    - yield-and-resume driver: setTimeout(resume_ms) → resume_program → repeat until pending_yield null
    - display precedence: modal > pending_yield.text > display_override > display_str
    - alarm:missing toast (D-07/D-08); alarm:interrupting silent-ignore arm removed
  affects:
    - Phase 64 (GETKEY) — builds on same suspend/resume infrastructure
    - Phase 66 (verification) — GUI criteria 1/4/5 now reachable

tech-stack:
  added: []
  patterns:
    - "yield-and-resume loop via command return values (no get_state polling — D-11)"
    - "resumeScheduledRef single-flight guard pattern (mirrors busyRef)"
    - "4-way R/S routing: modal > cancel > stop > start (D-25.6 CLI↔GUI parity)"

key-files:
  created: []
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx

key-decisions:
  - "63-06-D01: R/S branch 3 (stopped, no modal) now calls run_program('A'), replacing run_stop; label 'A' mirrors CLI F5 path (app.rs run_program('A'), D-16)"
  - "63-06-D02: yield-driver implemented as a useEffect on calcState?.pending_yield — fires on every state update, guards double-scheduling with resumeScheduledRef; no setInterval, no get_state poll (D-11)"
  - "63-06-D03: alarm:interrupting silent-ignore arm removed — interrupting alarms now execute entirely server-side inside run_program/resume_program via Phase-C; they no longer appear in event_buffer"
  - "63-06-D04: vi.useFakeTimers({ shouldAdvanceTime: true }) used in P1 so waitFor polling (setInterval-based) still resolves while explicit advanceTimersByTime controls the yield-driver setTimeout"

patterns-established:
  - "Yield-and-resume loop: run_program returns pending_yield → useEffect schedules setTimeout(resume_ms) → resume_program returns new view → setCalcState → useEffect fires again → terminates when pending_yield null"
  - "Single-flight ref guard: resumeScheduledRef.current = true before setTimeout; cleared in .finally() to prevent duplicate scheduling across React renders"

requirements-completed: [ALARM-02, ALARM-03, PRGM-01, PRGM-02]

duration: 25min
completed: "2026-06-06"
tasks_completed: 2
tasks_total: 2
files_changed: 2
---

# Phase 63 Plan 06: GUI Run-Loop Driver Summary

**GUI yield-and-resume run-loop driver: R/S 4-way routing starts run_program('A'), pending_yield auto-resumes via setTimeout → resume_program, alarm:missing toasts, alarm:interrupting stub removed; 340 TS tests green.**

---

## Performance

- **Duration:** 25 min
- **Started:** 2026-06-06T18:50:00Z
- **Completed:** 2026-06-06T18:58:00Z
- **Tasks:** 2 / 2
- **Files modified:** 2

---

## Accomplishments

- R/S 4-way routing: branch 3 (stopped, no modal) now invokes `run_program({ label: 'A' })` instead of `run_stop`, matching CLI F5 parity (D-25.6/D-16). Branches 1 (modal) and 2 (cancel) unchanged.
- Yield-and-resume driver: `useEffect` on `calcState?.pending_yield` schedules `invoke('resume_program')` after `resume_ms` via `setTimeout`; guarded by `resumeScheduledRef` single-flight ref. Loop terminates when `pending_yield` is null. No `get_state` polling (D-11).
- Display precedence: `modal > pending_yield.text > display_override > display_str` — yield text appears on the main LCD during PSE/VIEW/AVIEW pauses without routing through `display_override` (D-04 preserved).
- `alarm:interrupting:` silent-ignore arm removed (D-38.4 deferred stub gone; interrupting alarms now execute server-side via Phase-C/63-02 interrupt boundary).
- `alarm:missing:` arm added: emits `showToast("Alarm XEQ {label}: label not found")` (D-07/D-08 never-discard).
- 340 Vitest tests green (337 prior + 3 new: P1 yield render + auto-resume, P2 alarm:missing toast, P3 R/S start routing); H3 updated to expect `run_program` (old `run_stop` expectation replaced).

---

## Task Commits

1. **Task 1: R/S 4-way start/stop routing + yield-and-resume run-loop driver** — `7a12cdb` (feat)
2. **Task 2: alarm:missing toast + remove alarm:interrupting silent-ignore + Vitest tests** — `1df166f` (test)

---

## Files Created/Modified

- `hp41-gui/src/App.tsx` — pending_yield CalcStateView field; R/S branch 3 → run_program('A'); resumeScheduledRef; yield-driver useEffect; display precedence update; alarm:missing arm; alarm:interrupting arm removed
- `hp41-gui/src/App.test.tsx` — pending_yield in CalcStateView + makeEmptyView; H3 updated; Group P tests (P1/P2/P3)

---

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written, modulo one minor test approach deviation:

**1. [Rule 3 - Blocking] vi.useFakeTimers() timing interaction with renderAppAndWait**
- **Found during:** First test run — P1/P2/P3 timed out at 5000ms
- **Issue:** `vi.useFakeTimers()` without `shouldAdvanceTime:true` froze `waitFor`'s internal polling (which uses setInterval), causing all three Group P tests to hang.
- **Fix:** Used `vi.useFakeTimers({ shouldAdvanceTime: true })` so real-time promise resolution still works while `vi.advanceTimersByTime(1000)` controls the yield-driver setTimeout. Added `afterEach(vi.useRealTimers)` to the Group P describe block to prevent fake-timer leakage.
- **Files modified:** hp41-gui/src/App.test.tsx
- **Commit:** `1df166f`

---

## Known Stubs

None — the yield-and-resume loop is fully wired end-to-end. run_program/resume_program are real Tauri commands (63-04). The loop terminates naturally when pending_yield is null.

---

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.
- T-63-07 (DoS — double-scheduling): mitigated by `resumeScheduledRef` single-flight guard; loop terminates when server returns `pending_yield: null` (MAX_STEPS in core bounds it).
- T-63-11 (DoS — R/S start while already running): branch 2 (is_running) handles cancel; branch 3 only fires when stopped (no concurrent run_program storm).
- T-63-SC (no new npm deps): `git diff HEAD -- hp41-gui/package.json` empty.

---

## Self-Check: PASSED

- `hp41-gui/src/App.tsx` — FOUND; contains `run_program`, `resume_program`, `pending_yield`, `resumeScheduledRef`, `alarm:missing:`
- `hp41-gui/src/App.test.tsx` — FOUND; contains `pending_yield`, `resume_program`, `run_program`, Group P tests
- Commit `7a12cdb` — FOUND in `git log --oneline`
- Commit `1df166f` — FOUND in `git log --oneline`
- `grep -n "alarm:interrupting" hp41-gui/src/App.tsx` — EMPTY (good)
- `grep -c "resume_program|run_program" hp41-gui/src/App.tsx` — 18 (≥ 2; good)
- `just gui-ci` — 129 Rust + 340 TS tests PASS
- Info.plist — NOT staged (still dirty, not touched)
