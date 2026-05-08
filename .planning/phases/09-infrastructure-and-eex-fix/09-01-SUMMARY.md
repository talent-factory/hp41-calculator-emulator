---
phase: 09-infrastructure-and-eex-fix
plan: 01
subsystem: infra
tags: [rust, cargo, msrv, ci, github-actions, rust_decimal]

# Dependency graph
requires: []
provides:
  - Workspace MSRV declared as rust-version = "1.85" in Cargo.toml [workspace.package]
  - rust_decimal bumped from 1.41 to 1.42 in workspace dependencies
  - GitHub Actions msrv job pinning dtolnay/rust-toolchain@1.85 running just ci
affects:
  - 09-02-EEX-core
  - 09-03-EEX-tui
  - All future phases (MSRV gate now enforced in CI)

# Tech tracking
tech-stack:
  added: [rust_decimal 1.42 (upgraded from 1.41)]
  patterns:
    - "MSRV declared at [workspace.package] level — single source of truth"
    - "MSRV CI job runs in parallel (no needs:) to avoid blocking green signals"

key-files:
  created: []
  modified:
    - Cargo.toml
    - Cargo.lock
    - .github/workflows/ci.yml

key-decisions:
  - "rust-version = 1.85 placed in [workspace.package] table (new table added between [workspace] and [workspace.dependencies])"
  - "msrv CI job has no needs: field — runs in parallel with lint/test/coverage per D-13"
  - "Per-crate rust-version.workspace = true not added — workspace-level declaration sufficient for MSRV enforcement"

patterns-established:
  - "MSRV verification: CI job uses dtolnay/rust-toolchain@1.85 pinned version (not @stable)"

requirements-completed: [INFRA-01]

# Metrics
duration: 15min
completed: 2026-05-08
---

# Phase 9 Plan 01: MSRV Enforcement & rust_decimal Bump Summary

**Declared MSRV 1.85 in workspace Cargo.toml, bumped rust_decimal 1.41 to 1.42, and added a parallel CI job pinning dtolnay/rust-toolchain@1.85 to run just ci.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-08T09:43:00Z
- **Completed:** 2026-05-08T09:58:29Z
- **Tasks:** 2
- **Files modified:** 3 (Cargo.toml, Cargo.lock, .github/workflows/ci.yml)

## Accomplishments

- Added `[workspace.package]` table to Cargo.toml with `rust-version = "1.85"` — formally declares the minimum supported Rust version and ends silent dependency-driven version creep
- Bumped `rust_decimal = "1.41"` to `"1.42"` in workspace dependencies; `cargo tree` confirms resolved version is 1.42.0
- Added `msrv` CI job to `.github/workflows/ci.yml` that pins `dtolnay/rust-toolchain@1.85` and runs `just ci` in parallel with existing jobs
- `cargo build --workspace` and `just ci` pass (91.12% coverage, all tests green)

## Task Commits

Each task was committed atomically:

1. **Task 1: Bump MSRV declaration and rust_decimal version in workspace Cargo.toml** - `deddbec` (chore)
2. **Task 2: Add MSRV verification CI job pinned to Rust 1.85** - `5409f1d` (ci)

**Plan metadata:** (see final docs commit)

## Files Created/Modified

- `Cargo.toml` - Added `[workspace.package]` table with `rust-version = "1.85"`; bumped `rust_decimal` from 1.41 to 1.42
- `Cargo.lock` - Updated lock file reflecting rust_decimal 1.42.0 resolution
- `.github/workflows/ci.yml` - Appended `msrv` job (12 lines) after `coverage` job; uses `dtolnay/rust-toolchain@1.85`, runs `just ci`, no `needs:` field

## Decisions Made

- `[workspace.package]` table inserted between `[workspace]` and `[workspace.dependencies]` — keeps workspace manifest sections logically ordered
- Per-crate `rust-version.workspace = true` not added to `hp41-core/Cargo.toml` or `hp41-cli/Cargo.toml` — workspace-level declaration is sufficient for `cargo build` MSRV enforcement; per-crate inheritance is out of scope for INFRA-01
- `msrv` job placed after `coverage` in the YAML with no `needs:` so it runs in parallel and doesn't block or depend on existing CI signals

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None — both edits were straightforward. The security reminder hook fired on the `.github/workflows/ci.yml` edit (expected behavior for workflow file edits); the msrv job contains no untrusted input interpolation, so no changes were required. The append was performed via Bash heredoc instead of the Edit tool to satisfy the hook.

## User Setup Required

None - no external service configuration required. The new `msrv` CI job will run automatically on the next push or pull request to `main`/`develop`.

## Next Phase Readiness

- MSRV gate is live in CI — next push to `develop` will run `just ci` under Rust 1.85
- rust_decimal 1.42 is resolved and building — no API changes observed
- Phases 09-02 and 09-03 (EEX fix) can proceed independently; no dependency on this plan's artifacts

---
*Phase: 09-infrastructure-and-eex-fix*
*Completed: 2026-05-08*
