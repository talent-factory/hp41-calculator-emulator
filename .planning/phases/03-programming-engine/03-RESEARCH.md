# Phase 3: Programming Engine — Research

**Researched:** 2026-05-07
**Domain:** Rust interpreter loop, HP-41 keystroke programming semantics, PRGM mode recording
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** `program: Vec<Op>` flat list in `CalcState`. Labels are `Op::Lbl(String)` markers inside the Vec.

**D-02:** Linear label search: `run_program` scans `state.program` for `Op::Lbl(target)`.

**D-03:** `dispatch()` checks `state.prgm_mode` at the top. When true: `flush_entry_buf` records `Op::PushNum` to `state.program` (instead of stack), then the current Op is appended to `state.program`, and `dispatch()` returns without executing.

**D-04:** `flush_entry_buf` is extended to accept a `target: &mut Vec<Op>` or checks `prgm_mode` internally to route the PushNum to either stack or program.

**D-05:** Three fields added to `CalcState`: `pc: usize`, `call_stack: Vec<usize>`, `is_running: bool`.

**D-06:** `run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError>` sets `is_running = true`, finds the label, runs the interpreter loop until RTN from top-level call, then resets `is_running = false`.

**D-07:** `Op::Test(TestKind)` — single enum covers all 12 HP-41 conditional tests.

**D-08:** `TestKind` variants: `XEqZero`, `XNeZero`, `XLtZero`, `XGtZero`, `XLeZero`, `XGeZero`, `XEqY`, `XNeY`, `XLtY`, `XGtY`, `XLeY`, `XGeY`.

**D-09:** Skip-if-false: condition TRUE → pc += 1 (execute next); condition FALSE → pc += 2 (skip next).

**D-10:** ISG/DSE counter format `CCCCC.FFFDD` parsed by string-splitting at decimal point — never `floor()`/`fmod()`. Left of decimal = current (i64); right padded to 5 chars: first 3 = final, last 2 = step (00 → 1).

**D-11:** ISG: increment current by step; new_current > final → skip (loop exits); else → execute next (continue). DSE: decrement; new_current ≤ final → skip; else → execute next.

**D-12:** Counter stored back with updated CCCCC, preserving the full `CCCCC.FFFDD` format.

**D-13:** `HpError::CallDepth` added to `error.rs`. Message: `"try again"`.

**D-14:** `call_stack` max depth: 4 entries (matches HP-41 hardware). Each entry is a `usize` (return PC = index after the XEQ op).

### Claude's Discretion

- Stack-lift semantics for all programming ops: LBL, GTO, XEQ, RTN, PrgmMode, Test, ISG, DSE = **Neutral**
- `Op::Rtn` at top-level (empty call_stack): terminates execution normally (not an error)
- GTO / XEQ to unknown label → `HpError::InvalidOp`
- When `is_running = false`, GTO/XEQ/RTN dispatched in non-prgm mode: RTN is a no-op (or returns InvalidOp)

### Deferred Ideas (OUT OF SCOPE)

- Indirect addressing (`STO IND`, `GTO IND`)
- Step-number display in PRGM mode ("001 LBL A") — Phase 4 TUI concern
- SST/BST navigation — Phase 4 TUI keyboard concern
- R/S (run/stop) key handling — Phase 4 TUI keyboard concern
- Program checksum/size display — Phase 5 persistence
- Synthetic programming — explicitly out of scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PROG-01 | User can record keystroke programs using LBL, GTO, XEQ, RTN, conditional tests, ISG, and DSE | Op enum extension + dispatch prgm_mode gate + run_program interpreter loop |
| PROG-02 | ISG/DSE counter format (`CCCCC.FFFDD`) behaves identically to HP-41 hardware (no float arithmetic on counter fields) | String-split parsing algorithm; ADR-001 confirmed this rule |
</phase_requirements>

---

## Summary

Phase 3 adds a complete keystroke programming engine to `hp41-core`. The implementation spans three integration points: (1) extending `CalcState` with program storage and execution state, (2) modifying `dispatch()` to record ops to the program Vec when `prgm_mode = true`, and (3) implementing `run_program()` as a PC-driven interpreter loop in a new `ops/program.rs` submodule.

The codebase is in excellent shape for this extension. All architectural patterns from Phases 1 and 2 are directly reusable: `Result<(), HpError>` returns everywhere, `apply_lift_effect` for all ops, single `&mut CalcState` threading, and integration tests living in `hp41-core/tests/`. The `flush_entry_buf` routing problem (D-03/D-04) is the trickiest integration point because it must decide at runtime whether to push onto the stack or append `Op::PushNum(n)` to the program Vec.

The ISG/DSE counter parsing is algorithmically straightforward once the string-split rule is applied, but has several edge cases that must be locked down in tests before implementation. The Rust borrow checker concern (state holds program and interpreter reads from it) is a genuine constraint that requires either borrowing the program as a local slice before the loop or taking a snapshot — this is the primary Rust-specific hazard of the phase.

**Primary recommendation:** Implement `flush_entry_buf` routing by checking `state.prgm_mode` internally (not a second parameter), take a snapshot of `state.program.clone()` at the start of `run_program` to solve the borrow conflict, and make ISG/DSE the first tests written before any implementation.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| PRGM mode recording (append op to Vec) | `hp41-core` dispatch gate | — | Pure state mutation; no UI |
| Program interpreter loop | `hp41-core` `run_program()` | — | Business logic; UI must not reach inside the loop |
| Label lookup / branch resolution | `hp41-core` `ops/program.rs` | — | Linear scan over `state.program` |
| Conditional test evaluation | `hp41-core` `ops/program.rs` | — | Stack reads only; clean layer boundary |
| ISG/DSE counter field parsing | `hp41-core` `ops/program.rs` | — | Accesses `state.regs[n]` via HpNum string representation |
| Call stack depth enforcement | `hp41-core` `run_program` / `op_xeq` | — | Error before mutation; never panic |
| Run/stop (R/S key) wiring | **DEFERRED** Phase 4 TUI | — | Out of scope for this phase |

---

## Standard Stack

### Core (already in workspace — no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rust_decimal` | 1.41 (workspace) | `HpNum` inner type; ISG/DSE string representation via `to_string()` | ADR-001; already a dependency with `maths` feature [VERIFIED: Cargo.toml] |
| `thiserror` | workspace | `HpError` derive macro | Already used for error.rs [VERIFIED: Cargo.toml] |

### Dev dependencies (already present)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `proptest` | 1.11 | Property-based tests for ISG/DSE counter field edge cases | Counter parsing invariants |
| `insta` | 1.47 (yaml) | Snapshot tests for program state after execution sequences | Complex multi-step execution verification |

**No new dependencies required for Phase 3.** [VERIFIED: Cargo.toml examined; all needed tools present]

---

## Architecture Patterns

### System Architecture Diagram

```
HP-41 User Input
       │
       ▼
  dispatch(state, op)
       │
       ├─[prgm_mode = true]──► flush_entry_buf → append PushNum to program
       │                        then append current op to program → return Ok(())
       │
       └─[prgm_mode = false]─► flush_entry_buf → push to stack (existing path)
                                then match op:
                                  Lbl(s)      → Neutral (no-op at runtime)
                                  Gto(s)      → linear scan, set pc (only if is_running)
                                  Xeq(s)      → push call_stack, linear scan, set pc
                                  Rtn         → pop call_stack or halt
                                  Test(kind)  → evaluate, pc += 1 or pc += 2
                                  Isg(reg)    → parse CCCCC.FFFDD, increment, store back
                                  Dse(reg)    → parse CCCCC.FFFDD, decrement, store back
                                  PrgmMode    → toggle prgm_mode flag

External call: run_program(state, "A")
       │
       ├── find Op::Lbl("A") in state.program → get start pc
       ├── set is_running = true
       └── interpreter loop:
             current_op = program[pc]
             pc += 1
             match current_op:
               Rtn + empty call_stack → break (normal termination)
               Rtn + call_stack       → pc = call_stack.pop()
               Gto(s)                 → pc = find_label(program, s)?
               Xeq(s)                 → push pc to call_stack, pc = find_label(program, s)?
               Test(kind)             → if false: pc += 1 (skip next)
               Isg/Dse(reg)           → parse, update, if exit condition: pc += 1
               other ops              → execute via internal dispatch (no flush)
             loop
```

### Recommended Project Structure

```
hp41-core/src/
├── ops/
│   ├── mod.rs           # Op enum + dispatch() — extend with Phase 3 variants + prgm_mode gate
│   ├── arithmetic.rs    # (Phase 1, unchanged)
│   ├── stack_ops.rs     # (Phase 1, unchanged)
│   ├── math.rs          # (Phase 2, unchanged)
│   ├── registers.rs     # (Phase 2, unchanged)
│   ├── alpha.rs         # (Phase 2, unchanged)
│   └── program.rs       # NEW: LBL, GTO, XEQ, RTN, Test, ISG, DSE + run_program
├── state.rs             # extend CalcState with 5 new fields
├── error.rs             # add HpError::CallDepth
└── lib.rs               # export run_program as public API

hp41-core/tests/
└── program_tests.rs     # NEW: PROG-01 + PROG-02 integration tests
```

### Pattern 1: PRGM Mode Gate in dispatch()

**What:** At the top of `dispatch()`, check `state.prgm_mode`. If true, call a recording version of `flush_entry_buf` (appends PushNum to program), then push the current op to `state.program`, return `Ok(())`.

**When to use:** Every call to `dispatch()` when the user is recording a program.

**Key design decision (D-04):** `flush_entry_buf` should check `state.prgm_mode` internally rather than accepting a `target` parameter. This avoids changing the function signature (which would require updating all existing callers) and keeps the routing logic in one place. [ASSUMED — both approaches are valid Rust; this recommendation is based on minimal diff to existing code]

```rust
// Source: derived from ops/mod.rs existing flush_entry_buf pattern [VERIFIED: ops/mod.rs]
pub fn flush_entry_buf(state: &mut CalcState) -> Result<(), HpError> {
    if state.entry_buf.is_empty() {
        return Ok(());
    }
    let s = state.entry_buf.clone();
    state.entry_buf.clear();
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    let n = HpNum::rounded(d);

    if state.prgm_mode {
        // PRGM mode: record PushNum to program, not to stack
        state.program.push(Op::PushNum(n));
        // lift_enabled is NOT changed — recording does not affect execution state
    } else {
        // Normal mode: push onto stack (existing behavior)
        crate::stack::enter_number(state, n);
        crate::stack::apply_lift_effect(state, LiftEffect::Enable);
    }
    Ok(())
}

// In dispatch(), at the very top:
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    flush_entry_buf(state)?;

    if state.prgm_mode {
        // Recording mode: store op and return without executing
        // PrgmMode op is special — it executes immediately to EXIT prgm_mode
        if matches!(op, Op::PrgmMode) {
            state.prgm_mode = false;
            apply_lift_effect(state, LiftEffect::Neutral);
            return Ok(());
        }
        state.program.push(op);
        return Ok(());
    }

    match op { /* existing arms + new Phase 3 arms */ }
}
```

**Note on PrgmMode entering:** When `prgm_mode = false` and `Op::PrgmMode` is dispatched, it sets `prgm_mode = true`. When `prgm_mode = true` and `Op::PrgmMode` is dispatched again (user exits PRGM mode), it executes immediately (breaks recording) to set `prgm_mode = false`. This matches HP-41 behavior where pressing PRGM again exits PRGM mode. [ASSUMED — based on HP-41 hardware behavior; verify via ROADMAP.md success criteria]

### Pattern 2: run_program() Interpreter Loop

**What:** Public function that executes a recorded program starting at a named label.

**Rust borrow conflict:** `state.program` is a `Vec<Op>` inside `CalcState`. The interpreter loop must read `state.program[pc]` while also passing `&mut CalcState` to the op-execution sub-functions. This is a genuine Rust borrow checker problem: you cannot hold a reference into `state.program` and also hold `&mut state`. [VERIFIED: direct consequence of Rust ownership rules]

**Solution:** Clone the program into a local `Vec<Op>` at the start of `run_program`. HP-41 programs are ≤999 steps; cloning a Vec of ~1000 small Op variants is negligible cost. This completely avoids the borrow conflict and keeps the interpreter simple.

```rust
// Source: derived from codebase patterns [VERIFIED: ops/mod.rs, state.rs]
pub fn run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError> {
    // Clone program to avoid borrow conflict when executing ops against &mut state
    let program = state.program.clone();

    // Linear scan for entry label (D-02)
    let start = program.iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
        .ok_or(HpError::InvalidOp)?;

    state.pc = start + 1; // start executing the step AFTER the LBL
    state.call_stack.clear();
    state.is_running = true;

    let result = run_loop(state, &program);

    state.is_running = false;
    result
}

fn find_label(program: &[Op], label: &str) -> Result<usize, HpError> {
    program.iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == label))
        .ok_or(HpError::InvalidOp)
}

fn run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    loop {
        if state.pc >= program.len() {
            // Ran off end of program = implicit RTN at top level
            break;
        }
        let op = program[state.pc].clone();
        state.pc += 1;

        match op {
            Op::Rtn => {
                match state.call_stack.pop() {
                    Some(return_pc) => state.pc = return_pc,
                    None => break, // top-level RTN = normal termination
                }
            }
            Op::Lbl(_) => {
                // LBL is a no-op during execution — just advance pc (already done above)
            }
            Op::Gto(label) => {
                let target = find_label(program, &label)?;
                state.pc = target + 1; // jump to step after LBL
            }
            Op::Xeq(label) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth); // D-13/D-14
                }
                state.call_stack.push(state.pc); // save return address (step after XEQ)
                let target = find_label(program, &label)?;
                state.pc = target + 1;
            }
            Op::Test(kind) => {
                let condition = evaluate_test(state, kind);
                if !condition {
                    state.pc += 1; // skip next step (D-09)
                }
            }
            Op::Isg(reg) => {
                let skip = op_isg(state, reg)?;
                if skip { state.pc += 1; }
            }
            Op::Dse(reg) => {
                let skip = op_dse(state, reg)?;
                if skip { state.pc += 1; }
            }
            other => {
                // All other ops execute normally via internal execute (not dispatch — no flush)
                execute_op(state, other)?;
            }
        }
    }
    Ok(())
}
```

**Note:** `execute_op` is a private helper that matches all non-programming ops. It must NOT call `flush_entry_buf` (no digit entry happens mid-program) and must NOT check `prgm_mode`. This is distinct from `dispatch()`.

### Pattern 3: ISG/DSE Counter Parsing — Exact Algorithm

**What:** Parse `CCCCC.FFFDD` format by string-splitting at the decimal point. Never use `floor()`/`fmod()` (ADR-001 rule). [VERIFIED: state.rs ADR-001 comment, CONTEXT.md D-10]

**Counter format:**
- `CCCCC` = current count (integer part, can be negative, up to 5 digits)
- `.FFF` = final count (first 3 digits after decimal — thousands separator not applicable)
- `DD` = step increment (last 2 digits after decimal; `00` means step = 1)

**Exact success-criterion case:** `1.00500`
- Left of `.`: `"1"` → current = 1
- Right of `.`: `"00500"` (padded to 5 chars with trailing zeros if needed)
  - First 3: `"005"` → final = 5
  - Last 2: `"00"` → step = 0 → use 1

```rust
// Source: CONTEXT.md D-10, ADR-001 in state.rs [VERIFIED]
fn parse_counter(n: &HpNum) -> Result<(i64, i64, i64), HpError> {
    let s = n.inner().to_string(); // e.g., "1.005" from Decimal("1.00500")
    // Decimal normalizes trailing zeros — split on "."
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos+1..])
    } else {
        (s.as_str(), "")
    };

    let current: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;

    // Pad frac to exactly 5 characters with trailing zeros
    let frac_padded = format!("{:0<5}", frac_part); // left-align, pad right with zeros
    // e.g., "005" from "1.005" → "00500" after pad
    // e.g., "00500" from "1.00500" → "00500" (already 5 chars)

    // Final = first 3 digits of frac_padded
    let final_val: i64 = frac_padded[..3].parse().map_err(|_| HpError::InvalidOp)?;

    // Step = last 2 digits of frac_padded; 00 → 1
    let step_raw: i64 = frac_padded[3..5].parse().map_err(|_| HpError::InvalidOp)?;
    let step = if step_raw == 0 { 1 } else { step_raw };

    Ok((current, final_val, step))
}

fn build_counter(current: i64, frac_padded: &str) -> Result<HpNum, HpError> {
    // Reconstruct: new_current.FFFsstep (FFF and step digits unchanged)
    let s = format!("{}.{}", current, frac_padded);
    let d = rust_decimal::Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::rounded(d))
}
```

**Critical edge case — Decimal trailing zero normalization:** `rust_decimal` may normalize `1.00500` to `1.005` when stored. When parsing back, `"005"` has only 3 chars, so the frac_part `"005"` padded to 5 → `"00500"`. This works correctly. But `"1.005"` → `"1.00500"` may differ based on how HpNum is constructed and stored. Tests must verify the round-trip.

**Negative current:** `int_part` will be `"-3"` for a counter like `-3.00500`. `parse::<i64>()` handles the minus sign correctly. [VERIFIED: Rust std documentation]

### Pattern 4: TestKind Evaluation

**What:** Evaluate a conditional against the stack and return `bool` (true = execute next, false = skip next).

```rust
// Source: CONTEXT.md D-08, D-09 [VERIFIED]
fn evaluate_test(state: &CalcState, kind: TestKind) -> bool {
    let x = state.stack.x.inner();
    let y = state.stack.y.inner();
    use rust_decimal::Decimal;
    let zero = Decimal::ZERO;

    match kind {
        TestKind::XEqZero => x == zero,
        TestKind::XNeZero => x != zero,
        TestKind::XLtZero => x < zero,
        TestKind::XGtZero => x > zero,
        TestKind::XLeZero => x <= zero,
        TestKind::XGeZero => x >= zero,
        TestKind::XEqY    => x == y,
        TestKind::XNeY    => x != y,
        TestKind::XLtY    => x < y,
        TestKind::XGtY    => x > y,
        TestKind::XLeY    => x <= y,
        TestKind::XGeY    => x >= y,
    }
}
```

**Stack-lift effect:** All Test ops are Neutral — they read but do not consume X or Y. HP-41 hardware leaves the stack unchanged after a conditional test. [VERIFIED: CONTEXT.md D-09, Claude's Discretion section]

### Anti-Patterns to Avoid

- **Calling `dispatch()` from inside the run loop:** `dispatch()` calls `flush_entry_buf()`, which is wrong inside an executing program (there is no digit entry mid-execution). Use a private `execute_op()` or `dispatch_internal()` that skips the flush and the prgm_mode gate.
- **Holding a reference into `state.program` during the loop:** Will not compile if you also need `&mut state`. Clone the program Vec at the start of `run_program`.
- **Using `floor()`/`fmod()` for ISG/DSE field extraction:** Prohibited by ADR-001. `1.00500` as f64 would lose the step field entirely.
- **Panicking on unknown label:** Must return `HpError::InvalidOp`, not panic. Applies to both GTO and XEQ.
- **Mutating `state.program` during run_loop:** The cloned Vec guards against accidental mutation, but the design must not allow XEQ to record to `state.program` during execution (prgm_mode must be false when `is_running = true`).
- **Appending LBL itself to program when entering PRGM mode:** The very first op in prgm_mode should record normally. The `prgm_mode` gate must record ALL ops including `Op::Lbl(s)` — that is correct; Lbl becomes a marker in the Vec.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Decimal string parsing | Custom BCD parser | `Decimal::from_str` (already in HpNum) | Handles sign, precision, edge cases correctly |
| String formatting for counter | Manual digit assembly | `format!("{:.5}", ...)` or `format!("{}.{}", ...)` | Standard, correct for all corner cases |
| Error propagation | Manual match chains | `?` operator on `Result` | Already established pattern throughout codebase |

**Key insight:** No new dependencies needed. The existing `rust_decimal`, `thiserror`, `proptest`, and `insta` crates cover all Phase 3 needs. [VERIFIED: Cargo.toml]

---

## Common Pitfalls

### Pitfall 1: Borrow Conflict in run_loop

**What goes wrong:** `program[state.pc]` borrows `state.program`, but then you pass `&mut state` to execute an op — Rust rejects this as simultaneous `&` and `&mut` borrows of `state`.

**Why it happens:** `state.program` is a field of `CalcState`, and `&mut state` is a mutable reference to the whole struct, which includes `program`.

**How to avoid:** Clone `state.program` into a local `Vec<Op>` at the start of `run_program`. HP-41 programs are at most 999 steps; the clone is trivially cheap.

**Warning signs:** Compiler error `cannot borrow 'state.program' as immutable because it is also borrowed as mutable`.

### Pitfall 2: flush_entry_buf Called Inside Run Loop

**What goes wrong:** If `run_loop` calls `dispatch()` for sub-ops, `flush_entry_buf` fires and may alter state unexpectedly (entry_buf is empty mid-program, so it's a no-op, but the `prgm_mode` check in the new version could route wrongly if `is_running = true` and `prgm_mode` ever gets set during execution).

**Why it happens:** `dispatch()` is the public entry point and unconditionally calls `flush_entry_buf`. Inside a running program, no digit entry is pending.

**How to avoid:** Create a private `execute_op(state, op)` that is `dispatch()` minus the flush and prgm_mode gate. Only `run_loop` calls this. External callers use `dispatch()`.

**Warning signs:** Test where `state.entry_buf` is non-empty before `run_program` and the program double-pushes a value.

### Pitfall 3: ISG/DSE Decimal Trailing Zero Normalization

**What goes wrong:** `rust_decimal` internally normalizes `1.00500` and may print it as `1.005` when calling `.to_string()`. The frac_part extracted is `"005"` (3 chars), not `"00500"` (5 chars). The pad-to-5 step must pad with trailing zeros, not leading zeros.

**Why it happens:** `format!("{:0<5}", "005")` → `"00500"` (correct: left-align, pad right with `0`). `format!("{:0>5}", "005")` → `"00005"` (wrong: right-align with leading zeros). The `<` vs `>` in the format string is critical.

**How to avoid:** Use `format!("{:0<5.5}", frac_part)` (truncate at 5 if somehow longer, pad right if shorter). Test the exact case `1.00500` end-to-end.

**Warning signs:** ISG loop iterates wrong number of times; step or final value is miscalculated.

### Pitfall 4: Off-by-One on PC After Label Jump

**What goes wrong:** `run_program` finds label at index `start`. If `state.pc = start` (not `start + 1`), the first iteration executes `Op::Lbl(...)`, which is a no-op but wastes one cycle. This is harmless for labels, but for GTO/XEQ jumps to a label, setting `pc = find_label(...)` would re-execute the Lbl marker — still harmless, but semantically wrong.

**Why it happens:** The label itself is a record in the Vec. Execution begins at the instruction *after* the label.

**How to avoid:** Always set `state.pc = label_index + 1` when jumping to a label. Verify with a test: program `[Lbl("A"), PushNum(1), Rtn]` — after `run_program("A")`, X must be 1, not whatever Lbl does.

**Warning signs:** First instruction of program not executing, or Lbl reprocessed on every GTO jump.

### Pitfall 5: call_stack Depth Check — Off by One

**What goes wrong:** The check `call_stack.len() >= 4` must fire when attempting the *5th* XEQ. If the check is `> 4`, it allows 5 levels and the 6th triggers the error — wrong.

**Why it happens:** HP-41 allows 4 nested subroutine calls (call_stack holds 4 return addresses). The 5th attempt must be rejected.

**How to avoid:** Before pushing to `call_stack`, check `if call_stack.len() >= 4 { return Err(HpError::CallDepth) }`. Write a test that nests exactly 4 levels (must succeed) and exactly 5 levels (must return `HpError::CallDepth`). [VERIFIED: CONTEXT.md D-13/D-14]

**Warning signs:** 5-level nesting succeeds; or 4-level nesting (valid) returns an error.

### Pitfall 6: PrgmMode Exit is Not Recorded

**What goes wrong:** If the recording gate in `dispatch()` blindly appends all ops including `Op::PrgmMode`, pressing PRGM to exit will append `Op::PrgmMode` to the program and never actually exit PRGM mode.

**Why it happens:** The gate fires before the op-specific logic.

**How to avoid:** In the `prgm_mode = true` branch, handle `Op::PrgmMode` specially: execute it immediately (set `prgm_mode = false`), return without recording.

**Warning signs:** User can never exit PRGM mode; program Vec grows unboundedly.

### Pitfall 7: GTO in Non-Running, Non-PRGM Mode

**What goes wrong:** GTO dispatched interactively (not in a program) must not crash. CONTEXT.md says GTO/XEQ to unknown label → `InvalidOp`; but GTO to a known label in non-running mode is semantically undefined on HP-41 (interactive GTO sets a cursor position, not a jump).

**How to avoid:** For this phase, GTO/XEQ dispatched when `is_running = false` and `prgm_mode = false` should return `HpError::InvalidOp` (or be a no-op). This is tagged as Claude's Discretion in CONTEXT.md.

**Warning signs:** Interactive GTO panics or modifies `state.pc` when no program is running.

---

## Code Examples

### CalcState Extension

```rust
// Source: state.rs current CalcState [VERIFIED: hp41-core/src/state.rs]
pub struct CalcState {
    // ... existing fields (Phase 1 + Phase 2) ...
    pub stack: Stack,
    pub regs: [HpNum; 100],
    pub alpha_reg: String,
    pub alpha_mode: bool,
    pub angle_mode: AngleMode,
    pub display_mode: DisplayMode,
    pub entry_buf: String,

    // Phase 3 additions:
    /// Keystroke program storage — flat list of Ops (labels are Op::Lbl markers)
    pub program: Vec<Op>,
    /// PRGM mode: when true, dispatch records ops to program instead of executing
    pub prgm_mode: bool,
    /// Program counter — current execution position in program Vec
    pub pc: usize,
    /// Subroutine return stack (max 4 entries per HP-41 hardware limit)
    pub call_stack: Vec<usize>,
    /// True while run_program is executing
    pub is_running: bool,
}

impl CalcState {
    pub fn new() -> Self {
        CalcState {
            // ... existing field inits ...
            program: Vec::new(),
            prgm_mode: false,
            pc: 0,
            call_stack: Vec::new(),
            is_running: false,
        }
    }
}
```

### Op Enum Extension

```rust
// Source: ops/mod.rs Op enum [VERIFIED: hp41-core/src/ops/mod.rs]
// Phase 3 additions to the Op enum:

/// HP-41 conditional test kinds (12 total). LiftEffect: Neutral.
#[derive(Debug, Clone, PartialEq)]
pub enum TestKind {
    XEqZero, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero,
    XEqY,    XNeY,    XLtY,    XGtY,    XLeY,    XGeY,
}

// In Op enum:
/// LBL "name" — program label marker. No-op during execution. LiftEffect: Neutral.
Lbl(String),
/// GTO "name" — unconditional branch to label. LiftEffect: Neutral.
Gto(String),
/// XEQ "name" — subroutine call. LiftEffect: Neutral.
Xeq(String),
/// RTN — return from subroutine (or terminate if top-level). LiftEffect: Neutral.
Rtn,
/// PRGM — toggle program recording mode. LiftEffect: Neutral.
PrgmMode,
/// Conditional test. LiftEffect: Neutral (does not consume stack).
Test(TestKind),
/// ISG n — increment and skip if greater. LiftEffect: Neutral.
Isg(u8),
/// DSE n — decrement and skip if equal or less. LiftEffect: Neutral.
Dse(u8),
```

### HpError Extension

```rust
// Source: error.rs [VERIFIED: hp41-core/src/error.rs]
#[derive(Error, Debug, PartialEq, Clone)]
pub enum HpError {
    #[error("overflow")]
    Overflow,
    #[error("divide by zero")]
    DivideByZero,
    #[error("invalid operation")]
    InvalidOp,
    #[error("domain error")]
    Domain,
    // Phase 3 addition:
    #[error("try again")]
    CallDepth,
}
```

### lib.rs public API export

```rust
// Source: lib.rs [VERIFIED: hp41-core/src/lib.rs]
// Add to lib.rs:
pub use ops::program::run_program;
```

---

## Wave Decomposition

Phase 3 has clear dependency boundaries that allow some parallelization:

### Wave 0 — Test scaffolds (must precede implementation)
Write `hp41-core/tests/program_tests.rs` with all tests in RED state. This is a single file; cannot be parallelized.

### Wave 1 — Data model (no dependencies on each other, fully parallelizable)

| Plan | File | Changes |
|------|------|---------|
| 03-01 | `state.rs` | Add 5 new fields to CalcState + Default init |
| 03-02 | `error.rs` | Add `HpError::CallDepth` |
| 03-03 | `ops/mod.rs` (Op enum only) | Add `TestKind` enum + 8 new Op variants |

These three plans have zero inter-dependency and can be executed in parallel.

### Wave 2 — Core implementation (depends on Wave 1)

| Plan | File | Changes | Dependencies |
|------|------|---------|-------------|
| 03-04 | `ops/mod.rs` (dispatch gate) | prgm_mode gate + flush_entry_buf routing | Wave 1 complete |
| 03-05 | `ops/program.rs` (new file) | `op_lbl`, `op_gto`, `op_xeq`, `op_rtn`, `op_test`, `op_isg`, `op_dse` + `run_program` | Wave 1 complete |

Plans 03-04 and 03-05 can be executed in parallel. 03-04 modifies `ops/mod.rs`; 03-05 creates a new file. No file conflict.

### Wave 3 — Integration and exports (depends on Wave 2)

| Plan | File | Changes |
|------|------|---------|
| 03-06 | `ops/mod.rs` (dispatch match arms) | Wire new Op variants to program.rs functions |
| 03-07 | `lib.rs` | Export `run_program`; `just ci` green gate |

03-06 and 03-07 modify different files and can be parallelized. However, `just ci` green in 03-07 must wait for all Wave 2 + 03-06 to complete.

**Summary:** Wave 0 → [Wave 1: 3 parallel plans] → [Wave 2: 2 parallel plans] → [Wave 3: 2 parallel plans, then ci gate]

---

## Test Suite Structure (program_tests.rs)

All tests use the integration test pattern established by `entry_buf_tests.rs` and `register_tests.rs`: import via `hp41_core::*`, construct `CalcState::new()`, dispatch ops, assert on state. [VERIFIED: tests/ directory examined]

### Required Test Cases

**PRGM mode recording (PROG-01)**

| Test name | What it verifies |
|-----------|-----------------|
| `test_prgm_mode_toggle` | PrgmMode sets prgm_mode = true; second PrgmMode sets prgm_mode = false |
| `test_prgm_mode_records_ops` | Dispatching Add while prgm_mode=true appends Op::Add to program; does NOT execute |
| `test_prgm_mode_records_pushnum_via_entry_buf` | entry_buf="5"; dispatch(Add) in prgm_mode → program contains [PushNum(5), Add] |
| `test_prgm_mode_does_not_push_stack` | X remains unchanged when ops are recorded |
| `test_prgm_mode_exit_not_recorded` | PrgmMode op while recording exits without appending PrgmMode to program |

**Label and branch (PROG-01)**

| Test name | What it verifies |
|-----------|-----------------|
| `test_run_program_basic_lbl_rtn` | Program [Lbl("A"), PushNum(42), Rtn]; run_program("A") → X = 42 |
| `test_run_unknown_label_returns_invalid_op` | run_program("X") on empty program → Err(InvalidOp) |
| `test_gto_within_program` | Program with GTO forward jump; execution skips over intermediate steps |
| `test_gto_unknown_label_returns_invalid_op` | GTO to non-existent label inside running program → Err(InvalidOp) |

**Subroutine calls (PROG-01)**

| Test name | What it verifies |
|-----------|-----------------|
| `test_xeq_and_rtn` | XEQ "B" inside program "A" calls program "B"; RTN returns to correct step after XEQ |
| `test_xeq_nesting_4_levels_succeeds` | 4 nested XEQ calls succeed |
| `test_xeq_5th_level_returns_call_depth` | 5th XEQ → Err(HpError::CallDepth) |
| `test_rtn_at_top_level_terminates` | RTN with empty call_stack terminates normally, no error |

**Conditional tests (PROG-01)**

| Test name | What it verifies |
|-----------|-----------------|
| `test_test_x_eq_zero_true_executes_next` | X=0, Test(XEqZero) → next step executes |
| `test_test_x_eq_zero_false_skips_next` | X=1, Test(XEqZero) → next step skipped, step+2 executes |
| `test_all_12_test_kinds_correct_bool` | Parameterized: all 12 TestKind variants with true and false inputs |
| `test_conditional_skip_within_program` | Real program: Test(XEqZero); GTO("done"); do_something; Lbl("done"); Rtn |

**ISG/DSE counter (PROG-02)**

| Test name | What it verifies |
|-----------|-----------------|
| `test_isg_increments_4_times_before_skip` | **Success criterion**: R00=1.00500; ISG loop runs 4 times, then skips (R00 = 5.00500 after) |
| `test_isg_exit_condition_at_boundary` | current = final; ISG → increments then skips immediately (loop exits on first iteration) |
| `test_isg_step_zero_treated_as_one` | Step field "00" → step = 1; NOT step = 0 |
| `test_isg_negative_current` | R00 = -2.00500; ISG increments: -2→-1, -1→0, 0→1, 1→2, 2→3, 3→4, 4→5, 5>5 skip |
| `test_dse_decrements_until_skip` | R00 = 3.00100; DSE: 3→2→1→0 (0≤1, but 1≤1 triggers skip) — check exact HP-41 boundary |
| `test_isg_counter_string_round_trip` | Build counter from 1.00500; parse; verify current=1, final=5, step=1 |
| `test_isg_decimal_normalization` | Counter stored as HpNum(1.005); verify frac_padded = "00500" not "00500" truncation |

**Integration (end-to-end program)**

| Test name | What it verifies |
|-----------|-----------------|
| `test_full_program_sum_1_to_n` | Records a program via dispatch in prgm_mode, then runs it; verifies computed result |
| `test_is_running_reset_on_completion` | After run_program, is_running = false |
| `test_is_running_reset_on_error` | run_program returns Err → is_running still = false |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| N/A — Phase 3 is new | PC-driven interpreter with cloned program Vec | Phase 3 | Clean borrow solution |
| N/A | All test kinds as single `Op::Test(TestKind)` | Phase 3 | Symmetric with StoArith pattern |

**Deprecated/outdated:**
- None — this is a new module building on established Phase 1/2 patterns.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `flush_entry_buf` checks `state.prgm_mode` internally (not via parameter) to route PushNum | Pattern 1 | If parameter approach chosen, all existing call sites need updating — manageable but adds diff size |
| A2 | PrgmMode entered via `Op::PrgmMode` dispatch when `prgm_mode=false`; exited by same op when `prgm_mode=true` (toggle) | Pattern 1 | If HP-41 uses separate enter/exit ops, the dispatch gate needs two variants |
| A3 | GTO dispatched interactively (not in program, not recording) returns `HpError::InvalidOp` | Common Pitfalls 7 | If it should be a no-op, tests relying on the error will need to change |
| A4 | `run_loop` exposes ISG/DSE as functions returning `bool` (skip/no-skip) rather than calling `op_isg/dse` which returns `Result<(), HpError>` | Pattern 2 | Either signature works; affects whether ISG/DSE are callable from dispatch() as well |

---

## Open Questions (RESOLVED)

1. **PrgmMode Enter vs Exit as Separate Ops**
   - What we know: CONTEXT.md only mentions `Op::PrgmMode` (singular) and Claude's Discretion says it toggles
   - What's unclear: HP-41 hardware uses the same PRGM key to enter and exit; the toggle model matches
   - RESOLVED: Implement as toggle; add a comment in dispatch() for future reference

2. **GTO in Interactive Mode (Non-Running, Non-Recording)**
   - What we know: CONTEXT.md leaves this to Claude's Discretion; says "RTN is a no-op or error in run context only"
   - What's unclear: What should interactive GTO do? HP-41 moves the program cursor position.
   - RESOLVED: Return `HpError::InvalidOp` for Phase 3; Phase 4 can add cursor navigation when the TUI is built

3. **DSE Boundary Condition Exact Semantics**
   - What we know: D-11 says DSE: `new_current <= final → skip`; ISG: `new_current > final → skip`
   - What's unclear: For DSE with current=1, final=1, step=1: decrement to 0, then 0 ≤ 1 → skip (exits). Verify: does DSE skip when current falls BELOW final, or AT final?
   - RESOLVED: The success criterion only specifies ISG. For DSE: implement `new_current <= final → skip` per D-11; add a test that confirms the boundary case once.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 3 is a pure `hp41-core` Rust library addition. No external services, CLIs, databases, or runtimes beyond the Rust toolchain (already verified working from Phases 1 and 2).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` + `proptest` 1.11 + `insta` 1.47 |
| Config file | none (standard Cargo integration test discovery) |
| Quick run command | `cargo test -p hp41-core program` |
| Full suite command | `just ci` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROG-01 | PRGM mode recording + LBL/GTO/XEQ/RTN/Test | integration | `cargo test -p hp41-core program` | ❌ Wave 0 |
| PROG-02 | ISG/DSE counter field parsing (string-split, no float) | integration | `cargo test -p hp41-core isg` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-core program`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** `just ci` (lint → test → coverage ≥80%) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-core/tests/program_tests.rs` — covers PROG-01 and PROG-02 (all test cases listed in Test Suite Structure section above)
- [ ] No additional framework install needed (proptest and insta already in `[dev-dependencies]`) [VERIFIED: Cargo.toml]

---

## Security Domain

This phase adds no network access, file I/O, user-supplied input parsing beyond the existing `Decimal::from_str` already present in Phase 1/2, or external service integration. No ASVS categories apply to a pure in-memory interpreter loop. Security section: SKIPPED (pure in-memory library computation).

---

## Sources

### Primary (HIGH confidence)

- [VERIFIED: hp41-core/src/ops/mod.rs] — Op enum, dispatch(), flush_entry_buf() patterns
- [VERIFIED: hp41-core/src/state.rs] — CalcState fields, ADR-001 ISG/DSE string-split rule
- [VERIFIED: hp41-core/src/error.rs] — HpError enum structure
- [VERIFIED: hp41-core/src/stack.rs] — LiftEffect, enter_number, binary_result, unary_result
- [VERIFIED: hp41-core/src/num.rs] — HpNum::inner(), HpNum::rounded(), to_string() behavior
- [VERIFIED: hp41-core/Cargo.toml] — dependency versions (no new deps needed)
- [VERIFIED: hp41-core/tests/entry_buf_tests.rs] — integration test patterns
- [VERIFIED: hp41-core/tests/register_tests.rs] — integration test conventions
- [VERIFIED: justfile] — ci recipe: lint → test → coverage
- [VERIFIED: .planning/phases/03-programming-engine/03-CONTEXT.md] — all locked decisions
- [VERIFIED: .planning/REQUIREMENTS.md] — PROG-01, PROG-02 definitions
- [VERIFIED: .planning/ROADMAP.md] — Phase 3 success criteria (authoritative behavioral spec)
- [VERIFIED: .planning/config.json] — nyquist_validation: true

### Secondary (MEDIUM confidence)

- [CITED: Rust Reference — Ownership and Borrowing] — borrow conflict diagnosis for state.program inside run_loop; standard Rust ownership rule, well-established
- [CITED: rust_decimal documentation] — `to_string()` normalization behavior; trailing zeros may be dropped

### Tertiary (LOW confidence — ASSUMED)

- ISG boundary case when current = final exactly (D-11 covers the formula but exact HP-41 hardware semantics not independently verified this session)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all existing crates verified in Cargo.toml
- Architecture: HIGH — all patterns derived from verified existing code in ops/mod.rs
- ISG/DSE parsing: HIGH — algorithm follows directly from ADR-001 and CONTEXT.md D-10/D-11; edge cases documented
- Borrow conflict solution: HIGH — well-known Rust pattern for this exact scenario
- Pitfalls: HIGH — derived from direct code analysis + Rust ownership rules

**Research date:** 2026-05-07
**Valid until:** 2026-07-07 (stable domain — Rust language and rust_decimal API are stable)
