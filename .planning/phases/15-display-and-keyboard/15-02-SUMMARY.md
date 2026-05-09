---
phase: 15-display-and-keyboard
plan: "02"
subsystem: ui
tags: [tauri, rust, tdd, green-tests, ipc, stack-panel, eex-chs]

# Dependency graph
requires:
  - phase: 15-01
    provides: Wave 0 TDD RED test stubs for y_str/z_str/t_str/lastx_str/in_eex_mode and eex_chs
  - phase: 14
    provides: IPC layer with CalcStateView, handle_op, types.rs, commands.rs
provides:
  - CalcStateView with five new fields: y_str, z_str, t_str, lastx_str (String) and in_eex_mode (bool)
  - from_state() populating all five fields via format_hpnum and entry_buf.contains('e')
  - handle_op eex_chs branch before key_map::resolve() for exponent sign toggle
  - All 4 Wave 0 RED tests turned GREEN
affects:
  - 15-03 (Wave 2 CSS — stack panel React component can now read y_str/z_str/t_str/lastx_str/in_eex_mode)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wave 1 GREEN: implement fields targeted by Wave 0 RED stubs, run cargo test to confirm"
    - "eex_chs branch pattern: intercept before key_map::resolve(), mutate entry_buf directly, return early"
    - "Stack fields pattern: format_hpnum(&state.stack.y/z/t/lastx, &state.display_mode) — same call as x_str"

key-files:
  created: []
  modified:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/commands.rs

key-decisions:
  - "eex_chs block inserted before key_map::resolve() — no Op::EexChs variant needed, entry_buf mutated directly"
  - "Five new CalcStateView fields follow the same format_hpnum pattern as existing x_str"
  - "CalcStateView JSON remains ≤300 bytes after adding 5 fields (payload size gate continues to pass)"

patterns-established:
  - "Early-return pattern: digit/'.'/'e'/eex_chs blocks all use drain-and-return before key_map::resolve()"
  - "Stack field expansion: all five stack registers (x/y/z/t/lastx) now serialized in CalcStateView"

requirements-completed:
  - DISP-01
  - DISP-02
  - IPC-02

# Metrics
duration: 8min
completed: "2026-05-09"
---

# Phase 15 Plan 02: Display & Keyboard — Wave 1 Rust Implementation Summary

**CalcStateView extended with y_str/z_str/t_str/lastx_str/in_eex_mode and handle_op eex_chs branch wired — all 4 Wave 0 RED tests turned GREEN and 13 Rust tests pass**

## Performance

- **Duration:** 8 min
- **Started:** 2026-05-09T19:25:00Z
- **Completed:** 2026-05-09T19:33:00Z
- **Tasks:** 2 / 2
- **Files modified:** 2

## Accomplishments
- Added five new fields to `CalcStateView` struct: `y_str`, `z_str`, `t_str`, `lastx_str` (String), `in_eex_mode` (bool)
- `from_state()` now populates all five fields using `format_hpnum` for register strings and `entry_buf.contains('e')` for eex mode
- Inserted `eex_chs` early-return block in `handle_op` before `key_map::resolve()` — toggles exponent sign in entry_buf directly
- Turned all 4 Wave 0 RED compile-failing tests GREEN (`test_phase15_stack_fields_exist`, `test_in_eex_mode_false_without_e`, `test_eex_chs_toggles_exponent_sign`, `test_eex_chs_noop_without_e`)
- `CalcStateView` JSON remains ≤300 bytes (payload size gate passes)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend CalcStateView with stack fields and in_eex_mode** - `de6542c` (feat)
2. **Task 2: Add eex_chs branch to handle_op in commands.rs** - `9cf2104` (feat)

## Files Created/Modified
- `hp41-gui/src-tauri/src/types.rs` - Added 5 new fields to `CalcStateView` struct and populated them in `from_state()`
- `hp41-gui/src-tauri/src/commands.rs` - Added `eex_chs` early-return block before `key_map::resolve()` in `handle_op`

## Decisions Made
- `eex_chs` block uses `if let Some(e_pos) = calc.entry_buf.find('e')` — if-let with find() is bounds-safe, no index panic possible (T-15-02-01 mitigated)
- Reused the same `drain-and-return` pattern as the three existing early-return blocks in `handle_op` for consistency
- `in_eex_mode` computed as `state.entry_buf.contains('e')` — simple, no clone needed, idiomatic

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Wave 2 CSS work (Plan 15-03) can reference `y_str`/`z_str`/`t_str`/`lastx_str`/`in_eex_mode` in React components
- `handle_op("eex_chs")` is now a valid IPC key that React can dispatch
- All 13 Rust tests pass; `just gui-check` exits 0
- No clippy::unwrap_used violations in new code

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundary changes introduced. The `eex_chs` branch only mutates an in-memory `String` field on a mutex-protected `CalcState`. T-15-02-01, T-15-02-02, T-15-02-03 from the plan's threat model are all mitigated as designed.

## Self-Check

- [x] `hp41-gui/src-tauri/src/types.rs` contains `pub y_str: String`
- [x] `hp41-gui/src-tauri/src/types.rs` contains `pub in_eex_mode: bool`
- [x] `hp41-gui/src-tauri/src/commands.rs` contains `"eex_chs"` block before `key_map::resolve`
- [x] Commit `de6542c` exists (Task 1)
- [x] Commit `9cf2104` exists (Task 2)
- [x] `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0 (13 tests pass)
- [x] `just gui-check` exits 0

## Self-Check: PASSED

---
*Phase: 15-display-and-keyboard*
*Completed: 2026-05-09*
