---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-06
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + proptest 1.11.0 + insta 1.47.2 |
| **Config file** | none — standard `cargo test` discovery |
| **Quick run command** | `cargo test -p hp41-core` |
| **Full suite command** | `cargo llvm-cov --fail-under-lines 80 -p hp41-core` |
| **Estimated runtime** | ~5 seconds (unit tests only) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core`
- **After every plan wave:** Run `cargo llvm-cov --fail-under-lines 80 -p hp41-core`
- **Before `/gsd-verify-work`:** Full suite must be green + `cargo check -p hp41-core` with zero UI deps
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-W0-01 | W0 | 0 | CORE-01 | — | N/A | install | `cargo llvm-cov --version` | ❌ W0 | ⬜ pending |
| 01-W0-02 | W0 | 0 | CORE-01 | — | N/A | scaffold | `cargo check -p hp41-core` | ❌ W0 | ⬜ pending |
| 01-01-01 | 01 | 1 | CORE-01 | — | HpError::Overflow on overflow, not panic | unit | `cargo test -p hp41-core stack` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | CORE-01 | — | No panic on any valid op | unit | `cargo test -p hp41-core enter` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | CORE-01 | — | CLX disables lift | unit | `cargo test -p hp41-core clx` | ❌ W0 | ⬜ pending |
| 01-01-04 | 01 | 1 | CORE-01 | — | CHS neutral lift | unit | `cargo test -p hp41-core chs` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | CORE-02 | — | All ops declare lift effect | unit | `cargo test -p hp41-core lift` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | CORE-02 | — | Stack invariant across random sequences | proptest | `cargo test -p hp41-core proptest` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/src/lib.rs` — public API skeleton with re-exports
- [ ] `hp41-core/src/state.rs` — CalcState, Stack, HpNum type definitions
- [ ] `hp41-core/src/error.rs` — HpError enum (thiserror)
- [ ] `hp41-core/src/num.rs` — HpNum newtype with round_sf(10) enforcement
- [ ] `hp41-core/tests/stack_tests.rs` — CORE-01 unit tests (push, ENTER, CLX, CHS, LASTX)
- [ ] `hp41-core/tests/lift_tests.rs` — CORE-02 lift-effect declarations
- [ ] `hp41-core/tests/proptest_stack.rs` — CORE-02 property tests (no panic, stack invariants)
- [ ] `Justfile` — workspace root with build, test, lint, run, coverage, ci recipes
- [ ] `cargo install cargo-llvm-cov --locked && rustup component add llvm-tools-preview`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `just --list` renders all recipes | Phase SC5 | CLI output, not programmatic | Run `just --list`; verify build, test, lint, run, coverage, ci all appear |
| `just ci` passes on macOS | Phase SC5 | Requires CI-level environment | Run `just ci` end-to-end on developer machine |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING (❌) references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
