# Phase 15: Display & Keyboard — Research

**Researched:** 2026-05-09
**Domain:** React 19 + Tauri v2 IPC wiring; Rust CalcStateView extension
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `CalcStateView` gains `y_str`, `z_str`, `t_str`, `lastx_str` — all via `format_hpnum(&state.stack.{reg}, &state.display_mode)`.
- **D-02:** `CalcStateView` gains `in_eex_mode: bool` — derived from `state.entry_buf.contains('e')`.
- **D-03:** No `entry_buf_raw` field — boolean flag is sufficient.
- **D-04:** Only atomic 1:1 key→op bindings wired (those that `key_to_op()` maps to non-None Op), plus digits/./e. Full list in D-04 of CONTEXT.md.
- **D-05:** Modal-triggering keys (`S`, `R`, `f`, `F`, `P`, `X`) silently ignored — no IPC call, no error.
- **D-06:** EEX-CHS fix: `'n'` keypress checks `calcState.in_eex_mode`; dispatches `"eex_chs"` if true, `"chs"` if false.
- **D-07:** Keyboard listeners use `useCallback` + `useEffect` with cleanup return.
- **D-08:** Vertical layout: Annunciator row → Display row → Stack panel → `<div id="keyboard-area" />` placeholder.
- **D-09:** Annunciators are text badges: always visible, dim (~30% opacity) when inactive, bright (full opacity) when active.
- **D-10:** Vanilla CSS only — no Tailwind, no CSS-in-JS, no new npm dependencies. Styles in `src/index.css` or `src/App.css`.
- **D-11:** `useState(CalcStateView)` — `invoke("get_state")` on mount; `invoke("dispatch_op", { keyId })` on keypress; no polling.
- **D-12:** `useCallback` + `useEffect` with cleanup is mandatory (SC-4).

### Claude's Discretion

- Component breakdown (single App component vs. Display/Annunciators/StackPanel sub-components).
- Monospace font choice (system monospace or named font).
- Exact CSS values (font size, colors, spacing) — dark-background calculator aesthetic is the guide.

### Deferred Ideas (OUT OF SCOPE)

- Multi-step modal sequences (STO, RCL, FIX/SCI/ENG digit count, ALPHA text, hex byte, print).
- TypeScript type generation (`tauri-specta` or similar).
- CSS styling framework (Tailwind).
- Keyboard shortcut overlay (`?` help panel).

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DISP-01 | User sees 12-char HP-41 display string and all five annunciators update after every op | `display_str` already in CalcStateView; annunciators struct already in CalcStateView; React reads these from invoke response |
| DISP-02 | User sees stack register panel (X/Y/Z/T/LASTX) updating after every op | CalcStateView must gain y_str/z_str/t_str/lastx_str (D-01); same `format_hpnum` path as x_str |
| IPC-02 | User can operate GUI entirely from physical keyboard using same key bindings as hp41-cli | key_to_op() in keys.rs is verified authoritative list; React keydown listener dispatches key IDs; EEX-CHS requires special-case routing in handle_op |

</phase_requirements>

---

## Summary

Phase 15 wires the Phase 14 IPC layer into a working React UI. The work splits cleanly into a **Rust extension** (two new fields on CalcStateView + one new branch in handle_op for EEX-CHS) and a **React build** (display panel, annunciator row, stack panel, keyboard listener).

**Critical finding — Tailwind conflict:** The Phase 13 scaffold installed Tailwind v4 and Vite plugin (`@tailwindcss/vite`) — `index.css` has `@import "tailwindcss"` and `vite.config.ts` includes `tailwindcss()`. D-10 says "vanilla CSS only / no Tailwind." The planner must decide: either remove Tailwind from `index.css`/`vite.config.ts` (simplest, leaves devDependencies installed but inactive), or accept that Tailwind is available and use utility classes (but that requires overturning D-10). Research finding: Tailwind IS installed at the devDependency level. The code in `index.css` actively imports it. The resolution is a plan task to strip the `@import "tailwindcss"` line and the `tailwindcss()` vite plugin call, then write styles manually in `index.css` or a new `App.css`. [VERIFIED: read index.css, vite.config.ts]

**EEX-CHS gap architecture:** The CLI handles `'n'` in EEX mode by directly mutating `entry_buf` (no call to `dispatch()`) — lines 389–404 of `app.rs`. To replicate this in the IPC layer, `commands.rs::handle_op` must intercept the `"eex_chs"` key ID before `key_map::resolve()` and apply the same entry_buf mutation logic. The `"eex_chs"` key ID must NOT go through `key_map::resolve()` (no corresponding Op exists). [VERIFIED: read commands.rs, app.rs, stack_ops.rs]

**Primary recommendation:** Two plans run in Wave 1 (parallel): (a) extend CalcStateView + add eex_chs to handle_op in Rust; (b) build React UI in App.tsx + styles in App.css. Wave 2 is integration verification only.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| HP-41 display string formatting | API/Core (hp41-core) | — | `format_hpnum` + `format_alpha` + `entry_buf` priority chain already computed in `CalcStateView::from_state()` |
| Annunciator state | API/Core (hp41-core) | — | Derived from CalcState booleans in `Annunciators` struct; already serialized in CalcStateView |
| Stack register formatting (Y/Z/T/LASTX) | API/Core (hp41-gui/src-tauri) | — | `format_hpnum` call in `CalcStateView::from_state()` — same function as x_str |
| EEX-CHS entry_buf mutation | API/Core (hp41-gui/src-tauri commands.rs) | — | Direct entry_buf mutation, same as CLI app.rs; never reaches core dispatch() |
| in_eex_mode boolean | API/Core (hp41-gui/src-tauri types.rs) | — | `state.entry_buf.contains('e')` evaluated in from_state() |
| Keyboard → key_id routing | Browser/Client (React) | — | keydown event listener; 'n' key checks in_eex_mode from state |
| invoke() call to backend | Browser/Client (React) | — | @tauri-apps/api invoke; Promise-based; resolves CalcStateView JSON |
| Display/annunciator/stack rendering | Browser/Client (React) | — | Pure render from CalcStateView state; no local computation |

---

## Standard Stack

### Core (already installed — no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| react | 19.2 | UI rendering | Already in package.json |
| react-dom | 19.2 | DOM rendering | Already in package.json |
| @tauri-apps/api | 2.11 | invoke() IPC bridge | Already in package.json; provides typed Promise interface |
| typescript | 6.0 | Type safety | Already in devDependencies |

[VERIFIED: package.json — all versions confirmed]

**No new npm dependencies required.** D-10 enforces vanilla CSS; no CSS library needed.

### Rust-side (already in hp41-gui/src-tauri/Cargo.toml)

| Crate | Version | Purpose |
|-------|---------|---------|
| tauri | 2.11.1 | Tauri framework (verified from Cargo.lock) |
| hp41-core | path dep | `CalcState`, `format_hpnum`, `format_alpha`, `AngleMode` |
| serde | 1 | Serialize on CalcStateView fields |

[VERIFIED: Cargo.toml, Cargo.lock]

---

## Architecture Patterns

### System Architecture Diagram

```
[Physical Keyboard]
      |
      v keydown event
[React: useCallback handler]
      |
      +-- digit / '.' / 'e' chars  -->  key_id = "0"-"9" | "." | "e"
      |
      +-- 'n' key  -->  check calcState.in_eex_mode  -->  key_id = "eex_chs" OR "chs"
      |
      +-- modal keys (S/R/f/F/P/X)  -->  silently ignored (no invoke)
      |
      +-- other mapped keys  -->  key_id = "enter" | "plus" | "sin" | etc.
      |
      v invoke("dispatch_op", { keyId })
[Tauri IPC bridge]
      |
      v
[commands.rs: handle_op(&mut CalcState, key_id)]
      |
      +-- digit keys  -->  entry_buf.push(char)
      +-- "." / "e"   -->  entry_buf guards then push
      +-- "eex_chs"   -->  entry_buf toggle exponent sign (new branch, Phase 15)
      +-- other       -->  key_map::resolve(key_id) --> Op --> dispatch()
      |
      v drain print_buffer
      v CalcStateView::from_state() -- reads display_str, x_str, y_str, z_str, t_str, lastx_str, in_eex_mode, annunciators
      |
      v CalcStateView JSON (~200 bytes)
      |
[React: setState(view)]
      |
      v re-render
[Display panel] -- display_str monospace right-aligned dark background
[Annunciator row] -- USER PRGM ALPHA RAD GRAD text badges (dim/bright)
[Stack panel] -- X/Y/Z/T/L labeled rows
[keyboard-area div] -- empty placeholder for Phase 16 SVG skin
```

### Recommended Project Structure

```
hp41-gui/
├── src/
│   ├── main.tsx            # Entry point (already exists — StrictMode wrapper)
│   ├── App.tsx             # REPLACE empty scaffold: all UI + keyboard listener
│   ├── App.css             # NEW: calculator display styles (vanilla CSS)
│   └── index.css           # MODIFY: remove @import "tailwindcss"; add base reset only
├── src-tauri/
│   └── src/
│       ├── types.rs        # EXTEND: add y_str, z_str, t_str, lastx_str, in_eex_mode to CalcStateView
│       └── commands.rs     # EXTEND: add "eex_chs" branch in handle_op before key_map::resolve()
```

### Pattern 1: Tauri invoke() in React (TypeScript)

**What:** Call a Tauri command from React; receive typed response.
**When to use:** Every calculator operation and initial state load.

```typescript
// Source: @tauri-apps/api/core.d.ts (VERIFIED: installed package)
import { invoke } from '@tauri-apps/api/core';

// TypeScript interface matching Rust CalcStateView (manual — no tauri-specta per D-deferred)
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
  y_str: string;       // Phase 15 additions
  z_str: string;
  t_str: string;
  lastx_str: string;
  in_eex_mode: boolean;
  annunciators: Annunciators;
  print_lines: string[];
}

// On mount: get initial state
const view = await invoke<CalcStateView>('get_state');

// On keypress: dispatch and get updated state
const view = await invoke<CalcStateView>('dispatch_op', { keyId: 'enter' });
```

Tauri v2 invoke signature: `invoke<T>(cmd: string, args?: InvokeArgs, options?: InvokeOptions): Promise<T>`
Arguments are serialized as JSON; object keys use camelCase by default.
`dispatch_op` takes `key_id: &str` — Tauri serializes the JS `{ keyId }` to `{ "key_id": "..." }` (snake_case command arguments match Rust parameter names automatically in Tauri v2).

[VERIFIED: core.d.ts in installed node_modules]

### Pattern 2: useCallback + useEffect keyboard listener (React StrictMode safe)

**What:** Attach a `keydown` listener that survives React StrictMode double-mount/unmount without firing duplicate IPC calls.
**When to use:** SC-4 requirement — mandatory per D-12.

```typescript
// Source: React 19 docs + CONTEXT.md D-07/D-12
import { useState, useEffect, useCallback } from 'react';

const [calcState, setCalcState] = useState<CalcStateView | null>(null);
const [busy, setBusy] = useState(false);

const handleKey = useCallback((e: KeyboardEvent) => {
  if (busy) return; // debounce: ignore key if previous invoke still pending
  
  const keyId = mapKeyToId(e); // see mapping table in Pattern 3
  if (keyId === null) return;  // silently ignore unmapped / modal-trigger keys
  
  e.preventDefault(); // prevent browser scroll on arrow/space
  setBusy(true);
  invoke<CalcStateView>('dispatch_op', { keyId })
    .then(view => setCalcState(view))
    .catch(err => console.error('dispatch_op error:', err))
    .finally(() => setBusy(false));
}, [busy]); // NOTE: if busy is in deps, effect re-registers on every keypress
             // Alternative: use a ref for busy flag to avoid re-registration

useEffect(() => {
  window.addEventListener('keydown', handleKey);
  return () => window.removeEventListener('keydown', handleKey); // cleanup — required
}, [handleKey]);
```

**StrictMode pitfall:** In React StrictMode, `useEffect` fires mount → unmount → mount. The cleanup (`removeEventListener`) prevents double registration. The `busy` flag prevents duplicate IPC calls if a key fires twice before the Promise resolves.

**Ref-based alternative (avoids effect re-registration on every keypress):**
```typescript
const busyRef = useRef(false);
const handleKey = useCallback((e: KeyboardEvent) => {
  if (busyRef.current) return;
  // ... map key, call invoke, set busyRef.current = true in setBusy, false in finally
}, []); // stable reference — effect only registers/deregisters on mount/unmount
```

[ASSUMED] The ref-based approach is a well-known React pattern for avoiding stale closures and effect churn; implementer should choose whichever approach produces the cleanest code.

### Pattern 3: Key → key_id mapping in React (authoritative list from keys.rs)

**What:** Translate a browser `KeyboardEvent` to a key ID string.
**Source truth:** `hp41-cli/src/keys.rs::key_to_op()` (verified by reading the file).

Complete mapping table:

```typescript
// Source: hp41-cli/src/keys.rs key_to_op() — VERIFIED
function mapKeyToId(e: KeyboardEvent): string | null {
  // Special case: 'n' with EEX mode — caller decides key_id based on calcState.in_eex_mode
  // This function returns "chs_or_eex" as a sentinel; caller resolves:
  //   in_eex_mode → "eex_chs", else → "chs"
  
  if (e.key === 'Enter')     return 'enter';
  if (e.key === 'Backspace') return 'clx';
  if (e.key === '+')         return 'plus';
  if (e.key === '-')         return 'minus';
  if (e.key === '*')         return 'mul';
  if (e.key === '/')         return 'div';
  if (e.key === 'n')         return null; // resolved outside: "eex_chs" or "chs"
  if (e.key === 'r')         return 'rdn';
  if (e.key === 'x')         return 'xy_swap';
  if (e.key === 'l')         return 'lastx';
  if (e.key === 's')         return 'sqrt';
  if (e.key === 'p')         return 'prgm_mode';
  if (e.key === 'a')         return 'asin';
  if (e.key === 'c')         return 'acos';
  if (e.key === 'k')         return 'atan';
  if (e.key === 'C')         return 'cos';
  if (e.key === 'T')         return 'tan';
  if (e.key === 'L')         return 'ln';
  if (e.key === 'G')         return 'log';
  if (e.key === 'E')         return 'exp';
  if (e.key === 'H')         return 'tenpow';
  if (e.key === 'I')         return 'recip';
  if (e.key === 'W')         return 'sq';
  if (e.key === 'Y')         return 'ypow';
  if (e.key === 'u')         return 'user_mode';
  if (e.key === 'z')         return 'sigma_plus';
  if (e.key === 'Z')         return 'sigma_minus';
  if (e.key === 'm')         return 'mean';
  if (e.key === 'D')         return 'sdev';
  if (e.key === 'y')         return 'yhat';
  if (e.key === 'b')         return 'lr';
  if (e.key === 'O')         return 'corr';
  if (e.key === 'V')         return 'cl_sigma_stat';
  if (e.key === 'h')         return 'hms_to_h';
  if (e.key === 'j')         return 'hms_add';
  if (e.key === 'J')         return 'hms_sub';
  if (e.key === 'q')         return 'sin';    // Phase 8 reassignment
  if (e.key === 'g')         return 'clreg';  // Phase 8 addition
  // Digit entry — all map to their single-character key_id
  if ('0123456789'.includes(e.key) && e.key.length === 1) return e.key;
  if (e.key === '.')         return '.';
  if (e.key === 'e')         return 'e';
  // Modal-trigger keys — silently ignore (D-05)
  if ('SRfFPX'.includes(e.key)) return null;
  // All other keys — ignore
  return null;
}
```

**Note on key ID casing:** Browser `e.key` for shifted characters gives the capital letter directly (e.g., Shift+C → `'C'`), matching how crossterm delivers `KeyCode::Char('C')` in the CLI. No modifier check needed. [VERIFIED: keys.rs comment "Crossterm delivers Shift+s as KeyCode::Char('S')"; browser `e.key` follows same convention]

### Pattern 4: CalcStateView extension in Rust (types.rs)

**What:** Add y_str, z_str, t_str, lastx_str, in_eex_mode to CalcStateView struct and from_state().

```rust
// Source: hp41-gui/src-tauri/src/types.rs (VERIFIED existing code + D-01/D-02 additions)
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

// In from_state():
let y_str = format_hpnum(&state.stack.y, &state.display_mode);
let z_str = format_hpnum(&state.stack.z, &state.display_mode);
let t_str = format_hpnum(&state.stack.t, &state.display_mode);
let lastx_str = format_hpnum(&state.stack.lastx, &state.display_mode);
let in_eex_mode = state.entry_buf.contains('e');
```

**Payload size impact:** Adding 4 formatted HpNum strings (~12 chars each) + 1 bool adds ~60–70 bytes to the ~130-byte baseline. Expected total: ~200 bytes. Still within the SC-1 ≤300-byte gate. [ASSUMED — estimated from format_hpnum max output width]

### Pattern 5: EEX-CHS branch in commands.rs::handle_op

**What:** The `"eex_chs"` key ID must be handled as a special case BEFORE `key_map::resolve()`, directly mutating `entry_buf`. It must NOT be added to `key_map.rs` (no Op variant exists for in-buffer exponent sign toggle).

```rust
// Source: hp41-cli/src/app.rs lines 389-404 (VERIFIED — direct entry_buf mutation)
// Location in commands.rs: after "e" block, before "Named / parameterized op" block

if key_id == "eex_chs" {
    if let Some(e_pos) = calc.entry_buf.find('e') {
        let after_e = &calc.entry_buf[e_pos + 1..];
        if after_e.starts_with('-') {
            // Remove minus: "1e-2" → "1e2"
            calc.entry_buf.remove(e_pos + 1);
        } else {
            // Insert minus: "1e2" → "1e-2", "1e" → "1e-"
            calc.entry_buf.insert(e_pos + 1, '-');
        }
    }
    // No-op if entry_buf doesn't contain 'e' (shouldn't happen given React guards,
    // but safe to ignore gracefully — no error return needed)
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    return Ok(CalcStateView::from_state(calc, print_lines));
}
```

**Why NOT in key_map.rs:** `key_map::resolve()` returns `Op` — there is no `Op::EexChs` variant in hp41-core and adding one would duplicate CLI logic in core. The in-buffer sign toggle is a UI-layer concern. [VERIFIED: stack_ops.rs op_chs() negates X register — no entry_buf awareness]

### Pattern 6: CSS structure for calculator display

**What:** Dark-background display with monospace font; dim/bright annunciator badges.
**When to use:** D-08/D-09/D-10 visual requirements.

```css
/* src/App.css — all new styles for Phase 15 */

/* Remove Tailwind import from index.css first */
/* (index.css currently has: @import "tailwindcss"; — must be removed) */

.calculator {
  display: flex;
  flex-direction: column;
  width: 320px;
  font-family: system-ui, sans-serif;
}

/* Annunciator row — D-09 */
.annunciators {
  display: flex;
  gap: 8px;
  padding: 4px 8px;
  background: #1a1a1a;
}

.annunciator {
  font-size: 11px;
  letter-spacing: 0.05em;
  color: #555;       /* dim: inactive */
  opacity: 0.3;
}

.annunciator.active {
  color: #fff;       /* bright: active */
  opacity: 1;
}

/* Display row — D-08 */
.display {
  background: #111;
  padding: 6px 8px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 20px;
  text-align: right;
  color: #c8e6c9;   /* greenish LCD tone — implementer may choose */
  letter-spacing: 0.05em;
  min-height: 2em;
}

/* Stack panel — D-08 */
.stack-panel {
  background: #1e1e1e;
  padding: 6px 8px;
}

.stack-row {
  display: flex;
  justify-content: space-between;
  font-family: 'Courier New', Courier, monospace;
  font-size: 13px;
  color: #aaa;
  padding: 1px 0;
}

.stack-label {
  color: #666;
  min-width: 24px;
}
```

### Anti-Patterns to Avoid

- **Tailwind-then-vanilla mismatch:** Do not leave `@import "tailwindcss"` in `index.css` while writing hand-rolled CSS — Tailwind's base reset will conflict. Strip the import first.
- **Polling for state:** Never call `setInterval(() => invoke("get_state"), ...)`. State must update only on keypress (D-11). Polling wastes IPC budget and can cause stale display.
- **Sending "chs" during EEX mode:** If React sends `"chs"` when `in_eex_mode` is true, `Op::Chs` will negate the X register instead of toggling the exponent sign. The `in_eex_mode` check in React is load-bearing.
- **Routing "eex_chs" through key_map:** `key_map::resolve("eex_chs")` returns `Err(GuiError { message: "unknown key: eex_chs" })`. The branch in `handle_op` must come before the `key_map::resolve()` call.
- **useEffect without cleanup:** Omitting the cleanup return causes double-registration in React StrictMode (mount → unmount → mount pattern). Every keypress would fire two IPC calls.
- **'n' always maps to "chs":** The browser does not distinguish CHS vs. EEX-CHS. React must read `calcState.in_eex_mode` from the latest state to decide which key_id to send.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Number formatting | Custom JS formatter | `x_str`/`y_str` etc. from CalcStateView | `format_hpnum` in hp41-core already handles FIX/SCI/ENG, overflow, 12-char limit |
| Annunciator logic | Local boolean state in React | `annunciators` from CalcStateView | CalcState is the single source of truth; React only renders |
| EEX sign toggle logic | JS entry_buf manipulation | "eex_chs" → handle_op in Rust | Entry_buf is owned by Rust; duplicating in JS breaks the invariant |
| TypeScript types | Manual type maintenance | Sufficient for Phase 15 (deferred to tauri-specta) | Phase 15 uses a simple inline interface; full generation deferred |

**Key insight:** All calculator logic lives in hp41-core. React's only job is to display what CalcStateView says and to translate physical key events into key_id strings.

---

## Common Pitfalls

### Pitfall 1: Tailwind conflict in index.css
**What goes wrong:** Phase 13 scaffold put `@import "tailwindcss"` in `index.css`. If left in place, Tailwind's base/reset CSS runs alongside any hand-written styles, producing conflicts and overrides that are hard to debug.
**Why it happens:** Phase 13 installed Tailwind as a devDependency for potential use; the import was added to the scaffold.
**How to avoid:** Remove `@import "tailwindcss"` from `index.css`; remove `tailwindcss()` from `vite.config.ts` plugins array. Add a plain `*` box-sizing reset instead.
**Warning signs:** Display panel has unexpected margin/padding; calculator layout breaks.
[VERIFIED: read index.css — confirmed `@import "tailwindcss"` is present]
[VERIFIED: read vite.config.ts — confirmed `tailwindcss()` plugin is active]

### Pitfall 2: CalcStateView JSON payload size regression
**What goes wrong:** Adding 4 string fields + 1 bool to CalcStateView could push the payload over the SC-1 ≤300-byte gate.
**Why it happens:** Each register string can be up to ~16 chars (e.g., "-1.234567890E-99" in SCI 9 format).
**How to avoid:** Run the existing `test_dispatch_op_payload_size` test after extending the struct — it asserts `≤300 bytes`.
**Warning signs:** Test failure: "CalcStateView JSON must be ≤300 bytes, got XXX bytes".

### Pitfall 3: React StrictMode double-effect
**What goes wrong:** In development, React StrictMode mounts → unmounts → mounts every component. Without a cleanup return in `useEffect`, the keydown listener is registered twice. Each keypress fires two `dispatch_op` invocations, corrupting state.
**Why it happens:** `main.tsx` wraps `<App />` in `<React.StrictMode>` — verified by reading the file.
**How to avoid:** Always return a cleanup function: `return () => window.removeEventListener('keydown', handleKey)`.
**Warning signs:** Pressing `1` + `ENTER` twice — calculator shows wrong value; each key appears to execute twice.
[VERIFIED: main.tsx confirmed `<React.StrictMode>` wrapper]

### Pitfall 4: Tauri invoke argument naming (snake_case)
**What goes wrong:** React developer sends `{ keyId: "enter" }` — Tauri receives it as `key_id` in Rust automatically. But if the developer sends `{ key_id: "enter" }` directly, that also works. The trap is using `{ key: "enter" }` (wrong name) — Tauri returns an error.
**Why it happens:** Tauri v2 serializes JS object keys to snake_case to match Rust parameter names. The Rust command is `dispatch_op(key_id: &str, ...)` — the argument name must match `key_id`.
**How to avoid:** Use `invoke('dispatch_op', { keyId: 'enter' })` — Tauri v2 automatically converts `keyId` → `key_id`.
**Warning signs:** Error logged: "missing required argument `key_id`".
[VERIFIED: commands.rs `pub fn dispatch_op(key_id: &str, state: State<'_, AppState>)`]

### Pitfall 5: 'n' sends wrong key_id
**What goes wrong:** React sends `"chs"` for `'n'` regardless of EEX mode. When user presses `EEX` then `'n'`, the Op::Chs fires on the flushed value (entry_buf is flushed on non-digit dispatch) instead of toggling the exponent sign.
**Why it happens:** EEX-CHS gap documented in STATE.md — the CLI handles this as a special case without calling dispatch(), which has no equivalent in the current IPC layer.
**How to avoid:** React reads `calcState.in_eex_mode` from the latest state snapshot before dispatching 'n'. If true, sends `"eex_chs"`; if false, sends `"chs"`. The `handle_op` in Rust intercepts `"eex_chs"` before `key_map::resolve()`.
**Warning signs:** After pressing `EEX` then `n`, the display shows negated X value instead of toggled exponent sign.

---

## Code Examples

### Minimal React App.tsx structure (starting point)

```typescript
// Source: patterns derived from CONTEXT.md decisions + verified Tauri API types
import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

interface Annunciators {
  user: boolean; prgm: boolean; alpha: boolean; rad: boolean; grad: boolean;
}
interface CalcStateView {
  display_str: string; x_str: string; y_str: string; z_str: string;
  t_str: string; lastx_str: string; in_eex_mode: boolean;
  annunciators: Annunciators; print_lines: string[];
}

function App() {
  const [calcState, setCalcState] = useState<CalcStateView | null>(null);
  const busyRef = useRef(false);

  // Mount: load initial state
  useEffect(() => {
    invoke<CalcStateView>('get_state')
      .then(view => setCalcState(view))
      .catch(err => console.error('get_state error:', err));
  }, []);

  const handleKey = useCallback((e: KeyboardEvent) => {
    if (busyRef.current) return;
    const keyId = resolveKeyId(e, calcState);
    if (keyId === null) return;
    e.preventDefault();
    busyRef.current = true;
    invoke<CalcStateView>('dispatch_op', { keyId })
      .then(view => setCalcState(view))
      .catch(err => console.error('dispatch_op error:', err))
      .finally(() => { busyRef.current = false; });
  }, [calcState]);

  useEffect(() => {
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [handleKey]);

  if (!calcState) return <div className="calculator">Loading...</div>;

  return (
    <div className="calculator">
      <div className="annunciators">
        {(['user','prgm','alpha','rad','grad'] as const).map(name => (
          <span key={name}
            className={`annunciator${calcState.annunciators[name] ? ' active' : ''}`}>
            {name.toUpperCase()}
          </span>
        ))}
      </div>
      <div className="display">{calcState.display_str}</div>
      <div className="stack-panel">
        {[
          ['X', calcState.x_str], ['Y', calcState.y_str], ['Z', calcState.z_str],
          ['T', calcState.t_str], ['L', calcState.lastx_str],
        ].map(([label, value]) => (
          <div key={label} className="stack-row">
            <span className="stack-label">{label}:</span>
            <span>{value}</span>
          </div>
        ))}
      </div>
      <div id="keyboard-area" />
    </div>
  );
}
```

### resolveKeyId helper

```typescript
function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
  // EEX-CHS: 'n' resolves based on current in_eex_mode (D-06)
  if (e.key === 'n') return state?.in_eex_mode ? 'eex_chs' : 'chs';
  // Digit entry
  if (e.key.length === 1 && '0123456789'.includes(e.key)) return e.key;
  if (e.key === '.') return '.';
  if (e.key === 'e') return 'e';
  // Modal-trigger keys: silently ignore (D-05)
  if ('SRfFPX'.includes(e.key) && e.key.length === 1) return null;
  // Named op mapping (see Pattern 3 for full table)
  const MAP: Record<string, string> = {
    'Enter': 'enter', 'Backspace': 'clx',
    '+': 'plus', '-': 'minus', '*': 'mul', '/': 'div',
    'r': 'rdn', 'x': 'xy_swap', 'l': 'lastx', 's': 'sqrt', 'p': 'prgm_mode',
    'a': 'asin', 'c': 'acos', 'k': 'atan', 'C': 'cos', 'T': 'tan',
    'L': 'ln', 'G': 'log', 'E': 'exp', 'H': 'tenpow', 'I': 'recip',
    'W': 'sq', 'Y': 'ypow', 'u': 'user_mode',
    'z': 'sigma_plus', 'Z': 'sigma_minus', 'm': 'mean', 'D': 'sdev',
    'y': 'yhat', 'b': 'lr', 'O': 'corr', 'V': 'cl_sigma_stat',
    'h': 'hms_to_h', 'j': 'hms_add', 'J': 'hms_sub',
    'q': 'sin', 'g': 'clreg',
  };
  return MAP[e.key] ?? null;
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 `invoke()` from `@tauri-apps/api` (CJS) | Tauri v2 `invoke()` from `@tauri-apps/api/core` (ESM) | v2.0.0 | Import path changed; old examples using `window.__TAURI__.tauri.invoke` are stale |
| Manually calling `Tauri.invoke` | `import { invoke } from '@tauri-apps/api/core'` | v2.0.0 | Explicit import; tree-shakeable |
| React 18 StrictMode behavior | React 19 StrictMode (same double-mount behavior preserved) | v19.0.0 | Cleanup in useEffect still required |

**Deprecated/outdated:**
- `@tauri-apps/api` v1 pattern `window.__TAURI__.tauri.invoke()`: replaced by ESM import in v2.
- `import { invoke } from '@tauri-apps/api'` (top-level): works but `@tauri-apps/api/core` is the documented sub-path.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Adding y/z/t/lastx_str + in_eex_mode stays under 300-byte SC-1 gate | Architecture Patterns (Pattern 4) | Test `test_dispatch_op_payload_size` will catch this immediately; easy to fix by abbreviating format |
| A2 | `useRef` for busy flag avoids stale closure and effect re-registration churn better than `useState` | Pattern 2 | Minor — either approach works; useRef is cleaner for non-rendered values |
| A3 | Browser `e.key` for shifted characters gives the capital letter directly (e.g., `'C'` for Shift+C) | Pattern 3 | If wrong, shifted key bindings won't fire. Easily testable in the browser dev console |

---

## Open Questions (RESOLVED)

1. **Tailwind removal scope**
   - What we know: `index.css` imports Tailwind; `vite.config.ts` has the Tailwind plugin; `package.json` has `tailwindcss` and `@tailwindcss/vite` in devDependencies.
   - What's unclear: Whether Phase 13 used any Tailwind classes anywhere (check: `index.html` and `App.tsx` — both appear to use no classes).
   - Recommendation: Plan task strips `@import "tailwindcss"` from `index.css` and removes `tailwindcss()` from `vite.config.ts` plugins. Do NOT remove from `package.json`/`node_modules` (Phase 16 may reconsider D-10). Zero net change to compiled output if no classes are used.

2. **payload size after extension**
   - What we know: current CalcStateView is ~130 bytes; adding 4 register strings (max ~16 chars each) + 1 bool adds ≤70 bytes.
   - What's unclear: whether display_str could be close to 300 bytes on its own (it cannot — HP-41 display is 12 chars max).
   - Recommendation: The test will catch it; no pre-action needed.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | npm install, vite dev server | ✓ | v22.16.0 | — |
| npm | package management | ✓ | 10.9.2 | — |
| Rust / cargo | hp41-gui Tauri build | ✓ | 1.89.0 (stable) | — |
| @tauri-apps/api | React invoke() | ✓ | 2.11 (node_modules present) | — |
| Tailwind (devDep) | Phase 13 scaffold (to be removed) | ✓ | 4.3 | N/A — will be stripped |

[VERIFIED: node --version, cargo --version, node_modules/@tauri-apps/api/CHANGELOG.md]

**Missing dependencies with no fallback:** None.
**Missing dependencies with fallback:** None.

---

## Validation Architecture

nyquist_validation is enabled (config.json does not set it to false).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in tests (cargo test) — hp41-gui/src-tauri |
| Config file | none (standard cargo test) |
| Quick run command | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Full suite command | `just gui-check && cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |

**Frontend testing:** No frontend test framework (vitest/jest) is installed in hp41-gui. The Phase 15 frontend changes (App.tsx, CSS) are tested manually by running `just gui-dev` and verifying SC-1 through SC-5. The planner should not add a test framework (new npm deps forbidden by D-10).

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DISP-01 | CalcStateView contains display_str and annunciators | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- types::tests` | ✅ types.rs test module |
| DISP-01 | New fields y_str/z_str/t_str/lastx_str/in_eex_mode present in CalcStateView | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- types::tests::test_calc_state_view_structure` | ❌ Wave 0 — extend test |
| DISP-01 | CalcStateView JSON stays ≤300 bytes after extension | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- types::tests::test_dispatch_op_payload_size` | ✅ test_dispatch_op_payload_size |
| DISP-02 | Stack panel displays X/Y/Z/T/LASTX | manual (GUI) | `just gui-dev` → verify stack panel | ❌ manual only |
| IPC-02 | EEX-CHS: "eex_chs" key_id toggles exponent sign in entry_buf | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- commands::tests::test_eex_chs` | ❌ Wave 0 — new test needed |
| IPC-02 | All atomic key bindings produce correct op via handle_op | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- commands::tests` | ✅ existing; extend coverage |

### Sampling Rate

- **Per task commit:** `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Per wave merge:** `just gui-check && cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Phase gate:** Full suite green + manual SC-1 through SC-5 verification before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] Extend `types::tests::test_calc_state_view_structure` to assert y_str/z_str/t_str/lastx_str/in_eex_mode fields exist and match expected values — covers DISP-01/DISP-02.
- [ ] Add `commands::tests::test_eex_chs_toggles_exponent_sign` — send `"eex_chs"` to `handle_op` with `entry_buf = "1e2"` → assert entry_buf = `"1e-2"`; second call → `"1e2"` again — covers IPC-02.
- [ ] Add `commands::tests::test_eex_chs_noop_without_e` — send `"eex_chs"` with no 'e' in entry_buf → assert no panic, returns Ok(view) — defensive coverage.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A — local desktop app, no user auth |
| V3 Session Management | no | N/A — no network sessions |
| V4 Access Control | no | N/A — single-user desktop app |
| V5 Input Validation | yes | key_id validated by key_map::resolve() returning Err for unknown IDs (SC-2 from Phase 14) |
| V6 Cryptography | no | N/A — no encryption in Phase 15 |

### Known Threat Patterns for Tauri v2 React IPC

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Frontend injects arbitrary key_id string | Tampering | `key_map::resolve()` returns `Err(GuiError)` for unknown IDs — never panics |
| Rapid keypress flooding invoke() | Denial of Service | `busyRef` debounce in React; Mutex lock serializes Rust side |
| CSS injection via display_str | Tampering | React renders `display_str` as text node (not innerHTML) — safe by default in JSX |

**Zero-panic policy:** `#![deny(clippy::unwrap_used)]` is active in `hp41-gui/src-tauri/src/lib.rs`. All new Rust code must use `?`-propagation or `.unwrap_or_else(|e| e.into_inner())` on Mutex locks. [VERIFIED: lib.rs]

---

## Sources

### Primary (HIGH confidence)
- `hp41-gui/src-tauri/src/types.rs` — verified CalcStateView struct: fields display_str, x_str, annunciators, print_lines (no y/z/t/lastx/in_eex_mode yet)
- `hp41-gui/src-tauri/src/commands.rs` — verified handle_op routing: digit keys, '.', 'e', then key_map::resolve(); no "eex_chs" branch
- `hp41-gui/src-tauri/src/key_map.rs` — verified resolve() function: ~50 named ops + 7 parameterized families; "eex_chs" absent
- `hp41-cli/src/keys.rs` — verified key_to_op() full mapping; authoritative Phase 15 keyboard binding list
- `hp41-gui/src/App.tsx` — verified: empty scaffold (5 lines, no UI)
- `hp41-gui/src/index.css` — verified: `@import "tailwindcss"` is present (Tailwind conflict)
- `hp41-gui/vite.config.ts` — verified: `tailwindcss()` plugin active
- `hp41-gui/src/main.tsx` — verified: `<React.StrictMode>` wraps App
- `hp41-gui/node_modules/@tauri-apps/api/core.d.ts` — verified: `invoke<T>(cmd, args?, options?): Promise<T>` signature
- `hp41-core/src/state.rs` — verified: Stack struct fields (x, y, z, t, lastx); entry_buf: String
- `hp41-core/src/format.rs` — verified: `format_hpnum(n: &HpNum, mode: &DisplayMode) -> String`
- `hp41-cli/src/app.rs` lines 389-404 — verified: EEX-CHS mutates entry_buf directly, no dispatch()

### Secondary (MEDIUM confidence)
- WebFetch: v2.tauri.app/develop/calling-rust/ — invoke() signature and error handling patterns

### Tertiary (LOW confidence)
- None

---

## Project Constraints (from CLAUDE.md)

- **Zero-panic policy:** `#![deny(clippy::unwrap_used)]` active in hp41-gui/src-tauri — all new Rust code uses `?` or `.unwrap_or_else(|e| e.into_inner())`
- **No new npm deps:** D-10; package.json additions forbidden for Phase 15
- **No core duplication:** hp41-gui/src-tauri must not contain `op_*`, `flush_entry_buf`, or `format_hpnum` function definitions — only calls into hp41-core
- **hp41-core never depends on GUI crate:** Compile-time enforced; no new `use hp41_gui_*` in hp41-core
- **just task runner:** All build/test commands go through `just` recipes or `cargo test --manifest-path` for the nested workspace; never bare `cargo` from root
- **English commit messages:** All commit messages in English

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified from installed node_modules and Cargo.lock
- Architecture: HIGH — all source files read; IPC patterns verified from installed type definitions
- Pitfalls: HIGH — confirmed from direct source reading (Tailwind in index.css, StrictMode in main.tsx, EEX-CHS gap in commands.rs)

**Research date:** 2026-05-09
**Valid until:** 2026-06-09 (Tauri v2 and React 19 are stable; no fast-moving parts in Phase 15 scope)
