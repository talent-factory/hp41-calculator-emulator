---
phase: 05-persistence-and-ux
plan: 02
subsystem: ops
tags: [rust, serde, ops, alpha, user-mode, dispatch]

# Dependency graph
requires:
  - phase: 05-persistence-and-ux
    plan: 01
    provides: Op, StoArithKind, TestKind with Serialize/Deserialize; CalcState with user_mode field

provides:
  - op_alpha_backspace: HP-41 ← key behavior — removes last char from alpha_reg, no-op if empty
  - Op::UserMode variant: flips state.user_mode, Neutral lift effect
  - Op::AlphaBackspace variant: calls op_alpha_backspace, Neutral lift effect
  - dispatch() arms for both new variants
  - execute_op() arms for both new variants (program recording/playback context)

affects:
  - 05-06 (ALPHA backspace UX — AlphaBackspace variant available in dispatch)
  - 05-07 (USER mode dispatch — Op::UserMode variant and user_mode flip wired)
  - 05-03 (persistence — Op enum already serializable from Plan 01; new variants serialize automatically)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AlphaBackspace uses String::pop() — returns Option<char>, no-op on empty string — no error path needed"
    - "UserMode inline in dispatch and execute_op — simple boolean flip, no separate function needed"
    - "Both new variants added to execute_op() catch-before-wildcard pattern to maintain exhaustiveness"

key-files:
  created: []
  modified:
    - hp41-core/src/ops/alpha.rs (op_alpha_backspace function + 2 inline unit tests)
    - hp41-core/src/ops/mod.rs (Op::UserMode + Op::AlphaBackspace variants; alpha import updated; dispatch arms; 3 inline tests)
    - hp41-core/src/ops/program.rs (execute_op arms for UserMode and AlphaBackspace; alpha import updated)

key-decisions:
  - "UserMode inline dispatch (not a separate function) — toggle is a one-liner, no helper needed"
  - "AlphaBackspace reuses op_alpha_backspace from alpha.rs in both dispatch() and execute_op() — consistent with all other alpha ops"
  - "String::pop() selected over manual index splice — idiomatic Rust, handles multibyte chars correctly, no-op on empty"

requirements-completed: [PERS-01, UX-02]

# Metrics
duration: 10min
completed: 2026-05-07
---

# Phase 5 Plan 02: Op Serde + UserMode + AlphaBackspace Summary

**op_alpha_backspace implemented with String::pop(); Op::UserMode and Op::AlphaBackspace variants added to Op enum with full dispatch and program execution wiring**

## Performance

- **Duration:** ~10 min
- **Completed:** 2026-05-07
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- `op_alpha_backspace` added to `alpha.rs`: uses `String::pop()` — correct HP-41 ← key behavior (no-op on empty string, no error)
- `Op::UserMode` and `Op::AlphaBackspace` variants added to the `Op` enum in `ops/mod.rs`
- `dispatch()` in `mod.rs` routes both new variants: `AlphaBackspace` calls `op_alpha_backspace`, `UserMode` flips `state.user_mode`
- `execute_op()` in `program.rs` handles both variants in program recording/playback context
- All 287 hp41-core tests pass — zero regressions

## Task Commits

1. **Task 1: Add op_alpha_backspace to alpha.rs** - `c7a8b7b` (feat)
2. **Task 2: Add serde derives to Op; add UserMode + AlphaBackspace variants; wire dispatch** - `0336e3c` (feat)
3. **Task 3: Wire UserMode and AlphaBackspace into execute_op() in program.rs** - `6883649` (feat)

## Files Created/Modified

- `hp41-core/src/ops/alpha.rs` — added `op_alpha_backspace` (uses `String::pop()`, LiftEffect::Neutral) + 2 inline unit tests
- `hp41-core/src/ops/mod.rs` — added `Op::UserMode` and `Op::AlphaBackspace` variants; updated alpha import; added dispatch arms; 3 inline tests (UserMode toggle, serde round-trips)
- `hp41-core/src/ops/program.rs` — added `op_alpha_backspace` to alpha import; added `Op::AlphaBackspace` and `Op::UserMode` arms to `execute_op()` match

## Decisions Made

- `String::pop()` chosen over manual char removal — idiomatic Rust, handles multibyte UTF-8 safely, returns `Option<char>` (None on empty = correct HP-41 no-error behavior)
- `UserMode` wired inline in dispatch and execute_op (not a separate function) — a one-liner boolean flip needs no indirection
- Both variants added before the programming-ops catch-all in `execute_op()` — maintains exhaustive match structure

## Deviations from Plan

### Context: serde derives already present

The plan spec said "Add serde derives to Op/StoArithKind/TestKind" but Plan 01 already added these as an auto-fix (Rule 2 deviation in 05-01). The derives were already in place at the start of this plan. No action needed — noted as observation, not a deviation.

### Auto-fixed Issues

None — plan executed exactly as written (excluding the already-done serde derives noted above).

## Threat Model Coverage

Per the plan's threat model:

| Threat | Status |
|--------|--------|
| T-05-03: serde::from_reader on unknown Op variants | Covered — serde returns Err for unknown variants; caller (Plan 03) catches and resets |
| T-05-04: Op::UserMode in prgm_mode recording | Accepted — UserMode recorded to program Vec when prgm_mode=true, same as all other ops |

## Next Phase Readiness

- Plan 03 (persistence.rs) can serialize/deserialize programs containing `Op::UserMode` and `Op::AlphaBackspace` — both are serde-derivable via Op's existing derives
- Plan 06 (ALPHA backspace UX) can dispatch `Op::AlphaBackspace` — fully wired
- Plan 07 (USER mode dispatch) can dispatch `Op::UserMode` — fully wired, `state.user_mode` toggled correctly

## Self-Check

### Created files exist

- `hp41-core/src/ops/alpha.rs` — exists, contains `pub fn op_alpha_backspace`
- `hp41-core/src/ops/mod.rs` — exists, contains `Op::UserMode` and `Op::AlphaBackspace`
- `hp41-core/src/ops/program.rs` — exists, contains `Op::AlphaBackspace` and `Op::UserMode` arms

### Commits exist

- c7a8b7b — Task 1 (alpha.rs)
- 0336e3c — Task 2 (mod.rs)
- 6883649 — Task 3 (program.rs)

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
