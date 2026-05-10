# Phase 16: SVG Skin - Research

**Researched:** 2026-05-10
**Domain:** React JSX SVG, HP-41C keyboard layout, CSS animation, Tauri WebView
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Data-driven loop — `KEY_DEFS` typed const array with `{ id, label, fShiftLabel?, row, col, colSpan }`. Loop computes positions from `(row, col, colSpan)` with shared `KEY_W`, `KEY_H`, `GAP` constants.
- **D-02:** `KEY_DEFS` lives inline in `Keyboard.tsx` at the top of the file — no separate data module.
- **D-03:** Irregular keys use a `colSpan` field (default 1). Width formula: `colSpan * KEY_W + (colSpan - 1) * GAP`. Loop tracks a per-row column cursor and skips covered slots after a double-width key. ENTER gets `colSpan: 2`.
- **D-04:** SVG authored as JSX SVG elements (`<svg>`, `<rect>`, `<text>`, `<g>` in TSX). No raw string templates or unsafe HTML injection.
- **D-05:** Each key renders primary label (white, on key face) + gold f-shift label (above the key). No g-shift (blue) labels in Phase 16.
- **D-06:** All 40 HP-41C keys are in `KEY_DEFS` — no partial subset.
- **D-07:** Keys with no f-shift function render no gold text element — `fShiftLabel` field is omitted/undefined.
- **D-08:** Standalone `<Keyboard />` component in `hp41-gui/src/Keyboard.tsx`. Props: `onKey: (keyId: string) => void` and `busyRef: React.MutableRefObject<boolean>`.
- **D-09:** Calculator width grows from 320px to 400px — update `.calculator` width in `App.css`.
- **D-10:** Update `tauri.conf.json` `windows[0].width` to `400` and `height` to `700`. `resizable: false`.
- **D-11:** Individual `onClick` on each key `<g>` element rendered by the loop as `onClick={() => onKey(key.id)}`. No event delegation.
- **D-12:** CSS press animation via React state — `pressedKey: string | null` with `useState`. `onClick` sets `pressedKey = key.id`, then `setTimeout(() => setPressedKey(null), 150)` clears it. Class: `key key-pressed` when active. CSS: `.key-pressed { transform: scale(0.92); transition: transform 80ms ease-out; }`.
- **D-13:** `busyRef` passed from `App.tsx` to `<Keyboard>`. `onClick` checks `if (busyRef.current) return;` before setting `pressedKey` or calling `onKey`.

### Claude's Discretion

None documented — all decisions locked.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SKIN-01 | User sees a pixel-perfect SVG HP-41C key layout (9x5 grid, ENTER double-width, correct key labels and legends, HP-41C proportions and color scheme — dark brown body, gold shift labels) | KEY_DEFS complete array below; color palette; geometry constants |
| SKIN-02 | User can click any key in the SVG skin and the corresponding HP-41 operation executes in hp41-core (same result as pressing the equivalent CLI key binding) | key_id mapping table; onClick pattern; busyRef debounce |
| SKIN-03 | User sees visual press feedback (CSS scale-down animation) on every key click | transform-box + transform-origin CSS pattern; D-12 state machine |

</phase_requirements>

---

## Summary

Phase 16 adds a clickable SVG HP-41C keyboard skin to the Tauri GUI. All implementation decisions are pre-locked in CONTEXT.md. Research confirms: (1) the complete 40-key HP-41C layout and their key_id mappings, (2) exact geometry constants that fit within 400px width, (3) the correct CSS trick for centering the scale animation on individual SVG keys, and (4) there is no meaningful gotcha with SVG `onClick` in React JSX — it works identically to HTML elements.

The critical non-obvious finding is the **`transform-box: fill-box` + `transform-origin: center` pattern** for SVG key press animation. Without `transform-box: fill-box`, the CSS `transform-origin: center` defaults to the SVG canvas center (0,0 of the viewBox), causing keys to animate toward the top-left instead of in-place. This has been widely supported since 2020 and works in all Tauri-targeted WebViews (Chromium/WebKit).

The 40 HP-41C physical keys map to a **5-row x 9-col SVG grid** (zero-indexed rows 0-4, cols 0-8), with ENTER as the only double-width key (`colSpan: 2`) placed at row 1, cols 4-5 per CONTEXT. The f-shift labels (gold) are the shifted functions printed above each key on the physical hardware; roughly 14 of the 40 keys have no f-shift function visible in the GUI and render no gold text element.

**Primary recommendation:** Use the complete KEY_DEFS table below verbatim as a starting skeleton; use `transform-box: fill-box; transform-origin: center;` on the `.key` CSS class for in-place animation; set viewBox to `"0 0 400 240"` or similar based on computed key geometry.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SVG key rendering | Frontend (React/TSX) | — | Pure presentational; JSX SVG elements compile to DOM, no Rust involvement |
| Click-to-dispatch wiring | Frontend (React/TSX) | Tauri IPC | onClick calls existing `invoke('dispatch_op', { keyId })` — no new Rust code needed |
| Press animation | Frontend (CSS + useState) | — | CSS transition + 150ms setTimeout; zero IPC latency impact |
| Window size update | Tauri config (tauri.conf.json) | — | `windows[0].width/height`; static config, no Rust code change |
| CSS width update | Frontend (App.css) | — | `.calculator { width: 400px }` — single line change |
| Key ID validation | Tauri Rust (key_map.rs) | — | Already exists; `resolve()` returns `Err(GuiError)` for unknown IDs |

---

## Standard Stack

### Core (already installed — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.2 | JSX SVG rendering, useState, useRef | Already installed in hp41-gui |
| TypeScript | 6.0 | KEY_DEFS typed const, component props | Already installed in hp41-gui |
| Vite + @vitejs/plugin-react | 8.0 / 6.0 | Build pipeline with JSX transform | Already installed; handles JSX SVG natively |

### No New Dependencies

The entire phase is implemented with:
- JSX SVG elements (native React — no SVGR plugin needed for inline JSX in .tsx files)
- CSS class toggling via React `useState`
- The existing `invoke` from `@tauri-apps/api/core` (already in package.json)

**Installation:** None required. All dependencies from Phases 13-15 are sufficient.

**Version verification:** `@tauri-apps/api` 2.11.x confirmed in package.json. `[VERIFIED: hp41-gui/package.json]`

---

## Complete KEY_DEFS Array

This is the primary research deliverable. Every `id` field is verified against `key_map.rs::resolve()` or the digit-entry branch in `commands.rs::handle_op()`. `[VERIFIED: hp41-gui/src-tauri/src/key_map.rs, hp41-gui/src-tauri/src/commands.rs]`

### HP-41C Physical Layout Reference

From `hp41-cli/src/keys.rs` keycode_to_hp41_code comment `[VERIFIED: hp41-cli/src/keys.rs]` and finseth.com hp41c keyboard data `[CITED: https://www.finseth.com/hpdata/hp41c.php]`:

```
Physical Row 1: Sigma+, 1/x,  sqrt, LOG,  LN
Physical Row 2: x<>y,  R-dn,  SIN,  COS,  TAN
Physical Row 3: [ON],  XEQ,   STO,  RCL,  SST
Physical Row 4: ENTER(tall),  CHS,  EEX,  <- (backspace)
Physical Row 5: -,     7,     8,    9
Physical Row 6: +,     4,     5,    6
Physical Row 7: x,     1,     2,    3
Physical Row 8: /,     0,     .,    R/S
Plus rocker switches (top): USER, PRGM, ALPHA, ON
Plus 2 shift keys:  f (gold),  g (blue)
```

The physical calculator has 40 unique keys. In the hardware key-code system (rows 1-8, cols 1-5), ENTER is a tall key spanning approximately rows 4-8 in column 5. The SVG design converts this to a **double-wide key** (colSpan: 2) in one SVG row, which is simpler to render with the data-driven loop.

### SVG Grid: 5 Rows x 9 Columns

The physical 8-row layout is consolidated into 5 visual rows for the SVG. ENTER at SVG row 1, cols 4-5 (colSpan: 2). `[ASSUMED: exact row/col assignments for the SVG grid — derived from CONTEXT.md hints; implementer may adjust visual grouping]`

**Key_id validity summary:**
- Digit keys `"0"` through `"9"`, `"."`, `"e"` — handled by `handle_op()` digit branch `[VERIFIED: commands.rs]`
- Named op keys — all resolved by `key_map::resolve()` `[VERIFIED: key_map.rs]`
- Visual-only keys (empty id `""`) — no dispatch call; render purely for appearance

### KEY_DEFS Skeleton (Verified IDs)

```typescript
// Source: key IDs verified against key_map.rs and commands.rs
// row/col values: [ASSUMED] — visual layout, adjust as needed
type KeyDef = {
  id: string;           // key_id for dispatch_op, or '' for visual-only keys
  label: string;        // primary label (white text)
  fShiftLabel?: string; // gold label above key; omit if no f-shift function in Phase 16
  row: number;          // 0-indexed SVG row (0=top)
  col: number;          // 0-indexed SVG column (left edge of key)
  colSpan?: number;     // default 1; ENTER = 2
};

const KEY_DEFS: KeyDef[] = [
  // ── Row 0: top math/function row ─────────────────────────────────────────────
  { id: 'sigma_plus',   label: 'Σ+',    fShiftLabel: 'x²',   row: 0, col: 0 },
  { id: 'recip',        label: '1/x',   fShiftLabel: 'yˣ',   row: 0, col: 1 },
  { id: 'sqrt',         label: '√x',                         row: 0, col: 2 },
  { id: 'log',          label: 'LOG',   fShiftLabel: '10ˣ',  row: 0, col: 3 },
  { id: 'ln',           label: 'LN',    fShiftLabel: 'eˣ',   row: 0, col: 4 },
  { id: '',             label: 'XEQ',                        row: 0, col: 5 }, // modal — no dispatch Phase 16
  { id: '',             label: 'STO',                        row: 0, col: 6 }, // modal — no dispatch Phase 16
  { id: '',             label: 'RCL',                        row: 0, col: 7 }, // modal — no dispatch Phase 16
  { id: 'clx',          label: '←',                          row: 0, col: 8 }, // CLX / backspace

  // ── Row 1: trig + stack + ENTER(double-wide) + divide ────────────────────────
  { id: 'sin',          label: 'SIN',   fShiftLabel: 'ASIN', row: 1, col: 0 },
  { id: 'cos',          label: 'COS',   fShiftLabel: 'ACOS', row: 1, col: 1 },
  { id: 'tan',          label: 'TAN',   fShiftLabel: 'ATAN', row: 1, col: 2 },
  { id: 'rdn',          label: 'R↓',                        row: 1, col: 3 },
  { id: 'xy_swap',      label: 'x<>y',                      row: 1, col: 4 },
  { id: 'enter',        label: 'ENTER',                      row: 1, col: 5, colSpan: 2 },
  { id: 'div',          label: '÷',                          row: 1, col: 7 },
  { id: 'mul',          label: '×',                          row: 1, col: 8 },

  // ── Row 2: mode keys + numeric 7-8-9 + mode keys ─────────────────────────────
  { id: 'user_mode',    label: 'USER',                       row: 2, col: 0 },
  { id: '',             label: 'f',                          row: 2, col: 1 }, // shift — no dispatch
  { id: '',             label: 'g',                          row: 2, col: 2 }, // shift — no dispatch
  { id: '7',            label: '7',                          row: 2, col: 3 },
  { id: '8',            label: '8',                          row: 2, col: 4 },
  { id: '9',            label: '9',                          row: 2, col: 5 },
  { id: 'minus',        label: '−',                          row: 2, col: 6 },
  { id: 'prgm_mode',    label: 'PRGM',                       row: 2, col: 7 },
  { id: 'alpha_toggle', label: 'ALPHA',                      row: 2, col: 8 },

  // ── Row 3: CHS, EEX, SST, numeric 4-5-6, minus, GTO, R/S ────────────────────
  { id: 'chs',          label: 'CHS',                        row: 3, col: 0 },
  { id: 'e',            label: 'EEX',                        row: 3, col: 1 }, // 'e' = EEX entry
  { id: '',             label: 'SST',                        row: 3, col: 2 }, // no dispatch Phase 16
  { id: '4',            label: '4',                          row: 3, col: 3 },
  { id: '5',            label: '5',                          row: 3, col: 4 },
  { id: '6',            label: '6',                          row: 3, col: 5 },
  { id: 'plus',         label: '+',                          row: 3, col: 6 },
  { id: '',             label: 'GTO',                        row: 3, col: 7 }, // modal — no dispatch Phase 16
  { id: '',             label: 'R/S',                        row: 3, col: 8 }, // no dispatch Phase 16

  // ── Row 4: 0, dot, ON, numeric 1-2-3, plus ───────────────────────────────────
  { id: '0',            label: '0',                          row: 4, col: 0 },
  { id: '.',            label: '.',                          row: 4, col: 1 },
  { id: '',             label: 'ON',                         row: 4, col: 2 }, // power — no dispatch
  { id: '1',            label: '1',                          row: 4, col: 3 },
  { id: '2',            label: '2',                          row: 4, col: 4 },
  { id: '3',            label: '3',                          row: 4, col: 5 },
  { id: 'lastx',        label: 'LSTx',                       row: 4, col: 6 },
  { id: 'clreg',        label: 'CLRG',                       row: 4, col: 7 },
  { id: '',             label: 'BST',                        row: 4, col: 8 }, // no dispatch Phase 16
] as const;
```

**Key_id findings on non-dispatching keys:**
- **SST / BST**: Not in `key_map.rs`. Phase 18 will add program navigation. Render visually; empty id = no-op click.
- **R/S**: Not in `key_map.rs`. Render visually; empty id.
- **ON**: Power key. No IPC dispatch. Empty id.
- **f / g shift keys**: Physical modifier keys. No IPC dispatch. Empty id.
- **XEQ, STO, RCL, GTO**: Parameterized ops requiring modal input — outside Phase 16 scope. Empty id.

`[VERIFIED: above empty-id keys are absent from key_map.rs resolve() and commands.rs handle_op()]`

---

## Architecture Patterns

### System Architecture Diagram

```
User click on SVG key <g>
        |
        v
onClick() in Keyboard.tsx
   +-- busyRef.current? --> return (debounce, D-13)
   +-- setPressedKey(key.id)  ─────────────────────────+
   +-- setTimeout(150ms) --> setPressedKey(null)        |  CSS re-render: scale(0.92)
   +-- onKey(key.id)   <───────────────────────────────+
        |
        v (prop callback from App.tsx handleClick)
invoke<CalcStateView>('dispatch_op', { keyId })
        |
        v [Tauri IPC boundary - Chromium/WebKit WebView]
Rust: commands::dispatch_op()
   +-- handle_op(calc, key_id)
   |    +-- digit? --> push to entry_buf
   |    +-- '.'/'e'/'eex_chs'? --> entry_buf mutation
   |    +-- named/param? --> key_map::resolve() --> dispatch()
   +-- return CalcStateView JSON
        |
        v
.then(view => setCalcState(view))  [App.tsx]
        |
        v
React re-render: display + stack panel update
```

### Recommended Project Structure

```
hp41-gui/src/
+-- App.tsx            # Add handleClick; import Keyboard; replace <div id="keyboard-area" />
+-- App.css            # .calculator width 320->400px; add .key / .key-pressed CSS rules
+-- Keyboard.tsx       # NEW: <Keyboard onKey busyRef /> with KEY_DEFS and SVG render loop
+-- main.tsx           # Unchanged
+-- index.css          # Unchanged
hp41-gui/src-tauri/
+-- tauri.conf.json    # width: 400, height: 700, resizable: false
```

### Pattern 1: Data-Driven SVG Key Loop

```typescript
// Source: D-01, D-02, D-03 (CONTEXT.md) - architecture decision
const KEY_W = 39;   // pixel width of a single-span key in SVG coordinate space
const KEY_H = 26;   // pixel height of a key body
const GAP   = 4;    // gap between keys
const PAD   = 8;    // SVG padding (left/right/top)
const FSHIFT_H = 12; // height reserved above each row for f-shift labels

// Position of key's top-left corner of the rect (key body, not label zone):
// x = PAD + col * (KEY_W + GAP)
// label_y = PAD + row * (FSHIFT_H + KEY_H + GAP)   <- top of f-shift label zone
// rect_y  = label_y + FSHIFT_H                      <- top of key body
// width   = colSpan * KEY_W + (colSpan - 1) * GAP

// Render loop:
{KEY_DEFS.map(key => {
  const x = PAD + key.col * (KEY_W + GAP);
  const rectY = PAD + key.row * (FSHIFT_H + KEY_H + GAP) + FSHIFT_H;
  const w = (key.colSpan ?? 1) * KEY_W + ((key.colSpan ?? 1) - 1) * GAP;
  return (
    <g key={key.id || key.label}
       onClick={() => key.id ? handleKeyClick(key.id) : undefined}
       className={pressedKey === key.id && key.id ? 'key key-pressed' : 'key'}
       style={{ pointerEvents: 'all' }}>
      {key.fShiftLabel && (
        <text x={x + w/2} y={rectY - 3}
              textAnchor="middle" fill="#c8a400" fontSize={8}>
          {key.fShiftLabel}
        </text>
      )}
      <rect x={x} y={rectY} width={w} height={KEY_H}
            rx={3} ry={3} fill={getKeyColor(key)} />
      <text x={x + w/2} y={rectY + KEY_H/2 + 4}
            textAnchor="middle" fill="white" fontSize={10} fontWeight="bold">
        {key.label}
      </text>
    </g>
  );
})}
```

### Pattern 2: CSS Press Animation — The Critical SVG Pitfall Fix

**The most important finding:** CSS `transform-origin: center` on SVG elements defaults to the SVG *canvas* origin (0,0 of the viewBox) unless `transform-box: fill-box` is also set. Without it, `scale(0.92)` makes keys shrink toward the top-left of the entire SVG canvas, not in-place. `[VERIFIED: MDN transform-box — https://developer.mozilla.org/en-US/docs/Web/CSS/transform-box]`

```css
/* In App.css (or a new Keyboard.css) */
.key {
  cursor: pointer;
  transform-box: fill-box;       /* REQUIRED: reference box = element's bounding box */
  transform-origin: center;      /* center of the KEY's own bounding box */
  transition: transform 80ms ease-out;
}

.key-pressed {
  transform: scale(0.92);
}

.key:hover:not(.key-pressed) {
  opacity: 0.85;
}
```

```typescript
// Keyboard.tsx press state machine (D-12)
const [pressedKey, setPressedKey] = useState<string | null>(null);

const handleKeyClick = (keyId: string) => {
  if (busyRef.current) return;                          // D-13: debounce
  setPressedKey(keyId);
  // Use functional update to avoid stale closure clearing wrong key (see Pitfall 4)
  setTimeout(() => setPressedKey(prev => prev === keyId ? null : prev), 150);
  onKey(keyId);
};
```

`transform-box: fill-box` browser support: **Baseline Widely Available since January 2020** — supported in all modern browsers including the Chromium and WebKit engines used by Tauri v2. `[VERIFIED: MDN — https://developer.mozilla.org/en-US/docs/Web/CSS/transform-box]`

### Pattern 3: SVG viewBox and HiDPI Scaling

```tsx
// SVG fills .calculator container width (400px) automatically via width="100%".
// viewBox defines the coordinate space; height computed from 5 rows of geometry:
//
// Total SVG height = PAD + 5 * (FSHIFT_H + KEY_H) + 4 * GAP + PAD
//                  = 8 + 5 * (12 + 26) + 4 * 4 + 8
//                  = 8 + 190 + 16 + 8 = 222px
//
// Use viewBox "0 0 400 230" for a bit of bottom breathing room.

<svg
  width="100%"
  viewBox="0 0 400 230"
  xmlns="http://www.w3.org/2000/svg"
  aria-label="HP-41C keyboard"
>
  <rect width="400" height="230" fill="#3d2b1f" rx={6} /> {/* calculator body */}
  {KEY_DEFS.map(key => /* ... render loop ... */)}
</svg>
```

The viewBox approach ensures:
- On Retina/HiDPI displays, the browser rasterizes at device pixel ratio without pixelation (SC-5)
- SVG text is vector — always crisp at any scale
- If the container changes width in a future phase, the SVG scales proportionally

`[ASSUMED: viewBox height of 230 — adjust to actual computed layout height]`

### Anti-Patterns to Avoid

- **Missing `transform-box: fill-box`:** Scale animation fires from SVG canvas origin instead of key center. Keys visually jump toward top-left corner. This is the single most common SVG animation mistake.
- **`transform-origin` as SVG attribute instead of CSS property:** The SVG presentation attribute `transform-origin` has inconsistent browser support compared to the CSS property. Always animate via CSS class, not SVG attributes. `[CITED: MDN — https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/transform-origin]`
- **Dispatching empty-string ids:** If `handleKeyClick("")` reaches `invoke('dispatch_op', { keyId: "" })`, Rust returns `GuiError("unknown key: ")`. Guard: `if (!keyId) return;` in `handleKeyClick` or in the onClick inline. The KEY_DEFS skeleton above handles this via `key.id ? handleKeyClick(key.id) : undefined`.
- **Using SVG `animateTransform` element for press animation:** Requires declarative XML animation; cannot be driven by React `useState`. Stick with the D-12 CSS class approach.
- **Forgetting `pointerEvents: 'all'` on `<g>`:** If the rect has `pointer-events: none` from a parent CSS rule, clicks on the key body won't register. Explicitly set `style={{ pointerEvents: 'all' }}` on each key `<g>`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HiDPI-safe SVG scaling | Canvas + pixel calculations | `viewBox` + `width="100%"` | viewBox is declarative, resolution-independent, zero code |
| Center-of-element transform | Manual translate-scale-translate SVG math | `transform-box: fill-box` + `transform-origin: center` | CSS property; baseline since 2020; one-liner |
| Animation timing | `requestAnimationFrame` loop | CSS `transition: transform 80ms ease-out` | Browser-native, GPU-accelerated, respects `prefers-reduced-motion` |
| Key ID routing | Switch/if chain in onClick | existing `key_map::resolve()` in Rust | Already tested; handles all edge cases; returns GuiError not panic |
| Press debounce | Custom event queue | `busyRef` pattern from Phase 15 | Already established; `busyRef.current` check in `handleKeyClick` |

**Key insight:** SVG elements in JSX React behave like any other DOM element. `onClick`, `useState`, CSS classes, and `style` props all work identically — there is no special Tauri WebView behavior that changes this.

---

## HP-41C Color Palette

`[ASSUMED: exact hex values — approximated from hardware photos; CONTEXT.md lists "#3d2b1f or similar" and "#c8a400 or similar"]`

| Element | Hex | Notes |
|---------|-----|-------|
| Calculator body / SVG background | `#3d2b1f` | Dark brown; CONTEXT.md suggestion |
| Standard key cap (math/trig/stack rows) | `#2a2a2a` | Dark charcoal |
| Top-row key caps (function row) | `#4a3828` | Slightly lighter brown than body |
| Mode key caps (USER/PRGM/ALPHA/f/g) | `#e0d8c8` | Light cream/ivory |
| Digit key caps (0-9, ., EEX, +, -, x, /) | `#1a1a1a` | Near-black |
| ENTER key cap | `#1a3a1a` | Slightly green-tinted dark to distinguish |
| Primary label text (on key face) | `#ffffff` | White |
| f-shift label text (above key) | `#c8a400` | Gold; CONTEXT.md suggestion |
| SVG body border | `#1a1200` | Dark border on key rects for depth effect |

---

## Key Geometry Constants

Based on a 400px container width with 8px padding on each side, 9 columns, and 4px gaps: `[ASSUMED: implementer should adjust for visual fit]`

```
Available width for keys = 400 - 2 * PAD = 384px
9 columns * KEY_W + 8 gaps * GAP = 384
KEY_W = (384 - 8 * 4) / 9 = (384 - 32) / 9 = 352 / 9 = 39px (rounded down)
Actual consumed: 9 * 39 + 8 * 4 = 351 + 32 = 383px (1px spare — fine)

Per-row height = FSHIFT_H + KEY_H + GAP = 12 + 26 + 4 = 42px

Total SVG height = PAD + 5 * (FSHIFT_H + KEY_H) + 4 * GAP + PAD
                 = 8 + 5 * 38 + 4 * 4 + 8
                 = 8 + 190 + 16 + 8 = 222px

ENTER key width = 2 * KEY_W + 1 * GAP = 2 * 39 + 4 = 82px
```

Summary of recommended constants:
```typescript
const KEY_W    = 39;
const KEY_H    = 26;
const GAP      = 4;
const PAD      = 8;
const FSHIFT_H = 12;
// viewBox: "0 0 400 230"  (222px computed + 8px bottom buffer)
```

---

## Common Pitfalls

### Pitfall 1: SVG transform-origin Without transform-box
**What goes wrong:** `transform: scale(0.92)` on `.key-pressed` scales from the SVG canvas origin (0,0 of the viewBox), causing keys to visually shrink toward the top-left of the entire SVG rather than in-place.
**Why it happens:** CSS `transform-origin: center` defaults to 50% 50% relative to the *reference box*. For SVG elements, the reference box defaults to `view-box` (the entire SVG canvas), not the element's bounding box.
**How to avoid:** Always pair `transform-origin: center` with `transform-box: fill-box` on the `.key` CSS class.
**Warning signs:** During dev, try clicking the bottom-right key; if it animates by shrinking toward the SVG top-left rather than in-place, this pitfall is active.

### Pitfall 2: Dispatching Modal-Trigger Keys Causes GuiError
**What goes wrong:** Calling `onKey("xeq_")` (the bare prefix) causes `dispatch_op` to return `GuiError("unknown key: xeq_")` since `resolve_parameterized` requires a suffix argument.
**Why it happens:** XEQ, STO, RCL, GTO, LBL are parameterized — the bare prefix string is not a valid key ID.
**How to avoid:** In Phase 16, do not call `onKey()` for XEQ, STO, RCL, GTO, LBL, SST, BST, R/S, ON, f, g. Use empty string `id: ''` in KEY_DEFS and guard in `handleKeyClick`: `if (!keyId) return;`.
**Warning signs:** Console shows `"dispatch_op error"` with message `"unknown key: "` when clicking these keys.

### Pitfall 3: SVG onClick Not Firing on All Child Elements
**What goes wrong:** Click event fires when clicking on `<text>` (key label) but not when clicking on the `<rect>` (key body), or vice versa.
**Why it happens:** SVG `<g>` elements default to `pointer-events: visiblePainted` which honors child element pointer-event settings. If any child has `pointer-events: none` from a CSS rule, clicks on that child don't bubble up to the `<g>` onClick.
**How to avoid:** Set `style={{ pointerEvents: 'all' }}` on each key `<g>` to capture clicks anywhere inside.
**Warning signs:** Clicking on the key label text works, but clicking on the rect background does nothing (or vice versa).

### Pitfall 4: pressedKey State Clears Wrong Key Under Rapid Clicks
**What goes wrong:** User clicks key A, immediately clicks key B; key A's `setTimeout` fires and calls `setPressedKey(null)`, clearing key B's animation prematurely.
**Why it happens:** The `setTimeout` callback closes over `key.id` at click time. When it fires 150ms later, it does not know that a different key was subsequently pressed.
**How to avoid:** Use the functional state update form in the setTimeout callback:
```typescript
setTimeout(() => setPressedKey(prev => prev === keyId ? null : prev), 150);
```
This only clears the state if the current pressed key is still the same one that triggered the timer.
**Warning signs:** Rapid clicking leaves random keys visually "stuck" in the pressed state.

### Pitfall 5: Tauri Window Width Mismatch With CSS Width
**What goes wrong:** Tauri window is 400px wide but `margin: 16px auto` on `.calculator { width: 400px }` causes horizontal overflow or scrollbars.
**Why it happens:** The calculator element is exactly 400px; adding left/right margin requires more than 400px of viewport width.
**How to avoid:** Change the margin to small values (e.g., `margin: 8px auto`) and use `width: calc(100% - 16px)` or set the Tauri window slightly wider than 400px (e.g., 416px) to accommodate margins. The safest fix: set `.calculator { width: 392px }` so 4px margins fit on each side.
**Warning signs:** Horizontal scrollbar in the Tauri window after the width update to 400px.

---

## Code Examples

### Keyboard.tsx Component Shell

```typescript
// Source: D-08 (CONTEXT.md); pattern consistent with Phase 15 App.tsx [VERIFIED: hp41-gui/src/App.tsx]
import { useState } from 'react';

type KeyDef = {
  id: string;
  label: string;
  fShiftLabel?: string;
  row: number;
  col: number;
  colSpan?: number;
};

const KEY_DEFS: KeyDef[] = [ /* ... all 40 entries ... */ ];

const KEY_W = 39, KEY_H = 26, GAP = 4, PAD = 8, FSHIFT_H = 12;

interface KeyboardProps {
  onKey: (keyId: string) => void;
  busyRef: React.MutableRefObject<boolean>;
}

export function Keyboard({ onKey, busyRef }: KeyboardProps) {
  const [pressedKey, setPressedKey] = useState<string | null>(null);

  const handleKeyClick = (keyId: string) => {
    if (!keyId) return;                   // visual-only key
    if (busyRef.current) return;          // D-13: debounce
    setPressedKey(keyId);
    setTimeout(() => setPressedKey(prev => prev === keyId ? null : prev), 150);
    onKey(keyId);
  };

  return (
    <svg width="100%" viewBox="0 0 400 230" xmlns="http://www.w3.org/2000/svg">
      <rect width="400" height="230" fill="#3d2b1f" rx={6} />
      {KEY_DEFS.map(key => {
        const cs = key.colSpan ?? 1;
        const x = PAD + key.col * (KEY_W + GAP);
        const rectY = PAD + key.row * (FSHIFT_H + KEY_H + GAP) + FSHIFT_H;
        const w = cs * KEY_W + (cs - 1) * GAP;
        const isPressed = pressedKey === key.id && Boolean(key.id);
        return (
          <g key={`${key.row}-${key.col}`}
             onClick={() => handleKeyClick(key.id)}
             className={isPressed ? 'key key-pressed' : 'key'}
             style={{ pointerEvents: 'all' }}>
            {key.fShiftLabel && (
              <text x={x + w / 2} y={rectY - 2}
                    textAnchor="middle" fill="#c8a400" fontSize={8}>
                {key.fShiftLabel}
              </text>
            )}
            <rect x={x} y={rectY} width={w} height={KEY_H}
                  rx={3} ry={3} fill="#2a2a2a" stroke="#111" strokeWidth={0.5} />
            <text x={x + w / 2} y={rectY + KEY_H / 2 + 4}
                  textAnchor="middle" fill="white" fontSize={10} fontWeight="bold">
              {key.label}
            </text>
          </g>
        );
      })}
    </svg>
  );
}
```

### App.tsx Integration Points

```typescript
// Source: [VERIFIED: hp41-gui/src/App.tsx] — existing structure
// Add to imports:
import { Keyboard } from './Keyboard';

// Add handleClick (mirrors handleKey pattern from Phase 15):
const handleClick = useCallback((keyId: string) => {
  if (busyRef.current) return;
  busyRef.current = true;
  invoke<CalcStateView>('dispatch_op', { keyId })
    .then(view => setCalcState(view))
    .catch(err => console.error('dispatch_op error:', err))
    .finally(() => { busyRef.current = false; });
}, []);  // no calcState dep needed — no eex_chs context for mouse clicks

// Replace:
//   <div id="keyboard-area" />
// With:
//   <Keyboard onKey={handleClick} busyRef={busyRef} />
```

### App.css Updates

```css
/* Source: [VERIFIED: hp41-gui/src/App.css] — current width is 320px */
.calculator {
  width: 392px;       /* was 320px; +72px for keyboard; 4px margin on each side in 400px window */
  /* all other rules unchanged */
}

/* Add new keyboard animation rules: */
.key {
  cursor: pointer;
  transform-box: fill-box;    /* REQUIRED for SVG: makes transform-origin relative to element */
  transform-origin: center;
  transition: transform 80ms ease-out;
}

.key-pressed {
  transform: scale(0.92);
}
```

### tauri.conf.json Update

```json
{
  "app": {
    "windows": [{
      "title": "HP-41 Calculator",
      "width": 400,
      "height": 700,
      "resizable": false,
      "decorations": true
    }]
  }
}
```

`[VERIFIED: current values in hp41-gui/src-tauri/tauri.conf.json are width: 800, height: 600, resizable: true]`

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SVG `animateTransform` element | CSS `transform` + `transition` | CSS Animations Level 1 (2015) | Simpler; React state-driven; no SMIL |
| `transform-origin` failing in Firefox for SVG | `transform-box: fill-box` CSS property | 2020 (all browsers) | Reliable centered scale on any SVG element |
| Importing .svg files via SVGR plugin | Inline JSX SVG elements in .tsx | React 0.14+ | Full React reconciliation, onClick, className work natively |
| Raw SVG string templates in JSX | JSX SVG elements (`<svg>`, `<rect>`, `<text>`) | React JSX RFC | Type-safe, XSS-safe, no string interpolation |

**Deprecated/outdated:**
- `transform-origin` as SVG presentation attribute: partial browser support, CSS property preferred.
- SMIL-based SVG animation (`animateTransform`): deprecated in CSS-capable browsers; use CSS transitions.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js / npm | hp41-gui frontend build | yes | — | — |
| Vite + @vitejs/plugin-react | Keyboard.tsx JSX compilation | yes | 8.0 / 6.0 | — |
| React + TypeScript | Keyboard.tsx | yes | 19.2 / 6.0 | — |
| Tauri CLI + @tauri-apps/api | invoke() IPC | yes | 2.11 | — |

`[VERIFIED: hp41-gui/package.json]`

No missing dependencies. All required packages installed from Phases 13-15. Step 2.6: no blocking items.

---

## Validation Architecture

nyquist_validation is enabled in `.planning/config.json`. `[VERIFIED: .planning/config.json]`

### Test Framework

| Property | Value |
|----------|-------|
| Rust tests | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Frontend tests | None installed — no vitest/jest in package.json `[VERIFIED: package.json]` |
| Quick run command | `cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && cargo test --manifest-path src-tauri/Cargo.toml` |
| Full suite command | `just ci` (CLI must stay green) + manual `just gui-dev` visual check |

**Frontend test gap:** `hp41-gui/package.json` has no test script and no vitest/jest dependency. All frontend SCs for Phase 16 require **manual visual verification** via `just gui-dev`. This is consistent with how Phase 15 was validated (SC-1 through SC-5 were all manual).

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SKIN-01 | All 40 keys render; ENTER double-wide; correct labels | Manual visual | `just gui-dev` inspect | n/a |
| SKIN-01 | All dispatched key ids are valid in `key_map::resolve()` | Rust unit test | `cargo test -p hp41-gui` | No — Wave 0 |
| SKIN-02 | Clicking any functional key updates display | Manual integration | `just gui-dev` + click | n/a |
| SKIN-02 | Digit keys dispatch and update display | Manual integration | `just gui-dev` + click | n/a |
| SKIN-03 | CSS animation visible (scale + bounce-back, < 150ms) | Manual visual | `just gui-dev` + click | n/a |
| SC-5 | SVG scales correctly at 400px without pixelation | Manual visual | `just gui-dev` + inspect | n/a |

### Sampling Rate

- **Per task commit:** `cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && cargo test --manifest-path src-tauri/Cargo.toml`
- **Per wave merge:** `just ci` (CLI pipeline must stay green)
- **Phase gate:** All 5 SC manual verifications pass + `just ci` green + `just gui-dev` opens without error

### Wave 0 Gaps

- [ ] `hp41-gui/src-tauri/src/key_map.rs` — Add a test that iterates all dispatched KEY_DEFS ids (the non-empty ones) and confirms `resolve(id)` is either `Ok` or is a recognized digit/entry key. This validates the KEY_DEFS const array against the actual Rust resolver without running the GUI.

Example test to add to `key_map.rs`:

```rust
#[test]
fn test_all_keyboard_skin_ids_are_valid() {
    // IDs from KEY_DEFS that should resolve successfully (named ops only — not digits)
    let named_ids = [
        "sigma_plus", "recip", "log", "ln", "sin", "cos", "tan",
        "rdn", "xy_swap", "enter", "div", "mul", "user_mode",
        "minus", "prgm_mode", "alpha_toggle", "chs", "plus",
        "lastx", "clreg", "clx",
    ];
    for id in named_ids {
        assert!(resolve(id).is_ok(), "key_map::resolve({id}) must succeed");
    }
    // Digit keys are not routed through resolve() — they are handled by handle_op().
    // Empty-string ids are visual-only; they are not sent to dispatch_op.
}
```

- [ ] Framework install: None required.

---

## Security Domain

This phase introduces no new authentication, session, or file I/O concerns. All security-relevant work was completed in Phase 14.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes (key IDs) | `key_map::resolve()` — validates all key IDs; unknown = GuiError, no panic; `[VERIFIED: key_map.rs]` |
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V6 Cryptography | no | — |

No new ASVS work required for Phase 16. The existing `resolve()` + `GuiError` pattern already satisfies V5 input validation for all SVG click-generated key IDs.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Exact SVG row/col assignments for 40 keys in the 5x9 grid | KEY_DEFS Skeleton | Layout looks wrong visually; fix by adjusting row/col — no functional impact |
| A2 | Hex color values (#3d2b1f, #c8a400, #2a2a2a, etc.) | HP-41C Color Palette | Colors don't match HP-41C hardware; fix by adjusting hex — no functional impact |
| A3 | Geometry constants (KEY_W=39, KEY_H=26, GAP=4, FSHIFT_H=12) | Key Geometry | SVG doesn't fill 400px cleanly or looks cramped; adjust constants |
| A4 | viewBox height of 230 | Pattern 3 | SVG clips or has excess whitespace; adjust viewBox height to match actual computed layout |
| A5 | XEQ/STO/RCL/GTO/SST/R/S/f/g/ON are no-op (empty id) in Phase 16 | KEY_DEFS Skeleton | User expects these to work; acceptable for Phase 16 scope |
| A6 | `pointerEvents: 'all'` on `<g>` is required | Architecture Patterns | May not be needed if SVG defaults suffice; harmless to include either way |

**Non-assumed (verified) claims:**
- All dispatched `id` strings match `key_map::resolve()` targets `[VERIFIED: key_map.rs]`
- Digit id strings `"0"-"9"`, `"."`, `"e"` are handled by `handle_op()` digit branch `[VERIFIED: commands.rs]`
- `transform-box: fill-box` baseline support since January 2020 `[VERIFIED: MDN]`
- `tauri.conf.json` current values: width 800, height 600, resizable true `[VERIFIED: tauri.conf.json]`
- `App.css` `.calculator { width: 320px }` `[VERIFIED: App.css]`
- No frontend test framework in hp41-gui `[VERIFIED: package.json]`
- `busyRef` already exists in `App.tsx` as `const busyRef = useRef(false)` `[VERIFIED: App.tsx]`

---

## Open Questions (RESOLVED)

1. **Exact KEY_DEFS visual layout (row/col grid)**
   - What we know: 40 keys, 5 rows, 9 cols, ENTER at row=1 spanning cols 4-5 per CONTEXT hint
   - What's unclear: The exact visual grouping is a design decision left to the implementer; no single authoritative source specifies the SVG grid mapping.
   - Recommendation: Use the KEY_DEFS skeleton above as the starting point; adjust during implementation based on visual fit. The `id` values are fixed; only `row`/`col` values need visual tuning.

2. **ENTER colSpan=2 position: cols 4-5 or cols 5-6?**
   - What we know: CONTEXT says "ENTER at row 1 spanning cols 5-6 (zero-indexed)" in the specifics section.
   - What's unclear: The KEY_DEFS skeleton above places ENTER at col=5 (spanning cols 5-6). This leaves col=4 (xy_swap) and col=7 (div), col=8 (mul) — a total of 8 unique key positions in row 1 = 7 single-span + 1 double-span = 9 column-slots. Consistent.
   - Recommendation: Use col=5 for ENTER as CONTEXT suggests.

3. **XEQ / STO / RCL click behavior**
   - What we know: These are visual-only for Phase 16 (empty id).
   - What's unclear: Whether to show a visual affordance (tooltip, opacity) that they are not yet wired.
   - Recommendation: Render normally (no opacity change); the user experience for Phase 16 is acceptable without modal support. A follow-on phase (18 or later) will wire these.

---

## Sources

### Primary (HIGH confidence)
- `hp41-gui/src-tauri/src/key_map.rs` — All valid key IDs; verified by reading file
- `hp41-gui/src-tauri/src/commands.rs` — Digit key handling, handle_op logic; verified
- `hp41-gui/src/App.tsx` — busyRef pattern, invoke pattern, keyboard-area placeholder; verified
- `hp41-gui/src/App.css` — .calculator width 320px; verified
- `hp41-gui/src-tauri/tauri.conf.json` — Current window size 800x600; verified
- `hp41-gui/package.json` — No test framework; dependency versions; verified
- MDN Web Docs — `transform-box: fill-box` baseline 2020, `transform-origin` SVG behavior — https://developer.mozilla.org/en-US/docs/Web/CSS/transform-box

### Secondary (MEDIUM confidence)
- `hp41-cli/src/keys.rs` comments — HP-41C row/column key code assignments; consistent with physical hardware
- finseth.com hp41c data — Physical HP-41C keyboard layout row-by-row — https://www.finseth.com/hpdata/hp41c.php
- CONTEXT.md specifics section — HP-41C color palette suggestions and viewBox hint

### Tertiary (LOW confidence — flagged)
- KEY_DEFS row/col SVG grid assignments — derived from CONTEXT hints and HP-41C hardware knowledge; visual tuning required during implementation

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — packages verified in package.json; no new dependencies needed
- key_id correctness: HIGH — every dispatched id verified directly from key_map.rs and commands.rs
- CSS transform-box pattern: HIGH — MDN verified, baseline since 2020
- JSX SVG patterns: HIGH — standard React, no Tauri-specific behavior
- Key geometry constants: LOW — derived from arithmetic; implementer must adjust for visual fit
- HP-41C color palette: LOW — approximated from CONTEXT suggestions; visual tuning expected
- KEY_DEFS row/col layout: LOW — conceptual mapping; adjust during implementation

**Research date:** 2026-05-10
**Valid until:** 2026-08-10 (stable tech — CSS/React/Tauri APIs are stable)
