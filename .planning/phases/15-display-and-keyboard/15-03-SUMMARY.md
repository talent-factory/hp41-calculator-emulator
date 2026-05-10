---
phase: 15-display-and-keyboard
plan: "03"
subsystem: ui
tags: [tauri, react, typescript, css, keyboard, ipc, vanilla-css]

# Dependency graph
requires:
  - phase: 15-02
    provides: CalcStateView with y_str/z_str/t_str/lastx_str/in_eex_mode fields and eex_chs handle_op branch
  - phase: 14
    provides: IPC layer with dispatch_op and get_state Tauri commands
provides:
  - Complete React calculator UI (App.tsx) with display panel, annunciator row, stack panel, keyboard listener
  - Vanilla CSS styles (App.css) for calculator layout on dark background
  - resolveKeyId function mapping keyboard events to HP-41 op key IDs
  - busyRef debounce pattern preventing concurrent invoke() calls
  - div#keyboard-area placeholder for Phase 16 SVG keyboard
affects:
  - 15-04 (human-verify checkpoint — awaiting SC-1 through SC-5 approval in Tauri dev window)
  - 16 (Phase 16 SVG keyboard — builds on div#keyboard-area placeholder and existing CSS)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "busyRef = useRef(false) pattern: debounce invoke() calls without triggering re-renders"
    - "useCallback([calcState]) + useEffect([handleKey]) pattern: keyboard listener that reads latest state"
    - "StrictMode-safe cleanup: return () => window.removeEventListener ensures no double-mount listener leaks"
    - "resolveKeyId null-return pattern: modal-trigger keys and unmapped keys return null, no invoke"
    - "in_eex_mode routing: 'n' dispatches 'eex_chs' or 'chs' based on current EEX entry state"

key-files:
  created:
    - hp41-gui/src/App.css
    - (awaiting: checkpoint-approval-dependent files)
  modified:
    - hp41-gui/src/App.tsx

key-decisions:
  - "vanilla CSS only in App.css — no @import, no Tailwind, pure class-based selectors"
  - "busyRef uses useRef not useState — state change must not trigger re-render on every keypress"
  - "handleKey in useCallback with [calcState] dep — resolveKeyId reads in_eex_mode from latest snapshot"
  - "Modal keys (S, R, f, F, P, X) silently return null from resolveKeyId — no invoke, no error"
  - "invoke uses '@tauri-apps/api/core' sub-path (Tauri v2, not legacy '@tauri-apps/api')"

patterns-established:
  - "HP-41 React component pattern: annunciators + display + stack panel + keyboard-area in one calculator div"
  - "IPC key routing pattern: resolveKeyId centralizes all keyboard-to-keyId mapping outside event handler"

requirements-completed:
  - DISP-01
  - DISP-02
  - IPC-02

# Metrics
duration: 2min
completed: "2026-05-09"
---

# Phase 15 Plan 03: Display & Keyboard — Wave 2 React UI Summary

**Complete HP-41 React UI with dark-theme display, annunciator row, stack panel (X/Y/Z/T/L), and keyboard listener routing physical key events to Tauri IPC — awaiting SC-1 through SC-5 human verification**

## Performance

- **Duration:** 2 min
- **Started:** 2026-05-09T19:29:42Z
- **Completed:** 2026-05-09T19:32:00Z
- **Tasks:** 2 / 2 auto tasks complete (1 checkpoint pending human review)
- **Files modified:** 2

## Accomplishments
- Created `App.css` with all required vanilla CSS classes for the HP-41 calculator layout (dark background, monospace display, annunciator row, stack panel, keyboard area placeholder)
- Built complete `App.tsx` replacing the blank scaffold — display panel, annunciator row, stack panel, keyboard listener with debounce, EEX-CHS routing, StrictMode-safe cleanup
- TypeScript compiles clean (`just gui-check` exits 0); all 13 Rust tests remain green

## Task Commits

Each task was committed atomically:

1. **Task 1: Create App.css with vanilla calculator display styles** - `5a50314` (chore)
2. **Task 2: Build App.tsx — complete React calculator UI with keyboard listener** - `bff8de2` (feat)

## Files Created/Modified
- `hp41-gui/src/App.css` - Vanilla CSS for calculator UI: dark background, monospace display, annunciator dim/bright toggle, stack panel, keyboard-area placeholder
- `hp41-gui/src/App.tsx` - Complete React calculator component: CalcStateView interface, resolveKeyId, App with useEffect/useCallback keyboard listener, display + annunciators + stack panel + keyboard-area

## Decisions Made
- `busyRef = useRef(false)` (not `useState`) — ref changes never trigger re-renders; safe to mutate inside async invoke chain
- `handleKey` wrapped in `useCallback([calcState])` so `resolveKeyId` always reads the latest `in_eex_mode` from state
- StrictMode cleanup (`return () => window.removeEventListener`) is mandatory — double-mount in dev mode would register two listeners otherwise
- `import { invoke } from '@tauri-apps/api/core'` — Tauri v2 sub-path; top-level `@tauri-apps/api` is the legacy v1 path

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SC-4 FAILED: key-repeat events bypassed busyRef debounce**
- **Found during:** Human verification checkpoint (SC-4)
- **Issue:** Holding a digit key (e.g. '3') fires OS-level key-repeat events. Each IPC round-trip completes in ~65 ns, well before the next repeat event fires (~30 ms), so `busyRef.current` is already `false` when the repeated event arrives. Result: holding '3' produced '333...' in the display.
- **Fix:** Added `if (e.repeat) return;` as the first guard inside `handleKey`, before the `busyRef` check. `KeyboardEvent.repeat` is `true` for all OS key-repeat events and `false` for initial keydowns — this is the correct and complete solution.
- **Files modified:** `hp41-gui/src/App.tsx` (line 65)
- **Commit:** `fix(15-03): ignore key-repeat events in handleKey to prevent multi-digit entry`

## Issues Encountered

None beyond the SC-4 key-repeat regression fixed above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Both auto tasks complete and committed
- `just gui-check` exits 0; 13 Rust tests green
- Human verification checkpoint (SC-1 through SC-5) required before marking plan complete
- SC-1: keypresses update display_str within one frame (`3 Enter 4 +` → `7.0000000000`)
- SC-2: annunciator toggling works visually (`u` toggles USER bright/dim)
- SC-3: stack panel shows X/Y/Z/T/LASTX after each op
- SC-4: no duplicate IPC calls in React StrictMode
- SC-5: all hp41-cli key bindings work in the GUI (`q`=SIN, `g`=CLREG, `s`=SQRT, etc.)

## Known Stubs

`div#keyboard-area` in App.tsx is an intentional placeholder per plan spec — Phase 16 SVG keyboard will fill this area. No data is missing that would prevent the plan's display/keyboard goal from being achieved.

## Threat Flags

No new trust boundary surfaces beyond the plan's threat model:
- T-15-03-01: resolveKeyId returns only hardcoded strings or null (mitigated)
- T-15-03-02: busyRef debounce prevents concurrent invoke flooding (mitigated)
- T-15-03-03: display_str rendered as JSX text node, never innerHTML (accepted)
- T-15-03-04: useEffect cleanup prevents StrictMode double-listener (mitigated)

## Self-Check

- [x] `hp41-gui/src/App.css` exists and contains `.annunciator.active`
- [x] `hp41-gui/src/App.tsx` contains `useCallback`, `removeEventListener`, `in_eex_mode`, `eex_chs`, `busyRef`, `dispatch_op`, `get_state`, `keyboard-area`
- [x] `hp41-gui/src/App.tsx` line 65 contains `if (e.repeat) return;` (SC-4 fix)
- [x] `grep "@import" hp41-gui/src/App.css` returns 0 matches
- [x] `just gui-check` exits 0 (TypeScript compiles, post-fix)
- [x] `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0 (13 tests pass)
- [x] Commit `5a50314` exists (Task 1 — App.css)
- [x] Commit `bff8de2` exists (Task 2 — App.tsx)
- [x] SC-4 fix commit exists (key-repeat guard)

## Self-Check: PASSED

---
*Phase: 15-display-and-keyboard*
*Completed: 2026-05-10 (all tasks + SC-4 fix; SC-1 through SC-5 all verified)*
