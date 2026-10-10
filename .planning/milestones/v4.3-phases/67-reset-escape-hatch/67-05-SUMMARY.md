---
phase: 67-reset-escape-hatch
plan: 05
subsystem: docs
tags: [adr, divergence-doc, discoverability, react, onboarding, quality-gates]

# Dependency graph
requires:
  - phase: 67-reset-escape-hatch/67-01
    provides: CalcState::soft_reset() and memory_lost() core implementations
  - phase: 67-reset-escape-hatch/67-02
    provides: CLI Ctrl+R two-tier prompt and Rdprgm→Ctrl+E conflict resolution
  - phase: 67-reset-escape-hatch/67-03
    provides: GUI-Rust reset_soft/reset_full Tauri commands with autosave-overwrite
  - phase: 67-reset-escape-hatch/67-04
    provides: GUI/iOS ON-key tap=soft / long-press=full wiring and confirm sheet
provides:
  - ADR v4.3-007 documenting the reset escape-hatch architecture and the intentional ON-semantics divergence
  - D-CV-10 divergence entry in docs/hp41cv-divergences.md (soft reset vs hardware ON preservation)
  - One-line discoverability hint for the ON-key gesture in OnboardingWizard.tsx and HelpOverlay.tsx
  - Human-verify gate T-67-11 PASSED (all four quality gates green, confirmed 2026-06-10)
  - D-25.6 parity confirmation: CLI Ctrl+R contract == GUI ON contract
affects: [v4.3-release, pr-26-merge, tagging]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Reset not an Op: escape hatch is documented as bypassing dispatch; reset absent from all six docs/hp41*-functions.json pools"
    - "Intentional-divergence pattern: ADR + divergence-doc entry together record why hardware-accurate ON is deliberately not emulated for soft reset"

key-files:
  created:
    - docs/adr/v4.3-007-reset-escape-hatch.md
  modified:
    - docs/hp41cv-divergences.md
    - hp41-gui/src/OnboardingWizard.tsx
    - hp41-gui/src/HelpOverlay.tsx

key-decisions:
  - "67-05-D01: ADR records the outside-dispatch invariant and rejected alternatives (reset-as-Op: self-defeating + forces 4-way match; frontend-only: SC-4 duplication/drift risk) — both alternatives have correctness/maintenance implications, so the decision record is load-bearing."
  - "67-05-D02: Divergence D-CV-10 placed in docs/hp41cv-divergences.md (general CV divergence file, same home as GETKEY D-CV-05 and X-MEM entries) — hardware ON is a CX-class Continuous Memory event, not an XROM module behavior."

patterns-established:
  - "Two-document pattern for intentional divergence: an ADR provides rationale and tradeoffs; a divergence-doc entry gives the concise behavioral delta. Both are required for maintainer legibility."

requirements-completed: [RESET-01]

# Metrics
duration: 15min
completed: 2026-06-10
---

# Phase 67 Plan 05: Closeout — ADR + Divergence + Discoverability + Quality Gates Summary

**ADR v4.3-007 + D-CV-10 divergence entry + ON-key discoverability hint in onboarding/help overlay ship Phase 67's documentation layer; all four quality gates (just ci, just gui-ci, MSRV clippy, root fmt) confirmed green by human on 2026-06-10**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-06-10T16:00:00Z
- **Completed:** 2026-06-10T16:15:00Z
- **Tasks:** 2 auto + 1 human-verify checkpoint (T-67-11)
- **Files modified:** 4

## Accomplishments

- `docs/adr/v4.3-007-reset-escape-hatch.md` authored in English: documents the problem (persisted input-blocking state survives restart via shared autosave), the chosen approach (two pure `CalcState` methods `soft_reset`/`memory_lost` invoked outside dispatch; frontends persist synchronously to overwrite autosave), rejected alternatives (reset-as-Op, frontend-only), and the intentional divergence from hardware ON semantics (real ON preserves Continuous Memory via power toggle; soft reset deliberately clears working state to actually unstick). Status: Accepted.
- `docs/hp41cv-divergences.md` gains divergence entry D-CV-10: hardware ON preserves everything; emulator soft reset clears working state + traps while preserving stored data; full reset maps to the authentic ON+left-arrow MEMORY LOST sequence.
- Discoverability hint "ON: tap = soft reset (unstick), long-press = full reset (MEMORY LOST)" added to `OnboardingWizard.tsx` and `HelpOverlay.tsx` in English. Reset is not an `Op` and is absent from all six `docs/hp41*-functions.json` pools.
- Human-verify gate T-67-11 PASSED: all four quality checks confirmed green; D-25.6 parity confirmed.

## Task Commits

1. **Task 1: ADR v4.3-007 + D-CV-10 divergence entry + discoverability hint** - `3ec7675` (docs)
2. **Task 2: Quality-gate run (just ci + just gui-ci + MSRV clippy + root fmt) + D-25.6 parity self-check** - `0f8c35f` (feat)

**Plan metadata:** (this commit — docs: complete phase 67 closeout)

## Files Created/Modified

- `docs/adr/v4.3-007-reset-escape-hatch.md` — New ADR. English. Documents the escape-hatch architecture, the outside-dispatch invariant, rejected alternatives (reset-as-Op, frontend-only), intentional ON-semantics divergence, CLI D-07 exception, GUI long-press approximation. Status: Accepted.
- `docs/hp41cv-divergences.md` — Added D-CV-10 entry: soft-reset-vs-ON divergence (working-state clear vs hardware Continuous Memory preservation) and full-reset ON+left-arrow MEMORY LOST mapping.
- `hp41-gui/src/OnboardingWizard.tsx` — Added one-line ON-key reset discoverability hint to the onboarding wizard step copy.
- `hp41-gui/src/HelpOverlay.tsx` — Added one-line ON-key reset discoverability hint to the `?` overlay (plain UI copy; not a function-JSON entry).

## Decisions Made

- **67-05-D01:** ADR records the outside-dispatch invariant and rejected alternatives (reset-as-Op: self-defeating + forces 4-way match; frontend-only: SC-4 duplication/drift risk). Both alternatives have correctness/maintenance implications, so the decision record is load-bearing for future maintainers.
- **67-05-D02:** Divergence entry placed in `docs/hp41cv-divergences.md` (general CV divergence file) — same home as GETKEY (D-CV-05) and X-MEM entries; hardware ON is a CX-class Continuous Memory event, not an XROM module behavior.

## Human-Verify Checkpoint (T-67-11)

**Result: PASSED — confirmed by human on 2026-06-10**

All four quality gates confirmed green:

| Gate | Command | Result |
|------|---------|--------|
| Root cargo fmt | `cargo fmt --check` | PASS (clean) |
| MSRV clippy | `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings` | PASS (no warnings) |
| just ci | `just ci` | PASS (exit 0; all Rust tests; Free42 contamination none; coverage 95.26% regions / 93.07% lines) |
| just gui-ci | `just gui-ci` | PASS (exit 0; TS type-check clean; GUI Rust 108 tests; vitest 348 passed / 12 files; release build clean) |

**Coverage note:** 93.07% lines = pre-Phase-67 baseline (not a regression; Phase 67 added only docs + TS UI hints, no new Rust source paths). Region gate: 95.26% >= 93% target — passes.

**Threat mitigations confirmed:**

- T-67-12 (Repudiation — undocumented divergence from hardware ON semantics): mitigated by ADR v4.3-007 + D-CV-10 divergence entry.
- T-67-13 (Tampering — Plans 01-04 regression ships unverified): mitigated; all four quality gates green.

## D-25.6 CLI-GUI Parity Self-Check

**Result: CONFIRMED**

| Axis | CLI (Ctrl+R) | GUI (ON key) | Match |
|------|-------------|--------------|-------|
| Soft reset trigger | `Ctrl+R` → type `s` | Tap ON key (<600 ms) | Yes — same `soft_reset()` core call |
| Full reset trigger | `Ctrl+R` → type `f` → `y` confirm | Long-press ON key (>=600 ms) + Confirm | Yes — same `memory_lost()` core call |
| Soft behavior | Clears working state + traps; preserves programs, registers, flags, X-MEM | Clears working state + traps; preserves programs, registers, flags, X-MEM | Yes |
| Full behavior | Factory state (`CalcState::new()`) | Factory state (`CalcState::new()`) | Yes |
| Frontend transient state | `ResetPrompt` cleared; `pending_input` None | `shiftActive=false`; `pendingInput` cleared | Yes (both cleared) |
| Location in IPC / CalcState | `ResetPrompt` on `App` (not CalcState); `#[serde(skip)]` | `shiftActive` in React state (not CalcState/IPC) | Yes — neither crosses IPC or lives in CalcState |
| Dispatch bypass | Intercept above `pending_input` routing block (D-07 exception) | Wired via dedicated Keyboard pointer props, never through `key_map.resolve()` | Yes — both bypass dispatch |

Both frontends call the same two core methods (`soft_reset`/`memory_lost`) and both overwrite the autosave synchronously on the persist path. Parity holds.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 67 is complete: all 5 plans done, all success criteria met.
- Phase 67 deliverables: core soft_reset/memory_lost (67-01), CLI Ctrl+R two-tier prompt (67-02), GUI-Rust reset commands + autosave-overwrite (67-03), GUI/iOS ON-key escape hatch + confirm sheet (67-04), ADR + divergence doc + discoverability hints + gates green (67-05).
- Ready for v4.3 milestone release: tag `v4.3` on develop, then `gh pr merge 26 --merge` (NEVER `--squash`). See MEMORY.md project_milestone_state for the full release checklist.

---
*Phase: 67-reset-escape-hatch*
*Completed: 2026-06-10*

## Self-Check: PASSED

- [x] Commit `3ec7675` present: `git log --oneline | grep 3ec7675` → "3ec7675 docs(67-05): add ADR v4.3-007 reset escape hatch + D-CV-10 divergen..."
- [x] Commit `0f8c35f` present: `git log --oneline | grep 0f8c35f` → "0f8c35f feat(67-05): add ON-key reset discoverability hint in onboarding wi..."
- [x] `docs/adr/v4.3-007-reset-escape-hatch.md` created in commit `3ec7675`
- [x] `docs/hp41cv-divergences.md` modified in commit `3ec7675`
- [x] `hp41-gui/src/OnboardingWizard.tsx` modified in commit `0f8c35f`
- [x] `hp41-gui/src/HelpOverlay.tsx` modified in commit `0f8c35f`
- [x] Human-verify checkpoint T-67-11 confirmed PASSED (all 4 gates green; 2026-06-10)
- [x] D-25.6 parity self-check recorded
- [x] No source/doc content modified in this finalization step
