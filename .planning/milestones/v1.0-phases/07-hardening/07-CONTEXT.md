# Phase 7: Hardening - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Meet all non-functional quality requirements for the v1.0 CLI release: cross-platform CI matrix, cold-start performance, key-dispatch latency, test coverage gate, numerical accuracy suite, and zero-panic guarantee in `hp41-core`.

**Deliverables:**
- `.github/workflows/ci.yml` — GitHub Actions matrix (Windows 10+, macOS 12+, Ubuntu 22.04+) + separate coverage job
- `hp41-core/benches/dispatch_bench.rs` — criterion.rs benchmark for dispatch() latency
- `hp41-core/src/tests/accuracy_suite.rs` (or `tests/numerical_accuracy.rs`) — 500-case numerical accuracy test module
- Fixes to `hp41-core/src/ops/math.rs` — eliminate `unwrap()` in production code
- `hp41-core/src/lib.rs` — add `#![deny(clippy::unwrap_used)]`
- Additional unit tests for `hp41-core/src/ops/program.rs` (currently 59% coverage)
- `Justfile` — add `bench-startup` recipe for hyperfine cold-start measurement
- Phase 5 Plan 11 verification (gcd_ops and stack_stats_ops correctness)

**Not in scope:** New calculator features, GUI (Tauri), any FR-* functional requirements, extended module emulation.

</domain>

<decisions>
## Implementation Decisions

### CI Matrix (QUAL-01)

- **D-01:** CI provider is **GitHub Actions**. No CI exists today — `.github/workflows/ci.yml` must be created from scratch.
- **D-02:** Matrix runs **`cargo build --release` + `cargo test`** on all 3 platforms (ubuntu-latest, windows-latest, macos-latest). Coverage and clippy are NOT in the matrix — they add install overhead and produce identical results across platforms.
- **D-03:** CI triggers on **push to `main`/`develop`** and on **pull requests**. No per-branch trigger; no manual-only workflow.
- **D-04:** Coverage gate (`cargo-llvm-cov --fail-under-lines 80 -p hp41-core`) runs as a **separate job on ubuntu-latest** alongside the matrix. Keeps the matrix fast; coverage is platform-independent.

### Numerical Accuracy Suite (QUAL-05, QUAL-06)

- **D-05:** Reference values are **document-derived** from the HP-41 Owner's Handbook formulas and known mathematical constants. No physical hardware required. This is the established approach used by Free42 and similar HP RPN emulators.
- **D-06:** Suite is a **Rust test module in `hp41-core`** — `tests/numerical_accuracy.rs` or `src/tests/accuracy_suite.rs`. Cases are Rust structs with `input`, `expected`, and `tolerance` fields. Runs with `cargo test`. No external data files.
- **D-07:** **Weighted distribution** across 500 cases:
  - Arithmetic (`+ − × ÷`, `1/x`, `√x`, `x²`, `Y^X`): 100 cases
  - Trig with all 3 angle modes (DEG/RAD/GRAD): 150 cases
  - Logs and exponentials (`LN`, `LOG`, `e^x`, `10^x`): 100 cases
  - ISG/DSE edge cases (counter field extraction, boundary values): 50 cases
  - Transcendental accumulation error: 50 cases
  - HMS conversions (round-trip, edge cases, negative values): 30 cases
  - Statistics (Σ+, MEAN, SDEV, L.R., CORR): 20 cases
- **D-08:** 98% gate is **count-based**: a single `#[test]` function asserts `passes >= 490`. Failing cases are printed as diagnostic output. This matches the ROADMAP success criteria wording exactly.

### Performance Measurement (QUAL-02, QUAL-03)

- **D-09:** Cold-start (`≤ 0.5s`) is verified via **`hyperfine` + `just bench-startup` recipe** that runs the release binary. This is a **documented manual pre-release step** (not a CI gate), because CI VM performance varies too much for absolute timing gates. The recipe is: `hyperfine --runs 10 ./target/release/hp41-cli`.
- **D-10:** Key latency (`≤ 50ms median`) is verified by **criterion.rs benchmarks in `hp41-core/benches/dispatch_bench.rs`**. The bench calls `dispatch()` 1000x on a pre-built `CalcState` and reports statistical median. This captures the compute path (the variable part); terminal I/O latency is excluded as untestable in headless CI.
- **D-11:** Benchmarks are **advisory only** — they run and produce output but do not gate CI builds. CI VM contention would cause false failures on absolute timing thresholds.
- **D-12:** Add `criterion` as a `[dev-dependencies]` entry in `hp41-core/Cargo.toml`. Add a `bench` recipe to the Justfile: `cargo bench -p hp41-core`.

### Panic Audit (QUAL-04, zero-panic guarantee)

- **D-13:** **Strict interpretation** of zero panics: fix ALL `unwrap()` calls in non-test `hp41-core` production code. The two known instances are in `hp41-core/src/ops/math.rs` (`pi_over_180()` and `pi_over_200()` — `Decimal::from_str(...).unwrap()`). Convert these to use `OnceLock<HpNum>` or a `dec!()` macro if available, or initialize via `Decimal::from_str(...).expect("unreachable: literal is valid")` documented as a one-time-init-that-never-fails if OnceLock is too heavy.
- **D-14:** Add `#![deny(clippy::unwrap_used)]` at the **`hp41-core/src/lib.rs` crate root** to enforce the guarantee structurally. Test modules may use `#[allow(clippy::unwrap_used)]` per-file.
- **D-15:** **Targeted tests** for `hp41-core/src/ops/program.rs` coverage gaps (currently 59% line coverage). Read the uncovered lines, identify testable paths (error conditions, program counter edge cases, nested XEQ depth limit), write focused tests. Goal: bring `ops/program.rs` to ≥ 80%.
- **D-16:** Phase 7's **first plan verifies Phase 5 Plan 11** (gcd_ops and stack_stats_ops fixes). Run the behavioral tests from 05-11 and confirm they pass. If not, fix them as the opening step of Phase 7 before any other hardening work.

### Claude's Discretion

- The exact file path for the accuracy suite (`tests/numerical_accuracy.rs` vs `src/tests/accuracy_suite.rs`) — use whichever is consistent with the existing test structure in `hp41-core`.
- Exact tolerance epsilon for accuracy cases — use `≤ 1e-10` (10-digit rounding threshold consistent with HP-41 10-significant-digit display and `HpNum::rounded()`).
- Whether `just ci` needs a new recipe for the full CI workflow locally — if adding `criterion`, a `bench` recipe is needed; `ci` should remain `lint → test → coverage` only.
- GitHub Actions YAML specifics (cache config, Rust toolchain version pin, OS version strings).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Quality Requirements
- `.planning/ROADMAP.md` §Phase 7 — Success criteria for QUAL-01 through QUAL-06 (the 5 measurable gates)
- `.planning/REQUIREMENTS.md` §NFR-1 through §NFR-7 — Non-functional requirement text

### Core Code (panic audit and coverage targets)
- `hp41-core/src/ops/math.rs` — lines 25-28: two `unwrap()` calls on `Decimal::from_str` string literals that must be fixed
- `hp41-core/src/ops/program.rs` — 59% line coverage; uncovered paths are targets for new tests
- `hp41-core/src/lib.rs` — crate root where `#![deny(clippy::unwrap_used)]` is added

### CI and Build
- `Justfile` — sole task runner; `ci: lint test coverage` is the current gate; `bench` and `bench-startup` recipes must be added
- `hp41-core/Cargo.toml` — `[dev-dependencies]` where `criterion` is added
- `.planning/phases/01-foundation/01-CONTEXT.md` — ADR-001: HpNum = rust_decimal (10-digit rounding); all accuracy suite tolerances derive from this

### Prior Phase Context
- `.planning/phases/06-science-and-engineering/06-CONTEXT.md` — HMS, stats, key binding decisions that Phase 7 tests must cover
- `.planning/phases/05-persistence-and-ux/05-11-PLAN.md` — gcd_ops and stack_stats_ops fixes to verify in Phase 7 opening step
- `.planning/STATE.md` — current performance metrics table (all unmeasured except coverage)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hp41-core/src/state.rs` — `CalcState::default()` is used in all existing tests; also the entry point for dispatch benchmarks in criterion.rs
- `hp41-core/src/ops/mod.rs` → `dispatch()` function — the function to benchmark for key latency
- `hp41-core/src/tests.rs` — existing test module structure; accuracy suite follows the same pattern

### Established Patterns
- All existing tests use `CalcState::default()` + `dispatch()` + register/stack assertions — accuracy suite uses the same pattern with tolerance comparison
- `just ci` = `lint → test → coverage`; new recipes added alongside (not replacing) this chain
- `cargo test --workspace` already passes (371 tests, 0 failed) — no regressions expected

### Integration Points
- `.github/workflows/ci.yml` (new file) — matrix triggers on `push` and `pull_request`; separate `coverage` job on ubuntu-latest
- `hp41-core/benches/` (new directory) — criterion.rs benches live here; `Cargo.toml` needs `[[bench]]` entry
- `hp41-core/src/lib.rs` — `#![deny(clippy::unwrap_used)]` attribute added at crate root

### Coverage Baseline (as of Phase 6 completion)
- Overall workspace: 72.21% line
- `hp41-core` (all files): approximately 82.8% (gate: ≥80% PASSED)
- Weakest file: `ops/program.rs` at 59.07% — primary target for coverage improvement
- Strong files: `ops/stack_ops.rs` 100%, `ops/alpha.rs` 100%, `stack.rs` 100%, `num.rs` 100%

</code_context>

<specifics>
## Specific Ideas

- The 500-case accuracy suite should include the ROADMAP's canonical example: HMS 1.3045 → 1.5125 and round-trip back
- The `pi_over_180()` and `pi_over_200()` functions in `ops/math.rs` use `HpNum(raw_decimal)` (bypassing rounding) — this pattern must be preserved when eliminating the `unwrap()`; the fix is about the `unwrap()`, not the `HpNum(...)` wrapper
- ISG/DSE accuracy cases should include the canonical example from Phase 3: counter `1.00500` (current=1, final=5, step=1) iterates exactly 4 times before falling through

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 7-hardening*
*Context gathered: 2026-05-07*
