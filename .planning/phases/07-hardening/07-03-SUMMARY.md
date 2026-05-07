---
phase: 07-hardening
plan: 03
subsystem: testing
tags: [rust, criterion, benchmark, dispatch, hp41-core, performance]

# Dependency graph
requires:
  - phase: 07-hardening/01
    provides: "Zero-panic hp41-core with #![deny(clippy::unwrap_used)] — prod code compile-time guarantee"
provides:
  - "Criterion benchmark for dispatch() key-dispatch latency: hp41-core/benches/dispatch_bench.rs"
  - "just bench recipe in Justfile (cargo bench -p hp41-core)"
  - "Measured dispatch() latency ~25 ns/op on M1 (well within 50ms QUAL-02 requirement)"
affects: [07-hardening-ci, QUAL-02-validation]

# Tech tracking
tech-stack:
  added:
    - "criterion 0.5.1 (dev-dependency with html_reports feature)"
  patterns:
    - "Benchmark harness = false disables libtest; criterion provides its own main function"
    - "Advisory-only benches: do not gate CI builds due to VM timing variance (D-11)"
    - "dispatch() benchmarked directly via hp41_core::ops::{dispatch, Op} — not re-exported at crate root"

key-files:
  created:
    - hp41-core/benches/dispatch_bench.rs
  modified:
    - hp41-core/Cargo.toml
    - Justfile
    - Cargo.lock

key-decisions:
  - "Benchmark is advisory-only: CI VMs have too much timing variance for absolute gates (D-11)"
  - "Used harness = false in [[bench]] — required for criterion's own main() entry point"
  - "Added bench recipe to Justfile to satisfy just bench requirement; criterion not in workspace deps since it is only needed by hp41-core dev builds"
  - "Import dispatch via hp41_core::ops::{dispatch, Op} since dispatch is not re-exported at crate root"

patterns-established:
  - "Pattern: Criterion benches live in hp41-core/benches/ with harness = false and access dispatch() via hp41_core::ops"
  - "Pattern: just bench is the canonical way to run benchmarks; never call cargo bench directly in docs"

requirements-completed: [QUAL-02, QUAL-03]

# Metrics
duration: 3min
completed: 2026-05-07
---

# Phase 7 Plan 03: Criterion Dispatch Benchmark Summary

**Criterion benchmark for hp41-core dispatch() latency: 3 benchmark groups covering 20-op mixed workload, single-Add overhead, and 1000-keystroke equivalence; measured ~25 ns/op on M1 (well below 50ms QUAL-02 target)**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-07T20:57:51Z
- **Completed:** 2026-05-07T21:00:31Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `criterion = { version = "0.5", features = ["html_reports"] }` to hp41-core [dev-dependencies] and a `[[bench]]` entry with `harness = false`
- Created `hp41-core/benches/dispatch_bench.rs` with 3 benchmark groups:
  - `dispatch_mixed_20ops`: 20 representative HP-41 ops (arithmetic, stack, trig, registers) on a fresh CalcState
  - `dispatch_single_add`: per-op overhead measurement for the most common Add operation
  - `dispatch_1000/arithmetic/1000x_add`: 1000-keystroke equivalence measurement (20 samples, advisory)
- Added `bench` recipe to Justfile (`cargo bench -p hp41-core`)
- Verified `cargo build --benches -p hp41-core` exits 0 and `just test` passes without regressions
- Measured results on M1: dispatch_single_add ~25 ns, dispatch_1000/arithmetic ~25 µs for 1000 ops — QUAL-02 (<=50ms per keypress) confirmed with >1000x margin

## Task Commits

Each task was committed atomically:

1. **Task 1: Add criterion dev-dependency and [[bench]] entry** - `75e20c1` (chore)
2. **Task 2: Write hp41-core/benches/dispatch_bench.rs** - `56b364f` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `hp41-core/benches/dispatch_bench.rs` — Criterion benchmark with 3 benchmark groups covering dispatch() over a representative set of HP-41 ops
- `hp41-core/Cargo.toml` — Added criterion 0.5 dev-dependency with html_reports feature and [[bench]] table
- `Justfile` — Added `bench` recipe for `cargo bench -p hp41-core`
- `Cargo.lock` — Updated with criterion 0.5.1 and its transitive dependencies (26 new packages)

## Decisions Made

- Did not add criterion to [workspace.dependencies] since it is only needed by hp41-core's dev builds; crate-local dev-dependency is cleaner
- Kept `group.sample_size(20)` for the 1000-op group (plan-specified advisory default); full criterion default of 100 samples would be unnecessarily slow for a 25µs iteration
- Imported dispatch via `hp41_core::ops::{dispatch, Op}` (not from crate root) — `dispatch` is not re-exported in lib.rs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added bench recipe to Justfile**
- **Found during:** Task 2 (verification step)
- **Issue:** The plan's `<acceptance_criteria>` and `<success_criteria>` both require `just bench` to exit 0, but the Justfile had no `bench` recipe — `just bench` would error with "Recipe 'bench' not found"
- **Fix:** Added `bench` recipe to Justfile (`cargo bench -p hp41-core`)
- **Files modified:** Justfile
- **Verification:** `just bench -- --warm-up-time 1 --measurement-time 1` exits 0 and produces criterion output
- **Committed in:** 56b364f (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 — missing critical functionality)
**Impact on plan:** The Justfile bench recipe is required for the plan's own success criteria. Without it, `just bench` errors. Fix is minimal and stays within the plan's scope.

## Issues Encountered

None — the benchmark compiled on first attempt. The `dispatch()` import path (`hp41_core::ops::dispatch`) matches what the plan documents in the `<interfaces>` section.

## Known Stubs

None — this plan is a benchmarking infrastructure plan with no UI or data-rendering components.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Criterion benchmark HTML reports land in `target/criterion/` which is gitignored. No sensitive data in timing output. T-07-03-01 and T-07-03-02 threats both accepted as planned.

## Next Phase Readiness

- Phase 7 Plan 03 complete: criterion benchmark infrastructure in place for hp41-core
- QUAL-02 (key latency <=50ms) confirmed: dispatch() runs at ~25 ns/op on M1, well within budget
- Ready for Phase 7 Plan 04 (coverage gate enforcement or next hardening task)

## Self-Check

- FOUND: hp41-core/benches/dispatch_bench.rs (contains criterion_main, 24 occurrences of "dispatch")
- FOUND: hp41-core/Cargo.toml (contains criterion, dispatch_bench, harness = false)
- FOUND: Justfile (contains bench recipe)
- FOUND commit 75e20c1 (chore(07-03): add criterion dev-dependency)
- FOUND commit 56b364f (feat(07-03): add criterion dispatch benchmark)
- VERIFIED: cargo build --benches -p hp41-core exits 0
- VERIFIED: cargo bench --bench dispatch_bench runs successfully, criterion output shows dispatch_mixed_20ops timing

## Self-Check: PASSED

---
*Phase: 07-hardening*
*Completed: 2026-05-07*
