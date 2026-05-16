# Phase 26: GUI Integration & Polish — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 26-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-15
**Phase:** 26-gui-integration-and-polish
**Areas discussed:** Modal state ownership, Modal component architecture, 14-seg LCD display scope, Help overlay + USER mode polish, prx-migration target, plan structure

---

## Gray-area selection

User chose all 4 architecture-sensitive areas via multiSelect — modal state ownership, modal component architecture, 14-seg LCD scope, help overlay + USER overlay bundle. Smaller items (p-key remap, plan split) followed as wrap-up.

---

## Modal state ownership

| Option | Description | Selected |
|--------|-------------|----------|
| Frontend-owned (parity w/ shiftActive) | React useState mirrors CLI's PendingInput; backend only sees the final parameterized id (sto_05). No new CalcStateView fields. Matches v2.1 shiftActive precedent. | ✓ |
| Backend-owned (full CLI parity) | Add `pending_input: Option<PendingInputView>` to CalcStateView (~80–120 bytes); round-trip every keystroke during a modal through dispatch_op. | |
| Hybrid (modal-by-modal choice) | Short numeric modals frontend-owned; string/multi-step modals backend-owned. Two parallel models. | |

**User's choice:** Frontend-owned.
**Notes:** D-25.6 parity = observable behavior, not implementation identity. v2.1 `shiftActive` is the precedent; `pendingInput` extends the same pattern. PRGM-mode duality continues to work because final parameterized op (Op::Sto(5)) auto-routes inside core dispatch.

---

## IND-toggle inside an open modal

| Option | Description | Selected |
|--------|-------------|----------|
| Shift-0 via shiftActive (CLI parity) | Press SHIFT then '0' inside an open modal — toggles `ind` field, not accumulator. Reuses existing shiftActive state. Hardware-faithful per HP-41C/CV QRG p.14 (D-25.12). | ✓ |
| Dedicated IND button rendered into modal overlay | Modal renders an [IND] button next to the digit display; click toggles. Easier discoverability; diverges from hardware. | |
| Both — shift-0 AND a UI toggle | Two paths to the same state. More code. | |

**User's choice:** Shift-0, full CLI parity.
**Notes:** Hardware-faithful trumps discoverability for this project.

---

## Modal accumulator visual rendering

| Option | Description | Selected |
|--------|-------------|----------|
| Replace 12-char LCD content while modal is open | LCD shows `STO __` → `STO _5` → `STO 05`. Most HP-41CV-faithful. Plays nicely with the 14-seg font. | ✓ |
| Status row above/below LCD | Separate row shows `STO [_5]`; LCD continues to show display_str unchanged. Easier; less faithful. | |
| Floating React modal overlay | Semi-transparent panel covers part of the calculator face. Discoverable; least faithful; new visual idiom. | |

**User's choice:** LCD content replacement.
**Notes:** The `<Display14Seg text={...} />` component takes a single `text` prop; App.tsx derives `displayText` as either `calcState.display_str` or a modal-preview string.

---

## Modal component architecture (TypeScript shape)

| Option | Description | Selected |
|--------|-------------|----------|
| Single discriminated union + single component | One `useState<PendingInput \| null>`; one handler `handleModalKey`; one renderer `renderModalLcd`. Mirrors Rust enum 1:1. | ✓ |
| Separate useState per modal + per-component renderers | N parallel state slots, only one ever active. More verbose; harder to enforce "at most one modal open". | |
| useReducer with finite state machine | Heaviest; overkill for the variant count. | |

**User's choice:** Single discriminated union.
**Notes:** Mirrors the Rust enum 1:1 by intent; modulo TypeScript naming idioms.

---

## Prompt-ID routing

| Option | Description | Selected |
|--------|-------------|----------|
| Frontend intercepts in handleClick — never reaches dispatch_op | Backend stub-error arm stays as defense in depth. Consistent with D-26.1. | ✓ |
| Backend `resolve` returns a new Op::OpenModal(kind) variant | Violates "no core changes" constraint. | |
| Backend returns a typed `modal_request` field on CalcStateView | Conflicts with frontend-owned ownership; adds IPC complexity. | |

**User's choice:** Frontend intercepts; backend stub stays as defense in depth.

---

## 14-segment LCD visual fidelity

| Option | Description | Selected |
|--------|-------------|----------|
| Static SVG glyph map, lit segments only | Each char is a `<g>` with only the segments needed for the glyph drawn. Lightweight. | |
| Full segment grid with dim 'off' segments | All 14 segments always rendered; off segments dim (≈10% opacity). High-fidelity LCD aesthetic. | ✓ |
| Glow + shadow effects (modern stylized) | Lit segments have a glow filter. Diverges from authentic HP-41C flat LCD. | |

**User's choice:** Full segment grid with dim off-segments.
**Notes:** SVG cost (12 chars × 14 segments = 168 paths) is acceptable for a human-scale-refreshed display.

---

## 14-seg display integration

| Option | Description | Selected |
|--------|-------------|----------|
| Drop-in replacement: `<Display14Seg />` inside existing `.display` div | Existing CSS preserved; modal-preview flows through the same prop. | ✓ |
| Replace entire `.display` div with top-level Display14Seg | More invasive; no value over option 1. | |
| Side-by-side: keep CSS text + add 14-seg as secondary panel | Defeats FN-POLISH-01. | |

**User's choice:** Drop-in replacement.

---

## Help overlay UX

| Option | Description | Selected |
|--------|-------------|----------|
| Modal overlay (full-cover, scrollable, categorized) | Press `?` to open; `?` or Esc to close. Categories from JSON. Searchable input at top. | ✓ |
| Side panel (split layout, persistent) | Conflicts with the fixed-aspect SVG calculator layout. | |
| Cmd-K command palette (search-first) | Introduces a second op-dispatch path not on physical HP-41CV. | |

**User's choice:** Modal overlay, categorized + searchable.

---

## USER mode key-assignment overlay

| Option | Description | Selected |
|--------|-------------|----------|
| Per-key text relabel: ASN label replaces primary label when USER active | Visually clear; matches HP-41C behavior on real hardware. | ✓ |
| Semi-transparent overlay layer with ASN labels | Less hardware-faithful; harder to position with wide rows. | |
| Tooltip on hover during USER mode | Bad for keyboard-only users. | |
| Separate list panel showing all current ASN bindings | Doesn't show which physical key is reassigned. | |

**User's choice:** Per-key text relabel.

---

## prx migration target (FN-POLISH-04)

| Option | Description | Selected |
|--------|-------------|----------|
| Shifted on `P` — `p` = prgm_mode, SHIFT+P = prx | Clean physical-keyboard migration. Existing MAP convention supports uppercase-as-shift. | ✓ |
| Remove `prx` from physical-keyboard MAP entirely | Loses keyboard-accessibility for PRX users. | |
| Move `prx` to a different unused letter | Less intuitive than shifted-P. | |

**User's choice:** Shifted on `P`.

---

## Plan structure

| Option | Description | Selected |
|--------|-------------|----------|
| 3 plans, sequential | 26-01 modal-architecture-and-key-wiring; 26-02 14-seg-lcd; 26-03 polish-bundle. 26-02 and 26-03 parallelizable after 26-01. | ✓ |
| Single plan with waves | One 40+KB plan; harder to revisit individual waves. | |
| 5 plans — one per FN- requirement family | Most granular; risks tiny plans for trivial work. | |

**User's choice:** 3 plans, sequential dependency chain.

---

## Claude's Discretion

- TypeScript `FlagTestKind` / `RegisterOpKind` enum membership — planner finalizes string-union members to match Rust enums. Phase 27 may add codegen if drift becomes a problem.
- 14-seg glyph map authoring — planner picks SVG segment coordinates; references include Wikipedia 14-seg article and Adafruit HT16K33 character set.
- Help-overlay TypeScript type generation — hand-typed `HelpEntry` interface in Phase 26 default; codegen later if schema evolves.
- `KeyDef.keyCode` field source — planner decides between computed-from-(row,col) or hard-coded.
- Permission TOMLs for new Tauri commands — planner creates if Plan 26-01 introduces dedicated commands; default assumption is `dispatch_op` with parameterized ids.

## Deferred Ideas

- Full hardware-faithful ALPHA-mode-with-prefix + HP-41 special-charset in 14-seg (Σ, π glyph, μ-superscript) — v3.x.
- Cmd-K-style fuzzy command palette dispatch path — rejected (hardware-faithful principle).
- Floating React modal overlays — rejected (LCD-replace picked instead).
- Dedicated IND button on modal overlay — rejected (shift-0 picked instead).
- Backend-owned `pending_input` on CalcStateView — rejected (frontend-owned picked).
- Per-modal separate React components — rejected (single union picked).
- TypeScript codegen from Rust enums — Phase 27 if drift problem.
- `prx` migration to non-letter key — rejected.
- Playwright GUI E2E smoke test — Phase 27.
- Visual-regression snapshot tests for `<Display14Seg />` — Phase 27 or fold into Plan 26-02 acceptance.
- README "feature-complete HP-41CV" HARD claim — Phase 27 conditional on coverage gates.
