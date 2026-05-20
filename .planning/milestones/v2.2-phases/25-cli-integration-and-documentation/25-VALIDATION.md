---
phase: 25
slug: cli-integration-and-documentation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-14
---

# Phase 25 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Detailed validation architecture lives in `25-RESEARCH.md` §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `cargo clippy` + (new) `docs-matrix` regen-check |
| **Config file** | `justfile` (recipes), `Cargo.toml` (deps), `.github/workflows/ci.yml` |
| **Quick run command** | `just check` (per-task — fmt + clippy + test on hp41-cli) |
| **Full suite command** | `just ci` (workspace fmt + clippy + test + coverage + msrv) |
| **Estimated runtime** | quick ~30s · full ~5–7 min |

---

## Sampling Rate

- **After every task commit:** `just check` on hp41-cli (zero-warning gate)
- **After every plan wave:** `just ci` (full workspace) + `just docs-matrix-check` (drift catch)
- **Before `/gsd-verify-work`:** `just ci` AND `just gui-ci` (Phase 26 will rely on Phase 25 wiring; smoke-check parity) must both be green
- **Max feedback latency:** ~30 s on the quick path; ~5 min on the full path

---

## Per-Task Verification Map

> Plans are not yet decomposed — researcher recommends 4 plans (see RESEARCH.md §Recommended Plan Decomposition). This table will be filled by the planner; rows are illustrative anchors.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 25-01-01 | 01 | 0 | FN-CLI-01 | — | `App.shift_armed` state field; v1.x letter bindings removed; status bar shows `f→` | unit | `cargo test -p hp41-cli f_prefix_` | ❌ W0 | ⬜ pending |
| 25-01-02 | 01 | 1 | FN-TEST-01 (partial — 4 keyboard tests) | — | `f -` → X=Y; `f +` → X≤Y; `f *` → X>Y; `f /` → X=0 dispatched | unit | `cargo test -p hp41-cli f_shifted_conditionals` | ❌ W0 | ⬜ pending |
| 25-02-01 | 02 | 0 | FN-CLI-02 | — | New `PendingInput::FlagPrompt { kind, ind, acc }` + `RegisterPrompt { op, ind, acc }` + ClpLabel/DelCount/TonePrompt/XeqByName variants exist | unit | `cargo test -p hp41-cli pending_input_variants` | ❌ W0 | ⬜ pending |
| 25-02-02 | 02 | 1 | FN-CLI-02 | — | IND modifier toggles via shift-0 inside an open modal; dispatch picks `Op::*Ind` vs `Op::*` at digit completion | unit | `cargo test -p hp41-cli ind_modal_toggle` | ❌ W0 | ⬜ pending |
| 25-02-03 | 02 | 2 | FN-CLI-04 | — | `pending_prompt()` exhaustive match — no `_ =>` arm, no `unreachable!()`; compile-check passes | compile | `cargo build -p hp41-cli` | ✅ existing | ⬜ pending |
| 25-03-01 | 03 | 0 | FN-TEST-01 (full) | — | hp41-core `builtin_card_op` extended 4→12 mnemonics (XNeY, XLtY, XGeY, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero added) | unit | `cargo test -p hp41-core builtin_card_op_12` | ❌ W0 | ⬜ pending |
| 25-03-02 | 03 | 1 | FN-TEST-01 (full) | — | New CLI XEQ-by-Name modal dispatches all 12 conditional-test mnemonics; keystroke flow `XEQ → ALPHA-input → ENTER` works | integration | `cargo test -p hp41-cli xeq_by_name_conditionals` | ❌ W0 | ⬜ pending |
| 25-04-01 | 04 | 0 | FN-DOC-01 | — | `docs/hp41cv-functions.json` exists with ≥130 entries; schema matches D-25.16 | data | `cargo test -p hp41-cli functions_json_schema` | ❌ W0 | ⬜ pending |
| 25-04-02 | 04 | 1 | FN-CLI-03 | — | `help_data.rs` loads JSON via `include_str!` + `OnceLock<Vec<HelpEntry>>`; every entry has matching `key_path` or `xeq_only` flag | unit | `cargo test -p hp41-cli help_entries_load` | ❌ W0 | ⬜ pending |
| 25-04-03 | 04 | 1 | FN-DOC-01 | — | `scripts/docs-matrix/` Rust bin emits matrix.md; `just docs-matrix` + `just docs-matrix-check` recipes | regen-test | `just docs-matrix-check` | ❌ W0 | ⬜ pending |
| 25-04-04 | 04 | 2 | FN-DOC-01 + FN-CLI-01 | — | Bidirectional CI parity test: every JSON entry has matching `Op::` variant (or is `⏳ v3.x`); every `Op::` variant has matrix entry (or is in INTERNAL_OP_VARIANTS skip-list) | unit | `cargo test -p hp41-core op_matrix_parity` | ❌ W0 | ⬜ pending |
| 25-04-05 | 04 | 3 | FN-DOC-02 + FN-DOC-03 + FN-DOC-04 | — | CLAUDE.md "v2.2 additions" block; README "feature-complete HP-41CV" soft claim with matrix link; rustdoc cross-refs verified | manual + grep | `grep -c "v2.2 additions" CLAUDE.md && grep -c "hp41cv-function-matrix" README.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-cli/tests/phase25_keyboard.rs` — stubs for keyboard wiring tests (key_to_op coverage, KEY_REF_TABLE row count ≥ post-migration target)
- [ ] `hp41-cli/tests/phase25_pending_input.rs` — stubs for `PendingInput` variant tests (6 new variants + IND toggle behavior)
- [ ] `hp41-cli/tests/phase25_xeq_by_name.rs` — stubs for the new XEQ-by-Name CLI modal
- [ ] `hp41-cli/tests/phase25_help_data.rs` — stubs for JSON-loaded help_entries() correctness
- [ ] `hp41-core/tests/phase25_builtin_card_op.rs` — stubs for the 4→12 extension (the only hp41-core test in Phase 25; surgical extension cleared by user)
- [ ] `scripts/docs-matrix/Cargo.toml` + `scripts/docs-matrix/src/main.rs` — standalone non-workspace bin scaffold (researcher's recommendation)
- [ ] `docs/hp41cv-functions.json` — empty `[]` placeholder so `include_str!` doesn't break the build during early waves
- [ ] No new test framework — `cargo test` is the workspace standard; no install step

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TUI prefix indicator (`f→`) renders in status bar at the right slot | FN-CLI-01 | Cosmetic check; ratatui status-row layout is verified visually, not asserted | Run `just run-cli`, press `f`, confirm `f→` appears bottom-right; press any non-prefix key, confirm indicator clears |
| ALPHA-overrides-prefix divergence (D-25.5) feels right in interactive use | D-25.5 | UX assessment, not assertable | Enter ALPHA mode (`Ctrl+A`), press `f`, confirm `f` types literal `F`; exit ALPHA, press `f`, confirm prefix arms |
| HP-41CV keyboard binding "feels" hardware-faithful per user's physical CV | D-25.6 + D-25.7 | Hardware-faithfulness is a subjective gate; user owns final say | User runs CLI side-by-side with their physical HP-41CV; spot-check 10 random ops from the QRG; flag any divergence |
| README soft-claim wording reads naturally | FN-DOC-03 + D-25.17 | Copy review | User reads the updated README "feature-complete HP-41CV" section; approves wording before commit |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30 s (quick) / 7 min (full)
- [ ] `nyquist_compliant: true` set in frontmatter after planner fills the per-task map

**Approval:** pending
