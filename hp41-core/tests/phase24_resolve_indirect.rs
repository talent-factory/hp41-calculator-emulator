#![allow(clippy::unwrap_used)]
//! Phase 24 cross-cutting integration tests for resolve_indirect / resolve_indirect_decimal.
//!
//! 24-01 scope: scaffolding — the primary unit-test coverage for the helpers lives
//! inline in `hp41-core/src/ops/indirect.rs::tests`, and the GtoInd/XeqInd refactor
//! regression sentinels live in `hp41-core/tests/phase22_program_control.rs` (D-24.5).
//!
//! 24-02 scope: this file gains the per-variant `*_ind` integration tests OR a sibling
//! `phase24_ind_variants.rs` is created — see plan 24-02.

#[test]
fn resolve_indirect_is_reachable_from_integration_target() {
    // Compiles → public symbol path resolves from outside the crate.
    // Defends against Pitfall 5 ("forgetting `pub mod indirect;` in ops/mod.rs").
    let _ = hp41_core::ops::indirect::resolve_indirect;
}
