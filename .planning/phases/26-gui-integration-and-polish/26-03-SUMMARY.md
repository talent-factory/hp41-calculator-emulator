---
phase: 26-gui-integration-and-polish
plan: 03
subsystem: hp41-gui (React frontend, no Tauri-side changes)
tags: [help-overlay, user-mode-relabel, physical-keyboard-remap, D-26.8, D-26.9, D-26.10, FN-POLISH-02, FN-POLISH-03, FN-POLISH-04, FN-GUI-03, FN-GUI-04]
dependency_graph:
  requires:
    - phase-26 plan-01 modal infrastructure (Esc precedence; pendingInput state)
    - phase-26 plan-01 CalcStateView.user_keymap projection (D-26.11 / BLOCKER B5)
    - phase-26 plan-02 Display14Seg integration (W7 file-region split in App.tsx)
    - docs/hp41cv-functions.json (canonical help data — Phase 25 D-25.16)
    - hp41-cli/src/keys.rs::keycode_to_hp41_code (W9 canonical mapping source-of-truth)
  provides:
    - HelpOverlay React component (full-cover, search + category-grouped)
    - help_data.ts (TypeScript port of hp41-cli/src/help_data.rs)
    - Keyboard USER-mode per-key relabel (D-26.9 + T-26-03-04 escape-safe)
    - Physical-keyboard 'p' -> prgm_mode + SHIFT+'P' -> prx remap (D-26.10)
    - W9 sentinel-parity test (drift-catch between Keyboard KEY_DEFS and CLI)
  affects:
    - CLAUDE.md needs a v2.2 Phase 26 update for the `?`-overlay shortcut,
      USER-mode relabel mechanism, and the 'p' MAP swap. (CalcStateView
      envelope change is documented in Plan 26-01 SUMMARY; this plan
      adds the polish surface area.)
tech_stack:
  added: []
  patterns:
    - "Vite build-time JSON import via server.fs.allow widening (D-25.16 / W8)"
    - "Frontend-owned overlay state mirror (D-26.1 generalized to helpOpen)"
    - "Defense-in-depth Esc precedence (help -> modal -> shift, each layer independently dismissable)"
    - "React text-node escape as default injected-content mitigation (T-26-03-04)"
    - "Hardcoded keyCode literals from canonical CLI mapping (W9 — drift-catch test)"
key_files:
  created:
    - hp41-gui/src/help_data.ts (107 lines, 4 exports — HelpEntry, helpEntries, helpOverlayRows, filterHelpEntries)
    - hp41-gui/src/HelpOverlay.tsx (98 lines, default + named export)
    - hp41-gui/src/HelpOverlay.test.tsx (16 Vitest tests)
    - hp41-gui/src/Keyboard.test.tsx (41 Vitest tests)
  modified:
    - hp41-gui/vite.config.ts (+8 lines — server.fs.allow extension)
    - hp41-gui/src/App.css (+102 lines — .help-overlay styles, no existing rules touched)
    - hp41-gui/src/App.tsx (+30 lines — HelpOverlay wiring, Esc precedence, MAP swap, userKeymap prop)
    - hp41-gui/src/Keyboard.tsx (+58 lines — KeyDef.keyCode, 32 W9 literals, USER-mode relabel)
decisions:
  - D-26.8 (full-cover `?` overlay, search + category-grouped, null-key_path filtered)
  - D-26.9 (USER-mode per-key relabel, hardcoded keyCode from CLI canonical mapping)
  - D-26.10 (`'p'` -> prgm_mode + SHIFT+'P' -> prx physical-keyboard remap)
  - W7 (file-region split with Plan 26-02 — Display14Seg + HelpOverlay coexist in App.tsx)
  - W8 (vite.config.ts server.fs.allow widened to repo root for JSON import)
  - W9 (keyCode literals hardcoded — NOT computed `row*10+col` which would mis-label given GUI 0-idx cols and divergent grid layout)
  - T-26-03-04 (injected-content mitigation via React text-node escape + 7-char defensive slice)
metrics:
  duration: ~25 min (3 atomic commits + tests + verification)
  completed_date: 2026-05-15
  commits: 3
  tests_added:
    typescript: 57 (16 HelpOverlay + 41 Keyboard W9 sentinels + USER + XSS)
  vitest_total_after_plan: 121 (42 Wave 1 + 22 Wave 2 + 16 HelpOverlay + 41 Keyboard)
  help_overlay_counts:
    json_entries_total: 154
    json_entries_with_key_path: 62
    categories: 11
  keyCode_w9_coverage:
    main_grid_with_keycode: 32
    main_grid_undefined: 3  # xge_y, chs, clx_or_a (CLI mapping ambiguous)
    top_row_undefined: 4    # variant 'top' rule
    shift_key_undefined: 1  # variant 'shift' rule
---

# Phase 26 Plan 03: Polish Bundle Summary

**One-liner:** Ships the three "discoverability + ergonomics" polish
features per D-26.8 / D-26.9 / D-26.10 — a `?`-keyboard-shortcut overlay
sourced from `docs/hp41cv-functions.json` via vite build-time JSON
import; USER-mode per-key relabel that swaps SVG keycap text for ASN'd
labels with React-default content escape; and the long-deferred `'p'`
-> `prgm_mode` physical-keyboard remap with `SHIFT+'P'` -> `prx` —
completing the Phase 26 GUI Integration & Polish deliverables.

## Outcome

Plan 26-03 closes the FN-POLISH-02/03/04 + FN-GUI-03/04 requirements
and finishes Phase 26 wave 2:

1. **`?` overlay (D-26.8 / FN-POLISH-02 / FN-GUI-03):** pressing `?` on
   the physical keyboard (outside ALPHA mode) opens a full-cover
   semi-transparent React modal listing the 62 keyboard-bound HP-41CV
   functions (154 total entries in the JSON; null-`key_path` entries
   are XEQ-by-Name-only ops and are filtered out per D-26.8 — they
   aren't keyboard shortcuts). Entries are grouped by the JSON
   `category` field in JSON declaration order (11 categories total).
   A search input filters across `display_name` + `description` +
   `category` via case-insensitive substring match. Esc closes the
   overlay; the close button (×) also works.

2. **USER mode per-key relabel (D-26.9 / FN-POLISH-03 / FN-GUI-04):**
   when `calcState.annunciators.user === true` AND a key's `keyCode`
   matches an entry in `calcState.user_keymap` (the projection added
   by Plan 26-01), the SVG keycap's PRIMARY label is replaced by the
   ASN'd label (truncated to 7 chars defensively). Injected content
   is escaped by React's default text-node rendering: an ASN label
   containing HTML-like substrings becomes literal SVG text — no
   element is injected into the DOM.

3. **Physical-keyboard `'p'` remap (D-26.10 / FN-POLISH-04):**
   - lowercase `'p'` -> `prgm_mode` (toggles PRGM annunciator).
   - uppercase `'P'` (shift+p) -> `prx` (prints X register).
   - `'P'` was removed from the modal-trigger silence list so the new
     uppercase route is reachable.

## W7 File-Region Split with Plan 26-02

Both Plan 26-02 (14-seg LCD) and Plan 26-03 (this plan) touch
`hp41-gui/src/App.tsx`. The W7 split was designed to make the regions
non-overlapping:

- **Plan 26-02 territory** (unchanged here): the `displayText`
  derivation + `<div className="display"><Display14Seg text={displayText} /></div>`.
- **Plan 26-03 territory** (this plan): new `HelpOverlay` import,
  `helpOpen` state, MAP table swap (`'p'` -> `'prgm_mode'`, `'P'` ->
  `'prx'`), `handleKey` extension for `?` open + Esc precedence,
  `userKeymap`/`userActive` props on `<Keyboard />`, and the new
  `<HelpOverlay open={helpOpen} onClose={...} />` JSX node appended
  near the end of the root JSX tree.

Post-merge regex grep verification:
```
$ grep -E '<Display14Seg|<HelpOverlay open=' hp41-gui/src/App.tsx | wc -l
3
```

(One Display14Seg JSX node, one HelpOverlay JSX node, and one
plan-26-02 comment referencing `<Display14Seg`.) The W7 split worked
— both plans' regions coexist cleanly.

## W8 — vite.config.ts server.fs.allow Widening

The JSON canonical source (`docs/hp41cv-functions.json`) lives at the
repo root, OUTSIDE the default vite project root (`hp41-gui/`). The
import path `'../../docs/hp41cv-functions.json'` from
`hp41-gui/src/help_data.ts` climbs two levels up; vite's default
`server.fs.allow` blocks this. The fix:

```typescript
// hp41-gui/vite.config.ts
fs: {
  allow: [path.resolve(__dirname, '..')],  // repo root
},
```

This is additive — no other vite behavior changes. The production
build now succeeds: `vite build` bakes the entire 154-entry JSON
array into the bundle (~211 KB total bundle, ~66 KB gzipped). At
runtime there's zero fetch, zero filesystem access — the array is a
module-level constant evaluated once at module load.

Hard-build-blocker semantics (D-25.17 / D-26.8): if the JSON is
malformed, vite's build fails. The same guarantee the Rust side
provides via `include_str!` + `expect("hp41cv-functions.json is malformed")`.

## W9 — Hardcoded keyCode Literals (CRITICAL)

The original CONTEXT D-26.9 "Claude's Discretion" left it open whether
to compute `keyCode` from `(row, col)` at render time or hardcode. The
W9 audit established that computing `row * 10 + col` would be INCORRECT:

- GUI Keyboard.tsx uses 0-indexed cols (0..4); HP-41 hardware key codes
  in `keycode_to_hp41_code` use 1-indexed rows AND cols.
- GUI row numbering does NOT match HP-41 hardware row numbering. E.g.
  SIN at GUI `(row 2, col 2)` is HP-41 code 25 per `keys.rs`, but
  `2 * 10 + 2 = 22` is the STO code — naive computation would
  mis-label the SIN key as STO on every USER overlay.
- The bottom four rows of the GUI (5-8) use a 4-column wide layout
  with operators in `col 0`. HP-41 hardware places these same
  operators in `col 5` of various rows. The mapping is purely
  conventional, established by CLI Phase 19 R/S binding (`r_s` -> 31)
  and the canonical CLI key->code table.

**Resolution:** `keyCode` is HARDCODED as a literal on each
`KEY_DEFS` entry, sourced directly from `hp41-cli/src/keys.rs::keycode_to_hp41_code`.
Coverage:

- 32 of 35 main-grid entries gain a `keyCode` literal.
- 3 entries are LEFT UNDEFINED because the CLI mapping is ambiguous
  or unmapped:
  - `xge_y` (GUI-specific x>=y keycap at row 2 col 0; no CLI binding)
  - `chs` (HP-41 hardware row 4 col 5 = 45 conflicts with `div`'s code)
  - `clx_or_a` (synthetic merged GUI key, no HP-41 hardware equivalent)
- 4 top-row entries (`variant: 'top'`) and 1 SHIFT key (`variant:
  'shift'`) are left undefined per the D-26.9 variant-exception rule.

**Drift-catch:** `hp41-gui/src/Keyboard.test.tsx` includes 28 explicit
`KEY_DEFS '<id>'.keyCode === <expected>` assertions, one per CLI
sentinel. If `keys.rs` updates a code (e.g. R/S moves from 31 to a
different row), or if a typo lands in `KEY_DEFS`, this test fails
before users see incorrect USER overlays.

The prior draft's `hp41KeyCode(row, col)` helper has been ELIMINATED —
`grep -c "hp41KeyCode" hp41-gui/src/Keyboard.tsx` returns 0.

## T-26-03-04 — Injected-Content Mitigation

The user supplies ASN labels via `Op::Asn { name, key_code }`; the label
flows through `CalcStateView.user_keymap` -> React props -> SVG
`<text>` element. Untrusted content must not be interpreted as markup.
The mitigation chain:

1. **React default text-node rendering**: `{userLabel ?? key.label}`
   is a JSX expression that renders as a text node. React's reconciler
   escapes content before inserting it into the DOM; HTML-like
   substrings become literal characters inside the SVG `<text>` element.
2. **Defensive 7-char truncation**: `entry[1].slice(0, 7)` caps the
   maximum displayed label length. HP-41C ASN labels are spec'd at
   6 chars; the 7-char cap fits in the keycap and bounds the
   worst-case substring length.

Verified by `Keyboard.test.tsx` test "XSS-safety: malicious ASN label
renders as literal text (T-26-03-04)":
```typescript
const malicious = '<script>alert(1)</script>';
render(<TestHarness userActive={true} userKeymap={[[25, malicious]]} />);
// Asserts: querySelector('script') === null  (no element injected)
// Asserts: literal substring '<scrip' present in SVG <text> content
// Asserts: '</script>' NOT present (truncated by 7-char slice)
```

## HelpOverlay Entry Counts

| Metric | Count |
|---|---:|
| Total entries in `docs/hp41cv-functions.json` | 154 |
| Entries with `key_path !== null` (visible in overlay) | 62 |
| Distinct categories rendered as section headings | 11 |
| Wave-1 vitest count baseline | 42 |
| Wave-2 vitest count baseline (Plan 26-02) | 22 |
| New tests added (Plan 26-03) | 57 (16 HelpOverlay + 41 Keyboard) |
| Total Vitest count after Plan 26-03 | **121 / 121 PASSED** |

## CLAUDE.md Update Needed

Per ROADMAP cross-cutting constraint, the v2.2 Phase 26 section in
`CLAUDE.md` should be extended with:
- New `?` overlay keyboard shortcut and its escape semantics.
- USER-mode relabel mechanism: `KeyDef.keyCode` literals from CLI
  canonical mapping; W9 drift-catch test as the source of truth.
- Physical-keyboard `'p'` MAP swap (was `prx`, now `prgm_mode`);
  `'P'` (shift+p) is the new `prx` route.
- W7 file-region split as a recipe for future multi-plan App.tsx
  edits.
- T-26-03-04 content-escape note (React default text-node behavior +
  7-char defensive slice).

The CalcStateView envelope itself (`user_keymap`, `flags`,
`display_override`, `event_buffer`) was documented in Plan 26-01's
SUMMARY; this plan doesn't add new IPC surface.

## Manual Smoke Test Plan (post-merge)

The user should run `just gui-dev` and verify:
1. Press `?` -> overlay opens, search input has focus, ~62 entries
   visible grouped into 11 categories.
2. Type `sin` -> list narrows to entries containing "sin" (case-
   insensitive). Empty input restores full list.
3. Press Esc -> overlay closes.
4. Open a register modal (e.g. SHIFT+STO) -> press Esc -> modal closes,
   shift cleared, overlay still closed (Esc precedence: help -> modal
   -> shift; one layer per Esc keystroke).
5. ASN a label to key 22 (e.g. `f-ASN "TEST" 22` via the ASN modal
   flow from Plan 26-01) -> toggle USER mode -> SVG keycap at row 3
   col 2 (STO) now shows `TEST` instead of `STO`. Toggle USER off ->
   `STO` returns.
6. Press lowercase `'p'` on physical keyboard -> PRGM annunciator
   toggles. Press `SHIFT+'P'` -> PRX fires (X register printed to
   print panel).

No visual issues were anticipated; the .display CSS rule from Plan
26-02 D-26.7 is preserved byte-identical (Plan 26-03 only adds
`.help-overlay` rules to App.css; existing rules untouched).

## SC-4 Invariant Verification

```
$ grep -rEn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/
# (no output — zero matches)
```

Phase 26 Plan 03 is FRONTEND-ONLY. No `hp41-gui/src-tauri/` files
touched; no `hp41-core` / `hp41-cli` files touched. SC-4 invariant
unaffected.

## Save-File Backward Compatibility

No `CalcState` field changes. No `CalcStateView` envelope additions.
v1.0–v2.1 save files continue to load without migration. The
`user_keymap` projection consumed for USER-mode relabel was added in
Plan 26-01 (Phase 26 wave 1) per BLOCKER B5 and is already a
documented part of the v2.2 IPC schema.

## Deviations from Plan

### Auto-fixed Issues

1. **[Rule 3 - Blocking] KeyboardProps extension landed in Task 2 rather than Task 3.**
   The plan's W7 file-region split lists Keyboard.tsx changes only in
   Task 3, but Task 2 adds `userActive` and `userKeymap` props on
   `<Keyboard />` from App.tsx — which fails TypeScript compilation
   without first extending `KeyboardProps`. Task 2's acceptance
   criteria explicitly requires `tsc --noEmit` to be clean. Resolved
   by adding the OPTIONAL props (no behavior change yet) in the Task 2
   commit; the actual `keyCode` literals and the `resolveUserLabel`
   render logic land in Task 3. Files touched: `hp41-gui/src/Keyboard.tsx`
   gained a 4-line `KeyboardProps` extension in commit `9baaa9a` (Task 2);
   the rest of Keyboard.tsx changes are in commit `a1b024c` (Task 3).
   No deviation from the user-observable plan outcome.

2. **[Rule 3 - Blocking] `'P'` removed from the modal-trigger silence list.**
   `resolveKeyId` in App.tsx had `'P'` in the silence string `'SRfFPX'`
   (Phase 19 leftover when uppercase 'P' was a v2.0 modal trigger
   placeholder). Without removing 'P', the new D-26.10 MAP entry
   `'P': 'prx'` would never be reached because `resolveKeyId` returns
   `null` before the MAP lookup. Surgically narrowed the silence list
   to `'SRfFX'` (kept S, R, f, F, X — all legitimate modal-trigger
   letters used by v2.0 placeholder routes). Documented inline.

### Authentication Gates

None encountered.

## All Gates Green

| Gate | Status | Detail |
|------|--------|--------|
| `npx tsc --noEmit` | OK | clean (zero type errors) |
| `npx vitest run` | OK | 121 / 121 passed |
| `just gui-check` | OK | cargo check clean |
| `just gui-ci` | OK | tsc + cargo test + release build all green |
| `vite build` | OK | production bundle succeeds; JSON import resolves (W8) |
| SC-4 invariant grep | OK | zero matches in hp41-gui/src-tauri/ |
| W7 cross-plan grep | OK | `<Display14Seg` + `<HelpOverlay open=` both present in App.tsx (3 matches) |
| W9 sentinel parity (Vitest) | OK | 28 keyCode assertions green |
| Content-escape T-26-03-04 (Vitest) | OK | no element injected into DOM |
| `'p'` MAP swap (grep) | OK | `'p': 'prgm_mode'` + `'P': 'prx'` both present |

## Cross-References

- **Plan 26-01** (modal architecture & key wiring) — supplied the
  `pendingInput` Esc semantics that the new help -> modal -> shift
  precedence preserves, AND the `CalcStateView.user_keymap`
  projection that feeds the USER-mode relabel.
- **Plan 26-02** (14-segment LCD) — coexists with this plan in
  App.tsx per the W7 file-region split; the `.display` CSS rule and
  the `<Display14Seg>` JSX are untouched.
- **CLI Phase 25 D-25.16 / D-25.17** — the canonical JSON pipeline
  this plan's `help_data.ts` consumes. The CLI side and the GUI side
  read the SAME file at build time (no duplication).
- **CLI Phase 25 D-25.6** — CLI <-> GUI parity at the user-observable
  layer. The CLI's `?` overlay (in `hp41-cli/src/ui.rs::render_help_overlay`)
  shows the same 62 keyboard-bound entries; this plan ships the GUI
  side of that contract.

## Self-Check: PASSED

**Files created (verified):**
- `hp41-gui/src/help_data.ts` — FOUND
- `hp41-gui/src/HelpOverlay.tsx` — FOUND
- `hp41-gui/src/HelpOverlay.test.tsx` — FOUND
- `hp41-gui/src/Keyboard.test.tsx` — FOUND

**Commits (verified via `git log --oneline -3`):**
- `de70dff feat(26-03): add HelpOverlay + help_data port + vite fs.allow (Task 1)` — FOUND
- `9baaa9a feat(26-03): wire HelpOverlay + '?' handler + D-26.10 'p' remap (Task 2)` — FOUND
- `a1b024c feat(26-03): USER-mode relabel + KeyDef.keyCode W9 literals (Task 3)` — FOUND
