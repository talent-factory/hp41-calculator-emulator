---
phase: 21
slug: flags-display-control-sound
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) + proptest 1.x (property tests) |
| **Config file** | `Cargo.toml` workspace + `hp41-core/tests/` integration suite |
| **Quick run command** | `just test-core` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~12 s (core unit) / ~35 s (full workspace incl. clippy + fmt) |

---

## Sampling Rate

- **After every task commit:** Run `just test-core` (Phase 21 unit tests only — ~3 s)
- **After every plan wave:** Run `just ci` (workspace tests + clippy + fmt)
- **Before `/gsd-verify-work`:** Full suite + `just coverage` (≥80% on hp41-core) must be green
- **Max feedback latency:** 12 seconds

---

## Per-Task Verification Map

> Populated by gsd-planner once PLAN.md files exist. Each task gets a row; placeholders below.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 21-01-01 | 01 | 0 | FN-FLAG-01 | T-21-01 | flags field is `u64` with `#[serde(default)]` — old save files load with flags=0 | unit | `just test-core -- flags::tests::serde_backward_compat` | ❌ W0 | ⬜ pending |
| 21-01-02 | 01 | 1 | FN-FLAG-01 | — | SF/CF/FS?/FC?/FS?C/FC?C round-trip a flag 0..55 | unit | `just test-core -- flags::tests::flag_ops_round_trip` | ❌ W0 | ⬜ pending |
| 21-02-01 | 02 | 2 | FN-FLAG-02 | T-21-02 | FS? in running program skips next step on flag clear | integration | `just test-core --test flag_skip_next_step` | ❌ W0 | ⬜ pending |
| 21-03-01 | 03 | 1 | FN-DISP-01..05 | — | VIEW/AVIEW/PROMPT populate `display_override`; CLD clears it | unit | `just test-core -- display::tests::view_aview_prompt_cld` | ❌ W0 | ⬜ pending |
| 21-04-01 | 04 | 1 | FN-SOUND-01/02 | — | BEEP/TONE push event lines into print_buffer; no I/O in hp41-core | unit | `just test-core -- sound::tests::beep_tone_buffer` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/src/ops/flags.rs` — module file with `op_sf`, `op_cf`, `op_fs_test`, `op_fc_test`, `op_fs_test_clear`, `op_fc_test_clear` (+ #[cfg(test)] mod tests)
- [ ] `hp41-core/src/ops/sound.rs` — module file with `op_beep`, `op_tone` (+ tests)
- [ ] `hp41-core/tests/phase21_flags.rs` — integration suite covering skip-next-step semantics and v1.x save-file load (load fixture autosave.json captured before flags existed)
- [ ] `hp41-core/tests/fixtures/v20-autosave.json` — minimal v2.0-era save file fixture for backward-compat regression
- [ ] No new framework installs needed — `cargo test` + `proptest` already in `[dev-dependencies]`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `VIEW 03` text visible in TUI until next key | FN-DISP-01 | Visual rendering — ratatui frame composition can't be asserted in unit tests | `just run` → press digits to load `3` → `STO 03` → `S 03 ENTER` (or VIEW binding) → verify `R03=…` shows until any key |
| `VIEW 03` text visible in GUI LCD | FN-DISP-01 | Tauri webview rendering — outside Rust test boundary | `just gui-dev` → same sequence as above; verify LCD shows override |
| TUI prints `[BEEP]`/`[TONE n]` event lines | FN-SOUND-01/02 | Print-buffer consumer is hp41-cli — verified at workspace level but easier visually | `just run` → enter `BEEP` then `TONE 5` from program-mode → confirm event lines in print pane |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 12 s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
