# Phase 26: GUI Integration & Polish — Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 26 wires every v2.2-added `Op` variant (Phases 20–24, ~80 new variants in `hp41-core::ops::Op`) into the **Tauri GUI** by extending `hp41-gui/src-tauri/src/key_map.rs::resolve` (bare + parameterized prefixes) and `KEY_DEFS` in `hp41-gui/src/Keyboard.tsx` (three-label primary/shifted/alphaChar bindings). It routes the 13 previously-stubbed prompt IDs (`sto_prompt`, `rcl_prompt`, `fix_prompt`, `sci_prompt`, `eng_prompt`, `isg_prompt`, `sf_prompt`, `cf_prompt`, `fs_prompt`, `x_eq_y_prompt`, `x_le_y_prompt`, `x_gt_y_prompt`, `x_eq_0_prompt`) to real **React-frontend modals** — closing the parity gap with CLI Phase 25's `PendingInput` hybrid struct-variants — and removes 7 bare ops from the stub-error arm (`pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `view`, `catalog`, `asn`) by dispatching the now-existing core Ops or opening their modals.

GUI Polish ships in the same phase: a 14-segment SVG LCD font replaces the CSS-text display, a `?`-keyboard-shortcut overlay ports from `docs/hp41cv-functions.json` (vite JSON-import per D-25.16), USER mode shows current `Op::Asn` mappings via per-key text-relabel, and the `'p'` physical-keyboard binding remaps from `prx` to `prgm_mode` (resolving the v2.0 deferred conflict).

**Mandated by ROADMAP cross-cutting constraints:**
- **SC-4 invariant non-negotiable** — NEVER add `op_*` / `flush_entry_*` / `format_hpnum` to `hp41-gui/src-tauri/`. `op_display_name` in `prgm_display.rs` remains the ONLY display-formatter exception. Stricter grep `fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)` must return zero matches.
- **D-07 (no silent discards)** preserved — every unhandled ID still surfaces a `GuiError` toast, never silent.
- **CLI ↔ GUI parity invariant D-25.6** — Phase 26 must mirror the Phase 25 prefix-modal model exactly. Same logical states (FlagPrompt/RegisterPrompt/ClpLabel/DelCount/TonePrompt/XeqByName/FmtDigits/AssignKey/AssignLabel/ConfirmLoad/HexModal), same one-shot SHIFT lifetime, same ALPHA-overrides-SHIFT deferral, same IND-toggle via shift-0. Parity = observable behavior, not implementation identity.
- **Stub-error arm shrinks to v3.x module-Pac ops only** — every HP-41CV ROM op resolves successfully; only module-Pac functions (Math 1 / Stat 1 / Time / Advantage) remain as stubs.
- **Modal frontend components are React-only (TypeScript)** — they call `dispatch_op` with the resolved parameterized ID (e.g. `sto_05`, `sf_12`, `sf_ind_12`).
- **`CalcStateView` budget relaxed ≤500 bytes** (was ≤300) for FN-GUI-05 — but no new `pending_input` field needed per D-26.1 (frontend-owned modals).

**Out of scope (explicit):**
- Any `hp41-core` changes — Ops, `CalcState` fields, error variants are all locked from Phases 20–24, and Phase 25's "core frozen" invariant continues.
- Any `hp41-cli` changes — CLI Phase 25 is shipped; Phase 26 is GUI-only.
- 14-seg font for special HP-41 charset (Σ, π-glyph, μ-superscript, …) beyond what `format_alpha` and `format_hpnum` already emit — those land with the v3.x ALPHA-special-charset expansion (Phase 25 D-25.5).
- Full hardware-faithful ALPHA-mode-with-prefix behavior — v3.x (parity with CLI Phase 25 deferral D-25.5).
- Module-Pac emulation (Math 1 / Stat 1 / Time / Advantage) — permanent v2.x exclusion per REQUIREMENTS.md boundary.
- Test-coverage gate ≥95% on hp41-core, numerical-accuracy expansion, flag-semantics proptest, Playwright GUI E2E — Phase 27.
- New `Op` variants — Phase 26 wires already-landed variants; no Op additions.

</domain>

<decisions>
## Implementation Decisions

### Modal state ownership (D-26.1 — D-26.5)

- **D-26.1: Frontend-owned modal state.** React `useState<PendingInput | null>` in `App.tsx` mirrors the CLI's logical `PendingInput` states. Keys during an open modal accumulate in React state; the backend only sees the final parameterized op id (e.g. `sto_05`, `sf_ind_12`, `clp_MYPRG`, `del_010`, `tone_5`). **No new field on `CalcStateView`** — IPC budget unchanged. Mirrors the v2.1 `shiftActive` precedent (Phase 19 D-5) bit-for-bit. **Rationale:** D-25.6 parity = observable behavior, not implementation identity; the user explicitly chose frontend-owned SHIFT in v2.1 and the same logic applies to modals. PRGM-mode duality continues to work because the final parameterized op (`Op::Sto(5)`) auto-routes inside core dispatch based on `state.prgm_mode`.

- **D-26.2: IND-toggle via shift-0 — full CLI parity with D-25.12.** Inside an open `FlagPrompt` / `RegisterPrompt` modal, the user presses SHIFT (toggling `shiftActive`), then presses `0`: the `'0'` keystroke is intercepted and toggles the modal's `ind` field instead of appending to the accumulator. The dispatch decision is a single-point tuple match at end-of-accumulation: `if pending.ind { '<op>_ind_NN' } else { '<op>_NN' }`. Hardware-faithful per HP-41C/CV Quick Reference Guide p.14. Reuses the existing `shiftActive` state machine — no new state field.

- **D-26.3: Modal preview replaces LCD content during accumulation.** When a modal is active, the 14-seg LCD shows `STO __` → `STO _5` → `STO 05` as digits accumulate; `SF IND _5` when IND is on; `CLP MYPRG_` for text input. The `<Display14Seg />` component takes a single `text` prop — App.tsx passes either `calcState.display_str` (no modal) or a modal-preview string (modal active). Most HP-41CV-faithful per the user's "es muss einfach genau nach HP-41CV funktionieren" principle (Phase 25 D-25 specifics).

- **D-26.4: Single discriminated TypeScript union for `PendingInput`.** One `useState<PendingInput | null>` in `App.tsx`; one `handleModalKey(key, pending)` handler that returns the next state OR the final parameterized dispatch id; one `renderModalLcd(pending)` function that emits the LCD preview string. Schema:
  ```typescript
  type PendingInput =
    | { kind: 'flag', testKind: FlagTestKind, ind: boolean, acc: string }
    | { kind: 'register', op: RegisterOpKind, ind: boolean, acc: string }
    | { kind: 'clp', acc: string }
    | { kind: 'del', acc: string }
    | { kind: 'tone' }
    | { kind: 'xeq_name', acc: string }
    | { kind: 'fmt', mode: 'fix' | 'sci' | 'eng' }
    | { kind: 'assign_key' }
    | { kind: 'assign_label', keyCode: number, acc: string }
    | { kind: 'confirm_load', programIdx: number }
    | { kind: 'hex', acc: string }
    | { kind: 'print' };
  ```
  Mirrors `hp41-cli::app::PendingInput` 1:1 by intent (modulo TS naming idioms). `FlagTestKind` and `RegisterOpKind` are TypeScript string-union types (e.g. `'SF' | 'CF' | 'FsQuery' | …`) hand-defined to mirror the Rust enums — no codegen yet (revisit in Phase 27 if drift becomes a problem).

- **D-26.5: Frontend intercepts prompt-ids in `handleClick` — never sent to `dispatch_op`.** When `key.id` is a modal-opener (the 13 `*_prompt` ids plus `xeq_prompt`, `gto_prompt`, `lbl_prompt`, `asn`, `view`, `catalog`, plus the existing `fix_prompt`/`sci_prompt`/`eng_prompt` FmtDigits triggers), `handleClick` sets `pendingInput` state via `setPendingInput({ kind: …, … })` instead of calling `invokeForKey`. The backend stub-error arm for prompt ids in `key_map.rs::resolve` STAYS as defense-in-depth — a regression where the frontend forgets to intercept would surface as a toast, never silent. The existing `test_modal_prompt_ids_are_stubs_for_now` test asserts this contract continues to hold.

### Polish (D-26.6 — D-26.10)

- **D-26.6: 14-segment LCD font — full segment grid with dim 'off' segments.** Each character cell renders all 14 segments unconditionally. 'On' segments use bright LCD-green (`#a0ffa0`-ish); 'off' segments stay at ≈10% opacity (faintly visible — the authentic HP-41C LCD aesthetic). Glyph map covers HP-41C character set: A–Z, 0–9, period, comma, minus, plus parentheses, equals, slash, colon, space. Special HP-41 chars (Σ, π glyph, μ-superscript, …) are NOT in scope for Phase 26 — those land with v3.x ALPHA-special-charset (per D-25.5 deferral). SVG cost: 12 chars × 14 segments = 168 paths per render; acceptable for a 12-char display refreshed at human-scale rates.

- **D-26.7: `<Display14Seg text={...} />` is a drop-in replacement inside the existing `.display` div.** App.tsx changes from `<div className="display">{calcState.display_str}</div>` to `<div className="display"><Display14Seg text={displayText} /></div>` where `displayText` is either `calcState.display_str` (no modal) or the modal-preview string (modal active, per D-26.3). Existing `.display` CSS (positioning, background, border-radius) is preserved unchanged. The new file is `hp41-gui/src/Display14Seg.tsx`; glyph map lives in the same file as a constant `SEGMENT_MAP: Record<string, number[]>` (segment indices 0..13 set per glyph).

- **D-26.8: `?`-overlay = full-cover modal, categorized + searchable.** Press `?` to open a semi-transparent overlay covering the calculator; `?` or Esc closes it. Data source: `import functions from '../../docs/hp41cv-functions.json'` (vite JSON-import per D-25.16). Layout: a search input at the top filters across `display_name` + `description` + `category` (fuzzy substring match is fine — no need for a fuzzy-search library). Categories from JSON `category` field become section headings. Each entry row: `key_path | display_name | description` (filter out `key_path == null` entries — those are XEQ-by-Name-only ops, not relevant for a keyboard-shortcut overlay). Mirrors the CLI's `?` overlay UX. TypeScript types for the JSON entry are a hand-typed `interface HelpEntry { … }` in `hp41-gui/src/help_data.ts` — revisit codegen if the JSON schema starts evolving.

- **D-26.9: USER mode key-assignment overlay = per-key text relabel.** When `Op::UserMode` is active (i.e. `calcState.annunciators.user === true`) and the user has ASN'd a key, the SVG cap renders the ASN'd label (e.g. `MYPRG`) **in place of** the primary label. Implementation: `KeyDef` is extended to optionally carry a `keyCode: number` field (HP-41 row×10+col code per `keycode_to_hp41_code`); App.tsx passes the current `Op::Asn` mapping table (sourced from `CalcStateView.user_keymap` — new field, FN-GUI-05 — see D-26.11) to `<Keyboard />` via prop; Keyboard.tsx applies the relabel during render when the keyCode has an entry. This requires Phase 26 to expose the ASN map through `CalcStateView` (≤500-byte budget per FN-GUI-05).

- **D-26.10: Physical-keyboard `'p'` remap.** The MAP table in `App.tsx::resolveKeyId` swaps `'p': 'prx'` to `'p': 'prgm_mode'`. SHIFT+`'P'` (uppercase, since modifier is detected via case in the existing MAP convention) routes to `'prx'`. The SVG-keyboard click path is unaffected — the existing `KEY_DEFS` 'p' entries (the alphaChar 'P' on row 4 col 3, the explicit print key if any) stay as-is.

### CalcStateView budget (D-26.11)

- **D-26.11: CalcStateView gains `user_keymap: Vec<(u8, String)>`, `flags: u64` (or compact representation), `display_override: Option<String>` if it doesn't yet exist, `event_buffer: Vec<String>` (drained per IPC like print_lines).** Budget audit: each entry is ~10–20 bytes; HP-41CV typical ASN count ≈ 5–10 keys; `flags` is 8 bytes raw or ~30 bytes as a JSON list of set-flag-indices; `display_override` is usually None or a short string. Total budget projection: ~200 bytes baseline + ~100 bytes new fields = ≤500 bytes per FN-GUI-05. Document the new envelope in CLAUDE.md per ROADMAP cross-cutting constraint. **Planner's job:** confirm which of these fields already exist on `CalcState` (some may have shipped Phase 21 but not yet surfaced in `CalcStateView`); add the missing ones to `types.rs::from_state`; verify size with a sanity test (`serde_json::to_string(&view).len() <= 500`).

### Plan structure (D-26.12)

- **D-26.12: 3 plans, sequential dependency chain.**
  - **26-01 modal-architecture-and-key-wiring:**
    - Frontend `PendingInput` discriminated union + `useState<PendingInput | null>` in App.tsx
    - `handleModalKey()` dispatch logic (digit accumulation, IND-toggle via shift-0, end-of-modal dispatch)
    - `renderModalLcd()` LCD-preview string emitter
    - `handleClick` extension: intercept the 13 prompt-ids and the 4 stubbed-bare-ops-with-modals (`asn`, `view`, `catalog`); direct-dispatch the 4 bare-Op-no-modal ids (`pi`, `polar_to_rect`, `rect_to_polar`, `beep`)
    - `key_map.rs::resolve` extension: add named-op resolvers for every new v2.2 Op (Pi, PolarToRect, RectToPolar, Rnd, Frc, Abs, Sign, Fact, Mod, RUp, Sf*/Cf*, FsQuery*, View, Tone, Stop, Pse, Clp, Del, Ins, Size, Cla, Clst, Pack, Catalog, Asn, Arcl, Asto, Atox, Xtoa, Arot, Posa, all `*Ind` variants — total ~80–90 named resolvers + parameterized prefixes like `sto_ind_NN`, `sf_NN`, `sf_ind_NN`)
    - Shrink stub-error arm to v3.x-module ops only (Math Pac / Stat Pac / Time / Advantage names)
    - ASN 2-step flow (AssignKey → AssignLabel) wired through the discriminated-union modal
    - Tauri command registry: new permission TOMLs if any new dedicated commands are needed (audit `sst_step`-style precedent; most modal flows go through `dispatch_op` with the final parameterized id)
    - Vitest tests for `handleModalKey` (state-transition unit tests) + new Rust tests in `key_map::tests` for every new named id
    - Acceptance: every HP-41CV ROM op resolves successfully via `key_map::resolve`; the v3.x-module stub arm contains zero HP-41CV ops; the 13 prompt-ids are intercepted in `handleClick` and never reach `invokeForKey`.
  - **26-02 14-seg-lcd:**
    - `hp41-gui/src/Display14Seg.tsx` — SVG component with 14-segment grid per character, dim/bright segment-fill toggling per glyph
    - `SEGMENT_MAP: Record<string, number[]>` covering A–Z, 0–9, period, comma, minus, plus parens, equals, slash, colon, space
    - Drop-in replacement inside existing `.display` div per D-26.7
    - Wire modal-preview string through `displayText` prop per D-26.3
    - Vitest visual-regression-friendly tests (snapshot or pixel-diff via `vitest-image-snapshot` — planner picks)
    - Acceptance: rendering `2.0000` shows the digit glyphs with dim 'off' segments visible; modal preview `STO _5` renders with the trailing-underscore digit-entry cursor convention.
  - **26-03 polish-bundle:**
    - `?` overlay: `hp41-gui/src/HelpOverlay.tsx` + vite JSON-import of `docs/hp41cv-functions.json` + `hp41-gui/src/help_data.ts` (TypeScript HelpEntry interface + filter helpers)
    - USER mode per-key relabel: `KeyDef.keyCode` field + `<Keyboard />` `userKeymap` prop + render-time relabel; depends on Plan 26-01 having added `user_keymap` to `CalcStateView`
    - `'p'` → `prgm_mode` and SHIFT+`'P'` → `prx` in `App.tsx::resolveKeyId`'s MAP table
    - Acceptance: pressing `?` opens the overlay with all 130+ JSON entries categorized + searchable; toggling USER mode after `ASN "TEST" 22` shows `TEST` relabeling the key at code 22; pressing 'p' on physical keyboard toggles PRGM mode (verify via annunciator); pressing SHIFT+'P' invokes PRX.

  Each plan is independently testable, atomic-commit-friendly, and reaches the wave-3-of-Phase-26 verification gate independently. Dependencies: 26-02 and 26-03 both depend on 26-01 (modal infrastructure must exist before LCD-modal-preview integration or USER relabel). 26-02 and 26-03 are parallelizable.

### Claude's Discretion

- **TypeScript `FlagTestKind` / `RegisterOpKind` enum membership.** Planner finalizes the exact TS string-union members to match the Rust enums shipped Phase 21/23/24. Drift between Rust and TS is a known risk — a Vitest+Cargo doctest pair could lock parity, but for Phase 26 a manual cross-check during plan 26-01 is sufficient. Revisit codegen in Phase 27 if drift becomes a problem.
- **14-seg glyph map authoring.** Planner picks the SVG segment coordinates (typical 14-seg conventions: 7 outer segments + 4 mid-bar segments + 3 dot/decimal segments, or the more common 7+2+4+1 split). Free public-domain references include the Wikipedia "Fourteen-segment display" article and Adafruit's HT16K33 character set. Output: a constant `SEGMENT_MAP` in `Display14Seg.tsx`.
- **Help-overlay TypeScript type generation.** Hand-typed interface in `hp41-gui/src/help_data.ts` is the Phase 26 default. If the JSON schema gains fields later, planner is allowed to introduce a `quicktype` codegen step (or its equivalent) — document the choice in the plan.
- **USER mode keyCode mapping in `KeyDef`.** Planner decides whether to compute keyCode from `(row, col)` at render time via `keycode_to_hp41_code()`-equivalent or to hard-code the keyCode field in `KEY_DEFS`. Hard-coding is more explicit; computing keeps the source of truth in one place.
- **Permission TOMLs for any new Tauri commands.** If Plan 26-01 introduces dedicated commands beyond `dispatch_op` (e.g. for the ASN flow), planner creates the corresponding `permissions/<cmd-kebab>.toml` files and registers them in `capabilities/default.json`. Default assumption: ASN goes through `dispatch_op` with parameterized id `asn_NN_NAME` — no new command needed.

### Cross-cutting invariants (carried forward, NOT re-decided)

- **D-25.6 (CLI ↔ GUI parity, Phase 25).** Logical states, one-shot SHIFT lifetime, ALPHA-overrides-SHIFT, IND-toggle via shift-0 — Phase 26 mirrors CLI Phase 25 bit-for-bit at the user-observable layer.
- **D-07 (no silent discards, Phase 14).** Every clickable key produces a state-update OR a `GuiError` toast; the stub-error arm + the `test_modal_prompt_ids_are_stubs_for_now` defense-in-depth test stay in force.
- **SC-4 invariant (no calculator logic in `hp41-gui/src-tauri/`).** Phase 26 adds named-op resolvers and parameterized prefixes — pure dispatch routing. The stricter grep `fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)` must continue to return zero matches.
- **D-25.16 (vite JSON-import for shared help data).** Phase 26 implements the GUI side of this — the canonical source `docs/hp41cv-functions.json` is read at build time, zero duplication.
- **D-18 (Phase 8) — `help_data.rs` single-source-of-truth for CLI key descriptions.** Phase 26 does NOT touch `help_data.rs`; the GUI's `help_data.ts` is a TypeScript-side parallel reader of the SAME canonical JSON file.
- **4-place Op-variant landing rule (D-22.21 / D-23.12).** Phase 26 INVERTS this from Phase 25's perspective: the variants are already landed in `dispatch()` + `execute_op()` + both `prgm_display.rs` copies (Phase 25 closed the loop). Phase 26 only adds key_map resolvers — the rule does not generate new work here.
- **Save-file backward compat preserved.** No new `CalcState` fields in Phase 26 (`user_keymap`/`flags`/etc. are already on `CalcState` from earlier phases — Phase 26 only adds them to the `CalcStateView` projection). `#[serde(default)]` invariant continues to apply to any pre-existing fields that haven't yet been surfaced.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level
- `.planning/PROJECT.md` — Project goals, build sequence, evolution rules
- `.planning/REQUIREMENTS.md` §FN-GUI-01..05, §FN-POLISH-01..04 — locked v2.2 requirements for Phase 26
- `.planning/ROADMAP.md` Phase 26 section (lines 165–185) — phase goal, success criteria, cross-cutting constraints
- `.planning/STATE.md` — current milestone state (v2.2, Phases 20–25 shipped)
- `CLAUDE.md` — settled architecture decisions (especially v2.0 + v2.1 + v2.2 sections), critical implementation traps, key file index

### Prior phase context (carry-forward)
- `.planning/phases/25-cli-integration-and-documentation/25-CONTEXT.md` — D-25.1..D-25.18 (full CLI integration architecture; D-25.6 is the parity invariant this phase must satisfy; D-25.11..D-25.14 define the modal architecture this phase mirrors; D-25.16 the JSON pipeline this phase consumes)
- `.planning/phases/25-cli-integration-and-documentation/25-02-pending-input-modals-PLAN.md` — CLI's `PendingInput` hybrid struct-variants implementation (D-26.4's TypeScript union is the GUI mirror of this)
- `.planning/phases/25-cli-integration-and-documentation/25-04-json-pipeline-and-docs-PLAN.md` — JSON pipeline source-of-truth (D-26.8 consumes the same file via vite import)
- `.planning/phases/24-indirect-addressing/24-CONTEXT.md` — D-24.1..D-24.9 (IND variants Phase 26 wires keyboard paths for)
- `.planning/phases/21-flags-display-control-and-sound/` — FlagTestKind enum shape, run_loop skip semantics, `display_override`/`event_buffer` fields likely added on CalcState here
- `.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md` — D-22.18 ASN modal flow (Phase 26 wires its GUI)
- Phase 19 (no GSD directory; documented in `MILESTONES.md` v2.1 + `CLAUDE.md` v2.1 section) — GUI one-shot SHIFT pattern (`shiftActive`), D-5 ALPHA-overrides-SHIFT deferral, `KeyDef` three-label schema, toast overlay, `invokeForKey` + `extractErrMessage` helpers

### Codebase files (key integration targets)
- `hp41-gui/src-tauri/src/key_map.rs` — `resolve()` + `resolve_parameterized()` (this phase extends both significantly per D-26.5)
- `hp41-gui/src-tauri/src/types.rs` — `CalcStateView` (this phase adds `user_keymap`, `flags`, `display_override`, `event_buffer` projections per D-26.11)
- `hp41-gui/src-tauri/src/commands.rs` — `dispatch_op` etc. (may need new commands for ASN — planner decides per D-26.12)
- `hp41-gui/src-tauri/permissions/*.toml` — Tauri v2.11 inline-command permission registry (new TOMLs if new commands)
- `hp41-gui/src-tauri/src/lib.rs` — `generate_handler!` registration for any new commands
- `hp41-gui/src/App.tsx` — modal state (`useState<PendingInput | null>`), `handleClick` extension (D-26.5), `handleModalKey()`, `renderModalLcd()`, MAP table update (D-26.10), `displayText` derivation, USER overlay prop passing
- `hp41-gui/src/Keyboard.tsx` — `KEY_DEFS` extension (any missing v2.2 keys), `KeyDef.keyCode` field, USER overlay rendering (D-26.9)
- `hp41-gui/src/Display14Seg.tsx` — NEW (D-26.6 + D-26.7); segment grid SVG component + glyph map
- `hp41-gui/src/HelpOverlay.tsx` — NEW (D-26.8); `?`-overlay React component
- `hp41-gui/src/help_data.ts` — NEW (D-26.8); `HelpEntry` interface + JSON-import wrapper
- `hp41-gui/src/App.css` — minor adjustments for overlay + USER relabel styles
- `docs/hp41cv-functions.json` — CANONICAL DATA SOURCE; vite-imports at build time per D-25.16
- `hp41-cli/src/app.rs` — REFERENCE ONLY (the `PendingInput` shape this phase mirrors lives here; CLI is not modified)
- `hp41-cli/src/keys.rs` — REFERENCE ONLY (`key_to_op` + `shifted_key_to_op` semantics this phase mirrors at the resolve layer)
- `hp41-core/src/ops/mod.rs` — READ-ONLY (Op enum; source of named-op resolver targets in `key_map.rs`)
- `CLAUDE.md` — needs a Phase 26 update at end-of-phase (new envelope size for CalcStateView per FN-GUI-05; modal frontend-owned pattern D-26.1; 14-seg drop-in D-26.7)

### HP-41CV hardware references (external — planner sources only if a binding is ambiguous)
- HP-41C Owner's Manual Appendix B — keyboard reference card (already sourced for Phase 25; Phase 26 inherits the same bindings)
- Free42 source `keyboard.h` (or equivalent) — alternate authoritative source
- HP-41C/CV Quick Reference Guide p.14 — IND-toggle keystroke convention (D-25.12 / D-26.2)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`shiftActive` state machine in `App.tsx`** (Phase 19) — frontend-owned one-shot SHIFT precedent that D-26.1 generalizes to modal state. `setShiftActive`, Tab toggle, Esc cancel, click-on-SHIFT-key toggle — all directly usable.
- **`invokeForKey()` helper in `App.tsx:48`** — routes special ids (sst/bst/r_s) to dedicated commands, everything else to `dispatch_op`. The new `handleModalKey` either calls `invokeForKey(finalId)` at end-of-modal-accumulation or sets up the next pending state.
- **`extractErrMessage()` in `App.tsx:34`** — handles `GuiError { message }` shape; toasts surface via `showToast(extractErrMessage(err))`. No changes needed.
- **Toast overlay pattern in `App.tsx:119`** — `{ msg, seq }` with monotonic counter for re-firing identical messages; 2s auto-dismiss. Reusable for modal-error feedback.
- **`KeyDef.shifted: { id, label }` + `alphaChar`** (Keyboard.tsx) — three-label model fully shipped; Phase 26 only adds new shifted ids (`asn`, `view`, `catalog`, ALL new modal-opener ids) and the optional `keyCode` field for USER overlay.
- **`pressedKey` animation state in Keyboard.tsx** — 150ms visual press indicator; works for modal-opener clicks unchanged.
- **`busyRef` two-layer debounce** (App.tsx + Keyboard.tsx) — prevents concurrent `invoke()` calls; modal interactions don't go through `invoke()` until end-of-accumulation, so the existing guard remains correct.
- **`docs/hp41cv-functions.json`** (Phase 25, 1395 lines, ~130 entries) — canonical help data source already shipped; Phase 26 only adds the TypeScript consumer side.
- **`CalcState.user_keymap`** (Phase 22, FN-KEY-01 ASN) — already exists in core; Phase 26 only surfaces it through `CalcStateView`.

### Established Patterns

- **Frontend-owned UI state with backend round-trip on final dispatch** — `shiftActive` is the precedent; `pendingInput` extends the same pattern (D-26.1).
- **Discriminated unions for one-of-N states** — common React-with-TS pattern; matches how the Rust enum is shaped (D-26.4).
- **JSON import via vite** — already used elsewhere in the project; D-25.16 codifies it for help data (D-26.8 consumes).
- **D-07 no silent discards** — Phase 14 invariant; defense-in-depth pattern of "backend stub AND frontend intercept both safe" extends naturally (D-26.5).
- **Drop-in component replacement preserves CSS layout** — Phase 18 program-listing-panel and Phase 19 toast both followed this pattern (D-26.7).
- **Permission TOML registry per Tauri command** — Phase 14 + Phase 19 (run_stop) precedent; required for any new commands.

### Integration Points

- **`hp41-gui/src/App.tsx::handleClick`** — main extension point for D-26.5 (intercept modal-opener ids). Existing CL X/A and SHIFT-toggle branches stay; new branches dispatch into `setPendingInput`.
- **`hp41-gui/src/App.tsx::handleKey`** — physical-keyboard path; needs the `'p'` MAP swap (D-26.10) and likely a new "if `pendingInput`, route to `handleModalKey` instead of `dispatchKeyId`" branch so physical-keyboard digits feed the modal too.
- **`hp41-gui/src/App.tsx` display rendering** — `<div className="display">` body becomes `<Display14Seg text={displayText} />` where `displayText = pendingInput ? renderModalLcd(pendingInput) : calcState.display_str` (D-26.3 + D-26.7).
- **`hp41-gui/src-tauri/src/key_map.rs::resolve`** — new named-op resolvers; the stub-error arm shrinks (D-26.5 keeps it as defense-in-depth for prompt-ids only; the bare-op stubs `pi`/`polar_to_rect`/`rect_to_polar`/`beep` move to real `Ok(Op::*)` arms; `view`/`catalog`/`asn` remain in stub-or-modal land but the frontend intercepts them before they reach the stub).
- **`hp41-gui/src-tauri/src/key_map.rs::resolve_parameterized`** — adds the new parameterized prefixes: `sf_NN`, `sf_ind_NN`, `cf_NN`, `cf_ind_NN`, `fs_NN`, `fs_ind_NN`, `view_NN`, `view_ind_NN`, `arcl_NN`, `arcl_ind_NN`, `asto_NN`, `asto_ind_NN`, `tone_N`, `del_NNN`, `catalog_N`, `clp_LABEL` (label as raw string suffix), `sto_ind_NN`, `rcl_ind_NN`, `isg_ind_NN`, `dse_ind_NN`, `sto_arith_<op>_ind_NN`, plus the new conditional-test parameterized prefixes if any (the 4 keyboard-bound XEqY/XLeY/XGtY/XEqZero don't need params; the 8 XEQ-by-Name ones go through `Op::Xeq(name)` already wired).
- **`hp41-gui/src-tauri/src/types.rs::from_state`** — `CalcStateView` constructor; this phase adds the new projected fields (D-26.11).
- **`hp41-gui/src/Keyboard.tsx` `KEY_DEFS`** — verify completeness vs the 80+ new ops; some are reachable only via XEQ-by-Name (8 conditional tests, the IND variants are not directly bound — IND is reached via shift-0 inside an open register/flag modal), most other ROM ops are already represented in the v2.1 keyboard (just need correct `shifted`/`alphaChar` updates if any are missing). Plan 26-01 audits this systematically.

</code_context>

<specifics>
## Specific Ideas

- **"Es muss einfach genau nach HP-41CV funktionieren"** — Phase 25 D-25.18 specifics meta-principle continues into Phase 26. When two implementation choices have equal Tauri/React-idiomatic value, pick the one that mirrors the real HP-41CV hardware behavior. Hardware-faithful modal-preview-in-LCD (D-26.3), shift-0 IND-toggle (D-26.2), per-key text relabel in USER mode (D-26.9), full segment grid for the 14-seg display (D-26.6) all derive from this principle.

- **CLI is the reference design.** The user explicitly requires identical observable behavior in CLI Phase 25 and GUI Phase 26 per D-25.6. The CLI's `PendingInput` enum + `pending_prompt()` + `shift_armed` field is the canonical model; Phase 26 mirrors it in TypeScript. The frontend-owned vs. backend-owned trade is decided in the GUI's favor only because v2.1 already established `shiftActive` as the parity model — Phase 26 extends that, not contradicts it.

- **High-fidelity LCD aesthetic.** The user picked "full segment grid with dim 'off' segments" over "static lit-only" — Phase 26 ships the more visually demanding option (D-26.6). Phase 27 test hardening should include a visual-regression snapshot for the new display so accidental segment-map drift is caught.

- **3-plan split with clear dependency chain.** Plan 26-01 ships the modal infrastructure that 26-02 (LCD preview integration) and 26-03 (USER overlay) depend on. Plans 26-02 and 26-03 are parallelizable. Each plan stays under the ~30KB plan-size budget.

</specifics>

<deferred>
## Deferred Ideas

- **Full hardware-faithful ALPHA-mode-with-prefix behavior + HP-41 special-charset (Σ, π glyph, μ-superscript, …) in the 14-seg font** — v3.x territory (matches Phase 25 D-25.5 deferral). Phase 26's 14-seg glyph map covers A–Z, 0–9, common punctuation only.
- **Cmd-K-style fuzzy command palette dispatch path** — explicitly rejected during help-overlay discussion (D-26.8 picks the categorized full-cover modal instead). Introducing a "type op name, Enter to dispatch" path would be a second op-dispatch surface not present on the physical HP-41CV; rejected per the hardware-faithful principle.
- **Floating React modal overlays for accumulator input** — rejected during modal-display discussion (D-26.3 picks LCD-replace instead). Less hardware-faithful; not the v2.0 design language.
- **Dedicated IND button on the modal overlay** — rejected (D-26.2 picks shift-0). Hardware-faithful trumps discoverability for this project.
- **Backend-owned `pending_input` field on `CalcStateView`** — rejected (D-26.1 picks frontend-owned). Would have grown the IPC surface ~80–120 bytes and duplicated PendingInput shape across Rust+TS for no observable-behavior gain.
- **Per-modal separate React components (`StoModal.tsx`, `FlagModal.tsx`, …)** — rejected (D-26.4 picks single union). Would have required N parallel render functions and made the "at most one modal open" invariant harder to enforce.
- **TypeScript codegen from Rust enums** — deferred (Claude's Discretion). Phase 26 hand-types `FlagTestKind` / `RegisterOpKind`; Phase 27 may add a codegen step (quicktype or similar) if drift becomes a problem.
- **`prx` migration to a non-letter key** — rejected (D-26.10 picks SHIFT+`P`). The shifted-letter pattern is the cleanest physical-keyboard migration.
- **Playwright GUI E2E smoke test booting the Tauri app + asserting display** — Phase 27 (FN-QUAL-05).
- **Visual-regression snapshot tests for `<Display14Seg />`** — Phase 27 if not folded into Plan 26-02's acceptance tests.
- **README "feature-complete HP-41CV" HARD claim** — Phase 27 conditional on coverage gates (carried forward from D-25.17).

</deferred>

---

*Phase: 26-gui-integration-and-polish*
*Context gathered: 2026-05-15*
