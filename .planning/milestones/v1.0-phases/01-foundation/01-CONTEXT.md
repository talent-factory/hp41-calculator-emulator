# Phase 1: Foundation - Context

**Gathered:** 2026-05-06
**Status:** Ready for planning
**Mode:** Auto-generated (infrastructure phase — discuss skipped)

<domain>
## Phase Boundary

A Cargo workspace exists with a Justfile covering all build/test/lint/run targets, a compiling `hp41-core` crate that models a correct 4-level HP-41 RPN stack with full stack-lift semantics, resolves the BCD vs f64 numeric representation, and returns typed errors with zero panics.

**Deliverables:**
- Cargo workspace root with `hp41-core` (lib) and `hp41-cli` (bin) crates
- `Justfile` with build/test/lint/run/ci recipes
- `Stack` struct: 4-level X/Y/Z/T with `lift_enabled: bool`
- LASTX register
- `CalcState` as single source of truth
- BCD/f64 decision committed to code with ADR comment
- `Result<T, HpError>` typed errors, zero panics

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion
All implementation choices are at Claude's discretion — pure infrastructure phase. Key decisions already locked in STATE.md:

- **BCD vs f64:** Evaluate `rust_decimal` vs custom BCD struct; commit decision to `state.rs` as ADR comment
- **Stack-lift flag:** `lift_enabled: bool` in `Stack`; every operation declares Enable/Disable/Neutral effect
- **CalcState:** Single `&mut CalcState` passed through all operations; no global mutable state
- **Error type:** `Result<T, HpError>` — no panics in hp41-core
- **Workspace structure:** `hp41-core/` (library, zero UI deps) + `hp41-cli/` (binary, thin adapter)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- None yet (greenfield project)

### Established Patterns
- Rust stable toolchain (MSRV 1.78+)
- Cargo workspace with enforced crate boundaries

### Integration Points
- `hp41-core` exposes pure Rust API consumed by `hp41-cli` (and later `hp41-gui`)
- `cargo check -p hp41-core` must pass with zero UI/CLI dependencies

</code_context>

<specifics>
## Specific Ideas

- Justfile recipes: build, test, lint, run, ci (at minimum)
- `just ci` must pass on macOS from day one
- The BCD decision is the most critical: lock it here, document reasoning in ADR comment
- ISG/DSE counter fields use string-splitting at decimal point (not floor()/fmod() on f64)

</specifics>

<deferred>
## Deferred Ideas

None — infrastructure phase.

</deferred>
