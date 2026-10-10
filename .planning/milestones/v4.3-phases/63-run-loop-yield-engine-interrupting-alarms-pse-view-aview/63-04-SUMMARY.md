---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
plan: "04"
subsystem: hp41-gui/src-tauri
tags: [gui-run-loop, tauri-commands, pending-yield, pse-view-aview, SC-4, permissions, capabilities]

dependency_graph:
  requires:
    - pending_yield field on CalcState with YieldKind/YieldState types (63-01)
    - run_program + resume_program core functions (63-02)
    - PSE/VIEW/AVIEW yield arms in run_loop (63-02)
  provides:
    - run_program Tauri command (thin SC-4 glue around hp41_core::ops::program::run_program)
    - resume_program Tauri command (thin SC-4 glue around hp41_core::ops::program::resume_program)
    - pending_yield projected through CalcStateView as YieldView (kind/text/resume_ms)
    - permission TOMLs + capability references for both commands
    - generate_handler! registration for both commands
    - Rust unit tests asserting from_state pending_yield projection
  affects:
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/permissions/run-program.toml
    - hp41-gui/src-tauri/permissions/resume-program.toml
    - hp41-gui/src-tauri/capabilities/default.json

tech-stack:
  added: []
  patterns:
    - "SC-4 thin-glue pattern: lock → core fn call → drain print+event → from_state (4 lines, zero calculator logic)"
    - "Tauri v2.11 command registration: permission TOML (narrow commands.allow) + capability reference + generate_handler!"
    - "pending_yield projection: Option<YieldState> → Option<YieldView> with lowercase kind string for TS interop"
    - "Drain-before-from_state pattern (Pitfall 1): print_buffer.drain + event_buffer.drain before from_state(&calc, ...)"

key-files:
  created:
    - hp41-gui/src-tauri/permissions/run-program.toml
    - hp41-gui/src-tauri/permissions/resume-program.toml
  modified:
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/capabilities/default.json

key-decisions:
  - "63-04-D01: Use full core path hp41_core::ops::program::run_program (not re-export hp41_core::run_program) to avoid name-shadow with commands::run_stop."
  - "63-04-D02: Drain drain pattern mirrors handle_sst (not handle_get_state) so print+event lines from a run reach the view."
  - "63-04-D03: Mutex held for full run_loop segment (T-63-09 accepted tradeoff) — same as INTG/SOLVE/DIFEQ today; Phase-C check_alarms fires server-side within that call."
  - "63-04-D04: YieldView kind as lowercase string ('pse'/'view'/'aview') so TS layer needs no Rust enum knowledge."
  - "63-04-D05: display_override projection in from_state left completely untouched (D-04 / DISP-01 deferred)."

requirements-completed: [ALARM-02, ALARM-03, PRGM-01, PRGM-02]

duration: ~30min
completed: 2026-06-06
tasks_completed: 3
tasks_total: 3
files_changed: 6
---

# Phase 63 Plan 04: GUI Run-Loop Backend — Tauri Commands + pending_yield Projection Summary

**Two thin Tauri commands (`run_program` / `resume_program`) expose the core run loop to the GUI, registered with narrow permission TOMLs and capability references, and `pending_yield` is projected through `CalcStateView` — enabling the TS driver (63-06) to drive continuous program execution.**

---

## Performance

- **Duration:** ~30 min
- **Completed:** 2026-06-06
- **Tasks:** 3
- **Files modified:** 6

---

## What Was Built

### Task 1 — run_program + resume_program Tauri commands + pending_yield projection

**`hp41-gui/src-tauri/src/commands.rs`** — Two new `#[tauri::command]` functions, each ~4 lines of SC-4 thin glue:

```rust
// run_program: start a label program at the named entry point
pub fn run_program(label: String, state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::program::run_program(&mut calc, &label).map_err(GuiError::from)?;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(&calc, print_lines, event_lines))
}

// resume_program: continue after a yield/stop
pub fn resume_program(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::program::resume_program(&mut calc).map_err(GuiError::from)?;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(&calc, print_lines, event_lines))
}
```

**`hp41-gui/src-tauri/src/types.rs`** — `YieldView` serializable DTO and `pending_yield: Option<YieldView>` field on `CalcStateView`. Projected in `from_state` via `state.pending_yield.as_ref().map(YieldView::from_yield_state)`. `display_override` projection left completely untouched (D-04).

Three Rust unit tests added (mirror `handle_tick_time_drains_event_buffer`):
- `from_state_projects_pending_yield_when_set` — asserts kind=="pse", resume_ms==1000, display_override unaffected
- `from_state_pending_yield_none_when_not_set` — asserts None propagates
- `from_state_projects_yield_view_and_aview_kinds` — asserts "view" and "aview" lowercase mapping

### Task 2 — generate_handler! + permission TOMLs + capability references

- `hp41-gui/src-tauri/permissions/run-program.toml` — identifier `allow-run-program`, `commands.allow = ["run_program"]`
- `hp41-gui/src-tauri/permissions/resume-program.toml` — identifier `allow-resume-program`, `commands.allow = ["resume_program"]`
- `hp41-gui/src-tauri/src/lib.rs` — `commands::run_program,` and `commands::resume_program,` added to `generate_handler![]`
- `hp41-gui/src-tauri/capabilities/default.json` — `"allow-run-program"` and `"allow-resume-program"` added to `"permissions"` array

### Task 3 — iOS cfg(mobile) compile check + GUI clippy gate

- `cargo check --target aarch64-apple-ios`: 0 errors (new commands compile for iOS, no `#[cfg(mobile)]` regressions)
- `cargo clippy --all-targets -- -D warnings`: 2 pre-existing hits (types.rs:185 `unnecessary_unwrap`, commands.rs:596 `&*p`); both are documented pre-existing noise unrelated to this plan

---

## Accomplishments

- GUI can now drive continuous user-LBL programs — the capability the GUI has never had before this plan
- SC-4 preserved: zero calculator logic in hp41-gui/src-tauri/src/; both commands are 4-line lock-call-drain-return wrappers
- D-04 preserved: display_override projection in from_state is completely untouched
- T-63-10 (permission over-grant) mitigated: each command has its own narrow TOML with `commands.allow` containing exactly one entry
- 129 GUI Rust tests + 337 GUI TypeScript tests (just gui-ci) all green

---

## Task Commits

1. **Task 1: run_program + resume_program commands + pending_yield projection** — `46d4e3d` (feat)
2. **Task 2: register in generate_handler! + permission TOMLs + capability refs** — `d2c20d4` (feat)
3. **Task 3: iOS compile check + GUI clippy gate** — (verification-only; no code change needed)

---

## Files Created/Modified

- `hp41-gui/src-tauri/src/commands.rs` — Two new `#[tauri::command]` functions + 3 unit tests for pending_yield projection
- `hp41-gui/src-tauri/src/types.rs` — `YieldView` struct + `pending_yield: Option<YieldView>` on `CalcStateView` + `from_state` projection
- `hp41-gui/src-tauri/src/lib.rs` — `commands::run_program` + `commands::resume_program` in `generate_handler![]`
- `hp41-gui/src-tauri/permissions/run-program.toml` — New: narrow permission TOML for `run_program`
- `hp41-gui/src-tauri/permissions/resume-program.toml` — New: narrow permission TOML for `resume_program`
- `hp41-gui/src-tauri/capabilities/default.json` — `allow-run-program` + `allow-resume-program` added

---

## Deviations from Plan

None — plan executed exactly as written. All three tasks implemented precisely as specified. The two pre-existing clippy hits (types.rs:185 + commands.rs:596) were present before this plan and are documented acceptable noise.

---

## Known Stubs

None. Both commands are real implementations calling the core run_loop. The TS driver wiring (63-06) is a separate planned deliverable, not a stub.

---

## Threat Flags

None. The new commands introduce no new network endpoints, auth paths, or schema changes. Permission scope is intentionally narrow (one `commands.allow` entry per TOML — T-63-10 mitigated).

---

## Self-Check: PASSED

- `hp41-gui/src-tauri/src/commands.rs` — FOUND, contains `pub fn run_program` + `pub fn resume_program` + `from_state_projects_pending_yield_when_set` test
- `hp41-gui/src-tauri/src/types.rs` — FOUND, contains `YieldView` struct + `pending_yield: Option<YieldView>` field
- `hp41-gui/src-tauri/src/lib.rs` — FOUND, `generate_handler!` contains both `commands::run_program` and `commands::resume_program`
- `hp41-gui/src-tauri/permissions/run-program.toml` — FOUND
- `hp41-gui/src-tauri/permissions/resume-program.toml` — FOUND
- `hp41-gui/src-tauri/capabilities/default.json` — FOUND, contains `allow-run-program` and `allow-resume-program`
- Commit `46d4e3d` — FOUND in `git log --oneline`
- Commit `d2c20d4` — FOUND in `git log --oneline`
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — 129 passed
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` — clean
- `cargo check --target aarch64-apple-ios --manifest-path hp41-gui/src-tauri/Cargo.toml` — clean
- `just gui-ci` — 129 Rust + 337 TS tests passing
- SC-4: `grep -rn -E "fn op_(add|sub|...)" hp41-gui/src-tauri/src/` — empty (PASS)
- `git diff HEAD -- hp41-gui/src-tauri/Cargo.toml` — empty (no new deps)
- `git status` after commits — only `M hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` (pre-existing, NOT staged by this plan)
