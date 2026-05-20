// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Plan 32-01 / T-32-04 mitigation — assertion-discipline lint over `tests/math1_*.rs`.
//!
//! Two `#[test]` gates enforce the Phase 32 tolerance discipline that Plans
//! 28-02..28-10 left informal:
//!
//! 1. **`no_decimal_assert_eq_in_math1_tests`** (Pitfall 17): `assert_eq!`
//!    invocations comparing two BCD `HpNum` / `Decimal` / `f64` values are
//!    forbidden — those flow paths can drift across x86 and ARM FPUs by the
//!    last digit (the BCD layer hides the drift inside `hp41-core` but the
//!    `to_f64()` bridge re-exposes it). Use `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` instead.
//!
//! 2. **`no_manual_tolerance_pattern_in_math1_tests`** (Pitfall 14): manual
//!    `(actual - expected).abs() < EPSILON` patterns undermine the
//!    single-source-of-truth `max_relative = 1e-7` discipline established in
//!    `hp41-core/tests/math1_complex.rs`. Use `approx::assert_relative_eq!`
//!    so every relative-equality check lives under one canonical tolerance
//!    knob — D-27.1 risk-weighted discipline applied to tolerance choice.
//!
//! ## Heuristic
//!
//! The two lints are line-level greps with conservative recognition rules to
//! distinguish acceptable from forbidden patterns. Per RESEARCH Open Q2:
//!
//! - **Acceptable** (string / enum / integer equality — exact, cross-platform-safe):
//!   ```text
//!   assert_eq!(state.modal_prompt, Some("ORDER=?".to_string()))      // string
//!   assert_eq!(result, Err(HpError::InvalidOp))                       // enum
//!   assert_eq!(state.regs[0], HpNum::from(5i32))                      // int (LINT-EXEMPT)
//!   ```
//! - **Forbidden** (decimal / float equality — drift-prone):
//!   ```text
//!   assert_eq!(x_val, 0.5)                                            // float
//!   assert_eq!(state.stack.x.inner().to_f64().unwrap(), 4.0)          // bridge
//!   ```
//!
//! ## LINT-EXEMPT annotations
//!
//! A line containing `// LINT-EXEMPT: <reason>` is excluded from both lints.
//! The annotation MUST give a specific rationale (T-32-04: reviewers spot
//! drive-by allowlisting). Examples: integer-equality via `HpNum::from(<int>)`,
//! coarse-tolerance triangulation / Fourier / integration cases where the
//! algorithmic precision floor is intentionally above 1e-7.
//!
//! ## Scope
//!
//! Only `hp41-core/tests/math1_*.rs` files (Phase 32 Claude's Discretion
//! option (a)). Does NOT scan `tests/numerical_accuracy.rs` (the `case!` macro
//! has its own internal `AccuracyCase.tol` bookkeeping). Future widening
//! (option (b) — all `tests/`) is a v3.1+ consideration when a gap surfaces.
//!
//! ## Multi-line assert_eq detection (WR-02, Plan 32-09)
//!
//! The `no_decimal_assert_eq_in_math1_tests` lint detects both single-line and
//! multi-line `assert_eq!(decimal, decimal)` invocations. The lookahead window
//! is the `assert_eq!` line plus up to 3 following lines. This closes the
//! blind spot where a multi-line invocation like:
//! ```rust
//! assert_eq!(
//!     s.stack.x.inner(),  // .inner() was invisible to the old single-line heuristic
//!     x_before,
//!     "..."
//! );
//! ```
//! would not be detected by the prior single-line heuristic.
//!
//! **Known false-positive class:** `assert_eq!` calls whose COMPARISON ARGUMENTS
//! are integer-typed (e.g., `HpNum::from(5i32)`, `.to_i32()`) but where the
//! lookahead window happens to contain decimal tokens in SUBSEQUENT unrelated
//! lines (e.g., setup code after the assertion). These are annotated with
//! `// LINT-EXEMPT: integer-equality — <reason>` per the established pattern.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

/// Token that opts a line out of both lint gates. MUST carry a specific
/// rationale per T-32-04 (the threat is silent allowlisting in a future commit).
const LINT_EXEMPT_TOKEN: &str = "LINT-EXEMPT:";

/// Iterate `hp41-core/tests/math1_*.rs` files. Returns `Vec<(PathBuf, String)>`
/// of (path, file-contents). Mirrors the directory-scan loop in
/// `math1_op_test_count.rs::count_test_mentions` per the 32-PATTERNS.md analog.
fn collect_math1_test_files(tests_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
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
        files.push((path, content));
    }
    files
}

/// Format a single offender line as `"{filename}:{line_no}: {trimmed_source_line}"`.
/// Pattern matches `math1_op_test_count.rs::each_math1_op_has_at_least_5_tests`
/// failure-list formatting per the 32-PATTERNS.md analog.
fn format_offender(path: &Path, line_no: usize, line: &str) -> String {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<unknown>");
    format!("{}:{}: {}", name, line_no, line.trim())
}

/// Check the preceding lines for a `LINT-EXEMPT:` annotation that applies
/// to the assertion at `lines[idx]`. Walks upward, skipping continuation
/// lines of the same `assert!(...)` macro (lines that look like part of a
/// multi-line macro invocation — opening `assert!(` or trailing argument
/// lines like `Op::Integ ... must be ..."` or commas/whitespace). Once a
/// comment block is reached, the block is scanned for `LINT-EXEMPT:`.
///
/// The annotation MUST be adjacent to the offender's enclosing item (no
/// blank line OR semicolon-terminated statement between), so future
/// drive-by allowlisting is visible in diff review per T-32-04.
fn preceding_block_has_lint_exempt(lines: &[&str], idx: usize) -> bool {
    let mut i = idx;
    // Phase 1: walk upward past continuation lines of the same macro/expression.
    // Stop when we hit a comment, blank line, or terminating ';' / '}'.
    while i > 0 {
        i -= 1;
        let trimmed = lines[i].trim_start();
        if trimmed.is_empty() {
            return false;
        }
        if trimmed.starts_with("//") {
            // Reached the comment block — fall through to Phase 2.
            break;
        }
        let trimmed_end = lines[i].trim_end();
        if trimmed_end.ends_with(';') || trimmed_end.ends_with('}') {
            // Reached the previous statement; LINT-EXEMPT must precede it.
            return false;
        }
        // Otherwise: continuation line of the offender's enclosing
        // `assert!(...)` macro — keep walking upward.
    }
    // Phase 2: scan the contiguous comment block for LINT-EXEMPT.
    loop {
        let trimmed = lines[i].trim_start();
        if !trimmed.starts_with("//") {
            return false;
        }
        if trimmed.contains(LINT_EXEMPT_TOKEN) {
            return true;
        }
        if i == 0 {
            return false;
        }
        i -= 1;
        if lines[i].trim().is_empty() {
            return false;
        }
    }
}

/// Heuristic for forbidden `assert_eq!(decimal, decimal)` lines per Pitfall 17.
///
/// A line is forbidden iff it contains `assert_eq!` AND the line itself OR up
/// to 3 following lines (multi-line invocation lookahead) contain any of:
/// `.to_f64()`, `HpNum`, `Decimal`, `.inner()`.
///
/// **Multi-line detection (WR-02):** both single-line and multi-line
/// `assert_eq!(decimal, decimal)` invocations are detected. The lookahead
/// window is: current line + 3 following lines joined with `\n`.
/// Example caught by lookahead:
/// ```rust
/// assert_eq!(          // line N   — contains assert_eq!
///     s.stack.x.inner(), // line N+1 — contains .inner()
///     x_before,
///     "...",
/// );
/// ```
///
/// Lines bearing `LINT-EXEMPT:` (inline) OR carrying a `LINT-EXEMPT:` in the
/// preceding contiguous comment block are exempted (e.g., integer equality
/// via `HpNum::from(5i32)` is exact and cross-platform-safe). Comment lines
/// (`//` / `///` / `//!`) are exempted — they describe the code, not
/// execute it; mirrors `math1_op_test_count.rs::count_test_mentions`.
fn line_is_forbidden_assert_eq(line: &str, lines: &[&str], idx: usize) -> bool {
    if line.contains(LINT_EXEMPT_TOKEN) {
        return false;
    }
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return false;
    }
    if !line.contains("assert_eq!") {
        return false;
    }
    // Check the line itself AND up to 3 following lines (multi-line invocation
    // lookahead). take(4) = current line + 3 following.
    let next_3: String = lines
        .iter()
        .skip(idx)
        .take(4)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    let is_decimal = next_3.contains(".to_f64()")
        || next_3.contains("HpNum")
        || next_3.contains("Decimal")
        || next_3.contains(".inner()");
    if !is_decimal {
        return false;
    }
    if preceding_block_has_lint_exempt(lines, idx) {
        return false;
    }
    true
}

/// Heuristic for forbidden manual-tolerance `(a - b).abs() < EPSILON` lines
/// per Pitfall 14.
///
/// A line is forbidden iff it matches the textual pattern `).abs() <` AND
/// contains a `-` inside parentheses on the same line. The regex-free
/// approach keeps the lint dep-free; the false-positive rate against the
/// current `math1_*.rs` corpus is zero (comment-lines and LINT-EXEMPT
/// annotations are pre-filtered).
///
/// Lines bearing `LINT-EXEMPT:` (inline) OR carrying a `LINT-EXEMPT:` in the
/// preceding contiguous comment block are exempted. Comment lines (`//` /
/// `///` / `//!`) are exempted — they describe the code, not execute it.
fn line_is_forbidden_manual_tolerance(line: &str, lines: &[&str], idx: usize) -> bool {
    if line.contains(LINT_EXEMPT_TOKEN) {
        return false;
    }
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return false;
    }
    // Look for ").abs() <" anchor.
    let Some(abs_idx) = line.find(").abs() <") else {
        return false;
    };
    // Look for a "-" inside parentheses before the anchor (i.e., the
    // `(a - b)` pattern, not `(x).abs() < ...` which uses single-arg abs).
    let prefix = &line[..abs_idx];
    let Some(open_idx) = prefix.rfind('(') else {
        return false;
    };
    let inner = &prefix[open_idx + 1..];
    // Require a ' - ' (with surrounding spaces) so we don't match negative
    // literals like `(-1.0).abs()`.
    if !inner.contains(" - ") {
        return false;
    }
    if preceding_block_has_lint_exempt(lines, idx) {
        return false;
    }
    true
}

/// Catches: Pitfall 17 — `assert_eq!(decimal, decimal)` on iterated results
/// can drift across x86 and ARM FPUs by the last digit. The BCD layer hides
/// the drift inside `hp41-core`, but `to_f64()` bridges re-expose it. T-32-04:
/// the offender list is reported in full so a reviewer can spot weakening.
#[test]
fn no_decimal_assert_eq_in_math1_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let files = collect_math1_test_files(&tests_dir);

    let mut offenders: Vec<String> = Vec::new();
    for (path, content) in &files {
        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            if line_is_forbidden_assert_eq(line, &lines, idx) {
                offenders.push(format_offender(path, idx + 1, line));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "Pitfall 17 violation — `assert_eq!(decimal, decimal)` on iterated \
         results (use `approx::assert_relative_eq!(actual, expected, \
         max_relative = 1e-7)` or add a `// LINT-EXEMPT: <reason>` annotation):\n{}",
        offenders.join("\n")
    );
}

/// Catches: Pitfall 14 — manual `(a - b).abs() < EPSILON` patterns undermine
/// the single-source-of-truth `max_relative = 1e-7` discipline. T-32-04: the
/// offender list is reported in full so a reviewer can spot weakening.
#[test]
fn no_manual_tolerance_pattern_in_math1_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let files = collect_math1_test_files(&tests_dir);

    let mut offenders: Vec<String> = Vec::new();
    for (path, content) in &files {
        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            if line_is_forbidden_manual_tolerance(line, &lines, idx) {
                offenders.push(format_offender(path, idx + 1, line));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "Pitfall 14 violation — manual `(a - b).abs() < EPSILON` (use \
         `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` \
         or add a `// LINT-EXEMPT: <reason>` annotation):\n{}",
        offenders.join("\n")
    );
}
