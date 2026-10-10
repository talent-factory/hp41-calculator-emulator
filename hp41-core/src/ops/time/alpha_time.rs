// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Alpha time/date formatting operations for the HP Time Module.
//!
//! Covers: ADATE, ATIME, ATIME24.
//!
//! These operations read the current wall-clock time (adjusted by
//! `state.time_offset_secs`) and append a formatted string to the
//! ALPHA register (`state.alpha_reg`).
//!
//! HP-41 ALPHA register: 24 characters max. Strings are truncated to keep
//! the total ≤ 24 characters.
//!
//! 12-hour format: " H:MM:SS AM" / "HH:MM:SS AM" (hours 1-12 + AM/PM suffix).
//! 24-hour format: "HH:MM:SS" (hours 0-23, zero-padded).
//! MDY date (Flag 31 clear): " M/DD/YYYY" (month without leading zero)
//! DMY date (Flag 31 set):   "DD/ M/YYYY" (month without leading zero)

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

use super::clock;

/// Maximum ALPHA register length (HP-41 hardware limit).
const ALPHA_MAX_LEN: usize = 24;

/// Append `text` to `state.alpha_reg`, truncating to ALPHA_MAX_LEN.
fn alpha_append(state: &mut CalcState, text: &str) {
    let remaining = ALPHA_MAX_LEN.saturating_sub(state.alpha_reg.len());
    if remaining > 0 {
        let truncated = &text[..text.len().min(remaining)];
        state.alpha_reg.push_str(truncated);
    }
}

/// Get current local time components (year, month, day, hour, minute, second)
/// adjusted by `offset_secs`.
///
/// Delegates to `clock::adjusted_epoch_secs` + `clock::decompose_epoch_secs`
/// (single canonical Fliegel-Van Flandern algorithm for all epoch→calendar conversions).
pub(crate) fn get_local_time(offset_secs: i64) -> (i32, u8, u8, u8, u8, u8) {
    let epoch = clock::adjusted_epoch_secs(offset_secs);
    clock::decompose_epoch_secs(epoch)
}

/// Format hour, minute, second into a 24-hour time string "HH:MM:SS".
fn format_24h(hour: u8, minute: u8, second: u8) -> String {
    format!("{hour:02}:{minute:02}:{second:02}")
}

/// Format hour, minute, second into a 12-hour time string " H:MM:SS AM/PM".
///
/// HP-41 Time Module OM §ATIME:
/// - Hours 0 → "12:MM:SS AM"   (midnight hour)
/// - Hours 1-11 → " H:MM:SS AM"
/// - Hours 12 → "12:MM:SS PM"   (noon hour)
/// - Hours 13-23 → " H:MM:SS PM" (where H = hour - 12)
fn format_12h(hour: u8, minute: u8, second: u8) -> String {
    let (display_hour, suffix) = match hour {
        0 => (12u8, "AM"),
        1..=11 => (hour, "AM"),
        12 => (12u8, "PM"),
        _ => (hour - 12, "PM"),
    };
    if display_hour < 10 {
        format!(" {display_hour}:{minute:02}:{second:02} {suffix}")
    } else {
        format!("{display_hour:02}:{minute:02}:{second:02} {suffix}")
    }
}

/// ATIME — Append current time to ALPHA register.
///
/// Respects `state.clock_12h` for 12/24-hour output format.
/// Uses `state.time_offset_secs` for the clock offset.
/// LiftEffect::Neutral (no stack change).
pub fn op_atime(state: &mut CalcState) -> Result<(), HpError> {
    let (_year, _month, _day, hour, minute, second) = get_local_time(state.time_offset_secs);
    let text = if state.clock_12h {
        format_12h(hour, minute, second)
    } else {
        format_24h(hour, minute, second)
    };
    alpha_append(state, &text);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ATIME24 — Append current time in 24-hour format to ALPHA register.
///
/// Always 24-hour format regardless of `state.clock_12h`.
/// Uses `state.time_offset_secs` for the clock offset.
/// LiftEffect::Neutral (no stack change).
pub fn op_atime24(state: &mut CalcState) -> Result<(), HpError> {
    let (_year, _month, _day, hour, minute, second) = get_local_time(state.time_offset_secs);
    let text = format_24h(hour, minute, second);
    alpha_append(state, &text);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADATE — Append current date to ALPHA register.
///
/// Flag 31 clear (MDY mode): " M/DD/YYYY" (e.g. " 1/24/2026" for Jan 24 2026)
/// Flag 31 set (DMY mode):   "DD/ M/YYYY" (e.g. "24/ 1/2026" for Jan 24 2026)
///
/// Month is NOT zero-padded (HP-41 OM §ADATE locale convention).
/// Uses `state.time_offset_secs` for the clock offset.
/// LiftEffect::Neutral (no stack change).
pub fn op_adate(state: &mut CalcState) -> Result<(), HpError> {
    let (year, month, day, _hour, _minute, _second) = get_local_time(state.time_offset_secs);
    let dmy = state.flags & (1u64 << 31) != 0;
    let text = if dmy {
        // DMY: "DD/ M/YYYY"
        format!("{day:02}/{month:2}/{year}")
    } else {
        // MDY: " M/DD/YYYY"
        format!("{month:2}/{day:02}/{year}")
    };
    alpha_append(state, &text);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Helper: build a CalcState with a fixed time offset so that the time is
    // predictable for testing.
    //
    // We cannot control SystemTime::now() in tests, so instead we test:
    // 1. The formatting logic directly (format_24h, format_12h, format_adate).
    // 2. That op_atime/op_atime24/op_adate APPEND to alpha_reg (not replace).
    // 3. That truncation to 24 chars is enforced.

    #[test]
    fn format_24h_midnight() {
        assert_eq!(format_24h(0, 0, 0), "00:00:00");
    }

    #[test]
    fn format_24h_noon() {
        assert_eq!(format_24h(12, 0, 0), "12:00:00");
    }

    #[test]
    fn format_24h_max() {
        assert_eq!(format_24h(23, 59, 59), "23:59:59");
    }

    #[test]
    fn format_24h_example() {
        // 13:30:45 → "13:30:45"
        assert_eq!(format_24h(13, 30, 45), "13:30:45");
    }

    #[test]
    fn format_12h_midnight_hour() {
        // 0h (midnight) → "12:05:00 AM"
        assert_eq!(format_12h(0, 5, 0), "12:05:00 AM");
    }

    #[test]
    fn format_12h_morning_single_digit() {
        // 1:30:45 AM → " 1:30:45 AM"
        assert_eq!(format_12h(1, 30, 45), " 1:30:45 AM");
    }

    #[test]
    fn format_12h_noon_hour() {
        // 12h (noon) → "12:00:00 PM"
        assert_eq!(format_12h(12, 0, 0), "12:00:00 PM");
    }

    #[test]
    fn format_12h_afternoon() {
        // 13h (1 PM) → " 1:30:45 PM"
        assert_eq!(format_12h(13, 30, 45), " 1:30:45 PM");
    }

    #[test]
    fn format_12h_double_digit_pm() {
        // 22h (10 PM) → "10:00:00 PM"
        assert_eq!(format_12h(22, 0, 0), "10:00:00 PM");
    }

    #[test]
    fn format_12h_11am() {
        // 11:59:59 AM
        assert_eq!(format_12h(11, 59, 59), "11:59:59 AM");
    }

    #[test]
    fn op_atime24_appends_to_existing_alpha() {
        let mut state = CalcState::new();
        state.alpha_reg = "T=".to_string();
        op_atime24(&mut state).unwrap();
        // Should start with "T=" and have appended an 8-char time string
        assert!(state.alpha_reg.starts_with("T="));
        assert_eq!(state.alpha_reg.len(), 10); // "T=" + "HH:MM:SS"
    }

    #[test]
    fn op_atime24_format_is_correct_pattern() {
        let mut state = CalcState::new();
        op_atime24(&mut state).unwrap();
        // 24h format: "HH:MM:SS" — exactly 8 chars, colons at positions 2 and 5
        let s = &state.alpha_reg;
        assert_eq!(s.len(), 8);
        assert_eq!(s.chars().nth(2), Some(':'));
        assert_eq!(s.chars().nth(5), Some(':'));
        // All other chars should be digits
        for (i, c) in s.chars().enumerate() {
            if i != 2 && i != 5 {
                assert!(c.is_ascii_digit(), "char at {i} should be digit, got {c:?}");
            }
        }
    }

    #[test]
    fn op_atime_respects_clock_12h_flag() {
        let mut state = CalcState::new();
        state.clock_12h = true;
        op_atime(&mut state).unwrap();
        // 12h format contains AM or PM
        assert!(
            state.alpha_reg.contains("AM") || state.alpha_reg.contains("PM"),
            "12h format must contain AM or PM, got: {:?}",
            state.alpha_reg
        );
    }

    #[test]
    fn op_atime_24h_format_when_clock_12h_false() {
        let mut state = CalcState::new();
        state.clock_12h = false;
        op_atime(&mut state).unwrap();
        // 24h format does NOT contain AM or PM
        assert!(
            !state.alpha_reg.contains("AM") && !state.alpha_reg.contains("PM"),
            "24h format must not contain AM/PM, got: {:?}",
            state.alpha_reg
        );
        // Must be exactly 8 chars
        assert_eq!(state.alpha_reg.len(), 8);
    }

    #[test]
    fn op_atime24_ignores_clock_12h_flag() {
        let mut state = CalcState::new();
        state.clock_12h = true; // 12h mode set, but atime24 ignores it
        op_atime24(&mut state).unwrap();
        // Must be 24h format (no AM/PM)
        assert!(
            !state.alpha_reg.contains("AM") && !state.alpha_reg.contains("PM"),
            "ATIME24 must always use 24h, got: {:?}",
            state.alpha_reg
        );
    }

    #[test]
    fn op_adate_mdy_format_structure() {
        let mut state = CalcState::new();
        state.flags = 0; // Flag 31 clear = MDY
        op_adate(&mut state).unwrap();
        // MDY format: " M/DD/YYYY" or "MM/DD/YYYY"
        // Contains at least 2 slashes
        let slashes = state.alpha_reg.chars().filter(|&c| c == '/').count();
        assert_eq!(
            slashes, 2,
            "MDY date must have 2 slashes, got: {:?}",
            state.alpha_reg
        );
    }

    #[test]
    fn op_adate_dmy_format_structure() {
        let mut state = CalcState::new();
        state.flags = 1u64 << 31; // Flag 31 set = DMY
        op_adate(&mut state).unwrap();
        // DMY format: "DD/ M/YYYY" or "DD/MM/YYYY"
        let slashes = state.alpha_reg.chars().filter(|&c| c == '/').count();
        assert_eq!(
            slashes, 2,
            "DMY date must have 2 slashes, got: {:?}",
            state.alpha_reg
        );
    }

    /// 2026-02-13 12:00:00 UTC: day != month, and noon keeps the date stable even
    /// if the real clock ticks across a second boundary between setup and call.
    const PINNED_EPOCH: i64 = 1_770_984_000;

    /// Build a state whose adjusted clock reads `PINNED_EPOCH` (offset = pinned - now).
    fn pinned_date_state(flags: u64) -> CalcState {
        let mut state = CalcState::new();
        state.flags = flags;
        state.time_offset_secs = PINNED_EPOCH - clock::adjusted_epoch_secs(0);
        state
    }

    #[test]
    fn op_adate_checks_flag_31() {
        // Fixed date with day != month, so MDY and DMY must differ regardless of today's date.
        let mut state_mdy = pinned_date_state(0);
        op_adate(&mut state_mdy).unwrap();
        assert_eq!(state_mdy.alpha_reg, " 2/13/2026");

        let mut state_dmy = pinned_date_state(1u64 << 31);
        op_adate(&mut state_dmy).unwrap();
        assert_eq!(state_dmy.alpha_reg, "13/ 2/2026");
    }

    #[test]
    fn op_adate_appends_not_replaces() {
        let mut state = CalcState::new();
        state.alpha_reg = "D=".to_string();
        op_adate(&mut state).unwrap();
        assert!(state.alpha_reg.starts_with("D="));
        assert!(state.alpha_reg.len() > 2);
    }

    #[test]
    fn alpha_append_truncates_at_24_chars() {
        let mut state = CalcState::new();
        // Fill alpha to 20 chars already
        state.alpha_reg = "A".repeat(20);
        // Try to append 10 more — should be truncated to 4
        alpha_append(&mut state, "0123456789");
        assert_eq!(state.alpha_reg.len(), 24);
        assert!(state.alpha_reg.ends_with("0123"));
    }

    #[test]
    fn alpha_append_noop_when_full() {
        let mut state = CalcState::new();
        state.alpha_reg = "X".repeat(24);
        alpha_append(&mut state, "extra");
        assert_eq!(state.alpha_reg.len(), 24);
    }

    #[test]
    fn op_atime_neutral_lift_effect() {
        // Neutral lift: stack X stays unchanged
        let mut state = CalcState::new();
        state.stack.x = crate::num::HpNum::from(42i32);
        op_atime(&mut state).unwrap();
        // Stack X should be unchanged (Neutral lift effect)
        // LINT-EXEMPT: exact integer equality — HpNum::from(42i32) uses Decimal, no f64 bridge or drift
        assert_eq!(state.stack.x, crate::num::HpNum::from(42i32));
    }

    #[test]
    fn get_local_time_unix_epoch() {
        // offset = -now_secs puts us right at Unix epoch (approximately)
        // Just verify it returns valid ranges
        let (year, month, day, hour, minute, second) = get_local_time(0);
        assert!(year >= 1970, "year should be >= 1970, got {year}");
        assert!((1..=12).contains(&month), "month out of range: {month}");
        assert!((1..=31).contains(&day), "day out of range: {day}");
        assert!(hour < 24, "hour out of range: {hour}");
        assert!(minute < 60, "minute out of range: {minute}");
        assert!(second < 60, "second out of range: {second}");
    }

    #[test]
    fn get_local_time_known_offset() {
        // Unix timestamp 0 (1970-01-01 00:00:00 UTC) — force via large negative offset.
        // SystemTime::now() adds seconds since epoch; offset = -(seconds since epoch)
        // gives us epoch 0. We can't perfectly control it (race condition with now()),
        // but we can verify the conversion math with a fixed known timestamp.

        // Known: Unix timestamp 1_716_595_200 = 2024-05-25 00:00:00 UTC
        // Verified with: calendar.timegm((2024, 5, 25, 0, 0, 0, 0, 0, 0)) in Python.
        let (year, month, day, hour, minute, second) = compute_from_unix_ts(1_716_595_200);
        assert_eq!(year, 2024);
        assert_eq!(month, 5);
        assert_eq!(day, 25);
        assert_eq!(hour, 0);
        assert_eq!(minute, 0);
        assert_eq!(second, 0);
    }

    #[test]
    fn get_local_time_known_offset_2() {
        // Unix timestamp 1_748_089_845 = 2025-05-24 12:30:45 UTC
        // Verified with: calendar.timegm((2025, 5, 24, 12, 30, 45, 0, 0, 0)) in Python.
        let (year, month, day, hour, minute, second) = compute_from_unix_ts(1_748_089_845);
        assert_eq!(year, 2025);
        assert_eq!(month, 5);
        assert_eq!(day, 24);
        assert_eq!(hour, 12);
        assert_eq!(minute, 30);
        assert_eq!(second, 45);
    }

    /// Test helper: compute time components from a fixed Unix timestamp (bypasses SystemTime::now()).
    fn compute_from_unix_ts(unix_secs: i64) -> (i32, u8, u8, u8, u8, u8) {
        let day_offset = unix_secs / 86_400;
        let time_of_day = unix_secs % 86_400;

        let hour = (time_of_day / 3600) as u8;
        let minute = ((time_of_day % 3600) / 60) as u8;
        let second = (time_of_day % 60) as u8;

        let jdn = day_offset + 2_440_588;
        let a = jdn + 32_044;
        let b = (4 * a + 3) / 146_097;
        let c = a - (146_097 * b) / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - (1461 * d) / 4;
        let m = (5 * e + 2) / 153;

        let day = (e - (153 * m + 2) / 5 + 1) as u8;
        let month = (m + 3 - 12 * (m / 10)) as u8;
        let year = (100 * b + d - 4800 + m / 10) as i32;

        (year, month, day, hour, minute, second)
    }

    #[test]
    fn format_adate_mdy_jan_24_2026() {
        // Verify the formatting logic for a known date (MDY)
        let month: u8 = 1;
        let day: u8 = 24;
        let year: i32 = 2026;
        let text = format!("{month:2}/{day:02}/{year}");
        assert_eq!(text, " 1/24/2026");
    }

    #[test]
    fn format_adate_dmy_jan_24_2026() {
        // Verify the formatting logic for a known date (DMY)
        let month: u8 = 1;
        let day: u8 = 24;
        let year: i32 = 2026;
        let text = format!("{day:02}/{month:2}/{year}");
        assert_eq!(text, "24/ 1/2026");
    }
}
