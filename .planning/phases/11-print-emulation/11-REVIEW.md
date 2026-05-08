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
  critical: 3
  warning: 3
  info: 2
  total: 8
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-05-08
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Phase 11 adds `PRX`/`PRA`/`PRSTK` print operations to `hp41-core` using a buffer drain
pattern that keeps the core I/O-free. The overall architecture is sound: `CalcState.print_buffer`
is written by the core, drained by the CLI after interactive dispatches via
`call_dispatch_and_drain()`. However, three correctness bugs were found:

1. The F5 (R/S) execution path and the USER-mode F1-F4 path call `run_program` directly and
   never drain `print_buffer`, silently discarding all print output from programs.
2. `op_prstk` claims lines are 24 chars wide, but ENG 9 mode can produce an 18-char numeric
   value, making the label + value field 25 chars — breaking the fixed-width contract.
3. `print_buffer` is missing `#[serde(skip)]` and will be serialized to the JSON state file
   when an autosave fires before the buffer is drained, violating the stated invariant
   "Never persisted across sessions."

Two warnings and two informational items round out the findings.

---

## Critical Issues

### CR-01: `print_buffer` never drained after `run_program` (F5 / USER-mode F1–F4)

**File:** `hp41-cli/src/app.rs:402`
**Issue:** The F5 (R/S) handler calls `hp41_core::run_program` directly and checks only
`Ok/Err`. Any `PRX`/`PRA`/`PRSTK` ops executed inside the program accumulate in
`state.print_buffer` but are never drained, never displayed in the status bar, and
never written to the `--print-log` file. The same applies to USER-mode F1–F4 dispatch
at line 239. This means print operations inside programs are silently no-ops from the
user's perspective — the stated feature does not work for programmatic print.

The fix must drain and display `print_buffer` after `run_program` returns, mirroring
`call_dispatch_and_drain`. A helper is the cleanest approach:

```rust
// app.rs — new private helper
fn drain_and_show_print_output(&mut self) {
    let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
    if lines.is_empty() {
        return;
    }
    for line in &lines {
        if let Some(ref mut writer) = self.print_log_writer {
            let _ = writeln!(writer, "{}", line);
            let _ = writer.flush();
        }
    }
    if lines.len() > 1 {
        self.message = Some(format!("PRSTK \u{2192} {} lines", lines.len()));
    } else {
        self.message = lines.into_iter().next();
    }
}

// F5 handler (line 402):
if key.code == KeyCode::F(5) {
    match hp41_core::run_program(&mut self.state, "A") {
        Ok(()) => {
            self.drain_and_show_print_output();
            if self.message.is_none() {
                self.message = None; // already set by drain or stays None
            }
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
    return;
}
```

Apply the same drain call at lines 239–243 (USER F1–F4) and 809–813 (`try_user_dispatch`).

---

### CR-02: `op_prstk` numeric value field is 17 chars wide but ENG 9 produces 18 chars

**File:** `hp41-core/src/ops/print.rs:38`
**Issue:** The doc comment states "format_hpnum output for SCI 9 widest case is
`-1.234567890E-99` = 16 chars → fits in :>17." This is correct for SCI 9.
However, ENG mode allows mantissas with 1–3 digits before the decimal point.
The widest ENG 9 output is `-999.999999999E-99` = 18 characters.
With a 7-char label field (`"LASTX: "`) plus an 18-char value, the line total is
25 chars — one over the mandated 24-char output width.
The 24-char contract is checked by `test_prstk_all_lines_are_24_chars`, which does
**not** test ENG 9 mode. Running the test suite against an ENG 9 state would fail.

The simplest correct fix is to truncate the formatted value to fit:

```rust
// hp41-core/src/ops/print.rs — format_prstk_num helper
fn format_prstk_num(val: &crate::num::HpNum, mode: &crate::state::DisplayMode) -> String {
    let s = format_hpnum(val, mode);
    // Clamp to 17 chars to guarantee total line width = 7 + 17 = 24.
    // ENG 9 can produce 18 chars; truncation matches HP-41 printer behavior
    // of dropping the trailing digit when the value overflows the print column.
    if s.len() > 17 {
        s[..17].to_string()
    } else {
        format!("{:>17}", s)
    }
}
```

Replace the inline `format_hpnum` calls in `op_prstk` with `format_prstk_num`, and add
a test that exercises ENG 9 with a large negative value.

---

### CR-03: `print_buffer` persisted to JSON state file despite "never persisted" invariant

**File:** `hp41-core/src/state.rs:93`
**Issue:** The field comment states "Never persisted across sessions." The
`#[serde(default)]` attribute only enables deserialization from old save files that
lack the field — it does NOT prevent serialization. If the 30-second autosave fires
while `print_buffer` is non-empty (possible for long-running programs containing many
`PRX`/`PRSTK` calls), the buffer content is written to the JSON state file. On next
startup it is deserialized and immediately visible, violating the stated invariant and
potentially filling the status bar with stale output.

Fix: add `#[serde(skip)]` alongside `#[serde(default)]`:

```rust
// hp41-core/src/state.rs:93
#[serde(default, skip)]
pub print_buffer: Vec<String>,
```

`#[serde(skip)]` implies both `skip_serializing` and `skip_deserializing`. Since the
field uses `Vec::new()` as its default, deserialization from old files already works
correctly without the explicit `#[serde(default)]` attribute when `skip` is present,
but keeping both makes the intent explicit.

---

## Warnings

### WR-01: `call_dispatch_and_drain` flushed on every line — O(N) flushes for PRSTK

**File:** `hp41-cli/src/app.rs:839`
**Issue:** For `PRSTK` (6 print lines), `call_dispatch_and_drain` calls
`writer.flush()` once per line inside the loop. Each `flush()` call is a syscall.
While not a performance issue in normal use, it is logically inconsistent with the
drain-then-flush pattern described in the surrounding comment and could mask partial
write failures: a `writeln!` error on line 4 of 6 is silently discarded (`let _ =`),
but the 3 lines written before it are already flushed. The lines written after the
error are also flushed as if nothing happened. A single flush after the loop would
be more coherent.

```rust
// Replace the per-line flush:
for line in &lines {
    if let Some(ref mut writer) = self.print_log_writer {
        let _ = writeln!(writer, "{}", line);
    }
}
// Single flush after all lines are written:
if let Some(ref mut writer) = self.print_log_writer {
    let _ = writer.flush();
}
```

---

### WR-02: `?` overlay toggle fires unconditionally before modal guard

**File:** `hp41-cli/src/app.rs:160`
**Issue:** The `'?'` key check at line 160 runs before the `pending_input.is_some()`
guard at line 176. This means pressing `?` while a `PrintModal`, `StoRegister`, or any
other modal is active immediately opens the help overlay and leaves `pending_input`
set. The user's modal context is abandoned silently. The status bar still shows the
modal prompt (because `pending_input` was not cleared), but all subsequent keys route
to `handle_pending_input`, not to the help overlay navigation handlers. The result is
a split-brain state: overlay is visible but un-navigable; modal is "active" but
shadowed.

Fix: move the `'?'` check to after the `pending_input.is_some()` guard, or add
`self.pending_input = None;` inside the `'?'` handler:

```rust
// hp41-cli/src/app.rs — inside the '?' handler
if key.code == KeyCode::Char('?') {
    self.pending_input = None; // close any active modal first
    self.show_help = !self.show_help;
    self.show_programs = false;
    return;
}
```

The same issue applies to `Ctrl+P` at line 167 — it can open the programs overlay
while a modal is active. Add `self.pending_input = None;` there too.

---

### WR-03: `PrintModal` status prompt is under-specified (user discoverability)

**File:** `hp41-cli/src/ui.rs:264`
**Issue:** The status bar prompt for `PrintModal` renders as `"PRNT: _"` — it does
not show the available key choices (`X`, `A`, `S`). All other modals with multiple
choices display their option set (e.g., `"STO [__]"`, `"FIX [_]  (0–9 set digits, f cycles, Esc cancel)"`). A user who has never read the help data has no way to know
what key to press.

This is a quality defect, not a correctness bug — the modal works correctly.

```rust
// hp41-cli/src/ui.rs:264
PendingInput::PrintModal => "PRNT: X=PRX  A=PRA  S=PRSTK  Esc=cancel".to_string(),
```

---

## Info

### IN-01: `KEY_REF_TABLE` missing `P` entry for PrintModal

**File:** `hp41-cli/src/keys.rs:90`
**Issue:** The `KEY_REF_TABLE` constant drives the right-panel key reference display.
Phase 11 added the `P` (Shift+p) key to open the print modal, but no entry for `P`
was added to `KEY_REF_TABLE`. The help overlay (`HELP_DATA` in `help_data.rs`) has the
Print category entries, but those list `"P X"`, `"P A"`, `"P S"` — not the top-level
`P` key. The right-panel key reference therefore has no visible entry for the print
feature.

Add an entry to `KEY_REF_TABLE` and update the test in `keys_tests.rs` that asserts
the exact count of 54 entries (line 127):

```rust
// hp41-cli/src/keys.rs — add after the 'F' entry
("P", "PRX/PRA/PRSTK (print modal: X/A/S)"),
```

---

### IN-02: `op_prstk` creates an unnecessary `DisplayMode` clone

**File:** `hp41-core/src/ops/print.rs:36`
**Issue:** `let mode = &state.display_mode.clone();` clones a `Copy` type
(`DisplayMode` derives `Clone` and `Copy`). The clone is unneeded; a direct reference
or copy suffices:

```rust
let mode = state.display_mode; // DisplayMode is Copy
```

This is a trivial dead-clone producing slightly misleading code.

---

_Reviewed: 2026-05-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
