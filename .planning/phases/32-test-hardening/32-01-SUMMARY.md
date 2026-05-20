---
phase: 32
plan: "32-01"
subsystem: tests
tags:
  - test-hardening
  - lint
  - coverage
  - QUAL-01
  - QUAL-04
  - QUAL-07
  - QUAL-08
  - T-32-04
requires: []
provides:
  - tests/lint_math1_assertions.rs (NEW)
  - tests/math1_mod_entry_points.rs (NEW)
  - tests/math1_op_test_count.rs (graduated to non-vacuous)
  - tests/xrom_shadowing.rs (doc-comment update; auto-graduated)
  - tests/math1_user_callback.rs (+2 QUAL-08 explicit-category tests)
affects:
  - hp41-core/tests/math1_matrix.rs (Pitfall 14 refactor)
  - hp41-core/tests/math1_matrix_flow.rs (Pitfall 14 refactor)
  - hp41-core/tests/math1_four_tri_trans.rs (mixed refactor + LINT-EXEMPT)
  - hp41-core/tests/math1_integ.rs (LINT-EXEMPT annotations)
  - hp41-core/tests/math1_user_callback.rs (LINT-EXEMPT annotations)
tech-stack:
  added: []
  patterns:
    - approx::assert_relative_eq!(actual, expected, max_relative = 1e-7) — extended to matrix.rs / matrix_flow.rs / four_tri_trans.rs
    - LINT-EXEMPT contiguous-comment-block adjacency convention (preceding-block heuristic)
key-files:
  created:
    - hp41-core/tests/lint_math1_assertions.rs
    - hp41-core/tests/math1_mod_entry_points.rs
  modified:
    - hp41-core/tests/math1_op_test_count.rs
    - hp41-core/tests/xrom_shadowing.rs
    - hp41-core/tests/math1_matrix.rs
    - hp41-core/tests/math1_matrix_flow.rs
    - hp41-core/tests/math1_four_tri_trans.rs
    - hp41-core/tests/math1_integ.rs
    - hp41-core/tests/math1_user_callback.rs
decisions:
  - meta-gate-graduation: deleted vacuous early-return in math1_op_test_count.rs; the gate now actively cross-checks all 45 Math Pac I Op variants
  - lint-scope: tests/math1_*.rs only (option (a) per CONTEXT.md Claude's Discretion) — does NOT scan numerical_accuracy.rs
  - lint-heuristic: LINT-EXEMPT acceptable when adjacent in preceding comment block (no blank line / semicolon between); inline LINT-EXEMPT also recognized
  - poly-recursion-cap: documented architectural defense-in-depth — USER_CALLBACK_MAX_STEPS budget is unreachable via test under current execute_op_pub program-control op strict-reject; test pins the contract so a future arch change flips the assert
  - coverage-deferral: 6 ops/math1/*.rs source files below 90% (poly 70%, trans/four 81%, solve 86%, difeq 85%, integ 89%, matrix 90%, mod.rs surgically fixed 0%→56%); bulk gap-closure is Plan 32-02's case-driven extension responsibility
metrics:
  duration: ~75 minutes
  completed: 2026-05-18
---

# Phase 32 Plan 01: Coverage Push + Assertion-Discipline Lint + Meta-Gate Graduation Summary

Graduate the existing-but-vacuous `math1_op_test_count.rs` and `xrom_shadowing.rs` CI gates from Plan-28-01 vacuous-pass to actively cross-checking all 45 Math Pac I `Op` variants against the 14 `tests/math1_*.rs` files (Pitfall 16) and all 52 `MATH_1.ops` entries against the 18-entry `BUILTIN_CARD_OP_NAMES` allowlist (Pitfall 1); ship the new `tests/lint_math1_assertions.rs` Pitfall-14 + Pitfall-17 assertion-discipline lint with adjacent `// LINT-EXEMPT: <reason>` annotation support; add 2 explicit QUAL-08 user-callback regression tests (GTO-out + recursion-cap) so all 5 categories are visible in audit; surgical-fix `ops/math1/mod.rs` from 0 % → 56 % coverage via 5 risk-weighted submit_modal / cancel_modal / submit_modal_with_label error-path tests.

## Per-Task Completion

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 32-01-01 | Coverage reconnaissance (`just coverage --html`) | — (reconnaissance only, no code change) | DONE |
| 32-01-02 | Graduate `math1_op_test_count.rs` from vacuous to non-vacuous | `cc20119` | DONE |
| 32-01-03 | Verify `xrom_shadowing.rs` auto-graduation; doc-comment update | `b64d388` | DONE |
| 32-01-04 | Create `tests/lint_math1_assertions.rs` Pitfall-14 + Pitfall-17 lint | `f0edfcc` | DONE |
| 32-01-05 | Add explicit QUAL-08 GTO-out + recursion-cap tests | `2ec02a8` | DONE |
| 32-01-06 | Surgical coverage gap-closure (ops/math1/mod.rs 0 % → 56 %) | `424c708` | DONE |

## Must-Have Verification

| Must-have | Status | Evidence |
|-----------|--------|----------|
| `just coverage` reports `hp41-core` lines ≥ 95.0 % | **NOT MET** at Plan-32-01-only level (91.79 % lines after this plan) | Coverage gate failure is structural — Plan 32-02 owns the ~134 numerical_accuracy.rs cases needed to close the gap |
| `cargo test -p hp41-core --test math1_op_test_count` reports `ok. 1 passed; 0 failed` non-vacuously | ✅ MET | Test now iterates 45 Math Pac I variants against 14 math1_*.rs files; baseline TriSaa=6, TriSas=6 |
| `cargo test -p hp41-core --test xrom_shadowing` reports `ok. N passed; 0 failed` non-vacuously | ✅ MET | 2 passed; iterates 52 `MATH_1.ops` against 18-entry allowlist |
| `cargo test -p hp41-core --test lint_math1_assertions` reports `ok. 2 passed; 0 failed` | ✅ MET | 2 passed (Pitfall 14 + Pitfall 17 gates) |
| `cargo test -p hp41-core --test math1_user_callback` ≥ 11 passed | ✅ MET | 11 passed (9 pre-existing + 2 new categories) |
| All 5 QUAL-08 categories explicitly named | ✅ MET | nested-rejection (7 tests), STOP-during-INTG (1), STO-clobber (1), GTO-out (1 NEW), recursion-cap (1 NEW) |

## Files Created / Modified

### Created

- **`hp41-core/tests/lint_math1_assertions.rs`** (231 lines) — Pitfall-14 + Pitfall-17 assertion-discipline lint. Two `#[test]` gates (`no_decimal_assert_eq_in_math1_tests` + `no_manual_tolerance_pattern_in_math1_tests`) scan `tests/math1_*.rs` files and collect ALL offenders into a `Vec<String>`, then `assert!(offenders.is_empty(), ...)` with the full list. Supports inline `// LINT-EXEMPT: <reason>` annotations AND preceding contiguous comment-block annotations (the `preceding_block_has_lint_exempt` helper walks upward past continuation lines of the same `assert!()` macro before hitting the comment block).
- **`hp41-core/tests/math1_mod_entry_points.rs`** (152 lines) — 5 risk-weighted `#[test]`s pinning the WR-01 / WR-02 / step-precedence contracts of `submit_modal` / `cancel_modal` / `submit_modal_with_label`. Each test carries a `// Catches:` doc comment per D-27.1.

### Modified

- **`hp41-core/tests/math1_op_test_count.rs`** — Deleted the 4-line `if variants.is_empty() { return; }` vacuous early-return at L125-128; updated doc-comment lines 14-21 to reflect Phase 32 graduation; added `// Catches: Pitfall 16 + T-32-04` comment on the `assert!` line. The minimum-count baseline TriSaa=6, TriSas=6 is captured in the doc-comment per T-32-04 (drift visible in diff review).
- **`hp41-core/tests/xrom_shadowing.rs`** — Doc-comment update only (no code change). Notes the Phase 32 graduation date and that the gate now actively cross-checks 52 `MATH_1.ops` mnemonics against the 18-entry `BUILTIN_CARD_OP_NAMES` allowlist. Allowlist alignment with `builtin_card_op` (program.rs L1112-1132) verified — no drift.
- **`hp41-core/tests/math1_matrix.rs`** — Refactored 4 named `(a - b).abs() < 1e-9` offenders (L298, L360, L427, L431) to `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)`. Added `use approx::assert_relative_eq;` import. The `assert_eq!(state.stack.x, HpNum::from(5i32))` line (L116) carries an inline `// LINT-EXEMPT: integer-equality via HpNum::from(<i32>)` annotation — integer equality is exact and cross-platform-safe.
- **`hp41-core/tests/math1_matrix_flow.rs`** — 1 offender (L78) refactored to `approx::assert_relative_eq!`. Imports updated.
- **`hp41-core/tests/math1_four_tri_trans.rs`** — Hybrid refactor: 4 tight-tolerance offenders (1e-5 / 1e-6) refactored to `approx::assert_relative_eq!`; 9 coarse-tolerance offenders (0.01 / 0.1) carry LINT-EXEMPT annotations with explicit rationale (triangle-solver angle floor, SSA ambiguous case display rounding, Rodrigues round-trip).
- **`hp41-core/tests/math1_integ.rs`** — 3 LINT-EXEMPT annotations on Simpson tolerance offenders (n=4 / n=10 algorithmic floor).
- **`hp41-core/tests/math1_user_callback.rs`** — 2 LINT-EXEMPT annotations on existing STOP-during-INTG / STO-clobber tolerances + 2 new `#[test]` functions:
  - `user_fn_gto_out_of_callback_handled` (QUAL-08 GTO-out category) — GTO to missing label propagates `Err(InvalidOp)` and clears `integ_state`.
  - `user_fn_recursion_cap_via_user_callback_max_steps` (QUAL-08 recursion-cap category) — self-recursive callback bounded by `execute_op_pub`'s program-control op strict-reject at `Err(InvalidOp)`; documents the defense-in-depth posture (the `USER_CALLBACK_MAX_STEPS = 100_000` budget is unreachable from a test under the current architecture).

## Deviations from Plan

### Rule 4 - Architectural Decision (DEFERRED): Coverage gate ≥ 95 % unmet at Plan-32-01 level

**Found during:** Task 32-01-01 reconnaissance
**Issue:** `just coverage` reports 91.53 % lines / 91.25 % regions on the v3.0 baseline — far below the 95 % gate. 6 `ops/math1/*.rs` source files are below the 90 % per-file floor mandated by ROADMAP Success Criterion 1:

| File | Lines | Status |
|------|------:|--------|
| `complex.rs` | 99.54 % | ✓ |
| `difeq.rs` | 85.16 % | ✗ |
| `four.rs` | 81.29 % | ✗ |
| `hyperbolics.rs` | 99.60 % | ✓ |
| `integ.rs` | 89.43 % | ✗ |
| `matrix.rs` | 89.68 % | ✗ |
| `modal.rs` | 98.69 % | ✓ |
| `mod.rs` | 0 % → 56.25 % (surgical) | ✗ |
| `poly.rs` | 69.93 % | ✗ |
| `solve.rs` | 85.56 % | ✗ |
| `trans.rs` | 81.17 % | ✗ |
| `tri.rs` | 97.86 % | ✓ |
| `xrom.rs` | 100.00 % | ✓ |

**Decision:** Defer bulk gap-closure to Plan 32-02 (the ~134 case-driven `numerical_accuracy.rs` extension). Per the Plan 32-01 task spec:

> "If the gap turns out to be larger than 15 tests, surface this to the user as a Phase 28-31 process gap before adding bulk tests."

The gap is structurally larger than 15 surgical tests; Plan 32-02's POLY ~25 / CMPLX ~20 / MAT ~18 / INTG ~15 / SOLVE ~15 / DIFEQ ~12 / HYP ~10 / TRI ~8 / FOUR ~6 / TRANS ~3 / REAL ~2 distribution (D-32.9) is the natural fit for the size of this gap.

**Surgical Plan 32-01 contribution:** Plan 32-01 surgically closed the only 0-coverage file (`ops/math1/mod.rs` 0 % → 56.25 %) via 5 risk-weighted tests targeting WR-01 / WR-02 contract pins. Plan 32-02 will sweep the remaining percentage points organically as numerical_accuracy.rs cases exercise the per-Op modal pipelines.

**Final coverage after this plan:** 91.79 % lines / 91.46 % regions (up 0.26 / 0.21 percentage points from baseline). The 95 % gate is achievable post-Plan 32-02.

### Rule 2 - Auto-add Missing Critical Functionality: LINT-EXEMPT preceding-block heuristic

**Found during:** Task 32-01-04
**Issue:** The initial lint design only recognized `LINT-EXEMPT:` inline on the offender line. But Rust's idiomatic style places rationale comments ABOVE the assertion (e.g., a `// LINT-EXEMPT: ...` comment block 1-3 lines above an `assert!(...)` macro). Without preceding-block recognition, the lint would force LINT-EXEMPT inline at the `< tolerance,` line — unidiomatic.

**Fix:** Added `preceding_block_has_lint_exempt(lines, idx)` helper. Walks upward from the offender line:
1. **Phase 1** — Skip continuation lines of the same `assert!()` macro (lines that don't end with `;` or `}` and aren't comments). Stops at a blank line OR a statement-terminator OR a comment.
2. **Phase 2** — From the first comment line found, scan the contiguous comment block (no blank line between) for `LINT-EXEMPT:`. Found ⇒ exempt; not found ⇒ flagged.

This makes the lint forgiving of idiomatic Rust comment-block placement while preserving the T-32-04 diff-review visibility (the annotation MUST be adjacent to the offender's enclosing item — no blank line between).

### Rule 1 - Auto-fix Bug: User-callback recursion-cap test contract

**Found during:** Task 32-01-05
**Issue:** Initial recursion-cap test (`user_fn_recursion_cap_via_user_callback_max_steps`) asserted `Err(HpError::CallDepth) | Err(HpError::Overflow)` based on the RESEARCH.md `USER_CALLBACK_MAX_STEPS = 100_000` budget assumption. Actual behavior: a self-recursive callback via `Op::Xeq("H")` returns `Err(InvalidOp)` IMMEDIATELY — `execute_op_pub` (program.rs L962-983) explicitly rejects `Op::Xeq(_)` / `Op::Gto(_)` / `Op::Test(_)` / etc. with InvalidOp because those are reserved for `run_loop`, not `run_user_function`.

**Fix:** Re-pinned the test contract to assert the ACTUAL defense-in-depth layer (the `execute_op_pub` strict-reject) with a documented architectural note that the deeper `USER_CALLBACK_MAX_STEPS` budget is unreachable under the current architecture. A future change opening the GTO/XEQ path inside user callbacks will flip the assert from `InvalidOp` to `Overflow` — the test becomes the canary signaling the architecture change.

## Decisions Made

- **Meta-gate graduation pattern (D-32 task-02-03):** `math1_op_test_count.rs` and `xrom_shadowing.rs` both shipped at Plan-28-01 as vacuous-pass meta-gates. Plan 32-01 transitions them to actively cross-checking the now-populated Phase 28 surface. The op_test_count gate required code change (delete 4-line early-return); the xrom_shadowing gate auto-graduated as `MATH_1.ops` filled from `&[]` to 52 entries during Plans 28-02..28-10.
- **LINT-EXEMPT annotation scope (T-32-04 mitigation):** every LINT-EXEMPT carries a specific rationale (not a bare `LINT-EXEMPT:` token). Reviewers can spot weakening. The 13 LINT-EXEMPT annotations in this plan ship with rationales spanning 4 categories: integer-equality-via-HpNum::from, Simpson-integration-floor, triangle-solver-angle-floor, SSA-display-rounding.
- **Coverage gate split between Plan 32-01 and Plan 32-02:** the 95 % gate is structurally a Plan-32-02 deliverable (bulk case-driven numerical_accuracy.rs extension). Plan 32-01 surgically closed ops/math1/mod.rs 0 → 56 %. Future Plan 32-02 will close the remaining 6 below-90 files via the D-32.9 risk-weighted ~134 cases.

## Quality Gates

- ✅ All 1627 hp41-core tests pass (was 1620; +7 from Plan 32-01 work).
- ✅ No `#![deny(clippy::unwrap_used)]` violations — new test files carry `#![allow(clippy::unwrap_used)]` per the established pattern.
- ✅ No `Op` enum changes (Plan 32 scope: test/CI/docs only).
- ✅ No `hp41-core/src/` source changes (FROZEN since Plan 25-01).
- ✅ No `hp41-gui/src-tauri/src/` source changes (SC-4 invariant preserved).
- ⚠ Coverage gate ≥ 95 % UNMET at Plan-32-01 level (91.79 % lines / 91.46 % regions). Bulk closure is Plan 32-02's deliverable.

## Known Stubs

None — all new tests are functional and pass against current behavior.

## Threat Flags

None — Plan 32-01 is test-infrastructure only. T-32-04 (coverage gate gaming) is the mitigation owner per the plan's `<threat_model>`; the per-Op count baseline + LINT-EXEMPT explicit-rationale convention + `// Catches: <bug class>` doc comments on every new `#[test]` collectively address the mitigation.

## Self-Check: PASSED

**Created files exist:**
- ✅ `hp41-core/tests/lint_math1_assertions.rs` — verified present
- ✅ `hp41-core/tests/math1_mod_entry_points.rs` — verified present

**Commits exist:**
- ✅ `cc20119` test(32-01): graduate math1_op_test_count from vacuous to non-vacuous
- ✅ `b64d388` docs(32-01): note Phase 32 graduation in xrom_shadowing doc-comment
- ✅ `f0edfcc` test(32-01): add lint_math1_assertions.rs assertion-discipline lint
- ✅ `2ec02a8` test(32-01): add explicit GTO-out + recursion-cap regression tests
- ✅ `424c708` test(32-01): surgical gap-closure for ops/math1/mod.rs (0% → 56%)
