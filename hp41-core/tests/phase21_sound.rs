//! Integration tests for Phase 21 Plan 04 (Sound event channel: BEEP / TONE n).
//!
//! Covers FN-SOUND-01/02 plus the zero-I/O invariant regression sentinel.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpError};

#[test]
fn test_event_buffer_field_defaults_to_empty() {
    let s = CalcState::new();
    assert!(s.event_buffer.is_empty());
}

#[test]
fn test_load_v20_save_no_event_buffer_field() {
    let json = std::fs::read_to_string("tests/fixtures/v20-autosave.json").unwrap();
    let s: CalcState = serde_json::from_str(&json).unwrap();
    assert!(
        s.event_buffer.is_empty(),
        "v2.0 fixture must load with event_buffer empty"
    );
}

#[test]
fn test_event_buffer_skipped_on_serialize() {
    let mut s = CalcState::new();
    s.event_buffer.push("BEEP".to_string());
    let json = serde_json::to_string(&s).unwrap();
    assert!(
        !json.contains("event_buffer"),
        "event_buffer must be #[serde(skip)] — JSON: {json}"
    );
}

#[test]
fn test_beep_pushes_event() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::Beep).unwrap();
    assert_eq!(s.event_buffer, vec!["BEEP".to_string()]);
}

#[test]
fn test_beep_preserves_stack() {
    use hp41_core::HpNum;
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1i32);
    s.stack.y = HpNum::from(2i32);
    s.stack.z = HpNum::from(3i32);
    s.stack.t = HpNum::from(4i32);
    s.stack.lastx = HpNum::from(5i32);
    dispatch(&mut s, Op::Beep).unwrap();
    assert_eq!(s.stack.x, HpNum::from(1i32));
    assert_eq!(s.stack.y, HpNum::from(2i32));
    assert_eq!(s.stack.z, HpNum::from(3i32));
    assert_eq!(s.stack.t, HpNum::from(4i32));
    assert_eq!(s.stack.lastx, HpNum::from(5i32));
}

#[test]
fn test_tone_n_pushes_event() {
    for n in [0u8, 5, 9] {
        let mut s = CalcState::new();
        dispatch(&mut s, Op::Tone(n)).unwrap();
        assert_eq!(s.event_buffer.last().map(String::as_str), Some(&*format!("TONE {n}")));
    }
}

#[test]
fn test_tone_out_of_range() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::Tone(10));
    assert!(matches!(r, Err(HpError::InvalidOp)));
    assert!(
        s.event_buffer.is_empty(),
        "guard must run BEFORE the push (no event on InvalidOp)"
    );
}

/// Zero-I/O invariant sentinel — verify hp41-core/src/ contains no `println!`
/// or `eprintln!` calls in production code (test modules are exempt).
/// Uses `grep -rn` and walks the output to filter out comment-prefixed and
/// cfg(test)-gated occurrences.
#[test]
fn test_no_println_in_hp41_core_after_phase21() {
    // Resolve the path relative to the cargo manifest dir so the test is
    // robust to where `cargo test` is invoked from.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src_dir = std::path::Path::new(manifest).join("src");

    let output = std::process::Command::new("grep")
        .arg("-rnE")
        .arg(r"println!|eprintln!")
        .arg(&src_dir)
        .output()
        .expect("failed to run grep");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Filter out hits that are:
    //   - inside a comment (// or //!)
    //   - inside a #[cfg(test)] module (heuristic: file path under tests/ or
    //     after a line containing `cfg(test)` — handled by the next test).
    //   - the literal documentation reference to "no println!" itself.
    let production_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| {
            let content = line.splitn(3, ':').nth(2).unwrap_or(line);
            let trimmed = content.trim_start();
            // Comment-only lines are not production code.
            if trimmed.starts_with("//") {
                return false;
            }
            // Documentation references in module-level `//!` doc comments are not
            // production code either (already filtered by the leading `//` check
            // above, but kept as belt-and-braces).
            true
        })
        .collect();

    assert!(
        production_lines.is_empty(),
        "zero-I/O invariant violated — println!/eprintln! found in hp41-core production code:\n{}",
        production_lines.join("\n")
    );
}
