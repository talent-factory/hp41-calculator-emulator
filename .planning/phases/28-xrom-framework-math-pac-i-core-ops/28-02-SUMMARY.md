---
phase: 28-xrom-framework-math-pac-i-core-ops
plan: "02"
subsystem: hp41-core
tags:
  - hyperbolics
  - math-pac-i
  - xrom
  - proof-of-pattern
dependency_graph:
  requires:
    - 28-01 (XROM framework)
  provides:
    - Op::Sinh, Op::Cosh, Op::Tanh, Op::Asinh, Op::Acosh, Op::Atanh
    - MATH_1.ops populated with 6 entries
    - Proof-of-pattern for Plans 28-03..28-10
  affects:
    - hp41-core/src/ops/mod.rs (Op enum + dispatch)
    - hp41-core/src/ops/program.rs (execute_op)
    - hp41-core/src/ops/math1/hyperbolics.rs (new)
    - hp41-core/src/ops/math1/mod.rs (hyperbolics module added)
    - hp41-core/src/ops/math1/xrom.rs (MATH_1.ops + math1_resolve)
    - hp41-core/tests/numerical_accuracy.rs (18 new cases)
    - hp41-cli/src/prgm_display.rs (6 new arms)
    - hp41-gui/src-tauri/src/prgm_display.rs (6 new arms)
tech_stack:
  added: []
  patterns:
    - f64-bridge transcendental pattern (angle-mode-independent variant)
    - xrom_resolve bit-0 module-load gate
    - unary_result (LiftEffect::Enable) for all 6 hyperbolic ops
key_files:
  created:
    - hp41-core/src/ops/math1/hyperbolics.rs
    - hp41-core/tests/math1_hyperbolics.rs
  modified:
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/numerical_accuracy.rs
    - hp41-core/tests/xrom_chain_order.rs
    - hp41-core/tests/math1_op_test_count.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
decisions:
  - "LiftEffect::Enable for all 6 hyperbolics (via unary_result) — matches v2.2 transcendental pattern; plan text contained internal contradiction (stated Disable but referenced unary_result which always Enables)"
  - "Domain guards: op_acosh returns Domain for X < 1.0; op_atanh returns Domain for |X| >= 1.0; sinh/cosh return Overflow for extreme magnitudes (inf from f64)"
  - "math1_op_test_count.rs fixed to use CARGO_MANIFEST_DIR instead of relative 'hp41-core/tests' path (Rule 1 auto-fix: wrong CWD assumption in worktree context)"
metrics:
  duration: "~25 minutes"
  completed: "2026-05-16"
  tasks_completed: 4
  tasks_total: 4
  files_created: 2
  files_modified: 9
---

# Phase 28 Plan 02: Hyperbolic Functions (Proof-of-Pattern) Summary

**One-liner:** 6 hyperbolic Math Pac I ops (SINH/COSH/TANH/ASINH/ACOSH/ATANH) landed end-to-end through Op enum, dispatch, execute_op, XROM registration, and both prgm_display copies, validating the Plan 28-01 framework pipeline.

## What Was Built

### Task 1: hyperbolics.rs module

Created `hp41-core/src/ops/math1/hyperbolics.rs` with 6 public op functions:

- `op_sinh(state)` — f64 bridge, no angle-mode conversion, overflow on extreme magnitudes
- `op_cosh(state)` — f64 bridge, symmetric (cosh(-x) = cosh(x) verified)
- `op_tanh(state)` — f64 bridge, saturates to ±1 for large |X| (correct, not an error)
- `op_asinh(state)` — f64 bridge, no domain restriction
- `op_acosh(state)` — Domain guard: X < 1.0 → HpError::Domain
- `op_atanh(state)` — Domain guard: |X| >= 1.0 → HpError::Domain

All 6 ops use `unary_result` (saves X to LASTX, sets lift_enabled=true, LiftEffect::Enable).

Inline test suite: 32 unit tests (5+ per op covering identity, reference values, domain edges, LiftEffect, LASTX, angle-mode independence).

### Task 2: Wire through dispatch + execute_op + xrom

- Added 6 variants to `Op` enum: `Op::Sinh`, `Op::Cosh`, `Op::Tanh`, `Op::Asinh`, `Op::Acosh`, `Op::Atanh`
- Added 6 dispatch arms in `dispatch()` in `ops/mod.rs`
- Added 6 `execute_op` arms in `ops/program.rs` (using `crate::ops::math1::hyperbolics::*`)
- Populated `MATH_1.ops` with 6 entries: `("SINH", Op::Sinh)` through `("ATANH", Op::Atanh)`
- Implemented `math1_resolve` match block with all 6 mnemonics
- Updated `xrom_chain_order.rs`: replaced vacuous `SINH → InvalidOp` test with positive dispatch test `math1_sinh_resolves_via_xrom`
- Created `hp41-core/tests/math1_hyperbolics.rs`: 30 integration tests via dispatch()
- Fixed `math1_op_test_count.rs`: replaced `Path::new("hp41-core/tests")` with `CARGO_MANIFEST_DIR`-derived path (auto-fix: worktree CWD diverges from workspace root assumption)
- Added 5 positive-resolution tests to xrom.rs inline test module

### Task 3: Numerical accuracy cases

Added 18 cases to `hp41-core/tests/numerical_accuracy.rs` under "v3.0 EXTENSION" section:

| Op    | Cases | Values |
|-------|-------|--------|
| SINH  | 3     | sinh(0)=0, sinh(1)=1.175201194, sinh(-1)=-1.175201194 |
| COSH  | 3     | cosh(0)=1, cosh(1)=1.543080635, cosh(2)=3.762195691 |
| TANH  | 3     | tanh(0)=0, tanh(1)=0.761594156, tanh(-1)=-0.761594156 |
| ASINH | 3     | asinh(0)=0, asinh(1)=0.881373587, asinh(-1)=-0.881373587 |
| ACOSH | 3     | acosh(1)=0, acosh(2)=1.316957897, acosh(10)=2.993222846 |
| ATANH | 3     | atanh(0)=0, atanh(0.5)=0.549306144, atanh(-0.5)=-0.549306144 |

All cite HP 00041-90034 p.44-45 with Free42 v3.0.5 cross-check. v1.x 503-case baseline floor preserved (D-27.6).

### Task 4: prgm_display.rs exhaustive match arms

Added 6 arms to `op_display_name` in both copies under `// ── Phase 28: Hyperbolics (Plan 28-02)` section:
- `Op::Sinh => "SINH"`, `Op::Cosh => "COSH"`, `Op::Tanh => "TANH"`
- `Op::Asinh => "ASINH"`, `Op::Acosh => "ACOSH"`, `Op::Atanh => "ATANH"`

`cargo build --workspace` confirms exhaustive match (compiler enforces no missing arm).

## Test Results

| Gate | Result |
|------|--------|
| `cargo build -p hp41-core` | PASS |
| `cargo build --workspace` | PASS |
| `cargo test -p hp41-core` | 1029 passed, 9 ignored |
| `--test xrom_shadowing` | 2 passed (6 new mnemonics confirmed non-shadowing) |
| `--test xrom_chain_order` | 5 passed (SINH positive dispatch confirmed) |
| `--test math1_op_test_count` | 1 passed (>= 5 tests per variant confirmed) |
| `--test numerical_accuracy` | 8 passed (suite + 7 standalone; 18 new hyperbolic cases) |
| `--test math1_hyperbolics` | 30 passed (dispatch-path integration) |
| `--lib math1::hyperbolics::tests` | 32 passed (unit tests) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] LiftEffect plan contradiction**
- **Found during:** Task 1
- **Issue:** Plan stated "LiftEffect::Disable" and "state.stack.lift_enabled is false after op" but also said "Apply `unary_result(state, result)` for all 6" — `unary_result` always sets `lift_enabled = true` (Enable). The two instructions are mutually contradictory.
- **Fix:** Used `unary_result` as specified in the action, with test assertions corrected to `lift_enabled == true` (Enable) which matches the v2.2 transcendental pattern (op_sin, op_cos, op_asin all use `unary_result`). Hardware-faithful: HP-41 transcendental unary ops enable stack lift.
- **Files modified:** `hyperbolics.rs` (test assertions), `math1_hyperbolics.rs`

**2. [Rule 1 - Bug] math1_op_test_count CWD path wrong in worktree**
- **Found during:** Task 2
- **Issue:** `Path::new("hp41-core/tests")` assumes CWD = workspace root at runtime. In worktree context, the test binary's CWD behavior caused `read_dir` to silently fail (returns 0 mentions), making the meta-test count 0 for all variants.
- **Fix:** Replace with `env!("CARGO_MANIFEST_DIR").join("tests")` — compile-time macro resolves to the absolute path of hp41-core package directory, which is stable regardless of runtime CWD.
- **Files modified:** `hp41-core/tests/math1_op_test_count.rs`

**3. [Rule 3 - Blocking] xrom_chain_order test expected wrong result**
- **Found during:** Task 2
- **Issue:** The `future_math1_mnemonic_returns_invalid_op_at_plan_28_01` test asserted `XEQ "SINH" → InvalidOp` which was correct at Plan 28-01. At Plan 28-02 this would fail (SINH now resolves).
- **Fix:** Replaced with `math1_sinh_resolves_via_xrom` test asserting `XEQ "SINH" → Ok` (positive-resolution case required by Task 2's verification spec).
- **Files modified:** `hp41-core/tests/xrom_chain_order.rs`

## Proof-of-Pattern Validated

The Plan 28-01 framework now has a successful end-to-end run:

```
XEQ "SINH" (interactive op_xeq or programmatic Op::Xeq)
  → builtin_card_op: no match
  → xrom_resolve("SINH", xrom_modules)
    → math1 bit (bit 0) is set in CalcState::new() default
    → math1_resolve("SINH") → Some(Op::Sinh)
  → dispatch(state, Op::Sinh)
    → op_sinh(state)
      → f64 bridge: sinh(x)
      → unary_result (LASTX saved, lift enabled)
      → Ok(())
```

This validates the assembly line for downstream Plans 28-03..28-10. No re-design needed — the same pattern applies to every subsequent Math Pac I op group.

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `hp41-core/src/ops/math1/hyperbolics.rs` exists | FOUND |
| `hp41-core/tests/math1_hyperbolics.rs` exists | FOUND |
| Commit ecb51e5 exists (Task 1) | FOUND |
| Commit 9f913dd exists (Task 2) | FOUND |
| Commit acbd27c exists (Task 3) | FOUND |
| Commit 7edee31 exists (Task 4) | FOUND |
| 6 Op variants in hp41-cli prgm_display.rs | 6 lines confirmed |
| 6 Op variants in hp41-gui prgm_display.rs | 6 lines confirmed |
| cargo test -p hp41-core: all pass | 1029 passed |
| cargo build --workspace: clean | PASS |
