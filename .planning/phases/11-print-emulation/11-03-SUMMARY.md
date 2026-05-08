---
phase: 11-print-emulation
plan: "11-03"
subsystem: print-emulation
tags: [serde, rust, app, CalcState, print_buffer, run_program]

requires:
  - phase: 11-02
    provides: PRX/PRA/PRSTK ops implemented in hp41-core; call_dispatch_and_drain in hp41-cli

provides:
  - serde(skip) on print_buffer — transient print output never serialized to JSON save files
  - drain_and_show_print_output() helper on App — decoupled drain after programmatic execution
  - Three run_program() call sites wired: F5/R/S, F1-F4 USER handler, try_user_dispatch

affects: [phase-12, persistence, print-emulation]

tech-stack:
  added: []
  patterns:
    - "drain-after-programmatic-run: call drain_and_show_print_output() after every run_program() Ok(()) branch"
    - "serde-skip-transient: use #[serde(default, skip)] for fields that are runtime-only and must not be persisted"

key-files:
  created: []
  modified:
    - hp41-core/src/state.rs
    - hp41-cli/src/app.rs

key-decisions:
  - "serde(default, skip) on print_buffer: default preserves backward-compat deserialization of v1.0 save files; skip prevents serialization of transient state (CR-03)"
  - "drain_and_show_print_output() as dedicated helper: decouples drain logic from dispatch, keeps call sites clean at all three programmatic run paths (CR-01)"

patterns-established:
  - "Programmatic execution paths (run_program) must call drain_and_show_print_output() after Ok(()) — mirrors drain logic of call_dispatch_and_drain"

requirements-completed: [PRNT-01, PRNT-02, PRNT-03, PRNT-04]

duration: 15min
completed: "2026-05-08"
---

# Phase 11 Plan 03: Gap Closure (CR-01 + CR-03) Summary

**serde(skip) on print_buffer closes the autosave serialization gap (CR-03); drain_and_show_print_output() wired to all three run_program call sites closes the silent-discard gap (CR-01)**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-08T23:35:00Z
- **Completed:** 2026-05-08T23:50:00Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments

- CR-03 closed: `print_buffer` on `CalcState` now carries `#[serde(default, skip)]` — transient print output is never written to the 30-second autosave JSON file; old v1.0 save files still deserialize cleanly via `default`
- CR-01 closed: `drain_and_show_print_output()` private helper added to `App`; all three `run_program()` call sites (F5/R/S, F1-F4 USER handler, `try_user_dispatch`) now drain `print_buffer` and surface output in the TUI status bar after `Ok(())`
- `just ci` green: 94.00% hp41-core coverage (gate ≥80%), all tests pass, clippy clean

## Task Commits

1. **Task 1: Add serde(skip) to print_buffer in state.rs** - `f713e24` (fix)
2. **Task 2: Add drain_and_show_print_output() helper and wire three run_program call sites** - `4ae4fd7` (feat)
3. **Task 3: Verify CI + fix dead_code warning on test helper** - `91e381e` (fix)

## Files Created/Modified

- `hp41-core/src/state.rs` — Changed `#[serde(default)]` to `#[serde(default, skip)]` on `print_buffer` field (line 94); updated doc comment to explain both attributes
- `hp41-cli/src/app.rs` — Added `drain_and_show_print_output()` private method; wired to F5/R/S handler (line 409), F1-F4 USER handler (line 242), and `try_user_dispatch` (line 818); added `#[allow(dead_code)]` on unused `key()` test helper

## Decisions Made

- Used `#[serde(default, skip)]` (not just `skip`) so old v1.0 save files that lack `print_buffer` still deserialize via `Default::default()` — a single annotation handles both backward-compat deserialization and exclusion from serialization
- `drain_and_show_print_output()` is a private (not `pub(crate)`) method — only called by `handle_key()` paths, not by tests that exercise `call_dispatch_and_drain()` directly

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Suppressed dead_code warning on unused test helper `key()` in print_modal_tests**
- **Found during:** Task 3 (CI run)
- **Issue:** The `key()` helper function in the `print_modal_tests` module was defined but not called by any test — tests use `KeyEvent::new()` directly. The `-D warnings` clippy flag promoted this to an error, breaking CI.
- **Fix:** Added `#[allow(dead_code)]` to the function definition; the function is a legitimate utility for future test expansion
- **Files modified:** `hp41-cli/src/app.rs`
- **Verification:** `just ci` exits 0 after fix
- **Committed in:** `91e381e` (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug)
**Impact on plan:** The fix was necessary to achieve CI green. No scope change — one annotation added to a test helper.

## Issues Encountered

None beyond the auto-fixed dead_code lint above.

## Known Stubs

None — both gap fixes are complete. `print_buffer` is fully wired through all three programmatic execution paths.

## Next Phase Readiness

- Phase 11 blocker gaps CR-01 and CR-03 are fully closed
- All print operations (PRX/PRA/PRSTK) now surface in the TUI status bar regardless of whether triggered interactively (P+X/A/S) or programmatically (F5/F1-F4/USER key dispatch)
- Phase 12 can proceed without print-emulation blockers

## Self-Check

- [x] `grep -n 'serde.default, skip' hp41-core/src/state.rs` — line 94 confirmed
- [x] `grep -c 'drain_and_show_print_output' hp41-cli/src/app.rs` — returns 4 (1 definition + 3 call sites)
- [x] `git log --oneline` shows f713e24, 4ae4fd7, 91e381e
- [x] `just ci` exits 0, coverage 94.00%

## Self-Check: PASSED

---
*Phase: 11-print-emulation*
*Completed: 2026-05-08*
