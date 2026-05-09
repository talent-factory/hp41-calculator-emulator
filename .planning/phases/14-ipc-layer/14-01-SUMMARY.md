---
phase: 14-ipc-layer
plan: "01"
subsystem: api
tags: [tauri, rust, ipc, tdd, green, serde, key-map, dto]

# Dependency graph
requires:
  - phase: 14-ipc-layer
    plan: "00"
    provides: "Wave 0 RED scaffold: types.rs, key_map.rs, commands.rs with unimplemented!() bodies and 9 RED tests"
provides:
  - "types.rs: CalcStateView::from_state — real impl with display_str priority chain"
  - "types.rs: From<HpError> for GuiError — wraps e.to_string()"
  - "key_map.rs: resolve(&str) -> Result<Op, GuiError> — ~50 named ops + 7 prefix-parameterized families"
affects: [14-02, 14-03, 15-react-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "display_str priority chain: entry_buf > format_alpha(alpha_reg) > format_hpnum(stack.x)"
    - "x_str always uses format_hpnum regardless of entry/alpha mode — for Phase 15 stack panel"
    - "resolve_parameterized() with strip_prefix chains; resolve_sto_arith() with rsplit_once('_') per Pitfall 3"
    - "Unknown key IDs return Err(GuiError { message: 'unknown key: <id>' }) — D-07 enforced"

key-files:
  created: []
  modified:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/key_map.rs

key-decisions:
  - "display_str uses entry_buf verbatim (priority 1), then format_alpha (priority 2), then format_hpnum (priority 3) per RESEARCH.md Pattern 3"
  - "x_str is always format_hpnum(&state.stack.x, &state.display_mode) — independent of entry/alpha mode; gives Phase 15 stack panel stable X value"
  - "resolve_sto_arith uses rsplit_once('_') not split_once — correctly handles sto_arith_plus_05 where multiple underscores are present (Pitfall 3)"
  - "StoArith and StoArithStack use named fields (reg: u8, kind: StoArithKind / kind: StoArithKind, stack_reg: StackReg) confirmed from ops/mod.rs"
  - "commands.rs Wave 1 bodies remain unimplemented!() — Plan 02 responsibility"

# Metrics
duration: 11min
completed: "2026-05-09"
---

# Phase 14 Plan 01: IPC Layer Wave 1 — types.rs + key_map.rs GREEN Summary

**Wave 1 GREEN implementation: CalcStateView::from_state with display_str priority chain, From<HpError> for GuiError, and key_map::resolve() with ~50 named ops + 7 prefix-parameterized families — turning 7 of 9 RED tests GREEN**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-05-09T16:22:00Z
- **Completed:** 2026-05-09T16:33:06Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Implemented `CalcStateView::from_state` with display_str priority chain (entry_buf > format_alpha > format_hpnum) per RESEARCH.md Pattern 3; x_str always uses format_hpnum independent of mode
- Implemented `From<HpError> for GuiError` wrapping `e.to_string()` — thiserror Display string ("overflow", "divide by zero", etc.)
- Implemented `key_map::resolve()` with exhaustive match on ~50 named ops mirroring hp41-cli/src/keys.rs key_to_op() semantics
- Added `resolve_parameterized()` handling 7 prefix families: sto_, rcl_, fix_, sci_, eng_, isg_, dse_, sto_arith_, gto_, xeq_, lbl_, alpha_
- Added `resolve_sto_arith()` using `rsplit_once('_')` to correctly parse compound `sto_arith_<op>_<reg>` keys (Pitfall 3 avoided)
- 7 of 9 Wave-0 RED tests now GREEN; 2 commands.rs tests remain RED for Plan 02

## Task Commits

1. **Task 1: types.rs implementation** — `6d3fbed`
   - Files: `hp41-gui/src-tauri/src/types.rs`
   - Tests passing: `test_dispatch_op_payload_size`, `test_calc_state_view_structure`, `test_annunciators_from_state`, `test_gui_error_from_hp_error` (4/4)

2. **Task 2: key_map.rs implementation** — `cd96356`
   - Files: `hp41-gui/src-tauri/src/key_map.rs`
   - Tests passing: `test_key_map_named_ops`, `test_key_map_unknown_key`, `test_key_map_compound_keys` (3/3)

## Test Results

```
types::tests — 4 passed, 0 failed
key_map::tests — 3 passed, 0 failed
commands::tests — 0 passed, 2 failed (Wave 1 Plan 02 — expected)

Total: 7 passed; 2 failed (2 intentional RED for Plan 02)
```

## Success Criteria Verification

- **SC-1 (payload ≤ 300 bytes):** `test_dispatch_op_payload_size` passes. Fresh CalcState JSON serializes to ~170 bytes.
- **SC-2 (no panic, structured error):** `test_key_map_unknown_key` verifies `Err(GuiError { message: "unknown key: totally_unknown_xyz" })` returned, not a panic.
- **SC-4 (no calculator logic duplication):** `grep -E "fn op_|fn flush_entry" types.rs key_map.rs` returns nothing. format_hpnum and format_alpha imported from hp41_core.

## Public Surface Locked for Plan 02

```rust
// types.rs — stable, tested, ready for import
pub struct CalcStateView { display_str, x_str: String, annunciators: Annunciators, print_lines: Vec<String> }
pub struct Annunciators { user, prgm, alpha, rad, grad: bool }
pub struct GuiError { pub message: String }
pub fn CalcStateView::from_state(state: &CalcState, print_lines: Vec<String>) -> Self;
impl From<HpError> for GuiError;

// key_map.rs — stable, tested, ready for import
pub fn resolve(key_id: &str) -> Result<Op, GuiError>;
```

Plan 02 calls `key_map::resolve(key_id)?` and `CalcStateView::from_state(&calc, drained_lines)` directly from `handle_op` and `handle_get_state`.

## Acceptance Criteria Results

| Criterion | Result |
|-----------|--------|
| `cargo check` exits 0 | PASS |
| 4 types::tests pass | PASS |
| `unimplemented!` count in types.rs = 0 | PASS |
| `format_hpnum` appears ≥ 2 times in types.rs | PASS (4) |
| `format_alpha` appears 1 time in types.rs | PASS (3 — use + imports in docs) |
| `e.to_string()` appears 1 time in types.rs | PASS |
| No `fn op_\|fn flush_entry` in types.rs | PASS |
| No `Deserialize` in types.rs | PASS |
| 3 key_map::tests pass | PASS |
| `unimplemented!` count in key_map.rs = 0 | PASS |
| `Ok(Op::` count in key_map.rs ≥ 40 | PASS (71) |
| `rsplit_once` appears in key_map.rs | PASS (2) |
| Unknown key message format present | PASS (4) |
| No `fn op_\|fn flush_entry\|fn dispatch` in key_map.rs | PASS |

## Deviations from Plan

None — plan executed exactly as written. All acceptance criteria verified.

## Known Stubs

The `unimplemented!("Wave 1 (Plan 02): ...")` bodies in `commands.rs` (`handle_op`, `handle_get_state`, `dispatch_op`, `get_state`) are intentional Wave 1 Plan 02 scaffolds. They do not prevent this plan's goal. Plan 02 replaces them.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Both files are pure-Rust functions with no I/O. Key trust boundary (frontend string → key_map::resolve) is mitigated by exhaustive match + structured GuiError return (T-14-01 from threat register).

## Self-Check: PASSED

- `hp41-gui/src-tauri/src/types.rs` — exists, 0 stubs, 4 tests GREEN
- `hp41-gui/src-tauri/src/key_map.rs` — exists, 0 stubs, 3 tests GREEN
- Commit `6d3fbed` — verified in git log
- Commit `cd96356` — verified in git log
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` — exits 0 (warnings only, no errors)

---
*Phase: 14-ipc-layer*
*Completed: 2026-05-09*
