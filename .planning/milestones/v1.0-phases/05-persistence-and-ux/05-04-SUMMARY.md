---
phase: 05-persistence-and-ux
plan: 04
subsystem: ux-data
tags: [rust, static-data, help-overlay, sample-programs, OnceLock]

# Dependency graph
requires:
  - phase: 05-persistence-and-ux
    plan: 03
    provides: help_data.rs stub + programs.rs stub (Plan 03 created empty stubs)
  - phase: 03-programming-engine
    provides: Op enum variants (Lbl, Gto, Test, StoArith, etc.) used in program construction

provides:
  - hp41-cli/src/help_data.rs HELP_DATA static array — 75 entries, 12 category groups covering all keyboard-accessible HP-41 operations
  - hp41-cli/src/programs.rs SampleProgram struct + sample_programs() accessor + 10 programs via OnceLock

affects:
  - 05-05 (help overlay ui.rs will read HELP_DATA via crate::help_data::HELP_DATA)
  - 05-06 (program library overlay ui.rs will read sample_programs() via crate::programs::sample_programs)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - OnceLock<Vec<SampleProgram>> — lazy initialization for heap-allocated Op::Lbl(String) program data; thread-safe, initialized at first access (D-24 / RESEARCH Pitfall 2)
    - Category header pattern — ("", "", "=== Category ===") tuples in HELP_DATA for visual grouping in ratatui Table widget
    - Static &[(&str, &str, &str)] const for help data — compile-time verified, zero runtime allocation

key-files:
  created: []
  modified:
    - hp41-cli/src/help_data.rs (stub → full implementation: 75-entry HELP_DATA const, 3 inline tests)
    - hp41-cli/src/programs.rs (stub → full implementation: SampleProgram struct, sample_programs() via OnceLock, 10 programs, 5 inline tests)

key-decisions:
  - "HELP_DATA covers 12 categories (Stack, Arithmetic, Trig, Math, Registers, ALPHA, Programming, Display, Persistence, USER, Help, Quit) — plan specified 10 but Help and Quit are essential for discoverability; non-breaking addition"
  - "All 10 sample programs start with Op::Lbl(\"A\") — uniform run_program(state, \"A\") invocation for all programs loaded from library (D-23 specifics)"
  - "OnceLock chosen over lazy_static — OnceLock is std library (Rust 1.70+), no extra dependency needed (D-24)"

requirements-completed: [UX-01, UX-03]

# Metrics
duration: ~5min
completed: 2026-05-07
---

# Phase 5 Plan 04: Static Data Modules — help_data.rs + programs.rs Summary

**HELP_DATA static array with 75 entries across 12 categories and 10 bundled HP-41 sample programs via OnceLock, replacing Plan 03 stubs**

## Performance

- **Duration:** ~5 min
- **Completed:** 2026-05-07
- **Tasks:** 2
- **Files created:** 0, **Files modified:** 2

## Accomplishments

- `help_data.rs` filled from stub: `pub const HELP_DATA: &[(&str, &str, &str)]` with 75 entries across 12 category groups covering all keyboard-accessible HP-41 operations. Category headers use `("", "", "=== Category ===")` pattern for visual grouping in the ratatui Table widget (D-17/D-18).
- 3 inline tests in `help_data.rs`: minimum 50 entries, all 10 required categories present, no empty key/op fields in data rows.
- `programs.rs` filled from stub: `SampleProgram { name, description, ops }` struct + `PROGRAMS_CACHE: OnceLock<Vec<SampleProgram>>` + `sample_programs()` accessor.
- 10 HP-41 classic programs: Fibonacci, Factorial, Prime Test, Quadratic Solver, GCD (Euclidean), Newton Root, Mean+StdDev, Deg-to-Rad converter, Stack Stats, Countdown Timer.
- All programs begin with `Op::Lbl("A".to_string())` — uniform `run_program(state, "A")` invocation.
- Fibonacci test: `n=6` runs to completion without panic or error via `run_program()`.
- 5 inline tests in `programs.rs`: count ≥10, all non-empty, all start with LBL A, Fibonacci runs, names unique.
- Full hp41-cli test suite: **31 tests pass, zero regressions**.

## Task Commits

1. **Task 1: Create help_data.rs with HELP_DATA static array** - `93349f7` (feat)
2. **Task 2: Create programs.rs with SampleProgram + 10 programs via OnceLock** - `1487b41` (feat)

## Files Created/Modified

- `hp41-cli/src/help_data.rs` — stub replaced with full implementation: 75-entry HELP_DATA const across 12 categories, 3 inline tests
- `hp41-cli/src/programs.rs` — stub replaced with full implementation: SampleProgram struct, OnceLock lazy init, 10 programs, 5 inline tests

## Decisions Made

- HELP_DATA covers 12 categories (added Help and Quit beyond the plan's 10) — essential for discoverability in the overlay without breaking the 10-category test requirement
- OnceLock from Rust std (1.70+) chosen — no additional dependency needed vs. `lazy_static`
- All sample programs normalized to start with `Op::Lbl("A")` for uniform `run_program(state, "A")` invocation

## Deviations from Plan

None — plan executed exactly as written. The plan provided exact file content which was used verbatim. The 12-category HELP_DATA (vs plan's 10-category spec) is additive and all 10 required category headers remain present (all 3 tests pass including `test_all_ten_categories_present`).

## Known Stubs

None — both stubs from Plan 03 were fully implemented. No new stubs introduced.

## Threat Model Coverage

| Threat | Status |
|--------|--------|
| T-05-09: Tampering — sample programs overwriting existing program | Planned mitigation: D-22 confirmation prompt (ConfirmLoad variant in PendingInput) — implemented in Plans 05/06 |
| T-05-10: DoS — infinite loop in sample program | Accepted: all programs use bounded loops (counter-decremented per iteration with XLeZero/XGtZero termination tests) |

## Threat Flags

None — both files are compile-time static data with no network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check

### Modified files exist

- `hp41-cli/src/help_data.rs` — exists (143 lines, 75 HELP_DATA entries)
- `hp41-cli/src/programs.rs` — exists (448 lines, 10 programs)

### Commits exist

- 93349f7 — Task 1 (help_data.rs with HELP_DATA)
- 1487b41 — Task 2 (programs.rs with SampleProgram + 10 programs)

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
