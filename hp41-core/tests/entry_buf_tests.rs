//! Integration tests for the number entry buffer (entry_buf) flush semantics.
//!
//! entry_buf holds pending digit characters. On any dispatch() call,
//! flush_entry_buf() parses and pushes the buffered number before the op executes.

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Basic flush on math op ────────────────────────────────────────────────

#[test]
fn test_entry_buf_flushed_before_math_op() {
    // Set entry_buf = "4"; dispatch Sqrt → should compute sqrt(4) = 2
    let mut s = CalcState::new();
    s.entry_buf = "4".to_string();
    dispatch(&mut s, Op::Sqrt).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(2),
        "entry_buf '4' must flush before Sqrt"
    );
    assert!(
        s.entry_buf.is_empty(),
        "entry_buf must be cleared after flush"
    );
}

#[test]
fn test_entry_buf_flushed_before_add() {
    // Set X=1; entry_buf = "3"; dispatch Add → should compute 1 + 3 = 4
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1);
    s.stack.lift_enabled = true; // so entry_buf push lifts 1 to Y
    s.entry_buf = "3".to_string();
    dispatch(&mut s, Op::Add).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(4));
    assert!(
        s.entry_buf.is_empty(),
        "entry_buf must be cleared after flush"
    );
}

// ── Flush on Enter ────────────────────────────────────────────────────────

#[test]
fn test_entry_buf_flushed_before_enter() {
    // entry_buf = "7"; dispatch Enter → 7 should be on stack, then Enter duplicates it
    let mut s = CalcState::new();
    s.entry_buf = "7".to_string();
    dispatch(&mut s, Op::Enter).unwrap();
    // After flush, X = 7. Then Enter duplicates X → Y = 7, X = 7.
    assert_eq!(s.stack.x.inner(), Decimal::from(7));
    assert_eq!(s.stack.y.inner(), Decimal::from(7));
    assert!(s.entry_buf.is_empty());
}

// ── Flush on STO ──────────────────────────────────────────────────────────

#[test]
fn test_entry_buf_flushed_before_sto() {
    // entry_buf = "42"; dispatch StoReg(0) → 42 should be stored in R00
    let mut s = CalcState::new();
    s.entry_buf = "42".to_string();
    dispatch(&mut s, Op::StoReg(0)).unwrap();
    assert_eq!(s.regs[0].inner(), Decimal::from(42));
    assert!(s.entry_buf.is_empty());
}

// ── Empty buf is a no-op ──────────────────────────────────────────────────

#[test]
fn test_empty_entry_buf_is_noop() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(5);
    s.entry_buf = String::new(); // empty
    dispatch(&mut s, Op::Sq).unwrap(); // 5² = 25
    assert_eq!(s.stack.x.inner(), Decimal::from(25));
}

// ── Decimal number in buf ─────────────────────────────────────────────────

#[test]
fn test_entry_buf_decimal_number() {
    let mut s = CalcState::new();
    s.entry_buf = "3.14".to_string();
    dispatch(&mut s, Op::Clx).unwrap();
    // Flush pushes 3.14, then CLX overwrites X with 0.
    assert!(s.stack.x.is_zero(), "CLX must zero X");
    assert!(s.entry_buf.is_empty());
}

// ── Negative number in buf ────────────────────────────────────────────────

#[test]
fn test_entry_buf_negative_number() {
    let mut s = CalcState::new();
    s.entry_buf = "-9".to_string();
    dispatch(&mut s, Op::Sq).unwrap(); // (-9)² = 81
    assert_eq!(s.stack.x.inner(), Decimal::from(81));
    assert!(s.entry_buf.is_empty());
}

// ── Flush enables lift ────────────────────────────────────────────────────

#[test]
fn test_flush_enables_lift() {
    let mut s = CalcState::new();
    s.entry_buf = "5".to_string();
    s.stack.lift_enabled = false;
    // Dispatch a Neutral op (SetDeg) to trigger flush without changing stack
    dispatch(&mut s, Op::SetDeg).unwrap();
    assert!(
        s.stack.lift_enabled,
        "flush must enable lift after pushing a number"
    );
    assert_eq!(s.stack.x.inner(), Decimal::from(5));
}

// ── Multi-digit integer in buf ────────────────────────────────────────────

#[test]
fn test_entry_buf_multi_digit_integer() {
    // "150" in entry_buf; dispatch Sq → 150² = 22500
    let mut s = CalcState::new();
    s.entry_buf = "150".to_string();
    dispatch(&mut s, Op::Sq).unwrap();
    let expected = Decimal::from_str("22500").unwrap();
    assert_eq!(s.stack.x.inner(), expected);
    assert!(s.entry_buf.is_empty());
}

// ── Invalid entry_buf returns error ──────────────────────────────────────

#[test]
fn test_entry_buf_invalid_content_returns_error() {
    use hp41_core::HpError;
    let mut s = CalcState::new();
    s.entry_buf = "not_a_number".to_string();
    let result = dispatch(&mut s, Op::Sqrt);
    assert!(
        result.is_err(),
        "malformed entry_buf should yield InvalidOp error"
    );
    assert_eq!(result.unwrap_err(), HpError::InvalidOp);
    // WR-02: entry_buf must be PRESERVED on parse error so the user's input is not silently lost.
    // call_dispatch() will surface the error in self.message; the buffer stays for inspection.
    assert_eq!(
        s.entry_buf, "not_a_number",
        "entry_buf must be preserved on parse error (WR-02)"
    );
}
