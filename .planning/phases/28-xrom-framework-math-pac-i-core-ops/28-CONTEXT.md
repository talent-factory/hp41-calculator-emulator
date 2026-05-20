# Phase 28: XROM Framework + Math Pac I Core Ops — Context

**Gathered:** 2026-05-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 28 is the gating phase of milestone v3.0. It lands the XROM resolver framework and all Math Pac I math logic in `hp41-core/src/ops/math1/` — 10 prompt-driven workflow programs (`MATRIX`, `SOLVE`, `POLY`, `INTG`, `DIFEQ`, `FOUR`, complex-stack, hyperbolics, triangle solvers, `TRANS`) exposed via ~40 new `Op` variants reachable through ~55 XEQ-by-name entry points. Phase 28 is `hp41-core`-only — zero UI surface, zero `hp41-cli` / `hp41-gui` source touched. SC-4 invariant trivially holds. All 110 v3.0 requirements that map to Phase 28 (XROM-01..09, HYP-01..06, CMPLX-01..17, POLY-01..07, MAT-01..11, INTG-01..08, SOLV-01..08, DIFEQ-01..05, FOUR-01..06, TRI-01..05, TRANS-01..05) live in this phase across 10 plans (28-01 framework + ADRs → 28-10 FOUR + triangles + TRANS).

**In scope:**
- XROM framework: `XromModule` struct, `pub const MATH_1: XromModule { id: 7, … }`, `xrom_resolve(name, modules) -> Option<Op>` resolver
- Resolver-chain extension in `xeq_by_name_local_resolve` + `op_xeq` + `run_program::execute_op` (xrom fires LAST per Pitfall 1)
- 7 new `CalcState` fields with `#[serde(default)]` / `#[serde(skip)]` as appropriate: `xrom_modules: u8`, `complex_mode: bool`, `matrix_dim: Option<(u8, u8)>`, `matrix_active_reg: Option<u8>`, `modal_program: Option<ModalProgram>` (skip), `modal_prompt: Option<String>` (skip — per D-28.4), `integ_state` + `solve_state` (skip), `cancel_requested: Arc<AtomicBool>` (skip — per D-28.9)
- ~40 new `Op` variants per Op-strategy A (locked in PROJECT.md / STATE.md ADR-001)
- Modal-state-machine layer (`ModalProgram` enum + per-program step states) alongside v2.2 `PendingInput`
- User-program callback infrastructure: `run_loop` re-entrancy from `op_integ` / `op_solve` / `op_difeq`; pre-mutation 4-deep `call_stack` cap
- Per-64-samples cancellation check stubs in long-running solvers (Pitfall 11 plumbing — wiring lands Phase 31)

**Out of scope (explicit):**
- Any `hp41-cli` / `hp41-gui` source changes — Phase 29 (CLI) and Phase 31 (GUI) own those
- Tauri `request_cancel` command + GUI cancel button — Phase 31 / GUI-05
- `docs/hp41-math1-functions.json` + matrix regeneration — Phase 30 / DOC-01..03
- ADR documents — Phase 30 / DOC-07 (the *decisions* are locked in Plan 28-01 research-prep; the *ADR write-ups* live in Phase 30)
- Module-Pac emulation beyond Math Pac I (Stat 1, Time, Advanced Matrix, Advantage) — permanent v3.1+ boundary per REQUIREMENTS.md
- HP-copyrighted ROM image redistribution — permanent exclusion (behavioral emulation only)

**Mandated by ROADMAP cross-cutting constraints (lines 90–130 of `.planning/ROADMAP.md` and Phase 28 risks/decisions section):**
- 5 irreversible decisions (Op-strategy A, user-callback strict-reject, INV-EPSILON, INTG-threshold, JSON-pipeline shape) MUST be locked in Plan 28-01 research-prep BEFORE any implementation plan lands; OM page-references for ADR-003/ADR-004 MUST be transcribed before Plan 28-06 / Plan 28-07 lands
- `criterion bench/dispatch_overhead.rs` floor `< 200 ns/op` (v2.2 baseline 65 ns) — guards against enum-bloat regression (Pitfall 10)
- Per-Op test count ≥ 5 (Pitfall 16) — keeps Phase 32 coverage gate at 95 % reachable without mid-milestone scramble
- `#![deny(clippy::unwrap_used)]` continues to apply throughout `hp41-core/src/ops/math1/`
- SC-4 invariant: every new `Op` variant must land in BOTH `prgm_display.rs` copies (hp41-cli + hp41-gui) — but those edits happen in Phase 29 / Phase 31, not here

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / STATE.md / REQUIREMENTS.md (carried forward — NOT re-decided here)

- **C-28.1 (ADR-001):** Op-strategy A — one `Op` variant per Math Pac I function. Rejected Option B (`Op::XromCall(u16)` table dispatch) preserves the 4-way exhaustive-match invariant that has caught dozens of bugs since Phase 1.
  - **Why:** zero new test surface, compile-time `prgm_display.rs` coverage check, structurally identical to v1.1's `synthetic_byte_to_op` resolver.
- **C-28.2 (ADR-002):** User-callback re-entrancy — strict-reject nested INTG/SOLVE/DIFEQ at op entry with `HpError::InvalidOp`.
  - **Why:** matches Math Pac I OM 1979 Hardware-Verhalten; simplest invariant; avoids 4-deep `call_stack` overflow + cleanup-on-error complexity.
- **C-28.3 (ADR-005):** JSON-pipeline shape — separate `docs/hp41-math1-functions.json` file (sibling to `hp41cv-functions.json`), identical schema plus `xrom: { module, module_id, function_id }` object per entry.
  - **Why:** zero migration churn on 130 existing v2.2 entries; cleaner per-test scoping; aligns with future v3.1+ pacs each getting their own JSON.
- **C-28.4:** `xrom_resolve` fires LAST in the resolver chain (after `builtin_card_op`, before `Err(InvalidOp)`).
  - **Why:** Pitfall 1 mitigation — prevents Math Pac I from shadowing existing built-in mnemonics; `tests/xrom_shadowing.rs` CI gate confirms.
- **C-28.5:** `run_loop` (NOT `run_program`) re-entry for INTG/SOLVE/DIFEQ user-program callback.
  - **Why:** preserves outer program clone; avoids 30 KB × 1000 samples re-clone catastrophe; mirrors `Op::XeqInd` precedent at `hp41-core/src/ops/program.rs:479`.

### Discussed and decided in this session (D-28.1 — D-28.9)

#### ComplexStack location (CMPLX-01)

- **D-28.1: Overlay X/Y/Z/T — ζ = X+iY, τ = Z+iT.** Zero new HpNum storage fields on `CalcState`; just one new `complex_mode: bool` flag. Math Pac I OM 1979 describes the complex stack as an overlay; SUMMARY.md §"Complex Stack & Operations" lines 66–68 already canonicalizes this. Rejected (b) "dedicated R02–R05" because user `STO 02` would silently clobber the complex stack. Rejected (c) "new ComplexStack struct" because it ABWEICHT vom OM-Modell and pads every save-file with 4 extra HpNum fields.
  - **Why:** OM-faithful + zero save-file growth + no new clobbering surface. The cost — number-entry must learn `complex_mode` semantics — is contained in `flush_entry_buf()` (one function) and is testable in isolation.

- **D-28.2: `complex_mode: bool` auto-on at first complex-op, explicit XEQ "REAL" to deactivate.** First `C+` / `C-` / `C×` / `C÷` / `MAGZ` / `Z↑N` / etc. sets `complex_mode = true` AND triggers display annunciator hint ("r i" or similar). Explicit `XEQ "REAL"` clears it. Save-file load defaults to `complex_mode: false` (safe — re-activates on first complex op anyway). Rejected "always-on while Math Pac I loaded" because users would lose pure-real arithmetic without unloading XROM. Rejected "per-op transient (no flag)" because Display14Seg cannot render `re i` annunciator without a state bit.
  - **Why:** auto-on matches the implicit-state-machine pattern from v2.2 (`shift_armed`, `eex_mode`); explicit-off keeps the user in control; safe default on load avoids "phantom complex mode" surprises after restart.

- **D-28.3: `XEQ "REAL"` is a new derived XROM entry point — NOT in Math Pac I OM 1979.** Added solely to honor D-28.2's "explicit-off" choice. Documented as our extension in `docs/hp41-math1-divergences.md`; counts as a NEW requirement to add to REQUIREMENTS.md before Plan 28-04 lands (suggest `CMPLX-18: Op::Real (XEQ "REAL") — deactivates complex_mode; resets it to false; no other side effects on the stack`).
  - **Why:** OM-fidelity vs UX-pragmatism trade-off — this is the cleanest place to land the deviation. Adding REAL avoids needing a global "kill complex mode" toggle in settings or a magic key combo.

#### Modal-prompt channel (XROM-09 supersedes)

- **D-28.4: New `modal_prompt: Option<String>` field on `CalcState` with `#[serde(skip)]` (transient, never persisted).** REQUIREMENTS XROM-09 originally specified "Prompts via `state.print_buffer` (existing channel)" — this decision OVERRIDES that wording. `state.print_buffer` continues to carry PRX/PRA/PRSTK output ONLY; modal prompts (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`, etc.) write to `modal_prompt`. CLI renders `modal_prompt` in the existing `pending_prompt()` TUI line (Phase 29 wiring); GUI renders it as an overlay banner above the LCD (Phase 31 wiring). Rejected "LCD display direct overwrite" because 12-char limit truncates "FUNCTION NAME?" (14 chars) and would require scroll plumbing that doesn't exist.
  - **Why:** clean separation = clean testing. `print_buffer` semantics (drain on every command, scrollback in GUI print panel) stay unchanged. Modal prompts get their own transient state with clear lifecycle (set on prompt-open, cleared on prompt-resolve or modal-cancel).

- **D-28.5: R/S key submits numeric input in a modal prompt.** User types digits into `entry_buf`, presses R/S → `flush_entry_buf()` → `ModalProgram` advances → `modal_prompt` updates to next prompt (or clears on completion). Rejected ENTER-submits because ENTER's stack-push semantics conflict with MATRIX-Edit's "next element" flow (where ENTER would push a number that the modal then has to pop). Rejected auto-advance-on-complete-input because timing-dependent tests are brittle and users don't get to review entry before commit.
  - **Why:** Math Pac I OM 1979 page 13 explicitly says "Press R/S to continue"; reuses existing `run_stop` Tauri command from v2.1; HP-41 hardware-faithful.

#### Hyperbolics UX policy (HYP-01..06)

- **D-28.6: XEQ-by-name only — NO dedicated key bindings in `hp41-cli/src/keys.rs` or `hp41-gui/src/Keyboard.tsx`.** Hyperbolics (`SINH`, `COSH`, `TANH`, `ASINH`, `ACOSH`, `ATANH`) reach the user via the same XEQ-by-name mechanism as `MATRIX` / `SOLVE` / `POLY` etc. — full keyboard sequence `XEQ ALPHA S I N H ALPHA`. Rejected "f-prefix on SIN/COS/TAN" because the real HP-41C reserves f-prefix for `SIN⁻¹` / `COS⁻¹` / `TAN⁻¹` (already wired in v2.2). Rejected "new h-prefix" because that divergiert from HP-41 hardware and crosses the behavioral-emulation scope line.
  - **Why:** consistency over ergonomics. Real HP-41C with Math Pac I in slot also has no dedicated hyp keys — users invoke via XEQ-by-name. Keeping our v3.0 UX identical preserves the OM-faithful claim. Plan 28-02 must still ship hyperbolics first (proof-of-pattern for one-shot stack-acting ops in the Math Pac I family) but reaches them via the resolver chain, not dedicated keyboard arms.

#### Cancellation field timing (Pitfall 11)

- **D-28.7: Plumbing in Phase 28, wiring in Phase 31.** `CalcState.cancel_requested: Arc<AtomicBool>` lands in Plan 28-01 framework with `#[serde(skip)]` and `Arc::new(AtomicBool::new(false))` default. Reset to `false` at every `op_integ` / `op_solve` / `op_difeq` entry. Per-64-samples check inside the user-callback re-entry loop returns `Err(HpError::Canceled)` on `load(Ordering::Relaxed) == true`. Phase 31 / GUI-05 adds the `request_cancel` Tauri command + UI button only — zero subsequent edits to `hp41-core/src/ops/math1/` needed.
  - **Why:** "Op variants land before consumers" pattern (already documented in CLAUDE.md). Editing solver loops once and shipping them dormant is cheaper than re-opening Plan 28-07 / 28-08 / 28-09 in Phase 31 to thread a new field. Cost — `Arc<AtomicBool>` allocation per `CalcState::new()` — is negligible (~16 bytes + atomic, zero hot-path overhead since load is `Relaxed`).

- **D-28.8: Per-64-samples cadence — NOT per-sample.** Loop check fires every 64th iteration of the Simpson sum, secant step, or RK4 step. Granular enough for sub-second cancel responsiveness even at 32768-subdivision INTG, coarse enough that the atomic load doesn't dominate the inner loop.
  - **Why:** matches the pattern already used in v2.2 `run_loop` for `MAX_STEPS = 1_000_000` budget checks; cancellation responsiveness budget is < 50 ms at typical sample counts (1000 samples × 1 ms/sample / 64 = ~16 ms worst case).

- **D-28.9: `HpError::Canceled` is a NEW variant on `HpError` enum.** Distinct from `HpError::Domain("DATA ERROR")` because cancellation is user-initiated, not a numerical failure. `Display` impl returns `"CANCELED"`. Plan 28-01 ships this variant; CLI's `?` overlay and GUI's toast layer (Phases 29 / 31) format it. Save-file forward-compat unaffected — `HpError` is never serialized.
  - **Why:** clean error model = clean UX. Conflating cancel with "DATA ERROR" would surface as a misleading numerical-failure toast in the GUI; users would think their function blew up rather than that they hit cancel.

### Claude's Discretion

- **Number-entry semantics in `complex_mode`:** when complex_mode is active and user types `3.5 ENTER 2`, this is interpreted per HP-41 Math Pac I OM convention (`3.5 ENTER 2` → ζ with re=3.5, im=2 in X+iY). Stack-lift behavior for complex ops: C+/C-/C×/C÷ consume ζ AND τ (both 2-tuples = 4 stack levels), write result to ζ, T-replicate pattern fills τ from old T. Planner has discretion to match OM bit-for-bit; tests cover edge cases from OM worked examples (page-cited per D-27.1 pattern).
- **`integ_state` / `solve_state` struct shape:** transient mid-iteration values needed by Simpson's rule, secant iteration, RK4 stepping. Planner picks field layout from OM-cited algorithm; both carry `#[serde(skip)]` so save-file shape is irrelevant. Constraint: each struct must be `Default::default()`-constructible so `CalcState::new()` doesn't break.
- **`ModalProgram` enum variants:** one per Math Pac I top-level program with sub-mode where needed (`Matrix(MatrixInputStep)`, `Solve(SolveInputStep)`, `Integ(IntegInputStep)`, `Poly(PolyInputStep)`, `Difeq(DifeqInputStep)`, `Four(FourInputStep)`, `Trans(TransInputStep)`). Sub-enums carry the per-step state (`MatrixInputStep::OrderPrompt`, `::ElementPrompt(i, j)`, etc.). Planner has discretion to shape these to match OM workflow per program; constraint: `pending_prompt()` exhaustive match stays exhaustive (`_ =>` is forbidden per FN-CLI-04 precedent from Phase 25).
- **POLY multiplicity-as-cluster:** REQUIREMENTS POLY-06 says "matched OM hardware-cluster behavior." Planner documents this as a behavioral divergence from "ideal" multiplicity in `docs/hp41-math1-divergences.md` (Phase 30 / DOC-04). NO snap-to-zero post-processing; we faithfully reproduce the small-imaginary-part roots Math Pac I returns.
- **Triangle SSA ambiguous case (TRI-05):** OM-conformes Behandeln per REQUIREMENTS. Planner pattern-matches the two-solutions case from OM worked examples; output format mirrors OM display sequence.
- **Free42 contamination policy strictness:** per QUAL-05 the per-file header + audit script is the baseline. Planner has discretion to add a `git pre-commit` hook on `hp41-core/src/ops/math1/` that runs the audit; not blocking — script in Phase 32 / QUAL-05 covers it.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.0 milestone scope, target feature areas, build sequence, key decisions ledger (entries for ADR-001/002/005 already present)
- `.planning/REQUIREMENTS.md` — 110 v3.0 requirements; Phase 28 maps to 90 of them (XROM-01..09, HYP-01..06, CMPLX-01..17, POLY-01..07, MAT-01..11, INTG-01..08, SOLV-01..08, DIFEQ-01..05, FOUR-01..06, TRI-01..05, TRANS-01..05)
- `.planning/ROADMAP.md` — Phase 28 section (5 success criteria, 10 plans, 5 irreversible decisions, critical pitfalls 1/2/4/5/6/7/10/16)
- `.planning/STATE.md` — accumulated context, key decisions carried forward, critical implementation traps (Free42 GPL contamination, every new Op needs 4-place exhaustive-match update, `#[serde(default)]` on new fields)
- `CLAUDE.md` (repo root) — v2.2 additions block, settled architecture decisions (BCD/f64, stack-lift, ISG/DSE, no async, zero panics, MSRV 1.88, ratatui invariants)

### v3.0 research (consumed by gsd-phase-researcher and gsd-planner)

- `.planning/research/STACK.md` — 4 crate-level decisions (HpNum currency, hand-coded ComplexHp + MatrixView + Simpson/secant/RK4, reject num-complex / nalgebra / faer / ndarray / gauss-quad / argmin)
- `.planning/research/FEATURES.md` — authoritative Math Pac I function inventory (table-stakes vs differentiator vs anti-feature classification)
- `.planning/research/ARCHITECTURE.md` — Op-strategy A justification, CalcState field plan, re-entrancy plumbing, modal state machine layer, XEQ-by-name resolver chain extension
- `.planning/research/PITFALLS.md` — catalogue (function names need re-mapping to Math Pac I names; categories translate cleanly)
- `.planning/research/SUMMARY.md` — executive synthesis; scope-correction note (2026-05-16) explaining what changed from the original assumed scope

### v2.2 baseline (the contract Phase 28 builds on)

- `hp41-core/src/ops/mod.rs` — `Op` enum + `dispatch()` central integration hub; XROM resolver chain extension lands here
- `hp41-core/src/ops/program.rs` — `run_loop()`, `run_program()`, `execute_op()`, `parse_counter()`, `Op::XeqInd` precedent at `:479` for pre-mutation `call_stack` cap; new `xrom_resolve` call inserted after `builtin_card_op`
- `hp41-core/src/state.rs` — `CalcState` single source of truth; 7 new fields land here per the In Scope list
- `hp41-core/src/ops/registers.rs` — pattern for `op_sto_arith` reuse in Math Pac I scratch-register management
- `hp41-cli/src/help_data.rs` — `OnceLock<Vec<HelpEntry>>` pattern; Phase 29 adds a SECOND `OnceLock` for `hp41-math1-functions.json`
- `docs/hp41cv-functions.json` — JSON schema sibling format (Phase 30 / DOC-01 produces `hp41-math1-functions.json` with the same schema + `xrom` object)

### HP Math Pac I primary source (HP-copyrighted — DO NOT redistribute)

- HP-41C/CV Math Pac Owner's Manual (HP 00041-90034, 1979) — pages referenced per requirement category:
  - p.13: "Press R/S to continue" — D-28.5 ground truth
  - p.14: MATRIX worked example (3×3 determinant) — Plan 28-06 acceptance test source
  - p.234 analogy from v2.2 (MOD sign semantics, Free42 cross-checked) — pattern for D-28.x test-citation comments

### Free42 (reference oracle only — NOT a source for copying)

- `https://thomasokken.com/free42/` and `https://github.com/thomasokken/free42` — public GPL source, used as a sanity-check oracle for numerical-method ground truth (per the established v2.2 Phase 27 D-27.7 pattern: "Cross-checked against Free42 source ops_math.cc::do_X — Free42 returns Y, matching HP-41C OM p.Z"). Per-file headers in `hp41-core/src/ops/math1/` must disclaim Free42 source copying; `scripts/check-free42-contamination.sh` (Phase 32 / QUAL-05) enforces.

### v3.0 ADR documents (PRODUCED in Phase 30; the *decisions* are locked in Plan 28-01 research-prep)

- `docs/adr/v3.0-001-op-strategy.md` — ADR-001 write-up of C-28.1 (Op-strategy A)
- `docs/adr/v3.0-002-user-callback-policy.md` — ADR-002 write-up of C-28.2 (strict-reject nested)
- `docs/adr/v3.0-003-inv-epsilon.md` — ADR-003 (post-OM-transcription, value locked in Plan 28-01)
- `docs/adr/v3.0-004-intg-threshold.md` — ADR-004 (post-OM-transcription, formula locked in Plan 28-01)
- `docs/adr/v3.0-005-json-pipeline.md` — ADR-005 write-up of C-28.3 (separate JSON file)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`hp41-core/src/ops/mod.rs::synthetic_byte_to_op`** — pattern for `xrom_resolve`: string-keyed lookup returning a regular `Op::*` variant; never bypasses the exhaustive match. New `hp41-core/src/ops/math1/xrom.rs::xrom_resolve` is structurally identical.
- **`hp41-core/src/ops/program.rs::Op::XeqInd` at `:479`** — pre-mutation `call_stack` 4-deep cap pattern; mirror this in `op_integ` / `op_solve` / `op_difeq` user-callback re-entry guards.
- **`hp41-cli/src/help_data.rs` `OnceLock<Vec<HelpEntry>>`** — Phase 29 adds a SECOND `OnceLock` for `hp41-math1-functions.json` with the same lazy-init pattern.
- **Hybrid `PendingInput` struct-variants (D-25.11)** — `ModalProgram` enum mirrors the consolidation philosophy: collapse ~10 logical workflow programs into one carrier with per-program step state. Same exhaustive-match safety.
- **`shift_armed: bool` one-shot pattern (v2.2 Phase 25, D-25.6)** — `complex_mode: bool` follows the same shape: implicit auto-on, explicit-off, frontend-visible-state. Save-file load defaults to safe false.
- **`run_stop` Tauri command (v2.1 Phase 19)** — R/S key already toggles `is_running`; Phase 28 reuses the same key for modal-prompt submit (D-28.5).

### Established Patterns

- **`#![deny(clippy::unwrap_used)]` at `hp41-core/src/lib.rs`** — applies to all new `hp41-core/src/ops/math1/` files. Test modules carry `#[allow(clippy::unwrap_used)]` at file scope per the Phase 1+ pattern.
- **Every new `Op` variant must land in 4 places:** `dispatch()` (`ops/mod.rs`) + `execute_op()` (`ops/program.rs`) + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. The 3rd and 4th edits ship in Phase 29 + Phase 31 respectively — Phase 28 only owns the first two.
- **New `CalcState` fields need `#[serde(default)]`** for v1.0–v2.2 save-file backward compat. Transient fields (`integ_state`, `solve_state`, `modal_program`, `modal_prompt`, `cancel_requested`) additionally carry `#[serde(skip)]`.
- **`pending_input` routing block must remain ABOVE modal-opening interceptors** in `hp41-cli/src/app.rs` — Phase 29 must extend, not reorder.
- **No `println!` / `eprintln!` in `hp41-core`** — route side effects via `print_buffer` (PRX/PRA/PRSTK), `modal_prompt` (new — per D-28.4), or `event_buffer`.
- **`format_hpnum()` lives in `hp41-core/src/format.rs`** — Math Pac I prompts (`A1,1=?` showing actual element value) reuse this; never duplicated into hp41-gui (SC-4 invariant).
- **Coverage gate `≥ 95 %` lines on `hp41-core`** is held via Phase 32 / QUAL-01. Per-Op test count ≥ 5 (Pitfall 16) must be respected in every Plan 28-02..28-10.
- **`criterion bench/dispatch_overhead.rs`** floor `< 200 ns/op` — enum-bloat regression guard (Pitfall 10). Bench run as part of `just bench` (advisory, not CI-gated). v2.2 baseline 65 ns/op; ~40 new variants must not push past 200 ns/op (typical headroom is comfortable).

### Integration Points

- **Resolver chain extension (1 site, 3 callers):** `xrom_resolve` is called from `xeq_by_name_local_resolve` (`hp41-core/src/ops/mod.rs`), `op_xeq` (`hp41-core/src/ops/program.rs`), and `run_program::execute_op` (`hp41-core/src/ops/program.rs`). All three callers receive the same `state.xrom_modules` bitfield and call signature.
- **`tests/xrom_shadowing.rs` CI gate:** confirms no Math Pac I name shadows an existing built-in mnemonic. New test file lands in Plan 28-01.
- **`hp41-core/tests/numerical_accuracy.rs`:** 566-case baseline (v2.2). Plan 28-02 through 28-10 each add ≥ 5 hand-curated cases per new `Op` per Pitfall 16, citing OM page/example. Phase 32 / QUAL-02 extends the suite to ~700+ cases and re-asserts ≥ 98 % pass rate.
- **`docs/hp41-math1-divergences.md`:** new file (Phase 30 / DOC-04) documenting D-28.3 (XEQ "REAL" extension), POLY-06 multiplicity-as-cluster, INTG-08 threshold formula, FACT-extension-policy (carried from v2.2), and any other Math-Pac-I-specific quirks the planner identifies.

</code_context>

<specifics>
## Specific Ideas

- **D-28.5 R/S key submit ground truth:** "Press R/S to continue" — HP-41C/CV Math Pac Owner's Manual (HP 00041-90034, 1979) page 13.
- **D-28.1 overlay-stack ground truth:** "ζ = X+iY, τ = Z+iT" overlay — already documented in `.planning/research/SUMMARY.md` §"Complex Stack & Operations" lines 66–68; OM 1979 pages 24–26 are the primary source.
- **D-28.6 hyperbolics-via-XEQ ground truth:** physical HP-41C with Math Pac I cartridge has no dedicated hyperbolic keys — users invoke via `XEQ ALPHA S I N H ALPHA`. User confirmed via direct hardware inspection (the cartridge is currently in his calculator's slot, per PROJECT.md scope correction note 2026-05-16).
- **`tests/xrom_shadowing.rs` failure modes to test:** every Math Pac I XEQ name in `MATH_1.ops` must NOT collide with any v2.2 built-in mnemonic (`SIN`, `COS`, `STO`, `ARCL`, etc.). The shadowing test iterates `MATH_1.ops` and asserts `key_to_op(name)` / `shifted_key_to_op(name)` / `builtin_card_op(name)` all return `None` first.

</specifics>

<deferred>
## Deferred Ideas

- **GUI cancellation UI (request_cancel Tauri command + cancel button)** — Phase 31 / GUI-05. Phase 28 only ships the field + per-64-samples check stub.
- **ADR write-ups (5 documents in `docs/adr/v3.0-*.md`)** — Phase 30 / DOC-07. The decisions are locked in Plan 28-01 research-prep; the human-readable rationale documents ship later.
- **Tauri `request_cancel` command + GUI cancel button** — Phase 31 / GUI-05.
- **CLI modal-prompt rendering in `pending_prompt()`** — Phase 29 / CLI-05. Phase 28 only writes `modal_prompt: Option<String>`; CLI reads and renders it.
- **GUI modal-prompt overlay banner** — Phase 31 / GUI-06. Same field, different reader.
- **`hp41-math1-functions.json` + `docs/hp41-math1-function-matrix.md` regeneration + `docs/hp41-math1-divergences.md`** — Phase 30 / DOC-01..04.
- **Free42-contamination-guard audit script** — Phase 32 / QUAL-05. Per-file header comments ship with every Plan 28-02..28-10 file though.
- **Cross-platform numerical drift `approx::assert_relative_eq!` with `max_relative = 1e-7`** — Phase 32 / QUAL-06.
- **Math-Pac-I E2E smoke (WebdriverIO)** — Phase 32 / QUAL-03.
- **CATALOG 2 listing all loaded XROM modules** — Phase 31 / GUI-04.

</deferred>

---

*Phase: 28-xrom-framework-math-pac-i-core-ops*
*Context gathered: 2026-05-16*
