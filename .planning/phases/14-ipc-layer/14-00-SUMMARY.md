---
phase: 14-ipc-layer
plan: "00"
subsystem: api
tags: [tauri, rust, ipc, tdd, red-scaffold, serde]

# Dependency graph
requires:
  - phase: 13-workspace-skeleton
    provides: "AppState = Mutex<CalcState> type alias in lib.rs; Tauri v2 workspace skeleton; hp41-core path dependency"
provides:
  - "types.rs: CalcStateView, Annunciators, GuiError structs with Serialize derive and unimplemented!() constructors"
  - "key_map.rs: resolve(key_id: &str) -> Result<Op, GuiError> signature with 3 RED tests"
  - "commands.rs: dispatch_op, get_state Tauri thunks + handle_op, handle_get_state pure-Rust helpers with 2 RED tests"
  - "lib.rs mod wiring: mod commands; mod key_map; mod types; (generate_handler![] placeholder unchanged)"
  - "9 RED tests spanning all 3 success criteria (SC-1, SC-2, SC-3) — all fail with unimplemented!(Wave 1)"
affects: [14-01, 14-02, 14-03, 15-react-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wave 0 TDD scaffold: public struct shapes + fn signatures + #[cfg(test)] RED tests; Wave 1 fills bodies"
    - "Tauri command thunk/helper split: #[tauri::command] is 2-line glue; handle_* helpers are unit-testable"
    - "IPC DTO pattern: CalcStateView is outbound-only (Serialize, no Deserialize); GuiError is minimal (message: String)"

key-files:
  created:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/key_map.rs
    - hp41-gui/src-tauri/src/commands.rs
  modified:
    - hp41-gui/src-tauri/src/lib.rs

key-decisions:
  - "Wave 0 scaffold uses unimplemented!(\"Wave 1 ...\") in all production fn bodies — crate compiles but tests fail (canonical RED)"
  - "generate_handler![] placeholder comment intentionally unchanged — premature command registration would auto-generate capabilities before Plan 03 is ready (RESEARCH.md Pitfall 2)"
  - "Tauri thunk/helper split: dispatch_op/get_state are 2-line glue; handle_op/handle_get_state are pub helpers testable without WebView"
  - "GuiError: Serialize only (no Display, no std::error::Error) — Tauri v2 requires Serialize on error type; adding Error trait would conflict per RESEARCH.md Pitfall 6"
  - "CalcStateView: Serialize only (no Deserialize) — outbound DTO; no deserialization surface"
  - "mod declarations are private (not pub mod) — submodules are internal to hp41-gui crate"

patterns-established:
  - "IPC helper pattern: #[tauri::command] thunks call pure-Rust handle_* fns; tests call handle_* directly"
  - "Wave 0/Wave 1 split: Wave 0 creates files+tests, Wave 1 implements bodies; compile succeeds at both stages"
  - "print_buffer drain-before-from_state: drain takes &mut, then from_state takes &; avoids borrow conflict (Pitfall 1)"

requirements-completed: [IPC-01]

# Metrics
duration: 4min
completed: "2026-05-09"
---

# Phase 14 Plan 00: IPC Layer Wave 0 Scaffold Summary

**Wave 0 TDD RED scaffold: three IPC layer files (types.rs, key_map.rs, commands.rs) with public struct shapes, function signatures, and 9 failing tests that establish the contract for Wave 1 implementation**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-09T16:22:22Z
- **Completed:** 2026-05-09T16:26:06Z
- **Tasks:** 4
- **Files modified:** 4 (3 created, 1 modified)

## Accomplishments

- Created `types.rs` with `CalcStateView`, `Annunciators`, `GuiError` structs (Serialize derive, no Deserialize); `from_state` and `From<HpError>` bodies are `unimplemented!(Wave 1)`; 4 RED tests targeting SC-1 (payload size), display string structure, annunciators defaults, and GuiError conversion
- Created `key_map.rs` with `resolve(key_id: &str) -> Result<Op, GuiError>` signature; body is `unimplemented!(Wave 1)`; 3 RED tests covering named ops, unknown key GuiError (SC-2), and compound parameterized keys
- Created `commands.rs` with `#[tauri::command]` thunks (`dispatch_op`, `get_state`) + pure-Rust helpers (`handle_op`, `handle_get_state`); all 4 fn bodies are `unimplemented!(Wave 1)`; 2 RED tests covering unknown-key error return (SC-2) and print_buffer drain contract (SC-3)
- Modified `lib.rs` to add `mod commands; mod key_map; mod types;` (alphabetical); `generate_handler![]` placeholder comment intentionally left unchanged for Plan 02

## Task Commits

All four tasks committed atomically as a single Wave 0 scaffold commit:

1. **Tasks 1-4: Wave 0 RED scaffold (types, key_map, commands, lib.rs)** - `960ccbb` (test)

**Plan metadata:** (pending — committed after SUMMARY.md)

_Note: All 4 tasks are TDD RED phase — single commit captures the complete scaffold._

## Files Created/Modified

- `hp41-gui/src-tauri/src/types.rs` — CalcStateView, Annunciators, GuiError structs + 4 RED tests
- `hp41-gui/src-tauri/src/key_map.rs` — resolve() signature + 3 RED tests
- `hp41-gui/src-tauri/src/commands.rs` — Tauri command thunks + handle_* helpers + 2 RED tests
- `hp41-gui/src-tauri/src/lib.rs` — Added mod declarations (commands, key_map, types)

## RED State Confirmation

All 9 tests fail with `not implemented: Wave 1 ...` panics:

```
failures:
    commands::tests::test_dispatch_op_unknown_key
    commands::tests::test_print_buffer_drained
    key_map::tests::test_key_map_compound_keys
    key_map::tests::test_key_map_named_ops
    key_map::tests::test_key_map_unknown_key
    types::tests::test_annunciators_from_state
    types::tests::test_calc_state_view_structure
    types::tests::test_dispatch_op_payload_size
    types::tests::test_gui_error_from_hp_error
```

`cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` exits 0.
`cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --no-run` exits 0.

## Public Signatures Locked for Wave 1

Plan 01 and Plan 02 MUST implement against these exact signatures:

```rust
// types.rs
pub fn CalcStateView::from_state(state: &hp41_core::CalcState, print_lines: Vec<String>) -> Self;
impl From<HpError> for GuiError;

// key_map.rs
pub fn resolve(key_id: &str) -> Result<Op, GuiError>;

// commands.rs
pub fn handle_op(calc: &mut CalcState, key_id: &str) -> Result<CalcStateView, GuiError>;
pub fn handle_get_state(calc: &mut CalcState) -> Result<CalcStateView, GuiError>;
#[tauri::command] pub fn dispatch_op(key_id: &str, state: State<'_, AppState>) -> Result<CalcStateView, GuiError>;
#[tauri::command] pub fn get_state(state: State<'_, AppState>) -> Result<CalcStateView, GuiError>;
```

## Notes for Downstream Plans

**For Plan 01 (types.rs + key_map.rs implementation):**
- Replace `unimplemented!("Wave 1 (Plan 01): ...")` in `from_state`, `From<HpError>`, and `resolve`
- `from_state`: entry_buf priority, then alpha_mode, then format_hpnum; drain happens BEFORE calling from_state (borrow conflict Pitfall 1)
- `resolve`: flat match + `resolve_parameterized()` private fn per RESEARCH.md Pattern 5

**For Plan 02 (commands.rs implementation):**
- Replace `unimplemented!("Wave 1 (Plan 02): ...")` in all 4 command/helper fn bodies
- `generate_handler![]` placeholder comment MUST be replaced with `commands::dispatch_op, commands::get_state,`
- Digit key short-circuit ("0"-"9", ".", "e") goes BEFORE `key_map::resolve()` call per Pattern 6

**For Plan 03 (capabilities):**
- Capability update BLOCKED until Plan 02 registers commands and first `gui-check` confirms
  auto-generated permission IDs in `hp41-gui/src-tauri/gen/schemas/desktop-schema.json`
- Expected permission names: `"allow-dispatch-op"`, `"allow-get-state"` (kebab-case convention)

## Decisions Made

- Wave 0 uses `unimplemented!("Wave 1 (Plan NN): ...")` (not `todo!()` or `panic!()`) for grep-stable RED markers
- `generate_handler![]` placeholder intentionally left unchanged — premature registration would trigger capability auto-generation before Plan 03 is ready (Pitfall 2)
- All 4 tasks committed as a single atomic commit since they form a single coherent Wave 0 scaffold

## Deviations from Plan

None - plan executed exactly as written. All acceptance criteria verified.

## Issues Encountered

None. `cargo check` passed on first run. 9 RED tests confirmed with expected `unimplemented!(Wave 1)` panics.

## Known Stubs

The `unimplemented!("Wave 1 ...")` bodies in `from_state`, `From<HpError>`, `resolve`, `handle_op`, `handle_get_state`, `dispatch_op`, and `get_state` are intentional Wave 0 scaffolds. They do NOT prevent the plan's goal (creating the TDD RED scaffold). Plan 01 and Plan 02 replace them with real implementations. These stubs close at Wave 1 completion.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Wave 0 scaffold complete — 9 RED tests established, crate compiles, test binary compiles
- Plan 01 can begin immediately: implement `CalcStateView::from_state`, `From<HpError>`, and `resolve()`
- Plan 02 follows Plan 01: implement `handle_op`, `handle_get_state`, `dispatch_op`, `get_state`, wire into `generate_handler![]`
- Plan 03 blocked until Plan 02 first build confirms auto-generated permission IDs

---
*Phase: 14-ipc-layer*
*Completed: 2026-05-09*
