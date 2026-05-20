# Research Summary: HP-41 Calculator Emulator v3.0 — Math Pac I

**Synthesized:** 2026-05-16
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md, PROJECT.md
**Milestone:** v3.0 — HP-41C Math Pac I (HP 00041-90034) behavioral emulation as the first XROM application module on top of the shipped v2.2 ROM-built-in calculator

---

## Executive Summary

v3.0 lifts the HP-41C/CV emulator out of its built-in-ROM comfort zone and into **application-module territory** for the first time. The deliverable is behavioral emulation of the HP-41C Math Pac I, a 1979 plug-in cartridge containing 10 top-level prompt-driven workflow programs (`MATRIX`, `SOLVE`, `POLY`, `INTG`, `DIFEQ`, `FOUR`, complex-stack arithmetic, hyperbolics, triangle solutions, `TRANS` coordinate transforms) with roughly 55 named XEQ entry points. The technology cost is zero new runtime crates; the architectural cost is targeted — six new `CalcState` fields, ~40 new `Op` variants, one extension of the existing XEQ-by-name resolver chain, and a new prompt-driven modal layer alongside the v2.2 `PendingInput` system. The biggest risk is concentrated in two places: re-entering `run_loop` for the user-program callback that `SOLVE` / `INTG` / `DIFEQ` need, and choosing numerical-method algorithms that match the Owner's Manual's worked examples bit-for-bit (the silent-wrong-answer failure mode).

This is architecturally **integration, not redesign**. Every load-bearing pattern was settled in v1.0–v2.2: `CalcState` single source of truth, `LiftEffect`-tagged dispatch, JSON-canonical help pipeline, four-way exhaustive-match invariant, CLI ↔ GUI parity through shared hp41-core resolvers, `#[serde(default)]` save-file backward compat. Math Pac I inherits all of them. The single hard architectural choice — **Option A: one `Op` variant per Math Pac I function** — preserves the compile-time safety net that has caught dozens of bugs since Phase 1. The corresponding XROM registry (`xrom_resolve` in a new `hp41-core/src/ops/math1/`) is structurally identical to v1.1's `synthetic_byte_to_op` resolver: a string-keyed lookup that returns regular `Op::*` variants, never bypassing the exhaustive match.

The recommended phase count is **5 (Phases 28–32)**: framework + math-1 ops in hp41-core, CLI integration, documentation, GUI integration, test hardening — mirroring v2.2's `core → cli → docs → gui → tests` shape. The framework first (Phase 28) is the gating phase; everything else is incremental.

---

## Scope Correction Note (2026-05-16)

The downstream consumer prompt originally described v3.0 around `M+`, `MAT*`, `INV` (matrix transpose semantics), `PROOT`, `CABS`, `CARG`, `CCHS`, `CCONJ`, `V+`, `V-`, `VDOT` — discrete one-shot stack-acting ops. The FEATURES research, using the primary-source 1979 Math Pac Owner's Manual (HP 00041-90034) as ground truth, established that **those function names belong to the Advanced Matrix Pac and the Advantage Pac**, NOT to Math Pac I. The user confirmed the discovery on 2026-05-16 and locked v3.0 scope to **the real Math Pac I** — the physical cartridge currently in his calculator's slot.

**What this changes:**

| Aspect | Originally assumed | Actual (Math Pac I) |
|--------|--------------------|---------------------|
| UX style | One-shot stack ops, v2.2-compatible | Multi-step prompt-driven modal flows (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`) |
| Function count | ~40 discrete ops | 10 top-level programs with ~55 XEQ entry points |
| Implementation | Pure dispatch + LiftEffect | Modal state machine layered on `PendingInput`, plus user-program callback for `SOLVE`/`INTG`/`DIFEQ` |
| Matrix ops | `M+`/`MAT*`/`INV` etc. | `MATRIX` workflow (input → DET → INV → SIMEQ → VMAT → EDIT → VCOL); flag 4/5 protocol; up to 14×14 |
| Complex math | `CABS`/`CARG`/`CCHS`/`CCONJ` discrete | Two-complex-number stack (ζ/τ); `C+`/`C-`/`C×`/`C÷` arithmetic + 13 functions including `MAGZ`, `Z↑N`, `E↑Z`, `LNZ`, `SINZ` |
| Polynomial | `PROOT` (Advantage Pac) | `POLY`/`ROOTS` for degree 2–5 with `U=u`/`V=v`/`U=u`/`-V=-v` complex-pair output |

**What stays valid from the originally-assumed scope:**

The four crate-level decisions in STACK.md hold for EITHER scope — zero new runtime dependencies, hand-coded numerics on top of `rust_decimal`/`HpNum` with documented f64 bridges, `approx 0.5.1` as a single dev-dependency, static-linked XROM dispatch framework with no `.mod` runtime loader. The Op-strategy decision in ARCHITECTURE.md (Option A: one `Op` variant per Math Pac I function) is also scope-independent. The pitfalls catalogue in PITFALLS.md was drafted around the older scope assumption (it cites `PROOT`, `INTEG`, `CARG`, `MAT*` as if they were Math Pac I); the **pitfall categories** translate cleanly to the real scope (modal state, user-program callback re-entrancy, numerical-method ground truth, XROM-namespace collisions, long-running solver freezing the GUI, save-file mid-modal state, test coverage drift), but the specific function names cited need re-mapping to the real Math Pac I names (e.g. `PROOT` → `POLY`/`ROOTS`, `CARG` → not present, branch-cut concerns shift to `LNZ`/`Z↑W`).

**Roadmapper guidance:** prefer FEATURES.md (lines 36–222) as the authoritative function inventory. Use STACK.md's four decisions as written. Translate PITFALLS.md function names to FEATURES.md equivalents when phase-mapping. ARCHITECTURE.md's Op-strategy, CalcState-field, and re-entrancy plumbing are correct; its example Op variants (`Op::MatPlus`, `Op::CAdd`, `Op::PRoot`, `Op::VPlus`) need to be re-named to Math Pac I conventions (e.g. `Op::MatrixWorkflow`, `Op::CPlus`, `Op::PolyWorkflow`, `Op::Sinh`).

---

## Stack Decisions (STACK.md condensed)

Four crate-level calls, all scope-independent:

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | **HpNum is the unit of currency for every Math Pac I op** | Maintains v1.0–v2.2 type discipline (`rust_decimal` BCD with 10-digit rounding). Internal f64 bridges permitted in three documented cases (complex `atan2`, matrix LU pivoting on ≥ 5×5, polynomial-root iteration) using the established `num.rs::checked_asin` pattern. Save-file determinism and the 566-case accuracy harness's tolerance model rely on this. |
| 2 | **Roll our own `ComplexHp { re, im }` — reject `num-complex 0.4.6`** | `Complex::sqrt/exp/ln/powc` require `T: Float`; `rust_decimal::Decimal` cannot satisfy `Float` (no IEEE-754 NaN/INFINITY layout). Forcing `Complex<f64>` at every stack boundary would trash 5+ digits of precision. Hand-coded `ComplexHp` lives cleanly inside `#![deny(clippy::unwrap_used)]`. |
| 3 | **Hand-coded `MatrixView` — reject `nalgebra` / `faer` / `ndarray`** | All three impose `T: Float`/`T: Scalar` bounds that exclude `HpNum`; nalgebra alone adds ~150 KB compiled + ~10 transitive deps; ≤ 14×14 matrices never trip the BLAS path. Triple-loop multiplication, Gauss-Jordan inverse, and LU determinant fit in ~50 LOC each. |
| 4 | **Hand-coded Simpson `INTG` + secant `SOLVE` + Runge-Kutta `DIFEQ` — reject `gauss-quad` / `quadrature` / `argmin` / `roots`** | Behavioral spec requires algorithm-faithful match to the Owner's Manual's worked examples; third-party iteration crates guarantee subtle output divergence. All three algorithms are documented in the 1979 OM and fit in well-bounded LOC. |

**Dev-dependencies:** `approx 0.5.1` only (relative-tolerance assertions for matrix / complex tests). **Runtime additions: zero.** MSRV stays at 1.88.

**XROM framework:** static dispatch pattern (`pub const MATH_1: XromModule { id: 7, name: "MATH 1A", ops: &[...] }`), NOT a `.mod` file loader — dynamic loading is permanently excluded for legal reasons (HP-copyrighted ROM bytes never redistributed per PROJECT.md).

---

## Feature Inventory by Category (FEATURES.md)

Table-stakes (T) = required for the "feature-complete Math Pac" claim. Differentiator (D) = ships fidelity, not strictly required. Anti-feature (A) = explicit defer/exclude.

### Hyperbolics — 6 ops (T)
`SINH`, `COSH`, `TANH`, `ASINH`, `ACOSH`, `ATANH`. The ONLY family with one-shot UX matching v2.2 patterns. Each is `f(X) → X` via `rust_decimal` math, ~3 lines per op. **Lowest-cost / highest-confidence target** — implement first as proof of pattern.

### Complex Stack & Operations — 17 ops (T arithmetic, T+D functions)
Two-complex-number stack (ζ = X+iY, τ = Z+iT overlay) plus `C+`/`C-`/`C×`/`C÷` (T), `MAGZ`, `CINV`, `Z↑N`, `Z↑1/N`, `E↑Z`, `LNZ`, `SINZ`, `COSZ`, `TANZ` (T), and `A↑Z`, `LOGZ`, `Z↑W`, `Z↑1/W` (D). **No `CABS`/`CARG`/`CCHS`/`CCONJ` in Math Pac I** — those names belong to Advantage Pac. The pac's marquee feature. Uses R00–R04.

### Polynomial Roots — 1 program (T)
`POLY` (master, degree 2–5) + `ROOTS` (sub-entry) + evaluation. Coefficients R00–R04, working storage R00–R22. Complex root pair output in 4-line `U=u`/`V=v`/`U=u`/`-V=-v` format — exact reproduction is a fidelity gate. **No `PROOT`** — that's Advantage Pac.

### Matrix Workflow — 1 program with 8 entry points (T)
`MATRIX` master + `SIZE` + `VMAT` + `EDIT` + `DET` + `INV` + `SIMEQ` + `VCOL`. Gaussian elimination with partial pivoting. Order N at R14; column-major matrix elements from R15 onward. Up to 14×14. Flag 4 set during input phase, flag 5 set after SIMEQ stores column. `NO SOLUTION` display for singular matrices. **The most complex single program in the pac.**

### Numerical Integration — 1 program, 2 modes (T)
`INTG` discrete (T) — A=h, B=f(xⱼ) sample input, C=trapezoidal, D=Simpson; even-n check returns `N NOT EVEN`. `INTG` explicit (T) — A=(a,b) interval, B=n with `FUNCTION NAME?` prompt, Simpson with fixed n (no adaptive refinement; user controls accuracy). Uses R00–R07. **First feature requiring user-program callback infrastructure.**

### Real Root Solver — 1 program (T)
`SOLVE` master with `FUNCTION NAME?` + `GUESS 1=?` + `GUESS 2=?` prompts. `SOL` sub-entry bypasses prompting. Modified secant iteration. Three termination paths: `NO ROOT FOUND`, `ROOT IS <v>`, `ROOT IS BETWEEN <v1> AND <v2>`. Uses R00–R06. **Reuses INTG's callback infrastructure.**

### Differential Equations — 1 program (D)
`DIFEQ` with `FUNCTION NAME?` / `ORDER=?` (1 or 2) / `STEP SIZE=?` / `X0=?` / `Y0=?` (+ `Y'0=?` for 2nd order). 4th-order Runge-Kutta. Uses R00–R07.

### Fourier Series — 1 program (D)
`FOUR` with `NO. SAMPLES=?` / `NO. FREQ=?` / `1ST COEFF=?` + `Y1..YN=?` + `RECT?` toggle. USER-mode `E` key evaluates series at t after coefficients computed. Up to 10 (aₙ, bₙ) pairs. Uses R00–R26.

### Triangle Solutions — 5 programs (D)
`SSS`, `ASA`, `SAA`, `SAS`, `SSA`. Each is its own prompt flow. Law of Sines / Cosines. `SSA` has ambiguous-case handling.

### Coordinate Transformations — 1 program (D)
`TRANS` 2D (A=init `x₀,y₀,θ`, C=forward, E=inverse) and 3D (A=origin, B=`a,b,c,θ` rotation axis, C=forward, E=inverse via Rodrigues' rotation). Uses R00–R24.

### Anti-Features — Explicit defers / excludes (A)
- `M+`/`M-`/`MAT*`/`TRANS` (transpose)/`IDN`/`RSUM`/`CSUM`/`MMOVE` → Advanced Matrix Pac, future v3.2+ scope
- `V+`/`V-`/`VDOT`/`VLEN`/`VANG` → Advanced Matrix Pac, future v3.2+ scope
- `PROOT` → Advantage Pac, future v3.3+ scope
- `CABS`/`CARG`/`CCHS`/`CCONJ`/`CPOLAR`/`CRECT`/`CSQRT`/`CEXP`/`CLN`/`CY^X` → Advantage Pac naming, future v3.3+ (Math Pac I uses `MAGZ` + the Z↑/E↑Z/LNZ family)
- Romberg adaptive integration → Advantage Pac's `∫f(x)`, not Math Pac I
- `GAMMA`/`ERF`/`BESSEL`/probability distributions → Stat Pac or out-of-scope permanently
- Cycle-accurate Nut-CPU execution, `.rom` redistribution, magnetic-card-loading the pac → permanently excluded (legal + scope)

---

## Architecture Highlights (ARCHITECTURE.md)

### Op-strategy: Option A (one Op variant per function)

CHOSEN. Adds ~40 variants to `Op` enum (`Op::MatrixWorkflow`, `Op::CPlus`, `Op::Magz`, `Op::PolyWorkflow`, `Op::Sinh`, `Op::Integ`, `Op::Solve`, ...). Preserves the 4-exhaustive-match invariant (`dispatch()`, `execute_op()`, `hp41-cli/src/prgm_display.rs`, `hp41-gui/src-tauri/src/prgm_display.rs`) that has caught dozens of bugs Phases 1–27. **Rejected Option B** (`Op::XromCall(u16)` table dispatch) — forfeits exhaustive-match safety, doesn't unlock any v3.x requirement, would double the source-of-truth surface for tests.

The XROM registry (`hp41-core/src/ops/math1/xrom.rs`) is a **resolver**, not a dispatcher — structurally identical to v1.1's `synthetic_byte_to_op` (returns a regular `Op::*` variant; dispatch flows through the normal exhaustive match).

### CalcState additions — six fields, all `#[serde(default)]`

```rust
xrom_modules: u8,              // bitfield; bit 0 = Math 1 loaded (default 0b1)
complex_mode: bool,             // ζ/τ stack overlay active
matrix_dim: Option<(u8, u8)>,   // active matrix dimensions
matrix_active_reg: Option<u8>,  // base register pointer for matrix
integ_state: Option<IntegState>,  // #[serde(skip)] — transient, never persisted
solve_state: Option<SolveState>,  // #[serde(skip)] — transient, never persisted
modal_program: Option<ModalProgram>,  // multi-step prompt flow state (NEW — driven by FEATURES.md)
```

v1.0–v2.2 save files load cleanly via `#[serde(default = "default_xrom_modules")]`. v3.0 → v2.2 forward-compat drops the new fields silently per serde's unknown-field policy.

### Re-entrancy: `run_loop`, NOT `run_program`

`INTG`/`SOLVE`/`DIFEQ` re-enter `run_loop` directly (reusing the outer `run_program`'s program clone — avoids 30 KB × 1000 samples re-clone catastrophe). `state.is_running` stays `true` throughout. The 4-deep `state.call_stack` cap is preserved with pre-mutation guards (mirrors `Op::XeqInd` precedent at `hp41-core/src/ops/program.rs:479`). **Nested INTG/SOLVE is rejected at op entry** (`HpError::InvalidOp`) — matches the real Math Pac I ROM's documented behavior.

### Modal state machine — new infrastructure (FEATURES.md → ARCHITECTURE.md mapping)

Math Pac I is prompt-driven (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, ...) — fundamentally different from v2.2's one-shot stack ops. **New modal layer** lives alongside the v2.2 `PendingInput` system but is structurally distinct: `ModalProgram` enum with per-program step state (`MatrixInputStep::OrderPrompt`, `::ElementPrompt(i, j)`, etc.). Prompts surface via `state.print_buffer` (existing channel; no new IPC).

### XEQ-by-name resolver chain extension

The single targeted hp41-core change. Current v2.2 chain: user-LBL → `xeq_by_name_local_resolve` → `builtin_card_op` → `Err(InvalidOp)`. v3.0 adds `xrom_resolve(name, state.xrom_modules)` AFTER `builtin_card_op`. The CLI fast-path (`hp41-cli/src/keys.rs:347`) and GUI modal (`hp41-gui/src/App.tsx` XEQ modal) both go through this — CLI ↔ GUI parity per D-25.6 is preserved automatically because the fallback lives in shared hp41-core code.

### JSON-canonical pipeline — separate file per module

NEW: `docs/hp41-math1-functions.json` (sibling to `docs/hp41cv-functions.json`), identical schema plus an `xrom: { module, module_id, function_id }` object per entry. `scripts/docs-matrix/` extended to two-input mode. `hp41-cli/src/help_data.rs` adds a SECOND `OnceLock<Vec<HelpEntry>>`. v2.2 JSON file is **unchanged** (no migration of 130 existing entries).

### Build sequence (PROJECT.md — `core → cli → docs → gui → tests`)

Phases 28–32 mirror v2.2 Phase 25 → 26 → 27 shape, expanded for Math Pac I's larger surface.

### What does NOT change

`Op` enum lives in the same place. `dispatch()` keeps single-match shape. `flush_entry_buf` is unchanged. `key_map::resolve` gets ZERO new bare-id arms (Math Pac I has no dedicated keys — XEQ-by-name modal only). SC-4 invariant trivially holds (all math logic lands in `hp41-core/src/ops/math1/`, zero `op_*`/`flush_entry_*`/`format_hpnum` symbols added to `hp41-gui`).

---

## Top Pitfalls to Mitigate (PITFALLS.md, 7 critical)

Note: function names below are PITFALLS.md's; translate to Math Pac I equivalents per the scope correction (e.g. `INTEG` → `INTG`, `PROOT` → `POLY`/`ROOTS`, `CARG` → `LNZ`/`Z↑W` branch-cut concerns instead).

| # | Pitfall | Phase | Prevention |
|---|---------|-------|------------|
| 1 | **Function-name collision** between Math Pac I and built-in ops (`Σ+`, `MEAN`, `SDEV`, `MOD`, `FACT`, `ABS`). A v2.2 program that calls `XEQ "Σ+"` must not silently flip semantics after upgrade. | Phase 28 framework | Built-in mnemonics WIN against XROM in the resolver chain (xrom_resolve fires LAST). Disambiguated mnemonics (`MAT-Σ+`, `C-ABS`) for any true collision. CI gate `tests/xrom_shadowing.rs`. |
| 2 | **`INTG` convergence depends on `DisplayMode`** — hardware ties convergence threshold to `FIX n` / `SCI n`. Hard-coding `1e-10` diverges from documented examples in a non-auditable way. | Phase 28 research-prep + Phase 30 impl | Encode `threshold = 10^(-decimals - 1)` in a single helper; unit-test threshold independently. Cap subdivisions at 2^15. Cite OM page-and-example per test (D-27.1 pattern). |
| 3 | **`SOLVE` three input regimes have distinct hardware conventions** — no real root → local min of `|f|` OR `DATA ERROR` per mode; multiple roots → seed-rounding sensitivity; discontinuity → must surface step-limit error, not silent infinite loop. | Phase 30 | Encode three named branches with OM citations. 100-iteration cap. Reuse `MAX_STEPS = 1_000_000` budget per `run_program`. |
| 4 | **User-callback re-entrancy corrupts solver state** — user fn can clobber INTG's R00–R07 scratch, can nest INTG/SOLVE, can change flags, can `STOP`/`GTO`-out. v2.2's `is_running: bool` was never designed for nested run_program. | Phase 28 framework decision; surfaces Phase 30 | Pick one of three policies in Phase 28: strict (`InvalidOp` on nest), counter-based (`run_depth: u8` with cap 2), or caller-side save/restore of scratch around each user-fn call. ARCHITECTURE.md recommends rejecting nested-INTG/SOLVE at op entry to match OM. Five regression tests in `tests/math1_user_callback.rs`. |
| 5 | **`POLY`/`ROOTS` convergence failure on ill-conditioned polynomials** — 5-fold root at 1 in `(x-1)^5` produces a cluster of ~10⁻³ imaginary parts; hardware returns the cluster (multiplicity-as-cluster convention). Tests asserting exactness will fail when the implementation is correct. | Phase 30 | Document multiplicity-as-cluster in `docs/hp41-math1-divergences.md`. Treat `|imag| > 10⁹` during real-polynomial PROOT as did-not-converge → DATA ERROR. Owner's Manual Section 7 worked examples are the test source-of-truth. |
| 6 | **Complex `LNZ` / `Z↑W` branch cuts + (0,0) handling** — `atan2(0,0)` must return 0 in DEG/RAD mode (NOT NaN, NOT `DATA ERROR`). `Z↑W` with zero divisor must `DATA ERROR`. `rust_decimal` has no `atan2` so the f64 bridge must explicitly handle (0,0). | Phase 28 (complex ops are first Math-1 implementation after framework) | `complex_atan2(im, re)` first arm handles (0,0)→0. Branch on zero-divisor BEFORE division. Edge-case suite in `tests/math1_complex_edge_cases.rs`. |
| 7 | **`MATRIX` `INV` singularity detection EPSILON is hardware-specific** — community-cited as `5e-10` but OM quotes `1e-9` for `DET` zero-test. Wrong value means INV disagrees with OM on every near-singular example. | Phase 28 (matrix ops) | Source EPSILON from OM directly (Phase 28 research-prep MUST transcribe). Encode as `pub const EPSILON: HpNum` so it surfaces in code review. Three accuracy cases: `inv_singular`, `inv_near_singular`, `inv_back_sub_overflow`. |

**Cross-cutting** (moderate / minor severity, all in PITFALLS.md):

- Phase 28: enum-bloat vs `XromCall(u16)` dispatch (ADR locked; criterion gate `dispatch_overhead < 200 ns/op`)
- Phase 30: cross-platform numerical drift (x86 vs ARM f64 last-bit) → relative tolerance `1e-7` documented as the Math Pac I floor
- Phase 31 (GUI integration): GUI long-INTG freeze; cancellation channel via `cancel_requested: Arc<AtomicBool>` + `request_cancel` Tauri command
- Phase 31: save-file mid-modal/mid-solver state (`integ_state`/`solve_state`/`modal_program` MUST be `#[serde(default, skip)]` from first commit)
- Phase 31: Math Pac I discoverability through XEQ-by-name modal (40 mnemonics — `CATALOG 2` extension recommended)
- Cross-cutting: JSON-canonical parity via dedicated test files; per-Op test count ≥ 5 to avoid mid-milestone coverage drop below 95 %; Free42 GPL contamination guard (consult-not-copy, with per-file header comment + audit script)

---

## Roadmap Implications

**Recommended phase count: 5 (Phases 28–32).** Phase numbering continues from v2.2's Phase 27. The shape mirrors v2.2's `core → cli → docs → gui → tests` build sequence.

### Suggested phase structure

| Phase | Name | Delivers | Research flag |
|-------|------|----------|---------------|
| **28** | XROM Framework + Math Pac I Core Ops (hp41-core) | XROM registry + `xrom_modules` field + 6 new CalcState fields. Plan-by-plan: 28-01 framework, 28-02 hyperbolics (proof-of-pattern), 28-03 complex stack + arithmetic, 28-04 complex functions (MAGZ/CINV/E↑Z/LNZ/SINZ/COSZ/TANZ/Z↑N/Z↑1/N), 28-05 POLY/ROOTS, 28-06 MATRIX workflow + DET/INV/SIMEQ, 28-07 INTG (with user-callback infrastructure), 28-08 SOLVE, 28-09 DIFEQ, 28-10 FOUR / triangles / TRANS. | **YES** — research-prep required BEFORE phase entry: OM transcription for `INV` EPSILON, `INTG` adaptive threshold formula, `SOLVE` three-regime behavior, `POLY` complex-pair output format, user-callback re-entrancy policy decision. |
| **29** | CLI Integration | `xeq_by_name_local_resolve` xrom fallback; help_data.rs second OnceLock; prgm_display.rs ~40 new arms; KEY_REF_TABLE Math 1 entries; modal-prompt routing for `MATRIX`/`SOLVE`/`POLY`/`INTG`/`DIFEQ`/`FOUR`/`TRANS` workflows. | NO — well-trodden ground from v2.2 Phase 25. |
| **30** | Documentation | `docs/hp41-math1-functions.json` populated; `scripts/docs-matrix/` two-input mode; `docs/hp41-math1-function-matrix.md` regenerated; README v3.0 soft-claim; `docs/hp41-math1-divergences.md` for multiplicity-as-cluster, INTG threshold tying, FACT extension policy, etc. | NO — pattern locked in v2.2 Phase 25. |
| **31** | GUI Integration | hp41-gui prgm_display.rs ~40 new arms; XEQ modal verification of `xeq_M+` round-trip; `?` overlay loads Math 1 JSON; CATALOG 2 implementation; cancellation channel (`request_cancel` Tauri command); Web Audio for any new TONE-style ops (none expected in Math Pac I). | **YES** — research-prep for cancellation channel design (lock release strategy during long INTG/SOLVE); GUI modal-prompt rendering pattern (how to surface `ORDER=?` / `A1,1=?` prompts visually). |
| **32** | Test Hardening | numerical_accuracy.rs extended from 566 → ~700+ cases per FEATURES.md; coverage gate held at ≥ 95 % (no atomic raise this milestone); E2E smoke extended with one Math Pac I keystroke flow (e.g. `XEQ "SINH" 1 → 1.1752`); Vitest CI-gated; MSRV unchanged. | NO — gates and patterns from v2.2 Phase 27 carry over. |

### Build order rationale

- **Framework + foundation first (Phase 28-01)** — nothing else compiles without the XROM registry, CalcState fields, and Op enum variants. This is the irrevocable decision phase: Op-strategy A, user-callback policy, INV EPSILON value, INTG threshold formula, JSON-pipeline two-file shape.
- **Hyperbolics second (Plan 28-02)** — the SIX-op family with one-shot UX that matches v2.2 patterns exactly. Validates the framework end-to-end through CLI + GUI for the lowest-cost win. Immediate user value: anyone wanting `SINH` gets it.
- **Complex stack third (Plans 28-03/04)** — the pac's marquee feature; biggest "wow" delivery. Builds on hyperbolics' framework validation. Plan 28-03 lands `ComplexStack { zeta, tau }` scaffolding + `C+`/`C-`/`C×`/`C÷`; 28-04 layers the 13 unary/binary complex functions.
- **POLY before MATRIX (Plans 28-05 before 28-06)** — POLY is self-contained modal-state-machine; MATRIX is the most complex single program. Learning curve on modal infrastructure starts gentler.
- **MATRIX before INTG/SOLVE (Plan 28-06 before 28-07/08)** — MATRIX is pure-data modal (no user-program callback); INTG/SOLVE add the user-callback layer on top of a known modal pattern. The Pitfall-4 re-entrancy decision MUST be locked before Plan 28-07 lands.
- **DIFEQ + FOUR + triangles + TRANS last (Plans 28-09/10)** — differentiators, not table stakes. Can ship without and still claim "feature-complete Math Pac I core" if scope pressure builds.
- **CLI / docs / GUI / tests as separate phases (29/30/31/32)** — same shape as v2.2 Phase 25 → 26 → 27. Each phase has crisp acceptance criteria; phase boundary is a release boundary.

### MVP fallback path (FEATURES.md §"MVP Recommendation")

If shipping a minimal v3.0-alpha to validate the architecture before completing everything:
- **MVP-1:** Framework + hyperbolics (6 ops) + complex arithmetic + 5 most-used complex functions (`MAGZ`, `CINV`, `E↑Z`, `LNZ`, `SINZ`). ~3 plans of Phase 28.
- **MVP-2:** Add POLY/ROOTS for quadratic only (degree 2). +1 plan.
- **MVP-3:** Add MATRIX workflow for N ≤ 4 (smaller working set; same algorithm). +1 plan.

This gets a usable Math Pac v3.0-alpha out in ~5 Phase-28 plans; remaining content lands iteratively without architecture changes.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| **Stack** | HIGH | All crate metadata verified on crates.io 2026-05-16. `Float`-bound rejection for `num-complex`/`nalgebra` confirmed via docs.rs trait inspection. Zero new runtime deps is unambiguous. |
| **Features** | HIGH | Primary source = HP-41C Math Pac Owner's Manual 00041-90034 (1979) + QRC 00041-90065. Full inventory transcribed from manual's Table of Contents and Appendix B. Cross-referenced against hpmuseum.org and HP-41 archive. The 10 programs / ~55 entry points are not in dispute. |
| **Architecture** | HIGH | For pattern decisions (Op-strategy A, six CalcState fields, separate JSON file, run_loop re-entrancy), MEDIUM for example Op-variant names (drafted under the older scope assumption — names need re-mapping to Math Pac I conventions during Phase 28 planning). The core conclusions are scope-independent. |
| **Pitfalls** | MEDIUM | OM-cited pitfalls (INTG threshold, SOLVE three regimes, INV EPSILON, POLY clustering) are HIGH confidence; community-cited pitfalls (Free42 GPL contamination, cross-platform f64 drift, GUI freeze severity) are MEDIUM. Function names in PITFALLS.md reference the older scope; pitfall CATEGORIES translate cleanly. |

**Overall confidence: HIGH on scope and HIGH on architectural integration. MEDIUM on specific numerical-method ground-truth details that require Phase 28 research-prep before implementation begins.**

---

## Open Questions (consolidated from all four research files)

1. **XROM numbering** — Math Pac I is XROM 7 on real hardware (confirmed). Use `Op::Xrom(7, fn)` programmatic dispatch syntax? Bit position in `state.xrom_modules` — bit 0 (semantic: "Math 1 is the first XROM module we emulate") or bit 7 (semantic: "matches the real XROM ID")? **Decide Plan 28-01.** Recommend bit 0 (decoupled from hardware ID; the registry maps bit → XROM ID).
2. **Complex stack location** — three options per FEATURES.md §"Open Questions" Q2: (a) dedicated `ComplexStack` struct on CalcState; (b) overlaid on T/Z/Y/X (ζ=Y+iX, τ=T+iZ — matches OM "two-element complex stack"); (c) in dedicated registers R02/R03 (ζ) / R04/R05 (τ — matches OM's actual R00–R04 block). **Decide Plan 28-03.** Recommend option (b) or (c) for fidelity.
3. **`FACT` extension** — v2.2 has integer-only `FACT(0..69)`. Math Pac I MAY extend via gamma function (`FACT(2.5) = Γ(3.5)`), or MAY add a separate `GAMMA` and leave FACT alone. **Decide Phase 28 research-prep** before any commit lands. Recommend leaving FACT alone + adding GAMMA only if OM names it.
4. **User-callback re-entrancy policy** — PITFALLS Pitfall 4: strict-reject, run_depth counter, or save-restore scratch? **Decide Phase 28 research-prep**, before Plan 28-07 (INTG). ARCHITECTURE.md's recommendation is strict-reject nested INTG/SOLVE at op entry (matches OM); the within-INTG `STO` clobbering of scratch needs the save-restore policy regardless.
5. **`INV` singularity EPSILON** — PITFALLS Pitfall 7: community-cited `5e-10` vs OM-quoted `1e-9`. **MUST transcribe OM** before Plan 28-06 lands.
6. **`INTG` convergence threshold formula** — PITFALLS Pitfall 2: tied to `DisplayMode`. **MUST transcribe OM** before Plan 28-07 lands. Recommend `threshold = 10^(-decimals - 1)` as a single helper.
7. **Modal-program persistence semantics** — `modal_program: Option<ModalProgram>` MUST be `#[serde(default, skip)]` from Plan 28-06 first commit (PITFALLS Pitfall 12). But: should the user be ABLE to suspend a `MATRIX` workflow mid-input via save/load? Recommend NO (matches real Math Pac I behavior — modal state is volatile).
8. **GUI prompt rendering** — Math Pac I prompts (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`) appear in the ALPHA display register. How does this surface in the v2.0 GUI's 12-char LCD + print buffer? Recommend: use `state.print_buffer` for prompts (existing channel; appears in print panel below LCD). **Decide Phase 31 research-prep.**
9. **GUI cancellation channel** — PITFALLS Pitfall 11: long INTG holds Mutex, freezes UI. Recommend `request_cancel` Tauri command + `state.cancel_requested: Arc<AtomicBool>` + every-64-samples lock release in `op_integ`. **Decide Plan 31-02.**
10. **JSON file shape** — separate `hp41-math1-functions.json` (RECOMMENDED per ARCHITECTURE.md), OR add `module` field to combined `hp41cv-functions.json`? Recommend separate file (zero migration churn on 130 existing v2.2 entries; cleaner test surfaces). **Decide Plan 30-01.**
11. **`docs-matrix` output** — two separate matrices OR one combined matrix with a Module column? **Decide Plan 30-02.** Either works; preference is two matrices for readability.

---

## Sources

**HIGH confidence (primary):**
- HP-41C Math Pac Owner's Manual (00041-90034, 1979) — [hpcalc.org PDF](https://literature.hpcalc.org/community/hp41-pac-math-en.pdf) — authoritative function and algorithm spec
- HP-41C Math Pac I Quick Reference Card (00041-90065, 1979) — [hpcalc.org PDF](https://literature.hpcalc.org/community/hp41-pac-math-qrc-en.pdf) — confirms QRC entries
- HP-41C/CV Owner's Handbook (1980, 00041-90325) — built-in op semantics
- crates.io API + GitHub manifest inspection (2026-05-16) for `rust_decimal`, `num-complex`, `nalgebra`, `faer`, `ndarray`, `roots`, `peroxide`, `gauss-quad`, `argmin`, `approx`
- Direct codebase inspection: `hp41-core/src/state.rs`, `ops/mod.rs`, `ops/program.rs`, `num.rs`, `hp41-cli/src/keys.rs`, `help_data.rs`, `hp41-gui/src-tauri/src/key_map.rs`, `scripts/docs-matrix/src/main.rs`, `docs/hp41cv-functions.json`
- `.planning/PROJECT.md`, `CLAUDE.md` (full file — SC-4, D-25.6, D-22.15, JSON pipeline, exhaustive-match invariants)

**MEDIUM confidence (community / cross-reference):**
- Museum of HP Calculators HP-41 software library — [hpmuseum.org/software/soft41.htm](https://www.hpmuseum.org/software/soft41.htm), [hpmuseum.org/software/xroms.htm](https://www.hpmuseum.org/software/xroms.htm)
- HP-41 Matrix Operations community page — [hpmuseum.org/software/41/41matrix.htm](https://www.hpmuseum.org/software/41/41matrix.htm) — confirms 14×14 limit + Gaussian elimination algorithm
- HP-41 Archive — [hp41.org](http://www.hp41.org/)
- HPCalc community PDF index — [literature.hpcalc.org](https://literature.hpcalc.org/)
- Mike Sebastian's HP-41 forensics pages — cross-platform numerical drift data
- MoHPC forum discussions — community reverse-engineering of Math Pac internals
- Carnahan, Luther & Wilkes, _Applied Numerical Methods_ (1969) — manual's cited matrix algorithm reference
- Forsythe, Malcolm & Moler, _Computer Methods for Mathematical Computations_ (1972) — manual's secondary reference

**LOW confidence (consult-only — DO NOT COPY):**
- Free42 source code (Thomas Okken, GPL) — [thomasokken.com/free42/](https://thomasokken.com/free42/) — useful as a sanity-check oracle for INTG/SOLVE outputs ONLY; every Math Pac I algorithm in v3.0 must be independently re-derived from OM per PITFALLS Pitfall 19 + project's permissive-licensing position

**Cross-referenced project context:**
- `.planning/milestones/v2.0-research/SUMMARY.md` — structural template for this document
- v2.2 settled invariants: `CLAUDE.md` (esp. f-prefix one-shot model, hybrid `PendingInput` struct-variants, JSON-canonical pipeline D-25.16, four-arm exhaustive match D-25.13, CLI ↔ GUI parity D-25.6, save-file backward compat via `#[serde(default)]`)
- v3.0 scope lock: `.planning/PROJECT.md` (Math Pac I only; Stat 1 → v3.1; Time → v3.2; Advantage → v3.3; ROM-image redistribution permanently excluded)

---

*End of SUMMARY.md — orchestrator may now proceed to REQUIREMENTS.md generation.*
