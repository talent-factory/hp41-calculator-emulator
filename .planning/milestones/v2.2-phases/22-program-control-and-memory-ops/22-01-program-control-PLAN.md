---
plan_id: 22-01-program-control
phase: 22
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase22_program_control.rs
autonomous: true
requirements: [FN-PROG-01, FN-PROG-02, FN-PROG-06, FN-PROG-07]
must_haves:
  truths:
    - "A program containing Op::Stop halts execution; state.pc < state.program.len() AND state.is_running == false after run_program returns (D-22.1)"
    - "resume_program(state) re-enters run_loop from state.pc and resets is_running on BOTH Ok and Err paths (D-22.2, Pitfall 2)"
    - "Op::Pse writes state.display_override = format_hpnum(X) AND pushes 'PAUSE 1000' into state.event_buffer; run_loop continues without breaking (D-22.4, Pitfall 3)"
    - "Op::GtoInd(reg) reads state.regs[reg].trunc_int() with non-integer reject and resolves to label via find_in_program (D-22.15)"
    - "Op::XeqInd(reg) does pre-mutation 4-deep call_stack check (HpError::CallDepth), then same indirect resolution as GtoInd (D-22.15)"
    - "All four new variants Stop/Pse/GtoInd/XeqInd land in 4 places: Op enum + dispatch + execute_op (or catch-all) + BOTH prgm_display.rs copies (D-22.21)"
    - "Op::Stop arm in run_loop writes NOTHING to display_override — unlike Op::Prompt (Pitfall 1)"
    - "Programming-ops catch-all in execute_op gains Op::Stop, Op::GtoInd(_), Op::XeqInd(_); Op::Pse does NOT join (it runs in execute_op directly per D-22.5)"
  artifacts:
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Op::Stop, Op::Pse, Op::GtoInd(u8), Op::XeqInd(u8) variants + dispatch arms"
      contains: "Op::Stop, Op::Pse, Op::GtoInd, Op::XeqInd"
    - path: "hp41-core/src/ops/program.rs"
      provides: "resume_program() public fn + run_loop arms for Op::Stop/GtoInd/XeqInd + execute_op Op::Pse arm + extended programming-ops catch-all"
      contains: "pub fn resume_program, Op::Stop => break, Op::GtoInd(reg), Op::XeqInd(reg), Op::Pse =>"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "op_display_name arms for STOP / PSE / GTO IND nn / XEQ IND nn"
      contains: "\"STOP\", \"PSE\", \"GTO IND"
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "Same 4 display arms as CLI copy (4-place rule under SC-4)"
      contains: "\"STOP\", \"PSE\", \"GTO IND"
    - path: "hp41-core/tests/phase22_program_control.rs"
      provides: "Integration tests covering FN-PROG-01/02/06/07 + Pitfall 1/2/3 sentinels"
      min_lines: 80
  key_links:
    - from: "hp41-core/src/ops/program.rs::resume_program"
      to: "run_loop"
      via: "let result = run_loop(state, &program); state.is_running = false; result"
      pattern: "let result = run_loop"
    - from: "hp41-core/src/ops/program.rs run_loop Op::GtoInd arm"
      to: "find_in_program + state.regs.get + HpNum::trunc_int"
      via: "indirect-pointer resolution with non-integer reject"
      pattern: "state.regs.get\\(reg as usize\\).ok_or\\(HpError::InvalidOp\\)"
    - from: "hp41-core/src/ops/program.rs execute_op Op::Pse arm"
      to: "state.display_override + state.event_buffer"
      via: "two-channel write (Phase 21 BEEP pattern + Phase 21 AVIEW pattern)"
      pattern: "PAUSE 1000"
---

<objective>
Land the four interpreter-control-flow ops in `hp41-core` per D-22.1..D-22.6 + D-22.15: `Op::Stop` (break run_loop, no display write), `Op::Pse` (write display_override + event_buffer "PAUSE 1000", continue), `Op::GtoInd(u8)` and `Op::XeqInd(u8)` (inline indirect resolver — Phase 24 generalizes), plus the new `pub fn resume_program(state)` public entry point in `ops/program.rs`. All four variants are Neutral lift. Phase 22's only changes to interpreter machinery — Plan 22-02 will pile on top for program editing.

Purpose: Without STOP/PSE and resume_program, programs cannot pause/yield to user; without GTO/XEQ IND, indirect branching is impossible. The four ops unlock the entire programming-control surface that Phases 25 (CLI keyboard wiring) and 26 (GUI key_map) need to wire R/S to.

Output: 4 new Op variants in mod.rs + 4 dispatch arms + 3 run_loop arms (Stop, GtoInd, XeqInd) + 1 execute_op arm (Pse) + extended programming-ops catch-all + resume_program() function + both prgm_display.rs copies updated + integration test file `phase22_program_control.rs`.
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

<interfaces>
<!-- Extracted from existing source. Executor uses these directly, no exploration needed. -->

From hp41-core/src/ops/program.rs (existing entry points to mirror):
```rust
// :139–169 — run_program template for resume_program
pub fn run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError>;

// :201–204 — Op::Gto run_loop arm (GtoInd analog)
Op::Gto(label) => { let target = find_in_program(program, &label)?; state.pc = target + 1; }

// :205–230 — Op::Xeq run_loop arm (XeqInd analog) — pre-mutation 4-deep guard
Op::Xeq(label) => { if state.call_stack.len() >= 4 { return Err(HpError::CallDepth); } ... }

// :272–275 — Op::Prompt run_loop arm (Stop analog — but Stop writes NOTHING)
Op::Prompt => { state.display_override = Some(...); break; }

// :454–464 — programming-ops catch-all in execute_op (extend with Stop/GtoInd/XeqInd)
Op::Lbl(_) | Op::Gto(_) | Op::Xeq(_) | Op::Rtn | Op::PrgmMode | Op::Test(_)
| Op::Isg(_) | Op::Dse(_) | Op::FlagTest { .. } | Op::Prompt => Err(HpError::InvalidOp),

// :537–541 — helper for label search after stringifying pointer
pub fn find_in_program(program: &[Op], label: &str) -> Result<usize, HpError>;
```

From hp41-core/src/num.rs (used by indirect resolver):
```rust
// :213–227 — integer-part extraction
impl HpNum {
    pub fn trunc_int(&self) -> HpNum;  // discard fractional part
    pub fn inner(&self) -> &Decimal;   // for .to_string() label
}
```

From hp41-core/src/error.rs:
```rust
pub enum HpError {
    InvalidOp,    // most error paths (out-of-range reg, non-integer, label miss)
    CallDepth,   // XeqInd 4-deep guard ONLY (no new variants — D-22.23)
    // ... existing variants ...
}
```

From hp41-core/src/stack.rs:
```rust
pub fn apply_lift_effect(state: &mut CalcState, effect: LiftEffect);
pub enum LiftEffect { Enable, Disable, Neutral }
// All Phase 22 ops use LiftEffect::Neutral (D-22.25)
```

From hp41-core/src/format.rs:
```rust
pub fn format_hpnum(n: &HpNum, mode: &DisplayMode) -> String;
// Used by Op::Pse to format X for display_override
```

Existing struct fields on CalcState used here (state.rs:80–134):
- `is_running: bool`
- `pc: usize`
- `program: Vec<Op>`
- `call_stack: Vec<usize>`  (4-deep cap)
- `regs: Vec<HpNum>`
- `stack: Stack`
- `display_override: Option<String>` (Phase 21, #[serde(default)])
- `event_buffer: Vec<String>` (Phase 21, #[serde(skip)])
- `display_mode: DisplayMode`
</interfaces>
</context>

<tasks>

<task id="22-01-01" type="auto" tdd="true">
  <name>Task 22-01-01: Add Op::Stop variant + dispatch + run_loop break + programming-ops catch-all + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/mod.rs (full file — see existing Op enum end, dispatch() body; Phase 21 SfFlag/CfFlag/View/Beep/Tone arms at :570–589 are the precedent style)
    - hp41-core/src/ops/program.rs lines 177–285 (run_loop body) and 305–465 (execute_op) and 454–464 (programming-ops catch-all)
    - hp41-core/src/ops/program.rs lines 272–275 (Op::Prompt arm — this is the structural analog but Stop must NOT write display_override per Pitfall 1)
    - hp41-cli/src/prgm_display.rs lines 28–177 (existing op_display_name match arms)
    - hp41-gui/src-tauri/src/prgm_display.rs lines 47–197 (mirror copy — exact same arms required per SC-4)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Stop arm" (PATTERNS.md lines 95–117)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.1, D-22.5, D-22.21
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §2 Pitfall 1 + Pitfall 7
  </read_first>
  <behavior>
    - Op::Stop variant appended at END of Op enum in ops/mod.rs (preserves discriminant order, D-22.22)
    - dispatch() arm for Op::Stop: interactive (state.is_running == false) → Ok + Neutral lift (no-op per D-22.5)
    - run_loop match arm: `Op::Stop => break,` placed ADJACENT to Op::Prompt with comment "// Unlike Op::Prompt, Stop writes nothing to display_override"
    - execute_op programming-ops catch-all (:454–464) extends to include `Op::Stop` (returns InvalidOp — handled by run_loop directly)
    - hp41-cli/src/prgm_display.rs gains `Op::Stop => "STOP".to_string(),`
    - hp41-gui/src-tauri/src/prgm_display.rs gains the SAME `Op::Stop => "STOP".to_string(),` arm (SC-4: intentional duplication)
    - After STOP in a program: state.pc < state.program.len() (pc was advanced before the match per program.rs:189), state.is_running == false on return (run_program's safety reset)
    - display_override is NOT modified by Op::Stop (Pitfall 1 sentinel — the previous step's display persists)
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Stop,` to the end of the Op enum (after the last existing variant; per D-22.22 preserve discriminant order).
    2. In hp41-core/src/ops/mod.rs dispatch(): add match arm `Op::Stop => { apply_lift_effect(state, LiftEffect::Neutral); Ok(()) }` (per D-22.5, interactive Op::Stop is a Neutral no-op).
    3. In hp41-core/src/ops/program.rs run_loop match (find the existing Op::Prompt arm around line 272): insert IMMEDIATELY ABOVE it `Op::Stop => break,` with a leading comment line `// Phase 22 D-22.1 / Pitfall 1: STOP breaks run_loop only — NO display_override write (unlike Op::Prompt below). The previous step's display persists.`
    4. In hp41-core/src/ops/program.rs execute_op programming-ops catch-all (currently lines 454–464): add `| Op::Stop` to the existing pipe-separated pattern list that returns `Err(HpError::InvalidOp)`.
    5. In hp41-cli/src/prgm_display.rs op_display_name match: add `Op::Stop => "STOP".to_string(),` (use the same arm-style as existing variants — see Op::Sin at the head of the file).
    6. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name match: add the identical arm `Op::Stop => "STOP".to_string(),` (SC-4 invariant: the two prgm_display copies stay structurally synchronised).
    7. Run `cargo check --workspace` to verify exhaustive-match coverage compiles. The compiler will fail-closed if any of the 4 landing sites is missed (4-place rule, D-22.21).
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0 (exhaustive matches all happy)
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (zero-panic policy maintained — no .unwrap() introduced)
    - `grep -n '"STOP"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits (one per copy)
    - `grep -nE '^\s*Op::Stop\b' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs` shows: 1 Op enum addition + 1 dispatch arm + 1 run_loop break arm + 1 catch-all extension (4 production hits total minimum)
    - Test added in task 22-01-05 (`test_stop_then_resume`) goes RED-green on this step — depends on resume_program landing next
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-01-01.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-01-01.log; tail -1 /tmp/22-01-01.log</automated>
  </verify>
  <done>Op::Stop lands in all 4 places (enum + dispatch + run_loop + 2× prgm_display) + programming-ops catch-all extended; workspace compiles + clippy clean.</done>
</task>

<task id="22-01-02" type="auto" tdd="true">
  <name>Task 22-01-02: Add Op::Pse variant + dispatch + execute_op arm (display_override + event_buffer "PAUSE 1000") + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/sound.rs lines 1–30 (full file — op_beep / op_tone are the event_buffer push analog)
    - hp41-core/src/ops/display_ops.rs lines 28–33 (op_aview — display_override = Some(...) analog)
    - hp41-core/src/ops/program.rs lines 305–465 (execute_op body — Op::Pse arm goes near Op::Beep around :452)
    - hp41-core/src/ops/program.rs lines 405–423 (dispatch top — verify flush_entry_buf is already called per Pitfall 10; do NOT add a second flush in op_pse)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::Pse arm" (PATTERNS.md lines 187–220)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.4, D-22.21, D-22.25
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §2 Pitfall 3 + Pitfall 10
  </read_first>
  <behavior>
    - Op::Pse variant appended to Op enum
    - dispatch() arm calls execute_op via the existing pass-through OR delegates to a new `op_pse(state)` helper
    - execute_op match arm writes BOTH: `state.display_override = Some(format_hpnum(&state.stack.x, &state.display_mode))` AND `state.event_buffer.push("PAUSE 1000".to_string())`
    - Op::Pse does NOT join programming-ops catch-all (it runs mid-program — that's the point of PSE)
    - Op::Pse is Neutral lift (D-22.25)
    - run_loop does NOT break on Pse — execution continues to the next step
    - prgm_display arms in BOTH copies emit `"PSE"`
    - Pitfall 3: display_override survives subsequent run_loop iterations (run_loop calls execute_op directly, not dispatch — no dispatch-top clear between iterations). Test asserts: after Op::Pse → step → step, display_override still Some(...). After program exits, next dispatch() clears it.
    - Pitfall 10: no second flush_entry_buf inside op_pse (dispatch already called it). Test asserts entry_buf is flushed once for interactive PSE (e.g., type 1.23, hit PSE, display_override = "1.2300" in FIX 4).
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `Pse,` to the Op enum (after `Stop,` from task 22-01-01).
    2. In hp41-core/src/ops/mod.rs dispatch(): add match arm that calls `crate::ops::program::execute_op` for Op::Pse, OR (recommended for grep affinity) add inline `Op::Pse => { let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode); state.display_override = Some(formatted); state.event_buffer.push("PAUSE 1000".to_string()); apply_lift_effect(state, LiftEffect::Neutral); Ok(()) }`. Choose the inline path per the PATTERNS.md sketch at lines 207–215.
    3. In hp41-core/src/ops/program.rs execute_op: add match arm `Op::Pse => { let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode); state.display_override = Some(formatted); state.event_buffer.push("PAUSE 1000".to_string()); crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral); Ok(()) }` (PATTERNS.md sketch lines 207–215). Place adjacent to the Phase 21 Op::Beep / Op::AView arms for code-locality.
    4. Confirm Op::Pse is NOT in the programming-ops catch-all (per D-22.5 and Pitfall 7 — Pse executes in execute_op, the only Phase 22 op that does).
    5. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::Pse => "PSE".to_string(),`.
    6. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add identical `Op::Pse => "PSE".to_string(),`.
    7. Do NOT add a `flush_entry_buf` call inside op_pse (Pitfall 10 — dispatch already calls it at the top).
    8. Run `cargo check --workspace` + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -n '"PSE"' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits
    - `grep -nE 'PAUSE 1000' hp41-core/src/ops/` shows the event_buffer string (in mod.rs OR program.rs)
    - `grep -nE 'Op::Pse' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs` shows at least 2 hits (enum + 1 arm, with optional duplicate in dispatch vs execute_op)
    - No `flush_entry_buf` inside the Op::Pse handler body (verified by reading the arm; Pitfall 10)
    - Op::Pse NOT present in the programming-ops catch-all (verified by `grep -A 20 'Programming ops handled by run_loop' hp41-core/src/ops/program.rs | grep -v 'Op::Pse'` returning the catch-all without Pse)
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-01-02.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-01-02.log; tail -1 /tmp/22-01-02.log</automated>
  </verify>
  <done>Op::Pse lands in all 4 places; execute_op writes both display_override and event_buffer "PAUSE 1000"; Op::Pse is NOT in the programming-ops catch-all; workspace compiles clean.</done>
</task>

<task id="22-01-03" type="auto" tdd="true">
  <name>Task 22-01-03: Add pub fn resume_program(state) in ops/program.rs (mirror run_program — reset is_running on Err path per Pitfall 2)</name>
  <files>
    hp41-core/src/ops/program.rs
  </files>
  <read_first>
    - hp41-core/src/ops/program.rs lines 139–169 (run_program — EXACT template; resume_program copies this minus the entry-label search)
    - hp41-core/src/ops/program.rs lines 177–285 (run_loop signature: `fn run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError>`)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"resume_program()" (PATTERNS.md lines 34–88)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.2, D-22.6
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §2 Pitfall 2 + §7 verified sketch (RESEARCH.md lines 513–530)
  </read_first>
  <behavior>
    - New public function `pub fn resume_program(state: &mut CalcState) -> Result<(), HpError>` in `hp41-core/src/ops/program.rs`
    - Body:
      - `if state.pc >= state.program.len() { return Err(HpError::InvalidOp); }` (nothing to resume)
      - clone program: `let program = state.program.clone();`
      - `state.is_running = true;`
      - `let result = run_loop(state, &program);`
      - `state.is_running = false;` — ALWAYS runs, even on Err (Pitfall 2 — do NOT use `?` propagation)
      - `result`
    - Does NOT clear state.call_stack (unlike run_program — resume preserves pending XEQ frames so RTN behaves correctly per PATTERNS.md line 87)
    - Located adjacent to `run_program()` in the same file
  </behavior>
  <action>
    1. Open hp41-core/src/ops/program.rs. Find `pub fn run_program(...)` at line 139.
    2. Immediately AFTER the closing `}` of run_program (around line 169), insert the new function with a doc-comment referencing D-22.2 and FN-PROG-01:
       - Function signature `pub fn resume_program(state: &mut CalcState) -> Result<(), HpError>`.
       - Doc-comment lines naming: "Mirror of run_program but skips entry-label search; state.pc is the resume point. Resets is_running on BOTH Ok and Err paths (Pitfall 2)."
       - Body matches PATTERNS.md sketch lines 73–82 verbatim (using `let result = run_loop(state, &program); state.is_running = false; result` — NOT `?`).
    3. Do NOT use `?` to propagate run_loop's error. The let-result-then-reset pattern is mandatory per Pitfall 2.
    4. Do NOT call `state.call_stack.clear()` here — resume must preserve any pending XEQ frames (unlike run_program which clears it for fresh entry).
    5. Run `cargo check --package hp41-core` + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --package hp41-core` exits 0
    - `cargo clippy --package hp41-core --all-targets -- -D warnings` exits 0
    - `grep -n 'pub fn resume_program' hp41-core/src/ops/program.rs` shows exactly 1 hit
    - `grep -nB2 -A8 'pub fn resume_program' hp41-core/src/ops/program.rs` shows the body using `let result = run_loop(...)` NOT `run_loop(...)?` (verified by absence of `?` on the run_loop call)
    - `grep -n 'state.call_stack.clear' hp41-core/src/ops/program.rs` shows 1 hit (the existing one inside run_program), NOT 2 — resume_program must not clear the call stack
  </acceptance_criteria>
  <verify>
    <automated>cargo check --package hp41-core 2>&1 | tee /tmp/22-01-03.log; cargo clippy --package hp41-core --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-01-03.log; grep -c 'pub fn resume_program' hp41-core/src/ops/program.rs; grep -c 'state.call_stack.clear' hp41-core/src/ops/program.rs</automated>
  </verify>
  <done>pub fn resume_program landed adjacent to run_program; let-result pattern (not `?`) preserves is_running reset on Err; no call_stack.clear() added.</done>
</task>

<task id="22-01-04" type="auto" tdd="true">
  <name>Task 22-01-04: Add Op::GtoInd(u8) variant + dispatch + run_loop inline indirect-resolver arm + programming-ops catch-all + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/program.rs lines 201–204 (Op::Gto run_loop arm — direct analog)
    - hp41-core/src/ops/program.rs lines 537–541 (find_in_program helper)
    - hp41-core/src/num.rs lines 213–227 (HpNum::trunc_int + .inner() method)
    - hp41-core/src/state.rs lines 80–134 (regs: Vec<HpNum> field)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::GtoInd(reg) arm" (PATTERNS.md lines 119–140)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.15, D-22.21
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §7 verified sketch (lines 547–556)
  </read_first>
  <behavior>
    - Op::GtoInd(u8) variant appended to Op enum
    - dispatch() arm returns InvalidOp for interactive Op::GtoInd (it's a programming-only op per the catch-all; dispatching directly outside a program is undefined) — OR delegates to execute_op's catch-all which returns InvalidOp. Choose the latter for simplicity.
    - run_loop match arm for Op::GtoInd(reg):
      1. `let pointer = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();` (out-of-range register → InvalidOp, no panic)
      2. `let int_part = pointer.trunc_int();`
      3. `if int_part != pointer { return Err(HpError::InvalidOp); }` (non-integer pointer reject, FN-IND-02 semantics)
      4. `let label_str = int_part.inner().to_string();` (stringify integer for label search)
      5. `let target = find_in_program(program, &label_str)?;`
      6. `state.pc = target + 1;` (mirror Op::Gto)
    - execute_op programming-ops catch-all extended to include Op::GtoInd(_) (returns InvalidOp — handled by run_loop directly per Pitfall 7)
    - prgm_display in both copies emits `"GTO IND nn"` where nn is the 2-digit register (e.g., `format!("GTO IND {r:02}")`)
    - LiftEffect: Neutral (D-22.25) — but note run_loop arms don't call apply_lift_effect directly (the loop's outer dispatch path handles lift). Verify by checking how Op::Gto's arm is structured.
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `GtoInd(u8),` to the Op enum.
    2. In hp41-core/src/ops/mod.rs dispatch(): add `Op::GtoInd(_) => Err(HpError::InvalidOp)` — interactive indirect branching outside a program is invalid (it requires the run_loop state machine).
    3. In hp41-core/src/ops/program.rs run_loop match (near Op::Gto at line 201): insert the new arm per the PATTERNS.md sketch at lines 130–139. Use the EXACT 6-step structure listed in <behavior> above. Place adjacent to Op::Gto for code-locality.
    4. In hp41-core/src/ops/program.rs execute_op programming-ops catch-all: extend the pipe-separated pattern list to add `| Op::GtoInd(_)`.
    5. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::GtoInd(r) => format!("GTO IND {r:02}"),`.
    6. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical arm.
    7. Run `cargo check --workspace` + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -nE 'Op::GtoInd' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows ≥5 hits (1 enum + 1 dispatch + 1 run_loop + 1 catch-all + 2 prgm_display)
    - `grep -nE '"GTO IND' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits
    - The run_loop Op::GtoInd arm uses `.get(reg as usize).ok_or(HpError::InvalidOp)?` (not raw `state.regs[reg as usize]`) — verified by `grep -A 8 'Op::GtoInd' hp41-core/src/ops/program.rs | grep -E 'state\.regs\.get'`
    - The non-integer reject uses `if int_part != pointer { return Err(HpError::InvalidOp); }` — verified by grep
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-01-04.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-01-04.log; grep -A 8 'Op::GtoInd(reg)' hp41-core/src/ops/program.rs | grep -E 'state\.regs\.get' | grep -c .</automated>
  </verify>
  <done>Op::GtoInd lands in all 4 places + run_loop arm uses bounds-safe `.get()` + non-integer reject; workspace compiles clean.</done>
</task>

<task id="22-01-05" type="auto" tdd="true">
  <name>Task 22-01-05: Add Op::XeqInd(u8) variant + dispatch + run_loop arm with pre-mutation 4-deep call_stack guard + programming-ops catch-all + both prgm_display copies</name>
  <files>
    hp41-core/src/ops/mod.rs,
    hp41-core/src/ops/program.rs,
    hp41-cli/src/prgm_display.rs,
    hp41-gui/src-tauri/src/prgm_display.rs
  </files>
  <read_first>
    - hp41-core/src/ops/program.rs lines 205–230 (Op::Xeq run_loop arm — analog; pre-mutation `state.call_stack.len() >= 4` check at :206–207)
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Op::XeqInd(reg) arm" (PATTERNS.md lines 142–183)
    - .planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md D-22.15, D-22.21
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §7 verified sketch (lines 557–568)
  </read_first>
  <behavior>
    - Op::XeqInd(u8) variant appended to Op enum
    - dispatch() arm returns InvalidOp (same rationale as GtoInd)
    - run_loop arm:
      1. `if state.call_stack.len() >= 4 { return Err(HpError::CallDepth); }` (PRE-mutation check — error before any state change per Op::Xeq precedent)
      2. `let pointer = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone();`
      3. `let int_part = pointer.trunc_int();`
      4. `if int_part != pointer { return Err(HpError::InvalidOp); }`
      5. `let label_str = int_part.inner().to_string();`
      6. `let target = find_in_program(program, &label_str)?;`
      7. `state.call_stack.push(state.pc); state.pc = target + 1;`
    - execute_op programming-ops catch-all extends with Op::XeqInd(_)
    - prgm_display in both copies emits `"XEQ IND nn"`
    - Unlike Op::Xeq, indirect form does NOT have the builtin_card_op fallback (label is numeric string only — see PATTERNS.md line 182)
  </behavior>
  <action>
    1. In hp41-core/src/ops/mod.rs: append `XeqInd(u8),` to the Op enum.
    2. In hp41-core/src/ops/mod.rs dispatch(): add `Op::XeqInd(_) => Err(HpError::InvalidOp)`.
    3. In hp41-core/src/ops/program.rs run_loop match (near Op::Xeq at line 205): insert the new arm per the PATTERNS.md sketch at lines 167–178. CRITICAL: the `state.call_stack.len() >= 4` check must come BEFORE the `state.regs.get` call (pre-mutation atomicity — matches Op::Xeq's precedent at line 206).
    4. In hp41-core/src/ops/program.rs execute_op programming-ops catch-all: extend the pipe-separated pattern list to add `| Op::XeqInd(_)`.
    5. In hp41-cli/src/prgm_display.rs op_display_name: add `Op::XeqInd(r) => format!("XEQ IND {r:02}"),`.
    6. In hp41-gui/src-tauri/src/prgm_display.rs op_display_name: add the identical arm.
    7. Do NOT add a builtin_card_op fallback in the indirect arm — the label is a numeric string, not a function name.
    8. Run `cargo check --workspace` + clippy.
  </action>
  <acceptance_criteria>
    - `cargo check --workspace` exits 0
    - `cargo clippy --workspace --all-targets -- -D warnings` exits 0
    - `grep -nE 'Op::XeqInd' hp41-core/src/ops/mod.rs hp41-core/src/ops/program.rs hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows ≥5 hits
    - `grep -nE '"XEQ IND' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 2 hits
    - In hp41-core/src/ops/program.rs, the Op::XeqInd run_loop arm contains `HpError::CallDepth` BEFORE `state.regs.get` (line-order check via grep)
    - `grep -A 10 'Op::XeqInd(reg)' hp41-core/src/ops/program.rs | grep -c 'CallDepth'` returns ≥1
  </acceptance_criteria>
  <verify>
    <automated>cargo check --workspace 2>&1 | tee /tmp/22-01-05.log; cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a /tmp/22-01-05.log; awk '/Op::XeqInd\(reg\)/,/^\s*\}/' hp41-core/src/ops/program.rs | head -20</automated>
  </verify>
  <done>Op::XeqInd lands in all 4 places; run_loop arm has pre-mutation 4-deep guard returning CallDepth; workspace compiles clean.</done>
</task>

<task id="22-01-06" type="auto" tdd="true">
  <name>Task 22-01-06: Create hp41-core/tests/phase22_program_control.rs covering FN-PROG-01/02/06/07 with all RESEARCH §2 pitfall sentinels</name>
  <files>
    hp41-core/tests/phase22_program_control.rs
  </files>
  <read_first>
    - hp41-core/tests/phase21_sound.rs (full file — exact integration-test template per PATTERNS.md analog)
    - hp41-core/tests/phase21_flags.rs lines 39–46 (serde round-trip pattern, but not needed in this plan — no new fields)
    - .planning/phases/22-program-control-and-memory-ops/22-RESEARCH.md §5 (Validation map — per-FN-ID test names) + §2 Pitfalls 1, 2, 3
    - .planning/phases/22-program-control-and-memory-ops/22-PATTERNS.md §"Integration test files" (lines 681–769)
    - .planning/phases/22-program-control-and-memory-ops/22-VALIDATION.md per-task verification map (lines 44–47)
  </read_first>
  <behavior>
    Integration test file `hp41-core/tests/phase22_program_control.rs` with the following tests:
    - `test_stop_then_resume` (FN-PROG-01): build a program with LBL A → PushNum(42) → Stop → PushNum(99) → end; call run_program("A"), assert X == 42 AND is_running == false AND pc < program.len() (Pitfall 1 + D-22.1 invariant). Then call resume_program(state), assert X == 99 AND is_running == false.
    - `test_stop_does_not_write_display_override` (Pitfall 1 sentinel): build a program where display_override is None on entry; run a STOP; assert display_override is STILL None (the prior step did not write it, so STOP must not either).
    - `test_resume_resets_is_running_on_err` (Pitfall 2 sentinel): build a program that will Err during run_loop (e.g., XEQ to non-existent label), call resume_program — must return Err but state.is_running == false on return.
    - `test_pse_writes_both_channels` (FN-PROG-02): build a program with PushNum(1.23) → Pse → end; run; assert state.display_override == Some(format_hpnum(1.23, FIX 4)) AND state.event_buffer contains "PAUSE 1000".
    - `test_pse_display_override_survives_next_program_step` (Pitfall 3): program with PushNum(1.23) → Pse → PushNum(5) → end; run; assert display_override is STILL Some(format-of-1.23) after the program returns (run_loop does NOT route through dispatch).
    - `test_pse_display_override_cleared_by_next_dispatch` (Pitfall 3): after the above, call dispatch(state, Op::Add) (or any op); assert display_override == None (dispatch-top clear from Phase 21).
    - `test_gto_ind_happy` (FN-PROG-06): build a program with LBL "42" → PushNum(7) → end; set state.regs[5] = HpNum::from(42); main program: GtoInd(5); run; assert state.pc lands past the LBL.
    - `test_gto_ind_non_integer_rejects` (FN-IND-02 partial — for FN-PROG-06): set state.regs[5] = HpNum::from(12.345); GtoInd(5) → expects Err(HpError::InvalidOp).
    - `test_gto_ind_reg_out_of_range_rejects`: state.regs.len() == 5; GtoInd(10) → expects Err(HpError::InvalidOp), NOT panic.
    - `test_xeq_ind_happy` (FN-PROG-07): build a program with LBL "10" → PushNum(99) → Rtn → END; set regs[3] = HpNum::from(10); main: XeqInd(3); assert push to call_stack + pc lands past LBL.
    - `test_xeq_ind_4_deep_call_stack_rejects`: NOTE — `run_program` clears `state.call_stack` at entry (program.rs line 162), so pre-population would be wiped. Two valid drivers, pick (a): **(a) Use `resume_program(state)` to drive execution**, which does NOT clear `call_stack` per D-22.2. Sequence: build program `[XeqInd(3), END]`; set `state.regs[3] = HpNum::from(10)`; set `state.pc = 0`; pre-push 4 dummy `usize` frames onto `state.call_stack`; call `resume_program(state)` → expects `Err(HpError::CallDepth)` and `call_stack.len() == 4` (pre-mutation atomicity). **(b)** Alternatively nest four `LBL`+`XEQ` pairs organically to fill `call_stack` to 4 before the `XEQ IND` step — but driver (a) is the simpler test.
    - `test_xeq_ind_reg_out_of_range_rejects`: regs.len() < specified reg; expects Err(HpError::InvalidOp), no panic.
    - `test_xeq_ind_non_integer_rejects`: state.regs[3] = HpNum::from(10.5); XeqInd(3) → Err(InvalidOp).
    - Module header `#![allow(clippy::unwrap_used)]` per RESEARCH.md test convention.
    - Uses public imports `use hp41_core::ops::{dispatch, Op, program::{run_program, resume_program}};` (resume_program must be re-exported or directly accessible — verify via the existing exports module).
  </behavior>
  <action>
    1. Create new file `hp41-core/tests/phase22_program_control.rs`.
    2. Add module-level header: `//! Integration tests for Phase 22 Plan 01 (program control: STOP / PSE / GTO IND / XEQ IND + resume_program).` + `#![allow(clippy::unwrap_used)]`.
    3. Add `use` statements: `use hp41_core::ops::{dispatch, Op}; use hp41_core::ops::program::{run_program, resume_program}; use hp41_core::{CalcState, HpError, HpNum};`. If `resume_program` is not re-exported, use the fully-qualified path or add a pub-use in `hp41-core/src/ops/program.rs` mod declaration.
    4. Write each of the 13 tests listed in <behavior>. Each test is a small `#[test]` function constructing a fresh CalcState, populating program/regs as needed, dispatching, and asserting state.
    5. For test_stop_then_resume: explicitly construct program = `vec![Op::Lbl("A".to_string()), Op::PushNum(HpNum::from(42i32)), Op::Stop, Op::PushNum(HpNum::from(99i32))]`. Call run_program; assert post-STOP invariants. Call resume_program; assert continuation.
    6. For test_pse_writes_both_channels: include `state.display_mode` set explicitly (e.g., default FIX 4) and assert exact display_override string format matches format_hpnum output. Assert `state.event_buffer.contains(&"PAUSE 1000".to_string())`.
    7. For Pitfall 3 tests: use a 2-step then 3-step sequence; verify the dispatch-top clear behavior described in Phase 21 (Pitfall 5).
    8. For non-integer reject tests: use HpNum::from_str("12.345") or similar — confirm trunc_int != original.
    9. For call_stack 4-deep tests: pre-push 4 dummy frames onto state.call_stack, then drive XeqInd via **`resume_program(state)`** (NOT `run_program`, which clears call_stack at line 162). Sequence: `state.program = vec![Op::XeqInd(3), Op::End]`; `state.regs[3] = HpNum::from(10);`; `state.pc = 0;`; `state.call_stack.extend(vec![999usize; 4]);`; `let result = resume_program(&mut state);`; `assert!(matches!(result, Err(HpError::CallDepth)));`; `assert_eq!(state.call_stack.len(), 4);` (pre-mutation atomicity verified — push did not happen).
    10. Run `cargo test --package hp41-core --test phase22_program_control` and verify all tests pass.
  </action>
  <acceptance_criteria>
    - `cargo test --package hp41-core --test phase22_program_control` exits 0 with ≥13 tests passing (the count above is a minimum; planner may split or merge edge cases)
    - File `hp41-core/tests/phase22_program_control.rs` exists and is ≥80 lines
    - Every FN-ID in this plan (FN-PROG-01, -02, -06, -07) has at least one test referencing its semantics
    - Pitfall 1 (Stop-no-display) has an explicit sentinel test
    - Pitfall 2 (resume reset on Err) has an explicit sentinel test
    - Pitfall 3 (Pse display_override timing) has TWO sentinel tests (survives next step; cleared by next dispatch)
    - `just ci` exits 0 (clippy + fmt + workspace test stays green)
  </acceptance_criteria>
  <verify>
    <automated>cargo test --package hp41-core --test phase22_program_control 2>&1 | tee /tmp/22-01-06.log; tail -5 /tmp/22-01-06.log; wc -l hp41-core/tests/phase22_program_control.rs; just ci 2>&1 | tail -10</automated>
  </verify>
  <done>Integration test file exists, ≥13 tests pass, all 4 FN-IDs covered, Pitfalls 1/2/3 each have explicit sentinel tests, just ci green.</done>
</task>

</tasks>

<verification>
- `cargo check --workspace` exits 0 after every task (incremental compile).
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (no .unwrap() introduced; zero-panic policy preserved per D-22.23).
- `cargo test --package hp41-core --test phase22_program_control` exits 0 with all listed tests passing.
- `just ci` exits 0 (full workspace lint + test).
- Spot-check: `grep -nE '^\s*(Op::Stop|Op::Pse|Op::GtoInd|Op::XeqInd)\b' hp41-core/src/ops/mod.rs` shows each new variant in the Op enum.
- `grep -nE '"STOP"|"PSE"|"GTO IND|"XEQ IND' hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` shows 4 display strings × 2 copies = 8 hits minimum.
- Pitfall sentinels: test names visible in `cargo test --package hp41-core --test phase22_program_control -- --list` confirm the three Pitfall-specific tests exist by name.
</verification>

<success_criteria>
1. `Op::Stop` halts run_loop without writing display_override (Pitfall 1); `pc < program.len()` and `is_running == false` after run_program returns (D-22.1 STATE INVARIANT).
2. `resume_program(state)` re-enters run_loop from state.pc and resets is_running on both Ok and Err paths (D-22.2, Pitfall 2 verified by test).
3. `Op::Pse` writes display_override = format_hpnum(X) AND event_buffer.push("PAUSE 1000"); run_loop does NOT break; display_override survives subsequent run_loop iterations and is cleared by the next interactive dispatch (D-22.4, Pitfall 3 verified by tests).
4. `Op::GtoInd(reg)` and `Op::XeqInd(reg)` correctly resolve integer-pointer labels via find_in_program; non-integer reject returns InvalidOp; reg-out-of-range returns InvalidOp (not panic); XeqInd's 4-deep guard returns CallDepth before mutation (D-22.15, FN-IND-02 partial).
5. All four new variants land in 4 places (D-22.21) — verified by compile-time exhaustive-match coverage; workspace compiles + clippy clean + just ci green.
</success_criteria>

<output>
After completion, create `.planning/phases/22-program-control-and-memory-ops/22-01-program-control-SUMMARY.md` per the standard template.
</output>
