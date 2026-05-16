# Phase 24: Indirect Addressing (Cross-Cutting) — Research

**Researched:** 2026-05-14
**Domain:** Rust workspace refactor — shared resolver helper + Op-enum extension across `hp41-core`
**Confidence:** HIGH (entire phase is local-codebase work; CONTEXT.md already locked the design — research validates the implementation surface)

## Summary

Phase 24 ships a single private inner helper `resolve_indirect_decimal(state, reg) -> Result<Decimal, HpError>` plus its `pub fn resolve_indirect(state, reg) -> Result<u8, HpError>` u8-wrapper, then refactors the two Phase-22 inline indirect resolvers (`Op::GtoInd` / `Op::XeqInd`) to call the inner helper, then adds **12 new `*Ind` Op variants** that delegate to their existing direct-form counterparts. Every variant lands in 4 places (`dispatch()` + `execute_op()` + both `prgm_display.rs` copies). No new `CalcState` fields, no new `HpError` variants, no save-file breakage.

The work is straightforward — every load-bearing decision is already in CONTEXT.md (D-24.1..D-24.9). The remaining open questions are mechanical: where to put the inner helper file, whether `Op::StoArithInd` is one variant or four, and exactly which `to_string()` formatting to use for prgm_display. All decisions below are recommendations for the planner; none touch architectural invariants.

**Primary recommendation:** Create `hp41-core/src/ops/indirect.rs` (sibling to `flags.rs`, `display_ops.rs`) holding both `resolve_indirect_decimal` (private) and `resolve_indirect` (pub), plus the 12 `op_*_ind` shim functions. Single `Op::StoArithInd(u8, StoArithKind)` variant matching the existing `Op::StoArith` shape. `Op::FlagTestInd { kind, ind_reg: u8 }` per D-24.6. 12 new variants total. File size estimate: ~250–300 lines including unit tests.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Pointer validation (`regs[n] → u8`) | `hp41-core` (new `indirect.rs`) | — | Pure logic — no UI / no I/O. Lives next to its callers. |
| `Op::*Ind` variant declarations | `hp41-core/src/ops/mod.rs` (`Op` enum) | — | Single source of truth for the operation set. |
| `Op::*Ind` interactive dispatch | `hp41-core/src/ops/mod.rs::dispatch()` | — | All ops reach interactive dispatch via the central match. |
| `Op::*Ind` programmatic execution | `hp41-core/src/ops/program.rs::execute_op()` / inline `run_loop` | — | Programmable IND variants (most of them) need execute_op arms; `Op::FlagTestInd` is a `run_loop` inline arm like `Op::FlagTest`. |
| Listing display ("ARCL IND 12") | `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs` | — | Mirror copies — SC-4 invariant. Not a logic duplication. |
| CLI keyboard wiring | — | Phase 25 (FN-CLI-02) | Out of Phase 24 scope. |
| GUI key_map registration | — | Phase 26 (FN-GUI-01) | Out of Phase 24 scope. |

## Standard Stack

This is a refactor of an existing Rust workspace — no new dependencies. The stack already in use is sufficient.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rust_decimal` | 1.42 (workspace-pinned) | `Decimal::trunc()`, `Decimal::is_zero()`, `Decimal::to_i64()` (via `ToPrimitive`) | Already the BCD-accurate carrier through `HpNum` (CLAUDE.md "Settled Architecture Decisions"). `[VERIFIED: hp41-core/src/num.rs:3 imports `ToPrimitive`]` |
| `serde` + `serde_json` | workspace-pinned | `Op` enum derive `Serialize, Deserialize` for save-file round-trip | Phase 1 decision; CONTEXT.md confirms "no new `CalcState` fields → save-file backward compat preserved automatically." `[CITED: STATE.md "serde_json for persistence"]` |

### Supporting
None. No new `[dependencies]` entries needed in either `hp41-core/Cargo.toml` or root `Cargo.toml`.

### Alternatives Considered
None applicable. Adding a crate (e.g. `num-traits` for a more generic conversion) would violate the "minimal-deps" posture of `hp41-core`. The conversion chain `decimal.to_i64().ok_or(InvalidOp)? → u8::try_from(...).map_err(|_| InvalidOp)` uses only what's already imported — D-24.2 verbatim.

**Installation:** Not applicable.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FN-IND-01 | All addressable ops support indirect addressing — register-N integer part is the effective address | Confirmed implementable by 12 new `Op` variants delegating to existing direct-form ops; full variant set enumerated in §"Variant Set" below. |
| FN-IND-02 | Indirect resolution rejects non-integer register contents with `HpError::InvalidOp` | Direct copy of the existing Phase-22 inline pattern (program.rs:480-483: `pointer.trunc_int(); if int_part != pointer { return Err(InvalidOp); }`). Reused verbatim in `resolve_indirect_decimal`. |

## Project Constraints (from CLAUDE.md)

These constraints are non-negotiable for Phase 24:

- **Zero-panic in `hp41-core`:** `#![deny(clippy::unwrap_used)]` at crate root. Resolver path uses `.ok_or(InvalidOp)?` and `.map_err(|_| InvalidOp)?` only. Test modules carry `#[allow(clippy::unwrap_used)]`.
- **4-place Op-variant landing:** every new `*Ind` variant must appear in (1) `dispatch()` in `ops/mod.rs`, (2) `execute_op()` in `ops/program.rs` AND/OR `run_loop`, (3) `hp41-cli/src/prgm_display.rs::op_display_name`, (4) `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name`. Exhaustive matches will fail-compile if any place is forgotten.
- **English-only commit messages.**
- **MSRV 1.88** (`rust-version.workspace = true`). The `to_i64`/`try_from` chain is stable since Rust 1.34 — comfortably within MSRV. `[VERIFIED: rustc 1.95.0 in current env]`
- **`just` is the sole task runner.** Tests run via `just test`; coverage gate via `just coverage`. Never invoke `cargo` directly in CI/docs.
- **SC-4 invariant:** no `op_*` function bodies in `hp41-gui/src-tauri/src/`. Phase 24 only adds string formatters in `prgm_display.rs` — already on the documented exception list.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-24.1:** Two-tier resolver. Private `resolve_indirect_decimal(state: &CalcState, reg: u8) -> Result<Decimal, HpError>` is THE single source of pointer-validation truth. It performs (a) `state.regs.get(reg as usize).ok_or(InvalidOp)`, (b) `pointer.trunc_int()`, (c) `int_part != pointer → InvalidOp`. Both downstream wrappers consume this inner helper.
- **D-24.2:** Public signature locked: `pub fn resolve_indirect(state: &CalcState, reg: u8) -> Result<u8, HpError>`. Implementation: `let i = resolve_indirect_decimal(state, reg)?; u8::try_from(i.to_i64().ok_or(InvalidOp)?).map_err(|_| InvalidOp)`.
- **D-24.3:** Bounds responsibility = caller. The helper does NOT check resolved address against `regs.len()` or `< 56`. Each `*_ind` op DELEGATES to its existing direct counterpart with the resolved address.
- **D-24.4:** Sidecar / atomicity / lift-effect inherit via delegation — no replication.
- **D-24.5:** Phase-22 `Op::GtoInd` / `Op::XeqInd` get refactored onto the inner helper (with regression sentinel in 24-01). The `find_in_program` and `call_stack.len() >= 4 → CallDepth` paths are NOT touched.
- **D-24.6:** `Op::FlagTestInd { kind: FlagTestKind, ind_reg: u8 }` (struct variant, mirrors `Op::FlagTest { kind, flag }`).
- **D-24.7:** Naming pattern `<Name>Ind(u8)`. Total ~12 new variants.
- **D-24.8:** Two plans (24-01 Foundation, 24-02 Variants). Wave-sequential.
- **D-24.9:** File overlap (`ops/mod.rs`, `ops/program.rs`) forces wave-sequential.

### Claude's Discretion

- **Inner-helper module location:** `ops/indirect.rs` recommended; planner may collapse into `ops/mod.rs` if too small.
- **`op_sto_arith_ind` arity:** single `Op::StoArithInd(u8, StoArithKind)` (mirrors `StoArith`) vs four flat variants. Default = single with kind reuse.
- **Regression-test scaffolding location:** 24-01 holds GTO/XEQ-IND regression; 24-02 holds the new IND-variant suite. Planner may add a small `phase24_helper.rs` if inner-helper unit tests grow large.

### Deferred Ideas (OUT OF SCOPE)

- Proptest sweep for `resolve_indirect` → Phase 27 (Test Hardening)
- Keyboard wiring for IND prompts in `hp41-cli` → Phase 25 (CLI Integration)
- GUI `key_map.rs` registration for IND variants → Phase 26 (GUI Polish)
- HP-41CV function matrix entry for each IND variant → Phase 25 (FN-DOC-01..04)
- Indirect-of-indirect (`STO IND IND nn`) — HP-41 hardware does not support it; permanently out of scope.

## Architecture Patterns

### Inner-Helper File Location

**Recommendation: create new file `hp41-core/src/ops/indirect.rs`.**

Sizing the file:

| Section | Estimated lines |
|---------|----------------:|
| Module header / imports | ~10 |
| `resolve_indirect_decimal` (with rustdoc) | ~30 |
| `resolve_indirect` (with rustdoc) | ~15 |
| 12 × `op_*_ind` shim functions @ avg ~6 lines each | ~75 |
| `#[cfg(test)] mod tests` (helper unit tests + boundary cases for `to_i64` / `u8::try_from`) | ~120 |
| **Total** | **~250 lines** |

**Why a new file (not collapse into `mod.rs`):**

- `mod.rs` is already 1028 lines and is the dispatch hub. Adding 250 lines of indirect-specific code there pushes it past 1250 and harms findability — mirrors the precedent of every other op family living in its own file (`flags.rs`, `display_ops.rs`, `alpha.rs`, etc.).
- 12 shims + a 30-line resolver naturally cluster as a "module" in their own right.
- `mod.rs` remains the dispatch table only.

**Module wiring (planner adds to `ops/mod.rs`):**

```rust
pub mod indirect;
// ...
use indirect::{
    op_arcl_ind, op_asto_ind, op_cf_flag_ind, op_dse_ind, op_flag_test_ind,
    op_isg_ind, op_rcl_ind, op_sf_flag_ind, op_sto_arith_ind, op_sto_ind, op_view_ind,
    resolve_indirect, resolve_indirect_decimal,
};
```

`resolve_indirect` is `pub` (tests reach it from `hp41-core/tests/phase24_resolve_indirect.rs`). `resolve_indirect_decimal` is `pub(crate)` (only consumed by the same-crate `op_gto_ind` / `op_xeq_ind` refactor in `program.rs`). The `op_*_ind` shims are `pub(crate)` — `dispatch()` and `execute_op()` are the only callers.

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Caller (interactive or program)                │
│                                                                      │
│  dispatch(state, Op::StoInd(5))   or    execute_op(state, Op::...)   │
└─────────────────────────┬────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│  ops/mod.rs::dispatch()  /  ops/program.rs::execute_op()             │
│  Op::StoInd(reg) => op_sto_ind(state, reg)                           │
└─────────────────────────┬────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│  ops/indirect.rs::op_sto_ind(state, reg)                             │
│    let addr = resolve_indirect(state, reg)?;                         │
│    op_sto(state, addr)                                               │
└─────────────────────────┬─────────────────────────────────┬──────────┘
                          │                                 │
                          ▼                                 ▼
┌───────────────────────────────────────┐   ┌────────────────────────────┐
│  ops/indirect.rs::resolve_indirect    │   │  ops/registers.rs::op_sto  │
│    let i = resolve_indirect_decimal?  │   │  (existing direct form     │
│    u8::try_from(i.to_i64().ok_or(?))? │   │   with bounds + sidecar    │
└──────────────┬────────────────────────┘   │   + atomicity already in   │
               │                            │   place)                   │
               ▼                            └────────────────────────────┘
┌──────────────────────────────────────┐
│  ops/indirect.rs::                   │
│    resolve_indirect_decimal          │
│  state.regs.get(reg).ok_or(?)?       │
│  pointer.trunc_int() == pointer ?    │
│  → return Decimal pointer            │
└──────────────────────────────────────┘
                          ▲
                          │  (also called by)
┌─────────────────────────┴────────────────────────────────────────────┐
│  ops/program.rs::run_loop                                            │
│  Op::GtoInd(reg) => {                                                │
│    let i = resolve_indirect_decimal(state, *reg)?;                   │
│    let label_str = i.inner().to_string();                            │
│    let target = find_in_program(program, &label_str)?;               │
│    state.pc = target + 1;                                            │
│  }                                                                   │
│  Op::XeqInd(reg) => same + pre-mutation call_stack guard             │
└──────────────────────────────────────────────────────────────────────┘
```

### Pattern 1: Two-Tier Resolver (D-24.1)

**What:** A private inner helper validates the pointer (returns `Decimal`); a public wrapper converts to `u8`. Both downstream consumers (Phase 24 IND-variants and refactored Phase 22 GTO/XEQ-IND) call the inner helper directly to avoid u8↔Decimal round-trips.

**Why:** GTO/XEQ-IND uses the integer pointer as a label name (stringified, then `find_in_program`), not as a register address. A `u8` would conflate "register-index semantic" with "label-lookup semantic". The Decimal preserves "is an integer pointer" as the only invariant.

**Example:**

```rust
// hp41-core/src/ops/indirect.rs (NEW)

use crate::error::HpError;
use crate::state::CalcState;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Validate that `state.regs[reg]` holds an integer pointer; return its Decimal.
///
/// Failure modes (all return `HpError::InvalidOp`):
/// - `reg` is out of range for `state.regs`
/// - the register holds a non-integer value (fractional part non-zero)
///
/// This is the SINGLE source of pointer-validation truth in `hp41-core`.
/// Both `resolve_indirect` (u8 wrapper, used by Phase 24 IND variants) and
/// the refactored Phase-22 `Op::GtoInd` / `Op::XeqInd` arms in
/// `ops/program.rs::run_loop` call this helper directly.
pub(crate) fn resolve_indirect_decimal(
    state: &CalcState,
    reg: u8,
) -> Result<Decimal, HpError> {
    let pointer = state
        .regs
        .get(reg as usize)
        .ok_or(HpError::InvalidOp)?
        .clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer {
        return Err(HpError::InvalidOp);
    }
    Ok(int_part.inner())
}

/// Resolve register `reg`'s integer-part contents into a `u8` register address.
///
/// Two cascading rejection paths, both `HpError::InvalidOp`:
/// 1. `resolve_indirect_decimal` — non-integer pointer (FN-IND-02).
/// 2. `to_i64` / `u8::try_from` — pointer doesn't fit in `u8` (e.g. R05 = 300).
///
/// Per D-24.3, this helper does NOT check the resolved address against
/// `state.regs.len()` (regs ops) or `< 56` (flag ops). Bounds enforcement
/// is the caller's responsibility — the existing direct-form ops
/// (`op_sto`, `op_sf`, etc.) already do it via `.get().ok_or(InvalidOp)?`.
pub fn resolve_indirect(state: &CalcState, reg: u8) -> Result<u8, HpError> {
    let i = resolve_indirect_decimal(state, reg)?;
    let as_i64 = i.to_i64().ok_or(HpError::InvalidOp)?;
    u8::try_from(as_i64).map_err(|_| HpError::InvalidOp)
}
```

### Pattern 2: 2-Line Delegation Shim (D-24.3 / D-24.4)

**What:** Each `op_*_ind` is a 2-line function that resolves the indirect address, then calls the matching direct-form op.

**Example:**

```rust
// hp41-core/src/ops/indirect.rs (NEW)

use crate::ops::registers::{op_rcl, op_sto, op_sto_arith};
use crate::ops::flags::{op_cf, op_sf};
use crate::ops::alpha::{op_arcl, op_asto};
use crate::ops::display_ops::op_view;
use crate::ops::program::{op_dse, op_isg};
use crate::ops::{FlagTestKind, StoArithKind};

/// STO IND nn — store X into register pointed to by R[nn]'s integer part.
/// Delegates to `op_sto` after `resolve_indirect`. Inherits Neutral lift,
/// the D-23.4 sidecar-clearing on `text_regs`, and the D-22.11.1 bounds check.
pub(crate) fn op_sto_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_sto(state, addr)
}

/// RCL IND nn — recall register R[reg] pointed by R[nn]. Delegates to op_rcl.
/// Inherits Enable lift.
pub(crate) fn op_rcl_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_rcl(state, addr)
}

/// ISG IND nn — increment register pointed to by R[nn]; skip if exit.
/// Returns Result<bool, HpError> like `op_isg` so run_loop can act on the skip.
pub(crate) fn op_isg_ind(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_isg(state, addr)
}
// ... (DSE-IND, SF-IND, CF-IND, ARCL-IND, ASTO-IND, VIEW-IND identical pattern)

/// STO+/-/×/÷ IND nn — arithmetic via R[nn]'s integer-part address.
pub(crate) fn op_sto_arith_ind(
    state: &mut CalcState,
    reg: u8,
    kind: StoArithKind,
) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_sto_arith(state, addr, kind)
}

/// FS?/FC?/FS?C/FC?C IND — flag-test via R[nn]'s integer-part flag index.
/// Skip semantics live in run_loop (mirror of `Op::FlagTest`).
/// Returns the resolved flag for run_loop's match arm.
pub(crate) fn op_flag_test_ind_resolve(
    state: &CalcState,
    ind_reg: u8,
) -> Result<u8, HpError> {
    resolve_indirect(state, ind_reg)
}
```

> Note on `Op::FlagTestInd`: the dispatch path is more subtle than a simple shim because the existing `Op::FlagTest` is a `run_loop` inline arm (not a function call) — its skip-next-step + always-clear behavior lives in `program.rs:563-582`. The IND variant follows the same pattern: `run_loop` resolves the indirect register first, then runs the same inline match. Interactive dispatch is a Neutral no-op, identical to the direct form.

### Pattern 3: Phase-22 Refactor Diff Sketch (D-24.5)

**What:** Replace the duplicated trunc+InvalidOp logic in `Op::GtoInd` and `Op::XeqInd` with a call to `resolve_indirect_decimal`. The `find_in_program` lookup, `state.pc = target + 1` advance, and `call_stack.len() >= 4 → CallDepth` guard are NOT touched.

**Before (program.rs:474-487 — `Op::GtoInd`):**

```rust
Op::GtoInd(reg) => {
    let pointer = state
        .regs
        .get(reg as usize)
        .ok_or(HpError::InvalidOp)?
        .clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer {
        return Err(HpError::InvalidOp);
    }
    let label_str = int_part.inner().to_string();
    let target = find_in_program(program, &label_str)?;
    state.pc = target + 1; // mirrors Op::Gto: pc → step AFTER LBL marker
}
```

**After (program.rs:474-481 — `Op::GtoInd`):**

```rust
Op::GtoInd(reg) => {
    // Phase 24 (D-24.5): pointer-validation logic now lives in the shared
    // `resolve_indirect_decimal` helper — single source of truth across all
    // ~14 indirect-resolving callers (this + Op::XeqInd + 12 Op::*Ind variants).
    let i = crate::ops::indirect::resolve_indirect_decimal(state, *reg)?;
    let label_str = i.to_string();
    let target = find_in_program(program, &label_str)?;
    state.pc = target + 1; // mirrors Op::Gto: pc → step AFTER LBL marker
}
```

**Before (program.rs:500-517 — `Op::XeqInd`):**

```rust
Op::XeqInd(reg) => {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth); // pre-mutation atomicity
    }
    let pointer = state
        .regs
        .get(reg as usize)
        .ok_or(HpError::InvalidOp)?
        .clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer {
        return Err(HpError::InvalidOp);
    }
    let label_str = int_part.inner().to_string();
    let target = find_in_program(program, &label_str)?;
    state.call_stack.push(state.pc);
    state.pc = target + 1;
}
```

**After (program.rs:500-510 — `Op::XeqInd`):**

```rust
Op::XeqInd(reg) => {
    // Pre-mutation atomicity guard (D-22.15) — UNCHANGED, must run before
    // any pointer read.
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }
    // Phase 24 (D-24.5): shared pointer-validation helper. The label-lookup
    // path (find_in_program + call_stack.push + pc advance) is NOT modified —
    // GTO/XEQ-IND resolve to LABELS, not register addresses, so the inner
    // Decimal-returning helper (not the u8 wrapper) is the right consumer.
    let i = crate::ops::indirect::resolve_indirect_decimal(state, *reg)?;
    let label_str = i.to_string();
    let target = find_in_program(program, &label_str)?;
    state.call_stack.push(state.pc);
    state.pc = target + 1;
}
```

**Diff invariants preserved:**

1. `find_in_program(program, &label_str)` — call signature, error path, and `state.pc = target + 1` advance unchanged.
2. `call_stack.len() >= 4 → CallDepth` runs BEFORE pointer read — pre-mutation atomicity (D-22.15) preserved by keeping it as the first check in the `XeqInd` arm.
3. `int_part.inner().to_string()` becomes `i.to_string()` — `i` is already a `Decimal` (the helper returns `int_part.inner()`). One method call shorter, semantically identical: `Decimal::to_string` produces the same output as `Decimal::to_string` either way.
4. No `builtin_card_op` fallback in either arm — never had one (per existing code comment at program.rs:497-499). Indirect labels are numeric strings only.

### Anti-Patterns to Avoid

- **Anti-pattern: pushing bounds checks into `resolve_indirect`.** Two op categories have different limits (`regs.len()` for STO/RCL/etc., `< 56` for SF/CF/FlagTest). Centralizing bounds would force two helpers and violate the ROADMAP "single shared resolver" principle. D-24.3 forbids it.
- **Anti-pattern: replicating sidecar-clearing in `op_sto_ind`.** `op_sto` already clears `text_regs` (D-23.4 — registers.rs:25). Delegation inherits it; replication would create drift risk.
- **Anti-pattern: stringifying via `format!("{i}")` instead of `i.to_string()`.** Both work, but `to_string()` matches the existing Phase-22 idiom (`int_part.inner().to_string()` at program.rs:484, 513) and avoids a hidden `Display` allocation. Use `to_string()` for parity.
- **Anti-pattern: returning `HpNum` from `resolve_indirect_decimal`.** `Decimal::to_string()` is what `find_in_program` needs. Wrapping in `HpNum` only to call `.inner()` later is needless ceremony.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pointer validation (non-integer rejection) | A new `is_integer_pointer()` method on `HpNum` | Inline `int_part != pointer` comparison after `trunc_int()` | Phase 22 already established the idiom; introducing a method just for this one site adds API surface for no benefit. |
| Out-of-u8-range rejection | A custom enum mapping `Decimal` → `Result<u8, ...>` | `decimal.to_i64().ok_or(InvalidOp)? → u8::try_from(...).map_err(\|_\| InvalidOp)?` | D-24.2 locked this exact chain. `to_i64` is from the already-imported `ToPrimitive` trait. |
| Bounds check on resolved address | Re-checking `< regs.len()` in each `op_*_ind` shim | Delegate to direct-form op — it already checks | D-24.3 / D-24.4. Direct ops carry the bounds-check + sidecar + atomicity invariants through `op_sto`/`op_sf`/etc. Replicating is drift risk. |
| New `HpError` variant for "pointer out of u8 range" | `HpError::PointerRange` | Reuse `HpError::InvalidOp` | CONTEXT.md "no new HpError variants in Phase 24". Both non-integer AND out-of-u8-range collapse to `InvalidOp` per ROADMAP. |
| Generic-over-T trait `IndirectAddressable` | Bounds-via-traits abstraction | Concrete shim functions per op | 12 shims at ~6 lines each = 75 lines. A trait abstraction would be ~150 lines and require generic-bounds plumbing for non-uniform return types (`op_isg` returns `Result<bool, _>`, others return `Result<(), _>`). Concrete is simpler. |

**Key insight:** The whole phase is "delegation, not duplication". Every `op_*_ind` is a 2-liner. Every difficult invariant (bounds, sidecar, atomicity, lift effect) already lives in the direct-form op and inherits gratis through delegation.

## Variant Set (Recount of D-24.7 estimate)

The CONTEXT.md estimate was "~12 new variants". After cross-checking the ROADMAP-required addressable op list against the existing `Op` enum, the count is **exactly 12** (assuming single-variant `Op::StoArithInd` per Claude's-Discretion default).

| # | New Variant | Direct Counterpart | Direct-Op Function | Programmable? | run_loop arm? |
|---|-------------|---------------------|---------------------|--------------:|:-------------:|
| 1 | `Op::StoInd(u8)` | `Op::StoReg(u8)` | `op_sto` (registers.rs:16) | yes | no — execute_op |
| 2 | `Op::RclInd(u8)` | `Op::RclReg(u8)` | `op_rcl` (registers.rs:33) | yes | no — execute_op |
| 3 | `Op::StoArithInd(u8, StoArithKind)` | `Op::StoArith { reg, kind }` | `op_sto_arith` (registers.rs:53) | yes | no — execute_op |
| 4 | `Op::IsgInd(u8)` | `Op::Isg(u8)` | `op_isg` (program.rs:106) | yes | **YES — skip-next** |
| 5 | `Op::DseInd(u8)` | `Op::Dse(u8)` | `op_dse` (program.rs:119) | yes | **YES — skip-next** |
| 6 | `Op::SfFlagInd(u8)` | `Op::SfFlag(u8)` | `op_sf` (flags.rs:39) | yes | no — execute_op |
| 7 | `Op::CfFlagInd(u8)` | `Op::CfFlag(u8)` | `op_cf` (flags.rs:50) | yes | no — execute_op |
| 8 | `Op::FlagTestInd { kind: FlagTestKind, ind_reg: u8 }` | `Op::FlagTest { kind, flag }` | inline in `run_loop` (program.rs:563-582) | yes | **YES — skip-next + always-clear** |
| 9 | `Op::ArclInd(u8)` | `Op::Arcl(u8)` | `op_arcl` (alpha.rs:79) | yes | no — execute_op |
| 10 | `Op::AstoInd(u8)` | `Op::Asto(u8)` | `op_asto` (alpha.rs:119) | yes | no — execute_op |
| 11 | `Op::ViewInd(u8)` | `Op::View(u8)` | `op_view` (display_ops.rs:17) | yes | no — execute_op |
| — | (`Op::GtoInd(u8)` and `Op::XeqInd(u8)` already exist from Phase 22 — REFACTORED, not added) | — | — | yes | YES (already wired) |

**Confirmation: 11 brand-new variants + 1 reused (FlagTestKind enum) — total 12 enum lines added to `Op`** (the `StoArithInd` is one line carrying a `(u8, StoArithKind)` payload).

> **Note:** The list does NOT include `Op::StoArithStackInd` because `Op::StoArithStack` operates on `StackReg::{Y, Z, T, LastX}` (stack registers, not numbered regs). Indirect addressing through `StackReg` is meaningless — there are no register addresses to indirect through. CONTEXT.md "Out of scope: IND on stack-arithmetic targets" makes this explicit.

### 4-Place Landing Checklist

For each new variant, the planner must add an entry in all four of:

| # | New Variant | dispatch arm (mod.rs) | execute_op arm (program.rs) | hp41-cli prgm_display | hp41-gui prgm_display |
|---|-------------|------------------------|------------------------------|------------------------|------------------------|
| 1 | `Op::StoInd(reg)` | `Op::StoInd(reg) => op_sto_ind(state, reg)` | `Op::StoInd(reg) => op_sto_ind(state, reg)` | `Op::StoInd(r) => format!("STO IND {r:02}")` | same |
| 2 | `Op::RclInd(reg)` | `Op::RclInd(reg) => op_rcl_ind(state, reg)` | same | `Op::RclInd(r) => format!("RCL IND {r:02}")` | same |
| 3 | `Op::StoArithInd(reg, kind)` | `Op::StoArithInd(reg, kind) => op_sto_arith_ind(state, reg, kind)` | same | see "StoArithInd formatting" below | same |
| 4 | `Op::IsgInd(reg)` | `Op::IsgInd(reg) => op_isg_ind(state, reg).map(\|_\| ())` (drop bool — same pattern as Op::Isg at mod.rs:687-691) | run_loop inline arm: `Op::IsgInd(reg) => { if op_isg_ind(state, reg)? { state.pc += 1; } }` PLUS execute_op arm: also drop the bool with `.map(\|_\| ())` | `Op::IsgInd(r) => format!("ISG IND {r:02}")` | same |
| 5 | `Op::DseInd(reg)` | analogous to IsgInd | analogous (run_loop arm + execute_op arm) | `Op::DseInd(r) => format!("DSE IND {r:02}")` | same |
| 6 | `Op::SfFlagInd(reg)` | `Op::SfFlagInd(reg) => op_sf_flag_ind(state, reg)` | same | `Op::SfFlagInd(r) => format!("SF IND {r:02}")` | same |
| 7 | `Op::CfFlagInd(reg)` | `Op::CfFlagInd(reg) => op_cf_flag_ind(state, reg)` | same | `Op::CfFlagInd(r) => format!("CF IND {r:02}")` | same |
| 8 | `Op::FlagTestInd { kind, ind_reg }` | Neutral no-op (mirrors Op::FlagTest at mod.rs:745-748: `apply_lift_effect(Neutral); Ok(())`) | run_loop inline arm: resolve `ind_reg` → flag, then re-use the existing `Op::FlagTest` skip-next + always-clear logic verbatim. NOT in execute_op (mirrors Op::FlagTest's catch-all rejection at program.rs:833) | `Op::FlagTestInd { kind, ind_reg } => { let mn = match kind {…}; format!("{mn} IND {ind_reg:02}") }` | same |
| 9 | `Op::ArclInd(reg)` | `Op::ArclInd(reg) => op_arcl_ind(state, reg)` | same | `Op::ArclInd(r) => format!("ARCL IND {r:02}")` | same |
| 10 | `Op::AstoInd(reg)` | `Op::AstoInd(reg) => op_asto_ind(state, reg)` | same | `Op::AstoInd(r) => format!("ASTO IND {r:02}")` | same |
| 11 | `Op::ViewInd(reg)` | `Op::ViewInd(reg) => op_view_ind(state, reg)` | same | `Op::ViewInd(r) => format!("VIEW IND {r:02}")` | same |

**Compile-time enforcement:** every match in the four files is exhaustive (no `_ =>` catch-all anywhere except the program.rs:825-840 "programming ops handled by run_loop" block, which deliberately uses `|`-pattern enumeration to fail-compile when a new programming variant is missed). Forgetting any of the 4 places will fail-compile.

### `Op::StoArithInd` Arity Decision (Claude's Discretion)

**Recommendation: single variant `Op::StoArithInd(u8, StoArithKind)`** mirroring the existing `Op::StoArith { reg, kind }` shape exactly.

**Comparison:**

| Approach | New variants | dispatch arm count | prgm_display body |
|---|---:|---:|---|
| **Single (recommended)** | 1 | 1 | One match arm reusing the same `StoArithKind` → symbol mapping that `Op::StoArith` already has (lines 82-90 in hp41-cli/src/prgm_display.rs). Display string: `format!("STO{op_sym} IND {reg:02}")`. |
| Four flat | 4 (`StoAddInd` / `StoSubInd` / `StoMulInd` / `StoDivInd`) | 4 | Four 1-liner match arms: `Op::StoAddInd(r) => format!("STO+ IND {r:02}")`, etc. |

**Single-variant `op_display_name` body (cli + gui — identical):**

```rust
Op::StoArithInd(reg, kind) => {
    let op_sym = match kind {
        StoArithKind::Add => "+",
        StoArithKind::Sub => "-",
        StoArithKind::Mul => "\u{00D7}",
        StoArithKind::Div => "\u{00F7}",
    };
    format!("STO{op_sym} IND {reg:02}")
}
```

**Four-flat-variant alternative (cli + gui — identical):**

```rust
Op::StoAddInd(r) => format!("STO+ IND {r:02}"),
Op::StoSubInd(r) => format!("STO- IND {r:02}"),
Op::StoMulInd(r) => format!("STO\u{00D7} IND {r:02}"),
Op::StoDivInd(r) => format!("STO\u{00F7} IND {r:02}"),
```

**Why single wins:**

1. **Mirror precedent:** `Op::StoArith { reg, kind }` is single-variant; `Op::StoArithInd` matching that shape minimizes cognitive load.
2. **Single dispatch shim:** `op_sto_arith_ind(state, reg, kind)` is one function passing both arguments through to `op_sto_arith(state, addr, kind)`. Four-flat would require either four separate dispatch shims OR a private helper that takes a kind parameter — re-deriving the single-variant shape.
3. **Equal verbosity in prgm_display:** the single-variant approach reuses the same kind-to-symbol mapping table. Four-flat has 4 lines instead of 1 match block — slight prgm_display win for four-flat, but ledgered against new-variant cost.
4. **No save-file impact** (Phase 24 adds variants — never reaches old saves).

**Counter-argument for four-flat:** `Op::StoArith { reg, kind }` uses struct-variant syntax, but `StoArithInd` would be tuple-variant (matching the `<Name>Ind(u8)` D-24.7 pattern). Some readers find tuple-variants with two payloads (`(u8, StoArithKind)`) less self-documenting than struct-variants. **Counter to that counter:** `Op::Asn { name, key_code }` is struct-variant, `Op::Tone(u8)` is tuple — both shapes coexist in the codebase already. Either is acceptable.

**Final recommendation: single `Op::StoArithInd(u8, StoArithKind)`.** If the planner prefers struct-variant for clarity, `Op::StoArithInd { reg: u8, kind: StoArithKind }` is also acceptable — same dispatch shim, same prgm_display body, same delegation. The choice is purely stylistic.

## Common Pitfalls

### Pitfall 1: Forgetting the `run_loop` arm for `Op::IsgInd` / `Op::DseInd` / `Op::FlagTestInd`

**What goes wrong:** ISG/DSE return `Result<bool, HpError>` because the bool drives skip-next-step inside `run_loop`. `Op::Isg` has BOTH a `run_loop` arm (program.rs:549-553 — uses the bool) AND a `dispatch()` arm (mod.rs:687-691 — discards the bool). If `Op::IsgInd` is wired only into `dispatch()` and execute_op via `.map(|_| ())`, the skip semantic is silently lost inside running programs — ISG IND would always behave as "increment; continue" instead of "increment; skip if exit".

**Why it happens:** Three of the 12 new variants (`IsgInd`, `DseInd`, `FlagTestInd`) have skip-next-step semantics that live in `run_loop`, not `execute_op`. The same was true for their direct counterparts in Phase 21/22, but the planner could miss this when adding the IND variants.

**How to avoid:** For `Op::IsgInd`, `Op::DseInd`, `Op::FlagTestInd`, add a `run_loop` arm that mirrors the existing direct-counterpart arm. The `execute_op` arm is also required (defense-in-depth — it returns InvalidOp via the existing `Op::Isg(_) | Op::Dse(_) | Op::FlagTest { .. } => Err(InvalidOp)` catch-all at program.rs:830-833). Add the new IND variants to that same `|`-pattern.

**Warning signs:** Sentinel test "ISG IND inside a program causes loop exit on counter limit" must FAIL before the run_loop arm is added, then PASS after. If it passes pre-implementation, the test is wrong (probably never entered run_loop).

### Pitfall 2: Decimal stringification edge cases for negative pointers

**What goes wrong:** `Decimal::from(-3).to_string() == "-3"`. `find_in_program` searches for an `Op::Lbl(name)` where `name == "-3"` — labels can technically contain any String. A user storing `-3` in R05 and calling `GTO IND 05` would silently look for "LBL -3", and either find it (weird but valid) or fail with `InvalidOp` from `find_in_program`. This is the existing Phase-22 behavior — Phase 24 must preserve it through the refactor.

**Why it happens:** The shared helper returns the Decimal as-is (no sign-stripping); stringification is the caller's choice. The Decimal-returning inner helper preserves sign exactly as Phase 22 does today.

**How to avoid:** Add a sentinel regression test exercising a negative-pointer GTO IND case (R05 = -3, no LBL "-3" → expect `InvalidOp` from `find_in_program`). Confirms behavior is byte-for-byte equivalent post-refactor.

**Warning signs:** Any new behavior on negative pointers vs Phase 22 baseline indicates the refactor introduced a regression.

### Pitfall 3: `to_i64` overflow on `Decimal` values exceeding `i64::MAX`

**What goes wrong:** A register holding `Decimal::from_str("99999999999999999999")` (20 digits — well within HpNum's 28-digit Decimal range but past i64) returns `None` from `to_i64()`. `resolve_indirect` correctly maps that to `InvalidOp`. But the test coverage for this branch is awkward — you have to hand-construct a `Decimal` outside the i64 range and route it through `state.regs[n]` (which only holds `HpNum`-rounded values).

**Why it happens:** `HpNum::rounded` does NOT clamp magnitude — only significant-digit count. A Decimal with mantissa 1e25 is preserved post-rounding. The `to_i64` failure is a real branch but rarely hit in normal use.

**How to avoid:** Test it explicitly with an artificially-constructed value. See Test Matrix below — the `out_of_i64_range_rejects` test is hand-constructible with `Decimal::from_str("18446744073709551616")` (2^64) wrapped in `HpNum::rounded`.

**Warning signs:** Coverage report shows the `to_i64().ok_or(InvalidOp)?` branch as uncovered. Add the explicit test or the coverage gate slips.

### Pitfall 4: `prgm_display` formatting for `Op::FlagTestInd { kind, ind_reg }` — variable mnemonic prefix

**What goes wrong:** Easy to write `format!("{kind} IND {ind_reg:02}")` and miss that `FlagTestKind` doesn't impl `Display` — only the existing `op_display_name` branch (lines 158-166 in hp41-cli/src/prgm_display.rs) maps each kind to its mnemonic explicitly. Forgetting to mirror that mapping in the IND variant produces compile errors at best, wrong listings at worst (e.g. always showing "IsSet IND 05" instead of "FS? IND 05").

**Why it happens:** Struct-variant prgm_display arms are slightly more verbose than tuple-variant arms — easy to copy-paste the wrong template.

**How to avoid:** Reuse the existing kind-to-mnemonic mapping verbatim. Pattern:

```rust
Op::FlagTestInd { kind, ind_reg } => {
    let mnemonic = match kind {
        FlagTestKind::IsSet => "FS?",
        FlagTestKind::IsClear => "FC?",
        FlagTestKind::IsSetThenClear => "FS?C",
        FlagTestKind::IsClearThenClear => "FC?C",
    };
    format!("{mnemonic} IND {ind_reg:02}")
}
```

This block is identical in `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs` (SC-4 mirror).

**Warning signs:** Test asserting `op_display_name(&Op::FlagTestInd { kind: FlagTestKind::IsSetThenClear, ind_reg: 5 }) == "FS?C IND 05"` fails or doesn't exist.

### Pitfall 5: Forgetting to `pub use` re-exports in `lib.rs`

**What goes wrong:** `resolve_indirect` is documented as a Phase-24 deliverable that integration tests in `hp41-core/tests/phase24_resolve_indirect.rs` will reach via `hp41_core::ops::indirect::resolve_indirect`. If `pub mod indirect` isn't added to `ops/mod.rs`, the test path doesn't resolve.

**Why it happens:** The other op modules (`alpha`, `flags`, `display_ops`) are exposed as `pub mod` in mod.rs:9-21, so the pattern is established — but a fresh module is easy to add as `mod` (private) by accident.

**How to avoid:** Plan 24-01 task list explicitly includes "add `pub mod indirect;` to ops/mod.rs alongside the other family modules". The unit-test-from-tests/-integration target will fail-compile if forgotten, surfacing the omission immediately.

**Warning signs:** `error[E0433]: failed to resolve: could not find indirect in ops` in test output.

### Pitfall 6: Missing `Op::*Ind` from the run_loop "programming-ops catch-all"

**What goes wrong:** `execute_op` ends with a `|`-enumerated catch-all rejecting programming ops that should never reach it (program.rs:825-840). The new `Op::IsgInd`, `Op::DseInd`, `Op::FlagTestInd` belong on that catch-all (defense-in-depth). Forgetting to add them does NOT fail-compile (the exhaustive match is satisfied by the explicit arms above), but adds a silent fall-through if someone later removes the explicit arm — fragile.

**Why it happens:** `|`-enumeration is text-additive, not type-checked.

**How to avoid:** When writing the execute_op arms for IsgInd/DseInd/FlagTestInd, immediately add them to the `|`-list at the bottom in the same commit. Pattern matches existing precedent for Op::Isg/Dse/FlagTest.

**Warning signs:** None at compile time. A test that calls `execute_op(state, Op::IsgInd(5))` directly and expects `InvalidOp` will catch the regression.

## Code Examples

### Inner helper (full body)

```rust
// hp41-core/src/ops/indirect.rs

//! Phase 24 indirect-addressing helpers and op shims.
//!
//! The two-tier resolver (D-24.1):
//!   - `resolve_indirect_decimal` is the SINGLE source of pointer-validation
//!     truth in `hp41-core`. It validates that `state.regs[reg]` is in range,
//!     reads the value, truncates to integer, and rejects non-integer pointers.
//!   - `resolve_indirect` wraps the inner helper for the common case where
//!     callers want a `u8` register / flag address (FN-IND-01 / FN-IND-02).
//!
//! All ~12 Phase-24 `op_*_ind` shims in this module are 2-line delegations:
//! resolve the indirect address, then call the existing direct-form op. The
//! direct ops carry bounds-checking (D-22.11.1), sidecar-clearing (D-23.4),
//! atomicity (compute-then-write), and per-op lift effects — all of which
//! Phase 24 inherits gratis through delegation (D-24.4).

use crate::error::HpError;
use crate::ops::alpha::{op_arcl, op_asto};
use crate::ops::display_ops::op_view;
use crate::ops::flags::{op_cf, op_sf};
use crate::ops::program::{op_dse, op_isg};
use crate::ops::registers::{op_rcl, op_sto, op_sto_arith};
use crate::ops::StoArithKind;
use crate::state::CalcState;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

pub(crate) fn resolve_indirect_decimal(
    state: &CalcState,
    reg: u8,
) -> Result<Decimal, HpError> {
    let pointer = state
        .regs
        .get(reg as usize)
        .ok_or(HpError::InvalidOp)?
        .clone();
    let int_part = pointer.trunc_int();
    if int_part != pointer {
        return Err(HpError::InvalidOp);
    }
    Ok(int_part.inner())
}

pub fn resolve_indirect(state: &CalcState, reg: u8) -> Result<u8, HpError> {
    let i = resolve_indirect_decimal(state, reg)?;
    let as_i64 = i.to_i64().ok_or(HpError::InvalidOp)?;
    u8::try_from(as_i64).map_err(|_| HpError::InvalidOp)
}

pub(crate) fn op_sto_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_sto(state, addr)
}

pub(crate) fn op_rcl_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_rcl(state, addr)
}

pub(crate) fn op_sto_arith_ind(
    state: &mut CalcState,
    reg: u8,
    kind: StoArithKind,
) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_sto_arith(state, addr, kind)
}

pub(crate) fn op_isg_ind(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_isg(state, addr)
}

pub(crate) fn op_dse_ind(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_dse(state, addr)
}

pub(crate) fn op_sf_flag_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let flag = resolve_indirect(state, reg)?;
    op_sf(state, flag)
}

pub(crate) fn op_cf_flag_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let flag = resolve_indirect(state, reg)?;
    op_cf(state, flag)
}

pub(crate) fn op_arcl_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_arcl(state, addr)
}

pub(crate) fn op_asto_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_asto(state, addr)
}

pub(crate) fn op_view_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_view(state, addr)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn resolve_indirect_happy_integer_pointer() {
        let mut state = CalcState::new();
        state.regs[5] = HpNum::from(42i32);
        assert_eq!(resolve_indirect(&state, 5).unwrap(), 42u8);
    }

    #[test]
    fn resolve_indirect_non_integer_rejects() {
        let mut state = CalcState::new();
        state.regs[5] = HpNum::rounded(Decimal::from_str("12.345").unwrap());
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_reg_out_of_range_rejects() {
        let state = CalcState::new();
        // CalcState::new() ships 100 regs; reg=200 is out-of-range.
        assert!(matches!(
            resolve_indirect(&state, 200),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_pointer_exceeds_u8_range_rejects() {
        let mut state = CalcState::new();
        // 300 fits in i64 but not u8 — must reject via try_from path.
        state.regs[5] = HpNum::from(300i32);
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_pointer_exceeds_i64_range_rejects() {
        let mut state = CalcState::new();
        // 2^64 is well outside i64::MAX — must reject via to_i64 path
        // (the OTHER ?-arm in resolve_indirect, branch coverage).
        state.regs[5] = HpNum::rounded(
            Decimal::from_str("18446744073709551616").unwrap(),
        );
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_negative_integer_pointer_rejects_via_u8() {
        let mut state = CalcState::new();
        // -3 is an integer (passes the inner helper) but doesn't fit in u8.
        state.regs[5] = HpNum::from(-3i32);
        assert!(matches!(
            resolve_indirect(&state, 5),
            Err(HpError::InvalidOp)
        ));
    }

    #[test]
    fn resolve_indirect_decimal_preserves_sign_for_gto_ind_callers() {
        // Inner helper returns the Decimal as-is; sign preservation is
        // observable to the GtoInd / XeqInd refactor sites.
        let mut state = CalcState::new();
        state.regs[5] = HpNum::from(-3i32);
        let d = resolve_indirect_decimal(&state, 5).unwrap();
        assert_eq!(d.to_string(), "-3");
    }
}
```

### `Op::IsgInd` run_loop arm (program.rs)

```rust
// Inside run_loop, alongside the existing Op::Isg arm at program.rs:549-553

Op::IsgInd(reg) => {
    if crate::ops::indirect::op_isg_ind(state, reg)? {
        state.pc += 1; // loop exit: skip next (mirrors Op::Isg)
    }
}
Op::DseInd(reg) => {
    if crate::ops::indirect::op_dse_ind(state, reg)? {
        state.pc += 1; // loop exit: skip next (mirrors Op::Dse)
    }
}
```

### `Op::FlagTestInd` run_loop arm (program.rs)

```rust
// Inside run_loop, after the existing Op::FlagTest arm at program.rs:563-582

Op::FlagTestInd { kind, ind_reg } => {
    use crate::ops::flags::{flag_clear, flag_get};
    use crate::ops::FlagTestKind;
    let flag = crate::ops::indirect::resolve_indirect(state, ind_reg)?;
    let is_set = flag_get(state.flags, flag);
    let should_skip = match kind {
        FlagTestKind::IsSet => !is_set,
        FlagTestKind::IsClear => is_set,
        FlagTestKind::IsSetThenClear => {
            state.flags = flag_clear(state.flags, flag);
            !is_set
        }
        FlagTestKind::IsClearThenClear => {
            state.flags = flag_clear(state.flags, flag);
            is_set
        }
    };
    if should_skip {
        state.pc += 1;
    }
}
```

> Note: `op_sf` / `op_cf` enforce `flag <= 55` and return `InvalidOp` for anything higher — but `Op::FlagTestInd` doesn't go through them. The flag-bounds check happens implicitly via `flag_get`/`flag_clear` (flags.rs:12-35: out-of-range indices return `false` / are no-op). This matches the direct `Op::FlagTest` arm at program.rs:563-582, which also never bounds-checks `flag` — relies on the same defensive `flag_get`/`flag_clear` semantics. **Inheritance: identical to direct form.**

## Sentinel Regression Test Design (D-24.5)

**Location:** Add to existing `hp41-core/tests/phase22_program_control.rs` (preferred — keeps Phase-22 GTO/XEQ-IND coverage in one file) OR create a new `hp41-core/tests/phase24_phase22_refactor_sentinel.rs` if planner prefers strict per-phase organization. **Recommendation: extend the existing file with a clearly-marked Phase-24 section.**

The existing file already contains 5 tests covering GTO IND / XEQ IND happy path + reject paths (lines 222-350). Phase 24 must prove these tests still pass after the refactor — that's the primary regression check. Additional sentinel tests document the cross-call-site invariant.

**Required new tests (add 4 total):**

| # | Test name | Inputs | Expected | Why |
|---|-----------|--------|----------|-----|
| 1 | `phase24_gto_ind_uses_shared_helper` | R05 = 42, program: `[Lbl A, GtoInd 5, Lbl 42, PushNum 7]` | X = 7 (same as existing `test_gto_ind_happy`) | Sanity — proves the refactored arm still routes to `find_in_program`. Identical to existing test, kept under Phase-24 name for traceability. |
| 2 | `phase24_xeq_ind_uses_shared_helper` | R03 = 10, program: `[Lbl A, XeqInd 3, PushNum 2, Rtn, Lbl 10, PushNum 99, Rtn]` | X = 2 after RTN unwinding (same as existing `test_xeq_ind_happy`) | Sanity for XeqInd. |
| 3 | `phase24_xeq_ind_call_depth_guard_runs_before_pointer_read` | R03 = 12.345 (non-integer pointer), `state.call_stack = [999; 4]`, program: `[XeqInd 3, Lbl "12"]`, drive via `resume_program` | `Err(HpError::CallDepth)` (NOT `InvalidOp`) | **Critical sentinel** — proves the pre-mutation atomicity guard (D-22.15) STILL runs first after the refactor. If a planner accidentally moves the `call_stack.len() >= 4` check after the pointer read, this test catches it: `InvalidOp` would surface from the non-integer pointer instead of `CallDepth`. |
| 4 | `phase24_gto_ind_negative_pointer_stringifies_with_sign` | R05 = -3, program: `[Lbl A, GtoInd 5]` (no LBL "-3") | `Err(HpError::InvalidOp)` from `find_in_program` (NOT from non-integer rejection) | Pitfall 2 sentinel — confirms the `Decimal::to_string()` path still preserves negative-pointer behavior. Pre-refactor, the same case takes the same code path. If the refactor accidentally pre-validates "non-negative", this catches it. |

**Test header (add to phase22_program_control.rs):**

```rust
// ── Phase 24 D-24.5 sentinel: refactored GtoInd/XeqInd onto shared helper ────
//
// These tests exercise the same code paths as the original Phase-22 tests
// above, but assert specific invariants of the shared-helper refactor:
// (1) call-depth guard still runs FIRST (pre-mutation atomicity)
// (2) Decimal::to_string preserves sign for label-lookup callers
//
// If any of these tests fail, the refactor regressed Phase-22 behavior.
```

**Existing tests must continue to pass unchanged.** They are the primary regression suite; the new tests are sentinels for the specific concerns the refactor introduces.

## Per-Variant Test Coverage Matrix (24-02)

**File:** `hp41-core/tests/phase24_ind_variants.rs` (new). Plus inline `#[cfg(test)] mod tests` block in `ops/indirect.rs` for helper unit tests (already specified above in "Code Examples").

For each of the 11 new variants, three test classes are required:

| Test class | Pattern | Covers |
|------------|---------|--------|
| **Happy path** | Set up a register pointer, call the IND op, assert the same effect as the direct op | FN-IND-01: indirect resolution works |
| **Non-integer rejection** | Pointer = 12.345, call IND op, assert `InvalidOp` | FN-IND-02: non-integer rejection |
| **Out-of-bounds rejection** | Pointer = 200 (> regs.len()=100) for regs ops, OR pointer = 60 (> 55) for flag ops, assert `InvalidOp` | D-24.3 inheritance: direct-op bounds checks fire on resolved address |

Full coverage matrix (33 tests minimum):

| Variant | Happy path test | Non-integer test | Out-of-bounds test | Notes |
|---------|----------------|-------------------|---------------------|-------|
| `Op::StoInd(reg)` | `sto_ind_happy`: R05=12, X=7, dispatch StoInd(5), assert regs[12]=7 | `sto_ind_non_integer`: R05=12.5, dispatch StoInd(5), assert InvalidOp | `sto_ind_out_of_regs_len`: R05=200, dispatch StoInd(5), assert InvalidOp | Bonus test: sidecar inheritance (set text_regs[12]="ABC", then sto_ind, assert text_regs[12] cleared) |
| `Op::RclInd(reg)` | `rcl_ind_happy`: R05=12, regs[12]=99, dispatch RclInd(5), assert X=99 | `rcl_ind_non_integer`: same shape | `rcl_ind_out_of_regs_len`: same shape | Lift inheritance: assert lift_enabled=true after (Enable inherited from op_rcl) |
| `Op::StoArithInd(reg, Add)` | `sto_arith_ind_add_happy`: R05=12, regs[12]=10, X=3, dispatch StoArithInd(5,Add), assert regs[12]=13 | `sto_arith_ind_non_integer`: same | `sto_arith_ind_out_of_regs_len`: same | Repeat for Sub/Mul/Div (4 happy-path tests) — minimal verification of kind reuse |
| `Op::IsgInd(reg)` | `isg_ind_inside_run_loop`: R05=12, regs[12]="0.005" (counter), program `[Lbl A, IsgInd 5, Gto A, Lbl END]`, assert loop completes after counter exhaustion | `isg_ind_non_integer`: R05=12.5, IsgInd inside program, assert program returns InvalidOp | `isg_ind_out_of_regs_len`: same | **CRITICAL:** test runs inside `run_program`, not interactive dispatch — proves the run_loop arm wires the bool→skip semantic |
| `Op::DseInd(reg)` | `dse_ind_inside_run_loop`: similar to ISG | `dse_ind_non_integer`: same | `dse_ind_out_of_regs_len`: same | run_loop only |
| `Op::SfFlagInd(reg)` | `sf_flag_ind_happy`: R05=12, dispatch SfFlagInd(5), assert flag 12 set | `sf_flag_ind_non_integer`: same | `sf_flag_ind_out_of_flag_range`: R05=60 (>55), dispatch SfFlagInd(5), assert InvalidOp from op_sf | bounds inherited from op_sf:40-42 |
| `Op::CfFlagInd(reg)` | `cf_flag_ind_happy`: R05=12, set flag 12, dispatch CfFlagInd(5), assert flag 12 cleared | same | same | bounds inherited from op_cf:51-53 |
| `Op::FlagTestInd { kind, ind_reg }` | `flag_test_ind_is_set_happy_inside_run_loop`: R05=12, set flag 12, program `[Lbl A, FlagTestInd{IsSet,5}, PushNum 1, PushNum 2]`, assert X=2 (no skip) | `flag_test_ind_non_integer`: program with R05=12.5, assert InvalidOp from run_program | `flag_test_ind_high_flag_no_panic`: R05=100, program runs, no panic — flag_get returns false defensively | **CRITICAL:** test runs inside run_program. Repeat the happy test for IsClear/IsSetThenClear/IsClearThenClear (4 sub-tests minimum) — kind reuse coverage |
| `Op::ArclInd(reg)` | `arcl_ind_happy`: R05=12, regs[12]=42.0, alpha empty, dispatch ArclInd(5), assert alpha contains formatted "42.0000" (current display mode) | same | `arcl_ind_out_of_regs_len`: R05=200, assert InvalidOp from op_arcl's leading bounds check | text_regs sidecar inheritance: also test ARCL IND of an ASTO'd register (text_regs hit) |
| `Op::AstoInd(reg)` | `asto_ind_happy`: R05=12, alpha="HELLO", dispatch AstoInd(5), assert text_regs[12]="HELLO" AND regs[12]==zero (no-drift invariant) | same | `asto_ind_out_of_regs_len`: same | atomicity inheritance: failed AstoInd doesn't leave partial state |
| `Op::ViewInd(reg)` | `view_ind_happy`: R05=12, regs[12]=42, dispatch ViewInd(5), assert display_override is Some("42.0000") (formatted via current display_mode — D-24 §"specifics" — VIEW IND shows the resolved register's VALUE, not the pointer) | same | `view_ind_out_of_regs_len`: R05=200, assert InvalidOp from op_view | display_override semantics inherited from op_view |

**Total minimum new tests in `phase24_ind_variants.rs`:** 33 (11 × 3) + 6 sub-tests for kind reuse on FlagTestInd / StoArithInd ≈ **39 tests**.

Plus 7 inline unit tests in `ops/indirect.rs` (happy / non-integer / reg-out-of-range / pointer > u8 / pointer > i64 / negative-via-u8 / negative-Decimal-preservation).

Plus 4 new sentinel tests in `phase22_program_control.rs`.

**Grand total Phase 24 new tests:** ~50.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Inline `pointer.trunc_int(); if int_part != pointer { InvalidOp }` duplicated per-op | Single `resolve_indirect_decimal` helper | This phase (Phase 24) | One source of pointer-validation truth. Future "and accept up to 6 digits" or "reject NaN-equivalent" changes touch one site instead of 14. |
| Op variants documented as "Phase 22 ships an inline resolver — Phase 24 will extract" (mod.rs:368-370) | Refactored onto shared helper | Phase 24 | The placeholder comment in `Op::GtoInd`'s rustdoc was always going to be removed; this phase honors the promise. |

**Deprecated/outdated:** None. All Phase 22 GTO/XEQ-IND functionality is preserved; only the implementation is consolidated.

## Save-File Backward Compatibility Audit

**Confirmed: Phase 24 adds zero `CalcState` fields.**

The serializable surface that affects save files is `CalcState` (state.rs) — Phase 24 touches only the `Op` enum, which appears inside `CalcState.program: Vec<Op>` and `CalcState.assignments: HashMap<u8, String>` (the latter is unaffected — assignments hold String labels, not Op variants).

**Backward compatibility analysis:**

| Concern | Status |
|---------|--------|
| Older save files contain old `CalcState` schema | ✅ No change — all existing `Op` variants serialize identically; new variants don't appear in old saves |
| Older save files contain `Op` variants that Phase 24 might rename or restructure | ✅ No renames or restructures — Phase 24 is purely additive (new variants added; existing variants untouched) |
| Forward compat: an older `hp41-cli` reading a save written by Phase-24-ready code | ⚠️ NOT supported. An older binary loading a save with `Op::StoInd(5)` will fail serde deserialization with "unknown variant `StoInd`". This matches the existing Phase 22 / Phase 23 contract — saves are forward-compatible only within the same major-binary version. Documented behavior; not a Phase-24-specific concern. |
| `Op::FlagTest { kind, flag: 5 }` in v2.0/v2.1/v2.2 saves | ✅ Continues to deserialize unchanged. The new `Op::FlagTestInd { kind, ind_reg }` is a separate variant with a different serde tag — no collision. |

**Confirmed by code inspection:** the `Op` enum derives `#[derive(...Serialize, Deserialize)]` (mod.rs:102). Adding new variants is additive; serde tags each variant by name. No `#[serde(rename = "...")]` overrides or `#[serde(other)]` catch-all exists on `Op` — so old variants serialize/deserialize bit-identically post-Phase-24.

**Recommendation:** No save-file migration needed. The planner does not need to ship a "v2.2 save format upgrader" task.

## Coverage Estimate

**Current `hp41-core` coverage:** 92.5% lines (post-Phase 21; STATE.md row "92.68% lines (Phase 21)"). v2.2 target: 95% by Phase 27. Phase 24 should not regress this number.

**Lines added by Phase 24:**

| File | Lines added | Branches |
|------|------------:|----------|
| `ops/indirect.rs` (new) | ~250 (incl. tests) | resolver: 2 ?-arms in `resolve_indirect` + 2 in `resolve_indirect_decimal` = 4 error branches; happy path = 1. 11 shims = ~11 happy + 11 error branches (resolved via the helper). |
| `ops/mod.rs` (dispatch arms) | ~12 lines (one match arm per new variant + minor imports) | 12 happy paths (delegations) |
| `ops/program.rs` (execute_op arms + run_loop arms + GtoInd/XeqInd refactor) | ~30 lines (8 execute_op shim arms + 3 run_loop arms for IsgInd/DseInd/FlagTestInd + 2 refactored GtoInd/XeqInd arms - inline removed) | 8 execute_op happy paths + 3 run_loop happy + 3 run_loop skip-true branches |
| `hp41-cli/src/prgm_display.rs` | ~12 match arms | trivial — covered by `op_display_name` tests in same file |
| `hp41-gui/src-tauri/src/prgm_display.rs` | ~12 match arms | trivial — covered by mirror tests |

**Project impact estimate:**

- All new code paths in `indirect.rs` are explicitly tested (39 integration tests + 7 unit tests).
- `dispatch()` and `execute_op()` arms for IND variants are covered transitively by the integration tests.
- `prgm_display.rs` match arms add ~24 trivial lines per file; each needs at least one assertion test (extend the existing `test_display_phase22_op_labels` pattern, e.g., `test_display_phase24_ind_op_labels` with one assert per new variant).

**Hard-to-cover branches:**

1. **`to_i64().ok_or(InvalidOp)?` failure** — only fires for pointers exceeding i64::MAX. Test `resolve_indirect_pointer_exceeds_i64_range_rejects` (specified above) hand-constructs a 20-digit Decimal to hit this branch. Without this explicit test, this would be uncovered.
2. **`u8::try_from(...).map_err(|_| InvalidOp)?` failure for negative integer pointers** — covered by `resolve_indirect_negative_integer_pointer_rejects_via_u8` (specified above).
3. **`Op::FlagTestInd` interactive dispatch (no-op)** — easy to forget to test because the happy path lives in run_loop. Add `test_flag_test_ind_interactive_is_neutral_no_op`: dispatch `Op::FlagTestInd { kind: IsSet, ind_reg: 5 }` outside run_program, assert `Ok(())` and no state mutation.

**Coverage projection:** Adding ~250 lines of well-tested helpers + ~50 lines of trivially-covered dispatch/display arms should produce a net **0 to +0.5%** coverage delta. The phase is unlikely to regress the 92.5% baseline, and may marginally improve it through the focused unit-test density in `indirect.rs`.

**Coverage-gate recommendation:** Run `just coverage` post-Phase-24 (Wave 2 completion) to confirm. Phase 27 raises the gate to 95% — if Phase 24 leaves headroom under 95%, that's expected; closing the remaining gap is Phase 27's mandate.

## Validation Architecture

> Required by `workflow.nyquist_validation = true` in `.planning/config.json`. Section is extracted into VALIDATION.md by the validation tooling.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust stdlib test harness; integration tests in `hp41-core/tests/`, unit tests in `#[cfg(test)] mod tests` blocks) |
| Config file | None — `[lib]` and `[[test]]` sections of `hp41-core/Cargo.toml` |
| Quick run command | `just test -p hp41-core --test phase24_resolve_indirect --test phase24_ind_variants` (per-task — runs only the new test files; ~2 seconds) |
| Full suite command | `just test` (runs all `hp41-core` integration tests + unit tests + workspace tests; ~30 seconds) |
| Coverage command | `just coverage` (cargo-llvm-cov, gate at 80% for hp41-core; 92.5% baseline, post-Phase-24 target ≥92.5%) |

### Phase Requirements → Test Map

Resolution-path coverage (verifies the helper itself):

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FN-IND-01 | `resolve_indirect` with R05=42 returns Ok(42) | unit | `just test -p hp41-core --lib indirect::tests::resolve_indirect_happy_integer_pointer` | ❌ Wave 1 (24-01) |
| FN-IND-01 | `resolve_indirect` with R05=200 returns Err(InvalidOp) | unit | `just test -p hp41-core --lib indirect::tests::resolve_indirect_pointer_exceeds_u8_range_rejects` | ❌ Wave 1 (24-01) |
| FN-IND-02 | `resolve_indirect` with R05=12.345 returns Err(InvalidOp) | unit | `just test -p hp41-core --lib indirect::tests::resolve_indirect_non_integer_rejects` | ❌ Wave 1 (24-01) |
| FN-IND-02 | `resolve_indirect_decimal` for negative integer pointer preserves sign for GTO IND callers | unit | `just test -p hp41-core --lib indirect::tests::resolve_indirect_decimal_preserves_sign_for_gto_ind_callers` | ❌ Wave 1 (24-01) |

Regression coverage (proves Phase-22 GTO/XEQ-IND behavior unchanged):

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FN-PROG-06 (existing) | `Op::GtoInd(reg)` happy + non-integer + out-of-range still pass after refactor | integration | `just test -p hp41-core --test phase22_program_control` | ✅ existing — must not regress |
| FN-PROG-07 (existing) | `Op::XeqInd(reg)` happy + non-integer + out-of-range + 4-deep guard still pass after refactor | integration | `just test -p hp41-core --test phase22_program_control` | ✅ existing — must not regress |
| (sentinel) | `Op::XeqInd` call-depth guard runs BEFORE pointer-read (atomicity invariant) | integration | `just test -p hp41-core --test phase22_program_control phase24_xeq_ind_call_depth_guard_runs_before_pointer_read` | ❌ Wave 1 (24-01) |
| (sentinel) | `Op::GtoInd` with negative pointer stringifies with sign | integration | `just test -p hp41-core --test phase22_program_control phase24_gto_ind_negative_pointer_stringifies_with_sign` | ❌ Wave 1 (24-01) |

Variant coverage (verifies each new IND op):

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FN-IND-01 | `Op::StoInd(5)` with R05=12 stores X into regs[12] | integration | `just test -p hp41-core --test phase24_ind_variants sto_ind_happy` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::RclInd(5)` with R05=12 recalls regs[12] into X | integration | `just test -p hp41-core --test phase24_ind_variants rcl_ind_happy` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::StoArithInd(5, Add)` performs indirect addition | integration | `just test -p hp41-core --test phase24_ind_variants sto_arith_ind_add_happy` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::IsgInd(5)` skip-next semantic fires inside run_program | integration | `just test -p hp41-core --test phase24_ind_variants isg_ind_inside_run_loop` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::DseInd(5)` skip-next semantic fires inside run_program | integration | `just test -p hp41-core --test phase24_ind_variants dse_ind_inside_run_loop` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::SfFlagInd(5)` sets the indirected flag | integration | `just test -p hp41-core --test phase24_ind_variants sf_flag_ind_happy` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::CfFlagInd(5)` clears the indirected flag | integration | `just test -p hp41-core --test phase24_ind_variants cf_flag_ind_happy` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::FlagTestInd { IsSet, 5 }` skip-next semantic inside run_program | integration | `just test -p hp41-core --test phase24_ind_variants flag_test_ind_is_set_happy_inside_run_loop` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::ArclInd(5)` appends formatted regs[12] to alpha | integration | `just test -p hp41-core --test phase24_ind_variants arcl_ind_happy` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::AstoInd(5)` packs alpha into regs[12]'s text shadow | integration | `just test -p hp41-core --test phase24_ind_variants asto_ind_happy` | ❌ Wave 2 (24-02) |
| FN-IND-01 | `Op::ViewInd(5)` shows VALUE of regs[12] (not pointer) on display | integration | `just test -p hp41-core --test phase24_ind_variants view_ind_happy` | ❌ Wave 2 (24-02) |
| FN-IND-02 | Every IND variant rejects non-integer pointer with InvalidOp | integration | `just test -p hp41-core --test phase24_ind_variants` (the `*_non_integer` family) | ❌ Wave 2 (24-02) |
| (display) | `op_display_name(Op::*Ind)` returns the documented mnemonic for each new variant | unit | `just test -p hp41-cli prgm_display::tests::test_display_phase24_ind_op_labels` AND `just test -p hp41-gui-tauri prgm_display::tests::test_display_phase24_ind_op_labels` | ❌ Wave 2 (24-02) |

### Sampling Rate
- **Per task commit:** `just test -p hp41-core --lib indirect::tests` (Wave 1) or `just test -p hp41-core --test phase24_ind_variants --test phase24_resolve_indirect` (Wave 2) — ~2 seconds
- **Per wave merge:** `just test -p hp41-core` — ~10 seconds (includes phase22_program_control sentinels)
- **Phase gate:** `just test` (full workspace) + `just coverage` (≥80% gate, currently 92.5%) green before `/gsd-verify-work`

### Wave 0 Gaps

(In GSD parlance, "Wave 0" = pre-implementation scaffolding. Phase 24 has minimal Wave-0 work because the test infrastructure already exists.)

- [ ] `hp41-core/tests/phase24_resolve_indirect.rs` — new file, holds GTO/XEQ-IND regression sentinels (Wave 1, plan 24-01)
- [ ] `hp41-core/tests/phase24_ind_variants.rs` — new file, holds 33+ variant tests (Wave 2, plan 24-02)
- [ ] Inline `#[cfg(test)] mod tests` block in `ops/indirect.rs` — created alongside the helper module (Wave 1, plan 24-01)
- [ ] `phase24_xeq_ind_call_depth_guard_runs_before_pointer_read` and `phase24_gto_ind_negative_pointer_stringifies_with_sign` added to existing `phase22_program_control.rs` (Wave 1, plan 24-01)
- [ ] No new test framework, no new dependencies, no shared fixture file needed.

### Dimensions Satisfied

The Validation Architecture above satisfies four Nyquist dimensions:

| Dimension | Coverage |
|-----------|----------|
| **Resolution path coverage** | Every error branch in `resolve_indirect` / `resolve_indirect_decimal` has an explicit unit test (4 ?-arms × dedicated tests). |
| **Regression coverage** | Pre-existing Phase-22 GTO/XEQ-IND test suite must continue passing — that's the primary refactor-safety check. New sentinels (`*_call_depth_guard_runs_before_pointer_read`, `*_negative_pointer_stringifies_with_sign`) lock in invariants the refactor must not break. |
| **Variant coverage** | 11 new Op variants × 3 test classes (happy / non-integer / out-of-bounds) = 33 tests minimum. Plus `prgm_display.rs` mirror-display tests. |
| **Inheritance coverage** | Sidecar (D-23.4), atomicity (D-22.x), lift-effect inheritance is verified by spot-check tests (e.g. text_regs cleared after `Op::StoInd`; lift_enabled=true after `Op::RclInd`). |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` (rustc) | Build / test | ✓ | 1.95.0 | — |
| `just` | All recipes | ✓ (per CLAUDE.md) | n/a | — |
| `cargo-llvm-cov` | Coverage gate | ✓ (per CLAUDE.md "v1.0 shipped" — used by `just coverage`) | n/a | — |
| `rust_decimal` | `Decimal::to_i64`, `trunc`, etc. | ✓ (workspace dep) | 1.42 | — |
| `serde` / `serde_json` | `Op` enum derive + save-file round-trip tests | ✓ (workspace dep) | n/a | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Coverage delta will be 0 to +0.5% — Phase 24 will not regress the 92.5% baseline | "Coverage Estimate" | Low. If coverage regresses, Phase 27 (Test Hardening) will catch and address it. Phase 24 itself is not gated on a higher percentage. |
| A2 | `Op::FlagTestInd` interactive dispatch can mirror `Op::FlagTest`'s Neutral no-op exactly | "Pattern 2 / Variant Set #8" | Very low. The direct `Op::FlagTest` arm at mod.rs:745-748 establishes the precedent: "interactive dispatch is a no-op (mirrors `Op::Test` precedent — no next program step at the keyboard)". The IND variant inherits the same rationale. |
| A3 | `Decimal::to_string` produces identical output to `Decimal::inner().to_string()` for integer values | "Phase-22 Refactor Diff Sketch" | Very low — both call into the `Display` impl on `Decimal`. Sentinel test `phase24_gto_ind_negative_pointer_stringifies_with_sign` covers it explicitly. |
| A4 | The 11-variant count (12 enum lines including `Op::StoArithInd`) is correct against the FN-IND-01 list | "Variant Set" | Very low — re-counted against the FN-IND-01 enumerated list (`STO`, `RCL`, `ISG`, `DSE`, `SF`, `CF`, `FS?`, `FC?`, `FS?C`, `FC?C`, `STO+/-/×/÷`, `ARCL`, `ASTO`, `VIEW`) and confirmed: 4 flag-test ops collapse via `FlagTestKind` reuse → 1 variant; 4 sto-arith ops collapse via `StoArithKind` reuse → 1 variant. |

**All other claims in this research are verified against the local codebase via `Read` / `Grep` and tagged `[VERIFIED: <file:line>]` in the body. No assumed knowledge from training data was used for codebase-specific facts.**

## Open Questions

1. **Should `Op::IsgInd` / `Op::DseInd` be re-exported from `ops/indirect.rs::pub fn op_isg_ind` for direct external test access, or stay `pub(crate)` and be exercised only through `dispatch` / `run_program`?**
   - What we know: Phase 22 keeps `op_isg` as `pub fn` (program.rs:106) — the symmetric IND form would be `pub fn op_isg_ind`. The other shims could be `pub(crate)` because they're trivial delegations.
   - What's unclear: Whether the Phase 27 proptest sweep will want direct `op_*_ind` access for property-based fuzzing.
   - Recommendation: Keep all shims `pub(crate)` for now. Tests reach them through `dispatch` / `run_program` / `execute_op`. If Phase 27 needs direct access, change visibility then. Visibility relaxation is a backwards-compatible change.

2. **Should the planner add a `Op::*Ind` exhaustiveness sentinel test like `test_op_variant_count_matches_phase_target`?**
   - What we know: No such test exists today — the 4-place landing rule is enforced by exhaustive `match` failures at compile time.
   - What's unclear: Whether a "count enum variants and assert ≥ N" test adds value as an ergonomic safety net, or just adds friction.
   - Recommendation: Skip. The compile-time exhaustive-match failure is a more reliable safety net than a runtime count assertion. Existing convention (no such test in any prior phase) wins.

3. **Should the prgm_display arms use `format!("{mnemonic} IND {n:02}")` (space-separated) or `format!("{mnemonic}IND{n:02}")` (concatenated, like the existing `STO+ IND 12`)?**
   - What we know: The existing Phase-22 GtoInd/XeqInd uses the space-separated form: `format!("GTO IND {r:02}")` (cli/prgm_display.rs:180). The existing `Op::StoArith { reg, kind }` uses concatenated `STO+`/`STO-`/etc. (no IND). Format precedent for the IND family is set by Phase 22.
   - What's unclear: Nothing — Phase 22 set the precedent. Phase 24 follows it.
   - Recommendation: Use space-separated `format!("{mnemonic} IND {n:02}")` for all 12 new variants (matches Phase-22 `Op::GtoInd` precedent). For `Op::StoArithInd`, use `format!("STO{op_sym} IND {reg:02}")` — concatenated arithmetic symbol (matches existing `Op::StoArith` precedent), then space + IND.

## Risk Register

The planner and implementer should be aware of these specific concerns:

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| R1 | Refactor introduces silent regression in Phase-22 GTO/XEQ-IND behavior | Low | High (breaks shipped Phase 22 functionality) | (a) Existing `phase22_program_control.rs` 5 tests must continue passing. (b) New sentinel tests `phase24_xeq_ind_call_depth_guard_runs_before_pointer_read` and `phase24_gto_ind_negative_pointer_stringifies_with_sign` lock down the two specific concerns (atomicity + sign preservation). (c) Run full `just test` between 24-01 implementation and 24-02 implementation — wave-sequential makes this natural. |
| R2 | Forgetting `run_loop` arm for `Op::IsgInd` / `Op::DseInd` / `Op::FlagTestInd` (Pitfall 1) | Medium | Medium (silent skip-semantic loss) | (a) Test these specifically inside `run_program`, not just interactive dispatch. (b) Pitfall 1 documented above. (c) Add to the `|`-pattern catch-all in `execute_op` simultaneously (Pitfall 6). |
| R3 | `Op::StoArithInd` arity choice creates churn if changed mid-implementation | Low | Low | Decide single-variant before starting plan 24-02; document in plan task #1. Switching from single → four-flat after the fact requires touching dispatch/execute_op/cli-prgm/gui-prgm in atomic commit. |
| R4 | New `*_Ind` variants accidentally land in wrong serde-tag namespace | Very low | High (save-file corruption) | serde derive uses variant name as tag by default — no overrides anywhere on `Op`. As long as variant names are unique (and they are, by `<Name>Ind` convention), no collision possible. |
| R5 | `to_i64` overflow branch never tested → coverage slip | Medium | Low | Explicitly listed in test matrix (`resolve_indirect_pointer_exceeds_i64_range_rejects`). Coverage gate will catch if missed. |
| R6 | `Op::FlagTestInd` prgm_display formatting subtly wrong (Pitfall 4) | Low | Low (display bug only — listing shows wrong mnemonic) | Mirror the existing `Op::FlagTest` mnemonic mapping verbatim. Add explicit test for all 4 sub-kinds in `test_display_phase24_ind_op_labels` (cli + gui copies). |
| R7 | Plan 24-02 attempts to land 12 variants in one commit → too-large diff | Low | Low | Recommended task structure: split 24-02 into ≥3 task commits along category lines (regs IND / flag IND / alpha+display IND), each committing 3–4 variants with their tests. The wave-sequential constraint applies to plans, not tasks within a plan. |
| R8 | Wave 0 (test scaffold) merged into Wave 1 (implementation) prematurely | Low | Low | The test infrastructure already exists — Wave 0 is essentially empty (per "Wave 0 Gaps" above). The scaffold "tasks" are the new test-file creations, which naturally happen alongside the helper implementation in 24-01 / variant implementations in 24-02. No separate Wave-0 task gate is needed. |
| R9 | `Op::ViewInd` confused with "show pointer value" instead of "show resolved register value" (CONTEXT.md `<specifics>` warning) | Low | Medium (wrong UX behavior) | Pattern 2 above shows `op_view_ind` delegates to `op_view(state, addr)` — `op_view` formats `state.regs[addr]`, which is the RESOLVED register, not the pointer. The display value is the value of regs[12] (when R05=12, calling VIEW IND 05). Test `view_ind_happy` asserts this explicitly. |
| R10 | `pub use` of `resolve_indirect` from `lib.rs` forgotten — Phase 27 proptest breaks | Low | Low | Phase 27 is out of Phase 24 scope. If/when Phase 27 needs `resolve_indirect` from `hp41_core::ops::indirect::resolve_indirect`, it will be reachable via the `pub mod indirect;` declaration. No explicit `pub use` in lib.rs needed (callers can use the full path). |

## Sources

### Primary (HIGH confidence — verified against local codebase)
- `hp41-core/src/ops/program.rs:466-517` — current Phase-22 inline indirect-resolution pattern (exact source for the refactor diff sketch)
- `hp41-core/src/ops/mod.rs:55-71` — `FlagTestKind` enum (4 sub-cases, mirrors what `Op::FlagTestInd` reuses)
- `hp41-core/src/ops/mod.rs:319-329` — `Op::SfFlag(u8)`, `Op::CfFlag(u8)`, `Op::FlagTest { kind, flag }` — direct counterparts
- `hp41-core/src/ops/mod.rs:567-844` — full `dispatch()` table; verified Phase 24 will add 12 arms here
- `hp41-core/src/ops/program.rs:439-602` — `run_loop` body; verified ISG/DSE/FlagTest arm structure for IND mirrors
- `hp41-core/src/ops/program.rs:610-841` — full `execute_op()` body; verified the `|`-pattern catch-all at lines 825-840 needs IsgInd/DseInd/FlagTestInd added
- `hp41-core/src/ops/registers.rs:16-76` — `op_sto`, `op_rcl`, `op_sto_arith` (delegation targets)
- `hp41-core/src/ops/flags.rs:39-56` — `op_sf`, `op_cf` (delegation targets); flag-bounds-checks `< 56` confirmed
- `hp41-core/src/ops/alpha.rs:79-137` — `op_arcl`, `op_asto` (delegation targets); D-23.4 sidecar pattern observed
- `hp41-core/src/ops/display_ops.rs:17-26` — `op_view` (delegation target)
- `hp41-core/src/ops/program.rs:106-128` — `op_isg`, `op_dse` (delegation targets, return `Result<bool, _>`)
- `hp41-core/src/num.rs:217-226` — `HpNum::inner` and `HpNum::trunc_int` (resolver primitives)
- `hp41-core/src/num.rs:3` — `use rust_decimal::prelude::ToPrimitive;` confirms `Decimal::to_i64` is reachable
- `hp41-core/tests/phase22_program_control.rs:222-350` — 5 existing GTO/XEQ-IND tests that must continue passing
- `hp41-cli/src/prgm_display.rs:27-209` — exhaustive `op_display_name` (cli copy) — verified pattern for new IND arms
- `hp41-gui/src-tauri/src/prgm_display.rs:47-230` — exhaustive `op_display_name` (gui mirror) — same pattern
- `.planning/phases/24-indirect-addressing/24-CONTEXT.md` — D-24.1 through D-24.9 (locked decisions)
- `.planning/REQUIREMENTS.md:56-57` — FN-IND-01 / FN-IND-02 verbatim text
- `.planning/ROADMAP.md:122-138` — Phase 24 entry, success criteria, cross-cutting constraints
- `.planning/STATE.md` — current coverage baseline (92.5%), Phase 24 readiness
- `CLAUDE.md` — zero-panic invariant, 4-place Op landing, `just`-only task runner, MSRV 1.88, English-only commits

### Secondary (MEDIUM confidence — referenced from CONTEXT/discussion but not directly verified)
- `.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md` D-22.15 (XEQ IND atomicity), D-22.11.1 (regs bounds), D-22.21 (4-place landing) — referenced via CONTEXT.md cross-refs; concrete enforcement points verified in source above
- `.planning/phases/23-alpha-operations/23-CONTEXT.md` D-23.4 (sidecar-clearing), D-23.12 (4-place landing), D-23.14 (zero-panic) — referenced via CONTEXT.md cross-refs; sidecar code verified in `registers.rs:25, 71` and `alpha.rs:127-134`

### Tertiary (LOW confidence)
- None — every claim in this research is verified against the local codebase or sourced from CONTEXT.md/REQUIREMENTS.md/ROADMAP.md.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies needed; existing `rust_decimal` + `serde` already provide every primitive
- Architecture: HIGH — every decision is locked in CONTEXT.md; research validates implementation surface
- Variant set: HIGH — re-counted against FN-IND-01 list and existing `Op` enum; result is exactly 12 (matches D-24.7 estimate)
- Pitfalls: HIGH — all 6 derived from inspecting the existing dispatch/execute_op/run_loop structure for analogous bugs
- Coverage estimate: MEDIUM — projection based on line counts and test density; actual delta requires `just coverage` post-implementation
- `Op::StoArithInd` arity recommendation: MEDIUM — purely stylistic; either choice (single vs four-flat) is implementable

**Research date:** 2026-05-14
**Valid until:** 2026-06-14 (30 days — codebase is stable; research only invalidates if a new phase between 24 and execution lands new addressable ops, which is not on the v2.2 roadmap)

## RESEARCH COMPLETE
