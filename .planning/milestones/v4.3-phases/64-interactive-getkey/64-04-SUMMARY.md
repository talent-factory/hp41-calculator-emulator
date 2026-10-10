---
phase: 64-interactive-getkey
plan: 04
subsystem: gui-ts
tags: [typescript, react, vitest, getkey, wait-for-key, d-11, prgm-03, gui]

# Dependency graph
requires:
  - phase: 64-interactive-getkey
    plan: 03
    provides: resume_program_with_key Tauri command + WaitForKey projected as "wait_for_key" in CalcStateView

provides:
  - Yield-driver useEffect guard: wait_for_key skips setTimeout (D-11 honored)
  - invokeForKey WaitForKey guard: on-screen taps route to resume_program_with_key with HP-41 keyCode
  - handleKey WaitForKey guard: physical keyboard routes to resume_program_with_key with HP-41 keyCode
  - Esc/cancel path: detects wait_for_key via pending_yield.kind (not is_running) and sends sentinel 0
  - Group Q Vitest tests: PRGM-03-k (yield-driver guard) and PRGM-03-l (key-event routing)

affects:
  - GUI GETKEY end-to-end: App.tsx now fully wires the WaitForKey suspend/resume lifecycle

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "yield-driver guard: if (kind === 'wait_for_key') return before resumeScheduledRef — prevents timer-driven auto-resume of event-driven yields (Pitfall 2)"
    - "invokeForKey third param KeyDef?: passes KeyDef from handleClick so keyCode is readable without a second KEY_DEFS lookup"
    - "WaitForKey routing: key?.keyCode -> resume_program_with_key({keycode}); undefined keyCode -> Promise.resolve(state) no-op"
    - "handleKey physical guard: KEY_DEFS.find(k => k.id === keyId)?.keyCode — same pattern as shiftActive block above"
    - "cancel/Esc WaitForKey branch: BEFORE is_running branch (is_running is false during suspend)"

key-files:
  created: []
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx

key-decisions:
  - "64-04-D01: invokeForKey gains optional KeyDef third param — passes the full KeyDef from handleClick so keyCode is available without a secondary KEY_DEFS lookup inside invokeForKey (avoids a key=undefined path when invoked from applyModalResult)"
  - "64-04-D02: Esc cancel branch BEFORE is_running branch — during WaitForKey, is_running is false (program is suspended, not running); the guard must detect pending_yield.kind === 'wait_for_key' before the is_running branch would be reached"
  - "64-04-D03: physical keyboard guard returns after no-keyCode no-op — prevents fall-through to dispatchKeyId for keys like xge_y and CHS during WaitForKey (hardware faithful: HP-41 only captures calculator keys)"
  - "64-04-D04: invokeForKey no-keyCode path returns Promise.resolve(state) — callers always chain .then/.catch; a void return would break the promise chain and leave busyRef stuck"

requirements-completed: [PRGM-03]

# Metrics
duration: 8min
completed: 2026-06-07
---

# Phase 64 Plan 04: GUI TypeScript GETKEY Frontend Summary

**GUI GETKEY frontend fully wired: yield-driver skips timer for WaitForKey; on-screen taps + physical keys route to resume_program_with_key; Esc/cancel sends sentinel 0; Group Q Vitest tests green**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-06-07T08:40:00Z
- **Completed:** 2026-06-07T08:47:54Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

### Task 1: Yield-driver guard + key-event resume routing + cancel path (`App.tsx`)

- Added `if (calcState.pending_yield.kind === 'wait_for_key') return;` in the yield-driver `useEffect` immediately after the null guard — prevents `setTimeout(resume_ms)` from firing for event-driven yields (D-11 honored, Pitfall 2 prevented)
- Added WaitForKey guard at the top of `invokeForKey` — routes on-screen key taps to `invoke('resume_program_with_key', { keycode: hp41Code })` when `pending_yield.kind === 'wait_for_key'`; keys without `keyCode` (CHS, xge_y, etc.) return `Promise.resolve(state)` as a no-op
- Extended `invokeForKey` signature with optional `key?: KeyDef` third parameter — passed from `handleClick` so the HP-41 `keyCode` is available without a secondary `KEY_DEFS` lookup
- Added WaitForKey guard in `handleKey` (physical keyboard) — resolves `keyCode` via `KEY_DEFS.find(k => k.id === keyId)?.keyCode`; keys with `keyCode` invoke `resume_program_with_key`; no-keyCode keys return without dispatching (hardware faithful)
- Added WaitForKey cancel branch in `handleKey` Esc precedence block — positioned BEFORE the `is_running` branch (is_running is false during WaitForKey suspend); invokes `resume_program_with_key({ keycode: 0 })` (sentinel; D-02 cancel path)

### Task 2: Group Q Vitest tests (`App.test.tsx`)

- **PRGM-03-k** (`getkey_yield_driver_skips_wait_for_key`): fake timers with `shouldAdvanceTime:true`; triggers a `wait_for_key` pending_yield; advances 5 seconds; asserts `resume_program` was NOT invoked (D-11 guard confirmed)
- **PRGM-03-l** (on-screen tap routing): transitions to `wait_for_key` state; clicks `sigma_plus` (keyCode=11); asserts `resume_program_with_key({ keycode: 11 })` invoked and `dispatch_op` NOT called; transitions back to `wait_for_key`; clicks `xge_y` (no keyCode); asserts no Tauri call was made (no-op confirmed)
- Total: **343 Vitest tests green** (341 existing + 2 new Group Q)

## Task Commits

1. **Task 1: Yield-driver guard + key-event resume + cancel path** — `5ec9490` (feat)
2. **Task 2: Group Q Vitest tests PRGM-03-k and PRGM-03-l** — `6b419c8` (test)

## Files Created/Modified

- `hp41-gui/src/App.tsx` — yield-driver `useEffect` guard; `invokeForKey` WaitForKey guard + KeyDef optional param; `handleKey` WaitForKey guard + Esc cancel branch
- `hp41-gui/src/App.test.tsx` — Group Q describe block with PRGM-03-k and PRGM-03-l tests

## Decisions Made

- **64-04-D01 (invokeForKey KeyDef param):** `invokeForKey` gains an optional `key?: KeyDef` third parameter passed from `handleClick`. This avoids a secondary `KEY_DEFS.find` lookup inside `invokeForKey` (which would require knowing the original `effectiveId`), and makes the `keyCode` directly available at the call site.
- **64-04-D02 (Esc branch order):** The WaitForKey cancel branch in `handleKey` is placed BEFORE the `is_running` branch — during WaitForKey, `is_running` is `false` (the program is suspended, not running), so the `is_running` branch would never fire. The new branch detects `pending_yield.kind === 'wait_for_key'` directly.
- **64-04-D03 (physical keyboard no-keyCode return):** The `handleKey` WaitForKey guard always `return`s after processing (whether a keyCode was found or not) — prevents fall-through to `dispatchKeyId` for keys like `xge_y` and CHS during a WaitForKey suspend (hardware faithful).
- **64-04-D04 (invokeForKey no-keyCode promise):** When `pending_yield.kind === 'wait_for_key'` and the key has no `keyCode`, `invokeForKey` returns `Promise.resolve(state as CalcStateView)` — callers always chain `.then`/`.catch` and must not receive `undefined`. This keeps `busyRef` clean and avoids a broken promise chain.

## Deviations from Plan

None — plan executed exactly as written. The `invokeForKey` signature extension (D-01) is a direct implementation of the PATTERNS.md guidance. All four guards documented in the plan are implemented and verified.

## Known Stubs

None. All guards route to live Tauri commands (`resume_program_with_key` from Plan 03). No placeholder values.

## Threat Flags

No new threat surface. T-64-TS-01 (keyCode passed from static `KEY_DEFS` table, typed as `u8` Rust-side) and T-64-TS-02 (yield-driver duplicate scheduling prevented by early return) are both implemented as planned.

## Self-Check

- [x] `hp41-gui/src/App.tsx` — `wait_for_key` guard in yield-driver `useEffect` present
- [x] `hp41-gui/src/App.tsx` — `invokeForKey` WaitForKey guard at top of body present
- [x] `hp41-gui/src/App.tsx` — `handleClick` passes `key` to `invokeForKey`
- [x] `hp41-gui/src/App.tsx` — `handleKey` WaitForKey guard (physical keyboard) present
- [x] `hp41-gui/src/App.tsx` — Esc cancel branch for WaitForKey present (before is_running)
- [x] `hp41-gui/src/App.test.tsx` — Group Q tests PRGM-03-k and PRGM-03-l present
- [x] `just gui-ci` green — 343 Vitest tests pass; 130 GUI Rust tests pass
- [x] Commits: `5ec9490`, `6b419c8` present in git log

## Self-Check: PASSED

---
*Phase: 64-interactive-getkey*
*Completed: 2026-06-07*
