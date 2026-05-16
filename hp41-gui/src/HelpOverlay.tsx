// Phase 26 Plan 03 D-26.8 — `?` help overlay.
//
// Full-cover semi-transparent React modal listing every HP-41CV function
// from `docs/hp41cv-functions.json` (via help_data.ts). Search input
// filters across display_name + description + category. Categories from
// the JSON declaration order become section headings. Entries with
// `key_path === null` are excluded per D-26.8.
//
// Parent App.tsx owns the open/close state and the `?`-keystroke handler.
// The overlay's own Esc handler is defense-in-depth: App.tsx handles Esc
// precedence (help → modal → shift), and this component also closes on
// Esc when mounted.

import { useState, useEffect, useMemo } from 'react';
import { filterHelpEntries, type HelpEntry } from './help_data';

export type HelpOverlayProps = {
    open: boolean;
    onClose: () => void;
};

export function HelpOverlay({ open, onClose }: HelpOverlayProps) {
    const [query, setQuery] = useState('');
    const filtered = useMemo(() => filterHelpEntries(query), [query]);

    // Group filtered entries by category (preserve JSON declaration order
    // via Map insertion order).
    const grouped = useMemo(() => {
        const groups = new Map<string, HelpEntry[]>();
        for (const entry of filtered) {
            const arr = groups.get(entry.category);
            if (arr) {
                arr.push(entry);
            } else {
                groups.set(entry.category, [entry]);
            }
        }
        return Array.from(groups.entries()); // [[category, entries], ...] in insertion order
    }, [filtered]);

    // Esc-close: defense-in-depth. App.tsx::handleKey already handles Esc
    // precedence (help → modal → shift); this listener ensures the overlay
    // closes even if mounted in a context where the parent listener is
    // unavailable (e.g. tests rendering the component standalone).
    useEffect(() => {
        if (!open) return;
        const onKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            }
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, [open, onClose]);

    if (!open) return null;

    return (
        <div className="help-overlay" role="dialog" aria-label="HP-41 function reference">
            <div className="help-overlay-header">
                <input
                    className="help-overlay-search"
                    type="text"
                    value={query}
                    onChange={e => setQuery(e.target.value)}
                    placeholder="Search functions..."
                    autoFocus
                    aria-label="Search HP-41 functions"
                />
                <button
                    className="help-overlay-close"
                    onClick={onClose}
                    aria-label="Close help overlay"
                >
                    ×
                </button>
            </div>
            <div className="help-overlay-content">
                {grouped.map(([category, entries]) => (
                    <div key={category} className="help-overlay-category">
                        <h3 className="help-overlay-category-heading">{category}</h3>
                        {entries.map(entry => (
                            <div key={entry.op_variant} className="help-overlay-row">
                                <span className="help-overlay-key">{entry.key_path}</span>
                                <span className="help-overlay-op">{entry.display_name}</span>
                                <span className="help-overlay-desc">{entry.description}</span>
                            </div>
                        ))}
                    </div>
                ))}
                {filtered.length === 0 && (
                    <div className="help-overlay-empty">No functions match "{query}".</div>
                )}
            </div>
        </div>
    );
}

export default HelpOverlay;
