# Phase 11: Print Emulation - Pattern Map

**Mapped:** 2026-05-08
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/ops/print.rs` | op-module | transform | `hp41-core/src/ops/registers.rs` | exact |
| `hp41-core/src/ops/mod.rs` | op-registry | CRUD | self (add variants + arms) | self-extension |
| `hp41-core/src/ops/program.rs` | interpreter | CRUD | self (add execute_op arms) | self-extension |
| `hp41-core/src/state.rs` | model | CRUD | self (add field) | self-extension |
| `hp41-cli/src/app.rs` | controller | event-driven | self (add modal + drain) | self-extension |
| `hp41-cli/src/main.rs` | config/entry | request-response | self (add CLI arg) | self-extension |
| `hp41-cli/src/ui.rs` | component | request-response | self (add pending_prompt arm) | self-extension |
| `hp41-cli/src/help_data.rs` | config | CRUD | self (add entries) | self-extension |
| `hp41-core/tests/print_tests.rs` | test | CRUD | `hp41-core/tests/register_tests.rs` | exact |

---

## Pattern Assignments

### `hp41-core/src/ops/print.rs` (op-module, transform)

**Analog:** `hp41-core/src/ops/registers.rs`

**Imports pattern** (registers.rs lines 7-11):
```rust
use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
```
For print.rs, additionally import:
```rust
use crate::format::format_hpnum;
```

**Module doc comment pattern** (registers.rs line 1-6):
```rust
//! Phase N <feature> operations: <op names>.
//!
//! <brief description of LiftEffect>
```

**Core op pattern — Neutral LiftEffect, no stack modification** (registers.rs lines 14-21):
```rust
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    // ... body ...
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Concrete PRX pattern** (derived from registers.rs structure, CONTEXT D-12):
```rust
pub fn op_prx(state: &mut CalcState) -> Result<(), HpError> {
    let line = format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode));
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Concrete PRA pattern** (CONTEXT D-12, D-13 — note: NOT format_alpha which truncates to 12):
```rust
pub fn op_pra(state: &mut CalcState) -> Result<(), HpError> {
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    let line = format!("{:<24}", alpha);
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Concrete PRSTK pattern** (CONTEXT D-13):
```rust
pub fn op_prstk(state: &mut CalcState) -> Result<(), HpError> {
    let mode = &state.display_mode;
    let lines = [
        format!("{:<7}{:>17}", "T:",     format_hpnum(&state.stack.t,     mode)),
        format!("{:<7}{:>17}", "Z:",     format_hpnum(&state.stack.z,     mode)),
        format!("{:<7}{:>17}", "Y:",     format_hpnum(&state.stack.y,     mode)),
        format!("{:<7}{:>17}", "X:",     format_hpnum(&state.stack.x,     mode)),
        format!("{:<7}{:>17}", "LASTX:", format_hpnum(&state.stack.lastx, mode)),
        {
            let alpha = state.alpha_reg.chars().take(17).collect::<String>();
            format!("{:<7}{:<17}", "ALPHA:", alpha)
        },
    ];
    for line in lines {
        state.print_buffer.push(line);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Test module pattern** (registers.rs lines 103-110):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;
    // ... test bodies ...
}
```

**Anti-pattern warning:** Do NOT call `format_alpha()` for PRA — it truncates to 12 chars, not 24. Use `state.alpha_reg.chars().take(24)` directly.

---

### `hp41-core/src/ops/mod.rs` (op-registry, self-extension)

**Analog:** self (existing file)

**Module declaration pattern** (mod.rs lines 9-16):
```rust
pub mod alpha;
pub mod arithmetic;
// ... existing mods ...
pub mod registers;
pub mod stack_ops;
pub mod stats;
```
Add after the existing list:
```rust
pub mod print;
```

**Op enum variant comment pattern** (mod.rs lines 123-135):
```rust
// ── Display mode (Phase 2) ───────────────────────────────────────
/// FIX n — fixed decimal display (n = 0–9). LiftEffect: Neutral.
FmtFix(u8),
```
Copy this exact style for new variants:
```rust
// ── Print operations (Phase 11) ─────────────────────────────────
/// PRX — print X register in current display format, right-aligned to 24 chars. LiftEffect: Neutral.
PRX,
/// PRA — print ALPHA register, left-aligned to 24 chars. LiftEffect: Neutral.
PRA,
/// PRSTK — print full stack T/Z/Y/X/LASTX/ALPHA, 6 lines, 24 chars each. LiftEffect: Neutral.
PRSTK,
```

**dispatch() match arm pattern** (mod.rs lines 363-376):
```rust
// ── Phase 6: Science & Engineering ───────────────────────────────────────
Op::SigmaPlus => stats::op_sigma_plus(state),
Op::SigmaMinus => stats::op_sigma_minus(state),
// ...
Op::HmsSub => hms::op_hms_sub(state),
```
Add a new phase-tagged section in the same style:
```rust
// ── Phase 11: Print operations ───────────────────────────────────────────
Op::PRX   => print::op_prx(state),
Op::PRA   => print::op_pra(state),
Op::PRSTK => print::op_prstk(state),
```

---

### `hp41-core/src/ops/program.rs` (interpreter, self-extension)

**Analog:** self (existing execute_op)

**execute_op match arm pattern** (program.rs lines 309-315):
```rust
// ── Phase 6: Science & Engineering ───────────────────────────────────────
Op::SigmaPlus => super::stats::op_sigma_plus(state),
Op::SigmaMinus => super::stats::op_sigma_minus(state),
// ...
Op::HmsSub => super::hms::op_hms_sub(state),
```
Add a matching section inside execute_op using the `super::` path:
```rust
// ── Phase 11: Print operations ───────────────────────────────────────────
Op::PRX   => super::print::op_prx(state),
Op::PRA   => super::print::op_pra(state),
Op::PRSTK => super::print::op_prstk(state),
```

**Critical constraint:** These three arms must be added to `execute_op()` (private, line ~221) AND to `dispatch()` in mod.rs (line ~272). Missing either causes PRX/PRA/PRSTK to silently fail in programs.

---

### `hp41-core/src/state.rs` (model, self-extension)

**Analog:** self (existing CalcState)

**New field with serde(default) pattern** (state.rs lines 82-88 — key_assignments is most recent):
```rust
/// USER mode active: when true, key_assignments are consulted before normal dispatch.
/// D-25: default false.
pub user_mode: bool,
/// User key assignments: maps key char → program label name.
/// BTreeMap for deterministic JSON serialization order (D-25, D-29).
#[serde(default)]
pub key_assignments: BTreeMap<char, String>,
```
Copy this pattern for print_buffer:
```rust
/// Buffer of formatted print lines from PRX/PRA/PRSTK.
/// Drained by hp41-cli after each dispatch. Never persisted across sessions.
/// #[serde(default)] preserves backward compatibility with v1.0 save files.
#[serde(default)]
pub print_buffer: Vec<String>,
```

**CalcState::new() initialization pattern** (state.rs lines 92-109):
```rust
CalcState {
    stack: Stack::new(),
    regs: vec![HpNum::zero(); 100],
    // ... existing fields ...
    user_mode: false,
    key_assignments: BTreeMap::new(),
}
```
Add after key_assignments:
```rust
print_buffer: Vec::new(),
```

---

### `hp41-cli/src/app.rs` (controller, self-extension — 4 change points)

**Analog:** self (existing App / handle_key / handle_pending_input / call_dispatch)

**Change point 1: PendingInput enum** (app.rs lines 22-34) — add new variant:
```rust
pub enum PendingInput {
    StoRegister(String),
    // ... existing variants ...
    FmtDigits(hp41_core::DisplayMode),
    // ADD:
    PrintModal,
}
```

**Change point 2: App struct** (app.rs lines 37-55) — add file writer field:
```rust
pub struct App {
    pub state: CalcState,
    pub message: Option<String>,
    pub exit: bool,
    pub last_save: Instant,
    pub state_path: PathBuf,
    pub pending_input: Option<PendingInput>,
    pub show_help: bool,
    pub help_table_state: RefCell<TableState>,
    pub show_programs: bool,
    pub programs_table_state: RefCell<TableState>,
    // ADD (Phase 11):
    /// Buffered file writer for --print-log. None = no file logging.
    pub print_log_writer: Option<std::io::BufWriter<std::fs::File>>,
}
```

**Change point 3: 'P' interceptor in handle_key()** (app.rs lines 162-185 — existing 'S'/'R'/'F' interceptors are the pattern):
```rust
// 'S' key pattern (app.rs line 163-166) — copy exactly for 'P':
if key.code == KeyCode::Char('S') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::StoRegister(String::new()));
    self.message = None;
    return;
}
// Add after 'F' interceptor (line ~179):
if key.code == KeyCode::Char('P') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::PrintModal);
    self.message = None;
    return;
}
```

**Change point 4: PrintModal arm in handle_pending_input()** — closest analog is FmtDigits arm (app.rs lines 641-673) which is also a simple single-key dispatch modal:
```rust
Some(PendingInput::FmtDigits(mode)) => {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            // dispatch op, clear pending
            self.call_dispatch(op);
            self.pending_input = None;
        }
        KeyCode::Esc => {
            self.pending_input = None;
        }
        _ => {
            // Restore modal silently for unrecognized keys
            self.pending_input = Some(PendingInput::FmtDigits(mode));
        }
    }
}
```
Copy this skeleton for PrintModal:
```rust
Some(PendingInput::PrintModal) => {
    match key.code {
        KeyCode::Char('x') | KeyCode::Char('X') => {
            self.call_dispatch_and_drain(Op::PRX);
            self.pending_input = None;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            self.call_dispatch_and_drain(Op::PRA);
            self.pending_input = None;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            self.call_dispatch_and_drain(Op::PRSTK);
            self.pending_input = None;
        }
        KeyCode::Esc => {
            self.pending_input = None;
        }
        _ => {
            // Silently ignore unrecognized keys — keep modal open
            self.pending_input = Some(PendingInput::PrintModal);
        }
    }
}
```

**call_dispatch pattern** (app.rs lines 771-776) — extended to drain print_buffer:
```rust
// Existing call_dispatch (lines 771-776):
fn call_dispatch(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => self.message = None,
        Err(e) => self.message = Some(format!("{e}")),
    }
}

// New call_dispatch_and_drain method (add after call_dispatch):
fn call_dispatch_and_drain(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => {
            let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
            if !lines.is_empty() {
                for line in &lines {
                    if let Some(ref mut writer) = self.print_log_writer {
                        let _ = writeln!(writer, "{}", line);
                        let _ = writer.flush();
                    }
                }
                if lines.len() > 1 {
                    self.message = Some(format!("PRSTK \u{2192} {} lines", lines.len()));
                } else {
                    self.message = Some(lines.into_iter().next().unwrap_or_default());
                }
            } else {
                self.message = None;
            }
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
}
```

**App::new() signature extension pattern** (app.rs lines 58-71):
```rust
// Existing:
pub fn new(state: CalcState, state_path: PathBuf) -> Self {
    App {
        state,
        message: None,
        // ...
    }
}
// Extended for print_log:
pub fn new(state: CalcState, state_path: PathBuf, print_log: Option<std::path::PathBuf>) -> Self {
    let print_log_writer = print_log.and_then(|path| {
        match std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => Some(std::io::BufWriter::new(file)),
            Err(e) => {
                // Error stored in initial message — caller sets app.message after new() returns
                eprintln!("Warning: failed to open print log {}: {e}", path.display());
                None
            }
        }
    });
    App {
        state,
        message: None,
        // ... existing fields ...
        print_log_writer,
    }
}
```

**new_for_test() update pattern** (app.rs lines 784-788):
```rust
// Existing:
pub fn new_for_test() -> Self {
    App::new(CalcState::new(), PathBuf::from("/tmp/hp41-cli-test-state.json"))
}
// Updated:
pub fn new_for_test() -> Self {
    App::new(CalcState::new(), PathBuf::from("/tmp/hp41-cli-test-state.json"), None)
}
```

---

### `hp41-cli/src/main.rs` (config/entry, self-extension)

**Analog:** self (existing Cli struct)

**Existing Cli field pattern** (main.rs lines 28-35):
```rust
/// Path to the state file (JSON). Loaded on startup, saved on exit and every 30s.
/// Default: ~/.hp41/autosave.json
#[arg(long, value_name = "FILE")]
state_file: Option<std::path::PathBuf>,
```
Copy for print_log:
```rust
/// Append all PRX/PRA/PRSTK output to this file (created if absent, appended if exists).
#[arg(long, value_name = "FILE")]
print_log: Option<std::path::PathBuf>,
```

**App::new() call pattern** (main.rs line 66):
```rust
// Existing:
let mut app = App::new(initial_state, state_path);
// Updated:
let mut app = App::new(initial_state, state_path, cli.print_log);
```

---

### `hp41-cli/src/ui.rs` (component, self-extension)

**Analog:** self (existing pending_prompt function)

**pending_prompt arm pattern** (ui.rs lines 241-264):
```rust
fn pending_prompt(pending: &crate::app::PendingInput) -> String {
    use crate::app::PendingInput;
    match pending {
        PendingInput::StoRegister(acc) => format!("STO [{:_<2}]", acc),
        // ... existing arms ...
        PendingInput::FmtDigits(mode) => {
            let label = match mode { /* ... */ };
            format!("{label} [_]  (0\u{2013}9 set digits, f cycles, Esc cancel)")
        }
    }
}
```
Add the PrintModal arm at the end of the match:
```rust
PendingInput::PrintModal => "PRNT: _".to_string(),
```

---

### `hp41-cli/src/help_data.rs` (config, self-extension)

**Analog:** self (existing HELP_DATA constant)

**Category header pattern** (help_data.rs lines 12-13):
```rust
pub const HELP_DATA: &[(&str, &str, &str)] = &[
    // ── Stack ─────────────────────────────────────────────────────────────────
    ("", "", "=== Stack ==="),
    ("Enter", "ENTER", "Lift stack and duplicate X into Y"),
```

**Entry pattern for prefix-key ops** (help_data.rs lines 59-88 — STO/RCL entries):
```rust
(
    "S",
    "STO [nn]",
    "Store X to register nn (00–99) — press S then 2 digits",
),
```
Add after last category, new Print category:
```rust
// ── Print ─────────────────────────────────────────────────────────────────
("", "", "=== Print ==="),
("P X", "PRX", "Print X register to console (right-aligned, 24 chars)"),
("P A", "PRA", "Print ALPHA register to console (left-aligned, 24 chars)"),
("P S", "PRSTK", "Print full stack T/Z/Y/X/LASTX/ALPHA (6 lines) to console"),
```

**Category count test** — the test `test_all_thirteen_categories_present` in help_data.rs (or tests/) hardcodes 13 categories. Adding `=== Print ===` makes 14. The test assertion and the list must both be updated to include `"=== Print ==="`.

---

### `hp41-core/tests/print_tests.rs` (test, CRUD)

**Analog:** `hp41-core/tests/register_tests.rs`

**Test file imports pattern** (register_tests.rs lines 1-8):
```rust
//! Integration tests for <feature>: <op names>.

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;
```
For print_tests.rs use:
```rust
//! Integration tests for print operations: PRX, PRA, PRSTK.

use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, DisplayMode};
```

**Helper function pattern** (register_tests.rs lines 7-9):
```rust
fn push(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}
```

**Test function structure pattern** (register_tests.rs lines 13-27):
```rust
#[test]
fn test_sto_rcl_round_trip() {
    let mut s = CalcState::new();
    push(&mut s, 42);
    dispatch(&mut s, Op::StoReg(5)).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(42),
        "description"
    );
}
```
Copy for print tests:
```rust
#[test]
fn test_prx_pushes_to_buffer() {
    let mut s = CalcState::new();
    // set X, call dispatch(Op::PRX), assert print_buffer[0] matches expected format
}

#[test]
fn test_prx_right_aligned_24_chars() {
    // assert all PRX output lines are exactly 24 chars wide
}

#[test]
fn test_prstk_produces_six_lines() {
    // assert print_buffer.len() == 6 after PRSTK
}

#[test]
fn test_prstk_alpha_empty_line_format() {
    // s.alpha_reg = ""; dispatch PRSTK; assert ALPHA line == "ALPHA:                 "
}

#[test]
fn test_prx_in_program() {
    // record PRX into program, run_program, assert buffer has content
}
```

---

## Shared Patterns

### LiftEffect::Neutral ops (no stack modification)
**Source:** `hp41-core/src/ops/registers.rs` lines 14-21 (op_sto)
**Apply to:** op_prx, op_pra, op_prstk in print.rs
```rust
apply_lift_effect(state, LiftEffect::Neutral);
Ok(())
```

### Modal interceptor ordering — must come AFTER pending_input check
**Source:** `hp41-cli/src/app.rs` lines 153-185
**Apply to:** 'P' interceptor placement in handle_key()
```rust
// MUST be at this exact position: after pending_input routing, before alpha mode routing
if self.pending_input.is_some() {
    self.handle_pending_input(key);
    return;
}
// 'S', 'R', 'F', 'P' interceptors here — order within this block does not matter
```

### call_dispatch error-to-message pattern
**Source:** `hp41-cli/src/app.rs` lines 771-776
**Apply to:** call_dispatch_and_drain (error arm is identical)
```rust
Err(e) => self.message = Some(format!("{e}")),
```

### serde(default) for new CalcState fields
**Source:** `hp41-core/src/state.rs` lines 87-88 (key_assignments)
**Apply to:** print_buffer field
```rust
#[serde(default)]
pub print_buffer: Vec<String>,
```
Without this, v1.0 JSON save files (which lack the `print_buffer` key) will fail to deserialize.

### #[allow(clippy::unwrap_used)] in tests
**Source:** `hp41-core/src/ops/registers.rs` line 104; `hp41-core/tests/register_tests.rs` (implicit, integration tests use .unwrap() freely)
**Apply to:** All test code in print_tests.rs and inline test modules in print.rs
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests { ... }
```

---

## No Analog Found

All files have close matches in the codebase. No files require falling back to RESEARCH.md patterns exclusively.

---

## Metadata

**Analog search scope:** `hp41-core/src/ops/`, `hp41-cli/src/`, `hp41-core/tests/`
**Files scanned:** 12 source files, 3 test files
**Pattern extraction date:** 2026-05-08
