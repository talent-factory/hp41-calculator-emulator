---
plan_id: 22-02-program-edit
phase: 22
plan: 02
type: execute
wave: 1
depends_on: [22-01-program-control]
files_modified:
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase22_program_edit.rs
autonomous: true
requirements: [FN-PROG-03, FN-PROG-04, FN-PROG-05]
must_haves:
  truths:
    - "Op::Clp(String) drains program[start..end) where start = index of Op::Lbl(name) and end = index of next Op::Lbl or program.len() (D-22.7)"
    - "After Op::Clp, state.pc is repositioned to start (clamped to program.len()) so the cursor lands at the start of the deleted block (Pitfall 4/6)"
    - "Op::Del(u8) silently clamps nnn to min(nnn, program.len() - state.pc); nnn==0 OR pc==len → no-op (D-22.9)"
    - "Op::Ins inserts Op::Null at state.pc; state.pc is UNCHANGED so the cursor still points at the newly inserted NULL (D-22.8)"
    - "All three edit ops execute ONLY when state.prgm_mode == true; prgm_mode == false → HpError::InvalidOp (D-22.10)"
    - "Op::Clp/Del/Ins join the programming-ops catch-all in execute_op (return InvalidOp inside run_loop — they are PRGM-mode editing primitives only, never recorded)"
    - "All three variants land in 4 places: Op enum + dispatch + execute_op catch-all + BOTH prgm_display.rs copies (D-22.21)"
    - "Missing CLP label (find_position returns None) → HpError::InvalidOp (D-22.7)"
  artifacts:
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Op::Clp(String), Op::Del(u8), Op::Ins variants + dispatch arms (gated on prgm_mode)"
      contains: "Op::Clp, Op::Del, Op::Ins"
    - path: "hp41-core/src/ops/program.rs"
      provides: "op_clp / op_del / op_ins helper functions + execute_op programming-ops catch-all extension"
      contains: "pub fn op_clp, pub fn op_del, pub fn op_ins, state.program.drain, state.program.insert"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "op_display_name arms for CLP name / DEL nnn / INS"
      contains: "\"CLP {name}\", \"DEL {n:03}\", \"INS\""
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same 3 display arms as CLI copy (SC-4)"
      contains: "\"CLP {name}\", \"DEL {n:03}\", \"INS\""
    - path: "hp41-core/tests/phase22_program_edit.rs"
      provides: "Integration tests covering FN-PROG-03/04/05 + Pitfall 4/6 + prgm_mode guard"
      min_lines: 90
  key_links:
    - from: "hp41-core/src/ops/program.rs::op_clp"
      to: "state.program.drain(start..end) + state.pc = start.min(state.program.len())"
      via: "find LBL, find next LBL (or end), drain, reposition cursor"
      pattern: "state.program.drain\\(start..end\\)"
    - from: "hp41-core/src/ops/program.rs::op_del"
      to: "state.program.drain(state.pc..state.pc + n) where n = clamped nnn"
      via: "saturating_sub clamp + drain"
      pattern: "saturating_sub"
    - from: "hp41-core/src/ops/program.rs::op_ins"
      to: "state.program.insert(state.pc, Op::Null)"
      via: "in-place insert; pc preserved"
      pattern: "state.program.insert\\(state.pc, Op::Null\\)"
---

<objective>
Land the three program-edit ops in `hp41-core` per D-22.7, D-22.8, D-22.9, D-22.10: `Op::Clp(String)` (clear program from named LBL to next LBL/end), `Op::Del(u8)` (delete N steps from PC with clamping), `Op::Ins` (insert Op::Null at PC). All gated on `state.prgm_mode == true` AND `state.is_running == false` — they are PRGM-mode editing primitives that mutate `state.program` directly and are NEVER recorded into the program (D-22.10).

Purpose: Without CLP/DEL/INS, users cannot edit programs interactively — they would have to rebuild from scratch on every change. CLP is the most commonly used editing op on real HP-41 hardware (clear by name and re-enter).

Output: 3 new Op variants in mod.rs + 3 dispatch arms (each prgm_mode-gated) + 3 helper functions in program.rs + extended programming-ops catch-all (Stop/Clp/Del/Ins/GtoInd/XeqInd all returning InvalidOp from execute_op) + both prgm_display.rs copies updated + integration test file `phase22_program_edit.rs`.
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

<interfaces>
<!-- Plan 22-01 just landed Op::Stop/Pse/GtoInd/XeqInd and extended the programming-ops catch-all. -->
<!-- This plan extends the same catch-all and inherits the 4-place rule pattern. -->

From hp41-core/src/state.rs (existing fields used here):
```rust
pub prgm_mode: bool,    // PRGM-mode editing flag — gates CLP/DEL/INS
pub is_running: bool,   // Program running flag — must be false for editing
pub pc: usize,          // Cursor position
pub program: Vec<Op>,   // Program buffer to mutate
```

From hp41-core/src/ops/mod.rs (existing Op variants used here):
```rust
Op::Lbl(String),   // Label marker — CLP searches for these
Op::Null,          // No-op placeholder from Phase 12 — INS inserts these
```

From hp41-core/src/ops/program.rs (Phase 12 precedent):
```rust
// :413–416 — Op::Null execute_op arm (literal no-op + Neutral lift)
Op::Null => {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

After plan 22-01, the programming-ops catch-all looks like:
```rust
Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_)
| Op::Isg(_) | Op::Dse(_) | Op::FlagTest { .. } | Op::Prompt
| Op::Stop | Op::GtoInd(_) | Op::XeqInd(_) => Err(HpError::InvalidOp),
```

This plan extends it to:
```rust
... | Op::Stop | Op::GtoInd(_) | Op::XeqInd(_)
| Op::Clp(_) | Op::Del(_) | Op::Ins => Err(HpError::InvalidOp),
```
</interfaces>
</context>

<tasks>

<task id="22-02-01" type="auto" tdd="true">
  <name>Task 22-02-01: Add Op::Clp(String), Op::Del(u8), Op::Ins variants + dispatch arms with prgm_mode guard + extend programming-ops catch-all + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/mod.rs (full file — Op enum + dispatch); Op enum was just extended in 22-01 with Stop/Pse/GtoInd/XeqInd
    - hp41-core/src/ops/program.rs lines 305–465 (execute_op) and the programming-ops catch-all (extended in 22-01)
    - hp41-cli/src/prgm_display.rs lines 28–177 (existing arms)
    - hp41-gui/src-tauri/src/prgm_display.rs lines 47–197 (mirror)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Programming-ops catch-all extension" (lines 391–426) and §"hp41-cli/src/prgm_display.rs" (lines 622–667)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.10, D-22.21
  </read_first>
  <behavior>
    - Op::Clp(String), Op::Del(u8), Op::Ins variants appended to Op enum (after Op::XeqInd from plan 22-01)
    - dispatch() arms:
      - Op::Clp(name) → if !state.prgm_mode return InvalidOp; otherwise call op_clp(state, &name) (op_clp itself re-checks prgm_mode for defense-in-depth; redundant but harmless per the helper sketch)
      - Op::Del(nnn) → if !state.prgm_mode return InvalidOp; otherwise call op_del(state, nnn)
      - Op::Ins → if !state.prgm_mode return InvalidOp; otherwise call op_ins(state)
    - Programming-ops catch-all in execute_op extended to add `| Op::Clp(_) | Op::Del(_) | Op::Ins`
    - prgm_display arms in both copies:
      - `Op::Clp(name) => format!("CLP {name}"),`
      - `Op::Del(n) => format!("DEL {n:03}"),`
      - `Op::Ins => "INS".to_string(),`
    - The op_clp/op_del/op_ins helper FUNCTIONS are added in task 22-02-02; this task only lands the variants + dispatch wiring + catch-all + display arms
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Clp(String),`, `Del(u8),`, `Ins,` to the Op enum after the plan 22-01 additions.
    2. In hp41-core/src/ops/mod.rs dispatch(): add three match arms calling the helpers (helpers don't exist yet — this task adds the call sites; helpers land in 22-02-02). Use temporary stubs that return Err(HpError::InvalidOp) if needed, but PREFER calling the helpers directly using the names `crate::ops::program::op_clp(state, &name)`, etc. — that way 22-02-02 is the only place helper bodies need to land. Re-confirm helper module path with 22-02-02. The arms are:
       - `Op::Clp(name) => crate::ops::program::op_clp(state, &name)` (op_clp internally checks prgm_mode + is_running per D-22.10)
       - `Op::Del(n) => crate::ops::program::op_del(state, n)`
       - `Op::Ins => crate::ops::program::op_ins(state)`
    3. In hp41-core/src/ops/program.rs execute_op programming-ops catch-all: extend the existing pipe-separated pattern (already extended in 22-01) with `| Op::Clp(_) | Op::Del(_) | Op::Ins`.
    4. In hp41-cli/src/prgm_display.rs op_display_name: add the 3 arms listed in <behavior>.
    5. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical 3 arms.
    6. Important: this task will NOT compile yet because op_clp/op_del/op_ins are not yet defined. Land variants + dispatch arms + prgm_display + catch-all in a "WIP" state; task 22-02-02 adds the helper bodies and the workspace compiles after that task. Alternative (recommended): add the helpers as `pub fn op_clp(state: &mut CalcState, label: &str) -> Result<(), HpError> { unimplemented!("Task 22-02-02") }` etc. so cargo check still passes — then 22-02-02 fills bodies.
    7. Choose the alternative path: write the helper functions as `unimplemented!("filled by task 22-02-02")` stubs in hp41-core/src/ops/program.rs at the bottom of the file. This way `cargo check --workspace` passes after this task.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0 (with `unimplemented!` stubs in place — the workspace compiles even though running the helpers would panic; this is intentional since 22-02-02 immediately follows)
    - `cargo clippy --workspace --all-targets -- -D warnings` may emit a warning about `unimplemented!` — accept it as ONE allowed warning OR use a more compile-clean stub `Err(HpError::InvalidOp)` instead of unimplemented! to keep clippy clean. Recommend the `Err(InvalidOp)` stub path.
    - `grep -nE '^\s*(Clp|Del|Ins)' hp41-core/src/ops/mod.rs` shows the 3 enum additions
    - `grep -nE '"CLP|"DEL|"INS"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 6 hits total (3 strings × 2 files)
    - `grep -B 0 -A 3 'Programming ops handled by run_loop' hp41-core/src/ops/program.rs` shows the pattern list now contains `Op::Clp(_) | Op::Del(_) | Op::Ins`
    - Workspace builds; helper bodies will be supplied in next task
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-02-01.log; grep -cE '"CLP |"DEL |"INS"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs; tail -1 /tmp/22-02-01.log</automated>
  </verify>
  <done>Op::Clp/Del/Ins variants land in 4 places + programming-ops catch-all extended + helper-function stubs in place; workspace compiles.</done>
</task>

<task id="22-02-02" type="auto" tdd="true">
  <name>Task 22-02-02: Implement op_clp (drain LBL..next-LBL, reposition pc to start per Pitfall 4/6, prgm_mode guard, missing-label → InvalidOp)</name>
  <files>
    hp41-core/src/ops/program.rs
  </files>
  <read_first>
    - hp41-core/src/ops/program.rs (full file — find existing op_alpha_clear-style helper functions to confirm location convention; sketch from PATTERNS.md goes adjacent to other helpers)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"op_clp" (lines 462–477)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.7, D-22.10
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §2 Pitfall 4 + Pitfall 6 + §7 verified sketch (CLP)
  </read_first>
  <behavior>
    - Replace the unimplemented!/Err stub from task 22-02-01 with the full op_clp body:
      - Guard `if !state.prgm_mode { return Err(HpError::InvalidOp); }` (defense-in-depth — dispatch already gates, but redundancy here keeps helper safe-by-itself)
      - Find start: `state.program.iter().position(|op| matches!(op, Op::Lbl(n) if n == label))` → `.ok_or(HpError::InvalidOp)?` (missing label rejects)
      - Find end: scan from `start + 1` for next Op::Lbl; if none found, end = program.len() (last labelled block)
      - Drain: `state.program.drain(start..end);`
      - Reposition cursor: `state.pc = start.min(state.program.len());` (Pitfall 4/6 — cursor lands at start of deleted block, clamped to new len)
      - Apply lift: `apply_lift_effect(state, LiftEffect::Neutral);`
      - Return Ok(())
    - Documented divergence from HP-41 hardware: real hardware uses END/.END. markers; we use next-LBL boundaries. This is documented in CONTEXT.md D-22.7 — add an inline comment referencing it.
    - Behavior on last labelled block (no following LBL): drains to end-of-Vec; pc clamps to program.len().
  </behavior>
  <action>
    1. Open hp41-core/src/ops/program.rs. Find the `pub fn op_clp` stub created in task 22-02-01.
    2. Replace its body with the full implementation per PATTERNS.md sketch (lines 464–477):
       - Step 1: guard `if !state.prgm_mode { return Err(HpError::InvalidOp); }`
       - Step 2: `let start = state.program.iter().position(|op| matches!(op, Op::Lbl(n) if n == label)).ok_or(HpError::InvalidOp)?;`
       - Step 3: `let end = state.program.iter().skip(start + 1).position(|op| matches!(op, Op::Lbl(_))).map(|i| start + 1 + i).unwrap_or(state.program.len());`
       - Step 4: `state.program.drain(start..end);`
       - Step 5: `state.pc = start.min(state.program.len());` (Pitfall 6 — cursor reposition; clamp protects against the rare case start == program.len() after drain)
       - Step 6: `apply_lift_effect(state, LiftEffect::Neutral);`
       - Step 7: `Ok(())`
    3. Add a doc-comment on the function: `/// Phase 22 D-22.7 (FN-PROG-03). Clear program from LBL "label" to next LBL or end-of-Vec. /// Documented divergence: HP-41 hardware uses END/.END. markers; we use next-LBL boundaries /// because the flat-Vec program model has no explicit END marker.`
    4. Verify no `.unwrap()` introduced (zero-panic policy D-22.23). The `.ok_or(HpError::InvalidOp)?` pattern is the correct idiom.
    5. Run `cargo check --package hp41-core` + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (clippy stays green; no .unwrap())
    - `grep -A 20 'pub fn op_clp' hp41-core/src/ops/program.rs | grep -E 'state.program.drain'` shows the drain call
    - `grep -A 20 'pub fn op_clp' hp41-core/src/ops/program.rs | grep -E 'state.pc = start'` shows the cursor reposition (Pitfall 6)
    - `grep -A 20 'pub fn op_clp' hp41-core/src/ops/program.rs | grep -E 'ok_or\(HpError::InvalidOp\)'` shows the missing-label reject pattern
    - No `.unwrap()` in op_clp body (verified by `grep -A 20 'pub fn op_clp' | grep -v '#\[' | grep -c '\.unwrap()'` returning 0)
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-02-02.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-02-02.log; awk '/pub fn op_clp/,/^}/' hp41-core/src/ops/program.rs | head -25</automated>
  </verify>
  <done>op_clp body fully implemented; clippy clean; cursor reposition + missing-label reject + drain all verifiable by grep.</done>
</task>

<task id="22-02-03" type="auto" tdd="true">
  <name>Task 22-02-03: Implement op_del (clamp nnn with saturating_sub, no-op on nnn==0 or pc==len) + op_ins (insert Op::Null, pc unchanged) — both prgm_mode-gated</name>
  <files>
    hp41-core/src/ops/program.rs
  </files>
  <read_first>
    - hp41-core/src/ops/program.rs (full file — find the op_del / op_ins stubs from task 22-02-01)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"op_del" (lines 482–493) and §"op_ins" (lines 498–504)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.8, D-22.9, D-22.10
    - hp41-core/src/ops/program.rs lines 413–416 (existing Op::Null arm — confirms Op::Null is a no-op + Neutral lift)
  </read_first>
  <behavior>
    op_del:
    - Guard `if !state.prgm_mode { return Err(HpError::InvalidOp); }`
    - Compute clamped n: `let n = (nnn as usize).min(state.program.len().saturating_sub(state.pc));`
    - If `n == 0`: apply Neutral lift, return Ok (handles both nnn==0 AND pc==len cases)
    - Otherwise: `state.program.drain(state.pc..state.pc + n);`
    - pc is UNCHANGED (next op naturally falls at the same index — which is the post-drain position)
    - Apply Neutral lift, return Ok

    op_ins:
    - Guard `if !state.prgm_mode { return Err(HpError::InvalidOp); }`
    - `state.program.insert(state.pc, Op::Null);` (insert at cursor)
    - pc is UNCHANGED — still points at the freshly inserted Null
    - Apply Neutral lift, return Ok

    Both ops:
    - Documented as PRGM-mode editing primitives that mutate state.program directly (NEVER recorded into the program — they are dispatched-and-applied, never appended)
  </behavior>
  <action>
    1. Open hp41-core/src/ops/program.rs. Find the `pub fn op_del` stub.
    2. Replace its body with the full implementation per PATTERNS.md sketch (lines 482–493):
       - Step 1: `if !state.prgm_mode { return Err(HpError::InvalidOp); }`
       - Step 2: `let n = (nnn as usize).min(state.program.len().saturating_sub(state.pc));` (D-22.9 clamping; saturating_sub avoids underflow if pc > program.len(), though that shouldn't happen)
       - Step 3: if `n == 0` → apply Neutral lift + return Ok (no-op for nnn==0 or pc==len)
       - Step 4: `state.program.drain(state.pc..state.pc + n);`
       - Step 5: pc unchanged (do NOT touch state.pc — drain shifts the trailing tail down to fill the gap)
       - Step 6: apply Neutral lift, return Ok
    3. Add doc-comment: `/// Phase 22 D-22.9 (FN-PROG-04). Delete nnn program steps starting at state.pc. /// nnn silently clamps to remaining (program.len() - pc); nnn==0 or pc==len → no-op. /// PRGM-mode only.`
    4. Find the `pub fn op_ins` stub.
    5. Replace its body per PATTERNS.md sketch (lines 498–504):
       - Step 1: `if !state.prgm_mode { return Err(HpError::InvalidOp); }`
       - Step 2: `state.program.insert(state.pc, Op::Null);`
       - Step 3: pc unchanged (do NOT modify state.pc — cursor still points at the new Null)
       - Step 4: apply Neutral lift, return Ok
    6. Add doc-comment: `/// Phase 22 D-22.8 (FN-PROG-05). Insert Op::Null (no-op placeholder, Phase 12) at state.pc. /// state.pc is preserved — cursor still points at the freshly inserted Null. PRGM-mode only.`
    7. Verify no .unwrap() anywhere (zero-panic D-22.23).
    8. Run `cargo check --workspace` + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -A 12 'pub fn op_del' hp41-core/src/ops/program.rs | grep -E 'saturating_sub'` shows the clamp pattern
    - `grep -A 12 'pub fn op_del' hp41-core/src/ops/program.rs | grep -E 'state.program.drain\(state.pc'` shows the drain call
    - `grep -A 8 'pub fn op_ins' hp41-core/src/ops/program.rs | grep -E 'state.program.insert\(state.pc, Op::Null\)'` shows the insert call
    - `grep -A 12 'pub fn op_del' hp41-core/src/ops/program.rs | grep -E 'state.pc'` finds the pc ONLY inside the drain expression `state.program.drain(state.pc..state.pc + n)` — NO `state.pc =` assignment (pc unchanged invariant)
    - `grep -A 8 'pub fn op_ins' hp41-core/src/ops/program.rs | grep -cE 'state.pc =\s'` returns 0 (pc unchanged invariant)
    - No `.unwrap()` in either function body
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-02-03.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-02-03.log; awk '/pub fn op_del/,/^}/' hp41-core/src/ops/program.rs | head -20; awk '/pub fn op_ins/,/^}/' hp41-core/src/ops/program.rs | head -15</automated>
  </verify>
  <done>op_del and op_ins bodies fully implemented; pc-unchanged invariant verified by grep; clippy clean.</done>
</task>

<task id="22-02-04" type="auto" tdd="true">
  <name>Task 22-02-04: Create hp41-core/tests/phase22_program_edit.rs covering FN-PROG-03/04/05 + Pitfall 4/6 + prgm_mode guard</name>
  <files>
    hp41-core/tests/phase22_program_edit.rs
  </files>
  <read_first>
    - hp41-core/tests/phase21_flags.rs (full file — integration test template)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §5 (Validation map — test names) + §2 Pitfall 4 + Pitfall 6
    - .planning/phases/22-program-control-and-memory-ops/22-VALIDATION.md per-task verification (lines 48–50)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Integration test files" (lines 681–769)
  </read_first>
  <behavior>
    Integration tests:
    - `test_clp_boundary` (FN-PROG-03): build program = [LBL "A", PushNum(1), LBL "B", PushNum(2), LBL "C", PushNum(3)]; set prgm_mode = true; dispatch Op::Clp("B"); assert program == [LBL "A", PushNum(1), LBL "C", PushNum(3)] (LBL B + PushNum 2 drained, LBL C preserved).
    - `test_clp_last_block_drains_to_end` (Pitfall 4): build program = [LBL "A", PushNum(1), LBL "B", PushNum(2)]; dispatch Op::Clp("B"); assert program == [LBL "A", PushNum(1)] (drained to end-of-Vec since no LBL follows).
    - `test_clp_pc_repositioned_to_start` (Pitfall 6): build program = [LBL "A", PushNum(1), LBL "B", PushNum(2), LBL "C", PushNum(3)]; set state.pc = 5; dispatch Op::Clp("B"); assert state.pc == 2 (the start index of the deleted block, which is now the index where LBL "C" used to be).
    - `test_clp_pc_clamps_to_program_len` (Pitfall 6 edge): drain the last labelled block such that start == program.len() after drain; assert state.pc clamped correctly.
    - `test_clp_missing_label_rejects` (D-22.7): dispatch Op::Clp("NONEXISTENT") on a program without that label; expects Err(HpError::InvalidOp).
    - `test_clp_prgm_mode_false_rejects` (D-22.10): set prgm_mode = false; dispatch Op::Clp("A"); expects Err(HpError::InvalidOp). Program is UNCHANGED.
    - `test_del_clamping` (FN-PROG-04): build a 5-step program; set pc = 2; dispatch Op::Del(100); assert program.len() == 2 (clamped to 5 - 2 = 3 deletions).
    - `test_del_zero_is_noop`: set up program; dispatch Op::Del(0); assert program unchanged.
    - `test_del_pc_at_end_is_noop`: state.pc = state.program.len(); dispatch Op::Del(5); assert program unchanged.
    - `test_del_prgm_mode_false_rejects` (D-22.10): prgm_mode = false; Op::Del(2) → Err(InvalidOp); program unchanged.
    - `test_ins_inserts_null_at_pc` (FN-PROG-05): build a 3-step program; set pc = 1; dispatch Op::Ins; assert program.len() == 4 AND program[1] matches Op::Null AND state.pc == 1 (UNCHANGED — pc-preservation invariant).
    - `test_ins_prgm_mode_false_rejects` (D-22.10): prgm_mode = false; Op::Ins → Err(InvalidOp); program unchanged.
    - Module header `#![allow(clippy::unwrap_used)]`.
  </behavior>
  <action>
    1. Create new file `hp41-core/tests/phase22_program_edit.rs`.
    2. Module header: `//! Integration tests for Phase 22 Plan 02 (program editing: CLP / DEL / INS).` + `#![allow(clippy::unwrap_used)]`.
    3. Imports: `use hp41_core::ops::{dispatch, Op}; use hp41_core::{CalcState, HpError, HpNum};`.
    4. Write each test listed in <behavior>. For each test: construct a fresh CalcState, populate `state.program` with the test fixture, set `state.prgm_mode = true` (or false where the test demands it), dispatch the op, assert the post-state.
    5. For test_clp_pc_repositioned_to_start: explicitly assert `state.pc == 2`. This is the Pitfall 6 sentinel — without the `state.pc = start.min(...)` fix in op_clp, pc would still be 5 (pointing past the new program length).
    6. For test_ins_inserts_null_at_pc: assert `matches!(state.program[1], Op::Null)` and `state.pc == 1`.
    7. Run `cargo test --package hp41-core --test phase22_program_edit` and verify all tests pass.
  </action>
  <acceptance_criteria>
    - `cargo test --package hp41-core --test phase22_program_edit` exits 0 with ≥12 tests passing
    - File `hp41-core/tests/phase22_program_edit.rs` exists and is ≥90 lines
    - Each of FN-PROG-03/04/05 has at least one positive test + one prgm_mode-false rejection test (6 minimum)
    - Pitfall 4 (drain-to-end-of-Vec) has explicit `test_clp_last_block_drains_to_end`
    - Pitfall 6 (pc reposition) has explicit `test_clp_pc_repositioned_to_start`
    - `just ci` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo test --package hp41-core --test phase22_program_edit 2>&1 | tee /tmp/22-02-04.log; tail -5 /tmp/22-02-04.log; wc -l hp41-core/tests/phase22_program_edit.rs; just ci 2>&1 | tail -10</automated>
  </verify>
  <done>Integration test file exists, ≥12 tests pass, all 3 FN-IDs covered with both positive + prgm_mode-false rejection cases, Pitfall 4 & 6 sentinels named explicitly.</done>
</task>

</tasks>

<verification>
- `cargo check --workspace` exits 0 after every task.
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (zero-panic policy preserved; no .unwrap()).
- `cargo test --package hp41-core --test phase22_program_edit` exits 0 with all tests passing.
- `just ci` exits 0.
- `grep -nE '"CLP|"DEL|"INS"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 6 hits (3 strings × 2 copies).
- `grep -nE 'pub fn op_(clp|del|ins)' hp41-core/src/ops/program.rs` shows 3 functions defined.
- After plan 22-02, the programming-ops catch-all in execute_op contains all the Phase 22 control/edit variants: `Op::Stop | Op::Clp(_) | Op::Del(_) | Op::Ins | Op::GtoInd(_) | Op::XeqInd(_)` (verified by grep).
</verification>

<success_criteria>
1. `Op::Clp("name")` drains from LBL to next-LBL or end-of-Vec; missing label returns InvalidOp; PC repositions to start of (now-deleted) block, clamped to program.len() (D-22.7, Pitfall 6).
2. `Op::Del(nnn)` silently clamps to remaining steps from PC; nnn==0 and pc==len are no-ops (D-22.9).
3. `Op::Ins` inserts `Op::Null` at PC; PC is UNCHANGED (D-22.8).
4. All three ops return `HpError::InvalidOp` when `state.prgm_mode == false` (D-22.10) — verified by 3 dedicated tests.
5. All three variants land in 4 places (D-22.21) — verified by compile-time exhaustive coverage and grep; the programming-ops catch-all has been extended; both prgm_display copies stay in sync.
</success_criteria>

<output>
After completion, create `.planning/phases/22-program-control-and-memory-ops/22-02-program-edit-SUMMARY.md` per the standard template.
</output>
