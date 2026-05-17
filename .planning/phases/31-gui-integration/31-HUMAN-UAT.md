---
status: partial
phase: 31-gui-integration
source: [31-VERIFICATION.md]
started: 2026-05-17T22:55:00Z
updated: 2026-05-17T22:55:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Matrix-entry modal UX round-trip
expected: `just gui-dev`, press XEQ, type `MATRIX`, press Enter. At `ORDER=?` prompt, type `3`, press Enter (R/S). Continue through `A1,1=?`, `A1,2=?`, ... `A3,3=?` entries. Then invoke `XEQ DET`. Press Esc mid-sequence to verify cancellation. LCD shows each prompt (`ORDER=?`, `A1,1=?`, etc.) while waiting for input; once user starts typing, entry_buf overrides the prompt on LCD (LCD-alternation). R/S advances each step. Esc cancels the workflow and returns to normal calculator display. DET result displays after full entry.
result: [pending]

### 2. INTG cancellation timing
expected: `just gui-dev`, set up a slow integral (e.g. `XEQ INTG` with a computationally intensive function). During computation, press R/S or Esc to cancel. Computation cancels within 100ms. LCD renders `CANCELED` (uppercase). After cancellation, pressing INTG/SOLVE/DIFEQ again initiates a clean new run (sticky-cancel is absent — flag was reset at workflow opener entry).
result: [pending]

### 3. Help overlay two-section visual layout
expected: `just gui-dev`, press `?` to open the help overlay. Observe the two top-level section headings. Click "HP-41CV (built-in)" heading to collapse it. Click "Math 1 Pac (XROM 7)" to expand/collapse. Math1 categories (Math1 Hyperbolics, Math1 Complex, Math1 Matrix, etc.) appear as 2nd-level headers within the XROM 7 section.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
