---
phase: 29
slug: cli-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-17
---

# Phase 29 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: §6 Validation Architecture of `29-RESEARCH.md`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust integration tests (`#[test]` + `cargo test`) — MSRV 1.88 |
| **Config file** | None (Cargo built-in) |
| **Quick run command** | `cargo test -p hp41-cli --tests` |
| **Full suite command** | `just ci` (lint + test + coverage gate) |
| **Estimated runtime** | ~25 seconds for `hp41-cli` test suite; `just ci` ~3 minutes |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-cli --tests`
- **After every plan wave:** Run `just ci`
- **Before `/gsd:verify-work`:** `just ci` must be green AND one Matrix workflow walked end-to-end on a live `cargo run -p hp41-cli` against OM p.14 example
- **Max feedback latency:** ~25 seconds (per-task), ~3 minutes (full)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------|-------------------|-------------|--------|
| 29-01-W0a | 01 | 0 | CLI-02 | JSON file authored — hard-build-blocker per D-25.17 | smoke | `cargo test -p hp41-cli --test phase29_help_data_math1` | ❌ Wave 0 | ⬜ pending |
| 29-01-W0b | 01 | 0 | CLI-01 | resolver chain extended; no infinite recursion | unit | `cargo test -p hp41-cli --test phase25_xeq_by_name cli_resolver_matches_core_resolver` | ✅ extend | ⬜ pending |
| 29-01-W0c | 01 | 0 | CLI-02 | merged accessor enforces single source of truth | unit | `cargo test -p hp41-cli --test function_matrix_parity test_every_math1_rom_op_has_math1_json_entry` | ❌ extend | ⬜ pending |
| 29-01-W0d | 01 | 0 | CLI-02 | reverse parity (no orphan JSON entries) | parity | `cargo test -p hp41-cli --test function_matrix_parity test_every_math1_json_entry_has_xrom_resolver_match` | ❌ extend | ⬜ pending |
| 29-02-W0 | 02 | 1 | CLI-04 | KEY_REF_TABLE derived from merged accessor — no parallel hand-curated table (D-25.18) | unit | `cargo test -p hp41-cli --test phase29_key_ref_includes_math1` | ❌ Wave 0 | ⬜ pending |
| 29-02-compile | 02 | 1 | CLI-03 | exhaustive match on `Op` — no `_ =>` catch-all (FN-CLI-04) | compile-time | `cargo check -p hp41-cli` | ✅ already-shipped | ⬜ pending |
| 29-03-W0a | 03 | 2 | CLI-05 | pending_prompt widening renders `state.modal_prompt` | unit | `cargo test -p hp41-cli --test phase29_pending_prompt_modal` | ❌ Wave 0 | ⬜ pending |
| 29-03-W0b | 03 | 2 | CLI-05 | R/S submit-modal path advances state correctly | integration | `cargo test -p hp41-cli --test phase29_modal_flow matrix_workflow_order_prompt_advances_on_r_s` | ❌ Wave 0 | ⬜ pending |
| 29-03-W0c | 03 | 2 | CLI-05 | auto-open `XeqByName{CollectForModal}` fires when `requires_alpha_label() == true` | integration | `cargo test -p hp41-cli --test phase29_modal_flow solve_workflow_auto_opens_collect_for_modal` | ❌ Wave 0 | ⬜ pending |
| 29-03-W0d | 03 | 2 | CLI-05 | label collection → `submit_modal_with_label` advances Solve from FunctionNamePrompt to Guess1Prompt | integration | `cargo test -p hp41-cli --test phase29_modal_flow solve_workflow_label_submission_advances_to_guess1` | ❌ Wave 0 | ⬜ pending |
| 29-03-W0e | 03 | 2 | CLI-05 | Esc cancels open modal cleanly (D-29.6) | integration | `cargo test -p hp41-cli --test phase29_modal_flow esc_cancels_open_modal` | ❌ Wave 0 | ⬜ pending |
| 29-03-W0f | 03 | 2 | CLI-05 | Two-step Esc — shift_armed first, then modal (precedence) | integration | `cargo test -p hp41-cli --test phase29_modal_flow esc_shift_armed_takes_precedence_over_modal_cancel` | ❌ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `docs/hp41-math1-functions.json` — ~47-55 entries authored per D-29.1 / D-29.2 (Schema mirrors `hp41cv-functions.json` + optional `xrom: { module, module_id, function_id }` per entry). Hard-build-blocker on malformed JSON via `.expect("hp41-math1-functions.json is malformed — fix the JSON")`.
- [ ] `hp41-cli/tests/phase29_help_data_math1.rs` — smoke test for the second `OnceLock`; asserts count ≥ 47, asserts `xrom_module` field populated for every entry.
- [ ] `hp41-cli/tests/phase29_modal_flow.rs` — end-to-end Matrix/Solve modal flows including R/S submit, auto-open `CollectForModal`, label collection, Esc cancellation, shift_armed-vs-modal Esc precedence.
- [ ] `hp41-cli/tests/phase29_pending_prompt_modal.rs` — unit test for the widened `pending_prompt` signature.
- [ ] `hp41-cli/tests/phase29_key_ref_includes_math1.rs` — asserts `key_ref_entries()` after migration to `help_entries_all()` includes Math Pac I rows (e.g., `("XEQ \"SINH\"", "SINH")`).
- [ ] Extension to `hp41-cli/tests/function_matrix_parity.rs` — two new tests: forward parity (every `xrom_resolve` mnemonic has a JSON row) and reverse parity (every Math1 JSON entry resolves via `xrom_resolve`).
- [ ] Extension to `hp41-cli/tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` — adds Math Pac I names (SINH, MATRIX, SOLVE, INTG).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| MATRIX OM p.14 worked example end-to-end | CLI-05 | Multi-step interactive UX with TUI rendering — automated harness can't validate ratatui pixel output | `cargo run -p hp41-cli`, type `XEQ "MATRIX"`, then `2 R/S 1 R/S 2 R/S 3 R/S 4 R/S`, then `XEQ "DET"`, X-register must read -2.0 |
| `prgm_display::format_step` renders `Op::Sinh` as `SINH` in TUI program panel | CLI-03 | Visual rendering verification; CLI program panel layout | `cargo run -p hp41-cli`, enter PRGM mode, key in `XEQ "SINH"`, leave PRGM, verify panel shows `01 SINH` |
| `?`-overlay shows Math Pac I section visually distinct from HP-41CV built-ins | CLI-02 | Ratatui overlay layout — section ordering and grouping | `cargo run -p hp41-cli`, press `?`, scroll to bottom, verify `Math1 *` categories appear and contain expected entries |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies listed above
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references — 7 new/extended test files enumerated
- [ ] No watch-mode flags — all commands are one-shot `cargo test` / `just ci`
- [ ] Feedback latency < 25s per-task quick run
- [ ] `nyquist_compliant: true` set in frontmatter once all Wave 0 tests stubbed and passing

**Approval:** pending
