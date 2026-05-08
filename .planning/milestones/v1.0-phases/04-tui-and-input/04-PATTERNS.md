# Phase 4: TUI & Input — Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 6 (5 new source files + 1 Cargo.toml update)
**Analogs found:** 5 / 6 (Cargo.toml has a direct analog; all 5 src files are greenfield but
have close analogs in hp41-core for their structural patterns)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-cli/Cargo.toml` | config | — | `hp41-core/Cargo.toml` | exact (workspace dep pattern) |
| `hp41-cli/src/main.rs` | utility/entry-point | request-response | `hp41-core/src/lib.rs` + RESEARCH patterns | partial (no ratatui analog in codebase) |
| `hp41-cli/src/app.rs` | controller | request-response | `hp41-core/src/ops/mod.rs` (`dispatch`) | role-match (state mutation + Result handling) |
| `hp41-cli/src/ui.rs` | component | transform | `hp41-core/src/format.rs` | partial (output formatting → display rendering) |
| `hp41-cli/src/keys.rs` | utility | transform | `hp41-core/src/ops/mod.rs` (dispatch match) + `hp41-core/src/ops/program.rs` (evaluate_test match) | role-match (match on variant → Result/Option) |
| `hp41-cli/src/prgm_display.rs` | utility | transform | `hp41-core/src/format.rs` (`format_hpnum`) + `hp41-core/src/ops/program.rs` (`op_display_name`) | role-match (state → String) |

---

## Pattern Assignments

### `hp41-cli/Cargo.toml` (config)

**Analog:** `hp41-core/Cargo.toml` (lines 1–10)

**Existing pattern to copy:**
```toml
[package]
name = "hp41-core"
version = "0.1.0"
edition = "2021"

[dependencies]
rust_decimal = { workspace = true, features = ["maths"] }
thiserror = { workspace = true }
```

**Target pattern** — extend the existing `hp41-cli/Cargo.toml` (lines 1–12) by adding:
```toml
[dependencies]
hp41-core = { path = "../hp41-core" }
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = "0.29"
clap = { version = "4", features = ["derive"] }
```

**Note:** ratatui, crossterm, and clap are NOT workspace deps (they belong to hp41-cli only). Do
NOT add them to the root `Cargo.toml` `[workspace.dependencies]`. The root workspace (lines 1–8)
uses `resolver = "2"` — no change needed there.

---

### `hp41-cli/src/main.rs` (utility, request-response)

**No direct codebase analog** — the existing stub (lines 1–5) is 4 lines with no ratatui.
Use the RESEARCH.md Pattern 1 as the definitive source.

**Stub to replace** (`hp41-cli/src/main.rs`, lines 1–5):
```rust
//! hp41-cli — thin adapter crate. Implementation begins in Phase 4.

fn main() {
    println!("HP-41 emulator — Phase 4 TUI not yet implemented.");
}
```

**Target pattern** (from RESEARCH.md Pattern 1 + Pattern 2, cross-checked with D-04/D-05/D-17):
```rust
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    // clap arg parsing here (Phase 4: --help only)

    // ratatui::init() does all terminal setup + installs panic hook.
    // Returns DefaultTerminal (NOT RestoreTerminalGuard — that type does not exist in 0.30).
    // MUST hold this value alive until ratatui::restore() is called.
    let terminal = ratatui::init();

    let result = App::new().run(terminal);

    // Explicit restore — also fires automatically on unhandled panic via the hook.
    ratatui::restore();
    result
}
```

**Error handling pattern** — main returns `std::io::Result<()>`; all hp41-core errors are
surfaced as `app.message: Option<String>` (TUI status bar), never as `io::Error`.

**Module declarations** (to be added at top of main.rs):
```rust
mod app;
mod ui;
mod keys;
mod prgm_display;

use app::App;
```

---

### `hp41-cli/src/app.rs` (controller, request-response)

**Analog:** `hp41-core/src/ops/mod.rs` — specifically the `dispatch()` function (lines 182–274)
and the `flush_entry_buf()` function (lines 158–176).

The dispatch pattern (match op → call impl fn → propagate `Result<(), HpError>`) is exactly what
`app.handle_key()` mirrors at the UI layer: match key event → call impl → propagate result to
`self.message`.

**Imports pattern** — model after lib.rs re-export pattern (`hp41-core/src/lib.rs`, lines 13–18):
```rust
use hp41_core::{CalcState, AngleMode, DisplayMode, dispatch, run_program, format_hpnum, format_alpha};
use hp41_core::ops::Op;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::time::Duration;

use crate::{keys, ui};
```

**Core struct pattern** (flat, no state machine — Claude's Discretion):
```rust
pub struct App {
    pub state: CalcState,
    pub message: Option<String>,
    pub exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            state: CalcState::new(),
            message: None,
            exit: false,
        }
    }
}
```

**Event loop pattern** (RESEARCH.md Pattern 2 — D-04 mandates `event::poll`, NEVER `event::read`
directly):
```rust
pub fn run(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
    while !self.exit {
        terminal.draw(|frame| ui::render_ui(self, frame))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        // Phase 5: auto-save timer check goes here (PERS-02)
    }
    Ok(())
}
```

**Key handler pattern** — mirrors `dispatch()` gate structure (`ops/mod.rs` lines 183–196):
```rust
fn handle_key(&mut self, key: KeyEvent) {
    // D-06: filter Release immediately — Windows fires both Press and Release.
    // This is the FIRST check — no early return before it.
    if key.kind != KeyEventKind::Press {
        return;
    }

    // Quit handlers (q and Ctrl+C)
    if key.code == KeyCode::Char('q') {
        self.exit = true;
        return;
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        self.exit = true;
        return;
    }

    // Digit entry: append directly to entry_buf, do NOT call dispatch().
    // dispatch() calls flush_entry_buf() automatically on the next non-digit op.
    // (ops/mod.rs flush_entry_buf pattern, lines 158–176)
    if let KeyCode::Char(c) = key.code {
        if c.is_ascii_digit() || c == '.' || c == 'e' {
            self.state.entry_buf.push(c);
            self.message = None;
            return;
        }
    }

    // SST / BST — direct pc manipulation, not through dispatch()
    if key.code == KeyCode::F(7) {
        if self.state.pc < self.state.program.len() {
            self.state.pc += 1;
        }
        return;
    }
    if key.code == KeyCode::F(8) {
        self.state.pc = self.state.pc.saturating_sub(1);
        return;
    }

    // F5 — R/S: hardcoded run_program("A") in Phase 4 (D-16)
    if key.code == KeyCode::F(5) {
        match run_program(&mut self.state, "A") {
            Ok(()) => self.message = None,
            Err(e) => self.message = Some(format!("{e}")),
        }
        return;
    }

    // All other ops through key_to_op() → dispatch()
    if let Some(op) = keys::key_to_op(key, self) {
        match hp41_core::ops::dispatch(&mut self.state, op) {
            Ok(()) => self.message = None,
            Err(e) => self.message = Some(format!("{e}")),
        }
    }
}
```

**Draw method** — immutable borrow of App (critical for borrow checker, RESEARCH Pitfall 4):
```rust
fn draw(&self, frame: &mut ratatui::Frame) {
    ui::render_ui(self, frame);
}
```

**Error handling pattern** — copy from `ops/mod.rs` Result propagation (lines 182–183):
- All `hp41-core` errors arrive as `Result<(), HpError>`.
- `HpError` implements `Display` via `thiserror` (`error.rs`, lines 5–16).
- Map `Err(e)` to `self.message = Some(format!("{e}"))` — never panic.

---

### `hp41-cli/src/ui.rs` (component, transform)

**Analog:** `hp41-core/src/format.rs` — the state-to-string transformation pattern. `format_hpnum`
(lines 18–25) takes `&HpNum` + `&DisplayMode` → `String`; `ui.rs` takes `&App` + `&mut Frame` →
renders to terminal buffer. Same shape: read-only state input, formatted output.

**Imports pattern** (no existing analog — use RESEARCH.md Pattern 4/5/6):
```rust
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use hp41_core::{AngleMode, format_hpnum, format_alpha};

use crate::app::App;
use crate::prgm_display;
```

**Layout pattern** (RESEARCH.md Pattern 4 — `Layout::areas::<N>()` is ratatui 0.30 idiom):
```rust
pub fn render_ui(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Minimum size check (D-01)
    if area.width < 80 || area.height < 24 {
        frame.render_widget(
            Paragraph::new("Terminal too small (need 80x24)"),
            area,
        );
        return;
    }

    let [left, right] = Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ]).areas(area);

    render_left_panel(app, frame, left);
    render_key_reference(frame, right);
}
```

**Block/title pattern** (RESEARCH.md Pattern 6 — BREAKING CHANGE in 0.30):
```rust
// CORRECT for ratatui 0.30:
let block = Block::bordered().title_top(" HP-41 ");
// WRONG (compile error in 0.30):
// Block::default().title(Title::from("..."))
```

**Annunciator rendering pattern** (RESEARCH.md Pattern 5 — bold/dim via `Stylize` trait):
```rust
fn render_annunciators(app: &App, frame: &mut Frame, area: Rect) {
    let st = &app.state;
    let ann = |label: &'static str, active: bool| -> Span<'static> {
        if active {
            Span::styled(format!("[{label}]"), Style::new().bold())
        } else {
            Span::styled(format!("[{label}]"), Style::new().dim())
        }
    };
    let line = Line::from(vec![
        ann("USER",  false),                                  // Phase 5 always dim
        ann("PRGM",  st.prgm_mode),
        ann("ALPHA", st.alpha_mode),
        ann("SHIFT", false),                                  // Phase 5 always dim
        ann("RAD",   st.angle_mode == AngleMode::Rad),
        ann("DEG",   st.angle_mode == AngleMode::Deg),
        ann("GRAD",  st.angle_mode == AngleMode::Grad),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
```

**Display string logic** (RESEARCH.md Pattern 8 — entry_buf priority, D-12):
```rust
fn get_display_string(app: &App) -> String {
    let st = &app.state;
    if !st.entry_buf.is_empty() {
        st.entry_buf.clone()
    } else if st.prgm_mode {
        prgm_display::format_step(st)
    } else if st.alpha_mode {
        format_alpha(&st.alpha_reg)
    } else {
        format_hpnum(&st.stack.x, &st.display_mode)
    }
}
```

**Stack rendering pattern** — model after how `format.rs` decomposes the state into sub-fields
(`format_hpnum` calls `n.inner()` for each register value):
```rust
fn render_stack(app: &App, frame: &mut Frame, area: Rect) {
    let st = &app.state;
    let mode = &st.display_mode;
    let lines = vec![
        Line::from(format!("T: {}", format_hpnum(&st.stack.t, mode))),
        Line::from(format!("Z: {}", format_hpnum(&st.stack.z, mode))),
        Line::from(format!("Y: {}", format_hpnum(&st.stack.y, mode))),
        Line::from(format!("X: {}", format_hpnum(&st.stack.x, mode))),
        Line::from(format!("L: {}", format_hpnum(&st.stack.lastx, mode))),
    ];
    let block = Block::bordered().title_top(" Stack ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
```

---

### `hp41-cli/src/keys.rs` (utility, transform)

**Primary analog:** `hp41-core/src/ops/mod.rs` — the `dispatch()` match statement (lines 197–273).
The key→Op match in `key_to_op()` has exactly the same structural shape: one big match arm per
operation, returning a typed result (`Option<Op>` vs `Result<(), HpError>`).

**Secondary analog:** `hp41-core/src/ops/program.rs` — `evaluate_test()` (lines 305–323). This
shows the pattern for context-sensitive dispatch: read `state` fields (e.g. `state.prgm_mode`)
to determine branch behavior, without mutating state.

**Imports pattern** (mirrors `ops/mod.rs` lines 1–24 but for crossterm types):
```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hp41_core::ops::Op;

use crate::app::App;
```

**Core match pattern** — copy the structural pattern from `dispatch()` match (ops/mod.rs lines
197–273), adapted to return `Option<Op>`:
```rust
pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op> {
    match key.code {
        KeyCode::Enter               => Some(Op::Enter),
        KeyCode::Backspace           => Some(Op::Clx),
        KeyCode::Char('+')           => Some(Op::Add),
        KeyCode::Char('-')           => Some(Op::Sub),
        KeyCode::Char('*')           => Some(Op::Mul),
        KeyCode::Char('/')           => Some(Op::Div),
        KeyCode::Char('n')           => Some(Op::Chs),
        KeyCode::Char('r')           => Some(Op::Rdn),
        KeyCode::Char('x')           => Some(Op::XySwap),
        KeyCode::Char('l')           => Some(Op::Lastx),
        KeyCode::Char('s')           => Some(Op::Sqrt),
        KeyCode::Char('p')           => Some(Op::PrgmMode),
        // Angle mode cycle — 'd' dispatches through App, not here (see app.rs)
        // Display mode cycle — 'f' dispatches through App, not here (see app.rs)
        // Uppercase = Shift+letter (D-09, A1 in RESEARCH: also match SHIFT modifier defensively)
        KeyCode::Char('S')           => Some(Op::Sin),
        KeyCode::Char('C')           => Some(Op::Cos),
        KeyCode::Char('T')           => Some(Op::Tan),
        KeyCode::Char('L')           => Some(Op::Ln),
        KeyCode::Char('G')           => Some(Op::Log),
        KeyCode::Char('E')           => Some(Op::Exp),
        KeyCode::Char('H')           => Some(Op::TenPow),
        KeyCode::Char('I')           => Some(Op::Recip),
        KeyCode::Char('W')           => Some(Op::Sq),
        KeyCode::Char('Y')           => Some(Op::YPow),
        // F5/F7/F8 handled in app.handle_key directly — return None here
        KeyCode::F(5) | KeyCode::F(7) | KeyCode::F(8) => None,
        // F1-F4: Phase 5 stubs
        KeyCode::F(1) | KeyCode::F(2) | KeyCode::F(3) | KeyCode::F(4) => None,
        _ => None,
    }
}
```

**KEY_REF_TABLE constant** — static slice for the right-panel key reference (INPUT-01):
```rust
pub const KEY_REF_TABLE: &[(&str, &str)] = &[
    ("0-9 .",  "digit entry"),
    ("Enter",  "ENTER / duplicate X"),
    ("Bksp",   "CLX (clear X)"),
    ("+ - * /","arithmetic"),
    ("n",      "CHS (change sign)"),
    ("r",      "R↓ (roll down)"),
    ("x",      "X⟷Y (swap)"),
    ("l",      "LASTX"),
    ("s",      "√x"),
    ("S",      "SIN"),
    ("C",      "COS"),
    ("T",      "TAN"),
    ("L",      "LN"),
    ("G",      "LOG"),
    ("E",      "e^x"),
    ("H",      "10^x"),
    ("I",      "1/x"),
    ("W",      "x²"),
    ("Y",      "y^x"),
    ("p",      "PRGM toggle"),
    ("d",      "cycle DEG/RAD/GRAD"),
    ("f",      "cycle FIX/SCI/ENG"),
    ("F5",     "R/S (run program A)"),
    ("F7",     "SST (step forward)"),
    ("F8",     "BST (step back)"),
    ("q",      "quit"),
    ("Ctrl+C", "quit"),
];
```

**Context-sensitivity note** (from `evaluate_test` analog, `program.rs` lines 305–323):
`key_to_op()` receives `&App` for future context-sensitivity (D-07). In Phase 4 it is mostly
unused, but the signature is already correct to allow Phase 5 to add USER mode mapping without
changing call sites.

---

### `hp41-cli/src/prgm_display.rs` (utility, transform)

**Primary analog:** `hp41-core/src/format.rs` — `format_hpnum()` (lines 18–25) and
`format_alpha()` (lines 29–31). Both take a single value from `CalcState` and return a `String`.
`format_step()` does the same: `&CalcState` → `String`.

**Secondary analog:** `hp41-core/src/ops/program.rs` — `run_loop()` match on `Op` variants
(lines 163–209). The match structure for `op_display_name()` directly mirrors this — same set of
Op variants, same exhaustive coverage pattern.

**Imports pattern** (mirrors `format.rs` lines 1–14):
```rust
use hp41_core::CalcState;
use hp41_core::ops::Op;
```

**Core pattern** (D-14 — `{step_num:03} {op_name}`):
```rust
pub fn format_step(state: &CalcState) -> String {
    let step_num = state.pc;
    let op_name = state.program.get(step_num)
        .map(op_display_name)
        .unwrap_or_else(|| "END".to_string());
    format!("{step_num:03} {op_name}")
}
```

**Op name match pattern** — copy structural shape from `dispatch()` match (`ops/mod.rs` lines
197–273) and `run_loop` match (`program.rs` lines 164–208):
```rust
fn op_display_name(op: &Op) -> String {
    match op {
        Op::Add          => "+ ".to_string(),
        Op::Sub          => "- ".to_string(),
        Op::Mul          => "x ".to_string(),
        Op::Div          => "/ ".to_string(),
        Op::Enter        => "ENTER".to_string(),
        Op::Clx          => "CLX".to_string(),
        Op::Chs          => "CHS".to_string(),
        Op::Rdn          => "R↓".to_string(),
        Op::XySwap       => "X⟷Y".to_string(),
        Op::Lastx        => "LASTX".to_string(),
        Op::PushNum(n)   => format!("{}", n.inner()),
        Op::Recip        => "1/x".to_string(),
        Op::Sqrt         => "√x".to_string(),
        Op::Sq           => "x²".to_string(),
        Op::YPow         => "Y^X".to_string(),
        Op::Ln           => "LN".to_string(),
        Op::Log          => "LOG".to_string(),
        Op::Exp          => "e^x".to_string(),
        Op::TenPow       => "10^x".to_string(),
        Op::Sin          => "SIN".to_string(),
        Op::Cos          => "COS".to_string(),
        Op::Tan          => "TAN".to_string(),
        Op::Asin         => "ASIN".to_string(),
        Op::Acos         => "ACOS".to_string(),
        Op::Atan         => "ATAN".to_string(),
        Op::SetDeg       => "DEG".to_string(),
        Op::SetRad       => "RAD".to_string(),
        Op::SetGrad      => "GRAD".to_string(),
        Op::FmtFix(n)    => format!("FIX {n}"),
        Op::FmtSci(n)    => format!("SCI {n}"),
        Op::FmtEng(n)    => format!("ENG {n}"),
        Op::StoReg(r)    => format!("STO {r:02}"),
        Op::RclReg(r)    => format!("RCL {r:02}"),
        Op::StoArith { reg, .. } => format!("STO+ {reg:02}"), // approximate
        Op::Clreg        => "CLREG".to_string(),
        Op::AlphaToggle  => "ALPHA".to_string(),
        Op::AlphaAppend(c) => format!("'{c}'"),
        Op::AlphaClear   => "CLRALPHA".to_string(),
        Op::Lbl(s)       => format!("LBL {s}"),
        Op::Gto(s)       => format!("GTO {s}"),
        Op::Xeq(s)       => format!("XEQ {s}"),
        Op::Rtn          => "RTN".to_string(),
        Op::PrgmMode     => "PRGM".to_string(),
        Op::Test(_)      => "TEST".to_string(),
        Op::Isg(r)       => format!("ISG {r:02}"),
        Op::Dse(r)       => format!("DSE {r:02}"),
    }
}
```

**Note on return type:** Use `String` (not `&'static str`) throughout `op_display_name` because
`Op::Lbl(s)`, `Op::Gto(s)`, `Op::PushNum(n)`, etc. must include dynamic data. This matches the
approach used in `format_hpnum()` which returns `String` (`format.rs` line 18).

---

## Shared Patterns

### State Mutation Entry Point
**Source:** `hp41-core/src/ops/mod.rs`, `dispatch()` lines 182–183
**Apply to:** `app.rs` — all non-digit key handling must call `hp41_core::ops::dispatch(&mut self.state, op)`
```rust
// The ONLY mutation entry point — never call sub-functions directly from app.rs.
// flush_entry_buf() fires automatically at the start of every dispatch() call.
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    flush_entry_buf(state)?; // always first
    // ...
}
```

### Error Propagation Pattern
**Source:** `hp41-core/src/error.rs` lines 1–16 + `ops/mod.rs` `Result<(), HpError>` return type
**Apply to:** `app.rs` (`handle_key`), `main.rs` (`main` returns `io::Result<()>`)
```rust
// HpError is Display-able via thiserror; use format!("{e}") for the TUI status bar.
match dispatch(&mut self.state, op) {
    Ok(()) => self.message = None,
    Err(e) => self.message = Some(format!("{e}")),
}
```

### `CalcState::new()` Initialization
**Source:** `hp41-core/src/state.rs` lines 83–98
**Apply to:** `app.rs` `App::new()` — `App` owns exactly one `CalcState`, initialized via `CalcState::new()`.
All fields start at their HP-41 hardware cold-start defaults (angle_mode = Deg, display_mode = Fix(4), etc.).

### Entry Buffer Direct-Append (Bypass dispatch)
**Source:** `hp41-core/src/ops/mod.rs` `flush_entry_buf()` lines 158–176 + CONTEXT.md D-11
**Apply to:** `app.rs` `handle_key()` — digit keys (`0-9`, `.`, `e`) push chars directly to
`state.entry_buf` without calling `dispatch()`. The flush happens automatically on the next
non-digit `dispatch()` call.
```rust
// In handle_key(), before the key_to_op() call:
if let KeyCode::Char(c) = key.code {
    if c.is_ascii_digit() || c == '.' || c == 'e' {
        self.state.entry_buf.push(c);
        return;
    }
}
```

### KeyEventKind::Release Filter
**Source:** CONTEXT.md D-06, RESEARCH.md Pattern 3
**Apply to:** `app.rs` `handle_key()` — first line, before any other logic.
```rust
if key.kind != KeyEventKind::Press {
    return; // silently ignore Release and Repeat — required for Windows correctness
}
```

### Immutable Self in draw Closure
**Source:** RESEARCH.md Pitfall 4 (borrow checker constraint)
**Apply to:** `app.rs` `run()` and `draw()` — the draw method MUST take `&self`, not `&mut self`.
```rust
// run() holds &mut self for the loop body
// draw() must be &self so it doesn't conflict with the &mut terminal inside terminal.draw()
fn draw(&self, frame: &mut ratatui::Frame) {
    ui::render_ui(self, frame);
}
```

### ratatui 0.30 Block Title API
**Source:** RESEARCH.md Pattern 6 (breaking change in 0.30)
**Apply to:** `ui.rs` — all panel borders.
```rust
// CORRECT:  Block::bordered().title_top("label")
// WRONG:    Block::default().title(Title::from("label"))   // compile error in 0.30
```

---

## Test Pattern

**Analog:** `hp41-core/tests/entry_buf_tests.rs` (lines 1–133) and
`hp41-core/tests/prgm_mode_tests.rs` (lines 1–172)

**Apply to:** The three test files called out in RESEARCH.md Wave 0 Gaps.

Test structure to copy:
```rust
// File: hp41-cli/src/tests/keys_tests.rs
use hp41_core::ops::Op;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crate::app::App;
use crate::keys::key_to_op;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn test_enter_maps_to_op_enter() {
    let app = App::new();
    let key = make_key(KeyCode::Enter);
    assert_eq!(key_to_op(key, &app), Some(Op::Enter));
}

#[test]
fn test_unmapped_key_returns_none() {
    let app = App::new();
    let key = make_key(KeyCode::F(10));
    assert_eq!(key_to_op(key, &app), None);
}
```

```rust
// File: hp41-cli/src/tests/prgm_display_tests.rs
use hp41_core::{CalcState, ops::Op, ops::dispatch};
use crate::prgm_display::format_step;

#[test]
fn test_format_step_empty_program() {
    let state = CalcState::new(); // pc=0, program=[]
    assert_eq!(format_step(&state), "000 END");
}

#[test]
fn test_format_step_at_add() {
    let mut state = CalcState::new();
    state.prgm_mode = true;
    dispatch(&mut state, Op::Sin).unwrap(); // records Sin at index 0
    state.prgm_mode = false;
    state.pc = 0;
    assert_eq!(format_step(&state), "000 SIN");
}
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `hp41-cli/src/main.rs` (ratatui init) | entry-point | request-response | No ratatui code exists in the codebase; all ratatui patterns come from RESEARCH.md docs |

---

## Metadata

**Analog search scope:** `hp41-core/src/` (all .rs files), `hp41-cli/src/main.rs`, all `Cargo.toml` files
**Files scanned:** 15 source files (all non-target .rs files in the repo)
**Pattern extraction date:** 2026-05-07

**Key facts for executor:**
- `Op` enum has 35 variants (verified: `ops/mod.rs` lines 53–146) — `op_display_name()` must cover all 35.
- `HpNum::inner()` returns `Decimal` — use `.to_string()` or `format!("{}", n.inner())` for display.
- `hp41_core::ops::dispatch` is re-exported as a function (not a method) — call as `dispatch(&mut state, op)`.
- `run_program` is re-exported from `hp41-core` root (`lib.rs` line 18) — import as `hp41_core::run_program`.
- `AngleMode` variants: `Deg`, `Rad`, `Grad` — compare with `==` (derives `PartialEq`, `state.rs` line 31).
- `DisplayMode` variants: `Fix(u8)`, `Sci(u8)`, `Eng(u8)` — carries digit count.
- Workspace resolver is "2" (`Cargo.toml` line 2) — feature unification applies; do not duplicate shared deps.
