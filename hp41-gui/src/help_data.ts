// Phase 26 Plan 03 D-26.8 / D-25.16 — TypeScript port of hp41-cli/src/help_data.rs.
//
// `docs/hp41cv-functions.json` is the SINGLE SOURCE OF TRUTH for the GUI's
// `?` help overlay AND the CLI's help overlay AND the generated function
// matrix (`docs/hp41cv-function-matrix.md`). This module imports the JSON
// at build time via vite's static JSON-import (D-25.16); the resulting
// array is baked into the production bundle — zero runtime fetch.
//
// Hard-build-blocker semantics (D-25.17 / D-26.8): a malformed JSON file
// fails vite's build step. This is intentional — canonical data files must
// not be empty / malformed.
//
// Phase 31-04: parallel-loads docs/hp41-math1-functions.json via Vite static
// JSON-import (D-31.10 / C-28.3 / ADR-005). Mirrors hp41-cli/src/help_data.rs
// Phase 29 D-29.2 second OnceLock + merged accessor pattern.

import functions from '../../docs/hp41cv-functions.json';
import math1Functions from '../../docs/hp41-math1-functions.json';

/// XROM module reference attached to Math Pac I (and future v3.1+ pac) entries.
/// Matches the `xrom` object shape in docs/hp41-math1-functions.json (ADR-005 /
/// C-28.3 / Phase 29 D-29.1).
export interface XromEntry {
    /// Human-readable module name, e.g. "Math 1".
    module: string;
    /// HP-41 hardware XROM module ID (7 for Math Pac I).
    module_id: number;
    /// 1-based function index within the module.
    function_id: number;
}

/// One row in the canonical HP-41CV function table.
///
/// Mirrors hp41-cli::help_data::HelpEntry (Rust) field-for-field. See
/// `hp41-cli/src/help_data.rs` lines 24-57 for the canonical schema.
///
/// Phase 31-04 extends with optional `xrom` field: present on Math Pac I
/// entries (loaded from docs/hp41-math1-functions.json), absent on v2.2
/// built-in entries.
export interface HelpEntry {
    /// Op variant name (PascalCase, e.g. `"Pi"`). For XEQ-by-Name-only
    /// conditional tests this is an `_XEQ`-suffixed alias.
    op_variant: string;
    /// HP-41 mnemonic as shown on the display (e.g. `"PI"`).
    display_name: string;
    /// One of the 20 enumerated categories.
    category: string;
    /// `"implemented"`, `"deferred-v3"`, or `"na"`.
    status: 'implemented' | 'deferred-v3' | 'na';
    /// GSD phase ID string (e.g. `"21"`) or `null` for v3.x.
    phase: string | null;
    /// CLI keystroke (e.g. `"f-7"`) or `null` for internal / XEQ-by-Name-only.
    key_path: string | null;
    /// <= 80 chars, suitable for the `?` overlay row.
    description: string;
    /// Optional free-form notes about HP-41 hardware divergences.
    divergences?: string[];
    /// XROM module reference — present on Math Pac I entries, absent on
    /// v2.2 built-in entries. Used by HelpOverlay.tsx to partition entries
    /// into "HP-41CV (built-in)" vs "Math 1 Pac (XROM 7)" sections (D-31.8).
    xrom?: XromEntry;
}

/// Lazy-init cache. Vite's static `import` is itself the cache (module
/// evaluation is one-shot), so no OnceLock-equivalent is needed — the
/// `functions` binding is evaluated once at module load time.
export function helpEntries(): readonly HelpEntry[] {
    return functions as readonly HelpEntry[];
}

/// One row of the help overlay table, produced by `helpOverlayRows`.
/// Category headers carry `isHeader: true` with `desc: "=== <name> ==="`
/// and empty `key`/`op`.
export interface HelpOverlayRow {
    key: string;
    op: string;
    desc: string;
    isHeader: boolean;
    category: string;
}

/// Render a list of help overlay rows with category-header rows interleaved.
/// Categories appear in their first-appearance order in the JSON; within a
/// category, entries keep the JSON's declared order.
///
/// Entries with `key_path === null` are EXCLUDED per D-26.8 (XEQ-by-Name-only
/// ops aren't keyboard shortcuts and would just clutter the overlay).
export function helpOverlayRows(): readonly HelpOverlayRow[] {
    const entries = helpEntries();
    const categories: string[] = [];
    for (const entry of entries) {
        if (entry.key_path !== null && !categories.includes(entry.category)) {
            categories.push(entry.category);
        }
    }
    const rows: HelpOverlayRow[] = [];
    for (const cat of categories) {
        rows.push({
            key: '',
            op: '',
            desc: `=== ${cat} ===`,
            isHeader: true,
            category: cat,
        });
        for (const entry of entries.filter(e => e.category === cat && e.key_path !== null)) {
            rows.push({
                key: entry.key_path ?? '',
                op: entry.display_name,
                desc: entry.description,
                isHeader: false,
                category: cat,
            });
        }
    }
    return rows;
}

/// Filter help entries by a free-text query. The query is matched
/// case-insensitively against `display_name`, `description`, and `category`.
/// Entries with `key_path === null` are always excluded (D-26.8).
///
/// An empty query returns all entries that have a `key_path` (no filtering).
export function filterHelpEntries(query: string): readonly HelpEntry[] {
    const q = query.toLowerCase().trim();
    if (q === '') {
        return helpEntries().filter(e => e.key_path !== null);
    }
    return helpEntries().filter(e =>
        e.key_path !== null && (
            e.display_name.toLowerCase().includes(q) ||
            e.description.toLowerCase().includes(q) ||
            e.category.toLowerCase().includes(q)
        )
    );
}

/// Phase 31-04: Math Pac I function entries from docs/hp41-math1-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per
/// D-25.17 (parallel to hp41-cli/src/help_data.rs `.expect("...malformed")`).
/// Mirrors Phase 29 D-29.2 second OnceLock + accessor pattern in Rust.
export function helpEntriesMath1(): readonly HelpEntry[] {
    return math1Functions as readonly HelpEntry[];
}

/// Phase 31-04: Merged accessor returning all built-in + Math Pac I entries.
///
/// Parallel to hp41-cli/src/help_data.rs::help_entries_all() (Phase 29 D-29.2).
/// Used by HelpOverlay.tsx to obtain the full entry pool; the overlay then
/// partitions by presence of `entry.xrom` into two sections (D-31.8).
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1()];
}
