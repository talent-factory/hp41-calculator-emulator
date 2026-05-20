---
status: complete
phase: 29-cli-integration
source:
  - 29-01-SUMMARY.md
  - 29-02-SUMMARY.md
  - 29-03-SUMMARY.md
started: 2026-05-17T14:30:59Z
updated: 2026-05-17T15:05:00Z
completed: 2026-05-17T15:05:00Z
---

## Current Test

(all tests complete — see Summary)

## Tests

### 1. XEQ-by-name resolves Math Pac I function (SINH)
expected: |
  Launch `cargo run -p hp41-cli`. Put a non-zero value into X first (e.g. type `1` then Enter). Then press `f` (arms shift) followed by uppercase `N` — the XEQ-by-name modal opens. Type `SINH` and press Enter. The hyperbolic sine of X is computed (with X=1 the display reads ~1.1752).
result: pass
note: Display read `1.17520` — matches expected within rounding.

### 2. `?`-overlay shows Math Pac I section
expected: |
  In the running TUI, press `?` to open the help overlay. Scroll until you see a distinct `Math1 *` category band (e.g., `Math1 Hyperbolic`, `Math1 Complex`, `Math1 Matrix`). All 45 Math Pac I entries appear under those Math1 categories — not mixed into the v2.2 built-ins list.
result: pass

### 3. Right-panel key-reference includes Math Pac I
expected: |
  With the TUI running, look at the right-hand reference panel. It lists XEQ-by-name entries from the Math Pac I pool (e.g., `XEQ "SINH"`, `XEQ "MATRIX"`, `XEQ "INTG"`) alongside the v2.2 built-in keys — no hand-curated parallel list, just one merged view.
result: pass

### 4. Program listing shows Math Pac I mnemonic
expected: |
  Press `p` to enter PRGM mode (annunciator `[PRGM]` lights up). Open XEQ-by-name via `f N`, type `SINH`, press Enter — the step is recorded. The LCD shows the just-recorded step `NN SINH` (where NN is the step number). Use F8 (BST) and F7 (SST) to walk back and forth; each step displays the authentic mnemonic `SINH` — not a raw `Op::Sinh` form, not a generic XROM number.
result: pass
note: Display read `066 SINH` after recording (program was 66 steps; SINH appended at index 66, pc parked at END so display auto-updated). Authentic mnemonic confirmed.

### 5. MATRIX modal flow (ORDER=? → A1,1=?)
expected: |
  Launch `cargo run -p hp41-cli`. Press `f`, then `N`, type `MATRIX`, press Enter. Status bar shows `ORDER=?`. Type `2` and press F5 (R/S). Status bar advances to `A1,1=?`. Type a value and press F5 — it advances to the next cell (`A1,2=?` or similar). Pressing Esc shows `Cancelled` in the status bar.
result: pass

### 6. SOLVE auto-opens XEQ name collection
expected: |
  Press `f`, then `N`, type `SOLVE`, press Enter. Status bar shows `FUNCTION NAME?`. The XEQ-by-name collection mode auto-opens (you can type a label name, e.g., `F`, and press Enter). The modal advances to `GUESS 1=?`.
result: pass
note: Status bar showed `FUNCTION NAME?` after SOLVE + Enter. `maybe_auto_open_collect_for_modal` post-dispatch hook firing correctly. (Initial confusion was a stale FIX modal from test 5 leftover — once cleared with Esc, sequence worked as expected.)

### 7. Esc cancels modal cleanly
expected: |
  Open any Math Pac I modal (e.g., `f N INTG Enter`). Press Esc. The modal closes, status bar shows `Cancelled` momentarily, and the calculator returns to its normal idle state — no leaked `modal_prompt`, no frozen state.
result: pass

## Summary

total: 7
passed: 7
issues: 0
pending: 0
skipped: 0

## Gaps

None. All 7 user-facing CLI deliverables for Phase 29 (Math Pac I CLI Integration) verified end-to-end on a live `cargo run -p hp41-cli` session.

Pre-existing HP-41-hardware divergence noted during test 4 (program-recording does NOT auto-advance `pc` after `state.program.push(op)` in `dispatch()` at `hp41-core/src/ops/mod.rs:877`) — out of Phase 29 scope (this behavior pre-dates Phase 29 and affects all PRGM-mode recording, not just Math Pac I ops). Captured here for future v3.x consideration.
