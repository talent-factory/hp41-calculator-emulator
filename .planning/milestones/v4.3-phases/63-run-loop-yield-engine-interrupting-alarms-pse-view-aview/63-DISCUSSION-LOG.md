# Phase 63: Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-06
**Phase:** 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
**Areas discussed:** PSE/VIEW/AVIEW mid-run rendering, Mid-run interrupt granularity (Phase C), Repeating-alarm acknowledgment, Suppression & edge surfacing

---

## PSE/VIEW/AVIEW mid-run rendering — yield mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Yield-and-resume (break + auto-resume) | run_loop breaks like Op::Prompt, records display + resume duration, returns; frontend renders then auto-resumes (GUI on tick_time, CLI sleeps). Mutex released → GUI responsive; one unifying boundary; CLI↔GUI parity. | ✓ |
| Blocking sleep inside core run_loop | Op::Pse calls thread::sleep(~1s); freezes GUI for the pause, puts sleep into core. | |
| Split: CLI sleeps, GUI yields | Per-frontend optimal but breaks single-mechanism principle + risks D-25.6 divergence. | |

**User's choice:** Yield-and-resume (break + auto-resume)
**Notes:** Becomes the unifying "yield engine" — alarms + PSE/VIEW/AVIEW share the boundary; Mutex released between yields.

---

## PSE/VIEW/AVIEW mid-run rendering — CLI display path under DISP-01 deferral

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated yield channel carries the text | Yield signal carries the formatted string + kind + duration; both frontends read it; display_override untouched → DISP-01 stays deferred. | ✓ |
| Pull DISP-01 partially into v4.3 | CLI reads display_override during yields only; brings deferred v4.4 work forward (scope growth). | |
| Reuse existing 'PAUSE 1000' event marker | Overload stringly-typed marker to carry value; brittle, couples timing+value. | |

**User's choice:** Dedicated yield channel carries the text
**Notes:** Keeps DISP-01 cleanly deferred to v4.4; one typed channel both frontends read.

---

## Mid-run interrupt granularity (Phase C)

| Option | Description | Selected |
|--------|-------------|----------|
| Every ~1000 steps | check_alarms() inside run_loop every 1000 instructions; sub-ms latency, negligible cost; satisfies criterion 1 in both frontends. | ✓ |
| Every instruction | Literal next-boundary fidelity but up to 253× alarm-scan cost per step. | |
| Skip Phase C (yield-boundary only) | Only check at start + yields; yield-free compute loops miss alarms until end → fails criterion 1 in GUI. | |

**User's choice:** Every ~1000 steps
**Notes:** Required because GUI holds the Mutex for the whole run_loop, so tick_time's check_alarms can't run mid-program.

---

## Repeating-alarm acknowledgment

| Option | Description | Selected |
|--------|-------------|----------|
| In run_loop after handler RTNs | run_loop acks/reschedules right after the synthetic XEQ handler returns; self-contained; skips ack when handler never ran. | ✓ |
| Frontend after run_program returns | Late re-arm; ambiguous under multi-yield; splits logic across core + frontends. | |
| Auto in check_alarms | Acks before the handler runs → wrongly re-arms dropped/missing-label alarms (correctness hazard). | |

**User's choice:** In run_loop after handler RTNs
**Notes:** Index-tracking (paired field vs scan) left to the planner; lean to paired field.

---

## Suppression & edge surfacing — surfacing of non-executing interrupts

| Option | Description | Selected |
|--------|-------------|----------|
| Silent cap-drop, surface missing-label | Cap-full suppression silent + past_due (criterion 2); missing label surfaced (GUI toast / CLI status) per D-07. | ✓ |
| Fully silent for both | Typo'd label vanishes with no feedback — conflicts with D-07. | |
| Surface both | Cap-drop notice fires during legit deep programs — non-hardware-faithful noise. | |

**User's choice:** Silent cap-drop, surface missing-label

## Suppression & edge surfacing — pending_interrupt across Op::Stop

| Option | Description | Selected |
|--------|-------------|----------|
| Clear on resume (drop it) | resume_program clears pending_interrupt; removes ambiguous edge; real-HW undocumented. | ✓ |
| Preserve across STOP | Fires before post-STOP instruction; adds resume-ordering complexity for undocumented behavior. | |

**User's choice:** Clear on resume (drop it)

## Suppression & edge surfacing — solver/modal guard

| Option | Description | Selected |
|--------|-------------|----------|
| Demote when solver/modal active | Route interrupt to event_buffer when integ/solve/difeq/modal active; avoids corrupting solver state. | ✓ |
| No guard (allow during solver) | Synthetic XEQ mid-solve can corrupt integ/solve/difeq state. | |

**User's choice:** Demote when solver/modal active

---

## Claude's Discretion

- Exact shape/naming of the yield-channel field and the alarm-index-tracking field.
- Whether VIEW/AVIEW reuse the PSE pause duration or a separate "brief" duration.
- Test-fixture strategy for yields/timing without real wall-clock sleeps.

## Deferred Ideas

- DISP-01 (general CLI display_override rendering) — stays in v4.4.
- BEEP/notification on dropped interrupt — not adopted; cap-drop silent.
- True mid-solver interrupt re-entrancy — out of scope (interrupts demoted during solvers).

## Settled upstream (not re-discussed)

- ALARM-01 prefix semantics resolved by ADR v4.3-003 — code CONFIRMED CORRECT; no flag flip/rename in Phase 63.
