---
plan: 21-01
status: complete
date: 2026-05-14
phase: 21-flags-display-control-sound
requirements:
  - FN-FLAG-01
---

# Plan 21-01 — Flags Core: SUMMARY

## What landed

Foundational HP-41 flag storage subsystem in `hp41-core`, plus the Wave-0 prerequisites
(justfile recipe + backward-compat fixture) consumed by plans 21-02 / 21-03 / 21-04.

### Code (`hp41-core`)

- **`state.rs`** — added `pub flags: u64` field on `CalcState` with `#[serde(default)]`.
  Initialized to 0 in `CalcState::new()`. Mirrors the `last_key_code` precedent for
  backward-compatible deserialization.
- **`ops/flags.rs`** (NEW, 90 LOC) — bit helpers `flag_get` / `flag_set` / `flag_clear`
  (all `#[inline]`, defensive no-op for n > 55) plus op layer `op_sf` / `op_cf` mirroring
  the `op_sto` shape (range guard → mutate → `apply_lift_effect(Neutral)` → Ok).
  Inline `#[cfg(test)] mod tests` covers helper boundaries.
- **`ops/mod.rs`** — registered `pub mod flags;` (alphabetical between `cardreader_ops`
  and `hms`); added 2 new `Op` variants (`SfFlag(u8)`, `CfFlag(u8)`); added 2 new
  dispatch arms forwarding to `flags::op_sf` / `flags::op_cf`.
- **`ops/program.rs`** — added 2 new `execute_op` arms (Phase 21 Flags section) for
  programmable use. Closed the second compile-time gate of the 4-place rule.

### Display + tests

- **`hp41-cli/src/prgm_display.rs`** + **`hp41-gui/src-tauri/src/prgm_display.rs`** —
  added 2 byte-identical `op_display_name` arms: `SF {n:02}` / `CF {n:02}`.
- **`hp41-core/tests/phase21_flags.rs`** (NEW, 9 integration tests) — covers
  `flags` default zero, v2.0 fixture load (SC-5), serde round-trip, bit-helper
  boundaries, out-of-range defensiveness, `Op::SfFlag` / `Op::CfFlag` happy
  + error paths.

### Wave-0 prerequisites

- **`justfile`** — new `test-core *args:` recipe forwarding to
  `cargo test -p hp41-core {{args}}`. Enables `just test-core --test <name>`
  invocations used by plans 21-02 / 21-03 / 21-04.
- **`hp41-core/tests/fixtures/v20-autosave.json`** — hand-rolled v2.0-era CalcState
  serialization (omits `flags`, `display_override`, `event_buffer`). Used as the
  authoritative backward-compat probe by all four Phase 21 plans.

## Files touched

| File | Change |
|------|--------|
| `justfile` | +4 lines (test-core recipe) |
| `hp41-core/src/state.rs` | +9 lines (flags field + init) |
| `hp41-core/src/ops/flags.rs` | NEW (90 lines) |
| `hp41-core/src/ops/mod.rs` | +5 lines (mod, 2 variants, 2 dispatch arms) |
| `hp41-core/src/ops/program.rs` | +3 lines (2 execute_op arms) |
| `hp41-cli/src/prgm_display.rs` | +3 lines (2 display arms) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | +3 lines (2 display arms) |
| `hp41-core/tests/phase21_flags.rs` | NEW (9 integration tests) |
| `hp41-core/tests/fixtures/v20-autosave.json` | NEW (130 lines, hand-rolled) |

## Test results

- `just test-core --test phase21_flags` — **9 passed / 0 failed**
- `just test` (full workspace) — **all suites pass**
- `just lint` — clean
- `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` — clean
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — **51 passed across 4 suites**

## Followups

- **Plan 21-02** builds on the `flags: u64` field for the conditional-skip family
  (`FS?` / `FC?` / `FS?C` / `FC?C`).
- **Plans 21-03 and 21-04** consume the `v20-autosave.json` fixture for their own
  backward-compat tests (no need to recreate).
- **Phase 25** wires `SF` / `CF` to keyboard input via `key_to_op`.
- **Phase 26** wires `Op::SfFlag` / `Op::CfFlag` into `key_map::resolve` and the
  `KEY_DEFS` table, un-stubbing the SF / CF keys.
- Coverage gate deferred to phase-level verification (Task 6 in the orchestrator
  task list); Plan 21-01 baseline is non-regressing because all new code paths
  are exercised by the 9 integration tests.

## Status: Complete
