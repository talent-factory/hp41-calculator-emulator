//! Phase 6 HMS/H time-angle conversion operations: HMS→, →HMS, HMS+, HMS−.
//!
//! H.MMSS format: integer part = hours, decimal part = MMSS (2-digit minutes + 2-digit seconds).
//! Field extraction uses string-split at the decimal point — same pattern as parse_counter()
//! in program.rs (ADR-001: never use floor()/fmod() on f64 for field extraction).
//!
//! Validation: minutes >= 60 or seconds >= 60 → HpError::InvalidInput (D-06).
//! Negative values: sign applies to whole value; work on abs() internally (D-08).

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{binary_result, unary_result};
use crate::state::CalcState;

/// Parse H.MMSS format into (hours, minutes, seconds, is_negative).
///
/// CRITICAL: fraction is right-padded to 4 chars with {:0<4} (left-align = right-pad).
/// "30" → "3000" means 30 minutes, 00 seconds (NOT 03 minutes, 00 seconds).
/// This matches parse_counter() pattern but with 4-char (MMSS) instead of 5-char (FFFDD).
fn parse_hms(n: &HpNum) -> Result<(i64, i64, i64, bool), HpError> {
    let is_neg = n.inner().is_sign_negative();
    let abs_inner = n.inner().abs();
    let s = abs_inner.to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let hours: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // CRITICAL: left-align ({:0<4}) = right-pad with zeros
    let padded = format!("{:0<4}", frac_part);
    let padded = if padded.len() > 4 {
        padded[..4].to_string()
    } else {
        padded
    };
    let minutes: i64 = padded[..2].parse().map_err(|_| HpError::InvalidOp)?;
    let seconds: i64 = padded[2..4].parse().map_err(|_| HpError::InvalidOp)?;
    Ok((hours, minutes, seconds, is_neg))
}

/// Validate parsed HMS fields. Returns HpError::InvalidInput if minutes >= 60 or seconds >= 60.
fn validate_hms(minutes: i64, seconds: i64) -> Result<(), HpError> {
    if minutes >= 60 || seconds >= 60 {
        Err(HpError::InvalidInput)
    } else {
        Ok(())
    }
}

/// Convert H.MMSS fields to decimal hours using rust_decimal arithmetic.
/// hours + minutes/60 + seconds/3600. No f64 intermediate.
fn hms_fields_to_decimal(hours: i64, minutes: i64, seconds: i64) -> Result<HpNum, HpError> {
    let h = HpNum::from(hours as i32);
    let m = HpNum::from(minutes as i32).checked_div(&HpNum::from(60i32))?;
    let s = HpNum::from(seconds as i32).checked_div(&HpNum::from(3600i32))?;
    h.checked_add(&m)?.checked_add(&s)
}

/// Convert decimal hours to H.MMSS components using rust_decimal truncation.
/// Uses to_i64() via ToPrimitive (no floor() on f64).
fn decimal_to_hms_fields(decimal_hours: &Decimal) -> Result<(i64, i64, i64), HpError> {
    // total seconds = decimal_hours * 3600
    let d3600 = Decimal::from(3600i32);
    let total_secs_dec = decimal_hours * d3600;
    // Truncate toward zero to get integer total seconds
    let total_secs_trunc = total_secs_dec.trunc();
    let total_secs_i64 = total_secs_trunc.to_i64().ok_or(HpError::Overflow)?;

    let hours = total_secs_i64 / 3600;
    let rem_s = total_secs_i64 % 3600;
    let minutes = rem_s / 60;
    let seconds = rem_s % 60;
    Ok((hours, minutes, seconds))
}

/// Reconstruct H.MMSS HpNum from integer fields.
/// Uses integer formatting: format!("{}.{:02}{:02}", h, m, s) — never float formatting.
fn build_hms(hours: i64, minutes: i64, seconds: i64) -> Result<HpNum, HpError> {
    let s = format!("{}.{:02}{:02}", hours, minutes, seconds);
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::rounded(d))
}

/// HMS→: convert H.MMSS in X to decimal hours. Unary op — saves LASTX. LiftEffect: Enable.
/// HpError::InvalidInput if minutes >= 60 or seconds >= 60 (D-06).
pub fn op_hms_to_h(state: &mut CalcState) -> Result<(), HpError> {
    let (hours, minutes, seconds, is_neg) = parse_hms(&state.stack.x)?;
    validate_hms(minutes, seconds)?;
    let mut result = hms_fields_to_decimal(hours, minutes, seconds)?;
    if is_neg {
        result = HpNum::rounded(-result.inner());
    }
    unary_result(state, result);
    Ok(())
}

/// →HMS: convert decimal hours in X to H.MMSS format. Unary op — saves LASTX. LiftEffect: Enable.
pub fn op_h_to_hms(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.inner();
    let is_neg = x.is_sign_negative();
    let abs_x = x.abs();
    let (hours, minutes, seconds) = decimal_to_hms_fields(&abs_x)?;
    let mut result = build_hms(hours, minutes, seconds)?;
    if is_neg {
        result = HpNum::rounded(-result.inner());
    }
    unary_result(state, result);
    Ok(())
}

/// Convert HMS integer fields + sign to signed total seconds (integer arithmetic, no float).
/// Used by HMS+ and HMS− to avoid f64 precision loss in the field-to-decimal round trip.
fn hms_to_total_secs(hours: i64, minutes: i64, seconds: i64, is_neg: bool) -> i64 {
    let total = hours * 3600 + minutes * 60 + seconds;
    if is_neg {
        -total
    } else {
        total
    }
}

/// Convert signed total seconds back to (hours, minutes, seconds, is_negative) fields.
fn total_secs_to_hms_fields(total_secs: i64) -> (i64, i64, i64, bool) {
    let is_neg = total_secs < 0;
    let abs = total_secs.abs();
    let hours = abs / 3600;
    let rem = abs % 3600;
    let minutes = rem / 60;
    let seconds = rem % 60;
    (hours, minutes, seconds, is_neg)
}

/// HMS+: add Y + X in H.MMSS format with base-60 carry. Binary op — drops Y, saves LASTX.
/// Both operands validated for minutes/seconds < 60. LiftEffect: Enable.
pub fn op_hms_add(state: &mut CalcState) -> Result<(), HpError> {
    let (yh, ym, ys, y_neg) = parse_hms(&state.stack.y)?;
    validate_hms(ym, ys)?;
    let (xh, xm, xs, x_neg) = parse_hms(&state.stack.x)?;
    validate_hms(xm, xs)?;

    // Work entirely in integer seconds — no fractional arithmetic, no precision loss
    let y_secs = hms_to_total_secs(yh, ym, ys, y_neg);
    let x_secs = hms_to_total_secs(xh, xm, xs, x_neg);
    let sum_secs = y_secs + x_secs;

    let (rh, rm, rs, is_neg) = total_secs_to_hms_fields(sum_secs);
    let mut result = build_hms(rh, rm, rs)?;
    if is_neg {
        result = HpNum::rounded(-result.inner());
    }

    binary_result(state, result);
    Ok(())
}

/// HMS−: subtract Y − X in H.MMSS format with base-60 borrow. Binary op — drops Y, saves LASTX.
/// Both operands validated for minutes/seconds < 60. LiftEffect: Enable.
pub fn op_hms_sub(state: &mut CalcState) -> Result<(), HpError> {
    let (yh, ym, ys, y_neg) = parse_hms(&state.stack.y)?;
    validate_hms(ym, ys)?;
    let (xh, xm, xs, x_neg) = parse_hms(&state.stack.x)?;
    validate_hms(xm, xs)?;

    // Work entirely in integer seconds — no fractional arithmetic, no precision loss
    let y_secs = hms_to_total_secs(yh, ym, ys, y_neg);
    let x_secs = hms_to_total_secs(xh, xm, xs, x_neg);
    let diff_secs = y_secs - x_secs;

    let (rh, rm, rs, is_neg) = total_secs_to_hms_fields(diff_secs);
    let mut result = build_hms(rh, rm, rs)?;
    if is_neg {
        result = HpNum::rounded(-result.inner());
    }

    binary_result(state, result);
    Ok(())
}
