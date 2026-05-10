# Phase 14: IPC Layer - Research

**Researched:** 2026-05-09
**Domain:** Tauri v2 Rust commands + hp41-core integration + key_map.rs
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**CalcStateView Fields**
- D-01: `CalcStateView` contains exactly: `display_str: String`, `x_str: String`, `annunciators: Annunciators`, `print_lines: Vec<String>`
- D-02: Y/Z/T/LASTX NOT included in Phase 14 — deferred to Phase 15
- D-03: `Annunciators` is a separate derived struct reading `user_mode`, `prgm_mode`, `alpha_mode`, `angle_mode == Rad`, `angle_mode == Grad`

**Key ID Convention**
- D-04: Digit keys use bare characters: `"0"`–`"9"`, `"."`, `"e"`
- D-05: Named ops use snake_case: `"enter"`, `"plus"`, `"sin"`, `"chs"`, etc. Mirror CLI `key_to_op()`
- D-06: Parameterized ops use compound key IDs: `"sto_05"`, `"rcl_12"`, `"fix_4"`, `"sto_arith_plus_05"`, `"sto_arith_minus_y"`
- D-07: `key_map.rs` is exhaustive — unknown key IDs return `Err(GuiError { message: format!("unknown key: {key_id}") })`

**Modal State Ownership**
- D-08: Phase 14 is stateless IPC only — no multi-step modal sequencing
- D-09: No `PendingModal` state in `AppState` — `AppState = Mutex<CalcState>` is complete Rust-side state

**Error Response Shape**
- D-10: `dispatch_op` and `get_state` return `Result<CalcStateView, GuiError>`
- D-11: `GuiError` is `#[derive(Debug, Serialize)] pub struct GuiError { pub message: String }` with `impl From<HpError> for GuiError`

**Tauri Capabilities**
- D-12: `capabilities/default.json` updated to add IPC permissions for the two commands

### Claude's Discretion
- `display_str`: use `format_hpnum(&state.stack.x, &state.display_mode)` for normal mode; or mirror CLI `get_display_string()` logic — Claude decides
- `x_str`: use `state.stack.x.to_string()` or same formatted path as `display_str` — Claude decides what's most useful for Phase 15
- Poisoned-lock recovery: `.unwrap_or_else(|e| e.into_inner())` on `state.lock()` calls
- `#![deny(clippy::unwrap_used)]` applies to all new Phase 14 code

### Deferred Ideas (OUT OF SCOPE)
- Y/Z/T/LASTX in `CalcStateView` — Phase 15
- TypeScript type generation for `CalcStateView` and `GuiError` — Phase 15
- Stack panel rendering — Phase 15
- Physical keyboard wiring — Phase 15
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| IPC-01 | All user operations reach `hp41-core` via Tauri Rust commands (`dispatch_op`, `get_state`); the response is a `CalcStateView` (~200 bytes); `print_buffer` is drained on every command; no `hp41-core` logic is duplicated in the GUI crate | Tauri v2 command/state patterns verified; `dispatch()` and `flush_entry_buf()` signatures confirmed; `print_buffer.drain(..)` pattern confirmed from CLI Phase 11 implementation |
</phase_requirements>

---

## Summary

Phase 14 wires `hp41-core` into the Tauri v2 app as two Rust commands (`dispatch_op`, `get_state`) via the existing `AppState = Mutex<CalcState>` type alias already defined in `lib.rs`. The IPC response is a lean `CalcStateView` DTO. All calculator logic remains in `hp41-core` — the `hp41-gui/src-tauri` layer is a thin adapter with three responsibilities: key ID resolution (`key_map.rs`), command handler plumbing (`commands.rs`), and Tauri registration (`lib.rs` updated).

The `print_buffer` drain pattern is already established in Phase 11's CLI (`call_dispatch_and_drain`): call `dispatch()`, then immediately `state.print_buffer.drain(..).collect()` to move lines into the response DTO. The mutable borrow from `dispatch()` ends before the drain, so there is no borrow conflict.

**Primary recommendation:** Three new files in `hp41-gui/src-tauri/src/`: `types.rs` (CalcStateView, Annunciators, GuiError structs), `key_map.rs` (string key ID to Op resolver), `commands.rs` (dispatch_op and get_state handlers). One file modified: `lib.rs` (register commands, add #[allow] for module declarations). Capability JSON updated with explicit permission entries.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Key ID to Op resolution | API / Backend (Rust) | — | `key_map.rs` in `hp41-gui/src-tauri/src/` — string parsing is Rust-side; frontend sends opaque string |
| Calculator state mutation | API / Backend (Rust) | — | `hp41-core::dispatch()` owns all state; Tauri command is the entry point |
| CalcStateView serialization | API / Backend (Rust) | — | `serde::Serialize` on DTO types; Tauri serializes to JSON automatically |
| print_buffer drain | API / Backend (Rust) | — | Must happen synchronously after dispatch, before response is returned |
| Multi-step modal sequencing | Frontend Server (React) | — | D-08: Phase 15 responsibility; Phase 14 IPC is stateless |
| Error surface to frontend | API / Backend (Rust) | — | `Result<CalcStateView, GuiError>` — Tauri maps Err to rejected Promise |

---

## Standard Stack

### Core (all already present in hp41-gui/src-tauri/Cargo.toml)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.11.1 [VERIFIED: Cargo.lock] | Rust command framework | Already installed Phase 13 |
| serde | 1.x [VERIFIED: Cargo.toml] | Serialize CalcStateView / GuiError | Already present |
| serde_json | 1.x [VERIFIED: Cargo.toml] | JSON serialization | Already present |
| hp41-core | path dep [VERIFIED: Cargo.toml] | All calculator logic | Already present |

**No new Cargo dependencies are needed for Phase 14.** All required libraries are already in `hp41-gui/src-tauri/Cargo.toml`.

### Supporting (from hp41-core)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| hp41_core::ops::dispatch | internal | Execute Op against CalcState | Every dispatch_op call |
| hp41_core::ops::flush_entry_buf | internal | Flush digit entry buffer | Called automatically inside dispatch() |
| hp41_core::format_hpnum | internal | Format HpNum for display | Build display_str in CalcStateView |
| hp41_core::format_alpha | internal | Format alpha register | When alpha_mode is true |

---

## Architecture Patterns

### System Architecture Diagram

```
Frontend (React)
     |
     | invoke("dispatch_op", { keyId: "plus" })
     v
Tauri IPC Layer (src-tauri/src/commands.rs)
     |
     | 1. Lock AppState = Mutex<CalcState>
     | 2. key_map::resolve(key_id) -> Result<Op, GuiError>
     | 3. hp41_core::ops::dispatch(&mut state, op) -> Result<(), HpError>
     | 4. drain state.print_buffer -> Vec<String>
     | 5. build CalcStateView from &state
     | 6. return Ok(CalcStateView) or Err(GuiError)
     v
Frontend receives CalcStateView JSON (< 300 bytes)
  { display_str, x_str, annunciators: { user, prgm, alpha, rad, grad }, print_lines }
```

### Recommended Project Structure

```
hp41-gui/src-tauri/src/
├── lib.rs          # AppState alias, Tauri builder — updated to register commands
├── main.rs         # Binary entry point — unchanged
├── types.rs        # CalcStateView, Annunciators, GuiError structs
├── key_map.rs      # resolve(key_id: &str) -> Result<Op, GuiError>
└── commands.rs     # dispatch_op() and get_state() Tauri command handlers
```

### Pattern 1: Tauri v2 Command with State Extractor

**What:** Declare Rust functions with `#[tauri::command]` that accept `tauri::State<T>` for managed state injection.
**When to use:** Both `dispatch_op` and `get_state` commands.

```rust
// Source: https://v2.tauri.app/develop/calling-rust/
use tauri::State;
use crate::types::{CalcStateView, GuiError};
use crate::AppState;

#[tauri::command]
pub fn dispatch_op(
    key_id: &str,
    state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    let op = crate::key_map::resolve(key_id)?;
    hp41_core::ops::dispatch(&mut calc, op).map_err(GuiError::from)?;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(&calc, print_lines))
}

#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(&calc, print_lines))
}
```

**Key note:** The `State<'_, AppState>` lifetime annotation `'_` is required in Tauri v2 for the extractor to compile. [VERIFIED: Tauri v2 docs]

### Pattern 2: Tauri Builder Command Registration

**What:** Register commands in `tauri::Builder` via `generate_handler![]`.
**When to use:** `lib.rs` after adding modules.

```rust
// Source: https://v2.tauri.app/develop/calling-rust/
mod commands;
mod key_map;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(hp41_core::CalcState::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::dispatch_op,
            commands::get_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
```

### Pattern 3: CalcStateView and Annunciators Construction

**What:** Derive DTO from CalcState fields — no logic, only reads.
**When to use:** `types.rs` — called after every dispatch.

```rust
// Source: verified against hp41-core/src/state.rs field names
use serde::Serialize;
use hp41_core::{CalcState, AngleMode, format_hpnum, format_alpha};

#[derive(Debug, Serialize)]
pub struct Annunciators {
    pub user: bool,
    pub prgm: bool,
    pub alpha: bool,
    pub rad:   bool,
    pub grad:  bool,
}

#[derive(Debug, Serialize)]
pub struct CalcStateView {
    pub display_str: String,
    pub x_str:       String,
    pub annunciators: Annunciators,
    pub print_lines: Vec<String>,
}

impl CalcStateView {
    pub fn from_state(state: &CalcState, print_lines: Vec<String>) -> Self {
        let display_str = if !state.entry_buf.is_empty() {
            state.entry_buf.clone()
        } else if state.alpha_mode {
            format_alpha(&state.alpha_reg)
        } else {
            format_hpnum(&state.stack.x, &state.display_mode)
        };
        CalcStateView {
            display_str,
            x_str: format_hpnum(&state.stack.x, &state.display_mode),
            annunciators: Annunciators {
                user: state.user_mode,
                prgm: state.prgm_mode,
                alpha: state.alpha_mode,
                rad:  state.angle_mode == AngleMode::Rad,
                grad: state.angle_mode == AngleMode::Grad,
            },
            print_lines,
        }
    }
}
```

**Note on display_str vs. CLI:** The CLI's `get_display_string()` also handles `prgm_mode` (shows step number). Phase 14 omits this for simplicity since Phase 15 will own PRGM display. Claude's discretion to include it or not.

### Pattern 4: GuiError with HpError Conversion

**What:** Minimal error DTO — Tauri requires `Serialize` on the error type for `Result<T, E>` commands.
**When to use:** `types.rs`.

```rust
// Source: https://v2.tauri.app/develop/calling-rust/ (error handling section)
use serde::Serialize;
use hp41_core::HpError;

#[derive(Debug, Serialize)]
pub struct GuiError {
    pub message: String,
}

impl From<HpError> for GuiError {
    fn from(e: HpError) -> Self {
        GuiError { message: e.to_string() }
    }
}
```

`HpError` uses `#[derive(thiserror::Error)]` and implements `Display` — `.to_string()` gives the display string (e.g., `"overflow"`, `"divide by zero"`). [VERIFIED: hp41-core/src/error.rs]

### Pattern 5: key_map.rs Structure

**What:** String key ID to Op resolver. Digit keys append to entry_buf; named keys map to Op variants; compound keys are parsed with prefix matching.
**When to use:** `key_map.rs` — called from `dispatch_op` before `dispatch()`.

```rust
// Source: modeled on hp41-cli/src/keys.rs key_to_op() [VERIFIED]
use hp41_core::ops::{Op, StoArithKind, StackReg};
use crate::types::GuiError;

/// Resolve a string key ID to an Op, or GuiError for unknown keys.
/// Digit keys ("0"-"9", ".", "e") are handled DIFFERENTLY from the CLI:
/// the frontend sends them as key IDs; key_map.rs appends directly to entry_buf
/// by returning a special Op::PushNum or by the caller mutating entry_buf.
///
/// Decision D-04: digit keys send as bare character strings.
/// The handler in commands.rs must special-case digits before calling resolve().
pub fn resolve(key_id: &str) -> Result<Op, GuiError> {
    match key_id {
        // Stack
        "enter"   => Ok(Op::Enter),
        "clx"     => Ok(Op::Clx),
        "chs"     => Ok(Op::Chs),
        "rdn"     => Ok(Op::Rdn),
        "xy_swap" => Ok(Op::XySwap),
        "lastx"   => Ok(Op::Lastx),
        // Arithmetic
        "plus"  => Ok(Op::Add),
        "minus" => Ok(Op::Sub),
        "mul"   => Ok(Op::Mul),
        "div"   => Ok(Op::Div),
        // Math
        "sqrt"    => Ok(Op::Sqrt),
        "sq"      => Ok(Op::Sq),
        "ypow"    => Ok(Op::YPow),
        "recip"   => Ok(Op::Recip),
        "ln"      => Ok(Op::Ln),
        "log"     => Ok(Op::Log),
        "exp"     => Ok(Op::Exp),
        "tenpow"  => Ok(Op::TenPow),
        "int"     => Ok(Op::Int),
        // Trig
        "sin"  => Ok(Op::Sin),
        "cos"  => Ok(Op::Cos),
        "tan"  => Ok(Op::Tan),
        "asin" => Ok(Op::Asin),
        "acos" => Ok(Op::Acos),
        "atan" => Ok(Op::Atan),
        // Angle mode
        "set_deg"  => Ok(Op::SetDeg),
        "set_rad"  => Ok(Op::SetRad),
        "set_grad" => Ok(Op::SetGrad),
        // Registers
        "clreg" => Ok(Op::Clreg),
        "sto_m" => Ok(Op::StoM),
        "sto_n" => Ok(Op::StoN),
        "sto_o" => Ok(Op::StoO),
        "rcl_m" => Ok(Op::RclM),
        "rcl_n" => Ok(Op::RclN),
        "rcl_o" => Ok(Op::RclO),
        // ALPHA
        "alpha_toggle"    => Ok(Op::AlphaToggle),
        "alpha_clear"     => Ok(Op::AlphaClear),
        "alpha_backspace" => Ok(Op::AlphaBackspace),
        // Programming
        "prgm_mode" => Ok(Op::PrgmMode),
        "rtn"       => Ok(Op::Rtn),
        "null"      => Ok(Op::Null),
        "getkey"    => Ok(Op::GetKey),
        // User mode
        "user_mode" => Ok(Op::UserMode),
        // Stats
        "sigma_plus"    => Ok(Op::SigmaPlus),
        "sigma_minus"   => Ok(Op::SigmaMinus),
        "mean"          => Ok(Op::Mean),
        "sdev"          => Ok(Op::Sdev),
        "lr"            => Ok(Op::LR),
        "yhat"          => Ok(Op::Yhat),
        "corr"          => Ok(Op::Corr),
        "cl_sigma_stat" => Ok(Op::ClSigmaStat),
        // HMS
        "hms_to_h" => Ok(Op::HmsToH),
        "h_to_hms" => Ok(Op::HToHms),
        "hms_add"  => Ok(Op::HmsAdd),
        "hms_sub"  => Ok(Op::HmsSub),
        // Print
        "prx"   => Ok(Op::PRX),
        "pra"   => Ok(Op::PRA),
        "prstk" => Ok(Op::PRSTK),
        // Parameterized ops: prefix-matched below
        _ => resolve_parameterized(key_id),
    }
}

fn resolve_parameterized(key_id: &str) -> Result<Op, GuiError> {
    // "sto_NN" -> StoReg(nn), "rcl_NN" -> RclReg(nn)
    if let Some(rest) = key_id.strip_prefix("sto_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::StoReg(n));
        }
    }
    if let Some(rest) = key_id.strip_prefix("rcl_") {
        if let Ok(n) = rest.parse::<u8>() {
            return Ok(Op::RclReg(n));
        }
    }
    // "fix_N", "sci_N", "eng_N"
    if let Some(rest) = key_id.strip_prefix("fix_") {
        if let Ok(n) = rest.parse::<u8>() { return Ok(Op::FmtFix(n)); }
    }
    if let Some(rest) = key_id.strip_prefix("sci_") {
        if let Ok(n) = rest.parse::<u8>() { return Ok(Op::FmtSci(n)); }
    }
    if let Some(rest) = key_id.strip_prefix("eng_") {
        if let Ok(n) = rest.parse::<u8>() { return Ok(Op::FmtEng(n)); }
    }
    // "sto_arith_<op>_<reg>"
    if let Some(rest) = key_id.strip_prefix("sto_arith_") {
        return resolve_sto_arith(rest);
    }
    // "isg_NN", "dse_NN"
    if let Some(rest) = key_id.strip_prefix("isg_") {
        if let Ok(n) = rest.parse::<u8>() { return Ok(Op::Isg(n)); }
    }
    if let Some(rest) = key_id.strip_prefix("dse_") {
        if let Ok(n) = rest.parse::<u8>() { return Ok(Op::Dse(n)); }
    }
    // "gto_<label>", "xeq_<label>", "lbl_<name>"
    if let Some(rest) = key_id.strip_prefix("gto_") {
        return Ok(Op::Gto(rest.to_string()));
    }
    if let Some(rest) = key_id.strip_prefix("xeq_") {
        return Ok(Op::Xeq(rest.to_string()));
    }
    if let Some(rest) = key_id.strip_prefix("lbl_") {
        return Ok(Op::Lbl(rest.to_string()));
    }
    // "alpha_<char>" — single character
    if let Some(rest) = key_id.strip_prefix("alpha_") {
        let mut chars = rest.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            return Ok(Op::AlphaAppend(ch));
        }
    }
    Err(GuiError { message: format!("unknown key: {key_id}") })
}

fn resolve_sto_arith(rest: &str) -> Result<Op, GuiError> {
    // Format: "<op>_<reg>" where <op> in {plus,minus,mul,div}
    // and <reg> is NN (0-99) or {y,z,t,lastx}
    let (kind_str, reg_str) = rest
        .rsplit_once('_')
        .ok_or_else(|| GuiError { message: format!("unknown key: sto_arith_{rest}") })?;
    let kind = match kind_str {
        "plus"  => StoArithKind::Add,
        "minus" => StoArithKind::Sub,
        "mul"   => StoArithKind::Mul,
        "div"   => StoArithKind::Div,
        _ => return Err(GuiError { message: format!("unknown key: sto_arith_{rest}") }),
    };
    if let Ok(n) = reg_str.parse::<u8>() {
        return Ok(Op::StoArith { reg: n, kind });
    }
    let stack_reg = match reg_str {
        "y"     => StackReg::Y,
        "z"     => StackReg::Z,
        "t"     => StackReg::T,
        "lastx" => StackReg::LastX,
        _ => return Err(GuiError { message: format!("unknown key: sto_arith_{rest}") }),
    };
    Ok(Op::StoArithStack { kind, stack_reg })
}
```

### Pattern 6: Digit Key Handling in dispatch_op

**What:** Digit keys ("0"-"9", ".", "e") append to `entry_buf` rather than producing an `Op`. This mirrors the CLI's digit-entry block in `app.handle_key()`.
**When to use:** The `dispatch_op` command handler before calling `key_map::resolve()`.

```rust
// Source: modeled on hp41-cli/src/app.rs digit entry block [VERIFIED]
// Important: flush_entry_buf() is called INSIDE dispatch() automatically — callers
// that only append to entry_buf do NOT call flush_entry_buf() directly.
// The NEXT non-digit dispatch() call will flush it.

#[tauri::command]
pub fn dispatch_op(key_id: &str, state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    
    // Handle digit keys by appending to entry_buf (no Op dispatch)
    match key_id {
        "0"|"1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9" => {
            let c = key_id.chars().next().expect("single-char key_id is non-empty");
            // Cap exponent entry at 2 digits (mirrors CLI Phase 9 guard)
            if let Some(e_pos) = calc.entry_buf.find('e') {
                let after_e = &calc.entry_buf[e_pos + 1..];
                let exp_digits = after_e.chars().filter(|ch| ch.is_ascii_digit()).count();
                if exp_digits >= 2 {
                    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
                    return Ok(CalcStateView::from_state(&calc, print_lines));
                }
            }
            calc.entry_buf.push(c);
            let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
            return Ok(CalcStateView::from_state(&calc, print_lines));
        }
        "." => {
            if !calc.entry_buf.contains('.') && !calc.entry_buf.contains('e') {
                calc.entry_buf.push('.');
            }
            let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
            return Ok(CalcStateView::from_state(&calc, print_lines));
        }
        "e" => {
            if !calc.entry_buf.contains('e') {
                if calc.entry_buf.is_empty() {
                    calc.entry_buf.push_str("1e");
                } else {
                    calc.entry_buf.push('e');
                }
            }
            let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
            return Ok(CalcStateView::from_state(&calc, print_lines));
        }
        _ => {}
    }
    
    // Named ops and parameterized ops
    let op = crate::key_map::resolve(key_id)?;
    hp41_core::ops::dispatch(&mut calc, op).map_err(GuiError::from)?;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(&calc, print_lines))
}
```

### Pattern 7: Capability JSON for App Commands

**What:** Tauri v2 — by default, **all commands registered via `invoke_handler` are allowed from all windows** without explicit capability entries. [VERIFIED: v2.tauri.app/security/capabilities/]

**However**, D-12 requires explicit IPC permission entries. The confirmed format for non-plugin app commands is that only `${permission-name}` is required (no plugin namespace prefix). [VERIFIED: v2.tauri.app/reference/acl/capability/]

```json
{
  "identifier": "default",
  "description": "Default capability for hp41-gui",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "allow-dispatch-op",
    "allow-get-state"
  ]
}
```

**Important note:** The `allow-dispatch-op` and `allow-get-state` permissions are **auto-generated by the Tauri build system** at build time. They appear in `src-tauri/gen/schemas/` and `permissions/autogenerated/commands/`. The planner should be aware that these entries will only resolve after the first `cargo tauri dev` / `cargo build` run that includes the registered commands.

**Alternative (simpler) approach:** Because Tauri v2 allows all registered commands by default, the capability update is technically optional for functionality but satisfies D-12's explicit-permission requirement. If the auto-generated permissions are not present yet, the build will warn or error — in that case, run `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` first to generate them.

### Anti-Patterns to Avoid

- **Calling `flush_entry_buf()` directly in the command handler:** `dispatch()` already calls it internally at the start. Double-flushing is harmless but unnecessary for named ops. For digit keys, do NOT call dispatch() — only append to entry_buf.
- **Using `.unwrap()` on `state.lock()`:** Violates `#![deny(clippy::unwrap_used)]`. Always use `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery.
- **Returning `HpError` directly from commands:** `HpError` does not implement `Serialize`. Must convert to `GuiError` via `From` impl.
- **Duplicating format_hpnum logic:** Import `hp41_core::format_hpnum` — do not reimplement formatting.
- **Making CalcState or Op public on the frontend:** Phase 14 is purely Rust-side. The frontend only sees JSON.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Number formatting | Custom formatter | `hp41_core::format_hpnum` | Already 10-digit HP-41 faithful; format.rs has FIX/SCI/ENG modes |
| Alpha display | Custom truncation | `hp41_core::format_alpha` | Handles 12-char truncation per HP-41 spec |
| Calculator logic | Any duplicate dispatch | `hp41_core::ops::dispatch` | SC-4 prohibits ANY duplicate logic in hp41-gui |
| Mutex recovery | Panic on poisoned lock | `.unwrap_or_else(\|e\| e.into_inner())` | Zero-panic policy; poisoned locks are recoverable |
| Error serialization | Manual JSON error | `#[derive(Serialize)] struct GuiError` | Tauri requires Serialize on error type for Result<T,E> commands |

**Key insight:** The entire calculator is already in `hp41-core`. Phase 14's job is to route strings to `dispatch()` and convert the resulting state into a JSON-serializable DTO.

---

## Runtime State Inventory

> Not applicable — Phase 14 is a greenfield addition (new modules, new commands). No rename/migration involved.

None — verified by phase description. No existing stored data, live service config, OS-registered state, secrets, or build artifacts affected.

---

## Common Pitfalls

### Pitfall 1: Borrow Conflict When Draining print_buffer After dispatch()

**What goes wrong:** If `CalcStateView::from_state(&calc, ...)` tries to read `calc.print_buffer` while also draining it in the same expression.
**Why it happens:** Rust borrow checker prevents simultaneous mutable and shared borrows.
**How to avoid:** Drain first, then pass the drained vec to `from_state()`:
```rust
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
Ok(CalcStateView::from_state(&calc, print_lines))
```
The drain takes a mutable borrow of `print_buffer` only; `from_state` takes `&CalcState` which does not borrow `print_buffer` mutably. Since drain completes before `from_state` is called, there is no conflict. [VERIFIED: CLI Phase 11 uses identical pattern]

### Pitfall 2: Auto-generated Permissions Not Available Until First Build

**What goes wrong:** Adding `"allow-dispatch-op"` and `"allow-get-state"` to `capabilities/default.json` before the first build will cause a build error: "unknown permission identifier".
**Why it happens:** Tauri generates the `allow-*` permissions from the registered commands at `tauri::generate_handler![]` scan time during `tauri-build`. The permissions don't exist until the build runs.
**How to avoid:** In the plan, the capability update should come AFTER the commands are registered and the first `gui-check` or `gui-dev` succeeds. Alternatively, add the commands and skip the capability entries in Wave 1; add capability entries in Wave 2 after build confirms they exist.

### Pitfall 3: StoArith Compound Key Parsing Ambiguity

**What goes wrong:** `"sto_arith_plus_05"` — using `split_once('_')` from the left gives `("sto", "arith_plus_05")` instead of the expected `("sto_arith", "plus_05")`.
**Why it happens:** The key ID has multiple `_` separators.
**How to avoid:** Strip the `"sto_arith_"` prefix first, then use `rsplit_once('_')` on the remainder to separate `<op>` from `<reg>` — because the register (NN, y, z, t, lastx) is always the last segment. Example: `"plus_05"` → rsplit at `_` → `("plus", "05")`. The pattern in Pattern 5 above implements this correctly.

### Pitfall 4: CHS During EEX Entry (entry_buf contains 'e-' or 'e')

**What goes wrong:** If key_id `"chs"` dispatches `Op::Chs` when `entry_buf` contains `'e'`, `flush_entry_buf()` will flush the entry before CHS executes. The CLI handles this with special-case logic in `app.handle_key()` for `'n'` with an active exponent.
**Why it happens:** The CLI's 'n' key + EEX handling toggles the exponent sign IN the entry_buf rather than dispatching via `dispatch()`.
**How to avoid (D-08 scope):** Phase 14 does not need to replicate the CHS-in-exponent path since Phase 14 is IPC-only and the frontend sends explicit key IDs. When the frontend sends `"chs"` during exponent entry, `flush_entry_buf()` inside `dispatch()` will flush first (committing the partial exponent as 00 per Phase 9 normalization), then CHS will negate the result. This is slightly different from the CLI behavior but acceptable for Phase 14. **Flag for Phase 15** to handle the special case if needed.

### Pitfall 5: `#![deny(clippy::unwrap_used)]` at lib.rs Applies to All Submodules

**What goes wrong:** Using `.unwrap()` anywhere in `types.rs`, `key_map.rs`, or `commands.rs` triggers a compile error.
**Why it happens:** The `#![deny(clippy::unwrap_used)]` attribute at the top of `lib.rs` applies to the entire crate.
**How to avoid:** Use `.expect("reason")` for infallible operations, `?`-propagation for Results, and `.unwrap_or_else(|e| e.into_inner())` for Mutex. Tests can add `#[allow(clippy::unwrap_used)]` at the test module level.

### Pitfall 6: GuiError Must Implement Serialize (Not just Display)

**What goes wrong:** A Tauri `#[tauri::command]` returning `Result<T, E>` requires `E: Serialize`. `std::error::Error` or `Display` alone is not sufficient — Tauri will fail to compile if the error type lacks `Serialize`.
**Why it happens:** Tauri serializes the error to JSON so the frontend receives it as a rejected Promise payload.
**How to avoid:** `#[derive(Debug, Serialize)]` on `GuiError`. Do NOT add `impl std::error::Error for GuiError` (not needed, and would require implementing `Display`). [VERIFIED: Tauri v2 docs pattern]

---

## Code Examples

### CalcState Fields Used for CalcStateView

```rust
// Source: hp41-core/src/state.rs [VERIFIED]
// Fields accessed by from_state():
state.entry_buf         // String — pending digit input; display priority 1
state.alpha_mode        // bool — display priority 2
state.alpha_reg         // String — shown when alpha_mode; via format_alpha()
state.stack.x           // HpNum — formatted via format_hpnum() for display/x_str
state.display_mode      // DisplayMode — Fix(u8)/Sci(u8)/Eng(u8) — passed to format_hpnum
state.user_mode         // bool -> annunciators.user
state.prgm_mode         // bool -> annunciators.prgm
state.angle_mode        // AngleMode::{Deg,Rad,Grad} -> annunciators.rad, annunciators.grad
state.print_buffer      // Vec<String> — drained; NOT read by from_state (passed as arg)
```

### Key ID Mapping Summary (Named Ops)

This is the complete named-op mapping derived from `hp41-cli/src/keys.rs::key_to_op()` [VERIFIED]:

| key_id | Op | CLI binding | Notes |
|--------|----|-------------|-------|
| `"enter"` | `Op::Enter` | Enter key | |
| `"clx"` | `Op::Clx` | Backspace | |
| `"chs"` | `Op::Chs` | `'n'` | |
| `"rdn"` | `Op::Rdn` | `'r'` | |
| `"xy_swap"` | `Op::XySwap` | `'x'` | |
| `"lastx"` | `Op::Lastx` | `'l'` | |
| `"plus"` | `Op::Add` | `'+'` | |
| `"minus"` | `Op::Sub` | `'-'` | |
| `"mul"` | `Op::Mul` | `'*'` | |
| `"div"` | `Op::Div` | `'/'` | |
| `"sqrt"` | `Op::Sqrt` | `'s'` | |
| `"sin"` | `Op::Sin` | `'q'` | Phase 8 reassignment |
| `"cos"` | `Op::Cos` | `'C'` | |
| `"tan"` | `Op::Tan` | `'T'` | |
| `"asin"` | `Op::Asin` | `'a'` | |
| `"acos"` | `Op::Acos` | `'c'` | |
| `"atan"` | `Op::Atan` | `'k'` | |
| `"ln"` | `Op::Ln` | `'L'` | |
| `"log"` | `Op::Log` | `'G'` | |
| `"exp"` | `Op::Exp` | `'E'` | |
| `"tenpow"` | `Op::TenPow` | `'H'` | |
| `"recip"` | `Op::Recip` | `'I'` | |
| `"sq"` | `Op::Sq` | `'W'` | |
| `"ypow"` | `Op::YPow` | `'Y'` | |
| `"int"` | `Op::Int` | — | No CLI binding |
| `"clreg"` | `Op::Clreg` | `'g'` | Phase 8 |
| `"user_mode"` | `Op::UserMode` | `'u'` | |
| `"prgm_mode"` | `Op::PrgmMode` | `'p'` | |
| `"rtn"` | `Op::Rtn` | — | |
| `"alpha_toggle"` | `Op::AlphaToggle` | Ctrl+A | |
| `"alpha_clear"` | `Op::AlphaClear` | Delete (ALPHA mode) | |
| `"alpha_backspace"` | `Op::AlphaBackspace` | Backspace (ALPHA mode) | |
| `"sigma_plus"` | `Op::SigmaPlus` | `'z'` | |
| `"sigma_minus"` | `Op::SigmaMinus` | `'Z'` | |
| `"mean"` | `Op::Mean` | `'m'` | |
| `"sdev"` | `Op::Sdev` | `'D'` | |
| `"lr"` | `Op::LR` | `'b'` | |
| `"yhat"` | `Op::Yhat` | `'y'` | |
| `"corr"` | `Op::Corr` | `'O'` | |
| `"cl_sigma_stat"` | `Op::ClSigmaStat` | `'V'` | |
| `"hms_to_h"` | `Op::HmsToH` | `'h'` | |
| `"h_to_hms"` | `Op::HToHms` | `'F'` (intercepted) | |
| `"hms_add"` | `Op::HmsAdd` | `'j'` | |
| `"hms_sub"` | `Op::HmsSub` | `'J'` | |
| `"prx"` | `Op::PRX` | `'P'` modal | |
| `"pra"` | `Op::PRA` | `'P'` modal | |
| `"prstk"` | `Op::PRSTK` | `'P'` modal | |
| `"null"` | `Op::Null` | — | Synthetic programming |
| `"getkey"` | `Op::GetKey` | — | Synthetic programming |
| `"set_deg"` | `Op::SetDeg` | `'d'` (cycle) | |
| `"set_rad"` | `Op::SetRad` | `'d'` (cycle) | |
| `"set_grad"` | `Op::SetGrad` | `'d'` (cycle) | |

**Ops NOT mapped in Phase 14 (require compound IDs):**
`Op::StoReg(u8)`, `Op::RclReg(u8)`, `Op::FmtFix(u8)`, `Op::FmtSci(u8)`, `Op::FmtEng(u8)`, `Op::StoArith{...}`, `Op::StoArithStack{...}`, `Op::Isg(u8)`, `Op::Dse(u8)`, `Op::Lbl(String)`, `Op::Gto(String)`, `Op::Xeq(String)`, `Op::AlphaAppend(char)`, `Op::SyntheticByte(u8)`, `Op::PushNum(HpNum)` (handled via digit entry_buf path)

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 `#[tauri::command]` + `AppHandle` | Tauri v2 `tauri::State<T>` extractor directly in function signature | Tauri v2.0 | Cleaner ergonomics; no need to call `app.state()` manually |
| All commands allowed by default | Explicit capability-based ACL | Tauri v2.0 security model | Commands need `allow-*` entries in capabilities JSON |
| Tauri v1 plugin permissions | `${permission-name}` without prefix for app commands | Tauri v2.0 | Simpler for non-plugin commands |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `"allow-dispatch-op"` and `"allow-get-state"` are the auto-generated permission names for commands named `dispatch_op` and `get_state` | Tauri Capabilities pattern | Build error; would need to find actual generated names in `gen/schemas/` |
| A2 | Auto-generated permissions appear after first `cargo build` / `cargo check` in `hp41-gui/src-tauri/gen/` | Pitfall 2 | Plan may need to reorder: register commands before adding capability entries |
| A3 | `op_chs` during entry_buf with 'e' flushes the exponent first (slightly different from CLI) | Pitfall 4 | Minor UX difference; acceptable for Phase 14 per D-08 scope |

**High-risk assumption:** A1 and A2 — the permission naming and generation timing. The planner should structure Wave order so commands are registered and built before capability entries are added.

---

## Open Questions

1. **Capability permission names for non-plugin commands**
   - What we know: For plugin commands, format is `plugin-name:allow-command`. For app commands, only `${permission-name}` is required. Tauri generates `allow-<kebab-case-command-name>` automatically.
   - What's unclear: Whether the permission is `"allow-dispatch-op"` or `"allow-dispatch_op"` (snake vs kebab). The Tauri ACL docs say identifiers are limited to `[a-z]` and use kebab-case, suggesting `"allow-dispatch-op"`.
   - Recommendation: After the first successful `gui-check`, inspect `hp41-gui/src-tauri/gen/schemas/desktop-schema.json` to confirm the exact permission names. If confirmation cannot wait, omit capability entries initially and add them in a later plan step.

2. **display_str for PRGM mode**
   - What we know: The CLI's `get_display_string` shows step number in PRGM mode. Phase 14 CONTEXT says IPC is plumbing only.
   - What's unclear: Should `display_str` in `CalcStateView` show step info when `prgm_mode` is true, or just raw X?
   - Recommendation: Return `format_hpnum(&state.stack.x, &state.display_mode)` always in Phase 14 (simplest). Phase 15 can add PRGM display logic when it renders the UI.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Tauri build | ✓ | 1.89.0 [VERIFIED] | — |
| Cargo | Build system | ✓ | 1.89.0 [VERIFIED] | — |
| Node.js | Tauri dev server | ✓ | v22.16.0 [VERIFIED] | — |
| npm | Package management | ✓ | 10.9.2 [VERIFIED] | — |
| @tauri-apps/api | Frontend IPC | ✓ | 2.11.0 [VERIFIED] | — |
| @tauri-apps/cli | Dev server / build | ✓ (npm) | 2.11.x [VERIFIED: package.json] | — |
| hp41-core | Calculator logic | ✓ | path dep [VERIFIED: Cargo.toml] | — |
| tauri 2.11.1 | Rust IPC framework | ✓ | 2.11.1 [VERIFIED: Cargo.lock] | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None. All required tooling is present.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `#[test]` (no extra crate needed) + `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Config file | None (Cargo workspace handles it) |
| Quick run command | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Full suite command | `just test` (hp41-core + hp41-cli) + `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |

**Note on Tauri test harness:** Tauri v2 has a `tauri::test` module for integration testing commands, but it requires WebView spin-up which is expensive and platform-specific. For Phase 14 (pure Rust-side IPC plumbing), `#[test]` on `key_map::resolve()` and `CalcStateView::from_state()` as unit tests is sufficient and faster. The command handlers themselves can be tested by calling them directly with a `tauri::test::mock_app()`, but this is optional for Phase 14.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IPC-01/SC-1 | dispatch_op("plus") returns CalcStateView with JSON size ≤300 bytes | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_dispatch_op_payload_size` | ❌ Wave 0 |
| IPC-01/SC-2 | dispatch_op("unknown_key") returns GuiError, not panic | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_dispatch_op_unknown_key` | ❌ Wave 0 |
| IPC-01/SC-3 | print_buffer drain — PRX populates print_lines in CalcStateView | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml test_print_buffer_drained` | ❌ Wave 0 |
| IPC-01/SC-4 | No calculator logic in hp41-gui | static audit | `grep -r "fn op_" hp41-gui/src-tauri/src/ \|\| true` | manual |
| IPC-01/SC-5 | AppState = Mutex<CalcState> used throughout | compile | `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` | compile gate |

### SC-1 Verification Strategy (≤300 bytes)

```rust
#[test]
fn test_dispatch_op_payload_size() {
    use hp41_core::{CalcState, ops::{dispatch, Op}};
    let mut state = CalcState::new();
    dispatch(&mut state, Op::Add).unwrap();
    let print_lines: Vec<String> = state.print_buffer.drain(..).collect();
    let view = CalcStateView::from_state(&state, print_lines);
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.len() <= 300, "CalcStateView JSON must be ≤300 bytes, got {}", json.len());
}
```

### SC-3 Verification Strategy (print_buffer drain)

```rust
#[test]
fn test_print_buffer_drained() {
    use hp41_core::{CalcState, ops::{dispatch, Op}};
    let mut state = CalcState::new();
    state.stack.x = hp41_core::HpNum::from(42);
    dispatch(&mut state, Op::PRX).unwrap();
    assert!(!state.print_buffer.is_empty(), "PRX should populate print_buffer");
    let print_lines: Vec<String> = state.print_buffer.drain(..).collect();
    assert!(state.print_buffer.is_empty(), "buffer should be empty after drain");
    let view = CalcStateView::from_state(&state, print_lines);
    assert_eq!(view.print_lines.len(), 1, "print_lines should have 1 entry");
}
```

### Sampling Rate
- **Per task commit:** `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Per wave merge:** `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **Phase gate:** Full suite + `just gui-check` green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `hp41-gui/src-tauri/src/tests/mod.rs` or inline `#[cfg(test)]` blocks in `types.rs`, `key_map.rs`, `commands.rs` — covers SC-1, SC-2, SC-3
- [ ] No new test framework install needed — standard Rust `#[test]` works

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | n/a — local desktop app, no auth |
| V3 Session Management | no | n/a — single-user desktop |
| V4 Access Control | partial | Tauri capabilities ACL — commands locked to `"main"` window |
| V5 Input Validation | yes | `key_map::resolve()` validates all key IDs; unknown IDs return GuiError, never panic |
| V6 Cryptography | no | n/a — no crypto in IPC layer |

### Known Threat Patterns for Tauri v2 + Rust

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Arbitrary command invocation from injected scripts | Elevation of Privilege | Tauri capabilities window scoping (`"windows": ["main"]`); no wildcard |
| Panic in command handler crashes app | Denial of Service | `#![deny(clippy::unwrap_used)]`; poisoned lock recovery; GuiError return instead of panic |
| Entry_buf injection via crafted key_id | Tampering | key_map.rs only appends valid single chars to entry_buf; no eval/exec path |
| Unknown key IDs silently discarded | Information Disclosure | D-07: ALL unknown keys return GuiError, never silently ignored |

---

## Sources

### Primary (HIGH confidence)
- `hp41-core/src/state.rs` — CalcState fields verified: `user_mode`, `prgm_mode`, `alpha_mode`, `angle_mode`, `entry_buf`, `print_buffer`, `stack.x`, `display_mode`
- `hp41-core/src/ops/mod.rs` — Op enum complete list (Phase 1–12), `dispatch()` signature, `flush_entry_buf()` behavior
- `hp41-core/src/format.rs` — `format_hpnum()`, `format_alpha()` — public, accessible from hp41-gui
- `hp41-core/src/error.rs` — `HpError` variants with `thiserror::Error` derive; `.to_string()` available
- `hp41-core/src/lib.rs` — Public re-exports confirmed: `format_hpnum`, `format_alpha`, `HpError`, `CalcState`, `AngleMode`
- `hp41-gui/src-tauri/src/lib.rs` — `AppState = Mutex<hp41_core::CalcState>` already defined; `invoke_handler![]` placeholder confirmed
- `hp41-gui/src-tauri/Cargo.toml` — tauri 2.11, serde, serde_json, hp41-core path dep — no new deps needed
- `hp41-gui/src-tauri/Cargo.lock` — tauri v2.11.1 [VERIFIED]
- `hp41-cli/src/keys.rs` — Complete `key_to_op()` reference — all named key → Op mappings extracted
- `hp41-cli/src/app.rs` — Digit entry_buf logic (lines 342–388); print_buffer drain pattern (`call_dispatch_and_drain`)
- [Tauri v2 commands docs](https://v2.tauri.app/develop/calling-rust/) — `#[tauri::command]`, `State<'_, T>`, `generate_handler![]`, `Result<T, E>` return type, `Serialize` requirement on error

### Secondary (MEDIUM confidence)
- [Tauri v2 capabilities docs](https://v2.tauri.app/security/capabilities/) — "by default, all commands registered via invoke_handler are allowed"
- [Tauri v2 ACL reference](https://v2.tauri.app/reference/acl/capability/) — "For commands directly implemented in the application itself, only `${permission-name}` is required"
- [Tauri permissions docs](https://v2.tauri.app/security/permissions/) — auto-generated `allow-<command>` and `deny-<command>` permissions confirmed

### Tertiary (LOW confidence — needs validation)
- Exact permission names `"allow-dispatch-op"` and `"allow-get-state"` — inferred from kebab-case convention; should be confirmed in `gen/schemas/` after first build [ASSUMED]

---

## Project Constraints (from CLAUDE.md)

| Constraint | Impact on Phase 14 |
|------------|-------------------|
| `hp41-core` must never depend on `hp41-cli` or `hp41-gui` | No changes to hp41-core in Phase 14 |
| `#![deny(clippy::unwrap_used)]` active in hp41-core | Already active in lib.rs; all new hp41-gui code must also comply |
| Zero panics in `hp41-core` | GuiError wraps HpError; no .unwrap() anywhere in command handlers |
| Commits use `/git-workflow:commit --with-skills` | Research doc commit uses this workflow |
| `just` is sole task runner | Use `just gui-check`, `just gui-dev`, not bare `cargo` |
| `hp41-gui` is nested workspace (not root member) | `cargo test --workspace` will NOT run hp41-gui tests — use `--manifest-path` explicitly |

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified from Cargo.lock and Cargo.toml
- Architecture: HIGH — all CalcState fields and dispatch() signature verified from source
- Pitfalls: HIGH — borrow conflict pattern verified from Phase 11 CLI implementation; Tauri patterns verified from official docs
- Capability permission names: LOW — naming convention inferred, not confirmed from generated files

**Research date:** 2026-05-09
**Valid until:** 2026-06-09 (stable stack; Tauri v2 patch releases unlikely to break patterns)
