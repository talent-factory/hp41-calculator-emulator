// Phase 31 Plan 05 — GUI-07 stub-arm policy regression test (file-text scan).
//
// Context: `key_map::resolve` is declared as a PRIVATE module in lib.rs:
//   `mod key_map;`   (NOT `pub mod key_map;`)
// Integration tests therefore CANNOT call `hp41_gui_lib::key_map::resolve(...)`.
// This test uses include_str!("../src/key_map.rs") to load the file text at
// compile time — a pure file-text scan with no Rust API access.
//
// WHAT IT LOCKS:
//   The format! template `'{key_id}' is planned for a future phase` appears
//   EXACTLY BASELINE_N = 1 time in key_map.rs. This single template covers
//   ~20 ids in one `| "asn" | "catalog" | ...` match arm.
//
// WHY EXACT EQUALITY (assert_eq!, NOT >=):
//   - Count BELOW baseline: regression — someone removed or split the arm.
//   - Count ABOVE baseline: regression — someone added a new stub arm without
//     an explicit Phase 31+ review. Math Pac I uses XEQ-by-name only (D-28.6
//     / GUI-07); no new stub ids are expected in v3.0.
//
// HOW TO UPDATE BASELINE_N:
//   Increase ONLY after explicit Phase 31+ review approving a new stub arm.
//   Decrease ONLY after a planned stub-retirement (e.g., converting a stub to
//   a real `Ok(Op::*)` arm in a new phase).
//   NEVER auto-update; the exact lock is the guard.
//
// Per key_map.rs lib.rs: `mod key_map;` (private) — NO `pub mod` widening
// needed because this test accesses the file as text, not as Rust API.

#![allow(clippy::unwrap_used)]

/// key_map.rs source text baked in at compile time.
/// Tests reference this constant rather than re-invoking include_str! each time.
const KEY_MAP_SRC: &str = include_str!("../src/key_map.rs");

/// Baseline locked v3.0 — exactly 1 occurrence of the format! template.
/// The single occurrence covers ~20 id literals in one `| "asn" | ... | "tone"` arm.
///
/// Increase ONLY after explicit Phase 31+ review approving a new stub arm.
/// Decrease is REGRESSION — Math Pac I uses XEQ-by-name only (D-28.6 + GUI-07).
const BASELINE_N: usize = 1;

/// V2.1 baseline id literals that must remain present in key_map.rs.
/// These are the ids that fall through to the stub-error arm (frontend intercepts
/// them before dispatch_op; the arm is defense-in-depth per D-07).
///
/// Catches: shrinkage (an id was removed from the match arm without Phase 31 review).
const BASELINE_IDS: &[&str] = &[
    "\"asn\"",
    "\"catalog\"",
    "\"view\"",
    "\"xeq_prompt\"",
    "\"gto_prompt\"",
    "\"lbl_prompt\"",
    "\"sto_prompt\"",
    "\"rcl_prompt\"",
    "\"isg_prompt\"",
    "\"sf_prompt\"",
    "\"cf_prompt\"",
    "\"fs_prompt\"",
    "\"fix_prompt\"",
    "\"sci_prompt\"",
    "\"eng_prompt\"",
    "\"x_eq_y_prompt\"",
    "\"x_le_y_prompt\"",
    "\"x_gt_y_prompt\"",
    "\"x_eq_0_prompt\"",
    "\"tone\"",
];

/// Catches: silent growth OR shrinkage of the format!-template count.
///
/// If a new stub arm is added (count rises above baseline), this test fails —
/// forcing explicit Phase 31+ review before the change can merge.
///
/// If an existing stub arm is accidentally removed (count falls below baseline),
/// this test also fails — catching regressions in defense-in-depth error surfacing.
#[test]
fn stub_error_message_count_locked_to_v21_baseline() {
    let count = KEY_MAP_SRC
        .matches("is planned for a future phase")
        .count();
    assert_eq!(
        count, BASELINE_N,
        "key_map.rs: found {} occurrences of 'is planned for a future phase', expected {} (BASELINE_N). \
        Either a stub arm was added (count > BASELINE_N: requires explicit Phase 31+ review) \
        or removed (count < BASELINE_N: D-07 defense-in-depth regression). \
        Update BASELINE_N only after explicit review.",
        count, BASELINE_N,
    );
}

/// Catches: individual id removal from the stub-error arm.
///
/// Complements `stub_error_message_count_locked_to_v21_baseline` by asserting
/// each of the v2.1 baseline id literals appears in the file text. The count
/// test catches total changes; this test pinpoints WHICH id was removed.
#[test]
fn key_map_file_contains_v21_baseline_ids() {
    for id_literal in BASELINE_IDS {
        assert!(
            KEY_MAP_SRC.contains(id_literal),
            "key_map.rs is missing the v2.1 baseline stub-error id literal: {}. \
            Check if it was accidentally removed from the stub-error arm.",
            id_literal,
        );
    }
}

/// Sanity check: the file text is non-empty (include_str! must have resolved).
#[test]
fn key_map_src_is_nonempty() {
    assert!(
        !KEY_MAP_SRC.is_empty(),
        "include_str!(\"../src/key_map.rs\") returned empty — path resolution failure?"
    );
    // Additionally verify the file contains the GuiError type we depend on.
    assert!(
        KEY_MAP_SRC.contains("GuiError"),
        "key_map.rs does not reference GuiError — unexpected structural change"
    );
}
