---
phase: 26-gui-integration-and-polish
plan: 02
type: execute
wave: 2
depends_on:
  - 01
files_modified:
  - hp41-gui/src/Display14Seg.tsx
  - hp41-gui/src/App.tsx
  - hp41-gui/src/App.css
  - hp41-gui/src/Display14Seg.test.tsx
autonomous: true
requirements:
  - FN-POLISH-01
must_haves:
  truths:
    - "The HP-41C 12-character display in the GUI renders via a 14-segment SVG component (one <g> per character with 14 segments each, total 12*14=168 paths per render)"
    - "Off segments render at ~10% opacity (faintly visible — the authentic HP-41C LCD aesthetic per D-26.6); on segments render at full opacity in LCD-green"
    - "Rendering '2.0000' produces visible digit glyphs with dim 'off' segments visible behind them"
    - "Modal preview text from Plan 26-01's renderModalLcd (e.g. 'STO _5', 'SF IND _5', 'CLP MYPRG_') flows through the new <Display14Seg text={displayText} /> component without layout breakage"
    - "Existing .display CSS (positioning, background, border-radius, min-height) is preserved unchanged per D-26.7"
    - "The 12-character display fits within the existing display panel boundaries (no horizontal overflow, no vertical re-layout)"
    - "Decimal point is rendered as part of the input string before passing to Display14Seg (W4: handled OUTSIDE the 14-seg grid via the SEGMENT_MAP '.' glyph that lights a small bottom-right dot path)"
  artifacts:
    - path: "hp41-gui/src/Display14Seg.tsx"
      provides: "NEW SVG component rendering 12 character cells with 14 segments each; SEGMENT_MAP covering A-Z, 0-9, period, comma, minus, plus, parentheses, equals, slash, colon, space, underscore (modal-cursor); SEGMENT_PATHS as a constant array of 14 SVG path 'd' attributes in the W4-pinned order ['a','b','c','d','e','f','g1','g2','h','i','j','k','l','m']"
      contains: "SEGMENT_MAP"
      exports: ["Display14Seg", "SEGMENT_MAP", "SEGMENT_PATHS"]
    - path: "hp41-gui/src/App.tsx"
      provides: "Drop-in replacement of <div className='display'>{displayText}</div> with <div className='display'><Display14Seg text={displayText} /></div> per D-26.7; displayText derivation from Plan 26-01 stays unchanged"
      contains: "<Display14Seg"
    - path: "hp41-gui/src/App.css"
      provides: ".display svg { display: block; width: 100%; height: 100%; } rule explicitly committed (W6: no longer optional). .display rule itself stays unchanged."
      contains: ".display svg"
    - path: "hp41-gui/src/Display14Seg.test.tsx"
      provides: "Vitest tests covering: SEGMENT_PATHS length === 14; SEGMENT_MAP covers digits, A-Z, punctuation; the SVG renders 12 cells regardless of input length; off-segment opacity is < 0.5 PER-CELL (W5: tightened scoping); on-segment opacity for '8' is >= 0.99; SVG has reasonable jsdom bounding-box dimensions (W6 verification)"
      contains: "Display14Seg"
  key_links:
    - from: "hp41-gui/src/App.tsx::displayText derivation"
      to: "<Display14Seg text={displayText} />"
      via: "drop-in inside existing .display div per D-26.7"
      pattern: "<Display14Seg"
    - from: "hp41-gui/src/Display14Seg.tsx::SEGMENT_MAP"
      to: "per-cell <path /> elements with opacity={lit ? 1.0 : 0.1}"
      via: "SEGMENT_MAP[char.toUpperCase()].includes(segIdx) determines lit state per D-26.6"
      pattern: "opacity=\\{lit"
---

<objective>
Replace the CSS-text HP-41C display with an authentic 14-segment SVG LCD font per D-26.6 (full segment grid with dim 'off' segments) and D-26.7 (drop-in component preserving existing `.display` CSS layout).

Purpose: Visual fidelity is one of the v2.2 polish goals (FN-POLISH-01). The current monospace-text display renders correctly but does not match the real HP-41C LCD aesthetic. Shipping a 14-segment SVG component with always-rendered dim 'off' segments delivers the authentic look the user picked during context (D-26.6: "full segment grid with dim 'off' segments" over "static lit-only").

Output:
- New `hp41-gui/src/Display14Seg.tsx` SVG component
- `SEGMENT_MAP: Record<string, number[]>` glyph table covering A-Z, 0-9, common punctuation, and the underscore (modal digit-entry cursor convention from D-26.3 / Plan 26-01 `renderModalLcd`)
- `SEGMENT_PATHS: string[]` 14 SVG path 'd' attributes in the canonical W4-pinned order `['a','b','c','d','e','f','g1','g2','h','i','j','k','l','m']`
- Drop-in replacement in `App.tsx`'s `<div className="display">` body; `.display` CSS unchanged per D-26.7
- Explicit `.display svg { display: block; width: 100%; height: 100%; }` rule committed (W6: no longer optional)
- Vitest tests confirming correct rendering of digits, modal previews, dim 'off' segments (W5: per-cell scoped), and reasonable SVG layout dimensions (W6 verification)
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/26-gui-integration-and-polish/26-CONTEXT.md
@.planning/phases/26-gui-integration-and-polish/26-PATTERNS.md
@.planning/phases/26-gui-integration-and-polish/26-01-modal-architecture-and-key-wiring-PLAN.md
@CLAUDE.md

@hp41-gui/src/App.tsx
@hp41-gui/src/App.css
@hp41-gui/src/Keyboard.tsx

<interfaces>
<!-- Component contract for Display14Seg -->

```typescript
export type Display14SegProps = {
    text: string;       // up to 12 characters; longer is sliced; shorter is right-padded with space
};

export default function Display14Seg(props: Display14SegProps): JSX.Element;

export const SEGMENT_MAP: Record<string, number[]>;  // exported for testing
export const SEGMENT_PATHS: string[];                  // exported for testing (length === 14)
```

CSS contract (PRESERVED — do NOT modify .display rule):
```css
.display {
  background: #111;
  padding: 6px 10px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 22px;
  text-align: right;
  color: #c8e6c9;
  letter-spacing: 0.05em;
  min-height: 2em;
  border-bottom: 1px solid #222;
  white-space: pre;
}
```
The new SVG renders inside this container. The font-size/font-family/text-align/color rules become irrelevant for the SVG (it has its own viewBox sizing) but stay in place for any future fallback.

Color palette (D-26.6):
- LIT segment: `#c8e6c9` (matches existing display color) OR `#a0ffa0` (slightly more LCD-green per CONTEXT D-26.6) — planner picks; document choice in code comment
- OFF segment: same color as lit, but `opacity={0.1}` per D-26.6 ("≈10% opacity, faintly visible")

**WARNING W4 — 14-segment numbering convention pinned:**

The standard Wikipedia "Fourteen-segment display" canonical labeling is used. The 14 segments per cell:

```
 aaaaa
fh ij b
f hij b
f hijb
 g1g2
elkmc
e lkmc
e lkmc
 ddddd
```

(Refined ASCII rendering — each character cell is divided into segments:)

- `a`: top horizontal
- `b`: top-right vertical
- `c`: bottom-right vertical
- `d`: bottom horizontal
- `e`: bottom-left vertical
- `f`: top-left vertical
- `g1`: middle horizontal, left half
- `g2`: middle horizontal, right half
- `h`: top-left diagonal (upper-left to middle-center)
- `i`: top vertical (top-center to middle-center)
- `j`: top-right diagonal (upper-right to middle-center)
- `k`: bottom-right diagonal (middle-center to lower-right)
- `l`: bottom vertical (middle-center to bottom-center)
- `m`: bottom-left diagonal (middle-center to lower-left)

`SEGMENT_PATHS` is pinned to exactly this order at indices 0..13:
```typescript
// Pinned per Wikipedia "Fourteen-segment display" canonical labels.
// Index 0 = 'a', 1 = 'b', 2 = 'c', 3 = 'd', 4 = 'e', 5 = 'f',
//       6 = 'g1', 7 = 'g2', 8 = 'h', 9 = 'i', 10 = 'j',
//       11 = 'k', 12 = 'l', 13 = 'm'
const SEGMENT_ORDER = ['a', 'b', 'c', 'd', 'e', 'f', 'g1', 'g2', 'h', 'i', 'j', 'k', 'l', 'm'] as const;
```

**Decimal point handling (W4 pinned):** Phase 26 inserts the decimal point into the input STRING before passing to Display14Seg (handled at a higher level — `format_hpnum` and `displayText` derivation in App.tsx already include the period character). The Display14Seg SEGMENT_MAP includes a `.` entry that lights a small dot path positioned at the lower-right of each cell. This dot path is RENDERED OUTSIDE the 14 main segments — it is a 15th `<path>` element appended to each cell with its own opacity toggle. Implementation note: the dot is NOT one of the 14 segments (SEGMENT_PATHS still has length exactly 14); a separate `DECIMAL_DOT_PATH: string` constant defines the dot's `d` attribute, and each cell `<g>` renders it conditionally based on whether the next character is `.` OR via a separate cell-position lookup. **Pin the simpler "next-char-is-period" rendering**: when iterating cells, if `chars[i+1] === '.'` then the dot inside cell `i` is lit; the period itself does not consume a cell. This matches HP-41 LCD hardware behavior (the period overlays the previous digit, not its own cell).

CONTEXT.md decisions cited: D-26.3 (modal preview LCD-replacement — feeds via displayText prop), D-26.6 (full segment grid with dim off segments), D-26.7 (drop-in inside existing .display div)
</interfaces>

</context>

<tasks>

<task type="execute">
  <name>Task 1: Create Display14Seg.tsx with pinned SEGMENT_ORDER, SEGMENT_PATHS, SEGMENT_MAP, DECIMAL_DOT_PATH, and per-cell SVG rendering</name>
  <files>hp41-gui/src/Display14Seg.tsx</files>
  <read_first>
    - hp41-gui/src/Keyboard.tsx (full 289 lines — SVG-grid construction idiom: <svg viewBox> + <defs> + per-cell <g transform={`translate(...)`}> + nested shape primitives)
    - hp41-gui/src/App.css (.display rule lines 41-52 — color palette, padding constraints, min-height)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/Display14Seg.tsx (CREATE)" lines 484-548 (SVG-component skeleton, SEGMENT_MAP shape, SEGMENT_PATHS array, opacity-toggle pattern, color palette)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.6 + D-26.7 lines 64-69 (full segment grid spec, 14-seg layout choice rationale)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md "Claude's Discretion" 14-seg glyph map authoring lines 110-112 (planner picks Wikipedia/Adafruit-style 7+2+4+1 split — W4 PINS this to Wikipedia canonical 14-name labeling)
  </read_first>
  <action>
Create `hp41-gui/src/Display14Seg.tsx` with the following structure:

(a) Module-level constants:

`const CELL_WIDTH = 24;` and `const CELL_HEIGHT = 40;` (or planner-chosen viewBox-relative units; pick coordinates that produce a 12-char-wide display that fits within the existing `.display` container at default browser zoom). Document the coordinate system in a top-of-file comment.

**W4 pinned SEGMENT_ORDER constant + SEGMENT_PATHS array:**

```typescript
// W4 (Phase 26): pinned per Wikipedia "Fourteen-segment display" canonical labels.
// SEGMENT_PATHS[0] = 'a' (top horizontal), [1] = 'b' (top-right), [2] = 'c' (bottom-right),
// [3] = 'd' (bottom), [4] = 'e' (bottom-left), [5] = 'f' (top-left),
// [6] = 'g1' (middle-left), [7] = 'g2' (middle-right),
// [8] = 'h' (NW diagonal), [9] = 'i' (top vertical center),
// [10] = 'j' (NE diagonal), [11] = 'k' (SE diagonal),
// [12] = 'l' (bottom vertical center), [13] = 'm' (SW diagonal)
//
// ASCII layout reference:
//  aaaaa
// fhij b
// f hij b
//  g1g2     (split middle bar: g1=left half, g2=right half)
// elkmc
// e lkmc
//  ddddd
//
export const SEGMENT_ORDER = ['a', 'b', 'c', 'd', 'e', 'f', 'g1', 'g2', 'h', 'i', 'j', 'k', 'l', 'm'] as const;

export const SEGMENT_PATHS: string[] = [
  // index 0: 'a' — top horizontal
  "M 2 2 L 22 2 L 21 4 L 3 4 Z",
  // index 1: 'b' — top-right vertical
  "M 22 2 L 22 19 L 20 18 L 20 4 Z",
  // index 2: 'c' — bottom-right vertical
  "M 22 21 L 22 38 L 20 36 L 20 22 Z",
  // index 3: 'd' — bottom horizontal
  "M 2 38 L 22 38 L 21 36 L 3 36 Z",
  // index 4: 'e' — bottom-left vertical
  "M 2 21 L 2 38 L 4 36 L 4 22 Z",
  // index 5: 'f' — top-left vertical
  "M 2 2 L 2 19 L 4 18 L 4 4 Z",
  // index 6: 'g1' — middle horizontal, left half
  "M 3 19 L 11 19 L 12 20 L 11 21 L 3 21 L 2 20 Z",
  // index 7: 'g2' — middle horizontal, right half
  "M 13 19 L 21 19 L 22 20 L 21 21 L 13 21 L 12 20 Z",
  // index 8: 'h' — NW diagonal (top-left corner to middle-center)
  "M 5 5 L 5 7 L 11 18 L 12 18 L 12 17 L 6 5 Z",
  // index 9: 'i' — top vertical center (top to middle)
  "M 11 4 L 13 4 L 13 18 L 11 18 Z",
  // index 10: 'j' — NE diagonal (top-right corner to middle-center)
  "M 19 5 L 18 5 L 12 17 L 12 18 L 13 18 L 19 7 Z",
  // index 11: 'k' — SE diagonal (middle-center to bottom-right corner)
  "M 13 22 L 12 22 L 12 23 L 18 35 L 19 35 L 19 33 Z",
  // index 12: 'l' — bottom vertical center (middle to bottom)
  "M 11 22 L 13 22 L 13 36 L 11 36 Z",
  // index 13: 'm' — SW diagonal (middle-center to bottom-left corner)
  "M 11 22 L 12 22 L 12 23 L 6 35 L 5 35 L 5 33 Z",
];
```

Length is exactly 14 (verified by test in Task 3). The exact path coordinates are planner-chosen — what is locked is the INDEX-TO-SEGMENT-NAME mapping per W4.

**Decimal point (separate from the 14 segments):**

```typescript
// W4: decimal point is rendered OUTSIDE the 14-segment grid as a 15th conditional dot.
// Per HP-41 LCD hardware behavior, the period overlays the previous digit (not its own cell).
// Display14Seg.tsx lights this dot in cell i when chars[i+1] === '.'; the period itself
// does not consume a cell slot.
export const DECIMAL_DOT_PATH = "M 20 35 L 22 35 L 22 37 L 20 37 Z";
```

`export const SEGMENT_MAP: Record<string, number[]>` — for each supported glyph, the array of segment indices (0-13) that should be LIT. Required keys per D-26.6:
- Digits 0-9
- Uppercase A-Z (HP-41 ALPHA mode is uppercase-only)
- `'.'` (decimal point — special: NOT one of the 14 segments, lit via DECIMAL_DOT_PATH on the previous cell; SEGMENT_MAP['.'] returns `[]` empty array for the cell itself since the period does not consume a cell, but the cell-rendering loop ignores period characters as cells — handled below)
- `','` (comma — same as period for HP-41 display)
- `'-'` (minus sign — lights g1 + g2: indices 6, 7)
- `'+'`, `'('`, `')'`, `'='`, `'/'`, `':'`, `' '` (space — empty array, all segments off)
- `'_'` (underscore — modal digit-entry cursor per D-26.3; lights only the bottom segment 'd' = index 3)

For canonical SEGMENT_MAP values for digits and uppercase letters, use the public-domain Wikipedia "Fourteen-segment display" article's character-to-segment table (e.g. `'0'` = a+b+c+d+e+f+i+l = [0, 1, 2, 3, 4, 5, 9, 12]; `'A'` = a+b+c+e+f+g1+g2 = [0, 1, 2, 4, 5, 6, 7]; etc.). Planner fills in the full A-Z + 0-9 table from this reference; cite the source in a code comment.

Lowercase letters are NOT in scope (HP-41 ALPHA is uppercase). For unknown glyphs, fall back to all-off (treated as space). Special HP-41 chars (Σ, π glyph, μ-superscript, etc.) are explicitly OUT OF SCOPE per D-26.6 ("special HP-41 chars are NOT in scope for Phase 26 — those land with v3.x").

(b) Component implementation following PATTERNS.md §"hp41-gui/src/Display14Seg.tsx" code skeleton lines 524-541, EXTENDED for the decimal-overlay rule per W4:

```typescript
export default function Display14Seg({ text }: { text: string }): JSX.Element {
    // W4: decimal point overlays previous cell — strip periods from cell layout and
    // pass them as "decimal-after-this-cell" flags. Slice to fit 12 cells of non-period chars.
    const cells: { char: string; hasDecimal: boolean }[] = [];
    for (let i = 0; i < text.length && cells.length < 12; i++) {
        const ch = text[i];
        if (ch === '.' && cells.length > 0) {
            cells[cells.length - 1].hasDecimal = true;
        } else {
            cells.push({ char: ch, hasDecimal: false });
        }
    }
    while (cells.length < 12) cells.push({ char: ' ', hasDecimal: false });

    const totalWidth = CELL_WIDTH * 12;
    return (
        <svg
            viewBox={`0 0 ${totalWidth} ${CELL_HEIGHT}`}
            xmlns="http://www.w3.org/2000/svg"
            aria-label="HP-41 14-segment display"
            preserveAspectRatio="xMidYMid meet"
        >
            {cells.map((cell, i) => {
                const litSet = SEGMENT_MAP[cell.char.toUpperCase()] ?? [];
                return (
                    <g key={i} transform={`translate(${i * CELL_WIDTH}, 0)`}>
                        {SEGMENT_PATHS.map((d, segIdx) => {
                            const lit = litSet.includes(segIdx);
                            return (
                                <path
                                    key={segIdx}
                                    d={d}
                                    fill="#c8e6c9"
                                    opacity={lit ? 1.0 : 0.1}
                                />
                            );
                        })}
                        {/* W4: decimal-point dot is the 15th path, conditional on next-char-is-period */}
                        <path
                            key="dot"
                            d={DECIMAL_DOT_PATH}
                            fill="#c8e6c9"
                            opacity={cell.hasDecimal ? 1.0 : 0.1}
                        />
                    </g>
                );
            })}
        </svg>
    );
}
```

(c) `aria-label` on the SVG for accessibility. The text content is implicitly conveyed via the visual segments; screen readers see the `aria-label="HP-41 14-segment display"` and the parent `.display` div's enclosing context.

(d) NO state, NO useEffect — Display14Seg is a pure render-only component. Re-renders happen when the parent passes a new `text` prop.

(e) Color palette: use `#c8e6c9` to match the existing `.display` color (per App.css line 47). Document in code comment that the brighter `#a0ffa0` from CONTEXT D-26.6 was considered and rejected for visual continuity with v2.0.

(f) Performance: 12 cells × 15 paths (14 segments + 1 decimal dot) = 180 SVG path elements per render. React handles this fine at human-scale refresh rates (key presses, ~10-30 Hz max). No virtualization needed. Document this in a code comment.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && pnpm --filter hp41-gui-frontend exec tsc --noEmit 2>&1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - File `hp41-gui/src/Display14Seg.tsx` exists
    - `grep -c "export default function Display14Seg" hp41-gui/src/Display14Seg.tsx` returns at least 1
    - `grep -c "export const SEGMENT_MAP" hp41-gui/src/Display14Seg.tsx` returns at least 1
    - `grep -c "export const SEGMENT_PATHS" hp41-gui/src/Display14Seg.tsx` returns at least 1
    - `grep -c "SEGMENT_ORDER" hp41-gui/src/Display14Seg.tsx` returns at least 1 (W4: pinned order constant present)
    - `grep -c "DECIMAL_DOT_PATH" hp41-gui/src/Display14Seg.tsx` returns at least 1 (W4: decimal handled outside the 14-seg grid)
    - SEGMENT_PATHS array length is exactly 14 (verified via test in Task 3)
    - SEGMENT_ORDER tuple is `['a', 'b', 'c', 'd', 'e', 'f', 'g1', 'g2', 'h', 'i', 'j', 'k', 'l', 'm']` exactly (verified via test in Task 3)
    - SEGMENT_MAP contains keys for at minimum: 0-9 (10 entries), A-Z (26 entries), '.', ',', '-', '+', '(', ')', '=', '/', ':', ' ', '_' (13 punct entries) → total ≥ 49 keys
    - `pnpm --filter hp41-gui-frontend exec tsc --noEmit` returns no type errors
    - The component renders with `<path opacity={lit ? 1.0 : 0.1}` per D-26.6 (verified by grep `opacity=\{lit`)
    - No `useState`, `useEffect`, or other hooks (pure render component)
  </acceptance_criteria>
  <done>
    Display14Seg.tsx created with W4-pinned SEGMENT_ORDER + SEGMENT_PATHS (14 entries) + SEGMENT_MAP (≥49 glyphs) + DECIMAL_DOT_PATH (period overlays previous cell, not its own slot); per-cell <g> rendering with opacity-toggle for off-segment dim per D-26.6; TypeScript clean.
  </done>
</task>

<task type="execute">
  <name>Task 2: Wire Display14Seg into App.tsx as drop-in replacement inside .display div + commit explicit .display svg sizing rule (W6)</name>
  <files>hp41-gui/src/App.tsx, hp41-gui/src/App.css</files>
  <read_first>
    - hp41-gui/src/App.tsx (current display rendering — find the `<div className="display">{...}</div>` block, around line 289 per PATTERNS.md note; the displayText derivation was added in Plan 26-01 Task 3 step i)
    - hp41-gui/src/App.css (.display rule lines 41-52; check whether any `.display svg` or `.display *` selector exists already)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/App.tsx (extend) — Display-text derivation pattern" lines 348-361 (D-26.3/D-26.7 swap)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/App.css (extend) — Drop-in component preserves CSS pattern" lines 700-716
  </read_first>
  <action>
In `App.tsx`:

(a) Add the import: `import Display14Seg from './Display14Seg';`

(b) Replace the body of `<div className="display">...</div>` from `{displayText}` (the text content added in Plan 26-01 Task 3 step i) to `<Display14Seg text={displayText} />`. The `displayText` derivation itself is unchanged — it remains:
```typescript
const displayText = pendingInput
    ? renderModalLcd(pendingInput)
    : (calcState?.display_str ?? '');
```

The wrapping `<div className="display">` itself stays; only its inner content swaps. This preserves the existing CSS positioning + background per D-26.7.

(c) Verify no other component reads from the inner text content of the `.display` div (search for `display.textContent` or similar — none expected; if found, refactor those callers to read from `displayText` state directly).

In `App.css`:

(d) The `.display` rule (lines 41-52) MUST stay unchanged per D-26.7. Do not touch its background, padding, font, color, or text-align rules.

(e) **WARNING W6 resolution — commit the SVG sizing rule explicitly** (no longer optional). Add the rule (do NOT leave as "test in Task 3 whether needed"):
```css
.display svg {
  display: block;
  width: 100%;
  height: 100%;
}
```
Append this rule immediately after the existing `.display` rule. Rationale: the SVG element's default inline-flow behavior leaves a small vertical baseline gap; explicitly setting `display: block` removes it. `width: 100%` ensures the SVG fills the available width; `height: 100%` lets the SVG viewBox + `preserveAspectRatio="xMidYMid meet"` (from Display14Seg.tsx) maintain aspect ratio within the container. Verified by the Task 3 jsdom bounding-box test.

(f) Visual sanity check (manual, called out in Done): start the dev server (`just gui-dev`), confirm the display renders with visible 14-segment glyphs and dim 'off' segments faintly visible behind. The user is expected to do this sanity check after the plan ships; planner notes it in the SUMMARY.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && pnpm --filter hp41-gui-frontend exec tsc --noEmit 2>&1 | tail -10 && grep -c "Display14Seg\|.display svg" hp41-gui/src/App.tsx hp41-gui/src/App.css</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "import Display14Seg" hp41-gui/src/App.tsx` returns at least 1
    - `grep -c "<Display14Seg text=" hp41-gui/src/App.tsx` returns at least 1
    - `grep -c "<div className=\"display\">" hp41-gui/src/App.tsx` returns at least 1 (wrapper div preserved)
    - The .display CSS rule in App.css lines 41-52 (or wherever it now lives) is BYTE-IDENTICAL to its prior content: `git diff hp41-gui/src/App.css` shows ONLY additive changes (no deletions to the existing .display rule)
    - `grep -c "^.display svg" hp41-gui/src/App.css` returns at least 1 (W6: explicit committed rule, not optional)
    - `pnpm --filter hp41-gui-frontend exec tsc --noEmit` returns no type errors
    - `just gui-check` passes (cargo check + tsc combined)
  </acceptance_criteria>
  <done>
    Display14Seg wired as drop-in inside the existing .display div; displayText derivation from Plan 26-01 feeds the text prop; .display CSS rule preserved unchanged per D-26.7; explicit `.display svg` sizing rule committed per W6; TypeScript clean.
  </done>
</task>

<task type="execute">
  <name>Task 3: Vitest tests for Display14Seg covering digits, modal previews, edge cases, per-cell off-segment scoping (W5), and SVG layout verification (W6)</name>
  <files>hp41-gui/src/Display14Seg.test.tsx</files>
  <read_first>
    - hp41-gui/src/Display14Seg.tsx (the implementation from Task 1)
    - hp41-gui/src/App.test.tsx (the Vitest setup from Plan 26-01 Task 3 step k — same testing harness, same imports)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.3 lines 41 (modal preview strings: "STO __" -> "STO _5" -> "STO 05"; "SF IND _5"; "CLP MYPRG_")
  </read_first>
  <action>
Create `hp41-gui/src/Display14Seg.test.tsx` with Vitest + @testing-library/react tests:

(a) Setup imports:
```typescript
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import Display14Seg, { SEGMENT_MAP, SEGMENT_PATHS, SEGMENT_ORDER } from './Display14Seg';
```

(b) Constant-shape tests (planner sanity checks):
```typescript
describe('Display14Seg constants', () => {
    it('SEGMENT_PATHS has exactly 14 segments', () => {
        expect(SEGMENT_PATHS.length).toBe(14);
    });
    it('SEGMENT_ORDER is W4-pinned canonical labels', () => {
        // W4: pinned per Wikipedia "Fourteen-segment display" canonical labels
        expect([...SEGMENT_ORDER]).toEqual(['a', 'b', 'c', 'd', 'e', 'f', 'g1', 'g2', 'h', 'i', 'j', 'k', 'l', 'm']);
    });
    it('SEGMENT_MAP covers all digits 0-9', () => {
        for (let d = 0; d <= 9; d++) {
            expect(SEGMENT_MAP[String(d)]).toBeDefined();
        }
    });
    it('SEGMENT_MAP covers all uppercase A-Z', () => {
        for (let c = 65; c <= 90; c++) {
            expect(SEGMENT_MAP[String.fromCharCode(c)]).toBeDefined();
        }
    });
    it('SEGMENT_MAP covers underscore (modal digit-entry cursor) per D-26.3', () => {
        expect(SEGMENT_MAP['_']).toBeDefined();
        expect(SEGMENT_MAP['_'].length).toBeGreaterThan(0);
    });
    it('SEGMENT_MAP space is empty (all segments off)', () => {
        expect(SEGMENT_MAP[' ']).toEqual([]);
    });
});
```

(c) Render-output tests (render-and-count, NOT pixel snapshots — pixel snapshots are flaky and require external setup):
```typescript
describe('Display14Seg rendering', () => {
    it("renders '2.0000' with correct number of cells (12, period overlays prev cell per W4)", () => {
        const { container } = render(<Display14Seg text="2.0000" />);
        const cells = container.querySelectorAll('g');
        expect(cells.length).toBe(12);  // 12 cells regardless of input length; '.' folds into prev cell
    });

    it("renders 12 * 15 = 180 path elements per render (14 segments + 1 decimal dot per cell, W4)", () => {
        const { container } = render(<Display14Seg text="2.0000" />);
        const paths = container.querySelectorAll('path');
        expect(paths.length).toBe(180);
    });

    it("renders modal preview 'STO _5' (Plan 26-01 D-26.3 register-modal cursor) without crashing", () => {
        const { container } = render(<Display14Seg text="STO _5" />);
        expect(container.querySelector('svg')).toBeTruthy();
    });

    it("renders modal preview 'SF IND _5' for FlagPrompt with IND toggled (D-26.2)", () => {
        const { container } = render(<Display14Seg text="SF IND _5" />);
        expect(container.querySelector('svg')).toBeTruthy();
    });

    it("renders modal preview 'CLP MYPRG_' for ClpLabel input", () => {
        const { container } = render(<Display14Seg text="CLP MYPRG_" />);
        expect(container.querySelector('svg')).toBeTruthy();
    });

    it("renders empty string as all-spaces (12 empty cells)", () => {
        const { container } = render(<Display14Seg text="" />);
        const cells = container.querySelectorAll('g');
        expect(cells.length).toBe(12);
    });

    it("truncates input longer than 12 chars to 12 cells", () => {
        const { container } = render(<Display14Seg text="ABCDEFGHIJKLMNOPQR" />);
        const cells = container.querySelectorAll('g');
        expect(cells.length).toBe(12);
    });

    it("W5 (TIGHTENED): off-segment opacity is approximately 0.1 PER CELL (not container-global)", () => {
        // W5 resolution: render a single space and check FIRST CELL's 14 paths individually.
        // Container-global paths[0] would yield false positives across cells; this tightening
        // verifies each segment within the first cell is dim.
        const { container } = render(<Display14Seg text=" " />);
        const firstCell = container.querySelector('g');
        expect(firstCell).not.toBeNull();
        const cellPaths = Array.from(firstCell!.querySelectorAll('path'));
        // First 14 paths in the cell are the 14 segments; index 14 is the decimal dot (W4)
        for (let segIdx = 0; segIdx < 14; segIdx++) {
            const opacity = parseFloat(cellPaths[segIdx].getAttribute('opacity') ?? '1');
            expect(opacity).toBeLessThan(0.5);
        }
        // The decimal dot (15th path) is also off for a space:
        const dotOpacity = parseFloat(cellPaths[14].getAttribute('opacity') ?? '1');
        expect(dotOpacity).toBeLessThan(0.5);
    });

    it("on-segment opacity is 1.0 for digit '8' (all segments lit)", () => {
        // '8' lights all numeric segments — most segments should be on
        const { container } = render(<Display14Seg text="8" />);
        const firstCell = container.querySelector('g');
        const onSegments = Array.from(firstCell?.querySelectorAll('path') ?? [])
            .filter(p => parseFloat(p.getAttribute('opacity') ?? '0') >= 0.99);
        expect(onSegments.length).toBeGreaterThan(5);  // '8' has many lit segments
    });

    it("W4: decimal point overlays previous cell (period does not consume a slot)", () => {
        // Render '2.5' — should produce 12 cells (2 with dot lit, 5, then 10 spaces).
        // The decimal dot on cell 0 (the '2') is lit; cell 1 is '5' with dot off.
        const { container } = render(<Display14Seg text="2.5" />);
        const cells = Array.from(container.querySelectorAll('g'));
        expect(cells.length).toBe(12);
        // The decimal dot is the LAST path in each cell (index 14 of the 15-path group)
        const dot0Opacity = parseFloat(cells[0].querySelectorAll('path')[14].getAttribute('opacity') ?? '0');
        expect(dot0Opacity).toBeGreaterThanOrEqual(0.99);
        const dot1Opacity = parseFloat(cells[1].querySelectorAll('path')[14].getAttribute('opacity') ?? '1');
        expect(dot1Opacity).toBeLessThan(0.5);
    });

    it("W6: SVG renders with non-zero jsdom bounding box (sizing rule applied)", () => {
        // Render inside a sized container to verify the .display svg { width:100%; height:100% }
        // rule produces a non-degenerate SVG layout. jsdom doesn't do real layout, but it does
        // expose width/height attributes and viewBox; we assert these are present + sensible.
        const { container } = render(<Display14Seg text="2.0000" />);
        const svg = container.querySelector('svg');
        expect(svg).not.toBeNull();
        const viewBox = svg!.getAttribute('viewBox');
        expect(viewBox).toMatch(/^0 0 \d+ \d+$/);
        const [, , vbW, vbH] = viewBox!.split(' ').map(Number);
        expect(vbW).toBeGreaterThanOrEqual(12 * 20); // 12 cells × ≥20 width per cell
        expect(vbH).toBeGreaterThan(0);
        expect(svg!.getAttribute('preserveAspectRatio')).toBe('xMidYMid meet');
    });
});
```

(d) Verify the test file runs via `pnpm --filter hp41-gui-frontend test --run`. If `@testing-library/react` is not yet a dev dependency in `hp41-gui/package.json`, add it (`pnpm --filter hp41-gui-frontend add -D @testing-library/react jsdom`) and configure vitest with `environment: 'jsdom'` in `hp41-gui/vite.config.ts` or `vitest.config.ts`. (Plan 26-01 Task 3 step k may have already done this — if so, no action needed here.)

(e) The "snapshot" terminology in earlier drafts was misleading — these are render-output assertions (path counts, opacity values, cell counts, viewBox shape), NOT vitest's `toMatchSnapshot()` API. Render-and-count is more robust for SVG component tests.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && pnpm --filter hp41-gui-frontend test --run Display14Seg 2>&1 | tail -30</automated>
  </verify>
  <acceptance_criteria>
    - `pnpm --filter hp41-gui-frontend test --run Display14Seg` passes with all tests green
    - At minimum 13 test cases (6 constant-shape + 7 rendering + W5 + W6 verification) all pass
    - Test file uses no external snapshot files (no `__snapshots__/` directory created — render-and-count tests only)
    - The "W5 (TIGHTENED): off-segment opacity is approximately 0.1 PER CELL" test confirms the dim-segments contract per D-26.6 by iterating the FIRST cell's 14 paths individually (not container-global paths[0])
    - The "W4: decimal point overlays previous cell" test confirms the period folding behavior
    - The "W6: SVG renders with non-zero jsdom bounding box" test verifies the viewBox + preserveAspectRatio attributes are correctly set (the explicit `.display svg` CSS rule from Task 2 step e is required for actual rendering size; this test verifies the SVG declares its sizing contract)
    - `pnpm --filter hp41-gui-frontend test --run` passes the FULL Vitest suite (Plan 26-01 + this plan combined)
    - `just gui-ci` passes
  </acceptance_criteria>
  <done>
    Display14Seg.test.tsx ships with W4-pinned constant-shape tests + render-output tests + W5 tightened per-cell off-segment scoping + W6 SVG layout verification; all tests green; full Vitest suite passes; gui-ci passes.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| `text` prop input → SVG rendering | Plan 26-01's renderModalLcd produces the modal preview strings; calcState.display_str comes from the backend |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-26-02-01 | Tampering | Backend-supplied display_str injected into SVG | mitigate | React's text-node rendering escapes text content automatically; the SEGMENT_MAP lookup `SEGMENT_MAP[ch.toUpperCase()]` only reads from a pre-defined object — no eval, no innerHTML. Even a malicious display_str like `<script>alert(1)</script>` renders as 12 character cells (each char looked up in SEGMENT_MAP, falling back to empty/space for unknown chars). XSS surface = zero. |
| T-26-02-02 | Information Disclosure | SVG aria-label could expose internals | accept | aria-label is a fixed string "HP-41 14-segment display" (Task 1 step c), not derived from text content. No PII or secret data exposed. |
| T-26-02-03 | Denial of Service | Extremely long text prop crashing render | mitigate | Display14Seg slices input to exactly 12 non-period cells (`.padEnd(12, ' ').slice(0, 12)` extended for period folding per W4) — bounded render cost regardless of input length. Test "truncates input longer than 12 chars to 12 cells" asserts this. |
</threat_model>

<verification>
1. `cd hp41-gui && pnpm --filter hp41-gui-frontend exec tsc --noEmit` — no type errors
2. `cd hp41-gui && pnpm --filter hp41-gui-frontend test --run` — full Vitest suite green (Plan 26-01 + this plan)
3. `just gui-ci` — full GUI CI pipeline green
4. Manual visual sanity (post-merge): `just gui-dev` boots the app; the display shows 14-segment glyphs with visible dim 'off' segments behind any displayed text; the calculator visually resembles a real HP-41C LCD
</verification>

<success_criteria>
- `Display14Seg.tsx` exists with `SEGMENT_PATHS` of length 14 in W4-pinned canonical order and `SEGMENT_MAP` covering ≥ 49 glyphs
- W4: SEGMENT_ORDER equals `['a','b','c','d','e','f','g1','g2','h','i','j','k','l','m']` exactly
- W4: decimal point is rendered OUTSIDE the 14-seg grid as a 15th conditional dot path (DECIMAL_DOT_PATH) that overlays the previous cell
- The component renders 12 cells × 15 paths = 180 SVG path elements per call
- 'Off' segments render at opacity ≤ 0.5 PER CELL (W5: tightened scoping) — typically exactly 0.1
- 'On' segments render at opacity ≥ 0.99 — typically exactly 1.0
- The displayText flow from Plan 26-01's `renderModalLcd` works through the new `<Display14Seg text={...} />` prop without layout breakage
- The existing `.display` CSS rule in `App.css` is preserved BYTE-IDENTICAL per D-26.7
- W6: `.display svg { display: block; width: 100%; height: 100%; }` rule is committed (not optional), verified by Vitest viewBox + preserveAspectRatio assertions
- All Display14Seg vitest tests pass; full test suite (Plan 26-01 + 26-02) passes
- `just gui-ci` is green
- Underscore glyph (`_`) lights at minimum the bottom segment 'd' (index 3 in SEGMENT_ORDER — modal digit-entry cursor convention from D-26.3)
</success_criteria>

<output>
After completion, create `.planning/phases/26-gui-integration-and-polish/26-02-SUMMARY.md` documenting:
- The W4-pinned 14-segment naming convention used (a/b/c/d/e/f/g1/g2/h/i/j/k/l/m per Wikipedia)
- The reference source used for segment paths (Wikipedia "Fourteen-segment display" canonical character table)
- The W4 decimal-overlay rendering choice (period folds into previous cell, lit via DECIMAL_DOT_PATH)
- The total SEGMENT_MAP entry count and any glyphs intentionally omitted (Σ, π, μ all deferred to v3.x per D-26.6)
- The chosen color palette (#c8e6c9 vs #a0ffa0)
- The W6 explicit `.display svg` CSS rule committed
- Confirmation that the .display rule itself is byte-identical to its prior content
- A note for the user to manually `just gui-dev` and visually confirm the LCD aesthetic matches the HP-41C
</output>
</content>
</invoke>