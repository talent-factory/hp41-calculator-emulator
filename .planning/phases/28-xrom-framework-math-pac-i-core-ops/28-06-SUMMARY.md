---
phase: 28-xrom-framework-math-pac-i-core-ops
plan: "06"
subsystem: hp41-core
tags:
  - matrix
  - modal-workflow
  - gauss-jordan
  - lu-decomposition
  - math-pac-i
  - xrom
  - inv-epsilon
dependency_graph:
  requires:
    - 28-01 (XROM framework + ModalProgram + CalcState matrix_dim/matrix_active_reg/modal_program/modal_prompt)
    - 28-05 (proven modal-workflow pattern — PolyWorkflow/Roots as template)
  provides:
    - Op::MatrixWorkflow (master entry — opens ORDER=? modal)
    - Op::MatSize (returns matrix order from R14 to X)
    - Op::MatVmat (column-major display via print_buffer)
    - Op::MatEdit (ROW↑COL=? edit-mode modal)
    - Op::MatDet (LU det with partial pivoting)
    - Op::MatInv (Gauss-Jordan inverse; INV_EPSILON ADR-003 singularity gate)
    - Op::MatSimeq (Gauss elimination solver; flag 5 on success)
    - Op::MatVcol (B-vector display via print_buffer)
    - hp41-core/src/ops/math1/matrix.rs (600+ LOC including tests)
    - INV_EPSILON = 1e-10 consumed verbatim from ADR-003 (Plan 28-01)
    - MatrixInputStep::current_prompt() -> Option<String> (dynamic element prompts)
    - MATH_1.ops: 40 entries (was 32; +8 MATRIX mnemonics)
    - math1_matrix_flow.rs: 9 integration tests (full dispatch + flag 5 + NO SOLUTION)
    - math1_matrix.rs: 42 variant-coverage tests (Pitfall 16 gate satisfied)
    - 5 numerical_accuracy.rs cases (OM Chapter 3 + Free42 v3.0.5 citations)
  affects:
    - hp41-core/src/ops/math1/modal.rs (current_prompt broadened to Option<String>)
    - hp41-core/src/ops/math1/matrix.rs (created)
    - hp41-core/src/ops/math1/mod.rs (pub mod matrix added)
    - hp41-core/src/ops/math1/xrom.rs (MATH_1.ops + math1_resolve extended to 40)
    - hp41-core/src/ops/mod.rs (Op enum + dispatch + imports)
    - hp41-core/src/ops/program.rs (execute_op arms)
    - hp41-core/tests/numerical_accuracy.rs (+5 MATRIX cases)
    - hp41-core/tests/math1_matrix_flow.rs (created)
    - hp41-core/tests/math1_matrix.rs (created)
    - hp41-cli/src/prgm_display.rs (+8 arms)
    - hp41-gui/src-tauri/src/prgm_display.rs (+8 arms)
    - Plans 28-07..28-10 (inherit integration-test pattern from math1_matrix_flow.rs)
tech_stack:
  added: []
  patterns:
    - LU decomposition with partial pivoting (column-major access) for determinant
    - Gauss-Jordan on augmented [A|I] for inversion (INV_EPSILON singularity gate)
    - Gauss elimination on [A|b] for SIMEQ solution (stores at R(N+1)..R(2N))
    - Column-major matrix storage: A(r,c) at regs[base + c*rows + r]
    - NO SOLUTION modal_prompt (not HpError) for singular matrices per OM p.23
    - Flag 5 set on successful SIMEQ (MAT-11)
    - option<String> return type for current_prompt() (owned strings for dynamic element prompts)
key_files:
  created:
    - hp41-core/src/ops/math1/matrix.rs
    - hp41-core/tests/math1_matrix_flow.rs
    - hp41-core/tests/math1_matrix.rs
  modified:
    - hp41-core/src/ops/math1/modal.rs
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/numerical_accuracy.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
decisions:
  - "Option<String> signature broadening: current_prompt() returns owned String to support dynamic ElementPrompt ('A1,1=?') and SimeqInputPrompt ('B3=?') — pre-allocating 14×14 const table rejected per plan note (b)"
  - "NO SOLUTION as modal_prompt (not HpError): singular INV/SIMEQ sets state.modal_prompt = Some('NO SOLUTION') and returns Ok(()) per OM p.23/28 behavior"
  - "INV_EPSILON = 1e-10 verbatim from ADR-003 (Plan 28-01 OM transcription); more conservative than Free42 5e-10 heuristic"
  - "Column-major storage: A(r,c) at regs[base + c*n + r]; base=15 (DEFAULT_MATRIX_BASE_REG)"
  - "ORDER cap: MAX_ORDER=14; > 14 returns HpError::OutOfRange (MAT-09)"
  - "SIMEQ immediately solves using existing B-vector in R(N+1)..R(2N); Phase 29/31 wires R/S-submit modal advancement"
  - "math1_matrix.rs created (42 tests) to satisfy Pitfall 16 ≥5 test mentions per variant"
metrics:
  duration: "~45 minutes"
  completed: "2026-05-17"
  tasks_completed: 4
  tasks_total: 4
  files_created: 3
  files_modified: 8
---

# Phase 28 Plan 06: MATRIX — Column-Major Matrix Operations Summary

**One-liner:** MATRIX (modal master entry) + SIZE/VMAT/EDIT/DET/INV/SIMEQ/VCOL land with LU-partial-pivot determinant, Gauss-Jordan inversion using ADR-003 INV_EPSILON=1e-10, and SIMEQ Gauss-elimination solver with flag-5 side effect — the largest single Math Pac I workflow.

## What Was Built

### Task 1: MatrixInputStep Prompt Extension (modal.rs)

Broadened `ModalProgram::current_prompt()` and all per-step implementations from `Option<&'static str>` to `Option<String>` to support dynamic matrix element prompts:

| Variant | Prompt |
|---------|--------|
| `Matrix(OrderPrompt)` | `"ORDER=?"` |
| `Matrix(ElementPrompt(r, c))` | `"A{r+1},{c+1}=?"` (1-indexed, OM-faithful) |
| `Matrix(Ready)` | `None` |
| `Matrix(EditPrompt)` | `"ROW↑COL=?"` (Unicode ↑ U+2191) |
| `Matrix(SimeqInputPrompt(idx))` | `"B{idx+1}=?"` (1-indexed) |
| `Matrix(SimeqDone)` | `None` |

All existing callers updated; 7 new Matrix-specific tests added (22 total modal tests).

**Schema revision:** The `Option<String>` return type is a Plan 28-06 schema revision to modal.rs. Pre-allocating a 14×14 const table for ElementPrompt was rejected — 196 strings for matrix element prompts alone. This is documented in the plan as deviation option (a).

### Task 2: matrix.rs Implementation (~600 LOC)

Created `hp41-core/src/ops/math1/matrix.rs` with:

**INV_EPSILON constant (ADR-003):**
```rust
pub const INV_EPSILON: f64 = 1e-10;
```
Source: HP Museum ROM-observation (hardware ground truth). OM provides no numeric threshold.
More conservative than Free42's 5e-10 heuristic. ADR-003 (Plan 28-01) locks this value.

**Column-major storage helpers:**
- `matrix_get(state, r, c)` → reads `regs[base + c*rows + r]`
- `matrix_set(state, r, c, val)` → writes `regs[base + c*rows + r]`
- `DEFAULT_MATRIX_BASE_REG = 15` (first element A(0,0) at R15)
- `ORDER_REG = 14` (R14 stores ORDER N per OM Chapter 3)

**Numerical kernels:**
- `lu_det(state, n)` → LU decomposition with partial pivoting; det = product of diagonal pivots × pivot-sign
- `gauss_jordan_inv(state, n)` → augmented [A|I] with Gauss-Jordan; |pivot| < INV_EPSILON → `Err(Domain)` (caller converts to modal_prompt)
- `gauss_solve(state, n, b)` → forward-elimination on [A|b] with back-substitution; |pivot| < INV_EPSILON → `Err(Domain)`

**8 op functions:**

| Function | LiftEffect | Notes |
|----------|-----------|-------|
| `op_matrix_workflow` | Neutral | Opens Matrix(OrderPrompt); sets matrix_active_reg=15 |
| `op_mat_size` | Enable | Pushes R14 (ORDER) to X |
| `op_mat_vmat` | Neutral | Column-major display "A{r},{c}={val}" |
| `op_mat_edit` | Neutral | Opens Matrix(EditPrompt); modal_prompt="ROW↑COL=?" |
| `op_mat_det` | Enable | LU det; pushes to X; ORDER>14 → OutOfRange |
| `op_mat_inv` | Neutral | Gauss-Jordan; singular → modal_prompt="NO SOLUTION" |
| `op_mat_simeq` | Neutral | Gauss elim; solution at R(N+1)..R(2N); sets flag 5 |
| `op_mat_vcol` | Neutral | Displays B-vector "B{n}={val}" |

**19 inline unit tests:** column_major, order_cap (14 accepted, 15 rejected), inv_singular, inv_near_singular, inv_well_conditioned, inv_back_sub_extreme, det_3x3_singular, det_3x3_nonsingular, simeq_solves, simeq_flag5, simeq_singular, lift_effect assertions.

### Task 3: Op Enum + Dispatch Chain + MATH_1 Registry + Test Suite

**Op enum** (`ops/mod.rs`): 8 new variants: `MatrixWorkflow`, `MatSize`, `MatVmat`, `MatEdit`, `MatDet`, `MatInv`, `MatSimeq`, `MatVcol`.

**dispatch()** + **execute_op()**: 8 new arms each.

**MATH_1.ops** (`ops/math1/xrom.rs`):
- 8 new entries: MATRIX / SIZE / VMAT / EDIT / DET / INV / SIMEQ / VCOL
- Entry count: 32 → 40
- `"INV"` mnemonic confirmed non-shadowing: `builtin_card_op` registers no `"INV"` string (Op::Inv reciprocal uses `"1/x"` display name)

**math1_matrix_flow.rs** (9 integration tests):
- `matrix_workflow_dispatches_via_xeq` — XEQ "MATRIX" → OrderPrompt
- `matrix_det_dispatches_via_xeq` — XEQ "DET" on identity
- `matrix_inv_dispatches_via_xeq` — XEQ "INV" on well-conditioned
- `matrix_size_dispatches_via_xeq` — XEQ "SIZE" → R14
- `matrix_simeq_flag5` — XEQ "SIMEQ" → flag 5 set
- `matrix_inv_singular_no_solution` — XEQ "INV" on singular
- `matrix_vmat_column_major_display` — VMAT column-major order
- `matrix_edit_opens_modal` — EDIT opens EditPrompt
- `matrix_vcol_displays_b_vector` — VCOL displays B-vector

**math1_matrix.rs** (42 variant-coverage tests):
- ≥6 tests per variant (MatrixWorkflow, MatSize, MatVmat, MatEdit, MatDet, MatInv, MatSimeq, MatVcol)
- Satisfies Pitfall 16 gate (≥5 per variant)

**numerical_accuracy.rs** (+5 cases):
- `matrix_det_identity_2x2` — det(I₂) = 1
- `matrix_det_2x2_known_value` — det([[3,8],[4,6]]) = -14
- `matrix_inv_round_trip_2x2` — inv([[2,1],[1,2]])(0,0) ≈ 2/3
- `matrix_simeq_exact_solution` — [[2,1],[1,3]]·[x,y]=[5,10] → x=1, y=3
- `matrix_singular_detection_at_inv_epsilon` — pivot << 1e-10 → NO SOLUTION

### Task 4: prgm_display.rs Exhaustive Match Arms (Both Copies)

Added under `// ── Phase 28: MATRIX (Plan 28-06)` section header in BOTH files:
- `Op::MatrixWorkflow => "MATRIX".to_string()`
- `Op::MatSize => "SIZE".to_string()`
- `Op::MatVmat => "VMAT".to_string()`
- `Op::MatEdit => "EDIT".to_string()`
- `Op::MatDet => "DET".to_string()`
- `Op::MatInv => "INV".to_string()`
- `Op::MatSimeq => "SIMEQ".to_string()`
- `Op::MatVcol => "VCOL".to_string()`

`cargo build --workspace` confirms exhaustive match (compile-time verification).

## Test Results

| Gate | Result |
|------|--------|
| `cargo build -p hp41-core` | PASS (1 pre-existing warning: complex_atan2 dead_code) |
| `cargo build --workspace` | PASS |
| `cargo test -p hp41-core` | 1345 passed, 5 ignored |
| `--lib math1::modal::tests` | 22 passed |
| `--lib math1::matrix::tests` | 19 passed |
| `--test math1_matrix_flow` | 9 passed |
| `--test math1_matrix` | 42 passed |
| `--test math1_op_test_count` | 1 passed (≥5 per variant) |
| `--test xrom_shadowing` | 2 passed (40 mnemonics, non-shadowing) |
| `--test numerical_accuracy` | (part of 1345 total) |

## INV_EPSILON Flow (ADR-003)

```
docs/adr/v3.0-003-inv-epsilon.md
  → pub const INV_EPSILON: f64 = 1e-10;  (matrix.rs)
    → gauss_jordan_inv(): if pivot.abs() < INV_EPSILON → Err(Domain)
      → op_mat_inv(): Err(Domain) → state.modal_prompt = Some("NO SOLUTION")
        → math1_matrix_flow.rs: matrix_inv_singular_no_solution asserts this
          → numerical_accuracy.rs: matrix_singular_detection_at_inv_epsilon confirms
```

## Column-Major Storage (MAT-02)

```
Matrix A: n×n; base register = 15 (DEFAULT_MATRIX_BASE_REG)
A(r, c) → regs[15 + c * n + r]

Example: 2×2 [[1,2],[3,4]] (row-major input)
  A(0,0)=1 → regs[15 + 0*2 + 0] = regs[15]
  A(1,0)=3 → regs[15 + 0*2 + 1] = regs[16]
  A(0,1)=2 → regs[15 + 1*2 + 0] = regs[17]
  A(1,1)=4 → regs[15 + 1*2 + 1] = regs[18]

B vector (SIMEQ): R(15 + n*n) .. R(15 + n*n + n - 1)
```

## Deviation from Plan

### Option<String> signature broadening (Task 1 — Plan-documented schema revision)

**Found during:** Task 1 design review
**Issue:** `ElementPrompt(r, c)` requires dynamic strings "A1,1=?" through "A14,14=?" — not representable as `&'static str`. SimeqInputPrompt(idx) similarly dynamic.
**Fix:** Broadened `current_prompt()` from `Option<&'static str>` to `Option<String>`. All callers updated (test assertions use `.to_string()`). Plan 28-06 Task 1 explicitly documents this as option (a) — chosen approach.
**Files modified:** `hp41-core/src/ops/math1/modal.rs`
**Commit:** d05ba25

### math1_matrix.rs integration test file needed (Rule 2 — missing critical test coverage)

**Found during:** Task 3 (Pitfall 16 meta-test failure)
**Issue:** The `math1_op_test_count.rs` gate counts variant name mentions in `math1_*.rs` files. The integration test `math1_matrix_flow.rs` uses `Op::Xeq("MATRIX")` (string dispatch), not `Op::MatrixWorkflow` directly. The unit tests in matrix.rs are in-crate (`#[cfg(test)]` scope) and not scanned by the meta-test.
**Fix:** Created `math1_matrix.rs` (42 tests explicitly mentioning each variant name ≥6 times).
**Files modified:** `hp41-core/tests/math1_matrix.rs` (new)
**Commit:** 10457a0

### setup_matrix is cfg(test)-gated — integration tests needed inline helper (Rule 3)

**Found during:** Task 3 compilation of math1_matrix_flow.rs
**Issue:** `setup_matrix()` in matrix.rs is `#[cfg(test)]` and thus not accessible from integration test crates.
**Fix:** Each integration test file (`math1_matrix_flow.rs`, additions to `numerical_accuracy.rs`) defines its own local `mat_setup()` / `matrix_setup_acc()` helper inline.
**Files modified:** math1_matrix_flow.rs, numerical_accuracy.rs

## Known Stubs

- `MatEdit` / `MatVmat` / `MatVcol`: fully implemented for core dispatch; R/S-submit wiring for the interactive element-edit flow is Phase 29 (CLI-05) + Phase 31 (GUI-06) scope per D-28.5.
- `op_mat_simeq` opens `SimeqInputPrompt(0)` modal but immediately computes with existing B-vector contents. The Phase 29/31 modal-step-advance wiring (user enters B1..BN via keyboard, R/S advances) is deferred. The core solve path is fully functional.

## Threat Flags

None. Matrix operations read/write `state.regs` (existing trust boundary, already in scope). No new network endpoints, auth paths, or file access patterns introduced.

## math1_matrix_flow.rs as Integration-Test Template

Plans 28-07 (INTG), 28-08 (SOLVE), 28-09 (DIFEQ), and 28-10 (FOUR/TRANS) inherit the `math1_matrix_flow.rs` pattern:
1. Dispatch via `Op::Xeq("MNEMONIC")` through the resolver chain
2. Assert modal state machine transitions
3. Assert flag side effects
4. Assert NO SOLUTION / error behavior on edge inputs

## Self-Check

### Created files exist:
- [x] hp41-core/src/ops/math1/matrix.rs
- [x] hp41-core/tests/math1_matrix_flow.rs
- [x] hp41-core/tests/math1_matrix.rs

### Commits exist:
- [x] d05ba25 — feat(28-06): extend MatrixInputStep prompts + Option<String> signature broadening
- [x] c83872a — feat(28-06): implement matrix.rs (8 op fns + LU det + Gauss-Jordan inv + SIMEQ)
- [x] 10457a0 — feat(28-06): wire Op::Matrix* through dispatch chain + xrom_resolve + integration tests
- [x] b3c3ff9 — feat(28-06): add MATRIX op_display_name arms to both prgm_display.rs copies

## Self-Check: PASSED
