# Phase 62: Alarm Semantics Spec - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-06
**Phase:** 62-alarm-semantics-spec
**Areas discussed:** Prefix resolution, Verification bar, Code-fix plan, Contract home

---

## Gray-area selection

Four genuinely-open areas were offered (rest pre-decided in STATE.md Accumulated
Context: sync mechanism, depth-4 suppress, idle-fire, non-interrupting path). User
selected **all four**.

---

## Prefix resolution

| Option | Description | Selected |
|--------|-------------|----------|
| `>`=interrupt, `>>`=defer | Trust OM Vol 2 behavioral prose (research recommendation); implies current code's `interrupting:true`-on-`>>` is inverted → contract declares a flip needed in Phase 63. | |
| `>>`=interrupt, `>`=defer | Trust QREF labels + existing D-40-04 / code convention. No flip. | |
| Document both, resolve via verification | Contract records the conflict + both readings; final mapping contingent on the Verification-bar outcome. No guess committed. | ✓ |

**User's choice:** Document both, resolve via verification.
**Notes:** Appropriately cautious given the milestone-anchor risk (shipping the behavior
exactly backwards). The mechanism is prefix-agnostic, so deferring the prefix lock does
not block the engine design.

---

## Verification bar

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 62 researcher re-reads primary OM | `/gsd-plan-phase 62` researcher pulls a clean OM scan, cites exact page, resolves the prefix. Finalized in THIS phase. | ✓ |
| Researcher + your explicit sign-off | Same re-read, plus owner confirmation before commit (human-in-the-loop gate). | |
| Provisional lock now, verify in Phase 66 | Accept research's `>`=interrupt provisionally; hard re-verification folded into the Phase 66 VERIFY gate. | |

**User's choice:** Phase 62 researcher re-reads primary OM.
**Notes:** Resolution stays inside Phase 62 — not deferred to closeout. Free42 ruled out
as a cross-check (HP-42S, no XYZALM alarms); valid corroboration is a clean primary-OM
page or hardware. Owner sign-off is informative, not a hard gate.

---

## Code-fix plan

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: flip the boolean mapping | `parse_alarm_type` maps `>`→true, `>>`→false; field name kept; old saves re-parsed. Smallest diff vs ~3300-test surface. | |
| Rename to a self-documenting type | Replace `interrupting: bool` with a `ControlKind`/`AlarmPrefix` enum; accept serde/save-compat migration (D-38.8). | |
| You decide (planner picks) | Contract states only the goal (OM-correct + unambiguous); Phase 63 planner chooses flip-vs-rename after save-compat analysis. | ✓ |

**User's choice:** You decide (planner picks).
**Notes:** Contract pins the constraints (OM-correct, unambiguous, save-file back-compat
preserved) but leaves the flip-vs-rename mechanism to the Phase 63 planner. Conditional —
only applies if the OM confirms the flag is inverted.

---

## Contract home

| Option | Description | Selected |
|--------|-------------|----------|
| New ADR v4.3-003 | Formal decision record beside v4.3-001/002. D-40-04 update (already a Phase 66 deliverable) cross-refs it. | ✓ |
| Inline in D-40-04 only | Write resolution into docs/hp41-time-divergences.md §D-40-04 now; no new ADR. | |
| Both, fully now | New ADR AND update D-40-04 immediately in Phase 62. | |

**User's choice:** New ADR v4.3-003.
**Notes:** Clean split — ADR = the decision (Phase 62); divergence-doc D-40-04 = reflected
state, updated at closeout (Phase 66, already a success criterion). Avoids double-editing
D-40-04.

---

## Claude's Discretion

- Flip-vs-rename correction strategy (Code-fix plan) → delegated to the Phase 63 planner,
  constrained by OM-correctness + unambiguity + save-file back-compat.
- ADR filename slug + internal structure of `v4.3-003` → follow existing `docs/adr/`
  conventions.

## Deferred Ideas

None — discussion stayed within phase scope.
