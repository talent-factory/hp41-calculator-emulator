---
phase: 11-print-emulation
reviewed: 2026-05-08T00:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - hp41-cli/src/app.rs
  - hp41-cli/src/help_data.rs
  - hp41-cli/src/main.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-cli/src/tests/keys_tests.rs
  - hp41-cli/src/ui.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/print.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/state.rs
  - hp41-core/tests/print_tests.rs
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
status: issues_found
---

# Phase 11: Post-Gap-Closure Code Review (Plan 11-03)

**Reviewed:** 2026-05-08
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found — CR-01 and CR-03 fixed; CR-02 open

This review covers the plan 11-03 gap-closure changes. The two critical blockers identified in
the prior review (CR-01, CR-03) are verified correct. CR-02 (ENG 9 width overflow in op_prstk)
is confirmed reachable and remains open. One warning (WR-01) was worsened by 11-03 by duplicating
the per-line flush pattern into the new helper.

---

## Fix Verification

### CR-01 — FIXED

`drain_and_show_print_output()` added at `hp41-cli/src/app.rs:838`.
Called at all three required sites:
- F5/R/S handler: line ~409
- F1-F4 USER handler: line ~242
- `try_user_dispatch()`: line ~818

The helper correctly drains `state.print_buffer`, writes to `print_log_writer` if present
(best-effort, `let _ =`), and sets `self.message` to the single formatted line (1-line output)
or a `"PRSTK → N lines"` summary (multi-line output). When `print_buffer` is empty the helper
is a no-op, relying on the caller to have set `self.message = None` on the `Ok(())` branch.
Fix is complete and correct.

### CR-03 — FIXED

`#[serde(default, skip)]` confirmed at `hp41-core/src/state.rs:94`. `skip` prevents
serialization; `default` (resolves to `Vec::new()`) handles backward-compat deserialization of
v1.0 save files that predate the field. Fix is complete and correct.

---

## Critical Issues

### CR-02: op_prstk ENG 9 mode produces 25-char lines (1 over 24-char contract)

**File:** `hp41-core/src/ops/print.rs:38–44`
**Confidence:** 88
**Status:** OPEN — not addressed by 11-03

The comment at line 38 states: "format_hpnum output for SCI 9 widest case is
`-1.234567890E-99` = 16 chars → fits in :>17." This is correct for SCI mode only.

ENG mode allows 1–3 integer digits in the mantissa. The widest ENG 9 output is:

    -999.999999999E-99

Character count: `-` (1) + `999` (3) + `.` (1) + `999999999` (9) + `E-` (2) + `99` (2) = 18 chars.

The format `"{:<7}{:>17}"` places a 7-char label field + 17-char value field = 24 chars.
Rust's `{:>17}` right-aligns but does NOT truncate when input exceeds the width.
An 18-char formatted value produces a 7 + 18 = **25-char line**, breaking the contract.

**Reachability:** Set `ENG 9` display mode via the `'F'` modal, then press `P S` (PRSTK) with
any stack register containing a value whose ENG mantissa has 3 integer digits (e.g., X = -123456789000).
Normal usage path.

**Test gap:** `test_prstk_all_lines_are_24_chars` uses `push_val(&mut s, 1)` in default
`DisplayMode::Fix(4)` — zero ENG 9 test coverage.

**Fix — add a clamp helper in `hp41-core/src/ops/print.rs`:**

```rust
fn format_prstk_num(val: &HpNum, mode: &crate::state::DisplayMode) -> String {
    let s = format_hpnum(val, mode);
    if s.len() <= 17 {
        format!("{:>17}", s)
    } else {
        s[..17].to_string() // clamp to maintain 24-char line contract
    }
}
```

Replace the four inline `format_hpnum` calls in `op_prstk` with `format_prstk_num`.
Add a companion test:

```rust
#[test]
fn test_prstk_all_lines_are_24_chars_eng9() {
    let mut s = CalcState::new();
    s.display_mode = DisplayMode::Eng(9);
    dispatch(&mut s, Op::PushNum(HpNum::from(-123456789000i64))).unwrap();
    dispatch(&mut s, Op::PRSTK).unwrap();
    for (i, line) in s.print_buffer.iter().enumerate() {
        assert_eq!(line.len(), 24,
            "PRSTK line {} must be 24 chars in ENG 9, got {:?}", i, line);
    }
}
```

---

## Warnings

### WR-01: Per-line flush pattern duplicated in new drain_and_show_print_output helper

**File:** `hp41-cli/src/app.rs:844` (new) and `app.rs:877` (pre-existing in call_dispatch_and_drain)
**Confidence:** 82
**Status:** OPEN — worsened by 11-03 (was 1 location, now 2)

`drain_and_show_print_output()` mirrors `call_dispatch_and_drain` line-for-line, including the
`writer.flush()` call inside the line-iteration loop. Both methods perform O(N) syscalls for
PRSTK (6 flushes instead of 1). Fix in both methods: move the single `flush()` after the loop.

```rust
for line in &lines {
    if let Some(ref mut writer) = self.print_log_writer {
        let _ = writeln!(writer, "{}", line);
    }
}
if let Some(ref mut writer) = self.print_log_writer {
    let _ = writer.flush();
}
```

### WR-02: '?' help toggle fires before pending_input guard

**File:** `hp41-cli/src/app.rs:160`
**Confidence:** 80
**Status:** OPEN — unchanged from prior review

The `'?'` check at line 160 runs before the `pending_input.is_some()` guard at line 176.
Pressing `'?'` while PrintModal (or any modal) is active opens the help overlay while leaving
`pending_input` set. Subsequent keys route to `handle_pending_input` instead of overlay
navigation — a non-functional state. Same defect applies to `Ctrl+P` at line 167.

Fix: add `self.pending_input = None;` inside both the `'?'` and `Ctrl+P` handlers.

### WR-03: PrintModal status prompt shows no key choices

**File:** `hp41-cli/src/ui.rs:264`
**Confidence:** 80
**Status:** OPEN — unchanged from prior review

`"PRNT: _"` provides no discoverable options. Every other multi-choice modal displays its choices.
Fix: `"PRNT: X=PRX  A=PRA  S=PRSTK  Esc=cancel"`.

---

## Info

### IN-01: P key absent from KEY_REF_TABLE

**File:** `hp41-cli/src/keys.rs:168` (end of KEY_REF_TABLE)
**Confidence:** 85
**Status:** OPEN — unchanged from prior review

The right-panel key reference has no entry for `'P'` (PrintModal opener). The `keys_tests.rs:127`
assertion `KEY_REF_TABLE.len() == 54` requires updating if an entry is added.

Add: `("P", "PRX/PRA/PRSTK print modal (X/A/S)")` to `KEY_REF_TABLE` and update the count to 55.

### IN-02: Unnecessary .clone() on Copy type DisplayMode in op_prstk

**File:** `hp41-core/src/ops/print.rs:36`
**Confidence:** 90
**Status:** OPEN — unchanged from prior review

`let mode = &state.display_mode.clone();` — `DisplayMode` derives `Copy`. The `.clone()` is
a no-op allocation. Fix: `let mode = state.display_mode;`.

---

## Summary

| ID | Severity | Status | Location | Description |
|----|----------|--------|----------|-------------|
| CR-01 | Critical | FIXED | `app.rs:838` | drain helper + 3 call sites |
| CR-03 | Critical | FIXED | `state.rs:94` | `#[serde(default, skip)]` |
| CR-02 | Critical | OPEN | `ops/print.rs:38–44` | ENG 9 → 25-char line |
| WR-01 | Warning | OPEN (worse) | `app.rs:844` + `:877` | per-line flush in 2 methods |
| WR-02 | Warning | OPEN | `app.rs:160` | `?` before pending guard |
| WR-03 | Warning | OPEN | `ui.rs:264` | PrintModal prompt missing choices |
| IN-01 | Info | OPEN | `keys.rs:168` | `P` key not in KEY_REF_TABLE |
| IN-02 | Info | OPEN | `ops/print.rs:36` | clone on Copy type |

---

_Reviewed: 2026-05-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard — post-gap-closure verification (plan 11-03)_
