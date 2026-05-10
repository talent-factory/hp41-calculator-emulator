# Phase 16: SVG Skin - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 16-SVG Skin
**Areas discussed:** SVG authoring method, Key label depth, Component integration, Click handler pattern

---

## SVG Authoring Method

| Option | Description | Selected |
|--------|-------------|----------|
| Data-driven loop | KEY_DEFS array with id/label/row/col/colSpan; positions computed from constants | ✓ |
| Hardcoded static positions | Each key has hand-tuned x/y coordinates — max fidelity for irregular keys | |
| You decide | Claude picks most maintainable approach | |

**User's choice:** Data-driven loop

| Option | Description | Selected |
|--------|-------------|----------|
| Inline in Keyboard component | KEY_DEFS as typed const at top of Keyboard.tsx | ✓ |
| Separate keys.ts data file | hp41-gui/src/keys.ts exports KEY_DEFS | |
| You decide | Claude places it wherever cleanest | |

**User's choice:** Inline in Keyboard component

| Option | Description | Selected |
|--------|-------------|----------|
| colSpan field in KEY_DEFS | colSpan: 1 \| 2; width = colSpan * KEY_W + (colSpan-1) * GAP | ✓ |
| Explicit width override per key | Optional width?: number overrides computed width | |
| You decide | Claude picks cleanest approach | |

**User's choice:** colSpan field in KEY_DEFS

| Option | Description | Selected |
|--------|-------------|----------|
| JSX SVG elements | React renders svg/rect/text as JSX; onClick/className work natively | ✓ |
| Raw SVG string | Template literal injected into DOM; needs event delegation | |
| You decide | Claude picks right approach for stack | |

**User's choice:** JSX SVG elements

---

## Key Label Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Primary + gold f-shift legends | White primary on key face + gold text above for f-shift | ✓ |
| Primary label only | White text on key cap only; simpler but loses HP-41C identity | |
| Primary + f-shift + g-shift (blue) | All three rows; ~3× more SVG text elements | |

**User's choice:** Primary + gold f-shift legends

| Option | Description | Selected |
|--------|-------------|----------|
| All 40 keys with labels | KEY_DEFS defines all 40 HP-41C keys; SC-1 requires this | ✓ |
| Subset first, extend in same phase | Start with 10-15 keys to validate layout | |
| You decide | Claude decides based on implementation risk | |

**User's choice:** All 40 keys with labels

| Option | Description | Selected |
|--------|-------------|----------|
| Empty / no text | No gold text element for keys without f-shift; clean | ✓ |
| Show a placeholder or dash | Faint dash or nothing visible | |
| You decide | Claude decides most authentic | |

**User's choice:** Empty / no text

---

## Component Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone \<Keyboard /\> component | Keyboard.tsx with onKey/busyRef props; App.tsx replaces placeholder | ✓ |
| Inline SVG in App.tsx | No new file; SVG markup added directly in App() return | |
| You decide | Claude picks based on component size | |

**User's choice:** Standalone \<Keyboard /\> component in Keyboard.tsx

| Option | Description | Selected |
|--------|-------------|----------|
| Grow calculator to 400px wide | Update App.css from 320px to 400px; aligns with SC-5 | ✓ |
| Keep 320px, SVG scales to fit | SVG scales proportionally; narrower than roadmap target | |
| Side-by-side layout | Display left, keyboard right; larger layout refactor | |

**User's choice:** Grow the calculator to 400px wide

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — update tauri.conf.json to 400×700 | SC-5 requires it; width:400, height:700 in windows[] | ✓ |
| Defer window resize to later phase | Risk: SC-5 becomes a fail | |
| You decide | Claude decides if SC-5 requires it now | |

**User's choice:** Yes — update tauri.conf.json to 400×700 in Phase 16

---

## Click Handler Pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Individual onClick on each key \<g\> | onClick={() => onKey(key.id)} per key; idiomatic React JSX | ✓ |
| Delegated handler on SVG root | Single onClick on \<svg\> reads data-key-id via DOM traversal | |
| You decide | Claude picks cleanest integration | |

**User's choice:** Individual onClick on each key \<g\> element

| Option | Description | Selected |
|--------|-------------|----------|
| CSS class toggled via React state | pressedKey useState; setTimeout(150ms) clears; .key-pressed { scale(0.92) } | ✓ |
| CSS :active pseudo-class only | No JS state; fires on mousedown, releases on mouseup | |
| You decide | Claude picks approach matching SC-4 150ms requirement | |

**User's choice:** CSS class toggled via React state

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — pass busyRef down to Keyboard | Keyboard checks busyRef.current before invoking; prevents race condition | ✓ |
| No — separate debounce inside Keyboard | Independent useRef in Keyboard; risk of simultaneous keyboard+mouse race | |
| You decide | Claude decides safest debounce strategy | |

**User's choice:** Yes — pass the busyRef down to Keyboard

---

## Claude's Discretion

None — user selected recommended options for all decisions.

## Deferred Ideas

None — discussion stayed within phase scope.
