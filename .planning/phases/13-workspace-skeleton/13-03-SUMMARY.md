---
phase: 13-workspace-skeleton
plan: "03"
subsystem: ui
tags: [tauri, tauri-v2, capabilities, json, npm, workspace-isolation]

# Dependency graph
requires:
  - phase: 13-workspace-skeleton
    plan: "01"
    provides: "hp41-gui/src-tauri Rust crate skeleton (Cargo.toml, build.rs, main.rs, lib.rs)"
  - phase: 13-workspace-skeleton
    plan: "02"
    provides: "hp41-gui frontend scaffold (package.json, vite.config.ts, tsconfig.json, index.html, src/)"
provides:
  - "hp41-gui/src-tauri/tauri.conf.json with bundle identifier ch.talent-factory.hp41 and window title HP-41 Calculator"
  - "hp41-gui/src-tauri/capabilities/default.json with core:default Tauri v2 permissions"
  - "npm dependencies installed (node_modules present)"
  - "Phase 13 workspace skeleton fully verified: SC-1 through SC-5 all pass"
  - "Human-confirmed: Tauri window opens with title HP-41 Calculator and exits cleanly"
affects:
  - "14-gui-ipc — must add hp41-specific permissions to capabilities/default.json when IPC commands are registered"
  - "14-gui-ipc — AppState type alias in hp41-gui/src-tauri/src/lib.rs is the extension point"
  - "14-gui-ipc — Mutex lock pattern: .unwrap_or_else(|e| e.into_inner()), never .unwrap() or .expect()"

# Tech tracking
tech-stack:
  added:
    - "tauri.conf.json — Tauri v2 app configuration (bundle id, window config, build hooks)"
    - "capabilities/default.json — Tauri v2 minimum capability declaration"
  patterns:
    - "Bundle identifier pattern: reverse-DNS ch.talent-factory.hp41 (not com.tauri.dev scaffold default)"
    - "Tauri v2 capability-first model: capabilities/default.json required for any IPC; additional permissions added per feature"
    - "Workspace isolation: hp41-gui is excluded from root Cargo.toml members; Tauri deps never enter root resolver"

key-files:
  created:
    - "hp41-gui/src-tauri/tauri.conf.json — Tauri app config: identifier, window title, devUrl, frontendDist, build hooks"
    - "hp41-gui/src-tauri/capabilities/default.json — Tauri v2 core:default permissions (minimum viable capability)"
  modified: []

key-decisions:
  - "Bundle identifier ch.talent-factory.hp41 (D-02): avoids macOS sandbox issues from scaffold default com.tauri.dev"
  - "Window title HP-41 Calculator (D-03): matches productName for consistency; confirmed by human observation"
  - "capabilities/default.json grants core:default only: minimum viable permissions; hp41-specific IPC permissions deferred to Phase 14"
  - "devUrl localhost:5173 matches vite.config.ts server.port:5173 (port alignment is a hard constraint)"
  - "frontendDist ../dist resolves correctly from src-tauri/ to hp41-gui/dist/ (Vite outDir)"

patterns-established:
  - "Tauri v2 config pattern: productName + identifier + build hooks + app.windows[] in tauri.conf.json"
  - "Capability-file pattern: capabilities/default.json with core:default is Phase 13 baseline; extend per IPC command in Phase 14"
  - "Workspace isolation pattern: cargo build --workspace at repo root MUST NOT compile Tauri code"

requirements-completed: [WSPC-01, WSPC-02]

# Metrics
duration: 35min
completed: 2026-05-09
---

# Phase 13 Plan 03: Tauri Configuration and Integration Verification Summary

**Tauri v2 app config (tauri.conf.json) and capabilities/default.json created; human confirmed window titled "HP-41 Calculator" opens and exits cleanly; all five Phase 13 success criteria SC-1 through SC-5 verified**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-09T (Wave 2, continuation of Phase 13)
- **Completed:** 2026-05-09
- **Tasks:** 2 (1 automated + 1 human-verify checkpoint)
- **Files created:** 2 (tauri.conf.json, capabilities/default.json)

## Accomplishments

- Created `hp41-gui/src-tauri/tauri.conf.json` with correct bundle identifier (`ch.talent-factory.hp41`), window title (`HP-41 Calculator`), devUrl (`http://localhost:5173`), and frontendDist (`../dist`)
- Created `hp41-gui/src-tauri/capabilities/default.json` with `core:default` permission — minimum Tauri v2 requirement for IPC
- Ran `npm install` in `hp41-gui/` — all dependencies installed (react 19.2.6, @tauri-apps/api 2.11.0, vite 8.0.11, tailwindcss 4.3.0)
- Human checkpoint passed: Tauri window opened with title "HP-41 Calculator" and exited cleanly (SC-1)
- All five Phase 13 success criteria confirmed

## Success Criteria Results

| ID | Type | Criterion | Result |
|----|------|-----------|--------|
| SC-1 | Human | Window titled "HP-41 Calculator" opens and exits cleanly | PASS — human confirmed |
| SC-2 | Auto | `just ci` completes with exit code 0 (no CLI regression) | PASS |
| SC-3 | Auto | `cargo build --workspace` does not compile Tauri code | PASS |
| SC-4 | Auto | `tauri`/`tauri-build` not in root Cargo.toml | PASS (`grep -c "tauri" Cargo.toml` returns 0) |
| SC-5 | Auto | CI matrix covered by `just ci` | PASS (same as SC-2) |

## Task Commits

Each task was committed atomically:

1. **Task 1: Create tauri.conf.json and capabilities/default.json, run npm install and automated checks** - `c579657` (feat)
2. **Task 2: Human confirms Tauri window opens with correct title** - human checkpoint, no separate commit

**Plan metadata:** (this commit — docs: complete 13-03 plan)

## Files Created/Modified

- `hp41-gui/src-tauri/tauri.conf.json` — Tauri v2 app config: identifier `ch.talent-factory.hp41`, title `HP-41 Calculator`, devUrl `http://localhost:5173`, frontendDist `../dist`, beforeDevCommand/beforeBuildCommand npm run hooks
- `hp41-gui/src-tauri/capabilities/default.json` — Tauri v2 default capability: `core:default` permission for window management and IPC baseline

## npm Package Versions (as installed)

| Package | Version |
|---------|---------|
| react | 19.2.6 |
| react-dom | 19.2.6 |
| @tauri-apps/api | 2.11.0 |
| @tauri-apps/cli | 2.11.1 |
| vite | 8.0.11 |
| @vitejs/plugin-react | 6.0.1 |
| tailwindcss | 4.3.0 |
| @tailwindcss/vite | 4.3.0 |
| typescript | 6.0.3 |
| @types/react | 19.2.14 |
| @types/react-dom | 19.2.3 |

## Decisions Made

- Bundle identifier `ch.talent-factory.hp41` chosen (D-02) — overrides scaffold default `com.tauri.dev` which causes macOS keychain and sandbox issues per Pitfall #4
- `capabilities/default.json` grants `core:default` only — this is the minimum viable set; hp41-specific IPC permissions (e.g., `fs:read`, custom invoke permissions) will be added in Phase 14 when IPC commands are registered
- Window dimensions 800×600 set as provisional defaults; final layout deferred to Phase 16

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Phase 14 Handoff

Phase 14 (GUI IPC) should begin from this state:

**Extension points:**
- `hp41-gui/src-tauri/src/lib.rs` — `AppState` type alias (currently `pub type AppState = ();`) is the registration point for the shared Mutex-wrapped CalcState
- `hp41-gui/src-tauri/src/lib.rs` — `invoke_handler(tauri::generate_handler![])` is currently empty; add command functions here
- `hp41-gui/src-tauri/capabilities/default.json` — add hp41-specific permissions here when IPC commands are registered (one entry per `tauri::command` function)

**Hard constraints for Phase 14:**
- Mutex lock MUST use `.unwrap_or_else(|e| e.into_inner())` — never `.unwrap()` or `.expect()` (poisoned lock recovery, required by zero-panic policy in hp41-core and consistency in hp41-gui)
- Every new Tauri command requires a corresponding permission entry in `capabilities/default.json`
- `hp41-core` must remain UI-agnostic — no Tauri imports in hp41-core

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 13 workspace skeleton is complete and fully verified
- All five success criteria (SC-1 through SC-5) are confirmed
- Phase 14 can begin immediately: IPC bridge between hp41-core CalcState and the Tauri frontend
- No blockers

---
*Phase: 13-workspace-skeleton*
*Completed: 2026-05-09*
