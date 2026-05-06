---
phase: 01-foundation
plan: 01
subsystem: infra
tags: [rust, cargo, workspace, justfile, cargo-llvm-cov, hp41-core, hp41-cli]

# Dependency graph
requires: []
provides:
  - Cargo workspace root with resolver=2 and shared workspace.dependencies
  - hp41-core library crate with zero UI/CLI dependencies
  - hp41-cli binary crate depending on hp41-core via path dep
  - Module skeleton in hp41-core/src/lib.rs (error, num, state, stack, ops)
  - Justfile with all six required recipes (build, test, lint, run, coverage, ci)
  - cargo-llvm-cov 0.8.5 and llvm-tools-preview installed
affects: [02-core-model, 03-ops, 04-tui-input, all subsequent phases]

# Tech tracking
tech-stack:
  added:
    - rust_decimal 1.41 (workspace dep — BCD-safe decimal arithmetic)
    - thiserror 2.0 (workspace dep — typed error derives)
    - proptest 1.11 (hp41-core dev-dep — property-based testing)
    - insta 1.47 yaml (hp41-core dev-dep — snapshot testing)
    - cargo-llvm-cov 0.8.5 (installed tool — coverage gate)
    - just 1.49.0 (already installed — sole task runner)
  patterns:
    - Cargo workspace with resolver=2 enforcing crate boundaries
    - Justfile as single task runner entrypoint (never call cargo directly)
    - ci: lint test coverage recipe chain as quality gate

key-files:
  created:
    - Cargo.toml
    - Cargo.lock
    - Justfile
    - hp41-core/Cargo.toml
    - hp41-core/src/lib.rs
    - hp41-core/src/error.rs
    - hp41-core/src/num.rs
    - hp41-core/src/state.rs
    - hp41-core/src/stack.rs
    - hp41-core/src/ops/mod.rs
    - hp41-cli/Cargo.toml
    - hp41-cli/src/main.rs
  modified: []

key-decisions:
  - "Workspace resolver=2 with edition=2021 for MSRV 1.78+ compatibility (not resolver=3/edition=2024)"
  - "rust_decimal added without maths feature in Phase 1; Phase 2 will add features=[maths] for trig/log"
  - "cargo-llvm-cov coverage gate targets hp41-core only (not --workspace) per plan spec"

patterns-established:
  - "Pattern: All build/test/lint/run targets via just recipes — never bare cargo in CI/docs"
  - "Pattern: hp41-core has zero UI deps, enforced by workspace Cargo manifest at compile time"
  - "Pattern: coverage recipe uses --fail-under-lines 80 -p hp41-core as quality gate"

requirements-completed: [CORE-01, CORE-02]

# Metrics
duration: 12min
completed: 2026-05-06
---

# Phase 1 Plan 01: Workspace Scaffold Summary

**Cargo workspace with hp41-core (zero-UI lib) and hp41-cli (bin), Justfile covering build/test/lint/run/coverage/ci, and cargo-llvm-cov 0.8.5 installed**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-06T15:00:00Z
- **Completed:** 2026-05-06T15:12:00Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments
- Cargo workspace with resolver=2 compiles from scratch (`cargo check --workspace` exits 0)
- hp41-core crate has zero ratatui/crossterm/clap/tokio transitive dependencies (verified via `cargo tree`)
- Justfile delivers all six required recipes; `just build` and `just --list` both succeed
- cargo-llvm-cov 0.8.5 and llvm-tools-preview installed; `cargo llvm-cov --version` exits 0

## Task Commits

Each task was committed atomically:

1. **Task 1: Install cargo-llvm-cov and scaffold Cargo workspace** - `e795f0f` (feat)
2. **Task 2: Write Justfile with all six required recipes** - `5be3e26` (feat)

## Files Created/Modified
- `Cargo.toml` - Workspace manifest with resolver=2, rust_decimal 1.41, thiserror 2.0 workspace deps
- `Cargo.lock` - Generated lockfile after first cargo check
- `hp41-core/Cargo.toml` - Library crate manifest with no UI/CLI deps
- `hp41-core/src/lib.rs` - Public API skeleton declaring error, num, state, stack, ops modules
- `hp41-core/src/error.rs` - Placeholder (Plan 02 fills)
- `hp41-core/src/num.rs` - Placeholder (Plan 02 fills)
- `hp41-core/src/state.rs` - Placeholder (Plan 02 fills)
- `hp41-core/src/stack.rs` - Placeholder (Plan 02 fills)
- `hp41-core/src/ops/mod.rs` - Placeholder (Plan 03 fills)
- `hp41-cli/Cargo.toml` - Binary crate manifest, hp41-core path dep
- `hp41-cli/src/main.rs` - Thin placeholder main (Phase 4 TUI not yet implemented)
- `Justfile` - Six recipes: build, test, lint, run, coverage, ci

## Decisions Made
- Used `resolver = "2"` and `edition = "2021"` for MSRV 1.78+ compatibility rather than resolver=3/edition=2024, per research recommendation (A5 from RESEARCH.md)
- Added `rust_decimal` without `features = ["maths"]` in Phase 1; Phase 2 will add the maths feature when trig/log operations are implemented
- Coverage recipe targets `-p hp41-core` only (not `--workspace`) so the gate only applies to the core library, not the CLI placeholder

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Both cargo-llvm-cov and llvm-tools-preview needed installation from scratch; both installed cleanly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Cargo workspace compiles clean; all module stubs in hp41-core/src/ ready for Plan 02 to fill in
- `cargo llvm-cov --version` confirms coverage tool is available for Plan 04 and later quality gates
- Plan 02 (Core Model) can now add CalcState, Stack, HpNum, and HpError implementations to the placeholder module stubs

---
*Phase: 01-foundation*
*Completed: 2026-05-06*
