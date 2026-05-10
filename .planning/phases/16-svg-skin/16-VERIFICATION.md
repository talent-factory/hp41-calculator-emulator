---
phase: 16-svg-skin
verified: 2026-05-10T10:00:00Z
status: human_needed
score: 5/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run `just gui-dev` and visually inspect all 5 SCs in the live Tauri window"
    expected: "9x5 key grid visible with double-wide ENTER; dark brown body with gold shift legends; clicking digit keys updates display; key caps scale down on click and bounce back; window is fixed 400x700 with no resize handles"
    why_human: "Visual rendering, animation smoothness, click-to-display feedback, and window non-resizability cannot be verified programmatically without running the Tauri desktop application"
---

# Phase 16: SVG Skin Verification Report

**Phase Goal:** Users see a pixel-perfect HP-41C calculator skin rendered as SVG — dark brown body, gold shift legends, 9x5 key grid with ENTER spanning two columns — where every key is individually clickable and shows a CSS scale-down press animation on each click.
**Verified:** 2026-05-10T10:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SVG skin renders all HP-41C keys in the correct 9x5 grid with ENTER occupying a double-width position | VERIFIED | `Keyboard.tsx`: 44 KEY_DEFS entries across rows 0-4 (9+8+9+9+9 layout); row 1 col 5 has `colSpan: 2` for ENTER; SVG `viewBox="0 0 400 230"` with geometry formula `w = cs * KEY_W + (cs - 1) * GAP` handles double-width correctly |
| 2 | Color scheme matches HP-41C hardware: dark brown body, gold shift legends, white primary labels, light-colored caps for top row | VERIFIED | Body gradient: `#5a3828` → `#1e100a`; f-shift labels: `fill="#d4a800"` (deep gold, intentional upgrade from spec's `#c8a400`); primary labels: `fill={labelColor}` = `#ffffff` for dark keys, `#1e1008` for cream keys; row-0 gradient `#6a4830` → `#1e1008`; ENTER: `#346034` → `#0a180a`; cream mode keys: `#ede8d4` → `#a89870` |
| 3 | Clicking any key invokes `dispatch_op` with the correct key ID | VERIFIED | Data-flow chain confirmed: `key.id` → `handleKeyClick` (guards: `!keyId`, `busyRef.current`) → `onKey(keyId)` prop → `handleClick` in App.tsx → `invoke<CalcStateView>('dispatch_op', { keyId })`; all 23 named IDs validated by `test_all_keyboard_skin_ids_are_valid` Rust test (commit 4c2980f); 10 visual-only keys (id='') blocked at `if (!keyId) return` |
| 4 | Each key click triggers a visible CSS scale-down animation (scale ~0.92) within 150ms without blocking further input | VERIFIED | App.css: `.key { transform-box: fill-box; transform-origin: center; transition: transform 80ms ease-out; }` and `.key-pressed { transform: scale(0.92); }`; Keyboard.tsx: `setPressedKey(keyId)` + `setTimeout(() => setPressedKey(prev => prev === keyId ? null : prev), 150)` functional update form (prevents stale closure); `busyRef.current` debounce in both layers ensures non-blocking |
| 5 | SVG uses a viewBox and scales correctly at 400x700 window size | VERIFIED | `viewBox="0 0 400 230"` + `width="100%"` for responsive scaling; tauri.conf.json: `"width": 400`, `"height": 700`, `"resizable": false`; `.calculator { width: 392px; }` with `margin: 16px auto` fits within 400px Tauri window |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-gui/src/Keyboard.tsx` | SVG component with KEY_DEFS, render loop, press animation | VERIFIED | 235 lines; exports `Keyboard`; 44 KEY_DEFS entries; `colSpan: 2` on ENTER; `pressedKey` state machine; `busyRef.current` guard; `viewBox="0 0 400 230"`; 3D gradients + bevel highlights added in enhancement pass |
| `hp41-gui/src/App.tsx` | handleClick callback + Keyboard import + placeholder replaced | VERIFIED | Line 4: `import { Keyboard } from './Keyboard'`; lines 79-86: `handleClick` useCallback with `invoke('dispatch_op', { keyId })`; line 128: `<Keyboard onKey={handleClick} busyRef={busyRef} />`; old `<div id="keyboard-area" />` removed |
| `hp41-gui/src/App.css` | `.key` and `.key-pressed` CSS rules with transform-box: fill-box | VERIFIED | Lines 74-87: `.key { cursor: pointer; transform-box: fill-box; transform-origin: center; transition: transform 80ms ease-out; }` and `.key-pressed { transform: scale(0.92); }`; old `#keyboard-area` rule removed; width updated to `392px` |
| `hp41-gui/src-tauri/tauri.conf.json` | Window config at 400x700, resizable false | VERIFIED | `"width": 400`, `"height": 700`, `"resizable": false` confirmed at lines 15-17 |
| `hp41-gui/src-tauri/src/key_map.rs` | test_all_keyboard_skin_ids_are_valid test in mod tests block | VERIFIED | Lines 247-266: `test_all_keyboard_skin_ids_are_valid` inside the existing `#[cfg(test)] mod tests` block (line 195); 23 named IDs tested; exactly one `#[cfg(test)]` attribute in the file |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Keyboard.tsx handleKeyClick` | `App.tsx handleClick (onKey prop)` | `onKey(keyId)` prop callback | WIRED | Keyboard.tsx line 101: `onKey(keyId)`; App.tsx line 128: `onKey={handleClick}` |
| `App.tsx handleClick` | Tauri `dispatch_op` command | `invoke<CalcStateView>('dispatch_op', { keyId })` | WIRED | App.tsx lines 82-85: exact invoke call confirmed |
| `KEY_DEFS id values` | `key_map::resolve()` (Rust) | `dispatch_op keyId` parameter | WIRED | All 23 named IDs validated by `test_all_keyboard_skin_ids_are_valid`; digit IDs handled by digit branch in commands.rs; empty-string IDs blocked at `if (!keyId) return` in Keyboard.tsx |
| `.key-pressed CSS class` | SVG `<g>` elements | `className` prop driven by `pressedKey` React state | WIRED | Keyboard.tsx line 165: `className={isPressed ? 'key key-pressed' : 'key'}`; `isPressed = pressedKey === key.id && Boolean(key.id)` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `Keyboard.tsx` | `key.id` (from KEY_DEFS) | Static constant array (intentional — keys are fixed hardware layout) | Yes — static is correct for key definitions | FLOWING |
| `Keyboard.tsx` | `pressedKey` state | `setPressedKey(keyId)` in `handleKeyClick` | Yes — driven by real user click events | FLOWING |
| `App.tsx` | `calcState` (display, stack) | `invoke('dispatch_op', { keyId }).then(view => setCalcState(view))` | Yes — receives real `CalcStateView` from Rust on every key click | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Keyboard.tsx exports Keyboard function | `grep -F "export function Keyboard" Keyboard.tsx` | 1 match at line 93 | PASS |
| ENTER is double-width in KEY_DEFS | `awk '/const KEY_DEFS/,/^];/' Keyboard.tsx \| grep "colSpan: 2"` | 1 match: `row: 1, col: 5, colSpan: 2` | PASS |
| 44 keys in KEY_DEFS | `awk '/const KEY_DEFS/,/^];/' Keyboard.tsx \| grep "{ id:" \| wc -l` | 44 | PASS |
| 10 visual-only keys with empty id | count of `id: ''` in Keyboard.tsx | 10 (XEQ, STO, RCL, f, g, SST, GTO, R/S, ON, BST) | PASS |
| tauri window 400x700 non-resizable | `grep "width\|height\|resizable" tauri.conf.json` | width:400, height:700, resizable:false | PASS |
| CSS scale animation defined | `grep "scale(0.92)\|transform 80ms" App.css` | Both present at lines 82, 78 | PASS |
| Wave 0 Rust test present | `grep "test_all_keyboard_skin_ids_are_valid" key_map.rs` | 1 match at line 248 | PASS |
| All 4 commits verified in git | `git log --oneline 4c2980f 8909d66 a7a0e45 d2aa858` | All 4 commits exist and match descriptions | PASS |
| Live app behavior (dispatch_op, animation, visual) | Requires running `just gui-dev` | Not testable without running Tauri app | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| SKIN-01 | 16-01, 16-02 | User sees pixel-perfect SVG HP-41C key layout (9x5 grid, ENTER double-width, correct labels, HP-41C color scheme) | SATISFIED | 44-key KEY_DEFS with correct 5-row layout, ENTER colSpan:2, dark brown body gradient, gold f-shift labels, white primary labels, cream mode key caps verified in Keyboard.tsx |
| SKIN-02 | 16-01, 16-02 | User can click any key and corresponding HP-41 operation executes in hp41-core | SATISFIED | Full click-to-invoke chain verified; Wave 0 test confirms all 23 named IDs resolve in key_map::resolve(); visual-only keys blocked by empty-id guard |
| SKIN-03 | 16-02 | User sees visual press feedback (CSS scale-down animation) on every key click | SATISFIED | `.key { transform-box: fill-box; transition: transform 80ms ease-out; }` and `.key-pressed { transform: scale(0.92); }` present; pressedKey state machine with 150ms reset wired correctly |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | — | No TODOs, FIXMEs, empty returns, stub placeholders, or hollow implementations detected in any key file | — | — |

**Note on f-shift color:** `#d4a800` (implemented) vs `#c8a400` (originally specified). This is a documented intentional deviation from the 3D visual enhancement pass — both are "gold" and the success criterion says "gold shift legends" not a specific hex value. Not a stub or anti-pattern.

### Human Verification Required

The automated checks are complete and all 5 must-haves are verified at code level. One item requires human testing because it involves live Tauri rendering, CSS animation rendering, and tactile UX behavior that cannot be confirmed without running the application.

#### 1. Full HP-41C GUI Verification (SC-1 through SC-5)

**Test:** Run `just gui-dev` from the repo root and verify each success criterion in the live Tauri window:

- **SC-1 (Key layout):** Count 5 rows, 9 columns. ENTER must be visibly double-wide in row 1 (second row). Confirm all key labels visible: Σ+, 1/x, √x, LOG, LN, XEQ, STO, RCL, ← (row 0); SIN, COS, TAN, R↓, x↔y, ENTER, ÷, × (row 1); USER, f, g, 7, 8, 9, −, PRGM, ALPHA (row 2); CHS, EEX, SST, 4, 5, 6, +, GTO, R/S (row 3); 0, ., ON, 1, 2, 3, LSTx, CLRG, BST (row 4).
- **SC-2 (Colors):** Dark brown body; top-row keys slightly lighter brown; USER/PRGM/ALPHA/f/g caps light cream with dark text; ENTER dark green; other key caps near-black; f-shift labels gold (above Σ+, 1/x, LOG, LN, ASIN, ACOS, ATAN); primary labels white (or dark on cream keys).
- **SC-3 (Click dispatches):** Click digit `7` → display shows `7`. Click `+` with two numbers on stack → adds them. Click `ENTER` → duplicates X. Click `SIN` → computes sin of X. Click `USER` → USER annunciator toggles.
- **SC-4 (Animation):** Click any key → key cap visibly shrinks (scale ~0.92) and bounces back within approximately 150ms. No key remains stuck in pressed state after a single click. Animation does not block further input.
- **SC-5 (Window/HiDPI):** Tauri window fixed at 400x700 — no resize handles. SVG keyboard edges are crisp (no pixelation on Retina/HiDPI). No horizontal scrollbar. SVG fills full calculator width without gaps.

**Expected:** All 5 SCs pass as approved by the human verifier on 2026-05-10 during Task 3 of Plan 16-02.
**Why human:** Visual rendering quality (3D gradients, bevel highlights, shadow depth), CSS animation smoothness and timing feel, click-to-display feedback latency, and window non-resizability all require a running Tauri app to verify. Cannot be confirmed with static code analysis.

**Context:** The SUMMARY records that a human verifier approved SC-1 through SC-5 during Task 3 (commit d2aa858). This automated verification cannot independently confirm that approval, but all code-level evidence supporting those SCs is present and verified above.

### Gaps Summary

No gaps. All 5 must-have truths are verified at code level (existence, substantive implementation, wiring, and data-flow). The only open item is the human visual verification listed above, which reflects the inherent limitation of programmatic verification for GUI rendering — not a code defect.

---

_Verified: 2026-05-10T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
