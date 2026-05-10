---
phase: 11-print-emulation
verified: 2026-05-08T00:00:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 7/9
  gaps_closed:
    - "print_buffer serialized to JSON state file — serde(skip) absent (CR-03)"
    - "PRX/PRA/PRSTK output from programs run via F5/R/S, F1-F4 USER handler, try_user_dispatch silently discarded (CR-01)"
  gaps_remaining: []
  regressions: []
---

# Phase 11: Print Emulation Verification Report

**Phase Goal:** PRX, PRA, and PRSTK operations produce formatted print output — visible in the console and optionally appended to a file — while hp41-core remains free of any I/O dependency by buffering output through a new `print_buffer: Vec<String>` field on CalcState.
**Verified:** 2026-05-08
**Status:** passed
**Re-verification:** Yes — after gap closure plan 11-03

---

## Goal Achievement

### Observable Truths (Plan 11-03 Must-Haves)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `print_buffer` is never serialized to the JSON state file — the `serde(skip)` attribute prevents it | VERIFIED | `hp41-core/src/state.rs:94` reads `#[serde(default, skip)]`. No bare `#[serde(default)]` on print_buffer. Old v1.0 files still deserialize via `default`. |
| 2 | PRX/PRA/PRSTK output from programs run via F5/R/S is shown in the TUI status bar after `run_program` returns | VERIFIED | `app.rs:406-413`: `Ok(())` branch sets `self.message = None` then calls `self.drain_and_show_print_output()`. Buffer is drained and status bar updated. |
| 3 | PRX/PRA/PRSTK output from programs run via F1-F4 USER handler is shown in the TUI status bar after `run_program` returns | VERIFIED | `app.rs:239-245`: `Ok(())` branch calls `self.drain_and_show_print_output()`. Previously was bare `self.message = None`. |
| 4 | PRX/PRA/PRSTK output from programs run via `try_user_dispatch` is shown in the TUI status bar after `run_program` returns | VERIFIED | `app.rs:815-821`: `Ok(())` branch calls `self.drain_and_show_print_output()`. Previously was bare `self.message = None`. |
| 5 | All three programmatic drain paths also write to `print_log_writer` when `--print-log` is active | VERIFIED | `drain_and_show_print_output()` at `app.rs:838-855` includes `if let Some(ref mut writer) = self.print_log_writer` loop identical to `call_dispatch_and_drain`. All three call sites use this helper. |

**Score:** 5/5 must-haves verified

---

### Phase-Level Success Criteria (ROADMAP.md)

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | Executing PRX writes X in current display format, right-aligned to 24 chars, to console | VERIFIED | `ops/print.rs:14`: `format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode))`. Interactive path via `call_dispatch_and_drain`; programmatic path via `drain_and_show_print_output`. |
| 2 | Executing PRA writes ALPHA register contents, left-aligned to 24 chars, to console | VERIFIED | `ops/print.rs:26`: `format!("{:<24}", alpha)` with `take(24)` guard. Both paths drain and display. |
| 3 | Executing PRSTK writes full stack in hardware order (T, Z, Y, X, LASTX, ALPHA), one line per register, to console | VERIFIED | `ops/print.rs:39-50`: 6 lines formatted with `{:<7}{:>17}` (numeric) and `{:<7}{:<17}` (ALPHA). |
| 4 | Starting `hp41-cli` with `--print-log <path>` causes all PRX/PRA/PRSTK output to be appended to the specified file | VERIFIED | `main.rs:37-39` defines `--print-log FILE` arg; `App::new()` opens with `create(true).append(true)`; both `call_dispatch_and_drain` and `drain_and_show_print_output` write to `print_log_writer`. |
| 5 | Existing v1.0 JSON save files load without error after CalcState gains the `print_buffer` field | VERIFIED | `#[serde(default, skip)]` on `state.rs:94` — `default` provides `Vec::new()` for missing field; `skip` prevents serialization. Backward-compat confirmed. |

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/state.rs` | `print_buffer` with `#[serde(default, skip)]` | VERIFIED | Line 94: `#[serde(default, skip)]` confirmed. CR-03 closed. |
| `hp41-cli/src/app.rs` | `drain_and_show_print_output()` helper + 3 call sites | VERIFIED | 4 occurrences confirmed (`grep -c` = 4): 1 definition at line 838, 3 call sites at lines 242, 409, 818. |
| `hp41-core/src/ops/print.rs` | `op_prx`, `op_pra`, `op_prstk` — no I/O, push to print_buffer | VERIFIED | 57 lines, 3 public functions, zero `println!`/`print!`/`eprintln!` macros. |
| `hp41-core/tests/print_tests.rs` | Test coverage for all three ops | VERIFIED | 18 tests, all GREEN (confirmed by `just test`). |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `hp41-cli/src/app.rs F5 handler (line 406)` | `self.state.print_buffer` | `drain_and_show_print_output()` after `Ok(())` | WIRED | `app.rs:409`: call confirmed. |
| `hp41-cli/src/app.rs F1-F4 USER handler (line 239)` | `self.state.print_buffer` | `drain_and_show_print_output()` after `Ok(())` | WIRED | `app.rs:242`: call confirmed. |
| `hp41-cli/src/app.rs try_user_dispatch (line 815)` | `self.state.print_buffer` | `drain_and_show_print_output()` after `Ok(())` | WIRED | `app.rs:818`: call confirmed. |
| `drain_and_show_print_output` | `print_log_writer (BufWriter<File>)` | `writeln!(writer, "{}", line)` | WIRED | `app.rs:842-845`: same pattern as `call_dispatch_and_drain`. |
| `ops/print.rs` | `state.print_buffer` | `.push(line)` | WIRED | All three ops push formatted lines to `print_buffer`. |
| `hp41-core` crate | I/O (println!/print!) | (absent) | CLEAN | Zero I/O macros in `ops/print.rs` — core invariant preserved. |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Full workspace tests pass | `cargo test --workspace` | 494 passed, 0 failed | PASS |
| `drain_and_show_print_output` defined exactly once | `grep -c 'fn drain_and_show_print_output' app.rs` | 1 | PASS |
| 4 total occurrences (1 def + 3 call sites) | `grep -c 'drain_and_show_print_output' app.rs` | 4 | PASS |
| `serde(default, skip)` on print_buffer | `state.rs:94` read directly | `#[serde(default, skip)]` | PASS |
| No bare `serde(default)` remaining | `grep 'serde(default)\]' state.rs` | 0 matches | PASS |
| No undrained run_program Ok(()) branch | `grep 'run_program.*message = None' app.rs` | 0 matches | PASS |
| Zero I/O macros in hp41-core print module | Read `ops/print.rs` directly | No `println!`/`print!`/`eprintln!` | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| PRNT-01 | 11-00, 11-01, 11-02 | PRX prints X register, right-aligned 24 chars | SATISFIED | `op_prx` + `drain_and_show_print_output` wired to all call sites |
| PRNT-02 | 11-00, 11-01, 11-02 | PRA prints ALPHA register, left-aligned 24 chars | SATISFIED | `op_pra` + `drain_and_show_print_output` wired to all call sites |
| PRNT-03 | 11-00, 11-01, 11-02 | PRSTK prints full stack 6 lines | SATISFIED | `op_prstk` + `drain_and_show_print_output` wired to all call sites |
| PRNT-04 | 11-02 | `--print-log <path>` appends to file | SATISFIED | `drain_and_show_print_output` writes to `print_log_writer` — all programmatic paths covered |

---

### Anti-Patterns Found

No new blockers or warnings. Previously flagged warning-level items from initial verification remain unchanged and are not blockers:

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-cli/src/app.rs` | 844 | `writer.flush()` called per-line (O(N) for PRSTK) | Warning | Minor — 6 syscalls for PRSTK; cosmetic, not a correctness issue |
| `hp41-core/src/ops/print.rs` | 36 | `state.display_mode.clone()` on a Copy type | Info | Unnecessary clone; no functional impact |

---

### Human Verification Required

None — all must-haves verified programmatically. Phase goal is achieved through code inspection and test execution.

---

## Re-Verification Summary

Both BLOCKER gaps from the initial verification are now closed:

**CR-01 (closed):** `drain_and_show_print_output()` helper added to `App` at `app.rs:838-855`. All three `run_program()` call sites (F5/R/S at line 409, F1-F4 USER handler at line 242, `try_user_dispatch` at line 818) now drain `print_buffer` and surface output in the TUI status bar after `Ok(())`. The helper also writes to `print_log_writer` when `--print-log` is active — the fifth must-have is satisfied by the same helper.

**CR-03 (closed):** `hp41-core/src/state.rs:94` now reads `#[serde(default, skip)]`. The `skip` attribute prevents serialization of the transient buffer. The `default` attribute preserves backward-compatibility with v1.0 save files that lack the field. The comment at lines 91-93 accurately documents both attributes.

494 tests pass across the workspace. Phase 11 goal is fully achieved.

---

_Verified: 2026-05-08_
_Verifier: Claude (gsd-verifier)_
