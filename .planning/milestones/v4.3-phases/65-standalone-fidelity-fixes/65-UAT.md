---
status: passed
phase: 65-standalone-fidelity-fixes
source: [65-01-SUMMARY.md, 65-02-SUMMARY.md, 65-03-SUMMARY.md, 65-04-SUMMARY.md, 65-HUMAN-UAT.md]
started: 2026-06-07T17:10:27Z
updated: 2026-06-07T18:20:00Z
approved: 2026-06-07T18:20:00Z
---

## Key Legend (CLI)

Start the CLI with `just run` (GUI: `just gui-dev`). CLI keystrokes used below:

| Action | Key(s) |
|--------|--------|
| Digits / decimal | `0`–`9`, `.` |
| ENTER↑ | Enter |
| CHS (sign) | `n` (lowercase) |
| EEX (exponent) | `e` (lowercase) |
| STO modal | `S` (Shift+s) → register digits → Enter |
| VIEW | `f` then `v` → register digits → Enter |
| ALPHA on/off | `a` |
| XEQ-by-Name modal | `N` (Shift+n) → type the function name → Enter |

**IMPORTANT — XEQ-by-Name names must be UPPERCASE.** The resolver only matches
`AON` / `AOFF` / `AVIEW` / `PROMPT` / `FACT`, and the modal stores letters exactly
as typed. Hold **Shift** (or Caps Lock) while typing the name, otherwise the CLI
shows an `InvalidOp` toast.

## Current Test

number: 1
name: DISP-01 — CLI VIEW/AVIEW/PROMPT display override
steps: |
  1. `just run`
  2. Put 42 into R01: type `4` `2`, then `S` (Shift+s), then `0` `1`, then Enter.
  3. Put a DIFFERENT value into X so the override is distinguishable: type `7`, then Enter.
  4. VIEW R01: press `f`, then `v`, then `0` `1`, then Enter.
  5. (optional, AVIEW) press `a`, type `A` `B`, press `a`; then `N`, type `AVIEW`, Enter.
expected: |
  After step 4 the CLI main display line shows the VIEWed register's value `42.0000`
  (the override) even though X holds 7 — it refreshes immediately after Enter.
  (Optional step 5: the display shows the ALPHA string `AB`.)
  Clear check: type any digit (e.g. `9`) — the override disappears and normal X
  display returns. (CLD / a fresh VIEW also clear it.)
awaiting: none — approved by user 2026-06-07

## Tests

### 1. DISP-01 — CLI VIEW/AVIEW/PROMPT display override
steps: |
  1. `just run`
  2. `4` `2` → `S` → `0` `1` → Enter   (STO 42 into R01)
  3. `7` → Enter                        (X = 7, so override ≠ X)
  4. `f` → `v` → `0` `1` → Enter        (VIEW R01)
  5. (optional AVIEW) `a` → type `A` `B` → `a` → `N` → type `AVIEW` → Enter
expected: After step 4 the CLI main display line shows `42.0000` (the VIEWed register value / the override), NOT the X value 7, and refreshes right after the keypress. Typing a digit (e.g. `9`) clears the override and X display returns. (Optional: step 5 shows the ALPHA string `AB`.)
result: pass

### 2. DISP-02 — CHS in-buffer mantissa sign toggle
steps: |
  A. In-buffer toggle: type `3` `.` `1` `4`, then `n`, then `n` again.
  B. Empty-buffer CHS: type `5`, Enter, then `n`.
  C. EEX exponent sign: type `1`, `e`, `2`, then `n`.
expected: |
  A. The entry flips IN PLACE: `3.14` → `-3.14` → `3.14`, with no stack lift, no
     overwrite, no flush (you are still editing the same number).
  B. With an empty entry buffer, `n` negates X: `5` → `-5`.
  C. `n` toggles the EXPONENT sign: `1e2` → `1e-2` (EEX-CHS path unchanged).
result: pass

### 3. DISP-03 — AON flag-48 auto-display (CLI + GUI)
steps: |
  CLI (`just run`):
  1. Load ALPHA: `a` → type `A` `B` `C` → `a`   (exit ALPHA mode; alpha_reg = "ABC")
  2. AON: `N` → type `AON` (uppercase) → Enter   (sets flag 48)
  3. Run an op: `2` → Enter → `3` → `+`          (X becomes 5)
  4. AOFF: `N` → type `AOFF` (uppercase) → Enter
  GUI (`just gui-dev`): load ALPHA text, run AON via the `?` overlay → "All Functions"
  tab (tap-to-run), do an op, then run AOFF the same way.
expected: |
  After step 2, at rest the CLI display shows `ABC` (the ALPHA register), and it
  KEEPS showing `ABC` after the op in step 3 (auto-display after every operation),
  NOT the result 5. After step 4 (AOFF) the display reverts to X (`5.0000`).
  A fresh VIEW still wins over AON. GUI behaves identically (CLI↔GUI parity).
result: pass

### 4. MATH-01 — FACT(27..69) scientific-notation range
steps: |
  Each: type the number, then `N`, type `FACT` (uppercase), then Enter.
  - `2` `7`  → `N` `FACT` Enter
  - `6` `9`  → `N` `FACT` Enter
  - `7` `0`  → `N` `FACT` Enter
  - `3` `.` `5` → `N` `FACT` Enter
  - `5` `n` (CHS → -5) → `N` `FACT` Enter
expected: |
  - FACT(27) ≈ 1.088886945E28   (10 significant digits, scientific notation)
  - FACT(69) ≈ 1.711224524E98   (matches the HP-41 Owner's Manual table)
  - FACT(70) → OutOfRange error (status-bar toast)
  - FACT(3.5) → Domain error (non-integer)
  - FACT(-5) → Domain error (negative)
result: pass

## Summary

total: 4
passed: 4
issues: 0
pending: 0
skipped: 0
blocked: 0

approved_by: user
approved_at: 2026-06-07

## Gaps

### Found & fixed during UAT (MATH-01, GUI)

- **GUI 14-segment exponent truncation** — testing `FACT(27)` in the GUI revealed
  the 12-cell 14-segment display truncated the exponent of large scientific values
  (`1.088886945E 28` = 14 cells → exponent cut off). The math result was correct;
  only the GUI rendering overflowed. **Fixed (Option A, HP-41-authentic):** new
  `format_hpnum_lcd` (drops the "E", right-justifies the exponent, fits 12 cells),
  wired into `types.rs::from_state` for the GUI main display only. CLI / stack panel
  keep the wide "E" form. See `docs/adr/v4.3-006-lcd-12cell-scientific.md`.
  Tests: `hp41-core/tests/phase65_lcd_display.rs` (7) + `types.rs` GUI test.
  Status: implemented + green; **pending commit** (suggest as plan 65-05).
