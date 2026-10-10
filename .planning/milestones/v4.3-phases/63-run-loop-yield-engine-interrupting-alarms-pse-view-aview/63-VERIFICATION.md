---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
verified: 2026-06-06T14:30:00Z
status: human_needed
score: 5/5
overrides_applied: 0
re_verification:
  previous_status: human_needed
  previous_score: 4/5
  gaps_closed:
    - "SC-3 GUI idle-alarm path: alarm:xeq arm now calls invoke('run_program',{label}) instead of dispatch_op — commit abb46ef, App.tsx:1248"
    - "WR-04 CLI drain_pending_yields missing after drain_event_buffer: drain now called at app.rs:308 on every tick after check_alarms + drain_event_buffer — commit b62eff5"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Trigger a running program with PSE and observe the CLI display"
    expected: "Display shows X register value for ~1 second, then next step executes; visible pause"
    why_human: "std::thread::sleep and terminal redraw cannot be asserted programmatically without real UI"
  - test: "Trigger a running program with VIEW RNN and AVIEW and observe both CLI and GUI displays"
    expected: "CLI: display flips to register/ALPHA value for ~1 second; GUI: main display shows pending_yield.text for resume_ms before next step"
    why_human: "Requires visual confirmation that yield text is actually rendered in the TUI/WebView"
  - test: "Set a repeating interrupting alarm (>>LABEL), run a program, verify it fires mid-run and resumes"
    expected: "Program pauses, alarm handler executes (e.g. stores a value), program resumes from exact halted step; alarm reschedules for next fire"
    why_human: "Real-time alarm timing and mid-run visual inspection requires human observation"
---

# Phase 63: Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW — Verification Report

**Phase Goal:** Build the synchronous pending-interrupt mechanism in run_loop; implement interrupting control alarm execution and PSE/VIEW-AVIEW mid-run display yields (CLI + GUI).
**Verified:** 2026-06-06T14:30:00Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure (commits b62eff5, abb46ef)

> **Note on status:** All 5 truths are now VERIFIED (score 5/5). Status remains `human_needed` solely because the 3 pure-visual human_verification items (PSE render on terminal, VIEW/AVIEW render in WebView, real-time alarm timing) are inherently non-automatable — their underlying code paths are fully unit-tested, but the gsd-verifier rule requires human_verification to be empty for `passed`. The two previously-escalated developer decisions (SC-3 GUI parity gap and WR-04 CLI drain gap) have been resolved by code fixes and confirmed below.

## Re-Verification: Gap Closure Confirmation

### WR-04 — CLI drain_pending_yields after drain_event_buffer (commit b62eff5)

**Fix location:** `hp41-cli/src/app.rs`, lines 299–308

The `run()` loop now calls `drain_pending_yields(&mut terminal)?` at line 308, immediately after `drain_event_buffer()` at line 298 on every tick. This is symmetric with the post-handle_key drain at line 289. Comment at lines 299–307 explains the invariant ("Mirrors the after-handle_key drain invariant — 'wire ALL run_program call sites'").

**Regression test:** `drain_event_alarm_xeq_pse_sets_pending_yield` at app.rs:3471. The test:
1. Builds an App with a program containing PSE under label "ALRM"
2. Pushes `"alarm:xeq:ALRM"` into `event_buffer`
3. Calls `app.drain_event_buffer()` (the single tick operation, before the full run() loop drain)
4. Asserts `app.state.pending_yield.is_some()` — confirming `run_program` was called and the PSE arm was reached
5. Asserts no error message (run_program succeeded)

Status: **VERIFIED** — drain is wired at the run()-loop tick after drain_event_buffer; pending_yield from an alarm-launched PSE program is no longer stranded.

### SC-3 / CR-01 — GUI alarm:xeq arm calls run_program (commit abb46ef)

**Fix location:** `hp41-gui/src/App.tsx`, lines 1239–1251

The `alarm:xeq:` arm in the event_buffer useEffect (line 1239) now calls:
```
invoke<CalcStateView>('run_program', { label })
```
The old `dispatch_op({ keyId: 'xeq_'+label })` path is fully removed. A comment at lines 1229–1232 documents the gap that was closed (D-25.6 CLI parity: dispatch_op only resolved builtins/XROM and returned InvalidOp for user LBL programs). The returned `CalcStateView` is set via `setCalcState`; if `pending_yield` is non-null the existing yield-and-resume useEffect picks it up and schedules `resume_program` — no duplicated scheduling, no polling (D-11 preserved).

**Regression guard:** `grep -n "dispatch_op.*xeq_\|xeq_.*dispatch_op" App.tsx` → 0 active calls (1 match is a comment only).

**Test D5 updated** (App.test.tsx:393–408): "event_buffer 'alarm:xeq:TESTLBL' invokes run_program({ label: 'TESTLBL' }) (SC-3)" — asserts `mockInvoke` was called with `'run_program', { label: 'TESTLBL' }` and explicitly guards that `dispatch_op({ keyId: 'xeq_TESTLBL' })` was NOT called.

**Test P4 added** (App.test.tsx:1034–1073): "alarm:xeq:FOO invokes run_program and pending_yield from result schedules resume_program" — full end-to-end composition test: dispatch_op returns alarm event → alarm:xeq useEffect calls run_program → run_program returns pending_yield → yield-driver useEffect schedules resume_program after 500ms via fake timers.

Status: **VERIFIED** — alarm:xeq arm calls run_program; old dispatch_op path gone; D5 and P4 tests confirm.

### ADR Naming Deviation — Accepted/Resolved

The ADR exists as `docs/adr/v4.3-004-interrupt-alarm-pending-field.md` (not v4.3-001 as originally planned). v4.3-001/002/003 were already allocated. All cross-references in the codebase (divergences doc, plan files) consistently use v4.3-004. This deviation is accepted as resolved — not a gap.

---

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| SC-1 | A running program is interrupted at the next instruction boundary; the alarm's stored label program executes in the interrupted program's register environment; the original program resumes from exactly where it was halted | VERIFIED | `run_loop` takes `pending_interrupt` before pc-fetch (program.rs:520-544); pushes pc onto call_stack (synthetic XEQ frame); `Op::Rtn` pop restores pc; 12 unit tests green in `phase_63_interrupting_alarms.rs` |
| SC-2 | An interrupting alarm is suppressed (left past-due) and not executed when the call stack is already at the 4-level cap; call_stack never reaches 5 levels; is_running correctly restored on all paths | VERIFIED | program.rs:521-526: `if call_stack.len() >= 4` → silent suppress, clear index/depth, continue. Cap-drop test `interrupt_blocked_when_call_stack_at_4_level_cap` green. is_running reset in `run_program` via captured `let result = run_loop(...)` pattern (line 455). Matches ALARM-03 spec and CONTEXT.md D-07. |
| SC-3 | An idle-fired interrupting alarm (no program running) executes the alarm label program via the existing run_program path without any resume logic | VERIFIED | CLI: `drain_event_buffer` (app.rs:1850) calls `hp41_core::run_program` for `alarm:xeq:` events — confirmed. GUI: `App.tsx:1248` now calls `invoke<CalcStateView>('run_program', { label })` — commit abb46ef confirmed; old `dispatch_op` path removed. Both D5 (routing) and P4 (yield composition) tests confirm. |
| SC-4 | PSE during a running program pauses the display for approximately 1 second (visible in both CLI and GUI) before the next program step executes | VERIFIED (automated) / HUMAN NEEDED (visual) | Core: `Op::Pse` arm in run_loop sets `pending_yield{kind:Pse, text:format_hpnum(X), resume_ms:1000}` and breaks (program.rs:790-799). CLI: `drain_pending_yields` renders `yield_text` via `entry_buf`, sleeps `resume_ms`, calls `resume_program` (app.rs:322-358). GUI: `useEffect` on `pending_yield` schedules `invoke('resume_program')` via `setTimeout(resume_ms)` (App.tsx:592-606). Tests: `pse_mid_run_breaks_and_records_resume_ms`, `pse_resume_continues_to_next_step` green. Visual render needs human observation. |
| SC-5 | VIEW and AVIEW during a running program display their register/ALPHA value briefly before the program continues — not only after the program ends | VERIFIED (automated) / HUMAN NEEDED (visual) | Core: `Op::View(reg)` and `Op::AView` arms set `pending_yield{kind:View/Aview, text, resume_ms:1000}` and break without writing `display_override` (program.rs:800-831). Same frontend wiring as SC-4. Tests: `view_mid_run_captures_formatted_register_into_yield`, `aview_mid_run_captures_alpha_into_yield` green. |

**Score:** 5/5 truths verified (SC-3 upgraded from UNCERTAIN to VERIFIED — commit abb46ef)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/state.rs` | pending_interrupt/alarm_index/depth + pending_yield + YieldState/YieldKind + serde(skip) | VERIFIED | Lines 490-531: all 4 fields with `#[serde(default, skip)]`; YieldKind/YieldState types at lines 57-86; PSE_RESUME_MS=1000. Serde round-trip tests at lines 820-878. |
| `hp41-core/src/ops/time/alarm.rs` | dispatch_alarm_event: interrupting-arm routing + D-10 demotion | VERIFIED | Lines 535-558: if `interrupting && defer_to_run_loop && is_running && pending.is_none() && !solver/modal` → sets `pending_interrupt`. Else → `alarm:xeq:{label}` to event_buffer. check_alarms passes `defer_to_run_loop=true`; op_almnow passes `false`. |
| `hp41-core/tests/phase_63_interrupting_alarms.rs` | 12 interrupt tests covering all edge cases | VERIFIED | 666-line file; 12 `#[test]` functions covering: halts-and-resumes, stack preservation, 4-level cap, nesting blocked, idle regression, non-interrupting unchanged, message unchanged, solver/modal demotion, missing label, cleared-on-resume, backward-compat, repeating reschedule. |
| `hp41-core/src/ops/program.rs` | run_loop interrupt check + Phase-C + ack-after-RTN + PSE/VIEW/AVIEW yield arms + clear-on-entry | VERIFIED | Interrupt check: lines 520-544. Phase-C: lines 513-515. Ack-after-RTN: lines 567-576. PSE arm: 790-799. VIEW arm: 800-818. AVIEW arm: 822-831. Clear-on-entry: run_program 449-452, resume_program 482-485. |
| `hp41-core/tests/phase_63_yield_engine.rs` | 4 yield tests for PSE/VIEW/AVIEW | VERIFIED | 269-line file; 4 `#[test]` functions all matching plan spec. |
| `hp41-cli/src/app.rs` | yield render+sleep+resume loop; alarm:missing arm; alarm:interrupting removed; drain after drain_event_buffer (WR-04 fix) | VERIFIED | `drain_pending_yields` at lines 322-358 loops while pending_yield.is_some(); `drain_event_buffer` adds alarm:missing arm (line 1861); `grep "alarm:interrupting" app.rs` → empty. WR-04 fix: drain_pending_yields at line 308 after drain_event_buffer at line 298 on every run() tick. Regression test `drain_event_alarm_xeq_pse_sets_pending_yield` at line 3471. |
| `hp41-gui/src-tauri/src/commands.rs` | run_program + resume_program Tauri commands | VERIFIED | Lines 397-427: `run_program` command calls `hp41_core::ops::program::run_program`; `resume_program` calls `hp41_core::ops::program::resume_program`. Both return `CalcStateView`. |
| `hp41-gui/src-tauri/src/types.rs` | pending_yield projected in CalcStateView | VERIFIED | Line 152: `pub pending_yield: Option<YieldView>`; projection at lines 265-268 via `YieldView::from_yield_state`. |
| `hp41-gui/src-tauri/src/lib.rs` | both commands registered in generate_handler! | VERIFIED | Line 261-263: `run_program` and `resume_program` in `generate_handler!`. |
| `hp41-gui/src-tauri/permissions/run-program.toml` | permission TOML | VERIFIED | File exists (179B). |
| `hp41-gui/src-tauri/permissions/resume-program.toml` | permission TOML | VERIFIED | File exists (188B). |
| `hp41-gui/src-tauri/capabilities/default.json` | permissions referenced | VERIFIED | Lines 28-29: `allow-run-program` and `allow-resume-program`. |
| `hp41-gui/src/App.tsx` | R/S 4-way routing; yield-and-resume driver; alarm:missing toast; alarm:interrupting removed; alarm:xeq → run_program (SC-3 fix); pending_yield in CalcStateView interface | VERIFIED | R/S 4-way at lines 127-144. useEffect driver at lines 592-606. alarm:missing at line 1241-1243. `grep "alarm:interrupting" App.tsx` → empty. alarm:xeq arm at lines 1239-1251: invoke('run_program',{label}) confirmed; old dispatch_op path absent. |
| `hp41-gui/src/App.test.tsx` | Vitest: yield render+resume, alarm:missing, R/S start, D5 routing, P4 yield-composition | VERIFIED | D5 (alarm:xeq→run_program routing + dispatch_op regression guard) at lines 390-408. P4 (alarm:xeq yield composition end-to-end) at lines 1030-1073. |
| `docs/adr/v4.3-004-interrupt-alarm-pending-field.md` | ADR for interrupt mechanism (named v4.3-004, not v4.3-001 — accepted) | VERIFIED | EXISTS. Content verified: references `pending_interrupt`, `v4.3-003`, describes synthetic XEQ frame, 4-level cap, D-10 demotion, ack-after-RTN. Named v4.3-004 because v4.3-001/002/003 were already allocated. Consistent throughout codebase. |
| `docs/hp41-time-divergences.md` | §D-40-04 flipped to implemented | VERIFIED | Line 265: "IMPLEMENTED in Phase 63 (v4.3). Verified in Phase 66." References `v4.3-004` ADR at line 320. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| run_loop interrupt check | find_in_program + call_stack.push | synthetic XEQ frame; captures pending_interrupt_depth | VERIFIED | program.rs:532-534: `pending_interrupt_depth = Some(call_stack.len()); call_stack.push(state.pc); pc = target+1` |
| Op::Rtn pop | acknowledge_alarm | ack after handler returns to pending_interrupt_depth | VERIFIED | program.rs:567-576: guard on `pending_interrupt_alarm_index.is_some() && Some(call_stack.len()) == pending_interrupt_depth`; calls `time::alarm::acknowledge_alarm(state, idx)` |
| Op::Pse / Op::View / Op::AView run_loop arms | state.pending_yield | set YieldState + break (no display_override) | VERIFIED | program.rs:790-831: all three arms set `pending_yield = Some(YieldState{...})` and `break`. `display_override` not touched. |
| R/S keystroke handler (CLI) | resume_program | while pending_yield.is_some(): render text, sleep, resume | VERIFIED | app.rs:289 calls `drain_pending_yields` after `handle_key`; drain_pending_yields loop at 322-358 |
| run() tick (CLI) | drain_pending_yields | after check_alarms + drain_event_buffer on every frame | VERIFIED (WR-04 fix) | app.rs:308: `self.drain_pending_yields(&mut terminal)?` after `drain_event_buffer()` at line 298; alarm-launched program yields now drained on same frame |
| drain_event_buffer alarm:missing arm | self.message | alarm:missing → status line | VERIFIED | app.rs:1861-1864: `strip_prefix("alarm:missing:")` → `self.message = Some(format!(...))` |
| R/S keystroke (GUI) | run_program command | 4-way: modal > cancel > stop > start | VERIFIED | App.tsx:127-144: `invoke<CalcStateView>('run_program', { label: 'A' })` in branch 3 |
| yield driver (GUI) | resume_program command | setTimeout(pending_yield.resume_ms) → invoke resume_program | VERIFIED | App.tsx:592-606: `useEffect` schedules `invoke('resume_program')` after `resume_ms` |
| event consumer (GUI) | showToast | alarm:missing arm | VERIFIED | App.tsx:1241-1243 |
| alarm:xeq event (GUI) | run_program command | invoke('run_program',{label}) — SC-3 fix | VERIFIED (SC-3 fix) | App.tsx:1248: `invoke<CalcStateView>('run_program', { label })` — user LBL programs now execute; old dispatch_op path removed |
| hp41-time-divergences §D-40-04 | ADR v4.3-004 | cross-reference | VERIFIED | Line 320 references `v4.3-004-interrupt-alarm-pending-field.md` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `run_loop` PSE arm | `state.pending_yield.text` | `format_hpnum(&state.stack.x, &state.display_mode)` | Yes — real X register | FLOWING |
| `run_loop` VIEW arm | `state.pending_yield.text` | `format_hpnum(&regs[reg].numeric_or_zero(), &state.display_mode)` | Yes — real register | FLOWING |
| `run_loop` AVIEW arm | `state.pending_yield.text` | `alpha_reg.chars().take(24).collect()` | Yes — real ALPHA | FLOWING |
| `CalcStateView.pending_yield` | `YieldView` | `state.pending_yield.as_ref().map(YieldView::from_yield_state)` | Yes — real core state | FLOWING |
| `drain_pending_yields` (CLI) | yield text render | `state.pending_yield.as_ref().expect(...)text.clone()` into `entry_buf` | Yes — real yield text | FLOWING |
| `useEffect` driver (GUI) | `calcState.pending_yield` | Tauri IPC return from `run_program`/`resume_program` | Yes — real CalcStateView | FLOWING |
| alarm:xeq → run_program (GUI) | `CalcStateView` | `invoke('run_program',{label})` → Rust run_program → real core execution | Yes — user LBL program executes | FLOWING (SC-3 fix) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| State fields are serde(skip) | `grep -n "serde(default, skip)" state.rs` around pending_* fields | Lines 498,506,522,530 all have `#[serde(default, skip)]` | PASS |
| No alarm:interrupting silent-ignore in CLI | `grep -n "alarm:interrupting" hp41-cli/src/app.rs` | 0 matches | PASS |
| No alarm:interrupting silent-ignore in GUI | `grep -n "alarm:interrupting" hp41-gui/src/App.tsx` | 0 matches | PASS |
| PSE/VIEW/AVIEW yield arms in run_loop | `grep -n "pending_yield = Some" program.rs` | 3 matches (lines 793, 812, 825) | PASS |
| ack-after-RTN gate | `grep -n "acknowledge_alarm" program.rs` | Line 573 inside RTN arm, gated on depth check | PASS |
| D-40-04 flipped in divergences doc | `grep -n "IMPLEMENTED" docs/hp41-time-divergences.md` | "IMPLEMENTED in Phase 63 (v4.3)" | PASS |
| No unwrap() in production program.rs code | `grep -n "unwrap()" program.rs \| grep -v test` | All occurrences are in `#[cfg(test)]` section | PASS |
| math1/ freeze | `git diff HEAD~15..HEAD -- hp41-core/src/ops/math1/` | 1 line (whitespace-only context) | PASS |
| WR-04 fix: drain after drain_event_buffer in run() | `grep -n "drain_pending_yields" app.rs` at tick position | Line 308: drain_pending_yields after drain_event_buffer at line 298 | PASS |
| SC-3 fix: alarm:xeq uses run_program not dispatch_op | `grep -n "dispatch_op.*xeq_" App.tsx` | 0 active calls (1 comment-only match) | PASS |

### Probe Execution

Step 7c: SKIPPED — no probe-*.sh files in scripts/tests/; phase is not a migration/tooling phase. CI pass confirmed per verification_context: `just ci` green (all tests, coverage 95.09% lines, license-audit), `just gui-ci` green (129 Rust + 340 TS tests).

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| ALARM-02 | 63-01, 63-02, 63-03, 63-04, 63-06 | Control alarm interrupts running program at next boundary; handler runs in interrupted register env; resumes at halted pc | SATISFIED | `run_loop` interrupt injection verified; 12 core tests green; CLI R/S and GUI R/S paths verified |
| ALARM-03 | 63-01, 63-02, 63-03, 63-04, 63-06 | 4-level cap honored; idle-fired via existing path; repeating reschedules after RTN | SATISFIED | Cap-drop silent-suppress verified; ack-after-RTN verified; CLI idle→run_program verified; GUI idle→run_program now verified (SC-3 fix, commit abb46ef) |
| PRGM-01 | 63-02, 63-03, 63-04, 63-06 | PSE pauses display ~1 second in CLI and GUI | SATISFIED | PSE yield arm verified; CLI drain_pending_yields at R/S call site verified; GUI useEffect driver verified; alarm:xeq→run_program CLI path now has drain_pending_yields on same tick (WR-04 fix, commit b62eff5) |
| PRGM-02 | 63-02, 63-03, 63-04, 63-06 | VIEW/AVIEW display value mid-run in CLI and GUI | SATISFIED | VIEW/AVIEW yield arms verified; same frontend wiring as PSE; WR-04 fix applies equally |

### Code Review Findings — Phase Scope Assessment

| Finding | Severity | Phase Scope | Disposition |
|---------|----------|------------|-------------|
| CR-01: GUI alarm:xeq uses dispatch_op instead of run_program | BLOCKER (prior) | FIXED | Commit abb46ef routes alarm:xeq through invoke('run_program',{label}). D5 + P4 tests confirm. Closed. |
| CR-02: positional alarm index can be stale if handler mutates alarm table | WARNING | NARROW OUT-OF-SCOPE | ALARM-02/03 cover the common case. Edge case (handler runs XYZALM/CLALMA) not explicitly specified. Recommend follow-up: stable-key ack or re-find-at-RTN. |
| WR-01: cap-drop repeating alarm permanently lost | WARNING | OUT-OF-SCOPE | ALARM-03 explicitly specifies "suppressed and left past-due at depth 4". CONTEXT.md D-07/D-08 confirm cap-drop stays silent by design. Not a phase defect. |
| WR-02: second simultaneous past-due alarm not rescheduled | WARNING | FOLLOW-UP | Not an explicitly required test scenario. Single interrupting alarm case works correctly. Follow-up for future phase. |
| WR-03: GUI yield-window allows user input to corrupt paused program | WARNING | FOLLOW-UP | No requirement prohibits key input during PSE. Real HP-41 behavior: running program owns the keyboard during pause. Follow-up for D-25.6 parity in a future phase. |
| WR-04: CLI alarm:xeq run_program call site lacks drain_pending_yields | WARNING (prior) | FIXED | Commit b62eff5: drain_pending_yields at app.rs:308 after drain_event_buffer at line 298. Regression test `drain_event_alarm_xeq_pse_sets_pending_yield` at line 3471. Closed. |
| WR-05: interrupt handler shares RPN stack | WARNING | OUT-OF-SCOPE | CONTEXT.md explicitly notes this: "alarm handlers must STO/RCL defensively — Risk 2." Documented accepted design choice. |
| IN-01: MAX_STEPS guard resets per resume segment | INFO | OUT-OF-SCOPE | Informational; documented behavior. |
| IN-02: D5 GUI test asserts IPC shape not execution (prior) | INFO | RESOLVED | D5 was updated to assert run_program call and guard dispatch_op absence. P4 added for full yield-composition path. |
| ADR naming (v4.3-004 vs planned v4.3-001) | PARTIAL (prior) | ACCEPTED | v4.3-001/002/003 were already allocated. v4.3-004 is consistent throughout the codebase. No rename needed. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| program.rs | 502 | `return Err(HpError::Overflow)` comment says "CR-01" | Info | Comment references code review finding — harmless |

No TBD, FIXME, or XXX markers found in any phase-modified file. No unwrap() in production code paths (only in test sections per CLAUDE.md allowance). No println!/eprintln! in hp41-core.

### Human Verification Required

All three items below are pure visual/timing checks whose code paths are fully unit-tested. They remain as human-needed solely because automated assertions cannot substitute for visual observation of terminal output or WebView rendering.

#### 1. Visual PSE Pause — CLI

**Test:** Run a program with PSE (`LBL A; PushNum(42); PSE; Rtn`) via R/S (F5 or key `a` in RUN mode). Observe the CLI display.
**Expected:** The display shows the X register value for approximately 1 second, then the program continues. The pause is visually apparent (not instant).
**Why human:** `std::thread::sleep(1000ms)` executes correctly per code review but the terminal redraw via `entry_buf` trick requires visual confirmation that the display actually flips to the yield text.

#### 2. Visual VIEW/AVIEW Mid-Run — GUI

**Test:** Run a program with `VIEW 00` or `AVIEW` via R/S in the GUI. Observe the calculator display.
**Expected:** The GUI main display shows `pending_yield.text` (formatted register value or ALPHA content) for ~1 second before continuing.
**Why human:** The `useEffect` setTimeout driver is verified in unit tests with fake timers, but actual visual rendering in the WebView/Tauri window requires observation.

#### 3. Interrupting Alarm Mid-Run — Real Timing

**Test:** Set an alarm with `>>MYPROG` label (interrupting), start a long-running loop program, wait for the alarm time. Observe that the alarm program executes mid-run and the original resumes.
**Expected:** Original program halts, MYPROG executes, original program resumes at the exact halted step. For a repeating alarm, it fires again on the next trigger time.
**Why human:** Real-time alarm timing and the mid-run interrupt boundary require human observation to confirm the behavior matches the spec at real execution speed.

### Gaps Summary

No open gaps. Both previously-escalated developer decisions have been resolved by code fixes:

- **WR-04 (CLI alarm-launched PSE drain):** Fixed in commit b62eff5. `drain_pending_yields` now called at app.rs:308 on every run() tick after `drain_event_buffer`. Regression test confirms PSE yield is set (not stranded) after alarm:xeq processing.
- **SC-3 (GUI idle-alarm path):** Fixed in commit abb46ef. `App.tsx:1248` now calls `invoke('run_program',{label})` for alarm:xeq events. User LBL programs execute correctly on idle alarm fire in the GUI. D5 and P4 tests confirm routing and yield-composition.

The 3 human_verification items are inherently visual/timing checks and are not gaps — their underlying logic is code-verified and unit-tested.

---

_Verified: 2026-06-06T14:30:00Z_
_Verifier: Claude (gsd-verifier) — re-verification after commits b62eff5, abb46ef_
