---
phase: 05-persistence-and-ux
plan: 03
subsystem: persistence
tags: [rust, serde, serde_json, dirs, persistence, autosave, clap, ratatui]

# Dependency graph
requires:
  - phase: 05-persistence-and-ux
    plan: 01
    provides: CalcState with Serialize/Deserialize; user_mode/key_assignments fields
  - phase: 05-persistence-and-ux
    plan: 02
    provides: Op::UserMode/AlphaBackspace variants (needed for full Op serde round-trip in state files)

provides:
  - persistence::save_state(path, &CalcState) — writes versioned JSON with parent dir creation
  - persistence::load_state(path) -> Result<CalcState> — reads JSON, resets is_running=false
  - persistence::default_state_path() -> PathBuf — ~/.hp41/autosave.json with fallback
  - persistence::StateFile wrapper { version: 1, state: CalcState } (forward compat D-06)
  - App::new(state: CalcState, state_path: PathBuf) — updated constructor with Phase 5 fields
  - App::check_autosave() — testable 30s auto-save timer extracted from run()
  - PendingInput enum — all modal input variants for Plans 05-07
  - main.rs --state-file clap arg with startup load/fallback logic
  - help_data.rs stub + programs.rs stub (Plan 04 will fill)

affects:
  - 05-04 (help_data.rs stub declared; App fields show_help/show_programs wired)
  - 05-05 (programs.rs stub declared; App field show_programs wired)
  - 05-06 (PendingInput variants StoRegister/RclRegister/StoArith available)
  - 05-07 (PendingInput variants AssignKey/AssignLabel available; user_mode persisted)

# Tech tracking
tech-stack:
  added:
    - dirs = "6" (already added in Plan 01 — consumed here for home_dir())
    - serde/serde_json workspace deps (already added in Plan 01 — consumed here)
  patterns:
    - StateFile wrapper pattern: { version: 1, state: {...} } enables future migration without breaking saves
    - Instant subtraction for timer tests: `Instant::now() - Duration::from_secs(31)` to manipulate last_save in tests
    - RefCell<TableState> pattern for immutable draw() with mutable render_stateful_widget (RESEARCH Pitfall 1)
    - persistence::load_state returns Err (not panic) on all failure modes — checked via .is_err() in tests

key-files:
  created:
    - hp41-cli/src/persistence.rs (StateFile, save_state, load_state, default_state_path, 5 inline tests)
    - hp41-cli/src/help_data.rs (stub — Plan 04)
    - hp41-cli/src/programs.rs (stub — Plan 04)
  modified:
    - hp41-cli/src/app.rs (Phase 5 fields, PendingInput enum, check_autosave(), Ctrl+S, help/programs toggles, updated new())
    - hp41-cli/src/main.rs (--state-file arg, startup load, App::new(state, path))
    - hp41-cli/src/prgm_display.rs (Rule 3 fix: UserMode + AlphaBackspace arms added)
    - hp41-cli/src/tests/keys_tests.rs (Rule 1 fix: App::new() call sites updated)

key-decisions:
  - "StateFile wrapper with version field enables forward-compatible migration (D-06) — serde_json::from_reader returns Err on unknown version; future code handles migration"
  - "check_autosave() extracted from run() for testability — test directly manipulates last_save = Instant::now() - Duration::from_secs(31) to trigger timer without sleeping (BLOCKER 2 fix)"
  - "Instant::now() subtraction uses std::ops::Sub<Duration> — stable API, no unsafe needed"
  - "PendingInput as pub enum on App (not CalcState) — transient UI state not serialized (D-08)"
  - "RefCell<TableState> for help/programs overlays — single-threaded draw() stays immutable (D-08 / RESEARCH Pitfall 1)"

requirements-completed: [PERS-01, PERS-02]

# Metrics
duration: ~4min
completed: 2026-05-07
---

# Phase 5 Plan 03: Persistence Layer Summary

**Full persistence layer implemented: save/load JSON module with version wrapper, --state-file CLI arg, 30s auto-save timer extracted to check_autosave() for testability, Ctrl+S manual save, and save-on-exit**

## Performance

- **Duration:** ~4 min
- **Completed:** 2026-05-07
- **Tasks:** 3 (+ 2 Rule auto-fixes)
- **Files created:** 3, **Files modified:** 4

## Accomplishments

- `persistence.rs` created with `StateFile { version: 1, state }` wrapper, `save_state()`, `load_state()`, `default_state_path()`, and 5 inline tests covering all STRIDE threats (T-05-05 through T-05-08)
- `load_state()` always resets `is_running = false` after deserialization — prevents resuming mid-execution on reload (RESEARCH Pitfall 4)
- `default_state_path()` uses `dirs::home_dir()` with `.unwrap_or_else(|| PathBuf::from("."))` — no panic when home dir unavailable (T-05-08)
- `main.rs` updated with `--state-file <FILE>` clap arg; startup uses `load_state` on success, `CalcState::new()` + status bar message on corrupt file, silent fresh start on missing file (D-03)
- `App::new()` updated to `new(state: CalcState, state_path: PathBuf)` — all Phase 5 fields initialized
- `check_autosave()` extracted from `run()` loop — directly testable; test manipulates `last_save` to avoid 30s sleep
- `run()` calls `check_autosave()` each poll iteration; saves on graceful exit (D-05)
- `handle_key()` wired with Ctrl+S (manual save + status bar), `?` (help overlay toggle), Ctrl+P (program library toggle)
- All 310 workspace tests pass — zero regressions

## Task Commits

1. **Task 1: Create persistence.rs + fix prgm_display.rs** - `6c1c99d` (feat)
2. **Task 2: Wire --state-file in main.rs + stub modules** - `7d26dad` (feat)
3. **Task 3: Update app.rs with Phase 5 fields + check_autosave()** - `db560a1` (feat)

## Files Created/Modified

- `hp41-cli/src/persistence.rs` — created: StateFile, save_state, load_state, default_state_path, 5 tests
- `hp41-cli/src/help_data.rs` — created: stub for Plan 04
- `hp41-cli/src/programs.rs` — created: stub for Plan 04
- `hp41-cli/src/main.rs` — updated: --state-file arg, startup load, App::new(state, path)
- `hp41-cli/src/app.rs` — updated: Phase 5 fields, PendingInput enum, check_autosave(), Ctrl+S, overlays
- `hp41-cli/src/prgm_display.rs` — fixed (Rule 3): added UserMode + AlphaBackspace arms to op_display_name
- `hp41-cli/src/tests/keys_tests.rs` — fixed (Rule 1): updated App::new() calls to new signature

## Decisions Made

- `StateFile` wrapper with `version` field chosen (D-06) — future migration path without breaking existing saves
- `check_autosave()` extracted from `run()` — critical for testability; `Instant::now() - Duration::from_secs(31)` is a stable, idiomatic way to manipulate Instant in tests without `pub(test)` hacks
- `RefCell<TableState>` for table state in `App` — draw() takes `&self` (immutable, required by ratatui borrow rules); `borrow_mut()` is safe because draw is never re-entrant in single-threaded TUI
- Stub files `help_data.rs` and `programs.rs` created immediately to allow `mod` declarations in `main.rs` — Plan 04 will fill them

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] prgm_display.rs non-exhaustive match on Op enum**
- **Found during:** Pre-task compile check
- **Issue:** Plan 02 added `Op::UserMode` and `Op::AlphaBackspace` to the `Op` enum, but `prgm_display.rs` has an exhaustive match on `Op` in `op_display_name()`. This caused a compile error blocking all tests.
- **Fix:** Added `Op::UserMode => "USER".to_string()` and `Op::AlphaBackspace => "\u{2190}".to_string()` arms
- **Files modified:** `hp41-cli/src/prgm_display.rs`
- **Commit:** 6c1c99d (included in Task 1 commit)

**2. [Rule 1 - Bug] keys_tests.rs used old App::new() signature**
- **Found during:** Task 1 test run (after Task 3 app.rs changes)
- **Issue:** `App::new()` was changed from `new() -> Self` to `new(state: CalcState, state_path: PathBuf) -> Self`. All 8 `App::new()` call sites in `tests/keys_tests.rs` broke.
- **Fix:** Introduced `make_app()` helper — `App::new(CalcState::new(), PathBuf::from("/tmp/hp41_test.json"))` — and replaced all call sites
- **Files modified:** `hp41-cli/src/tests/keys_tests.rs`
- **Commit:** db560a1 (included in Task 3 commit)

## Known Stubs

| File | Content | Reason |
|------|---------|--------|
| `hp41-cli/src/help_data.rs` | Empty stub comment | Plan 04 adds help overlay data |
| `hp41-cli/src/programs.rs` | Empty stub comment | Plan 05 adds sample program library |

These stubs are intentional — they allow `mod help_data;` and `mod programs;` to be declared in `main.rs` now so the declaration order is stable. Plan 04 (help overlay) and Plan 05 (sample programs) will fill them.

## Threat Model Coverage

| Threat | Status |
|--------|--------|
| T-05-05: Tampering — corrupt JSON in load_state() | Covered: serde_json returns Err; main.rs catches and shows status bar message, uses CalcState::new() |
| T-05-06: DoS — disk full on save | Covered: save_state returns Err; check_autosave shows one-time warning, retries next tick; Ctrl+S shows "Save failed: ..." |
| T-05-07: Tampering — path traversal via --state-file | Accepted: single-user local CLI, no privilege boundary |
| T-05-08: DoS — home_dir() returns None | Covered: unwrap_or_else(|| PathBuf::from(".")) in default_state_path() |

## Self-Check

### Created files exist

- `hp41-cli/src/persistence.rs` — exists
- `hp41-cli/src/help_data.rs` — exists
- `hp41-cli/src/programs.rs` — exists

### Commits exist

- 6c1c99d — Task 1 (persistence.rs + prgm_display.rs fix)
- 7d26dad — Task 2 (main.rs --state-file + stubs)
- db560a1 — Task 3 (app.rs Phase 5 fields)

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
