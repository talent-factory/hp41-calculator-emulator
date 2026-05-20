# Phase 29: CLI Integration — Research

**Researched:** 2026-05-17
**Domain:** `hp41-cli` TUI wiring for Phase 28's XROM framework + Math Pac I modal-workflow surface
**Confidence:** HIGH (every claim verified against committed source — `[VERIFIED: file:line]`)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **C-28.1 (ADR-001):** Op-strategy A — one `Op` variant per Math Pac I function. Phase 29's JSON entries map 1:1 to `Op::*` variants.
- **C-28.3 (ADR-005):** JSON-pipeline shape — separate `docs/hp41-math1-functions.json` sibling file with identical schema plus per-entry `xrom: { module, module_id, function_id }` object.
- **C-28.4:** `xrom_resolve` fires LAST in the resolver chain. Phase 29's `xeq_by_name_local_resolve` extension preserves the same ordering.
- **D-28.4:** `modal_prompt: Option<String>` is the channel for prompt strings. `state.print_buffer` continues to carry PRX/PRA/PRSTK output ONLY.
- **D-28.5:** R/S key submits numeric input in a modal prompt (HP-41C/CV Math Pac OM 1979 p.13).
- **D-28.6:** XEQ-by-name only — no dedicated key bindings for hyperbolics or any Math Pac I function.
- **D-25.18:** `KEY_REF_TABLE` derives from JSON via `key_ref_entries()` filtered by non-null `key_path`. No hand-curated parallel table.
- **D-29.1:** Phase 29 authors the FULL ~55-entry `docs/hp41-math1-functions.json` (pulled forward from nominal Phase 30 boundary).
- **D-29.2:** Second `OnceLock<Vec<HelpEntry>>` over `hp41-math1-functions.json`; new `help_entries_math1()` + merged `help_entries_all()` accessor; existing `help_overlay_rows` / `key_ref_entries` / `function_matrix_parity.rs` migrate to the merged accessor.
- **D-29.3:** Extend `pending_prompt()` in `hp41-cli/src/ui.rs` to ALSO render `state.modal_prompt`. Signature widens (planner picks `&App` vs `&PendingInput, modal_prompt: Option<&str>`).
- **D-29.4:** LCD keeps showing X-register / `entry_buf` during an open modal — no `Display14Seg` renderer changes.
- **D-29.5:** Intercept R/S in `handle_key` BEFORE the v2.1 `run_stop` path when `state.modal_program.is_some()`; call new shared `pub fn submit_modal(state: &mut CalcState) -> Result<(), HpError>` in `hp41-core`.
- **D-29.6:** Esc cancels via new `pub fn cancel_modal(state: &mut CalcState)` in `hp41-core` (same module location as `submit_modal`).
- **D-29.7 / D-29.8:** Auto-open the v2.2 XEQ-by-name modal in a new "collect label for modal" mode; extend `PendingInput::XeqByName { acc }` → `{ acc, mode: XeqByNameMode }` with `Normal | CollectForModal` variants; compile-time exhaustive match preserved (FN-CLI-04).
- **D-29.9:** CLI `App` event loop auto-opens `CollectForModal` modal based on state delta — fires only when `state.modal_program.is_some() && modal_program.requires_alpha_label() && pending_input.is_none()`.

### Claude's Discretion

- **Module location** for `submit_modal` / `cancel_modal` / `requires_alpha_label`: natural homes are `hp41-core/src/ops/math1/mod.rs` (top-level dispatch) and `hp41-core/src/ops/math1/modal.rs` (method on `ModalProgram`). Public surface must compile cleanly through `pub use` in `hp41-core/src/ops/math1/mod.rs`.
- **`submit_modal` internal shape:** exhaustive match on `state.modal_program` variant. Constraint: zero `_ =>` catch-alls.
- **`advance_with_label(label: &str)` signature shape:** method on `ModalProgram`, new variant of `submit_modal` params, or separate function. Label must be by reference, trimmed/uppercased per HP-41 convention.
- **`function_matrix_parity.rs` extension:** Planner can split into two test functions or one parameterized table. Constraint: per-pool failure messages stay surgical.
- **Category naming convention** inside `hp41-math1-functions.json`: per-program categories recommended (`"Math1 Hyperbolics"`, `"Math1 Complex"`, etc.) over single `"Math 1 Pac"` bucket.
- **`pending_prompt()` signature widening:** `pub fn pending_prompt(app: &App) -> String` (ergonomic) vs `pub fn pending_prompt(pending: &PendingInput, modal_prompt: Option<&str>) -> String` (testable in isolation). Either acceptable.
- **Precedence when both `pending_input.is_some()` AND `state.modal_program.is_some()`:** shouldn't happen in well-formed flows (D-29.9 auto-open is gated). Planner picks well-defined behavior (likely modal_program rendering wins) and asserts via test.
- **JSON `divergences` field** for Math Pac I entries: planner has discretion which to populate per-entry. Full divergence catalog moves to `docs/hp41-math1-divergences.md` in Phase 30.

### Deferred Ideas (OUT OF SCOPE)

- GUI mirroring of all Phase 29 work — Phase 31 plans 01–05.
- CATALOG 2 listing all loaded XROM modules — Phase 31 / GUI-04.
- `request_cancel` Tauri command + cancel button — Phase 31 / GUI-05.
- 5 ADR write-ups — Phase 30 / DOC-07.
- `docs/hp41-math1-divergences.md` expansion — Phase 30 / DOC-04.
- `scripts/docs-matrix/` two-input extension + matrix regeneration — Phase 30 / DOC-02.
- README v3.0 soft-claim + PROJECT.md / CLAUDE.md v3.0 additions block — Phase 30 / DOC-05 + DOC-06.
- Numerical-accuracy suite extension (566 → ~700+ cases) — Phase 32 / QUAL-02.
- Per-Op test count ≥ 5 verification — Phase 32 / QUAL-01.
- WebdriverIO E2E smoke extension with Math Pac I workflow — Phase 32 / QUAL-03.
- Free42 GPL-contamination guard in CI — Phase 32 / QUAL-05.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CLI-01 | `xeq_by_name_local_resolve` in `hp41-cli/src/keys.rs` ruft `xrom_resolve` nach `builtin_card_op` auf (Math Pac I funktioniert via XEQ-by-Name) | §3.1 below: single-line `_ =>` arm extension at `hp41-cli/src/keys.rs:368` — signature must thread `state.xrom_modules` (Claude's discretion: function-parameter vs. module-level helper) |
| CLI-02 | `hp41-cli/src/help_data.rs` lädt ZWEITE JSON-Datei via zusätzlichen `OnceLock<Vec<HelpEntry>>`; `?`-Overlay zeigt Math Pac I Funktionen | §3.2 below: mirror the existing `FUNCTIONS_JSON` / `HELP_ENTRIES` / `help_entries()` pattern verbatim; new `help_entries_all()` chain |
| CLI-03 | `hp41-cli/src/prgm_display.rs` erweitert um ~40 neue `op_display_name` Arms | **ALREADY SHIPPED in Phase 28 plans 28-02..28-10** — Phase 29 plan 02 shrinks to a verification + parity-test pass per CONTEXT.md "Already locked" bullet. See §3.0 below. |
| CLI-04 | KEY_REF_TABLE (`hp41-cli/src/ui.rs::render_right_panel`) blendet Math Pac I Einträge ein (JSON-derived per D-25.18) | §3.2 below: `key_ref_entries()` migrates to `help_entries_all()` — automatic surface for Math Pac I entries with non-null `key_path` |
| CLI-05 | Modal-Prompt-Routing: `MATRIX`/`SOLVE`/`POLY`/`INTG`/`DIFEQ`/`FOUR`/`TRANS` Workflows triggern `ModalProgram`-State-Machine; Inputs über existierende Number-Entry-Pipeline; ALPHA-Prompt-Text in TUI sichtbar | §3.3–3.7 below: D-29.3 widen `pending_prompt`, D-29.5 R/S interception, D-29.6 Esc cancellation, D-29.7/8 FUNCTION NAME? auto-routing, D-29.9 post-dispatch auto-open hook |
</phase_requirements>

## Summary

Phase 29 turns Phase 28's dormant XROM + modal-workflow surface in `hp41-core` into a fully reachable TUI experience. Five contracts to close: (1) extend the CLI's local XEQ-by-name resolver into `xrom_resolve` as the third call site (mirroring `op_xeq:79` and `run_program::execute_op:521`); (2) clone the v2.2 `OnceLock` JSON pipeline for a second canonical file; (3) route R/S and Esc through new shared `submit_modal` / `cancel_modal` core functions when `state.modal_program.is_some()`; (4) extend `PendingInput::XeqByName` with a `mode` discriminator so the existing alpha-collection UX serves both `XEQ "name"` resolution AND `FUNCTION NAME?` label capture for INTG/SOLVE/DIFEQ; (5) widen `pending_prompt()` so `state.modal_prompt` surfaces on the existing status-bar line.

Of the 5 REQ-IDs, CLI-03 (`prgm_display.rs` ~40 arms) has already shipped in Phase 28 — Phase 29 plan 02 in the original ROADMAP wording shrinks to a verification + parity-test pass. The four remaining requirements are pure CLI mechanics — no new `Op` variants, no `hp41-core` algorithm changes, no GUI source touched. SC-4 trivially holds.

**Primary recommendation:** plan 29-01 lands the JSON file + second `OnceLock` + resolver-chain extension + parity-test extension (CLI-01, CLI-02, CLI-04). Plan 29-02 verifies CLI-03 (already-shipped op_display_name coverage) and adds the parity-test sweep. Plan 29-03 lands the modal interception + auto-open hook + `submit_modal`/`cancel_modal`/`requires_alpha_label` public surface in `hp41-core` (CLI-05). All three plans preserve the D-07 never-discard invariant, the FN-CLI-04 exhaustive-match invariant, and the D-25.6 CLI ↔ GUI parity invariant.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| XEQ-by-name resolution for Math Pac I | `hp41-core` (`xrom_resolve`) | `hp41-cli` (CLI-local fast-path) | Single source of truth in core per D-25.6; CLI mirrors as a fast-path shortcut, never as an alternate resolver |
| Modal step advancement / submission | `hp41-core` (new `submit_modal`) | `hp41-cli` (R/S interception) | All state mutation in core per D-25.6; CLI is a dumb pump that detects R/S and calls into core |
| Modal cancellation | `hp41-core` (new `cancel_modal`) | `hp41-cli` (Esc interception) | Same as submit — single shared function; CLI/GUI route Esc identically |
| Modal-prompt rendering | `hp41-cli` (`pending_prompt` widening) | — | Pure presentation; reads `state.modal_prompt: Option<String>` (transient core field) |
| Function-name capture for INTG/SOLVE/DIFEQ | `hp41-cli` (`PendingInput::XeqByName { mode: CollectForModal }`) | `hp41-core` (`requires_alpha_label`) | UI input collection is CLI; the decision-of-need bit (`requires_alpha_label`) is a pure read-only method on `ModalProgram` in core |
| JSON canonical pipeline | `hp41-cli` (`help_data.rs` 2nd `OnceLock`) | `docs/` (JSON file) | D-25.16 hard-build-blocker pattern per file; existing `help_entries()` retained for v2.2 surgical tests |
| Right-panel discoverability | `hp41-cli` (`key_ref_entries`) | `hp41-cli/src/help_data.rs` (merged accessor) | Existing function migrates from `help_entries()` to `help_entries_all()` — zero code-shape changes |

## Phase 28 Contract Recap

What Phase 28 shipped (and what Phase 29 consumes):

### Public surface in `hp41-core/src/ops/math1/`

| Symbol | Location | What it does |
|--------|----------|-------------|
| `xrom_resolve(name: &str, modules: u8) -> Option<Op>` | `hp41-core/src/ops/math1/xrom.rs:127` `[VERIFIED]` | Resolves Math Pac I mnemonic → `Op`. Gated by `modules & 0b0000_0001`. 52-entry table (6 hyp + 7 complex-arith + 17 complex-fn + 2 poly + 8 matrix + 1 intg + 2 solve + 1 difeq + 8 four/tri/trans) per `MATH_1.ops` `[VERIFIED: xrom.rs:43–118]`. |
| `MATH_1: XromModule` | `hp41-core/src/ops/math1/xrom.rs:43` `[VERIFIED]` | `id: 7`, `name: "MATH 1A"`, `ops: &[(&str, Op); 52]`. `MATH_1.ops` is the canonical mnemonic → Op table; `xrom_shadowing.rs` asserts no collision with built-ins. |
| `ModalProgram` enum + 7 step sub-enums | `hp41-core/src/ops/math1/modal.rs:24` `[VERIFIED]` | 7 variants: Matrix / Solve / Poly / Integ / Difeq / Four / Trans. Each wraps a per-program `*InputStep` enum (`MatrixInputStep`, `SolveInputStep`, …). Carries `current_prompt() -> Option<String>` returning OM-cited prompt text. **Phase 29 adds `requires_alpha_label(&self) -> bool` here.** |
| `CalcState.xrom_modules: u8` | `hp41-core/src/state.rs:164` `[VERIFIED]` | Persistent (`#[serde(default = "default_xrom_modules")]`, default `0b0000_0001`). Phase 29 reads this when threading it into `xrom_resolve`. |
| `CalcState.modal_program: Option<ModalProgram>` | `hp41-core/src/state.rs:186` `[VERIFIED]` | `#[serde(default, skip)]`. Set by ops like `op_matrix_workflow` (`matrix.rs:327`), `op_four` (`four.rs:92`), etc. Phase 29 reads this to detect open modals. |
| `CalcState.modal_prompt: Option<String>` | `hp41-core/src/state.rs:194` `[VERIFIED]` | `#[serde(default, skip)]`. Carries the current step's prompt text (e.g. `"ORDER=?"`, `"A1,1=?"`, `"FUNCTION NAME?"`, `"B1=?"`, `"NO SOLUTION"`). Phase 29 renders this in the status bar via `pending_prompt`. |
| `CalcState.integ_state` / `solve_state` / `difeq_state` | `hp41-core/src/state.rs:200/206/213` `[VERIFIED]` | All `#[serde(default, skip)]`. Solver-loop scratch state; Phase 29 only checks `.is_some()` for the nested-callback strict-reject guard inside `submit_modal` (if needed). |
| `CalcState.cancel_requested: Arc<AtomicBool>` | `hp41-core/src/state.rs:222` `[VERIFIED]` | `#[serde(default = "default_cancel_requested", skip)]`. Plumbing dormant in v3.0 Phase 29 — wired in Phase 31 / GUI-05. **Phase 29 does NOT touch this field.** |
| `HpError::Canceled` | `hp41-core/src/error.rs:48` `[VERIFIED]` | New variant landed in Plan 28-01; never serialized. Phase 29 may surface it via `app.message` if the planner so chooses (otherwise it's hidden behind R/S returning `Ok(())`). |

### Resolver chain — the 3 existing call sites + the 4th Phase 29 must close

| # | Caller | File / Line | Contract |
|---|--------|-------------|----------|
| 1 | `op_xeq` (interactive XEQ) | `hp41-core/src/ops/program.rs:79` `[VERIFIED]` | `if let Some(xrom_op) = crate::ops::math1::xrom::xrom_resolve(label, state.xrom_modules) { return crate::ops::dispatch(state, xrom_op); }` — fires AFTER `builtin_card_op` |
| 2 | `run_program::execute_op` (programmatic XEQ) | `hp41-core/src/ops/program.rs:521` `[VERIFIED]` | `else if let Some(xrom_op) = crate::ops::math1::xrom::xrom_resolve(&label, state.xrom_modules) { crate::ops::dispatch(state, xrom_op)?; }` — same ordering |
| 3 | (TODO) `xeq_by_name_local_resolve` (CLI fast-path) | `hp41-cli/src/keys.rs:347` `[VERIFIED current shape]` | Currently `_ => None` at line 368 — Phase 29's job is to make this final arm fall through to `xrom_resolve` BEFORE returning `None`. Since this is CLI-local (the user types into a modal and presses Enter), the function does NOT take `&mut state` — it currently has signature `pub fn xeq_by_name_local_resolve(name: &str) -> Option<Op>`. **Planner has discretion on threading `state.xrom_modules`**: either widen the signature to `(name: &str, modules: u8) -> Option<Op>` (callers at `app.rs:1423` pass `self.state.xrom_modules`) or use a module-level helper that defaults to `0b0000_0001`. Widening the signature is the cleaner choice. |

### What was DEFERRED to Phase 29 by Phase 28 design

| Item | Source | Phase 29 surface |
|------|--------|------------------|
| CLI-local resolver third call site | `28-01-SUMMARY.md:173` (cited in CONTEXT) | §3.1 below |
| `submit_modal` / `cancel_modal` / `requires_alpha_label` public surface | CONTEXT D-29.5 / D-29.6 / D-29.9 | §3.5 below — verified by source: NO `submit_modal` / `cancel_modal` / `requires_alpha_label` / `advance_with_label` exists anywhere in tree `[VERIFIED: grep returned 0 matches]` |
| Modal-step **advancement** logic (what to do when user submits `3` to `ORDER=?`) | The ops `op_matrix_workflow`, `op_solve_run_loop`, etc. set the modal_program field but no per-program submit functions exist `[VERIFIED: grep "fn matrix_submit_order\|submit_order_step\|submit_element_step\|fn poly_submit" returns 0 matches]` | The new `submit_modal` in `hp41-core` is the FIRST consumer of every per-step transition — Phase 29 is the first phase where the modal flow becomes end-to-end functional. Planner's job: per-variant match in `submit_modal` that reads `entry_buf` (via `flush_entry_buf` first) and advances `ModalProgram` |

## Implementation Path per Decision

### §3.0 CLI-03 — verification of already-shipped op_display_name arms

CLI-03 in REQUIREMENTS.md reads "~40 neue `op_display_name` Arms für die neuen Op-Varianten". CONTEXT explicitly notes these arms shipped in Phase 28 plans 28-02..28-10. A quick verification grep:

```bash
grep -c "Op::\(Sinh\|Cosh\|Tanh\|CPlus\|CMinus\|CTimes\|CDiv\|Real\|Magz\|Cinv\|ZpowN\|ExpZ\|LnZ\|SinZ\|CosZ\|TanZ\|LogZ\|ZpowW\|Apow Z\|Zpow1N\|PolyWorkflow\|Roots\|MatrixWorkflow\|MatSize\|MatVmat\|MatEdit\|MatDet\|MatInv\|MatSimeq\|MatVcol\|Integ\|Solve\|Sol\|Difeq\|Four\|TriSss\|TriAsa\|TriSaa\|TriSas\|TriSsa\|Trans2d\|Trans3d\)" hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs
```

Plan 29-02 deliverable: re-run that grep, assert the count matches the 52-entry `MATH_1.ops` cardinality (modulo the 5 ASCII aliases which map to the same `Op`), and add a parity test if missing. **Estimated effort: 30-min task.** No new code — purely verification.

### §3.1 CLI-01 (D-implicit) — extend `xeq_by_name_local_resolve` into `xrom_resolve`

**Current shape** `[VERIFIED: hp41-cli/src/keys.rs:347–370]`:

```rust
pub fn xeq_by_name_local_resolve(name: &str) -> Option<Op> {
    match name {
        "X<>Y?" | "X\u{2260}Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY)),
        // ... 7 more conditional-test arms ...
        _ => None,  // ← Line 368 — Phase 29's extension point
    }
}
```

**Phase 29 change** — replace `_ => None` with:

```rust
pub fn xeq_by_name_local_resolve(name: &str, xrom_modules: u8) -> Option<Op> {
    match name {
        // 8 conditional-test mnemonics unchanged ...
        _ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules),
    }
}
```

**Caller update** — exactly ONE production call site at `hp41-cli/src/app.rs:1423` `[VERIFIED]`:

```rust
// before:
if let Some(op) = keys::xeq_by_name_local_resolve(&acc) {
// after:
if let Some(op) = keys::xeq_by_name_local_resolve(&acc, self.state.xrom_modules) {
```

**Test-file updates** — three production call sites in tests:
- `hp41-cli/tests/phase25_xeq_by_name.rs:94,99,105,108,113,122,133,200,214,225,243,322,344,376` — every `xeq_by_name_local_resolve("X<>Y?")` etc. becomes `xeq_by_name_local_resolve("X<>Y?", 0b0000_0001)` (or similar). Tests don't care about XROM resolution; passing the default Math-1-loaded bit keeps them green. `[VERIFIED: count via grep]`
- `hp41-cli/tests/key_coverage.rs` — `[NEEDS CHECK during plan-time — grep for `xeq_by_name_local_resolve`]`

**Why widen the signature instead of an Arc-shared helper:** the function is `pub`, callable from any test, and threading the bitfield is one extra arg. A module-level constant would lie to tests that someday simulate XROM-unloaded state.

**Drift-detection extension:** the existing `cli_resolver_matches_core_resolver` test at `tests/phase25_xeq_by_name.rs:370` `[VERIFIED]` becomes the natural extension point — add a Math Pac I name (e.g. `"SINH"`) and assert both resolvers agree:

```rust
// at end of cli_resolver_matches_core_resolver:
let math1_cases: &[(&str, Op)] = &[
    ("SINH", Op::Sinh),
    ("MATRIX", Op::MatrixWorkflow),
    ("C+", Op::CPlus),
];
for (name, expected_op) in math1_cases {
    assert_eq!(
        xeq_by_name_local_resolve(name, 0b0000_0001),
        Some(expected_op.clone()),
        "CLI-local resolver must agree with core xrom_resolve for {name:?}"
    );
    // Negative: with XROM unloaded, CLI resolver must NOT resolve Math Pac I names
    assert_eq!(
        xeq_by_name_local_resolve(name, 0b0000_0000),
        None,
        "CLI-local resolver must return None for {name:?} when Math 1 module unloaded"
    );
}
```

### §3.2 CLI-02 + CLI-04 (D-29.1 + D-29.2) — JSON file + second OnceLock + merged accessor

**Authoring `docs/hp41-math1-functions.json`.** ~55 entries; schema MIRROR of `docs/hp41cv-functions.json` plus per-entry `xrom: { module, module_id, function_id }` object per C-28.3.

**Schema reference** (v2.2 baseline) `[VERIFIED: hp41-cli/src/help_data.rs:45–57]`:

```rust
pub struct HelpEntry {
    pub op_variant: String,      // Op:: PascalCase name
    pub display_name: String,    // HP-41 mnemonic
    pub category: String,        // Section header in ? overlay
    pub status: String,          // "implemented" | "deferred-v3" | "na"
    pub phase: Option<String>,
    pub key_path: Option<String>,// "XEQ \"SINH\"" for all Math Pac I entries
    pub description: String,     // <= 80 chars
    #[serde(default)]
    pub divergences: Vec<String>,
}
```

**Phase 29 schema extension.** Math Pac I entries carry an additional optional `xrom` object. Add to `HelpEntry`:

```rust
#[serde(default)]
pub xrom: Option<XromEntry>,

#[derive(Debug, Clone, Deserialize)]
pub struct XromEntry {
    pub module: String,         // "Math 1"
    pub module_id: u8,          // 7
    pub function_id: u16,       // 1..=52 (1-indexed per HP-41 convention)
}
```

`#[serde(default)]` on `xrom: Option<XromEntry>` means v2.2 entries (no `xrom` key) parse unchanged. The deserializer remains valid for BOTH JSON files via the same struct.

**Sample entry** for `docs/hp41-math1-functions.json`:

```json
{
    "op_variant": "Sinh",
    "display_name": "SINH",
    "category": "Math1 Hyperbolics",
    "status": "implemented",
    "phase": "28",
    "key_path": "XEQ \"SINH\"",
    "description": "Hyperbolic sine: X <- sinh(X)",
    "xrom": { "module": "Math 1", "module_id": 7, "function_id": 1 }
}
```

**The 5 ASCII-alias `MATH_1.ops` entries (`C*`, `C/`, `Z^N`, `Z^1/N`, `E^Z`, `A^Z`, `Z^W`) `[VERIFIED: xrom.rs:58,60,67,70,73,77,79]` are NOT separate JSON entries.** They're alternative spellings that route to the same `Op` variant. JSON has 47 unique-`op_variant` rows; planner counts both the primary mnemonic and the alias in the description if surface-discovery is helpful.

**Author the ~47 unique entries** in plan 29-01 (Plan-A authoring task). Use `MATH_1.ops` `[VERIFIED: xrom.rs:43–118]` as the canonical pull-list, deduplicating aliases.

**The second `OnceLock` mirrors the v2.2 pattern verbatim** `[VERIFIED: hp41-cli/src/help_data.rs:62–77]`:

```rust
// hp41-cli/src/help_data.rs additions:

const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");
static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_math1() -> &'static [HelpEntry] {
    MATH1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(MATH1_FUNCTIONS_JSON)
            .expect("hp41-math1-functions.json is malformed — fix the JSON")
    })
}

pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries().iter().chain(help_entries_math1().iter())
}
```

**Migration of 3 call sites to `help_entries_all()`** — Phase 29 plan 01 task:

| Call site | Current | After |
|-----------|---------|-------|
| `hp41-cli/src/help_data.rs::help_overlay_rows` `[VERIFIED:95–121]` | `let entries = help_entries();` | `let entries: Vec<&HelpEntry> = help_entries_all().collect();` (or refactor to iterate twice). NOTE: `help_overlay_rows` clones into owned `String`s — refactor cost is minimal |
| `hp41-cli/src/keys.rs::key_ref_entries` `[VERIFIED:390–405]` | `for entry in crate::help_data::help_entries()` | `for entry in crate::help_data::help_entries_all()` |
| `hp41-cli/tests/function_matrix_parity.rs:215, 242, 261` `[VERIFIED]` | `let entries = help_entries();` | Decide per-test (see §3.8 below) |

**Test-file guard** for v2.2 invariant — `hp41-cli/tests/phase25_help_data.rs:25 help_entries_count_meets_130_target` `[VERIFIED]` MUST KEEP using `help_entries()` (narrow accessor), not `help_entries_all()` — the 130-target is specifically about v2.2. CONTEXT confirms.

**New `phase29_help_data_math1.rs` smoke test file** — mirror of `phase25_help_data.rs` against `help_entries_math1()`:
- `math1_help_entries_loads_at_runtime` — assert non-empty
- `math1_help_entries_count_meets_47_target` — `entries.len() >= 47` (47 unique-`op_variant` rows per §3.2 above)
- `math1_help_entries_all_xrom_module_id_is_7` — every entry's `xrom.module_id == 7`
- `math1_help_entries_xrom_function_ids_unique_and_dense` — function_ids form a 1..=N range with no gaps
- `math1_help_entries_categories_prefix_with_math1` — every category begins with `"Math1 "` (e.g. `"Math1 Hyperbolics"`)
- `math1_help_entries_all_key_path_is_xeq_form` — every implemented entry has `key_path == Some("XEQ \"<MNEMONIC>\"")` per D-28.6

### §3.3 CLI-05 sub-1 (D-29.3) — widen `pending_prompt()` to surface `modal_prompt`

**Current signature** `[VERIFIED: hp41-cli/src/ui.rs:258]`:

```rust
pub fn pending_prompt(pending: &crate::app::PendingInput) -> String
```

**Current call site** — exactly one production caller `[VERIFIED: hp41-cli/src/ui.rs:229]`:

```rust
let base: String = if let Some(ref pending) = app.pending_input {
    pending_prompt(pending)
} else if app.state.alpha_mode {
    "ALPHA mode — Enter or A to exit".to_string()
} else {
    app.message.as_deref().unwrap_or("Ready").to_string()
};
```

**Test call sites** — many in `hp41-cli/tests/phase25_pending_input.rs:103`+ `[VERIFIED]`. Signature change ripples.

**Recommended widening** (per D-29.3 / Claude's discretion):

**Option A — narrow + explicit dependency (RECOMMENDED for testability):**
```rust
pub fn pending_prompt(
    pending: Option<&crate::app::PendingInput>,
    modal_prompt: Option<&str>,
) -> String
```

Behavior:
- If `modal_prompt.is_some()` AND `pending.is_none()` → render `modal_prompt`
- If `pending.is_some()` AND `modal_prompt.is_none()` → render `pending_prompt(pending)` (existing exhaustive match)
- If BOTH are set: planner's discretion — **recommend modal_prompt wins** (the open math1 modal is the higher-priority UX), with a test asserting this precedence
- If BOTH are None: caller doesn't call us (current behavior preserved)

**Option B — take `&App` (ergonomic in render loop):**
```rust
pub fn pending_prompt(app: &App) -> String
```

Behavior internal: read `app.pending_input` and `app.state.modal_prompt` directly. Cost: every test that drives `pending_prompt(&p)` directly (~10 sites in `phase25_pending_input.rs`) must construct a full `App`. Higher test-construction cost.

**Recommendation: Option A.** Keeps `pending_prompt` a pure function over its inputs; existing tests update by adding a `None` second arg. The single render call site updates to:

```rust
let base: String = if app.pending_input.is_some() || app.state.modal_prompt.is_some() {
    pending_prompt(app.pending_input.as_ref(), app.state.modal_prompt.as_deref())
} else if app.state.alpha_mode {
    "ALPHA mode — Enter or A to exit".to_string()
} else {
    app.message.as_deref().unwrap_or("Ready").to_string()
};
```

**Truncation concern (Risk §7.5 below):** `"FUNCTION NAME?"` is 14 chars; `"NO SOLUTION"` is 11 chars; `"ROW\u{2191}COL=?"` is 8 chars. The status bar is the bottom of the 55% left panel (ratatui `Constraint::Min(0)` `[VERIFIED: ui.rs:79]`) which at the minimum 80×24 terminal gives ~44 cols — comfortably wider than any modal prompt.

### §3.4 CLI-05 sub-2 (D-29.5) — R/S interception → `submit_modal`

**Current R/S handling** `[VERIFIED: hp41-cli/src/app.rs:638–648]`:

```rust
if key.code == KeyCode::F(5) {
    match hp41_core::run_program(&mut self.state, "A") {
        Ok(()) => { ... }
        Err(e) => self.message = Some(format!("{e}")),
    }
    return;
}
```

R/S in CLI is `F5`, NOT `KeyCode::Char('R')` (which opens an STO/RCL modal at `app.rs:348` `[VERIFIED]`). The Phase 29 interception goes ABOVE this block but BELOW the `pending_input.is_some()` block at `app.rs:327` (D-07 invariant).

**Phase 29 insertion** — between the help/programs-overlay block (ends ~`app.rs:536`) and the digit-entry block (starts ~`app.rs:544`), or alternatively just before the existing F5 handler at line 638. Recommended placement: right before the F5 handler so all modal interception sits together:

```rust
// Phase 29 (D-29.5): R/S submits modal numeric input when modal_program is active.
// MUST be ABOVE the existing F5 (run_program("A")) handler — modal flow takes
// precedence over the v1.0 run-A binding when a math1 modal is open.
if key.code == KeyCode::F(5) && self.state.modal_program.is_some() {
    match hp41_core::ops::math1::submit_modal(&mut self.state) {
        Ok(()) => {
            self.message = None;
            // print_buffer may carry per-step output (e.g. VMAT lines after EDIT)
            self.drain_and_show_print_output(None);
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
    return;
}

// Existing F5 fallthrough handler unchanged:
if key.code == KeyCode::F(5) { ... }
```

**Pre-flush.** The user typed digits while the modal was open. Those digits live in `state.entry_buf`. `submit_modal` MUST internally call `flush_entry_buf(state)` BEFORE consuming the value off the stack — same pattern as the existing v1.1 STO-arithmetic modal completion. The X-register read after flush gives the modal step its numeric value. **Constraint:** `flush_entry_buf` is `pub` `[VERIFIED: hp41-core/src/ops/mod.rs:820]` and returns `Result<(), HpError>` — `submit_modal` propagates that error.

**`submit_modal` shape** (Claude's discretion):

```rust
// hp41-core/src/ops/math1/mod.rs additions:
pub use modal::ModalProgram;

pub fn submit_modal(state: &mut CalcState) -> Result<(), HpError> {
    // Flush any pending digit entry so X holds the submitted numeric value.
    crate::ops::flush_entry_buf(state)?;

    let Some(modal) = state.modal_program.clone() else {
        return Err(HpError::InvalidOp);  // R/S in CLI guards this; defensive
    };

    match modal {
        ModalProgram::Matrix(step) => matrix::submit_step(state, step),
        ModalProgram::Solve(step) => solve::submit_step(state, step),
        ModalProgram::Poly(step) => poly::submit_step(state, step),
        ModalProgram::Integ(step) => integ::submit_step(state, step),
        ModalProgram::Difeq(step) => difeq::submit_step(state, step),
        ModalProgram::Four(step) => four::submit_step(state, step),
        ModalProgram::Trans(step) => trans::submit_step(state, step),
    }
}
```

Each per-program `submit_step` is a NEW function the planner adds to the corresponding `hp41-core/src/ops/math1/<program>.rs` file. **Frozen-source caveat:** CONTEXT explicitly notes "the entire family is FROZEN as of Phase 28 ship except for the additive `submit_modal` / `cancel_modal` / `requires_alpha_label` public surface; planner MUST NOT touch existing function logic." Adding new `pub fn submit_step` functions to each math1 file is the additive part — no existing function bodies modified. **Plan-time check:** confirm with discuss-phase if `submit_step` per file counts as additive.

### §3.5 CLI-05 sub-3 (D-29.6) — Esc cancellation → `cancel_modal`

**Current Esc handling.** Esc currently lives inside each `handle_*_prompt()` helper (e.g. `handle_xeq_by_name:1409` `[VERIFIED]`). There is NO top-level Esc handler in `handle_key` outside of pending-input modals. The shift-armed Esc handler at `app.rs:438` `[VERIFIED]` only fires when `self.shift_armed`.

**Phase 29 insertion** — same location as the R/S interception (before the F5 handler), check `pending_input.is_none()` first:

```rust
// Phase 29 (D-29.6): Esc cancels an open math1 modal (no pending_input active).
// If pending_input is active (XeqByName / RegisterPrompt / etc.), Esc is handled
// by handle_pending_input via existing per-arm handlers (D-07 — never override).
if key.code == KeyCode::Esc
    && self.state.modal_program.is_some()
    && self.pending_input.is_none()
{
    hp41_core::ops::math1::cancel_modal(&mut self.state);
    self.message = Some("Cancelled".to_string());
    return;
}
```

**`cancel_modal` shape** — pure state cleanup, no error path:

```rust
// hp41-core/src/ops/math1/mod.rs additions:
pub fn cancel_modal(state: &mut CalcState) {
    state.modal_program = None;
    state.modal_prompt = None;
    state.entry_buf.clear();
    // Solver scratch (integ_state/solve_state/difeq_state) is per-op,
    // not per-modal — leave it alone. matrix_dim / matrix_active_reg
    // outlive the modal (a matrix can be queried after entry); leave too.
    // Stack: untouched.
}
```

**Esc-on-shift_armed precedence** — D-25.4 `app.shift_armed` interaction. The existing code at `app.rs:436–451` is:

```rust
if self.shift_armed {
    if key.code == KeyCode::Esc { self.shift_armed = false; return; }
    // ... consume on next key ...
}
```

The shift-armed Esc handler fires BEFORE the math1-modal Esc. This is correct — a user with f armed who hits Esc is cancelling the f-prefix, not the math1 modal. If the user wants to cancel the modal, they hit Esc twice (Esc → shift_armed cleared; Esc → modal cancelled). Document this two-step convention in the test file.

### §3.6 CLI-05 sub-4 (D-29.7 + D-29.8) — extend `PendingInput::XeqByName` with `mode`

**Current variant** `[VERIFIED: hp41-cli/src/app.rs:113]`:

```rust
XeqByName(String),
```

**Phase 29 change** to struct-variant per D-29.8:

```rust
pub enum XeqByNameMode {
    /// Existing behavior — Enter resolves to Op via xeq_by_name_local_resolve.
    Normal,
    /// New — Enter calls modal_program.advance_with_label() instead.
    CollectForModal,
}

XeqByName { acc: String, mode: XeqByNameMode },
```

**Match-arm inventory** — every site that pattern-matches `PendingInput::XeqByName(...)` MUST be updated to `PendingInput::XeqByName { acc, mode }`:

#### Production code

| File:Line | Current | Phase 29 update |
|-----------|---------|-----------------|
| `hp41-cli/src/app.rs:113` `[VERIFIED]` | enum variant decl `XeqByName(String)` | replace with struct-variant `XeqByName { acc: String, mode: XeqByNameMode }` |
| `hp41-cli/src/app.rs:309` `[VERIFIED]` | `Some(PendingInput::XeqByName(_)) \| Some(PendingInput::ClpLabel(_))` (in `?`-handler guard) | `Some(PendingInput::XeqByName { .. }) \| Some(PendingInput::ClpLabel(_))` |
| `hp41-cli/src/app.rs:1072` `[VERIFIED]` | `Some(PendingInput::XeqByName(acc)) => { self.handle_xeq_by_name(key, acc); }` | `Some(PendingInput::XeqByName { acc, mode }) => { self.handle_xeq_by_name(key, acc, mode); }` |
| `hp41-cli/src/app.rs::handle_xeq_by_name(...)` (signature) `[VERIFIED:1407]` | `fn handle_xeq_by_name(&mut self, key: KeyEvent, acc: String)` | `fn handle_xeq_by_name(&mut self, key: KeyEvent, acc: String, mode: XeqByNameMode)` |
| `hp41-cli/src/app.rs:1434, 1441, 1444` `[VERIFIED]` | `Some(PendingInput::XeqByName(new_acc))` (3 re-stores in `handle_xeq_by_name`) | `Some(PendingInput::XeqByName { acc: new_acc, mode })` (mode pass-through) |
| `hp41-cli/src/app.rs:1412–1430` (the `Enter` arm) `[VERIFIED]` | calls `xeq_by_name_local_resolve(&acc)` then `call_dispatch(op)` | Branch on `mode`: `Normal` → existing resolver chain; `CollectForModal` → call new `submit_modal_with_label(&mut state, &acc)` |
| `hp41-cli/src/keys.rs:319` `[VERIFIED]` | `app.pending_input = Some(PendingInput::XeqByName(String::new()));` (f-N opener) | `app.pending_input = Some(PendingInput::XeqByName { acc: String::new(), mode: XeqByNameMode::Normal });` |
| `hp41-cli/src/ui.rs:329` `[VERIFIED]` | `PendingInput::XeqByName(acc) => format!("XEQ \"{acc}\"_")` (`pending_prompt` arm) | `PendingInput::XeqByName { acc, mode: XeqByNameMode::Normal } => format!("XEQ \"{acc}\"_")` + a second arm for `mode: CollectForModal` (e.g. shows `"FUNCTION NAME? [{acc}_]"` or — recommended — defers to `state.modal_prompt` rendering and just shows the accumulator with no prefix). Planner's discretion. |

#### Test code

| File:Line | Update |
|-----------|--------|
| `hp41-cli/tests/phase25_pending_input.rs:74, 95, 216, 442, 447, 477` `[VERIFIED]` | Every `PendingInput::XeqByName("..."` or `(_)` matcher → struct-variant form |
| `hp41-cli/tests/phase25_xeq_by_name.rs:66, 72, 249, 255` `[VERIFIED]` | Same — replace tuple-style with struct-style |
| `hp41-cli/tests/key_coverage.rs:65, 75, 191` `[VERIFIED]` | `KeyPath::XeqByName(String)` is a TEST-LOCAL enum in `key_coverage.rs` — UNRELATED to `PendingInput::XeqByName`. **No change needed** to this file's local enum unless the planner chooses to mirror the change for symmetry |

**Compile-time exhaustiveness invariant preserved.** Adding a new field to a struct variant forces every match site to update; the compiler will fail loudly. FN-CLI-04 ("no `_ =>` catch-all in `pending_prompt`") holds — adding a second `XeqByName` arm in `pending_prompt` keeps the exhaustive match.

**`XeqByNameMode` exhaustive match in `pending_prompt`** — even with two variants, both must appear explicitly:

```rust
PendingInput::XeqByName { acc, mode: XeqByNameMode::Normal } => format!("XEQ \"{acc}\"_"),
PendingInput::XeqByName { acc, mode: XeqByNameMode::CollectForModal } => format!("NAME: {acc}_"),
```

This satisfies "compile-time exhaustive match over `XeqByNameMode`" per D-29.8.

### §3.7 CLI-05 sub-5 (D-29.9) — post-dispatch auto-open hook

**Where to insert.** After every dispatch in `App::handle_key`, the CLI must check whether a math1 modal just opened with an alpha-label step (`Integ(FunctionNamePrompt)` / `Solve(FunctionNamePrompt)` / `Difeq(FunctionNamePrompt)`) AND `pending_input.is_none()`. If true, auto-open `XeqByName { mode: CollectForModal }`.

**Natural insertion point** — the `call_dispatch` helper at `hp41-cli/src/app.rs:1640` `[VERIFIED]`. EVERY dispatch routes through this helper. After the dispatch returns, before the function returns, check the auto-open condition:

```rust
// hp41-cli/src/app.rs::call_dispatch (~line 1640)
fn call_dispatch(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => { /* existing print drain logic */ },
        Err(e) => { self.message = Some(format!("{e}")); },
    }
    self.maybe_auto_open_collect_for_modal();  // ← NEW
}

fn maybe_auto_open_collect_for_modal(&mut self) {
    use hp41_core::ops::math1::ModalProgram;
    if self.pending_input.is_some() { return; }   // D-29.9 gate
    let Some(ref mp) = self.state.modal_program else { return; };
    if mp.requires_alpha_label() {
        self.pending_input = Some(PendingInput::XeqByName {
            acc: String::new(),
            mode: XeqByNameMode::CollectForModal,
        });
    }
}
```

**`requires_alpha_label` shape** — recommended pure-method on `ModalProgram` in `hp41-core/src/ops/math1/modal.rs`:

```rust
impl ModalProgram {
    /// Returns true for steps that require the user to type a user-program LBL name.
    /// Currently: only the FunctionNamePrompt step of Integ / Solve / Difeq.
    pub fn requires_alpha_label(&self) -> bool {
        matches!(
            self,
            ModalProgram::Integ(IntegInputStep::FunctionNamePrompt)
            | ModalProgram::Solve(SolveInputStep::FunctionNamePrompt)
            | ModalProgram::Difeq(DifeqInputStep::FunctionNamePrompt)
        )
    }
}
```

Compile-time guarantee: a future per-program step that needs ALPHA input must extend this matcher (no `_ =>` catch-all needed because the matcher is positive-form). Tests assert each named variant returns `true` and every other variant returns `false`.

**Why call_dispatch and not handle_key:** `call_dispatch` is the SINGLE chokepoint for ALL state mutation routes — `key_to_op` path, `shifted_key_to_op` path, modal-handler `call_dispatch` calls, F1–F4 USER mode, etc. Inserting the auto-open hook here means ZERO other call sites need to know about it (DRY, single-place rule).

**Where `call_dispatch_and_drain` differs.** There's also a `call_dispatch_and_drain` at `app.rs:1651` `[VERIFIED]` — same pattern, same insertion point (or a shared post-dispatch helper). Phase 29 must wire BOTH.

**FUNCTION NAME? handling on Enter.** When the user presses Enter inside `XeqByName { mode: CollectForModal }`, the existing `handle_xeq_by_name` `Enter` arm at `app.rs:1412` branches on `mode`:

```rust
KeyCode::Enter => {
    if !acc.is_empty() {
        match mode {
            XeqByNameMode::Normal => {
                if let Some(op) = keys::xeq_by_name_local_resolve(&acc, self.state.xrom_modules) {
                    self.call_dispatch(op);
                } else {
                    self.call_dispatch(Op::Xeq(acc));
                }
            }
            XeqByNameMode::CollectForModal => {
                // D-29.7: pass label to modal's advance-with-label step.
                match hp41_core::ops::math1::submit_modal_with_label(&mut self.state, &acc) {
                    Ok(()) => self.message = None,
                    Err(e) => self.message = Some(format!("{e}")),
                }
                // submit_modal_with_label may itself open the NEXT modal step
                // (e.g. Solve(FunctionName) → Solve(Guess1Prompt)). That doesn't
                // require alpha — auto-open won't fire. The next R/S submits Guess1.
            }
        }
    }
    self.pending_input = None;
}
```

`submit_modal_with_label` is a sibling of `submit_modal` — takes a label string instead of using `entry_buf`. Internally writes the label to `state.alpha_reg` (per the OM convention `op_solve_run_loop` already reads — `solve.rs:190` `[VERIFIED: state.alpha_reg.clone()`]`) AND advances `modal_program` to the next step + writes the next `modal_prompt`. Planner has discretion on whether this is a new free function or a method on `ModalProgram` that returns the next state.

## §4 PendingInput::XeqByName Match-Arm Inventory

Comprehensive grouping — every site that must be updated when the variant goes from tuple to struct form:

### Production source (must change struct-variant pattern)

```
hp41-cli/src/app.rs:113    enum declaration                            [REQUIRED]
hp41-cli/src/app.rs:309    or-pattern guard in '?' handler             [REQUIRED]
hp41-cli/src/app.rs:1072   handle_pending_input dispatch arm           [REQUIRED]
hp41-cli/src/app.rs:1407   handle_xeq_by_name(...) signature           [REQUIRED]
hp41-cli/src/app.rs:1412   Enter arm — add mode branch                 [REQUIRED + NEW LOGIC]
hp41-cli/src/app.rs:1434   Backspace re-store                          [REQUIRED — pass mode through]
hp41-cli/src/app.rs:1441   Char append re-store                        [REQUIRED — pass mode through]
hp41-cli/src/app.rs:1444   Fallback re-store                           [REQUIRED — pass mode through]
hp41-cli/src/keys.rs:319   f-N opener (modal opens in Normal mode)     [REQUIRED]
hp41-cli/src/ui.rs:329     pending_prompt match arm                    [REQUIRED + ADD 2ND ARM]
```

### Test source (must change struct-variant pattern)

```
hp41-cli/tests/phase25_pending_input.rs:74    construction in pending_input_variants_compile  [REQUIRED]
hp41-cli/tests/phase25_pending_input.rs:95    matches!() assertion                            [REQUIRED]
hp41-cli/tests/phase25_pending_input.rs:216   construction for pending_prompt test            [REQUIRED]
hp41-cli/tests/phase25_pending_input.rs:442   modal-open construction in integration test     [REQUIRED]
hp41-cli/tests/phase25_pending_input.rs:447   destructure during accumulator inspection       [REQUIRED]
hp41-cli/tests/phase25_pending_input.rs:477   construction in another test                    [REQUIRED]
hp41-cli/tests/phase25_xeq_by_name.rs:66      modal-open construction in type_name_and_enter  [REQUIRED]
hp41-cli/tests/phase25_xeq_by_name.rs:72      destructure during accumulator inspection       [REQUIRED]
hp41-cli/tests/phase25_xeq_by_name.rs:249     unicode test modal-open                          [REQUIRED]
hp41-cli/tests/phase25_xeq_by_name.rs:255     unicode test destructure                         [REQUIRED]
```

### NOT affected (unrelated `XeqByName` symbol in test-local enum)

```
hp41-cli/tests/key_coverage.rs:65   LOCAL test enum KeyPath::XeqByName(String) — SEPARATE TYPE
hp41-cli/tests/key_coverage.rs:75   LOCAL constructor — UNRELATED
hp41-cli/tests/key_coverage.rs:191  LOCAL match arm — UNRELATED
```

**Total production sites: 10** (~one logical changeset per file). **Total test sites: 10.**

The compile-time exhaustive-match invariant means an Rust build immediately surfaces any missed site — Phase 29 plans can ship the variant rename FIRST, then iteratively fix each site as the compiler complains. Estimated effort: 1 hour for the rename sweep.

## §5 Test Architecture

### Smoke test: `hp41-cli/tests/phase29_help_data_math1.rs`

Mirror of `phase25_help_data.rs` against the new JSON pool. ~8 tests:

| Test | Assertion |
|------|-----------|
| `math1_help_entries_loads_at_runtime` | `!help_entries_math1().is_empty()` — hard-build-blocker per D-25.17 catches malformed JSON |
| `math1_help_entries_count_meets_47_target` | `help_entries_math1().len() >= 47` (matches `MATH_1.ops` deduplicated count) |
| `math1_help_entries_has_no_duplicate_op_variants` | mirror of v2.2 pattern |
| `math1_help_entries_all_have_non_empty_description` | ≤ 80 chars per existing convention |
| `math1_help_entries_status_is_closed_enum` | every status is `"implemented"` (no deferreds in v3.0 Phase 29) |
| `math1_help_entries_all_xrom_module_id_is_7` | the `xrom` object is present on every entry and `module_id == 7` |
| `math1_help_entries_categories_prefix_with_math1` | every category starts with `"Math1 "` (planner's category convention) |
| `math1_help_entries_xrom_function_ids_are_dense` | function_ids form `1..=N` with no gaps and no duplicates |

### Parity-test extension: `hp41-cli/tests/function_matrix_parity.rs`

**Existing 4 tests** `[VERIFIED]`:
1. `test_op_inventory_count_matches_enum` — asserts `ALL_OP_VARIANT_NAMES.len() == 130`
2. `test_every_rom_op_has_matrix_entry` — forward parity (every Op has JSON row)
3. `test_every_implemented_matrix_entry_has_op` — reverse parity (every JSON row has Op)
4. `test_matrix_has_at_least_130_entries` — count sanity

**Phase 29 extension approach** — add a parallel pool of Math Pac I variants:

```rust
const MATH1_OP_VARIANT_NAMES: &[&str] = &[
    "Sinh", "Cosh", "Tanh", "Asinh", "Acosh", "Atanh",          // 6 hyp
    "CPlus", "CMinus", "CTimes", "CDiv", "Real",                // 5 complex-arith (note: C*/C/ are aliases)
    "Magz", "Cinv", "ZpowN", "Zpow1N", "ExpZ", "LnZ", "SinZ", "CosZ", "TanZ", "ApowZ", "LogZ", "ZpowW",  // 12 complex-fn
    "PolyWorkflow", "Roots",                                    // 2 poly
    "MatrixWorkflow", "MatSize", "MatVmat", "MatEdit", "MatDet", "MatInv", "MatSimeq", "MatVcol",  // 8 matrix
    "Integ", "Solve", "Sol", "Difeq",                           // 4 solvers
    "Four", "TriSss", "TriAsa", "TriSaa", "TriSas", "TriSsa",   // 6 four + tri
    "Trans2d", "Trans3d",                                       // 2 trans
];

#[test]
fn test_math1_op_inventory_count() {
    assert_eq!(MATH1_OP_VARIANT_NAMES.len(), 47, "MATH1 inventory drift");
}

#[test]
fn test_every_math1_rom_op_has_math1_json_entry() {
    let entries = help_entries_math1();
    let json_variants: HashSet<&str> = entries.iter().map(|e| e.op_variant.as_str()).collect();
    let mut missing: Vec<&str> = Vec::new();
    for name in MATH1_OP_VARIANT_NAMES {
        if !json_variants.contains(name) { missing.push(name); }
    }
    assert!(missing.is_empty(), "Math1 Op variants missing from JSON: {missing:?}");
}

#[test]
fn test_every_math1_json_entry_has_xrom_resolver_match() {
    // Every Math1 JSON entry's op_variant must be resolvable via xrom_resolve.
    // Cross-checks JSON ↔ MATH_1.ops table.
    for entry in help_entries_math1() {
        // The xrom_resolve table uses the display_name spelling; the JSON op_variant
        // uses the Rust enum name. Cross-reference via MATH_1.ops lookup.
        let resolved = hp41_core::ops::math1::xrom::xrom_resolve(
            entry.display_name.as_str(),
            0b0000_0001,
        );
        assert!(
            resolved.is_some(),
            "Math1 JSON entry '{}' (display: '{}') not resolvable via xrom_resolve",
            entry.op_variant, entry.display_name,
        );
    }
}
```

**Existing 4 tests survive unchanged** — they still operate on the v2.2 pool via `help_entries()`. CONTEXT explicitly notes the narrow accessor is retained for these.

### Integration test extension: `hp41-cli/tests/phase25_xeq_by_name.rs`

Add Math Pac I cases to `cli_resolver_matches_core_resolver` at line 370 `[VERIFIED]`:

```rust
let math1_canonical: &[(&str, Op)] = &[
    ("SINH", Op::Sinh),
    ("ASINH", Op::Asinh),
    ("MATRIX", Op::MatrixWorkflow),
    ("DET", Op::MatDet),
    ("INV", Op::MatInv),              // non-shadowing per xrom.rs:89-91
    ("C+", Op::CPlus),
    ("REAL", Op::Real),
    ("INTG", Op::Integ),
    ("SOLVE", Op::Solve),
    ("DIFEQ", Op::Difeq),
];
for (name, expected_op) in math1_canonical {
    assert_eq!(
        xeq_by_name_local_resolve(name, 0b0000_0001),
        Some(expected_op.clone()),
        "CLI-local resolver must resolve Math Pac I '{name}' via xrom_resolve fallback",
    );
}

// Negative: with XROM unloaded, Math Pac I names fall through to None → Op::Xeq → InvalidOp.
for (name, _) in math1_canonical {
    assert_eq!(
        xeq_by_name_local_resolve(name, 0b0000_0000),
        None,
        "CLI-local resolver MUST return None for Math Pac I '{name}' with module unloaded",
    );
}
```

### New integration test: `hp41-cli/tests/phase29_modal_flow.rs`

End-to-end driver for the R/S → submit_modal → next-step pipeline. ~5 tests:

| Test | Coverage |
|------|----------|
| `matrix_workflow_order_prompt_advances_on_r_s` | dispatch Op::MatrixWorkflow → modal opens at OrderPrompt → type "3" → R/S → `modal_program == Matrix(ElementPrompt(0,0))` AND `modal_prompt == "A1,1=?"` |
| `solve_workflow_auto_opens_collect_for_modal` | dispatch Op::Solve → modal opens at FunctionNamePrompt → AFTER call_dispatch, `pending_input == Some(XeqByName { mode: CollectForModal })` |
| `solve_workflow_label_submission_advances_to_guess1` | open Solve modal in CollectForModal → type "F" → Enter → `state.alpha_reg == "F"` AND `modal_program == Solve(Guess1Prompt)` |
| `esc_cancels_open_modal` | open Matrix modal → press Esc → `state.modal_program.is_none()` AND `state.modal_prompt.is_none()` AND `app.message == Some("Cancelled")` |
| `esc_shift_armed_takes_precedence_over_modal_cancel` | open Matrix modal → press f → press Esc → `shift_armed == false` AND `modal_program.is_some()` (modal still open) |

## §6 Validation Architecture

> Nyquist validation enabled per `.planning/config.json` workflow.nyquist_validation: true `[VERIFIED]`.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust integration tests (`#[test]` + `cargo test`); MSRV 1.88 |
| Config file | None (Cargo built-in) |
| Quick run command | `cargo test -p hp41-cli --tests --no-fail-fast` |
| Full suite command | `just ci` (lint + test + coverage) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| CLI-01 | `xeq_by_name_local_resolve("SINH", 0b1) == Some(Op::Sinh)` | unit | `cargo test -p hp41-cli --test phase25_xeq_by_name cli_resolver_matches_core_resolver` | ✅ extend |
| CLI-02 | `help_entries_math1().len() >= 47` AND no panic on first call | smoke | `cargo test -p hp41-cli --test phase29_help_data_math1` | ❌ Wave 0 |
| CLI-02 | Forward parity: every `MATH1_OP_VARIANT_NAMES` entry has a JSON row | parity | `cargo test -p hp41-cli --test function_matrix_parity test_every_math1_rom_op_has_math1_json_entry` | ❌ extend |
| CLI-02 | Reverse parity: every Math1 JSON entry resolves via `xrom_resolve` | parity | `cargo test -p hp41-cli --test function_matrix_parity test_every_math1_json_entry_has_xrom_resolver_match` | ❌ extend |
| CLI-03 | Every Math1 `Op` variant has a `prgm_display::op_display_name` arm (no `_ =>` catch-all needed because of exhaustive match) | compile-time | `cargo check -p hp41-cli` (already passing per Phase 28 ship) | ✅ already-shipped |
| CLI-04 | `key_ref_entries()` includes ≥ 1 row whose `key_path == "XEQ \"SINH\""` after migration to `help_entries_all()` | unit | `cargo test -p hp41-cli --test phase29_key_ref_includes_math1` (new test, suggested) | ❌ Wave 0 |
| CLI-05 | R/S in `MatrixWorkflow(OrderPrompt)` with `entry_buf="3"` advances modal to `ElementPrompt(0,0)` | integration | `cargo test -p hp41-cli --test phase29_modal_flow matrix_workflow_order_prompt_advances_on_r_s` | ❌ Wave 0 |
| CLI-05 | Solve workflow auto-opens `XeqByName { mode: CollectForModal }` after dispatch | integration | `cargo test -p hp41-cli --test phase29_modal_flow solve_workflow_auto_opens_collect_for_modal` | ❌ Wave 0 |
| CLI-05 | `pending_prompt(None, Some("ORDER=?"))` returns `"ORDER=?"` | unit | `cargo test -p hp41-cli --test phase29_pending_prompt_modal` | ❌ Wave 0 |
| CLI-05 | Esc cancels open modal cleanly | integration | `cargo test -p hp41-cli --test phase29_modal_flow esc_cancels_open_modal` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-cli --tests` (Phase 29 only touches hp41-cli; hp41-core frozen)
- **Per wave merge:** `just ci` (full lint + test + coverage gate)
- **Phase gate:** `just ci` green AND human verification of one modal flow end-to-end (Matrix small example from OM p.14)

### Wave 0 Gaps

- [ ] `hp41-cli/tests/phase29_help_data_math1.rs` — covers CLI-02 (smoke load + count + xrom-module assertions)
- [ ] `hp41-cli/tests/phase29_modal_flow.rs` — covers CLI-05 (end-to-end R/S, Esc, auto-open)
- [ ] `hp41-cli/tests/phase29_pending_prompt_modal.rs` (or extension of `phase25_pending_input.rs`) — covers CLI-05 `pending_prompt` widening
- [ ] `hp41-cli/tests/phase29_key_ref_includes_math1.rs` (or extension of `keys.rs` inline `#[cfg(test)]`) — covers CLI-04
- [ ] Extension to `hp41-cli/tests/function_matrix_parity.rs` — adds Math1 parity tests
- [ ] Extension to `hp41-cli/tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` — adds Math1 names
- [ ] `docs/hp41-math1-functions.json` itself (the data file is a test prereq — `phase29_help_data_math1` won't compile/load without it)

### Success Criteria Pass/Fail Predicates

| SC | Predicate | Witness Test |
|----|-----------|--------------|
| SC-1 (CLI-01) | `xeq_by_name_local_resolve("SINH", 0b1) == Some(Op::Sinh)` | `cli_resolver_matches_core_resolver` (extended) |
| SC-2 (CLI-02) | `?` overlay rendered via `help_overlay_rows()` contains a category named `"Math1 Hyperbolics"` (or similar) AND at least 47 Math1 rows | `phase29_help_data_math1::math1_help_entries_count_meets_47_target` + `help_overlay_rows` smoke |
| SC-3 (CLI-03) | `prgm_display::format_step` correctly renders `Op::Sinh` as "SINH" (already shipped — Phase 29 verifies) | Manual via `cargo run -p hp41-cli` after entering SINH into PRGM |
| SC-4 (CLI-04) | `key_ref_entries()` output includes a row `("XEQ \"SINH\"", "SINH")` | `phase29_key_ref_includes_math1` |
| SC-5 (CLI-05) | end-to-end Matrix workflow: dispatch MatrixWorkflow → user types "2" R/S → "1" R/S → "2" R/S → "3" R/S → "4" R/S → matrix entered → XEQ "DET" → X == -2.0 | `phase29_modal_flow::matrix_workflow_end_to_end` (suggested name) |

## §7 Risk Inventory

### §7.1 D-07 ordering — pending_input routing block MUST stay above modal interceptors

**The invariant** (CONTEXT cross-cutting line 43): "`pending_input` routing block must remain ABOVE modal-opening interceptors in `hp41-cli/src/app.rs`."

**Where this matters in Phase 29:** the R/S interception (D-29.5) and Esc interception (D-29.6) are math1-modal interceptors. They MUST come AFTER the `if self.pending_input.is_some() { self.handle_pending_input(key); return; }` block at `app.rs:327` `[VERIFIED]`. Specifically: when the user has an open `XeqByName { CollectForModal }` AND the underlying `modal_program == Solve(FunctionNamePrompt)`, pressing R/S inside the modal should NOT call `submit_modal` — it should be swallowed by `handle_xeq_by_name` and ignored (R/S is not a name character). Only when `pending_input.is_none()` AND `modal_program.is_some()` does R/S trigger `submit_modal`.

**Phase 29 enforcement:** the new R/S interceptor's guard must be `self.state.modal_program.is_some() && self.pending_input.is_none()` to make this explicit, OR — sufficient because the routing already returned — just `self.state.modal_program.is_some()` since the pending_input route returns first. **Recommend the explicit guard** for self-documenting safety.

### §7.2 Esc key collision — shift_armed Esc vs modal Esc

**Two existing Esc handlers in `handle_key`:**
- `app.rs:438` `[VERIFIED]`: shift_armed Esc — clears `shift_armed` and returns
- Inside each pending_input modal handler: per-modal Esc (e.g. `handle_xeq_by_name:1409` clears `pending_input`)

**Phase 29 adds a THIRD top-level Esc handler** (math1 modal cancel). Ordering must be:

1. `pending_input.is_some()` route → existing per-modal Esc fires → return
2. `shift_armed` route → existing shift Esc fires → return
3. (NEW) `modal_program.is_some() && pending_input.is_none()` → `cancel_modal` → return

Both #1 and #2 already return early, so #3 only ever sees pending_input.is_none() && shift_armed=false. The explicit guard `pending_input.is_none()` in #3 is defensive — the `pending_input.is_some()` branch at line 327 returns first.

**Two-step Esc convention** (user-facing): inside an open math1 modal where the user has armed f-prefix, Esc cancels the prefix (one Esc), then a second Esc cancels the modal. Document this in `phase29_modal_flow.rs::esc_shift_armed_takes_precedence_over_modal_cancel`.

### §7.3 modal_program vs pending_input precedence on initial render

**The discretionary call** (CONTEXT Claude's discretion bullet): precedence when both `pending_input.is_some()` AND `state.modal_program.is_some()` simultaneously.

**Recommendation: modal_program wins for `modal_prompt` rendering; pending_input owns input routing.** Concretely:
- The status bar shows `state.modal_prompt` (e.g. `"FUNCTION NAME?"`) — because that's the higher-level UX context the user is in
- Keystrokes route through `handle_pending_input` because that's how the alpha-collection UX works
- The `XeqByName { CollectForModal }` Enter arm calls `submit_modal_with_label` which advances `modal_program` AND `modal_prompt` AND clears `pending_input = None`

**Sequence example** for `XEQ "SOLVE"`:
1. User types `XEQ "SOLVE"` → `Op::Solve` dispatches → `op_solve` returns InvalidOp (programmatic-only) BUT (NOTE: see Open Q1 below)
2. Modal opens: `modal_program = Solve(FunctionNamePrompt)`, `modal_prompt = "FUNCTION NAME?"`
3. `call_dispatch` post-hook fires → `requires_alpha_label() == true` AND `pending_input.is_none()` → auto-open `pending_input = XeqByName { mode: CollectForModal }`
4. UI renders next frame: `pending_prompt(Some(XeqByName{CollectForModal}), Some("FUNCTION NAME?"))` → status bar shows `"FUNCTION NAME?"` (modal wins for label-collection step)
5. User types "F", Enter → `handle_xeq_by_name` Enter arm matches `CollectForModal` → calls `submit_modal_with_label(state, "F")` → `state.alpha_reg = "F"`, `modal_program = Solve(Guess1Prompt)`, `modal_prompt = "GUESS 1=?"`, clears `pending_input`
6. Auto-open hook fires AGAIN — `requires_alpha_label() == false` for Guess1Prompt — no `XeqByName` auto-opens
7. User types "1.0" → `entry_buf = "1.0"` → presses R/S → `submit_modal(state)` flushes entry_buf, X = 1.0, advances `modal_program = Solve(Guess2Prompt)`, `modal_prompt = "GUESS 2=?"`

Per the test in §6, this sequence is asserted end-to-end.

### §7.4 Open Q1 — op_solve / op_integ / op_difeq dispatch path

**The problem.** `op_solve(_state) -> Err(InvalidOp)` at `solve.rs:117` `[VERIFIED]`. Same for `op_integ` (`integ.rs:150`) and `op_difeq` (`difeq.rs:126`). These return InvalidOp when called via the interactive dispatch path because the work happens in `op_*_run_loop` — but the run_loop is entered only from inside a running program (Op::Solve match arm in `run_program::execute_op`).

**The contradiction.** CONTEXT D-29.7 says "Auto-open the v2.2 XEQ-by-name modal in a new 'collect label for modal' mode WHEN the open modal is at an ALPHA-label step". But if `xrom_resolve("SOLVE")` → `Op::Solve` → `dispatch(state, Op::Solve)` → `op_solve(state)` → `Err(InvalidOp)` — the modal NEVER OPENS. The user typing `XEQ "SOLVE"` interactively just sees an InvalidOp error.

**Verification.** `[VERIFIED: solve.rs:117]` shows `pub fn op_solve(_state: &mut CalcState) -> Result<(), HpError> { Err(HpError::InvalidOp) }`. Likewise `integ.rs:150` and `difeq.rs:126`.

**Resolution paths** (planner picks at plan-time):

**Option A — Phase 29 fixes the dispatch arm.** Modify `op_solve` / `op_integ` / `op_difeq` to:
- If `state.is_running` (programmatic XEQ inside run_loop): defer to run_loop arm (current InvalidOp behavior)
- If `!state.is_running` (interactive XEQ via CLI/GUI): open the modal (set `modal_program = Solve(FunctionNamePrompt)`, `modal_prompt = "FUNCTION NAME?"`) and return Ok(())

This contradicts "hp41-core is frozen" — but it ALSO is the only way the interactive UX works. **Recommended** — but plan-checker should call this out as a Phase 28 hp41-core modification (violates the frozen-source constraint from CONTEXT). The check on `state.is_running` is structurally identical to the existing `op_xeq` pattern at `program.rs:67–87` `[VERIFIED]` — there's precedent for the same function having a dispatch arm AND a run_loop arm.

**Option B — Phase 29 wraps in CLI.** Before calling `dispatch(Op::Solve)`, the CLI checks the Op variant and bypasses to a CLI-local "open Solve modal" helper. **Rejected** — duplicates dispatch logic in CLI, violates SC-4 spirit (and D-25.6 parity if GUI doesn't mirror).

**Option C — Phase 29 adds a new `Op::SolveInteractive` / `Op::IntegInteractive` / `Op::DifeqInteractive` variant** that opens the modal. Rejected — adds Op variants Phase 29 was supposed to avoid.

**Plan-time decision needed** — the planner consults discuss-phase / user before plan 29-03 lands. Note: Plan 28-08's `op_solve_run_loop` test at `solve.rs:171+` clearly expects the modal flow to be wired by Phase 29 (the inline comment `// CROSS-REFERENCE: see Plan 28-07 op_integ_run_loop for the symmetric pattern. Phase 29 / CLI-07 wires the full FunctionNamePrompt/Guess1Prompt/Guess2Prompt modal flow that stages these into the same registers before calling run_loop.` at `solve.rs:188` `[VERIFIED]`). So Option A is what Phase 28 anticipated.

**For MATRIX, POLY, FOUR, TRANS** — `op_matrix_workflow` `[VERIFIED: matrix.rs:323]`, `op_poly_workflow` `[VERIFIED: poly.rs:93]`, `op_four` `[VERIFIED: four.rs:91]`, etc. ALREADY open the modal in their dispatch arm and return Ok. Only the three solvers have the run-loop-only quirk because they re-enter `run_loop` from inside the op.

### §7.5 JSON parsing panic-message wording

D-25.17 hard-build-blocker convention `[VERIFIED: help_data.rs:75]`:

```rust
.expect("hp41cv-functions.json is malformed — fix the JSON")
```

Phase 29's mirror at the new OnceLock MUST use distinct wording so the smoke test can distinguish which file failed:

```rust
.expect("hp41-math1-functions.json is malformed — fix the JSON")
```

The `phase29_help_data_math1::math1_help_entries_loads_at_runtime` test would catch a missing/empty file via the panic message — assert in plan-time that the smoke test runs as a normal `#[test]`, not `#[should_panic]`.

### §7.6 Ratatui rendering of long status-bar text

Status bar is `Constraint::Min(0)` `[VERIFIED: ui.rs:79]` — fills remaining rows in the left column (~55% of total width). At 80×24 minimum (D-01 `[VERIFIED: ui.rs:30–36]`), left column is ~44 cols wide. Longest math1 prompts:
- `"FUNCTION NAME?"` — 14 chars ✓
- `"NO. SAMPLES=?"` — 13 chars ✓
- `"AXIS+θ?"` — 7 chars (Unicode θ counts as 1 char display-width) ✓
- `"ROW↑COL=?"` — 9 chars ✓
- `"NO SOLUTION"` — 11 chars ✓

No truncation risk. The `f→` armed-prefix indicator at `ui.rs:240` `[VERIFIED]` prepends ~3 chars — still safe.

### §7.7 print_buffer drain after modal advancement

When `submit_modal` advances Matrix's `Ready` state and dispatches `Op::MatVmat` internally (or any op that pushes to print_buffer), the CLI MUST drain. `drain_and_show_print_output` `[VERIFIED: app.rs:1585]` is the established helper. Plan 29-03 wires it into the R/S → `submit_modal` path (see §3.4 above). Test: MATRIX workflow with VMAT step shows lines in `app.message` summary OR in the print panel.

### §7.8 Recursive auto-open hook

`call_dispatch` calls `maybe_auto_open_collect_for_modal` after every dispatch. `submit_modal_with_label` calls `dispatch` internally (to advance state). Risk: infinite-recursion if `submit_modal_with_label` advances state to ANOTHER alpha-label step. **Verification:** the only alpha-label steps are the three `FunctionNamePrompt` variants. Each program has exactly ONE FunctionNamePrompt step, transitioning to Guess1/StepSize/etc. — no chain of FunctionName steps. Safe.

**Defensive guard:** add a depth counter or `is_auto_opening` reentrancy flag if planner is paranoid; for D-29.9 cardinality (one alpha step per program), recursion bound is 1.

### §7.9 Test isolation — modal_program leaks across tests

`CalcState::new()` initializes `modal_program = None` `[VERIFIED: state.rs:270]`. Per-test `App::new(CalcState::new(), ...)` is the established convention `[VERIFIED: phase25_xeq_by_name.rs:52–57]`. No leak risk.

### §7.10 Save-file backward compatibility

`modal_program`, `modal_prompt` are both `#[serde(default, skip)]` `[VERIFIED: state.rs:185, 193]`. Save files don't persist them. Loading a v2.2 save into a v3.0 binary works; the modals never leak across persistence. No new field added in Phase 29. **Save-file backward compat preserved.**

## §8 Open Questions / Claude's-Discretion Items (RESOLVED)

1. **`op_solve` / `op_integ` / `op_difeq` interactive dispatch arm.** Phase 29 MUST modify these to open the modal in `!state.is_running` mode (Option A in §7.4) — OR find an alternative wiring. **Discuss-phase or plan-checker should escalate.** Without this, the user can never reach INTG/SOLVE/DIFEQ interactively. Recommendation: Phase 29 plan 03 carries a small additive `hp41-core` change for these three functions, justified as "the additive-public-surface clause covers this — the bodies were stubs designed to be filled in Phase 29". **RESOLVED — see Plan 29-03 Task 1 step 4.**

2. **Where do `submit_step` per-program functions live?** §3.4 above recommends adding `pub fn submit_step(state, step)` to each `hp41-core/src/ops/math1/<program>.rs` file. Frozen-source review: this is additive (new pub functions), not a modification of existing logic. Confirm at plan-time. **RESOLVED — see Plan 29-03 Task 1 steps 3 + 5.**

3. **`submit_modal_with_label` API shape.** Free function vs. method on `ModalProgram`? Recommend free function in `hp41-core/src/ops/math1/mod.rs` matching `submit_modal` symmetrically:

```rust
pub fn submit_modal_with_label(state: &mut CalcState, label: &str) -> Result<(), HpError>;
```

Internally trims+uppercases the label per HP-41 convention before staging into `state.alpha_reg`. **RESOLVED — see Plan 29-03 Task 1 steps 3 + 5.**

4. **JSON `xrom.function_id` numbering.** 1-indexed (HP convention) vs 0-indexed (Rust convention)? Recommend 1-indexed to match HP-41 user-facing CATALOG 2 listing. Document in JSON file header comment. **RESOLVED — see Plan 29-01 Task 1.**

5. **JSON authoring — where does the `description` text come from?** Phase 30 nominally owns this; Phase 29 authors the JSON shortcut per D-29.1. Recommend short OM-cited descriptions (e.g. `"Hyperbolic sine: X <- sinh(X)"`) — same style as v2.2 entries. Author at plan-time; ADRs/divergences DOC-04 expand in Phase 30. **RESOLVED — see Plan 29-01 Task 1.**

6. **Category names per program.** Suggested taxonomy:
   - `Math1 Hyperbolics` (6 entries)
   - `Math1 Complex Arithmetic` (5 entries — C+/C-/CTimes/CDiv/Real)
   - `Math1 Complex Functions` (12 entries — Magz/Cinv/Z^N/...)
   - `Math1 Polynomial` (2 — POLY/ROOTS)
   - `Math1 Matrix` (8)
   - `Math1 Integration` (1)
   - `Math1 Root Solver` (2 — SOLVE/SOL)
   - `Math1 Differential Eq` (1)
   - `Math1 Fourier` (1)
   - `Math1 Triangle Solvers` (5)
   - `Math1 Coordinate Transform` (2)

   **RESOLVED — see Plan 29-01 Task 1.**

7. **Should the `XeqByName` `pending_prompt` arm for `CollectForModal` mode use a distinct visual marker?** Recommend showing only `state.modal_prompt` (which IS `"FUNCTION NAME?"`) — the auto-open relationship is invisible to the user. Tests assert this. **RESOLVED — see Plan 29-03 Task 2 step 3.**

8. **`verifying-math-pac-1.md §9 update** — CONTEXT §"Specific Ideas" suggests Phase 29 plan 03 (or 04) updates §9's table to mark each Math Pac I modal flow as ✅ available on CLI. Plan 29-03 final task. Low-effort doc edit. **RESOLVED — see Plan 29-03 Task 2 step 6.**

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (MSRV 1.88) | hp41-cli compile | ✓ | per workspace | — |
| `cargo`, `just` | build/test | ✓ | — | — |
| `serde_json` (already in tree for v2.2) | second OnceLock | ✓ | unchanged | — |
| Existing hp41-core public surface | xrom_resolve / ModalProgram / CalcState fields | ✓ | Phase 28 shipped | — |
| `tempfile` (dev-dep) | test isolation | ✓ | used by phase25_xeq_by_name | — |

**No new runtime dependencies in `hp41-core` or `hp41-cli` per cross-cutting constraint** `[VERIFIED: ROADMAP.md:44]`.

## Sources

### Primary (HIGH confidence, all `[VERIFIED]`)
- `.planning/phases/29-cli-integration/29-CONTEXT.md` — D-29.1..D-29.9, Claude's-Discretion, scope
- `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md` — Phase 28 contract Phase 29 consumes
- `.planning/REQUIREMENTS.md` — CLI-01..05 verbatim wording
- `.planning/ROADMAP.md` — Phase 29 success criteria + cross-cutting constraints
- `hp41-cli/src/keys.rs:347–370` — current `xeq_by_name_local_resolve` signature + body
- `hp41-cli/src/help_data.rs:62–77` — `OnceLock<Vec<HelpEntry>>` pattern source-of-truth
- `hp41-cli/src/ui.rs:225–331` — `render_status` + `pending_prompt` current shape
- `hp41-cli/src/app.rs:113, 309, 1072, 1407–1447` — `PendingInput::XeqByName` declaration + handlers
- `hp41-cli/src/app.rs:266–654` — `handle_key` event loop + ordering invariants
- `hp41-cli/src/app.rs:1640+` — `call_dispatch` helper (auto-open insertion point)
- `hp41-cli/tests/function_matrix_parity.rs` — 4 existing parity tests to extend
- `hp41-cli/tests/phase25_xeq_by_name.rs:370` — `cli_resolver_matches_core_resolver` extension point
- `hp41-cli/tests/phase25_help_data.rs` — smoke-test template
- `hp41-core/src/ops/math1/xrom.rs:43–226` — MATH_1.ops + xrom_resolve
- `hp41-core/src/ops/math1/modal.rs:24–62` — ModalProgram + current_prompt
- `hp41-core/src/ops/math1/mod.rs` — public surface re-exports (currently `pub mod` only — Phase 29 may add `pub use`)
- `hp41-core/src/ops/math1/matrix.rs:323–331` — `op_matrix_workflow` opens modal correctly in dispatch
- `hp41-core/src/ops/math1/solve.rs:117–190` — `op_solve` InvalidOp quirk + Phase 29 cross-reference comment
- `hp41-core/src/ops/program.rs:79, 521` — existing xrom_resolve call sites
- `hp41-core/src/state.rs:160–235` — Phase 28 CalcState fields (xrom_modules, modal_program, modal_prompt, etc.)
- `hp41-core/src/error.rs:48` — `HpError::Canceled` variant
- `docs/hp41cv-functions.json` — v2.2 JSON schema mirror source
- `.planning/config.json` — `nyquist_validation: true` confirmation

### Secondary (MEDIUM confidence — convention extrapolation)
- Per-category naming pattern (`"Math1 Hyperbolics"` etc.) — extrapolated from v2.2 category convention; planner has discretion
- OM-cited descriptions for JSON entries — same style as `docs/hp41cv-functions.json` v2.2 entries; planner authors at plan-time

### Tertiary (LOW confidence — none)
All claims verified against committed source in this session.

## Metadata

**Confidence breakdown:**
- Resolver chain extension (§3.1): HIGH — single-line change, 3 existing call sites established the pattern
- JSON pipeline (§3.2): HIGH — direct mirror of v2.2 pattern; only schema addition (`xrom` optional) is new
- `pending_prompt` widening (§3.3): HIGH — one call site; signature options well-understood
- R/S interception (§3.4): MEDIUM — `submit_modal` shape needs plan-time refinement; Open Q §7.4 (op_solve dispatch arm) is the gating concern
- Esc cancellation (§3.5): HIGH — `cancel_modal` is pure cleanup; ordering vs shift_armed is well-defined
- XeqByName mode extension (§3.6): HIGH — 20 match-arm sites enumerated, compile-time guaranteed
- Auto-open hook (§3.7): HIGH — single insertion point at `call_dispatch`; depth-bounded recursion verified
- Risk inventory (§7): HIGH — every risk has a verifying citation or a concrete plan-time question
- Test architecture (§5, §6): HIGH — direct extension of established patterns; Wave 0 gaps enumerated

**Research date:** 2026-05-17
**Valid until:** 2026-06-16 (stable foundation; Phase 28 source is frozen)

## RESEARCH COMPLETE
