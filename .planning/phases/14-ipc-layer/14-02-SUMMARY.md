---
phase: 14-ipc-layer
plan: "02"
subsystem: api
tags: [tauri, rust, ipc, tdd, green, commands, dispatch, key-handler]

# Dependency graph
requires:
  - phase: 14-ipc-layer
    plan: "01"
    provides: "CalcStateView::from_state, From<HpError> for GuiError, key_map::resolve — all public surface needed by commands.rs"
provides:
  - "commands.rs: handle_op(&mut CalcState, &str) — digit entry, EEX guards, key_map dispatch, print_buffer drain"
  - "commands.rs: handle_get_state(&mut CalcState) — drain print_buffer, build CalcStateView"
  - "commands.rs: dispatch_op Tauri thunk — lock + delegate to handle_op"
  - "commands.rs: get_state Tauri thunk — lock + delegate to handle_get_state"
  - "lib.rs: generate_handler![commands::dispatch_op, commands::get_state] registered"
affects: [14-03, 15-react-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "handle_op: digit keys (0-9, '.', 'e') appended to entry_buf — NO dispatch; mirrors CLI app.rs lines 342-388"
    - "INPUT-02 guard: cap EEX exponent at 2 digits via find('e') + count ascii_digit after position"
    - "D-07 implicit mantissa: empty entry_buf + 'e' pushes '1e' not 'e'"
    - "print_buffer drain pattern: drain(..).collect() BEFORE CalcStateView::from_state — borrow safety (Pitfall 1)"
    - "Tauri thunks: .unwrap_or_else(|e| e.into_inner()) for poisoned-lock recovery — zero .unwrap()"
    - "helper/thunk split: all logic in handle_op/handle_get_state (unit-testable); thunks are 2-line glue"

key-files:
  created: []
  modified:
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs

key-decisions:
  - "handle_op factored as a public pure-Rust helper (no Tauri State extractor) to enable unit tests without WebView mock harness"
  - "Digit key routing: explicit matches! on 12 string literals ('0'..'9', '.', 'e') — Rust string patterns do not support '0'..='9' range syntax"
  - "Tauri thunks use .unwrap_or_else(|e| e.into_inner()) — satisfies #![deny(clippy::unwrap_used)] from lib.rs root"
  - "dispatch() called with calc (not &mut *calc) because MutexGuard<CalcState> auto-derefs — compiles cleanly"
  - "generate_handler![] uses full-path commands::dispatch_op, commands::get_state for unambiguous macro resolution"

# Metrics
duration: 3min
completed: "2026-05-09"
---

# Phase 14 Plan 02: IPC Layer — commands.rs + lib.rs GREEN Summary

**Tauri command handlers handle_op and handle_get_state implemented with digit-entry guards, key_map dispatch, and print_buffer drain; both Tauri thunks registered in lib.rs generate_handler![] — all 9 Wave-0 tests GREEN**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-09T16:35:19Z
- **Completed:** 2026-05-09T16:38:21Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Implemented `handle_op(&mut CalcState, &str)` with complete routing:
  - Digit keys "0".."9": INPUT-02 guard (cap EEX exponent at 2 digits), push char to entry_buf, drain, return view — NO dispatch
  - ".": block duplicate '.' and '.' after 'e', push if valid, drain, return view — NO dispatch
  - "e": block double-EEX; empty entry_buf pushes "1e" (implicit mantissa D-07); else pushes "e" — NO dispatch
  - Named/parameterized keys: `key_map::resolve(key_id)?` then `dispatch(calc, op).map_err(GuiError::from)?` then drain, return view
- Implemented `handle_get_state(&mut CalcState)`: drain print_buffer, build CalcStateView::from_state
- Both Tauri thunks (`dispatch_op`, `get_state`) are 2-line glue: lock with poisoned-lock recovery + delegate to helper
- Updated lib.rs `generate_handler![]` with `commands::dispatch_op` and `commands::get_state`
- cargo check exits 0 (171 crates compiled); gen/schemas/ files generated

## Task Commits

1. **Task 1: commands.rs implementation** — `0d74627`
   - Files: `hp41-gui/src-tauri/src/commands.rs`
   - Tests passing: `test_dispatch_op_unknown_key`, `test_print_buffer_drained` (2/2 new GREEN)

2. **Task 2: lib.rs generate_handler registration** — `9f038cc`
   - Files: `hp41-gui/src-tauri/src/lib.rs`
   - All 9/9 tests pass; cargo check exits 0

## Test Results

```
types::tests    — 4 passed, 0 failed
key_map::tests  — 3 passed, 0 failed
commands::tests — 2 passed, 0 failed

Total: 9 passed; 0 failed (all Wave-0 tests GREEN)
```

## Success Criteria Verification

- **SC-2 (no panic on unknown key):** `test_dispatch_op_unknown_key` PASS — `handle_op(&mut calc, "totally_unknown_xyz")` returns `Err(GuiError { message: "unknown key: totally_unknown_xyz" })`, never panics.
- **SC-3 (print_buffer drained):** `test_print_buffer_drained` PASS — `handle_get_state` empties `calc.print_buffer` and the returned view contains 1 `print_lines` entry.
- **SC-4 (no calculator logic duplication):** `grep -rn "fn op_|fn flush_entry|fn format_hpnum" hp41-gui/src-tauri/src/` returns nothing. PASS.
- **SC-5 (AppState alias enforced):** `State<'_, AppState>` in both thunks; `AppState = Mutex<hp41_core::CalcState>` unchanged in lib.rs. Compile gate: PASS.

## Auto-Generated Permission IDs (for Plan 03)

After the `cargo check` run (171 crates compiled), the following gen/schemas/ files exist:

- `hp41-gui/src-tauri/gen/schemas/desktop-schema.json` — 113.3K (JSON Schema for capability validation)
- `hp41-gui/src-tauri/gen/schemas/macOS-schema.json` — 113.3K
- `hp41-gui/src-tauri/gen/schemas/acl-manifests.json` — 64.4K
- `hp41-gui/src-tauri/gen/schemas/capabilities.json` — 147B

**Permission ID status:** The auto-generated permission identifiers `allow-dispatch-op` and `allow-get-state` are generated by `tauri-build` during a FULL build (not cargo check alone). Per RESEARCH.md Pitfall 2 and assumption A2, Plan 03 must run `cargo check` or `just gui-check` as the first step to confirm the exact permission names before adding them to `capabilities/default.json`. The `generate_handler![]` macro is now populated with both commands, so the next build will produce the permissions.

**Note for Plan 03:** After running `just gui-check`, inspect `hp41-gui/src-tauri/permissions/autogenerated/` or the build output for the exact permission identifiers. Expected names (per kebab-case convention): `"allow-dispatch-op"` and `"allow-get-state"`. Verify against actual generated names before adding to `capabilities/default.json`.

## Acceptance Criteria Results

| Criterion | Result |
|-----------|--------|
| `cargo check` exits 0 | PASS |
| 2 commands::tests pass | PASS |
| `unimplemented!` count in commands.rs = 0 | PASS |
| `unwrap_or_else(|e| e.into_inner())` count = 2 (both thunks) | PASS |
| `key_map::resolve` appears 1 time | PASS |
| `print_buffer.drain(..)` appears ≥ 4 times | PASS (5 in production code, 1 in tests) |
| `CalcStateView::from_state` appears ≥ 4 times | PASS (5 in production code, 1 in tests) |
| No `.unwrap()` outside `#[cfg(test)]` block | PASS |
| No calculator logic (`fn op_`, `fn flush_entry`, `fn format_hpnum`) | PASS |
| `commands::dispatch_op` in lib.rs | PASS |
| `commands::get_state` in lib.rs | PASS |
| Placeholder comment removed from lib.rs | PASS |
| `mod commands/key_map/types` still present in lib.rs | PASS |
| `pub type AppState = Mutex<hp41_core::CalcState>` unchanged | PASS |
| All 9 Wave-0 tests GREEN | PASS |
| gen/schemas/desktop-schema.json exists | PASS |

## Deviations from Plan

None — plan executed exactly as written. Both tasks implemented per the exact code in the plan's `<action>` blocks without modification.

## Known Stubs

None — all `unimplemented!()` bodies replaced with real implementations.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced beyond what the plan's threat model covers. The `handle_op` digit routing appends only single ASCII chars to entry_buf (T-14-W2-03 mitigated). Unknown key IDs return structured GuiError (T-14-W2-04 mitigated). Both thunks use poisoned-lock recovery — no panic path (T-14-W2-02 mitigated).

## Self-Check: PASSED

- `hp41-gui/src-tauri/src/commands.rs` — exists, 0 unimplemented!, 2 commands tests GREEN
- `hp41-gui/src-tauri/src/lib.rs` — exists, commands::dispatch_op + commands::get_state registered
- Commit `0d74627` — verified in git log
- Commit `9f038cc` — verified in git log
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — 9 passed; 0 failed
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` — exits 0
- `hp41-gui/src-tauri/gen/schemas/desktop-schema.json` — exists (113.3K)

---
*Phase: 14-ipc-layer*
*Completed: 2026-05-09*
