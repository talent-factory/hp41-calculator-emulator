---
phase: 6
slug: science-and-engineering
status: draft
nyquist_compliant: false
wave_0_complete: true
created: 2026-05-07
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]`) + proptest + insta (already installed) |
| **Config file** | None — standard `cargo test` |
| **Quick run command** | `cargo test -p hp41-core 2>&1 | tail -5` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~5 seconds (just test) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core 2>&1 | tail -5`
- **After every plan wave:** Run `just test`
- **Before `/gsd-verify-work`:** `just ci` (lint + test + coverage ≥ 80%) must be green
- **Max feedback latency:** ~5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | SCI-01 | — | N/A | unit | `cargo test -p hp41-core sigma_plus sigma_minus` | ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 1 | SCI-01 | — | N/A | unit | `cargo test -p hp41-core mean sdev` | ❌ W0 | ⬜ pending |
| 06-01-03 | 01 | 1 | SCI-01 | — | N/A | unit | `cargo test -p hp41-core lr yhat corr` | ❌ W0 | ⬜ pending |
| 06-01-04 | 01 | 1 | SCI-01 | — | N/A | unit | `cargo test -p hp41-core cl_sigma_stat stats_lift` | ❌ W0 | ⬜ pending |
| 06-02-01 | 02 | 2 | SCI-02 | — | N/A | unit | `cargo test -p hp41-core hms_to_h h_to_hms hms_add hms_sub` | ❌ W0 | ⬜ pending |
| 06-02-02 | 02 | 2 | SCI-02 | — | N/A | unit | `cargo test -p hp41-core hms_invalid hms_negative` | ❌ W0 | ⬜ pending |
| 06-03-01 | 03 | 3 | SCI-01, SCI-02 | — | N/A | unit | `just test` | ✅ existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/stats_tests.rs` — stubs for SCI-01 (Σ+, Σ−, MEAN, SDEV, L.R., YHAT, CORR, CLΣSTAT, lift effects)
- [ ] `hp41-core/tests/hms_tests.rs` — stubs for SCI-02 (HMS→, →HMS, HMS+, HMS−, validation, negative)
- [ ] Update `hp41-core/src/tests.rs` — add `InvalidInput` to `hperror_has_four_variants` and `hperror_display_messages` tests after adding the variant to `error.rs`

*Note: Existing test infrastructure (`just test`, proptest, insta) covers the framework; only new test files need to be added.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TUI key bindings render correct label in help overlay | SCI-01, SCI-02 | Visual layout check | Launch TUI with `just run`, press `?`, verify all 12 new ops appear with correct key labels |
| KEY_REF_TABLE displays Σ+/Σ−/MEAN/SDEV/L.R./YHAT/CORR/CLΣSTAT/HMS→/→HMS/HMS+/HMS− | INPUT-01 | TUI rendering | Launch TUI, observe key reference panel at bottom |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
