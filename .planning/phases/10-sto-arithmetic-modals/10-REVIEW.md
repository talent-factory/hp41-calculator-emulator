---
phase: 10-sto-arithmetic-modals
reviewed: 2026-05-08T12:28:29Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - hp41-cli/src/app.rs
  - hp41-cli/src/help_data.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/registers.rs
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
fixed_during_review:
  - CR-01: step-2 routing and Y/Z/T/L dispatch restored in app.rs (merge conflict resolution had lost these)
  - WR-01: plain STO key column corrected from "Shift+R" to "S" in help_data.rs
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-05-08T12:28:29Z
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 10 added the `StackReg` enum, `Op::StoArithStack` variant, `op_sto_arith_stack()`, dispatch arms in both `dispatch()` and `execute_op()`, and program display support. The hp41-core layer (Plans 01, 03) is implemented correctly and passes all tests.

**However, Plan 02 — the TUI modal routing in `hp41-cli/src/app.rs` — was never merged into `develop`.** The worktree branch `worktree-agent-a39e877e38d64dd41` contains the completed Plan 02 implementation (commits `a306c78` and `fcb588a`), but only Plans 01 and 03 were merged. The git log confirms: `chore: merge Plan 01` and `chore: merge Plan 03` appear on `develop`; there is no `chore: merge Plan 02` commit. The ROADMAP marks `10-02` as `[x]` complete, and the docs commit `cf06889` says "all 3 plans executed — pending verification", but the source code tells a different story.

The consequence is that the entire STO arithmetic feature (keyboard flow S → op → register/stack-reg) is non-functional from the TUI. The `#[allow(dead_code)]` attributes on `StoAdd`/`StoSub`/`StoMul`/`StoDiv` in `PendingInput` were supposed to be removed in Plan 02; they remain, confirming the merge was skipped.

The help overlay (Plan 03) correctly documents the modal flow (`S +/-/*/÷`) even though the flow itself is not wired up. This creates a discoverability gap where the help overlay shows a feature that does not work.

---

## Critical Issues

### CR-01: Plan 02 (TUI Modal Routing) Never Merged — STO Arithmetic Keyboard Flow Is Non-Functional

**File:** `hp41-cli/src/app.rs:393-434`
**Issue:** The `StoRegister` match arm in `handle_pending_input()` delegates directly to `handle_reg_modal()` with `Op::StoReg` as the op — it never intercepts `+`, `-`, `*`, `/` to transition into `StoAdd`/`StoSub`/`StoMul`/`StoDiv` modal states. Consequently:

1. Pressing `S` then `+` in the TUI silently ignores the `+` (it falls into the `_` arm of `handle_reg_modal`, which restores `StoRegister("")`) — the `StoAdd` state is never entered.
2. The `StoAdd`/`StoSub`/`StoMul`/`StoDiv` match arms (lines 399–434) are structurally reachable but semantically unreachable from any keyboard path — no code creates these `PendingInput` states.
3. The `Y`/`Z`/`T`/`L` single-key dispatch to `Op::StoArithStack` is entirely absent — these arms still call `handle_reg_modal` which accepts only digit keys and never produces `Op::StoArithStack`.
4. The four `#[allow(dead_code)]` attributes (lines 27, 29, 31, 33) confirm the compiler itself flags these variants as dead.

Git evidence: commits `a306c78` and `fcb588a` on `worktree-agent-a39e877e38d64dd41` implement the fix but were never merged to `develop`. Only Plans 01 and 03 were merged (`100d498` and `2dcfde0`). The ROADMAP falsely marks `10-02` as `[x]`.

**Fix:** Merge the Plan 02 worktree branch, or apply the two changes from `10-02-PLAN.md`:

**Step-2 routing in StoRegister arm (replaces current line 393–395):**
```rust
Some(PendingInput::StoRegister(ref acc)) => {
    match key.code {
        KeyCode::Char('+') => {
            self.pending_input = Some(PendingInput::StoAdd(String::new()));
        }
        KeyCode::Char('-') => {
            self.pending_input = Some(PendingInput::StoSub(String::new()));
        }
        KeyCode::Char('*') => {
            self.pending_input = Some(PendingInput::StoMul(String::new()));
        }
        KeyCode::Char('/') => {
            self.pending_input = Some(PendingInput::StoDiv(String::new()));
        }
        _ => {
            self.handle_reg_modal(key, acc.clone(), Op::StoReg, PendingInput::StoRegister)
        }
    }
}
```

**Y/Z/T/L dispatch in each StoAdd arm (replaces current lines 399–407; replicate for Sub/Mul/Div):**
```rust
Some(PendingInput::StoAdd(ref acc)) => {
    match key.code {
        KeyCode::Char('Y') | KeyCode::Char('y') => {
            self.call_dispatch(Op::StoArithStack {
                kind: hp41_core::ops::StoArithKind::Add,
                stack_reg: hp41_core::ops::StackReg::Y,
            });
            self.pending_input = None;
        }
        KeyCode::Char('Z') | KeyCode::Char('z') => { /* ... StackReg::Z */ }
        KeyCode::Char('T') | KeyCode::Char('t') => { /* ... StackReg::T */ }
        KeyCode::Char('L') | KeyCode::Char('l') => { /* ... StackReg::LastX */ }
        _ => {
            self.handle_reg_modal(
                key, acc.clone(),
                |reg| Op::StoArith { reg, kind: hp41_core::ops::StoArithKind::Add },
                PendingInput::StoAdd,
            )
        }
    }
}
```

Also remove the four `#[allow(dead_code)]` attributes and update the comment on lines 25–26.

---

## Warnings

### WR-01: help_data.rs Key Binding for STO Contradicts Itself

**File:** `hp41-cli/src/help_data.rs:65`
**Issue:** The key binding column for the `STO [nn]` entry is `"Shift+R"`, but the description body on line 67 correctly says `"press S then 2 digits"`, and the actual binding in `app.rs:167` is `KeyCode::Char('S')`. The column value `"Shift+R"` is factually wrong and will mislead users consulting the `?` overlay. (Note: this is distinct from the STO arithmetic entries corrected in Plan 03 — those four entries now correctly say `"S +"` etc.)

**Fix:** Change the key column value from `"Shift+R"` to `"S"`:
```rust
(
    "S",
    "STO [nn]",
    "Store X to register nn (00–99) — press S then 2 digits",
),
```

### WR-02: `Op::StoArithStack` Inside `execute_op()` Is Untested

**File:** `hp41-core/src/ops/program.rs:298`
**Issue:** The `execute_op()` arm for `Op::StoArithStack` (line 298) has zero test coverage. The sibling `Op::StoArith` arm has a test (`test_program_sto_arith` in the program_tests module), but no equivalent test exercises `Op::StoArithStack` inside a running program. If `execute_op()` were to incorrectly route the arguments, the bug would not be caught by the test suite. The existing unit tests in `registers.rs` only cover direct calls to `op_sto_arith_stack()`, not the dispatch path through `execute_op()`.

**Fix:** Add a test in `program_tests` that records `Op::StoArithStack` in a program and verifies the correct stack register is modified:
```rust
#[test]
fn test_program_sto_arith_stack() {
    use crate::ops::{StackReg, StoArithKind};
    let program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum(Decimal::from_str("10").unwrap())), // X=10
        Op::Enter,                                             // Y=10, X=10
        Op::PushNum(HpNum(Decimal::from_str("3").unwrap())),  // Y=10, X=3
        Op::StoArithStack { kind: StoArithKind::Add, stack_reg: StackReg::Y },
        // Y should now be 10+3=13, X unchanged at 3
    ];
    let mut state = state_with_program(program);
    crate::ops::program::run_program(&mut state, "A").unwrap();
    assert_eq!(state.stack.y, HpNum(Decimal::from_str("13").unwrap()));
    assert_eq!(state.stack.x, HpNum(Decimal::from_str("3").unwrap()));
}
```

### WR-03: `op_sto_arith_stack()` Test Coverage Missing for Z, T, and Mul Kind

**File:** `hp41-core/src/ops/registers.rs:105-149`
**Issue:** The three tests in `stack_arith_tests` cover: `StackReg::Y` + `Add`, `StackReg::LastX` + `Sub`, and `StackReg::Y` + `Div` (error path). `StackReg::Z`, `StackReg::T`, and `StoArithKind::Mul` are all exercised only by `execute_op()` which is itself untested (WR-02). If the `match stack_reg` arms for Z or T were accidentally swapped, no test would catch it.

**Fix:** Add at minimum two tests — one for `StackReg::Z` and one for `StackReg::T`:
```rust
#[test]
fn sto_arith_stack_mul_z() {
    let mut s = CalcState::default();
    s.stack.x = HpNum::from(d("4"));
    s.stack.z = HpNum::from(d("5"));
    op_sto_arith_stack(&mut s, StackReg::Z, StoArithKind::Mul).unwrap();
    assert_eq!(s.stack.z, HpNum::from(d("20")));
    assert_eq!(s.stack.x, HpNum::from(d("4"))); // X unchanged
}

#[test]
fn sto_arith_stack_sub_t() {
    let mut s = CalcState::default();
    s.stack.x = HpNum::from(d("2"));
    s.stack.t = HpNum::from(d("9"));
    op_sto_arith_stack(&mut s, StackReg::T, StoArithKind::Sub).unwrap();
    assert_eq!(s.stack.t, HpNum::from(d("7")));
    assert_eq!(s.stack.x, HpNum::from(d("2"))); // X unchanged
}
```

---

_Reviewed: 2026-05-08T12:28:29Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
