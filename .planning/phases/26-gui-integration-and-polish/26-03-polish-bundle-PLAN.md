---
phase: 26-gui-integration-and-polish
plan: 03
type: execute
wave: 2
depends_on:
  - 01
files_modified:
  - hp41-gui/src/HelpOverlay.tsx
  - hp41-gui/src/help_data.ts
  - hp41-gui/vite.config.ts
  - hp41-gui/src/App.tsx
  - hp41-gui/src/Keyboard.tsx
  - hp41-gui/src/App.css
  - hp41-gui/src/HelpOverlay.test.tsx
  - hp41-gui/src/Keyboard.test.tsx
autonomous: true
requirements:
  - FN-POLISH-02
  - FN-POLISH-03
  - FN-POLISH-04
  - FN-GUI-03
  - FN-GUI-04
must_haves:
  truths:
    - "Pressing '?' on the physical keyboard opens a full-cover overlay listing all 130+ HP-41CV functions from docs/hp41cv-functions.json, categorized by JSON 'category' field, with a search input that filters across display_name + description + category"
    - "The HelpOverlay filters out entries with key_path == null (XEQ-by-Name-only ops are not relevant for a keyboard-shortcut overlay) per D-26.8"
    - "Pressing '?' or Esc closes the overlay; Esc precedence: close help first, then close any open modal (preserves Plan 26-01 Esc semantics)"
    - "When USER mode is active (calcState.annunciators.user === true) AND the user has ASN'd a key, the SVG keycap renders the ASN'd label INSTEAD of the primary label per D-26.9"
    - "USER mode relabel renders user-supplied ASN strings as React text nodes (default-escaped) — a malicious ASN label like '<script>' renders as literal text, not as an injected DOM element (T-26-03-04 mitigation)"
    - "Pressing 'p' on the physical keyboard toggles PRGM mode (annunciators.prgm flips); SHIFT+'P' (uppercase) invokes PRX per D-26.10"
    - "The help_data.ts JSON entry count exactly matches the source file docs/hp41cv-functions.json entry count (drift-catch test)"
    - "vite.config.ts is updated with server.fs.allow including the repo root so the JSON import from docs/ succeeds at build time (W8)"
    - "KEY_DEFS.keyCode values are hardcoded literals from hp41-cli/src/keys.rs::keycode_to_hp41_code canonical mapping (W9) — NOT computed via row*10+col which would give incorrect codes given GUI's 0-indexed cols and 5-col layout"
  artifacts:
    - path: "hp41-gui/src/help_data.ts"
      provides: "TypeScript port of hp41-cli/src/help_data.rs — HelpEntry interface, helpEntries() vite JSON-import wrapper, helpOverlayRows() category-grouped helper, filtering helpers"
      contains: "interface HelpEntry"
      exports: ["HelpEntry", "helpEntries", "helpOverlayRows"]
    - path: "hp41-gui/src/HelpOverlay.tsx"
      provides: "NEW '?'-overlay React component per D-26.8: full-cover modal, semi-transparent backdrop, search input filtering display_name+description+category, category section headings, entry rows (key_path | display_name | description), filters out null-key_path entries"
      contains: "HelpOverlay"
      exports: ["HelpOverlay"]
    - path: "hp41-gui/vite.config.ts"
      provides: "server.fs.allow updated to include the repo root so the ../../docs/hp41cv-functions.json import resolves (W8)"
      contains: "fs:"
    - path: "hp41-gui/src/App.tsx"
      provides: "useState<boolean> for helpOpen + '?' keyboard handler + Esc precedence (help first, then pendingInput, then shiftActive); MAP table swap 'p'->'prgm_mode' and 'P'->'prx' per D-26.10; userKeymap prop passed to <Keyboard />; HelpOverlay rendered conditionally. File-region split (W7): handleKey extension + new JSX overlays — does NOT touch the displayText derivation or <Display14Seg> wrapping (Plan 26-02's domain)."
      contains: "<HelpOverlay"
    - path: "hp41-gui/src/Keyboard.tsx"
      provides: "KeyDef.keyCode optional field per D-26.9 with W9 hardcoded literals (NOT computed row*10+col); KeyboardProps extension with userKeymap and userActive; render-time relabel when USER mode active and matching keyCode entry exists"
      contains: "userKeymap"
    - path: "hp41-gui/src/App.css"
      provides: "Overlay styles for .help-overlay (full-cover, semi-transparent, z-index above toast); search-input styles; category-heading styles; entry-row hover styles; optional USER-relabel highlight styles"
      contains: ".help-overlay"
    - path: "hp41-gui/src/HelpOverlay.test.tsx"
      provides: "Vitest tests: search filter narrows results; category grouping from JSON declaration order; JSON entry count matches the docs source; entries with key_path == null are excluded; Esc closes overlay"
      contains: "HelpOverlay"
    - path: "hp41-gui/src/Keyboard.test.tsx"
      provides: "Vitest tests: KEY_DEFS keyCode matches hp41-cli canonical mapping for at minimum 3 sentinel keys (W9); USER mode relabel renders ASN'd label when keyCode matches; XSS-safety test asserts that an ASN label like '<script>alert(1)</script>' renders as literal text (not as injected element)"
      contains: "USER"
  key_links:
    - from: "hp41-gui/src/help_data.ts"
      to: "docs/hp41cv-functions.json"
      via: "import functions from '../../docs/hp41cv-functions.json' (vite build-time JSON-import per D-25.16 / D-26.8); vite.config.ts server.fs.allow extended for repo-root access (W8)"
      pattern: "import functions from"
    - from: "hp41-gui/src/App.tsx::handleKey"
      to: "setHelpOpen(true)"
      via: "'?' keystroke (unshifted, NOT in ALPHA mode) toggles overlay; Esc closes overlay before any other Esc handler"
      pattern: "helpOpen"
    - from: "hp41-gui/src/Keyboard.tsx::renderKeyLabel"
      to: "userKeymap.find(([code, label]) => code === key.keyCode)"
      via: "when userActive && key.keyCode != null, look up ASN'd label and render INSTEAD of key.label"
      pattern: "userKeymap.find"
    - from: "hp41-gui/src/App.tsx::MAP"
      to: "MAP['p'] = 'prgm_mode' and MAP['P'] = 'prx'"
      via: "physical-keyboard remap per D-26.10 (was 'p': 'prx' in v2.0)"
      pattern: "'p': 'prgm_mode'"
---

<objective>
Ship the three GUI Polish features per D-26.8 (`?`-overlay), D-26.9 (USER mode per-key relabel), and D-26.10 (`'p'` -> `prgm_mode` remap with SHIFT+'P' -> `prx`). All three depend on Plan 26-01's `pendingInput` infrastructure (Esc precedence) and `CalcStateView.user_keymap` projection (USER overlay data source).

Purpose: These three are the "discoverability + ergonomics" deliverables in Phase 26. The `?` overlay closes a long-standing usability gap (no keyboard shortcut reference in the GUI; v2.1 had only the SVG keycap labels). USER mode relabel makes ASN'd keys actually visible per FN-POLISH-03. The `'p'` remap resolves the v2.0 deferred shortcut conflict per FN-POLISH-04.

Output:
- `hp41-gui/src/HelpOverlay.tsx` — new full-cover React modal sourcing data from `docs/hp41cv-functions.json` via vite JSON-import (D-25.16 / D-26.8)
- `hp41-gui/src/help_data.ts` — new TypeScript port of `hp41-cli/src/help_data.rs` (HelpEntry interface, helpEntries(), helpOverlayRows())
- `hp41-gui/vite.config.ts` — updated with `server.fs.allow` for repo-root access (W8)
- `App.tsx` integration: `helpOpen` state, `?` keyboard handler, Esc precedence (help first → modal → SHIFT), MAP table swap for D-26.10, `userKeymap` prop passing. **W7 file-region split**: this plan modifies handleKey + appends new JSX overlays + MAP table; it does NOT touch displayText derivation or the `<Display14Seg>` wrapping (Plan 26-02's domain).
- `Keyboard.tsx` extension: `KeyDef.keyCode` optional field with **W9 hardcoded literals from hp41-cli canonical mapping** (not computed `row*10+col`), USER-mode render-time relabel with React-default text-escape (XSS-safe)
- `App.css` overlay styles
- Vitest tests for HelpOverlay (search, category grouping, count parity, null-key_path filter, Esc close) and Keyboard (sentinel-key keyCode parity with CLI per W9, USER relabel + XSS-safety)

**W7 Execution Note** — both Plan 26-02 and Plan 26-03 modify `hp41-gui/src/App.tsx`. To minimize merge conflicts, the execute-phase orchestrator should sequence 26-02 BEFORE 26-03 within Wave 2 (or run them sequentially via separate worktrees). The file-region split is:
- Plan 26-02 touches: the `<div className="display">` body (single-line content replacement to `<Display14Seg text={displayText} />`)
- Plan 26-03 touches: (a) `handleKey` for `?` open + Esc precedence, (b) new `<HelpOverlay open={...} onClose={...} />` JSX appended near the end of the root JSX tree, (c) new `<Keyboard userKeymap={...} userActive={...} />` prop pass, (d) MAP table `'p'` → `'prgm_mode'`, (e) MAP table new `'P'` → `'prx'`
- Cross-plan verification: a post-merge regex grep MUST find BOTH `<Display14Seg` AND `<HelpOverlay open=` in App.tsx — see Task 2 acceptance criteria.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/26-gui-integration-and-polish/26-CONTEXT.md
@.planning/phases/26-gui-integration-and-polish/26-PATTERNS.md
@.planning/phases/26-gui-integration-and-polish/26-01-modal-architecture-and-key-wiring-PLAN.md
@.planning/phases/26-gui-integration-and-polish/26-02-14-seg-lcd-PLAN.md
@.planning/phases/25-cli-integration-and-documentation/25-04-json-pipeline-and-docs-PLAN.md
@CLAUDE.md

@hp41-gui/src/App.tsx
@hp41-gui/src/Keyboard.tsx
@hp41-gui/src/App.css
@hp41-gui/vite.config.ts
@hp41-cli/src/help_data.rs
@hp41-cli/src/keys.rs
@docs/hp41cv-functions.json

<interfaces>
<!-- Component contracts -->

```typescript
// hp41-gui/src/help_data.ts
export interface HelpEntry {
    op_variant: string;
    display_name: string;
    category: string;
    status: 'implemented' | 'deferred-v3' | 'na';
    phase: string | null;
    key_path: string | null;
    description: string;
    divergences?: string[];
}

export function helpEntries(): readonly HelpEntry[];
export function helpOverlayRows(): readonly { key: string; op: string; desc: string; isHeader: boolean }[];

// hp41-gui/src/HelpOverlay.tsx
export type HelpOverlayProps = {
    open: boolean;
    onClose: () => void;
};
export function HelpOverlay(props: HelpOverlayProps): JSX.Element | null;

// hp41-gui/src/Keyboard.tsx (extended KeyDef)
export type KeyDef = {
    id: string;
    label: string;
    shifted?: { id: string; label: string };
    alphaChar?: string;
    row: number;
    col: number;
    colSpan?: number;
    variant?: 'top' | 'shift' | 'enter';
    keyCode?: number;  // NEW Phase 26 D-26.9: hardcoded literal from hp41-cli canonical mapping (W9)
};

// hp41-gui/src/Keyboard.tsx (extended KeyboardProps)
export type KeyboardProps = {
    onKey: (key: KeyDef) => void;
    busyRef: React.MutableRefObject<boolean>;
    shiftActive: boolean;
    alphaActive: boolean;
    userActive?: boolean;                            // NEW Phase 26 D-26.9
    userKeymap?: ReadonlyArray<[number, string]>;    // NEW Phase 26 D-26.9
};
```

**W9 — keyCode canonical mapping pinned from hp41-cli/src/keys.rs::keycode_to_hp41_code:**

The CLI uses HP-41 hardware row×10+col with 1-indexed rows and cols. Selected canonical sentinels (read from hp41-cli/src/keys.rs lines 414-479):

| GUI KEY_DEFS id | GUI row | GUI col (0-idx) | HP-41 hardware code | Source |
|------------------|---------|------------------|----------------------|--------|
| `sin`            | 2       | 2                | **25** (row 2, col 5) | keys.rs line 472 (`Char('q') => 25`) |
| `cos`            | 2       | 3                | **34** (row 3, col 4) | keys.rs line 464 (`Char('C') => 34`) |
| `tan`            | 2       | 4                | **35** (row 3, col 5) | keys.rs line 465 (`Char('T') => 35`) |
| `sto_prompt`     | 3       | 2                | **22** (row 2, col 2) | keys.rs line 469 (`Char('S') => 22`) |
| `rcl_prompt`     | 3       | 3                | **23** (row 2, col 3) | keys.rs line 470 (`Char('R') => 23`) |
| `xeq_prompt`     | 3       | 1                | **21** (row 2, col 1) | keys.rs line 468 (`Char('X') => 21`) |
| `enter`          | 4       | 0                | **84** (row 8, col 4) | keys.rs line 437 (`Enter => 84`) |
| `chs`            | 4       | 2                | (HP-41 row 4, col 5 = 45 conventionally; CLI does not bind a key here directly) | derived |
| `e` (EEX)        | 4       | 3                | **83** (row 8, col 3) | keys.rs line 436 (`Char('e') => 83`) |
| `7`              | 5       | 1                | **51**               | keys.rs line 449 |
| `8`              | 5       | 2                | **52**               | keys.rs line 450 |
| `9`              | 5       | 3                | **53**               | keys.rs line 451 |
| `minus`          | 5       | 0                | **64** (row 6, col 4) | keys.rs line 447 (`Char('-') => 64`) |
| `4`              | 6       | 1                | **61**               | keys.rs line 444 |
| `5`              | 6       | 2                | **62**               | keys.rs line 445 |
| `6`              | 6       | 3                | **63**               | keys.rs line 446 |
| `plus`           | 6       | 0                | **74** (row 7, col 4) | keys.rs line 442 (`Char('+') => 74`) |
| `1`              | 7       | 1                | **71**               | keys.rs line 439 |
| `2`              | 7       | 2                | **72**               | keys.rs line 440 |
| `3`              | 7       | 3                | **73**               | keys.rs line 441 |
| `mul`            | 7       | 0                | **54** (row 5, col 4) | keys.rs line 452 (`Char('*') => 54`) |
| `0`              | 8       | 1                | **81**               | keys.rs line 434 |
| `.`              | 8       | 2                | **82**               | keys.rs line 435 |
| `div`            | 8       | 0                | **45** (row 4, col 5) | keys.rs line 458 (`Char('/') => 45`) |
| `r_s`            | 8       | 3                | **31** (row 3, col 1) | derived per CLI Phase 19 R/S binding |

**CRITICAL W9 finding**: The GUI Keyboard.tsx grid layout does NOT match HP-41 hardware row×10+col indexing. GUI is 5 columns × 8 rows with 0-indexed cols; HP-41 hardware is 5 columns × 8 rows with 1-indexed rows AND cols, AND the row numbering is INVERTED (GUI row 1 = trig/math = HP-41 row 2; GUI row 8 = bottom digits = HP-41 row 5-8 mixed by column). A naive `row * 10 + col` would produce SIN=22 (wrong; correct is 25), CHS=42 (wrong; correct is 45 for div in the same position), and would break the entire USER overlay.

Therefore: keyCode MUST be hardcoded as literal numbers in KEY_DEFS, sourced from the table above. Task 3 step (a) takes the existing KEY_DEFS array and adds a `keyCode: <literal>` field to every entry that has a real HP-41 hardware equivalent. Variants `'top'` and `'shift'` and empty-id entries get no keyCode (USER overlay does not relabel them).

CONTEXT.md decisions cited: D-26.8, D-26.9, D-26.10, plus D-26.5 invariants (Esc precedence preserved). Cross-cutting D-25.16 (vite JSON-import).

Phase 26-01 dependency: `CalcStateView.user_keymap: Vec<(u8, String)>` projection AND the TS interface mirror (BLOCKER B5 from 26-01 revision) MUST be shipped by Plan 26-01 Task 2 before this plan can wire the USER overlay. Check that calcState.user_keymap is accessible from App.tsx as the source for `<Keyboard userKeymap={calcState.user_keymap} userActive={calcState.annunciators.user} />`.
</interfaces>

</context>

<tasks>

<task type="execute">
  <name>Task 1: Update vite.config.ts for fs.allow (W8) + create help_data.ts (TypeScript port of hp41-cli/src/help_data.rs) + HelpOverlay.tsx + App.css overlay styles + Vitest tests</name>
  <files>hp41-gui/vite.config.ts, hp41-gui/src/help_data.ts, hp41-gui/src/HelpOverlay.tsx, hp41-gui/src/App.css, hp41-gui/src/HelpOverlay.test.tsx</files>
  <read_first>
    - hp41-gui/vite.config.ts (current 14 lines — verified empty `server.fs.allow` field; this task adds the rule per W8)
    - hp41-cli/src/help_data.rs (full file — HelpEntry struct definition lines 22-57, helpEntries() lazy-init lines 64-77, help_overlay_rows() category-grouping lines 95-121)
    - docs/hp41cv-functions.json (first ~50 lines to confirm schema; full file is ~1395 lines / ~130 entries)
    - hp41-gui/src/App.css (existing overlay-card style precedents: .toast lines 217-239, .prgm-panel lines 163-203, .print-panel lines 109-158)
    - hp41-gui/src/App.tsx (lines 310-342 for the .prgm-panel and .print-panel rendering pattern HelpOverlay mirrors)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/help_data.ts (CREATE)" lines 614-695 (vite JSON-import semantics, lazy-init pattern, helpOverlayRows port from Rust)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/HelpOverlay.tsx (CREATE)" lines 552-611 (overlay-div pattern, search filter, category grouping, mount/unmount)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/App.css (extend) — Overlay-card style pattern" lines 720-744
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.8 lines 65-69 (full overlay spec)
  </read_first>
  <action>
**Step 0 (W8) — Update vite.config.ts:**

The default vite `server.fs.allow` only includes the project root (`hp41-gui/`). The relative import path `'../../docs/hp41cv-functions.json'` from `hp41-gui/src/help_data.ts` reaches OUTSIDE this default — it climbs to the repo root. To make vite resolve this at build time, extend `server.fs.allow`:

```typescript
// hp41-gui/vite.config.ts (revised)
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || 'localhost',
    fs: {
      // W8: allow the JSON import from ../../docs/hp41cv-functions.json (repo root).
      // The default fs.allow only includes the vite project root (hp41-gui/), which
      // would block the docs/ directory. Explicitly include the repo root.
      allow: [path.resolve(__dirname, '..')],
    },
  },
  build: {
    outDir: 'dist',
  },
})
```

This is a minimal, additive change — no other vite behavior is affected. `path.resolve(__dirname, '..')` yields the repo root (`/Users/.../hp41-calculator-emulator`), which is the parent of `hp41-gui/`.

Alternative considered (W8): copy/symlink the JSON into `hp41-gui/src/help_data.json` at build time. Rejected because it duplicates the source of truth and breaks D-25.16's "single canonical JSON" principle. The vite config update is the cleaner solution.

**Step 1 — Create `hp41-gui/src/help_data.ts`:**

(a) Imports and interface (mirror Rust HelpEntry shape from hp41-cli/src/help_data.rs lines 22-57):
```typescript
import functions from '../../docs/hp41cv-functions.json';

export interface HelpEntry {
    op_variant: string;
    display_name: string;
    category: string;
    status: 'implemented' | 'deferred-v3' | 'na';
    phase: string | null;
    key_path: string | null;
    description: string;
    divergences?: string[];
}
```

The `import functions from '../../docs/hp41cv-functions.json'` path is relative from `hp41-gui/src/help_data.ts` to the repo-root `docs/` directory. With the W8 vite.config.ts update, this resolves at build time. If the path resolution still fails, vite errors out — this matches D-25.17's hard-build-blocker semantics.

If TypeScript complains about the JSON import (no type declarations), add a `declare module '*.json'` shim in a `.d.ts` file (e.g. `hp41-gui/src/json-modules.d.ts`) OR use `import functions from '../../docs/hp41cv-functions.json' assert { type: 'json' };` syntax (vite supports both). Planner: pick whichever Vite version supports cleanly; the simpler `import` works in modern Vite.

(b) Lazy-init via vite import semantics — no OnceLock needed, the import IS the cache:
```typescript
export function helpEntries(): readonly HelpEntry[] {
    return functions as readonly HelpEntry[];
}
```

If runtime validation is desired, add a `validate()` helper that asserts each entry has the required fields — but this is optional; the JSON malformed-case is a build-time failure per D-25.17 / D-26.8.

(c) helpOverlayRows category-grouping helper (port from hp41-cli/src/help_data.rs lines 95-121):
```typescript
export interface HelpOverlayRow {
    key: string;
    op: string;
    desc: string;
    isHeader: boolean;
    category: string;
}

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
        rows.push({ key: '', op: '', desc: `=== ${cat} ===`, isHeader: true, category: cat });
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
```

(d) Filter helper for search:
```typescript
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
```

**Step 2 — Create `hp41-gui/src/HelpOverlay.tsx`:**

(e) Component implementation per PATTERNS.md §"hp41-gui/src/HelpOverlay.tsx" lines 574-598:

```typescript
import { useState, useEffect, useMemo } from 'react';
import { filterHelpEntries, type HelpEntry } from './help_data';

export type HelpOverlayProps = {
    open: boolean;
    onClose: () => void;
};

export function HelpOverlay({ open, onClose }: HelpOverlayProps): JSX.Element | null {
    const [query, setQuery] = useState('');
    const filtered = useMemo(() => filterHelpEntries(query), [query]);

    // Group filtered entries by category (preserve JSON declaration order)
    const grouped = useMemo(() => {
        const groups = new Map<string, HelpEntry[]>();
        for (const entry of filtered) {
            const arr = groups.get(entry.category);
            if (arr) arr.push(entry); else groups.set(entry.category, [entry]);
        }
        return Array.from(groups.entries());  // [[category, entries], ...] in insertion order
    }, [filtered]);

    // Esc-close: parent App.tsx already handles Esc precedence; this is a defense-in-depth.
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
                />
                <button className="help-overlay-close" onClick={onClose} aria-label="Close help overlay">×</button>
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
```

**Step 3 — In `App.css`, ADD (do not modify any existing rules):**

(f) Overlay styles per PATTERNS.md §"Overlay-card style pattern" lines 720-744 + adapted for full-cover layout:
```css
.help-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(20, 20, 20, 0.94);
  z-index: 60;  /* above toast (z-index 50), below Tauri devtools */
  display: flex;
  flex-direction: column;
  font-family: 'Courier New', Courier, monospace;
  color: #c8e6c9;
}

.help-overlay-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: #252525;
  border-bottom: 1px solid #3a3a3a;
}

.help-overlay-search {
  flex: 1;
  padding: 6px 10px;
  background: #111;
  border: 1px solid #3a3a3a;
  color: #c8e6c9;
  font-family: inherit;
  font-size: 14px;
  border-radius: 4px;
  margin-right: 12px;
}

.help-overlay-close {
  background: transparent;
  border: none;
  color: #c8e6c9;
  font-size: 22px;
  cursor: pointer;
  padding: 0 8px;
}

.help-overlay-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
  font-size: 12px;
}

.help-overlay-category-heading {
  margin: 12px 0 6px 0;
  color: #f5a423;  /* HP-41 orange — same family as SHIFT key gradient */
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  border-bottom: 1px solid #3a3a3a;
  padding-bottom: 4px;
}

.help-overlay-row {
  display: grid;
  grid-template-columns: 100px 80px 1fr;
  gap: 10px;
  padding: 3px 4px;
  border-radius: 2px;
}

.help-overlay-row:hover {
  background: rgba(200, 230, 201, 0.08);
}

.help-overlay-key {
  color: #f5a423;
}

.help-overlay-op {
  color: #c8e6c9;
  font-weight: bold;
}

.help-overlay-desc {
  color: #888;
}

.help-overlay-empty {
  padding: 20px;
  text-align: center;
  color: #888;
}
```

**Step 4 — Create `hp41-gui/src/HelpOverlay.test.tsx`:**

(g) Vitest tests:
```typescript
import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/react';
import { HelpOverlay } from './HelpOverlay';
import { helpEntries, helpOverlayRows, filterHelpEntries } from './help_data';
import sourceJson from '../../docs/hp41cv-functions.json';

describe('help_data', () => {
    it('helpEntries returns all entries from docs/hp41cv-functions.json', () => {
        expect(helpEntries().length).toBe((sourceJson as unknown[]).length);
        expect(helpEntries().length).toBeGreaterThanOrEqual(130);
    });

    it('filterHelpEntries with empty query returns only key_path != null entries', () => {
        const all = helpEntries();
        const nonNullCount = all.filter(e => e.key_path !== null).length;
        expect(filterHelpEntries('').length).toBe(nonNullCount);
    });

    it('filterHelpEntries narrows by display_name match', () => {
        const result = filterHelpEntries('STO');
        expect(result.length).toBeGreaterThan(0);
        for (const entry of result) {
            const matches = entry.display_name.toLowerCase().includes('sto') ||
                            entry.description.toLowerCase().includes('sto') ||
                            entry.category.toLowerCase().includes('sto');
            expect(matches).toBe(true);
        }
    });

    it('helpOverlayRows produces category headers in JSON declaration order', () => {
        const rows = helpOverlayRows();
        const headers = rows.filter(r => r.isHeader).map(r => r.category);
        // Each header should appear at most once
        expect(new Set(headers).size).toBe(headers.length);
    });

    it('helpOverlayRows excludes null-key_path entries from rendered rows (D-26.8)', () => {
        const rows = helpOverlayRows();
        const dataRows = rows.filter(r => !r.isHeader);
        // Every data row must have a non-empty key (which derived from non-null key_path)
        for (const row of dataRows) {
            expect(row.key).not.toBe('');
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
    });

    it('search input filters the rendered rows', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const initialRows = container.querySelectorAll('.help-overlay-row').length;
        const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
        fireEvent.change(searchInput, { target: { value: 'sin' } });
        const filteredRows = container.querySelectorAll('.help-overlay-row').length;
        expect(filteredRows).toBeLessThan(initialRows);
        expect(filteredRows).toBeGreaterThan(0);
    });

    it('Esc key calls onClose', () => {
        let closed = false;
        render(<HelpOverlay open={true} onClose={() => { closed = true; }} />);
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(closed).toBe(true);
    });

    it('close button calls onClose', () => {
        let closed = false;
        const { container } = render(<HelpOverlay open={true} onClose={() => { closed = true; }} />);
        const closeButton = container.querySelector('.help-overlay-close') as HTMLButtonElement;
        fireEvent.click(closeButton);
        expect(closed).toBe(true);
    });
});
```
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && pnpm --filter hp41-gui-frontend exec vite build 2>&1 | tail -20 && pnpm --filter hp41-gui-frontend test --run HelpOverlay help_data 2>&1 | tail -30</automated>
  </verify>
  <acceptance_criteria>
    - File `hp41-gui/vite.config.ts` updated with `server.fs.allow` extension; `grep -c "fs:" hp41-gui/vite.config.ts` returns at least 1; `grep -c "path.resolve" hp41-gui/vite.config.ts` returns at least 1 (W8)
    - `pnpm --filter hp41-gui-frontend exec vite build` succeeds (W8: JSON import resolves) — OR equivalently `just gui-build` succeeds
    - File `hp41-gui/src/help_data.ts` exists; exports `HelpEntry` interface, `helpEntries()`, `helpOverlayRows()`, `filterHelpEntries()`
    - File `hp41-gui/src/HelpOverlay.tsx` exists; exports `HelpOverlay` (named + default)
    - `grep -c "import functions from '../../docs/hp41cv-functions.json'" hp41-gui/src/help_data.ts` returns at least 1
    - The JSON entry-count parity test passes: `helpEntries().length === (sourceJson as unknown[]).length` AND `>= 130`
    - The null-key_path filter test passes: every rendered row in `helpOverlayRows()` has non-empty `key`
    - The search filter test passes: typing "sin" reduces rendered row count
    - The Esc-close test passes
    - The .help-overlay CSS rule is added to App.css; existing .display, .toast, .prgm-panel rules are UNCHANGED
    - `pnpm --filter hp41-gui-frontend exec tsc --noEmit` returns no type errors
    - `pnpm --filter hp41-gui-frontend test --run HelpOverlay` passes all tests
  </acceptance_criteria>
  <done>
    vite.config.ts updated with server.fs.allow for repo-root JSON access (W8); vite build succeeds; HelpOverlay component + help_data port from Rust + JSON entry-count parity test + search/category/Esc tests all green; CSS overlay styles added without modifying existing rules.
  </done>
</task>

<task type="execute">
  <name>Task 2: Wire HelpOverlay into App.tsx with '?' keyboard handler + Esc precedence + MAP table swap (D-26.10) + userKeymap prop passing (W7 file-region split with Plan 26-02)</name>
  <files>hp41-gui/src/App.tsx</files>
  <read_first>
    - hp41-gui/src/App.tsx (the modified version after Plan 26-01 + Plan 26-02: handleKey listener around lines 210-227, MAP table around lines 82-96, Esc handler in handleKey, pendingInput state and Esc-clears-pendingInput logic from Plan 26-01, `<Display14Seg text={displayText} />` from Plan 26-02)
    - hp41-gui/src/Keyboard.tsx (current KeyboardProps shape; verify the shiftActive + alphaActive props pattern — Task 3 extends with userActive + userKeymap)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/App.tsx (extend) — Physical-keyboard MAP table pattern" lines 364-376 (D-26.10 swap)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/App.tsx (extend) — Tab/Esc one-shot consumption pattern" lines 380-403 (Esc precedence extension)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.10 lines 73 (full remap spec)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.9 lines 71 (USER-mode relabel data flow: from CalcStateView.user_keymap → App.tsx → Keyboard prop)
  </read_first>
  <action>
**W7 file-region split (vs Plan 26-02):**

This task touches the following App.tsx regions:
1. New imports (top of file)
2. New `helpOpen` state in the hook block
3. MAP table mutation (`'p'` → `'prgm_mode'`, new `'P'` → `'prx'`)
4. `handleKey` extension (Esc precedence + `?` open)
5. New `<HelpOverlay open={...} onClose={...} />` JSX appended near the end of the root JSX tree
6. `<Keyboard userKeymap={...} userActive={...} />` prop extension

This task does NOT touch:
- The `displayText` derivation (added by Plan 26-01 Task 3 step i; preserved unchanged)
- The `<Display14Seg text={displayText} />` JSX inside `<div className="display">` (added by Plan 26-02 Task 2)

Cross-plan verification: after both Wave 2 plans land, `grep -c "<Display14Seg\\|<HelpOverlay open=" hp41-gui/src/App.tsx` returns at least 2 — proof that both plans' regions coexist.

In `App.tsx`:

(a) Add the import: `import HelpOverlay from './HelpOverlay';`

(b) Add state: `const [helpOpen, setHelpOpen] = useState(false);` in the same hook block as `shiftActive` / `pendingInput` / `toast`.

(c) Update the `MAP` table per D-26.10:
- Change `'p': 'prx'` to `'p': 'prgm_mode'` (toggle program mode)
- Add new entry `'P': 'prx'` (uppercase, dispatched when SHIFT modifier produces an uppercase letter via the existing case-detection convention)

The existing case-detection convention in App.tsx's MAP system handles uppercase vs lowercase via the raw `e.key` value — no separate SHIFT-detect needed since browsers report `key='P'` when shift+p is pressed.

(d) Extend `handleKey` (the physical-keyboard listener) per PATTERNS.md §"Tab/Esc one-shot consumption pattern" extension:

```typescript
const handleKey = useCallback((e: KeyboardEvent) => {
    if (e.repeat) return;

    // '?' key opens the help overlay (UNLESS already open or in ALPHA mode where ? could be input)
    const alphaOn = calcState?.annunciators.alpha ?? false;
    if (e.key === '?' && !alphaOn && !helpOpen) {
        e.preventDefault();
        setHelpOpen(true);
        return;
    }

    // Esc precedence (D-26.5 + D-26.8 + Plan 26-01 Esc handler):
    //   1. Help overlay first (closes on Esc)
    //   2. Modal pendingInput second (closes on Esc, clears shiftActive)
    //   3. shiftActive last (clears on Esc when nothing else open)
    if (e.key === 'Escape') {
        if (helpOpen) {
            e.preventDefault();
            setHelpOpen(false);
            return;
        }
        if (pendingInput) {
            e.preventDefault();
            setPendingInput(null);
            setShiftActive(false);
            return;
        }
        setShiftActive(false);
        return;
    }

    if (e.key === 'Tab') { e.preventDefault(); setShiftActive(prev => !prev); return; }
    if (busyRef.current) return;

    // If a modal is open, route through handleModalKey (Plan 26-01 wiring stays unchanged)
    if (pendingInput) {
        // ... (existing Plan 26-01 routing)
        return;
    }

    const keyId = resolveKeyId(e, calcState);
    if (keyId === null) return;
    e.preventDefault();
    dispatchKeyId(keyId);
}, [calcState, dispatchKeyId, helpOpen, pendingInput]);
```

(e) Render `<HelpOverlay open={helpOpen} onClose={() => setHelpOpen(false)} />` near the end of the App's JSX tree (after the toast and other overlays — z-index 60 puts it above toast's z-index 50). The component returns null when `open=false`, so it's safe to always include in the tree.

(f) Pass userKeymap and userActive props to `<Keyboard />`:
```typescript
<Keyboard
    onKey={handleClick}
    busyRef={busyRef}
    shiftActive={shiftActive}
    alphaActive={calcState?.annunciators.alpha ?? false}
    userActive={calcState?.annunciators.user ?? false}
    userKeymap={calcState?.user_keymap ?? []}
/>
```

The `calcState.user_keymap` and `calcState.annunciators.user` are projections from CalcStateView added in Plan 26-01 Task 2. The TS CalcStateView interface mirror was extended in Plan 26-01 Task 2 step f (BLOCKER B5), so `calcState?.user_keymap` is type-safe.

(g) Verify the `?` keystroke handling does not conflict with any existing key id. Search `MAP` and the existing `?`-handling code (the `?` overlay narrowing was a CLI Phase 25 fix; in the GUI, `?` was previously unbound). If `?` is currently in MAP, remove it.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && pnpm --filter hp41-gui-frontend exec tsc --noEmit 2>&1 | tail -10 && grep -c "helpOpen\|HelpOverlay\|Display14Seg" hp41-gui/src/App.tsx</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "import HelpOverlay" hp41-gui/src/App.tsx` returns at least 1
    - `grep -c "setHelpOpen" hp41-gui/src/App.tsx` returns at least 3 (state setter declared + open on '?' + close on Esc/click)
    - `grep -c "<HelpOverlay open=" hp41-gui/src/App.tsx` returns at least 1
    - **W7 cross-plan verification**: `grep -E '<Display14Seg|<HelpOverlay open=' hp41-gui/src/App.tsx | wc -l` returns at least 2 — proof that Plan 26-02's Display14Seg AND Plan 26-03's HelpOverlay coexist in App.tsx
    - `grep "'p':" hp41-gui/src/App.tsx | grep -c "'prgm_mode'"` returns at least 1
    - `grep "'P':" hp41-gui/src/App.tsx | grep -c "'prx'"` returns at least 1
    - `grep -c "userKeymap" hp41-gui/src/App.tsx` returns at least 1 (passed to Keyboard)
    - `grep -c "userActive" hp41-gui/src/App.tsx` returns at least 1 (passed to Keyboard)
    - The Esc precedence: helpOpen → pendingInput → shiftActive is verifiable by reading the handleKey function (3 distinct Esc branches in order)
    - `pnpm --filter hp41-gui-frontend exec tsc --noEmit` returns no type errors
    - `just gui-check` passes
  </acceptance_criteria>
  <done>
    HelpOverlay wired into App.tsx with '?' keyboard handler; Esc precedence (help → modal → shift) preserved; MAP table swapped per D-26.10; userKeymap + userActive props passed to Keyboard for Task 3 to consume; W7 file-region split honored (Display14Seg + HelpOverlay coexist); TypeScript clean.
  </done>
</task>

<task type="execute">
  <name>Task 3: Keyboard.tsx KeyDef.keyCode field with W9 hardcoded literals from CLI canonical mapping + USER-mode render-time relabel + XSS-safety test + Vitest sentinel-key parity test</name>
  <files>hp41-gui/src/Keyboard.tsx, hp41-gui/src/Keyboard.test.tsx</files>
  <read_first>
    - hp41-gui/src/Keyboard.tsx (full 289 lines — KeyDef type lines 23-36, KEY_DEFS table lines 50-94, KeyboardProps lines 136-141, label rendering lines 259-269)
    - hp41-cli/src/keys.rs (lines 405-479 — keycode_to_hp41_code canonical mapping; W9 SOURCE OF TRUTH for the literal keyCode values per the table in the plan's <interfaces> section)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/Keyboard.tsx (extend) — KeyDef three-label model pattern" lines 412-433 (KeyDef.keyCode extension)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/Keyboard.tsx (extend) — USER-mode relabel pattern" lines 446-470
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.9 lines 71 (per-key text relabel spec)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md "Claude's Discretion" USER mode keyCode mapping in KeyDef lines 113-114 (the discretion was "compute or hardcode" — W9 RESOLVES to hardcode because the GUI grid does not align with HP-41 hardware indexing)
  </read_first>
  <action>
In `Keyboard.tsx`:

(a) Extend `KeyDef` type (lines 23-36) with optional `keyCode` field:
```typescript
export type KeyDef = {
    id: string;
    label: string;
    shifted?: { id: string; label: string };
    alphaChar?: string;
    row: number;
    col: number;
    colSpan?: number;
    variant?: 'top' | 'shift' | 'enter';
    keyCode?: number;  // Phase 26 D-26.9 / W9: HARDCODED literal from hp41-cli canonical mapping
};
```

(b) **W9 RESOLUTION — hardcode keyCode literals from hp41-cli/src/keys.rs::keycode_to_hp41_code**:

The original CONTEXT D-26.9 "Claude's Discretion" allowed either hardcoding OR computing `row*10+col`. The W9 audit established that computing `row*10+col` is INCORRECT because:
- GUI Keyboard.tsx uses 0-indexed cols (0..4); CLI HP-41 hardware mapping uses 1-indexed rows+cols
- GUI row numbering does NOT match HP-41 hardware row numbering (e.g. SIN at GUI row 2 col 2 = hardware row 2 col 5 = code 25, but `2*10+2=22` which is the STO code in HP-41 hardware)
- The GUI's 5-col landscape layout DIFFERS from the HP-41's 5-col layout in the bottom 4 rows (operators in col 0 are HP-41 col 5 originally)

Therefore: REMOVE the prior plan's `hp41KeyCode(row, col)` helper. Hardcode `keyCode: <literal>` on each KEY_DEFS entry per the canonical mapping table in <interfaces>. Implementation: edit MAIN_GRID and TOP_ROW entries to include `keyCode` literals. Example transformation:

```typescript
// BEFORE (Phase 26 first-draft):
{ id: 'sin', label: 'SIN', shifted: { id: 'asin', label: 'SIN⁻¹' }, alphaChar: 'H', row: 2, col: 2 },

// AFTER (W9 fix):
{ id: 'sin', label: 'SIN', shifted: { id: 'asin', label: 'SIN⁻¹' }, alphaChar: 'H', row: 2, col: 2, keyCode: 25 },
```

Apply the full canonical mapping from the <interfaces> table. Entries without a CLI-canonical mapping (e.g. `sigma_plus`, `recip`, `sqrt`, `log`, `ln`, `rdn`, `xge_y`, `clx_or_a`, mode-toggle keys) either get the HP-41-derived code from CLI's `key_to_op` semantics OR get NO keyCode (variant 'top' and variant 'shift' and id `''` already get none). For ambiguous keys, planner falls back to "no keyCode" rather than assigning incorrect ones — better to silently skip USER relabel than to relabel the wrong key.

Sentinel keys with confirmed CLI-canonical codes (must appear in KEY_DEFS):
- `sin` -> 25
- `cos` -> 34
- `tan` -> 35
- `sto_prompt` -> 22
- `rcl_prompt` -> 23
- `xeq_prompt` -> 21
- `enter` -> 84
- `e` (EEX) -> 83
- `0` -> 81, `.` -> 82, `1` -> 71, ..., `9` -> 53 (digits per the table)
- `plus` -> 74, `minus` -> 64, `mul` -> 54, `div` -> 45

(c) Extend `KeyboardProps` (lines 136-141):
```typescript
export type KeyboardProps = {
    onKey: (key: KeyDef) => void;
    busyRef: React.MutableRefObject<boolean>;
    shiftActive: boolean;
    alphaActive: boolean;
    userActive?: boolean;                              // Phase 26 D-26.9
    userKeymap?: ReadonlyArray<[number, string]>;      // Phase 26 D-26.9
};
```

(d) Update the Keyboard function signature to destructure userActive and userKeymap (with defaults `userActive = false, userKeymap = []`).

(e) Update the label-rendering block (lines 259-269) to apply USER-mode relabel:
```typescript
const userLabel = (userActive && key.keyCode != null)
    ? userKeymap.find(([code]) => code === key.keyCode)?.[1]
    : null;

// ... in JSX ...
<text
    x={x + w / 2}
    y={y + h / 2 + 5}
    textAnchor="middle"
    fill={labelColor}
    fontSize={key.variant === 'enter' ? 13 : 14}
    fontWeight="bold"
>
    {userLabel ?? key.label}
</text>
```

CRITICAL: `{userLabel ?? key.label}` is rendered as a React text node — React's default rendering escapes content automatically. A malicious ASN label like `<script>alert(1)</script>` renders as the literal string "<script>alert(1)</script>" inside the SVG text element, NOT as an injected script tag. Task 3 step (g) test asserts this directly.

(f) Truncate long labels to fit the keycap. The HP-41 ASN label is up to 6 chars (per the HP-41C ALPHA pack convention from Phase 23); SVG text rendering will overflow if longer. Apply `userLabel.slice(0, 7)` defensively (allow up to 7 chars to fit comfortably; longer labels are visually truncated at render time by SVG clipping, but `.slice(0, 7)` makes the truncation explicit). Document this in a code comment.

In `hp41-gui/src/Keyboard.test.tsx` (NEW or extend existing if present):

(g) Vitest tests:
```typescript
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { Keyboard, KEY_DEFS } from './Keyboard';
import { useRef } from 'react';

function TestHarness({ userActive, userKeymap }: { userActive: boolean; userKeymap: Array<[number, string]> }) {
    const busyRef = useRef(false);
    return (
        <Keyboard
            onKey={() => {}}
            busyRef={busyRef}
            shiftActive={false}
            alphaActive={false}
            userActive={userActive}
            userKeymap={userKeymap}
        />
    );
}

describe('Keyboard USER-mode relabel', () => {
    // W9 sentinel-parity test (must match hp41-cli/src/keys.rs::keycode_to_hp41_code):
    it('W9: KEY_DEFS sentinel keyCodes match hp41-cli canonical mapping', () => {
        const sentinels: Array<[string, number]> = [
            ['sin', 25],
            ['cos', 34],
            ['tan', 35],
            ['sto_prompt', 22],
            ['rcl_prompt', 23],
            ['xeq_prompt', 21],
            ['enter', 84],
            ['e', 83],
            ['7', 51],
            ['0', 81],
            ['plus', 74],
            ['minus', 64],
            ['mul', 54],
            ['div', 45],
        ];
        for (const [id, expectedCode] of sentinels) {
            const key = KEY_DEFS.find(k => k.id === id);
            expect(key, `KEY_DEFS entry '${id}' must exist`).toBeDefined();
            expect(key!.keyCode, `'${id}'.keyCode must be ${expectedCode} (hp41-cli canonical)`).toBe(expectedCode);
        }
    });

    it('top-row and shift keys have NO keyCode (variant exception)', () => {
        for (const key of KEY_DEFS) {
            if (key.variant === 'top' || key.variant === 'shift' || key.id === '') {
                expect(key.keyCode, `'${key.id}'.keyCode must be undefined`).toBeUndefined();
            }
        }
    });

    it('renders primary label when userActive=false', () => {
        const { container } = render(<TestHarness userActive={false} userKeymap={[[22, 'TEST']]} />);
        const texts = Array.from(container.querySelectorAll('text')).map(t => t.textContent);
        expect(texts).not.toContain('TEST');
    });

    it('renders ASN label when userActive=true and keyCode matches (D-26.9)', () => {
        // Use sto_prompt key (keyCode 22 per W9 canonical mapping)
        const { container } = render(<TestHarness userActive={true} userKeymap={[[22, 'MYPRG']]} />);
        const texts = Array.from(container.querySelectorAll('text')).map(t => t.textContent);
        expect(texts).toContain('MYPRG');
    });

    it('XSS-safety: ASN label "<script>alert(1)</script>" renders as literal text (T-26-03-04)', () => {
        // Pick a confirmed-wired key (sin keyCode 25)
        const code = 25;
        const malicious = '<script>alert(1)</script>';
        const { container } = render(<TestHarness userActive={true} userKeymap={[[code, malicious]]} />);
        // No <script> element should appear in the rendered DOM
        expect(container.querySelector('script')).toBeNull();
        // The literal-truncated text should appear (truncated at 7 chars per defensive slice)
        const texts = Array.from(container.querySelectorAll('text')).map(t => t.textContent);
        expect(texts.some(t => t?.includes('<script') || t?.includes('<scrip'))).toBe(true);
    });

    it('USER mode without an ASN entry for a key falls back to the primary label', () => {
        const { container } = render(<TestHarness userActive={true} userKeymap={[]} />);
        const texts = Array.from(container.querySelectorAll('text')).map(t => t.textContent);
        // The primary label of the first wired key should still be present
        const firstWired = KEY_DEFS.find(k => k.id !== '' && k.label !== '');
        expect(texts).toContain(firstWired!.label);
    });
});
```

The XSS-safety test is the security mitigation for T-26-03-04 in the threat model. The W9 sentinel-parity test catches any drift between Keyboard.tsx KEY_DEFS keyCode literals and the hp41-cli canonical mapping; if the CLI changes its mapping, this test fails before users see incorrect USER relabels.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && pnpm --filter hp41-gui-frontend test --run Keyboard 2>&1 | tail -30</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "keyCode?: number" hp41-gui/src/Keyboard.tsx` returns at least 1 (KeyDef extension)
    - `grep -c "userActive" hp41-gui/src/Keyboard.tsx` returns at least 2 (KeyboardProps + render conditional)
    - `grep -c "userKeymap" hp41-gui/src/Keyboard.tsx` returns at least 2 (KeyboardProps + lookup)
    - `grep -cE "keyCode: (21|22|23|25|34|35|45|54|64|74|81|82|83|84|51|52|53|61|62|63|71|72|73)" hp41-gui/src/Keyboard.tsx` returns at least 14 (W9 hardcoded literals — sentinel keys verified)
    - **`grep -c "hp41KeyCode" hp41-gui/src/Keyboard.tsx` returns 0** (W9: prior draft's row*10+col helper REMOVED)
    - The W9 sentinel-parity test passes (`sin`=25, `cos`=34, `tan`=35, `sto_prompt`=22, `rcl_prompt`=23, `xeq_prompt`=21, `enter`=84, `e`=83, digits per the table)
    - The top-row-and-shift-no-keyCode test passes (variant exceptions handled correctly)
    - The `renders ASN label when userActive=true and keyCode matches` test passes
    - The XSS-safety test passes — no <script> element in DOM, literal text rendered
    - The fallback-to-primary-label test passes (USER mode + no matching ASN = primary label)
    - `pnpm --filter hp41-gui-frontend exec tsc --noEmit` returns no type errors
    - `pnpm --filter hp41-gui-frontend test --run` passes the FULL Vitest suite (Plan 26-01 + 26-02 + 26-03 combined)
    - `just gui-ci` passes
  </acceptance_criteria>
  <done>
    KeyDef.keyCode field added with W9 hardcoded literals from hp41-cli canonical mapping (NOT computed row*10+col which would mismap due to GUI 0-idx cols and divergent grid layout); the W9 sentinel-parity test catches drift between GUI keyCodes and hp41-cli mapping; KeyboardProps extended with userActive + userKeymap; render-time USER-mode relabel logic in place; XSS-safety verified via test asserting React's default text-escape behavior; full Vitest suite green; gui-ci passes.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| User-supplied ASN label → SVG text rendering | The user can ASN any string label via Op::Asn { name, key_code }; that string flows through CalcStateView.user_keymap → App.tsx prop → Keyboard.tsx text node |
| docs/hp41cv-functions.json (build-time) → HelpOverlay | vite JSON-import resolves at build time; runtime data is the build-frozen array |
| '?' keystroke (physical keyboard) → setHelpOpen | Standard React keyboard event; no escape needed |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-26-03-01 | Tampering | docs/hp41cv-functions.json modified between build and runtime | accept | vite resolves the import at build time and bakes the array into the bundle. Runtime modification is impossible. A malicious build-time replacement is a higher-trust scenario (developer's own filesystem) — out of scope per CONTEXT (no network endpoints, no auth). |
| T-26-03-02 | Information Disclosure | HelpOverlay shows function descriptions including divergences | accept | All entries in docs/hp41cv-functions.json are intentionally public-documentation content. No secrets. |
| T-26-03-03 | Denial of Service | Search input with regex special chars / very long string | mitigate | filterHelpEntries uses `.includes()` (NOT `RegExp`), so no ReDoS. Query length is unbounded but capped by browser `<input>` default behavior; large queries are O(n) over ~130 entries — fast enough. |
| T-26-03-04 | Elevation of Privilege (XSS) | Malicious ASN label injected via Op::Asn renders into SVG text | mitigate | React's default text-node rendering escapes content. The `{userLabel ?? key.label}` JSX expression renders as a text node (not innerHTML) — `<script>` becomes literal text. Verified by Task 3 step g `XSS-safety: ASN label "<script>alert(1)</script>" renders as literal text` test. Also `.slice(0, 7)` defensive truncation reduces blast radius for very long labels. |
| T-26-03-05 | Spoofing | '?' keyhandler accidentally firing inside ALPHA mode where '?' is a valid input | mitigate | Task 2 step (d) explicitly checks `!alphaOn` before opening the help overlay; the `?` keystroke flows to ALPHA-mode input when alphaOn=true. Identical guard pattern to CLI Phase 25's `?` overlay. |
| T-26-03-06 | Tampering | vite.config.ts fs.allow widening exposes more files to dev-server requests | accept | The widening (W8) is repo-root scope. dev-server is loopback (127.0.0.1:5173) by default; production builds bake the JSON into the bundle and serve no files. Single-developer-machine threat model per project boundary. |
| T-26-03-07 | Spoofing | GUI USER relabel applied to wrong key due to keyCode drift | mitigate | W9 resolution: keyCode is hardcoded from hp41-cli canonical mapping (NOT computed); the W9 sentinel-parity test in Keyboard.test.tsx catches any drift between Keyboard.tsx literals and hp41-cli/src/keys.rs::keycode_to_hp41_code. CI test failure precedes user-visible mismapping. |
</threat_model>

<verification>
1. `cd hp41-gui && pnpm --filter hp41-gui-frontend exec tsc --noEmit` — no type errors
2. `cd hp41-gui && pnpm --filter hp41-gui-frontend test --run` — full Vitest suite green (Plan 26-01 + 26-02 + 26-03)
3. `cd hp41-gui && pnpm --filter hp41-gui-frontend exec vite build` — production build succeeds (W8: JSON import resolves)
4. `just gui-ci` — full GUI CI pipeline green
5. `grep -c "import functions from '../../docs/hp41cv-functions.json'" hp41-gui/src/help_data.ts` returns at least 1 (D-25.16 vite import)
6. SC-4 sanity check: `grep -rEn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns ZERO matches (Phase 26 only touches frontend; SC-4 invariant unaffected)
7. **W7 cross-plan verification**: `grep -E '<Display14Seg|<HelpOverlay open=' hp41-gui/src/App.tsx | wc -l` returns at least 2
8. Manual smoke: `just gui-dev` boots; press `?` → overlay opens with categorized + searchable entries; type "sin" → list narrows; press Esc → overlay closes; ASN a key (`f-ASN "TEST" 22` or via the ASN modal flow from Plan 26-01); toggle USER mode (`f-USER`); confirm key 22 (sto_prompt) now shows "TEST" instead of "STO"; press 'p' on physical keyboard → PRGM annunciator toggles
</verification>

<success_criteria>
- `?` keystroke opens HelpOverlay (full-cover, semi-transparent, search input + category-grouped entries from docs/hp41cv-functions.json)
- vite.config.ts updated with server.fs.allow for repo-root access (W8); `vite build` succeeds
- Search input filters across display_name + description + category; null-key_path entries excluded per D-26.8
- Esc closes the overlay; Esc precedence: help → modal → shift (preserves Plan 26-01 Esc semantics)
- HelpOverlay JSON entry count exactly matches the source file (drift-catch test passes)
- USER mode (`calcState.annunciators.user === true`) + an ASN entry for a key's keyCode → key cap renders the ASN'd label INSTEAD of primary label
- USER mode without matching ASN entry → primary label renders unchanged (fallback path)
- XSS-safety: malicious ASN label like `<script>alert(1)</script>` renders as literal text; no script element injected (T-26-03-04 mitigated and tested)
- Physical-keyboard `'p'` → `prgm_mode` (annunciators.prgm flips); SHIFT+'P' → `prx` per D-26.10
- **W9: KEY_DEFS keyCode values are hardcoded literals from hp41-cli/src/keys.rs canonical mapping** (sentinel parity test passes); NO `hp41KeyCode(row, col)` helper anywhere in Keyboard.tsx
- **W7 cross-plan: both Display14Seg and HelpOverlay coexist in App.tsx**
- All Vitest tests pass; `just gui-ci` green
- SC-4 invariant intact (no calculator/math logic in `hp41-gui/src-tauri/`)
</success_criteria>

<output>
After completion, create `.planning/phases/26-gui-integration-and-polish/26-03-SUMMARY.md` documenting:
- The total HelpOverlay entry count (from `helpEntries().length`) and the count after null-key_path filter
- The W9 keyCode strategy (hardcoded literals from hp41-cli canonical mapping, NOT computed); the sentinel-parity test as the drift-catch
- The W8 vite.config.ts update (server.fs.allow extension); confirmation that `vite build` succeeds
- The W7 file-region split outcome (post-merge regex verification that both Plan 26-02 and Plan 26-03 regions coexist)
- Cross-reference to CLAUDE.md update needed for the v2.2 GUI section: new `?` overlay shortcut, USER-mode relabel mechanism, `'p'` MAP swap (per ROADMAP's "Document the new envelope in CLAUDE.md" cross-cutting constraint — but the CalcStateView envelope itself is documented in Plan 26-01's SUMMARY)
- Any visual issues discovered during manual `just gui-dev` smoke (label truncation, overlay z-index conflicts) — and whether they were fixed in-cycle or deferred to a follow-up
- Confirmation that the .display CSS rule (Plan 26-02 D-26.7 contract) is still byte-identical AFTER this plan's App.css additions
</output>
</content>
</invoke>