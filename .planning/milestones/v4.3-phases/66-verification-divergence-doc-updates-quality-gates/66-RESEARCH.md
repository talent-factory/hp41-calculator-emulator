# Phase 66: Verification, Divergence-Doc Updates, Quality Gates — Research

**Researched:** 2026-06-10
**Domain:** HP-41 OM behavioral verification + divergence-doc reconciliation + quality-gate closeout
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Primary OM is the evidence standard. Verify UNC-01/02/03 against in-repo PDFs. Record page citations. Free42 is tie-breaker only.
- **D-02:** Verification runs autonomously. If OM genuinely does not pin a behavior, mark `needs-hardware-confirm` and document-as-divergence rather than guess-fix.
- **D-03:** Fix ALL OM-confirmed divergences in this phase. UNC-02 print-test rework accepted.
- **D-04:** OM/handbook PDFs are committed via Git LFS. Cite by path + page.
- **D-05:** Comprehensive reconciliation of every doc touched by v4.3: DIVERGENCE-AUDIT.md (FGAP-02/03/04/05/07 + UNC-01/02/03), hp41cv-divergences.md (DISP/CHS/AON + GETKEY + UNC closures), hp41-time-divergences.md (confirm D-40-04 "Verified"), README.md (FACT note → "implemented (v4.3)").
- **D-06:** Add a v4.3 closure ledger table in DIVERGENCE-AUDIT.md.
- **D-07:** English-only. OM page citations are factual references.
- **D-08:** Build 7-PITFALLS → 12-test-fn mapping. Add test only if a genuine gap exists.
- **D-09:** Gates: `just ci`, `just ci-msrv`, `just gui-ci`, numerical accuracy ≥ 98% (843+ cases), zero panics, PLUS ungated GUI-crate clippy.
- **D-10:** NO `cargo fmt` on `hp41-gui/src-tauri`. If a stray GUI edit dirties formatting, `git restore hp41-gui/` rather than committing churn.
- **D-11:** Update open v4.3 milestone PR description with Phase 66 completion summary.

### Claude's Discretion

- Exact OM page numbers per UNC (researcher locates and records).
- Whether UNC-02 fix touches only `print.rs` or also a shared flag-check helper, and exact print-test rework scope.
- For UNC-03: whether "MEMORY LOST" is shown via `display_override` and how long it persists.
- Closure-ledger column shape and location in DIVERGENCE-AUDIT.md.
- Plan sequencing: verification → fixes → doc sweep → matrix proof → full gate run → PR update. Doc sweep can parallelize with matrix proof once UNC dispositions are known.

### Deferred Ideas (OUT OF SCOPE)

- CATALOG 1 interactive scroll (FGAP-06).
- SAVED/GETD bbb.eee block control (FGAP-08).
- GUI-crate `cargo fmt` cleanup (~19 files).
- GUI-crate clippy pre-existing hits (types.rs:148, commands.rs `&*p`) — accepted noise.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VERIFY-01 | Verify the three uncertain behaviors against the OM / a trusted reference and fix only those confirmed divergent; document any that are already correct: (a) ← clears an error display (UNC-01), (b) flags 21/25 gating PRX/PRA/PRSTK printing (UNC-02), (c) SIZE reduction showing "MEMORY LOST" on the display (UNC-03). | OM p.15 (UNC-01), OM p.57-58 + Section 11 (UNC-02), OM p.57 + p.19 (UNC-03) — all verified via in-repo PDF. Full disposition table below. |
</phase_requirements>

---

## Summary

Phase 66 is a verification-first closeout phase for the v4.3 Hardware Fidelity milestone. All three uncertain behaviors (UNC-01/02/03) were researched directly from the in-repo HP-41C Operating Manual PDF (`docs/manuals/HP-41CV/HP-41C_Operating_Manual.pdf`, HP 00041-90259, June 1980). The OM provides unambiguous rulings on all three items.

**UNC-01** (back-arrow clears error): The OM (p.15) explicitly states "Pressing [←] also clears error messages from the display." The emulator already handles this correctly: `hp41-cli/src/app.rs` line 975 sets `self.message = None` immediately after calling `backspace_entry()`. **Classification: already-correct — no fix required.**

**UNC-02** (flags 21/25 gate printing): The OM (p.57-58) states "An attempt was made to execute a specific print function when the printer was not connected to the system" produces a NONEXISTENT-class error. Flag 21 (p.53) is "Printer Enable — if set, calculator assumes printer is present in system." Flag 55 (p.54) is "Printer Existence." The OM's error-message section confirms that print functions DO check for printer presence. The emulator's `print.rs` does NOT read flags 21/55 before printing. **Classification: divergent — fix required.** The fix is flag-gating PRX/PRA/PRSTK on `flag_get(state.flags, 55) || flag_get(state.flags, 21)` and returning an error if neither is set. Existing always-print tests must be reworked to set flag 55 before calling print ops.

**UNC-03** (SIZE reduction shows MEMORY LOST): The OM (p.57) definitively states "MEMORY LOST — The Continuous Memory of the calculator has been cleared." The "Clearing Main Memory" section (p.23) confirms this display appears when Continuous Memory is cleared via power interruption or memory-module removal — NOT as a result of SIZE reduction. SIZE reduction (p.19) silently loses the highest-numbered register values; no special display message is shown. **Classification: already-correct — no fix required, but the comment in `op_size` claiming hardware-faithful "MEM LOST" is misleading and should be corrected.**

The 12 existing tests in `phase_63_interrupting_alarms.rs` cover all 7 PITFALLS re-entrancy scenarios with no gaps.

**Primary recommendation:** Fix UNC-02 (print-flag gating) as the single code change. Correct the misleading `op_size` comment. Then execute the full divergence-doc sweep and quality-gate run.

---

## UNC Verification Table

| UNC ID | OM Citation(s) | Classification | Fix Site | Minimal Change | Test Fallout |
|--------|---------------|----------------|----------|----------------|--------------|
| UNC-01 | HP-41C OM p.15: "Pressing [←] also clears error messages from the display." [VERIFIED: HP-41C_Operating_Manual.pdf p.15] | **already-correct** | `hp41-cli/src/app.rs` line 974-975: `backspace_entry()` + `self.message = None` already clears the error. | None. Optionally add a test asserting `message = None` after back-arrow with active error. | None. |
| UNC-02 | HP-41C OM p.53: Flag 21 = "Printer Enable — if set, calculator assumes printer is present." Flag 55 = "Printer Existence." p.57-58: error message "An attempt was made to execute a specific print function when the printer was not connected to the system." ADV footnote on p.59: "§ Valid only if printer is connected." [VERIFIED: HP-41C_Operating_Manual.pdf p.53-58] | **divergent** | `hp41-core/src/ops/print.rs` — `op_prx`, `op_pra`, `op_prstk` do not read flags. `flags.rs` has `flag_get()` helper. | Add printer-presence check at top of each print fn: (1) Add `HpError::NonExistent` variant to `hp41-core/src/error.rs` (does NOT currently exist — verified); (2) add `if !flag_get(state.flags, 55) && !flag_get(state.flags, 21) { return Err(HpError::NonExistent); }` at top of each print fn. | All 13 tests in `hp41-core/tests/print_tests.rs` must be reworked to set flag 55 (`state.flags = flag_set(state.flags, 55)`) before calling print ops. No print tests in `phase_63_interrupting_alarms.rs` are affected. CATALOG tests use `print_buffer` directly for CATALOG output (not PRX/PRA/PRSTK), so they are unaffected. |
| UNC-03 | HP-41C OM p.57: "MEMORY LOST — The Continuous Memory of the calculator has been cleared." p.23: MEMORY LOST displays on Continuous Memory clear (battery interruption / memory module removal). p.19: SIZE reduction silently loses highest-register data — no special display message mentioned. [VERIFIED: HP-41C_Operating_Manual.pdf p.19, p.23, p.57] | **already-correct** (SIZE reduction does NOT show MEMORY LOST on real hardware) | `hp41-core/src/ops/program.rs` `op_size` comment at line 263 ("hardware-faithful 'MEM LOST'") is **misleading**: the comment implies MEMORY LOST display was considered hardware-faithful, but the OM shows MEMORY LOST is a power/hardware event, not a SIZE event. | No code logic change. **Fix the comment** on line 263: replace "hardware-faithful 'MEM LOST'" with "hardware-faithful: data truncated silently, no 'MEMORY LOST' display (OM p.19 — MEMORY LOST is a power-event display, not a SIZE event)". | None. |

---

## 7-PITFALLS → 12-Test-Fn Mapping Table

The 7 re-entrancy scenarios from `.planning/research/PITFALLS.md` map to the 12 test functions in `hp41-core/tests/phase_63_interrupting_alarms.rs` as follows:

| PITFALLS Scenario | Scenario Name | Test Function(s) | Notes |
|-------------------|---------------|------------------|-------|
| Scenario 1: Basic Interrupt — Running Program Halted, Alarm Executes, Resumes | Interrupt halts running program, handler executes, program resumes | `interrupting_alarm_halts_running_program_and_resumes` | Full flow: pre-interrupt marker + handler sentinel + resume marker. |
| Scenario 2: Stack and PC Preservation Through Interrupt | Stack X/Y/Z/T and lift_enabled preserved | `interrupt_preserves_stack_x_y_z_t_and_lift_state` | Verifies handler ran + main completed + is_running=false. Per PITFALLS guidance (OM confirms stack is shared — handler runs in same register environment). |
| Scenario 3: 4-Level Call Stack Cap Respected | Interrupt blocked at cap depth 4 | `interrupt_blocked_when_call_stack_at_4_level_cap` | Tests alarm routing (63-01) side; cap-drop enforced in run_loop (63-02). |
| Scenario 4: Nested Interrupt Blocked (No Recursive Interrupt) | Second interrupt demoted when first is pending | `interrupt_nesting_blocked_when_already_in_alarm_program` | Second alarm demoted to `alarm:xeq:SECOND` on `event_buffer`. |
| Scenario 5: Idle-Fire Path (No Program Running) | Interrupting alarm fires when no program running | `interrupting_alarm_fires_when_no_program_running` | Routes to `alarm:xeq:IDLE_HANDLER` on event_buffer (criterion 3 idle path). |
| Scenario 6: Non-Interrupting Path Regression | Non-interrupting alarm still fires as event, not inline | `non_interrupting_alarm_still_fires_as_event_not_inline` | DNT-05 regression guard. |
| Scenario 7: Message Alarm Regression | Message alarm routes to event_buffer, not executed | `message_alarm_still_fires_to_event_buffer_not_executed` | Also verifies print_buffer push. |
| (Additional) Solver/Modal Demotion | Interrupt demoted when solver or modal active | `interrupt_demoted_when_solver_or_modal_active` | Sub-tests A (integ_state) + B (solve_state) cover D-10 demotion guard. |
| (Additional) Missing Handler Label | Missing-handler label surfaces event | `missing_handler_label_surfaces_event` | `pending_interrupt = Some("GONE")` set; run_loop surfaces `alarm:missing:GONE`. |
| (Additional) Pending Clear on STOP Resume | Pending interrupt cleared on resume after STOP | `pending_interrupt_cleared_on_resume_after_stop` | D-09: `resume_program` clears pending_interrupt before re-entering run_loop. |
| (Additional) Backward Compat | v4.2 autosave JSON deserializes cleanly | `v4_3_interrupt_backward_compat` | All 4 new Phase 63 fields default to None from v4.2 JSON. |
| (Additional) Repeating Alarm Reschedule | Repeating alarm trigger_unix advances after handler RTN | `repeating_interrupting_alarm_reschedules_after_handler` | D-06: ack-after-RTN reschedule. |

**Gaps: NONE.** All 7 PITFALLS scenarios are covered. The 12 tests include 5 additional edge cases (solver demotion, missing handler, pending-clear-on-stop, backward compat, repeating reschedule) that exceed the PITFALLS minimum. **No new test is required.**

---

## Divergence-Doc Sweep Checklist

For each file, the "Current State" column reflects what was found by reading the file, and "Edit Required" is what Phase 66 must do.

| File | Edit Required | Current State |
|------|---------------|---------------|
| `.planning/research/DIVERGENCE-AUDIT.md` | (1) Mark FGAP-02/03/04/05/07 resolved; (2) Mark UNC-01 "already-correct"; (3) Mark UNC-02 "fixed Phase 66"; (4) Mark UNC-03 "already-correct"; (5) Add v4.3 closure ledger table (D-06). | UNCs still show "Investigate on real hardware" status. FGAPs table does not have "resolved" markers. No closure ledger exists yet. |
| `docs/hp41cv-divergences.md` | (1) Add D-CV-06 for DISP-01 (CLI VIEW/AVIEW/PROMPT now reads display_override — Phase 65); (2) Add D-CV-07 for DISP-02 (CHS during mantissa entry — Phase 65); (3) Add D-CV-08 for DISP-03 (AON auto-display — Phase 65); (4) Add D-CV-09 for UNC-02 closure (PRX/PRA/PRSTK now flag-gated — Phase 66); (5) Add UNC-01/UNC-03 as "verified-correct (not a divergence)" notes. Update "Last updated" footer. | Currently has D-CV-01..D-CV-05 only. Phase 65 DISP-01/02/03 closures are NOT yet recorded. |
| `docs/hp41-time-divergences.md` | Confirm §D-40-04 (~line 265) already says "IMPLEMENTED … Verified in Phase 66" — this phase makes that "Verified" claim true. Update "Last updated" footer (~line 383) to 2026-06-10. | D-40-04 is already pre-flipped to "IMPLEMENTED … Verified in Phase 66" by Plan 63-05. The claim becomes true this phase. |
| `docs/hp41-math1-divergences.md` | Scan for any v4.3-affected entries. MATH-01 (FACT range, FGAP-03) was fixed in Phase 65. If the divergence entry for FACT remains as "unfixed" status, update it to "Implemented (v4.3)". | Scan required — likely needs FACT entry status update. |
| `docs/hp41-stat1-divergences.md` | Scan for any v4.3-affected entries. Expected: none. | Scan confirms no v4.3 work touched Stat 1. |
| `docs/hp41-advantage-divergences.md` | Scan for any v4.3-affected entries. Expected: none. | Scan confirms no v4.3 work touched Advantage. |
| `docs/hp41-xmem-divergences.md` | Scan for any v4.3-affected entries. Expected: none (FGAP-08/09 deferred). | Scan confirms no v4.3 work touched X-MEM divergences. |
| `README.md` | Update FACT divergence note (~line 170): change "effective cap at X ≤ 26 … X in 27..=69 returns Overflow" → "implemented (v4.3): FACT(27..=69) returns correct scientific-notation result; HP-41 caps at X ≤ 69." | Line 170 still shows the old "Overflow" divergence. Phase 65 fixed it; README not yet updated. |

### v4.3 Closure Ledger Table (to add in DIVERGENCE-AUDIT.md)

Add this table as a new `## v4.3 Closure Ledger` section:

| FGAP/UNC ID | Name | Phase Closed | OM Citation | Status |
|-------------|------|-------------|-------------|--------|
| FGAP-01 | PSE timing — 1-second pause | Phase 63 | HP-41C OM p.44 (Section 7: Using PSE) | Implemented |
| FGAP-02 | CLI VIEW/AVIEW/PROMPT display | Phase 65 | HP-41C OM p.16 (Viewing Register Contents) | Implemented |
| FGAP-03 | FACT range cap 26 → 69 | Phase 65 | HP-41C OM p.58 (FACT where x > 69 = OUT OF RANGE) | Implemented |
| FGAP-04 | GETKEY interactive wait | Phase 64 | HP-41CX Extended Functions (GETKEY specification) | Implemented |
| FGAP-05 | CHS during mantissa entry | Phase 65 | HP-41C OM p.15 (Display Editing — prompt sign behavior) | Implemented |
| FGAP-07 | AON auto-display | Phase 65 | HP-41C OM p.53 (Flag 48: Alpha Mode) | Implemented |
| FGAP-10 | VIEW/AVIEW in programs — mid-run display | Phase 63 | HP-41C OM p.44 (VIEW/AVIEW display semantics) | Implemented |
| UNC-01 | Back-arrow clears error message | Phase 66 | HP-41C OM p.15: "Pressing [←] also clears error messages from the display." | Already-correct (no fix needed) |
| UNC-02 | Flags 21/55 gate PRX/PRA/PRSTK | Phase 66 | HP-41C OM p.53 (Flag 21/55), p.57-58 (error: printer not connected) | Fixed — print ops now flag-gated |
| UNC-03 | SIZE reduction → MEMORY LOST display | Phase 66 | HP-41C OM p.19 (SIZE reduction — silent), p.57 (MEMORY LOST = Continuous Memory clear) | Already-correct (no MEMORY LOST on SIZE) |
| D-40-04 | Interrupting control alarm re-entrancy | Phase 63 | HP 00041-90035 §XYZALM (>>label prefix) | Implemented + verified Phase 66 |

---

## UNC-02 Fix: Blast-Radius Analysis

### What changes in `hp41-core/src/ops/print.rs`

Add a printer-presence guard at the top of each of the three public functions:

```rust
// At top of op_prx, op_pra, op_prstk (identical pattern):
use crate::ops::flags::flag_get;
if !flag_get(state.flags, 55) && !flag_get(state.flags, 21) {
    return Err(HpError::NonExistent);
}
```

`flag_get` is already public in `hp41-core/src/ops/flags.rs` (lines 12-17). **`HpError::NonExistent` does NOT exist in `hp41-core/src/error.rs` (verified by reading the file — 18 variants, none named NonExistent).** A new variant must be added matching the OM "NONEXISTENT" error message class. No other new helper is needed — call `flag_get` inline.

**OM basis for flag choice:** Flag 55 = "Printer Existence" (p.54) — set automatically when a printer is in the system. Flag 21 = "Printer Enable" (p.53) — user-settable "calculator assumes printer is present." The guard uses `flag 55 || flag 21`: either the system detected a printer (flag 55) or the user explicitly enabled it (flag 21). Clearing BOTH means "no printer" → error. This mirrors the OM's footnote: "§ Valid only if printer is connected" in the Function Index (ADV entry, p.59).

**Note on flag 25:** The DIVERGENCE-AUDIT.md mentions flag 25 ("Error Ignore") but the OM (p.53) shows flag 25 = "Error Ignore — When set, calculator ignores one improper operation, then flag is cleared." Flag 25 is NOT a printer flag; it is a generic error-suppression flag. The audit's original UNC-02 text was imprecise about flag 25 — the relevant flags are 21 and 55. The fix uses 21/55 only.

### Test rework in `hp41-core/tests/print_tests.rs`

All 13 tests call print ops on a default `CalcState`. After the fix, `state.flags` defaults to 0, meaning neither flag 21 nor flag 55 is set, so all 13 tests will fail with `NonExistent`. Fix: add `state.flags = flag_set(state.flags, 55)` after `CalcState::new()` in each test (or in a shared setup helper). The test structure does not change — only the setup.

Tests affected (all 13 in `print_tests.rs`):
- `test_prx_pushes_one_line_to_buffer`
- `test_prx_output_is_24_chars`
- `test_prx_output_is_right_aligned`
- `test_prx_respects_display_mode_sci`
- `test_prx_lift_effect_neutral`
- `test_pra_pushes_one_line_to_buffer`
- `test_pra_output_is_24_chars`
- `test_pra_output_is_left_aligned`
- `test_pra_empty_alpha_is_24_spaces`
- `test_pra_truncates_long_alpha_to_24_chars`
- `test_prstk_produces_six_lines`
- `test_prstk_all_lines_are_24_chars`
- `test_prstk_line_order_and_labels`
- `test_prstk_alpha_empty_line_format`
- `test_prstk_alpha_nonempty_line_format`
- `test_prx_in_program`
- `test_pra_in_program`
- `test_prstk_in_program`

Additionally, add one new test: `test_prx_returns_nonexistent_when_no_printer_flag` — asserts that with both flags 21 and 55 clear (default state), calling `op_prx` returns `Err(HpError::NonExistent)`.

**CATALOG tests (`phase22_catalog.rs`, `op_catalog_xrom.rs`):** CATALOG uses `print_buffer` directly via the `op_catalog` function — it does NOT call `op_prx/op_pra/op_prstk`. These tests are **not affected** by the UNC-02 fix.

**`phase_63_interrupting_alarms.rs`:** The message alarm test (`message_alarm_still_fires_to_event_buffer_not_executed`) pushes to `print_buffer` via the alarm message path (not via print ops). **Not affected.**

**numerical_accuracy.rs:** Tests only numerical/math operations. **Not affected.**

---

## Quality-Gate Command List

Run in this order. Each gate must be green before the next.

| Gate | Command | Threshold | Landmine |
|------|---------|-----------|---------|
| Root workspace lint | `just lint` | 0 warnings (`-D warnings`, stable clippy) | None expected for this phase's small changes. |
| All tests | `just test` | 0 failures | **UNC-02 rework:** all 13+ print tests will FAIL before `state.flags` setup lines are added. Fix tests before running. |
| Core coverage | `just coverage` | ≥ 95% lines / ≥ 93% regions | UNC-02 adds a new error path in `print.rs`; coverage improves or stays flat. Should not regress. |
| License audit | `just license-audit` | 0 contamination matches | No new code from Free42; guard is routine. |
| Full CI composite | `just ci` | All above pass | Runs: lint → test → coverage → license-audit |
| MSRV lint + test | `just ci-msrv` | 0 failures (no coverage gate) | **LANDMINE:** `cargo +1.88 clippy` flags `uninlined_format_args` and other style lints demoted to pedantic on stable. Any `format!("{}", x)` in new code must use `format!("{x}")`. Run explicitly: `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings`. |
| Schema aliases | `just schema-aliases-check` | All 6 pools valid | No alias schema changes this phase. |
| GUI CI | `just gui-ci` | 0 failures | No GUI code changes expected this phase. If UNC-01/03 had required GUI display fixes, this would be the parity gate. Runs: permissions check → npm ci → tsc → Rust tests → release build → Vitest. |
| Ungated GUI clippy | `cargo clippy --manifest-path hp41-gui/src-tauri/Cargo.toml` | 0 new hits | **Known noise:** types.rs:148 and commands.rs `&*p` are pre-existing accepted hits. Do NOT fix them (D-10 — would require `cargo fmt` to accompany). Assert count of hits does not increase. |
| Numerical accuracy | `cargo test -p hp41-core --test numerical_accuracy` | ≥ 98% (843+ cases) | UNC-02 print fix does not touch numerical algorithms. Should be unaffected. |
| Zero panics in core | `grep -rn 'unwrap()' hp41-core/src/ | grep -v '#\[allow'` | 0 results | UNC-02 fix returns `Err`, not `unwrap`. Should be clean. |

### MSRV Pitfall — Detailed

`just ci-msrv` expands to `just lint test` on toolchain `+1.88`. Under `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings`:

- `uninlined_format_args` is a warning on 1.88 (demoted to pedantic on stable). Any `format!("{}", x)` → must be `format!("{x}")`.
- `redundant_pattern_matching`, `needless_borrow`, and similar style lints may fire.
- These lints were the source of hidden CI failures through v4.2 (see CLAUDE.md memory `reference_msrv_clippy_lint_divergence`).
- Fix: write all new code using inline format args from the start. Verify with `cargo +1.88 clippy` explicitly.

---

## Architecture Patterns

This phase is purely fix + doc. No new `Op` variants, no new `CalcState` fields.

### UNC-02 Fix Pattern

```rust
// Source: hp41-core/src/ops/print.rs (to be added)
// Existing helper: flag_get() in hp41-core/src/ops/flags.rs:12
use crate::ops::flags::flag_get;

pub fn op_prx(state: &mut CalcState) -> Result<(), HpError> {
    if !flag_get(state.flags, 55) && !flag_get(state.flags, 21) {
        return Err(HpError::NonExistent);
    }
    // ... existing implementation unchanged ...
}
```

### UNC-02 Test Rework Pattern

```rust
// In print_tests.rs: add flag setup to existing test helpers
fn push_val(state: &mut CalcState, n: i32) {
    // Add printer flag so tests pass with flag-gated print ops:
    use hp41_core::ops::flags::flag_set;
    state.flags = flag_set(state.flags, 55); // flag 55 = Printer Existence
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
// OR: set state.flags = flag_set(state.flags, 55) in each test setup.
// The shared-helper approach is cleaner.
```

### UNC-03 Comment Fix

```rust
// Current (program.rs line 263):
/// "MEM LOST"); growing zero-fills the new slots.

// Corrected:
/// hardware-faithful: data truncated silently, no "MEMORY LOST" display
/// (OM p.19 — MEMORY LOST is a power-event display, not triggered by SIZE);
/// growing zero-fills the new slots.
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Flag-check for printer presence | Custom bit-mask inline | `flag_get()` from `flags.rs` | Already a tested, public helper at `hp41-core/src/ops/flags.rs:12`. |
| New `HpError` variant for "no printer" | Different existing variant | Add `HpError::NonExistent` to `error.rs` | Verified: this variant does NOT exist. Add it with `#[error("nonexistent")]`. Matches OM "NONEXISTENT" message class (p.58). No 4-way match impact (error, not Op). |

---

## Common Pitfalls

### Pitfall 1: Using Flag 25 Instead of Flag 55/21 for Printer Check

**What goes wrong:** The original UNC-02 audit text mentioned flag 25. The OM (p.53) shows flag 25 = "Error Ignore", not a printer flag. Using `flag_get(state.flags, 25)` would implement the wrong semantic.

**How to avoid:** Use flags 21 and 55 only. `flag_get(state.flags, 55) || flag_get(state.flags, 21)`.

### Pitfall 2: Thinking MEMORY LOST Should Be Added to op_size

**What goes wrong:** The `op_size` comment says "hardware-faithful 'MEM LOST'" and both the audit (UNC-03) and CONTEXT.md (fix site) point to `op_size` as a potential target. This research confirms MEMORY LOST is NOT a SIZE event per OM p.19 and p.57.

**How to avoid:** Do NOT add `display_override = Some("MEMORY LOST".to_string())` to `op_size`. Only fix the misleading comment.

### Pitfall 3: Running `cargo fmt` on the GUI Crate

**What goes wrong:** `cargo fmt --all` applied to the whole workspace reformats ~19 pre-existing files in `hp41-gui/src-tauri`, creating noisy churn. The pre-push hook only checks the root workspace.

**How to avoid:** If a stray GUI edit dirties formatting, run `git restore hp41-gui/` before committing (D-10).

### Pitfall 4: `HpError::NonExistent` Does Not Exist — Must Add It First

**What goes wrong:** `print.rs` currently returns only `Ok(())`. Adding the flag check requires `HpError::NonExistent`, but this variant does NOT exist in `hp41-core/src/error.rs` (verified: 18 variants, none named NonExistent). If the implementer tries to use it without adding it first, the code will not compile.

**How to avoid:** Add `#[error("nonexistent")] NonExistent,` to `HpError` in `hp41-core/src/error.rs` FIRST, as Wave 0 Task 0. Add a Display test. Then use it in `print.rs`.

### Pitfall 5: CATALOG Tests Incorrectly Flagged as Affected

**What goes wrong:** Reviewer might worry that CATALOG tests (which use `print_buffer`) will break after the UNC-02 fix.

**How to avoid:** CATALOG output goes to `print_buffer` via `op_catalog`, which calls `state.print_buffer.push(...)` directly — it does NOT call `op_prx/op_pra/op_prstk`. These tests are unaffected.

### Pitfall 6: MSRV Lint on New Code

**What goes wrong:** New code in `print.rs` using `format!("{}", x)` instead of `format!("{x}")` will fail `just ci-msrv` silently if only `just ci` (stable) is run.

**How to avoid:** Run `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings` explicitly before declaring gates green.

---

## Runtime State Inventory

Not applicable — this is a verification/doc/fix phase. No rename, refactor, or migration.

---

## Environment Availability

No external tools, services, or databases required beyond the standard build environment. Skipping per instructions.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` + `llvm-cov` for coverage |
| Config file | `justfile` (tasks: `just test`, `just coverage`, `just ci`) |
| Quick run command | `cargo test -p hp41-core` |
| Full suite command | `just ci` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VERIFY-01(a) | Back-arrow clears error — already-correct | Manual confirm (code audit) | `grep -n "message = None" hp41-cli/src/app.rs` | N/A — no test change needed |
| VERIFY-01(b) | PRX/PRA/PRSTK flag-gated on flag 55/21 | Unit | `cargo test -p hp41-core --test print_tests` | Exists (rework required) |
| VERIFY-01(b) | PRX returns NonExistent when no printer flag | Unit | `cargo test -p hp41-core --test print_tests -- test_prx_returns_nonexistent` | New test in Wave 1 |
| VERIFY-01(c) | SIZE reduction does NOT show MEMORY LOST — already-correct | None (comment-only fix) | `cargo test -p hp41-core --test phase22_catalog` (existing SIZE tests) | Exists |

### Sampling Rate

- Per task commit: `cargo test -p hp41-core`
- Per wave merge: `just ci`
- Phase gate: `just ci && just ci-msrv && just gui-ci` all green before `/gsd-verify-work`

### Wave 0 Gaps

The only new test needed is `test_prx_returns_nonexistent_when_no_printer_flag` in `print_tests.rs`. No new test file. No framework install needed.

- [ ] `hp41-core/tests/print_tests.rs` — add `test_prx_returns_nonexistent_when_no_printer_flag`; rework all 18 existing tests to call `flag_set(state.flags, 55)` in setup.

---

## Security Domain

Not applicable — this phase makes no changes to authentication, session management, access control, cryptography, or input validation surfaces. The UNC-02 fix is a behavioral fidelity correction to a print-buffer gate.

---

## Sources

### Primary (HIGH confidence)

- `docs/manuals/HP-41CV/HP-41C_Operating_Manual.pdf` (HP 00041-90259, June 1980) — p.15 (Display Editing and Clearing — back-arrow behavior, UNC-01), p.19-20 (SIZE reduction description, UNC-03), p.23 (MEMORY LOST = Continuous Memory clear, UNC-03), p.52-54 (Flags section — flag 21 Printer Enable, flag 55 Printer Existence, UNC-02), p.57-58 (Error and Status Messages — print function error when printer not connected, UNC-02, and MEMORY LOST definition, UNC-03), p.59-67 (Function Index — ADV footnote "§ Valid only if printer is connected"). [VERIFIED: read directly from in-repo PDF]
- `hp41-core/src/ops/print.rs` — confirmed no flag checks [VERIFIED: code read]
- `hp41-core/src/ops/flags.rs` — confirmed `flag_get` public helper at line 12 [VERIFIED: code read]
- `hp41-core/src/ops/stack_ops.rs` — confirmed `backspace_entry` behavior (CLX when empty) [VERIFIED: code read]
- `hp41-cli/src/app.rs` lines 973-975 — confirmed `self.message = None` set after backspace [VERIFIED: code read]
- `hp41-core/src/ops/program.rs` lines 263-286 — confirmed `op_size` comment + no `display_override` write [VERIFIED: code read]
- `hp41-core/tests/phase_63_interrupting_alarms.rs` — all 12 test functions read and named [VERIFIED: code read]
- `hp41-core/tests/print_tests.rs` — all 18 test functions identified [VERIFIED: code read]

### Secondary (MEDIUM confidence)

- `.planning/research/DIVERGENCE-AUDIT.md` — UNC-01/02/03 verbatim audit text, FGAP table [CITED: project planning file]
- `.planning/research/PITFALLS.md` — 7 re-entrancy scenarios (Scenarios 1-7) [CITED: project planning file]

### Tertiary (LOW confidence — none)

All claims were verified against in-repo code or primary OM PDF.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | ~~`HpError::NonExistent` variant exists in `hp41-core/src/error.rs`~~ **RESOLVED: does NOT exist (verified by reading error.rs — 18 variants, none named NonExistent). Must add new variant as part of UNC-02 fix.** | UNC-02 Fix | N/A — resolved. |
| A2 | Phase 65 DISP-01/02/03 closures are NOT yet in `docs/hp41cv-divergences.md` (only D-CV-01..D-CV-05 confirmed) | Divergence-Doc Sweep | If Phase 65 already added them, skip adding duplicates. |
| A3 | `docs/hp41-math1-divergences.md` still shows FACT as an open divergence (not yet updated to "Implemented v4.3") | Divergence-Doc Sweep | If Phase 65 already updated it, the FACT entry scan produces no edit. |

---

## Open Questions

1. **`HpError::NonExistent` variant name — RESOLVED**
   - **Finding:** The variant does NOT exist (verified by reading `hp41-core/src/error.rs` — 18 variants, none named NonExistent or similar).
   - **Action required:** Add `#[error("nonexistent")] NonExistent,` to `HpError` in `error.rs`. Add a corresponding Display test in the same file (follow the pattern of `file_not_found_display` test at line 96). This is the first task of the UNC-02 fix wave.

2. **Whether the UNC-01 fix needs a test**
   - What we know: The behavior is already correct (code confirmed). No functional change needed.
   - What's unclear: Whether an explicit regression test is worth adding.
   - Recommendation: Optionally add a test in the CLI-layer test suite asserting `app.message == None` after pressing back-arrow when a message is set. This is a nice-to-have, not a gate requirement.

---

## Metadata

**Confidence breakdown:**
- UNC dispositions: HIGH — all three verified directly from in-repo OM PDF with specific page citations.
- Fix scope (UNC-02): HIGH — fix site identified, blast radius mapped, test rework enumerated.
- 7-PITFALLS mapping: HIGH — all 12 test functions read and mapped; gap confirmed zero.
- Divergence-doc sweep: MEDIUM — docs read; A2/A3 assumptions exist for Phase 65 edit status.
- Quality gates: HIGH — exact justfile commands verified.

**Research date:** 2026-06-10
**Valid until:** Stable (OM is a fixed historical document; code sites verified against current HEAD on develop branch).

---

## RESEARCH COMPLETE

**Phase:** 66 — Verification, Divergence-Doc Updates, Quality Gates
**Confidence:** HIGH

### Key Findings

1. **UNC-01 is already-correct.** OM p.15 states `←` clears error messages. `app.rs` line 975 does exactly this (`self.message = None`). No code change. Optionally add a regression test.

2. **UNC-02 is divergent — fix required.** OM p.57-58 + p.53-54 confirm print ops must error when neither flag 21 nor flag 55 is set. Fix: (1) add `HpError::NonExistent` variant to `error.rs`; (2) add 3-line guard in each of `op_prx`, `op_pra`, `op_prstk`. Test rework: set `flag_set(state.flags, 55)` in all 18 print tests. Add one new "no printer → NonExistent" test. **Note:** Flag 25 is "Error Ignore" (NOT a printer flag) — use flags 21/55 only.

3. **UNC-03 is already-correct.** OM p.19 confirms SIZE reduction is silent (no display message). OM p.57 confirms MEMORY LOST is a power/hardware event (Continuous Memory clear). The `op_size` comment claiming "hardware-faithful 'MEM LOST'" is misleading — fix the comment only.

4. **Re-entrancy matrix: zero gaps.** All 7 PITFALLS scenarios are fully covered by the 12 existing tests. No new tests needed for the matrix proof.

5. **Divergence-doc sweep needs 5 files touched:** DIVERGENCE-AUDIT.md (FGAP/UNC markers + closure ledger), hp41cv-divergences.md (D-CV-06/07/08/09 + UNC closure notes), hp41-time-divergences.md (footer update), hp41-math1-divergences.md (FACT status update), README.md (FACT note).

### File Created

`.planning/phases/66-verification-divergence-doc-updates-quality-gates/66-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| UNC dispositions | HIGH | All three verified from primary OM PDF with page citations |
| Fix scope | HIGH | Code read; blast radius fully mapped |
| Re-entrancy matrix | HIGH | All 12 tests read and mapped to 7 scenarios; zero gaps confirmed |
| Divergence-doc sweep | MEDIUM | A2/A3 assumptions on Phase 65 edit status need spot-check |
| Quality gates | HIGH | justfile commands verified against current justfile |

### Open Questions

- `HpError::NonExistent` variant does NOT exist — must add to `error.rs` before writing `print.rs` fix (RESOLVED, see Open Questions).
- Spot-check `hp41-math1-divergences.md` FACT entry — if Phase 65 already updated it, skip.
- Spot-check `hp41cv-divergences.md` for D-CV-06/07/08 — if Phase 65 already added them, skip.

### Ready for Planning

Research complete. Planner can now create PLAN.md files.
