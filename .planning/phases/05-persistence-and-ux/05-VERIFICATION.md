---
phase: 05-persistence-and-ux
verified: 2026-05-07T17:40:47Z
status: gaps_found
score: 4/5 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 3/5
  gaps_closed:
    - "SC-3: 'q' in help overlay no longer quits app — guarded with !show_help && !show_programs && !alpha_mode && pending_input.is_none()"
    - "SC-5 prime_test_ops: spurious XySwap removed; Op::Int added for exact modulo; loop exit XGtY→XLtY fixed"
    - "SC-5 mean_sdev_ops: replaced with correct 7-op 4-value stack mean (Stack Mean (4 values))"
    - "SC-5 quadratic_ops: comment corrected to 'c in X, b in Y, a in Z'"
  gaps_remaining:
    - "SC-5 partial: gcd_ops missing Op::Int causes wrong GCD for most non-trivial integer inputs (CR-02); stack_stats_ops has inverted Test+XySwap logic (CR-03) — 8/10 programs produce documented outputs, 2 do not"
  regressions: []
gaps:
  - truth: "User can load at least 10 bundled sample programs from within the TUI and run them to produce documented outputs"
    status: partial
    reason: "8 of 10 programs produce correct documented outputs. Two programs have behavioral bugs discovered by post-gap-closure code review (CR-02, CR-03): gcd_ops computes wrong GCD for most non-trivial inputs because floor-division truncation is missing (Op::Int absent — e.g. GCD(12,8) returns 8 instead of 4); stack_stats_ops stores the WRONG candidate in every comparison step due to inverted Test+XySwap logic (XGtY fires XySwap, placing smaller value into X for 'max' tracking). Neither bug has a behavioral test. Programs verified correct: Fibonacci, Factorial, Prime Test (n=2..13), Quadratic Solver, Newton Root, Stack Mean (4 values), Deg-to-Rad, Countdown."
    artifacts:
      - path: "hp41-cli/src/programs.rs"
        issue: "gcd_ops() at line 246-249: RclReg(0), RclReg(1), Div, RclReg(1), Mul — missing Op::Int between Div and second RclReg(1). For GCD(7,3): 7/3=2.333..., 3*2.333...=6.999..., 7-6.999...=0.000... not zero. For GCD(12,8): 12/8=1.5, 8*1.5=12 exact, 12-12=0 wrong (GCD should be 4)."
      - path: "hp41-cli/src/programs.rs"
        issue: "stack_stats_ops() at line 329-343: Test(XGtY) followed by XySwap. HP-41 TRUE=execute next. When X>Y (TRUE), XySwap fires, putting smaller value into X, then StoReg(5) stores smaller as 'max'. Trace: stack T=4,Z=3,Y=2,X=5; after Enter+Rdn stack has X=5,Y=2; XGtY TRUE → XySwap → X=2 stored as R05 (wrong — max is 5). Same inversion in min section."
    missing:
      - "Fix gcd_ops: insert Op::Int after Div (mirror prime_test_ops fix): Op::RclReg(0), Op::RclReg(1), Op::Div, Op::Int, Op::RclReg(1), Op::Mul, Op::RclReg(0), Op::XySwap, Op::Sub"
      - "Fix stack_stats_ops: invert test conditions — use XLtY for max-finding (XySwap when X<Y brings larger Y into X), use XGtY for min-finding (XySwap when X>Y brings smaller Y into X)"
      - "Add behavioral tests for gcd (e.g. gcd(12,8)=4, gcd(7,3)=1) and stack_stats (X=min, Y=max for known 4-value inputs)"
human_verification:
  - test: "Open TUI, press '?' to open help overlay, press 'q' — verify app does NOT quit, overlay closes instead"
    expected: "Help overlay closes. App remains running."
    why_human: "Unit test test_q_does_not_quit_when_help_overlay_open confirms logic; interactive visual confirmation of TUI rendering path"
  - test: "Toggle USER mode ('u'), assign key via Ctrl+A (press 'z', type 'MYPROG', Enter), save Ctrl+S, quit, relaunch with --state-file, verify 'z' in USER mode runs MYPROG"
    expected: "Key assignment persists across save/reload cycle; USER annunciator shows active; assigned key triggers run_program"
    why_human: "Full interactive USER mode assign + persist + reload flow requires file system and live TUI"
  - test: "Open TUI, Ctrl+P, navigate to 'Stack Mean (4 values)', Enter to load; push 1 ENTER 2 ENTER 3 ENTER 4; press F5 to run; verify X=2.5"
    expected: "Program loads from library overlay; Stack Mean produces 2.5 for inputs [1,2,3,4]"
    why_human: "Regression guard for SC-5 original fix — interactive overlay + program load + run flow"
---

# Phase 5: Persistence & UX — Re-Verification Report

**Phase Goal:** Users can save and reload complete calculator state between sessions, auto-save fires every 30 seconds, an inline help system is accessible from within the TUI, USER mode works with persisted key assignments, and a bundled sample program library is ready to load.
**Verified:** 2026-05-07T17:40:47Z
**Status:** gaps_found
**Re-verification:** Yes — after gap-closure plans 05-09 (program bugs) and 05-10 ('q' routing)

## Goal Achievement

### Observable Truths (Success Criteria from ROADMAP.md)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | User can save full calculator state (stack, registers, programs, flags, USER assignments) to named JSON file and reload in fresh session | VERIFIED | `persistence.rs`: `save_state`/`load_state`/`StateFile` wrapper; `test_user_mode_roundtrip` verifies key_assignments round-trip; `test_calc_state_serde_roundtrip` (hp41-core) covers all fields; `test_is_running_reset_on_load` confirms Pitfall 4 guard |
| SC-2 | If process killed without manual save, at most 30 seconds of work is lost | VERIFIED | `check_autosave()` in `app.rs:80-88` at `Duration::from_secs(30)`; called each 16ms poll iteration in `run()` loop; save-on-exit at `app.rs:108`; `test_autosave_timer_logic` verifies with manipulated `last_save`; `test_autosave_timer_no_premature_save` confirms no early fires |
| SC-3 | User can press '?' and see searchable function reference; overlay navigable without quitting app | VERIFIED | '?' toggles overlay (`app.rs:157`); HELP_DATA has 69 entries across 12 categories; `q` now guarded with `!show_help && !show_programs && !alpha_mode && pending_input.is_none()`; overlay `q` arm is now reachable; `test_q_does_not_quit_when_help_overlay_open` confirms SC-3 closure; `test_help_scroll` verifies TableState scroll |
| SC-4 | User can assign program label to key in USER mode; assignment survives save/reload | VERIFIED | `try_user_dispatch()` in `app.rs:507`; `key_assignments: BTreeMap<char, String>` in CalcState; F1-F4 pre-wired; `test_user_mode_dispatch_runs_program` verifies assignment→run_program flow; `test_user_mode_roundtrip` in `persistence.rs` confirms round-trip |
| SC-5 | User can load at least 10 bundled sample programs and run them to produce documented outputs | PARTIAL | 10 programs exist (`test_program_count` passes); 8/10 produce documented outputs. Prime (n=2..13), Fibonacci, Factorial, Quadratic, Newton Root, Stack Mean (2.5 for [1,2,3,4] confirmed), Deg-to-Rad, Countdown are verified correct. GCD (CR-02: missing Op::Int) and Stack Stats (CR-03: inverted XGtY/XySwap) produce wrong outputs for typical inputs — no behavioral tests cover these programs. |

**Score:** 4/5 truths verified (SC-5 partial → gap)

### Deferred Items

None. No later phases address sample program behavioral correctness.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-cli/src/persistence.rs` | save_state, load_state, default_state_path, StateFile, 6 tests | VERIFIED | All 6 persistence tests pass; `is_running=false` reset on load confirmed |
| `hp41-cli/src/app.rs` | PendingInput enum, check_autosave(), guarded 'q' quit, handle_pending_input, try_user_dispatch | VERIFIED | All methods present; 8 app::tests pass including 3 overlay-guard tests |
| `hp41-cli/src/main.rs` | --state-file clap arg, load_state on startup | VERIFIED | `state_file: Option<PathBuf>`, `persistence::load_state` and fallback wired |
| `hp41-cli/src/help_data.rs` | HELP_DATA const ≥50 entries, 10 categories | VERIFIED | 69 entries; all 3 help_data tests pass |
| `hp41-cli/src/programs.rs` | SampleProgram, sample_programs(), 10 programs via OnceLock | PARTIAL | 10 programs exist; 7 programs::tests pass; 2 programs have behavioral bugs (GCD, Stack Stats) |
| `hp41-cli/src/ui.rs` | render_help_overlay, render_programs_overlay, USER annunciator wiring | VERIFIED | Both overlays present; `st.user_mode` wired; 2 ui::tests pass |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `main.rs` | `persistence::load_state` | startup load sequence | VERIFIED | `persistence::load_state(&state_path)` with fallback to `CalcState::new()` |
| `app.rs run()` | `app.rs check_autosave()` | each poll iteration | VERIFIED | `self.check_autosave()` called after poll block in `run()` |
| `app.rs check_autosave()` | `persistence::save_state` | 30s elapsed check | VERIFIED | `elapsed() >= Duration::from_secs(30)` triggers `persistence::save_state(...)` |
| `app.rs handle_key()` | show_help guard | `!show_help && !show_programs && !alpha_mode && pending_input.is_none()` compound guard | VERIFIED | grep confirms 4-condition guard at line 130-134 |
| `ui.rs render_ui()` | `render_help_overlay()` | `if app.show_help` | VERIFIED | Called post-main-panels for correct z-ordering |
| `app.rs try_user_dispatch()` | `hp41_core::run_program()` | `key_assignments.get(&c)` lookup | VERIFIED | `test_user_mode_dispatch_runs_program` confirms end-to-end |
| `programs.rs prime_test_ops` | `Test(XLtY)` loop exit | no XySwap before Test(XLeY) early-exit | VERIFIED | `test_prime_test_correctness` passes for n=2,3,4,9,13 |
| `programs.rs gcd_ops` | Op::Int for floor division | Div without Int truncation | FAILED | `Op::Int` missing in modulo step; `a - b*(a/b)` uses real division — wrong for most integer pairs (e.g. GCD(12,8)=8 instead of 4) |
| `programs.rs stack_stats_ops` | correct max/min tracking | `Test(XGtY)+XySwap` stores smaller as max | FAILED | Code review CR-03 trace: T=4,Z=3,Y=2,X=5 → R05=2 (should be 5); no behavioral test exists |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `persistence.rs save_state` | `&CalcState` | App.state (live state) | Yes — serializes all fields to JSON | FLOWING |
| `persistence.rs load_state` | `StateFile.state` | `serde_json::from_reader(fs::File::open(path))` | Yes — reads from disk, resets is_running | FLOWING |
| `ui.rs render_help_overlay` | `help_data::HELP_DATA` | compile-time static array | Yes — 69 entries | FLOWING |
| `ui.rs render_programs_overlay` | `programs::sample_programs()` | OnceLock-initialized Vec | Yes — 10 programs | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| programs::tests: 7 pass | `cargo test -p hp41-cli -- programs::tests` | 7 passed, 0 failed | PASS |
| app::tests: 8 pass (inc. 3 overlay-guard tests) | `cargo test -p hp41-cli -- app::tests` | 8 passed, 0 failed | PASS |
| persistence::tests: 6 pass | `cargo test -p hp41-cli -- persistence::tests` | 6 passed, 0 failed | PASS |
| help_data::tests: 3 pass | `cargo test -p hp41-cli -- help_data::tests` | 3 passed, 0 failed | PASS |
| ui::tests: 2 pass | `cargo test -p hp41-cli -- ui::tests` | 2 passed, 0 failed | PASS |
| Full hp41-cli suite | `cargo test -p hp41-cli` | 47 passed, 0 failed | PASS |
| Full hp41-core suite | `cargo test -p hp41-core` | 288 passed, 0 failed | PASS |
| Clippy clean | `cargo clippy -p hp41-core -p hp41-cli --all-targets -- -D warnings` | No issues found | PASS |
| Coverage hp41-core ≥80% | `just ci` (llvm-cov) | 81.73% lines, 83.79% overall | PASS |
| q-closes-help-overlay (SC-3) | `cargo test -- app::tests::test_q_does_not_quit_when_help_overlay_open` | 1 passed | PASS |
| prime_test_correctness (SC-5) | `cargo test -- programs::tests::test_prime_test_correctness` | 1 passed (n=2,3,4,9,13) | PASS |
| stack_mean_correctness (SC-5) | `cargo test -- programs::tests::test_stack_mean_correctness` | 1 passed (2.5 for [1,2,3,4]) | PASS |
| GCD behavioral correctness | gcd(12,8) expected 4 | No test exists — manual analysis shows CR-02 bug | FAIL |
| Stack Stats behavioral correctness | X=min, Y=max for known inputs | No test exists — code review CR-03 trace shows inversion | FAIL |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| PERS-01 | 05-01, 05-03 | Save/load programs and full calc state to versioned JSON | SATISFIED | persistence.rs + 6 tests + serde round-trip test in hp41-core |
| PERS-02 | 05-03 | Auto-save every 30s and on graceful shutdown | SATISFIED | check_autosave() + run() save-on-exit + 2 timer tests |
| UX-01 | 05-04, 05-05, 05-10 | Built-in function reference accessible from TUI | SATISFIED | HELP_DATA 69 entries; '?' toggle; 'q' guard fixed; 3 tests pass |
| UX-02 | 05-02, 05-06, 05-07 | USER mode with persisted key assignments | SATISFIED | try_user_dispatch + key_assignments BTreeMap + persistence round-trip test |
| UX-03 | 05-04, 05-09 | ≥10 bundled sample programs running to documented outputs | PARTIAL | 10 programs exist; 8/10 produce documented outputs; GCD (CR-02) and Stack Stats (CR-03) are broken |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-cli/src/programs.rs` | 246-249 | `Div` without `Op::Int` in gcd_ops modulo step | Blocker | GCD wrong for most integer pairs; documented as GCD algorithm |
| `hp41-cli/src/programs.rs` | 329-343 | `Test(XGtY)` + `XySwap` inverted — stores smaller as max | Blocker | Stack Stats produces X=max, Y=min instead of documented X=min, Y=max |
| `hp41-cli/src/programs.rs` | 160 | `Test(XLeY)` classifies n=0 and n=1 as prime | Warning | Mathematical edge case: 0 and 1 are wrongly classified; test suite omits n=0,n=1 |
| `hp41-cli/src/app.rs` | 172-187 | STO/RCL/Ctrl+A modal triggers activate before overlay guard check | Warning | Modals can become active while help/programs overlay is visible; silent state confusion |
| `hp41-cli/src/programs.rs` | 439-444 | `dedup()` only catches consecutive duplicates in name-unique test | Info | Test weakness; not a current bug (all 10 names are unique and non-consecutive) |

### Human Verification Required

#### 1. Interactive Help Overlay Close via 'q'

**Test:** Run `cargo run -p hp41-cli`, press `?` to open help overlay, press `q`
**Expected:** Overlay closes; app continues running; pressing `q` a second time (no overlay open) quits normally
**Why human:** Unit test `test_q_does_not_quit_when_help_overlay_open` confirms the routing logic. Interactive confirmation validates the full rendering/event path including crossterm key delivery on the test machine.

#### 2. USER Mode Full Round-Trip (Persist + Reload)

**Test:** Toggle USER mode (`u`), assign key via Ctrl+A (press `z`, type `MYPROG`, Enter), save with Ctrl+S, quit, relaunch with `--state-file ~/.hp41/autosave.json`, verify USER annunciator still shows, press `z` to confirm it runs MYPROG
**Expected:** Key assignment persists across save/reload cycle; USER mode state preserved; assigned key triggers run_program
**Why human:** Requires interactive TUI + file system + live program execution confirmation

#### 3. Program Library Load + Run Flow

**Test:** Open TUI, press Ctrl+P, navigate to "Stack Mean (4 values)", press Enter to load, push 1 ENTER 2 ENTER 3 ENTER 4, press F5 to run, verify X=2.5
**Expected:** Program loads from library; Stack Mean produces 2.5 for inputs [1,2,3,4]
**Why human:** Regression guard for original SC-5 fix; interactive overlay + program load + run flow

### Gaps Summary

**Previous gaps SC-3 and SC-5 (original) were closed by plans 05-09 and 05-10.** All three originally-identified SC-5 bugs (prime_test XySwap, mean_sdev unconditional R00, quadratic comment mismatch) were fixed. The SC-3 'q' routing dead-code issue was fixed with the compound guard and confirmed by 3 unit tests.

**Remaining gap: SC-5 PARTIAL — 2 programs with behavioral bugs discovered post-gap-closure:**

A code review (05-REVIEW.md) identified two BLOCKER-severity bugs in sample programs not covered by the original gap analysis and not addressed by plans 05-09/05-10:

1. **CR-02 — gcd_ops missing Op::Int:** The Euclidean algorithm modulo step `a - b*(a/b)` lacks `Op::Int` to floor the quotient. `Op::Int` was correctly added to `prime_test_ops` in plan 05-09 but `gcd_ops` was not updated. GCD(12,8) returns 8 (wrong; should be 4). GCD(7,3) loops incorrectly. Most integer pairs with a non-integer quotient will produce wrong results.

2. **CR-03 — stack_stats_ops inverted comparison:** `Test(XGtY)` + `XySwap` fires XySwap when X>Y (TRUE), moving the SMALLER value into X before `StoReg(5)`. This systematically stores the smaller value as the "max" candidate. The min-finding section has the same inversion. The program produces X=max, Y=min instead of the documented X=min, Y=max. No behavioral test covers this program.

**Fix plan:** A single plan should fix both CR-02 and CR-03 in `hp41-cli/src/programs.rs` and add behavioral tests for both programs. Root cause: the gap-closure test suite (05-09) focused on the three explicitly-identified programs and did not add tests for the remaining seven programs.

---

_Verified: 2026-05-07T17:40:47Z_
_Verifier: Claude (gsd-verifier)_
_Re-verification: Yes — after gap-closure plans 05-09 and 05-10_
