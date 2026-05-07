---
phase: 03-programming-engine
reviewed: 2026-05-07T00:00:00Z
depth: deep
files_reviewed: 6
files_reviewed_list:
  - hp41-core/src/state.rs
  - hp41-core/src/error.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/lib.rs
  - hp41-core/tests/program_tests.rs
findings:
  critical: 2
  warning: 4
  info: 2
  total: 8
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-05-07
**Depth:** deep
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 3 introduces the programming engine: flat program storage, a recording gate in `dispatch()`, `run_program` / `run_loop` interpreter, `flush_entry_buf` PRGM routing, the full ISG/DSE counter mechanism, and 12 conditional tests. The structural design decisions are sound — cloning `state.program` to escape the borrow conflict, handling all flow-control ops directly in `run_loop` rather than routing through `execute_op`, and using string-split parsing for ISG/DSE counters. ISG/DSE semantics, call-stack depth enforcement, the recording gate toggle, and all 12 conditional test comparisons are correct.

Two critical defects are present. First, `run_loop` contains no step counter, timeout, or interruption channel — any infinite loop (including the trivially constructible `Lbl("A"), Gto("A")`) blocks the calling thread forever and violates the 50 ms key-latency quality gate. Second, interactive `XEQ` is documented as running the subroutine via `run_program` but the implementation only mutates `state.pc` and `state.call_stack` without triggering execution, producing an undetectable behavior gap versus real HP-41 hardware.

---

## Critical Issues

### CR-01: `run_loop` has no step limit — infinite loops block the thread permanently

**File:** `hp41-core/src/ops/program.rs:146–201`

**Issue:** `run_loop` is an unbounded `loop {}`. A user-created program containing `Op::Lbl("A"), Op::Gto("A")` will run forever and never return control. There is no step counter, `AtomicBool` stop flag, timeout, or any other interruption mechanism. Because hp41-core uses a single-threaded synchronous model (`poll → update → redraw`), this permanently freezes the TUI, violating the 50 ms key-latency quality gate and the "zero panics" spirit (a hang is as bad as a panic for the UI layer).

The HP-41 hardware stops a running program when the user presses a key (R/S or any key). In an emulator the equivalent is a step-count limit or a shared stop flag checked each iteration.

**Fix:**
```rust
// Add a stop signal field to CalcState (or pass as parameter):
pub stop_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,

// In run_loop, check at the top of each iteration:
fn run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    let mut steps: u64 = 0;
    const MAX_STEPS: u64 = 1_000_000; // ~1 M steps ≈ well above any real HP-41 program
    loop {
        if steps >= MAX_STEPS {
            return Err(HpError::InvalidOp); // or a new HpError::StepLimit
        }
        steps += 1;
        if state.pc >= program.len() { break; }
        // ... rest of loop unchanged
    }
    Ok(())
}
```

A simpler first step is a hard maximum step count (`1_000_000` is generous; a 999-step HP-41 program cannot loop more than ~1 000 times before a counter loop exits). A production solution uses an `Arc<AtomicBool>` stop flag set by the TUI on keypress.

---

### CR-02: Interactive `XEQ` corrupts state without executing the subroutine

**File:** `hp41-core/src/ops/program.rs:56–66`

**Issue:** `op_xeq` is called when the user presses XEQ interactively (outside `run_loop`). It pushes `state.pc` onto `call_stack` and sets `state.pc = target + 1`, then returns `Ok(())`. No execution loop is triggered. The docstring explicitly claims:

> Interactive XEQ (not running) also runs via run_program for Phase 3.

This is false. The function never calls `run_program`. The effect is:
1. `state.call_stack` gains a stale entry (`state.pc` at the time of the interactive call, which is an arbitrary value left by a prior run or zero).
2. `state.pc` is set to a position inside the program that no one will execute.
3. If the user subsequently presses `RTN` interactively, `op_rtn` pops the stale return address and sets `state.pc` to it — further corrupting state.
4. A subsequent `run_program` call clears `call_stack` (line 135), so the damage is cleaned up — but only if the user calls `run_program`. An interactive `Rtn` after an interactive `XEQ` leaves `state.pc` pointing at an arbitrary program position.

On real HP-41 hardware, pressing `XEQ "A"` from keyboard mode runs the subroutine immediately and returns control to the keyboard when the program reaches `RTN`.

**Fix:** Replace the silent-state-mutation approach with an actual execution call, or at minimum match `op_gto`'s pattern and return `InvalidOp` when `!state.is_running` (so the TUI can show a "NOT YET" error until Phase 4 implements interactive XEQ properly):

```rust
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        // Option A (correct): run the subroutine immediately
        return run_program(state, label);
        // Option B (safe stub): reject until properly implemented
        // return Err(HpError::InvalidOp);
    }
    // ... rest of code unchanged for in-program XEQ
}
```

Option A requires that `run_program` handle the `is_running` re-entrancy guard (see WR-02). Option B is a safe stub that prevents state corruption until the feature is complete.

---

## Warnings

### WR-01: `op_gto` is unreachable code — interactive path always returns `InvalidOp`, program path uses `run_loop` directly

**File:** `hp41-core/src/ops/program.rs:43–51`

**Issue:** `op_gto` is called from `dispatch()` via the `Op::Gto(s) => program::op_gto(state, &s)` arm. Its first action is:

```rust
if !state.is_running {
    return Err(HpError::InvalidOp);
}
```

`dispatch()` is never called while `run_loop` is active (the loop uses `execute_op`, not `dispatch`), so `state.is_running` is always `false` at `dispatch()` call time. The early-return fires every time, making `find_label_in_state` and the `state.pc = target + 1` assignment dead code. Inside a running program, `run_loop` handles `Op::Gto` directly (lines 165–167) and never delegates to `op_gto`. The function is never exercised along a code path that reaches line 48.

**Fix:** The `find_label_in_state` call and `state.pc` assignment below the `is_running` guard are dead code and should be removed. Either delete `op_gto` and make the dispatch arm always return `Err(HpError::InvalidOp)`, or document that the function is a placeholder. Add a test asserting that `dispatch(Gto(...))` returns `InvalidOp` so the invariant is encoded.

```rust
pub fn op_gto(state: &mut CalcState, _label: &str) -> Result<(), HpError> {
    // GTO is only meaningful inside a running program; run_loop handles it directly.
    // Interactive GTO is not a supported HP-41 operation from keyboard mode.
    let _ = state; // suppress unused warning
    Err(HpError::InvalidOp)
}
```

---

### WR-02: `run_program` does not guard against re-entrancy — second call while running silently destroys the outer call's state

**File:** `hp41-core/src/ops/program.rs:124–142`

**Issue:** `run_program` sets `state.is_running = true` but does not check whether it is already `true` on entry. If `run_program` were called while already running (for example if CR-02 is fixed by having `op_xeq` call `run_program`), the inner call executes `state.call_stack.clear()` (line 135), wiping the outer call's return-address stack, and sets `state.pc = start + 1`, overwriting the outer call's program counter. When the inner call returns, the outer `run_loop` resumes with a garbage `pc` value, producing undefined behavior (likely running wrong program steps or terminating silently).

Even in today's code where re-entrancy is impossible (single-threaded sync model, `execute_op` never calls `dispatch`), fixing CR-02 will expose this gap immediately.

**Fix:** Check `is_running` on entry and return an error before any mutation:

```rust
pub fn run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError> {
    if state.is_running {
        return Err(HpError::InvalidOp); // or a dedicated HpError::Reentrant
    }
    let program = state.program.clone();
    // ... rest unchanged
}
```

---

### WR-03: `parse_counter` silently truncates fractional parts longer than 5 digits — wrong counter fields extracted without error

**File:** `hp41-core/src/ops/program.rs:338`

**Issue:** The HP-41 ISG/DSE counter format is `CCCCC.FFFDD` — exactly 5 fractional digits. `parse_counter` documents this, but when the stored `HpNum` has more than 5 fractional digits (e.g. a user stored `1.123456789` in a counter register via `STO`), the code silently truncates:

```rust
let frac_padded = if frac_padded.len() > 5 {
    frac_padded[..5].to_string()  // silent truncation
} else { frac_padded };
```

For `1.123456789` this yields `frac_padded = "12345"`, extracting `final_val = 123` and `step = 45` — values that have no relation to the user's intent. No error is returned; ISG/DSE proceeds silently with wrong semantics. The HP-41 hardware would not accept an arbitrary decimal value as a counter; the emulator should reject it.

**Fix:** Return `HpError::InvalidOp` when `frac_part.len() > 5`, enforcing the documented format:

```rust
let frac_padded = format!("{:0<5}", frac_part);
if frac_padded.len() > 5 {
    return Err(HpError::InvalidOp); // register does not contain a valid HP-41 counter
}
```

---

### WR-04: `op_rtn` docstring is factually wrong about run-termination responsibility

**File:** `hp41-core/src/ops/program.rs:68–78`

**Issue:** The docstring for `op_rtn` states:

> RTN — return from subroutine. If call_stack is empty, terminates run (top-level RTN).

`op_rtn` does not terminate any run. It pops `call_stack` and sets `state.pc`, then returns `Ok(())`. Termination is the responsibility of `run_loop`'s `None => break` arm (line 159), which handles `Op::Rtn` directly and never calls `op_rtn`. `op_rtn` is only reachable from `dispatch()` (the interactive path), where no "run" is active to terminate. The claim in the docstring is false and will mislead future maintainers about the control-flow architecture.

**Fix:** Correct the docstring to describe what `op_rtn` actually does:

```rust
/// RTN: interactive dispatch arm. If call_stack is non-empty, pops the top entry
/// and sets pc to the saved return address. If call_stack is empty, this is a no-op.
///
/// NOTE: program-execution RTN is handled directly by run_loop, which breaks the
/// interpreter loop on empty call_stack. This function is only reached from dispatch()
/// (the interactive keyboard path).
/// LiftEffect: Neutral.
pub fn op_rtn(state: &mut CalcState) -> Result<(), HpError> {
```

---

## Info

### IN-01: `push_decimal` test helper is dead code suppressed by a workaround wrapper

**File:** `hp41-core/tests/program_tests.rs:25–28` and `449–452`

**Issue:** `push_decimal` is defined but never called in any test. Instead of removing it, the code adds a `_use_push_decimal` wrapper marked `#[allow(dead_code)]` to suppress the compiler warning. This pattern actively fights the compiler's unused-code detection rather than removing the dead code.

**Fix:** Remove both `push_decimal` and `_use_push_decimal`. If `push_decimal` is needed for future tests, add it then.

---

### IN-02: `HpError::CallDepth` display string `"try again"` does not describe the error

**File:** `hp41-core/src/error.rs:15`

**Issue:**
```rust
#[error("try again")]
CallDepth,
```

`"try again"` is not a meaningful error message for a call-stack depth overflow. The HP-41 hardware displays `TRY AGAIN` for several transient conditions, but the context here is a subroutine nesting limit exceeded — a permanent error for the current program structure, not a transient one. A consumer displaying the error to the user will show a confusing message.

**Fix:** Use a descriptive message:
```rust
#[error("subroutine nesting too deep")]
CallDepth,
```

Or if HP-41 hardware compatibility of the display string is important, `"RTRY"` (HP-41's actual error code) with a doc comment explaining the hardware source.

---

_Reviewed: 2026-05-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
