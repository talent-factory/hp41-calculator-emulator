# Phase 29: CLI Integration — Context

**Gathered:** 2026-05-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 29 wires `hp41-cli` to Phase 28's XROM framework so every Math Pac I function is reachable from the TUI. Five concrete consequences:

1. The CLI-local resolver `xeq_by_name_local_resolve` (`hp41-cli/src/keys.rs:347`) gains a final fallback into `hp41_core::ops::math1::xrom::xrom_resolve`, closing the third call site explicitly deferred by Phase 28 Plan 01 (see `.planning/phases/28-…/28-01-SUMMARY.md:173`).
2. `hp41-cli/src/help_data.rs` grows a second `OnceLock<Vec<HelpEntry>>` over a new compile-time-embedded `docs/hp41-math1-functions.json`. A new merged accessor `help_entries_all()` becomes the single source of truth for the `?` overlay and the right-panel `key_ref_entries()` discoverability listing.
3. `docs/hp41-math1-functions.json` is authored in full (~55 entries) inside this phase — pulled forward from the nominal Phase 30 / DOC-01 boundary per D-29.1.
4. `hp41-cli/src/ui.rs::pending_prompt()` learns to read `state.modal_prompt` and renders it on the existing status-bar line; the LCD keeps showing the live X-register / `entry_buf` while a modal is open.
5. `hp41-cli/src/app.rs::handle_key` interposes on R/S and Esc when `state.modal_program.is_some()`, routing to two new shared `hp41-core` entry points: `submit_modal(state)` and `cancel_modal(state)`. The v2.2 `PendingInput::XeqByName` variant gains a `mode: XeqByNameMode` field so the same alpha-collection UI handles both normal `XEQ` resolution and `FUNCTION NAME?` label collection for INTG/SOLVE/DIFEQ.

**In scope:**
- `xeq_by_name_local_resolve` final-fallback extension into `xrom_resolve`
- `docs/hp41-math1-functions.json` full ~55-entry authoring (schema mirror of `hp41cv-functions.json` plus the C-28.3 `xrom: { module, module_id, function_id }` object per entry)
- Second `OnceLock` + `help_entries_math1()` + merged `help_entries_all()` accessor; existing `help_overlay_rows`, `key_ref_entries`, `function_matrix_parity.rs` switch to the merged accessor
- `pending_prompt()` signature widening so it can read `state.modal_prompt` (signature: take state or App, not just `&PendingInput`)
- `App::handle_key` R/S + Esc interception when `state.modal_program.is_some()`
- New `hp41-core` public functions: `pub fn submit_modal(state: &mut CalcState) -> HpResult<()>` and `pub fn cancel_modal(state: &mut CalcState)` (planner picks module location — `ops/math1/mod.rs` or `ops/math1/modal.rs`)
- New `pub fn ModalProgram::requires_alpha_label(&self) -> bool` (or equivalent) so CLI App can detect the `FUNCTION NAME?` step generically
- `PendingInput::XeqByName` extension to `{ acc, mode: XeqByNameMode }` where `XeqByNameMode` is `Normal | CollectForModal`; compile-time exhaustive match preserved per the v2.2 FN-CLI-04 invariant
- CLI App event-loop post-dispatch hook that auto-opens `PendingInput::XeqByName { mode: CollectForModal }` when the open modal is at an alpha-label step
- `function_matrix_parity.rs` extension covering both JSON pools and both Op-name pools (built-in v2.2 + xrom Math Pac I)
- Right-panel discoverability rows surfacing Math Pac I XEQ-by-name entries (no code change needed — `key_ref_entries` already filters by non-null `key_path`; the JSON entries authored in this phase populate them)

**Out of scope (explicit):**
- Any `hp41-core/src/ops/math1/` algorithm changes — the entire family is FROZEN as of Phase 28 ship except for the additive `submit_modal` / `cancel_modal` / `requires_alpha_label` public surface; planner MUST NOT touch existing function logic
- Any `hp41-gui` source changes — Phase 31 owns GUI mirroring (key_map XEQ-fallback, prgm_display arms already shipped in Phase 28, `?`-overlay parallel-load, CATALOG 2, modal-prompt rendering, cancellation channel)
- `op_display_name` arms in both `prgm_display.rs` copies — already shipped in Phase 28 plans 28-02..28-10 (verified: every Math Pac I `Op` variant has its arm)
- `scripts/docs-matrix/` two-input regeneration + `docs/hp41-math1-function-matrix.md` — Phase 30 / DOC-02
- 5 ADR write-ups + `docs/hp41-math1-divergences.md` expansion — Phase 30 / DOC-04 + DOC-07 (the *decisions* were locked in Plan 28-01 research-prep; the *documents* land in Phase 30)
- README v3.0 soft-claim — Phase 30 / DOC-05
- Cancellation UI / `request_cancel` Tauri command — Phase 31 / GUI-05 (the field plumbing already shipped in Phase 28 per D-28.7)
- WebdriverIO E2E smoke extension + Free42 contamination guard — Phase 32 / QUAL-03 + QUAL-05

**Mandated by ROADMAP cross-cutting constraints (lines 35–45 of `.planning/ROADMAP.md`):**
- SC-4 invariant: stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` must return nothing. Phase 29 touches `hp41-cli/` + `docs/` only — SC-4 trivially preserved.
- `#![deny(clippy::unwrap_used)]` continues to apply throughout `hp41-core` (no test files touch core here; CLI new code may use `.unwrap()` only in test modules per established pattern).
- `pending_input` routing block must remain ABOVE modal-opening interceptors in `hp41-cli/src/app.rs` — Phase 29 must extend the block, not reorder it (D-07 never-discard invariant).
- CLI ↔ GUI parity (D-25.6): every behavior the CLI gains here is mirrored in Phase 31 via the SAME `submit_modal` / `cancel_modal` / `requires_alpha_label` / `XeqByNameMode::CollectForModal` shared core surface. No CLI-only routing paths permitted.
- MSRV 1.88 unchanged. Zero new runtime dependencies in `hp41-core` or `hp41-cli`.

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / STATE.md / 28-CONTEXT.md (carried forward — NOT re-decided here)

- **C-28.1 (ADR-001):** Op-strategy A — one `Op` variant per Math Pac I function. Phase 29's JSON entries map 1:1 to `Op::*` variants per the v2.2 D-25.16 schema.
- **C-28.3 (ADR-005):** JSON-pipeline shape — separate `docs/hp41-math1-functions.json` sibling file with identical schema plus the `xrom: { module, module_id, function_id }` object per entry. D-29.1 below operationalizes the authoring timing.
- **C-28.4:** `xrom_resolve` fires LAST in the resolver chain. Phase 28 wired this in `op_xeq` and `run_program::execute_op`; Phase 29's `xeq_by_name_local_resolve` extension preserves the same ordering (built-in card-op names win over xrom names).
- **D-28.4:** `modal_prompt: Option<String>` is the channel for prompt strings. `state.print_buffer` continues to carry PRX/PRA/PRSTK output ONLY. Phase 29 surfaces `modal_prompt` to the TUI (D-29.3) without touching `print_buffer`.
- **D-28.5:** R/S key submits numeric input in a modal prompt — hardware-faithful per HP-41C/CV Math Pac OM 1979 p.13. D-29.5 operationalizes the CLI routing.
- **D-28.6:** XEQ-by-name only — no dedicated key bindings for hyperbolics or any Math Pac I function. JSON `key_path` is `"XEQ \"SINH\""` etc. throughout.
- **D-25.18:** `KEY_REF_TABLE` derives from JSON via `key_ref_entries()` filtered by non-null `key_path`. No hand-curated parallel table. D-29.2 below preserves this — Math Pac I entries flow through the merged accessor.
- **`op_display_name` arms in both `prgm_display.rs` copies** — already shipped in Phase 28 plans 28-02..28-10 (verified by `grep "Op::(Sinh|CPlus|Magz|MatrixWorkflow|Integ|Solve|Difeq|Four|TriSss|Trans2d)" hp41-cli/src/prgm_display.rs hp41-gui/src-tauri/src/prgm_display.rs` returning the full set in both files). Phase 29 plan 29-02 in the original ROADMAP wording shrinks to a verification + parity-test pass.

### Discussed and decided in this session (D-29.1 — D-29.9)

#### JSON file authoring timing

- **D-29.1: Phase 29 authors the full ~55-entry `docs/hp41-math1-functions.json`.** Rejected the minimal-stub option because Phase 29 SC-2 (`?` overlay loads the second JSON file) and SC-4 (`KEY_REF_TABLE` derives Math Pac I rows from non-null `key_path` entries) both NEED real entries; a stub doesn't satisfy either acceptance criterion. Rejected the reorder-Phase-30-first option because it would contradict the locked Phase 28 → 29 → 30 → 31 → 32 build sequence and the dependency claim "Phase 30 depends on Phase 29 (CLI integration validates JSON entries via `tests/function_matrix_parity.rs`)" in ROADMAP line 127. Phase 30 / DOC-01 shrinks to "regenerate `docs/hp41-math1-function-matrix.md` via `scripts/docs-matrix` two-input extension"; the file authoring moves into Phase 29 plan 29-01.
  - **Why:** SC-driven acceptance criteria are the contract. A stub-then-expand round-trip burns ~half a phase of churn (re-touching parity tests, re-running `function_matrix_parity.rs`, re-publishing). Authoring the JSON in Phase 29 also unblocks `function_matrix_parity.rs` parity assertion as a Phase 29 deliverable rather than a Phase 30 retrofit.

- **D-29.2: Second `OnceLock<Vec<HelpEntry>>` + merged accessor pattern.** `hp41-cli/src/help_data.rs` adds:
  - `const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");`
  - `static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();`
  - `pub fn help_entries_math1() -> &'static [HelpEntry]` mirroring the existing `help_entries()` with its own panic message `"hp41-math1-functions.json is malformed — fix the JSON"` per the D-25.17 hard-build-blocker pattern.
  - `pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry>` chaining `help_entries().iter().chain(help_entries_math1().iter())`.
  Existing `help_overlay_rows()` and `keys::key_ref_entries()` migrate to `help_entries_all()`. The narrow-scope `help_entries()` is retained for `phase25_help_data` tests asserting the 130-target count.
  - **Why:** D-25.16 hard-build-blocker pattern preserved per-file. Surgical smoke tests for the v2.2 JSON unaffected. New `phase29_help_data_math1` smoke test gets its own per-file panic message. KEY_REF_TABLE and overlay each have a single source of truth (no parallel hand-curated table; no JSON-pair drift) since the merged accessor enforces inclusion.

#### Modal-prompt rendering site

- **D-29.3: Extend `pending_prompt()` in `hp41-cli/src/ui.rs` to also render `state.modal_prompt`.** Signature widens — instead of `pub fn pending_prompt(pending: &PendingInput) -> String`, it takes `&App` (or `&PendingInput, modal_prompt: Option<&str>`; planner picks the cleanest call shape). When `state.modal_program.is_some()`, the status-bar line renders `state.modal_prompt.as_deref().unwrap_or("")`. The existing v2.2 `PendingInput` arms continue to render unchanged when `modal_program.is_none()`. Precedence when both are somehow active simultaneously is Claude's discretion (modal_program likely wins; tests assert both states are well-defined).
  - **Why:** Single visual channel for ALL prompts — users learn one place to look. Preserves the v2.2 status-bar convention. Avoids adding new vertical real estate or new render functions. Avoids LCD truncation for `FUNCTION NAME?` (14 chars).

- **D-29.4: LCD keeps showing the normal X-register / `entry_buf` during an open modal.** As the user types digits for `ORDER=?` / `A1,1=?` / etc., the LCD reflects `entry_buf` live (the existing v2.2 RegisterPrompt UX). The modal prompt itself stays only on the status bar. No `Display14Seg` renderer changes.
  - **Why:** Mirrors v2.2 `RegisterPrompt` and `FlagPrompt` UX (LCD stays live, prompt on status bar). Lowest implementation risk; no new render code path; no second text source for the Display14Seg renderer.

#### R/S → modal-submit routing

- **D-29.5: Intercept R/S in `hp41-cli/src/app.rs::handle_key` BEFORE the v2.1 `run_stop` / `Op::Stop` path.** When `state.modal_program.is_some()`, drain `entry_buf` via the existing `flush_entry_buf` pipeline and call a new shared `pub fn submit_modal(state: &mut CalcState) -> HpResult<()>` in `hp41-core` that internally dispatches on `modal_program` variant. Planner has discretion on the internal shape (a single match over `ModalProgram` variants is natural, mirroring the D-25.13 carrier-variant consolidation pattern) but the function MUST be the single CLI/GUI entry point so D-25.6 parity holds. CLI does NOT add a new `Op` variant (rejected option B); core does NOT overload `Op::Stop` (rejected option C — would violate D-22.5 Neutral-no-op invariant when not running).
  - **Why:** CLI-local routing keeps `hp41-core`'s `Op::Stop` semantics unchanged. 4-place exhaustive-match invariant maintained without a new Op variant. CLI ↔ GUI parity (D-25.6) reduced to: both frontends interpose on R/S identically and call the same shared `submit_modal`.

- **D-29.6: Esc cancels an open modal cleanly via new `pub fn cancel_modal(state: &mut CalcState)`** in `hp41-core` (same module location as `submit_modal`). Clears `state.modal_program = None`, clears `state.modal_prompt = None`, drops `entry_buf`, leaves stack untouched. Both `hp41-cli` (Esc key) and `hp41-gui` (Esc key + future toast-style cancel) route through it.
  - **Why:** Preserves v2.1 stub-error toast philosophy (no silent discard, D-07 never-discard invariant). Prevents the user from being stuck in a 6-step prompt sequence after an accidental `XEQ POLY`. Same shared function = same behavior on both UIs.

#### FUNCTION NAME? prompt integration

- **D-29.7: Auto-open the v2.2 XEQ-by-name modal in a new "collect label for modal" mode** when the open modal is at an ALPHA-label step (i.e., `FUNCTION NAME?` for INTG/SOLVE/DIFEQ). Reuses 100% of the existing alpha-collection key handling (A-Z keys append to `acc`; Backspace deletes; Enter commits). On Enter, instead of resolving via `xeq_by_name_local_resolve` / `Op::Xeq` fallback chain, the handler calls `modal_program.advance_with_label(label)` (planner shapes this API; likely a new method on `ModalProgram` or a new field on `submit_modal`'s argument signature).
  - **Why:** Reuses one alpha-collection codepath = drift-resistant. User types `F` + Enter exactly as for `XEQ "F"` — the same physical keystroke sequence they already know. Avoids the discovery problem of the rejected "manual ALPHA + type + R/S" option.

- **D-29.8: Extend `PendingInput::XeqByName { acc }` to `{ acc, mode: XeqByNameMode }`** where `XeqByNameMode` is a new enum with two variants: `Normal` (existing behavior — resolves to an Op on Enter) and `CollectForModal` (calls modal-advance on Enter). Compile-time exhaustive match over `XeqByNameMode` preserves the v2.2 FN-CLI-04 "no `_ =>` catch-all" invariant. `pending_prompt()` rendering and `pending_input` keystroke handling treat both modes identically except at the Enter-commit step.
  - **Why:** Both modes share the alpha-collection UX bit-for-bit; the routing-mode field is the smallest possible structural change. Rejected the heuristic-string-matching option (B) because it parses prompt text — fragile if prompt wording changes. Rejected the two-separate-variants option (C) because it duplicates the alpha-collection key handling for what is fundamentally one UX.

- **D-29.9: CLI `App` event loop auto-opens the `CollectForModal` modal based on state delta.** After each dispatch in `App::handle_key`, check: if `state.modal_program.is_some()` AND `modal_program.requires_alpha_label()` (a new method the planner adds — likely a match returning true for `Integ(FunctionNamePrompt)` / `Solve(FunctionNamePrompt)` / `Difeq(FunctionNamePrompt)`) AND `pending_input.is_none()`, set `pending_input = Some(PendingInput::XeqByName { acc: String::new(), mode: XeqByNameMode::CollectForModal })`. Detection lives in CLI; mirrored verbatim in `hp41-gui`'s IPC layer for parity. Core stays UI-agnostic — `requires_alpha_label` is a pure read-only method that takes no UI concerns.
  - **Why:** Single source of truth for "this modal step needs alpha input" lives in core (`requires_alpha_label`), but each frontend owns its own modal-opening mechanism. Core does NOT add a transient UI-request flag to `CalcState` (rejected option B) — that would couple core to a frontend concept. User does NOT have to manually press XEQ (rejected option C) — discovery problem.

### Claude's Discretion

- **Module location for `submit_modal` / `cancel_modal` / `requires_alpha_label`:** all three are public surface additions. Natural homes: `hp41-core/src/ops/math1/mod.rs` (top-level module dispatch) for `submit_modal`/`cancel_modal`; `hp41-core/src/ops/math1/modal.rs` (which already owns `ModalProgram` and per-program step state) for `requires_alpha_label`. Planner picks; constraint: the public surface must compile cleanly through `pub use` in `hp41-core/src/ops/math1/mod.rs`.
- **`submit_modal` internal shape:** a single exhaustive match on `state.modal_program` variant is natural (per-variant arm calls the matching per-program logic — `matrix::submit_order_step` / `poly::submit_degree_step` / etc.). Planner has discretion to either (a) inline the per-variant logic into `submit_modal` or (b) re-export per-module submit functions and dispatch via thin wrappers. Constraint: zero `_ =>` catch-alls.
- **`advance_with_label(label: &str)` signature shape:** could be a method on `ModalProgram`, a new variant of `submit_modal`'s parameters, or a separate function. Planner picks; constraint: the label string is passed by reference (not owned) and trimmed/uppercased per HP-41 convention before being used as a key into the user-program label table.
- **`function_matrix_parity.rs` extension:** the four existing parity tests assert v2.2 JSON ↔ Op-name bidirectional parity. Planner extends them to cover both JSON pools (v2.2 + Math Pac I) and both Op-name pools (`Op::*` variants + `xrom_resolve` registered mnemonics from `MATH_1.ops`). Approach: walk `help_entries_all()` once, partition entries by `xrom` field presence, assert each partition against its matching Op-name pool. Constraint: per-pool failure messages stay surgical so a future v3.1 Stat Pac doesn't break the v2.2 assertion. Planner can split into two test functions or use one parameterized table — `tests/function_matrix_parity.rs` is the only test file affected.
- **Category naming convention inside `hp41-math1-functions.json`:** v2.2 uses 20 enumerated categories (`"Arithmetic"`, `"Stack"`, `"Math"`, `"Trig"`, ...). Math Pac I needs distinct categories for the `?`-overlay sectioning. Planner picks between (a) one top-level category like `"Math 1 Pac"` for all ~55 entries, or (b) per-program categories like `"Math1 Hyperbolics"`, `"Math1 Complex"`, `"Math1 Matrix"`, `"Math1 Integration"`, etc. Recommendation: per-program categories — finer grouping helps users; help_overlay_rows already groups by first-appearance category order; users can still see Math Pac I sections cluster together.
- **`pending_prompt()` signature widening:** planner picks between `pub fn pending_prompt(app: &App) -> String` (takes the whole App and pulls state + pending) versus `pub fn pending_prompt(pending: &PendingInput, modal_prompt: Option<&str>) -> String` (narrower, explicit dependency on the modal_prompt slice). The latter is more testable in isolation; the former is more ergonomic in the render loop. Either is acceptable.
- **Precedence when `pending_input.is_some()` AND `state.modal_program.is_some()` simultaneously:** shouldn't happen in well-formed flows (the auto-open in D-29.9 only fires when `pending_input.is_none()`). Planner picks the well-defined behavior (likely modal_program wins) and adds a test asserting it.
- **JSON `divergences` field for Math Pac I entries:** the v2.2 schema's optional `divergences: Vec<String>` field is for HP-41 hardware-divergence notes. Math Pac I has known divergences (POLY multiplicity-as-cluster per D-28.x, INTG threshold tied to DisplayMode per ADR-004, the new `XEQ "REAL"` extension per D-28.3). Planner has discretion on which to populate per-entry; the full divergence catalog moves to `docs/hp41-math1-divergences.md` in Phase 30 (Phase 28 already seeded that file with the first entry).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.0 milestone scope, target feature areas, build sequence, key decisions ledger
- `.planning/REQUIREMENTS.md` — 110 v3.0 requirements; Phase 29 maps to CLI-01..05
- `.planning/ROADMAP.md` — Phase 29 section lines 93–119 (5 success criteria, 3 plans, notable risks/decisions); cross-cutting constraints lines 35–45
- `.planning/STATE.md` — accumulated context, key decisions carried forward
- `CLAUDE.md` (repo root) — v2.2 additions block, settled architecture decisions, JSON-canonical pipeline pattern (D-25.16/D-25.17/D-25.18)

### Phase 28 (the contract Phase 29 builds on)

- `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md` — full Phase 28 decisions; D-28.4 / D-28.5 / D-28.6 / D-28.7 carry forward to Phase 29
- `.planning/phases/28-…/28-01-SUMMARY.md` — XROM framework + resolver chain (lines 160–173 are the contract for the third call site Phase 29 closes)
- `.planning/phases/28-…/28-RESEARCH.md` — Math Pac I behavioral inventory; modal-state-machine layer rationale
- `.planning/phases/28-…/28-PATTERNS.md` — code-pattern analogues; relevant chapter: "JSON pipeline (D-25.16)" and "Hybrid PendingInput (D-25.11)"

### v2.2 baseline (the pattern reservoir)

- `hp41-cli/src/keys.rs:347` — `xeq_by_name_local_resolve` — the function Phase 29 extends; current `_ => None` arm becomes `_ => xrom_resolve(name, state.xrom_modules)` (the planner threads `xrom_modules` through the call-site or uses a module-level helper)
- `hp41-cli/src/help_data.rs` — `OnceLock<Vec<HelpEntry>>` pattern + D-25.16/D-25.17/D-25.18 doc comments; Phase 29 mirrors with the second OnceLock
- `hp41-cli/src/ui.rs:258` — `pending_prompt(pending: &PendingInput) -> String` — the function Phase 29 widens (D-29.3)
- `hp41-cli/src/ui.rs:229` — call site of `pending_prompt` — update accordingly when the signature changes
- `hp41-cli/src/app.rs::handle_key` — `App` event loop; Phase 29 inserts the modal R/S + Esc interception + post-dispatch auto-open hook (D-29.5 / D-29.6 / D-29.9). Critical: `pending_input` routing block must remain ABOVE modal-opening interceptors (D-07 never-discard invariant)
- `hp41-cli/src/app.rs` `PendingInput` enum — Phase 29 modifies the `XeqByName` variant to carry `mode: XeqByNameMode` (D-29.8); `pending_prompt()` exhaustive match stays exhaustive over all 18 v2.2 variants + the new XeqByName shape
- `hp41-cli/tests/function_matrix_parity.rs` — 4 parity tests Phase 29 extends to cover both JSON pools + both Op-name pools
- `hp41-cli/tests/phase25_help_data.rs` — 130-entry smoke test; Phase 29 adds a sibling `phase29_help_data_math1.rs` with the ~55-entry smoke
- `hp41-cli/tests/phase25_xeq_by_name.rs` — `cli_resolver_matches_core_resolver` integration test (T-25-09 mitigation); Phase 29 extends with a Math Pac I name to assert the local resolver and core resolver agree

### hp41-core public surface (the Math Pac I framework Phase 29 consumes)

- `hp41-core/src/ops/math1/xrom.rs` — `xrom_resolve(name, modules) -> Option<Op>`, `MATH_1: XromModule` constant with `ops: &[(name, Op)]` mapping; Phase 29 calls this from CLI's local resolver and the new parity test
- `hp41-core/src/ops/math1/modal.rs` — `ModalProgram` enum + per-program step enums (`MatrixInputStep`, `PolyInputStep`, `IntegInputStep`, `SolveInputStep`, `DifeqInputStep`, `FourInputStep`, `TransInputStep`) + `current_prompt()` methods; Phase 29 adds `requires_alpha_label(&self) -> bool` here
- `hp41-core/src/ops/math1/mod.rs` (or `modal.rs` — planner picks) — Phase 29 adds new public `submit_modal(state: &mut CalcState) -> HpResult<()>` and `cancel_modal(state: &mut CalcState)`
- `hp41-core/src/state.rs` — `CalcState` fields landed in Phase 28: `xrom_modules: u8`, `complex_mode: bool`, `matrix_dim`, `matrix_active_reg`, `modal_program: Option<ModalProgram>` (skip), `modal_prompt: Option<String>` (skip), `integ_state`/`solve_state`/`difeq_state` (skip), `cancel_requested: Arc<AtomicBool>` (skip). Phase 29 reads `modal_program` + `modal_prompt`; writes neither (Math Pac I ops own those mutations)
- `hp41-core/src/ops/program.rs:79, :521` — the two existing `xrom_resolve` call sites in `op_xeq` and `run_program::execute_op`; Phase 29's CLI-local call is the third, structurally identical site

### Math Pac I documentation already in place

- `docs/hp41-math1-divergences.md` — first entry seeded in Plan 28-07; Phase 30 expands. Phase 29 does NOT add new entries here.
- `docs/adr/v3.0-003-inv-epsilon.md` — ADR-003 (matrix inverse epsilon); written in Phase 28
- `docs/adr/v3.0-004-intg-threshold.md` — ADR-004 (INTG convergence threshold); written in Phase 28
- ADR-001 / ADR-002 / ADR-005 documents — NOT YET WRITTEN; Phase 30 / DOC-07 owns them. The decisions are recorded in PROJECT.md / STATE.md / 28-CONTEXT.md.
- `docs/verifying-math-pac-1.md` — operator walk-through procedure (Phase 28 deliverable); §9 catalogs the modal flows Phase 29 (CLI) + Phase 31 (GUI) close. Phase 29 SHOULD update §9 to mark the CLI side as ✅ once the modal flows work end-to-end.

### JSON pipeline (canonical pattern source)

- `docs/hp41cv-functions.json` — v2.2 schema source; Phase 29 mirrors structure for `hp41-math1-functions.json` (~55 entries, identical schema plus per-entry `xrom: { module: "Math 1", module_id: 7, function_id: <n> }` object per C-28.3)
- `scripts/docs-matrix/` (standalone non-workspace crate) — currently consumes one JSON; Phase 30 / DOC-02 extends to two-input. Phase 29 does NOT touch this crate (write the JSON; let Phase 30 wire the matrix regeneration).

### HP Math Pac I primary source (HP-copyrighted — DO NOT redistribute)

- HP-41C/CV Math Pac Owner's Manual (HP 00041-90034, 1979) — page references relevant to Phase 29:
  - p.13: "Press R/S to continue" — D-28.5 ground truth; D-29.5 routing rationale
  - p.14: MATRIX `ORDER=?` example — JSON description-field source for `MATRIX` entry
  - Function-name cross-reference: every Math Pac I mnemonic in `MATH_1.ops` needs a `display_name` matching the OM's documented spelling

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`hp41-cli/src/help_data.rs`** — entire pattern is reusable: `include_str!` + `OnceLock` + `serde::from_str` + `.expect("malformed")` hard-blocker + `help_entries()` accessor. Phase 29 clones the pattern for `hp41-math1-functions.json`.
- **`hp41-cli/src/keys.rs::key_ref_entries`** — already JSON-derived per D-25.18. After D-29.2 lands (merged accessor), Math Pac I entries with non-null `key_path` automatically appear in the right-panel discoverability listing without code changes to this function. Confirms SC-4.
- **`hp41-cli/src/ui.rs::pending_prompt`** — existing exhaustive match over 18 `PendingInput` variants. Phase 29 widens the signature (D-29.3) and extends behavior — does not reorder existing arms.
- **`hp41-cli/src/app.rs::PendingInput::XeqByName { acc }`** — existing v2.2 modal carrier for XEQ-by-name name collection. Phase 29 adds `mode: XeqByNameMode` (D-29.8); existing alpha-collection key handling, `acc.push(c)`, Backspace, ENTER commit, all reusable verbatim.
- **`hp41-cli/src/app.rs::flush_entry_buf`** — the established pipeline for converting `entry_buf` into a numeric value on the stack. D-29.5's `submit_modal` will call this (directly or via core) to consume user-typed digits for `ORDER=?` / `A1,1=?` / etc.
- **`hp41-core/src/ops/math1/xrom.rs::xrom_resolve`** — already does the name → Op mapping bounded by `state.xrom_modules` bitfield. Phase 29's CLI-local extension calls into it identically to the two existing core call sites.
- **`hp41-core/src/ops/math1/modal.rs::ModalProgram::current_prompt()` methods** — return `Option<String>` per variant. Phase 29's new `requires_alpha_label()` method follows the same per-variant shape (`Self::Integ(IntegInputStep::FunctionNamePrompt) => true`, etc.).
- **`hp41-cli/tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver`** — drift-detection pattern between the two resolvers. Phase 29 extends with Math Pac I name cases.

### Established Patterns

- **JSON canonical pipeline (D-25.16/17/18):** `include_str!` + `OnceLock` + hard-build-blocker on malformed JSON + JSON-derived KEY_REF_TABLE + bidirectional parity test. Phase 29 mirrors per the second-file shape (D-29.1/29.2).
- **PendingInput hybrid struct-variants (D-25.11):** the `XeqByName` variant already lives in this family. The new `mode: XeqByNameMode` field follows the same compile-time-exhaustive-match shape (FN-CLI-04 invariant).
- **Op variants land before consumers (CLAUDE.md "Op variants land before TUI code"):** doesn't apply directly here since Phase 29 adds ZERO new Op variants. The new public surface is `submit_modal` / `cancel_modal` / `requires_alpha_label` — three free functions / methods. Compile-time exhaustive matches in `submit_modal` provide the same compile-time safety the Op pattern provides.
- **D-25.6 CLI ↔ GUI parity:** every behavior added in Phase 29 routes through shared `hp41-core` code (`xrom_resolve`, `submit_modal`, `cancel_modal`, `requires_alpha_label`). Phase 31 frontend will call the SAME functions identically — no parallel implementations.
- **No `println!` / `eprintln!` in `hp41-core`** — Phase 29 doesn't add any to core. CLI's `pending_prompt` already runs in the ratatui render path (no stdout).

### Integration Points

- **`xeq_by_name_local_resolve` extension:** single-line change inside the `_ => None` arm at `hp41-cli/src/keys.rs:368` — becomes a final fallback into `xrom_resolve`. The function signature must thread `state.xrom_modules` through, or grow a module-level helper that reads it. Planner picks.
- **`pending_prompt` signature widening:** affects one call site in `hp41-cli/src/ui.rs:229`. Trivial caller-side update.
- **`PendingInput::XeqByName { acc, mode }`:** all pattern-match sites for this variant must be updated. Grep `PendingInput::XeqByName` to find them all; expected sites: `app.rs` (handler creation, key-routing, Enter-commit), `ui.rs` (`pending_prompt`), tests.
- **`App` event-loop post-dispatch hook (D-29.9):** new logic inserted after the existing `dispatch` call in the event loop; must NOT run during program execution (`state.is_running == true`) — only during interactive single-step dispatch. Existing `call_dispatch_and_drain` is the natural insertion point.
- **`help_entries_all()` migration:** three call sites — `help_overlay_rows`, `keys::key_ref_entries`, `tests::function_matrix_parity` — all migrate to the new merged accessor.

</code_context>

<specifics>
## Specific Ideas

- **JSON `key_path` format for Math Pac I entries:** identical to v2.2 conditional tests — `"XEQ \"SINH\""`, `"XEQ \"MATRIX\""`, `"XEQ \"INTG\""`, etc. Right-panel discoverability surfaces these verbatim. No special-casing.
- **JSON `xrom` block content (per C-28.3):** `{ "module": "Math 1", "module_id": 7, "function_id": <n> }`. `function_id` is the entry's position in `MATH_1.ops` (1-indexed) — planner has discretion to use 0-indexed, but 1-indexed matches HP-41 convention.
- **`requires_alpha_label()` initial coverage:** returns `true` exactly for `Integ(IntegInputStep::FunctionNamePrompt)`, `Solve(SolveInputStep::FunctionNamePrompt)`, `Difeq(DifeqInputStep::FunctionNamePrompt)`. All other modal steps require numeric input and `requires_alpha_label()` returns `false`. Grep `FunctionNamePrompt` in `hp41-core/src/ops/math1/modal.rs` to confirm the exact variant names; planner adjusts if the actual step enum names differ.
- **Esc binding sanity check:** v2.1 Phase 19 uses Esc for "cancel SHIFT armed" in the GUI. CLI's Esc binding (if any) currently goes to … the planner verifies. If Esc is already taken in CLI, fall back to a different key (Backspace? Ctrl+C is reserved for quit). Planner has discretion to pick the keybinding; constraint: it must be reachable in BOTH the v2.2 CLI key map and from `hp41-gui`'s frontend.
- **Verifying-math-pac-1.md §9 update:** Phase 29 plan 03 (or 04) should update the document's §9 table to mark each Math Pac I modal flow as ✅ available on CLI; the GUI column remains ❌ until Phase 31 ships.

</specifics>

<deferred>
## Deferred Ideas

- **GUI mirroring of all Phase 29 work** — Phase 31 plans 01–05. Every shared core function (`submit_modal`, `cancel_modal`, `requires_alpha_label`) is reused; every UI behavior (R/S interception, Esc cancellation, XeqByName CollectForModal mode, modal_prompt rendering, post-dispatch auto-open) gets a parallel implementation in `hp41-gui/src/App.tsx` + `hp41-gui/src-tauri/src/commands.rs`.
- **CATALOG 2 listing all loaded XROM modules** — Phase 31 / GUI-04. Math Pac I module enumeration UI lives in the GUI.
- **`request_cancel` Tauri command + cancel button** — Phase 31 / GUI-05. Phase 28's `cancel_requested: Arc<AtomicBool>` is fully plumbed in core; only the frontend trigger is missing.
- **5 ADR write-ups** — Phase 30 / DOC-07. The decisions exist; the human-readable rationale documents land later.
- **`docs/hp41-math1-divergences.md` expansion** — Phase 30 / DOC-04. Phase 29 does NOT add divergence entries here; if a Phase 29 implementation surfaces a new divergence (e.g., a subtle modal-prompt timing quirk), capture it as a deferred idea here and land it in Phase 30.
- **`scripts/docs-matrix/` two-input extension + `docs/hp41-math1-function-matrix.md` regeneration** — Phase 30 / DOC-02. Phase 29 ships the JSON; Phase 30 wires the matrix regeneration.
- **README v3.0 soft-claim + PROJECT.md / CLAUDE.md v3.0 additions block** — Phase 30 / DOC-05 + DOC-06.
- **Numerical-accuracy suite extension from 566 → ~700+ cases with Math Pac I citations** — Phase 32 / QUAL-02.
- **Per-Op test count ≥ 5 verification** — Phase 32 / QUAL-01. Phase 28 already added per-op tests; Phase 32 audits and closes gaps.
- **WebdriverIO E2E smoke extension with Math Pac I workflow** — Phase 32 / QUAL-03.
- **Free42 GPL-contamination guard in CI** — Phase 32 / QUAL-05.

</deferred>

---

*Phase: 29-cli-integration*
*Context gathered: 2026-05-17*
