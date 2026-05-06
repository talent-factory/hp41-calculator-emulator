---
phase: 2
slug: core-math
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-06
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + proptest 1.11 + insta 1.47 |
| **Config file** | none — cargo discovers tests automatically |
| **Quick run command** | `cargo test -p hp41-core` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core`
- **After every plan wave:** Run `just test` (full workspace including proptest)
- **Before `/gsd-verify-work`:** `just ci` must be green (lint + test + coverage ≥ 80%)
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 2-01-01 | 01 | 1 | MATH-01 | — | N/A | unit | `cargo test -p hp41-core math_ops` | ❌ Wave 0 | ⬜ pending |
| 2-01-02 | 01 | 1 | MATH-01 | — | N/A | unit | `cargo test -p hp41-core math_ops::test_ypow` | ❌ Wave 0 | ⬜ pending |
| 2-01-03 | 01 | 1 | MATH-01 | — | N/A | unit+snapshot | `cargo test -p hp41-core math_ops::test_ln_accuracy` | ❌ Wave 0 | ⬜ pending |
| 2-01-04 | 01 | 1 | MATH-01 | — | N/A | unit | `cargo test -p hp41-core lift_tests` | ❌ Wave 0 | ⬜ pending |
| 2-02-01 | 02 | 1 | MATH-02 | — | N/A | unit | `cargo test -p hp41-core trig_tests::test_sin_deg` | ❌ Wave 0 | ⬜ pending |
| 2-02-02 | 02 | 1 | MATH-02 | — | N/A | unit | `cargo test -p hp41-core trig_tests::test_asin_deg` | ❌ Wave 0 | ⬜ pending |
| 2-02-03 | 02 | 1 | MATH-02 | — | N/A | unit | `cargo test -p hp41-core trig_tests::test_sin_rad` | ❌ Wave 0 | ⬜ pending |
| 2-02-04 | 02 | 1 | MATH-02 | — | N/A | unit | `cargo test -p hp41-core trig_tests::test_tan_grad` | ❌ Wave 0 | ⬜ pending |
| 2-03-01 | 03 | 2 | MATH-03 | — | N/A | unit | `cargo test -p hp41-core format_tests::test_fix4` | ❌ Wave 0 | ⬜ pending |
| 2-03-02 | 03 | 2 | MATH-03 | — | N/A | unit | `cargo test -p hp41-core format_tests::test_sci4` | ❌ Wave 0 | ⬜ pending |
| 2-03-03 | 03 | 2 | MATH-03 | — | N/A | unit | `cargo test -p hp41-core format_tests::test_eng3` | ❌ Wave 0 | ⬜ pending |
| 2-03-04 | 03 | 2 | MATH-03 | — | N/A | unit | `cargo test -p hp41-core format_tests::test_fix_overflow` | ❌ Wave 0 | ⬜ pending |
| 2-04-01 | 04 | 2 | REGS-01 | — | N/A | unit | `cargo test -p hp41-core register_tests` | ❌ Wave 0 | ⬜ pending |
| 2-04-02 | 04 | 2 | REGS-01 | — | N/A | unit | `cargo test -p hp41-core register_tests::test_sto_add` | ❌ Wave 0 | ⬜ pending |
| 2-04-03 | 04 | 2 | REGS-01 | — | N/A | unit | `cargo test -p hp41-core lift_tests` (extend) | ❌ Wave 0 | ⬜ pending |
| 2-05-01 | 05 | 2 | ALPH-01 | — | N/A | unit | `cargo test -p hp41-core alpha_tests` | ❌ Wave 0 | ⬜ pending |
| 2-05-02 | 05 | 2 | ALPH-01 | — | N/A | unit | `cargo test -p hp41-core alpha_tests::test_24_char_limit` | ❌ Wave 0 | ⬜ pending |
| 2-05-03 | 05 | 2 | ALPH-01 | — | N/A | unit | `cargo test -p hp41-core alpha_tests::test_alpha_clear` | ❌ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/math_tests.rs` — unit tests for MATH-01 (all 13 unary math ops, LASTX, stack-lift)
- [ ] `hp41-core/tests/trig_tests.rs` — unit tests for MATH-02 (SIN/COS/TAN/ASIN/ACOS/ATAN × DEG/RAD/GRAD modes)
- [ ] `hp41-core/tests/format_tests.rs` — unit tests for MATH-03 (FIX/SCI/ENG, overflow edge cases)
- [ ] `hp41-core/tests/register_tests.rs` — unit tests for REGS-01 (STO/RCL/STO+/-/×/÷, lift semantics)
- [ ] `hp41-core/tests/alpha_tests.rs` — unit tests for ALPH-01 (AlphaAppend/AlphaClear/24-char limit)
- [ ] Extend `hp41-core/tests/lift_tests.rs` — add lift assertions for all ~24 new op variants

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| FIX overflow exact threshold | MATH-03 | Needs cross-reference with V41 emulator or HP-41 hardware | Enter 9.9999E14 in FIX 4; confirm display switches to SCI |
| CHS behavior during EEX entry | MATH-01 | Complex 3-state keyboard interaction | Enter "1 EEX 3 CHS" and verify exponent becomes -3 not mantissa flip |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
