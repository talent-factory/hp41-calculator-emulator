---
phase: 18
slug: program-listing-and-ci-cd
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust native tests (`#[test]`), TypeScript `npx tsc --noEmit` |
| **Config file** | None (Rust inline tests in `mod tests {}` blocks) |
| **Quick run command** | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| **Full suite command** | `just test && cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml && cd hp41-gui && npx tsc --noEmit` |
| **Estimated runtime** | ~30 seconds (Rust compile + test + tsc) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **After every plan wave:** Run full suite command above
- **Before `/gsd-verify-work`:** Full suite green + human SC-1..SC-5 verification
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 18-01-01 | 01 | 0 | PROG-01 | — | handle_sst bounds-check prevents pc overflow | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- test_handle_sst` | ❌ W0 | ⬜ pending |
| 18-01-02 | 01 | 0 | PROG-01 | — | handle_bst saturating_sub prevents underflow | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- test_handle_bst` | ❌ W0 | ⬜ pending |
| 18-01-03 | 01 | 0 | PROG-01 | — | format_all_steps returns ["000 END"] for empty program | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- test_format_all_steps` | ❌ W0 | ⬜ pending |
| 18-01-04 | 01 | 0 | PROG-01 | — | CalcStateView serializes with program_steps and pc | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- test_phase18_fields` | ❌ W0 | ⬜ pending |
| 18-02-01 | 02 | 1 | PROG-01 | — | sst_step/bst_step permissions granted in capabilities | manual | Verify capabilities/default.json contains allow-sst-step, allow-bst-step | ✅ | ⬜ pending |
| 18-02-02 | 02 | 1 | PROG-01 | — | TypeScript compiles without errors | integration | `cd hp41-gui && npx tsc --noEmit` | ✅ | ⬜ pending |
| 18-02-03 | 02 | 1 | PROG-01 | — | Program listing appears in PRGM mode, SST/BST highlight correct step | manual (SC-1–SC-3) | Human verify | manual | ⬜ pending |
| 18-03-01 | 03 | 1 | SC-4/SC-5 | — | CI job runs on all 3 platforms without error | integration | GitHub Actions ci-gui.yml | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-gui/src-tauri/src/commands.rs` — add test stubs: `test_handle_sst_advances_pc`, `test_handle_sst_clamps_at_end`, `test_handle_bst_decrements_pc`, `test_handle_bst_clamps_at_zero`
- [ ] `hp41-gui/src-tauri/src/prgm_display.rs` — add test stubs: `test_format_all_steps_empty_program`, `test_format_all_steps_nonempty`
- [ ] `hp41-gui/src-tauri/src/types.rs` — add test stub: `test_phase18_fields_exist` (verifies `program_steps` and `pc` are in `CalcStateView`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Program listing appears below keyboard when PRGM mode active | PROG-01 / SC-1 | Visual/interactive — Tauri window cannot be headlessly tested | Launch app, press PRGM key, verify listing panel appears below keyboard |
| SST advances highlighted step by one | PROG-01 / SC-2 | Visual + interaction | In PRGM mode with a loaded program, press F7 or click SST key; verify highlighted row advances |
| BST moves highlighted step backward and listing scrolls | PROG-01 / SC-3 | Visual + scroll behavior | In PRGM mode, press F8 or click BST key; verify highlighted row goes back and scrolls into view |
| GUI CI job passes on all 3 platforms | SC-4 | Cross-platform — requires GitHub Actions runners | Push to develop; verify ci-gui.yml passes on macOS, Windows, Ubuntu |
| GUI CI is independent from CLI CI | SC-5 | Requires separate workflow configuration | Verify ci-gui.yml and ci.yml are separate files; introduce a deliberate GUI error and confirm only ci-gui fails |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
