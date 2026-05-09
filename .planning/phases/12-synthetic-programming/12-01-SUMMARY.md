---
phase: 12-synthetic-programming
plan: "01"
subsystem: hp41-core + hp41-cli/prgm_display
tags: [rust, hp41-core, synthetic-programming, getkey, hidden-registers, synthetic-byte, tdd, green]

# Dependency graph
requires:
  - phase: 12-synthetic-programming
    plan: "00"
    provides: "Wave 0 RED test scaffold in synthetic_tests.rs (21 tests)"
provides:
  - "4 new CalcState fields: last_key_code, reg_m, reg_n, reg_o with #[serde(default)]"
  - "9 new Op variants: GetKey, Null, StoM/N/O, RclM/N/O, SyntheticByte(u8)"
  - "dispatch() arms for all 9 new Op variants in hp41-core/src/ops/mod.rs"
  - "execute_op() arms for all 9 new Op variants in hp41-core/src/ops/program.rs"
  - "7 new register functions: op_getkey, op_sto_m/n/o, op_rcl_m/n/o in registers.rs"
  - "synthetic_byte_to_op() with 23-entry conservative safe subset"
  - "op_display_name() arms for all 9 new Op variants in prgm_display.rs"
  - "Wave 0 RED tests (21) now GREEN"
affects:
  - 12-02-synthetic-cli

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Phase 12 CalcState field pattern: #[serde(default)] on u8 and HpNum fields after print_buffer"
    - "Hidden register ops mirror op_sto/op_rcl exactly: lift_enabled=true before enter_number for RCL variants"
    - "synthetic_byte_to_op() conservative safe subset: 23 entries, never returns Some(Op::SyntheticByte(_))"
    - "execute_op() Phase 12 arms placed BEFORE the programming-ops catch-all (critical trap avoided)"

key-files:
  created:
    - hp41-core/tests/synthetic_tests.rs
  modified:
    - hp41-core/src/state.rs
    - hp41-core/src/ops/registers.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs

key-decisions:
  - "HpNum import added to registers.rs (was missing — needed for op_getkey HpNum::from conversion)"
  - "synthetic_tests.rs copied to worktree from main repo (worktree predates Wave 0 commit)"
  - "Lint fix applied to synthetic_tests.rs: &CalcState::new() → CalcState::new() in to_value() call"
  - "conservative 23-entry safe subset in synthetic_byte_to_op() includes arithmetic, stack, math, trig, synthetic primitives, and hidden register codes"

# Metrics
duration: 4min
completed: 2026-05-09
---

# Phase 12 Plan 01: Synthetic Programming Core Implementation Summary

**9 new Op variants, 4 CalcState fields, 7 register functions, synthetic_byte_to_op() safe subset, and prgm_display arms — all 21 Wave 0 RED tests now GREEN**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-09T06:59:47Z
- **Completed:** 2026-05-09T07:04:23Z
- **Tasks:** 2 of 2
- **Files modified:** 5 modified, 1 created

## Accomplishments

### Task 1: CalcState fields, Op variants, registers.rs functions, dispatch arms, and synthetic_byte_to_op

- Added 4 new `CalcState` fields with `#[serde(default)]`:
  - `last_key_code: u8` (default 0) — tracks last HP-41 hardware key code pressed
  - `reg_m: HpNum`, `reg_n: HpNum`, `reg_o: HpNum` (default HpNum::zero()) — hidden registers
- Added matching initializers in `CalcState::new()`
- Added 7 new functions to `hp41-core/src/ops/registers.rs`:
  - `op_getkey()` — pushes last_key_code to X (LiftEffect::Enable)
  - `op_sto_m/n/o()` — stores X to hidden registers (LiftEffect::Neutral)
  - `op_rcl_m/n/o()` — recalls hidden registers to X (LiftEffect::Enable, forces lift_enabled=true)
- Added 9 new `Op` variants to the enum in `ops/mod.rs`: `GetKey`, `Null`, `StoM/N/O`, `RclM/N/O`, `SyntheticByte(u8)`
- Updated `use registers::{...}` import to include all 7 new helper functions
- Added 9 `dispatch()` arms for all new Phase 12 ops
- Added `pub fn synthetic_byte_to_op(byte: u8) -> Option<Op>` with a 23-entry conservative safe subset covering arithmetic, stack, math, trig, synthetic primitives (0xCF→Null, 0xCE→GetKey), and hidden register codes

### Task 2: execute_op arms, prgm_display arms, Wave 0 tests GREEN

- Added 9 `execute_op()` arms in `program.rs` for Phase 12 ops — critically placed BEFORE the `Op::Lbl(_) | Op::Gto(_) | ...` programming-ops catch-all to avoid silent InvalidOp errors
- Added 9 `op_display_name()` arms in `prgm_display.rs`: `"GETKEY"`, `"NULL"`, `"STO M/N/O"`, `"RCL M/N/O"`, `format!("SYN {:02X}", b)` for uppercase hex zero-padded display
- Copied `synthetic_tests.rs` from main repo to worktree (worktree was created before Wave 0 ran)
- Applied lint fix: `&CalcState::new()` → `CalcState::new()` in `serde_json::to_value()` call

## Task Commits

1. **Task 1** — `f5bbcfb`: `feat(12-01): add CalcState fields, Op variants, registers functions, and synthetic_byte_to_op`
2. **Task 2** — `24757e7`: `feat(12-01): add execute_op arms, prgm_display arms, and turn Wave 0 RED tests GREEN`

## Files Created/Modified

- `hp41-core/src/state.rs` — 4 new `#[serde(default)]` fields + new() initializers
- `hp41-core/src/ops/registers.rs` — 7 new pub functions (op_getkey, op_sto_m/n/o, op_rcl_m/n/o); added `use crate::num::HpNum` import
- `hp41-core/src/ops/mod.rs` — 9 new Op variants, 9 new dispatch() arms, synthetic_byte_to_op(), updated use statement
- `hp41-core/src/ops/program.rs` — 9 new execute_op() arms before the programming-ops catch-all
- `hp41-cli/src/prgm_display.rs` — 9 new op_display_name() arms
- `hp41-core/tests/synthetic_tests.rs` — copied from main repo + lint fix

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing `HpNum` import in registers.rs**
- **Found during:** Task 1 — first build attempt
- **Issue:** `op_getkey()` uses `HpNum::from()` but `crate::num::HpNum` was not imported in registers.rs
- **Fix:** Added `use crate::num::HpNum;` to the imports at the top of registers.rs
- **Files modified:** `hp41-core/src/ops/registers.rs`
- **Commit:** f5bbcfb (included in Task 1 commit)

**2. [Rule 3 - Blocking] synthetic_tests.rs missing from worktree**
- **Found during:** Task 2 verification — cargo test could not find the `synthetic_tests` test target
- **Issue:** Worktree was created before Wave 0 committed `synthetic_tests.rs`. The test file existed only in the main repo, not in this worktree's file tree.
- **Fix:** Copied `hp41-core/tests/synthetic_tests.rs` from the main repo to the worktree
- **Files modified:** `hp41-core/tests/synthetic_tests.rs` (created in worktree)
- **Commit:** 24757e7

**3. [Rule 1 - Bug] Clippy warning in synthetic_tests.rs**
- **Found during:** `just lint` run after Task 2
- **Issue:** `serde_json::to_value(&CalcState::new())` — needless borrow for generic args (clippy::needless_borrows_for_generic_args). With `-D warnings` active, this fails lint.
- **Fix:** Changed to `serde_json::to_value(CalcState::new())` (by-value)
- **Files modified:** `hp41-core/tests/synthetic_tests.rs`
- **Commit:** 24757e7

## Known Stubs

None — all 9 Op variants have full implementations. No placeholder or TODO code in any production path.

## Threat Flags

No new security-relevant surface beyond what the plan's threat model covers. `synthetic_byte_to_op()` rejects all codes outside the curated 23-entry subset as required by T-12-W1-01.

## Verification Results

| Check | Result |
|-------|--------|
| `just build` exits 0 | PASS |
| `just lint` exits 0 | PASS |
| `cargo test -p hp41-core --test synthetic_tests` — 21 tests | PASS |
| `just test` — all suites | PASS |
| `grep -c "hp41-cli" hp41-core/Cargo.toml` returns 0 | PASS |
| New Op variants NOT in execute_op catch-all | PASS |
| `synthetic_byte_to_op(0xCF)` returns `Some(Op::Null)` | PASS |
| `synthetic_byte_to_op(0x00)` returns `None` | PASS |

## Self-Check

- [x] `hp41-core/src/state.rs` has 4 new `#[serde(default)]` fields (`last_key_code`, `reg_m`, `reg_n`, `reg_o`)
- [x] `hp41-core/src/state.rs` CalcState::new() initializes `last_key_code: 0` and reg_m/n/o to HpNum::zero()
- [x] `hp41-core/src/ops/registers.rs` exports `op_getkey`, `op_sto_m/n/o`, `op_rcl_m/n/o` (7 functions)
- [x] `hp41-core/src/ops/mod.rs` has 9 new Op variants
- [x] `hp41-core/src/ops/mod.rs` has 9 new dispatch() arms
- [x] `hp41-core/src/ops/mod.rs` has `pub fn synthetic_byte_to_op()` with 0xCF→Null mapping
- [x] `hp41-core/src/ops/program.rs` has 9 new execute_op() arms BEFORE catch-all
- [x] `hp41-cli/src/prgm_display.rs` has 9 new op_display_name() arms with correct strings
- [x] `hp41-core/tests/synthetic_tests.rs` exists in worktree
- [x] `just build` passes
- [x] `just lint` passes
- [x] `just test` passes (21 synthetic tests GREEN, no regressions)
- [x] `hp41-core` has no dependency on `hp41-cli` (Cargo.toml clean)
- [x] Commits f5bbcfb and 24757e7 exist

## Self-Check: PASSED

---
*Phase: 12-synthetic-programming*
*Completed: 2026-05-09*
