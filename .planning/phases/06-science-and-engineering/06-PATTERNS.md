# Phase 6: Science & Engineering - Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 8 (6 new, 2 modified in hp41-core; 2 modified in hp41-cli)
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-core/src/ops/stats.rs` | service (op module) | CRUD (register read/write) | `hp41-core/src/ops/registers.rs` | role-match |
| `hp41-core/src/ops/hms.rs` | service (op module) | transform | `hp41-core/src/ops/program.rs` (`parse_counter`) | data-flow-match |
| `hp41-core/src/ops/mod.rs` | config / dispatch | request-response | `hp41-core/src/ops/mod.rs` (self — extend existing) | exact |
| `hp41-core/src/error.rs` | model | — | `hp41-core/src/error.rs` (self — add variant) | exact |
| `hp41-core/tests/stats_tests.rs` | test | CRUD | `hp41-core/tests/register_tests.rs` | exact |
| `hp41-core/tests/hms_tests.rs` | test | transform | `hp41-core/tests/math_tests.rs` | role-match |
| `hp41-cli/src/keys.rs` | config | request-response | `hp41-cli/src/keys.rs` (self — extend existing) | exact |
| `hp41-cli/src/help_data.rs` | config | — | `hp41-cli/src/help_data.rs` (self — extend existing) | exact |

---

## Pattern Assignments

### `hp41-core/src/ops/stats.rs` (op module, CRUD — register read/write)

**Primary analog:** `hp41-core/src/ops/registers.rs`
**Secondary analog:** `hp41-core/src/ops/arithmetic.rs` (for binary two-operand pattern)

**Imports pattern** (`registers.rs` lines 1–9):
```rust
use crate::error::HpError;
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::num::HpNum;
```
Note: stats.rs does NOT need `binary_result` or `unary_result` — Σ+ has non-standard stack semantics. Import `enter_number` and `apply_lift_effect` directly (same as `op_rcl`).

**Core Σ accumulation pattern** (`registers.rs` lines 25–36 — `op_rcl` as model):
```rust
// op_rcl pattern — enter_number + LiftEffect::Enable for pushing results
pub fn op_rcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let val = state.regs[reg as usize].clone();
    state.stack.lift_enabled = true;   // force lift before enter_number
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```
For Σ+: do NOT call `binary_result()` (it saves LASTX and drops Y). Directly mutate `state.regs[1..=6]`, then push n into X using the `op_rcl` push pattern above. R01–R06 are at indices 1–6 in `state.regs` (0-indexed Vec).

**Register write pattern** (`registers.rs` lines 14–21 — `op_sto`):
```rust
// Direct register write without stack manipulation
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    state.regs[reg as usize] = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```
For CLΣSTAT: copy this pattern but write `HpNum::zero()` to indices 1–6.

**Two-result push pattern** (`registers.rs` `op_rcl` repeated — MEAN, SDEV, L.R.):
Push ȳ/σy/m (goes to Y), then x̄/σx/b (goes to X). Each push must set `lift_enabled = true` before calling `enter_number`, then call `apply_lift_effect(state, LiftEffect::Enable)` to leave lift enabled after.
```rust
// Two-value result push (MEAN, SDEV, L.R.)
state.stack.lift_enabled = true;
enter_number(state, y_result);   // y_result → X, old X → Y → Z → T
apply_lift_effect(state, LiftEffect::Enable);
state.stack.lift_enabled = true;
enter_number(state, x_result);   // x_result → X, y_result → Y
apply_lift_effect(state, LiftEffect::Enable);
```

**HpNum arithmetic chain** (`hp41-core/src/num.rs` lines 33–58):
```rust
// All arithmetic through HpNum checked_* methods — no raw Decimal operations
state.regs[1].checked_add(&x.checked_sq()?)?   // Σx² += x²
state.regs[3].checked_add(&HpNum::from(1i32))?  // n++
n.checked_mul(sum_xy)?                           // n·Σxy
    .checked_sub(&sum_x.checked_mul(sum_y)?)?    // n·Σxy - Σx·Σy
denom_x.checked_mul(&denom_y)?.checked_sqrt()?  // √(denom_x · denom_y)
```

**Error handling pattern** (`math.rs` lines 64–68 for domain errors):
```rust
// Domain guard — used for SDEV (n<2), CORR (zero denominator), MEAN (n=0)
if n.is_zero() {
    return Err(HpError::InvalidOp);   // not enough data
}
```
Use `HpError::InvalidOp` for insufficient data (n=0, n<2). Use `HpError::Domain` for mathematical domain errors (sqrt of negative from CORR denominator via `checked_sqrt()?`). Use `HpError::DivideByZero` propagated from `checked_div()?`.

---

### `hp41-core/src/ops/hms.rs` (op module, transform)

**Primary analog:** `hp41-core/src/ops/program.rs` — `parse_counter()` function (lines 344–360)
**Secondary analog:** `hp41-core/src/ops/math.rs` — `op_sqrt` / `unary_result` pattern (for single-result HMS ops)

**Imports pattern** (`program.rs` lines 11–18):
```rust
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::HpError;
use crate::num::HpNum;
use crate::state::CalcState;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
```
HMS→ and →HMS are unary (read X, write X) — use `unary_result()` from `stack.rs`.
HMS+ and HMS− are binary (read X and Y, write X with drop) — use `binary_result()` from `stack.rs`.

**String-split field extraction pattern** (`program.rs` lines 344–360 — CRITICAL):
```rust
pub fn parse_counter(n: &HpNum) -> Result<(i64, i64, i64, String), HpError> {
    let s = n.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let current: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // CRITICAL: {:0<5} = left-align = right-pad with zeros (NOT {:0>5} which pads LEFT)
    let frac_padded = format!("{:0<5}", frac_part);
    let frac_padded = if frac_padded.len() > 5 { frac_padded[..5].to_string() } else { frac_padded };
    // ...
}
```
For HMS: adapt to 4-char fractional field (MMSS) instead of 5-char (FFFDD). Key differences:
- Pad to 4 chars: `format!("{:0<4}", frac_part)` (right-pad, NOT left-pad)
- `frac_padded[..2]` = minutes (MM), `frac_padded[2..4]` = seconds (SS)
- Extract sign flag first: `n.inner().is_sign_negative()` → work with `n.inner().abs()` throughout

**Negative value handling** (`program.rs` pattern + RESEARCH.md Pitfall 8):
```rust
// Always extract sign before string-splitting — negative Decimal to_string() = "-1.3045"
// int_part would be "-1" which parses correctly as i64(-1), but frac must be from abs value
let is_neg = n.inner().is_sign_negative();
let abs_val = n.inner().abs();
let s = abs_val.to_string();
// ... parse int_part and frac_part from s ...
// Apply sign to final result only
```

**Validation pattern** (new — `HpError::InvalidInput` required before HMS ops compile):
```rust
if minutes >= 60 || seconds >= 60 {
    return Err(HpError::InvalidInput);
}
```
This error variant does not yet exist in `error.rs` — it must be added in Wave 0 before hms.rs.

**H.MMSS reconstruction pattern** (adapt from `build_counter` in `program.rs` lines 364–368):
```rust
fn build_counter(current: i64, frac_padded: &str) -> Result<HpNum, HpError> {
    let s = format!("{}.{}", current, frac_padded);
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::rounded(d))
}
```
For →HMS: construct as `format!("{}.{:02}{:02}", hours, minutes, seconds_int)` using integer minutes/seconds — never float formatting. Parse back to `Decimal` via `Decimal::from_str`, then wrap in `HpNum::rounded`.

---

### `hp41-core/src/ops/mod.rs` (dispatch — extend existing)

**Analog:** Self — extend existing file at `/hp41-core/src/ops/mod.rs`

**Op enum extension pattern** (lines 54–156):
```rust
// Pattern: add new phase block after last existing phase block
// Current last block (Phase 5, lines 150–155):
// ── USER mode (Phase 5) ──────────────────────────────────────────────
/// USER mode toggle: flip state.user_mode. LiftEffect: Neutral.
UserMode,
// ── ALPHA backspace (Phase 5) ────────────────────────────────────────
/// ALPHA backspace: remove last char from alpha_reg (HP-41 ← key). LiftEffect: Neutral.
AlphaBackspace,
```
Add new block after `AlphaBackspace`:
```rust
// ── Science & Engineering (Phase 6) ─────────────────────────────────
/// Σ+ — accumulate X,Y into Σ registers R01–R06; push n into X. LiftEffect: Enable.
SigmaPlus,
/// Σ− — remove X,Y from Σ registers; push n into X. LiftEffect: Enable.
SigmaMinus,
/// MEAN — push x̄ to X, ȳ to Y from Σ registers. LiftEffect: Enable.
Mean,
/// SDEV — push σx to X, σy to Y (sample, n-1). LiftEffect: Enable.
Sdev,
/// L.R. — linear regression: push slope m to Y, intercept b to X. LiftEffect: Enable.
LR,
/// YHAT — ŷ prediction: read x from X, push ŷ. LiftEffect: Enable.
Yhat,
/// CORR — correlation coefficient r in X. LiftEffect: Enable.
Corr,
/// CLΣSTAT — zero Σ registers R01–R06. LiftEffect: Neutral.
ClSigmaStat,
/// HMS→ — convert H.MMSS to decimal hours in X. LiftEffect: Enable.
HmsToH,
/// →HMS — convert decimal hours in X to H.MMSS. LiftEffect: Enable.
HToHms,
/// HMS+ — add two H.MMSS values (Y + X), result in X with stack drop. LiftEffect: Enable.
HmsAdd,
/// HMS− — subtract H.MMSS values (Y − X), result in X with stack drop. LiftEffect: Enable.
HmsSub,
```

**Module declaration pattern** (lines 9–16 of `mod.rs`):
```rust
// Existing:
pub mod arithmetic;
pub mod stack_ops;
pub mod math;
pub mod registers;
pub mod alpha;
pub mod program;
// Add:
pub mod stats;
pub mod hms;
```

**dispatch() match arm extension pattern** (lines 192–291):
```rust
// Pattern: add use declarations at top of dispatch body, then add match arms
// Current last arms (lines 287–290):
Op::Isg(reg)   => { program::op_isg(state, reg).map(|_| ()) }
Op::Dse(reg)   => { program::op_dse(state, reg).map(|_| ()) }
// Add after Dse:
Op::SigmaPlus  => stats::op_sigma_plus(state),
Op::SigmaMinus => stats::op_sigma_minus(state),
Op::Mean       => stats::op_mean(state),
Op::Sdev       => stats::op_sdev(state),
Op::LR         => stats::op_lr(state),
Op::Yhat       => stats::op_yhat(state),
Op::Corr       => stats::op_corr(state),
Op::ClSigmaStat => stats::op_cl_sigma_stat(state),
Op::HmsToH     => hms::op_hms_to_h(state),
Op::HToHms     => hms::op_h_to_hms(state),
Op::HmsAdd     => hms::op_hms_add(state),
Op::HmsSub     => hms::op_hms_sub(state),
```

**CRITICAL: execute_op() in `program.rs`** (lines 218–304) must also get all 12 arms:
```rust
// Existing execute_op structure — add identical arms after the UserMode arm (line 296–300):
Op::UserMode => {
    state.user_mode = !state.user_mode;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
// Add:
Op::SigmaPlus  => stats::op_sigma_plus(state),
// ... all 12 arms ...
```
These use declarations must be added at the top of `execute_op` — or the stats/hms modules can be called via full path `crate::ops::stats::op_sigma_plus(state)`.

---

### `hp41-core/src/error.rs` (model — add one variant)

**Analog:** Self — extend existing file at `/hp41-core/src/error.rs`

**Current file** (lines 1–16):
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
    // Phase 3 addition — HP-41 call-depth exceeded (5th subroutine level, D-13/D-14)
    #[error("try again")]
    CallDepth,
}
```

**Add after `CallDepth`:**
```rust
// Phase 6 addition — HMS field-range validation: minutes ≥ 60 or seconds ≥ 60 (D-06)
#[error("invalid input")]
InvalidInput,
```

**WARNING: `hp41-core/src/tests.rs` update required.** Two tests will fail after adding `InvalidInput`:
- `hperror_has_four_variants` (line 9–20): add `HpError::InvalidInput` construction and assert
- `hperror_display_messages` (line 22–28): add `assert_eq!(HpError::InvalidInput.to_string(), "invalid input")`

---

### `hp41-core/tests/stats_tests.rs` (test — new file)

**Analog:** `hp41-core/tests/register_tests.rs` (exact role match — dispatch-level integration tests)

**File header pattern** (`register_tests.rs` lines 1–9):
```rust
//! Integration tests for REGS-01: storage registers R00–R99, STO/RCL, STO-arith.

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op, StoArithKind};
use rust_decimal::Decimal;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
```

**For stats_tests.rs, use:**
```rust
//! Integration tests for SCI-01: statistics operations Σ+, Σ−, MEAN, SDEV, L.R., YHAT, CORR, CLΣSTAT.

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}
```
Note: `push_dec` helper is established in `math_tests.rs` lines 13–16 — copy it here too.

**Test body pattern** (`register_tests.rs` lines 11–55 — direct dispatch calls):
```rust
#[test]
fn test_sigma_plus_accumulates_into_r03_count() {
    let mut s = CalcState::new();
    // X=3, Y=5 → Σ+ → R03 (n) should be 1
    s.stack.y = HpNum::from(5i32);
    push(&mut s, 3);
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::from(1), "n must be 1 after first Σ+");
    assert_eq!(s.stack.x.inner(), Decimal::from(1), "X must hold n after Σ+");
}
```

**Error path pattern** (`register_tests.rs` lines 101–111):
```rust
#[test]
fn test_mean_with_no_data_returns_invalid_op() {
    let mut s = CalcState::new();
    // All Σ registers zero (n=0) → MEAN must error
    assert_eq!(dispatch(&mut s, Op::Mean), Err(HpError::InvalidOp));
}
```

**Lift semantics pattern** (`register_tests.rs` lines 113–149):
```rust
#[test]
fn test_sigma_plus_enables_lift() {
    let mut s = CalcState::new();
    s.stack.lift_enabled = false;
    push(&mut s, 3);
    dispatch(&mut s, Op::SigmaPlus).unwrap();
    assert!(s.stack.lift_enabled, "Σ+ must enable lift after pushing n");
}
```

---

### `hp41-core/tests/hms_tests.rs` (test — new file)

**Analog:** `hp41-core/tests/math_tests.rs` (role match — unary/binary transform dispatch tests)

**File header pattern** (`math_tests.rs` lines 1–16):
```rust
//! Integration tests for MATH-01: unary math ops...

use hp41_core::{CalcState, HpError, HpNum};
use hp41_core::ops::{dispatch, Op};
use rust_decimal::Decimal;
use std::str::FromStr;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}
```

**Canonical round-trip test pattern** (from RESEARCH.md Code Examples):
```rust
#[test]
fn test_hms_to_h_canonical_1_3045() {
    // 1.3045 (1h 30m 45s in H.MMSS) → HMS→ → 1.5125 decimal hours
    // 1 + 30/60 + 45/3600 = 1.5125 exactly
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected = Decimal::from_str("1.5125").unwrap();
    assert_eq!(s.stack.x.inner(), expected);
}
```

**Error path for invalid HMS** (`math_tests.rs` line 33–35 adapted):
```rust
#[test]
fn test_hms_to_h_invalid_minutes_returns_invalid_input() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.6000");  // 60 minutes = invalid
    assert_eq!(dispatch(&mut s, Op::HmsToH), Err(HpError::InvalidInput));
}
```

---

### `hp41-cli/src/keys.rs` (config — extend existing)

**Analog:** Self — append to existing `key_to_op()` match in `/hp41-cli/src/keys.rs`

**Append location:** After the existing `KeyCode::Char('u') => Some(Op::UserMode)` arm (line 53), before `KeyCode::Char('S') | KeyCode::Char('R') => None` (line 58).

**Binding extension pattern** (lines 19–66 of `keys.rs` — existing match arms):
```rust
// Existing pattern — copy exactly:
KeyCode::Char('u')           => Some(Op::UserMode),
// Add Phase 6 stats bindings (lines appended after UserMode):
KeyCode::Char('z')           => Some(Op::SigmaPlus),
KeyCode::Char('Z')           => Some(Op::SigmaMinus),
KeyCode::Char('m')           => Some(Op::Mean),
// SDEV: use 'D' (uppercase) — 'd' is intercepted in app.rs for angle cycle (RESEARCH Pitfall 1)
KeyCode::Char('D')           => Some(Op::Sdev),
KeyCode::Char('y')           => Some(Op::Yhat),
KeyCode::Char('R')           => None,  // already: STO/RCL modal (keep as-is)
// L.R. needs a key; 'R' is taken by RCL modal; use Shift+R? No — use lowercase 'q'-adjacent
// Per CONTEXT.md binding table: L.R. = 'R' (but 'R' is taken by RCL modal — planner must decide)
// CORR = 'O' per CONTEXT.md
KeyCode::Char('O')           => Some(Op::Corr),
// CLΣSTAT = 'V'
KeyCode::Char('V')           => Some(Op::ClSigmaStat),
// HMS ops: 'h' = HMS→, →HMS: use 'F' (uppercase) since 'f' intercepted for fmt cycle (Pitfall 1)
KeyCode::Char('h')           => Some(Op::HmsToH),
KeyCode::Char('F')           => Some(Op::HToHms),
KeyCode::Char('j')           => Some(Op::HmsAdd),
KeyCode::Char('J')           => Some(Op::HmsSub),
```

**KEY_REF_TABLE extension pattern** (lines 71–113 — append entries):
```rust
// Append after existing Phase 5 entries:
("z",      "Σ+  (accumulate X,Y into stats registers)"),
("Z",      "Σ−  (remove X,Y from stats registers)"),
("m",      "MEAN (x̄ in X, ȳ in Y)"),
("D",      "SDEV (σx in X, σy in Y, sample n-1)"),
// ...
```

**CRITICAL conflict note:** Per RESEARCH.md Pitfall 1 (`app.rs` lines 291–310), lowercase `d` and `f` are intercepted before `key_to_op()` is called. Bindings added for `'d'` or `'f'` in `key_to_op()` compile but are dead code. Use `'D'` (Shift+d) for SDEV and `'F'` (Shift+f) for →HMS instead. L.R. binding: CONTEXT.md says `'R'` but `'R'` returns `None` for STO/RCL modal — planner must assign L.R. to an unused key (e.g., `'b'` or adjust modal handling). This is within Claude's Discretion per D-09.

---

### `hp41-cli/src/help_data.rs` (config — extend existing)

**Analog:** Self — append to existing `HELP_DATA` array in `/hp41-cli/src/help_data.rs`

**Category header pattern** (lines 12–13, 20–21 of `help_data.rs`):
```rust
("",        "",          "=== Stack ==="),
("Enter",   "ENTER",     "Lift stack and duplicate X into Y"),
```

**Append location:** Before the `"=== Help ==="` block (line 84). Add a new category:
```rust
// ── Science & Engineering ─────────────────────────────────────────────────────
("",        "",          "=== Science & Engineering ==="),
("z",       "Σ+",        "Accumulate X,Y into Σ registers R01–R06; push count n to X"),
("Z",       "Σ−",        "Remove X,Y from Σ registers; push count n to X"),
("m",       "MEAN",      "Mean: X ← x̄, Y ← ȳ from Σ registers"),
("D",       "SDEV",      "Std dev (sample n-1): X ← σx, Y ← σy from Σ registers"),
("y",       "YHAT",      "ŷ prediction: read x from X, compute ŷ via linear regression"),
// L.R. key TBD — planner resolves conflict with 'R' (RCL modal)
("?",       "L.R.",      "Linear regression: slope m to Y, intercept b to X"),
("O",       "CORR",      "Correlation coefficient r in X"),
("V",       "CLΣSTAT",   "Clear Σ statistics registers R01–R06 to zero"),
("h",       "HMS→",      "Convert H.MMSS (hours-minutes-seconds) to decimal hours"),
("F",       "→HMS",      "Convert decimal hours to H.MMSS format"),
("j",       "HMS+",      "Add two H.MMSS values: Y + X → X (base-60 carry)"),
("J",       "HMS−",      "Subtract H.MMSS values: Y − X → X (base-60 borrow)"),
```

**Test update pattern** (`help_data.rs` lines 96–130 — existing tests):
The `test_all_ten_categories_present` test at line 110 checks for exactly 10 categories by name. Adding `"=== Science & Engineering ==="` does NOT break this test (it checks `any()` for listed names, not `all()` exclusive). The `test_help_data_has_minimum_entries` test asserts `>= 50` — adding 14 entries is safe.

---

### `hp41-cli/src/prgm_display.rs` (config — extend existing)

This file is NOT listed in the phase deliverables but RESEARCH.md Pattern 5 identifies it as a required modification. The `op_display_name()` function (lines 26–96) has no wildcard arm — missing variants cause a compile error.

**Analog:** Self — append match arms to `op_display_name()` in `/hp41-cli/src/prgm_display.rs`

**Append pattern** (after `Op::AlphaBackspace` arm, line 94):
```rust
// Phase 5: new Op variants
Op::UserMode       => "USER".to_string(),
Op::AlphaBackspace => "\u{2190}".to_string(),
// Phase 6: Science & Engineering
Op::SigmaPlus   => "\u{03A3}+".to_string(),
Op::SigmaMinus  => "\u{03A3}-".to_string(),
Op::Mean        => "MEAN".to_string(),
Op::Sdev        => "SDEV".to_string(),
Op::LR          => "L.R.".to_string(),
Op::Yhat        => "\u{0177}".to_string(),
Op::Corr        => "CORR".to_string(),
Op::ClSigmaStat => "CL\u{03A3}".to_string(),
Op::HmsToH      => "HMS\u{2192}".to_string(),
Op::HToHms      => "\u{2192}HMS".to_string(),
Op::HmsAdd      => "HMS+".to_string(),
Op::HmsSub      => "HMS-".to_string(),
```

---

## Shared Patterns

### Register Index Convention
**Source:** `hp41-core/src/state.rs` + `hp41-core/src/ops/registers.rs` lines 14–21
**Apply to:** `stats.rs`, `hms.rs` (CLΣSTAT)
```rust
// Σ registers (0-indexed Vec<HpNum> with 100 slots):
state.regs[1]  // R01 = Σx²
state.regs[2]  // R02 = Σx
state.regs[3]  // R03 = n (count)
state.regs[4]  // R04 = Σy²
state.regs[5]  // R05 = Σy
state.regs[6]  // R06 = Σxy
// Direct index access is always safe — Vec is always 100 elements (CalcState::new)
```

### LiftEffect Application
**Source:** `hp41-core/src/stack.rs` lines 16–24 + `hp41-core/src/ops/registers.rs` lines 19, 34
**Apply to:** All ops in `stats.rs` and `hms.rs`
```rust
// End every op implementation with an explicit apply_lift_effect call:
apply_lift_effect(state, LiftEffect::Enable);   // result ops (MEAN, SDEV, HMS→, etc.)
apply_lift_effect(state, LiftEffect::Neutral);  // state-changing no-result ops (CLΣSTAT)
// Σ+/Σ− use Enable (they push n to X)
```

### Error Propagation with `?`
**Source:** `hp41-core/src/ops/arithmetic.rs` lines 7–10
**Apply to:** All ops in `stats.rs` and `hms.rs`
```rust
// Use ? for all HpNum checked_* calls — never unwrap() in hp41-core
let result = state.stack.y.checked_add(&state.stack.x)?;
// The ? propagates HpError up to dispatch(), which propagates to the TUI
```

### Decimal String Construction
**Source:** `hp41-core/src/ops/program.rs` lines 364–368 (`build_counter`)
**Apply to:** `hms.rs` `op_h_to_hms`
```rust
fn build_counter(current: i64, frac_padded: &str) -> Result<HpNum, HpError> {
    let s = format!("{}.{}", current, frac_padded);
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::rounded(d))
}
```

### Test Helper Functions
**Source:** `hp41-core/tests/register_tests.rs` lines 7–9, `hp41-core/tests/math_tests.rs` lines 9–16
**Apply to:** `stats_tests.rs`, `hms_tests.rs`
```rust
fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
fn push_dec(state: &mut CalcState, s: &str) {
    let d = Decimal::from_str(s).expect("valid decimal literal in test");
    dispatch(state, Op::PushNum(HpNum::from(d))).unwrap();
}
```

---

## No Analog Found

All files have close analogs. No entries in this section.

---

## Anti-Patterns (Extracted from Codebase + RESEARCH.md)

| Trap | Wrong Pattern | Correct Pattern | Source |
|---|---|---|---|
| Σ+ stack semantics | `binary_result(state, n)` — drops Y, saves LASTX | Direct `enter_number` + `apply_lift_effect` as in `op_rcl` | `stack.rs` lines 63–74 |
| HMS field extraction | `floor(x)` / `fmod()` on f64 | `n.inner().to_string()` → `s.find('.')` → slice | `program.rs` `parse_counter` lines 344–360 |
| HMS padding direction | `format!("{:0>4}", frac)` — left-pads (wrong) | `format!("{:0<4}", frac)` — right-pads (MMSS left-to-right) | `program.rs` line 353 comment |
| Negative HMS | Parse `"-1.3045"` integer as -1 then minutes from raw frac | Extract `is_sign_negative()` first, work on `abs()` | RESEARCH.md Pitfall 8 |
| L.R. output order | Push slope m last (ends in X) | Push m first (Y), b last (X) — L.R. convention | CONTEXT.md D-05 |
| SDEV denominator | Divide by n (population) | Divide by n-1 (sample) | CONTEXT.md Specific Requirements |
| key 'd'/'f' bindings | `KeyCode::Char('d') => Some(Op::Sdev)` — dead code | `KeyCode::Char('D') => Some(Op::Sdev)` | `app.rs` lines 291–310 (intercepted) |
| Missing execute_op arms | New ops work interactively but fail inside programs | Add arms to BOTH `dispatch()` in `mod.rs` AND `execute_op()` in `program.rs` | `program.rs` lines 218–304 |
| Missing prgm_display arms | Compile error (no wildcard) | Add arm for every new Op variant | `prgm_display.rs` lines 26–96 |

---

## Metadata

**Analog search scope:** `hp41-core/src/ops/`, `hp41-core/src/`, `hp41-core/tests/`, `hp41-cli/src/`
**Files scanned:** 14 source files + 12 test files
**Pattern extraction date:** 2026-05-07
