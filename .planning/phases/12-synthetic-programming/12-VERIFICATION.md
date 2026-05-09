---
phase: 12-synthetic-programming
verified: 2026-05-09T07:45:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Manually press any key in the TUI, then run a GETKEY program and confirm X shows the correct HP-41 row-column code"
    expected: "Pressing '5' sets last_key_code=62; a program containing GETKEY pushes 62 to X"
    why_human: "keycode_to_hp41_code() dispatch and the last_key_code→program path cannot be exercised without running the TUI binary"
  - test: "In PRGM mode, press Shift+X, then type 'C' 'F'. Confirm program shows 'SYN CF' and pc advances. Then type 'C' 'E' to insert GETKEY."
    expected: "Valid hex codes (0xCF, 0xCE) insert SyntheticByte step; program listing shows 'SYN CF' / 'SYN CE'; pc advances by 1"
    why_human: "HexModal insertion flow requires live TUI interaction to verify the display string and pc state together"
  - test: "In PRGM mode, press Shift+X then '0' '0'. Confirm display shows 'INVALID' and program is unchanged."
    expected: "Byte 0x00 is not in the safe subset; app.message = 'INVALID'; program Vec unmodified"
    why_human: "Rejection path requires TUI interaction to verify the user-visible 'INVALID' message in the display area"
  - test: "Press Shift+X outside PRGM mode (normal mode). Confirm no HexModal opens and no error message appears."
    expected: "Uppercase 'X' is consumed silently outside PRGM mode — no modal, no side effects"
    why_human: "Mode-gate behavior needs TUI interaction to verify the 'X' key does nothing outside PRGM mode"
  - test: "Press 'S' then 'M' with X=42. Confirm reg_m=42. Press 'R' then 'M' with X=0. Confirm X=42."
    expected: "STO M and RCL M via the STO/RCL keyboard modal work correctly; acc.is_empty() guard lets M through only as first char"
    why_human: "Modal flow (StoRegister/RclRegister with M/N/O dispatch) requires TUI keyboard interaction"
  - test: "Press Esc inside HexModal after typing one hex digit. Confirm modal closes, program unchanged, no INVALID message."
    expected: "Esc cancels with zero side effects"
    why_human: "Cancellation behavior requires TUI interaction"
---

# Phase 12: Synthetic Programming Verification Report

**Phase Goal:** Users can use GETKEY, NULL, and the hidden registers M/N/O inside keystroke programs, and can insert a synthetic op via a 2-digit hex byte modal that enforces a curated safe subset, expanding programmable power without unsafe byte codes.
**Verified:** 2026-05-09T07:45:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| #   | Truth | Status | Evidence |
| --- | ----- | ------ | -------- |
| 1   | A program containing GETKEY pushes the HP-41 row-column key code of the last pressed key to X when the step executes | ✓ VERIFIED | `test_getkey_pushes_last_key_code` and `test_getkey_in_program` pass (21/21 synthetic_tests GREEN); `op_getkey()` reads `state.last_key_code` and calls `enter_number`; `execute_op()` arm for `Op::GetKey` calls `super::registers::op_getkey(state)` |
| 2   | A program containing NULL executes without changing any stack register, any numbered register, or the lift flag (Neutral stack-lift effect) | ✓ VERIFIED | `test_null_does_not_modify_stack`, `test_null_neutral_lift_effect`, `test_null_does_not_modify_regs` pass; `Op::Null` arm calls `apply_lift_effect(state, LiftEffect::Neutral)` only; execute_op arm before catch-all confirmed at program.rs:328 |
| 3   | `STO M`/`RCL M`, `STO N`/`RCL N`, `STO O`/`RCL O` work in programs identically to numbered registers and survive a JSON save/load round-trip | ✓ VERIFIED | `test_sto_m_rcl_m_round_trip`, `test_hidden_regs_serde_round_trip`, `test_calcstate_loads_without_new_fields`, `test_hidden_reg_in_program` all pass; state.rs has 4 new fields with `#[serde(default)]`; registers.rs has all 7 pub functions |
| 4   | The hex-byte insertion modal accepts a 2-digit hex code from a curated safe subset and inserts the synthetic op at the current program step; codes outside the safe subset are rejected with a visible error | ✓ VERIFIED (partial — see human_needed) | `HexModal(String)` variant added to PendingInput; `synthetic_byte_to_op(byte)` called before `program.insert()`; `None` branch sets `app.message = Some("INVALID")`; CR-01 fix applied: `insert_pos = self.state.pc.min(self.state.program.len())`; 5 app.rs inline tests pass |

**Score:** 4/4 truths verified — code evidence complete; user-facing display and modal interaction need human confirmation

### Deferred Items

None — all four SYNT requirements are addressed in this phase.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `hp41-core/tests/synthetic_tests.rs` | 21-test RED→GREEN scaffold | ✓ VERIFIED | 281 lines; 21 tests; all pass (`cargo test -p hp41-core --test synthetic_tests`: 21 passed) |
| `hp41-core/src/state.rs` | 4 new `#[serde(default)]` CalcState fields | ✓ VERIFIED | `last_key_code: u8`, `reg_m/n/o: HpNum` at lines 100–114; initialized in `new()` at lines 135–138 |
| `hp41-core/src/ops/mod.rs` | 9 new Op variants + dispatch arms + `synthetic_byte_to_op()` | ✓ VERIFIED | Variants at lines 222–240; dispatch arms at lines 413–432; `pub fn synthetic_byte_to_op()` at lines 451–485; `apply_lift_effect` in scope via line 3 |
| `hp41-core/src/ops/program.rs` | 9 new `execute_op()` arms before catch-all | ✓ VERIFIED | Arms at lines 327–345; placed before `Op::Lbl(_)` catch-all at line 347; `apply_lift_effect` in scope via line 19 |
| `hp41-core/src/ops/registers.rs` | `op_getkey` + 6 sto/rcl hidden register functions | ✓ VERIFIED | All 7 pub functions at lines 108–163; correct LiftEffect semantics (Enable for GetKey/RclM/N/O; Neutral for StoM/N/O) |
| `hp41-cli/src/prgm_display.rs` | `op_display_name()` arms for all 9 Phase 12 variants | ✓ VERIFIED | `"GETKEY"`, `"NULL"`, `"STO M/N/O"`, `"RCL M/N/O"`, `format!("SYN {:02X}", b)` at lines 129–137 |
| `hp41-cli/src/app.rs` | `HexModal(String)` in PendingInput; `last_key_code` update; 'X' interceptor; M/N/O branches; HexModal handler | ✓ VERIFIED | `HexModal(String)` at line 40; `last_key_code` update at line 167; 'X' interceptor at lines 235–243; M/N/O branches with `acc.is_empty()` guard at lines 467–529; HexModal handler at lines 811–855; CR-01 fix (`.min()` clamp) at lines 826–827 |
| `hp41-cli/src/keys.rs` | `keycode_to_hp41_code()` function | ✓ VERIFIED | `pub fn keycode_to_hp41_code(code: crossterm::event::KeyCode) -> u8` at line 190; 35 mapped key codes; `_ => 0` catch-all at line 245 |
| `hp41-cli/src/ui.rs` | `PendingInput::HexModal(acc)` arm in `pending_prompt()` | ✓ VERIFIED | Arm at line 265; renders `"HEX: __"` (empty) or `format!("HEX: {}_", acc)` (one digit) |
| `hp41-cli/src/help_data.rs` | `=== Synthetic Programming ===` category with 7 entries | ✓ VERIFIED | Category header at line 267; 7 entries (X nn, S M/N/O, R M/N/O) at lines 269–299; test renamed to `test_all_fifteen_categories_present` |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `handle_key()` after release filter | `self.state.last_key_code` | `self.state.last_key_code = keys::keycode_to_hp41_code(key.code)` | ✓ WIRED | Line 167 in app.rs — placed before Ctrl+C, modal dispatch, and `key_to_op()` |
| 'X' interceptor in `handle_key()` | `PendingInput::HexModal(String::new())` | `self.pending_input = Some(PendingInput::HexModal(String::new()))` | ✓ WIRED | Lines 235–242; gated on `self.state.prgm_mode` |
| `StoRegister` arm | `Op::StoM/StoN/StoO` | `self.call_dispatch(Op::StoM)` with `acc.is_empty()` guard | ✓ WIRED | Lines 470–489; M/N/O checked before arithmetic intercepts |
| `RclRegister` arm | `Op::RclM/RclN/RclO` | `self.call_dispatch(Op::RclM)` with `acc.is_empty()` guard | ✓ WIRED | Lines 514–530 |
| `HexModal` arm | `synthetic_byte_to_op(byte)` | `match synthetic_byte_to_op(byte) { Some(_) => insert, None => INVALID }` | ✓ WIRED | Lines 822–838; validation before `program.insert()` |
| `HexModal` Some branch | `state.program.insert(insert_pos, ...)` + `state.pc += 1` | CR-01 fix: `insert_pos = self.state.pc.min(self.state.program.len())` | ✓ WIRED | Lines 826–831; panic guard for ISG/DSE skip-past-end case |
| `dispatch()` `Op::GetKey` arm | `registers::op_getkey(state)` | `Op::GetKey => op_getkey(state)` | ✓ WIRED | ops/mod.rs line 413 |
| `execute_op()` `Op::GetKey` arm | `super::registers::op_getkey(state)` | `Op::GetKey => super::registers::op_getkey(state)` | ✓ WIRED | program.rs line 327 — before catch-all |
| `dispatch()` `Op::SyntheticByte(b)` arm | `synthetic_byte_to_op(b)` lookup → recursive `dispatch(state, op)` | `if let Some(op) = synthetic_byte_to_op(b) { dispatch(state, op) }` | ✓ WIRED | ops/mod.rs lines 424–432; never returns `Some(Op::SyntheticByte(_))` |
| `execute_op()` `Op::SyntheticByte(b)` arm | `super::synthetic_byte_to_op(b)` → recursive `execute_op(state, op)` | Same pattern as dispatch | ✓ WIRED | program.rs lines 338–345 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `op_getkey()` in registers.rs | `state.last_key_code` | Set by `handle_key()` via `keycode_to_hp41_code(key.code)` on every press | Yes — real key code from hardware event | ✓ FLOWING |
| `op_sto_m()` / `op_rcl_m()` | `state.reg_m` | `reg_m = state.stack.x.clone()` / `let val = state.reg_m.clone()` | Yes — real stack value | ✓ FLOWING |
| `HexModal` handler | `state.program` | `state.program.insert(insert_pos, Op::SyntheticByte(byte))` after `synthetic_byte_to_op(byte)` check | Yes — user-supplied hex byte validated before insertion | ✓ FLOWING |
| `synthetic_byte_to_op()` | Return value | Static lookup table mapping u8 → `Option<Op>` | Yes — 23 real Op mappings; `None` for all unlisted bytes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| 21 synthetic_tests pass | `cargo test -p hp41-core --test synthetic_tests` | 21 passed | ✓ PASS |
| 95 hp41-cli tests pass | `cargo test -p hp41-cli` | 95 passed | ✓ PASS |
| `synthetic_byte_to_op(0xCF)` returns `Some(Op::Null)` | Asserted in `test_synthetic_byte_to_op_includes_null` | PASS | ✓ PASS |
| `synthetic_byte_to_op(0x00)` returns `None` | Asserted in `test_synthetic_byte_to_op_rejects_unknown` | PASS | ✓ PASS |
| `SyntheticByte(0x00)` returns error | Asserted in `test_synthetic_byte_unmapped_returns_error` | PASS | ✓ PASS |
| hp41-core has no hp41-cli dependency | `grep -c "hp41-cli" hp41-core/Cargo.toml` | 0 matches | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| SYNT-01 | 12-00, 12-01, 12-02 | GETKEY pushes last key code to X | ✓ SATISFIED | `op_getkey()` reads `state.last_key_code`; 4 tests in synthetic_tests.rs; `last_key_code` updated in `handle_key()` via `keycode_to_hp41_code()` |
| SYNT-02 | 12-00, 12-01 | NULL is a no-op with Neutral lift effect | ✓ SATISFIED | `Op::Null` arm calls only `apply_lift_effect(state, LiftEffect::Neutral)`; 3 tests pass; no CLI binding needed (reachable via HexModal 0xCF) |
| SYNT-03 | 12-00, 12-01, 12-02 | Hidden registers M/N/O accessible in programs; JSON round-trip | ✓ SATISFIED | 7 tests in synthetic_tests.rs; `#[serde(default)]` on reg_m/n/o; M/N/O dispatch in StoRegister/RclRegister arms |
| SYNT-04 | 12-00, 12-01, 12-02 | Hex-byte insertion modal with curated safe subset | ✓ SATISFIED | `PendingInput::HexModal`; `synthetic_byte_to_op()` with 23-entry table; program.insert with CR-01 clamp; INVALID rejection; 5 app.rs inline tests |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `hp41-core/src/ops/mod.rs` | 503, 510, 517, 568, 579 | `.unwrap()` calls | ℹ️ Info | These are all inside `#[cfg(test)]` test modules (the `flush_eex_tests` and `op_serde_tests` modules at line 487+) which carry `#[allow(clippy::unwrap_used)]`. Not a violation. |

No blockers found. No stubs. No TODO/FIXME/placeholder comments in Phase 12 production code.

**CR-01 fix confirmed:** The code review finding about `Vec::insert` panic when `state.pc > program.len()` (ISG/DSE skip-past-end) was fixed with `insert_pos = self.state.pc.min(self.state.program.len())` at app.rs lines 826–827. A dedicated test `test_hex_modal_insert_at_pc_past_end` exercises this edge case.

### Human Verification Required

#### 1. GETKEY Key Code Tracking End-to-End

**Test:** Run the TUI (`cargo run -p hp41-cli`). Enter PRGM mode (press P twice to toggle). Record a program: LBL A, GETKEY, RTN. Exit PRGM mode. Press '5'. Run the program (XEQ A). Check X register.
**Expected:** X shows 62 (HP-41 code for '5' key = row 6, col 2)
**Why human:** The `handle_key()` → `last_key_code` → `op_getkey()` → X data path spans CLI ↔ core and requires the live event loop to exercise

#### 2. HexModal Valid Byte Insertion (PRGM mode)

**Test:** Enter PRGM mode. Press Shift+X (uppercase X). The status bar should show `HEX: __`. Type 'C' then 'F'. Check that the program listing shows `SYN CF` and the display is normal.
**Expected:** `SYN CF` appears in program listing; pc advances by 1; no error message
**Why human:** Program listing display rendering and the full modal flow require TUI interaction

#### 3. HexModal Invalid Byte Rejection

**Test:** In PRGM mode, press Shift+X, then type '0' '0'.
**Expected:** The HP-41 display area shows `INVALID`; program listing is unchanged
**Why human:** User-visible error message (`app.message`) rendering in the TUI display area cannot be verified without running the binary

#### 4. HexModal PRGM-mode Gate

**Test:** In normal (non-PRGM) mode, press Shift+X (uppercase X).
**Expected:** Nothing happens — no modal opens, no message, no side effects. Calculator continues normally.
**Why human:** Mode-gate behavior needs keyboard interaction to confirm the key is consumed silently

#### 5. STO M / RCL M via Keyboard Modal

**Test:** Push 42 to X (type 42 ENTER). Press 'S' (opens STO modal). Status bar should show `STO [__]`. Press 'M'. Confirm reg_m now holds 42 (verifiable by pressing 'R' then 'M' to recall).
**Expected:** STO M stores 42 in reg_m; RCL M recalls it; `acc.is_empty()` guard fires — first char 'M' dispatches StoM immediately
**Why human:** StoRegister modal flow with M/N/O first-char dispatch requires keyboard interaction

#### 6. HexModal Esc Cancellation

**Test:** In PRGM mode, press Shift+X. Type one hex digit (e.g. 'C'). Status should show `HEX: C_`. Press Esc.
**Expected:** Modal closes; program unchanged; no `INVALID` message; display returns to normal
**Why human:** Cancellation mid-accumulation requires TUI interaction

### Gaps Summary

No gaps found. All four SYNT requirements have complete implementation evidence:

- SYNT-01 (GETKEY): `op_getkey()` reads `state.last_key_code`, all 4 tests pass, `keycode_to_hp41_code()` provides the HP-41 row×10+col mapping
- SYNT-02 (NULL): `Op::Null` dispatch and execute_op arms apply only `LiftEffect::Neutral`, all 3 tests pass
- SYNT-03 (Hidden Registers M/N/O): All 7 register functions substantive, all 7 tests pass, `#[serde(default)]` backward compatibility confirmed
- SYNT-04 (Hex Modal): HexModal complete with validation before insertion, CR-01 panic guard applied, 5 inline tests pass

The 6 human verification items cover user-visible TUI behavior and modal flow — things that cannot be tested programmatically without running the TUI binary.

---

_Verified: 2026-05-09T07:45:00Z_
_Verifier: Claude (gsd-verifier)_
