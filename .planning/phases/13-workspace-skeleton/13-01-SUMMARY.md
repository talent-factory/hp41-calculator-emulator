---
phase: 13-workspace-skeleton
plan: 01
subsystem: infra
tags: [tauri, cargo-workspace, rust, nested-workspace, justfile, hp41-gui]

# Dependency graph
requires:
  - phase: 12-synthetic-programming
    provides: hp41-core library with CalcState, all ops, and zero-panic guarantee
provides:
  - hp41-gui/src-tauri nested standalone Cargo workspace with tauri 2.11 and tauri-build 2.6
  - cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml passes
  - Root workspace (hp41-core, hp41-cli) is completely isolated from Tauri dependencies
  - Four gui-* Justfile recipes (gui-install, gui-dev, gui-build, gui-check)
  - AppState type alias (Mutex<CalcState>) in lib.rs for Phase 14 compile-time safety
affects:
  - 13-02 (tauri.conf.json + frontend scaffold depends on hp41-gui/src-tauri/ being present)
  - 13-03 (window launch verification depends on full scaffold from 13-01 + 13-02)
  - 14-ipc-layer (Tauri commands build on AppState alias and empty invoke_handler skeleton)

# Tech tracking
tech-stack:
  added:
    - tauri 2.11 (Rust crate, hp41-gui/src-tauri only)
    - tauri-build 2.6 (build-dependency, hp41-gui/src-tauri only)
  patterns:
    - Nested standalone Cargo workspace pattern: hp41-gui/src-tauri/Cargo.toml declares [workspace] with resolver="2"
    - Cross-workspace path dependency: hp41-core = { path = "../../hp41-core" } from src-tauri/
    - AppState type alias pattern: pub type AppState = Mutex<hp41_core::CalcState> for Phase 14 compile-time safety
    - Phase 14 Mutex lock constraint: use .unwrap_or_else(|e| e.into_inner()) NOT .unwrap() (Pitfall #7)

key-files:
  created:
    - hp41-gui/src-tauri/Cargo.toml
    - hp41-gui/src-tauri/build.rs
    - hp41-gui/src-tauri/src/main.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/tauri.conf.json
    - hp41-gui/src-tauri/icons/icon.png
    - hp41-gui/src-tauri/Cargo.lock
    - hp41-gui/src-tauri/.gitignore
  modified:
    - Justfile

key-decisions:
  - "Nested workspace pattern: hp41-gui/src-tauri declares [workspace] + resolver='2' — NOT a root workspace member"
  - "tauri and tauri-build declared ONLY in hp41-gui/src-tauri/Cargo.toml, never in root [workspace.dependencies] (Tauri issue #6122)"
  - "[lib] name = 'hp41_gui_lib' declared explicitly in Cargo.toml to match hp41_gui_lib::run() in main.rs"
  - "macOSPrivateApi enabled in both Cargo.toml features and tauri.conf.json app section (required to match)"
  - "use tauri::Manager import required in lib.rs for app.manage() call"
  - "AppState = Mutex<hp41_core::CalcState> defined now so Phase 14 gets compile error if State<> type is wrong"
  - "gen/ and target/ excluded via hp41-gui/src-tauri/.gitignore (root pattern does not match nested path)"

patterns-established:
  - "Nested workspace: [workspace] section in hp41-gui/src-tauri/Cargo.toml isolates Tauri from root resolver"
  - "Phase 14 contract: Mutex lock must use .unwrap_or_else(|e| e.into_inner()) not .unwrap() or .expect()"
  - "Phase 14 contract: AppState type alias is the single source of truth for State<> parameter type in commands"
  - "gui-check recipe uses --manifest-path not -p (nested workspace not visible to root workspace resolver)"

requirements-completed: [WSPC-01, WSPC-02]

# Metrics
duration: 35min
completed: 2026-05-09
---

# Phase 13 Plan 01: Workspace Skeleton Summary

**Nested standalone hp41-gui Cargo workspace with tauri 2.11/tauri-build 2.6, isolated from root workspace via [workspace] declaration, cargo check passes, and four gui-* Justfile recipes added**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-09T12:00:00Z
- **Completed:** 2026-05-09T12:35:00Z
- **Tasks:** 2 (plus 3 auto-fix deviations)
- **Files modified:** 9

## Accomplishments

- Created hp41-gui/src-tauri as a nested standalone Cargo workspace with resolver="2" — Tauri's dependency tree (wry, tokio, webkit2gtk) never enters root workspace resolver
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0; `cargo build --workspace` builds only hp41-core and hp41-cli
- AppState type alias (`Mutex<hp41_core::CalcState>`) established in lib.rs as Phase 14 compile-time safety guard (Pitfall #5 prevention)
- Four gui-* Justfile recipes added; existing `ci: lint test coverage` recipe byte-for-byte unchanged

## Task Commits

Each task was committed atomically:

1. **Task 1: Create hp41-gui Rust crate** - `c4881bc` (feat)
2. **Task 2: Add four gui-* Justfile recipes** - `23a8d17` (feat)
3. **Auto-fix: Cargo.lock and .gitignore** - `219fae9` (chore)

**Plan metadata:** committed with SUMMARY.md (docs: complete plan)

## Files Created/Modified

- `hp41-gui/src-tauri/Cargo.toml` — Nested workspace root with tauri 2.11, tauri-build 2.6, hp41-core path dep, [lib] section for hp41_gui_lib
- `hp41-gui/src-tauri/build.rs` — Tauri build codegen entry point (`tauri_build::build()`)
- `hp41-gui/src-tauri/src/main.rs` — Binary entry point delegating to `hp41_gui_lib::run()`
- `hp41-gui/src-tauri/src/lib.rs` — Tauri Builder shell with AppState alias and empty invoke_handler
- `hp41-gui/src-tauri/tauri.conf.json` — Bundle identifier ch.talent-factory.hp41, window title "HP-41 Calculator", macOSPrivateApi enabled
- `hp41-gui/src-tauri/icons/icon.png` — Placeholder RGBA 32x32 icon (Phase 16 will replace with HP-41 artwork)
- `hp41-gui/src-tauri/Cargo.lock` — Locked dependency tree for reproducible Tauri builds
- `hp41-gui/src-tauri/.gitignore` — Excludes target/ and gen/ from nested workspace
- `Justfile` — Added gui-install, gui-dev, gui-build, gui-check recipes after existing content

## Decisions Made

- **[lib] section required:** Cargo.toml must declare `[lib] name = "hp41_gui_lib"` explicitly because main.rs calls `hp41_gui_lib::run()`. Without the `[lib]` section, Cargo defaults the lib crate name to `hp41_gui` (package name with hyphens converted), which doesn't match the call site. This is the correct Tauri v2 pattern.
- **macOSPrivateApi symmetry:** The `macos-private-api` Cargo feature requires `"macOSPrivateApi": true` in `tauri.conf.json`'s `app` section. Mismatch causes build error during tauri_build::build(). Both must be set together.
- **tauri::Manager import:** `app.manage()` in the setup closure requires `use tauri::Manager;` in scope. The trait is not automatically in scope from `use tauri::Builder`.
- **tauri.conf.json required for cargo check:** `tauri_build::build()` in build.rs reads `tauri.conf.json` at compile time. The file must exist even for `cargo check`. Since Plan 13-02 was planned to create this file, it was created here as a Rule 3 (blocking) deviation.
- **gen/ gitignore:** `tauri_build::build()` generates `gen/schemas/` at cargo check time. These are build artifacts that should not be committed. Root `.gitignore`'s `src-tauri/target/` pattern uses a path separator, which anchors it to the root and doesn't match `hp41-gui/src-tauri/target/`. A nested `.gitignore` is required.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] tauri.conf.json required by tauri_build::build() for cargo check to pass**
- **Found during:** Task 1 verification (`cargo check --manifest-path`)
- **Issue:** `tauri_build::build()` in build.rs reads `tauri.conf.json` at compile time. Without it, build script exits with "unable to read Tauri config file". The plan assigned tauri.conf.json to Plan 13-02.
- **Fix:** Created `hp41-gui/src-tauri/tauri.conf.json` with the exact content from PATTERNS.md (bundle identifier ch.talent-factory.hp41, window title "HP-41 Calculator")
- **Files modified:** hp41-gui/src-tauri/tauri.conf.json
- **Verification:** cargo check passes after creation
- **Committed in:** c4881bc (Task 1 commit)

**2. [Rule 1 - Bug] macOSPrivateApi feature mismatch between Cargo.toml and tauri.conf.json**
- **Found during:** Task 1 verification (second cargo check attempt)
- **Issue:** `tauri = { version = "2.11", features = ["macos-private-api"] }` in Cargo.toml requires `"macOSPrivateApi": true` in tauri.conf.json's `app` section. Without it, tauri_build reports "The tauri dependency features does not match the allowlist".
- **Fix:** Added `"macOSPrivateApi": true` to the `app` object in tauri.conf.json
- **Files modified:** hp41-gui/src-tauri/tauri.conf.json
- **Verification:** cargo check passes
- **Committed in:** c4881bc (Task 1 commit)

**3. [Rule 1 - Bug] Missing tauri::Manager import causes app.manage() to fail**
- **Found during:** Task 1 verification (third cargo check attempt)
- **Issue:** `app.manage(Mutex::new(...))` in lib.rs setup closure requires `use tauri::Manager;`. The trait is not in scope by default.
- **Fix:** Added `use tauri::Manager;` import to lib.rs
- **Files modified:** hp41-gui/src-tauri/src/lib.rs
- **Verification:** cargo check passes
- **Committed in:** c4881bc (Task 1 commit)

**4. [Rule 1 - Bug] [lib] section missing — hp41_gui_lib::run() unresolved in main.rs**
- **Found during:** Task 1 verification (fourth cargo check attempt)
- **Issue:** Without an explicit `[lib]` section, the crate's lib name defaults to the package name with hyphens replaced by underscores: `hp41_gui`. main.rs calls `hp41_gui_lib::run()` which doesn't match.
- **Fix:** Added `[lib] name = "hp41_gui_lib" path = "src/lib.rs"` to Cargo.toml
- **Files modified:** hp41-gui/src-tauri/Cargo.toml
- **Verification:** cargo check passes; lib is accessible as hp41_gui_lib
- **Committed in:** c4881bc (Task 1 commit)

**5. [Rule 3 - Blocking] Build-generated files untracked after cargo check**
- **Found during:** Post-commit untracked file check
- **Issue:** `cargo check` generates `hp41-gui/src-tauri/gen/` (schema files) and `hp41-gui/src-tauri/target/` (build artifacts). Root .gitignore pattern `src-tauri/target/` anchors to root path and doesn't match the nested `hp41-gui/src-tauri/` path.
- **Fix:** Created `hp41-gui/src-tauri/.gitignore` with `target/` and `gen/` entries. Also committed `Cargo.lock` for reproducible binary builds.
- **Files modified:** hp41-gui/src-tauri/.gitignore, hp41-gui/src-tauri/Cargo.lock
- **Verification:** `git status --short | grep '^??'` returns nothing
- **Committed in:** 219fae9

---

**Total deviations:** 5 auto-fixed (1 Rule 3 blocking, 3 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All auto-fixes were correctness requirements discovered during verification. tauri.conf.json creation was originally Plan 13-02 scope; moving it to Plan 13-01 enables cargo check to pass in this plan. No scope creep — the content is exactly what PATTERNS.md specified.

## Issues Encountered

The plan's verification criterion (`cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml exits 0`) required resolving four cascading compiler errors, each fixed via deviation rules. The root cause was that `tauri_build::build()` and `tauri::generate_context!()` have compile-time requirements (config file, icon file, feature flag parity) that must be satisfied before `cargo check` can succeed. All issues are Tauri-specific compile-time requirements, not design problems.

## Phase 14 Constraints (Documented for Next Plan)

Per Pitfall #7 in PITFALLS.md, Phase 14 command handlers MUST use:
```rust
let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
```
NOT `.unwrap()` or `.expect("poisoned")`. The `#![deny(clippy::unwrap_used)]` lint in lib.rs enforces this at compile time.

The `AppState` type alias is the single source of truth:
```rust
pub type AppState = Mutex<hp41_core::CalcState>;
```
Use `State<'_, AppState>` in all command signatures — not `State<'_, Mutex<CalcState>>` directly — to get a compile error if the managed type doesn't match.

## Exact Cargo.toml Content (for Phase 14 reference)

```toml
[workspace]
resolver = "2"

[package]
name = "hp41-gui"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"

[[bin]]
name = "hp41-gui"
path = "src/main.rs"

[lib]
name = "hp41_gui_lib"
path = "src/lib.rs"

[dependencies]
tauri = { version = "2.11", features = ["macos-private-api"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
hp41-core = { path = "../../hp41-core" }

[build-dependencies]
tauri-build = { version = "2.6", features = [] }
```

## Workspace Isolation Verification

```
$ cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.88s

$ cargo build --workspace
Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.35s
(only hp41-core and hp41-cli compiled — Tauri not touched)

$ grep "members" Cargo.toml
members = ["hp41-core", "hp41-cli"]

$ grep -r "tauri" Cargo.toml
(empty — tauri not in root workspace)
```

## Next Phase Readiness

- Plan 13-02 can build the frontend scaffold (package.json, vite.config.ts, tsconfig.json, index.html, src/ files) — hp41-gui/src-tauri/ is in place
- Plan 13-03 (window launch verification) can proceed after 13-02 completes npm install
- Phase 14 (IPC layer) has all prerequisites: AppState type alias in lib.rs, empty invoke_handler, hp41-core path dependency wired

No blockers for Plan 13-02.

## User Setup Required

None — no external service configuration required. `just gui-install` (npm install) is handled in Plan 13-02/13-03.

---
*Phase: 13-workspace-skeleton*
*Completed: 2026-05-09*
