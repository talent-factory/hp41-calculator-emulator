# Phase 17: Persistence & Print Output - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire persistence for the GUI calculator: load `~/.hp41/autosave.json` on startup, write it every 30 seconds in a Rust background thread, and handle v1.x CLI save files gracefully. Also make PRX/PRA/PRSTK print output visible in a collapsible print panel that auto-shows when output arrives and collapses (without clearing history) on dismiss.

This phase delivers PERS-01 and PERS-02 from the ROADMAP.

</domain>

<decisions>
## Implementation Decisions

### Auto-save Mechanism

- **D-01:** Auto-save is driven by a **`std::thread::spawn` Rust background thread** spawned in `lib.rs setup()`. The thread loops: `thread::sleep(Duration::from_secs(30)) → lock Mutex<CalcState> → save_state()`. No tokio, no new Tauri command — fully self-contained in Rust.
- **D-02:** On save failure, log silently to stderr. No UI notification — matches hp41-cli behavior and avoids modal interruption.

### Startup Load

- **D-03:** In `lib.rs setup()`, **attempt to load `~/.hp41/autosave.json`** via `load_state()`. On success, initialize `AppState` with the loaded `CalcState`. On any error (missing file, parse failure, version mismatch), fall back to `CalcState::new()` silently — never panic, never block startup.
- **D-04:** `CalcState::is_running` is always reset to `false` after load (already enforced in `load_state()` — Pitfall 4).

### Persistence Code Location

- **D-05:** **Copy `persistence.rs` into `hp41-gui/src-tauri/src/persistence.rs`**. Do not create a shared crate. The module is ~100 lines; DRY violation is minor and avoids workspace complexity. The GUI copy may diverge slightly in error handling without affecting the CLI.
- **D-06:** The GUI `persistence.rs` must use the same `StateFile { version: u32, state: CalcState }` wrapper so save files are interoperable between CLI and GUI (SC-4).

### Print Panel UI

- **D-07:** The print panel is **collapsible**. It is **hidden by default** and **auto-shows** the first time `print_lines` arrives in a `CalcStateView` (i.e., when PRX, PRA, or PRSTK produces output).
- **D-08:** The panel stays visible once shown. The user can **dismiss it with a close button** — closing collapses the panel but **preserves the accumulated print log** in React state. Reopening (triggered by the next print output) shows the full history including previous lines.
- **D-09:** Print line accumulation is in **React state**: `setPrintLog(prev => [...prev, ...view.print_lines])`. The panel renders the full accumulated log with `overflow-y: auto` scroll. Since `print_buffer` is drained on every IPC call, React is responsible for retaining history.

### Claude's Discretion

- Visual style of the print panel (dark calculator style vs "paper tape" look), exact CSS values, font size, height — implementer's judgment. Should feel consistent with the existing calculator dark aesthetic.
- Whether the close button is an ×, a "Clear" label, or an icon — implementer's choice as long as it's clearly dismissible.
- Whether the panel header says "Print Output", "PRINT", or similar — implementer's choice.
- Line retention cap: implementer may cap at 200 lines if needed for memory, but no cap is required by spec.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope
- `.planning/ROADMAP.md` §Phase 17 — Goal, 5 success criteria (SC-1 through SC-5), Requirements PERS-01 / PERS-02
- `.planning/REQUIREMENTS.md` §PERS — PERS-01, PERS-02 definitions

### Existing Persistence Logic (CLI)
- `hp41-cli/src/persistence.rs` — `save_state()`, `load_state()`, `default_state_path()`, `StateFile` wrapper; **copy this module** into `hp41-gui/src-tauri/src/persistence.rs` and adapt as needed

### Existing GUI Backend
- `hp41-gui/src-tauri/src/lib.rs` — `AppState` type alias, `setup()` hook, `invoke_handler` registration; Phase 17 extends `setup()` to load state and spawn auto-save thread
- `hp41-gui/src-tauri/src/commands.rs` — `dispatch_op`, `get_state` handlers; `print_buffer` drain already implemented — no changes needed
- `hp41-gui/src-tauri/src/types.rs` — `CalcStateView` with `print_lines: Vec<String>` already present; no Rust-side type changes needed for print panel

### Existing Frontend
- `hp41-gui/src/App.tsx` — Current app state with `calcState`, `busyRef`, `handleKey`, `handleClick`; Phase 17 adds `printLog: string[]` state and collapsible panel rendering
- `hp41-gui/src/App.css` — Existing calculator styles; Phase 17 adds print panel styles

### Known Implementation Traps (from STATE.md)
- `STATE.md` §Critical Implementation Traps — `#[serde(default)]` required for new CalcState fields (SC-2 compat with v1.x saves); `is_running` must always be reset to `false` after load (Pitfall 4)

### Prior Phase Context
- `.planning/phases/15-display-and-keyboard/15-CONTEXT.md` — D-10: Vanilla CSS only; D-11/D-12: React state and `useCallback`/`useEffect` patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hp41-cli/src/persistence.rs` — `save_state()`, `load_state()`, `default_state_path()`, `StateFile` (copy verbatim, adjust crate paths)
- `CalcStateView.print_lines: Vec<String>` — already populated by `commands.rs` drain; React reads this field on every dispatch response
- `busyRef: React.MutableRefObject<boolean>` — IPC debounce guard; print panel open/close is local state, no IPC needed

### Established Patterns
- **Zero-panic policy:** `#![deny(clippy::unwrap_used)]` — all new Rust code uses `?`-propagation; poisoned Mutex: `.unwrap_or_else(|e| e.into_inner())`
- **Vanilla CSS (no libraries)** — Phase 15 decision; print panel uses className strings + App.css rules
- **`useState` + `useEffect` with cleanup** — React state management pattern established in Phase 15
- **`#[serde(default)]`** on new CalcState fields — mandatory for backward compat with v1.x CLI saves (documented trap)

### Integration Points
- `lib.rs setup()` — add `load_state()` call + `std::thread::spawn` auto-save loop
- `App.tsx` — add `printLog: string[]` state, accumulation logic, and print panel render
- New `hp41-gui/src-tauri/src/persistence.rs` module (copied from hp41-cli)
- New Tauri capability permission TOML if `save_state()` needs filesystem access (check if `fs:default` already covers `~/.hp41/`)

</code_context>

<specifics>
## Specific Ideas

- The print panel should auto-open (become visible) the first time `view.print_lines.length > 0` is received — no explicit user action to show it.
- Closing the panel (× button) sets a `printPanelOpen: boolean` state flag to false; the `printLog` array is NOT cleared. The next print output sets `printPanelOpen` back to true.
- The panel is a fixed-height (e.g., 120–150px) scrollable `<div>` using `overflow-y: auto`. New lines appended to the bottom; the div should auto-scroll to bottom when new content arrives.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 17-Persistence & Print Output*
*Context gathered: 2026-05-10*
