---
phase: 05-persistence-and-ux
plan: 07
subsystem: ux-user-mode
tags: [rust, ratatui, tui, user-mode, key-assignments, run_program, f1-f4]

# Dependency graph
requires:
  - phase: 05-persistence-and-ux/05-06
    provides: UserMode toggle binding ('u'), Ctrl+A key assignment modal, key_assignments BTreeMap on CalcState
  - phase: 05-persistence-and-ux/05-01
    provides: CalcState fields user_mode and key_assignments
provides:
  - try_user_dispatch() method in App: USER mode key assignment lookup → run_program()
  - F1-F4 pre-wired to USER labels a/b/c/d when user_mode is active
  - Complete USER mode runtime dispatch (D-28)
affects: [06-science-and-engineering, 07-hardening]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "try_user_dispatch(): early-return guard pattern — returns bool to signal key consumed"
    - "F1-F4 pre-wired USER key block placed after try_user_dispatch, before overlay nav"

key-files:
  created: []
  modified:
    - hp41-cli/src/app.rs

key-decisions:
  - "try_user_dispatch() placed in handle_key() AFTER pending_input and ALPHA mode guards, BEFORE overlay navigation — consistent with D-28 dispatch priority"
  - "F1-F4 USER keys consume the keypress even when no assignment exists (return; regardless), matching HP-41 behavior where F-keys in USER mode do not fall through to other functions"
  - "try_user_dispatch() returns false for non-Char keys (F-keys handled by separate F1-F4 block)"

patterns-established:
  - "USER mode dispatch guard (try_user_dispatch) must come before overlay navigation and digit entry blocks"
  - "F-key USER pre-wiring uses dedicated block after try_user_dispatch; F1-F4 unconditionally consumed in USER mode"

requirements-completed: [UX-02]

# Metrics
duration: 15min
completed: 2026-05-07
---

# Phase 5 Plan 07: USER Mode Runtime Dispatch Summary

**try_user_dispatch() wires key_assignments lookup to run_program(); F1-F4 pre-mapped to HP-41 USER labels a/b/c/d**

## Performance

- **Duration:** 15 min
- **Started:** 2026-05-07T00:00:00Z
- **Completed:** 2026-05-07T00:15:00Z
- **Tasks:** 1 (+ human-verify checkpoint, approved)
- **Files modified:** 1

## Accomplishments
- Implemented `try_user_dispatch()` in `App`: when `user_mode` is true and the pressed char key has a `key_assignments` entry, `run_program()` is called and the key is consumed (returns `true`)
- Wired `try_user_dispatch()` into `handle_key()` at the correct priority (after pending_input and ALPHA guards, before overlay navigation and digit entry)
- Added F1-F4 USER key block: in USER mode, F1→'a', F2→'b', F3→'c', F4→'d' look up assignments and call `run_program()`; unconditionally consumed in USER mode
- Added three unit tests: dispatch-runs-program, skipped-when-off, no-assignment
- All 39 hp41-cli tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add try_user_dispatch() and F1-F4 USER key wiring** - `57c2cae` (feat)

**Plan metadata:** (docs commit — see final commit)

## Files Created/Modified
- `hp41-cli/src/app.rs` - Added `try_user_dispatch()` method, F1-F4 USER key block in `handle_key()`, three new tests

## Decisions Made
- `try_user_dispatch()` handles `KeyCode::Char` only; F-keys are handled by the dedicated F1-F4 block below it — clean separation of concerns
- F1-F4 unconditionally consume keypress in USER mode (even if no assignment) — matches HP-41 hardware behavior where USER-mode F-keys do not route to any non-USER function
- No changes to `keys.rs` required; F1-F4 stubs already return `None` there, and the new handling in `handle_key()` fires before `key_to_op()` is reached

## Deviations from Plan

None — plan executed exactly as written. Previous agent had reverted a test-only commit before implementation; this execution implemented the full feature cleanly in one commit.

## Issues Encountered

The previous executor agent attempted this task but reverted its test commit before implementing the feature, leaving the worktree with only reverted work and no SUMMARY.md. This agent re-executed Task 1 from scratch following the TDD flow inline (implementation + tests in one commit since the checkpoint was pre-approved by the human).

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness
- Complete Phase 5 (Persistence & UX) feature set is now fully implemented:
  - Ctrl+S save, 30s auto-save, state round-trip (Plans 03)
  - HELP_DATA static array, sample programs module (Plan 04)
  - Help overlay, program library overlay, USER annunciator (Plan 05)
  - STO/RCL modal, ALPHA mode, Ctrl+A assignment modal, 'u' toggle (Plan 06)
  - USER mode runtime dispatch via try_user_dispatch + F1-F4 (this plan)
- Phase 6 (Science & Engineering) can proceed — no UX blockers remain

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
