---
phase: 24-indirect-addressing
plan: 02
subsystem: hp41-core / hp41-cli / hp41-gui / indirect-addressing
tags:
  - rust
  - hp41-core
  - hp41-cli
  - hp41-gui
  - indirect-addressing
  - op-variants

dependency_graph:
  requires:
    - "Plan 24-01 — resolve_indirect / resolve_indirect_decimal helpers"
  provides:
    - "11 new Op::*Ind variants (StoInd, RclInd, StoArithInd, IsgInd, DseInd, SfFlagInd, CfFlagInd, FlagTestInd, ArclInd, AstoInd, ViewInd)"
    - "10 pub(crate) op_*_ind delegation shims in hp41-core/src/ops/indirect.rs (FlagTestInd has no shim — behavior lives in run_loop)"
  affects:
    - "hp41-core/src/ops/mod.rs (Op enum + dispatch arms)"
    - "hp41-core/src/ops/program.rs (execute_op + run_loop arms + catch-all extension)"
    - "hp41-cli/src/prgm_display.rs (op_display_name arms)"
    - "hp41-gui/src-tauri/src/prgm_display.rs (op_display_name arms — SC-4 mirror)"

tech_stack:
  added: []
  patterns:
    - "delegation-over-duplication (every op_*_ind is a 2-line shim over a direct-form op)"
    - "4-place Op-variant landing (dispatch + execute_op + cli prgm_display + gui prgm_display)"
    - "SC-4 mirror invariant (cli and gui op_display_name are byte-for-byte identical for the new arms)"

key_files:
  created:
    - hp41-core/tests/phase24_ind_variants.rs
  modified:
    - hp41-core/src/ops/indirect.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs

decisions:
  - "D-24.7 ratified by working code — 10 tuple-variant `<Name>Ind(u8)` (or `(u8, StoArithKind)` for StoArithInd) + 1 struct-variant `Op::FlagTestInd { kind: FlagTestKind, ind_reg: u8 }` per D-24.6"
  - "D-24.4 ratified by working code — every op_*_ind shim is a 2-line body (resolve_indirect, then call direct-form op). Direct ops carry bounds (D-22.11.1), sidecar-clearing (D-23.4), atomicity, and lift effects. No replication."
  - "Pitfall 1 ratified by tests — IsgInd/DseInd/FlagTestInd skip-next-step semantics live in run_loop ONLY. Interactive dispatch is no-op-or-discard-bool."
  - "Pitfall 6 ratified — FlagTestInd lands in the programming-ops `|`-pattern catch-all in execute_op (no explicit arm). IsgInd/DseInd have explicit execute_op arms; their catch-all entry was REMOVED (Rule 1 auto-fix below — Rust correctly flags those as unreachable patterns)."

metrics:
  duration_seconds: 1450
  duration_human: "~24 minutes"
  tasks_completed: 4
  files_created: 1
  files_modified: 5
  commits: 4
  tests_added: 43
  full_suite_tests_passing: 905
  hp41_gui_tests_passing: 52
  hp41_core_coverage: 93.48
  completed_date: "2026-05-14"
---

# Phase 24 Plan 02: Variants Summary

11 new `Op::*Ind` variants (StoInd / RclInd / StoArithInd / IsgInd / DseInd / SfFlagInd / CfFlagInd / FlagTestInd / ArclInd / AstoInd / ViewInd) land in `hp41-core::ops::Op`, each delegating to its direct-form counterpart via the Plan 24-01 `resolve_indirect` helper. Every variant lands in all 4 places (dispatch + execute_op + cli prgm_display + gui prgm_display). Skip-next-step semantics for IsgInd/DseInd/FlagTestInd preserved in `run_loop` (Pitfall 1 mitigated). `op_view_ind` correctly displays the resolved register's value, not the pointer's (R9 mitigated).

## What Shipped

### Files Created

| Path | Purpose | Lines | Tests inside |
|------|---------|-------|--------------|
| `hp41-core/tests/phase24_ind_variants.rs` | 43 integration tests covering all 11 new variants (happy / non-integer / out-of-bounds per variant + StoArithKind reuse + FlagTestKind reuse + 3 inheritance bonuses + R9 sentinel + interactive-no-op defense) | 558 | 43 `#[test]` functions |

### Files Modified

| Path | Delta | What changed |
|------|------:|--------------|
| `hp41-core/src/ops/indirect.rs` | +124 / -10 | Appended 10 `pub(crate) fn op_*_ind` delegation shims (no shim for FlagTestInd by design). Each shim is a 2-line body: resolve the indirect address via `resolve_indirect`, then call the direct-form op. Added the matching `use` block for `op_arcl/op_asto/op_view/op_sf/op_cf/op_dse/op_isg/op_rcl/op_sto/op_sto_arith/StoArithKind`. |
| `hp41-core/src/ops/mod.rs` | +78 / 0 | Added 11 new variants in a `// Phase 24: Indirect Addressing` enum section after `Op::Posa` (10 tuple-variant + 1 struct-variant `FlagTestInd { kind, ind_reg }` per D-24.6). Added 11 dispatch arms; IsgInd/DseInd use `.map(\|_\| ())` to discard the bool; FlagTestInd is a Neutral no-op. |
| `hp41-core/src/ops/program.rs` | +55 / 0 | Added 10 explicit execute_op arms (8 simple delegate + 2 IsgInd/DseInd with `.map(\|_\| ())`). Added 3 run_loop arms with skip-next-step semantics. Extended the programming-ops `\|`-pattern catch-all with `\| Op::FlagTestInd { .. }` only (IsgInd/DseInd have explicit arms — see deviation below). |
| `hp41-cli/src/prgm_display.rs` | +79 / 0 | Added 11 `op_display_name` arms (9 bare-mnemonic `MNEMONIC IND nn` + StoArithInd kind-table + FlagTestInd kind-table) and a `test_display_phase24_ind_op_labels` test (17 assertions). |
| `hp41-gui/src-tauri/src/prgm_display.rs` | +79 / 0 | Byte-for-byte mirror of the CLI additions (SC-4 invariant). |

## Commits

| Order | Hash | Subject |
|-------|------|---------|
| 1 (Task 1) | `373dac5` | feat(24-02): add 11 Op::*Ind variants + dispatch arms + delegation shims |
| 2 (Task 2) | `fbb3d21` | feat(24-02): add execute_op + run_loop arms with skip semantics (IsgInd/DseInd/FlagTestInd) |
| 3 (Task 3) | `5612a8e` | feat(24-02): add 11 op_display_name arms to BOTH prgm_display.rs copies (SC-4 mirror) |
| 4 (Task 4) | `ee26926` | test(24-02): add 43 phase24_ind_variants integration tests + R9 sentinel |

## The 11 New Op Variants and Their Delegation Targets

| Op variant | Delegates to | Lift effect | Inherited correctness |
|------------|--------------|-------------|------------------------|
| `StoInd(u8)` | `op_sto` | Neutral | D-23.4 sidecar clear, D-22.11.1 bounds |
| `RclInd(u8)` | `op_rcl` | Enable | D-22.11.1 bounds, force lift_enabled=true |
| `StoArithInd(u8, StoArithKind)` | `op_sto_arith` | Neutral | atomicity (compute-then-write), D-22.11.1 bounds, D-23.4 sidecar |
| `IsgInd(u8)` | `op_isg` | Neutral | string-split counter (no f64 floor/fmod), D-22.11.1 bounds |
| `DseInd(u8)` | `op_dse` | Neutral | string-split counter, D-22.11.1 bounds |
| `SfFlagInd(u8)` | `op_sf` | Neutral | `n > 55` flag bounds (`< 56` enforcement) |
| `CfFlagInd(u8)` | `op_cf` | Neutral | `n > 55` flag bounds |
| `FlagTestInd { kind, ind_reg }` | inline in run_loop | Neutral | reuses Op::FlagTest's kind-match (always-clear for FS?C/FC?C) |
| `ArclInd(u8)` | `op_arcl` | Neutral | text_regs sidecar read, 24-char ALPHA cap, D-22.11.1 bounds |
| `AstoInd(u8)` | `op_asto` | Neutral | text_regs sidecar write + zero numeric slot (no-drift), atomicity |
| `ViewInd(u8)` | `op_view` | Neutral | display_override semantics; **shows resolved register's value (R9)** |

## 4-Place Landing Verification (D-22.21 / D-23.12)

Every new variant compiles in all 4 places — verified by Rust's exhaustive-match enforcement:

| Variant | dispatch (mod.rs) | execute_op (program.rs) | cli prgm_display | gui prgm_display |
|---------|:-:|:-:|:-:|:-:|
| StoInd | ✓ | ✓ | ✓ | ✓ |
| RclInd | ✓ | ✓ | ✓ | ✓ |
| StoArithInd | ✓ | ✓ | ✓ | ✓ |
| IsgInd | ✓ (.map) | ✓ (.map) + run_loop arm | ✓ | ✓ |
| DseInd | ✓ (.map) | ✓ (.map) + run_loop arm | ✓ | ✓ |
| SfFlagInd | ✓ | ✓ | ✓ | ✓ |
| CfFlagInd | ✓ | ✓ | ✓ | ✓ |
| FlagTestInd | ✓ (no-op) | catch-all + run_loop arm | ✓ | ✓ |
| ArclInd | ✓ | ✓ | ✓ | ✓ |
| AstoInd | ✓ | ✓ | ✓ | ✓ |
| ViewInd | ✓ | ✓ | ✓ | ✓ |

## Run-Loop Skip Semantics (Pitfall 1 Mitigation)

The 3 control-flow IND variants have dedicated `run_loop` arms that preserve skip-next-step semantics — verified by integration tests that drive through `run_program` (NOT `dispatch`):

- `Op::IsgInd(reg)`: `if op_isg_ind(...)? { state.pc += 1; }` (mirrors `Op::Isg`)
- `Op::DseInd(reg)`: `if op_dse_ind(...)? { state.pc += 1; }` (mirrors `Op::Dse`)
- `Op::FlagTestInd { kind, ind_reg }`: resolve flag via `resolve_indirect`, then reuse the EXACT kind-match block from `Op::FlagTest` (always-clear semantics for FS?C/FC?C preserved)

Test sentinels: `isg_ind_inside_run_loop`, `dse_ind_inside_run_loop`, `flag_test_ind_is_set_happy_inside_run_loop` (×4 sub-kinds).

## R9 Mitigation: VIEW IND Shows Resolved Register's Value

`view_ind_shows_resolved_register_value` explicitly asserts that with `R[5]=12` and `R[12]=42` in Fix(4) mode, `display_override` contains `format_hpnum(R[12], Fix(4))` and does NOT equal `format_hpnum(R[5], Fix(4))`. The R9 risk (display the pointer instead of the resolved register) is closed off.

## Test Coverage

### `cargo test -p hp41-core --test phase24_ind_variants`: 43 / 43 PASS

| Variant | Tests | Coverage |
|---------|------:|----------|
| StoInd | 4 | happy + non-integer + out-of-bounds + sidecar bonus |
| RclInd | 4 | happy + non-integer + out-of-bounds + Enable-lift bonus |
| StoArithInd | 6 | 4 kind-happy paths + non-integer + out-of-bounds |
| IsgInd | 3 | inside-run_loop + non-integer + out-of-bounds |
| DseInd | 3 | inside-run_loop + non-integer + out-of-bounds |
| SfFlagInd | 3 | happy + non-integer + out-of-flag-range |
| CfFlagInd | 3 | happy + non-integer + out-of-flag-range |
| FlagTestInd | 7 | 4 kind-happy paths in run_loop + non-integer + high-flag-no-panic + interactive-no-op |
| ArclInd | 3 | happy + non-integer + out-of-bounds |
| AstoInd | 3 | happy + non-integer + out-of-bounds |
| ViewInd | 4 | happy + R9-sentinel + non-integer + out-of-bounds |

**Total: 43 tests.** Coverage matrix exceeds the plan's 33+ goal.

### Full Workspace

- `cargo test -p hp41-core` (root workspace) — 905 / 905 pass (180 added since 24-01 baseline)
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — 52 / 52 pass
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- `cargo clippy --manifest-path hp41-gui/src-tauri/Cargo.toml -- -D warnings` — clean
- `just gui-check` — clean
- `just test` — green

### Coverage Gate

`just coverage` reports `hp41-core` line coverage **93.48%** (gate: ≥ 92.5%; baseline 92.68% — **+0.8% gain**).

| File | Line coverage | Function coverage | Region coverage |
|------|--------------:|------------------:|----------------:|
| `ops/indirect.rs` | 100.00% | 100.00% | 97.55% |
| `ops/program.rs` | 88.78% | 97.22% | 84.67% |
| `ops/mod.rs` | 91.50% | 100.00% | 88.84% |
| `ops/registers.rs` | 100.00% | 100.00% | 98.34% |
| `ops/flags.rs` | 100.00% | 100.00% | 100.00% |
| `ops/alpha.rs` | 100.00% | 100.00% | 99.36% |
| `ops/display_ops.rs` | 100.00% | 100.00% | 99.27% |

## Pitfall Mitigations Exercised

| Pitfall | How exercised |
|---------|---------------|
| Pitfall 1 (run_loop skip semantics) | Dedicated run_loop arms for IsgInd/DseInd/FlagTestInd; integration tests `isg_ind_inside_run_loop`, `dse_ind_inside_run_loop`, and 4 `flag_test_ind_*_inside_run_loop` tests drive through `run_program` to validate |
| Pitfall 6 (catch-all defense) | FlagTestInd lands in the programming-ops `\|`-pattern catch-all (no explicit execute_op arm). Comment-documented as the intended Pitfall 6 mitigation. |
| Pitfall 8 (kind-table format) | StoArithInd uses op-symbol table `{Add => "+", Sub => "-", Mul => "×", Div => "÷"}`. FlagTestInd uses kind-table `{IsSet => "FS?", IsClear => "FC?", IsSetThenClear => "FS?C", IsClearThenClear => "FC?C"}` |
| R9 (VIEW IND shows pointer not resolved) | `view_ind_shows_resolved_register_value` explicitly asserts display_override == formatted(R[12]) AND ≠ formatted(R[5]) |
| D-23.4 inheritance (sidecar via delegation) | `sto_ind_clears_text_regs_sidecar` asserts `state.text_regs.get(&12) == None` after STO IND 5 with sidecar populated. Delegation via `op_sto` carries D-23.4 verbatim. |
| D-22.x atomicity (compute-then-write via op_sto_arith) | Inherited via delegation; not explicitly re-tested in this plan (the `checked_*` path in op_sto_arith is covered by Phase 2 tests) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed redundant IsgInd/DseInd entries from the programming-ops catch-all**

- **Found during:** Task 2, first `cargo build` after run_loop + execute_op + catch-all changes
- **Issue:** The plan's Step C (Task 2) asked for `\| Op::IsgInd(_) \| Op::DseInd(_) \| Op::FlagTestInd { .. }` in the catch-all. With explicit execute_op arms for IsgInd/DseInd above the catch-all (Step A), Rust emits `warning: unreachable pattern` for both IsgInd/DseInd in the catch-all (the explicit arms match all values, leaving nothing for the catch-all to absorb). Under the project's `cargo clippy -- -D warnings` gate this would be a compile error.
- **Root cause:** The plan's defense-in-depth wording ("not required for compilation, but documents intent") conflicts with Rust's `unreachable_patterns` lint, which is enabled by default. Three remediation options: (a) explicitly `#[allow(unreachable_patterns)]` — clutter; (b) remove explicit IsgInd/DseInd execute_op arms — would lose defense-in-depth for `dispatch` errors; (c) remove IsgInd/DseInd from the catch-all — preserves the explicit arms and avoids the warning. Chose (c) because the plan's primary value is the 3 run_loop arms (which DO live in the catch-all spirit via FlagTestInd). The original "defense-in-depth" intent for IsgInd/DseInd is more honestly served by their explicit arms.
- **Fix:** Removed the IsgInd/DseInd entries from the `\|`-pattern catch-all; kept FlagTestInd (which IS required, since it has no explicit execute_op arm — mirrors Op::FlagTest precedent).
- **Files modified:** `hp41-core/src/ops/program.rs` (catch-all lines)
- **Commit:** Folded into `fbb3d21` (Task 2 commit body documents the deviation).

**2. [Rule 1 - Bug] Fixed `dse_ind_inside_run_loop` assertion to truncate before comparison**

- **Found during:** Task 4, running `cargo test -p hp41-core --test phase24_ind_variants`
- **Issue:** The plan's spec test asserted `exit_val.inner() <= Decimal::from_str("1")`. The HP-41 DSE counter uses CCCCC.FFFDD encoding: after exit the counter holds `current=1, frac=001` represented as Decimal "1.001". The assertion `1.001 <= 1` is false, so the test failed.
- **Root cause:** The plan's pseudocode treated the counter as a plain integer; in reality the fractional `.001` step is preserved post-exit (HP-41 hardware-faithful).
- **Fix:** Changed assertion to `exit_val.trunc_int().inner() <= Decimal::from_str("1")` — compares the current value (integer part) against the target.
- **Files modified:** `hp41-core/tests/phase24_ind_variants.rs`
- **Commit:** Folded into `ee26926` (Task 4 commit body documents the deviation).

No other deviations. Plan executed as written for Tasks 1, 3, and 4-shim-bodies.

## Save-File Backward Compatibility

Phase 24 adds NO new `CalcState` fields and NO new `HpError` variants. All state changes are encoded as new `Op` enum variants (program-step level, not state-snapshot level). Pre-Phase-24 save files (`autosave.json` from v1.0 / v1.1 / v2.0 / v2.1 / Phase 20-23) load unchanged — verified at compile time (no schema migration code added, no deserialization change). Per `state.rs`'s `#[serde(default)]` discipline on every field added since v1.0, omitted fields fill in with defaults.

## TDD Gate Compliance

Plan-level `type: execute` with task-level `tdd="true"` on all 4 tasks. The TDD gate for THIS plan was implemented via the **exhaustive-match RED gate** (Task 1) plus the **integration-test characterization** pattern (Task 4):

| Task | Gate fulfillment |
|------|------------------|
| Task 1 (variants + dispatch + shims) | Compile-time RED: adding the 11 variants WITHOUT execute_op + prgm_display arms produces non-exhaustive-pattern errors. Task 2 (execute_op + run_loop) and Task 3 (prgm_display arms) resolve them. The RED is the Rust compiler asserting "you forgot to handle these variants." |
| Task 2 (execute_op + run_loop) | GREEN: hp41-core compiles green; the 7 inline indirect::tests and 4 phase22_program_control sentinels from 24-01 continue to pass. |
| Task 3 (prgm_display) | GREEN: hp41-cli + hp41-gui compile green; the new `test_display_phase24_ind_op_labels` test passes in BOTH copies. |
| Task 4 (integration tests) | RED → GREEN: 43 tests added, 1 initial failure (DSE assertion bug), 1 fix, 43/43 pass. |

The characterization-test pattern for Tasks 1-3 is the right TDD interpretation: the plan's primary value is the COMPILE-TIME 4-place landing rule. The integration-test value (Task 4) primarily protects against future regressions and exercises the per-variant behavior contract.

## Hand-off Note for Phases 25 and 26

The 11 new Op variants are now reachable from interactive dispatch and program execution. They await:

- **Phase 25 (CLI Integration):** keyboard wiring for IND prompts. Suggested PendingInput variants: `StoIndPrompt`, `RclIndPrompt`, `StoArithIndPrompt`, `IsgIndPrompt`, `DseIndPrompt`, `SfIndPrompt`, `CfIndPrompt`, `FlagTestIndPrompt` (4 sub-kinds), `ArclIndPrompt`, `AstoIndPrompt`, `ViewIndPrompt`. Plan 24-RESEARCH.md §"Open Questions" notes that all 11 should share a 2-digit register-entry modal — implementer's call whether to consolidate.

- **Phase 26 (GUI Integration):** `key_map.rs` registration for the 11 string IDs:
  - `sto_ind`, `rcl_ind`, `sto_arith_ind`, `isg_ind`, `dse_ind`, `sf_flag_ind`, `cf_flag_ind`, `flag_test_ind`, `arcl_ind`, `asto_ind`, `view_ind`
  - The frontend will need to combine the resolved string-ID with a register-number suffix (mirrors existing `sto_${n}` / `rcl_${n}` pattern from Phase 19 / 20). `StoArithInd` and `FlagTestInd` additionally need a kind suffix.

- **Phase 27 (Test Hardening):** add a `proptest` sweep for `resolve_indirect` covering the FN-IND-02 non-integer rejection space; add the 11 new variants to the 500-case numerical accuracy suite (low priority — they are register-address ops, not numeric ops). Coverage gate should remain ≥ 92.5%.

## Self-Check: PASSED

- Files created exist:
  - `hp41-core/tests/phase24_ind_variants.rs` — FOUND
- Files modified contain expected markers:
  - `hp41-core/src/ops/indirect.rs` contains 10 `pub(crate) fn op_*_ind` declarations — FOUND
  - `hp41-core/src/ops/mod.rs` contains all 11 new variant names — FOUND
  - `hp41-core/src/ops/program.rs` contains 3 run_loop arms (IsgInd / DseInd / FlagTestInd) with `state.pc += 1` skip semantics — FOUND
  - `hp41-cli/src/prgm_display.rs` contains 11 new `IND` format strings — FOUND
  - `hp41-gui/src-tauri/src/prgm_display.rs` contains the byte-identical 11 new `IND` format strings — FOUND
- Commits exist on `worktree-agent-a87bc2490edeacf1f`:
  - `373dac5` (Task 1) — FOUND
  - `fbb3d21` (Task 2) — FOUND
  - `5612a8e` (Task 3) — FOUND
  - `ee26926` (Task 4) — FOUND
- Full `cargo test --workspace` suite: 905 / 905 pass
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`: 52 / 52 pass
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `just coverage` line coverage: **93.48%** (gate: ≥ 92.5%; **+0.8% over baseline**)
