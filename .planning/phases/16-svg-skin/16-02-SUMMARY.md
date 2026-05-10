---
phase: 16-svg-skin
plan: "02"
subsystem: frontend
tags: [react, typescript, svg, tauri, keyboard-skin, animation]

# Dependency graph
requires:
  - phase: 16-01
    provides: Wave 0 gate — all 23 named KEY_DEFS ids validated via key_map::resolve()
  - phase: 15-display-keyboard
    provides: App.tsx with busyRef, CalcStateView, handleKey pattern
  - phase: 14-ipc-layer
    provides: dispatch_op Tauri command accepting keyId string
provides:
  - Keyboard.tsx: complete HP-41C SVG skin with 44-key layout, press animation, dispatch wiring
  - handleClick callback in App.tsx for SVG click-to-dispatch
  - .key/.key-pressed CSS animation rules
  - Tauri window fixed at 400x700
affects: [human-verify-sc1-through-sc5]

# Tech tracking
tech-stack:
  added:
    - SVG inline rendering in React (no external SVG library)
    - CSS transform-box: fill-box for SVG element-local transforms
  patterns:
    - pressedKey state machine with 150ms setTimeout and functional update form (prevents stale closure Pitfall 4)
    - busyRef.current debounce in both Keyboard.tsx (handleKeyClick) and App.tsx (handleClick) — two-layer guard
    - KEY_DEFS as typed array: colSpan field on ENTER enables single geometry formula for all keys
    - Empty id guard: visual-only keys (XEQ, STO, RCL, f, g, SST, GTO, R/S, ON, BST) blocked at handleKeyClick
    - getKeyColor function: row/id-based color dispatch for authentic HP-41C appearance

key-files:
  created:
    - hp41-gui/src/Keyboard.tsx
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.css
    - hp41-gui/src-tauri/tauri.conf.json

key-decisions:
  - "KEY_DEFS has 44 entries (9+8+9+9+9), not 40 — plan text said '40 entries' but the plan's own key list specifies 44; implementation follows the actual list"
  - "handleClick deps array is empty [] — does not read calcState (no EEX-CHS context needed for mouse clicks)"
  - "transform-box: fill-box is CSS-only — the TSX has zero transform-box references (acceptance criteria confirmed)"
  - "Keyboard.tsx uses MutableRefObject<boolean> import (not React.MutableRefObject) to avoid default React import"

requirements-completed:
  - SKIN-01
  - SKIN-02
  - SKIN-03

# Metrics
duration: ~4min (tasks 1-2 only; task 3 human-verify pending)
completed: CHECKPOINT — awaiting human verification
---

# Phase 16 Plan 02: SVG Keyboard Skin Summary

**Complete HP-41C SVG keyboard component with 44 keys, press animation, and click-to-dispatch — pending human visual verification (SC-1 through SC-5)**

## Status

CHECKPOINT REACHED at Task 3 (human-verify). Tasks 1 and 2 are complete and committed. This SUMMARY will be finalized after human verification approval.

## Performance

- **Duration:** ~4 min (tasks 1-2)
- **Started:** 2026-05-10T08:46:01Z
- **Checkpoint reached:** 2026-05-10T08:50:09Z
- **Tasks complete:** 2 of 3 (task 3 awaits human verification)
- **Files modified/created:** 4

## Accomplishments

### Task 1: Keyboard.tsx (commit 8909d66)
- Created `hp41-gui/src/Keyboard.tsx` with 44-key HP-41C layout
- KEY_DEFS array: 5 rows (9+8+9+9+9 keys), ENTER with `colSpan: 2` in row 1
- SVG render loop: `viewBox="0 0 400 230"`, PAD=8, KEY_W=39, KEY_H=26, GAP=4, FSHIFT_H=12
- Color scheme: dark brown body (#3d2b1f), gold f-shift labels (#c8a400), cream mode keys (#d4c9b0), dark green ENTER (#1a3a1a), near-black digits (#1a1a1a)
- pressedKey state machine: `setPressedKey(prev => ...)` functional update, 150ms timeout
- Security: `if (!keyId) return` blocks 10 visual-only keys; `if (busyRef.current) return` debounces clicks

### Task 2: App.tsx + App.css + tauri.conf.json (commit a7a0e45)
- App.tsx: added `import { Keyboard }`, `handleClick` useCallback, replaced `<div id="keyboard-area" />` with `<Keyboard onKey={handleClick} busyRef={busyRef} />`
- App.css: `.calculator` width 320px → 392px; `#keyboard-area` placeholder replaced with `.key`/`.key-pressed` animation CSS (`transform-box: fill-box`, `scale(0.92)`)
- tauri.conf.json: window 800×600 resizable=true → 400×700 resizable=false
- Rust tests: 14 passed, 0 failed after all changes

## Task Commits

1. **Task 1: Create Keyboard.tsx** — `8909d66`
2. **Task 2: Wire Keyboard into App, update CSS and window config** — `a7a0e45`

## Files Created/Modified

- `hp41-gui/src/Keyboard.tsx` — new file, 163 lines
- `hp41-gui/src/App.tsx` — Keyboard import + handleClick + JSX replacement (+11 lines)
- `hp41-gui/src/App.css` — width update + animation CSS (+14 lines, -4 lines)
- `hp41-gui/src-tauri/tauri.conf.json` — window config (3 values changed)

## Decisions Made

- KEY_DEFS has 44 entries matching the plan's actual key list (plan header said "40" in error)
- handleClick uses empty deps `[]` — no calcState dependency needed for mouse-click dispatch
- transform-box lives entirely in CSS; TSX has zero transform-box references
- MutableRefObject<boolean> import style avoids default React import while maintaining TypeScript type safety

## Deviations from Plan

**1. [Rule 1 - Minor] KEY_DEFS entry count is 44, not 40**
- **Found during:** Task 1 implementation
- **Issue:** Plan header states "40 entries, 5 rows x 9 columns" but the plan's own KEY_DEFS specification lists 9+8+9+9+9 = 44 entries. The HP-41C actually has more than 40 keys.
- **Fix:** Implemented all 44 entries exactly as specified in the plan's KEY_DEFS list. The plan's acceptance check `grep -c "row: 0\|row: 1..." returns 40` is incorrect for the same reason.
- **Files modified:** Keyboard.tsx (44 entries, as specified)
- **Commit:** 8909d66

## Known Stubs

None. All 44 keys render with correct labels; functional keys dispatch to Rust via invoke(); visual-only keys (id='') are silently blocked.

## Threat Flags

No new security surface introduced beyond the plan's threat model.

## Self-Check: CHECKPOINT

Tasks 1 and 2 verified. Task 3 (human-verify) has not been executed — this is expected; awaiting orchestrator to present verification steps to the user.

Files exist:
- hp41-gui/src/Keyboard.tsx: EXISTS
- hp41-gui/src/App.tsx: EXISTS (modified)
- hp41-gui/src/App.css: EXISTS (modified)
- hp41-gui/src-tauri/tauri.conf.json: EXISTS (modified)

Commits exist:
- 8909d66: feat(16-02): create Keyboard.tsx with 44-key HP-41C SVG skin
- a7a0e45: feat(16-02): wire Keyboard into App, update CSS and window config
