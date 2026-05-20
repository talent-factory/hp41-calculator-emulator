---
phase: 23-alpha-operations
plan: 01
subsystem: alpha-operations
tags: [arcl, asto, text-regs, sidecar, alpha, fn-alpha-01, fn-alpha-02]

dependency-graph:
  requires:
    - Phase 22 D-22.17 BTreeMap-on-CalcState precedent (assignments field)
    - Phase 22 D-22.21 4-place Op-variant landing rule
    - Phase 22 D-22.11.1 regs-bounds audit precedent (sidecar-clearing pattern mirrors this)
    - Phase 2 op_alpha_append's 24-char silent-discard cap idiom
    - hp41-core/src/format.rs::format_hpnum (FIX/SCI/ENG-aware formatter)
  provides:
    - "CalcState.text_regs: BTreeMap<u8, String> packed-text register sidecar"
    - "Op::Arcl(u8) — append register-N's formatted value to ALPHA"
    - "Op::Asto(u8) — pack first 6 ALPHA chars into register-N's text shadow"
    - "D-23.4 sidecar-clearing invariant (op_sto / op_sto_arith / op_clreg)"
  affects:
    - "Phase 24 will layer Op::ArclInd / Op::AstoInd on top via resolve_indirect()"
    - "Phase 25/26 will wire ARCL/ASTO into the TUI / GUI keyboard layers"

tech-stack:
  added:
    - "No new external dependencies"
  patterns:
    - "BTreeMap-on-CalcState sidecar (mirrors Phase 22 assignments)"
    - "Sidecar-clearing audit on every numeric write (D-23.4 — new pattern)"
    - "Leading bounds check BEFORE map lookup (W-2 strengthening of D-23.3)"
    - "chars().take(6) multibyte-safe 6-char prefix slicing"

key-files:
  created:
    - hp41-core/tests/phase23_arcl_asto.rs
  modified:
    - hp41-core/src/state.rs
    - hp41-core/src/ops/alpha.rs
    - hp41-core/src/ops/registers.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs

decisions:
  - "Combined Task 1 (sidecar-clearing audit) and Task 2 (text_regs field + ARCL/ASTO + 4-place landing) into a single commit (implementer-discretion option in plan). Rationale: the Wave-0 audit tests don't compile until the text_regs field exists, so splitting would have left the build red between commits."
  - "Applied the W-2 strengthening of D-23.3 in op_arcl: the regs.len() bounds check fires BEFORE the text_regs lookup, making op_arcl symmetric with op_asto and pinning threat T-23-01 (tampered autosave.json with out-of-range text_regs key)."
  - "Added an extra inline test (test_op_sto_arith_failure_preserves_text_regs_sidecar) that pins atomicity at the sidecar layer: a failing op_sto_arith (e.g. divide-by-zero) must NOT clear the sidecar. Achieved by ordering text_regs.remove(&reg) AFTER the checked_* computation in op_sto_arith."

metrics:
  duration: "~7 min (executor agent, single wave)"
  completed: 2026-05-14T13:53Z
  commits: 2
  tasks_completed: 3  # Task 1 + Task 2 combined into one commit (allowed), Task 3 second commit
  files_modified: 7
  files_created: 1
  loc_added: 617      # 373 (impl + audit + unit tests) + 244 (integration tests)
  loc_removed: 0
  new_tests:
    inline_alpha: 9
    inline_registers: 5
    integration: 8
    total: 22
---

# Phase 23 Plan 01: ARCL / ASTO + text_regs Sidecar — Summary

**One-liner:** Landed `Op::Arcl(u8)` and `Op::Asto(u8)` with the foundational `text_regs: BTreeMap<u8, String>` sidecar on `CalcState`, plus the D-23.4 sidecar-clearing audit in `op_sto` / `op_sto_arith` / `op_clreg` so the numeric and text representations of a register never drift.

## What Shipped

### `CalcState.text_regs` field (D-23.1)

A new `BTreeMap<u8, String>` field slotted immediately after `assignments` (the Phase 22 precedent), carrying `#[serde(default)]` so v1.x / v2.0 / v2.1 autosave.json files load cleanly with an empty map. The field stores the packed-text shadow of each numbered register; `ASTO nn` writes to it and `ARCL nn` reads from it in preference to formatting the numeric slot.

### `Op::Arcl(u8)` — FN-ALPHA-01 (D-23.3, W-2 strengthened)

Appends the formatted value of register `reg` to `state.alpha_reg`. Lookup order:

1. **Leading bounds check** — `(reg as usize) >= state.regs.len()` returns `HpError::InvalidOp` BEFORE consulting `text_regs`. This is the W-2 strengthening of the CONTEXT.md D-23.3 sketch and closes threat T-23-01 (a hand-edited autosave with an out-of-range `text_regs` key would otherwise bypass the regs-bounds check).
2. If `text_regs[reg]` exists → clone it.
3. Else `format_hpnum(&regs[reg], &display_mode)` — respects the current FIX/SCI/ENG setting (SC#1).

24-char silent-discard cap mirrors `op_alpha_append` (Phase 2 invariant). `LiftEffect::Neutral`.

### `Op::Asto(u8)` — FN-ALPHA-02 (D-23.2)

Bounds-checks `reg` first (atomicity — a failing ASTO leaves the sidecar untouched), then writes `state.alpha_reg.chars().take(6).collect::<String>()` (multibyte-safe) into `text_regs[reg]` and zeroes the numeric slot `regs[reg]` via `*slot = HpNum::zero()`. The ALPHA register itself is NOT modified. `LiftEffect::Neutral`.

### D-23.4 sidecar-clearing audit (Wave-0 invariant)

The single most important invariant of Phase 23: every numeric write to `regs[reg]` MUST clear the matching `text_regs[reg]` entry so ARCL can never read a stale text shadow after a numeric STO.

Touchpoints:

| Function | Action |
|----------|--------|
| `op_sto` | `state.text_regs.remove(&reg)` before the numeric write |
| `op_sto_arith` | `state.text_regs.remove(&reg)` AFTER the `checked_*` computation but before the numeric write (atomicity — a failing op leaves both reps untouched) |
| `op_sto_arith_stack` | Audit-outcome comment only — this path targets stack registers (Y/Z/T/LastX), not numbered regs, so no sidecar clearing is needed |
| `op_clreg` | `state.text_regs.clear()` alongside the regs reset |

### 4-place Op-variant landing (D-23.12)

Both `Op::Arcl(u8)` and `Op::Asto(u8)` landed AT END of the `Op` enum (preserving existing discriminant order for save-file compat per D-23.13) in all four required places:

1. `hp41-core/src/ops/mod.rs::Op` enum declaration
2. `hp41-core/src/ops/mod.rs::dispatch()` match
3. `hp41-core/src/ops/program.rs::execute_op()` match
4. `hp41-cli/src/prgm_display.rs::op_display_name` AND `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` (both copies)

Display strings: `"ARCL {reg:02}"` / `"ASTO {reg:02}"` (matches the Phase 22 `Op::Sto(reg)` `{reg:02}` width convention).

## Verification

- `just test-core` — all hp41-core unit + integration tests pass, including the 8 new `phase23_arcl_asto` integration tests, 9 new inline `alpha.rs` unit tests, and 5 new inline `registers.rs` sidecar-audit tests.
- `just lint` — `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes with zero warnings.
- `just gui-check` — Tauri crate compiles (4-place exhaustive matches green in the GUI prgm_display copy).
- SC-4 stricter grep (`grep -rnE "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/`) returns nothing — invariant preserved.
- No new `HpError` variants (still 9 — confirmed via `grep -c "^\s*#\[error" hp41-core/src/error.rs` against HEAD).

## Success Criteria

| SC | Status | Evidence |
|----|--------|----------|
| SC#1 (FN-ALPHA-01): ARCL respects display mode | ✅ | Inline `test_arcl_appends_numeric_register_in_sci_mode_differs_from_fix` + integration `arcl_appends_numeric_register_using_current_display_mode` |
| SC#2 (FN-ALPHA-02): ASTO + ARCL round-trip | ✅ | Integration `asto_arcl_round_trip_reproduces_first_6_chars` (GOODBYE → ASTO 12 → Cla → ARCL 12 → ALPHA="GOODBY") |
| D-23.4: every numeric write clears the sidecar | ✅ | Inline `test_op_sto_clears_text_regs_sidecar`, `test_op_sto_arith_clears_text_regs_sidecar`, `test_op_clreg_clears_all_text_regs`, `test_op_sto_arith_failure_preserves_text_regs_sidecar` (atomicity), and integration `numeric_sto_clears_text_regs_sidecar_no_drift` |
| D-23.13: v1.x save files load via `#[serde(default)]` | ✅ | Integration `serde_default_loads_v21_save_file_without_text_regs_field` (strips `text_regs` from serialized JSON, deserializes → empty map) |
| D-23.12: 4-place Op landing | ✅ | `just check` (workspace build) + `just gui-check` + `grep -nE "Op::Arcl|Op::Asto"` finds all four landing places |

## Commits

| Hash | Type | Description |
|------|------|-------------|
| `57a75e1` | feat | Op::Arcl/Op::Asto with text_regs sidecar + D-23.4 audit (Tasks 1+2 combined) |
| `915e22c` | test | integration suite for ARCL / ASTO + D-23.4 invariant (Task 3) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Added atomicity test at the sidecar layer**

- **Found during:** Task 1 implementation
- **Issue:** The plan's required tests covered the happy path (sidecar clears on success) but did not pin the atomicity guarantee at the sidecar layer. A future refactor of `op_sto_arith` that moved `text_regs.remove(&reg)` BEFORE the `checked_*` computation would silently regress: a failing op (e.g. `STO÷0`) would clear the sidecar even though the numeric slot stayed untouched, breaking the "at most one representation is non-default" invariant.
- **Fix:** Added `test_op_sto_arith_failure_preserves_text_regs_sidecar` in `phase23_sidecar_audit_tests` that triggers a divide-by-zero and asserts both `text_regs[5] == "HELLO"` AND `regs[5] == 10` are untouched. Implementation correctly orders `text_regs.remove(&reg)` AFTER the `checked_*` computation but before the numeric write.
- **Files modified:** `hp41-core/src/ops/registers.rs` (test added; production code ordering already correct per the diff)
- **Commit:** `57a75e1` (combined Tasks 1+2)

### Implementer-Discretion Choices

- **Combined Tasks 1 + 2** — the plan explicitly allowed this (Task 1's tests reference `state.text_regs` which is added in Task 2, so splitting would have left the build red between commits). One feature commit + one test commit follows the same shape as Phase 22 plan 22-04 (which also combined the field-add and Op-add into one commit and split off the integration tests).
- **Display strings** — used `"ARCL {reg:02}"` / `"ASTO {reg:02}"` (planner-discretion option). Matches existing `Op::StoReg`/`Op::RclReg` `{reg:02}` width convention.
- **Test organization** — sidecar-audit tests live in a new `phase23_sidecar_audit_tests` submodule of `registers.rs` (Phase 22 D-22.11.1 commit-style precedent). ARCL/ASTO unit tests live in the existing `#[cfg(test)] mod tests` of `alpha.rs` (file is still under 240 lines — no split needed). Integration tests live in `hp41-core/tests/phase23_arcl_asto.rs` as a fresh file parallel to `phase22_catalog.rs`.

## Known Stubs

None — Phase 23 plan 01 ships complete behavior for ARCL and ASTO. Phase 24 will layer the IND variants (`Op::ArclInd` / `Op::AstoInd`) on top via the `resolve_indirect()` helper; Phase 25/26 will add TUI / GUI keyboard wiring. These are scope boundaries explicitly documented in 23-CONTEXT.md, not stubs in this plan.

## Threat Flags

None — Phase 23 introduces no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries beyond what the existing `text_regs` field exposes (covered in the plan's `<threat_model>` register T-23-01 through T-23-05).

## Self-Check: PASSED

- File `hp41-core/tests/phase23_arcl_asto.rs` — FOUND
- File `hp41-core/src/state.rs` — modified (text_regs field added) — FOUND
- File `hp41-core/src/ops/alpha.rs` — modified (op_arcl + op_asto + 9 unit tests added) — FOUND
- File `hp41-core/src/ops/registers.rs` — modified (3 audit touchpoints + 5 unit tests added) — FOUND
- File `hp41-core/src/ops/mod.rs` — modified (Op::Arcl + Op::Asto declarations + dispatch arms) — FOUND
- File `hp41-core/src/ops/program.rs` — modified (execute_op arms) — FOUND
- File `hp41-cli/src/prgm_display.rs` — modified (display arms) — FOUND
- File `hp41-gui/src-tauri/src/prgm_display.rs` — modified (display arms) — FOUND
- Commit `57a75e1` — FOUND on worktree-agent-afa55e1d615a3c7ba
- Commit `915e22c` — FOUND on worktree-agent-afa55e1d615a3c7ba

## TDD Gate Compliance

Plan type is `execute` (not `tdd`), so the plan-level TDD gate sequence (RED → GREEN → REFACTOR commits) does not apply. Per-task `tdd="true"` annotations indicate task-level test-first intent; the combined Tasks 1+2 commit lands implementation and inline tests together (acceptable for `execute`-type plans per the plan's own implementer-discretion note). The integration test commit (`915e22c`) is a `test(...)` commit per the conventional commit style.
