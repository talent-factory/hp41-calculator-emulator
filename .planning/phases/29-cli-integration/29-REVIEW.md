---
phase: 29-cli-integration
reviewed: 2026-05-17T12:30:00Z
depth: standard
files_reviewed: 27
files_reviewed_list:
  - docs/hp41-math1-functions.json
  - docs/verifying-math-pac-1.md
  - hp41-cli/src/app.rs
  - hp41-cli/src/help_data.rs
  - hp41-cli/src/keys.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/tests/function_matrix_parity.rs
  - hp41-cli/tests/key_coverage.rs
  - hp41-cli/tests/phase25_help_data.rs
  - hp41-cli/tests/phase25_pending_input.rs
  - hp41-cli/tests/phase25_xeq_by_name.rs
  - hp41-cli/tests/phase29_help_data_math1.rs
  - hp41-cli/tests/phase29_key_ref_includes_math1.rs
  - hp41-cli/tests/phase29_modal_flow.rs
  - hp41-cli/tests/phase29_pending_prompt_modal.rs
  - hp41-core/src/ops/math1/difeq.rs
  - hp41-core/src/ops/math1/four.rs
  - hp41-core/src/ops/math1/integ.rs
  - hp41-core/src/ops/math1/matrix.rs
  - hp41-core/src/ops/math1/mod.rs
  - hp41-core/src/ops/math1/modal.rs
  - hp41-core/src/ops/math1/poly.rs
  - hp41-core/src/ops/math1/solve.rs
  - hp41-core/src/ops/math1/trans.rs
  - hp41-core/tests/math1_difeq.rs
  - hp41-core/tests/math1_integ.rs
  - hp41-core/tests/math1_solve.rs
findings:
  critical: 4
  blocker: 4
  warning: 9
  info: 5
  total: 18
status: issues_found
---

# Phase 29: Code Review Report

**Reviewed:** 2026-05-17T12:30:00Z
**Depth:** standard
**Files Reviewed:** 27
**Status:** issues_found

## Summary

Phase 29 wires `hp41-cli` to Phase 28's XROM framework and adds modal-routing for Math Pac I. The additive changes to `hp41-core` (`submit_modal`, `cancel_modal`, `submit_modal_with_label`, per-program `submit_step`) are well-structured; the JSON pipeline mirror (`docs/hp41-math1-functions.json`, `help_entries_math1`, `help_entries_all`) is faithful to the v2.2 D-25.16 pattern; the resolver widening (`xeq_by_name_local_resolve(name, xrom_modules)`) preserves the C-28.4 ordering.

However, the submit_step implementations across `integ.rs`, `four.rs`, and `trans.rs` show three classes of defects:

1. **Modal/run_loop parameter disconnect** — `submit_step` stores params in registers the corresponding `op_*_run_loop` does not read (INTG: R02/R03 vs stack X/Y; FOUR: samples overwrite N/L/start/RECT registers).
2. **Silent partial implementations** — `TransInputStep::Init2dPrompt` reads only x₀ and discards y₀/θ; `ForwardPrompt`/`InversePrompt` do NO transform, just clear the prompt; `Op::Integ` modal opens at `FunctionNamePrompt` (skipping documented `ModeChoice` step).
3. **Error-path state leaks** — `op_integ_run_loop` leaks `integ_state = Some(_)` on three `Decimal::from_f64(_).ok_or(HpError::Overflow)?` paths, then poisons every subsequent INTG via the XROM-08 nested-rejection guard.

The CLI plumbing has a coverage hole (`key_coverage.rs` only iterates the v2.2 narrow accessor, never probing the new 45 Math1 XEQ entries) and a `pending_prompt` dead-code unwrap. A documentation/code mismatch in `difeq.rs` (R-register layout in the file-level doc-comment contradicts the code) misleads future maintainers.

## Critical Issues

### CR-01: `op_integ_run_loop` leaks `integ_state` on every Overflow path, poisoning subsequent INTG calls

**File:** `hp41-core/src/ops/math1/integ.rs:278-281, 303, 339, 369`
**Issue:** Three `Decimal::from_f64(_).ok_or(HpError::Overflow)?` (and one `to_f64().ok_or(HpError::Overflow)?`) early-returns occur AFTER `state.integ_state = Some(integ.clone())` is set at line 273, but BEFORE the function's normal cleanup paths. When any of these fire, `integ_state` remains `Some(_)` permanently. The next `op_integ_run_loop` invocation hits the XROM-08 nested-rejection guard at line 211 and returns `Err(HpError::InvalidOp)` even though nothing is nested — INTG is now permanently broken until the user finds a way to manually clear `integ_state`. The same pattern exists at line 369 (final result push) where `integ_state` is already cleared at line 366, so that one is safe; but lines 278/281/303/339 are not.

Concretely:
- Line 278-281: `a_f64 = a.inner().to_f64().ok_or(...)?` — if X holds a NaN-derived HpNum (impossible today, but the guard exists), state leaks.
- Line 303: `Decimal::from_f64(x_k).ok_or(...)?` — if `x_k` is NaN/Inf (possible when `(b_f64 - a_f64) / n_f64` produces NaN from b=Inf or n=0), state leaks. n is min-clamped to 2, but b/a from stack are user-controlled.
- Line 339: `state.stack.x.inner().to_f64().ok_or(...)?` after user function returns — user could STO a value that round-trips to None.

**Fix:**
```rust
// Replace `?` with explicit error-handling that clears integ_state first.
// Helper closure / macro to avoid duplication:
macro_rules! bail_integ {
    ($state:expr, $save_pc:expr, $e:expr) => {{
        $state.integ_state = None;
        $state.pc = $save_pc;
        return Err($e);
    }};
}
// Then:
let a_f64 = match a.inner().to_f64() {
    Some(v) => v,
    None => {
        state.integ_state = None;
        return Err(HpError::Overflow);
    }
};
// ...similarly for all `?` sites between integ_state = Some(...) and the
// final `state.integ_state = None` at line 366.
```

### CR-02: `op_integ_run_loop` ignores modal-staged params; reads from stack/R00 instead of R02/R03/R04 where submit_step wrote them

**File:** `hp41-core/src/ops/math1/integ.rs:208-242` vs `hp41-core/src/ops/math1/integ.rs:466-495`
**Issue:** `submit_step(IntegInputStep::IntervalPrompt, ...)` stores the integration bounds into `regs[2]` and `regs[3]`, and `submit_step(IntegInputStep::SubdivisionPrompt, ...)` stores N into `regs[4]`. But `op_integ_run_loop` at lines 234-241 reads `a` from `state.stack.x`, `b` from `state.stack.y`, and `n` from `regs[0]`. The two are entirely disjoint — the values collected via the modal are NEVER consumed when the user finally invokes INTG. Even if a user completes the entire `FunctionNamePrompt → IntervalPrompt → SubdivisionPrompt → Ready` flow, then runs a program with `Op::Integ`, the run_loop will see stale or empty stack/R00 values.

This is a hardcoded contract mismatch: `submit_step` and `op_integ_run_loop` were authored against different scratch-register conventions.

**Fix:** Pick one convention and make both sides agree. Either:
- `submit_step` writes to stack X/Y and regs[0] (matching what `op_integ_run_loop` already reads), or
- `op_integ_run_loop` reads from regs[2]/regs[3]/regs[4] (matching what `submit_step` writes).

Then add an integration test that runs the full modal flow followed by a programmatic `Op::Integ` invocation and asserts the integration uses the values entered in the modal.

### CR-03: `Four::SamplePrompt` writes samples into R00..R{N-1}, overwriting the params at R23 (N), R24 (L), R25 (start_idx), R26 (RECT)

**File:** `hp41-core/src/ops/math1/four.rs:384-407`
**Issue:** `submit_step(FourInputStep::SamplePrompt(idx), ...)` writes `state.regs[idx as usize] = state.stack.x.clone()` at line 396. But the previous steps stored N in R23, L in R24, start_idx in R25, and RECT in R26 (lines 341, 357, 367, 378). When the user enters sample 24 (`idx=23`), R23 is overwritten — destroying the N value that the SamplePrompt advance condition `next_idx < n` reads from R23 at line 387.

This means:
- For N ≤ 23 samples: the loop completes correctly but R24/R25/R26 are still corrupted on the LAST sample (idx=23 writes over R23=N) — but `next_idx < n` was already evaluated before the write, so the loop terminates. However R24/R25/R26 will hold the last sample values, NOT L/start/RECT — making downstream FOUR computation read garbage parameters.
- For N ≥ 24 samples: even worse — `idx=23` overwrites R23 (N), then `n` is re-read at line 387 as the new sample value, completely changing the loop bound.
- For N ≥ 27 samples: corrupts ALL four parameter registers.

Since N can legitimately be larger than 23 (FFT on, say, 32 or 64 samples is common), this is a hot-path bug.

**Fix:** Move sample storage to a register range that does not overlap with the parameter registers. Suggested layout: store samples at R00..R{N-1} ONLY if N ≤ 23; or, better, move the parameter registers out of the sample storage range entirely (e.g., to R50..R53) and update both `submit_step` and the future FFT consumer to read from the new locations.

### CR-04: `op_integ` dispatch arm skips documented `ModeChoice` step; never reachable

**File:** `hp41-core/src/ops/math1/integ.rs:150-167`
**Issue:** The modal definition (`modal.rs:235-250`) declares `IntegInputStep::ModeChoice` as the first prompt ("INTG MODE?") and `submit_step(ModeChoice)` advances to `FunctionNamePrompt`. But `op_integ` opens the modal directly at `FunctionNamePrompt`, bypassing `ModeChoice` entirely. The comment at line 156 admits this: "For CLI simplicity: open at FunctionNamePrompt directly (Explicit mode default)." This means:

1. `IntegMode::Discrete` is unreachable from the CLI (the only path to set Discrete mode would be ModeChoice → submit_step → choose Discrete, but ModeChoice is never opened).
2. `op_integ_run_loop` line 243 hardcodes `IntegMode::Explicit`, so even if Discrete were reachable, it'd return `Err(HpError::InvalidOp)` at line 349-356.
3. `submit_step(IntegInputStep::ModeChoice)` is dead code on the interactive path.
4. The unit test `integ_modal_dispatch_round_trip` at modal.rs:929 expects ModeChoice → "INTG MODE?", but no real user flow exercises that prompt.

Either ship a 2-prompt flow that asks the user to pick Discrete/Explicit, or DELETE `ModeChoice` and `IntegMode::Discrete` so the dead code can't mislead future maintainers.

**Fix:** Either implement Discrete mode end-to-end (modal collection + run_loop dispatch) or remove the `ModeChoice` variant and the `Discrete` arm. The current half-shipped state is worse than either alternative.

## Blockers

### BL-01: `TransInputStep::Init2dPrompt` silently drops y₀ and θ; transform parameters are never captured

**File:** `hp41-core/src/ops/math1/trans.rs:463-473`
**Issue:** The 2D transform initialization needs three parameters: x₀, y₀, θ. The comment at line 443-447 acknowledges this and says "accepts a single R/S submit that reads the current X value and stores x₀". But `submit_step(Init2dPrompt)` ONLY stores `state.regs[0] = state.stack.x.clone()` (= x₀, assuming user typed it last). y₀ and θ are NEVER read — they are lost. The user thinks they've configured a transform with `0 ENTER 0 ENTER 90`, but only `90` ends up captured (and into R00 where the transform code reads x₀, so x₀=90 and y₀=θ=0). The subsequent FWD/INV outputs are silently wrong.

This is a SHIPPED silent-correctness bug. Test `trans2d_master_opens_modal` at trans.rs:537 only checks that the modal opens; no integration test verifies the param-capture chain.

**Fix:** Either:
- Read all three params from the stack at once (Z=x₀, Y=y₀, X=θ) and store to R00/R01/R02 in `Init2dPrompt`, plus a comment in the modal_prompt to indicate the user must enter all three values, OR
- Split `Init2dPrompt` into `Init2dX0Prompt → Init2dY0Prompt → Init2dThetaPrompt → Ready`, each advancing on R/S, so the user enters one value per R/S press (matching the HP-41 convention).

The current implementation gives users no way to set θ on a 2D transform.

### BL-02: `TransInputStep::ForwardPrompt` and `InversePrompt` perform NO transform; just clear the prompt

**File:** `hp41-core/src/ops/math1/trans.rs:492-500`
**Issue:** `submit_step(ForwardPrompt)` and `submit_step(InversePrompt)` both fall through to the same arm that "Just acknowledge and clear prompt" — they do NOT invoke the transform helpers (`do_trans2d_forward`, `do_trans3d_forward`, etc. that exist in the same file). When the user types an input point and presses R/S in `ForwardPrompt`, the modal transitions to `Ready` and NO computation happens — the stack is unchanged.

The comment admits this: "Actual computation is deferred to full Phase 31 wiring with the op_trans2d_forward / op_trans3d_forward helpers."

But the CLI status table at `docs/verifying-math-pac-1.md:381` claims "✅ Interactive — `X0,Y0,θ?` → `FWD?`/`INV?` (simplified Phase 29)". This is an overclaim — the modal is reachable but produces no output. A user pressing R/S after entering an input point will see no result, no error, no print line. Silent failure.

**Fix:** Either invoke the corresponding transform helper at this dispatch point, OR return `Err(HpError::InvalidOp)` with a message that points the user to use `XEQ "TRANS"` then the existing FWD/INV operations. Do not silently swallow the user's input.

### BL-03: `Op::DifeqState` documentation contradicts the actual register layout — future maintainers will reach for the wrong R-numbers

**File:** `hp41-core/src/ops/math1/difeq.rs:46-49` vs `hp41-core/src/ops/math1/difeq.rs:166-225`
**Issue:** The file-level doc-comment at lines 46-49 claims:
> "R00 (x), R01 (y), R02 (y' for order=2), R03 (step_size), R04 (k1), R05 (k2), R06 (k3), R07 (k4) per OM convention."

But the actual register layout used by `op_difeq_run_loop` (lines 209-225) and `submit_step` (lines 754-827) is:
- R00 = order (1 or 2)
- R01 = step size h
- R02 = x0
- R03 = y0
- R04 = y'0 (only when order=2)
- R05 = max_steps

These two layouts are completely DIFFERENT. The doc-comment R00-R07 layout doesn't match a single line of the implementation. The "OM convention" claim is unverifiable — neither layout cites a specific OM page.

When Phase 30/31 maintainers read difeq.rs to figure out which registers are "scratch", they'll see the doc-comment and assume R00=x, R03=step_size — but the code uses R00=order, R01=step_size. The test `scratch_registers` at line 1201 confirms the code convention, not the documented one.

**Fix:** Update the doc-comment at lines 46-49 to match the actual register layout used by the code (R00=order, R01=step_size, R02=x0, R03=y0, R04=y'0, R05=max_steps), or move RK4 intermediate registers (k1-k4) to local Rust variables (which they already are) and remove the misleading "R04 (k1), R05 (k2)..." text entirely.

### BL-04: `key_coverage.rs` only iterates v2.2 narrow pool — 45 Math1 JSON entries with `key_path: "XEQ \"<NAME>\""` are NEVER probed for resolver closure

**File:** `hp41-cli/tests/key_coverage.rs:119-243`
**Issue:** The test `key_coverage_implemented_entries_dispatch` iterates `help_entries()` (v2.2 narrow accessor). But Phase 29 Plan 02 explicitly migrated `help_overlay_rows` and `key_ref_entries` to `help_entries_all()` to include the 45 Math1 entries. The `key_coverage` test was NOT migrated — so a future JSON-authoring mistake (e.g., misspelling `SINH` as `SHIN` in `hp41-math1-functions.json`) will:
- Pass `function_matrix_parity::test_every_math1_json_entry_has_xrom_resolver_match` IF the display_name `SHIN` happens to exist in `MATH_1.ops` (it doesn't, so this test would catch THAT specific drift).
- But fail no test if the display_name is a typo that resolves to None — because `key_coverage` won't probe it.

The closure invariant from D-25.18 ("every implemented JSON entry with non-null `key_path` dispatches to a known `Op::` variant") is broken for the entire Math1 pool.

**Fix:** Change line 120 from `let entries = help_entries();` to `let entries: Vec<&_> = help_entries_all().collect();` (then iterate `entries.iter()`), and bump the lower-bound probe count assertion at line 238 from `>= 50` to `>= 50 + 45 = 95` (or similar) to ensure the new entries are actually probed. Add a Math1-specific assertion sub-loop that asserts every Math1 `XeqByName` resolves via `xeq_by_name_local_resolve(&name, 0b0000_0001).is_some()`.

## Warnings

### WR-01: `submit_modal` flushes entry_buf BEFORE checking that a modal is active — wasted work, but more importantly leaks state on misuse

**File:** `hp41-core/src/ops/math1/mod.rs:44-64`
**Issue:** `submit_modal` calls `flush_entry_buf(state)?` at line 48 unconditionally, THEN checks `modal_program.is_some()` at line 50. If the function is invoked when no modal is active (which the CLI guards against, but other future callers — e.g., the GUI — may not), `flush_entry_buf` pushes the entry buffer onto the stack with lift enabled, modifying the visible calculator state, and only THEN does the function return `Err(HpError::InvalidOp)`. The user sees their typed digits pushed to X but gets a "invalid op" error — confusing.

**Fix:** Reorder the checks so the modal-active guard fires first:
```rust
pub fn submit_modal(state: &mut CalcState) -> Result<(), HpError> {
    let modal = match state.modal_program.clone() {
        Some(m) => m,
        None => return Err(HpError::InvalidOp),
    };
    crate::ops::flush_entry_buf(state)?;  // only flushed if we'll consume it
    match modal {
        ModalProgram::Matrix(step) => matrix::submit_step(state, step),
        // ...
    }
}
```

### WR-02: `submit_modal_with_label` writes alpha_reg BEFORE checking modal_program

**File:** `hp41-core/src/ops/math1/mod.rs:92-116`
**Issue:** Same anti-pattern as WR-01. Lines 95-96 mutate `state.alpha_reg = upper.clone()` unconditionally, then line 99 checks `modal_program.is_some()`. If no modal is active, `alpha_reg` is silently corrupted with the label string and the function returns `Err(HpError::InvalidOp)` — but the corrupted alpha_reg persists, potentially breaking subsequent ALPHA operations.

Worse: even if a modal IS active but is not at the three FunctionNamePrompt variants (the `_ => Err(InvalidOp)` fallback at line 114), `alpha_reg` has still been mutated. The user's existing ALPHA register contents are clobbered for no semantic benefit.

**Fix:**
```rust
pub fn submit_modal_with_label(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    // Verify the modal is in a label-accepting state BEFORE mutating anything
    let modal = match state.modal_program.clone() {
        Some(m) if m.requires_alpha_label() => m,
        Some(_) => return Err(HpError::InvalidOp),
        None => return Err(HpError::InvalidOp),
    };
    let upper = label.trim().to_ascii_uppercase();
    state.alpha_reg = upper;
    match modal { /* ... */ }
}
```

### WR-03: `op_difeq` modal-opener and `op_difeq_run_loop` ORDER validation has dead-code paths

**File:** `hp41-core/src/ops/math1/difeq.rs:760-771` (submit_step OrderPrompt) vs `hp41-core/src/ops/math1/difeq.rs:234-237` (op_difeq_run_loop)
**Issue:** `submit_step(OrderPrompt)` reads X, calls `.clamp(1, 2)` at line 767, then writes the clamped value to R00. If the user enters "3", it silently becomes "2" — the user thinks they're running a 3rd-order ODE but actually gets a 2nd-order one. No warning. Then `op_difeq_run_loop` at line 234 validates `order_raw != 1 && order_raw != 2`, but since `submit_step` has already clamped to {1, 2}, this validation is dead code on the modal-driven path. Dead validation is misleading — it suggests the validation is enforced, but the modal already silently coerced the value.

The validation IS reachable from the programmatic path (`Op::Difeq` inside a saved program with R00 = 3), so removing it would weaken that path. But the modal path needs a real validation.

**Fix:** Remove the `.clamp(1, 2)` from `submit_step(OrderPrompt)`. Let the raw value through (capped to `u8::MAX` via `to_u8`), and rely on the run_loop validation to surface "ORDER MUST BE 1 OR 2". The user gets a clear error message instead of silent coercion.

### WR-04: `PolyInputStep::CoefficientPrompt(degree, idx)` stores degree separately in R06 but also coefficients land in R00..R05 — R06 may collide if `op_roots` reads coefficients into a 6-element slice

**File:** `hp41-core/src/ops/math1/poly.rs:467-516`
**Issue:** `submit_step(DegreePrompt)` stores degree in R06 (line 480). `submit_step(CoefficientPrompt(degree, idx))` stores coefficient at `R{idx}` (line 491). For degree=5, idx ranges 0..5, so coefficients fill R00..R05 and degree sits in R06. So far so good.

But `op_roots` (the consumer) reads coefficients from R00..R05 — and as documented at the file header, it "infers [degree] from the leading non-zero coefficient register (R00..R05)". This means R06 (degree storage) is IGNORED by `op_roots`. So the user enters degree=3, then coefficients A=3, B=2, C=0, D=1 (constant). `op_roots` sees R00=3, R01=2, R02=0, R03=1, infers degree=3 from "leading non-zero" — but if R04 or R05 are non-zero from previous use, it'll infer the WRONG degree.

The `submit_step(CoefficientPrompt(degree, idx))` does NOT zero R{idx+1..=5} when the user finishes coefficient entry. So stale coefficient values from a previous POLY session can leak into the next one.

**Fix:** When `submit_step` transitions to `PolyInputStep::Ready` (line 509), zero `state.regs[(degree as usize + 1)..=5]` to clear any stale coefficients. Or: read degree from R06 in `op_roots` instead of inferring it from leading non-zero.

### WR-05: `pending_prompt` line 280 uses `.unwrap_or("")` after `modal_prompt.is_some()` check — defensive code suggests the invariant could be violated, but it can't

**File:** `hp41-cli/src/ui.rs:279-281`
**Issue:** After `if pending.is_none() && modal_prompt.is_some()` evaluates true, line 280 does `return modal_prompt.unwrap_or("").to_string();`. Since `modal_prompt.is_some()` was just checked, `unwrap_or("")` can never return the fallback `""` — it'll always be the inner `&str`. The `.unwrap_or("")` is dead defensive code that obscures the intent. Should be `.expect("checked is_some above")` per the project's `clippy::unwrap_used` convention, or restructured to use `if let Some(mp) = modal_prompt` which makes the invariant explicit.

**Fix:**
```rust
if pending.is_none() {
    if let Some(mp) = modal_prompt {
        return mp.to_string();
    }
}
```

### WR-06: `keycode_to_hp41_code` has zero coverage for the new Phase 29 F5/Esc handlers — clobbering of `last_key_code` not analyzed

**File:** `hp41-cli/src/keys.rs:444-497` vs `hp41-cli/src/app.rs:297-300`
**Issue:** `App::handle_key` updates `state.last_key_code` at lines 297-300 BEFORE the Phase 29 F5/Esc modal interceptors run. `keycode_to_hp41_code(KeyCode::F(5))` returns `None`, so `last_key_code` is not updated for F5 (good). But the Phase 29 modal-submit path invokes `submit_modal` which calls `flush_entry_buf` and then `submit_step`. None of those reset `last_key_code`. Compare this to the F5/F7/F8 reset behavior documented in CLAUDE.md ("F5 / R / S code paths reset `last_key_code` to 0 BEFORE GETKEY runs").

If a user is mid-program with `Op::GetKey` waiting for keypress data, then enters a Math1 modal, the modal's R/S submission does NOT reset `last_key_code`. The next `Op::GetKey` invocation will see the stale code. Minor consequence in v3.x where Math1 is unlikely to interact with GetKey, but a latent footgun.

**Fix:** In the new F5 modal interceptor at app.rs:661-673, after `submit_modal(&mut self.state)`, add `self.state.last_key_code = 0;` to maintain the documented invariant.

### WR-07: `MAX_SUB_STEPS = 100_000` in `run_user_function` is duplicated across solve.rs, integ.rs, difeq.rs — silent drift risk

**File:** `hp41-core/src/ops/math1/solve.rs:502, integ.rs:397, difeq.rs:646`
**Issue:** The infinite-loop guard constant `const MAX_SUB_STEPS: u64 = 100_000;` is defined three times across three files. If any one is changed without the others, the behavior diverges. The `// CROSS-REFERENCE: mirrors op_integ_run_loop user-callback pattern` doc-comment acknowledges the duplication but provides no mechanical enforcement.

**Fix:** Move the constant to `hp41-core/src/ops/math1/mod.rs` as `pub const USER_CALLBACK_MAX_STEPS: u64 = 100_000;` and import from each file. Or, factor `run_user_function` itself into a shared `pub(super)` helper in `mod.rs` since the three copies are identical (line-for-line).

### WR-08: `cli_resolver_returns_none_for_card_reader_names` test (phase25_xeq_by_name.rs) now misleadingly named after Phase 29 widening

**File:** `hp41-cli/tests/phase25_xeq_by_name.rs:449-471`
**Issue:** The test name and doc-comment claim CLI-local returns None because card-reader names "fall through to `Op::Xeq` → core builtin_card_op". But after Phase 29, the fall-through goes through `xrom_resolve(name, xrom_modules)` first (the new `_ =>` arm at keys.rs:382). `xrom_resolve` returns None for WPRGM/RDPRGM/WDTA/RDTA (they're not in MATH_1.ops), so the test still passes — but the doc-comment is now stale. A reader of this test would not realize that the resolver chain runs `xrom_resolve` for these names.

**Fix:** Update the doc-comment at lines 451-454 to reflect the Phase 29 resolver chain: "CLI-local resolver covers ONLY the 8 conditional tests and the 45 Math Pac I XROM ops via `xrom_resolve`. For the 4 v2.1 card-reader names, BOTH the conditional-test arms AND `xrom_resolve(name, 0b0000_0001)` return None — the fall-through to `Op::Xeq` → `builtin_card_op` happens at the call site."

### WR-09: `op_integ_run_loop` line 254 silently increments odd n to n+1 with no user notification

**File:** `hp41-core/src/ops/math1/integ.rs:251-261`
**Issue:** `let n_even = if n % 2 == 1 { n + 1 } else { n };` silently converts odd subdivision count to even (required for Simpson's rule). The user types N=5, gets N=6 subdivisions instead. No warning surfaces. The accuracy claim "exact tolerance per integ_threshold" silently degrades if the user expected N exactly.

Also the cast `n_even.min(u16::MAX as u32) as u16` at line 261 silently overflows if `n_even` is between 65535 and INTG_MAX_EVALS=32768 — wait, n_val is already capped at 32768 above, so n_even is at most 32769 which still fits in u16. OK.

**Fix:** Push a print_buffer line like `format!("N rounded to {n_even}")` when n is forced to even. Documented user-friendly behavior over silent coercion.

## Info

### IN-01: `pending_input` reset on Esc-cancel inside `XeqByName{CollectForModal}` leaves dangling `modal_program` — two-step Esc UX may surprise users

**File:** `hp41-cli/src/app.rs:1461-1465` (handle_xeq_by_name) and `hp41-cli/src/app.rs:681-688` (modal Esc handler)
**Issue:** When `pending_input = XeqByName{CollectForModal}` is active, pressing Esc clears only `pending_input` (line 1463). The `modal_program` remains set (e.g., to `Solve(FunctionNamePrompt)`). The user is now in a state with a dangling modal_program but no visible prompt. Pressing Esc AGAIN cancels the modal_program via the line 681 handler. This two-step Esc is undocumented in the UX and may confuse users who expect one Esc to fully exit.

**Fix:** In `handle_xeq_by_name`'s Esc arm, if `mode == XeqByNameMode::CollectForModal`, also call `cancel_modal(&mut self.state)`. Single-Esc fully exits the workflow.

### IN-02: `matrix::submit_step(OrderPrompt)` clamps order with no user feedback when entered N exceeds 14

**File:** `hp41-core/src/ops/math1/matrix.rs:355-371`
**Issue:** `let n = n_raw.clamp(1, MAX_ORDER)` silently coerces invalid order values. User types `25 R/S`, gets a 14×14 matrix instead of an error. Same UX issue as WR-03 / WR-09 — silent coercion vs explicit rejection.

**Fix:** Replace `.clamp(1, 14)` with an explicit check: `if n_raw < 1 || n_raw > MAX_ORDER { state.modal_prompt = Some("ORDER MUST BE 1..=14".to_string()); return Ok(()); }`. Match the existing `ORDER MUST BE 1 OR 2` precedent in `op_difeq_run_loop`.

### IN-03: `four::SamplePrompt` uses `state.regs[idx as usize]` indexing with no `regs.len()` guard — out-of-bounds panic on tiny CalcState

**File:** `hp41-core/src/ops/math1/four.rs:391-396`
**Issue:** Lines 393-394 guard against `idx as usize >= state.regs.len()` and return `InvalidOp`. But the comparison happens AFTER computing `next_idx = idx + 1`. If `idx` is u8::MAX, `next_idx = idx + 1` wraps to 0 (silent wrap in release; panic in debug). Then `next_idx < n` evaluates `0 < n` which is true for any N≥1, so the loop continues incorrectly.

Practical exposure: user would have to enter ≥255 samples, which requires a `state.regs` of length ≥255 — much larger than the default. Low practical risk but easy to harden.

**Fix:** Change `idx: u8` to `idx: u32` in `FourInputStep::SamplePrompt(u32)` and update the modal.rs definition. Or guard against the wrap explicitly: `let next_idx = match idx.checked_add(1) { Some(v) => v, None => { ...transition to Ready... } };`.

### IN-04: Plan summaries claim "27 total citations" / "≥80 Catches comments" — none of those metrics are checked by tests

**File:** `.planning/phases/29-cli-integration/29-03-SUMMARY.md` lines 244-255 (Performance) and CLAUDE.md "Phase 27 additions" section
**Issue:** The summary docs cite specific test counts and "Catches:" comment counts but no CI/test gate enforces them. A future commit that strips comments wouldn't fail any test. Pure documentation drift risk.

**Fix:** Either add a grep-audit test (mirror of `function_matrix_parity.rs` style) that asserts `grep -c "// Catches:" path/to/test/*.rs >= N`, OR remove the unverified numeric claims from the summaries.

### IN-05: `docs/hp41-math1-functions.json` description text is inconsistent across hyperbolic vs complex entries — "X <- sinh(X)" vs "(Yre+iYim) + (Xre+iXim) -> stack (Y=re, X=im)"

**File:** `docs/hp41-math1-functions.json` (entries 1-6 vs entries 7-23)
**Issue:** Hyperbolic descriptions use mathematical arrow notation `"X <- sinh(X)"`. Complex arithmetic descriptions use parenthesized component notation `"(Yre+iXim)..."`. The two styles render differently in the `?` overlay — users see inconsistent visual density. This isn't a correctness bug but degrades the overlay's professional feel. Also: `"(Yre+iZre)^N"` at entry 14 is a typo — should be `(Yre+iXim)^N` since the complex value lives in (X, Y) per the Real/CTimes convention. Same potential typo in entries 15, 21 — needs an OM cross-check.

**Fix:** Normalize the description style across all 45 entries. Either use `"X ← f(X)"` (Unicode arrow) consistently or `"X = f(X)"`. Verify the i-component register naming in entries 14-23 against the OM Chapter 4 complex arithmetic chapter. The current "Z" naming in `iZre` is suspect because elsewhere Z refers to the stack T-Z-Y-X register, not the complex-pair convention.

---

_Reviewed: 2026-05-17T12:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
