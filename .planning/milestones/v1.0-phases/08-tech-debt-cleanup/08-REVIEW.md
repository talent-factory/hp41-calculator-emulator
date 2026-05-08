---
phase: 08-tech-debt-cleanup
status: findings
files_reviewed: 5
findings:
  critical: 0
  warning: 3
  info: 1
  total: 4
---

# Phase 08 Tech Debt Cleanup — Code Review

Standard depth. 5 files reviewed against CLAUDE.md conventions.

## Files Reviewed

`hp41-cli/src/app.rs`, `hp41-cli/src/help_data.rs`, `hp41-cli/src/keys.rs`, `hp41-cli/src/tests/keys_tests.rs`, `hp41-core/src/ops/mod.rs`

---

## Warnings

### WARN-01: help_data.rs help-close entry omits 'q' (confidence: 95)

`help_data.rs` shows `("Esc/?", ...)` but `app.rs` `show_help` match arm still accepts `'q'` (runs before `key_to_op`). A test at app.rs:695 explicitly validates `'q'` closes the overlay. The label was incorrectly narrowed by Plan 03.

**Fix:** `("Esc/q/?", "HELP close", "Close this overlay (Esc, q, or ? again)")`

### WARN-02: ALPHA mode section missing Delete → AlphaClear (confidence: 90)

Phase 8 added `Delete` → `Op::AlphaClear` in ALPHA mode but `help_data.rs` ALPHA section does not document it. Users cannot discover the feature from `?`.

**Fix:** Add `("Del", "ALPHA CLR", "While in ALPHA mode: clear entire ALPHA register")` after the Bksp entry.

### WARN-03: Category test covers 10/13 categories; doc comment stale (confidence: 90)

`test_all_ten_categories_present` misses `"=== Science & Engineering ==="`, `"=== Help ==="`, `"=== Quit ==="`. Module doc says `"10 categories"` (stale).

**Fix:** Extend test to all 13 categories; update module doc.

---

## Info

### INFO-01: Trailing 'e' in entry_buf silently discards number (confidence: 85)

`"1.5e"` (no exponent digits) fails both `from_str` and `from_scientific` → `HpError::InvalidOp` + silent clear. HP-41 hardware waits for exponent digits before accepting other keys. Safe (no panic) but surprising UX. No test covers it.

**Recommendation:** Document the current behavior with an explicit test so any future change is deliberate.

---

## Confirmed Correct

- `flush_entry_buf` chained parse handles `1.5e3`, `2.5E-2`, plain decimals, invalid strings
- `KeyEventKind::Release` filter is unconditionally first in `handle_key()`
- `'q'` → `Op::Sin` and `'g'` → `Op::Clreg` in `key_to_op()` with dedicated tests
- `KEY_REF_TABLE` count assertion (54) matches actual table
- Entry buffer guards (duplicate `.`, duplicate `'e'`, `'e'` before mantissa) complete and tested
- `Delete` → `Op::AlphaClear` in `handle_alpha_mode_key()` implemented and tested
- `hp41-core` has no `hp41-cli`/`hp41-gui` imports — crate invariant upheld
