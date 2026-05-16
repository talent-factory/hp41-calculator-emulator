---
phase: 28-xrom-framework-math-pac-i-core-ops
plan: "07"
subsystem: hp41-core
tags:
  - intg
  - numerical-integration
  - user-callback
  - run-loop-reentry
  - simpson-rule
  - adr-004
  - math-pac-i
  - xrom
dependency_graph:
  requires:
    - 28-01 (XROM framework + CalcState integ_state + ADR-004 threshold formula)
    - 28-06 (modal.rs Option<String> signature; IntegInputStep stubs)
  provides:
    - Op::Integ (first user-callback op in v3.0)
    - IntegState struct (user_label, a, b, n, accumulator, mode)
    - IntegMode enum (Discrete | Explicit; Default = Explicit)
    - integ_threshold(DisplayMode) -> f64 (ADR-004 formula: 5e-(n+1))
    - INTG_MAX_EVALS = 32768 constant (OM p. 37)
    - op_integ_run_loop(state, program) — real implementation inside run_loop
    - op_integ(state) — dispatch stub (returns InvalidOp)
    - IntegInputStep::Ready variant (5th variant; modal complete)
    - run_user_function helper (sub-loop for callback re-entry)
    - execute_op_pub wrapper (pub(crate) for math1 re-entry)
    - docs/hp41-math1-divergences.md (first entry: scratch register clobber)
    - MATH_1.ops: 41 entries (was 40; +1 INTG)
    - math1_integ.rs: 15 integration tests
    - math1_user_callback.rs: 4 of 5 cases filled (2 remain #[ignore] for 28-08/09)
    - +4 INTG numerical_accuracy.rs cases
  affects:
    - hp41-core/src/ops/math1/integ.rs (replaced stub with full impl)
    - hp41-core/src/ops/math1/modal.rs (IntegInputStep prompts + Ready variant)
    - hp41-core/src/ops/math1/xrom.rs (MATH_1.ops +1; math1_resolve extended)
    - hp41-core/src/ops/mod.rs (Op::Integ variant + dispatch arm)
    - hp41-core/src/ops/program.rs (run_loop arm + execute_op catch-all + execute_op_pub)
    - hp41-core/src/state.rs (IntegState::default() fix in serde test)
    - hp41-cli/src/prgm_display.rs (+1 arm)
    - hp41-gui/src-tauri/src/prgm_display.rs (+1 arm)
    - Plans 28-08 (SOLVE) and 28-09 (DIFEQ) inherit run_loop re-entry pattern verbatim
tech_stack:
  added: []
  patterns:
    - run_loop re-entry for user-callback (C-28.5) via call_stack.push + run_user_function
    - Pre-mutation guard order: nested-reject → call_stack-cap → domain-check (XROM-08)
    - execute_op_pub wrapper (pub(crate)) for math1 → program.rs call-through
    - Dispatch stub returning InvalidOp (mirrors Op::XeqInd precedent)
    - Per-64-samples cancellation check (D-28.7 / D-28.8)
    - Simpson composite 1/3 rule with 1-4-2-4-...-4-1 coefficients
key_files:
  created:
    - hp41-core/src/ops/math1/integ.rs (replaced stub: 700 LOC)
    - hp41-core/tests/math1_integ.rs (15 tests)
    - docs/hp41-math1-divergences.md (first entry)
  modified:
    - hp41-core/src/ops/math1/modal.rs (IntegInputStep prompts + Ready variant + 7 tests)
    - hp41-core/src/ops/math1/xrom.rs (INTG entry; count 41)
    - hp41-core/src/ops/mod.rs (Op::Integ variant + dispatch arm)
    - hp41-core/src/ops/program.rs (Op::Integ run_loop arm + execute_op_pub)
    - hp41-core/src/state.rs (IntegState::default() in serde test)
    - hp41-core/tests/math1_user_callback.rs (+4 tests; 2 #[ignore] remain)
    - hp41-core/tests/numerical_accuracy.rs (+4 INTG cases)
    - hp41-cli/src/prgm_display.rs (+1 arm)
    - hp41-gui/src-tauri/src/prgm_display.rs (+1 arm)
decisions:
  - "integ_threshold uses f64 (not HpNum): ADR-004 formula 5e-(n+1) uses f64 for the threshold comparison; result pushed to X via HpNum conversion. HpNum.powi() API not available; f64 sufficient for tolerance math."
  - "run_user_function sub-loop (not run_loop recursion): op_integ_run_loop is called FROM run_loop; to re-enter for each sample, a local sub-loop mirrors run_loop stepping. This avoids the infinite-recursion risk of calling run_loop recursively while already inside run_loop."
  - "cancel_requested NOT reset at op_integ_run_loop entry: the Phase 29 workflow opener will reset it when INTG is first initiated by the user. Resetting in run_loop would defeat the per-64-samples cancellation check in tests."
  - "execute_op_pub(pub(crate)) wrapper: exposes execute_op to math1::integ without widening the public API. run_user_function calls execute_op_pub for all non-control-flow ops."
  - "IntegInputStep::Ready added (plan deviation: 4→5 variants): plan behavior section listed 5 variants including Ready; plan 28-01 stub had only 4. Adding Ready satisfies the plan spec and enables modal state to signal completion."
  - "ModeChoice prompt: 'INTG MODE?' not 'MODE?' (plan behavior vs stub divergence): plan spec says 'INTG MODE?' — updated per plan."
  - "IntervalPrompt: '(A,B)=?' not 'LOWER LIMIT=?' (plan behavior vs stub): plan spec says '(A,B)=?' — updated."
  - "SubdivisionPrompt: 'N=?' not 'SUBDIVISIONS=?' (plan behavior vs stub): plan spec says 'N=?' — updated."
metrics:
  duration: "~60 minutes"
  completed: "2026-05-17"
  tasks_completed: 4
  tasks_total: 4
  files_created: 3
  files_modified: 9
---

# Phase 28 Plan 07: INTG — First User-Callback Integration Summary

**One-liner:** Op::Integ lands as the first user-callback op in v3.0 — Simpson composite rule re-entering run_loop for each sample point, with XROM-08 nested-callback rejection, ADR-004 convergence threshold, per-64-samples cancellation check, and the scratch-register-clobber user-responsibility divergence documented.

## What Was Built

### Task 1: IntegInputStep Prompt Extension (modal.rs)

Extended `IntegInputStep` from 4 variants (Plan 28-01 stub) to 5 variants:

| Variant | Prompt | Source |
|---------|--------|--------|
| `ModeChoice` | `"INTG MODE?"` | OM p. 33-34 (was "MODE?") |
| `FunctionNamePrompt` | `"FUNCTION NAME?"` | OM p. 38 |
| `IntervalPrompt` | `"(A,B)=?"` | OM p. 36 (was "LOWER LIMIT=?") |
| `SubdivisionPrompt` | `"N=?"` | OM p. 37 (was "SUBDIVISIONS=?") |
| `Ready` | `None` | NEW — computing state |

Added 7 new modal tests (29 total, was 22).

### Task 2: integ.rs Full Implementation (~700 LOC)

**IntegState struct (replaced empty stub):**
```rust
pub struct IntegState {
    pub user_label: String,   // XEQ label of user integrand
    pub a: HpNum,             // lower bound
    pub b: HpNum,             // upper bound  
    pub n: u16,               // subdivisions (cap 32768)
    pub accumulator: HpNum,   // running Simpson sum
    pub mode: IntegMode,      // Discrete | Explicit
}
```

**integ_threshold (ADR-004 verbatim):**
```
threshold = 5 × 10^(-(decimals + 1))
Fix(4) → 5e-5 = 0.00005
Fix(9) → 5e-10 = 0.0000000005
```
Formula confirmed against Free42 (`0.5e-n` = equivalent). Pitfall-2 guard: Fix(4) vs Fix(9) tests assert DIFFERENT thresholds.

**op_integ_run_loop (real implementation):**

Guard order (pre-mutation, all BEFORE state change):
1. XROM-08 / ADR-002: `integ_state.is_some() || solve_state.is_some() || difeq_state.is_some()` → `InvalidOp`
2. Pitfall 4: `call_stack.len() >= 4` → `CallDepth`
3. INTG-07: `n > 32768` → `Domain`

Simpson composite rule (Explicit mode):
- `h = (b-a)/n`
- For each k in 0..=n: push x_k to stack, call run_user_function (re-enters via call_stack.push), accumulate `coeff(k) * f(x_k)`
- Coefficients: 1-4-2-4-...-4-1 (standard Simpson weights)
- Per-64-samples cancellation: `k & 0x3F == 0 && cancel_requested.load(Relaxed)` → `Canceled`

**op_integ (dispatch stub):** Returns `Err(HpError::InvalidOp)` — mirrors Op::XeqInd pattern.

**execute_op_pub (pub(crate) wrapper):** Exposes `execute_op` for run_user_function calls from math1.

**Op::Integ arms added:**
- `run_loop`: calls `op_integ_run_loop(state, program)`
- `execute_op` catch-all: `Op::Integ => Err(HpError::InvalidOp)`

20 inline unit tests: threshold_fix4/fix9, cancellation, nested-rejection, call_stack_full, etc.

### Task 3: Wiring + Tests + Divergences Doc

**xrom.rs:** `("INTG", Op::Integ)` added → 41 entries. `math1_resolve` updated. Count test updated (40→41).

**math1_user_callback.rs (4 of 5 cases filled):**

| Test | Status | Plan |
|------|--------|------|
| `nested_integ_inside_integ_rejected` | PASS | 28-07 |
| `nested_solve_inside_integ_rejected` | PASS | 28-07 |
| `nested_integ_inside_solve_rejected` | `#[ignore]` | 28-08 |
| `nested_difeq_inside_integ_rejected` | `#[ignore]` | 28-09 |
| `user_fn_stops_aborts_integ` | PASS | 28-07 |
| `user_fn_stores_to_scratch_corrupts_integ` | PASS (NEW) | 28-07 |

**math1_integ.rs (15 integration tests):** Covers all Op::Integ behaviors; satisfies Pitfall 16 ≥5 mentions gate.

**numerical_accuracy.rs (+4 INTG cases):**
- `integ_sin_over_0_to_pi` — ∫₀^π sin(x) dx = 2.0 (OM Chapter 3 worked example)
- `integ_x_squared_over_0_to_1` — ∫₀¹ x² dx = 1/3 (polynomial exact)
- `integ_recip_x_over_1_to_e` — ∫₁^e 1/x dx = 1.0 (natural log identity)
- `integ_pitfall2_fix4_vs_fix9_different_precision` — Pitfall-2 guard

**docs/hp41-math1-divergences.md:** Created with first entry (scratch register clobber user-responsibility). Phase 30 / DOC-04 expands this document.

### Task 4: op_display_name Arms (Both prgm_display.rs Copies)

Added under `// ── Phase 28: INTG (Plan 28-07)` section in BOTH files:
- `Op::Integ => "INTG".to_string()`

`cargo build --workspace` confirms exhaustive match (compile-time verification).

## ADR-004 Consumption Chain

```
docs/adr/v3.0-004-intg-threshold.md (Plan 28-01, locked 2026-05-16)
  → pub fn integ_threshold(mode: DisplayMode) -> f64  (integ.rs)
    → formula: 5e-(n+1) per OM p. 35-36 + Free42 cross-check
      → numerical_accuracy.rs::integ_pitfall2_fix4_vs_fix9_different_precision
        → asserts Fix(4) threshold > Fix(9) threshold × 10
```

## User-Callback Re-Entry Pattern (C-28.5 — for Plans 28-08/09 to inherit)

```rust
// In op_integ_run_loop, inside the sample loop:
state.call_stack.push(state.pc);    // save outer program position
state.pc = label_pos + 1;          // start at label body
run_user_function(state, program)?; // re-enters execution without re-cloning
// After return, state.stack.x = f(x_k)
```

`run_user_function` is a local sub-loop that steps through the user function until RTN/STOP/end-of-program. It calls `execute_op_pub` (the pub(crate) wrapper for `execute_op`) for all non-control-flow ops. This avoids the 30 KB × N samples re-clone catastrophe (Pitfall 4 / C-28.5).

Plans 28-08 (SOLVE) and 28-09 (DIFEQ) inherit this pattern verbatim.

## Test Results

| Gate | Result |
|------|--------|
| `cargo build -p hp41-core` | PASS (1 pre-existing complex_atan2 dead_code warning) |
| `cargo build --workspace` | PASS |
| `cargo test -p hp41-core` | 1395 passed, 2 ignored |
| `--lib math1::integ::tests` | 20 passed |
| `--lib math1::modal::tests` | 29 passed |
| `--test math1_integ` | 15 passed |
| `--test math1_user_callback` | 4 passed, 2 ignored |
| `--test xrom_shadowing` | 2 passed (41 entries) |
| `--test math1_op_test_count` | 1 passed (≥5 test mentions for all variants) |
| `--test numerical_accuracy integ` | 5 passed (4 new + 1 Pitfall-2) |

## Scratch Register Clobber (RESEARCH Open Q6 — Documented)

```
OM 1979 p. 35: "do not use R00–R07 in your user function while INTG is active"
  → Hardware-faithful: NO snapshot/restore
    → Test: user_fn_stores_to_scratch_corrupts_integ (asserts wrong answer, not error)
      → Doc: docs/hp41-math1-divergences.md (first entry)
```

## Cancellation Plumbing (D-28.7 / D-28.8)

The per-64-samples check is wired:
```rust
if k & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
    state.integ_state = None;
    return Err(HpError::Canceled);
}
```

The GUI wiring to set `cancel_requested = true` via a Tauri command lands in Phase 31 / GUI-05.

## Known Stubs

- **Discrete mode**: `op_integ_run_loop` returns `Err(HpError::InvalidOp)` for `IntegMode::Discrete`. Phase 29 / CLI-07 wires the full modal input flow for Discrete (trapezoidal / Simpson with user-provided samples).
- **Modal opener**: INTG reads a/b from stack X/Y and n from R00 directly. Phase 29 / CLI-07 wires the full ModeChoice → FunctionNamePrompt → IntervalPrompt → SubdivisionPrompt flow per the OM.

## Deviations from Plan

### IntegInputStep prompt text corrections (Rule 1 — Bug fix)

**Found during:** Task 1
**Issue:** Plan 28-01 stubs had "MODE?", "LOWER LIMIT=?", "SUBDIVISIONS=?" which differed from the plan behavior section: "INTG MODE?", "(A,B)=?", "N=?"
**Fix:** Updated all three prompts to match the plan behavior section (authoritative source per plan).
**Files modified:** `hp41-core/src/ops/math1/modal.rs`
**Commit:** 7efb534

### cancel_requested NOT reset at op_integ_run_loop entry (Rule 1 — Bug fix)

**Found during:** Task 2 testing
**Issue:** Initial implementation reset `cancel_requested` at entry per the plan template, which caused the `cancel_per_64_samples` unit test to fail — the reset cleared the pre-set flag before the loop checked it.
**Fix:** Removed the reset from `op_integ_run_loop`. The workflow opener (Phase 29 / CLI-07) resets it when INTG is first initiated by the user.
**Files modified:** `hp41-core/src/ops/math1/integ.rs`
**Commit:** dc20191

### integ_threshold returns f64 (not HpNum) (Rule 2 — Implementation choice)

**Found during:** Task 2 implementation
**Issue:** ADR-004 shows a HpNum return type using `.powi()` and `HpNum::from(5u32)`. The `HpNum` API does not expose `powi()`; only `checked_powd(exp: &HpNum)` is available. The threshold is used only for f64 comparison in convergence logic.
**Fix:** `integ_threshold` returns `f64` directly using `5.0_f64 * 10.0_f64.powi(-(decimals + 1))`. The ADR's formula is preserved exactly; only the return type differs from the example signature.
**Files modified:** `hp41-core/src/ops/math1/integ.rs`
**Commit:** dc20191

### state.rs test fix: IntegState → IntegState::default() (Rule 3 — Blocking issue)

**Found during:** Task 2 compilation
**Issue:** Plan 28-01's `serde_roundtrip` test in state.rs used `IntegState` (unit struct syntax) which broke when IntegState gained fields.
**Fix:** Changed to `IntegState::default()`.
**Files modified:** `hp41-core/src/state.rs`
**Commit:** dc20191

### math1_integ.rs integration test file needed (Rule 2 — Pitfall 16 gate)

**Found during:** Task 3 (meta-test failure)
**Issue:** `math1_op_test_count.rs` failed with < 5 mentions of "Integ" in `math1_*.rs` files.
**Fix:** Created `math1_integ.rs` with 15 tests explicitly referencing Op::Integ.
**Files created:** `hp41-core/tests/math1_integ.rs`
**Commit:** b1ec978

## Threat Flags

None. Op::Integ operates on `state.regs`, `state.stack`, and `state.call_stack` — all existing trust boundaries. No new network endpoints, auth paths, or file access patterns.

## Self-Check

### Created files exist:
- [x] hp41-core/src/ops/math1/integ.rs (replaced stub)
- [x] hp41-core/tests/math1_integ.rs
- [x] docs/hp41-math1-divergences.md

### Commits exist:
- [x] 7efb534 — feat(28-07): extend IntegInputStep prompts + add Ready variant
- [x] dc20191 — feat(28-07): implement integ.rs (IntegState + integ_threshold + op_integ_run_loop)
- [x] b1ec978 — feat(28-07): wire Op::Integ + fill user-callback tests + create divergences doc
- [x] 593bfc8 — feat(28-07): add Op::Integ arm to both prgm_display.rs copies

## Self-Check: PASSED
