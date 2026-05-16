---
phase: 28-xrom-framework-math-pac-i-core-ops
plan: "03"
subsystem: hp41-core
tags:
  - complex-arithmetic
  - math-pac-i
  - xrom
  - stack-overlay
dependency_graph:
  requires:
    - 28-01 (XROM framework + CalcState complex_mode field)
    - 28-02 (hyperbolics proof-of-pattern)
  provides:
    - Op::CPlus, Op::CMinus, Op::CTimes, Op::CDiv (binary complex arithmetic)
    - Op::Real (complex_mode deactivation — CMPLX-18 derived requirement)
    - complex_atan2 helper (pub(super) — available to Plan 28-04 transcendentals)
    - CMPLX-18 entry in REQUIREMENTS.md
    - math1_complex_edge_cases.rs partially filled (2 of 4 arithmetic cases)
  affects:
    - hp41-core/src/ops/mod.rs (Op enum + dispatch)
    - hp41-core/src/ops/program.rs (execute_op)
    - hp41-core/src/ops/math1/complex.rs (new)
    - hp41-core/src/ops/math1/mod.rs (complex module added)
    - hp41-core/src/ops/math1/xrom.rs (MATH_1.ops + math1_resolve)
    - hp41-core/tests/math1_complex.rs (new)
    - hp41-core/tests/math1_complex_edge_cases.rs (2 of 4 cases filled)
    - hp41-core/tests/numerical_accuracy.rs (8 new complex cases)
    - hp41-cli/src/prgm_display.rs (5 new arms)
    - hp41-gui/src-tauri/src/prgm_display.rs (5 new arms)
    - .planning/REQUIREMENTS.md (CMPLX-18 added)
tech_stack:
  added: []
  patterns:
    - Complex-stack overlay model (D-28.1): ζ=X+iY, τ=Z+iT; no new HpNum storage
    - Binary-complex T-replicate: new Z and T both get old T (HP-41 hardware behavior)
    - auto-on complex_mode (D-28.2): every binary complex op sets complex_mode=true first
    - Zero-divisor guard BEFORE mutation (Pitfall 6 / CMPLX-05): checked before auto-on
    - complex_atan2 (0,0)→HpNum::zero() first arm (Pitfall 6 gate)
key_files:
  created:
    - hp41-core/src/ops/math1/complex.rs
    - hp41-core/tests/math1_complex.rs
  modified:
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/math1_complex_edge_cases.rs
    - hp41-core/tests/numerical_accuracy.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
    - .planning/REQUIREMENTS.md
decisions:
  - "Complex stack is an overlay on X/Y/Z/T: ζ=X+iY, τ=Z+iT (D-28.1 — zero new HpNum storage)"
  - "Binary complex ops T-replicate: new Z and T both get old T value after the stack drop"
  - "complex_mode auto-on fires AFTER zero-divisor guard in CDiv (Pitfall 6 — guard is pre-mutation)"
  - "complex_atan2 pub(super) — not pub — available only to sibling math1 modules (Plan 28-04 uses it)"
  - "MATH_1.ops has 13 entries: 6 hyperbolic + 7 complex (C+, C-, C×, C*, C÷, C/, REAL with ASCII aliases)"
metrics:
  duration: "~35 minutes"
  completed: "2026-05-16"
  tasks_completed: 4
  tasks_total: 4
  files_created: 2
  files_modified: 9
---

# Phase 28 Plan 03: Complex Stack Arithmetic Summary

**One-liner:** 5 complex-arithmetic Op variants (CPlus/CMinus/CTimes/CDiv/Real) landed as a zero-storage overlay on X/Y/Z/T with Pitfall-6-safe zero-divisor guard and complex_atan2 helper ready for Plan 28-04 transcendentals.

## What Was Built

### Task 1: CMPLX-18 in REQUIREMENTS.md

- Added CMPLX-18 entry to the Complex Stack section: `Op::Real (XEQ "REAL") — deactivates complex_mode; no stack effects; derived from D-28.3`
- Added CMPLX-18 row to traceability table: Phase 28 / Plan 28-03
- Updated coverage count: 110 → 111 requirements total

### Task 2: complex.rs module (complex_atan2 + 5 op functions)

Created `hp41-core/src/ops/math1/complex.rs`:

**complex_atan2 helper (Pitfall 6 gate):**
- First arm: `(im.is_zero() && re.is_zero())` returns `HpNum::zero()` (not NaN, not Domain)
- All other cases: f64 bridge via `im.atan2(re)` with Decimal round-trip

**5 op functions:**
- `op_c_plus` — ζ+τ: X'=X+Z, Y'=Y+T; T-replicate (Z'=T'=old_T); LiftEffect::Enable; auto-on
- `op_c_minus` — ζ-τ: X'=X-Z, Y'=Y-T; same T-replicate shape
- `op_c_times` — ζ·τ: X'=XZ-YT, Y'=XT+YZ (4 mults + 2 adds); T-replicate
- `op_c_div` — zero-divisor guard FIRST (before `state.complex_mode = true`), then ((XZ+YT)+i(YZ-XT))/(Z²+T²)
- `op_real` — `state.complex_mode = false`; LiftEffect::Neutral; stack untouched

Inline test suite: 35 unit tests (≥5 per op + 5 for complex_atan2).

Added `pub mod complex;` to `hp41-core/src/ops/math1/mod.rs`.

### Task 3: Op enum + dispatch + execute_op + xrom registration

- Added 5 variants to `Op` enum in `ops/mod.rs` with full doc comments
- Added 5 dispatch arms in `dispatch()` in `ops/mod.rs`
- Added 5 `execute_op` arms in `ops/program.rs`
- Populated `MATH_1.ops` with 7 entries (C+, C-, C×, C*, C÷, C/, REAL — ASCII aliases for × and ÷)
- Extended `math1_resolve()` with 5 match arms (Unicode | ASCII pattern per OM multi-spelling tolerance)
- Updated `math1_ops_has_correct_entry_count` test: 6 → 13 entries
- Filled `math1_complex_edge_cases.rs` arithmetic cases (2 of 4 active; 2 remain `#[ignore]` for Plan 28-04)
- Added 8 complex numerical accuracy cases to `numerical_accuracy.rs` (C+, C-, C×, C÷ each with 2 cases)
- Created `hp41-core/tests/math1_complex.rs`: 31 dispatch-path integration tests (≥6 per variant, satisfies Pitfall-16 meta-test)

### Task 4: prgm_display.rs exhaustive match arms (both copies)

Added 5 arms under `// ── Phase 28: Complex Stack Arithmetic (Plan 28-03)` section:
- `Op::CPlus => "C+"`
- `Op::CMinus => "C-"`
- `Op::CTimes => "C\u{00D7}"` (Unicode ×)
- `Op::CDiv => "C\u{00F7}"` (Unicode ÷)
- `Op::Real => "REAL"`

`cargo build --workspace` confirms exhaustive match on both `hp41-cli` and `hp41-gui/src-tauri`.

## Test Results

| Gate | Result |
|------|--------|
| `cargo build -p hp41-core` | PASS (1 dead_code warning — complex_atan2 pub(super), used by Plan 28-04) |
| `cargo build --workspace` | PASS |
| `cargo test -p hp41-core` | 1096 passed, 7 ignored |
| `--test math1_complex` | 31 passed |
| `--lib math1::complex::tests` | 35 passed |
| `--test math1_complex_edge_cases` | 2 passed, 2 ignored (Plan 28-04 placeholders) |
| `--test xrom_shadowing` | 2 passed (13 mnemonics confirmed non-shadowing) |
| `--test xrom_chain_order` | 5 passed |
| `--test math1_op_test_count` | 1 passed (≥5 tests per variant confirmed) |
| `--test numerical_accuracy` | 8 passed (8 new complex cases: C+, C-, C×, C÷) |

## Deviations from Plan

### Auto-fixed Issues

None - plan executed exactly as written with one minor clarification:

**[Rule 1 - Bug] math1_op_test_count expected old count**
- **Found during:** Task 3
- **Issue:** `math1_ops_has_six_hyperbolic_entries` test asserted `MATH_1.ops.len() == 6`, but Plan 28-03 adds 7 more entries (C+, C-, C×, C*, C÷, C/, REAL). The old test was correct at Plan 28-02 but becomes a false failure at Plan 28-03.
- **Fix:** Renamed test to `math1_ops_has_correct_entry_count` and updated assertion to `== 13`.
- **Files modified:** `hp41-core/src/ops/math1/xrom.rs`

## Known Stubs

None. All 5 Op variants are fully implemented and wired. `complex_atan2` is a real implementation (not a stub), though marked `dead_code` until Plan 28-04 uses it in transcendental complex functions.

The 2 `#[ignore]`-tagged test cases in `math1_complex_edge_cases.rs` are intentional Plan 28-04 placeholders (not stubs in the UI/data-flow sense — they represent future behavior tests for LNZ and Z↑W operations).

## Plan 28-04 Inheritance

Plan 28-04 (complex transcendentals) can now rely on:
- `state.complex_mode` auto-on/off lifecycle (established by this plan)
- `crate::ops::math1::complex::complex_atan2` helper (pub(super) access from sibling module)
- T-replicate stack semantics pattern (established for binary complex ops)
- CMPLX-05 zero-divisor-before-mutation pattern (reusable for Z↑W zero-case check)

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `hp41-core/src/ops/math1/complex.rs` exists | FOUND |
| `hp41-core/tests/math1_complex.rs` exists | FOUND |
| Commit 08a5568 exists (Task 1) | FOUND |
| Commit be2d184 exists (Task 2) | FOUND |
| Commit 392fbc6 exists (Task 3) | FOUND |
| Commit 12a867a exists (Task 4) | FOUND |
| 5 Op variants in hp41-cli prgm_display.rs | FOUND (5 lines) |
| 5 Op variants in hp41-gui prgm_display.rs | FOUND (5 lines) |
| CMPLX-18 in REQUIREMENTS.md | FOUND |
| CMPLX-18 traceability row → Plan 28-03 | FOUND |
| cargo test -p hp41-core: all pass | 1096 passed |
| cargo build --workspace: clean | PASS |
| STATE.md and ROADMAP.md unmodified | CONFIRMED |
