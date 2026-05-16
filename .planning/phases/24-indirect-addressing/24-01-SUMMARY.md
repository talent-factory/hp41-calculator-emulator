---
phase: 24-indirect-addressing
plan: 01
subsystem: hp41-core / indirect-addressing
tags:
  - rust
  - hp41-core
  - refactor
  - indirect-addressing
  - tdd

dependency_graph:
  requires:
    - Phase 22 (Op::GtoInd / Op::XeqInd inline trunc-and-validate already shipped)
  provides:
    - "crate::ops::indirect::resolve_indirect_decimal (private — single source of pointer-validation truth, D-24.1)"
    - "crate::ops::indirect::resolve_indirect (public — u8-wrapper, D-24.2)"
  affects:
    - "hp41-core/src/ops/program.rs (Op::GtoInd and Op::XeqInd run_loop arms refactored)"
    - "hp41-core/src/ops/mod.rs (`pub mod indirect;` module wiring)"

tech_stack:
  added: []
  patterns:
    - "two-tier resolver (private inner + public u8 wrapper)"
    - "characterization-test refactor (lock behavior with sentinels, then refactor)"
    - "delegation-over-duplication (Phase 22's Op::GtoInd / Op::XeqInd consume the shared helper)"

key_files:
  created:
    - hp41-core/src/ops/indirect.rs
    - hp41-core/tests/phase24_resolve_indirect.rs
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/phase22_program_control.rs

decisions:
  - "D-24.1 / D-24.2 / D-24.3 ratified by working code — two-tier resolver lives in hp41-core/src/ops/indirect.rs; resolve_indirect_decimal is private to crate, resolve_indirect is public; neither performs caller-bounds checks (regs.len() / 56)"
  - "D-24.5 ratified by working code — Phase-22 Op::GtoInd and Op::XeqInd arms now delegate to resolve_indirect_decimal; call_stack.len() >= 4 -> CallDepth pre-mutation guard remains the FIRST executable line of XeqInd arm; 4 sentinel tests prove byte-for-byte equivalence"
  - "Plan literal `*reg` corrected to `reg` (Rule 3 auto-fix) — the run_loop match binds `op` by value (clone at program.rs:450), so the variant field `reg: u8` is already a value, not a reference; the `*reg` form is a build error (E0614). Functional contract identical."

metrics:
  duration_seconds: 357
  duration_human: "~6 minutes"
  tasks_completed: 2
  files_created: 2
  files_modified: 3
  commits: 4
  tests_added: 12  # 7 inline indirect::tests + 4 sentinels in phase22_program_control.rs + 1 smoke in phase24_resolve_indirect.rs
  full_suite_tests_passing: 725  # hp41-core
  completed_date: "2026-05-14"
---

# Phase 24 Plan 01: Foundation — resolve_indirect helper + Phase-22 refactor Summary

Two-tier `resolve_indirect` family of helpers lands in `hp41-core/src/ops/indirect.rs` (private `resolve_indirect_decimal(state, reg) -> Decimal` + public `resolve_indirect(state, reg) -> u8`), and the existing Phase-22 `Op::GtoInd` / `Op::XeqInd` `run_loop` arms in `program.rs` are refactored from inline 9-line trunc-and-validate blocks onto the new shared helper — single source of pointer-validation truth (D-24.1) now in place for plan 24-02's 11 `*Ind` Op variants to delegate to.

## What Shipped

### Files Created

| Path | Purpose | Lines | Tests inside |
|------|---------|-------|--------------|
| `hp41-core/src/ops/indirect.rs` | Two-tier resolver helpers + inline unit tests (D-24.1 / D-24.2) | 138 | 7 inline `#[cfg(test)] mod tests` |
| `hp41-core/tests/phase24_resolve_indirect.rs` | Wave-1 scaffolding file — Pitfall 5 defense (public-symbol-path reachability smoke test) | 14 | 1 smoke (`resolve_indirect_is_reachable_from_integration_target`) |

### Files Modified

| Path | Delta | What changed |
|------|------:|--------------|
| `hp41-core/src/ops/mod.rs` | +1 / -0 | Added `pub mod indirect;` between `pub mod hms;` and `pub mod math;` |
| `hp41-core/src/ops/program.rs` | +14 / -40 | Replaced the inline 9-line trunc-and-validate blocks in `Op::GtoInd(reg)` and `Op::XeqInd(reg)` run_loop arms with `let i = crate::ops::indirect::resolve_indirect_decimal(state, reg)?; let label_str = i.to_string();`. Preserved the XeqInd `call_stack.len() >= 4 -> CallDepth` pre-mutation atomicity guard as the FIRST executable line of its arm. |
| `hp41-core/tests/phase22_program_control.rs` | +112 / -0 | Appended a `// ── Phase 24 D-24.5 sentinel ─────` section with 4 new regression tests. The pre-existing 15 Phase-22 tests are byte-for-byte unchanged. |

## Commits

| Order | Hash | Subject |
|-------|------|---------|
| 1 (Task 1 RED) | `6bb92d4` | test(24-01): add failing tests for resolve_indirect two-tier helper |
| 2 (Task 1 GREEN) | `dbc5699` | feat(24-01): implement resolve_indirect two-tier helper (D-24.1) |
| 3 (Task 2 characterization) | `52191bc` | test(24-01): add D-24.5 sentinel tests and Phase 24 integration scaffold |
| 4 (Task 2 REFACTOR) | `0f53597` | refactor(24-01): route Op::GtoInd / Op::XeqInd through resolve_indirect_decimal (D-24.5) |

## Tests Added (12 total)

### Inline `hp41-core/src/ops/indirect.rs::tests` (7 unit tests — all PASS)

1. `resolve_indirect_happy_integer_pointer` — R05=42 → `Ok(42u8)`
2. `resolve_indirect_non_integer_rejects` — R05=12.345 → `Err(InvalidOp)`
3. `resolve_indirect_reg_out_of_range_rejects` — reg=200 (>100) → `Err(InvalidOp)`
4. `resolve_indirect_pointer_exceeds_u8_range_rejects` — R05=300 → `Err(InvalidOp)` via `u8::try_from` arm
5. `resolve_indirect_pointer_exceeds_i64_range_rejects` — R05=2^64 → `Err(InvalidOp)` via `to_i64().ok_or` arm (Pitfall 3 — branch coverage)
6. `resolve_indirect_negative_integer_pointer_rejects_via_u8` — R05=-3 → `Err(InvalidOp)` via u8 try_from
7. `resolve_indirect_decimal_preserves_sign_for_gto_ind_callers` — inner helper returns `Decimal::to_string() == "-3"` for R05=-3 (Pitfall 2 — sign preservation across the refactor)

### `hp41-core/tests/phase22_program_control.rs` — 4 D-24.5 sentinel tests (all PASS)

1. `phase24_gto_ind_uses_shared_helper` — sanity: R05=42, GTO IND 5 → X=7 via the shared helper
2. `phase24_xeq_ind_uses_shared_helper` — sanity: R03=10, XEQ IND 3 → X=2 after RTN via the shared helper
3. **`phase24_xeq_ind_call_depth_guard_runs_before_pointer_read`** — CRITICAL: driven via `resume_program` with 4-deep `call_stack` + non-integer R03=12.345; expects `Err(CallDepth)` (NOT `Err(InvalidOp)`) and `state.call_stack.len() == 4` post-condition. Proves pre-mutation atomicity guard fires FIRST after the refactor.
4. `phase24_gto_ind_negative_pointer_stringifies_with_sign` — R05=-3 + no LBL "-3" → `Err(InvalidOp)` from `find_in_program` (NOT from non-integer rejection). Pitfall 2 sentinel.

### `hp41-core/tests/phase24_resolve_indirect.rs` — 1 smoke test (PASS)

- `resolve_indirect_is_reachable_from_integration_target` — Pitfall 5 defense: asserts the public symbol path `hp41_core::ops::indirect::resolve_indirect` resolves from outside the crate.

## Phase-22 Regression Verification (Refactor is Byte-for-Byte Equivalent)

All 5 pre-existing Phase-22 GTO/XEQ-IND tests in `hp41-core/tests/phase22_program_control.rs` continue to pass **unchanged** after the refactor (verified via `cargo test -p hp41-core --test phase22_program_control`):

| # | Pre-existing test | Result |
|---|-------------------|:------:|
| 1 | `test_gto_ind_happy` | PASS |
| 2 | `test_gto_ind_non_integer_rejects` | PASS |
| 3 | `test_gto_ind_reg_out_of_range_rejects` | PASS |
| 4 | `test_xeq_ind_happy` | PASS |
| 5 | `test_xeq_ind_4_deep_call_stack_rejects` (the D-22.15 atomicity test) | PASS |
| 6 | `test_xeq_ind_reg_out_of_range_rejects` | PASS |
| 7 | `test_xeq_ind_non_integer_rejects` | PASS |

(The plan documented "all 5 pre-existing" but the file actually contains 7 GTO/XEQ-IND tests — same code path, additional reject-path coverage. All 7 pass.)

## Verification Results

| Check | Expected | Result |
|-------|----------|--------|
| `cargo build -p hp41-core` | exits 0 | PASS |
| `cargo test -p hp41-core --lib indirect::tests` | 7 / 7 pass | PASS |
| `cargo test -p hp41-core --test phase22_program_control` | 19 / 19 pass (15 original + 4 sentinels) | PASS |
| `cargo test -p hp41-core --test phase24_resolve_indirect` | 1 / 1 pass | PASS |
| `cargo test -p hp41-core` (full suite) | all green | **725 / 725 pass** |
| `cargo clippy -p hp41-core -- -D warnings` | clean | PASS |
| `just test-core` | green | PASS |
| `grep -c "pub fn resolve_indirect" hp41-core/src/ops/indirect.rs` | ≥ 1 | 1 |
| `grep -c "pointer.trunc_int" hp41-core/src/ops/program.rs` | 0 | 0 |
| `grep -c "resolve_indirect_decimal(state" hp41-core/src/ops/program.rs` | ≥ 2 | 2 |
| `grep -A3 "Op::XeqInd(reg) =>" hp41-core/src/ops/program.rs \| grep -c "call_stack.len() >= 4"` | ≥ 1 | 1 |
| Production code `.unwrap()` / `as u8` outside `mod tests` in indirect.rs | 0 | 0 (none) |

## Pitfall Mitigations Exercised

| Pitfall | How exercised |
|---------|---------------|
| **Pitfall 2** — Decimal stringification for negative pointers | `resolve_indirect_decimal_preserves_sign_for_gto_ind_callers` (inline unit) + `phase24_gto_ind_negative_pointer_stringifies_with_sign` (integration). Both confirm `Decimal::to_string()` preserves the `-` prefix; the inner helper returns the Decimal as-is without sign-stripping. |
| **Pitfall 3** — `to_i64` overflow on values exceeding `i64::MAX` | `resolve_indirect_pointer_exceeds_i64_range_rejects` constructs `HpNum::rounded(Decimal::from_str("18446744073709551616"))` (2^64, well past i64::MAX) and asserts `InvalidOp` via the `to_i64().ok_or` branch — the OTHER cascading rejection arm in `resolve_indirect`. Branch coverage. |
| **Pitfall 5** — Forgetting `pub mod indirect;` in `ops/mod.rs` | `resolve_indirect_is_reachable_from_integration_target` in the integration test file `phase24_resolve_indirect.rs` references `hp41_core::ops::indirect::resolve_indirect` — the test compiles only if the public symbol path resolves from outside the crate. Compile-time guard. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking issue] Plan literal `*reg` argument corrected to `reg`**
- **Found during:** Task 2, first `cargo build` after refactor
- **Issue:** The plan's `must_haves.truths` and RESEARCH.md "After" code blocks both write the call as `crate::ops::indirect::resolve_indirect_decimal(state, *reg)?`. With `*reg`, `cargo build -p hp41-core` fails with `error[E0614]: type `u8` cannot be dereferenced`.
- **Root cause:** The `run_loop` match at `hp41-core/src/ops/program.rs:453` is `match op { ... }` where `op = program[state.pc].clone()` (line 450) — `op` is an OWNED `Op` value, so each variant's payload field (`reg: u8`) is bound as a `u8` value, not a reference. `*reg` only makes sense when matching by reference (`match &op { ... }`).
- **Fix:** Changed `(state, *reg)` to `(state, reg)` in both arms. Functional contract is identical (both forms pass the u8 by value); the build error was a purely syntactic artifact of the plan's pattern-binding assumption not matching the actual `match op` shape. No semantic change.
- **Files modified:** `hp41-core/src/ops/program.rs` (two call sites in `Op::GtoInd` and `Op::XeqInd` arms)
- **Commit:** Folded into `0f53597` (refactor commit); documented in commit body.

No other deviations. Plan executed as written.

## TDD Gate Compliance

Plan-level `type: execute` with task-level `tdd="true"` on both tasks.

| Task | RED commit | GREEN/REFACTOR commit | Status |
|------|------------|------------------------|--------|
| Task 1 (resolve_indirect helper) | `6bb92d4` (test: failing inline tests) | `dbc5699` (feat: implementation) | RED → GREEN gate satisfied |
| Task 2 (Phase-22 refactor) | `52191bc` (test: 4 sentinels + scaffold, characterization-test pattern — sentinels PASS against pre-refactor code, locking baseline) | `0f53597` (refactor: extract inline blocks onto shared helper; sentinels STILL pass) | RED → REFACTOR gate satisfied via the characterization-test variant of TDD (sentinels are a "lock baseline, then change implementation" pattern — not a new-behavior RED) |

Note on Task 2 TDD: the plan's intent is "behavior-preserving refactor", not "new behavior". The strict RED-must-fail gate doesn't directly apply because the behavior already exists in inline form. The characterization-test pattern (lock current behavior with sentinels, then refactor and verify same behavior) is the correct TDD interpretation for this category. All 4 new sentinels passed against the pre-refactor inline code (commit `52191bc`) AND continue to pass against the post-refactor helper-delegation code (commit `0f53597`). The 5 (actually 7) pre-existing Phase-22 GTO/XEQ-IND tests are the primary regression suite — they too pass byte-for-byte after the refactor.

## Hand-off Note for Plan 24-02

Plan 24-02 will land ~11 new `Op::*Ind` variants (`StoInd`, `RclInd`, `StoArithInd`, `IsgInd`, `DseInd`, `SfFlagInd`, `CfFlagInd`, `FlagTestInd`, `ArclInd`, `AstoInd`, `ViewInd`). Each variant's dispatch shim should be a 2-liner that consumes the public helper:

```rust
pub(crate) fn op_<name>_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = crate::ops::indirect::resolve_indirect(state, reg)?;
    op_<name>(state, addr)  // delegate to direct-form op (Phase 21/22/23 ops)
}
```

The contract:
- **`crate::ops::indirect::resolve_indirect`** is the public helper (returns `Result<u8, HpError>`); use this for register-and-flag-address consumers.
- **`crate::ops::indirect::resolve_indirect_decimal`** is `pub(crate)` (returns `Result<Decimal, HpError>`); use this only when the caller needs to stringify the integer for label lookup (the Phase-22 `Op::GtoInd` / `Op::XeqInd` pattern, already wired in this plan).
- D-24.3 / D-24.4: do NOT replicate `regs.len()` or `< 56` bounds in the IND shim — the direct-form `op_sto` / `op_sf` / etc. already enforce them via the D-22.11.1 `.get().ok_or(InvalidOp)?` pattern. Sidecar-clearing (D-23.4 — `text_regs.remove(&reg)`) and lift-effect inherit gratis via delegation.

The 11 shims may live in `hp41-core/src/ops/indirect.rs` (alongside the helpers) — the module already exists and is wired. Plan 24-02 will append per-variant integration tests to `hp41-core/tests/phase24_resolve_indirect.rs` (currently a 14-line scaffold with one smoke test).

## Self-Check: PASSED

- Files created exist:
  - `hp41-core/src/ops/indirect.rs` — FOUND
  - `hp41-core/tests/phase24_resolve_indirect.rs` — FOUND
- Files modified contain the expected markers:
  - `hp41-core/src/ops/mod.rs` contains `pub mod indirect;` — FOUND
  - `hp41-core/src/ops/program.rs` contains `resolve_indirect_decimal` (2 call sites) — FOUND
  - `hp41-core/src/ops/program.rs` no longer contains `pointer.trunc_int` — FOUND (0 matches)
  - `hp41-core/tests/phase22_program_control.rs` contains all 4 sentinel test names — FOUND
- Commits exist on `worktree-agent-a26a43bfb45cfc4f2`:
  - `6bb92d4` — FOUND
  - `dbc5699` — FOUND
  - `52191bc` — FOUND
  - `0f53597` — FOUND
- Full `cargo test -p hp41-core` suite: 725 / 725 pass
- `cargo clippy -p hp41-core -- -D warnings`: clean
