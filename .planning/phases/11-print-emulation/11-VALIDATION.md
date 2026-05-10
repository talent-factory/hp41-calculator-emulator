---
phase: 11
slug: print-emulation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-08
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) + cargo llvm-cov for coverage |
| **Config file** | justfile (`just test`, `just ci`, `just coverage`) |
| **Quick run command** | `cargo test --workspace` |
| **Full suite command** | `just ci` (lint + test + coverage ≥80%) |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --workspace`
- **After every plan wave:** Run `just ci`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | PRNT-01 | — | N/A | unit | `cargo test -p hp41-core --test print_tests test_prx_` | ❌ W0 | ⬜ pending |
| 11-01-02 | 01 | 1 | PRNT-01 | — | N/A | unit | `cargo test -p hp41-core --test print_tests test_prx_display_mode` | ❌ W0 | ⬜ pending |
| 11-01-03 | 01 | 1 | PRNT-02 | — | N/A | unit | `cargo test -p hp41-core --test print_tests test_pra_` | ❌ W0 | ⬜ pending |
| 11-01-04 | 01 | 1 | PRNT-03 | — | N/A | unit | `cargo test -p hp41-core --test print_tests test_prstk_` | ❌ W0 | ⬜ pending |
| 11-01-05 | 01 | 1 | PRNT-03 | — | N/A | unit | `cargo test -p hp41-core --test print_tests test_prstk_alpha_` | ❌ W0 | ⬜ pending |
| 11-01-06 | 01 | 1 | PRNT-01/02/03 | — | N/A | unit | `cargo test -p hp41-core --test print_tests test_prx_in_program` | ❌ W0 | ⬜ pending |
| 11-02-01 | 02 | 2 | PRNT-04 | T-11-01 | Path sanitized by OS via OpenOptions; no panic on open failure | integration | `cargo test -p hp41-cli` | ❌ W0 | ⬜ pending |
| 11-02-02 | 02 | 2 | PRNT-04 | T-11-01 | Open failure sets app.message, never panics | unit | `cargo test -p hp41-cli` | ❌ W0 | ⬜ pending |
| 11-02-03 | 02 | 2 | PRNT-01/02/03 | — | N/A | unit | `cargo test -p hp41-cli` | ❌ W0 | ⬜ pending |
| 11-regr | all | — | All | — | serde(default) — v1.0 JSON loads without error | regression | `cargo test -p hp41-core --test program_tests` | ✅ existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/print_tests.rs` — unit tests for PRNT-01 (PRX), PRNT-02 (PRA), PRNT-03 (PRSTK), and program execution arms
- [ ] CLI test module in `hp41-cli/src/app.rs` — `--print-log` file append and open-failure handling (PRNT-04)

*No new test infrastructure needed — cargo test is already wired in the justfile.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `"PRNT: _"` modal display shown during pending print input | D-08 | TUI rendering, no headless test harness | Launch `hp41-cli`, press `P`, verify display area shows `PRNT: _` |
| `"PRSTK → 6 lines"` appears in status bar after PRSTK | D-01 | TUI message field, not testable headlessly | Launch `hp41-cli`, press `P S`, verify status bar message |
| `--print-log <path>` appends lines across multiple sessions | PRNT-04 | File persistence across app restarts | Run `hp41-cli --print-log /tmp/hp.txt`, press `P X`, exit, re-run, press `P A`, verify both lines appended |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
