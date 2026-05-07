---
phase: 05-persistence-and-ux
verified: 2026-05-07T20:00:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 4/5
  gaps_closed:
    - "SC-3: 'q' in help overlay no longer quits app — guarded with !show_help && !show_programs && !alpha_mode && pending_input.is_none()"
    - "SC-5 prime_test_ops: spurious XySwap removed; Op::Int added for exact modulo; loop exit XGtY→XLtY fixed"
    - "SC-5 mean_sdev_ops: replaced with correct 7-op 4-value stack mean (Stack Mean (4 values))"
    - "SC-5 quadratic_ops: comment corrected to 'c in X, b in Y, a in Z'"
    - "SC-5 gcd_ops CR-02: Op::Int inserted after Op::Div at line 248 — floor-truncates BCD quotient; gcd(12,8)=4, gcd(7,3)=1, gcd(15,5)=5 verified by test_gcd_correctness"
    - "SC-5 stack_stats_ops CR-03: rewritten with R00-R03 register-save pattern; max section uses XLtY (3 occurrences), min section uses XGtY (3 occurrences); X=1 (min), Y=5 (max) for inputs [3,1,4,5] verified by test_stack_stats_correctness"
  gaps_remaining: []
  regressions: []
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

# Phase 5: Persistence & UX — Re-Verification Report (After Plan 05-11 Gap Closure)

**Phase Goal:** Users can save and reload complete calculator state between sessions, auto-save fires every 30 seconds, an inline help system is accessible from within the TUI, USER mode works with persisted key assignments, and a bundled sample program library is ready to load.
**Verified:** 2026-05-07T20:00:00Z
**Status:** passed
**Re-verification:** Yes — after gap-closure plan 05-11 (gcd_ops CR-02 and stack_stats_ops CR-03)

## Goal Achievement

### Observable Truths (Success Criteria from ROADMAP.md)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | User can save full calculator state (stack, registers, programs, flags, USER assignments) to named JSON file and reload in fresh session | VERIFIED | `persistence.rs`: `save_state`/`load_state`/`StateFile` wrapper; `test_user_mode_roundtrip` verifies key_assignments round-trip; `test_calc_state_serde_roundtrip` (hp41-core) covers all fields; `test_is_running_reset_on_load` confirms Pitfall 4 guard |
| SC-2 | If process killed without manual save, at most 30 seconds of work is lost | VERIFIED | `check_autosave()` in `app.rs:80-88` at `Duration::from_secs(30)`; called each 16ms poll iteration in `run()` loop; save-on-exit at `app.rs:108`; `test_autosave_timer_logic` verifies with manipulated `last_save`; `test_autosave_timer_no_premature_save` confirms no early fires |
| SC-3 | User can press '?' and see searchable function reference; overlay navigable without quitting app | VERIFIED | '?' toggles overlay (`app.rs:157`); HELP_DATA has 69 entries across 12 categories; `q` guarded with `!show_help && !show_programs && !alpha_mode && pending_input.is_none()`; `test_q_does_not_quit_when_help_overlay_open` confirms SC-3 closure; `test_help_scroll` verifies TableState scroll |
| SC-4 | User can assign program label to key in USER mode; assignment survives save/reload | VERIFIED | `try_user_dispatch()` in `app.rs:507`; `key_assignments: BTreeMap<char, String>` in CalcState; F1-F4 pre-wired; `test_user_mode_dispatch_runs_program` verifies assignment→run_program flow; `test_user_mode_roundtrip` in `persistence.rs` confirms round-trip |
| SC-5 | User can load at least 10 bundled sample programs and run them to produce documented outputs | VERIFIED | All 10 programs exist and produce correct documented outputs. CR-02 fixed: `Op::Int` at line 248 of `programs.rs` follows `Op::Div` in gcd_ops modulo step; `test_gcd_correctness` passes gcd(12,8)=4, gcd(7,3)=1, gcd(15,5)=5. CR-03 fixed: stack_stats_ops rewritten with R00-R03 register-save; max section uses XLtY (lines 340,344,348); min section uses XGtY (lines 354,358,362); `test_stack_stats_correctness` passes X=1 (min), Y=5 (max) for inputs [3,1,4,5]. hp41-cli test suite: 49 passed, 0 failed. |

**Score:** 5/5 truths verified

### Deferred Items

None.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-cli/src/persistence.rs` | save_state, load_state, default_state_path, StateFile, 6 tests | VERIFIED | All 6 persistence tests pass; `is_running=false` reset on load confirmed |
| `hp41-cli/src/app.rs` | PendingInput enum, check_autosave(), guarded 'q' quit, handle_pending_input, try_user_dispatch | VERIFIED | All methods present; 8 app::tests pass including 3 overlay-guard tests |
| `hp41-cli/src/main.rs` | --state-file clap arg, load_state on startup | VERIFIED | `state_file: Option<PathBuf>`, `persistence::load_state` and fallback wired |
| `hp41-cli/src/help_data.rs` | HELP_DATA const ≥50 entries, 10 categories | VERIFIED | 69 entries; all 3 help_data tests pass |
| `hp41-cli/src/programs.rs` | SampleProgram, sample_programs(), 10 programs via OnceLock, behavioral tests for gcd and stack_stats | VERIFIED | 10 programs exist; 9 programs::tests pass (7 prior + 2 new: test_gcd_correctness, test_stack_stats_correctness) |
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
| `programs.rs gcd_ops` | Op::Int for floor division | `Op::Div` at line 246 immediately followed by `Op::Int` at line 248 | VERIFIED | CR-02 closed; `test_gcd_correctness` passes gcd(12,8)=4, gcd(7,3)=1, gcd(15,5)=5 |
| `programs.rs stack_stats_ops` | correct max/min tracking via XLtY (max) and XGtY (min) | register-save R00-R03 pattern then pairwise RclReg comparisons | VERIFIED | CR-03 closed; lines 340/344/348 use XLtY; lines 354/358/362 use XGtY; `test_stack_stats_correctness` passes X=1, Y=5 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `persistence.rs save_state` | `&CalcState` | App.state (live state) | Yes — serializes all fields to JSON | FLOWING |
| `persistence.rs load_state` | `StateFile.state` | `serde_json::from_reader(fs::File::open(path))` | Yes — reads from disk, resets is_running | FLOWING |
| `ui.rs render_help_overlay` | `help_data::HELP_DATA` | compile-time static array | Yes — 69 entries | FLOWING |
| `ui.rs render_programs_overlay` | `programs::sample_programs()` | OnceLock-initialized Vec | Yes — 10 programs, all behaviorally correct | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| programs::tests: 9 pass (inc. 2 new) | `cargo test -p hp41-cli -- programs::tests` | 9 passed, 0 failed | PASS |
| test_gcd_correctness | `cargo test -p hp41-cli -- programs::tests::test_gcd_correctness` | 1 passed — gcd(12,8)=4, gcd(7,3)=1, gcd(15,5)=5 | PASS |
| test_stack_stats_correctness | `cargo test -p hp41-cli -- programs::tests::test_stack_stats_correctness` | 1 passed — X=1 (min), Y=5 (max) | PASS |
| app::tests: 8 pass (inc. 3 overlay-guard tests) | `cargo test -p hp41-cli -- app::tests` | 8 passed, 0 failed | PASS |
| persistence::tests: 6 pass | `cargo test -p hp41-cli -- persistence::tests` | 6 passed, 0 failed | PASS |
| help_data::tests: 3 pass | `cargo test -p hp41-cli -- help_data::tests` | 3 passed, 0 failed | PASS |
| ui::tests: 2 pass | `cargo test -p hp41-cli -- ui::tests` | 2 passed, 0 failed | PASS |
| Full hp41-cli suite | `cargo test -p hp41-cli` | 49 passed, 0 failed | PASS |
| Full hp41-core suite | `cargo test -p hp41-core` | 288 passed, 0 failed | PASS |
| prime_test_correctness (SC-5) | `cargo test -- programs::tests::test_prime_test_correctness` | 1 passed (n=2,3,4,9,13) | PASS |
| stack_mean_correctness (SC-5) | `cargo test -- programs::tests::test_stack_mean_correctness` | 1 passed (2.5 for [1,2,3,4]) | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| PERS-01 | 05-01, 05-03 | Save/load programs and full calc state to versioned JSON | SATISFIED | persistence.rs + 6 tests + serde round-trip test in hp41-core |
| PERS-02 | 05-03 | Auto-save every 30s and on graceful shutdown | SATISFIED | check_autosave() + run() save-on-exit + 2 timer tests |
| UX-01 | 05-04, 05-05, 05-10 | Built-in function reference accessible from TUI | SATISFIED | HELP_DATA 69 entries; '?' toggle; 'q' guard fixed; 3 tests pass |
| UX-02 | 05-02, 05-06, 05-07 | USER mode with persisted key assignments | SATISFIED | try_user_dispatch + key_assignments BTreeMap + persistence round-trip test |
| UX-03 | 05-04, 05-09, 05-11 | ≥10 bundled sample programs running to documented outputs | SATISFIED | 10 programs exist; all 10 produce documented outputs; test_gcd_correctness and test_stack_stats_correctness confirm CR-02 and CR-03 fixes; 9 programs::tests pass |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-cli/src/programs.rs` | 160 | `Test(XLeY)` classifies n=0 and n=1 as prime | Warning | Mathematical edge case: 0 and 1 are wrongly classified as prime; test suite omits n=0,n=1 — does not affect SC-5 (documented behavior is for inputs ≥2) |
| `hp41-cli/src/app.rs` | 172-187 | STO/RCL/Ctrl+A modal triggers activate before overlay guard check | Warning | Modals can become active while help/programs overlay is visible; silent state confusion — not a blocker for SC-5 |
| `hp41-cli/src/programs.rs` | 439-444 | `dedup()` only catches consecutive duplicates in name-unique test | Info | Test weakness; not a current bug (all 10 names are unique and non-consecutive) |

No blocker-severity anti-patterns remain. The two blockers from the previous verification (gcd_ops missing Op::Int; stack_stats_ops inverted tests) are both resolved and confirmed by behavioral tests.

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

All gaps are closed. The two BLOCKER-severity behavioral bugs discovered by post-gap-closure code review are resolved:

**CR-02 (gcd_ops):** `Op::Int` is present at line 248 of `programs.rs`, immediately after `Op::Div` at line 246. The comment was updated from "approximate floor via truncation: use as-is" to "floor-truncate quotient then multiply back for exact integer modulo". `test_gcd_correctness` confirms correct results for three representative inputs including the non-trivial case gcd(7,3)=1 that previously infinite-looped.

**CR-03 (stack_stats_ops):** The function was rewritten with a register-save approach (R00-R03 via Rdn cycling) that guarantees all four stack values are compared. The original Enter+Rdn loop never surfaced Z to X, causing the minimum value to be missed. The new algorithm: max section uses XLtY at lines 340, 344, 348; min section uses XGtY at lines 354, 358, 362. This matches the plan key_links exactly. `test_stack_stats_correctness` confirms X=1 (min), Y=5 (max) for inputs [3,1,4,5].

The hp41-cli test suite grew from 47 to 49 tests with the two new behavioral tests. hp41-core at 288 tests. All 337 workspace tests pass.

SC-5 (UX-03) is fully satisfied. Phase 5 goal is achieved.

---

_Verified: 2026-05-07T20:00:00Z_
_Verifier: Claude (gsd-verifier)_
_Re-verification: Yes — after gap-closure plans 05-09, 05-10, and 05-11_
