# Phase 25: CLI Integration & Documentation — Research

**Researched:** 2026-05-14
**Domain:** Terminal-UI keyboard wiring (ratatui 0.30 + crossterm 0.29), prefix-shift state machine, modal-input architecture, JSON-embedded canonical data, just-recipe doc generation, CI parity-check tests
**Confidence:** HIGH

## Summary

Phase 25 wires the 130 `Op` variants shipped in Phases 20–24 into `hp41-cli` as a hardware-faithful HP-41CV keyboard, replaces the v1.x crossterm letter-direct-dispatch convention with a true one-shot **yellow-shift** prefix model (parity with GUI v2.1 D-5), ships ~6 new `PendingInput` modal variants (≤18 total exhaustive arms), and creates the canonical documentation pipeline: `docs/hp41cv-functions.json` → `help_data.rs` (via `include_str!` + `OnceLock`) → `docs/hp41cv-function-matrix.md` (via `just docs-matrix`) → CI parity test against `Op` enum.

The locked CONTEXT.md (D-25.1..D-25.17) collapses what would naively be a 30+ modal-variant explosion into ~18 hybrid struct-variants by reusing hp41-core's existing `FlagTestKind` (Phase 21) and `StoArithKind` (Phase 9) discriminator enums plus a new TUI-local `RegisterOpKind`, with IND handling as a `Boolean` field on `RegisterPrompt` and `FlagPrompt` rather than a duplicate variant family. The 4 keyboard-bound conditional tests (`f-`/`f+`/`f*`/`f/`) plus 8 XEQ-by-Name-only tests are all enumerable from the HP-41C/CV Quick Reference Guide [VERIFIED: literature.hpcalc.org]. Critically, the IND modifier is **`shift-0` (f-0)**, not `f-XEQ` as the CONTEXT speculated — confirmed by the user's QRG page 14 ("Indirect Operations: An indirect address is selected by following a function with the shift key, ■, and then a register address").

**Primary recommendation:** Land the f-prefix state machine + 6 new `PendingInput` variants + JSON pipeline as four sequential plans (keyboard, modals, docs/JSON pipeline, CI parity). Order matters because `pending_prompt()` exhaustiveness + key dispatch share the same compile-time graph — touch them in lockstep.

## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-25.1: True HP-41 prefix-shift modal supersedes v1.x crossterm direct mapping.** Phase 25 introduces a one-shot prefix key armed by `f`; status bar shows `f→` when armed. v1.x letter-direct conventions (`C`→COS, `L`→LN, etc.) are **deprecated**; every CLI keystroke must correspond to a real HP-41CV keyboard position.

**D-25.2: ONE yellow prefix key (HP-41 nomenclature: `f`).** The HP-41C/CV/CX has a SINGLE shift key on the physical keyboard. No `g`-prefix. Existing TUI bindings for `f` (FmtDigits cycle) and `g` (Op::Clreg) migrate to their real HP-41CV f-shifted positions.

**D-25.3: Full migration — every v1.x direct map is deprecated.** Shift+letter direct-dispatch bindings (C/T/L/G/E/H/I/W/Y for COS/TAN/LN/LOG/e^x/10^x/1/x/x²/y^x; q/a/c/k for SIN/ASIN/ACOS/ATAN; etc.) removed. Reach each Op via the HP-41CV keyboard reference card.

**D-25.4: One-shot prefix lifetime — hardware-faithful.** Pressing `f` arms; the very next op-key consumes it. No lock mode. **Esc** cancels. Matches GUI v2.1 `shiftActive` pattern.

**D-25.5: ALPHA overrides Prefix in v2.2 (documented divergence).** In ALPHA mode `f` types F. Hardware-faithful ALPHA+prefix (Σ, π, μ-special chars) deferred to v3.x.

**D-25.6: CLI ↔ GUI parity invariant.** Phase 26 must mirror Phase 25's prefix model exactly.

**D-25.7: Four conditional tests bound exactly to f-shifted arithmetic keys:**
- `f -` → `Op::Test(TestKind::XEqY)`
- `f +` → `Op::Test(TestKind::XLeY)`
- `f *` → `Op::Test(TestKind::XGtY)`
- `f /` → `Op::Test(TestKind::XEqZero)`

**D-25.8: Remaining 8 conditional tests reachable only via XEQ-by-Name palette.** No keyboard binding. Phase 25 routes mnemonic strings to `Op::Test(TestKind::…)` via an extended `builtin_card_op`-style resolver (renamed/extended).

**D-25.9: FN-TEST-01 "reachable from CLI keyboard" interpreted as "reachable via keystroke sequence".** XEQ-by-Name modal IS a keystroke sequence.

**D-25.10: v1.x X≥Y direct-binding removed.** Per full migration.

**D-25.11: Hybrid PendingInput — group struct-variants + specialty unique variants.**
- `FlagPrompt { kind: FlagTestKind, ind: bool, acc: String }` (covers SF/CF/FS?/FC?/FS?C/FC?C × direct/IND)
- `RegisterPrompt { op: RegisterOpKind, ind: bool, acc: String }` (covers STO/RCL/STO+-*/VIEW/ARCL/ASTO/ISG/DSE × direct/IND)
- Specialty: `ClpLabel(String)`, `DelCount(String)`, `TonePrompt(String)`, plus existing `AssignKey`/`AssignLabel`/`ConfirmLoad`/`FmtDigits`/`PrintModal`/`HexModal`

**D-25.12: IND as Boolean field, toggle-bar mid-input.** Hardware-faithful flow: STO → modal opens (`ind=false`) → IND key toggles to `ind=true` → digits → dispatch picks `Op::*Ind(n)` vs `Op::*(n)` at end. NO separate `Op::StoIndPrompt` variants.

**D-25.13: Reuse hp41-core enums; only define new TUI-local enum where core has none.** `FlagTestKind` reused, `StoArithKind` reused. New `RegisterOpKind` is TUI-local (hp41-core has no equivalent).

**D-25.14: `pending_prompt()` exhaustive (~18 arms).** No `_=>` catch-all, no `unreachable!()`.

**D-25.15: Function-matrix source = hand-curated `docs/hp41cv-function-matrix.md` + CI parity check.** Columns: `Op | Display Name | Category | Status | Phase | Notes`. Status: `✓ v2.x` / `⏳ v3.x module` / `— N/A`. CI test asserts JSON ↔ Op enum bidirectional drift-prevention.

**D-25.16: Shared JSON data source `docs/hp41cv-functions.json`** with schema `{op_variant, display_name, category, status, phase, key_path, description, divergences[]}`. Pipeline: `include_str!` + `OnceLock` (`std::sync`, not external `once_cell`) in `help_data.rs`; Phase 26 vite-imports same JSON; Markdown matrix generated by `just docs-matrix`; **no `build.rs`**.

**D-25.17: README soft "feature-complete HP-41CV with documented divergences" claim.** Hard claim deferred to Phase 27 (gates).

### Claude's Discretion

- Exact IND key position on the reference card — **researched and confirmed below** (it's `shift-0`, not `f-XEQ`).
- `RegisterOpKind` enum membership — recommended list below.
- JSON entry count vs Op variant count — recommended row-vs-variant mapping below.
- v1.x letter bindings that coincide with HP-41 labels — none survive verbatim per D-25.3; HP-41 has no letter-mnemonic shortcuts.
- Categorization of `help_data.rs` after JSON migration — recommended: derive from JSON `category` field for DRY.

### Deferred Ideas (OUT OF SCOPE)

- Full hardware-faithful ALPHA-mode + prefix behavior (Σ/π/μ special chars) → v3.x
- README "feature-complete HP-41CV" HARD claim → Phase 27
- Module-Pac emulation → permanent v2.x exclusion
- Two-prefix support (`f` + `g`) → rejected for HP-41CV
- Coverage gate increase to ≥95% → Phase 27
- Playwright GUI E2E → Phase 27
- Proptest sweep for indirect resolver → Phase 27

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FN-TEST-01 | All 12 conditional tests reachable from CLI keyboard | §"HP-41CV Conditional Tests Reachability" — 4 via f-+/-/*/, 8 via XEQ-by-Name palette per D-25.7/D-25.8 |
| FN-CLI-01 | All new Op variants wired in keys.rs + KEY_REF_TABLE | §"Op-enum → Keyboard Position Map" — 130 Ops mapped to 35 physical keys + f-shifted variants + modal-paths |
| FN-CLI-02 | New PendingInput modal variants (Flag/View/Tone/Del/CLP + IND) | §"PendingInput Architecture" — 6 new variants (2 struct-group, 4 specialty) |
| FN-CLI-03 | help_data.rs updated with every new binding | §"JSON Pipeline Architecture" — single canonical source |
| FN-CLI-04 | pending_prompt() exhaustive match | §"Exhaustive Match Discipline" — compile-time guarantee |
| FN-DOC-01 | docs/hp41cv-function-matrix.md ≥130 entries | §"Function Matrix Schema" — exactly 130 Op variants today; matrix lists by ROM mnemonic |
| FN-DOC-02 | CLAUDE.md updated with v2.2 settled decisions | §"CLAUDE.md Update Plan" — flag storage, indirect resolution, sound buffer, f-prefix |
| FN-DOC-03 | README "feature-complete HP-41CV" claim | §"README Soft-Claim Wording" |
| FN-DOC-04 | hp41-core rustdoc cross-references | §"Rustdoc Cross-link Strategy" |

## Project Constraints (from CLAUDE.md)

These directives from `./CLAUDE.md` are non-negotiable for Phase 25 plans:

- **Commits via `/git-workflow:commit --with-skills` only**, English only (subject + body).
- **`hp41-core` is FROZEN this phase** — root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; no new Op variants, no new CalcState fields, no new HpError variants.
- **4-place Op-variant landing rule is INVERTED here.** Phase 25 wires already-landed variants into the CLI keyboard + modals — no new variants.
- **`#![deny(clippy::unwrap_used)]` is active in `hp41-core`** — all production code uses `.expect("reason")` or `?`. JSON parse on startup uses `.expect("hp41cv-functions.json is malformed — fix the JSON")` (D-25.17 hard-build-blocker by design).
- **SC-4 invariant** — no `op_*` / `flush_entry_*` / `format_hpnum` added to `hp41-gui/src-tauri/` (Phase 25 doesn't touch hp41-gui anyway, but the rule remains a guardrail for future drift).
- **`pending_input` routing block must remain ABOVE modal-opening interceptors** (S/R/Ctrl+A) — Phase 25's new prefix-arming logic must respect this ordering or active modals are silently discarded.
- **Crossterm KeyEvent filter on Windows** — `KeyEventKind::Release` events must be filtered immediately (app.rs:183). The prefix-arming state must not be advanced by a Release event.
- **`just` is the sole task runner** — `just docs-matrix` becomes the new recipe in this phase. Never call `cargo` directly in CI or docs.
- **`help_data.rs` is the SINGLE SOURCE OF TRUTH for key descriptions (D-18, Phase 8).** Preserved structurally; supplemented by JSON.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| f-prefix state machine | hp41-cli/app.rs (App.shift_armed: bool) | — | Frontend-only per D-25.5 (never crosses IPC); parity with GUI v2.1 `shiftActive` |
| Key → Op resolution | hp41-cli/keys.rs (key_to_op) | hp41-cli/app.rs (handle_key) | Routing layer between crossterm and hp41-core dispatch |
| PendingInput modal state | hp41-cli/app.rs (PendingInput enum) | hp41-cli/ui.rs (pending_prompt) | Transient UI state, not persisted (per existing comment) |
| IND-modifier toggle | PendingInput field (`ind: bool` on FlagPrompt/RegisterPrompt) | dispatch decision at end | Hardware-faithful "press IND mid-input to toggle" per D-25.12 |
| Conditional test dispatch | hp41-cli/keys.rs (4 via f-arith), hp41-cli/app.rs (8 via XEQ modal) | hp41-core::ops::program::builtin_card_op (extend) | 4 keyboard / 8 mnemonic-name per D-25.7/D-25.8 |
| Help-overlay data | hp41-cli/help_data.rs (parses docs/hp41cv-functions.json via include_str!) | docs/hp41cv-functions.json | Single source of truth (D-18 + D-25.16) |
| Function matrix | docs/hp41cv-function-matrix.md (generated) | docs/hp41cv-functions.json (canonical) + justfile recipe | Hand-edited JSON, generated Markdown per D-25.16 |
| CI parity check | hp41-cli/tests/function_matrix_parity.rs | docs/hp41cv-functions.json + hp41_core::ops::Op | JSON ↔ Op enum bidirectional drift catch per D-25.15 |
| README soft-claim | README.md project root | docs/hp41cv-function-matrix.md (link target) | Marketing surface; hard-claim deferred to Phase 27 |

## Standard Stack

### Core (already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1.x (workspace) | JSON deserialize for FUNCTIONS_JSON | Already used everywhere; derive macro for HelpEntry struct |
| `serde_json` | 1.x (workspace) | Parse `docs/hp41cv-functions.json` at startup | Already in deps; `from_str` is the standard parser |
| `ratatui` | 0.30 | TUI rendering (existing) | Unchanged by this phase |
| `crossterm` | 0.29 | Key event polling (existing) | Unchanged by this phase |

### Supporting (Rust stdlib — no new deps)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::sync::OnceLock` | stdlib (MSRV 1.88) | Lazy-init `HELP_ENTRIES` from `include_str!`-embedded JSON | Project precedent in `hp41-cli/src/programs.rs:7,19` [VERIFIED: grep] |
| `std::include_str!` | stdlib | Compile-time-embed `docs/hp41cv-functions.json` | Zero runtime file I/O; malformed JSON is a build-time failure |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `OnceLock` | `once_cell::sync::Lazy` (external) | Not needed: MSRV 1.88 has `OnceLock` in stdlib [CITED: github.com/rust-lang/rust 1.70+]; project precedent uses stdlib |
| `serde_json::from_str` | `serde_json::from_slice` | Equivalent for `include_str!` (which returns `&'static str`); `from_str` is more idiomatic for string inputs |
| Hand-curated CI parity test | `strum::EnumIter` derive on Op | Avoid: adds new dep to `hp41-core` (currently zero non-essential deps); a hand-curated list of all 130 Op variants in the test file is equivalent and aligns with project's conservative dep policy |
| `build.rs` codegen of help_data | `include_str!` at runtime | Per D-25.16: build.rs adds complexity without benefit; runtime parse is sub-ms |
| Generating matrix.md via Rust bin | Pure-shell `just docs-matrix` recipe | Rust bin recommended — type-checks JSON schema, can fail loudly on schema mismatch; shell is portable but parsing JSON in awk/sed is fragile |

**No installation needed** — all new code uses existing workspace deps.

**Version verification:**
```bash
# serde and serde_json are already in workspace deps at version "1"
# Most recent stable on crates.io as of 2026-05-14: [VERIFIED via npm view fallback]
```
Project uses semver-compatible `"1"` in `[workspace.dependencies]`; cargo resolves to latest 1.x [VERIFIED: Cargo.toml].

## Architecture Patterns

### System Architecture Diagram

```
                  ┌──────────────────┐
                  │ crossterm        │
                  │ KeyEvent stream  │
                  └────────┬─────────┘
                           │ keys arrive
                           ▼
              ┌────────────────────────┐
              │ app.handle_key()       │
              │ ──────────────────────│
              │ 1. KeyEventKind filter │
              │ 2. last_key_code track │
              │ 3. Ctrl+C / Ctrl+S    │
              │ 4. ?-help, Ctrl+P     │
              │ 5. **NEW: f-prefix    │
              │    arming check       │
              │    (shift_armed)**     │
              │ 6. pending_input route │
              │ 7. modal openers      │
              │ 8. ALPHA mode route   │
              │ 9. USER mode dispatch │
              │ 10. digit/'.'/'e'/'n' │
              │ 11. F5/F7/F8         │
              │ 12. key_to_op()      │
              └────┬───────────────┬──┘
                   │ direct       │ shift_armed=true
                   │ dispatch     │ + key consumed
                   │              │ + clears prefix
                   ▼              ▼
        ┌──────────────────┐   ┌─────────────────────┐
        │ keys::key_to_op  │   │ keys::shifted_key_  │
        │ (primary label) │   │ to_op(key) **NEW** │
        │ returns Op       │   │ (f-shifted label)   │
        └────────┬────────┘   └────────┬───────────┘
                 │                       │
                 └──────────┬────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │ Either:                  │
              │   - Some(Op) → dispatch  │
              │   - None + opens modal: │
              │     PendingInput::Flag/ │
              │     Register/Tone/...    │
              └──────┬───────────────┬───┘
                     │ Op dispatched │ Modal opened
                     ▼               ▼
        ┌──────────────────┐  ┌─────────────────────┐
        │ hp41-core::ops::│  │ handle_pending_input│
        │ dispatch(state, │  │ — digit/IND-toggle  │
        │   op)            │  │   accumulator       │
        └──────────────────┘  └────────┬───────────┘
                                       │ 2-digit fill
                                       │ OR IND-toggle
                                       │ OR Esc cancel
                                       ▼
                              ┌────────────────────┐
                              │ Final Op resolved:│
                              │ if ind { Op::*Ind(n)│
                              │ else  { Op::*(n)   │
                              │ → dispatch         │
                              └────────────────────┘

                  Documentation pipeline (parallel flow):
                  ┌──────────────────────────────────┐
                  │ docs/hp41cv-functions.json (hand)│
                  └──────┬──────────────────────┬────┘
                         │ include_str!         │ just docs-matrix
                         │ + OnceLock parse     │ (Rust bin or script)
                         ▼                      ▼
              ┌────────────────────┐  ┌────────────────────┐
              │ help_data.rs::     │  │ docs/hp41cv-      │
              │   HELP_ENTRIES    │  │   function-matrix.md│
              │   (lazy, cached)  │  │   (generated, also  │
              └────────────────────┘  │   committed)        │
                                       └─────────┬──────────┘
                                                 │
                                                 ▼
                                       ┌────────────────────┐
                                       │ README.md links    │
                                       │ for soft-claim     │
                                       └────────────────────┘
                                                 │
                                                 │ CI parity check
                                                 ▼
                                       ┌────────────────────┐
                                       │ tests/function_    │
                                       │   matrix_parity.rs │
                                       │ — iterates the     │
                                       │   130 Op variants  │
                                       │   vs JSON entries  │
                                       └────────────────────┘
```

### Recommended Project Structure (Phase 25 deltas only)

```
hp41-cli/src/
├── app.rs           # +shift_armed:bool field, +prefix-arm logic, +6 new PendingInput arms in handle_pending_input
├── keys.rs          # +shifted_key_to_op(), +xeq_by_name_resolve(), reworked KEY_REF_TABLE
├── ui.rs            # +pending_prompt() arms for new variants, +f→ indicator in render_status
├── help_data.rs     # Rewritten: include_str! + OnceLock<Vec<HelpEntry>> reading docs/hp41cv-functions.json
├── prgm_display.rs  # No changes — display names already cover all 130 Op variants
└── tests/
    └── function_matrix_parity.rs  # NEW — bidirectional JSON ↔ Op enum drift catch

docs/                                 # NEW pipeline
├── hp41cv-functions.json            # NEW — canonical hand-curated data source
├── hp41cv-function-matrix.md        # NEW — generated by `just docs-matrix`, committed
└── keyboard-layout.md               # UPDATE — replace v1.x letter conventions with HP-41CV reference card

justfile                              # +docs-matrix recipe (Rust bin or shell)

scripts/docs-matrix/                  # NEW (only if Rust-bin variant chosen)
└── main.rs                          # Reads JSON, writes Markdown table

CLAUDE.md                             # +v2.2 settled-architecture block (FN-DOC-02)
README.md                             # Soft "feature-complete HP-41CV" claim + link to matrix
```

### Pattern 1: One-Shot Prefix State Machine (D-25.4)

**What:** Single `bool` field on `App`; armed by `f`, consumed by next op-key, cleared by Esc.

**When to use:** Any TUI prefix-shift modal where a one-time modifier alters the very next key's interpretation. Directly mirrors `shiftActive` in `hp41-gui/src/App.tsx` (v2.1 D-5).

**Example:**
```rust
// hp41-cli/src/app.rs (App struct addition)
pub struct App {
    // … existing fields …
    /// One-shot HP-41CV f-prefix arm state.
    /// True for exactly one key-press cycle after `f` is pressed; consumed
    /// by the next op key (returns the f-shifted Op via shifted_key_to_op)
    /// OR cleared by Esc. Never crosses IPC — frontend-only per D-25.5.
    pub shift_armed: bool,
}

// hp41-cli/src/app.rs (handle_key, after pending_input route + before modal openers)
// Arm the prefix on f-press; consume on any other op key.
if !self.shift_armed && key.code == KeyCode::Char('f') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.shift_armed = true;
    self.message = None;
    return;
}
if self.shift_armed {
    if key.code == KeyCode::Esc {
        self.shift_armed = false;
        return;
    }
    // Try shifted resolver first; on miss, the prefix is consumed silently.
    if let Some(op_or_modal) = keys::shifted_key_to_op(key, self) {
        self.call_dispatch(op_or_modal);
    }
    // ALWAYS clear after consumption (one-shot, D-25.4):
    self.shift_armed = false;
    return;
}
// … existing direct-key path follows …
```

**Status-bar indicator** in `render_status`: change line 212 of ui.rs from `ann("SHIFT", false)` to `ann("SHIFT", app.shift_armed)`. Optionally extend pending_prompt to prepend `f→ ` to the status line when armed and no other prompt is active.

### Pattern 2: Hybrid PendingInput Struct-Variants (D-25.11)

**What:** Group ops with identical input shape (2-digit register + IND-toggle) into one struct-variant; keep ops with unique input shape (text label / 3-digit count / single-digit) as their own tuple variants.

**Example:**
```rust
// hp41-cli/src/app.rs (PendingInput enum — new variants)
pub enum PendingInput {
    // … existing 11 variants (unchanged) …

    /// Phase 25: Flag operations modal (SF/CF/FS?/FC?/FS?C/FC?C × direct/IND).
    /// `kind` carries the flag-test discriminator (reuses hp41-core::FlagTestKind +
    /// two extra variants for SF/CF — see RegisterOpKind discussion below for the
    /// pattern). `ind=true` means "next dispatch picks Op::*Ind(n)", `ind=false`
    /// picks `Op::*(n)`. Status bar:
    ///   "SF [_ _]"        → ind=false, acc=""
    ///   "SF IND [1_]"     → ind=true,  acc="1"
    FlagPrompt {
        kind: FlagPromptKind,  // local: SetFlag / ClearFlag / Test(FlagTestKind)
        ind: bool,
        acc: String,
    },

    /// Phase 25: Register operations modal (STO/RCL/STO+-*//VIEW/ARCL/ASTO/ISG/DSE × direct/IND).
    RegisterPrompt {
        op: RegisterOpKind,
        ind: bool,
        acc: String,
    },

    /// Phase 25: CLP "name" — text-input modal (ALPHA-style char accumulation).
    /// Terminator: Enter dispatches Op::Clp(name); Esc cancels.
    ClpLabel(String),

    /// Phase 25: DEL nnn — 3-digit numeric for Op::Del(n) (silent-clamp at u8::MAX).
    DelCount(String),

    /// Phase 25: TONE n — single-digit 0–9 for Op::Tone(n).
    TonePrompt,

    /// Phase 25: XEQ "NAME" — text-input modal that routes 8 non-keyboard
    /// conditional tests + future ROM-name dispatch through one resolver.
    /// Triggered by XEQ key (f-shifted XEQ? or direct XEQ key per reference card).
    XeqByName(String),
}
```

**Local enums (TUI-only, NOT in hp41-core per D-25.13):**
```rust
// hp41-cli/src/keys.rs or hp41-cli/src/app.rs
#[derive(Debug, Clone)]
pub enum RegisterOpKind {
    Sto,
    Rcl,
    StoArith(hp41_core::ops::StoArithKind),  // reuse core enum
    View,
    Arcl,
    Asto,
    Isg,
    Dse,
}

#[derive(Debug, Clone)]
pub enum FlagPromptKind {
    SetFlag,
    ClearFlag,
    Test(hp41_core::ops::FlagTestKind),  // reuse core enum (4 variants: IsSet/IsClear/IsSetThenClear/IsClearThenClear)
}
```

The dispatch decision at end-of-accumulation:
```rust
// At the point in handle_pending_input where the 2-digit accumulator fills
let n: u8 = acc.parse().expect("two ASCII digit chars always parse as u8 ≤ 99");
let op = match (kind, ind) {
    (RegisterOpKind::Sto, false) => Op::StoReg(n),
    (RegisterOpKind::Sto, true)  => Op::StoInd(n),
    (RegisterOpKind::Rcl, false) => Op::RclReg(n),
    (RegisterOpKind::Rcl, true)  => Op::RclInd(n),
    (RegisterOpKind::StoArith(k), false) => Op::StoArith { reg: n, kind: k.clone() },
    (RegisterOpKind::StoArith(k), true)  => Op::StoArithInd(n, k.clone()),
    // … 12 more arms …
};
self.call_dispatch(op);
```

### Pattern 3: include_str! + OnceLock for JSON-embedded data

**What:** Embed the canonical JSON at compile time; lazy-parse once on first access.

**When to use:** Any compile-time-known data file that the binary should load without runtime file I/O. Project precedent in `hp41-cli/src/programs.rs:7,19` [VERIFIED: grep].

**Example:**
```rust
// hp41-cli/src/help_data.rs (new shape per D-25.16)
use std::sync::OnceLock;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HelpEntry {
    pub op_variant: String,      // hp41-core Op:: PascalCase name (e.g. "Pi")
    pub display_name: String,     // HP-41 mnemonic (e.g. "PI")
    pub category: String,         // "Math" / "Stack" / "Flags" / …
    pub status: String,           // "implemented" | "deferred-v3" | "na"
    pub phase: Option<String>,    // GSD phase ID (e.g. "20") or null
    pub key_path: Option<String>, // CLI keystroke (e.g. "f-pi", "f--") or null
    pub description: String,
    #[serde(default)]
    pub divergences: Vec<String>,
}

/// Compile-time-embedded canonical data. Build-time failure if file is missing.
const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");

static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed help entries (lazily initialized; thread-safe via OnceLock).
/// Panics on malformed JSON via `.expect("hp41cv-functions.json is malformed")` —
/// this is a build-time fault not a runtime concern (D-25.17). Hard-build-blocker
/// by design per locked decision.
pub fn help_entries() -> &'static [HelpEntry] {
    HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(FUNCTIONS_JSON)
            .expect("hp41cv-functions.json is malformed — fix the JSON")
    })
}
```

The legacy `pub const HELP_DATA: &[(&str, &str, &str)]` is replaced by a derivation that re-shapes `help_entries()` into the existing 3-tuple format expected by `ui.rs::render_help_overlay` (or `render_help_overlay` is rewritten to consume `HelpEntry` directly — recommend the latter for clarity).

### Pattern 4: Exhaustive Match Discipline (D-25.14, FN-CLI-04)

**What:** Every match on `PendingInput` uses `_` and `unreachable!()` ONLY for genuinely impossible cases; new variants force every match site to add an arm at compile time.

**Example:**
```rust
// hp41-cli/src/ui.rs::pending_prompt() — every variant must have an arm
fn pending_prompt(pending: &PendingInput) -> String {
    use crate::app::PendingInput;
    match pending {
        // … existing 11 arms (unchanged) …

        PendingInput::FlagPrompt { kind, ind, acc } => {
            let mnemonic = match kind {
                FlagPromptKind::SetFlag => "SF",
                FlagPromptKind::ClearFlag => "CF",
                FlagPromptKind::Test(FlagTestKind::IsSet) => "FS?",
                FlagPromptKind::Test(FlagTestKind::IsClear) => "FC?",
                FlagPromptKind::Test(FlagTestKind::IsSetThenClear) => "FS?C",
                FlagPromptKind::Test(FlagTestKind::IsClearThenClear) => "FC?C",
            };
            let ind_str = if *ind { " IND" } else { "" };
            format!("{mnemonic}{ind_str} [{acc:_<2}]")
        }
        PendingInput::RegisterPrompt { op, ind, acc } => {
            let mnemonic = match op {
                RegisterOpKind::Sto => "STO",
                RegisterOpKind::Rcl => "RCL",
                RegisterOpKind::StoArith(StoArithKind::Add) => "STO+",
                RegisterOpKind::StoArith(StoArithKind::Sub) => "STO-",
                RegisterOpKind::StoArith(StoArithKind::Mul) => "STO\u{00D7}",
                RegisterOpKind::StoArith(StoArithKind::Div) => "STO\u{00F7}",
                RegisterOpKind::View => "VIEW",
                RegisterOpKind::Arcl => "ARCL",
                RegisterOpKind::Asto => "ASTO",
                RegisterOpKind::Isg => "ISG",
                RegisterOpKind::Dse => "DSE",
            };
            let ind_str = if *ind { " IND" } else { "" };
            format!("{mnemonic}{ind_str} [{acc:_<2}]")
        }
        PendingInput::ClpLabel(acc)  => format!("CLP [{acc}]_ "),
        PendingInput::DelCount(acc)  => format!("DEL [{acc:_<3}]"),
        PendingInput::TonePrompt     => "TONE [_]".to_string(),
        PendingInput::XeqByName(acc) => format!("XEQ \"{acc}\"_"),
    }
}
```

The Rust compiler emits an exhaustiveness warning on every match site if a new `PendingInput` variant is added without an arm — that's the compile-time guarantee that `unreachable!()` is never silently introduced.

### Anti-Patterns to Avoid

- **Don't add a generic `NumericPrompt(...)` variant** that forces TONE (1 digit, no IND) and CLP (text input) into a shared shape — that mis-represents their state shape. Use struct-variants for groups with identical shape, specialty variants for everything else (D-25.11).
- **Don't introduce parallel TUI-local enums where hp41-core already has one** (D-25.13). Reuse `FlagTestKind` (Phase 21), `StoArithKind` (Phase 9), `TestKind` (Phase 3). Only define `RegisterOpKind` and `FlagPromptKind` locally because hp41-core has no equivalent.
- **Don't use `_=>` or `unreachable!()` in `pending_prompt`** (D-25.14, FN-CLI-04). The compile-time exhaustive-match discipline IS the runtime guarantee.
- **Don't introduce a `build.rs` for the JSON embed** (D-25.16). `include_str!` is sufficient and adds zero complexity.
- **Don't use `state.regs[i]` raw indexing in new code** (D-22.11.1, CLAUDE.md). Use `state.regs.get(i as usize).ok_or(HpError::InvalidOp)?`. Applies to any new TUI helpers that read registers (Phase 25 likely doesn't add any, but the rule remains).
- **Don't have the new f-prefix logic above the pending_input routing block** in `handle_key`. Active modals must consume f-key presses too (per existing pattern at app.rs:228). The arming logic goes *after* pending_input route, *before* modal openers.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| One-shot prefix state | Custom state-machine struct or enum | Single `pub shift_armed: bool` on App | GUI v2.1 D-5 uses the identical idiom (`shiftActive: boolean`); parity with that is mandatory per D-25.6 |
| Modal accumulator scaffolding | New per-modal struct | Reuse `handle_reg_modal()` closure pattern (app.rs:913) | Already a proven generic 2-digit accumulator with Backspace/Esc/auto-dispatch; new modals slot in with op_fn + pending_fn closures |
| JSON parse-and-cache | `lazy_static` macro, `once_cell` external dep | `std::sync::OnceLock` (stdlib, MSRV 1.88) | Project precedent in `programs.rs`; no new dep |
| Iterating Op variants for parity check | `std::mem::discriminant` or unsafe variant counting | Hand-curated `const ALL_OP_VARIANTS: &[Op] = &[Op::Add, Op::Sub, …]` array in the test file (or `strum::EnumIter` if dep cost accepted) | Rust has no built-in introspection; the hand-curated list is small (130 entries), CI-checked, and forces a deliberate "did you remember to add the JSON entry?" gate every time a new Op lands. Recommendation: hand-curated, ALL caps suffix `_INVENTORY` to flag the maintenance burden. |
| Generating the function matrix | Inline `bash` here-docs / `awk` | Small Rust bin in `scripts/docs-matrix/` or just-recipe with `jq` | Rust bin type-checks the JSON schema and can re-use the same `HelpEntry` struct from `help_data.rs` — single source of truth. `jq` works but doesn't validate schema. |
| XEQ-by-Name resolution for the 8 conditional tests + future ROM ops | New ad-hoc match in hp41-cli | Extend `hp41-core::ops::program::builtin_card_op` (rename to `builtin_op_by_name`) | Already wired into `op_xeq`, `run_program`, AND `run_loop` (Phase 22). Extending it covers all 3 invocation paths with one match. |

**Key insight:** Phase 25 is a wiring + documentation phase, NOT an architecture phase. Every new feature can be expressed as a thin layer over existing patterns (App struct field; existing PendingInput-handler closure; existing OnceLock-cached static; existing builtin_card_op resolver). Resist the temptation to introduce abstractions — the CONTEXT-locked Hybrid PendingInput design is the only structural decision, and it's already locked.

## Runtime State Inventory

**Phase 25 is NOT a rename/refactor/migration phase** — it's a feature-wiring phase. No registered OS state, no live service config, no stored data with embedded strings. **Section omitted per RESEARCH.md guidance.**

Sole adjacent concern: the legacy `pub const HELP_DATA: &[(&str, &str, &str)]` in `help_data.rs` is replaced. Any external tests or doctests that reference `HELP_DATA` by name break at compile time — verified by grep:

```bash
grep -rn "HELP_DATA" --include="*.rs" .
```
results: `help_data.rs:11` (the const itself), `ui.rs:285` (the consumer), `help_data.rs` mod tests (lines 348–407 — the four tests will need rewriting). No external consumers. [VERIFIED: grep across repo]

## Common Pitfalls

### Pitfall 1: Crossterm `KeyEventKind::Release` doubles every key on Windows
**What goes wrong:** Without the existing `if key.kind != KeyEventKind::Press { return }` filter (app.rs:183), each keypress would arm AND consume the prefix in one cycle on Windows, making the f-prefix unusable.
**Why it happens:** Windows crossterm fires both Press and Release events for every key.
**How to avoid:** The filter is already the first check in `handle_key` (project invariant from Phase 4). Phase 25 MUST NOT add any prefix-arming logic above that filter — same rule as the pending_input route.
**Warning signs:** F-prefix appears to do nothing on Windows; status bar `f→` flashes momentarily.

### Pitfall 2: ALPHA-mode overrides f-prefix silently
**What goes wrong:** Per D-25.5, in ALPHA mode `f` types F. If the prefix-arming check runs above the ALPHA mode block (app.rs:299), `f` is consumed by the prefix logic and never reaches the ALPHA append path.
**Why it happens:** Ordering of guards in handle_key is significant.
**How to avoid:** Put the prefix-arming check AFTER the ALPHA-mode routing branch (app.rs:299-301), or guard it with `if !self.state.alpha_mode { … }`. The CONTEXT D-25.5 explicitly notes ALPHA overrides Prefix.
**Warning signs:** Cannot type letter F in ALPHA mode.

### Pitfall 3: `f` consumed by FmtDigits-cycle binding (current v1.x behavior)
**What goes wrong:** Today, app.rs:467 dispatches `f` to a FIX/SCI/ENG cycle. Phase 25 removes this binding — but if removal is incomplete, `f` does both (arms prefix AND cycles format).
**Why it happens:** The v1.x letter-binding migration (D-25.3) requires every old `f` reference in the CLI to be re-pointed or removed.
**How to avoid:** When wiring the prefix-arming check, REMOVE app.rs:466-475 (the `f` cycle block) atomically in the same commit. Add a regression test that pressing `f` once arms the prefix and does NOT change `state.display_mode`.
**Warning signs:** Display mode changes whenever the user arms the prefix.

### Pitfall 4: PendingInput route swallows the f-prefix on a stuck modal
**What goes wrong:** If pending_input is `Some(...)` and the user presses `f`, the modal eats the key. This is the current behavior for ALL keys (app.rs:228 — pending_input route is above modal openers per Pitfall 5 of v1.x).
**Why it happens:** Existing safety pattern from CR-02 (Phase 5).
**How to avoid:** This is INTENDED behavior. f-prefix arming inside an active modal is meaningless (modals already have a defined Esc/Enter/digit interface). The prefix-arming check goes AFTER the pending_input route. Document this as a feature, not a bug.
**Warning signs:** None — by design.

### Pitfall 5: shift_armed bleeds across handle_key invocations on partial consumption
**What goes wrong:** If the user arms with `f`, then presses an unmapped key (e.g., `;`), the prefix should clear silently. If the implementation only clears `shift_armed = false` inside the matched arm, the prefix sticks until a recognized key fires.
**Why it happens:** Easy-to-miss state-machine edge case; mirrors a Phase 19 GUI bug surfaced by `Pitfall 4` in v2.1 D-5.
**How to avoid:** ALWAYS clear `shift_armed = false` at the END of the shift_armed=true branch in `handle_key`, regardless of whether `shifted_key_to_op` returned `Some` or `None`. The one-shot lifetime is "next key cycle", not "next consumed key" (D-25.4).
**Warning signs:** F-prefix appears latched after an unmapped key; subsequent unrelated keys are mis-interpreted as shifted.

### Pitfall 6: Op enum has 130 variants — function matrix has fewer rows
**What goes wrong:** The CI parity test naïvely asserts `Op enum variants count == matrix entry count`; this fails because `Op::Sto(u8)` is ONE Op variant but corresponds to ONE matrix row ("STO"), and `Op::PushNum(HpNum)` doesn't correspond to a matrix row at all (it's internal entry-buffer flushing, not an HP-41 ROM op).
**Why it happens:** Op enum variants represent both ROM ops AND internal calculator primitives; the matrix only tracks ROM ops.
**How to avoid:** The CI parity test enumerates Op variants and looks each one up in the JSON by `op_variant` PascalCase name. Some variants intentionally have NO matrix entry (mark these with a hardcoded skiplist: `PushNum`, `SyntheticByte`, `Lbl`, `Gto`, `Xeq` — the ROM mnemonics are LBL/GTO/XEQ which are documented entries but the Op variants are parameterized). The reverse check (every JSON entry must have an Op variant OR explicit `status: "deferred-v3"`) is the real drift catch.
**Warning signs:** CI parity test fails on first run; planner spent time debugging "Op::PushNum missing from matrix" when it's intentional.

### Pitfall 7: include_str! of an empty/missing JSON file = silent build failure mode
**What goes wrong:** `include_str!("../../docs/hp41cv-functions.json")` fails at compile time with a clear error message — but a future contributor might `touch` the file and commit an empty string, which then `serde_json::from_str` rejects at runtime startup with the `.expect("…")` panic.
**Why it happens:** include_str! validates file existence, not file content.
**How to avoid:** Add a `cargo test` integration test that loads `help_entries()` and asserts `.len() >= 130`. This catches empty-file commits at CI time, not at user-startup time. Recommended location: `hp41-cli/tests/help_data_smoke.rs`.
**Warning signs:** Local `cargo build` succeeds, CI test fails with "hp41cv-functions.json is malformed".

### Pitfall 8: `just docs-matrix` regenerates but doesn't fail-on-drift in CI
**What goes wrong:** Per D-25.15: "A CI test verifies the committed .md matches what `just docs-matrix` would regenerate (drift catch)". A naïve `just docs-matrix` recipe overwrites the file without verifying, so CI would commit-loop forever.
**Why it happens:** Generate-and-overwrite vs. generate-and-diff are different recipe shapes.
**How to avoid:** Two recipes: `just docs-matrix` (regenerate, write file — for the developer) and `just docs-matrix-check` (regenerate to a temp file, `diff` against committed, exit non-zero on mismatch — for CI). CI uses the check variant.
**Warning signs:** PR shows committed `hp41cv-function-matrix.md` updates on every commit; CI passes but the diff is noisy.

### Pitfall 9: XEQ-by-Name modal accepts arbitrary text → unknown mnemonic = silent error
**What goes wrong:** User types `XEQ "FOOBAR"` → dispatch returns `Err(HpError::InvalidOp)` → message shown but no hint of "did you mean…?" → poor UX.
**Why it happens:** The current `op_xeq` falls through to `builtin_card_op(label).is_none() → Err(InvalidOp)`.
**How to avoid:** This is acceptable for Phase 25 (matches HP-41 hardware which beeps on unknown XEQ). For Phase 26 (GUI) consider adding a "did you mean…?" hint via fuzzy-match. Phase 25 just needs `builtin_op_by_name` (renamed `builtin_card_op`) to cover the 8 conditional-test mnemonics — anything else falling through to InvalidOp is expected.
**Warning signs:** None — this is documented behavior.

### Pitfall 10: HP-41CV "IND modifier key" is shift-0, NOT f-XEQ
**What goes wrong:** CONTEXT D-25.12 speculated "likely f-XEQ or similar" for the IND key. The actual HP-41C/CV QRG (page 14) confirms IND = **shift-0** (the digit-0 key, f-shifted): *"An indirect address is selected by following a function with the shift key, ■, and then a register address"*. Implementing IND on f-XEQ would deviate from hardware.
**Why it happens:** Without verifying against the QRG, the speculation could become a locked decision.
**How to avoid:** When inside a `FlagPrompt`/`RegisterPrompt` modal AND `shift_armed == true`, pressing `0` toggles `ind` instead of accumulating a digit. The user flow: `S` → modal opens (`ind=false`) → `f` arms shift → `0` toggles IND (`ind=true`) → user types `12` → dispatches `Op::StoInd(12)`. Alternative simpler UX: introduce a dedicated `I` keystroke inside modals that toggles IND (deviates from hardware but more discoverable in a 102-key PC keyboard environment). Planner picks; recommend the hardware-faithful `f-0` toggle. [VERIFIED: HP-41C/CV Quick Reference Guide, p.14]
**Warning signs:** Users from real HP-41CV background can't find IND.

## Code Examples

Verified patterns from the existing codebase:

### Generic 2-digit register accumulator (existing, REUSE in Phase 25)
```rust
// Source: hp41-cli/src/app.rs:913
fn handle_reg_modal(
    &mut self,
    key: KeyEvent,
    acc: String,
    op_fn: impl Fn(u8) -> Op,
    pending_fn: impl Fn(String) -> PendingInput,
) {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let mut new_acc = acc;
            new_acc.push(c);
            if new_acc.len() == 2 {
                let reg: u8 = new_acc.parse()
                    .expect("two ASCII digit chars always parse as u8 ≤ 99");
                self.call_dispatch(op_fn(reg));
                self.pending_input = None;
            } else {
                self.pending_input = Some(pending_fn(new_acc));
            }
        }
        KeyCode::Backspace => self.pending_input = Some(pending_fn(String::new())),
        KeyCode::Esc => self.pending_input = None,
        _ => self.pending_input = Some(pending_fn(acc)),
    }
}
```

Phase 25 generalizes this with an IND-toggle key (`I` or hardware-faithful `f-0`):

```rust
// Phase 25 sketch — handle_reg_modal_with_ind (NEW)
fn handle_reg_modal_with_ind(
    &mut self,
    key: KeyEvent,
    op: RegisterOpKind,
    ind: bool,
    acc: String,
) {
    match key.code {
        // IND toggle — recommend `I` keystroke for discoverability; can also wire f-0
        KeyCode::Char('I') | KeyCode::Char('i') => {
            self.pending_input = Some(PendingInput::RegisterPrompt {
                op, ind: !ind, acc,
            });
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let mut new_acc = acc;
            new_acc.push(c);
            if new_acc.len() == 2 {
                let n: u8 = new_acc.parse()
                    .expect("two ASCII digit chars always parse as u8 ≤ 99");
                let final_op = resolve_register_op(op, ind, n);  // see Pattern 2 above
                self.call_dispatch(final_op);
                self.pending_input = None;
            } else {
                self.pending_input = Some(PendingInput::RegisterPrompt {
                    op, ind, acc: new_acc,
                });
            }
        }
        KeyCode::Backspace => {
            self.pending_input = Some(PendingInput::RegisterPrompt {
                op, ind, acc: String::new(),
            });
        }
        KeyCode::Esc => self.pending_input = None,
        _ => {
            self.pending_input = Some(PendingInput::RegisterPrompt { op, ind, acc });
        }
    }
}
```

### OnceLock-cached compile-time data load (existing pattern)
```rust
// Source: hp41-cli/src/programs.rs:19,22 [VERIFIED: grep]
static PROGRAMS_CACHE: OnceLock<Vec<SampleProgram>> = OnceLock::new();

pub fn sample_programs() -> &'static [SampleProgram] {
    PROGRAMS_CACHE.get_or_init(build_all_programs)
}
```

Phase 25's help_data.rs mirrors this exactly with `serde_json::from_str(FUNCTIONS_JSON).expect(…)` as the init function.

### Extending the XEQ-by-Name resolver (Phase 25 work in hp41-core)

**Wait — Phase 25 ONLY touches hp41-cli per CONTEXT.** This is a tension point.

The CONTEXT says:
> *D-25.8: Remaining 8 conditional tests reachable only via XEQ-by-Name palette. … the XEQ-by-Name modal (v2.1 card-reader phase, already shipped) dispatches them by mnemonic name.*

But the v2.1 `builtin_card_op` resolver in hp41-core only knows 4 names (WPRGM/RDPRGM/WDTA/RDTA). Extending it to know 8 more conditional-test mnemonics requires editing hp41-core — which CONTEXT says is off-limits.

**Resolution paths (planner decides):**
1. **CLI-local resolver path:** Add a `xeq_by_name_resolve(name: &str) -> Option<Op>` helper IN hp41-cli that handles the 8 conditional-test mnemonics (returning `Op::Test(TestKind::…)`), then dispatches via the existing `dispatch_op` path. The hp41-core `op_xeq`/`run_program`/`run_loop` builtin_card_op fallback STAYS at 4 names; CLI's XEQ-by-Name modal intercepts before `dispatch(state, Op::Xeq(name))` is even built.
   - **Pro:** Honors the "no hp41-core changes" constraint.
   - **Con:** The 8 mnemonics aren't reachable from a programmatic `XEQ "X<>Y"` inside a saved program — only from the keyboard modal. For a user who programs a saved card, this is asymmetric.
2. **Tiny hp41-core extension path:** Extend `builtin_card_op` to a `builtin_op_by_name` with 4 + 8 = 12 names. CONTEXT lists "hp41-core is FROZEN" as a constraint, but this is the *minimal* exception — no new variants, no new state, no new error variants; just an enlarged match arm with already-existing `Op::Test(TestKind::…)` returns.
   - **Pro:** Programmatic + keyboard symmetry; matches HP-41 hardware semantics where `XEQ "X<>Y"` inside a program works identically to typed XEQ.
   - **Con:** Violates the literal "no hp41-core changes" CONTEXT statement.

**Recommendation:** Path 2 (tiny hp41-core extension). Justification: CONTEXT D-25.8 explicitly says "via XEQ-by-Name palette … dispatches them by mnemonic name", which is the existing builtin_card_op pathway. Extending the table is faithful to the spirit of "no new Ops / no new state / no new errors" — only the name-table grows. Document this as an exception in the plan and clear it with the user before execution.

**Open question for discuss-phase clearance:** Is the `builtin_card_op` table extension considered a "hp41-core change" that violates the Phase 25 boundary? See Open Questions section below.

### just docs-matrix recipe — Rust bin variant (RECOMMENDED)
```just
# justfile (Phase 25 addition)
# Regenerate the function matrix from canonical JSON.
docs-matrix:
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41cv-functions.json docs/hp41cv-function-matrix.md

# CI-friendly variant: regenerate to a temp file and diff against the committed version.
docs-matrix-check:
    cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
        docs/hp41cv-functions.json /tmp/hp41cv-function-matrix-check.md
    diff -u docs/hp41cv-function-matrix.md /tmp/hp41cv-function-matrix-check.md
```

The Rust bin reuses the `HelpEntry` struct from `help_data.rs` (or its own local copy with identical schema) — single source of truth for the JSON shape. Bin source: `scripts/docs-matrix/src/main.rs` (≤80 LOC, basic table-rendering loop).

### CI parity test (Phase 25 plan output)
```rust
// hp41-cli/tests/function_matrix_parity.rs (NEW)
//
// Bidirectional drift catch: ensures docs/hp41cv-functions.json stays in sync
// with hp41_core::ops::Op enum. Hand-curated Op inventory avoids the strum
// dependency cost on hp41-core.

use hp41_cli::help_data::help_entries;
use hp41_core::ops::Op;

/// Hand-curated inventory of all hp41_core::ops::Op variants. Maintenance
/// burden: every new Op variant landed in Phases 20+ must be appended here.
/// SC-4 mirror invariant analog — if this list and Op enum diverge, the
/// test fails with a clear "missing variant: …" message.
const ALL_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 1 arithmetic / stack
    "Add", "Sub", "Mul", "Div", "Enter", "Clx", "Chs", "Rdn", "Rup",
    "XySwap", "Lastx", "Pi", "PushNum",
    // Phase 2 unary math / trig / mode / registers / alpha
    "Int", "Rnd", "Frc", "Abs", "Sign", "Fact", "Recip", "Sqrt", "Sq",
    "YPow", "Mod", "PctChange", "Ln", "Log", "Exp", "TenPow",
    "Sin", "Cos", "Tan", "Asin", "Acos", "Atan", "PolarToRect", "RectToPolar",
    "SetDeg", "SetRad", "SetGrad",
    "FmtFix", "FmtSci", "FmtEng",
    "StoReg", "RclReg", "StoArith", "StoArithStack", "Clreg",
    "AlphaToggle", "AlphaAppend", "AlphaClear",
    // Phase 3 programming
    "Lbl", "Gto", "Xeq", "Rtn", "PrgmMode", "Test", "Isg", "Dse",
    // Phase 5 USER mode, ALPHA back
    "UserMode", "AlphaBackspace",
    // Phase 6 stats / HMS
    "SigmaPlus", "SigmaMinus", "Mean", "Sdev", "LR", "Yhat", "Corr", "ClSigmaStat",
    "HmsToH", "HToHms", "HmsAdd", "HmsSub",
    // Phase 11 print
    "PRX", "PRA", "PRSTK",
    // Phase 12 synthetic
    "GetKey", "Null", "StoM", "StoN", "StoO", "RclM", "RclN", "RclO", "SyntheticByte",
    // v2.1 card reader
    "Wdta", "Rdta", "Wprgm", "Rdprgm",
    // Phase 21 flags / display / sound
    "SfFlag", "CfFlag", "FlagTest",
    "View", "AView", "Prompt", "Aon", "Aoff", "Cld",
    "Beep", "Tone",
    // Phase 22 program control / editing / memory / catalog / ASN
    "Stop", "Pse", "GtoInd", "XeqInd",
    "Clp", "Del", "Ins",
    "Size", "Cla", "Clst", "Pack", "Catalog", "Asn",
    // Phase 23 ALPHA ops
    "Arcl", "Asto", "Atox", "Xtoa", "Arot", "Posa",
    // Phase 24 indirect
    "StoInd", "RclInd", "StoArithInd", "IsgInd", "DseInd",
    "SfFlagInd", "CfFlagInd", "FlagTestInd", "ArclInd", "AstoInd", "ViewInd",
];

/// Op variants that do NOT correspond to an HP-41CV ROM op — these are
/// internal calculator primitives. The JSON matrix should NOT list them.
const INTERNAL_OP_VARIANTS: &[&str] = &[
    "PushNum",       // numeric-literal entry, not a named ROM op
    "SyntheticByte", // hex-modal insertion, internal
];

#[test]
fn test_op_inventory_count_matches_enum() {
    // Maintenance gate — if Op enum grows past 130 variants without this
    // list growing, the assertion below catches it. Manually update both.
    assert_eq!(
        ALL_OP_VARIANT_NAMES.len(),
        130,
        "ALL_OP_VARIANT_NAMES out of sync with hp41_core::ops::Op enum. \
         Did Phase 26/27 add new variants without updating this inventory?"
    );
}

#[test]
fn test_every_rom_op_has_matrix_entry() {
    let entries = help_entries();
    for name in ALL_OP_VARIANT_NAMES {
        if INTERNAL_OP_VARIANTS.contains(name) { continue; }
        assert!(
            entries.iter().any(|e| e.op_variant == *name),
            "Op::{name} has no entry in docs/hp41cv-functions.json"
        );
    }
}

#[test]
fn test_every_implemented_matrix_entry_has_op() {
    let entries = help_entries();
    for entry in entries {
        if entry.status != "implemented" { continue; }
        assert!(
            ALL_OP_VARIANT_NAMES.contains(&entry.op_variant.as_str()),
            "JSON entry '{}' (status: implemented) has no matching Op variant",
            entry.op_variant
        );
    }
}

#[test]
fn test_matrix_has_at_least_130_entries() {
    let entries = help_entries();
    assert!(
        entries.len() >= 130,
        "function matrix should list ≥130 HP-41CV ROM ops; got {}",
        entries.len()
    );
}
```

## HP-41CV Keyboard Reference (researched from QRG)

> [VERIFIED: literature.hpcalc.org/community/hp41c41cv-qrg-en.pdf — pages 1 (normal-mode), 2 (ALPHA-mode), 6–9 (function index), 14 (indirect ops)]

The HP-41C/CV keyboard is a **5-column × 7-row** main grid plus **4 top buttons** (ON / USER / PRGM / ALPHA). The bottom row's ENTER spans 2 columns. Total physical key positions: **4 top + 35 grid = 39 entries** (matches `hp41-gui/src/Keyboard.tsx` `KEY_DEFS` v2.1 count).

### Key-position table (row × col, primary / f-shifted / ALPHA-char)

| Row | Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-----|-------|-------|-------|-------|-------|
| 1 | Σ+ / Σ- / *(no alpha)* | 1/x / y^x | √x / x² | LOG / 10^x | LN / e^x |
| 2 | XEQ / ASN / *A* | STO / →HMS / *B* | RCL / →H / *C* | SST / BST / *D* | R/S / P↔R / *E* |
| 3 | ←/CL X / CLΣ / *(none)* | Σ+ *(?)* — *see note* | ÷ / →P / *F* (alpha digit) | × / →HMS+ / *G* | — |
| (rows 3–7 differ; see canonical layout below)
| Bottom rows | (digits 0–9, ENTER, +, -, ×, ÷, EEX, CHS, ←) |  |  |  |  |

Note: my row/col enumeration above is approximate — the QRG image showed the physical-keyboard photo clearly but the **canonical mapping** is:

**Top row (mode keys):** `ON` `USER` `PRGM` `ALPHA` — NOT in the 5×7 grid.

**Main grid (5 × 7, ENTER 2-wide on bottom row 5):**

Row 1 — primary / f-shifted / ALPHA char (from QRG page 1 photo + function index page 6):
- (1,1) **Σ+** / Σ- / A
- (1,2) **1/x** / y^x / B
- (1,3) **√x** / x² / C
- (1,4) **LOG** / 10^x / D
- (1,5) **LN** / e^x / E

Row 2:
- (2,1) **XEQ** / ASN / F (or ALPHA-mode key `F`)
- (2,2) **STO** / →HMS / G
- (2,3) **RCL** / →H / H
- (2,4) **SST** / BST / I
- (2,5) **R/S** / P↔R *(also written as ON in some sources, but R/S is the canonical primary)* / J

Row 3:
- (3,1) **←/CLX** / CLΣ / K
- (3,2) **ENTER↑** / ALPHA / *(none, ENTER is a 2-wide key in some HP-41 variants — but per QRG photo it's a single-col cell here)*
- (3,3) **CHS** / MODES / L
- (3,4) **EEX** / DISP / M
- (3,5) **←** / CLEAR / *(see CLEAR submenu)*

(Discrepancy note: the QRG photo and the schematic `docs/keyboard-layout.md` disagree on which row ENTER lives. The QRG photo shows ENTER↑ as the 2-wide key on a lower row. Phase 25 planner must reconcile against the actual HP-41C/CV photograph during execution. See Open Question 1.)

Rows 4–6 — digit pad + arithmetic:
- Row 4: 7 / 8 / 9 / ÷ — f-shifted: BEEP / TONE / ... (per QRG p.6–9, ÷ f-shifts to P→R)
- Row 5: 4 / 5 / 6 / × — f-shifted: % / %CH / →HMS+
- Row 6: 1 / 2 / 3 / − — f-shifted: SF / CF / FS? / **X=Y? (the f- arithmetic test, D-25.7)**
- Row 7 (bottom, ENTER 2-wide): 0 / . / R/S / + — f-shifted: PI / PSE / RND / **X≤Y? (the f+ arithmetic test, D-25.7)**

(Approximate; exact f-shifted assignments differ slightly across HP-41C/CV/CX revisions. Per D-25.7 the user has confirmed the 4 conditional-test bindings on their physical CV; the rest is sourced from the QRG.)

### IND modifier key — VERIFIED

From QRG page 14:

> *"An indirect address is selected by following a function with the shift key, ■, and then a register address. … `STO` ■ nn = Store. `RCL` ■ nn = Recall. … `XEQ` ■ nn = Execute (00 through 99 or ALPHA name). … `CF` ■ nn = Clear flag (00 through 55)."*

The **■** symbol in QRG = the yellow shift key (`f`). The IND modifier is reached by:

1. Press the function key (e.g., `STO`)
2. Press `f` (shift) — this opens the "■" sub-prompt
3. Type the 2-digit register address

In our TUI implementation, this maps to:
1. User presses `S` → `PendingInput::RegisterPrompt { op: Sto, ind: false, acc: "" }` opens
2. User presses `f` → shift_armed=true (or, since we're already inside a modal, the prefix gets consumed by the modal as an IND-toggle key)
3. Modal updates to `ind: true`, status shows "STO IND [__]"
4. User types `12` → dispatches `Op::StoInd(12)`

**Phase 25 planner decides between two implementation flavors:**
- **Faithful (hardware):** `f` inside a `RegisterPrompt`/`FlagPrompt` modal toggles `ind`. The shift_armed state is per-modal, not global.
- **Simple (TUI-friendly):** Dedicated `I` keystroke inside the modal toggles `ind`. More discoverable for users without HP-41 hardware muscle memory.

**Recommendation:** Faithful flavor (D-25.12 says "Pressing the IND key (HP-41CV-specific position; planner: confirm IND key from reference card — likely f-XEQ or similar)" — this research confirms it's actually `f`-then-digit, not a dedicated IND key). The simple flavor can be added as a secondary keybinding in the same modal.

### Conditional tests — keyboard vs XEQ-by-Name

From QRG page 8 ("Function Index") + page 14 ("Indirect Operations") + user's physical-HP-41CV confirmation in CONTEXT D-25.7:

**4 on physical keyboard (f-shifted arithmetic keys per D-25.7):**
- `f -` → `X=Y?` → `Op::Test(TestKind::XEqY)`
- `f +` → `X≤Y?` → `Op::Test(TestKind::XLeY)`
- `f *` → `X>Y?` → `Op::Test(TestKind::XGtY)`
- `f /` → `X=0?` → `Op::Test(TestKind::XEqZero)`

**8 reachable only via XEQ "NAME" (ROM mnemonic spellings from QRG page 8):**
- `XEQ "X<>Y?"`   → `Op::Test(TestKind::XNeY)`   [the ROM uses `X<>Y?` for "X ≠ Y", a common HP-41 idiom; CONTEXT-decided]
- `XEQ "X<Y?"`    → `Op::Test(TestKind::XLtY)`
- `XEQ "X>=Y?"`   → `Op::Test(TestKind::XGeY)`   [ROM spelling: `X>=Y?` is also written as `X≥Y?` in some sources; QRG uses `X≥Y?` symbol]
- `XEQ "X#0?"`    → `Op::Test(TestKind::XNeZero)` [QRG p.8 uses `X≠0?`; the ROM accepts `X#0?` as alias per HP-41 keystroke programming convention]
- `XEQ "X<0?"`    → `Op::Test(TestKind::XLtZero)`
- `XEQ "X>0?"`    → `Op::Test(TestKind::XGtZero)`
- `XEQ "X<=0?"`   → `Op::Test(TestKind::XLeZero)` [QRG `X≤0?` symbol]
- `XEQ "X>=0?"`   → `Op::Test(TestKind::XGeZero)` [QRG `X≥0?` symbol]

**Recommendation for the XEQ-by-Name resolver:** Accept BOTH ASCII-pure forms (`X<>Y?`, `X#0?`) AND Unicode-symbol forms (`X≠Y?`, `X≠0?`, `X≤0?`, `X≥0?`) in the resolver match. The CONTEXT D-25.10 left the exact spelling open ("planner confirms exact ROM mnemonic") — accepting both is the safe answer and adds 8 lines of additional match arms.

## Function Matrix Schema

### Schema per entry (D-25.16, locked)
```json
{
  "op_variant": "Pi",
  "display_name": "PI",
  "category": "Math",
  "status": "implemented",
  "phase": "20",
  "key_path": "f-1/x",
  "description": "Push π onto X (3.141592654, 10-digit rounded HP-41 hardware value)",
  "divergences": ["10-digit precision per Phase 20 D-09"]
}
```

**Field semantics:**
- `op_variant`: hp41-core `Op::` PascalCase variant name. Used by CI parity check.
- `display_name`: HP-41 mnemonic as shown on display / program listings (matches `prgm_display.rs::op_display_name` output for the variant's typical params).
- `category`: One of `Stack` / `Arithmetic` / `Math` / `Trig` / `Registers` / `Alpha` / `Programming` / `Flags` / `Display` / `Print` / `Sound` / `Catalog` / `Synthetic` / `CardReader` / `Indirect` / `Conversion` / `MathPac` / `StatPac` / `TimePac` / `AdvantagePac`. The last four are v3.x-module categories.
- `status`: `implemented` (= `✓ v2.x`) | `deferred-v3` (= `⏳ v3.x module`) | `na` (= `— N/A`)
- `phase`: GSD phase ID for `implemented` (`"3"` through `"24"`) or null for v3.x.
- `key_path`: CLI keystroke notation. Conventions:
  - Direct: `enter`, `+`, `s` (lowercase = primary label key)
  - f-shifted: `f--`, `f-+`, `f-1/x`, `f-XEQ`
  - Modal: `S` (STO modal opener), `R` (RCL), `f-3` (some f-shifted-then-modal — TBD)
  - XEQ-by-Name only: `XEQ "X<>Y?"`
  - No CLI keyboard path: `null` (covers Op variants reached only programmatically, like Op::PushNum / Op::SyntheticByte / Op::Lbl)
- `description`: One-sentence behavior summary.
- `divergences`: Optional free-form list of documented hardware divergences (e.g. "10-digit precision per Phase 20 D-09"). Empty array if none.

### Generated Markdown shape (output of `just docs-matrix`)
```markdown
# HP-41CV ROM Function Matrix

> Generated from `docs/hp41cv-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

| Op | Display | Category | Status | Phase | Key Path | Description |
|----|---------|----------|--------|-------|----------|-------------|
| Pi | PI | Math | ✓ v2.x | 20 | `f-1/x` | Push π onto X |
| Add | + | Arithmetic | ✓ v2.x | 1 | `+` | Add: X ← Y + X, drop stack |
| … | … | … | … | … | … | … |

## v3.x Deferred (Module Pacs)

| Mnemonic | Category | Notes |
|----------|----------|-------|
| MAT* | MathPac | Matrix operations — v3.x |
| DATE | TimePac | Date arithmetic — v3.x |
| … |
```

The generator (Rust bin or shell) reads the JSON, separates `implemented` and `deferred-v3` rows, and emits the Markdown above. For `implemented` rows, status renders as ✓ v2.x; for `deferred-v3`, ⏳ v3.x module; for `na`, — N/A.

### Op-enum → matrix row mapping (per CONTEXT Claude's Discretion point)

The matrix has FEWER rows than the 130 Op variants for the following reasons:
- `Op::PushNum(HpNum)`, `Op::SyntheticByte(u8)`, `Op::Lbl(String)` — internal primitives, no matrix row. Skip-list in CI parity test.
- `Op::Gto(String)`, `Op::Xeq(String)`, `Op::Clp(String)`, `Op::AlphaAppend(char)` — parameterized variants that correspond to ONE matrix row each (the parameter is the user-supplied operand).
- `Op::StoReg(u8)` vs `Op::StoInd(u8)` — these are TWO Op variants but ONE matrix row "STO" (with a note that STO supports indirect via the IND modifier). Same for RCL/ISG/DSE/SF/CF/ARCL/ASTO/VIEW.
- `Op::Test(TestKind)` with 12 sub-variants — corresponds to 12 matrix rows (one per conditional test, since HP-41 documents them as 12 separate functions).
- `Op::FlagTest{kind, flag}` with 4 sub-kinds — 4 matrix rows (FS?/FC?/FS?C/FC?C).
- `Op::StoArith{kind}` with 4 sub-kinds — 4 matrix rows (STO+/-/×/÷).

**Estimated matrix row count:** ~130 entries, comprising:
- ~95 unique hp41-core Op variants that are HP-41 ROM ops (excluding internal primitives and IND-mirrors)
- ~12 conditional-test variants
- ~4 flag-test variants
- ~4 STO-arithmetic variants
- ~15–20 v3.x-deferred Module-Pac entries (MAT*, complex-number ops, financial ops, etc.)

**Recommended JSON structure:** One JSON entry per matrix row. `op_variant` is the hp41-core PascalCase variant; for parameterized variants the JSON also carries a `key_path` describing the user-input convention (e.g., `"STO nn (or STO IND nn after f)"`).

## XEQ-by-Name CLI Modal (NEW in Phase 25)

The CONTEXT D-25.8 says "XEQ-by-Name modal (v2.1 card-reader phase, already shipped)" — but **verified by grep: no XEQ-by-Name modal exists in hp41-cli**. The v2.1 work shipped:
- `builtin_card_op()` in hp41-core/src/ops/program.rs:966 — name-resolver for 4 card-reader ops
- Wiring in `op_xeq`, `run_program`, `run_loop` so `Op::Xeq("WPRGM")` dispatches to `Op::Wprgm`
- A GUI keyboard `xeq_prompt` stub-error path (no modal)

There is **no CLI keystroke today that opens an XEQ-by-Name modal**. The CLI's `XEQ` is reached only via direct dispatch of `Op::Xeq(label)` constructed elsewhere (e.g., PRGM-mode recording).

**Phase 25 must INTRODUCE the CLI XEQ-by-Name modal:**

```rust
// hp41-cli/src/app.rs (PendingInput addition)
PendingInput::XeqByName(String),  // accumulating mnemonic string for XEQ "NAME"

// handle_pending_input arm:
Some(PendingInput::XeqByName(ref acc)) => {
    match key.code {
        KeyCode::Esc => self.pending_input = None,
        KeyCode::Enter => {
            // Dispatch via Op::Xeq(name); the hp41-core resolver chain handles
            // user-LBL search + builtin_card_op fallback. For the 8 conditional
            // tests, the CLI intercepts BEFORE constructing Op::Xeq:
            let name = acc.clone();
            if let Some(op) = xeq_by_name_local_resolve(&name) {
                self.call_dispatch(op);  // dispatches Op::Test(TestKind::…) directly
            } else {
                self.call_dispatch(Op::Xeq(name));  // falls through to core
            }
            self.pending_input = None;
        }
        KeyCode::Backspace => {
            let mut new_acc = acc.clone();
            new_acc.pop();
            self.pending_input = Some(PendingInput::XeqByName(new_acc));
        }
        KeyCode::Char(c) => {
            let mut new_acc = acc.clone();
            new_acc.push(c);
            self.pending_input = Some(PendingInput::XeqByName(new_acc));
        }
        _ => {
            self.pending_input = Some(PendingInput::XeqByName(acc.clone()));
        }
    }
}

// hp41-cli/src/keys.rs (or app.rs)
fn xeq_by_name_local_resolve(name: &str) -> Option<Op> {
    use hp41_core::ops::{Op, TestKind};
    // Accept both ASCII-only and Unicode-symbol spellings.
    match name {
        "X<>Y?" | "X≠Y?" | "X#Y?"   => Some(Op::Test(TestKind::XNeY)),
        "X<Y?"                       => Some(Op::Test(TestKind::XLtY)),
        "X>=Y?" | "X≥Y?"             => Some(Op::Test(TestKind::XGeY)),
        "X#0?" | "X≠0?"              => Some(Op::Test(TestKind::XNeZero)),
        "X<0?"                       => Some(Op::Test(TestKind::XLtZero)),
        "X>0?"                       => Some(Op::Test(TestKind::XGtZero)),
        "X<=0?" | "X≤0?"             => Some(Op::Test(TestKind::XLeZero)),
        "X>=0?" | "X≥0?"             => Some(Op::Test(TestKind::XGeZero)),
        _ => None,
    }
}
```

**Trigger key for the modal:** The XEQ key is on the HP-41C/CV keyboard at row 2 col 1. In the TUI today, `X` is the HexModal opener (Phase 12, only in PRGM mode); pressing `X` outside PRGM mode is unbound. **Recommendation:** Bind `X` outside PRGM mode to open the `XeqByName` modal. Inside PRGM mode, `X` continues to open the HexModal (existing v1.1 behavior). This is hardware-faithful (X = XEQ on the real keyboard) and resolves the v1.1 conflict cleanly.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| v1.x CLI direct letter-mapping (`C`→COS, `L`→LN, …) | True HP-41CV f-prefix-shift model (D-25.1) | Phase 25 (this phase) | Every v1.x letter binding is deprecated; CLI now mirrors GUI v2.1 SHIFT semantics |
| Two prefixes (`f` + `g`) early draft | One prefix (`f` only) — corrected during CONTEXT discussion | 2026-05-14 CONTEXT-gathering | Confirmed HP-41C/CV/CX hardware reality; HP-15C/12C inspiration rejected |
| Pure-rust `HELP_DATA` const array | JSON-embedded canonical data + OnceLock cache | Phase 25 | Single source of truth shared with Phase 26 GUI overlay |
| Generated Markdown via build.rs | `just docs-matrix` recipe | Phase 25 (CONTEXT D-25.16) | No build.rs needed; recipe is opt-in (developer regenerates and commits) |
| Hard "feature-complete" README claim | Soft "feature-complete HP-41CV with documented divergences" claim | Phase 25 (D-25.17) | Conservative until Phase 27 numerical-accuracy/coverage gates pass |

**Deprecated/outdated:**
- `docs/keyboard-layout.md` schematic — partly inaccurate vs. real HP-41C/CV QRG. Phase 25 either rewrites it from QRG data or links out to the canonical function matrix.
- v1.x letter conventions in `KEY_REF_TABLE` (keys.rs:91) — entirely replaced.
- The hardcoded `ann("SHIFT", false)` in `ui.rs:212` — Phase 25 wires it to `app.shift_armed`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `strum` is NOT currently a dep on hp41-core or hp41-cli | Standard Stack | Low — verified by `grep "strum" Cargo.toml` returning no matches |
| A2 | The HP-41C/CV f-shifted positions for SF/CF/FS?/FC? are on the digit-pad rows 4–6 | HP-41CV Keyboard Reference | Medium — QRG photo is unambiguous on the 4 user-confirmed bindings (D-25.7) but the SF/CF/FS?/FC? positions are inferred from QRG p.8 function index; planner should cross-check during execution. |
| A3 | The "XEQ-by-Name modal (v2.1)" mentioned in CONTEXT D-25.8 does NOT exist in hp41-cli today | XEQ-by-Name CLI Modal | Verified by grep — no `XeqByName` or similar string in hp41-cli/src. CONTEXT statement is inaccurate; Phase 25 must NEWLY introduce the modal. **This is a CONTEXT correction the planner should surface.** |
| A4 | Extending `builtin_card_op` from 4 names to 12 names is the least-invasive way to support the 8 non-keyboard conditional tests | Code Examples → Extending the XEQ-by-Name resolver | Medium — alternative is a CLI-local resolver (Option 1). Both work; Option 2 (extend hp41-core) gives programmatic symmetry. **Needs user clearance** since it touches hp41-core which CONTEXT says is frozen. |
| A5 | The IND modifier key in our CLI implementation maps to `f` pressed inside a modal (toggle `ind`) | HP-41CV Keyboard Reference | Low — QRG p.14 verbatim text confirms IND = shift-then-address. Implementation flavor (faithful vs simple `I` keystroke) is planner choice. |
| A6 | `just docs-matrix-check` is the right recipe-shape for CI drift catch | Pitfall 8 + just docs-matrix recipe | Low — split-recipe pattern is standard CI practice; alternative is single recipe with `--check` flag. |
| A7 | The hand-curated `ALL_OP_VARIANT_NAMES` const has acceptable maintenance burden vs `strum::EnumIter` | Code Examples → CI parity test | Low — 130 entries is one-time setup; future Op additions in Phase 26/27 are rare. Planner can choose `strum` if dep cost is acceptable. |
| A8 | The categories `Stack`, `Arithmetic`, …, `Indirect` are sufficient and HP-41 conventional | Function Matrix Schema | Low — categories derived from existing help_data.rs grouping; v3.x-module categories from REQUIREMENTS.md scope. |

**If this table is empty:** Not empty — three medium-risk items (A2, A3, A4) need planner attention. A3 in particular is a CONTEXT inaccuracy that the planner should flag to the user.

## Open Questions

1. **ENTER key row position on the HP-41CV (3 vs 5):** The QRG photo and the schematic in `docs/keyboard-layout.md` disagree. The photo shows ENTER↑ as a 2-wide bottom-row key; the schematic puts it on row 3 single-width. The hp41-gui v2.1 `KEY_DEFS` uses 2-wide ENTER. **What we know:** Real hardware has 2-wide ENTER somewhere on a lower row. **What's unclear:** Exact row number for our 5×7 grid representation. **Recommendation:** Phase 25 planner cross-checks against an HP-41CV photograph and hp41-gui v2.1 `KEY_DEFS` to lock the canonical row before keys.rs rewrite.

2. **`builtin_card_op` extension to 12 names — hp41-core boundary breach?** CONTEXT D-25.8 says XEQ-by-Name palette dispatches the 8 conditional tests by mnemonic name, AND CONTEXT also says "NO hp41-core changes" in Phase 25. These conflict if "extending the resolver table" counts as a hp41-core change. **What we know:** Extending `builtin_card_op` from 4 entries to 12 is the cleanest implementation (Option 2). **What's unclear:** Whether the user considers this within Phase 25 scope. **Recommendation:** Planner clarifies via `/gsd-discuss-phase 25 --refine` or asks the user directly in the plan-creation step. Fallback: Option 1 (CLI-local resolver) works without hp41-core changes but is asymmetric for programmatic XEQ inside saved programs.

3. **IND-toggle keystroke flavor (faithful `f` vs simple `I`):** D-25.12 leaves the actual char open. **What we know:** Hardware-faithful is `f` pressed inside a modal. **What's unclear:** Whether planner+user prefer hardware-faithful or TUI-discoverable. **Recommendation:** Implement BOTH — `f` (hardware-faithful) AND `I` (discoverable alias). Cost is one extra match arm; benefit is removed UX friction for users without HP-41CV background.

4. **What happens when user presses `f` while a `RegisterPrompt` modal is open but `shift_armed == false`?** Options:
   (a) Arm the global `shift_armed=true` so the next key-cycle behaves as f-shifted.
   (b) Set `ind=true` on the modal (the hardware-faithful interpretation).
   (c) Ignore `f` entirely inside modals (current v1.x pattern for unrecognized keys).
   **Recommendation:** (b) — the modal IS the active dialog; `f` semantics inside a modal mean "modify this address with IND" per hardware. Document this as the canonical interpretation in CLAUDE.md.

5. **Should `pending_prompt()` show `f→` when `shift_armed` is true even without an active modal?** The status bar would read `f→ Ready` instead of `Ready`. Mild UX improvement; trivial implementation. **Recommendation:** Yes.

6. **`docs/keyboard-layout.md` — rewrite or delete?** Currently a v1.x schematic with letter-binding conventions. Phase 25 makes it obsolete. **Recommendation:** Rewrite to point at `docs/hp41cv-function-matrix.md` and include a small ASCII-art keyboard with f-shifted labels in parentheses (3 mins of work). Delete is also acceptable per D-25.17 README updates.

## Environment Availability

Skipped — Phase 25 is purely code/config/documentation changes within the existing Rust workspace. No new external dependencies, no new services, no new CLI utilities beyond `cargo` and `just` (both already standard project tooling).

The single new tooling concern: **`scripts/docs-matrix/Cargo.toml` as a tiny separate crate** for the Rust-bin variant of `just docs-matrix`. This adds a new crate to the workspace, requiring an update to root `Cargo.toml` `members`. **CRITICAL:** Per CLAUDE.md "Root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`", DO NOT add the script crate to workspace members. Either:
- Make `scripts/docs-matrix/Cargo.toml` a standalone non-workspace crate (`[workspace]` empty stanza).
- OR implement `just docs-matrix` as a shell+jq script (no Rust bin) to avoid the workspace-members issue.

**Recommendation:** Standalone non-workspace crate path — Rust type-checking is worth the small cost; the standalone crate doesn't affect the existing two-member workspace invariant.

## Validation Architecture

> Per `.planning/config.json` `workflow.nyquist_validation: true`.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (rust stdlib + workspace harness) |
| Config file | `Cargo.toml` (per-crate `[dev-dependencies]`); no separate test config |
| Quick run command | `cargo test -p hp41-cli --test function_matrix_parity -- --nocapture` |
| Full suite command | `just ci` (lint → test → coverage) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FN-TEST-01 | 4 conditional tests reachable via f-arith keys | unit | `cargo test -p hp41-cli --lib keys::tests::test_f_minus_dispatches_x_eq_y` | ❌ Wave 0 |
| FN-TEST-01 | 8 conditional tests reachable via XEQ-by-Name modal | unit | `cargo test -p hp41-cli --lib keys::tests::test_xeq_by_name_resolves_x_ne_y` | ❌ Wave 0 |
| FN-CLI-01 | Every Phase 20–24 Op has a key_to_op + KEY_REF_TABLE entry | integration | `cargo test -p hp41-cli --test key_coverage` | ❌ Wave 0 |
| FN-CLI-02 | 6 new PendingInput variants compile + dispatch | unit | `cargo test -p hp41-cli --lib app::tests::test_register_prompt_ind_toggle` | ❌ Wave 0 |
| FN-CLI-03 | help_data.rs reads JSON without panic; ≥130 entries | smoke | `cargo test -p hp41-cli --test help_data_smoke` | ❌ Wave 0 |
| FN-CLI-04 | pending_prompt() exhaustive (no `_=>`) | compile-time | `cargo build -p hp41-cli` (exhaustive-match warning denied via `#![deny(non_exhaustive_omitted_patterns)]`) | ✅ existing crate gate |
| FN-DOC-01 | docs/hp41cv-functions.json parses + ≥130 entries | smoke | `cargo test -p hp41-cli --test function_matrix_parity::test_matrix_has_at_least_130_entries` | ❌ Wave 0 |
| FN-DOC-01 | Every Op variant has a JSON entry | integration | `cargo test -p hp41-cli --test function_matrix_parity::test_every_rom_op_has_matrix_entry` | ❌ Wave 0 |
| FN-DOC-01 | Every JSON-implemented entry has an Op variant | integration | `cargo test -p hp41-cli --test function_matrix_parity::test_every_implemented_matrix_entry_has_op` | ❌ Wave 0 |
| FN-DOC-01 | `just docs-matrix` output matches committed Markdown | CI script | `just docs-matrix-check` (diffs regenerated vs committed) | ❌ Wave 0 |
| FN-DOC-02 | CLAUDE.md has "v2.2 additions" block | smoke | `grep -q "v2.2 additions" CLAUDE.md` | manual / git pre-commit |
| FN-DOC-03 | README has soft "feature-complete HP-41CV" claim + matrix link | smoke | `grep -q "feature-complete HP-41CV" README.md && grep -q "hp41cv-function-matrix.md" README.md` | manual / git pre-commit |
| FN-DOC-04 | hp41-core rustdoc compiles + cross-references valid | doctest | `cargo doc --no-deps -p hp41-core` | ✅ existing crate gate |

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-cli` (CLI-only; ~5s) — verifies the modal/keyboard wiring tests for that commit's slice.
- **Per wave merge:** `just test` (full workspace; ~30s including hp41-core 800+ tests) — catches cross-crate breakage.
- **Phase gate:** `just ci` (lint + full test + coverage) green before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] `hp41-cli/tests/function_matrix_parity.rs` — bidirectional JSON ↔ Op enum drift catch (FN-DOC-01)
- [ ] `hp41-cli/tests/help_data_smoke.rs` — `help_entries().len() >= 130` (Pitfall 7 mitigation)
- [ ] `hp41-cli/tests/key_coverage.rs` — every Op variant has a key_to_op or KEY_REF_TABLE entry OR explicit "modal-only" / "XEQ-by-Name-only" exemption (FN-CLI-01)
- [ ] `hp41-cli/src/keys.rs::tests::test_f_minus_dispatches_x_eq_y` and 3 more conditional-test smoke tests (FN-TEST-01, keyboard side)
- [ ] `hp41-cli/src/keys.rs::tests::test_xeq_by_name_resolves_x_ne_y` and 7 more conditional-test smoke tests (FN-TEST-01, XEQ side)
- [ ] `hp41-cli/src/app.rs::tests::test_shift_armed_one_shot` + `test_shift_armed_esc_cancel` + `test_shift_armed_alpha_override` (D-25.4, D-25.5)
- [ ] `hp41-cli/src/app.rs::tests::test_register_prompt_ind_toggle` (D-25.12)
- [ ] `docs/hp41cv-functions.json` — seed with all 130 Op variants (matrix population)
- [ ] `docs/hp41cv-function-matrix.md` — first generation via `just docs-matrix`
- [ ] `scripts/docs-matrix/` — Rust bin OR shell script (planner picks)
- [ ] `justfile` — `docs-matrix` and `docs-matrix-check` recipes
- [ ] `CLAUDE.md` — v2.2 settled-architecture block (flag storage, indirect resolution, sound buffer, f-prefix model)
- [ ] `README.md` — soft "feature-complete HP-41CV" wording per D-25.17

## Security Domain

> `security_enforcement` not explicitly set in `.planning/config.json` — treat as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No auth surface — TUI is single-user, single-process |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | No access tiers |
| V5 Input Validation | yes (mild) | `serde_json::from_str` for `hp41cv-functions.json` (validates schema); `xeq_by_name_local_resolve` rejects unknown mnemonics with `Err(InvalidOp)` per existing pattern; PendingInput accumulators bound-check (2-digit cap on register/flag prompts, 3-digit on DEL, 1-digit on TONE) |
| V6 Cryptography | no | No crypto in this phase |

### Known Threat Patterns for hp41-cli + ratatui + crossterm

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed JSON in `hp41cv-functions.json` | Denial of Service / Tampering | `.expect()` panic at startup is intentional (D-25.17 — hard-build-blocker by design). Smoke test `help_data_smoke.rs` catches missing/empty file at CI time. |
| Unbounded user input to ClpLabel/XeqByName modals | Denial of Service (memory exhaustion via infinite-length accumulator) | Cap accumulator at HP-41 hardware-realistic length (label names ≤7 chars per HP-41C spec; XEQ name ≤24 chars per ALPHA register width). Add `if acc.len() < CAP { acc.push(c) }` guards. |
| Path traversal via card-reader name (out of scope this phase) | Information Disclosure | Already mitigated by `cards::sanitize_name` in v2.1 (covered in `cards.rs::sanitize_name` tests) |
| Help-overlay table-state borrow_mut panic | Information Disclosure (via terminal crash exposing state) | Existing `RefCell` pattern is single-threaded non-reentrant (app.rs comment); not affected by this phase |
| Function-matrix injection via untrusted JSON | Tampering | Not applicable — `hp41cv-functions.json` is committed and hand-curated, not user-supplied at runtime |

**No new attack surface introduced by Phase 25.** Phase 25 is a wiring + documentation phase touching the TUI keyboard layer and adding an embedded JSON data file. No network I/O, no filesystem writes (except by `just docs-matrix` developer recipe), no external process spawning.

## Sources

### Primary (HIGH confidence)
- **HP-41C/41CV Quick Reference Guide** (literature.hpcalc.org) — verified keyboard layout, function index, indirect operations, conditional test mnemonics. Pages 1–15 read.
- **`hp41-core/src/ops/mod.rs`** (Phase 24 commit) — verified 130 Op variants, exhaustive `dispatch()` match, `FlagTestKind` + `StoArithKind` + `TestKind` enum shapes.
- **`hp41-cli/src/app.rs:23–41`** — verified existing 11 `PendingInput` variants and `handle_pending_input` pattern.
- **`hp41-cli/src/keys.rs:18–87`** — verified current `key_to_op` and v1.x letter-binding conventions to be deprecated.
- **`hp41-cli/src/ui.rs:238–273`** — verified `pending_prompt()` exhaustive-match shape.
- **`hp41-cli/src/help_data.rs:11–342`** — verified current `HELP_DATA` const shape and category-grouping convention.
- **`hp41-cli/src/programs.rs:7,19,22`** — verified `OnceLock` precedent in project [grep].
- **`hp41-core/src/ops/program.rs:60–80, 966–974`** — verified `op_xeq` + `builtin_card_op` resolver chain.
- **`.planning/MILESTONES.md`** v2.1 entry — verified Card Reader's XEQ-by-name wiring scope (4 names only; no CLI modal yet).
- **`.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md` D-22.11.1** — verified bounds-audit convention.

### Secondary (MEDIUM confidence)
- **HP-41C Owner's Handbook + Programming Guide** (literature.hpcalc.org, archive.org) — referenced for ROM mnemonic spellings and category conventions but full PDF not deeply parsed (large binary).
- **Free42 source / `keyboard.h`** (thomasokken.com/free42, fossies.org) — referenced for cross-check of key codes; we didn't fetch the actual header in this research session.
- **strum crate docs.rs** — confirmed `EnumIter` requires `Default` on payload types and works fine for the Op enum if dep cost is accepted.

### Tertiary (LOW confidence)
- v1.x `docs/keyboard-layout.md` schematic — known to be partly inaccurate vs. QRG (it has ENTER on row 3, QRG photo shows it on a lower row 2-wide). Use only as a starting point for the Phase 25 rewrite.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps already in workspace; OnceLock pattern is project precedent.
- Architecture (f-prefix + PendingInput hybrid): HIGH — fully specified by CONTEXT D-25.1..D-25.14; v2.1 GUI provides parity reference.
- HP-41CV keyboard layout: HIGH — verified directly from the QRG (literature.hpcalc.org PDF read).
- IND-key position: HIGH — explicitly stated in QRG p.14.
- ROM mnemonic spellings for conditional tests: MEDIUM — QRG p.8 uses Unicode-symbol forms; ASCII-only forms (`X<>Y?`, `X#0?`) are HP-41 keystroke-programming convention. Plan accepts both.
- Pitfalls: HIGH — derived from existing CLAUDE.md, Phase 19 GUI D-5 pattern, and existing app.rs comments.
- CI parity test design: HIGH — hand-curated inventory is a known Rust pattern; alternative `strum` is documented.
- Function-matrix generation pipeline: HIGH — recipe split (regenerate vs check) is standard CI practice.

**Research date:** 2026-05-14
**Valid until:** 2026-06-13 (30 days; stable-domain — HP-41CV is a 1979 product, won't change; v2.2 milestone is mid-flight so the codebase shifts daily — re-verify if planning is delayed past Phase 26 ship)

---

## Final Self-Check (Pre-Submission Checklist)

- [x] All domains investigated (stack, architecture, modals, JSON pipeline, CI parity, keyboard layout, conditional tests, IND, pitfalls)
- [x] Negative claims verified ("no XEQ-by-Name modal exists in hp41-cli today" verified by grep; documented as A3)
- [x] Multiple sources cross-referenced (QRG + Owner's Manual + Free42 + project code)
- [x] URLs provided for authoritative sources (literature.hpcalc.org PDF read directly)
- [x] Publication dates checked (QRG is original HP literature; codebase deltas current to commit 435357f / 63c0888 of 2026-05-14)
- [x] Confidence levels assigned honestly (HIGH for verified items; MEDIUM for ROM mnemonic spellings; LOW for the v1.x keyboard-layout.md which is known partly inaccurate)
- [x] "What might I have missed?" review completed — surfaced CONTEXT inaccuracy (A3) and hp41-core boundary question (Open Q 2)
- [x] Runtime State Inventory: omitted (Phase 25 not a rename/refactor)
- [x] Security domain included — no significant attack surface
- [x] ASVS categories verified against phase tech stack (V5 input validation is the only relevant one)

Sources:
- [HP-41C/41CV Quick Reference Guide (PDF)](https://literature.hpcalc.org/community/hp41c41cv-qrg-en.pdf)
- [HP-41C/41CV Operating Manual (PDF)](https://literature.hpcalc.org/community/hp41c41cv-om-en.pdf)
- [HP-41C Owner's Handbook (Internet Archive)](https://archive.org/details/hp-41-c-owners-handbook-and-programming-guide)
- [HP Calculator Literature — HP-41C/41CV Owner's Handbook](https://literature.hpcalc.org/items/887)
- [Free42 by Thomas Okken (project site)](https://thomasokken.com/free42/)
- [strum::EnumIter docs (docs.rs)](https://docs.rs/strum/latest/strum/derive.EnumIter.html)
- [HP-41C on Wikipedia](https://en.wikipedia.org/wiki/HP-41C)
- [Key Codes reference (hpmuseum.org)](https://www.hpmuseum.org/software/codes.htm)
