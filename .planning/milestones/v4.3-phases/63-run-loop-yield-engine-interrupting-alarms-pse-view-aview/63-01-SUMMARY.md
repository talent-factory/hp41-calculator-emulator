---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
plan: "01"
subsystem: hp41-core/state + hp41-core/ops/time/alarm
tags: [interrupting-alarms, yield-engine, state-fields, serde, tdd-red]
dependency_graph:
  requires: []
  provides:
    - pending_interrupt field on CalcState (D-12)
    - pending_interrupt_alarm_index field on CalcState (D-06a)
    - pending_interrupt_depth field on CalcState (63-02 ack gate)
    - pending_yield field on CalcState with YieldKind/YieldState types (D-04)
    - PSE_RESUME_MS constant (single source of truth for yield duration)
    - dispatch_alarm_event interrupting-arm routing (running→pending_interrupt; idle/demoted→alarm:xeq)
    - Wave-0 RED test scaffold (phase_63_interrupting_alarms.rs, phase_63_yield_engine.rs)
  affects:
    - hp41-core/src/state.rs (CalcState struct + new() + serde test)
    - hp41-core/src/ops/time/alarm.rs (dispatch_alarm_event signature + routing)
    - hp41-core/tests/phase_63_interrupting_alarms.rs (new)
    - hp41-core/tests/phase_63_yield_engine.rs (new)
tech_stack:
  added: []
  patterns:
    - "#[serde(default, skip)] transient field pattern (matches event_buffer/display_override)"
    - "YieldState typed channel replaces stringly-typed PAUSE 1000 event (D-04)"
    - "dispatch_alarm_event index+defer_to_run_loop params prevent double-ack hazard"
key_files:
  created:
    - hp41-core/tests/phase_63_interrupting_alarms.rs
    - hp41-core/tests/phase_63_yield_engine.rs
  modified:
    - hp41-core/src/state.rs
    - hp41-core/src/ops/time/alarm.rs
decisions:
  - "YieldKind/YieldState/PSE_RESUME_MS named as recommended in RESEARCH (Claude's discretion D-04)"
  - "pending_interrupt_depth declared in 63-01 (state.rs) so 63-02 (program.rs-only) can read/write it without breaking wave isolation"
  - "dispatch_alarm_event gains defer_to_run_loop=bool param: check_alarms passes true (defers to run_loop); op_almnow passes false (owns its own ack — avoids double-ack/stale-index hazard)"
  - "Updated existing check_alarms_interrupting_control_pushes_deferred_event test to reflect new idle routing; added running-path test"
metrics:
  duration: "~35 minutes"
  completed: "2026-06-06"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 4
---

# Phase 63 Plan 01: Wave-0 Scaffold + State Fields + Alarm Routing Summary

**One-liner:** YieldKind/YieldState typed yield channel + four transient CalcState fields + dispatch_alarm_event interrupting-arm routing (running→pending_interrupt; idle/demoted→alarm:xeq) with Wave-0 RED test scaffolds.

---

## What Was Built

### Task 1 — Wave-0 test scaffolds (3695cd0)

Two new integration test files authored FIRST per TDD discipline:

**`hp41-core/tests/phase_63_interrupting_alarms.rs`** — 12 alarm/run-loop scenarios:
- Deterministic `make_past_due_interrupting_alarm` helper (`trigger_unix=0`, `time_offset_secs=0`)
- 7 core scenarios (alarms + run-loop: interrupt halts/resumes, preserves stack, cap-block, nesting-block, idle-queue, DNT-05 regression, message arm)
- 5 edge scenarios (solver demotion D-10, missing label D-08, STOP clear D-09, backward compat I-03, repeating reschedule D-06)
- Run_loop-dependent scenarios clearly marked `// GREEN after 63-02`

**`hp41-core/tests/phase_63_yield_engine.rs`** — 4 PSE/VIEW/AVIEW yield scenarios:
- Asserts `pending_yield.resume_ms == PSE_RESUME_MS` as DATA (never sleep in core)
- Asserts `display_override` is NOT written (D-04 discipline)
- Asserts `pending_yield.text` contains the formatted register/alpha value

### Task 2 — CalcState fields + types (b8f4e3e)

**New types in `hp41-core/src/state.rs`:**
- `pub enum YieldKind { Pse, View, Aview }` — typed yield discriminant
- `pub struct YieldState { kind: YieldKind, text: String, resume_ms: u64 }` — typed yield channel
- `pub const PSE_RESUME_MS: u64 = 1000` — single source of truth for PSE/VIEW/AVIEW duration

**Four new transient fields on `CalcState`** (all `#[serde(default, skip)]`):
- `pending_interrupt: Option<String>` — D-12 pending label for run_loop injection
- `pending_interrupt_alarm_index: Option<usize>` — D-06a paired alarm index for ack-after-RTN
- `pending_interrupt_depth: Option<usize>` — D-06 call_stack depth at injection; 63-02 ack gate
- `pending_yield: Option<YieldState>` — D-04 yield channel for PSE/VIEW/AVIEW

**CalcState::new()** initializes all four to `None`.

**serde_roundtrip test extended** to assert all new fields absent from JSON and reset to None after deserialization.

### Task 3 — dispatch_alarm_event routing (03650ac)

Replaced the `"alarm:interrupting:deferred"` dead-end stub with real Phase 63 routing:

```
Running + no pending + no solver/modal → pending_interrupt = Some(label) + alarm_index = Some(i)
Idle (D-13) / already-pending / solver/modal (D-10) / defer=false → alarm:xeq:{label}
```

Signature extended with `index: usize` + `defer_to_run_loop: bool`:
- `check_alarms` passes `i` (loop index) + `true`
- `op_almnow` passes `idx` + `false` (owns its own synchronous ack; passing true would create double-ack/stale-index hazard)

DNT-05: message arm and non-interrupting arm are byte-for-byte unchanged.

---

## Test Status After Plan 63-01

| Test | Status | Notes |
|------|--------|-------|
| `v4_3_interrupt_backward_compat` | GREEN | serde compat — no 63-02 needed |
| `message_alarm_still_fires_to_event_buffer_not_executed` | GREEN | message arm unchanged |
| `non_interrupting_alarm_still_fires_as_event_not_inline` | GREEN | DNT-05 |
| `interrupting_alarm_fires_when_no_program_running` | GREEN | idle path D-13 |
| `interrupt_nesting_blocked_when_already_in_alarm_program` | GREEN | nesting guard |
| `interrupt_demoted_when_solver_or_modal_active` | GREEN | D-10 demotion |
| `interrupt_blocked_when_call_stack_at_4_level_cap` | GREEN (partial) | 63-01 routing side only; cap enforcement in 63-02 run_loop |
| `missing_handler_label_surfaces_event` | GREEN (partial) | 63-01 sets pending_interrupt; alarm:missing surfaces in 63-02 |
| `interrupting_alarm_halts_running_program_and_resumes` | RED | run_loop injection arm — GREEN after 63-02 |
| `interrupt_preserves_stack_x_y_z_t_and_lift_state` | RED | run_loop injection arm — GREEN after 63-02 |
| `pending_interrupt_cleared_on_resume_after_stop` | RED | resume_program clear (D-09) — GREEN after 63-02 |
| `repeating_interrupting_alarm_reschedules_after_handler` | RED | run_loop ack-after-RTN (D-06) — GREEN after 63-02 |
| All PSE/VIEW/AVIEW yield tests | RED | run_loop yield arms — GREEN after 63-02 |

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wrong Op variant name in test files**
- **Found during:** Task 2 compile check
- **Issue:** Tests used `Op::Sto(n)` but the variant is `Op::StoReg(n)`
- **Fix:** Global replace `Op::Sto(` → `Op::StoReg(` in both test files
- **Files modified:** phase_63_interrupting_alarms.rs, phase_63_yield_engine.rs
- **Commit:** b8f4e3e

**2. [Rule 1 - Bug] Wrong ClockDisplayMode variant in backward-compat fixture**
- **Found during:** Task 2 test run
- **Issue:** `"clock_display_mode": "HhMmSs"` is not a valid variant; correct default is `"Off"`
- **Fix:** Changed to `"Off"` in the v4_3_interrupt_backward_compat JSON fixture
- **Files modified:** phase_63_interrupting_alarms.rs
- **Commit:** b8f4e3e

**3. [Rule 1 - Bug] Unused imports triggering clippy -D warnings**
- **Found during:** Task 2 clippy check
- **Issue:** `acknowledge_alarm`, `dispatch`, `YieldKind`, `std::str::FromStr`, `push_x` fn unused
- **Fix:** Removed unused imports and dead helper function from test files
- **Files modified:** phase_63_interrupting_alarms.rs, phase_63_yield_engine.rs
- **Commit:** b8f4e3e

**4. [Rule 1 - Bug] Existing test `check_alarms_interrupting_control_pushes_deferred_event` expected old dead-end stub**
- **Found during:** Task 3 lib test run
- **Issue:** The existing lib test asserted `"alarm:interrupting:deferred"` which the routing change removed
- **Fix:** Updated test to reflect new idle routing (`alarm:xeq:IPROG`) and renamed to `check_alarms_interrupting_control_idle_queues_xeq_event`; added new running-path test `check_alarms_interrupting_control_running_sets_pending_interrupt`
- **Files modified:** hp41-core/src/ops/time/alarm.rs
- **Commit:** 03650ac

---

## Known Stubs

None — the yield channel and pending_interrupt fields are real data fields. The run_loop injection and yield arms are deferred to 63-02 (plan boundary by design, not stubs).

---

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

---

## Self-Check: PASSED

- `hp41-core/tests/phase_63_interrupting_alarms.rs` — FOUND
- `hp41-core/tests/phase_63_yield_engine.rs` — FOUND
- `hp41-core/src/state.rs` modified with new types and fields — FOUND
- `hp41-core/src/ops/time/alarm.rs` rewired — FOUND
- Commit 3695cd0 (Task 1) — FOUND in git log
- Commit b8f4e3e (Task 2) — FOUND in git log
- Commit 03650ac (Task 3) — FOUND in git log
- `grep -rn '"alarm:interrupting' hp41-core/src/ | grep -v "//"` — EMPTY (stub removed)
- `cargo +1.88 clippy -p hp41-core --all-targets -- -D warnings` — CLEAN
- `git diff HEAD -- hp41-core/Cargo.toml hp41-core/src/ops/math1/` — EMPTY (no new deps, math1 frozen)
- Info.plist NOT staged or modified — CONFIRMED
