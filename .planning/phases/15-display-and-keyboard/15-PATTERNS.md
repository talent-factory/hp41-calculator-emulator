# Phase 15: Display & Keyboard — Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 6 (4 modified, 2 read-only reference)
**Analogs found:** 4 / 4 modifiable files (read-only refs need no analog)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/src/types.rs` | model/DTO | request-response | `hp41-gui/src-tauri/src/types.rs` (extend in-place) | self |
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | `hp41-gui/src-tauri/src/commands.rs` (extend in-place) | self |
| `hp41-gui/src/App.tsx` | component | event-driven | `hp41-gui/src/main.tsx` (React scaffold) | partial |
| `hp41-gui/src/index.css` | config | — | `hp41-gui/src/index.css` (extend in-place) | self |
| `hp41-cli/src/keys.rs` | utility | — | — (read-only reference) | n/a |
| `hp41-core/src/format.rs` | utility | transform | — (read-only reference) | n/a |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/types.rs` (model/DTO, request-response)

**Analog:** `hp41-gui/src-tauri/src/types.rs` (extend the existing struct in-place)

**Existing imports pattern** (lines 12–13):
```rust
use hp41_core::{format_alpha, format_hpnum, AngleMode, CalcState, HpError};
use serde::Serialize;
```
No import changes needed — `format_hpnum` and `CalcState` are already imported.

**Existing struct pattern** (lines 24–30 — current shape before Phase 15):
```rust
#[derive(Debug, Serialize)]
pub struct CalcStateView {
    pub display_str: String,
    pub x_str: String,
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>,
}
```

**Extended struct pattern** (D-01, D-02 — what Phase 15 must produce):
```rust
#[derive(Debug, Serialize)]
pub struct CalcStateView {
    pub display_str: String,
    pub x_str: String,
    pub y_str: String,      // Phase 15: D-01
    pub z_str: String,      // Phase 15: D-01
    pub t_str: String,      // Phase 15: D-01
    pub lastx_str: String,  // Phase 15: D-01
    pub in_eex_mode: bool,  // Phase 15: D-02
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>,
}
```

**Existing from_state() pattern** (lines 37–68 — copy the x_str call 4 more times):
```rust
// x_str is always the formatted X register — independent of entry/alpha mode.
let x_str = format_hpnum(&state.stack.x, &state.display_mode);
```
Stack field names: `state.stack.x`, `state.stack.y`, `state.stack.z`, `state.stack.t`, `state.stack.lastx`
All are `pub HpNum` — same call signature for each.

**New from_state() lines to add** (after line 52, before annunciators block):
```rust
let y_str = format_hpnum(&state.stack.y, &state.display_mode);
let z_str = format_hpnum(&state.stack.z, &state.display_mode);
let t_str = format_hpnum(&state.stack.t, &state.display_mode);
let lastx_str = format_hpnum(&state.stack.lastx, &state.display_mode);
let in_eex_mode = state.entry_buf.contains('e');
```

**CalcStateView constructor update** (lines 62–67 — add new fields to struct literal):
```rust
CalcStateView {
    display_str,
    x_str,
    y_str,
    z_str,
    t_str,
    lastx_str,
    in_eex_mode,
    annunciators,
    print_lines,
}
```

**Test extension pattern** (lines 86–133 — copy existing test style):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::CalcState;

    #[test]
    fn test_in_eex_mode_true_when_entry_buf_contains_e() {
        let mut state = CalcState::new();
        state.entry_buf = "1e".to_string();
        let view = CalcStateView::from_state(&state, vec![]);
        assert!(view.in_eex_mode);
    }

    #[test]
    fn test_in_eex_mode_false_without_e() {
        let mut state = CalcState::new();
        state.entry_buf = "42".to_string();
        let view = CalcStateView::from_state(&state, vec![]);
        assert!(!view.in_eex_mode);
    }

    #[test]
    fn test_dispatch_op_payload_size() {
        // Existing test — must still pass after adding 4 strings + 1 bool.
        // SC-1 gate: CalcStateView JSON must be <=300 bytes.
        let state = CalcState::new();
        let view = CalcStateView::from_state(&state, vec![]);
        let json = serde_json::to_string(&view).unwrap();
        assert!(
            json.len() <= 300,
            "CalcStateView JSON must be <=300 bytes, got {} bytes: {}",
            json.len(),
            json
        );
    }
}
```

---

### `hp41-gui/src-tauri/src/commands.rs` (controller, request-response)

**Analog:** `hp41-gui/src-tauri/src/commands.rs` (add one branch to handle_op())

**Existing imports** (lines 18–23 — no changes needed):
```rust
use crate::key_map;
use crate::types::{CalcStateView, GuiError};
use crate::AppState;
use hp41_core::ops::dispatch;
use hp41_core::CalcState;
use tauri::State;
```

**Existing handle_op routing structure** (lines 54–107 — the branch-per-key-type pattern):
The function has three consecutive `if key_id == ...` early-return blocks for special cases
(digits lines 55–78, "." lines 80–87, "e" lines 89–100), then falls through to
`key_map::resolve()` at line 103. The "eex_chs" branch slots in AFTER the "e" block at
line 100, BEFORE the `key_map::resolve()` call at line 103.

**drain-and-return pattern** (used 3 times already — copy exactly):
```rust
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
return Ok(CalcStateView::from_state(calc, print_lines));
```

**New "eex_chs" branch** (insert at line 101, between "e" block and key_map::resolve):
```rust
// ── "eex_chs" — toggle exponent sign in entry_buf (Phase 15 D-06) ────────
// Source pattern: hp41-cli/src/app.rs lines 389-404 (verified)
// MUST come before key_map::resolve() — "eex_chs" has no Op variant.
if key_id == "eex_chs" {
    if let Some(e_pos) = calc.entry_buf.find('e') {
        let after_e = &calc.entry_buf[e_pos + 1..];
        if after_e.starts_with('-') {
            // Remove minus: "1e-2" -> "1e2", "1e-" -> "1e"
            calc.entry_buf.remove(e_pos + 1);
        } else {
            // Insert minus: "1e2" -> "1e-2", "1e" -> "1e-"
            calc.entry_buf.insert(e_pos + 1, '-');
        }
    }
    // No-op if entry_buf has no 'e' — React guards this but Rust is defensive
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    return Ok(CalcStateView::from_state(calc, print_lines));
}
```
Source analog: `hp41-cli/src/app.rs` lines 389–404 (verbatim match — toggle logic and
return-without-dispatch are identical to the CLI's EEX-CHS handling).

**Zero-panic pattern** (line 35 — all Mutex locks use poisoned-lock recovery):
```rust
let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
```

**Test pattern** (lines 118–167 — copy the #[allow] and helper-call style):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_eex_chs_toggles_exponent_sign() {
        let mut calc = CalcState::new();
        calc.entry_buf = "1e2".to_string();
        handle_op(&mut calc, "eex_chs").unwrap();
        assert_eq!(calc.entry_buf, "1e-2");
        handle_op(&mut calc, "eex_chs").unwrap();
        assert_eq!(calc.entry_buf, "1e2");
    }

    #[test]
    fn test_eex_chs_noop_without_e() {
        let mut calc = CalcState::new();
        calc.entry_buf = "42".to_string();
        let result = handle_op(&mut calc, "eex_chs");
        assert!(result.is_ok(), "eex_chs with no 'e' must not panic or error");
        assert_eq!(calc.entry_buf, "42", "entry_buf must be unchanged");
    }
}
```

---

### `hp41-gui/src/App.tsx` (component, event-driven)

**Analog:** `hp41-gui/src/main.tsx` (the only existing React file — partial match, same import style)

**main.tsx import pattern** (lines 1–4 — follow the same ESM sub-path convention):
```typescript
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'
```

**App.tsx must follow the same ESM import style** (D-10: no new deps):
```typescript
import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
```
Note: `@tauri-apps/api/core` is the Tauri v2 sub-path (verified from installed node_modules).
Do NOT use `@tauri-apps/api` (top-level) — that is the Tauri v1 import path.

**TypeScript interface pattern** (inline manual types — tauri-specta deferred per CONTEXT.md):
```typescript
interface Annunciators {
  user: boolean;
  prgm: boolean;
  alpha: boolean;
  rad: boolean;
  grad: boolean;
}

interface CalcStateView {
  display_str: string;
  x_str: string;
  y_str: string;
  z_str: string;
  t_str: string;
  lastx_str: string;
  in_eex_mode: boolean;
  annunciators: Annunciators;
  print_lines: string[];
}
```
Field names are snake_case — Tauri v2 serializes Rust structs with the same casing as the
Rust field names (serde default). Do NOT use camelCase field names in the interface.

**useState pattern** (D-11 — null initial state, populated on mount):
```typescript
const [calcState, setCalcState] = useState<CalcStateView | null>(null);
```

**Mount effect pattern** (D-11 — invoke get_state on mount, no polling):
```typescript
useEffect(() => {
  invoke<CalcStateView>('get_state')
    .then(view => setCalcState(view))
    .catch(err => console.error('get_state error:', err));
}, []); // empty deps — runs once on mount
```

**Keyboard listener pattern** (D-07/D-12 — useCallback + useEffect with cleanup, busyRef):
```typescript
const busyRef = useRef(false);

const handleKey = useCallback((e: KeyboardEvent) => {
  if (busyRef.current) return;              // debounce: ignore while invoke pending
  const keyId = resolveKeyId(e, calcState); // returns null for ignored keys
  if (keyId === null) return;
  e.preventDefault();
  busyRef.current = true;
  invoke<CalcStateView>('dispatch_op', { keyId })
    .then(view => setCalcState(view))
    .catch(err => console.error('dispatch_op error:', err))
    .finally(() => { busyRef.current = false; });
}, [calcState]); // calcState needed to read in_eex_mode for 'n' key

useEffect(() => {
  window.addEventListener('keydown', handleKey);
  return () => window.removeEventListener('keydown', handleKey); // cleanup — required for StrictMode
}, [handleKey]);
```
`busyRef` is a `useRef(false)` — not `useState` — because changes to it must not trigger
re-renders. The effect re-registers whenever `handleKey` changes (i.e., when `calcState`
changes), which is correct: the 'n' key decision reads the latest `calcState.in_eex_mode`.

**EEX-CHS key resolver function** (D-06 — resolves 'n' based on in_eex_mode):
```typescript
function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
  // EEX-CHS: 'n' routes to "eex_chs" when in EEX entry mode (D-06)
  if (e.key === 'n') return state?.in_eex_mode ? 'eex_chs' : 'chs';
  // Digit entry
  if (e.key.length === 1 && '0123456789'.includes(e.key)) return e.key;
  if (e.key === '.') return '.';
  if (e.key === 'e') return 'e';
  // Modal-trigger keys — silently ignore, no invoke (D-05)
  if (e.key.length === 1 && 'SRfFPX'.includes(e.key)) return null;
  // Named op mapping (authoritative source: hp41-cli/src/keys.rs key_to_op())
  const MAP: Record<string, string> = {
    'Enter': 'enter', 'Backspace': 'clx',
    '+': 'plus', '-': 'minus', '*': 'mul', '/': 'div',
    'r': 'rdn', 'x': 'xy_swap', 'l': 'lastx', 's': 'sqrt', 'p': 'prgm_mode',
    'a': 'asin', 'c': 'acos', 'k': 'atan',
    'C': 'cos', 'T': 'tan', 'L': 'ln', 'G': 'log', 'E': 'exp',
    'H': 'tenpow', 'I': 'recip', 'W': 'sq', 'Y': 'ypow',
    'u': 'user_mode',
    'z': 'sigma_plus', 'Z': 'sigma_minus', 'm': 'mean', 'D': 'sdev',
    'y': 'yhat', 'b': 'lr', 'O': 'corr', 'V': 'cl_sigma_stat',
    'h': 'hms_to_h', 'j': 'hms_add', 'J': 'hms_sub',
    'q': 'sin',    // Phase 8 reassignment: 'q' = SIN
    'g': 'clreg',  // Phase 8 addition: 'g' = CLREG
  };
  return MAP[e.key] ?? null;
}
```
Key names match `e.key` from KeyboardEvent: `'Enter'`, `'Backspace'`, `'+'`, `'C'` (Shift+C
yields `'C'` directly — same convention as crossterm's `KeyCode::Char('C')`).

**Invoke argument naming** (RESEARCH.md Pitfall 4):
```typescript
invoke<CalcStateView>('dispatch_op', { keyId })
// Tauri v2 converts JS camelCase { keyId } to Rust snake_case param key_id automatically.
// Command signature in commands.rs: pub fn dispatch_op(key_id: &str, ...)
```

**JSX render pattern** (D-08/D-09 — vertical stack layout):
```typescript
if (!calcState) return <div className="calculator">Loading...</div>;

return (
  <div className="calculator">
    {/* Annunciator row — D-09: always visible, dim/bright via CSS class */}
    <div className="annunciators">
      {(['user', 'prgm', 'alpha', 'rad', 'grad'] as const).map(name => (
        <span
          key={name}
          className={`annunciator${calcState.annunciators[name] ? ' active' : ''}`}
        >
          {name.toUpperCase()}
        </span>
      ))}
    </div>
    {/* Display row — D-08: 12-char string, monospace, dark bg */}
    <div className="display">{calcState.display_str}</div>
    {/* Stack panel — D-08: X/Y/Z/T/L labeled rows */}
    <div className="stack-panel">
      {([
        ['X', calcState.x_str],
        ['Y', calcState.y_str],
        ['Z', calcState.z_str],
        ['T', calcState.t_str],
        ['L', calcState.lastx_str],
      ] as const).map(([label, value]) => (
        <div key={label} className="stack-row">
          <span className="stack-label">{label}:</span>
          <span>{value}</span>
        </div>
      ))}
    </div>
    {/* Phase 16 SVG keyboard placeholder — D-08 */}
    <div id="keyboard-area" />
  </div>
);
```
All string fields from CalcStateView (`display_str`, `x_str`, etc.) are rendered as React
text nodes — this is the safe JSX default and prevents any injection risk from calculator
output values.

---

### `hp41-gui/src/index.css` (config)

**Analog:** `hp41-gui/src/index.css` (modify in-place) + `hp41-gui/src/main.tsx` (CSS import pattern)

**Current state** (line 1 — the only content):
```css
@import "tailwindcss";
```

**Required action** (RESEARCH.md Pitfall 1 — strip Tailwind, add plain reset):
Remove `@import "tailwindcss"` and replace with a minimal box-sizing reset only:
```css
*, *::before, *::after {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
}
```

**vite.config.ts must also lose the Tailwind plugin** (lines 3 and 6 of vite.config.ts):
```typescript
// REMOVE this import (line 3):
import tailwindcss from '@tailwindcss/vite'

// REMOVE tailwindcss() from plugins array (line 6), keep react():
plugins: [react()],
```
Do NOT remove tailwindcss from package.json — Phase 16 may reconsider D-10.

**New App.css pattern** (D-10 — new file, vanilla CSS, dark calculator aesthetic):
```css
/* src/App.css — Phase 15 calculator display styles */

.calculator {
  display: flex;
  flex-direction: column;
  width: 320px;
  background: #0d0d0d;
  border: 1px solid #333;
  border-radius: 4px;
  overflow: hidden;
  font-family: system-ui, sans-serif;
}

/* Annunciator row — D-09: always visible, dim=inactive, bright=active */
.annunciators {
  display: flex;
  gap: 8px;
  padding: 4px 8px;
  background: #1a1a1a;
}

.annunciator {
  font-size: 11px;
  letter-spacing: 0.05em;
  color: #555;
  opacity: 0.35;
  text-transform: uppercase;
}

.annunciator.active {
  color: #e8e8c0;
  opacity: 1.0;
}

/* Display row — D-08: right-aligned monospace on dark background */
.display {
  background: #111;
  padding: 6px 10px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 22px;
  text-align: right;
  color: #c8e6c9;
  letter-spacing: 0.05em;
  min-height: 2em;
  border-bottom: 1px solid #222;
}

/* Stack panel — D-08: labeled rows for X/Y/Z/T/L */
.stack-panel {
  background: #1a1a1a;
  padding: 6px 10px;
}

.stack-row {
  display: flex;
  justify-content: space-between;
  font-family: 'Courier New', Courier, monospace;
  font-size: 13px;
  color: #aaa;
  padding: 2px 0;
}

.stack-label {
  color: #666;
  min-width: 20px;
}

/* Phase 16 SVG keyboard placeholder */
#keyboard-area {
  min-height: 0;
}
```

---

## Shared Patterns

### Zero-Panic Policy (Rust)

**Source:** `hp41-gui/src-tauri/src/lib.rs` line 1
**Apply to:** All new Rust code in types.rs and commands.rs

```rust
#![deny(clippy::unwrap_used)]
// Already active at crate root — new code must use:
// ? propagation for Result
// .unwrap_or_else(|e| e.into_inner()) for Mutex
// .expect("reason") only in contexts where panic is acceptable (not in production paths)
```

### Poisoned-Lock Recovery (Rust)

**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 35 and 44
**Apply to:** All `state.lock()` calls (existing pattern, no new callsites in Phase 15)

```rust
let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
```

### Drain-and-Return (Rust)

**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 75–77, 84–86, 97–99, 104–106
**Apply to:** Every early-return branch and the final return in handle_op, including the new eex_chs branch

```rust
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
return Ok(CalcStateView::from_state(calc, print_lines));
```
This pattern is used identically 4 times in the existing code. The eex_chs branch is the 5th instance.

### Test Module Header (Rust)

**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 118–119
**Apply to:** All new test functions in types.rs and commands.rs

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    // ...
}
```

### React StrictMode Cleanup (TypeScript)

**Source:** `hp41-gui/src/main.tsx` lines 7–9 (StrictMode is active — verified)
**Apply to:** Every `useEffect` that registers a side effect in App.tsx

```typescript
useEffect(() => {
  // register side effect
  return () => { /* cleanup — removes side effect on unmount */ };
}, [deps]);
```
Without the cleanup return, React StrictMode's mount-unmount-mount cycle will double-register
the keydown listener and fire two IPC calls per keypress.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `hp41-gui/src/App.tsx` (complete build) | component | event-driven | No existing Tauri+React component in the codebase; main.tsx is a thin entry point only. RESEARCH.md Code Examples section provides the reference pattern. |

---

## Key Decisions That Affect Implementation Order

1. **Strip Tailwind first** — `index.css` and `vite.config.ts` must be updated before adding App.css styles, or Tailwind's base reset will override them. This is a Wave 0 prerequisite.
2. **Extend types.rs before commands.rs** — The test for CalcStateView payload size must pass before the eex_chs test can exercise the full round-trip.
3. **Wave 1 parallel work** — Rust changes (types.rs + commands.rs) and React changes (App.tsx + App.css) are independent; they can be built in parallel.

---

## Metadata

**Analog search scope:** `hp41-gui/src-tauri/src/`, `hp41-gui/src/`, `hp41-cli/src/`, `hp41-core/src/`
**Files scanned:** 11 (types.rs, commands.rs, key_map.rs, lib.rs, keys.rs, app.rs, format.rs, App.tsx, index.css, main.tsx, vite.config.ts)
**Pattern extraction date:** 2026-05-09
