//! Integration tests for MATH-03: FIX/SCI/ENG display mode formatting.
//! These tests require hp41-core::format to exist (created in Plan 04).
//! Until Plan 04 runs, these tests will fail to compile.

use hp41_core::HpNum;
use hp41_core::format::format_hpnum;
use hp41_core::state::DisplayMode;
use rust_decimal::Decimal;
use std::str::FromStr;

fn num(s: &str) -> HpNum {
    HpNum::from(Decimal::from_str(s).expect("valid decimal in test"))
}

// ── FIX mode ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_fix4_trailing_zeros() {
    // HP-41 FIX 4: integer 1 shows as "1.0000" (trailing zeros REQUIRED)
    assert_eq!(format_hpnum(&num("1"), &DisplayMode::Fix(4)), "1.0000");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_fix4_pi_rounded() {
    // FIX 4 of 3.14159265359 → "3.1416" (rounds up at digit 4)
    assert_eq!(format_hpnum(&num("3.14159265359"), &DisplayMode::Fix(4)), "3.1416");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_fix0_integer() {
    assert_eq!(format_hpnum(&num("42"), &DisplayMode::Fix(0)), "42.");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_fix4_negative() {
    assert_eq!(format_hpnum(&num("-1.5"), &DisplayMode::Fix(4)), "-1.5000");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_fix4_zero() {
    assert_eq!(format_hpnum(&HpNum::zero(), &DisplayMode::Fix(4)), "0.0000");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_fix4_overflow_to_sci() {
    // When integer part exceeds display width, fall back to SCI format.
    // FIX 4 with a 10^15 value: integer part is too wide for 12-char display.
    let n = num("1E15");
    let result = format_hpnum(&n, &DisplayMode::Fix(4));
    // Must contain 'E' — the HP-41 overflow to SCI indicator
    assert!(result.contains('E'), "FIX overflow must fall back to SCI: got '{result}'");
}

// ── SCI mode ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_sci4_speed_of_light() {
    // 299792500 in SCI 4 = "2.9979E 08" (HP-41 format: space before positive exponent)
    assert_eq!(format_hpnum(&num("299792500"), &DisplayMode::Sci(4)), "2.9979E 08");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_sci4_small_number() {
    // 0.00001234 in SCI 4 = "1.2340E-05"
    assert_eq!(format_hpnum(&num("0.00001234"), &DisplayMode::Sci(4)), "1.2340E-05");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_sci0_single_digit() {
    // SCI 0: one significant digit; 123 → "1.E 02"
    assert_eq!(format_hpnum(&num("123"), &DisplayMode::Sci(0)), "1.E 02");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_sci4_zero() {
    assert_eq!(format_hpnum(&HpNum::zero(), &DisplayMode::Sci(4)), "0.0000E 00");
}

// ── ENG mode ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_eng3_12345() {
    // ENG 3 of 12345.678 → "12.346E 03" (exponent is multiple of 3; mantissa 2 digits before decimal)
    assert_eq!(format_hpnum(&num("12345.678"), &DisplayMode::Eng(3)), "12.346E 03");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_eng3_small_number() {
    // ENG 3 of 0.001234 → "1.234E-03"
    assert_eq!(format_hpnum(&num("0.001234"), &DisplayMode::Eng(3)), "1.234E-03");
}

#[test]
#[ignore = "format_hpnum stub — implemented in Wave 3 plan 02-05"]
fn test_eng3_million() {
    // ENG 3 of 1000000 → "1.000E 06"
    assert_eq!(format_hpnum(&num("1000000"), &DisplayMode::Eng(3)), "1.000E 06");
}
