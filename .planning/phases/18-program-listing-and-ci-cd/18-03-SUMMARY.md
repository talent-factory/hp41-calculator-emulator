---
phase: 18-program-listing-and-ci-cd
plan: "03"
subsystem: ci-cd
tags: [github-actions, ci, gui, tauri, justfile]
dependency_graph:
  requires: []
  provides: [ci-gui.yml, gui-ci-recipe]
  affects: [Justfile, .github/workflows/ci-gui.yml]
tech_stack:
  added: []
  patterns: [github-actions-path-filter, just-as-ci-runner, tauri-webkit-linux-deps]
key_files:
  created:
    - .github/workflows/ci-gui.yml
  modified:
    - Justfile
decisions:
  - "gui-ci Justfile recipe wraps npm install + npx tsc --noEmit + cargo build --release so ci-gui.yml never calls bare cargo (CLAUDE.md compliance)"
  - "Linux WebKit apt-get step stays in YAML (not in just recipe) because it requires if: matrix.os == 'ubuntu-latest' conditional guard"
  - "Swatinem/rust-cache@v2 uses workspaces key pointing to hp41-gui/src-tauri target directory to avoid caching the CLI workspace artifacts"
metrics:
  duration: "101 seconds"
  completed: "2026-05-10T15:51:43Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 1
---

# Phase 18 Plan 03: GUI CI Workflow Summary

**One-liner:** Cross-platform GitHub Actions workflow for hp41-gui using path filters, 3-OS matrix, and `just gui-ci` task runner (SC-4 + SC-5 compliance).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add gui-ci recipe to Justfile | 2d19e5c | Justfile |
| 2 | Create .github/workflows/ci-gui.yml | 52aa881 | .github/workflows/ci-gui.yml |

## What Was Built

**Task 1 — Justfile `gui-ci` recipe**

Added after `gui-check` in the Justfile:
```just
# gui-ci: CI gate — TypeScript type-check and Rust release build (called from ci-gui.yml)
gui-ci:
    cd hp41-gui && npm install
    cd hp41-gui && npx tsc --noEmit
    cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
```

This recipe is the single CI entrypoint for all non-OS-conditional GUI build steps. The Linux WebKit `apt-get` step cannot be moved here because it requires the `if: matrix.os == 'ubuntu-latest'` YAML conditional guard.

**Task 2 — `.github/workflows/ci-gui.yml`**

New workflow file with:
- Path filter on `hp41-gui/**` and `hp41-core/**` (both `push` and `pull_request` triggers on `main`/`develop`)
- 3-OS matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`; `fail-fast: false`
- `Swatinem/rust-cache@v2` with `workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target`
- `actions/setup-node@v4` with `node-version: 'lts/*'`
- `taiki-e/install-action@v2` to install `just` (matches existing `ci.yml` pattern)
- Conditional Linux step: `apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
- Final step: `just gui-ci` (CLAUDE.md compliance — no bare `cargo` in CI)

`ci.yml` is byte-for-byte identical to its pre-plan state (verified by empty `git diff`).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. This plan creates CI infrastructure only; no UI rendering code involved.

## Threat Flags

None. This plan creates CI configuration files only. No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The threat model in the plan (T-18-W1-CI-01, T-18-W1-CI-02) correctly categorizes path filter bypasses and third-party action trust as accepted risks for a build-only CI job with no secrets.

## Self-Check: PASSED

- [x] `.github/workflows/ci-gui.yml` exists — FOUND
- [x] `Justfile` modified — FOUND  
- [x] Commit 2d19e5c exists — FOUND
- [x] Commit 52aa881 exists — FOUND
- [x] `ci.yml` unchanged — verified by empty `git diff`
- [x] `gui-ci` appears 2 times in Justfile (comment + recipe name) — VERIFIED
- [x] `npx tsc --noEmit` in Justfile — VERIFIED
- [x] `cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml` in Justfile — VERIFIED
- [x] All 10 ci-gui.yml acceptance criteria pass — VERIFIED
