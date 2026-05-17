//! Wave-0 integration tests: Math Pac I rows surface in the right-panel
//! discoverability listing after migrating `key_ref_entries()` to the merged
//! `help_entries_all()` accessor (D-29.2 / CLI-04 / D-25.18).
//!
//! These tests verify SC-4 of the ROADMAP: the right-panel key-reference table
//! (produced by `keys::key_ref_entries()`) includes Math Pac I entries with
//! `key_path = "XEQ \"<NAME>\""` after the `help_entries()` → `help_entries_all()`
//! migration.

#![allow(clippy::unwrap_used)]

use hp41_cli::keys::key_ref_entries;

/// Asserts that `key_ref_entries()` contains a Math Pac I entry for SINH.
///
/// After migrating `key_ref_entries()` from `help_entries()` to
/// `help_entries_all()` (D-29.2 / CLI-04), the Math Pac I rows with
/// `key_path = "XEQ \"SINH\""` must appear in the right-panel discoverability
/// listing. This test is the acceptance criterion for SC-4 of the ROADMAP
/// (Math Pac I in right-panel) and D-25.18 (no parallel hand-curated table).
#[test]
fn key_ref_entries_includes_math1_sinh() {
    let entries = key_ref_entries();
    let found = entries
        .iter()
        .any(|(key_path, display)| key_path == "XEQ \"SINH\"" && display == "SINH");
    assert!(
        found,
        "key_ref_entries() must include (\"XEQ \\\"SINH\\\"\", \"SINH\") after \
         migration to help_entries_all() — D-29.2 / CLI-04 / SC-4 ROADMAP. \
         The right-panel discoverability listing is JSON-derived (D-25.18) and \
         must include all Math Pac I entries with non-null key_path."
    );
}

/// Asserts that `key_ref_entries()` contains a Math Pac I entry for MATRIX.
///
/// Same rationale as `key_ref_entries_includes_math1_sinh`. MATRIX is a
/// separate category (Math1 Matrix) confirming multi-category coverage of
/// the merged pool across the right-panel listing.
#[test]
fn key_ref_entries_includes_math1_matrix() {
    let entries = key_ref_entries();
    let found = entries
        .iter()
        .any(|(key_path, display)| key_path == "XEQ \"MATRIX\"" && display == "MATRIX");
    assert!(
        found,
        "key_ref_entries() must include (\"XEQ \\\"MATRIX\\\"\", \"MATRIX\") after \
         migration to help_entries_all() — D-29.2 / CLI-04 / SC-4 ROADMAP. \
         Confirms that Math1 Matrix category entries surface in the right-panel."
    );
}

/// Asserts that `key_ref_entries()` still contains v2.2 entries after the
/// migration to `help_entries_all()`.
///
/// Negative-regression test: the migration must not drop existing v2.2 entries
/// from the right-panel listing. The v2.2 Add op (`("+", "+")`) is chosen as
/// a sentinel because it is a fundamental arithmetic entry present since Phase 1.
#[test]
fn key_ref_entries_preserves_v22_entries() {
    let entries = key_ref_entries();
    let found = entries
        .iter()
        .any(|(key_path, display)| key_path == "+" && display == "+");
    assert!(
        found,
        "key_ref_entries() must still include (\"+\", \"+\") after migration to \
         help_entries_all() — negative-regression guard ensuring the v2.2 entry \
         pool is preserved (D-29.2). The merged accessor chains v2.2 + Math Pac I \
         in order; the v2.2 pool must not be dropped."
    );
}
