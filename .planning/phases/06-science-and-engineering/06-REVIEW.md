---
status: issues_found
files_reviewed: 12
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
reviewed_by: claude-sonnet-4-6
reviewed_on: 2026-05-07
phase: 06-science-and-engineering
depth: standard
---

# Phase 6 Code Review — Science & Engineering

Reviewed 12 files: `hp41-cli/src/help_data.rs`, `hp41-cli/src/keys.rs`,
`hp41-cli/src/prgm_display.rs`, `hp41-cli/src/tests/keys_tests.rs`,
`hp41-core/src/error.rs`, `hp41-core/src/ops/hms.rs`,
`hp41-core/src/ops/mod.rs`, `hp41-core/src/ops/program.rs`,
`hp41-core/src/ops/stats.rs`, `hp41-core/src/tests.rs`,
`hp41-core/tests/hms_tests.rs`, `hp41-core/tests/stats_tests.rs`.

Focus: bugs, logic errors, Rust-specific issues (panics, unsafe casts, integer
overflow, off-by-one), HP-41 numerical correctness, test quality.

Overall the Phase 6 implementation is well-structured. The ADR-001 string-split
field-extraction pattern is correctly applied in `parse_hms`. Stack-lift
semantics and LASTX behavior are correct for all new ops. The integration test
suites provide good happy-path coverage. Six issues were found at or above the
80-confidence threshold.

---

## Critical

### C-01 — Narrowing `i64 as i32` cast in `hms_fields_to_decimal` silently wraps for large inputs

- **File:** `hp41-core/src/ops/hms.rs`, lines 54–56
- **Confidence:** 95

`hms_fields_to_decimal` receives `hours: i64` (parsed from the integer part of
an `HpNum`, which can represent values up to ~10^99) but creates the `HpNum`
via `HpNum::from(hours as i32)`. The `as i32` cast silently wraps for any
`hours` value exceeding 2,147,483,647. An input of `2200000000.0000` would
produce a wrong negative hours value with no error returned.

`minutes as i32` and `seconds as i32` are safe in practice because both fields
are validated to 0–59 before reaching this function. The `hours` cast is the
only real risk, but it is on the code path executed for every `Op::HmsToH` call.

**Recommendation:** Replace `HpNum::from(hours as i32)` with a direct
`Decimal`-based construction that avoids the narrowing cast:

```rust
// Before (line 54):
let h = HpNum::from(hours as i32);

// After:
let h = HpNum(rust_decimal::Decimal::from(hours));
```

If `HpNum::from(i64)` is needed more broadly, add `impl From<i64> for HpNum`
to `num.rs` mirroring the existing `impl From<i32>`.

---

## Warning

### W-01 — `op_sigma_minus` allows n to go negative, corrupting subsequent stats ops

- **File:** `hp41-core/src/ops/stats.rs`, lines 53–75
- **Confidence:** 85

`op_sigma_minus` does not guard against being called when the count register
R03 (`state.regs[3]`) is already zero. It unconditionally subtracts 1, storing
`n = -1`. Every downstream stats function (`op_mean`, `op_sdev`, `op_lr`,
`op_yhat`, `op_corr`) checks `n.is_zero()` and returns `HpError::InvalidOp`,
but none check for `n < 0`. After one unguarded Σ− on empty data, `n` becomes
-1 and all subsequent accumulations and computations proceed with a negative
count, producing numerically incorrect results without raising any error. The
HP-41 hardware returns an error when Σ− would go below zero.

**Recommendation:** Add a pre-condition guard at the start of `op_sigma_minus`:

```rust
if state.regs[3].is_zero() {
    return Err(HpError::InvalidOp);
}
```

Add a corresponding integration test in `stats_tests.rs`:

```rust
#[test]
fn test_sigma_minus_on_empty_returns_invalid_op() {
    let mut s = CalcState::new();
    push(&mut s, 5);
    push(&mut s, 3);
    assert_eq!(dispatch(&mut s, Op::SigmaMinus), Err(HpError::InvalidOp));
}
```

### W-02 — `test_all_ten_categories_present` does not check the Phase 6 category

- **File:** `hp41-cli/src/help_data.rs`, lines 124–143
- **Confidence:** 85

The test was not updated when Phase 6 added `"=== Science & Engineering ==="`
to `HELP_DATA`. The `categories` array in the test lists exactly 10 entries;
the new 11th category is absent. The test continues to pass (the 10 original
categories are still present), but there is no regression guard for the Science
& Engineering header.

**Recommendation:** Add `"=== Science & Engineering ==="` to the `categories`
array and rename the test to `test_all_categories_present`.

### W-03 — Help overlay documents `"a"` as ALPHA toggle; `keys.rs` maps `'a'` to `Op::Asin`

- **Files:**
  - `hp41-cli/src/help_data.rs`, lines 55–56
  - `hp41-cli/src/keys.rs`, line 36
- **Confidence:** 82

The ALPHA Mode section of `HELP_DATA` has two entries with key `"a"` but
`key_to_op` unconditionally maps `KeyCode::Char('a')` to `Some(Op::Asin)`.
Users consulting the help overlay will press `'a'` expecting ALPHA mode and
trigger ASIN instead.

**Recommendation:** Audit `app.handle_key` for the actual ALPHA toggle
mechanism. If `'a'` is intercepted upstream for ALPHA, document the dual
behavior with a comment. Otherwise, correct the ALPHA mode help entries to
show the real key binding.

---

## Info

### I-01 — `hms_to_total_secs` multiplication can overflow `i64` for extreme inputs

- **File:** `hp41-core/src/ops/hms.rs`, lines 114–116
- **Confidence:** 80

`hms_to_total_secs` computes `hours * 3600 + minutes * 60 + seconds` in `i64`
arithmetic. For `hours` values above approximately 2.57 × 10^15 (within
HP-41's numeric range), `hours * 3600` overflows `i64::MAX`. In debug builds
this panics; in release builds it wraps silently.

**Recommendation:** Use `saturating_mul`/`saturating_add`, or document the
safe input range in the function comment.

### I-02 — Duplicate test body in `stats_tests.rs`

- **File:** `hp41-core/tests/stats_tests.rs`, lines 173–182 and 214–223
- **Confidence:** 80

`test_corr_denominator_zero_returns_error` (in the L.R. section) and
`test_corr_singular_returns_error` (in the CORR section) are exact duplicates.
The first was likely intended to test `Op::LR`'s singular-denominator case but
was accidentally written with `Op::Corr`. The `Op::LR` singular path has no
dedicated test.

**Recommendation:** Replace `test_corr_denominator_zero_returns_error` with:

```rust
#[test]
fn test_lr_denominator_zero_returns_invalid_op() {
    let mut s = CalcState::new();
    for y in [1i32, 2, 3] {
        add_point(&mut s, y, 5); // All X=5 → denom = 0
    }
    assert_eq!(dispatch(&mut s, Op::LR), Err(HpError::InvalidOp));
}
```

---

## Summary Table

| ID | Severity | File | Line(s) | Confidence | Description |
|----|----------|------|---------|------------|-------------|
| C-01 | Critical | `hp41-core/src/ops/hms.rs` | 54–56 | 95 | `hours as i32` silent narrowing cast wraps for hours > 2,147,483,647 |
| W-01 | Warning | `hp41-core/src/ops/stats.rs` | 53–75 | 85 | `op_sigma_minus` no guard for n=0; produces n=−1 corrupting all subsequent stats |
| W-02 | Warning | `hp41-cli/src/help_data.rs` | 124–143 | 85 | `test_all_ten_categories_present` omits Science & Engineering category |
| W-03 | Warning | `hp41-cli/src/help_data.rs` + `hp41-cli/src/keys.rs` | 55–56 / 36 | 82 | Key `"a"` documented as ALPHA toggle in help; mapped to `Op::Asin` in key_to_op |
| I-01 | Info | `hp41-core/src/ops/hms.rs` | 114–116 | 80 | `hms_to_total_secs` multiplication overflows i64 for hours > ~2.57 × 10^15 |
| I-02 | Info | `hp41-core/tests/stats_tests.rs` | 173–182 / 214–223 | 80 | Duplicate test body — should test Op::LR singular case, not Op::Corr again |
