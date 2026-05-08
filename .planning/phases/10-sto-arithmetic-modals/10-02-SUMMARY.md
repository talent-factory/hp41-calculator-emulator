---
phase: 10-sto-arithmetic-modals
plan: 02
status: complete
completed: 2026-05-08
key-files:
  modified:
    - hp41-cli/src/app.rs
self_check: PASSED
---

# Plan 02: STO Arithmetic TUI Modal Routing

## What Was Built

Wired the TUI keyboard modal step-2 and step-3 routing in `hp41-cli/src/app.rs` so the sequence S → op-key → register/stack-letter executes STO arithmetic.

## Changes Made

**`hp41-cli/src/app.rs`:**

- Removed four `#[allow(dead_code)]` attributes from `StoAdd`/`StoSub`/`StoMul`/`StoDiv` `PendingInput` enum variants; updated comment to reflect active v1.1 modal flow
- Added `StoArithKind` and `StackReg` to `use hp41_core::ops` import
- Replaced the single-line `StoRegister` arm with a match block that intercepts `+`/`-`/`*`/`/` keys for step-2 transition to `StoAdd`/`StoSub`/`StoMul`/`StoDiv`; other keys fall through to `handle_reg_modal` for digit accumulation
- Replaced all four `StoAdd`/`StoSub`/`StoMul`/`StoDiv` arms with match blocks that intercept `Y`/`y`, `Z`/`z`, `T`/`t`, `L`/`l` for immediate `Op::StoArithStack` dispatch; `_` arm falls through to `handle_reg_modal` for 2-digit numbered register entry

## Verification

- `just ci` exits 0
- No `#[allow(dead_code)]` on StoAdd/StoSub/StoMul/StoDiv
- `grep -c "StoAdd(String::new())" hp41-cli/src/app.rs` returns 1 (step-2 routing present)
- `grep -c "StackReg::LastX" hp41-cli/src/app.rs` returns 4 (one per arm)
- 16 `Op::StoArithStack` dispatch sites (4 stack registers × 4 arithmetic kinds)
- Esc cancellation handled by existing `handle_reg_modal` implementation (unchanged)

## Deviations

None. Implementation follows Plan 02 specification exactly.

## Note

The original Plan 02 executor commits (`a306c78`, `fcb588a`) were completed correctly in the worktree but were partially lost during a merge conflict resolution with an unrelated quick-fix branch. The step-2 routing and Y/Z/T/L dispatch were re-applied manually by the orchestrator (commit after `10-REVIEW.md`). The `#[allow(dead_code)]` removal was completed at the same time.
