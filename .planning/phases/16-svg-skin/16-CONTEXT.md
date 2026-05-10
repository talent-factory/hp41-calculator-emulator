# Phase 16: SVG Skin - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the empty `<div id="keyboard-area" />` placeholder in `hp41-gui/src/App.tsx` with a pixel-perfect, clickable SVG HP-41C keyboard: 40 keys in a 9×5 grid, ENTER spanning two columns, HP-41C color scheme (dark brown body, white primary labels, gold f-shift labels), click handlers wired to the existing `dispatch_op` IPC, and a 150ms CSS scale-down press animation. Also update the Tauri window to 400×700 and the calculator width to 400px.

This phase delivers SKIN-01, SKIN-02, SKIN-03.

</domain>

<decisions>
## Implementation Decisions

### SVG Authoring Method

- **D-01:** **Data-driven loop** — `KEY_DEFS` is a typed `const` array; each entry contains `{ id, label, fShiftLabel?, row, col, colSpan }`. The `<Keyboard>` component loops over `KEY_DEFS` to render SVG `<g>` elements. Key positions are computed from `(row, col, colSpan)` with shared `KEY_W`, `KEY_H`, `GAP` constants.
- **D-02:** `KEY_DEFS` lives **inline in `Keyboard.tsx`** at the top of the file — no separate data module.
- **D-03:** Irregular keys use a **`colSpan` field** (default 1). Width formula: `colSpan * KEY_W + (colSpan - 1) * GAP`. The loop tracks a per-row column cursor and skips covered slots after a double-width key. ENTER gets `colSpan: 2`.
- **D-04:** SVG is authored as **JSX SVG elements** (`<svg>`, `<rect>`, `<text>`, `<g>` in TSX). No raw string templates or unsafe HTML injection — `onClick`, `className`, and `style` props work natively in JSX.

### Key Label Depth

- **D-05:** Each key renders **primary label (white, on key face) + gold f-shift label (above the key)**. No g-shift (blue) labels in Phase 16.
- **D-06:** **All 40 HP-41C keys** are defined in `KEY_DEFS` — no partial subset. SC-1 requires all 40 to be rendered.
- **D-07:** Keys with no f-shift function (digit keys 0–9, ENTER, arithmetic ops, etc.) render **no gold text element** — the `fShiftLabel` field is omitted/undefined and the loop skips rendering it.

### Component Integration

- **D-08:** SVG keyboard is a **standalone `<Keyboard />` component** in `hp41-gui/src/Keyboard.tsx`. It accepts `onKey: (keyId: string) => void` and `busyRef: React.MutableRefObject<boolean>` props. `App.tsx` replaces `<div id="keyboard-area" />` with `<Keyboard onKey={handleClick} busyRef={busyRef} />`, where `handleClick` calls `invoke<CalcStateView>('dispatch_op', { keyId })`.
- **D-09:** **Calculator width grows from 320px to 400px** — update `.calculator` width in `App.css`. SVG keyboard fills the full 400px width below the existing display/annunciator/stack panel. The calculator container flows vertically (column flex) as before.
- **D-10:** **Update `tauri.conf.json`** `windows[0].width` to `400` and `height` to `700` (SC-5 requirement). `resizable: false` to lock the window at the design size.

### Click Handler Pattern

- **D-11:** **Individual `onClick` on each key `<g>` element** — rendered by the loop as `onClick={() => onKey(key.id)}`. No event delegation, no `data-*` attribute traversal. Idiomatic React JSX.
- **D-12:** **CSS press animation via React state** — `Keyboard.tsx` tracks `pressedKey: string | null` with `useState`. `onClick` sets `pressedKey = key.id`, then `setTimeout(() => setPressedKey(null), 150)` clears it. Each `<g>` gets `className={pressedKey === key.id ? 'key key-pressed' : 'key'}`. CSS: `.key-pressed { transform: scale(0.92); transition: transform 80ms ease-out; }`. Animation completes within 150ms (SC-4).
- **D-13:** **`busyRef` passed from `App.tsx` to `<Keyboard>`**. `onClick` checks `if (busyRef.current) return;` before setting `pressedKey` or calling `onKey`. Prevents click-spamming while an IPC round-trip is in flight, matching the keyboard debounce pattern from Phase 15.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope
- `.planning/ROADMAP.md` §Phase 16 — Goal, Success Criteria SC-1 through SC-5, Requirements SKIN-01/02/03
- `.planning/REQUIREMENTS.md` §SKIN — SKIN-01, SKIN-02, SKIN-03 definitions

### Existing IPC Layer
- `hp41-gui/src-tauri/src/key_map.rs` — Authoritative list of all valid `key_id` strings; SVG `KEY_DEFS` `id` fields MUST match entries here
- `hp41-gui/src/App.tsx` — Current App with `busyRef`, `handleKey`, `resolveKeyId`, `<div id="keyboard-area" />` placeholder; Keyboard integration point
- `hp41-gui/src/App.css` — Existing `.calculator { width: 320px }` that Phase 16 must update to 400px
- `hp41-cli/src/keys.rs` — `key_to_op()` and `KEY_REF_TABLE` — cross-reference for key label text (what each op is called)

### Tauri Configuration
- `hp41-gui/src-tauri/tauri.conf.json` — Window size config; `windows[0].width` and `height` to be updated to 400×700

### Prior Phase Decisions
- `.planning/phases/14-ipc-layer/14-CONTEXT.md` — D-04..D-07: key ID naming convention (snake_case strings, digit keys special-cased in dispatch_op, unknown IDs return GuiError)
- `.planning/phases/15-display-and-keyboard/15-03-PLAN.md` — Phase 15 keyboard wiring; `busyRef` debounce pattern; `resolveKeyId` function

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `busyRef: React.MutableRefObject<boolean>` in `App.tsx` — already exists; pass as prop to `<Keyboard>` for shared debounce
- `invoke<CalcStateView>('dispatch_op', { keyId })` pattern — established in Phase 15; `handleClick` in App.tsx mirrors `handleKey`
- `key_map::resolve()` in `hp41-gui/src-tauri/src/key_map.rs` — complete list of all valid key IDs; use as the authoritative source for `KEY_DEFS` `id` values

### Established Patterns
- **Vanilla CSS (no component library)** — Phase 13/15 decision; Keyboard.tsx uses className strings and CSS rules in App.css (or a new Keyboard.css)
- **`#![deny(clippy::unwrap_used)]`** — applies to hp41-core; Rust-side changes in this phase are minimal (no new commands expected)
- **JSX SVG** — same Vite/React build pipeline already handles JSX; no SVGR plugin needed for inline JSX SVG
- **`useRef` for busyRef, `useState` for pressedKey** — consistent with Phase 15 React patterns

### Integration Points
- `App.tsx` line with `<div id="keyboard-area" />` — replace with `<Keyboard onKey={handleClick} busyRef={busyRef} />`
- `App.tsx` add `handleClick` callback: `(keyId: string) => invoke<CalcStateView>('dispatch_op', { keyId }).then(setCalcState)`
- `App.css` `.calculator { width: 320px }` → `400px`
- `hp41-gui/src-tauri/tauri.conf.json` window dimensions

</code_context>

<specifics>
## Specific Ideas

- **HP-41C color palette** (from roadmap SC-2): dark brown body (`#3d2b1f` or similar), light-colored key caps for top row (USER/PRGM/ALPHA/f/g keys), gold shift legend text (`#c8a400` or similar), white primary legend text
- **Key geometry**: 9 columns, 5 rows, ENTER at row 1 spanning cols 5–6 (zero-indexed). Top row (row 0) contains the A–E softkeys plus SST/BST, GTO, and possibly ON
- **viewBox**: should be `"0 0 400 560"` or similar so SVG fills the 400px width without stretching
- **CSS transition**: `.key-pressed { transform: scale(0.92); transition: transform 80ms ease-out; }` — quick snap-down, 150ms total before pressedKey clears

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 16-SVG Skin*
*Context gathered: 2026-05-10*
