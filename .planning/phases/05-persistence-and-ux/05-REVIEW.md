---
phase: 05-persistence-and-ux
reviewed: 2026-05-07T00:00:00Z
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
  critical: 3
  warning: 5
  info: 3
  total: 11
status: issues_found
---

# Phase 05: Code Review Report (Post-Gap-Closure)

**Reviewed:** 2026-05-07
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

This is a post-gap-closure review for Phase 5 (Persistence & UX). The gap-closure work addressed two previously identified gaps: (1) prime_test_ops XySwap bug and mean_sdev_ops replacement, and (2) the help-overlay 'q' routing fix. Additionally, Op::Int was added to hp41-core for exact integer truncation.

The 'q' quit guard (gap SC-3) is correctly implemented: the guard at `app.rs:130` checks all four blocking conditions (`show_help`, `show_programs`, `alpha_mode`, `pending_input`) before allowing quit. The overlay navigation blocks correctly consume all keys when active, and the three new unit tests for this behavior are valid.

Op::Int is correctly wired in `dispatch()`, `execute_op()`, and `prgm_display`. `trunc_int()` correctly uses `Decimal::trunc()` (truncates toward zero, matching HP-41 INT semantics for positive operands).

The `prime_test_ops` XySwap removal is verified correct for the tested inputs n={2,3,4,9,13}. The `mean_sdev_ops` replacement produces the correct mean of 2.5 for the test inputs.

However, three BLOCKER bugs exist in sample programs that were not caught by the gap-closure test suite: prime_test_ops misclassifies n=0 and n=1 as prime, gcd_ops produces wrong results for most non-trivial integer inputs (missing `Op::Int` in the floor-division step), and stack_stats_ops has an inverted Test+XySwap pattern that always stores the wrong candidate for both max and min.

---

## Critical Issues

### CR-01: `prime_test_ops` classifies n=0 and n=1 as prime

**File:** `hp41-cli/src/programs.rs:160`
**Issue:** The early-exit test `Test(TestKind::XLeY)` compares `X=n` against `Y=2`. When `n <= 2` the condition is TRUE and execution jumps to label "P" (result = 1, prime). This correctly handles n=2 but also classifies n=0 and n=1 as prime, which is mathematically wrong (1 is not prime by definition; 0 is not prime). The test suite `test_prime_test_correctness` tests n={2,3,4,9,13} but omits n=0 and n=1, so the bug is undetected.

**Fix:** Replace the single `XLeY` early-exit with two separate conditionals — one to reject n < 2 as non-prime, one to accept n == 2 as prime:
```rust
// Before the divisor loop setup:
Op::PushNum(HpNum::from(2i32)),
Op::RclReg(0),                        // X=n, Y=2
Op::Test(TestKind::XLtY),            // n < 2 → TRUE → not prime
Op::Gto("N".to_string()),
Op::Test(TestKind::XEqY),            // n == 2 → TRUE → prime
Op::Gto("P".to_string()),
// fall through: n > 2, start trial division
```
Extend the test:
```rust
assert_eq!(run_prime(0), HpNum::from(0i32), "prime(0) must be 0");
assert_eq!(run_prime(1), HpNum::from(0i32), "prime(1) must be 0 (not prime by definition)");
```

---

### CR-02: `gcd_ops` missing `Op::Int` causes wrong GCD for most integer inputs

**File:** `hp41-cli/src/programs.rs:246-249`
**Issue:** The Euclidean algorithm modulo step computes `r = a - b*(a/b)`. The code performs `RclReg(0), RclReg(1), Div` then immediately `RclReg(1), Mul` without applying `Op::Int` to truncate `a/b` to an integer. With `rust_decimal`, `7 / 3` evaluates to `2.333333333` (rounded to 10 sig digits via `HpNum::rounded`). Multiplying back: `2.333333333 * 3 = 6.999999999`. The remainder `7 - 6.999999999 = 0.000000001` is not zero, so the GCD loop never terminates correctly — it computes a fractional remainder rather than the true modulo. For most non-trivially divisible integer pairs, the loop will exhaust `MAX_STEPS` and return `HpError::Overflow`.

The comment at line 248 acknowledges the imprecision: "approximate floor via truncation: use as-is (integer inputs assumed)." Integer inputs do not help — `Decimal::checked_div` does not produce integer results for non-evenly-divisible operands. The gap-closure correctly added `Op::Int` to `prime_test_ops` but `gcd_ops` was not updated.

**Fix:** Insert `Op::Int` after `Op::Div` in `gcd_ops`, mirroring the prime_test fix:
```rust
Op::RclReg(0), Op::RclReg(1), Op::Div,
Op::Int,                                 // trunc(a/b) — exact integer step
Op::RclReg(1), Op::Mul,
Op::RclReg(0), Op::XySwap, Op::Sub,     // r = a - b*trunc(a/b)
```
Also add a behavioral test (e.g., `gcd(12, 8) = 4`, `gcd(7, 3) = 1`).

---

### CR-03: `stack_stats_ops` max/min comparison logic is inverted

**File:** `hp41-cli/src/programs.rs:328-343`
**Issue:** The max-finding section uses `Test(TestKind::XGtY)` immediately followed by `Op::XySwap`. `run_loop` applies skip-if-false semantics: `evaluate_test` returns true → execute next op; returns false → skip next op. When `XGtY` is TRUE (X is the larger value), `XySwap` executes and places the smaller value into X. `StoReg(5)` then stores the smaller value as the "max candidate." When `XGtY` is FALSE (X is not larger), `XySwap` is skipped and X (the smaller or equal value) is stored. In both cases the smaller of the two values is stored as the running maximum.

The same inverted logic applies to the min-finding section using `Test(TestKind::XLtY)` — the larger value is stored as the min candidate.

There is no behavioral test for `stack_stats_ops`, so this has been silently broken. The comment "ensure larger in X" describes the correct intent but the code implements the opposite.

Concrete trace with stack T=4, Z=3, Y=2, X=5:
- After `Enter`: T=3, Z=2, Y=5, X=5
- After `Rdn`: T=5, Z=3, Y=2, X=5
- `Test(XGtY)`: 5>2 TRUE → execute `XySwap` → X=2, Y=5
- `StoReg(5)`: stores X=2 as "max" — wrong (max is 5)

**Fix:** Invert the test conditions so XySwap fires only when the current X is *not* the target (max or min):
```rust
// Max-finding: swap when X < Y so the larger value ends up in X
Op::Test(TestKind::XLtY),   // X < Y → swap to bring larger (Y) into X
Op::XySwap,
Op::StoReg(5),              // R05 = max(X, Y)

// Min-finding: swap when X > Y so the smaller value ends up in X
Op::Test(TestKind::XGtY),   // X > Y → swap to bring smaller (Y) into X
Op::XySwap,
Op::StoReg(4),              // R04 = min(X, Y)
```
A behavioral test verifying X=min, Y=max for a known 4-value stack must be added.

---

## Warnings

### WR-01: `'S'`, `'R'`, and `Ctrl+A` modal triggers not guarded against active overlays

**File:** `hp41-cli/src/app.rs:172-187`
**Issue:** The STO modal (`'S'`, line 172), RCL modal (`'R'`, line 177), and USER-assign modal (`Ctrl+A`, line 183) are activated before the `show_help` overlay dispatch block (line 230). When the help overlay is open and the user presses `'S'`, `PendingInput::StoRegister` is set without closing the overlay. On the next keypress, `handle_pending_input` runs while `show_help=true` — the UI renders the help overlay but keystrokes silently feed the invisible STO modal. The user has no visual feedback that a modal is active.

**Fix:** Add overlay guards to each modal trigger:
```rust
if key.code == KeyCode::Char('S')
    && !key.modifiers.contains(KeyModifiers::CONTROL)
    && !self.show_help
    && !self.show_programs
{
    self.pending_input = Some(PendingInput::StoRegister(String::new()));
    ...
}
```
Apply the same `!self.show_help && !self.show_programs` guard to the `'R'` and `Ctrl+A` checks.

---

### WR-02: `test_program_names_unique` uses `dedup()` — only detects consecutive duplicates

**File:** `hp41-cli/src/programs.rs:439-444`
**Issue:** `Vec::dedup()` removes only adjacent duplicate elements. A program list with names `["A", "B", "A"]` has the same length before and after `dedup()` and the test passes despite a duplicate. As the program list grows, a non-adjacent duplicate name would go undetected.

**Fix:**
```rust
use std::collections::HashSet;
let names: Vec<&str> = sample_programs().iter().map(|p| p.name).collect();
let unique: HashSet<&str> = names.iter().copied().collect();
assert_eq!(names.len(), unique.len(), "Program names must be unique");
```

---

### WR-03: `prime_test_ops` has no test coverage for n=0 and n=1

**File:** `hp41-cli/src/programs.rs:447-468`
**Issue:** `test_prime_test_correctness` covers n={2,3,4,9,13} but omits n=0 and n=1 — the exact inputs that expose CR-01. The gap-closure comment declares the XySwap bug "fixed" without evidence that edge cases around the boundary condition were verified.

**Fix:** Add to `test_prime_test_correctness`:
```rust
assert_eq!(run_prime(0), HpNum::from(0i32), "prime(0) must be 0");
assert_eq!(run_prime(1), HpNum::from(0i32), "prime(1) must be 0 (not prime by definition)");
```

---

### WR-04: `op_int` (new `Op::Int`) has no unit tests in `hp41-core`

**File:** `hp41-core/src/ops/math.rs:81-85`
**Issue:** `op_int` was added as the enabler for the prime_test gap-closure fix but has no unit tests. The function's correctness for negative inputs, its `LASTX` save behavior (via `unary_result`), and its lift effect are all untested. A regression in `Decimal::trunc()` behavior across a `rust_decimal` version upgrade would go undetected.

**Fix:** Add tests to the `#[cfg(test)]` module in `hp41-core/src/ops/math.rs` or `mod.rs`:
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

#[test]
fn test_op_int_saves_lastx() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::PushNum(HpNum::from_str("3.5").unwrap())).unwrap();
    dispatch(&mut state, Op::Int).unwrap();
    assert_eq!(state.stack.lastx, HpNum::from_str("3.5").unwrap());
}
```

---

### WR-05: `test_fibonacci_runs_without_panic` does not assert the computed value

**File:** `hp41-cli/src/programs.rs:426-436`
**Issue:** The test only calls `assert!(result.is_ok(), ...)` — it never checks the stack after the run. The comment itself says "F(6)=8 or similar," indicating uncertainty rather than specification. A behavioral regression (wrong result, but no error) would pass this test.

**Fix:**
```rust
let result = hp41_core::run_program(&mut state, "A");
assert!(result.is_ok(), "Fibonacci must run without error: {:?}", result);
// F(6) = 8  (sequence: 0,1,1,2,3,5,8,...)
assert_eq!(state.stack.x, HpNum::from(8i32), "Fibonacci(6) must equal 8");
```

---

## Info

### IN-01: `Quadratic Solver` sample program description conflicts with function docstring

**File:** `hp41-cli/src/programs.rs:45` and `197`
**Issue:** The `SampleProgram.description` field (rendered in the UI overlay) reads `"Stack: a(T) b(Z) c(Y) → roots in X,Y."` The function docstring says `"Stack entry: c in X, b in Y, a in Z"`. The code matches the docstring (`StoReg(2)` stores X as c, then two `Rdn` calls extract b and a from Y and Z). The description shown to users is wrong: it implies `a` is in T and nothing is in X, which would leave one register uninitialized.

**Fix:** Update the description string:
```rust
description: "Roots of ax²+bx+c. Stack: c(X) b(Y) a(Z) → root1 in X, root2 in Y.",
```

---

### IN-02: `prime_test_ops` comment uses "floor(n/d)" but `Op::Int` is truncate-toward-zero

**File:** `hp41-cli/src/programs.rs:174`
**Issue:** The comment reads `"X=floor(n/d) (truncate toward zero)"`. These are not synonyms: `floor` rounds toward negative infinity; `trunc` rounds toward zero. For prime testing with positive `n` and `d` they produce the same result, so there is no runtime error, but the comment mixes two distinct operations in a way that would mislead anyone working with negative inputs.

**Fix:**
```rust
Op::Int,  // X = trunc(n/d) — integer part toward zero (equals floor for positive n, d)
```

---

### IN-03: `handle_reg_modal` — `if reg < 100` guard is unreachable dead code

**File:** `hp41-cli/src/app.rs:451`
**Issue:** `new_acc` is always a 2-character string of ASCII decimal digits. The maximum value is `"99"` which parses to `99u8`, always less than 100. The `else` branch producing `"Invalid register: {new_acc}"` is unreachable. The `unwrap_or(0)` fallback on the same line is also superfluous since a valid 2-char ASCII digit string never fails `u8::from_str`.

**Fix:** Remove the dead guard:
```rust
let reg: u8 = new_acc.parse().expect("two ASCII digits always parse as u8 ≤ 99");
self.call_dispatch(op_fn(reg));
self.pending_input = None;
```

---

_Reviewed: 2026-05-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
