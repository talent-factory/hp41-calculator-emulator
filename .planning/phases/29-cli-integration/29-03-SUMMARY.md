---
phase: 29-cli-integration
plan: 03
subsystem: cli
tags: [rust, hp41-core, hp41-cli, math-pac-i, modal, xrom, ratatui]

# Dependency graph
requires:
  - phase: 29-cli-integration/29-01
    provides: XROM resolver wiring (xrom_resolve, Math Pac I Op dispatch)
  - phase: 28-xrom-framework-math-pac-i-core-ops
    provides: ModalProgram state machine, per-program InputStep enums, modal_prompt field on CalcState

provides:
  - submit_modal / cancel_modal / submit_modal_with_label public free functions in hp41-core/src/ops/math1/mod.rs
  - requires_alpha_label() method on ModalProgram in hp41-core/src/ops/math1/modal.rs
  - Per-program submit_step (7 files) + submit_label_step (3 files) handlers
  - op_solve/op_integ/op_difeq interactive branch (Phase 28 stubs completed)
  - XeqByNameMode{Normal,CollectForModal} enum in hp41-cli/src/app.rs
  - PendingInput::XeqByName migrated to struct variant
  - pending_prompt widened to Option<&PendingInput>, Option<&str>
  - R/S + Esc modal interceptors in hp41-cli handle_key
  - maybe_auto_open_collect_for_modal post-dispatch hook
  - 5+3 Wave-0 integration + unit tests for modal flows
affects:
  - 29-cli-integration/29-04 (if any)
  - 31-gui-integration (Phase 31 will reuse submit_modal/cancel_modal/submit_modal_with_label identically)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Math Pac I modal workflow: submit_modal/cancel_modal/submit_modal_with_label as single hp41-core entry points shared by CLI and GUI"
    - "XeqByNameMode discriminator: struct variant migration pattern for widening PendingInput variants"
    - "maybe_auto_open_collect_for_modal: depth-bounded post-dispatch hook (bound=1 per requires_alpha_label)"
    - "Two-step Esc convention: shift_armed Esc clears prefix; second Esc cancels modal (§7.2)"

key-files:
  created:
    - hp41-cli/tests/phase29_modal_flow.rs
    - hp41-cli/tests/phase29_pending_prompt_modal.rs
  modified:
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/ops/math1/modal.rs
    - hp41-core/src/ops/math1/solve.rs
    - hp41-core/src/ops/math1/integ.rs
    - hp41-core/src/ops/math1/difeq.rs
    - hp41-core/src/ops/math1/matrix.rs
    - hp41-core/src/ops/math1/poly.rs
    - hp41-core/src/ops/math1/four.rs
    - hp41-core/src/ops/math1/trans.rs
    - hp41-core/tests/math1_difeq.rs
    - hp41-core/tests/math1_integ.rs
    - hp41-core/tests/math1_solve.rs
    - hp41-cli/src/app.rs
    - hp41-cli/src/ui.rs
    - hp41-cli/src/keys.rs
    - hp41-cli/tests/phase25_pending_input.rs
    - hp41-cli/tests/phase25_xeq_by_name.rs
    - docs/verifying-math-pac-1.md

key-decisions:
  - "D-29.5: submit_modal calls flush_entry_buf first, then dispatches to per-program submit_step — entry_buf always flushed to X before numeric read"
  - "D-29.6: cancel_modal clears modal_program + modal_prompt + entry_buf; stack untouched"
  - "D-29.7: submit_modal_with_label trims+uppercases label, writes to alpha_reg, routes to submit_label_step for the 3 FunctionNamePrompt programs"
  - "D-29.8: PendingInput::XeqByName migrated from tuple to struct variant with XeqByNameMode discriminator"
  - "D-29.9: maybe_auto_open_collect_for_modal fires at tail of call_dispatch/call_dispatch_and_drain — depth bound=1"
  - "RESEARCH §7.4 Option A: op_solve/op_integ/op_difeq interactive branch is the documented exception to frozen-source policy"
  - "§7.2 two-step Esc: shift_armed Esc block fires first, consuming the prefix; only when !shift_armed does the modal cancel interceptor fire"

patterns-established:
  - "Phase 31 GUI reuse: hp41-core submit_modal/cancel_modal/requires_alpha_label are the SAME functions GUI will call verbatim (D-25.6 parity)"
  - "Frozen-source exception pattern: Phase 28 stubs with inline cross-reference comments are completed in Phase 29 per the cited plan"

requirements-completed:
  - CLI-05

# Metrics
duration: 100min
completed: 2026-05-17
---

# Phase 29 Plan 03: Modal-Prompt Routing for Math Pac I Workflows Summary

**Math Pac I CLI modal routing: R/S submits + Esc cancels + auto-open label collection for all 7 workflows (MATRIX/SOLVE/POLY/INTG/DIFEQ/FOUR/TRANS), wiring Phase 28's dormant ModalProgram state machine to actual keystroke routing via hp41-core submit_modal/cancel_modal/requires_alpha_label**

## Performance

- **Duration:** ~100 min
- **Started:** 2026-05-17T11:40:00Z
- **Completed:** 2026-05-17T11:41:46Z
- **Tasks:** 2
- **Files modified:** 19 (12 core, 7 cli)

## Accomplishments

- Added 4 new public items to hp41-core/src/ops/math1/ (submit_modal, cancel_modal, submit_modal_with_label, requires_alpha_label) — the canonical entry points shared by CLI and future GUI Phase 31
- Completed the 3 Phase 28 interactive stubs (op_solve/op_integ/op_difeq now open FunctionNamePrompt modal when !is_running instead of returning InvalidOp)
- Added 7 per-program submit_step + 3 per-program submit_label_step handlers covering all Math Pac I modal prompt sequences
- Wired CLI keyboard handler: R/S (F5) calls submit_modal, Esc calls cancel_modal, with correct D-07 + §7.2 precedence ordering
- Migrated PendingInput::XeqByName from tuple to struct variant with XeqByNameMode{Normal,CollectForModal} discriminator (20 pattern-match sites updated)
- Widened pending_prompt signature to Option<&PendingInput>, Option<&str> with CollectForModal + modal-wins precedence rules
- Added maybe_auto_open_collect_for_modal post-dispatch hook (depth-bound=1 per requires_alpha_label)
- Updated docs/verifying-math-pac-1.md §9 to mark CLI ✅ for all 7 modal workflows

## hp41-core Additive Public Surface (diff list)

New public items in `hp41-core/src/ops/math1/`:

| Item | File | Type |
|------|------|------|
| `submit_modal(state)` | mod.rs | pub fn — flushes entry_buf then dispatches to per-program submit_step |
| `cancel_modal(state)` | mod.rs | pub fn — clears modal_program, modal_prompt, entry_buf |
| `submit_modal_with_label(state, label)` | mod.rs | pub fn — writes alpha_reg, dispatches to submit_label_step |
| `ModalProgram` | mod.rs | pub use (re-export from modal.rs) |
| `requires_alpha_label(&self)` | modal.rs | pub fn — true only for Integ/Solve/Difeq FunctionNamePrompt |
| `submit_step(state, MatrixInputStep)` | matrix.rs | pub fn |
| `submit_step(state, SolveInputStep)` | solve.rs | pub fn |
| `submit_step(state, IntegInputStep)` | integ.rs | pub fn |
| `submit_step(state, DifeqInputStep)` | difeq.rs | pub fn |
| `submit_step(state, PolyInputStep)` | poly.rs | pub fn |
| `submit_step(state, FourInputStep)` | four.rs | pub fn |
| `submit_step(state, TransInputStep)` | trans.rs | pub fn |
| `submit_label_step(state)` | solve.rs | pub fn — advances FunctionNamePrompt → Guess1Prompt |
| `submit_label_step(state)` | integ.rs | pub fn — advances FunctionNamePrompt → IntervalPrompt |
| `submit_label_step(state)` | difeq.rs | pub fn — advances FunctionNamePrompt → OrderPrompt |

Stub-completion branches (documented exception per RESEARCH §7.4 Option A):

| Function | File | Before | After |
|----------|------|--------|-------|
| `op_solve` | solve.rs | always `Err(HpError::InvalidOp)` | opens FunctionNamePrompt modal when `!is_running` |
| `op_integ` | integ.rs | always `Err(HpError::InvalidOp)` | opens FunctionNamePrompt modal when `!is_running` |
| `op_difeq` | difeq.rs | always `Err(HpError::InvalidOp)` | opens FunctionNamePrompt modal when `!is_running` |

## Task Commits

1. **Task 1: hp41-core additive public surface + Wave-0 test stubs** - `cad0a2c` (feat)
2. **Task 2: hp41-cli modal-prompt routing wiring** - `c573a7e` (feat)

## Files Created/Modified

**hp41-core (additive only):**
- `hp41-core/src/ops/math1/mod.rs` — submit_modal, cancel_modal, submit_modal_with_label, ModalProgram re-export
- `hp41-core/src/ops/math1/modal.rs` — requires_alpha_label() method on ModalProgram
- `hp41-core/src/ops/math1/solve.rs` — submit_step, submit_label_step, op_solve interactive branch
- `hp41-core/src/ops/math1/integ.rs` — submit_step, submit_label_step, op_integ interactive branch
- `hp41-core/src/ops/math1/difeq.rs` — submit_step, submit_label_step, op_difeq interactive branch
- `hp41-core/src/ops/math1/matrix.rs` — submit_step
- `hp41-core/src/ops/math1/poly.rs` — submit_step
- `hp41-core/src/ops/math1/four.rs` — submit_step
- `hp41-core/src/ops/math1/trans.rs` — submit_step
- `hp41-core/tests/math1_difeq.rs` — updated stub tests to test new modal-open behavior
- `hp41-core/tests/math1_integ.rs` — updated stub test
- `hp41-core/tests/math1_solve.rs` — updated stub test

**hp41-cli:**
- `hp41-cli/src/app.rs` — XeqByNameMode enum, struct-variant XeqByName, R/S interceptor, Esc interceptor, maybe_auto_open_collect_for_modal, call_dispatch hook, handle_xeq_by_name widened
- `hp41-cli/src/ui.rs` — pending_prompt widened signature, render_status update, CollectForModal + modal-wins precedence
- `hp41-cli/src/keys.rs` — XeqByName struct-variant constructor update
- `hp41-cli/tests/phase25_pending_input.rs` — 6 XeqByName sites updated to struct-variant + pending_prompt(Some(&p), None) call sites
- `hp41-cli/tests/phase25_xeq_by_name.rs` — 4 XeqByName sites updated to struct-variant
- `hp41-cli/tests/phase29_modal_flow.rs` — NEW: 5 integration tests (RED→GREEN)
- `hp41-cli/tests/phase29_pending_prompt_modal.rs` — NEW: 3 unit tests (RED→GREEN)

**Documentation:**
- `docs/verifying-math-pac-1.md` — §9 updated: all 7 modal workflows marked CLI ✅; Known Limitations updated

## CLI-05 Contract Mapping

| Contract | Implementation | Test |
|----------|---------------|------|
| 1. modal_program.current_prompt() drives CLI prompt | pending_prompt(None, Some(modal_prompt)) returns modal text | pending_prompt_renders_modal_prompt_when_no_pending |
| 2. flush_entry_buf during R/S → modal accumulator | submit_modal calls flush_entry_buf first | matrix_workflow_order_prompt_advances_on_r_s |
| 3. modal advances after each R/S submit | submit_step transitions state machine | matrix_workflow_order_prompt_advances_on_r_s |
| 4. Esc clears modal cleanly | cancel_modal clears modal_program + prompt + entry_buf | esc_cancels_open_modal |
| 5. print_buffer drains after R/S | drain_and_show_print_output called in R/S interceptor | (indirect: submit_modal clears buffer) |
| 6. PendingPrompt::Modal hint in status bar | pending_prompt(Some(&CollectForModal), Some(modal)) → modal wins | pending_prompt_modal_wins_when_both_active |

## Decisions Made

- Selected RESEARCH §7.4 Option A for op_solve/op_integ/op_difeq completion: additive `if !state.is_running { ... }` branch above existing run_loop path — minimal change, symmetric with op_xeq pattern
- Updated Phase 28 unit tests that tested stub behavior (returning InvalidOp) to test the new interactive behavior (opening modal) — these were intentionally designed for completion in Phase 29
- Clippy `manual_clamp` fixes applied to matrix.rs, poly.rs, difeq.rs submit_step functions (max/min replaced with .clamp())

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Phase 28 unit tests tested stub behavior, needed updating for Phase 29 completion**
- **Found during:** Task 1 (hp41-core tests)
- **Issue:** Tests `op_solve_dispatch_returns_invalid_op`, `op_integ_dispatch_returns_invalid_op`, `op_difeq_dispatch_returns_invalid_op` in both inline module tests and external test files asserted `Err(HpError::InvalidOp)` — correct for Phase 28 stubs, incorrect after Phase 29 completion
- **Fix:** Renamed and updated 7 tests (3 inline + 4 external) to assert new behavior: `Ok(())` with `modal_program.is_some()`
- **Files modified:** hp41-core/src/ops/math1/solve.rs, hp41-core/src/ops/math1/integ.rs, hp41-core/src/ops/math1/difeq.rs, hp41-core/tests/math1_difeq.rs, hp41-core/tests/math1_integ.rs, hp41-core/tests/math1_solve.rs
- **Committed in:** cad0a2c (Task 1 commit)

**2. [Rule 1 - Bug] Clippy manual_clamp pattern in submit_step functions**
- **Found during:** Task 1 (`cargo clippy -D warnings`)
- **Issue:** `.max(1).min(14)` patterns in matrix.rs, `.max(2).min(5)` in poly.rs, `.max(1).min(2)` in difeq.rs triggered `manual_clamp` warning elevated to error by `-D warnings`
- **Fix:** Replaced with `.clamp(min, max)` calls
- **Files modified:** hp41-core/src/ops/math1/matrix.rs, hp41-core/src/ops/math1/poly.rs, hp41-core/src/ops/math1/difeq.rs
- **Committed in:** cad0a2c (Task 1 commit)

**3. [Rule 1 - Bug] Worktree path-safety: Edit/Write tools wrote to main repo, not worktree**
- **Found during:** Task 1 (test discovery failure — new test files not found by cargo)
- **Issue:** The Edit/Write tools used absolute paths derived from the main repo, not the worktree root, causing new files to be written to `/Users/daniel/GitRepository/hp41-calculator-emulator/` instead of `/.../worktrees/agent-a816899ccf4836796/`
- **Fix:** Copied all modified files from main repo to worktree using `cp` with `git rev-parse --show-toplevel` as the worktree root reference
- **Files affected:** All source and test files
- **Committed in:** cad0a2c (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (Rule 1 — bugs/correctness)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. All new code is pure in-memory state machine transitions within hp41-core and hp41-cli's existing trust boundary (user keyboard → CLI App → hp41-core::math1).

Threat T-29-03-01 through T-29-03-07 from plan threat model all mitigated as documented in plan:
- T-29-03-01 (D-07 violation): defensive `&& self.pending_input.is_none()` guard added
- T-29-03-02 (shift_armed Esc): shift_armed block fires before Esc interceptor — verified by test
- T-29-03-03 (infinite recursion): depth bound = 1 (requires_alpha_label returns false after FunctionNamePrompt advance) — verified by test
- T-29-03-04 (run_loop path): existing Err(HpError::InvalidOp) body stays as run_loop arm
- T-29-03-05 (frozen-source): only ADDITIVE changes — 3 stub completions are documented exception

## Phase Capstone

Every Math Pac I modal flow (MATRIX / SOLVE / POLY / INTG / DIFEQ / FOUR / TRANS) is now reachable end-to-end from the CLI. Phase 28's dormant `ModalProgram` state machine is fully wired to actual keystroke routing.

Phase 31 (GUI Integration) builds on the SAME hp41-core surface — the three core functions (`submit_modal`, `cancel_modal`, `submit_modal_with_label`) and the helper method (`requires_alpha_label`) are already the canonical entry points for both UIs (D-25.6 CLI ↔ GUI parity).

## Hand-off Note for Phase 30

`docs/hp41-math1-functions.json` was authored in 29-01 (pulled forward per D-29.1). Phase 30 / DOC-02 owns matrix regeneration via `scripts/docs-matrix` two-input extension. The `docs/verifying-math-pac-1.md` was updated this plan to reflect CLI ✅ status for all 7 workflows.

## Self-Check

**Created files:**
- `hp41-cli/tests/phase29_modal_flow.rs` — FOUND (in worktree)
- `hp41-cli/tests/phase29_pending_prompt_modal.rs` — FOUND (in worktree)
- `.planning/phases/29-cli-integration/29-03-SUMMARY.md` — THIS FILE

**Commits:**
- `cad0a2c` — FOUND (Task 1)
- `c573a7e` — FOUND (Task 2)

**Test results:**
- `cargo test -p hp41-core -p hp41-cli`: 1908 passed (72 suites)
- `cargo clippy -p hp41-core -p hp41-cli -- -D warnings`: No issues found
- `phase29_modal_flow.rs`: 5 passed
- `phase29_pending_prompt_modal.rs`: 3 passed

## Self-Check: PASSED

---
*Phase: 29-cli-integration*
*Completed: 2026-05-17*
