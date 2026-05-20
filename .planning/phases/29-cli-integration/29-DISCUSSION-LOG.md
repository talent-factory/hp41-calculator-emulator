# Phase 29: CLI Integration — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 29-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-17
**Phase:** 29-cli-integration
**Areas discussed:** JSON file authoring timing, Modal-prompt rendering site, R/S → modal-submit routing, FUNCTION NAME? prompt integration

---

## JSON file authoring timing

| Option | Description | Selected |
|--------|-------------|----------|
| Full ~55 entries in Phase 29 | Phase 29 authors the complete JSON; Phase 30 shrinks to matrix regen + divergences + ADRs. | ✓ |
| Minimal stub in Phase 29, full content in Phase 30 | Phase 29 ships ~5–10 placeholder entries; Phase 30 expands. Risks SC-2/SC-4 being not-met. | |
| Reorder: Phase 30 lands before Phase 29 | Swap build sequence so docs ship first. Contradicts locked Phase 28 → 29 → 30 → 31 → 32 order. | |

**User's choice:** Full ~55 entries in Phase 29.
**Notes:** Operationalized as **D-29.1**. Phase 30 / DOC-01 shrinks to matrix regeneration + divergences expansion + 5 ADRs.

### Follow-up: JSON shape integration

| Option | Description | Selected |
|--------|-------------|----------|
| Second OnceLock + merged accessor | Add `help_entries_math1()` + `help_entries_all()`; callers migrate to merged. | ✓ |
| Single OnceLock loading both JSONs | Concatenate at init time. Panic message becomes ambiguous. | |
| Two completely separate accessors | Each caller picks pool. Risk of drift if callers forget to walk both. | |

**User's choice:** Second OnceLock + merged accessor.
**Notes:** Operationalized as **D-29.2**. Per-file hard-build-blocker preserved; narrow-scope `help_entries()` retained for v2.2 smoke tests.

---

## Modal-prompt rendering site

| Option | Description | Selected |
|--------|-------------|----------|
| Extend pending_prompt() to also read modal_prompt | Single status-bar line for ALL prompts. Signature widens. | ✓ |
| New dedicated line above the LCD | Separate render function. More vertical real estate. | |
| Inside the LCD (with truncation) | HP-41 hardware-faithful. `FUNCTION NAME?` (14 chars) overflows. | |

**User's choice:** Extend pending_prompt().
**Notes:** Operationalized as **D-29.3**. Same status-bar channel users already know from v2.2 FlagPrompt / RegisterPrompt / etc.

### Follow-up: LCD policy while modal is open

| Option | Description | Selected |
|--------|-------------|----------|
| Normal X-register / entry_buf | LCD stays live; mirrors v2.2 RegisterPrompt UX. | ✓ |
| Blank LCD while modal is open | Visually emphasizes pause. Diverges from v2.2 UX. | |
| Mirror the status-bar prompt into the LCD | Closer to HP-41 hardware. `FUNCTION NAME?` overflow. | |

**User's choice:** Normal X-register / entry_buf.
**Notes:** Operationalized as **D-29.4**. Lowest implementation risk; no Display14Seg renderer changes.

---

## R/S → modal-submit routing

| Option | Description | Selected |
|--------|-------------|----------|
| Intercept R/S in App::handle_key before Op::Stop | CLI-local routing; new shared `submit_modal` in hp41-core. | ✓ |
| New Op::ModalSubmit variant | 4-place exhaustive match enforces coverage. Widens Op enum for input concern. | |
| Overload Op::Stop with modal-aware behavior | Violates D-22.5 Op::Stop Neutral-no-op invariant. | |

**User's choice:** Intercept R/S in App::handle_key.
**Notes:** Operationalized as **D-29.5**. Single shared `submit_modal(state)` is the CLI/GUI entry point — D-25.6 parity preserved.

### Follow-up: ESC / cancel behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Esc cancels modal cleanly | New `cancel_modal(state)`; clears modal_program + modal_prompt + entry_buf. | ✓ |
| Esc is a no-op; only completing the modal exits | Authentic HP-41 hardware (no Esc). Regression from v2.1 UX. | |
| Backspace cancels (no Esc binding) | Reuses existing binding. Semantically wrong. | |

**User's choice:** Esc cancels modal cleanly.
**Notes:** Operationalized as **D-29.6**. Preserves v2.1 D-07 never-discard invariant; prevents user from being stuck in a 6-step prompt sequence.

---

## FUNCTION NAME? prompt integration

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-open v2.2 XEQ-by-name modal in collect-label mode | Reuses 100% of existing alpha-collection UI. | ✓ |
| New dedicated ALPHA-text sub-modal for Math Pac I prompts | New PendingInput variant. Duplicates alpha-collection. | |
| Manual: user presses ALPHA, types name, R/S commits | Authentic hardware. Discovery problem. | |

**User's choice:** Auto-open v2.2 XEQ-by-name modal.
**Notes:** Operationalized as **D-29.7**. Same physical keystroke sequence users already know.

### Follow-up: How the XEQ-by-name modal knows it's in collect-label mode

| Option | Description | Selected |
|--------|-------------|----------|
| Extend PendingInput::XeqByName with a routing-mode field | New XeqByNameMode enum: Normal \| CollectForModal. | ✓ |
| Inspect state.modal_program at Enter time, no PendingInput change | Heuristic string-match on prompt text. Fragile. | |
| Two separate PendingInput variants | Maximally explicit. Duplicates alpha-collection key handling. | |

**User's choice:** Extend with routing-mode field.
**Notes:** Operationalized as **D-29.8**. Compile-time exhaustive match over XeqByNameMode preserves FN-CLI-04 "no _ => " invariant.

### Follow-up: Who triggers the auto-open

| Option | Description | Selected |
|--------|-------------|----------|
| hp41-cli App event loop on state delta | Detection in CLI; mirrored in hp41-gui IPC layer. Core stays UI-agnostic. | ✓ |
| hp41-core sets a request flag CLI/GUI consume | Single source of truth in core. Couples core to UI concept. | |
| User must manually press XEQ | No auto-open logic. Discovery problem. | |

**User's choice:** hp41-cli App event loop on state delta.
**Notes:** Operationalized as **D-29.9**. New `ModalProgram::requires_alpha_label() -> bool` method is the pure read-only contract; each frontend owns its own modal-opening mechanism.

---

## Claude's Discretion

The following implementation details were explicitly left to the planner's discretion (captured in 29-CONTEXT.md `<decisions>` § "Claude's Discretion"):

- Module location for `submit_modal` / `cancel_modal` / `requires_alpha_label` (likely `math1/mod.rs` + `math1/modal.rs`)
- `submit_modal` internal shape (exhaustive match vs thin wrappers around per-module submits)
- `advance_with_label(label: &str)` signature (method on ModalProgram vs separate function vs param of submit_modal)
- `function_matrix_parity.rs` extension shape (one parameterized test vs two test functions)
- Category naming convention inside `hp41-math1-functions.json` (single top-level `"Math 1 Pac"` vs per-program categories — recommendation: per-program)
- `pending_prompt()` signature widening (`&App` vs `&PendingInput, Option<&str>` — either acceptable)
- Precedence when `pending_input.is_some()` AND `state.modal_program.is_some()` simultaneously (likely modal_program wins; planner tests it)
- JSON `divergences` field per-entry population (full catalog moves to `docs/hp41-math1-divergences.md` in Phase 30)

Also help-overlay sectioning was deferred to Claude's discretion at the gray-area-selection step (user did not select it for discussion).

## Deferred Ideas

All deferred ideas listed in 29-CONTEXT.md `<deferred>` map cleanly to Phase 30, Phase 31, or Phase 32 boundaries — no new deferred ideas surfaced during discussion. The Phase 29 ↔ Phase 30 boundary did get sharpened though (JSON authoring pulled forward into Phase 29 per D-29.1; Phase 30 / DOC-01 shrinks accordingly).
