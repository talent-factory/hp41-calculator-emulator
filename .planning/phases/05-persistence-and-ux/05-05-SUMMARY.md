---
phase: 05-persistence-and-ux
plan: 05
subsystem: ux-overlays
tags: [rust, ratatui, tui, help-overlay, program-library, annunciators, pending-input]

# Dependency graph
requires:
  - phase: 05-persistence-and-ux
    plan: 03
    provides: App struct Phase 5 fields (show_help, show_programs, help_table_state, programs_table_state, pending_input)
  - phase: 05-persistence-and-ux
    plan: 04
    provides: HELP_DATA static array + SampleProgram struct + sample_programs() via OnceLock

provides:
  - hp41-cli/src/ui.rs render_help_overlay() — centered 80%x90% Table from HELP_DATA with category headers
  - hp41-cli/src/ui.rs render_programs_overlay() — centered 70%x80% Table from sample_programs()
  - hp41-cli/src/ui.rs pending_prompt() — formats all PendingInput variants for status bar
  - hp41-cli/src/ui.rs render_annunciators() USER wired to state.user_mode (D-26)
  - hp41-cli/src/ui.rs render_status() priority: pending_input > alpha_mode > message (D-11/D-14)

affects:
  - 05-06 (STO/RCL pending input prompts now display correctly in status bar)
  - 05-07 (AssignKey/AssignLabel/ConfirmLoad prompts also display correctly)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Overlay z-ordering via draw-call order — ratatui renders in draw order; overlays drawn last appear on top
    - Rect::centered(h_constraint, v_constraint) from ratatui-core-0.1.0 — no manual Rect arithmetic needed
    - RefCell<TableState>.borrow_mut() inside draw(&self) — single-threaded non-reentrant draw makes this safe (RESEARCH Pitfall 1)
    - Table::new(rows, widths).block(...).row_highlight_style(...) with render_stateful_widget
    - Category header rows via desc.starts_with("===") pattern — applies bold style for visual grouping

key-files:
  created: []
  modified:
    - hp41-cli/src/ui.rs (193 → 345 lines: overlays, pending_prompt, USER wiring, tests)

key-decisions:
  - "Overlay z-ordering achieved by draw-call order only — no z-index needed in ratatui; overlay fns called after render_right_panel() covers both columns"
  - "pending_prompt() uses {:_<2} format spec so partially-typed register numbers show trailing underscores as placeholder"
  - "TableState unused dead_code warnings for help_scroll/programs_scroll fields in App are acceptable — those fields are used by Plan 06/07 scroll key bindings"

requirements-completed: [UX-01, UX-02, UX-03]

# Metrics
duration: ~8min
completed: 2026-05-07
---

# Phase 5 Plan 05: UI Overlays — Help + Program Library Summary

**Help overlay, program library overlay, pending input status bar, and USER annunciator wiring added to ui.rs — visual layer completing the Phase 5 overlay stack**

## Performance

- **Duration:** ~8 min
- **Completed:** 2026-05-07
- **Tasks:** 2
- **Files created:** 0, **Files modified:** 1

## Accomplishments

- `render_annunciators()`: USER annunciator now wired to `st.user_mode` (D-26). Was hardcoded `false` in Phase 4.
- `render_status()`: Priority order implemented — `pending_input` prompts override normal message, ALPHA mode shows "ALPHA mode — Enter or A to exit" hint, fallback to `app.message` or "Ready" (D-11/D-14).
- `pending_prompt()` helper: formats all 9 `PendingInput` variants for the status bar. STO/RCL variants use `{:_<2}` format to show placeholder underscores while digits are being typed.
- `render_help_overlay()`: centered 80%×90% `Table` widget, reads `HELP_DATA` from `help_data.rs`, category headers rendered bold (desc starts with "==="), `Row::row_highlight_style(reversed())` for selection, `RefCell::borrow_mut()` for stateful render.
- `render_programs_overlay()`: centered 70%×80% `Table` widget, reads `sample_programs()` from `programs.rs`, same stateful render pattern.
- `render_ui()` updated: overlay calls appear after `render_left_panel` + `render_right_panel` — correct z-ordering by draw-call order.
- `ui::tests` module added: `test_help_scroll` and `test_programs_scroll` verify `TableState::select_next()` and `select_previous()` do not panic on `RefCell<TableState>`.
- Full test suite: **33 tests pass, zero regressions** (31 pre-existing + 2 new overlay scroll tests).

## Task Commits

1. **Task 1: Wire USER annunciator + pending input status bar** - `28b5d64` (feat)
2. **Task 2: Add help overlay + program library overlay** - `8f2d7a8` (feat)

## Files Created/Modified

- `hp41-cli/src/ui.rs` — modified: USER annunciator wired, render_status priority logic, pending_prompt(), render_help_overlay(), render_programs_overlay(), overlay calls in render_ui(), ui::tests module

## Decisions Made

- Overlay z-ordering by draw-call order — ratatui renders in paint order; no z-index API needed. render_help_overlay() and render_programs_overlay() called after main panels.
- `pending_prompt()` uses `{:_<2}` format spec — partially typed "5" shows "STO [5_]", helping users see how many more digits to type.
- `Rect::centered()` confirmed in ratatui-core-0.1.0 (re-exported by ratatui 0.30.0) — no manual Rect arithmetic needed.

## Deviations from Plan

None — plan executed exactly as written. `Rect::centered()` API was confirmed present in ratatui-core-0.1.0 which backs ratatui 0.30.0. The plan's interface spec matched exactly.

## Known Stubs

None — all functionality fully wired. Note: `help_scroll: usize` and `programs_scroll: usize` fields in `App` remain unused by ui.rs (Plan 06/07 will add Up/Down key bindings calling `help_table_state.borrow_mut().select_next()`). These are not stubs in ui.rs — they are data fields in app.rs consumed by future plans.

## Threat Model Coverage

| Threat | Status |
|--------|--------|
| T-05-11: DoS — RefCell::borrow_mut() panic | Covered: draw() is single-threaded non-reentrant; comment documents the invariant; test_help_scroll + test_programs_scroll exercise borrow_mut() in tests |
| T-05-12: Tampering — program load overwrites user program without confirmation | Planned: ConfirmLoad(usize) in PendingInput is already handled in pending_prompt() displaying "[Y/n]" prompt; Plan 06 wires the Y/n key handler |

## Self-Check

### Modified files exist

- `hp41-cli/src/ui.rs` — exists (345 lines)

### Commits exist

- 28b5d64 — Task 1 (USER annunciator + pending_prompt + render_status priority)
- 8f2d7a8 — Task 2 (render_help_overlay + render_programs_overlay + ui::tests)

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
