# Roadmap — v3.0 Math Pac I Emulation

**Project:** HP-41 Calculator Emulator
**Milestone goal:** Behavioral Emulation des HP-41C Math Pac I (HP-Teilenummer 00041-90034, Owner's Manual 1979) als erstes XROM-Modul — 10 prompt-getriebene Workflow-Programme mit ~55 XEQ-by-Name Entry Points, nutzbar in CLI + GUI über eine neue Modal-Workflow-Schicht hinaus dem v2.x-Built-in-Pattern, ohne HP-copyrighted ROM-Image-Redistribution.

**Phase numbering:** continued from v2.2 (last shipped Phase 27). v3.0 phases start at **Phase 28**.

**Granularity:** standard (config.json). Phase count: 5 (28–32). Plans per phase: 10 / 3–4 / 4 / 4–5 / 3.

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, shipped 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix / STO modals / print / synthetic — SHIPPED 2026-05-09 · [Archive](milestones/v1.1-ROADMAP.md)
- ✅ **v2.0 Tauri GUI** — Phases 13–18, pixel-perfect HP-41C desktop app — SHIPPED 2026-05-10 · [Archive](milestones/v2.0-ROADMAP.md)
- ✅ **v2.1 Card Reader + Keyboard Authenticity** — quick-task entries (no Phase 19 GSD directory) — SHIPPED 2026-05-13 · see MILESTONES.md
- ✅ **v2.2 HP-41CV Feature Completeness** — Phases 20–27, full ROM built-in set + JSON pipeline + GUI integration + coverage gate raise — SHIPPED 2026-05-15 · [Archive](milestones/v2.2-ROADMAP.md)
- ⏳ **v3.0 Math Pac I Emulation** — Phases 28–32, first XROM application module (THIS DOCUMENT)

---

## Phases

### v3.0 — Math Pac I Emulation (Phases 28–32)

- [ ] **Phase 28: XROM Framework + Math Pac I Core Ops** — Land XROM registry, ~40 new `Op` variants, modal-workflow state machine, user-program callback infrastructure for INTG/SOLVE/DIFEQ, and all 10 Math Pac I top-level programs (`MATRIX`, `SOLVE`, `POLY`, `INTG`, `DIFEQ`, `FOUR`, complex stack, hyperbolics, triangle solvers, `TRANS`) in `hp41-core`
- [ ] **Phase 29: CLI Integration** — Wire `xeq_by_name_local_resolve` to call `xrom_resolve`, extend `help_data.rs` with a second JSON `OnceLock`, add ~40 `op_display_name` arms, surface modal prompts (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`) via existing `print_buffer` channel
- [ ] **Phase 30: Documentation & ADRs** — Publish `docs/hp41-math1-functions.json` (~55 entries), regenerate `docs/hp41-math1-function-matrix.md` via two-input `scripts/docs-matrix`, write 5 ADRs for Phase 28 irreversible decisions, README v3.0 soft-claim
- [ ] **Phase 31: GUI Integration** — Mirror CLI surface in `hp41-gui` (key_map XEQ-fallback, prgm_display arms, `?`-overlay JSON parallel-load, CATALOG 2, GUI modal-prompt rendering, cancellation channel for long-running INTG/SOLVE/DIFEQ)
- [ ] **Phase 32: Test Hardening** — Hold `hp41-core` coverage ≥ 95 %; extend `numerical_accuracy.rs` from 566 → ~700+ cases with Math Pac I citations; ≥ 5 tests per new `Op`; extend WebdriverIO E2E smoke with a Math Pac I workflow; Free42 GPL-contamination guard in CI

---

## Cross-cutting constraints (carried from v2.x)

- **SC-4 invariant**: no calculator logic duplication in `hp41-gui` (stricter grep for `op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)`). All Math Pac I math lives in `hp41-core/src/ops/math1/`. `op_display_name` in `prgm_display.rs` remains the only intentional display-formatter exception.
- **No HP-copyrighted ROM bytes** — behavioral emulation only (PROJECT.md scope-line). Per-file header comment in every Math Pac I op file: `// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979); Free42 source consulted only as sanity-check oracle, not copied.` Audit script `scripts/check-free42-contamination.sh` in CI.
- **`#![deny(clippy::unwrap_used)]`** continues to apply in `hp41-core`. Test modules carry `#[allow(clippy::unwrap_used)]` at the file scope.
- **`#[serde(default)]`** on every new `CalcState` field for v1.x–v2.2 save-file backward compatibility. Transient fields (`integ_state`, `solve_state`, `modal_program`) additionally carry `#[serde(skip)]`.
- **4-exhaustive-match invariant**: every new `Op` variant must land in `dispatch()` (ops/mod.rs) + `execute_op()` (ops/program.rs) + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. Compile-time exhaustive matches block missing arms.
- **CLI ↔ GUI parity** (D-25.6): every new Math Pac I function reachable in both surfaces through the shared `xrom_resolve` in `hp41-core`. No frontend-only resolver paths.
- **`pending_input` routing above modal interceptors** — D-07 (no silent discards) preserved for Math Pac I modal flows.
- **MSRV 1.88 unchanged.** Zero new runtime dependencies in `hp41-core`. One dev-dependency added: `approx 0.5.1` (relative-tolerance assertion macros for matrix/complex tests).

---

## Phase Details

### Phase 28: XROM Framework + Math Pac I Core Ops

**Goal**: Users can call every Math Pac I function (10 top-level programs + ~55 XEQ entry points) from `hp41-core` via a new XROM resolver chain — `XEQ "SINH"` returns `sinh(x)`, `XEQ "MATRIX"` opens the `ORDER=?` prompt flow, `XEQ "INTG"` accepts a user-program label and computes the Simpson integral with user-callback re-entrancy. All math logic lives in `hp41-core/src/ops/math1/`; SC-4 invariant trivially preserved.

**Depends on**: v2.2 baseline (Phase 27 shipped — coverage gate 95 %, JSON-canonical pipeline, hybrid PendingInput, builtin_card_op resolver chain).

**Build stage**: `hp41-core`

**Requirements (47 mapped)**: XROM-01..09 (framework), HYP-01..06 (hyperbolics), CMPLX-01..17 (complex stack), POLY-01..07, MAT-01..11, INTG-01..08, SOLV-01..08, DIFEQ-01..05, FOUR-01..06, TRI-01..05, TRANS-01..05.

**Success Criteria** (what must be TRUE):
  1. `xrom_resolve("SINH", state.xrom_modules)` returns `Some(Op::Sinh)`; `dispatch(state, Op::Sinh)` consumes X, writes `sinh(X)` back to X, declares `LiftEffect::Disable`; `xrom_resolve("UNKNOWN", state.xrom_modules)` returns `None`; the resolver fires LAST in the chain (after `builtin_card_op`) — verified by `tests/xrom_shadowing.rs` confirming no Math Pac I name shadows an existing built-in mnemonic
  2. `XEQ "MATRIX"` opens a `ModalProgram::Matrix(MatrixInputStep::OrderPrompt)` flow; entering `3` advances to `MatrixInputStep::ElementPrompt(0, 0)` with the prompt text `A1,1=?` written to `state.print_buffer`; entering nine values via the existing number-entry pipeline stores them column-major from R15 onward with order 3 in R14; `XEQ "DET"` then returns the correct determinant for the test case from Owner's Manual page 14
  3. `XEQ "INTG"` with a user-program LBL `"F"` defined as `LBL F / 1 / + / SIN / RTN` re-enters `run_loop` (NOT `run_program`) for each Simpson sample point; `state.call_stack` depth never exceeds 4; nested `XEQ "INTG"` from inside the user callback is rejected with `HpError::InvalidOp` per XROM-08; the 503-case v1.x numerical_accuracy baseline continues to pass at ≥ 498/503 (Phase 27 D-27.6 invariant preserved)
  4. `Op::CPlus` / `Op::CMinus` / `Op::CTimes` / `Op::CDiv` operate on the two-complex-number stack (ζ, τ); `Op::CDiv` with a zero divisor (re=0, im=0) returns `HpError::DivideByZero` BEFORE the division (Pitfall 6 mitigation); `complex_atan2(HpNum::ZERO, HpNum::ZERO)` returns `HpNum::ZERO` (not NaN, not DataError) and is unit-tested as the first arm of the f64-bridge function
  5. Every one of the ~40 new `Op` variants appears in `dispatch()` (ops/mod.rs), `execute_op()` (ops/program.rs), and BOTH `prgm_display.rs` copies (hp41-cli + hp41-gui); compile-time exhaustive matches confirm coverage; `hp41-core` continues to build with `#![deny(clippy::unwrap_used)]` and zero panics

**Plans**: 10 plans (per SUMMARY.md "Suggested phase structure" build order)
  - [ ] 28-01-PLAN.md — XROM framework: `ops/math1/xrom.rs` with `XromModule` struct + `MATH_1` const + `xrom_resolve(name, modules) -> Option<Op>` + 6 new CalcState fields (`xrom_modules`, `complex_mode`, `matrix_dim`, `matrix_active_reg`, `modal_program`, `integ_state`, `solve_state`) with `#[serde(default)]` / `#[serde(skip)]` where transient; resolver chain extension in `xeq_by_name_local_resolve` + `op_xeq` + `run_program::execute_op` (xrom fires LAST per Pitfall 1); 5 irreversible decisions locked via ADR (chosen Op-strategy A, user-callback strict-reject policy, INV-EPSILON post-OM, INTG-threshold post-OM, JSON-pipeline two-file shape); XROM-01..09
  - [ ] 28-02-PLAN.md — Hyperbolics (proof-of-pattern): `Op::{Sinh, Cosh, Tanh, Asinh, Acosh, Atanh}` + domain-error returns for `Acosh(X<1)` and `Atanh(|X|≥1)`; mirrors v2.2 one-shot stack-acting pattern; ≥ 5 tests per op per Pitfall 16; HYP-01..06
  - [ ] 28-03-PLAN.md — Complex stack scaffolding + arithmetic: `ComplexStack { zeta, tau }` location decision (overlay X/Y/Z/T vs dedicated R02–R05 vs eigener Struct — locked Plan 28-01); `Op::{CPlus, CMinus, CTimes, CDiv}` + `complex_atan2` f64-bridge handling (0,0)→0; `Op::CDiv` zero-divisor branch returns `HpError::DivideByZero`; CMPLX-01..05
  - [ ] 28-04-PLAN.md — Complex functions (13 ops): `Op::{Magz, Cinv, ZpowN, Zpow1N, ExpZ, LnZ, SinZ, CosZ, TanZ, ApowZ, LogZ, ZpowW, Zpow1W}`; branch-cut tests for `LnZ(0,0)` and `ZpowW` zero-divisor (Pitfall 6); CMPLX-06..17
  - [ ] 28-05-PLAN.md — POLY/ROOTS: `Op::{PolyWorkflow, Roots}` + modal `MatrixInputStep`-style state machine for `DEGREE=?` + `A=?`..`F=?` prompts; complex root-pair output format `U=u`/`V=v`/`U=u`/`-V=-v` (Pitfall 5 fidelity gate); multiplicity-as-cluster convention documented in `docs/hp41-math1-divergences.md`; non-convergence at `|imag| > 10⁹` returns `HpError::Domain` "DATA ERROR"; POLY-01..07
  - [ ] 28-06-PLAN.md — MATRIX workflow: `Op::{MatrixWorkflow, MatSize, MatVmat, MatEdit, MatDet, MatInv, MatSimeq, MatVcol}`; Gauss-Jordan inverse with hardware-sourced EPSILON (OM-transcribed in Plan 28-01 research-prep per Pitfall 7); order N in R14, column-major elements from R15 onward; flag 4 set during input, flag 5 set after SIMEQ-column-storage; max ORDER=14; `NO SOLUTION` display for singular matrices; MAT-01..11
  - [ ] 28-07-PLAN.md — INTG (with user-callback infrastructure): `Op::Integ` discrete (A=h, B=f(xⱼ), C=trapezoidal, D=Simpson, even-n check returns `N NOT EVEN`) + explicit mode (A=(a,b), B=n, `FUNCTION NAME?` prompt); `run_loop` re-entrancy from `op_integ()` (NOT `run_program` recursion — preserves outer program clone, avoids 30 KB × 1000 samples re-clone catastrophe); `state.call_stack` 4-deep cap enforced pre-mutation per `Op::XeqInd` precedent; subdivision cap 2^15; convergence threshold = `10^(-decimals - 1)` tied to `state.display_mode` (Pitfall 2 mitigation); `integ_state: Option<IntegState>` with `#[serde(skip)]`; INTG-01..08
  - [ ] 28-08-PLAN.md — SOLVE: `Op::{Solve, Sol}` + modified secant iteration (OM-spec); three termination paths `NO ROOT FOUND` / `ROOT IS <v>` / `ROOT IS BETWEEN <v1> AND <v2>` (Pitfall 3 mitigation, OM-cited branches); 100-iteration cap; reuses INTG's user-program callback infrastructure (same `run_loop` re-entrancy); `solve_state: Option<SolveState>` with `#[serde(skip)]`; nested INTG-in-SOLVE / SOLVE-in-INTG rejected per XROM-08; SOLV-01..08
  - [ ] 28-09-PLAN.md — DIFEQ: `Op::Difeq` + 4th-order Runge-Kutta + `FUNCTION NAME?` / `ORDER=?` (1 or 2) / `STEP SIZE=?` / `X0=?` / `Y0=?` (+ `Y'0=?` for 2nd order) prompts; step-by-step output via `print_buffer`; reuses INTG callback infrastructure; DIFEQ-01..05
  - [ ] 28-10-PLAN.md — FOUR + Triangle Solutions + TRANS (differentiators): `Op::Four` (DFT with `NO. SAMPLES=?` / `NO. FREQ=?` / `1ST COEFF=?` + `Y1..YN=?` + `RECT?` toggle, USER-mode `E`-key for evaluation); `Op::{TriSss, TriAsa, TriSaa, TriSas, TriSsa}` (Law of Sines/Cosines, ambiguous-case for SSA); `Op::{Trans2d, Trans3d}` (Rodrigues rotation for 3D); FOUR-01..06, TRI-01..05, TRANS-01..05

**Notable risks/decisions (Phase 28 is the gating phase)**:
  - **5 irreversible decisions** must be locked in Plan 28-01 research-prep BEFORE any implementation:
    1. **Op-strategy** (Option A vs B) — LOCKED A (one Op variant per Math Pac I function) per ADR-001
    2. **User-callback re-entrancy policy** — LOCKED strict-reject nested INTG/SOLVE/DIFEQ per ADR-002 (matches OM Hardware-Verhalten)
    3. **INV-EPSILON value** — TBD (community-cited 5e-10 vs OM-quoted 1e-9 per Pitfall 7) — MUST transcribe OM before Plan 28-06 lands; documented in ADR-003
    4. **INTG convergence threshold formula** — TBD (`threshold = 10^(-decimals - 1)` tied to `DisplayMode` per Pitfall 2) — MUST transcribe OM before Plan 28-07 lands; documented in ADR-004
    5. **JSON-pipeline shape** (separate file vs combined) — LOCKED separate `hp41-math1-functions.json` per ADR-005 (zero migration churn on 130 existing v2.2 entries)
  - Critical pitfalls in this phase: 1 (function-name collision; mitigated by xrom-fires-LAST), 2 (INTG threshold), 4 (user-callback re-entrancy), 5 (POLY clustering), 6 (complex branch cuts + (0,0) handling), 7 (matrix INV EPSILON)
  - **Cross-cutting**: enum-bloat regression (Pitfall 10) — `criterion bench/dispatch_overhead.rs` floor `< 200 ns/op` (v2.2 baseline 65 ns); per-Op test count ≥ 5 per Pitfall 16 to avoid mid-milestone coverage drop

**UI hint**: no (Phase 28 is purely `hp41-core`; no UI surface)

---

### Phase 29: CLI Integration

**Goal**: Every Math Pac I function reachable from `hp41-cli` via `XEQ`-by-name; ALPHA prompts (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`) surface in `state.print_buffer` and render in the TUI; `?`-overlay lists Math Pac I entries in their own section.

**Depends on**: Phase 28 (all `Op` variants must exist before keyboard / help wiring can compile).

**Build stage**: `hp41-cli`

**Requirements (5 mapped)**: CLI-01..05.

**Success Criteria** (what must be TRUE):
  1. `xeq_by_name_local_resolve("SINH")` in `hp41-cli/src/keys.rs` invokes `hp41_core::ops::math1::xrom_resolve` and returns `Op::Sinh`; pressing `X / S / I / N / H / Enter` in the XEQ-by-name modal executes `Op::Sinh` correctly; identical resolver path used inside `op_xeq`, `run_program`, and `run_loop` (D-25.6 CLI ↔ GUI parity preserved through shared hp41-core code)
  2. `hp41-cli/src/help_data.rs` loads a SECOND JSON file (`docs/hp41-math1-functions.json`) via an additional `OnceLock<Vec<HelpEntry>>`; the `?` overlay groups Math Pac I entries under a "Math 1 Pac" section distinct from the v2.2 HP-41CV built-ins; both JSON files load on first access per the v2.2 D-25.16 pattern
  3. `hp41-cli/src/prgm_display.rs` `op_display_name(op)` exhaustive match has ~40 new arms covering every Phase-28 `Op` variant (no `_ =>` catch-all); program listings show `SINH`, `MATRIX`, `INTG`, `C+`, `MAGZ`, etc. as their authentic HP-41 mnemonics
  4. `KEY_REF_TABLE` in `hp41-cli/src/ui.rs::render_right_panel` derives from `help_data::help_entries()` filtered by non-null `key_path` (D-25.18 pattern continues); Math Pac I entries appear in the right-panel discoverability listing without a parallel hand-curated table
  5. Modal-prompt routing for `MATRIX` / `SOLVE` / `POLY` / `INTG` / `DIFEQ` / `FOUR` / `TRANS` workflows: pressing the corresponding XEQ-by-name target triggers the `ModalProgram` state machine from Phase 28; prompt text (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`) appears in the TUI status bar / print panel; user input flows through the existing number-entry pipeline; ALPHA-text prompts for `FUNCTION NAME?` integrate with the v2.2 XEQ-by-name modal

**Plans**: 3 plans
  - [ ] 29-01-PLAN.md — XEQ-by-name resolver chain extension + help_data second OnceLock + JSON wiring; CLI-01, CLI-02
  - [ ] 29-02-PLAN.md — prgm_display.rs ~40 new arms + KEY_REF_TABLE derivation from JSON; CLI-03, CLI-04
  - [ ] 29-03-PLAN.md — Modal-prompt routing for Math Pac I workflows (re-uses Phase 28's `ModalProgram` infrastructure); CLI-05

**Notable risks/decisions**:
  - **Discovery problem** (Pitfall 13): 40 new mnemonics push the XEQ-by-name modal past the "easily scrollable" threshold; CATALOG 2 (Phase 31) is the structural fix; Phase 29 ships JSON-derived KEY_REF_TABLE entries so Math Pac I functions are at least visible in the `?`-overlay
  - **No core/GUI changes** in this phase — preserves the SC-4 invariant and keeps Phase 29 surgically scoped to `hp41-cli/` + `docs/`

**UI hint**: no (TUI-only; v2.2 already classified `hp41-cli` as non-frontend per the v2.2 ROADMAP convention)

---

### Phase 30: Documentation & ADRs

**Goal**: `docs/hp41-math1-functions.json` ships as the second authoritative source-of-truth alongside `hp41cv-functions.json`; `scripts/docs-matrix` regenerates `docs/hp41-math1-function-matrix.md` from both inputs; 5 ADRs document the Phase 28 irreversible decisions; README claims "Math Pac I behavioral emulation included"; `docs/hp41-math1-divergences.md` documents OM-Abweichungen (multiplicity-as-cluster for POLY, INTG-threshold-tying, FACT-extension-policy).

**Depends on**: Phase 29 (CLI integration validates JSON entries against actual `Op` variants via `tests/function_matrix_parity.rs`).

**Build stage**: `docs`

**Requirements (7 mapped)**: DOC-01..07.

**Success Criteria** (what must be TRUE):
  1. `docs/hp41-math1-functions.json` exists with ~55 entries; identical schema to `hp41cv-functions.json` plus `xrom: { module: "Math 1", module_id: 7, function_id: <n> }` object per entry; loads via `include_str!` + `OnceLock` from both `hp41-cli/src/help_data.rs` and (Phase 31) `hp41-gui/src/help_data.ts`; bidirectional parity test in `hp41-cli/tests/function_matrix_parity.rs` asserts every JSON entry has a matching `Op::*` variant and every `xrom_resolve` mnemonic has a JSON entry
  2. `scripts/docs-matrix/` (standalone non-workspace crate from v2.2 Plan 25-04) extended to two-input mode: reads both JSON files, regenerates `docs/hp41cv-function-matrix.md` (130 entries) AND `docs/hp41-math1-function-matrix.md` (~55 entries); `just docs-matrix` regenerates both; `just docs-matrix-check` is the CI drift-catch (mirror of v2.2 Pitfall 8 mitigation)
  3. `docs/hp41-math1-divergences.md` documents every Math Pac I behavior where the emulator's answer differs from the OM's quoted example by more than 1 ULP at 10-digit precision: multiplicity-as-cluster for POLY (Pitfall 5), INTG-threshold tied to DisplayMode (Pitfall 2), FACT-extension policy (kept v2.2 integer-only; new `GAMMA` not added — Pitfall 9 decision), strict-reject nested INTG/SOLVE per XROM-08
  4. `docs/adr/` directory carries 5 new ADR documents — ADR-001 (Op-Strategy A vs B), ADR-002 (User-Callback Re-entrancy Policy: strict-reject nested), ADR-003 (INV-EPSILON value post-OM-transcription), ADR-004 (INTG-Threshold Formula post-OM-transcription), ADR-005 (JSON-Pipeline Shape: separate `hp41-math1-functions.json`); each ADR cites Owner's Manual page / community source per Pitfall 18
  5. `README.md` carries the v3.0 soft-claim "Math Pac I behavioral emulation included (10 top-level programs, ~55 XEQ entry points, documented divergences)" + link to `docs/hp41-math1-function-matrix.md`; `PROJECT.md` and `CLAUDE.md` gain a "v3.0 additions" block analog to the v2.2 additions block

**Plans**: 4 plans
  - [ ] 30-01-PLAN.md — `docs/hp41-math1-functions.json` authoring (~55 entries) + JSON schema parity with `hp41cv-functions.json`; DOC-01
  - [ ] 30-02-PLAN.md — `scripts/docs-matrix` two-input extension + `just docs-matrix` + `just docs-matrix-check` CI gate; DOC-02, DOC-03
  - [ ] 30-03-PLAN.md — `docs/hp41-math1-divergences.md` authoring + 5 ADR documents (`docs/adr/v3.0-001-op-strategy.md` etc.); DOC-04, DOC-07
  - [ ] 30-04-PLAN.md — README soft-claim + PROJECT.md / CLAUDE.md v3.0 additions block; DOC-05, DOC-06

**Notable risks/decisions**:
  - **Pitfall 18 (citation provenance)**: every divergence-doc entry and ADR must cite OM page-and-example, MoHPC URL, or Mike Sebastian forensic page; no uncited assertions
  - **Pitfall 19 (Free42 GPL contamination)**: ADR-002 (user-callback policy) explicitly disclaims Free42; per-file header comment + audit script `scripts/check-free42-contamination.sh` documented here, enforced in CI from Phase 32

**UI hint**: no (documentation-only)

---

### Phase 31: GUI Integration

**Goal**: Math Pac I reaches users in `hp41-gui` through the same shared `xrom_resolve` in `hp41-core` (CLI ↔ GUI parity D-25.6 trivially preserved); `?`-overlay loads Math Pac I JSON in parallel; modal prompts render in the print panel below the LCD; `CATALOG 2` lists all loaded XROM modules; long-running INTG/SOLVE/DIFEQ are cancellable via a new `request_cancel` Tauri command.

**Depends on**: Phase 30 (JSON files must exist before parallel-loading in `?`-overlay; ADRs document the cancellation-channel design).

**Build stage**: `hp41-gui`

**Requirements (7 mapped)**: GUI-01..07.

**Success Criteria** (what must be TRUE):
  1. `hp41-gui/src-tauri/src/prgm_display.rs` `op_display_name(op)` exhaustive match has ~40 new arms covering every Phase-28 `Op` variant — SC-4 trivially holds because `op_display_name` is the documented display-formatter exception (CLAUDE.md "SC-4 invariant"); no `op_*` / `flush_entry_*` / `format_hpnum` functions added to `hp41-gui/src-tauri/src/`; stricter SC-4 grep returns nothing
  2. XEQ-by-name modal in `hp41-gui/src/App.tsx` resolves Math Pac I function names through the shared `xrom_resolve` in `hp41-core` (NO duplicate resolver in `key_map.rs`); typing `XEQ "SINH" 1.5 Enter` displays `2.1293` on the LCD; CLI ↔ GUI parity (D-25.6) automatically preserved through the shared hp41-core path; `key_map::resolve` stub-error arm shrinks NOT in v3.0 (Math Pac I has no dedicated keys — only XEQ-by-name per GUI-07)
  3. `?`-overlay loads `docs/hp41-math1-functions.json` in parallel with `hp41cv-functions.json` via Vite JSON-import; Math Pac I functions appear as a categorized section ("Math 1 Pac") distinct from the HP-41CV built-ins; `CATALOG 2` implementation (new `Op::Catalog(2)` arm in Phase 28) lists all loaded XROM modules with their function counts; reachable via existing `catalog` key
  4. **Cancellation channel** (Pitfall 11 mitigation): `state.cancel_requested: Arc<AtomicBool>` (new field, `#[serde(default, skip)]`) + new `request_cancel` Tauri command + permission file `hp41-gui/src-tauri/permissions/request-cancel.toml` (per v2.2 Pitfall 21 mitigation); `op_integ` / `op_solve` / `op_difeq` check the AtomicBool every 64 samples and return `HpError::Interrupted` if set; `op_integ` releases-and-reacquires the AppState Mutex between sample batches so the 30s auto-save thread and `request_cancel` can interleave; the R/S key on the frontend routes to `request_cancel` (NOT `run_stop` — different semantic per Phase 31 research-prep)
  5. **Modal-prompt rendering**: `ORDER=?` / `A1,1=?` / `FUNCTION NAME?` / `GUESS 1=?` prompts written to `state.print_buffer` by Math Pac I ops appear in the existing scrollable print panel below the LCD; user input via Number-Entry with ENTER to confirm; ESC cancels the modal; no new IPC surface (drains through the existing `print_buffer` channel established in v1.1 Phase 11)

**Plans**: 5 plans
  - [ ] 31-01-PLAN.md — `hp41-gui/src-tauri/src/prgm_display.rs` ~40 new arms; SC-4 verification grep test extends to cover new files; GUI-01
  - [ ] 31-02-PLAN.md — Cancellation channel: `cancel_requested: Arc<AtomicBool>` field + `request_cancel` Tauri command + permissions TOML + `op_integ` / `op_solve` / `op_difeq` integration (every-64-samples check + lock release); GUI-05
  - [ ] 31-03-PLAN.md — XEQ modal resolves Math Pac I functions through shared `xrom_resolve`; D-25.6 CLI ↔ GUI parity verification test; GUI-02
  - [ ] 31-04-PLAN.md — `?`-overlay parallel-loads Math Pac I JSON via Vite JSON-import; categorized section rendering; CATALOG 2 implementation; GUI-03, GUI-04
  - [ ] 31-05-PLAN.md — Modal-prompt rendering: re-uses existing print-panel channel; user-input via Number-Entry pipeline; ESC cancellation; GUI-06, GUI-07 (stub-arm policy preserved)

**Notable risks/decisions**:
  - **Pitfall 11 (GUI freeze on long INTG)**: cancellation channel is the structural fix; Plan 31-02 research-prep specifies the lock-release strategy (every 64 samples) and verifies the auto-save thread + `request_cancel` can interleave without deadlock
  - **Pitfall 12 (mid-solver save state)**: `integ_state` / `solve_state` / `modal_program` all `#[serde(default, skip)]` from Phase 28 first commit — preserved through to Phase 31 GUI hooks (round-trip test in Phase 32)
  - **Pitfall 21 (Tauri permissions)**: `request_cancel` requires a new TOML in `hp41-gui/src-tauri/permissions/request-cancel.toml`; CI gate `scripts/check-tauri-permissions.sh` (v2.2 pattern) verifies every `generate_handler!` member has a matching permission

**UI hint**: yes (Math Pac I prompts render via print panel + LCD; cancellation channel surfaces in frontend UI; `?`-overlay extension; CATALOG 2 listing)

---

### Phase 32: Test Hardening

**Goal**: `hp41-core` coverage held ≥ 95 % lines / ≥ 93 % regions; `numerical_accuracy.rs` extended from 566 → ~700+ cases with Math Pac I cases per program (OM-cited per case per D-27.1); ≥ 5 tests per new `Op` (per Pitfall 16 to avoid mid-milestone coverage drop); WebdriverIO E2E smoke extended with one Math Pac I workflow; Free42 GPL-contamination guard in CI; cross-platform numerical-drift tolerance documented.

**Depends on**: Phase 31 (all functionality must be in place before final coverage push and E2E test extension).

**Build stage**: `tests`

**Requirements (8 mapped)**: QUAL-01..08.

**Success Criteria** (what must be TRUE):
  1. `just coverage` reports `hp41-core` line coverage ≥ 95.0 % (gate held from v2.2 — NO atomic raise this milestone per D-27.2 lessons-learned); regions ≥ 93 %; the 5 new `ops/math1/*.rs` source files reach ≥ 90 % each via the 5 new test files (`math1_complex_edge_cases.rs`, `math1_user_callback.rs`, `xrom_shadowing.rs`, plus extensions to `numerical_accuracy.rs` and `program_execution_coverage.rs`); per-Op test count ≥ 5 invariant enforced by `tests/math1_op_test_count.rs`
  2. `hp41-core/tests/numerical_accuracy.rs` grows from 566 → ~700+ cases with Math-Pac-I-specific cases per program (POLY, MATRIX, INTG, SOLVE, DIFEQ, FOUR, complex, hyperbolics, triangles, TRANS); each new case carries an Owner's Manual page+example citation in a `// Source: HP 00041-90034 p.<n>, ex.<m>` doc-comment per D-27.7 pattern; combined pass rate ≥ 98 %; v1.x 503-case baseline floor 498/503 preserved per D-27.6 (independently asserted)
  3. **E2E smoke extension (FN-QUAL-03)**: `hp41-gui/e2e/smoke.spec.ts` carries a second test case clicking a Math Pac I workflow — `XEQ "SINH" 1 Enter` expects LCD reads `1.1752` (sinh(1) to 4 decimals) OR a MATRIX mini-flow (`XEQ "MATRIX" 2 Enter 1 Enter 2 Enter 3 Enter 4 Enter XEQ "DET" Enter` expects LCD reads `-2.0000`); runs only in `e2e-linux` job on Ubuntu (matches v2.2 D-27.15 AMENDED); existing `2 ENTER 3 +` smoke test unchanged
  4. **Free42 GPL-contamination guard (QUAL-05)**: every file in `hp41-core/src/ops/math1/` carries a header comment `// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979); Free42 source consulted only as sanity-check oracle, not copied.`; CI gate `scripts/check-free42-contamination.sh` greps for distinctive Free42 variable names + asserts the header comment is present in every math1 file; runs in `just ci`
  5. **Cross-platform drift (QUAL-06)**: every Math Pac I numerical test uses `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` (Math Pac I floor — 6 of HP-41's 10 digits guaranteed per Pitfall 14); zero `assert_eq!(decimal, decimal)` on iterated results in `tests/math1_*.rs` (lint enforced by `tests/lint_math1_assertions.rs` per Pitfall 17); `tests/math1_user_callback.rs` carries 5 regression tests for user-callback re-entrancy (nested INTG/SOLVE rejection, STO clobbering, STOP-during-INTG, GTO-out-of-callback, recursion-cap); `tests/xrom_shadowing.rs` asserts no Math Pac I name shadows an existing built-in mnemonic (Pitfall 1 CI gate)

**Plans**: 3 plans
  - [ ] 32-01-PLAN.md — Coverage push for Math Pac I ops (per-Op test count ≥ 5; close gaps surfaced by `just coverage`); `tests/xrom_shadowing.rs` + `tests/math1_user_callback.rs` + `tests/lint_math1_assertions.rs`; QUAL-01, QUAL-04, QUAL-07, QUAL-08
  - [ ] 32-02-PLAN.md — `numerical_accuracy.rs` extension from 566 → ~700+ cases with OM citations; `approx 0.5.1` dev-dep added; relative-tolerance discipline; QUAL-02, QUAL-06
  - [ ] 32-03-PLAN.md — E2E smoke extension (one Math Pac I workflow in `hp41-gui/e2e/smoke.spec.ts`); Free42-contamination guard (`scripts/check-free42-contamination.sh` in CI); `data-testid="lcd-display"` carries Math Pac I-mode output; QUAL-03, QUAL-05

**Notable risks/decisions**:
  - **Coverage gate NOT raised** in v3.0 — held at v2.2 level (≥ 95 % lines / ≥ 93 % regions) per D-27.2 lessons-learned (atomic raise requires risk-weighted tests, not coverage padding)
  - **Per-Op test count ≥ 5** (Pitfall 16) — risk-weighted per D-27.3, NOT coverage padding; documented `// Catches: <bug class>` comments per D-27.1
  - **Pitfall 14 (cross-platform drift)**: relative tolerance 1e-7 documented as Math Pac I floor; criterion benchmarks pinned to ubuntu-latest; three-OS CI catches x86-vs-ARM divergence on first PR

**UI hint**: no (test-only)

---

## Build Sequence

```
Phase 28 (hp41-core)
   │  framework + 5 ADR decisions + ~40 new Op variants + modal state machine + user-callback re-entrancy
   ▼
Phase 29 (hp41-cli)
   │  xeq_by_name_local_resolve + help_data second JSON + prgm_display arms + modal-prompt routing
   ▼
Phase 30 (docs)
   │  hp41-math1-functions.json + docs-matrix two-input + 5 ADRs + divergences doc + README claim
   ▼
Phase 31 (hp41-gui)
   │  prgm_display arms + cancellation channel + XEQ modal + ?-overlay parallel-load + CATALOG 2 + modal rendering
   ▼
Phase 32 (tests)
      coverage hold + 566→700+ accuracy cases + E2E Math Pac I smoke + Free42 contamination guard + drift tolerance
```

**Do not break this order.** Phase 28 contains 5 irreversible decisions (Op-strategy, user-callback policy, INV-EPSILON, INTG-threshold, JSON-pipeline shape) that gate every downstream phase. Plan 28-01 research-prep MUST transcribe OM for ADR-003 (INV-EPSILON) and ADR-004 (INTG-threshold) BEFORE any implementation lands.

---

## Coverage

✓ **110 / 110 v3.0 requirements mapped** to phases 28–32
✓ No orphaned requirements
✓ No requirement appears in more than one phase
✓ Each phase has 2–5 observable success criteria
✓ Each phase has a single dominant build stage (core / cli / docs / gui / tests)

Traceability table is maintained in `.planning/REQUIREMENTS.md` "Traceability" section.

---

## Progress Table

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 28. XROM Framework + Math Pac I Core Ops | v3.0 | 0/10 | Planned | |
| 29. CLI Integration | v3.0 | 0/3 | Planned | |
| 30. Documentation & ADRs | v3.0 | 0/4 | Planned | |
| 31. GUI Integration | v3.0 | 0/5 | Planned | |
| 32. Test Hardening | v3.0 | 0/3 | Planned | |

---

## v2.x ROADMAP archives

- v1.0: `milestones/v1.0-ROADMAP.md`
- v1.1: `milestones/v1.1-ROADMAP.md`
- v2.0: `milestones/v2.0-ROADMAP.md`
- v2.2: `milestones/v2.2-ROADMAP.md`

---

*Last updated: 2026-05-16 — v3.0 ROADMAP drafted by `/gsd:roadmapper`; 5 phases (28–32), 25 plans, 110 requirements mapped 1:1; Phase 28 carries 5 irreversible decisions; awaiting `/gsd:plan-phase 28`.*
