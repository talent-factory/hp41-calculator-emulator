---
phase: 16
slug: svg-skin
status: issues
reviewed_by: claude-sonnet-4-6
review_date: 2026-05-10
files_reviewed: 5
finding_counts:
  critical: 0
  warning: 1
  info: 2
  total: 3
---

# Phase 16 SVG Skin — Code Review

Files reviewed: `key_map.rs`, `Keyboard.tsx`, `App.tsx`, `App.css`, `vite-env.d.ts`
No Critical findings. One Warning (dead SVG filter). Two Info items.

---

## Critical

None.

---

## Warning

### W-01 — Dead SVG `<filter>` declaration in Keyboard.tsx

**File:** `hp41-gui/src/Keyboard.tsx` lines 143–146
**Confidence:** 95

The 3D enhancement pass added `<filter id="key-shadow">` with a `feDropShadow` primitive
inside `<defs>`, but no element in the render loop references it via `filter="url(#key-shadow)"`.
The implementation uses a manually-drawn offset black `<rect>` for shadows instead.

Impact: Parse overhead today; maintenance trap tomorrow — a future developer who adds
`filter="url(#key-shadow)"` to the 44 key elements would unknowingly trigger expensive
GPU-composited drop shadows on every key. The coexistence of two shadow techniques is a
readability hazard.

**Fix:** Remove the `<filter id="key-shadow">` block from `<defs>`.

---

## Info

### I-01 — Plan acceptance criterion for KEY_DEFS count is wrong (planning artifact only)

**File:** `.planning/phases/16-svg-skin/16-02-PLAN.md`

The plan says `grep -c "row: 0|row: 1|..." returns 40`. Actual count is 44 (9+8+9+9+9).
The 16-02-SUMMARY.md already documents this as a known deviation. No code change needed;
update the acceptance criterion to 44 if ever re-run.

### I-02 — `getKeyGrad` and `isCreamKey` share an identical predicate

**File:** `hp41-gui/src/Keyboard.tsx` lines 74–89

Both functions contain the same two-line cream-key boolean expression. Not a bug today,
but both sites must be kept in sync if the cream key set changes. Optional refactor:
have `getKeyGrad` call `isCreamKey`. Not required before shipping.

---

## Verified Clean

- `key_map.rs` test: all 23 named ids correct; `'e'` (EEX) correctly excluded (handled by digit branch, not resolve())
- `App.tsx` handleClick: empty deps `[]` correct; double busyRef guard valid defense-in-depth
- `App.css` transform-box: `fill-box` correctly scopes scale() to element's own box, not SVG canvas origin
- `vite-env.d.ts`: standard Vite boilerplate, resolves pre-existing TS2882
- SVG geometry: rightmost content at 391px, bottom at 214px — both fit within 400×230 viewBox
