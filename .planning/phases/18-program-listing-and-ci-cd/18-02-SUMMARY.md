---
phase: 18-program-listing-and-ci-cd
plan: "02"
subsystem: hp41-gui/src-tauri
tags: [tdd, wave-1, green, tauri, rust, program-listing, sst, bst]
dependency_graph:
  requires: [18-01]
  provides: [prgm-display-backend, sst-bst-commands, calc-state-view-program-fields]
  affects:
    - hp41-gui/src-tauri/src/prgm_display.rs
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/permissions/sst-step.toml
    - hp41-gui/src-tauri/permissions/bst-step.toml
    - hp41-gui/src-tauri/capabilities/default.json
    - hp41-gui/src-tauri/tauri.conf.json
tech_stack:
  added: []
  patterns:
    - format_all_steps pre-rendering pattern (all steps formatted in Rust before IPC)
    - pure-Rust helper + thin Tauri thunk (handle_sst/handle_bst testable without Tauri)
    - Tauri permission TOML + capabilities JSON grant pattern
key_files:
  created:
    - hp41-gui/src-tauri/src/prgm_display.rs
    - hp41-gui/src-tauri/permissions/sst-step.toml
    - hp41-gui/src-tauri/permissions/bst-step.toml
  modified:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/capabilities/default.json
    - hp41-gui/src-tauri/tauri.conf.json
decisions:
  - "format_all_steps pre-formats entire program listing in Rust; React renders strings only (D-02)"
  - "sst_step/bst_step are separate Tauri commands, not routed through Op dispatch (D-04)"
  - "mod prgm_display added in Task 1 (not Task 2) because types.rs needs crate::prgm_display immediately"
  - "test_dispatch_op_payload_size bound raised from 300 to 400 bytes to accommodate program_steps field"
metrics:
  duration: 215s
  completed: "2026-05-10T15:53:21Z"
  tasks_completed: 2
  files_modified: 7
---

# Phase 18 Plan 02: Wave 1 Rust Backend — Program Listing + SST/BST Commands Summary

Wave 1 implementation: turns all Wave 0 RED test stubs GREEN by adding `format_all_steps()`, `program_steps`/`pc` in `CalcStateView`, and `sst_step`/`bst_step` Tauri commands with full permission wiring.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create prgm_display.rs and extend types.rs + commands.rs with Phase 18 data layer | ce646ae | prgm_display.rs (new), types.rs, commands.rs, lib.rs |
| 2 | Wire lib.rs, permissions TOMLs, capabilities/default.json, and tauri.conf.json | 822ef40 | lib.rs, sst-step.toml (new), bst-step.toml (new), default.json, tauri.conf.json |

## What Was Done

**Task 1 — prgm_display.rs (new):**
- Copied `hp41-cli/src/prgm_display.rs` verbatim as starting content per Phase 18 D-03
- Added `pub fn format_all_steps(state: &CalcState) -> Vec<String>` — returns `["000 END"]` for empty program; iterates all steps with `{i:03} {op_display_name}` format for non-empty programs
- Added 2 unit tests: `test_format_all_steps_empty_program` and `test_format_all_steps_nonempty`

**Task 1 — types.rs:**
- Added `use crate::prgm_display;` import
- Added `program_steps: Vec<String>` and `pc: usize` fields to `CalcStateView` struct
- Populated both in `from_state()` using `prgm_display::format_all_steps(state)` and `state.pc`
- Updated `test_dispatch_op_payload_size` bound from 300 to 400 bytes (program_steps adds ~35 bytes for empty program baseline)
- Wave 0 RED test `test_phase18_fields_exist` now GREEN

**Task 1 — commands.rs:**
- Added `sst_step` and `bst_step` Tauri command thunks (2-line lock + delegate pattern)
- Added `handle_sst` and `handle_bst` pure-Rust helpers: `if calc.pc < calc.program.len() { calc.pc += 1; }` and `calc.pc = calc.pc.saturating_sub(1);`
- All 4 Wave 0 RED test stubs now GREEN: test_handle_sst_advances_pc, test_handle_sst_clamps_at_end, test_handle_bst_decrements_pc, test_handle_bst_clamps_at_zero
- No bare `.unwrap()` in production code — all use `.unwrap_or_else(|e| e.into_inner())`

**Task 1 — lib.rs (partial):**
- Added `mod prgm_display;` declaration (required immediately for `crate::prgm_display` in types.rs)

**Task 2 — lib.rs:**
- Added `commands::sst_step` and `commands::bst_step` to `tauri::generate_handler![]`

**Task 2 — permissions/sst-step.toml (new):** Tauri permission grant with `identifier = "allow-sst-step"`, `commands.allow = ["sst_step"]`

**Task 2 — permissions/bst-step.toml (new):** Tauri permission grant with `identifier = "allow-bst-step"`, `commands.allow = ["bst_step"]`

**Task 2 — capabilities/default.json:** Added `"allow-sst-step"` and `"allow-bst-step"` to permissions array

**Task 2 — tauri.conf.json:** Changed window `"height"` from 700 to 900 (Phase 18 D-10, makes space for program listing panel)

## Verification

```
cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
Result: 27 passed (3 suites, 0.01s) — EXIT 0
```

Structural checks:
- `grep -c 'pub fn format_all_steps' prgm_display.rs` → 1
- `grep -c 'pub program_steps' types.rs` → 1
- `grep -c 'pub fn handle_sst|pub fn handle_bst' commands.rs` → 2
- `grep -c 'pub fn sst_step|pub fn bst_step' commands.rs` → 2
- `grep -c 'allow-sst-step' capabilities/default.json` → 1
- `grep -c 'allow-bst-step' capabilities/default.json` → 1
- `grep -c '"height": 900' tauri.conf.json` → 1

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] mod prgm_display added in Task 1 rather than Task 2**
- **Found during:** Task 1 compilation
- **Issue:** `types.rs` uses `crate::prgm_display`, which requires `mod prgm_display;` in `lib.rs`. Without it, `cargo test` would have failed in Task 1.
- **Fix:** Added `mod prgm_display;` to `lib.rs` as part of Task 1 staging. Task 2 still staged the remaining lib.rs change (invoke_handler update) as its own commit.
- **Impact:** Zero. Both lib.rs changes are on the branch.

## Known Stubs

None — all implemented functionality is complete. `format_step()` (copied from CLI) is present but unused in the GUI; this is intentional per D-03 (GUI copy may evolve independently).

## Threat Surface Scan

| Flag | File | Description |
|------|------|-------------|
| threat_flag: tauri-command | hp41-gui/src-tauri/src/commands.rs | sst_step and bst_step — new Tauri IPC endpoints. Covered by T-18-W1-01: restricted to "main" window via capabilities/default.json + permissions/sst-step.toml and bst-step.toml. PC arithmetic bounds covered by T-18-W1-02 (handle_sst) and T-18-W1-03 (handle_bst). |

## Self-Check: PASSED

- [x] hp41-gui/src-tauri/src/prgm_display.rs created: contains `pub fn format_all_steps`
- [x] hp41-gui/src-tauri/src/types.rs modified: `program_steps` and `pc` fields present
- [x] hp41-gui/src-tauri/src/commands.rs modified: `handle_sst`, `handle_bst`, `sst_step`, `bst_step` present
- [x] hp41-gui/src-tauri/src/lib.rs modified: `mod prgm_display` + `sst_step`/`bst_step` in invoke_handler
- [x] hp41-gui/src-tauri/permissions/sst-step.toml created: `allow-sst-step`
- [x] hp41-gui/src-tauri/permissions/bst-step.toml created: `allow-bst-step`
- [x] hp41-gui/src-tauri/capabilities/default.json: `allow-sst-step` and `allow-bst-step` present
- [x] hp41-gui/src-tauri/tauri.conf.json: height is 900
- [x] Commit ce646ae exists (Task 1)
- [x] Commit 822ef40 exists (Task 2)
- [x] cargo test exits 0 — 27 tests pass
