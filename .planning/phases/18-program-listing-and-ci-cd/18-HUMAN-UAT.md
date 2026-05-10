---
status: partial
phase: 18-program-listing-and-ci-cd
source: [18-VERIFICATION.md]
started: 2026-05-10T22:00:00.000Z
updated: 2026-05-10T22:00:00.000Z
---

## Current Test

SC-1, SC-2, SC-3 approved during Plan 18-04 checkpoint. SC-5 requires push to GitHub.

## Tests

### 1. PRGM mode panel visual appearance (SC-1)
expected: Dark panel appears below keyboard in PRGM mode with step rows and green-highlighted current step
result: approved (2026-05-10 — Plan 18-04 checkpoint)

### 2. SST advances highlighted step (SC-2)
expected: F7 or SVG SST click advances pc, highlight moves, auto-scroll follows
result: approved (2026-05-10 — Plan 18-04 checkpoint)

### 3. BST steps backward and clamps at 000 (SC-3)
expected: F8 or SVG BST click decrements pc, clamps at 0 without underflow
result: approved (2026-05-10 — Plan 18-04 checkpoint)

### 4. Live GitHub Actions CI independence (SC-5)
expected: Push to hp41-gui/** triggers two separate workflow runs; ci-gui run executes cargo test before cargo build
result: [pending]

## Summary

total: 4
passed: 3
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
