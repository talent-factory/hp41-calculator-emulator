//! SC-4 invariant integration test.
//!
//! Asserts that the stricter SC-4 grep `grep -rn -E
//! "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)\("
//! hp41-gui/src-tauri/src/` returns zero matches.
//!
//! The spirit of SC-4: "no calculator/math logic duplicated in hp41-gui".
//! The existing `fn op_display_name(...)` in `prgm_display.rs` is the documented
//! exception (a display formatter, not calculator logic); the STRICTER pattern
//! used here excludes `op_display_name` and matches only the forbidden math
//! function names.
//!
//! This test is a CI-gate equivalent. If a future commit introduces one of the
//! forbidden functions in `hp41-gui/src-tauri/src/`, this test fails immediately,
//! converting the CLAUDE.md narrative reminder into an enforced runtime gate.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

/// Catches: SC-4 violation (math logic duplicated in hp41-gui/src-tauri/src/)
///
/// The test passes when `grep` exits with status code 1 (no matches found).
/// The test fails when `grep` exits with status code 0 (matches found),
/// indicating a new forbidden function has been introduced in the GUI source tree.
#[test]
fn sc4_grep_returns_no_matches() {
    // Resolve path to hp41-gui/src-tauri/src/ relative to the test crate manifest dir.
    // CARGO_MANIFEST_DIR is hp41-gui/src-tauri/ for this crate.
    // Parent is hp41-gui/, parent-parent would be repo root.
    // We want: <CARGO_MANIFEST_DIR>/src
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = manifest_dir.join("src");

    assert!(
        src_path.exists(),
        "Expected hp41-gui/src-tauri/src/ to exist at: {}",
        src_path.display()
    );

    // The stricter SC-4 pattern (per CLAUDE.md — excludes op_display_name).
    // Matches: fn op_add(, fn op_sub(, fn op_mul(, fn op_div(, fn op_sin(,
    //          fn op_cos(, fn op_tan(, fn op_sto(, fn op_rcl(,
    //          fn flush_entry(, fn format_hpnum(
    let grep_pattern = "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)\\(";

    let output = Command::new("grep")
        .args(["-rn", "-E", grep_pattern, src_path.to_str().unwrap()])
        .output()
        .expect("failed to invoke grep — is grep installed?");

    // grep exit code semantics:
    //   0 → at least one match found (SC-4 VIOLATED — test must fail)
    //   1 → no matches found (SC-4 PRESERVED — test must pass)
    //   2 → error (grep itself failed — treat as test failure)
    let exit_code = output.status.code().unwrap_or(2);

    assert_ne!(
        exit_code, 0,
        "SC-4 invariant VIOLATED: grep found forbidden math functions in hp41-gui/src-tauri/src/.\n\
         Matches found:\n{}\n\n\
         The following function names are forbidden in hp41-gui/src-tauri/src/:\n\
         op_add, op_sub, op_mul, op_div, op_sin, op_cos, op_tan, op_sto, op_rcl,\n\
         flush_entry, format_hpnum\n\n\
         (See CLAUDE.md §SC-4 invariant for the rationale.)",
        String::from_utf8_lossy(&output.stdout)
    );

    assert_eq!(
        exit_code, 1,
        "grep exited with error code {exit_code} (expected 0=match or 1=no-match). \
         Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
