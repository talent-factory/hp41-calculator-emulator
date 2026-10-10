---
phase: 64-interactive-getkey
plan: 03
subsystem: gui-tauri
tags: [rust, tauri, gui, getkey, wait-for-key, ipc, permissions, sc-4]

# Dependency graph
requires:
  - phase: 64-interactive-getkey
    plan: 01
    provides: YieldKind::WaitForKey + resume_program_with_key() in hp41-core

provides:
  - WaitForKey projected as "wait_for_key" through CalcStateView (types.rs YieldView match arm)
  - resume_program_with_key Tauri command (SC-4 thin-glue, commands.rs)
  - Command registered in invoke_handler (lib.rs)
  - Permission TOML (resume-program-with-key.toml, identifier allow-resume-program-with-key)
  - Capability entry in default.json
  - PRGM-03-j unit test in types.rs

affects:
  - 64-04 (GUI TS: App.tsx yield-driver guard + invokeForKey WaitForKey routing uses this command)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SC-4 thin-glue command: keycode: u8 first, State last (Tauri v2 param ordering)"
    - "WaitForKey projection: kind=wait_for_key, text='', resume_ms=0 (event-driven, no timer)"
    - "Permission TOML naming: <kebab-case>.toml (no allow- prefix) per check-tauri-permissions.sh CI gate"

key-files:
  created:
    - hp41-gui/src-tauri/permissions/resume-program-with-key.toml
  modified:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/capabilities/default.json

key-decisions:
  - "64-03-D01: Permission TOML filename must be <kebab-case>.toml without allow- prefix — check-tauri-permissions.sh CI gate maps command name snake_case→kebab-case, not allow-<kebab-case>"
  - "64-03-D02: Tauri v2 param ordering — keycode: u8 (custom) FIRST, State<AppState> LAST; mirrors existing commands"
  - "64-03-D03: SC-4 invariant maintained — no calculator logic in commands.rs; pure lock→core→drain→from_state"

requirements-completed: [PRGM-03]

# Metrics
duration: 6min
completed: 2026-06-07
---

# Phase 64 Plan 03: GUI Tauri GETKEY IPC Surface Summary

**WaitForKey projected as "wait_for_key" via CalcStateView; resume_program_with_key Tauri command wired, permitted, and callable from the TS frontend**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-06-07T08:28:17Z
- **Completed:** 2026-06-07T08:34:40Z
- **Tasks:** 3
- **Files modified:** 4 (+ 1 created)

## Accomplishments

- Extended `YieldView::from_yield_state` match with `YieldKind::WaitForKey => "wait_for_key"` arm — fixed the compile error that blocked the GUI crate since Plan 01 added the variant
- Added `from_state_projects_wait_for_key` unit test (PRGM-03-j): asserts `kind == "wait_for_key"`, `resume_ms == 0`, `text == ""`
- Added `resume_program_with_key(keycode: u8, state: State<AppState>)` SC-4 thin-glue command in `commands.rs`
- Registered command in `lib.rs` invoke_handler after `resume_program` (Phase 64 comment block)
- Created `resume-program-with-key.toml` permission TOML with identifier `allow-resume-program-with-key`
- Added `"allow-resume-program-with-key"` to `capabilities/default.json`
- All 130 GUI Rust tests pass; `just ci` green; `just gui-ci` green; iOS-target check clean

## Task Commits

1. **Task 1: WaitForKey projection arm + PRGM-03-j test** - `4d437f7` (feat)
2. **Task 2: resume_program_with_key command + lib.rs registration** - `e4748b7` (feat)
3. **Task 3: Permission TOML + capability entry** - `362e4c5` (feat)
4. **Task 3 fix: Rename TOML to match CI script** - `280dc73` (fix)

## Files Created/Modified

- `hp41-gui/src-tauri/src/types.rs` — `WaitForKey => "wait_for_key"` match arm in `from_yield_state`; `from_state_projects_wait_for_key` unit test
- `hp41-gui/src-tauri/src/commands.rs` — `resume_program_with_key` SC-4 thin-glue Tauri command
- `hp41-gui/src-tauri/src/lib.rs` — `commands::resume_program_with_key` in invoke_handler
- `hp41-gui/src-tauri/permissions/resume-program-with-key.toml` — Tauri v2.11 permission TOML
- `hp41-gui/src-tauri/capabilities/default.json` — `"allow-resume-program-with-key"` capability entry

## Decisions Made

- **64-03-D01 (TOML filename):** The CI gate `check-tauri-permissions.sh` expects `<kebab-case>.toml` (no `allow-` prefix) — maps `snake_case` command name to `kebab-case` filename. All other permission TOMLs (e.g. `resume-program.toml`, `run-program.toml`) follow this naming. PLAN.md specified `allow-resume-program-with-key.toml` based on the identifier, not the filename — corrected by renaming.
- **64-03-D02 (Tauri v2 param ordering):** `keycode: u8` (custom param) is first, `State<'_, AppState>` is last — required by Tauri v2 command macro. Mirrors all other commands with custom params.
- **64-03-D03 (SC-4 maintained):** The `resume_program_with_key` command is exactly 5 lines: lock → core function call → drain print_buffer → drain event_buffer → from_state. No calculator logic in the GUI crate.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Permission TOML filename did not match CI script expectation**
- **Found during:** Task 3 verification (`just gui-ci` failed at `check-tauri-permissions.sh`)
- **Issue:** Created `allow-resume-program-with-key.toml` (following PLAN.md artifact path), but the CI permission-coverage script maps command `snake_case` → `kebab-case` filename without `allow-` prefix. Script expected `resume-program-with-key.toml`.
- **Fix:** Renamed to `resume-program-with-key.toml`; internal identifier `allow-resume-program-with-key` and capability JSON entry unchanged.
- **Files modified:** permissions directory (rename)
- **Commit:** `280dc73` (fix)

## Known Stubs

None. All projections are wired to live hp41-core data. The `resume_program_with_key` command calls the real hp41-core function; no placeholder values.

## Threat Flags

No new threat surface beyond what the plan's STRIDE register covers. The `T-64-GUI-02` mitigation (permission gating via `allow-resume-program-with-key` in `capabilities/default.json`) is implemented as planned.

## Self-Check

- [x] `hp41-gui/src-tauri/src/types.rs` — `WaitForKey => "wait_for_key"` arm present; test present
- [x] `hp41-gui/src-tauri/src/commands.rs` — `resume_program_with_key` command defined
- [x] `hp41-gui/src-tauri/src/lib.rs` — command registered in invoke_handler
- [x] `hp41-gui/src-tauri/permissions/resume-program-with-key.toml` — exists with correct identifier
- [x] `hp41-gui/src-tauri/capabilities/default.json` — contains `"allow-resume-program-with-key"`
- [x] SC-4 guard empty (no `fn op_*` calculator logic in GUI src)
- [x] `cargo check` host clean; `cargo check --target aarch64-apple-ios` clean (0 errors)
- [x] `just ci` green (3049+ hp41-core tests); `just gui-ci` green (130 GUI Rust + 341 GUI TS tests)
- [x] Commits: `4d437f7`, `e4748b7`, `362e4c5`, `280dc73` all present in git log

## Self-Check: PASSED

---
*Phase: 64-interactive-getkey*
*Completed: 2026-06-07*
