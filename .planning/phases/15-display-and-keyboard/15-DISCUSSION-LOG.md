# Phase 15: Display & Keyboard - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-09
**Phase:** 15-display-and-keyboard
**Areas discussed:** Stack view data source, EEX mode detection, Keyboard modal scope, Layout approach

---

## Stack View Data Source

| Option | Description | Selected |
|--------|-------------|----------|
| Extend CalcStateView | Add y_str, z_str, t_str, lastx_str to CalcStateView in types.rs. Every dispatch_op already returns CalcStateView — stack panel gets fresh values on every keypress with no extra round-trip. | ✓ |
| Separate get_state() call | Keep CalcStateView lean. Stack panel calls get_state() independently after each dispatch. Two IPC calls per keypress. | |
| New richer view type | Add FullCalcStateView alongside CalcStateView. Opt-in — dispatch_op returns lean view, get_full_state() returns rich one. | |

**User's choice:** Extend CalcStateView (Recommended)
**Notes:** Include all four registers: Y, Z, T, LASTX. All use format_hpnum() for consistent formatting with x_str.

**Follow-up: Include LASTX?**

| Option | Description | Selected |
|--------|-------------|----------|
| Include LASTX | y_str, z_str, t_str, lastx_str — full parity with CLI display. | ✓ |
| Y/Z/T only | Skip LASTX for now; less commonly watched. | |

**User's choice:** Include LASTX (Recommended)

---

## EEX Mode Detection

| Option | Description | Selected |
|--------|-------------|----------|
| Add in_eex_mode flag to CalcStateView | bool derived from entry_buf.contains('e') in Rust. React checks flag on 'n' keypress — sends 'eex_chs' if true, 'chs' if false. | ✓ |
| Detect from display_str | React parses display_str for EEX pattern. Fragile — display format can change. | |
| Handle in Rust key_map.rs | Add EEX mode to AppState, let key_map.rs auto-convert 'chs' → 'eex_chs'. key_map.rs is currently stateless. | |

**User's choice:** Add in_eex_mode flag to CalcStateView (Recommended)
**Notes:** Closes the EEX-CHS gap noted in STATE.md. Frontend sends "eex_chs" key ID when in_eex_mode is true.

**Follow-up: Expose entry_buf_raw?**

| Option | Description | Selected |
|--------|-------------|----------|
| Boolean flag only | in_eex_mode: bool only. Clean abstraction boundary. | ✓ |
| Expose entry_buf_raw too | Add entry_buf: String alongside in_eex_mode. More data, blurs boundary. | |

**User's choice:** Boolean flag only (Recommended)

---

## Keyboard Modal Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Atomic keys only | Wire only 1:1 key→op bindings from key_to_op(). Multi-step modals (STO, RCL, FIX/SCI/ENG, ALPHA, hex byte, print) deferred. | ✓ |
| Atomic + STO/RCL + FIX/SCI/ENG | Most common modals included. More React state logic. | |
| All modals | Full parity with CLI in Phase 15. Triples implementation scope. | |

**User's choice:** Atomic keys only (Recommended)
**Notes:** Covers SC-1 through SC-5. key_to_op() returns None for modal-triggering keys (S, R, f, P, X) — same behavior applies in Phase 15.

**Follow-up: What to do with modal keys?**

| Option | Description | Selected |
|--------|-------------|----------|
| Silently ignore | No IPC call, no state change. Matches key_to_op() returning None. | ✓ |
| Show 'not yet implemented' toast | Brief on-screen feedback. Small extra effort. | |

**User's choice:** Silently ignore (Recommended)

---

## Layout Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Functional vertical stack | Annunciators row + 12-char display, then stack panel (X/Y/Z/T/LASTX), then placeholder div for Phase 16 keyboard. | ✓ |
| Side-by-side panels | Display+annunciators left, stack panel right. Phase 16 SVG keyboard would need to reshape the layout. | |

**User's choice:** Functional vertical stack (Recommended)
**Notes:** User selected the layout mockup showing annunciators above display, with stack panel below and keyboard placeholder at bottom.

**Follow-up: Annunciator style?**

| Option | Description | Selected |
|--------|-------------|----------|
| Text badges, dim when off | Small text labels. Active = bright white/yellow; inactive = dim gray. Matches HP-41 hardware aesthetic. | ✓ |
| Colored chips/pills | Pill-shaped with color (green for RAD, blue for USER). No HP-41 precedent. | |
| You decide | Leave to implementer. | |

**User's choice:** Text badges, dim when off (Recommended)

**Follow-up: CSS framework?**

| Option | Description | Selected |
|--------|-------------|----------|
| Vanilla CSS only | No Tailwind, no CSS-in-JS. Phase 15 extends index.css or adds App.css. No new npm dependencies. | ✓ |
| Add Tailwind CSS | Right time to add before more components. Adds build step and dependency. | |
| You decide | Leave to implementer. | |

**User's choice:** Vanilla CSS only (Recommended)

---

## Claude's Discretion

- Component breakdown (separate Display, Annunciators, StackPanel components vs. one component) — implementer decides
- Monospace font choice — system monospace or specific font; implementer judges what looks best
- Exact CSS values (font size, colors, spacing) — implementer judgment; dark-background calculator aesthetic is the guide

## Deferred Ideas

- Multi-step modal sequences (STO, RCL, FIX/SCI/ENG, ALPHA, hex byte, print) — Phase 16 or quick tasks
- TypeScript type generation for CalcStateView — deferred; Phase 15 uses manual types
- CSS framework (Tailwind) — deferred; revisit in Phase 16 if needed
- Keyboard shortcut overlay — v2.1 (SKIN-05)
