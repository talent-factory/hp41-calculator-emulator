# Phase 11: Print Emulation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-08
**Phase:** 11-print-emulation
**Areas discussed:** Print output in TUI, Keyboard bindings, PRSTK format

---

## Print output in TUI

| Option | Description | Selected |
|--------|-------------|----------|
| Small static print tape | Add 2–3 line print tape strip in the left panel (replace spacer). Layout change in ui.rs + Vec<String> in App. | |
| Status bar shows last print | After print op, set app.message to formatted output. No layout change. Message clears on next action. | ✓ |
| File-only, message confirms | Print always goes to file (--print-log or default). TUI message bar shows brief confirmation. | |

**User's choice:** Leaned toward file-only (option 3); after clarification discussion agreed on status bar (option 2) as the console-visible mechanism, with file as additive via --print-log.
**Notes:** User initially asked for recommendation. Agreed status bar is cleanest for v1.1 given PRNT-05 (scrollable panel) is explicitly deferred. PRSTK shows summary "PRSTK → 6 lines" in status bar; full content only in file.

---

## Keyboard bindings

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated letters X/A/K | Direct single-key assignments: X=PRX, A=PRA, K=PRSTK. | |
| P-prefix modal: P→X/A/K | Press 'P' to enter PRINT modal, then X/A/S. Mirrors Phase 10 STO modal. | ✓ |
| You decide the keys | Claude picks based on free-key analysis. | |

**User's choice:** Delegated to Claude ("You decide the keys").
**Notes:** Claude analyzed keys.rs and found 'P' (uppercase) is free (lowercase 'p' = PrgmMode). Chose P-modal for consistency with Phase 10 STO 'S'-modal pattern. Inside modal: x/X=PRX, a/A=PRA, s/S=PRSTK. TUI shows "PRNT: _" during modal.

---

## PRSTK format

| Option | Description | Selected |
|--------|-------------|----------|
| Labeled, right-aligned values | Each line: label field (7 chars) + right-aligned value (17 chars) = 24 chars. | ✓ |
| Hardware-faithful, no labels | Just right-aligned values in 24 chars per line. Order implicit. | |
| You decide | Claude picks whatever is most readable. | |

**User's choice:** Delegated to Claude ("You decide").
**Notes:** Claude chose labeled lines for file log readability. Format: `format!("{:<7}{:>17}", label, value)` = 24 chars. Labels: "T:", "Z:", "Y:", "X:", "LASTX:", "ALPHA:". ALPHA uses left-align. Empty ALPHA → "ALPHA:                 " (17 spaces).

---

## Claude's Discretion

- **Keyboard bindings:** User delegated key selection. Claude chose `'P'`-prefix modal mirroring Phase 10 STO pattern.
- **PRSTK format:** User delegated format choice. Claude chose labeled lines (7-char label + 17-char value).
- **File handle lifetime:** Either drain in `handle_key()` after dispatch or in `run()` before draw. Planner decides.
- **File open error handling:** Show error in `app.message`, continue without file logging.

## Deferred Ideas

- **PRNT-05: Scrollable print history TUI panel** — v2+ per REQUIREMENTS.md. Status bar is the v1.1 placeholder.
- **PRNT-06: ADV, PRREG, Flag 26/TRACE** — niche printer ops, deferred to v2+.
- **Default print log path always active** — rejected for v1.1 in favor of explicit `--print-log` opt-in.
