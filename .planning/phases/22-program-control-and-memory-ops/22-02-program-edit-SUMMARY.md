---
phase: 22-program-control-and-memory-ops
plan: 02-program-edit
subsystem: core
tags: [hp41, rust, program-editing, prgm-mode, clp, del, ins]

# Dependency graph
requires:
  - phase: 22-01-program-control
    provides: "Op enum extended with Phase 22 control variants (Stop/Pse/GtoInd/XeqInd); programming-ops catch-all extended; 4-place rule + SC-4 invariant patterns established in current wave"
  - phase: 12-synthetic-programming
    provides: "Op::Null no-op placeholder (Ins inserts these); 4-place Op-variant rule scaffolding"
  - phase: 03-programming-engine
    provides: "state.program: Vec<Op>; state.pc cursor; state.prgm_mode flag; programming-ops catch-all in execute_op; PRGM-mode recording gate in dispatch()"
provides:
  - "Op::Clp(String) — clear program from LBL to next LBL/end-of-Vec with cursor reposition (Pitfall 6)"
  - "Op::Del(u8) — delete N steps from pc with silent clamping (saturating_sub guard); pc unchanged"
  - "Op::Ins — insert Op::Null at pc; pc unchanged"
  - "pub fn op_clp / op_del / op_ins helpers in hp41-core/src/ops/program.rs"
  - "PRGM-mode recording-gate special-case: Clp/Del/Ins execute immediately even while prgm_mode == true (D-22.10 — editing primitives, never self-recorded)"
  - "prgm_display.rs (CLI + GUI) display strings: CLP name / DEL nnn / INS"
affects:
  - phase: 22-03-memory-ops
    note: "Plan 22-03 piles Op::Size/Cla/Clst/Pack onto ops/mod.rs + program.rs + both prgm_display.rs copies using the same 4-place rule pattern this plan exercised. The catch-all stays unchanged (memory ops execute in both interactive and program contexts)."
  - phase: 22-04-catalog-and-asn
    note: "Op::Catalog and Op::Asn follow the same 4-place rule landing pattern."
  - phase: 25-cli-keyboard-wiring
    note: "Phase 25 will wire CLI keyboard CLP/DEL/INS keystrokes (interactive PRGM-mode editing primitives, no recording semantics)."
  - phase: 26-gui-key-map
    note: "Phase 26 will surface the v2.1 stub-error pattern for clp/del/ins ids — they exist in key_map.rs but currently return GuiError. Now that core has the ops, key_map can resolve them to real Op variants."

# Tech tracking
tech-stack:
  added: []  # purely additive on hp41-core; no new dependencies
  patterns:
    - "PRGM-mode dispatch-gate special-case for editing primitives — Clp/Del/Ins bypass the auto-record branch and fall through to immediate execution"
    - "Vec::drain(start..end) + cursor reposition via .min(post_drain_len) clamp — pattern for any future block-deletion op"
    - "Defense-in-depth prgm_mode guard inside each helper (redundant with dispatch gate, but keeps helpers safe-by-themselves)"
    - "saturating_sub for clamp-on-clamping operations on unsigned cursor/length arithmetic"

key-files:
  created:
    - "hp41-core/tests/phase22_program_edit.rs — 13 integration tests (307 lines)"
  modified:
    - "hp41-core/src/ops/mod.rs — 3 new Op variants (Clp/Del/Ins) + 3 dispatch arms + PRGM-mode gate special-case"
    - "hp41-core/src/ops/program.rs — 3 new pub helper functions (op_clp/op_del/op_ins) + extended programming-ops catch-all"
    - "hp41-cli/src/prgm_display.rs — 3 display arms (CLP {name} / DEL {n:03} / INS)"
    - "hp41-gui/src-tauri/src/prgm_display.rs — same 3 display arms (SC-4 duplication)"

key-decisions:
  - "PRGM-mode gate special-case lives in dispatch() (ops/mod.rs) — the auto-record branch checks `!matches!(op, Op::Clp(_) | Op::Del(_) | Op::Ins)` before pushing onto state.program. Without this, the Phase 3 recording gate would silently append the edit ops as program steps, self-corrupting the program buffer."
  - "Helpers carry a defense-in-depth prgm_mode guard even though dispatch already gates. Rationale: protects the helpers if a future direct caller (test harness, alternate dispatcher) invokes them — and the redundancy is documented in the doc-comments rather than being a stale check."
  - "Op::Del uses saturating_sub on program.len() - state.pc, not raw subtraction. The pc > len case shouldn't happen in well-formed states but the guard keeps the helper bounds-safe under any state (CLAUDE.md zero-panic invariant)."
  - "Op::Clp pc-reposition uses start.min(program.len()) clamp (not the raw `start`), so the rare case where start == post-drain program.len() (entire labelled suffix deleted) yields a valid one-past-the-end pc rather than a dangling index."
  - "INS uses the existing Op::Null variant from Phase 12 — zero new variants, hardware-faithful no-op semantics with Neutral lift. (CONTEXT.md D-22.8 + Phase 12 precedent at program.rs Op::Null arm.)"
  - "CLP missing-label rejects with InvalidOp (D-22.7). Documented divergence from HP-41 hardware (END/.END. markers vs next-LBL boundary) is captured inline in the op_clp doc-comment."

patterns-established:
  - "PRGM-mode editing-primitive pattern: Op variants that mutate state.program directly must be special-cased in the dispatch recording gate; the catch-all in execute_op rejects with InvalidOp so accidental run_loop reach fails closed."
  - "Cursor-reposition-after-drain idiom (Pitfall 6 sentinel): state.pc = anchor.min(program.len()) — bound-clamped to be valid one-past-the-end."

requirements-completed: [FN-PROG-03, FN-PROG-04, FN-PROG-05]

# Metrics
duration: ~25 min
completed: 2026-05-14
---

# Phase 22 Plan 02: Program Editing Summary

**Three PRGM-mode editing primitives (CLP / DEL / INS) — programs can now be incrementally edited by name (CLP) and by cursor position (DEL/INS) without rebuilding from scratch, completing the v2.2 keystroke-programming editing surface.**

## Performance

- **Duration:** ~25 min (single uninterrupted execution wave; no checkpoints, no deviations)
- **Started:** 2026-05-14 (worktree-agent-a6daba3cbb7f34457)
- **Tasks:** 4 of 4 complete
- **Files modified:** 4 production files + 1 new integration test file
- **Lines added:** ~155 production + 307 test

## Accomplishments

- **Op::Clp(String) — clear by name:** Finds the target `Op::Lbl(label)` via exact string match (.position), determines the end boundary as the next `Op::Lbl(_)` or program.len() if the target is the last labelled block (Pitfall 4 / drain-to-end-of-Vec), drains `[start..end)` from state.program, then repositions `state.pc = start.min(program.len())` so the cursor lands at the start of (now-deleted) block, clamped to the new program length (Pitfall 6). Missing label → InvalidOp; outside PRGM mode → InvalidOp (D-22.7 + D-22.10).
- **Op::Del(u8) — delete by count:** Silently clamps the requested count to `min(nnn, program.len() - state.pc)` using `.saturating_sub` (guards the pathological pc > len edge); `n == 0` → no-op (covers both `nnn == 0` AND `pc == len`); otherwise `state.program.drain(state.pc..state.pc + n)`. **state.pc is UNCHANGED** — drain shifts the tail down so pc naturally falls at the same index. Outside PRGM mode → InvalidOp (D-22.9 + D-22.10).
- **Op::Ins — insert blank step:** `state.program.insert(state.pc, Op::Null)` — reuses the Phase 12 no-op placeholder; zero new variants needed. **state.pc is UNCHANGED** — cursor still points at the freshly inserted Null (HP-41 hardware "INS lands a blank step at cursor" semantic). Outside PRGM mode → InvalidOp (D-22.8 + D-22.10).
- **PRGM-mode recording-gate special-case:** The existing Phase 3 dispatch gate (ops/mod.rs) appended every non-PrgmMode op to state.program while `prgm_mode == true`. Phase 22 extends the gate so that `Op::Clp(_) | Op::Del(_) | Op::Ins` bypass the auto-record branch and fall through to immediate execution. Without this, calling CLP while editing a program would have silently appended `Op::Clp` to the program buffer instead of removing the labelled block — a hard-to-debug self-corruption. Verified by `test_ins_is_not_self_recorded_in_prgm_mode`.
- **4-place rule + SC-4 invariant honored:** All three new variants land in (1) ops/mod.rs `Op` enum, (2) ops/mod.rs `dispatch()` match, (3) ops/program.rs `execute_op()` programming-ops catch-all, (4) **both** prgm_display.rs copies (CLI and GUI, intentional duplication per CLAUDE.md §SC-4). Compile-time exhaustive-match coverage is intact.
- **Test coverage:** 13 integration tests pass, each FN-ID has at least one positive test + one prgm_mode-false rejection test, both RESEARCH §2 pitfalls (Pitfall 4 drain-to-end + Pitfall 6 pc reposition) have explicit sentinel tests, plus the bonus `test_ins_is_not_self_recorded_in_prgm_mode` sentinel that verifies the dispatch-gate special-case. `just ci` green; hp41-core coverage 92.52% lines / 90.34% regions (≥ 80% gate).

## Task Commits

Each task committed atomically on `worktree-agent-a6daba3cbb7f34457`:

1. **Task 22-02-01: Op::Clp/Del/Ins variants + dispatch + display arms + helper stubs + PRGM-mode gate special-case** — `29028d7` (feat)
2. **Task 22-02-02: op_clp full body — drain LBL..next-LBL with cursor reposition** — `8d64692` (feat)
3. **Task 22-02-03: op_del + op_ins bodies — DEL clamp + INS Null insert** — `7770446` (feat)
4. **Task 22-02-04: hp41-core/tests/phase22_program_edit.rs — 13 integration tests** — `e579720` (test)

Plan metadata (this SUMMARY): committed separately as `docs(22-02)`.

## Files Created/Modified

### Created
- `hp41-core/tests/phase22_program_edit.rs` (307 lines) — 13 integration tests covering FN-PROG-03/04/05 + Pitfalls 4 & 6 + D-22.10 prgm_mode-gate + dispatch-gate special-case sentinel.

### Modified
- `hp41-core/src/ops/mod.rs` — 3 new Op variants (Clp(String) / Del(u8) / Ins) appended at end of Op enum per D-22.22; 3 new dispatch arms routing to crate::ops::program::op_clp / op_del / op_ins; the Phase 3 PRGM-mode recording gate now special-cases `Op::Clp(_) | Op::Del(_) | Op::Ins` to fall through to the dispatch match (executing immediately) instead of auto-recording (D-22.10).
- `hp41-core/src/ops/program.rs` — 3 new public helper functions (`op_clp`, `op_del`, `op_ins`) placed adjacent to the existing Phase 3 helpers (`op_lbl`/`op_gto`/`op_xeq`/etc.); programming-ops catch-all extended with `Op::Clp(_) | Op::Del(_) | Op::Ins => Err(HpError::InvalidOp)`. Each helper carries a defense-in-depth `prgm_mode` guard (redundant with dispatch gate, but keeps the helper safe-by-itself).
- `hp41-cli/src/prgm_display.rs` — 3 new arms: `Op::Clp(name) => format!("CLP {name}")`, `Op::Del(n) => format!("DEL {n:03}")`, `Op::Ins => "INS"`.
- `hp41-gui/src-tauri/src/prgm_display.rs` — same 3 arms, intentional duplication per CLAUDE.md SC-4 invariant.

## Decisions Made

- **Helper-stub strategy for task 22-02-01:** the plan offered two paths (`unimplemented!()` stubs vs `Err(HpError::InvalidOp)` stubs). I picked the `Err(InvalidOp)` path because `unimplemented!()` emits a clippy warning under our `-D warnings` setting and would have polluted the commit. The stubs were trivially replaced by full bodies in tasks 22-02-02 and 22-02-03.
- **PRGM-mode gate special-case lives in dispatch(), not in the helpers.** The catch-all check is `if !matches!(op, Op::Clp(_) | Op::Del(_) | Op::Ins)` — this keeps the entire PRGM-mode recording policy in one place (the gate) and avoids spreading the "don't record this op" knowledge across every editing-primitive helper. The helpers stay generic.
- **Helper guards are defense-in-depth, not the primary policy enforcement.** The dispatch gate already ensures CLP/DEL/INS only reach the helpers when invoked through dispatch in PRGM mode. The redundant `if !state.prgm_mode { return Err(InvalidOp); }` inside each helper protects against future direct-caller scenarios (test harness, alternate dispatcher). Documented in the doc-comments.
- **CLP cursor reposition uses `.min(program.len())` clamp, not raw `start`.** The rare case where `start == post-drain program.len()` (entire labelled suffix deleted) needs to yield a valid one-past-the-end pc; without the clamp, pc would be one PAST that. Test `test_clp_pc_clamps_to_program_len` exercises this edge.
- **DEL uses `program.len().saturating_sub(state.pc)` for clamp arithmetic.** Raw subtraction would underflow if `pc > len`. The saturating variant is the bounds-safe idiom under our zero-panic invariant.

## Deviations from Plan

None — plan executed exactly as written. All 4 tasks landed in order with no deviation rules triggered:
- No bugs found in the existing Phase 22 Plan 01 surface.
- No missing critical functionality discovered (the dispatch-gate special-case was anticipated by the plan's "ALL three Ops MUST NEVER be recorded into state.program" note + my reading of the existing Phase 3 recording gate).
- No blocking issues; no architectural changes needed.
- No auth gates (purely core code).

The plan's task 22-02-01 stub-strategy ambiguity (unimplemented! vs Err(InvalidOp)) was resolved by choosing the clippy-clean path, but this is a normal planner-discretion call, not a deviation.

## Issues Encountered

None functional. One minor observation: `cargo check --workspace` re-builds 2 crates when only program.rs changed in task 22-02-02 (the workspace includes `hp41-core` and `hp41-cli`); this is expected because `hp41-cli` depends on `hp41-core` and Rust re-checks dependents whenever the dependency changes. Not a problem — just an artifact of `--workspace`.

## User Setup Required

None — entirely additive on `hp41-core`. No new dependencies, no env vars, no service config. PRGM-mode CLP/DEL/INS keyboard wiring (CLI in Phase 25, GUI in Phase 26) will surface the new ops to end users; Phase 22-02 just lands them in core.

## Next Plan Readiness

**Plan 22-03 (memory-ops) is unblocked.** It will land `Op::Size(u16)`, `Op::Cla`, `Op::Clst`, `Op::Pack` onto `ops/mod.rs` + `program.rs` + both `prgm_display.rs` copies using the same 4-place rule pattern this plan exercised. Plan 22-03 includes the D-22.11.1 Wave-0 bounds audit (28 production sites in registers/display_ops/stats) — that's a meaty prep task. The catch-all stays unchanged (memory ops execute in both interactive and program contexts, unlike the edit primitives).

**Phase 25 (CLI keyboard) can now wire CLP/DEL/INS modal flows.** PRGM-mode editing keystrokes (typically a label-prompt for CLP, a step-count prompt for DEL, a single press for INS) need new `PendingInput` variants per the CONTEXT.md "Out of scope (Phase 25)" note. The hp41-core surface is ready.

**Phase 26 (GUI key_map) can replace the v2.1 stub-error for clp/del/ins.** The v2.1 stub-error pattern surfaces "'<id>' is planned for a future phase" toasts for these three ids; now that core has the ops, `key_map::resolve` can return real Op variants. The modal-prompt frontend wiring lands in Phase 26.

## Self-Check: PASSED

Files claimed created/modified verified present:
- `hp41-core/tests/phase22_program_edit.rs` — FOUND (307 lines)
- `hp41-core/src/ops/mod.rs` — FOUND (modified, contains Op::Clp/Del/Ins variants + dispatch arms + PRGM-mode gate special-case)
- `hp41-core/src/ops/program.rs` — FOUND (modified, contains pub fn op_clp/op_del/op_ins + extended catch-all)
- `hp41-cli/src/prgm_display.rs` — FOUND ("CLP", "DEL", "INS" arms present)
- `hp41-gui/src-tauri/src/prgm_display.rs` — FOUND (same 3 arms present)

Commit hashes verified present on `worktree-agent-a6daba3cbb7f34457`:
- `29028d7` — feat(22-02): variants + dispatch + display + stubs + PRGM-mode gate ✓
- `8d64692` — feat(22-02): op_clp body ✓
- `7770446` — feat(22-02): op_del + op_ins bodies ✓
- `e579720` — test(22-02): phase22_program_edit.rs (13 tests) ✓

Quality gates verified green:
- `cargo check --workspace` — exit 0
- `cd hp41-gui/src-tauri && cargo check` — exit 0
- `cargo clippy -p hp41-core -p hp41-cli --all-targets -- -D warnings` — exit 0
- `cargo test -p hp41-core --test phase22_program_edit` — 13 passed, 0 failed
- `cargo test -p hp41-core --test phase22_program_control` — 15 passed, 0 failed (Plan 22-01 suite preserved)
- `just ci` — exit 0 (workspace tests + clippy + fmt + coverage); hp41-core 92.52% lines / 90.34% regions
- Zero `.unwrap()` / `panic!()` in production code (all new helpers use `?`-propagation; pc-unchanged invariant verified by grep returning 0 for `state.pc =` inside op_del/op_ins bodies)

---
*Phase: 22-program-control-and-memory-ops*
*Plan: 02-program-edit*
*Completed: 2026-05-14*
