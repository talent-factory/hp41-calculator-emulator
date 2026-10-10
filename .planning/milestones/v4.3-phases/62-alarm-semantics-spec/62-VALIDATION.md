---
phase: 62
slug: alarm-semantics-spec
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-06
---

# Phase 62 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> **Phase 62 is documentation-only** (a single ADR). There are zero runtime code
> changes, so the validation reduces to "CI stays green" + manual review of the
> contract against the OM-confirmed research finding.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test suite (`cargo test` via `just ci`) |
| **Config file** | Workspace `Cargo.toml` (existing infra) |
| **Quick run command** | `just ci` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | existing CI runtime (unchanged — no new tests) |

---

## Sampling Rate

- **After every task commit:** `just ci` must stay green (proves docs-only, zero code drift).
- **After every plan wave:** `just ci`.
- **Before `/gsd:verify-work`:** `just ci` green.
- **Max feedback latency:** existing CI runtime.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 62-01-01 | 01 | 1 | ALARM-01 | — / — | N/A (docs-only — no runtime behavior) | manual review + CI-green guard | `just ci` | ✅ (existing) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No test framework install,
no test stubs, no fixtures — the deliverable is an ADR Markdown file
(`docs/adr/v4.3-003-*.md`) that is neither compiled nor executed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ADR states the OM-confirmed prefix mapping unambiguously (`>>` = interrupting, `>` = conditional/deferred), in observable-behavior terms, with the primary-OM citation | ALARM-01 (SC#1) | Prose/decision content — not machine-checkable | Read `docs/adr/v4.3-003-*.md`: confirm an implementer reading ONLY the ADR cannot reproduce the `>`/`>>` confusion; confirm the citation (HP 82182A Time Module QRC 82182-90002 and/or HP-41CX QRG 00041-90475 p.32) is present |
| Audit verdict recorded: current `parse_alarm_type` (`>>`→`interrupting:true`) is CONFIRMED CORRECT (not inverted); D-05 correction plan documented as NOT-triggered | ALARM-01 (SC#2) | Cross-doc reasoning vs code | Confirm the ADR cites the actual `alarm.rs` lines and states the no-flip verdict + the conditional-correction constraints carried for the record |
| Edge cases covered: 4-level call-stack cap, idle-fire behavior, non-interrupting `alarm:xeq:` path unchanged | ALARM-01 (SC#3) | Spec prose | Confirm all three edge-case sections are present in the ADR |
| `just ci` green — zero runtime code changes | ALARM-01 (SC#4) | — | `just ci` |

---

## Validation Sign-Off

- [x] All tasks have automated verify (`just ci` green-guard) or are manual-only by nature (docs-only phase)
- [x] Sampling continuity: single task; no 3-consecutive-without-verify risk
- [x] Wave 0 covers all MISSING references (none — existing infra)
- [x] No watch-mode flags
- [x] Feedback latency = existing CI runtime
- [x] `nyquist_compliant: true` set in frontmatter (docs-only phase: automated guard is CI-green, behavior verification is manual ADR review)

**Approval:** pending
