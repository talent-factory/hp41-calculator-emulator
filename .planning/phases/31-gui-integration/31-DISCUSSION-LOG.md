# Phase 31: GUI Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 31-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-17
**Phase:** 31-gui-integration
**Areas discussed:** Cancellation UX, Modal-prompt placement, ?-overlay sectioning, CATALOG 2 rendering

---

## Cancellation UX

### Q1: How should the user trigger cancellation of a long-running INTG/SOLVE/DIFEQ in the GUI?

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated cancel button | 'Cancel' button overlay rendered conditionally on `is_running`. Clear affordance, no key collision. | |
| R/S 3-way state routing | Extend R/S key: `modal_program.is_some()` → submit_modal; `is_running` → request_cancel; else → run_stop. Hardware-faithful. | ✓ (combined) |
| Esc universal cancel | Esc cancels modal AND long-compute AND shiftActive in priority order. Single mental model. | ✓ (combined) |
| Both R/S + cancel button | R/S for keyboard users + visible cancel button as discoverable mouse fallback. | |

**User's choice:** "Können wir `R/S 3-way state routing` und `Esc as universal cancel` kombinieren?"
**Notes:** Both wired in parallel. R/S preserves hardware fidelity for keyboard users; Esc gives modern GUI users the "back out" mental model. Both call the same `request_cancel` Tauri command during `is_running`. No dedicated cancel button — explicit rejection of the with-button options. D-25.6 parity flag captured: CLI Phase 29 has no `request_cancel` route yet — backport to CLI is a deferred quick-task.

### Q2: When cancellation fires (HpError::Canceled returned), what does the user see?

| Option | Description | Selected |
|--------|-------------|----------|
| Existing toast only | HpError::Canceled flows through v2.1 From<HpError> for GuiError; 2s toast renders 'CANCELED'. Zero new render code. | ✓ |
| LCD message + toast | Display14Seg renders 'CANCELED' for ~1s plus toast. More prominent, new render path. | |
| Silent return | No toast, no LCD; op returns Ok with X holding partial result. Risk: user wonders if cancel worked. | |
| Inline status banner | Reuse modal-prompt banner for 'CANCELED' display. Couples to banner-placement choice. | |

**User's choice:** Existing toast only.
**Notes:** Minimum new surface; matches v2.1 stub-error pattern; HpError::Canceled `Display` impl returns "CANCELED" — toast inherits automatically.

### Q3: While INTG/SOLVE/DIFEQ is computing, how does the user know R/S or Esc will cancel?

| Option | Description | Selected |
|--------|-------------|----------|
| is_running annunciator only | Existing v2.1 annunciator lights during long compute, like real HP-41 BUSY annunciator. No new UI. | ✓ |
| is_running + inline hint | 'press Esc to cancel' text near LCD when is_running. More discoverable, diverges from HP-41. | |
| is_running + tooltip on R/S | Hover R/S during is_running shows 'cancel' tooltip. Mouse-only. | |

**User's choice:** is_running annunciator only.
**Notes:** Hardware-faithful (real HP-41 had only the BUSY annunciator); minimum new surface; users learn R/S/Esc behavior from docs.

---

## Modal-prompt placement

### Q1 (initial): Where should the modal prompt text appear in the GUI?

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated banner above LCD | New DOM row renders modal_prompt; cleanest separation. | (passed over) |
| Print panel (ROADMAP suggestion) | Append prompts into existing scrollable widget. | (passed over) |
| Inline above print panel | Thin prompt strip between stack panel and print panel. | (passed over) |
| Persistent toast (top-right) | Reuse v2.1 toast machinery as persistent (no auto-dismiss) corner toast. | (passed over) |

**User's choice:** None — user asked "Was ist dein Vorschlag, damit es so nah wie nur möglich dem realen HP-41CV entspricht?"
**Notes:** Original four options all diverged from real HP-41CV behavior (which has no second display — single 12-char LCD alternates between prompt and typed value). I proposed a new option centered on LCD-alternation.

### Q1 (revised): Render the modal prompt directly on the LCD when entry_buf is empty?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — LCD alternation + truncate | LCD shows modal_prompt when entry_buf empty; switches to entry_buf on type; switches to next prompt after R/S. Long prompts truncate with HP-41 `≡` continuation marker. | ✓ |
| Yes — LCD alternation + scroll | Same but AVIEW-scroll long prompts. Higher cost; defer scrolling to v3.1 polish. | |
| Yes — LCD alternation + wider LCD | Allow Display14Seg up to 16 chars when modal_prompt active. Spirit, not letter, of fidelity. | |
| Keep original dedicated banner | Fall back to first-question's banner option. Less hardware-faithful. | |

**User's choice:** LCD alternation + truncate. Selected preview text included "prompt fully visible in PRINT panel for accessibility" suggestion, but Q2 below clarified this.
**Notes:** Maximum HP-41CV fidelity. D-29.4 ("LCD shows entry_buf live during entry") preserved — that speaks to during entry; before entry begins the LCD is free.

### Q2: How should the full prompt text appear in the print panel (accessibility fallback)?

| Option | Description | Selected |
|--------|-------------|----------|
| Pinned header line above PRX history | Print panel renders 'Prompt: FUNCTION NAME?' header above scrollable PRX history. | |
| Push into print_buffer on each prompt change | Math Pac I ops push prompt strings into print_buffer alongside PRX. Conflicts with D-28.4 spirit. | |
| Tooltip on the LCD on hover | Truncated LCD shows ≡; hover reveals full prompt. Mouse-only. | |
| Skip the accessibility fallback | LCD truncation is sufficient. Most hardware-faithful (real HP-41CV has no fallback either). | ✓ |

**User's choice:** Skip the accessibility fallback.
**Notes:** Pure hardware fidelity preferred. D-28.4 (print_buffer reserved for PRX/PRA/PRSTK) trivially preserved.

---

## ?-overlay sectioning

### Q1: How should the ?-overlay present the 55 Math Pac I entries alongside the 130 HP-41CV entries?

| Option | Description | Selected |
|--------|-------------|----------|
| Two top-level sections | Collapsible 'HP-41CV (built-in)' + 'Math 1 Pac (XROM 7)', both expanded; JSON categories as 2nd-level headers. | ✓ |
| Tabs at top of overlay | Tab bar HP-41CV / Math 1 Pac / All. User clicks to switch view. | |
| Single combined list | All 185 entries flat-grouped by JSON category. Loses XROM identity. | |
| Two sections + filter input | Same as two sections plus text filter at top. Excellent for discovery, adds search-input widget. | |

**User's choice:** Two top-level sections (selected with preview).
**Notes:** Clear module identity; anticipates v3.1+ pac additions as new top-level sections without touching existing two. Both sections expanded by default. JSON's per-program categories preserved as 2nd-level headers.

---

## CATALOG 2 rendering

### Q1: How should CATALOG 2 render in the GUI?

| Option | Description | Selected |
|--------|-------------|----------|
| Hardware-faithful: PSE scroll via print_buffer | Mirror v2.2 CAT 1 pattern. Push each XROM module's function names into print_buffer with ~500ms PSE delay; R/S pauses, other keys cancel. | ✓ |
| Scrollable modal overlay | Open modal listing all loaded XROM modules + functions. Faster discovery but breaks HP-41 fidelity. | |
| Pipe to ?-overlay | CATALOG 2 opens ?-overlay with 'Math 1 Pac' section pre-focused. Reuses overlay, furthest from CAT 2 UX. | |
| Hardware-faithful + summary header | Same as PSE scroll but verbose first-line ('XROM 7 MATH 1 (55 fns)') header. | |

**User's choice:** Hardware-faithful PSE scroll (selected with preview showing "XROM Math 1" header + per-function lines).
**Notes:** Maximum HP-41 fidelity; reuses v2.2 Phase 22 CAT 1 infrastructure (print_buffer + PSE-step). First line is the simple "XROM Math 1" module header (no verbose summary). Research-prep flag: planner must verify whether Phase 28 fully implemented `Op::Catalog(2)` body or shipped a stub; surgical hp41-core exception may be needed if the latter.

---

## Claude's Discretion

- **Implementation site for the LCD-alternation routing (D-31.5):** `commands.rs::handle_get_state` (GUI-only, recommended) vs `hp41-core::state::display()` (CLI + GUI both inherit, follow-up backport) vs `Display14Seg.tsx` (least DRY). Recommendation: option 1 for Phase 31; option 2 as a quick-task backport.
- **`Op::Catalog(2)` body verification — full vs stub in Phase 28.** Research-prep determines; if stub, Phase 31 takes a surgical hp41-core exception (analogous to v2.2 Phase 25-03's `builtin_card_op` 4→12 extension).
- **Vite path-alias vs symlink** for cross-directory JSON imports (`docs/` outside `hp41-gui/src/`).
- **`request_cancel` Tauri command signature** — likely `fn request_cancel(state: State<'_, AppState>) -> Result<(), GuiError>`; idempotent; no-op when `is_running == false`.
- **`PendingInput::XeqByName { acc, mode }` TypeScript port shape** — Rust enum → TS string-literal union `'normal' | 'collect-for-modal'`; planner picks exact serialization matching IPC.
- **HP-41 `≡` continuation marker rendering** — verify U+2261 glyph in Display14Seg's SVG 14-segment font (SKIN-04); fallback to ASCII if absent.
- **PSE delay for CAT 2** — match v2.2 CAT 1 exactly; tunable in future polish.
- **`?`-overlay default expand/collapse persistence** — reset on each open (no localStorage) recommended.
- **Sort order within categories** — alphabetical within each category for consistency (vs OM-manual order).
- **ALPHA annunciator behavior during modal alpha-label collection** — mirror v2.2 XEQ-by-name modal behavior; if v2.2 doesn't light ALPHA during name entry, Phase 31 doesn't either.
- **Precedence when `pending_input.is_some() && modal_program.is_some()` simultaneously** — modal_program wins; planner adds regression test.

## Deferred Ideas

- **CLI Esc/Ctrl-C → `request_cancel` backport** — D-25.6 parity quick-task after Phase 31 ships the shared `request_cancel` surface.
- **Backport LCD-alternation routing into `hp41-core::state::display()`** — quick-task to deprecate CLI's Phase 29 D-29.3 status-bar in favor of direct LCD prompt rendering.
- **AVIEW-style scrolling for prompts > 12 chars on the GUI LCD** — v3.1 polish replaces truncation if user feedback demands it.
- **Filter input on `?`-overlay** — Pitfall 13 mitigation when v3.1+ pacs push entry count past discoverability threshold.
- **Tab UI for `?`-overlay** — structural fix when v3.1+ adds enough pacs to make sections unwieldy.
- **CAT 2 module-header verbosity** ("XROM 7 MATH 1 (55 fns)") — cosmetic polish if user research shows it helps.
- **Tauri command `submit_modal_with_label`** for post-dispatch auto-open in `CollectForModal` mode (Phase 29 D-29.7 `advance_with_label` path).

---

*End of discussion log — see 31-CONTEXT.md for the canonical decision record.*
