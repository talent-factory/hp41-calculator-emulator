---
phase: 05-persistence-and-ux
verified: 2026-05-07T16:47:53Z
status: gaps_found
score: 3/5 must-haves verified
overrides_applied: 0
gaps:
  - truth: "User can load at least 10 bundled sample programs from within the TUI and run them to produce documented outputs"
    status: failed
    reason: "3 of 10 bundled programs produce incorrect or non-functional outputs. Prime Test always returns 1 (prime) for every integer n>=2 due to logically inverted early-exit condition. Mean+StdDev program always sums R00 n times regardless of iteration index (acknowledged in code comment: 'simplified: use R00 value always'). Quadratic solver description documents wrong stack entry convention."
    artifacts:
      - path: "hp41-cli/src/programs.rs"
        issue: "prime_test_ops() XySwap before Test(XLeY) causes 2<=n to always be TRUE — program jumps to prime label for every n>=2. Line ~148-150."
      - path: "hp41-cli/src/programs.rs"
        issue: "mean_sdev_ops() uses RclReg(0) unconditionally at line 293 instead of using R13 as iteration index — program sums R00 n times, not distinct registers. Comment acknowledges: 'simplified: use R00 value always'."
      - path: "hp41-cli/src/programs.rs"
        issue: "quadratic_ops() description says 'a(T) b(Z) c(Y)' but StoReg(2) stores X (not Y=c). Users following documented entry order get wrong results."
    missing:
      - "Fix prime_test_ops: remove XySwap at line ~148 so X=n and Y=2, keeping Test(XLeY) for n<=2 check"
      - "Fix or replace mean_sdev_ops with a correct program, or update description to document actual behavior (sums R00 n times)"
      - "Fix quadratic_ops description to match actual stack entry order (c in X, b in Y, a in Z)"

  - truth: "User can press '?' or type HELP in the TUI and see a searchable function reference — overlay is fully navigable without quitting the app"
    status: partial
    reason: "Help overlay opens correctly on '?'. However 'q' closes the app unconditionally even when help overlay is active — the overlay's 'q' close arm at app.rs:224 is dead code because the unconditional quit check at app.rs:127-130 fires first. Per HELP_DATA, 'Esc/q/?' should close the overlay, but pressing 'q' quits the application instead. Esc and '?' (second press) still close it correctly."
    artifacts:
      - path: "hp41-cli/src/app.rs"
        issue: "Unconditional 'q' quit at line 127-130 fires before show_help overlay guard at line 222. The overlay's KeyCode::Char('q') => show_help=false arm at line 224 is unreachable dead code."
    missing:
      - "Guard the 'q' quit with !self.show_help && !self.show_programs && !self.state.alpha_mode && self.pending_input.is_none() OR move 'q' quit check to after all context guards"
human_verification:
  - test: "Open TUI, press '?' to open help overlay, press 'q' — verify app does NOT quit, overlay closes instead"
    expected: "Help overlay closes. App remains running."
    why_human: "CR-01 in REVIEW.md confirms this is broken (q unconditionally quits). Automated test cannot verify interactive TUI behavior."
  - test: "Open TUI, push 3 onto stack, press '?' to open help, scroll with Up/Down arrows, verify overlay content shows HP-41 operations"
    expected: "75 HELP_DATA entries shown in scrollable table; categories visible; keyboard bindings shown"
    why_human: "Visual rendering of help overlay cannot be verified programmatically"
  - test: "Toggle USER mode with 'u', press Ctrl+A, press 'A', type 'MYFIB' then Enter, save with Ctrl+S, restart with --state-file, verify 'A' key in USER mode still runs MYFIB"
    expected: "Key assignment persists across save/reload cycle"
    why_human: "Full interactive USER mode assign + persist + reload flow"
---

# Phase 5: Persistence & UX Verification Report

**Phase Goal:** Users can save and reload complete calculator state between sessions, auto-save fires every 30 seconds, an inline help system is accessible from within the TUI, USER mode works with persisted key assignments, and a bundled sample program library is ready to load.

**Verified:** 2026-05-07T16:47:53Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can save full state (stack, registers, programs, flags, USER assignments) to JSON and reload in fresh session | VERIFIED | persistence.rs: save_state/load_state/StateFile wrapper; test_user_mode_roundtrip verifies key_assignments round-trip; test_calc_state_serde_roundtrip covers all fields |
| 2 | Auto-save fires every 30s; at most 30s of work lost if process killed | VERIFIED | check_autosave() in app.rs:80-88 at Duration::from_secs(30); called each poll iteration in run() loop; save-on-exit at app.rs:108; test_autosave_timer_logic verifies with manipulated last_save |
| 3 | User can press '?' to see searchable function reference with all HP-41 ops and keyboard mappings | PARTIAL | '?' opens help overlay (app.rs:149-153); HELP_DATA has 75 entries across 12 categories (≥50 confirmed by passing test); BUT pressing 'q' inside overlay unconditionally quits app (CR-01 in REVIEW.md — dead code at app.rs:224) |
| 4 | User can assign program label to key in USER mode; toggle USER mode; assignment survives save/reload | VERIFIED | try_user_dispatch() wired (app.rs:499-513); key_assignments BTreeMap in CalcState; F1-F4 pre-wired; test_user_key_assignment_persists verifies BTreeMap round-trip; test_user_mode_roundtrip in persistence.rs |
| 5 | User can load at least 10 bundled sample programs and run them to produce documented outputs | FAILED | 10 programs exist and load via Ctrl+P overlay. Prime Test: always returns 1 for every n>=2 (inverted early-exit logic, CR-04 in REVIEW.md). Mean+StdDev: sums R00 n times instead of n distinct registers (WR-03 in REVIEW.md, acknowledged in code comment). Quadratic Solver: documented stack convention is wrong (CR-05 in REVIEW.md). 3/10 programs do not produce documented outputs. |

**Score:** 3/5 truths fully verified (1 partial with a known bug, 1 failed)

### Deferred Items

None identified.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/num.rs` | HpNum with Serialize/Deserialize via rust_decimal::serde::str | VERIFIED | Line 9: `#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]`; `#[serde(with = "rust_decimal::serde::str")]` on Decimal field |
| `hp41-core/src/state.rs` | CalcState/Stack serde; Vec<HpNum> regs; user_mode + key_assignments | VERIFIED | All 4 types derive Serialize+Deserialize; `pub regs: Vec<HpNum>` at line 56; `pub user_mode: bool` and `pub key_assignments: BTreeMap<char, String>` present |
| `hp41-core/Cargo.toml` | serde + serde-with-str features | VERIFIED | `rust_decimal = { ..., features = ["maths", "serde-with-str"] }`; `serde = { workspace = true }` present |
| `hp41-cli/src/persistence.rs` | save_state, load_state, default_state_path, StateFile | VERIFIED | All 4 exports confirmed; 5 inline tests pass (including test_user_mode_roundtrip) |
| `hp41-cli/src/main.rs` | --state-file clap arg + startup load | VERIFIED | cli.state_file arg at line 28; load_state called at line 39; App::new(initial_state, state_path) at line 54 |
| `hp41-cli/src/app.rs` | App with check_autosave(), 30s timer in run(), Ctrl+S handler, Phase 5 fields | VERIFIED | check_autosave() at line 80; called in run() loop at line 105; save on exit at line 108; Ctrl+S at lines 137-146 |
| `hp41-cli/src/help_data.rs` | HELP_DATA const with ≥50 entries, 10 categories | VERIFIED | 75 entries, 12 categories; test_help_data_has_minimum_entries passes; test_all_ten_categories_present passes |
| `hp41-cli/src/programs.rs` | SampleProgram + sample_programs() + 10 programs via OnceLock | VERIFIED (partial) | 10 programs exist, OnceLock lazy init, all start with Lbl("A"), count test passes — BUT 3 programs have wrong logic (see gaps) |
| `hp41-cli/src/ui.rs` | render_help_overlay, render_programs_overlay, USER annunciator wired | VERIFIED | render_help_overlay() at line 220+; render_programs_overlay() at line 264+; called in render_ui at lines 50/53 conditioned on show_help/show_programs; USER annunciator at line 162 uses `st.user_mode` |
| `hp41-core/src/ops/alpha.rs` | op_alpha_backspace with String::pop() | VERIFIED | `pub fn op_alpha_backspace` at line 44; String::pop() used; 2 inline tests pass |
| `hp41-core/src/ops/mod.rs` | Op::UserMode + Op::AlphaBackspace + serde derives | VERIFIED | Both variants present; dispatch arms at lines 263-268; Serialize/Deserialize on all 3 enums |
| `hp41-core/src/ops/program.rs` | execute_op arms for UserMode and AlphaBackspace | VERIFIED | Arms at lines 293-294 confirmed |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `hp41-cli/src/main.rs` | `persistence.rs load_state` | `load_state(&path)` on startup | WIRED | Line 39: `persistence::load_state(&state_path)` |
| `app.rs run() poll loop` | `app.rs check_autosave()` | `self.check_autosave()` each iteration | WIRED | Line 105 in run() |
| `app.rs check_autosave()` | `persistence.rs save_state` | `elapsed() >= 30s` → `save_state()` | WIRED | Lines 81-83: `Duration::from_secs(30)` threshold |
| `app.rs handle_key '?'` | `app.rs show_help toggle` | `KeyCode::Char('?') → show_help = !show_help` | WIRED | Line 149-153 |
| `ui.rs render_ui()` | `ui.rs render_help_overlay()` | `if app.show_help { render_help_overlay(...) }` | WIRED | Lines 49-51 |
| `ui.rs render_help_overlay()` | `help_data.rs HELP_DATA` | `HELP_DATA.iter()` inside overlay render | WIRED | Confirmed in ui.rs render_help_overlay |
| `app.rs handle_key '?'` | overlay close via `q` | `show_help match KeyCode::Char('q')` | BROKEN | Line 224 `q` close arm is dead code — unconditional quit at line 127 fires first (CR-01) |
| `app.rs try_user_dispatch()` | `run_program()` | `key_assignments.get(&c)` → `run_program(label)` | WIRED | Lines 503-509 |
| `app.rs handle_pending_input AssignKey` | `state.key_assignments` | `key_assignments.insert(c, label)` | WIRED | Line 386 |
| `ui.rs render_annunciators()` | `state.user_mode` | `ann("USER", st.user_mode)` | WIRED | Line 162 |
| `persistence.rs save_state` | `state.key_assignments` | CalcState serde includes BTreeMap<char,String> | WIRED | Verified by test_user_mode_roundtrip |
| `programs.rs sample_programs()` | `app.rs program load` | `programs[idx].ops.clone()` into `state.program` | WIRED | Lines 253-260 in app.rs |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `ui.rs render_help_overlay` | HELP_DATA | `help_data.rs` static const | Yes — 75 compile-time entries | FLOWING |
| `ui.rs render_programs_overlay` | sample_programs() | `programs.rs` OnceLock<Vec<SampleProgram>> | Yes — 10 programs constructed at first call | FLOWING |
| `app.rs check_autosave` | state | CalcState from App | Yes — live CalcState written to disk | FLOWING |
| `app.rs try_user_dispatch` | key_assignments | state.key_assignments BTreeMap | Yes — populated via handle_pending_input AssignKey | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| hp41-core tests pass | `cargo test -p hp41-core` | 288 passed (14 suites) | PASS |
| hp41-cli tests pass | `cargo test -p hp41-cli` | 42 passed | PASS |
| CalcState serde round-trip | `cargo test -p hp41-core -- serde_tests` | 1 passed | PASS |
| Persistence tests pass | `cargo test -p hp41-cli -- persistence::tests` | 6 passed | PASS |
| HELP_DATA minimum entries | `cargo test -p hp41-cli -- help_data::tests` | 3 passed | PASS |
| Programs count ≥10 | `cargo test -p hp41-cli -- programs::tests` | 5 passed | PASS |
| No clippy warnings | `cargo clippy -p hp41-core -p hp41-cli --all-targets -- -D warnings` | No issues | PASS |
| USER mode toggle + dispatch | `cargo test -p hp41-cli -- keys::tests` | 2 passed | PASS |
| Autosave timer logic | `cargo test -p hp41-cli -- tests::test_autosave_timer_logic` | (included in 42) | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| PERS-01 | 05-01, 05-02, 05-03 | Save and load programs and full calculator state to/from versioned JSON files | SATISFIED | persistence.rs StateFile wrapper; save_state/load_state; round-trip test; --state-file arg; key_assignments included |
| PERS-02 | 05-03 | Auto-save every 30s and on graceful shutdown | SATISFIED | check_autosave() at 30s; called in run() loop; save-on-exit in run() before return |
| UX-01 | 05-04, 05-05 | Built-in function reference from within TUI | PARTIALLY SATISFIED | '?' opens help overlay; HELP_DATA has 75 entries with all categories; HELP_DATA accessible from within TUI. BUT 'q' close is broken (CR-01) — pressing q inside help quits the app |
| UX-02 | 05-01, 05-02, 05-06, 05-07 | USER mode with custom key assignments persisted | SATISFIED | user_mode in CalcState; Op::UserMode toggles; try_user_dispatch() runs assigned programs; F1-F4 wired; key_assignments serialized |
| UX-03 | 05-04, 05-05, 05-06 | ≥10 bundled sample programs | PARTIALLY SATISFIED | 10 programs exist and are loadable via Ctrl+P. But Prime Test is broken (CR-04), Mean+StdDev is non-functional (WR-03), Quadratic description is wrong (CR-05). "Run them to produce documented outputs" not fully met. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-cli/src/app.rs` | 127-130 | `q` quit before overlay/alpha guards | BLOCKER | 'q' closes app inside help overlay and in ALPHA mode instead of context-appropriate action |
| `hp41-cli/src/programs.rs` | 148-150 | XySwap + XLeY inverts prime early-exit | BLOCKER | Prime Test always returns 1 for n>=2 — fundamental logic inversion |
| `hp41-cli/src/programs.rs` | 293 | `// simplified: use R00 value always` | BLOCKER | Mean+StdDev acknowledged non-functional — sums same register n times |
| `hp41-cli/src/programs.rs` | 183-188 | Wrong stack description for quadratic | WARNING | Documented "a(T) b(Z) c(Y)" is incorrect — c must be in X for correct computation |
| `hp41-cli/src/app.rs` | 164-178 | S/R/Ctrl+A modal triggers before alpha_mode check | WARNING | 'S' and 'R' open STO/RCL modals in ALPHA mode instead of appending chars to alpha_reg |
| `hp41-cli/src/programs.rs` | 443-447 | `unique.dedup()` without sort | WARNING | test_program_names_unique does not catch non-adjacent duplicates |

### Human Verification Required

### 1. Help Overlay 'q' Close Bug

**Test:** Open TUI, press '?' to open help overlay, then press 'q'.
**Expected:** Help overlay should close. App should remain running (as documented in HELP_DATA "Esc/q/? close").
**Why human:** REVIEW CR-01 confirms this is broken at code level — 'q' unconditionally quits at line 127 before the overlay guard. Human must confirm the broken UX and approve the gap.

### 2. Help Overlay Scrolling and Content

**Test:** Open TUI, press '?', use Up/Down arrows and j/k to scroll through help overlay.
**Expected:** Scrollable table with 75 entries grouped by category headers; keyboard bindings and operation names visible.
**Why human:** Visual rendering of the ratatui overlay cannot be verified programmatically.

### 3. USER Mode Full Assign → Save → Reload Flow

**Test:** Toggle USER mode (press 'u'), assign a key via Ctrl+A (press 'A' for key, type 'MYFIB', Enter), save state (Ctrl+S), exit, restart with --state-file, re-enable USER mode, press 'A' key.
**Expected:** 'A' key runs program labeled 'MYFIB' in the reloaded session.
**Why human:** Full interactive multi-session flow cannot be replicated in unit tests.

### 4. Sample Programs — Fibonacci and Factorial Correct Outputs

**Test:** Load Fibonacci program from program library (Ctrl+P, select Fibonacci, Enter), push n=8, press F5 (R/S). Check result.
**Expected:** Result 21 (F(8) = 21) in X register without error.
**Why human:** Must verify the correct program output appears in the TUI display, not just that no error was returned.

---

## Gaps Summary

**2 BLOCKER gaps prevent full phase goal achievement:**

**Gap 1 — SC-5 FAILED: Sample programs do not produce documented outputs**

Three of the ten bundled programs have confirmed logic bugs found in the post-execution code review (REVIEW.md):

- **Prime Test** (CR-04): The `XySwap` before `Test(XLeY)` means the condition checks `2 <= n` (true for all n≥2), so every integer ≥2 is declared prime. Trial division never executes. Fix: remove the `XySwap` so X=n, Y=2, and `Test(XLeY)` checks `n<=2` correctly.
- **Mean+StdDev** (WR-03): Uses `RclReg(0)` unconditionally instead of using R13 as an iteration index. Acknowledged in code comment. Always computes R00 × n / n = R00. Fix: either implement correctly or replace with an honest description.
- **Quadratic Solver** (CR-05): Description says "a(T) b(Z) c(Y)" but `StoReg(2)` stores X (c must be in X). Users following the documented convention get wrong results.

**Gap 2 — SC-3 PARTIAL: Help overlay 'q' close is broken (BLOCKER for expected UX)**

The 'q' key quits the application unconditionally (app.rs:127-130) before the help overlay guard (app.rs:222). HELP_DATA documents "Esc/q/?" to close the overlay. The 'q' close arm at line 224 is dead code. Esc and '?' (second press) still work. Fix: guard the 'q' quit with `!self.show_help && !self.show_programs && !self.state.alpha_mode && self.pending_input.is_none()`.

**What IS working:**
- PERS-01: Full CalcState JSON save/load with versioned StateFile wrapper — complete
- PERS-02: 30-second auto-save + save-on-exit — complete  
- UX-02: USER mode toggle, key assignment, try_user_dispatch(), F1-F4 wiring, persisted key assignments — complete
- Help overlay opens and displays correctly; Esc and '?' close it correctly
- 7 of 10 sample programs are correct (Fibonacci, Factorial, GCD, Newton Root, Deg-to-Rad, Stack Stats, Countdown Timer)

---

_Verified: 2026-05-07T16:47:53Z_
_Verifier: Claude (gsd-verifier)_
