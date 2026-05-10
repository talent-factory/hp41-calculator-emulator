---
phase: 11-print-emulation
plan: "02"
subsystem: cli-ui
tags: [rust, hp41-cli, print, modal, file-logging, clap, ratatui]

# Dependency graph
requires:
  - phase: 11-01
    provides: print_buffer on CalcState, Op::PRX/PRA/PRSTK in dispatch() and execute_op()
provides:
  - PendingInput::PrintModal variant for 'P'-prefix print modal
  - print_log_writer: Option<BufWriter<File>> on App for --print-log file output
  - App::new() accepting print_log: Option<PathBuf> third argument
  - call_dispatch_and_drain method draining print_buffer to app.message and file
  - 'P' interceptor in handle_key() routing to PrintModal (D-06)
  - PrintModal arm in handle_pending_input() with x/X/a/A/s/S/Esc routing (D-07)
  - PRNT: _ pending_prompt display string (D-08)
  - --print-log FILE CLI argument in Cli struct (D-05)
  - Print category in HELP_DATA with P X/PRX, P A/PRA, P S/PRSTK entries (D-09)
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Print modal follows the 'P'-prefix 'S'-prefix pattern from Phase 10 STO modals"
    - "call_dispatch_and_drain wraps dispatch + buffer drain — clean single-responsibility method"
    - "open failure captured via initial_message into app.message — no panic (D-04)"
    - "BufWriter<File> with create(true).append(true) for safe log file writes"

key-files:
  created: []
  modified:
    - hp41-cli/src/app.rs
    - hp41-cli/src/main.rs
    - hp41-cli/src/ui.rs
    - hp41-cli/src/help_data.rs
    - hp41-cli/src/tests/keys_tests.rs

key-decisions:
  - "call_dispatch_and_drain as new method on App — drain only happens for print ops; other dispatch paths unaffected"
  - "initial_message captures open-failure error and feeds app.message in App::new() struct literal"
  - "PRNT: _ arm added to ui.rs pending_prompt() for PrintModal display (D-08)"
  - "Print category added before Help in HELP_DATA; category test updated from 13 to 14"

patterns-established:
  - "Drain print_buffer immediately after print op dispatch in call_dispatch_and_drain"

requirements-completed: [PRNT-01, PRNT-02, PRNT-03, PRNT-04]

# Metrics
duration: ~15min
completed: 2026-05-08
---

# Phase 11 Plan 02: Print CLI Wire-Up Summary

**PrintModal 'P'-prefix modal + call_dispatch_and_drain + --print-log BufWriter — 494 workspace tests GREEN**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-08T18:20:00Z
- **Completed:** 2026-05-08T18:35:14Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `PendingInput::PrintModal` to `PendingInput` enum in app.rs
- Added `print_log_writer: Option<BufWriter<File>>` field to `App` struct
- Updated `App::new()` signature to accept `print_log: Option<PathBuf>` as 3rd argument
- File open failure captured via `initial_message` into `app.message` — no panic (D-04)
- Added 'P' (Shift+P) interceptor in `handle_key()` before `key_to_op()` (D-06)
- Added `PrintModal` arm to `handle_pending_input()`: x/X → PRX, a/A → PRA, s/S → PRSTK, Esc → cancel (D-07)
- Added `call_dispatch_and_drain()` method: dispatch + drain print_buffer + write to file + set app.message
- Added `--print-log FILE` arg to `Cli` struct in main.rs with correct `#[arg(long, value_name = "FILE")]` (D-05)
- Added `PrintModal => "PRNT: _"` arm to `pending_prompt()` in ui.rs (D-08)
- Added Print category header + 3 entries (P X/PRX, P A/PRA, P S/PRSTK) to HELP_DATA (D-09)
- Updated `test_all_thirteen_categories_present` to `test_all_fourteen_categories_present` (Pitfall 4 resolved)
- 4 new tests for: modal routing (PRX), Esc cancel, file append, invalid path error message
- Full workspace: 494 tests GREEN, 0 failures

## Task Commits

1. **Task 1: PrintModal PendingInput, print_log_writer, call_dispatch_and_drain** - `c8b970d` (feat)
2. **Task 2: --print-log CLI arg, PRNT:_ display, Print help category** - `33235ee` (feat)

## Files Created/Modified

- `hp41-cli/src/app.rs` - Added PrintModal variant, print_log_writer field, App::new() signature change, 'P' interceptor, PrintModal handle arm, call_dispatch_and_drain method, new_for_test() updated, 4 new tests
- `hp41-cli/src/main.rs` - Added --print-log field to Cli struct; updated App::new() call to cli.print_log
- `hp41-cli/src/ui.rs` - Added PrintModal arm in pending_prompt(); updated test helper App::new() call
- `hp41-cli/src/help_data.rs` - Added Print category header + 3 entries; renamed test to test_all_fourteen_categories_present
- `hp41-cli/src/tests/keys_tests.rs` - Updated make_app() to pass None as 3rd arg to App::new()

## Decisions Made

- `call_dispatch_and_drain` is a separate method (not modifying `call_dispatch`) — keeps non-print dispatch paths clean
- `initial_message` pattern: App::new() computes open-failure message as a local variable and uses it in the struct literal, making the error visible in the TUI status bar immediately on startup
- `let _ = writeln!` and `let _ = writer.flush()` — best-effort file writes per D-04 (never panic on write failure)
- Print category placed before Help in HELP_DATA for logical grouping with other operator categories

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed compile errors in ui.rs, main.rs, keys_tests.rs as part of Task 1**
- **Found during:** Task 1 compile verification
- **Issue:** Changing `App::new()` signature from 2-arg to 3-arg broke compilation in ui.rs (non-exhaustive match on PrintModal in pending_prompt()), main.rs (wrong arg count), and hp41-cli/src/tests/keys_tests.rs (wrong arg count)
- **Fix:** Added minimal PrintModal arm to ui.rs `pending_prompt()` (the full D-08 fix from Task 2); updated main.rs to pass `None` for print_log (upgraded to `cli.print_log` in Task 2); updated keys_tests.rs make_app() to pass `None`
- **Files modified:** hp41-cli/src/ui.rs, hp41-cli/src/main.rs, hp41-cli/src/tests/keys_tests.rs
- **Impact:** Zero scope creep; these are exactly the changes Task 2 specifies — only the sequencing was adjusted for compilation. Task 2 then completed the full main.rs wiring (`cli.print_log`)
- **Committed in:** c8b970d (Task 1 commit, minimal fixes bundled)

---

**Total deviations:** 1 auto-fixed (Rule 3 — compile blockers requiring companion file updates)

## Known Stubs

None — all print modal functionality is fully wired. PRX/PRA/PRSTK dispatch through the real hp41-core ops and drain the print_buffer to app.message. File logging is active when --print-log is specified.

## Threat Flags

No new threat surface beyond the T-11-02-01/02/03 entries documented in the plan's threat_model. All three mitigations are implemented:
- T-11-02-03: `let _ = writeln!; let _ = writer.flush()` — errors discarded (best-effort, no panic)

## Self-Check

Files created/modified:
- [FOUND] hp41-cli/src/app.rs
- [FOUND] hp41-cli/src/main.rs
- [FOUND] hp41-cli/src/ui.rs
- [FOUND] hp41-cli/src/help_data.rs
- [FOUND] hp41-cli/src/tests/keys_tests.rs

Commits:
- [FOUND] c8b970d (Task 1)
- [FOUND] 33235ee (Task 2)
