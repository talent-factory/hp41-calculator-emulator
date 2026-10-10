---
phase: 67-reset-escape-hatch
plan: 04
subsystem: ui
tags: [react, tauri, ios, portals, vitest, reset]

# Dependency graph
requires:
  - phase: 67-reset-escape-hatch/67-03
    provides: reset_soft and reset_full Tauri commands (IPC contract + permissions)
provides:
  - ON-key tap handler invoking reset_soft outside key_map.resolve()/dispatch_op
  - ON-key long-press (~600ms) opening a createPortal-ed MEMORY LOST confirm sheet
  - reset_full invoked only on explicit Confirm; no double-fire on pointer-up after long-press
  - Frontend transient state (shiftActive, pendingInput) cleared on both reset tiers
  - Vitest coverage (4 tests): tap, long-press, confirm/cancel, no-double-fire, portal cleanup
affects: [67-reset-escape-hatch/67-05, gui-ios-layout, gui-key-dispatch]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ON-key bypass: dedicated pointer props (onOnPointerDown/Up/Cancel) on Keyboard — never flows through handleKeyClick/invokeForKey/key_map.resolve()"
    - "createPortal to document.body for the confirm sheet — escapes the CSS transform:scale ancestor (ADR-v4.1-005)"
    - "Refs (not state) for timer id + fired flag — prevents re-render reset of timer state"
    - "longPressFiredRef no-double-fire guard — pointer-up after long-press does nothing"

key-files:
  created: []
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/Keyboard.tsx
    - hp41-gui/src/App.css
    - hp41-gui/src/App.test.tsx

key-decisions:
  - "67-04-D01: Wiring via dedicated Keyboard props (onOnPointerDown/Up/Cancel) rather than special-casing inside App.tsx handleClick — keeps the ON bypass cleanly in Keyboard and avoids any path through handleKeyClick's guard."
  - "67-04-D02: Refs for timer id + longPressFiredRef — re-renders must not reset the timer or allow double-fire; state variables would reset on intermediate renders."
  - "67-04-D03: createPortal to document.body for the MEMORY LOST confirm sheet — identical to the print bottom-sheet portal precedent; the transform:scale ancestor must not be the containing block for position:fixed overlays (reference_ios_gui_layout_gotchas)."

patterns-established:
  - "Escape-hatch key pattern: empty-id ON key bypasses the normal resolve chain via explicit pointer props; the confirm sheet portals to document.body."

requirements-completed: [RESET-01]

# Metrics
duration: 25min
completed: 2026-06-10
---

# Phase 67 Plan 04: GUI ON-Key Escape Hatch Summary

**ON-key wired outside key_map.resolve() as a two-tier GUI escape hatch: tap=reset_soft, long-press+confirm=reset_full, MEMORY LOST sheet portaled to document.body (escapes transform:scale), 4 vitest tests green, human-verify on-device PASSED**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-06-10T15:30:00Z
- **Completed:** 2026-06-10T15:55:00Z
- **Tasks:** 2 auto + 1 human-verify checkpoint
- **Files modified:** 4

## Accomplishments

- ON key (empty id, top-left) now invokes `reset_soft` on tap (<600 ms) and opens a portaled MEMORY LOST confirm sheet on long-press (>=600 ms), completely outside the `handleKeyClick`/`invokeForKey`/`key_map.resolve()`/`dispatch_op` chain — the escape hatch works even when dispatch is stuck.
- Confirm sheet rendered via `createPortal(..., document.body)` so the `transform:scale` ancestor is not its containing block on iOS and desktop; verified on-device to be centered and fully hittable.
- `longPressFiredRef` no-double-fire guard: pointer-up after a fired long-press does not also invoke `reset_soft`.
- Both reset tiers clear frontend transient state: `shiftActive=false`, `pendingInput` cleared.
- 4 Phase-67 vitest tests cover all branches headlessly; portal nodes queried on `document.body`; `afterEach(cleanup)` + `vi.useRealTimers()` guard portal/timer leaks.

## Task Commits

1. **Task 1: Wire ON-key tap/long-press handler + portaled confirm sheet** - `0584129` (feat)
2. **Task 2: vitest coverage (tap, long-press, confirm, no-double-fire, portal cleanup)** - `aa26232` (test)
3. **STATE.md progress update (tasks 1+2 done, awaiting checkpoint)** - `6cdb227` (docs)

## Files Created/Modified

- `hp41-gui/src/App.tsx` — ON-key pointer handlers, longPressFiredRef guard, reset invokes, portaled MemoryLostSheet component, transient state clearing on reset
- `hp41-gui/src/Keyboard.tsx` — `onOnPointerDown`/`onOnPointerUp`/`onOnPointerCancel` props; ON key `<g>` rendered with dedicated pointer handlers; `handleKeyClick` guard unchanged (`if (!key.id) return`)
- `hp41-gui/src/App.css` — `.on-key-confirm-overlay`, `.on-key-confirm-sheet`, `.on-key-confirm-btn` styles for the portaled confirm sheet
- `hp41-gui/src/App.test.tsx` — Group Q (Phase 67 Plan 04), 4 tests: Q1 tap, Q2 long-press sheet, Q3 confirm/cancel, Q4 no-double-fire; portal queried on `document.body`

## Decisions Made

- **67-04-D01:** Wiring via dedicated Keyboard props (`onOnPointerDown/Up/Cancel`) rather than special-casing inside App.tsx `handleClick` — keeps the ON bypass cleanly in Keyboard and avoids any path through `handleKeyClick`'s guard.
- **67-04-D02:** Refs (`longPressTimerRef`, `longPressFiredRef`) for timer id + fired flag — re-renders must not reset the timer or allow double-fire; state variables would reset on intermediate renders.
- **67-04-D03:** `createPortal` to `document.body` for the MEMORY LOST confirm sheet — identical to the print bottom-sheet portal precedent; the `transform:scale` ancestor must not be the containing block for `position:fixed` overlays (reference_ios_gui_layout_gotchas).

## Human-Verify Checkpoint (T-67-10)

**Result: PASSED — approved by human on 2026-06-10**

All 5 on-device steps confirmed:
1. Tap ON key → soft reset; stack/working state clears; SHIFT clears; stored data preserved.
2. Long-press ON key → MEMORY LOST confirm sheet appears centered and fully visible; portal correctly escapes transform:scale; Cancel closes sheet with no data loss.
3. Long-press + Confirm → factory state (programs/registers wiped).
4. Pointer-up after long-press does NOT trigger a second soft reset (no-double-fire guard confirmed).
5. Portal rendering verified unclipped on iOS build.

Threat T-67-10 (confirm sheet unhittable inside transform:scale) mitigated and verified.

## Deviations from Plan

None — plan executed exactly as written. The implementation approach (Keyboard prop wiring vs. App.tsx special-case) was left open in the plan; the executor chose props (documented as D01 above), which is the cleaner boundary. This is a design selection within plan scope, not a deviation.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- ON-key escape hatch is fully wired and on-device verified.
- `key_map.resolve()` is unchanged; the ON key bypass is fully isolated.
- Ready for 67-05 (CLI-GUI parity verification and final wave-3 quality gate).

---
*Phase: 67-reset-escape-hatch*
*Completed: 2026-06-10*

## Self-Check: PASSED

- [x] `0584129` present: `git log --oneline | grep 0584129` → confirmed above
- [x] `aa26232` present: `git log --oneline | grep aa26232` → confirmed above
- [x] `6cdb227` present: `git log --oneline | grep 6cdb227` → confirmed above
- [x] SUMMARY.md written at `.planning/phases/67-reset-escape-hatch/67-04-SUMMARY.md`
- [x] Human-verify checkpoint T-67-10 recorded as PASSED
- [x] No source code modified in this finalization step
