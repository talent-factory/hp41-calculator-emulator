# Emulator Architecture

This document describes the internal structure of the HP-41 emulator for contributors.

## Crate Layout

```
hp41-calculator-emulator/
├── hp41-core/                     ← calculator engine (no UI dependencies)
│   ├── src/
│   │   ├── lib.rs                 ← #![deny(clippy::unwrap_used)] at crate root
│   │   ├── num.rs                 ← HpNum (wraps rust_decimal::Decimal)
│   │   ├── state.rs               ← CalcState — single source of truth
│   │   ├── stack.rs               ← 4-register stack + LASTX, lift_enabled flag
│   │   ├── synthetic.rs           ← synthetic_byte_to_op() — 23-entry safe subset
│   │   └── ops/                   ← one module per operation category
│   │       ├── mod.rs             ← Op enum, dispatch(), flush_entry_buf(), LiftEffect
│   │       ├── arithmetic.rs
│   │       ├── math.rs            ← trig, log, exp, pow
│   │       ├── stack_ops.rs
│   │       ├── registers.rs       ← STO/RCL, STO+/-/×/÷, op_sto_arith_stack()
│   │       ├── conditionals.rs
│   │       ├── flags.rs
│   │       ├── program.rs         ← run_program(), run_loop(), parse_counter(), execute_op()
│   │       ├── alpha.rs
│   │       ├── statistics.rs
│   │       ├── conversions.rs
│   │       └── print.rs           ← op_prx, op_pra, op_prstk (buffer-only, NO println!)
│   └── tests/                     ← stack_tests, ops_tests, print_tests, synthetic_tests, numerical_accuracy
│
├── hp41-cli/                      ← TUI binary (ratatui 0.30 + crossterm 0.29)
│   └── src/
│       ├── main.rs                ← entry; clap parsing of --print-log
│       ├── app.rs                 ← App, PendingInput, handle_key(), call_dispatch_and_drain()
│       ├── ui.rs                  ← ratatui widget tree, format_entry_buf_display()
│       ├── keys.rs                ← key_to_op(), KEY_REF_TABLE, keycode_to_hp41_code()
│       ├── help_data.rs           ← HELP_DATA — single source of truth for `?` overlay
│       └── persistence.rs         ← save_state(), load_state() — JSON serde
│
└── hp41-gui/                      ← Tauri v2 desktop app (nested standalone workspace)
    ├── src-tauri/                 ← Rust backend
    │   ├── src/
    │   │   ├── main.rs            ← thin shim; defers to lib.rs::run()
    │   │   ├── lib.rs             ← setup(), AppState = Mutex<CalcState>, 30s auto-save thread
    │   │   ├── commands.rs        ← dispatch_op, get_state, sst_step, bst_step Tauri thunks
    │   │   ├── types.rs           ← CalcStateView, Annunciators, GuiError, From<HpError>
    │   │   ├── key_map.rs         ← resolve() — string ID → Op (no calculator logic!)
    │   │   ├── persistence.rs     ← shared ~/.hp41/autosave.json (same schema as CLI)
    │   │   └── prgm_display.rs    ← format_all_steps() — always appends END
    │   ├── permissions/           ← Tauri v2.11 inline-command permission TOML files
    │   └── capabilities/default.json
    └── src/                       ← React + TypeScript frontend (Vite)
        ├── main.tsx
        ├── App.tsx                ← display, annunciators, stack panel, busyRef, resolveKeyId
        ├── Keyboard.tsx           ← 44-key SVG skin, KEY_DEFS, pressedKey state
        └── App.css                ← layout + key animation (requires transform-box: fill-box)
```

**Core invariant:** `hp41-core` must never import from `hp41-cli` or `hp41-gui`. Enforced at compile time by Cargo's dependency graph.

**Workspace isolation:** Root `Cargo.toml` declares `members = ["hp41-core", "hp41-cli"]`. `hp41-gui` is a **nested standalone workspace** — the `tauri` and `tauri-build` dependencies never enter the root Cargo resolver, and `cargo build --workspace` from the repo root does not touch the Tauri binary.

**No core duplication (SC-4 invariant):** `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` MUST return nothing. The GUI Rust layer only routes IPC; all calculator logic lives in `hp41-core`.

---

## CalcState — Single Source of Truth

`hp41_core::CalcState` (in `hp41-core/src/state.rs`) holds every piece of calculator state. There is no separate `Calculator` wrapper — each op takes `&mut CalcState`.

```rust
pub struct CalcState {
    pub stack: Stack,                  // X, Y, Z, T, LASTX + lift_enabled
    pub registers: Vec<HpNum>,         // R00–R99
    pub flags: [bool; 56],             // flags 0–55
    pub alpha: String,                 // 24-char ALPHA register
    pub program: Vec<Op>,              // program memory
    pub pc: usize,                     // program counter
    pub mode: Mode,                    // Normal / Prgm / Alpha / User
    pub angle_mode: AngleMode,         // Deg / Rad / Grad
    pub display_mode: DisplayMode,     // Fix(n) / Sci(n) / Eng(n) / All
    pub entry_buf: String,             // in-progress digit entry (EEX-aware)

    // v1.1 additions
    #[serde(skip)]
    pub print_buffer: Vec<String>,     // PRX/PRA/PRSTK output; transient, never persisted
    #[serde(default)]
    pub last_key_code: u8,             // updated each keypress; consumed by Op::GetKey
    #[serde(default)]
    pub reg_m: HpNum,                  // hidden synthetic register M
    #[serde(default)]
    pub reg_n: HpNum,                  // hidden synthetic register N
    #[serde(default)]
    pub reg_o: HpNum,                  // hidden synthetic register O
}
```

Every new field added since v1.0 carries `#[serde(default)]` so v1.x JSON save files load unchanged. `print_buffer` carries `#[serde(skip)]` because it is transient runtime state.

Dispatch is performed by free functions:

```rust
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<LiftEffect, HpError>;
pub fn execute_op(state: &mut CalcState, op: Op) -> Result<(), HpError>;  // for programmatic run
```

Every new `Op` variant must be added to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs`, AND to the exhaustive `prgm_display` match in `hp41-gui/src-tauri/src/prgm_display.rs`. Missing any of these is a compile-time error.

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

## Op Dispatch

Operations are represented as a plain Rust enum in `hp41-core/src/ops/mod.rs`:

```rust
pub enum Op {
    // Arithmetic
    Add, Sub, Mul, Div, Chs,
    // Math
    Sqrt, Square, Recip, Pow, Log, Alog, Ln, Exp,
    // Trig
    Sin, Cos, Tan, Asin, Acos, Atan,
    // Stack
    Enter, Clx, RollDown, RollUp, Swap, LastX,
    // Storage
    Sto(u8), Rcl(u8), StoAdd(u8), StoSub(u8), StoMul(u8), StoDiv(u8),
    StoArithStack(StoArithKind, StackReg),     // v1.1: STO+/-/×/÷ to Y/Z/T/LASTX
    StoM, StoN, StoO, RclM, RclN, RclO,         // v1.1: synthetic hidden registers
    // Conditionals
    XeqY, XneY, XltY, XgtY, XleY, XgeY, /* … */
    // Flags
    Sf(u8), Cf(u8), FsTest(u8), /* … */
    // Program control
    Lbl(Label), Gto(Label), Xeq(Label), Rtn, End, Stop, Pse,
    Isg(u8), Dse(u8),
    // Alpha
    Cla, Aview, Prompt, Arcl(u8), Asto(u8), Xtoa, Atox, AlphaClear,
    // v1.1: Print emulation (push to print_buffer, never println!)
    PRX, PRA, PRSTK,
    // v1.1: Synthetic programming
    GetKey,                                     // pushes last_key_code to X
    Null,                                       // Neutral stack-lift, no state change
    SyntheticByte(u8),                          // validated via synthetic_byte_to_op() before insert
}
```

Each variant also declares its `LiftEffect` (Enable / Disable / Neutral) in the `dispatch()` match. The TUI maps physical key events to `Op` values in `hp41-cli/src/keys.rs::key_to_op()`. The GUI maps string IDs (`"enter"`, `"plus"`, `"sin"`, ...) to `Op` values in `hp41-gui/src-tauri/src/key_map.rs::resolve()`.

---

## Numerical Precision

- **Storage type:** `rust_decimal::Decimal` (96-bit integer mantissa, no binary floating-point)
- **Display rounding:** 10 significant decimal digits (matches hardware)
- **Trig results:** computed via f64 (`sin`, `cos`, etc.), then converted back to `Decimal` and rounded to 10 sig-figs
- **ISG/DSE counters:** fields extracted by splitting the decimal string representation — never with `floor()` / `fmod()` on f64

---

## State Persistence

`CalcState` derives `Serialize` / `Deserialize` directly — there is no separate snapshot DTO. The serialized JSON contains every persistent field listed in the `CalcState` definition above (stack, registers, flags, alpha, program, mode, angle_mode, display_mode, entry_buf, last_key_code, reg_m/n/o, …) **except** `print_buffer`, which is marked `#[serde(skip)]` because it is transient runtime output.

Saved as human-readable JSON at `~/.hp41/autosave.json`.

**Forward/backward compatibility:** Every field added since v1.0 carries `#[serde(default)]`. v1.x save files load unchanged in v1.1 / v2.0 — missing fields default to their zero value. Save files written by v2.0 also load in v1.0 because the new fields are simply ignored.

**Shared between CLI and GUI:** Both `hp41-cli` and `hp41-gui` read and write the **same** file path via the `dirs` crate. A state saved in the CLI appears in the GUI on next launch and vice versa. Both binaries auto-save every 30 s; the GUI runs its auto-save on a dedicated thread and releases the `AppState` Mutex before disk I/O.

---

## TUI Event Loop (`hp41-cli`)

`hp41-cli` runs a single-threaded loop (no async):

```
loop {
    if poll(timeout)? {
        let event = read()?;
        if let Some(op) = keys::key_to_op(event) {
            app.call_dispatch_and_drain(op)?;   // dispatch + drain print_buffer
        }
    }
    terminal.draw(|frame| ui::render(frame, &app))?;
}
```

- `poll(timeout)` blocks for at most `timeout` (≈16 ms for ~60 fps).
- On Windows, `KeyEventKind::Release` events are filtered immediately to prevent double-firing.
- The panic hook is installed via `ratatui::init()` (not `Terminal::new()`), which restores the terminal on panic.
- `call_dispatch_and_drain()` is the **only** interactive dispatch path; for programmatic execution (`run_program()` after R/S, F1–F4 USER, `try_user_dispatch`) the wrapper `drain_and_show_print_output()` runs after every successful `run_program()` so PRX/PRA/PRSTK output is never silently dropped.
- `PendingInput` carries multi-step keyboard modal state (STO register entry, STO arithmetic, HexModal for synthetic-byte insertion, …). The `pending_input` routing block must remain ABOVE the modal-opening interceptors (`S`, `R`, `Ctrl+A`) so an active modal is not cancelled by another modal trigger.

---

## GUI Architecture (`hp41-gui`)

`hp41-gui` is a Tauri v2 app reusing `hp41-core` unchanged. The Rust backend is a thin IPC adapter; all calculator logic lives in `hp41-core`.

### IPC contract

Four Tauri commands, all returning a `CalcStateView`:

```rust
#[tauri::command] async fn dispatch_op(state: State<AppState>, key_id: String) -> Result<CalcStateView, GuiError>;
#[tauri::command] async fn get_state(state: State<AppState>) -> Result<CalcStateView, GuiError>;
#[tauri::command] async fn sst_step(state: State<AppState>) -> Result<CalcStateView, GuiError>;
#[tauri::command] async fn bst_step(state: State<AppState>) -> Result<CalcStateView, GuiError>;

type AppState = Mutex<CalcState>;
```

`CalcStateView` is the JSON payload sent to the frontend on every command — a lean ~170-byte projection of `CalcState` (display string, annunciators, X/Y/Z/T/LASTX strings, `in_eex_mode`, `print_lines` drained from `print_buffer`, `program_steps`, `pc`). It is **not** a full `CalcState` mirror — only the fields the React UI renders.

### Key routing

The frontend never references Rust enums. Every key sends a string ID (`"enter"`, `"plus"`, `"sin"`, `"sto-add"`, `"sst"`, …) that `key_map::resolve(key_id)` turns into an `Op`. New keys are added in two places only: `KEY_DEFS` in `Keyboard.tsx` (or `resolveKeyId()` in `App.tsx` for physical keyboard) and the `resolve()` match in `key_map.rs`.

### Concurrency

- One auto-save thread spawned by `setup()` writes `~/.hp41/autosave.json` every 30 s. It locks `AppState`, clones the state, releases the lock, then writes to disk.
- Mutex locks always use `.unwrap_or_else(|e| e.into_inner())` so a poisoned lock is recovered — never `.unwrap()` or `.expect()`. This preserves the zero-panic invariant in the GUI crate.

### Frontend (`hp41-gui/src/`)

- `App.tsx` — root component: display, annunciators, stack panel, scrollable print panel, conditional program-listing panel. Physical keyboard listener uses `useCallback` + `useEffect` with `e.repeat` guard and a `busyRef = useRef(false)` debounce to prevent concurrent `invoke()` calls.
- `Keyboard.tsx` — 44-key inline SVG (no external SVG library). `KEY_DEFS` array drives layout; `pressedKey` state machine with a 150 ms `setTimeout` and functional setState (avoids stale closure) renders the CSS scale-down animation. CSS requires `transform-box: fill-box` on `.key` so SVG `scale()` transforms from each key's own centre rather than the canvas origin.
- `App.css` — vanilla CSS, no Tailwind (Tailwind was removed in Phase 15).

---

## Testing

| Layer | Tool | Target | Current |
|-------|------|--------|---------|
| Unit & property | `cargo test` + `proptest` | `hp41-core` | 150+ tests |
| Print emulation | integration | `hp41-core/tests/print_tests.rs` | passing |
| Synthetic ops | integration | `hp41-core/tests/synthetic_tests.rs` | 21 tests |
| Snapshot | `insta` | display formatting, state serialization | passing |
| Coverage gate | `cargo-llvm-cov` | ≥80% line coverage on `hp41-core` | 94%+ |
| Numerical accuracy | hand-crafted 500-case suite | ≥98% agreement vs HP-41 hardware | 99% (495/500) |
| TUI integration | `cargo test --bin hp41-cli` | `hp41-cli` | ~99 tests |
| GUI Rust | `cargo test` (gui workspace) | `hp41-gui/src-tauri` | 13+ tests |
| TypeScript build | `tsc --noEmit` + Vite build | `hp41-gui/src` | gated by `gui-ci` |

Run the full gates:

```bash
just ci        # CLI pipeline:  lint → test → coverage → MSRV
just gui-ci    # GUI pipeline:  cargo test → cargo build --release  (3-OS matrix in CI)
```

`just ci` and `just gui-ci` are independent — a GUI build failure does not block the CLI pipeline and vice versa.

---

## Adding a New Operation

1. Add a variant to `Op` in `hp41-core/src/ops/mod.rs`.
2. Implement the logic in the appropriate `ops/*.rs` module (signature: `fn op_xxx(state: &mut CalcState) -> Result<(), HpError>`). If the op pushes print output, use `state.print_buffer.push(line)` — never `println!`.
3. Declare the `LiftEffect` (Enable / Disable / Neutral) in the `dispatch()` match.
4. Add a corresponding arm in `execute_op()` in `ops/program.rs` (programmatic-run path).
5. If `Op` carries a parameter and appears in program listings, add an arm to the exhaustive match in `hp41-gui/src-tauri/src/prgm_display.rs::format_step()`.
6. **CLI:** map the key in `hp41-cli/src/keys.rs::key_to_op()` and add a `KEY_REF_TABLE` entry; add a `HELP_DATA` entry in `hp41-cli/src/help_data.rs` if the key is user-visible.
7. **GUI:** add a string-ID arm in `hp41-gui/src-tauri/src/key_map.rs::resolve()`; if it has a SVG key, add a `KEY_DEFS` entry in `hp41-gui/src/Keyboard.tsx` (or a physical-keyboard mapping in `App.tsx::resolveKeyId()`).
8. Add a unit test in the appropriate `tests/*.rs` file and, if it produces display output, a snapshot test.
9. Update [Operations Reference](operations-reference.md).

If you forget step 4, programs that contain the new op will fail at run-time; if you forget step 5, the GUI program listing will refuse to compile.
