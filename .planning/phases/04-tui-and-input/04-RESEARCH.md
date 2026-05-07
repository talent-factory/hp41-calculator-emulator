# Phase 4: TUI & Input - Research

**Researched:** 2026-05-07
**Domain:** ratatui 0.30 + crossterm 0.29 TUI; Rust module architecture; terminal event loop
**Confidence:** HIGH (ratatui/crossterm APIs verified via docs.rs + official ratatui.rs; codebase verified by direct read)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**TUI Layout (D-01 through D-03)**
- Fixed single-panel layout (no dynamic resize required for v1.0). Columns: left = stack+display+annunciators, right = key-reference table. Minimum terminal size: 80x24.
- Stack display: X/Y/Z/T each on own line, labeled. LASTX shown below T. 12-char HP-41 display shown prominently. Annunciator bar: `[USER] [PRGM] [ALPHA] [SHIFT] [RAD] [DEG] [GRAD]` — lit/dim based on CalcState flags.
- Key-reference panel shows a compact table: key → function. Discoverable without external reference. Updated dynamically when USER mode is active.

**Event Loop (D-04 through D-06)**
- `event::poll(Duration::from_millis(16))` — NEVER `event::read()`; required for future auto-save timer (Phase 5)
- `ratatui::init()` not `Terminal::new()` — installs panic hook for SC-4
- Filter `KeyEventKind::Release` immediately — Windows crossterm fires both Press and Release; only `KeyEventKind::Press` is processed

**Keyboard Mapping (D-07 through D-10)**
- `fn key_to_op(key: KeyEvent, app: &App) -> Option<Op>` in `keys.rs`
- Core key assignments defined in D-08 (digits, Enter, Backspace, +, -, *, /, n, r, x, l, s, q, p, F5, F1-F4)
- Shift-key modifier for trig/math: `S`=SIN, `C`=COS, `T`=TAN, `L`=LN, `G`=LOG, `E`=EXP, `H`=10^x, `I`=1/x, `W`=x², `Y`=y^x
- `d` cycles angle mode DEG→RAD→GRAD; `f` cycles display format FIX→SCI→ENG (digit count 4, adjustment deferred)

**Digit Entry (D-11 through D-13)**
- Digit keys (`0-9`, `.`) append to `state.entry_buf` directly (NOT via dispatch)
- `flush_entry_buf` called automatically by dispatch() on next op
- `e` key appends "E" separator to entry_buf for SCI entry
- Display shows `entry_buf` content while non-empty; otherwise `format_hpnum(&state.stack.x, &state.display_mode)`

**PRGM Mode Display (D-14 through D-16)**
- When `state.prgm_mode = true`, main display area shows `{step_num:03} {op_name}` where step_num = `state.pc`
- SST: `F7` key — increment state.pc. BST: `F8` key — decrement state.pc
- R/S: `F5` calls `run_program(state, "A")` hardcoded in Phase 4

**Panic Handling (D-17 through D-18)**
- `ratatui::init()` return value (DefaultTerminal) must be held until program exit — do NOT drop early
- All hp41-core errors are `Result<(), HpError>` — no panics in core; CLI boundary displays errors in status bar

**Claude's Discretion**
- Color scheme: minimal — use terminal defaults + bold for active annunciators, dim for inactive
- App architecture: `struct App { state: CalcState, message: Option<String> }` — simple flat struct
- Exit: `q` or `Ctrl+C` both quit cleanly

**Files locked:**
- `hp41-cli/src/main.rs` — entry point, clap args, ratatui::init(), event loop
- `hp41-cli/src/app.rs` — App struct with CalcState; update(event) and render(frame) methods
- `hp41-cli/src/ui.rs` — ratatui widget layout
- `hp41-cli/src/keys.rs` — key→Op mapping table + digit entry state machine
- `hp41-cli/src/prgm_display.rs` — PRGM mode step display

### Claude's Discretion
(listed above under Claude's Discretion heading)

### Deferred Ideas (OUT OF SCOPE)
- Help overlay / searchable function reference — Phase 5 (UX-01)
- USER mode key assignments — Phase 5 (UX-02)
- F1-F4 user-assignable keys — Phase 5
- Auto-save timer inside event loop — Phase 5
- Label entry dialog for R/S — Phase 5
- Full digit-count adjustment (FIX 4, SCI 2, etc.) via key sequence — Phase 5
- Terminal resize handling — deferred; minimum 80x24 enforced with error message
- Mouse support — out of scope for v1.0
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DISP-01 | User sees the HP-41 alphanumeric 12-character display with annunciators (USER, PRGM, ALPHA, SHIFT, RAD/DEG/GRAD) in the TUI | `format_hpnum` and `format_alpha` already exist in hp41-core; annunciator state maps directly from CalcState fields; ratatui Paragraph + styled Span enables lit/dim rendering |
| DISP-02 | User sees a persistent TUI panel showing the 4-level stack (X/Y/Z/T), LASTX, annunciator bar, and current display at all times | Two-column Layout::horizontal split; left column nested Layout::vertical for stack rows; immediate-mode draw loop ensures persistence |
| INPUT-01 | User can operate all calculator functions via physical keyboard in the TUI without needing an external reference for basic operations | Right-panel key-reference table built from the same key→Op mapping table in keys.rs; all ops in Op enum are mapped in keys.rs lookup |
</phase_requirements>

---

## Summary

Phase 4 wires together the fully-built `hp41-core` library (CalcState, dispatch, format_hpnum, run_program) with a ratatui 0.30 + crossterm 0.29 TUI shell. The research confirms all locked decisions are technically sound and the APIs are available exactly as planned.

The critical API fact is that `ratatui::init()` returns `DefaultTerminal` (NOT a `RestoreTerminalGuard`). The `DefaultTerminal` value IS the terminal handle and also contains the panic hook installed during `init()`. The guard pattern is implicit: hold the `DefaultTerminal` variable alive until `main()` exits or until `ratatui::restore()` is called explicitly. CONTEXT.md's D-17 phrasing ("RestoreTerminalGuard") is inaccurate for ratatui 0.30 — the actual return type is `DefaultTerminal`.

The borrow-checker pattern for `App` struct + ratatui `terminal.draw()` is well-established: `terminal.draw(|frame| self.draw(frame))` where `draw(&self, frame: &mut Frame)` takes `&self` (immutable borrow of `App`). This separates the mutable self borrow (for `update()`) from the immutable borrow (for `render()`). Since the draw closure borrows `self` immutably and immediately returns, there is no lifetime conflict. Key handling happens OUTSIDE the draw closure, preventing any overlap.

The `hp41-cli` crate currently has a 4-line stub `main.rs`. All five target files (`main.rs`, `app.rs`, `ui.rs`, `keys.rs`, `prgm_display.rs`) must be created from scratch. The `Cargo.toml` must be updated to add ratatui 0.30 + crossterm 0.29 + clap 4.x.

**Primary recommendation:** Follow the locked architecture exactly. The standard ratatui app pattern (App struct, `terminal.draw(|f| self.draw(f))`, event loop with `event::poll`) is well-documented and the only complexity is the key mapping table completeness and the ratatui 0.30 breaking change around `Block::title()`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Stack/register state mutation | hp41-core (library) | — | All state lives in CalcState; dispatch() is the single mutation entry point |
| Number formatting (FIX/SCI/ENG) | hp41-core (format.rs) | — | format_hpnum() and format_alpha() are already exported from lib.rs |
| Display rendering | hp41-cli (ui.rs) | — | Ratatui widgets are UI-layer concerns; hp41-core has zero ratatui dep |
| Key event → Op mapping | hp41-cli (keys.rs) | — | Purely UI-layer translation; Op enum from hp41-core is the output |
| Digit entry accumulation | hp41-cli (keys.rs/app.rs) | hp41-core (entry_buf field + flush_entry_buf) | CLI appends chars; core flushes on next dispatch |
| PRGM mode step display | hp41-cli (prgm_display.rs) | hp41-core (program Vec, pc field) | Display logic in CLI; program data from CalcState |
| Panic terminal restoration | hp41-cli (main.rs) | ratatui::init() hook | ratatui::init() installs hook; CLI must not drop DefaultTerminal early |
| Event loop timing | hp41-cli (main.rs) | crossterm (event::poll) | 16ms poll loop owns the main thread |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 | TUI rendering framework — widgets, layout, buffered draw | Only TUI library with Windows 10+ support, active maintenance, and crossterm integration; project-mandated [VERIFIED: cargo search] |
| crossterm | 0.29.0 | Cross-platform terminal backend (raw mode, alternate screen, key events) | Default backend for ratatui; Windows + macOS + Linux support [VERIFIED: cargo search] |
| clap | 4.x | CLI argument parsing | Project-mandated in CLAUDE.md; derive feature for ergonomic usage [CITED: CLAUDE.md] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| hp41-core | workspace path | All calculator state and ops | Always — it is the domain library being surfaced by the CLI |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ratatui 0.30 | tui-rs (archived) | tui-rs is unmaintained; ratatui is the active fork; no real tradeoff — ratatui wins |
| crossterm backend | termion | termion is UNIX-only; crossterm provides Windows support required by QUAL-05 |

### Installation (Cargo.toml update for hp41-cli)
```toml
[dependencies]
hp41-core = { path = "../hp41-core" }
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = "0.29"
clap = { version = "4", features = ["derive"] }
```

**Note:** `ratatui = "0.30"` enables crossterm by default. The explicit `features = ["crossterm"]` is for clarity. [VERIFIED: ratatui.rs/installation]

**Version verification:**
- ratatui 0.30.0 — current stable [VERIFIED: cargo search ratatui, 2026-05-07]
- crossterm 0.29.0 — current stable [VERIFIED: cargo search crossterm, 2026-05-07]

---

## Architecture Patterns

### System Architecture Diagram

```
Physical Keyboard
      |
      v
crossterm::event::poll(16ms)
      |
      | KeyEvent { code, modifiers, kind, state }
      v
keys.rs: key_to_op(key, &app)
      |
      +-- digit key? --> append to state.entry_buf (direct write, no dispatch)
      |
      +-- op key? --> Op variant
      |                    |
      |                    v
      |             app.rs: dispatch(&mut state, op)
      |                    |
      |                    +--> flush_entry_buf() [auto, inside dispatch]
      |                    +--> hp41-core: execute op, mutate CalcState
      |                    +--> Result<(), HpError> --> app.message on Err
      |
      +-- 'q' / Ctrl+C --> set exit = true
      |
      v
terminal.draw(|frame| app.draw(frame))
      |
      v
ui.rs: render_ui(&app, frame)
      |
      +-- Layout::horizontal [60%, 40%]
      |        |                    |
      |        v                    v
      |   left_panel          right_panel
      |   Layout::vertical    keys.rs: KEY_REF_TABLE
      |   [3 rows:            Paragraph with key list
      |    stack, display,
      |    annunciators]
      |
      +-- stack panel: X/Y/Z/T + LASTX from state.stack
      +-- display: entry_buf or format_hpnum(x, mode)
      |            OR prgm_display::format_step(state) if prgm_mode
      +-- annunciators: Span bold/dim based on CalcState flags
      |
      v
Buffer diff flush to terminal stdout
```

### Recommended Project Structure
```
hp41-cli/
├── Cargo.toml          # Add ratatui, crossterm, clap deps
└── src/
    ├── main.rs         # clap args, ratatui::init(), event loop, exit
    ├── app.rs          # App struct { state, message, exit }; update/draw
    ├── ui.rs           # ratatui widget layout (render_ui function)
    ├── keys.rs         # key_to_op(); KEY_REF_TABLE constant
    └── prgm_display.rs # format_step(state) -> String for PRGM mode
```

### Pattern 1: ratatui Initialization and Event Loop

**What:** Standard ratatui 0.30 synchronous event loop with poll-based input.
**When to use:** Always — this is the only supported pattern given D-04/D-05 decisions.

```rust
// Source: docs.rs/ratatui/0.30.0/ratatui/fn.init.html + ratatui.rs/tutorials/hello-ratatui/
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    // Parse clap args here

    // ratatui::init() does ALL of:
    //   1. Creates CrosstermBackend writing to stdout
    //   2. Enables raw mode
    //   3. Enters alternate screen
    //   4. Installs panic hook that calls ratatui::restore() before printing panic
    // Returns DefaultTerminal — NOT RestoreTerminalGuard. Hold until program exit.
    let terminal = ratatui::init();

    let result = App::new().run(terminal);

    // restore() must be called explicitly when not using ratatui::run()
    ratatui::restore();
    result
}
```

**Key fact:** `ratatui::init()` signature is:
```rust
pub fn init() -> DefaultTerminal
```
[VERIFIED: docs.rs/ratatui/0.30.0/ratatui/fn.init.html]

`DefaultTerminal` is a type alias for `Terminal<CrosstermBackend<io::Stdout>>`. There is no separate `RestoreTerminalGuard` type in ratatui 0.30.

### Pattern 2: App Event Loop with poll()

**What:** Synchronous 16ms poll loop — no blocking, supports future timer injection.
**When to use:** D-04 mandates this; never use `event::read()`.

```rust
// Source: crossterm docs.rs/crossterm/0.29.0/crossterm/event
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

impl App {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key),
                    _ => {}
                }
            }
            // Future Phase 5: insert auto-save timer check here
        }
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        ui::render_ui(self, frame);
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // D-06: Filter Release immediately — Windows fires both Press and Release
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.code == KeyCode::Char('q') {
            self.exit = true;
            return;
        }
        // Ctrl+C also exits cleanly
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit = true;
            return;
        }
        // Delegate to keys.rs
        if let Some(op) = keys::key_to_op(key, self) {
            match hp41_core::dispatch(&mut self.state, op) {
                Ok(()) => self.message = None,
                Err(e) => self.message = Some(format!("{e}")),
            }
        }
    }
}
```

### Pattern 3: KeyEventKind::Release Filter (Windows fix)

**What:** Filter to process only Press events; Release events cause double-execution on Windows.
**When to use:** Immediately at the top of any key handler — harmless on macOS/Linux, required on Windows.

```rust
// Source: github.com/ratatui/ratatui/issues/347 + crossterm docs
// Exact filter code (D-06):
if key.kind != KeyEventKind::Press {
    return; // silently ignore Release and Repeat events
}
```

The `KeyEvent::is_key_press()` helper method also works:
```rust
if !key.is_key_press() { return; }
```
[VERIFIED: docs.rs/crossterm/0.29.0/crossterm/event/struct.KeyEvent.html]

### Pattern 4: Layout Split (two-column, left nested)

**What:** Fixed two-column layout with left panel subdivided vertically.
**When to use:** D-01 defines the layout structure.

```rust
// Source: docs.rs/ratatui/0.30.0/ratatui/layout/struct.Layout.html
use ratatui::layout::{Constraint, Direction, Layout};

pub fn render_ui(app: &App, frame: &mut ratatui::Frame) {
    let area = frame.area();

    // Two columns: left 55%, right 45%
    let [left, right] = Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ]).areas(area);

    // Left: 5 rows: T, Z, Y, X+LASTX section, display+annunciators
    let [row_t, row_z, row_y, row_x_lastx, row_display, row_annunc] = Layout::vertical([
        Constraint::Length(1), // T register
        Constraint::Length(1), // Z register
        Constraint::Length(1), // Y register
        Constraint::Length(2), // X + LASTX
        Constraint::Length(3), // 12-char display (prominent)
        Constraint::Length(1), // annunciator bar
    ]).areas(left);

    // Right: key-reference panel fills the column
    render_key_reference(frame, right);
    render_stack(app, frame, row_t, row_z, row_y, row_x_lastx);
    render_display(app, frame, row_display);
    render_annunciators(app, frame, row_annunc);
}
```

**Note:** `Layout::areas::<N>()` returns `[Rect; N]` and is the idiomatic ratatui 0.30 API. [VERIFIED: docs.rs/ratatui/0.30.0/ratatui/layout/struct.Layout.html]

### Pattern 5: Paragraph Widget with Styled Annunciators

**What:** Render annunciators as a single Line of styled Spans.
**When to use:** D-02 requires lit/dim annunciators.

```rust
// Source: docs.rs/ratatui/0.30.0/ratatui/widgets/struct.Paragraph.html
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

fn render_annunciators(app: &App, frame: &mut ratatui::Frame, area: Rect) {
    let st = &app.state;
    let ann = |label: &'static str, active: bool| -> Span<'static> {
        let s = format!("[{label}]");
        if active {
            Span::styled(s, Style::new().bold())
        } else {
            Span::styled(s, Style::new().dim())
        }
    };

    let line = Line::from(vec![
        ann("USER", false),          // Phase 5 — always dim
        ann("PRGM", st.prgm_mode),
        ann("ALPHA", st.alpha_mode),
        ann("SHIFT", false),         // Phase 5 — always dim
        ann("RAD",  st.angle_mode == AngleMode::Rad),
        ann("DEG",  st.angle_mode == AngleMode::Deg),
        ann("GRAD", st.angle_mode == AngleMode::Grad),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
```

### Pattern 6: Block with Title (ratatui 0.30 API)

**What:** Bordered block with title — BREAKING CHANGE from older ratatui.
**When to use:** Any panel that needs a visible border label.

```rust
// Source: docs.rs/ratatui/0.30.0/ratatui/widgets/struct.Block.html
// BREAKING CHANGE: Block::title() struct removed in 0.30.
// Use Block::bordered().title_top("label") — accepts &str or Line directly.
use ratatui::widgets::Block;

let block = Block::bordered().title_top(" HP-41 ");
// Do NOT use the old: Block::default().title(Title::from(...))
```
[VERIFIED: ratatui.rs/highlights/v030]

### Pattern 7: Key → Op Mapping Table

**What:** Flat match statement translating KeyEvent → Option<Op>.
**When to use:** Called once per key press from app.handle_key().

```rust
// Source: crossterm docs + hp41-core/src/ops/mod.rs Op enum
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hp41_core::ops::Op;

pub fn key_to_op(key: KeyEvent, _app: &App) -> Option<Op> {
    match key.code {
        // Digit entry — handled BEFORE this function by checking is_digit()
        // This function returns None for pure digit keys; app.handle_key appends directly.

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
        // Angle mode cycle handled separately in app — dispatches SetDeg/Rad/Grad
        // Display mode cycle handled separately in app — dispatches FmtFix/Sci/Eng

        // Shift + letter for trig/math (uppercase chars via Shift)
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

        // F-keys
        KeyCode::F(5)                => Some(Op::Rtn), // R/S stub (hardcoded run_program("A") in app)
        KeyCode::F(7)                => None,          // SST — handled in app directly (state.pc++)
        KeyCode::F(8)                => None,          // BST — handled in app directly (state.pc--)
        KeyCode::F(1) | KeyCode::F(2) | KeyCode::F(3) | KeyCode::F(4) => None, // Phase 5 stub

        _ => None,
    }
}
```

**Important:** `KeyCode::Char('S')` (uppercase) is what crossterm emits when Shift+s is pressed — the modifier field may also carry `KeyModifiers::SHIFT` but the char variant already reflects the shifted character. [VERIFIED: crossterm docs.rs]

### Pattern 8: Display Rendering with entry_buf Priority

**What:** Shows live digit preview while typing; switches to formatted number when buffer empty.
**When to use:** D-12 requirement.

```rust
// Source: hp41-core/src/format.rs + hp41-core/src/state.rs
use hp41_core::{format_hpnum, format_alpha};

fn get_display_string(state: &CalcState) -> String {
    if !state.entry_buf.is_empty() {
        // Live digit preview during entry
        state.entry_buf.clone()
    } else if state.alpha_mode {
        format_alpha(&state.alpha_reg)
    } else {
        format_hpnum(&state.stack.x, &state.display_mode)
    }
}
```

### Anti-Patterns to Avoid

- **Using `event::read()` instead of `poll()`:** Blocks the thread indefinitely — prevents future auto-save timer injection and breaks the 16ms redraw cycle.
- **Dropping DefaultTerminal before program exit:** Calling `drop(terminal)` or letting it go out of scope early means `ratatui::restore()` is NOT called — terminal stays in raw mode after exit.
- **Not filtering `KeyEventKind::Release`:** On Windows, every keystroke fires two events. Every calculator operation executes twice. Filter at the FIRST line of the key handler.
- **Using `Block::title(Title::from(...))` (old API):** Removed in ratatui 0.30. Use `Block::bordered().title_top("label")` or `Block::bordered().title("label")` with a `Line` or `&str`.
- **Borrowing `state` mutably inside `terminal.draw()` closure:** The draw closure borrows `self` immutably through `self.draw(frame)`. Do NOT call `dispatch()` inside the draw closure — event handling and rendering must be in separate phases of the loop.
- **Calling `flush_entry_buf()` directly from keys.rs:** It is called automatically at the start of every `dispatch()`. Only digit keys (appending to `entry_buf` directly) bypass dispatch — and that is intentional.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Terminal raw mode + alternate screen | Custom terminal manipulation | `ratatui::init()` | Panic restoration, platform differences, cleanup on exit are all handled |
| Diff-based terminal update | Full terminal redraw every frame | `terminal.draw()` buffered render | ratatui diffs buffers and writes only changed cells — essential for ≤50ms latency |
| Number formatting (FIX/SCI/ENG) | Custom format logic | `hp41_core::format_hpnum()` | Already implemented, tested, HP-41-accurate — do not duplicate |
| Key modifier detection | Bit-twiddling on raw scancodes | `KeyEvent.modifiers.contains(KeyModifiers::SHIFT)` | crossterm abstracts platform-specific modifier encoding |
| Layout math (pixel/character positions) | Manual Rect arithmetic | `Layout::horizontal/vertical` with `Constraint` | ratatui handles clipping, overflow, minimum-size enforcement |

**Key insight:** The hp41-core library already provides all domain logic (format, dispatch, state). Phase 4 is entirely UI wiring — do not reimplement anything that exists in hp41-core.

---

## Runtime State Inventory

Phase 4 is greenfield UI code — no rename/refactor involved. This section is intentionally omitted.

---

## Common Pitfalls

### Pitfall 1: RestoreTerminalGuard Does Not Exist in Ratatui 0.30
**What goes wrong:** Planner or executor searches for `RestoreTerminalGuard` type and fails to compile.
**Why it happens:** CONTEXT.md D-17 uses the term "RestoreTerminalGuard" which was from an older ratatui version or conceptual description.
**How to avoid:** `ratatui::init()` returns `DefaultTerminal`. The terminal restoration on drop/panic is built into the panic hook that `init()` installs. Store the returned `DefaultTerminal` and call `ratatui::restore()` after the run loop finishes.
**Warning signs:** `error[E0412]: cannot find type 'RestoreTerminalGuard' in crate 'ratatui'`

### Pitfall 2: Windows Double Key Execution
**What goes wrong:** Every calculator operation fires twice on Windows — user presses `+` and gets two additions.
**Why it happens:** crossterm on Windows 10+ reports both `KeyEventKind::Press` and `KeyEventKind::Release` for every physical key press.
**How to avoid:** Add `if key.kind != KeyEventKind::Press { return; }` as the very first line of `handle_key()`.
**Warning signs:** Manual testing on Windows shows stack operations executing in pairs.

### Pitfall 3: Block::title() API Removed in Ratatui 0.30
**What goes wrong:** Code using the old `Block::default().title(Title::from("text"))` API fails to compile.
**Why it happens:** ratatui 0.30 removed the `block::Title` struct as part of the widget API cleanup.
**How to avoid:** Use `Block::bordered().title_top("label")` or `Block::new().title("label")` — both accept `&str`, `String`, or `Line` directly.
**Warning signs:** `error[E0412]: cannot find struct 'Title' in module 'ratatui::widgets::block'`

### Pitfall 4: Mutable Borrow of App in terminal.draw() Closure
**What goes wrong:** Attempting to call `dispatch()` or mutate state inside the `terminal.draw()` closure fails because `self` is already borrowed.
**Why it happens:** `terminal.draw(|frame| ...)` takes `&mut terminal`; if the closure also captures `&mut self`, the borrow checker rejects it.
**How to avoid:** The draw closure ONLY reads state for rendering. All state mutation (dispatch, entry_buf append) happens in `handle_key()` which runs between draw calls, never inside the closure. Use `terminal.draw(|frame| self.draw(frame))` where `draw(&self, frame)` takes `&self` (immutable).
**Warning signs:** `error[E0502]: cannot borrow 'self' as mutable because it is also borrowed as immutable`

### Pitfall 5: entry_buf Digit Handling Bypass
**What goes wrong:** Routing digit keys through `dispatch()` instead of appending to `entry_buf` directly. Result: every digit press pushes a number onto the stack instead of building a multi-digit number.
**Why it happens:** The natural instinct is to call `dispatch(state, Op::PushNum(digit))`, but HP-41 entry model requires accumulating digits first.
**How to avoid:** In `handle_key()`, check if `key.code` is `KeyCode::Char(c)` where `c.is_ascii_digit() || c == '.'` — if so, append to `state.entry_buf` directly (without dispatch). dispatch() will flush entry_buf automatically on the next non-digit key.
**Warning signs:** Single-digit numbers work; multi-digit numbers push multiple stack values.

### Pitfall 6: Rust 2024 Edition + MSRV 1.86
**What goes wrong:** ratatui 0.30 adopted the Rust 2024 edition and bumped MSRV to 1.86. Code using older edition idioms may not work.
**Why it happens:** ratatui 0.30 is an ecosystem-wide upgrade that aligns with Rust's new edition.
**How to avoid:** The project already uses Rust 1.89 (verified: `rustc --version`), which exceeds the 1.86 MSRV. No action needed — but do not add a lower `rust-version` in Cargo.toml.
**Warning signs:** Compile errors on older Rust installs in CI.

### Pitfall 7: F5 Hardcoded "A" Label for run_program()
**What goes wrong:** Pressing F5 when no program with label "A" exists causes `HpError::InvalidOp` — displayed as an error message.
**Why it happens:** D-16 hardcodes `run_program(state, "A")` for Phase 4. The error is expected behavior and should display gracefully in the status bar, not panic.
**How to avoid:** Wrap the F5 handler in a match on Result, setting `app.message` on Err.
**Warning signs:** F5 with no "A" label crashes or shows unhandled error.

---

## Code Examples

### Minimal Complete Event Loop (main.rs structure)

```rust
// Source: ratatui.rs/tutorials/hello-ratatui/ + docs.rs/ratatui/0.30.0
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    // Optional: parse clap args here before init()

    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
```

### App Struct (app.rs)

```rust
// Source: ratatui.rs pattern (ratatui-a-more-structured-way-to-handle-state)
use hp41_core::CalcState;

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

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        use std::time::Duration;
        use crossterm::event::{self, Event};

        while !self.exit {
            terminal.draw(|frame| ui::render_ui(self, frame))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }
}
```

### Op Name for PRGM Display (prgm_display.rs)

```rust
// Source: hp41-core/src/ops/mod.rs Op enum
use hp41_core::CalcState;
use hp41_core::ops::Op;

pub fn format_step(state: &CalcState) -> String {
    let step_num = state.pc;
    let op_name = state.program.get(step_num)
        .map(op_display_name)
        .unwrap_or("END");
    format!("{step_num:03} {op_name}")
}

fn op_display_name(op: &Op) -> &'static str {
    match op {
        Op::Add     => "+ ",
        Op::Sub     => "- ",
        Op::Mul     => "×",
        Op::Div     => "÷",
        Op::Enter   => "ENTER",
        Op::Clx     => "CLX",
        Op::Sin     => "SIN",
        Op::Cos     => "COS",
        Op::Tan     => "TAN",
        Op::Ln      => "LN",
        Op::Log     => "LOG",
        Op::Exp     => "e^x",
        Op::TenPow  => "10^x",
        Op::Sqrt    => "√x",
        Op::Recip   => "1/x",
        Op::Sq      => "x²",
        Op::YPow    => "Y^X",
        Op::Rtn     => "RTN",
        Op::PrgmMode => "PRGM",
        Op::Lbl(s)  => "LBL",   // Note: label name lost in &'static str; format separately if needed
        // ... all remaining variants
        _           => "???",
    }
}
```

**Note:** `op_display_name` should return `String` (not `&'static str`) for `Op::Lbl(s)`, `Op::Gto(s)`, `Op::Xeq(s)` to include the label name. Adjust the return type accordingly.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Block::title(Title::from("text"))` | `Block::bordered().title_top("text")` | ratatui 0.30.0 | Title struct removed; compile error if using old API |
| `Terminal::new(CrosstermBackend(stdout))` | `ratatui::init()` | ratatui 0.26+ | init() is now the standard; installs panic hook automatically |
| `ratatui::run()` wrapper | `ratatui::init()` + manual event loop | ratatui 0.30 new | run() is simpler but does not support the poll(16ms) pattern required by D-04 — use init() directly |
| tui-rs crate | ratatui crate | 2022 (tui-rs archived) | tui-rs is unmaintained; ratatui is the active fork |
| `Flex::SpaceAround` | `Flex::SpaceEvenly` | ratatui 0.30.0 | Renamed to match CSS flexbox terminology |

**Deprecated/outdated:**
- `block::Title` struct: removed in 0.30; code using it will not compile
- `ratatui::Terminal::new()` for basic use: still works but `ratatui::init()` is now idiomatic
- `event::read()` for event loop: still works but poll() is required for non-blocking design (D-04)

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `KeyCode::Char('S')` (uppercase) is emitted by crossterm when user presses Shift+s, without needing to check `KeyModifiers::SHIFT` explicitly | Pattern 7: Key Mapping | If wrong: shift+s might emit `KeyCode::Char('s')` + `KeyModifiers::SHIFT` instead; fix is to also match `(KeyCode::Char('s'), modifiers) if modifiers.contains(KeyModifiers::SHIFT)` |
| A2 | `op_display_name` for PRGM mode step display is complete enough for Phase 4 with a catch-all `"???"` | Code Examples | If wrong: some op variants show `???` in PRGM display; cosmetic issue only, not functional |
| A3 | 16ms poll interval is sufficient for ≤50ms key latency on all target platforms (QUAL-02 target) | Event Loop pattern | If wrong: increase poll frequency; latency is a Phase 7 concern |

---

## Open Questions

1. **`KeyCode::Char('S')` vs `KeyCode::Char('s') + SHIFT modifier`**
   - What we know: crossterm docs say Char variant contains the character; Shift+s *usually* produces uppercase 'S' in the Char value on most terminals
   - What's unclear: some terminal emulators may send lowercase with SHIFT modifier instead of uppercase char
   - Recommendation: Match both `KeyCode::Char('S')` AND `(KeyCode::Char('s'), m) if m.contains(KeyModifiers::SHIFT)` for defensive coverage. The CONTEXT.md decision (uppercase = Shift) is correct for standard terminals.

2. **clap args for Phase 4**
   - What we know: clap 4.x with derive feature is required (CLAUDE.md)
   - What's unclear: which CLI arguments hp41 needs at this phase (file to load? no-ui mode?)
   - Recommendation: Minimal `--help` only in Phase 4; no functional args until Phase 5 adds file persistence

3. **Minimum terminal size enforcement**
   - What we know: D-01 specifies minimum 80x24; resize handling is deferred
   - What's unclear: exact error message display strategy when terminal < 80x24
   - Recommendation: On each draw call, check `frame.area()` dimensions; if too small, render a single "Terminal too small (need 80x24)" Paragraph and skip normal rendering

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build | Yes | 1.89.0 | — |
| just | Build recipes | Yes | 1.49.0 | — |
| ratatui 0.30 | TUI rendering | Not yet in Cargo.toml | 0.30.0 on crates.io | — |
| crossterm 0.29 | Key events, terminal | Not yet in Cargo.toml | 0.29.0 on crates.io | — |
| clap 4.x | CLI args | Not yet in Cargo.toml | 4.x on crates.io | — |

**Missing dependencies:** All three (ratatui, crossterm, clap) must be added to `hp41-cli/Cargo.toml`. This is Wave 0 work — no fallback needed, they are crates.io packages with no system install requirement.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + cargo test |
| Config file | None (workspace Cargo.toml) |
| Quick run command | `just test` |
| Full suite command | `just ci` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DISP-01 | Annunciators render correctly in all states | Manual TUI | `just run` + visual inspection | N/A — TUI |
| DISP-02 | Stack panel shows X/Y/Z/T, LASTX at all times | Manual TUI | `just run` + visual inspection | N/A — TUI |
| INPUT-01 | All ops reachable via keyboard | Manual TUI | `just run` + keystroke test | N/A — TUI |
| Keys unit | key_to_op() returns correct Op for each key | Unit (automated) | `cargo test -p hp41-cli -- keys` | Wave 0 gap |
| PRGM display | format_step() produces correct "003 SIN" format | Unit (automated) | `cargo test -p hp41-cli -- prgm_display` | Wave 0 gap |
| Digit entry | entry_buf append + flush produces correct stack value | Integration | `cargo test -p hp41-cli -- entry` | Wave 0 gap |

### Sampling Rate
- **Per task commit:** `just test` (all workspace tests including hp41-core)
- **Per wave merge:** `just ci` (lint + test + coverage gate)
- **Phase gate:** Full suite green + manual TUI inspection before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `hp41-cli/src/tests/keys_tests.rs` — unit tests for key_to_op() mapping table completeness
- [ ] `hp41-cli/src/tests/prgm_display_tests.rs` — unit tests for format_step()
- [ ] `hp41-cli/src/tests/entry_buf_tests.rs` — integration tests for digit accumulation flow

*(TUI rendering tests — DISP-01, DISP-02, INPUT-01 — are manual-only: terminal rendering cannot be automatically verified in a CI pipeline without a virtual terminal harness. This is acceptable for Phase 4.)*

---

## Security Domain

> `security_enforcement` is not set in config.json — treated as enabled per convention.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | TUI has no user authentication |
| V3 Session Management | No | No sessions — local-only tool |
| V4 Access Control | No | Single-user local CLI |
| V5 Input Validation | Yes (LOW risk) | All key input routed through key_to_op() which returns Option<Op>; only known Op variants reach dispatch() |
| V6 Cryptography | No | No cryptographic operations in Phase 4 |

### Known Threat Patterns for CLI TUI

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed entry_buf causes panic | Tampering | flush_entry_buf() uses Decimal::from_str with map_err — returns HpError::InvalidOp, not panic; already implemented |
| Terminal left in raw mode on crash | Elevation of Privilege | ratatui::init() panic hook + ratatui::restore() in main() |

**Security assessment:** Phase 4 is a local-only single-user calculator TUI. The attack surface is the keyboard — all input is filtered through the key_to_op() type-safe dispatch. No network, no file I/O (Phase 5), no secrets. Risk is minimal.

---

## Sources

### Primary (HIGH confidence)
- [docs.rs/ratatui/0.30.0/ratatui/fn.init.html](https://docs.rs/ratatui/latest/ratatui/fn.init.html) — init() return type, signature, panic hook behavior
- [docs.rs/ratatui/0.30.0/ratatui/struct.Terminal.html](https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html) — Terminal::draw() signature
- [docs.rs/ratatui/0.30.0/ratatui/layout/struct.Layout.html](https://docs.rs/ratatui/latest/ratatui/layout/struct.Layout.html) — Layout::horizontal/vertical, Constraint types, areas()
- [docs.rs/ratatui/0.30.0/ratatui/widgets/struct.Paragraph.html](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Paragraph.html) — Paragraph API, Text/Line/Span hierarchy
- [docs.rs/ratatui/0.30.0/ratatui/struct.Frame.html](https://docs.rs/ratatui/latest/ratatui/struct.Frame.html) — frame.area(), render_widget(), render_stateful_widget()
- [docs.rs/ratatui/0.30.0/ratatui/widgets/struct.Block.html](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Block.html) — Block API, title_top() change in 0.30
- [docs.rs/crossterm/0.29.0/crossterm/event/struct.KeyEvent.html](https://docs.rs/crossterm/latest/crossterm/event/struct.KeyEvent.html) — KeyEvent fields, KeyEventKind, KeyCode::Char, KeyCode::F(n)
- [docs.rs/crossterm/0.29.0/crossterm/event/index.html](https://docs.rs/crossterm/latest/crossterm/event/index.html) — event::poll(), event::read()
- [ratatui.rs/highlights/v030](https://ratatui.rs/highlights/v030/) — breaking changes in 0.30 (Block::title removed, MSRV 1.86)
- cargo search ratatui → 0.30.0 [VERIFIED: 2026-05-07]
- cargo search crossterm → 0.29.0 [VERIFIED: 2026-05-07]
- hp41-core/src/ops/mod.rs — Op enum, dispatch(), flush_entry_buf() [VERIFIED: direct read]
- hp41-core/src/state.rs — CalcState fields, AngleMode, DisplayMode [VERIFIED: direct read]
- hp41-core/src/format.rs — format_hpnum(), format_alpha() signatures [VERIFIED: direct read]
- hp41-core/src/ops/program.rs — run_program(state, label) signature [VERIFIED: direct read]
- hp41-core/src/lib.rs — public exports [VERIFIED: direct read]

### Secondary (MEDIUM confidence)
- [ratatui.rs/tutorials/hello-ratatui](https://ratatui.rs/tutorials/hello-ratatui/) — event loop pattern, draw closure structure
- [rust.code-maven.com ratatui structured state](https://rust.code-maven.com/other/rust/ratatui/ratatui-a-more-structured-way-to-handle-state) — App owns state, terminal.draw(|f| self.draw(f)) borrow pattern
- [github.com/ratatui/ratatui/issues/347](https://github.com/ratatui/ratatui/issues/347) — Windows duplicate key event confirmation
- [ratatui.rs/concepts/widgets](https://ratatui.rs/concepts/widgets/) — Widget trait, render method signature

### Tertiary (LOW confidence)
- None — all critical claims verified via primary sources.

---

## Metadata

**Confidence breakdown:**
- Standard stack (versions): HIGH — verified via cargo search registry, 2026-05-07
- ratatui 0.30 API (init, draw, Layout, Block, Paragraph): HIGH — verified via docs.rs
- crossterm 0.29 KeyEvent API: HIGH — verified via docs.rs
- Borrow checker patterns: HIGH — verified via official ratatui tutorial and community discussion
- Breaking change (Block::title removed): HIGH — verified via ratatui 0.30 release notes
- Key mapping completeness: MEDIUM — all Op variants listed; uppercase-via-Shift assumption logged as A1

**Research date:** 2026-05-07
**Valid until:** 2026-08-07 (ratatui releases every ~3 months; 0.31 may introduce further changes)
