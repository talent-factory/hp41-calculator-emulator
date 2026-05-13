# HP-41 %CH (Percent Change) Operation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the HP-41 `%CH` operation (`Y %CH X → ((X − Y) / Y) × 100`, Y preserved) to `hp41-core`, with keystroke surfaces on both `hp41-cli` and `hp41-gui`.

**Architecture:** new `HpNum::checked_pct_change` pure-math method in `num.rs`; new `op_pct_change` orchestrator in `ops/math.rs` reusing the existing `unary_result()` stack helper (because %CH's stack effect is unary — Y is read but not consumed); new `Op::PctChange` enum variant landing simultaneously in `dispatch()`, `execute_op()`, and both `prgm_display.rs` copies to satisfy the CLAUDE.md v2.0 invariant; `%` keystroke wired in CLI `key_to_op()` and in the GUI `resolveKeyId` map; no SVG button on the GUI skin.

**Tech Stack:** Rust 1.88, `rust_decimal` 1.42 (via HpNum), crossterm 0.29 (CLI), Tauri v2.11 + React 18 + TypeScript (GUI), `just` task runner, `/git-workflow:commit --with-skills` for every commit (English commits, per project CLAUDE.md).

**Spec reference:** `docs/superpowers/specs/2026-05-12-percent-change-op-design.md`

---

## File map

**Created/modified:**

| File | Action | Responsibility |
|---|---|---|
| `hp41-core/src/num.rs` | modify | Add `HpNum::checked_pct_change` method |
| `hp41-core/src/tests.rs` | modify | Add HpNum unit tests in `num_scalar_math_tests` mod |
| `hp41-core/src/ops/math.rs` | modify | Add `op_pct_change` orchestrator |
| `hp41-core/src/ops/mod.rs` | modify | Add `Op::PctChange` variant + import + dispatch arm |
| `hp41-core/src/ops/program.rs` | modify | Add `execute_op` arm + import |
| `hp41-core/tests/math_tests.rs` | modify | Add op-level integration tests + PRGM mode tests |
| `hp41-core/tests/numerical_accuracy.rs` | modify | Add 3 `%CH` accuracy cases |
| `hp41-cli/src/keys.rs` | modify | Bind `%` to `Op::PctChange` + `KEY_REF_TABLE` row + test |
| `hp41-cli/src/help_data.rs` | modify | Add `%` row to `HELP_DATA` |
| `hp41-cli/src/prgm_display.rs` | modify | Add `Op::PctChange => "%CH"` arm |
| `hp41-gui/src-tauri/src/key_map.rs` | modify | Add `"pct_change"` arm + test |
| `hp41-gui/src-tauri/src/prgm_display.rs` | modify | Add `Op::PctChange => "%CH"` arm |
| `hp41-gui/src/App.tsx` | modify | Add `'%'` → `"pct_change"` entry in `resolveKeyId` MAP |

**Untouched (deliberately):** `state.rs`, `stack.rs`, `error.rs`, `ops/mod.rs::synthetic_byte_to_op`, `Keyboard.tsx`.

---

## Task 1 — `HpNum::checked_pct_change` (TDD)

**Files:**
- Modify: `hp41-core/src/num.rs` (add method)
- Test: `hp41-core/src/tests.rs` (`num_scalar_math_tests` mod, ~line 375)

- [ ] **Step 1: Write the failing tests**

Append to the `num_scalar_math_tests` module in `hp41-core/src/tests.rs` (find the existing module that already tests `checked_recip`, `checked_powd`, etc.):

```rust
    // ── checked_pct_change ────────────────────────────────────────────────
    // %CH: ((X − self) / self) × 100, where self is Y (base) and x is X (new).
    #[test]
    fn checked_pct_change_plus_10_percent() {
        // Y=100, X=110 → +10
        let y = HpNum::from(100i32);
        let x = HpNum::from(110i32);
        assert_eq!(y.checked_pct_change(&x).unwrap(), HpNum::from(10i32));
    }

    #[test]
    fn checked_pct_change_minus_10_percent() {
        // Y=100, X=90 → −10
        let y = HpNum::from(100i32);
        let x = HpNum::from(90i32);
        assert_eq!(y.checked_pct_change(&x).unwrap(), HpNum::from(-10i32));
    }

    #[test]
    fn checked_pct_change_plus_50_percent() {
        // Y=80, X=120 → +50
        let y = HpNum::from(80i32);
        let x = HpNum::from(120i32);
        assert_eq!(y.checked_pct_change(&x).unwrap(), HpNum::from(50i32));
    }

    #[test]
    fn checked_pct_change_zero_when_equal() {
        // Y=42, X=42 → 0
        let y = HpNum::from(42i32);
        let x = HpNum::from(42i32);
        assert_eq!(y.checked_pct_change(&x).unwrap(), HpNum::zero());
    }

    #[test]
    fn checked_pct_change_negative_base() {
        // Y=−100, X=−80: ((−80 − (−100)) / (−100)) × 100 = 20 / −100 × 100 = −20
        let y = HpNum::from(-100i32);
        let x = HpNum::from(-80i32);
        assert_eq!(y.checked_pct_change(&x).unwrap(), HpNum::from(-20i32));
    }

    #[test]
    fn checked_pct_change_sign_cross_zero() {
        // Y=10, X=−5: (−5 − 10) / 10 × 100 = −15 / 10 × 100 = −150
        let y = HpNum::from(10i32);
        let x = HpNum::from(-5i32);
        assert_eq!(y.checked_pct_change(&x).unwrap(), HpNum::from(-150i32));
    }

    #[test]
    fn checked_pct_change_tiny_delta_rounds_to_10_sig_digits() {
        // Y=1_000_000_000, X=1_000_000_001
        // (1 / 1_000_000_000) × 100 = 1e−7 = 0.0000001
        let y = HpNum::from(1_000_000_000i32);
        let x = HpNum::from(1_000_000_001i32);
        let expected = HpNum(Decimal::from_str("0.0000001").unwrap());
        assert_eq!(y.checked_pct_change(&x).unwrap(), expected);
    }

    #[test]
    fn checked_pct_change_y_zero_returns_divide_by_zero() {
        let y = HpNum::zero();
        let x = HpNum::from(42i32);
        assert_eq!(y.checked_pct_change(&x), Err(HpError::DivideByZero));
    }

    #[test]
    fn checked_pct_change_overflow_at_times_100() {
        // Y=1, X=1e27 → ratio = (1e27 − 1) rounded to 10 sig digits = 1e27
        // 1e27 × 100 = 1e29 → exceeds rust_decimal max (~7.9e28) → Overflow
        let y = HpNum::from(1i32);
        let x = HpNum(Decimal::from_str("1000000000000000000000000000").unwrap());
        assert_eq!(y.checked_pct_change(&x), Err(HpError::Overflow));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `just test-core --lib tests::num_scalar_math_tests`

Alternative if the recipe does not accept lib filters: `cargo test -p hp41-core --lib num_scalar_math_tests::checked_pct_change` (the project uses `just`; fall back to `cargo` only if `just` cannot scope.)

Expected: 9 failures — `error[E0599]: no method named 'checked_pct_change' found for struct 'HpNum'`.

- [ ] **Step 3: Implement `HpNum::checked_pct_change`**

In `hp41-core/src/num.rs`, add the following method to `impl HpNum`, placing it directly after `checked_powd` (the existing binary math method, around the "Scalar math methods" section):

```rust
    /// %CH — percent change from self (base, Y) to x (new value, X).
    /// Computes `((x − self) / self) × 100`.
    /// Returns `DivideByZero` if self is zero; `Overflow` on intermediate or final overflow.
    /// Sign emerges naturally from the arithmetic — negative bases are not special-cased.
    pub fn checked_pct_change(&self, x: &HpNum) -> Result<HpNum, HpError> {
        let delta = x.checked_sub(self)?; // X − Y
        let ratio = delta.checked_div(self)?; // / Y  (DivideByZero if Y=0)
        ratio.checked_mul(&HpNum::from(100i32)) // × 100
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `just test-core --lib tests::num_scalar_math_tests`

Expected: all 9 new `checked_pct_change_*` tests PASS. Existing tests in the module remain green.

- [ ] **Step 5: Commit**

```bash
/git-workflow:commit --with-skills
```

When the skill prompts for context, indicate this is "feat: add HpNum::checked_pct_change pure-math method for HP-41 %CH" (English only — overrides any plugin default). Stage only `hp41-core/src/num.rs` and `hp41-core/src/tests.rs`.

---

## Task 2 — `Op::PctChange` type-system surface + `op_pct_change` orchestrator

**Goal of this task:** add the new enum variant and ALL exhaustive-match arms simultaneously so the workspace stays compileable. This is the CLAUDE.md v2.0 invariant: a new `Op` variant must land in `dispatch()`, `execute_op()`, and BOTH `prgm_display.rs` copies before any caller compiles.

**Files (modified in one commit):**
- `hp41-core/src/ops/math.rs` — add `op_pct_change` fn
- `hp41-core/src/ops/mod.rs` — add `PctChange` variant + import + dispatch arm
- `hp41-core/src/ops/program.rs` — add `execute_op` arm + import
- `hp41-cli/src/prgm_display.rs` — add display-name arm
- `hp41-gui/src-tauri/src/prgm_display.rs` — add display-name arm

- [ ] **Step 1: Add `op_pct_change` in `hp41-core/src/ops/math.rs`**

Append directly after `op_ypow` (the existing binary math op, around line 130 of `math.rs`):

```rust
/// %CH: ((X − Y) / Y) × 100, leaving Y on the stack.
///
/// HP-41 % family — reads Y as base and X as new value, but does NOT
/// consume Y. Stack effect is unary (LASTX←oldX, X←result, Y/Z/T fixed) —
/// we reuse `unary_result()` even though the math is binary. Future base `%`
/// and `Δ%` ops will follow the same pattern.
/// LiftEffect: Enable (via unary_result).
pub fn op_pct_change(state: &mut CalcState) -> Result<(), HpError> {
    let result = state.stack.y.checked_pct_change(&state.stack.x)?;
    unary_result(state, result);
    Ok(())
}
```

The `unary_result` import already exists at the top of `math.rs`; no import change needed.

- [ ] **Step 2: Add `Op::PctChange` variant + dispatch arm + import in `hp41-core/src/ops/mod.rs`**

In the `Op` enum (Phase 2 math block, right after the `YPow` variant — around line 104):

```rust
    /// %CH — percent change ((X−Y)/Y)×100; Y preserved. LiftEffect: Enable.
    PctChange,
```

In the `math::{…}` use import at the top of the file (around line 21), add `op_pct_change` alphabetically:

```rust
use math::{
    op_acos, op_asin, op_atan, op_cos, op_exp, op_int, op_ln, op_log, op_pct_change, op_recip,
    op_set_deg, op_set_grad, op_set_rad, op_sin, op_sq, op_sqrt, op_tan, op_tenpow, op_ypow,
};
```

In the `dispatch()` match (Phase 2 math/trig/angle ops block, after `Op::YPow => op_ypow(state),` around line 325):

```rust
        Op::PctChange => op_pct_change(state),
```

- [ ] **Step 3: Add `execute_op` arm + import in `hp41-core/src/ops/program.rs`**

In the `use crate::ops::math::{…}` block inside `execute_op` (around line 224), add `op_pct_change` alphabetically:

```rust
    use crate::ops::math::{
        op_acos, op_asin, op_atan, op_cos, op_exp, op_int, op_ln, op_log, op_pct_change,
        op_recip, op_set_deg, op_set_grad, op_set_rad, op_sin, op_sq, op_sqrt, op_tan,
        op_tenpow, op_ypow,
    };
```

In the `match op` block, after `Op::YPow => op_ypow(state),` (around line 257):

```rust
        Op::PctChange => op_pct_change(state),
```

- [ ] **Step 4: Add display arm in `hp41-cli/src/prgm_display.rs`**

Locate the `op_display_name` function (or the equivalent exhaustive `match op { Op::Foo => "FOO".to_string(), … }` block). After the `Op::YPow => "Y^X".to_string(),` arm, add:

```rust
        Op::PctChange => "%CH".to_string(),
```

(If `prgm_display.rs` uses a `&'static str` return type instead of `String`, drop `.to_string()` and use `"%CH"`. Match the convention of the surrounding arms exactly.)

- [ ] **Step 5: Add display arm in `hp41-gui/src-tauri/src/prgm_display.rs`**

Same arm, exact same position relative to `Op::YPow`:

```rust
        Op::PctChange => "%CH".to_string(),
```

- [ ] **Step 6: Build green**

Run: `just check`

Expected: clean `cargo check` across the root workspace (`hp41-core` + `hp41-cli`).

Then run: `just gui-check`

Expected: clean `cargo check` inside the nested `hp41-gui/src-tauri` workspace. If either fails with `non-exhaustive patterns: Op::PctChange not covered`, a `prgm_display.rs` arm was missed — revisit Steps 4 and 5.

- [ ] **Step 7: Run the full library test suite to confirm no regressions**

Run: `just test`

Expected: all existing tests still pass (this task only added code paths, did not change any). The Task 1 `checked_pct_change` tests stay green.

- [ ] **Step 8: Commit**

```bash
/git-workflow:commit --with-skills
```

Stage all five modified files. Suggested subject: "feat: add Op::PctChange variant + op_pct_change orchestrator". English only.

---

## Task 3 — Op-level integration tests (stack mechanics)

**Files:**
- Modify: `hp41-core/tests/math_tests.rs` (the existing integration-test file that already houses `test_ypow_*` and other math op-level tests)

- [ ] **Step 1: Write the failing tests**

Append to `hp41-core/tests/math_tests.rs`. The file already has helpers like `Decimal::from(_)` and uses `dispatch(&mut s, Op::YPow)` — match that style.

```rust
// ── %CH (percent change) op-level integration tests ───────────────────────────
// These cover the stack mechanics: Y preservation (the defining feature of
// the HP-41 % family), LASTX capture, lift_enabled, and error atomicity.

#[test]
fn test_pct_change_basic_plus_15_percent() {
    // 200 ENTER 230 %CH → X=15
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(15));
}

#[test]
fn test_pct_change_preserves_y() {
    // The DEFINING test for this op. Y must survive intact.
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    s.stack.z = HpNum::from(7i32);
    s.stack.t = HpNum::from(13i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(15), "X must be the result");
    assert_eq!(s.stack.y.inner(), Decimal::from(200), "Y must be preserved (% family)");
    assert_eq!(s.stack.z.inner(), Decimal::from(7), "Z must be untouched");
    assert_eq!(s.stack.t.inner(), Decimal::from(13), "T must be untouched");
}

#[test]
fn test_pct_change_saves_old_x_to_lastx() {
    // LASTX must capture the *old* X (230), not the result (15).
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.lastx.inner(), Decimal::from(230));
}

#[test]
fn test_pct_change_enables_lift() {
    // After %CH, the next number-entry must lift the stack (not overwrite X).
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(100i32);
    s.stack.x = HpNum::from(125i32);
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::PctChange).unwrap();
    assert!(s.stack.lift_enabled, "%CH must enable stack lift after execution");
}

#[test]
fn test_pct_change_divide_by_zero_leaves_stack_untouched() {
    // Atomicity invariant: Err path makes no partial writes.
    let mut s = CalcState::new();
    s.stack.y = HpNum::zero();
    s.stack.x = HpNum::from(42i32);
    s.stack.z = HpNum::from(7i32);
    s.stack.t = HpNum::from(13i32);
    let lastx_before = s.stack.lastx.clone();
    let result = dispatch(&mut s, Op::PctChange);
    assert_eq!(result, Err(HpError::DivideByZero));
    assert_eq!(s.stack.y, HpNum::zero(), "Y must be untouched on Err");
    assert_eq!(s.stack.x.inner(), Decimal::from(42), "X must be untouched on Err");
    assert_eq!(s.stack.z.inner(), Decimal::from(7), "Z must be untouched on Err");
    assert_eq!(s.stack.t.inner(), Decimal::from(13), "T must be untouched on Err");
    assert_eq!(s.stack.lastx, lastx_before, "LASTX must be untouched on Err");
}

#[test]
fn test_pct_change_lastx_round_trip() {
    // 200 ENTER 230 %CH LASTX  →  X is the *original* 230 (not 15).
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(15)); // sanity
    dispatch(&mut s, Op::Lastx).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(230), "LASTX must restore old X");
}

#[test]
fn test_pct_change_chained_invocation() {
    // Y=100, X=125 → %CH → X=25, Y still 100.
    // Then enter 150 → X=150, Y=25 (lift). Then %CH again with the original Y=100?
    // Actually, after the first %CH and stack lift on entry, Y will be 25 (the prior result),
    // NOT 100. This test verifies the lift behaviour matches every other value-producing op.
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(100i32);
    s.stack.x = HpNum::from(125i32);
    dispatch(&mut s, Op::PctChange).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(25));
    assert_eq!(s.stack.y.inner(), Decimal::from(100));
    // Now push 150 via Op::PushNum (integration tests use the public dispatch path,
    // not crate::stack::enter_number, since stack is a separate public module and
    // PushNum honours lift_enabled the same way enter_number does).
    dispatch(&mut s, Op::PushNum(HpNum::from(150i32))).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(150));
    assert_eq!(s.stack.y.inner(), Decimal::from(25), "Y after lift is prior X (25)");
}
```

- [ ] **Step 2: Run tests to verify they fail or pass deliberately**

Run: `just test-core --test math_tests`

Expected: all 7 new `test_pct_change_*` tests PASS immediately, because Task 2 already wired the op end-to-end. This is acceptable: Task 1 was the TDD-red-then-green step for the pure math; this Task 3 is a *verification* layer covering stack mechanics that Task 1 cannot reach.

If any of the 7 tests FAIL, that is a bug in Task 2's wiring — return to Task 2 and fix the offending arm.

- [ ] **Step 3: Commit**

```bash
/git-workflow:commit --with-skills
```

Stage only `hp41-core/tests/math_tests.rs`. Suggested subject: "test: add %CH op-level stack-mechanics integration tests".

---

## Task 4 — PRGM mode tests (recording + playback)

**Files:**
- Modify: `hp41-core/tests/math_tests.rs` (continue with the same test file as Task 3)

- [ ] **Step 1: Write the PRGM-mode tests**

Append after the Task 3 block in `hp41-core/tests/math_tests.rs`:

```rust
// ── %CH PRGM mode: recording and playback ─────────────────────────────────────

#[test]
fn test_pct_change_recorded_into_program_when_prgm_mode() {
    // In prgm_mode = true, dispatching Op::PctChange must APPEND to state.program
    // (recording) and MUST NOT touch the stack.
    let mut s = CalcState::new();
    s.stack.y = HpNum::from(200i32);
    s.stack.x = HpNum::from(230i32);
    s.prgm_mode = true;
    let program_len_before = s.program.len();

    dispatch(&mut s, Op::PctChange).unwrap();

    assert_eq!(
        s.program.len(),
        program_len_before + 1,
        "PctChange must be appended to program Vec"
    );
    assert_eq!(
        s.program.last(),
        Some(&Op::PctChange),
        "the appended op must be PctChange"
    );
    assert_eq!(s.stack.x.inner(), Decimal::from(230), "stack must be untouched in PRGM mode");
    assert_eq!(s.stack.y.inner(), Decimal::from(200), "stack must be untouched in PRGM mode");
}

#[test]
fn test_pct_change_playback_via_run_program() {
    // Build a tiny program: LBL "T", PushNum(200), Enter, PushNum(230), PctChange, Rtn.
    // Run it. Expect X=15, Y=200.
    use hp41_core::ops::program::run_program;
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("T".to_string()),
        Op::PushNum(HpNum::from(200i32)),
        Op::Enter,
        Op::PushNum(HpNum::from(230i32)),
        Op::PctChange,
        Op::Rtn,
    ];
    // Start execution at LBL "T" (index 0). Use the existing run_program entry point —
    // match the call style used by other tests in this file for op_ypow / op_sqrt.
    run_program(&mut s, "T").unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(15), "playback result must equal 15");
    assert_eq!(s.stack.y.inner(), Decimal::from(200), "Y preserved after playback");
}
```

If the `run_program` signature in this codebase differs (e.g. takes `&str` vs `String`, or returns a different `Result` type), match the calling convention already used elsewhere in `math_tests.rs` for other multi-op program tests. Grep for `run_program(` in `hp41-core/tests/` to find the convention.

- [ ] **Step 2: Run the tests**

Run: `just test-core --test math_tests test_pct_change`

Expected: both new PRGM tests PASS. Combined with Task 3, all 9 `test_pct_change_*` tests are green.

- [ ] **Step 3: Commit**

```bash
/git-workflow:commit --with-skills
```

Stage `hp41-core/tests/math_tests.rs`. Suggested subject: "test: add %CH PRGM-mode recording and playback tests".

---

## Task 5 — CLI surface: `%` keystroke + KEY_REF_TABLE + HELP_DATA

**Files:**
- Modify: `hp41-cli/src/keys.rs`
- Modify: `hp41-cli/src/help_data.rs`

- [ ] **Step 1: Write the failing CLI keystroke test**

Append to the `#[cfg(test)] mod tests` block at the bottom of `hp41-cli/src/keys.rs` (the block that already houses `test_q_dispatches_sin`, `test_g_dispatches_clreg`):

```rust
    #[test]
    fn test_pct_keystroke_dispatches_pct_change() {
        // '%' maps to Op::PctChange — verify the op produces the right result and preserves Y.
        let mut state = CalcState::new();
        state.stack.y = hp41_core::HpNum::from(100);
        state.stack.x = hp41_core::HpNum::from(125);
        let result = hp41_core::ops::dispatch(&mut state, Op::PctChange);
        assert!(result.is_ok(), "Op::PctChange must not error on valid input");
        assert_eq!(format!("{}", state.stack.x), "25", "%CH(100→125) must be 25");
        assert_eq!(format!("{}", state.stack.y), "100", "Y must be preserved");
    }

    #[test]
    fn test_key_ref_table_has_pct_entry() {
        let has_pct = KEY_REF_TABLE
            .iter()
            .any(|(k, desc)| *k == "%" && desc.contains("%CH"));
        assert!(has_pct, "KEY_REF_TABLE must contain a '%' → %CH entry");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `just test --bin hp41-cli`

Alternative scoped: `cargo test -p hp41-cli --lib tests::test_pct_keystroke_dispatches_pct_change tests::test_key_ref_table_has_pct_entry`

Expected: `test_key_ref_table_has_pct_entry` fails ("KEY_REF_TABLE must contain a '%' → %CH entry"). `test_pct_keystroke_dispatches_pct_change` likely passes already (Op::PctChange is wired), but is included for symmetry with the existing `test_q_dispatches_sin` pattern.

- [ ] **Step 3: Add the `%` keystroke binding**

In `hp41-cli/src/keys.rs`, inside `key_to_op()`, add a new arm near the other punctuation-key bindings (the `+`, `-`, `*`, `/` block, around line 24):

```rust
        // %CH — percent change: ((X−Y)/Y)×100, Y preserved (the HP-41 % family).
        // '%' was unbound; crossterm delivers Shift+5 as KeyCode::Char('%') reliably,
        // same mechanism as 'S'/'L'/'+'.
        KeyCode::Char('%') => Some(Op::PctChange),
```

- [ ] **Step 4: Add the KEY_REF_TABLE row**

In the same file, in the `KEY_REF_TABLE` constant (around line 90), insert a new tuple near the binary-math rows (after the `"Y"` → y^x entry is a good spot):

```rust
    ("%", "%CH (percent change: ((X\u{2212}Y)/Y)\u{00D7}100, Y preserved)"),
```

(Uses the same unicode escape style as the existing rows for ± / × characters.)

- [ ] **Step 5: Add the HELP_DATA row**

Open `hp41-cli/src/help_data.rs`. The `HELP_DATA` constant has shape `&[(&str, &str, &str)]` = `(key_binding, hp41_op_name, description)`. Insert directly after the `Y^x` row (around line 29), still inside the `Arithmetic` category:

```rust
    ("%", "%CH", "Percent change: ((X−Y)/Y)×100, Y preserved (% family)"),
```

(Use the literal `−` and `×` characters as nearby rows do — `help_data.rs` writes them inline rather than via `\u{}` escapes.)

- [ ] **Step 6: Add the `prgm_display.rs` arm if not already done**

Verify Task 2 Step 4 landed correctly:

Run: `grep -n "PctChange" hp41-cli/src/prgm_display.rs`

Expected: one match showing `Op::PctChange => "%CH".to_string(),` (or string-slice equivalent). If missing, return to Task 2 Step 4.

- [ ] **Step 7: Run the CLI tests**

Run: `just test --bin hp41-cli` (or fall back to `cargo test -p hp41-cli`)

Expected: both new `test_pct_*` tests PASS; all existing CLI tests stay green.

- [ ] **Step 8: Manual smoke test (CLI)**

Run: `just run`

In the TUI, type: `200 Enter 230 %`. Expected display: `X: 15`, `Y: 200`. Press `l` (LASTX): X becomes 230. Press `?` and confirm the help overlay shows the `%` → `%CH` row.

Exit with `Ctrl+C`.

- [ ] **Step 9: Commit**

```bash
/git-workflow:commit --with-skills
```

Stage `hp41-cli/src/keys.rs` and `hp41-cli/src/help_data.rs`. Suggested subject: "feat(cli): bind '%' to Op::PctChange + KEY_REF_TABLE + help overlay".

---

## Task 6 — GUI surface: key_map.rs + App.tsx (no SVG button)

**Files:**
- Modify: `hp41-gui/src-tauri/src/key_map.rs`
- Modify: `hp41-gui/src/App.tsx`

- [ ] **Step 1: Write the failing key_map test**

In the `#[cfg(test)] mod tests` block at the bottom of `hp41-gui/src-tauri/src/key_map.rs` (the block that already tests `resolve("plus") == Op::Add` etc.), append:

```rust
    #[test]
    fn resolve_pct_change_id() {
        assert_eq!(resolve("pct_change").unwrap(), Op::PctChange);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `just gui-check` (or scope: `cd hp41-gui/src-tauri && cargo test --lib key_map::tests::resolve_pct_change_id`)

Expected: the test fails — `resolve("pct_change")` currently returns `Err(GuiError::UnknownKey(_))` (or whatever the default-arm error variant is).

- [ ] **Step 3: Add the `"pct_change"` arm**

In `hp41-gui/src-tauri/src/key_map.rs`, inside the `resolve()` match, add a new arm in the "Unary math" section (it's the closest grouping to %CH — the `ypow` entry is the existing binary outlier and sits there):

```rust
        "pct_change" => Ok(Op::PctChange),
```

Insert it directly after `"ypow" => Ok(Op::YPow),`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `just gui-check` (then `cd hp41-gui/src-tauri && cargo test key_map::tests::resolve_pct_change_id`)

Expected: PASS.

- [ ] **Step 5: Wire the `%` keystroke in `App.tsx`**

In `hp41-gui/src/App.tsx`, locate the `MAP` Record inside `resolveKeyId` (around line 41). Add a new entry inside the object literal:

```ts
    '%': 'pct_change',   // %CH (percent change) — no SVG button; physical keyboard only
```

Place it near the binary-math entries (after `'Y': 'ypow',` is a natural spot to keep the file co-located).

Verify the modal-trigger ignore list a few lines above (`'SRfFPX'`) does NOT contain `%` — it doesn't (current contents are letters only), so no conflict.

- [ ] **Step 6: Confirm `prgm_display.rs` arm is in place**

Run: `grep -n "PctChange" hp41-gui/src-tauri/src/prgm_display.rs`

Expected: one match showing `Op::PctChange => "%CH".to_string(),`. If missing, return to Task 2 Step 5.

- [ ] **Step 7: Run the GUI CI checks**

Run: `just gui-ci`

Expected: clean. This runs `cargo test` then `cargo build --release` in the nested workspace per CLAUDE.md.

- [ ] **Step 8: Manual smoke test (GUI)**

Run: `just gui-dev`

Wait for the Vite dev server + Tauri window to come up. Click the display area to focus, then type: `2 0 0 Enter 2 3 0 %`.

Expected:
- X line shows `15`
- Y line shows `200`
- Press the `LASTX` SVG button — X becomes `230`
- Toggle PRGM mode (click `PRGM` or whatever the GUI exposes), enter `200`, `Enter`, `230`, then press `%`. The program panel should show a new line with `%CH` as the op label.

Close the dev window.

- [ ] **Step 9: Commit**

```bash
/git-workflow:commit --with-skills
```

Stage `hp41-gui/src-tauri/src/key_map.rs` and `hp41-gui/src/App.tsx`. Suggested subject: "feat(gui): bind '%' to Op::PctChange via resolveKeyId + key_map".

---

## Task 7 — Numerical accuracy suite: 3 `%CH` cases

**Files:**
- Modify: `hp41-core/tests/numerical_accuracy.rs`

- [ ] **Step 1: Add 3 `%CH` cases to the suite**

The suite uses a consistent style: pre-set Y and X, dispatch `Op::Foo`, assert the result against an `expected` constant. Grep for a `Op::YPow` block in `numerical_accuracy.rs` (around line 825) to match the style.

Append 3 cases at the end of the suite (or in the most natural grouping — the file is organized by op family; a new `// Cases NNN–NNN: %CH (percent change)` block at the bottom is fine):

```rust
    // Cases: %CH (percent change) — additive, not part of the 500-case baseline.
    {
        // +25%: Y=80, X=100 → 25
        let mut s = CalcState::new();
        s.stack.y = HpNum::from(80i32);
        s.stack.x = HpNum::from(100i32);
        dispatch(&mut s, Op::PctChange).unwrap();
        assert_eq!(s.stack.x.inner(), Decimal::from(25));
    }
    {
        // −33.33333333% (10 sig digits): Y=300, X=200
        // (200 − 300) / 300 × 100 = −100/300 × 100 = −33.33333333…
        let mut s = CalcState::new();
        s.stack.y = HpNum::from(300i32);
        s.stack.x = HpNum::from(200i32);
        dispatch(&mut s, Op::PctChange).unwrap();
        let expected = Decimal::from_str("-33.33333333").unwrap();
        assert_eq!(s.stack.x.inner(), expected);
    }
    {
        // Doubling: Y=50, X=100 → +100
        let mut s = CalcState::new();
        s.stack.y = HpNum::from(50i32);
        s.stack.x = HpNum::from(100i32);
        dispatch(&mut s, Op::PctChange).unwrap();
        assert_eq!(s.stack.x.inner(), Decimal::from(100));
    }
```

If the suite uses a different harness pattern (a `cases` array consumed by a loop, for example), inspect the existing structure and add 3 entries in that style instead. Whatever pattern is already used is the right one.

- [ ] **Step 2: Run the suite**

Run: `just test-core --test numerical_accuracy`

Expected: still ≥98% pass rate (the gate). As-shipped: the suite total moves from 500 → 503; tighten the gate proportionally to `passes >= 493` in the same commit so the 98% policy strength is preserved at the new total. New cases must all pass deterministically.

- [ ] **Step 3: Commit**

```bash
/git-workflow:commit --with-skills
```

Stage `hp41-core/tests/numerical_accuracy.rs`. Suggested subject: "test: add %CH cases to numerical accuracy suite".

---

## Task 8 — Full CI dry-run and final validation

**Goal:** confirm nothing regressed across the workspace before declaring the feature done.

- [ ] **Step 1: Run the full CLI/CORE CI**

Run: `just ci`

Expected: green. This runs format-check, clippy with `-D warnings`, tests, MSRV check, and coverage (if `just ci` is wired that way — match the recipe in the project's `justfile`).

If the coverage gate trips (`<80%`), inspect the report: the new code is ~30 LoC across `num.rs` + `math.rs` + `mod.rs` and is well-covered by Tasks 1, 3, 4. A failure here is most likely the spec-suspected coverage drift since v1.0, not anything %CH-specific — note it in the commit/PR but do not block on it.

- [ ] **Step 2: Run the full GUI CI**

Run: `just gui-ci`

Expected: green across the 3-OS matrix locally (just the current OS; the GitHub Actions matrix will exercise Win/macOS/Ubuntu when pushed).

- [ ] **Step 3: Verify the SC-4 invariant grep**

The CLAUDE.md SC-4 invariant: no calculator/math logic duplicated in `hp41-gui`. Run the stricter pattern from CLAUDE.md:

```bash
grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum\|pct_change)" hp41-gui/src-tauri/src/
```

Expected: NO output. `pct_change` must NOT appear anywhere in `hp41-gui/src-tauri/src/` — the GUI crate only references `Op::PctChange` via the `resolve()` string lookup and via `prgm_display`'s display-name match.

- [ ] **Step 4: Run a smoke test that exercises persistence parity**

Run: `just run` (CLI). Enter `200 ENTER 230 %`. Press `Ctrl+S` to save. Quit.

Run: `just gui-dev` (GUI). The auto-loaded state should show X=15, Y=200.

Press `LASTX` on the GUI — X becomes 230. This validates that the v2.0 invariant "GUI and CLI share `~/.hp41/autosave.json`" still holds with the new op recorded in the stack/LASTX fields.

Close the GUI.

- [ ] **Step 5: Final cleanup commit (if any)**

If you needed any small fixups during Step 3 or 4, commit them now:

```bash
/git-workflow:commit --with-skills
```

If no fixups were needed, skip this step. The series is done.

---

## Verification checklist

- [ ] `HpNum::checked_pct_change` exists in `num.rs` and is covered by 9 unit tests.
- [ ] `op_pct_change` exists in `ops/math.rs` and reuses `unary_result()` (Y preserved).
- [ ] `Op::PctChange` appears in the `Op` enum and in BOTH `dispatch()` AND `execute_op()` AND BOTH `prgm_display.rs` copies.
- [ ] `%` keystroke is bound in `hp41-cli/src/keys.rs::key_to_op()` and in `hp41-gui/src/App.tsx::resolveKeyId`.
- [ ] `KEY_REF_TABLE` and `HELP_DATA` both have a `%` row.
- [ ] `hp41-gui/src-tauri/src/key_map.rs::resolve` handles `"pct_change"`.
- [ ] No SVG key added in `Keyboard.tsx` (pixel-perfect skin preserved).
- [ ] No code in `synthetic_byte_to_op` (deliberately deferred).
- [ ] `just ci` and `just gui-ci` both green.
- [ ] SC-4 invariant grep returns no output for `pct_change` in `hp41-gui/src-tauri/src/`.
- [ ] Smoke-tested in both CLI (`just run`) and GUI (`just gui-dev`): `200 ENTER 230 %` → X=15, Y=200.
