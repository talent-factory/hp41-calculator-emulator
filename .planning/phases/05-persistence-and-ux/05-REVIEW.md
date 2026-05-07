---
phase: 05-persistence-and-ux
reviewed: 2026-05-07T10:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - hp41-cli/src/app.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-cli/src/programs.rs
  - hp41-core/src/num.rs
  - hp41-core/src/ops/math.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
findings:
  critical: 1
  warning: 6
  info: 3
  total: 10
status: issues_found
---

# Phase 05: Code Review Report (Re-review after plan 05-11 gap closure)

**Reviewed:** 2026-05-07 (re-review of hp41-cli/src/programs.rs)
**Depth:** standard
**Files Reviewed:** 7 (full scope); re-review focused on programs.rs
**Status:** issues_found

## Summary

This is the second-pass review of Phase 5 (Persistence & UX), focused on `hp41-cli/src/programs.rs` after plan 05-11 gap closure. Two previously identified BLOCKER bugs were addressed:

**CR-02 (gcd_ops) — FIXED.** `Op::Int` is now present at line 248 immediately after `Op::Div`, truncating the quotient to an integer before multiply-back. The Euclidean modulo step `a - b * trunc(a/b)` is now arithmetically exact. Manually verified for gcd(12,8)=4, gcd(7,3)=1, gcd(15,5)=5. The new `test_gcd_correctness` test at line 504 confirms these cases.

**CR-03 (stack_stats_ops) — FIXED.** The register-save pattern (R00-R03 via 3×Rdn cycling) correctly captures the original X, Y, Z, T values. The max-finding section now uses `Test(XLtY)` + XySwap (fires when X is smaller, swapping larger Y into X) and the min-finding section now uses `Test(XGtY)` + XySwap (fires when X is larger, swapping smaller Y into X). Both sections were manually traced with the test inputs [3,1,4,5] and with a descending sequence [5,4,3,2] — all produce correct min=1/5, max=5/5 results. The new `test_stack_stats_correctness` test at line 529 validates the documented [3,1,4,5] inputs.

**One BLOCKER remains open (CR-01):** `prime_test_ops` classifies n=0 and n=1 as prime due to the `XLeY` early-exit test at line 160. This was not in scope for plan 05-11.

Four pre-existing warnings remain open. One new warning is added for the shallow depth of the new `test_gcd_correctness` test suite. Warnings from other files in the full review (WR-01 in app.rs, WR-04 in math.rs) are preserved in their original sections below.

---

## Critical Issues

### CR-01: `prime_test_ops` classifies n=0 and n=1 as prime [OPEN — not in plan 05-11 scope]

**File:** `hp41-cli/src/programs.rs:160`
**Issue:** The early-exit test `Test(TestKind::XLeY)` compares X=n against Y=2. When n <= 2 the condition is TRUE and execution jumps to label "P" (result = 1, prime). This correctly handles n=2 but also classifies n=0 and n=1 as prime, which is mathematically wrong (0 and 1 are not prime by definition). The test suite at line 467-473 covers n={2,3,4,9,13} but omits n=0 and n=1, so this bug remains undetected.

**Fix:** Replace the single `XLeY` early-exit with two separate conditionals:
```rust
// Before the divisor loop setup:
Op::PushNum(HpNum::from(2i32)),
Op::RclReg(0),                        // X=n, Y=2
Op::Test(TestKind::XLtY),            // n < 2 → TRUE → not prime
Op::Gto("N".to_string()),
Op::Test(TestKind::XEqY),            // n == 2 → TRUE → prime
Op::Gto("P".to_string()),
// fall through: n > 2, start trial division at d=3 (or keep d=2 for even check)
```
Extend the test to include:
```rust
assert_eq!(run_prime(0), HpNum::from(0i32), "prime(0) must be 0");
assert_eq!(run_prime(1), HpNum::from(0i32), "prime(1) must be 0 (not prime by definition)");
```

---

## Warnings

### WR-01: `'S'`, `'R'`, and `Ctrl+A` modal triggers not guarded against active overlays [OPEN — app.rs, not in plan 05-11 scope]

**File:** `hp41-cli/src/app.rs:172-187`
**Issue:** The STO modal (`'S'`), RCL modal (`'R'`), and USER-assign modal (`Ctrl+A`) are activated before the overlay dispatch block. When the help overlay is open and the user presses `'S'`, `PendingInput::StoRegister` is set without closing the overlay. On the next keypress, `handle_pending_input` runs while `show_help=true` — the UI renders the help overlay but keystrokes silently feed the invisible STO modal.

**Fix:** Add overlay guards to each modal trigger:
```rust
if key.code == KeyCode::Char('S')
    && !key.modifiers.contains(KeyModifiers::CONTROL)
    && !self.show_help
    && !self.show_programs
{
    self.pending_input = Some(PendingInput::StoRegister(String::new()));
}
```
Apply the same `!self.show_help && !self.show_programs` guard to the `'R'` and `Ctrl+A` checks.

---

### WR-02: `test_program_names_unique` uses `dedup()` — only detects consecutive duplicates [OPEN]

**File:** `hp41-cli/src/programs.rs:444-448`
**Issue:** `Vec::dedup()` removes only adjacent duplicate elements. A program list with names `["A", "B", "A"]` has the same length before and after `dedup()` and the test passes despite a duplicate. As the program list grows beyond 10 entries, a non-adjacent duplicate would go undetected.

**Fix:**
```rust
use std::collections::HashSet;
let names: Vec<&str> = sample_programs().iter().map(|p| p.name).collect();
let unique: HashSet<&str> = names.iter().copied().collect();
assert_eq!(names.len(), unique.len(), "Program names must be unique");
```

---

### WR-03: `prime_test_ops` has no test coverage for n=0 and n=1 [OPEN]

**File:** `hp41-cli/src/programs.rs:467-473`
**Issue:** `test_prime_test_correctness` covers n={2,3,4,9,13} but omits n=0 and n=1 — the exact inputs that expose the still-open CR-01. The gap-closure comment declares the XySwap bug "fixed" without evidence that edge cases around the boundary condition were verified.

**Fix:** Add to `test_prime_test_correctness`:
```rust
assert_eq!(run_prime(0), HpNum::from(0i32), "prime(0) must be 0");
assert_eq!(run_prime(1), HpNum::from(0i32), "prime(1) must be 0 (not prime by definition)");
```

---

### WR-04: `op_int` has no unit tests in `hp41-core` [OPEN — math.rs, not in plan 05-11 scope]

**File:** `hp41-core/src/ops/math.rs:81-85`
**Issue:** `op_int` was added as the enabler for the prime_test and gcd gap-closure fixes but has no unit tests. The function's correctness for negative inputs, its `LASTX` save behavior (via `unary_result`), and its lift effect are all untested. A regression in `Decimal::trunc()` behavior across a `rust_decimal` version upgrade would go undetected.

**Fix:** Add tests to the `#[cfg(test)]` module in `hp41-core/src/ops/math.rs`:
```rust
#[test]
fn test_op_int_truncates_positive() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::PushNum(HpNum::from_str("2.9").unwrap())).unwrap();
    dispatch(&mut state, Op::Int).unwrap();
    assert_eq!(state.stack.x, HpNum::from(2i32));
}

#[test]
fn test_op_int_truncates_negative_toward_zero() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::PushNum(HpNum::from_str("-2.7").unwrap())).unwrap();
    dispatch(&mut state, Op::Int).unwrap();
    assert_eq!(state.stack.x, HpNum::from(-2i32)); // NOT -3
}
```

---

### WR-05: `test_fibonacci_runs_without_panic` does not assert the computed value [OPEN]

**File:** `hp41-cli/src/programs.rs:431-441`
**Issue:** The test only calls `assert!(result.is_ok(), ...)` — it never checks the stack after the run. The comment "F(6)=8 or similar" signals uncertainty rather than specification. A behavioral regression (wrong result, no error) would pass this test.

**Fix:**
```rust
let result = hp41_core::run_program(&mut state, "A");
assert!(result.is_ok(), "Fibonacci must run without error: {:?}", result);
// F(6) = 8  (sequence: 0,1,1,2,3,5,8,...)
assert_eq!(state.stack.x, HpNum::from(8i32), "Fibonacci(6) must equal 8");
```

---

### WR-06: `test_gcd_correctness` test cases require at most 2 Euclidean iterations [NEW]

**File:** `hp41-cli/src/programs.rs:523-525`
**Issue:** All three test cases in `test_gcd_correctness` terminate in two or fewer modulo steps:
- `gcd(12, 8)`: r=4, then r=0. Two iterations.
- `gcd(7, 3)`: r=1, then r=0. Two iterations.
- `gcd(15, 5)`: r=0. One iteration.

None of these cases exercises the multi-iteration path of the Euclidean loop, which is the most structurally complex part of `gcd_ops`. A subtle bug in the loop's register-update logic (`RclReg(1), StoReg(0)` / `RclReg(2), StoReg(1)`) or in the `Gto("L")` dispatch would not be caught by the current test suite.

**Fix:** Add at least one multi-step case. `gcd(21, 13)` requires six iterations (gcd=1):
```rust
assert_eq!(run_gcd(21, 13), HpNum::from(1i32), "gcd(21,13) must be 1 (6 Euclidean steps)");
```

---

## Info

### IN-01: `Quadratic Solver` description conflicts with code behavior [OPEN]

**File:** `hp41-cli/src/programs.rs:45`
**Issue:** `SampleProgram.description` reads `"Stack: a(T) b(Z) c(Y) → roots in X,Y."` The code at lines 203-207 stores `StoReg(2)` = X (c), `Rdn → StoReg(1)` = Y (b), `Rdn → StoReg(0)` = Z (a). The correct stack entry order is c(X) b(Y) a(Z), not a(T) b(Z) c(Y). The description shown to users is incorrect.

**Fix:**
```rust
description: "Roots of ax²+bx+c. Stack: c(X) b(Y) a(Z) → root1 in X, root2 in Y.",
```

---

### IN-02: `prime_test_ops` comment uses "floor" where the operation is "trunc" [OPEN]

**File:** `hp41-cli/src/programs.rs:174`
**Issue:** The comment reads `"X=floor(n/d) (truncate toward zero)"`. `floor` (toward negative infinity) and `trunc` (toward zero) differ for negative inputs. For positive prime testing inputs they produce the same result, so no runtime error, but the comment conflates two distinct mathematical operations.

**Fix:**
```rust
Op::Int,  // X = trunc(n/d) — integer part toward zero (equals floor for positive n, d)
```

---

### IN-03: `handle_reg_modal` dead `if reg < 100` guard [OPEN — app.rs, not in plan 05-11 scope]

**File:** `hp41-cli/src/app.rs:451`
**Issue:** `new_acc` is always a 2-character string of ASCII decimal digits, maximum value 99. The guard `if reg < 100` and its `else` error branch are unreachable. The `unwrap_or(0)` fallback on the same line is also dead.

**Fix:** Remove the dead guard:
```rust
let reg: u8 = new_acc.parse().expect("two ASCII digits always parse as u8 ≤ 99");
self.call_dispatch(op_fn(reg));
self.pending_input = None;
```

---

## Gap Closure Verification (plan 05-11)

| Finding | Status | Evidence |
|---------|--------|---------|
| CR-02: gcd missing Op::Int | **CLOSED** | `Op::Int` at line 248; `test_gcd_correctness` at line 504; manual trace confirms correct modulo for gcd(12,8), gcd(7,3), gcd(15,5) |
| CR-03: stack_stats inverted tests | **CLOSED** | XLtY for max-finding, XGtY for min-finding at lines 340-362; manual traces for [3,1,4,5] and [5,4,3,2] produce correct X=min, Y=max; `test_stack_stats_correctness` at line 529 validates documented inputs |

---

_Reviewed: 2026-05-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
