---
phase: 05-persistence-and-ux
plan: 01
subsystem: database
tags: [rust, serde, serde_json, rust_decimal, BTreeMap, Vec]

# Dependency graph
requires:
  - phase: 04-tui-and-input
    provides: CalcState, Op enum, Stack — all the types that now receive serde derives

provides:
  - HpNum with Serialize/Deserialize via rust_decimal::serde::str (lossless string encoding)
  - CalcState with Serialize/Deserialize — full state round-trip to JSON
  - Stack with Serialize/Deserialize
  - AngleMode, DisplayMode, Op, StoArithKind, TestKind with Serialize/Deserialize
  - regs migrated from [HpNum; 100] to Vec<HpNum> (resolves serde 32-element array limit)
  - user_mode: bool and key_assignments: BTreeMap<char, String> fields on CalcState (Phase 5 USER mode)
  - serde, serde_json workspace deps; dirs = "6" in hp41-cli

affects:
  - 05-03 (persistence.rs — saves/loads CalcState as JSON, requires this plan's serde impls)
  - 05-04 (USER mode dispatch — key_assignments field added here)
  - All downstream Phase 5 plans that consume CalcState

# Tech tracking
tech-stack:
  added:
    - serde = "1" (workspace dep, features = ["derive"])
    - serde_json = "1" (workspace dep; dev-dep in hp41-core, runtime dep in hp41-cli)
    - rust_decimal feature "serde-with-str" (lossless Decimal ↔ JSON string)
    - dirs = "6" (hp41-cli only — home dir resolution for autosave path)
  patterns:
    - HpNum serializes as JSON string not float (rust_decimal::serde::str) — prevents float precision loss
    - BTreeMap<char, String> for key_assignments — deterministic JSON key order for human-readable diffs
    - Vec<HpNum> for regs — serde works on Vec of any size (bypasses serde's [T; N > 32] limitation)

key-files:
  created: []
  modified:
    - Cargo.toml (workspace — serde + serde_json workspace deps)
    - hp41-core/Cargo.toml (serde dep, serde-with-str feature, serde_json dev-dep)
    - hp41-cli/Cargo.toml (serde, serde_json, dirs = "6")
    - hp41-core/src/num.rs (Serialize/Deserialize derives + serde(with) + two serde round-trip tests)
    - hp41-core/src/state.rs (Serialize/Deserialize on all 4 types; regs: Vec<HpNum>; user_mode + key_assignments)
    - hp41-core/src/ops/mod.rs (Serialize/Deserialize on Op, StoArithKind, TestKind)
    - hp41-core/src/ops/registers.rs (op_clreg: vec![HpNum::zero(); 100] — Vec migration fix)

key-decisions:
  - "HpNum serializes as JSON string via rust_decimal::serde::str — not float — to preserve 10-digit decimal precision across save/load cycles"
  - "regs migrated from [HpNum; 100] to Vec<HpNum> — serde's derive macro cannot handle fixed arrays > 32 elements"
  - "BTreeMap<char, String> for key_assignments — deterministic alphabetical JSON key order, no custom serde needed (D-25, D-29)"
  - "Op, StoArithKind, TestKind also needed serde derives — CalcState.program is Vec<Op> and must serialize"
  - "dirs = 6 used (not 5 as D-01 states) — RESEARCH.md corrects this; dirs 6 is current stable"

patterns-established:
  - "serde-with pattern: tuple struct field annotated with #[serde(with = \"rust_decimal::serde::str\")] for custom encoding"
  - "Workspace deps pattern: serde/serde_json in [workspace.dependencies] consumed via { workspace = true } in each crate"

requirements-completed: [PERS-01]

# Metrics
duration: 18min
completed: 2026-05-07
---

# Phase 5 Plan 01: Serde Foundation Summary

**CalcState, Stack, HpNum, and all Op types now implement Serialize/Deserialize; regs migrated from fixed array to Vec; user_mode and key_assignments fields added — full JSON persistence foundation in place**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-07T (worktree execution)
- **Completed:** 2026-05-07
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- HpNum serializes as a JSON string (not float) via rust_decimal::serde::str — preserves full 10-digit decimal precision through save/load cycles
- CalcState, Stack, AngleMode, DisplayMode, Op, StoArithKind, TestKind all derive Serialize + Deserialize — the complete state graph is serializable
- `regs` field migrated from `[HpNum; 100]` (incompatible with serde derive) to `Vec<HpNum>` — backward-compatible index syntax unchanged throughout the codebase
- Phase 5 USER mode fields added to CalcState: `user_mode: bool` and `key_assignments: BTreeMap<char, String>`
- serde + serde_json added as workspace dependencies; dirs = "6" added to hp41-cli
- All 282 existing hp41-core tests continue to pass — zero regressions

## Task Commits

1. **Task 1: Add serde deps to Cargo workspace files** - `07e7b50` (feat)
2. **Task 2: Add Serialize/Deserialize to HpNum via serde-with-str** - `15cfc25` (feat)
3. **Task 3: Add serde derives to CalcState/Stack/enums; migrate regs to Vec; add Phase 5 fields** - `af5e56e` (feat)

## Files Created/Modified

- `Cargo.toml` — added serde = "1" (with derive feature) and serde_json = "1" to workspace.dependencies
- `hp41-core/Cargo.toml` — added serde-with-str feature to rust_decimal, serde dep, serde_json dev-dep
- `hp41-cli/Cargo.toml` — added serde, serde_json (workspace), dirs = "6"
- `hp41-core/src/num.rs` — HpNum: added Serialize/Deserialize derives with #[serde(with = "rust_decimal::serde::str")]; added two round-trip tests
- `hp41-core/src/state.rs` — all four types serializable; regs: Vec<HpNum>; user_mode + key_assignments added
- `hp41-core/src/ops/mod.rs` — Op, StoArithKind, TestKind: added Serialize/Deserialize derives
- `hp41-core/src/ops/registers.rs` — op_clreg: fixed to use vec![HpNum::zero(); 100] (Vec migration)

## Decisions Made

- Used `rust_decimal::serde::str` feature (not a custom serialize impl) — upstream provides this exactly for lossless decimal-as-string encoding
- `dirs = "6"` used over D-01's "5" per RESEARCH.md correction — 6 is current stable
- Added serde derives to Op, StoArithKind, TestKind even though plan only mentioned CalcState/Stack/enums — required because CalcState.program is Vec<Op> (deviation auto-fixed inline)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] op_clreg still used std::array::from_fn after regs type changed to Vec**
- **Found during:** Task 3 (state.rs migration)
- **Issue:** After changing `regs: [HpNum; 100]` to `regs: Vec<HpNum>` in CalcState, the `op_clreg` function in registers.rs still assigned `state.regs = std::array::from_fn(|_| HpNum::zero())` — mismatched types, compilation error
- **Fix:** Changed to `state.regs = vec![HpNum::zero(); 100]` in op_clreg
- **Files modified:** hp41-core/src/ops/registers.rs
- **Verification:** cargo test -p hp41-core passes (282 tests)
- **Committed in:** af5e56e (Task 3 commit)

**2. [Rule 2 - Missing Critical] Op, StoArithKind, TestKind needed serde derives**
- **Found during:** Task 3 (attempting to compile CalcState with Serialize/Deserialize)
- **Issue:** Plan specified adding serde to CalcState/Stack/AngleMode/DisplayMode but CalcState.program is Vec<Op> — without serde on Op, the CalcState derive fails. Op contains StoArithKind and TestKind, which also need derives
- **Fix:** Added Serialize/Deserialize to StoArithKind, TestKind, Op in ops/mod.rs
- **Files modified:** hp41-core/src/ops/mod.rs
- **Verification:** All derives compile; all 282 tests pass
- **Committed in:** af5e56e (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both fixes were strictly necessary for correct compilation of the plan's deliverables. No scope creep.

## Issues Encountered

- Commit e8e22bd accidentally landed on the `develop` branch in the main repo (not the worktree branch) because a `cd /path/to/main-repo && git commit` sequence was used. Recovered by resetting `develop` to ea5df96 and re-applying all changes in the worktree with correct absolute paths. All subsequent commits are correctly on `worktree-agent-a3a0ac27f71807516`.

## Next Phase Readiness

- Plan 03 (persistence.rs) can now implement `save_state` / `load_state` using `serde_json::to_string(&state)` and `serde_json::from_str::<CalcState>(&json)` — the serialization foundation is complete
- Plan 04 (USER mode dispatch) can use `state.user_mode` and `state.key_assignments` — fields initialized and persisted
- No blockers for any downstream Phase 5 plan

## Self-Check

### Created files exist
- `hp41-core/src/num.rs` — exists, contains `#[serde(with = "rust_decimal::serde::str")]`
- `hp41-core/src/state.rs` — exists, contains `pub regs: Vec<HpNum>`, `pub user_mode: bool`, `pub key_assignments: BTreeMap<char, String>`

### Commits exist
- 07e7b50 — FOUND (Task 1)
- 15cfc25 — FOUND (Task 2)
- af5e56e — FOUND (Task 3)

## Self-Check: PASSED

---
*Phase: 05-persistence-and-ux*
*Completed: 2026-05-07*
