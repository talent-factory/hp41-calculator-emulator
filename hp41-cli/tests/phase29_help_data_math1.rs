//! Phase 29 Plan 01 Task 1 smoke tests — `docs/hp41-math1-functions.json` is the
//! canonical data source for `hp41-cli/src/help_data.rs::help_entries_math1()` via
//! include_str! + OnceLock per D-29.1 / D-29.2.
//!
//! Mirrors `phase25_help_data.rs` structure exactly; swaps the accessor and
//! count target (>= 45 unique Op variants after deduplication of ASCII aliases).
//!
//! Pitfall 7 belt-and-braces: assert >= 45 entries so a future contributor
//! cannot silently land an empty/short JSON without CI failing.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries_all, help_entries_math1};

#[test]
fn math1_help_entries_loads_at_runtime() {
    // Catches: hard-build-blocker not firing on malformed JSON (D-25.17 / D-29.2)
    let entries = help_entries_math1();
    assert!(
        !entries.is_empty(),
        "help_entries_math1() must return a non-empty slice — \
         docs/hp41-math1-functions.json may be empty or malformed (D-29.2)"
    );
}

#[test]
fn math1_help_entries_count_meets_45_target() {
    // D-29.1 / D-29.2 / Pitfall 7: the canonical JSON must list every Math Pac I Op
    // variant (>= 45, deduplicated — 52 total MATH_1.ops entries minus 7 ASCII aliases).
    // An empty-file commit would slip past include_str! but fail this assertion.
    let entries = help_entries_math1();
    assert!(
        entries.len() >= 45,
        "help_entries_math1().len() = {} — must be >= 45 per D-29.1 (Pitfall 7 \
         CI guard against empty/short hp41-math1-functions.json commits)",
        entries.len()
    );
}

#[test]
fn math1_help_entries_has_no_duplicate_op_variants() {
    // Catches: duplicate op_variant rows in hp41-math1-functions.json.
    // The bidirectional parity test assumes unique op_variant per row.
    let entries = help_entries_math1();
    let mut seen: HashSet<&str> = HashSet::with_capacity(entries.len());
    for entry in entries {
        assert!(
            seen.insert(entry.op_variant.as_str()),
            "duplicate op_variant in docs/hp41-math1-functions.json: {}",
            entry.op_variant
        );
    }
}

#[test]
fn math1_help_entries_all_have_non_empty_description() {
    // Catches: empty or over-long descriptions that break the ? overlay.
    // Description renders in the ? overlay — empty rows would render as
    // blank lines and confuse users. Also doubles as a JSON sanity check.
    for entry in help_entries_math1() {
        assert!(
            !entry.description.is_empty(),
            "entry '{}' in hp41-math1-functions.json has empty description",
            entry.op_variant
        );
        assert!(
            entry.description.len() <= 200,
            "entry '{}' description exceeds 200 chars ({} chars)",
            entry.op_variant,
            entry.description.len()
        );
    }
}

#[test]
fn math1_help_entries_status_is_closed_enum() {
    // Catches: invalid status string in hp41-math1-functions.json.
    // The status field is conceptually an enum but encoded as a string.
    // Constrain it to the three permitted values per D-25.16.
    for entry in help_entries_math1() {
        assert!(
            matches!(entry.status.as_str(), "implemented" | "deferred-v3" | "na"),
            "entry '{}' in hp41-math1-functions.json has invalid status '{}' — \
             must be implemented | deferred-v3 | na",
            entry.op_variant,
            entry.status
        );
    }
}

#[test]
fn math1_help_entries_all_xrom_module_id_is_7() {
    // Catches: missing or incorrect xrom.module_id (C-28.3 invariant).
    // Every Math Pac I entry MUST carry an xrom block with module_id == 7
    // (HP-41 Math Pac I hardware XROM module ID).
    for entry in help_entries_math1() {
        let xrom = entry.xrom.as_ref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-math1-functions.json is missing the xrom block (C-28.3 invariant: every Math1 entry must carry xrom.module_id == 7)",
                entry.op_variant
            )
        });
        assert_eq!(
            xrom.module_id, 7,
            "entry '{}' in hp41-math1-functions.json has xrom.module_id == {} — must be 7 (HP Math Pac I hardware module ID)",
            entry.op_variant, xrom.module_id
        );
    }
}

#[test]
fn math1_help_entries_categories_prefix_with_math1() {
    // Catches: wrong category prefix breaking ? overlay sectioning.
    // Every Math Pac I entry category must start with "Math1 " so the overlay
    // sections cluster separately from the v2.2 built-in categories.
    for entry in help_entries_math1() {
        assert!(
            entry.category.starts_with("Math1 "),
            "entry '{}' in hp41-math1-functions.json has category '{}' — \
             must start with 'Math1 ' (per D-29.2 overlay sectioning)",
            entry.op_variant,
            entry.category
        );
    }
}

#[test]
fn math1_help_entries_xrom_function_ids_are_dense() {
    // Catches: duplicate or non-contiguous function_id assignment.
    // function_ids must form a dense 1..=N range with no gaps and no duplicates.
    // This ensures the JSON is consistent and matches the MATH_1.ops ordering.
    let entries = help_entries_math1();
    let mut ids: Vec<u16> = entries
        .iter()
        .map(|e| {
            e.xrom
                .as_ref()
                .unwrap_or_else(|| panic!("entry '{}' missing xrom block", e.op_variant))
                .function_id
        })
        .collect();
    ids.sort_unstable();

    // Check no duplicates
    let unique_count = {
        let mut deduped = ids.clone();
        deduped.dedup();
        deduped.len()
    };
    assert_eq!(
        ids.len(),
        unique_count,
        "hp41-math1-functions.json contains duplicate function_id values"
    );

    // Check dense range starting at 1
    let max_id = *ids.last().unwrap_or(&0);
    assert_eq!(
        ids.len() as u16,
        max_id,
        "hp41-math1-functions.json function_ids must form a dense 1..={} range, \
         but the max is {} with {} entries",
        max_id,
        max_id,
        ids.len()
    );
    assert_eq!(
        ids.first().copied().unwrap_or(0),
        1,
        "hp41-math1-functions.json function_ids must start at 1 (HP-41 convention)"
    );
}

#[test]
fn math1_help_entries_all_key_path_is_xeq_form() {
    // Catches: wrong key_path format for Math Pac I entries (D-28.6 invariant).
    // Every Math Pac I entry key_path must be Some("XEQ \"<MNEMONIC>\"") because
    // Math Pac I functions are XEQ-by-name only — no dedicated key bindings.
    for entry in help_entries_math1() {
        let key_path = entry.key_path.as_deref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-math1-functions.json has no key_path (D-28.6: XEQ-by-name only, key_path must be Some(...))",
                entry.op_variant
            )
        });
        assert!(
            key_path.starts_with("XEQ \"") && key_path.ends_with('"'),
            "entry '{}' key_path '{}' must be in XEQ \"<MNEMONIC>\" form (D-28.6)",
            entry.op_variant,
            key_path
        );
    }
}

#[test]
fn math1_help_entries_all_returns_both_pools() {
    // Catches: help_entries_all() not chaining both pools.
    // help_entries_all() must return at least help_entries() + help_entries_math1() entries.
    let math1_count = help_entries_math1().len();
    let all_count = help_entries_all().count();
    assert!(
        all_count >= math1_count,
        "help_entries_all() returned {all_count} entries but help_entries_math1() alone has {math1_count} — \
         merged accessor must chain both pools"
    );
    // Must also include some v2.2 entries (>= 130 from the first pool)
    assert!(
        all_count >= 130 + math1_count,
        "help_entries_all() returned {} entries but should include >= 130 v2.2 entries + {} Math1 entries = {} total minimum",
        all_count,
        math1_count,
        130 + math1_count
    );
}
