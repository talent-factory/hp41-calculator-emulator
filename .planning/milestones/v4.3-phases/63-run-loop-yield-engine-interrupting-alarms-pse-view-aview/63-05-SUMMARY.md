---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
plan: "05"
subsystem: docs
tags: [interrupting-alarms, adr, divergence-doc, phase-g, docs-only]
dependency_graph:
  requires:
    - run_loop interrupt boundary + Phase-C + ack-after-RTN (63-02)
    - pending_interrupt / pending_interrupt_depth / pending_interrupt_alarm_index fields (63-01)
    - alarm prefix semantics confirmed (ADR v4.3-003 / Phase 62)
  provides:
    - ADR v4.3-004-interrupt-alarm-pending-field.md (Phase G documentation)
    - docs/hp41-time-divergences.md §D-40-04 flipped to implemented (v4.3)
  affects:
    - docs/adr/v4.3-004-interrupt-alarm-pending-field.md (new)
    - docs/hp41-time-divergences.md (§D-40-04 updated)
tech_stack:
  added: []
  patterns:
    - "Docs-only plan: ADR + divergence entry flip"
key_files:
  created:
    - docs/adr/v4.3-004-interrupt-alarm-pending-field.md
  modified:
    - docs/hp41-time-divergences.md
decisions:
  - "ADR numbered v4.3-004 (not v4.3-001 as planned) — v4.3-001 through v4.3-003 were already allocated to single-instance guard, macOS global hotkey, and alarm prefix semantics; next available number is v4.3-004 (Rule 1 naming-collision auto-fix)"
  - "§D-40-04 title updated from 'Deferral' to 'Re-Entrancy — Implemented' to reflect status flip"
  - "Phase 66 verification note embedded in both the ADR and the divergence entry"
metrics:
  duration: "~4 minutes"
  completed: "2026-06-06"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 2
---

# Phase 63 Plan 05: Documentation (Phase G) — ADR + Divergence Entry Summary

**One-liner:** ADR v4.3-004 documents the pending-interrupt run-loop-boundary mechanism as-built; §D-40-04 in hp41-time-divergences.md flipped from deferred to implemented (v4.3) with full edge-case prose and ADR cross-reference.

---

## What Was Built

### Task 1 — ADR v4.3-004-interrupt-alarm-pending-field.md (1384370)

New ADR created at `docs/adr/v4.3-004-interrupt-alarm-pending-field.md` following the existing ADR format used by the 23 prior ADRs in the repo. Records the as-built Phase 63 mechanism:

- **Transient control fields** (`pending_interrupt`, `pending_interrupt_alarm_index`, `pending_interrupt_depth`, `pending_yield`) all `#[serde(default, skip)]` — no save-file migration needed.
- **Phase-C alarm scan** inside `run_loop` (`steps == 1 || steps.is_multiple_of(1000)`) — required for GUI Mutex scenario where external `tick_time` cannot fire mid-program.
- **Interrupt injection boundary** — `pending_interrupt.take()` → push `state.pc` onto `call_stack` → jump to handler label (synthetic XEQ frame; reuses existing machinery, no new `Op` variant).
- **Ack-after-RTN** — `pending_interrupt_depth` gate fires `acknowledge_alarm` only when `Op::Rtn` pops back to the injection depth; handler's nested XEQ calls skip the gate.
- **Clear-on-entry** — `run_program` + `resume_program` both clear all four transient fields.
- **Edge handling table**: 4-level cap suppress (D-07), missing label surface (D-08), STOP clear (D-09), solver/modal demotion (D-10), idle path (D-13), repeating-alarm reschedule (D-06).
- **Relationship to ADR v4.3-003** — builds on the alarm-prefix semantics confirmation (`>>`=interrupting correct; no flag fix needed).
- **Alternatives considered**: per-instruction scan (rejected — CPU overhead), parallel thread (rejected — architectural violation), frontend ack (rejected — late re-arm), auto-ack at check_alarms (rejected — correctness hazard).

### Task 2 — §D-40-04 flipped to implemented (bf9957c)

Updated `docs/hp41-time-divergences.md` §D-40-04:

- **Title changed**: "Interrupting Control Alarm Deferral — Re-Entrancy Not Supported" → "Interrupting Control Alarm Re-Entrancy — Implemented via Pending-Interrupt at Run-Loop Boundary (v4.3)".
- **Status banner added**: "IMPLEMENTED in Phase 63 (v4.3). Verified in Phase 66."
- **"Our behavior" section rewritten**: documents the as-built mechanism (Phase-C scan, synthetic XEQ frame, ack-after-RTN, idle path, all six edge cases with decision IDs D-06/07/08/09/10/13).
- **Rationale section updated**: explains the synchronous approach, Mutex constraint, no-new-Op-variant property, save-file compat.
- **See section updated**: cross-references ADR v4.3-004, all relevant source files, and all Phase 63 + Phase 38 decision IDs.
- **Last-updated line updated**: 2026-06-06 with plan attribution.
- All other divergence entries (D-40-01 through D-40-06) are byte-for-byte unchanged.

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ADR number collision — v4.3-001 already allocated**
- **Found during:** Task 1 file creation
- **Issue:** Plan 63-05 specifies `docs/adr/v4.3-001-interrupt-alarm-pending-field.md` but `docs/adr/v4.3-001-single-instance-guard.md` was already created during Phase 63 planning (v4.3 milestone). Creating a second `v4.3-001` would overwrite an existing ADR.
- **Fix:** Used next available number: `v4.3-004-interrupt-alarm-pending-field.md`. All must_haves checks (grep for `pending_interrupt`, cross-ref to `v4.3-003`) pass against the v4.3-004 filename. The `v4.3-004` number is noted in the ADR footer and in this SUMMARY.
- **Files modified:** docs/adr/v4.3-004-interrupt-alarm-pending-field.md (created at corrected path)
- **Commits:** 1384370 (ADR), bf9957c (divergence entry referencing v4.3-004)

---

## Known Stubs

None — this is a docs-only plan. All prose describes as-built behavior from Plans 63-01 and 63-02.

---

## Threat Flags

None. Docs-only plan; no runtime code, no trust boundary crossed.

---

## Self-Check: PASSED

- `docs/adr/v4.3-004-interrupt-alarm-pending-field.md` — FOUND
- `grep -q "pending_interrupt" docs/adr/v4.3-004-interrupt-alarm-pending-field.md` — FOUND
- `grep -q "v4.3-003" docs/adr/v4.3-004-interrupt-alarm-pending-field.md` — FOUND
- `grep -n "D-40-04" docs/hp41-time-divergences.md` — 2 matches (heading + last-updated)
- `grep -q "implemented" docs/hp41-time-divergences.md` — FOUND
- `grep -q "v4.3-004" docs/hp41-time-divergences.md` — FOUND
- Commit 1384370 (Task 1) — FOUND in git log
- Commit bf9957c (Task 2) — FOUND in git log
- `git status` — Only pre-existing dirty file `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` remains; no unintended changes
- `git diff HEAD -- hp41-core/ hp41-cli/ hp41-gui/src-tauri/src/` — EMPTY (docs-only)
