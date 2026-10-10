# Phase 63: Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW — Pattern Map

**Mapped:** 2026-06-06
**Files analyzed:** 6 edited core/frontend files + 1 new test file (no new src files)
**Analogs found:** 7 / 7 (all exact, same-codebase)

This phase adds NO new `hp41-core` src files — it edits existing ones and adds ONE
integration test file. Every unit below maps to an existing in-repo idiom. Copy the
analog, do not invent. English-only docs rule honored throughout.

---

## File Classification

| New/Modified unit | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `pending_interrupt` + yield-channel + alarm-index fields | model (CalcState) | transient state | `event_buffer` / `display_override` / `modal_program` (state.rs) | exact |
| `interrupting:true` arm in `dispatch_alarm_event` | service (routing) | event-driven | the sibling `interrupting:false` arm (alarm.rs:507) | exact |
| `run_loop` interrupt-check + synthetic frame + ack-after-RTN | service (engine) | request-response loop | `Op::Xeq` push + `Op::Rtn` pop (program.rs:538/500) | exact |
| PSE/VIEW/AVIEW yield-break | service (engine) | event-driven break | `Op::Prompt` break (program.rs:660) | exact (PSE), role-match (VIEW/AVIEW) |
| `tests/phase_63_interrupting_alarms.rs` | test | n/a | `time_alarm_latency.rs` + `program_tests.rs` | exact |
| CLI `alarm:missing:` branch + drain | controller (CLI) | event drain | `alarm:xeq:` branch (app.rs:1783) | exact |
| GUI tick/drain mirror | controller (GUI IPC) | event drain | `handle_tick_time` (commands.rs:342) | exact |

---

## Pattern Assignments

### 1. New transient `CalcState` fields (model, transient state)

**File to edit:** `hp41-core/src/state.rs`
**Analog:** `event_buffer` (state.rs:151-152), `display_override` (state.rs:143-144),
`modal_program` (state.rs:218-219), `pending_chisqd_nu` (state.rs:287-288).

**Annotation idiom to copy** — all three new fields use `#[serde(default, skip)]`
(transient: default for backward-compat deserialization, skip so they never serialize):

```rust
/// HP-41 sound event buffer: BEEP and TONE n push structured event lines here.
/// ...
/// Transient — never persisted (`#[serde(default, skip)]`).
#[serde(default, skip)]
pub event_buffer: Vec<String>,
```

Apply the SAME shape to the three new fields. Per ARCHITECTURE.md and D-12 / D-04 /
D-06a, the planner's discretion shapes are e.g.:

```rust
/// Pending interrupting-alarm label awaiting injection at the next run_loop
/// instruction boundary (D-12). Set by `check_alarms` only when `is_running == true`.
/// Cleared at `run_program` / `resume_program` entry (D-09). Transient — never
/// persisted (`#[serde(default, skip)]`).
#[serde(default, skip)]
pub pending_interrupt: Option<String>,

/// Paired index into `state.alarms` identifying WHICH past-due alarm set
/// `pending_interrupt`, so `run_loop` can `acknowledge_alarm` it after the
/// synthetic handler RTNs (D-06a, D-06). Transient — `#[serde(default, skip)]`.
#[serde(default, skip)]
pub pending_interrupt_alarm_index: Option<usize>,

/// Yield channel carrying the formatted display string + kind + resume duration
/// for PSE/VIEW/AVIEW yields (D-04). Read by both frontends; leaves
/// `display_override` untouched so DISP-01 stays deferred. Transient —
/// `#[serde(default, skip)]`.
#[serde(default, skip)]
pub pending_yield: Option<YieldState>,
```

**Initialization site to copy** — `CalcState::new()` at state.rs:466-537 initializes
every field positionally. Add the three new fields alongside the existing transient
inits (the `display_override: None, event_buffer: Vec::new(),` block at state.rs:489-490
is the exact neighbor pattern):

```rust
display_override: None,
event_buffer: Vec::new(),
// Phase 63 (v4.3): run-loop yield engine + interrupting alarms
pending_interrupt: None,
pending_interrupt_alarm_index: None,
pending_yield: None,
```

**`migrate_after_load()` — NO change required.** state.rs:570-602 only migrates
*persistent* fields (xrom bits, stopwatch freeze). Transient `#[serde(skip)]` fields are
always `None`/default after deserialization and need no migration. Confirm by inspection;
do not add a migration arm.

**Divergence to AVOID:** Do NOT mimic the TWO documented `#[serde(default)]`-WITHOUT-`skip`
exceptions — `rand_seed` (state.rs:202-203, see the block-the-PR comment at lines 182-201)
and `adv_tvm_state` (state.rs:381-382), mirrored by `xmem_active_file` (state.rs:441-442).
Those persist on purpose. The Phase 63 fields are transient interrupt/yield bookkeeping —
they MUST carry `skip`. Adding a field without `skip` here is a save-file-pollution bug.

**Test obligation:** the existing `serde_roundtrip` test (state.rs:698-756) asserts each
transient field does NOT appear in serialized JSON — extend it to assert
`!json.contains("pending_interrupt")` etc., copying the `modal_prompt` / `integ_state`
assertions verbatim.

---

### 2. Alarm event routing change (service, event-driven)

**File to edit:** `hp41-core/src/ops/time/alarm.rs`
**Analog:** the sibling `else` branch (non-interrupting) at alarm.rs:507-509 and the
`Message` arm at alarm.rs:495-498, both inside `dispatch_alarm_event` (alarm.rs:493-512).

**Current dead-end to replace** (alarm.rs:503-509):

```rust
if *interrupting {
    state
        .event_buffer
        .push("alarm:interrupting:deferred".to_string());
} else {
    state.event_buffer.push(format!("alarm:xeq:{label}"));
}
```

**Routing idiom to copy** — the `interrupting:false` arm shows the exact "stringly-typed
event push" pattern (`format!("alarm:xeq:{label}")`). The new `interrupting:true` arm
follows ARCHITECTURE.md: set `pending_interrupt` only when running AND none pending,
else demote to the SAME `alarm:xeq:{label}` event the non-interrupting arm emits (D-13
idle path, D-10 solver/modal demotion):

```rust
if *interrupting {
    // D-13/D-10: only inject when a program is running, no interrupt already
    // pending, and no solver/modal is mid-execution; otherwise demote to the
    // proven non-interrupting event path (idle-fire / concurrent-second / solver guard).
    let solver_active = state.integ_state.is_some()
        || state.solve_state.is_some()
        || state.difeq_state.is_some()
        || state.modal_program.is_some();
    if state.is_running && state.pending_interrupt.is_none() && !solver_active {
        state.pending_interrupt = Some(label.clone());
        // D-06a: pair the alarm index so run_loop can ack the right one after RTN.
    } else {
        state.event_buffer.push(format!("alarm:xeq:{label}"));
    }
} else {
    state.event_buffer.push(format!("alarm:xeq:{label}"));
}
```

**Divergence to AVOID (DNT-05):** do NOT touch the `interrupting:false` arm or the
`Message` arm — they are tested and correct. Only the `if *interrupting` block changes.
Note `dispatch_alarm_event` receives a CLONED `&AlarmEntry` (alarm.rs:455-463), not an
index — the index for D-06a must be threaded from `check_alarms` (the `for i in 0..len`
loop at alarm.rs:451-464 already has `i`), not recovered inside `dispatch_alarm_event`.

---

### 3. run_loop boundary check + synthetic frame + ack-after-RTN (service, engine loop)

**File to edit:** `hp41-core/src/ops/program.rs`
**Analogs (all in `run_loop`, program.rs:485-715):**
- `MAX_STEPS` guard (program.rs:488-490) and `pc >= program.len()` break (program.rs:492-495)
  — the exact insertion bracket for the interrupt check + Phase-C check.
- `Op::Xeq` push idiom (program.rs:538-550): `call_stack.len() >= 4` guard, then
  `state.call_stack.push(state.pc); state.pc = target + 1;`.
- `Op::Rtn` pop idiom (program.rs:500-505): `call_stack.pop()` → `state.pc = return_pc`,
  or `break` when empty.

**Interrupt-injection idiom to copy** — insert between `steps += 1` (program.rs:491) and
the `pc >= program.len()` break (program.rs:492), reusing the `Op::Xeq` push verbatim:

```rust
steps += 1;
// Phase 63 (v4.3): interrupt-injection boundary. `state.pc` points at the NEXT
// instruction (previous op fully executed) — exactly Op::Xeq's resume contract.
if let Some(label) = state.pending_interrupt.take() {
    // D-07: 4-level cap is load-bearing — never push a 5th frame. On a full
    // stack the interrupt is silently dropped (alarm stays past_due), mirroring
    // the Op::Xeq CallDepth guard but WITHOUT erroring (hardware drops, not faults).
    if state.call_stack.len() < 4 {
        match find_in_program(program, &label) {
            Ok(target) => {
                state.call_stack.push(state.pc); // copy of Op::Xeq:548
                state.pc = target + 1;           // copy of Op::Xeq:549
                // D-06: ack/reschedule the alarm now that the handler frame is live.
                if let Some(idx) = state.pending_interrupt_alarm_index.take() {
                    let _ = crate::ops::time::alarm::acknowledge_alarm(state, idx);
                }
            }
            Err(_) => {
                // D-08: missing label is a user authoring error — surface it.
                state.event_buffer.push(format!("alarm:missing:{label}"));
            }
        }
    }
    // (cap-drop: D-07 — leave past_due, push nothing, fully silent)
}
if state.pc >= program.len() {
    break;
}
```

The synthetic frame needs NO new resume mechanism: when the handler hits `Op::Rtn` or
runs off the end, the existing `Op::Rtn` pop (program.rs:500-504) / end-of-program break
(program.rs:492-494) restores `state.pc` to the pushed resume address automatically.

**Phase-C idiom (D-05):** call `check_alarms` every ~1000 steps inside the same loop,
gated on a `steps % 1000 == 0` style boundary, placed right beside the interrupt check.
`check_alarms` (alarm.rs:448) is the existing entry point — no new helper.

**`run_program` / `resume_program` clear-on-entry (D-09, D-12):** `run_program`
(program.rs:422-452) clears `call_stack` at line 445; add `state.pending_interrupt = None;`
(and the index field) there. `resume_program` (program.rs:468-477) must clear it before
`run_loop` at line 474 so a stale interrupt set just before `Op::Stop` does not redirect
the resume.

**Divergence to AVOID:** the interrupt cap-drop must NOT `return Err(HpError::CallDepth)`
like `Op::Xeq` (program.rs:539-540) does — the alarm interrupt is dropped silently
(D-07/D-08), not propagated as a fault. Also preserve the Pitfall-2 `is_running` reset
discipline (program.rs:450/475): never `?`-propagate through the cleanup.

---

### 4. PSE / VIEW / AVIEW yield break (service, event-driven break)

**File to edit:** `hp41-core/src/ops/program.rs` (Op::Pse arm), `display_ops.rs` (VIEW/AVIEW)
**Analog:** `Op::Prompt` arm in `run_loop` (program.rs:660-663) — the canonical
"write display, then break run_loop" idiom.

**Current PSE gap** (program.rs:902-907) — writes display + pushes `"PAUSE 1000"` but does
NOT break, so the pause never actually halts execution:

```rust
Op::Pse => {
    let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode);
    state.display_override = Some(formatted);
    state.event_buffer.push("PAUSE 1000".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Break idiom to generalize** — `Op::Prompt` (program.rs:660-663) is the template:

```rust
Op::Prompt => {
    state.display_override = Some(state.alpha_reg.chars().take(24).collect::<String>());
    break;
}
```

Per D-04, PSE/VIEW/AVIEW must become `run_loop` arms that populate the NEW yield channel
(`pending_yield`, carrying kind + formatted text + resume_ms) and `break` — exactly like
`Op::Prompt` breaks. Do NOT route through `display_override` (DISP-01 stays deferred).
Note: `Op::Pse` is currently handled in `execute_op` (the program.rs:902 dispatch path),
but `Op::Prompt` is a dedicated `run_loop` arm (program.rs:660) — PSE must MOVE to a
`run_loop` arm to be able to `break`, mirroring the Prompt precedent (see the comment at
display_ops.rs:38-40 explaining Prompt's run_loop arm has its own inline body).

**VIEW/AVIEW display-string source** — `op_view` (display_ops.rs:17-26) produces the
formatted string via `format_hpnum(&val.numeric_or_zero(), &state.display_mode)`;
`op_aview` (display_ops.rs:30-34) produces `state.alpha_reg.chars().take(24).collect()`.
These are the exact strings the yield channel must capture. The planner adds `run_loop`
arms for VIEW/AVIEW (when running) that compute the SAME string, stash it in
`pending_yield`, and `break` — leaving the existing `op_view`/`op_aview` interactive
(non-program) path on `display_override` unchanged.

**Divergence to AVOID:** do NOT keep the legacy `event_buffer.push("PAUSE 1000")` marker
as the value carrier (D-04 explicitly rejects overloading the stringly-typed marker). The
formatted value rides the typed yield channel instead.

---

### 5. New integration test file (test)

**File to create:** `hp41-core/tests/phase_63_interrupting_alarms.rs`
**Analogs:** `hp41-core/tests/time_alarm_latency.rs` (alarm construction + check_alarms
assertions) and `hp41-core/tests/program_tests.rs` (labeled-program loading + run_program).
Also the in-module alarm tests at `alarm.rs:537+`.

**File-level convention to copy** (time_alarm_latency.rs:1-19): the Free42-oracle header
comment, the `#![allow(clippy::unwrap_used)]` attribute, and the imports:

```rust
// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Interrupting control-alarm re-entrancy tests for ALARM-02 / ALARM-03 (v4.3).
#![allow(clippy::unwrap_used)]

use hp41_core::ops::program::run_program;
use hp41_core::ops::time::alarm::{check_alarms, acknowledge_alarm, AlarmEntry, AlarmType};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
```

**Deterministic alarm-construction helper to copy** (time_alarm_latency.rs:23-30, and the
PITFALLS.md `make_past_due_interrupting_alarm` recipe). Use `trigger_unix = 0` (or `1000`
as time_alarm_latency does) so `trigger_unix <= now` is permanently true; leave
`time_offset_secs = 0`:

```rust
fn past_due_interrupting_alarm(label: &str) -> AlarmEntry {
    AlarmEntry {
        trigger_unix: 0,        // Unix epoch — always in the past
        repeat_secs: 0,
        alarm_type: AlarmType::Control { label: label.to_string(), interrupting: true },
        past_due: false,
    }
}
```

**Program-loading helper to copy** (program_tests.rs:113-127): set `state.program = vec![...]`
with `Op::Lbl("MAIN".to_string())` markers and call `run_program(&mut s, "MAIN")`:

```rust
let mut s = CalcState::new();
s.program = vec![
    Op::Lbl("MAIN".to_string()),
    /* ... 1, STO 10, 2, STO 11 ... */
    Op::Rtn,
    Op::Lbl("ALARM".to_string()),
    /* ... 99, STO 12 ... */
    Op::Rtn,
];
s.alarms.push(past_due_interrupting_alarm("ALARM"));
run_program(&mut s, "MAIN").unwrap();
```

**Assertion style to copy** (time_alarm_latency.rs:58-68): assert on `CalcState` fields
only — `state.regs[..]`, `state.call_stack.is_empty()`, `state.is_running`,
`state.event_buffer.contains(&"alarm:missing:ALARM".to_string())` — NEVER on
`SystemTime::now()` (PITFALLS.md "Avoiding Wall-Clock Races"). Map the 7 scenarios from
PITFALLS.md "Verification Strategy" (basic interrupt, stack/PC preservation, 4-level cap,
nested-blocked, idle-fire, non-interrupting regression, message regression).

**Divergence to AVOID:** do not call `std::thread::sleep` or assert resume timing against
the wall clock — surface `resume_ms` as DATA on the yield channel and assert the value
(Claude's Discretion bullet in CONTEXT.md).

---

### 6. CLI drain wiring (controller, event drain)

**File to edit:** `hp41-cli/src/app.rs`
**Analog:** the `alarm:xeq:` branch in `drain_event_buffer` (app.rs:1783-1796) and the
print-drain idiom in `call_dispatch_and_drain` (app.rs:1936+) / `drain_and_show_print_output`
(app.rs:1865).

**Branch idiom to copy** (app.rs:1783-1796) — `strip_prefix` → run_program → drain card +
print buffers on success, route error to `self.message`:

```rust
} else if let Some(label) = event.strip_prefix("alarm:xeq:") {
    let label = label.to_string();
    match hp41_core::run_program(&mut self.state, &label) {
        Ok(()) => {
            let card_err = self.drain_pending_card_op();
            self.drain_and_show_print_output(card_err);
        }
        Err(e) => self.message = Some(format!("Alarm XEQ {label}: {e}")),
    }
}
```

**New `alarm:missing:` branch (D-08)** — add a sibling `strip_prefix("alarm:missing:")`
arm that surfaces the missing label to `self.message` (CLI status line analog of the
`alarm:message:` arm at app.rs:1781-1782), per D-07 never-swallow:

```rust
} else if let Some(label) = event.strip_prefix("alarm:missing:") {
    self.message = Some(format!("Alarm: missing label {label}"));
}
```

**Remove the dead comment:** app.rs:1797 `// "alarm:interrupting:..." and other events
are silently ignored.` — the interrupting path now flows through `pending_interrupt`
inside `run_loop`, so the CLI no longer needs an `alarm:interrupting:` event branch; only
the `alarm:missing:` and existing `alarm:xeq:` branches remain. Update the doc-comment at
app.rs:1775-1777 accordingly.

**Divergence to AVOID:** any new `run_program` call site MUST wire `drain_pending_card_op`
+ `drain_and_show_print_output` (I-07 print-buffer discipline) — but note the interrupting
handler runs INSIDE the original `run_program`'s `run_loop`, so its print output drains via
the SAME existing call site that started MAIN; do NOT add a second nested `run_program`.

---

### 7. GUI tick / drain wiring (controller, IPC event drain)

**File to edit:** `hp41-gui/src-tauri/src/commands.rs`
**Analog:** `handle_tick_time` (commands.rs:342-349), `handle_op_finalize`
(commands.rs:291-304), `handle_get_state` (commands.rs:313-317) — all three drain
`event_buffer` into `event_lines` and build the `CalcStateView`.

**Drain idiom to mirror** (commands.rs:345-348) — `check_alarms` first, then drain both
buffers into the view (D-25.6 parity: the GUI sees the SAME `alarm:missing:` /
`alarm:xeq:` event lines the CLI sees):

```rust
hp41_core::ops::time::alarm::check_alarms(calc);
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
Ok(CalcStateView::from_state(calc, print_lines, event_lines))
```

The GUI work is largely VERIFICATION (ARCHITECTURE Phase E): the existing
`event_buffer → event_lines` drain at commands.rs:302/315/347 already carries the new
`alarm:missing:{label}` event to the frontend unchanged. The frontend surfaces it as a
toast (D-08), mirroring how non-interrupting `alarm:xeq:LABEL` lines already drive
`dispatch_op("xeq_LABEL")`. The yield channel (`pending_yield`) is projected through
`CalcStateView::from_state` for the GUI to render the PSE/VIEW/AVIEW value and schedule the
auto-resume on its existing `tick_time`/`setInterval` (D-01, D-11 no-polling honored).

**Divergence to AVOID:** no new Tauri command and no new capability/permission TOML — the
interrupt mechanism is Rust-side inside `run_loop`; the GUI reuses `tick_time` /
`dispatch_op` / the existing drain. After editing commands.rs, run the
`cargo check --target aarch64-apple-ios` mobile gate (project memory:
`project_cfg_mobile_gate_blindspot`) since `from_state` projection touches a shared path.

---

## Shared Patterns

### Transient-field serde discipline
**Source:** `state.rs:143-152` (`display_override` / `event_buffer`)
**Apply to:** all three new `CalcState` fields — `#[serde(default, skip)]` + a doc-comment
ending "Transient — never persisted (`#[serde(default, skip)]`)." NEVER the bare
`#[serde(default)]` shape reserved for `rand_seed` / `adv_tvm_state` / `xmem_active_file`.

### Stringly-typed event push
**Source:** `alarm.rs:496/508` (`format!("alarm:message:{msg}")` / `format!("alarm:xeq:{label}")`)
**Apply to:** the new `alarm:missing:{label}` event (alarm.rs + run_loop) — same
`format!("alarm:<kind>:<payload>")` convention both frontends `strip_prefix` on.

### call_stack push/pop + 4-level cap
**Source:** `program.rs:538-550` (Op::Xeq push) + `program.rs:500-504` (Op::Rtn pop) +
`program.rs:539/525` (cap guard)
**Apply to:** the synthetic interrupt frame — push `state.pc`, jump `target + 1`, rely on
the existing Rtn/end-of-program pop to resume. Cap-drop is SILENT (no `Err`), unlike Xeq.

### is_running safety reset (Pitfall 2)
**Source:** `program.rs:448-451` / `program.rs:473-476`
**Apply to:** any path that mutates `is_running` during interrupt handling — capture
`run_loop` result into a `let`, reset, then return; never `?`-short-circuit the cleanup.

### run_program call-site drain
**Source:** `app.rs:1786-1792` + `app.rs:1936+` / `commands.rs:301-303,346-347`
**Apply to:** every place a program (or alarm handler) can emit print/card output —
print_buffer + pending_card_op + event_buffer all drain at the existing single call site.

---

## No Analog Found

None. Every Phase 63 unit maps to an existing in-repo idiom (same codebase, same module
family). The phase is deliberately scoped to generalize existing patterns
(`Op::Prompt` break, `Op::Xeq` push, transient `#[serde(default, skip)]` fields,
`alarm:xeq:` event routing) rather than introduce new mechanisms.

## Metadata

**Analog search scope:** `hp41-core/src/state.rs`, `hp41-core/src/ops/time/alarm.rs`,
`hp41-core/src/ops/program.rs`, `hp41-core/src/ops/display_ops.rs`,
`hp41-core/tests/{time_alarm_latency,program_tests}.rs`, `hp41-cli/src/app.rs`,
`hp41-gui/src-tauri/src/commands.rs`
**Files scanned:** 8
**Pattern extraction date:** 2026-06-06
