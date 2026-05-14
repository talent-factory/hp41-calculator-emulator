---
plan: 21-04
status: complete
date: 2026-05-14
phase: 21-flags-display-control-sound
requirements:
  - FN-SOUND-01
  - FN-SOUND-02
---

# Plan 21-04 — Sound: SUMMARY

## What landed

HP-41CV sound subsystem in `hp41-core`: 2 new ops (BEEP, TONE n) backed by a
new `event_buffer: Vec<String>` channel on `CalcState`. Both ops push structured
event lines into the buffer; the frontend (CLI in Phase 25, GUI in Phase 26)
drains the buffer and routes the events to audio output. **No direct I/O in
hp41-core** — the zero-I/O invariant from Phase 11 (`print_buffer`) is honored
and now actively asserted by an integration test.

### Code (`hp41-core`)

- **`state.rs`** — added `pub event_buffer: Vec<String>` with
  `#[serde(default, skip)]` (transient — mirrors `print_buffer` precedent
  verbatim). Initialized to `Vec::new()` in `CalcState::new()`.
- **`ops/sound.rs`** (NEW, ~50 LOC) —
  - `op_beep(state)` — push the literal `"BEEP"` to `event_buffer`,
    `apply_lift_effect(Neutral)`, Ok.
  - `op_tone(state, n)` — `if n > 9 return InvalidOp` (range guard runs
    BEFORE the push — Phase 11/12 atomicity); push `format!("TONE {n}")`;
    Neutral lift; Ok.
  - Inline `#[cfg(test)]` covers BEEP-push and TONE-out-of-range atomicity.
- **`ops/mod.rs`** — registered `pub mod sound;` (alphabetical, between
  `registers` and `stack_ops`); added 2 new `Op` variants (`Beep`,
  `Tone(u8)`); added 2 new dispatch arms.
- **`ops/program.rs`** — added 2 new `execute_op` arms forwarding to
  `super::sound::*`. (No `run_loop` arm needed — Beep and Tone are regular
  ops, not programming-control.)

### Display + tests

- **`hp41-cli/src/prgm_display.rs`** + **`hp41-gui/src-tauri/src/prgm_display.rs`** —
  2 byte-identical `op_display_name` arms: `BEEP` (nullary) and `TONE {n}`.
- **`hp41-core/tests/phase21_sound.rs`** (NEW, 8 integration tests):
  - `event_buffer` defaults to empty
  - v2.0 fixture loads with empty `event_buffer` (SC-5 spillover)
  - `event_buffer` skipped on serialize (`#[serde(skip)]`)
  - `Op::Beep` pushes `"BEEP"` exactly once
  - `Op::Beep` preserves stack X/Y/Z/T/LASTX
  - `Op::Tone(n)` parameterized for n ∈ {0, 5, 9} pushes `TONE n`
  - `Op::Tone(10)` returns `InvalidOp`; buffer stays empty (atomicity)
  - **Zero-I/O invariant sentinel**: grep walks `hp41-core/src/` for
    `println!|eprintln!`, filters comment-prefixed hits, and asserts no
    production code matches. Runs as a regular Rust test so it fires on
    every workspace test pass; CI catches accidental I/O introductions.

## Files touched

| File | Change |
|------|--------|
| `hp41-core/src/state.rs` | +8 lines (event_buffer field + init) |
| `hp41-core/src/ops/sound.rs` | NEW (~50 lines) |
| `hp41-core/src/ops/mod.rs` | +13 lines (mod, 2 variants, 2 dispatch arms) |
| `hp41-core/src/ops/program.rs` | +3 lines (2 execute_op arms) |
| `hp41-cli/src/prgm_display.rs` | +3 lines (2 display arms) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | +3 lines (2 display arms) |
| `hp41-core/tests/phase21_sound.rs` | NEW (8 integration tests, ~100 LOC) |

## Test results

- `just test-core --test phase21_sound` — **8 passed / 0 failed**
- `just test` (full workspace) — all 29 test suites pass
- `just lint` — clean
- `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` — clean
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — green

## Architectural notes for CLAUDE.md "v2.2 additions"

1. `event_buffer` mirrors `print_buffer` (transient `Vec<String>` with
   `#[serde(default, skip)]`). Frontend drains after each dispatch.
2. Event-line format is plain text: `BEEP` (literal) and `TONE n`
   (e.g. `TONE 5`). Future structured-events upgrade (JSON-tagged) is
   non-breaking because `event_buffer` is `#[serde(skip)]`.
3. **Flag 21 (printer enable) is intentionally NOT wired to gate
   PRX/PRA/PRSTK in this milestone.** Phase 11 behavior is unchanged —
   stored flag-21 bit is data only. Phase 27 backlog candidate.

## Followups

- **Plan 21-02** (next in the chain) builds on the flags from 21-01 plus
  the run_loop edits already made for Op::Prompt in 21-03. It adds the
  conditional flag-test family (FS? / FC? / FS?C / FC?C).
- **Phase 25** wires BEEP / TONE to `key_to_op`.
- **Phase 26** un-stubs the corresponding KEY_DEFS entries and plays
  WebAudio tones from the drained `event_buffer`.

## Status: Complete
