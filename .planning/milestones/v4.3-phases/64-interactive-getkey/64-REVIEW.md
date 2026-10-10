---
phase: 64-interactive-getkey
reviewed: 2026-06-07T12:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - hp41-cli/src/app.rs
  - hp41-core/src/lib.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/registers.rs
  - hp41-core/src/state.rs
  - hp41-core/tests/phase_64_getkey.rs
  - hp41-core/tests/synthetic_tests.rs
  - hp41-gui/src-tauri/permissions/resume-program-with-key.toml
  - hp41-gui/src-tauri/capabilities/default.json
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-gui/src/App.test.tsx
  - hp41-gui/src/App.tsx
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
status: issues_found
---

# Phase 64: Code Review Report

**Reviewed:** 2026-06-07T12:00:00Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 64 adds event-driven `WaitForKey` yield semantics to the run-loop yield engine established in Phase 63. The implementation is broadly well-structured: the `CalcState` transient field `getkey_captured_code` is correctly annotated `#[serde(default, skip)]`, the 4-way exhaustive-match invariant is satisfied, stack-lift semantics (`LiftEffect::Enable`) are correct, and the CLI/GUI key routing correctly prevents `handle_key`/`dispatchKeyId` from double-processing events during a WaitForKey suspend.

Two correctness bugs were found: one in `resume_program_with_key` that can corrupt state when called spuriously (no guard against calling outside a WaitForKey suspend), and one in the GUI TS layer where ignored GETKEY keys (no `keyCode`) silently return stale state instead of the current server state. Three warnings cover a double-lift in `op_getkey`, a missing `is_running` reset if `op_getkey` errors before `run_loop`, and an unguarded `execute_op` fallthrough for `Op::GetKey` that would silently consume `last_key_code` inside solvers.

---

## Critical Issues

### CR-01: `resume_program_with_key` has no guard against being called when not in WaitForKey suspend

**File:** `hp41-core/src/ops/program.rs:514-538`

**Issue:** `resume_program_with_key` does not check whether `state.pending_yield` is actually `Some(WaitForKey)` before executing. If the frontend calls it spuriously (e.g., a race between two key events, or a bug in the TS layer) the function will:
1. Overwrite `state.getkey_captured_code` with the stale keycode
2. Clear `state.pending_yield` (even if it was a PSE/VIEW yield the timer-driver was about to consume)
3. Call `op_getkey`, pushing the keycode to X — corrupting the stack
4. Re-enter `run_loop` at whatever `state.pc` happens to be — potentially executing the wrong program continuation

The CLI explicitly guards against double-processing (`handle_key` returns early on `WaitForKey`; the inner poll loop owns events). The GUI TS layer also has a guard in `invokeForKey`. However, those are frontend-only; the core function should fail closed on misuse, matching the `resume_program` pattern (which at least checks `state.pc < program.len()`).

The scenario is plausible: a user taps a key, the first tap resolves to `resume_program_with_key`, and before `setCalcState` lands (updating `pending_yield` to `null`) the user taps again. The second `invokeForKey` sees stale `state?.pending_yield?.kind === 'wait_for_key'` and fires a second `resume_program_with_key`, corrupting the resumed execution.

**Fix:**

```rust
pub fn resume_program_with_key(state: &mut CalcState, keycode: u8) -> Result<(), HpError> {
    // Guard: only valid when actually suspended on WaitForKey.
    match &state.pending_yield {
        Some(y) if y.kind == crate::state::YieldKind::WaitForKey => {}
        _ => return Err(HpError::InvalidOp),
    }
    // ... rest of function unchanged
```

This aligns with `resume_program`'s bounds check and makes the function safe to call from any context.

---

### CR-02: GUI TS — ignored GETKEY key (no `keyCode`) returns stale `CalcStateView` to callers

**File:** `hp41-gui/src/App.tsx:131-139`

**Issue:** In `invokeForKey`, when `state?.pending_yield?.kind === 'wait_for_key'` and the key has no `hp41Code` (CHS, clx_or_a, xge_y, shift, ON-equivalent), the function returns `Promise.resolve(state as CalcStateView)` — i.e., the *stale* React state object captured at the time of the call. Callers (`handleClick`, `dispatchKeyId` via its WaitForKey guard in `handleKey`) then call `setCalcState(view)`, which sets the stale object back into React state.

In `handleClick` this is reached via:

```typescript
view = await invokeForKey(effectiveId, calcState, key); // returns stale `state`
setCalcState(view);   // puts stale state back in, triggering a re-render with old data
```

The practical consequence is mild (the stale state equals the current state), but it is semantically wrong: it re-renders with an object reference that may be out of date if any other concurrent update landed between the key tap and `setCalcState`. It also sets `errorMessage` to `null` unconditionally, potentially clearing a visible error that should persist.

More critically, in `handleClick` the stale state is also passed to `maybeFireErrorHaptic(view.display_str, ...)`, which checks whether the display string is an error. If the actual server state has an error visible on the display but the stale state does not (e.g., an error was set by an immediately-preceding operation), the haptic feedback is silently skipped.

**Fix:** Return `Promise.resolve(state)` (no cast needed; `state` is already `CalcStateView`) is acceptable for the wait-continues case, but callers should not call `setCalcState` with it. The cleaner fix is to return a resolved Promise with a sentinel and have callers skip the `setCalcState` path. The simplest correct approach:

```typescript
// In invokeForKey, no-keyCode path during WaitForKey:
// Do NOT return stale state — callers must not setCalcState with it.
// Return a new Promise that never resolves (or a discriminated union).
// Simplest: throw with a sentinel the caller recognizes.
```

Or, restructure `invokeForKey` to return `Promise<CalcStateView | null>` where `null` means "wait continues, do not update state". The caller already does:

```typescript
view = await invokeForKey(...);
setCalcState(view);  // only reached if non-null
```

Currently the stale-state return masks the intent. The no-op path should be explicit.

---

## Warnings

### WR-01: Double `apply_lift_effect(Enable)` in `op_getkey`

**File:** `hp41-core/src/ops/registers.rs:151-163`

**Issue:** `op_getkey` sets `state.stack.lift_enabled = true` explicitly at line 159, then calls `enter_number(state, code)` which performs the lift internally, and then calls `apply_lift_effect(state, LiftEffect::Enable)` at line 161. `apply_lift_effect` with `Enable` sets `lift_enabled = true` again. This is functionally harmless (idempotent), but `enter_number` + `apply_lift_effect(Enable)` is the standard pattern: the manual `state.stack.lift_enabled = true` before `enter_number` is the real "always lift" guarantee. The trailing `apply_lift_effect(Enable)` is redundant but not wrong.

The pattern is consistent with `op_rcl` (which does the same three-step sequence), so this is a pre-existing pattern. However, the comment in `op_getkey` says "GETKEY always lifts (produces a new value)" without noting this is the established `op_rcl` pattern. A future reader may erroneously remove the `state.stack.lift_enabled = true` assignment thinking `apply_lift_effect(Enable)` is sufficient — but `apply_lift_effect(Enable)` runs AFTER `enter_number`, so the lift for the current call is driven by the pre-call `lift_enabled` state.

**Fix:** Add a comment clarifying the ordering, or consolidate to match the exact `op_rcl` pattern without the redundant trailing call. No functional change required.

---

### WR-02: `resume_program_with_key` does not reset `is_running` if `op_getkey` returns `Err`

**File:** `hp41-core/src/ops/program.rs:527`

**Issue:** The function calls `crate::ops::registers::op_getkey(state)?` using the `?` operator at line 527. If `op_getkey` returns an error (currently it always returns `Ok(())`, but this is a fragile assumption), the early return via `?` skips the `state.is_running = false` reset on line 535 because `is_running` is only set to `true` at line 533, AFTER the `op_getkey` call. So in practice this is currently safe because `is_running` is not yet `true` at line 527.

However, the current structure also skips `state.getkey_captured_code = None` cleanup if `op_getkey` errors. Specifically: `state.getkey_captured_code` is set at line 519, then `op_getkey` calls `.take()` on it (line 152 of `registers.rs`), consuming it. So after `op_getkey` returns either Ok or Err, `getkey_captured_code` is already `None` (because `.take()` consumed it). This is correct by coincidence — `.take()` on `Option` always leaves it as `None` regardless of the result of the subsequent logic.

The documentation comment says "Also cleans up `getkey_captured_code` even on error (Pitfall 6)" and points to line 536, but the actual cleanup happens implicitly via `.take()` in `op_getkey` itself. The explicit `state.getkey_captured_code = None` at line 536 is therefore a no-op (field is already `None`). This is a latent documentation/logic bug: the stated safety net at line 536 doesn't actually do the work it claims.

**Fix:** Document that `.take()` in `op_getkey` already consumes the field, and that the `None` assignment at line 536 is a belt-and-suspenders guard for any future refactor that changes the `take()` pattern:

```rust
// op_getkey already `.take()`s getkey_captured_code, leaving it None.
// This explicit assignment is a safety guard against future refactors.
state.getkey_captured_code = None;
```

---

### WR-03: `execute_op` arm for `Op::GetKey` would silently consume `last_key_code` if `GetKey` ever falls through from `run_loop`

**File:** `hp41-core/src/ops/program.rs:1036`

**Issue:** `execute_op` contains `Op::GetKey => super::registers::op_getkey(state)` at line 1036. In the current code this arm is unreachable during program execution because `run_loop` explicitly matches `Op::GetKey` at line 885 and breaks before the `other =>` arm. However, `execute_op` is also called by `execute_op_pub` (the `pub(crate)` shim used by `op_integ_run_loop`, `op_solve_run_loop`, and the XROM math solvers for user-callback re-entry).

If a user embeds `Op::GetKey` inside a solver callback (e.g., inside a function being integrated by INTG), the re-entrant `execute_op_pub` call would execute `op_getkey` immediately with `getkey_captured_code = None` (falling back to `last_key_code`) instead of suspending for user input. This is behaviorally wrong: GETKEY inside a solver callback should arguably surface as an error, not silently use `last_key_code`.

The real HP-41 does not support GETKEY inside a solver loop (the solver runs uninterrupted), so the correct behavior is `Err(HpError::InvalidOp)` from `execute_op` when `is_running` is true and `getkey_captured_code` is not set.

**Fix:**

```rust
Op::GetKey => {
    // GETKEY inside a solver callback or execute_op_pub re-entry:
    // only valid when getkey_captured_code is set (WaitForKey resume path).
    // Silently using last_key_code inside a solver is hardware-unfaithful.
    if state.getkey_captured_code.is_some() {
        super::registers::op_getkey(state)
    } else {
        Err(HpError::InvalidOp)
    }
}
```

Alternatively, the design decision to allow GETKEY in solver callbacks could be explicitly documented, but returning `InvalidOp` is more faithful to the HP-41.

---

## Info

### IN-01: `YieldView` in `types.rs` does not document the `"wait_for_key"` kind in its struct comment

**File:** `hp41-gui/src-tauri/src/types.rs:54-66`

**Issue:** The `YieldView` struct's doc comment at line 58 says `kind` is `"pse"` / `"view"` / `"aview"` but does not mention `"wait_for_key"`. This is a documentation gap that could cause a future TS developer to incorrectly assume the type union `"pse" | "view" | "aview"` is exhaustive (the TypeScript interface in `App.tsx:74` also defines `kind: string` rather than a discriminated union, so the gap is partly mitigated).

**Fix:** Update the struct doc comment:
```
/// `kind` is a lowercase string: `"pse"` / `"view"` / `"aview"` / `"wait_for_key"`.
/// `resume_ms` is 0 for `"wait_for_key"` (event-driven — no timer fires).
```

---

### IN-02: CLI inner poll loop does not drain print/event buffers after `resume_program_with_key` on Esc (cancel path)

**File:** `hp41-cli/src/app.rs:359-372`

**Issue:** When the user presses Esc during a WaitForKey suspend, the CLI calls `resume_program_with_key(&mut self.state, 0)` and then calls `self.drain_and_show_print_output(None)` at line 371. This is correct.

However, the success/error match at lines 364-370 does not call `self.drain_pending_card_op()` before `drain_and_show_print_output`. The resumed program (after receiving the sentinel keycode 0) might execute a card op. The same omission exists in the non-Esc HP-41-key capture path at lines 388-402. Contrast with the PSE/VIEW/AVIEW resume path at lines 427-437 which explicitly calls `drain_pending_card_op`.

This is a pre-existing omission in the PSE resume wiring that Phase 64 carries forward, but Phase 64 adds two new call sites (Esc path and key-capture path) that should both follow the full drain pattern.

**Fix:** Mirror the PSE resume path pattern for both GETKEY resume paths:

```rust
Ok(()) => {
    self.message = None;
    let card_err = self.drain_pending_card_op();
    self.drain_and_show_print_output(card_err);
}
```

---

_Reviewed: 2026-06-07T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
