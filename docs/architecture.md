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
│   │   ├── error.rs               ← HpError (incl. AlphaData, CardData(String))
│   │   ├── format.rs              ← format_hpnum, format_alpha (display formatting)
│   │   ├── cardreader/            ← Card Reader codecs + state helpers (no disk I/O)
│   │   │   ├── mod.rs             ← CardOpRequest, insert_program_ops, capture/load_data_card
│   │   │   ├── raw.rs             ← bare .raw byte codec (V41/Free42 family)
│   │   │   └── data.rs            ← .card.json codec (hp41-data-v1 magic tag)
│   │   └── ops/                   ← one module per operation category
│   │       ├── mod.rs             ← Op enum, dispatch(), flush_entry_buf(), LiftEffect,
│   │       │                        synthetic_byte_to_op() — 24-entry safe subset
│   │       ├── arithmetic.rs      ← Add/Sub/Mul/Div
│   │       ├── math.rs            ← trig, log, exp, ypow, sq, sqrt, recip, int
│   │       ├── stack_ops.rs       ← Enter/Clx/Chs/Rdn/XySwap/Lastx
│   │       ├── registers.rs       ← op_sto / op_rcl / op_sto_arith / op_sto_arith_stack
│   │       ├── alpha.rs           ← AlphaToggle/Append/Clear/Backspace
│   │       ├── program.rs         ← run_program(), run_loop(), parse_counter(), execute_op()
│   │       ├── print.rs           ← op_prx, op_pra, op_prstk (buffer-only, NO println!)
│   │       ├── stats.rs           ← Σ+/Σ−, Mean, Sdev, L.R., Yhat, Corr, ClSigmaStat
│   │       ├── hms.rs             ← HmsToH, HToHms, HmsAdd, HmsSub
│   │       └── cardreader_ops.rs  ← op_wdta / op_rdta / op_wprgm / op_rdprgm (stage requests)
│   └── tests/                     ← stack_tests, ops_tests, print_tests, synthetic_tests,
│                                    cardreader_tests, entry_buf_tests, numerical_accuracy
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

`hp41_core::CalcState` (in [`hp41-core/src/state.rs`](../hp41-core/src/state.rs)) holds every piece of calculator state. There is no separate `Calculator` wrapper — each op takes `&mut CalcState`.

Rather than duplicating the struct here (and rotting), read the source. The fields fall into five groups:

1. **Stack & data registers** — `stack: Stack`, `regs: Vec<HpNum>` (100 entries, R00–R99), `alpha_reg: String` (24-char ALPHA), `entry_buf: String` (in-progress digit/EEX entry).
2. **Mode state** — `angle_mode: AngleMode` (Deg/Rad/Grad), `display_mode: DisplayMode` (Fix/Sci/Eng with `u8` digit count), and three independent booleans `alpha_mode`, `prgm_mode`, `user_mode` (there is no enum-based `Mode`).
3. **Program memory & execution** — `program: Vec<Op>`, `pc: usize`, `call_stack: Vec<usize>` (max 4 deep), `is_running: bool` (re-entrancy guard).
4. **USER mode** — `key_assignments: BTreeMap<char, String>` (BTreeMap for deterministic JSON ordering).
5. **v1.1 additions** —
   - `print_buffer: Vec<String>` with `#[serde(default, skip)]` (transient PRX/PRA/PRSTK output, never persisted).
   - `last_key_code: u8` with `#[serde(default)]` (consumed by `Op::GetKey`).
   - `reg_m`, `reg_n`, `reg_o: HpNum` with `#[serde(default)]` (hidden synthetic registers).
6. **Card Reader staging slot** — `pending_card_op: Option<CardOpRequest>` with `#[serde(default, skip)]` (transient I/O request, drained by the frontend; see [Card Reader I/O](#card-reader-io)).

**HP-41 flags (0–55) are not implemented** in v1.0/v1.1/v2.0 — there is no `flags` field on `CalcState` and no `Op::Sf` / `Op::Cf` / `Op::FsTest` variants. This is a documented v2.x+ gap; programs that rely on flags do not run.

Every new field added since v1.0 carries `#[serde(default)]` so v1.x JSON save files load unchanged. `print_buffer` carries `#[serde(default, skip)]` because it is transient runtime state that must never appear in JSON.

Dispatch is performed by free functions in `hp41-core/src/ops/`:

```rust
pub fn dispatch(state: &mut CalcState, op: Op) -> Result<LiftEffect, HpError>;     // ops/mod.rs
pub fn execute_op(state: &mut CalcState, op: Op) -> Result<(), HpError>;            // ops/program.rs (for programmatic run)
pub fn synthetic_byte_to_op(byte: u8) -> Option<Op>;                                // ops/mod.rs:451 (Phase 12 safe subset)
```

Every new `Op` variant must be added to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs`, AND to the exhaustive `prgm_display` match in BOTH `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs`. Missing any of these is a compile-time error.

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

Operations are represented as a plain Rust enum [`hp41_core::ops::Op`](../hp41-core/src/ops/mod.rs) — read the source for the authoritative variant list (it is the single source of truth and changes faster than this doc). The variants fall into these categories:

- **Arithmetic** — `Add`, `Sub`, `Mul`, `Div`.
- **Stack** — `Enter`, `Clx`, `Chs`, `Rdn`, `XySwap`, `Lastx`, `PushNum(HpNum)`.
- **Unary math** — `Int`, `Recip`, `Sqrt`, `Sq`, `YPow`, `Ln`, `Log`, `Exp`, `TenPow`.
- **Trig** — `Sin`, `Cos`, `Tan`, `Asin`, `Acos`, `Atan`.
- **Mode setters** — `SetDeg`, `SetRad`, `SetGrad`, `FmtFix(u8)`, `FmtSci(u8)`, `FmtEng(u8)`.
- **Storage registers** — `StoReg(u8)`, `RclReg(u8)`, `StoArith { reg: u8, kind: StoArithKind }`, `StoArithStack { kind: StoArithKind, stack_reg: StackReg }`, `Clreg`.
- **ALPHA** — `AlphaToggle`, `AlphaAppend(char)`, `AlphaClear`, `AlphaBackspace`.
- **Programming** — `Lbl(String)`, `Gto(String)`, `Xeq(String)`, `Rtn`, `PrgmMode`, `Test(TestKind)`, `Isg(u8)`, `Dse(u8)`.
- **USER mode** — `UserMode`.
- **Statistics (Phase 6)** — `SigmaPlus`, `SigmaMinus`, `Mean`, `Sdev`, `LR`, `Yhat`, `Corr`, `ClSigmaStat`.
- **HMS conversions (Phase 6)** — `HmsToH`, `HToHms`, `HmsAdd`, `HmsSub`.
- **Print emulation (Phase 11)** — `PRX`, `PRA`, `PRSTK` (push lines to `state.print_buffer`; never `println!`).
- **Synthetic programming (Phase 12)** — `GetKey` (pushes `last_key_code` to X), `Null` (Neutral no-op), `StoM/N/O`, `RclM/N/O` (hidden registers), `SyntheticByte(u8)` (deferred — resolved at execute-time via `synthetic_byte_to_op()`).
- **Card Reader** — `Wdta`, `Rdta`, `Wprgm`, `Rdprgm` (each stages a `CardOpRequest` on `state.pending_card_op`; no disk I/O in core).

Supporting enums (also in `ops/mod.rs`):

```rust
pub enum StoArithKind { Add, Sub, Mul, Div }
pub enum StackReg     { Y, Z, T, Lastx }
pub enum TestKind     { /* 12 variants — see source */ }
```

Each `Op` variant also declares its `LiftEffect` (Enable / Disable / Neutral) in the `dispatch()` match. The TUI maps physical key events to `Op` values in `hp41-cli/src/keys.rs::key_to_op()`. The GUI maps string IDs (`"enter"`, `"plus"`, `"sin"`, ...) to `Op` values in `hp41-gui/src-tauri/src/key_map.rs::resolve()` — the frontend never references Rust enums directly.

---

## Numerical Precision

- **Storage type:** `rust_decimal::Decimal` (96-bit integer mantissa, no binary floating-point)
- **Display rounding:** 10 significant decimal digits (matches hardware)
- **Trig results:** computed via f64 (`sin`, `cos`, etc.), then converted back to `Decimal` and rounded to 10 sig-figs
- **ISG/DSE counters:** fields extracted by splitting the decimal string representation — never with `floor()` / `fmod()` on f64

---

## Card Reader I/O

The HP 82104A card reader is emulated with a **staging-drain** pattern that
mirrors `print_buffer`: `hp41-core` performs no disk I/O at all. The four ops
`Op::Wdta` / `Op::Rdta` / `Op::Wprgm` / `Op::Rdprgm` read the file name from
the ALPHA register (empty → `HpError::AlphaData`) and write a request into
`state.pending_card_op: Option<CardOpRequest>`. The frontend reads (and
clears) that field on the next round-trip, performs the read or write itself,
and — for read ops — calls back into the core helpers
`cardreader::insert_program_ops()` and `cardreader::load_data_card()` to
install the decoded content.

```rust
pub enum CardOpRequest {
    WriteProgram { name: String },
    WriteData    { name: String },
    ReadProgram  { name: String },
    ReadData     { name: String },
}
```

`pending_card_op` carries `#[serde(default, skip)]` so an in-flight request is
never persisted across autosave/load. The four op handlers refuse to stage a
new request when `pending_card_op.is_some()` — back-to-back card ops inside a
running program surface as `HpError::CardData(...)` rather than silently
dropping the first request.

### Codecs

`hp41-core::cardreader` provides two codecs, both UI-agnostic and side-effect-free:

| Module | Format | Magic / shape | Decode error |
|--------|--------|---------------|--------------|
| `cardreader::raw` | bare `.raw` byte stream — single-byte FOCAL codes plus two-byte forms for `STO nn` / `RCL nn` and `LBL/GTO/XEQ "name"`; END marker `C0 00 0D` is always appended on encode and required on decode | no header; END-terminated | truncation, oversize register, bad UTF-8 alpha payload, missing END marker |
| `cardreader::data` | `.card.json` — JSON envelope `{ format: "hp41-data-v1", version: 1, registers: [...] }` | `format` magic tag + numeric `version` | wrong tag, unsupported version, malformed JSON |

Both codec error paths surface as `HpError::CardData(String)`, where the
payload carries a short diagnostic (serde line/column, the offending byte,
which register was out of range) so the frontend can show something more
useful than a bare "CARD DATA".

### Hardware-fidelity notes

- **`Op::Null` encodes to `0xCD`** — `0xCF` is reserved for the LBL alpha
  prefix (`CF Fx ...`). Sharing the byte would corrupt round-trips when a
  `NULL` is followed by a synthetic byte in the `F0..=FF` range. `0xCD` is
  unused by our subset and unambiguous.
- **`STO nn` / `RCL nn` use prefixes `0xE0` / `0xE1`** to avoid colliding with
  the hidden-register synthetic bytes `0x90-0x92` / `0xB0-0xB2` used by
  `RclM/N/O` and `StoM/N/O`. This is deliberately not byte-identical with V41 —
  byte-for-byte V41 compatibility is a future deliverable.
- **`load_data_card` zero-pads `state.regs` to ≥100.** The op_sto/op_rcl gate
  is `reg < 100`, not `reg < state.regs.len()`. Shrinking `regs` below 100
  after loading a small card would turn a subsequent `STO 50` into a raw
  index panic.
- **`Op::SyntheticByte(b)` encoding** is resolved through
  `synthetic_byte_to_op(b)` when a canonical `Op` exists (`0xCF → Op::Null →
  0xCD`). Naked two-byte prefixes (`0x1D`, `0x1E`, `0xE0`, `0xE1`) are
  refused; other unknown bytes pass through verbatim and round-trip via
  `Op::SyntheticByte` on decode.

### Frontend contract

The frontend is responsible for:

1. **Drain.** After every `dispatch_op()` (GUI) or `call_dispatch_and_drain()`
   (CLI), take `pending_card_op` and act on it before the next op runs.
2. **Resolve the path.** Card names are application-defined — there is no
   hardware-prescribed directory. Typical resolution is
   `<cards-dir>/<name>.{raw,card.json}` with sanitisation against path
   separators in `name`.
3. **Encode/decode.** Call `cardreader::encode_program` /
   `cardreader::encode_data` for writes; `cardreader::decode_program` /
   `cardreader::decode_data` for reads.
4. **Install reads.** For `ReadProgram`, call
   `cardreader::insert_program_ops(state, ops)` (replace-or-insert-after-pc
   semantics). For `ReadData`, call `cardreader::load_data_card(state, card)`.
5. **Surface errors.** Any `HpError::CardData(msg)` returned by the codecs
   should be surfaced to the user — the `msg` payload already carries
   actionable detail.

---

## State Persistence

`CalcState` derives `Serialize` / `Deserialize` directly. Both `hp41-cli/src/persistence.rs` and `hp41-gui/src-tauri/src/persistence.rs` wrap it in a version-tagged container:

```rust
pub struct StateFile {
    pub version: u32,        // currently 1
    pub state: CalcState,
}
```

`load_state()` reads the wrapper and returns the inner `CalcState` with `is_running = false` forced (Pitfall 4 guard — never resume mid-execution after a reload). Every persistent field of `CalcState` appears in the JSON **except** `print_buffer`, which carries `#[serde(default, skip)]` so it is omitted on write and tolerated as absent on read.

Saved as human-readable JSON at `~/.hp41/autosave.json`.

**Forward/backward compatibility:** Every field added since v1.0 carries `#[serde(default)]`. v1.x save files load unchanged in v1.1 / v2.0 — missing fields default to their zero value. Save files written by v2.0 also load in v1.0 because the new fields are simply ignored.

**Shared between CLI and GUI:** Both binaries resolve to the **same** path via the `dirs` crate. A state saved in the CLI appears in the GUI on next launch and vice versa. Both binaries auto-save every 30 s; the GUI runs its auto-save on a dedicated thread and releases the `AppState` Mutex before disk I/O. Both distinguish "file exists but unreadable" (warn) from "file missing" (silent first-run case).

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
| Unit & property | `cargo test` + `proptest` (19 properties) | `hp41-core` | 1202/1202 workspace-wide |
| Print emulation | integration | `hp41-core/tests/print_tests.rs` | passing |
| Synthetic ops | integration | `hp41-core/tests/synthetic_tests.rs` | 21 tests |
| Indirect addressing | integration | `hp41-core/tests/indirect_addressing.rs` | 42 tests |
| Snapshot | `insta` | display formatting, state serialization | passing |
| Coverage gate | `cargo-llvm-cov` | ≥95% line coverage on `hp41-core` (Phase 27 atomic raise from 80%) | 95.25% lines / 93.75% regions |
| Numerical accuracy | hand-crafted 566-case suite | ≥98% agreement vs HP-41 hardware | 99.1% (561/566) |
| TUI integration | `cargo test --bin hp41-cli` | `hp41-cli` | ~99 tests |
| GUI Rust | `cargo test` (gui workspace) | `hp41-gui/src-tauri` | 13+ tests |
| GUI frontend | Vitest | `hp41-gui/src` | 142/142 |
| GUI E2E | WebdriverIO + tauri-driver (Ubuntu-only) | `hp41-gui/e2e/smoke.spec.ts` | smoke green on CI |
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
