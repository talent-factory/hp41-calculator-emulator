---
phase: 25-cli-integration-and-documentation
plan: 02
type: execute
wave: 2
depends_on: [01]
files_modified:
  - hp41-cli/src/app.rs
  - hp41-cli/src/keys.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/tests/phase25_pending_input.rs
autonomous: true
requirements:
  - FN-CLI-02
  - FN-CLI-04
  - FN-CLI-01
user_setup: []
tags:
  - cli
  - modal
  - pending-input

must_haves:
  truths:
    - "User can open a FlagPrompt modal for SF / CF / FS? / FC? / FS?C / FC?C; status bar shows e.g. 'SF [__]'"
    - "User can open a RegisterPrompt modal for STO / RCL / STO+ / STO- / STO× / STO÷ / VIEW / ARCL / ASTO / ISG / DSE; status bar shows e.g. 'STO [__]'"
    - "Inside an active FlagPrompt or RegisterPrompt modal, pressing 'f' followed by '0' toggles the IND flag (hardware-faithful per QRG p.14 + Pitfall 10); status bar updates to e.g. 'STO IND [__]'; pressing again toggles back"
    - "After 2-digit accumulation, dispatch chooses Op::*Ind(n) when ind==true and Op::*(n) when ind==false — single decision point at end of input"
    - "User can open ClpLabel modal for CLP \"name\" (Enter dispatches Op::Clp, capped at 7 chars), DelCount modal for DEL nnn (3-digit numeric, silent-clamp at u8::MAX), TonePrompt modal for TONE n (single-digit 0–9 auto-dispatches), XeqByName modal for XEQ \"NAME\" (capped at 24 chars)"
    - "pending_prompt() in hp41-cli/src/ui.rs is exhaustive — no `_ =>` catch-all, no `unreachable!()`; adding a new PendingInput variant forces every match site to add an arm at compile time (FN-CLI-04)"
    - "Pressing 'S' / 'R' from the primary keyboard opens RegisterPrompt with op=Sto/Rcl; pressing the f-shifted register-operation keys (`SF`/`CF`/`FS?`/`FC?`/`VIEW`/`ARCL`/`ASTO`/`ISG`/`DSE`) opens the appropriate `FlagPrompt` or `RegisterPrompt` variant"
    - "STO-arithmetic (STO+ / STO- / STO× / STO÷) remains reachable via the existing v1.1 `S → +/-/×/÷ → register` modal chain — NO f-shifted opener in Phase 25 per D-25.7 keyboard layout (f-arith keys are LOCKED to the 4 conditional tests X=Y, X≤Y, X>Y, X=0 per Plan 01)"
    - "All 12 existing PendingInput variants (StoRegister, RclRegister, StoAdd, StoSub, StoMul, StoDiv, AssignKey, AssignLabel, ConfirmLoad, FmtDigits, PrintModal, HexModal) continue to compile and dispatch as before — they are NOT yet removed in this plan; deprecation is tracked for Plan 04"
  artifacts:
    - path: "hp41-cli/src/app.rs"
      provides: "6 new PendingInput variants (FlagPrompt struct, RegisterPrompt struct, ClpLabel, DelCount, TonePrompt, XeqByName) + handle_pending_input arms + handle_reg_modal_with_ind scaffold + IND-toggle logic (reuses App.shift_armed from Plan 01 per W2 fix — no new shift_pending field) + modal openers wired to shifted_key_to_op return paths"
      contains: "FlagPrompt"
    - path: "hp41-cli/src/keys.rs"
      provides: "Two TUI-local discriminator enums (FlagPromptKind wrapping hp41_core::FlagTestKind; RegisterOpKind wrapping hp41_core::StoArithKind) + shifted_key_to_op extended with modal-opener f-shifted bindings (returns None when a modal is opened inline by handle_key)"
      contains: "RegisterOpKind"
    - path: "hp41-cli/src/ui.rs"
      provides: "pending_prompt() extended with 6 new match arms (no _ catch-all, no unreachable)"
      contains: "FlagPrompt { kind, ind, acc }"
    - path: "hp41-cli/tests/phase25_pending_input.rs"
      provides: "Tests for FlagPrompt/RegisterPrompt/ClpLabel/DelCount/TonePrompt/XeqByName variants + IND toggle via shift-0 + Esc cancel + 2/3-digit accumulator behavior + exhaustive-match compile guarantee"
      contains: "test_register_prompt_ind_toggle"
  key_links:
    - from: "hp41-cli/src/app.rs::handle_pending_input"
      to: "hp41_core::ops::Op (StoInd / RclInd / StoArithInd / ArclInd / AstoInd / IsgInd / DseInd / ViewInd / SfFlagInd / CfFlagInd / FlagTestInd)"
      via: "End-of-accumulation match: `if ind { Op::*Ind(n) } else { Op::*(n) }`"
      pattern: "Op::StoInd|Op::RclInd|Op::FlagTestInd"
    - from: "hp41-cli/src/app.rs::handle_pending_input (IND-toggle branch)"
      to: "App.shift_armed (Plan 01 — reused, NOT shadowed by a new shift_pending field per W2 fix)"
      via: "Pressing 'f' inside FlagPrompt/RegisterPrompt arms App.shift_armed; subsequent '0' toggles ind field and clears shift_armed"
      pattern: "shift_armed"
---

<objective>
Land the Hybrid PendingInput modal architecture per D-25.11 — extend the existing 12-variant enum with 6 new variants (2 struct-group: FlagPrompt + RegisterPrompt; 4 specialty: ClpLabel + DelCount + TonePrompt + XeqByName), implement IND-toggle via the hardware-faithful shift-0 keystroke per D-25.12 + Pitfall 10, extend pending_prompt() exhaustively per D-25.14 / FN-CLI-04, and wire f-shifted modal openers (SF/CF/FS?/FC?/FS?C/FC?C, VIEW, STO+/-/×/÷, ARCL, ASTO, ISG, DSE) through shifted_key_to_op (Plan 01) into the new modal variants.

Purpose: This plan closes the FN-CLI-02 + FN-CLI-04 requirements. It depends on Plan 01 (shift_armed must exist before IND-toggle and f-shifted modal openers can be wired). The XEQ-by-Name modal scaffold introduced here is consumed by Plan 03 (the 8 non-keyboard conditional tests resolve via the same modal). The 6 new PendingInput variants and their exhaustive pending_prompt() arms become the basis for the JSON-driven key_path field in Plan 04.

Output: PendingInput enum carries 18 variants (12 legacy + 6 new); handle_pending_input services all 18 with the existing 2-digit / text / 3-digit / 1-digit accumulator scaffolds; pending_prompt() match is exhaustive without `_` arms; shifted_key_to_op opens the right modal variant on every f-shifted modal-opener key; IND-toggle via shift-0 inside a modal flips `ind: bool` and dispatch picks Op::*Ind vs Op::*(n) at end-of-accumulation.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md
@.planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md
@.planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md
@.planning/phases/25-cli-integration-and-documentation/25-01-SUMMARY.md
@CLAUDE.md

<interfaces>
<!-- Key types and contracts for this plan. -->
<!-- Plan 01 introduced App.shift_armed; this plan REUSES it for IND-toggle without modification. -->

From hp41-core/src/ops/mod.rs (DO NOT MODIFY — wire these existing variants from the CLI side):

  Phase 22:
  - `Op::Clp(String)` — clear program by label name
  - `Op::Del(u8)` — delete N program steps from current pc
  - `Op::Stop`, `Op::Pse`
  - `Op::GtoInd(u8)`, `Op::XeqInd(u8)`

  Phase 21:
  - `pub enum FlagTestKind { IsSet, IsClear, IsSetThenClear, IsClearThenClear }`
  - `Op::FlagTest { kind: FlagTestKind, flag: u8 }` (struct variant — precedent for D-25.11)
  - `Op::SfFlag(u8)`, `Op::CfFlag(u8)`
  - `Op::View(u8)`, `Op::AView`, `Op::Prompt`, `Op::Aon`, `Op::Aoff`, `Op::Cld`
  - `Op::Beep`, `Op::Tone(u8)`

  Phase 23:
  - `Op::Arcl(u8)`, `Op::Asto(u8)`, `Op::Atox`, `Op::Xtoa`, `Op::Arot`, `Op::Posa`

  Phase 24 (IND variants — wired here):
  - `Op::StoInd(u8)`, `Op::RclInd(u8)`
  - `Op::StoArithInd(u8, StoArithKind)` (existing struct-variant kind reuse)
  - `Op::IsgInd(u8)`, `Op::DseInd(u8)`
  - `Op::SfFlagInd(u8)`, `Op::CfFlagInd(u8)`
  - `Op::FlagTestInd { kind: FlagTestKind, flag: u8 }`
  - `Op::ArclInd(u8)`, `Op::AstoInd(u8)`, `Op::ViewInd(u8)`

  Phase 9 (reused):
  - `pub enum StoArithKind { Add, Sub, Mul, Div }`
  - `Op::StoArith { reg: u8, kind: StoArithKind }`

From hp41-cli/src/app.rs (existing 12-variant enum — lines 23–41 — PRESERVE all 12; ADD 6 new):
- `pub enum PendingInput { StoRegister(String), RclRegister(String), StoAdd(String), StoSub(String), StoMul(String), StoDiv(String), AssignKey, AssignLabel(char, String), ConfirmLoad(usize), FmtDigits(hp41_core::DisplayMode), PrintModal, HexModal(String) }`

Phase 25 NEW variants (canonical names — match the exhaustive match arms downstream):
- `FlagPrompt { kind: FlagPromptKind, ind: bool, acc: String }`
- `RegisterPrompt { op: RegisterOpKind, ind: bool, acc: String }`
- `ClpLabel(String)`
- `DelCount(String)`
- `TonePrompt`
- `XeqByName(String)`

From hp41-cli/src/app.rs::handle_reg_modal (lines 913–949 — generic 2-digit accumulator pattern to REUSE):
- Signature: `fn handle_reg_modal(&mut self, key: KeyEvent, acc: String, op_fn: impl Fn(u8) -> Op, pending_fn: impl Fn(String) -> PendingInput)`

From hp41-cli/src/app.rs::AssignLabel arm (lines 746–787 — text-input accumulator pattern to REUSE for ClpLabel / XeqByName):
- Enter → dispatch, Backspace → pop, Esc → cancel, Char → push (with length cap)

From hp41-cli/src/ui.rs::pending_prompt (lines 238–273 — exhaustive match — EXTEND with 6 arms, NO `_ =>`).

TUI-local enums (NEW — defined in hp41-cli/src/keys.rs or new submodule app::modal_kinds):
- `pub enum FlagPromptKind { SetFlag, ClearFlag, Test(hp41_core::ops::FlagTestKind) }` — 6 logical variants via inner FlagTestKind reuse
- `pub enum RegisterOpKind { Sto, Rcl, StoArith(hp41_core::ops::StoArithKind), View, Arcl, Asto, Isg, Dse }` — 11 logical variants via StoArithKind reuse

</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add 6 new PendingInput variants + TUI-local discriminator enums + extend pending_prompt() exhaustively</name>
  <files>hp41-cli/src/app.rs, hp41-cli/src/keys.rs, hp41-cli/src/ui.rs</files>
  <read_first>
    - hp41-cli/src/app.rs (lines 23–41 — existing PendingInput enum; lines 746–787 — AssignLabel text-accumulator pattern; lines 913–949 — handle_reg_modal 2-digit pattern)
    - hp41-cli/src/ui.rs (lines 238–273 — existing pending_prompt exhaustive match)
    - hp41-core/src/ops/mod.rs (locate FlagTestKind + StoArithKind enum definitions for the `use` import)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"Pattern 2: Hybrid PendingInput Struct-Variants" + §"Pattern 4: Exhaustive Match Discipline"
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (PendingInput enum section + pending_prompt extension section)
  </read_first>
  <behavior>
    - PendingInput grows from 12 variants (12 existing + 0 added before this plan) to 18 variants by ADDING (not replacing) 6: FlagPrompt, RegisterPrompt, ClpLabel, DelCount, TonePrompt, XeqByName.
    - FlagPromptKind is a new TUI-local enum with three variants: SetFlag, ClearFlag, Test(hp41_core::ops::FlagTestKind). Total 2 + 4 = 6 logical flag-op variants reached via this kind.
    - RegisterOpKind is a new TUI-local enum with: Sto, Rcl, StoArith(hp41_core::ops::StoArithKind), View, Arcl, Asto, Isg, Dse. Total 7 + 4 = 11 logical register-op variants.
    - pending_prompt() in ui.rs handles every variant via destructured pattern match. No `_ =>` catch-all. No `unreachable!()`. The compiler emits an exhaustiveness error if any new variant lands without an arm — that IS the FN-CLI-04 guarantee.
    - Status-bar formatting per Pattern 4 reference (use it verbatim where listed): "SF [__]", "SF IND [1_]", "STO [__]", "STO IND [1_]", "STO+ [__]", "STO× [__]", "STO÷ [__]", "VIEW [__]", "ARCL [__]", "ASTO [__]", "ISG [__]", "DSE [__]", "CLP [name]_ ", "DEL [___]", "TONE [_]", "XEQ \"name\"_". Use the Unicode form `STO×` / `STO÷` (existing precedent at ui.rs:245/246).
  </behavior>
  <action>
    Add the new TUI-local discriminator enums `FlagPromptKind` and `RegisterOpKind` in hp41-cli/src/keys.rs (or in a new `hp41-cli/src/modal_kinds.rs` submodule — executor's choice; just keep them OUT of hp41-core). Both enums derive `Debug, Clone`. FlagPromptKind wraps `hp41_core::ops::FlagTestKind` via its `Test(FlagTestKind)` arm. RegisterOpKind wraps `hp41_core::ops::StoArithKind` via its `StoArith(StoArithKind)` arm. Do NOT define a parallel set of these enums in hp41-core (D-25.13 reuse rule).

    In hp41-cli/src/app.rs, EXTEND the existing PendingInput enum (line 23) with these 6 new variants at the bottom, BEFORE the closing brace, KEEPING all 11 existing variants:
    1. `FlagPrompt { kind: FlagPromptKind, ind: bool, acc: String }` (struct-variant for D-25.11 group)
    2. `RegisterPrompt { op: RegisterOpKind, ind: bool, acc: String }` (struct-variant)
    3. `ClpLabel(String)` (specialty tuple-variant — text label accumulator)
    4. `DelCount(String)` (specialty — 3-digit numeric accumulator)
    5. `TonePrompt` (specialty — unit variant; auto-dispatches on first digit)
    6. `XeqByName(String)` (specialty — XEQ "NAME" text accumulator; scaffold for Plan 03)

    In hp41-cli/src/ui.rs::pending_prompt (line 238), ADD exactly 6 new match arms (one per new variant), in the same style as the existing 11 arms. Use destructured patterns for the struct-variants (`PendingInput::FlagPrompt { kind, ind, acc }` etc.). Inner `match kind` / `match op` arms format the mnemonic prefix; `ind_str = if *ind { " IND" } else { "" }`; final format string e.g. `format!("{mnemonic}{ind_str} [{acc:_<2}]")`. For ClpLabel use `format!("CLP [{acc}]_ ")`; for DelCount use `format!("DEL [{acc:_<3}]")`; for TonePrompt use `"TONE [_]".to_string()`; for XeqByName use `format!("XEQ \"{acc}\"_")`. NO `_ =>` catch-all anywhere. NO `unreachable!()`.

    Add imports at the top of ui.rs as needed: `use hp41_core::ops::{FlagTestKind, StoArithKind};` and import the new TUI-local enums via `use crate::keys::{FlagPromptKind, RegisterOpKind};` (or `use crate::modal_kinds::*;` if you chose the submodule layout).

    Use `.expect("…")` not `.unwrap()`. No fenced code blocks in this action prose — refer to <interfaces> for canonical names and to 25-PATTERNS.md for status-bar formatting strings.
  </action>
  <verify>
    <automated>cargo build -p hp41-cli && cargo test -p hp41-cli --test phase25_pending_input -- pending_input_variants_compile pending_prompt_exhaustive</automated>
  </verify>
  <acceptance_criteria>
    - `grep -nE "(FlagPrompt|RegisterPrompt|ClpLabel|DelCount|TonePrompt|XeqByName)" hp41-cli/src/app.rs | grep -v '^[^:]*:[[:space:]]*//' | wc -l` ≥ 6 (one per new variant — at least one match each in PendingInput enum body)
    - `grep -n "pub enum FlagPromptKind" hp41-cli/src/keys.rs hp41-cli/src/modal_kinds.rs 2>/dev/null` returns exactly 1 line across the candidate files
    - `grep -n "pub enum RegisterOpKind" hp41-cli/src/keys.rs hp41-cli/src/modal_kinds.rs 2>/dev/null` returns exactly 1 line
    - In hp41-cli/src/ui.rs::pending_prompt body: `grep -nE "_ =>|unreachable!\\(" hp41-cli/src/ui.rs | grep -v '^[^:]*:[[:space:]]*//'` returns 0 lines inside the pending_prompt function (the function is the exhaustive guarantee — `_ =>` is permitted in OTHER functions in ui.rs but NOT in pending_prompt). Verify by reading lines 238–290.
    - `cargo build -p hp41-cli` compiles with zero warnings
    - `cargo clippy -p hp41-cli -- -D warnings` passes
    - All 6 new PendingInput variants visible via `grep` (matches under FlagPrompt etc.)
  </acceptance_criteria>
  <done>
    PendingInput carries 18 variants total (12 legacy + 6 new); FlagPromptKind + RegisterOpKind defined as TUI-local enums wrapping hp41-core FlagTestKind + StoArithKind per D-25.13; pending_prompt() in ui.rs is exhaustive with no `_ =>` catch-all (FN-CLI-04 compile-time guarantee active); build is clean.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement handle_pending_input arms for 6 new variants (with IND-toggle via shift-0) + wire shifted_key_to_op modal openers</name>
  <files>hp41-cli/src/app.rs, hp41-cli/src/keys.rs</files>
  <read_first>
    - hp41-cli/src/app.rs (Task 1 output — new PendingInput variants must already exist; existing handle_pending_input at the various Some(PendingInput::…) arms; handle_reg_modal at lines 913–949 — REUSE pattern)
    - hp41-cli/src/keys.rs (Plan 01 output — shifted_key_to_op stub for the 4 conditional tests; this task EXTENDS it with modal-opener arms)
    - .planning/phases/25-cli-integration-and-documentation/25-RESEARCH.md §"Common Pitfalls" #10 (IND = shift-0) + §"Code Examples" handle_reg_modal_with_ind
    - .planning/phases/25-cli-integration-and-documentation/25-PATTERNS.md (handle_pending_input arms section + shifted_key_to_op extension)
  </read_first>
  <behavior>
    - handle_pending_input gains 6 new arms:
      * `Some(PendingInput::FlagPrompt { kind, ind, acc })` — 2-digit numeric accumulator. IND-toggle (reuses `App.shift_armed` from Plan 01 — NO new `shift_pending` field per W2 fix + D-25.12 verbatim): if `key == 'f'` (no Ctrl) AND `app.shift_armed == false`, set `app.shift_armed = true` and return (the modal stays open; the next-key cycle is the toggle); if `app.shift_armed == true && key == '0'`, flip `ind`, set `app.shift_armed = false`, do NOT push '0' into acc. Standard digit `0`–`9` input ONLY runs when `app.shift_armed == false` (otherwise the shift-0 path consumes it). On 2 digits dispatch: per `kind` (`FlagPromptKind::SetFlag` → `Op::SfFlag(n)` or `Op::SfFlagInd(n)`; `ClearFlag` → `CfFlag/CfFlagInd`; `Test(k)` → `FlagTest{kind:k, flag:n}` or `FlagTestInd{kind:k, flag:n}`). Esc cancels (clears `app.shift_armed` to false). Backspace clears acc.
      * `Some(PendingInput::RegisterPrompt { op, ind, acc })` — identical accumulator pattern with the SAME `App.shift_armed`-reuse IND-toggle logic. Final dispatch per `op` variant: Sto → StoReg/StoInd, Rcl → RclReg/RclInd, StoArith(k) → StoArith{reg:n,kind:k}/StoArithInd(n,k), View → View/ViewInd, Arcl → Arcl/ArclInd, Asto → Asto/AstoInd, Isg → Isg/IsgInd, Dse → Dse/DseInd.
      * `Some(PendingInput::ClpLabel(acc))` — text-input modal mirroring AssignLabel; Enter dispatches `Op::Clp(acc.clone())`; cap at 7 chars per HP-41 LBL hardware limit.
      * `Some(PendingInput::DelCount(acc))` — 3-digit accumulator; on 3 digits parses to u8 with `.parse::<u8>().unwrap_or(u8::MAX)` (silent-clamp); dispatches `Op::Del(n)`.
      * `Some(PendingInput::TonePrompt)` — single-digit auto-dispatch; on first 0–9 dispatches `Op::Tone(digit)`; non-digit cancels.
      * `Some(PendingInput::XeqByName(acc))` — text-input modal; Enter dispatches via a Plan-03-supplied resolver path (Plan 02 wires the modal scaffold only; Plan 03 supplies xeq_by_name_local_resolve and the builtin_card_op extension). For Plan 02, Enter falls through to `self.call_dispatch(Op::Xeq(acc.clone()))` which uses the existing 4-name builtin_card_op fallback — the 8 conditional-test mnemonics will resolve in Plan 03. Cap at 24 chars per HP-41 ALPHA register width.
    - NO new `shift_pending: bool` field on App — the modal-local IND-toggle reuses `App.shift_armed` from Plan 01. Reset `app.shift_armed = false` on every dispatch AND on every Esc inside a modal (the existing top-level Plan-01 reset already covers the non-modal case). This honors Pitfall 10 + D-25.12 verbatim and avoids a parallel single-shot bit.
    - shifted_key_to_op extension: when `shift_armed==true` and key matches a modal-opener key per the HP-41CV reference card (RESEARCH §"Key-position table"), return None from shifted_key_to_op but ALSO populate `self.pending_input = Some(PendingInput::FlagPrompt { kind, ind: false, acc: String::new() })` etc. inline in `handle_key` before clearing shift_armed. Concretely: map `f-7` → FlagPrompt{SetFlag}, `f-8` → FlagPrompt{ClearFlag}, `f-9` → FlagPrompt{Test(IsSet)}, `f-4` → FlagPrompt{Test(IsClear)}, `f-5` → FlagPrompt{Test(IsSetThenClear)}, `f-6` → FlagPrompt{Test(IsClearThenClear)}, `f-x` (where 'x' is the f-shifted VIEW position — RESEARCH leaves exact key TBD) → RegisterPrompt{View}. **STO-arithmetic openers (`STO+` / `STO-` / `STO×` / `STO÷`) get NO f-shifted modal-opener arm in Plan 02 (W3 fix).** The f-arith keys (`+`/`-`/`*`/`/`) are LOCKED to the 4 hardware-anchored conditional tests per D-25.7 (Plan 01). STO-arithmetic remains reachable via the existing v1.1 `S → +/-/×/÷ → register` modal chain — `S` opens RegisterPrompt{Sto}; the user picks the arithmetic op via the existing `pending_input` route. Document this routing in 25-02-SUMMARY.md.
    - Add modal openers for the specialty variants: a TUI key opens ClpLabel (RESEARCH leaves the exact key TBD — recommend `f-X` or a dedicated unmapped key; pick one and document); a key opens DelCount; the BEEP/TONE key opens TonePrompt; a key opens XeqByName (RESEARCH §"XEQ-by-Name CLI Modal" recommends `X` outside PRGM mode — preserves existing HexModal binding inside PRGM mode).
  </behavior>
  <action>
    Step 1 — App field: NO new field added (W2 fix per the 2026-05-14 plan revision). The modal-local IND-toggle reuses `App.shift_armed` from Plan 01. If a prior draft of this plan added `pub shift_pending: bool` — REMOVE it.

    Step 2 — handle_pending_input arms: insert 6 new `Some(PendingInput::…) =>` arms inside the existing handle_pending_input match. Reuse the patterns from <interfaces>: handle_reg_modal at lines 913–949 (2-digit numeric); AssignLabel at lines 746–787 (text-input). For FlagPrompt and RegisterPrompt arms, encode the IND-toggle as: at the TOP of the arm body, first check `key.code == KeyCode::Char('f') && !key.modifiers.contains(KeyModifiers::CONTROL) && !self.shift_armed` → set `self.shift_armed = true`, store the (kind/op, ind, acc) unchanged back into `self.pending_input`, return. Then check `if self.shift_armed && key.code == KeyCode::Char('0')` → flip the local `ind`, set `self.shift_armed = false`, store back. Otherwise fall through to the standard digit-accumulator logic. Use `App.shift_armed` (the Plan-01 one-shot bit) — do NOT create a parallel `shift_pending` field.

    For RegisterPrompt's final-dispatch decision, write an inline `let final_op = match (op.clone(), ind) { (RegisterOpKind::Sto, false) => Op::StoReg(n), (RegisterOpKind::Sto, true) => Op::StoInd(n), (RegisterOpKind::Rcl, false) => Op::RclReg(n), (RegisterOpKind::Rcl, true) => Op::RclInd(n), (RegisterOpKind::StoArith(k), false) => Op::StoArith{reg:n,kind:k}, (RegisterOpKind::StoArith(k), true) => Op::StoArithInd(n, k), (RegisterOpKind::View, false) => Op::View(n), (RegisterOpKind::View, true) => Op::ViewInd(n), (RegisterOpKind::Arcl, false) => Op::Arcl(n), (RegisterOpKind::Arcl, true) => Op::ArclInd(n), (RegisterOpKind::Asto, false) => Op::Asto(n), (RegisterOpKind::Asto, true) => Op::AstoInd(n), (RegisterOpKind::Isg, false) => Op::Isg(n), (RegisterOpKind::Isg, true) => Op::IsgInd(n), (RegisterOpKind::Dse, false) => Op::Dse(n), (RegisterOpKind::Dse, true) => Op::DseInd(n) };` then call `self.call_dispatch(final_op)`. Same structure for FlagPrompt.

    Step 3 — shifted_key_to_op extension: extend hp41-cli/src/keys.rs::shifted_key_to_op (added in Plan 01) with f-shifted MODAL-OPENER arms that return None but populate `app.pending_input` via a mutable-borrow path. SHIFT THE SIGNATURE if needed: change to `pub fn shifted_key_to_op(key: KeyEvent, app: &mut App) -> Option<Op>` (mutable reference). Update the Plan-01 call site in app.rs::handle_key accordingly. Inside shifted_key_to_op, on match arms for modal-opener f-shifted keys (per <behavior> mapping), set `app.pending_input = Some(PendingInput::FlagPrompt { kind: <appropriate>, ind: false, acc: String::new() })` and return None. The conditional-test arms (Plan 01) still return Some(Op::Test(…)). Document the exact key→modal mapping in a new top-of-file table-comment.

    Step 4 — specialty modal openers: in handle_key (BEFORE the key_to_op fallthrough but AFTER shift_armed consumption), add openers for ClpLabel/DelCount/TonePrompt/XeqByName per <behavior>. For XeqByName: bind `X` (uppercase) outside PRGM mode to `self.pending_input = Some(PendingInput::XeqByName(String::new()))`; keep `X` inside PRGM mode pointing to HexModal (existing v1.1 behavior — guard with `if !state.prgm_mode { … } else { … }`).

    Use `.expect("reason")` not `.unwrap()`. No fenced code blocks in this prose — refer to <interfaces> and 25-PATTERNS.md.
  </action>
  <verify>
    <automated>cargo test -p hp41-cli --test phase25_pending_input -- test_ind_toggle_via_shift_0 test_flag_prompt_dispatches test_clp_label_capped test_del_count_silent_clamp test_tone_prompt_auto_dispatch test_xeq_by_name_modal_scaffold</automated>
  </verify>
  <acceptance_criteria>
    - NO `shift_pending: bool` field exists on App (W2 fix): `grep -n "pub shift_pending: bool" hp41-cli/src/app.rs` returns 0 lines
    - The modal-local IND-toggle path references `self.shift_armed` inside the FlagPrompt and RegisterPrompt arms: `grep -n "self.shift_armed" hp41-cli/src/app.rs | wc -l` ≥ 3 (at least one reference in each of the FlagPrompt arm, the RegisterPrompt arm, and the existing Plan-01 top-level handler)
    - `grep -c "Some(PendingInput::FlagPrompt" hp41-cli/src/app.rs` ≥ 1 (handle_pending_input arm exists)
    - `grep -c "Some(PendingInput::RegisterPrompt" hp41-cli/src/app.rs` ≥ 1
    - `grep -c "Some(PendingInput::ClpLabel" hp41-cli/src/app.rs` ≥ 1
    - `grep -c "Some(PendingInput::DelCount" hp41-cli/src/app.rs` ≥ 1
    - `grep -c "Some(PendingInput::TonePrompt" hp41-cli/src/app.rs` ≥ 1
    - `grep -c "Some(PendingInput::XeqByName" hp41-cli/src/app.rs` ≥ 1
    - Final-dispatch tuple-match in RegisterPrompt arm includes ALL 11 RegisterOpKind variants × 2 ind values = 22 arms: `grep -c "RegisterOpKind::" hp41-cli/src/app.rs` ≥ 22 in handle_pending_input area
    - shifted_key_to_op signature was updated to take &mut App: `grep -n "pub fn shifted_key_to_op" hp41-cli/src/keys.rs` shows the &mut App signature
    - Integration test `test_ind_toggle_via_shift_0` passes: with an open RegisterPrompt{Sto, ind:false} modal AND `app.shift_armed = true` (armed by pressing 'f' inside the modal), pressing '0' flips `ind` to true and clears `app.shift_armed`; subsequently pressing '0' then '5' dispatches Op::StoInd(5) and leaves pending_input=None
    - Build clean: `cargo build -p hp41-cli`; `cargo clippy -p hp41-cli -- -D warnings`
  </acceptance_criteria>
  <done>
    handle_pending_input services all 6 new variants with IND-toggle via shift-0 working hardware-faithfully per Pitfall 10; shifted_key_to_op extended to open f-shifted modals; specialty variants (ClpLabel/DelCount/TonePrompt/XeqByName) open via their TUI keys with correct length caps and silent-clamp; all targeted tests GREEN.
  </done>
</task>

<task type="auto">
  <name>Task 3: Integration test scaffold for 6 new PendingInput variants + IND-toggle behavior</name>
  <files>hp41-cli/tests/phase25_pending_input.rs</files>
  <read_first>
    - hp41-cli/tests/phase25_keyboard.rs (Plan 01 output — reuse the `key(c)` helper + App construction pattern)
    - hp41-cli/tests/card_io_tests.rs (lines 1–36 — integration-test scaffold reference)
    - hp41-cli/src/app.rs (Task 1+2 output — new variants and arms must exist; signatures of handle_key and handle_pending_input)
    - .planning/phases/25-cli-integration-and-documentation/25-VALIDATION.md §"Per-Task Verification Map" rows 25-02-01..25-02-03
  </read_first>
  <action>
    Create hp41-cli/tests/phase25_pending_input.rs with `#![allow(clippy::unwrap_used)]` at module head. Imports mirror phase25_keyboard.rs from Plan 01. Provide tests:

    1. `pending_input_variants_compile` — no runtime check; the test simply pattern-matches each new variant once to force a compile-time exhaustiveness check. e.g. construct `PendingInput::FlagPrompt { kind: FlagPromptKind::SetFlag, ind: false, acc: String::new() }`, `PendingInput::RegisterPrompt { op: RegisterOpKind::Sto, ind: false, acc: String::new() }`, `PendingInput::ClpLabel("HELLO".into())`, `PendingInput::DelCount("123".into())`, `PendingInput::TonePrompt`, `PendingInput::XeqByName("FOO".into())`. Assert each `matches!` test returns true.

    2. `pending_prompt_exhaustive` — call `ui::pending_prompt` (re-export the function via `pub(crate)` or via a public helper) for each of the 18 variants and assert the returned string starts with the expected mnemonic ("SF", "STO", "STO IND", "CLP", "DEL", "TONE", "XEQ"). This test is a structural one — if pending_prompt panics or returns wrong prefix, fail.

    3. `test_ind_toggle_via_shift_0` — construct App. Set `app.pending_input = Some(PendingInput::RegisterPrompt { op: RegisterOpKind::Sto, ind: false, acc: String::new() })`. Send `key('f')` (no Ctrl). Assert `app.shift_armed == true` AND `ind` still false (the modal absorbed the `f` and armed the one-shot — W2 fix: reuses `App.shift_armed`, no separate `shift_pending`). Send `key('0')`. Assert `app.shift_armed == false` AND the pending_input is now `Some(RegisterPrompt { op: Sto, ind: true, acc: "" })` (the '0' was consumed by the IND-toggle, not pushed into the accumulator). Send `key('0')` then `key('5')`. Assert the dispatched op was `Op::StoInd(5)` (check via state mutation: pre-set `app.state.regs[5] = HpNum::from(99)`, pre-set `app.state.stack.x = HpNum::from(42)`, after dispatch verify `app.state.regs[5] == HpNum::from(42)` — STO behavior) AND pending_input is None.

    4. `test_flag_prompt_dispatches` — construct App. Send keys to arm f-prefix then `7` (or whatever the F-shifted SF key is — use the canonical mapping from Plan 02 SUMMARY). Assert pending_input is `Some(FlagPrompt { kind: FlagPromptKind::SetFlag, ind: false, acc: "" })`. Send `key('1')` then `key('2')`. Assert dispatched op is `Op::SfFlag(12)` AND `app.state.flags & (1u64 << 12) != 0`. Re-run with IND toggle (shift-0) and assert SfFlagInd dispatch.

    5. `test_clp_label_capped` — open ClpLabel modal. Send 8 character keys. Assert acc length stays ≤ 7 (cap at HP-41 LBL hardware limit). Send Enter. Assert dispatched op is `Op::Clp("name").

    6. `test_del_count_silent_clamp` — open DelCount modal. Send `key('9')`, `key('9')`, `key('9')` (decimal 999, exceeds u8). Assert dispatched op is `Op::Del(u8::MAX)` (silent-clamp per <behavior>).

    7. `test_tone_prompt_auto_dispatch` — open TonePrompt. Send `key('5')`. Assert dispatched op is `Op::Tone(5)` AND `app.state.event_buffer` contains a TONE entry; pending_input is None.

    8. `test_xeq_by_name_modal_scaffold` — open XeqByName. Type `H E L L O` then Enter. Assert dispatched op is `Op::Xeq("HELLO")` (falls through to existing core resolver — the conditional-test mnemonic resolution comes in Plan 03; this test verifies the modal scaffold).

    9. `test_esc_cancels_all_new_variants` — for each of the 6 new variants, open the modal, send Esc, assert pending_input is None and no dispatch occurred.

    Use `.expect("reason")` outside `#[test]` bodies; `.unwrap()` is allowed inside test bodies (module has `#![allow(clippy::unwrap_used)]`).
  </action>
  <verify>
    <automated>cargo test -p hp41-cli --test phase25_pending_input</automated>
  </verify>
  <acceptance_criteria>
    - File hp41-cli/tests/phase25_pending_input.rs exists with `#![allow(clippy::unwrap_used)]`
    - At least 9 `#[test]` functions present: `grep -c "^#\\[test\\]" hp41-cli/tests/phase25_pending_input.rs` ≥ 9
    - `cargo test -p hp41-cli --test phase25_pending_input` exits 0
    - `cargo clippy -p hp41-cli --tests -- -D warnings` passes
  </acceptance_criteria>
  <done>
    Integration tests cover: 6 new variant constructions; pending_prompt exhaustive behavior; IND-toggle via shift-0; FlagPrompt dispatch (direct + IND); ClpLabel cap-at-7; DelCount silent-clamp; TonePrompt auto-dispatch; XeqByName scaffold; Esc-cancel uniformity. All GREEN.
  </done>
</task>

</tasks>

<verification>
- `cargo test -p hp41-cli --test phase25_pending_input` exits 0 with ≥9 tests GREEN.
- `cargo test -p hp41-cli --test phase25_keyboard` (Plan 01 regression) exits 0 — no regression in 4-conditional-test dispatches.
- `cargo build -p hp41-cli` compiles with zero warnings — confirms FN-CLI-04 (exhaustive pending_prompt).
- `cargo clippy -p hp41-cli -- -D warnings` passes.
- `just check` (workspace fmt+clippy+test) GREEN.
- Manual smoke: `just run-cli`; press `S`, observe `STO [__]` modal; press `f` then `0` and observe `STO IND [__]`; press `0` then `5`; verify `R05` shows 0 (or whatever X was). Press a modal-opener f-shifted key (e.g. for SF), observe `SF [__]` modal; press `1` `2` and observe flag 12 set.
</verification>

<success_criteria>
- PendingInput enum carries 18 variants (12 legacy + 6 new): FlagPrompt, RegisterPrompt, ClpLabel, DelCount, TonePrompt, XeqByName.
- FlagPromptKind + RegisterOpKind TUI-local enums defined; both wrap existing hp41-core enums (FlagTestKind, StoArithKind) per D-25.13.
- handle_pending_input services all 18 variants; IND-toggle via shift-0 (Pitfall 10) flips `ind` and end-of-accumulation dispatch picks Op::*Ind vs Op::*(n) at single decision point per D-25.12.
- pending_prompt() in ui.rs is EXHAUSTIVE — no `_ =>` catch-all, no `unreachable!()` — FN-CLI-04 compile-time guarantee active.
- shifted_key_to_op extended with modal-opener arms (mutable App reference); 4 conditional-test arms from Plan 01 untouched.
- NO `App.shift_pending` field (W2 fix); the modal-local IND-toggle reuses `App.shift_armed` from Plan 01 so a single one-shot bit serves both the global f-prefix AND the modal IND-toggle, honoring D-25.12 + Pitfall 10 verbatim.
- All Wave-0 tests in hp41-cli/tests/phase25_pending_input.rs GREEN.
- FN-CLI-02 closed; FN-CLI-04 closed; FN-CLI-01 progresses (every modal-bearing op now has a keyboard path).
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| User keystroke → PendingInput modal accumulator | Untrusted-length string input (text modals); numeric-bounded otherwise |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-25-05 | Denial of Service | Unbounded accumulator length in ClpLabel or XeqByName | mitigate | Hard cap at 7 chars (LBL) / 24 chars (XEQ) via `if acc.len() < CAP { acc.push(c) }` guards (RESEARCH §Security V5); covered by `test_clp_label_capped` |
| T-25-06 | Tampering | DelCount overflow when user types 999 → u16 overflow on `.parse::<u8>()` | mitigate | `.parse::<u8>().unwrap_or(u8::MAX)` silent-clamp pattern; covered by `test_del_count_silent_clamp` |
| T-25-07 | Information Disclosure | `app.shift_armed` leaks across modal dismiss → next modal opens with ind already toggled, OR next non-modal key cycle behaves f-shifted | mitigate | Reset `app.shift_armed = false` on every dispatch AND on every Esc inside a modal (W2 fix reuses the Plan-01 single-shot bit); covered by `test_esc_cancels_all_new_variants` and `test_ind_toggle_via_shift_0` |
| T-25-08 | Tampering | New PendingInput variant added in future without pending_prompt arm → silent runtime mismatch | mitigate | Exhaustive match discipline at ui.rs:238 (FN-CLI-04 hard rule); compiler emits error on missing arm |
</threat_model>

<output>
After completion, create `.planning/phases/25-cli-integration-and-documentation/25-02-SUMMARY.md` per execute-plan template. Record: final modal-opener key-mapping table (which f-shifted keys open which modal variant); the resolved STO-arithmetic conflict (D-25.7 wins, STO-arith stays on legacy S→op→reg path per <behavior>); the canonical XeqByName trigger (X-outside-PRGM-mode); any deferred Plan-03 requirements (specifically that XEQ-by-Name still falls through to 4-name builtin_card_op in this plan and the 8 conditional-test mnemonics resolve only after Plan 03 lands).
</output>
