---
phase: 31
slug: gui-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-17
revised: 2026-05-17
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
| 31-01-02 | 01 | 1 | GUI-01 | — | All Phase 28 `Op` variants have an `op_display_name` arm in `hp41-gui/src-tauri/src/prgm_display.rs` (file-text scan, not Rust API — `prgm_display` is a private module per `lib.rs:10`) | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test prgm_display_math1_arms` | ❌ W0 | ⬜ pending |
| 31-02-01 | 02 | 1 | GUI-05 | — | `scripts/check-tauri-permissions.sh` exists & exits 0; all `generate_handler!` commands have permission TOMLs | shell | `bash scripts/check-tauri-permissions.sh` | ❌ W0 | ⬜ pending |
| 31-02-02 | 02 | 2 | GUI-05 | — | `request_cancel` Tauri command sets `cancel_requested` AtomicBool without acquiring AppState Mutex | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test cancel_command_no_deadlock` | ❌ W0 | ⬜ pending |
| 31-02-03 | 02 | 2 | GUI-05 | — | `op_integ`/`op_solve`/`op_difeq` check `cancel_requested` every 64 samples and return `HpError::Canceled` (basic cancellation case covered by existing in-module test `hp41-core/src/ops/math1/integ.rs:862-871` — `#[test] fn cancel_per_64_samples`) | unit (in-module) | `cargo test --package hp41-core ops::math1::integ::tests::cancel_per_64_samples` | ✅ existing (`integ.rs:862-871`) | ⬜ pending |
| 31-02-04 | 02 | 2 | GUI-05 | — | Auto-save thread + `request_cancel` interleave without deadlock under stress | integration | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test cancel_autosave_stress` | ❌ W0 | ⬜ pending |
| 31-02-05 | 02 | 2 | GUI-05 | — | `cancel_requested` is reset on workflow-opener entry for ALL THREE programs (idempotency — integ + solve + difeq) | unit | `cargo test --package hp41-core --test cancel_flag_reset_on_open` | ❌ W0 | ⬜ pending |
| 31-03-01 | 03 | 2 | GUI-02 | — | XEQ "SINH" 1.5 → 2.1293; identical X-reg value between CLI and GUI dispatch paths (D-25.6 parity) | integration | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test cli_gui_parity` | ❌ W0 | ⬜ pending |
| 31-03-02 | 03 | 2 | GUI-02 | — | XEQ-by-name modal in `App.tsx` constructs `Op::Xeq(name)` payload routed through `dispatch_op` | unit (vitest) | `cd hp41-gui && npm test -- App.xeq_modal` | ❌ W0 | ⬜ pending |
| 31-03-03 | 03 | 2 | GUI-02 | — | `submit_modal` and `cancel_modal` Tauri thunks registered with permission TOMLs | shell | `bash scripts/check-tauri-permissions.sh` | ❌ W0 (covered by 31-02-01) | ⬜ pending |
| 31-04-01 | 04 | 2 | GUI-03 | — | `?`-overlay imports BOTH `hp41cv-functions.json` AND `hp41-math1-functions.json` via Vite | unit (vitest) | `cd hp41-gui && npm test -- HelpOverlay.math1_section` | ❌ W0 | ⬜ pending |
| 31-04-02 | 04 | 2 | GUI-03 | — | Math Pac I rendered as a distinct categorized section ("Math 1 Pac") in the overlay | unit (vitest) | `cd hp41-gui && npm test -- HelpOverlay.categories` | ❌ W0 | ⬜ pending |
| 31-04-03 | 04 | 2 | GUI-04 | — | `Op::Catalog(2)` extended from stub → enumerates loaded XROM modules with function counts (instant-scroll per W1 fix — D-31.12/D-31.14 PSE-step deferred to v3.1) | unit | `cargo test --package hp41-core --test op_catalog_xrom` | ❌ W0 | ⬜ pending |
| 31-05-01 | 05 | 2 | GUI-06 | — | `state.modal_prompt` rendered on the LCD via `types.rs::CalcStateView::from_state` (new top-of-chain branch; NOT `handle_get_state`, NOT `display_override` per W2 fix); ≡ continuation marker for >12 chars | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test lcd_alternation_modal_prompt` | ❌ W0 | ⬜ pending |
| 31-05-02 | 05 | 2 | GUI-06 | — | Number-entry pipeline routes to modal accumulator while modal_program is active; ENTER submits, ESC cancels | unit (vitest) + integration | `cd hp41-gui && npm test -- App.modal_input_pipeline` | ❌ W0 | ⬜ pending |
| 31-05-03 | 05 | 2 | GUI-07 | — | Stub-error format!-template count in `key_map::resolve` exactly equals the captured v2.1 baseline N (file-text scan locked to `==`, not `>=`; no shrink AND no silent growth) | unit (file-text) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test key_map_stub_error_arms` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `scripts/check-tauri-permissions.sh` — author NEW (RESEARCH Open Q1 body); CI gate verifies every `generate_handler!` member has a matching permission TOML
- [ ] `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` — assert all Phase 28 `Op` variants appear in `hp41-gui/src-tauri/src/prgm_display.rs` via file-text scan (private module; no Rust API access)
- [ ] `hp41-gui/src-tauri/tests/cancel_command_no_deadlock.rs` — `request_cancel` thunk uses `tauri::State<CancelFlag>` (NOT AppState Mutex)
- [ ] `hp41-core/src/ops/math1/integ.rs:862-871` (existing — `#[test] fn cancel_per_64_samples`) — basic INTG cancellation case covered. Granularity (64-sample boundary) verified by this existing test that injects cancel mid-loop. The parallel-coverage need for solve and difeq is satisfied by the Plan 31-02 Task 3 file `hp41-core/tests/cancel_flag_reset_on_open.rs` (idempotency invariant — verifies all three openers reset the flag) plus the existing per-64-samples poll-site at `solve.rs:384` and `difeq.rs:305` shipped in Phase 28. No standalone `integ_cancellation.rs` test file is authored — that earlier draft reference (W5) was an orphan and is removed from this list.
- [ ] `hp41-gui/src-tauri/tests/cancel_autosave_stress.rs` — multi-thread interleaving test (auto-save + request_cancel + active INTG)
- [ ] `hp41-gui/src-tauri/tests/cli_gui_parity.rs` — drive SINH/ASINH/TANH/MOD/GAMMA through both paths, assert identical X-register
- [ ] `hp41-gui/src/App.test.tsx` — extend with `xeq_modal` test (vitest)
- [ ] `hp41-gui/src/HelpOverlay.test.tsx` — extend with `math1_section` + `categories` tests (vitest)
- [ ] `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs` — assert `CalcStateView::from_state` new top branch produces correct truncated display_str when modal_program.is_some() && entry_buf.is_empty()
- [ ] `hp41-gui/src-tauri/tests/key_map_stub_error_arms.rs` — file-text count of "is planned for a future phase" locked to exact baseline N (no shrink, no silent growth — GUI-07)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Long INTG of `LN(X)` from 0 to 1 cancellable from GUI within ~100ms of pressing R/S | GUI-05 | Wall-clock timing on a real ~5s integration is environment-dependent; automation would be flaky on CI | 1) Launch `just gui-dev`. 2) Enter `0 ENTER 1 XEQ "INTG"` with `LN(X)` as f(X). 3) After 1s, press R/S. 4) Verify LCD displays `CANCELED` within 100–200ms. |
| Matrix-entry modal UX (`A1,1=?` prompts walk through every cell) | GUI-06 | Multi-prompt loop with explicit timing/keystrokes; covered by Plan 31-05 unit test for shape, but full UX is manual | 1) `XEQ "MAT" 2 ENTER 2 ENTER`. 2) Confirm prompt cycles `A1,1=? → A1,2=? → A2,1=? → A2,2=?`. 3) ESC mid-loop returns to idle; print panel shows partial entries. |
| `CATALOG 2` UI scroll behavior (instant-scroll per W1 fix; PSE-step + R/S pause/resume D-31.12/D-31.14 deferred to v3.1) | GUI-04 | Instant-scroll matches v2.2 CAT 1 (verified RESEARCH Open Q2); ensure all module + function lines appear in a single get_state response | 1) `XEQ "CAT" 2`. 2) Verify scrollable print panel shows all loaded XROM modules with function counts in one shot (no progressive reveal). |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (10 items above)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s (quick) / < 5 min (full) / < 10 min (E2E)
- [ ] `hp41-core` coverage stays ≥ 95% lines / 93.75% regions (v2.2 Phase 27 floor)
- [ ] `nyquist_compliant: true` set in frontmatter after sign-off

**Approval:** pending

---

## Revision Notes (2026-05-17)

The following targeted corrections were applied based on `/gsd:check-plan` feedback:

- **W7 (package name):** All `cargo test -p hp41-gui-tauri` invocations replaced with `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` (the package is named `hp41-gui` in `Cargo.toml`; the manifest-path form matches the conventions used in PLAN files and avoids the `-p` package-name lookup ambiguity).
- **W5 (orphan test file):** The earlier reference to `hp41-core/tests/integ_cancellation.rs` was an orphan — no plan authored it. Replaced with a reference to the existing in-module test `cancel_per_64_samples` at `hp41-core/src/ops/math1/integ.rs:862-871` (shipped in Phase 28; covers the basic cancellation case AND the per-64-samples granularity). The parallel-coverage need for solve and difeq is satisfied by Plan 31-02 Task 3 (`cancel_flag_reset_on_open.rs` — three tests, one per program) combined with the existing per-64-samples poll-sites in `solve.rs:384` and `difeq.rs:305`. No new standalone `integ_cancellation.rs` test file is authored.
- **W2 (LCD-alternation routing path) reflected in row 31-05-01:** the routing site is `types.rs::CalcStateView::from_state` (new top-of-chain branch); `handle_get_state` is NOT modified for LCD-alternation; `display_override` is NOT used (reserved for Phase 21 VIEW/AVIEW/PROMPT/CLD per `hp41-core/src/state.rs:142-143`).
- **W6 (stub-arm count) reflected in row 31-05-03:** the regression test uses `assert_eq!(..., BASELINE_N)` exact equality (NOT `>=`) — catches BOTH deletion AND silent addition.
- **W1 (CAT 2 instant-scroll deviation) reflected in row 31-04-03 and the Manual-Only `CATALOG 2` entry:** D-31.12 / D-31.14 PSE-step + R/S pause/resume DEFERRED to v3.1; Plan 31-04 ships instant-scroll matching v2.2 CAT 1.
