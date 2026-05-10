---
phase: 09-infrastructure-and-eex-fix
plan: "03"
subsystem: hp41-cli
tags:
  - eex
  - entry-buf
  - tui-display
  - hardware-fidelity
  - integration-tests
dependency_graph:
  requires:
    - "09-02: flush_entry_buf trailing-e normalization (D-09)"
  provides:
    - "EEX empty-buffer implicit '1' insertion (D-07)"
    - "2-digit exponent cap in handle_key (D-05/D-06)"
    - "Exponent placeholder display rendering (D-01..D-04)"
    - "format_entry_buf_display helper + 7 unit tests"
    - "4 end-to-end EEX integration tests"
  affects:
    - hp41-cli/src/app.rs
    - hp41-cli/src/ui.rs
tech_stack:
  added: []
  patterns:
    - "entry_buf.find('e') + chars().filter(is_ascii_digit).count() for exponent digit cap"
    - "format_entry_buf_display: split at 'e', uppercase E, render 0/1/2 slots with underscore placeholders"
    - "App::new_for_test() cfg(test) constructor for handle_key integration tests"
key_files:
  created: []
  modified:
    - hp41-cli/src/app.rs
    - hp41-cli/src/ui.rs
decisions:
  - "D-07: empty-buffer EEX inserts \"1e\" (implicit mantissa) instead of blocking"
  - "D-05/D-06: exponent capped at 2 digits via find('e') + digit-count pre-check; silent block"
  - "D-08: double-EEX block retained (contains('e') guard is separate from is_empty guard)"
  - "D-01..D-04: format_entry_buf_display renders underscore placeholders for unfilled exponent slots"
  - "Compare HpNum via format_hpnum (not rust_decimal::Decimal) — rust_decimal not a direct dep of hp41-cli"
metrics:
  duration: "15m"
  completed: "2026-05-08"
  tasks_completed: 3
  files_modified: 2
---

# Phase 9 Plan 03: CLI EEX Hardware-Fidelity Wiring Summary

## One-liner

Wired the CLI-side EEX hardware-fidelity: implicit "1" mantissa on empty-buffer EEX, 2-digit exponent cap in handle_key, and `format_entry_buf_display` placeholder rendering in the TUI — completing INPUT-01/02/03 end-to-end with plan 09-02.

---

## What Was Built

### Task 1: Update handle_key EEX/digit branches (D-05/D-06/D-07/D-08)

Modified the `if let KeyCode::Char(c) = key.code { ... }` block in `handle_key()` in `hp41-cli/src/app.rs`:

**D-07 change:** Removed the `is_empty()` condition from the combined guard `self.state.entry_buf.is_empty() || self.state.entry_buf.contains('e')`. The empty case now inserts `"1e"` as the implicit mantissa:

```rust
if c == 'e' {
    if self.state.entry_buf.contains('e') {
        return; // D-08: double-EEX block retained
    }
    if self.state.entry_buf.is_empty() {
        self.state.entry_buf.push_str("1e"); // D-07: implicit mantissa
    } else {
        self.state.entry_buf.push('e');
    }
    self.message = None;
    return;
}
```

**D-05/D-06 change:** Added a pre-check in the digit branch that counts digits after the `'e'` position and silently blocks a 3rd digit:

```rust
if c.is_ascii_digit() {
    if let Some(e_pos) = self.state.entry_buf.find('e') {
        let after_e = &self.state.entry_buf[e_pos + 1..];
        let exp_digit_count = after_e.chars().filter(|ch| ch.is_ascii_digit()).count();
        if exp_digit_count >= 2 {
            return; // silently block 3rd exponent digit
        }
    }
    self.state.entry_buf.push(c);
    ...
}
```

**Test update:** The pre-existing test `test_eex_blocked_when_entry_buf_empty` was updated to `test_eex_on_empty_entry_buf_inserts_implicit_one` — it now asserts the new D-07 behavior (`entry_buf == "1e"` after pressing 'e' on an empty buffer) instead of the old blocking behavior.

**Commit:** `31b2a77`

---

### Task 2: Add format_entry_buf_display helper in ui.rs (D-01..D-04)

Made three edits to `hp41-cli/src/ui.rs`:

**Edit 1 — New helper function** placed after `get_display_string`:

```rust
fn format_entry_buf_display(s: &str) -> String {
    let Some(e_pos) = s.find('e') else { return s.to_string(); };
    let mantissa = &s[..e_pos];
    let after_e = &s[e_pos + 1..];
    let (sign, digits) = if let Some(rest) = after_e.strip_prefix('-') { ("-", rest) }
                         else if let Some(rest) = after_e.strip_prefix('+') { ("+", rest) }
                         else { ("", after_e) };
    let typed: Vec<char> = digits.chars().take(2).collect();
    let slot_render = match typed.len() {
        0 => "_ _".to_string(),
        1 => format!("{}_", typed[0]),
        _ => format!("{}{}", typed[0], typed[1]),
    };
    format!("{mantissa}E{sign}{slot_render}")
}
```

Display mapping (per decisions D-01..D-04):
| entry_buf | Display |
|-----------|---------|
| `"1.5e"` | `"1.5E_ _"` |
| `"1.5e2"` | `"1.5E2_"` |
| `"1.5e23"` | `"1.5E23"` |
| `"1e"` | `"1E_ _"` |
| `"1.5e-2"` | `"1.5E-2_"` |
| `"1.5e-23"` | `"1.5E-23"` |

**Edit 2 — Route `get_display_string` through the helper:**

```rust
if !st.entry_buf.is_empty() {
    if st.entry_buf.contains('e') {
        format_entry_buf_display(&st.entry_buf)
    } else {
        st.entry_buf.clone()
    }
}
```

**Edit 3 — 7 unit tests** in `entry_buf_display_tests` module covering all D-01..D-04 cases plus negative exponent (one and two digits) and verbatim-fallback (no 'e' in input).

All 7 tests pass.

**Commit:** `e1b9e8e`

---

### Task 3: End-to-end integration tests (Task 3)

Added `App::new_for_test()` (cfg(test) impl block) and a `eex_integration_tests` module with 4 tests to `hp41-cli/src/app.rs`:

| Test | Behavior Verified |
|------|-------------------|
| `test_eex_trailing_e_then_enter_pushes_mantissa` | "1.5e" + Op::Enter → stack.x = 1.5 (exponent 00 via D-09 normalization) |
| `test_empty_buffer_eex_inserts_implicit_one` | 'e' on empty buf → entry_buf = "1e" (D-07) |
| `test_exponent_digit_cap_blocks_third_digit` | "1.5e23" + '4' → entry_buf stays "1.5e23" (D-05/D-06) |
| `test_double_eex_blocked` | "1.5e" + 'e' → entry_buf stays "1.5e" (D-08) |

**Implementation note:** `rust_decimal::Decimal` is not a direct dependency of `hp41-cli`. The X register comparison uses `hp41_core::format_hpnum(&app.state.stack.x, &app.state.display_mode)` against `"1.5000"` (FIX 4, the default display mode).

All 4 tests pass.

**Commit:** `55fd6c8`

---

## Test Results

```
cargo test -p hp41-cli eex_integration_tests
  test result: ok. 4 passed; 0 failed (all integration tests)

cargo test -p hp41-cli entry_buf_display_tests
  test result: ok. 7 passed; 0 failed (all display unit tests)

cargo test -p hp41-cli
  test result: ok. 76 passed; 0 failed (full cli suite; was 65 before this plan)

cargo test --workspace
  test result: ok — all suites pass (0 failed across workspace)

just ci
  clippy: clean (no warnings)
  test: all pass
  coverage hp41-core: 94.22% (gate: ≥ 80%) — PASS
```

---

## Files Modified

| File | Changes |
|------|---------|
| `hp41-cli/src/app.rs` | `handle_key()`: D-07/D-05/D-06/D-08 EEX guard updates; updated existing EEX-empty test; added `App::new_for_test()`; added `eex_integration_tests` module (4 tests) |
| `hp41-cli/src/ui.rs` | `get_display_string()`: branch via `format_entry_buf_display` when entry_buf contains 'e'; new `format_entry_buf_display` helper function; new `entry_buf_display_tests` module (7 tests) |

---

## Phase 9 ROADMAP Success Criteria Status

| Criterion | Plan | Status |
|-----------|------|--------|
| 1. Typing 1.5 then EEX then ENTER pushes 1.5 | 09-02 + 09-03 | DONE — `test_eex_trailing_e_then_enter_pushes_mantissa` |
| 2. EEX on empty buffer shows "1E_ _" | 09-03 | DONE — `test_empty_buffer_eex_inserts_implicit_one` + `test_d04_implicit_one_mantissa` |
| 3. Partial-exponent state shows placeholder cursor | 09-03 | DONE — `test_d01_trailing_e_no_digits` + `test_d02_one_exponent_digit` |
| 4. `just ci` passes with Rust 1.85; rust-version declared; rust_decimal 1.42 | 09-01 | DONE — delivered by plan 09-01 |
| 5. Inverted test passes with hardware-faithful assertion | 09-02 | DONE — `test_flush_trailing_e_without_exponent_commits_zero_exponent` |

All Phase 9 acceptance gates are now met. Plans 09-01 + 09-02 + 09-03 together close all 5 success criteria.

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated stale test asserting old blocking behavior**
- **Found during:** Task 1
- **Issue:** `test_eex_blocked_when_entry_buf_empty` asserted `entry_buf.is_empty()` after pressing 'e' — this was the correct assertion for the OLD (buggy) behavior. After D-07 it would fail.
- **Fix:** Renamed test to `test_eex_on_empty_entry_buf_inserts_implicit_one`; updated assertion to `entry_buf == "1e"` (the new hardware-faithful behavior)
- **Files modified:** `hp41-cli/src/app.rs`

**2. [Rule 1 - Bug] Fixed HpNum field access from hp41-cli test code**
- **Found during:** Task 3
- **Issue:** Plan suggested `app.state.stack.x.0` to get the underlying Decimal. `HpNum.0` is `pub(crate)` in `hp41-core`, not visible from `hp41-cli`.
- **Fix:** Used `hp41_core::format_hpnum(&app.state.stack.x, &app.state.display_mode)` compared against `"1.5000"` (FIX 4 format)
- **Files modified:** `hp41-cli/src/app.rs` (test assertion only)

**3. [Rule 1 - Bug] Fixed missing rust_decimal import in test module**
- **Found during:** Task 3
- **Issue:** Plan's test template used `use rust_decimal::Decimal` and `Decimal::from_str("1.5")`. `rust_decimal` is not a direct dependency of `hp41-cli` (only transitively via `hp41-core`).
- **Fix:** Removed `use rust_decimal` and `use std::str::FromStr`; replaced comparison with `format_hpnum` approach
- **Files modified:** `hp41-cli/src/app.rs` (test imports only)

---

## Known Stubs

None — all behaviors are wired end-to-end. No placeholder data flows to the UI.

---

## Threat Flags

None — this plan modifies internal entry_buf string handling and display formatting only. No new network endpoints, auth paths, file access patterns, or schema changes.

---

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `hp41-cli/src/app.rs` | FOUND |
| `hp41-cli/src/ui.rs` | FOUND |
| `09-03-SUMMARY.md` | FOUND |
| Commit `31b2a77` (Task 1) | FOUND |
| Commit `e1b9e8e` (Task 2) | FOUND |
| Commit `55fd6c8` (Task 3) | FOUND |
