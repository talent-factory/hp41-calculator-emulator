# Phase 16: SVG Skin - Pattern Map

**Mapped:** 2026-05-10
**Files analyzed:** 5 (1 new, 4 modified)
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src/Keyboard.tsx` | component | request-response | `hp41-gui/src/App.tsx` | role-match (same IPC + useState + useRef patterns) |
| `hp41-gui/src/App.tsx` | component | request-response | `hp41-gui/src/App.tsx` (self) | exact (integration point: add import + callback + replace placeholder) |
| `hp41-gui/src/App.css` | config | — | `hp41-gui/src/App.css` (self) | exact (add `.key` / `.key-pressed` rules; update width) |
| `hp41-gui/src-tauri/tauri.conf.json` | config | — | `hp41-gui/src-tauri/tauri.conf.json` (self) | exact (field-level update: width, height, resizable) |
| `hp41-gui/src-tauri/src/key_map.rs` | test | — | `hp41-gui/src-tauri/src/key_map.rs` (self) | exact (copy test module pattern; add Wave 0 test) |

---

## Pattern Assignments

### `hp41-gui/src/Keyboard.tsx` (NEW — component, request-response)

**Analog:** `hp41-gui/src/App.tsx`

**Imports pattern** (App.tsx lines 1–3):
```typescript
import { useState, useCallback, useRef } from 'react';
// Keyboard.tsx only needs useState (pressedKey) — no useCallback/useRef needed
// because busyRef is passed in as a prop and onKey is a stable prop function.
import { useState } from 'react';
// No @tauri-apps/api import in Keyboard.tsx — invoke() lives in App.tsx
```

**Props interface pattern** — modeled on CalcStateView interface style (App.tsx lines 5–23):
```typescript
// In Keyboard.tsx — new interface, same style as App.tsx interface blocks
interface KeyboardProps {
  onKey: (keyId: string) => void;
  busyRef: React.MutableRefObject<boolean>;
}
```

**busyRef debounce pattern** (App.tsx lines 64–76 — copy guard logic):
```typescript
// In App.tsx handleKey (the established debounce + IPC pattern):
const handleKey = useCallback((e: KeyboardEvent) => {
  if (e.repeat) return;
  if (busyRef.current) return;   // <-- copy this guard into Keyboard.tsx handleKeyClick
  // ...
  busyRef.current = true;
  invoke<CalcStateView>('dispatch_op', { keyId })
    .then(view => setCalcState(view))
    .catch(err => console.error('dispatch_op error:', err))
    .finally(() => { busyRef.current = false; });
}, [calcState]);
```
In `Keyboard.tsx`, `busyRef.current` is checked but NOT set — the caller (`App.tsx handleClick`) owns the `true`/`false` lifecycle, per D-13.

**Core component pattern** (App.tsx lines 52–119 — structure and JSX style):
```typescript
// Named export (not default) — consistent with future barrel imports
export function Keyboard({ onKey, busyRef }: KeyboardProps) {
  const [pressedKey, setPressedKey] = useState<string | null>(null);

  const handleKeyClick = (keyId: string) => {
    if (!keyId) return;                    // visual-only key guard (D-07, Pitfall 2)
    if (busyRef.current) return;           // debounce (D-13)
    setPressedKey(keyId);
    // Functional update avoids stale closure clearing wrong key (Pitfall 4):
    setTimeout(() => setPressedKey(prev => prev === keyId ? null : prev), 150);
    onKey(keyId);
  };

  return (
    <svg width="100%" viewBox="0 0 400 230" xmlns="http://www.w3.org/2000/svg"
         aria-label="HP-41C keyboard">
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

**KEY_DEFS constant pattern** — inline at top of file per D-02:
```typescript
// Geometry constants (computed for 400px container — adjust during implementation)
const KEY_W    = 39;   // single-span key width
const KEY_H    = 26;   // key body height
const GAP      = 4;    // gap between keys
const PAD      = 8;    // left/right/top SVG padding
const FSHIFT_H = 12;   // f-shift label zone height above each key body

type KeyDef = {
  id: string;           // key_id for dispatch_op; '' = visual-only (no dispatch)
  label: string;        // primary label (white, on key face)
  fShiftLabel?: string; // gold label above key; omit when no f-shift function
  row: number;          // 0-indexed SVG row (0 = top)
  col: number;          // 0-indexed SVG column (left edge of key)
  colSpan?: number;     // default 1; ENTER = 2
};

const KEY_DEFS: KeyDef[] = [
  // Row 0
  { id: 'sigma_plus',   label: 'Σ+',    fShiftLabel: 'x²',   row: 0, col: 0 },
  { id: 'recip',        label: '1/x',   fShiftLabel: 'yˣ',   row: 0, col: 1 },
  { id: 'sqrt',         label: '√x',                         row: 0, col: 2 },
  { id: 'log',          label: 'LOG',   fShiftLabel: '10ˣ',  row: 0, col: 3 },
  { id: 'ln',           label: 'LN',    fShiftLabel: 'eˣ',   row: 0, col: 4 },
  { id: '',             label: 'XEQ',                        row: 0, col: 5 },
  { id: '',             label: 'STO',                        row: 0, col: 6 },
  { id: '',             label: 'RCL',                        row: 0, col: 7 },
  { id: 'clx',          label: '←',                          row: 0, col: 8 },
  // Row 1
  { id: 'sin',          label: 'SIN',   fShiftLabel: 'ASIN', row: 1, col: 0 },
  { id: 'cos',          label: 'COS',   fShiftLabel: 'ACOS', row: 1, col: 1 },
  { id: 'tan',          label: 'TAN',   fShiftLabel: 'ATAN', row: 1, col: 2 },
  { id: 'rdn',          label: 'R↓',                        row: 1, col: 3 },
  { id: 'xy_swap',      label: 'x<>y',                      row: 1, col: 4 },
  { id: 'enter',        label: 'ENTER',                      row: 1, col: 5, colSpan: 2 },
  { id: 'div',          label: '÷',                          row: 1, col: 7 },
  { id: 'mul',          label: '×',                          row: 1, col: 8 },
  // Row 2
  { id: 'user_mode',    label: 'USER',                       row: 2, col: 0 },
  { id: '',             label: 'f',                          row: 2, col: 1 },
  { id: '',             label: 'g',                          row: 2, col: 2 },
  { id: '7',            label: '7',                          row: 2, col: 3 },
  { id: '8',            label: '8',                          row: 2, col: 4 },
  { id: '9',            label: '9',                          row: 2, col: 5 },
  { id: 'minus',        label: '−',                          row: 2, col: 6 },
  { id: 'prgm_mode',    label: 'PRGM',                       row: 2, col: 7 },
  { id: 'alpha_toggle', label: 'ALPHA',                      row: 2, col: 8 },
  // Row 3
  { id: 'chs',          label: 'CHS',                        row: 3, col: 0 },
  { id: 'e',            label: 'EEX',                        row: 3, col: 1 },
  { id: '',             label: 'SST',                        row: 3, col: 2 },
  { id: '4',            label: '4',                          row: 3, col: 3 },
  { id: '5',            label: '5',                          row: 3, col: 4 },
  { id: '6',            label: '6',                          row: 3, col: 5 },
  { id: 'plus',         label: '+',                          row: 3, col: 6 },
  { id: '',             label: 'GTO',                        row: 3, col: 7 },
  { id: '',             label: 'R/S',                        row: 3, col: 8 },
  // Row 4
  { id: '0',            label: '0',                          row: 4, col: 0 },
  { id: '.',            label: '.',                          row: 4, col: 1 },
  { id: '',             label: 'ON',                         row: 4, col: 2 },
  { id: '1',            label: '1',                          row: 4, col: 3 },
  { id: '2',            label: '2',                          row: 4, col: 4 },
  { id: '3',            label: '3',                          row: 4, col: 5 },
  { id: 'lastx',        label: 'LSTx',                       row: 4, col: 6 },
  { id: 'clreg',        label: 'CLRG',                       row: 4, col: 7 },
  { id: '',             label: 'BST',                        row: 4, col: 8 },
] as const;
```

---

### `hp41-gui/src/App.tsx` (MODIFY — component, request-response)

**Analog:** `hp41-gui/src/App.tsx` (self — 3 surgical additions)

**Change 1 — Add import** (after line 3):
```typescript
import { Keyboard } from './Keyboard';
```

**Change 2 — Add handleClick callback** (after the handleKey useCallback, ~line 76):
```typescript
// Mirrors handleKey pattern (lines 64-76) but for mouse clicks — no e.repeat guard needed
const handleClick = useCallback((keyId: string) => {
  if (busyRef.current) return;
  busyRef.current = true;
  invoke<CalcStateView>('dispatch_op', { keyId })
    .then(view => setCalcState(view))
    .catch(err => console.error('dispatch_op error:', err))
    .finally(() => { busyRef.current = false; });
}, []);  // no calcState dep — no eex_chs context for mouse clicks
```

**Change 3 — Replace placeholder** (line 118):
```typescript
// BEFORE (line 118):
<div id="keyboard-area" />

// AFTER:
<Keyboard onKey={handleClick} busyRef={busyRef} />
```

**Context: busyRef already exists** (App.tsx line 54):
```typescript
const busyRef = useRef(false);  // already declared — no change needed
```

---

### `hp41-gui/src/App.css` (MODIFY — config)

**Analog:** `hp41-gui/src/App.css` (self)

**Change 1 — Update .calculator width** (lines 3–13, change line 5):
```css
/* BEFORE (line 5): */
width: 320px;

/* AFTER: */
width: 392px;  /* 400px window; 4px margin each side avoids overflow (Pitfall 5) */
```

**Change 2 — Replace/update placeholder rule** (lines 73–75):
```css
/* BEFORE — Phase 16 placeholder (lines 73-75): */
#keyboard-area {
  min-height: 0;
}

/* AFTER — remove placeholder, add keyboard animation rules: */
.key {
  cursor: pointer;
  transform-box: fill-box;    /* REQUIRED: makes transform-origin relative to element's own box */
  transform-origin: center;   /* center of the KEY's bounding box, not the SVG canvas */
  transition: transform 80ms ease-out;
}

.key-pressed {
  transform: scale(0.92);
}

.key:hover:not(.key-pressed) {
  opacity: 0.85;
}
```

**Critical CSS note:** `transform-box: fill-box` is mandatory. Without it, `scale(0.92)` on SVG `<g>` elements animates from the SVG canvas origin (0,0 of the viewBox), causing all keys to shrink toward the top-left corner of the SVG instead of in-place. Baseline widely available since January 2020; supported in all Tauri-targeted WebViews.

---

### `hp41-gui/src-tauri/tauri.conf.json` (MODIFY — config)

**Analog:** `hp41-gui/src-tauri/tauri.conf.json` (self)

**Current state** (lines 12–19 — verified):
```json
"windows": [
  {
    "title": "HP-41 Calculator",
    "width": 800,
    "height": 600,
    "resizable": true,
    "decorations": true
  }
]
```

**Target state** (change width, height, resizable only — all other fields unchanged):
```json
"windows": [
  {
    "title": "HP-41 Calculator",
    "width": 400,
    "height": 700,
    "resizable": false,
    "decorations": true
  }
]
```

---

### `hp41-gui/src-tauri/src/key_map.rs` (MODIFY — test)

**Analog:** `hp41-gui/src-tauri/src/key_map.rs` (self — add test to existing `#[cfg(test)]` module)

**Existing test module pattern** (lines 195–246 — copy `#[test]` structure exactly):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_key_map_named_ops() {
        assert_eq!(resolve("plus").unwrap(), Op::Add);
        // ...
    }

    #[test]
    fn test_key_map_unknown_key() {
        let err = resolve("totally_unknown_xyz").unwrap_err();
        assert!(err.message.contains("unknown key"), "...");
    }
}
```

**New test to add inside the existing `mod tests` block** (after line 245, before closing `}`):
```rust
    #[test]
    fn test_all_keyboard_skin_ids_are_valid() {
        // IDs from KEY_DEFS that resolve through key_map::resolve() (named ops only).
        // Digit keys ("0"-"9", ".", "e") are handled by handle_op() digit branch — not tested here.
        // Empty-string ids are visual-only — never sent to dispatch_op.
        let named_ids = [
            "sigma_plus", "recip", "log", "ln",
            "sin", "cos", "tan", "rdn", "xy_swap",
            "enter", "div", "mul",
            "user_mode", "minus", "prgm_mode", "alpha_toggle",
            "chs", "plus",
            "lastx", "clreg", "clx",
        ];
        for id in named_ids {
            assert!(
                resolve(id).is_ok(),
                "key_map::resolve({id:?}) must succeed for a KEY_DEFS id"
            );
        }
    }
```

**Note:** The test module already has `#[allow(clippy::unwrap_used)]` at the module level (line 196), so `.unwrap()` / `assert!(...)` inside the new test is fine. Do NOT add a second `#[allow]` attribute.

---

## Shared Patterns

### IPC invoke Pattern
**Source:** `hp41-gui/src/App.tsx` lines 71–75
**Apply to:** `Keyboard.tsx` (via `onKey` prop callback) and `App.tsx handleClick`
```typescript
busyRef.current = true;
invoke<CalcStateView>('dispatch_op', { keyId })
  .then(view => setCalcState(view))
  .catch(err => console.error('dispatch_op error:', err))
  .finally(() => { busyRef.current = false; });
```
The IPC pattern always: sets `busyRef.current = true` before, clears in `.finally()`. The `.catch` logs to console (never swallowed silently).

### busyRef Debounce Guard
**Source:** `hp41-gui/src/App.tsx` lines 65–67
**Apply to:** `Keyboard.tsx handleKeyClick` AND `App.tsx handleClick`
```typescript
if (busyRef.current) return;
```
This single-line guard appears at the top of every function that calls `invoke`. `Keyboard.tsx` checks it but does NOT own the `true`/`false` flip — `App.tsx handleClick` does.

### TypeScript Interface Style
**Source:** `hp41-gui/src/App.tsx` lines 5–23
**Apply to:** `Keyboard.tsx` KeyDef type and KeyboardProps interface
```typescript
// Plain interface keyword, no `export` unless needed by consumers.
// PascalCase names, readonly fields not used (plain mutable interfaces).
interface KeyboardProps {
  onKey: (keyId: string) => void;
  busyRef: React.MutableRefObject<boolean>;
}
```

### Rust Test Module Style
**Source:** `hp41-gui/src-tauri/src/key_map.rs` lines 195–246
**Apply to:** New `test_all_keyboard_skin_ids_are_valid` test
```rust
// Module-level #[allow(clippy::unwrap_used)] already present — do not duplicate.
// Each test: one concern, descriptive name, inline assert message.
assert!(condition, "descriptive failure message including the offending value");
```

---

## No Analog Found

All files have close analogs. No entries in this section.

---

## Metadata

**Analog search scope:** `hp41-gui/src/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src-tauri/`
**Files read:** 5 (App.tsx, App.css, tauri.conf.json, key_map.rs, CONTEXT.md, RESEARCH.md)
**Pattern extraction date:** 2026-05-10
