---
phase: 21
slug: flags-display-control-sound
status: passed
date: 2026-05-14
requirements:
  - FN-FLAG-01
  - FN-FLAG-02
  - FN-DISP-01
  - FN-DISP-02
  - FN-DISP-03
  - FN-DISP-04
  - FN-DISP-05
  - FN-SOUND-01
  - FN-SOUND-02
---

# Phase 21 — Verification

> Goal-backward verification of Phase 21 (Flags, Display Control & Sound).
> All 9 phase requirements implemented and tested.

## Phase Goal

Restore HP-41CV parity for flag storage + conditional skip, display control,
and sound — the engine-side foundation for Phase 25 (CLI integration) and
Phase 26 (GUI integration).

## Requirement → Implementation → Test mapping

| Requirement | Implementation | Test gate | Result |
|------------|----------------|-----------|--------|
| FN-FLAG-01 | `state.flags: u64` field + `ops::flags::{flag_get, flag_set, flag_clear, op_sf, op_cf}` | `phase21_flags::test_op_sf_sets_bit`, `test_op_cf_clears_bit`, `test_op_sf_out_of_range_returns_invalid_op`, `test_op_cf_out_of_range_returns_invalid_op` | ✅ |
| FN-FLAG-02 | `FlagTestKind` enum + `Op::FlagTest { kind, flag }` + run_loop arm | `phase21_flags::test_fs_q_*`, `test_fc_q_*`, `test_fs_q_c_*`, `test_fc_q_c_*` (8 tests) + `test_op_flag_test_interactive_dispatch_is_no_op` | ✅ |
| FN-DISP-01 | `Op::View(reg)` + `op_view` writes `format_hpnum(reg)` to `display_override` | `phase21_display::test_view_writes_register_to_override`, `test_view_preserves_stack`, `test_view_out_of_range` | ✅ |
| FN-DISP-02 | `Op::AView` + `op_aview` writes ALPHA-truncated-to-24 | `phase21_display::test_aview_writes_alpha_to_override` | ✅ |
| FN-DISP-03 | `Op::Prompt` + run_loop arm writes ALPHA + `break`s | `phase21_display::test_prompt_exits_run_loop`, `test_prompt_inside_program_returns_quickly` | ✅ |
| FN-DISP-04 | `Op::Aon` / `Op::Aoff` toggle system flag 48 | `phase21_display::test_aon_sets_flag_48`, `test_aoff_clears_flag_48` | ✅ |
| FN-DISP-05 | `Op::Cld` clears `display_override`; dispatch-top clear at the top of `dispatch()` | `phase21_display::test_cld_clears_only_override`, `test_dispatch_top_clears_stale_override` | ✅ |
| FN-SOUND-01 | `Op::Beep` + `op_beep` pushes `BEEP` to `event_buffer` | `phase21_sound::test_beep_pushes_event`, `test_beep_preserves_stack` | ✅ |
| FN-SOUND-02 | `Op::Tone(u8)` + `op_tone(n)` with `n > 9 → InvalidOp` guard | `phase21_sound::test_tone_n_pushes_event` (n=0,5,9), `test_tone_out_of_range` | ✅ |

## Must-have invariants (cross-cutting)

| Invariant | Check | Result |
|-----------|-------|--------|
| 4-place rule for every new Op variant | `just build` exits 0 (enum + dispatch + execute_op + BOTH prgm_display.rs copies; exhaustive matches confirmed at compile time) | ✅ |
| Zero-panic gate on `hp41-core` | `just lint` clean — `#![deny(clippy::unwrap_used)]` enforced; new modules use `?`-propagation or `.expect("...")` only | ✅ |
| Zero-I/O invariant on `hp41-core` | `phase21_sound::test_no_println_in_hp41_core_after_phase21` — greps `hp41-core/src/`, filters comment-prefixed hits, asserts no production matches | ✅ |
| SC-4 invariant | `grep -rnE 'fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)' hp41-gui/src-tauri/src/` returns 0 matches | ✅ |
| SC-5 backward compat | v2.0-era autosave fixture loads with `flags=0`, `display_override=None`, `event_buffer=[]` (via `#[serde(default)]` / `#[serde(default, skip)]` attributes) | ✅ |
| `display_override` lifecycle (Pitfall 5) | Cleared at top of `dispatch()` between `flush_entry_buf` and the prgm_mode gate; VIEW/AVIEW/PROMPT write AFTER the clear so their override survives their own dispatch | ✅ |
| FS?C / FC?C always-clear (RESEARCH A4) | run_loop arm matches strict reading: ALWAYS clear the flag regardless of test outcome; skip decision uses PRE-clear state | ✅ |
| Coverage non-regression vs. v2.1 / Phase 20 baseline (92.5%) | `just coverage` reports **92.68%** hp41-core lines (improvement) | ✅ |

## Test inventory

| File | Tests | Status |
|------|-------|--------|
| `hp41-core/tests/phase21_flags.rs` | **19** (9 from Plan 21-01 + 10 from Plan 21-02) | ✅ all pass |
| `hp41-core/tests/phase21_display.rs` | **13** | ✅ all pass |
| `hp41-core/tests/phase21_sound.rs` | **8** (incl. zero-I/O sentinel) | ✅ all pass |
| Inline `#[cfg(test)] mod tests` (flags.rs, display_ops.rs, sound.rs) | **8** | ✅ all pass |
| **Phase 21 new tests total** | **48** | ✅ |

## Quality gates summary

| Gate | Command | Result |
|------|---------|--------|
| Workspace tests | `just test` | ✅ 29 test suites, all passing (216 + others) |
| Lint | `just lint` | ✅ clean |
| Coverage (hp41-core) | `just coverage` | ✅ **92.68%** lines (≥ 80% default; ≥ 92.5% Phase 20 baseline non-regression) |
| Full CI | `just ci` | ✅ green |
| GUI build | `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` | ✅ clean |
| GUI tests | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | ✅ green |
| GUI CI | `just gui-ci` | ✅ green (TypeScript, Tauri build, GUI tests) |

## Architectural notes for CLAUDE.md "v2.2 additions"

1. **`flags: u64`** — packed flag word on CalcState. `#[serde(default)]` for
   v2.0 backward compat. Bit n = flag n; helpers `flag_get` / `flag_set` /
   `flag_clear` defensively no-op for n > 55.
2. **`display_override: Option<String>`** — transient (skipped on serialize).
   Cleared at the top of `dispatch()` between `flush_entry_buf` and the
   prgm_mode gate (Pitfall 5). VIEW/AVIEW/PROMPT/CLD write to this; next
   op's dispatch clears it.
3. **`event_buffer: Vec<String>`** — transient (skipped on serialize).
   BEEP and TONE n push structured event lines (plain text: `"BEEP"`,
   `"TONE n"`); the frontend drains and plays.
4. **AON/AOFF target system flag 48** (HP-42S compat, RESEARCH §Pattern 6).
   User-visible auto-display effect is a Phase 25/26 frontend concern.
5. **PROMPT exits `run_loop` via `break`**. Full STOP/resume is Phase 22
   territory (RESEARCH A5).
6. **`FlagTestKind`** — 4 variants, mirrors `TestKind` / `StoArithKind`.
   The `?C` variants ALWAYS clear the flag as a side effect (RESEARCH A4
   strict reading); skip decision uses the PRE-clear state.
7. **Op::Prompt** + **Op::FlagTest** are run_loop-only — listed in the
   `execute_op` catch-all so they explicitly return `InvalidOp` if reached
   there (Pitfall 2 defense-in-depth).
8. **Zero-I/O regression sentinel**: `hp41-core/tests/phase21_sound.rs`
   includes a Rust-level grep test that fires on every workspace test
   pass. Adding a stray `println!` / `eprintln!` to `hp41-core/src/`
   breaks CI.

## Human verification

No human-only test items — every Phase 21 behavior is asserted by an
automated test. Frontend rendering (TUI display panel reading
`display_override`, GUI WebAudio reading `event_buffer`) is Phase 25/26
scope and is verified there.

## Status: PASSED

All 9 requirements are implemented, tested, and verified at the engine
level. Coverage improved (92.65% → 92.68%). No regressions in any
pre-existing suite. The phase is ready to mark complete in ROADMAP.md.
