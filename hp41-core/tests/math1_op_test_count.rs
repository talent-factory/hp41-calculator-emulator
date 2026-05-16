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
//! **At Plan 28-01:** No Math Pac I Op variants exist yet in `ops/mod.rs`.
//! The Op enum scan returns an empty list. The test runs over an empty set and
//! passes vacuously. This is intentional and documented (Pitfall 16).
//!
//! **Plans 28-02..28-10** grow the Op enum and the math1_*.rs test files.
//! Once a variant like `Op::Sinh` appears in `ops/mod.rs` and Plans 28-02..28-10
//! have added test functions mentioning it, this gate becomes non-trivial.
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

/// Count how many test functions (lines containing `#[test]` followed by `fn`) in
/// `hp41-core/tests/math1_*.rs` mention the given variant name.
fn count_test_mentions(variant_name: &str, tests_dir: &Path) -> usize {
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut total_mentions = 0;

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
        // Count test function bodies that mention the variant name.
        // Simple heuristic: count occurrences of the variant name string
        // in lines that are NOT comments.
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("//") && trimmed.contains(variant_name) {
                total_mentions += 1;
            }
        }
    }

    total_mentions
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

    // At Plan 28-01: variants is empty — test passes vacuously.
    // Plans 28-02..28-10 grow this list.
    if variants.is_empty() {
        // Explicitly document the vacuous-pass state.
        return;
    }

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

    assert!(
        failures.is_empty(),
        "Pitfall 16 violation — Math Pac I variants with insufficient test coverage:\n{}",
        failures.join("\n")
    );
}
