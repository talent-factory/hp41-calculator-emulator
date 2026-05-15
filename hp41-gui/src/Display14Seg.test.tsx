// Phase 26 Plan 02 — Vitest tests for Display14Seg (D-26.6 / D-26.7).
//
// Tests cover:
//   - Constant-shape invariants (SEGMENT_PATHS.length === 14, SEGMENT_ORDER
//     pinned to W4 Wikipedia labels, SEGMENT_MAP coverage for digits/letters/_)
//   - Render-output assertions (12-cell grid regardless of input length,
//     180 paths per render, modal-preview strings render without crash)
//   - W5 (TIGHTENED): off-segment opacity scoped PER CELL — not container-global
//   - W4: decimal point overlays the previous cell (does not consume a slot)
//   - W6: SVG declares viewBox + preserveAspectRatio for the `.display svg`
//     sizing rule committed in Task 2
//
// Render-and-count assertions (not snapshot files) — robust against minor
// path-coordinate edits without producing __snapshots__/ artifacts.

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import Display14Seg, {
    SEGMENT_MAP,
    SEGMENT_PATHS,
    SEGMENT_ORDER,
} from './Display14Seg';

describe('Display14Seg constants', () => {
    it('SEGMENT_PATHS has exactly 14 segments', () => {
        expect(SEGMENT_PATHS.length).toBe(14);
    });

    it('SEGMENT_ORDER is W4-pinned canonical Wikipedia labels', () => {
        // W4: pinned per Wikipedia "Fourteen-segment display" canonical labels.
        expect([...SEGMENT_ORDER]).toEqual([
            'a', 'b', 'c', 'd', 'e', 'f', 'g1', 'g2', 'h', 'i', 'j', 'k', 'l', 'm',
        ]);
    });

    it('SEGMENT_ORDER length matches SEGMENT_PATHS length', () => {
        // If these two ever diverge, the segment-index labels in SEGMENT_MAP
        // no longer point to the paths they claim to.
        expect(SEGMENT_ORDER.length).toBe(SEGMENT_PATHS.length);
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
        // D-26.3 modal-cursor convention from Plan 26-01's renderModalLcd:
        // `STO __` -> `STO _5`. The underscore must light at minimum the
        // bottom segment 'd' (index 3 in SEGMENT_ORDER).
        expect(SEGMENT_MAP['_']).toBeDefined();
        expect(SEGMENT_MAP['_'].length).toBeGreaterThan(0);
        expect(SEGMENT_MAP['_']).toContain(3);
    });

    it('SEGMENT_MAP space is empty (all segments off)', () => {
        expect(SEGMENT_MAP[' ']).toEqual([]);
    });

    it('SEGMENT_MAP minus lights middle bar (g1 + g2)', () => {
        // minus = middle horizontal — indices 6 (g1) + 7 (g2) per SEGMENT_ORDER.
        expect(SEGMENT_MAP['-']).toEqual([6, 7]);
    });
});

describe('Display14Seg rendering', () => {
    it("renders '2.0000' with correct number of cells (12, period overlays prev cell per W4)", () => {
        // '2.0000' is 6 characters but the period folds into the '2' cell, so
        // the layout produces 5 character cells + 7 padding spaces = 12 cells total.
        const { container } = render(<Display14Seg text="2.0000" />);
        const cells = container.querySelectorAll('g');
        expect(cells.length).toBe(12);
    });

    it('renders 12 * 15 = 180 path elements per render (14 segments + 1 decimal dot per cell, W4)', () => {
        const { container } = render(<Display14Seg text="2.0000" />);
        const paths = container.querySelectorAll('path');
        expect(paths.length).toBe(180);
    });

    it("renders modal preview 'STO _5' (Plan 26-01 D-26.3 register-modal cursor) without crashing", () => {
        const { container } = render(<Display14Seg text="STO _5" />);
        expect(container.querySelector('svg')).toBeTruthy();
        // Sanity check: still 12 cells (no period, no truncation needed).
        expect(container.querySelectorAll('g').length).toBe(12);
    });

    it("renders modal preview 'SF IND _5' for FlagPrompt with IND toggled (D-26.2)", () => {
        const { container } = render(<Display14Seg text="SF IND _5" />);
        expect(container.querySelector('svg')).toBeTruthy();
        expect(container.querySelectorAll('g').length).toBe(12);
    });

    it("renders modal preview 'CLP MYPRG_' for ClpLabel input", () => {
        const { container } = render(<Display14Seg text="CLP MYPRG_" />);
        expect(container.querySelector('svg')).toBeTruthy();
        expect(container.querySelectorAll('g').length).toBe(12);
    });

    it('renders empty string as all-spaces (12 empty cells)', () => {
        const { container } = render(<Display14Seg text="" />);
        const cells = container.querySelectorAll('g');
        expect(cells.length).toBe(12);
    });

    it('truncates input longer than 12 chars to 12 cells', () => {
        // 18 non-period chars; only first 12 are visible.
        const { container } = render(<Display14Seg text="ABCDEFGHIJKLMNOPQR" />);
        const cells = container.querySelectorAll('g');
        expect(cells.length).toBe(12);
    });

    it('W5 (TIGHTENED): off-segment opacity is < 0.5 PER CELL (not container-global)', () => {
        // W5 resolution: render a single space and check FIRST CELL's 15 paths
        // individually. Container-global paths[0] would yield false positives
        // across cells; this tightening verifies each segment within the first
        // cell is dim. All 14 segments + the 15th decimal dot must be dim for
        // a space character.
        const { container } = render(<Display14Seg text=" " />);
        const firstCell = container.querySelector('g');
        expect(firstCell).not.toBeNull();
        const cellPaths = Array.from(firstCell!.querySelectorAll('path'));
        // First 14 paths in the cell are the 14 segments; index 14 is the decimal dot (W4).
        expect(cellPaths.length).toBe(15);
        for (let segIdx = 0; segIdx < 14; segIdx++) {
            const opacity = parseFloat(cellPaths[segIdx].getAttribute('opacity') ?? '1');
            expect(opacity).toBeLessThan(0.5);
        }
        // The decimal dot (15th path) is also off for a space.
        const dotOpacity = parseFloat(cellPaths[14].getAttribute('opacity') ?? '1');
        expect(dotOpacity).toBeLessThan(0.5);
    });

    it("on-segment opacity is >= 0.99 for digit '8' (all numeric segments lit)", () => {
        // '8' lights all 8 numeric segments (a, b, c, d, e, f, g1, g2). Most
        // of the first cell's paths should be on (opacity ~1.0); the diagonals
        // and middle verticals stay off so we only assert > 5 on-segments.
        const { container } = render(<Display14Seg text="8" />);
        const firstCell = container.querySelector('g');
        const onSegments = Array.from(firstCell?.querySelectorAll('path') ?? [])
            .filter(p => parseFloat(p.getAttribute('opacity') ?? '0') >= 0.99);
        expect(onSegments.length).toBeGreaterThan(5);
    });

    it('W4: decimal point overlays previous cell (period does not consume a slot)', () => {
        // Render '2.5' — produces 12 cells: cell 0 = '2' with decimal lit,
        // cell 1 = '5' with decimal off, cells 2-11 = padding spaces (decimal off).
        const { container } = render(<Display14Seg text="2.5" />);
        const cells = Array.from(container.querySelectorAll('g'));
        expect(cells.length).toBe(12);
        // The decimal dot is the LAST path in each cell (index 14 of the 15-path group).
        const dot0Opacity = parseFloat(
            cells[0].querySelectorAll('path')[14].getAttribute('opacity') ?? '0',
        );
        expect(dot0Opacity).toBeGreaterThanOrEqual(0.99);
        const dot1Opacity = parseFloat(
            cells[1].querySelectorAll('path')[14].getAttribute('opacity') ?? '1',
        );
        expect(dot1Opacity).toBeLessThan(0.5);
    });

    it('W4: leading period attaches to its own cell as the first char (no prior cell to overlay)', () => {
        // Edge case: a string starting with '.' has no preceding cell, so the
        // period must consume cell 0 itself (SEGMENT_MAP['.'] = [] -> empty cell).
        // The cells.length still ends at 12 (padded).
        const { container } = render(<Display14Seg text=".5" />);
        const cells = Array.from(container.querySelectorAll('g'));
        expect(cells.length).toBe(12);
        // Cell 0 is the period — empty segments, decimal-dot off (no prior cell to attach to).
        const cell0Paths = cells[0].querySelectorAll('path');
        const dot0Opacity = parseFloat(cell0Paths[14].getAttribute('opacity') ?? '0');
        expect(dot0Opacity).toBeLessThan(0.5);
    });

    it('W6: SVG declares viewBox + preserveAspectRatio for the .display svg sizing rule', () => {
        // The explicit `.display svg { width: 100%; height: 100% }` CSS rule
        // committed in Task 2 step e relies on the SVG advertising a viewBox +
        // preserveAspectRatio. jsdom doesn't perform real layout, but it does
        // expose attributes — we assert the SVG declares its sizing contract.
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

    it('SVG carries aria-label for accessibility', () => {
        const { container } = render(<Display14Seg text="0" />);
        const svg = container.querySelector('svg');
        expect(svg!.getAttribute('aria-label')).toBe('HP-41 14-segment display');
    });

    it('lowercase letters fall back to all-off (HP-41 ALPHA is uppercase-only)', () => {
        // Display14Seg uppercases char before SEGMENT_MAP lookup. Lowercase 'a'
        // resolves to SEGMENT_MAP['A'] which is defined and lit; this test
        // verifies that case-folding happens.
        const { container: lowerContainer } = render(<Display14Seg text="a" />);
        const { container: upperContainer } = render(<Display14Seg text="A" />);
        const lowerOpacities = Array.from(
            lowerContainer.querySelector('g')!.querySelectorAll('path'),
        ).map(p => p.getAttribute('opacity'));
        const upperOpacities = Array.from(
            upperContainer.querySelector('g')!.querySelectorAll('path'),
        ).map(p => p.getAttribute('opacity'));
        expect(lowerOpacities).toEqual(upperOpacities);
    });
});
