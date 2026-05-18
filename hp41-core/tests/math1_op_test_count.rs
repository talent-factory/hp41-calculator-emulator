// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 meta-test: each Math Pac I `Op` variant added in Plans 28-02..28-10
//! must have at least 5 test functions mentioning it (Pitfall 16 guard).
//!
//! **Strategy (Plan 28-01 stage — vacuous):**
//!
//! 1. Read `hp41-core/src/ops/mod.rs` at test time via `include_str!` to find
//!    Phase-28 Op variant names (those matching the pattern `Op::Sinh`, `Op::Cosh`, etc.).
//! 2. Scan `hp41-core/tests/math1_*.rs` files via `std::fs::read_to_string` to count
//!    how many `#[test]` functions mention each variant name.
//! 3. Assert ≥ 5 mentions per variant.
//!
//! **Plan 32-01 (graduation, 2026-05-18):** gate graduated to non-vacuous —
//! all 45 Math Pac I variants meet the ≥ 5 mentions threshold per the Per-Op
//! Test Count Audit in `.planning/phases/32-test-hardening/32-RESEARCH.md`.
//! The minimum-count baseline is **TriSaa=6, TriSas=6**; any drop below the
//! baseline in a future commit is immediately visible in diff review per the
//! T-32-04 meta-test-gaming threat mitigation. Plans 28-02..28-10 grew the
//! Op enum and the `math1_*.rs` test files; this gate now actively counts
//! variant mentions across the 14 `math1_*.rs` files.
//!
//! **Scan scope:**
//! - Source file: `hp41-core/src/ops/mod.rs` (Op enum definition)
//! - Test files: `hp41-core/tests/math1_*.rs` (glob-matched at test time via
//!   `std::fs::read_dir`)
//!
//! **Math Pac I variant detection heuristic:**
//! Op variants added in Phase 28 are expected to follow the naming pattern used
//! in `math1_resolve()` in `hp41-core/src/ops/math1/xrom.rs`. The scan looks for
//! variants listed in the `math1_resolve` match block (by searching for lines
//! matching `"NAME" => Some(Op::VariantName)` in xrom.rs), then counts test function
//! bodies mentioning `VariantName` in math1_*.rs.
//!
//! This heuristic is intentionally conservative: it requires new variants to appear
//! in `xrom.rs` (which they must — otherwise `xrom_resolve` can't dispatch them)
//! AND have ≥ 5 test references. It does NOT scan `ops/mod.rs` for ALL Op variants
//! (which would force ≥ 5 tests on pre-existing v2.2 variants too, which is wrong).
//!
//! **Word-boundary matching (WR-03, Plan 32-09):**
//! `count_test_mentions` uses word-boundary matching to prevent `Op::Sol` from
//! matching `Op::Solve` (substring inflation). The match token is `Op::VariantName`
//! anchored such that the character immediately after the variant name must NOT be
//! alphanumeric or underscore. Counts distinct `#[test]` function blocks whose body
//! contains at least one such match (not raw line occurrences).

#![allow(clippy::unwrap_used)]

use std::path::Path;

/// Scan `hp41-core/src/ops/math1/xrom.rs` for Math Pac I variant names registered
/// in `math1_resolve()`. Returns a list of variant name strings (e.g. "Sinh", "Cosh").
///
/// Detection: lines matching `Some(Op::` are scanned for the variant name.
/// Empty list at Plan 28-01 (math1_resolve returns None for all names).
fn collect_math1_variant_names() -> Vec<String> {
    // Locate xrom.rs relative to the test binary's working directory.
    // `cargo test` sets cwd to the workspace root, so the relative path works.
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    let mut variants = Vec::new();

    for line in xrom_src.lines() {
        let trimmed = line.trim();
        // Detect lines like: `"SINH" => Some(Op::Sinh),`
        if trimmed.contains("=> Some(Op::") && !trimmed.starts_with("//") {
            if let Some(after_op) = trimmed.split("Some(Op::").nth(1) {
                // Extract variant name (up to ')' or ',')
                let variant_name: String = after_op
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !variant_name.is_empty() {
                    variants.push(variant_name);
                }
            }
        }
    }
    variants
}

/// Return true if `line` contains `Op::<variant_name>` as a whole token — i.e., the
/// character immediately after `variant_name` is NOT alphanumeric or `_`. This
/// prevents `Op::Sol` from matching lines that contain `Op::Solve`, `Op::Sinh`
/// matching `Op::Asinh`, etc. (WR-03 word-boundary fix).
///
/// Comment lines are pre-filtered before calling this helper.
fn line_mentions_variant(line: &str, variant_name: &str) -> bool {
    let token = format!("Op::{variant_name}");
    let mut search_start = 0;
    while let Some(pos) = line[search_start..].find(&token) {
        let abs_pos = search_start + pos;
        let after_pos = abs_pos + token.len();
        // Check word boundary: char after variant must not be alphanumeric or '_'
        let boundary_ok = match line[after_pos..].chars().next() {
            None => true, // end of line — boundary ok
            Some(c) => !c.is_alphanumeric() && c != '_',
        };
        if boundary_ok {
            return true;
        }
        search_start = abs_pos + 1;
    }
    false
}

/// Count how many distinct `#[test]` function blocks in `hp41-core/tests/math1_*.rs`
/// contain at least one word-boundary mention of `Op::<variant_name>`.
///
/// Strategy: split each file on `#[test]` markers to get per-function slices, then
/// check each slice for a word-boundary `Op::VariantName` match. This avoids the
/// line-count inflation of the prior implementation where 5 mentions in 1 test
/// function could satisfy the "≥ 5 test functions" intent.
///
/// Catches: WR-03 substring inflation (`Op::Sol` matching `Op::Solve`) — word-boundary
/// matching ensures each Op variant is only counted when it appears unambiguously.
fn count_test_mentions(variant_name: &str, tests_dir: &Path) -> usize {
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut total_fn_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.starts_with("math1_") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Split on `#[test]` markers to get per-function body slices.
        // Each slice covers one test function's declaration + body.
        let test_slices: Vec<&str> = content.split("#[test]").skip(1).collect();

        for slice in test_slices {
            // Check each non-comment line in this test function's slice for a
            // word-boundary Op::VariantName mention.
            let found = slice.lines().any(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("//") && line_mentions_variant(line, variant_name)
            });
            if found {
                total_fn_count += 1;
            }
        }
    }

    total_fn_count
}

/// Meta-test: every Math Pac I Op variant registered in `math1_resolve` must have
/// ≥ 5 test function references in `hp41-core/tests/math1_*.rs`.
///
/// At Plan 28-01: vacuously passes (no variants registered yet).
/// Becomes non-trivial in Plans 28-02..28-10 as variants are registered.
///
/// Catches: Pitfall 16 — Op variants with fewer than 5 tests risk missing edge cases
/// that the Phase 32 coverage gate would otherwise catch.
#[test]
fn each_math1_op_has_at_least_5_tests() {
    let variants = collect_math1_variant_names();

    // Find the tests directory using CARGO_MANIFEST_DIR (set at compile time to the
    // hp41-core package directory). This is robust across workspace root / package root
    // CWD differences when running tests via `cargo test -p hp41-core`.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir_buf = manifest_dir.join("tests");
    let tests_dir = tests_dir_buf.as_path();

    let mut failures: Vec<String> = Vec::new();
    for variant_name in &variants {
        let count = count_test_mentions(variant_name, tests_dir);
        if count < 5 {
            failures.push(format!(
                "Op::{variant_name}: only {count} test mention(s) in math1_*.rs (need ≥ 5)"
            ));
        }
    }

    // Catches: Pitfall 16 — Op variants with insufficient test coverage risk
    // missing edge cases that the Phase 32 coverage gate would otherwise catch.
    // Catches: WR-03 substring inflation (`Sol` matching `Solve`) — now uses
    // word-boundary matching; `Op::Sol` no longer borrows count from `Op::Solve`.
    // T-32-04: per-Op count baseline (TriSaa=6, TriSas=6 minimum); a drop below
    // this floor in a future commit is visible in diff review.
    assert!(
        failures.is_empty(),
        "Pitfall 16 violation — Math Pac I variants with insufficient test coverage:\n{}",
        failures.join("\n")
    );
}
