---
phase: 02-core-math
reviewed: 2026-05-06T00:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - hp41-core/Cargo.toml
  - hp41-core/src/format.rs
  - hp41-core/src/lib.rs
  - hp41-core/src/num.rs
  - hp41-core/src/ops/alpha.rs
  - hp41-core/src/ops/math.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/registers.rs
  - hp41-core/src/stack.rs
  - hp41-core/src/state.rs
  - hp41-core/src/tests.rs
  - hp41-core/tests/alpha_tests.rs
  - hp41-core/tests/entry_buf_tests.rs
  - hp41-core/tests/format_tests.rs
  - hp41-core/tests/lift_tests.rs
  - hp41-core/tests/math_tests.rs
  - hp41-core/tests/phase2_scaffold_tests.rs
  - hp41-core/tests/register_tests.rs
  - hp41-core/tests/trig_tests.rs
findings:
  critical: 3
  warning: 7
  info: 3
  total: 13
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-05-06
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

Reviewed all Phase 2 source files for the HP-41 Core Math implementation. The overall
architecture is sound: the `HpNum` newtype, the dispatch table, `apply_lift_effect`, and
`unary_result`/`binary_result` helpers are all correctly structured. Stack-lift semantics
are correctly declared for every operation. No panics, no unchecked array accesses, no SQL
or command injection.

Three correctness bugs were found, all in `format.rs`. Two affect numerical display
correctness for valid HP-41 register values (SCI/ENG mantissa carry-over and FIX display
digit clamping). One affects error-type fidelity in `num.rs`. Seven warnings cover
`unwrap()` in production code, wrong error classification, dead code with panic paths, and
test coverage gaps. Three info items cover minor code quality issues.

---

## Critical Issues

### CR-01: SCI/ENG Formatter Produces Invalid Mantissa When Rounding Carries to 10

**File:** `hp41-core/src/format.rs:88-100` (SCI) and `127-133` (ENG)

**Issue:** `format_sci` and `format_eng` round the mantissa with `round_dp_with_strategy`
_after_ computing the scientific exponent. For a valid `HpNum` value such as
`9.999999999` (a legal 10-significant-digit value), the mantissa in SCI(4) mode rounds
from `9.999999999` to `10.0000`. The code then formats this as `"10.0000E 00"` instead of
the correct `"1.0000E 01"`. The same flaw exists in `format_eng`.

Concrete example:
- Input: `HpNum` storing `9.999999999`, mode `Sci(4)`
- `compute_sci_exp(9.999999999)` → `floor(log10(9.999...)) = 0`
- Mantissa = `9.999999999`, rounded to 4 dp → `10.0000`
- Output: `"10.0000E 00"` — **wrong**
- Expected: `"1.0000E 01"`

**Fix:** After rounding the mantissa, check whether it has reached `>= 10`. If so,
increment `sci_exp` (or `eng_exp`) by 1, divide the mantissa by 10, and re-format:

```rust
// In format_sci, after computing mantissa_rounded:
let (mantissa_rounded, sci_exp) = if mantissa_rounded >= Decimal::from(10) {
    (mantissa_rounded / Decimal::from(10), sci_exp + 1)
} else {
    (mantissa_rounded, sci_exp)
};
```

The same normalization is needed in `format_eng`, followed by re-clamping `eng_exp` to
the nearest multiple of 3 if the increment changes its alignment.

---

### CR-02: `checked_powd` Returns `HpError::Domain` for Overflow

**File:** `hp41-core/src/num.rs:128-131`

**Issue:** When `rust_decimal::checked_powd` returns `None` (which happens both for
overflow and for `0^negative`), the function maps it to `HpError::Domain`. The domain
check for complex results (`base < 0` with fractional exponent) is already handled
explicitly at line 125. If `checked_powd` returns `None` after that guard, the actual
cause is overflow or `0^negative`, not a domain error. Callers that match on error type
to decide whether to display "INVALID DATA" vs. display an overflow indicator will receive
wrong information.

```rust
// Current (wrong):
self.0.checked_powd(exp.0)
    .map(HpNum::rounded)
    .ok_or(HpError::Domain)  // line 130

// Fix:
self.0.checked_powd(exp.0)
    .map(HpNum::rounded)
    .ok_or(HpError::Overflow)
```

---

### CR-03: `FmtFix`/`FmtSci`/`FmtEng` Digit Count Is Not Validated (0–9 Not Enforced)

**File:** `hp41-core/src/ops/mod.rs:182-195`

**Issue:** `Op::FmtFix(u8)`, `Op::FmtSci(u8)`, and `Op::FmtEng(u8)` accept any `u8`
value (0–255). The HP-41 hardware only supports digit counts 0–9. Values outside this
range produce incorrect display output:

- `FmtFix(11)`: `overflow_exp = 10_usize.saturating_sub(11) = 0`, so `overflow_threshold = 1`.
  Any number `>= 1` unconditionally falls back to SCI—e.g., `format_fix(42, 11)` returns
  `"4.200000000E 01"` instead of an error or a clamped value.
- `FmtFix(200)`: `format!("{:.200}", ...)` produces a 200-digit mantissa—far exceeding
  HP-41's 12-character display—and is a severe correctness violation.

Neither `dispatch` nor the `Op` enum constructors reject out-of-range values.

**Fix:** Validate in `dispatch` and return `HpError::InvalidOp` for out-of-range digits:

```rust
Op::FmtFix(n) => {
    if n > 9 { return Err(HpError::InvalidOp); }
    state.display_mode = DisplayMode::Fix(n);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
// Same guard for FmtSci and FmtEng
```

---

## Warnings

### WR-01: `unwrap()` in Production Code in `math.rs` (Violates Zero-Panic Invariant)

**File:** `hp41-core/src/ops/math.rs:25` and `math.rs:28`

**Issue:** `pi_over_180()` and `pi_over_200()` call `.unwrap()` on a `Decimal::from_str`
parse of a hardcoded literal:

```rust
fn pi_over_180() -> HpNum {
    HpNum(Decimal::from_str("0.01745329251994329576").unwrap())  // line 25
}
```

These functions reside in a production source file. Even though they are currently dead
code (called only from the `#[allow(dead_code)]` function `to_radians_hpnum`), they are
compiled into the binary and can be invoked by future callers. The project invariant is
"zero panics in `hp41-core`". An `unwrap()` on an infallible parse should use a `const`
or `lazy_static`, or be rewritten as:

```rust
fn pi_over_180() -> HpNum {
    // "0.01745329251994329576" is a valid Decimal literal; from_str cannot fail here.
    HpNum(Decimal::from_str("0.01745329251994329576")
        .unwrap_or(Decimal::ONE))  // at minimum; better: use a const
}
```

The cleanest fix is to compute these as compile-time constants or remove the entire dead
code block until it is needed.

---

### WR-02: `checked_sin`/`checked_cos`/`checked_tan` Map `None` to `HpError::Domain`

**File:** `hp41-core/src/num.rs:139`, `146`, `153`

**Issue:** `rust_decimal::MathematicalOps::checked_sin/cos/tan` return `None` when the
input exceeds their internal series computation range (large radians value), which is an
overflow condition, not a domain error. All three functions use `.ok_or(HpError::Domain)`.
A caller that inspects the error to decide what message to show the user (e.g., "INVALID
DATA" for domain vs. overflow indicator) will display the wrong message for a very large
angular input.

**Fix:** Change to `.ok_or(HpError::Overflow)` on lines 141, 148, and 155.

---

### WR-03: `compute_sci_exp` Uses `to_f64().unwrap_or(1.0)` — Silent Wrong Exponent

**File:** `hp41-core/src/format.rs:147`

**Issue:**

```rust
fn compute_sci_exp(abs_d: Decimal) -> i32 {
    let f = abs_d.to_f64().unwrap_or(1.0);
    f.log10().floor() as i32
}
```

If `abs_d.to_f64()` returns `None` (possible for Decimal values whose magnitude exceeds
`f64::MAX`, approximately `1.8e308`), the function silently falls back to `1.0`, returning
`sci_exp = 0`. For a very large Decimal value this produces a completely wrong exponent
(e.g., the display would show `1.2345E 00` instead of `1.2345E 28`). While the HP-41
hardware cannot hold numbers above `~9.99999999e99`, the library places no such cap on
`HpNum`, so a large Decimal value could reach `format_sci`.

**Fix:** Propagate the error rather than silently defaulting:

```rust
fn compute_sci_exp(abs_d: Decimal) -> Option<i32> {
    let f = abs_d.to_f64()?;
    Some(f.log10().floor() as i32)
}
```

Then have `format_sci` and `format_eng` return `String` errors or fall back to an error
display string when `compute_sci_exp` returns `None`.

---

### WR-04: `scale_decimal` Silent Wrong Mantissa on `checked_mul` Overflow

**File:** `hp41-core/src/format.rs:175`

**Issue:**

```rust
fn scale_decimal(d: Decimal, exp_shift: i32) -> Decimal {
    if exp_shift == 0 { return d; }
    let scale = decimal_pow10(exp_shift);
    d.checked_mul(scale).unwrap_or(d)  // line 175
}
```

When `d * 10^exp_shift` overflows `Decimal`, the function returns `d` unchanged. In
`format_sci`/`format_eng` this means the mantissa printed would be the original
un-scaled value (e.g., `123456789.0` instead of `1.23456789`), producing garbage output
with no indication of failure.

Related: `decimal_pow10` for large `exp > 28` builds a string that overflows Decimal and
falls back to `Decimal::ONE` (line 165). Both cases cause silent wrong output.

**Fix:** Return `Option<Decimal>` from `scale_decimal` and `decimal_pow10`, propagate
through `format_sci`/`format_eng`, and render an error placeholder string if scaling
fails.

---

### WR-05: `op_rcl` and `op_lastx` Directly Mutate `lift_enabled` Bypassing `apply_lift_effect`

**File:** `hp41-core/src/ops/registers.rs:32` and `hp41-core/src/ops/stack_ops.rs:79`

**Issue:** Both functions force `state.stack.lift_enabled = true` directly before calling
`enter_number`, then also call `apply_lift_effect(state, LiftEffect::Enable)` afterward.
The direct mutation bypasses the single-authority pattern that `apply_lift_effect` is
meant to provide, making it harder to audit lift semantics. If the semantics of "force
lift before enter" ever need to change (e.g., adding side effects to lift state changes),
the two code paths will diverge silently.

**Fix:** Either document clearly why the direct mutation is necessary (it IS necessary:
`enter_number` reads `lift_enabled`, so the state must be set before the call), or
introduce a helper such as `force_lift_then_enter(state, value)` that encapsulates the
pattern:

```rust
/// Push a value that unconditionally lifts before placing it (RCL, LASTX semantics).
pub fn lift_and_enter(state: &mut CalcState, value: HpNum) {
    state.stack.lift_enabled = true;
    enter_number(state, value);
    // lift_enabled remains true (Enable effect)
}
```

This eliminates the redundant `apply_lift_effect` call and makes the pattern self-documenting.

---

### WR-06: Trig Lift Test in `trig_tests.rs` Is Vacuous — Will Never Fail

**File:** `hp41-core/tests/trig_tests.rs:178-183`

**Issue:** The test `test_trig_ops_enable_lift` is self-defeating:

```rust
let _ = dispatch(&mut s, op.clone());
// Only check lift if op succeeded
if s.stack.lift_enabled {
    assert!(s.stack.lift_enabled, "{op:?} must enable lift on success");
}
```

The condition `if s.stack.lift_enabled` and the assertion `assert!(s.stack.lift_enabled)`
check the same variable. If an op fails to enable lift (the bug being tested), the
condition is false and the assertion is never reached. The test will always pass regardless
of the actual behavior of trig ops.

**Fix:**

```rust
let result = dispatch(&mut s, op.clone());
if result.is_ok() {
    assert!(s.stack.lift_enabled, "{op:?} must enable lift on success");
}
```

---

### WR-07: `Op::Sqrt`, `Op::YPow`, and All Six Trig Ops Missing from `lift_tests.rs`

**File:** `hp41-core/tests/lift_tests.rs`

**Issue:** The structured lift-effect test file does not include `Op::Sqrt`, `Op::YPow`,
`Op::Sin`, `Op::Cos`, `Op::Tan`, `Op::Asin`, `Op::Acos`, or `Op::Atan`. These nine
operations are all specified as `LiftEffect::Enable` but are completely absent from
`lift_tests.rs`, which is the canonical location for lift-semantics regression tests.
`math_tests.rs` covers `Sqrt` and some arithmetic ops but not the trig ops.

**Fix:** Add explicit lift-enable tests for each missing op to `lift_tests.rs`, following
the existing pattern:

```rust
#[test]
fn test_sqrt_enables_lift() {
    let mut s = make_state_with_values();  // x = 2
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::Sqrt).unwrap();
    assert!(s.stack.lift_enabled, "Sqrt must enable lift");
}

#[test]
fn test_ypow_enables_lift() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(2);
    s.stack.y = HpNum::from(3);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::YPow).unwrap();
    assert!(s.stack.lift_enabled, "YPow must enable lift");
}
// ... similarly for Sin, Cos, Tan, Asin, Acos, Atan
```

---

## Info

### IN-01: Dead Code `to_radians_hpnum`, `pi_over_180`, `pi_over_200` Left in Production File

**File:** `hp41-core/src/ops/math.rs:24-40`

**Issue:** `pi_over_180()`, `pi_over_200()`, and `to_radians_hpnum()` are dead code
(suppressed via `#[allow(dead_code)]` only on `to_radians_hpnum`). The two helper
functions will generate compiler warnings. Dead code in a production file adds cognitive
noise and contains the `unwrap()` calls flagged in WR-01.

**Fix:** Remove the three dead functions until the `HpNum`-path trig conversion is
actually needed, or at minimum apply `#[allow(dead_code)]` to all three.

---

### IN-02: `format_alpha` Truncates to 12 Characters but HP-41 ALPHA Register Holds 24

**File:** `hp41-core/src/format.rs:29-31`

**Issue:** `format_alpha` truncates the ALPHA register to 12 characters for display
(HP-41 screen width). This is correct for display rendering, but the function name and
docstring do not make clear that this is a _display_ truncation only—the underlying
`alpha_reg` is preserved up to 24 characters. If callers use `format_alpha` as a
persistence or comparison path, they would silently lose the second half of a 24-character
message.

**Fix:** Rename to `format_alpha_display` or add a prominent doc comment:

```rust
/// Format the ALPHA register for **display only** (first 12 chars).
/// The full alpha_reg string (up to 24 chars) is preserved in CalcState.
pub fn format_alpha_display(reg: &str) -> String {
    reg.chars().take(12).collect()
}
```

---

### IN-03: Missing Test Coverage for SCI/ENG Zero Display Format

**File:** `hp41-core/tests/format_tests.rs`

**Issue:** `format_tests.rs` tests `test_sci4_zero` which checks `SCI(4)` for zero.
There is no test for `ENG(4)` zero, `SCI(0)` zero, or `ENG(0)` zero. There is also no
test for negative numbers in SCI or ENG mode, and no test for the SCI/ENG mantissa
rounding carry case documented in CR-01 (e.g., `9.9995` in `Sci(3)` should display
`"1.000E 01"` not `"10.000E 00"`). Adding tests for these cases would lock in the correct
behavior once CR-01 is fixed.

**Fix:** Add at minimum:
- `test_sci3_mantissa_carry`: value `9.9995` in `Sci(3)` → `"1.000E 01"`
- `test_sci4_negative`: value `-2.5` in `Sci(4)` → `"-2.5000E 00"`
- `test_eng4_zero`: value `0` in `Eng(4)` → `"0.0000E 00"`

---

_Reviewed: 2026-05-06_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
