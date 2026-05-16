# Phase 24: Indirect Addressing (Cross-Cutting) - Pattern Map

**Mapped:** 2026-05-14
**Files analyzed:** 7 (3 new, 4 modified)
**Analogs found:** 7 / 7 (every file has an exact-match analog already shipped)

## File Classification

| New / Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------------|------|-----------|----------------|---------------|
| `hp41-core/src/ops/indirect.rs` (NEW) | helper module (sibling to `flags.rs`) — two-tier resolver + ~11 delegation shims + `#[cfg(test)] mod tests` | request-response (`&CalcState` -> `Result<u8, HpError>`) | `hp41-core/src/ops/flags.rs` | exact (sibling helper module with inline unit tests, all-Neutral lift, `#[cfg(test)]` boundary block, free-helper + op-fn split) |
| `hp41-core/tests/phase24_resolve_indirect.rs` (NEW) | integration test — helper unit coverage + GTO/XEQ-IND regression sentinels | request-response | `hp41-core/tests/phase22_program_control.rs` (FN-PROG-06/07 GTO/XEQ-IND blocks) | exact (same dispatch + `run_program` style, same `state.regs[n] = HpNum::from(..)` setup, same `Decimal::from_str + HpNum::rounded` for non-integer cases) |
| `hp41-core/tests/phase24_ind_variants.rs` (NEW) | integration test — happy / non-integer / out-of-bounds for each new `Op::*Ind` variant (~33-39 tests) | request-response | `hp41-core/tests/phase23_arcl_asto.rs` | exact (Op-variant integration tests with `regs[i] = HpNum::from(...)` setup pattern, `dispatch(&mut state, Op::*)`, sidecar/lift inheritance assertions) |
| `hp41-core/src/ops/mod.rs` (MOD) | dispatcher hub — add ~11 new `Op::*Ind` enum variants + dispatch arms + `pub mod indirect;` declaration | request-response (delegate to indirect module) | existing `Op::StoArith { reg, kind }` enum entry + `Op::StoReg(r) => op_sto(state, r)` dispatch arm + Phase-22 `Op::GtoInd(_) \| Op::XeqInd(_) => Err(InvalidOp)` catch-all entry | exact (mirror naming, mirror payload shape, mirror `Result<bool, _>` -> `.map(\|_\| ())` pattern for IsgInd/DseInd, struct variant for FlagTestInd) |
| `hp41-core/src/ops/program.rs` (MOD) | execute_op + run_loop — refactor inline GtoInd/XeqInd onto inner helper, add execute_op arms for new IND variants, add run_loop arms for IsgInd/DseInd/FlagTestInd, extend programming-ops catch-all | request-response (run_loop -> resolve -> delegate) | existing `Op::GtoInd(reg) => { state.regs.get(...) ... pointer.trunc_int() ... find_in_program ... }` block (lines 474-487) + `Op::Isg(reg) => { if op_isg(state, reg)? { state.pc += 1; } }` (lines 549-553) + `Op::FlagTest { kind, flag } => { ... }` (lines 563-582) + catch-all `\|`-pattern at lines 825-840 | exact (same structural shape, helper consolidation collapses 9 lines into 2 per arm) |
| `hp41-cli/src/prgm_display.rs` (MOD) | display formatter — add ~12 match arms for new variants | one-pass transform (`&Op` -> `String`) | existing `Op::GtoInd(r) => format!("GTO IND {r:02}")` (line 180) + `Op::FlagTest { kind, flag } => { let mnemonic = match kind { ... }; format!("{mnemonic} {flag:02}") }` (lines 158-166) + `Op::StoArith { reg, kind }` mnemonic-table block (lines 82-90) | exact (identical "<MNEMONIC> IND <reg:02>" pattern, identical kind-table pattern for FlagTestInd / StoArithInd) |
| `hp41-gui/src-tauri/src/prgm_display.rs` (MOD) | display formatter (mirror of CLI copy — SC-4 invariant) | one-pass transform | `hp41-cli/src/prgm_display.rs` (the GUI file is a byte-for-byte mirror of the same function body — Phase 22/23 precedent) | exact (each new arm added in CLI must be added verbatim here) |

## Pattern Assignments

### `hp41-core/src/ops/indirect.rs` (NEW; helper module + delegation shims)

**Analog:** `hp41-core/src/ops/flags.rs`

**Why this analog:** Same role (sibling helper module under `ops/`), same data flow (pure functions over `&mut CalcState`), same shape (free-helper section + per-op functions + `#[cfg(test)] mod tests` block at the bottom). All Neutral lift policy on most ops matches.

**Module-header + imports pattern** (flags.rs lines 1-7):

```rust
//! Phase 21 flag operations: SF (set), CF (clear), and the bit-twiddling free helpers.
//!
//! Both ops have LiftEffect::Neutral. Flag indices are 0..=55; indices > 55 return InvalidOp.

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
```

**Free-helper pattern** (flags.rs lines 9-17 — defensive bounds + early-return):

```rust
/// Read flag `n` (0..=55) from the packed `flags` word.
/// Out-of-range indices (n > 55) return false defensively.
#[inline]
pub fn flag_get(flags: u64, n: u8) -> bool {
    if n > 55 {
        return false;
    }
    (flags & (1u64 << n)) != 0
}
```

**Op function pattern with `Result<(), HpError>` and explicit `InvalidOp` guard** (flags.rs lines 37-46):

```rust
/// SF n - set flag n (0..=55). LiftEffect: Neutral.
/// Returns `HpError::InvalidOp` for n > 55 without mutating state (range guard runs first).
pub fn op_sf(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n > 55 {
        return Err(HpError::InvalidOp);
    }
    state.flags = flag_set(state.flags, n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Inline `#[cfg(test)] mod tests` pattern** (flags.rs lines 59-92):

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::identity_op)] // n=0 is a valid LSB boundary case in this loop
    fn flag_set_get_boundaries() {
        for n in [0u8, 1, 31, 55] {
            let f = flag_set(0, n);
            assert_eq!(f, 1u64 << n, "flag_set({n}) must return 1<<{n}");
            assert!(flag_get(f, n), "flag_get must observe the set bit at {n}");
        }
    }
    // ...
}
```

**Bonus analog — direct sidecar+atomicity pattern (D-22.11.1 + D-23.4) that the IND shims inherit** (registers.rs lines 14-29):

```rust
/// STO n: copy X register into storage register n. Stack unchanged.
/// LiftEffect: Neutral. LASTX: not saved (STO is not an arithmetic operation).
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    // Phase 22 D-22.11.1: honor current SIZE (was hardcoded 100)
    let idx = reg as usize;
    if idx >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    // Phase 23 D-23.4: every numeric write to regs[reg] MUST clear the
    // packed-text shadow so ARCL never reads a stale string after a
    // numeric STO. Wave-0 sidecar-clearing audit.
    state.text_regs.remove(&reg);
    state.regs[idx] = state.stack.x.clone(); // safe — bounds-checked above
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**What to mirror:**

- File header `//! Phase 24 indirect-addressing helpers and op shims.` followed by `use crate::error::HpError; use crate::state::CalcState;` plus the additional `use rust_decimal::{Decimal, prelude::ToPrimitive}` (the only delta from `flags.rs`'s import block).
- Two-tier helpers `pub(crate) fn resolve_indirect_decimal(...)` and `pub fn resolve_indirect(...)` placed at the top of the file in the same section position as `flag_get` / `flag_set` / `flag_clear`.
- Per-op shim functions (`op_sto_ind`, `op_rcl_ind`, `op_sto_arith_ind`, `op_isg_ind`, `op_dse_ind`, `op_sf_flag_ind`, `op_cf_flag_ind`, `op_arcl_ind`, `op_asto_ind`, `op_view_ind`) following the `op_sf` shape: `pub(crate) fn op_<name>_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> { let addr = resolve_indirect(state, reg)?; op_<direct>(state, addr) }`. Inherits sidecar/atomicity/lift through the delegated direct op (see registers.rs:14-29) — do NOT replicate.
- Inline `#[cfg(test)] #[allow(clippy::unwrap_used)] mod tests { use super::*; ... }` at the bottom of the file (mirrors flags.rs:59-92).

---

### `hp41-core/tests/phase24_resolve_indirect.rs` (NEW; helper unit + GTO/XEQ-IND regression)

**Analog:** `hp41-core/tests/phase22_program_control.rs` (the FN-PROG-06 / FN-PROG-07 sections at lines 220-350)

**Why this analog:** Phase 22's GTO IND / XEQ IND tests are the *exact* code paths Phase 24 must prove unchanged after the refactor — same `state.regs[n] = HpNum::from(...)` setup, same `run_program` / `resume_program` driver choice, same `matches!(result, Err(HpError::InvalidOp))` rejection assertion shape, same `Decimal::from_str("12.345")` / `HpNum::rounded` for the non-integer case.

**Test-file header + imports pattern** (phase22_program_control.rs lines 1-17):

```rust
//! Integration tests for Phase 22 Plan 01 (program control:
//! STOP / PSE / GTO IND / XEQ IND + resume_program).
//!
//! Covers FN-PROG-01 / FN-PROG-02 / FN-PROG-06 / FN-PROG-07 plus the
//! three RESEARCH.md §2 pitfall sentinels:
//! ...

#![allow(clippy::unwrap_used)]

use hp41_core::ops::program::{resume_program, run_program};
use hp41_core::ops::{dispatch, Op};
use hp41_core::{format_hpnum, CalcState, DisplayMode, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;
```

**GTO IND happy-path pattern** (phase22_program_control.rs lines 222-242):

```rust
#[test]
fn test_gto_ind_happy() {
    // R05 = 42 (integer pointer); program has LBL "42" target.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(42i32);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::PushNum(HpNum::from(111i32)), // would-be unreachable after GTO
        Op::Lbl("42".to_string()),
        Op::PushNum(HpNum::from(7i32)),
    ];

    run_program(&mut state, "A").unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(7i32),
        "GTO IND R05 (= 42) must branch to LBL 42; got X = {:?}",
        state.stack.x
    );
}
```

**Non-integer-pointer rejection pattern** (phase22_program_control.rs lines 244-259):

```rust
#[test]
fn test_gto_ind_non_integer_rejects() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.345").unwrap());
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::Lbl("12".to_string()), // would-be target if truncation were allowed
    ];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "GTO IND with non-integer pointer must return InvalidOp (FN-IND-02); got {result:?}"
    );
}
```

**`resume_program`-driven 4-deep call-stack atomicity pattern** (phase22_program_control.rs lines 302-325 — KEY for the `phase24_xeq_ind_call_depth_guard_runs_before_pointer_read` sentinel):

```rust
#[test]
fn test_xeq_ind_4_deep_call_stack_rejects() {
    // Drive via resume_program (NOT run_program) so the pre-set call_stack
    // is NOT cleared at entry — run_program would wipe it (line 162).
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(10i32);
    state.program = vec![Op::XeqInd(3), Op::Lbl("10".to_string())];
    state.pc = 0;
    state.call_stack = vec![999usize; 4]; // pre-fill to 4 frames

    let result = resume_program(&mut state);

    assert!(
        matches!(result, Err(HpError::CallDepth)),
        "XEQ IND at 4-deep call_stack must return CallDepth; got {result:?}"
    );
    // Pre-mutation atomicity: the push did NOT happen — still exactly 4.
    assert_eq!(state.call_stack.len(), 4, "...");
}
```

**What to mirror:**

- Same `#![allow(clippy::unwrap_used)]` crate-level attribute, same `use hp41_core::ops::program::{resume_program, run_program}; use hp41_core::ops::{dispatch, Op}; use hp41_core::{CalcState, HpError, HpNum}; use rust_decimal::Decimal; use std::str::FromStr;` import block, same `// ── Phase 24 D-24.5 sentinel ─────` section dividers as the Phase-22 file.
- For the 4 sentinel tests (`phase24_gto_ind_uses_shared_helper`, `phase24_xeq_ind_uses_shared_helper`, `phase24_xeq_ind_call_depth_guard_runs_before_pointer_read`, `phase24_gto_ind_negative_pointer_stringifies_with_sign`): copy the existing `test_gto_ind_happy` / `test_xeq_ind_happy` / `test_xeq_ind_4_deep_call_stack_rejects` skeletons verbatim and only adjust the assertion comments to call out the shared-helper invariant being verified.
- For the call-depth-runs-FIRST sentinel: drive via `resume_program` (NOT `run_program`) so the pre-set `state.call_stack` survives entry — see the embedded comment at phase22_program_control.rs:304-305.
- For the helper unit tests (the 7 cases in §"Code Examples / Inner helper" of RESEARCH.md): use the inline `#[cfg(test)] mod tests` block in `ops/indirect.rs`, NOT this integration file. This file is reserved for cross-cutting GTO/XEQ-IND regression.

---

### `hp41-core/tests/phase24_ind_variants.rs` (NEW; ~33-39 tests for new IND variants)

**Analog:** `hp41-core/tests/phase23_arcl_asto.rs`

**Why this analog:** Same role (per-Op-variant integration tests for a sidecar-aware register-addressed family), same setup pattern (`state.regs[n] = HpNum::from(...)` then `dispatch(&mut state, Op::*)`), same sidecar inheritance assertions (`state.text_regs.get(&n)`), same per-test docstring style ("Test #N — ...").

**File header + imports pattern** (phase23_arcl_asto.rs lines 1-25):

```rust
//! Phase 23 (FN-ALPHA-01 + FN-ALPHA-02) integration tests for ARCL / ASTO.
//!
//! Covers the plan's documented success criteria and invariants:
//!   #1  ARCL respects current display mode (SC#1 — FIX/SCI/ENG)
//!   #2  ASTO + ARCL round-trip via the text_regs sidecar (SC#2)
//!   ...
//!
//! Test modules allow unwrap (CLAUDE.md "Zero panics" applies to production
//! code only; tests carry the precedent #[allow(clippy::unwrap_used)]).

#![allow(clippy::unwrap_used)]

use hp41_core::format::format_hpnum;
use hp41_core::num::HpNum;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::{CalcState, DisplayMode};
use hp41_core::HpError;
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Happy-path setup-then-dispatch-then-assert pattern** (phase23_arcl_asto.rs lines 73-91):

```rust
#[test]
fn asto_arcl_round_trip_reproduces_first_6_chars() {
    let mut state = CalcState::new();
    state.alpha_reg = "GOODBYE".to_string();
    dispatch(&mut state, Op::Asto(12)).unwrap();
    assert_eq!(state.text_regs.get(&12), Some(&"GOODBY".to_string()));
    assert_eq!(
        state.regs[12],
        HpNum::zero(),
        "no-drift invariant: ASTO zeroes the numeric slot"
    );

    // Clear ALPHA via Op::Cla. ...
    dispatch(&mut state, Op::Cla).unwrap();
    assert!(state.alpha_reg.is_empty());

    dispatch(&mut state, Op::Arcl(12)).unwrap();
    assert_eq!(state.alpha_reg, "GOODBY");
}
```

**Sidecar-inheritance assertion pattern** (phase23_arcl_asto.rs lines 96-119):

```rust
#[test]
fn numeric_sto_clears_text_regs_sidecar_no_drift() {
    let mut state = CalcState::new();

    // 1) ASTO 7 with ALPHA="HELLO" → text_regs[7]="HELLO", regs[7]=0.
    state.alpha_reg = "HELLO".to_string();
    dispatch(&mut state, Op::Asto(7)).unwrap();
    assert_eq!(state.text_regs.get(&7), Some(&"HELLO".to_string()));
    assert_eq!(state.regs[7], HpNum::zero());

    // 2) Put 3.14 in X and STO 7 → text_regs[7] must be CLEARED (D-23.4)
    //    AND regs[7] must hold the new numeric value.
    state.stack.x = HpNum::from(Decimal::from_str("3.14").unwrap());
    dispatch(&mut state, Op::StoReg(7)).unwrap();
    assert_eq!(
        state.text_regs.get(&7),
        None,
        "D-23.4: numeric STO must clear the text_regs sidecar"
    );
    // ...
}
```

**What to mirror:**

- Same `#![allow(clippy::unwrap_used)]` + same import block (add `use hp41_core::ops::program::run_program;` for the `Op::IsgInd` / `Op::DseInd` / `Op::FlagTestInd` run_loop tests — see Pitfall 1).
- Three test classes per variant (happy / non-integer / out-of-bounds), naming convention `<op_name>_ind_happy`, `<op_name>_ind_non_integer`, `<op_name>_ind_out_of_regs_len` (or `_out_of_flag_range` for SF/CF).
- Non-integer rejection setup: `state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap()); ... assert!(matches!(result, Err(HpError::InvalidOp)));` — verbatim from phase22_program_control.rs:247.
- Out-of-bounds setup: `state.regs[5] = HpNum::from(200i32);` (regs ops) or `HpNum::from(60i32)` (flag ops) — direct ops will return `InvalidOp` because `200 >= state.regs.len()` (default 100) or `60 > 55`. NO need to set `state.regs.len()` to anything special; default `CalcState::new()` ships 100 regs (verified at phase22_program_control.rs:264).
- For `Op::IsgInd` / `Op::DseInd` / `Op::FlagTestInd`: drive the happy-path test through `run_program`, NOT `dispatch` (Pitfall 1 — skip-next-step semantic only fires inside `run_loop`). Add a `Op::Lbl("A".to_string())` entry-point label and run with `run_program(&mut state, "A").unwrap()`.
- Optional bonus test per variant: sidecar/lift inheritance — for `Op::StoInd`, set `state.text_regs.insert(12, "ABC".to_string())` BEFORE the dispatch and assert it is cleared after; for `Op::RclInd`, assert `state.stack.lift_enabled == true` after.

---

### `hp41-core/src/ops/mod.rs` (MODIFIED; +11 enum variants + 11 dispatch arms + `pub mod indirect;`)

**Analog 1 (enum-variant shape):** existing `Op::StoArith { reg: u8, kind: StoArithKind }` and `Op::FlagTest { kind: FlagTestKind, flag: u8 }` (mod.rs lines 208-211 and 326-329).

**Analog 2 (dispatch arm):** existing `Op::StoReg(r) => op_sto(state, r)` block at mod.rs lines 664-667 plus `Op::Isg(reg) => program::op_isg(state, reg).map(|_| ())` at lines 687-691 plus `Op::FlagTest { .. } => { apply_lift_effect(state, LiftEffect::Neutral); Ok(()) }` at lines 745-748 plus `Op::GtoInd(_) | Op::XeqInd(_) => Err(HpError::InvalidOp)` at line 780.

**Why this analog:** Every new IND variant has an exact analog of the same structural shape already in this file — payload (`(u8)` or `(u8, kind)` or struct `{ kind, ind_reg }`), dispatch (delegate to op fn or `.map(|_| ())` for ISG/DSE), interactive-no-op (FlagTest), and run_loop-only (`Op::GtoInd(_) | Op::XeqInd(_) => Err(...)` catch-all).

**Module-declaration pattern** (mod.rs lines 9-21 — add `pub mod indirect;` to this list):

```rust
pub mod alpha;
pub mod arithmetic;
pub mod cardreader_ops;
pub mod display_ops;
pub mod flags;
pub mod hms;
pub mod math;
pub mod print;
pub mod program;
pub mod registers;
pub mod sound;
pub mod stack_ops;
pub mod stats;
```

**Enum-variant tuple shape** (mod.rs lines 202-216):

```rust
// ── Storage registers (Phase 2) ──────────────────────────────────
/// STO n — store X into register n (0–99). LiftEffect: Neutral.
StoReg(u8),
/// RCL n — recall register n into X (0–99). LiftEffect: Enable.
RclReg(u8),
/// STO+/−/×/÷ n — arithmetic on register n using X. LiftEffect: Neutral.
StoArith {
    reg: u8,
    kind: StoArithKind,
},
```

**Enum-variant struct shape (FlagTest)** (mod.rs lines 326-329):

```rust
FlagTest {
    kind: FlagTestKind,
    flag: u8,
},
```

**Programming-only IND tuple shape (precedent for every new `*Ind(u8)`)** (mod.rs lines 372-379):

```rust
GtoInd(u8),
/// XEQ IND nn — indirect subroutine call through register nn. ...
XeqInd(u8),
```

**Dispatch arms — direct delegation** (mod.rs lines 664-667):

```rust
Op::StoReg(r) => op_sto(state, r),
Op::RclReg(r) => op_rcl(state, r),
Op::StoArith { reg, kind } => op_sto_arith(state, reg, kind),
Op::StoArithStack { kind, stack_reg } => op_sto_arith_stack(state, stack_reg, kind),
```

**Dispatch arms — bool-discard for run_loop-only skip semantic** (mod.rs lines 687-695):

```rust
Op::Isg(reg) => {
    // op_isg returns Result<bool>; dispatch() returns Result<()>.
    // Discard the bool skip signal — skip semantics only apply inside run_loop.
    program::op_isg(state, reg).map(|_| ())
}
Op::Dse(reg) => {
    // Same as Isg: discard the bool skip signal — skip only applies inside run_loop.
    program::op_dse(state, reg).map(|_| ())
}
```

**Dispatch arm — interactive-no-op for FlagTest (PRECEDENT for FlagTestInd)** (mod.rs lines 740-748):

```rust
Op::SfFlag(n) => flags::op_sf(state, n),
Op::CfFlag(n) => flags::op_cf(state, n),
// Interactive FlagTest is a no-op (mirrors Op::Test). The skip-next-step
// and always-clear semantics live in run_loop. At the keyboard there is
// no "next program step" to skip; pc and flags are untouched.
Op::FlagTest { .. } => {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Run_loop-only catch-all for GTO IND / XEQ IND** (mod.rs line 780 — the new `Op::IsgInd(_) | Op::DseInd(_) | Op::FlagTestInd { .. }` go in their respective places NOT here, but interactive `Op::FlagTestInd { .. }` joins the same Neutral-no-op block above):

```rust
// GTO IND / XEQ IND require the run_loop state machine to manipulate
// pc and call_stack — interactive dispatch outside a running program
// is undefined. Return InvalidOp; run_loop handles them directly.
Op::GtoInd(_) | Op::XeqInd(_) => Err(HpError::InvalidOp),
```

**What to mirror:**

- Add `pub mod indirect;` to the module-declaration list at mod.rs:9-21 alongside `pub mod flags;` (alphabetical order suggests insertion between `hms` and `math`, or at end).
- For each of the 11 new variants, add the enum entry with a doc comment in the same format as `Op::GtoInd(u8)` (mod.rs:366-372): `/// <NAME> IND nn — ... LiftEffect: <X>. Phase 24 (FN-IND-01).`. Place all 11 in a contiguous `// ── Phase 24: Indirect Addressing ─────────` section near the end of the enum (after Phase 23's `Op::Posa`).
- For `Op::StoArithInd(u8, StoArithKind)`: tuple variant matching the `Op::Tone(u8)` style, NOT struct variant — D-24.7 says `<Name>Ind(u8)` is the family pattern. (Alternative struct variant `{ reg, kind }` is acceptable per RESEARCH.md "Claude's Discretion".)
- For `Op::FlagTestInd { kind: FlagTestKind, ind_reg: u8 }`: struct variant matching `Op::FlagTest { kind, flag }` (D-24.6) — this is the only struct-variant in the new family, intentional per CONTEXT D-24.6.
- Dispatch arms for every IND variant: 1-line delegation `Op::<Name>Ind(reg) => indirect::op_<name>_ind(state, reg)` (mirror lines 664-665). For `Op::IsgInd` / `Op::DseInd`: use `.map(|_| ())` to drop the bool (mirror lines 687-695). For `Op::FlagTestInd { .. }`: Neutral no-op (mirror lines 745-748). For `Op::StoArithInd(reg, kind)`: `indirect::op_sto_arith_ind(state, reg, kind)` (mirror line 666).
- Add an additional import line at mod.rs:23-35 importing the indirect module's shim functions, OR call them via the qualified path `indirect::op_sto_ind` to avoid touching the import block. Per RESEARCH §"Module wiring", the qualified-path approach is acceptable.

---

### `hp41-core/src/ops/program.rs` (MODIFIED; refactor GtoInd/XeqInd + add execute_op + run_loop arms + extend catch-all)

**Analog 1 (refactor target):** the existing inline `Op::GtoInd` block at program.rs:474-487 and `Op::XeqInd` block at lines 500-517. **These are exactly what is being refactored** — Phase 24 replaces the inline `state.regs.get(...).ok_or(InvalidOp)?.clone(); pointer.trunc_int(); if int_part != pointer { return Err(InvalidOp); }` chain with a single call to `crate::ops::indirect::resolve_indirect_decimal(state, *reg)?`.

**Analog 2 (run_loop skip-semantic arm for IsgInd/DseInd):** existing `Op::Isg(reg) => { if op_isg(state, reg)? { state.pc += 1; } }` block at program.rs:549-553.

**Analog 3 (run_loop FlagTest with always-clear semantics — for FlagTestInd):** `Op::FlagTest { kind, flag }` block at program.rs:563-582.

**Analog 4 (execute_op delegation arms):** `Op::StoReg(r) => op_sto(state, r)` and `Op::SfFlag(n) => super::flags::op_sf(state, n)` at lines 698-700 and 758-759.

**Analog 5 (programming-ops catch-all):** `|`-pattern at program.rs:825-840 — adds `Op::IsgInd(_) | Op::DseInd(_) | Op::FlagTestInd { .. }` to the existing list.

**GTO IND inline block to be refactored** (program.rs lines 467-487):

```rust
// ── Phase 22 (D-22.15, FN-PROG-06): GTO IND nn ────────────────
// Inline indirect resolver. Phase 24 will extract this into a
// shared `resolve_indirect()` helper for ~15 IND variants.
//
// 1. Read register (bounds-safe via .get() — D-22.23 zero-panic).
// 2. Truncate to integer; reject non-integer pointers (FN-IND-02).
// 3. Stringify the integer and reuse find_in_program (mirrors Op::Gto).
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

**XEQ IND inline block to be refactored — preserve `call_stack.len() >= 4` PRE-mutation guard** (program.rs lines 488-517):

```rust
// ── Phase 22 (D-22.15, FN-PROG-07): XEQ IND nn ────────────────
// Same inline indirect resolver as Op::GtoInd, but performs a
// subroutine call: push pc onto call_stack BEFORE redirecting.
//
// CRITICAL: the 4-deep call_stack guard is PRE-mutation (D-13 /
// D-14 precedent of Op::Xeq at line 206). The check fires BEFORE
// reading the register, so an over-deep call returns CallDepth
// without partially mutating any state.
//
// No builtin_card_op fallback — indirect labels are numeric
// strings only ...
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

**Run_loop ISG arm — analog for `Op::IsgInd` and `Op::DseInd`** (program.rs lines 549-558):

```rust
Op::Isg(reg) => {
    if op_isg(state, reg)? {
        state.pc += 1; // loop exit: skip next
    }
}
Op::Dse(reg) => {
    if op_dse(state, reg)? {
        state.pc += 1; // loop exit: skip next
    }
}
```

**Run_loop FlagTest arm — analog for `Op::FlagTestInd`** (program.rs lines 559-582):

```rust
// ── Phase 21: Flag tests (skip next step pattern, mirrors Op::Test) ──
// FS?/FC? skip the next step when the flag is in the "false" state.
// FS?C/FC?C ALWAYS clear the flag as a side effect (RESEARCH A4), THEN
// decide the skip based on the PRE-clear state.
Op::FlagTest { kind, flag } => {
    use crate::ops::flags::{flag_clear, flag_get};
    use crate::ops::FlagTestKind;
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

**Programming-ops catch-all to be extended** (program.rs lines 824-840):

```rust
// Programming ops handled by run_loop directly — must not reach here
Op::Lbl(_)
| Op::Gto(_)
| Op::Xeq(_)
| Op::Rtn
| Op::PrgmMode
| Op::Test(_)
| Op::Isg(_)
| Op::Dse(_)
| Op::FlagTest { .. }
| Op::Prompt
| Op::Stop                              // Phase 22: STOP handled by run_loop break
| Op::GtoInd(_)                         // Phase 22: GTO IND has run_loop arm
| Op::XeqInd(_)                         // Phase 22: XEQ IND has run_loop arm
| Op::Clp(_)                            // Phase 22: CLP is a PRGM-mode editing primitive
| Op::Del(_)                            // Phase 22: DEL is a PRGM-mode editing primitive
| Op::Ins => Err(HpError::InvalidOp),   // Phase 22: INS is a PRGM-mode editing primitive
```

**What to mirror:**

- Refactor `Op::GtoInd(reg)` arm: replace the 9-line inline block (lines 475-484) with `let i = crate::ops::indirect::resolve_indirect_decimal(state, *reg)?; let label_str = i.to_string();`. Keep `find_in_program` call and `state.pc = target + 1` advance UNCHANGED.
- Refactor `Op::XeqInd(reg)` arm: same Decimal-helper substitution. The `if state.call_stack.len() >= 4 { return Err(HpError::CallDepth); }` guard MUST stay as the FIRST line of the arm (D-22.15 / sentinel test #3 pre-mutation atomicity). Keep `state.call_stack.push(state.pc); state.pc = target + 1;` UNCHANGED.
- Add run_loop arms for `Op::IsgInd(reg)` and `Op::DseInd(reg)` next to lines 549-558, mirroring the existing arms exactly: `Op::IsgInd(reg) => { if crate::ops::indirect::op_isg_ind(state, reg)? { state.pc += 1; } }` (and same for DseInd). The `op_*_ind` helper returns `Result<bool, _>` — do NOT discard the bool here.
- Add run_loop arm for `Op::FlagTestInd { kind, ind_reg }` next to lines 563-582: resolve `ind_reg` via `let flag = crate::ops::indirect::resolve_indirect(state, ind_reg)?;`, then reuse the existing kind-match block verbatim with that resolved `flag`.
- Add execute_op arms for the non-skip-semantic IND variants (`Op::StoInd`, `Op::RclInd`, `Op::StoArithInd`, `Op::SfFlagInd`, `Op::CfFlagInd`, `Op::ArclInd`, `Op::AstoInd`, `Op::ViewInd`) next to lines 698-700: `Op::StoInd(reg) => crate::ops::indirect::op_sto_ind(state, reg)` etc. For `Op::IsgInd` / `Op::DseInd`: also add execute_op arm with `.map(|_| ())` (defense-in-depth — these primarily run via run_loop but must compile-pass execute_op exhaustiveness).
- Extend the programming-ops catch-all at lines 825-840 by inserting `| Op::IsgInd(_) | Op::DseInd(_) | Op::FlagTestInd { .. }` BEFORE the `=> Err(HpError::InvalidOp)` (Pitfall 6 — defense-in-depth).

---

### `hp41-cli/src/prgm_display.rs` (MODIFIED; +12 match arms)

**Analog 1 (`<NAME> IND <r:02>` literal pattern):** existing `Op::GtoInd(r) => format!("GTO IND {r:02}")` and `Op::XeqInd(r) => format!("XEQ IND {r:02}")` at lines 180-181.

**Analog 2 (FlagTest kind-table pattern — for `Op::FlagTestInd`):** existing `Op::FlagTest { kind, flag }` block at lines 158-166.

**Analog 3 (StoArith op-symbol-table pattern — for `Op::StoArithInd`):** existing `Op::StoArith { reg, kind }` block at lines 82-90.

**Why this analog:** Every new IND display arm is one of three shapes that already exist in this exact file: bare `format!("<MNEMONIC> IND {r:02}")` (mirrors GTO IND / XEQ IND), kind-table-then-format (mirrors FlagTest), or op-symbol-table-then-format (mirrors StoArith).

**Bare-mnemonic IND pattern** (cli/src/prgm_display.rs lines 178-181):

```rust
// Phase 22: Program control
Op::Stop => "STOP".to_string(),
Op::Pse => "PSE".to_string(),
Op::GtoInd(r) => format!("GTO IND {r:02}"),
Op::XeqInd(r) => format!("XEQ IND {r:02}"),
```

**Kind-table-then-format pattern (FlagTest)** (cli/src/prgm_display.rs lines 158-166):

```rust
Op::FlagTest { kind, flag } => {
    let mnemonic = match kind {
        FlagTestKind::IsSet => "FS?",
        FlagTestKind::IsClear => "FC?",
        FlagTestKind::IsSetThenClear => "FS?C",
        FlagTestKind::IsClearThenClear => "FC?C",
    };
    format!("{mnemonic} {flag:02}")
}
```

**Op-symbol-table-then-format pattern (StoArith)** (cli/src/prgm_display.rs lines 82-90):

```rust
Op::StoArith { reg, kind } => {
    let op_sym = match kind {
        StoArithKind::Add => "+",
        StoArithKind::Sub => "-",
        StoArithKind::Mul => "\u{00D7}",
        StoArithKind::Div => "\u{00F7}",
    };
    format!("STO{op_sym} {reg:02}")
}
```

**What to mirror:**

- For 8 simple variants (`StoInd`, `RclInd`, `IsgInd`, `DseInd`, `SfFlagInd`, `CfFlagInd`, `ArclInd`, `AstoInd`, `ViewInd`): one-line `Op::<Name>Ind(r) => format!("<MNEMONIC> IND {r:02}")` in a new `// Phase 24: Indirect Addressing` section near the end of the function (after Phase 23's `Op::Posa => "POSA".to_string(),`). Mnemonics are the existing direct-form mnemonics: `STO`, `RCL`, `ISG`, `DSE`, `SF`, `CF`, `ARCL`, `ASTO`, `VIEW`.
- For `Op::FlagTestInd { kind, ind_reg }`: copy the existing `Op::FlagTest { kind, flag }` block verbatim, change `flag` to `ind_reg`, change the final `format!("{mnemonic} {flag:02}")` to `format!("{mnemonic} IND {ind_reg:02}")`.
- For `Op::StoArithInd(reg, kind)`: copy the existing `Op::StoArith { reg, kind }` block verbatim, change `format!("STO{op_sym} {reg:02}")` to `format!("STO{op_sym} IND {reg:02}")`.
- Imports already cover `FlagTestKind` and `StoArithKind` (line 7: `use hp41_core::ops::{FlagTestKind, Op, StackReg, StoArithKind};`) — no import change needed.

---

### `hp41-gui/src-tauri/src/prgm_display.rs` (MODIFIED; mirror of CLI)

**Analog:** the CLI copy (`hp41-cli/src/prgm_display.rs`). The GUI file is a deliberate byte-for-byte mirror of the same `op_display_name` function — see the existing parallel lines for `Op::Arcl(reg) => format!("ARCL {reg:02}")` (cli line 201, gui line 221), `Op::FlagTest { kind, flag } => { ... }` (cli lines 158-166, gui lines 178-186), and `Op::GtoInd(r) => format!("GTO IND {r:02}")` (cli line 180, gui line 200).

**GUI mirror — verbatim FlagTest block** (gui/src-tauri/src/prgm_display.rs lines 178-186):

```rust
Op::FlagTest { kind, flag } => {
    let mnemonic = match kind {
        FlagTestKind::IsSet => "FS?",
        FlagTestKind::IsClear => "FC?",
        FlagTestKind::IsSetThenClear => "FS?C",
        FlagTestKind::IsClearThenClear => "FC?C",
    };
    format!("{mnemonic} {flag:02}")
}
```

**GUI mirror — Phase 22/23 IND + ARCL/ASTO arms** (gui/src-tauri/src/prgm_display.rs lines 200-222):

```rust
Op::GtoInd(r) => format!("GTO IND {r:02}"),
Op::XeqInd(r) => format!("XEQ IND {r:02}"),
// ...
Op::Arcl(reg) => format!("ARCL {reg:02}"),
Op::Asto(reg) => format!("ASTO {reg:02}"),
// Phase 23 plan 02 (FN-ALPHA-03..06): bare-string variants (mirror
// of the hp41-cli copy — SC-4 invariant requires identical display
// listing on both frontends).
Op::Atox => "ATOX".to_string(),
```

**What to mirror:**

- Add the SAME 12 match arms in the same order as the CLI copy (D-22.21 / D-23.12 4-place landing rule). The existing precedent — explicit comment block `// Phase 23 plan 02 ... mirror of the hp41-cli copy — SC-4 invariant requires identical display listing on both frontends.` (gui lines 223-226) — should be repeated with `// Phase 24: Indirect Addressing — mirror of the hp41-cli copy ...`.
- Verify `FlagTestKind` and `StoArithKind` are already imported in this file — they are (the existing `Op::FlagTest` and `Op::StoArith` arms compile, so the imports are present). No new imports needed.

---

## Shared Patterns

### Sidecar / Atomicity / Lift-effect Inheritance via Delegation (D-24.4)

**Source:** `hp41-core/src/ops/registers.rs` (op_sto + op_sto_arith — sidecar + atomicity already in place)

**Apply to:** ALL 11 new `op_*_ind` shim functions in `ops/indirect.rs`

**The shim is exactly 2 lines** — bounds, sidecar (D-23.4 `text_regs.remove(&reg)`), atomicity (compute-then-write), and `apply_lift_effect(state, LiftEffect::Neutral|Enable)` are inherited GRATIS through the delegated direct op:

```rust
pub(crate) fn op_sto_ind(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let addr = resolve_indirect(state, reg)?;
    op_sto(state, addr)  // <-- direct op carries D-22.11.1 bounds + D-23.4 sidecar + Neutral lift
}
```

**Anti-pattern (NEVER do this in any IND shim):** Re-doing the bounds check (`if addr as usize >= state.regs.len()`) or sidecar clear (`state.text_regs.remove(&addr)`) in the shim. That creates drift risk and violates D-24.3.

### Zero-Panic Resolver Path (D-23.14)

**Source:** existing zero-panic invariant `#![deny(clippy::unwrap_used)]` at `hp41-core/src/lib.rs`

**Apply to:** ALL Phase 24 production code (`ops/indirect.rs`, refactored `ops/program.rs` arms, dispatch arms in `ops/mod.rs`)

**Rules:**
- `state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?` — never `state.regs[reg as usize]` (no raw indexing on the IND path).
- `decimal.to_i64().ok_or(HpError::InvalidOp)?` — never `.unwrap()`.
- `u8::try_from(as_i64).map_err(|_| HpError::InvalidOp)` — never `as u8` (silent truncation hides the out-of-range bug).
- Test modules carry `#![allow(clippy::unwrap_used)]` (CLAUDE.md precedent — applied at file level for `tests/phase24_resolve_indirect.rs` and `tests/phase24_ind_variants.rs`, applied at `mod tests {}` level for the inline `ops/indirect.rs` tests).

### 4-Place Landing for Every New Op Variant (D-22.21 / D-23.12)

**Sources (all four MUST be updated for every new IND variant):**

1. `hp41-core/src/ops/mod.rs::dispatch()` — interactive arm (typically delegate; FlagTestInd is Neutral no-op; IsgInd/DseInd use `.map(|_| ())`)
2. `hp41-core/src/ops/program.rs::execute_op()` — programmatic arm (every IND variant; IsgInd/DseInd/FlagTestInd ALSO need `run_loop` arms above for skip semantics)
3. `hp41-cli/src/prgm_display.rs::op_display_name` — listing display
4. `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` — GUI mirror

**Compile-time enforcement:** Every match in the four files is exhaustive with no `_ =>` catch-all (except the deliberate `|`-pattern at program.rs:825-840). Forgetting any of the four sites will fail-compile.

**Apply to:** ALL 11 new IND variants — no exceptions.

### Stub-Error Pattern (NOT applicable to Phase 24)

Phase 24 ships fully-functional ops, NOT stubs — explicitly NOT the v2.1 stub-error pattern (`Err(GuiError { message: "planned for a future phase" })`). All new IND variants must work end-to-end.

## No Analog Found

None. Every file has at least one exact-match analog already in the codebase. The phase is a refactor + extension — every shape Phase 24 introduces was already established by Phase 21, 22, or 23.

## Metadata

**Analog search scope:**
- `hp41-core/src/ops/` (all `.rs` files for sibling-module precedent)
- `hp41-core/src/ops/mod.rs` lines 1-50 (module declarations), 200-450 (Op enum), 650-840 (dispatch)
- `hp41-core/src/ops/program.rs` lines 100-130 (op_isg/op_dse), 460-602 (run_loop), 605-841 (execute_op + catch-all)
- `hp41-core/src/ops/registers.rs` lines 1-100 (op_sto/op_rcl/op_sto_arith — direct-form delegation targets)
- `hp41-core/src/ops/flags.rs` lines 1-92 (sibling-module structural template + inline tests)
- `hp41-core/tests/phase22_program_control.rs` lines 1-350 (GTO/XEQ-IND test precedent)
- `hp41-core/tests/phase23_arcl_asto.rs` lines 1-120 (Op-variant integration test precedent)
- `hp41-cli/src/prgm_display.rs` lines 75-205 (display-formatter precedent)
- `hp41-gui/src-tauri/src/prgm_display.rs` lines 175-230 (GUI mirror precedent)

**Files scanned:** 9

**Pattern extraction date:** 2026-05-14
