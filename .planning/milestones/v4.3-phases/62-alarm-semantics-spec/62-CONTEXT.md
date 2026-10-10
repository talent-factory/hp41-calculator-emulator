# Phase 62: Alarm Semantics Spec - Context

**Gathered:** 2026-06-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Produce the **written behavioral contract** that locks the `>` / `>>` control-alarm
prefix semantics against the primary HP-41 Operating Manual, resolving the inversion
question the audit surfaced, *before* any alarm runtime code is written (that begins
in Phase 63).

**This is a documentation / spec-only phase.** No runtime behavior changes. No new
`Op` variants, no engine code. `just ci` stays green throughout. The single
deliverable is a locked decision record (ADR) that downstream phases build against.

**In scope:** verify the prefix→behavior mapping against the OM; lock the contract;
audit the existing `interrupting` flag + `parse_alarm_type` logic and note (not apply)
a correction plan if inverted; cover the required edge-case behaviors in the contract
(4-level cap, idle-fire, non-interrupting path unchanged).

**Out of scope (Phase 63+):** implementing interrupting-alarm execution, the
`run_loop` yield mechanism, `pending_interrupt` field, any code change to
`alarm.rs` / `program.rs` / `state.rs`.
</domain>

<decisions>
## Implementation Decisions

### Prefix resolution (ALARM-01 — the blocking question)
- **D-01:** The contract does **not** guess the mapping. It **documents both readings
  and the source conflict**, then resolves via the verification step (D-03). The two
  readings on record:
  - `CONTROL-ALARMS.md` reading of OM Vol 2 behavioral *prose*: `>label` (control) =
    **interrupts** a running program; `>>label` (conditional) = **defers** (idle/off only).
  - QREF labels + existing `D-40-04` + current code (`parse_alarm_type` → `>>` =
    `interrupting: true`): `>>` = interrupting, `>` = non-interrupting. The QREF's
    own "Interrupting/Noninterrupting" labels read *backwards* vs the behavioral prose.
- **D-02:** Note the **citation discrepancy** in the contract: `D-40-04` cites the
  *82182A Time Module OM* (HP 00041-90035), while `CONTROL-ALARMS.md` cites the
  *HP-41CX OM Vol 2*. The conflict may be a genuine notation difference between the two
  manuals **or** a transcription/OCR slip on either side — the verification step must
  disambiguate which manual + page is authoritative.

### Verification bar (how the mapping is settled)
- **D-03:** **Finalize the mapping in this phase.** `/gsd-plan-phase 62` spawns a
  phase-researcher whose mandatory task is to pull a **clean scan** of the primary OM
  (HP-41CX OM Vol 2 §XYZALM and/or the 82182A Time Module OM) from a trusted archive
  (hpmuseum.org / TOS), **cite the exact page**, and resolve the prefix. The resolution
  is locked here — **not** deferred to the Phase 66 VERIFY gate.
- **D-04:** Free42 is explicitly **not** a valid cross-check for this — it is an HP-42S
  emulator with no Time-module/XYZALM alarms. Acceptable corroboration is a clean
  primary-OM page or (if available) hardware behavior. The researcher's finding will be
  surfaced to the owner, but owner sign-off is **not** a hard gate.

### Code-fix plan (SC#2 — conditional correction)
- **D-05:** The contract carries a correction plan that is **conditional** on the OM
  verdict (only applies if the flag is confirmed inverted). The **strategy is left to
  the Phase 63 planner** ("you decide") — the contract does **not** pre-commit
  flip-vs-rename. The contract states only the **goal + constraints** the fix must meet:
  1. final mapping is **OM-correct**, and
  2. **unambiguous** (the recurring `>`/`>>` confusion must not be able to silently
     re-invert), and
  3. **save-file back-compat preserved** — the alarm catalog is serialized
     (`AlarmType::Control { interrupting }` persists per D-38.8), so existing v4.x
     stored alarms must be re-parsed / normalized / migrated on load, not silently
     re-interpreted under flipped semantics.
- Reference shapes for the planner (illustrative, not prescriptive): minimal boolean
  flip in `parse_alarm_type` (smallest diff vs the ~3300-test surface) **or** replace
  `interrupting: bool` with a self-documenting `ControlKind`/`AlarmPrefix` enum
  (Interrupting | Conditional) that kills the ambiguity permanently.

### Contract home (SC#1)
- **D-06:** The locked contract lives in a **new ADR `docs/adr/v4.3-003-*.md`**
  (alongside the existing `v4.3-001` / `v4.3-002`). ADR = the decision record.
- **D-07:** `docs/hp41-time-divergences.md §D-40-04` is **not** edited in Phase 62.
  Its update is already a Phase 66 success criterion ("update D-40-04 to reflect
  resolution") — Phase 66 updates D-40-04 to reflect the closed state and cross-refs
  the new ADR. Clean split: **ADR = decision (now), divergence-doc = reflected state
  (Phase 66).**

### Claude's Discretion
- **D-05** delegates the flip-vs-rename correction strategy to the Phase 63 planner
  (constrained by the three goals above). No strategy is pre-committed in this phase.
- ADR filename slug and exact internal structure of `v4.3-003` left to the planner /
  executor, following the existing `docs/adr/` ADR conventions.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary research — alarm semantics (the contract source material)
- `.planning/research/CONTROL-ALARMS.md` — the deep behavioral study of the three
  alarm types and the `>`/`>>` reading; the ⚠️ BLOCKING inversion open-question. The
  single most important input to this phase.
- `.planning/research/SUMMARY.md` — v4.3 audit synthesis; "Anchor: Interrupting Control
  Alarms (D-40-04)" section + the explicit "`>` / `>>` naming inversion" blocker.
- `.planning/research/DIVERGENCE-AUDIT.md` — the audit that (reading the same code)
  followed the existing `>>`=interrupting convention; the counter-reading on record.
- `.planning/research/ARCHITECTURE.md` — the recommended synchronous, single-threaded
  interrupt mechanism (informs why "the mechanism is correct regardless of which prefix
  interrupts; only the routing condition flips"). Phase 63 detail, but grounds the contract.
- `.planning/research/PITFALLS.md` — P-43-01 (`>`/`>>` inversion — audit the flag before
  any implementation) and the broader run_loop regression guardrails.

### Existing decision/divergence records
- `docs/hp41-time-divergences.md §D-40-04` — "Interrupting Control Alarm Deferral —
  Re-Entrancy Not Supported"; the existing OM citation (HP 00041-90035 §XYZALM) and the
  current `>>`=interrupting convention. **Will be updated in Phase 66, not Phase 62.**
- `docs/adr/v4.3-001-single-instance-guard.md`, `docs/adr/v4.3-002-macos-global-hotkey.md`
  — existing v4.3 ADRs; the new contract becomes `v4.3-003` beside them.

### Code under audit (read to verify the current flag/parse logic — do NOT modify in Phase 62)
- `hp41-core/src/ops/time/alarm.rs` — `AlarmType::Control { label, interrupting }` (def
  ~L57), `parse_alarm_type` (~L78: `>>`→`interrupting:true`, `>`→`false`),
  `dispatch_alarm_event` (~L493: interrupting→`"alarm:interrupting:deferred"` dead-end,
  non-interrupting→`"alarm:xeq:{label}"`), `check_alarms`, `alarm_matches_alpha`.
- `hp41-core/src/ops/program.rs` — `run_loop` / 4-level call stack / `is_running` /
  `MAX_STEPS` (the engine Phase 63 will modify; context only for Phase 62).

### Primary OM (external — the researcher must obtain a clean scan and cite the page)
- HP-41CX Owner's Manual Vol. 2, §XYZALM / alarm chapter (ManualsLib / hpmuseum.org).
- HP 82182A Time Module Owner's Manual (HP 00041-90035), §XYZALM — the manual D-40-04
  cites. The verification step must determine which is authoritative for the prefix.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `parse_alarm_type` (`alarm.rs`): already parses `>>` before `>` (shadow-safe) and
  stores `AlarmType::Control { label, interrupting }`. The data model is complete; only
  the **flag semantics** are in question. Re-derivation of the display form
  (`format!(">>{label}")` when interrupting) means a flip touches display reconstruction too.
- `AlarmType` round-trips through serde (alarm catalog persisted in `CalcState`, D-38.8) —
  this is why the correction plan (D-05) must address save-file back-compat.

### Established Patterns
- The contract/ADR pattern: `docs/adr/` holds 23+ ADRs (`v4.3-001`/`002` are the latest).
  The new contract follows that format.
- "Document the divergence" pattern: `docs/hp41-time-divergences.md` (D-NN entries) is the
  canonical place divergences are reflected — updated at milestone closeout (Phase 66).

### Integration Points
- None for Phase 62 (docs only). The contract's *consumers* are Phase 63 (alarm execution)
  and Phase 66 (D-40-04 update + verification gate).

</code_context>

<specifics>
## Specific Ideas

- The contract must be unambiguous enough that an implementer reading **only the ADR**
  cannot reproduce the `>`/`>>` confusion. Whatever the OM verdict, the ADR should state
  the mapping in terms of **observable behavior** ("the alarm that interrupts a running
  program is `>X`") rather than relying on the ambiguous `interrupting: bool` field name.
- The mechanism is prefix-agnostic: per ARCHITECTURE.md/SUMMARY.md, the synchronous
  interrupt design is correct regardless of which prefix wins — **only the routing
  condition flips**. The contract should make this explicit so Phase 63 is unblocked the
  moment the prefix is settled.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. (The actual alarm-execution mechanism,
`run_loop` yield, `pending_interrupt` field, and the D-40-04 doc update are not deferred
*ideas* — they are already scoped into Phase 63 and Phase 66 respectively.)

</deferred>

---

*Phase: 62-alarm-semantics-spec*
*Context gathered: 2026-06-06*
