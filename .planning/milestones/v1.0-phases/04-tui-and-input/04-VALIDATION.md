---
phase: 4
slug: tui-and-input
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-07
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (unit) + manual TUI testing |
| **Config file** | `hp41-cli/Cargo.toml` |
| **Quick run command** | `cargo check -p hp41-cli` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check -p hp41-cli`
- **After every plan wave:** Run `just ci` + manual smoke test
- **Before `/gsd-verify-work`:** Full suite green + manual TUI verification
- **Max feedback latency:** 10 seconds (cargo check)

---

## Per-Task Verification Map

| Task | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|------|------|------|-------------|-----------|-------------------|--------|
| Cargo.toml deps | 04-01 | 1 | DISP-01 | compile | `cargo check -p hp41-cli` | ⬜ pending |
| App struct | 04-01 | 1 | DISP-01 | compile | `cargo check -p hp41-cli` | ⬜ pending |
| ui.rs layout | 04-02 | 2 | DISP-01 DISP-02 | compile | `cargo check -p hp41-cli` | ⬜ pending |
| keys.rs mapping | 04-03 | 2 | INPUT-01 | unit | `cargo test -p hp41-cli` | ⬜ pending |
| main.rs event loop | 04-04 | 3 | DISP-01 INPUT-01 | compile+manual | `cargo build -p hp41-cli` | ⬜ pending |
| prgm_display.rs | 04-05 | 3 | DISP-02 | compile | `cargo check -p hp41-cli` | ⬜ pending |
| just ci gate | 04-06 | 4 | all | full | `just ci` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-cli/src/lib.rs` — expose `App` struct and key handler for unit tests

*Existing just ci infrastructure covers the full suite.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TUI renders correctly | DISP-01 DISP-02 | Terminal rendering requires human eye | Run `cargo run -p hp41-cli`, verify layout |
| Annunciator updates on mode change | DISP-02 | Requires interaction | Press `d` to cycle angle modes, verify annunciator bar updates |
| Panic hook restores terminal | SC-4 | Requires simulated panic | N/A for Phase 4 automated testing |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
