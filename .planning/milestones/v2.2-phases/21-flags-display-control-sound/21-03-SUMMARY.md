---
plan: 21-03
status: complete
date: 2026-05-14
phase: 21-flags-display-control-sound
requirements:
  - FN-DISP-01
  - FN-DISP-02
  - FN-DISP-03
  - FN-DISP-04
  - FN-DISP-05
---

# Plan 21-03 — Display Control: SUMMARY

## What landed

HP-41CV display-control subsystem in `hp41-core`: 6 new ops (VIEW, AVIEW,
PROMPT, AON, AOFF, CLD), a transient `display_override` channel, and the
dispatch-top clear that gives the "VIEW shows until next key" semantic.
PROMPT additionally pauses program execution by `break`ing out of `run_loop`.

### Code (`hp41-core`)

- **`state.rs`** — added `pub display_override: Option<String>` with
  `#[serde(default, skip)]` (transient, never persisted; mirrors `print_buffer`
  precedent). Initialized to `None` in `CalcState::new()`.
- **`ops/display_ops.rs`** (NEW, ~110 LOC) — 6 op functions
  (`op_view` / `op_aview` / `op_prompt` / `op_aon` / `op_aoff` / `op_cld`),
  all with `LiftEffect::Neutral`. `op_view` guards `reg >= 100` and uses
  `format_hpnum` for the display string. `op_aview` / `op_prompt` truncate
  ALPHA to 24 chars (mirrors `op_pra`). `op_aon` / `op_aoff` toggle system
  flag 48 via `flags::flag_set` / `flag_clear` (HP-42S compat, RESEARCH
  Pattern 6). `op_cld` explicit-clears `display_override`. Inline tests
  cover the 4 "interesting" behaviors.
- **`ops/mod.rs`** —
  - Registered `pub mod display_ops;` (alphabetical, between `cardreader_ops`
    and `flags`).
  - Added the dispatch-top clear: `state.display_override = None;` between
    `flush_entry_buf` and the prgm_mode gate (Pitfall 5). VIEW/AVIEW/PROMPT
    write AFTER this line so their override survives their own dispatch;
    the next op's dispatch clears it again.
  - Added 6 new `Op` variants: `View(u8)`, `AView`, `Prompt`, `Aon`,
    `Aoff`, `Cld`.
  - Added 6 new dispatch arms forwarding to the `display_ops` module.
- **`ops/program.rs`** —
  - Added a new `run_loop` arm for `Op::Prompt`: writes ALPHA-truncated-to-24
    to `display_override` then `break`s. Mirrors the top-level RTN exit
    semantic; `is_running` is reset by `run_program`. Full STOP/resume
    deferred to Phase 22 (RESEARCH A5).
  - Added 5 new `execute_op` arms for View/AView/Aon/Aoff/Cld using
    `super::display_ops::*` (matches the Plan 21-01 / 21-02 pattern).
  - Appended `Op::Prompt` to the catch-all "programming-only" block so
    `execute_op` explicitly returns `InvalidOp` if it ever sees Prompt
    (defense-in-depth against future regressions of the run_loop arm).

### Display + tests

- **`hp41-cli/src/prgm_display.rs`** + **`hp41-gui/src-tauri/src/prgm_display.rs`** —
  6 byte-identical `op_display_name` arms: `VIEW {r:02}`, `AVIEW`, `PROMPT`,
  `AON`, `AOFF`, `CLD`.
- **`hp41-core/tests/phase21_display.rs`** (NEW, 13 integration tests):
  - Field defaults / serde-skip / v2.0 backward-compat
  - VIEW writes register / preserves stack / out-of-range
  - AVIEW writes ALPHA
  - AON sets flag 48 / AOFF clears flag 48
  - CLD clears only `display_override` (alpha + stack untouched)
  - Dispatch-top clears stale override before next op (Pitfall 5)
  - PROMPT exits run_loop before unreachable `PushNum(99)`
  - PROMPT inside-program returns in < 100 ms (Pitfall 3 busy-wait sentinel)

## Files touched

| File | Change |
|------|--------|
| `hp41-core/src/state.rs` | +8 lines (display_override field + init) |
| `hp41-core/src/ops/display_ops.rs` | NEW (~110 lines) |
| `hp41-core/src/ops/mod.rs` | +25 lines (mod, 6 variants, 6 dispatch arms, dispatch-top clear) |
| `hp41-core/src/ops/program.rs` | +18 lines (run_loop arm, 5 execute_op arms, catch-all entry) |
| `hp41-cli/src/prgm_display.rs` | +7 lines (6 display arms) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | +7 lines (6 display arms) |
| `hp41-core/tests/phase21_display.rs` | NEW (13 integration tests, ~150 LOC) |

## Test results

- `just test-core --test phase21_display` — **13 passed / 0 failed**
- `just test` (full workspace) — **all suites pass**
- `just lint` — clean
- `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` — clean
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — green

## Architectural notes for CLAUDE.md "v2.2 additions"

1. `display_override` is core-managed: cleared at the top of `dispatch()` (Pitfall 5)
   and written by VIEW/AVIEW/PROMPT/CLD. Transient — `#[serde(default, skip)]`,
   never persisted.
2. AON/AOFF target system flag 48 (HP-42S compat). User-visible
   "ALPHA-auto-display" effect is a Phase 25/26 frontend concern; Phase 21
   only stores the bit.
3. PROMPT exits `run_loop` via `break` (writes ALPHA then yields control).
   Full STOP/resume from PROMPT is Phase 22 territory (RESEARCH A5).

## Followups

- **Plan 21-04** consumes the same v2.0 fixture for its `event_buffer`
  backward-compat test (already in place from Plan 21-01).
- **Phase 25** wires VIEW/AVIEW/PROMPT/AON/AOFF/CLD to `key_to_op`.
- **Phase 26** un-stubs the corresponding KEY_DEFS entries and renders
  `display_override` in the GUI LCD.
- The CLI rendering of `display_override` (Phase 25 `ui.rs`) is the next
  user-visible step.

## Status: Complete
