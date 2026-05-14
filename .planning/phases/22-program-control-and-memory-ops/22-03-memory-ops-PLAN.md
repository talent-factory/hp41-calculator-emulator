---
plan_id: 22-03-memory-ops
phase: 22
plan: 03
type: execute
wave: 1
depends_on: [22-02-program-edit]
files_modified:
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/registers.rs
  - hp41-core/src/ops/display_ops.rs
  - hp41-core/src/ops/stats.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase22_memory_ops.rs
autonomous: true
requirements: [FN-MEM-01, FN-MEM-02, FN-MEM-03, FN-MEM-04]
must_haves:
  truths:
    - "Wave-0 bounds audit (D-22.11.1): every `state.regs[i]` access in production code uses `.get(i).ok_or(HpError::InvalidOp)?` or a bounds-guarded indexed write — out-of-range register access returns InvalidOp, NEVER panic (D-22.23 zero-panic invariant)"
    - "op_sto / op_rcl / op_sto_arith / op_view replace hardcoded `reg >= 100` with `idx >= state.regs.len()` — honors current SIZE"
    - "op_clreg switches from hardcoded `vec![..; 100]` to `vec![..; state.regs.len()]` — preserves current SIZE on clear (Pitfall 5)"
    - "Σ+/Σ-/MEAN/SDEV/LR/YHAT/CORR/ClSigmaStat in stats.rs gain entry-guard `if state.regs.len() < 7 { return Err(InvalidOp); }` BEFORE accessing R01..R06 (Pitfall 5)"
    - "Op::Size(u16) per AMENDED D-22.11 / OQ-2: nnn==0 silently clamps to 1, nnn > 319 returns InvalidOp, otherwise resize(nnn, HpNum::zero())"
    - "Op::Cla per D-22.13: new variant, delegates to existing op_alpha_clear(state); displays as \"CLA\" (NOT \"CLRALPHA\" — Op::AlphaClear stays untouched for v1.0 save compat, Pitfall 8)"
    - "Op::Clst per D-22.14: zeros stack.x/y/z/t but PRESERVES state.lastx AND lift_enabled — NOT touched by Neutral lift"
    - "Op::Pack per D-22.12: no-op + Neutral lift; documented divergence from HP-41 hardware (flat-Vec model has no gaps to compact)"
    - "All four new variants land in 4 places (D-22.21): Op enum + dispatch + execute_op + BOTH prgm_display.rs copies"
    - "Pitfall 5 sentinel: Op::Size(3) followed by Op::SigmaPlus returns InvalidOp (NOT panic) — verified by integration test"
  artifacts:
    - path: "hp41-core/src/ops/registers.rs"
      provides: "Wave-0 audit: op_sto / op_rcl / op_sto_arith / op_clreg bounds-safe rewrites"
      contains: "state.regs.len(), .get(idx), .ok_or(HpError::InvalidOp)"
    - path: "hp41-core/src/ops/display_ops.rs"
      provides: "Wave-0 audit: op_view bounds-safe rewrite"
      contains: "state.regs.len(), .get(reg as usize)"
    - path: "hp41-core/src/ops/stats.rs"
      provides: "Wave-0 audit: 8 entry-guards on Σ-family functions"
      contains: "if state.regs.len() < 7"
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Op::Size(u16), Op::Cla, Op::Clst, Op::Pack variants + dispatch arms"
      contains: "Op::Size, Op::Cla, Op::Clst, Op::Pack"
    - path: "hp41-core/src/ops/program.rs"
      provides: "execute_op arms for Op::Size/Cla/Clst/Pack + helper functions op_size/op_clst (or in dedicated module)"
      contains: "pub fn op_size, pub fn op_clst, Op::Cla =>, Op::Pack =>"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "op_display_name arms for SIZE nnn / CLA / CLST / PACK"
      contains: "\"SIZE {n:03}\", \"CLA\", \"CLST\", \"PACK\""
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same 4 display arms as CLI copy (SC-4)"
      contains: "\"SIZE {n:03}\", \"CLA\", \"CLST\", \"PACK\""
    - path: "hp41-core/tests/phase22_memory_ops.rs"
      provides: "Integration tests covering FN-MEM-01/02/03/04 + Pitfall 5 sentinel + OQ-2 (SIZE 0 → 1)"
      min_lines: 100
  key_links:
    - from: "hp41-core/src/ops/registers.rs::op_sto/op_rcl/op_sto_arith"
      to: "state.regs.get(idx) instead of state.regs[idx]"
      via: "bounds-safe indexed access; reject with InvalidOp instead of panic"
      pattern: "state.regs.len\\(\\)"
    - from: "hp41-core/src/ops/stats.rs::op_sigma_plus and 7 siblings"
      to: "entry-guard `if state.regs.len() < 7 { return Err(InvalidOp); }`"
      via: "fail-closed before R01..R06 access"
      pattern: "if state.regs.len\\(\\) < 7"
    - from: "hp41-core/src/ops/program.rs::op_size"
      to: "state.regs.resize(target, HpNum::zero())"
      via: "OQ-2 clamp: nnn.max(1) for 0, error for >319"
      pattern: "state.regs.resize"
    - from: "hp41-core/src/ops/program.rs::op_clst"
      to: "stack.x/y/z/t = zero; lastx + lift_enabled PRESERVED"
      via: "explicit zero of 4 stack levels, no touch of lastx/lift"
      pattern: "state.stack.x = HpNum::zero"
---

<objective>
Land the four memory/stack-management ops in `hp41-core` per D-22.11, D-22.12, D-22.13, D-22.14 — `Op::Size(u16)`, `Op::Cla`, `Op::Clst`, `Op::Pack`. CRITICAL PREREQUISITE: complete the Wave-0 bounds audit (D-22.11.1) FIRST as three separate commits, before `Op::Size` lands — otherwise SIZE shrinking the register vector below the hardcoded 100 boundary in `op_sto`/`op_rcl`/`op_sto_arith`/`op_view`/`op_clreg` and below 7 in Σ-family operations would cause RUNTIME PANICS (Pitfall 4 + Pitfall 5), violating the project's zero-panic invariant.

Purpose: Without the bounds audit, `Op::Size(50)` followed by `RCL 75` panics — catastrophic violation of D-22.23 (`#![deny(clippy::unwrap_used)]` doesn't catch raw `Vec[i]` indexing). The audit converts ~28 production sites to `.get()` patterns and adds 8 entry-guards. Once the audit is in place, `Op::Size` becomes safe to land. `Op::Cla` provides hardware-faithful "CLA" listing without breaking v1.0 saves (Pitfall 8). `Op::Clst` clears stack while preserving LASTX + lift_enabled (D-22.14 invariant). `Op::Pack` is a documented no-op for flat-Vec model compatibility (D-22.12).

Output: 3 audit commits (registers.rs / stats.rs / op_clreg dynamic) + 4 new variants in mod.rs + 4 dispatch arms + 4 execute_op arms + op_size / op_clst helper functions + both prgm_display.rs copies updated + integration test file `phase22_memory_ops.rs` with Pitfall 5 sentinel.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md
@.planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md
@.planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md
@.planning/phases/22-program-control-and-memory-ops/22-VALIDATION.md
@CLAUDE.md
@.planning/phases/22-program-control-and-memory-ops/22-01-program-control-PLAN.md
@.planning/phases/22-program-control-and-memory-ops/22-02-program-edit-PLAN.md

<interfaces>
<!-- Pre-audit state of registers.rs (the BEFORE for tasks 22-03-01..03): -->
<!-- From hp41-core/src/ops/registers.rs (Wave-0 audit targets): -->

```rust
// :15–22 — op_sto (BEFORE)
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {                          // ← hardcoded 100, must become state.regs.len()
        return Err(HpError::InvalidOp);
    }
    state.regs[reg as usize] = state.stack.x.clone();  // ← raw index — panics if out of range under SIZE shrink
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// :26–37 — op_rcl (BEFORE) — same hardcoded-100 + raw-index pattern
// :44–58 — op_sto_arith (BEFORE) — same; 4 reads + 1 write
// :98–102 — op_clreg (BEFORE)
pub fn op_clreg(state: &mut CalcState) -> Result<(), HpError> {
    state.regs = vec![HpNum::zero(); 100];   // ← hardcoded 100, must become state.regs.len()
    ...
}
```

```rust
// hp41-core/src/ops/display_ops.rs :16–24 — op_view (BEFORE) same shape
```

```rust
// hp41-core/src/ops/stats.rs :22–48 — op_sigma_plus (BEFORE — no length guard)
pub fn op_sigma_plus(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    let new_r1 = state.regs[1].checked_add(&x.checked_sq()?)?;  // ← raw index of R01..R06
    // ... R2..R6 ...
}
// Same pattern in op_sigma_minus / op_mean / op_sdev / op_lr / op_yhat / op_corr / op_cl_sigma_stat
```

Hardware values referenced in this plan:
- SIZE range [0, 319] hardware-allowed; OQ-2 amends to 0-clamps-to-1
- Σ block = R01..R06 hardcoded (system-flag-26/27/28 redirection deferred to v3.x)
</interfaces>
</context>

<tasks>

<task id="22-03-01" type="auto" tdd="true">
  <name>Task 22-03-01: Wave-0 bounds audit COMMIT 1 — replace hardcoded `reg >= 100` + raw `state.regs[idx]` access in op_sto / op_rcl / op_sto_arith / op_view with bounds-safe `state.regs.len()` + `.get()` patterns (D-22.11.1)</name>
  <files>
    hp41-core/src/ops/registers.rs,
    hp41-core/src/ops/display_ops.rs
  </files>
  <read_first>
    - hp41-core/src/ops/registers.rs (full file — Wave-0 audit BEFORE state; ~163 lines)
    - hp41-core/src/ops/display_ops.rs lines 13–72 (op_view target — same audit shape)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §3 "Wave-0 Bounds Audit Scope" — full table of 28 production sites with line numbers + fix patterns (lines 275–322)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Bounds-audit replacement pattern" (lines 816–837) — 3 acceptable AFTER shapes
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.11.1, D-22.23
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §2 Pitfall 4 — root cause analysis
  </read_first>
  <behavior>
    Replace hardcoded `100` bound + raw indexing in 4 functions across 2 files. Audit scope from RESEARCH.md §3:
    - `op_sto` (registers.rs:15–22): write site. Pattern: `let idx = reg as usize; if idx >= state.regs.len() { return Err(InvalidOp); } state.regs[idx] = state.stack.x.clone();` (Shape 1 from PATTERNS.md).
    - `op_rcl` (registers.rs:26–37): read site. Pattern: `let val = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();` (Shape 2).
    - `op_sto_arith` (registers.rs:44–58): 4 reads + 1 write. Use Shape 1 for the write, Shape 2 for the reads — OR use a single bounds guard at the top: `let idx = reg as usize; if idx >= state.regs.len() { return Err(InvalidOp); }` then existing indexed reads/writes are safe. Recommend the entry-guard approach for atomicity (matches `Σ+` style after task 22-03-02).
    - `op_view` (display_ops.rs:16–24): read site. Use Shape 2 (`.get(reg as usize).ok_or(InvalidOp)?.clone()`).
    All 4 functions: keep their existing return-Ok / apply_lift_effect tail unchanged. The audit is mechanical — pattern-replace the bounds check, no logic change.
    Single commit (per RESEARCH.md §3 recommendation, lines 315–321 — "One commit does the bounds-audit for op_sto, op_rcl, op_sto_arith, op_view, op_clreg" — but op_clreg moves to task 22-03-03 for git-blame clarity).
  </behavior>
  <action>
    1. Open hp41-core/src/ops/registers.rs. For `op_sto` (around line 15):
       - Replace `if reg >= 100 { return Err(HpError::InvalidOp); }` with `let idx = reg as usize; if idx >= state.regs.len() { return Err(HpError::InvalidOp); }`.
       - Replace `state.regs[reg as usize] = state.stack.x.clone();` with `state.regs[idx] = state.stack.x.clone();` (bounds-checked above).
       - Add an inline comment: `// Phase 22 D-22.11.1: honor current SIZE (was hardcoded 100)`.
    2. For `op_rcl` (around line 26): use Shape 2 — `let val = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();` followed by the existing assignment to stack.x. Remove the old `if reg >= 100` guard.
    3. For `op_sto_arith` (around line 44): add entry-guard `let idx = reg as usize; if idx >= state.regs.len() { return Err(HpError::InvalidOp); }` at the top; replace existing `if reg >= 100` guard. Update all 4 reads + 1 write inside to use `state.regs[idx]` (now safe under the entry guard).
    4. For `op_view` in hp41-core/src/ops/display_ops.rs (around line 16): use Shape 2 — `let val = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();`. Remove the old `if reg >= 100` guard.
    5. Do NOT touch `op_clreg` in this task — moved to task 22-03-03 for separate-commit git-blame clarity per RESEARCH.md §3.
    6. Confirm zero new `.unwrap()` introduced.
    7. Run `cargo test --package hp41-core` to confirm existing tests stay green (the audit must be behavior-preserving for the default SIZE=100 case).
    8. Run `cargo clippy --workspace --all-targets -- -D warnings` — must stay clean.
  </action>
  <acceptance_criteria>
    - `cargo test --package hp41-core` exits 0 — existing tests (all SIZE=100) still pass; audit is behavior-preserving for the default case
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -nE 'reg >= 100' hp41-core/src/ops/registers.rs hp41-core/src/ops/display_ops.rs` returns 0 hits in these 4 functions (op_sto/rcl/sto_arith/view) — the hardcoded bound is gone
    - `grep -nE 'state.regs.len\(\)' hp41-core/src/ops/registers.rs hp41-core/src/ops/display_ops.rs` returns ≥4 hits (one per audited function)
    - No raw `state.regs[reg as usize]` without a preceding bounds check via `.len()` (verified by manual review or paired grep)
    - No `.unwrap()` introduced
  </acceptance_criteria>
  <verify>
    <automated>cargo test --package hp41-core 2>&1 | tee /tmp/22-03-01.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-03-01.log; grep -cE 'reg >= 100' hp41-core/src/ops/registers.rs; grep -cE 'state.regs.len\(\)' hp41-core/src/ops/registers.rs hp41-core/src/ops/display_ops.rs</automated>
  </verify>
  <done>op_sto/rcl/sto_arith/view all use state.regs.len() bounds; existing tests stay green; clippy clean; ready for SIZE landing.</done>
</task>

<task id="22-03-02" type="auto" tdd="true">
  <name>Task 22-03-02: Wave-0 bounds audit COMMIT 2 — add entry-guard `if state.regs.len() < 7 { return Err(InvalidOp); }` to op_sigma_plus / op_sigma_minus / op_mean / op_sdev / op_lr / op_yhat / op_corr / op_cl_sigma_stat in stats.rs (Pitfall 5)</name>
  <files>
    hp41-core/src/ops/stats.rs
  </files>
  <read_first>
    - hp41-core/src/ops/stats.rs (full file — 8 Σ-family functions; ~242+ lines)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §3 "stats.rs entry-guards" — full function-by-function table (lines 296–305)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §2 Pitfall 5 — Σ+ SIZE-shrink interaction (the entire reason for this audit)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"stats.rs entry guards" (lines 593–614)
  </read_first>
  <behavior>
    Add a single-line entry-guard at the top of each of these 8 functions:
    - `op_sigma_plus` (Σ+): reads R1..R6, writes R1..R6
    - `op_sigma_minus` (Σ-): reads R1..R6, writes R1..R6
    - `op_mean`: reads R1, R2, R3, R5 — guard needs len ≥ 6 (round up to 7 for consistency)
    - `op_sdev`: reads R1, R2, R3, R4, R5 — guard ≥ 7
    - `op_lr`: reads R1, R2, R3, R5, R6 — guard ≥ 7
    - `op_yhat`: reads R1, R2, R3, R5, R6 — guard ≥ 7
    - `op_corr`: reads R1, R2, R3, R4, R5, R6 — guard ≥ 7
    - `op_cl_sigma_stat`: writes R1..R6 — guard ≥ 7
    Use `if state.regs.len() < 7 { return Err(HpError::InvalidOp); }` as the uniform guard for all 8 (some need only ≥ 6 strictly, but ≥ 7 is conservative and matches the Σ-block convention — see PATTERNS.md line 607).
    After the guard, the existing `state.regs[1..=6]` indexing is safe (no further changes to function bodies needed).
    Single commit (RESEARCH.md §3 line 316 says "Second commit").
  </behavior>
  <action>
    1. Open hp41-core/src/ops/stats.rs.
    2. For each of the 8 functions listed in <behavior>, add as the FIRST line of the function body (immediately after the opening `{`):
       ```
       if state.regs.len() < 7 {
           return Err(HpError::InvalidOp);
       }
       ```
       Add an inline comment: `// Phase 22 D-22.11.1 / Pitfall 5: fail-closed when Σ block R01..R06 unaddressable under SIZE shrink`.
    3. Do NOT modify the function body indexing — the existing `state.regs[1]`..`state.regs[6]` is safe under the entry guard.
    4. Confirm no `.unwrap()` introduced.
    5. Run `cargo test --package hp41-core` — existing Σ-family tests must still pass (they use SIZE=100, which is ≥ 7).
    6. Run clippy — must stay clean.
  </action>
  <acceptance_criteria>
    - `cargo test --package hp41-core` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -c 'if state.regs.len() < 7' hp41-core/src/ops/stats.rs` returns ≥ 8 (one guard per Σ-family function)
    - All 8 function names (op_sigma_plus, op_sigma_minus, op_mean, op_sdev, op_lr, op_yhat, op_corr, op_cl_sigma_stat) show the guard as their first body line (manual review or grep with `grep -A 2 'pub fn op_sigma_plus' | grep -c 'regs.len() < 7'`)
    - No `.unwrap()` introduced
  </acceptance_criteria>
  <verify>
    <automated>cargo test --package hp41-core 2>&1 | tee /tmp/22-03-02.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-03-02.log; grep -c 'if state.regs.len() < 7' hp41-core/src/ops/stats.rs</automated>
  </verify>
  <done>8 Σ-family entry guards in place; existing tests stay green; clippy clean.</done>
</task>

<task id="22-03-03" type="auto" tdd="true">
  <name>Task 22-03-03: Wave-0 bounds audit COMMIT 3 — op_clreg switches from hardcoded `vec![..; 100]` to `vec![..; state.regs.len()]` so CLREG respects current SIZE (D-22.11.1)</name>
  <files>
    hp41-core/src/ops/registers.rs
  </files>
  <read_first>
    - hp41-core/src/ops/registers.rs lines 96–102 (op_clreg — full body)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §3 row "op_clreg" + bullet 3 of recommendation block (line 318)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Affected sites in registers.rs" line 586 — op_clreg case
  </read_first>
  <behavior>
    Single small change in `op_clreg`:
    - BEFORE: `state.regs = vec![HpNum::zero(); 100];`
    - AFTER: `let n = state.regs.len(); state.regs = vec![HpNum::zero(); n];`
    Rationale: after `Op::Size(50)` (task 22-03-04), if the user then dispatches CLREG, the result should be 50 zero registers — NOT silently re-grown back to 100. The audit honors the current SIZE.
  </behavior>
  <action>
    1. Open hp41-core/src/ops/registers.rs at op_clreg (around line 96).
    2. Replace the line `state.regs = vec![HpNum::zero(); 100];` with:
       ```
       // Phase 22 D-22.11.1: honor current SIZE (was hardcoded 100)
       let n = state.regs.len();
       state.regs = vec![crate::num::HpNum::zero(); n];
       ```
       (Use the qualified path `crate::num::HpNum::zero()` if it's not already imported; check existing imports in the file.)
    3. Run `cargo test --package hp41-core` — existing tests must pass (default SIZE=100 so behavior matches).
    4. Run clippy — must stay clean.
  </action>
  <acceptance_criteria>
    - `cargo test --package hp41-core` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -A 3 'pub fn op_clreg' hp41-core/src/ops/registers.rs | grep -c 'state.regs.len()'` returns ≥ 1
    - `grep -A 3 'pub fn op_clreg' hp41-core/src/ops/registers.rs | grep -c '; 100\]'` returns 0 (the hardcoded 100 is gone)
  </acceptance_criteria>
  <verify>
    <automated>cargo test --package hp41-core 2>&1 | tee /tmp/22-03-03.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-03-03.log; awk '/pub fn op_clreg/,/^}/' hp41-core/src/ops/registers.rs</automated>
  </verify>
  <done>op_clreg dynamic-sized; existing tests green; ready for SIZE landing in task 22-03-04.</done>
</task>

<task id="22-03-04" type="auto" tdd="true">
  <name>Task 22-03-04: Add Op::Size(u16) variant + dispatch + execute_op arm + both prgm_display copies; implement op_size with OQ-2 semantics (nnn==0 → 1, nnn > 319 → InvalidOp)</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/mod.rs (full file — Op enum end + dispatch)
    - hp41-core/src/ops/program.rs lines 356–363 (Op::FmtFix arm — guard-then-mutate-then-Neutral analog from PATTERNS.md)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Size(u16) arm" (lines 276–301)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.11 (AMENDED 2026-05-14 per OQ-2 Option A: SIZE 0 clamps to 1), D-22.21
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §6 OQ-2 resolution (lines 458–474)
  </read_first>
  <behavior>
    - Op::Size(u16) variant appended to Op enum
    - dispatch arm calls op_size(state, nnn)
    - op_size function body (per AMENDED D-22.11 + PATTERNS.md sketch lines 292–300):
      - If `nnn > 319 { return Err(HpError::InvalidOp); }`
      - `let target = nnn.max(1) as usize;` (OQ-2: SIZE 0 silently clamps to 1)
      - `state.regs.resize(target, HpNum::zero());` (shrink truncates tail; grow zero-fills)
      - `apply_lift_effect(state, LiftEffect::Neutral);`
      - Return Ok(())
    - execute_op arm: `Op::Size(n) => op_size(state, n)` OR inline equivalent
    - prgm_display arms in BOTH copies: `Op::Size(n) => format!("SIZE {n:03}"),`
    - This task is safe to land NOW because tasks 22-03-01..03 completed the bounds audit; shrinking via SIZE will not panic
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Size(u16),` to the Op enum (after plan 22-02's Op::Ins).
    2. In hp41-core/src/ops/mod.rs dispatch(): add `Op::Size(n) => crate::ops::program::op_size(state, n)` (or inline body).
    3. In hp41-core/src/ops/program.rs: add `pub fn op_size(state: &mut CalcState, nnn: u16) -> Result<(), HpError>` adjacent to the existing helpers added in plan 22-02. Body per PATTERNS.md sketch lines 292–300:
       ```
       pub fn op_size(state: &mut CalcState, nnn: u16) -> Result<(), HpError> {
           if nnn > 319 {
               return Err(HpError::InvalidOp);
           }
           let target = nnn.max(1) as usize;  // OQ-2: SIZE 0 → silently clamp to 1
           state.regs.resize(target, crate::num::HpNum::zero());
           crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
           Ok(())
       }
       ```
       Doc-comment: `/// Phase 22 D-22.11 / FN-MEM-01. Resize state.regs to nnn ∈ [1, 319]. /// OQ-2: nnn == 0 silently clamps to 1 (documented divergence — hardware allows SIZE 0). /// nnn > 319 returns InvalidOp.`
    4. In hp41-core/src/ops/program.rs execute_op match: add `Op::Size(n) => op_size(state, n),`.
    5. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::Size(n) => format!("SIZE {n:03}"),`.
    6. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical arm.
    7. Verify no .unwrap() introduced.
    8. Run cargo check + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -n 'pub fn op_size' hp41-core/src/ops/program.rs` shows exactly 1 hit
    - `grep -A 8 'pub fn op_size' hp41-core/src/ops/program.rs | grep -E 'nnn > 319'` shows the upper bound check
    - `grep -A 8 'pub fn op_size' hp41-core/src/ops/program.rs | grep -E 'nnn.max\(1\)'` shows the OQ-2 clamp
    - `grep -nE '"SIZE \{n:03\}"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-03-04.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-03-04.log; awk '/pub fn op_size/,/^}/' hp41-core/src/ops/program.rs | head -15</automated>
  </verify>
  <done>Op::Size(u16) lands in all 4 places with OQ-2 semantics; clippy clean; workspace compiles.</done>
</task>

<task id="22-03-05" type="auto" tdd="true">
  <name>Task 22-03-05: Add Op::Cla variant — delegates to existing op_alpha_clear (D-22.13); displays as "CLA" (Pitfall 8: Op::AlphaClear stays untouched)</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/alpha.rs lines 34–38 (existing op_alpha_clear — the delegation target)
    - hp41-core/src/ops/program.rs line 387 (existing Op::AlphaClear arm — analog for the Op::Cla arm)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Cla arm" (lines 222–245)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.13, D-22.21
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §2 Pitfall 8 (do NOT remove Op::AlphaClear — v1.0 save compat)
  </read_first>
  <behavior>
    - Op::Cla (unit variant) appended to Op enum
    - dispatch arm: `Op::Cla => crate::ops::alpha::op_alpha_clear(state)`
    - execute_op arm: same delegation
    - prgm_display in BOTH copies: `Op::Cla => "CLA".to_string(),`
    - Op::AlphaClear remains UNCHANGED — still in the enum, still displays as "CLRALPHA"
    - Add a doc-comment on Op::Cla pointing at Op::AlphaClear and explaining the dual-variant rationale (Pitfall 8)
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Cla,` to the Op enum.
    2. Add doc-comment above the variant: `/// Phase 22 D-22.13 (FN-MEM-02). Hardware-faithful CLA — clears ALPHA register. /// Delegates to op_alpha_clear (same body as Op::AlphaClear). Displays as "CLA". /// Op::AlphaClear stays in the enum for v1.0 save-file backward compat (Pitfall 8).`
    3. In hp41-core/src/ops/mod.rs dispatch(): add `Op::Cla => crate::ops::alpha::op_alpha_clear(state),`.
    4. In hp41-core/src/ops/program.rs execute_op: add `Op::Cla => crate::ops::alpha::op_alpha_clear(state),` (adjacent to the existing `Op::AlphaClear => op_alpha_clear(state),` line at :387 for code-locality).
    5. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::Cla => "CLA".to_string(),`. Verify Op::AlphaClear still emits `"CLRALPHA"` (do NOT change it).
    6. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical Op::Cla arm.
    7. Run cargo check + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -nE 'Op::Cla\b' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows ≥5 hits (1 enum + 1 dispatch + 1 execute_op + 2 prgm_display)
    - `grep -nE '"CLA"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows exactly 2 hits
    - `grep -nE 'Op::AlphaClear' hp41-core/src/ops/mod.rs` shows the variant STILL EXISTS (NOT removed — Pitfall 8)
    - `grep -nE '"CLRALPHA"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows the EXISTING 2 hits unchanged (Op::AlphaClear's display name preserved)
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-03-05.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-03-05.log; grep -c '"CLA"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs; grep -c '"CLRALPHA"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs</automated>
  </verify>
  <done>Op::Cla landed; Op::AlphaClear preserved; both display as their respective HP-41 names ("CLA" vs "CLRALPHA").</done>
</task>

<task id="22-03-06" type="auto" tdd="true">
  <name>Task 22-03-06: Add Op::Clst variant + op_clst helper (zero X/Y/Z/T; PRESERVE lastx + lift_enabled per D-22.14) + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/state.rs (Stack struct definition + lastx + lift_enabled fields)
    - hp41-core/src/stack.rs (apply_lift_effect — confirm Neutral does NOT modify lift_enabled)
    - hp41-core/src/ops/registers.rs lines 96–102 (op_clreg — whole-aggregate-zero analog)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Clst arm" (lines 247–275)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.14, D-22.21, D-22.25
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §1 D-22.14 row (HP-41 Owner's Manual ch. 7 citation)
  </read_first>
  <behavior>
    - Op::Clst (unit variant) appended to Op enum
    - dispatch arm calls op_clst(state)
    - op_clst function:
      ```
      pub fn op_clst(state: &mut CalcState) -> Result<(), HpError> {
          state.stack.x = HpNum::zero();
          state.stack.y = HpNum::zero();
          state.stack.z = HpNum::zero();
          state.stack.t = HpNum::zero();
          // lastx UNTOUCHED (D-22.14)
          // lift_enabled UNTOUCHED (Neutral lift does not modify it)
          apply_lift_effect(state, LiftEffect::Neutral);
          Ok(())
      }
      ```
    - execute_op arm: `Op::Clst => op_clst(state)`
    - prgm_display in BOTH copies: `Op::Clst => "CLST".to_string(),`
    - Doc-comment must explicitly call out LASTX + lift_enabled preservation (D-22.14 invariant — Pitfall sentinel for the test)
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Clst,` to the Op enum.
    2. In hp41-core/src/ops/mod.rs dispatch(): add `Op::Clst => crate::ops::program::op_clst(state),`.
    3. In hp41-core/src/ops/program.rs: add `pub fn op_clst(state: &mut CalcState) -> Result<(), HpError>` adjacent to the other Phase 22 helpers. Body per PATTERNS.md sketch lines 261–270.
    4. Add doc-comment: `/// Phase 22 D-22.14 (FN-MEM-03). Zero stack X/Y/Z/T. PRESERVES state.lastx AND lift_enabled. /// Verified by integration test test_clst_preserves_lastx_and_lift in tests/phase22_memory_ops.rs.`
    5. In hp41-core/src/ops/program.rs execute_op: add `Op::Clst => op_clst(state),`.
    6. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::Clst => "CLST".to_string(),`.
    7. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical arm.
    8. Confirm Neutral lift does NOT modify lift_enabled — read stack.rs apply_lift_effect to verify; if Neutral path leaves lift_enabled untouched, the implementation is correct as written. If Neutral DOES touch lift_enabled, the helper must capture the prior value and restore it — but per current code (apply_lift_effect implementation), Neutral is a no-op for lift_enabled.
    9. Run cargo check + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -n 'pub fn op_clst' hp41-core/src/ops/program.rs` shows 1 hit
    - `grep -A 10 'pub fn op_clst' hp41-core/src/ops/program.rs | grep -E 'state.stack.(x|y|z|t)\s*=' | wc -l` returns 4 (x, y, z, t all zeroed)
    - `grep -A 10 'pub fn op_clst' hp41-core/src/ops/program.rs | grep -cE 'state.lastx\s*='` returns 0 (lastx UNTOUCHED)
    - `grep -A 10 'pub fn op_clst' hp41-core/src/ops/program.rs | grep -cE 'lift_enabled\s*='` returns 0 (lift_enabled UNTOUCHED)
    - `grep -nE '"CLST"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-03-06.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-03-06.log; awk '/pub fn op_clst/,/^}/' hp41-core/src/ops/program.rs | head -15</automated>
  </verify>
  <done>Op::Clst landed; lastx and lift_enabled preservation verified by absence of assignment grep.</done>
</task>

<task id="22-03-07" type="auto" tdd="true">
  <name>Task 22-03-07: Add Op::Pack variant — documented no-op + Neutral lift (D-22.12) — flat-Vec model has no gaps to compact; both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/program.rs lines 413–416 (existing Op::Null arm — exact no-op analog)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Pack arm" (lines 303–321)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.12, D-22.21
  </read_first>
  <behavior>
    - Op::Pack (unit variant) appended to Op enum
    - dispatch arm: `Op::Pack => { apply_lift_effect(state, LiftEffect::Neutral); Ok(()) }` (inline — no helper function needed for a 2-line no-op)
    - execute_op arm: same inline body
    - prgm_display in BOTH copies: `Op::Pack => "PACK".to_string(),`
    - Doc-comment cites D-22.12 documented divergence from HP-41 hardware (real PACK compacts program memory; our flat-Vec model has no gaps)
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Pack,` to the Op enum.
    2. Add doc-comment above the variant: `/// Phase 22 D-22.12 (FN-MEM-04). Documented no-op. /// HP-41 hardware PACK compacts program memory; our flat-Vec model has no gaps to compact.`
    3. In hp41-core/src/ops/mod.rs dispatch(): add inline arm `Op::Pack => { apply_lift_effect(state, LiftEffect::Neutral); Ok(()) }`.
    4. In hp41-core/src/ops/program.rs execute_op: add `Op::Pack => { crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral); Ok(()) },` (per PATTERNS.md sketch lines 315–320).
    5. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::Pack => "PACK".to_string(),`.
    6. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical arm.
    7. Run cargo check + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -nE 'Op::Pack\b' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows ≥5 hits
    - `grep -nE '"PACK"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-03-07.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-03-07.log; grep -nE 'Op::Pack' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs</automated>
  </verify>
  <done>Op::Pack landed in all 4 places as documented no-op + Neutral lift.</done>
</task>

<task id="22-03-08" type="auto" tdd="true">
  <name>Task 22-03-08: Create hp41-core/tests/phase22_memory_ops.rs covering FN-MEM-01/02/03/04 + Pitfall 5 sentinel (Σ+ on SIZE&lt;7 → InvalidOp not panic) + OQ-2 (SIZE 0 clamps to 1)</name>
  <files>
    hp41-core/tests/phase22_memory_ops.rs
  </files>
  <read_first>
    - hp41-core/tests/phase21_flags.rs (integration test template)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §5 (Validation map) + §3 bounds-audit testing (lines 350–358) + §6 OQ-2 confirmation
    - .planning/phases/22-program-control-and-memory-ops/22-VALIDATION.md per-task verification (lines 51–55)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"phase22_memory_ops" (line 767)
  </read_first>
  <behavior>
    Integration tests:
    - `test_size_basic` (FN-MEM-01): Op::Size(50) → state.regs.len() == 50, all entries zero.
    - `test_size_zero_clamps_to_one` (OQ-2): Op::Size(0) → Ok AND state.regs.len() == 1 (NOT InvalidOp, NOT 0).
    - `test_size_over_319_rejects`: Op::Size(320) → Err(InvalidOp); state.regs unchanged.
    - `test_size_shrink_truncates_tail`: Set regs = [HpNum::from(1); 100]; Op::Size(10) → regs.len() == 10.
    - `test_size_grow_zero_fills`: Op::Size(5) then Op::Size(20) → regs[5..20] all zero.
    - `test_sto_out_of_range_after_shrink` (Pitfall 4 sentinel — bounds-audit verification): Op::Size(5) then dispatch Op::StoReg(50) → Err(InvalidOp), MUST NOT PANIC. Use `std::panic::catch_unwind` if needed; otherwise the test framework reports a panic as a failure.
    - `test_sigma_plus_on_shrunk_size_rejects` (Pitfall 5 sentinel — the critical test): Op::Size(3) then dispatch Op::SigmaPlus → Err(InvalidOp), MUST NOT PANIC.
    - `test_clreg_after_size_honors_current_size`: Op::Size(20); fill regs with values; Op::Clreg → regs.len() == 20 AND all entries zero (NOT silently re-grown to 100).
    - `test_cla_clears_alpha` (FN-MEM-02): state.alpha_reg = "HELLO"; dispatch Op::Cla → alpha_reg.is_empty().
    - `test_cla_equivalent_to_alpha_clear` (D-22.13 sentinel): dispatch Op::AlphaClear on state A; dispatch Op::Cla on state B (identical input); assert state.alpha_reg is equal — confirms Op::Cla delegates correctly.
    - `test_clst_zeros_xyzt` (FN-MEM-03): set stack.x = 1, y = 2, z = 3, t = 4; dispatch Op::Clst → all zero.
    - `test_clst_preserves_lastx_and_lift_enabled` (D-22.14 sentinel): set stack.lastx = HpNum::from(42), lift_enabled = false; dispatch Op::Clst → lastx STILL 42 AND lift_enabled STILL false.
    - `test_pack_is_noop` (FN-MEM-04): build state with some program + regs; dispatch Op::Pack → state UNCHANGED (program.len() same, regs same, stack same).
    - `test_pack_returns_ok`: dispatch Op::Pack → returns Ok(()) with no error.
    - Module header `#![allow(clippy::unwrap_used)]`.
  </behavior>
  <action>
    1. Create new file `hp41-core/tests/phase22_memory_ops.rs`.
    2. Module header `//! Integration tests for Phase 22 Plan 03 (memory ops: SIZE / CLA / CLST / PACK + bounds-audit sentinels).` + `#![allow(clippy::unwrap_used)]`.
    3. Imports: `use hp41_core::ops::{dispatch, Op}; use hp41_core::{CalcState, HpError, HpNum};`.
    4. Write each test listed in <behavior>. Focus on the two CRITICAL sentinels:
       - `test_sto_out_of_range_after_shrink` — proves Pitfall 4 / D-22.11.1 audit (no panic on out-of-range write after shrink).
       - `test_sigma_plus_on_shrunk_size_rejects` — proves Pitfall 5 (stats.rs entry guards).
    5. Use `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ...))` if needed to assert NO panic; but the standard Rust test harness automatically reports panics as failures, so a normal `assert!(matches!(result, Err(HpError::InvalidOp)))` suffices if the implementation is correct.
    6. For `test_size_zero_clamps_to_one`: assert `dispatch(&mut s, Op::Size(0)).is_ok()` AND `s.regs.len() == 1` (sentinel for OQ-2 Option A).
    7. For `test_clst_preserves_lastx_and_lift_enabled`: explicitly initialize `state.stack.lastx = HpNum::from(42i32); state.stack.lift_enabled = false;`. After dispatch, assert both fields are STILL their initial values.
    8. Run `cargo test --package hp41-core --test phase22_memory_ops` — all tests must pass.
  </action>
  <acceptance_criteria>
    - `cargo test --package hp41-core --test phase22_memory_ops` exits 0 with ≥14 tests passing
    - File `hp41-core/tests/phase22_memory_ops.rs` exists and is ≥100 lines
    - Each of FN-MEM-01/02/03/04 has at least one positive test
    - `test_sto_out_of_range_after_shrink` exists and passes (Pitfall 4 sentinel)
    - `test_sigma_plus_on_shrunk_size_rejects` exists and passes (Pitfall 5 sentinel)
    - `test_size_zero_clamps_to_one` exists and passes (OQ-2 sentinel)
    - `test_clst_preserves_lastx_and_lift_enabled` exists and passes (D-22.14 sentinel)
    - `just ci` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo test --package hp41-core --test phase22_memory_ops 2>&1 | tee /tmp/22-03-08.log; tail -5 /tmp/22-03-08.log; wc -l hp41-core/tests/phase22_memory_ops.rs; just ci 2>&1 | tail -10</automated>
  </verify>
  <done>Integration test file exists, ≥14 tests pass, all 4 FN-IDs covered, Pitfalls 4 & 5 + OQ-2 + D-22.14 each have explicit sentinel tests, just ci green.</done>
</task>

</tasks>

<verification>
- `cargo check --workspace` exits 0 after every task.
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (zero-panic invariant preserved after the bounds audit).
- `cargo test --package hp41-core --test phase22_memory_ops` exits 0 with all tests passing.
- `cargo test --package hp41-core` exits 0 (existing pre-Phase-22 tests stay green through the audit).
- `just ci` exits 0.
- Audit completeness: `grep -nE 'state.regs\[' hp41-core/src/ops/registers.rs hp41-core/src/ops/display_ops.rs` shows only bounds-checked indexed accesses (no raw indexing without a prior `.len()` guard).
- Σ-family completeness: `grep -c 'if state.regs.len() < 7' hp41-core/src/ops/stats.rs` returns ≥ 8.
- Op landing completeness: `grep -nE '"SIZE |"CLA"|"CLST"|"PACK"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 8 hits (4 strings × 2 copies).
</verification>

<success_criteria>
1. Wave-0 bounds audit completes in 3 separate commits (D-22.11.1): registers.rs+display_ops.rs (op_sto/rcl/sto_arith/view), stats.rs (8 entry-guards), registers.rs op_clreg dynamic-sized.
2. `Op::Size(u16)` resizes state.regs per AMENDED D-22.11 / OQ-2: nnn==0 → 1, nnn > 319 → InvalidOp, otherwise resize.
3. `Op::Cla` delegates to `op_alpha_clear` (D-22.13) and displays as "CLA"; `Op::AlphaClear` remains in the enum with display "CLRALPHA" (Pitfall 8 — no removal).
4. `Op::Clst` zeros stack X/Y/Z/T and PRESERVES `state.lastx` AND `state.stack.lift_enabled` (D-22.14 — verified by dedicated sentinel test).
5. `Op::Pack` is a documented no-op + Neutral lift (D-22.12).
6. Pitfall 4 + Pitfall 5 sentinels pass: after `Op::Size(5)` + `Op::StoReg(50)` returns Err(InvalidOp) NOT panic; after `Op::Size(3)` + `Op::SigmaPlus` returns Err(InvalidOp) NOT panic.
7. All four new variants land in 4 places (D-22.21); both prgm_display.rs copies stay in sync; SC-4 invariant preserved.
</success_criteria>

<output>
After completion, create `.planning/phases/22-program-control-and-memory-ops/22-03-memory-ops-SUMMARY.md` per the standard template. Note the OQ-4 acknowledgement in the SUMMARY (Op::Cla = "CLA", Op::AlphaClear = "CLRALPHA" — intentional duplication for v1.0 save-file compat).
</output>
