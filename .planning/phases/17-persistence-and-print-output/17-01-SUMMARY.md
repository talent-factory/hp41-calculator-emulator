---
phase: 17-persistence-and-print-output
plan: "01"
subsystem: hp41-gui persistence
tags: [persistence, tauri, rust, auto-save, interop]
dependency_graph:
  requires: []
  provides: [hp41-gui persistence module, startup state load, 30s auto-save thread]
  affects: [hp41-gui/src-tauri/src/lib.rs, hp41-gui/src-tauri/Cargo.toml]
tech_stack:
  added: [dirs = "6"]
  patterns: [std::thread::spawn auto-save loop, unwrap_or_else poisoned-mutex recovery, serde StateFile version wrapper]
key_files:
  created:
    - hp41-gui/src-tauri/src/persistence.rs
  modified:
    - hp41-gui/src-tauri/Cargo.toml
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/Cargo.lock
decisions:
  - "D-05: Copy persistence.rs from hp41-cli rather than creating a shared crate — avoids workspace complexity for ~100 LOC module"
  - "D-06: StateFile { version: u32, state: CalcState } wrapper enables CLI/GUI interop via shared autosave.json path"
  - "D-03: load_state() failures fall back to CalcState::new() silently — startup never panics on missing/malformed file"
  - "D-01: std::thread::spawn loop with 30s sleep keeps auto-save off UI and IPC threads"
metrics:
  duration: "209s"
  completed: "2026-05-10"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 3
---

# Phase 17 Plan 01: GUI Persistence (Auto-Save & Startup Load) Summary

Persistence module added to hp41-gui Tauri backend: `persistence.rs` copied from hp41-cli with updated module comment, wired into `lib.rs setup()` to load `~/.hp41/autosave.json` on startup and auto-save every 30 seconds via a Rust background thread.

## What Was Implemented

### Task 1 — Add dirs dependency and create persistence.rs

- Added `dirs = "6"` to `hp41-gui/src-tauri/Cargo.toml` `[dependencies]`
- Created `hp41-gui/src-tauri/src/persistence.rs` as a verbatim copy of `hp41-cli/src/persistence.rs` with one change: the module-level doc comment now references `hp41-gui` and SC-4 CLI/GUI interoperability
- All four public symbols preserved: `StateFile`, `save_state()`, `load_state()`, `default_state_path()`
- All 6 test functions preserved under `#[cfg(test)] #[allow(clippy::unwrap_used)]`

### Task 2 — Wire persistence into lib.rs setup()

- Added `mod persistence;` before `pub type AppState` in `lib.rs`
- `setup()` now calls `persistence::load_state(&save_path)` and falls back to `CalcState::new()` on any error via `.unwrap_or_else(|_| CalcState::new())`
- Spawns a `std::thread::spawn` auto-save loop: sleep 30s → lock `AppState` via `handle.state::<AppState>()` → `persistence::save_state()` → drop `MutexGuard` before next sleep
- Poisoned Mutex recovery: `.unwrap_or_else(|e| e.into_inner())` (zero-panic policy)
- `app.handle().clone()` called inside `setup()` before thread spawn (satisfies `Send + 'static` bound)

## Files Modified

| File | Change | Key Lines |
|------|--------|-----------|
| `hp41-gui/src-tauri/Cargo.toml` | Add dirs dep | +1 line |
| `hp41-gui/src-tauri/src/persistence.rs` | New file | 149 lines |
| `hp41-gui/src-tauri/src/lib.rs` | Extend setup() | +23 lines |
| `hp41-gui/src-tauri/Cargo.lock` | Lock update for dirs | auto |

## Test Results

All 20 tests pass (3 suites):

**Persistence tests (6/6 new):**
- `test_roundtrip_fresh_state` — OK
- `test_missing_file_returns_err` — OK
- `test_corrupt_json_returns_err` — OK
- `test_is_running_reset_on_load` — OK
- `test_user_mode_roundtrip` — OK
- `test_version_field_in_json` — OK

**Existing tests still passing (14):** key_map (4), commands (3), types (5), main (0), doc (0)

## Build Results

- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` — exit 0, zero errors
- `cargo clippy --manifest-path hp41-gui/src-tauri/Cargo.toml` — zero warnings
- Zero bare `.unwrap()` calls in non-test production code

## Success Criteria Verification

| Criterion | Status |
|-----------|--------|
| `cargo check` exits 0 | PASS |
| All 6 persistence tests pass | PASS |
| `persistence.rs` contains StateFile, save_state, load_state, default_state_path | PASS |
| `grep -c 'dirs = ' Cargo.toml` returns 1 | PASS |
| `persistence::load_state` in lib.rs | PASS |
| `from_secs(30)` in lib.rs | PASS |
| No bare `unwrap()` in non-test code | PASS |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all data paths are wired. `load_state()` reads real filesystem; `save_state()` writes real filesystem; auto-save thread is live.

## Threat Surface Scan

No new threat surface beyond what was specified in the plan's threat model. The plan's T-17-01 mitigation (graceful JSON parse failure via `unwrap_or_else`) is implemented. T-17-02 (save failure logging to stderr without retry) is implemented. T-17-03 and T-17-04 are accepted risks per the plan.

## Commits

| Task | Commit | Message |
|------|--------|---------|
| Task 1 | c68f293 | feat(17-01): add dirs dependency and create persistence.rs for hp41-gui |
| Task 2 | a9eb1be | feat(17-01): wire persistence into lib.rs setup() with auto-save thread |
