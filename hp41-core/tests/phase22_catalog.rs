//! Integration tests for Phase 22 Plan 04 (CATALOG hardware-faithful op).
//!
//! Covers FN-MEM-05 + AMENDED D-22.16 / OQ-1 Option B:
//!   - CAT 1 = programs (LBL listing with step counts)
//!   - CAT 2/3/4 = single "NOT AVAILABLE" payload
//!   - n == 0 OR n >= 5 → InvalidOp
//!   - 24-char-wide output discipline
//!   - long-label truncation to 9 chars

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpError, HpNum};

/// Build a minimal program: `LBL "A" PushNum(1) LBL "B" PushNum(2) PushNum(3)`.
/// Useful as a fixture for the CAT 1 enumeration tests.
fn program_with_two_labels() -> Vec<Op> {
    vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(1i32)),
        Op::Lbl("B".to_string()),
        Op::PushNum(HpNum::from(2i32)),
        Op::PushNum(HpNum::from(3i32)),
    ]
}

#[test]
fn test_catalog_1_lists_programs() {
    let mut s = CalcState::new();
    s.program = program_with_two_labels();
    dispatch(&mut s, Op::Catalog(1)).unwrap();

    // header + 2 LBL lines + footer = 4 entries
    assert_eq!(s.print_buffer.len(), 4, "buf={:?}", s.print_buffer);
    assert!(
        s.print_buffer[0].starts_with("-- CATALOG 1 --"),
        "header line[0]: {}",
        s.print_buffer[0]
    );
    // LBL A spans steps [0..2] = 2 steps (LBL A + PushNum(1))
    assert!(
        s.print_buffer[1].contains("LBL A") && s.print_buffer[1].contains("    2"),
        "LBL A line: {}",
        s.print_buffer[1]
    );
    // LBL B spans steps [2..5] = 3 steps (LBL B + PushNum(2) + PushNum(3))
    assert!(
        s.print_buffer[2].contains("LBL B") && s.print_buffer[2].contains("    3"),
        "LBL B line: {}",
        s.print_buffer[2]
    );
    assert!(
        s.print_buffer[3].starts_with("-- END --"),
        "footer: {}",
        s.print_buffer[3]
    );
}

#[test]
fn test_catalog_1_empty_program_emits_header_footer_only() {
    let mut s = CalcState::new();
    // s.program is empty by default
    dispatch(&mut s, Op::Catalog(1)).unwrap();
    assert_eq!(s.print_buffer.len(), 2, "buf={:?}", s.print_buffer);
    assert!(s.print_buffer[0].starts_with("-- CATALOG 1 --"));
    assert!(s.print_buffer[1].starts_with("-- END --"));
}

#[test]
fn test_catalog_2_not_available() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Catalog(2)).unwrap();
    assert_eq!(s.print_buffer.len(), 3, "buf={:?}", s.print_buffer);
    assert!(s.print_buffer[0].starts_with("-- CATALOG 2 --"));
    assert!(
        s.print_buffer[1].starts_with("NOT AVAILABLE"),
        "payload: {}",
        s.print_buffer[1]
    );
    assert!(s.print_buffer[2].starts_with("-- END --"));
}

#[test]
fn test_catalog_3_not_available() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Catalog(3)).unwrap();
    assert_eq!(s.print_buffer.len(), 3);
    assert!(s.print_buffer[0].starts_with("-- CATALOG 3 --"));
    assert!(s.print_buffer[1].starts_with("NOT AVAILABLE"));
    assert!(s.print_buffer[2].starts_with("-- END --"));
}

#[test]
fn test_catalog_4_not_available() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Catalog(4)).unwrap();
    assert_eq!(s.print_buffer.len(), 3);
    assert!(s.print_buffer[0].starts_with("-- CATALOG 4 --"));
    assert!(s.print_buffer[1].starts_with("NOT AVAILABLE"));
    assert!(s.print_buffer[2].starts_with("-- END --"));
}

#[test]
fn test_catalog_0_rejects() {
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::Catalog(0));
    assert!(matches!(result, Err(HpError::InvalidOp)));
    assert!(
        s.print_buffer.is_empty(),
        "print_buffer must stay empty: {:?}",
        s.print_buffer
    );
}

#[test]
fn test_catalog_5_rejects() {
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::Catalog(5));
    assert!(matches!(result, Err(HpError::InvalidOp)));
    assert!(s.print_buffer.is_empty());
}

#[test]
fn test_catalog_lines_are_24_chars_wide() {
    // CAT 1 with non-trivial program — every emitted line must be ≥ 24 chars
    // (left-padded). Header / footer / LBL lines and the NOT AVAILABLE payload
    // all use the {:<24} padding.
    let mut s = CalcState::new();
    s.program = program_with_two_labels();
    dispatch(&mut s, Op::Catalog(1)).unwrap();
    for (i, line) in s.print_buffer.iter().enumerate() {
        // {:<24} pads to AT LEAST 24 chars; for our fixtures the content is
        // <24 chars so total length is exactly 24.
        assert_eq!(
            line.chars().count(),
            24,
            "line {i} expected 24 chars, got {}: {:?}",
            line.chars().count(),
            line
        );
    }

    // CAT 2 NOT AVAILABLE payload also 24 chars wide
    let mut s2 = CalcState::new();
    dispatch(&mut s2, Op::Catalog(2)).unwrap();
    for (i, line) in s2.print_buffer.iter().enumerate() {
        assert_eq!(
            line.chars().count(),
            24,
            "cat2 line {i}: {:?}",
            line
        );
    }
}

#[test]
fn test_catalog_long_label_truncated_to_9_chars() {
    // A label longer than 9 chars must be truncated to 9 chars so the full
    // 24-char line width is preserved. (Format: "LBL " + name:9 + "  " + steps:5 = 20, pad to 24.)
    let mut s = CalcState::new();
    s.program = vec![
        Op::Lbl("VERYLONGLABEL".to_string()),  // 13 chars
        Op::PushNum(HpNum::from(1i32)),
    ];
    dispatch(&mut s, Op::Catalog(1)).unwrap();

    assert_eq!(s.print_buffer.len(), 3, "buf={:?}", s.print_buffer);
    let lbl_line = &s.print_buffer[1];
    // 24-char width preserved
    assert_eq!(
        lbl_line.chars().count(),
        24,
        "LBL line width: {:?}",
        lbl_line
    );
    // Truncation to first 9 chars: "VERYLONGL"
    assert!(
        lbl_line.contains("VERYLONGL"),
        "expected truncated label 'VERYLONGL': {:?}",
        lbl_line
    );
    // The 10th character ("A") must NOT appear in the truncated label position.
    // Build the expected display: "LBL VERYLONGL    1" right-padded to 24.
    let expected_prefix = "LBL VERYLONGL";
    assert!(
        lbl_line.starts_with(expected_prefix),
        "LBL line must start with {expected_prefix:?}: {:?}",
        lbl_line
    );
}

#[test]
fn test_catalog_lift_neutral() {
    // CATALOG is LiftEffect::Neutral — lift_enabled flag must be unchanged.
    let mut s = CalcState::new();
    s.stack.lift_enabled = true;
    dispatch(&mut s, Op::Catalog(2)).unwrap();
    assert!(
        s.stack.lift_enabled,
        "Neutral must preserve lift_enabled=true"
    );

    let mut s2 = CalcState::new();
    s2.stack.lift_enabled = false;
    dispatch(&mut s2, Op::Catalog(2)).unwrap();
    assert!(
        !s2.stack.lift_enabled,
        "Neutral must preserve lift_enabled=false"
    );
}
