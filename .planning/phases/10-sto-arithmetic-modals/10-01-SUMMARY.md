---
phase: 10-sto-arithmetic-modals
plan: "01"
subsystem: hp41-core
tags: [core, ops, registers, sto-arithmetic, stack-reg]
dependency_graph:
  requires: []
  provides: [StackReg enum, Op::StoArithStack variant, op_sto_arith_stack function]
  affects: [hp41-core/src/ops/mod.rs, hp41-core/src/ops/registers.rs, hp41-core/src/ops/program.rs, hp41-cli/src/prgm_display.rs]
tech_stack:
  added: []
  patterns: [TDD RED/GREEN, atomicity guarantee (compute-first write-on-success), non-exhaustive pattern exhaustiveness]
key_files:
  created: []
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/registers.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
decisions:
  - StackReg uses Y/Z/T/LastX variants matching HP-41 stack register names
  - op_sto_arith_stack follows atomicity guarantee identical to op_sto_arith (compute first, write only on success)
  - LiftEffect::Neutral for StoArithStack (consistent with StoArith)
  - LASTX not saved (consistent with StoArith)
  - Tests use Decimal::from_str() instead of rust_decimal_macros::dec! (not in dev-dependencies)
  - prgm_display.rs display name for LastX is "L" (single char matches HP-41 style)
metrics:
  duration: "~15 minutes"
  completed: "2026-05-08T12:10:00Z"
  tasks_completed: 2
  files_modified: 4
---

# Phase 10 Plan 01: STO Arithmetic Stack Register Primitives Summary

StackReg enum + Op::StoArithStack variant + op_sto_arith_stack() with atomicity guarantee enabling STO+/-/x/div on Y/Z/T/LastX stack registers from hp41-core.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| RED  | Add failing tests for op_sto_arith_stack | b49756f | hp41-core/src/ops/registers.rs |
| 1    | StackReg enum, Op::StoArithStack, op_sto_arith_stack impl | 62806d9 | hp41-core/src/ops/mod.rs, registers.rs |
| 2    | Op::StoArithStack arm in execute_op() | f955603 | hp41-core/src/ops/program.rs |
| fix  | StoArithStack arm in prgm_display (Rule 3) | 260de42 | hp41-cli/src/prgm_display.rs |

## Verification Results

- `just ci` exits 0 — all lint, test, and coverage gates pass
- Coverage: 93.99% (gate ≥80%)
- 388 tests pass (148 lib tests including 3 new stack_arith_tests)
- No clippy warnings under `#![deny(clippy::unwrap_used)]`
- `grep "pub enum StackReg" hp41-core/src/ops/mod.rs` → 1 match
- `grep "pub fn op_sto_arith_stack" hp41-core/src/ops/registers.rs` → 1 match
- `grep "StoArithStack" hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs` → 3 matches

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tests referenced unavailable `rust_decimal_macros::dec!` macro**
- **Found during:** Task 1 GREEN phase
- **Issue:** Plan specified `use rust_decimal_macros::dec;` in test code but `rust_decimal_macros` is not in hp41-core dev-dependencies and would fail to compile
- **Fix:** Replaced all `dec!(N)` calls with `Decimal::from_str("N").expect("test literal")` pattern using a local `fn d(s: &str) -> Decimal` helper. No Cargo.toml change needed.
- **Files modified:** `hp41-core/src/ops/registers.rs`
- **Commit:** 62806d9

**2. [Rule 3 - Blocking] Op::StoArithStack caused non-exhaustive pattern error in prgm_display.rs**
- **Found during:** Task 1/2 combined — `just ci` lint pass
- **Issue:** `hp41-cli/src/prgm_display.rs::op_display_name()` exhaustively matches all Op variants; adding `Op::StoArithStack` without updating prgm_display.rs broke the hp41-cli build
- **Fix:** Added `StackReg` import and `Op::StoArithStack { kind, stack_reg }` match arm with display format `"STO{op_sym} {reg_name}"` where LastX displays as "L"
- **Files modified:** `hp41-cli/src/prgm_display.rs`
- **Commit:** 260de42

**3. [Rule 3 - Blocking] Op::StoArithStack non-exhaustive in program.rs execute_op()**
- **Found during:** Task 1 GREEN phase compilation
- **Issue:** Adding `Op::StoArithStack` to the Op enum immediately caused a non-exhaustive patterns error in `execute_op()` in program.rs (Task 2's work was required to compile Task 1)
- **Fix:** Completed Task 2 inline — added `op_sto_arith_stack` to the use statement and the `Op::StoArithStack { kind, stack_reg }` dispatch arm — as part of Task 1 GREEN phase to unblock compilation
- **Files modified:** `hp41-core/src/ops/program.rs`
- **Commit:** f955603 (committed separately as Task 2)

## TDD Gate Compliance

- RED commit (b49756f): `test(10-01): add failing tests for op_sto_arith_stack RED phase`
  - Tests referenced undefined `StackReg` and `op_sto_arith_stack` — confirmed compile failure
- GREEN commit (62806d9): `feat(10-01): implement StackReg enum, Op::StoArithStack, op_sto_arith_stack`
  - All 3 new tests pass; 148 total lib tests pass
- No REFACTOR needed — implementation was clean on first pass

## Known Stubs

None — all functionality is fully implemented. Plans 02 and 03 will wire the TUI modal.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Op::StoArithStack is pure in-memory computation on CalcState, same trust boundary as the existing Op::StoArith.

## Self-Check: PASSED

- hp41-core/src/ops/mod.rs exists and contains `pub enum StackReg` and `StoArithStack`
- hp41-core/src/ops/registers.rs exists and contains `pub fn op_sto_arith_stack`
- hp41-core/src/ops/program.rs exists and contains `StoArithStack` arm
- hp41-cli/src/prgm_display.rs exists and contains `StoArithStack` arm
- Commits b49756f, 62806d9, f955603, 260de42 all present in git log
