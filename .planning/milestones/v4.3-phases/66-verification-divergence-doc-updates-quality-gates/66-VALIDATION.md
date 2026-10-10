---
phase: 66
slug: verification-divergence-doc-updates-quality-gates
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 66 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `66-RESEARCH.md` → "Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` + `cargo llvm-cov` for coverage |
| **Config file** | `justfile` (tasks: `just test`, `just coverage`, `just ci`, `just ci-msrv`, `just gui-ci`) |
| **Quick run command** | `cargo test -p hp41-core` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~60–120 seconds (`just ci`); MSRV + gui-ci add several minutes |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core` (quick)
- **After every plan wave:** Run `just ci`
- **Before `/gsd:verify-work`:** `just ci && just ci-msrv && just gui-ci` all green
- **Max feedback latency:** ~120 seconds for the quick/`just ci` loop

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 66-UNC02-error | 01 | 0 | VERIFY-01(b) | — | N/A (fidelity gate, no security surface) | unit | `cargo test -p hp41-core --test error_tests` (or inline error.rs Display test) | ❌ W0 (new `HpError::NonExistent` + Display test) | ⬜ pending |
| 66-UNC02-gate | 01 | 1 | VERIFY-01(b) | — | PRX/PRA/PRSTK error with `NonExistent` when flags 21 AND 55 both clear | unit | `cargo test -p hp41-core --test print_tests` | ✅ (rework: set flag 55 in setup) | ⬜ pending |
| 66-UNC02-newtest | 01 | 1 | VERIFY-01(b) | — | `op_prx` on default state returns `Err(NonExistent)` | unit | `cargo test -p hp41-core --test print_tests -- test_prx_returns_nonexistent` | ❌ W0 (new test fn) | ⬜ pending |
| 66-UNC01-verify | 02 | 1 | VERIFY-01(a) | — | Back-arrow clears `app.message` (already-correct; OM p.15) | unit (optional regression) | `cargo test -p hp41-cli` | ✅ (code already correct; optional new test) | ⬜ pending |
| 66-UNC03-comment | 02 | 1 | VERIFY-01(c) | — | SIZE reduction silent — no MEMORY LOST (already-correct; OM p.19) | none (comment-only) | `cargo test -p hp41-core` (existing SIZE tests stay green) | ✅ | ⬜ pending |
| 66-matrix-proof | 03 | 1 | VERIFY-01 | — | 7 PITFALLS → 12 test-fn mapping documented; suite green | unit | `cargo test -p hp41-core --test phase_63_interrupting_alarms` | ✅ (12 tests already pass; zero gaps) | ⬜ pending |
| 66-gates | NN | last | VERIFY-01 | — | Full gate suite green incl. ungated GUI clippy | composite | `just ci && just ci-msrv && just gui-ci && cargo clippy --manifest-path hp41-gui/src-tauri/Cargo.toml` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/src/error.rs` — add `HpError::NonExistent` variant + Display test (follow the `file_not_found_display` pattern). **Blocks the UNC-02 `print.rs` guard** — must land before any code uses the variant.
- [ ] `hp41-core/tests/print_tests.rs` — Wave-1 rework of all 18 existing tests to set `flag_set(state.flags, 55)` in setup; add `test_prx_returns_nonexistent_when_no_printer_flag`.

*No new framework install needed — existing Rust test + llvm-cov infra covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| UNC dispositions cite primary-OM pages | VERIFY-01 | OM page citations are read-and-record evidence, not a runtime assertion (D-01/D-02 autonomous verification) | Each UNC verification record + closure ledger row carries an `HP-41C OM p.NN` citation; reviewer cross-checks against `docs/manuals/HP-41CV/HP-41C_Operating_Manual.pdf` |
| Divergence-doc sweep reflects shipped reality | VERIFY-01 / D-05 | Prose/doc correctness is reviewed, not unit-tested | Diff each swept doc (DIVERGENCE-AUDIT.md, hp41cv-divergences.md, hp41-time-divergences.md, hp41-math1-divergences.md, README.md) against the closure ledger |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`HpError::NonExistent` + Display test before print.rs guard)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s (quick loop)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
