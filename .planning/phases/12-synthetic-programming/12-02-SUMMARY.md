---
phase: 12-synthetic-programming
plan: "02"
subsystem: cli
tags: [rust, hp41-cli, synthetic-programming, hex-modal, getkey, hidden-registers, keyboard-wiring]

# Dependency graph
requires:
  - phase: 12-synthetic-programming
    plan: "01"
    provides: "Op::GetKey, Op::Null, StoM/N/O, RclM/N/O, Op::SyntheticByte(u8), synthetic_byte_to_op(), CalcState.last_key_code, reg_m/n/o fields"
provides:
  - "keycode_to_hp41_code() — HP-41 hardware key code lookup in hp41-cli/src/keys.rs"
  - "PendingInput::HexModal — 2-digit hex byte insertion modal"
  - "last_key_code update on every key press (D-01, SYNT-01)"
  - "M/N/O hidden register dispatch via STO/RCL modals"
  - "HEX modal display in ui.rs pending_prompt()"
  - "Synthetic Programming category in help_data.rs (7 entries)"
affects:
  - "hp41-cli/src/app.rs — PendingInput, handle_key, handle_pending_input"
  - "hp41-cli/src/keys.rs — keycode_to_hp41_code(), KEY_REF_TABLE"
  - "hp41-cli/src/ui.rs — pending_prompt() exhaustive match"
  - "hp41-cli/src/help_data.rs — HELP_DATA category list"

# Tech stack
added: []
patterns:
  - "2-digit hex accumulation modal mirroring StoRegister pattern (PendingInput::HexModal(String))"
  - "acc.is_empty() guard for M/N/O first-char dispatch in StoRegister/RclRegister arms"
  - "Row×10+col HP-41 keyboard encoding in keycode_to_hp41_code()"

# Key files
created: []
modified:
  - hp41-cli/src/keys.rs
  - hp41-cli/src/app.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/src/help_data.rs
  - hp41-cli/src/tests/keys_tests.rs

# Key decisions
decisions:
  - "HexModal arm added to ui.rs in Task 2 commit (not Task 3) to unblock compilation"
  - "Wave 1 work merged from develop branch before starting execution (worktree was created before wave 1 landed)"
  - "test_all_fourteen_categories_present renamed to test_all_fifteen_categories_present"
  - "KEY_REF_TABLE count test updated from 54 to 55 (deviation: pre-existing test needed repair)"

# Metrics
duration_seconds: 562
completed_date: "2026-05-09T07:17:58Z"
tasks_completed: 3
tasks_total: 3
files_modified: 5
---

# Phase 12 Plan 02: CLI Wiring for Synthetic Programming Summary

HP-41 CLI interactive layer wired for synthetic programming: GETKEY key tracking via `keycode_to_hp41_code()`, hidden register M/N/O modal extensions in STO/RCL flows, hex-byte insertion modal (PendingInput::HexModal), and help/UI display updates.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add keycode_to_hp41_code() and KEY_REF_TABLE entry | 310f496 | hp41-cli/src/keys.rs |
| 2 | HexModal variant + last_key_code + 'X' interceptor + M/N/O branches + HexModal handler | 03e91a3 | hp41-cli/src/app.rs, hp41-cli/src/ui.rs, hp41-cli/src/tests/keys_tests.rs |
| 3 | Synthetic Programming category in help_data.rs | 18841f7 | hp41-cli/src/help_data.rs |

## What Was Built

**keycode_to_hp41_code() (keys.rs):** Maps crossterm `KeyCode` values to HP-41 hardware key codes (row×10+col, 1-indexed). Covers all 8 rows of the HP-41C keyboard: digit rows (rows 5-8), arithmetic ops, STO/RCL/SIN/COS/TAN, and function keys. Returns 0 for unmapped keys (GETKEY returns 0 when no key pressed, per D-03).

**PendingInput::HexModal (app.rs):** New variant added to the PendingInput enum. `'X'` (uppercase/Shift+X) opens the modal in PRGM mode only. First hex digit is accumulated; on second hex digit, the 2-digit hex byte is parsed and validated via `synthetic_byte_to_op()`. Valid bytes insert `Op::SyntheticByte(byte)` at `state.pc` and advance pc. Invalid bytes set `app.message = "INVALID"` and leave the program Vec unchanged. Esc cancels cleanly. Non-hex keys silently restore the modal.

**last_key_code update (app.rs):** `self.state.last_key_code = keys::keycode_to_hp41_code(key.code)` placed as the first action after the release filter in `handle_key()` — before Ctrl+C, pending_input dispatch, and key_to_op(). Ensures every key press (including digits, modal navigation) updates the code read by Op::GetKey.

**M/N/O modal extensions (app.rs):** StoRegister and RclRegister arms in `handle_pending_input()` now check for M/N/O as the first character (before any digit is typed, via `acc.is_empty()` guard). Pressing M/N/O immediately dispatches Op::StoM/StoN/StoO or Op::RclM/RclN/RclO and closes the modal. Lowercase m/n/o also dispatches. If a digit was already typed, M/N/O falls through to the normal digit accumulation path.

**HEX display (ui.rs):** `pending_prompt()` now handles `PendingInput::HexModal(acc)` — renders `"HEX: __"` when empty, `"HEX: n_"` after first digit.

**Help data (help_data.rs):** `=== Synthetic Programming ===` category added with 7 entries: X nn (HEX modal), S M/S N/S O (STO hidden regs), R M/R N/R O (RCL hidden regs). Total category count: 15.

## Test Coverage

8 new inline tests in `app::synthetic_modal_tests`:
- `test_last_key_code_updated_on_press` — '5' sets last_key_code = 62
- `test_hex_modal_opens_only_in_prgm_mode` — 'X' outside PRGM = no-op, inside PRGM = opens HexModal
- `test_hex_modal_invalid_byte_rejects_with_message` — 0x00 → INVALID, program unchanged
- `test_hex_modal_valid_byte_inserts_synthetic` — 0xCE → SyntheticByte(0xCE) inserted at pc, pc advances
- `test_hex_modal_esc_cancels_cleanly` — Esc closes modal, no program change, no INVALID message
- `test_sto_m_via_modal` — S then M dispatches Op::StoM, stores X to reg_m
- `test_rcl_m_via_modal` — R then m (lowercase) dispatches Op::RclM, recalls reg_m to X
- `test_mno_guard_only_when_acc_empty` — StoRegister("0") + M does not dispatch StoM (guard works)

All 94 hp41-cli tests pass. Full workspace test suite passes (150+ tests across hp41-core and hp41-cli).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ui.rs HexModal arm added with Task 2 (not Task 3)**
- **Found during:** Task 2 — adding PendingInput::HexModal to app.rs caused a non-exhaustive match compile error in ui.rs
- **Issue:** The `pending_prompt()` function in ui.rs is an exhaustive match on PendingInput; adding a new variant without a match arm is a compile error
- **Fix:** Added the HexModal arm to `pending_prompt()` immediately during Task 2 execution to restore compilation; committed with Task 2 changes
- **Files modified:** hp41-cli/src/ui.rs
- **Commit:** 03e91a3

**2. [Rule 1 - Bug] KEY_REF_TABLE count test expected wrong value**
- **Found during:** Task 2 test run
- **Issue:** `key_ref_table_has_33_entries` expected 54 entries but Task 1 added 1 new entry (making it 55)
- **Fix:** Updated count assertion from 54 to 55, updated comment to reflect Phase 12 addition
- **Files modified:** hp41-cli/src/tests/keys_tests.rs
- **Commit:** 03e91a3

**3. [Rule 3 - Blocking] Wave 1 work merged from develop before starting**
- **Found during:** Pre-execution check — worktree was created from commit `dead665` (before wave 1 ran)
- **Issue:** hp41-core wave 1 changes (Op::GetKey, SyntheticByte, etc.) not present in worktree
- **Fix:** `git merge develop` fast-forwarded to include commits `dc55132`, `b82c288`, `f5bbcfb`, `24757e7`, `67d05ed` (waves 0 and 1)
- **Impact:** None — clean fast-forward merge, no conflicts

### Category Test Rename

The test `test_all_fourteen_categories_present` was renamed to `test_all_fifteen_categories_present` per plan step 3.3. The new category `=== Synthetic Programming ===` was added to the test array.

## Threat Surface Scan

All threat mitigations from the plan's threat_model are implemented:

| Threat ID | Status | Evidence |
|-----------|--------|---------|
| T-12-W2-01 | Mitigated | `c.is_ascii_hexdigit()` guard in HexModal arm — only 0-9, a-f, A-F enter accumulator |
| T-12-W2-02 | Mitigated | `synthetic_byte_to_op(byte)` called before program.insert(); None branch sets INVALID and leaves Vec unchanged |
| T-12-W2-03 | Accepted | `.expect("two ASCII hex digits must parse as u8")` — invariant guaranteed by 2-char length check + hexdigit guard |
| T-12-W2-04 | Mitigated | 'X' interceptor gated on `self.state.prgm_mode`; test_hex_modal_opens_only_in_prgm_mode enforces both branches |
| T-12-W2-05 | Mitigated | Inherited from Plan 12-01; synthetic_byte_to_op() never returns SyntheticByte(_) |
| T-12-W2-06 | Accepted | Inherited acceptance from Plan 12-01 T-12-W1-03 |

No new unplanned threat surfaces introduced.

## Self-Check: PASSED

Files exist:
- hp41-cli/src/keys.rs: keycode_to_hp41_code function confirmed present
- hp41-cli/src/app.rs: HexModal variant, last_key_code update, 'X' interceptor, M/N/O branches confirmed
- hp41-cli/src/ui.rs: HexModal arm in pending_prompt confirmed
- hp41-cli/src/help_data.rs: Synthetic Programming category with 7 entries confirmed

Commits exist:
- 310f496: feat(12-02): add keycode_to_hp41_code() and KEY_REF_TABLE hex modal entry
- 03e91a3: feat(12-02): wire HexModal, last_key_code, M/N/O modal in app.rs and ui.rs
- 18841f7: feat(12-02): add Synthetic Programming category to help_data.rs

Quality gates: `just build` green, `just test` green (94 hp41-cli + 150+ hp41-core), `just lint` green.
