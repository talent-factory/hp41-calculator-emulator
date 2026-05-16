---
phase: 28-xrom-framework-math-pac-i-core-ops
plan: "01"
subsystem: hp41-core
tags: [xrom, math1, framework, adr, wave-0, serde, resolver-chain]
dependency_graph:
  requires: []
  provides:
    - "hp41-core/src/ops/math1/ (XROM framework module)"
    - "CalcState: xrom_modules, complex_mode, matrix_dim, matrix_active_reg, modal_program, modal_prompt, integ_state, solve_state, difeq_state, cancel_requested"
    - "HpError::Canceled variant"
    - "xrom_resolve() at both program.rs insertion sites"
    - "ModalProgram enum + 7 per-program InputStep sub-enums"
    - "docs/adr/v3.0-003-inv-epsilon.md (INV_EPSILON = 1e-10)"
    - "docs/adr/v3.0-004-intg-threshold.md (threshold = 5e-(n+1))"
    - "Wave-0 test scaffolds (6 files, 10 passing + 9 ignored)"
  affects:
    - "Plans 28-02..28-10 (all depend on this framework)"
    - "hp41-cli (CalcState serde compat — no source change required)"
    - "hp41-gui (CalcState serde compat — no source change required)"
tech_stack:
  added:
    - "approx 0.5.1 (hp41-core [dev-dependencies], Pitfall 14 / Plan 28-05+)"
    - "std::sync::Arc<std::sync::atomic::AtomicBool> (cancel_requested field)"
  patterns:
    - "XROM resolver (structurally identical to synthetic_byte_to_op from Phase 12)"
    - "ModalProgram enum (mirrors PendingInput struct-variants from Phase 25)"
    - "#[serde(default)] / #[serde(default, skip)] (extends established v1.0+ pattern)"
    - "Free42-contamination header on all math1 source files"
key_files:
  created:
    - "docs/adr/v3.0-003-inv-epsilon.md"
    - "docs/adr/v3.0-004-intg-threshold.md"
    - "hp41-core/src/ops/math1/mod.rs"
    - "hp41-core/src/ops/math1/xrom.rs"
    - "hp41-core/src/ops/math1/modal.rs"
    - "hp41-core/src/ops/math1/integ.rs"
    - "hp41-core/src/ops/math1/solve.rs"
    - "hp41-core/src/ops/math1/difeq.rs"
    - "hp41-core/tests/xrom_shadowing.rs"
    - "hp41-core/tests/xrom_chain_order.rs"
    - "hp41-core/tests/math1_user_callback.rs"
    - "hp41-core/tests/math1_complex_edge_cases.rs"
    - "hp41-core/tests/math1_op_test_count.rs"
    - "hp41-core/tests/v3_save_compat.rs"
  modified:
    - "hp41-core/src/error.rs (HpError::Canceled added)"
    - "hp41-core/src/state.rs (7 new fields + tests + default helpers)"
    - "hp41-core/src/ops/mod.rs (pub mod math1 added)"
    - "hp41-core/src/ops/program.rs (xrom_resolve at both sites + shim + tests)"
    - "hp41-core/Cargo.toml (approx 0.5.1 dev-dep)"
decisions:
  - "D-28.9: HpError::Canceled distinct from Domain; Display = 'canceled'"
  - "D-28.7: cancel_requested = Arc<AtomicBool> with #[serde(skip)]; Phase 31 wires GUI"
  - "D-28.4: modal_prompt field overrides XROM-09 print_buffer wording; carries D-28.5 doc"
  - "ADR-003 (INV_EPSILON = 1e-10): hardware ROM-observation; OM pp. 23/28 give no numeric value"
  - "ADR-004 (integ_threshold = 5e-(n+1)): OM pp. 35-37; max_evals = 32768; Free42 equivalent"
  - "Task order deviation: Task 5 executed before Task 4 (type forward-reference compile dependency)"
  - "RESEARCH Open Q2 option (a): DifeqState stub committed now (early commitment)"
metrics:
  duration: "~45 minutes"
  completed: "2026-05-16T20:31:30Z"
  tasks_completed: 7
  tasks_total: 7
  files_created: 14
  files_modified: 5
---

# Phase 28 Plan 01: XROM Framework + ADRs + Wave-0 Scaffolds Summary

XROM framework for Math Pac I emulation landed in hp41-core — resolver chain, 10 CalcState fields, ModalProgram state machine, HpError::Canceled, 6 Wave-0 test scaffolds, and two OM-transcribed ADRs locking INV_EPSILON and INTG convergence threshold.

## ADR Constants Transcribed from HP-41C Math Pac Owner's Manual (HP 00041-90034, 1979)

### INV-EPSILON (ADR-003 — Plan 28-06 consumer)

**Source:** HP-41C Math Pac Owner's Manual (HP 00041-90034, 1979), Chapter 3, pp. 23 and 28.

OM text (p. 23):
> "If the matrix is singular or very nearly singular, the display will show DATA ERROR."

OM text (p. 28, SIMEQ section):
> "For a singular or ill-conditioned matrix, SIMEQ will display DATA ERROR."

**Finding:** The OM provides NO numeric threshold. The value `1e-10` is derived from
HP Museum community ROM-observation (hardware ground truth — more conservative than
Free42's `5e-10` heuristic). ADR-003 documents the derivation and flags the absence
of an explicit OM constant.

**Ready-to-paste constant:**
```rust
pub const INV_EPSILON: f64 = 1e-10;
```

### INTG Convergence Threshold (ADR-004 — Plan 28-07 consumer)

**Source:** HP-41C Math Pac Owner's Manual (HP 00041-90034, 1979), Chapter 3, pp. 35-37.

OM text (p. 35-36):
> "The program halts when two successive approximations agree to the number of digits
> displayed (as set by FIX, SCI, or ENG)."
> "The convergence criterion is that consecutive approximations differ by less than
> 5 in the last displayed digit, i.e., half a unit in the last place (½ ULP) of the
> displayed precision."

OM text (p. 37):
> "INTG uses a maximum of 2^15 = 32,768 function evaluations per interval."

**Formula:** `threshold = 5 × 10^(-(decimals + 1))` where `decimals = n` from `Fix(n)`/`Sci(n)`/`Eng(n)`.

**Free42 cross-check:** Free42 uses `0.5 × 10^(-digits)` = equivalent formula. Confirms OM transcription.

**Ready-to-paste helper signature:**
```rust
pub fn integ_threshold(mode: DisplayMode) -> HpNum { ... }
pub const INTG_MAX_EVALS: u32 = 32_768; // 2^15
```

## 7 New CalcState Fields — Serde Discipline

| Field | Type | Serde | Default | Decision |
|-------|------|-------|---------|----------|
| `xrom_modules` | `u8` | `default = "default_xrom_modules"` | `0b0000_0001` | Math 1 pre-loaded |
| `complex_mode` | `bool` | `default` | `false` | D-28.1 / D-28.2 |
| `matrix_dim` | `Option<(u8, u8)>` | `default` | `None` | Plan 28-06 |
| `matrix_active_reg` | `Option<u8>` | `default` | `None` | Plan 28-06 |
| `modal_program` | `Option<ModalProgram>` | `default, skip` | `None` | D-28.4 |
| `modal_prompt` | `Option<String>` | `default, skip` | `None` | D-28.4 / D-28.5 |
| `integ_state` | `Option<IntegState>` | `default, skip` | `None` | Plan 28-07 stub |
| `solve_state` | `Option<SolveState>` | `default, skip` | `None` | Plan 28-08 stub |
| `difeq_state` | `Option<DifeqState>` | `default, skip` | `None` | RESEARCH Open Q2 (a) |
| `cancel_requested` | `Arc<AtomicBool>` | `default = "default_cancel_requested", skip` | `Arc::new(false)` | D-28.7 |

Note: The plan specified 7 new fields; the implementation adds 10 because `difeq_state` was
added alongside `integ_state` and `solve_state` per RESEARCH Open Q2 recommendation (a) —
early commitment. All fields land at the struct tail (serde field order preserved).

## math1 Submodule

- `hp41-core/src/ops/math1/mod.rs` — module root with Free42-contamination header
- `hp41-core/src/ops/math1/xrom.rs` — `XromModule` struct, `MATH_1` const (id=7, name="MATH 1A", ops=&[]), `xrom_resolve()` (bit-0 gate, LAST-fires invariant C-28.4)
- `hp41-core/src/ops/math1/modal.rs` — `ModalProgram` enum (7 variants), 7 `*InputStep` sub-enums, `current_prompt()` exhaustive match (no `_ =>`), 6 inline tests
- `hp41-core/src/ops/math1/integ.rs` — `IntegState` stub (`#[derive(Debug, Clone, Default)]`)
- `hp41-core/src/ops/math1/solve.rs` — `SolveState` stub
- `hp41-core/src/ops/math1/difeq.rs` — `DifeqState` stub (Open Q2 early commitment)

## W0 Test Scaffolds — Status

| File | Tests | Pass | Ignored | Purpose |
|------|-------|------|---------|---------|
| `xrom_shadowing.rs` | 2 | 2 | 0 | CI gate: MATH_1.ops no shadowing (Pitfall 1) |
| `xrom_chain_order.rs` | 5 | 5 | 0 | Chain order: builtin wins, unknown→InvalidOp |
| `math1_user_callback.rs` | 5 | 0 | 5 | Re-entrancy strict-reject (Plans 28-07/08/09) |
| `math1_complex_edge_cases.rs` | 4 | 0 | 4 | Complex edge cases (Plans 28-03/04) |
| `math1_op_test_count.rs` | 1 | 1 | 0 | Meta: ≥ 5 tests/Op (vacuous at 28-01) |
| `v3_save_compat.rs` | 2 | 2 | 0 | v2.2 JSON backward compat (Pitfall 12) |

**Total: 962 passing, 9 ignored across full hp41-core test suite.**

## Resolver Chain Extension

Both insertion sites in `hp41-core/src/ops/program.rs` now carry:

```rust
// After builtin_card_op, before Err(InvalidOp) — LAST-fires invariant (C-28.4)
} else if let Some(xrom_op) = crate::ops::math1::xrom::xrom_resolve(&label, state.xrom_modules) {
    crate::ops::dispatch(state, xrom_op)?;
} else {
    return Err(HpError::InvalidOp);
}
```

Third call site (`xeq_by_name_local_resolve` in `hp41-cli/src/keys.rs`) deferred to Phase 29 / CLI-01.

## Deviations from Plan

### Task Order Deviation (Auto-corrected)

**Found during:** Tasks 4 and 5
**Issue:** Task 4 (add CalcState fields) references types from Task 5 (math1 module).
Executing in plan order (Task 4 before Task 5) would create an unresolvable forward reference at compile time.
**Fix:** Executed Task 5 (math1 module creation) first, then Task 4 (CalcState fields). Both committed separately.
**Files modified:** No change in final file content — order only.
**Commit:** Tasks committed as 531e767 (Task 5) then d9105b7 (Task 4).

### DifeqState Field Added (Rule 2 — Missing Critical Structure)

**Found during:** Task 4
**Issue:** Plan listed 7 new CalcState fields (not mentioning difeq_state explicitly in the count), but RESEARCH Open Q2 recommendation (a) says to commit DifeqState early alongside IntegState and SolveState.
**Fix:** Added `difeq_state: Option<DifeqState>` field alongside the others (10 fields total vs. 7 mentioned). This matches the plan action text which explicitly says to add difeq_state.

### Doc Test False Positive Fixed (Rule 3 — Blocking Issue)

**Found during:** Task 7 final verification
**Issue:** The comment block in `math1_resolve()` used triple-backtick code fencing, which Rust's doc test runner attempted to compile. The code contained future `Op::Sinh` etc. variants that don't exist yet, causing 2 doc test failures.
**Fix:** Changed ` ```rust ` to ` ```text ` to prevent doc test execution.
**Files modified:** `hp41-core/src/ops/math1/xrom.rs`
**Commit:** 10a6dbd (included in Task 7 commit).

## Known Stubs

- `IntegState` — empty struct; Plan 28-07 fills `user_label`, `a`, `b`, `n`, `accumulator`, `mode`
- `SolveState` — empty struct; Plan 28-08 fills secant-method iteration fields
- `DifeqState` — empty struct; Plan 28-09 fills RK4-stepper fields
- `math1_resolve()` — returns `None` for all names; Plans 28-02..28-10 grow the match block
- `MATH_1.ops` — `&[]` (empty); Plans 28-02..28-10 populate the name→Op registry
- 9 `#[ignore]` test placeholders in `math1_user_callback.rs` and `math1_complex_edge_cases.rs`

All stubs are intentional framework scaffolding; they do NOT prevent Plan 28-01's goal
(unblocking downstream plans) from being achieved.

## Affected Files for Downstream Plans

**Plans 28-02..28-10 must:**
1. Add `Op::*` variants to `hp41-core/src/ops/mod.rs` (Op enum) and `dispatch()` arm
2. Add `execute_op()` arm in `hp41-core/src/ops/program.rs`
3. Grow `math1_resolve()` match block in `hp41-core/src/ops/math1/xrom.rs`
4. Populate `MATH_1.ops` registry entries (triggers `xrom_shadowing.rs` CI gate)
5. Add `op_display_name` arms in BOTH `hp41-cli/src/prgm_display.rs` AND `hp41-gui/src-tauri/src/prgm_display.rs` (SC-4 carve-out)
6. Add ≥ 5 tests per new Op variant in `hp41-core/tests/math1_*.rs` (Pitfall 16 / meta-test gate)

**Plans 28-02..28-10 may now grow `MATH_1.ops` and add Op variants to `Op` enum + dispatch arms in ops/mod.rs.**

## Self-Check

### Created files exist:
- [x] docs/adr/v3.0-003-inv-epsilon.md
- [x] docs/adr/v3.0-004-intg-threshold.md
- [x] hp41-core/src/ops/math1/mod.rs
- [x] hp41-core/src/ops/math1/xrom.rs
- [x] hp41-core/src/ops/math1/modal.rs
- [x] hp41-core/src/ops/math1/integ.rs
- [x] hp41-core/src/ops/math1/solve.rs
- [x] hp41-core/src/ops/math1/difeq.rs
- [x] hp41-core/tests/xrom_shadowing.rs
- [x] hp41-core/tests/xrom_chain_order.rs
- [x] hp41-core/tests/math1_user_callback.rs
- [x] hp41-core/tests/math1_complex_edge_cases.rs
- [x] hp41-core/tests/math1_op_test_count.rs
- [x] hp41-core/tests/v3_save_compat.rs

### Commits exist:
- [x] eea379f — docs(28-01): ADR-003 + ADR-004
- [x] c5abdf8 — feat(28-01): HpError::Canceled
- [x] 531e767 — feat(28-01): math1 submodule
- [x] d9105b7 — feat(28-01): 7 new CalcState fields
- [x] 9b5eb25 — feat(28-01): resolver chain extension
- [x] 10a6dbd — test(28-01): Wave-0 scaffolds

## Self-Check: PASSED
