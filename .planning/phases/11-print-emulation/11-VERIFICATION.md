---
phase: 11-print-emulation
verified: 2026-05-08T00:00:00Z
status: gaps_found
score: 7/9 must-haves verified
overrides_applied: 0
gaps:
  - truth: "PRX/PRA/PRSTK work inside running programs — print_buffer output visible to user after run_program"
    status: partial
    reason: "print ops correctly buffer output in print_buffer during program execution (tests pass), but the F5 (R/S) handler, try_user_dispatch(), and F1-F4 USER-mode handler in app.rs never drain print_buffer after run_program returns. Output accumulates in the buffer silently and is never displayed in the TUI status bar or written to the --print-log file. The stated feature is non-functional for programmatic print from the user's perspective. Identified as CR-01 in 11-REVIEW.md."
    artifacts:
      - path: "hp41-cli/src/app.rs"
        issue: "F5 handler (line 402-408): calls run_program but sets message = None on success with no drain. try_user_dispatch (line 809-812): calls run_program but sets message = None on success with no drain. F1-F4 USER handler (lines 238-244): calls run_program but sets message = None on success with no drain."
    missing:
      - "Add drain_and_show_print_output() helper method to App (mirrors call_dispatch_and_drain drain logic)"
      - "Call drain_and_show_print_output() after every run_program() Ok(()) return in handle_key() (F5, F1-F4, try_user_dispatch)"
  - truth: "Existing v1.0 JSON save files load without error — print_buffer field carries serde(default)"
    status: partial
    reason: "serde(default) is present, enabling backward-compat deserialization. However, print_buffer is NOT annotated with #[serde(skip)], so a non-empty buffer is serialized to the JSON state file when the 30-second autosave fires. This violates the stated invariant 'Never persisted across sessions' (documented in state.rs comment line 91). On next startup, stale print output is deserialized and fills the status bar. Identified as CR-03 in 11-REVIEW.md."
    artifacts:
      - path: "hp41-core/src/state.rs"
        issue: "Line 93: only #[serde(default)] present; missing #[serde(skip)] to prevent serialization of transient buffer"
    missing:
      - "Change attribute at line 93 from #[serde(default)] to #[serde(default, skip)]"
---

# Phase 11: Print Emulation Verification Report

**Phase Goal:** PRX, PRA, and PRSTK operations produce formatted print output — visible in the console and optionally appended to a file — while hp41-core remains free of any I/O dependency by buffering output through a new `print_buffer: Vec<String>` field on CalcState.
**Verified:** 2026-05-08
**Status:** gaps_found
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | PRX writes X in current display format, right-aligned to 24 chars, to print_buffer | VERIFIED | `op_prx` in `hp41-core/src/ops/print.rs:13-18` uses `format!("{:>24}", format_hpnum(...))` and pushes to `state.print_buffer`. 5 tests passing including `test_prx_output_is_24_chars`, `test_prx_output_is_right_aligned`, `test_prx_respects_display_mode_sci`. |
| 2 | PRA writes ALPHA register, left-aligned to 24 chars, to print_buffer | VERIFIED | `op_pra` in `hp41-core/src/ops/print.rs:23-30` uses `format!("{:<24}", alpha)` with `take(24)`. 5 tests passing including `test_pra_output_is_24_chars`, `test_pra_output_is_left_aligned`, `test_pra_truncates_long_alpha_to_24_chars`. |
| 3 | PRSTK writes full stack T/Z/Y/X/LASTX/ALPHA, 6 lines of 24 chars each, in correct order | VERIFIED | `op_prstk` in `hp41-core/src/ops/print.rs:34-56` uses `{:<7}{:>17}` for numeric lines and `{:<7}{:<17}` for ALPHA. 4 tests passing including `test_prstk_produces_six_lines`, `test_prstk_all_lines_are_24_chars`, `test_prstk_line_order_and_labels`. |
| 4 | All three ops have LiftEffect::Neutral — stack unchanged after each | VERIFIED | All three functions call `apply_lift_effect(state, LiftEffect::Neutral)`. `test_prx_lift_effect_neutral` passes. |
| 5 | PRX/PRA/PRSTK work inside running programs (execute_op arms present and buffer fills) | PARTIAL | Arms exist in `program.rs:323-325` and 3 program-execution tests pass (`test_prx_in_program`, `test_pra_in_program`, `test_prstk_in_program`). HOWEVER: the CLI paths that invoke `run_program` (F5 R/S handler, F1-F4 USER mode, `try_user_dispatch`) never drain `print_buffer`. Output from programmatic PRX/PRA/PRSTK is silently discarded from the user's perspective. CR-01 in 11-REVIEW.md. |
| 6 | Starting hp41-cli with --print-log appends output to file; open failure is non-fatal | VERIFIED | `main.rs:38-39` defines `#[arg(long, value_name = "FILE")] print_log`; `App::new()` opens with `create(true).append(true)`; open failure sets `initial_message` and `print_log_writer = None`. `call_dispatch_and_drain` writes to writer. Tests `test_print_log_file_append` and `test_print_log_invalid_path_sets_message` both pass. |
| 7 | hp41-core has zero I/O dependencies — no println!/eprintln!/print! in ops/print.rs | VERIFIED | `grep` of `println!\|eprintln!\|print!` in `hp41-core/src/ops/print.rs` returns zero matches. |
| 8 | Existing v1.0 JSON save files load without error (`serde(default)` on print_buffer) | PARTIAL | `#[serde(default)]` is present at `state.rs:93` enabling backward-compat deserialization. However `#[serde(skip)]` is absent, meaning a non-empty buffer IS serialized on autosave. The comment at line 91 states "Never persisted across sessions" but this invariant is violated. CR-03 in 11-REVIEW.md. |
| 9 | 'P' key opens PrintModal; x/X/a/A/s/S dispatch ops; Esc cancels; PRNT:_ shown in TUI | VERIFIED | `app.rs:200-205` adds 'P' interceptor after `pending_input.is_some()` check; `app.rs:700-722` handles PrintModal arm; `ui.rs:264` returns `"PRNT: _"`. Tests `test_print_modal_prx_sets_message` and `test_print_modal_esc_cancels_without_dispatch` both pass. |

**Score:** 7/9 truths verified (2 partial — both are BLOCKERs)

---

### Deferred Items

None identified.

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/tests/print_tests.rs` | Test scaffold, 18+ tests, RED→GREEN | VERIFIED | Exists, 18 tests, all GREEN. `cargo test -p hp41-core --test print_tests` → 18 passed. |
| `hp41-core/src/ops/print.rs` | `op_prx`, `op_pra`, `op_prstk` functions | VERIFIED | Exists, 57 lines, 3 public functions. Zero I/O macros. |
| `hp41-core/src/state.rs` | `print_buffer: Vec<String>` field with `serde(default)` | VERIFIED (partial) | Field exists at line 94, `serde(default)` at line 93. Missing `serde(skip)` — CR-03. |
| `hp41-core/src/ops/mod.rs` | `Op::PRX`, `Op::PRA`, `Op::PRSTK` variants + dispatch arms | VERIFIED | Lines 212/214/216 (variants), lines 385-387 (dispatch arms). `pub mod print` declared. |
| `hp41-core/src/ops/program.rs` | `execute_op` arms for PRX/PRA/PRSTK | VERIFIED | Lines 323-325, placed before the `Lbl/Gto/Rtn` catch-all at line 327. |
| `hp41-cli/src/app.rs` | `PrintModal` variant, `print_log_writer`, `App::new(…, print_log)`, `call_dispatch_and_drain` | VERIFIED | All present. 4 new tests pass. |
| `hp41-cli/src/main.rs` | `--print-log FILE` CLI arg | VERIFIED | Lines 37-39. `App::new` called with `cli.print_log` at line 70. |
| `hp41-cli/src/ui.rs` | `"PRNT: _"` in `pending_prompt()` | VERIFIED | Line 264. |
| `hp41-cli/src/help_data.rs` | `=== Print ===` category + 3 entries | VERIFIED | Lines 250-253. Category test updated to `test_all_fourteen_categories_present` at line 296. |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ops/print.rs` | `state.print_buffer` | `state.print_buffer.push(line)` | WIRED | All three op functions push to `print_buffer`. |
| `ops/mod.rs` dispatch() | `ops/print.rs` | `Op::PRX => print::op_prx(state)` | WIRED | Lines 385-387 in mod.rs. |
| `ops/program.rs` execute_op() | `ops/print.rs` | `Op::PRX => super::print::op_prx(state)` | WIRED | Lines 323-325 in program.rs, before catch-all. |
| `handle_pending_input()` PrintModal arm | `call_dispatch_and_drain(Op::PRX/PRA/PRSTK)` | `self.call_dispatch_and_drain(Op::PRX)` | WIRED | Lines 703/707/711 in app.rs. |
| `call_dispatch_and_drain` | `print_log_writer (BufWriter<File>)` | `writeln!(writer, "{}", line)` | WIRED | Lines 837-840 in app.rs. |
| F5 / R/S handler | `print_buffer drain` | (missing) | NOT_WIRED | **BLOCKER**: `app.rs:402-408` calls `run_program` but does not drain `print_buffer`. CR-01. |
| `try_user_dispatch()` | `print_buffer drain` | (missing) | NOT_WIRED | **BLOCKER**: `app.rs:809-812` calls `run_program` but does not drain. CR-01. |
| F1-F4 USER handler | `print_buffer drain` | (missing) | NOT_WIRED | **BLOCKER**: `app.rs:238-244` calls `run_program` but does not drain. CR-01. |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `op_prx` | formatted line | `format_hpnum(&state.stack.x, &state.display_mode)` | Yes — live stack X and display mode | FLOWING |
| `op_pra` | formatted line | `state.alpha_reg.chars().take(24)` | Yes — live alpha_reg | FLOWING |
| `op_prstk` | 6 formatted lines | `format_hpnum` for all stack registers + `state.alpha_reg` | Yes — live stack + alpha | FLOWING |
| `call_dispatch_and_drain` | `lines` drained from buffer | `self.state.print_buffer.drain(..)` | Yes — real buffer populated by ops | FLOWING |
| `app.message` (after modal PRX) | string | `lines.into_iter().next()` | Yes — set to 24-char formatted line | FLOWING |
| `app.message` (after F5 run_program with PRX) | string | (not drained) | No — buffer fills but is never drained | HOLLOW — programmatic path disconnected |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 18 print_tests pass | `cargo test -p hp41-core --test print_tests` | 18 passed | PASS |
| CLI tests pass (4 new modal tests) | `cargo test -p hp41-cli` | 86 passed | PASS |
| Full workspace no regressions | `cargo test --workspace` | 494 passed | PASS |
| Zero I/O in hp41-core print module | `grep println!\|eprintln! hp41-core/src/ops/print.rs` | 0 matches | PASS |
| print_buffer.serde(skip) present | `grep "serde(skip)" hp41-core/src/state.rs` | 0 matches | FAIL — CR-03 unresolved |
| run_program drain after F5 | `grep "drain\|print_buffer" app.rs` (near F5 handler) | absent | FAIL — CR-01 unresolved |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| PRNT-01 | 11-00, 11-01, 11-02 | PRX prints X register, right-aligned 24 chars | SATISFIED | `op_prx` exists, 5 tests pass, modal P+X wired |
| PRNT-02 | 11-00, 11-01, 11-02 | PRA prints ALPHA register, left-aligned 24 chars | SATISFIED | `op_pra` exists, 5 tests pass, modal P+A wired |
| PRNT-03 | 11-00, 11-01, 11-02 | PRSTK prints full stack 6 lines | SATISFIED | `op_prstk` exists, 4 tests pass, modal P+S wired |
| PRNT-04 | 11-02 | `--print-log <path>` appends to file | SATISFIED | `--print-log` arg in Cli, `BufWriter` append, open-failure non-fatal |

Note: Requirements are satisfied at the unit/integration test level. PRNT-01/02/03 have a gap in the programmatic (run_program) path — print output during F5/USER program execution is buffered but never shown. This is a correctness gap but the core requirement clauses ("prints to console") are satisfied for the interactive keyboard path.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-core/src/state.rs` | 93 | `#[serde(default)]` without `#[serde(skip)]` on transient buffer | Blocker | Violates "Never persisted across sessions" invariant; stale print output loaded on next startup |
| `hp41-cli/src/app.rs` | 402-408 | `run_program` result not drained from print_buffer | Blocker | PRX/PRA/PRSTK ops inside programs silently discard output — feature non-functional for programmatic print |
| `hp41-cli/src/app.rs` | 809-812 | `try_user_dispatch` does not drain print_buffer | Blocker | Same: USER mode key-assigned programs silently discard print output |
| `hp41-cli/src/app.rs` | 238-244 | F1-F4 USER handler does not drain print_buffer | Blocker | Same: F1-F4 USER programs silently discard print output |
| `hp41-cli/src/app.rs` | 839 | `writer.flush()` called per-line in loop (O(N) flushes for PRSTK) | Warning | 6 syscalls for PRSTK; a single flush after the loop would be cleaner (WR-01) |
| `hp41-cli/src/app.rs` | 160-163 | `'?'` check fires before `pending_input.is_some()` guard | Warning | Opening help overlay while PrintModal is active leaves split-brain state (WR-02) |
| `hp41-core/src/ops/print.rs` | 36 | `state.display_mode.clone()` on a Copy type | Info | Unnecessary clone on Copy type; use `state.display_mode` directly (IN-02) |

---

### Human Verification Required

None — all truths are verifiable programmatically. The phase goal targets buffer behavior and file output, both of which have automated test coverage.

---

## Gaps Summary

Two correctness bugs identified in 11-REVIEW.md are not fixed and constitute blockers for the phase goal.

**Gap 1: CR-01 — programmatic print output never shown (F5/USER-mode/try_user_dispatch)**

The phase goal states print output should be "visible in the console." For the interactive keyboard path (P+X, P+A, P+S), this works: `call_dispatch_and_drain` drains the buffer and sets `app.message`. For the programmatic path (F5/R/S, F1-F4, USER-mode key), `run_program` is called directly, ops push to `print_buffer`, but no code ever drains the buffer afterward. The output is silently lost. Three separate call sites in `app.rs` need a `drain_and_show_print_output()` call added after each `run_program(…).Ok(())` branch.

**Gap 2: CR-03 — print_buffer serialized to JSON state file**

`state.rs` line 91 states "Never persisted across sessions." `#[serde(default)]` only fixes deserialization of old files — it does not prevent serialization. The 30-second autosave will write a non-empty `print_buffer` to disk if a print op fires before the drain runs (possible for long-running programs). On next startup the stale output appears in the status bar. Fix: change `#[serde(default)]` to `#[serde(default, skip)]` at `state.rs:93`.

Both gaps require changes to existing files only — no new files needed. Gap 2 is a one-line fix. Gap 1 requires adding a helper method (~10 lines) and three call sites.

---

_Verified: 2026-05-08_
_Verifier: Claude (gsd-verifier)_
