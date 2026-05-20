---
phase: 31-gui-integration
reviewed: 2026-05-17T00:00:00Z
depth: standard
files_reviewed: 32
files_reviewed_list:
  - hp41-core/src/ops/math1/difeq.rs
  - hp41-core/src/ops/math1/integ.rs
  - hp41-core/src/ops/math1/solve.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/tests/cancel_flag_reset_on_open.rs
  - hp41-core/tests/op_catalog_xrom.rs
  - hp41-gui/src-tauri/capabilities/default.json
  - hp41-gui/src-tauri/permissions/cancel-modal.toml
  - hp41-gui/src-tauri/permissions/request-cancel.toml
  - hp41-gui/src-tauri/permissions/submit-modal-with-label.toml
  - hp41-gui/src-tauri/permissions/submit-modal.toml
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-gui/src-tauri/tests/cancel_autosave_stress.rs
  - hp41-gui/src-tauri/tests/cancel_command_no_deadlock.rs
  - hp41-gui/src-tauri/tests/d25_6_parity.rs
  - hp41-gui/src-tauri/tests/key_map_stub_error_arms.rs
  - hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs
  - hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs
  - hp41-gui/src-tauri/tests/sc4_invariant.rs
  - hp41-gui/src/App.css
  - hp41-gui/src/App.test.tsx
  - hp41-gui/src/App.tsx
  - hp41-gui/src/Display14Seg.test.tsx
  - hp41-gui/src/Display14Seg.tsx
  - hp41-gui/src/HelpOverlay.test.tsx
  - hp41-gui/src/HelpOverlay.tsx
  - hp41-gui/src/help_data.ts
  - hp41-gui/src/pending_input.test.ts
  - hp41-gui/src/pending_input.ts
  - scripts/check-tauri-permissions.sh
findings:
  critical: 0
  warning: 5
  info: 6
  total: 11
status: issues_found
---

# Phase 31: Code Review Report

**Reviewed:** 2026-05-17
**Depth:** standard
**Files Reviewed:** 32
**Status:** issues_found

## Summary

Phase 31 wires the existing Phase 28 hp41-core Math Pac I implementation through to the Tauri GUI. The review covers 5 plans across 3 waves: new Tauri commands (`request_cancel`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`), CancelFlag managed state for deadlock-free cancellation, CalcStateView modal field projection, LCD-alternation modal-prompt rendering, frontend R/S 3-way + Esc 4-way cascade, and a surgical hp41-core `op_catalog` XROM enumeration.

**Adversarial verdict:** No CRITICAL defects. The 6 Pitfalls and 6 D-decisions called out in the request are all implemented correctly. The CancelFlag/AppState separation (Pitfall 1) is sound — `request_cancel` reads `State<'_, CancelFlag>` only, the deadlock test asserts it, and the stress test exercises concurrent autosave + cancel. `HpError::Canceled` correctly maps to UPPERCASE `"CANCELED"` (Pitfall 4). All 9 generate_handler! commands have matching permission TOMLs (verified by ls). LCD-alternation routing places the new branch FIRST in the priority chain (D-31.5), entry_buf still wins on user typing.

The hp41-core surgical exceptions (cancel_requested resets in 3 dispatch arms; `op_catalog` Math 1 enumeration) stay within scope — no API widening, no new pub functions, no new `Op` variants. Test coverage is generous: 7 integration tests in `hp41-gui/src-tauri/tests/` plus 18 new test cases in `App.test.tsx` group H + `pending_input.test.ts`. The 4-exhaustive-match invariant is locked behind both the prgm_display.rs file-text-scan test and the stub_error_message_count baseline lock.

5 WARNING-class findings around lost diagnostic output on cancel, the integ.rs print_buffer pre-mutation before label scan, the check-tauri-permissions.sh empty-file false-positive, the lack of a permission-TOML schema check for orphan TOMLs, and a missing call_stack restoration on the rk4_step_orderN error path. 6 INFO items cover doc/comment drift, magic prefix collision risk, and minor test gaps.

## Warnings

### WR-01: `dispatch_op` does not drain `print_buffer` on `HpError::Canceled`

**File:** `hp41-gui/src-tauri/src/commands.rs:195`
**Issue:** `handle_op_prepare` calls `dispatch(calc, op).map_err(GuiError::from)?;`. When INTG/SOLVE/DIFEQ returns `Err(HpError::Canceled)` mid-loop, the `?` returns early. The print_buffer entries accumulated during partial computation (e.g. DIFEQ's per-step `X=... Y=...` lines, INTG's `N rounded to ...` warning) are NEVER drained — they leak into the next `get_state` or `dispatch_op` response, appearing as phantom output from the cancelled run. For DIFEQ specifically, the diagnostic value of these per-step lines is high (they're literally the user-visible output the OM documents).

**Fix:** Pattern-match the dispatch result and drain `print_buffer`/`event_buffer` into the returned view (or the GuiError) even on `Err`:
```rust
let dispatch_result = dispatch(calc, op);
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
match dispatch_result {
    Ok(()) => { /* continue with pending_card_op */ }
    Err(HpError::Canceled) => {
        // Preserve partial output: return a view-bearing error path,
        // or attach print_lines to the GuiError message.
        return Err(GuiError::from(HpError::Canceled));
    }
    Err(e) => return Err(GuiError::from(e)),
}
```
A simpler interim fix: ALWAYS drain at function exit (use a `defer`-style pattern via a scope guard or a `let print_lines = ...; match dispatch_result { ... }` reorder).

### WR-02: `op_integ_run_loop` pushes "N rounded" line BEFORE label-not-found check

**File:** `hp41-core/src/ops/math1/integ.rs:263-288`
**Issue:** When `n` is odd, the function pushes `"N rounded to {bumped} (Simpson requires even)"` to `print_buffer` at line 267. The user-label scan happens at line 285-288 and can return `InvalidOp` early. If the label is missing, the user sees a "N rounded" line in the print panel even though integration never ran. This is a state-leak across the failure path — minor user-confusion bug.

**Fix:** Move the label scan above the odd-N print_buffer mutation, OR delay the print_buffer.push until after label_pos is resolved:
```rust
let label_pos = program
    .iter()
    .position(|op| matches!(op, Op::Lbl(l) if *l == user_label))
    .ok_or(HpError::InvalidOp)?;
// Now safe to mutate print_buffer:
let n_even = if n % 2 == 1 { ... };
```

### WR-03: `check-tauri-permissions.sh` false-positive on empty/missing handler file

**File:** `scripts/check-tauri-permissions.sh:16-29`
**Issue:** The script extracts commands via `grep -oE 'commands::[a-z_]+'`. If `HANDLER_FILE` is missing or contains no `commands::*` references (refactor accident or file rename), the `commands` variable is empty. The `for cmd in $commands` loop then iterates zero times, `missing` stays at 0, the script prints `OK: all 0 commands have permission TOMLs` and exits 0 — passing CI while actually validating NOTHING. The line `wc -w | tr -d ' '` would print "0" but the success exit code masks it.

Also: the regex `[a-z_]+` does NOT match digits or uppercase, so a future command like `v3_dispatch` or `dispatchV3` would be silently missed. Unlikely in this codebase (Rust snake_case + no digits), but worth fixing.

**Fix:** Assert at least one command is found, and broaden the regex:
```bash
commands=$(grep -oE 'commands::[a-zA-Z0-9_]+' "$HANDLER_FILE" | sed 's/commands:://' | sort -u)
if [[ -z "$commands" ]]; then
    echo "ERROR: no commands found in $HANDLER_FILE — pattern broken or file empty"
    exit 2
fi
```

### WR-04: No reverse check — orphan permission TOMLs are not detected

**File:** `scripts/check-tauri-permissions.sh:14-29`
**Issue:** The script asserts every `commands::xxx` reference has a matching `permissions/<kebab>.toml`, but the reverse is not enforced: if a future commit DELETES a `commands::xxx` from `generate_handler!` but leaves the `permissions/xxx.toml` behind, the script reports OK. The orphan TOML is dead config that lingers indefinitely — minor maintainability issue, and a defense-in-depth gap (declared permissions for ops that no longer exist could be a foothold if `tauri::generate_context!` ever reads them for some other purpose).

**Fix:** Add a reverse-direction loop:
```bash
for tomlpath in "$PERMS_DIR"/*.toml; do
    kebab=$(basename "$tomlpath" .toml)
    snake=$(echo "$kebab" | sed 's/-/_/g')
    if ! grep -q "commands::$snake" "$HANDLER_FILE"; then
        echo "ORPHAN: $tomlpath  (no matching commands::$snake)"
        missing=$((missing + 1))
    fi
done
```

### WR-05: `rk4_step_order2` / `rk4_step_order1` user-callback Err paths do not clear `difeq_state`

**File:** `hp41-core/src/ops/math1/difeq.rs:451-465, 479-495` (and parallel locations)
**Issue:** Inside `rk4_step_order1` / `rk4_step_order2`, the per-`k_i` evaluation pushes onto `call_stack` and runs `run_user_function`. On Err, the cleanup is:
```rust
while state.call_stack.len() > save_call_stack_len {
    state.call_stack.pop();
}
r?;  // propagate Err
```
The `?` returns the user-callback error up to `op_difeq_run_loop`, where the loop body's match at line 403-414 DOES restore `pc` and clear `difeq_state`. BUT — `save_pc` for `op_difeq_run_loop` is captured at line 298, AFTER `state.difeq_state = Some(...)`. If `rk4_step_order1` returns Err, the outer caller's cleanup at 405-413 restores pc but `save_pc` is correct (captured before the loop). This is currently OK — no bug, just brittle: a future refactor that adds a `?` propagation between the difeq_state assignment (line 258-271) and the save_pc capture (line 298) would leak `difeq_state = Some(...)` permanently and the next DIFEQ would hit the XROM-08 nested-rejection guard. The CR-01 fix that integ.rs documents (lines 296-307) is not applied symmetrically to difeq.rs.

**Fix:** Hoist `save_pc` and `save_call_stack_len` capture to BEFORE the `state.difeq_state = Some(...)` assignment (currently lines 258, 298, 299 — reorder to 298, 299 before 258). Alternatively, document the invariant explicitly as a doc-comment so future refactors don't break it. The same pattern should be checked in solve.rs (`run_secant_loop` at 320-321 vs `state.solve_state = Some(...)` at 211 / 277). solve.rs has `state.solve_state = None` clearing in the eval_fn closure (line 363), so it's already defensively correct — but difeq.rs lacks an equivalent in `rk4_step_orderN`.

## Info

### IN-01: Doc comment in `integ.rs` documents a CR-01 fix pattern that `difeq.rs` does not follow symmetrically

**File:** `hp41-core/src/ops/math1/integ.rs:296-302`, `hp41-core/src/ops/math1/difeq.rs` (no analogous comment)
**Issue:** The integ.rs CR-01 doc comment lays out a careful "every `?`-conversion between state.integ_state = Some(...) and state.integ_state = None MUST clear integ_state" invariant. difeq.rs has the same shape (set difeq_state, then ?-prone f64 conversions) but no equivalent doc comment. Future refactors of difeq.rs would lose this institutional knowledge.
**Fix:** Copy the CR-01-style doc-comment into difeq.rs op_difeq_run_loop above the `state.difeq_state = Some(...)` block.

### IN-02: Magic-prefix `__submit_modal_with_label__` is not collision-checked against future label content

**File:** `hp41-gui/src/pending_input.ts:538`, `hp41-gui/src/App.tsx:80-83`
**Issue:** `SUBMIT_MODAL_WITH_LABEL_PREFIX = '__submit_modal_with_label__'`. The slice extracts everything after the prefix as the label. If a user types a label that happens to start with that prefix (extremely unlikely, but theoretically possible via clipboard paste), the slice would chop off the leading "label" chars. Same risk exists with the older `__keycode__` prefix. HP-41 label charset is uppercase A-Z + digits + a few punct symbols, none of which start with `_`, so the practical collision probability is zero. Still worth a comment.
**Fix:** Add a doc comment to `SUBMIT_MODAL_WITH_LABEL_PREFIX` noting that HP-41 labels never start with `_`, so the magic-prefix is collision-free in practice.

### IN-03: `op_catalog` Catalog(2) helper comment shows "MATH 1A" but produces "MATH 1"

**File:** `hp41-core/tests/op_catalog_xrom.rs:50-53`
**Issue:** The test assertion's failure message reads `"Module header should contain 'MATH 1A'"` but the actual `assert!` checks for `"MATH 1"` (no `A`). Cosmetic inconsistency — the failure message would mislead a maintainer debugging a regression. The actual module name (`MATH_1.name` in `xrom.rs`) is `"MATH 1A"` per HP-41C hardware nameplate, but this test only checks the substring `"MATH 1"`.
**Fix:** Either tighten the assertion to `assert!(module_header.contains("MATH 1A"))` or update the message to match the substring being checked.

### IN-04: `setExpanded` reset on `open` does not preserve user preference

**File:** `hp41-gui/src/HelpOverlay.tsx:62-67`
**Issue:** Every time the `?` overlay opens, both sections are forced back to expanded=true. A user who prefers Math 1 Pac collapsed will see it expanded every time they open the overlay. This is a UX preference, not a defect — the comment says "clean-slate UX" which is the intentional design. Minor: not a bug.
**Fix:** None required unless user feedback shifts; consider a `localStorage` persistence in a future polish phase.

### IN-05: `commands.rs::cancel_modal` returns `Result` but the core fn cannot fail

**File:** `hp41-gui/src-tauri/src/commands.rs:308-312`
**Issue:** `hp41_core::ops::math1::cancel_modal(&mut calc)` returns `()` (always succeeds). The Tauri thunk wraps it in `Result<CalcStateView, GuiError>` for IPC consistency with other modal commands. The Result return is harmless but slightly misleading — a future maintainer might add a `?` propagation thinking cancel_modal can fail.
**Fix:** None required; the IPC-consistency rationale is documented in the doc-comment ("Always succeeds").

### IN-06: `cancel_autosave_stress.rs` uses `unwrap()` for `handle_a.join()` etc. instead of catching the `JoinError`

**File:** `hp41-gui/src-tauri/tests/cancel_autosave_stress.rs:140-142`
**Issue:** `.join().expect("Thread A (long compute) must not panic catastrophically")` — if the worker thread panics, the expect message fires, but the panic_unwind catch_unwind inside each thread should have already converted panics to flag stores. The expect path is unreachable in practice. Defensive code, not a defect.
**Fix:** None required. The test correctness is fine; this is just dead defensive code.

---

## Verification Notes (Phase 31 specific checks)

**Pitfall 1 (request_cancel must not lock AppState):** VERIFIED. `commands.rs:280` declares `request_cancel(cancel: State<'_, CancelFlag>)` — separate managed state, not `State<'_, AppState>`. `lib.rs:58-61` clones the `Arc<AtomicBool>` from `initial_state.cancel_requested` BEFORE wrapping in the Mutex, and manages both `Mutex<CalcState>` and the cloned `CancelFlag` Arc separately via `app.manage()`. The `cancel_command_no_deadlock.rs` test asserts this with a 100ms timeout while AppState is held; `cancel_autosave_stress.rs` exercises 3-thread concurrent autosave + cancel + long-compute. Sound.

**Pitfall 4 (HpError::Canceled → uppercase "CANCELED"):** VERIFIED. `types.rs:236-243` `From<HpError>` impl pattern-matches `HpError::Canceled => "CANCELED"`. Asserted by `types.rs:371-378` test_canceled_maps_to_uppercase.

**Pitfall 10 (CalcStateView::from_state projects 4 modal fields):** VERIFIED. `types.rs:94-104` declares `is_running`, `modal_program_active`, `modal_requires_alpha_label`, `modal_prompt`. `types.rs:193-201` projects them. `test_modal_fields_default_projection` covers fresh-state defaults; integration test `lcd_alternation_modal_prompt.rs` covers modal_prompt rendering. JSON budget raised from 500→600 bytes for realistic load.

**Pitfall 15 (PendingInput::XeqByName mode + magic-prefix routing):** VERIFIED. `pending_input.ts:62` adds optional `mode?: 'normal' | 'collect-for-modal'`. `pending_input.ts:348-369` Enter dispatch branches on `mode` to produce either `xeq_<acc>` or `__submit_modal_with_label__<acc>`. `App.tsx:80-83` invokeForKey checks the magic prefix BEFORE the `r_s` branch. Tests at `pending_input.test.ts:506-590` cover both modes including Backspace preserving mode.

**D-31.1 (R/S 3-way state-routed):** VERIFIED. `App.tsx:86-99` routes `r_s` keypress: modal_program_active → submit_modal; is_running → request_cancel + get_state; else → run_stop. `App.test.tsx` H1/H2/H3 cover all three branches.

**D-31.2 (Esc 4-way cascade):** VERIFIED. `App.tsx:474-509` cascade: helpOpen → close help; pendingInput → close modal+clear shift; modal_program_active → cancel_modal; is_running → request_cancel+get_state; shiftActive → clear. Comment correctly states "NEVER silently discard — D-07 holds" but the cascade falls through silently when ALL conditions are false — this is the intended behavior (Esc has no effect when nothing is active). `App.test.tsx` H4 covers the cancel_modal branch.

**D-31.5/D-31.6 (LCD-alternation):** VERIFIED. `types.rs:130-146` places modal-prompt routing FIRST in display_str priority chain (BEFORE entry_buf, alpha_reg, format_hpnum). `truncate_with_continuation` uses Unicode-correct char iteration. 5 integration tests in `lcd_alternation_modal_prompt.rs` cover ≤12/=12/13/14-char prompts and entry_buf-overrides-prompt.

**hp41-core surgical exceptions (cancel_requested resets):** VERIFIED. Single-line `state.cancel_requested.store(false, Relaxed)` added to the `!is_running` branch of `op_integ` (integ.rs:169), `op_solve` (solve.rs:125), `op_difeq` (difeq.rs:148). No new pub functions, no new Op variants, no API widening. 3 tests in `cancel_flag_reset_on_open.rs` lock the invariant per dispatch arm.

**hp41-core surgical exception (op_catalog XROM enumeration):** VERIFIED. `program.rs:343-353` enumerates `MATH_1.ops` when `xrom_modules & 0b0000_0001 != 0`, falls back to "NO XROM" when no modules. CAT 3/4 still emit "NOT AVAILABLE" (program.rs:355-358) — regression test `catalog_3_and_4_still_not_available` covers this.

**Permission coverage (every command in generate_handler! has a TOML):** VERIFIED. `ls permissions/` shows 9 TOMLs: bst-step, cancel-modal, dispatch-op, get-state, request-cancel, run-stop, sst-step, submit-modal, submit-modal-with-label. `lib.rs:83-92` registers exactly 9 commands. `capabilities/default.json` lists all 9 `allow-*` identifiers.

**SC-4 invariant:** VERIFIED. `sc4_invariant.rs` integration test runs the strict grep at runtime. No new `op_*` math helpers in hp41-gui/src-tauri/src/.

**D-25.6 CLI↔GUI parity:** VERIFIED via `d25_6_parity.rs` (3 tests: SINH, ASINH, TANH route identically via xrom_resolve and run_program → Op::Xeq paths).

---

_Reviewed: 2026-05-17_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
