---
status: partial
phase: 12-synthetic-programming
source: [12-VERIFICATION.md]
started: 2026-05-09T00:00:00Z
updated: 2026-05-09T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. GETKEY key code tracking end-to-end
expected: Press a key (e.g. digit '5'), run a program containing GETKEY — X should show the HP-41 row-column code for '5' (key 35 = row 3 × 10 + col 5)
result: [pending]

### 2. HexModal valid byte insertion — program listing
expected: In PRGM mode, press X, type CF → program listing shows `SYN CF` (or equivalent GETKEY/NULL label) at the current step
result: [pending]

### 3. HexModal invalid byte — INVALID message
expected: In PRGM mode, press X, type 00 → display area shows `INVALID`, program Vec unchanged
result: [pending]

### 4. Uppercase 'X' outside PRGM mode — silently ignored
expected: Press uppercase X (Shift+X) in RUN mode → no HexModal opens, no key_to_op fallthrough, no state change
result: [pending]

### 5. S then M dispatches StoM immediately
expected: With X=42 on stack, press S then M → reg_m shows 42, modal closes
result: [pending]

### 6. Esc mid-HexModal cancels cleanly
expected: Press X (opens HexModal), type one hex digit, then Esc → modal closes, program unchanged, no INVALID message
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps
