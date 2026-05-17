//! Phase 25 Plan 04 Task 1 smoke tests — `docs/hp41cv-functions.json` is the
//! canonical data source for `hp41-cli/src/help_data.rs` via include_str! +
//! OnceLock per D-25.16 / D-25.17.
//!
//! Pitfall 7 belt-and-braces: assert >= 130 entries so a future contributor
//! cannot silently land an empty/short JSON without CI failing.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries, help_entries_all, help_overlay_rows};

#[test]
fn help_entries_loads_at_runtime() {
    let entries = help_entries();
    assert!(
        !entries.is_empty(),
        "help_entries() must return a non-empty slice — \
         docs/hp41cv-functions.json may be empty or malformed (D-25.17)"
    );
}

#[test]
fn help_entries_count_meets_130_target() {
    // D-25.16 / Pitfall 7: the canonical JSON must list every HP-41CV ROM op
    // (>= 130) plus the v3.x-deferred Module-Pac entries. An empty-file
    // commit would slip past `include_str!` but fail this assertion.
    let entries = help_entries();
    assert!(
        entries.len() >= 130,
        "help_entries().len() = {} — must be >= 130 per D-25.16 (Pitfall 7 \
         CI guard against empty/short JSON commits)",
        entries.len()
    );
}

#[test]
fn help_entries_has_at_least_thirteen_categories() {
    // Pre-Plan-04 the legacy HELP_DATA had 13 visible category headers; the
    // JSON pipeline must preserve at least this breadth.
    let entries = help_entries();
    let distinct: HashSet<&str> = entries.iter().map(|e| e.category.as_str()).collect();
    assert!(
        distinct.len() >= 13,
        "expected >= 13 distinct categories, got {}: {:?}",
        distinct.len(),
        distinct
    );
}

#[test]
fn help_entries_has_no_duplicate_op_variants() {
    // The bidirectional parity test in tests/function_matrix_parity.rs
    // assumes a unique op_variant per row. Catch duplicates here so the
    // parity-test failure mode points at the JSON, not at the parity test.
    let entries = help_entries();
    let mut seen: HashSet<&str> = HashSet::with_capacity(entries.len());
    for entry in entries {
        assert!(
            seen.insert(entry.op_variant.as_str()),
            "duplicate op_variant in docs/hp41cv-functions.json: {}",
            entry.op_variant
        );
    }
}

#[test]
fn help_entries_all_have_non_empty_description() {
    // Description renders in the ? overlay — empty rows would render as
    // blank lines and confuse users. Also doubles as a JSON sanity check.
    for entry in help_entries() {
        assert!(
            !entry.description.is_empty(),
            "entry '{}' has empty description",
            entry.op_variant
        );
        assert!(
            entry.description.len() <= 80,
            "entry '{}' description exceeds 80 chars ({} chars) — \
             will overflow the ? overlay table column",
            entry.op_variant,
            entry.description.len()
        );
    }
}

#[test]
fn help_entries_status_is_closed_enum() {
    // The status field is conceptually an enum but encoded as a string.
    // Constrain it to the three permitted values per D-25.16.
    for entry in help_entries() {
        assert!(
            matches!(entry.status.as_str(), "implemented" | "deferred-v3" | "na"),
            "entry '{}' has invalid status '{}' — must be \
             implemented | deferred-v3 | na",
            entry.op_variant,
            entry.status
        );
    }
}

#[test]
fn help_overlay_rows_contain_category_headers() {
    // The ? overlay consumes help_overlay_rows() which interleaves synthetic
    // === <category> === header rows between groups. Verify each distinct
    // category produces exactly one header row.
    //
    // D-29.2: help_overlay_rows() now reads from help_entries_all() (merged
    // v2.2 + Math Pac I pool), so the expected category count is derived from
    // help_entries_all() to stay in sync with the merged accessor.
    let rows = help_overlay_rows();

    let mut distinct_categories: Vec<String> = Vec::new();
    for e in help_entries_all() {
        if !distinct_categories.iter().any(|c| c == &e.category) {
            distinct_categories.push(e.category.clone());
        }
    }

    let header_count = rows
        .iter()
        .filter(|r| r.desc.starts_with("===") && r.desc.ends_with("==="))
        .count();
    assert_eq!(
        header_count,
        distinct_categories.len(),
        "help_overlay_rows must produce one === header === per category, \
         got {} headers for {} categories",
        header_count,
        distinct_categories.len()
    );
}
