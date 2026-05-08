# Emulator Architecture

This document describes the internal structure of the HP-41 emulator for contributors.

## Crate Layout

```
hp41-calculator-emulator/
├── hp41-core/          ← calculator engine (no UI dependencies)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── calculator.rs   ← top-level Calculator struct
│   │   ├── stack.rs        ← 4-register stack + LAST X
│   │   ├── registers.rs    ← data register file (R00–R99)
│   │   ├── flags.rs        ← flag array (0–55)
│   │   ├── alpha.rs        ← alpha register (24 chars)
│   │   ├── ops/            ← one module per operation category
│   │   │   ├── arithmetic.rs
│   │   │   ├── math.rs
│   │   │   ├── stack_ops.rs
│   │   │   ├── storage.rs
│   │   │   ├── conditionals.rs
│   │   │   ├── flags.rs
│   │   │   ├── program.rs
│   │   │   ├── alpha.rs
│   │   │   ├── statistics.rs
│   │   │   └── conversions.rs
│   │   ├── program.rs      ← program memory & step execution
│   │   └── state.rs        ← serde-based snapshot (save/load)
│   └── tests/
│       ├── stack_tests.rs
│       ├── ops_tests.rs
│       └── snapshots/      ← insta snapshot files
│
└── hp41-cli/           ← TUI binary (ratatui + crossterm)
    └── src/
        ├── main.rs         ← event loop: poll → update → redraw
        ├── app.rs          ← App state (wraps Calculator)
        ├── ui.rs           ← ratatui widget tree
        └── input.rs        ← key → Operation mapping
```

**Core invariant:** `hp41-core` must never import from `hp41-cli` or `hp41-gui`.  
This is enforced at compile time by Cargo's dependency graph.

---

## Calculator Struct

`hp41_core::Calculator` is the single entry point for the engine:

```rust
pub struct Calculator {
    pub stack: Stack,           // X, Y, Z, T, LAST X
    pub registers: Registers,   // R00–R99
    pub flags: Flags,           // flags 0–55
    pub alpha: AlphaRegister,   // 24-char string
    pub program: Program,       // program steps + PC
    pub display: Display,       // formatted display string
    mode: Mode,                 // Normal / Prgm / Alpha / User
    lift_enabled: bool,         // current stack-lift state
    angle_mode: AngleMode,      // Deg / Rad / Grad
    display_mode: DisplayMode,  // Fix(n) / Sci(n) / Eng(n) / All
}

impl Calculator {
    pub fn execute(&mut self, op: Operation) -> Result<(), CalcError>;
    pub fn save(&self) -> State;          // serialize to JSON-ready struct
    pub fn load(state: State) -> Self;    // deserialize
}
```

---

## Stack

```rust
pub struct Stack {
    pub x: Decimal,
    pub y: Decimal,
    pub z: Decimal,
    pub t: Decimal,
    pub last_x: Decimal,
}
```

All values use `rust_decimal::Decimal` (BCD-backed, 28 significant digits).  
Results are rounded to **10 significant decimal digits** to match HP-41 hardware.

### Stack-lift Protocol

Every `Operation` declares its lift behaviour as one of:

```rust
pub enum LiftEffect { Enable, Disable, Neutral }
```

`Calculator::execute` applies the declared effect *after* the operation completes:

1. **Before** the operation: if `lift_enabled`, digit entry would push the stack.
2. The operation runs and writes its result to X (or wherever).
3. **After** the operation: set `lift_enabled` according to `LiftEffect`.

---

## Operation Dispatch

Operations are represented as a plain Rust enum:

```rust
pub enum Operation {
    // Arithmetic
    Add, Sub, Mul, Div, ChangeSign,
    // Math
    Sqrt, Square, Recip, Pow, Log, Alog, Ln, Exp,
    // Trig
    Sin, Cos, Tan, Asin, Acos, Atan,
    // Stack
    Enter, Clx, RollDown, RollUp, Swap, LastX,
    // Storage
    Sto(u8), Rcl(u8), StoAdd(u8), /* … */
    // Conditionals
    XeqY, XneY, XltY, /* … */
    // Flags
    Sf(u8), Cf(u8), FsTest(u8), /* … */
    // Program control
    Lbl(Label), Gto(Label), Gsb(Label), Rtn, End, Stop, Pse,
    // Alpha
    Cla, Aview, Prompt, Arcl(u8), Asto(u8), Xtoa, Atox,
    // …
}
```

The TUI maps physical key events to `Operation` values in `hp41-cli/src/input.rs`.

---

## Numerical Precision

- **Storage type:** `rust_decimal::Decimal` (96-bit integer mantissa, no binary floating-point)
- **Display rounding:** 10 significant decimal digits (matches hardware)
- **Trig results:** computed via f64 (`sin`, `cos`, etc.), then converted back to `Decimal` and rounded to 10 sig-figs
- **ISG/DSE counters:** fields extracted by splitting the decimal string representation — never with `floor()` / `fmod()` on f64

---

## State Persistence

`hp41_core::State` is a plain serde-serializable struct:

```rust
#[derive(Serialize, Deserialize)]
pub struct State {
    pub stack: [String; 5],      // X, Y, Z, T, LAST X as decimal strings
    pub registers: Vec<String>,
    pub flags: [bool; 56],
    pub alpha: String,
    pub program: Vec<Step>,
    pub mode: String,
    pub angle_mode: String,
    pub display_mode: String,
}
```

Saved as human-readable JSON at `~/.hp41/state.json` (path configurable).  
The format is **version-stable**: adding new fields with `#[serde(default)]` is backward-compatible.

---

## TUI Event Loop

`hp41-cli` runs a single-threaded loop (no async):

```
loop {
    if poll(timeout)? {
        let event = read()?;
        if let Some(op) = input::map(event) {
            app.calculator.execute(op)?;
        }
    }
    terminal.draw(|frame| ui::render(frame, &app))?;
}
```

- `poll(timeout)` blocks for at most `timeout` (≈16 ms for ~60 fps).
- On Windows, `KeyEventKind::Release` events are filtered immediately to prevent double-firing.
- The panic hook is installed via `ratatui::init()` (not `Terminal::new()`), which restores the terminal on panic.

---

## Testing

| Layer | Tool | Target |
|-------|------|--------|
| Unit & property | `cargo test` + `proptest` | `hp41-core` |
| Snapshot | `insta` | display formatting, state serialization |
| Coverage gate | `cargo-llvm-cov` | ≥80% line coverage on `hp41-core` |
| Numerical accuracy | hand-crafted 500-case suite | ≥98% agreement vs HP-41 hardware |

Run the full gate:

```bash
just ci   # lint → test → coverage
```

---

## Adding a New Operation

1. Add a variant to `Operation` in `hp41-core/src/ops/mod.rs`.
2. Implement the logic in the appropriate `ops/*.rs` module.
3. Declare the `LiftEffect` for the new operation.
4. Map the key in `hp41-cli/src/input.rs`.
5. Add a unit test and, if it produces display output, a snapshot test.
6. Update [Operations Reference](operations-reference.md).
