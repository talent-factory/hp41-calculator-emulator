// Phase 31-04 — Regression tests for Op::Catalog(2) XROM enumeration.
//
// Verifies the surgical hp41-core exception that replaces the "NOT AVAILABLE"
// stub with a real XROM module listing. Instant-scroll behavior (single-pass
// synchronous push) matches v2.2 CAT 1 per W1 fix. PSE-step deferred to v3.1.
//
// Catches: wrong stub left in place (regression), NO XROM fallback broken,
//          CAT 3/4 accidentally changed to enumerate instead of "NOT AVAILABLE".
#![allow(clippy::unwrap_used)]

use hp41_core::ops::math1::xrom::MATH_1;
use hp41_core::ops::program::op_catalog;
use hp41_core::state::CalcState;

/// CAT 2 with Math 1 loaded (default v3.0 config: xrom_modules bit 0 = 1)
/// must emit a module header line followed by every name in MATH_1.ops.
#[test]
fn catalog_2_with_math1_loaded_lists_header_and_functions() {
    let mut state = CalcState::default();
    // Verify default xrom_modules has bit 0 set (Math 1 pre-loaded).
    assert_ne!(
        state.xrom_modules & 0b0000_0001,
        0,
        "Default CalcState must have Math 1 loaded (bit 0 of xrom_modules)"
    );

    let initial_buf_len = state.print_buffer.len();
    op_catalog(&mut state, 2).unwrap();

    // op_catalog always prepends "-- CATALOG N --" and appends "-- END --"
    // so print_buffer grows by: 1 (header banner) + 1 (module header) + MATH_1.ops.len() + 1 (END).
    let new_lines = &state.print_buffer[initial_buf_len..];

    // First new line is the "-- CATALOG 2 --" banner.
    assert!(
        new_lines[0].contains("CATALOG 2"),
        "First line should be the catalog banner: {:?}",
        new_lines[0]
    );

    // Second new line is the module header.
    let module_header = &new_lines[1];
    assert!(
        module_header.contains("XROM 7"),
        "Module header should contain 'XROM 7': {module_header:?}"
    );
    assert!(
        module_header.contains("MATH 1"),
        "Module header should contain 'MATH 1A': {module_header:?}"
    );

    // Lines 2..N-1 are function names (skip banner + header + END).
    let function_lines = &new_lines[2..new_lines.len() - 1];
    assert_eq!(
        function_lines.len(),
        MATH_1.ops.len(),
        "Number of function lines should equal MATH_1.ops.len() ({})",
        MATH_1.ops.len()
    );

    // Total print_buffer growth = 1 (banner) + 1 (module header) + ops + 1 (END)
    assert_eq!(
        new_lines.len(),
        1 + 1 + MATH_1.ops.len() + 1,
        "Total lines = banner + module_header + ops + END"
    );

    // Verify the last line is the END banner.
    assert!(
        new_lines.last().unwrap().contains("END"),
        "Last line should be END banner: {:?}",
        new_lines.last()
    );

    // Verify MATH_1.ops has enough entries (at least 40 per must_have truth).
    assert!(
        MATH_1.ops.len() >= 40,
        "MATH_1.ops should have at least 40 entries, got {}",
        MATH_1.ops.len()
    );
}

/// CAT 2 with xrom_modules == 0 (no modules loaded) must emit a single
/// "NO XROM" line instead of module enumeration.
#[test]
fn catalog_2_without_xrom_emits_no_xrom() {
    let mut state = CalcState {
        xrom_modules: 0,
        ..CalcState::default()
    };

    let initial_buf_len = state.print_buffer.len();
    op_catalog(&mut state, 2).unwrap();

    let new_lines = &state.print_buffer[initial_buf_len..];

    // Expected: banner + "NO XROM" + END = 3 lines.
    assert_eq!(
        new_lines.len(),
        3,
        "With no modules, should emit banner + NO XROM + END (3 lines); got {}: {:?}",
        new_lines.len(),
        new_lines
    );

    // The "NO XROM" line should be new_lines[1].
    assert!(
        new_lines[1].contains("NO XROM"),
        "Second line should contain 'NO XROM': {:?}",
        new_lines[1]
    );
}

/// CAT 3 and CAT 4 (HP-IL and peripherals) must still emit "NOT AVAILABLE"
/// — they were not changed by the Plan 31-04 surgical exception.
#[test]
fn catalog_3_and_4_still_not_available() {
    let mut state = CalcState::default();

    // CAT 3
    let len_before = state.print_buffer.len();
    op_catalog(&mut state, 3).unwrap();
    let cat3_lines = &state.print_buffer[len_before..];
    assert!(
        cat3_lines.iter().any(|l| l.contains("NOT AVAILABLE")),
        "CAT 3 should still emit 'NOT AVAILABLE': {cat3_lines:?}"
    );

    // CAT 4
    let len_before = state.print_buffer.len();
    op_catalog(&mut state, 4).unwrap();
    let cat4_lines = &state.print_buffer[len_before..];
    assert!(
        cat4_lines.iter().any(|l| l.contains("NOT AVAILABLE")),
        "CAT 4 should still emit 'NOT AVAILABLE': {cat4_lines:?}"
    );
}
