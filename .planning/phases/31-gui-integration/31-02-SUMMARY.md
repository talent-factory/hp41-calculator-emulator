---
phase: 31-gui-integration
plan: "02"
subsystem: hp41-gui/hp41-core
tags: [cancellation, tauri-command, atomicbool, deadlock-avoidance, ci-gate, permission-coverage]
dependency_graph:
  requires: []
  provides: [request_cancel-tauri-command, CancelFlag-managed-state, cancel_requested-reset-on-open, check-tauri-permissions-ci-gate]
  affects: [hp41-gui/src-tauri/src/commands.rs, hp41-gui/src-tauri/src/lib.rs, hp41-core/src/ops/math1/integ.rs, hp41-core/src/ops/math1/solve.rs, hp41-core/src/ops/math1/difeq.rs]
tech_stack:
  added: []
  patterns: [separate-managed-state-for-cancel, deadlock-avoidance-via-atomic, workflow-opener-reset-idempotency]
key_files:
  created:
    - scripts/check-tauri-permissions.sh
    - hp41-gui/src-tauri/permissions/request-cancel.toml
    - hp41-gui/src-tauri/tests/cancel_command_no_deadlock.rs
    - hp41-gui/src-tauri/tests/cancel_autosave_stress.rs
    - hp41-core/tests/cancel_flag_reset_on_open.rs
  modified:
    - Justfile
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/capabilities/default.json
    - hp41-core/src/ops/math1/integ.rs
    - hp41-core/src/ops/math1/solve.rs
    - hp41-core/src/ops/math1/difeq.rs
decisions:
  - "reset cancel_requested in interactive dispatch arms (op_integ/op_solve/op_difeq) not in run_loop arms — preserves the existing per-64-samples cancel-mid-run behavior"
  - "CancelFlag = Arc<AtomicBool> as separate Tauri managed state (not AppState) to avoid deadlock when dispatch_op holds the AppState Mutex during long runs"
metrics:
  duration: 11m 5s
  completed: "2026-05-17"
  tasks_completed: 4
  tasks_total: 4
---

# Phase 31 Plan 02: Cancellation Channel — Summary

**One-liner:** `request_cancel` Tauri command with separate CancelFlag managed state + sticky-cancel fix in three workflow openers + permission CI gate.

## What Was Built

### Task 1: scripts/check-tauri-permissions.sh CI gate (604a73d, 2708330)

Authored `scripts/check-tauri-permissions.sh` — a bash CI gate that verifies every
command registered in `generate_handler!` in `hp41-gui/src-tauri/src/lib.rs` has a
matching `permissions/<kebab>.toml` file. Uses `grep -oE 'commands::[a-z_]+'` to
extract command names, kebab-cases via `sed 's/_/-/g'`, checks TOML existence.
Exits 0 when all commands covered; non-zero with `MISSING:` lines otherwise.

Wired into `Justfile`'s `gui-ci` recipe as the FIRST step (before npm ci) so
permission-coverage failures fail fast before expensive build steps.

Result: `bash scripts/check-tauri-permissions.sh` → `OK: all 5 commands have permission TOMLs`
(against the pre-Task-2 baseline of 5 commands).

### Task 2: request_cancel Tauri command + CancelFlag managed state + permission TOML (82da9d4)

**lib.rs changes:**
- Added `pub type CancelFlag = std::sync::Arc<std::sync::atomic::AtomicBool>;` after the
  existing `AppState` type alias
- In `setup()`, clone the Arc BEFORE wrapping `initial_state` in Mutex:
  `let cancel_flag: CancelFlag = std::sync::Arc::clone(&initial_state.cancel_requested);`
- Added `app.manage(cancel_flag);` AFTER `app.manage(Mutex::new(initial_state));`
- Added `commands::request_cancel` to `generate_handler!`

**commands.rs changes:**
- Added `use crate::{AppState, CancelFlag};` to imports
- Added `pub fn request_cancel(cancel: State<'_, CancelFlag>) -> Result<(), GuiError>`:
  - Body: `cancel.store(true, std::sync::atomic::Ordering::Relaxed); Ok(())`
  - CRITICAL: takes `State<'_, CancelFlag>`, NOT `State<'_, AppState>` — deadlock avoidance
  - Does NOT call `state.lock()` or touch AppState in any way

**Created permissions/request-cancel.toml** (copied from run-stop.toml shape):
```toml
"$schema" = "../gen/schemas/desktop-schema.json"
[[permission]]
identifier = "allow-request-cancel"
description = "Allows the request_cancel command."
commands.allow = ["request_cancel"]
```

**Updated capabilities/default.json**: added `"allow-request-cancel"` to `permissions` array.

Result: `bash scripts/check-tauri-permissions.sh` → `OK: all 6 commands have permission TOMLs`.
`cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0.

### Task 3: cancel_requested reset at workflow opener entry (5295070)

**Surgical 3×1-line hp41-core exception** — documented carve-out per Plan 31-02 objective.

Insertion points (in the `!is_running` interactive dispatch arm of each function,
BEFORE opening the modal, AFTER any guard logic):

| File | Line | Function |
|------|------|----------|
| `hp41-core/src/ops/math1/integ.rs` | **169** | `op_integ()` — interactive dispatch arm |
| `hp41-core/src/ops/math1/solve.rs` | **125** | `op_solve()` — interactive dispatch arm |
| `hp41-core/src/ops/math1/difeq.rs` | **148** | `op_difeq()` — interactive dispatch arm |

Each insertion:
```rust
state.cancel_requested.store(false, std::sync::atomic::Ordering::Relaxed);
```

**Why the dispatch arm, not the run_loop arm:** The existing test
`integ_cancel_requested_fires_at_sample_boundary` (math1_integ.rs:200) pre-sets
`cancel_requested = true` and calls `op_integ_run_loop` directly — this test simulates
a cancel arriving mid-run. If the reset were in the run_loop arm, this test would break
(the reset would clear the flag before the per-64-samples check fires). The dispatch arm
(called when the user presses INTG/SOLVE/DIFEQ interactively) is the correct "workflow opener"
— it runs when the user initiates a new computation, clearing any prior sticky cancel.

**API widening confirmation:** NO new `pub` or `pub(super)` functions added. NO new
`Op::*` variants. NO new `CalcState` fields. The carve-out is purely 3 additive one-line
`store(false, Relaxed)` calls inside existing functions.

**Authored `hp41-core/tests/cancel_flag_reset_on_open.rs`** with 3 tests:
- `cancel_flag_resets_on_integ_open` — pre-set `true`, dispatch `Op::Integ`, assert `false`
- `cancel_flag_resets_on_solve_open` — pre-set `true`, dispatch `Op::Solve`, assert `false`
- `cancel_flag_resets_on_difeq_open` — pre-set `true`, dispatch `Op::Difeq`, assert `false`

All 1616 hp41-core tests pass after this change (including all 16 existing integ tests).

### Task 4: cancel_command_no_deadlock + cancel_autosave_stress integration tests (51a035c)

**cancel_command_no_deadlock.rs** — `request_cancel_does_not_acquire_appstate_mutex`:
- Acquires AppState Mutex in foreground thread (simulating dispatch_op holding lock during INTG)
- Spawns worker calling `simulate_request_cancel` (the production body without Tauri wrapper)
- Asserts worker completes within 100ms (deadlock would cause worker to block indefinitely)
- Asserts `cancel_flag` reads `true` after worker
- Asserts AppState Mutex is still held by foreground during worker execution
- No-deadlock wall-clock: ~0ms (atomic store, no contention expected)

**cancel_autosave_stress.rs** — `autosave_and_cancel_interleave_without_deadlock`:
- Thread A: holds AppState for 200ms, polls cancel_flag inside the lock (simulates INTG)
- Thread B: clones AppState Arc, locks every 30ms (simulates 30s auto-save thread)
- Thread C: calls simulate_request_cancel every 10ms (simulates GUI cancel presses)
- Runtime: ~200ms (Thread A release triggers Thread B progress)
- Asserts: A observed cancel signal inside held lock; B acquired lock >=1 time; no thread panicked

Both tests pass within 30s: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test cancel_command_no_deadlock --test cancel_autosave_stress` exits 0.

## Verification Results

| Check | Result |
|-------|--------|
| `bash scripts/check-tauri-permissions.sh` | OK: all 6 commands have permission TOMLs |
| `cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml` | Finished [optimized] |
| `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test cancel_command_no_deadlock --test cancel_autosave_stress` | 2 passed |
| `cargo test --package hp41-core --test cancel_flag_reset_on_open` | 3 passed |
| `grep cancel_requested.store count in math1/` | Exactly 3 matches |
| SC-4 grep (no `fn op_add|sub|mul|...` in gui src-tauri) | 0 matches (PASS) |
| All 1616 hp41-core tests | PASS |
| All 2 new GUI integration tests | PASS |

## Threat Mitigations

| Threat | Status |
|--------|--------|
| T-31-W1-permission-coverage | Mitigated: `check-tauri-permissions.sh` gates `gui-ci`; all 6 commands covered |
| T-31-W1-deadlock | Mitigated: `request_cancel` takes `State<'_, CancelFlag>` (not AppState); two tests confirm |
| T-31-W1-sticky-cancel | Mitigated: 3×store(false) at workflow-opener entry; cancel_flag_reset_on_open.rs confirms |

## Surgical Exception Documentation

Per Plan 31-02 `<surgical_exception>` block:
- **3 insertion points recorded:** integ.rs:169, solve.rs:125, difeq.rs:148
- **API surface unchanged:** no new pub functions, no new Op::* variants, no new CalcState fields
- **Phase 28 math1 freeze intact:** only the 3×store(false) lines were added to the interactive
  dispatch arms; the run_loop arms, all algorithm code, and all other math1 files are untouched

## Deviations from Plan

### Auto-adjusted Issue: Reset in dispatch arm vs run_loop arm

**Found during:** Task 3
**Issue:** The plan text initially described placing the reset in "workflow openers" generically.
If placed in the `op_integ_run_loop` / `op_solve_run_loop` / `op_difeq_run_loop` functions
(the run_loop arms), the existing test `integ_cancel_requested_fires_at_sample_boundary`
would break — that test pre-sets cancel_requested = true and calls op_integ_run_loop directly
to verify the per-64-samples cancellation fires. The reset would clear the flag before the check.
**Fix (Rule 1 — bug):** Placed the resets in the interactive dispatch arms (`op_integ`,
`op_solve`, `op_difeq` with `!state.is_running`) instead. This is the correct "workflow opener"
— it fires when the user presses the key interactively to start a new computation, not during
the computation itself.
**Behavior:** The idempotency invariant is correctly satisfied: pre-cancel → press INTG →
reset → new run is clean. Mid-run cancel continues to work (per-64-samples checks in run_loop
still read the flag set by request_cancel).
**Tests:** All 1616 hp41-core tests pass, confirming no regression.

## Known Stubs

None — Plan 31-02 is infrastructure only (cancellation plumbing). No data flows that could produce
stubs. The `request_cancel` command is fully wired end-to-end (Tauri thunk → CancelFlag → AtomicBool).

## Threat Flags

None — no new network endpoints, auth paths, or file access patterns introduced.
The `request_cancel` command is idempotent and lock-free.

## Self-Check: PASSED

Files created/committed:
- scripts/check-tauri-permissions.sh: FOUND
- hp41-gui/src-tauri/permissions/request-cancel.toml: FOUND
- hp41-gui/src-tauri/tests/cancel_command_no_deadlock.rs: FOUND
- hp41-gui/src-tauri/tests/cancel_autosave_stress.rs: FOUND
- hp41-core/tests/cancel_flag_reset_on_open.rs: FOUND

Commits verified:
- 604a73d (check-tauri-permissions.sh): FOUND
- 2708330 (Justfile gui-ci wire): FOUND
- 82da9d4 (request_cancel command): FOUND
- 5295070 (cancel_requested resets): FOUND
- 51a035c (integration tests): FOUND
