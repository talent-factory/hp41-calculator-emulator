---
phase: 05-persistence-and-ux
reviewed: 2026-05-07T00:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - Cargo.toml
  - hp41-cli/Cargo.toml
  - hp41-cli/src/app.rs
  - hp41-cli/src/help_data.rs
  - hp41-cli/src/keys.rs
  - hp41-cli/src/main.rs
  - hp41-cli/src/persistence.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-cli/src/programs.rs
  - hp41-cli/src/tests/keys_tests.rs
  - hp41-cli/src/tests/mod.rs
  - hp41-cli/src/ui.rs
  - hp41-core/Cargo.toml
  - hp41-core/src/num.rs
  - hp41-core/src/ops/alpha.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/registers.rs
  - hp41-core/src/state.rs
  - hp41-core/src/tests.rs
findings:
  critical: 5
  warning: 4
  info: 3
  total: 12
status: issues_found
---

# Phase 05: Code Review Report

**Reviewed:** 2026-05-07
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

Phase 5 delivered persistence, overlays, USER mode, ALPHA backspace, and the sample
program library. The persistence core (save/load/autosave) is solid: atomic writes,
version-tagged JSON, `is_running` reset on load, and a thorough test suite. The TUI
overlay architecture (`RefCell<TableState>`, z-ordered rendering) is sound.

However, five blockers were found. The most impactful are an incorrect key-dispatch
ordering in `handle_key` that makes the 'q' key bypass overlay and alpha-mode
contexts; 'S'/'R' STO/RCL modal activation bypassing alpha mode entirely; a
completely broken prime-test sample program; and the EEX ('e') key being documented
and advertised but non-functional due to `rust_decimal::Decimal::from_str` not
accepting scientific notation. A quadratic solver description mismatch also
misguides users.

---

## Critical Issues

### CR-01: 'q' key quits application even when help overlay or alpha mode is active

**File:** `hp41-cli/src/app.rs:127-130`

**Issue:** The unconditional `'q'` quit check fires at line 127, before the
`show_help` guard (line 222) and the `alpha_mode` guard (line 188). Consequence:

- Pressing `'q'` while the help overlay is open quits the application instead
  of closing the overlay as documented (`Esc/q/?` in `HELP_DATA`).
- Pressing `'q'` in ALPHA mode quits the application instead of appending `'q'`
  to the ALPHA register.

The `show_help` match arm at line 224 (`KeyCode::Char('q') => show_help = false`)
is dead code for this reason.

**Fix:** Move the `'q'` quit guard to after the overlay and alpha-mode gates, or
check that no context-specific mode is active before quitting:

```rust
fn handle_key(&mut self, key: KeyEvent) {
    if key.kind != KeyEventKind::Press { return; }

    // Always-active global keys (quit, Ctrl+C) must check context
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        self.exit = true;
        return;
    }

    // Quit 'q' is context-sensitive: only quit when no overlay/modal/alpha is active
    if key.code == KeyCode::Char('q')
        && !self.show_help
        && !self.show_programs
        && !self.state.alpha_mode
        && self.pending_input.is_none()
    {
        self.exit = true;
        return;
    }
    // ... rest of handle_key
```

---

### CR-02: 'S', 'R', Ctrl+A modal triggers bypass ALPHA mode — 'S' and 'R' cannot be typed in ALPHA register

**File:** `hp41-cli/src/app.rs:164-183`

**Issue:** The STO modal (`'S'`, line 164), RCL modal (`'R'`, line 170), and USER
ASSIGN modal (`Ctrl+A`, line 175) are intercepted unconditionally before the
`alpha_mode` check at line 188. When `alpha_mode` is active:

- Pressing `'S'` opens the STO register modal instead of appending `'S'` to
  `alpha_reg`. The letters S and R are common in HP-41 ALPHA labels (e.g. "START").
- Pressing `'R'` opens the RCL modal instead of appending `'R'`.
- `Ctrl+A` launches the USER key assignment modal.

**Fix:** Guard these modal activations with `!self.state.alpha_mode`:

```rust
if key.code == KeyCode::Char('S')
    && !key.modifiers.contains(KeyModifiers::CONTROL)
    && !self.state.alpha_mode          // <- add this guard
{
    self.pending_input = Some(PendingInput::StoRegister(String::new()));
    self.message = None;
    return;
}
if key.code == KeyCode::Char('R')
    && !key.modifiers.contains(KeyModifiers::CONTROL)
    && !self.state.alpha_mode          // <- add this guard
{
    self.pending_input = Some(PendingInput::RclRegister(String::new()));
    self.message = None;
    return;
}
```

Similarly for `Ctrl+A`.

---

### CR-03: EEX key ('e') is non-functional — `Decimal::from_str` rejects scientific notation

**File:** `hp41-cli/src/app.rs:275`, `hp41-core/src/ops/mod.rs:171`

**Issue:** The key `'e'` is documented as "EEX (sci notation entry)" in
`KEY_REF_TABLE` (keys.rs:74) and in `HELP_DATA` (help_data.rs:71), implying that
typing e.g. `1e3` then Enter pushes 1000. In practice, the key appends the literal
character `'e'` to `entry_buf`. When the entry is flushed, `flush_entry_buf` calls:

```rust
let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
```

`rust_decimal::Decimal::from_str` does NOT handle `e`/`E` scientific notation (its
`parse_str_radix_10` implementation accepts only `[0-9]`, `.`, `_`, `+`, `-`). A
separate `Decimal::from_scientific()` API exists for that purpose. Any entry
containing `'e'` (e.g. `"1e3"`, `"2.5e-2"`) returns `HpError::InvalidOp` at flush
time, showing "invalid operation" in the status bar. The key is effectively broken.

**Fix:** In `flush_entry_buf`, try both parsers:

```rust
let d = Decimal::from_str(&s)
    .or_else(|_| Decimal::from_scientific(&s))
    .map_err(|_| HpError::InvalidOp)?;
```

Or pre-process the buffer to detect the `e`/`E` marker and route to
`from_scientific`.

---

### CR-04: Prime Test sample program returns 1 (prime) for every integer >= 2

**File:** `hp41-cli/src/programs.rs:140-181`

**Issue:** The early-exit condition in `prime_test_ops` is logically inverted.
The intent is "if n <= 2, treat as prime immediately" but the code does:

```
PushNum(2)        // X=2
RclReg(0)         // X=n, Y=2  (RclReg forces lift)
XySwap            // X=2, Y=n
Test(XLeY)        // condition: X <= Y  →  2 <= n  →  TRUE when n >= 2
Gto("P")          // executed when condition is TRUE → declares PRIME
```

`Test(XLeY)` skips `Gto("P")` only when `2 > n` (i.e. n < 2). For every n >= 2
(including 4, 6, 9, 100, ...) the program immediately jumps to label "P" and
returns 1 (prime). Trial division is never reached for any input >= 2.

**Fix:** Remove the `XySwap` so that X=n and Y=2, then use `TestKind::XLeY` (n <= 2
→ prime) or keep the swap and use `TestKind::XGeY` (2 >= n → n <= 2):

```rust
Op::PushNum(HpNum::from(2i32)),
Op::RclReg(0),
// After RclReg: X=n, Y=2
// No XySwap needed
Op::Test(TestKind::XLeY),   // n <= 2 → goto P (prime)
Op::Gto("P".to_string()),
```

---

### CR-05: Quadratic solver description states wrong stack convention

**File:** `hp41-cli/src/programs.rs:45`, `hp41-cli/src/programs.rs:183-188`

**Issue:** The `SampleProgram` description says:

```
"Stack: a(T) b(Z) c(Y) → roots in X,Y"
```

implying the user places `a` in T, `b` in Z, `c` in Y, with X irrelevant. The
program however opens with `Op::StoReg(2)` which stores the **current X** register,
not Y. If the stack is literally `T=a, Z=b, Y=c, X=<garbage>` as described, then:

- `StoReg(2)` → R02 = garbage (X)
- `Rdn` → X=c, Y=b, Z=a
- `StoReg(1)` → R01 = c (comment says b)
- `Rdn` → X=b, Y=a
- `StoReg(0)` → R00 = b (comment says a)

R00=b, R01=c, R02=garbage: the formula computes the wrong result with wrong
coefficients.

The code is only correct if the user enters coefficients such that **c is in X**
at program start (i.e. the actual entry order is "push a, push b, push c with c
ending up in X"). The description must be corrected to match this reality:

**Fix:** Change the description to reflect the actual required entry state:

```rust
description: "Roots of ax²+bx+c. Enter a, b, c (c in X, b in Y, a in Z) → root1 in X, root2 in Y.",
```

And update the inline comment in `quadratic_ops` to match.

---

## Warnings

### WR-01: `handle_reg_modal` silently maps parse overflow to register 0

**File:** `hp41-cli/src/app.rs:442`

**Issue:**

```rust
let reg: u8 = new_acc.parse().unwrap_or(0);
if reg < 100 {
    self.call_dispatch(op_fn(reg));
} else {
    self.message = Some(format!("Invalid register: {new_acc}"));
}
```

`new_acc` is exactly 2 ASCII digits (loop auto-dispatches when `len == 2`), so the
only two-digit strings that fail `u8::parse` are values 256–99 — none, since all two
ASCII digit combinations are 00–99 and fit in u8. However the `unwrap_or(0)` silently
maps a hypothetical parse failure to register 0 (R00), which would store/recall
R00 without user intent. The `< 100` guard then also passes for 0. The defensive
posture should fail loudly rather than silently use R00:

**Fix:**

```rust
match new_acc.parse::<u8>() {
    Ok(reg) if reg < 100 => self.call_dispatch(op_fn(reg)),
    _ => self.message = Some(format!("Invalid register: {new_acc}")),
}
self.pending_input = None;
```

---

### WR-02: `test_program_names_unique` uses `dedup()` without sorting — does not guarantee uniqueness

**File:** `hp41-cli/src/programs.rs:443-447`

**Issue:**

```rust
let mut unique = names.clone();
unique.dedup();
assert_eq!(names.len(), unique.len(), "Program names must be unique");
```

`Vec::dedup()` removes only **consecutive** duplicates. If two programs with the
same name are added in non-adjacent positions (e.g. indices 0 and 5), `dedup`
leaves both and the test passes incorrectly. This is a test-reliability defect: it
cannot catch the bug it is designed to catch.

**Fix:**

```rust
let mut sorted = names.clone();
sorted.sort_unstable();
sorted.dedup();
assert_eq!(names.len(), sorted.len(), "Program names must be unique");
```

---

### WR-03: `mean_sdev_ops` always reads R00 instead of the indexed register — program is non-functional

**File:** `hp41-cli/src/programs.rs:293`

**Issue:** The comment at line 293 acknowledges the defect:

```rust
Op::RclReg(13),
Op::RclReg(0),                      // simplified: use R00 value always
Op::StoArith { reg: 11, kind: StoArithKind::Add },
```

`RclReg(13)` recalls the index counter but discards it immediately (it is pushed
onto the stack and then overwritten by `RclReg(0)`). The program always adds R00
to the running sum, regardless of which iteration is executing. For any input where
R00 is the only non-zero register, it sums the same value n times and divides by
n, returning R00 — only accidentally correct. The program's own description says
"Enter n values" which implies distinct registers, none of which are read.

This sample program is misleading and non-functional as documented. Either the
program should be removed or correctly implemented (using indirect addressing if
supported, or with a clear note that it sums n copies of R00).

**Fix (minimal — correct the description):** Change description to:

```rust
description: "Sums R00 n times (R10=n) then divides. Demonstration only; not a general mean.",
```

Or implement with proper per-register access using R13 as an index to drive multiple
RclReg calls via a jump table, or remove and replace with a simpler honest example.

---

### WR-04: `'?'` help toggle bypasses ALPHA mode — cannot type '?' into ALPHA register

**File:** `hp41-cli/src/app.rs:149-153`

**Issue:** The `'?'` key check at line 149 fires before the alpha-mode guard at
line 188. In ALPHA mode, pressing `'?'` opens the help overlay instead of
appending `'?'` to `alpha_reg`. This is the same class of routing-order defect as
CR-02, but lower severity since `'?'` is uncommon in HP-41 label names.

**Fix:** Guard the `'?'` handler with `!self.state.alpha_mode`:

```rust
if key.code == KeyCode::Char('?') && !self.state.alpha_mode {
    self.show_help = !self.show_help;
    self.show_programs = false;
    return;
}
```

---

## Info

### IN-01: `HELP_DATA` Registers section has incorrect key labels for STO operations

**File:** `hp41-cli/src/help_data.rs:48-52`

**Issue:** The help table shows:

```
("Shift+R", "STO [nn]",  "Store X … — press S then 2 digits"),
("Shift+R+", "STO+ [nn]","Add X to register nn — press Shift+R+, then 2 digits"),
```

The key column says "Shift+R" but the description says "press S then 2 digits". The
actual key binding for STO is `'S'` (capital S = Shift+S), not `Shift+R`. STO+ /
STO- / STO× / STO÷ are not yet wired to key bindings (marked dead code in
`PendingInput`). The key column is misleading — it should read `"S"` for STO and
`"(unassigned)"` for STO arithmetic variants.

**Fix:** Correct the key labels in `HELP_DATA`:

```rust
("S",          "STO [nn]",  "Store X to register nn (00–99) — press S then 2 digits"),
("(unassigned)","STO+ [nn]","Add X to register nn (Phase 7 polish — not yet bound)"),
// etc.
```

---

### IN-02: `check_autosave` resets `last_save` even on save failure

**File:** `hp41-cli/src/app.rs:82-87`

**Issue:**

```rust
pub fn check_autosave(&mut self) {
    if self.last_save.elapsed() >= Duration::from_secs(30) {
        if let Err(e) = persistence::save_state(&self.state_path, &self.state) {
            self.message = Some(format!("Auto-save failed: {e}"));
        }
        self.last_save = Instant::now(); // reset even on failure
    }
}
```

The comment acknowledges this is intentional ("retry on next 30s tick"). However,
on a persistent I/O error (e.g. disk full), the user sees the error message for one
frame and then loses it — the next autosave attempt only happens 30 seconds later.
This means a disk-full situation produces one warning flash every 30 seconds with no
persistent indicator. Consider leaving `last_save` unchanged on failure so the next
poll cycle (16ms) immediately retries and keeps the error message visible until the
condition resolves. This is a quality concern, not a data-loss risk (the on-exit
save also uses the same path and will also fail if the disk is full).

**Fix (optional):** On save failure, do not reset `last_save`:

```rust
if let Err(e) = persistence::save_state(&self.state_path, &self.state) {
    self.message = Some(format!("Auto-save failed: {e}"));
    // Do NOT reset last_save — immediate retry keeps error visible
} else {
    self.last_save = Instant::now();
}
```

---

### IN-03: `Ctrl+S` in ALPHA mode saves state (acceptable but undocumented)

**File:** `hp41-cli/src/app.rs:137-146`

**Issue:** `Ctrl+S` is intercepted before the `alpha_mode` check, so pressing
`Ctrl+S` in ALPHA mode saves state rather than appending `'s'` (the save path is
already handled by the CONTROL modifier check, so no character is lost). This is
probably desirable behavior but is not mentioned in help text or alpha mode status
message. Low priority — no functional defect.

**Fix (documentation only):** Update the ALPHA mode status bar message or help data
to note that `Ctrl+S` saves even in ALPHA mode.

---

_Reviewed: 2026-05-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
