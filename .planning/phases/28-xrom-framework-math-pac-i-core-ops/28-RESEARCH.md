# Phase 28: XROM Framework + Math Pac I Core Ops — Research

**Researched:** 2026-05-16
**Domain:** XROM application-module emulation; numerical methods (Simpson, secant, RK4); modal state machine; user-callback re-entrancy
**Confidence:** HIGH on architecture and integration paths; MEDIUM on two TBD numerical constants (INV-EPSILON, INTG threshold formula) pending OM transcription

---

## Executive Summary

Phase 28 is the **gating phase** of v3.0. It lands the XROM resolver framework, ~40 new `Op` variants for the 10 Math Pac I top-level programs, a new modal-state-machine layer alongside the v2.2 `PendingInput` system, and the user-program callback infrastructure that `INTG`/`SOLVE`/`DIFEQ` need. All work lives in `hp41-core/src/ops/math1/` and `hp41-core/src/state.rs`; the SC-4 invariant is trivially preserved (zero `hp41-gui/src-tauri/src/` source touched).

**Architectural posture: integration, not redesign.** Every load-bearing pattern was settled in v1.0–v2.2 — `CalcState` SSOT, `LiftEffect`-tagged dispatch, the 4-way exhaustive-match invariant, `#[serde(default)]` save-file backward compat, JSON-canonical pipeline, CLI ↔ GUI parity via shared hp41-core resolvers. Math Pac I inherits all of them. The XROM registry is a **resolver** that returns regular `Op::*` variants (structurally identical to v1.1's `synthetic_byte_to_op`) — it never bypasses the exhaustive match.

**Primary recommendation:** Build Plan 28-01 strictly as `framework + 5 ADRs + 7 new CalcState fields + xrom_resolve resolver chain extension + new `HpError::Canceled` variant`. Do NOT land any Math Pac I math logic in 28-01 — that is the proof-of-pattern role of Plan 28-02 (hyperbolics, lowest-cost / highest-confidence target). Lock the two TBD numerical constants (INV-EPSILON in §"5 Irreversible Decisions" #3 and INTG threshold formula in #4) via OM transcription as the **first task of Plan 28-01**, BEFORE the resolver chain ships, because Plan 28-06 (MATRIX) and Plan 28-07 (INTG) cannot meaningfully begin without them and a TBD constant baked into 28-01 will require an ADR amendment downstream.

**Phase shape: 10 plans (28-01 framework → 28-10 FOUR+TRI+TRANS) as drafted in ROADMAP.** Build order is gentle on modal complexity (hyperbolics → complex arithmetic → complex functions → POLY simple modal → MATRIX complex modal → INTG user-callback → SOLVE → DIFEQ → FOUR/TRI/TRANS). Each plan adds 5+ tests per new Op (Pitfall 16) and respects the `criterion bench/dispatch_overhead.rs` < 200 ns/op floor (Pitfall 10).

---

## Settled Context (do not relitigate)

Phase 28 builds on 9 D-28.x decisions already locked in `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md` `<decisions>`. Research output must **build on** them, not contradict them.

| ID | Topic | Locked outcome |
|----|-------|----------------|
| **D-28.1** | ComplexStack location | Overlay X/Y/Z/T — ζ = X+iY, τ = Z+iT. Zero new HpNum storage. One new `complex_mode: bool` flag. |
| **D-28.2** | `complex_mode` lifecycle | Auto-on at first complex op + display annunciator hint; explicit `XEQ "REAL"` deactivates. Load default `false`. |
| **D-28.3** | `XEQ "REAL"` | NEW derived XROM entry point (Math Pac I OM does NOT include it). Documented in `docs/hp41-math1-divergences.md`. Adds CMPLX-18 to REQUIREMENTS.md before Plan 28-04. |
| **D-28.4** | Modal-prompt channel | New `modal_prompt: Option<String>` field (`#[serde(skip)]`). OVERRIDES XROM-09's "via print_buffer" wording. Print buffer continues to carry PRX/PRA/PRSTK only. |
| **D-28.5** | Modal input submit key | R/S submits numeric input. OM page 13 ground truth. Reuses v2.1 `run_stop` Tauri command. |
| **D-28.6** | Hyperbolics UX | XEQ-by-name only. No dedicated key bindings. Real HP-41C + Math Pac I cartridge confirmed by user direct inspection. |
| **D-28.7** | Cancellation plumbing | Field `cancel_requested: Arc<AtomicBool>` + per-64-samples check lands in Phase 28; Tauri command + UI button lands in Phase 31. |
| **D-28.8** | Cancellation cadence | Per-64-samples, NOT per-sample. Matches `MAX_STEPS = 1_000_000` budget check pattern from v2.2 run_loop. |
| **D-28.9** | Canceled error variant | NEW `HpError::Canceled` variant. Display: `"CANCELED"`. Distinct from `HpError::Domain("DATA ERROR")`. |

Also carried forward from PROJECT.md / STATE.md / REQUIREMENTS.md:

| ID | Locked outcome |
|----|----------------|
| **C-28.1 (ADR-001)** | Op-strategy A — one Op variant per Math Pac I function |
| **C-28.2 (ADR-002)** | User-callback re-entrancy — strict-reject nested INTG/SOLVE/DIFEQ |
| **C-28.3 (ADR-005)** | JSON-pipeline shape — separate `docs/hp41-math1-functions.json` |
| **C-28.4** | XROM resolver fires LAST in the chain (Pitfall 1 mitigation) |
| **C-28.5** | `run_loop` (NOT `run_program`) for user-program callback re-entry |

---

## The 5 Irreversible Decisions

These decisions are LOCKED before Plan 28-01 ships any implementation. Three are already settled; two require OM transcription as the FIRST research task inside Plan 28-01.

### Decision 1 — Op-strategy A (LOCKED, ADR-001 carried forward)

**Status:** LOCKED Option A — one `Op` variant per Math Pac I function (~40 new variants). Rejected Option B (`Op::XromCall(u16)` table dispatch).
**Recommendation:** No change. The ~40 new variants are the price we pay for preserving the 4-way exhaustive-match invariant (`dispatch()` + `execute_op()` + both `prgm_display.rs` copies) that has caught dozens of bugs across Phases 1–27.
**Risk if wrong:** Loss of compile-time `prgm_display.rs` coverage — every variant becomes a runtime "missing arm" risk surface. Mitigation if Option B becomes necessary in v3.1+: revisit AFTER Stat 1 Pac is shipped, when the total variant count exceeds 250 and the exhaustive matches become genuinely painful.

### Decision 2 — User-callback re-entrancy policy (LOCKED, ADR-002 carried forward)

**Status:** LOCKED strict-reject nested INTG/SOLVE/DIFEQ. At op entry, check `state.integ_state.is_some() || state.solve_state.is_some()` → `HpError::InvalidOp` if true.
**Recommendation:** No change. Matches Math Pac I OM 1979 hardware behavior. Cited in XROM-08 requirement.
**Risk if wrong:** Wrong reference behavior — community programs that exploit nested numeric methods will misbehave. Counter-evidence required to revisit: a verified OM example showing a successful nested INTG-in-SOLVE call.

### Decision 3 — INV-EPSILON value (TBD — OM TRANSCRIPTION REQUIRED before Plan 28-06)

**Status:** TBD per Pitfall 7. Community-cited as `5e-10`; OM-quoted as `1e-9` for `DET` zero-test. The two values are NOT necessarily the same threshold.
**Recommendation:** Transcribe HP-41C Math Pac Owner's Manual (HP 00041-90034, 1979) **Chapter 4 "Matrix Operations"** (specifically the `INV` and `SIMEQ` algorithm sections beginning ~p.14) before Plan 28-06 lands. Encode the final value as `pub const INV_EPSILON: HpNum = HpNum::from_scientific("1e-9");` (or whatever OM quotes) in `hp41-core/src/ops/math1/matrix.rs` so it surfaces in code review and in `docs/hp41-math1-divergences.md`.
**OM transcription target:** verbatim quote of the singularity-detection paragraph including its exact constant + the OM page number + the QRC card reference if it differs.
**Risk if wrong:** INV disagrees with every near-singular OM worked example. Hardware-faithfulness gate fails. Detection: numerical_accuracy.rs cases `inv_singular`, `inv_near_singular`, `inv_back_sub_overflow` cite the OM page; deviation > 1 ULP at 10-digit precision = wrong EPSILON.

### Decision 4 — INTG convergence threshold formula (TBD — OM TRANSCRIPTION REQUIRED before Plan 28-07)

**Status:** TBD per Pitfall 2. Community floats `threshold = 10^(-decimals - 1)` tied to `DisplayMode` (Fix(n) / Sci(n) / Eng(n)). The formula needs OM verification.
**Recommendation:** Transcribe HP-41C Math Pac Owner's Manual **Chapter 4 "Numerical Integration"** (OM ~p.25, INTG-01..08 ground truth) before Plan 28-07 lands. The Explicit Mode B-entry (`n` subdivisions) and the convergence behavior should be quoted verbatim. Encode as a single helper:
```rust
// hp41-core/src/ops/math1/integ.rs
pub fn integ_threshold(mode: DisplayMode) -> HpNum {
    let decimals = match mode {
        DisplayMode::Fix(n) | DisplayMode::Sci(n) | DisplayMode::Eng(n) => n,
    };
    // OM citation here — verbatim from page X
    HpNum::from_scientific(format!("1e-{}", decimals + 1))  // or whatever OM says
}
```
**OM transcription target:** the verbatim paragraph describing when INTG considers itself converged, including any DisplayMode tie-in, AND the subdivision-cap behavior (REQUIREMENTS lists 2^15 = 32768; verify OM agrees).
**Risk if wrong:** INTG of `∫₀^π sin(x) dx` produces 2.0001 in Fix(4) but 2.000000183 in Fix(9); if the formula's wrong, BOTH tests pass with the same tolerance and the bug is invisible. Detection: numerical_accuracy.rs cases must run the SAME integral in TWO `DisplayMode` settings and assert different precision.

### Decision 5 — JSON-pipeline shape (LOCKED, ADR-005 carried forward)

**Status:** LOCKED separate `docs/hp41-math1-functions.json` file (sibling to `hp41cv-functions.json`). Identical schema plus `xrom: { module, module_id, function_id }` object per entry.
**Recommendation:** No change. Zero migration churn on 130 existing v2.2 entries. Aligns with future v3.1+ pacs each getting their own JSON file. The Phase-30 work to extend `scripts/docs-matrix` to two-input mode follows the v2.2 D-25.16 pattern.
**Risk if wrong:** Combined-file alternative would force a one-time migration of 130 entries with no downstream benefit. Counter-evidence required: a use case where cross-pac lookup needs single-file iteration (none identified in v3.0–v3.3).

---

## XROM Resolver Chain

**Insertion site:** v2.2 has THREE callers that need to grow a `xrom_resolve(name, state.xrom_modules)` fallback. All three receive the same `&str` and call signature; all three currently fall through to `Err(InvalidOp)` after `builtin_card_op` returns `None`.

| Caller | File:Line (v2.2) | Current fallback behavior |
|--------|-----------------|---------------------------|
| `op_xeq` (run-loop `Op::Xeq` arm) | `hp41-core/src/ops/program.rs:492` (the `Op::Xeq(label)` match arm at line ~500–516) | `match find_in_program(program, &label)` → `Err(_)` arm tries `builtin_card_op` → else `Err(InvalidOp)` |
| `run_program::execute_op` indirect-XEQ path | same file, in `execute_op` (line 623+) — search for `builtin_card_op` callers | Same chain via `op_xeq` |
| `xeq_by_name_local_resolve` (CLI fast-path) | `hp41-cli/src/keys.rs:347` | Match block returns `Op::Test(...)` for 8 conditional spellings; `_ => None` defers to caller's modal Enter-arm which calls `builtin_card_op` then falls through |

**LAST-fires invariant (C-28.4 / Pitfall 1):** insert `xrom_resolve` AFTER `builtin_card_op` but BEFORE `Err(InvalidOp)`:

```rust
// hp41-core/src/ops/program.rs ~line 506 (existing v2.2 code)
Err(_) => {
    if let Some(card_op) = builtin_card_op(&label) {
        crate::ops::dispatch(state, card_op)?;
    } else if let Some(xrom_op) =                       // ◄── NEW v3.0
        crate::ops::math1::xrom::xrom_resolve(&label, state.xrom_modules)
    {
        crate::ops::dispatch(state, xrom_op)?;
    } else {
        return Err(HpError::InvalidOp);
    }
}
```

**Why LAST is mandatory:** v2.2 program `XEQ "SIN"` must continue to resolve to `Op::Sin` (built-in transcendental). If Math Pac I ever shipped a function literally named `SIN` (it does not — see §"Resolver-Chain Conflict Map"), the built-in must win silently. The xrom-fires-LAST invariant makes this guarantee structural, not behavioral.

**Shadowing detector test (`tests/xrom_shadowing.rs`):**

```rust
// hp41-core/tests/xrom_shadowing.rs (new in Plan 28-01)
#[test]
fn math1_names_do_not_shadow_builtins() {
    use hp41_core::ops::math1::xrom::MATH_1;
    for (name, _op) in MATH_1.ops {
        // Each Math 1 mnemonic must NOT resolve via any v2.2 resolver
        assert!(
            hp41_cli::keys::xeq_by_name_local_resolve(name).is_none(),
            "Math Pac I name {name:?} shadows xeq_by_name conditional"
        );
        assert!(
            hp41_core::ops::program::builtin_card_op(name).is_none(),
            "Math Pac I name {name:?} shadows builtin_card_op"
        );
        // key_to_op and shifted_key_to_op are keyboard-key resolvers in
        // hp41-cli — XEQ-by-name never reaches them, but check for completeness.
    }
}
```

**Resolver call signature** (Plan 28-01 sets up):

```rust
// hp41-core/src/ops/math1/xrom.rs
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {           // bit 0 = Math 1 loaded
        if let Some(op) = math1_resolve(name) { return Some(op); }
    }
    // v3.1+: if modules & 0b0010 != 0 { stat1_resolve(name) } else None
    None
}

fn math1_resolve(name: &str) -> Option<Op> {
    match name {
        // Hyperbolics (Plan 28-02)
        "SINH"  => Some(Op::Sinh),
        "COSH"  => Some(Op::Cosh),
        // ... etc.
        // Complex stack (Plan 28-03/04)
        "C+"    => Some(Op::CPlus),
        "MAGZ"  => Some(Op::Magz),
        // ... etc.
        // Workflows (Plan 28-05..28-10)
        "MATRIX" => Some(Op::MatrixWorkflow),
        "POLY"   => Some(Op::PolyWorkflow),
        "INTG"   => Some(Op::Integ),
        "SOLVE"  => Some(Op::Solve),
        "DIFEQ"  => Some(Op::Difeq),
        "FOUR"   => Some(Op::Four),
        // Triangles (Plan 28-10)
        "SSS"    => Some(Op::TriSss),
        // Coord transforms (Plan 28-10)
        "TRANS"  => Some(Op::Trans2d),  // 2D init; 3D via XEQ "T3D" or similar
        // D-28.3 derived
        "REAL"   => Some(Op::Real),
        _ => None,
    }
}
```

**`MATH_1` const** lives alongside `xrom_resolve`:

```rust
pub struct XromModule {
    pub id: u8,                                  // 7 (real HP hardware Math Pac ID)
    pub name: &'static str,                      // "MATH 1A"
    pub ops: &'static [(&'static str, Op)],
}
pub const MATH_1: XromModule = XromModule {
    id: 7,
    name: "MATH 1A",
    ops: &[
        ("SINH", Op::Sinh),
        ("COSH", Op::Cosh),
        // ... full list mirroring math1_resolve
    ],
};
```

Phase 30 / DOC-01 (`hp41-math1-functions.json`) is the JSON mirror of this const. The CI parity test (Phase 32 / QUAL-07) asserts `MATH_1.ops` and the JSON have the same keys.

---

## Modal-State-Machine Layer

Math Pac I is **prompt-driven** (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`, `DEGREE=?`, etc.) — fundamentally different from v2.2's one-shot stack ops. A new modal layer lives alongside the existing `PendingInput` system but is **structurally distinct**.

### `ModalProgram` enum + per-program step states

```rust
// hp41-core/src/ops/math1/modal.rs (new — Plan 28-01)
#[derive(Debug, Clone, PartialEq)]
pub enum ModalProgram {
    Matrix(MatrixInputStep),
    Solve(SolveInputStep),
    Poly(PolyInputStep),
    Integ(IntegInputStep),
    Difeq(DifeqInputStep),
    Four(FourInputStep),
    Trans(TransInputStep),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatrixInputStep {
    OrderPrompt,                // "ORDER=?"
    ElementPrompt(u8, u8),      // "A1,1=?", "A1,2=?" ... (row, col); column-major iteration
    Ready,                      // matrix fully entered; awaiting next sub-op (XEQ "DET" etc.)
    EditPrompt,                 // "ROW↑COL=?"
    SimeqInputPrompt(u8),       // "B1=?" .. "BN=?"
    SimeqDone,                  // solution stored at R(N+1) onward
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolveInputStep {
    FunctionNamePrompt,         // "FUNCTION NAME?"
    Guess1Prompt,               // "GUESS 1=?"
    Guess2Prompt,               // "GUESS 2=?"
}

// ... PolyInputStep, IntegInputStep, DifeqInputStep, FourInputStep, TransInputStep
//     each carries its program-specific step state per FEATURES.md program flow.
```

### Composition with v2.2 `PendingInput` (orthogonal — Phase 28 does NOT touch CLI)

| Layer | Lives in | Carries | Phase 28 touch |
|-------|----------|---------|----------------|
| `PendingInput` (hybrid struct-variants per D-25.11) | `hp41-cli/src/app.rs` | `XeqPrompt`, `GtoPrompt`, `LblPrompt`, `FlagPrompt`, `RegisterPrompt`, `HexModal`, `StoArith`, `FixPrompt`, `SciPrompt`, etc. | **NONE** — CLI variants land in Phase 29 |
| `ModalProgram` (new) | `hp41-core/src/state.rs` field; `hp41-core/src/ops/math1/modal.rs` enum def | `Matrix`, `Solve`, `Poly`, `Integ`, `Difeq`, `Four`, `Trans` workflow state | **Phase 28** — enum + field land in Plan 28-01; per-program variants fill in as plans 28-05..28-10 ship |

The CLI's `pending_prompt()` exhaustive match (Phase 29 / CLI-05) will compose the two layers: if `state.modal_program.is_some()`, the modal renderer reads `state.modal_prompt` (D-28.4) and displays it; otherwise the existing `PendingInput` rendering takes over.

### `modal_prompt: Option<String>` channel (per D-28.4)

```rust
// hp41-core/src/state.rs — new field, lands in Plan 28-01
/// Transient modal-prompt text written by Math Pac I programs.
/// Cleared at modal entry, set on each step transition, cleared on modal exit.
/// Distinct from `print_buffer` (which carries PRX/PRA/PRSTK output only).
/// `#[serde(skip)]` — transient, never persisted; resume-after-load resets to None.
#[serde(default, skip)]
pub modal_prompt: Option<String>,
```

Lifecycle:
1. `op_matrix_workflow(state)` enters → `state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::OrderPrompt))`; `state.modal_prompt = Some("ORDER=?".to_string())`.
2. User enters digit `3` and presses R/S (D-28.5) → CLI's `flush_entry_buf` → step transition to `MatrixInputStep::ElementPrompt(0, 0)` → `state.modal_prompt = Some("A1,1=?".to_string())`.
3. After last element entered → `state.modal_program = Some(ModalProgram::Matrix(MatrixInputStep::Ready))`; `state.modal_prompt = None`; control returns to user (XEQ "DET" / "INV" / "SIMEQ" available).
4. ESC or modal-cancel → `state.modal_program = None`; `state.modal_prompt = None`.

**Why a new field, not `print_buffer`:** D-28.4 rationale — `print_buffer` semantics (drain-on-every-command, scrollback in GUI print panel) stay unchanged. Modal prompts have their own transient lifecycle that doesn't survive a command cycle, so they need their own channel. `print_buffer` would either accumulate stale prompts or require new draining semantics specific to modal display.

---

## User-Callback Infrastructure

INTG/SOLVE/DIFEQ are the only architecturally novel piece of Phase 28 — they re-enter `run_loop` for each sample point. Everything else (hyperbolics, complex arithmetic, MATRIX) is "more of the same."

### `run_loop` re-entrancy from `op_integ` / `op_solve` / `op_difeq`

**Why `run_loop`, not `run_program` (per C-28.5):** `run_program` clones the program vec for safety. For an INTG with 1000 sample points and a 30 KB program, recursive re-entry through `run_program` would re-clone 30 MB. `run_loop` reuses the outer program's clone; the user-callback re-entry is structurally identical to `Op::XeqInd`'s recursion at `hp41-core/src/ops/program.rs:476–491`.

**Pre-mutation 4-deep `call_stack` cap (Pitfall 4 mitigation):**

```rust
// hp41-core/src/ops/math1/integ.rs (Plan 28-07)
pub fn op_integ(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    // ── XROM-08: strict-reject nested INTG/SOLVE/DIFEQ ──────────────────
    if state.integ_state.is_some() || state.solve_state.is_some() {
        return Err(HpError::InvalidOp);
    }
    // ── Pre-mutation call_stack cap (mirrors Op::XeqInd at line 479) ────
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }
    // Reset cancellation flag at op entry (D-28.7).
    state.cancel_requested.store(false, Ordering::Relaxed);
    state.integ_state = Some(IntegState::new(/* a, b, n, user_label */));

    // Outer Simpson loop — re-enters run_loop per sample point.
    for sample_idx in 0..n_samples {
        // Per-64-samples cancellation check (D-28.8).
        if sample_idx & 0x3F == 0 && state.cancel_requested.load(Ordering::Relaxed) {
            state.integ_state = None;
            return Err(HpError::Canceled);                       // D-28.9
        }
        // MAX_STEPS 1_000_000 budget per outer run_program is preserved
        // automatically — each user-callback step counts against it.
        let x_k = compute_sample_point(sample_idx, state);
        state.stack.push_lift(x_k);
        state.call_stack.push(state.pc);                          // 1 level used
        state.pc = find_in_program(program, &user_label)? + 1;
        run_loop(state, program, MAX_STEPS_REMAINING)?;           // re-entrant
        // On return, X = f(x_k).
        accumulate_simpson_term(state.integ_state.as_mut().unwrap(), state.stack.x());
    }

    let result = finalize_simpson(state.integ_state.as_mut().unwrap());
    state.integ_state = None;
    state.stack.push_lift(result);                                // LiftEffect::Enable
    Ok(())
}
```

**Why pre-mutation cap is mandatory:** if `state.call_stack.push` happens BEFORE the cap check, an over-budget call corrupts the stack with one extra entry that's never popped. v2.2 D-22.15 enshrined this invariant; Plan 28-07 inherits it.

**Strict-reject of nested INTG/SOLVE/DIFEQ (XROM-08 / D-28.7 / ADR-002):** at op entry, check `state.integ_state.is_some() || state.solve_state.is_some()`. If true, return `HpError::InvalidOp` without mutation. DIFEQ-specific state field TBD (`difeq_state: Option<DifeqState>` likely, with the same pattern); decide in Plan 28-09.

**Strict-reject test surface** (`tests/math1_user_callback.rs`, lands in Plan 28-07):

```rust
#[test]
fn nested_integ_inside_integ_rejected() {
    // outer XEQ "INTG" with user fn "F" defined as: LBL F / XEQ "INTG" / RTN
    // expect: HpError::InvalidOp on the inner XEQ entry
}
#[test]
fn nested_solve_inside_integ_rejected() { /* ... */ }
#[test]
fn nested_integ_inside_solve_rejected() { /* ... */ }
#[test]
fn nested_difeq_inside_integ_rejected() { /* ... */ }
#[test]
fn user_fn_stops_aborts_integ() { /* ... */ }  // STOP inside callback
```

### `integ_state` / `solve_state` struct shape

Both transient, both `#[serde(default, skip)]`. Planner chooses layout per OM-cited algorithm. Constraint: each struct must be `Default::default()`-constructible so `CalcState::new()` doesn't break.

```rust
// hp41-core/src/ops/math1/integ.rs
#[derive(Debug, Clone, Default)]
pub struct IntegState {
    pub user_label: String,
    pub a: HpNum,
    pub b: HpNum,
    pub n: u16,           // subdivisions, cap 32768 per INTG-07
    pub accumulator: HpNum,
    pub mode: IntegMode,  // Discrete vs Explicit per INTG-02/INTG-03
}

#[derive(Debug, Clone, Default)]
pub struct SolveState {
    pub user_label: String,
    pub x1: HpNum,        // GUESS 1
    pub x2: HpNum,        // GUESS 2
    pub fx1: HpNum,       // f(GUESS 1)
    pub fx2: HpNum,       // f(GUESS 2)
    pub iteration: u8,    // cap 100 per SOLV-07
}
```

### `HpError::Canceled` (NEW variant per D-28.9)

```rust
// hp41-core/src/error.rs — Plan 28-01 extension
pub enum HpError {
    Overflow,
    DivideByZero,
    InvalidOp,
    Domain,
    CallDepth,
    Canceled,              // ◄── NEW Phase 28
}

impl std::fmt::Display for HpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HpError::Canceled => write!(f, "CANCELED"),
            // ... other arms unchanged
        }
    }
}
```

`HpError` is never serialized (per CONTEXT.md note) — save-file forward-compat is unaffected.

---

## Complex-Stack Layout

**Final answer per D-28.1: Overlay X/Y/Z/T — ζ = X+iY, τ = Z+iT.**

Zero new HpNum storage fields on `CalcState`. One new `complex_mode: bool` flag (auto-on at first complex op per D-28.2; explicit `XEQ "REAL"` to deactivate per D-28.3).

### Why overlay (vs dedicated registers vs eigener struct)

| Option | Cost | Verdict |
|--------|------|---------|
| (a) Overlay X/Y/Z/T | Zero new storage. Stack-lift semantics on `C+`/`C-`/etc. consume Y+iX AND T+iZ; result fills ζ; T-replicate fills τ from old T. | **CHOSEN.** OM-faithful + zero save-file growth + no clobbering surface. The cost (number-entry learning `complex_mode` semantics) is contained in `flush_entry_buf()` (one function) and is testable in isolation. |
| (b) Dedicated R02–R05 | User `STO 02` silently clobbers the complex stack. | Rejected — clobbering surface. |
| (c) New `ComplexStack` struct | Pads every save-file with 4 extra `HpNum` fields. Diverges from OM "two-element complex stack" mental model. | Rejected — save-file growth + OM-model divergence. |

### `complex_atan2(0,0) = 0` first arm (Pitfall 6 mitigation)

```rust
// hp41-core/src/ops/math1/complex.rs (Plan 28-03)
fn complex_atan2(im: HpNum, re: HpNum) -> HpNum {
    // ── FIRST ARM: (0,0) → 0, NOT NaN, NOT DATA ERROR ──────────────────
    // OM 1979 p.XX confirms DEG/RAD mode returns 0 for atan2(0,0).
    if im.is_zero() && re.is_zero() {
        return HpNum::ZERO;
    }
    // f64 bridge per STACK.md Decision 1 (pattern from num.rs::checked_asin)
    let result_f64 = im.to_f64().atan2(re.to_f64());
    HpNum::from_f64_rounded(result_f64)
}
```

### Branch-cut policy for `LnZ` / `ZpowW`

- **`LnZ(0+0i)`**: REQUIREMENTS CMPLX-11 says "(0,0)-Handling per Pitfall 6". Recommendation: return `HpError::Domain` ("DATA ERROR") on `re == 0 && im == 0`. ln(0) on a real-valued stack is already DATA ERROR in v2.2 (`op_ln`); the complex variant must be symmetric.
- **`ZpowW(0+0i, w)`** with Re(w) ≤ 0 (CMPLX-17): return `HpError::Domain`. With Re(w) > 0: returns 0.
- **`CDiv(numer, 0+0i)`**: REQUIREMENTS CMPLX-05 mandates `HpError::DivideByZero` BEFORE the actual division. Symmetric with v2.2 `1/0 → DivideByZero` from `op_recip`.

### Stack-lift semantics in `complex_mode`

**Number entry:** `3.5 ENTER 2` in `complex_mode = true` → ζ with re=3.5, im=2 (X+iY). The planner has discretion to match OM bit-for-bit; this is the "Claude's discretion" item from CONTEXT.md `<decisions>` (number-entry semantics line).

**Binary ops (`C+`, `C-`, `C×`, `C÷`):** consume ζ AND τ (4 stack levels), write result to ζ, T-replicate pattern fills τ from old T. LiftEffect: `Enable` (matches v2.2 binary-arithmetic pattern).

**Unary ops (`MAGZ`, `CINV`, `E↑Z`, `LNZ`, etc.):** consume ζ, write result to ζ, τ unchanged. LiftEffect: `Disable` (matches v2.2 transcendental pattern).

### `XEQ "REAL"` (D-28.3 derived requirement CMPLX-18)

```rust
// hp41-core/src/ops/math1/complex.rs (Plan 28-04)
pub fn op_real(state: &mut CalcState) -> Result<(), HpError> {
    state.complex_mode = false;
    // Stack and ζ/τ overlay state untouched — user keeps current numbers
    // in X/Y/Z/T, now interpreted as real values per pre-v3.0 semantics.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

Add `CMPLX-18: Op::Real (XEQ "REAL") — deactivates complex_mode; resets to false; no other side effects` to REQUIREMENTS.md before Plan 28-04 lands (CONTEXT.md note).

---

## Per-Program Implementation Notes

One short block per program. OM citations are placeholder where Plan 28-01 OM transcription has not happened yet.

### Hyperbolics (Plan 28-02) — proof-of-pattern

| Op | Domain check | LiftEffect | OM page |
|----|---------------|------------|---------|
| `Op::Sinh` | none | Disable | 44 |
| `Op::Cosh` | none | Disable | 44 |
| `Op::Tanh` | none | Disable | 44 |
| `Op::Asinh` | none | Disable | 44 |
| `Op::Acosh` | X < 1 → `HpError::Domain` | Disable | 44 |
| `Op::Atanh` | \|X\| ≥ 1 → `HpError::Domain` | Disable | 44 |

f64 bridge: `rust_decimal` has no `sinh`/`cosh`/`tanh`. Use established `num.rs` pattern (`HpNum::to_f64().sinh()` → `HpNum::from_f64_rounded`). Five tests per op per Pitfall 16.

### Complex Stack & Arithmetic (Plan 28-03)

| Op | Notes |
|----|-------|
| `Op::CPlus` / `Op::CMinus` | Component-wise add/sub on (re, im). Trivial. |
| `Op::CTimes` | `(a+bi)(c+di) = (ac-bd) + (ad+bc)i`. Four `HpNum` multiplies + 2 add/sub. |
| `Op::CDiv` | **DivideByZero check FIRST** (Pitfall 6). Standard `(a+bi)/(c+di)` formula. |

### Complex Functions (Plan 28-04)

| Op | Algorithm | Edge cases |
|----|-----------|-----------|
| `Op::Magz` (MAGZ) | `sqrt(re² + im²)` via f64 bridge | none |
| `Op::Cinv` (CINV) | `1/(re+im·i)` — DivideByZero on (0,0) | (0,0) |
| `Op::ZpowN` (Z↑N) | Integer-exponent power via repeated multiply (no f64) | none |
| `Op::Zpow1N` (Z↑1/N) | Polar form: `r^(1/n) · cis(θ/n)` via complex_atan2 + sqrt | (0,0) returns 0 |
| `Op::ExpZ` (E↑Z) | Euler: `e^re · (cos(im) + i sin(im))` | none |
| `Op::LnZ` (LNZ) | `ln(r) + i·θ` via complex_atan2 | **(0,0) → Domain** per CMPLX-11 |
| `Op::SinZ` / `Op::CosZ` / `Op::TanZ` | hyperbolic identities | TanZ: `cos(re)=0 && sinh(im)=0` → Domain |
| `Op::ApowZ` (A↑Z) | `exp(z · ln(a))` | a=0+0i: Domain |
| `Op::LogZ` (LOGZ) | `LnZ / ln(10)` | (0,0) → Domain |
| `Op::ZpowW` (Z↑W) | `exp(w · LnZ)` | (0,0)^w with Re(w) ≤ 0: Domain |

### POLY / ROOTS (Plan 28-05) — Pitfall 5 fidelity gate

| Aspect | Behavior |
|--------|----------|
| `Op::PolyWorkflow` (XEQ "POLY") | Master: opens `ModalProgram::Poly(PolyInputStep::DegreePrompt)` → 2..5 → `CoefficientPrompt(A, B, C, ...)` per degree → executes |
| `Op::Roots` (XEQ "ROOTS") | Sub-entry: bypasses `DegreePrompt`; coefficients assumed in R00–R04 |
| Output format | **EXACTLY** `U=u` / `V=v` / `U=u` / `-V=-v` for complex pairs (Pitfall 5 fidelity gate — OM-cited 4-line format) |
| Multiplicity-as-cluster | `(x-1)^5` returns 5 roots with small imag parts (~10⁻³); **NO snap-to-zero post-processing**. Documented in `docs/hp41-math1-divergences.md` (Phase 30 / DOC-04) |
| Non-convergence | `\|imag\| > 10⁹` during real-polynomial iteration → `HpError::Domain` ("DATA ERROR") |
| Algorithm | Closed-form quadratic for degree 2; iterative (Bairstow-like per OM Chapter 7) for cubic/quintic |
| Test source | OM Section 7 worked examples; tests use `approx::assert_relative_eq!(max_relative = 1e-7)` per Pitfall 14 |

### MATRIX (Plan 28-06) — most complex single program

| Op | OM page | Behavior |
|----|---------|----------|
| `Op::MatrixWorkflow` | 10 | Master: `ORDER=?` (1..14) → SET SIZE → `A1,1=?` ... `AN,N=?` (column-major) → Ready |
| `Op::MatSize` | 10 | Returns ORDER from R14 |
| `Op::MatVmat` | 10 | Sequentially displays `A1,1=` ... `AN,N=` via `state.print_buffer` |
| `Op::MatEdit` | 10 | `ROW↑COL=?` prompt → `I ENTER J` → `Ai,j=?` → new value |
| `Op::MatDet` | 14 | LU with partial pivoting (column-major access); det = Πpivots × pivot-sign |
| `Op::MatInv` | 14 | Gauss-Jordan on `[A \| I]`. **INV_EPSILON from D-3 (TBD).** Singular → `state.modal_prompt = Some("NO SOLUTION")`; result undefined |
| `Op::MatSimeq` | 16 | `B1=?` ... `BN=?` (RHS input) → Gauss elimination → solution at R(N+1)..R(2N). Sets flag 5 after column storage |
| `Op::MatVcol` | 16 | Displays B1..BN |
| Storage convention | column-major from R15 onward; ORDER in R14 (MAT-02) |
| Max ORDER | 14 (memory-bounded) |
| Flags | Flag 4 set during input phase, flag 5 set after SIMEQ column storage |

**INV_EPSILON encoding:**
```rust
// hp41-core/src/ops/math1/matrix.rs (Plan 28-06)
/// Singularity-detection threshold for matrix INV. Sourced from
/// HP-41C Math Pac Owner's Manual (00041-90034, 1979) p.XX — exact quote
/// transcribed during Plan 28-01 research-prep per Decision 3.
pub const INV_EPSILON: HpNum = /* TBD — locked in Plan 28-01 */;
```

### INTG (Plan 28-07) — first user-callback program

| Aspect | Behavior |
|--------|----------|
| `Op::Integ` (XEQ "INTG") | Master: opens `ModalProgram::Integ(IntegInputStep::ModeChoice)` → Discrete OR Explicit |
| Discrete mode | A=`h`, B=`f(xⱼ)` sample input, C=trapezoidal, D=Simpson. Even-`n` check returns `N NOT EVEN` (display modal_prompt) |
| Explicit mode | A=`(a,b)` interval bounds, B=`n` subdivisions, `FUNCTION NAME?` prompt for user LBL |
| Algorithm | Simpson's rule with fixed `n` (NO adaptive refinement — user controls accuracy per INTG-04) |
| Subdivision cap | 2^15 = 32768 (INTG-07); exceeded → `HpError::Domain` |
| Convergence threshold | **TBD from Decision 4** — `threshold = 10^(-decimals - 1)` tied to `DisplayMode` |
| Scratch | R00–R07; `integ_state: Option<IntegState>` (`#[serde(skip)]`) for mid-iteration |
| Re-entry | `run_loop` (NOT `run_program`) per Pitfall 4 / C-28.5 |
| Cancellation | Per-64-samples `state.cancel_requested.load(Ordering::Relaxed)` check per D-28.8 |

### SOLVE (Plan 28-08) — reuses INTG user-callback infra

| Aspect | Behavior |
|--------|----------|
| `Op::Solve` (XEQ "SOLVE") | Master: `FUNCTION NAME?` + `GUESS 1=?` + `GUESS 2=?` prompts |
| `Op::Sol` (XEQ "SOL") | Sub-entry; bypasses prompts (guesses pre-set in R00/R01) |
| Algorithm | Modified secant iteration (OM-specified) |
| Termination paths | Three: `NO ROOT FOUND`, `ROOT IS <v>`, `ROOT IS BETWEEN <v1> AND <v2>` (Pitfall 3) |
| Iteration cap | 100 (SOLV-07); MAX_STEPS 1_000_000 outer budget |
| Scratch | R00–R06; `solve_state: Option<SolveState>` |
| Re-entry | Same as INTG |
| Nested rejection | `state.integ_state.is_some() \|\| state.solve_state.is_some()` → InvalidOp per XROM-08 |

### DIFEQ (Plan 28-09)

| Aspect | Behavior |
|--------|----------|
| `Op::Difeq` (XEQ "DIFEQ") | `FUNCTION NAME?` / `ORDER=?` (1 or 2) / `STEP SIZE=?` / `X0=?` / `Y0=?`; if ORDER=2 also `Y'0=?` |
| Algorithm | 4th-order Runge-Kutta (OM-specified) |
| Scratch | R00–R07 |
| User callback | `f(x, y)` for ORDER=1; `f(x, y, y')` for ORDER=2 |
| Output | Step-by-step via `print_buffer` (DIFEQ-05) |
| Re-entry | Same pattern as INTG/SOLVE; needs `difeq_state: Option<DifeqState>` field (Plan 28-09 introduces) |
| Nested rejection | Add `state.difeq_state.is_some()` to the XROM-08 guard |

### FOUR (Plan 28-10)

| Aspect | Behavior |
|--------|----------|
| `Op::Four` (XEQ "FOUR") | `NO. SAMPLES=?` / `NO. FREQ=?` / `1ST COEFF=?` |
| Sample input | `Y1=?` ... `YN=?` |
| `RECT?` toggle | Rectangular (aₙ, bₙ) vs polar (cₙ, φₙ) coefficients (FOUR-03) |
| Pairs cap | Up to 10 (aₙ, bₙ) pairs |
| Scratch | R00–R26 |
| USER-mode `E`-key | Evaluates Fourier series at time `t` after coefficient computation (FOUR-06) |

### Triangles (Plan 28-10)

| Op | Algorithm | OM page |
|----|-----------|---------|
| `Op::TriSss` | Law of Cosines: all three angles from three sides | 46 |
| `Op::TriAsa` | Law of Sines | 46 |
| `Op::TriSaa` | Law of Sines | 46 |
| `Op::TriSas` | Law of Cosines | 46 |
| `Op::TriSsa` | **Ambiguous case** — two possible solutions; OM-conformes Behandeln per TRI-05; pattern-match from OM worked examples | 46 |

### TRANS — coordinate transformations (Plan 28-10)

| Aspect | Behavior |
|--------|----------|
| `Op::Trans2d` | A=init `(x₀, y₀, θ)` → C=forward → E=inverse |
| `Op::Trans3d` | A=`(origin)` → B=`(a, b, c, θ)` rotation-axis → C=forward → E=inverse via Rodrigues' rotation formula |
| Scratch | R00–R24 |

---

## Resolver-Chain Conflict Map

Math Pac I names vs v2.2 built-in mnemonics — search for shadowing risks.

### No collision (safe to register straight)

| Math Pac I name | v2.2 status |
|-----------------|-------------|
| `SINH`, `COSH`, `TANH`, `ASINH`, `ACOSH`, `ATANH` | NOT in v2.2 — clean registration |
| `MATRIX`, `MATRIX`-sub-entries (`SIZE`, `VMAT`, `EDIT`, `DET`, `SIMEQ`, `VCOL`) | None of these names appear in `xeq_by_name_local_resolve` or `builtin_card_op` |
| `POLY`, `ROOTS` | Not in v2.2 |
| `INTG`, `SOLVE`, `SOL`, `DIFEQ`, `FOUR` | Not in v2.2 |
| `C+`, `C-`, `C×`, `C÷`, `MAGZ`, `CINV`, `Z↑N`, `Z↑1/N`, `E↑Z`, `LNZ`, `SINZ`, `COSZ`, `TANZ`, `A↑Z`, `LOGZ`, `Z↑W` | All clean — v2.2 has no complex-prefixed names |
| `SSS`, `ASA`, `SAA`, `SAS`, `SSA` | Triangle names — not in v2.2 |
| `TRANS` | Not in v2.2 (the v2.2 `Op::Trans` candidate referenced in old ARCHITECTURE.md draft was Advanced Matrix Pac transpose, NOT Math Pac I) |
| `REAL` | NEW per D-28.3; not in v2.2 |

### One potential collision — verify before Plan 28-06

| Math Pac I name | v2.2 conflict | Resolution |
|------------------|---------------|------------|
| `INV` (matrix inverse) | **Maybe** — v2.2 has `Op::Inv` (reciprocal `1/x`). v2.2 `xeq_by_name_local_resolve` does NOT register `"INV"` (line 348+ match has only the 8 conditional spellings). v2.2 `builtin_card_op` does NOT register `"INV"` (line 988+ has only WPRGM/RDPRGM/WDTA/RDTA + 8 conditionals). So `XEQ "INV"` currently returns `Err(InvalidOp)` in v2.2 — **no collision**. Math Pac I `XEQ "INV"` (matrix inverse) routes cleanly through `xrom_resolve`. | Safe — register `("INV", Op::MatInv)` in `MATH_1.ops`. The keyboard `1/x` key continues to dispatch `Op::Inv` (reciprocal) directly without going through XEQ-by-name resolution. The names are different `Op` variants, the mnemonic is reused, and the LAST-fires invariant is moot because v2.2 has no resolver entry for the bare string `"INV"`. |

### Recommendation

Run an explicit pre-Plan-28-01 grep audit:
```bash
grep -nE '"(SINH|COSH|TANH|ASINH|ACOSH|ATANH|MATRIX|POLY|ROOTS|INTG|SOLVE|SOL|DIFEQ|FOUR|MAGZ|CINV|C\+|C-|C×|C÷|LNZ|SINZ|COSZ|TANZ|ExpZ|LogZ|Z\+N|REAL|SSS|ASA|SAA|SAS|SSA|TRANS|INV|DET|EDIT|SIZE|VMAT|VCOL|SIMEQ)"' \
  hp41-core/src/ hp41-cli/src/
```

If any match returns inside `xeq_by_name_local_resolve` or `builtin_card_op`, that name needs disambiguation. The `tests/xrom_shadowing.rs` CI gate (Plan 28-01) makes this assertion structural and automated, but a manual grep audit before the test runs catches embarrassing surprises during plan-checking.

---

## State-Field Changes to `CalcState`

Seven new fields land in Plan 28-01. All are positioned at the end of the struct (preserves serde field-order for v1.0–v2.2 save-file backward compat).

```rust
// hp41-core/src/state.rs — Plan 28-01

pub struct CalcState {
    // ... existing v1.0–v2.2 fields unchanged ...

    // ── v3.0 Math Pac I additions ───────────────────────────────────────

    /// XROM module bitfield. Bit 0 = Math 1 loaded (default 0b1 for v3.0).
    /// Bits 1..7 reserved for Stat 1 (v3.1), Time (v3.2), Advantage (v3.3).
    /// `#[serde(default = "default_xrom_modules")]` — v1.0–v2.2 save files
    /// load with `default_xrom_modules() = 0b1`, ensuring xrom_resolve fires
    /// for Math 1 names from the first dispatch after load.
    #[serde(default = "default_xrom_modules")]
    pub xrom_modules: u8,

    /// Complex-mode flag (D-28.2). Auto-on at first complex op; explicit
    /// XEQ "REAL" clears (D-28.3 / CMPLX-18). Persistent: v1.0–v2.2 save
    /// files load with `false`; safe default — re-arms on first complex op.
    #[serde(default)]
    pub complex_mode: bool,

    /// Active matrix dimensions (rows, cols). Set by MATRIX workflow's
    /// ORDER=? prompt. None = no matrix declared.
    #[serde(default)]
    pub matrix_dim: Option<(u8, u8)>,

    /// Matrix base register pointer. Set by MATRIX workflow alongside
    /// matrix_dim. Per MAT-02: column-major elements from R15 onward, so
    /// matrix_active_reg is typically Some(15) but the field is
    /// future-proof for matrices stored elsewhere.
    #[serde(default)]
    pub matrix_active_reg: Option<u8>,

    /// Active modal-workflow program state (Plan 28-05..28-10 fill the
    /// variants). Transient — `#[serde(skip)]` per D-28.4 / Pitfall 12.
    /// None means no Math Pac I prompt-driven workflow is active.
    #[serde(default, skip)]
    pub modal_program: Option<crate::ops::math1::modal::ModalProgram>,

    /// Modal-prompt text channel (D-28.4). Distinct from print_buffer.
    /// Set on each modal step transition; cleared on modal exit.
    /// `#[serde(skip)]` — transient, never persisted.
    #[serde(default, skip)]
    pub modal_prompt: Option<String>,

    /// In-flight INTG state. Some(_) iff an INTG re-entry recursion is
    /// active. `#[serde(skip)]` — Pitfall 12 mitigation.
    #[serde(default, skip)]
    pub integ_state: Option<crate::ops::math1::integ::IntegState>,

    /// In-flight SOLVE state. Same lifecycle as integ_state.
    #[serde(default, skip)]
    pub solve_state: Option<crate::ops::math1::solve::SolveState>,

    /// Cancellation channel (D-28.7 plumbing; D-28.8 cadence; D-28.9 error).
    /// Field lands in Phase 28; Tauri command + UI button lands in Phase 31.
    /// `#[serde(skip)]` — transient.
    /// Note: `Arc<AtomicBool>` is not Default, so a #[serde(default)] needs
    /// a custom `default_cancel_requested()` fn that returns
    /// `Arc::new(AtomicBool::new(false))`.
    #[serde(default = "default_cancel_requested", skip)]
    pub cancel_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

fn default_xrom_modules() -> u8 { 0b0000_0001 }
fn default_cancel_requested() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))
}
```

**Note: `difeq_state` field not in the Plan 28-01 list above.** Plan 28-09 will add it when DIFEQ implementation lands. Adding it later is safe because `#[serde(default, skip)]` makes it backward-compat with any save files written between Plan 28-08 and Plan 28-09.

**Save-file backward compat verification:** Plan 28-01 ships a regression test (`tests/v3_save_compat.rs`) that loads a synthetic v2.2 save file and confirms `CalcState::default_xrom_modules() == 0b1` after deserialization. Pitfall 12 mitigation.

---

## Op-Strategy A Consequences

### Variant count growth

| Group | Variants | Plan |
|-------|----------|------|
| Hyperbolics | 6 | 28-02 |
| Complex stack arith | 4 (CPlus, CMinus, CTimes, CDiv) | 28-03 |
| Complex functions | 12 (Magz, Cinv, ZpowN, Zpow1N, ExpZ, LnZ, SinZ, CosZ, TanZ, ApowZ, LogZ, ZpowW) + Real | 28-04 |
| POLY/ROOTS | 2 (PolyWorkflow, Roots) | 28-05 |
| MATRIX | 8 (MatrixWorkflow, MatSize, MatVmat, MatEdit, MatDet, MatInv, MatSimeq, MatVcol) | 28-06 |
| INTG | 1 (Integ) | 28-07 |
| SOLVE | 2 (Solve, Sol) | 28-08 |
| DIFEQ | 1 (Difeq) | 28-09 |
| FOUR | 1 (Four) | 28-10 |
| Triangles | 5 (TriSss, TriAsa, TriSaa, TriSas, TriSsa) | 28-10 |
| TRANS | 2 (Trans2d, Trans3d) | 28-10 |
| **Total** | **~44** | — |

### Exhaustive-match cost

Each new variant lands in 4 places before any caller compiles (per CLAUDE.md "Op variants land before TUI code"):
1. `hp41-core/src/ops/mod.rs::dispatch` (`Op::Sinh => math1::hyperbolics::op_sinh(state)`) — Phase 28
2. `hp41-core/src/ops/program.rs::execute_op` (delegate to dispatch OR special-case for run_loop-only ops) — Phase 28
3. `hp41-cli/src/prgm_display.rs::op_display_name` — Phase 29 (not Phase 28; "Op variants land before TUI code" means hp41-core dispatch+execute_op land before; the TUI display arms land in 29)
4. `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` — Phase 31

Phase 28's compile boundary: variants exist in `Op` enum + `dispatch()` arms + `execute_op()` arms. The two `prgm_display.rs` files in CLI/GUI are NOT part of `hp41-core` compilation; they fail to compile when Phase 29/31 starts unless the new arms are added. The compile-time enforcement makes drift structurally impossible.

### Enum bloat regression guard (Pitfall 10)

`criterion bench/dispatch_overhead.rs` floor: < 200 ns/op. v2.2 baseline: 65 ns/op. Headroom: 3× — comfortable, but the bench MUST run as part of `just bench` and the result MUST be inspected in the PR description (advisory, not CI-gated per ROADMAP cross-cutting).

Plan 28-01 ships an updated baseline measurement in the PR body so subsequent plans (28-02..28-10) can be compared.

### `execute_op` vs `run_loop` split for callback ops

Per ARCHITECTURE.md guidance + STATE.md "Op variants land before TUI code":
- **`dispatch()` arms** for Integ/Solve/Difeq return `HpError::InvalidOp` if called outside a `run_loop` context (mirrors `Op::GtoInd` / `Op::XeqInd` precedent at line 839 of program.rs).
- **`run_loop` match block** holds the actual implementation that calls back into itself.
- **`execute_op` arms** for Integ/Solve/Difeq also return `HpError::InvalidOp` — they cannot be reached except via `run_loop`.

This three-place pattern was settled in v2.2 for XeqInd; Phase 28 reuses it.

---

## Validation Architecture

Phase 28 has `nyquist_validation_enabled = true` (per config.json `workflow.nyquist_validation: true`). RESEARCH.md must describe how plans should be validated per Dimension 8 (testability).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (workspace) + `criterion` (benches) + `approx 0.5.1` (dev-dep, relative-tolerance assertions) |
| Config file | `Cargo.toml` `[dev-dependencies]` + `[[bench]]` sections; per-test attributes inline |
| Quick run command | `cargo test -p hp41-core --lib --test xrom_shadowing --test math1_user_callback` |
| Full suite command | `just test` (workspace-wide) |
| Coverage gate | `just coverage` — ≥ 95% lines on `hp41-core` (Phase 32 / QUAL-01) |

### Phase Requirements → Test Map

| REQ-ID | Behavior | Test type | Automated command | File exists? |
|--------|----------|-----------|-------------------|-------------|
| XROM-04 | `xrom_resolve("SINH", 0b1)` returns `Some(Op::Sinh)` | unit | `cargo test -p hp41-core --lib xrom::tests` | ❌ Plan 28-01 |
| XROM-05 | Resolver fires LAST (xrom never shadows built-ins) | integration | `cargo test --test xrom_shadowing` | ❌ Plan 28-01 |
| XROM-08 | Nested INTG-in-SOLVE rejected at op entry | integration | `cargo test --test math1_user_callback nested_integ_inside_solve_rejected` | ❌ Plan 28-07 |
| XROM-09 | ModalProgram state machine writes to modal_prompt | unit | `cargo test --lib math1::modal::tests` | ❌ Plan 28-01 |
| HYP-01..06 | Hyperbolic ops + domain errors | unit, ≥ 5 cases each per Pitfall 16 | `cargo test --lib math1::hyperbolics::tests` | ❌ Plan 28-02 |
| CMPLX-05 | `op_c_div` with zero divisor returns DivideByZero BEFORE division | unit | `cargo test --lib math1::complex::tests cdiv_zero_divisor` | ❌ Plan 28-03 |
| CMPLX-06..17 | Complex functions + branch cuts + (0,0) handling | integration | `cargo test --test math1_complex_edge_cases` | ❌ Plan 28-04 |
| POLY-04 | `U=u`/`V=v`/`U=u`/`-V=-v` output format | unit + OM cite | `cargo test --lib math1::poly::tests output_format` | ❌ Plan 28-05 |
| POLY-06 | Multiplicity-as-cluster: `(x-1)^5` returns 5 clustered roots | unit | `cargo test --lib math1::poly::tests cluster_multiplicity` | ❌ Plan 28-05 |
| MAT-07 | `INV` singular detection at INV_EPSILON | unit, 3 cases (singular, near-singular, overflow) | `cargo test --lib math1::matrix::tests inv_*` | ❌ Plan 28-06 |
| INTG-08 | Convergence threshold tied to DisplayMode | unit, 2 cases (Fix(4) vs Fix(9), same integral) | `cargo test --lib math1::integ::tests threshold_*` | ❌ Plan 28-07 |
| SOLV-04 | Three termination paths (`NO ROOT FOUND` / `ROOT IS` / `ROOT IS BETWEEN`) | integration, ≥ 3 cases | `cargo test --test math1_solve_paths` | ❌ Plan 28-08 |
| All Math Pac I cases | ≥ 5 tests per new Op (Pitfall 16) | meta-test | `cargo test --test math1_op_test_count` | ❌ Phase 32 |
| Enum bloat | `dispatch_overhead < 200 ns/op` | criterion bench (advisory) | `cargo bench --bench dispatch_overhead` | ✅ extends v2.2 bench |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-core --lib` (~5–10 s)
- **Per wave merge:** `just test` (full workspace; ~30 s)
- **Phase gate:** Full suite green + `just coverage` ≥ 95% + benchmark within budget before `/gsd:verify-work`

### Wave 0 Gaps (files that must exist before Plan 28-02 implementation can start)

- [ ] `hp41-core/tests/xrom_shadowing.rs` — covers XROM-05 / Pitfall 1
- [ ] `hp41-core/tests/math1_user_callback.rs` — covers XROM-08 / Pitfall 4 (5 regression cases)
- [ ] `hp41-core/tests/math1_complex_edge_cases.rs` — covers Pitfall 6 ((0,0), zero-divisor branch cuts)
- [ ] `hp41-core/tests/math1_op_test_count.rs` — grep meta-test enforcing ≥ 5 tests per new Op (Pitfall 16 CI gate)
- [ ] `scripts/check-free42-contamination.sh` — Phase 32, but per-file header comments ship in every Plan 28-02..28-10 file from the start

### Pitfall-specific validation strategies

| Pitfall | Detection mechanism |
|---------|---------------------|
| **Pitfall 1** (shadowing) | `tests/xrom_shadowing.rs` integration test iterates `MATH_1.ops` and asserts `xeq_by_name_local_resolve(name).is_none() && builtin_card_op(name).is_none()` for every entry. CI-gated. |
| **Pitfall 4** (call_stack overflow) | Property test in `tests/math1_user_callback.rs`: random user-callback program of length 1–8 steps × random nesting depth 0–6; assert `call_stack.len() ≤ 4` AND on `>= 4` entry returns `HpError::CallDepth` |
| **Pitfall 10** (enum bloat) | `criterion bench/dispatch_overhead.rs` extension: bench `dispatch(Op::Sinh, ...)`, `dispatch(Op::CPlus, ...)`, `dispatch(Op::MatInv, ...)` etc. and assert per-op median < 200 ns. PR description prints baseline. |
| **Pitfall 16** (per-op test count) | `tests/math1_op_test_count.rs` grep meta-test: for each `Op::*` variant added in Plan 28-02..28-10, count `#[test]` functions in `hp41-core/tests/math1_*.rs` files mentioning that variant by name; assert ≥ 5. |
| **Pitfall 7 / Pitfall 2** (OM-transcribed constants) | The constants are exported as `pub const`. Tests in `numerical_accuracy.rs` import them by name; the test docstring carries `// Source: HP 00041-90034 p.<n>, ex.<m>` AND `// Free42 cross-check: integration.c outputs <Y> for <input>`. Pre-merge PR review verifies the citation. |

### Free42 cross-check protocol

For each Math Pac I numerical case in `numerical_accuracy.rs`:
1. Run the OM-cited input through Free42 (consult-only — see Pitfall 19 / QUAL-05)
2. Record Free42's output in the test docstring: `// Free42 v3.0.5: 1.1752 — agrees with OM`
3. The test asserts the emulator's output matches the OM-quoted value within `approx::assert_relative_eq!(max_relative = 1e-7)` (Pitfall 14)

NO Free42 source code is copied — algorithm derives from OM. Free42 is a **sanity-check oracle** only.

---

## Suggested Plan Structure

**Endorsement of ROADMAP's 10-plan breakdown:** YES — the build order (framework → hyperbolics → complex stack → complex functions → POLY → MATRIX → INTG → SOLVE → DIFEQ → FOUR/TRI/TRANS) is sound. Each plan introduces exactly one new architectural concept and validates it before the next plan ships.

### Plan-by-plan dependencies (refinement, not revision)

| Plan | Title | Critical dependencies | Adds to public surface |
|------|-------|-----------------------|------------------------|
| **28-01** | XROM framework + 5 ADRs | OM transcription complete (Decisions 3 & 4) | `xrom_resolve`, `MATH_1`, 7 new CalcState fields, `HpError::Canceled`, resolver chain extension at 3 call sites, `tests/xrom_shadowing.rs`, `tests/math1_user_callback.rs` scaffold |
| **28-02** | Hyperbolics | 28-01 framework, `prgm_display.rs` arms NOT YET added (Phase 29 will) | 6 Op variants, dispatch+execute_op arms only |
| **28-03** | Complex stack + arithmetic | 28-02 framework validated end-to-end | 4 Op variants (CPlus..CDiv) + `complex_atan2` helper + `ComplexHp` newtype + Op::Real (CMPLX-18 from D-28.3) |
| **28-04** | Complex functions | 28-03 arithmetic primitives | 13 Op variants (Magz..ZpowW) + branch-cut tests |
| **28-05** | POLY/ROOTS | 28-04 complex primitives (for complex root pairs) | 2 Op variants + `PolyInputStep` modal sub-state + Output format `U=u`/`V=v`/`U=u`/`-V=-v` |
| **28-06** | MATRIX | 28-05 modal pattern validated; INV_EPSILON from 28-01 | 8 Op variants + `MatrixInputStep` modal sub-state + `MatrixView` helper |
| **28-07** | INTG | 28-06 modal pattern, INTG threshold from 28-01, `run_loop` re-entrancy infrastructure | 1 Op variant + `IntegInputStep` modal sub-state + `IntegState` struct + user-callback infrastructure |
| **28-08** | SOLVE | 28-07 user-callback infrastructure | 2 Op variants (Solve, Sol) + `SolveInputStep` + `SolveState` |
| **28-09** | DIFEQ | 28-08 user-callback infrastructure + RK4 | 1 Op variant + `DifeqInputStep` + `DifeqState` (NEW state field) |
| **28-10** | FOUR + Triangles + TRANS | 28-09 user-callback (FOUR USER-mode E-key) | 8 Op variants (Four, TriSss..TriSsa, Trans2d, Trans3d) + `FourInputStep` + `TransInputStep` |

### Revision suggestion (minor): consider splitting 28-10

**Observation:** Plan 28-10 bundles 8 Op variants across 3 unrelated programs (FOUR, Triangles, TRANS). The other plans average 2–4 Ops with tight thematic coherence. 28-10 risks becoming a "rest-bucket" PR with weak focus.

**Suggested revision:** split into 28-10a (FOUR), 28-10b (Triangles), 28-10c (TRANS) — three smaller PRs. This raises the plan count to 12 but each plan stays focused.

**Counter-argument:** ROADMAP's 10-plan structure was set by `/gsd:roadmapper` after seeing the full milestone scope; the three differentiator-tier programs in 28-10 are each smaller in scope than any single 28-05/06/07 plan. Bundling them keeps the milestone shape symmetric (10 plans matches v2.2's Phase-25 10-plan shape).

**Recommendation:** keep ROADMAP's 10-plan structure UNLESS plan-checker (next step) flags 28-10 as oversized. The thematic-coherence argument is real but secondary to milestone symmetry.

---

## Risk Register

Top 5 risks ranked by likelihood × impact.

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **OM transcription drift** — Decision 3 (INV-EPSILON) or Decision 4 (INTG threshold) gets transcribed wrong; Plan 28-06 or 28-07 ships with wrong reference behavior; bug surfaces only at Phase 32 numerical_accuracy harness | MEDIUM | HIGH (full sub-plan rework) | **Plan 28-01 Task 1 = OM transcription**, verbatim quote into ADR-003 / ADR-004 PRIOR to any matrix/INTG code; Free42 cross-check before locking; PR-description includes OM page-and-paragraph quote |
| 2 | **User-callback re-entrancy edge case** — strict-reject policy missed some nested-call path (e.g. SOLVE-inside-DIFEQ-inside-INTG via subtle XEQ-by-name aliasing); program corrupts solver state silently | MEDIUM | HIGH (silent wrong answer, hardest bug class to detect) | 5-test regression suite (`math1_user_callback.rs`) lands in Plan 28-07 BEFORE INTG implementation; property test asserts `state.integ_state.is_some() \|\| state.solve_state.is_some() \|\| state.difeq_state.is_some()` triggers `InvalidOp` from any entry point; CI gate |
| 3 | **Modal-prompt UX surprise** — Phase 29 / Phase 31 wiring discovers `modal_prompt` channel can't cleanly compose with `PendingInput` exhaustive match; Phase 28's modal design needs amendment | LOW-MED | MED (Phase 29 plan rework, no hp41-core rebuild) | Plan 28-01 ships `tests/modal_program_transitions.rs` that exercises every `MatrixInputStep` / `SolveInputStep` / etc. step transition via direct state mutation. Phase 29 inherits a working state machine and only adds CLI rendering — clean handoff. |
| 4 | **`Arc<AtomicBool>` field breaks serde Default** — `CalcState::default()` fails to compile in some test crate that derives `Default` on a wrapper struct | LOW | LOW (compile error, easy to fix) | Plan 28-01 ships custom `default_cancel_requested()` fn; tests for `CalcState::default()` and `CalcState::new()` ship in same PR; clippy lint catches |
| 5 | **Free42 GPL contamination via OM-similar algorithm** — well-intentioned implementer copies Free42's variable names while "translating" the algorithm | LOW | HIGH (legal exposure, contamination audit gate fails) | Per-file header comment in every Plan 28-02..28-10 file from the start; pre-PR grep audit by reviewer; Phase 32 / QUAL-05 audit script catches missed cases. Reviewer rejects any PR where Free42 source was open in a window adjacent to the implementation file (operational discipline) |

### Lower-priority risks (documented but not in top 5)

- Cross-platform numerical drift (Pitfall 14) — mitigated by `approx::assert_relative_eq!(max_relative = 1e-7)` discipline; surfaces in Phase 32 CI
- Coverage drop mid-milestone (Pitfall 16) — mitigated by per-Op ≥ 5 test count meta-test
- Enum bloat regression (Pitfall 10) — `criterion bench dispatch_overhead` floor 200 ns; 3× headroom from v2.2 baseline
- Save-file mid-modal state corruption (Pitfall 12) — `#[serde(default, skip)]` on every transient field from Plan 28-01

---

## Open Questions for the Planner

1. **OM transcription artifact location.** Where does the transcribed OM text live? Options:
   - (a) Inline in `docs/adr/v3.0-003-inv-epsilon.md` and `docs/adr/v3.0-004-intg-threshold.md` (Phase 30 / DOC-07)
   - (b) Quoted in code comments in `hp41-core/src/ops/math1/matrix.rs` and `integ.rs`
   - (c) Both — code comment cites ADR; ADR has the full quote
   - **Recommendation:** (c). Code comments are short; ADR carries the full verbatim quote plus OM page-number scan attribution.

2. **`DifeqState` field timing.** The `<decisions>` block of CONTEXT.md mentions `integ_state` and `solve_state` as the new state fields. But DIFEQ also needs mid-RK4-iteration state. Two options:
   - (a) Land `difeq_state` field in Plan 28-01 alongside the others (early commitment)
   - (b) Land `difeq_state` in Plan 28-09 when DIFEQ implementation begins (just-in-time)
   - **Recommendation:** (a). The save-file backward-compat machinery is exercised cleanest with one field-addition commit, not three; the `#[serde(default, skip)]` discipline is the same. Adding (a) makes the XROM-08 strict-reject guard symmetric across all three solvers from the start.

3. **`MatrixView` location.** STACK.md Decision 3 suggests a `MatrixView<'a>` helper that borrows a slice of `state.regs` and exposes `get(r,c)` / `set(r,c, HpNum)`. Where does it live?
   - (a) `hp41-core/src/ops/math1/matrix.rs` (module-local)
   - (b) `hp41-core/src/matrix_view.rs` (crate-public helper)
   - **Recommendation:** (a) for v3.0; if v3.2+ Advanced Matrix Pac wants to reuse it, refactor to (b) then. YAGNI.

4. **`ModalProgram` exhaustive match in CLI Phase 29.** Phase 29's `pending_prompt()` will need a new exhaustive match over `ModalProgram` variants. Does Phase 28 ship a stub `pending_prompt_modal()` function in `hp41-core` (for unit testing) or does Phase 29 own that entirely?
   - **Recommendation:** Phase 28 ships a `ModalProgram::current_prompt() -> Option<&str>` method on the enum (pure-data accessor) — used by Phase 28 unit tests AND by Phase 29's `pending_prompt()` rendering. No CLI dependency.

5. **`complex_atan2` location.** It's a single helper used by MAGZ, LNZ, Z↑1/N, SinZ, CosZ, etc. Should it live in `hp41-core/src/num.rs` (alongside `checked_asin`) or in `hp41-core/src/ops/math1/complex.rs`?
   - **Recommendation:** `hp41-core/src/ops/math1/complex.rs`. Math Pac I scope; not used by v2.2 ops. Migration to `num.rs` is one-line if v3.x ever needs it.

6. **Per-program scratch register conflicts.** REQUIREMENTS says INTG uses R00–R07, SOLVE uses R00–R06, FOUR uses R00–R26, etc. If a user's program calls INTG and inside the user-callback executes `STO 03`, that clobbers INTG's scratch. Pitfall 4 documents this as a known issue but ADR-002 (strict-reject nested) doesn't address it. Does Plan 28-07 snapshot R00–R07 around each user-callback invocation, or document the clobber as user-responsibility?
   - **Recommendation:** Document as user-responsibility (matches real Math Pac I hardware behavior — the OM says "do not use registers R00–R07 in your user function while INTG is active"). Phase 28 ships an integration test `user_fn_stores_to_scratch_corrupts_integ` that asserts the documented behavior (wrong answer, no error) and `docs/hp41-math1-divergences.md` documents the user-responsibility convention.

---

## Sources

### Primary (HIGH confidence)
- HP-41C Math Pac Owner's Manual (00041-90034, 1979) — [hpcalc.org PDF](https://literature.hpcalc.org/community/hp41-pac-math-en.pdf) (17.8 MB)
- HP-41C Math Pac I Quick Reference Card (00041-90065, 1979) — [hpcalc.org PDF](https://literature.hpcalc.org/community/hp41-pac-math-qrc-en.pdf)
- `.planning/research/STACK.md`, `FEATURES.md`, `ARCHITECTURE.md`, `PITFALLS.md`, `SUMMARY.md` (Phase 28 research synthesis, dated 2026-05-16)
- `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md`, `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md`
- Direct codebase inspection: `hp41-core/src/{state.rs, error.rs, ops/mod.rs, ops/program.rs}`, `hp41-cli/src/keys.rs`

### Secondary (MEDIUM confidence)
- Museum of HP Calculators — [hpmuseum.org/software/soft41.htm](https://www.hpmuseum.org/software/soft41.htm), [hpmuseum.org/software/41/41matrix.htm](https://www.hpmuseum.org/software/41/41matrix.htm)
- HP-41 Archive — [hp41.org](http://www.hp41.org/)
- HP Journal April 1979 — Personal Programmable Calculator Routines (Romberg integration reference)
- Mike Sebastian's HP-41 forensics — cross-platform numerical drift data

### Tertiary (LOW confidence — consult only, never copy)
- Free42 source code (Thomas Okken, GPL) — [thomasokken.com/free42/](https://thomasokken.com/free42/) — used as sanity-check oracle only per Pitfall 19 / QUAL-05

---

## Metadata

**Confidence breakdown:**
- Standard stack (zero new runtime deps; `approx 0.5.1` dev-dep): **HIGH** — verified on crates.io 2026-05-16
- Architecture (Op-strategy A, resolver-LAST, run_loop re-entry, overlay complex stack): **HIGH** — locked decisions + carrying v2.2 patterns
- Modal-state-machine design: **HIGH** — patterned on v2.2 `PendingInput` hybrid struct-variants
- INV-EPSILON value: **LOW** — TBD pending OM transcription (Decision 3)
- INTG threshold formula: **LOW** — TBD pending OM transcription (Decision 4)
- Per-program algorithms (Simpson, secant, Gauss-Jordan, LU, RK4, Laguerre/Bairstow): **MEDIUM** — algorithms well-known, OM-citation-pending for fidelity gates
- Pitfall mitigations: **HIGH** — every critical pitfall (1, 2, 4, 5, 6, 7, 10, 11, 16) has a concrete CI gate or per-op test

**Research date:** 2026-05-16
**Valid until:** 2026-06-15 (30 days — Phase 28 is the gating phase; downstream phases inherit these decisions unchanged through Phase 32)

---

## RESEARCH COMPLETE

**Phase:** 28 — XROM Framework + Math Pac I Core Ops
**Confidence:** HIGH on architecture; MEDIUM on two TBD numerical constants (INV-EPSILON, INTG threshold) pending OM transcription in Plan 28-01 Task 1.

### Key Findings

- **Plan 28-01 must transcribe Owner's Manual pages for Decisions 3 & 4 BEFORE any implementation lands.** INV-EPSILON (matrix singularity threshold) and INTG convergence-threshold formula are the two non-locked irreversible decisions; both gate downstream plans (28-06 MATRIX, 28-07 INTG).
- **Resolver chain extension is a 3-site, well-bounded change.** `xrom_resolve` fires LAST in `op_xeq` (program.rs:506), `run_program::execute_op` (same fallback chain), and `xeq_by_name_local_resolve` (keys.rs:347). Pitfall 1 mitigation is structural via the `tests/xrom_shadowing.rs` CI gate.
- **Seven new `CalcState` fields, all `#[serde]`-disciplined.** `xrom_modules` (default 0b1), `complex_mode` (default false), `matrix_dim`/`matrix_active_reg` (both `Option`), `modal_program`/`modal_prompt` (both `#[serde(skip)]`), `integ_state`/`solve_state`/`cancel_requested` (all `#[serde(skip)]`). Save-file backward compat: v1.0–v2.2 load cleanly via `#[serde(default)]`.
- **No name-shadowing collisions identified between Math Pac I and v2.2 built-ins.** Verified by grep across `xeq_by_name_local_resolve` (8 conditional spellings only) and `builtin_card_op` (4 card-reader + 8 conditional only). `INV` (matrix inverse) is safe because v2.2's `Op::Inv` is a keyboard-routed reciprocal that doesn't go through XEQ-by-name. `tests/xrom_shadowing.rs` automates the assertion.
- **Open question for planner: split Plan 28-10 into 28-10a/b/c?** ROADMAP's 10-plan structure is sound; 28-10 bundles 8 Ops across FOUR/Triangles/TRANS but each is small. Recommendation: KEEP unless plan-checker flags as oversized.

### File Created
`/Users/daniel/GitRepository/hp41-calculator-emulator/.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-RESEARCH.md`

### Ready for Planning
Planner can now create the 10 Plan-28-XX PLAN.md files, beginning with 28-01 (framework + 5 ADRs + OM-transcription tasks).
