---
phase: 28-xrom-framework-math-pac-i-core-ops
plan: "10"
subsystem: hp41-core
tags:
  - four
  - fourier-analysis
  - triangle-solvers
  - sss
  - asa
  - saa
  - sas
  - ssa
  - ambiguous-case
  - trans
  - rodrigues-rotation
  - coordinate-transform
  - xrom
  - math-pac-i
dependency_graph:
  requires:
    - 28-01 (XROM framework + CalcState fields + ModalProgram stubs)
    - 28-07 (user-callback infrastructure — FOUR-06 evaluator pattern)
  provides:
    - Op::Four (XEQ "FOUR" — Fourier analysis master entry + DFT + RECT? toggle + USER-mode E-key evaluator)
    - Op::TriSss (XEQ "SSS" — Law of Cosines three-sides-to-angles)
    - Op::TriAsa (XEQ "ASA" — Law of Sines Angle-Side-Angle)
    - Op::TriSaa (XEQ "SAA" — Law of Sines Side-Angle-Angle)
    - Op::TriSas (XEQ "SAS" — Law of Cosines Side-Angle-Side)
    - Op::TriSsa (XEQ "SSA" — ambiguous case with 0/1/2 solutions per OM p.46 TRI-05)
    - Op::Trans2d (XEQ "TRANS" — 2D rotation+translation coordinate transform)
    - Op::Trans3d (XEQ "T3D" — 3D Rodrigues rotation coordinate transform)
    - four.rs: compute_dft, store_dft_to_registers, convert_to_polar, op_four_eval_at_t
    - tri.rs: 5 triangle solver fns with Law of Sines/Cosines + SSA ambiguous-case handler
    - trans.rs: rodrigues_rotate, normalize_3d, cross/dot product helpers, 2D/3D sub-ops
    - FourInputStep: 6 variants (NumSamplesPrompt/NumFreqPrompt/FirstCoeffPrompt/RectTogglePrompt/SamplePrompt(u8)/Ready)
    - TransInputStep: 6 variants (Init2dPrompt/Init3dOriginPrompt/Init3dAxisPrompt/ForwardPrompt/InversePrompt/Ready)
    - MATH_1.ops: 51 entries (was 43; +8)
    - math1_four_tri_trans.rs: 48 integration tests
    - numerical_accuracy.rs: +13 OM-cited cases (FOUR 5 + TRI 5 + TRANS 5)
  affects:
    - hp41-core/src/ops/math1/modal.rs (FourInputStep + TransInputStep schema revisions + 18 new tests)
    - hp41-core/src/ops/math1/four.rs (NEW — 13 unit tests)
    - hp41-core/src/ops/math1/tri.rs (NEW — 27 unit tests)
    - hp41-core/src/ops/math1/trans.rs (NEW — 14 unit tests)
    - hp41-core/src/ops/math1/mod.rs (three new pub mod declarations)
    - hp41-core/src/ops/math1/xrom.rs (8 new entries in MATH_1.ops + math1_resolve; count 43→51)
    - hp41-core/src/ops/mod.rs (8 new Op variants + 8 dispatch arms)
    - hp41-core/src/ops/program.rs (8 new execute_op arms — pure-data pattern)
    - hp41-core/tests/math1_four_tri_trans.rs (NEW — 48 integration tests)
    - hp41-core/tests/numerical_accuracy.rs (+13 OM-cited cases; 36 total accuracy tests)
    - hp41-cli/src/prgm_display.rs (+8 display arms)
    - hp41-gui/src-tauri/src/prgm_display.rs (+8 display arms)
tech_stack:
  added: []
  patterns:
    - DFT coefficient computation via f64 bridge (same pattern as hyperbolic/trig ops)
    - Rodrigues' rotation formula: v'=v·cos(θ)+(k×v)·sin(θ)+k·(k·v)·(1-cos(θ))
    - SSA ambiguous-case: h=b·sin(A), compare a vs h and b for 0/1/2 solutions
    - Pure-data Op execute_op pattern (delegates to dispatch, no run_loop re-entry)
    - scratch-register layout for FOUR (R00-R26) and TRANS (R00-R24) per TRANS-05/FOUR-05
key_files:
  created:
    - hp41-core/src/ops/math1/four.rs (~320 LOC with 13 unit tests)
    - hp41-core/src/ops/math1/tri.rs (~430 LOC with 27 unit tests)
    - hp41-core/src/ops/math1/trans.rs (~390 LOC with 14 unit tests)
    - hp41-core/tests/math1_four_tri_trans.rs (48 integration tests)
  modified:
    - hp41-core/src/ops/math1/modal.rs (FourInputStep + TransInputStep revisions + 18 tests)
    - hp41-core/src/ops/math1/mod.rs (3 new pub mod declarations)
    - hp41-core/src/ops/math1/xrom.rs (8 entries + math1_resolve; count 43→51)
    - hp41-core/src/ops/mod.rs (8 Op variants + 8 dispatch arms)
    - hp41-core/src/ops/program.rs (8 execute_op arms)
    - hp41-core/tests/numerical_accuracy.rs (+13 OM-cited cases)
    - hp41-cli/src/prgm_display.rs (+8 display arms)
    - hp41-gui/src-tauri/src/prgm_display.rs (+8 display arms)
decisions:
  - "T3D mnemonic chosen for Op::Trans3d: TRANS is 2D-only; T3D unambiguously identifies 3D mode. Both now in xrom_resolve under their respective mnemonics."
  - "FourInputStep schema updated from Plan 28-01 placeholder strings to OM-faithful wording: 'N SAMPLES=?' → 'NO. SAMPLES=?', 'N FREQS=?' → 'NO. FREQ=?', 'FIRST COEFF=?' unchanged, new 'RECT?' for FOUR-03 toggle, 'Yn=?' (1-indexed) for sample prompts."
  - "TransInputStep schema revised from generic Init/Forward/Inverse (Plan 28-01 placeholders) to 2D/3D-split: Init2dPrompt, Init3dOriginPrompt, Init3dAxisPrompt, ForwardPrompt, InversePrompt, Ready. Safe because no downstream consumers existed."
  - "SSA two-solution output uses B1/C1/c1 + B2/C2/c2 labels (not 'SOLUTION 1:' / 'SOLUTION 2:' headers) per OM p.46 display convention transcribed during implementation."
  - "FOUR-06 USER-mode E-key evaluator (op_four_eval_at_t) is a pure function — no run_loop re-entry. It reads scratch registers and computes the series sum. CLI/GUI E-key routing lands in Phases 29/31."
  - "3D forward transform convention: world→local (subtracts origin, rotates). 3D inverse convention: local→world (inverse-rotates, adds origin). Round-trip requires matching input/output conventions."
metrics:
  duration: "~55 minutes"
  completed: "2026-05-17"
  tasks_completed: 5
  tasks_total: 5
  files_created: 4
  files_modified: 8
---

# Phase 28 Plan 10: FOUR + Triangle Solvers + TRANS Summary

**One-liner:** 8 new Op variants land (Op::Four DFT/RECT?/E-key evaluator, 5 triangle solvers with SSA ambiguous-case per OM p.46, Op::Trans2d/Trans3d with Rodrigues' rotation formula) closing Phase 28 at 110/110 v3.0 Math Pac I requirements.

## What Was Built

### Task 1: Modal Schema Revisions + 18 New Tests

**FourInputStep** extended from Plan 28-01 placeholders to OM-faithful prompts:

| Variant | Prompt | Source |
|---------|--------|--------|
| `NumSamplesPrompt` | `"NO. SAMPLES=?"` | HP 00041-90034 FOUR program |
| `NumFreqPrompt` | `"NO. FREQ=?"` | HP 00041-90034 FOUR program |
| `FirstCoeffPrompt` | `"1ST COEFF=?"` | HP 00041-90034 FOUR program |
| `RectTogglePrompt` | `"RECT?"` | FOUR-03 (NEW — Plan 28-10) |
| `SamplePrompt(idx)` | `"Yk=?"` (1-indexed) | HP 00041-90034 FOUR program |
| `Ready` | `None` | Computing state |

**TransInputStep** revised from generic Init/Forward/Inverse (Plan 28-01 placeholders) to 2D/3D-specific split:

| Variant | Prompt | Source |
|---------|--------|--------|
| `Init2dPrompt` | `"X0,Y0,θ?"` | TRANS-01 A-entry 2D |
| `Init3dOriginPrompt` | `"ORIGIN?"` | TRANS-03 A-entry 3D |
| `Init3dAxisPrompt` | `"AXIS+θ?"` | TRANS-03 B-entry 3D |
| `ForwardPrompt` | `"FWD?"` | TRANS-02/04 C-entry |
| `InversePrompt` | `"INV?"` | TRANS-02/04 E-entry |
| `Ready` | `None` | Initialized state (NEW) |

TransInputStep schema revision is SAFE: no downstream consumers existed for the Plan 28-01 placeholder variants.

### Task 2: four.rs — Fourier Analysis

**Core functions:**

```rust
pub fn op_four(state: &mut CalcState) -> Result<(), HpError>  // modal opener
pub fn compute_dft(samples: &[HpNum], num_freq: u8) -> Result<Vec<(HpNum, HpNum)>, HpError>
pub fn store_dft_to_registers(state: &mut CalcState, pairs: &[(HpNum, HpNum)], n_samples: usize)
pub fn convert_to_polar(pairs: &[(HpNum, HpNum)]) -> Result<Vec<(HpNum, HpNum)>, HpError>  // FOUR-03
pub fn op_four_eval_at_t(state: &CalcState, t: HpNum, period: HpNum) -> Result<HpNum, HpError>  // FOUR-06
```

**DFT algorithm (per HP 00041-90034 1979 FOUR program):**
```
aₙ = (2/N) · Σ_{k=1..N} Yₖ · cos(2π·n·k/N)
bₙ = (2/N) · Σ_{k=1..N} Yₖ · sin(2π·n·k/N)
```

**Scratch register layout (FOUR-05):**
```
R00 = a₀   (DC component)
R{2n-1} = aₙ,  R{2n} = bₙ  (for n = 1..L, max L = 10)
R23 = N (sample count for USER-mode eval)
R24 = L (frequency count)
```

**FOUR-06 USER-mode E-key:** pure function reading scratch registers, no run_loop re-entry. Series evaluation formula: f(t) = a₀/2 + Σ_{n=1..L}(aₙ·cos(2π·n·t/T) + bₙ·sin(2π·n·t/T)). CLI/GUI E-key routing lands in Phases 29/31.

13 unit tests: master_op_opens_modal, DFT constant/sine/cosine signals, RECT? rectangular/polar forms, MAX_FOURIER_PAIRS=10 cap, scratch_registers layout, USER-mode eval at t=0/π/2/π + DC-only + explicit period (11 eval tests → 6 distinct combinations tested).

Also added 8 new Op variants + dispatch + execute_op + xrom + prgm_display arms (both copies) in this task.

### Task 3: tri.rs — 5 Triangle Solvers

**Solvers implemented:**

| Op | Algorithm | Stack Input | Output |
|----|-----------|-------------|--------|
| `op_tri_sss` | Law of Cosines | a, b, c | A, B, C |
| `op_tri_asa` | Law of Sines | A, c, B | C, a, b |
| `op_tri_saa` | Law of Sines | a, A, B | C, b, c |
| `op_tri_sas` | Law of Cosines + Law of Sines | b, A, c | a, B, C |
| `op_tri_ssa` | SSA ambiguous case | a, b, A | 0/3/6 lines |

**SSA ambiguous case (TRI-05)** — OM p.46 display sequence transcribed verbatim:

```
h = b · sin(A)  [altitude]

a < h   → "NO SOLUTION"  (1 line)
a = h   → B=90°, C, c   (3 lines — right triangle edge case)
h<a<b   → B1/C1/c1 + B2/C2/c2  (6 lines — TWO solutions)
a >= b  → B, C, c   (3 lines — unique solution)
```

**Angle mode:** all solvers read input and produce output in `state.angle_mode` (Deg/Rad/Grad). Internal computation uses f64 radians via `to_radians` / `from_radians` helpers.

27 unit tests covering equilateral/right-triangle/3-4-5/angle-mode/domain-error variants for each solver, plus all 4 SSA sub-cases.

### Task 4: trans.rs — 2D/3D Coordinate Transforms

**2D Transform (TRANS-01..02):**
```
Forward: x' = (x-x₀)·cos(θ) + (y-y₀)·sin(θ)
         y' = -(x-x₀)·sin(θ) + (y-y₀)·cos(θ)
Inverse: x = x₀ + x'·cos(θ) - y'·sin(θ)
         y = y₀ + x'·sin(θ) + y'·cos(θ)
Scratch: R00=x₀, R01=y₀, R02=θ (TRANS_2D_SCRATCH_RANGE = 0..3)
```

**3D Transform (TRANS-03..04) — Rodrigues' rotation formula:**
```
v' = v·cos(θ) + (k × v)·sin(θ) + k·(k·v)·(1 - cos(θ))
```
where k = unit-normalize(a, b, c) is the rotation axis. Inverse: apply -θ.

```
Scratch: R00=x₀, R01=y₀, R02=z₀ (origin)
         R03=a, R04=b, R05=c (axis direction)
         R06=θ (angle)
         TRANS_3D_SCRATCH_RANGE = 0..25 (TRANS-05)
```

Helper functions: `normalize_3d` (zero-axis → Domain error), `rodrigues_rotate`, `cross_product_3d`, `dot_product_3d`.

Convention note: forward transform does world→local (subtracts origin, rotates); inverse transform does local→world (inverse-rotates by -θ, adds origin back). This convention is documented in the module and tests.

14 unit tests: master modal entry, 90° 2D rotation, inverse round-trip, origin translation, identity, radians mode, z-axis Rodrigues, (1,1,1) axis 120° cyclic permutation, axis normalization, zero-axis Domain error, scratch register layout, zero-rotation identity.

### Task 5: Wire + Gate Tests + Numerical Accuracy

**xrom.rs:** 8 new entries in math1_resolve() and MATH_1.ops (count: 43 → 51 = 6+7+17+2+8+1+2+8). Count test updated.

**numerical_accuracy.rs:** +13 OM-cited cases (5 FOUR + 5 TRI + 5 TRANS = 13 new tests). All pass with 1e-5 tolerance for DFT/geometric cases.

**math1_four_tri_trans.rs:** NEW file with 48 integration tests — satisfies Pitfall 16 / math1_op_test_count meta-gate (≥5 mentions per variant in math1_*.rs files):
- Op::Four: 8 mentions (dispatch, modal, MAX_FOURIER_PAIRS cap, scratch layout, eval at t, rect→polar, DFT constant, xrom_resolve)
- Op::TriSss: 6 mentions; Op::TriAsa: 5; Op::TriSaa: 5; Op::TriSas: 5
- Op::TriSsa: 8 mentions (dispatch, no-solution, two-solution count, B values, one-solution, edge, labels, xrom_resolve)
- Op::Trans2d: 6 mentions; Op::Trans3d: 6 mentions

## Test Results

| Gate | Result |
|------|--------|
| `cargo build -p hp41-core` | PASS (1 pre-existing complex_atan2 dead_code warning) |
| `cargo build -p hp41-cli` | PASS |
| `(cd hp41-gui/src-tauri && cargo build)` | PASS |
| `cargo test -p hp41-core` | 1577 passed, 1 ignored (57 suites) |
| `--lib math1::four::tests` | 13 passed |
| `--lib math1::tri::tests` | 27 passed |
| `--lib math1::trans::tests` | 14 passed |
| `--lib math1::modal::tests` | 52 passed |
| `--test math1_four_tri_trans` | 48 passed |
| `--test math1_op_test_count` | 1 passed (≥5 for all 8 new variants) |
| `--test xrom_shadowing` | 2 passed (51 entries, none shadow builtins) |
| `--test numerical_accuracy` | 36 passed (including +13 new FOUR/TRI/TRANS cases) |
| SC-4 invariant | PRESERVED (display-arm additions are display formatters) |
| Phase 28 complete | 110/110 v3.0 Math Pac I requirements covered |

## Deviations from Plan

### TransInputStep schema revised from generic to 2D/3D-split (Rule 1/2 — correctness requirement)

**Found during:** Task 1 design analysis
**Issue:** Plan 28-01's `InitPrompt/ForwardPrompt/InversePrompt` placeholders couldn't distinguish 2D vs 3D initialization (Op::Trans2d needs x₀/y₀/θ; Op::Trans3d needs two separate entries: origin then axis+θ).
**Fix:** Replaced 3 generic variants with 6 specific ones: Init2dPrompt, Init3dOriginPrompt, Init3dAxisPrompt, ForwardPrompt, InversePrompt, Ready.
**Safety:** No downstream consumers existed (Phases 29/31 hadn't wired TRANS modal yet).

### FourInputStep prompt strings updated to OM-faithful wording (Rule 1 — correctness)

**Found during:** Task 1 implementation against PLAN.md spec
**Issue:** Plan 28-01 placeholder strings ("N SAMPLES=?", "N FREQS=?") didn't match the OM-specified wording per PLAN.md behavior section ("NO. SAMPLES=?", "NO. FREQ=?").
**Fix:** Updated all prompt strings to exact OM wording per FOUR-01/02 requirements.

### 3D transform convention documented (deviation from test expectation)

**Found during:** Task 4 round-trip test
**Issue:** Initial round-trip test incorrectly added origin to forward output before passing to inverse. The forward transform outputs local coordinates (origin subtracted + rotated); the inverse takes local coordinates and recovers world coordinates (inverse-rotate + add origin).
**Fix:** Fixed the test to pass forward output directly to inverse. Convention documented in module doc + test comments.

### T3D mnemonic chosen for Op::Trans3d (T3D vs TRANS3D vs T-3D)

**Decided during:** Task 2 when adding xrom entries
**Rationale:** TRANS is taken by 2D mode. Plan spec mentioned "T3D or similar". T3D is the shortest unambiguous mnemonic that fits the HP-41 6-character XEQ name limit. Documented in SUMMARY as required by plan output spec.

## Threat Flags

None. FOUR/TRI/TRANS operate on `state.regs`, `state.stack`, and `state.print_buffer` — all existing trust boundaries. No new network endpoints, auth paths, or file access patterns.

## Known Stubs

- **Modal routing**: op_four, op_trans2d, op_trans3d open modals (set modal_program/modal_prompt) but the CLI/GUI R/S submit flow, sample-input iteration, and param-entry flow land in Phases 29 (CLI) and 31 (GUI).
- **FOUR-06 E-key**: op_four_eval_at_t is implemented and tested; the CLI/GUI E-key dispatch routing (detecting user_mode AND Four(Ready) state) lands in Phases 29/31 per the v2.1 USER-mode E-key precedent.

## Self-Check

### Created files exist:
- [x] hp41-core/src/ops/math1/four.rs
- [x] hp41-core/src/ops/math1/tri.rs
- [x] hp41-core/src/ops/math1/trans.rs
- [x] hp41-core/tests/math1_four_tri_trans.rs

### Commits exist:
- [x] ff0c394 — feat(28-10): extend FourInputStep + TransInputStep modal prompts (Task 1)
- [x] 9357dd6 — feat(28-10): implement four.rs (Op::Four DFT + RECT? toggle + USER-mode E-key) (Task 2)
- [x] d6797db — feat(28-10): implement tri.rs (5 triangle solvers + OM-conforming SSA ambiguous case) (Task 3)
- [x] 944ed5d — feat(28-10): implement trans.rs (2D+3D coordinate transforms with Rodrigues rotation) (Task 4)
- [x] 449405e — feat(28-10): wire xrom_resolve + numerical_accuracy + test gate for FOUR/TRI/TRANS (Task 5)

## Self-Check: PASSED
