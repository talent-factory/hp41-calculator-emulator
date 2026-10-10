---
phase: 63
slug: run-loop-yield-engine-interrupting-alarms-pse-view-aview
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-06
revised: 2026-06-06
---

# Phase 63 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `63-RESEARCH.md` § Validation Architecture (16 core scenarios) and `.planning/research/PITFALLS.md` § Verification Strategy.
> **Revised 2026-06-06:** Phase 63 now ALSO delivers the GUI continuous-program-execution path (new `run_program`/`resume_program` Tauri commands in 63-04 + the App.tsx yield-and-resume driver in 63-06), reaching CLI parity for running programs + PSE/VIEW/AVIEW + interrupting alarms. The 16 core scenarios are unchanged; GUI run-loop validation is added below.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust integration tests (`cargo` via `just`) + Vitest (GUI TS) + GUI-crate Rust unit tests |
| **Config file** | none — workspace `Cargo.toml` + existing `hp41-core/tests/` convention + `hp41-gui/src-tauri` crate |
| **Quick run command** | `just test-core` |
| **Full suite command** | `just ci` (lint + test + coverage + license-audit) + `just gui-ci` (GUI 3-OS) |
| **Estimated runtime** | ~30–60 s (`just test-core`); ~3–5 min (`just ci`); +GUI |

**New test files:**
- `hp41-core/tests/phase_63_interrupting_alarms.rs` — alarm / run-loop engine tests (Scenarios 1–7 + edges)
- `hp41-core/tests/phase_63_yield_engine.rs` — PSE / VIEW / AVIEW yield tests
- GUI: new Vitest cases in `hp41-gui/src/App.test.tsx` (yield render + auto-resume, alarm:missing toast, R/S start routing) + a Rust unit test in `hp41-gui/src-tauri/src/commands.rs` (from_state pending_yield projection).

**Determinism contract:** `trigger_unix = 0`, `time_offset_secs = 0`; assert on `CalcState` fields (`regs`, `call_stack`, `event_buffer`, `print_buffer`, `pending_yield.resume_ms`) — **never** on `SystemTime::now()`. Surface `resume_ms` as **data** and assert it; **never `sleep`** in core unit tests (the frontend owns the wait). GUI Vitest uses fake timers (`vi.useFakeTimers()`) to assert the scheduled resume — no real wall-clock wait.

---

## Sampling Rate

- **After every task commit:** Run `just test-core` (core) / `cd hp41-gui && npm test` (TS) / `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` (GUI Rust)
- **After every plan wave:** Run `just ci` (and `just gui-ci` if GUI files touched in the wave)
- **Before `/gsd:verify-work`:** Full `just ci` + `just ci-msrv` green; `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings` green; `cargo clippy --manifest-path hp41-gui/src-tauri/Cargo.toml --all-targets -- -D warnings` green; `cargo check --target aarch64-apple-ios --manifest-path hp41-gui/src-tauri/Cargo.toml` green
- **Max feedback latency:** ~60 s (quick) / ~5 min (full)

---

## Per-Task Verification Map — Core (16 scenarios, UNCHANGED)

The rows below bind each core validation scenario to its Success Criterion (SC 1–5) and Requirement. These are frontend-agnostic (run inside `hp41-core`'s `run_loop`) and underpin BOTH frontends.

| Scenario / Test | Requirement | SC | Test Type | Automated Command | File | Status |
|-----------------|-------------|----|-----------|-------------------|------|--------|
| `interrupting_alarm_halts_running_program_and_resumes` | ALARM-02/03 | 1 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `interrupt_preserves_stack_x_y_z_t_and_lift_state` | ALARM-02 | 1 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `interrupt_blocked_when_call_stack_at_4_level_cap` | ALARM-03 | 2 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `interrupt_nesting_blocked_when_already_in_alarm_program` | ALARM-03 | 2 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `interrupting_alarm_fires_when_no_program_running` | ALARM-02 | 3 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `non_interrupting_alarm_still_fires_as_event_not_inline` (DNT-05) | ALARM-03 | — | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `message_alarm_still_fires_to_event_buffer_not_executed` | — | — | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `pse_mid_run_breaks_and_records_resume_ms` | PRGM-01 | 4 | integration | `just test-core --test phase_63_yield_engine` | phase_63_yield_engine.rs | ⬜ pending |
| `pse_resume_continues_to_next_step` | PRGM-01 | 4 | integration | `just test-core --test phase_63_yield_engine` | phase_63_yield_engine.rs | ⬜ pending |
| `view_mid_run_captures_formatted_register_into_yield` | PRGM-02 | 5 | integration | `just test-core --test phase_63_yield_engine` | phase_63_yield_engine.rs | ⬜ pending |
| `aview_mid_run_captures_alpha_into_yield` | PRGM-02 | 5 | integration | `just test-core --test phase_63_yield_engine` | phase_63_yield_engine.rs | ⬜ pending |
| `interrupt_demoted_when_solver_or_modal_active` (D-10) | ALARM-03 | 1 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `missing_handler_label_surfaces_event` (D-08) | ALARM-03 | 1 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `pending_interrupt_cleared_on_resume_after_stop` (D-09) | ALARM-02 | 1 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `v4_3_interrupt_backward_compat` (I-03) | ALARM-02 | 1 | integration | `just test-core` | phase_63_interrupting_alarms.rs | ⬜ pending |
| `repeating_interrupting_alarm_reschedules_after_handler` (D-06) | ALARM-03 | 1 | integration | `just test-core --test phase_63_interrupting_alarms` | phase_63_interrupting_alarms.rs | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Per-Task Verification Map — GUI run loop (NEW, revision)

GUI Rust commands are NOT line-coverage-gated (the `hp41-core` ≥95% gate does not extend to `hp41-gui/src-tauri`); the GUI's automated backstop is the from_state Rust unit test + Vitest with fake timers, with a manual E2E as the final visual check.

| Scenario / Test | Requirement | SC | Test Type | Automated Command | File | Status |
|-----------------|-------------|----|-----------|-------------------|------|--------|
| `from_state` projects pending_yield (kind/text/resume_ms) | PRGM-01/02 | 4/5 | GUI Rust unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | commands.rs | ⬜ pending |
| run_program + resume_program commands registered + invokable | PRGM-01/02 | 1/4/5 | GUI Rust compile | `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` | lib.rs / capabilities | ⬜ pending |
| GUI PSE yield renders text + auto-resumes after resume_ms | PRGM-01 | 4 | Vitest (fake timers) | `cd hp41-gui && npm test` | App.test.tsx | ⬜ pending |
| GUI VIEW/AVIEW yield renders captured text + auto-resumes | PRGM-02 | 5 | Vitest (fake timers) | `cd hp41-gui && npm test` | App.test.tsx | ⬜ pending |
| GUI R/S on a stopped program starts run_program('A') | PRGM-01 | 1 | Vitest | `cd hp41-gui && npm test` | App.test.tsx | ⬜ pending |
| GUI alarm:missing:{label} surfaces a toast | ALARM-03, D-08 | 2 | Vitest | `cd hp41-gui && npm test` | App.test.tsx | ⬜ pending |
| GUI interrupting alarm fires during a GUI-run program | ALARM-02 | 1 | Manual E2E (vitest backstop for the path) | `just gui-dev` | (manual) | ⬜ pending |

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/phase_63_interrupting_alarms.rs` — file-level `#![allow(clippy::unwrap_used)]`, deterministic `make_past_due_interrupting_alarm` helper (`trigger_unix = 0`), program-load helper (`state.program = vec![Op::Lbl…]`). **Authored FIRST in 63-01 Task 1** (before the TDD state+routing tasks).
- [ ] `hp41-core/tests/phase_63_yield_engine.rs` — same conventions; asserts `pending_yield.resume_ms` as data. **Authored FIRST in 63-01 Task 1.**

*Framework already present (cargo + existing `hp41-core/tests/phase_*.rs` convention; analog: `time_alarm_latency.rs`, `program_tests.rs`; Vitest already configured for GUI). No install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLI yield render + sleep-then-resume | PRGM-01/02 | `hp41-cli` is NOT coverage-gated; wall-clock sleep is frontend-owned | Run a program with PSE/VIEW/AVIEW in the TUI; confirm the value shows ~1 s before the next step; targeted `hp41-cli/src/` tests where feasible (63-03) |
| GUI yield render + scheduled resume (run loop) | PRGM-01/02 | Tauri runtime + visual timing; GUI Rust not coverage-gated | `just gui-dev`; R/S a program with PSE; confirm display pause + auto-resume; D-25.6 parity with CLI. Vitest fake-timer cases (63-06) are the automated backstop |
| GUI interrupting-alarm fire during a running program | ALARM-02, D-08 | E2E smoke does not yet cover alarms (CI gap, PITFALLS) | `just gui-dev`; start a user-LBL program; let an interrupting alarm come due mid-run; confirm the handler runs + resumes; for a bad label confirm a toast surfaces. E2E smoke is a "should-have" |
| iOS `#[cfg(mobile)]` compile of GUI run-loop wiring | — | `just gui-ci` host target cannot see `#[cfg(mobile)]` | `cargo check --target aarch64-apple-ios --manifest-path hp41-gui/src-tauri/Cargo.toml` after editing `commands.rs`/`lib.rs` (63-04 Task 3) |

---

## Success-Criterion → Frontend reachability (re-confirmed, revision)

| SC | Behavior | CLI path | GUI path |
|----|----------|----------|----------|
| 1 | Interrupting alarm halts a running program, handler runs, resumes | 63-02 run_loop; CLI runs programs today (app.rs F5 run_program) | **63-04 run_program/resume_program commands + 63-06 R/S driver** make the GUI run continuous programs; Phase-C fires the interrupt server-side mid-run |
| 2 | 4-level cap silent suppress + missing-label surface | 63-02 (cap); 63-03 (CLI status-line missing-label) | 63-02 (cap); **63-06 alarm:missing toast** |
| 3 | Idle interrupting alarm fires | 63-01 idle→alarm:xeq; CLI drain_event_buffer | 63-01 idle→alarm:xeq; GUI tick_time event consumer (already worked) |
| 4 | PSE mid-run pauses then resumes | 63-02 yield arm; 63-03 render→sleep→resume | 63-02 yield arm; **63-04 projection + 63-06 render + scheduled resume** |
| 5 | VIEW/AVIEW mid-run pause | 63-02 yield arm; 63-03 render→sleep→resume | 63-02 yield arm; **63-04 projection + 63-06 render + scheduled resume** |

All five criteria are now reachable in BOTH frontends.

---

## Validation Sign-Off

- [x] All scenarios have an `<automated>` verify or a Manual-Only justification
- [x] Sampling continuity: every engine task gates on `just test-core`; GUI tasks gate on `npm test` / GUI-crate `cargo test`; no 3 consecutive tasks without automated verify
- [x] Wave 0 covers the two new test files (authored FIRST in 63-01 Task 1)
- [x] GUI run-loop scenarios added; SC 1–5 re-confirmed reachable in both frontends
- [x] No watch-mode flags (one-shot `just test-core` / `just ci` / `npm test`)
- [x] Feedback latency < 60 s (quick) documented
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-06 (revised — GUI run loop added)
</content>
