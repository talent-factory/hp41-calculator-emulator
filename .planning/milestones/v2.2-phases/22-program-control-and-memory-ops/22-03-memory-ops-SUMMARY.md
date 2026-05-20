---
phase: 22-program-control-and-memory-ops
plan: 03-memory-ops
subsystem: core
tags: [hp41, rust, memory-ops, stack-mgmt, size, cla, clst, pack, bounds-audit]

# Dependency graph
requires:
  - phase: 22-02-program-edit
    provides: "Op enum + dispatch + execute_op + both prgm_display.rs copies pattern landed for 6 prior variants (Stop/Pse/GtoInd/XeqInd + Clp/Del/Ins); 4-place rule and SC-4 invariant already established"
  - phase: 22-01-program-control
    provides: "Op::Pse precedent for memory-mgmt-style execute_op arms (regular dispatch op, NOT in programming-ops catch-all)"
  - phase: 06-science-and-engineering
    provides: "Σ-family ops (Σ+, Σ-, MEAN, SDEV, LR, YHAT, CORR, ClΣStat) in stats.rs — all 8 functions audited in 22-03-02 for bounds-safety under SIZE shrink"
  - phase: 02-core-math
    provides: "op_sto/op_rcl/op_sto_arith/op_view/op_clreg — all 5 functions audited in 22-03-01..03 to use state.regs.len() dynamically instead of hardcoded 100"
  - phase: 02-core-math
    provides: "op_alpha_clear — Op::Cla delegates to this (D-22.13 hardware-faithful display alias)"
provides:
  - "Op::Size(u16) — resize state.regs to nnn ∈ [1, 319]; OQ-2: nnn==0 clamps to 1; nnn>319 returns InvalidOp"
  - "Op::Cla — hardware-faithful CLA listing; delegates to op_alpha_clear (coexists with Op::AlphaClear)"
  - "Op::Clst — clears X/Y/Z/T while PRESERVING LASTX and lift_enabled (D-22.14 critical invariant)"
  - "Op::Pack — documented no-op for flat-Vec program model (D-22.12)"
  - "pub fn op_size, pub fn op_clst in hp41-core/src/ops/program.rs"
  - "Wave-0 bounds audit (D-22.11.1): op_sto / op_rcl / op_sto_arith / op_view / op_clreg honor state.regs.len() dynamically"
  - "8 Σ-family entry guards (Pitfall 5): if state.regs.len() < 7 return InvalidOp — fail-closed under SIZE shrink"
  - "prgm_display.rs (CLI + GUI) display strings: SIZE nnn / CLA / CLST / PACK"
affects:
  - phase: 22-04-catalog-and-asn
    note: "Plan 22-04 piles Op::Catalog and Op::Asn onto ops/mod.rs + ops/program.rs + both prgm_display.rs copies using the same 4-place rule pattern this plan exercised. The state.regs.len()-aware bounds discipline carries forward — CATALOG would not panic under SIZE shrink because no register access is involved."
  - phase: 25-cli-keyboard-wiring
    note: "Phase 25 will wire CLI keyboard SIZE prompt modal (3-digit numeric entry), R/S key routing for CLST/CLA/PACK keystrokes; the core surface is fully ready."
  - phase: 26-gui-key-map
    note: "Phase 26 GUI key_map can resolve new ids (size_prompt, cla, clst, pack) to real Op variants. SIZE will need a numeric-modal frontend; CLA/CLST/PACK are single-press resolves."
  - phase: 27-test-hardening
    note: "Future proptest opportunity: SIZE-resize roundtrip property (regs.len() == final_nnn after any sequence of Op::Size); skipped as low-value."

# Tech tracking
tech-stack:
  added: []  # purely additive on hp41-core; no new dependencies
  patterns:
    - "Wave-0 bounds audit: replace hardcoded length constants (100, etc.) with dynamic Vec::len() reads BEFORE introducing the op that can mutate the underlying Vec — separate commits per file for git-blame clarity"
    - "Σ-family fail-closed entry guard: single-line if state.regs.len() < 7 { return InvalidOp } at the top of any function that does fixed-position regs[1..=6] access"
    - "Vec::resize(target, default) for both grow and shrink semantics — single API covers truncate-tail-shrink AND zero-fill-grow"
    - "Documented no-op variant pattern (Op::Pack ↔ Op::Null precedent): 2-line dispatch+execute body that emits Neutral lift and Ok(())"
    - "LASTX/lift_enabled preservation pattern: absence of any assignment to those fields in the helper is the invariant; doc-comment + sentinel test enforce it"
    - "Dual-variant intentional duplication for display divergence (Op::Cla emits CLA, Op::AlphaClear emits CLRALPHA) — the 4-place rule forces per-variant display names"

key-files:
  created:
    - "hp41-core/tests/phase22_memory_ops.rs — 20 integration tests (344 lines)"
  modified:
    - "hp41-core/src/ops/mod.rs — 4 new Op variants (Size/Cla/Clst/Pack) + 4 dispatch arms"
    - "hp41-core/src/ops/program.rs — 2 new pub helpers (op_size, op_clst) + 4 execute_op arms"
    - "hp41-core/src/ops/registers.rs — Wave-0 audit (op_sto/op_rcl/op_sto_arith/op_clreg use state.regs.len())"
    - "hp41-core/src/ops/display_ops.rs — Wave-0 audit (op_view uses state.regs.len())"
    - "hp41-core/src/ops/stats.rs — 8 Σ-family entry guards added"
    - "hp41-cli/src/prgm_display.rs — 4 display arms (SIZE / CLA / CLST / PACK)"
    - "hp41-gui/src-tauri/src/prgm_display.rs — same 4 display arms (SC-4 duplication)"

key-decisions:
  - "Wave-0 bounds audit lands as 3 separate commits BEFORE Op::Size lands. Without this, Op::Size(50) followed by RCL 75 would PANIC (raw Vec[i] indexing), violating D-22.23 zero-panic invariant. The three commits separate concerns for git-blame clarity: (1) registers.rs+display_ops.rs op_sto/rcl/sto_arith/view, (2) stats.rs 8 entry guards, (3) op_clreg dynamic length."
  - "Σ-family threshold is uniform `< 7` not per-function-precise. Some functions strictly need only `< 6` (e.g., op_mean reads R05), but uniform `< 7` matches the Σ-block convention (R01..R06 spans indices 1..=6 so len < 7 means R06 unreachable) and simplifies the contract."
  - "Op::Size(0) → silently clamps to 1 per OQ-2 Option A (user-confirmed). The implementation uses `nnn.max(1) as usize` AFTER the `nnn > 319` early return. Documented divergence from real HP-41 (which accepts SIZE 000); the clamp avoids surprising InvalidOp on subsequent STO/RCL."
  - "Op::Cla and Op::AlphaClear COEXIST. The 4-place rule forces per-variant display names, so there is no way to give one variant two prgm_display strings. Hardware-faithful program listings show CLA; v1.0 save files contain Op::AlphaClear (display CLRALPHA). Removing Op::AlphaClear would break v1.0 save loading (Pitfall 8). The duplication is intentional and documented in both variants' doc-comments."
  - "Op::Clst preservation invariant is enforced by ABSENCE of assignment in the helper body (no `state.stack.lastx = ...` and no `state.stack.lift_enabled = ...` lines). apply_lift_effect(Neutral) is a no-op for lift_enabled (verified by reading stack.rs::apply_lift_effect). The sentinel test test_clst_preserves_lastx_and_lift_enabled explicitly asserts both fields are unchanged after CLST."
  - "Op::Pack inline body (no helper function) mirrors Op::Null precedent — a 2-line no-op (apply_lift_effect + Ok(())) doesn't warrant a named helper. Op::Size and Op::Clst do warrant helpers because they have non-trivial bodies."
  - "Σ+ boundary test added (test_sigma_plus_on_size_7_succeeds): without it, an over-aggressive `< 8` guard would have silently broken Σ+ at the exact boundary. The boundary test catches off-by-one errors that the standard reject-test misses."

patterns-established:
  - "Wave-0 bounds audit per separate-commit-per-file: when introducing an op that mutates a Vec's length, audit every existing access site BEFORE landing the new op, in separate commits for git-blame clarity. Pattern: hardcoded constants → `state.field.len()` reads + `.get()` patterns or entry guards. The 3-commit split (write sites / multi-read entry guards / re-init sites) is reusable."
  - "Σ-family entry-guard pattern: single-line `if state.field.len() < threshold { return Err(InvalidOp); }` at the top of any function that does fixed-position indexed access. Cheaper than rewriting bodies to use `.get()`; uniform threshold simplifies maintenance."
  - "Documented no-op variant pattern: Op variants for hardware-faithful program listings that have no observable effect (PACK on flat-Vec, future contributions). 2-line inline body + doc-comment that flags the divergence."
  - "Preservation-by-absence invariant: when a helper must NOT touch certain fields, the body simply omits assignments to those fields. Combined with a doc-comment listing the preserved fields and an integration test asserting their unchanged state, this is sufficient enforcement. No special `preserve!()` macro needed."

requirements-completed: [FN-MEM-01, FN-MEM-02, FN-MEM-03, FN-MEM-04]

# Metrics
duration: ~35 min
completed: 2026-05-14
---

# Phase 22 Plan 03: Memory & Stack Ops Summary

**Four memory/stack-management ops (SIZE / CLA / CLST / PACK) land in hp41-core behind a critical Wave-0 bounds-audit prep that converts 28 production sites in registers.rs / display_ops.rs / stats.rs from hardcoded 100-register assumptions to dynamic `state.regs.len()` reads — without the audit, SIZE shrinking would have introduced runtime panics in op_sto/op_rcl/op_sto_arith/op_view/op_clreg and the entire Σ-family.**

## Performance

- **Duration:** ~35 min (single uninterrupted execution wave; no checkpoints, no deviations)
- **Started:** 2026-05-14 (worktree-agent-aad395012efa4d806)
- **Tasks:** 8 of 8 complete (3 audit commits + 4 op-variant commits + 1 test commit)
- **Files modified:** 7 production files + 1 new integration test file
- **Lines added:** ~150 production + 344 test

## Accomplishments

- **Wave-0 bounds audit (D-22.11.1):** three separate commits convert 28 production register-access sites from hardcoded `reg >= 100` / `100`-element Vec assumptions to dynamic `state.regs.len()` reads. Commit 1 covers op_sto/op_rcl/op_sto_arith in registers.rs + op_view in display_ops.rs (Shape 1 indexed-write entry guards + Shape 2 .get().ok_or() reads). Commit 2 adds 8 Σ-family entry guards (op_sigma_plus / op_sigma_minus / op_mean / op_sdev / op_lr / op_yhat / op_corr / op_cl_sigma_stat) — single-line `if state.regs.len() < 7 { return Err(InvalidOp); }` at the top of each. Commit 3 makes op_clreg dynamic-sized: `vec![HpNum::zero(); state.regs.len()]` instead of `vec![..; 100]` so CLREG honors current SIZE. All 620 existing tests stay green through the entire audit — pure behavior-preserving refactor at the default SIZE=100 baseline.
- **Op::Size(u16) (FN-MEM-01):** resizes `state.regs` per AMENDED D-22.11. `nnn > 319` → `InvalidOp`; `nnn == 0` silently clamps to 1 via `nnn.max(1)` (OQ-2 Option A; documented divergence from real HP-41 which accepts SIZE 000). Otherwise `state.regs.resize(target, HpNum::zero())`: shrinking truncates the tail (hardware-faithful "MEM LOST"); growing zero-fills new slots; overlapping range preserves values. LiftEffect: Neutral. `u16` because 319 > u8::MAX.
- **Op::Cla (FN-MEM-02):** hardware-faithful HP-41 listing alias for CLRALPHA. Delegates to existing `op_alpha_clear(state)` — same body as legacy `Op::AlphaClear`. Both variants coexist (Pitfall 8); `Op::AlphaClear` stays in the enum for v1.0 save-file backward compat with its existing `"CLRALPHA"` display name. The duplication is intentional — the 4-place rule mandates per-variant display names so the only way to honor both HP-41 listing conventions ("CLA" in program listings) AND v1.0 save-file compat is two variants with distinct prgm_display arms.
- **Op::Clst (FN-MEM-03):** zeros stack X/Y/Z/T while PRESERVING `state.stack.lastx` AND `state.stack.lift_enabled` (D-22.14 invariant per HP-41 Owner's Manual ch. 7). Implementation enforces preservation by ABSENCE — `op_clst` body has zero assignments to lastx or lift_enabled, and `apply_lift_effect(Neutral)` is confirmed in `stack.rs` to be a no-op for `lift_enabled`. Sentinel integration test `test_clst_preserves_lastx_and_lift_enabled` explicitly asserts both fields are unchanged after CLST.
- **Op::Pack (FN-MEM-04):** documented no-op + Neutral lift per D-22.12. Real HP-41 PACK compacts program memory by removing gaps left by in-place edits; our flat-Vec program model has no gaps to compact, so PACK is a no-op. Inline 2-line body (matches `Op::Null` precedent) — no helper function for a no-op. Documented divergence is flagged in the doc-comment to discourage future "fixes" that don't first introduce gaps into the program Vec.
- **4-place rule + SC-4 invariant honored:** all 4 new variants land in (1) `ops/mod.rs` Op enum, (2) `ops/mod.rs` dispatch arm, (3) `ops/program.rs` execute_op arm (NOT in the programming-ops catch-all — these are regular dispatch ops that execute fine inside run_loop AND interactively), (4) BOTH `prgm_display.rs` copies (CLI and GUI, intentional duplication). Compile-time exhaustive-match coverage intact.
- **Pitfall 4 / Pitfall 5 sentinel tests:** explicit integration tests prove the audit works. `test_sto_out_of_range_after_shrink_returns_invalid_op_not_panic` runs `Op::Size(5)` then `Op::StoReg(50)` and asserts `Err(InvalidOp)` — without the audit, this would have PANICKED on raw `state.regs[50]`. `test_sigma_plus_on_shrunk_size_rejects_not_panic` runs `Op::Size(3)` then `Op::SigmaPlus` and asserts `Err(InvalidOp)` — without the Σ-family entry guards, this would have PANICKED on `state.regs[1]`. The standard test harness reports panics as failures, so the successful `Err` matches are sufficient evidence the audits are in place.
- **Test coverage:** 20 integration tests pass, each FN-ID has at least one positive test, all four pitfalls/decisions (Pitfall 4, Pitfall 5, OQ-2, D-22.14) have explicit sentinel tests, plus a boundary test (`test_sigma_plus_on_size_7_succeeds`) that catches off-by-one errors in the Σ-family threshold. `just ci` green; hp41-core coverage 90.16% lines / 92.28% regions (≥ 80% gate).

## Task Commits

Each task committed atomically on `worktree-agent-aad395012efa4d806`:

1. **Task 22-03-01: Wave-0 bounds audit COMMIT 1 — op_sto/op_rcl/op_sto_arith/op_view use state.regs.len()** — `1df6dc3` (refactor)
2. **Task 22-03-02: Wave-0 bounds audit COMMIT 2 — 8 Σ-family entry guards** — `1e71d1b` (refactor)
3. **Task 22-03-03: Wave-0 bounds audit COMMIT 3 — op_clreg respects current SIZE** — `9678029` (refactor)
4. **Task 22-03-04: Op::Size(u16) variant + helper + OQ-2 clamp-zero-to-one** — `2d0f5de` (feat)
5. **Task 22-03-05: Op::Cla (hardware-faithful CLA) coexists with Op::AlphaClear** — `1ec4572` (feat)
6. **Task 22-03-06: Op::Clst preserves LASTX + lift_enabled (D-22.14)** — `871ff60` (feat)
7. **Task 22-03-07: Op::Pack documented no-op for flat-Vec model** — `96cb54c` (feat)
8. **Task 22-03-08: hp41-core/tests/phase22_memory_ops.rs — 20 integration tests** — `79deb89` (test)

Plan metadata (this SUMMARY): will be committed separately as `docs(22-03)` per the parallel-executor protocol.

## Files Created/Modified

### Created
- `hp41-core/tests/phase22_memory_ops.rs` (344 lines) — 20 integration tests covering FN-MEM-01/02/03/04 + Pitfall 4 sentinel + Pitfall 5 sentinel + Σ+ boundary + OQ-2 + D-22.14 + CLREG-honors-SIZE + PACK full-state snapshot.

### Modified
- `hp41-core/src/ops/mod.rs` — 4 new Op variants (Size(u16) / Cla / Clst / Pack) appended at end of Op enum per D-22.22; 4 new dispatch arms; one inline 2-line PACK body, three delegating to helpers (op_size / op_alpha_clear / op_clst).
- `hp41-core/src/ops/program.rs` — 2 new public helper functions (`op_size`, `op_clst`) placed adjacent to the existing Phase 22 helpers (`op_clp`/`op_del`/`op_ins`); 4 new execute_op arms (Op::Size → op_size, Op::Cla → super::alpha::op_alpha_clear, Op::Clst → op_clst, Op::Pack → inline 2-line no-op). NONE of these join the programming-ops catch-all — they are regular dispatch ops that execute fine in both interactive AND run_loop contexts.
- `hp41-core/src/ops/registers.rs` — Wave-0 audit: op_sto/op_rcl/op_sto_arith now use `state.regs.len()` bounds (idx entry guard for op_sto + op_sto_arith, Shape 2 .get() for op_rcl); op_clreg uses dynamic `vec![..; state.regs.len()]` reinit; module header updated.
- `hp41-core/src/ops/display_ops.rs` — Wave-0 audit: op_view uses Shape 2 .get().ok_or().clone() pattern; doc-comment updated to note dynamic bound.
- `hp41-core/src/ops/stats.rs` — Wave-0 audit: 8 entry guards added (one per Σ-family function). Single-line `if state.regs.len() < 7 { return Err(InvalidOp); }` at the top of each.
- `hp41-cli/src/prgm_display.rs` — 4 new arms: `Op::Size(n) => format!("SIZE {n:03}")`, `Op::Cla => "CLA"`, `Op::Clst => "CLST"`, `Op::Pack => "PACK"`.
- `hp41-gui/src-tauri/src/prgm_display.rs` — same 4 arms, intentional duplication per CLAUDE.md SC-4 invariant.

## Decisions Made

- **Three-commit Wave-0 audit split for git-blame clarity:** the plan offered to fold the audit into the Op::Size landing OR split it. I chose the three-commit split per the plan's recommendation (and PATTERNS.md "Affected sites" table organization). Commit 1 covers all 4 fixed-shape sites in registers.rs + 1 in display_ops.rs (op_sto/op_rcl/op_sto_arith/op_view — share the `reg >= 100` BEFORE pattern). Commit 2 is the multi-read Σ-family entry guards in stats.rs (different shape — Shape 3 from PATTERNS.md). Commit 3 is the op_clreg dynamic-length reinit (different shape again — re-allocation rather than indexed access). Three separate concerns; three separate commits.
- **Σ-family threshold is uniform `< 7`:** strict per-function thresholds would be `< 6` for op_mean (reads R01..R05) and `< 7` for the others. Uniform `< 7` matches the Σ-block convention (R01..R06) and avoids per-function magic numbers. The boundary test (`test_sigma_plus_on_size_7_succeeds`) catches off-by-one errors.
- **Op::Size uses `u16` not `u32`:** 319 < u16::MAX = 65535; `u16` is sufficient and matches the planner's discretion call. Save-file shape: `{"Size": 50}` — same as Op::PushNum's tagged-variant shape.
- **Op::Size body uses `nnn.max(1)` not `nnn.clamp(1, 319)`:** the upper bound is already validated by the early-return `if nnn > 319`, so a second clamp would be redundant. `.max(1)` cleanly expresses "OQ-2: clamp 0 to 1" without re-checking the upper bound.
- **Op::Pack uses inline 2-line body, not a helper function:** matches Op::Null precedent. A named `op_pack` helper for a 2-line no-op would just add boilerplate without grep affinity benefit. Doc-comment on the variant captures the divergence rationale.
- **Op::Cla doc-comment explicitly flags Pitfall 8:** the variant doc-comment cross-references Op::AlphaClear and explains why the duplication is intentional. Future maintainers seeing "two variants do the same thing" will read the cross-reference before "cleaning up".
- **Op::Clst preservation is enforced by helper-body ABSENCE:** rather than a comment-only invariant, the helper simply omits assignments to lastx + lift_enabled. The integration test `test_clst_preserves_lastx_and_lift_enabled` is the active sentinel that catches any future addition. Pattern reusable for any "preserves field X" semantic.

## Deviations from Plan

None — plan executed exactly as written. All 8 tasks landed in order with no deviation rules triggered:
- No bugs found in the prior Phase 22 surface (22-01, 22-02). The audit was prep-work, not bug-fix.
- No missing critical functionality discovered beyond what the plan anticipated (Wave-0 bounds audit was the centerpiece of the plan).
- No blocking issues; no architectural changes needed.
- No auth gates (purely core code).
- Initial draft of tasks 4–7 was bundled into one set of file edits; reverted before commit and redone per-task to honor the 8-commit requirement. Net effect: same final state on disk, plus the per-task commit cadence the plan specified.

## Issues Encountered

None functional. One ergonomic observation: my first attempt at tasks 22-03-04..07 bundled all four Op variant additions into a single set of file edits (since they touch the same 4 files and the changes are mechanically similar). I caught this before the first per-task commit, reverted the uncommitted changes with `git checkout --`, and redid the tasks one variant at a time to honor the plan's 8-commit cadence. This was a workflow mis-step, not a code defect.

The 3-commit Wave-0 audit refactor is behavior-preserving at SIZE=100, which is exactly what every existing test exercises — so all 620 pre-existing tests stay green through the entire audit, then jump to 640 with the 20 new tests in 22-03-08.

## User Setup Required

None — entirely additive on `hp41-core`. No new dependencies, no env vars, no service config. The new ops surface to end users via Phase 25 (CLI keyboard) and Phase 26 (GUI key_map) — Phase 22-03 just lands them in core.

## Next Plan Readiness

**Plan 22-04 (catalog-and-asn) is unblocked.** It will land `Op::Catalog(u8)` and `Op::Asn { name, key_code }` onto `ops/mod.rs` + `ops/program.rs` + both `prgm_display.rs` copies using the same 4-place rule pattern this plan exercised. Plan 22-04 also introduces a NEW field on `CalcState` (`assignments: BTreeMap<u8, String>`) with `#[serde(default)]` — first state.rs touch in Phase 22. CATALOG output goes to `print_buffer` (Phase 11 channel) so no new I/O channels are needed. The Σ-family entry guards added in 22-03-02 are dormant under default SIZE=100; they activate only if a future SIZE shrink reduces regs.len() below 7.

**Phase 25 (CLI keyboard) can now wire SIZE/CLA/CLST/PACK keystrokes.** SIZE needs a 3-digit numeric prompt modal (`PendingInput::SizePrompt(String)`); CLA/CLST/PACK are single-press resolves. Phase 25 also gains the option to bind keyboard CLA to Op::Cla (hardware-faithful) and reserve Op::AlphaClear for v1.0 save-file loading only.

**Phase 26 (GUI key_map) can resolve new ids.** v2.1 stubs for `size_prompt` (currently surfaces "unknown key: size_prompt" toast per the modal-prompt-id pattern) will route to a real Op variant once Phase 26 adds the SizePrompt frontend modal. `cla`, `clst`, `pack` are single-id resolves.

## OQ-4 Acknowledgement (per plan output spec)

**Op::Cla and Op::AlphaClear coexist intentionally.** A v1.x save file containing `Op::AlphaClear` continues to list as `"CLRALPHA"` after v2.2; fresh user-recorded `CLA` keystrokes (Phase 25/26) will use `Op::Cla` and list as `"CLA"`. This visible divergence in program listings is documented in CONTEXT.md OQ-4 resolution and will be highlighted in the Phase 22 milestone changelog. Future maintainers should NOT consolidate the two variants — removing Op::AlphaClear breaks v1.0 save-file loading (Pitfall 8).

## Self-Check: PASSED

Files claimed created/modified verified present:
- `hp41-core/tests/phase22_memory_ops.rs` — FOUND (344 lines, 20 tests)
- `hp41-core/src/ops/mod.rs` — FOUND (modified, contains Op::Size/Cla/Clst/Pack variants + 4 dispatch arms)
- `hp41-core/src/ops/program.rs` — FOUND (modified, contains pub fn op_size + pub fn op_clst + 4 execute_op arms)
- `hp41-core/src/ops/registers.rs` — FOUND (audit: op_sto/op_rcl/op_sto_arith/op_clreg use state.regs.len())
- `hp41-core/src/ops/display_ops.rs` — FOUND (audit: op_view uses Shape 2 .get())
- `hp41-core/src/ops/stats.rs` — FOUND (audit: 8 entry guards, 1 per Σ-family function)
- `hp41-cli/src/prgm_display.rs` — FOUND ("SIZE", "CLA", "CLST", "PACK" arms present)
- `hp41-gui/src-tauri/src/prgm_display.rs` — FOUND (same 4 arms present)

Commit hashes verified present on `worktree-agent-aad395012efa4d806`:
- `1df6dc3` — refactor(22-03): audit commit 1 (op_sto/rcl/sto_arith/view) ✓
- `1e71d1b` — refactor(22-03): audit commit 2 (Σ-family entry guards) ✓
- `9678029` — refactor(22-03): audit commit 3 (op_clreg dynamic) ✓
- `2d0f5de` — feat(22-03): Op::Size with OQ-2 ✓
- `1ec4572` — feat(22-03): Op::Cla ✓
- `871ff60` — feat(22-03): Op::Clst preserves LASTX + lift_enabled ✓
- `96cb54c` — feat(22-03): Op::Pack documented no-op ✓
- `79deb89` — test(22-03): phase22_memory_ops.rs (20 tests) ✓

Quality gates verified green:
- `cargo check --workspace` — exit 0
- `cd hp41-gui/src-tauri && cargo check` — exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0
- `cargo test -p hp41-core --test phase22_memory_ops` — 20 passed, 0 failed
- `cargo test -p hp41-core` — 640 passed, 0 failed (was 620 before this plan)
- `just ci` — exit 0 (workspace tests + clippy + fmt + coverage); hp41-core 90.16% lines / 92.28% regions
- Zero `.unwrap()` / `panic!()` introduced in production code
- Bounds audit completeness: every `state.regs[idx]` indexed access in non-test code is preceded by either a `.len()` guard or a `.get()` shape
- Pitfall 4 + Pitfall 5 sentinels: both pass — proves SIZE shrink no longer panics legacy register access paths

---
*Phase: 22-program-control-and-memory-ops*
*Plan: 03-memory-ops*
*Completed: 2026-05-14*
