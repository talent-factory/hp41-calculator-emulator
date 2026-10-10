---
phase: 67-reset-escape-hatch
plan: 02
subsystem: cli
tags: [rust, hp41-cli, reset, escape-hatch, state-machine, ratatui]

# Dependency graph
requires:
  - phase: 67-01
    provides: "CalcState::soft_reset() and CalcState::memory_lost() — called directly from the CLI reset handler"
provides:
  - "CLI Ctrl+R reset escape-hatch — two-tier status-bar state machine in hp41-cli"
  - "ResetPrompt enum (None/AwaitingTier/AwaitingFullConfirm) on App"
  - "Ctrl+E reassignment for RDPRGM card shortcut (Ctrl+R conflict resolved)"
  - "13 integration tests in phase67_reset_cli.rs covering all state transitions"
affects:
  - phase 67-05 (docs/discoverability — this plan's behavior is what the hint describes)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "D-07 documented exception — reset intercept placed ABOVE pending_input routing block so escape hatch beats stuck modals"
    - "Two-tier ResetPrompt state machine on App — None/AwaitingTier/AwaitingFullConfirm, never persisted"
    - "Synchronous save_state inside reset handler — overwrites autosave immediately so recovery survives restart"
    - "CLI-transient-state sweep on reset — shift_armed, pending_input, show_help, help_search_query, show_programs all cleared"

key-files:
  created:
    - hp41-cli/tests/phase67_reset_cli.rs
  modified:
    - hp41-cli/src/app.rs
    - hp41-cli/src/ui.rs

key-decisions:
  - "67-02-D01: Ctrl+R conflict — Ctrl+R was previously the RDPRGM card-reader comfort shortcut; reassigned to Ctrl+E (mnemonic: rEad program); the existing test was renamed to test_ctrl_e_dispatches_rdprgm"
  - "67-02-D02: ResetPrompt placed on App (not CalcState) — it is purely CLI transient UI state; never serialised; always None on startup"
  - "67-02-D03: Reset intercept placed above pending_input routing block but BELOW the WaitForKey and Release-filter guards — those two must stay first; the D-07 exception applies specifically to the escape-hatch intercept vs modal routing"
  - "67-02-D04: render_status checks reset_prompt FIRST (highest priority) — overrides pending_input prompts, ALPHA message, and normal message during the two-tier flow"

patterns-established:
  - "Pattern: CLI escape-hatch beats modal — document exception explicitly with a comment naming the invariant being broken (D-07) and why"

requirements-completed: [RESET-01]

# Metrics
duration: 16min
completed: 2026-06-10
---

# Phase 67 Plan 02: CLI Reset Escape-Hatch Summary

**Ctrl+R intercept in hp41-cli with two-tier status-bar prompt (AwaitingTier/AwaitingFullConfirm) calling CalcState::soft_reset() or CalcState::memory_lost() with synchronous persistence**

## Performance

- **Duration:** 16 min
- **Started:** 2026-06-10T12:57:35Z
- **Completed:** 2026-06-10T13:13:33Z
- **Tasks:** 1 (implementation + tests in single atomic commit)
- **Files modified:** 2 (app.rs, ui.rs)
- **Files created:** 1 (phase67_reset_cli.rs — 13 integration tests)

## Accomplishments

- Added `ResetPrompt` enum (None/AwaitingTier/AwaitingFullConfirm) to `App` struct; field initialised to `ResetPrompt::None` in `App::new`
- Intercept Ctrl+R at the very top of `handle_key`, above the `pending_input` routing block — documented intentional D-07 exception; the escape hatch must beat every stuck state including a trapped modal
- Two-tier status-bar state machine: Ctrl+R → show tier prompt; `s` → soft reset + persist; `f` → show confirm; `y` confirm → full reset + persist; Esc/other → cancel
- Both reset paths sweep CLI-only transient state: `shift_armed`, `pending_input`, `show_help`, `help_search_query`, `show_programs`
- Both reset paths persist synchronously via `save_state` — overwrites the shared autosave so recovery survives an app restart
- Updated `render_status` in `ui.rs` to show the verbatim locked prompt copy when reset_prompt is active (highest priority — overrides all other status text)
- Resolved Ctrl+R conflict: Ctrl+R was the RDPRGM card-reader comfort shortcut; reassigned to Ctrl+E; existing test renamed `test_ctrl_e_dispatches_rdprgm`
- 13 integration tests in `phase67_reset_cli.rs`: RST-CLI-01 (Ctrl+R over active pending_input), RST-CLI-02 (soft reset path), RST-CLI-03 (full reset path), RST-CLI-04 (f+n/Esc/other cancel), RST-CLI-05 (tier Esc/other cancel), RST-CLI-06 (shift_armed cleared), RST-CLI-07 (Ctrl+R no longer dispatches RDPRGM)

## Task Commits

1. **Task 1: Implementation + tests** — `32e0c4e` (feat)

## Files Created/Modified

- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/src/app.rs` — ResetPrompt enum, App.reset_prompt field, Ctrl+R intercept block, two-tier state machine in handle_key, Ctrl+E reassignment for Rdprgm, existing test renamed
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/src/ui.rs` — render_status updated to show reset_prompt text at highest priority
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/tests/phase67_reset_cli.rs` — 13 integration tests for the CLI reset escape-hatch

## Decisions Made

- **67-02-D01:** Ctrl+R conflict resolved by reassigning RDPRGM to Ctrl+E — the reset escape-hatch is the higher-priority use of Ctrl+R; card-reader shortcuts are convenience bindings; the hardware-faithful `XEQ "RDPRGM"` path is unchanged. Documented in SUMMARY.
- **67-02-D02:** `ResetPrompt` placed on `App`, not `CalcState` — it is purely CLI transient UI state that must never be serialised or cross the IPC boundary.
- **67-02-D03:** Reset intercept positioned BELOW the WaitForKey guard and Release-filter guard (those must remain first) but ABOVE the `pending_input` routing block — this is the narrowest D-07 exception needed.
- **67-02-D04:** `render_status` checks `reset_prompt` first — ensures the locked prompt text is always visible even when `pending_input` or `alpha_mode` would otherwise override the status bar.

## Ctrl+R Conflict Check (Required by Plan)

The plan required verifying Ctrl+R against existing CLI app-level bindings. **Conflict found:**

| Binding | Previous op | Resolution |
|---------|-------------|------------|
| Ctrl+R  | `Op::Rdprgm` (card-reader "Read program" comfort shortcut) | Reassigned to Ctrl+E |

Ctrl+W (Wprgm), Ctrl+D (Wdta), Ctrl+F (Rdta), Ctrl+S (manual save), Ctrl+P (programs overlay), Ctrl+A (key assignment), Ctrl+C (quit) — all free of conflict.

## Deviations from Plan

None — plan executed exactly as written. The Ctrl+R conflict was expected by the plan ("check Ctrl+R for conflicts before wiring it") and the resolution (reassign Rdprgm) is the natural outcome.

## Known Stubs

None — all state transitions are fully wired; persistence calls are real.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The persistence call in the reset handler uses the same `save_state` path already in use for the 30-second autosave.

---

## Self-Check: PASSED

- FOUND: `hp41-cli/src/app.rs`
- FOUND: `hp41-cli/src/ui.rs`
- FOUND: `hp41-cli/tests/phase67_reset_cli.rs`
- FOUND: commit `32e0c4e`
- Tests: 536 passed (13 new), 4 ignored — `cargo test -p hp41-cli` green
- Clippy: `cargo clippy -p hp41-cli --all-targets -- -D warnings` — no issues
- MSRV: `cargo +1.88 clippy -p hp41-cli --all-targets -- -D warnings` — clean
- Format: `cargo fmt --check -p hp41-cli` — clean

---
*Phase: 67-reset-escape-hatch*
*Completed: 2026-06-10*
