# Phase 25: CLI Integration & Documentation — Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 25 wires every v2.2-added Op variant (Phases 20–24, ~80 new variants in `hp41-core::ops::Op`) into the **hp41-cli TUI** as a keyboard-reachable operation, and ships the v2.2 **documentation deliverables** (HP-41CV function matrix + CLAUDE.md additions + README claim). hp41-core is FROZEN (no new Ops, no new CalcState fields); hp41-gui changes are Phase 26 territory.

**This phase introduces a fundamental UX shift:** the CLI moves from its v1.x crossterm-style direct-letter mapping (e.g. `C` → COS, `L` → LN) to a **true HP-41CV prefix-shift model** with ONE yellow prefix key (`f` in HP-41 nomenclature). Every Op reaches via the HP-41CV keyboard reference card — not via ad-hoc letter conventions.

**Mandated by ROADMAP cross-cutting constraints:**
- This phase ONLY touches `hp41-cli`, `docs/`, and project-root `*.md` — NO `hp41-core` changes (all core Ops landed Phases 20–24)
- Build sequence: core → cli → docs → gui → tests; documentation runs synchronously with CLI integration (function matrix has authoritative coverage data)
- Function matrix entry format: ≥130 HP-41CV ROM ops with status (`✓ v2.x` / `⏳ v3.x module` / `— N/A`)
- `help_data.rs` remains the SINGLE SOURCE OF TRUTH for key descriptions (D-18, Phase 8) — supplemented by a shared JSON data file in this phase
- `pending_prompt()` in `hp41-cli/src/ui.rs` must be exhaustive — no `_ =>` catch-all, no `unreachable!()`
- `#![deny(clippy::unwrap_used)]` enforced

**Out of scope (explicit):**
- Any `hp41-core` changes — Ops, CalcState, error variants are all locked from Phases 20–24
- GUI integration (`hp41-gui/src-tauri/src/key_map.rs`, `KEY_DEFS` in `Keyboard.tsx`, modal-routing) — Phase 26
- 14-seg SVG LCD font, GUI `?`-overlay implementation — Phase 26
- Full hardware-faithful ALPHA-mode-with-prefix behavior (Σ, π, μ, etc. as ALPHA-special-chars) — v3.x
- Module-Pac emulation (Math 1 / Stat 1 / Time / Advantage) — v3.x permanent exclusion
- Test-coverage gates (≥ 95 % hp41-core line coverage; proptest sweep; Playwright E2E) — Phase 27
- Final "feature-complete HP-41CV" hard claim in README — soft claim in this phase, hard claim conditional on Phase 27 gates

</domain>

<decisions>
## Implementation Decisions

### Keyboard binding strategy (D-25.1 — D-25.6)

- **D-25.1: True HP-41 prefix-shift modal supersedes v1.x crossterm direct mapping.** Phase 25 introduces a **prefix key** to the TUI that arms a one-shot shifted-op dispatch — mirroring the HP-41C/CV/CX hardware. The v1.x convention (`C` direct-dispatch COS, `L` direct-dispatch LN, `S`/`R` open STO/RCL modals, etc.) is **deprecated**; every CLI keystroke must correspond to a real HP-41CV keyboard position.

- **D-25.2: ONE yellow prefix key (HP-41 nomenclature: `f`).** The HP-41C/CV/CX has a SINGLE shift key on the physical keyboard — orange/yellow, labelled `f` in HP nomenclature. The earlier draft of D-25.1 proposed two prefixes (`f` + `g`) confused with HP-15C / HP-12C hardware; that's wrong for HP-41 and was corrected during discussion. The CLI gets one prefix key. Status bar shows `f→` when armed. Existing TUI bindings for `f` (FmtDigits cycle for FIX/SCI/ENG) and `g` (Op::Clreg) migrate to their real HP-41CV f-shifted positions.

- **D-25.3: Full migration — every v1.x direct map is deprecated.** Existing Shift+letter direct-dispatch bindings (`C` for COS, `T` for TAN, `L` for LN, `G` for LOG, `E` for e^x, `H` for 10^x, `I` for 1/x, `W` for x², `Y` for y^x, `q` for SIN, `a` for ASIN, `c` for ACOS, `k` for ATAN, `s` for sqrt, etc.) are removed. Each Op is now reached via the HP-41CV keyboard reference card (planner: source the card from the HP-41C Owner's Manual Appendix B or Free42's keyboard.h). The v1.x letter bindings stay only where they coincide with a real HP-41 key label.

- **D-25.4: One-shot prefix lifetime — hardware-faithful.** On the real HP-41CV, pressing `f` arms the prefix; the very next op-key consumes it and dispatches the f-shifted op. Prefix auto-clears after consumption. No lock mode on the hardware. **Esc** cancels the armed prefix. Matches hp41-gui v2.1's `shiftActive` one-shot pattern (Phase 19 D-5).

- **D-25.5: ALPHA overrides Prefix in v2.2 (documented divergence from HP-41CV hardware).** In ALPHA mode, the `f` key types the letter F (or is otherwise an ALPHA-mode input character), NOT a prefix trigger. To enter an f-shifted op while in ALPHA, the user must exit ALPHA first. **Matches GUI v2.1 D-5's existing deferral.** Full hardware-faithful behavior — `f` in ALPHA mode triggering special-character lookup (Σ, π, μ, μ-superscript, …) — wanders to **v3.x** alongside the ALPHA-special-charset expansion (which needs its own table of ≈20 characters and modal/display work).

- **D-25.6: CLI ↔ GUI parity invariant.** The user explicitly requires identical prefix behavior in CLI Phase 25 and GUI Phase 26. Phase 26 must (a) keep the single-prefix model already shipped in v2.1, (b) inherit the one-shot lifetime (already done), and (c) preserve the ALPHA-overrides-Prefix D-5 deferral until v3.x resolves it. Any divergence here is a phase blocker.

### Conditional tests on the keyboard (D-25.7 — D-25.10)

- **D-25.7: Four conditional tests bound exactly to HP-41CV f-shifted positions on arithmetic keys.** Per the user's physical HP-41CV (hardware ground truth):
  - `f -` → `Op::FlagTest` for X=Y *(actually `Op::XEqY`)*
  - `f +` → `Op::XLeY` (X≤Y)
  - `f *` → `Op::XGtY` (X>Y)
  - `f /` → `Op::XEqZero` (X=0)

  These 4 are the ONLY conditional tests on the physical HP-41CV keyboard. Any other binding mechanism for these 4 violates the "hardware-faithful" Area 1 lock.

- **D-25.8: Remaining 8 conditional tests reachable only via XEQ-by-Name palette.** The other 8 conditional tests (X≠Y, X<Y, X≥Y, X≠0, X<0, X>0, X≤0, X≥0) are ROM ops in `hp41-core::ops::Op` (shipped Phase 21) but have NO physical keyboard position on the HP-41C/CV/CX. On real hardware, they're reached only via `XEQ "X<>Y"`-style invocation (or via synthetic programming). Phase 25 mirrors this exactly: the XEQ-by-Name modal (v2.1 card-reader phase, already shipped) dispatches them by mnemonic name. NO direct or shifted keyboard binding.

- **D-25.9: FN-TEST-01 "reachable from the CLI keyboard" interpreted as "reachable via keystroke sequence".** The XEQ-by-Name palette IS a keystroke sequence (open the modal, type the mnemonic, press Enter). This interpretation satisfies the success criterion without forcing non-hardware bindings.

- **D-25.10: v1.x X≥Y direct-binding is removed.** Today only X≥Y is keyboard-reachable in the v1.x CLI via an ad-hoc binding. Per D-25.3 full migration, that binding goes away; X≥Y is now only via `XEQ "X>=Y"` (or however HP-41CV mnemonic-spells it; planner: confirm the actual ROM mnemonic).

### PendingInput modal architecture (D-25.11 — D-25.14)

- **D-25.11: Hybrid struct-variants — group ops with identical state shape, keep specialty ops unique.** PendingInput grows from the current 11 variants to ~16–18 (NOT 30+):
  - **Group variants (struct):**
    - `FlagPrompt { kind: FlagTestKind, ind: bool, acc: String }` — covers SF/CF/FS?/FC?/FS?C/FC?C × direct/IND (12 logical ops collapsed via `FlagTestKind` reuse from Phase 21)
    - `RegisterPrompt { op: RegisterOpKind, ind: bool, acc: String }` — covers STO/RCL/STO+-*/VIEW/ARCL/ASTO/ISG/DSE × direct/IND (≈20 logical ops collapsed via a new `RegisterOpKind` enum local to hp41-cli)
  - **Specialty variants (unique, retain v1.x style):**
    - `ClpLabel(String)` — text-input modal for `CLP "name"`
    - `DelCount(String)` — 3-digit numeric for `DEL nnn`
    - `TonePrompt(String)` — single-digit 0–9 for `TONE n`
    - Existing `AssignKey` / `AssignLabel(char, String)` / `ConfirmLoad(usize)` / `FmtDigits(DisplayMode)` / `PrintModal` / `HexModal(String)` are preserved (no change)
  - **Rationale:** Rust best practice — "make illegal states unrepresentable". A vollgenerische `NumericPrompt(...)` would force TONE (single-digit, no IND) and CLP-Label (text input) into a shared shape that misrepresents them. Struct-variants with reused hp41-core enums (FlagTestKind, StoArithKind) keep semantic identity while collapsing parallel-state variants.

- **D-25.12: IND modifier as Boolean field in modal state, toggle-bar mid-input.** Hardware-faithful flow: User presses op key (e.g. STO) → modal opens with `ind = false`, status shows `STO [__]` → User presses the IND key (HP-41CV-specific position; planner: confirm IND key from reference card — likely f-XEQ or similar) → modal toggles to `ind = true`, status updates to `STO IND [__]` → User types `05` → dispatch picks `Op::StoInd(5)` vs `Op::Sto(5)` based on the flag. Pressing IND again during the same modal toggles back. Dispatch decision is **single-point at end** of digit accumulation: `if state.ind { Op::*Ind(n) } else { Op::*(n) }`. NO separate `Op::StoIndPrompt`-style PendingInput variants — IND is purely a struct field.

- **D-25.13: Reuse hp41-core enums; do NOT define parallel TUI-local discriminator enums.** `FlagTestKind` is shipped (Phase 21, used by `Op::FlagTest`). `StoArithKind` is shipped (Phase 9, used by `Op::StoArith` + `Op::StoArithStack`). The new `RegisterOpKind` is a TUI-local enum only because hp41-core has no equivalent (different ops like RCL/VIEW/ARCL/ASTO need a common discriminator). Document this asymmetry in the plan.

- **D-25.14: `pending_prompt()` stays exhaustive (~18 match arms).** No `_ =>` catch-all, no `unreachable!()`. Each match arm uses struct-pattern-matching for the group variants (`PendingInput::FlagPrompt { kind, ind, acc }`) and explicit destructuring for specialty variants. The exhaustive-match compile-check is the runtime guarantee that no `PendingInput` slips through silently (per FN-CLI-04).

### Function matrix + data sharing (D-25.15 — D-25.17)

- **D-25.15: Function-matrix source = hand-curated + CI-parity-check against Op enum.** `docs/hp41cv-function-matrix.md` is hand-edited from HP-41C Owner's Manual Appendix B + Free42 source cross-check. Columns: `Op | Display Name | Category | Status | Phase | Notes`. Status uses three values: `✓ v2.x` / `⏳ v3.x module` / `— N/A`. v3.x-deferred ops (Math Pac, Stat Pac, Time Pac, Advantage Pac functions) appear in the matrix with `⏳`-status — they will NOT have Op variants in hp41-core during v2.x. A CI test in `hp41-core/tests/` reads the JSON source (see D-25.16), iterates over every `Op::` variant, and asserts:
  - Every `✓ v2.x` entry in the matrix has a corresponding `Op::` variant
  - Every `Op::` variant has a matching matrix entry (no Ops missing from docs)

  Bidirectional drift-prevention.

- **D-25.16: Shared JSON data source for help_data.rs + Phase-26 TS-overlay + matrix.md.** Single canonical file: `docs/hp41cv-functions.json` (hand-edited; committed). Schema per entry:
  ```json
  {
    "op_variant": "Pi",          // hp41-core Op::-name (PascalCase)
    "display_name": "PI",         // HP-41 mnemonic as shown on display
    "category": "Math",           // help-overlay section
    "status": "implemented",      // implemented | deferred-v3 | na
    "phase": "20",                // GSD phase that shipped it (or null for v3.x)
    "key_path": "f-pi",           // CLI keystroke (or null if XEQ-by-Name only)
    "description": "Push π onto X",
    "divergences": ["10-digit precision per Phase 20 D-09"]  // optional, free-form
  }
  ```

  **Pipeline:**
  - **`hp41-cli/src/help_data.rs`** — `pub const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");`. A `pub fn help_entries() -> &'static [HelpEntry]` lazy-parses via `serde_json::from_str` (cached via `once_cell::Lazy` or equivalent). Compile-time JSON-embedding; zero runtime file I/O for the TUI overlay.
  - **`hp41-gui/src/help_data.ts`** (Phase 26) — vite's JSON-import: `import functions from '../../docs/hp41cv-functions.json'` with a TS-type assertion. Same source, zero duplication.
  - **`docs/hp41cv-function-matrix.md`** — generated via `just docs-matrix` (justfile recipe). The Markdown is committed (human-readable view), but the JSON is the canonical edit-target. A CI test verifies the committed `.md` matches what `just docs-matrix` would regenerate (drift catch).
  - **NO build.rs codegen.** `include_str!` + serde at-runtime is sufficient; build.rs adds complexity without benefit for this use case.

- **D-25.17: README makes a soft "feature-complete HP-41CV" claim with explicit Divergences link.** Phase 25 updates `README.md` with wording roughly: *"Implements the full HP-41CV ROM built-in function set (≈130 ops) with documented divergences. See [HP-41CV function matrix](docs/hp41cv-function-matrix.md) for status per op and known hardware divergences."* The Divergences section enumerates: PI 10-digit precision, FACT effective cap ≤ 27, SIGN-on-ALPHA = 0, CLP boundary = next LBL (no END/.END. markers), PACK no-op, POSA single-char, AROT silent-truncate non-integer N. A potential **hard claim** (drop the "documented divergences" caveat) is **deferred to Phase 27** — conditional on whether numerical-accuracy and coverage gates pass cleanly.

### Cross-cutting invariants (carried forward, NOT re-decided)

- **D-18 (Phase 8) — `help_data.rs` is the SINGLE SOURCE OF TRUTH for key descriptions.** Preserved; supplemented in Phase 25 by `docs/hp41cv-functions.json` (which `help_data.rs` reads).
- **D-22.21 / D-23.12 (4-place Op-variant landing rule).** Phase 25 INVERTS this: this phase wires the ALREADY-landed variants into the CLI keyboard + modals — no new Op variants. The compile-time exhaustive matches in `dispatch()` and `execute_op()` are unchanged.
- **D-22.11.1 (no raw indexing — `.get().ok_or(InvalidOp)?` pattern).** Applies to any new TUI-side helpers that read registers (though Phase 25 likely doesn't add new register-read paths — it routes via existing hp41-core dispatch).
- **D-23.14 / zero-panic.** No `.unwrap()` in production code. All `pending_prompt()` and dispatch paths use `?` or `.expect("reason")`. JSON parsing on TUI startup uses `.expect("hp41cv-functions.json is malformed — fix the JSON")` because a malformed canonical-data file is a hard-build-blocker by design.
- **PRGM-mode duality (Phase 9+).** Modal-bearing ops (STO/RCL/FIX/etc.) insert as program steps in PRGM mode rather than executing. Phase 25 extends this pattern to all new modal-bearing ops (SF/CF/VIEW/TONE/etc.) — pattern is established, planner mirrors precedent.
- **v2.1 XEQ-by-Name modal — already shipped.** Phase 25 routes the 8 non-keyboard conditional tests through this existing modal. Planner extends the resolver to handle the new Op mnemonics.
- **CLI ↔ GUI parity (D-25.6).** Phase 26 MUST mirror the Phase 25 prefix-modal model exactly. No CLI-only divergences; no GUI-only divergences.

### Claude's Discretion

- **Exact IND key position on the HP-41CV reference card.** Planner / researcher must source the HP-41C Owner's Manual Appendix B (or Free42's keyboard.h) to confirm which physical key holds the IND modifier. Common candidates: f-XEQ, or a dedicated key labeled "IND" on later HP-41CX revisions. The TUI maps that position to a single character or escape sequence — D-25.12 leaves the actual char open.
- **`RegisterOpKind` enum membership.** Planner finalizes the exact list of ops collapsed into `RegisterPrompt` — at minimum: Sto / Rcl / StoAdd / StoSub / StoMul / StoDiv / View / Arcl / Asto / Isg / Dse. May extend or shrink based on whether ISG/DSE have a meaningfully different state-shape (they have skip-next-step semantics in `run_loop`, but the input flow is identical 2-digit register entry).
- **JSON entry count vs. Op variant count.** The function matrix targets ≥130 entries (HP-41CV ROM full set). hp41-core's Op enum has ~190 variants today (post-Phase 24). The matrix can have FEWER entries than Op variants if some Ops correspond to one matrix row (e.g., `Op::Sto(u8)` is one matrix entry "STO", not 100 entries per register). Planner builds the row-vs-variant mapping logic in the CI parity test.
- **Migration of v1.x letter bindings that happen to coincide with HP-41 labels.** Some v1.x letters might naturally correspond to HP-41 primary labels (e.g., `s` for sqrt if sqrt is labelled "√x" at row-X-col-Y where the natural mnemonic is "s"). Planner is allowed to keep v1.x bindings IF they match the HP-41CV reference card; otherwise they go away per D-25.3.
- **Categorization of `help_data.rs` after JSON migration.** The JSON has a `category` field. Planner decides whether existing `=== Stack ===` / `=== Math ===` / etc. category headers in help_data.rs are derived from the JSON or hand-grouped. Recommended: derive from JSON for DRY.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level
- `.planning/PROJECT.md` — Project goals, build sequence, evolution rules
- `.planning/REQUIREMENTS.md` §FN-TEST-01, §FN-CLI-01..04, §FN-DOC-01..04 — locked v2.2 requirements for Phase 25
- `.planning/ROADMAP.md` Phase 25 section — phase goal, success criteria, cross-cutting constraints
- `.planning/STATE.md` — current milestone state (v2.2, Phases 20–24 shipped)
- `CLAUDE.md` — settled architecture decisions, critical implementation traps, key file index

### Prior phase context (carry-forward)
- `.planning/phases/24-indirect-addressing/24-CONTEXT.md` — D-24.1..D-24.9 (resolve_indirect, IND variants); IND modifier semantics
- `.planning/phases/23-alpha-operations/23-CONTEXT.md` — D-23.4 (text_regs sidecar), D-23.12 (4-place landing rule, inverted here)
- `.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md` — D-22.21 (4-place landing rule origin), D-22.11.1 (regs[] bounds via `.get()`)
- `.planning/phases/21-flags-display-control-and-sound/` (if exists) — FlagTestKind enum shape, run_loop skip semantics
- Phase 8 `.planning/phases/08-tech-debt-cleanup/*-CONTEXT.md` (if exists) — D-18 help_data.rs single-source-of-truth invariant
- Phase 19 (no GSD directory; documented in MILESTONES.md v2.1) — GUI one-shot SHIFT pattern (D-5 ALPHA-overrides-SHIFT deferral)

### HP-41CV hardware references (external — planner sources)
- HP-41C Owner's Manual Appendix B — keyboard reference card (canonical key position → op mnemonic mapping)
- Free42 source `keyboard.h` (or equivalent) — alternate authoritative source for HP-41CV keystroke mapping
- HP-41CV Quick Reference Card — abbreviated keyboard layout for cross-checking

### Codebase files (key wiring targets)
- `hp41-cli/src/keys.rs` — `key_to_op()` + `KEY_REF_TABLE` (this phase rewrites significantly per D-25.3)
- `hp41-cli/src/app.rs` — `PendingInput` enum (this phase adds Hybrid struct-variants per D-25.11) + modal-input dispatch
- `hp41-cli/src/ui.rs` — `pending_prompt()` exhaustive match (this phase extends per D-25.14)
- `hp41-cli/src/help_data.rs` — current static const; this phase migrates to JSON-loaded per D-25.16
- `hp41-core/src/ops/mod.rs` — Op enum (READ-ONLY in this phase; source for CI parity check)
- `docs/hp41cv-function-matrix.md` — NEW file in this phase; output of `just docs-matrix`
- `docs/hp41cv-functions.json` — NEW file in this phase; canonical data source
- `README.md` — soft-claim update per D-25.17
- `CLAUDE.md` — v2.2 "Settled Architecture Decisions" additions per FN-DOC-02

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`PendingInput` enum pattern** (`hp41-cli/src/app.rs:23`) — current 11-variant enum; the Hybrid struct-variants of D-25.11 follow the existing precedent (StoAdd/Sub/Mul/Div are already grouped with shared shape).
- **`pending_prompt()` exhaustive match** (`hp41-cli/src/ui.rs:238`) — pattern for status-bar prompt rendering; trivially extended to new struct-variants with destructuring patterns.
- **`HELP_DATA` static const** (`hp41-cli/src/help_data.rs`) — 33 entries today; migration path: define a struct `HelpEntry { key, op, desc, category }` that mirrors the JSON schema, replace the const with a lazy-loaded `Vec<HelpEntry>`, keep the 13-category grouping derived from the JSON's `category` field.
- **`KEY_REF_TABLE`** (`hp41-cli/src/keys.rs:91`) — right-panel discoverability table (key → description); regenerated per D-25.16 from the same JSON.
- **XEQ-by-Name modal** (v2.1, shipped) — entry point for the 8 non-keyboard conditional tests per D-25.8.
- **`Op::FlagTest { kind: FlagTestKind, flag: u8 }`** (`hp41-core::ops`, Phase 21) — struct-variant precedent for the new `PendingInput::FlagPrompt { kind, ind, acc }`.
- **`Op::StoArith(u8, StoArithKind)`** (`hp41-core::ops`, Phase 9) — kind-reuse precedent; `PendingInput::RegisterPrompt { op: RegisterOpKind, ind, acc }` mirrors this.
- **`Op::StoInd(u8)` / `Op::RclInd(u8)` etc.** (`hp41-core::ops`, Phase 24) — already exist; this phase wires their keyboard path per D-25.12.

### Established Patterns

- **Modal entry → digit accumulation → dispatch** — every modal-bearing op (STO/RCL/FmtDigits/etc.) follows this 3-step pattern. Phase 25 extends with a 4th optional step (IND toggle) per D-25.12.
- **PRGM-mode duality** — modal-bearing ops insert as program steps when PRGM-mode is active. Established Phase 9+; Phase 25 extends consistently.
- **Compile-time exhaustive matches** — every Op enum match in `dispatch()`, `execute_op()`, `prgm_display.rs` uses no catch-all; the same discipline applies to `pending_prompt()` per D-25.14.
- **Single source of truth via `help_data.rs`** (D-18, Phase 8) — preserved structurally; supplemented by JSON per D-25.16. Drift between JSON and Op enum is CI-blocked per D-25.15.
- **`include_str!` for compile-time data embedding** — Rust idiomatic for canonical data files; zero runtime I/O in TUI overlay path.

### Integration Points

- **`hp41-cli/src/app.rs::handle_key()`** — main key dispatch; this phase rewrites the f-prefix arming logic (state field `pub shift_armed: bool` on `App`, mirroring GUI v2.1's `shiftActive`). One-shot consumption per D-25.4.
- **`hp41-cli/src/ui.rs::render_status()`** — extends to show `f→` indicator when `shift_armed = true` (consistent with GUI v2.1's pressedKey indicator).
- **`hp41-cli/src/app.rs::handle_pending_input()`** — existing function; extended for the new Hybrid variants with IND-toggle handling (struct-mutation, not state-replace).
- **`hp41-cli/src/cards.rs::xeq_by_name()`** (or equivalent, shipped v2.1) — extended to resolve the 8 non-keyboard conditional-test mnemonics ("X<>Y", "X<Y", "X>=Y", "X#0", "X<0", "X<=0", "X>0", "X>=0" — planner confirms exact mnemonics from ROM reference).
- **Justfile** — new recipe `just docs-matrix` invokes a small Rust binary or shell script that reads `docs/hp41cv-functions.json` and emits `docs/hp41cv-function-matrix.md` in the documented Op|Display|Category|Status|Phase|Notes column order.
- **`hp41-cli/Cargo.toml`** — likely adds `serde`, `serde_json`, `once_cell` (or use `std::sync::OnceLock`) as new dev / runtime deps for the JSON-loading path. Document this in the plan.

</code_context>

<specifics>
## Specific Ideas

- **Hardware ground truth from the user's physical HP-41CV** (gathered during discussion):
  - `f -` → X=Y
  - `f +` → X≤Y
  - `f *` → X>Y
  - `f /` → X=0
  These 4 conditional tests are bound EXACTLY as on the user's device. Planner uses this as the anchor when sourcing the rest of the keyboard reference card.

- **GUI v2.1 SHIFT model as the parity reference.** The user wants CLI Phase 25 to behave identically to the GUI v2.1 SHIFT state machine — same one-shot lifetime, same ALPHA-overrides-SHIFT deferral (D-5), same `Esc` cancel. Phase 26 (GUI) inherits the locked-in behavior without changes; the v2.1 GUI implementation is the de-facto reference design.

- **"Es muss einfach genau nach HP-41CV funktionieren"** (user, verbatim, on prefix lifetime) — this is the meta-principle for Phase 25 conflict resolution. When two implementation choices have equal Rust-idiomatic value, pick the one that mirrors the real HP-41CV hardware behavior. Diverge only with explicit justification (e.g. D-25.5's deferral of ALPHA-mode-with-prefix).

</specifics>

<deferred>
## Deferred Ideas

- **Full hardware-faithful ALPHA-mode + prefix behavior** — currently deferred to v3.x. Requires a Special-Character Table (≈20 chars: Σ, π, μ, μ-superscript, etc.), display-layer adjustments to render these, AND modal-routing changes. Both CLI and GUI inherit the v2.2 deferral; v3.x bundles the change in both.
- **README "feature-complete HP-41CV" HARD claim** — potentially elevated in Phase 27 if numerical-accuracy and coverage gates pass cleanly. v2.2 ships with soft claim per D-25.17.
- **Module-Pac emulation** (Math 1 / Stat 1 / Time / Advantage) — permanent v2.x exclusion per `REQUIREMENTS.md` boundary. The function matrix lists these as `⏳ v3.x module` for discoverability, but no implementation in v2.x.
- **Two-prefix support (`f` + `g`)** — explicitly rejected for HP-41CV (hardware has only one). May resurface only if the project pivots to emulate HP-15C / HP-12C in a future milestone (not in any current roadmap).
- **Test-coverage gate increase to ≥95 %** — Phase 27 territory (FN-QUAL-01).
- **Playwright GUI E2E test harness** — Phase 27 / Phase 26-Polish territory (FN-QUAL-05).
- **Proptest for indirect-addressing resolver** — Phase 27 (FN-QUAL-04).
- **Numerical-accuracy suite expansion** for new math ops (PI / P→R / R→P / RND / FRC / MOD / FACT) — Phase 27 (FN-QUAL-02).
- **Flag-semantics property tests** across all 56 user flags — Phase 27 (FN-QUAL-03).

</deferred>

---

*Phase: 25-cli-integration-and-documentation*
*Context gathered: 2026-05-14*
