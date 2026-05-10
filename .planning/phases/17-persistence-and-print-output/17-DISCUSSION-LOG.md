# Phase 17: Persistence & Print Output - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 17-persistence-and-print-output
**Areas discussed:** Auto-save mechanism, Persistence code location, Print panel UI

---

## Auto-save mechanism

### Q1: How should the 30-second auto-save be driven?

| Option | Description | Selected |
|--------|-------------|----------|
| Rust background thread | std::thread::spawn in lib.rs setup(). Loops: sleep 30s → lock Mutex → save_state(). No new IPC command, no UI involvement. | ✓ |
| Frontend setInterval | React calls a new save_now Tauri command every 30s. Simpler but adds extra IPC command and couples save timing to frontend lifecycle. | |

**User's choice:** Rust background thread
**Notes:** Consistent with hp41-cli's event-loop-timer approach; keeps all persistence logic in Rust.

### Q2: Should the background thread use std::thread::sleep or Tauri's async runtime?

| Option | Description | Selected |
|--------|-------------|----------|
| std::thread::spawn + std::thread::sleep | One dedicated OS thread, sleeps 30s, locks Mutex, saves. Simple, no tokio dependency. | ✓ |
| tauri::async_runtime::spawn (tokio) | Uses tokio::time::sleep inside async block. Cleaner for async apps but adds complexity for a simple periodic save. | |

**User's choice:** std::thread::spawn + std::thread::sleep
**Notes:** No existing async in the Tauri backend; keeps things simple.

---

## Persistence code location

### Q1: Where should the GUI's persistence logic live?

| Option | Description | Selected |
|--------|-------------|----------|
| Copy into hp41-gui/src-tauri/src/persistence.rs | Small module (~100 lines), easy to own. Minor DRY violation. No new crate. | ✓ |
| Extract shared hp41-persist crate | New Cargo workspace member. Eliminates duplication but adds overhead for 100 lines that rarely change. | |
| Move into hp41-core | Violates zero-I/O invariant. Not viable. | |

**User's choice:** Copy into hp41-gui/src-tauri/src/persistence.rs
**Notes:** Pragmatic choice; the module is small enough that duplication cost is low.

---

## Print panel UI

### Q1: Where does the print panel appear in the layout?

| Option | Description | Selected |
|--------|-------------|----------|
| Between stack panel and keyboard | Visible at all times, no toggling, fixed height with scroll. | |
| Below the keyboard | Keeps calculator visual intact; might be offscreen on small windows. | |
| Collapsible sidebar or overlay | Hidden by default, shown when content exists. More complex. | ✓ |

**User's choice:** Collapsible sidebar or overlay

### Q2: How does the collapsible print panel open/close?

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-show when output arrives, stays open | Panel appears on first PRX/PRA/PRSTK output, stays visible until dismissed. | ✓ |
| Toggle button on the calculator | Small toggle opens/closes panel regardless of content. | |
| Overlay: slides down from top | Overlays keyboard area, closes with X or keypress. | |

**User's choice:** Auto-show when output arrives, stays open
**Notes:** No action needed to see output — it just appears naturally when the calculator produces print lines.

### Q3: What happens to print panel lines when user dismisses?

| Option | Description | Selected |
|--------|-------------|----------|
| Clear + collapse | Closing also clears print log. Simple. | |
| Collapse only (lines preserved, reopen shows history) | Panel hides but accumulated lines kept in React state. Next print output reopens panel with full history. | ✓ |

**User's choice:** Collapse only — lines preserved, reopen shows history
**Notes:** History accumulation is natural; user doesn't lose previous print output by accidentally dismissing the panel.

---

## Claude's Discretion

- Visual style of the print panel (dark vs "paper tape" aesthetic), exact CSS, font size, height
- Close button label (×, "Clear", icon)
- Panel header text ("Print Output", "PRINT", etc.)
- Optional line retention cap (up to 200 lines)
- Auto-scroll to bottom behavior on new content

## Deferred Ideas

None — discussion stayed within phase scope.
