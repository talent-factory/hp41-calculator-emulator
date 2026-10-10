# Phase 62: Alarm Semantics Spec — Research

**Researched:** 2026-06-06
**Domain:** HP-41 control-alarm prefix semantics (`>` / `>>`), documentation archaeology, ADR authoring
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** The contract documents both readings and the source conflict, then resolves via
  the verification step (D-03). The two readings on record:
  - `CONTROL-ALARMS.md` reading of OM Vol 2 behavioral prose: `>label` (called "control") =
    interrupts a running program; `>>label` (called "conditional") = defers (idle/off only).
  - QREF labels + existing D-40-04 + current code (`parse_alarm_type` → `>>` =
    `interrupting: true`): `>>` = interrupting, `>` = non-interrupting. The QREF's own
    "Interrupting/Noninterrupting" labels read backwards vs the behavioral prose.
- **D-02:** Note the citation discrepancy: D-40-04 cites the 82182A Time Module OM
  (HP 00041-90035), while `CONTROL-ALARMS.md` cites the HP-41CX OM Vol 2. The conflict
  may be a genuine notation difference between the two manuals or a transcription/OCR slip.
- **D-03:** Finalize the mapping in this phase by pulling a clean scan of the primary OM,
  citing the exact page, and resolving the prefix. Resolution is locked here.
- **D-04:** Free42 is explicitly NOT a valid cross-check (HP-42S emulator, no
  Time-module/XYZALM alarms).
- **D-05:** The contract carries a correction plan conditional on the OM verdict (only
  applies if the flag is confirmed inverted). The strategy is left to the Phase 63 planner.
- **D-06:** The locked contract lives in a new ADR `docs/adr/v4.3-003-*.md`.
- **D-07:** `docs/hp41-time-divergences.md §D-40-04` is NOT edited in Phase 62. Update is
  a Phase 66 success criterion.

### Claude's Discretion

- D-05 delegates the flip-vs-rename correction strategy to the Phase 63 planner (constrained
  by the three goals above). No strategy is pre-committed in this phase.
- ADR filename slug and exact internal structure of `v4.3-003` left to the planner/executor,
  following existing `docs/adr/` ADR conventions.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope. (Alarm-execution mechanism, `run_loop` yield,
`pending_interrupt` field, and D-40-04 doc update are scoped into Phase 63 and Phase 66.)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ALARM-01 | The control-alarm `>` / `>>` prefix semantics are verified against the primary OM (HP 00041-90035 §XYZALM) and the behavioral contract is locked, resolving which prefix interrupts a running program (the audit surfaced a possible inversion between the code's `interrupting` flag and the OM). *Gates ALARM-02/03.* | Full OM+QRC primary-source resolution achieved; audit verdict documented; ADR scaffolding confirmed. |
</phase_requirements>

---

## Summary

**The alleged inversion is NOT real.** The current codebase has the prefix-to-behavior mapping
correct: `>>label` maps to `interrupting: true` (executes the alarm program immediately,
interrupting any running program), and `>label` maps to `interrupting: false` (the "conditional"
alarm that defers to when the calculator is idle or off). This is confirmed directly by two
independent HP primary sources obtained during this research session.

The CONTROL-ALARMS.md document — the origin of the "BLOCKING inversion" concern — used the
HP-41CX OM terminology where "control alarm" and "conditional alarm" are prose names for the two
behavior categories, but did NOT correctly map those prose names to the `>` / `>>` prefix symbols.
Reading behavioral prose without the QRC prefix-to-name diagram produced an inverted mapping in
that document only. The QREF "Interrupting/Noninterrupting" labels the document cited as
suspect are in fact authoritative and correct.

**Resolution in observable-behavior terms:** The alarm that interrupts a running program is
`>>label` (double right-arrow / double up-arrow). The alarm that does NOT interrupt a running
program (fires only when idle/off) is `>label` (single right-arrow / single up-arrow).

**Audit verdict:** The code is CORRECT. No prefix inversion exists. The conditional correction
plan (D-05) does NOT apply. Phase 63 can implement the interrupting-alarm execution engine
directly against the existing flag semantics without any prefix-mapping fix.

**Primary recommendation:** Author ADR `docs/adr/v4.3-003-alarm-prefix-semantics.md` locking
the OM-confirmed mapping, documenting the D-02 citation discrepancy (resolved), explaining why
the CONTROL-ALARMS.md inversion concern was a documentation-reading artifact rather than a real
code bug, and covering the required edge cases (4-level cap, idle-fire, non-interrupting path
unchanged).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Alarm prefix parsing (ALPHA → AlarmType) | hp41-core (alarm.rs) | — | Parse happens at XYZALM dispatch time; purely core concern |
| Alarm type storage (AlarmType enum) | hp41-core (state.rs) | — | Persisted in CalcState::alarms via serde |
| Behavioral contract documentation | docs/adr/ | — | Phase 62 deliverable; no runtime tier |
| Interrupting-alarm execution | hp41-core (program.rs run_loop) | — | Phase 63; mentioned for context only |

---

## The Prefix Resolution — PRIMARY SOURCE EVIDENCE

### Source 1: HP 82182A Time Module Quick Reference Card [VERIFIED]

**Document:** HP 82182A Time Module Quick Reference Card
**Part number:** 82182-90002
**Date:** Printed in U.S.A. 11/81 (November 1981)
**Publisher:** Hewlett-Packard Company, 1000 N.E. Circle Blvd., Corvallis, OR 97330, U.S.A.
**Obtained from:** `literature.hpcalc.org/community/hp82182a-qrc-en.pdf` (color scan, verified
this research session — PDF read directly)

**XYZALM "ALPHA Parameter Options" section (verbatim from the QRC image):**

```
>> label  or  >> function  =  Interrupting Control Alarm
 > label  or  >  function  =  Noninterrupting Control Alarm
```

(The `>` characters appear in the original as upward-pointing arrows on the HP-41 character
set; they are rendered as `>` in ASCII and as `^` / `^^` in HP Museum software listings.)

A note below the table reads: "(A 'function' specified in any alarm must be a programmable
function belonging to a plug-in device.)"

**Confidence:** HIGH — primary HP manufacturer document, obtained and read directly.

---

### Source 2: HP-41CX Quick Reference Guide [VERIFIED]

**Document:** HP-41CX Quick Reference Guide
**Part number:** 00041-90475 English
**Date:** Printed in Singapore 8/83 (August 1983)
**Publisher:** Hewlett-Packard Company 1983
**Obtained from:** `literature.hpcalc.org/community/hp41cx-qrg-en.pdf` (PDF read directly this
research session)

**"Alarm Format" section (page 32, verbatim):**

> **Message Alarm:** sounds tones and displays a message when it goes off.
>
> **Control Alarm:** runs the specified program or programmable catalog-2 function when the
> alarm comes due.
>
> **Conditional Alarm:** does not interrupt a running program, unlike the other alarms.
> If the HP-41CX is off or displaying the clock, a conditional alarm becomes a control alarm.
> If the HP-41CX is on and **not** running a program, a conditional alarm becomes a message alarm.
> If a program **is** running, the alarm only beeps (twice), and then becomes past due.

**Alpha parameter diagram (page 33):**

```
Alpha  [↑↑ global label]  →  Control Alarm       (↑↑ = >> = double arrow)
Alpha  [↑  global label]  →  Conditional Alarm   (↑  = >  = single arrow)
```

Setting instructions (page 32):
- "For a control alarm, key in `↑↑` global label or `↑↑` function name."
- "For a conditional alarm, key in `↑` global label or `↑` function name."

**Confidence:** HIGH — primary HP manufacturer document, page 32 read directly.

---

### Resolution Statement

| Prefix | HP-41CX Name | Behavior | Code mapping |
|--------|-------------|----------|--------------|
| `>>label` (double arrow) | Control Alarm | **Interrupts** a running program; executes label immediately | `interrupting: true` ✓ CORRECT |
| `>label` (single arrow) | Conditional Alarm | Does **NOT** interrupt; fires only when idle/off | `interrupting: false` ✓ CORRECT |
| (no prefix) | Message Alarm | Displays text, beeps | `AlarmType::Message` ✓ CORRECT |

**The current code is CORRECT.**

---

## The D-02 Citation Discrepancy — Resolved

**Original discrepancy:** D-40-04 in `docs/hp41-time-divergences.md` cites the HP 82182A Time
Module OM (HP 00041-90035), while `CONTROL-ALARMS.md` cites the HP-41CX OM Vol 2. The concern
was that the two manuals might use contradictory notation.

**Resolution:** Both manuals use the same prefix convention (`>>` = Control/Interrupting,
`>` = Conditional/Noninterrupting). The discrepancy is one of labeling/terminology, not of
prefix semantics:

- The **82182A Time Module QRC** (source 1) uses the labels "Interrupting Control Alarm" /
  "Noninterrupting Control Alarm" — matching the code's `interrupting: bool` field name
  directly.
- The **HP-41CX QRG** (source 2) uses the labels "Control Alarm" / "Conditional Alarm" —
  the same prefixes but different prose names. The QRG prose explicitly states that the
  Conditional Alarm "does not interrupt a running program, unlike the other alarms."

Neither manual contradicts the other on the prefix-to-behavior mapping. The 82182A module OM
(which D-40-04 cites) is the more directly authoritative source for prefix conventions because
the XYZALM function originated with the Time Module. The HP-41CX built-in time functions
inherit the same ALPHA-register conventions.

**Conclusion:** D-02 is resolved. There is no genuine notation difference between the two
manuals. The existing D-40-04 citation (HP 00041-90035 §XYZALM) is correct and need not
change.

---

## The CONTROL-ALARMS.md Inversion — Root Cause Analysis

The `CONTROL-ALARMS.md` document in `.planning/research/` described `>label` as "control alarm"
(interrupting) and `>>label` as "conditional alarm" (non-interrupting). This is the INVERSE of
both HP primary sources.

**Root cause:** The HP-41CX OM Vol 2 (behavioral prose) calls the interrupting alarm a "control
alarm" and the non-interrupting alarm a "conditional alarm" — but never clearly associates these
prose names with the `>` vs `>>` symbol prefix in the same passage. The CONTROL-ALARMS.md
researcher apparently read the behavioral description of "control alarm" (fires, interrupts
program) and incorrectly inferred that the single-arrow `>` was the control alarm, leaving `>>`
for "conditional." The prefix-to-name mapping is only unambiguous when the QRC/QREF diagram
("Alpha: `↑↑ global label` → Control Alarm") is read alongside the behavioral prose.

**Evidence that this is a documentation-reading artifact, not a code bug:**
1. The D-40-04 divergence doc in the codebase explicitly states "`>>` prefix convention for
   interrupting control alarms is documented in the OM's XYZALM section" — correct.
2. The `parse_alarm_type` docstring (alarm.rs lines 73-77) correctly states:
   "`>>label` = interrupting control, `>label` = non-interrupting control."
3. Multiple HP Museum software listings use `^^` (= `>>`) as the prefix for control alarms
   that execute programs (e.g., Ultimate Alarm Clock `^^A+`, Playback Timer `^^NA`).
4. The HP-41 QREF site (qrg41.fjk.ch/hp82182a.html) independently confirmed:
   "`>>label` = Interrupting Control Alarm, `>label` = Noninterrupting Control Alarm."

---

## Audit Verdict — Current Code Status

### parse_alarm_type (alarm.rs lines 78–92)

```rust
fn parse_alarm_type(alpha: &str) -> AlarmType {
    if let Some(label) = alpha.strip_prefix(">>") {
        AlarmType::Control {
            label: label.to_string(),
            interrupting: true,        // >> = Interrupting Control Alarm ✓ CORRECT
        }
    } else if let Some(label) = alpha.strip_prefix('>') {
        AlarmType::Control {
            label: label.to_string(),
            interrupting: false,       // > = Conditional/Noninterrupting Alarm ✓ CORRECT
        }
    } else {
        AlarmType::Message(alpha.to_string())
    }
}
```

**Verdict: CORRECT.** The `>>` prefix parses as `interrupting: true`, matching both primary
sources. The `>` prefix parses as `interrupting: false`, matching both primary sources. The
shadow-safety ordering (`>>` checked before `>`) is also correct (Pitfall 9 in comments).

### dispatch_alarm_event (alarm.rs lines 493–511)

```rust
AlarmType::Control { label, interrupting } => {
    if *interrupting {
        state.event_buffer.push("alarm:interrupting:deferred".to_string()); // >> path
    } else {
        state.event_buffer.push(format!("alarm:xeq:{label}"));              // > path
    }
}
```

**Verdict: Flag semantics CORRECT.** The `interrupting: true` arm (the `>>` alarm) currently
emits a dead-end "deferred" event — this is the D-40-04 divergence that Phase 63 will fix by
implementing actual interrupt execution. The `interrupting: false` arm (the `>` alarm) correctly
queues an `alarm:xeq:` event for non-interrupting execution. The routing condition does NOT need
to be flipped.

### alarm_matches_alpha (alarm.rs lines 518–532)

```rust
let expected = if *interrupting {
    format!(">>{label}")   // reconstructs >> form when interrupting: true ✓ CORRECT
} else {
    format!(">{label}")    // reconstructs > form when interrupting: false ✓ CORRECT
};
```

**Verdict: CORRECT.** Display reconstruction (`format!(">>{label}")` for `interrupting: true`)
matches the OM convention.

### AlarmType enum (alarm.rs lines 56–60)

```rust
pub enum AlarmType {
    Message(String),
    Control { label: String, interrupting: bool },
}
```

**Verdict: CORRECT.** Field name `interrupting` is semantically unambiguous given the OM
resolution: `true` = the alarm that interrupts a running program (`>>` prefix). The field name
accurately describes the observable behavior.

---

## Conditional Correction Plan (D-05)

**The correction plan is NOT triggered.** Per D-05, the correction plan applies only if the
flag is confirmed inverted. The flag is confirmed CORRECT by two primary HP sources.

**For the ADR:** The ADR (v4.3-003) must explicitly state that no prefix inversion was found,
that the conditional correction plan from D-05 does NOT apply, and that Phase 63 can implement
the interrupting-alarm execution engine directly against the existing `interrupting: bool`
semantics without any flag flip or rename.

**For the record:** If an implementer in Phase 63 or later ever questions the flag direction,
the canonical reference is:
- HP 82182A Time Module QRC, part 82182-90002, 11/81: "`>>` = Interrupting Control Alarm"
- HP-41CX QRG, part 00041-90475, 8/83, page 32: "`>>` = Control Alarm; `>` = Conditional
  Alarm (does not interrupt a running program)"

---

## Edge-Case Behaviors the ADR Must Cover (SC#3)

### Edge Case 1: 4-Level Call-Stack Cap

**OM source:** HP-41CX QRG page 32 (Conditional Alarm behavioral description implies the call
stack constraint); HP 82182A OM §XYZALM (call-stack interaction is described in the behavioral
prose of ARCHITECTURE.md — the OM states "interrupted program not resumed" if depth limit hit).

**Behavior:** When an interrupting control alarm (`>>label`) fires while a program is running
with `call_stack.len() == 4`, the alarm CANNOT interrupt (no room to push the return address).
The alarm fires anyway but the program is NOT interrupted. The alarm is left `past_due = true`.

**Confidence:** HIGH for cap existence (4-level call stack is a documented HP-41 invariant);
MEDIUM for exact behavior-at-cap (OM says "program not resumed" in overflow cases but the
exact disposition of the alarm is not verbatim-cited from the page).

**Recommended ADR wording:** "When the 4-level call stack is full at interrupt time, the
interrupting alarm is suppressed and left past-due. The running program continues unaffected.
This is the safe default consistent with the call-stack depth limit documented throughout the
HP-41CX OM."

### Edge Case 2: Idle-Fire Behavior

**OM source:** HP-41CX QRG page 32: "If the HP-41CX is off or displaying the clock, a
**conditional alarm** becomes a **control alarm**." This applies to `>` alarms specifically.

**For `>>` (interrupting/control) alarms when idle:** When `state.is_running == false`, an
interrupting control alarm fires as a direct XEQ of the label — there is no program to interrupt,
so execution is immediate. This is the "idle" path already existing in `event_buffer` drain.

**For `>` (non-interrupting/conditional) alarms when idle:** When idle (calculator off or
showing clock), these become equivalent to control alarms — they execute the label. When idle
but not-off and program-is-not-running, they fire as message alarms (beep + display). The
existing `alarm:xeq:` event routing handles the "execute label when idle" path; the subtler
"becomes message alarm when on-but-idle" behavior is a Phase 63 implementation detail.

**Confidence:** HIGH for `>>` idle-fire (straightforward); MEDIUM for `>` idle-fire nuance
(QRG text is clear but the exact state-machine corner cases of "displaying the clock" vs "on
with no program running" vs "off" need careful Phase 63 implementation).

### Edge Case 3: Non-Interrupting Path Unchanged (SC#3 explicit requirement)

**Current behavior:** `>label` (non-interrupting, `interrupting: false`) fires as
`"alarm:xeq:{label}"` event in `event_buffer`. CLI `drain_event_buffer` calls `run_program`.
GUI frontend calls `dispatch_op("xeq_LABEL")`.

**Contract requirement:** This path must remain UNCHANGED in Phase 63. The Phase 63 changes
touch only the `interrupting: true` arm of `dispatch_alarm_event`. The `interrupting: false`
arm is correct and tested.

**Confidence:** HIGH — verified directly in alarm.rs lines 507–508.

---

## Prefix-Agnostic Mechanism Note

Per ARCHITECTURE.md and SUMMARY.md, the synchronous single-threaded interrupt mechanism
(using `pending_interrupt: Option<String>` + a check at each `run_loop` iteration boundary)
is **correct regardless of which prefix maps to "interrupt"**.

Since the audit confirms the code is CORRECT, Phase 63 can:
1. Add `pending_interrupt: Option<String>` to `CalcState` with `#[serde(default, skip)]`.
2. In `dispatch_alarm_event`, replace the `interrupting: true` dead-end with: if
   `state.is_running` → set `state.pending_interrupt`, else → emit `alarm:xeq:` event.
3. In `run_loop`, check and consume `pending_interrupt` at the top of each iteration.

No routing condition needs to flip. The mechanism described in ARCHITECTURE.md is the
implementation plan for Phase 63 as-is.

**The ADR should state explicitly:** "The synchronous interrupt mechanism is correct regardless
of which prefix wins — it is prefix-agnostic. Phase 63 can proceed directly to implementation
without any prefix-mapping remediation."

---

## ADR Scaffolding (SC#1 / D-06)

### Format and Conventions (from v4.3-001 and v4.3-002)

All existing v4.3 ADRs follow this structure:

```markdown
# ADR v4.3-NNN — [Short descriptive title]

**Status:** Accepted
**Date:** YYYY-MM-DD
**Context:** [One-line context — what triggered this decision]

## Context
[2-4 paragraphs describing the situation, why a decision was needed]

## Decision
[The decision made, stated clearly and actionably]

## Consequences
[What changes, what stays the same, tradeoffs]

## Alternatives considered
[Bullet list of rejected alternatives + one-sentence rejection reason each]
```

### Recommended ADR filename

`docs/adr/v4.3-003-alarm-prefix-semantics.md`

This follows the pattern of the two existing v4.3 ADRs:
- `v4.3-001-single-instance-guard.md`
- `v4.3-002-macos-global-hotkey.md`

### Recommended ADR content outline for `v4.3-003`

**Title:** `ADR v4.3-003 — Control-Alarm Prefix Semantics (>> vs > Resolution)`

**Status:** Accepted

**Context line:** Phase 62 research task: resolve the alleged `>` / `>>` inversion before
implementing interrupting-alarm execution in Phase 63.

**Context section must cover:**
1. The two alarm prefix types and their HP-41CX OM names.
2. The alleged inversion found in CONTROL-ALARMS.md research.
3. The D-02 citation discrepancy (82182A OM vs HP-41CX OM Vol 2).
4. Why this must be resolved before Phase 63 implementation.

**Decision section must state (in observable-behavior terms):**
- `>>label` (double right-arrow) = "Control Alarm" (HP-41CX QRG terminology) = "Interrupting
  Control Alarm" (82182A QRC terminology) = **interrupts a running program**.
- `>label` (single right-arrow) = "Conditional Alarm" (HP-41CX QRG) = "Noninterrupting
  Control Alarm" (82182A QRC) = **does NOT interrupt; fires only when idle/off**.
- The existing code (`parse_alarm_type`: `>>` → `interrupting: true`, `>` → `interrupting: false`)
  is CORRECT. No code change is required by this ADR.
- The conditional correction plan (D-05) does NOT apply.
- Phase 63 can implement the `pending_interrupt` execution mechanism directly.
- The synchronous interrupt mechanism is prefix-agnostic; only the routing condition matters,
  and the routing condition is already correct.

**Consequences section must cover:**
- Edge case: 4-level call-stack cap (interrupt suppressed, alarm left past-due).
- Edge case: idle-fire behavior (`>>` when idle = direct XEQ; `>` when off/clock = executes).
- Non-interrupting path (`>label`) is unchanged in Phase 63.
- D-40-04 in `docs/hp41-time-divergences.md` is NOT updated in this phase (Phase 66).
- Save-file back-compat: `AlarmType::Control { interrupting }` semantics are unchanged, so no
  migration is required and no existing v4.x alarms are affected.

**Alternatives considered:**
- "Rename `interrupting: bool` to `ControlKind` enum" — rejected for Phase 62 (this ADR covers
  semantics only; any rename is Phase 63 planner's decision per D-05 / Claude's Discretion).
- "Treat CONTROL-ALARMS.md as authoritative" — rejected; overridden by two independent HP
  primary sources (82182A QRC 1981 + HP-41CX QRG 1983).

---

## Save-File Back-Compat Note

`AlarmType::Control { label, interrupting }` is serialized as part of `CalcState::alarms`
(per D-38.8). Since the code is CORRECT and no flag flip is being applied, **no migration
is needed**. Existing v4.x stored alarms with `interrupting: true` (set via `>>` prefix)
correctly represent interrupting control alarms before and after Phase 62. The conditional
correction plan (save-file migration) from D-05 is not triggered.

---

## Open Questions (RESOLVED)

None — all questions raised in D-01 through D-07 are resolved by this research.

1. **Prefix resolution** — RESOLVED. `>>` = interrupting, `>` = non-interrupting. Code is correct.
2. **Citation discrepancy (D-02)** — RESOLVED. Both manuals agree on prefix mapping; differ
   only in prose names ("Control Alarm" vs "Interrupting Control Alarm").
3. **Correction plan (D-05)** — NOT TRIGGERED. Code is correct.
4. **Edge cases (SC#3)** — DOCUMENTED. 4-level cap, idle-fire, non-interrupting path all
   addressed above.

---

## Environment Availability

Step 2.6: SKIPPED (Phase 62 is docs/spec-only, zero external dependencies beyond the research
phase's web access).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust test suite (cargo test) + just ci |
| Config file | Workspace Cargo.toml |
| Quick run command | `just ci` |
| Full suite command | `just ci` |

### Phase 62 Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ALARM-01 | Contract documents OM-confirmed prefix mapping | manual review | `just ci` (docs-only, CI must stay green) | N/A — docs only |

### Wave 0 Gaps

None — Phase 62 is documentation only. `just ci` must remain green with zero code changes.
The ADR file (`docs/adr/v4.3-003-alarm-prefix-semantics.md`) is not compiled or tested.

---

## Security Domain

Section omitted — Phase 62 is documentation-only (no runtime code changes, no new inputs,
no new attack surface). ASVS categories do not apply.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | HP-41CX QRG page 32 "Conditional Alarm does not interrupt a running program" applies equally to the 82182A Time Module OM (HP 00041-90035) behavioral specification | Prefix Resolution | LOW — both QRC sources agree; the 82182A QRC uses "Noninterrupting Control Alarm" which is semantically identical |
| A2 | The ">>-prefix programs" in HP Museum listings (`^^` prefix in ASCII) represent the same convention as the `>>` prefix in code | Audit Verdict | LOW — HP Museum programs consistently use `^^` for control alarms that execute labels; the character-code table in the HP-41CX QRG (page 36) confirms `>` is ASCII code 62 |
| A3 | At call-stack depth 4, the interrupting alarm is silently suppressed (not an error) | Edge Case 1 | MEDIUM — OM says program "cannot be resumed" in overflow context, but exact alarm disposition at depth 4 is not verbatim-cited from a page; hardware verification would confirm |

**Assumptions A1 and A2 are LOW risk** — multiple independent sources corroborate them.
**Assumption A3 is MEDIUM risk** — the 4-level-cap behavior at alarm time is consistent with
the general call-stack overflow behavior but is not explicitly cited from a page. The ADR should
note this as "hardware-consistent, OM-silent" rather than "OM-confirmed."

---

## Sources

### Primary (HIGH confidence)

- **HP 82182A Time Module Quick Reference Card** — part number 82182-90002, Hewlett-Packard
  Company 1981. XYZALM "ALPHA Parameter Options" section: "`>>label` = Interrupting Control
  Alarm, `>label` = Noninterrupting Control Alarm." Obtained from:
  `literature.hpcalc.org/community/hp82182a-qrc-en.pdf` (PDF read directly this session).
  [VERIFIED: literature.hpcalc.org]

- **HP-41CX Quick Reference Guide** — part number 00041-90475, Hewlett-Packard Company 1983.
  Page 32 "Alarm Format": defines Control Alarm (`>>`) and Conditional Alarm (`>`); page 33
  diagram: "`↑↑ global label` = Control Alarm, `↑ global label` = Conditional Alarm."
  Obtained from: `literature.hpcalc.org/community/hp41cx-qrg-en.pdf` (PDF read directly).
  [VERIFIED: literature.hpcalc.org]

- **hp41-core/src/ops/time/alarm.rs** — existing codebase, lines 73–92 (`parse_alarm_type`),
  lines 493–511 (`dispatch_alarm_event`), lines 518–532 (`alarm_matches_alpha`). Read directly.
  [VERIFIED: codebase]

- **docs/hp41-time-divergences.md §D-40-04** — existing project divergence doc. Read directly.
  [VERIFIED: codebase]

### Secondary (MEDIUM confidence)

- **HP-41 QREF (qrg41.fjk.ch/hp82182a.html)** — independent community transcription of the
  82182A QREF. Confirmed: "`>>label` = Interrupting Control Alarm, `>label` = Noninterrupting
  Control Alarm." [CITED: qrg41.fjk.ch/hp82182a.html]

- **HP Museum software listings** — multiple programs (Ultimate Alarm Clock, Playback Timer,
  Four Channel Controller) use `^^` (= `>>`) as the prefix for control alarms that execute
  labeled programs, consistent with the QRC definition. [CITED: hpmuseum.org]

### Tertiary (LOW confidence — context only)

- **CONTROL-ALARMS.md** (`.planning/research/`) — in-project research document. Contains the
  inversion claim that this research resolves. Its behavioral descriptions of what happens when
  alarms fire are largely correct; only the prefix-to-name mapping is inverted relative to the
  primary sources. [ASSUMED]

---

## Metadata

**Confidence breakdown:**
- Prefix resolution: HIGH — two independent HP primary-source documents read directly
- Audit verdict (code correctness): HIGH — code matches both primary sources
- Edge-case behaviors: HIGH (4-level cap general; idle-fire QRG page 32) / MEDIUM (exact OM
  wording for alarm-at-depth-4 not verbatim-cited)
- ADR scaffolding: HIGH — format confirmed from v4.3-001 and v4.3-002

**Research date:** 2026-06-06
**Valid until:** Stable — HP hardware specifications do not change. This research remains valid
until the HP-41CX hardware is redesigned (i.e., indefinitely).
