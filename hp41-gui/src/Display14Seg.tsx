// Phase 26 Plan 02 — D-26.6 / D-26.7 — 14-segment SVG LCD font for the HP-41C display.
//
// Drop-in replacement inside the existing `.display` div (App.tsx). The .display CSS
// (background, padding, color, min-height) remains unchanged per D-26.7; only the
// inner text content is swapped for this SVG component.
//
// ── Coordinate system ─────────────────────────────────────────────────────────
//
// Each character cell is rendered in a 24-wide × 40-high local coordinate space
// (CELL_WIDTH × CELL_HEIGHT). The 12-character display has a 288 × 40 viewBox.
// The SVG's preserveAspectRatio="xMidYMid meet" + the `.display svg { width: 100%;
// height: 100% }` rule (App.css, committed in Task 2 per W6) make the SVG scale
// to the existing `.display` container while preserving aspect ratio.
//
// ── Segment numbering (W4 pinned — Wikipedia "Fourteen-segment display") ────
//
// SEGMENT_PATHS[0] = 'a' (top horizontal),    [1] = 'b' (top-right vertical),
//                [2] = 'c' (bottom-right),    [3] = 'd' (bottom horizontal),
//                [4] = 'e' (bottom-left),     [5] = 'f' (top-left vertical),
//                [6] = 'g1' (middle-left),    [7] = 'g2' (middle-right),
//                [8] = 'h' (NW diagonal),     [9] = 'i' (top vertical center),
//               [10] = 'j' (NE diagonal),    [11] = 'k' (SE diagonal),
//               [12] = 'l' (bottom vertical center), [13] = 'm' (SW diagonal)
//
//      aaaaa
//     fhij b
//     f hij b
//      g1g2     (split middle bar)
//     elkmc
//     e lkmc
//      ddddd
//
// The decimal point is rendered OUTSIDE the 14-segment grid as a 15th conditional
// dot path (DECIMAL_DOT_PATH). Per HP-41 LCD hardware behavior, the period overlays
// the previous digit, not its own cell. When iterating cells, if chars[i+1] === '.'
// the dot inside cell i is lit; the period itself does not consume a cell slot.
//
// ── Color palette (D-26.6) ────────────────────────────────────────────────────
//
// LIT/OFF segments use the same color (#c8e6c9, matching the existing `.display`
// color from App.css line 47) and differ only in opacity. The brighter `#a0ffa0`
// candidate from CONTEXT.md D-26.6 was considered and rejected for visual
// continuity with v2.0's existing LCD aesthetic. OFF segments render at
// opacity=0.1 (faintly visible — the authentic HP-41C LCD aesthetic per D-26.6).
//
// ── Performance ────────────────────────────────────────────────────────────────
//
// 12 cells × 15 paths (14 segments + 1 decimal dot) = 180 SVG path elements per
// render. React handles this fine at human-scale refresh rates (key presses,
// ~10-30 Hz max). No virtualization needed. No state, no useEffect — pure render.

const CELL_WIDTH = 24;
const CELL_HEIGHT = 40;
const LIT_COLOR = '#c8e6c9';      // matches existing .display color (App.css line 47)
const ON_OPACITY = 1.0;
const OFF_OPACITY = 0.1;          // D-26.6: faintly visible dim 'off' segments

// W4 (Phase 26): pinned per Wikipedia "Fourteen-segment display" canonical labels.
// Exported for the constant-shape test in Display14Seg.test.tsx.
export const SEGMENT_ORDER = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g1', 'g2', 'h', 'i', 'j', 'k', 'l', 'm',
] as const;

// SVG path 'd' attributes for each segment, indexed per SEGMENT_ORDER above.
// Coordinates are within the 24×40 cell coordinate space; each segment is a
// trapezoidal/rectangular polygon for clean LCD-like rendering. Length is
// exactly 14 (verified by `Display14Seg.test.tsx::SEGMENT_PATHS has exactly
// 14 segments`).
export const SEGMENT_PATHS: string[] = [
    // index 0: 'a' — top horizontal
    'M 4 2 L 20 2 L 18 4 L 6 4 Z',
    // index 1: 'b' — top-right vertical
    'M 21 3 L 21 19 L 19 18 L 19 5 Z',
    // index 2: 'c' — bottom-right vertical
    'M 21 21 L 21 37 L 19 35 L 19 22 Z',
    // index 3: 'd' — bottom horizontal
    'M 4 38 L 20 38 L 18 36 L 6 36 Z',
    // index 4: 'e' — bottom-left vertical
    'M 3 21 L 3 37 L 5 35 L 5 22 Z',
    // index 5: 'f' — top-left vertical
    'M 3 3 L 3 19 L 5 18 L 5 5 Z',
    // index 6: 'g1' — middle horizontal, left half
    'M 4 19 L 11 19 L 12 20 L 11 21 L 4 21 L 3 20 Z',
    // index 7: 'g2' — middle horizontal, right half
    'M 13 19 L 20 19 L 21 20 L 20 21 L 13 21 L 12 20 Z',
    // index 8: 'h' — NW diagonal (top-left to middle-center); 2-unit parallelogram
    'M 6 5 L 8 5 L 12 18 L 10 18 Z',
    // index 9: 'i' — top vertical center (top to middle)
    'M 11 4 L 13 4 L 13 18 L 11 18 Z',
    // index 10: 'j' — NE diagonal (top-right to middle-center); 2-unit parallelogram
    'M 18 5 L 16 5 L 12 18 L 14 18 Z',
    // index 11: 'k' — SE diagonal (middle-center to bottom-right); 2-unit parallelogram
    'M 12 22 L 14 22 L 18 35 L 16 35 Z',
    // index 12: 'l' — bottom vertical center (middle to bottom)
    'M 11 22 L 13 22 L 13 36 L 11 36 Z',
    // index 13: 'm' — SW diagonal (middle-center to bottom-left); 2-unit parallelogram
    'M 12 22 L 10 22 L 6 35 L 8 35 Z',
];

// W4: decimal point is rendered OUTSIDE the 14-segment grid as a 15th conditional
// dot. Per HP-41 LCD hardware behavior, the period overlays the previous digit
// (not its own cell). Display14Seg.tsx lights this dot in cell i when
// chars[i+1] === '.'; the period itself does not consume a cell slot.
//
// Sized 3×3 at (x=22-25, y=37-40). Horizontally centered in the inter-digit
// gap (cell 'c' right edge at x=21, next cell digit-stem at global x=27, so
// gap center at local x≈24). Vertically placed below the 'd' baseline (y=38)
// with only a 1-unit overlap so it remains visually distinct from the digit's
// segments. The prior position at x=20-23 sat inside the 'c' bottom-right
// vertical segment and was visually hidden for almost every digit.
export const DECIMAL_DOT_PATH = 'M 22 37 L 25 37 L 25 40 L 22 40 Z';

// Character-to-segment-indices map (W4 pinned).
//
// Canonical glyph data follows the public-domain Wikipedia "Fourteen-segment
// display" article's character-to-segment table, restricted to the HP-41
// character set per D-26.6 (A-Z, 0-9, period, comma, minus, plus, parens,
// equals, slash, colon, space, underscore). HP-41 ALPHA mode is uppercase-only
// so lowercase letters are NOT in scope. Special HP-41 chars (Σ, π glyph,
// μ-superscript, …) are explicitly OUT OF SCOPE per D-26.6 — those land with
// v3.x ALPHA-special-charset expansion. Unknown glyphs fall back to all-off
// (treated as space).
//
// Indices reference SEGMENT_ORDER:
//   a=0, b=1, c=2, d=3, e=4, f=5, g1=6, g2=7, h=8, i=9, j=10, k=11, l=12, m=13
//
// (`.` is mapped to an empty cell-segment array — the period does not consume
// a cell; the decimal dot lights via DECIMAL_DOT_PATH on the PREVIOUS cell,
// handled by the next-char-is-period rule in the component below.)
export const SEGMENT_MAP: Record<string, number[]> = {
    // ── Digits 0-9 ─────────────────────────────────────────────────────────
    '0': [0, 1, 2, 3, 4, 5, 10, 13],          // include j+m diagonals so '0' looks distinct from 'O'
    '1': [1, 2, 10],                           // top-right + bottom-right + NE diagonal entering
    '2': [0, 1, 6, 7, 4, 3],
    '3': [0, 1, 7, 2, 3],
    '4': [5, 6, 7, 1, 2],
    '5': [0, 5, 6, 7, 2, 3],
    '6': [0, 5, 6, 7, 4, 3, 2],
    '7': [0, 1, 2],
    '8': [0, 1, 2, 3, 4, 5, 6, 7],
    '9': [0, 1, 2, 3, 5, 6, 7],
    // ── Uppercase A-Z (HP-41 ALPHA is uppercase only) ─────────────────────
    'A': [0, 1, 2, 4, 5, 6, 7],
    'B': [0, 1, 2, 3, 7, 9, 12],               // distinct from 8 via i+l verticals
    'C': [0, 5, 4, 3],
    'D': [0, 1, 2, 3, 9, 12],                  // distinct from 0 via i+l verticals
    'E': [0, 5, 6, 7, 4, 3],
    'F': [0, 5, 6, 7, 4],
    'G': [0, 5, 4, 3, 2, 7],
    'H': [5, 1, 6, 7, 4, 2],
    'I': [0, 9, 12, 3],
    'J': [1, 2, 3, 4],
    'K': [5, 4, 6, 10, 11],                    // left-bar + diagonals
    'L': [5, 4, 3],
    'M': [5, 1, 4, 2, 8, 10],                  // left+right + NW + NE diagonals
    'N': [5, 1, 4, 2, 8, 11],                  // left+right + NW + SE diagonals
    'O': [0, 1, 2, 3, 4, 5],
    'P': [0, 1, 5, 6, 7, 4],
    'Q': [0, 1, 2, 3, 4, 5, 11],               // O + SE diagonal
    'R': [0, 1, 5, 6, 7, 4, 11],               // P + SE diagonal
    'S': [0, 5, 6, 7, 2, 3],                   // identical to '5' (standard 14-seg convention)
    'T': [0, 9, 12],                           // top + center verticals
    'U': [5, 1, 4, 2, 3],
    'V': [5, 4, 10, 13],                       // left bars + NE + SW diagonals (mirror of A on bottom)
    'W': [5, 1, 4, 2, 11, 13],                 // left+right + SE + SW diagonals
    'X': [8, 10, 11, 13],                      // all four diagonals
    'Y': [8, 10, 12],                          // NW + NE diagonals + bottom vertical center
    'Z': [0, 10, 13, 3],                       // top + NE + SW + bottom (Z shape via diagonals)
    // ── Punctuation per D-26.6 ─────────────────────────────────────────────
    '.': [],                                   // period folds into previous cell via DECIMAL_DOT_PATH
    ',': [],                                   // same as period for HP-41 display
    '-': [6, 7],                               // minus = middle bar (g1 + g2)
    '+': [6, 7, 9, 12],                        // plus = middle bar + center verticals
    '(': [0, 5, 4, 3],                         // looks like C
    ')': [0, 1, 2, 3],                         // looks like reversed C
    '=': [6, 7, 3],                            // middle bar + bottom (two horizontals)
    '/': [10, 13],                             // NE + SW diagonals
    ':': [9, 12],                              // two center verticals (compromise — 14-seg has no dots)
    ' ': [],                                   // space — all segments off
    '_': [3],                                  // underscore = bottom segment only (D-26.3 modal cursor)
    '?': [0, 1, 7, 12],                        // upper hook + center bottom vertical
    '*': [6, 7, 8, 9, 10, 11, 12, 13],         // asterisk = full center + all 4 diagonals
};

export type Display14SegProps = {
    text: string;                              // up to 12 non-period cells; longer is sliced; shorter is right-padded with space
};

/**
 * 14-segment SVG LCD display (D-26.6 + D-26.7).
 *
 * Pure render-only component. Re-renders happen when the parent passes a new
 * `text` prop. No state, no useEffect. Renders 12 character cells; each cell
 * has 14 segments (+ 1 decimal-overlay dot). Off segments are dimmed via
 * opacity, never hidden — the always-rendered grid is the authentic HP-41C
 * LCD aesthetic per D-26.6.
 */
export default function Display14Seg({ text }: Display14SegProps) {
    // W4: decimal point overlays previous cell — strip periods from cell layout
    // and pass them as "decimal-after-this-cell" flags. Slice to fit 12 cells of
    // non-period chars. Longer input is truncated; shorter input is right-padded
    // with space so the display always renders a stable 12-cell grid.
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
            // Phase 27 Plan 04 Task 1 — WebdriverIO E2E locator (RESEARCH Pitfall 10).
            // data-testid is the selector hook; data-text exposes the raw text so the
            // smoke spec can assert against a DOM attribute when the LCD renders only
            // SVG segment paths (no plain text content). Allowed under SC-4 — this is
            // hp41-gui/src/ which is OUTSIDE the SC-4 boundary (which scopes hp41-gui/src-tauri/).
            data-testid="lcd-display"
            data-text={text}
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
                                    fill={LIT_COLOR}
                                    opacity={lit ? ON_OPACITY : OFF_OPACITY}
                                />
                            );
                        })}
                        {/* W4: decimal-point dot is the 15th path, conditional on next-char-is-period. */}
                        <path
                            key="dot"
                            d={DECIMAL_DOT_PATH}
                            fill={LIT_COLOR}
                            opacity={cell.hasDecimal ? ON_OPACITY : OFF_OPACITY}
                        />
                    </g>
                );
            })}
        </svg>
    );
}
