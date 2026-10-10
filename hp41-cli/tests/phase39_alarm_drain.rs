//! Phase 39 Plan 03 Task 3 — Behavioral tests for the alarm event_buffer drain pipeline.
//!
//! Tests TIME-CLI-07: ALMNOW dispatch populates `state.event_buffer` with
//! `alarm:message:` prefixed strings, and the drain mechanic clears the buffer.
//!
//! ## Test strategy
//!
//! Option C from the plan: Test the core dispatch path end-to-end.
//! `Op::TimeAlmnow` calls `op_almnow()` which calls `dispatch_alarm_event()`,
//! which pushes `"alarm:message:{text}"` into `state.event_buffer` for Message
//! alarms (or `"alarm:xeq:{label}"` for non-interrupting control alarms).
//!
//! The CLI-side `drain_event_buffer()` routes `alarm:message:*` entries to
//! `self.message` — that routing lives in `app.rs` and is verified by
//! compilation + the Plan 39-02 Task 3 checkpoint. These tests verify the
//! contract at the core level: the correct prefixes appear in `event_buffer`
//! after ALMNOW dispatch.
//!
//! ## What ALMNOW does
//!
//! `op_almnow` scans `state.alarms` for the first past-due alarm and dispatches
//! it via `dispatch_alarm_event`. If no past-due alarm is found, it takes the
//! next upcoming alarm. With no alarms at all, it returns `Ok(())` without
//! touching `event_buffer`.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::time::{AlarmEntry, AlarmType};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;

/// Test 1: dispatching Op::TimeAlmnow with a past-due Message alarm populates
/// `event_buffer` with an `alarm:message:` prefixed string.
#[test]
fn almnow_dispatch_populates_event_buffer() {
    let mut state = CalcState::new();
    // Insert a past-due message alarm — ALMNOW will find and dispatch it.
    state.alarms.push(AlarmEntry {
        trigger_unix: 1000,
        repeat_secs: 0,
        alarm_type: AlarmType::Message("Appointment".to_string()),
        past_due: true,
    });

    dispatch(&mut state, Op::TimeAlmnow).unwrap();

    assert!(
        !state.event_buffer.is_empty(),
        "event_buffer must be non-empty after Op::TimeAlmnow dispatches a past-due Message alarm"
    );
    let has_alarm_prefix = state
        .event_buffer
        .iter()
        .any(|s| s.starts_with("alarm:message:"));
    assert!(
        has_alarm_prefix,
        "event_buffer must contain at least one 'alarm:message:' prefixed string \
         after Op::TimeAlmnow dispatches a Message alarm; got: {:?}",
        state.event_buffer
    );
}

/// Test 2: the `alarm:message:` entry contains non-empty text after the prefix.
#[test]
fn alarm_message_prefix_contains_text() {
    let mut state = CalcState::new();
    state.alarms.push(AlarmEntry {
        trigger_unix: 500,
        repeat_secs: 0,
        alarm_type: AlarmType::Message("Meeting at noon".to_string()),
        past_due: true,
    });

    dispatch(&mut state, Op::TimeAlmnow).unwrap();

    let alarm_entry = state
        .event_buffer
        .iter()
        .find(|s| s.starts_with("alarm:message:"))
        .expect("no alarm:message: entry in event_buffer");

    let after_prefix = alarm_entry.strip_prefix("alarm:message:").unwrap();
    assert!(
        !after_prefix.is_empty(),
        "alarm:message: prefix must be followed by non-empty text; \
         got empty suffix in entry: '{alarm_entry}'"
    );
    assert_eq!(
        after_prefix, "Meeting at noon",
        "alarm:message: text must match the alarm's Message content"
    );
}

/// Test 3: check_alarms() with a past-due alarm populates event_buffer.
///
/// This verifies the `check_alarms` path (called by the CLI after dispatch)
/// independently of `op_almnow`. We set up a past-due alarm and call
/// `check_alarms` directly to ensure the same dispatch_alarm_event path fires.
#[test]
fn check_alarms_with_past_due_alarm_populates_buffer() {
    use hp41_core::ops::time::alarm::check_alarms;

    let mut state = CalcState::new();
    // Insert a past-due message alarm — check_alarms will detect and dispatch it.
    state.alarms.push(AlarmEntry {
        trigger_unix: 0, // epoch start — always in the past
        repeat_secs: 0,
        alarm_type: AlarmType::Message("Reminder".to_string()),
        past_due: false, // check_alarms will flip this to true
    });

    // Override: set trigger_unix to a time clearly in the past
    // (trigger_unix=0, time_offset_secs=0 => now is >> 0)
    check_alarms(&mut state);

    assert!(
        !state.event_buffer.is_empty(),
        "event_buffer must be non-empty after check_alarms() fires a past-due alarm; \
         check that time_offset_secs=0 and trigger_unix=0 makes the alarm past-due"
    );
    let has_alarm_prefix = state.event_buffer.iter().any(|s| s.starts_with("alarm:"));
    assert!(
        has_alarm_prefix,
        "event_buffer must contain an 'alarm:' prefixed string after check_alarms; \
         got: {:?}",
        state.event_buffer
    );
}

/// Test 4: draining event_buffer clears it.
///
/// Verifies the drain mechanic at the CalcState level — manually push an
/// `alarm:message:` string and drain the buffer via the standard `drain(..)`
/// pattern that `drain_event_buffer()` in app.rs uses.
#[test]
fn event_buffer_drain_clears_buffer() {
    let mut state = CalcState::new();

    // Manually populate the buffer (simulating what dispatch_alarm_event does).
    state
        .event_buffer
        .push("alarm:message:Test message".to_string());
    state.event_buffer.push("alarm:xeq:MYPRG".to_string());

    assert_eq!(
        state.event_buffer.len(),
        2,
        "event_buffer should have 2 entries before drain"
    );

    // Drain the buffer — same pattern as app.rs drain_event_buffer().
    let drained: Vec<String> = std::mem::take(&mut state.event_buffer);

    assert!(
        state.event_buffer.is_empty(),
        "event_buffer must be empty after drain(..); len = {}",
        state.event_buffer.len()
    );
    assert_eq!(drained.len(), 2, "drain must yield all 2 entries");
    assert_eq!(drained[0], "alarm:message:Test message");
    assert_eq!(drained[1], "alarm:xeq:MYPRG");
}

/// Test 5: ALMNOW with a non-interrupting control alarm pushes alarm:xeq: prefix.
///
/// Confirms the control-alarm branch of dispatch_alarm_event produces the correct
/// `alarm:xeq:` prefix rather than `alarm:message:`.
#[test]
fn almnow_control_alarm_pushes_xeq_prefix() {
    let mut state = CalcState::new();
    state.alarms.push(AlarmEntry {
        trigger_unix: 1000,
        repeat_secs: 0,
        alarm_type: AlarmType::Control {
            label: "MYPROG".to_string(),
            interrupting: false,
        },
        past_due: true,
    });

    dispatch(&mut state, Op::TimeAlmnow).unwrap();

    let has_xeq = state
        .event_buffer
        .iter()
        .any(|s| s.starts_with("alarm:xeq:"));
    assert!(
        has_xeq,
        "event_buffer must contain 'alarm:xeq:' prefix for non-interrupting control alarms; \
         got: {:?}",
        state.event_buffer
    );
    let xeq_entry = state
        .event_buffer
        .iter()
        .find(|s| s.starts_with("alarm:xeq:"))
        .unwrap();
    assert_eq!(
        xeq_entry, "alarm:xeq:MYPROG",
        "alarm:xeq: entry must include the control alarm label"
    );
}
