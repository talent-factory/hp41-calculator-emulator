# Phase 25: CLI Integration & Documentation — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 25-cli-integration-and-documentation
**Areas discussed:** Keyboard binding strategy, 12 Conditional Tests, PendingInput modal consolidation, Function matrix + JSON sharing

---

## Area 1 — Keyboard binding strategy

### Turn 1 — Binding style

| Option | Description | Selected |
|--------|-------------|----------|
| Direct + extended modals (status quo) | Keep v1.x crossterm-style direct mapping; expand modal triggers; rare ops via XEQ palette | |
| True HP-41 f/g-prefix shift modal | Hardware-faithful prefix model; one-shot armed-state | ✓ |
| Hybrid — F-keys for math, Shift+letter for ALPHA/program, XEQ-palette for rare | Tiered keymap by frequency | |

**User's choice:** True HP-41 prefix shift modal.

### Turn 2 — Prefix count + key labelling

| Option | Description | Selected |
|--------|-------------|----------|
| Two prefixes: 'f' (orange) + 'g' (blue), hardware-faithful | Mirror HP-15C / HP-12C with two shift keys | initially picked, **then corrected** |
| One prefix: 'f' only — matches GUI v2.1 | Single SHIFT key (matches actual HP-41C/CV/CX hardware) | ✓ (post-correction) |
| Two prefixes reassigned to '`' and '\\' | Keep f/g as direct ops; backtick/backslash as prefix triggers | |

**User's correction:** *"Mein physischer HP-41CV kennt nur die ‚gelbe‘ Taste."* — Corrected the initial "two prefixes" answer to ONE prefix, matching real HP-41CV hardware reality. Claude had been misremembering HP-41 hardware as similar to HP-15C/12C (which have two shift keys). The HP-41C/CV/CX actually has ONE orange/yellow shift key labelled `f`.

**Final lock:** D-25.2 → ONE yellow prefix (`f` in HP-41 nomenclature).

### Turn 2b — Migration scope

| Option | Description | Selected |
|--------|-------------|----------|
| Keep existing direct maps; prefix-modal only for NEW ops | Don't break v1.x muscle memory | |
| Migrate everything to the HP-41 model; deprecate existing direct maps | Full HP-41 keyboard reference card fidelity | ✓ |
| Migrate only display/format/modal ops; keep math/trig direct | Compromise hybrid migration | |

**User's choice:** Full migration. v1.x bindings like `C` for COS, `L` for LN, `q` for SIN are deprecated.

### Turn 3 — Prefix lifetime + ALPHA interaction

| Option | Description | Selected |
|--------|-------------|----------|
| One-shot, matches GUI v2.1 | Arm-then-consume; auto-clear after next op-key | ✓ (implicit, matches HP-41CV hardware) |
| Sticky until cancelled | Lock-mode allows repeated f-shifted ops without re-arming | |
| One-shot with f-f lock | Compromise (double-tap locks) | |

**User's response:** *"Diesen Entscheid verstehe ich noch nicht ganz. Es muss einfach genau nach HP-41CV funktionieren."* — Deferred to hardware-truth. HP-41CV hardware behavior is one-shot, no lock mode.

**Final lock:** D-25.4 → One-shot prefix, hardware-faithful.

### Turn 3b — ALPHA-mode + prefix interaction

| Option | Description | Selected |
|--------|-------------|----------|
| ALPHA overrides Prefix (matches GUI v2.1 D-5) | In ALPHA mode, `f` types F; no prefix arming | ✓ (implicit, matches Phase 19 D-5 deferral) |
| Prefix overrides ALPHA (hardware-faithful) | Type Σ, π, μ etc. via prefix in ALPHA mode | |
| Defer to v3.x | No prefix behavior in ALPHA for v2.2 | (same effect as option 1 for v2.2) |

**User's response:** *"Ich wünsche einfach ein reales HP-41CV Verhalten. Dies bedeutet, dass dies sowohl im CLI als auch in der GUI Version identisch implementiert werden muss."* — Wants identical CLI/GUI behavior; full hardware-faithful ALPHA-prefix interaction is a larger v3.x scope (needs special-char table + display work).

**Follow-up AskUserQuestion was DENIED** — user replied "Einverstanden" to Claude's text-proposal: defer the full hardware ALPHA-prefix to v3.x (matches existing GUI v2.1 D-5 deferral); v2.2 stays with ALPHA-overrides-Prefix in both CLI and GUI.

**Final lock:** D-25.5 → ALPHA overrides Prefix in v2.2; v3.x deferral.

---

## Area 2 — 12 Conditional Tests on the keyboard

### Turn 1 — Surfacing strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Test-Submodal via `f-?` or `f-T` opens 12-entry picker | Single entry point with picker UI | |
| Six direct (X-vs-Y) + six modal (X-vs-0) | Compromise | |
| All 12 only via XEQ-by-Name palette | Minimal keyboard, maximal modal | |
| Inner-Test-Prefix: `f-?` arms 'Test'-mode, next key picks 1–12 | Two-stage prefix | |

**User's response (hardware ground truth provided):** *"Mein HP-41CV hat aktuell folgendes Layout, erreichbar durch die ‚gelbe‘ Taste: `x=y` → `-`, `x <= y` → `+`, `x>y` → `*`, `x=0` → `/`"* — User gave concrete hardware-faithful bindings for 4 of the 12 tests directly from their physical device. None of the proposed options matched this; the answer effectively redefined the question.

**Final lock from user's correction:** D-25.7 → 4 tests on `f-` / `f+` / `f*` / `f/` keys exactly mirroring the user's HP-41CV.

### Turn 2 — Reachability of the OTHER 8 tests

| Option | Description | Selected |
|--------|-------------|----------|
| Hardware-faithful: only via XEQ-by-Name | The 8 non-keyboard tests reached via `XEQ "X<>Y"` mnemonic | ✓ |
| Hardware-faithful + Catalog-Helper: `f-?` opens 12-entry picker | 4 hardware-shipped + 8 picker-accessible | |
| Extended f-bindings: assign 8 tests to creative f-shifted slots | Diverges from "Migrate everything to HP-41CV" principle | |

**User's choice:** Hardware-faithful — XEQ-only for the 8 non-keyboard tests.

**Final lock:** D-25.8 → 8 non-keyboard tests reachable only via XEQ-by-Name (matches HP-41CV ROM access pattern); D-25.9 → "reachable from CLI keyboard" satisfied via keystroke-sequence (XEQ counts); D-25.10 → v1.x X≥Y direct-binding removed.

---

## Area 3 — PendingInput modal consolidation

### Turn 1 — Modal structure

| Option | Description | Selected |
|--------|-------------|----------|
| Unique variant per op (status quo, ~30+ variants) | StoIndPrompt, RclIndPrompt, SfPrompt, etc. each as separate enum variant | |
| Generic NumericPrompt struct with OpKind tag | Single variant with NumericOpKind discriminator | |
| Hybrid: group variants for Flag/Register, unique for specialty | FlagPrompt + RegisterPrompt structs + CLP/DEL/TONE unique | ✓ (after Claude's recommendation) |

**User's response:** *"Was ist dein Vorschlag? Best Practice in Rust"* — Asked for Claude's recommendation.

**Claude's recommendation (delivered as a follow-up text turn):** Hybrid (Option C). Rationale: "make illegal states unrepresentable" — a generic NumericPrompt forces TONE (single-digit) and CLP-Label (text) into a Procrustean shape. Hybrid reuses existing hp41-core enums (FlagTestKind from Phase 21, StoArithKind from Phase 9) for the group variants while preserving specialty semantics. Reduces ~30 arms to ~18.

**User confirmed:** "Einverstanden: Hybrid + Boolean-Flag".

**Final lock:** D-25.11 → Hybrid struct-variants.

### Turn 2 — IND modifier in modal flow

| Option | Description | Selected |
|--------|-------------|----------|
| Boolean flag in modal state (`ind: bool`) | Toggle-bar mid-input via IND key; single state struct | ✓ (after Claude's recommendation) |
| Separate IND-Modal-Variant (StoIndPrompt vs StoPrompt) | Double the modal variants | |
| Pre-IND-State: User presses IND FIRST, then Op | Diverges from HP-41CV hardware flow | |

**User's response:** *"So Hardwarenah wir nur möglich. Was ist hier dein Vorschlag?"* — Asked for hardware-faithful recommendation.

**Claude's recommendation:** Boolean flag (Option A). Hardware-faithful — HP-41CV input order is Op-Press → IND → Digits, with IND toggle-able mid-input. Boolean flag in struct maps 1:1; dispatch decision is single-point at end (`if ind { Op::*Ind(n) } else { Op::*(n) }`).

**User confirmed:** "Einverstanden: Hybrid + Boolean-Flag".

**Final lock:** D-25.12 → IND as `ind: bool` Field in modal structs.

---

## Area 4 — Function matrix + JSON sharing

### Turn 1 — Matrix source of truth

| Option | Description | Selected |
|--------|-------------|----------|
| Hybrid: hand-curated + CI-parity-check against Op enum | Matrix.md hand-edited; bidirectional CI drift-check | ✓ |
| Pure mechanical generation from Op enum | Zero drift; cannot represent v3.x-deferred ops | |
| Hand-curated without CI check | Drift-prone | |
| Community CSV as source | Inconsistent naming risk | |

**User's choice:** Hybrid — hand-curated + CI-parity-check.

**Final lock:** D-25.15.

### Turn 2 — Data-sharing architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Shared JSON (docs/hp41cv-functions.json) — Rust build.rs + TS-import | Single canonical JSON | ✓ (after Claude's recommendation, simplified) |
| Markdown matrix as source; help_data.rs + TS-overlay generated | One file, lots of tooling | |
| help_data.rs manual + Phase 26 TS mirror with doctest parity | Lowest-tooling | |
| Fully separate, no parity check | Drift guaranteed | |

**User's response:** *"Was wäre dein Vorschlag (Best Practices)?"* — Asked for Claude's recommendation.

**Claude's recommendation (delivered as a follow-up text turn):** Option 1 (JSON source) but with `include_str!` pattern, NO build.rs codegen — just `pub const FUNCTIONS_JSON: &str = include_str!(...)` + serde-deserialize lazy-parse via OnceLock. Phase 26 vite-imports the JSON natively. Matrix.md is generated from JSON via a `just docs-matrix` justfile recipe and committed (CI test ensures committed-MD matches what regen would produce).

**User confirmed:** "Einverstanden — JSON-Source, include_str!, justfile-recipe".

**Final lock:** D-25.16.

### Turn 3 — README claim timing

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 25 + soft claim with link to Matrix + Divergences | "Implements full HP-41CV ROM with documented divergences" | ✓ |
| Phase 25 hard claim "feature-complete HP-41CV" | No caveats | |
| Defer to Phase 27 | Wait for test-hardening to validate | |

**User's choice:** Phase 25 + soft claim.

**Final lock:** D-25.17.

---

## Claude's Discretion

Areas explicitly delegated to planner / researcher (NOT decided in this discussion):

- **Exact IND key position on HP-41CV reference card** — planner sources HP-41C Owner's Manual Appendix B / Free42 `keyboard.h` to confirm the physical position holding IND (likely f-XEQ, possibly a dedicated key on HP-41CX revisions).
- **`RegisterOpKind` enum membership** — planner finalizes which ops collapse into `RegisterPrompt` group variant (at minimum Sto / Rcl / StoAdd / StoSub / StoMul / StoDiv / View / Arcl / Asto / Isg / Dse).
- **JSON entry count vs Op variant count** — function matrix has ≥130 entries but Op enum has ~190 variants today; the mapping is N-to-M (e.g. `Op::Sto(u8)` is one matrix row "STO", not 100). Planner builds the parity test's row-to-variant logic.
- **v1.x letter binding survival rules** — planner may keep v1.x letters IF they coincide with HP-41CV primary key labels; otherwise removed per D-25.3.
- **Help-overlay categorization derivation** — Planner decides whether `=== Stack ===` / `=== Math ===` etc. headers are derived from the JSON's `category` field or hand-grouped. Recommended: derive from JSON for DRY.

---

## Deferred Ideas

- **Full hardware-faithful ALPHA-mode + prefix behavior** — v3.x (needs ALPHA-special-char table for Σ, π, μ, etc.; affects both CLI and GUI).
- **README "feature-complete HP-41CV" HARD claim** — potential Phase 27 elevation if FN-QUAL gates pass.
- **Module-Pac emulation** (Math 1 / Stat 1 / Time / Advantage) — permanent v2.x exclusion; matrix lists them as `⏳ v3.x module`.
- **Two-prefix support (`f` + `g`)** — explicitly rejected for HP-41CV. May resurface only with a hypothetical HP-15C/12C emulator milestone.
- **Test-coverage gate ≥ 95 %, proptest sweep, Playwright E2E** — Phase 27 territory.
