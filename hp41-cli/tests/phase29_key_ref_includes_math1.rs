//! Post-v3.0 right-panel UX revert: Math Pac I rows are EXCLUDED from the
//! right-panel discoverability listing, returning the panel to its v2.2-era
//! role as a pure physical-keyboard reference.
//!
//! HISTORICAL CONTEXT — Phase 29 SC-4 + D-29.2 + CLI-04 originally specified
//! that `key_ref_entries()` should INCLUDE Math Pac I entries (since v2.2's
//! D-25.18 had migrated the right-panel from a hand-curated const to a
//! JSON-derived listing, and Math Pac I entries also carry `key_path`).
//! Once v3.0 shipped the user evaluation surfaced that the resulting ~45
//! `XEQ "..."` rows crowded out the actual keyboard reference and pushed the
//! v2.2 entries off-screen on typical terminal sizes — a UX regression that
//! the v3.0 PR review process did not catch because the JSON-derivation logic
//! itself was correct, but the resulting UX was not.
//!
//! POST-v3.0 RESOLUTION (this file, originally `phase29_key_ref_includes_math1.rs`):
//! `key_ref_entries()` gained an `entry.xrom.is_none()` filter excluding all
//! XROM-module functions from the right-panel. Module functions remain fully
//! discoverable via the `?` overlay, which groups them under a dedicated
//! "Math 1 Pac (XROM 7)" section (mirroring `hp41-gui`'s `HelpOverlay.tsx`
//! two-section collapsible layout from v3.0 Phase 31). The right-panel
//! returns to ~15–25 rows regardless of how many XROM modules are loaded,
//! which keeps the listing usable as more modules ship (v3.1 Stat, v3.2 Time,
//! v3.3 Advantage). The Phase 29 SC-4 / D-29.2 acceptance criteria are
//! formally superseded by this filter.
//!
//! The previous tests (`key_ref_entries_includes_math1_sinh` +
//! `_includes_math1_matrix`) are inverted to ENFORCE the exclusion as a
//! regression guard.

#![allow(clippy::unwrap_used)]

use hp41_cli::keys::key_ref_entries;

/// Asserts that `key_ref_entries()` EXCLUDES the Math Pac I `XEQ "SINH"` entry.
///
/// Catches: regression where the `entry.xrom.is_none()` filter is removed
/// from `key_ref_entries()`, causing the right-panel to balloon back to ~60
/// rows on v3.0 alone (and proportionally more as v3.1+ XROM modules ship).
/// SINH is the canonical Math Pac I discoverability sentinel (also used by
/// the GUI E2E smoke spec) so a regression here would be caught both in unit
/// tests AND in the E2E surface.
#[test]
fn key_ref_entries_excludes_math1_sinh() {
    let entries = key_ref_entries();
    let leaked = entries
        .iter()
        .any(|(key_path, display)| key_path == "XEQ \"SINH\"" && display == "SINH");
    assert!(
        !leaked,
        "key_ref_entries() must NOT include (\"XEQ \\\"SINH\\\"\", \"SINH\") — \
         the post-v3.0 `entry.xrom.is_none()` filter excludes XROM-module \
         functions from the right-panel. Math Pac I functions live in the `?` \
         overlay's \"Math 1 Pac (XROM 7)\" section, NOT in the keyboard reference."
    );
}

/// Asserts that `key_ref_entries()` EXCLUDES the Math Pac I `XEQ "MATRIX"` entry.
///
/// Same rationale as `key_ref_entries_excludes_math1_sinh`. MATRIX is in a
/// different category (Math1 Matrix vs Math1 Hyperbolic for SINH), confirming
/// the exclusion applies category-wide and not just to one branch.
#[test]
fn key_ref_entries_excludes_math1_matrix() {
    let entries = key_ref_entries();
    let leaked = entries
        .iter()
        .any(|(key_path, display)| key_path == "XEQ \"MATRIX\"" && display == "MATRIX");
    assert!(
        !leaked,
        "key_ref_entries() must NOT include (\"XEQ \\\"MATRIX\\\"\", \"MATRIX\") — \
         the post-v3.0 `entry.xrom.is_none()` filter excludes XROM-module \
         functions from the right-panel. Math Pac I MATRIX lives in the `?` \
         overlay, NOT in the keyboard reference."
    );
}

/// Asserts that `key_ref_entries()` still contains v2.2 built-in entries after
/// the post-v3.0 XROM-exclusion filter.
///
/// Negative-regression test: the new filter must not accidentally drop v2.2
/// entries (whose `xrom` field is `None`, distinguishing them from Math Pac I).
/// The v2.2 Add op (`("+", "+")`) is chosen as a sentinel because it is a
/// fundamental arithmetic entry present since Phase 1.
#[test]
fn key_ref_entries_preserves_v22_entries() {
    let entries = key_ref_entries();
    let found = entries
        .iter()
        .any(|(key_path, display)| key_path == "+" && display == "+");
    assert!(
        found,
        "key_ref_entries() must still include (\"+\", \"+\") after the post-v3.0 \
         XROM-exclusion filter — negative-regression guard ensuring v2.2 built-in \
         entries (with `xrom: None`) are NOT accidentally caught by the filter. \
         The filter drops ONLY entries where `xrom.is_some()`."
    );
}
