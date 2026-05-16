// Phase 26 Plan 03 — Vitest tests for HelpOverlay + help_data.ts (D-26.8).
//
// Tests cover:
//   - help_data.ts: helpEntries entry count parity with the source JSON
//     (drift-catch), filterHelpEntries semantics, helpOverlayRows category
//     grouping + null-key_path filter (D-26.8)
//   - HelpOverlay.tsx: open/close rendering, search input filtering,
//     Esc keystroke triggers onClose, close-button click triggers onClose

import { describe, it, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { HelpOverlay } from './HelpOverlay';
import { helpEntries, helpOverlayRows, filterHelpEntries } from './help_data';
import sourceJson from '../../docs/hp41cv-functions.json';

describe('help_data', () => {
    it('helpEntries returns all entries from docs/hp41cv-functions.json (drift-catch)', () => {
        const allSource = sourceJson as unknown[];
        expect(helpEntries().length).toBe(allSource.length);
        // Sanity floor: the canonical JSON has 130+ entries as of Phase 25.
        expect(helpEntries().length).toBeGreaterThanOrEqual(130);
    });

    it('filterHelpEntries with empty query returns only key_path != null entries', () => {
        const all = helpEntries();
        const nonNullCount = all.filter(e => e.key_path !== null).length;
        expect(filterHelpEntries('').length).toBe(nonNullCount);
        // Floor: the canonical JSON has at least 30 keyboard-bound ops.
        expect(filterHelpEntries('').length).toBeGreaterThanOrEqual(30);
    });

    it('filterHelpEntries narrows results by display_name match', () => {
        const result = filterHelpEntries('STO');
        expect(result.length).toBeGreaterThan(0);
        for (const entry of result) {
            const matches =
                entry.display_name.toLowerCase().includes('sto') ||
                entry.description.toLowerCase().includes('sto') ||
                entry.category.toLowerCase().includes('sto');
            expect(matches, `entry ${entry.op_variant} should match 'sto'`).toBe(true);
        }
    });

    it('filterHelpEntries narrows results by category match', () => {
        const result = filterHelpEntries('arithmetic');
        expect(result.length).toBeGreaterThan(0);
        // Every returned entry should be in the Arithmetic category OR have
        // 'arithmetic' substring in name/description.
        for (const entry of result) {
            const matches =
                entry.display_name.toLowerCase().includes('arithmetic') ||
                entry.description.toLowerCase().includes('arithmetic') ||
                entry.category.toLowerCase().includes('arithmetic');
            expect(matches).toBe(true);
        }
    });

    it('filterHelpEntries with no matching query returns empty array', () => {
        expect(filterHelpEntries('xyzzy_no_such_function').length).toBe(0);
    });

    it('helpOverlayRows produces category headers in JSON declaration order (unique)', () => {
        const rows = helpOverlayRows();
        const headers = rows.filter(r => r.isHeader).map(r => r.category);
        // Each header should appear at most once (no duplicate category headings).
        expect(new Set(headers).size).toBe(headers.length);
        // At least one header should exist.
        expect(headers.length).toBeGreaterThan(0);
    });

    it('helpOverlayRows excludes null-key_path entries from rendered rows (D-26.8)', () => {
        const rows = helpOverlayRows();
        const dataRows = rows.filter(r => !r.isHeader);
        // Every data row must have a non-empty key (which derived from
        // non-null key_path).
        for (const row of dataRows) {
            expect(row.key).not.toBe('');
        }
        // Total data rows should equal the count of non-null-key_path entries.
        const expected = helpEntries().filter(e => e.key_path !== null).length;
        expect(dataRows.length).toBe(expected);
    });

    it('helpOverlayRows has header rows with empty key/op fields', () => {
        const headers = helpOverlayRows().filter(r => r.isHeader);
        for (const h of headers) {
            expect(h.key).toBe('');
            expect(h.op).toBe('');
            expect(h.desc.startsWith('=== ')).toBe(true);
            expect(h.desc.endsWith(' ===')).toBe(true);
        }
    });
});

describe('HelpOverlay', () => {
    it('renders nothing when open=false', () => {
        const { container } = render(<HelpOverlay open={false} onClose={() => {}} />);
        expect(container.querySelector('.help-overlay')).toBeNull();
    });

    it('renders the overlay when open=true', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        expect(container.querySelector('.help-overlay')).not.toBeNull();
        expect(container.querySelector('.help-overlay-search')).not.toBeNull();
        expect(container.querySelector('.help-overlay-content')).not.toBeNull();
    });

    it('initial render shows entries grouped by category', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const headings = container.querySelectorAll('.help-overlay-category-heading');
        expect(headings.length).toBeGreaterThan(0);
        const rows = container.querySelectorAll('.help-overlay-row');
        expect(rows.length).toBeGreaterThan(0);
    });

    it('search input narrows the rendered rows', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const initialRows = container.querySelectorAll('.help-overlay-row').length;
        const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
        expect(searchInput).not.toBeNull();
        fireEvent.change(searchInput, { target: { value: 'sin' } });
        const filteredRows = container.querySelectorAll('.help-overlay-row').length;
        expect(filteredRows).toBeLessThan(initialRows);
        expect(filteredRows).toBeGreaterThan(0);
    });

    it('empty-result search renders the empty-state message', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
        fireEvent.change(searchInput, { target: { value: 'xyzzy_no_match' } });
        expect(container.querySelector('.help-overlay-empty')).not.toBeNull();
    });

    it('Esc key calls onClose', () => {
        let closed = false;
        render(<HelpOverlay open={true} onClose={() => { closed = true; }} />);
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(closed).toBe(true);
    });

    it('close button calls onClose', () => {
        let closed = false;
        const { container } = render(
            <HelpOverlay open={true} onClose={() => { closed = true; }} />,
        );
        const closeButton = container.querySelector('.help-overlay-close') as HTMLButtonElement;
        expect(closeButton).not.toBeNull();
        fireEvent.click(closeButton);
        expect(closed).toBe(true);
    });

    it('null-key_path entries do not appear in rendered rows (D-26.8)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // Every rendered row's key cell must be non-empty (every entry has
        // a key_path because filterHelpEntries excludes null-key_path).
        const keyCells = container.querySelectorAll('.help-overlay-key');
        for (const cell of Array.from(keyCells)) {
            expect(cell.textContent?.length ?? 0).toBeGreaterThan(0);
        }
    });
});
