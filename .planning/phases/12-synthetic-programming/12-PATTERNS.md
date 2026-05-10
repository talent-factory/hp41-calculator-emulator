# Phase 12: Synthetic Programming - Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 10
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/state.rs` | model | CRUD | `hp41-core/src/state.rs` (existing — same file, add fields) | exact |
| `hp41-core/src/ops/mod.rs` | service | request-response | `hp41-core/src/ops/mod.rs` (existing — same file, add variants+arms) | exact |
| `hp41-core/src/ops/program.rs` | service | request-response | `hp41-core/src/ops/program.rs` (existing — same file, add execute_op arms) | exact |
| `hp41-core/src/ops/registers.rs` | service | CRUD | `hp41-core/src/ops/registers.rs` (existing — same file, mirror op_sto/op_rcl) | exact |
| `hp41-cli/src/app.rs` | controller | request-response | `hp41-cli/src/app.rs` (existing — add PendingInput variant + handle_key arms) | exact |
| `hp41-cli/src/keys.rs` | utility | request-response | `hp41-cli/src/keys.rs` (existing — add keycode_to_hp41_code function) | exact |
| `hp41-cli/src/ui.rs` | component | request-response | `hp41-cli/src/ui.rs` (existing — add HexModal arm in pending_prompt) | exact |
| `hp41-cli/src/prgm_display.rs` | utility | transform | `hp41-cli/src/prgm_display.rs` (existing — add op_display_name arms) | exact |
| `hp41-cli/src/help_data.rs` | config | — | `hp41-cli/src/help_data.rs` (existing — add Synthetic category) | exact |
| `hp41-core/tests/synthetic_tests.rs` | test | CRUD | `hp41-core/tests/register_tests.rs` + `print_tests.rs` | role-match |

---

## Pattern Assignments

### `hp41-core/src/state.rs` (model, CRUD — add 4 fields)

**Analog:** `hp41-core/src/state.rs` lines 89–116 (print_buffer serde(default) + new() init)

**Field declaration pattern** (lines 89–96 — `#[serde(default)]` on new fields):
```rust
// ── Phase 11: Print emulation ─────────────────────────────────────────────
/// Buffer of formatted print lines from PRX/PRA/PRSTK.
/// #[serde(default, skip)] — default enables backward-compat deserialization of v1.0
/// save files that lack this field; skip prevents serialization of transient state.
#[serde(default, skip)]
pub print_buffer: Vec<String>,
```

**Copy this pattern for Phase 12 fields** — but use `#[serde(default)]` only (no `skip`), since `last_key_code` and `reg_m/n/o` ARE persistent:
```rust
// ── Phase 12: Synthetic Programming ──────────────────────────────────────
/// Last HP-41 row-column key code pressed (row×10+col). 0 = none since startup.
/// Updated by hp41-cli handle_key() on every Press event. Default: 0.
#[serde(default)]
pub last_key_code: u8,

/// Hidden register M — accessible via STO M / RCL M in programs.
#[serde(default)]
pub reg_m: HpNum,

/// Hidden register N — accessible via STO N / RCL N in programs.
#[serde(default)]
pub reg_n: HpNum,

/// Hidden register O — accessible via STO O / RCL O in programs.
#[serde(default)]
pub reg_o: HpNum,
```

**CalcState::new() initialization pattern** (lines 99–117):
```rust
pub fn new() -> Self {
    CalcState {
        stack: Stack::new(),
        regs: vec![HpNum::zero(); 100],
        // ...existing fields...
        print_buffer: Vec::new(),
    }
}
```
Add to `new()`:
```rust
last_key_code: 0,
reg_m: HpNum::zero(),
reg_n: HpNum::zero(),
reg_o: HpNum::zero(),
```

**HpNum Default confirmation** (`hp41-core/src/num.rs` lines 237–239): `impl Default for HpNum` exists — returns `HpNum::zero()`. No additional work needed for `#[serde(default)]` on `HpNum` fields.

---

### `hp41-core/src/ops/mod.rs` (service, request-response — add 9 Op variants + dispatch arms)

**Analog:** `hp41-core/src/ops/mod.rs` lines 76–217 (Op enum), lines 265–388 (dispatch)

**Op enum comment/doc-comment style** (lines 210–216 — Phase 11 block at end of enum):
```rust
// ── Print operations (Phase 11) ─────────────────────────────────────────────────
/// PRX — print X register in current display format, right-aligned to 24 chars. LiftEffect: Neutral.
PRX,
/// PRA — print ALPHA register, left-aligned to 24 chars. LiftEffect: Neutral.
PRA,
/// PRSTK — print full stack T/Z/Y/X/LASTX/ALPHA, 6 lines, 24 chars each. LiftEffect: Neutral.
PRSTK,
```

**Pattern to copy for Phase 12 variants** — append after `PRSTK`:
```rust
// ── Synthetic Programming (Phase 12) ────────────────────────────────────────────
/// GETKEY — push last key code (HP-41 row×10+col) to X. LiftEffect: Enable.
GetKey,
/// NULL — true no-op; does not modify any state. LiftEffect: Neutral.
Null,
/// STO M — store X into hidden register M. LiftEffect: Neutral.
StoM,
/// STO N — store X into hidden register N. LiftEffect: Neutral.
StoN,
/// STO O — store X into hidden register O. LiftEffect: Neutral.
StoO,
/// RCL M — recall hidden register M into X. LiftEffect: Enable.
RclM,
/// RCL N — recall hidden register N into X. LiftEffect: Enable.
RclN,
/// RCL O — recall hidden register O into X. LiftEffect: Enable.
RclO,
/// SyntheticByte(u8) — synthetic op inserted via hex modal. At execution time,
/// dispatches to the corresponding Op via synthetic_byte_to_op() lookup. LiftEffect: varies.
SyntheticByte(u8),
```

**dispatch() arm pattern** (lines 384–388 — Phase 11 block at end of match):
```rust
// ── Phase 11: Print operations ───────────────────────────────────────────────
Op::PRX => print::op_prx(state),
Op::PRA => print::op_pra(state),
Op::PRSTK => print::op_prstk(state),
```

**Inline simple op pattern** (lines 348–352 — UserMode inline arm):
```rust
Op::UserMode => {
    state.user_mode = !state.user_mode;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Pattern to copy for Phase 12 dispatch arms** — append after PRSTK arm:
```rust
// ── Phase 12: Synthetic Programming ─────────────────────────────────────────────
Op::GetKey => registers::op_getkey(state),
Op::Null => {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
Op::StoM => registers::op_sto_m(state),
Op::StoN => registers::op_sto_n(state),
Op::StoO => registers::op_sto_o(state),
Op::RclM => registers::op_rcl_m(state),
Op::RclN => registers::op_rcl_n(state),
Op::RclO => registers::op_rcl_o(state),
Op::SyntheticByte(b) => {
    if let Some(op) = synthetic_byte_to_op(b) {
        // Recursive dispatch — safe because synthetic_byte_to_op never returns
        // Some(Op::SyntheticByte(_)), so recursion depth is exactly 1.
        dispatch(state, op)
    } else {
        Err(HpError::InvalidOp)
    }
}
```

**Also add** the `use registers::{..., op_sto_m, op_sto_n, op_sto_o, op_rcl_m, op_rcl_n, op_rcl_o, op_getkey}` import at line 25 (existing registers imports line).

---

### `hp41-core/src/ops/program.rs` (service, request-response — add 9 execute_op arms)

**Analog:** `hp41-core/src/ops/program.rs` lines 215–335 (execute_op function)

**Critical constraint** (lines 217–220): `execute_op` MUST NOT call `flush_entry_buf` and MUST NOT check `prgm_mode`. It mirrors `dispatch()` but is purely for in-program execution.

**Phase 11 block at end of execute_op** (lines 322–335):
```rust
// ── Phase 11: Print operations ───────────────────────────────────────────────────
Op::PRX => super::print::op_prx(state),
Op::PRA => super::print::op_pra(state),
Op::PRSTK => super::print::op_prstk(state),
// Programming ops handled by run_loop directly — must not reach here
Op::Lbl(_)
| Op::Gto(_)
| Op::Xeq(_)
| Op::Rtn
| Op::PrgmMode
| Op::Test(_)
| Op::Isg(_)
| Op::Dse(_) => Err(HpError::InvalidOp),
```

**Pattern to copy for Phase 12** — insert BEFORE the `Op::Lbl(_) | ...` programming-ops catch-all block:
```rust
// ── Phase 12: Synthetic Programming ─────────────────────────────────────────────────
Op::GetKey => super::registers::op_getkey(state),
Op::Null => {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
Op::StoM => super::registers::op_sto_m(state),
Op::StoN => super::registers::op_sto_n(state),
Op::StoO => super::registers::op_sto_o(state),
Op::RclM => super::registers::op_rcl_m(state),
Op::RclN => super::registers::op_rcl_n(state),
Op::RclO => super::registers::op_rcl_o(state),
Op::SyntheticByte(b) => {
    if let Some(op) = super::synthetic_byte_to_op(b) {
        execute_op(state, op)
    } else {
        Err(HpError::InvalidOp)
    }
}
```

**Also add** `Op::GetKey | Op::Null | Op::StoM | Op::StoN | Op::StoO | Op::RclM | Op::RclN | Op::RclO | Op::SyntheticByte(_)` MUST NOT appear in the programming-ops catch-all.

---

### `hp41-core/src/ops/registers.rs` (service, CRUD — add 7 new functions)

**Analog:** `hp41-core/src/ops/registers.rs` lines 14–36 (op_sto + op_rcl — exact pattern)

**op_sto pattern** (lines 14–21):
```rust
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    state.regs[reg as usize] = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**op_rcl pattern** (lines 25–36):
```rust
pub fn op_rcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 {
        return Err(HpError::InvalidOp);
    }
    let val = state.regs[reg as usize].clone();
    // Force lift_enabled = true so enter_number performs the stack lift.
    // This matches HP-41 hardware: RCL always lifts regardless of prior state.
    state.stack.lift_enabled = true;
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**Copy these patterns directly for hidden register ops** (no bounds check needed — named field, not indexed):
```rust
pub fn op_sto_m(state: &mut CalcState) -> Result<(), HpError> {
    state.reg_m = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_rcl_m(state: &mut CalcState) -> Result<(), HpError> {
    let val = state.reg_m.clone();
    state.stack.lift_enabled = true; // RCL always lifts — matches op_rcl line 32
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
// Repeat identically for op_sto_n/op_rcl_n and op_sto_o/op_rcl_o
```

**op_getkey — new function for GETKEY** (modeled after op_rcl, reads CalcState field instead of regs[]):
```rust
pub fn op_getkey(state: &mut CalcState) -> Result<(), HpError> {
    let code = HpNum::from(state.last_key_code as i32);
    state.stack.lift_enabled = true; // GETKEY produces a new value — always lifts
    enter_number(state, code);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**Also add a `synthetic_byte_to_op()` function in `mod.rs`** (or as a public function in registers.rs / its own module). The function is referenced from both `dispatch()` and `execute_op()`. Pattern: `pub fn synthetic_byte_to_op(byte: u8) -> Option<Op>` returning a conservative match of ~10–15 known byte codes. See RESEARCH.md Pattern 5 for the initial table.

---

### `hp41-cli/src/app.rs` (controller, request-response — 3 changes)

**Analog:** `hp41-cli/src/app.rs` lines 23–36 (PendingInput enum), lines 151–434 (handle_key), lines 438–744 (handle_pending_input)

#### Change 1: PendingInput enum — add HexModal variant

**Pattern** (lines 23–36 — existing PendingInput enum):
```rust
#[derive(Debug, Clone)]
pub enum PendingInput {
    StoRegister(String),
    RclRegister(String),
    StoAdd(String),
    StoSub(String),
    StoMul(String),
    StoDiv(String),
    AssignKey,
    AssignLabel(char, String),
    ConfirmLoad(usize),
    FmtDigits(hp41_core::DisplayMode),
    PrintModal,
}
```
Add after `PrintModal`:
```rust
HexModal(String), // accumulating 2-digit hex code for synthetic byte insertion (PRGM mode only)
```

#### Change 2: handle_key — two additions

**Addition A: last_key_code update** — insert AFTER the release filter (line 154) and BEFORE the Ctrl+C check (line 159):
```rust
// [Phase 12 D-01] Update last_key_code on every Press event — before any modal/dispatch.
// Placement: after release filter, before Ctrl+C and all modal logic.
self.state.last_key_code = keys::keycode_to_hp41_code(key.code);
```

**Addition B: 'X' interceptor for HexModal** — insert after the 'P' interceptor (lines 214–219) and before Ctrl+A (lines 221–225):
```rust
// 'X' (Shift+x) opens hex-byte insertion modal in PRGM mode (Phase 12 D-14).
// 'x' (lowercase) is Op::XySwap — intercepted separately in key_to_op().
// Gate on prgm_mode: hex insertion only makes sense in program recording context (D-18).
if key.code == KeyCode::Char('X') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    if self.state.prgm_mode {
        self.pending_input = Some(PendingInput::HexModal(String::new()));
        self.message = None;
    }
    return;
}
```

#### Change 3: handle_pending_input — two arm extensions

**Extension A: M/N/O branches in StoRegister arm** — insert BEFORE the existing arithmetic key intercepts (line 443–463). The `acc.is_empty()` guard is critical:
```rust
Some(PendingInput::StoRegister(ref acc)) => {
    // [Phase 12 D-08] M/N/O dispatch — only valid as FIRST character (acc.is_empty() guard).
    // If user has started typing a digit, ignore M/N/O (no valid register "0M").
    if acc.is_empty() {
        match key.code {
            KeyCode::Char('M') | KeyCode::Char('m') => {
                self.call_dispatch(Op::StoM);
                self.pending_input = None;
                return;
            }
            KeyCode::Char('N') | KeyCode::Char('n') => {
                self.call_dispatch(Op::StoN);
                self.pending_input = None;
                return;
            }
            KeyCode::Char('O') | KeyCode::Char('o') => {
                self.call_dispatch(Op::StoO);
                self.pending_input = None;
                return;
            }
            _ => {} // fall through to arithmetic key intercepts below
        }
    }
    // Existing arithmetic key intercepts (+ - * /) and handle_reg_modal...
```

Same pattern for `RclRegister` arm (dispatch `RclM/RclN/RclO`), inserted before `handle_reg_modal` call at line 466.

**Extension B: new HexModal arm** — insert after PrintModal arm (lines 720–742) and before `None => {}`:
```rust
Some(PendingInput::HexModal(ref acc)) => {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_hexdigit() => {
            let hex_char = c.to_ascii_uppercase(); // normalize to uppercase for display
            let mut new_acc = acc.clone();
            new_acc.push(hex_char);
            if new_acc.len() == 2 {
                // Two hex chars always parse as u8 — unwrap safe here.
                let byte = u8::from_str_radix(&new_acc, 16)
                    .expect("two uppercase hex chars always parse as u8");
                match synthetic_byte_to_op(byte) {
                    Some(_) => {
                        // Insert Op::SyntheticByte at current PC, advance PC past it (D-16).
                        self.state.program.insert(self.state.pc, Op::SyntheticByte(byte));
                        self.state.pc += 1;
                        self.message = None;
                    }
                    None => {
                        self.message = Some("INVALID".to_string()); // D-13
                    }
                }
                self.pending_input = None; // modal always closes (D-13)
            } else {
                self.pending_input = Some(PendingInput::HexModal(new_acc));
            }
        }
        KeyCode::Esc => {
            self.pending_input = None; // cancel with no side effects
        }
        _ => {
            // Non-hex key: restore modal silently (existing convention — see PrintModal arm)
            self.pending_input = Some(PendingInput::HexModal(acc.clone()));
        }
    }
}
```

Add `use hp41_core::ops::synthetic_byte_to_op;` to the imports at the top of app.rs (line 15 block).

---

### `hp41-cli/src/keys.rs` (utility, request-response — add new function)

**Analog:** `hp41-cli/src/keys.rs` lines 18–86 (key_to_op function structure)

**key_to_op pattern** (lines 18–24 — function signature + match structure):
```rust
pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op> {
    match key.code {
        KeyCode::Enter => Some(Op::Enter),
        // ...
        _ => None,
    }
}
```

**New function** — add after key_to_op, same file:
```rust
/// Map a crossterm KeyCode to the HP-41 hardware key code (row×10 + col, 1-indexed).
/// Returns 0 for keys with no HP-41 hardware equivalent (function keys, Ctrl combos, etc.).
/// Called from app.handle_key() to update CalcState.last_key_code on every Press event (D-01).
///
/// HP-41C keyboard layout: 8 rows × 5 columns.
/// Key code = row × 10 + col (rows 1-8 top-to-bottom, cols 1-5 left-to-right).
pub fn keycode_to_hp41_code(code: KeyCode) -> u8 {
    match code {
        // Row 8: 0, ., EEX, ENTER (bottom row)
        KeyCode::Char('0') => 81,
        KeyCode::Char('.') => 82,
        KeyCode::Char('e') => 83, // EEX
        KeyCode::Enter => 84,     // ENTER (also row 4 col 4 on hardware — map to row 8 here)
        // Row 7: 1, 2, 3, +
        KeyCode::Char('1') => 71,
        KeyCode::Char('2') => 72,
        KeyCode::Char('3') => 73,
        KeyCode::Char('+') => 74,
        // Row 6: 4, 5, 6, -
        KeyCode::Char('4') => 61,
        KeyCode::Char('5') => 62,
        KeyCode::Char('6') => 63,
        KeyCode::Char('-') => 64,
        // Row 5: 7, 8, 9, ×
        KeyCode::Char('7') => 51,
        KeyCode::Char('8') => 52,
        KeyCode::Char('9') => 53,
        KeyCode::Char('*') => 54,
        // Row 4: USER, f-shift, g-shift, ÷
        KeyCode::Char('u') | KeyCode::Char('U') => 41, // USER
        KeyCode::Char('f') | KeyCode::Char('F') => 42, // f-key
        KeyCode::Char('g') | KeyCode::Char('G') => 43, // g-key
        KeyCode::Char('/') => 45,                       // ÷
        // Row 3: R/S, SST, GTO, COS, TAN
        KeyCode::F(5) => 31,                            // R/S
        KeyCode::F(7) | KeyCode::F(8) => 32,            // SST / BST
        KeyCode::Char('C') => 34,                       // COS
        KeyCode::Char('T') => 35,                       // TAN
        // Row 2: XEQ, STO, RCL, R↓, SIN
        KeyCode::Char('X') => 21, // XEQ (also HexModal opener — key code is still 21)
        KeyCode::Char('S') => 22, // STO
        KeyCode::Char('R') => 23, // RCL
        KeyCode::Char('r') => 24, // R↓
        KeyCode::Char('q') => 25, // SIN (Phase 8 reassignment)
        // Row 1: Σ+, 1/x, √x, LOG, LN
        KeyCode::Char('z') => 11, // Σ+
        KeyCode::Char('I') => 12, // 1/x
        KeyCode::Char('s') => 13, // √x
        KeyCode::Char('L') => 15, // LN (G=LOG conflicts with row 4 col 3; use L for LN)
        // Unmapped: all other keys (modifiers, F-keys not listed, Esc, etc.)
        _ => 0,
    }
}
```

**KEY_REF_TABLE addition** — append to the `&[(&str, &str)]` slice (lines 90–168). The `'X'` modal entry:
```rust
("X nn", "HEX modal", "Insert synthetic hex byte at current PC (PRGM mode only)"),
```

---

### `hp41-cli/src/ui.rs` (component, request-response — add HexModal arm in pending_prompt)

**Analog:** `hp41-cli/src/ui.rs` lines 238–265 (pending_prompt function — exhaustive match)

**PrintModal arm pattern** (line 264 — single-line simple arm):
```rust
PendingInput::PrintModal => "PRNT: _".to_string(),
```

**StoRegister arm pattern** (line 241 — format with {:_<2} placeholder):
```rust
PendingInput::StoRegister(acc) => format!("STO [{:_<2}]", acc),
```

**HexModal arm to add** — insert after PrintModal arm (line 264), before closing brace of match:
```rust
PendingInput::HexModal(acc) => {
    if acc.is_empty() {
        "HEX: __".to_string()
    } else {
        format!("HEX: {}_", acc)
    }
}
```

Note: `pending_prompt` is a private function (no `pub`). The `match pending {` is exhaustive — adding `HexModal` to `PendingInput` without adding this arm will cause a compiler error. Both changes must land in the same commit.

---

### `hp41-cli/src/prgm_display.rs` (utility, transform — add 9 arms to op_display_name)

**Analog:** `hp41-cli/src/prgm_display.rs` lines 27–129 (op_display_name match — all 35 existing variants)

**Phase 11 arms pattern** (lines 124–128 — last group before closing brace):
```rust
// Phase 11: Print operations
Op::PRX => "PRX".to_string(),
Op::PRA => "PRA".to_string(),
Op::PRSTK => "PRSTK".to_string(),
```

**Tuple variant with format pattern** (lines 64–65):
```rust
Op::FmtFix(n) => format!("FIX {n}"),
```

**Add after PRSTK arms**:
```rust
// Phase 12: Synthetic Programming
Op::GetKey => "GETKEY".to_string(),
Op::Null => "NULL".to_string(),
Op::StoM => "STO M".to_string(),
Op::StoN => "STO N".to_string(),
Op::StoO => "STO O".to_string(),
Op::RclM => "RCL M".to_string(),
Op::RclN => "RCL N".to_string(),
Op::RclO => "RCL O".to_string(),
Op::SyntheticByte(b) => format!("SYN {:02X}", b), // uppercase hex, zero-padded (D-11)
```

---

### `hp41-cli/src/help_data.rs` (config — add Synthetic category)

**Analog:** `hp41-cli/src/help_data.rs` lines 249–265 (Print category — most recent addition)

**Category header pattern** (line 250):
```rust
// ── Print ─────────────────────────────────────────────────────────────────
("", "", "=== Print ==="),
(
    "P X",
    "PRX",
    "Print X register to console (right-aligned, 24 chars)",
),
```

**Category to add** — insert after Print category and before Help category (line 267):
```rust
// ── Synthetic Programming ─────────────────────────────────────────────────
("", "", "=== Synthetic Programming ==="),
(
    "X nn",
    "HEX",
    "Insert synthetic hex byte at current PC (PRGM mode) — press X then 2 hex digits",
),
(
    "S M",
    "STO M",
    "Store X to hidden register M — press S then M",
),
(
    "S N",
    "STO N",
    "Store X to hidden register N — press S then N",
),
(
    "S O",
    "STO O",
    "Store X to hidden register O — press S then O",
),
(
    "R M",
    "RCL M",
    "Recall hidden register M into X — press R then M",
),
(
    "R N",
    "RCL N",
    "Recall hidden register N into X — press R then N",
),
(
    "R O",
    "RCL O",
    "Recall hidden register O into X — press R then O",
),
```

**Test impact:** `test_help_data_has_minimum_entries` checks `>= 80`. Adding 8 entries + 1 header = 9 new rows. Will still pass. `test_all_fourteen_categories_present` checks exact category names — add `"=== Synthetic Programming ==="` to that test's array (it becomes 15 categories).

---

### `hp41-core/tests/synthetic_tests.rs` (test, CRUD — new file)

**Analog:** `hp41-core/tests/register_tests.rs` lines 1–9 (imports + helper function pattern) + `hp41-core/tests/print_tests.rs` lines 1–6 (allow directive pattern)

**File header pattern** (register_tests.rs lines 1–9):
```rust
//! Integration tests for REGS-01: storage registers R00–R99, STO/RCL, STO-arith.

use hp41_core::ops::{dispatch, Op, StoArithKind};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
```

**allow directive** (print_tests.rs line 5):
```rust
#![allow(clippy::unwrap_used)]
```

**Test function structure** (register_tests.rs lines 13–27):
```rust
#[test]
fn test_sto_rcl_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 42);
    dispatch(&mut s, Op::StoReg(5)).unwrap();
    s.stack.x = HpNum::zero();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclReg(5)).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(42),
        "RCL must restore STO'd value"
    );
}
```

**Scaffold structure for `synthetic_tests.rs`** — Wave 0 tests compile but fail (RED) until Wave 1 ships:
```rust
//! Integration tests for Phase 12 Synthetic Programming.
//! SYNT-01 (GETKEY), SYNT-02 (NULL), SYNT-03 (hidden regs M/N/O), SYNT-04 (SyntheticByte).
//!
//! Wave 0: all tests are RED (compile but fail) until Wave 1 ships Op variants.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;

fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

// ── SYNT-01: GETKEY ───────────────────────────────────────────────────────────

#[test]
fn test_getkey_zero_when_no_key_pressed() {
    let mut s = CalcState::new();
    // last_key_code = 0 (default) → GetKey must push 0 to X
    dispatch(&mut s, Op::GetKey).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(0));
}

#[test]
fn test_getkey_pushes_last_key_code() {
    let mut s = CalcState::new();
    s.last_key_code = 62; // row 6 col 2 = '5' key
    dispatch(&mut s, Op::GetKey).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(62));
}

// ── SYNT-02: NULL ─────────────────────────────────────────────────────────────

#[test]
fn test_null_does_not_modify_stack() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(Decimal::from(42));
    s.stack.y = HpNum::from(Decimal::from(7));
    let lift_before = s.stack.lift_enabled;
    dispatch(&mut s, Op::Null).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(42));
    assert_eq!(s.stack.y.inner(), Decimal::from(7));
    assert_eq!(s.stack.lift_enabled, lift_before, "NULL must not change lift flag");
}

// ── SYNT-03: Hidden registers ─────────────────────────────────────────────────

#[test]
fn test_sto_m_rcl_m_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 99);
    dispatch(&mut s, Op::StoM).unwrap();
    s.stack.x = HpNum::zero();
    s.stack.lift_enabled = false;
    dispatch(&mut s, Op::RclM).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(99));
}

#[test]
fn test_hidden_regs_serde_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 5);
    dispatch(&mut s, Op::StoM).unwrap();
    let json = serde_json::to_string(&s).unwrap();
    let s2: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(s2.reg_m.inner(), Decimal::from(5));
}

// ── SYNT-04: SyntheticByte ────────────────────────────────────────────────────

#[test]
fn test_synthetic_byte_serde_round_trip() {
    let op = Op::SyntheticByte(0x40);
    let json = serde_json::to_string(&op).unwrap();
    let op2: Op = serde_json::from_str(&json).unwrap();
    assert_eq!(op, op2);
}
```

Also add to `hp41-core/Cargo.toml` if not already present:
```toml
[dev-dependencies]
serde_json = "..."
```
(check existing Cargo.toml — likely already present given print_tests.rs uses serde patterns).

---

## Shared Patterns

### LiftEffect application
**Source:** `hp41-core/src/ops/registers.rs` lines 14–36 — `apply_lift_effect(state, LiftEffect::Neutral/Enable)`
**Apply to:** All 9 new Op dispatch/execute_op arms. Each arm ends with `apply_lift_effect(state, LiftEffect::Xxx)` matching the declared effect.

Pattern summary:
- STO ops (StoM/StoN/StoO): `apply_lift_effect(state, LiftEffect::Neutral)` — no stack change
- RCL ops (RclM/RclN/RclO): `state.stack.lift_enabled = true; enter_number(state, val); apply_lift_effect(state, LiftEffect::Enable)` — produces value
- GetKey: same as RCL pattern above (produces value)
- Null: `apply_lift_effect(state, LiftEffect::Neutral)` — empty body otherwise
- SyntheticByte: delegates entirely to the target Op; inherits that Op's lift behavior

### Modal interceptor ordering
**Source:** `hp41-cli/src/app.rs` lines 186–225 — the `if self.pending_input.is_some() { ... return; }` block MUST precede all modal-opening interceptors.
**Apply to:** 'X' interceptor for HexModal — add after 'P' interceptor (line 219), following the same structure as 'S', 'R', 'F', 'P' interceptors.

### PendingInput Esc/none convention
**Source:** `hp41-cli/src/app.rs` lines 720–742 (PrintModal arm):
```rust
KeyCode::Esc => {
    self.pending_input = None; // cancel with no side effects
}
_ => {
    // Silently ignore unrecognized keys — keep modal open (existing convention).
    self.pending_input = Some(PendingInput::PrintModal);
}
```
**Apply to:** HexModal arm — same Esc → close, unrecognized → restore modal behavior.

### #[serde(default)] backward compatibility
**Source:** `hp41-core/src/state.rs` lines 92–95 (print_buffer serde annotation)
**Apply to:** All four new CalcState fields. Without `#[serde(default)]`, loading v1.0 save files fails with JSON deserialization error.

### Error handling in core ops
**Source:** `hp41-core/src/ops/registers.rs` lines 14–21 (op_sto bounds check + early return)
**Apply to:** All new core functions — use `-> Result<(), HpError>` signature, `?`-propagation, no `.unwrap()` in non-test code (enforced by `#![deny(clippy::unwrap_used)]`).

### app.message for user feedback
**Source:** `hp41-cli/src/app.rs` — `self.message = Some("INVALID".to_string())` pattern used throughout handle_pending_input
**Apply to:** HexModal rejection branch (D-13) — sets `self.message = Some("INVALID".to_string())` before `self.pending_input = None`.

---

## No Analog Found

None. All 10 files have exact or role-match analogs in the existing codebase.

---

## Metadata

**Analog search scope:** `hp41-core/src/`, `hp41-core/tests/`, `hp41-cli/src/`
**Files scanned:** 16 source files read directly; 4 grep passes for function locations
**Pattern extraction date:** 2026-05-09

**Key assumption from RESEARCH.md (A3):** `HpNum` implements `Default` — VERIFIED at `hp41-core/src/num.rs` lines 237–239. No additional work required.

**Key assumption from RESEARCH.md (A2):** Exact NUT byte codes for `synthetic_byte_to_op()` — marked `[ASSUMED]` in RESEARCH.md. Implementor should define a conservative initial table (~10 entries) with a `// [ASSUMED]` comment, expandable in SYNT-05.
