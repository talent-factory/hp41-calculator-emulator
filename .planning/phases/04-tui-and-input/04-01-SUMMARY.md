---
phase: 04-tui-and-input
plan: 01
subsystem: ui
tags: [ratatui, crossterm, clap, tui, event-loop, rust]

# Dependency graph
requires:
  - phase: 03-programming-engine
    provides: CalcState, dispatch(), Op enum, run_program(), HpError

provides:
  - hp41-cli crate compiles with ratatui 0.30 + crossterm 0.29 + clap 4.x
  - App struct with poll-based event loop (16ms poll interval)
  - handle_key() dispatcher with Release filter and digit-direct-append semantics
  - Module stubs for ui.rs, keys.rs, prgm_display.rs

affects: [04-02-ui-layout, 04-03-key-mapping, 05-persistence]

# Tech tracking
tech-stack:
  added:
    - ratatui 0.30 (with crossterm feature)
    - crossterm 0.29
    - clap 4.x (with derive feature)
  patterns:
    - Poll-based event loop (event::poll(16ms)) — never blocking event::read()
    - draw(&self) immutable for borrow-checker safety with &mut terminal
    - Digit entry appended directly to entry_buf, not dispatched individually

key-files:
  created:
    - hp41-cli/src/app.rs
    - hp41-cli/src/ui.rs
    - hp41-cli/src/keys.rs
    - hp41-cli/src/prgm_display.rs
  modified:
    - hp41-cli/Cargo.toml
    - hp41-cli/src/main.rs

key-decisions:
  - "ratatui::init() returns DefaultTerminal — RestoreTerminalGuard does not exist in 0.30; explicit ratatui::restore() required after run()"
  - "KeyEventKind::Press filter is the absolute first check in handle_key() — prevents double-fire on Windows"
  - "Digits 0-9 / '.' / 'e' append to entry_buf directly; dispatch() flushes automatically on next non-digit op"
  - "draw(&self) takes immutable self to avoid borrow conflict with &mut terminal inside terminal.draw()"
  - "F5 hardcoded to run_program('A') for Phase 4 — full key mapping deferred to Plan 04-03"

patterns-established:
  - "Pattern: poll(16ms) loop — poll first, then read if event available, then draw; allows future timer injection"
  - "Pattern: call_dispatch() wraps hp41_core::ops::dispatch() and maps HpError → self.message"
  - "Pattern: module stubs with correct signatures compile immediately, filled in subsequent plans"

requirements-completed: [DISP-01, DISP-02, INPUT-01]

# Metrics
duration: 12min
completed: 2026-05-07
---

# Phase 4 Plan 01: TUI Skeleton Summary

**ratatui 0.30 + crossterm 0.29 TUI skeleton with App event loop, digit entry semantics, and compiling module stubs**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-07T00:00:00Z
- **Completed:** 2026-05-07T00:12:00Z
- **Tasks:** 3
- **Files modified:** 6 (1 modified + 5 created)

## Accomplishments

- hp41-cli now depends on ratatui 0.30, crossterm 0.29, clap 4.x — crate compiles with zero errors
- App struct created with complete poll-based event loop: draw → poll(16ms) → handle_key → repeat
- handle_key() implements all Phase 4 critical constraints: Release filter, digit-direct-append, angle/display cycling, SST/BST, R/S
- Module stubs (ui.rs, keys.rs, prgm_display.rs) provide correct public signatures so main.rs compiles immediately
- main.rs uses ratatui::init() and ratatui::restore() — correct ratatui 0.30 API (no RestoreTerminalGuard)

## Task Commits

Each task was committed atomically:

1. **Task 1: Cargo.toml — add ratatui, crossterm, clap** - `e29af9a` (chore)
2. **Task 2: app.rs — App struct with full event loop** - `da45ecb` (feat)
3. **Task 3: main.rs + module stubs** - `eb7a59e` (feat)

## Files Created/Modified

- `hp41-cli/Cargo.toml` — added ratatui 0.30, crossterm 0.29, clap 4.x dependencies
- `hp41-cli/src/main.rs` — replaced 4-line stub with ratatui::init()/restore() entry point, 4 mod declarations
- `hp41-cli/src/app.rs` — App struct, run() event loop, handle_key() dispatcher, call_dispatch() helper
- `hp41-cli/src/ui.rs` — stub: render_ui() renders placeholder Paragraph (Plan 04-02 fills this)
- `hp41-cli/src/keys.rs` — stub: key_to_op() returns None, KEY_REF_TABLE empty (Plan 04-03 fills this)
- `hp41-cli/src/prgm_display.rs` — stub: format_step() returns "{pc:03} END" (Plan 04-03 fills this)

## Decisions Made

- `ratatui::init()` returns `DefaultTerminal`, not `RestoreTerminalGuard` — the plan's critical note was verified correct; `ratatui::restore()` is called explicitly in main() after the event loop.
- `draw(&self)` is immutable (not `&mut self`) to avoid the Rust borrow checker conflict where `terminal.draw()` requires `&mut terminal` while also calling `self.draw()` — this is enforced by the plan (D-17).
- Digit keys ('0'-'9', '.', 'e') append directly to `state.entry_buf` without calling `dispatch()` — the next non-digit op causes `dispatch()` to flush automatically via `flush_entry_buf()` inside hp41-core.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None — all three tasks compiled cleanly on first attempt. The two dead-code warnings for `KEY_REF_TABLE` and `format_step` are expected for stubs and will disappear when Plans 04-02 and 04-03 wire up the modules.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 04-02 (TUI layout) can now implement `render_ui()` in ui.rs — App struct and Frame signature are in place
- Plan 04-03 (key mapping) can implement `key_to_op()` in keys.rs and populate `KEY_REF_TABLE`
- All module boundaries are established; no file moves or renames needed in subsequent plans

---
*Phase: 04-tui-and-input*
*Completed: 2026-05-07*
