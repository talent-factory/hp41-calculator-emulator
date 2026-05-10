# Phase 10: STO Arithmetic Modals - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the 3-step keyboard modal (S → arithmetic op key → register) to connect the existing `op_sto_arith` hp41-core function to TUI keyboard input. This includes:

- **Step 2 routing:** Detect `+`/`-`/`*`/`/` op keys in the `StoRegister` state and transition to `StoAdd`/`StoSub`/`StoMul`/`StoDiv` PendingInput variants (currently dead code).
- **Stack register support (STOA-03):** New `Op::StoArithStack` variant + `StackReg` enum in hp41-core for STO arithmetic to Y/Z/T/LASTX.
- **Help overlay update:** Correct the placeholder "Shift+R+" key descriptions in `help_data.rs` to reflect the actual 3-step modal.

No new calculator operations beyond STOA-01/02/03. Plain STO nn (S → 2 digits) behavior is unchanged.

</domain>

<decisions>
## Implementation Decisions

### Stack Register Design (STOA-03)

- **D-01:** Add `pub enum StackReg { Y, Z, T, LastX }` in `hp41-core/src/ops/mod.rs`, alongside `StoArithKind` — mirrors the existing enum pattern.
- **D-02:** Add `Op::StoArithStack { kind: StoArithKind, stack_reg: StackReg }` to the `Op` enum — a new variant separate from `Op::StoArith { reg: u8, kind }`.
- **D-03:** Add `pub fn op_sto_arith_stack(state: &mut CalcState, stack_reg: StackReg, kind: StoArithKind) -> Result<(), HpError>` in `hp41-core/src/ops/registers.rs`. The function operates on `state.stack.y`, `state.stack.z`, `state.stack.t`, and `state.lastx` (not `state.regs[]`). `LiftEffect: Neutral`.
- **D-04:** Add `Op::StoArithStack` to BOTH `dispatch()` in `hp41-core/src/ops/mod.rs` AND `execute_op()` in `hp41-core/src/ops/program.rs` — same pattern as `Op::StoArith`.

### Modal Step-2 Routing (app.rs)

- **D-05:** In `handle_pending_input`, the `StoRegister(acc)` match arm must intercept arithmetic op keys BEFORE delegating to `handle_reg_modal`. When in `StoRegister("")` state: `+` → `StoAdd("")`, `-` → `StoSub("")`, `*` → `StoMul("")`, `/` → `StoDiv("")`. Plain STO nn path (digit key → accumulate register number) is unaffected.
- **D-06:** Remove `#[allow(dead_code)]` from the `StoAdd`, `StoSub`, `StoMul`, `StoDiv` variants in the `PendingInput` enum — they will be actively used.
- **D-07:** In `StoAdd`/`StoSub`/`StoMul`/`StoDiv` states (step 3 of modal), extend `handle_reg_modal` or add inline handling to also accept letter keys for stack registers: `Y` → `StackReg::Y`, `Z` → `StackReg::Z`, `T` → `StackReg::T`, `L` → `StackReg::LastX`. These dispatch `Op::StoArithStack` immediately (single keypress, no 2-digit accumulation). All other non-digit, non-letter-reg keys are silently ignored (existing pattern).
- **D-08:** Esc cancels the modal at any step with no state change — already implemented in `handle_reg_modal`.

### Step 1 TUI Display

- **D-09:** Keep `STO [__]` as the display string for `StoRegister` state — no change to `ui.rs`. The `?` help overlay is the discovery path for arithmetic variants.

### Help Overlay (help_data.rs)

- **D-10:** Replace the 4 placeholder entries with the correct 3-step modal key descriptions. Use `"S +"` / `"S -"` / `"S *"` / `"S /"` in the key column.
- **D-11:** The description field should say: `"Add X to register nn or stack Y/Z/T/L — press S then +, then nn or Y/Z/T/L"` (and similarly for -, *, /).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/ROADMAP.md` — Phase 10 goal, 4 success criteria, STOA-01/02/03 requirements
- `.planning/REQUIREMENTS.md` — Full acceptance criteria for STOA-01, STOA-02, STOA-03

### Core Implementation Files

- `hp41-core/src/ops/mod.rs` — `Op` enum (add `StoArithStack`), `StoArithKind` enum (add `StackReg` nearby), `dispatch()` function
- `hp41-core/src/ops/registers.rs` — `op_sto_arith()` (add `op_sto_arith_stack()` here)
- `hp41-core/src/ops/program.rs` — `execute_op()` (add `Op::StoArithStack` arm, same pattern as `Op::StoArith`)
- `hp41-cli/src/app.rs` — `PendingInput` enum (remove dead_code), `handle_pending_input()` (add step-2 routing in `StoRegister` arm), `handle_reg_modal()` (extend to accept Y/Z/T/L for stack regs)
- `hp41-cli/src/ui.rs` — lines 224–246: pending_input display strings (already correct — no changes needed)
- `hp41-cli/src/help_data.rs` — lines 71–76: STO arithmetic entries (need key description corrections)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `op_sto_arith()` at `hp41-core/src/ops/registers.rs:43` — already handles R00–R99; `op_sto_arith_stack()` is a parallel function for stack registers. Same atomicity pattern: compute first, write only on success.
- `StoArithKind` enum at `hp41-core/src/ops/mod.rs:29` — `StackReg` enum follows this exact pattern.
- `handle_reg_modal()` at `hp41-cli/src/app.rs:485` — generic 2-digit accumulator. For stack registers, the single-letter path (Y/Z/T/L) bypasses accumulation and dispatches immediately.
- `PendingInput::StoAdd/Sub/Mul/Div(String)` — already defined, already matched in `handle_pending_input`, already has TUI display strings. Only missing: the `StoRegister` → `StoArith` transition (step 2 routing).

### Established Patterns

- `#![deny(clippy::unwrap_used)]` is active in `hp41-core` — `op_sto_arith_stack()` must use `.expect("reason")` or `?`-propagation; new test code carries `#[allow(clippy::unwrap_used)]`.
- Every new `Op` variant must appear in BOTH `dispatch()` (ops/mod.rs) AND `execute_op()` (ops/program.rs) — Critical Implementation Trap from STATE.md.
- `handle_key()` guards are silent (no message, no beep) — unknown keys in modal state are silently ignored.
- `pending_input` routing block is ABOVE modal-opening interceptors (`S`/`R`/Ctrl+A) — this order must be preserved.

### Integration Points

- The `StoRegister` arm in `handle_pending_input()` currently calls `handle_reg_modal()` directly. Step 2 routing requires a key-code check before that delegation: if `+`/`-`/`*`/`/` → set `StoAdd("")` etc.; otherwise fall through to `handle_reg_modal()` for digit handling.
- Stack register dispatch: `op_sto_arith_stack()` is called via `Op::StoArithStack` in `dispatch()` — same call chain as all other ops.
- TUI display in `ui.rs` lines 241–246 already renders `STO [__]`, `STO+ [__]`, `STO- [__]`, `STO× [__]`, `STO÷ [__]` correctly. No display changes needed.

</code_context>

<specifics>
## Specific Ideas

- **StackReg keyboard mapping in modal:** `Y` → Y, `Z` → Z, `T` → T, `L` → LastX. The letter `L` (not `X`) maps to LASTX — `X` is not a valid target for STO arithmetic (you'd be overwriting the source). The planner should add single-letter dispatch in the `StoAdd/Sub/Mul/Div` arms before the digit-accumulation path.
- **`handle_reg_modal` extension:** Either add Y/Z/T/L handling inline in each `StoAdd/Sub/Mul/Div` match arm, or pass an optional `StackReg` dispatch closure to `handle_reg_modal`. The former is simpler and avoids overcomplicating the generic helper.
- **Dead code removal:** The `#[allow(dead_code)]` on StoAdd/Sub/Mul/Div can be removed in the same commit that adds the step-2 routing.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 10-STO Arithmetic Modals*
*Context gathered: 2026-05-08*
