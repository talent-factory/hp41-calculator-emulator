---
phase: 29-cli-integration
fixed_at: 2026-05-17T15:45:00Z
review_path: .planning/phases/29-cli-integration/29-REVIEW.md
iteration: 1
findings_in_scope: 17
fixed: 17
skipped: 0
status: all_fixed
---

# Phase 29: Code Review Fix Report

**Fixed at:** 2026-05-17T15:45:00Z
**Source review:** `.planning/phases/29-cli-integration/29-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 17 (4 Critical, 4 Blocker, 9 Warning)
- Fixed: 17
- Skipped: 0
- Out of scope: 5 Info findings (would require `--all`)

All in-scope findings were applied successfully. The full test suite (`just test`, 72 suites) and lint (`just lint`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`) pass green after every commit.

Two pairs of findings were committed together because they touched the same function and share a single logical fix: BL-01 + BL-02 (TRANS modal capture + transform invocation in `trans::submit_step`) and WR-01 + WR-02 (modal guard reordering in `submit_modal` and `submit_modal_with_label`).

## Fixed Issues

### CR-01: `op_integ_run_loop` leaks `integ_state` on Overflow paths

**Files modified:** `hp41-core/src/ops/math1/integ.rs`
**Commit:** 5da4183
**Applied fix:** Replaced four `Decimal::from_f64(_).ok_or(Overflow)?` / `to_f64().ok_or(Overflow)?` chain calls between the `state.integ_state = Some(_)` assignment and the success-path `state.integ_state = None` cleanup with explicit `match` arms that clear `integ_state` (and restore `pc` where applicable) on the Err arm. INTG can no longer be permanently poisoned by a one-off Overflow during a normal session.

### CR-02: INTG modal/run_loop parameter disconnect (R02/R03/R04 vs X/Y/R00)

**Files modified:** `hp41-core/src/ops/math1/integ.rs`, `hp41-core/tests/math1_integ.rs`
**Commit:** e071a88
**Applied fix:** Picked the modal-side of the contract per the project guidance. `submit_step(IntervalPrompt)` now swaps X and Y so the run_loop's stack-X-as-`a` / stack-Y-as-`b` reads see the right bounds. `submit_step(SubdivisionPrompt)` now stores N in R00 (where the run_loop reads it) and restores (a, b) to (X, Y) via the post-flush stack lift positions (Z, Y). Added an integration test `integ_modal_flow_stages_params_for_run_loop` that drives modal → run_loop and asserts ∫₀¹ x² dx ≈ 1/3.

### CR-03: FOUR samples overwrite param registers R23–R26

**Files modified:** `hp41-core/src/ops/math1/four.rs`
**Commit:** 856be2b
**Applied fix:** Introduced `pub const SAMPLE_OFFSET: usize = 27` and shifted sample storage from `state.regs[idx]` to `state.regs[SAMPLE_OFFSET + idx]`. The default `CalcState` has 100 registers, so R27..R127 covers any N up to 100. Parameter registers R23 (N), R24 (L), R25 (start_idx), R26 (RECT) are now pristine through the entire sample-collection loop, preventing the previous "loop bound `n` is re-read from a register the loop itself just overwrote" failure mode.

### CR-04: `op_integ` skips documented `ModeChoice` step

**Files modified:** `hp41-core/src/ops/math1/integ.rs`
**Commit:** ce49e54
**Applied fix:** Changed `op_integ` to open the modal at `IntegInputStep::ModeChoice` with prompt "INTG MODE?" instead of jumping straight to `FunctionNamePrompt`. The existing `submit_step(ModeChoice)` arm now becomes reachable; it advances to `FunctionNamePrompt` on first R/S (Explicit-mode default), preserving end-to-end behavior for Explicit users. Updated the `op_integ_dispatch_opens_modal_when_interactive` unit test to assert the new prompt. Discrete-mode end-to-end wiring remains a future-plan deliverable per 28-07-SUMMARY:245 but the variant + arm are no longer dead.

### BL-01 + BL-02: TRANS prompts drop data; ForwardPrompt/InversePrompt perform no transform

**Files modified:** `hp41-core/src/ops/math1/trans.rs`
**Commit:** f6366b9
**Applied fix:** Combined fix — both findings live in `trans::submit_step` and share the same scratch-register layout. `Init2dPrompt` now captures (x₀, y₀, θ) from (Z, Y, X) via `store_trans2d_params` (R00..R02). `Init3dOriginPrompt` captures (x₀, y₀, z₀) from (Z, Y, X) into R00..R02. `Init3dAxisPrompt` captures (a, b, c, θ) from (T, Z, Y, X) into R03..R06. `ForwardPrompt` and `InversePrompt` now actually invoke `do_trans2d_forward/inverse` or `do_trans3d_forward/inverse` based on a 2D-vs-3D heuristic (R03..R06 populated → 3D), and stay in their respective prompt state for repeated FWD/INV queries per OM convention.

### BL-03: `difeq.rs` doc-comment contradicts actual register layout

**Files modified:** `hp41-core/src/ops/math1/difeq.rs`
**Commit:** 745792b
**Applied fix:** Rewrote the file-level "Scratch Register Convention" block to match the code (R00=order, R01=h, R02=x0, R03=y0, R04=y'0, R05=max_steps) with an explicit note that k₁..k₄ live in local Rust variables, not registers. Eliminates the "R04 (k1), R05 (k2)..." text that misled future maintainers.

### BL-04: `key_coverage.rs` skips Math1 pool — CLI-04 verification gap

**Files modified:** `hp41-cli/tests/key_coverage.rs`
**Commit:** ce19010
**Applied fix:** Migrated the test's iterator from `help_entries()` (narrow v2.2 accessor) to `help_entries_all()` (merged v2.2 + Math1 pool). Raised the lower-bound probe assertion from `>= 50` to `>= 95` to absorb the ~45 new Math1 entries. Added a Math1-specific sub-loop that filters by `xrom.module_id == 7` (HP Math Pac I hardware ID, not the bitmask) and asserts every XEQ-by-name path resolves via `xeq_by_name_local_resolve(name, 0b0000_0001)` with a `math1_probed >= 40` floor. This closes the CLI-04 verification gap flagged by 29-VERIFICATION.md.

### WR-01 + WR-02: `submit_modal` and `submit_modal_with_label` mutate before checking modal state

**Files modified:** `hp41-core/src/ops/math1/mod.rs`
**Commit:** 03a7625
**Applied fix:** Combined fix — both findings live in the same module and share the same anti-pattern. `submit_modal` now resolves the `modal_program` BEFORE calling `flush_entry_buf`, so a misuse without an open modal no longer pushes the user's typed digits onto the stack. `submit_modal_with_label` now uses `ModalProgram::requires_alpha_label()` as a single guard that fires BEFORE the `state.alpha_reg = upper` mutation, so the user's ALPHA register cannot be silently clobbered when no modal is active or when the current step does not accept a label.

### WR-03: DIFEQ `submit_step(OrderPrompt)` silently clamps to {1, 2}

**Files modified:** `hp41-core/src/ops/math1/difeq.rs`
**Commit:** 943789c
**Applied fix:** Removed the `.clamp(1, 2)` from `submit_step(OrderPrompt)`. The raw value now reaches `op_difeq_run_loop` which reports "ORDER MUST BE 1 OR 2" via `modal_prompt` — explicit user feedback replaces silent coercion. The programmatic-path validation (R00 = 3 from a saved program) was already protected by the run_loop check, so its behavior is unchanged.

### WR-04: POLY stale coefficient registers leak between sessions

**Files modified:** `hp41-core/src/ops/math1/poly.rs`
**Commit:** 242843a
**Applied fix:** When `submit_step(CoefficientPrompt)` transitions to `Ready`, zero `state.regs[(degree as usize + 1)..=5]` so `op_roots`' "infer degree from the leading non-zero coefficient" heuristic cannot pick up leaked values from a previous higher-degree POLY session.

### WR-05: `pending_prompt` `unwrap_or("")` defensive dead code

**Files modified:** `hp41-cli/src/ui.rs`
**Commit:** a9dcf09
**Applied fix:** Replaced `return modal_prompt.unwrap_or("").to_string();` with `if let Some(mp) = modal_prompt { return mp.to_string(); }` so the non-None invariant is structural and there is no `""` fallback path. Matches the existing project preference for `if-let` over `unwrap_or` patterns.

### WR-06: F5 Math1 modal interceptor doesn't reset `last_key_code`

**Files modified:** `hp41-cli/src/app.rs`
**Commit:** e91a161
**Applied fix:** Added `self.state.last_key_code = 0;` after the `submit_modal` call in the F5 modal interceptor, before the `return`. Maintains the documented CLAUDE.md v1.1 invariant "F5 / R / S code paths reset `last_key_code` to 0 BEFORE GETKEY runs" for the new Phase 29 dispatch route.

### WR-07: `MAX_SUB_STEPS = 100_000` duplicated across solve/integ/difeq

**Files modified:** `hp41-core/src/ops/math1/mod.rs`, `hp41-core/src/ops/math1/integ.rs`, `hp41-core/src/ops/math1/solve.rs`, `hp41-core/src/ops/math1/difeq.rs`
**Commit:** 62e7f49
**Applied fix:** Hoisted the constant to `pub const USER_CALLBACK_MAX_STEPS: u64 = 100_000` in `hp41-core/src/ops/math1/mod.rs` and replaced each of the three `const MAX_SUB_STEPS: u64 = 100_000` definitions with a `use crate::ops::math1::USER_CALLBACK_MAX_STEPS;` import. The three `run_user_function` sub-loops are now mechanically locked to a single budget.

### WR-08: `cli_resolver_returns_none_for_card_reader_names` test comment stale post-Phase 29

**Files modified:** `hp41-cli/tests/phase25_xeq_by_name.rs`
**Commit:** a771100
**Applied fix:** Rewrote the test's leading comment to accurately describe the Phase 29 resolver chain (conditional-test arms → `xrom_resolve(name, xrom_modules)` → caller falls through to `Op::Xeq` for card-reader names). The test assertions were correct; only the explanation needed updating.

### WR-09: `op_integ_run_loop` silently rounds odd N to N+1

**Files modified:** `hp41-core/src/ops/math1/integ.rs`
**Commit:** 97723ab
**Applied fix:** When the Simpson "even N" coercion fires (`n % 2 == 1`), push `format!("N rounded to {bumped} (Simpson requires even)")` to `state.print_buffer` before computing. The CLI drains `print_buffer` on every run, so the rounding is now visible to the user instead of silently changing their requested precision.

## Skipped Issues

None — all 17 in-scope findings were successfully fixed.

---

_Fixed: 2026-05-17T15:45:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
