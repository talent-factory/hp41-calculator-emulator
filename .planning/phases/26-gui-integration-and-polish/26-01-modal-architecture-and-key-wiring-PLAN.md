---
phase: 26-gui-integration-and-polish
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-gui/src-tauri/src/key_map.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src/App.tsx
  - hp41-gui/src/Keyboard.tsx
autonomous: true
requirements:
  - FN-GUI-01
  - FN-GUI-02
  - FN-GUI-05
must_haves:
  truths:
    - "Every HP-41CV ROM op variant added in Phases 20-24 resolves successfully via key_map::resolve or key_map::resolve_parameterized — only v3.x module-Pac names remain in the stub-error arm"
    - "Clicking SHIFT+STO (or any of the 13 prompt-id keys per D-26.5) opens a frontend React modal — no GuiError toast surfaces, no IPC call to dispatch_op fires (with the 4 conditional-test prompts dispatching immediately via the `direct` variant per the revision)"
    - "Inside an open FlagPrompt or RegisterPrompt modal, pressing SHIFT then 0 toggles the modal's ind boolean and updates the LCD preview to show 'IND' (does NOT append '0' to the accumulator)"
    - "End-of-2-digit accumulation in a register-modal dispatches Op::Sto(NN) when ind=false and Op::StoInd(NN) when ind=true — single-point tuple decision per D-26.2"
    - "ASN flow opens AssignKey modal -> press a key -> AssignLabel modal -> type text -> Enter dispatches asn_NN_NAME parameterized id (no new Tauri command needed)"
    - "CalcStateView serializes to JSON ≤ 500 bytes for an empty CalcState plus 0-10 user_keymap entries (FN-GUI-05 budget honored per D-26.11)"
    - "Esc inside an open modal cancels the modal (sets pendingInput to null) AND clears shiftActive — mirrors hp41-cli Phase 25 W3 fix"
    - "DEL prompt accepts values 0..=255 (Op::Del field is u8 in hp41-core, confirmed); values 256+ produce an LCD-rendered error preview before dispatch — hp41-core stays frozen per phase invariant"
    - "CalcStateView TS interface mirror in App.tsx is extended with user_keymap, flags, display_override, event_buffer fields so tsc --noEmit passes"
  artifacts:
    - path: "hp41-gui/src-tauri/src/key_map.rs"
      provides: "Extended bare-op resolver (Pi, PolarToRect, RectToPolar, Rnd, Frc, Abs, Sign, Fact, Mod, RUp, View, Tone, Stop, Pse, Clp, Del, Ins, Size, Cla, Clst, Pack, Catalog, Asn, Arcl, Asto, Atox, Xtoa, Arot, Posa, Beep, AView, Prompt, AOn, AOff, Cld, all *Ind variants — total ~80-90 named arms) + extended parameterized prefixes (sf_NN, sf_ind_NN, cf_NN, cf_ind_NN, fs_NN, fs_ind_NN, view_NN, view_ind_NN, arcl_NN, arcl_ind_NN, asto_NN, asto_ind_NN, tone_N, del_NNN, catalog_N, clp_LABEL, sto_ind_NN, rcl_ind_NN, isg_ind_NN, dse_ind_NN, sto_arith_<op>_ind_NN)"
      contains: "Ok(Op::Pi)"
    - path: "hp41-gui/src-tauri/src/types.rs"
      provides: "CalcStateView projections for user_keymap, flags, display_override, event_buffer per D-26.11; from_state signature extended to (state, print_lines, event_lines); 6 existing test call sites updated; budget assertion raised to 500 bytes per FN-GUI-05"
      contains: "user_keymap"
    - path: "hp41-gui/src-tauri/src/commands.rs"
      provides: "Drain event_buffer alongside print_buffer in handle_op_finalize / handle_get_state / handle_sst_step / handle_bst_step / handle_run_stop (existing drain-then-pass pattern per PATTERNS.md §3 lines 211-215); exactly 5 from_state(...,event_lines) call sites"
      contains: "event_buffer"
    - path: "hp41-gui/src/App.tsx"
      provides: "useState<PendingInput | null> per D-26.1; PendingInput discriminated union per D-26.4 (revised: 14 variants — 12 from D-26.4 + 1 new `single_digit` (Tone/Catalog merge) + 1 new `direct` for conditional-test prompts); handleModalKey() returning struct { nextPending, dispatchId, consumesShift } per the revision; renderModalLcd() preview emitter per D-26.3; handleClick MODAL_OPENERS intercept block per D-26.5 (covering all 13 *_prompt ids); Esc clears both shiftActive and pendingInput; physical-keyboard digits route through handleModalKey when pendingInput is non-null; TS CalcStateView interface mirror extended for the 4 new backend projections"
      contains: "PendingInput"
    - path: "hp41-gui/src/Keyboard.tsx"
      provides: "KEY_DEFS audit: every primary and shifted id present in KEY_DEFS resolves successfully via key_map::resolve OR is a documented modal-opener (the 13 *_prompt ids + asn/view/catalog/xeq_prompt/gto_prompt/lbl_prompt)"
      contains: "KEY_DEFS"
  key_links:
    - from: "hp41-gui/src/App.tsx::handleClick"
      to: "MODAL_OPENERS table"
      via: "intercept BEFORE invokeForKey when effectiveId in MODAL_OPENERS — call setPendingInput(MODAL_OPENERS[effectiveId]())"
      pattern: "MODAL_OPENERS\\["
    - from: "hp41-gui/src/App.tsx::handleModalKey"
      to: "invokeForKey(parameterizedId)"
      via: "end-of-2-digit-accumulation tuple decision: pending.ind ? '<op>_ind_<NN>' : '<op>_<NN>'"
      pattern: "_ind_"
    - from: "hp41-gui/src-tauri/src/key_map.rs::resolve_parameterized"
      to: "Op::*Ind(NN) variants"
      via: "strip_prefix('sto_ind_') BEFORE strip_prefix('sto_') — more-specific-first ordering per PATTERNS.md §1"
      pattern: "strip_prefix\\(\"sto_ind_\"\\)"
    - from: "hp41-gui/src-tauri/src/types.rs::from_state"
      to: "CalcStateView fields user_keymap/flags/display_override/event_buffer"
      via: "drain event_buffer in commands.rs (NOT in from_state) and pass as parameter alongside print_lines"
      pattern: "event_buffer.drain"
    - from: "hp41-gui/src/App.tsx (TS CalcStateView mirror)"
      to: "Rust CalcStateView struct fields"
      via: "TS interface fields user_keymap / flags / display_override / event_buffer mirror the new Rust projections — tsc --noEmit passes after this plan"
      pattern: "user_keymap"
---

<objective>
Wire every v2.2 HP-41CV ROM op into the Tauri GUI's `key_map.rs` resolver and ship the frontend modal infrastructure that intercepts the 13 previously-stubbed prompt-ids and routes them to React modals — closing the parity gap with CLI Phase 25's `PendingInput` hybrid struct-variants per D-25.6.

Purpose: The user explicitly required CLI ↔ GUI behavioral parity (D-25.6). Phase 25 delivered the CLI side; this plan delivers the GUI side. Every clickable HP-41CV key must produce the correct Op (or open the correct modal) with no `unknown key` toasts for any ROM built-in. Modal state lives in React (frontend-owned, mirroring the v2.1 `shiftActive` precedent per D-26.1) — no new IPC surface.

Output:
- Extended `key_map.rs::resolve` (~80 new bare-op arms) and `resolve_parameterized` (~20 new prefixes including IND-bearing variants)
- Shrunk stub-error arm: only v3.x module-Pac names remain (no HP-41CV ROM ops)
- New `PendingInput` discriminated union (14 variants — see <interfaces>) in `App.tsx`, including the `single_digit` variant (Tone/Catalog merge) and the `direct` variant (immediate-dispatch openers for the 4 conditional-test prompts)
- `handleModalKey()` with digit accumulation + IND-toggle via shift-0 + struct-return signature (D-26.2 + revision B6)
- `renderModalLcd()` LCD preview emitter (D-26.3)
- `handleClick` MODAL_OPENERS intercept block (D-26.5)
- `CalcStateView` extended with `user_keymap`/`flags`/`display_override`/`event_buffer` projections (D-26.11) — BOTH the Rust struct AND the TS interface mirror
- Comprehensive Vitest + Rust tests for all new resolver arms and modal state-transitions
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/26-gui-integration-and-polish/26-CONTEXT.md
@.planning/phases/26-gui-integration-and-polish/26-PATTERNS.md
@.planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md
@CLAUDE.md

@hp41-gui/src-tauri/src/key_map.rs
@hp41-gui/src-tauri/src/types.rs
@hp41-gui/src-tauri/src/commands.rs
@hp41-gui/src/App.tsx
@hp41-gui/src/Keyboard.tsx
@hp41-cli/src/app.rs
@hp41-cli/src/ui.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/state.rs

<interfaces>
<!-- Key types and contracts the executor needs. Extracted from codebase + CONTEXT D-26.4 + revision. -->

From hp41-core/src/ops/mod.rs (Op enum — already shipped Phases 20-24, READ-ONLY here):
- Op::Pi, Op::PolarToRect, Op::RectToPolar, Op::Rnd, Op::Frc, Op::Mod, Op::Abs, Op::Fact, Op::Sign, Op::RUp (Phase 20)
- Op::SfFlag(u8), Op::CfFlag(u8), Op::FlagTest { kind: FlagTestKind, flag: u8 } where FlagTestKind = FsQuery | FcQuery | FsQueryClear | FcQueryClear (Phase 21)
- Op::View(u8), Op::AView, Op::Prompt, Op::AOn, Op::AOff, Op::Cld (Phase 21)
- Op::Beep, Op::Tone(u8) (Phase 21)
- Op::Stop, Op::Pse (Phase 22)
- **Op::Del(u8)** (Phase 22) — field width confirmed u8 by reading hp41-core/src/ops/mod.rs line 397: `Del(u8),`. HP-41 hardware native supports DEL 000-999; the u8 cap is a documented v2.2 divergence (BLOCKER B3 resolution: clamp to 0..=255 with explicit modal-side error preview). hp41-core is FROZEN per phase boundary — width change deferred to v3.x.
- Op::Clp(String), Op::Ins (Phase 22)
- Op::GtoInd(u8), Op::XeqInd(u8) (Phase 22)
- Op::Size(u16), Op::Cla, Op::Clst, Op::Pack, Op::Catalog(u8) (Phase 22)
- Op::Asn { name: String, key_code: u8 } (Phase 22)
- Op::Arcl(u8), Op::Asto(u8) (Phase 23)
- Op::Atox, Op::Xtoa, Op::Arot, Op::Posa (Phase 23)
- Op::StoInd(u8), Op::RclInd(u8), Op::IsgInd(u8), Op::DseInd(u8), Op::SfFlagInd(u8), Op::CfFlagInd(u8), Op::FlagTestInd { kind: FlagTestKind, reg: u8 }, Op::ArclInd(u8), Op::AstoInd(u8), Op::ViewInd(u8), Op::StoArithInd { kind: StoArithKind, reg: u8 } (Phase 24, all 11 *Ind variants)
- Op::Test(TestKind) where TestKind = XEqY | XLeY | XGtY | XEqZero (the 4 keyboard-bound conditional tests per Phase 25 D-25.7) — bare ids `x_eq_y`, `x_le_y`, `x_gt_y`, `x_eq_0` already resolve in v2.1.

PendingInput discriminated union per D-26.4 + REVISION (BLOCKERS B1 and B2) (14 variants total — NEW in App.tsx):
```typescript
type FlagTestKind = 'SF' | 'CF' | 'FsQuery' | 'FcQuery' | 'FsQueryClear' | 'FcQueryClear';
type RegisterOpKind = 'Sto' | 'Rcl' | 'StoAdd' | 'StoSub' | 'StoMul' | 'StoDiv'
                    | 'View' | 'Arcl' | 'Asto' | 'Isg' | 'Dse';
type FmtMode = 'fix' | 'sci' | 'eng';
type SingleDigitOp = 'Tone' | 'Catalog';

type PendingInput =
  // 12 base variants from D-26.4
  | { kind: 'flag'; testKind: FlagTestKind; ind: boolean; acc: string }
  | { kind: 'register'; op: RegisterOpKind; ind: boolean; acc: string }
  | { kind: 'clp'; acc: string }
  | { kind: 'del'; acc: string }
  | { kind: 'xeq_name'; acc: string }
  | { kind: 'fmt'; mode: FmtMode }
  | { kind: 'assign_key' }
  | { kind: 'assign_label'; keyCode: number; acc: string }
  | { kind: 'confirm_load'; programIdx: number }
  | { kind: 'hex'; acc: string }
  | { kind: 'print' }
  // REVISION B2: merge Tone + Catalog into one `single_digit` variant with op + max discriminator.
  // Tone accepts 0..=9 (max: 9), Catalog accepts 1..=3 (max: 3). Replaces the prior bare `{ kind: 'tone' }`.
  | { kind: 'single_digit'; op: SingleDigitOp; max: number }
  // REVISION B1: `direct` variant for immediate-dispatch openers — the 4 conditional-test prompt aliases
  // (x_eq_y_prompt, x_le_y_prompt, x_gt_y_prompt, x_eq_0_prompt) route through MODAL_OPENERS via this
  // variant. handleModalKey returns dispatchId=`dispatchId` immediately on the very first invocation;
  // no accumulator, no IND-toggle, no Esc-able state.
  | { kind: 'direct'; dispatchId: string };
```

CalcStateView projections to ADD per D-26.11 (Rust side AND TS interface mirror per BLOCKER B5):
- pub user_keymap: Vec<(u8, String)>     // already on CalcState as `assignments: BTreeMap<u8, String>` per Phase 22 — collect into Vec for serialization
- pub flags: Vec<u8>                       // EITHER raw u64 OR Vec<u8> of set flag indices (planner discretion: pick the smaller per FN-GUI-05 budget; recommend Vec<u8> set indices since typical state has 0-3 flags set)
- pub display_override: Option<String>     // already on CalcState (Phase 21); surface via Option<String> projection
- pub event_buffer: Vec<String>            // drained per IPC like print_lines

TS interface mirror (App.tsx line 14, BLOCKER B5):
```typescript
interface CalcStateView {
  display_str: string;
  x_str: string;
  y_str: string;
  z_str: string;
  t_str: string;
  lastx_str: string;
  in_eex_mode: boolean;
  annunciators: Annunciators;
  print_lines: string[];
  program_steps: string[];
  pc: number;
  // Phase 26 D-26.11 (BLOCKER B5):
  user_keymap: Array<[number, string]>;
  flags: number[];          // Vec<u8> of set-flag indices (matches Rust projection shape)
  display_override: string | null;
  event_buffer: string[];
}
```

handleModalKey signature (BLOCKER B6 — struct-return form):
```typescript
function handleModalKey(
  key: string,
  pending: PendingInput,
  shiftActive: boolean
): { nextPending: PendingInput | null; dispatchId: string | null; consumesShift: boolean };
```
- `nextPending`: PendingInput to keep the modal open with updated state, OR null to close the modal
- `dispatchId`: parameterized op id to dispatch via invokeForKey, OR null if no dispatch this turn
- `consumesShift`: true if the keystroke consumed shiftActive (e.g. IND-toggle path) — caller must setShiftActive(false)

CONTEXT.md decisions cited: D-26.1, D-26.2, D-26.3, D-26.4 (extended by revision), D-26.5, D-26.11
</interfaces>

</context>

<tasks>

<task type="execute">
  <name>Task 1: Extend key_map.rs resolver — bare ops + parameterized prefixes + shrunk stub arm + DEL u8-clamp + flag-prompt stub-arm audit + Rust tests</name>
  <files>hp41-gui/src-tauri/src/key_map.rs</files>
  <read_first>
    - hp41-gui/src-tauri/src/key_map.rs (full 412 lines — current resolve/resolve_parameterized + tests)
    - hp41-core/src/ops/mod.rs (Op enum — line 397 confirms `Del(u8),`; also confirm exact variant names for Phase 20-24 ops; particularly the FlagTest/FlagTestInd struct-variant shape and StoArithInd/StoArithKind shape)
    - hp41-cli/src/keys.rs (CLI-side parallel — confirm naming conventions for the new keyboard ids; the GUI key_map ids must match what the GUI Keyboard.tsx and CLI keys.rs both use to avoid drift)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src-tauri/src/key_map.rs (extend)" lines 27-115 (bare-op pattern, stub-arm shrink, parameterized-prefix more-specific-first ordering, sto_arith_ multi-segment parse)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.5 + "Integration Points" subsection lines 199-207 (full list of new prefixes)
  </read_first>
  <action>
Extend `pub fn resolve(key_id: &str)` with named-op resolvers for every v2.2 Op that has a keyboard-reachable bare id. Required new arms (group by section comment, follow PATTERNS.md §"key_map.rs" exact format `"<id>" => Ok(Op::<Variant>),`):

Math/stack section (Phase 20): `pi` -> Op::Pi, `polar_to_rect` -> Op::PolarToRect, `rect_to_polar` -> Op::RectToPolar, `rnd` -> Op::Rnd, `frc` -> Op::Frc, `mod_op` -> Op::Mod, `abs` -> Op::Abs, `fact` -> Op::Fact, `sign` -> Op::Sign, `r_up` -> Op::RUp.

Display/sound section (Phase 21): `aview` -> Op::AView, `prompt` -> Op::Prompt, `aon` -> Op::AOn, `aoff` -> Op::AOff, `cld` -> Op::Cld, `beep` -> Op::Beep.

Program control section (Phase 22): `stop` -> Op::Stop, `pse` -> Op::Pse, `ins` -> Op::Ins.

Memory section (Phase 22): `cla` -> Op::Cla, `clst` -> Op::Clst, `pack` -> Op::Pack.

ALPHA section (Phase 23): `atox` -> Op::Atox, `xtoa` -> Op::Xtoa, `arot` -> Op::Arot, `posa` -> Op::Posa.

Note: `sf`/`cf`/`fs_q`/`fc_q`/`fs_qc`/`fc_qc`/`view`/`tone`/`clp`/`del`/`catalog`/`asn`/`arcl`/`asto`/`size` are MODAL openers — they STAY in the stub-error arm of `resolve` per D-26.5 (frontend intercepts in handleClick; the stub is defense-in-depth; never reaches resolve in practice, the test_modal_prompt_ids_are_stubs_for_now contract continues). Document this in a comment block above the bare-op section.

Extend `fn resolve_parameterized(key_id: &str)` with the new prefixes. CRITICAL: more-specific-first ordering — `strip_prefix("sto_ind_")` MUST appear BEFORE `strip_prefix("sto_")`; same for rcl/isg/dse/sf/cf/fs/view/arcl/asto/sto_arith. New prefixes:
- `sto_ind_NN` -> Op::StoInd(parse_u8(rest)?), and the existing `sto_NN` arm stays (Op::StoReg/Op::Sto as currently named)
- `rcl_ind_NN` -> Op::RclInd(NN)
- `isg_ind_NN` -> Op::IsgInd(NN); `dse_ind_NN` -> Op::DseInd(NN)
- `sf_NN` -> Op::SfFlag(NN); `sf_ind_NN` -> Op::SfFlagInd(NN)
- `cf_NN` -> Op::CfFlag(NN); `cf_ind_NN` -> Op::CfFlagInd(NN)
- `fs_NN` / `fs_ind_NN` / `fc_NN` / `fc_ind_NN` / `fs_c_NN` / `fs_c_ind_NN` / `fc_c_NN` / `fc_c_ind_NN` -> Op::FlagTest { kind: …, flag } / Op::FlagTestInd { kind, reg } — pattern-match on the prefix kind via a helper `parse_flag_test_prefix(rest) -> Option<(FlagTestKind, bool, u8)>` that returns (kind, is_ind, n). NOTE: `fs_c_NN` corresponds to `FsQueryClear` (FS?C), `fc_c_NN` to `FcQueryClear` (FC?C); audit confirms all 4 FlagTestKind variants are wired.
- `view_NN` -> Op::View(NN); `view_ind_NN` -> Op::ViewInd(NN)
- `arcl_NN` -> Op::Arcl(NN); `arcl_ind_NN` -> Op::ArclInd(NN)
- `asto_NN` -> Op::Asto(NN); `asto_ind_NN` -> Op::AstoInd(NN)
- `tone_N` -> Op::Tone(N) where N parses as u8 in 0..=9
- **`del_NNN` -> Op::Del(NNN as u8) WITH EXPLICIT CLAMP CHECK** (BLOCKER B3 resolution): parse the 3-digit accumulator as u16; if value > 255, return `Err(GuiError { message: "DEL value must be 0-255 (hp41-core Op::Del field is u8 — Phase 26 divergence from HP-41 hardware 0-999, deferred to v3.x)" })`. Otherwise return `Ok(Op::Del(value as u8))`. The clamp lives at the resolver level so a programmatic `dispatch_op` call with `del_999` produces a documented GuiError — frontend renders this as a toast. Frontend modal preview ALSO clamps (Task 3 step k): when the accumulator reads "256"+, renderModalLcd emits `"DEL ERR"` instead of the numeric preview, signaling the value is out of range before dispatch. This preserves the hardware-faithful divergence as a user-visible behavior, not a silent truncation.
- `catalog_N` -> Op::Catalog(N) with N in 1..=3 (HP-41CV catalogs 1=programs, 2=ROM-modules, 3=registers/flags; Catalog 4 is HP-41CX-only — Phase 26 scope is HP-41CV per ROADMAP)
- `clp_<LABEL>` -> Op::Clp(label.to_string()) — strip_prefix("clp_") and pass the remainder (including any underscores in the label) as the String arg per PATTERNS.md note about labels being raw string suffixes
- `sto_arith_<op>_ind_NN` -> Op::StoArithInd { kind, reg } — extend the existing `resolve_sto_arith` helper to recognize the `_ind_` infix (use `rsplit_once('_')` then check whether the inner segment is `_ind_<NN>` vs `_<NN>`)
- ASN parameterized: `asn_NN_NAME` -> Op::Asn { name: NAME.to_string(), key_code: NN } — strip_prefix("asn_") then split_at the first underscore-after-numeric to recover key_code and name; helper `parse_asn(rest, original) -> Result<Op, GuiError>` mirroring `resolve_sto_arith` shape

Conditional-test prefixes for the 4 keyboard-bound and 8 XEQ-by-name tests are already wired via `Op::Test(TestKind::XEqY)` etc. — confirm by grep that `xeq_y`/`x_eq_y`/etc. resolve already; extend ONLY if any are missing.

Shrink the stub-error arm: remove `pi`, `polar_to_rect`, `rect_to_polar`, `beep` (they now resolve to real Ops above). KEEP `xeq_prompt`, `gto_prompt`, `lbl_prompt`, `asn`, `view`, `catalog`, `sto_prompt`, `rcl_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `isg_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, `x_le_y_prompt`, `x_gt_y_prompt`, `x_eq_0_prompt` as defense-in-depth per D-26.5. Also keep v3.x module-Pac names if any are currently in the arm (audit by reading the current arm).

**WARNING W1 resolution (flag-prompt stub-arm audit):** Audit which `*_prompt` ids exist in Keyboard.tsx KEY_DEFS for the full FlagTestKind family. Current KEY_DEFS bindings (verified by reading hp41-gui/src/Keyboard.tsx lines 75-90):
- Row 5 col 1: shifted `'sf_prompt'` (SF) ✓ in test list
- Row 5 col 2: shifted `'cf_prompt'` (CF) ✓ in test list
- Row 5 col 3: shifted `'fs_prompt'` (FS?) ✓ in test list
- NO KEY_DEFS bindings exist for `fc_prompt`, `fs_c_prompt`, or `fc_c_prompt` — the four FlagTest variants FC?, FS?C, FC?C are reachable only via XEQ-by-Name modal (D-25.9 deferral pattern carried into v2.2 per Phase 25 D-25.7: only 4 conditional tests on the physical keyboard; the other 8 route through XEQ).
- Therefore `test_modal_prompt_ids_are_stubs_for_now` ids list stays UNCHANGED (no new entries to add). Document this finding in a comment block above the test:
  ```rust
  // Audit (Phase 26 W1 revision): the full FlagTestKind family is {SF, CF, FsQuery,
  // FcQuery, FsQueryClear, FcQueryClear}. Only SF, CF, and FsQuery have keyboard
  // bindings (Row 5 cols 1-3 shifted ids). FcQuery / FsQueryClear / FcQueryClear
  // are XEQ-by-Name reachable only (Phase 25 D-25.7 / D-25.9 pattern). No new
  // *_prompt ids to add to this test.
  ```

Tests in `#[cfg(test)] mod tests` (extend, do not replace existing tests):
- `test_new_v22_named_op_resolvers`: assert_eq!(resolve("pi").unwrap(), Op::Pi); …one assertion per new bare arm above (~30 assertions)
- `test_new_v22_parameterized_prefixes`: assert_eq!(resolve("sto_ind_05").unwrap(), Op::StoInd(5)); …(at minimum two assertions per new prefix family — direct + IND, total ~40 assertions). Include sto_arith_plus_ind_07, clp_MYPRG (label), tone_5, del_010, catalog_2, asn_22_TEST.
- **`test_del_clamps_at_u8_max`**: `assert_eq!(resolve("del_255").unwrap(), Op::Del(255));` AND `assert!(resolve("del_256").is_err());` AND assert the err.message contains "0-255". This locks the BLOCKER B3 contract.
- Update the existing `test_stub_error_for_v22_backlog_ops` to drop pi/polar_to_rect/rect_to_polar/beep from the asserted stub-id list. The `test_modal_prompt_ids_are_stubs_for_now` (lines 322-352) STAYS unchanged — its id list is the defense-in-depth contract.
- `test_more_specific_prefix_wins`: assert that "sto_ind_05" resolves to StoInd(5) and NOT to a hypothetical Sto interpretation — guards the Pitfall 3 ordering invariant.

Maintain `#![deny(clippy::unwrap_used)]` discipline: production code uses `?` or `.expect()`; the test module's existing `#[allow(clippy::unwrap_used)]` covers test assertions.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator && just gui-check && cd hp41-gui/src-tauri && cargo test --no-fail-fast key_map::tests 2>&1 | tail -40</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p hp41-gui-tauri key_map::tests` passes; all new test_new_v22_named_op_resolvers and test_new_v22_parameterized_prefixes assertions green
    - `grep -c '"pi" =>' hp41-gui/src-tauri/src/key_map.rs` returns at least 1 (real arm exists; no longer in stub)
    - `grep -c 'strip_prefix("sto_ind_")' hp41-gui/src-tauri/src/key_map.rs` returns at least 1
    - `grep -B1 'strip_prefix("sto_")' hp41-gui/src-tauri/src/key_map.rs | grep -c 'strip_prefix("sto_ind_")'` returns at least 1 (more-specific-first ordering)
    - The existing `test_modal_prompt_ids_are_stubs_for_now` test passes unchanged (W1: no new ids added — the family audit confirmed no missing keyboard-bound flag prompts)
    - `cargo test -p hp41-gui-tauri key_map::tests::test_del_clamps_at_u8_max` passes (BLOCKER B3 lock)
    - `cargo clippy -p hp41-gui-tauri --all-targets -- -D warnings` returns no warnings (SC-4 invariant: `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns zero matches)
    - The `test_stub_error_for_v22_backlog_ops` arm no longer includes `"pi"`, `"polar_to_rect"`, `"rect_to_polar"`, or `"beep"`
  </acceptance_criteria>
  <done>
    All v2.2 HP-41CV ROM ops resolve via key_map; all new parameterized prefixes parse correctly with more-specific-first ordering; stub-arm shrunk to defense-in-depth-only ids; DEL u8 clamp contract locked by test; flag-prompt family audit documented; all key_map tests green; clippy clean; SC-4 invariant intact.
  </done>
</task>

<task type="execute">
  <name>Task 2: Extend CalcStateView with user_keymap/flags/display_override/event_buffer projections + Tauri commands drain event_buffer + TS interface mirror update + update 6 existing test call sites</name>
  <files>hp41-gui/src-tauri/src/types.rs, hp41-gui/src-tauri/src/commands.rs, hp41-gui/src/App.tsx</files>
  <read_first>
    - hp41-gui/src-tauri/src/types.rs (full 206 lines — current CalcStateView struct + from_state + budget test; line 45 from_state signature; existing test call sites at lines 124, 139, 147, 169, 186, 198 — these MUST be updated when the signature changes per BLOCKER B4)
    - hp41-gui/src-tauri/src/commands.rs (handle_op_finalize / handle_get_state / handle_sst_step / handle_bst_step / handle_run_stop — the 5 from_state(calc, print_lines) call sites at lines 214, 226, 264, 271, 279)
    - hp41-gui/src/App.tsx (lines 14-26: existing TS `interface CalcStateView` definition — BLOCKER B5 requires extending this mirror)
    - hp41-core/src/state.rs (confirm field names on CalcState: assignments / flags / display_override / event_buffer — these were added in Phases 21/22; verify the exact types and #[serde(default)] annotations)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src-tauri/src/types.rs (extend)" lines 119-181 (struct field pattern, drain-before-call pitfall, budget-test pattern raised to 500)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"Drain-print-buffer-before-from_state pattern" lines 871-880
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.11 lines 76-77 (CalcStateView extension specification)
  </read_first>
  <action>
In `types.rs`:

(a) Add four new fields to `CalcStateView` struct, each carrying a `// Phase 26 D-26.11:` provenance comment matching the Phase-15/Phase-18 precedent:
- `pub user_keymap: Vec<(u8, String)>` — collected from `state.assignments` (BTreeMap<u8, String>) into a Vec; sort by key for deterministic JSON output
- `pub flags: Vec<u8>` — list of currently-set user flag indices (0..=55); derive from `state.flags` (the underlying u64 or bitfield from Phase 21). Recommended: emit `Vec<u8>` of set indices because typical state has 0-3 flags set, keeping JSON small (`"flags":[]` is 11 bytes; `"flags":[5,12]` is 18 bytes). If the field already exists on CalcState as `flags: u64`, write a small helper `set_flag_indices(flags: u64) -> Vec<u8>` that iterates 0..=55 and collects set bits.
- `pub display_override: Option<String>` — direct projection from `state.display_override` (Phase 21); use `.clone()` if already Option<String>
- `pub event_buffer: Vec<String>` — passed in as a parameter to `from_state` (drained in commands.rs before from_state is called, mirroring the print_lines drain pattern at line 44)

(b) Update `pub fn from_state(state: &CalcState, print_lines: Vec<String>)` signature to `pub fn from_state(state: &CalcState, print_lines: Vec<String>, event_lines: Vec<String>)` — add the event_lines parameter as the third argument. Construct the four new fields inside the function body. Document the drain-before-call invariant with a new comment block updated for both buffers.

(c) **BLOCKER B4 resolution — update the 6 existing test call sites in `types.rs` tests** (lines 124, 139, 147, 169, 186, 198): every existing `CalcStateView::from_state(&state, vec![])` MUST become `CalcStateView::from_state(&state, vec![], vec![])` (third arg empty Vec for event_lines). Specific line numbers to update:
- Line 124 (`test_dispatch_op_payload_size`)
- Line 139 (`test_calc_state_view_structure`)
- Line 147 (`test_annunciators_from_state`)
- Line 169 (`test_phase15_stack_fields_exist`)
- Line 186 (`test_in_eex_mode_false_without_e`)
- Line 198 (`test_phase18_fields_exist`)

(d) Update the budget assertion test `test_dispatch_op_payload_size`: raise the threshold from 400 to 500 bytes per FN-GUI-05 / D-26.11. Update the docstring to "CalcStateView JSON (empty program + empty assignments + no flags) must be ≤500 bytes". Add a SECOND test `test_dispatch_op_payload_size_with_realistic_load` that constructs a CalcState with ~5 assignments (e.g. (11, "SIN"), (12, "COS"), (21, "TEST"), (22, "MYPRG"), (33, "SUB")) and 3 set flags (e.g. flags 5, 10, 22) and asserts the JSON length is still ≤500 bytes.

In `commands.rs`:

(e) For each of `handle_op_finalize`, `handle_get_state`, `handle_sst_step`, `handle_bst_step`, `handle_run_stop` (the 5 CalcStateView-returning helpers at lines 214, 226, 264, 271, 279), add ONE line BEFORE the `CalcStateView::from_state(...)` call: `let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();`. Pass `event_lines` as the new third argument to `from_state`. The print_lines drain pattern stays unchanged.

Verify `#![deny(clippy::unwrap_used)]` discipline: production code uses Mutex `.unwrap_or_else(|e| e.into_inner())`; no new `.unwrap()` introduced. The lock-poison-recovery pattern (PATTERNS.md §"Lock-poison-recovery pattern") continues unchanged.

In `hp41-gui/src/App.tsx`:

(f) **BLOCKER B5 resolution — extend the TS CalcStateView interface mirror** (App.tsx lines 14-26) with the four new fields exactly matching the Rust projection shapes:
```typescript
interface CalcStateView {
  display_str: string;
  x_str: string;
  y_str: string;
  z_str: string;
  t_str: string;
  lastx_str: string;
  in_eex_mode: boolean;
  annunciators: Annunciators;
  print_lines: string[];
  program_steps: string[];
  pc: number;
  // Phase 26 D-26.11 (BLOCKER B5):
  user_keymap: Array<[number, string]>;   // mirrors Vec<(u8, String)>
  flags: number[];                          // mirrors Vec<u8> of set-flag indices
  display_override: string | null;          // mirrors Option<String>
  event_buffer: string[];                   // mirrors Vec<String> (drained per IPC)
}
```
This is a pure type-shape edit; no runtime behavior change in App.tsx in this task. Plan 26-03 will later READ `calcState.user_keymap` to pass to <Keyboard userKeymap={...}/>; Plan 26-01 Task 3 will not yet consume the four new fields (modal infrastructure does not need them).
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator && cd hp41-gui/src-tauri && cargo test --no-fail-fast --lib types::tests commands::tests 2>&1 | tail -30 && cd .. && pnpm --filter hp41-gui-frontend exec tsc --noEmit 2>&1 | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p hp41-gui-tauri test_dispatch_op_payload_size` passes with the threshold ≤500 bytes
    - `cargo test -p hp41-gui-tauri test_dispatch_op_payload_size_with_realistic_load` passes (new test asserts realistic ~5-assignment + 3-flag CalcStateView serializes to ≤500 bytes)
    - `cargo test -p hp41-gui-tauri types::tests` compiles AND passes — all 6 updated call sites (B4 resolution: lines 124, 139, 147, 169, 186, 198) carry the new three-arg signature
    - `grep -c "pub user_keymap" hp41-gui/src-tauri/src/types.rs` returns at least 1
    - `grep -c "pub flags" hp41-gui/src-tauri/src/types.rs` returns at least 1
    - `grep -c "pub event_buffer" hp41-gui/src-tauri/src/types.rs` returns at least 1
    - `grep -c "event_buffer.drain" hp41-gui/src-tauri/src/commands.rs` returns at least 5 (one per CalcStateView-returning helper)
    - `grep -cE "from_state\([^,]*,[^,]*,[^)]*event_lines" hp41-gui/src-tauri/src/commands.rs` returns exactly 5 (BLOCKER B4: ALL 5 call sites carry the new event_lines arg with exactly 3 comma-separated args)
    - `grep -c "user_keymap" hp41-gui/src/App.tsx` returns at least 1 (BLOCKER B5: TS interface mirror has the field)
    - `grep -c "display_override" hp41-gui/src/App.tsx` returns at least 1
    - `grep -c "event_buffer" hp41-gui/src/App.tsx` returns at least 1
    - `pnpm --filter hp41-gui-frontend exec tsc --noEmit` returns no type errors (BLOCKER B5: TS interface mirror added; Plan 26-03 can safely read calcState.user_keymap)
    - `cargo clippy -p hp41-gui-tauri --all-targets -- -D warnings` returns no warnings
    - `just gui-check` passes
  </acceptance_criteria>
  <done>
    CalcStateView carries the four new D-26.11 projections; from_state signature extended to 3 args; all 6 existing types.rs test call sites updated to the new signature; budget assertion passes at ≤500 bytes including realistic load; event_buffer drained per IPC mirroring print_buffer pattern (exactly 5 commands.rs call sites carry the new arg); App.tsx TS CalcStateView interface mirror extended with all four new fields so tsc --noEmit passes; all existing tests still pass; clippy clean.
  </done>
</task>

<task type="execute">
  <name>Task 3: App.tsx PendingInput discriminated union (14 variants) + handleModalKey struct-return + renderModalLcd + handleClick MODAL_OPENERS intercept + SHIFT-during-modal flow + Esc/keyboard wiring + Vitest tests</name>
  <files>hp41-gui/src/App.tsx, hp41-gui/src/Keyboard.tsx, hp41-gui/src/App.test.tsx, hp41-gui/src/key_defs_ids.ts (NEW)</files>
  <read_first>
    - hp41-gui/src/App.tsx (full 347 lines — shiftActive state machine, handleClick resolution priority around line 165, invokeForKey at line 48, extractErrMessage at line 34, handleKey physical-keyboard listener, displayText derivation, MAP table; **note the existing SHIFT-during-modal click handler branch (handleClick early-return for `key.variant === 'shift'`) — W2 verification depends on this**)
    - hp41-gui/src/Keyboard.tsx (full 289 lines — KEY_DEFS table at lines 50-94; identify which v2.2 keys are missing or have wrong shifted-id assignments)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"hp41-gui/src/App.tsx (extend)" lines 242-405 (frontend-owned state pattern, resolution-priority click handler, MODAL_OPENERS intercept block, invokeForKey, extractErrMessage, displayText swap, Tab/Esc one-shot consumption)
    - .planning/phases/26-gui-integration-and-polish/26-PATTERNS.md §"Hardware-faithful one-shot bit reuse for IND-toggle" lines 825-861 (TypeScript port of CLI check_ind_toggle)
    - .planning/phases/26-gui-integration-and-polish/26-CONTEXT.md D-26.1 + D-26.2 + D-26.3 + D-26.4 + D-26.5 (modal architecture decisions)
    - hp41-cli/src/app.rs (CLI PendingInput hybrid struct-variants — REFERENCE for the TS port; the FlagPrompt/RegisterPrompt/ClpLabel/DelCount/TonePrompt/XeqByName logical states must mirror)
    - hp41-cli/src/ui.rs::pending_prompt (CLI LCD preview emitter — REFERENCE for renderModalLcd output strings)
  </read_first>
  <action>
In `App.tsx`:

(a) Add the discriminated union type definition at the top of the file (after imports, before the App function). **REVISION B1 + B2 — 14 variants total** (12 from D-26.4 + `single_digit` merging Tone/Catalog + `direct` for immediate-dispatch openers):
```typescript
type FlagTestKind = 'SF' | 'CF' | 'FsQuery' | 'FcQuery' | 'FsQueryClear' | 'FcQueryClear';
type RegisterOpKind = 'Sto' | 'Rcl' | 'StoAdd' | 'StoSub' | 'StoMul' | 'StoDiv'
                    | 'View' | 'Arcl' | 'Asto' | 'Isg' | 'Dse';
type FmtMode = 'fix' | 'sci' | 'eng';
type SingleDigitOp = 'Tone' | 'Catalog';

type PendingInput =
  | { kind: 'flag'; testKind: FlagTestKind; ind: boolean; acc: string }
  | { kind: 'register'; op: RegisterOpKind; ind: boolean; acc: string }
  | { kind: 'clp'; acc: string }
  | { kind: 'del'; acc: string }
  | { kind: 'xeq_name'; acc: string }
  | { kind: 'fmt'; mode: FmtMode }
  | { kind: 'assign_key' }
  | { kind: 'assign_label'; keyCode: number; acc: string }
  | { kind: 'confirm_load'; programIdx: number }
  | { kind: 'hex'; acc: string }
  | { kind: 'print' }
  | { kind: 'single_digit'; op: SingleDigitOp; max: number }   // B2: Tone (max=9) + Catalog (max=3)
  | { kind: 'direct'; dispatchId: string };                     // B1: immediate-dispatch openers
```

The `single_digit` variant accepts ONE keystroke 0..=max and immediately dispatches `tone_${digit}` or `catalog_${digit}`. The `direct` variant dispatches `dispatchId` on the very first call to handleModalKey (effectively a "no modal, just pass through"). Both replace ad-hoc workarounds discussed in the prior draft.

(b) Add `const [pendingInput, setPendingInput] = useState<PendingInput | null>(null);` in the same hook block as the existing `shiftActive` state (App.tsx around line 111-124, after `toast` state).

(c) Define a `MODAL_OPENERS: Record<string, () => PendingInput>` constant mapping the 13 prompt-ids per D-26.5 + auxiliary opener ids (`asn`, `view`, `catalog`, `xeq_prompt`, `gto_prompt`, `lbl_prompt`) to factory functions returning the initial PendingInput state. **REVISION B1 resolution: the 4 conditional-test prompts route through the `direct` variant** — they dispatch immediately, no accumulation, no IND. Concrete factory map (acceptance criterion lists all 13 of D-26.5):

```typescript
const MODAL_OPENERS: Record<string, () => PendingInput> = {
  // Register-modal openers (D-26.5): accumulator + IND-toggle
  'sto_prompt': () => ({ kind: 'register', op: 'Sto', ind: false, acc: '' }),
  'rcl_prompt': () => ({ kind: 'register', op: 'Rcl', ind: false, acc: '' }),
  'isg_prompt': () => ({ kind: 'register', op: 'Isg', ind: false, acc: '' }),
  // Flag-modal openers (D-26.5): accumulator + IND-toggle
  'sf_prompt': () => ({ kind: 'flag', testKind: 'SF', ind: false, acc: '' }),
  'cf_prompt': () => ({ kind: 'flag', testKind: 'CF', ind: false, acc: '' }),
  'fs_prompt': () => ({ kind: 'flag', testKind: 'FsQuery', ind: false, acc: '' }),
  // Fmt-modal openers (D-26.5): single-digit FIX/SCI/ENG N
  'fix_prompt': () => ({ kind: 'fmt', mode: 'fix' }),
  'sci_prompt': () => ({ kind: 'fmt', mode: 'sci' }),
  'eng_prompt': () => ({ kind: 'fmt', mode: 'eng' }),
  // BLOCKER B1: conditional-test prompts dispatch IMMEDIATELY via `direct` variant
  'x_eq_y_prompt': () => ({ kind: 'direct', dispatchId: 'x_eq_y' }),
  'x_le_y_prompt': () => ({ kind: 'direct', dispatchId: 'x_le_y' }),
  'x_gt_y_prompt': () => ({ kind: 'direct', dispatchId: 'x_gt_y' }),
  'x_eq_0_prompt': () => ({ kind: 'direct', dispatchId: 'x_eq_0' }),
  // Auxiliary openers (not in D-26.5's 13, but mentioned in CONTEXT D-26.5 "plus xeq_prompt, gto_prompt, lbl_prompt, asn, view, catalog"):
  'xeq_prompt': () => ({ kind: 'xeq_name', acc: '' }),
  // BLOCKER B2: catalog and tone merge into single_digit
  'catalog': () => ({ kind: 'single_digit', op: 'Catalog', max: 3 }),
  'tone': () => ({ kind: 'single_digit', op: 'Tone', max: 9 }),
  // ASN flow: AssignKey → (next key click) → AssignLabel → text → Enter
  'asn': () => ({ kind: 'assign_key' }),
  // VIEW: takes a register, same shape as register-modal but op='View'
  'view': () => ({ kind: 'register', op: 'View', ind: false, acc: '' }),
  // gto_prompt and lbl_prompt: label-bearing modals (text input + Enter); xeq_name shape works for all three
  'gto_prompt': () => ({ kind: 'xeq_name', acc: '' }),   // re-uses xeq_name shape; dispatch id rewrites to gto_<LABEL>
  'lbl_prompt': () => ({ kind: 'xeq_name', acc: '' }),   // re-uses xeq_name shape; dispatch id rewrites to lbl_<LABEL>
};
```

Note: the 3 label-bearing prompts (`xeq_prompt`, `gto_prompt`, `lbl_prompt`) share the `xeq_name` PendingInput shape but dispatch different parameterized ids. handleModalKey distinguishes by storing the dispatch prefix in a closure variable when the opener fires — implementation detail: extend `xeq_name` to carry an internal `dispatchPrefix?: 'xeq' | 'gto' | 'lbl'` if needed, OR keep MODAL_OPENERS returning xeq_name and route dispatch via a separate state. Planner choice (W5-style discretion): add `dispatchPrefix` to the `xeq_name` variant for cleanliness.

Revised xeq_name shape:
```typescript
| { kind: 'xeq_name'; acc: string; dispatchPrefix?: 'xeq' | 'gto' | 'lbl' }
```

(d) **BLOCKER B6 resolution — implement `handleModalKey` with struct-return**:
```typescript
type ModalKeyResult = {
  nextPending: PendingInput | null;   // null = close modal
  dispatchId: string | null;          // non-null = dispatch this id via invokeForKey
  consumesShift: boolean;             // true = caller must setShiftActive(false)
};

function handleModalKey(
  key: string,
  pending: PendingInput,
  shiftActive: boolean
): ModalKeyResult { ... }
```

Logic per CONTEXT D-26.2 + D-26.4 + revisions:

- Esc key -> return `{ nextPending: null, dispatchId: null, consumesShift: false }`
- `pending.kind === 'direct'` -> return `{ nextPending: null, dispatchId: pending.dispatchId, consumesShift: false }` IMMEDIATELY (B1: do not consume the keystroke; the caller already detected the modal-opener click; this branch fires as the modal opens and is the only path through `direct`)
- `pending.kind === 'flag' | 'register'`:
  - if `shiftActive && key === '0'` -> return `{ nextPending: { ...pending, ind: !pending.ind }, dispatchId: null, consumesShift: true }` (W2: IND-toggle path)
  - if key is a digit 0-9 -> append to acc; if acc.length === 2 -> compute dispatchId tuple-style: `pending.kind === 'register' ? (pending.ind ? `${regPrefix(pending.op)}_ind_${acc}` : `${regPrefix(pending.op)}_${acc}`) : (pending.ind ? `${flagPrefix(pending.testKind)}_ind_${acc}` : `${flagPrefix(pending.testKind)}_${acc}`)`. Return `{ nextPending: null, dispatchId, consumesShift: false }`. Helpers: `regPrefix('Sto')==='sto'`, `regPrefix('Rcl')==='rcl'`, etc. `flagPrefix('SF')==='sf'`, `flagPrefix('FsQuery')==='fs'`, `flagPrefix('FsQueryClear')==='fs_c'`, etc.
  - otherwise (not digit, not shift-0): return `{ nextPending: pending, dispatchId: null, consumesShift: false }` (ignore unmapped key)
- `pending.kind === 'clp'`: accept letters/digits, append to acc; Enter -> return `{ nextPending: null, dispatchId: `clp_${pending.acc}`, consumesShift: false }`
- `pending.kind === 'del'`: 3-digit numeric, dispatch on acc.length === 3 with `del_${acc}`. **B3 frontend clamp**: if accumulator parses to a u16 > 255, the LCD preview (renderModalLcd) shows `"DEL ERR"`; dispatch still fires with `del_${acc}` and key_map.rs returns the documented GuiError → toast.
- `pending.kind === 'single_digit'` (B2): single 0-9 keystroke -> if `digit > pending.max` ignore the keystroke (return pending unchanged); else return `{ nextPending: null, dispatchId: pending.op === 'Tone' ? `tone_${digit}` : `catalog_${digit}`, consumesShift: false }`
- `pending.kind === 'xeq_name'`: alphanumeric input + Enter dispatch. On Enter return `{ nextPending: null, dispatchId: `${pending.dispatchPrefix ?? 'xeq'}_${pending.acc}`, consumesShift: false }`
- `pending.kind === 'fmt'`: single 0-9 digit -> return `{ nextPending: null, dispatchId: `${pending.mode}_${digit}`, consumesShift: false }` (i.e. `fix_3`, `sci_5`, `eng_2`)
- `pending.kind === 'assign_key'`: next keystroke must resolve to a keyCode (via Keyboard click or KEY_DEFS lookup at the caller) -> return `{ nextPending: { kind: 'assign_label', keyCode, acc: '' }, dispatchId: null, consumesShift: false }`. Note: this transition is best handled at the caller (handleClick) which has the KeyDef in scope; pass `key` as a string key id and look up keyCode via KEY_DEFS in the caller.
- `pending.kind === 'assign_label'`: text input + Enter -> return `{ nextPending: null, dispatchId: `asn_${pending.keyCode}_${pending.acc}`, consumesShift: false }`
- `pending.kind === 'confirm_load' | 'hex' | 'print'`: handle per existing v1.x-CLI semantics (planner ports from hp41-cli/src/app.rs)

(e) Implement `function renderModalLcd(pending: PendingInput): string` — emits the LCD preview string per D-26.3. Examples:
- `kind: 'register', op: 'Sto', ind: false, acc: ''` -> `"STO __"` (12 chars total, padded to display width)
- `kind: 'register', op: 'Sto', ind: true, acc: '5'` -> `"STO IND _5"`
- `kind: 'flag', testKind: 'SF', ind: false, acc: '12'` -> `"SF 12"` (final form before dispatch)
- `kind: 'clp', acc: 'MYPRG'` -> `"CLP MYPRG_"` (trailing underscore as digit-entry cursor)
- `kind: 'single_digit', op: 'Tone', max: 9` -> `"TONE _"` (waiting for single digit)
- `kind: 'single_digit', op: 'Catalog', max: 3` -> `"CAT _"`
- `kind: 'fmt', mode: 'fix'` -> `"FIX _"`
- `kind: 'assign_key'` -> `"ASN __"`
- `kind: 'assign_label', keyCode: 22, acc: 'TEST'` -> `"ASN TEST_"`
- `kind: 'direct', dispatchId: 'x_eq_y'` -> never visible (modal closes on the same tick it opened); return empty string (defensive)
- **B3 DEL clamp preview**: `kind: 'del', acc: '256'` -> `"DEL ERR"` (NOT the numeric preview); for acc.length < 3, render `"DEL ${acc.padEnd(3, '_')}"` normally.

Output is right-aligned-ish per the existing `.display` CSS `text-align: right` — actual padding can use `.padEnd(12, ' ')` since the SVG component will center-align glyphs.

(f) Extend `handleClick` per PATTERNS.md §"Resolution-priority click handler pattern" — INSERT the MODAL_OPENERS intercept BEFORE the `busyRef.current = true` block, AFTER `effectiveId` is computed. If `MODAL_OPENERS[effectiveId]`:
- Compute the initial pending = `MODAL_OPENERS[effectiveId]()`.
- If `pending.kind === 'direct'` (B1 fast path): immediately call handleModalKey to retrieve the dispatchId, then `await invokeForKey(dispatchId)`; consume shiftActive if appropriate; return. This handles the 4 conditional-test prompts in one click.
- Else: `setPendingInput(pending); setShiftActive(false); return;` (open the modal, skip dispatch).

Also extend `handleClick` to route SUBSEQUENT clicks (when `pendingInput !== null`) through `handleModalKey(effectiveId, pendingInput, shiftActive)` — **EXCEPT** when `key.variant === 'shift'` (existing branch at App.tsx ~line 165 retains its early-return; SHIFT toggle inside a modal goes through the SHIFT branch, NOT handleModalKey, per W2). Convert the ModalKeyResult struct:
- `nextPending !== null && dispatchId === null` -> `setPendingInput(nextPending)`; if `consumesShift` then `setShiftActive(false)`; no IPC
- `dispatchId !== null` -> `setPendingInput(null)`; if `consumesShift` then `setShiftActive(false)`; `await invokeForKey(dispatchId)`; update calcState; toast on error
- `nextPending === null && dispatchId === null` -> `setPendingInput(null)` (Esc cancel)

(g) Extend `handleKey` (physical-keyboard listener, App.tsx around lines 210-227) per PATTERNS.md:
- Esc clears BOTH shiftActive AND pendingInput
- If `pendingInput !== null`, route the key event to `handleModalKey(keyStr, pendingInput, shiftActive)` (using the mapped key id from MAP, OR the raw key for digit/Enter/letter inputs in CLP/AssignLabel/XeqName modals)
- Otherwise existing behavior unchanged

(h) Update the `MAP` table swap per D-26.10 — actually D-26.10 is in Plan 26-03's scope, so leave MAP unchanged here. Plan 26-03 will update `'p'` -> `'prgm_mode'`.

(i) Update the display rendering per D-26.3 + D-26.7 — actually `<Display14Seg />` lands in Plan 26-02. Here, just add the `displayText` derivation:
```
const displayText = pendingInput
    ? renderModalLcd(pendingInput)
    : (calcState?.display_str ?? '');
```
Replace `{calcState.display_str}` inside `<div className="display">` with `{displayText}`. Plan 26-02 will swap the inner content to `<Display14Seg text={displayText} />`. This way both plans converge cleanly without merge conflicts.

In `Keyboard.tsx`:

(j) **WARNING W3 resolution — replace hand-curated whitelist with explicit ID export**: Create a new file `hp41-gui/src/key_defs_ids.ts` that re-exports the explicit list of all KEY_DEFS ids as a const-tuple. Audit by inspection of Keyboard.tsx lines 41-93 (TOP_ROW + MAIN_GRID — 4 + 35 = 39 entries plus `shifted.id` siblings). The new file:

```typescript
// hp41-gui/src/key_defs_ids.ts
// EXHAUSTIVE list of all KEY_DEFS primary + shifted ids — must match
// hp41-gui/src/Keyboard.tsx KEY_DEFS. When adding a key to KEY_DEFS, update
// this list. The Rust-side test_keyboard_skin_ids_complete in key_map::tests
// reads a hand-listed mirror of this array and asserts every entry either
// resolves via key_map::resolve, opens a modal via MODAL_OPENERS, or is in
// the documented bare-handler set ('sst', 'bst', 'r_s', 'shift', '', 'clx_or_a').

export const KEY_DEFS_PRIMARY_IDS = [
  // ... explicit literal list from Keyboard.tsx KEY_DEFS — 39 primary ids
] as const;

export const KEY_DEFS_SHIFTED_IDS = [
  // ... explicit literal list of shifted ids — count from Keyboard.tsx
] as const;

export const KEY_DEFS_HANDLED_OUTSIDE_RESOLVE = [
  'sst', 'bst', 'r_s', 'shift', '', 'clx_or_a',
] as const;
```

Add a Vitest test `test_all_keyboard_skin_ids_are_valid` that iterates `[...KEY_DEFS_PRIMARY_IDS, ...KEY_DEFS_SHIFTED_IDS]` and asserts each id is either: (i) a key in MODAL_OPENERS, or (ii) in `KEY_DEFS_HANDLED_OUTSIDE_RESOLVE`, or (iii) **must be a known resolvable id**. Since Vitest cannot invoke Tauri commands, use a fixture: the test imports `KEY_DEFS_PRIMARY_IDS` + `KEY_DEFS_SHIFTED_IDS` and asserts every entry that's NOT in (i) or (ii) IS PRESENT in a separately-maintained `KNOWN_RESOLVABLE_IDS` set, which the planner generates from `key_map.rs` by inspection.

To prevent drift, ALSO add a Rust-side test `test_keyboard_skin_ids_resolve_or_are_modal_openers` in `hp41-gui/src-tauri/src/key_map.rs::tests` that hand-mirrors the same id arrays as Rust constants with a sentinel comment ("must match hp41-gui/src/key_defs_ids.ts — when adding a key to KEY_DEFS, update BOTH this Rust array AND key_defs_ids.ts"). The Rust test iterates the array and asserts each id either resolves successfully OR is in the modal-opener stub set OR is in the bare-handler exception set. This catches drift at `just gui-check` time in either direction.

Do NOT change KeyDef shape here — that lands in Plan 26-03 (the `keyCode` field for USER overlay).

In `hp41-gui/src/App.test.tsx` (NEW or extend existing):

(k) Vitest unit tests for `handleModalKey` (extracted as a pure function for testability — implemented as top-level function per step d):
- `test('register modal accumulates two digits then dispatches sto_05')` — assert `nextPending=null, dispatchId="sto_05"`
- `test('flag modal IND-toggle via shift-0 sets ind=true AND consumesShift=true')` — W2 verification
- `test('flag modal IND-toggle does NOT append 0 to acc')` — W2 verification
- `test('register modal with ind=true dispatches sto_ind_NN')` — assert dispatchId carries `_ind_`
- `test('SHIFT-during-modal flow: open register modal → SHIFT (handled by handleClick early-return, not handleModalKey) → click 0 → ind toggles, consumesShift=true')` — W2 explicit test of the cross-layer flow
- `test('clp modal accumulates label and dispatches on Enter')`
- `test('del modal accumulates 3 digits then dispatches del_010')`
- `test('del modal value 256 renders DEL ERR in LCD preview (B3)')` — verify renderModalLcd("256") === "DEL ERR" (padded to display width)
- `test('single_digit modal Tone dispatches on first 0-9 keystroke without IND (B2)')`
- `test('single_digit modal Catalog rejects digits > 3 (B2)')` — assert nextPending unchanged, dispatchId=null when key='5'
- `test('fmt modal dispatches fix_3 / sci_5 / eng_2')`
- `test('assign_key modal advances to assign_label on next key')`
- `test('Esc returns nextPending=null, dispatchId=null (cancel)')`
- `test('direct modal dispatches dispatchId immediately on first handleModalKey call (B1)')` — assert `{ nextPending: null, dispatchId: 'x_eq_y', consumesShift: false }`
- `test('renderModalLcd emits "STO __" for register Sto direct, empty acc')`
- `test('renderModalLcd emits "STO IND _5" for register Sto IND, acc="5"')`
- `test('renderModalLcd emits "SF 12" for flag SF direct, acc="12"')`
- `test('renderModalLcd emits "CLP MYPRG_" for clp acc="MYPRG"')`
- `test('renderModalLcd emits "TONE _" for single_digit Tone')` — B2 verification
- `test('renderModalLcd emits "CAT _" for single_digit Catalog')` — B2 verification

Configure Vitest if not already present (`pnpm --filter hp41-gui-frontend add -D vitest @testing-library/react jsdom @testing-library/jest-dom`); add `"test": "vitest run"` script in `hp41-gui/package.json` if missing. The PATTERNS.md does not specify a Vitest setup; planner: if Vitest is not yet installed, this task adds it; otherwise extend the existing test suite.
  </action>
  <verify>
    <automated>cd /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui && pnpm --filter hp41-gui-frontend test --run 2>&1 | tail -40 && pnpm --filter hp41-gui-frontend exec tsc --noEmit 2>&1 | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - `pnpm --filter hp41-gui-frontend test --run` passes; all new handleModalKey unit tests green (incl. B1 direct-variant test, B2 single_digit tests, B3 DEL ERR test, W2 SHIFT-during-modal test)
    - `grep -c "type PendingInput" hp41-gui/src/App.tsx` returns at least 1
    - `grep -c "kind: 'single_digit'" hp41-gui/src/App.tsx` returns at least 1 (B2: variant exists)
    - `grep -c "kind: 'direct'" hp41-gui/src/App.tsx` returns at least 1 (B1: variant exists)
    - `grep -c "MODAL_OPENERS" hp41-gui/src/App.tsx` returns at least 2 (definition + usage)
    - `grep -c "renderModalLcd" hp41-gui/src/App.tsx` returns at least 2 (definition + usage in displayText)
    - `grep -c "setPendingInput(null)" hp41-gui/src/App.tsx` returns at least 2 (Esc handler + post-dispatch close)
    - `grep -c "pendingInput" hp41-gui/src/App.tsx` returns at least 4 (state + Esc + handleClick branch + displayText)
    - `grep -c "ModalKeyResult" hp41-gui/src/App.tsx` returns at least 1 (B6: struct return type defined)
    - `grep -c "consumesShift" hp41-gui/src/App.tsx` returns at least 3 (struct field + handler branches)
    - The MODAL_OPENERS table contains all 13 *_prompt ids per D-26.5 (sto_prompt, rcl_prompt, fix_prompt, sci_prompt, eng_prompt, isg_prompt, sf_prompt, cf_prompt, fs_prompt, x_eq_y_prompt, x_le_y_prompt, x_gt_y_prompt, x_eq_0_prompt) AND the 4 conditional-test ones map to `kind: 'direct'` (B1)
    - `pnpm --filter hp41-gui-frontend exec tsc --noEmit` returns no type errors (the discriminated union is type-safe; the new TS interface fields from Task 2 are consumed cleanly)
    - The Vitest test for KEY_DEFS audit (W3) passes against `hp41-gui/src/key_defs_ids.ts` constants; the Rust-side mirror test passes too
    - `cargo test -p hp41-gui-tauri test_keyboard_skin_ids_resolve_or_are_modal_openers` passes (W3)
  </acceptance_criteria>
  <done>
    PendingInput discriminated union (14 variants — 12 base + single_digit + direct) ships in App.tsx; handleModalKey returns the ModalKeyResult struct uniformly (B6); MODAL_OPENERS intercepts the 13 prompt-ids before they reach invokeForKey; B1 conditional-test prompts dispatch immediately via the `direct` variant; B2 Tone+Catalog merged into single_digit; B3 DEL clamp surfaces "DEL ERR" preview at 256+; W1 flag-prompt stub-arm audit confirmed no missing entries; W2 SHIFT-during-modal flow explicitly tested; W3 KEY_DEFS audit driven by `key_defs_ids.ts` constants + Rust-side mirror test (no drift-prone whitelist); Esc cancels both shiftActive and pendingInput; physical-keyboard digits route through handleModalKey when a modal is open; all Vitest tests green; TypeScript clean.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Frontend (React) → Tauri command | All keyboard input crosses here as String key_id; resolved via key_map.rs::resolve |
| Tauri command → hp41-core dispatch | Final Op variant dispatched against shared CalcState (Mutex-guarded) |
| event_buffer → frontend (per IPC drain) | New Phase 26 channel mirroring print_buffer; drained on every command response |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-26-01-01 | Spoofing | MODAL_OPENERS intercept in handleClick | mitigate | The 13 *_prompt ids stay as defense-in-depth in key_map.rs stub-error arm per D-26.5; if frontend regression skips the intercept, backend produces a `GuiError` toast — never silent (D-07 invariant). Verified by `test_modal_prompt_ids_are_stubs_for_now` continuing to pass. |
| T-26-01-02 | Tampering | event_buffer drain leaking earlier-program-execution data | mitigate | Mirror print_buffer drain semantics exactly: drain on every command response in commands.rs (handle_op_finalize / handle_get_state / handle_sst / handle_bst / handle_run_stop). Verified by Task 2 acceptance criteria asserting exactly 5 drain call sites. |
| T-26-01-03 | Information Disclosure | CalcStateView size budget exceeded | mitigate | Two budget tests in types.rs (`test_dispatch_op_payload_size` empty + `test_dispatch_op_payload_size_with_realistic_load` ~5 ASN + 3 flags) both assert ≤500 bytes per FN-GUI-05 / D-26.11. |
| T-26-01-04 | Denial of Service | Large CLP label ("clp_AAAAAA...") overflowing memory | accept | Op::Clp(String) accepts arbitrary label per HP-41 hardware behavior (LBL labels are user-controlled); the CLP modal's text-input has no upper bound on the physical HP-41 either. No PII in scope; emulator is single-user; risk is the user crashing their own calc. Defensive cap could be added in Phase 27. |
| T-26-01-05 | Elevation of Privilege | IND-toggle path bypassing Op::*Ind dispatch decision | mitigate | The handleModalKey end-of-2-digit dispatch is a SINGLE tuple-match decision per D-26.2 (`pending.ind ? '<op>_ind_NN' : '<op>_NN'`). No alternative path constructs the parameterized id. Vitest tests `register modal with ind=true dispatches sto_ind_NN` and `flag modal IND-toggle via shift-0 sets ind=true` directly verify. |
| T-26-01-06 | Spoofing | Physical-keyboard digits during open modal accidentally dispatching unrelated Op | mitigate | handleKey extension routes ALL events to handleModalKey when pendingInput !== null (per Task 3 step g); the existing dispatchKeyId path is short-circuited. Esc precedence: Esc ALWAYS closes the modal first, before any other key handling. |
| T-26-01-07 | Denial of Service | DEL value out of u8 range (BLOCKER B3) | mitigate | Resolver-level clamp returns documented GuiError for `del_256..=del_999`; frontend renders the LCD preview as `"DEL ERR"` at the modal-preview layer so the user sees the divergence before committing dispatch. Locked by `test_del_clamps_at_u8_max` (Task 1) and the `del modal value 256 renders DEL ERR in LCD preview` Vitest test (Task 3). hp41-core u8 width is a documented v2.2 divergence; widening to u16 deferred to v3.x. |
</threat_model>

<verification>
1. `cd hp41-gui/src-tauri && cargo test --all-targets` — all key_map + types + commands tests pass
2. `cd hp41-gui && pnpm --filter hp41-gui-frontend test --run` — all Vitest tests pass
3. `cd hp41-gui && pnpm --filter hp41-gui-frontend exec tsc --noEmit` — no TypeScript errors (B5: extended TS interface mirror compiles)
4. `just gui-ci` — full GUI CI pipeline green (build + test + clippy)
5. SC-4 sanity check: `grep -rEn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns ZERO matches
6. `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` returns only the documented `fn op_display_name(...)` exception in prgm_display.rs
7. `cargo clippy --workspace --all-targets -- -D warnings` is clean across the entire workspace (`hp41-core` + `hp41-cli` + `hp41-gui-tauri`)
</verification>

<success_criteria>
- Every HP-41CV ROM op variant added in Phases 20-24 resolves successfully via `key_map::resolve` or `key_map::resolve_parameterized` with no panic, no `InvalidOp` error
- The stub-error arm in `key_map::resolve` no longer contains `pi`, `polar_to_rect`, `rect_to_polar`, `beep` (they have real `Ok(Op::*)` arms)
- The 13 `*_prompt` ids + `asn`/`view`/`catalog`/`xeq_prompt`/`gto_prompt`/`lbl_prompt` REMAIN in the stub-error arm as defense-in-depth; `test_modal_prompt_ids_are_stubs_for_now` test continues to pass (W1: ids list unchanged after audit)
- Frontend `handleClick` intercepts every MODAL_OPENERS entry BEFORE invoking `dispatch_op`; opens a React modal via `setPendingInput(...)`
- The 4 conditional-test prompts (`x_eq_y_prompt`, etc.) route through the `direct` variant and dispatch immediately (B1)
- Tone and Catalog modals share the `single_digit` variant with `op` + `max` discriminator (B2)
- DEL modal preview shows "DEL ERR" for accumulated values 256+ (B3); key_map.rs returns documented GuiError for `del_256..=del_999`
- The TS `CalcStateView` interface in App.tsx mirrors the four new Rust projections (B5); `tsc --noEmit` passes
- All 6 existing `types.rs` test call sites use the three-arg `from_state` signature (B4)
- Inside an open FlagPrompt or RegisterPrompt modal, SHIFT then 0 toggles the modal's `ind` boolean; LCD preview updates to show "IND"; the '0' is NOT appended to acc; `consumesShift=true` returned from handleModalKey (W2)
- End-of-2-digit accumulation dispatches the parameterized id with correct `_ind_` infix when `ind=true`
- ASN flow opens AssignKey → AssignLabel → text input → Enter dispatches `asn_NN_NAME` parameterized id; key_map resolves to `Op::Asn { name, key_code }`
- Esc inside an open modal closes the modal AND clears shiftActive
- Physical-keyboard digits during an open modal feed the modal (do NOT dispatch as bare digits)
- `CalcStateView` carries `user_keymap`, `flags`, `display_override`, `event_buffer` projections
- `CalcStateView` JSON serializes to ≤ 500 bytes for both empty and realistic-load test fixtures
- `event_buffer` is drained alongside `print_buffer` on every command response (exactly 5 call sites)
- KEY_DEFS audit (W3) is driven by exported `key_defs_ids.ts` constants + Rust-side mirror — no hand-curated whitelist
- All Rust + Vitest + TypeScript checks pass via `just gui-ci`
- SC-4 invariant intact: no calculator/math logic in `hp41-gui/src-tauri/`
</success_criteria>

<output>
After completion, create `.planning/phases/26-gui-integration-and-polish/26-01-SUMMARY.md` documenting:
- The exact list of bare-op resolvers added (count + section grouping)
- The exact list of parameterized-prefixes added (with example resolutions)
- The DEL u8-clamp resolution (BLOCKER B3): how the documented v2.2 divergence is surfaced (LCD "DEL ERR" preview + GuiError toast on dispatch attempt of 256+)
- Whether the budget assertion held at ≤500 bytes for the realistic-load test (record actual byte count for Phase 27 trend tracking)
- Any KEY_DEFS additions or fixes discovered during the audit (Task 3 step j)
- The W3 audit outcome: how the `key_defs_ids.ts` + Rust-side mirror prevents drift
- The W1 flag-prompt audit outcome: which FlagTestKind variants are keyboard-bound vs XEQ-only
- Any planner discretion decisions made (e.g. helper function shapes for parse_flag_test_prefix / parse_asn, regPrefix/flagPrefix shape, xeq_name.dispatchPrefix design)
- Cross-references to Plans 26-02 and 26-03 (which depend on this plan's PendingInput infrastructure and CalcStateView projections including the TS interface mirror)
</output>
</content>
</invoke>