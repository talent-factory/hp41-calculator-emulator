---
phase: 22-program-control-and-memory-ops
plan: 01-program-control
subsystem: core
tags: [hp41, rust, interpreter, run_loop, indirect-addressing, stop-resume, pse]

# Dependency graph
requires:
  - phase: 21-flags-display-sound
    provides: "display_override + event_buffer channels (BEEP/TONE/AVIEW/PROMPT precedents); run_loop break-with-state pattern (Op::Prompt template); dispatch-top display_override clear (Pitfall 5)"
  - phase: 03-programming-engine
    provides: "run_program / run_loop / execute_op skeleton; programming-ops catch-all; find_in_program; 4-deep call_stack semantics"
  - phase: 20-additional-math-and-rounding
    provides: "HpNum::trunc_int (integer-pointer extraction primitive for GTO/XEQ IND)"
provides:
  - "Op::Stop variant with run_loop break (no display_override write)"
  - "Op::Pse variant with two-channel write (display_override + event_buffer \"PAUSE 1000\")"
  - "Op::GtoInd(u8) — inline indirect-branch resolver with non-integer reject and bounds-safe register read"
  - "Op::XeqInd(u8) — same resolver plus pre-mutation 4-deep call_stack guard returning CallDepth"
  - "pub fn resume_program(state) -> Result<(), HpError> — let-result reset-on-Err entry point that re-enters run_loop from state.pc and preserves call_stack"
  - "prgm_display.rs (CLI + GUI) display strings: STOP / PSE / GTO IND nn / XEQ IND nn"
affects:
  - phase: 24-indirect-addressing
    note: "Will extract the Phase 22 inline indirect resolver (GtoInd/XeqInd) into a shared resolve_indirect(state, reg) helper for ~13 other IND variants. The inline integer-truncate-then-equality-check is intentionally duplicated here so 24 has a concrete refactor target."
  - phase: 22-02-program-edit
    note: "Plan 22-02 piles Op::Clp/Del/Ins onto ops/mod.rs + program.rs catch-all using the same 4-place rule landing pattern this plan exercised."
  - phase: 25-cli-keyboard-wiring
    note: "Phase 25 wires CLI R/S key to call resume_program; STOP/PSE become keyboard-reachable end-to-end."
  - phase: 26-gui-key-map
    note: "Phase 26 wires the v2.1 cosmetic run_stop Tauri command to call resume_program; GUI gains real R/S behavior."

# Tech tracking
tech-stack:
  added: []  # purely additive on hp41-core; no new dependencies
  patterns:
    - "Inline indirect resolver (6-step pattern: get-clone → trunc_int → equality reject → inner().to_string() → find_in_program → pc adjust) ready for Phase 24 extraction"
    - "Pre-mutation 4-deep call-stack guard for XEQ-family (matches Op::Xeq precedent at program.rs:206-207)"
    - "Two-channel display+event write for sub-second timing (display_override + event_buffer marker; frontend interprets timing)"
    - "let-result reset-on-Err pattern for is_running-toggle entry points (resume_program mirrors run_program; do NOT use `?` propagation)"

key-files:
  created:
    - "hp41-core/tests/phase22_program_control.rs — 15 integration tests (348 lines)"
  modified:
    - "hp41-core/src/ops/mod.rs — 4 new Op variants + 4 dispatch arms"
    - "hp41-core/src/ops/program.rs — Op::Stop / Op::GtoInd / Op::XeqInd run_loop arms + Op::Pse execute_op arm + extended programming-ops catch-all + pub fn resume_program()"
    - "hp41-core/src/lib.rs — re-export resume_program"
    - "hp41-cli/src/prgm_display.rs — 4 display arms (STOP / PSE / GTO IND nn / XEQ IND nn)"
    - "hp41-gui/src-tauri/src/prgm_display.rs — same 4 display arms (SC-4 duplication)"

key-decisions:
  - "Op::Stop arm in run_loop is a bare `break` — NO display_override write (Pitfall 1)"
  - "Op::Pse uses inline body in BOTH dispatch and execute_op (rather than delegating to a helper) per PATTERNS.md sketch — minimizes new files for grep affinity"
  - "resume_program uses `let result = run_loop(state, &program); state.is_running = false; result` pattern — never `?`-propagation (Pitfall 2)"
  - "resume_program does NOT clear state.call_stack — unlike run_program, preserves pending XEQ frames so a STOP-inside-subroutine + R/S resumes correctly"
  - "Op::GtoInd / Op::XeqInd dispatch arms return InvalidOp (programming-only; run_loop has the real arms)"
  - "Op::Pse is the ONLY Phase 22 op NOT in the programming-ops catch-all — by design, it executes in execute_op mid-program (that's the whole point of PSE)"
  - "Stringify integer via int_part.inner().to_string() — note HpNum::inner() returns Decimal by value, not a reference (different from CONTEXT.md sketch wording)"

patterns-established:
  - "Inline indirect resolver: 6-step bounds-safe pattern in run_loop, ready for Phase 24 extraction into resolve_indirect()"
  - "STOP/PSE asymmetry: STOP breaks run_loop (yields to user), PSE writes display+event marker and continues (timed-display continuation)"
  - "Pre-mutation atomicity for call-stack-pushing ops: guard fires BEFORE register read, so an over-deep call returns CallDepth without partial mutation"

requirements-completed: [FN-PROG-01, FN-PROG-02, FN-PROG-06, FN-PROG-07]

# Metrics
duration: ~50 min
completed: 2026-05-14
---

# Phase 22 Plan 01: Program Control & Indirect Branching Summary

**Four interpreter control-flow ops (STOP / PSE / GTO IND / XEQ IND) plus pub fn resume_program — programs can now pause, yield to the user via R/S, and indirect-branch through register pointers, unlocking the entire keyboard-programming control surface for Phase 25/26.**

## Performance

- **Duration:** ~50 min (single uninterrupted execution wave; no checkpoints, no deviations)
- **Started:** 2026-05-14 (worktree-agent-a891c99d8d3e4bccd)
- **Tasks:** 6 of 6 complete
- **Files modified:** 5 production files + 1 new integration test file
- **Lines added:** ~430 production + tests

## Accomplishments

- **STOP/resume foundation:** `Op::Stop` breaks `run_loop` without overwriting `display_override` (preserves the previous step's value — HP-41 hardware semantic verified by `test_stop_does_not_write_display_override`). The new `pub fn resume_program(state)` re-enters `run_loop` from `state.pc` with the proven `let-result-then-reset` pattern that resets `is_running` on the Err path too (Pitfall 2 sentinel test passing).
- **PSE pause-display semantics:** `Op::Pse` writes `format_hpnum(X, display_mode)` to `state.display_override` AND pushes `"PAUSE 1000"` into `state.event_buffer`. Frontend (Phase 25/26) will read the marker and insert the ~1s delay; core stays clock-free. Display_override survives subsequent run_loop iterations (run_loop bypasses dispatch, so the dispatch-top clear at `mod.rs:410` does not fire mid-program) and is cleared by the next interactive dispatch — matches HP-41 "value visible until next key" behavior.
- **Inline indirect branching:** `Op::GtoInd(u8)` and `Op::XeqInd(u8)` ship with a 6-step inline resolver — bounds-safe register read via `.get().ok_or(InvalidOp)?`, integer-truncate-then-equality-check for non-integer reject (FN-IND-02 hardware semantic), stringify via `int_part.inner().to_string()`, then reuse `find_in_program`. XEQ IND additionally performs the pre-mutation 4-deep call-stack guard (matches `Op::Xeq` precedent at line 206) — over-deep returns `HpError::CallDepth` before any state changes.
- **4-place rule honored:** All four new variants land in `ops/mod.rs` Op enum + `dispatch()` + `execute_op()` (in catch-all for Stop/GtoInd/XeqInd; explicit arm for Pse) + both `prgm_display.rs` copies (CLI and GUI). Compile-time exhaustive-match coverage is intact.
- **Test coverage:** 15 integration tests pass, each FN-ID in scope has at least one named test, and all three RESEARCH §2 pitfalls have explicit sentinel tests. `just ci` green; coverage stays at 92.40% lines / 90.19% regions on hp41-core (≥ 80% gate).

## Task Commits

Each task was committed atomically on `worktree-agent-a891c99d8d3e4bccd`:

1. **Task 22-01-01: Op::Stop variant + dispatch + run_loop break + catch-all + prgm_display ×2** — `e7468c3` (feat)
2. **Task 22-01-02: Op::Pse variant + dispatch + execute_op two-channel write + prgm_display ×2** — `10d857b` (feat)
3. **Task 22-01-03: pub fn resume_program() with let-result reset-on-Err pattern** — `6dbbb87` (feat)
4. **Task 22-01-04: Op::GtoInd(u8) inline indirect resolver + run_loop arm + catch-all + prgm_display ×2** — `7302b5d` (feat)
5. **Task 22-01-05: Op::XeqInd(u8) with pre-mutation 4-deep guard + run_loop arm + catch-all + prgm_display ×2** — `c33c463` (feat)
6. **Task 22-01-06: hp41-core/tests/phase22_program_control.rs — 15 integration tests** — `9f7b94a` (test)

Plan metadata (this SUMMARY): committed separately as `docs(22-01)`.

## Files Created/Modified

### Created
- `hp41-core/tests/phase22_program_control.rs` (348 lines) — 15 integration tests covering FN-PROG-01/02/06/07 + the three Pitfall sentinels.

### Modified
- `hp41-core/src/ops/mod.rs` — 4 new Op variants appended at end of Op enum (preserve discriminant order per D-22.22); 4 new dispatch arms (Stop interactive Neutral no-op; Pse inline two-channel write; GtoInd/XeqInd return InvalidOp).
- `hp41-core/src/ops/program.rs` — 3 new run_loop arms (Op::Stop bare `break`; Op::GtoInd 6-step inline resolver; Op::XeqInd pre-mutation guard + same resolver + call_stack push); 1 new execute_op arm (Op::Pse inline two-channel write); extended programming-ops catch-all with Op::Stop, Op::GtoInd(_), Op::XeqInd(_); new public function `resume_program()` placed adjacent to `run_program`.
- `hp41-core/src/lib.rs` — added `resume_program` to the `pub use ops::program::{...}` re-export tuple.
- `hp41-cli/src/prgm_display.rs` — 4 new arms: `Op::Stop => "STOP"`, `Op::Pse => "PSE"`, `Op::GtoInd(r) => format!("GTO IND {r:02}")`, `Op::XeqInd(r) => format!("XEQ IND {r:02}")`.
- `hp41-gui/src-tauri/src/prgm_display.rs` — same 4 arms, intentional duplication per CLAUDE.md SC-4 invariant.

## Decisions Made

- **`HpNum::inner()` returns `Decimal` by value, not a reference.** The CONTEXT.md sketch at line 610 reads `int_part.inner().to_string()` which I used as-is; the call works because `Decimal: Copy + Display`. No code change required, just noted here for Phase 24's resolver extraction.
- **`Op::Pse` body is inlined in both `dispatch()` and `execute_op()` rather than delegating to a new `op_pse()` helper.** Matches PATTERNS.md sketch (lines 207–215); avoids adding a new function for a 4-line body. Phase 24 may revisit if a delegating helper becomes useful.
- **Resume preserves `call_stack` (no `.clear()` call).** Documented in the function's doc-comment with a forward reference to "STOP-inside-subroutine + R/S resumes correctly with RTN still working".
- **`builtin_card_op` fallback intentionally NOT replicated for indirect ops.** The integer pointer route resolves a numeric label string only; textual function-name fallback is meaningless for indirect branching. Verified against the locked PATTERNS.md line 182 note.

## Deviations from Plan

None — plan executed exactly as written. All 6 tasks landed in order with no deviation rules triggered. No bugs found; no missing critical functionality discovered; no blocking issues; no architectural changes needed.

The only "surprise" was that `HpNum`'s tuple-struct constructor `HpNum(Decimal)` is `pub(crate)` and therefore not callable from integration tests (which are an external crate). The fix was to use the public `HpNum::rounded(Decimal::from_str("...").unwrap())` constructor throughout the test file — this is the established pattern in `tests/phase21_flags.rs` and other integration tests. Caught immediately by the first `cargo test` invocation; no commit churn.

## Issues Encountered

- **`just ci` flagged a clippy `doc_overindented_list_items` warning** in the test file's module-header doc-comment (3-space continuation indent on a `//!` list item). Fixed by collapsing to 2-space continuation per the lint's suggestion. Same commit as the test file itself (test was rewritten before commit so no separate fix commit was needed). One-line lint, no functional impact.

## User Setup Required

None — entirely additive on `hp41-core`. No new dependencies, no env vars, no service config. R/S keyboard wiring (CLI in Phase 25, GUI in Phase 26) will surface the new ops to end users; Phase 22 just lands them in core.

## Next Phase Readiness

**Plan 22-02 (program-edit) is unblocked.** It will pile `Op::Clp / Op::Del / Op::Ins` onto `ops/mod.rs` + `program.rs` catch-all + both `prgm_display.rs` copies using the same 4-place rule pattern this plan exercised. The plan-22-02 file overlaps `ops/mod.rs` and `program.rs` with 22-01 — git merge will be trivial (additive lines at the end of the Op enum + new arms in the existing match blocks).

**Phase 24 (indirect-addressing) has a clean refactor target.** The 6-step inline resolver in Op::GtoInd / Op::XeqInd arms is intentionally duplicated. Phase 24's `resolve_indirect(state, reg) -> Result<u8, HpError>` helper extracts steps 1–4 (register read + integer truncate + non-integer reject + stringify); the run_loop arms then become 2 lines each.

**Phases 25 (CLI keyboard) and 26 (GUI key_map) can wire R/S to `resume_program()` immediately.** The v2.1 cosmetic `run_stop` Tauri command currently toggles `is_running` without effect — Phase 26 swaps the implementation to call `resume_program(state)` when `is_running == false` and to set a stop-requested sentinel (Phase 22 deferred) when `is_running == true`.

## Self-Check: PASSED

Files claimed created/modified verified present:
- `hp41-core/tests/phase22_program_control.rs` — FOUND (348 lines)
- `hp41-core/src/ops/mod.rs` — FOUND (modified, contains Op::Stop, Op::Pse, Op::GtoInd, Op::XeqInd)
- `hp41-core/src/ops/program.rs` — FOUND (modified, contains pub fn resume_program)
- `hp41-core/src/lib.rs` — FOUND (re-exports resume_program)
- `hp41-cli/src/prgm_display.rs` — FOUND ("STOP", "PSE", "GTO IND", "XEQ IND" arms present)
- `hp41-gui/src-tauri/src/prgm_display.rs` — FOUND (same 4 arms present)

Commit hashes verified present on `worktree-agent-a891c99d8d3e4bccd`:
- `e7468c3` — feat(22-01): Op::Stop ✓
- `10d857b` — feat(22-01): Op::Pse ✓
- `6dbbb87` — feat(22-01): resume_program ✓
- `7302b5d` — feat(22-01): Op::GtoInd ✓
- `c33c463` — feat(22-01): Op::XeqInd ✓
- `9f7b94a` — test(22-01): phase22_program_control.rs ✓

Quality gates verified green:
- `cargo check -p hp41-core -p hp41-cli` — exit 0
- `cd hp41-gui/src-tauri && cargo check` — exit 0
- `cargo clippy -p hp41-core -p hp41-cli --all-targets -- -D warnings` — exit 0
- `cargo test -p hp41-core --test phase22_program_control` — 15 passed, 0 failed
- `just ci` — exit 0 (clippy + fmt + workspace tests + coverage); hp41-core 92.40% lines / 90.19% regions
- Zero `.unwrap()` / `panic!()` in production code (all new code uses `?`-propagation or bounds-safe `.get().ok_or(InvalidOp)?`)

---
*Phase: 22-program-control-and-memory-ops*
*Plan: 01-program-control*
*Completed: 2026-05-14*
