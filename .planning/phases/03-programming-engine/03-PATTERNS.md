# Phase 3: Programming Engine - Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 6
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/state.rs` | model | CRUD | `hp41-core/src/state.rs` (self) | exact — extend existing struct |
| `hp41-core/src/error.rs` | model | — | `hp41-core/src/error.rs` (self) | exact — add variant to existing enum |
| `hp41-core/src/ops/mod.rs` | controller | request-response | `hp41-core/src/ops/mod.rs` (self) | exact — extend Op enum + dispatch() |
| `hp41-core/src/ops/program.rs` | service | event-driven | `hp41-core/src/ops/registers.rs` | role-match — same function signature pattern; `alpha.rs` for bool-toggle pattern |
| `hp41-core/src/lib.rs` | config | — | `hp41-core/src/lib.rs` (self) | exact — add one pub use line |
| `hp41-core/tests/program_tests.rs` | test | CRUD | `hp41-core/tests/register_tests.rs` | role-match — identical integration test conventions |

---

## Pattern Assignments

### `hp41-core/src/state.rs` (model — extend CalcState)

**Analog:** `hp41-core/src/state.rs` (self, lines 51–85)

**Existing struct pattern** (lines 51–79):
```rust
#[derive(Debug, Clone)]
pub struct CalcState {
    pub stack: Stack,
    pub regs: [HpNum; 100],
    pub alpha_reg: String,
    pub alpha_mode: bool,
    pub angle_mode: AngleMode,
    pub display_mode: DisplayMode,
    pub entry_buf: String,
}

impl CalcState {
    pub fn new() -> Self {
        CalcState {
            stack: Stack::new(),
            regs: std::array::from_fn(|_| HpNum::zero()),
            alpha_reg: String::new(),
            alpha_mode: false,
            angle_mode: AngleMode::Deg,
            display_mode: DisplayMode::Fix(4),
            entry_buf: String::new(),
        }
    }
}
```

**Phase 3 additions — copy this block after `entry_buf: String` in the struct, and after `entry_buf: String::new()` in `new()`:**
```rust
// ── Phase 3: Programming Engine ──────────────────────────────────────────
/// Keystroke program storage. Flat list — Op::Lbl markers delimit subroutines.
pub program: Vec<Op>,
/// PRGM mode: when true dispatch() records ops instead of executing.
pub prgm_mode: bool,
/// Program counter — index of the next op to execute in `program`.
pub pc: usize,
/// Subroutine return stack. Max 4 entries (HP-41 hardware limit).
pub call_stack: Vec<usize>,
/// True while run_program() is active; guards against re-entrancy.
pub is_running: bool,
```

**`new()` additions:**
```rust
program: Vec::new(),
prgm_mode: false,
pc: 0,
call_stack: Vec::new(),
is_running: false,
```

**Import note:** `state.rs` imports `crate::num::HpNum` (line 26). When `Op` is added to state.rs it will need `use crate::ops::Op;` — but this creates a circular dependency (ops/mod.rs imports state). Resolution: define the `Op` type in a separate `ops/op.rs` or in `state.rs`, or add `program: Vec<crate::ops::Op>` inline. Check whether ops/mod.rs already imports state before deciding — if so, move `Op` enum definition to its own module that neither state nor ops imports.

**Alternative (simpler):** Keep `program: Vec<crate::ops::Op>` written as the full path; Rust resolves it at compile time without a circular import because `ops` depends on `state`, not the other way around. The struct field type uses `crate::ops::Op` but `state.rs` does not need to `use` it — just spell the full path.

---

### `hp41-core/src/error.rs` (model — add HpError::CallDepth)

**Analog:** `hp41-core/src/error.rs` (self, lines 1–13)

**Full existing file** (lines 1–13):
```rust
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum HpError {
    #[error("overflow")]
    Overflow,
    #[error("divide by zero")]
    DivideByZero,
    #[error("invalid operation")]
    InvalidOp,
    #[error("domain error")]
    Domain,
}
```

**Phase 3 addition — append before the closing `}`:**
```rust
    // Phase 3 addition — HP-41 "try again" (5th subroutine level exceeded)
    #[error("try again")]
    CallDepth,
```

**Pattern note:** All variants use `#[error("...")]` string literals. No variant carries data. `PartialEq + Clone` derived — mandatory because tests use `assert_eq!(result, Err(HpError::CallDepth))`.

---

### `hp41-core/src/ops/mod.rs` (controller — extend Op enum + dispatch())

**Analog:** `hp41-core/src/ops/mod.rs` (self, lines 1–208)

**Import block** (lines 1–23) — new imports to add:
```rust
// Existing (do not change):
use crate::error::HpError;
use crate::num::HpNum;
use crate::state::{CalcState, DisplayMode};
use crate::stack::{apply_lift_effect, LiftEffect};
use rust_decimal::Decimal;
use std::str::FromStr;

// Phase 3 additions:
pub mod program;
use program::run_program;     // not re-exported here; lib.rs re-exports it
```

**Existing StoArithKind enum pattern** (lines 27–32) — copy for TestKind:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoArithKind {
    Add,
    Sub,
    Mul,
    Div,
}
```
Copy this pattern exactly for `TestKind`:
```rust
/// HP-41 conditional test kind — 12 total. Used in Op::Test(TestKind).
#[derive(Debug, Clone, PartialEq)]
pub enum TestKind {
    XEqZero, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero,
    XEqY,    XNeY,    XLtY,    XGtY,    XLeY,    XGeY,
}
```

**Op enum extension** — copy the doc-comment + variant pattern from lines 94–117:
```rust
// ── Programming (Phase 3) ────────────────────────────────────────────
/// LBL "name" — program label marker. No-op during execution. LiftEffect: Neutral.
Lbl(String),
/// GTO "name" — unconditional branch. LiftEffect: Neutral.
Gto(String),
/// XEQ "name" — subroutine call (max 4 deep). LiftEffect: Neutral.
Xeq(String),
/// RTN — return from subroutine; terminates run if call_stack is empty. LiftEffect: Neutral.
Rtn,
/// PRGM — toggle prgm_mode recording flag. LiftEffect: Neutral.
PrgmMode,
/// Conditional test (skip-next-if-false). LiftEffect: Neutral.
Test(TestKind),
/// ISG n — increment reg n, skip if new value > final. LiftEffect: Neutral.
Isg(u8),
/// DSE n — decrement reg n, skip if new value <= final. LiftEffect: Neutral.
Dse(u8),
```

**dispatch() prgm_mode gate** — insert at the top of `dispatch()`, after `flush_entry_buf(state)?;`, before the `match op {` (lines 146–148):
```rust
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    flush_entry_buf(state)?;

    // ── Phase 3: PRGM mode recording gate ─────────────────────────────────
    if state.prgm_mode {
        // PrgmMode op exits recording immediately (HP-41 toggle behavior).
        if matches!(op, Op::PrgmMode) {
            state.prgm_mode = false;
            apply_lift_effect(state, LiftEffect::Neutral);
            return Ok(());
        }
        // All other ops are recorded; stack is not modified.
        state.program.push(op);
        return Ok(());
    }

    match op {
        // ... existing arms unchanged ...
        // Phase 3 arms (added in Plan 03-06):
        Op::Lbl(_)     => program::op_lbl(state),
        Op::Gto(s)     => program::op_gto(state, &s),
        Op::Xeq(s)     => program::op_xeq(state, &s),
        Op::Rtn        => program::op_rtn(state),
        Op::PrgmMode   => program::op_prgm_mode(state),
        Op::Test(kind) => program::op_test(state, kind),
        Op::Isg(reg)   => program::op_isg(state, reg),
        Op::Dse(reg)   => program::op_dse(state, reg),
    }
}
```

**flush_entry_buf routing** (lines 119–140) — extend to check `prgm_mode`:
```rust
pub fn flush_entry_buf(state: &mut CalcState) -> Result<(), HpError> {
    if state.entry_buf.is_empty() {
        return Ok(());
    }
    let s = state.entry_buf.clone();
    state.entry_buf.clear();
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    let n = HpNum::rounded(d);

    if state.prgm_mode {
        // Recording mode: PushNum goes to program Vec, not stack
        state.program.push(Op::PushNum(n));
        // lift_enabled is NOT changed — recording does not affect execution state
    } else {
        // Execute mode: existing behavior unchanged
        crate::stack::enter_number(state, n);
        crate::stack::apply_lift_effect(state, LiftEffect::Enable);
    }
    Ok(())
}
```

---

### `hp41-core/src/ops/program.rs` (service — NEW FILE)

**Primary analog:** `hp41-core/src/ops/registers.rs` (lines 1–67) — same function signature, same error handling, same LiftEffect pattern.

**Secondary analog:** `hp41-core/src/ops/alpha.rs` (lines 1–38) — bool toggle pattern for `op_prgm_mode`.

**File header pattern** (copy from `registers.rs` lines 1–6, adapted):
```rust
//! Phase 3 programming engine: LBL, GTO, XEQ, RTN, PRGM, Test, ISG, DSE, run_program.
//!
//! All programming ops have LiftEffect: Neutral (they do not modify lift_enabled).
//! run_program() is the public interpreter entry point exported via lib.rs.
```

**Imports pattern** (from `registers.rs` lines 7–10, adapted):
```rust
use crate::error::HpError;
use crate::num::HpNum;
use crate::state::CalcState;
use crate::stack::apply_lift_effect;
use crate::stack::LiftEffect;
use crate::ops::{Op, TestKind};
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Simple Neutral op pattern** — copy from `alpha.rs` `op_alpha_toggle` (lines 13–17):
```rust
/// LBL: no-op during interactive execution; marker only when running a program.
/// LiftEffect: Neutral.
pub fn op_lbl(state: &mut CalcState) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// PrgmMode: toggle prgm_mode flag (enter recording).
/// Exit path is handled in dispatch() directly; this handles the enter path.
/// LiftEffect: Neutral.
pub fn op_prgm_mode(state: &mut CalcState) -> Result<(), HpError> {
    state.prgm_mode = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Validated index op pattern** — copy from `registers.rs` `op_sto` (lines 14–21):
```rust
/// GTO: branch to label. Only meaningful when is_running; returns InvalidOp otherwise.
/// LiftEffect: Neutral.
pub fn op_gto(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    // Interactive (non-running) GTO has no defined behavior for Phase 3.
    if !state.is_running {
        return Err(HpError::InvalidOp);
    }
    let target = find_label_in_state(state, label)?;
    state.pc = target + 1;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// XEQ: subroutine call. Enforces 4-level call stack limit.
/// LiftEffect: Neutral.
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if state.call_stack.len() >= 4 {
        return Err(HpError::CallDepth);
    }
    let target = find_label_in_state(state, label)?;
    state.call_stack.push(state.pc);
    state.pc = target + 1;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RTN: return from subroutine. Terminates run if call_stack is empty.
/// LiftEffect: Neutral.
pub fn op_rtn(state: &mut CalcState) -> Result<(), HpError> {
    if let Some(return_pc) = state.call_stack.pop() {
        state.pc = return_pc;
    }
    // Empty call_stack = top-level RTN — run_loop handles termination via this no-op
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Error-guarded compute-first pattern** — copy from `registers.rs` `op_sto_arith` (lines 43–58):
```rust
/// ISG n: increment register n, return true if loop should skip (current > final).
/// Uses string-split parsing per ADR-001 — never floor()/fmod().
/// LiftEffect: Neutral.
pub fn op_isg(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    if reg as usize >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let (current, final_val, step, frac_padded) = parse_counter(&state.regs[reg as usize])?;
    let new_current = current + step;
    state.regs[reg as usize] = build_counter(new_current, &frac_padded)?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(new_current > final_val)   // true = skip next (loop exits)
}

/// DSE n: decrement register n, return true if loop should skip (current <= final).
pub fn op_dse(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    if reg as usize >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let (current, final_val, step, frac_padded) = parse_counter(&state.regs[reg as usize])?;
    let new_current = current - step;
    state.regs[reg as usize] = build_counter(new_current, &frac_padded)?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(new_current <= final_val)  // true = skip next (loop exits)
}
```

**ISG/DSE helper pattern** — no analog in codebase; use RESEARCH.md algorithm:
```rust
/// Parse CCCCC.FFFDD counter format by string-splitting at '.'.
/// Returns (current, final, step, frac_padded_5_chars).
/// ADR-001: never use floor()/fmod() on f64.
fn parse_counter(n: &HpNum) -> Result<(i64, i64, i64, String), HpError> {
    let s = n.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let current: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // Pad RIGHT with zeros to exactly 5 chars (trailing-zero normalization fix)
    let frac_padded = format!("{:0<5}", frac_part); // left-align, pad right
    let final_val: i64 = frac_padded[..3].parse().map_err(|_| HpError::InvalidOp)?;
    let step_raw: i64 = frac_padded[3..5].parse().map_err(|_| HpError::InvalidOp)?;
    let step = if step_raw == 0 { 1 } else { step_raw };
    Ok((current, final_val, step, frac_padded))
}

fn build_counter(current: i64, frac_padded: &str) -> Result<HpNum, HpError> {
    let s = format!("{}.{}", current, frac_padded);
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::rounded(d))
}

fn find_label_in_state(state: &CalcState, label: &str) -> Result<usize, HpError> {
    state.program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == label))
        .ok_or(HpError::InvalidOp)
}
```

**op_test pattern** — no analog; uses TestKind enum:
```rust
/// Test: evaluate conditional; caller (run_loop) skips next op if false.
/// Returns Ok(true) = condition true (execute next), Ok(false) = skip next.
/// LiftEffect: Neutral (stack read-only).
pub fn op_test(state: &mut CalcState, kind: TestKind) -> Result<(), HpError> {
    // Note: test result is communicated via the bool skip mechanism in run_loop,
    // not via this function return. This interactive dispatch arm is a no-op.
    // The real evaluation lives in evaluate_test() called from run_loop.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn evaluate_test(state: &CalcState, kind: &TestKind) -> bool {
    let x = state.stack.x.inner();
    let y = state.stack.y.inner();
    let zero = Decimal::ZERO;
    match kind {
        TestKind::XEqZero => x == zero,
        TestKind::XNeZero => x != zero,
        TestKind::XLtZero => x < zero,
        TestKind::XGtZero => x > zero,
        TestKind::XLeZero => x <= zero,
        TestKind::XGeZero => x >= zero,
        TestKind::XEqY    => x == y,
        TestKind::XNeY    => x != y,
        TestKind::XLtY    => x < y,
        TestKind::XGtY    => x > y,
        TestKind::XLeY    => x <= y,
        TestKind::XGeY    => x >= y,
    }
}
```

**run_program public API** — no direct analog; designed to match the `Result<(), HpError>` convention:
```rust
/// Execute a recorded program starting at the given label.
///
/// Clones state.program to avoid Rust borrow conflict (cannot hold &program[pc]
/// and &mut state simultaneously — standard Rust ownership constraint).
/// HP-41 programs are ≤999 steps; the clone is negligible.
pub fn run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError> {
    let program = state.program.clone();

    let start = program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
        .ok_or(HpError::InvalidOp)?;

    state.pc = start + 1;
    state.call_stack.clear();
    state.is_running = true;

    let result = run_loop(state, &program);

    state.is_running = false;  // always reset, even on error
    result
}

fn run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    loop {
        if state.pc >= program.len() {
            break; // ran off end = implicit top-level RTN
        }
        let op = program[state.pc].clone();
        state.pc += 1;

        match &op {
            Op::Rtn => match state.call_stack.pop() {
                Some(return_pc) => state.pc = return_pc,
                None => break,  // top-level RTN = normal termination
            },
            Op::Lbl(_) => { /* no-op during execution */ }
            Op::Gto(label) => {
                let target = find_in_program(program, label)?;
                state.pc = target + 1;
            }
            Op::Xeq(label) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth);
                }
                state.call_stack.push(state.pc);
                let target = find_in_program(program, label)?;
                state.pc = target + 1;
            }
            Op::Test(kind) => {
                if !evaluate_test(state, kind) {
                    state.pc += 1;  // skip next step (D-09: skip-if-false)
                }
            }
            Op::Isg(reg) => {
                if op_isg(state, *reg)? {
                    state.pc += 1;  // loop exit: skip next
                }
            }
            Op::Dse(reg) => {
                if op_dse(state, *reg)? {
                    state.pc += 1;  // loop exit: skip next
                }
            }
            other => {
                // All other ops execute without flush_entry_buf (no digit entry mid-program)
                execute_op(state, other.clone())?;
            }
        }
    }
    Ok(())
}

fn find_in_program(program: &[Op], label: &str) -> Result<usize, HpError> {
    program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == label))
        .ok_or(HpError::InvalidOp)
}

/// Private op executor for use inside run_loop.
/// Does NOT call flush_entry_buf and does NOT check prgm_mode.
/// Must cover all Op variants except the programming ops (handled above in run_loop).
fn execute_op(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    use crate::ops::{arithmetic::*, stack_ops::*, math::*, registers::*, alpha::*};
    match op {
        Op::Add        => op_add(state),
        Op::Sub        => op_sub(state),
        Op::Mul        => op_mul(state),
        Op::Div        => op_div(state),
        Op::Enter      => op_enter(state),
        Op::Clx        => op_clx(state),
        Op::Chs        => op_chs(state),
        Op::Rdn        => op_rdn(state),
        Op::XySwap     => op_xy_swap(state),
        Op::Lastx      => op_lastx(state),
        Op::PushNum(v) => { crate::stack::enter_number(state, v); Ok(()) }
        Op::Recip      => op_recip(state),
        Op::Sqrt       => op_sqrt(state),
        // ... all other non-programming ops ...
        // Programming ops (Lbl/Gto/Xeq/Rtn/Test/Isg/Dse/PrgmMode)
        // are handled by run_loop directly and must NOT appear here.
        _ => Err(HpError::InvalidOp),
    }
}
```

---

### `hp41-core/src/lib.rs` (config — add pub use)

**Analog:** `hp41-core/src/lib.rs` (self, lines 1–21)

**Full existing file** (lines 1–21):
```rust
pub mod error;
pub mod num;
pub mod state;
pub mod stack;
pub mod ops;
pub mod format;

pub use error::HpError;
pub use num::HpNum;
pub use state::{CalcState, Stack, AngleMode, DisplayMode};
pub use stack::LiftEffect;
pub use format::{format_hpnum, format_alpha};
```

**Phase 3 addition — append one line after `pub use format::{...}`:**
```rust
pub use ops::program::run_program;
```

**Pattern note:** All public re-exports are on `pub use` lines grouped by module. Follow the same grouping — no blank lines needed within the block.

---

### `hp41-core/tests/program_tests.rs` (test — NEW FILE)

**Analog:** `hp41-core/tests/register_tests.rs` (lines 1–159) and `hp41-core/tests/entry_buf_tests.rs` (lines 1–133).

**File header pattern** (from `register_tests.rs` lines 1–2):
```rust
//! Integration tests for PROG-01 + PROG-02: keystroke programming engine.
```

**Import pattern** (from `register_tests.rs` lines 3–6):
```rust
use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op, TestKind};
use hp41_core::run_program;
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Helper function pattern** (from `register_tests.rs` lines 7–9):
```rust
fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
```

**Section comment pattern** (from `register_tests.rs` lines 11–12):
```rust
// ── PRGM mode recording ──────────────────────────────────────────────────────
```

**Test structure pattern** (from `register_tests.rs` lines 13–23):
```rust
#[test]
fn test_prgm_mode_toggle() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::PrgmMode).unwrap();
    assert!(s.prgm_mode, "PrgmMode must set prgm_mode = true");
    dispatch(&mut s, Op::PrgmMode).unwrap();
    assert!(!s.prgm_mode, "Second PrgmMode must exit prgm_mode");
}
```

**Error assertion pattern** (from `register_tests.rs` lines 101–105):
```rust
#[test]
fn test_run_unknown_label_returns_invalid_op() {
    let mut s = CalcState::new();
    assert_eq!(run_program(&mut s, "X"), Err(HpError::InvalidOp));
}
```

**Direct state mutation pattern** (from `register_tests.rs` lines 17–20 and `entry_buf_tests.rs` lines 27–28):
```rust
// For ISG/DSE: set register directly, then run program
s.regs[0] = HpNum::from(/* value */);
// For entry_buf testing:
s.entry_buf = "1.00500".to_string();
// For lift state:
s.stack.lift_enabled = true;
```

**ISG success-criterion test skeleton** (the canonical PROG-02 case):
```rust
#[test]
fn test_isg_increments_4_times_before_skip() {
    // Success criterion: R00=1.00500 (current=1, final=5, step=1).
    // ISG runs 4 times (current: 1→2→3→4→5), then on 5th attempt 5>5 is false,
    // so 5+1=6 > 5 is true → skip. Total executions of loop body = 4.
    let mut s = CalcState::new();
    // Program: Lbl("A"), [body op], Isg(0), Gto("A"), Rtn
    // body op: StoReg(1) to count iterations in R01
    // ... build program, run, assert R01 == 4 and state.regs[0] encodes current=6
}
```

---

## Shared Patterns

### Result<(), HpError> Return Convention
**Source:** Every function in `hp41-core/src/ops/registers.rs`, `hp41-core/src/ops/alpha.rs`, `hp41-core/src/ops/math.rs`
**Apply to:** All functions in `ops/program.rs`

```rust
pub fn op_xxx(state: &mut CalcState) -> Result<(), HpError> {
    // ... work ...
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

ISG/DSE are the only exceptions — they return `Result<bool, HpError>` where `bool` signals skip. This is the same pattern already used by `checked_*` methods on `HpNum` that return `Result<HpNum, HpError>`.

### LiftEffect::Neutral for All Programming Ops
**Source:** `hp41-core/src/ops/alpha.rs` lines 15, 28, 36 — `apply_lift_effect(state, LiftEffect::Neutral)` as the last call before `Ok(())`.
**Apply to:** All functions in `ops/program.rs` (LBL, GTO, XEQ, RTN, PrgmMode, Test, ISG, DSE all declared Neutral per CONTEXT.md Claude's Discretion).

### Error-Before-Mutation Pattern
**Source:** `hp41-core/src/ops/registers.rs` `op_sto_arith` (lines 43–58) — compute first, write only on success.
**Apply to:** `op_xeq` (check `call_stack.len() >= 4` before pushing), `op_isg`/`op_dse` (parse counter before writing back), `op_gto`/`op_xeq` (resolve label before modifying pc).

```rust
// Pattern: validate → compute → write → apply_lift_effect → Ok(())
if state.call_stack.len() >= 4 {
    return Err(HpError::CallDepth);  // fail fast, no mutation
}
// ... only mutate state after all validation passes ...
```

### Integration Test Helper
**Source:** `hp41-core/tests/register_tests.rs` lines 7–9
**Apply to:** `hp41-core/tests/program_tests.rs` — use the same `push()` helper.

```rust
fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
```

### is_running Safety Reset
**Source:** No existing analog — new pattern for Phase 3.
**Apply to:** `run_program()` function only.

```rust
state.is_running = true;
let result = run_loop(state, &program);
state.is_running = false;   // always reset, even if run_loop returns Err
result
```

This matches the RAII guard pattern implied by the test `test_is_running_reset_on_error`.

---

## No Analog Found

All files have analogs. The following sub-patterns within `ops/program.rs` have no direct codebase analog:

| Sub-pattern | Role | Reason | Use Instead |
|-------------|------|---------|-------------|
| `run_loop()` interpreter loop | service | No loop-based op interpreter exists yet | RESEARCH.md Pattern 2 (verified design) |
| `parse_counter()` / `build_counter()` | utility | No string-split numeric parsing exists | RESEARCH.md Pattern 3 (exact algorithm) |
| `evaluate_test()` | utility | No conditional evaluator exists | RESEARCH.md Pattern 4 (exact code) |
| `execute_op()` private dispatcher | utility | No flush-free dispatch exists | Mirror `dispatch()` match arms, minus flush and prgm_mode gate |

---

## Metadata

**Analog search scope:** `hp41-core/src/`, `hp41-core/tests/`
**Files scanned:** 11 (state.rs, error.rs, ops/mod.rs, ops/registers.rs, ops/alpha.rs, ops/math.rs, stack.rs, num.rs, lib.rs, tests/register_tests.rs, tests/entry_buf_tests.rs)
**Pattern extraction date:** 2026-05-07
