---
phase: 65-standalone-fidelity-fixes
plan: "05"
subsystem: display
tags: [disp-04, lcd, 14-segment, scientific, large-exponent, gui, uat-followup]
dependency_graph:
  requires: [65-01]
  provides: [DISP-04]
  affects:
    - hp41-core/src/format.rs
    - hp41-core/src/lib.rs
    - hp41-gui/src-tauri/src/types.rs
tech_stack:
  added: []
  patterns:
    - "reflow-only-on-overflow: format_hpnum_lcd returns format_hpnum unchanged when <=12 cells"
    - "shared round_sci_parts helper (format_sci_large + format_hpnum_lcd)"
    - "cell count ignores the folded decimal point (mirrors Display14Seg.tsx)"
key_files:
  created:
    - docs/adr/v4.3-006-lcd-12cell-scientific.md
    - hp41-core/tests/phase65_lcd_display.rs
  modified:
    - hp41-core/src/format.rs
    - hp41-core/src/lib.rs
    - hp41-gui/src-tauri/src/types.rs
decisions:
  - "65-05-D01: Discriminator is OUTPUT WIDTH (>12 cells), not HpNum.exponent != 0 — FACT(27)=1.088886945E28 is below the Decimal ceiling (~7.92E28) so it has exponent==0 yet still overflows the display in scientific form. The first TDD red test exposed this."
  - "65-05-D02: GUI main display only. CLI status line + GUI stack panel keep the wide 'E' form (not 12-cell constrained). Deliberate scope limit, not a D-25.6 parity violation."
  - "65-05-D03: Right-justified exponent variant (UAT user choice 2026-06-07) — positive exp keeps full 10 sig digits (mantissa + exponent butt together flush); negative exp drops to 9 sig digits to make room for the sign."
metrics:
  completed_date: "2026-06-07"
  tasks_completed: 4
  files_modified: 3
  files_created: 2
  origin: "UAT follow-up (MATH-01 GUI)"
---

# Phase 65 Plan 05: DISP-04 — Authentic 12-Cell LCD for Scientific Overflow

## One-Liner

The GUI 14-segment display (12 cells) truncated the exponent of large scientific values
(`1.088886945E 28` = 14 cells), caught during MATH-01 UAT of `FACT(27)`. New
`format_hpnum_lcd` drops the "E" and right-justifies the exponent so the full value fits
12 cells; wired into the GUI main display only.

## What Was Built (TDD)

### Task 1 — Core formatter (`hp41-core/src/format.rs`, `lib.rs`)

- Extracted `round_sci_parts(mantissa, base_exp, digits) -> (String, i32, bool)` — the
  shared mantissa rounding + carry logic, now used by both `format_sci_large` (wide "E"
  form) and the new LCD formatter.
- Added `pub fn format_hpnum_lcd(n, mode)`: renders via `format_hpnum`, returns it
  unchanged when `<= 12` cells; otherwise splits the scientific string, drops the "E",
  re-rounds the mantissa to fit, and right-justifies the exponent (sign only when
  negative). Re-exported from `lib.rs`.

### Task 2 — GUI wiring (`hp41-gui/src-tauri/src/types.rs`)

- `from_state` `display_str` X fallback now uses `format_hpnum_lcd`. `x_str`/`y_str`/…
  (stack panel) keep `format_hpnum` (wide "E" form). No IPC/App.tsx/Display14Seg change —
  the fix is purely the string that already feeds the existing 12-cell component.

### Task 3 — Tests (TDD: red → green)

- `hp41-core/tests/phase65_lcd_display.rs` (7): `FACT(27)`/`FACT(69)` fit exactly 12
  cells, no "E", exponent right-justified, negative exponent reserves the sign cell
  (via SCI 9), small SCI value unchanged, in-range value delegates to `format_hpnum`.
- `types.rs::test_large_exponent_x_fits_12_cell_lcd` — GUI `from_state` produces
  `1.08888694528` (12 cells, no "E") for a large X.

### Task 4 — ADR

- `docs/adr/v4.3-006-lcd-12cell-scientific.md` (Accepted).

## Result Examples

| Input | Before (truncated) | After (12 cells) |
|-------|--------------------|------------------|
| FACT(27) | `1.088886945E` (exp cut) | `1.08888694528` |
| FACT(69) | `1.711224524E` (exp cut) | `1.71122452498` |
| 1.23e-5 @ SCI 9 | `1.23000000E` (exp cut) | `1.23000000-05` |

## Commits

Atomic, via `/git-workflow:commit --with-skills` (English). See `git log` for hashes:

1. `feat(65-05)` — core `format_hpnum_lcd` + `round_sci_parts` refactor + re-export
2. `feat(65-05)` — GUI `from_state` 12-cell LCD display for large exponents
3. `test(65-05)` — LCD 12-cell display tests + ADR v4.3-006
4. `docs(65-05)` — 65-05 plan/summary, 65-UAT approval, STATE.md update

## Verification

- `cargo test -p hp41-core --test phase65_lcd_display`: 7 passed
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --lib types`: 19 passed
- `just test`: all workspace suites green (no regression; `format_sci_large` refactor
  guarded by existing format tests)
- clippy (MSRV 1.88 workspace + GUI crate): 0 NEW warnings (2 pre-existing GUI hits
  `commands.rs:620` / `types.rs:186` unchanged)

## Deviations from Plan

The discriminator was corrected mid-implementation after the first TDD red test
(65-05-D01): `FACT(27)` has `exponent == 0` (it fits in `Decimal`) yet still overflows
the display, so the trigger is output width `> 12 cells`, not `HpNum.exponent != 0`.

## Threat Flags

None — read-only display formatting of an existing core-owned value; no new network,
auth, file, or parse surface.

## Self-Check: PASSED

- `hp41-core/src/format.rs` `format_hpnum_lcd`: FOUND
- `hp41-core/src/lib.rs` re-export: FOUND
- `hp41-gui/src-tauri/src/types.rs` uses `format_hpnum_lcd`: FOUND
- `hp41-core/tests/phase65_lcd_display.rs`: 7 tests, all green
- `docs/adr/v4.3-006-lcd-12cell-scientific.md`: FOUND
