---
phase: 13-workspace-skeleton
verified: 2026-05-09T18:30:00Z
status: human_needed
score: 9/10 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run `just gui-dev` and observe the Tauri window"
    expected: "A desktop window opens with the title bar showing exactly 'HP-41 Calculator'; content area is blank/white; closing the window (X or Cmd+Q) exits cleanly with no crash."
    why_human: "Cannot launch a GUI window programmatically during automated verification. This was confirmed by the developer during Plan 13-03 Task 2 execution and documented as PASS in 13-03-SUMMARY.md. Re-confirm only if there is doubt about the tauri.conf.json window title having been changed since then."
---

# Phase 13: Workspace Skeleton — Verification Report

**Phase Goal:** Users can launch an empty Tauri v2 window via `just gui-dev`, the hp41-gui crate is a nested workspace member that does not affect `cargo build --workspace`, and `just ci` (the CLI pipeline) remains green without modification.
**Verified:** 2026-05-09T18:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `just gui-dev` opens a blank Tauri window titled "HP-41 Calculator" on macOS | ? HUMAN NEEDED | tauri.conf.json `windows[0].title = "HP-41 Calculator"` confirmed; App.tsx renders empty div; human checkpoint in 13-03-SUMMARY SC-1 row recorded as PASS by developer |
| 2 | `just ci` completes with exit code 0 (no CLI regression) | VERIFIED | `just ci` exited 0; 285 tests pass, 0 failures; hp41-core coverage 89.89% (above 80% gate) |
| 3 | `cargo build --workspace` builds hp41-core and hp41-cli only, NOT Tauri code | VERIFIED | `cargo build --workspace` output shows no Compiling lines for Tauri crates; Finished in 0.16s (already cached); root workspace `members = ["hp41-core", "hp41-cli"]` unchanged |
| 4 | `tauri` and `tauri-build` appear only in `hp41-gui/src-tauri/Cargo.toml`, not in root `[workspace.dependencies]` | VERIFIED | `grep -c "tauri" Cargo.toml` returns 0; both crates confirmed in `hp41-gui/src-tauri/Cargo.toml` only |
| 5 | CI matrix (Windows, macOS, Ubuntu) continues to pass the existing CLI jobs | VERIFIED | Same as Truth 2 — `just ci` covers the full CLI pipeline (`lint test coverage`); exit code 0 |
| 6 | `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` exits with code 0 | VERIFIED | Command completes: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.87s` |
| 7 | `hp41-gui/src-tauri/Cargo.toml` declares nested `[workspace]` with `resolver = "2"` | VERIFIED | File contains `[workspace]` on line 1 and `resolver = "2"` on line 2 |
| 8 | `tauri` and `tauri-build` crate versions are 2.11 and 2.6 respectively | VERIFIED | `tauri = { version = "2.11", features = ["macos-private-api"] }` and `tauri-build = { version = "2.6", features = [] }` confirmed in hp41-gui/src-tauri/Cargo.toml |
| 9 | Four `gui-*` Justfile recipes present; `ci` recipe is byte-for-byte unchanged | VERIFIED | `gui-install`, `gui-dev`, `gui-build`, `gui-check` each have exactly 1 match; `^ci:` line is `ci: lint test coverage` |
| 10 | `hp41-gui/node_modules` exists (npm install completed) | VERIFIED | Directory exists; `@tauri-apps/api`, `@tauri-apps/cli`, `@tailwindcss/vite` packages present |

**Score:** 9/10 truths verified (1 requires human re-confirmation)

---

## Required Artifacts

### Rust Crate (Plan 13-01)

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-gui/src-tauri/Cargo.toml` | Nested standalone workspace; tauri 2.11; tauri-build 2.6; hp41-core path dep | VERIFIED | Contains `[workspace]`, `resolver = "2"`, `[lib]` section with `name = "hp41_gui_lib"`, correct versions |
| `hp41-gui/src-tauri/build.rs` | `tauri_build::build()` entry point | VERIFIED | Exact content: `fn main() { tauri_build::build() }` |
| `hp41-gui/src-tauri/src/main.rs` | Delegates to `hp41_gui_lib::run()` | VERIFIED | Contains `hp41_gui_lib::run()` and Windows console suppression attribute |
| `hp41-gui/src-tauri/src/lib.rs` | Tauri Builder shell; `AppState` alias; `#![deny(clippy::unwrap_used)]` | VERIFIED | All required elements present: lint, `use tauri::Manager`, `pub type AppState = Mutex<hp41_core::CalcState>`, `pub fn run()` with empty invoke_handler |

### Frontend Scaffold (Plan 13-02)

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-gui/package.json` | react 19.2, @tauri-apps/api 2.11, vite 8.0, tailwindcss 4.3; `"type": "module"` | VERIFIED | All 11 packages declared at correct version ranges; `"type": "module"` and `"tauri": "tauri"` script present |
| `hp41-gui/vite.config.ts` | `strictPort: true`, port 5173, react() and tailwindcss() plugins | VERIFIED | All three requirements confirmed in file |
| `hp41-gui/tsconfig.json` | `moduleResolution: "bundler"`, `jsx: "react-jsx"`, `strict: true` | VERIFIED | All compilerOptions confirmed |
| `hp41-gui/index.html` | `<div id="root">`, `<title>HP-41 Calculator</title>`, module script | VERIFIED | All three elements present |
| `hp41-gui/src/main.tsx` | `ReactDOM.createRoot`, `React.StrictMode`, imports `./index.css` | VERIFIED | All three confirmed |
| `hp41-gui/src/App.tsx` | `className="app"`, `export default App` | VERIFIED | Present; renders empty `<div className="app">` |
| `hp41-gui/src/index.css` | `@import "tailwindcss"` (only line) | VERIFIED | Exactly one line: `@import "tailwindcss";` |

### Tauri Configuration (Plan 13-03)

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-gui/src-tauri/tauri.conf.json` | `identifier = "ch.talent-factory.hp41"`, `title = "HP-41 Calculator"`, `devUrl = "http://localhost:5173"`, `frontendDist = "../dist"` | VERIFIED | All values confirmed; `com.tauri.dev` scaffold default absent; `macOSPrivateApi: true` present (required for Cargo feature parity) |
| `hp41-gui/src-tauri/capabilities/default.json` | `"core:default"` permission | VERIFIED | Contains `"core:default"` in permissions array |
| `hp41-gui/node_modules` | npm dependencies installed | VERIFIED | Directory exists; @tauri-apps/api, @tauri-apps/cli, @tailwindcss/vite packages confirmed present |

### Justfile Recipes

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Justfile` (gui-* recipes) | `gui-install`, `gui-dev`, `gui-build`, `gui-check` appended; `ci` unchanged | VERIFIED | All four recipes present with correct bodies using `cd hp41-gui && npm ...` pattern; `gui-check` uses `--manifest-path` not `-p`; `ci: lint test coverage` byte-for-byte unchanged |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `hp41-gui/src-tauri/Cargo.toml` | Root workspace | Isolated — `[workspace]` declaration prevents membership | VERIFIED | `[workspace]` present; root `members = ["hp41-core", "hp41-cli"]` unchanged |
| `hp41-gui/src-tauri/src/lib.rs` | `hp41-core` | `hp41-core = { path = "../../hp41-core" }` path dependency | VERIFIED | Path dep confirmed in Cargo.toml; `hp41_core::CalcState` used in AppState alias and `Mutex::new()` call |
| `hp41-gui/src-tauri/tauri.conf.json` | `hp41-gui/vite.config.ts` | `devUrl: http://localhost:5173` must match vite `server.port: 5173` | VERIFIED | Both files have port 5173 |
| `hp41-gui/src-tauri/tauri.conf.json` | `hp41-gui/dist` | `frontendDist: ../dist` must match vite `build.outDir: dist` | VERIFIED | `frontendDist: "../dist"` in tauri.conf.json; `outDir: 'dist'` in vite.config.ts |
| `hp41-gui/index.html` | `hp41-gui/src/main.tsx` | `<script type="module" src="/src/main.tsx">` | VERIFIED | Present in index.html |
| `hp41-gui/src/main.tsx` | `hp41-gui/src/App.tsx` | `import App from './App'` | VERIFIED | Present in main.tsx |

---

## Data-Flow Trace (Level 4)

Not applicable. Phase 13 delivers infrastructure only — no dynamic data rendering. App.tsx renders a static empty `<div>`. Data flow from hp41-core through Tauri IPC to the frontend is deferred to Phase 14.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Nested workspace Rust type-check passes | `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` | `Finished dev profile in 0.87s` | PASS |
| Root workspace isolation — tauri not in root deps | `grep -c "tauri" Cargo.toml` | 0 | PASS |
| Root workspace members unchanged | `grep "members" Cargo.toml` | `members = ["hp41-core", "hp41-cli"]` | PASS |
| CLI CI pipeline unaffected | `just ci` | exit 0; 285 tests passed; 89.89% coverage | PASS |
| All four gui-* recipes in Justfile | `grep -c "gui-dev:" Justfile` (×4) | 1 each | PASS |
| ci recipe unchanged | `grep "^ci:" Justfile` | `ci: lint test coverage` | PASS |
| npm packages installed | `ls hp41-gui/node_modules/@tauri-apps` | api, cli, cli-darwin-arm64 present | PASS |
| Tauri bundle identifier correct | `grep "ch.talent-factory.hp41" hp41-gui/src-tauri/tauri.conf.json` | 1 match | PASS |
| Scaffold default bundle id absent | `grep "com.tauri.dev" hp41-gui/src-tauri/tauri.conf.json` | 0 matches | PASS |
| `just gui-dev` opens titled window | Visual observation (GUI) | Recorded PASS in 13-03-SUMMARY.md by developer | HUMAN |

---

## Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| WSPC-01 | 13-01, 13-02, 13-03 | User can build and launch `hp41-gui` from the Cargo workspace via `just gui-dev`; `just ci` (CLI pipeline) still passes without modification | SATISFIED (human confirmation for launch) | `just ci` exits 0; Justfile `gui-dev` recipe wired; tauri.conf.json and package.json structurally correct; node_modules installed; `cargo check` passes. Human confirmed window launch in 13-03 checkpoint. |
| WSPC-02 | 13-01, 13-03 | User can build both `hp41-cli` and `hp41-gui` in the same workspace without either binary's CI regressing | SATISFIED | `cargo build --workspace` builds only hp41-core/hp41-cli (Tauri isolated); `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` passes; `just ci` exits 0 |

No orphaned requirements — both WSPC-01 and WSPC-02 are the only Phase 13 requirements in REQUIREMENTS.md and both are claimed by the plans.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-gui/src-tauri/src/lib.rs` | 16 | `// Tauri commands registered here in Phase 14` comment in empty `generate_handler![]` | Info | Intentional Phase 13 skeleton — not a blocker; Phase 14 responsibility |

No blocker or warning anti-patterns found. The empty `invoke_handler` is correct for Phase 13 scope. The `AppState = Mutex<hp41_core::CalcState>` alias is substantive (not a placeholder `= ()`), contrary to an error in 13-03-SUMMARY.md which mistakenly wrote `pub type AppState = ()` — the actual code in lib.rs has the correct Mutex alias.

---

## Human Verification Required

### 1. Tauri Window Launch (SC-1)

**Test:** In a terminal, run `just gui-dev` from the repo root.
**Expected:** A desktop window opens. The title bar shows exactly "HP-41 Calculator". The content area is blank/white. Closing the window (X button or Cmd+Q on macOS) exits cleanly with no crash or stuck process.
**Why human:** GUI window launch cannot be verified programmatically in this environment. The structural evidence is complete (correct title in tauri.conf.json, correct App.tsx empty div, `cargo check` passing), and the developer recorded PASS in 13-03-SUMMARY.md SC-1 row. This item is included here because the phase goal explicitly requires it as a first-class success criterion (SC-1) and it is the primary user-visible outcome of the phase.

Note: If the developer already confirmed this during Plan 13-03 execution and no files relevant to window launch have been modified since (tauri.conf.json, lib.rs, main.rs, App.tsx), a re-run is not strictly required. The developer may accept the previously recorded human checkpoint.

---

## Gaps Summary

No gaps. All automated success criteria are VERIFIED. The only item pending is SC-1 (Tauri window visual launch), which was already confirmed by the developer during Plan 13-03 execution and recorded as PASS in the 13-03-SUMMARY.md. The `human_needed` status reflects that this verification pass cannot independently confirm a GUI window launch.

The phase goal — workspace isolation, `just ci` unaffected, `cargo build --workspace` isolation, tauri crates not in root workspace, and `just gui-dev` wiring — is structurally complete and consistent with a passing state.

---

_Verified: 2026-05-09T18:30:00Z_
_Verifier: Claude (gsd-verifier)_
