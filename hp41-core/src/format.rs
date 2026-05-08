//! HP-41 display number formatting.
//!
//! Produces HP-41-style display strings for FIX, SCI, and ENG modes.
//!
//! HP-41 display is 12 characters wide.
//! FIX overflow: when integer part exceeds display capacity, falls back to SCI 9.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use std::str::FromStr;

use crate::num::HpNum;
use crate::state::DisplayMode;

/// Format an HpNum according to the current display mode.
/// Returns the HP-41-style display string.
pub fn format_hpnum(n: &HpNum, mode: &DisplayMode) -> String {
    let d = n.inner();
    match mode {
        DisplayMode::Fix(digits) => format_fix(d, *digits as usize),
        DisplayMode::Sci(digits) => format_sci(d, *digits as usize),
        DisplayMode::Eng(digits) => format_eng(d, *digits as usize),
    }
}

/// Format the ALPHA register for display.
/// HP-41 display is 12 characters; truncate if longer.
pub fn format_alpha(reg: &str) -> String {
    reg.chars().take(12).collect()
}

// ── FIX formatting ────────────────────────────────────────────────────────

fn format_fix(d: Decimal, digits: usize) -> String {
    if d.is_zero() {
        if digits == 0 {
            return "0.".to_string();
        }
        return format!("0.{:0>width$}", "", width = digits);
    }

    // FIX overflow check: if |integer part| >= 10^(10 - digits), fall back to SCI 9.
    // This matches HP-41 hardware: can't display more than 10 significant digits in FIX.
    let abs_d = d.abs();
    let overflow_exp = 10_usize.saturating_sub(digits);
    let overflow_threshold = decimal_pow10(overflow_exp as i32);

    if abs_d >= overflow_threshold {
        return format_sci(d, 9);
    }

    // Round to `digits` decimal places with HP-41's round-half-away-from-zero strategy.
    let rounded = d.round_dp_with_strategy(digits as u32, RoundingStrategy::MidpointAwayFromZero);

    // Rust's format! with precision handles FIX formatting including trailing zeros.
    let s = format!("{:.prec$}", rounded, prec = digits);

    // HP-41 FIX 0 shows trailing decimal point: "42."
    if digits == 0 {
        format!("{s}.")
    } else {
        s
    }
}

// ── SCI formatting ────────────────────────────────────────────────────────

fn format_sci(d: Decimal, digits: usize) -> String {
    if d.is_zero() {
        if digits == 0 {
            return "0.E 00".to_string();
        }
        let frac = format!(".{:0>width$}", "", width = digits);
        return format!("0{frac}E 00");
    }

    let is_negative = d.is_sign_negative();
    let abs_d = d.abs();

    // Compute base-10 exponent: floor(log10(|d|))
    let sci_exp = compute_sci_exp(abs_d);

    // Compute mantissa = abs_d / 10^sci_exp = abs_d * 10^(-sci_exp)
    let mantissa = scale_decimal(abs_d, -sci_exp);

    // Round mantissa to `digits` decimal places with HP-41 rounding
    let mut mantissa_rounded =
        mantissa.round_dp_with_strategy(digits as u32, RoundingStrategy::MidpointAwayFromZero);

    // Carry: rounding can push mantissa from e.g. 9.9995 to 10.000
    let mut sci_exp = sci_exp;
    if mantissa_rounded >= Decimal::from(10) {
        mantissa_rounded /= Decimal::from(10);
        sci_exp += 1;
    }

    // Format mantissa to `digits` decimal places
    let mantissa_str = format!("{:.prec$}", mantissa_rounded, prec = digits);

    // HP-41 SCI 0 format: "1.E 02" — need to keep the decimal point
    let mantissa_with_point = ensure_decimal_point(mantissa_str);

    let sign = if is_negative { "-" } else { "" };
    assemble_sci(&format!("{sign}{mantissa_with_point}"), sci_exp)
}

// ── ENG formatting ────────────────────────────────────────────────────────

fn format_eng(d: Decimal, digits: usize) -> String {
    if d.is_zero() {
        if digits == 0 {
            return "0.E 00".to_string();
        }
        let frac = format!(".{:0>width$}", "", width = digits);
        return format!("0{frac}E 00");
    }

    let is_negative = d.is_sign_negative();
    let abs_d = d.abs();

    // Compute SCI exponent
    let sci_exp = compute_sci_exp(abs_d);

    // Clamp to nearest multiple of 3 (round towards negative infinity)
    let eng_exp = floor_to_multiple_of_3(sci_exp);

    // Mantissa = abs_d * 10^(-eng_exp) — will have 1-3 digits before the decimal point
    let mantissa = scale_decimal(abs_d, -eng_exp);

    // Round mantissa to `digits` decimal places with HP-41 rounding
    let mut mantissa_rounded =
        mantissa.round_dp_with_strategy(digits as u32, RoundingStrategy::MidpointAwayFromZero);

    // Carry: rounding can push the mantissa past the current power-of-10 boundary.
    // E.g. mantissa 999.9995 in ENG(3) rounds to 1000.000 → must become 1.000E+3 higher.
    let mut eng_exp = eng_exp;
    let carry_threshold = decimal_pow10(sci_exp - eng_exp + 1);
    if mantissa_rounded >= carry_threshold {
        let new_eng_exp = floor_to_multiple_of_3(sci_exp + 1);
        mantissa_rounded = scale_decimal(mantissa_rounded, eng_exp - new_eng_exp);
        eng_exp = new_eng_exp;
    }

    // Format mantissa to `digits` decimal places
    let mantissa_str = format!("{:.prec$}", mantissa_rounded, prec = digits);

    // ENG 0 format also needs decimal point
    let mantissa_with_point = ensure_decimal_point(mantissa_str);

    let sign = if is_negative { "-" } else { "" };
    assemble_sci(&format!("{sign}{mantissa_with_point}"), eng_exp)
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Compute the base-10 exponent for SCI notation: floor(log10(|d|)).
/// d must be positive and non-zero.
fn compute_sci_exp(abs_d: Decimal) -> i32 {
    // All valid HpNum values fit in f64 (max ~7.9e28 vs f64 max ~1.8e308).
    let f = abs_d.to_f64().expect("HpNum is always within f64 range");
    f.log10().floor() as i32
}

/// Compute 10^exp as a Decimal without E notation (which rust_decimal from_str rejects).
/// For positive exp: "1" + "0" * exp (e.g., exp=8 → "100000000")
/// For negative exp: "0." + "0" * (|exp|-1) + "1" (e.g., exp=-8 → "0.00000001")
/// For zero: Decimal::ONE
fn decimal_pow10(exp: i32) -> Decimal {
    if exp == 0 {
        return Decimal::ONE;
    }
    let s = if exp > 0 {
        "1".to_string() + &"0".repeat(exp as usize)
    } else {
        let abs_exp = (-exp) as usize;
        "0.".to_string() + &"0".repeat(abs_exp - 1) + "1"
    };
    Decimal::from_str(&s).expect("string built from known-valid exp always parses")
}

/// Scale a Decimal by 10^exp_shift: returns d * 10^exp_shift.
/// Uses decimal string construction to avoid E notation.
fn scale_decimal(d: Decimal, exp_shift: i32) -> Decimal {
    if exp_shift == 0 {
        return d;
    }
    let scale = decimal_pow10(exp_shift);
    d.checked_mul(scale)
        .expect("scale_decimal: mantissa and bounded scale must not overflow")
}

/// Ensure that a formatted number string has a decimal point.
/// Used for SCI/ENG digit=0 mode: "1" → "1." so the E separator is correct.
fn ensure_decimal_point(s: String) -> String {
    if s.contains('.') {
        s
    } else {
        format!("{s}.")
    }
}

/// Assemble a SCI/ENG string from mantissa string and exponent.
/// HP-41 format: space before positive exponent, minus sign before negative.
fn assemble_sci(mantissa: &str, exp: i32) -> String {
    if exp >= 0 {
        format!("{mantissa}E {:02}", exp)
    } else {
        format!("{mantissa}E-{:02}", -exp)
    }
}

/// Floor-divide exponent to the nearest multiple of 3 (towards negative infinity).
/// Examples: 4 → 3, 3 → 3, 1 → 0, -1 → -3, -3 → -3, -4 → -6
fn floor_to_multiple_of_3(exp: i32) -> i32 {
    exp.div_euclid(3) * 3
}
