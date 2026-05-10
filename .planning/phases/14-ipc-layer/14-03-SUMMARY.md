---
phase: 14-ipc-layer
plan: 03
subsystem: infra
tags: [tauri, permissions, capabilities, acl]

requires:
  - phase: 14-02
    provides: Tauri commands dispatch_op/get_state registered in generate_handler![]

provides:
  - hp41-gui Tauri ACL capabilities scoped to main window with explicit IPC permissions
  - Permission TOML files for dispatch_op and get_state commands

affects: [Phase 15 Display & Keyboard, Phase 18 CI/CD]

tech-stack:
  added: []
  patterns: [Tauri v2 app-command permission TOML pattern]

key-files:
  created:
    - hp41-gui/src-tauri/permissions/dispatch-op.toml
    - hp41-gui/src-tauri/permissions/get-state.toml
  modified:
    - hp41-gui/src-tauri/capabilities/default.json

key-decisions:
  - "Permission TOML files required: Tauri v2.11 does NOT auto-generate allow-dispatch-op / allow-get-state for inline app commands — only for plugin commands. TOML files in src-tauri/permissions/ must be created first, then referenced in capabilities."
  - "Permission IDs confirmed as allow-dispatch-op and allow-get-state (kebab-case from snake_case function names)"
  - "windows: [main] preserved — no wildcard scoping"
  - "core:default preserved from Phase 13 baseline"

patterns-established:
  - "Tauri v2 app-command permissions: create TOML in src-tauri/permissions/<command-kebab>.toml with [[permission]] identifier + commands.allow = [\"function_name\"] pattern"
  - "capabilities/default.json references permission IDs directly (no namespace prefix)"

requirements-completed:
  - IPC-01

duration: 10min
completed: 2026-05-09
---

# Phase 14-03: IPC Capabilities Summary

**Tauri v2 ACL wired: explicit `allow-dispatch-op` and `allow-get-state` permissions created via TOML files and scoped to main window in capabilities/default.json**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-09T21:30:00Z
- **Completed:** 2026-05-09T21:40:00Z
- **Tasks:** 1
- **Files modified:** 3 (2 new TOML + 1 updated JSON)

## Accomplishments
- Created `src-tauri/permissions/dispatch-op.toml` — registers `allow-dispatch-op` as a valid Tauri permission for the `dispatch_op` command
- Created `src-tauri/permissions/get-state.toml` — registers `allow-get-state` as a valid Tauri permission for the `get_state` command
- Updated `capabilities/default.json` to list both permissions explicitly plus `core:default`, scoped to `["main"]`
- `cargo check` passes with no unknown permission identifier errors
- All 9 Phase 14 unit tests remain GREEN

## Task Commits

Each task was committed atomically:

1. **Task 1: Capabilities JSON + permission TOML files** — (committed inline by orchestrator)

**Plan metadata:** (committed inline by orchestrator)

## Files Created/Modified
- `hp41-gui/src-tauri/permissions/dispatch-op.toml` — Declares `allow-dispatch-op` permission for the dispatch_op Tauri command
- `hp41-gui/src-tauri/permissions/get-state.toml` — Declares `allow-get-state` permission for the get_state Tauri command
- `hp41-gui/src-tauri/capabilities/default.json` — Updated permissions array: core:default + allow-dispatch-op + allow-get-state, windows: ["main"]

## Decisions Made

**CRITICAL deviation from RESEARCH.md Pattern 7:** The plan's Pattern 7 assumed that Tauri v2 auto-generates `allow-dispatch-op` / `allow-get-state` permission identifiers in `gen/schemas/` during `cargo check`. This is INCORRECT for Tauri v2.11 with inline app commands (not plugins).

**What actually happens in Tauri v2.11:**
- Auto-generated permissions (`allow-<kebab-name>`) are generated ONLY for Tauri PLUGIN commands
- App-level inline commands (registered via `generate_handler![]`) do NOT get auto-generated permissions
- To use explicit permission IDs for app commands, TOML files must be created manually in `src-tauri/permissions/`
- These TOML files use `[[permission]] identifier = "allow-<kebab-name>"` + `commands.allow = ["snake_case_fn_name"]`

**Permission IDs confirmed:**
- `allow-dispatch-op` (from function `dispatch_op`)
- `allow-get-state` (from function `get_state`)

## Deviations from Plan

### Auto-fixed Issues

**1. Tauri v2 app-command permissions not auto-generated**
- **Found during:** Task 1 (discovering permission IDs from gen/schemas/)
- **Issue:** Pattern 7 in RESEARCH.md stated permissions appear in `gen/schemas/` after `cargo check`. In practice (Tauri v2.11), they do NOT for inline app commands — only for plugin commands. Adding `allow-dispatch-op` to capabilities/default.json without the TOML files causes build error: `Permission allow-dispatch-op not found`.
- **Fix:** Created two TOML files in `src-tauri/permissions/` that declare the custom permissions using Tauri v2's permission definition schema. The TOML format `[[permission]] identifier / commands.allow` is the standard Tauri v2 mechanism for app-level command permissions.
- **Files modified:** hp41-gui/src-tauri/permissions/dispatch-op.toml (new), hp41-gui/src-tauri/permissions/get-state.toml (new)
- **Verification:** `cargo check` exits 0 with the new permissions in capabilities/default.json

---

**Total deviations:** 1 auto-fixed (RESEARCH.md planning assumption corrected at execution time)
**Impact on plan:** D-12 satisfied. No scope creep — the TOML files are the standard Tauri v2 pattern for this scenario.

## Issues Encountered
- RESEARCH.md Pattern 7 contained an incorrect assumption about Tauri v2.11 auto-permission generation. The correct Tauri v2 mechanism for app-level command permissions uses TOML files in `src-tauri/permissions/`. This required creating 2 additional files not in the plan's `files_modified` list — both are minimal (6 lines each) and follow official Tauri v2 conventions.

## Next Phase Readiness

Phase 14 IPC Layer is complete:
- All 5 success criteria verified (SC-1 through SC-5)
- `dispatch_op` and `get_state` commands are accessible from the main window
- Capability scoping prevents arbitrary window invocation (T-14-01 mitigation)
- 9/9 unit tests GREEN
- `just ci` passes (hp41-core coverage 89.89% > 80% gate)

**For Phase 15:** React frontend can invoke `invoke('dispatch_op', { keyId: 'sin' })` and `invoke('get_state')` via the standard `@tauri-apps/api/core` package. The response types are `CalcStateView` (with display_str, x_str, annunciators, print_lines) and `GuiError`. TypeScript type shims for these Rust structs are deferred to Phase 15.

**For Phase 18 CI/CD:** `just gui-check` should be added to the GUI CI job per D-13 (not to the existing `just ci`).

**Note on autogenerated TypeScript types:** The `@tauri-apps/api` tauri-bindgen step (generating TS interfaces from Rust structs) is out of scope for Phase 14 and deferred to Phase 15.

---
*Phase: 14-ipc-layer*
*Completed: 2026-05-09*
