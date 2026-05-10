---
status: partial
phase: 10-STO-Arithmetic-Modals
source: [10-VERIFICATION.md]
started: 2026-05-08
updated: 2026-05-08
---

## Current Test

[awaiting human testing]

## Tests

### 1. Modal prompt display after S
expected: Press `S` → status bar shows `STO [__]` (or similar step-1 prompt)
result: [pending]

### 2. STO+ numbered register (S → + → 0 → 5)
expected: With a value in X (e.g. 3), press `S`, `+`, `0`, `5` → R05 becomes R05+3; X unchanged; modal dismissed
result: [pending]

### 3. STO- stack Y register (S → - → Y)
expected: Press `S`, `-`, `Y` → Y register becomes Y−X; X unchanged; modal dismissed
result: [pending]

### 4. Esc at step 2 cancels cleanly
expected: Press `S`, `+`, `Esc` → no state change; modal dismissed; no register modified
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
