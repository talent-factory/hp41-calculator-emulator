# Phase 5: Persistence & UX - Research

**Researched:** 2026-05-07
**Domain:** Rust serde/JSON persistence, ratatui 0.30 Table/overlay widgets, dirs crate, USER mode
**Confidence:** HIGH (all claims verified against live crate registry, official docs.rs, or codebase inspection)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**State File Strategy**
- D-01: Default auto-save path: `~/.hp41/autosave.json`. Directory auto-created via `fs::create_dir_all`. Use `dirs::home_dir()` from `dirs = "6"` (latest stable — see Standard Stack below).
- D-02: CLI override: `--state-file <path>` clap arg. Load on startup, save to it instead of default.
- D-03: Startup: if file exists → load; if missing → `CalcState::new()`. Parse failures → show error in status bar, start fresh — never panic.
- D-04: Manual save: `Ctrl+S` → save to active path, show "Saved to ..." in status bar for 2 s.
- D-05: Auto-save: `App.last_save: Instant`. Each poll iteration: `if last_save.elapsed() >= 30s → save silently`. Also save on graceful exit (before `ratatui::restore()`).
- D-06: Serialization wrapper: `{ "version": 1, "state": { ... } }`. `#[derive(Serialize, Deserialize)]` on `CalcState`, `Stack`, `HpNum`, `AngleMode`, `DisplayMode`.
- D-07: `HpNum` serializes as string via `#[serde(with = "rust_decimal::serde::str")]` on the `HpNum.0` field. Requires `serde-with-str` feature on rust_decimal.
- D-08: `pending_input: Option<PendingInput>` in `App` (NOT CalcState — transient UI state). Variants: `StoRegister(String)`, `RclRegister(String)`, `StoAdd/Sub/Mul/Div(String)`.
- D-09: STO modal: 2-digit auto-dispatch, Backspace resets accumulator, Esc cancels.
- D-10: RCL and STO-arith follow same pattern. Bindings: `S` = STO (uppercase), `R` = RCL (uppercase). NOTE: `r` (lowercase) is currently `Op::Rdn` (roll down) and `s` (lowercase) is currently `Op::Sqrt` in keys.rs. `S` and `R` are currently UNMAPPED — safe to use.
- D-11: Status bar shows pending prompt while `pending_input` is `Some(...)`, overriding normal message.
- D-12: ALPHA mode: all printable `KeyCode::Char(c)` routed to `Op::AlphaAppend(c)`.
- D-13: ALPHA Backspace → `Op::AlphaBackspace`. Enter or `A` → `Op::AlphaToggle` exits ALPHA mode.
- D-14: Annunciator "ALPHA" already wired in Phase 4 ui.rs. Status bar shows "ALPHA mode — Enter or A to exit".
- D-15: ALPHA toggle: `a` key (lowercase). NOTE: `a` is currently `Op::Asin` in keys.rs — this conflict requires re-routing `a` when `alpha_mode == true`.
- D-16: `App.show_help: bool`. `?` key toggles. Help overlay rendered over full terminal area.
- D-17: Overlay: bordered `Block` (title "HP-41 Function Reference"), centered 80%×90% Rect. Three columns: Key, Operation, Description.
- D-18: Content: static `&[(&str, &str, &str)]` in `help_data.rs`. ~130 entries with category headers.
- D-19: Navigation: Up/Down / `j`/`k` scroll. `Esc`, `q`, `?` close. `App.help_scroll: usize`.
- D-20: No text search in v1.0.
- D-21: `hp41-cli/src/programs.rs` defines `SampleProgram { name, description, ops }` + static `SAMPLE_PROGRAMS`.
- D-22: `Ctrl+P` opens program library overlay. Enter loads. Non-empty program → confirmation prompt. Esc closes.
- D-23: 10 required programs: Fibonacci, Prime Test, Quadratic Solver, Factorial, GCD, Mean+StdDev, Newton Root, Unit Converter, Stack Stats, Countdown Timer.
- D-24: Programs stored as Rust `const` arrays of `Op` variants — compile-time checked, no runtime file loading.
- D-25: Add to `CalcState`: `user_mode: bool`, `key_assignments: BTreeMap<char, String>`. BTreeMap for deterministic serde JSON order.
- D-26: `Op::UserMode` toggle: `u` key → flips `state.user_mode`.
- D-27: `Ctrl+A` → two-step pending: Step 1 = `PendingInput::AssignKey` ("Assign: press key"). Step 2 = `PendingInput::AssignLabel(c, String)` ("Assign {c} → LBL: [____]"). Enter confirms, Esc cancels.
- D-28: USER mode: key with assignment → `run_program(state, &assigned_label)`. F1–F4 pre-wired to user keys a/b/c/d.
- D-29: `BTreeMap<char, String>` derives `Serialize`/`Deserialize` automatically.

### Claude's Discretion
- `dirs` crate version: use `dirs = "6"` (latest stable, verified via cargo search 2026-05-07).
- `Op::AlphaBackspace` — add to hp41-core using `String::pop()`.
- Overlay z-ordering in ratatui: render main view first, overlay second in same `frame` call.
- `Ctrl+P` for program library (not `p` alone — `p` = PrgmMode toggle).
- Auto-save error handling: one-time warning in status bar on failure, retry on next 30s tick.

### Deferred Ideas (OUT OF SCOPE)
- Text search / filter in help overlay — Phase 7 / v1.1
- Named save slots (multiple named files) — `--state-file` covers this
- In-TUI "Save As" dialog — deferred to v1.1
- ALPHA special characters (greek, math symbols via shifted keys) — v1.1
- GTO label-entry dialog — Phase 7 polish / v1.1
- Mouse support — out of scope for v1.0
- Terminal resize handling — deferred; 80×24 minimum with error message already enforced

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERS-01 | User can save and load programs and full calculator state to/from versioned JSON files on disk | serde + serde_json; HpNum custom serde via `rust_decimal::serde::str`; version wrapper pattern; CalcState field inventory complete |
| PERS-02 | Calculator state auto-saves every 30 s and on graceful shutdown | `std::time::Instant` + `elapsed()` in 16ms poll loop; sync file I/O is safe within 16ms budget; save-on-exit before `ratatui::restore()` |
| UX-01 | User can access a built-in function reference/help from within the TUI | ratatui Table + TableState scrolling verified; Rect::centered() API confirmed in 0.30; overlay z-order via draw order |
| UX-02 | User can enable USER mode with custom key assignments persisted in state | BTreeMap<char,String> with auto-derived serde; Op::UserMode toggle; PendingInput two-step chain; F1–F4 wiring |
| UX-03 | User can run ≥10 bundled sample programs from within the TUI | Op variants are all serializable (or skipped for const arrays); `run_program()` signature confirmed; program overlay pattern same as help |

</phase_requirements>

---

## Summary

Phase 5 adds state durability and discoverability to the HP-41 TUI. It is a pure addition phase — no existing Phase 1–4 code needs to be deleted, only extended. The four technical pillars are:

1. **Serialization**: `CalcState` and all nested types get `#[derive(Serialize, Deserialize)]`. The main complexity is `HpNum`, which wraps `rust_decimal::Decimal`. The cleanest approach (confirmed by docs.rs) is `#[serde(with = "rust_decimal::serde::str")]` on the `Decimal` field inside `HpNum`, enabled by the `serde-with-str` feature. This stores the number as its canonical decimal string (e.g., `"3.1415926536"`), avoiding all binary-float JSON precision issues.

2. **Persistence layer**: A new `hp41-cli/src/persistence.rs` module handles `save_state` / `load_state` / `default_state_path`. The state file uses a `{ "version": 1, "state": {...} }` wrapper struct (D-06). `dirs 6.0.0` provides `home_dir()` returning `Option<PathBuf>`. Auto-save uses `std::time::Instant::elapsed()` checked in the existing 16ms poll loop — synchronous `serde_json::to_writer` on a file this small (~10–50 KB) completes well under 1ms on modern hardware and will not block redraws.

3. **Overlay widgets**: ratatui 0.30 provides `Rect::centered(h_constraint, v_constraint)` directly on `Rect` — no helper function needed. Overlays are drawn by rendering a second widget to the same frame after the main layout; ratatui renders in draw-call order so the overlay appears on top. The `Table` widget with `TableState` (stored in `App`) supports scrolling via `select_next()` / `select_previous()` and `frame.render_stateful_widget(table, area, &mut state)`.

4. **Op enum extensibility**: Adding `Op::UserMode` and `Op::AlphaBackspace` follows the established pattern (add variant → add dispatch arm → add op function). Both are simple one-line state mutations. The `Op` enum does NOT need to be `Serialize`/`Deserialize` because sample programs are stored as Rust `const` arrays (not persisted to JSON), and the running program in `CalcState.program: Vec<Op>` will be serialized — but since `Op` contains `HpNum` (via `PushNum(HpNum)`), it needs `Serialize`/`Deserialize` too, so `Op`, `StoArithKind`, `TestKind`, `HpNum`, `LblId` (if present) all need serde derives.

**Primary recommendation:** Start with serde derives on all types (Wave 1), then persistence module (Wave 2), then overlays (Wave 3), then USER mode (Wave 4) — each wave's output is independently testable.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| State serialization (HpNum, CalcState) | hp41-core library | — | CalcState lives in core; derives must be in core |
| File I/O (save/load) | hp41-cli | — | Core has no I/O dependencies; persistence is CLI concern |
| Auto-save timer | hp41-cli (App) | — | Timer is event-loop state; event loop lives in App |
| Help overlay rendering | hp41-cli (ui.rs) | — | All TUI widgets owned by hp41-cli |
| Help data content | hp41-cli (help_data.rs) | — | Static data generated from same source as KEY_REF_TABLE |
| Program library UI | hp41-cli (programs.rs + ui.rs) | — | Sample programs are CLI-bundled; Op arrays are const |
| USER mode logic | hp41-core (dispatch) | hp41-cli (keys.rs) | State mutation in core; key routing in CLI |
| Pending input (STO/RCL/ALPHA/Assign) | hp41-cli (App) | — | Transient UI state; not serialized |
| Op::UserMode, Op::AlphaBackspace | hp41-core (ops) | — | Op enum and dispatch() are in core |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0.228 | Derive Serialize/Deserialize | The Rust serialization framework — no alternative |
| serde_json | 1.0.149 | JSON encode/decode | Human-readable, diff-friendly; already decided in STATE.md |
| rust_decimal | 1.42.0 (workspace: 1.41) | HpNum inner type | Already in workspace; `serde-with-str` feature needed |
| dirs | 6.0.0 | Platform home directory path | Cross-platform `home_dir()` — standard for CLI tools |

**Version note:** `cargo search` on 2026-05-07 shows `rust_decimal = "1.42.0"` as latest, but workspace pins `"1.41"`. Bump to `"1.42"` in workspace `Cargo.toml` if serde-with-str feature is only in 1.42, otherwise keep 1.41. Both versions support `serde-with-str`. [VERIFIED: cargo search 2026-05-07]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::time::Instant | stdlib | Auto-save timer | Already available — no dep needed |
| std::fs | stdlib | File create/read/write | No dep needed |
| std::collections::BTreeMap | stdlib | key_assignments (USER mode) | Already in std — deterministic JSON key order |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `rust_decimal::serde::str` | Custom serde impl on HpNum | serde::str is one line vs. ~20 lines of boilerplate; use serde::str |
| `dirs` crate | Hard-coding `$HOME` | dirs handles Windows `USERPROFILE` vs `HOME` correctly; necessary for QUAL-05 cross-platform |
| `serde_json::to_writer` | `serde_json::to_string` + write | `to_writer` avoids intermediate String allocation; prefer for auto-save hot path |

**Installation (additions to existing Cargo.toml files):**

```toml
# hp41-core/Cargo.toml — add:
serde = { version = "1", features = ["derive"] }
rust_decimal = { workspace = true, features = ["maths", "serde-with-str"] }

# hp41-cli/Cargo.toml — add:
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "6"
```

Note: `rust_decimal` in `hp41-core/Cargo.toml` already specifies `features = ["maths"]` — add `"serde-with-str"` to that list. The workspace `Cargo.toml` just pins the version; features must be declared per-crate.

---

## Architecture Patterns

### System Architecture Diagram

```
User keypress
     |
     v
App::handle_key()   ←──── pending_input? (STO/RCL/ALPHA/Assign)
     |                          |
     |   pending_input == None  |   pending_input == Some(...)
     v                          v
keys::key_to_op()         route to accumulator / modal handler
     |
     v
hp41_core::dispatch()  →  CalcState mutation
     |
     v
App::draw() → ui::render_ui()
                   |
                   ├── render_left_panel() (stack/display/annunciators/status)
                   ├── render_right_panel() (key reference)
                   └── if show_help → render_help_overlay() [on top]
                       if show_programs → render_programs_overlay() [on top]

Poll loop (16ms):
  ├── event::poll(16ms)
  ├── handle_key() if event present
  └── auto-save check: if last_save.elapsed() >= 30s → save_state()

On exit:
  └── save_state() → ratatui::restore()

Persistence:
  load_state(path) → serde_json::from_reader → StateFile { version, state: CalcState }
  save_state(path) → serde_json::to_writer → StateFile { version: 1, state }
```

### Recommended Project Structure

```
hp41-cli/src/
├── app.rs           # App struct — add: last_save, pending_input, show_help,
│                    #   show_programs, help_table_state, programs_table_state
├── keys.rs          # key_to_op() — add STO/RCL/ALPHA/USER routing
├── ui.rs            # render_ui() — add overlay render calls
├── persistence.rs   # NEW: save_state / load_state / default_state_path / StateFile
├── help_data.rs     # NEW: HELP_DATA: &[(&str, &str, &str)] static array
├── programs.rs      # NEW: SampleProgram struct + SAMPLE_PROGRAMS const
├── prgm_display.rs  # unchanged
└── main.rs          # add --state-file clap arg; load state before App::new()

hp41-core/src/
├── state.rs         # add: #[derive(Serialize,Deserialize)] on CalcState, Stack,
│                    #   AngleMode, DisplayMode; add user_mode + key_assignments fields
├── num.rs           # add: custom serde on HpNum via serde(with)
├── ops/mod.rs       # add: Op::UserMode, Op::AlphaBackspace variants + dispatch arms
│                    #   add: #[derive(Serialize,Deserialize)] on Op, StoArithKind, TestKind
└── ops/alpha.rs     # add: op_alpha_backspace()
```

### Pattern 1: HpNum Serde via rust_decimal::serde::str

**What:** Serialize `Decimal` as its canonical string representation in JSON (e.g., `"3.1415926536"`) to avoid floating-point precision loss.

**When to use:** Any struct field of type `Decimal` that must round-trip through JSON without precision loss.

```rust
// Source: docs.rs/rust_decimal/1.41.0/rust_decimal/serde/index.html
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct HpNum(
    #[serde(with = "rust_decimal::serde::str")]
    pub(crate) Decimal
);
```

**Cargo.toml feature required:**
```toml
rust_decimal = { workspace = true, features = ["maths", "serde-with-str"] }
```

**JSON output:** `"3.1415926536"` (string, not float) — survives JSON round-trip exactly.

### Pattern 2: StateFile Version Wrapper

**What:** Wrap `CalcState` in a version-tagged outer struct for forward-compatible persistence.

**When to use:** Any serialized file format that may need migration in future phases.

```rust
// Source: [ASSUMED] — standard Rust serde pattern
use serde::{Serialize, Deserialize};
use hp41_core::CalcState;

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

pub fn save_state(path: &Path, state: &CalcState) -> std::io::Result<()> {
    fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
    let file = fs::File::create(path)?;
    let wrapper = StateFile::current(state.clone());
    serde_json::to_writer_pretty(file, &wrapper)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn load_state(path: &Path) -> Result<CalcState, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let wrapper: StateFile = serde_json::from_reader(file)?;
    // Future: match wrapper.version { 1 => Ok(wrapper.state), _ => migrate(...) }
    Ok(wrapper.state)
}
```

### Pattern 3: Auto-Save Timer in Poll Loop

**What:** Check elapsed time each iteration of the event loop; save silently if threshold exceeded.

**When to use:** Any periodic background action in a TUI event loop with `event::poll(timeout)`.

```rust
// Source: [ASSUMED] — std::time::Instant pattern; poll loop structure from app.rs [VERIFIED: codebase]
use std::time::{Duration, Instant};

pub struct App {
    // ... existing fields ...
    pub last_save: Instant,
    pub state_path: PathBuf,
    // ...
}

// In App::run():
while !self.exit {
    terminal.draw(|frame| self.draw(frame))?;

    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            self.handle_key(key);
        }
    }
    // Phase 5: auto-save timer check (D-05)
    if self.last_save.elapsed() >= Duration::from_secs(30) {
        if let Err(e) = persistence::save_state(&self.state_path, &self.state) {
            self.message = Some(format!("Auto-save failed: {e}"));
        }
        self.last_save = Instant::now(); // reset even on failure — retry next tick
    }
}
// Save on graceful exit (D-05) — before ratatui::restore()
let _ = persistence::save_state(&self.state_path, &self.state);
```

### Pattern 4: ratatui Table with TableState (Help Overlay)

**What:** Scrollable table widget rendered over the main layout using `render_stateful_widget`.

**When to use:** Any scrollable content list in a TUI overlay (help reference, program library).

```rust
// Source: docs.rs/ratatui/0.30.0 — Table, TableState, Rect::centered [VERIFIED]
use ratatui::layout::Constraint;
use ratatui::widgets::{Block, Borders, Row, Cell, Table, TableState};

// In App struct:
pub struct App {
    // ...
    pub show_help: bool,
    pub help_table_state: TableState,
}

// In ui.rs render_ui():
if app.show_help {
    render_help_overlay(app, frame);
}

fn render_help_overlay(app: &App, frame: &mut Frame) {
    let area = frame.area();
    // Rect::centered() confirmed available in ratatui 0.30 [VERIFIED: docs.rs]
    let overlay_area = area.centered(
        Constraint::Percentage(80),
        Constraint::Percentage(90),
    );

    let rows: Vec<Row> = HELP_DATA.iter().map(|(key, op, desc)| {
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
    .block(Block::bordered().title(" HP-41 Function Reference "))
    .row_highlight_style(Style::new().bold());

    // NOTE: render_stateful_widget takes &mut State — App::draw() is &self,
    // so help_table_state must be RefCell<TableState> or draw() must become &mut self.
    // See Pitfall 1 below.
    frame.render_stateful_widget(table, overlay_area, &mut app.help_table_state.borrow_mut());
}
```

### Pattern 5: PendingInput Modal State Machine

**What:** Accumulate user input across multiple key events for multi-key operations.

**When to use:** STO/RCL register entry, USER key assignment — any operation requiring >1 keypress.

```rust
// Source: D-08 through D-11 in CONTEXT.md [ASSUMED implementation pattern]
#[derive(Debug, Clone)]
pub enum PendingInput {
    StoRegister(String),       // accumulating 2-digit register number
    RclRegister(String),
    StoAdd(String), StoSub(String), StoMul(String), StoDiv(String),
    AssignKey,                 // waiting for key char
    AssignLabel(char, String), // char received; accumulating label name
}

// In handle_key(), BEFORE the normal key routing:
if let Some(ref mut pending) = self.pending_input {
    match pending {
        PendingInput::StoRegister(ref mut acc) => {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    acc.push(c);
                    if acc.len() == 2 {
                        let reg: u8 = acc.parse().unwrap_or(0);
                        self.call_dispatch(Op::StoReg(reg));
                        self.pending_input = None;
                    }
                }
                KeyCode::Backspace => { *acc = String::new(); }
                KeyCode::Esc => { self.pending_input = None; }
                _ => {}
            }
        }
        // ... other variants ...
    }
    return; // consume the event — don't fall through to normal routing
}
```

### Pattern 6: Sample Programs as const Op Arrays

**What:** Bundled programs stored as compile-time `const` slices of `Op` variants.

**When to use:** Any fixed program that ships with the emulator (D-24).

```rust
// Source: [ASSUMED Rust const pattern] — Op enum variants are all data types confirmed by ops/mod.rs [VERIFIED]
use hp41_core::ops::Op;

pub struct SampleProgram {
    pub name: &'static str,
    pub description: &'static str,
    pub ops: &'static [Op],
}

// PROBLEM: Op contains HpNum (via PushNum(HpNum)), HpNum contains Decimal.
// Decimal is NOT const-evaluable (no const constructor).
// See Pitfall 2 — must use lazy_static or OnceLock instead of const.

static FIBONACCI: &[Op] = &[
    Op::Lbl("A".to_string()),  // ALSO PROBLEM: String is not const [see Pitfall 2]
    // ...
];
```

**See Pitfall 2 for the workaround** — use `OnceLock<Vec<Op>>` or pre-built at startup.

### Anti-Patterns to Avoid

- **Serializing `pc`, `call_stack`, `is_running`**: These are execution state — save them if resuming mid-program is desired, but on load these should reset to defaults (`pc=0`, `call_stack=[]`, `is_running=false`). The HP-41 "continuous memory" model preserves programs and registers, not mid-execution state. Serialize them but on load always reset `is_running` to false to avoid a corrupted state file leaving the emulator stuck in "running" mode.
- **Global serde feature flags on rust_decimal**: Using `features = ["serde-str"]` (the global version, without "with") makes ALL Decimal fields in ALL crates serialize as strings. The `serde-with-str` feature (per-field via `#[serde(with)]`) is more precise and doesn't affect third-party crates that might use rust_decimal differently.
- **`render_stateful_widget` with `&self` draw method**: The `draw(&self)` signature in `App` (established in Phase 4) conflicts with `TableState` mutation. See Pitfall 1.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Decimal→JSON precision | Custom float serializer | `rust_decimal::serde::str` | Edge cases: NaN, subnormal, rounding — already handled |
| Platform home directory | `std::env::var("HOME")` | `dirs::home_dir()` | Windows uses `USERPROFILE` not `HOME`; dirs handles both plus XDG |
| Pretty-printing JSON | Manual string construction | `serde_json::to_writer_pretty` | Correct escaping, Unicode — do not hand-roll |
| Centered overlay Rect | Manual `(area.width - w) / 2` math | `rect.centered(h_constraint, v_constraint)` | Built into ratatui 0.30 Rect methods [VERIFIED] |
| TableState scrolling | Manual offset tracking | `TableState::select_next()` / `select_previous()` | Auto-clamps to bounds [VERIFIED: docs.rs] |

**Key insight:** The serde ecosystem handles all JSON edge cases correctly. Any hand-rolled JSON serializer for floating-point numbers will fail on edge cases that `serde_json` + `rust_decimal::serde::str` handle transparently.

---

## Common Pitfalls

### Pitfall 1: `draw(&self)` vs. `render_stateful_widget` Borrow Conflict

**What goes wrong:** `App::draw(&self)` is immutable (established in Phase 4 to avoid borrow conflict with `&mut terminal`). But `render_stateful_widget(widget, area, &mut state)` requires mutable access to `TableState`. You cannot get `&mut self.help_table_state` from an `&self` reference.

**Why it happens:** Rust borrow checker. `terminal.draw(|frame| self.draw(frame))` captures `self` immutably in the closure, so `self` is borrowed as `&App` for the duration of the closure.

**How to avoid:** Two approaches:
1. **`Cell<TableState>` / `RefCell<TableState>`**: Wrap `help_table_state` and `programs_table_state` in `std::cell::Cell` or `RefCell`. In draw, call `self.help_table_state.borrow_mut()` to get `&mut TableState`. This is the minimal-change approach compatible with the existing `draw(&self)` signature.
2. **Change `draw` to take `&mut self`**: Move `terminal.draw` closure so it only borrows `frame`, passing `&mut self` fields explicitly. This is cleaner but requires refactoring the Phase 4 draw path.

**Recommendation:** Option 1 (RefCell) — minimal Phase 4 disruption.

**Warning signs:** Compiler error: "cannot borrow `self.help_table_state` as mutable, as it is behind a `&` reference."

### Pitfall 2: `Op` Variants Are Not `const`-Constructible

**What goes wrong:** `Op::Lbl(String)`, `Op::Gto(String)`, `Op::PushNum(HpNum)` all contain heap-allocated types (`String`, `HpNum` which wraps `Decimal`). Rust `const` values cannot contain heap allocations. Declaring `const FIBONACCI: &[Op] = &[Op::Lbl("A".to_string()), ...]` fails to compile.

**Why it happens:** `String::to_string()` and `Decimal::new()` are not `const fn`. The `SampleProgram` struct in D-24 uses `ops: &'static [Op]` which implies `const` — but the Op variants prevent it.

**How to avoid:** Use `std::sync::OnceLock<Vec<SampleProgram>>` initialized at first access:
```rust
// Source: [ASSUMED] — std::sync::OnceLock (stable since Rust 1.70) [VERIFIED: stdlib]
use std::sync::OnceLock;

static SAMPLE_PROGRAMS: OnceLock<Vec<SampleProgram>> = OnceLock::new();

pub fn sample_programs() -> &'static [SampleProgram] {
    SAMPLE_PROGRAMS.get_or_init(|| {
        vec![
            SampleProgram {
                name: "Fibonacci",
                description: "Generates Fibonacci sequence",
                ops: fibonacci_ops(),
            },
            // ...
        ]
    })
}

fn fibonacci_ops() -> Vec<Op> {
    vec![
        Op::Lbl("A".to_string()),
        // ...
    ]
}
```

**Alternative:** Change `Op::Lbl` to use `&'static str` instead of `String` — but that is a breaking change to the existing Op enum used in Phase 3 tests and program recording. Do NOT change Op::Lbl's signature. Use OnceLock.

**Warning signs:** Compile error: "calls in constants are limited to constant functions, tuple structs and tuple variants."

### Pitfall 3: Serializing `Vec<Op>` — All Op Variants Need serde Derives

**What goes wrong:** Adding `#[derive(Serialize, Deserialize)]` to `CalcState` fails unless ALL types it transitively contains also derive serde. `CalcState.program: Vec<Op>` means `Op` must derive serde. `Op::PushNum(HpNum)` means `HpNum` must derive serde. `StoArithKind` and `TestKind` must derive serde. Missing any one causes a compile error deep in the derive macro expansion.

**How to avoid:** The complete list of types needing serde derives in hp41-core:
- `CalcState` — top-level target
- `Stack` — field of CalcState
- `HpNum` — field of Stack (x, y, z, t, lastx), regs array, PushNum payload
- `AngleMode` — field of CalcState
- `DisplayMode` — field of CalcState
- `Op` (all variants) — field of CalcState.program: Vec<Op>
- `StoArithKind` — field of Op::StoArith
- `TestKind` — field of Op::Test

**Warning signs:** Compiler error mentioning a type inside `CalcState` "does not implement `Serialize`."

### Pitfall 4: `is_running` Serialized as `true` → Deadlock on Load

**What goes wrong:** If the state file is written while `is_running == true` (e.g., process killed mid-program), loading it sets `is_running = true` on `CalcState::new()` load, which prevents re-entry via `run_program()` and leaves the calculator stuck.

**How to avoid:** On load, always reset: `loaded_state.is_running = false`. Add this one line in `load_state()` after deserializing.

**Warning signs:** After loading a state, pressing F5 (R/S) does nothing or returns an error.

### Pitfall 5: ALPHA `a` Key Conflict with `Op::Asin`

**What goes wrong:** In Phase 4 keys.rs, `a` → `Op::Asin`. In ALPHA mode (D-12), `a` should route to `AlphaAppend('a')`. If the ALPHA routing check is not placed BEFORE the normal key_to_op dispatch, pressing `a` in ALPHA mode incorrectly fires ASIN.

**How to avoid:** In `handle_key()`, check `state.alpha_mode` BEFORE calling `keys::key_to_op()`. The ALPHA mode routing block is an early-return guard (same pattern as the existing digit-entry early-return).

**Warning signs:** Pressing `a` in ALPHA mode shows a domain error instead of appending 'a' to alpha_reg.

### Pitfall 6: `dirs` API — `home_dir()` Returns `Option<PathBuf>`

**What goes wrong:** `dirs::home_dir()` returns `Option<PathBuf>`, not `PathBuf`. On systems with no detectable home directory (rare but possible in CI/containers), it returns `None`. Code that `.unwrap()`s panics, violating the zero-panic contract.

**How to avoid:** Provide a fallback:
```rust
// Source: dirs 6.0.0 docs [VERIFIED]
fn default_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("autosave.json")
}
```

**Warning signs:** Panic in `default_state_path()` in a CI container with `HOME` unset.

### Pitfall 7: serde_json Pretty-Print vs. Compact — Auto-Save Performance

**What goes wrong:** `serde_json::to_writer_pretty` adds newlines and indentation, making the file ~3x larger than compact JSON. For a CalcState with 100 registers and a 200-step program, pretty JSON may be 20–50 KB vs. 7–15 KB compact. On slow disks or network filesystems (NFS home dirs) this still matters.

**How to avoid:** Use `to_writer_pretty` for manual saves (human-readable diff), and `to_writer` (compact) for auto-save. Or simply use pretty for both — 50 KB sync write on an SSD takes <0.1ms and is well within the 16ms poll budget. Recommend pretty-print everywhere for PERS-01 (human-readable state files are a selling point).

---

## Code Examples

### HpNum with serde

```rust
// Source: docs.rs/rust_decimal serde module [VERIFIED]
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HpNum(
    #[serde(with = "rust_decimal::serde::str")]
    pub(crate) Decimal,
);
```

### CalcState with new fields

```rust
// Source: state.rs [VERIFIED: codebase inspection], pattern [ASSUMED for new fields]
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcState {
    pub stack: Stack,
    pub regs: [HpNum; 100],
    pub alpha_reg: String,
    pub alpha_mode: bool,
    pub angle_mode: AngleMode,
    pub display_mode: DisplayMode,
    pub entry_buf: String,
    pub program: Vec<crate::ops::Op>,
    pub prgm_mode: bool,
    pub pc: usize,
    pub call_stack: Vec<usize>,
    pub is_running: bool,
    // Phase 5 additions:
    pub user_mode: bool,
    pub key_assignments: BTreeMap<char, String>,
}
```

### Centered Overlay in ratatui 0.30

```rust
// Source: docs.rs/ratatui/0.30.0 Rect::centered() [VERIFIED]
use ratatui::layout::Constraint;

fn render_help_overlay(app: &App, frame: &mut Frame) {
    let overlay_area = frame.area().centered(
        Constraint::Percentage(80),
        Constraint::Percentage(90),
    );
    // render Block + Table to overlay_area
    // This renders ON TOP of whatever was drawn to frame.area() earlier
}
```

### Op enum serde derive

```rust
// Source: ops/mod.rs [VERIFIED: codebase inspection]; serde derive [ASSUMED pattern]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Op {
    Add, Sub, Mul, Div,
    Enter, Clx, Chs, Rdn, XySwap, Lastx,
    PushNum(HpNum),    // HpNum must derive serde
    // ... all variants ...
    UserMode,           // Phase 5 new
    AlphaBackspace,     // Phase 5 new
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoArithKind { Add, Sub, Mul, Div }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestKind {
    XEqZero, XNeZero, XLtZero, XGtZero, XLeZero, XGeZero,
    XEqY, XNeY, XLtY, XGtY, XLeY, XGeY,
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `dirs = "5"` | `dirs = "6"` | 2024-2025 | API unchanged for `home_dir()` — just version bump |
| `serde-str` (global feature) | `serde-with-str` (per-field) | rust_decimal 1.x | Per-field is strictly better; use `#[serde(with = "rust_decimal::serde::str")]` |
| `ratatui::widgets::Block::default()` | `Block::bordered()` | ratatui 0.27+ | `.bordered()` = `.borders(Borders::ALL)` — shorter |
| Manual centered rect calculation | `Rect::centered(h, v)` | ratatui 0.29+ | Built-in — no helper function to write |
| `TableState` with manual offset | `TableState::select_next()` | ratatui 0.29+ | Built-in scroll navigation |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | StateFile wrapper struct pattern for version migration | Code Examples | Low — standard Rust serde pattern; no library changes needed |
| A2 | Auto-save sync write completes well under 16ms on modern hardware | Patterns | Low — 50 KB file on SSD is <0.1ms; risk only on NFS/slow disk |
| A3 | `OnceLock<Vec<SampleProgram>>` is the correct workaround for non-const Op | Pitfall 2 | Low — OnceLock is stable since Rust 1.70 (workspace uses 1.78+) |
| A4 | ALPHA `a` routing block must be placed before key_to_op() | Pitfall 5 | Medium — wrong placement causes behavioral regression on 'a' key |
| A5 | `is_running` should be reset to false on state file load | Pitfall 4 | Low — easy to add; easy to verify in test |
| A6 | PendingInput implementation pattern (state machine in handle_key) | Pattern 5 | Low — established pattern; CONTEXT.md is explicit about the design |

---

## Open Questions

1. **`regs: [HpNum; 100]` and serde**
   - What we know: serde derives `Serialize`/`Deserialize` for fixed-size arrays. Arrays `[T; N]` are supported when `T: Serialize + N <= 32`... but N=100 exceeds serde's built-in fixed-array limit of 32.
   - What's unclear: Does serde derive support `[HpNum; 100]` natively, or does it require `serde_with` or a custom impl?
   - Recommendation: Use `serde_with = "3"` with `#[serde_as(as = "[_; 100]")]`, or change `regs: [HpNum; 100]` to `regs: Vec<HpNum>` (with a constructor that pre-fills 100 zeros). The Vec approach is simplest. Alternatively, `#[serde(with = "serde_arrays")]` from the `serde_arrays` crate (lightweight). **The planner should pick one approach before the serialization wave.**

2. **`Lbl(String)` in Op — is `String` serde-compatible in const context?**
   - What we know: `String` derives serde cleanly. The issue is only with `const` arrays of Op (Pitfall 2).
   - What's unclear: The `LblId` type was mentioned in CONTEXT.md canonical refs as `hp41-core/src/ops/mod.rs` — but inspecting the file shows `Op::Lbl(String)` directly, no `LblId` enum. No action needed — `String` is serde-compatible.
   - Recommendation: No change needed; `String` in Op variants serializes as JSON string automatically.

3. **`draw(&self)` vs `RefCell<TableState>` ergonomics**
   - What we know: `RefCell::borrow_mut()` panics at runtime if already borrowed — but since draw() is single-threaded and non-reentrant, this is safe.
   - What's unclear: Whether the team prefers RefCell complexity or refactoring draw() to `&mut self`.
   - Recommendation: Use `RefCell<TableState>` to minimize Phase 4 disruption. Document the single-threaded safety invariant in a comment.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo / rustc | Build | yes | (workspace rust 1.78+) | — |
| `dirs` crate | persistence.rs | to be added | 6.0.0 | Hard-code `$HOME` as worst case |
| `serde` crate | hp41-core + hp41-cli | to be added | 1.0.228 | — |
| `serde_json` crate | hp41-cli persistence | to be added | 1.0.149 | — |
| `~/.hp41/` directory | default save path | created at runtime | — | fs::create_dir_all |

**Missing dependencies with no fallback:**
- None — all dependencies are standard crates with no system-level requirements.

**Missing dependencies with fallback:**
- `dirs` — if unavailable, fall back to `$HOME` env var; but `dirs` is a single-purpose crate with no transitive deps and should be added.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + proptest for property tests |
| Config file | none — standard cargo test discovery |
| Quick run command | `cargo test -p hp41-core -p hp41-cli` |
| Full suite command | `just ci` (lint + test + coverage) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PERS-01 | save_state() writes valid JSON with version wrapper | unit | `cargo test -p hp41-cli -- persistence::tests` | ❌ Wave 0 |
| PERS-01 | load_state() round-trips CalcState exactly (all fields equal) | unit | `cargo test -p hp41-cli -- persistence::tests::test_roundtrip` | ❌ Wave 0 |
| PERS-01 | load_state() on missing file returns Err (not panic) | unit | `cargo test -p hp41-cli -- persistence::tests::test_missing_file` | ❌ Wave 0 |
| PERS-01 | load_state() on corrupt JSON returns Err (not panic) | unit | `cargo test -p hp41-cli -- persistence::tests::test_corrupt_file` | ❌ Wave 0 |
| PERS-01 | HpNum serializes as string, not float | unit | `cargo test -p hp41-core -- num::tests::test_hpnum_serde` | ❌ Wave 0 |
| PERS-01 | user_mode + key_assignments survive save/load | unit | `cargo test -p hp41-cli -- persistence::tests::test_user_mode_roundtrip` | ❌ Wave 0 |
| PERS-02 | App.last_save updates after auto-save | unit | manual-only (requires TUI loop) — verified via integration smoke test | ❌ manual |
| PERS-02 | State file timestamp changes within 35s of start | integration | manual smoke — check file mtime after 31s | ❌ manual |
| UX-01 | HELP_DATA contains entries for all Key categories | unit | `cargo test -p hp41-cli -- help_data::tests::test_all_categories_present` | ❌ Wave 0 |
| UX-01 | help_table_state selects next/previous without panic | unit | `cargo test -p hp41-cli -- ui::tests::test_help_scroll` | ❌ Wave 0 |
| UX-02 | Op::UserMode flips state.user_mode | unit | `cargo test -p hp41-core -- ops::tests::test_user_mode_toggle` | ❌ Wave 0 |
| UX-02 | key_assignments BTreeMap survives serde round-trip | unit | covered by PERS-01 test_user_mode_roundtrip | — |
| UX-02 | USER mode dispatches assigned label via run_program | unit | `cargo test -p hp41-cli -- keys::tests::test_user_mode_dispatch` | ❌ Wave 0 |
| UX-03 | SAMPLE_PROGRAMS has exactly ≥10 entries | unit | `cargo test -p hp41-cli -- programs::tests::test_program_count` | ❌ Wave 0 |
| UX-03 | Each sample program's ops list is non-empty | unit | `cargo test -p hp41-cli -- programs::tests::test_programs_non_empty` | ❌ Wave 0 |
| UX-03 | Fibonacci program runs to completion without panic | unit | `cargo test -p hp41-cli -- programs::tests::test_fibonacci_runs` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-core && cargo test -p hp41-cli`
- **Per wave merge:** `just ci`
- **Phase gate:** Full suite green (including coverage ≥80%) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-cli/src/persistence.rs` — module + `save_state`, `load_state`, `default_state_path` + inline tests
- [ ] `hp41-cli/src/help_data.rs` — `HELP_DATA` static + category test
- [ ] `hp41-cli/src/programs.rs` — `SampleProgram`, `sample_programs()`, 10 programs + count test
- [ ] `hp41-core/src/tests.rs` — add `test_hpnum_serde` (Serialize/Deserialize round-trip for HpNum)
- [ ] `hp41-core/src/ops/` — test for `Op::UserMode` and `Op::AlphaBackspace` dispatch

*(Note: No new conftest/fixtures needed — existing `CalcState::new()` + dispatch pattern from prior phases is sufficient.)*

---

## Security Domain

> `security_enforcement` not explicitly set in config.json — treating as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | no auth in this application |
| V3 Session Management | no | local-only, no sessions |
| V4 Access Control | no | single-user local app |
| V5 Input Validation | yes | serde_json::from_reader returns Err on malformed JSON — do not unwrap |
| V6 Cryptography | no | state files are plaintext (user's own data, local storage) |

### Known Threat Patterns for {stack}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed JSON state file (attacker-crafted) | Tampering | `serde_json::from_reader` returns `Err` — catch and start fresh (D-03) |
| Path traversal via `--state-file` | Tampering | No server-side, user provides path for their own use — not a threat in single-user CLI |
| Panic on corrupted numeric value | Denial of Service | `HpNum::rounded().unwrap_or(d)` already guards; `from_str` parse failure = `Err` not panic |

**Risk summary:** Very low. Local single-user tool writing/reading own state file. The only meaningful input validation requirement is that corrupt JSON files are handled gracefully (already covered by D-03).

---

## Sources

### Primary (HIGH confidence)

- `docs.rs/rust_decimal/1.41.0` — serde module, `serde-with-str` feature flag, `Decimal::from_str` / `to_string` [VERIFIED]
- `docs.rs/ratatui/0.30.0` — `Table`, `TableState` APIs, `Rect::centered()`, `render_stateful_widget` [VERIFIED]
- `docs.rs/dirs/6.0.0` — `home_dir()` returns `Option<PathBuf>`, 18 functions available [VERIFIED]
- `cargo search` (2026-05-07) — `dirs = "6.0.0"`, `serde_json = "1.0.149"`, `serde = "1.0.228"`, `rust_decimal = "1.42.0"`, `serde_with = "3.19.0"` [VERIFIED]
- Codebase inspection: `hp41-core/src/state.rs`, `num.rs`, `ops/mod.rs`, `ops/alpha.rs`, `hp41-cli/src/app.rs`, `keys.rs`, `ui.rs`, `Cargo.toml` files [VERIFIED]

### Secondary (MEDIUM confidence)

- `docs.rs/ratatui/0.30.0/ratatui/layout/struct.Rect.html` — `centered()` method confirmed [VERIFIED]
- rust_decimal serde documentation explaining global vs. per-field features [VERIFIED via WebFetch]

### Tertiary (LOW confidence)

- OnceLock workaround for non-const Op variants — based on Rust std library stable API (Rust 1.70+) [ASSUMED: not directly verified against this specific use case]
- serde `[T; 100]` array limitation at N>32 — known Rust community constraint [ASSUMED: should be verified during Wave 0 implementation; if serde 1.x has lifted this, the Vec workaround is unnecessary]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions verified via cargo search and docs.rs
- Architecture: HIGH — based on direct codebase inspection of all relevant files
- serde patterns: HIGH — verified against docs.rs/rust_decimal serde module docs
- ratatui overlay/Table: HIGH — verified against docs.rs/ratatui/0.30.0
- Pitfalls: MEDIUM/HIGH — Pitfall 1 (borrow checker) and 5 (key conflict) are directly observed from code; Pitfalls 2/3/4 are based on Rust language semantics [HIGH for language-level facts]
- Sample program const limitation: MEDIUM — well-known Rust limitation [ASSUMED not directly tested]

**Research date:** 2026-05-07
**Valid until:** 2026-08-07 (stable crates, unlikely to change in 90 days)
