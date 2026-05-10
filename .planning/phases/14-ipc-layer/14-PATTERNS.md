# Phase 14: IPC Layer - Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 5 (3 new, 2 modified)
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-gui/src-tauri/src/types.rs` | model/DTO | request-response | `hp41-core/src/state.rs` (CalcState fields) + `hp41-core/src/error.rs` (HpError) | role-match |
| `hp41-gui/src-tauri/src/key_map.rs` | utility | request-response | `hp41-cli/src/keys.rs` (`key_to_op()`) | exact |
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | `hp41-cli/src/app.rs` (`call_dispatch_and_drain()`, digit entry block) | role-match |
| `hp41-gui/src-tauri/src/lib.rs` (modify) | config | request-response | itself (existing skeleton) | exact |
| `hp41-gui/src-tauri/capabilities/default.json` (modify) | config | — | itself (existing `core:default` entry) | exact |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/types.rs` (model/DTO, request-response)

**Analogs:** `hp41-core/src/state.rs` (fields to read), `hp41-core/src/error.rs` (HpError Display)

**Imports pattern** — required imports for types.rs:
```rust
use serde::Serialize;
use hp41_core::{CalcState, AngleMode, format_hpnum, format_alpha};
use hp41_core::HpError;
```

**State fields used to build CalcStateView** — from `hp41-core/src/state.rs` lines 59–101:
```rust
// Priority-ordered display fields:
state.entry_buf         // String  — pending digit input; display priority 1 (non-empty → show it)
state.alpha_mode        // bool    — display priority 2 (true → format_alpha)
state.alpha_reg         // String  — shown when alpha_mode; via format_alpha()
state.stack.x           // HpNum  — normal display; via format_hpnum(&state.stack.x, &state.display_mode)
state.display_mode      // DisplayMode — Fix(u8)/Sci(u8)/Eng(u8) — passed to format_hpnum
// Annunciators:
state.user_mode         // bool   — annunciators.user
state.prgm_mode         // bool   — annunciators.prgm
state.alpha_mode        // bool   — annunciators.alpha
state.angle_mode        // AngleMode::{Deg,Rad,Grad} — annunciators.rad, annunciators.grad
// Print drain (passed in, not read from state):
state.print_buffer      // Vec<String> — drained BEFORE from_state() is called; passed as arg
```

**HpError Display** — from `hp41-core/src/error.rs` lines 1–19 (thiserror derive):
```rust
// HpError variants and their Display strings (from #[error("...")] attrs):
// Overflow      → "overflow"
// DivideByZero  → "divide by zero"
// InvalidOp     → "invalid operation"
// Domain        → "domain error"
// CallDepth     → "try again"
// InvalidInput  → "invalid input"
// Use: e.to_string() in GuiError::from(e: HpError)
```

**Derive pattern** — both DTO structs require only `Serialize` (not `Deserialize`); `GuiError` must NOT derive `std::error::Error` (Tauri v2 constraint):
```rust
#[derive(Debug, Serialize)]
pub struct Annunciators { ... }

#[derive(Debug, Serialize)]
pub struct CalcStateView { ... }

#[derive(Debug, Serialize)]
pub struct GuiError { pub message: String }
```

**Zero-panic rule** — `hp41-gui/src-tauri/src/lib.rs` line 1:
```rust
#![deny(clippy::unwrap_used)]
// This attribute at lib.rs root applies to ALL submodules including types.rs.
// No .unwrap() anywhere. Use .expect("reason") for infallible ops, ? for Results.
```

---

### `hp41-gui/src-tauri/src/key_map.rs` (utility, request-response)

**Analog:** `hp41-cli/src/keys.rs` — `key_to_op()` (lines 18–86)

**Imports pattern** — key_map.rs uses string matching instead of KeyCode; note Op variants come from `hp41_core::ops::Op`:
```rust
// hp41-cli/src/keys.rs line 10-13 (adapted — no crossterm dependency in hp41-gui):
use hp41_core::ops::Op;
use crate::types::GuiError;
// For parameterized ops:
use hp41_core::{StoArithKind};  // re-exported from hp41-core/src/lib.rs line 18
```

**Core mapping pattern** — analog is `key_to_op()` in `hp41-cli/src/keys.rs` lines 19–86. The GUI version maps `&str` instead of `KeyCode`. Direct translation examples:
```rust
// CLI (keys.rs line 21):       KeyCode::Enter    => Some(Op::Enter)
// GUI key_map.rs:              "enter"           => Ok(Op::Enter)

// CLI (keys.rs line 22):       KeyCode::Backspace => Some(Op::Clx)
// GUI key_map.rs:              "clx"             => Ok(Op::Clx)

// CLI (keys.rs line 29):       KeyCode::Char('n') => Some(Op::Chs)
// GUI key_map.rs:              "chs"             => Ok(Op::Chs)

// CLI (keys.rs line 72):       KeyCode::Char('q') => Some(Op::Sin)   // Phase 8
// GUI key_map.rs:              "sin"             => Ok(Op::Sin)       // semantic name, not key char

// CLI (keys.rs line 73):       KeyCode::Char('g') => Some(Op::Clreg) // Phase 8
// GUI key_map.rs:              "clreg"           => Ok(Op::Clreg)
```

**Exhaustive fallthrough** — CLI returns `None` for unmapped keys; GUI returns `Err(GuiError)` (D-07). The CLI has `_ => None` (line 84); GUI has `_ => resolve_parameterized(key_id)` followed by `Err(GuiError { message: format!("unknown key: {key_id}") })`.

**Parameterized ops — strip_prefix pattern** (no CLI analog; pure string parsing):
```rust
// Strip prefix, parse remainder. Use rsplit_once for multi-segment keys (Pitfall 3 in RESEARCH.md).
if let Some(rest) = key_id.strip_prefix("sto_arith_") {
    // "plus_05" → rsplit_once('_') → ("plus", "05") — NOT split_once (would fail on "plus_05")
    let (kind_str, reg_str) = rest.rsplit_once('_')
        .ok_or_else(|| GuiError { message: format!("unknown key: sto_arith_{rest}") })?;
    ...
}
```

**Test pattern** — analog is `hp41-cli/src/keys.rs` lines 254–355. Tests call `key_to_op` directly (no App needed). key_map.rs tests call `resolve()` directly with no Tauri state needed:
```rust
// hp41-cli/src/keys.rs lines 331–341 (test_q_dispatches_sin):
#[test]
fn test_q_dispatches_sin() {
    let mut state = CalcState::new();
    state.stack.x = hp41_core::HpNum::from(30);
    let result = hp41_core::ops::dispatch(&mut state, Op::Sin);
    assert!(result.is_ok(), ...);
}
// key_map.rs equivalent: call resolve("sin"), assert Ok(Op::Sin), then dispatch
```

---

### `hp41-gui/src-tauri/src/commands.rs` (controller, request-response)

**Analog:** `hp41-cli/src/app.rs` — `call_dispatch_and_drain()` (lines 998–1019) + digit entry block (lines 342–388)

**Imports pattern**:
```rust
use tauri::State;
use crate::types::{CalcStateView, GuiError};
use crate::AppState;   // = Mutex<hp41_core::CalcState>, defined in lib.rs line 6
use hp41_core::ops::dispatch;
```

**Mutex lock pattern with poisoned-lock recovery** — from CONTEXT.md D (Claude's Discretion) and RESEARCH.md Pattern 1. Zero-.unwrap() rule requires this form:
```rust
// hp41-gui/src-tauri/src/lib.rs line 6 (AppState type):
pub type AppState = Mutex<hp41_core::CalcState>;
// In every command handler:
let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
// NOT: state.lock().unwrap() — violates #![deny(clippy::unwrap_used)]
```

**print_buffer drain pattern** — analog is `hp41-cli/src/app.rs` line 1001. Drain BEFORE calling from_state to avoid borrow conflict (RESEARCH.md Pitfall 1):
```rust
// hp41-cli/src/app.rs line 1001:
let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
// commands.rs equivalent:
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
Ok(CalcStateView::from_state(&calc, print_lines))
// CRITICAL: drain() ends its mutable borrow before from_state(&calc, ...) begins shared borrow
```

**Digit entry block pattern** — analog is `hp41-cli/src/app.rs` lines 342–388. Digits append to entry_buf WITHOUT calling dispatch(); the next non-digit op auto-flushes via `flush_entry_buf()` inside `dispatch()`:
```rust
// hp41-cli/src/app.rs lines 349–362 (digit path):
if c.is_ascii_digit() {
    if let Some(e_pos) = self.state.entry_buf.find('e') {
        let after_e = &self.state.entry_buf[e_pos + 1..];
        let exp_digit_count = after_e.chars().filter(|ch| ch.is_ascii_digit()).count();
        if exp_digit_count >= 2 {
            return; // silently block 3rd exponent digit
        }
    }
    self.state.entry_buf.push(c);
    return;  // NO dispatch() call for digits
}
// commands.rs equivalent: push to calc.entry_buf, return CalcStateView immediately
```

**'.' guard** — analog `hp41-cli/src/app.rs` lines 364–372:
```rust
if c == '.' {
    if self.state.entry_buf.contains('.') || self.state.entry_buf.contains('e') {
        return; // block duplicate '.' and '.' after 'e'
    }
    self.state.entry_buf.push('.');
    return;
}
```

**'e' (EEX) guard** — analog `hp41-cli/src/app.rs` lines 373–388:
```rust
if c == 'e' {
    if self.state.entry_buf.contains('e') { return; } // block double-EEX
    if self.state.entry_buf.is_empty() {
        self.state.entry_buf.push_str("1e"); // implicit "1" mantissa (Phase 9 D-07)
    } else {
        self.state.entry_buf.push('e');
    }
    return;
}
```

**Error propagation pattern** — `HpError` to `GuiError` via `?` operator (requires `impl From<HpError> for GuiError` in types.rs):
```rust
// hp41-cli/src/app.rs lines 999–991 (error path sets message):
Err(e) => self.message = Some(format!("{e}"))
// commands.rs equivalent uses ? operator:
dispatch(&mut calc, op).map_err(GuiError::from)?;
// or equivalently:
dispatch(&mut calc, op)?;  // if GuiError implements From<HpError>
```

**Tauri command signature** — `State<'_, AppState>` lifetime annotation is required in Tauri v2 (RESEARCH.md Pattern 1, verified from Tauri v2 docs):
```rust
#[tauri::command]
pub fn dispatch_op(key_id: &str, state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    ...
}

#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    ...
}
```

---

### `hp41-gui/src-tauri/src/lib.rs` (modify — config)

**Analog:** itself (current content, lines 1–20)

**Current state** — `hp41-gui/src-tauri/src/lib.rs` lines 1–20:
```rust
#![deny(clippy::unwrap_used)]

use std::sync::Mutex;
use tauri::Manager;

pub type AppState = Mutex<hp41_core::CalcState>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(hp41_core::CalcState::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tauri commands registered here in Phase 14
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
```

**Required modifications** — add three `mod` declarations before `run()`, and fill `generate_handler![]`:
```rust
// Add after existing use statements:
mod commands;
mod key_map;
mod types;

// Replace the placeholder comment in generate_handler![]:
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
])
```

**`tauri::Manager` import** — already present (line 4); used by `.setup()` / `app.manage()`. Keep as-is.

---

### `hp41-gui/src-tauri/capabilities/default.json` (modify — config)

**Analog:** itself (current content, lines 1–8)

**Current state** — `hp41-gui/src-tauri/capabilities/default.json` lines 1–8:
```json
{
  "identifier": "default",
  "description": "Default capability for hp41-gui",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

**Required modification** — add two auto-generated app command permissions (D-12). Per RESEARCH.md Pitfall 2 and assumption A1, add AFTER first successful `gui-check` to avoid "unknown permission identifier" build error:
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

**Build-order constraint:** These permission names are auto-generated by `tauri-build` from the `generate_handler![]` registration. They only appear in `hp41-gui/src-tauri/gen/schemas/` after the first `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`. The plan must sequence: (1) add commands to `generate_handler![]`, (2) run `gui-check`, (3) then add capability entries.

---

## Shared Patterns

### Zero-Panic Policy
**Source:** `hp41-gui/src-tauri/src/lib.rs` line 1
**Apply to:** All new files — `types.rs`, `key_map.rs`, `commands.rs`
```rust
#![deny(clippy::unwrap_used)]
// Applies to all submodules from lib.rs root.
// Compliant alternatives:
//   state.lock().unwrap_or_else(|e| e.into_inner())  // Mutex poisoned-lock recovery
//   some_option.expect("descriptive reason")          // infallible unwrap with explanation
//   result?                                           // propagate via ? operator
```

### Print Buffer Drain
**Source:** `hp41-cli/src/app.rs` line 1001 (`call_dispatch_and_drain`)
**Apply to:** `commands.rs` — both `dispatch_op` and `get_state` handlers
```rust
let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
Ok(CalcStateView::from_state(&calc, print_lines))
// drain() mutable borrow ends before from_state(&calc, ...) shared borrow begins.
// print_buffer has #[serde(default, skip)] — never persisted; always drained fresh.
```

### Digit Entry Guard (No Dispatch for Digits)
**Source:** `hp41-cli/src/app.rs` lines 342–388
**Apply to:** `commands.rs` `dispatch_op` handler — digit key_ids ("0"–"9", ".", "e")
```rust
// Digits: push to entry_buf, return CalcStateView immediately. No dispatch() call.
// dispatch() calls flush_entry_buf() at its own entry point for non-digit ops.
// DO NOT call dispatch() for digit keys — would push each digit as a PushNum.
```

### Op Dispatch with HpError Conversion
**Source:** `hp41-cli/src/app.rs` line 999 + `hp41-core/src/error.rs`
**Apply to:** `commands.rs` — the non-digit path in `dispatch_op`
```rust
// Requires impl From<HpError> for GuiError in types.rs.
dispatch(&mut calc, op).map_err(GuiError::from)?;
// HpError implements Display via thiserror — e.to_string() yields "overflow" etc.
```

### Serde Derive for Tauri Commands
**Source:** `hp41-gui/src-tauri/Cargo.toml` line 20 (`serde = { version = "1", features = ["derive"] }`)
**Apply to:** `types.rs` — all three public structs
```rust
// CalcStateView and Annunciators: Serialize only (sent TO frontend)
// GuiError: Serialize only (Tauri maps Err variant to rejected Promise payload)
// Do NOT add Deserialize — these are outbound-only DTOs
```

---

## No Analog Found

All 5 files have close analogs. No entries in this section.

---

## Metadata

**Analog search scope:** `hp41-cli/src/`, `hp41-core/src/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src-tauri/capabilities/`
**Files scanned:** 8 source files read directly
**Pattern extraction date:** 2026-05-09

**Key public re-exports from `hp41-core/src/lib.rs` lines 14–20** (confirmed accessible in hp41-gui):
- `hp41_core::HpError` — error type; implements `Display` via thiserror
- `hp41_core::format_hpnum` — formats `HpNum` per `DisplayMode` (Fix/Sci/Eng)
- `hp41_core::format_alpha` — formats alpha register string (12-char truncation)
- `hp41_core::CalcState` — full calculator state; `#[derive(Serialize, Deserialize, Clone)]`
- `hp41_core::AngleMode` — enum `{Deg, Rad, Grad}`; needed for annunciator booleans
- `hp41_core::ops::dispatch` — `pub fn dispatch(state: &mut CalcState, op: Op) -> Result<(), HpError>`
- `hp41_core::StoArithKind` — needed in key_map.rs for `Op::StoArith { reg, kind }`
