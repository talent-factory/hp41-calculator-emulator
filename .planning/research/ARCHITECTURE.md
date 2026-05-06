# Architecture Patterns: HP-41 Calculator Emulator

**Domain:** Behavioral calculator emulator with TUI frontend, Rust workspace
**Researched:** 2026-05-06
**Overall confidence:** HIGH (core patterns), MEDIUM (HP-41 specific behavioral details)

---

## Recommended Architecture

A three-layer architecture enforced by Cargo workspace membership:

```
┌─────────────────────────────────────────────────────┐
│                   hp41-cli (binary)                  │
│  TUI rendering · key mapping · auto-save · help      │
│  Depends on: hp41-core                               │
└────────────────────────┬────────────────────────────┘
                         │ calls core API only
                         ▼
┌─────────────────────────────────────────────────────┐
│                 hp41-core (library)                  │
│  Calculator state · arithmetic · programming engine  │
│  No UI deps. No std I/O. Serializable.               │
└─────────────────────────────────────────────────────┘
                (future)
┌─────────────────────────────────────────────────────┐
│                hp41-gui (Tauri, v2.0)                │
│  Also depends on hp41-core only                      │
└─────────────────────────────────────────────────────┘
```

This mirrors the Free42 architecture (common/ core + per-platform UIs) and the WezTerm pattern
(wezterm-term independent of wezterm-gui). Cargo enforces the dependency direction at compile time:
hp41-core has zero dependency on hp41-cli, verified by `cargo check -p hp41-core` in CI.

---

## Component Boundaries

### hp41-core

| Sub-module | Responsibility | Communicates With |
|---|---|---|
| `state::CalcState` | Authoritative calculator state struct | All other core modules |
| `stack` | X/Y/Z/T registers, LASTX, stack-lift flag | `CalcState` (owns it) |
| `registers` | R00–R99 numeric storage, STO/RCL/STO+−×÷ | `CalcState` |
| `display` | 12-char alphanumeric display string, annunciators | `CalcState` |
| `arithmetic` | Basic ops, trig, log/exp, power; reads/writes stack | `stack`, `display` |
| `math_modes` | FIX/SCI/ENG n, DEG/RAD/GRAD, angle conversion | `CalcState`, `display` |
| `alpha` | ALPHA mode input buffer (12 chars), alpha key maps | `CalcState`, `display` |
| `flags` | 30 user flags (0–29) + system flags (30+) | `CalcState` |
| `program_memory` | Linear byte array of FOCAL instructions, labels index | `CalcState` |
| `engine` | Program counter, return stack (4 levels), step executor | `program_memory`, all ops |
| `conditionals` | IF tests, ISG, DSE — skip-next-step semantics | `engine`, `stack`, `registers` |
| `persistence` | Serde-based save/load of full `CalcState` to JSON | `CalcState` |
| `error` | `HpError` enum, no panics | All modules |

### hp41-cli

| Sub-module | Responsibility | Communicates With |
|---|---|---|
| `tui::app` | `App` struct holding `CalcState` + UI state | `hp41-core` |
| `tui::ui` | Ratatui render functions (view layer, side-effect-free) | `App` (read-only) |
| `tui::events` | Crossterm event polling; translates keys → `Message` | `App` via `update()` |
| `tui::layout` | Widget placement: stack panel, display, keyboard grid | `ui` |
| `keymap` | Physical key → HP-41 key translation table | `events` |
| `autosave` | Tokio timer task; calls `persistence::save` every 30s | `CalcState` (shared ref) |
| `help` | Static built-in function reference rendered in TUI | `ui` |
| `cli` | Clap argument parsing, entry point, `run()` orchestration | All of the above |

---

## Data Flow

### Interactive keystroke (normal mode)

```
Physical key press
  → crossterm KeyEvent (in event task)
  → keymap::translate() → Option<HpKey>
  → Message::KeyPress(HpKey)
  → update(app, msg)
      → hp41-core::CalcState::handle_key(key)
          → stack-lift check
          → dispatch to arithmetic / stack / display / alpha
          → update display string
          → return Ok(())
      → app.state modified in place
  → terminal.draw(|f| ui::render(f, &app))
  → user sees updated TUI
```

### Program execution (PRGM running mode)

```
XEQ / R/S key
  → engine::run_program(&mut calc_state)
      loop:
        fetch instruction at program_counter
        engine::execute_step()
          → dispatch on Instruction enum variant
          → arithmetic / stack / conditionals / control flow
        if conditional skip: advance PC by 2
        if GTO/XEQ: resolve label → set PC
        if RTN / return stack empty: stop
        check for STOP / interactive key interrupt
  → display updated after each step
  → return to idle state
```

### Save/load

```
CalcState (in memory)
  ← serde::Deserialize ← serde_json::from_str ← file on disk
  → serde::Serialize → serde_json::to_string → file on disk
```

All state lives inside `CalcState`. No global mutable state anywhere in hp41-core.

---

## Key Design Decisions

### 1. CalcState as the Single Source of Truth

Model everything the calculator "is" in one owned struct:

```rust
pub struct CalcState {
    pub stack: Stack,           // X, Y, Z, T, LAST_X, stack_lift_enabled
    pub registers: [f64; 100],  // R00–R99
    pub display: Display,       // 12-char string + annunciators bitfield
    pub flags: Flags,           // [bool; 56]  (30 user + 26 system)
    pub alpha_buffer: String,   // current ALPHA mode accumulation
    pub mode: CalcMode,         // Normal / Alpha / Prgm
    pub angle_mode: AngleMode,  // Deg / Rad / Grad
    pub number_format: NumberFormat, // Fix(n) / Sci(n) / Eng(n)
    pub program_memory: ProgramMemory,
    pub user_key_assignments: HashMap<HpKey, String>,
    pub stats_registers: StatsRegisters,  // Σ accumulators
    // Internal engine state (not user-visible, but serialized)
    pub engine: EngineState,    // program_counter, return_stack
}
```

Rationale: Rust's ownership model works best when one struct owns all state. Borrowing rules
prevent split borrows across modules if state is scattered. A single `&mut CalcState` passed to
each operation avoids lifetime gymnastics. (HIGH confidence pattern — aligns with how rscalc
and Free42's common/core_globals handle calculator state.)

### 2. Stack-Lift Semantics as a Boolean Flag

The `stack_lift_enabled` boolean in `Stack` controls whether the next numeric entry lifts the stack:

```rust
pub struct Stack {
    x: f64,
    y: f64,
    z: f64,
    t: f64,
    last_x: f64,
    lift_enabled: bool,  // the "push flag"
}

impl Stack {
    pub fn enter_number(&mut self, value: f64) {
        if self.lift_enabled {
            self.t = self.z;
            self.z = self.y;
            self.y = self.x;
        }
        self.x = value;
        self.lift_enabled = true;
    }

    pub fn enter_key(&mut self) {
        self.y = self.x;
        self.lift_enabled = false;  // ENTER disables lift for next entry
    }

    pub fn binary_op<F: Fn(f64, f64) -> f64>(&mut self, f: F) {
        self.last_x = self.x;
        self.x = f(self.x, self.y);
        self.y = self.z;
        self.z = self.t;
        // T stays (duplicates on pop — HP RPN rule)
        self.lift_enabled = true;
    }
}
```

Stack-lift rules (MEDIUM confidence — verified across multiple HP sources):
- **Disables** lift: ENTER, CLX, CHS (sign change after fresh entry), digit key after result
- **Enables** lift: all arithmetic ops, RCL, functions that produce a result
- **Neutral** (no change to flag): STO, flag ops, display format changes, mode switches

### 3. Instruction Enum for the Programming Engine

Use a closed-world `Instruction` enum, not `dyn Trait`. The HP-41's instruction set is fixed
and known at compile time. Enum dispatch is faster (no vtable) and lets the compiler exhaustively
verify all instructions are handled:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // Entry / stack
    EnterNumber(f64),
    Enter,
    Clx,
    Chs,
    // Arithmetic
    Plus, Minus, Times, Divide, Reciprocal, Sqrt, Square, YpowX,
    // Logs
    Ln, Log, Exp, TenPowX,
    // Trig
    Sin, Cos, Tan, Asin, Acos, Atan,
    // Stack ops
    RollDown, RollUp, SwapXY, LastX,
    // Storage
    Sto(u8), Rcl(u8), StoAdd(u8), StoSub(u8), StoMul(u8), StoDiv(u8),
    // Modes
    Fix(u8), Sci(u8), Eng(u8), Deg, Rad, Grad,
    // Program control
    Lbl(LabelRef), Gto(LabelRef), Xeq(LabelRef), Rtn, End,
    // Conditionals (skip-next-step on FALSE)
    IfXgtY, IfXltY, IfXgeY, IfXleY, IfXeqY, IfXneY,
    IfXgt0, IfXlt0, IfXeq0, IfXne0,
    Isg(u8), Dse(u8),
    IfFlagSet(u8), IfFlagClear(u8),
    // Alpha
    AlphaLiteral(String), AlphaView,
    // Flags
    Sf(u8), Cf(u8),
    // Statistics
    SigmaPlus, SigmaMinus, Mean, Sdev, LinearRegression,
    // HMS
    HmsToHr, HrToHms, HmsPlus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelRef {
    Local(u8),        // 00–99
    Global(String),   // ALPHA label (up to 7 chars)
}
```

### 4. Programming Engine: Return Stack (4 levels)

The HP-41 hardware supports exactly 4 subroutine nesting levels (verified: the Nut CPU return
stack is 4 deep). Model this as a fixed-size array:

```rust
pub struct EngineState {
    pub program_counter: usize,       // index into program_memory.instructions
    pub return_stack: [usize; 4],     // up to 4 nested XEQ/GSB calls
    pub return_stack_depth: u8,       // 0..=4
    pub running: bool,
    pub prgm_mode: bool,
}
```

RTN/END behavior:
- If `return_stack_depth > 0`: pop return address, continue running
- If `return_stack_depth == 0`: stop execution (program complete)
- XEQ when return stack is full (depth == 4): HP-41 shows "TRY AGAIN" error

### 5. Conditional Tests: Skip-Next-Step Model

The HP-41 uses a "skip-if-false" conditional model. When a condition is TRUE, the next step
executes normally. When FALSE, the next step is skipped:

```rust
fn execute_step(state: &mut CalcState) -> Result<StepResult, HpError> {
    let instr = state.program_memory.get(state.engine.program_counter)?;
    state.engine.program_counter += 1;
    match instr {
        Instruction::IfXgtY => {
            if !(state.stack.x > state.stack.y) {
                state.engine.program_counter += 1;  // skip next step
            }
            Ok(StepResult::Continue)
        }
        // ...
    }
}
```

ISG/DSE register format: `±ccccccc.fffii` where `ccccccc` = current value,
`fff` = final/limit value (3 decimal digits), `ii` = increment (2 decimal digits).

### 6. Error Handling: thiserror in Core, anyhow in CLI

```rust
// hp41-core/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum HpError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Square root of negative number")]
    SqrtNegative,
    #[error("Logarithm of non-positive number")]
    LogNonPositive,
    #[error("Overflow: result exceeds 9.999999999e99")]
    Overflow,
    #[error("Underflow")]
    Underflow,
    #[error("Subroutine nesting too deep (max 4)")]
    ReturnStackFull,
    #[error("Label '{0}' not found")]
    LabelNotFound(String),
    #[error("Program counter out of bounds")]
    PcOutOfBounds,
    #[error("Invalid register index {0}: must be 00–99")]
    InvalidRegister(u8),
    #[error("Invalid flag {0}: user flags are 0–29")]
    InvalidFlag(u8),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

No panics in hp41-core (NFR-3). All operations return `Result<T, HpError>`.
hp41-cli uses `anyhow::Result` at the application boundary and converts HpError via `?`.

### 7. TUI Architecture: Elm Architecture (TEA) Pattern

Ratatui recommends the Elm Architecture as the most scalable pattern for complex TUI apps:

```rust
// hp41-cli/src/app.rs
pub struct App {
    pub calc: CalcState,          // the calculator (from hp41-core)
    pub running: bool,
    pub ui_mode: UiMode,          // Normal, Help, PrgmList, ...
    pub prgm_cursor: usize,       // for PRGM mode display
}

// hp41-cli/src/events.rs
pub enum Message {
    KeyPress(HpKey),
    Tick,                         // 30s autosave timer
    Resize(u16, u16),
    Quit,
    SaveState,
    LoadState(PathBuf),
    ShowHelp,
    HideHelp,
}

// update function: pure state transition
fn update(app: &mut App, msg: Message) -> Option<Message> {
    match msg {
        Message::KeyPress(key) => {
            match app.calc.handle_key(key) {
                Ok(_) => None,
                Err(e) => { app.last_error = Some(e.to_string()); None }
            }
        }
        Message::Tick => Some(Message::SaveState),
        Message::SaveState => {
            let _ = persistence::save(&app.calc, &app.state_path);
            None
        }
        // ...
    }
}

// Main event loop
fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    let (event_tx, event_rx) = mpsc::unbounded_channel::<Message>();
    // spawn event task: crossterm events → Message::KeyPress
    // spawn tick task: 30s timer → Message::Tick
    let mut app = App::new(load_or_default_state()?);
    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        let msg = event_rx.recv()?;
        if matches!(msg, Message::Quit) { break; }
        let mut current = Some(msg);
        while let Some(m) = current {
            current = update(&mut app, m);
        }
    }
    Ok(())
}
```

### 8. Display and Annunciators

```rust
pub struct Display {
    pub text: String,            // up to 12 chars (alphanumeric)
    pub annunciators: Annunciators,
}

pub struct Annunciators {
    pub user: bool,   // USER mode active
    pub prgm: bool,   // PRGM mode active
    pub alpha: bool,  // ALPHA mode active
    pub shift: bool,  // shift key pending
    pub rad: bool,    // Radians mode
    pub grad: bool,   // Grads mode (neither = DEG)
    pub run: bool,    // program running
    pub low_bat: bool, // (cosmetic only — always false in emulator)
}
```

The `display.text` string is what hp41-cli renders in the display widget. hp41-core is
responsible for formatting numbers per the current FIX/SCI/ENG setting and writing them into
`display.text` after each operation. The CLI reads this string — no number formatting in CLI.

### 9. Program Memory Layout

```rust
pub struct ProgramMemory {
    pub instructions: Vec<Instruction>,  // linear program store
    pub labels: HashMap<LabelRef, usize>, // pre-indexed label positions
    pub end_positions: Vec<usize>,        // positions of END instructions (program separators)
}
```

Labels are re-indexed on any program edit (insert/delete step). The `end_positions` vector
marks program boundaries for GTO's downward-then-wrap search semantics.

For numeric labels (LBL 00–99): search starts at current PC, scans forward to END, wraps to
start of current program. For global alpha labels: searches all programs.

### 10. Persistence: Versioned JSON

```rust
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,         // schema version; current = 1
    pub saved_at: String,     // ISO 8601
    pub calc: CalcState,
}
```

On load, check `version` field first. Unknown future versions: fail with a clear error message.
Past versions: migration functions keyed on version number. Use `#[serde(default)]` on all new
fields so v1 files load cleanly in v2 without migration code for purely additive changes.

---

## Suggested Build Order

This is the dependency-first order for implementation. Each item can only be built after
all items it depends on are complete. Cargo enforces this at the crate level; within hp41-core,
the module ordering is a team convention.

```
1. hp41-core/error.rs          — HpError; no deps
2. hp41-core/stack.rs          — Stack struct with lift semantics; depends on error
3. hp41-core/registers.rs      — R00–R99 storage; depends on error
4. hp41-core/flags.rs          — user/system flags; depends on error
5. hp41-core/display.rs        — Display + Annunciators; depends on nothing
6. hp41-core/math_modes.rs     — AngleMode, NumberFormat, formatting; depends on display
7. hp41-core/state.rs          — CalcState (aggregates all above); depends on all above
8. hp41-core/arithmetic.rs     — math ops on CalcState; depends on state, error
9. hp41-core/alpha.rs          — ALPHA mode entry; depends on state
10. hp41-core/program_memory.rs — Instruction enum, ProgramMemory; depends on error
11. hp41-core/engine.rs         — step executor, return stack, conditionals; depends on state + program_memory
12. hp41-core/persistence.rs    — save/load; depends on state (serde)
13. hp41-core/lib.rs            — public API surface; re-exports
    ──── hp41-core complete ────
14. hp41-cli/error.rs           — anyhow integration
15. hp41-cli/keymap.rs          — physical key → HpKey; depends on hp41-core public types
16. hp41-cli/app.rs             — App, Message, update(); depends on hp41-core
17. hp41-cli/tui/layout.rs      — widget geometry
18. hp41-cli/tui/ui.rs          — ratatui render functions; depends on app
19. hp41-cli/tui/events.rs      — crossterm event loop, autosave timer; depends on app
20. hp41-cli/help.rs            — static help content
21. hp41-cli/main.rs            — entry point, clap args, run()
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Global Calculator State

**What:** `static mut CALC: Mutex<CalcState>` or thread-local state.
**Why bad:** Makes hp41-core impossible to test in parallel; prevents multiple instances; breaks the core/UI separation contract.
**Instead:** Pass `&mut CalcState` through the call stack. One `CalcState` per `App`.

### Anti-Pattern 2: Number Formatting in the Stack

**What:** Storing formatted strings in X/Y/Z/T instead of f64.
**Why bad:** Arithmetic on formatted strings is wrong; FIX/SCI/ENG changes require re-formatting all registers.
**Instead:** Store raw f64 in stack. Format only when writing `display.text`. Keep formatting in `math_modes.rs::format_number(value: f64, fmt: NumberFormat) -> String`.

### Anti-Pattern 3: dyn Trait for Instructions

**What:** `Box<dyn Executable>` for each instruction stored in program memory.
**Why bad:** Heap allocation per step; no exhaustive match verification; harder to serialize/deserialize; ~10x slower dispatch.
**Instead:** `Instruction` enum with exhaustive `match`. The HP-41 instruction set is closed and fixed.

### Anti-Pattern 4: Parsing FOCAL Syntax at Runtime

**What:** Storing programs as text strings and parsing them during execution.
**Why bad:** Parsing overhead per step; error reporting is harder; editing individual steps requires re-parsing.
**Instead:** Represent programs as `Vec<Instruction>` (already parsed). Keystroke entry adds one `Instruction` at a time.

### Anti-Pattern 5: Blocking the TUI Render Thread

**What:** Running a long program synchronously inside the event loop, blocking rendering.
**Why bad:** TUI becomes unresponsive; user cannot press R/S to stop a running program.
**Instead:** Run programs in a Tokio task. Use a shared `AtomicBool` running flag. Send display updates back via mpsc channel to the render loop. (This is the main async complexity in the architecture — flag for phase-specific research.)

### Anti-Pattern 6: Panics in hp41-core

**What:** Using `unwrap()`, `expect()`, `panic!()` in hp41-core for expected calculator errors like divide-by-zero.
**Why bad:** NFR-3 requires crash-free sessions; panics crash the process.
**Instead:** Return `Err(HpError::DivisionByZero)`. hp41-cli catches and displays "DIVIDE BY 0" in the display widget.

---

## Scalability Considerations

| Concern | v1.0 approach | v2.0 concern |
|---|---|---|
| Program execution speed | Synchronous step-by-step with Tokio task | Same — behavioral emulation is fast enough |
| State sharing (CLI → GUI) | Not needed; GUI replaces CLI | Tauri uses Rust backend directly, same `CalcState` |
| Memory size | `Vec<Instruction>` is trivially small (HP-41 max ~2200 bytes of program) | No concern |
| Serialization compatibility | v1 JSON + version field | Migration functions keyed on version number |
| Test isolation | `CalcState::default()` per test; no shared state | Same pattern scales |

---

## Sources

- [Free42 Source Code (Codeberg)](https://codeberg.org/thomasokken/free42) — core/ vs platform/ separation pattern (HIGH confidence)
- [Ratatui Elm Architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/) — Model/Message/update/view pattern (HIGH confidence)
- [Ratatui Event Handling](https://ratatui.rs/concepts/event-handling/) — centralized catch + message passing (HIGH confidence)
- [Ratatui Tui.rs Template](https://ratatui.rs/templates/component/tui-rs/) — Tui struct, async event loop, Drop cleanup (HIGH confidence)
- [HP-41C Programming Guide](https://www.hpmuseum.org/prog/hp41prog.htm) — FOCAL execution model, conditional skip semantics (MEDIUM confidence — 403 on fetch but corroborated by multiple sources)
- [HP-41C Wikipedia](https://en.wikipedia.org/wiki/HP-41C) — memory layout, register structure (MEDIUM confidence)
- [FOCAL Language (Academic Kids)](https://academickids.com/encyclopedia/index.php/Focal_(HP-41)) — instruction model, linear program store (MEDIUM confidence)
- [HP Museum — GSB/XEQ/GTO/LBL/RTN quirks](https://www.hpmuseum.org/forum/thread-15118-post-132721.html) — return stack behavior, RTN vs END (MEDIUM confidence — 403 on fetch but content referenced in search result)
- [HP-41 Info](http://dan.pfeiffer.net/hp41/hp41info.htm) — stack lift rules, flags, annunciators (attempted fetch — connection refused; content referenced from other sources)
- [ISG/DSE format — HP 35s Manual (ManualsLib)](https://www.manualslib.com/manual/257003/Hp-35s.html?page=228) — ccccccc.fffii counter format; identical on HP-41 (MEDIUM confidence)
- [enum_dispatch vs dyn Trait](https://docs.rs/enum_dispatch/latest/enum_dispatch/) — closed-world enum preferred for fixed instruction sets (HIGH confidence)
- [thiserror](https://github.com/dtolnay/thiserror) — library error types (HIGH confidence)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) — crate dependency direction, build order (HIGH confidence)
- [WezTerm DeepWiki](https://deepwiki.com/wezterm/wezterm) — terminal emulator workspace pattern, UI-agnostic core (MEDIUM confidence)
- [rscalc (D0ntPanic)](https://github.com/D0ntPanic/rscalc) — RPN stack calculator Rust source structure (MEDIUM confidence)
- [SwissMicros Forum — Stack Lift Rules](https://forum.swissmicros.com/viewtopic.php?t=2699) — stack lift enable/disable/neutral classification (MEDIUM confidence — 403 on fetch; content summary from search result)
