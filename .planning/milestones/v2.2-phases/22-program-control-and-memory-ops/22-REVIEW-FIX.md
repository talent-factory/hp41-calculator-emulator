---
phase: "22"
fixed_at: 2026-05-14
review_path: .planning/phases/22-program-control-and-memory-ops/22-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 22: Code Review Fix Report

**Fixed at:** 2026-05-14
**Source review:** `.planning/phases/22-program-control-and-memory-ops/22-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope (Critical + Warning): 1
- Fixed: 1
- Skipped: 0

The Phase 22 review identified zero Critical issues and one Warning. Info
items (4) are out of scope for `--fix --scope critical_warning`.

## Fixed Issues

### WR-01: `op_ins` panic path under corrupted-load scenario

**Files modified:** `hp41-core/src/ops/program.rs`, `hp41-core/tests/phase22_program_edit.rs`
**Commit:** `eeae825`
**Applied fix:** Defense-in-depth clamp of `state.pc` to `state.program.len()`
before `Vec::insert` in `op_ins`.

**Original issue (from REVIEW.md):**

> `hp41-core/src/ops/program.rs:208` — `op_ins` performs
> `state.program.insert(state.pc, Op::Null)`. `Vec::insert` panics if
> `index > len()`. Under normal Phase 22 control flow,
> `state.pc <= state.program.len()` always holds (verified by tracing
> CLP cursor-reposition, DEL pc-preservation, run_loop break/Gto/Xeq
> arms). However, a corrupted or malicious `~/.hp41/autosave.json` could
> deserialize `CalcState` with `pc > program.len()` — no field-level
> validation exists at load time. Invoking INS in that state would
> panic, violating the literal reading of the D-22.23 zero-panic
> invariant (`#![deny(clippy::unwrap_used)]` is necessary-but-not-sufficient
> to enforce it).

**What changed:**

`hp41-core/src/ops/program.rs::op_ins` — replaced

```rust
state.program.insert(state.pc, Op::Null);
```

with

```rust
let idx = state.pc.min(state.program.len());
state.program.insert(idx, Op::Null);
```

plus an inline doc-comment explaining the defense-in-depth rationale and
linking to D-22.23. The silent clamp mirrors the existing `op_del`
`saturating_sub` / `.min(...)` neutralization (`op_del` and `op_clp` are
already immune to a bad-pc load because they apply the same pattern).
No new `HpError` variant introduced — the clamp is silent, matching the
established sibling-op behavior.

**Regression coverage added** in `hp41-core/tests/phase22_program_edit.rs`:

1. `test_ins_at_pc_past_len_does_not_panic` — the corrupted-load
   scenario itself: `state.pc = 99` against a 2-step program. Asserts no
   panic, asserts `Op::Null` lands at the clamped index, asserts
   `state.pc` stays unchanged (same contract as the normal-path INS).
2. `test_ins_at_pc_equals_len_appends_null` — the legitimate
   "append-at-end" edge case the review noted as previously untested
   (`pc == program.len()` is in-range for `Vec::insert`, but the test
   pins it explicitly so a future refactor cannot regress).

**Verification:**

- Tier 1: re-read of the modified `op_ins` body confirms the clamp is
  present and the surrounding `prgm_mode` guard / `apply_lift_effect`
  call are intact.
- Tier 2: `cargo test -p hp41-core` — 662 passed (was 660; the two new
  regression tests bring the count to 662).
- Tier 2: `cargo clippy -p hp41-core --all-targets -- -D warnings` —
  clean, no warnings or errors.

**Plan-tag selection:** The review references `program.rs:208`, but the
`op_ins` body was authored in commit `7770446 feat(22-02): implement
op_del + op_ins bodies — DEL clamp + INS Null …` under Plan 22-02 (not
22-03 as the original suggestion in the fixer task brief). The commit
prefix therefore reads `fix(22-02): …` to keep the conventional-commit
plan attribution consistent with where the affected code was introduced.

## Skipped Issues

None. The single Warning in scope was fixed cleanly.

## Out-of-scope (Info findings, not addressed)

The 4 Info items in REVIEW.md are out of scope for this run
(`fix_scope: critical_warning`). They remain as documented optional
polish for a future cleanup pass:

- IN-01: `op_catalog` double-`format!` allocation pattern
  (`program.rs:286, 309-312`).
- IN-02: Unreachable defensive `_ => Err(InvalidOp)` arm at
  `program.rs:322`.
- IN-03: Mixed `[idx]` vs `.get().ok_or()?` idiom in
  `registers.rs`/`program.rs`/`stats.rs`.
- IN-04: Wording nudge in `registers.rs:4` doc-comment.

---

_Fixed: 2026-05-14_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
