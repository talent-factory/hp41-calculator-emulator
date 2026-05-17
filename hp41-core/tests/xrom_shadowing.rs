// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 CI gate: asserts no Math Pac I mnemonic shadows an existing v2.2 built-in.
//!
//! This test file iterates `MATH_1.ops` (currently `&[]` — empty until Plans 28-02..28-10
//! populate it) and asserts that no entry collides with a `builtin_card_op` mnemonic.
//!
//! **Why this matters (C-28.4 / Pitfall 1):** the XROM resolver fires LAST in the
//! chain — after `builtin_card_op`, before `Err(InvalidOp)`. If a Math Pac I mnemonic
//! shadowed a builtin (e.g., if Math Pac I defined "WPRGM" or "X<>Y?"), the builtin
//! would silently win and the Math Pac I op would be unreachable via XEQ.
//! The shadowing test catches this at CI time so Plans 28-02..28-10 can't introduce
//! a shadow without failing this gate.
//!
//! **Current state:** MATH_1.ops is `&[]`, so this test is vacuously true at Plan 28-01.
//! Plans 28-02..28-10 grow MATH_1.ops and the gate becomes non-trivial.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::math1::xrom::MATH_1;

/// All mnemonic strings recognized by `builtin_card_op` in `hp41-core/src/ops/program.rs`.
///
/// This list must be kept in sync with `builtin_card_op`'s match arms.
/// If a new builtin is added to `builtin_card_op`, add it here to maintain the
/// shadowing gate's correctness.
///
/// Canonical set as of v2.2 (Plan 25-03):
/// - 4 Card Reader ops: WPRGM, RDPRGM, WDTA, RDTA
/// - 8 conditional tests (ASCII + Unicode spellings): X<>Y?, X≠Y?, X#Y?,
///   X<Y?, X>=Y?, X≥Y?, X#0?, X≠0?, X<0?, X>0?, X<=0?, X≤0?, X>=0?, X≥0?
const BUILTIN_CARD_OP_NAMES: &[&str] = &[
    // Card Reader ops
    "WPRGM",
    "RDPRGM",
    "WDTA",
    "RDTA",
    // Conditional tests (ASCII spellings)
    "X<>Y?",
    "X#Y?",
    "X<Y?",
    "X>=Y?",
    "X#0?",
    "X<0?",
    "X>0?",
    "X<=0?",
    "X>=0?",
    // Conditional tests (Unicode spellings)
    "X\u{2260}Y?",
    "X\u{2265}Y?",
    "X\u{2260}0?",
    "X\u{2264}0?",
    "X\u{2265}0?",
];

/// CI gate: no Math Pac I mnemonic may shadow a v2.2 built-in name.
///
/// Currently vacuous (MATH_1.ops is empty). Non-trivial once Plans 28-02..28-10
/// populate MATH_1.ops with hyperbolic, complex, and workflow op mnemonics.
///
/// Catches: Pitfall 1 — a Math Pac I mnemonic accidentally matching a v2.2 builtin,
/// making the XROM op permanently unreachable via XEQ.
#[test]
fn math1_names_do_not_shadow_builtins() {
    for (name, _op) in MATH_1.ops {
        assert!(
            !BUILTIN_CARD_OP_NAMES.contains(name),
            "Math Pac I mnemonic {name:?} shadows a builtin_card_op entry. \
             The XROM resolver fires LAST (C-28.4), so the builtin would silently \
             win and the Math Pac I op would be permanently unreachable via XEQ. \
             Rename the Math Pac I mnemonic to avoid the collision."
        );
    }
}

/// Smoke: MATH_1 const fields are present and correct.
/// Catches: const field regression during Plans 28-02..28-10 MATH_1.ops growth.
#[test]
fn math1_const_fields() {
    assert_eq!(
        MATH_1.id, 7,
        "MATH_1.id must be 7 (HP Math Pac I hardware module ID)"
    );
    assert_eq!(MATH_1.name, "MATH 1A", "MATH_1.name must be 'MATH 1A'");
}
