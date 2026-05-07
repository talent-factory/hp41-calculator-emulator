# Phase 5: Persistence & UX - Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 10 new/modified files
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/state.rs` | model | CRUD | `hp41-core/src/state.rs` (self — add fields + derives) | exact |
| `hp41-core/src/num.rs` | model | transform | `hp41-core/src/num.rs` (self — add serde) | exact |
| `hp41-core/src/ops/mod.rs` | model + dispatcher | CRUD | `hp41-core/src/ops/mod.rs` (self — add variants + dispatch arms) | exact |
| `hp41-core/src/ops/alpha.rs` | service | CRUD | `hp41-core/src/ops/alpha.rs` (self — add `op_alpha_backspace`) | exact |
| `hp41-cli/src/persistence.rs` | service | file-I/O | `hp41-core/src/ops/program.rs` (pub fn + error path pattern) | role-match |
| `hp41-cli/src/app.rs` | controller | event-driven | `hp41-cli/src/app.rs` (self — add fields + timer) | exact |
| `hp41-cli/src/keys.rs` | utility | request-response | `hp41-cli/src/keys.rs` (self — add modal routing) | exact |
| `hp41-cli/src/ui.rs` | component | request-response | `hp41-cli/src/ui.rs` (self — add overlay render fns) | exact |
| `hp41-cli/src/help_data.rs` | config/static-data | transform | `hp41-cli/src/keys.rs` `KEY_REF_TABLE` const | role-match |
| `hp41-cli/src/programs.rs` | config/static-data | transform | `hp41-core/src/ops/program.rs` (OnceLock + Vec<Op> pattern) | partial |

---

## Pattern Assignments

### `hp41-core/src/state.rs` (model, CRUD)

**Analog:** `hp41-core/src/state.rs` (same file — add fields and derives)

**Current struct derives** (lines 51–52) — must add `Serialize, Deserialize`:
```rust
// BEFORE:
#[derive(Debug, Clone)]
pub struct CalcState { ... }

// AFTER:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcState { ... }
```

**Same pattern for every nested type** (lines 31–45, 111–125):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AngleMode { Deg, Rad, Grad }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DisplayMode { Fix(u8), Sci(u8), Eng(u8) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack { ... }
```

**New fields to add to CalcState** (after `is_running: bool` at line 79):
```rust
// Phase 5 additions (D-25):
pub user_mode: bool,
pub key_assignments: std::collections::BTreeMap<char, String>,
```

**CalcState::new() initialization** (lines 83–98) — add defaults:
```rust
user_mode: false,
key_assignments: std::collections::BTreeMap::new(),
```

**regs array serde note** — `[HpNum; 100]` exceeds serde's built-in fixed-array limit of 32.
Use `serde_with` crate with `#[serde_as(as = "[_; 100]")]`, OR change `regs` to `Vec<HpNum>`
(recommended by RESEARCH.md). If using `Vec<HpNum>`, constructor stays:
```rust
regs: vec![HpNum::zero(); 100],
```
And the field declaration changes:
```rust
pub regs: Vec<HpNum>,
```

**Required imports** (add to top of state.rs):
```rust
use serde::{Serialize, Deserialize};
```

---

### `hp41-core/src/num.rs` (model, transform)

**Analog:** `hp41-core/src/num.rs` (same file — add serde derive + `#[serde(with)]`)

**Current struct** (lines 8–9):
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct HpNum(pub(crate) Decimal);
```

**After** — add derive + field attribute (D-07, RESEARCH Pattern 1):
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HpNum(
    #[serde(with = "rust_decimal::serde::str")]
    pub(crate) Decimal,
);
```

**Required imports** (add to top of num.rs after existing `use` lines):
```rust
use serde::{Serialize, Deserialize};
```

**Required Cargo.toml feature** (hp41-core/Cargo.toml):
```toml
# Change from:
rust_decimal = { workspace = true, features = ["maths"] }
# To:
rust_decimal = { workspace = true, features = ["maths", "serde-with-str"] }
# Also add:
serde = { version = "1", features = ["derive"] }
```

**JSON output shape:** `"3.1415926536"` — a JSON string, not a float. Survives round-trip exactly.

---

### `hp41-core/src/ops/mod.rs` (model + dispatcher, CRUD)

**Analog:** `hp41-core/src/ops/mod.rs` (same file — add variants + derive + dispatch arms)

**Current Op enum derive** (lines 52–53):
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Op { ... }
```

**After** — add Serialize, Deserialize (RESEARCH Pitfall 3 — all nested types must also derive):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Op { ... }
```

**Same derive pattern for sub-enums** (lines 27–33, 36–42):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoArithKind { Add, Sub, Mul, Div }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestKind {
    XEqZero, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero,
    XEqY,    XNeY,    XLtY,    XGtY,    XLeY,    XGeY,
}
```

**New Op variants to add** (after `Op::Dse(u8)` at line 145, before closing `}`):
```rust
// ── USER mode (Phase 5) ──────────────────────────────────────────────
/// USER mode toggle: flip state.user_mode. LiftEffect: Neutral.
UserMode,
// ── ALPHA backspace (Phase 5) ────────────────────────────────────────
/// ALPHA backspace: remove last char from alpha_reg (HP-41 ← key). LiftEffect: Neutral.
AlphaBackspace,
```

**Import addition** (add to top of mod.rs after existing `use` lines):
```rust
use serde::{Serialize, Deserialize};
```

**Import alpha::op_alpha_backspace** (update line 24 — current):
```rust
use alpha::{op_alpha_toggle, op_alpha_append, op_alpha_clear};
```
After:
```rust
use alpha::{op_alpha_toggle, op_alpha_append, op_alpha_clear, op_alpha_backspace};
```

**New dispatch arms** (add inside the `match op { }` block after `Op::AlphaClear` at line 255):
```rust
Op::AlphaBackspace => op_alpha_backspace(state),
Op::UserMode       => {
    state.user_mode = !state.user_mode;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**prgm_mode recording gate** — `UserMode` and `AlphaBackspace` must also be handled in
`execute_op()` in `program.rs` (lines 218–296). Copy the exact same arms pattern.

---

### `hp41-core/src/ops/alpha.rs` (service, CRUD)

**Analog:** `hp41-core/src/ops/alpha.rs` (same file — add `op_alpha_backspace`)

**Existing pattern to copy from** (lines 31–38 — `op_alpha_clear`):
```rust
/// AlphaClear: clear the alpha_reg string.
/// LiftEffect: Neutral.
pub fn op_alpha_clear(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_reg.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**New function** — same signature, `String::pop()` instead of `String::clear()`:
```rust
/// AlphaBackspace: remove the last character from alpha_reg.
/// HP-41 hardware ← (backspace) key behavior in ALPHA mode.
/// No-op if alpha_reg is already empty.
/// LiftEffect: Neutral.
pub fn op_alpha_backspace(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_reg.pop(); // String::pop() is a no-op on empty string — safe
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Imports:** No new imports needed — same `use` block as existing functions (lines 1–8).

---

### `hp41-cli/src/persistence.rs` (service, file-I/O) — NEW FILE

**Analog:** `hp41-core/src/ops/program.rs` — pub function + `Result<T, E>` error propagation pattern

**Full module structure** — copy the pub-function-with-error-propagation pattern from `program.rs`
(e.g., `run_program` signature at lines 124–142):
```rust
//! State persistence for hp41-cli: save/load CalcState to/from JSON.
//!
//! D-01: Default path ~/.hp41/autosave.json
//! D-06: StateFile version wrapper { "version": 1, "state": {...} }
//! D-03: Load failures start fresh — never panic.

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use hp41_core::CalcState;

/// Version-tagged wrapper around CalcState for forward-compatible JSON files.
/// D-06: version field enables future migration without a breaking change.
#[derive(Serialize, Deserialize)]
pub struct StateFile {
    pub version: u32,
    pub state: CalcState,
}

impl StateFile {
    pub fn current(state: CalcState) -> Self {
        StateFile { version: 1, state }
    }
}

/// Resolve the default state file path: ~/.hp41/autosave.json
/// Fallback to "./.hp41/autosave.json" if home_dir() returns None (D-06, RESEARCH Pitfall 6).
pub fn default_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("autosave.json")
}

/// Save CalcState to path as pretty-printed JSON with version wrapper.
/// Creates the parent directory if missing (D-01 — fs::create_dir_all).
pub fn save_state(path: &Path, state: &CalcState) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(path)?;
    let wrapper = StateFile::current(state.clone());
    serde_json::to_writer_pretty(file, &wrapper)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Load CalcState from path. Returns Err on missing file or parse failure.
/// Caller (D-03): show error in status bar, start fresh — never panic or propagate.
/// Pitfall 4: always reset is_running=false on load.
pub fn load_state(path: &Path) -> Result<CalcState, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let wrapper: StateFile = serde_json::from_reader(file)?;
    let mut state = wrapper.state;
    state.is_running = false; // PERS Pitfall 4: never resume mid-execution
    Ok(state)
}
```

**Test pattern** — copy inline `#[cfg(test)]` block style from `hp41-core/src/ops/registers.rs`
(test helpers use `CalcState::new()` directly, no fixtures needed):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hp41_core::CalcState;
    use std::path::PathBuf;

    #[test]
    fn test_roundtrip() {
        let dir = std::env::temp_dir().join("hp41_test_roundtrip");
        let path = dir.join("state.json");
        let state = CalcState::new();
        save_state(&path, &state).unwrap();
        let loaded = load_state(&path).unwrap();
        // Minimal check: x register and is_running
        assert!(loaded.stack.x.is_zero());
        assert!(!loaded.is_running);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_missing_file_returns_err() {
        let path = PathBuf::from("/nonexistent/path/state.json");
        assert!(load_state(&path).is_err());
    }

    #[test]
    fn test_corrupt_json_returns_err() {
        let dir = std::env::temp_dir().join("hp41_test_corrupt");
        let path = dir.join("state.json");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, b"this is not json").unwrap();
        assert!(load_state(&path).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
```

**Required Cargo.toml additions** (hp41-cli/Cargo.toml):
```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "6"
```

---

### `hp41-cli/src/app.rs` (controller, event-driven)

**Analog:** `hp41-cli/src/app.rs` (same file — add fields + auto-save timer + pending_input)

**Current App struct** (lines 17–23) — add new fields:
```rust
// BEFORE:
pub struct App {
    pub state: CalcState,
    pub message: Option<String>,
    pub exit: bool,
}
```

```rust
// AFTER (Phase 5 additions):
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::cell::RefCell;
use ratatui::widgets::TableState;
use crate::persistence;

pub struct App {
    pub state: CalcState,
    pub message: Option<String>,
    pub exit: bool,
    // Phase 5: persistence (D-05)
    pub last_save: Instant,
    pub state_path: PathBuf,
    // Phase 5: modal input (D-08)
    pub pending_input: Option<PendingInput>,
    // Phase 5: overlays (D-16, D-22)
    pub show_help: bool,
    pub help_scroll: usize,
    pub help_table_state: RefCell<TableState>, // RefCell: draw(&self) borrow compat (RESEARCH Pitfall 1)
    pub show_programs: bool,
    pub programs_scroll: usize,
    pub programs_table_state: RefCell<TableState>,
    // Phase 5: pending program load (D-22 confirmation)
    pub pending_load_program: Option<usize>, // index into SAMPLE_PROGRAMS
}
```

**PendingInput enum** — define adjacent to App (or in a `mod pending` block):
```rust
/// Transient UI state for multi-key input (D-08). NOT serialized.
#[derive(Debug, Clone)]
pub enum PendingInput {
    StoRegister(String),
    RclRegister(String),
    StoAdd(String), StoSub(String), StoMul(String), StoDiv(String),
    AssignKey,                  // waiting for key char (D-27 step 1)
    AssignLabel(char, String),  // char received; accumulating label (D-27 step 2)
    ConfirmLoad(usize),         // awaiting Y/n confirmation before loading program (D-22)
}
```

**App::new() update** (lines 26–33) — add new field initializers:
```rust
pub fn new(state: CalcState, state_path: PathBuf) -> Self {
    App {
        state,
        message: None,
        exit: false,
        last_save: Instant::now(),
        state_path,
        pending_input: None,
        show_help: false,
        help_scroll: 0,
        help_table_state: RefCell::new(TableState::default()),
        show_programs: false,
        programs_scroll: 0,
        programs_table_state: RefCell::new(TableState::default()),
        pending_load_program: None,
    }
}
```

**Auto-save timer** — insert at the exact comment marker in `run()` (line 48 of app.rs):
```rust
// Phase 5: auto-save timer check goes here (PERS-02 — 30s auto-save)
// AFTER (D-05):
if self.last_save.elapsed() >= Duration::from_secs(30) {
    if let Err(e) = persistence::save_state(&self.state_path, &self.state) {
        // One-time warning; retry on next 30s tick (Claude's Discretion)
        self.message = Some(format!("Auto-save failed: {e}"));
    }
    self.last_save = Instant::now(); // reset even on failure
}
```

**Save on graceful exit** — add before `Ok(())` at end of `run()`:
```rust
// D-05: save on graceful exit before ratatui::restore()
let _ = persistence::save_state(&self.state_path, &self.state);
```

**Ctrl+S pattern** — follows existing Ctrl+C quit pattern (lines 72–75):
```rust
// D-04: Ctrl+S — manual save
if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
    match persistence::save_state(&self.state_path, &self.state) {
        Ok(()) => self.message = Some(format!(
            "Saved to {}",
            self.state_path.display()
        )),
        Err(e) => self.message = Some(format!("Save failed: {e}")),
    }
    return;
}
```

**pending_input early-return guard** — insert in `handle_key()` BEFORE the digit-entry block
(before line 80) to intercept modal key events:
```rust
// Phase 5: modal input routing — check BEFORE alpha mode and digit entry (D-11, Pitfall 5)
if self.pending_input.is_some() {
    self.handle_pending_input(key);
    return;
}
```

**ALPHA mode routing guard** — insert AFTER pending_input guard, BEFORE digit-entry block (Pitfall 5):
```rust
// Phase 5: ALPHA mode routing (D-12) — must be BEFORE normal key_to_op routing
if self.state.alpha_mode {
    self.handle_alpha_mode_key(key);
    return;
}
```

**call_dispatch pattern** (lines 141–146) — Phase 5 adds USER mode dispatch before calling
`key_to_op`. The existing `call_dispatch` method is unchanged; add a new method:
```rust
/// USER mode dispatch: if key has an assignment, run the assigned program label.
/// Called by handle_key when state.user_mode == true (D-28).
fn try_user_dispatch(&mut self, key: KeyEvent) -> bool {
    if let KeyCode::Char(c) = key.code {
        if let Some(label) = self.state.key_assignments.get(&c).cloned() {
            match hp41_core::run_program(&mut self.state, &label) {
                Ok(()) => self.message = None,
                Err(e) => self.message = Some(format!("{e}")),
            }
            return true; // consumed
        }
    }
    false // not consumed — fall through to normal routing
}
```

---

### `hp41-cli/src/keys.rs` (utility, request-response)

**Analog:** `hp41-cli/src/keys.rs` (same file — add new mappings + modal routing)

**New key bindings to add** (to the `match key.code { }` block, lines 19–57):
```rust
// Phase 5: STO / RCL modal entry (D-10)
// S and R are currently UNMAPPED (verified: 'S' absent from key_to_op match)
// They start pending_input modal, handled in App::handle_key BEFORE key_to_op.
// key_to_op should NOT handle S/R — they are intercepted upstream.

// Phase 5: USER mode toggle (D-26)
KeyCode::Char('u')           => Some(Op::UserMode),

// Phase 5: ALPHA backspace (D-13) — only reached when NOT in alpha_mode
// (alpha_mode routing is handled upstream in app.handle_key)
// Backspace in normal mode = CLX (existing line 22) — add AlphaBackspace dispatch
// to the ALPHA mode handler in App, not here.
```

**KEY_REF_TABLE update** (lines 63–97) — add new entries for Phase 5 operations:
```rust
// Add to KEY_REF_TABLE (after existing entries):
("u",      "USER mode toggle"),
("S (sh)", "STO [nn] (register store)"),
("R (sh)", "RCL [nn] (register recall)"),
("a",      "ALPHA toggle (or append in ALPHA mode)"),
("?",      "help overlay (toggle)"),
("Ctrl+S", "save state"),
("Ctrl+P", "program library"),
("Ctrl+A", "assign key (USER mode)"),
```

**`handle_pending_input` method** — new private method in `app.rs`, but the key-routing logic
follows the pattern already established in `handle_key` (lines 59–138). Copy the
`match key.code { }` + `return` guard structure:
```rust
// Pattern to copy for PendingInput::StoRegister modal (D-09):
fn handle_pending_input(&mut self, key: KeyEvent) {
    let pending = self.pending_input.take(); // take() clears it; re-set on continuation
    match pending {
        Some(PendingInput::StoRegister(ref acc)) => {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let mut new_acc = acc.clone();
                    new_acc.push(c);
                    if new_acc.len() == 2 {
                        let reg: u8 = new_acc.parse().unwrap_or(0);
                        self.call_dispatch(Op::StoReg(reg));
                        self.pending_input = None; // auto-dispatch done
                    } else {
                        self.pending_input = Some(PendingInput::StoRegister(new_acc));
                    }
                }
                KeyCode::Backspace => {
                    // D-09: Backspace resets entire accumulator (not just last digit)
                    self.pending_input = Some(PendingInput::StoRegister(String::new()));
                }
                KeyCode::Esc => {
                    self.pending_input = None; // D-09: Esc cancels
                }
                _ => {
                    self.pending_input = pending; // restore — consume but ignore
                }
            }
        }
        // ... RclRegister, StoAdd, StoSub, StoMul, StoDiv follow identical structure ...
        _ => { self.pending_input = pending; }
    }
}
```

---

### `hp41-cli/src/ui.rs` (component, request-response)

**Analog:** `hp41-cli/src/ui.rs` (same file — add overlay render calls after existing panels)

**Overlay call pattern** — insert at end of `render_ui()` (after line 43, after `render_right_panel`):
```rust
// Phase 5: overlays — rendered AFTER main panels so they appear on top (draw-order z-ordering)
if app.show_help {
    render_help_overlay(app, frame);
}
if app.show_programs {
    render_programs_overlay(app, frame);
}
```

**Annunciator update** — update `render_annunciators()` (line 150 — currently `ann("USER", false)`):
```rust
// BEFORE:
ann("USER",  false),
// AFTER (D-26 — wire to state.user_mode):
ann("USER",  st.user_mode),
```

**Status bar pending_input display** — update `render_status()` (lines 167–173).
Copy the existing `app.message.as_deref().unwrap_or("Ready")` pattern, prepend pending check:
```rust
fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    // D-11: pending_input prompts override normal message
    let text = if let Some(ref pending) = app.pending_input {
        pending_prompt(pending)
    } else if app.state.alpha_mode {
        // D-14: ALPHA mode status
        "ALPHA mode — Enter or A to exit".to_string()
    } else {
        app.message
            .as_deref()
            .unwrap_or("Ready")
            .to_string()
    };
    frame.render_widget(Paragraph::new(text), area);
}

fn pending_prompt(pending: &PendingInput) -> String {
    match pending {
        PendingInput::StoRegister(acc) => format!("STO [{:_<2}]", acc),
        PendingInput::RclRegister(acc) => format!("RCL [{:_<2}]", acc),
        PendingInput::StoAdd(acc)      => format!("STO+ [{:_<2}]", acc),
        PendingInput::StoSub(acc)      => format!("STO- [{:_<2}]", acc),
        PendingInput::StoMul(acc)      => format!("STO× [{:_<2}]", acc),
        PendingInput::StoDiv(acc)      => format!("STO÷ [{:_<2}]", acc),
        PendingInput::AssignKey        => "Assign: press key".to_string(),
        PendingInput::AssignLabel(c, acc) => format!("Assign {c} → LBL: [{acc}]"),
        PendingInput::ConfirmLoad(idx) => format!(
            "Load program {}? [Y/n]", idx
        ),
    }
}
```

**Help overlay render function** — new private function, follows `render_right_panel` structure
(lines 177–192) for the `Block::bordered().title_top()` pattern:
```rust
fn render_help_overlay(app: &App, frame: &mut Frame) {
    use ratatui::layout::Constraint;
    use ratatui::widgets::{Row, Cell, Table};

    let area = frame.area().centered(
        Constraint::Percentage(80),
        Constraint::Percentage(90),
    );

    let rows: Vec<Row> = crate::help_data::HELP_DATA.iter().map(|(key, op, desc)| {
        Row::new(vec![
            Cell::from(*key),
            Cell::from(*op),
            Cell::from(*desc),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Min(30),
    ])
    .block(Block::bordered().title_top(" HP-41 Function Reference "))
    .row_highlight_style(Style::new().bold());

    // RESEARCH Pitfall 1: draw(&self) requires RefCell for TableState mutation
    frame.render_stateful_widget(
        table,
        area,
        &mut app.help_table_state.borrow_mut(),
    );
}
```

**New imports** needed in ui.rs:
```rust
use ratatui::widgets::{Row, Cell, Table, TableState};
use crate::app::PendingInput;
use crate::help_data;
use crate::programs;
```

---

### `hp41-cli/src/help_data.rs` (config/static-data, transform) — NEW FILE

**Analog:** `hp41-cli/src/keys.rs` `KEY_REF_TABLE` constant (lines 63–97)

**Copy the static `&[(...)]` const array pattern exactly**:
```rust
//! Static help data for the HP-41 function reference overlay (D-18).
//!
//! Format: (key_binding, hp41_op_name, description)
//! Category headers use ("", "", "=== Category ===") pattern for visual grouping.
//! ~130 entries covering all keyboard-accessible operations.

/// All HP-41 operations with key bindings and descriptions.
/// Same source of truth as keys.rs KEY_REF_TABLE — extended with descriptions.
/// Category headers: key="" op="" desc="=== Category ===" for Table rendering.
pub const HELP_DATA: &[(&str, &str, &str)] = &[
    // Stack
    ("",        "",          "=== Stack ==="),
    ("Enter",   "ENTER",     "Lift stack, duplicate X"),
    ("Bksp",    "CLX",       "Clear X register"),
    ("n",       "CHS",       "Change sign of X"),
    ("r",       "R↓",        "Roll stack down"),
    ("x",       "X⟷Y",      "Swap X and Y"),
    ("l",       "LASTX",     "Recall last X"),
    // Arithmetic
    ("",        "",          "=== Arithmetic ==="),
    ("+",       "+",         "Add Y + X"),
    ("-",       "-",         "Subtract Y - X"),
    ("*",       "×",         "Multiply Y × X"),
    ("/",       "÷",         "Divide Y ÷ X"),
    // ... continue for all ~130 entries per D-18 categories ...
];
```

**Category structure** (D-18): Stack, Arithmetic, Trig, Math, Registers, ALPHA, Programming,
Display, Persistence, USER — use category header rows with empty key/op fields.

**Module declaration** — add `mod help_data;` to `hp41-cli/src/main.rs`.

---

### `hp41-cli/src/programs.rs` (config/static-data, transform) — NEW FILE

**Analog:** `hp41-core/src/ops/program.rs` for the `Vec<Op>` construction pattern;
`hp41-cli/src/keys.rs` for the static data module structure.

**CRITICAL: OnceLock pattern** (RESEARCH Pitfall 2) — `Op::Lbl(String)` is not const-constructible.
Use `std::sync::OnceLock<Vec<SampleProgram>>`:

```rust
//! Sample program library for the HP-41 TUI (D-21 through D-24).
//!
//! Programs stored as Vec<Op> via OnceLock (RESEARCH Pitfall 2 — Op::Lbl(String) is not const).
//! All programs start with Op::Lbl("A") so run_program(state, "A") works uniformly (D-specifics).

use std::sync::OnceLock;
use hp41_core::ops::Op;
use hp41_core::HpNum;

/// A bundled sample program with metadata.
pub struct SampleProgram {
    pub name: &'static str,
    pub description: &'static str,
    pub ops: Vec<Op>,
}

static PROGRAMS_CACHE: OnceLock<Vec<SampleProgram>> = OnceLock::new();

/// Access the bundled sample programs (lazily initialized, thread-safe).
pub fn sample_programs() -> &'static [SampleProgram] {
    PROGRAMS_CACHE.get_or_init(build_programs)
}

fn build_programs() -> Vec<SampleProgram> {
    vec![
        SampleProgram {
            name: "Fibonacci",
            description: "Generates Fibonacci sequence via ISG loop",
            ops: fibonacci_ops(),
        },
        // ... 9 more programs ...
    ]
}

fn fibonacci_ops() -> Vec<Op> {
    vec![
        Op::Lbl("A".to_string()),
        // ... program body using Op variants from hp41-core/src/ops/mod.rs ...
    ]
}
```

**Op variant availability** — all Op variants available in Phase 5:
`Add, Sub, Mul, Div, Enter, Clx, Chs, Rdn, XySwap, Lastx, PushNum(HpNum),
Recip, Sqrt, Sq, YPow, Ln, Log, Exp, TenPow, Sin, Cos, Tan, Asin, Acos, Atan,
StoReg(u8), RclReg(u8), StoArith, Clreg, AlphaToggle, AlphaAppend(char), AlphaClear,
Lbl(String), Gto(String), Xeq(String), Rtn, PrgmMode, Test(TestKind), Isg(u8), Dse(u8),
UserMode, AlphaBackspace`

**HpNum construction for PushNum** — follow `hp41-core/src/tests.rs` pattern (lines 60–62):
```rust
// For integer values:
Op::PushNum(HpNum::from(1i32)),
// For decimal values:
use rust_decimal::Decimal;
use std::str::FromStr;
Op::PushNum(HpNum::from(Decimal::from_str("0.001").unwrap())),
```

**Test pattern** — inline `#[cfg(test)]` (copy from `hp41-core/src/tests.rs` module style):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_count() {
        assert!(sample_programs().len() >= 10, "Must have at least 10 programs");
    }

    #[test]
    fn test_programs_non_empty() {
        for prog in sample_programs() {
            assert!(!prog.ops.is_empty(), "Program '{}' has empty ops", prog.name);
        }
    }

    #[test]
    fn test_all_programs_start_with_lbl_a() {
        for prog in sample_programs() {
            assert!(
                matches!(prog.ops.first(), Some(Op::Lbl(l)) if l == "A"),
                "Program '{}' must start with Op::Lbl(\"A\")", prog.name
            );
        }
    }
}
```

**Module declaration** — add `mod programs;` to `hp41-cli/src/main.rs`.

---

## Shared Patterns

### Error Handling (all files)
**Source:** `hp41-core/src/ops/registers.rs` (lines 14–21) + `hp41-cli/src/app.rs` (lines 141–146)
**Apply to:** All `persistence.rs` functions, all new dispatch arms

Pattern: `Result<T, E>` propagation in core; `match` + `self.message = Some(format!("{e}"))` in CLI:
```rust
// In hp41-core: propagate with ?
pub fn op_alpha_backspace(state: &mut CalcState) -> Result<(), HpError> {
    state.alpha_reg.pop();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// In hp41-cli App::call_dispatch (lines 141-146):
fn call_dispatch(&mut self, op: Op) -> {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => self.message = None,
        Err(e) => self.message = Some(format!("{e}")),
    }
}
```

### LiftEffect: Neutral (all new core ops)
**Source:** `hp41-core/src/ops/alpha.rs` (lines 14–17, 28–30, 35–38)
**Apply to:** `op_alpha_backspace`, `Op::UserMode` dispatch arm

Every new op in hp41-core must end with `apply_lift_effect(state, LiftEffect::Neutral)` and
return `Ok(())`. No exceptions for Phase 5 ops (all are state toggles / simple mutations).

### KeyEventKind::Release Filter
**Source:** `hp41-cli/src/app.rs` (lines 63–65)
**Apply to:** `handle_pending_input`, `handle_alpha_mode_key` (all new handle_* methods in App)

```rust
// This MUST be the first check in every key handler method:
if key.kind != KeyEventKind::Press {
    return;
}
```

### Module Declaration in main.rs
**Source:** `hp41-cli/src/main.rs` (lines 11–17)
**Apply to:** All new CLI modules (`persistence`, `help_data`, `programs`)

```rust
// Follow the existing pattern:
mod app;
mod ui;
mod keys;
mod prgm_display;
// Phase 5 additions:
mod persistence;
mod help_data;
mod programs;
```

### Static Data Array
**Source:** `hp41-cli/src/keys.rs` `KEY_REF_TABLE` (lines 63–97)
**Apply to:** `help_data.rs` HELP_DATA constant

Tuple slice pattern `&[(&str, &str, &str)]` with `pub const` — no allocation, zero-cost.

### Block::bordered().title_top() Widget Pattern
**Source:** `hp41-cli/src/ui.rs` (lines 107–112, 180–181)
**Apply to:** Help overlay block, program library overlay block

```rust
// Established pattern for all bordered panels:
let block = Block::bordered().title_top(" Title ");
frame.render_widget(widget.block(block), area);
```

---

## No Analog Found

All files have analogs in the codebase. No entries in this section.

---

## Critical Implementation Notes

### Wave Order (from RESEARCH.md)
1. **Wave 1 — Serde derives:** `num.rs` → `state.rs` → `ops/mod.rs` (+ sub-enums) — must compile before any other wave
2. **Wave 2 — Persistence:** `persistence.rs` + `main.rs` clap arg + App field additions
3. **Wave 3 — Overlays:** `help_data.rs` + `programs.rs` + `ui.rs` overlay functions
4. **Wave 4 — USER mode + modal input:** `keys.rs` new bindings + `app.rs` handle methods

### Open Question Resolution Required Before Planning
From RESEARCH.md Open Question 1: `regs: [HpNum; 100]` — serde derive limit at N=32.
**Recommended resolution:** Change `regs` field type from `[HpNum; 100]` to `Vec<HpNum>`.
Constructor: `regs: vec![HpNum::zero(); 100]`. Accessor index `state.regs[n as usize]` is
unchanged — Vec and array have identical index syntax. This is the lowest-risk solution.
Alternative: `serde_arrays` crate (one extra dep). Do NOT use `serde_with` for this alone.

From RESEARCH.md Open Question 3: `draw(&self)` vs `RefCell<TableState>`.
**Resolved:** Use `RefCell<TableState>` for `help_table_state` and `programs_table_state`.
Single-threaded, non-reentrant draw loop — `borrow_mut()` will never panic in practice.

---

## Metadata

**Analog search scope:** `hp41-core/src/`, `hp41-cli/src/`
**Files scanned:** 14 source files + 3 Cargo.toml files
**Pattern extraction date:** 2026-05-07
