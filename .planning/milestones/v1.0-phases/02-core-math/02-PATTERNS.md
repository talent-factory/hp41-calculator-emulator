# Phase 2: Core Math - Pattern Map

**Mapped:** 2026-05-06
**Files analyzed:** 13 (new/modified files from CONTEXT.md + RESEARCH.md)
**Analogs found:** 13 / 13

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/Cargo.toml` | config | — | `hp41-core/Cargo.toml` (self) | exact |
| `hp41-core/src/state.rs` | model | CRUD | `hp41-core/src/state.rs` (self, extend) | exact |
| `hp41-core/src/num.rs` | model/utility | transform | `hp41-core/src/num.rs` (self, extend) | exact |
| `hp41-core/src/ops/math.rs` | service | request-response | `hp41-core/src/ops/arithmetic.rs` | exact (same role + data flow) |
| `hp41-core/src/ops/registers.rs` | service | CRUD | `hp41-core/src/ops/arithmetic.rs` | role-match |
| `hp41-core/src/ops/alpha.rs` | service | CRUD | `hp41-core/src/ops/stack_ops.rs` | role-match |
| `hp41-core/src/format.rs` | utility | transform | `hp41-core/src/num.rs` (Display impl) | partial-match |
| `hp41-core/src/ops/mod.rs` | controller | request-response | `hp41-core/src/ops/mod.rs` (self, extend) | exact |
| `hp41-core/tests/math_tests.rs` | test | — | `hp41-core/tests/stack_tests.rs` | exact |
| `hp41-core/tests/trig_tests.rs` | test | — | `hp41-core/tests/stack_tests.rs` | exact |
| `hp41-core/tests/format_tests.rs` | test | — | `hp41-core/src/tests.rs` (num_tests module) | role-match |
| `hp41-core/tests/register_tests.rs` | test | — | `hp41-core/tests/stack_tests.rs` | exact |
| `hp41-core/tests/alpha_tests.rs` | test | — | `hp41-core/tests/stack_tests.rs` | exact |

---

## Pattern Assignments

### `hp41-core/Cargo.toml` (config)

**Analog:** `hp41-core/Cargo.toml` (current file)

**Current content** (lines 1–13):
```toml
[package]
name = "hp41-core"
version = "0.1.0"
edition = "2021"

[dependencies]
rust_decimal = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
proptest = "1.11"
insta = { version = "1.47", features = ["yaml"] }
```

**Required change** (line 7 only — add features flag):
```toml
rust_decimal = { workspace = true, features = ["maths"] }
```

The workspace `Cargo.toml` is NOT changed. The `features = ["maths"]` is crate-local. `maths-nopanic` is NOT needed because all calls use `checked_*` variants.

---

### `hp41-core/src/state.rs` (model, CRUD)

**Analog:** `hp41-core/src/state.rs` (self — extend the existing struct)

**Existing struct pattern** (lines 31–43) — copy this and add fields:
```rust
#[derive(Debug, Clone)]
pub struct CalcState {
    pub stack: Stack,
    // Phase 2 additions: regs: [HpNum; 100], alpha: String, flags: CalcFlags
}

impl CalcState {
    pub fn new() -> Self {
        CalcState {
            stack: Stack::new(),
        }
    }
}

impl Default for CalcState {
    fn default() -> Self {
        Self::new()
    }
}
```

**New fields to add** (from CONTEXT.md decisions):
```rust
pub struct CalcState {
    pub stack: Stack,
    pub regs: [HpNum; 100],       // R00–R99, 0-indexed; all zero on startup
    pub alpha_reg: String,         // max 24 chars
    pub alpha_mode: bool,          // true = keyboard sends alpha chars
    pub angle_mode: AngleMode,     // DEG/RAD/GRAD; default DEG
    pub display_mode: DisplayMode, // Fix(u8)/Sci(u8)/Eng(u8); u8 = digit count 0–9
    pub entry_buf: String,         // pending digit string; empty = not in entry
}
```

**New enums to add in state.rs (or a dedicated types module):**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleMode { Deg, Rad, Grad }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayMode { Fix(u8), Sci(u8), Eng(u8) }
```

**CalcState::new() initializer pattern** — follow existing `Stack::new()` style (lines 72–83):
```rust
impl CalcState {
    pub fn new() -> Self {
        CalcState {
            stack: Stack::new(),
            regs: std::array::from_fn(|_| HpNum::zero()),  // avoids Copy requirement
            alpha_reg: String::new(),
            alpha_mode: false,
            angle_mode: AngleMode::Deg,           // HP-41 hardware default
            display_mode: DisplayMode::Fix(4),    // HP-41 hardware default
            entry_buf: String::new(),
        }
    }
}
```

**HpNum::Default prerequisite** — add to `num.rs` before `[HpNum; 100]` works with `Default::default()`:
```rust
impl Default for HpNum {
    fn default() -> Self { HpNum::zero() }
}
```

---

### `hp41-core/src/num.rs` (model/utility, transform)

**Analog:** `hp41-core/src/num.rs` (self — extend the existing impl block)

**Existing checked method pattern** (lines 26–51) — all new math methods MUST follow this signature and `HpNum::rounded()` return:
```rust
pub fn checked_add(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
    self.0.checked_add(rhs.0)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

pub fn checked_div(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
    if rhs.0.is_zero() {
        return Err(HpError::DivideByZero);
    }
    self.0.checked_div(rhs.0)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}
```

**Import to add at top of num.rs:**
```rust
use rust_decimal::MathematicalOps;
use rust_decimal::prelude::ToPrimitive;  // for .to_f64() in f64 bridge
```

**New math methods — pattern for rust_decimal maths feature (native decimal):**
```rust
// Domain guard BEFORE checked_* call; checked_* returns None for BOTH domain AND overflow.
// Distinguish the two manually when the domain condition is knowable.
pub fn checked_ln(&self) -> Result<HpNum, HpError> {
    if self.0 <= Decimal::ZERO {
        return Err(HpError::Domain);
    }
    self.0.checked_ln()
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

pub fn checked_sqrt(&self) -> Result<HpNum, HpError> {
    if self.0 < Decimal::ZERO {
        return Err(HpError::Domain);
    }
    self.0.checked_sqrt()
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

pub fn checked_exp(&self) -> Result<HpNum, HpError> {
    self.0.checked_exp()
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

// sin/cos/tan: no domain guard needed; checked_* returns None only on overflow
pub fn checked_sin(&self) -> Result<HpNum, HpError> {
    self.0.checked_sin()
        .map(HpNum::rounded)
        .ok_or(HpError::Domain)
}
```

**New math methods — f64 bridge pattern for asin/acos/atan (not in rust_decimal maths):**
```rust
pub fn checked_asin(&self) -> Result<HpNum, HpError> {
    let v = self.0.to_f64().ok_or(HpError::Overflow)?;
    if !(-1.0..=1.0).contains(&v) {
        return Err(HpError::Domain);
    }
    Decimal::from_f64(v.asin())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}
// atan has no domain restriction — same pattern, remove the range check
```

**x² pattern — use checked_mul, NOT powd:**
```rust
// x² = x * x — reuses existing checked_mul, no maths feature path needed
pub fn checked_sq(&self) -> Result<HpNum, HpError> {
    self.checked_mul(self)
}
```

**Recip (1/x) pattern:**
```rust
pub fn checked_recip(&self) -> Result<HpNum, HpError> {
    HpNum::from(1).checked_div(self)   // reuses existing checked_div + DivideByZero guard
}
```

---

### `hp41-core/src/ops/math.rs` (service, request-response)

**Analog:** `hp41-core/src/ops/arithmetic.rs` — exact role + data flow match

**Imports pattern** (copy from arithmetic.rs lines 1–3, extend):
```rust
use crate::error::HpError;
use crate::state::{CalcState, AngleMode};
use crate::stack::{apply_lift_effect, unary_result, binary_result, LiftEffect};
```

**Core unary op pattern** (mirrors arithmetic.rs op_add structure):
```rust
/// 1/x: reciprocal of X
/// LiftEffect: Enable (via unary_result)
pub fn op_recip(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_recip()?;
    unary_result(state, result);
    Ok(())
}

/// LN: natural log of X
/// Domain error if X ≤ 0
pub fn op_ln(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.x.checked_ln()?;
    unary_result(state, result);
    Ok(())
}
```

**Y^X binary op pattern** (uses binary_result, same as arithmetic.rs):
```rust
/// Y^X: raise Y to the power of X
/// LiftEffect: Enable (via binary_result — consumes Y)
pub fn op_ypow(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_powd(&state.stack.x)?;
    binary_result(state, result);
    Ok(())
}
```

**Trig with angle mode conversion pattern:**
```rust
// Module-private helpers — not pub
fn to_radians(x: &crate::num::HpNum, mode: AngleMode) -> Result<crate::num::HpNum, HpError> {
    use rust_decimal::Decimal;
    match mode {
        AngleMode::Deg  => x.checked_mul(&/* PI_OVER_180 constant */),
        AngleMode::Rad  => Ok(x.clone()),
        AngleMode::Grad => x.checked_mul(&/* PI_OVER_200 constant */),
    }
}

fn from_radians(x: &crate::num::HpNum, mode: AngleMode) -> Result<crate::num::HpNum, HpError> {
    match mode {
        AngleMode::Deg  => x.checked_mul(&/* DEG_PER_RAD constant */),
        AngleMode::Rad  => Ok(x.clone()),
        AngleMode::Grad => x.checked_mul(&/* GRAD_PER_RAD constant */),
    }
}

pub fn op_sin(state: &mut CalcState) -> Result<(), HpError> {
    let radians = to_radians(&state.stack.x, state.angle_mode)?;
    let result = radians.checked_sin()?;
    unary_result(state, result);
    Ok(())
}

pub fn op_asin(state: &mut CalcState) -> Result<(), HpError> {
    let result_rad = state.stack.x.checked_asin()?;  // returns radians
    let result = from_radians(&result_rad, state.angle_mode)?;  // convert to angle_mode
    unary_result(state, result);
    Ok(())
}
```

**Angle mode op pattern** (Neutral lift — matches stack_ops.rs op_chs structure):
```rust
pub fn op_set_deg(state: &mut CalcState) -> Result<(), HpError> {
    state.angle_mode = AngleMode::Deg;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

---

### `hp41-core/src/ops/registers.rs` (service, CRUD)

**Analog:** `hp41-core/src/ops/arithmetic.rs` (role-match) + `hp41-core/src/ops/stack_ops.rs` (Neutral lift pattern)

**Imports pattern:**
```rust
use crate::error::HpError;
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
```

**STO pattern** (Neutral lift — matches op_chs / op_rdn in stack_ops.rs):
```rust
/// STO n: store X into register n. Neutral lift, no LASTX save.
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    state.regs[reg as usize] = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**RCL pattern** (Enable lift — matches op_lastx in stack_ops.rs lines 76–83):
```rust
/// RCL n: recall register n into X (with stack lift). Enable lift, no LASTX save.
pub fn op_rcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    let val = state.regs[reg as usize].clone();
    state.stack.lift_enabled = true;       // force lift before enter_number
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**STO arith pattern** (compute THEN write — atomicity requirement from RESEARCH.md Pitfall 6):
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StoArithKind { Add, Sub, Mul, Div }

pub fn op_sto_arith(state: &mut CalcState, reg: u8, kind: StoArithKind)
    -> Result<(), HpError>
{
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    // Compute first, write only on success — Pitfall 6 guard
    let new_val = match kind {
        StoArithKind::Add => state.regs[reg as usize].checked_add(&state.stack.x)?,
        StoArithKind::Sub => state.regs[reg as usize].checked_sub(&state.stack.x)?,
        StoArithKind::Mul => state.regs[reg as usize].checked_mul(&state.stack.x)?,
        StoArithKind::Div => state.regs[reg as usize].checked_div(&state.stack.x)?,
    };
    state.regs[reg as usize] = new_val;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**CLREG pattern:**
```rust
pub fn op_clreg(state: &mut CalcState) -> Result<(), HpError> {
    state.regs = std::array::from_fn(|_| crate::num::HpNum::zero());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

---

### `hp41-core/src/ops/alpha.rs` (service, CRUD)

**Analog:** `hp41-core/src/ops/stack_ops.rs` — same Neutral lift + direct state mutation pattern

**Imports pattern** (copy from stack_ops.rs lines 1–4, trim unused):
```rust
use crate::error::HpError;
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, LiftEffect};
```

**AlphaToggle pattern** (matches op_chs structure — single field flip, Neutral lift):
```rust
/// ALPHA toggle: flip alpha_mode flag. Neutral lift.
pub fn op_alpha_toggle(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_mode = !state.alpha_mode;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**AlphaAppend pattern** (24-char enforcement — silent discard per RESEARCH.md):
```rust
/// AlphaAppend: append char to alpha_reg (max 24 chars, silent discard on overflow).
pub fn op_alpha_append(state: &mut CalcState, ch: char) -> Result<(), HpError> {
    if state.alpha_reg.chars().count() < 24 {
        state.alpha_reg.push(ch);
    }
    // HP-41 hardware silently discards excess chars — no error
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**AlphaClear pattern:**
```rust
/// AlphaClear: clear alpha_reg. Neutral lift.
pub fn op_alpha_clear(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_reg.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

---

### `hp41-core/src/format.rs` (utility, transform)

**Analog:** `hp41-core/src/num.rs` `Display` impl (lines 74–78) — same pattern of producing a `String` from `HpNum`

**Module structure pattern** (new file — no existing analog for FIX/SCI/ENG formatting):
```rust
use crate::num::HpNum;
use crate::state::DisplayMode;
use rust_decimal::Decimal;

/// Format an HpNum according to the current DisplayMode.
/// Returns the HP-41-style display string.
pub fn format_hpnum(n: &HpNum, mode: &DisplayMode) -> String {
    // ...
}

/// Format the alpha register for display (direct truncation to 12 chars if needed).
pub fn format_alpha(reg: &str) -> String {
    reg.chars().take(12).collect()
}
```

**FIX n formatting approach** (use Rust's built-in `format!` — from RESEARCH.md "Don't Hand-Roll"):
```rust
// FIX n: trailing zeros shown. Overflow to SCI when integer part exceeds display width.
// n = mode digit count (0–9)
DisplayMode::Fix(n) => {
    let digits = *n as usize;
    // Use Decimal's string formatting via format!
    format!("{:.prec$}", inner_decimal, prec = digits)
    // Then check overflow condition and fall back to SCI if needed
}
```

**SCI n formatting approach:**
```rust
// SCI n: uppercase E, 2-digit exponent, space before positive exponent
// HP-41 format: "2.9979E 08" (space) vs "2.9979E-08" (minus)
// standard format!("{:.prec$e}") produces lowercase 'e' — post-process to uppercase
DisplayMode::Sci(n) => {
    let digits = *n as usize;
    // format then transform e→E, adjust exponent padding to 2 digits
}
```

**ENG n approach** (requires ~20 lines of custom exponent rounding — no Rust built-in):
```rust
// ENG n: exponent is always multiple of 3; mantissa is 1–3 digits before decimal
// Algorithm: round to (n+1) sig digits, find largest multiple-of-3 exponent
// that makes mantissa ≥ 1 and < 1000
DisplayMode::Eng(n) => {
    // custom exponent clamping logic — see RESEARCH.md Pattern 5 for algorithm
}
```

---

### `hp41-core/src/ops/mod.rs` (controller, request-response)

**Analog:** `hp41-core/src/ops/mod.rs` (self — extend existing enum and dispatch)

**Existing Op enum pattern** (lines 18–34) — add new variants following same naming conventions:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // Arithmetic (Phase 1)
    Add, Sub, Mul, Div,
    // Stack operations (Phase 1)
    Enter, Clx, Chs, Rdn, XySwap, Lastx,
    PushNum(HpNum),
    // Phase 2 additions — follow same CamelCase naming:
    Recip, Sqrt, Sq, YPow,
    Ln, Log, Exp, TenPow,
    Sin, Cos, Tan, Asin, Acos, Atan,
    SetDeg, SetRad, SetGrad,
    FmtFix(u8), FmtSci(u8), FmtEng(u8),
    StoReg(u8), RclReg(u8),
    StoArith { reg: u8, kind: StoArithKind },  // struct variant — consistent with CONTEXT.md
    Clreg,
    AlphaToggle, AlphaAppend(char), AlphaClear,
}
```

**Existing dispatch pattern** (lines 40–57) — extend the match arm, maintain same style:
```rust
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    match op {
        Op::Add        => op_add(state),
        // ... existing arms ...
        Op::PushNum(v) => { crate::stack::enter_number(state, v); Ok(()) }
        // Phase 2 — same one-liner style:
        Op::Recip      => math::op_recip(state),
        Op::Sin        => math::op_sin(state),
        Op::StoReg(r)  => registers::op_sto(state, r),
        Op::RclReg(r)  => registers::op_rcl(state, r),
        Op::StoArith { reg, kind } => registers::op_sto_arith(state, reg, kind),
        Op::AlphaAppend(ch) => alpha::op_alpha_append(state, ch),
        // ...
    }
}
```

**Module declaration additions** (extend lines 5–6):
```rust
pub mod arithmetic;
pub mod stack_ops;
pub mod math;       // new Phase 2
pub mod registers;  // new Phase 2
pub mod alpha;      // new Phase 2
```

**StoArithKind import** — add to mod.rs use block or re-export from registers.rs:
```rust
pub use registers::StoArithKind;  // so callers can do ops::StoArithKind::Add
```

---

### `hp41-core/tests/math_tests.rs` (test)

**Analog:** `hp41-core/tests/stack_tests.rs` — exact structure match (integration test file using public API)

**File header pattern** (copy from stack_tests.rs lines 1–5):
```rust
//! Integration tests for MATH-01: unary math ops, 10-digit accuracy, LASTX behavior.

use hp41_core::{CalcState, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Helper function pattern** (copy from stack_tests.rs lines 6–8):
```rust
fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
```

**Test function pattern** (copy from stack_tests.rs — assert_eq! with .inner() for exact decimal comparison):
```rust
#[test]
fn test_recip_of_4_is_0_25() {
    let mut state = CalcState::new();
    push(&mut state, 4);
    dispatch(&mut state, Op::Recip).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from_str("0.25").unwrap());
}

#[test]
fn test_unary_op_saves_lastx() {
    // After SIN: LASTX must be the pre-sin X value
    let mut state = CalcState::new();
    push(&mut state, 1);
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Sin).unwrap();
    assert_eq!(state.stack.lastx.inner(), Decimal::from(1));
}

#[test]
fn test_unary_op_enables_lift() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = false;
    push(&mut state, 2);
    dispatch(&mut state, Op::Sqrt).unwrap();
    assert!(state.stack.lift_enabled, "Sqrt must enable lift");
}
```

**Accuracy test pattern** (use insta snapshot for 10-digit golden values):
```rust
#[test]
fn test_ln_2_accuracy() {
    let mut state = CalcState::new();
    // LN(2) = 0.6931471806 (10 sig digits)
    state.stack.x = HpNum::from(Decimal::from(2));
    dispatch(&mut state, Op::Ln).unwrap();
    // Snapshot the exact decimal value — reviewed once, then locked
    insta::assert_yaml_snapshot!(state.stack.x.inner().to_string());
}
```

---

### `hp41-core/tests/trig_tests.rs` (test)

**Analog:** `hp41-core/tests/stack_tests.rs` (same structure)

**File header pattern:**
```rust
//! Integration tests for MATH-02: trig ops in DEG/RAD/GRAD modes.

use hp41_core::{CalcState, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Angle mode setup pattern** (uses new Op variants):
```rust
fn set_deg(state: &mut CalcState) {
    dispatch(state, Op::SetDeg).unwrap();
}
fn set_rad(state: &mut CalcState) {
    dispatch(state, Op::SetRad).unwrap();
}
```

**Trig accuracy test pattern:**
```rust
#[test]
fn test_sin_30_deg_is_0_5() {
    let mut state = CalcState::new();
    set_deg(&mut state);
    state.stack.x = HpNum::from(30);
    dispatch(&mut state, Op::Sin).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from_str("0.5").unwrap());
}

#[test]
fn test_asin_0_5_is_30_deg() {
    // Inverse trig: result must be in angle_mode units
    let mut state = CalcState::new();
    set_deg(&mut state);
    state.stack.x = HpNum::from_str_radix("0.5", 10)...;
    dispatch(&mut state, Op::Asin).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(30));
}
```

---

### `hp41-core/tests/format_tests.rs` (test)

**Analog:** `hp41-core/src/tests.rs` `num_tests` module (lines 31–111) for exact value assertions; `hp41-core/tests/stack_tests.rs` for file structure

**File header pattern:**
```rust
//! Integration tests for MATH-03: FIX/SCI/ENG display formatting.

use hp41_core::HpNum;
use hp41_core::format::format_hpnum;   // new public function
use hp41_core::state::DisplayMode;
use rust_decimal::Decimal;
use std::str::FromStr;
```

**Format test pattern** (assert_eq! on String output):
```rust
#[test]
fn test_fix4_trailing_zeros() {
    let n = HpNum::from(1);
    let s = format_hpnum(&n, &DisplayMode::Fix(4));
    assert_eq!(s, "1.0000");
}

#[test]
fn test_sci4_uppercase_e() {
    let n = HpNum::from(Decimal::from_str("299792500").unwrap());
    let s = format_hpnum(&n, &DisplayMode::Sci(4));
    assert_eq!(s, "2.9979E 08");
}

#[test]
fn test_eng3_exponent_multiple_of_3() {
    let n = HpNum::from(Decimal::from_str("12345.678").unwrap());
    let s = format_hpnum(&n, &DisplayMode::Eng(3));
    assert_eq!(s, "12.346E 03");
}
```

---

### `hp41-core/tests/register_tests.rs` (test)

**Analog:** `hp41-core/tests/stack_tests.rs` (same structure — uses dispatch + assert on state fields)

**File header pattern:**
```rust
//! Integration tests for REGS-01: STO/RCL/STO-arith register operations.

use hp41_core::{CalcState, HpNum};
use hp41_core::ops::{dispatch, Op, StoArithKind};
use rust_decimal::Decimal;
```

**Register test pattern:**
```rust
#[test]
fn test_sto_rcl_round_trip() {
    let mut state = CalcState::new();
    state.stack.x = HpNum::from(42);
    dispatch(&mut state, Op::StoReg(5)).unwrap();
    state.stack.x = HpNum::zero();  // clear X
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::RclReg(5)).unwrap();
    assert_eq!(state.stack.x.inner(), Decimal::from(42));
}

#[test]
fn test_sto_is_neutral_lift() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::StoReg(0)).unwrap();
    assert!(!state.stack.lift_enabled, "STO must be Neutral — must not set lift");
}

#[test]
fn test_rcl_enables_lift() {
    let mut state = CalcState::new();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::RclReg(0)).unwrap();
    assert!(state.stack.lift_enabled, "RCL must enable lift");
}

#[test]
fn test_sto_add_updates_register() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(10);
    state.stack.x = HpNum::from(3);
    dispatch(&mut state, Op::StoArith { reg: 5, kind: StoArithKind::Add }).unwrap();
    assert_eq!(state.regs[5].inner(), Decimal::from(13));
    assert_eq!(state.stack.x.inner(), Decimal::from(3)); // X unchanged
}
```

---

### `hp41-core/tests/alpha_tests.rs` (test)

**Analog:** `hp41-core/tests/stack_tests.rs` (same structure)

**File header pattern:**
```rust
//! Integration tests for ALPH-01: ALPHA register mode.

use hp41_core::{CalcState, HpNum};
use hp41_core::ops::{dispatch, Op};
```

**Alpha test pattern:**
```rust
#[test]
fn test_alpha_append_builds_string() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::AlphaAppend('H')).unwrap();
    dispatch(&mut state, Op::AlphaAppend('P')).unwrap();
    assert_eq!(state.alpha_reg, "HP");
}

#[test]
fn test_alpha_24_char_limit_enforced() {
    let mut state = CalcState::new();
    for c in "ABCDEFGHIJKLMNOPQRSTUVWXY".chars() {  // 25 chars
        dispatch(&mut state, Op::AlphaAppend(c)).unwrap();
    }
    assert_eq!(state.alpha_reg.chars().count(), 24, "must stop at 24");
    assert!(!state.alpha_reg.contains('Y'), "25th char must be silently discarded");
}

#[test]
fn test_alpha_clear_empties_register() {
    let mut state = CalcState::new();
    state.alpha_reg = "TEST".to_string();
    dispatch(&mut state, Op::AlphaClear).unwrap();
    assert!(state.alpha_reg.is_empty());
}

#[test]
fn test_alpha_toggle_flips_flag() {
    let mut state = CalcState::new();
    assert!(!state.alpha_mode);
    dispatch(&mut state, Op::AlphaToggle).unwrap();
    assert!(state.alpha_mode);
    dispatch(&mut state, Op::AlphaToggle).unwrap();
    assert!(!state.alpha_mode);
}
```

---

## Shared Patterns

### unary_result() helper (NEW — add to stack.rs)

**Source:** `hp41-core/src/stack.rs` — add alongside `binary_result()` (lines 46–58)

**Apply to:** All unary math ops in `ops/math.rs` (Recip, Sqrt, Sq, Ln, Log, Exp, TenPow, Sin, Cos, Tan, Asin, Acos, Atan)

```rust
// Modeled after binary_result() — same LASTX save; differs in that Y/Z/T are NOT modified
pub fn unary_result(state: &mut CalcState, result: HpNum) {
    // Save X to LASTX BEFORE overwriting — same ordering as binary_result
    state.stack.lastx = state.stack.x.clone();
    // Place result in X — Y, Z, T are unchanged (unary: no stack drop)
    state.stack.x = result;
    // Unary results always enable lift (same as binary)
    state.stack.lift_enabled = true;
}
```

**Critical:** Every unary math op MUST use `unary_result()`. Using `state.stack.x = result` directly skips LASTX save — the most common HP emulator bug (RESEARCH.md Pitfall 1).

---

### LiftEffect declaration (existing — REQUIRED for all new ops)

**Source:** `hp41-core/src/stack.rs` lines 3–24 (LiftEffect enum + apply_lift_effect)

**Apply to:** Every new op function; every function MUST call `apply_lift_effect(state, LiftEffect::X)` as its last action before `Ok(())`.

**Phase 2 lift effect summary:**
- `Enable`: All unary math (Recip, Sqrt, Sq, Ln, Log, Exp, TenPow, Sin, Cos, Tan, Asin, Acos, Atan), YPow (via binary_result), RclReg
- `Neutral`: StoReg, StoArith, Clreg, FmtFix/Sci/Eng, SetDeg/Rad/Grad, AlphaToggle, AlphaAppend, AlphaClear
- `Disable`: (no new Phase 2 ops use Disable)

---

### Error handling (existing — no changes)

**Source:** `hp41-core/src/error.rs` (all 13 lines)

**Apply to:** All new op functions; always propagate with `?` operator

```rust
// All four variants needed in Phase 2:
HpError::Overflow    // checked_* returns None on overflow → .ok_or(HpError::Overflow)
HpError::DivideByZero // checked_div guard → already in HpNum::checked_div
HpError::InvalidOp   // register bounds check: if reg >= 100 { return Err(HpError::InvalidOp); }
HpError::Domain      // pre-guard on ln(≤0), sqrt(<0), asin/acos(outside ±1)
```

---

### Result<(), HpError> signature (existing — invariant)

**Source:** `hp41-core/src/ops/arithmetic.rs` lines 7, 16, 24, 33

**Apply to:** Every op function in math.rs, registers.rs, alpha.rs

Every public op function signature MUST be:
```rust
pub fn op_xxx(state: &mut CalcState) -> Result<(), HpError>
// or with parameters:
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError>
pub fn op_alpha_append(state: &mut CalcState, ch: char) -> Result<(), HpError>
```

No panics. No unwrap(). No f64 arithmetic on register values (only in f64 bridge methods on HpNum).

---

### Doc comment pattern (existing — mandatory)

**Source:** `hp41-core/src/ops/arithmetic.rs` lines 5–6, 14–16, 22–23, 31–33; `hp41-core/src/ops/stack_ops.rs` lines 1–13

Every pub op function MUST have a doc comment declaring:
1. What it does (HP-41 name in parentheses)
2. LiftEffect

```rust
/// STO n: copy X register into storage register n. Stack unchanged.
/// LiftEffect: Neutral
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
```

---

## No Analog Found

All Phase 2 files have analogs in the codebase. No file requires falling back to RESEARCH.md patterns exclusively.

| File | Closest Match | Gap |
|------|---------------|-----|
| `src/format.rs` | `src/num.rs` Display impl | No FIX/SCI/ENG formatter exists; algorithm from RESEARCH.md "HP-41 Display Formatting Rules" section |
| `src/ops/math.rs` trig constants | — | PI_OVER_180, DEG_PER_RAD, GRAD_PER_RAD constants needed; use `Decimal::from_str("0.01745329252")` etc. — verify via RESEARCH.md |

---

## Metadata

**Analog search scope:** `hp41-core/src/`, `hp41-core/tests/`
**Files scanned:** 13 source files, all read in full
**Pattern extraction date:** 2026-05-06
