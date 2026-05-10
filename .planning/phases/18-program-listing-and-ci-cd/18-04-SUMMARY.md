---
phase: 18-program-listing-and-ci-cd
plan: "04"
subsystem: hp41-gui/src
tags: [wave-2, react, frontend, program-listing, sst, bst, typescript]
dependency_graph:
  requires: [18-02]
  provides: [prgm-panel-ui, sst-bst-frontend-routing, keyboard-sst-bst-ids]
  affects:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/Keyboard.tsx
    - hp41-gui/src/App.css
tech_stack:
  added: []
  patterns:
    - conditional JSX for auto-show/hide panel (annunciators.prgm gate)
    - handleClick routing pattern (sst/bst branch before dispatch_op fallthrough)
    - activeStepRef + useEffect auto-scroll (scrollIntoView on pc change)
    - handleKey-delegates-to-handleClick (avoids duplicate IPC path for SST/BST)
key_files:
  created: []
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/Keyboard.tsx
    - hp41-gui/src/App.css
decisions:
  - "handleClick moved before handleKey to satisfy React dependency ordering (handleKey depends on handleClick)"
  - "calcState.annunciators.prgm check is safe after early-return null guard at line 127 — no double null-check needed"
  - "npm install run in worktree hp41-gui since node_modules not present in fresh worktree"
metrics:
  duration: 480s
  completed: "2026-05-10T16:20:00Z"
  tasks_completed: 2
  files_modified: 3
---

# Phase 18 Plan 04: Wave 2 React Frontend — Program Listing Panel Summary

React frontend implementation for the PRGM-mode program listing panel: conditional JSX panel below the keyboard with SST/BST routing via dedicated Tauri commands, F7/F8 keyboard bindings, auto-scroll to the active step, and dark HP-41 green LCD highlight aesthetic.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend App.tsx with program listing panel JSX, SST/BST routing, and resolveKeyId F7/F8 | e0da96f | hp41-gui/src/App.tsx |
| 2 | Update Keyboard.tsx KEY_DEFS and add App.css program panel styles | 70ea6d9 | hp41-gui/src/Keyboard.tsx, hp41-gui/src/App.css |

## What Was Done

**Task 1 — App.tsx:**
- Added `program_steps: string[]` and `pc: number` to `CalcStateView` interface (Phase 18 D-01)
- Added `activeStepRef = useRef<HTMLDivElement>(null)` alongside existing refs
- Added auto-scroll `useEffect` watching `calcState?.pc` — fires `scrollIntoView({ behavior: 'smooth', block: 'nearest' })`
- Added F7/F8 handling in `resolveKeyId()` before the MAP lookup (D-07)
- Rewrote `handleClick` to branch on `keyId === 'sst'`/`'bst'` and call `invoke('sst_step')`/`invoke('bst_step')` (D-04/D-06)
- Rewrote `handleKey` to delegate to `handleClick(keyId)` instead of calling `invoke('dispatch_op', ...)` directly — prevents SST/BST hitting the wrong Tauri command
- Note: `handleClick` moved before `handleKey` in source order to satisfy React's `useCallback` dependency chain
- Added program listing panel JSX conditional on `calcState.annunciators.prgm` (D-08)
  - Header: "PRGM — N step/steps" with step count
  - Content: scrollable div with one `.step-row` per step
  - Active step receives `ref={activeStepRef}` and `.step-active` class when `calcState.pc === i`

**Task 2 — Keyboard.tsx:**
- Changed SST entry from `id: ''` to `id: 'sst'` (row 3, col 2) — enables click routing
- Changed BST entry from `id: ''` to `id: 'bst'` (row 4, col 8) — enables click routing
- `handleKeyClick` guard `if (!keyId) return;` continues to work correctly

**Task 2 — App.css:**
- Appended `.prgm-panel`, `.prgm-panel-header`, `.prgm-panel-content`, `.step-row`, `.step-active` styles
- All color values sourced from UI-SPEC (approved 2026-05-10):
  - Panel bg: #1a1a1a (matches .print-panel)
  - Header bg: #252525, text #888 (matches .print-panel-header)
  - Step text: #c8c8c8 (matches .print-line)
  - Active step bg: #1e3a1e (HP-41 green LCD aesthetic)
  - Active step text: #c8e6c9 (matches .display — green LCD tone)
- Panel content max-height: 160px with overflow-y: auto (D-09)

## Verification

```
npx tsc --noEmit (from hp41-gui/)
Result: EXIT 0 — TypeScript clean

cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
Result: 27 passed (3 suites, 0.01s) — EXIT 0
```

Structural checks:
- `grep -c 'program_steps: string\[\]' App.tsx` → 1
- `grep -c "pc: number" App.tsx` → 1
- `grep -c "activeStepRef" App.tsx` → 3 (declaration + useEffect + JSX ref)
- `grep -c "sst_step" App.tsx` → 1
- `grep -c "bst_step" App.tsx` → 1
- `grep -c "F7" App.tsx` → 2 (comment + code)
- `grep -c "F8" App.tsx` → 2 (comment + code)
- `grep -c "prgm-panel" App.tsx` → 3
- `grep -c "handleClick(keyId)" App.tsx` → 1
- `grep -c "id: 'sst'" Keyboard.tsx` → 1
- `grep -c "id: 'bst'" Keyboard.tsx` → 1
- `grep -c ".step-active" App.css` → 1
- `grep -c "#1e3a1e" App.css` → 1
- `grep -c "max-height: 160px" App.css` → 1

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] handleClick must be declared before handleKey**
- **Found during:** Task 1 implementation
- **Issue:** The plan's code snippets showed `handleKey` before `handleClick`, but `handleKey`'s `useCallback` must list `handleClick` in its dependency array. React requires declared hooks to precede their dependents — if `handleClick` appears after `handleKey` in source, TypeScript/ESLint would see a temporal use-before-declaration violation.
- **Fix:** Moved `handleClick` declaration before `handleKey` in source order. `handleKey` now correctly declares `[calcState, handleClick]` as its dependencies.
- **Files modified:** hp41-gui/src/App.tsx
- **Commit:** e0da96f

**2. [Rule 3 - Blocking] npm install required in worktree**
- **Found during:** Task 1 verification
- **Issue:** The worktree (`agent-a39c93d3d58d27d08`) did not have `node_modules` installed — fresh worktree checkout. `npx tsc --noEmit` returned "This is not the tsc command you are looking for."
- **Fix:** Ran `npm install --prefer-offline` in `hp41-gui/` within the worktree. Takes ~5 seconds, uses package-lock.json.
- **Impact:** Blocking for TypeScript verification; non-blocking for runtime. CI matrix handles this via `npm install` step.
- **Commit:** Not a code change — environment setup only.

## Known Stubs

None — all implemented functionality is complete. The panel renders live data from `CalcStateView.program_steps` and `CalcStateView.pc`, both of which are populated by the Rust backend in Plan 02.

## Threat Surface Scan

No new network endpoints or auth paths introduced. All changes are React UI components rendering data from the existing `CalcStateView` IPC channel (already secured via Tauri capabilities in Plan 02).

## Self-Check: PASSED

- [x] hp41-gui/src/App.tsx modified: program_steps + pc in interface, activeStepRef, F7/F8, SST/BST routing, handleKey delegation, prgm-panel JSX
- [x] hp41-gui/src/Keyboard.tsx modified: id: 'sst' and id: 'bst' in KEY_DEFS
- [x] hp41-gui/src/App.css modified: .prgm-panel, .prgm-panel-header, .prgm-panel-content, .step-row, .step-active styles present
- [x] Commit e0da96f exists (Task 1)
- [x] Commit 70ea6d9 exists (Task 2)
- [x] npx tsc --noEmit exits 0 — TypeScript clean
- [x] cargo test exits 0 — 27 Rust tests pass
- [x] Checkpoint task reached — human verification required for SC-1, SC-2, SC-3
