---
phase: 64
slug: interactive-getkey
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-07
---

# Phase 64 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `64-RESEARCH.md` → Validation Architecture. Task IDs are filled in by the planner;
> this contract maps every PRGM-03 sub-behavior to an automated command + file.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (`hp41-core`, `hp41-gui/src-tauri`) + Vitest (`hp41-gui`) |
| **Config file** | `hp41-core/Cargo.toml` (workspace), `hp41-gui/vitest.config.ts` |
| **Quick run command** | `just test-core --test phase_64_getkey` |
| **Full suite command** | `just ci && just gui-ci` |
| **Estimated runtime** | ~core tests < 30 s; full `ci` + `gui-ci` several minutes |

---

## Sampling Rate

- **After every task commit:** Run `just test-core --test phase_64_getkey`
- **After every plan wave:** Run `just ci` (core + CLI + license-audit)
- **Before `/gsd:verify-work`:** `just ci && just gui-ci` fully green
- **Max feedback latency:** < 30 s (core quick run)

---

## Per-Requirement Verification Map

> PRGM-03 decomposed into the testable sub-behaviors from RESEARCH.md. `File Exists` = whether the
> test file/case is present today; ❌ W0 = must be created in Wave 0.

| Sub-ID | Behavior | Test Type | Automated Command | File Exists | Status |
|--------|----------|-----------|-------------------|-------------|--------|
| PRGM-03-a | GETKEY mid-run breaks `run_loop` + sets `pending_yield = WaitForKey` | unit | `just test-core --test phase_64_getkey -- getkey_mid_run_breaks_and_sets_wait_for_key` | ❌ W0 | ⬜ pending |
| PRGM-03-b | `resume_program_with_key(code)` pushes row×col code to X (LiftEffect::Enable) | unit | `just test-core --test phase_64_getkey -- resume_with_key_pushes_code_to_x` | ❌ W0 | ⬜ pending |
| PRGM-03-c | `resume_program_with_key(0)` pushes sentinel 0 (cancel path) | unit | `just test-core --test phase_64_getkey -- resume_with_key_zero_pushes_sentinel` | ❌ W0 | ⬜ pending |
| PRGM-03-d | Program continues to the next step after GETKEY + resume | unit | `just test-core --test phase_64_getkey -- getkey_resume_continues_to_next_step` | ❌ W0 | ⬜ pending |
| PRGM-03-e | GETKEY does NOT write `display_override` (D-03: display unchanged) | unit | `just test-core --test phase_64_getkey -- getkey_does_not_write_display_override` | ❌ W0 | ⬜ pending |
| PRGM-03-f | GETKEY inside alarm handler: `pending_interrupt*` fields survive resume | unit | `just test-core --test phase_64_getkey -- getkey_inside_alarm_handler_preserves_interrupt_state` | ❌ W0 | ⬜ pending |
| PRGM-03-g | GETKEY then PSE: sequential yields work (WaitForKey then Pse) | unit | `just test-core --test phase_64_getkey -- getkey_then_pse_sequential_yields` | ❌ W0 | ⬜ pending |
| PRGM-03-h | Non-program (interactive) dispatch still uses `last_key_code` (back-compat) | unit | `just test-core --test synthetic_tests -- test_getkey_pushes_last_key_code` | ✅ existing | ⬜ pending |
| PRGM-03-i | `getkey_captured_code` cleared after `run_loop` error (no leakage) | unit | `just test-core --test phase_64_getkey -- getkey_captured_code_cleared_on_error` | ❌ W0 | ⬜ pending |
| PRGM-03-j | GUI `CalcStateView` projects WaitForKey as `"wait_for_key"` | Rust unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- from_state_projects_wait_for_key` | ❌ W0 | ⬜ pending |
| PRGM-03-k | GUI yield driver ignores WaitForKey (no `setTimeout` scheduled) | Vitest | `just gui-ci -- getkey_yield_driver_skips_wait_for_key` | ❌ W0 | ⬜ pending |
| PRGM-03-l | GUI key event during WaitForKey → `resume_program_with_key` (not `dispatch_op`) | Vitest | `just gui-ci -- getkey_key_press_routes_to_resume` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/phase_64_getkey.rs` — covers PRGM-03-a … PRGM-03-i (9 new tests; surface the captured keycode as data, no real keyboard input — mirror Phase 63's no-wall-clock-sleep approach)
- [ ] `hp41-gui/src-tauri/src/commands.rs` test `from_state_projects_wait_for_key` — covers PRGM-03-j
- [ ] `hp41-gui/src/App.test.tsx` — Group Q tests covering PRGM-03-k and PRGM-03-l
- [ ] Prereq variant `YieldKind::WaitForKey` + transient `getkey_captured_code: Option<u8>` in `hp41-core/src/state.rs` (state deliverable; tests depend on it)

*Existing `synthetic_tests.rs::test_getkey_pushes_last_key_code` covers PRGM-03-h — no gap there.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real keypress during a running program returns the correct row×col code and resumes (CLI) | PRGM-03 | End-to-end keyboard capture in a live TUI is not unit-testable | Run a short program containing GETKEY in `hp41-cli`; press a key; confirm X = that key's row×col code; press R/S → expect **31**; press Esc → expect sentinel **0** + program ends |
| On-screen + physical key capture during GETKEY (GUI) | PRGM-03 / D-04 | Live WKWebView key capture is integration-level | In `just gui-dev`, run a program with GETKEY; tap an on-screen key and press a physical key; confirm both resume with the correct code; confirm CHS / x≥y / ← taps are ignored (no `keyCode`) |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30 s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
