# Phase 64: Interactive GETKEY — Discussion Log

**Date:** 2026-06-06
**Mode:** discuss (default, interactive — 4 areas batched in one turn)

This log is for human reference (audit/retrospective). Downstream agents read CONTEXT.md.

## Gray areas presented (user selected all 4)

1. Wait & no-key behavior
2. Key capture during wait
3. Display while suspended
4. Input mapping & GUI capture

## Q&A

### Area 1 — Wait & no-key behavior
- Options: (a) Wait indefinitely; 0 only on cancel · (b) Wait with timeout → push 0 · (c) Match real HP-41 exactly — research decides
- **Selected:** (c) Match real HP-41 exactly — research decides → **D-01**. Intent locked (faithful interactive wait, FGAP-04); researcher resolves indefinite-vs-timeout and GETKEYX scope against the HP-41 Extended Functions / CX manual.

### Area 2 — Key capture during wait
- Options: (a) All keys → row×col (faithful); cancel via request_cancel · (b) Capture most keys, R/S still stops · (c) All keys captured; only window/app cancel
- **Selected:** "you decide" (Claude's discretion) → **D-02**. Chose faithful all-key capture (R/S = 84, does not stop), consistent with D-01; escape = Phase 63 `request_cancel` → no-key sentinel (0). Rejected R/S-retains-stop (non-faithful).

### Area 3 — Display while suspended
- Options: (a) Keep current display unchanged (faithful) · (b) Subtle prompt indicator
- **Selected:** (a) Keep current display unchanged → **D-03**. Yield channel carries no override text; `display_override` untouched; CLI↔GUI identical.

### Area 4 — Input mapping & GUI capture
- Options: (a) Reuse CLI map; GUI captures on-screen taps + physical keys · (b) GUI on-screen keys only · (c) Let research/planner define coverage
- **Selected:** (a) → **D-04**. CLI reuses existing handle_key row×col map; GUI (which currently never sets `last_key_code`) must capture on-screen taps AND physical keys during the suspend. Full parity.

## Claude's discretion / deferred-to-research captured
- D-05: extend Phase 63 yield channel with an event-driven wait-for-key kind (planner's shape).
- D-06: ALPHA-mode / nested-yield / re-entrancy edges → research.

## Deferred ideas
- GETKEYX timed variant (unless research finds it intrinsic to faithful GETKEY).
- CATALOG interactive scroll (FGAP-06) → v4.4.
- DISP-01 general display_override → v4.4.

## Scope creep
- None raised.
