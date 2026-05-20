---
plan: 21-02
status: complete
date: 2026-05-14
phase: 21-flags-display-control-sound
requirements:
  - FN-FLAG-02
---

# Plan 21-02 — Conditional Flag Tests: SUMMARY

## What landed

HP-41CV conditional flag-test family (FN-FLAG-02) — the four ops `FS?` /
`FC?` / `FS?C` / `FC?C` — as a single struct-variant `Op::FlagTest { kind:
FlagTestKind, flag: u8 }`. Inside `run_loop`, the variant performs the
HP-41 "skip-next-step on false" semantic plus the always-clear side
effect for the `?C` variants (RESEARCH A4 strict reading). Interactive
dispatch is a no-op — mirrors the `Op::Test` precedent, because at the
keyboard there is no "next program step" to skip.

### Code (`hp41-core`)

- **`ops/mod.rs`** —
  - New `FlagTestKind` enum (4 variants: `IsSet`, `IsClear`,
    `IsSetThenClear`, `IsClearThenClear`) with the standard
    `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` chain
    used by the other sub-enums.
  - New `Op::FlagTest { kind: FlagTestKind, flag: u8 }` struct variant
    in the Phase 21 section (after SfFlag/CfFlag).
  - New interactive dispatch arm: `Op::FlagTest { .. } => { apply_lift_effect(Neutral); Ok(()) }`
    (no PC advance, no flag mutation; pure no-op at the keyboard).
- **`ops/program.rs`** —
  - New run_loop arm `Op::FlagTest { kind, flag }`:
    - reads `is_set = flag_get(state.flags, flag)` once;
    - matches on `kind` to compute `should_skip` and, for the two `?C`
      variants, ALWAYS clears the flag (regardless of test outcome —
      RESEARCH A4 strict reading);
    - `if should_skip { state.pc += 1 }` (mirrors `Op::Test` skip-next-step).
  - Extended the execute_op catch-all to include `Op::FlagTest { .. }`
    so any accidental reach there returns `InvalidOp` (Pitfall 2
    defense-in-depth — FlagTest belongs only in run_loop).

### Display + tests

- **`hp41-cli/src/prgm_display.rs`** + **`hp41-gui/src-tauri/src/prgm_display.rs`** —
  imported `FlagTestKind`; added 1 byte-identical `op_display_name` arm
  that emits the mnemonic-plus-flag form: `FS? 05`, `FC? 12`, `FS?C 10`,
  `FC?C 00`.
- **`hp41-core/tests/phase21_flags.rs`** — extended with 10 new tests for
  conditional-skip semantics (total file: **19 tests**):
  - `test_fs_q_in_program_executes_next_when_flag_set` — FS? on SET: no skip
  - `test_fs_q_in_program_skips_next_when_flag_clear` — FS? on CLEAR: skip
  - `test_fc_q_in_program_skips_when_flag_set` — FC? on SET: skip
  - `test_fc_q_in_program_executes_when_flag_clear` — FC? on CLEAR: no skip
  - `test_fs_q_c_clears_flag_after_test` — FS?C on SET clears the flag
  - `test_fs_q_c_on_clear_flag_idempotent` — FS?C on CLEAR keeps it clear
  - `test_fc_q_c_clears_flag_after_test` — FC?C on SET clears the flag
  - `test_fs_q_c_skip_branch` — FS?C on CLEAR skips
  - `test_fc_q_c_skip_branch` — FC?C on SET skips and clears
  - `test_op_flag_test_interactive_dispatch_is_no_op` — interactive: no PC
    advance, no flag mutation, no stack change

  All program-mode tests construct `state.program = vec![Op::Lbl("T"), …]`
  and call `run_program(&mut s, "T")`; final state is then asserted on the
  stack and on `flag_get(s.flags, …)`.

## Files touched

| File | Change |
|------|--------|
| `hp41-core/src/ops/mod.rs` | +20 lines (FlagTestKind enum, Op::FlagTest variant, interactive dispatch arm) |
| `hp41-core/src/ops/program.rs` | +24 lines (run_loop arm with 4-kind match + catch-all entry) |
| `hp41-cli/src/prgm_display.rs` | +10 lines (FlagTestKind import + 1 display arm) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | +10 lines (same) |
| `hp41-core/tests/phase21_flags.rs` | +186 lines (10 new tests, helper, imports update) |

## Test results

- `just test-core --test phase21_flags` — **19 passed / 0 failed**
  (9 from Plan 21-01 + 10 from this plan)
- `just test` (full workspace) — all 29 test suites pass
- `just lint` — clean
- `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml` — clean
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — green

## Architectural notes for CLAUDE.md "v2.2 additions"

- **RESEARCH A4 honored**: FS?C / FC?C always clear the target flag as a
  side effect, regardless of test outcome. The pre-clear state of the
  flag determines the skip; for an already-clear flag the always-clear
  is a no-op on the bit (idempotent).
- **Interactive FlagTest is a no-op** (mirrors `Op::Test` interactive
  behavior). At the keyboard there is no "next program step" to skip;
  PC and flags are untouched.
- **Op::FlagTest is run_loop-only** — listed in the execute_op catch-all
  so any direct reach returns InvalidOp (Pitfall 2 defense-in-depth).
- The struct-variant shape `Op::FlagTest { kind, flag }` matches the
  `Op::StoArith { reg, kind }` precedent and reads cleanly in tests.

## Followups

- **Phase 22 (Program Control & Memory Ops)** can build on the
  conditional-skip foundation here for any new flag-driven control flow.
- **Phase 24 (Indirect Addressing)** can route `Op::FlagTest { flag: <ind> }`
  through its indirect resolver once that subsystem lands.
- **Phase 25** wires FS?/FC?/FS?C/FC?C to keyboard input via `key_to_op`.
- **Phase 26** un-stubs the corresponding KEY_DEFS entries in the GUI.

## Status: Complete
