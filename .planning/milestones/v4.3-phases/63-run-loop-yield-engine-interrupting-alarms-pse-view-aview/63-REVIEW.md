---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
reviewed: 2026-06-06T00:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - hp41-core/src/ops/program.rs
  - hp41-core/src/state.rs
  - hp41-core/src/ops/time/alarm.rs
  - hp41-cli/src/app.rs
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-gui/src/App.tsx
  - hp41-core/tests/phase_63_interrupting_alarms.rs
  - hp41-core/tests/phase_63_yield_engine.rs
  - hp41-gui/src/App.test.tsx
  - hp41-gui/src-tauri/capabilities/default.json
  - hp41-gui/src-tauri/permissions/run-program.toml
  - hp41-gui/src-tauri/permissions/resume-program.toml
findings:
  critical: 2
  warning: 5
  info: 2
  total: 9
status: issues_found
---

# Phase 63: Code Review Report

**Reviewed:** 2026-06-06
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 63 adds the run-loop yield engine (PSE/VIEW/AVIEW yield-and-resume) and
interrupting control alarms (`pending_interrupt` injected at run_loop instruction
boundaries). The serde backward-compat discipline is clean (all four new
`CalcState` fields are `#[serde(default, skip)]` with round-trip + v4.2-JSON
regression tests), the yield channel correctly leaves `display_override` untouched
(D-04), and the core run_program/resume_program entry points correctly clear
all four transient fields. The Tauri command/permission/capability wiring is
complete and consistent.

However, the review found **two BLOCKERs** that break the headline feature on one
frontend, plus **five WARNINGs** concentrated in the alarm-acknowledgment index
discipline and the GUI yield-window concurrency model. The most serious is that
the GUI routes idle/demoted control alarms through `dispatch_op` (a single-op
dispatch) instead of `run_program`, so a control alarm pointing at a user program
— the entire purpose of a control alarm — silently fails on the GUI. The
existing test masks this because it mocks the IPC return rather than exercising
the core.

## Critical Issues

### CR-01: GUI runs control-alarm programs via `dispatch_op` (Op::Xeq), which never runs a user LBL when idle

**File:** `hp41-gui/src/App.tsx:1233-1240` (consumer); `hp41-core/src/ops/program.rs:69-89` (op_xeq idle behavior); `hp41-gui/src-tauri/src/key_map.rs:427-428` (`xeq_<label>` → `Op::Xeq`)

**Issue:** When an interrupting control alarm is demoted (idle, nested, solver/modal
active, or call-stack-full) or a non-interrupting control alarm fires, `dispatch_alarm_event`
pushes `"alarm:xeq:{label}"` to `event_buffer`. The CLI consumes this correctly by
calling `hp41_core::run_program(&mut self.state, &label)` (`app.rs:1850`). The GUI
consumes it by calling `invoke('dispatch_op', { keyId: \`xeq_${label}\` })`, which
resolves to `Op::Xeq(label)` and dispatches it. But `op_xeq` with `is_running == false`
does **not** run a user program — it only tries `builtin_card_op` then `xrom_resolve`,
and returns `Err(HpError::InvalidOp)` for any user-defined LBL (program.rs:70-84,
explicitly: "No user-label scan here: user-program XEQ goes through run_loop, not op_xeq").
Net effect: every control alarm targeting a user program is silently broken in the GUI
— the user sees an "invalid op" toast instead of their program running. This is a
direct CLI↔GUI parity violation (D-25.6) and defeats the alarm feature on the GUI.

The test `App.test.tsx:391-404` (D5) passes only because it `mockResolvedValueOnce`s
the second invoke; it never asserts the program actually executed.

**Fix:** Route `alarm:xeq:{label}` through the new `run_program` command, not `dispatch_op`:
```ts
} else if (line.startsWith('alarm:xeq:')) {
  const label = line.slice('alarm:xeq:'.length);
  if (busyRef.current) continue;
  busyRef.current = true;
  invoke<CalcStateView>('run_program', { label })
    .then(view => { setCalcState(view); setErrorMessage(null); })
    .catch(err => showToast(extractErrMessage(err)))
    .finally(() => { busyRef.current = false; });
}
```
The returned view may carry `pending_yield` (an alarm program with PSE/VIEW/AVIEW);
`setCalcState` will hand it to the yield-and-resume driver, so this also fixes the
GUI yield path for alarm-launched programs. Update the D5 test to assert
`run_program` is invoked and (ideally) drive a real core program to prove execution.

### CR-02: `pending_interrupt_alarm_index` can be stale at ack-after-RTN, acking/rescheduling the wrong alarm

**File:** `hp41-core/src/ops/time/alarm.rs:456-475` (index captured as loop position); `hp41-core/src/ops/program.rs:567-576` (ack-after-RTN uses captured index)

**Issue:** `check_alarms` captures the alarm's position `i` into `pending_interrupt_alarm_index`
(alarm.rs:459-473). The handler then executes inside `run_loop`, and on the RTN that
returns the call stack to the injection depth, `acknowledge_alarm(state, idx)` is called
with that captured index (program.rs:570-573). But `state.alarms` is a **sorted, mutable
Vec**: an alarm handler that runs `XYZALM` (inserts sorted → shifts indices), `CLALMA`/
`CLALMX` (removes → shifts indices), or `CLRALMS` (empties) invalidates the captured index.
The ack then either (a) reschedules/removes the **wrong** alarm, or (b) hits the
out-of-bounds branch and silently no-ops (`let _ = acknowledge_alarm(...)`), leaving the
intended repeating alarm permanently `past_due` so it never re-fires. The comment at
state.rs:501-504 claims the paired index "disambiguates two alarms sharing a fire time,"
but it does not survive any structural mutation of the Vec during the handler.

This is data-correctness loss for repeating interrupting alarms whose handler touches the
alarm list — a realistic pattern (e.g. a handler that clears or reschedules itself).

**Fix:** Do not carry a positional index across handler execution. Either (a) identify the
alarm by a stable key (e.g. snapshot `trigger_unix` + `alarm_type` and re-find it at ack
time), or (b) capture and ack the alarm by value: pop/clone the `AlarmEntry` at injection
time, and at ack-after-RTN re-insert the rescheduled copy (sorted) rather than indexing.
Add a regression test: handler that calls `CLALMX`/`XYZALM`, then assert the original
repeating alarm rescheduled correctly.

## Warnings

### WR-01: Repeating interrupting alarm is permanently lost when call_stack is at the 4-level cap

**File:** `hp41-core/src/ops/program.rs:520-526`

**Issue:** When `pending_interrupt` is taken but `call_stack.len() >= 4`, the interrupt is
"SILENTLY suppressed" — `pending_interrupt_alarm_index`/`pending_interrupt_depth` are
cleared with no ack and no event pushed. The alarm was already marked `past_due = true`
in `check_alarms` (alarm.rs:461). For a **repeating** alarm, `check_alarms` skips
`past_due` entries forever (alarm.rs:460), and nothing ever resets `past_due` for the
cap-dropped alarm. The comment says "stays past_due for manual ack," but no diagnostic is
surfaced (no `event_buffer` push) and there is no documented manual-ack path from this
state — so the repeating alarm is effectively lost until the user manually rediscovers and
clears it.

**Fix:** On cap-drop, push a diagnostic (e.g. `"alarm:xeq:{label}"` to demote it to the
event path, consistent with the other demotion cases in `dispatch_alarm_event`) so the
alarm still fires once and a repeating alarm can be acked/rescheduled. At minimum, surface
a `alarm:dropped:{label}` event so D-07 (never silently swallow) is honored.

### WR-02: Second past-due interrupting alarm is marked `past_due` AND demoted, then never rescheduled

**File:** `hp41-core/src/ops/time/alarm.rs:456-475` + `:535-554`

**Issue:** `check_alarms` iterates all alarms; for each past-due interrupting alarm it sets
`past_due = true` (line 461) *before* calling `dispatch_alarm_event`. Only the first
satisfies `pending_interrupt.is_none()`; subsequent ones are demoted to
`"alarm:xeq:{label}"` (lines 546-553). So when two interrupting alarms are simultaneously
past-due, the second is both marked `past_due` and queued as an event. If that second
alarm is repeating, it relies on a frontend ack to reschedule — but the GUI ack path is
itself broken (CR-01), and even the CLI path runs the program without rescheduling the
alarm (`drain_event_buffer` alarm:xeq just runs the program). The repeating second alarm
therefore stays `past_due` and never re-fires.

**Fix:** Decide and document the reschedule responsibility for demoted (`alarm:xeq`)
repeating alarms. The demotion path should either reschedule the repeating alarm at demo
time, or the frontend `alarm:xeq` handler must call `acknowledge_alarm` for the matching
alarm. Add a test covering two simultaneously-past-due interrupting repeating alarms.

### WR-03: GUI yield-window allows user input to corrupt a paused program before resume

**File:** `hp41-gui/src/App.tsx:592-606` (yield driver), `:699-710` / `:747` (key dispatch)

**Issue:** While a program is paused at a PSE/VIEW/AVIEW yield, `run_loop` has returned and
`is_running == false`. The yield driver schedules `resume_program` after `resume_ms`, but
nothing blocks the user from pressing keys during that window: `dispatchKeyId`/`handleClick`
only guard against concurrent IPC via `busyRef`, not against the "program is paused"
logical state. A keystroke dispatches `dispatch_op`, mutating `CalcState` (stack, registers,
entry_buf). When `resume_program` fires, the program continues from `pc` against the
user-corrupted state — non-faithful to the HP-41, where the running program owns the
keyboard during a pause. (The `resume_program` setTimeout callback also neither reads nor
sets `busyRef`, so it can interleave with an in-flight keypress IPC.)

**Fix:** Gate calculator key dispatch while `pending_yield` is non-null (treat it like a
"program busy" state, similar to the `is_running`/modal interception that already exists),
and/or set `busyRef.current = true` for the duration of the scheduled resume. Add a test
that presses a key during the yield window and asserts it is ignored (or queued).

### WR-04: Idle/USER-mode/alarm `run_program` call sites that yield are stranded mid-program (CLI)

**File:** `hp41-cli/src/app.rs:1843-1868` (`drain_event_buffer` alarm:xeq path)

**Issue:** `drain_pending_yields` (app.rs:289) only runs after `handle_key`. The
`drain_event_buffer` alarm path (app.rs:1850) calls `run_program` from the main run loop
*after* the yield drain (app.rs:298) and is **not** followed by a `drain_pending_yields`
call. If an alarm-triggered program contains PSE/VIEW/AVIEW, `run_program` returns with
`pending_yield: Some(...)` and the program is stalled — it will not resume until the next
keypress happens to invoke `drain_pending_yields`, and even then the displayed yield text
ordering is off. (USER-mode F1-F4 at app.rs:601 and `try_user_dispatch` at app.rs:1880 are
OK only because they sit inside `handle_key`, so the app.rs:289 drain follows them.)

**Fix:** Call `self.drain_pending_yields(&mut terminal)?` after the alarm-launched
`run_program` in `drain_event_buffer` — or restructure so every `run_program` Ok-path is
uniformly followed by a yield drain (mirrors the existing "wire ALL run_program call sites"
print-drain invariant in CLAUDE.md). Note: `drain_event_buffer` has no `terminal` handle
today, so this needs the same plumbing as the keypress path.

### WR-05: Interrupt handler shares the program stack — X/Y/Z/T not preserved across an interrupting alarm

**File:** `hp41-core/src/ops/program.rs:520-544` (injection); `hp41-core/tests/phase_63_interrupting_alarms.rs:165-211`

**Issue:** The synthetic handler frame is pushed onto the same `call_stack` and runs against
the **same** RPN stack as the interrupted program. Any stack-lifting op in the handler
(arithmetic, `PushNum`, `Xeq`) clobbers the interrupted program's X/Y/Z/T and `lastx`.
On real HP-41 hardware an interrupting alarm handler is expected not to corrupt the running
program's stack state. The test `interrupt_preserves_stack_x_y_z_t_and_lift_state`
sidesteps this by using a handler that only does `STO 6` (no lift), and Scenario 1's
handler (`PushNum(99)`) only checks register side effects, never the post-resume stack — so
the gap is undefended and untested.

**Fix:** Either document this as an accepted divergence (and add a test asserting the
current behavior so it is intentional), or save/restore the stack (X/Y/Z/T/lastx +
lift_enabled) around the injected handler frame, popping it back at the ack-after-RTN
boundary. Given the run-loop already tracks `pending_interrupt_depth`, the restore point is
identifiable.

## Info

### IN-01: MAX_STEPS infinite-loop guard resets per resume segment

**File:** `hp41-core/src/ops/program.rs:496-504`

**Issue:** `run_loop` initializes `steps = 0` on every entry, so the 1,000,000-step guard is
per-`run_program`/`resume_program` segment, not cumulative. A pathological program that
yields (PSE) inside a tight loop will resume indefinitely (~1 op per segment), never
tripping the guard. This is bounded by real-time (1s/yield) so it is not a hang, but the
"infinite-loop guard" is weaker than its docstring implies for yield-heavy programs.

**Fix:** If cumulative bounding is desired, persist a step counter across resumes on
`CalcState` (transient `#[serde(skip)]`) and reset it only in `run_program`, not
`resume_program`. Otherwise, note the per-segment scope in the docstring.

### IN-02: D5 GUI test asserts only the IPC call shape, not program execution — masks CR-01

**File:** `hp41-gui/src/App.test.tsx:391-404`

**Issue:** The `alarm:xeq` test mocks both invokes and asserts only that `dispatch_op` was
called with `xeq_TESTLBL`. Because the mock returns a canned view, the test cannot detect
that `dispatch_op` of a user-LBL XEQ does nothing useful when idle (CR-01). Tests that
assert IPC shape rather than effect provide false confidence here.

**Fix:** After fixing CR-01, assert the `run_program` command is invoked, and add at least
one core-level integration test (or a GUI test driving the real backend) proving an idle
control alarm actually executes the target program.

---

_Reviewed: 2026-06-06_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
