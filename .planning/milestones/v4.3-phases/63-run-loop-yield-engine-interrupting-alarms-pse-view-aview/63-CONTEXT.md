# Phase 63: Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW - Context

**Gathered:** 2026-06-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a **synchronous yield/interrupt point** to `hp41-core`'s `run_loop` (a single-threaded clone-and-loop interpreter) so that, mid-program:

1. **Interrupting control alarms** (`>>label`) execute their stored label program in the interrupted program's register environment, then the original program resumes from exactly where it was halted (ALARM-02, ALARM-03).
2. **PSE** pauses the display ~1 s before the next step (PRGM-01).
3. **VIEW / AVIEW** show their register/ALPHA value briefly (PSE-like) before the program continues — not only after the program ends (PRGM-02).

**In scope:** the run-loop yield engine and the two consumers above, in `hp41-core` plus the CLI + GUI frontend wiring needed to render/resume yields. Requirements: ALARM-02, ALARM-03, PRGM-01, PRGM-02.

**Out of scope:** full DISP-01 (CLI rendering `display_override` generally) — deferred to v4.4; this phase uses a dedicated yield channel instead (see D-04). No new user-authored instructions/Ops. No async, no threads, no `sleep` in `hp41-core`.

</domain>

<decisions>
## Implementation Decisions

### Yield mechanism (PSE/VIEW/AVIEW + alarms)
- **D-01:** Use **yield-and-resume**. `run_loop` *breaks* at PSE/VIEW/AVIEW exactly like `Op::Prompt` does today (program.rs ~660), recording what to display and that a timed auto-resume is owed, then returns control. The frontend renders the value, then auto-resumes via the existing resume path (`resume_program` → re-enter `run_loop`). GUI schedules the resume on its existing `tick_time`/`setInterval` (Mutex is released between yields, so the GUI stays responsive — D-11 honored); CLI sleeps the duration then resumes. **Rejected:** blocking `std::thread::sleep` inside `run_loop` (freezes GUI for the full pause, puts timing into sleep-free core); per-frontend split (CLI sleeps / GUI yields — breaks CLI↔GUI parity D-25.6).
- **D-02:** This single yield boundary is the unifying "yield engine" — alarms, PSE, VIEW, and AVIEW all flow through it.

### CLI display path under DISP-01 deferral
- **D-04:** The **yield signal itself carries the formatted display string** (alongside yield kind + resume duration) on a **dedicated typed yield channel** (a new transient field on `CalcState`, shape is planner's discretion — e.g. `Option<YieldState{kind, text, resume_ms}>`, `#[serde(default, skip)]`). Both frontends read this one channel; the CLI renders the carried text directly. **`display_override` is left untouched**, so DISP-01 stays cleanly deferred to v4.4 — no scope bleed. **Rejected:** pulling DISP-01 partially forward (CLI reads `display_override` during yields); overloading the existing stringly-typed `"PAUSE 1000"` event marker to carry the value.

### Mid-run interrupt granularity (Phase C)
- **D-05:** **Include "Phase C"** — call `check_alarms(state)` *inside* `run_loop` **every ~1000 steps** (ARCHITECTURE.md recommendation). Required because the GUI holds the `AppState` Mutex for the whole `run_loop`, so `tick_time`'s `check_alarms` cannot run mid-program; without an in-loop check a yield-free compute loop would only see an alarm when it ends, failing criterion 1 ("next instruction boundary"). At real execution speed, 1000 steps is sub-millisecond — effectively "next boundary" to a human, at ~one extra call per 1000 ops. **Rejected:** every-instruction check (up to 253× alarm-scan cost per step for imperceptible gain); skip Phase C (fails criterion 1 in the GUI for compute-heavy programs).

### Repeating-alarm acknowledgment (ALARM-03)
- **D-06:** **`run_loop` acknowledges + reschedules the alarm immediately after the synthetic XEQ handler returns** (when `Op::Rtn` pops back to the resume address). Most hardware-faithful (re-arm before the interrupted program continues), self-contained in core, and naturally **skips the ack when the handler never ran** (4-level-cap drop or missing label). **Rejected:** frontend-acknowledges-after-`run_program`-returns (late re-arm; ambiguous under multi-yield; splits alarm logic across core + both frontends); auto-ack in `check_alarms` when it sets `pending_interrupt` (acknowledges *before* the handler runs → wrongly re-arms alarms that get dropped/have a missing label — correctness hazard).
- **D-06a (Claude's discretion):** how `run_loop` identifies *which* alarm to acknowledge — a paired transient field (e.g. `pending_interrupt_alarm_index: Option<usize>`) vs scanning `state.alarms` for the matching past-due entry — is the planner's call. Lean toward the paired field for clarity/avoiding ambiguity when two alarms share a fire time.

### Suppression & edge surfacing (ALARM-03 edges)
- **D-07 (locked by criterion 2, restated):** At the 4-level call-stack cap the interrupt is **suppressed and left `past_due`** for later manual acknowledgment; `state.call_stack` is **never** pushed to a 5th level; `state.is_running` is **restored on all paths including errors**.
- **D-08:** **Cap-drop stays fully silent** (hardware-faithful), but a **missing handler label IS surfaced** (GUI toast / CLI status line) — honoring CLAUDE.md D-07 "NEVER silently swallow an unknown id," since a missing label is a user authoring error, not hardware behavior. **Rejected:** fully silent for both (a typo'd label vanishes, conflicts with D-07); surfacing the cap-drop too (fires noise during legitimately deep 4-level programs, not hardware-faithful).
- **D-09:** **Clear `pending_interrupt` on resume after `Op::Stop`** (drop the interrupt). `resume_program` clears it before re-entering `run_loop`, removing the ambiguous "interrupt set just before STOP" edge; real-HP-41CX behavior at this exact edge is undocumented. Document as a known simplification. **Rejected:** preserve across STOP (fires before the post-STOP instruction — adds resume-ordering complexity for undocumented behavior).
- **D-10:** **Demote the interrupt to a deferred `alarm:xeq:{label}` event when a solver/modal is mid-execution** — i.e. when `integ_state` / `solve_state` / `difeq_state` is `Some` or a `modal_program` is active — instead of injecting a synthetic XEQ frame, to avoid corrupting solver re-entrancy state (Risk 3 / D-43.x isolation discipline). **Rejected:** no guard (synthetic XEQ mid-solve can corrupt `integ/solve/difeq` state).

### Settled upstream — NOT re-decided here (carry forward)
- **D-11:** ALARM-01 prefix semantics are **resolved**: ADR v4.3-003 verdict confirms the current code is **CORRECT** (`>>` ⇒ `interrupting: true`, `>` ⇒ `interrupting: false`). The conditional flip/rename correction plan from Phase 62 (D-05) is **NOT triggered** — Phase 63 needs **no** flag fix. Do not flip the boolean or introduce a `ControlKind` enum.
- **D-12:** New transient field `pending_interrupt: Option<String>` on `CalcState` with `#[serde(default, skip)]`, initialized `None` in `CalcState::new()` and cleared at `run_program`/`resume_program` entry (per ARCHITECTURE.md). No new `Op` variants → the 4-way exhaustive-match invariant is **not** triggered; no JSON pool entry, no `builtin_card_op` registration.
- **D-13 (idle path, no new code):** an interrupting alarm that fires while **no** program is running (`is_running == false`) is queued as `alarm:xeq:{label}` to `event_buffer` and executed by the existing `run_program` path at the next interaction boundary — same path as non-interrupting alarms (criterion 3). `check_alarms` sets `pending_interrupt` only when `is_running == true` and `pending_interrupt.is_none()`; a concurrent second interrupting alarm is demoted to `event_buffer`.

### Claude's Discretion
- Exact shape/naming of the yield-channel field and the alarm-index tracking field (D-04, D-06a).
- Whether VIEW/AVIEW reuse the PSE pause duration or a separate "brief" duration (pick a sensible HP-41-faithful default; both are "PSE-like" per criterion 5).
- Test-fixture strategy for asserting yields/timing without real wall-clock sleeps (e.g. resume_ms surfaced as data and asserted, not slept, in core unit tests).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Re-entrancy & interrupt architecture (primary)
- `.planning/research/ARCHITECTURE.md` — recommended synchronous, single-threaded interrupt mechanism: `pending_interrupt` field, run-loop boundary check, reuse of `call_stack` + `Op::Rtn` resume, Phases A/B/C build order, Risks 1–4, Open Questions 1–3. **The core design source for this phase.**
- `.planning/research/PITFALLS.md` — P-43-01 (`>`/`>>` inversion audit) and run-loop regression pitfalls.

### Alarm prefix semantics (settled)
- `docs/adr/v4.3-003-*.md` — Control-alarm prefix semantics verdict: code is CONFIRMED CORRECT (`>>`=interrupting). Authoritative on why no flag fix is needed (D-11).
- `.planning/research/CONTROL-ALARMS.md` — behavioral study of the three alarm types (note: its prose-name inversion root-caused in ADR v4.3-003).
- `.planning/research/SUMMARY.md` — "Anchor: Interrupting Alarms (D-40-04)".
- `.planning/research/DIVERGENCE-AUDIT.md` — `>>`=interrupting confirmation.

### Divergence record to update
- `docs/hp41-time-divergences.md` §D-40-04 — "Interrupting Control Alarm Re-Entrancy"; currently "deferred / not supported". Phase 63 must flip this to "implemented via pending-interrupt at run-loop boundary (v4.3)".
- `docs/hp41-*-divergences.md` — DISP-01 deferral context (CLI `display_override`).

### Frozen invariants (must honor)
- `CLAUDE.md` — No async / no panics (`#![deny(clippy::unwrap_used)]`); 4-way exhaustive match; save-file backward compat (`#[serde(default)]` / `#[serde(skip)]`); math1 freeze; D-11 no-polling; D-25.6 CLI↔GUI parity; D-07 never-swallow-unknown-id (surface as toast/status); print-buffer discipline (`println!` forbidden in core).

### Phase 62 predecessor
- `.planning/phases/62-alarm-semantics-spec/62-CONTEXT.md` — prior alarm-semantics decisions; the conditional correction plan that this phase does NOT trigger.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Op::Prompt` break pattern** (`hp41-core/src/ops/program.rs` ~658–663): writes `display_override` + breaks `run_loop`. The yield-and-resume mechanism (D-01) generalizes this — break + record an auto-resume.
- **`call_stack` + `Op::Rtn` machinery** (`program.rs` ~500–504, push at `Op::Xeq` ~538–573): the synthetic interrupt frame reuses this verbatim — push `state.pc`, jump to `label+1`, and the existing `Op::Rtn`/program-end pop restores the resume address. **No new resume mechanism needed.**
- **`run_program` / `run_loop`** (`program.rs` ~485–714): clone-and-loop interpreter; existing guards `steps >= MAX_STEPS` (~488) and `pc >= program.len()` (~492) bracket exactly where the interrupt check + Phase-C check are inserted.
- **`check_alarms` / `dispatch_alarm_event`** (`hp41-core/src/ops/time/alarm.rs` ~448–511): already routes `interrupting: true` to the `"alarm:interrupting:deferred"` dead-end that Phase 63 replaces; `interrupting: false` → `alarm:xeq:{label}` deferred path is the template for both idle-fired and demoted interrupts.
- **`Op::Pse`** (`program.rs` ~902): already writes `display_override` + pushes `"PAUSE 1000"` but does NOT break — this is the PRGM-01 gap the yield engine closes.

### Established Patterns
- **No per-frame locals in `run_loop`** — all state lives in `&mut CalcState`, so a synthetic XEQ frame in the same loop invocation is safe (X/Y/Z/T persist; alarm handlers must STO/RCL defensively — Risk 2).
- **Event-buffer draining**: CLI drains via `drain_event_buffer` (`hp41-cli/src/app.rs` ~1778–1815) / `call_dispatch_and_drain` (~1936–1957); GUI via `commands.rs` `handle_tick_time`/`handle_run_stop`/`run_stop`/`request_cancel`. The "ignored" comment for interrupting alarms must be removed.
- **GUI Mutex**: `AppState` Mutex held for the whole `run_loop`; D-05 (Phase C) + D-01 (Mutex released between yields) are what make mid-run detection/responsiveness possible.

### Integration Points
- `hp41-core/src/state.rs` — add transient `pending_interrupt` (+ optional index field, + yield-channel field).
- `hp41-core/src/ops/program.rs` — `run_loop` interrupt check, Phase-C check, ack-after-RTN, PSE/VIEW/AVIEW yield breaks, `resume_program` clear-on-STOP.
- `hp41-core/src/ops/time/alarm.rs` — `dispatch_alarm_event` routing + solver/modal demotion guard.
- `hp41-cli/src/app.rs` + `hp41-gui/src-tauri/src/commands.rs` — drain/resume wiring, yield rendering, missing-label surfacing.

</code_context>

<specifics>
## Specific Ideas

- The phase name "**Run-Loop Yield Engine**" is the unifying frame: build one yield/resume boundary, then make alarms (criteria 1–3) and PSE/VIEW/AVIEW (criteria 4–5) ride it.
- Follow ARCHITECTURE.md's **Phase A → B → C** build order (core state + `check_alarms` routing → `run_loop` interrupt check → periodic `check_alarms`), then layer the PSE/VIEW/AVIEW yield consumers and frontend wiring.
- Update `docs/hp41-time-divergences.md` §D-40-04 and add an ADR (ARCHITECTURE.md suggests `docs/adr/v4.3-001-interrupt-alarm-pending-field.md`) recording the pending-interrupt mechanism.

</specifics>

<deferred>
## Deferred Ideas

- **DISP-01** (general CLI `display_override` rendering) — stays in v4.4; this phase deliberately routes around it via the dedicated yield channel (D-04).
- **BEEP/notification on dropped interrupt** (the rejected "surface both" option in D-08) — not adopted; cap-drop stays silent.
- **Interrupt firing during INTG/SOLVE/DIFEQ** — demoted (D-10), not executed mid-solve; true mid-solver interrupt re-entrancy is out of scope.

</deferred>

---

*Phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview*
*Context gathered: 2026-06-06*
