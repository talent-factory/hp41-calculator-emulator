---
phase: 15-display-and-keyboard
plan: "01"
subsystem: ui
tags: [tauri, vite, react, css, rust, tdd, red-tests]

# Dependency graph
requires:
  - phase: 15-00
    provides: Phase 15 Display & Keyboard plan and context
  - phase: 14
    provides: IPC layer with CalcStateView, handle_op, types.rs, commands.rs
provides:
  - Tailwind-free Vite config and CSS reset baseline for Wave 1 CSS work
  - RED test stubs in types.rs (y_str/z_str/t_str/lastx_str/in_eex_mode fields)
  - RED test stubs in commands.rs (eex_chs key routing)
affects:
  - 15-02 (Wave 1 Rust implementation — must turn these RED tests GREEN)
  - 15-03 (Wave 2 CSS — builds on Tailwind-free index.css)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED-first: write failing tests before implementing; compile errors count as RED"
    - "Tailwind removed from Vite plugin chain; vanilla CSS reset as baseline"

key-files:
  created: []
  modified:
    - hp41-gui/src/index.css
    - hp41-gui/vite.config.ts
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/commands.rs

key-decisions:
  - "Tailwind package left in node_modules/package.json; only removed from Vite plugin chain and CSS import"
  - "Compile errors on y_str/z_str/t_str/lastx_str/in_eex_mode fields count as RED (Wave 1 must fix)"
  - "eex_chs branch in handle_op deferred to Wave 1; Wave 0 RED tests verify the contract"

patterns-established:
  - "Wave 0 = TDD RED baseline: test stubs must fail before any Wave 1 implementation"
  - "Vanilla CSS base reset (box-sizing: border-box + body margin/padding zero) is the Phase 15 CSS baseline"

requirements-completed:
  - DISP-01
  - IPC-02

# Metrics
duration: 3min
completed: "2026-05-09"
---

# Phase 15 Plan 01: Display & Keyboard — Wave 0 TDD Baseline Summary

**Tailwind stripped from Vite/CSS pipeline and four RED compile-failing Rust test stubs planted for y_str/z_str/t_str/lastx_str/in_eex_mode fields and eex_chs key routing**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-09T19:18:36Z
- **Completed:** 2026-05-09T19:21:XX Z
- **Tasks:** 2 / 2
- **Files modified:** 4

## Accomplishments
- Replaced `@import "tailwindcss"` with plain `box-sizing: border-box` CSS reset; cleared path for hand-written Phase 15 styles
- Removed `tailwindcss()` Vite plugin (and its import) from `vite.config.ts`; `react()` plugin retained
- Added two RED test stubs to `types.rs` test module: `test_phase15_stack_fields_exist` and `test_in_eex_mode_false_without_e` — both cause compile errors on `y_str`, `z_str`, `t_str`, `lastx_str`, `in_eex_mode` fields that Wave 1 must add
- Added two RED test stubs to `commands.rs` test module: `test_eex_chs_toggles_exponent_sign` and `test_eex_chs_noop_without_e` — verify the `eex_chs` branch contract for Wave 1

## Task Commits

Each task was committed atomically:

1. **Task 1: Strip Tailwind from index.css and vite.config.ts** - `286af10` (chore)
2. **Task 2: Write RED test stubs for Phase 15 Rust changes** - `6e5883e` (test)

## Files Created/Modified
- `hp41-gui/src/index.css` - Replaced single `@import "tailwindcss"` line with box-sizing reset
- `hp41-gui/vite.config.ts` - Removed `tailwindcss` import and `tailwindcss()` plugin call
- `hp41-gui/src-tauri/src/types.rs` - Added 2 RED test stubs in `mod tests`
- `hp41-gui/src-tauri/src/commands.rs` - Added 2 RED test stubs in `mod tests`

## Decisions Made
- Tailwind devDependency left in `package.json` and `node_modules` intact — Phase 16 may reconsider D-10; only the active pipeline usage is removed
- Compile errors on missing struct fields are accepted as the RED state; Wave 1 will add the fields and the tests will turn green
- `eex_chs` tests assert `Ok` return (not `Err`) — Wave 1 must add the branch before `key_map::resolve`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Wave 1 executor (Plan 15-02) has clear RED targets: 6 compile errors on `CalcStateView` fields plus 2 runtime failures on `eex_chs` routing
- `index.css` is a clean baseline — Wave 2 CSS work (Plan 15-03) can write styles without Tailwind conflict
- All 4 pre-existing tests (`test_dispatch_op_unknown_key`, `test_print_buffer_drained`, `test_dispatch_op_payload_size`, `test_calc_state_view_structure`, `test_annunciators_from_state`, `test_gui_error_from_hp_error`) remain unmodified

## Self-Check

- [x] hp41-gui/src/index.css exists and contains `box-sizing: border-box`
- [x] hp41-gui/vite.config.ts exists and has no `tailwindcss` references
- [x] hp41-gui/src-tauri/src/types.rs contains `test_phase15_stack_fields_exist`
- [x] hp41-gui/src-tauri/src/commands.rs contains `test_eex_chs_toggles_exponent_sign`
- [x] Commit `286af10` exists (Task 1)
- [x] Commit `6e5883e` exists (Task 2)
- [x] cargo test fails with E0609 errors on the new fields (RED confirmed)

## Self-Check: PASSED

---
*Phase: 15-display-and-keyboard*
*Completed: 2026-05-09*
