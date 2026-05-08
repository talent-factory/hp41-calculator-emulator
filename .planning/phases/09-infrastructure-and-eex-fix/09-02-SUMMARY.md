---
phase: 09-infrastructure-and-eex-fix
plan: "02"
subsystem: hp41-core
tags:
  - eex
  - flush_entry_buf
  - tdd
  - bug-fix
  - hardware-fidelity
dependency_graph:
  requires: []
  provides:
    - "flush_entry_buf trailing-e normalization (D-09)"
    - "test_flush_trailing_e_without_exponent_commits_zero_exponent (D-10)"
  affects:
    - hp41-core/src/ops/mod.rs
tech_stack:
  added: []
  patterns:
    - "trailing-e normalization via ends_with('e') + push_str(\"00\") before parse chain"
key_files:
  created: []
  modified:
    - hp41-core/src/ops/mod.rs
decisions:
  - "D-09: append '00' to trailing-e entry_buf before parse chain so from_scientific handles it"
  - "D-10: inverted test replaced with two positive tests asserting Ok(()) and correct X values"
metrics:
  duration: "2m"
  completed: "2026-05-08"
  tasks_completed: 2
  files_modified: 1
---

# Phase 9 Plan 02: EEX Core Fix — flush_entry_buf Trailing-e Normalization Summary

## One-liner

Corrected `flush_entry_buf` to normalize trailing-e entry buffers (e.g., `"1.5e"`) by appending `"00"` before parsing, matching HP-41 hardware behavior where a bare EEX commit pushes the number with exponent 00.

---

## What Was Built

### Task 1: Invert the trailing-e test (RED phase)

Replaced the bug-documenting test `test_flush_trailing_e_without_exponent_returns_err` with two new HP-41 hardware-faithful tests:

**`test_flush_trailing_e_without_exponent_commits_zero_exponent`**
- Input: `entry_buf = "1.5e"`
- Asserts: `Ok(())`, `stack.x = 1.5`, `entry_buf` empty after call

**`test_flush_implicit_one_with_trailing_e_commits_one`**
- Input: `entry_buf = "1e"`
- Asserts: `Ok(())`, `stack.x = 1`

Both tests fail at RED phase (implementation not yet updated). The four pre-existing regression tests (`test_flush_scientific_lowercase_e`, `test_flush_scientific_uppercase_e`, `test_flush_plain_decimal_still_works`, `test_flush_invalid_returns_err`) continue passing.

**Commit:** `cc32796`

---

### Task 2: Implement trailing-e normalization (GREEN phase)

Modified `flush_entry_buf()` at `hp41-core/src/ops/mod.rs:207`. The exact diff:

**Change 1:** `let s = state.entry_buf.clone();` → `let mut s = state.entry_buf.clone();` (mutability required)

**Change 2 (inserted between `entry_buf.clear()` and parse chain):**
```rust
// D-09 (Phase 9): trailing 'e' with no exponent digits is HP-41 hardware-faithful
// shorthand for "exponent 00". Normalize by appending "00" so from_scientific accepts it.
// We check both 'e' and 'E' for safety even though entry_buf is always lowercase per
// app.rs; this also makes the normalization robust to future case-folding changes.
if s.ends_with('e') || s.ends_with('E') {
    s.push_str("00");
}
```

The existing `Decimal::from_scientific()` fallback (added in Phase 8) already handles `"1.5e00"` and `"1e00"` correctly — D-09 just ensures the string reaches it in parseable form.

All other lines in `flush_entry_buf` are unchanged: signature, doc comment, early-return, parse chain, prgm_mode branch, stack-lift call, `Ok(())` return.

**Commit:** `34b89f9`

---

## Test Results

```
cargo test -p hp41-core flush_eex_tests
  test result: ok. 6 passed; 0 failed (was 5 in old code — 1 removed, 2 added = net +1)
```

```
cargo test -p hp41-core
  test result: ok. 385 passed; 0 failed (17 suites, 0.34s)
```

```
cargo clippy -p hp41-core --all-targets -- -D warnings
  No issues found
```

---

## Files Modified

| File | Changes |
|------|---------|
| `hp41-core/src/ops/mod.rs` | `flush_entry_buf`: +`mut`, +5-line normalization block; `flush_eex_tests`: removed 1 test, added 2 tests |

No other files were modified. No new files created. No dependencies added.

---

## Deviations from Plan

None — plan executed exactly as written.

---

## Note for 09-03 Executor

The core `flush_entry_buf` is now hardware-faithful. Plan 09-03 must implement:

1. **D-07** (`hp41-cli/src/app.rs`): Remove the guard that blocks EEX when `entry_buf` is empty. Instead insert `"1"` as implicit mantissa so `entry_buf = "1e"`. `flush_entry_buf("1e")` will now commit as 1.
2. **D-05/D-06** (`hp41-cli/src/app.rs`): Cap exponent entry at 2 digits — silently block a 3rd digit after `'e'` in `entry_buf` in `handle_key()`.
3. **D-01 through D-04** (`hp41-cli/src/ui.rs`): Render exponent placeholders in `get_display_string()`: `"1.5e"` → `"1.5E_ _"`, `"1.5e2"` → `"1.5E2_"`, `"1.5e23"` → `"1.5E23"`, `"1e"` → `"1E_ _"`.

---

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED (`test(...)`) | cc32796 | PASS — 2 new tests fail, 4 old pass |
| GREEN (`feat(...)`) | 34b89f9 | PASS — all 6 tests pass, no regressions |

---

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `hp41-core/src/ops/mod.rs` | FOUND |
| `09-02-SUMMARY.md` | FOUND |
| Commit `cc32796` (RED) | FOUND |
| Commit `34b89f9` (GREEN) | FOUND |
