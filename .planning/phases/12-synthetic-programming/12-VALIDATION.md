---
phase: 12
slug: synthetic-programming
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-08
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | `justfile` — `just test` = `cargo test --workspace` |
| **Quick run command** | `just test` |
| **Full suite command** | `just ci` (lint + test + coverage) |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `just test`
- **After every plan wave:** Run `just ci`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 12-00-* | 00 | 0 | SYNT-01/02/03/04 | — | N/A | unit | `cargo test -p hp41-core synthetic_tests` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | SYNT-01 | — | N/A | unit | `cargo test -p hp41-core synthetic_tests -- getkey` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | SYNT-02 | — | N/A | unit | `cargo test -p hp41-core synthetic_tests -- null_no_op` | ❌ W0 | ⬜ pending |
| 12-01-03 | 01 | 1 | SYNT-03 | — | N/A | unit | `cargo test -p hp41-core synthetic_tests -- sto_rcl_m` | ❌ W0 | ⬜ pending |
| 12-01-04 | 01 | 1 | SYNT-04 | — | N/A | unit | `cargo test -p hp41-core synthetic_tests -- synthetic_byte_exec` | ❌ W0 | ⬜ pending |
| 12-02-01 | 02 | 2 | SYNT-01 | — | N/A | unit | `cargo test -p hp41-cli synthetic -- last_key_code` | ❌ W0 | ⬜ pending |
| 12-02-02 | 02 | 2 | SYNT-03 | — | N/A | unit | `cargo test -p hp41-cli synthetic -- hidden_reg_serde` | ❌ W0 | ⬜ pending |
| 12-02-03 | 02 | 2 | SYNT-04 | — | N/A | integration | `cargo test -p hp41-cli hex_modal -- insert_valid` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/synthetic_tests.rs` — RED stubs for SYNT-01, SYNT-02, SYNT-03, SYNT-04 core-side
  - `test_getkey_pushes_last_key_code`
  - `test_getkey_zero_when_no_key_pressed`
  - `test_null_no_op` (stack/lift/register unchanged)
  - `test_sto_rcl_m`, `test_sto_rcl_n`, `test_sto_rcl_o`
  - `test_hidden_reg_serde` (JSON round-trip)
  - `test_hidden_reg_in_program`
  - `test_synthetic_byte_exec`
  - `test_synthetic_byte_serde`
- [ ] `hp41-cli` test module inline (in `app.rs` test module, following print_modal_tests pattern) — `test_last_key_code_updated`, `test_hex_modal_insert_valid`, `test_hex_modal_reject_invalid`

*Wave 0 tests must be RED (compile but fail) until Wave 1 adds the Op variants.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TUI shows `HEX: _` after pressing X in PRGM mode | SYNT-04 | TUI rendering requires visual inspection | Run `just run`, press `P` to enter PRGM mode, press `X`, verify display shows `HEX: _` |
| TUI shows `HEX: 3_` after typing first hex digit | SYNT-04 | TUI rendering requires visual inspection | Continue above, type `3`, verify display shows `HEX: 3_` |
| Modal closes after valid or invalid code | SYNT-04 | TUI state machine requires visual inspection | Type `A3` (valid) or `FF` (invalid), verify modal closes and program step or INVALID shown |
| STO M / RCL M keyboard path | SYNT-03 | Modal keyboard routing requires visual inspection | Press `S`, then `M`, verify STO M executes and X is stored to reg_m |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
