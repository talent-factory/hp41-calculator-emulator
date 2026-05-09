---
phase: 14
slug: ipc-layer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-09
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[test]` (standard library — no extra crate) |
| **Config file** | `hp41-gui/src-tauri/Cargo.toml` (nested workspace, not root) |
| **Quick run command** | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| **Full suite command** | `just test && cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **After every plan wave:** Run `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Before `/gsd-verify-work`:** Full suite + `just gui-check` must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 14-W0-01 | 00 | 0 | IPC-01/SC-1,2,3 | — | test stubs catch regressions | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | ❌ Wave 0 | ⬜ pending |
| 14-01-01 | 01 | 1 | IPC-01/SC-4 | T-14-01 | no hp41-core logic in hp41-gui | compile | `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` | ❌ Wave 0 | ⬜ pending |
| 14-01-02 | 01 | 1 | IPC-01/SC-1 | T-14-02 | CalcStateView JSON ≤300 bytes | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_dispatch_op_payload_size` | ❌ Wave 0 | ⬜ pending |
| 14-02-01 | 02 | 1 | IPC-01/SC-2 | T-14-03 | unknown key → GuiError, no panic | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_dispatch_op_unknown_key` | ❌ Wave 0 | ⬜ pending |
| 14-02-02 | 02 | 1 | IPC-01/SC-3 | T-14-04 | print_buffer drain on every call | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_print_buffer_drained` | ❌ Wave 0 | ⬜ pending |
| 14-03-01 | 03 | 2 | IPC-01/SC-5 | T-14-05 | AppState = Mutex<CalcState> compile-enforced | compile | `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-gui/src-tauri/src/types.rs` — inline `#[cfg(test)]` block with `test_dispatch_op_payload_size`, `test_calc_state_view_structure`
- [ ] `hp41-gui/src-tauri/src/key_map.rs` — inline `#[cfg(test)]` block with `test_key_map_named_ops`, `test_key_map_unknown_key`, `test_key_map_compound_keys`
- [ ] `hp41-gui/src-tauri/src/commands.rs` — inline `#[cfg(test)]` block with `test_dispatch_op_unknown_key`, `test_print_buffer_drained`

*No new test framework needed — standard Rust `#[test]` works.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SC-4: Zero calculator logic in hp41-gui | IPC-01/SC-4 | Static audit — no automated check for architectural intent | `grep -r "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` should return empty |
| Capability permissions auto-generated | IPC-01 (D-12) | Permission IDs only appear in `gen/schemas/` after first build | After `just gui-check`, inspect `hp41-gui/src-tauri/gen/schemas/desktop-schema.json` for `"allow-dispatch-op"` and `"allow-get-state"` IDs, then update `capabilities/default.json` accordingly |

---

## Threat Model

| Threat | STRIDE | Mitigation | In Plan? |
|--------|--------|------------|----------|
| Arbitrary command invocation from injected JS | Elevation of Privilege | Tauri capabilities window scoping (`"windows": ["main"]`); no wildcard | Phase 14 caps update |
| Panic in command handler crashes app | Denial of Service | `#![deny(clippy::unwrap_used)]`; poisoned-lock recovery; GuiError instead of panic | All plans |
| Unknown key ID silently discarded | Information Disclosure | D-07: ALL unknown key IDs return GuiError, never silently ignored | key_map.rs |
| entry_buf corruption via crafted key_id | Tampering | key_map.rs only appends valid single chars; no eval/exec path | key_map.rs |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
