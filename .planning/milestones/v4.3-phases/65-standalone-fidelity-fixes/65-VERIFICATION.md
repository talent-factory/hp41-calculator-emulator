---
phase: 65-standalone-fidelity-fixes
verified: 2026-06-07T16:25:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
re_verification: false
human_verification:
  - test: "CLI VIEW/AVIEW/PROMPT display — interactive smoke test"
    expected: "After pressing VIEW 01 (or running a program with AVIEW), the CLI main display shows the formatted register name and value string, not the X register. Dismissed by CLD or starting number entry."
    why_human: "get_display_string reads display_override correctly (code verified, tests pass), but the end-to-end TUI render after an actual keypress can only be confirmed visually."
  - test: "CHS during mantissa entry — keyboard interaction"
    expected: "Typing '3', '.', '1', '4', then CHS shows '-3.14' in the display with no stack lift, no X register change. Pressing CHS again returns to '3.14'. Empty buffer CHS still negates X register."
    why_human: "The code path (app.rs lines 853-858) is correct and has 14 integration tests, but keystroke-level TUI responsiveness requires a human to confirm there are no timing or rendering artefacts."
  - test: "AON auto-display behavior after every operation"
    expected: "With AON active (SF 48), after pressing any key (e.g., +, ENTER, RCL), the main display shows the ALPHA register contents rather than the X register. AOF (CF 48) reverts to X. Both CLI and GUI show identical behavior."
    why_human: "The flag-48 branch in both get_display_string (CLI) and from_state (GUI) is code-verified. The claim 'after every operation' relies on the render loop re-evaluating the display after each dispatch, which is architecturally guaranteed but only visually confirmable."
  - test: "FACT(27..=69) scientific-notation output in the CLI and GUI"
    expected: "Entering 27 FACT shows approximately '1.088886945 28' in HP-41 SCI display; 69 FACT shows approximately '1.711224524 98'. Pressing 70 FACT shows 'OUT OF RANGE' error."
    why_human: "Golden fixtures pass in tests/numerical_accuracy.rs and phase65_hpnum_large_exp.rs. A live HP-41 (or HP-41 manual table) comparison of the 10-digit clipped values would provide final hardware-fidelity confidence."
---

# Phase 65: Standalone Fidelity Fixes — Verification Report

**Phase Goal:** Four independent hardware-fidelity gaps are closed: the CLI renders VIEW/AVIEW/PROMPT values on the display, CHS correctly flips the sign of the entry buffer in place, AON causes the ALPHA register to auto-display after every operation, and FACT(X) returns correct scientific-notation results for X in 27..=69.
**Verified:** 2026-06-07T16:25:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | After VIEW/AVIEW/PROMPT executes, CLI main display shows register name/value or ALPHA string | VERIFIED | `hp41-cli/src/ui.rs:154-158` — `else if let Some(ref s) = st.display_override { s.clone() }` inserted between prgm_mode and alpha_mode branches (D-06 priority); two unit tests pass in ui.rs tests module |
| 2 | Pressing CHS during mantissa entry toggles sign in place; no stack lift, no flush | VERIFIED | `hp41-cli/src/app.rs:853-858` — gate `c == 'n' && !entry_buf.is_empty() && !entry_buf.contains('e')`, remove/insert leading `-`; 14 integration tests in `disp02_chs_mantissa_toggle_tests` module |
| 3 | AON (flag 48 set) shows ALPHA register at rest on both CLI and GUI; AOFF reverts to X | VERIFIED | CLI: `hp41-cli/src/ui.rs:162-166`; GUI: `hp41-gui/src-tauri/src/types.rs:197-201` — both use `state.flags & (1u64 << 48) != 0`; 3 tests each; correct precedence (after display_override, after alpha_mode) |
| 4 | FACT(27..=69) returns 10-sig-digit scientific-notation factorials; FACT(70) returns OutOfRange | VERIFIED | `hp41-core/src/ops/math.rs:487` — `HpNum::from_f64(acc).ok_or(HpError::Overflow)?` (replaces Decimal wall); golden fixtures in `numerical_accuracy.rs:2870-2923`; 41 tests pass including FACT(27,30,40,50,60,68,69) and FACT(70)=OutOfRange |

**Score:** 4/4 truths verified

### CR-01 Fix Verification (critical — exponent overflow silent wrap)

The code review identified a confirmed silent-wrong-result bug: `checked_mul`/`checked_div` computed result exponent as `i32` then narrowed with `as i8` before the range check, causing wrap (198 as i8 == -58). This has been **FIXED and tested**.

| Evidence Item | Finding |
|---|---|
| `from_sci` signature | `pub fn from_sci(mantissa: Decimal, exponent: i32) -> Result<HpNum, HpError>` — takes `i32`, range-checks BEFORE narrowing to `i8` (num.rs line 261-279) |
| `checked_mul` large-exponent path | `let new_exp = e_l + e_r; HpNum::from_sci(product, new_exp)` — `new_exp` is `i32`, no `as i8` before `from_sci` (num.rs lines 512-516) |
| `checked_div` large-exponent path | `let new_exp = e_l - e_r; HpNum::from_sci(quotient, new_exp)` — identical pattern (num.rs lines 536-539) |
| Regression test: 1e80 × 1e80 | `mul_1e80_times_1e80_overflows` in phase65_hpnum_large_exp.rs — asserts `Err(HpError::Overflow)` |
| Regression test: 9e99 × 9e99 | `mul_9e99_times_9e99_overflows` — asserts `Err(HpError::Overflow)` |
| Regression test: 9.999e99 / 0.1 | `div_at_ceiling_overflows` — asserts `Err(HpError::Overflow)` |
| Regression test: 1e80 / 1e-40 | `div_large_by_small_exponent_overflows` — asserts `Err(HpError::Overflow)` |
| Test results | `cargo test --test phase65_hpnum_large_exp`: **11 passed** |

CR-01 is RESOLVED with regression coverage.

### Deferred Items

None — all four roadmap success criteria are implemented. Items from the code review (WR-01 through WR-05) are architectural quality improvements, not blockers for the phase goal.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/num.rs` | HpNum `{mantissa: Decimal, exponent: i8}` two-tier struct + from_sci(i32) + checked_mul/div overflow-safe | VERIFIED | Struct exists at line ~148; from_sci at line ~275 takes i32; CR-01 fix confirmed |
| `hp41-core/src/format.rs` | format_hpnum large-exponent path via format_sci_large | VERIFIED | Modified in ec6324c; numerical_accuracy tests exercise it |
| `hp41-core/src/ops/math.rs` | op_fact uses HpNum::from_f64 (not Decimal::from_f64 wall) | VERIFIED | Line 487: `HpNum::from_f64(acc).ok_or(HpError::Overflow)?` |
| `hp41-cli/src/ui.rs` | display_override branch + AON flag-48 branch in get_display_string | VERIFIED | Lines 154-166; 5 unit tests present |
| `hp41-cli/src/app.rs` | CHS mantissa in-buffer toggle (no flush, no dispatch) | VERIFIED | Lines 853-858; 14 integration tests |
| `hp41-gui/src-tauri/src/types.rs` | AON flag-48 branch in from_state display_str chain | VERIFIED | Lines 197-201; 3 unit tests |
| `hp41-core/src/state.rs` | Stale "DISP-01 deferred to v4.4" comments removed | VERIFIED | grep confirms "deferred to v4.4" returns 0 matches in state.rs; line 80 and 531 now read "DISP-01 resolved in Phase 65" |
| `docs/adr/v4.3-005-hpnum-range-extension.md` | ADR amending rust_decimal Frozen Invariant | VERIFIED | File exists at docs/adr/v4.3-005-hpnum-range-extension.md |
| `CLAUDE.md` | BCD/f64 Frozen Invariant bullet updated to describe (mantissa, exponent) + cite ADR v4.3-005 | VERIFIED | Line 43 now describes the two-tier HpNum struct |
| `hp41-core/tests/numerical_accuracy.rs` | FACT(27,30,40,50,60,68,69) golden fixtures + FACT(70)=OutOfRange | VERIFIED | Lines 2870-2923; wide-tolerance goldens for 7 values |
| `hp41-core/tests/phase65_hpnum_large_exp.rs` | Large-exponent overflow regression tests for CR-01 | VERIFIED | File exists; 11 tests covering mul/div overflow + boundary + serde |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `op_view`/`op_aview`/`op_prompt` | CLI `get_display_string` | `state.display_override: Option<String>` read at D-06 slot | WIRED | display_ops.rs writes; ui.rs reads at line 154 |
| `op_aon`/`op_aoff` (flag 48) | CLI `get_display_string` | `state.flags & (1u64 << 48)` | WIRED | ui.rs lines 162-166 |
| `op_aon`/`op_aoff` (flag 48) | GUI `from_state` display_str | `state.flags & (1u64 << 48)` | WIRED | types.rs lines 197-201 |
| CLI app.rs CHS key (`'n'`) | entry_buf in-place toggle | Gate: `!empty && !contains('e')` | WIRED | Lines 853-858; early-return before EEX-CHS block |
| `HpNum::from_f64` | `op_fact` post-compute wall | `HpNum::from_f64(acc).ok_or(Overflow)` | WIRED | math.rs line 487 |
| `from_sci(i32)` | `checked_mul`/`checked_div` exponent overflow detection | Pass `new_exp: i32` before any narrowing | WIRED | CR-01 fix confirmed; from_sci range-checks full i32 before `as i8` store |
| Legacy saves (bare Decimal strings) | `HpNum::Deserialize` | `HpNumWire::Legacy(s)` → `from_decimal` | WIRED | Serde: untagged enum, Extended arm first (Pitfall 2), Legacy fallback |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `get_display_string` (CLI) | `st.display_override` | `CalcState.display_override: Option<String>` written by `op_view`/`op_aview`/`op_prompt` in display_ops.rs | Yes — core writes real register/alpha strings | FLOWING |
| `get_display_string` (CLI) | `st.flags & (1u64 << 48)` + `st.alpha_reg` | `op_aon` sets bit 48 via `flag_set`; alpha_reg is live ALPHA register string | Yes — live flag and register | FLOWING |
| `from_state` (GUI types.rs) | `state.flags & (1u64 << 48)` + `state.alpha_reg` | Same CalcState fields | Yes — live state | FLOWING |
| `app.rs` entry_buf CHS toggle | `self.state.entry_buf` | User keystroke input | Yes — live string buffer | FLOWING |
| `op_fact` | `HpNum::from_f64(acc)` | f64 iterative product `acc` for n in 1..=69 | Yes — f64 product, converted via new extended HpNum path | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| FACT(27..=69) not Overflow | `cargo test -p hp41-core --test numerical_accuracy` | 41 passed | PASS |
| FACT(70) = OutOfRange | included in above suite | assert `matches!(Err(OutOfRange))` passes | PASS |
| CR-01 overflow cases | `cargo test -p hp41-core --test phase65_hpnum_large_exp` | 11 passed | PASS |
| display_override CLI render | `cargo test -p hp41-cli` (test_display_override_renders) | 521 passed | PASS |
| CHS mantissa toggle | `cargo test -p hp41-cli` (disp02_chs_mantissa_toggle_tests) | 14 tests in suite | PASS |
| AON CLI flag-48 | `cargo test -p hp41-cli` (test_aon_flag48_shows_alpha_reg) | passes | PASS |
| AON GUI flag-48 | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --lib types` | 18 passed | PASS |
| Full workspace | `cargo test --workspace` | **3588 passed, 6 ignored** | PASS |

### Probe Execution

No probes declared in PLAN files; phase has no `scripts/*/tests/probe-*.sh` files. Step 7c: SKIPPED (no probe scripts).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| DISP-01 | 65-02-PLAN.md | CLI shows VIEW/AVIEW/PROMPT values on display | SATISFIED | ui.rs:154-158; state.rs stale comments removed; REQUIREMENTS.md marked Complete |
| DISP-02 | 65-03-PLAN.md | CHS during mantissa entry toggles sign in place | SATISFIED | app.rs:853-858; 14 tests; REQUIREMENTS.md marked Complete |
| DISP-03 | 65-04-PLAN.md | AON auto-displays ALPHA register; AOFF stops it | SATISFIED | ui.rs:162-166 + types.rs:197-201; REQUIREMENTS.md marked Complete |
| MATH-01 | 65-01-PLAN.md | FACT(27..=69) returns correct factorial, not Overflow | SATISFIED (code complete) | math.rs:487; numerical_accuracy.rs FACT goldens pass; **REQUIREMENTS.md still shows "Pending" — tracking doc not updated** |

**Note on MATH-01 "Pending" in REQUIREMENTS.md:** The implementation is code-complete and verified by 41 passing tests. The traceability table at line 82 was not updated from "Pending" to "Complete" as part of Phase 65 execution. This is a documentation housekeeping gap only — no code is missing.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-cli/src/ui.rs` | 144, 274 | `placeholder` word in comments | Info | Benign — describes EEX format underscore slots and status bar format string; not code stubs |
| `hp41-core/src/ops/program.rs` | 847 | `"DISP-01 stays deferred to v4.4"` comment | Warning | Stale — D-11 called for removing these comments; Plan 02 cleaned state.rs lines 80 and 531 but missed this instance in program.rs; behavioral code is correct (PSE deliberately does not write display_override); the comment's factual claim is now wrong but does not affect behavior |

No TBD, FIXME, or XXX markers found in any Phase 65 modified files.

### Human Verification Required

#### 1. CLI VIEW/AVIEW/PROMPT live display

**Test:** In the running CLI TUI (`cargo run --bin hp41-cli`): type `1` `ENTER`, then press `f`+`VIEW` and enter register `00`. Observe the main display area.
**Expected:** Display shows the VIEW-formatted string for register 00 (e.g., `R00 1.000000000`), not the X register. Press any number key — display reverts to entry mode.
**Why human:** `get_display_string` reads `display_override` correctly and 2 unit tests pass, but TUI rendering after a real keypress can only be confirmed visually.

#### 2. CHS mantissa in-buffer sign toggle

**Test:** In the running CLI: type `3`, `.`, `1`, `4`, observe display shows `3.14`. Press CHS (`n` key). Observe display. Press CHS again.
**Expected:** First CHS: display shows `-3.14`, no stack change, entry buffer active. Second CHS: reverts to `3.14`. Then press ENTER — stack lifts with value 3.14. Also confirm: on empty buffer, CHS negates X register.
**Why human:** The toggle logic (app.rs:853-858) has 14 integration tests, but keystroke-level TUI timing and display refresh require a human to confirm there are no artefacts.

#### 3. AON/AOFF automatic ALPHA display — both CLI and GUI

**Test:** CLI: enter `SF 48` (set flag 48, AON). Store something in ALPHA register (e.g., `ALPHA`, type `HP41`, `ALPHA`). Then press `ENTER` or any numeric op. Observe main display. Then `CF 48` (AOFF). Observe revert.
GUI: same sequence via the GUI interface.
**Expected:** With AON active, the ALPHA register string appears on the main display instead of X, on both frontends independently. AOFF reverts to X. Both frontends show identical behavior per D-25.6.
**Why human:** The flag-48 branch in both frontends is code-verified with unit tests, but "after every operation" behavior in the live event loop requires visual confirmation of both CLI and GUI.

#### 4. FACT(27..=69) scientific-notation output in running calculator

**Test:** In either CLI or GUI: push 27, press FACT. Note the displayed value. Push 69, press FACT. Push 70, press FACT.
**Expected:** FACT(27) ≈ `1.088886945 28` (HP-41 scientific notation); FACT(69) ≈ `1.711224524 98`; FACT(70) → `OUT OF RANGE` error.
**Why human:** 41 golden tests pass including wide-tolerance fixture comparison. A live comparison against the HP-41 Owner's Manual factorial table (or a real HP-41) provides final hardware-fidelity confidence.

### Gaps Summary

No blocking gaps. All four roadmap success criteria are implemented, wired, and covered by passing tests (3,588 total; 11 phase65-specific large-exponent tests; 41 numerical accuracy tests). The critical CR-01 exponent-wrap bug identified in the code review is fixed and has dedicated regression tests. Human verification is requested for the four interactive behaviors — these cannot be verified by static analysis.

**Non-blocking items noted:**
- `hp41-core/src/ops/program.rs:847` has a stale `"DISP-01 stays deferred to v4.4"` comment (missed by D-11 cleanup; behavioral code correct)
- `REQUIREMENTS.md` MATH-01 row still shows "Pending" instead of "Complete" (documentation housekeeping)

---

_Verified: 2026-06-07T16:25:00Z_
_Verifier: Claude (gsd-verifier)_
