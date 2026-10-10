---
phase: 67-reset-escape-hatch
plan: 03
subsystem: gui
tags: [rust, tauri, hp41-gui, reset, persistence, ipc]

# Dependency graph
requires:
  - phase: 67-01
    provides: "CalcState::soft_reset() and CalcState::memory_lost() core methods"
provides:
  - "reset_soft Tauri command — locks AppState, calls soft_reset(), persists under mutex, returns CalcStateView"
  - "reset_full Tauri command — locks AppState, calls memory_lost(), persists under mutex, returns CalcStateView"
  - "permissions/reset-soft.toml + reset-full.toml — Tauri v2.11 allow-reset-soft/full"
  - "Round-trip autosave-overwrite tests proving persisted-trap recovery"
affects:
  - phase 67-04 (iOS ON-key wiring calls reset_soft/reset_full via Tauri IPC)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Persist-under-lock (T-67-06 ordering invariant): reset + save_state() called within the same AppState mutex critical section so the auto-save thread cannot write back a stale pre-reset snapshot"
    - "GuiError { message } for io::Error — GuiError only has From<HpError>; io::Error mapped via .map_err(|e| GuiError { message: e.to_string() })"
    - "Test: use serialized trapping fields (prgm_mode, user_mode) not serde(skip) fields (display_override) for round-trip assertions"

key-files:
  created:
    - hp41-gui/src-tauri/permissions/reset-soft.toml
    - hp41-gui/src-tauri/permissions/reset-full.toml
  modified:
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/capabilities/default.json
    - hp41-gui/src-tauri/src/persistence.rs

key-decisions:
  - "67-03-D01: persist INSIDE the AppState mutex lock (T-67-06 ordering invariant) — prevents the auto-save background thread from racing and writing a stale pre-reset snapshot"
  - "67-03-D02: return type Result<CalcStateView, GuiError> (not Result<(), String> like save_state) — mirrors run_program pattern so the frontend can update display without a separate get_state round-trip"
  - "67-03-D03: display_override has #[serde(default, skip)] and is never persisted — round-trip tests use prgm_mode/user_mode as serialized trapping indicators"

requirements-completed: [RESET-01]

# Metrics
duration: 7min
completed: 2026-06-10
---

# Phase 67 Plan 03: GUI reset_soft + reset_full Tauri Commands Summary

**Two Tauri commands (`reset_soft`/`reset_full`) lock AppState, call the core reset methods, persist the reset state to the autosave while holding the mutex (T-67-06 race-prevention), and return CalcStateView; round-trip tests confirm a persisted trap is recoverable after relaunch**

## Performance

- **Duration:** 7 min
- **Started:** 2026-06-10T13:17:09Z
- **Completed:** 2026-06-10T13:24:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added `reset_soft` and `reset_full` `#[tauri::command]` functions in `commands.rs`, each implementing the persist-under-lock ordering invariant (T-67-06): lock AppState, call core reset method, persist to autosave WHILE HOLDING THE MUTEX, drain print/event buffers, return `CalcStateView`
- Created `permissions/reset-soft.toml` and `permissions/reset-full.toml` with `allow-reset-soft`/`allow-reset-full` identifiers and correct `commands.allow` entries
- Added `"allow-reset-soft"` and `"allow-reset-full"` to `capabilities/default.json` permissions array
- Registered `commands::reset_soft` and `commands::reset_full` in `generate_handler!` in `lib.rs`
- Added two round-trip tests in `persistence.rs`: `test_reset_soft_overwrites_persisted_trap` and `test_reset_full_overwrites_persisted_state`, each proving the overwrite-survives-restart contract
- All 136 GUI crate tests pass; SC-4 invariant unaffected

## Task Commits

Each task was committed atomically:

1. **Task 1: reset_soft + reset_full commands + permissions** — `1ef0a2e` (feat)
2. **Task 2: Round-trip autosave-overwrite tests** — `5d2c658` (test)

**Plan metadata:** (this commit)

## Files Created/Modified

- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/src/commands.rs` — Added `reset_soft` + `reset_full` commands after `save_state` (Phase 67 Reset Escape Hatch section, ~line 679)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/src/lib.rs` — Registered `commands::reset_soft` and `commands::reset_full` in `generate_handler!`
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/permissions/reset-soft.toml` — New permission TOML for `allow-reset-soft`
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/permissions/reset-full.toml` — New permission TOML for `allow-reset-full`
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/capabilities/default.json` — Added `allow-reset-soft` + `allow-reset-full` to permissions array
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src-tauri/src/persistence.rs` — Added 2 round-trip overwrite tests in the existing test module

## Decisions Made

- **67-03-D01:** Persist INSIDE the AppState mutex lock (T-67-06 ordering invariant). The `persistence::save_state` call appears within the same `MutexGuard` scope as the reset method call — the auto-save background thread acquires the same mutex AFTER the command returns, ensuring it serializes the already-reset state, not a stale pre-reset snapshot.
- **67-03-D02:** Return type `Result<CalcStateView, GuiError>` (not `Result<(), String>` like `save_state`) — mirrors the `run_program` pattern so the frontend refreshes its display immediately without a separate `get_state` round-trip.
- **67-03-D03:** `display_override` has `#[serde(default, skip)]` and is never persisted. Round-trip tests use `prgm_mode` and `user_mode` (both serialized, no `skip`) as the trapping indicators — discovered during test authoring (auto-fixed, Rule 1).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed incorrect use of display_override in round-trip test**
- **Found during:** Task 2 (first test run — assertion `display_override must survive reload` failed)
- **Issue:** `display_override` has `#[serde(default, skip)]` — it is intentionally not serialized. The initial test asserted it would survive a save/load round-trip, which is wrong by design.
- **Fix:** Replaced `display_override = Some(...)` trapping indicator with `prgm_mode = true` and `user_mode = true`, which ARE serialized and correctly model a persisted trap. Also removed the corresponding `display_override.is_none()` assertion from `test_reset_full_overwrites_persisted_state`.
- **Files modified:** `hp41-gui/src-tauri/src/persistence.rs`
- **Committed in:** `5d2c658`

---

**Total deviations:** 1 auto-fixed (incorrect serde assumption about display_override in test)
**Impact on plan:** Clarifies the contract: only serialized fields constitute a persisted trap; `display_override` is transient by design and clears on any reload.

## Issues Encountered

None beyond the auto-fixed test assertion error.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `reset_soft` and `reset_full` Tauri commands are ready for Plan 67-04 (iOS ON-key tap/long-press wiring)
- Both commands are registered in `generate_handler!` and allowed in `capabilities/default.json`
- Round-trip tests confirm autosave-overwrite contract; all 136 GUI crate tests pass

## Known Stubs

None — both commands are fully wired: `soft_reset()` / `memory_lost()` are called, autosave is overwritten synchronously, and `CalcStateView` is returned.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns beyond what was planned. The two new Tauri IPC commands are gated by the Tauri capability permission system (`allow-reset-soft`, `allow-reset-full`) — consistent with every other command in this project (T-67-07 mitigated). No new threat surface beyond the plan's threat register.

## Self-Check: PASSED

- `hp41-gui/src-tauri/src/commands.rs` — FOUND (pub fn reset_soft, pub fn reset_full)
- `hp41-gui/src-tauri/src/lib.rs` — FOUND (commands::reset_soft, commands::reset_full in generate_handler!)
- `hp41-gui/src-tauri/permissions/reset-soft.toml` — FOUND
- `hp41-gui/src-tauri/permissions/reset-full.toml` — FOUND
- `hp41-gui/src-tauri/capabilities/default.json` — FOUND (allow-reset-soft, allow-reset-full)
- `hp41-gui/src-tauri/src/persistence.rs` — FOUND (test_reset_soft_overwrites_persisted_trap, test_reset_full_overwrites_persisted_state)
- Commit `1ef0a2e` — FOUND
- Commit `5d2c658` — FOUND
- `cargo test` 136 passed — VERIFIED
- SC-4 grep — EMPTY (no forbidden op_ functions in GUI crate)

---
*Phase: 67-reset-escape-hatch*
*Completed: 2026-06-10*
