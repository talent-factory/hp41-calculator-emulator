---
phase: 5
slug: persistence-and-ux
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-07
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) + proptest for property tests |
| **Config file** | none — standard cargo test discovery |
| **Quick run command** | `cargo test -p hp41-core && cargo test -p hp41-cli` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core && cargo test -p hp41-cli`
- **After every plan wave:** Run `just ci`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~15 seconds

---

## Per-Task Verification Map

| Req ID | Behavior | Test Type | Automated Command | File Exists | Status |
|--------|----------|-----------|-------------------|-------------|--------|
| PERS-01 | save_state() writes valid JSON with version wrapper | unit | `cargo test -p hp41-cli -- persistence::tests` | ❌ Wave 0 | ⬜ pending |
| PERS-01 | load_state() round-trips CalcState exactly | unit | `cargo test -p hp41-cli -- persistence::tests::test_roundtrip` | ❌ Wave 0 | ⬜ pending |
| PERS-01 | load_state() on missing file returns Err (not panic) | unit | `cargo test -p hp41-cli -- persistence::tests::test_missing_file` | ❌ Wave 0 | ⬜ pending |
| PERS-01 | load_state() on corrupt JSON returns Err (not panic) | unit | `cargo test -p hp41-cli -- persistence::tests::test_corrupt_file` | ❌ Wave 0 | ⬜ pending |
| PERS-01 | HpNum serializes as string, not float | unit | `cargo test -p hp41-core -- num::tests::test_hpnum_serde` | ❌ Wave 0 | ⬜ pending |
| PERS-01 | user_mode + key_assignments survive save/load | unit | `cargo test -p hp41-cli -- persistence::tests::test_user_mode_roundtrip` | ❌ Wave 0 | ⬜ pending |
| PERS-02 | App.last_save updates after auto-save | manual | manual smoke — observe status bar "Saved" after 31s | ❌ manual | ⬜ pending |
| PERS-02 | State file timestamp changes within 35s of start | manual | check file mtime after 31s of idle use | ❌ manual | ⬜ pending |
| UX-01 | HELP_DATA contains entries for all key categories | unit | `cargo test -p hp41-cli -- help_data::tests::test_all_categories_present` | ❌ Wave 0 | ⬜ pending |
| UX-01 | help table scrolls next/previous without panic | unit | `cargo test -p hp41-cli -- ui::tests::test_help_scroll` | ❌ Wave 0 | ⬜ pending |
| UX-02 | Op::UserMode flips state.user_mode | unit | `cargo test -p hp41-core -- ops::tests::test_user_mode_toggle` | ❌ Wave 0 | ⬜ pending |
| UX-02 | USER mode dispatches assigned label via run_program | unit | `cargo test -p hp41-cli -- keys::tests::test_user_mode_dispatch` | ❌ Wave 0 | ⬜ pending |
| UX-03 | SAMPLE_PROGRAMS has ≥10 entries | unit | `cargo test -p hp41-cli -- programs::tests::test_program_count` | ❌ Wave 0 | ⬜ pending |
| UX-03 | Each sample program's ops list is non-empty | unit | `cargo test -p hp41-cli -- programs::tests::test_programs_non_empty` | ❌ Wave 0 | ⬜ pending |
| UX-03 | Fibonacci program runs to completion without panic | unit | `cargo test -p hp41-cli -- programs::tests::test_fibonacci_runs` | ❌ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-cli/src/persistence.rs` — `save_state`, `load_state`, `default_state_path` + inline tests
- [ ] `hp41-core/src/num.rs` — add `test_hpnum_serde` (Serialize/Deserialize round-trip for HpNum)
- [ ] `hp41-cli/src/help_data.rs` — `HELP_DATA` static + category count test
- [ ] `hp41-cli/src/programs.rs` — `SampleProgram`, `sample_programs()`, ≥10 programs + count test
- [ ] `hp41-core/src/ops/` — test for `Op::UserMode` and `Op::AlphaBackspace` dispatch

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Auto-save fires at 30s interval | PERS-02 | Requires running TUI loop with real time | Start `cargo run`, idle 31s, check file mtime |
| Help overlay renders correctly (80%×90% centered) | UX-01 | Visual rendering in TUI | Run app, press `?`, verify overlay appears centered |
| Program library overlay renders list | UX-03 | Visual rendering in TUI | Run app, press `Ctrl+P`, verify 10+ entries |
| USER mode assignment persists through save/reload | UX-02 | Requires full TUI interaction flow | Assign key, Ctrl+S, quit, reload, verify assignment active |
