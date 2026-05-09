---
phase: 15
slug: display-and-keyboard
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-09
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in tests (cargo test) — hp41-gui/src-tauri |
| **Config file** | none (standard cargo test) |
| **Quick run command** | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| **Full suite command** | `just gui-check && cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| **Estimated runtime** | ~5 seconds |

> **Frontend testing:** No frontend test framework is installed (vitest/jest). New npm deps are forbidden by D-10 (vanilla CSS, no new deps). Frontend UI changes (App.tsx, CSS) are verified manually via `just gui-dev` against SC-1 through SC-5.

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **After every plan wave:** Run `just gui-check && cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Before `/gsd-verify-work`:** Full suite must be green + manual SC-1 through SC-5 check
- **Max feedback latency:** ~5 seconds (Rust tests only)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 0 | DISP-01 | N/A | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- types::tests` | ✅ extend existing | ⬜ pending |
| 15-01-02 | 01 | 0 | IPC-02 | No panic on unknown key | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- commands::tests` | ❌ Wave 0 new | ⬜ pending |
| 15-02-01 | 02 | 1 | DISP-01 | CalcStateView JSON ≤300 bytes | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- types::tests::test_dispatch_op_payload_size` | ✅ existing | ⬜ pending |
| 15-02-02 | 02 | 1 | IPC-02 | EEX-CHS toggles entry_buf | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- commands::tests::test_eex_chs` | ❌ Wave 0 new | ⬜ pending |
| 15-03-01 | 03 | 2 | DISP-01/DISP-02 | Display + stack panel visible | manual (GUI) | `just gui-dev` → verify SC-1, SC-2, SC-3 | ❌ manual only | ⬜ pending |
| 15-03-02 | 03 | 2 | IPC-02 | Keyboard drives all bindings | manual (GUI) | `just gui-dev` → verify SC-4, SC-5 | ❌ manual only | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-gui/src-tauri/src/types.rs` — Extend `types::tests::test_calc_state_view_structure` to assert `y_str`, `z_str`, `t_str`, `lastx_str`, `in_eex_mode` fields exist on CalcStateView — covers DISP-01/DISP-02
- [ ] `hp41-gui/src-tauri/src/commands.rs` — Add `commands::tests::test_eex_chs_toggles_exponent_sign`: send `"eex_chs"` to `handle_op` with `entry_buf = "1e2"` → assert entry_buf = `"1e-2"`; second call → `"1e2"` — covers IPC-02
- [ ] `hp41-gui/src-tauri/src/commands.rs` — Add `commands::tests::test_eex_chs_noop_without_e`: send `"eex_chs"` with no 'e' in entry_buf → assert no panic, returns `Ok(view)` — defensive guard

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Display panel shows 12-char display_str on dark background | DISP-01/SC-1 | No frontend test framework (D-10 no new npm deps) | `just gui-dev` → press `3`, `+`, `ENTER`; verify display updates within one frame |
| All 5 annunciators update visually on mode toggle | DISP-01/SC-2 | No frontend test framework | `just gui-dev` → press `u` (USER mode); verify USER annunciator lights |
| Stack panel shows X/Y/Z/T/LASTX after each op | DISP-02/SC-3 | No frontend test framework | `just gui-dev` → press `3`, `ENTER`, `4`; verify X=4, Y=3 in stack panel |
| No duplicate IPC calls in React StrictMode | IPC-02/SC-4 | Requires runtime observation | `just gui-dev` → press one key; verify only 1 dispatch_op call fires (check DevTools network) |
| Key binding set matches hp41-cli key_to_op() | IPC-02/SC-5 | Requires manual key-by-key sweep | `just gui-dev` → press each binding in key_to_op(); verify all produce correct state change |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING (❌) references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s (Rust tests ≤5s; manual UI verification is separate)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
