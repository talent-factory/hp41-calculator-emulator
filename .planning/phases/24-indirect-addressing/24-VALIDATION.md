---
phase: 24
slug: indirect-addressing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 24 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Detailed test matrix lives in `24-RESEARCH.md` §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust stdlib test harness; integration tests in `hp41-core/tests/`, unit tests in `#[cfg(test)] mod tests` blocks) |
| **Config file** | None — `[lib]` and `[[test]]` sections of `hp41-core/Cargo.toml` |
| **Quick run command** | `just test -p hp41-core --test phase24_resolve_indirect --test phase24_ind_variants` |
| **Full suite command** | `just test` |
| **Coverage command** | `just coverage` (cargo-llvm-cov, gate at 80% for hp41-core; 92.5% baseline) |
| **Estimated runtime** | ~2 s per-task (quick) / ~30 s full / ~60 s coverage |

---

## Sampling Rate

- **After every task commit:** Run quick run command
- **After every plan wave:** Run `just test -p hp41-core` (Wave 1) / `just test` (Wave 2)
- **Before `/gsd-verify-work`:** Full suite + `just coverage` must be green (≥92.5% for `hp41-core`)
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

> See `24-RESEARCH.md` §"Validation Architecture" for the full ~50-test matrix
> with exact test names, files, and expected pass/fail signals. The planner is
> instructed to copy each row into the corresponding task's `<acceptance_criteria>`
> as a `cargo test` assertion.

Summary:

| Plan | Wave | Tests Added | New Files | Coverage Targets |
|------|------|-------------|-----------|------------------|
| 24-01 | 1 | ~7 unit (inline in `indirect.rs`) + 4 sentinel additions to `phase22_program_control.rs` | `hp41-core/src/ops/indirect.rs`, `hp41-core/tests/phase24_resolve_indirect.rs` | `resolve_indirect_decimal`, `resolve_indirect`, refactored `Op::GtoInd` / `Op::XeqInd` arms |
| 24-02 | 2 | ~33 integration tests | `hp41-core/tests/phase24_ind_variants.rs` | All 11 new IND variants × {happy, non-integer, out-of-bounds where applicable} |

---

## Wave 0 Requirements

- [ ] No new framework install — `cargo test` is already wired
- [ ] No new fixtures — existing `CalcState::default()` + `state.regs[i] = HpNum::from(...)` setup pattern (used in `phase22_program_control.rs` and `phase23_arcl_asto.rs`) is reused

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | — | All Phase 24 behaviors are deterministic library calls — fully automatable via `cargo test` | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify (cargo test commands in `<acceptance_criteria>`) or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (none — existing harness)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30 s
- [ ] `nyquist_compliant: true` set in frontmatter (after planner adds tasks and gsd-plan-checker confirms)

**Approval:** pending
