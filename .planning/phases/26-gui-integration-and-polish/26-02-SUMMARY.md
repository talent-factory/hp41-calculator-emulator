---
phase: 26-gui-integration-and-polish
plan: 02
subsystem: hp41-gui (React frontend SVG component)
tags: [14-segment-lcd, svg-display, visual-fidelity, D-26.6, D-26.7, FN-POLISH-01]
dependency_graph:
  requires:
    - phase-26 plan-01 modal infrastructure (renderModalLcd LCD preview emitter
      + displayText derivation in App.tsx)
  provides:
    - Display14Seg SVG component with 14-segment + decimal-dot per-cell rendering
    - SEGMENT_MAP / SEGMENT_PATHS / SEGMENT_ORDER / DECIMAL_DOT_PATH exports
    - `.display svg` CSS sizing rule (W6 — no longer optional)
    - Vitest + @testing-library/react + jsdom test infrastructure for future
      React render tests
  affects:
    - Plan 26-03 (polish bundle): LCD aesthetic now in place; the `?` overlay
      and USER-mode relabel land on top of this visual baseline
tech_stack:
  added:
    - "@testing-library/react ^17 (devDependency — React render tests)"
    - "jsdom ^28 (devDependency — DOM environment for vitest)"
  patterns:
    - "Drop-in SVG component preserving existing CSS (D-26.7 mirrors v2.0 Phase 18 program-listing panel)"
    - "Per-cell `<g transform=translate(…)>` SVG-grid construction (mirrors Keyboard.tsx idiom)"
    - "Constant-shape + render-output assertions instead of snapshot files (W5 tightening)"
    - "Opacity-based on/off rendering for always-present segment grid (D-26.6 aesthetic)"
key_files:
  created:
    - hp41-gui/src/Display14Seg.tsx (242 lines, 49-glyph SEGMENT_MAP)
    - hp41-gui/src/Display14Seg.test.tsx (22 tests covering constants + rendering + W4/W5/W6)
  modified:
    - hp41-gui/src/App.tsx (+1 import line, 1 inner content swap inside `.display`)
    - hp41-gui/src/App.css (+13 lines for `.display svg` rule; `.display` rule byte-identical)
    - hp41-gui/vite.config.ts (+vitest config block — environment: 'jsdom')
    - hp41-gui/package.json (+@testing-library/react, +jsdom devDeps)
decisions:
  - D-26.6 (full segment grid with dim 'off' segments at opacity=0.1)
  - D-26.7 (drop-in component preserves byte-identical `.display` CSS)
  - W4 (SEGMENT_ORDER pinned to Wikipedia canonical 14-name labels; DECIMAL_DOT_PATH
    rendered as 15th conditional path that overlays the previous cell — period
    does not consume a slot)
  - W5 (TIGHTENED — off-segment opacity assertion scoped PER CELL, not container-global)
  - W6 (`.display svg { display: block; width: 100%; height: 100% }` rule committed
    as non-optional; jsdom-friendly viewBox + preserveAspectRatio attributes
    assert the SVG declares its sizing contract)
  - Color palette: #c8e6c9 (matches existing .display color in App.css line 47);
    the brighter #a0ffa0 from CONTEXT D-26.6 was considered and rejected for
    visual continuity with v2.0
metrics:
  duration: ~7 min (3 atomic commits)
  completed_date: 2026-05-15
  commits: 3
  tests_added:
    typescript: 22 (Vitest — 8 constant-shape + 14 render-output incl. W4/W5/W6)
  budget_actual:
    paths_per_render: 180 (12 cells × 15 paths = 14 segments + 1 decimal dot)
    segment_map_entries: 49 (10 digits + 26 uppercase letters + 13 punctuation)
---

# Phase 26 Plan 02: 14-Segment LCD Summary

**One-liner:** Replaces the CSS-text HP-41C display with `Display14Seg`, a
14-segment SVG LCD component drop-in inside the existing `.display` div per
D-26.7; off-segments render at opacity=0.1 for the authentic HP-41C "dim
grid behind lit text" aesthetic (D-26.6); modal previews from Plan 26-01's
`renderModalLcd` flow through the new `text` prop unchanged.

## Outcome

Phase 26 Plan 02 delivers the visual-fidelity polish (FN-POLISH-01) that
takes the GUI from "calculator that works correctly" to "calculator that
looks like a real HP-41C". The CSS-text display rendering used since v2.0
is replaced by a pure-render SVG component; every character cell renders
all 14 segments unconditionally, with lit/unlit toggled via opacity rather
than visibility. The result is the "always-on dim segment grid behind any
displayed text" that defines the real HP-41C LCD's look.

Plans 26-01's modal preview infrastructure flows through unchanged:
`renderModalLcd(pendingInput)` produces strings like `STO _5`, `SF IND _5`,
`CLP MYPRG_`, and the new component renders them as 14-segment glyphs with
the underscore lighting only the bottom 'd' segment per D-26.3.

## W4-Pinned 14-Segment Convention

The component uses the Wikipedia "Fourteen-segment display" canonical
labeling:

| Index | Name | Position |
|-------|------|----------|
| 0 | a | top horizontal |
| 1 | b | top-right vertical |
| 2 | c | bottom-right vertical |
| 3 | d | bottom horizontal |
| 4 | e | bottom-left vertical |
| 5 | f | top-left vertical |
| 6 | g1 | middle horizontal, left half |
| 7 | g2 | middle horizontal, right half |
| 8 | h | NW diagonal |
| 9 | i | top vertical center |
| 10 | j | NE diagonal |
| 11 | k | SE diagonal |
| 12 | l | bottom vertical center |
| 13 | m | SW diagonal |

The decimal point is rendered as a 15th conditional `DECIMAL_DOT_PATH`
per cell. Per HP-41 LCD hardware behavior, a `.` in the input string
overlays the **previous** cell — it does NOT consume a cell slot. The
component implements this via a "next-char-is-period" rule when slicing
the input into the 12-cell layout:

```typescript
for (let i = 0; i < text.length && cells.length < 12; i++) {
    const ch = text[i];
    if (ch === '.' && cells.length > 0) {
        cells[cells.length - 1].hasDecimal = true;
    } else {
        cells.push({ char: ch, hasDecimal: false });
    }
}
```

A leading period (no prior cell) becomes its own empty cell with the
decimal dot off — locked by an explicit Vitest edge-case test.

## Glyph Source

Canonical character-to-segment mappings follow the public-domain
Wikipedia "Fourteen-segment display" article's character table, restricted
to the HP-41 character set per D-26.6:

- **10 digits** (`0`-`9`)
- **26 uppercase letters** (`A`-`Z`) — HP-41 ALPHA mode is uppercase-only;
  lowercase characters fall through to the `toUpperCase()` cell lookup
- **13 punctuation entries** (`.`, `,`, `-`, `+`, `(`, `)`, `=`, `/`, `:`,
  ` ` space, `_` underscore, `?`, `*`)

**Total: 49 glyph entries.** Underscore lights only segment `d` (index 3)
matching the D-26.3 modal-cursor convention from Plan 26-01's
`renderModalLcd` ("`STO __` → `STO _5`"). Period and comma map to empty
arrays (`[]`) because they fold into the previous cell.

### Intentionally Omitted

Per D-26.6 explicit scope boundary:
- **Σ** (summation glyph)
- **π** (pi glyph as character — `PI` resolves to the constant push op)
- **μ-superscript** (micro prefix)
- **Other HP-41 special charset** (degree, prime, …)

All deferred to v3.x ALPHA-special-charset expansion (per Phase 25 D-25.5
deferral). Phase 26 covers the standard A-Z / 0-9 / common-punctuation set.

## Color Palette

**LIT** segments: `#c8e6c9` (matching the existing `.display` color in
App.css line 47). **OFF** segments: same `#c8e6c9`, but opacity=0.1
(faintly visible — the authentic HP-41C LCD aesthetic per D-26.6).

The brighter `#a0ffa0` candidate from CONTEXT D-26.6 was considered and
rejected for visual continuity with v2.0. Should a future phase want a
more LCD-green palette, the constants `LIT_COLOR`/`ON_OPACITY`/
`OFF_OPACITY` are module-level in Display14Seg.tsx (single-source change).

## W6 — Explicit `.display svg` CSS Rule (Non-Optional)

Committed in App.css immediately after the existing `.display` rule:

```css
.display svg {
  display: block;
  width: 100%;
  height: 100%;
}
```

Without `display: block` the SVG's default inline-flow leaves a small
vertical baseline gap; `width:100%`/`height:100%` lets the SVG fill the
container while its own `preserveAspectRatio="xMidYMid meet"` maintains
aspect ratio within the `.display` `min-height: 2em` constraint. The
existing `.display` rule (background, padding, font, color, min-height,
border, white-space) is preserved BYTE-IDENTICAL per D-26.7:

```
$ git show ab392c9:hp41-gui/src/App.css | sed -n '/^.display {/,/^}/p' \
  | diff - <(sed -n '/^.display {/,/^}/p' hp41-gui/src/App.css | head -13)
# (no output — byte-identical)
```

## Test Coverage

**22 new Vitest tests** in `Display14Seg.test.tsx`:

### Constant-shape tests (8)
- SEGMENT_PATHS.length === 14
- SEGMENT_ORDER is `['a','b','c','d','e','f','g1','g2','h','i','j','k','l','m']`
- SEGMENT_ORDER.length === SEGMENT_PATHS.length (no drift)
- SEGMENT_MAP covers all digits 0-9
- SEGMENT_MAP covers all A-Z
- SEGMENT_MAP['_'] is defined AND contains index 3 (bottom segment — D-26.3 cursor)
- SEGMENT_MAP[' '] === [] (space — all off)
- SEGMENT_MAP['-'] === [6, 7] (g1+g2 middle bar)

### Render-output tests (14)
- '2.0000' renders exactly 12 cells (period folds into cell 0)
- 12 × 15 = 180 path elements per render
- 'STO _5', 'SF IND _5', 'CLP MYPRG_' modal previews render without crash
- Empty input → 12 empty cells (space-padded)
- 18-char input → truncated to 12 cells
- **W5 (TIGHTENED)**: PER-CELL off-segment opacity scoping — iterates
  the first cell's 15 paths individually; container-global paths[0]
  would yield false positives across cells
- '8' digit → > 5 lit segments (opacity >= 0.99)
- **W4 decimal-overlay**: '2.5' renders 12 cells; cell 0's decimal dot
  lit (opacity >= 0.99), cell 1's decimal dot off (opacity < 0.5)
- **W4 leading-period edge case**: '.5' produces cell 0 with empty
  segments and decimal-dot off (no prior cell to attach to)
- **W6 SVG layout contract**: viewBox matches `^0 0 \d+ \d+$`,
  width >= 240, preserveAspectRatio === 'xMidYMid meet'
- aria-label === 'HP-41 14-segment display' for accessibility
- Lowercase 'a' renders identically to 'A' (toUpperCase() in lookup)

**Full Vitest suite status:**
```
$ npm run test
Test Files  2 passed (2)
     Tests  64 passed (64)
```

42 Wave 1 (`pending_input.test.ts`) + 22 new = 64 / 64 green.

## Manual Visual Sanity (Post-Merge)

The user should run `just gui-dev` and visually confirm:
- The 12-character display renders with 14-segment glyphs
- Dim 'off' segments are faintly visible behind any lit text
- Decimal points appear at the bottom-right of the preceding digit cell
- The display aesthetic resembles a real HP-41C LCD
- Modal-preview transitions (open STO, type 0, type 5 → "STO 05" →
  dispatch) render cleanly through the SVG

If the dim 'off' segments are TOO dim or TOO bright on the user's
display, the single tunable is `OFF_OPACITY` (currently 0.1) at the top
of Display14Seg.tsx.

## SC-4 Invariant Verification

```
$ grep -rEn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/
# (no output — zero matches)
```

Phase 26 Plan 02 changes are frontend-only (`hp41-gui/src/`); no
calculator/math logic added to `hp41-gui/src-tauri/`.

## Save-File Backward Compat

No `CalcState` field changes. v1.0–v2.1 save files continue to load
without migration. The Display14Seg component receives its `text` prop
directly from `displayText` in App.tsx, which derives from either
`renderModalLcd(pendingInput)` (frontend-owned modal state) or
`calcState.display_str` (existing CalcStateView field unchanged).

## All Gates Green

| Gate | Status | Detail |
|------|--------|--------|
| `npx tsc --noEmit` | OK | clean |
| `npm run test` (Vitest) | OK | 64 / 64 passed (42 Wave 1 + 22 new) |
| `just gui-check` | OK | cargo check clean |
| `just gui-ci` | OK | tsc + cargo test (58 + 0 + 3 + 0) + cargo build --release |
| SC-4 invariant grep | OK | zero matches |
| `.display` CSS byte-identical | OK | diff against ab392c9 shows no changes |

## Deviations from Plan

### Auto-fixed Issues

None substantive. Two minor adjustments documented inline:

1. **Rule 3 — JSX namespace fix for React 19**: the initial draft of
   `Display14Seg.tsx` declared the return type as `: JSX.Element` per
   the plan's interface contract. React 19's TypeScript types removed
   the global `JSX` namespace import; tsc reported `TS2503 Cannot find
   namespace 'JSX'`. Fix: dropped the explicit `: JSX.Element` return
   type annotation (TypeScript infers it correctly from JSX returns).
   No behavioral change. Caught and fixed before the Task 1 commit.

2. **Rule 3 — `@testing-library/react` + `jsdom` not installed**: the
   plan's Task 3 step (d) explicitly anticipated this — Wave 1
   (Plan 26-01) installed vitest but the React render tests in this
   plan need an in-process DOM. Added both as devDependencies (53
   transitive packages) and added `test.environment: 'jsdom'` to
   `vite.config.ts`. The existing Wave 1 `pending_input.test.ts` is
   pure-function and runs equally well under jsdom — no behavior
   change there. Verified by running the full suite (64 / 64 green).

### Authentication Gates

None encountered.

## Cross-References

- **Plan 26-01** (modal architecture & key wiring) — provides the
  `renderModalLcd` LCD-preview emitter + `displayText` derivation that
  feeds Display14Seg's `text` prop. The wave dependency chain is
  satisfied: 26-01 ships modal state → 26-02 ships the visual LCD it
  renders into.
- **Plan 26-03** (polish bundle, planned) — will layer the `?` help
  overlay (D-26.8) and USER-mode per-key relabel (D-26.9) on top of
  this visual baseline. No changes to Display14Seg needed for those
  features.

## Self-Check: PASSED

**Files created (verified):**
- `hp41-gui/src/Display14Seg.tsx` — FOUND
- `hp41-gui/src/Display14Seg.test.tsx` — FOUND

**Commits (verified via `git log --oneline -3`):**
- `30e6eeb feat(26-02): add Display14Seg.tsx — 14-segment SVG LCD component (D-26.6/D-26.7)` — FOUND
- `3f9df28 feat(26-02): wire Display14Seg into App.tsx + commit .display svg sizing rule` — FOUND
- `12ca8b0 test(26-02): add Display14Seg vitest suite (constants + render + W4/W5/W6)` — FOUND
