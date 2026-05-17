---
phase: 31
slug: gui-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-17
---

# Phase 31 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo test (Rust, `hp41-core` + `hp41-gui/src-tauri`) + Vitest 1.x (`hp41-gui/src/`) + WebdriverIO 9.x + tauri-driver 2.0.6 (E2E, Ubuntu only) |
| **Config file** | `Cargo.toml` workspaces; `hp41-gui/vite.config.ts`; `hp41-gui/wdio.conf.cjs` |
| **Quick run command** | `just test` (cargo test, all workspaces) |
| **Full suite command** | `just gui-ci` (cargo test + clippy + fmt + `cd hp41-gui && npm test`) |
| **Estimated runtime** | ~60s quick / ~5 min full / ~10 min with E2E |

---

## Sampling Rate

- **After every task commit:** Run `just test` (and `cd hp41-gui && npm test` for `hp41-gui/src/` edits)
- **After every plan wave:** Run `just gui-ci`
- **Before `/gsd:verify-work`:** Full suite must be green; E2E smoke must pass on Ubuntu CI
- **Max feedback latency:** ~60 seconds for quick; ~5 min for full

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 31-01-01 | 01 | 1 | GUI-01 | — | SC-4 grep on `hp41-gui/src-tauri/src/` returns nothing | grep | `! grep -rn "fn op_\\(add\\|sub\\|mul\\|div\\|sin\\|cos\\|tan\\|sto\\|rcl\\|flush_entry\\|format_hpnum\\)" hp41-gui/src-tauri/src/` | ✅ existing pattern | ⬜ pending |
| 31-01-02 | 01 | 1 | GUI-01 | — | All Phase 28 `Op` variants have an `op_display_name` arm in `hp41-gui/src-tauri/src/prgm_display.rs` | unit | `cargo test -p hp41-gui-tauri prgm_display_math1_arms` | ❌ W0 | ⬜ pending |
| 31-02-01 | 02 | 1 | GUI-05 | — | `scripts/check-tauri-permissions.sh` exists & exits 0; all `generate_handler!` commands have permission TOMLs | shell | `bash scripts/check-tauri-permissions.sh` | ❌ W0 | ⬜ pending |
| 31-02-02 | 02 | 2 | GUI-05 | — | `request_cancel` Tauri command sets `cancel_requested` AtomicBool without acquiring AppState Mutex | unit | `cargo test -p hp41-gui-tauri cancel_command_no_deadlock` | ❌ W0 | ⬜ pending |
| 31-02-03 | 02 | 2 | GUI-05 | — | `op_integ`/`op_solve`/`op_difeq` check `cancel_requested` every 64 samples and return `HpError::Canceled` | unit | `cargo test -p hp41-core test_integ_cancellation` | ❌ W0 | ⬜ pending |
| 31-02-04 | 02 | 2 | GUI-05 | — | Auto-save thread + `request_cancel` interleave without deadlock under stress | integration | `cargo test -p hp41-gui-tauri --test cancel_autosave_stress` | ❌ W0 | ⬜ pending |
| 31-02-05 | 02 | 2 | GUI-05 | — | `cancel_requested` is reset on workflow-opener entry (idempotency) | unit | `cargo test -p hp41-core test_cancel_flag_reset_on_open` | ❌ W0 | ⬜ pending |
| 31-03-01 | 03 | 2 | GUI-02 | — | XEQ "SINH" 1.5 → 2.1293; identical X-reg value between CLI and GUI dispatch paths (D-25.6 parity) | integration | `cargo test -p hp41-gui-tauri --test cli_gui_parity` | ❌ W0 | ⬜ pending |
| 31-03-02 | 03 | 2 | GUI-02 | — | XEQ-by-name modal in `App.tsx` constructs `Op::Xeq(name)` payload routed through `dispatch_op` | unit (vitest) | `cd hp41-gui && npm test -- App.xeq_modal` | ❌ W0 | ⬜ pending |
| 31-03-03 | 03 | 2 | GUI-02 | — | `submit_modal` and `cancel_modal` Tauri thunks registered with permission TOMLs | shell | `bash scripts/check-tauri-permissions.sh` | ❌ W0 (covered by 31-02-01) | ⬜ pending |
| 31-04-01 | 04 | 2 | GUI-03 | — | `?`-overlay imports BOTH `hp41cv-functions.json` AND `hp41-math1-functions.json` via Vite | unit (vitest) | `cd hp41-gui && npm test -- HelpOverlay.math1_section` | ❌ W0 | ⬜ pending |
| 31-04-02 | 04 | 2 | GUI-03 | — | Math Pac I rendered as a distinct categorized section ("Math 1 Pac") in the overlay | unit (vitest) | `cd hp41-gui && npm test -- HelpOverlay.categories` | ❌ W0 | ⬜ pending |
| 31-04-03 | 04 | 2 | GUI-04 | — | `Op::Catalog(2)` extended from stub → enumerates loaded XROM modules with function counts | unit | `cargo test -p hp41-core test_catalog_2_lists_modules` | ❌ W0 | ⬜ pending |
| 31-05-01 | 05 | 2 | GUI-06 | — | `state.modal_prompt` text rendered in print panel below LCD when active | unit (vitest) | `cd hp41-gui && npm test -- PrintPanel.modal_prompt` | ❌ W0 | ⬜ pending |
| 31-05-02 | 05 | 2 | GUI-06 | — | Number-entry pipeline routes to modal accumulator while modal_program is active; ENTER submits, ESC cancels | unit (vitest) + integration | `cd hp41-gui && npm test -- App.modal_input_pipeline` | ❌ W0 | ⬜ pending |
| 31-05-03 | 05 | 2 | GUI-07 | — | Stub-error arm policy in `key_map::resolve` preserved (does NOT shrink in v3.0) | grep | `grep -n "is planned for a future phase" hp41-gui/src-tauri/src/key_map.rs` | ✅ existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `scripts/check-tauri-permissions.sh` — author NEW (RESEARCH Open Q1 body); CI gate verifies every `generate_handler!` member has a matching permission TOML
- [ ] `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` — assert all Phase 28 `Op` variants have exhaustive `op_display_name` arms
- [ ] `hp41-gui/src-tauri/tests/cancel_command_no_deadlock.rs` — `request_cancel` thunk uses `tauri::State<CancelFlag>` (NOT AppState Mutex)
- [ ] `hp41-core/tests/integ_cancellation.rs` — `op_integ` returns `HpError::Canceled` when AtomicBool is set; every-64-samples granularity verified
- [ ] `hp41-gui/src-tauri/tests/cancel_autosave_stress.rs` — multi-thread interleaving test (auto-save + request_cancel + active INTG)
- [ ] `hp41-gui/src-tauri/tests/cli_gui_parity.rs` — drive SINH/ASINH/TANH/MOD/GAMMA through both paths, assert identical X-register
- [ ] `hp41-gui/src/App.test.tsx` — extend with `xeq_modal` test (vitest)
- [ ] `hp41-gui/src/HelpOverlay.test.tsx` — extend with `math1_section` + `categories` tests (vitest)
- [ ] `hp41-gui/src/PrintPanel.test.tsx` — NEW vitest file for modal_prompt rendering (if no PrintPanel test exists)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Long INTG of `LN(X)` from 0 to 1 cancellable from GUI within ~100ms of pressing R/S | GUI-05 | Wall-clock timing on a real ~5s integration is environment-dependent; automation would be flaky on CI | 1) Launch `just gui-dev`. 2) Enter `0 ENTER 1 XEQ "INTG"` with `LN(X)` as f(X). 3) After 1s, press R/S. 4) Verify LCD displays `CANCELED` within 100–200ms. |
| Matrix-entry modal UX (`A1,1=?` prompts walk through every cell) | GUI-06 | Multi-prompt loop with explicit timing/keystrokes; covered by Plan 31-05 unit test for shape, but full UX is manual | 1) `XEQ "MAT" 2 ENTER 2 ENTER`. 2) Confirm prompt cycles `A1,1=? → A1,2=? → A2,1=? → A2,2=?`. 3) ESC mid-loop returns to idle; print panel shows partial entries. |
| `CATALOG 2` UI scroll behavior (instant render vs. PSE-step) | GUI-04 | RESEARCH Open Q2 — v3.0 ships instant-render; PSE-step deferred to v3.1 | 1) `XEQ "CAT" 2`. 2) Verify scrollable print panel shows all loaded XROM modules with function counts. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (9 items above)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s (quick) / < 5 min (full) / < 10 min (E2E)
- [ ] `hp41-core` coverage stays ≥ 95% lines / 93.75% regions (v2.2 Phase 27 floor)
- [ ] `nyquist_compliant: true` set in frontmatter after sign-off

**Approval:** pending
