# Phase 12: Synthetic Programming - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Add GETKEY, NULL, and hidden registers M/N/O to `hp41-core` as first-class Op variants, and add a hex-byte insertion modal to `hp41-cli` that lets users insert synthetic ops into the current program step using a 2-digit hex code from a curated safe subset.

This phase does NOT add interactive GETKEY (program pauses and waits for a key — SYNT-06, deferred). It does NOT add the full FOCAL byte-code table (~200 codes — SYNT-05, deferred). It does NOT add indirect addressing for hidden registers.

</domain>

<decisions>
## Implementation Decisions

### GETKEY (SYNT-01)

- **D-01:** `last_key_code: u8` stored on `CalcState` with `#[serde(default)]`. Updated in `handle_key()` before any dispatch, on every key press. Persists across save/load — consistent with all other calculator state.
- **D-02:** HP-41 row-column key encoding: key code = row×10 + col (1-indexed). Requires a lookup table mapping crossterm `KeyCode` → HP-41 hardware code. The lookup table lives in `hp41-cli/src/keys.rs` or a new `hp41-cli/src/keycode_map.rs`.
- **D-03:** When no key has been pressed yet (fresh CalcState, `last_key_code` = 0), `Op::GetKey` pushes `0` to X with `LiftEffect::Enable`. Consistent with HP-41 hardware behavior.
- **D-04:** `Op::GetKey` has `LiftEffect::Enable` — it produces a value on the stack.

### NULL (SYNT-02)

- **D-05:** `Op::Null` is a true no-op with `LiftEffect::Neutral`. Does not touch any stack register, any numbered register, or the lift flag. Implementation: empty body in dispatch arm. Display name in `prgm_display.rs`: `"NULL"`.

### Hidden Registers M/N/O (SYNT-03)

- **D-06:** Three separate named fields on `CalcState`: `reg_m: HpNum`, `reg_n: HpNum`, `reg_o: HpNum` — each with `#[serde(default)]`. Initialized to `HpNum::zero()` in `CalcState::new()`. Not part of the numbered `regs: Vec<HpNum>`.
- **D-07:** Six new Op variants: `StoM`, `StoN`, `StoO` (LiftEffect::Neutral) and `RclM`, `RclN`, `RclO` (LiftEffect::Enable). Implemented in `hp41-core/src/ops/registers.rs` (existing module, mirrors StoReg/RclReg pattern).
- **D-08:** Keyboard entry: extend the existing `S` and `R` modal interceptors in `handle_key()`. After the `'S'` or `'R'` prefix is typed, when the user presses `'M'`, `'N'`, or `'O'` (before any register digit), dispatch `StoM/StoN/StoO` or `RclM/RclN/RclO` immediately. Esc cancels as usual.
- **D-09:** TUI modal display during `StoRegister` / `RclRegister` state: already shows `"STO [__]"` / `"RCL [__]"`. After M/N/O dispatch, the display string becomes `"STO M"` / `"RCL M"` etc. (same area as the register number would appear).
- **D-10:** Program listing display in `prgm_display.rs`: `Op::StoM` → `"STO M"`, `Op::RclM` → `"RCL M"` (and similarly for N, O).

### Hex-Byte Insertion Modal (SYNT-04)

- **D-11:** `Op::SyntheticByte(u8)` — new variant that stores the raw accepted hex byte code. During execution, it is dispatched to the corresponding Op using the same lookup table used for validation at insertion time. Preserves the information that the step was inserted synthetically (useful for program listing display: `"SYN nn"` where nn is the hex code).
- **D-12:** Safe subset = only hex codes that map to already-implemented Ops in our enum. A `const` lookup table (or `match` arm) validates input codes. Anything outside the subset is rejected.
- **D-13:** Rejection behavior: set `app.message = Some("INVALID")`, do NOT modify the program Vec. The modal closes after any result (valid or rejected). Consistent with HP-41 hardware signaling and existing modal pattern.
- **D-14:** Keyboard binding: `'X'` (uppercase, Shift+X) opens the hex-byte insertion modal. `'x'` (lowercase) remains mapped to `Op::XySwap`. The `'X'` interceptor is added in `handle_key()` before `key_to_op()`, setting `PendingInput::HexModal(String::new())`.
- **D-15:** Modal flow mirrors the STO [nn] 2-digit accumulation pattern exactly: `X` → `PendingInput::HexModal(String::new())`; user types first hex digit (0-9, a-f, A-F) → accumulator becomes `"3"`, TUI shows `"HEX: 3_"`; user types second hex digit → validate, insert or reject. Esc cancels at any point with no side effects.
- **D-16:** Insertion position: at the current `state.pc` position (insert before current step, shifting existing steps). After insertion, `state.pc` advances to the next step. This mirrors HP-41 PRGM mode behavior.
- **D-17:** TUI modal display string: `"HEX: _"` when accumulator is empty, `"HEX: n_"` after first digit. Added to `ui.rs` pending_input display block.
- **D-18:** Help overlay entry: `"X nn"` → "Insert synthetic hex byte (PRGM mode)".

### Claude's Discretion

- Exact HP-41 key code lookup table contents (which crossterm KeyCode maps to which HP-41 row-column code). Claude defines the table based on HP-41 documentation; user reviews if something looks wrong.
- Whether the key code update happens before or after `flush_entry_buf()` in `handle_key()` — Claude picks the correct order.
- Exact contents of the safe hex subset lookup table (which codes map to which Ops).
- Program listing display name for `Op::SyntheticByte(nn)`: `"SYN nn"` (hex) or similar — Claude picks a readable format.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/ROADMAP.md` — Phase 12 goal, 4 success criteria (SYNT-01/02/03/04)
- `.planning/REQUIREMENTS.md` — SYNT-01/02/03/04 acceptance criteria; SYNT-05/06 explicitly deferred to v2+

### Core Implementation Files

- `hp41-core/src/state.rs` — `CalcState` struct (add `last_key_code: u8`, `reg_m/reg_n/reg_o: HpNum` with `#[serde(default)]`); `Stack` struct
- `hp41-core/src/ops/mod.rs` — `Op` enum (add GetKey, Null, StoM/StoN/StoO, RclM/RclN/RclO, SyntheticByte(u8)), `dispatch()` (add arms for all new Ops)
- `hp41-core/src/ops/program.rs` — `execute_op()` (add arms for all new Ops — CRITICAL: must mirror dispatch())
- `hp41-core/src/ops/registers.rs` — `op_sto_reg()`, `op_rcl_reg()` pattern to follow for StoM/RclM/etc.
- `hp41-cli/src/app.rs` — `PendingInput` enum (add `HexModal(String)`), `handle_key()` (add `'X'` interceptor, update `last_key_code` on every key press), `handle_pending_input()` (add `HexModal` arm, M/N/O branch in StoRegister/RclRegister arms)
- `hp41-cli/src/keys.rs` — HP-41 key code lookup table (new: `keycode_to_hp41_code()`)
- `hp41-cli/src/ui.rs` — pending_input display block (add `HexModal` → `"HEX: _"` / `"HEX: n_"`)
- `hp41-cli/src/prgm_display.rs` — `op_display_name()` (add arms for GetKey, Null, StoM/N/O, RclM/N/O, SyntheticByte)
- `hp41-cli/src/help_data.rs` — add entry for `"X nn"` hex modal

### Prior Phase Context

- `.planning/phases/11-print-emulation/11-CONTEXT.md` — print modal pattern (D-06 through D-09) that the hex modal mirrors
- `.planning/phases/10-sto-arithmetic-modals/10-CONTEXT.md` — STO arithmetic modal pattern; hidden register M/N/O modal extends the same `StoRegister` flow

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `PendingInput` enum + `handle_pending_input()` in `hp41-cli/src/app.rs` — the modal state machine. `HexModal(String)` follows the exact same 2-char accumulation pattern as `StoRegister(String)` / `RclRegister(String)`.
- `StoRegister` / `RclRegister` modal arms in `handle_pending_input()` — extend these to check for `'M'`/`'N'`/`'O'` input before the normal digit accumulation path.
- `op_sto_reg()` and `op_rcl_reg()` in `hp41-core/src/ops/registers.rs` — exact pattern to follow for `op_sto_m()`, `op_rcl_m()` etc.
- `app.message: Option<String>` — used for INVALID error display (D-13), same as existing error messages.
- `KEY_REF_TABLE` in `hp41-cli/src/keys.rs` — add `"X nn"` entry for hex modal discoverability.

### Established Patterns

- **Critical trap:** Every new `Op` variant must be added to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs`. Missing either causes silent program step skips.
- New `CalcState` fields → `#[serde(default)]` for backward compatibility with v1.1 save files.
- `#![deny(clippy::unwrap_used)]` in `hp41-core` — all new core code uses `.expect("reason")` or `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`.
- Modal interceptors in `handle_key()`: check `pending_input` first, THEN check modal-opening keys (`'S'`/`'R'`/`'P'`/`'X'`), THEN call `key_to_op()`.
- `LiftEffect::Neutral` for ops that don't produce/consume stack values; `LiftEffect::Enable` for ops that push a new value to X.
- `prgm_display.rs::op_display_name()` must be exhaustive — adding a new `Op` variant without a match arm will cause a compiler warning (non-exhaustive match).

### Integration Points

- `hp41-core/src/ops/mod.rs` `Op` enum: add `GetKey`, `Null`, `StoM`, `StoN`, `StoO`, `RclM`, `RclN`, `RclO`, `SyntheticByte(u8)` variants.
- `hp41-core/src/ops/mod.rs` `dispatch()`: add arms for all 9 new Ops.
- `hp41-core/src/ops/program.rs` `execute_op()`: add matching arms for all 9.
- `hp41-core/src/state.rs` `CalcState`: add `last_key_code: u8`, `reg_m: HpNum`, `reg_n: HpNum`, `reg_o: HpNum`.
- `hp41-cli/src/app.rs` `handle_key()`: (1) update `self.state.last_key_code` on every key event before dispatch; (2) add `'X'` interceptor for hex modal; (3) add `'M'`/`'N'`/`'O'` branch in the `StoRegister`/`RclRegister` arms of `handle_pending_input()`.
- `hp41-cli/src/prgm_display.rs` `op_display_name()`: add 9 new match arms.

</code_context>

<specifics>
## Specific Ideas

- For GETKEY key code update: call `self.state.last_key_code = keycode_to_hp41_code(key.code)` at the top of `handle_key()` (after release filter, before any modal/dispatch logic). This ensures the code is updated for every user key press, including digit entry and modal navigation.
- For the HP-41 key code table: HP-41C key layout is 8 rows × 5 columns. Key code = row×10 + col. Example codes: `11`=Σ+, `12`=1/x, `13`=√x, `14`=LOG, `15`=LN; `21`=XEQ, `22`=STO, `23`=RCL, `24`=R↓, `25`=SIN; row 7 is `71`=ENTER, `72`=CHS, `73`=EEX, `74`=R/S; row 8 is `81`=÷, `82`=×, `83`=−, `84`=+, `85`=unused on some models. The full HP-41C keycode table is in HP-41C Owner's Manual.
- For `Op::SyntheticByte` display in `prgm_display.rs`: `format!("SYN {:02X}", byte)` — uppercase hex, zero-padded. E.g., `"SYN A3"`.
- For M/N/O modal extension: in the `StoRegister(ref acc)` arm of `handle_pending_input()`, if `acc.is_empty()` and the key is `'M'`/`'N'`/`'O'`, immediately dispatch `Op::StoM`/`Op::StoN`/`Op::StoO` and clear `pending_input`. If `acc` is non-empty, ignore M/N/O (user already started typing a number).

</specifics>

<deferred>
## Deferred Ideas

- **SYNT-05: Full FOCAL byte-code table (~200 codes)** — explicitly deferred to v2+ in REQUIREMENTS.md. Phase 12 covers only codes mapping to already-implemented Ops.
- **SYNT-06: Interactive GETKEY (program pauses waiting for next key)** — requires event loop redesign (program execution would need to yield to the event loop), deferred to v2+.
- **Indirect addressing for M/N/O** — e.g., `STO IND M` — deferred to v1.2+ along with STOA-04.
- **PRGM mode insertion polish** — e.g., step-delete, step-browse navigation — not in scope for Phase 12.

</deferred>

---

*Phase: 12-Synthetic-Programming*
*Context gathered: 2026-05-08*
