---
quick_id: 260516-c1p
slug: fix-gui-clp-binding-and-modal-letter-clicks
status: complete
date: "2026-05-16"
release_tag: v2.2.1
duration_minutes: 35
tags: [gui, modal, keyboard, clp, parity, hotfix]
key_files:
  modified:
    - hp41-gui/src/Keyboard.tsx
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
    - hp41-gui/src/test_setup.ts
    - docs/verifying-card-reader.md
    - .planning/notes/v3.x-tech-debt-on-screen-modal-letter-input.md
    - .planning/ROADMAP.md
  created:
    - .planning/quick/260516-c1p-fix-gui-clp-binding-and-modal-letter-clicks/260516-c1p-PLAN.md
    - .planning/quick/260516-c1p-fix-gui-clp-binding-and-modal-letter-clicks/260516-c1p-SUMMARY.md
decisions:
  - "Mode-aware shifted via new optional `shiftedInPrgm` field on KeyDef (vs. replacing the existing `shifted` slot) — keeps x² reachable outside PRGM and matches HP-41C/CV hardware key_path 'f-C' literally"
  - "alphaChar fallback applies even when consumesShift=true — UX consistency for letter keys; the shifted variant of a letter key has no useful meaning inside a label modal anyway"
  - "scrollIntoView stub lives in src/test_setup.ts (one-line jsdom polyfill) — fixes hard test failure for Group G AND silences pre-existing unhandled-exception noise in Groups A–F"
  - "Doc patch documents BOTH key sequences (CLI + GUI) inline rather than splitting into separate sections — keeps the verification procedure scannable as a single flow"
---

# Quick Task 260516-c1p — v2.2.1 GUI Parity Hotfix — Summary

**One-liner:** Wired CLP modal opener to PRGM-mode SHIFT+√x (hardware-faithful
mode-aware shifted), added on-screen-keyboard letter-click support to
LBL/XEQ/GTO/CLP/ASN-label modals via `alphaChar` fallback, and corrected
`docs/verifying-card-reader.md` for both surfaces. Closes a documentation-
breaking GUI parity gap that violated v2.2's D-25.6 CLI ↔ GUI parity
invariant.

## Tasks Completed

| # | Task | Status |
|---|------|--------|
| 1 | KeyDef.shiftedInPrgm field + sqrt entry in Keyboard.tsx | Done |
| 2 | MODAL_OPENERS.clp_prompt in App.tsx | Done |
| 3 | handleClick: mode-aware shifted resolution in App.tsx | Done |
| 4 | handleClick: alphaChar fallback for label modals in App.tsx | Done |
| 5 | Vitest tests (Group G — 5 cases) + scrollIntoView jsdom stub | Done |
| 6 | docs/verifying-card-reader.md GUI sequences corrected | Done |
| 7 | Quality gates (just gui-check, just gui-ci) green | Done |
| 8 | Backlog note flipped to RESOLVED-IN-v2.2.1 + ROADMAP.md crossed-out | Done |

## Must-Have Verification

| Truth | Result |
|-------|--------|
| GUI: SHIFT + √x in PRGM mode opens CLP modal (LCD `CLP _`) | PASS — Vitest G1 |
| GUI: outside PRGM mode, SHIFT + √x dispatches Op::Sq (x²) | PASS — Vitest G2 |
| GUI: alphaChar click appends letter into open xeq_name/clp/assign_label modal | PASS — Vitest G3 (LBL), G4 (CLP) |
| GUI: pre-fix EEX-types-'E' bug fixed (now types alphaChar 'P') | PASS — Vitest G5 |
| docs/verifying-card-reader.md Section 2.01, 3.4, and 6 are executable end-to-end on GUI | PASS — visual review |
| Backend unchanged (SC-4 + CalcState backward-compat preserved) | PASS — only hp41-gui/src/ touched |
| `npm test` green on hp41-gui (147 / 147) | PASS — `just gui-ci` |
| `cargo check` + tests green on hp41-gui/src-tauri | PASS — `just gui-ci` |
| Backlog note flipped to resolved-in-v2.2.1 | PASS |

## Test Coverage Delta

| Metric | Before | After |
|--------|--------|-------|
| Vitest total tests | 142 | 147 (+5) |
| Vitest test files | 5 | 5 (unchanged) |
| Group G new cases | — | G1, G2, G3, G4, G5 |
| Vitest unhandled exceptions | scrollIntoView noise on every test | 0 (jsdom stub in test_setup.ts) |

## Deviations from Plan

### Auto-fixed Issues

**1. Test infrastructure — jsdom missing Element.scrollIntoView**

- **Found during:** Task 5 (running Group G tests).
- **Symptom:** G1 and G4 (both seed `prgm: true` on initial mount) failed with `Error: could not find key id="shift" in keyboard SVG`. Root cause traced via the unhandled `TypeError: activeStepRef.current?.scrollIntoView is not a function` in `App.tsx:528` — the program panel renders when prgm=true, the ref captures the div, scrollIntoView throws, the render tree fails to settle, and `findKey('shift')` no longer locates the SHIFT key.
- **Fix:** Added a one-line `Element.prototype.scrollIntoView` no-op stub to `src/test_setup.ts`. Bonus benefit — this silences pre-existing unhandled-exception noise from Groups A–F where scrollIntoView was also called but the assertion happened to complete before the exception bubbled.
- **Pattern:** matches the rest of `test_setup.ts` (jsdom polyfill scope, single-line `if (!Element.prototype.scrollIntoView)` guard so future browsers picking it up don't get overridden).

**2. Plan-time CLP modal expectation: shift gets consumed by alphaChar fallback even when both `shifted` and `alphaChar` exist**

- **Found during:** Task 4 (reviewing the routedKey-building chain).
- **Concern:** if a user has SHIFT armed and clicks a letter key inside a label modal (e.g. SHIFT + Σ+ → shifted variant is `sigma_minus`), should the modal append 'A' or no-op?
- **Decision:** append 'A' (consume shift). The shifted variants of letter-bearing keys (Σ-, yx, x², 10ˣ, eˣ, …) have no useful meaning inside a label modal; a letter click clearly signals intent to type. Documented in the PLAN's Task 4 + Decision log above.
- **No code change vs. plan; documenting decision lineage.**

## Self-Check: PASSED

- hp41-gui/src/Keyboard.tsx — modified: `KeyDef.shiftedInPrgm?` field added, `sqrt` entry carries `shiftedInPrgm: { id: 'clp_prompt', label: 'CLP' }`.
- hp41-gui/src/App.tsx — modified: `MODAL_OPENERS.clp_prompt` registered; `handleClick` resolution rule 3 honors `shiftedInPrgm` when `annunciators.prgm` is active; new `alphaChar` fallback branch in the modal routing chain (case (d) in the comment).
- hp41-gui/src/App.test.tsx — appended Group G (5 cases: G1, G2, G3, G4, G5).
- hp41-gui/src/test_setup.ts — appended scrollIntoView jsdom stub.
- docs/verifying-card-reader.md — Section 2.01 split into CLI/GUI flows; Section 3.4 CLP step clarified for both surfaces; Section 6 expanded with three GUI input-path notes.
- .planning/notes/v3.x-tech-debt-on-screen-modal-letter-input.md — status flipped to RESOLVED-IN-v2.2.1 with cross-reference.
- .planning/ROADMAP.md — backlog item struck through, points at this quick-task.
- `just gui-ci` exits 0 — Vitest 147/147 passing, Rust check + tests green, release build green.

## Follow-ups (not in this quick-task)

- **`v2.2.1` git tag** — user owns release tagging (per project convention).
- **E2E smoke spec extension** (`hp41-gui/e2e/smoke.spec.ts`) — could add a CLP / LBL flow check; deferred per FN-QUAL-05 / D-27.13 (smoke stays at literal ROADMAP scope until v3.x).
- **`f LBL` direct mnemonic for the CLI status bar** — out of scope; CLI is unchanged.
