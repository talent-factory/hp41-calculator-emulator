---
phase: 04-tui-and-input
plan: 02
subsystem: ui
tags: [ratatui, tui, layout, annunciators, stack-panel, rust]

# Dependency graph
requires:
  - phase: 04-01
    provides: App struct, CalcState, module stubs (ui.rs, keys.rs, prgm_display.rs)
  - phase: 03-programming-engine
    provides: CalcState, format_hpnum, format_alpha, AngleMode, DisplayMode

provides:
  - render_ui() fully implemented: two-column ratatui 0.30 layout
  - Stack panel: T/Z/Y/X/LASTX rendered via format_hpnum()
  - Display panel: entry_buf priority chain (entry_buf > prgm > alpha > formatted X)
  - Annunciator bar: PRGM/ALPHA/RAD/DEG/GRAD bold when active, USER/SHIFT dim (Phase 5)
  - Status bar: app.message or "Ready"
  - Key-reference panel: KEY_REF_TABLE in Block::bordered().title_top() right column
  - Terminal-too-small guard: < 80x24 renders single error line

affects: [04-03-key-mapping, 04-04-main-wiring, 05-persistence]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Layout::horizontal/vertical().areas::<N>() for idiomatic ratatui 0.30 splits
    - Block::bordered().title_top() — ratatui 0.30 API (Block::title(Title::from()) removed)
    - Span::styled(text, Style::new().bold()/dim()) for lit/dim annunciators
    - Closure-based formatter (fmt = |label, val| -> Paragraph<'static>) avoids borrow conflicts
    - Display priority chain: entry_buf > prgm_mode > alpha_mode > format_hpnum(X)

key-files:
  created: []
  modified:
    - hp41-cli/src/ui.rs

key-decisions:
  - "Stylize import removed — Style::new().bold()/dim() work without explicit `use Stylize` (auto-resolved via prelude)"
  - "L label for LASTX in stack panel (not LASTX as a full label) — keeps stack lines short and uniform"
  - "Display priority order: entry_buf first (live digit preview), then prgm step, alpha, then X register"
  - "USER and SHIFT annunciators always dim in Phase 4 — USER mode deferred to Phase 5"

# Metrics
duration: 8min
completed: 2026-05-07
---

# Phase 4 Plan 02: TUI Widget Layout Summary

**Full ratatui 0.30 two-column widget layout replacing Plan 04-01 stub: stack panel, display panel with priority chain, bold/dim annunciators, key-reference panel**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-07T09:25:00Z
- **Completed:** 2026-05-07T09:34:07Z
- **Tasks:** 1
- **Files modified:** 1 (hp41-cli/src/ui.rs — 184 insertions, 7 deletions)

## Accomplishments

- Replaced the Plan 04-01 stub (15-line placeholder) with the full 185-line ratatui 0.30 TUI layout
- Two-column split: left 55% (stack + display + annunciators + status), right 45% (key reference)
- Stack panel renders T/Z/Y/X and LASTX (as "L:") each on their own labeled line using `format_hpnum()`
- Display panel implements correct HP-41 priority chain: entry_buf (live digit preview) > prgm_mode step > alpha_mode register > formatted X register
- Annunciator bar uses `Span::styled()` with `Style::new().bold()`/`.dim()`: PRGM/ALPHA/RAD/DEG/GRAD respond to CalcState; USER and SHIFT remain dim (Phase 5)
- Terminal-too-small guard: < 80x24 short-circuits with a single error Paragraph, no panic
- Key-reference panel on the right renders KEY_REF_TABLE entries with bold key labels (currently empty stub, populated in Plan 04-03)
- `cargo check -p hp41-cli`: 0 errors, 0 warnings

## Task Commits

1. **Task 1: Full ui.rs implementation** - `c7521ce` (feat)

## Files Created/Modified

- `hp41-cli/src/ui.rs` — complete implementation replacing stub

## Decisions Made

- `Stylize` import removed from `use ratatui::style::{Style, Stylize}` — `Style::new().bold()` and `.dim()` resolve correctly without an explicit `use Stylize` statement (ratatui's prelude covers this), eliminating the unused-import warning.
- Display label for LASTX is "L:" rather than "LASTX:" to keep all five stack lines short and visually uniform.
- `get_display_string()` extracted as a standalone function (not inlined) to keep `render_display()` readable and testable in future.

## Deviations from Plan

None — plan executed exactly as written, with one micro-fix:

**[Rule 1 - Warning] Removed unused `Stylize` import**
- **Found during:** Verification (cargo check)
- **Issue:** `use ratatui::style::{Style, Stylize}` triggered an unused-import warning — `Stylize` is not referenced by name in the file
- **Fix:** Changed import to `use ratatui::style::Style;` — `.bold()` and `.dim()` on `Style` resolve via ratatui's prelude
- **Files modified:** hp41-cli/src/ui.rs (import line only)
- **Commit:** c7521ce (included in the same task commit)

## Known Stubs

None in ui.rs itself. The right-panel key-reference renders `KEY_REF_TABLE`, which is currently an empty slice from Plan 04-01. The panel will display correctly once Plan 04-03 populates the table. This is intentional and documented in the plan.

## Threat Flags

None — ui.rs is read-only over CalcState; no new network endpoints, auth paths, file access, or schema changes introduced.

## Self-Check: PASSED

- FOUND: hp41-cli/src/ui.rs
- FOUND: 04-02-SUMMARY.md
- FOUND commit: c7521ce

---
*Phase: 04-tui-and-input*
*Completed: 2026-05-07*
