---
phase: 3
slug: programming-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-07
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test (`cargo test`) + proptest |
| **Config file** | `hp41-core/Cargo.toml` (existing) |
| **Quick run command** | `cargo test -p hp41-core program_tests` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core`
- **After every plan wave:** Run `just ci`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 3-01-01 | CalcState extensions | 1 | PROG-01 | unit | `cargo test -p hp41-core` | ⬜ pending |
| 3-02-01 | HpError::CallDepth | 1 | PROG-01 | unit | `cargo test -p hp41-core` | ⬜ pending |
| 3-03-01 | Op variants + TestKind | 1 | PROG-01 | unit | `cargo test -p hp41-core` | ⬜ pending |
| 3-04-01 | PRGM mode gate in dispatch | 2 | PROG-01 | integration | `cargo test -p hp41-core program_tests` | ⬜ pending |
| 3-05-01 | ops/program.rs (core logic) | 2 | PROG-01 PROG-02 | unit | `cargo test -p hp41-core program_tests` | ⬜ pending |
| 3-06-01 | Dispatch wiring + lib export | 3 | PROG-01 PROG-02 | integration | `cargo test -p hp41-core` | ⬜ pending |
| 3-07-01 | program_tests.rs full suite | 3 | PROG-01 PROG-02 | integration | `just ci` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/program_tests.rs` — stubs for PROG-01 and PROG-02 (fails to compile until plan implementation)

*Existing `just ci` infrastructure covers the full suite.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| None | — | — | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
