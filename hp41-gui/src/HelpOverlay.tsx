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
//
// Phase 31-04 D-31.8 / D-31.9: Two top-level collapsible sections:
//   1. "HP-41CV (built-in)" — entries without `xrom` field
//   2. "Math 1 Pac (XROM 7)" — entries with `xrom.module === "Math 1"`
// Both sections expanded by default. Each section collapses independently.
// Within each section, JSON's per-program categories render as 2nd-level
// headers (existing .help-overlay-category-heading pattern). Entries sorted
// alphabetically within each category.

import { useState, useEffect, useMemo } from 'react';
import { helpEntriesAll, type HelpEntry } from './help_data';

export type HelpOverlayProps = {
    open: boolean;
    onClose: () => void;
};

/// Section descriptor for the two top-level overlay sections.
/// `predicate` selects which entries belong to this section.
interface SectionDef {
    id: 'hp41cv' | 'math1';
    heading: string;
    predicate: (e: HelpEntry) => boolean;
}

/// Two top-level sections per D-31.8. Order: built-in first, Math 1 Pac second.
const SECTIONS: SectionDef[] = [
    {
        id: 'hp41cv',
        heading: 'HP-41CV (built-in)',
        predicate: (e: HelpEntry) => !e.xrom,
    },
    {
        id: 'math1',
        heading: 'Math 1 Pac (XROM 7)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Math 1',
    },
];

export function HelpOverlay({ open, onClose }: HelpOverlayProps) {
    const [query, setQuery] = useState('');

    // D-31.8: Both sections expanded by default; state resets on each overlay open.
    const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean }>({
        hp41cv: true,
        math1: true,
    });

    // Reset query and expand state whenever overlay opens (clean-slate UX).
    useEffect(() => {
        if (open) {
            setQuery('');
            setExpanded({ hp41cv: true, math1: true });
        }
    }, [open]);

    // Full pool: built-in + Math Pac I entries (Phase 31-04 D-31.8).
    // Entries with key_path === null are excluded per D-26.8.
    const allEntries = useMemo(() =>
        helpEntriesAll().filter(e => e.key_path !== null),
    []);

    // Filter entries by search query.
    const filtered = useMemo(() => {
        const q = query.toLowerCase().trim();
        if (q === '') return allEntries;
        return allEntries.filter(e =>
            e.display_name.toLowerCase().includes(q) ||
            e.description.toLowerCase().includes(q) ||
            e.category.toLowerCase().includes(q)
        );
    }, [query, allEntries]);

    // Group entries by section → category. Sort alphabetically within each category.
    const sectionGroups = useMemo(() => {
        return SECTIONS.map(section => {
            const sectionEntries = filtered.filter(section.predicate);
            // Group by category (preserve insertion order for category discovery).
            const catMap = new Map<string, HelpEntry[]>();
            for (const entry of sectionEntries) {
                const arr = catMap.get(entry.category);
                if (arr) {
                    arr.push(entry);
                } else {
                    catMap.set(entry.category, [entry]);
                }
            }
            // Sort entries alphabetically within each category.
            for (const arr of catMap.values()) {
                arr.sort((a, b) => a.display_name.localeCompare(b.display_name));
            }
            return {
                section,
                groups: Array.from(catMap.entries()), // [[category, entries], ...]
                count: sectionEntries.length,
            };
        });
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

    const toggleSection = (id: 'hp41cv' | 'math1') => {
        setExpanded(prev => ({ ...prev, [id]: !prev[id] }));
    };

    const totalFiltered = filtered.length;

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
                {totalFiltered === 0 && query !== '' && (
                    <div className="help-overlay-empty">No functions match "{query}".</div>
                )}
                {sectionGroups.map(({ section, groups, count }) => (
                    <div key={section.id} className="help-overlay-section">
                        {/* Top-level collapsible section heading (D-31.8 / UI-SPEC §Accessibility) */}
                        <button
                            className="help-overlay-section-heading"
                            onClick={() => toggleSection(section.id)}
                            aria-expanded={expanded[section.id]}
                        >
                            {section.heading}
                            {query !== '' && ` (${count})`}
                        </button>
                        {/* Section body — only rendered when expanded */}
                        {expanded[section.id] && (
                            <div className="help-overlay-section-body">
                                {groups.map(([category, entries]) => (
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
                                {groups.length === 0 && query !== '' && (
                                    <div className="help-overlay-empty">No matches in this section.</div>
                                )}
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}

export default HelpOverlay;
