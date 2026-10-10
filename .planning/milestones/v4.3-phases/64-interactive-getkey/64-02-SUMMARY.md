---
phase: 64-interactive-getkey
plan: 02
subsystem: cli-frontend
tags: [rust, cli, getkey, interactive, waitforkey, suspend-resume, hp41-cli]

# Dependency graph
requires:
  - phase: 64-interactive-getkey
    plan: 01
    provides: YieldKind::WaitForKey + resume_program_with_key(keycode: u8)

provides:
  - WaitForKey branch in drain_pending_yields (poll+redraw inner loop; Esc→0; Ctrl+C→quit)
  - handle_key early-return guard during WaitForKey suspend (prevents double-processing)
  - Three new CLI tests: F5=None contract, Esc-cancel sentinel 0, handle_key guard

affects:
  - 64-03 (GUI Rust: resume_program_with_key Tauri command)
  - 64-04 (GUI TS: App.tsx yield-driver WaitForKey routing)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "WaitForKey event-driven inner loop: poll(16ms)+redraw in drain_pending_yields; no blocking event::read()"
    - "Belt-and-suspenders handle_key guard: early-return on WaitForKey prevents double-processing from outer run() loop"
    - "Hardware-faithful key ignore: keycode_to_hp41_code None = no HP-41 equivalent = continue waiting"

key-files:
  created: []
  modified:
    - hp41-cli/src/app.rs

key-decisions:
  - "64-02-D01: F5 (TUI R/S) returns None from keycode_to_hp41_code — ignored during WaitForKey. HP-41 code 31 is unreachable from the current CLI keyboard map (TUI R/S is run/stop, not a capturable HP-41 key). This is hardware-faithful: only physical HP-41-equivalent keys are captured."
  - "64-02-D02: drain_pending_yields owns the event loop during WaitForKey. The handle_key guard is belt-and-suspenders (in practice run() does not call handle_key while drain_pending_yields blocks)."

requirements-completed: [PRGM-03]

# Metrics
duration: 20min
completed: 2026-06-07
---

# Phase 64 Plan 02: CLI GETKEY Frontend Wiring Summary

**WaitForKey branch in drain_pending_yields + handle_key guard: CLI suspends on GETKEY without freezing, captures next keypress, resumes with HP-41 row×col code**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-06-07
- **Completed:** 2026-06-07
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `YieldKind::WaitForKey` branch in `drain_pending_yields`: inner event loop with `poll(16ms)` + `terminal.draw(...)` per iteration (Pitfall 5 compliant — no blocking `event::read()` without poll guard); Esc → `resume_program_with_key(&mut self.state, 0)` (sentinel 0 = cancel); Ctrl+C → `self.exit = true; return Ok(())`; HP-41 keys (via `keycode_to_hp41_code`) → `resume_program_with_key(code)` break; F5/F7/F8/None/CTRL-mod keys ignored (hardware faithful)
- Added `handle_key` early-return guard (belt-and-suspenders): when `pending_yield.kind == WaitForKey`, return immediately without routing the key through digit/op/modal handlers
- Three new CLI tests: `test_keycode_to_hp41_code_f5_returns_none_rs_is_tui_only` (F5/F7/F8 = None contract), `test_getkey_esc_cancel_pushes_sentinel_zero` (Esc-cancel → X=0, pending_yield=None), `test_handle_key_ignores_keys_during_waitforkey_suspend` (guard fires correctly)
- All 497 hp41-cli tests pass; `just ci` green (95.09% line coverage)

## Task Commits

1. **Tasks 1+2: WaitForKey drain branch + handle_key guard + tests** - `d19f326` (feat)

Note: Tasks 1 and 2 were implemented together in a single pass on `hp41-cli/src/app.rs` and committed jointly.

## Files Created/Modified

- `hp41-cli/src/app.rs` — WaitForKey branch in `drain_pending_yields`, handle_key guard, three new tests

## Decisions Made

- **64-02-D01 (F5 not capturable as HP-41 keycode):** F5 (TUI R/S) returns `None` from `keycode_to_hp41_code` — ignored during WaitForKey. HP-41 code 31 (row 3, col 1 — the real R/S physical position) is not wired in the CLI keyboard map. The TUI R/S binding (F5) is a run/stop action, not a capturable HP-41 key event. Hardware faithful: GETKEY only captures physical HP-41-equivalent keys. The locked decision "R/S=31 via keycode_to_hp41_code, no special-casing" means no NEW special-casing is introduced for R/S in the WaitForKey path — F5 simply falls through the `None` arm and is ignored.
- **64-02-D02 (event loop ownership):** `drain_pending_yields` owns all crossterm events during the WaitForKey inner loop. The `run()` main loop does not call `handle_key` during this period (the `drain_pending_yields` call inside `run()` blocks until the inner loop breaks). The `handle_key` guard is belt-and-suspenders against future code-shape changes or edge cases (e.g. a stale Release event arriving after resume).

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written. The F5/code-31 ambiguity in the plan description was resolved by reading `keycode_to_hp41_code`: F5 returns `None`, so during WaitForKey F5 is ignored (hardware faithful). The test encodes this contract rather than asserting `Some(31)` for F5.

## Known Stubs

None — all behavior is fully implemented and tested.

## Threat Flags

None — no new trust boundaries or network/file surfaces introduced.

## Self-Check

- [x] `hp41-cli/src/app.rs` has `YieldKind::WaitForKey` branch in `drain_pending_yields`
- [x] `hp41-cli/src/app.rs` has handle_key guard for `WaitForKey`
- [x] Three new tests present and passing
- [x] Commit `d19f326` present in git log
- [x] `just ci` green (95.09% line coverage; 0 clippy errors; license-audit clean)
- [x] `cargo test -p hp41-cli` exits 0 (497 passed)

## Self-Check: PASSED

---
*Phase: 64-interactive-getkey*
*Completed: 2026-06-07*
