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

/// Round an `HpNum` to the precision of the current `DisplayMode` (D-01/D-02/D-03).
///
/// This is the **single source of truth** for "round-to-display-precision" semantics,
/// shared between the display path (`format_hpnum`) and the value-mutation path
/// (`op_rnd` in `ops/math.rs`). Both consumers route through this helper so RND always
/// matches what FIX/SCI/ENG would have displayed.
///
/// Semantics per D-01/D-02/D-03:
/// - **Fix(n):** round inner Decimal to `n` decimal places (round-half-away-from-zero).
/// - **Sci(n):** round to `n + 1` significant digits (the SCI mantissa carries 1 leading
///   digit + `n` fractional digits).
/// - **Eng(n):** round to `n + 1` significant digits with the engineering exponent
///   constraint (multiples of 3); mantissa keeps 1–3 digits before the decimal point.
/// - Zero: returns `HpNum::zero()` regardless of mode.
///
/// The final value is wrapped via `HpNum::rounded`, which re-applies the 10-sig-digit
/// gate as an idempotent safety pass.
///
/// **Phase 20 (D-01/D-02):** consumed by `op_rnd` in `hp41-core/src/ops/math.rs` so
/// the value-mutation RND keystroke produces the same number that FIX/SCI/ENG would
/// have displayed.
pub fn round_to_display_precision(n: &HpNum, mode: &DisplayMode) -> HpNum {
    if n.is_zero() {
        return HpNum::zero();
    }
    let d = n.inner();
    let rounded = match mode {
        DisplayMode::Fix(digits) => {
            d.round_dp_with_strategy(*digits as u32, RoundingStrategy::MidpointAwayFromZero)
        }
        DisplayMode::Sci(digits) => d
            .round_sf_with_strategy((*digits as u32) + 1, RoundingStrategy::MidpointAwayFromZero)
            .expect("round_sf_with_strategy(<= 10) cannot fail for finite Decimal"),
        DisplayMode::Eng(digits) => round_eng(d, *digits as usize),
    };
    HpNum::rounded(rounded)
}

/// Engineering-mode rounding: mantissa keeps `digits` decimal places after the
/// engineering normalization (exponent constrained to multiples of 3).
///
/// Mirrors the internal rounding sequence in `format_eng` so RND in ENG mode produces
/// the same value the display would show. Carry handling matches `format_eng` (a
/// mantissa that grows past the boundary after rounding bumps the engineering exponent
/// to the next multiple of 3).
fn round_eng(d: Decimal, digits: usize) -> Decimal {
    let is_negative = d.is_sign_negative();
    let abs_d = d.abs();

    let sci_exp = compute_sci_exp(abs_d);
    let eng_exp = floor_to_multiple_of_3(sci_exp);

    // Bring mantissa into engineering form: mantissa = abs_d * 10^(-eng_exp).
    let mantissa = scale_decimal(abs_d, -eng_exp);

    let mut mantissa_rounded =
        mantissa.round_dp_with_strategy(digits as u32, RoundingStrategy::MidpointAwayFromZero);

    // Carry: mantissa may cross the next power-of-10 boundary after rounding.
    let mut eng_exp = eng_exp;
    let carry_threshold = decimal_pow10(sci_exp - eng_exp + 1);
    if mantissa_rounded >= carry_threshold {
        let new_eng_exp = floor_to_multiple_of_3(sci_exp + 1);
        mantissa_rounded = scale_decimal(mantissa_rounded, eng_exp - new_eng_exp);
        eng_exp = new_eng_exp;
    }

    // Re-scale mantissa back to absolute magnitude.
    let abs_rounded = scale_decimal(mantissa_rounded, eng_exp);
    if is_negative {
        -abs_rounded
    } else {
        abs_rounded
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
    let s = format!("{rounded:.digits$}");

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
    let mantissa_str = format!("{mantissa_rounded:.digits$}");

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
    let mantissa_str = format!("{mantissa_rounded:.digits$}");

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
        format!("{mantissa}E {exp:02}")
    } else {
        format!("{mantissa}E-{:02}", -exp)
    }
}

/// Floor-divide exponent to the nearest multiple of 3 (towards negative infinity).
/// Examples: 4 → 3, 3 → 3, 1 → 0, -1 → -3, -3 → -3, -4 → -6
fn floor_to_multiple_of_3(exp: i32) -> i32 {
    exp.div_euclid(3) * 3
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    //! Inline smoke tests for `round_to_display_precision` (D-01/D-02/D-03).
    //! Integration coverage lives in `hp41-core/tests/phase20_math.rs`.
    use super::*;

    fn hp(s: &str) -> HpNum {
        HpNum::from(Decimal::from_str(s).expect("test literal must parse"))
    }

    #[test]
    fn round_fix1_negative_5_65_is_minus_5_7() {
        // Round-half-away-from-zero: -5.65 → -5.7 at FIX(1).
        let out = round_to_display_precision(&hp("-5.65"), &DisplayMode::Fix(1));
        assert_eq!(out.inner(), Decimal::from_str("-5.7").unwrap());
    }

    #[test]
    fn round_fix0_negative_5_7_is_minus_6() {
        let out = round_to_display_precision(&hp("-5.7"), &DisplayMode::Fix(0));
        assert_eq!(out.inner(), Decimal::from(-6));
    }

    #[test]
    fn round_sci3_carry_9_9995_is_10() {
        // SCI(3): keep 4 sig digits — 9.9995 → 10 (mantissa carry at the digit-4 boundary).
        let out = round_to_display_precision(&hp("9.9995"), &DisplayMode::Sci(3));
        assert_eq!(out.inner(), Decimal::from(10));
    }

    #[test]
    fn round_zero_returns_zero() {
        for mode in [
            DisplayMode::Fix(4),
            DisplayMode::Sci(4),
            DisplayMode::Eng(3),
        ] {
            assert_eq!(
                round_to_display_precision(&HpNum::zero(), &mode).inner(),
                Decimal::ZERO,
            );
        }
    }
}
