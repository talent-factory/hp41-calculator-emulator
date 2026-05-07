---
phase: 07-hardening
plan: 02
subsystem: infra
tags: [ci, github-actions, justfile, hyperfine, release-build, cross-platform]

# Dependency graph
requires:
  - phase: 07-hardening/07-01
    provides: "Zero-panic hp41-core with compile-time unwrap_used denial"
provides:
  - "CI test matrix runs just build-release on ubuntu-latest, macos-latest, windows-latest"
  - "CI test matrix runs just test on all three platforms"
  - "Justfile build-release recipe: cargo build --release"
  - "Justfile bench recipe: cargo bench -p hp41-core (advisory, non-gating)"
  - "Justfile bench-startup recipe: hyperfine --runs 10 ./target/release/hp41 (manual pre-release cold-start measurement)"
affects: [08-hardening-coverage, all plans referencing just build-release, future CI additions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "just build-release as the canonical release-build command in CI and docs — never bare cargo build --release"
    - "bench-startup as advisory-only recipe: hyperfine documents the cold-start measurement procedure without gating CI"

key-files:
  created: []
  modified:
    - .github/workflows/ci.yml
    - Justfile

key-decisions:
  - "Use just build-release (not cargo build --release directly) in CI to honor the project convention that just is the sole task runner in CI and docs"
  - "bench-startup uses ./target/release/hp41 (the actual [[bin]] name from hp41-cli/Cargo.toml) — not hp41-cli which does not exist"
  - "bench and bench-startup are advisory-only: they do not gate CI, preserving fast CI cycles while providing measurement tooling"

patterns-established:
  - "Pattern: All CI run steps use just recipes, never bare cargo commands — enforced by project CLAUDE.md constraint"
  - "Pattern: Justfile recipe order: default → build → build-release → test → lint → run → coverage → ci → fmt-check → fmt → install-hooks → bench → bench-startup"

requirements-completed: [QUAL-01, QUAL-05]

# Metrics
duration: 8min
completed: 2026-05-07
---

# Phase 7 Plan 02: CI Release Matrix + bench-startup Summary

**Cross-platform release binary verification added to CI matrix (ubuntu/macos/windows) via just build-release; hyperfine cold-start measurement documented as just bench-startup recipe**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-07T22:20:00Z
- **Completed:** 2026-05-07T22:28:00Z
- **Tasks:** 2 (plus 1 auto-fix)
- **Files modified:** 2

## Accomplishments

- Updated .github/workflows/ci.yml test matrix job to run `just build-release` before `just test` on all three platforms — cross-platform release binary compilation is now a CI gate on every push and PR
- Added `build-release` recipe to Justfile (`cargo build --release`) immediately after the existing `build` recipe
- Added `bench` recipe to Justfile (`cargo bench -p hp41-core`) for criterion benchmark runs (advisory, non-gating)
- Added `bench-startup` recipe to Justfile (`hyperfine --runs 10 ./target/release/hp41`) documenting the manual pre-release cold-start measurement procedure
- All coverage and lint CI jobs remain unchanged (coverage stays ubuntu-latest only per D-04)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add cargo build --release to CI test matrix (D-02)** — `bc0d767` (feat)
2. **Task 2: Add bench and bench-startup recipes to Justfile (D-09, D-12)** — `9a2c0e0` (feat)
3. **Auto-fix: Correct bench-startup binary path from hp41-cli to hp41** — `f632552` (fix)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `.github/workflows/ci.yml` — Added `just build-release` step (with name label) before `just test` in the test matrix job; coverage and lint jobs unchanged
- `Justfile` — Added three recipes: `build-release` (after build), `bench` (after install-hooks), `bench-startup` (after bench)

## Decisions Made

- Used `just build-release` in CI (not `cargo build --release` directly) to honor the project constraint that `just` is the sole task runner in CI and docs — consistent with CLAUDE.md
- Made `bench` and `bench-startup` advisory-only (no CI integration) to keep CI cycles fast while still documenting the measurement procedure
- `bench-startup` is documented as a manual pre-release step with a prerequisite comment requiring `just build-release` first

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected bench-startup binary path from hp41-cli to hp41**
- **Found during:** Task 2 post-verification (smoke-testing build-release recipe)
- **Issue:** The plan specified `hyperfine --runs 10 ./target/release/hp41-cli` but the actual binary produced by `cargo build --release` is `./target/release/hp41`. The `[[bin]]` name in `hp41-cli/Cargo.toml` is `hp41`, not `hp41-cli`. Using `hp41-cli` would cause `hyperfine` to fail with "command not found" on every invocation.
- **Fix:** Changed `bench-startup` recipe body from `./target/release/hp41-cli` to `./target/release/hp41`
- **Files modified:** Justfile
- **Verification:** `ls target/release/hp41` confirms binary exists; `grep "hyperfine" Justfile` shows correct path
- **Committed in:** f632552 (fix commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — incorrect binary path that would break bench-startup)
**Impact on plan:** Essential correctness fix. The bench-startup recipe would have been non-functional without it. No scope creep.

## Issues Encountered

None — both CI YAML modification and Justfile recipe additions were straightforward. The binary name mismatch was caught during the acceptance-criteria verification step.

## User Setup Required

None — no external service configuration required. CI changes take effect on next push to `main` or `develop`. `hyperfine` must be installed locally to use `just bench-startup`.

## Known Stubs

None — this plan is pure infrastructure (CI config + Justfile recipes). No data-rendering or UI code.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The CI workflow additions use no user-controlled inputs (matrix only selects from a fixed OS list; run steps call `just` recipes with no interpolated values). Threat T-07-02-01 (Tampering via CI YAML) mitigated: `just build-release` accepts no external inputs. Threat T-07-02-02 (DoS via missing binary) accepted: `bench-startup` is a manual recipe with documented prerequisites.

## Next Phase Readiness

- Phase 7 Plan 02 complete: CI now gates release builds on all three platforms
- `just build-release` available as the canonical release-build command for developers and CI
- Cold-start measurement procedure documented in Justfile; developers can run `just build-release && just bench-startup` before releases
- Ready for Phase 7 Plan 03 (coverage hardening or next wave)

## Self-Check: PASSED

- FOUND: .github/workflows/ci.yml (YAML valid, contains `just build-release`)
- FOUND: Justfile (contains build-release, bench, bench-startup recipes)
- FOUND: target/release/hp41 (binary produced by just build-release)
- FOUND commit bc0d767 (feat(07-02): add release build step to CI test matrix)
- FOUND commit 9a2c0e0 (feat(07-02): add build-release, bench, bench-startup recipes)
- FOUND commit f632552 (fix(07-02): correct bench-startup binary path)
- VERIFIED: python3 yaml.safe_load(ci.yml) prints "YAML valid"
- VERIFIED: just --list shows bench, bench-startup, build-release
- VERIFIED: just build-release exits 0

---
*Phase: 07-hardening*
*Completed: 2026-05-07*
