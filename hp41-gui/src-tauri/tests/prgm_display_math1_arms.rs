//! Phase 28 Math Pac I `op_display_name` arm regression test.
//!
//! Verifies that every Math Pac I `Op` variant shipped in Phase 28 (plans 28-02..28-10)
//! has a corresponding `Op::<Id>` arm in `hp41-gui/src-tauri/src/prgm_display.rs`,
//! and that NO `_ =>` wildcard catch-all exists in `op_display_name`.
//!
//! # Design rationale
//!
//! `prgm_display` is declared as a PRIVATE `mod prgm_display;` (not `pub mod`) in
//! `hp41-gui/src-tauri/src/lib.rs`. The function `op_display_name` is therefore NOT
//! crate-public and cannot be called from integration test code without widening the
//! visibility — which would violate Plan 31-01's "zero source edits in
//! hp41-gui/src-tauri/src/" pledge.
//!
//! This test is therefore a FILE-TEXT SCAN of `prgm_display.rs`. It does NOT call
//! any function from `hp41-gui/src-tauri/src/prgm_display.rs`.
//! It does NOT import `hp41_gui_lib::prgm_display::*` (that would fail to compile).
//!
//! # What this test catches
//!
//! - Missing `op_display_name` arm for a Math Pac I Op variant (Catches: silent silently
//!   unreachable code warning + wrong PRGM listing for the variant)
//! - `_ =>` wildcard catch-all in `op_display_name` (Catches: future Op variants silently
//!   falling through to an incorrect display string; violates the 4-exhaustive-match invariant)

#![allow(clippy::unwrap_used)]

/// The `prgm_display.rs` source text loaded at compile time.
/// Path is relative to this test file's location (`hp41-gui/src-tauri/tests/`),
/// so `../src/prgm_display.rs` correctly resolves to `hp41-gui/src-tauri/src/prgm_display.rs`.
const PRGM_DISPLAY_SRC: &str = include_str!("../src/prgm_display.rs");

/// Hard-coded list of Math Pac I Op variant identifier strings whose arms must appear
/// in `prgm_display.rs` as `Op::<Id>` substrings.
///
/// Source: MATH_1.ops static slice in `hp41-core/src/ops/math1/xrom.rs`
/// (plans 28-02..28-10). Duplicate entries (where multiple mnemonics share one Op
/// variant, e.g. ASCII + Unicode aliases for CTimes/CDiv/ZpowN/etc.) are collapsed
/// to the unique Op variant identifier.
///
/// Count: 44 unique Math Pac I Op variants shipped in Phase 28.
const MATH1_VARIANT_IDS: &[&str] = &[
    // ── Plan 28-02: Hyperbolics (6 variants) ──────────────────────────────────
    "Sinh", "Cosh", "Tanh", "Asinh", "Acosh", "Atanh",
    // ── Plan 28-03: Complex Stack Arithmetic (5 variants) ─────────────────────
    // CTimes and CDiv each have Unicode+ASCII aliases in MATH_1.ops, but only
    // one Op variant each. Real is the REAL deactivate-complex-mode entry point.
    "CPlus", "CMinus", "CTimes", "CDiv", "Real",
    // ── Plan 28-04: Complex Functions (12 variants) ───────────────────────────
    // ZpowN, Zpow1N, ExpZ, ApowZ, ZpowW each have Unicode+ASCII aliases in
    // MATH_1.ops, but only one Op variant each.
    "Magz", "Cinv", "ZpowN", "Zpow1N", "ExpZ", "LnZ", "SinZ", "CosZ", "TanZ",
    "ApowZ", "LogZ", "ZpowW",
    // ── Plan 28-05: POLY / ROOTS (2 variants) ─────────────────────────────────
    "PolyWorkflow", "Roots",
    // ── Plan 28-06: MATRIX (8 variants) ───────────────────────────────────────
    "MatrixWorkflow", "MatSize", "MatVmat", "MatEdit", "MatDet", "MatInv",
    "MatSimeq", "MatVcol",
    // ── Plan 28-07: INTG (1 variant) ──────────────────────────────────────────
    "Integ",
    // ── Plan 28-08: SOLVE / SOL (2 variants) ──────────────────────────────────
    "Solve", "Sol",
    // ── Plan 28-09: DIFEQ (1 variant) ─────────────────────────────────────────
    "Difeq",
    // ── Plan 28-10: FOUR / Triangle Solvers / TRANS (8 variants) ─────────────
    "Four", "TriSss", "TriAsa", "TriSaa", "TriSas", "TriSsa", "Trans2d", "Trans3d",
];

/// Catches: missing `op_display_name` arm for a Phase 28 Math Pac I Op variant,
///          causing wrong PRGM listing output for that variant.
///
/// Catches: `_ =>` wildcard catch-all in `op_display_name`, which would hide
///          future missing arms at compile time (violates 4-exhaustive-match invariant).
#[test]
fn every_math1_op_appears_in_prgm_display() {
    // Sanity: the file must be non-empty (guards against include_str! resolving
    // to the wrong path or a blank file, which would produce false-positive passes).
    assert!(
        !PRGM_DISPLAY_SRC.trim().is_empty(),
        "prgm_display.rs loaded via include_str! is empty — check the relative path \
         '../src/prgm_display.rs' from hp41-gui/src-tauri/tests/"
    );

    // Assert each Phase 28 Math Pac I variant identifier appears as `Op::<Id>` in
    // the prgm_display.rs source text.
    let mut missing: Vec<&str> = Vec::new();
    for id in MATH1_VARIANT_IDS {
        let needle = format!("Op::{id}");
        if !PRGM_DISPLAY_SRC.contains(needle.as_str()) {
            missing.push(id);
        }
    }

    assert!(
        missing.is_empty(),
        "Missing op_display_name arms in hp41-gui/src-tauri/src/prgm_display.rs:\n{}\n\n\
         Each of the above Op variants was shipped in Phase 28 plans 28-02..28-10 \
         but does NOT appear as `Op::<Id>` in prgm_display.rs. Add the missing arm(s) \
         to restore the 4-exhaustive-match invariant.",
        missing
            .iter()
            .map(|id| format!("  - missing arm: Op::{id}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Assert NO `_ =>` wildcard catch-all exists in op_display_name.
    //
    // Strategy: find the `fn op_display_name` declaration line, then scan from
    // that point to the end of the file for either `_ =>` or `_=>` substrings.
    // This is a conservative check (it scans past the function boundary), but
    // `op_display_name` is the only `match op {` in the file; a wildcard elsewhere
    // would also be a problem worth catching.
    //
    // We accept slight over-approximation: if a future comment or string literal
    // contains `_ =>`, the test would fail. That's acceptable — the test owner
    // updates the check to narrow the window.
    let fn_decl = "fn op_display_name";
    let fn_start = PRGM_DISPLAY_SRC.find(fn_decl).unwrap_or_else(|| {
        panic!(
            "Could not locate `{fn_decl}` in prgm_display.rs. \
             Has the function been renamed? Update this test if so."
        )
    });

    let body_text = &PRGM_DISPLAY_SRC[fn_start..];

    // Check for `_ =>` (with space) — the common rustfmt form
    assert!(
        !body_text.contains("_ =>"),
        "Found a `_ =>` wildcard catch-all in or after `op_display_name` in prgm_display.rs.\n\
         The 4-exhaustive-match invariant requires every Op variant to have an explicit arm.\n\
         A catch-all silently hides future missing arms. Remove the wildcard and add \
         explicit arms for each unhandled Op variant."
    );

    // Check for `_=>` (without space) — alternative rustfmt-off form
    assert!(
        !body_text.contains("_=>"),
        "Found a `_=>` wildcard catch-all (no space) in or after `op_display_name` in \
         prgm_display.rs.\n\
         The 4-exhaustive-match invariant requires every Op variant to have an explicit arm.\n\
         A catch-all silently hides future missing arms. Remove the wildcard and add \
         explicit arms for each unhandled Op variant."
    );
}
