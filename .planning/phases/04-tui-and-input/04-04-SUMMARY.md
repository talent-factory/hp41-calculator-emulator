---
phase: 04-tui-and-input
plan: 04
subsystem: ui
tags: [clap, ratatui, crossterm, tui, event-loop, rust]

# Dependency graph
requires:
  - phase: 04-tui-and-input
    plan: 02
    provides: ui.rs full widget layout
  - phase: 04-tui-and-input
    plan: 03
    provides: key_to_op(), KEY_REF_TABLE, prgm_display

provides:
  - Complete main.rs entry point with clap argument parsing
  - End-to-end verified TUI: digit entry, stack ops, annunciators, PRGM mode, error display, clean exit

affects: [04-05-ci-gate, 05-persistence]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Cli::parse() invoked before ratatui::init() — clap exits cleanly without touching the terminal"
    - "ratatui::init() then App::new().run(terminal) then ratatui::restore() — canonical Phase 4 entry sequence"
    - "explicit ratatui::restore() after run() for normal exit; panic hook covers the crash path"

key-files:
  modified:
    - hp41-cli/src/main.rs

key-decisions:
  - "Cli::parse() called before ratatui::init() so --help/--version exit before raw mode is entered"
  - "Phase 5 arg stub left as comment in Cli struct: --state <file.json> for PERS-01"
  - "ratatui::restore() in main() is mandatory — panic hook only fires on panics, not normal return"

requirements-completed:
  - DISP-01
  - DISP-02
  - INPUT-01

# Metrics
duration: ~5min
completed: 2026-05-07
---

# Phase 4 Plan 04: main.rs clap + Event-Loop Verification Summary

**Complete HP-41 TUI vertical slice: clap args wired, ratatui::init/restore sequence confirmed, all 17 manual smoke tests pass**

## Performance

- **Duration:** ~5 min
- **Completed:** 2026-05-07
- **Tasks:** 2 (auto: clap wiring; checkpoint: human smoke test)
- **Files modified:** 1

## Accomplishments

- `Cli::parse()` integrated before `ratatui::init()` — `--help` and `--version` exit cleanly without touching the terminal
- `ratatui::init()` / `App::new().run(terminal)` / `ratatui::restore()` sequence confirmed as canonical Phase 4 entry pattern
- Phase 5 argument stub (`--state <file>`, PERS-01) documented as comment in `Cli` struct
- All 17 human smoke test checks passed:
  - Two-column TUI renders correctly (stack, display, annunciators, key reference)
  - Live digit entry via `entry_buf` preview works
  - Enter pushes to stack and clears entry_buf
  - Arithmetic operations (+, SIN) produce correct results
  - `d` cycles angle mode with same-frame annunciator update
  - PRGM mode (`p`) toggles boldness of [PRGM] annunciator, shows `000 END`
  - Error path (F5 with no program) shows status bar message — no crash
  - `q` and Ctrl+C restore terminal cleanly

## Task Commits

1. **Task 1: Add clap arg parsing to main.rs** — `305e07c` (feat(04-04))

## Files Created/Modified

- `hp41-cli/src/main.rs` — clap `Cli` struct added; entry sequence finalized

## Decisions Made

- `Cli::parse()` before `ratatui::init()`: clap must run before raw mode so that `--help`/`--version` print normally and exit without terminal artifacts.
- Explicit `ratatui::restore()` mandatory in `main()`: the panic hook only fires on panics; normal return through `run()` bypasses the hook entirely.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

- Full Phase 4 vertical slice is working and human-verified
- All three TUI modules (ui.rs, keys.rs, prgm_display.rs) + App event loop functional
- Ready for Plan 04-05: `just ci` gate (full workspace tests + coverage + clippy)

---
*Phase: 04-tui-and-input*
*Completed: 2026-05-07*
