# Phase 11: Print Emulation - Research

**Researched:** 2026-05-08
**Domain:** Rust — hp41-core op extension, hp41-cli modal input, BufWriter file logging
**Confidence:** HIGH

## Summary

Phase 11 adds three print operations (PRX, PRA, PRSTK) to the HP-41 emulator. All three are purely I/O-free at the core layer: `hp41-core` buffers formatted strings into `CalcState.print_buffer: Vec<String>`, and `hp41-cli` drains that buffer after each dispatch to show output in the status bar (`app.message`) and optionally append to a file.

This is the fourth op-extension phase in v1.1 (after Phases 9/10). The codebase is highly regularised: every new op follows the same four-file pattern (`state.rs` field, `ops/mod.rs` enum + dispatch arm, `ops/program.rs` execute_op arm, new `ops/print.rs` module). The print modal follows the Phase 10 `'S'`-prefix pattern exactly.

All decisions are locked in CONTEXT.md. The research below confirms the integration points, verifies the exact function signatures already available, and identifies the one meaningful pitfall area: the help_data test asserts exactly 13 category headers — adding a "Print" category will require updating that assertion or placing print ops inside an existing category.

**Primary recommendation:** Follow the four-file op-extension pattern + Phase 10 modal pattern. The only decision left to the planner (D-15 vs D-16) is where to drain the buffer in `app.rs` — both are valid; D-16 (inside `handle_key` after `call_dispatch`) is simpler and avoids changing `run()`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Console output = `app.message`. PRX/PRA show the single formatted line. PRSTK shows `"PRSTK → 6 lines"` as a confirmation summary.
**D-02:** No layout changes to `ui.rs`. Status bar (`Constraint::Min(0)`) is the only console. Scrollable panel (PRNT-05) is deferred.
**D-03:** File logging is opt-in via `--print-log <path>`. Without the flag, no file is written. With it, all output is appended (not overwritten).
**D-04:** File handle: `Option<BufWriter<File>>` in `App`. Opened in `App::new()`. Open failure → set `app.message` to error, continue without file logging (never panic).
**D-05:** `print_log: Option<std::path::PathBuf>` in `Cli` struct with `#[arg(long, value_name = "FILE")]`. Passed to `App::new()`.
**D-06:** `'P'` (Shift+P) is intercepted in `handle_key()` before `key_to_op()`, setting `PendingInput::PrintModal`.
**D-07:** Inside PrintModal: `x`/`X` → PRX, `a`/`A` → PRA, `s`/`S` → PRSTK, Esc → cancel. Other keys silently ignored.
**D-08:** TUI modal display string: `"PRNT: _"`.
**D-09:** `help_data.rs` gains three new entries: `"P X"` → PRX, `"P A"` → PRA, `"P S"` → PRSTK.
**D-10:** `print_buffer: Vec<String>` on `CalcState` with `#[serde(default)]`. Initialized to `Vec::new()` in `new()`. Serialized but cleared on load.
**D-11:** `Op::PRX`, `Op::PRA`, `Op::PRSTK` added to Op enum. All three `LiftEffect::Neutral`.
**D-12:** New `hp41-core/src/ops/print.rs` module: `op_prx`, `op_pra`, `op_prstk`.
**D-13:** PRSTK line format: `format!("{:<7}{:>17}", label, value)` = 24 chars. Labels: T:, Z:, Y:, X:, LASTX:, ALPHA:. ALPHA line is left-aligned value. When ALPHA is empty: `"ALPHA:                 "` (17 spaces).
**D-14:** Add Op::PRX/PRA/PRSTK to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs`.
**D-15/D-16:** Drain `state.print_buffer` after dispatch — either in `run()` before next draw, or inside `handle_key()` after `call_dispatch()`. Planner decides.

### Claude's Discretion

- File handle lifetime and exact drain location (D-15 vs D-16).
- Error message wording for file open failure.
- Whether `format_hpnum` output for PRSTK lines needs trimming before right-alignment.

### Deferred Ideas (OUT OF SCOPE)

- PRNT-05: Scrollable print history panel in TUI.
- PRNT-06: ADV (paper advance), PRREG (print all registers), Flag 26 / TRACE mode.
- Default print log path always active.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PRNT-01 | PRX prints X register in current display format (FIX/SCI/ENG), right-aligned to 24 chars, to console | `format_hpnum(&state.stack.x, &state.display_mode)` already exists and is exported; `format!("{:>24}", ...)` handles right-alignment |
| PRNT-02 | PRA prints ALPHA register, left-aligned to 24 chars, to console | `state.alpha_reg` is a `String`; `format!("{:<24}", &state.alpha_reg)` with truncation handles it |
| PRNT-03 | PRSTK prints full stack T→Z→Y→X→LASTX→ALPHA to console | Six `format!` calls using `format_hpnum` for numeric values; ALPHA special-cased per D-13 |
| PRNT-04 | `--print-log <path>` appends all PRX/PRA/PRSTK output to file | `BufWriter<File>` with `OpenOptions::new().create(true).append(true).open(path)` in `App::new()` |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Format print lines (PRX, PRA, PRSTK) | hp41-core (`ops/print.rs`) | — | Core invariant: all formatting is I/O-free and lives in core |
| Buffer print output | hp41-core (`CalcState.print_buffer`) | — | Preserves zero I/O dependency in core |
| Display print output in status bar | hp41-cli (`app.rs`) | — | Buffer drain is a CLI concern |
| Append print output to file | hp41-cli (`app.rs`) | — | File I/O is CLI-only per project invariant |
| Keyboard modal for print op selection | hp41-cli (`app.rs`) | `ui.rs` | Modal state machine lives in App; prompt displayed by ui.rs |
| CLI argument parsing (`--print-log`) | hp41-cli (`main.rs`) | — | clap 4.x Cli struct already in this file |

---

## Standard Stack

### Core (already in workspace — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rust_decimal | 1.42 [VERIFIED: Cargo.toml] | HpNum arithmetic | Project standard; `format_hpnum` already uses it |
| serde + serde_json | workspace version [VERIFIED: Cargo.toml] | JSON persistence; `#[serde(default)]` on new field | Project standard |
| std::io::BufWriter + std::fs::File | stdlib | Buffered file append for `--print-log` | No external dep needed |
| std::fs::OpenOptions | stdlib | `create(true).append(true)` for log file | No external dep needed |

### No new dependencies required [VERIFIED: codebase inspection]

All required capabilities are in the Rust stdlib or already in the workspace. Phase 11 introduces zero new Cargo dependencies.

---

## Architecture Patterns

### System Architecture Diagram

```
KeyEvent('P')
    │
    ▼
handle_key() — 'P' interceptor (before key_to_op)
    │
    ▼
pending_input = PrintModal
    │
    ▼  (next key)
handle_pending_input(PrintModal)
    │
    ├─ 'x'/'X' ──► call_dispatch(Op::PRX)
    ├─ 'a'/'A' ──► call_dispatch(Op::PRA)
    ├─ 's'/'S' ──► call_dispatch(Op::PRSTK)
    └─ Esc     ──► pending_input = None

call_dispatch(Op::PRX/PRA/PRSTK)
    │
    ▼
hp41_core::ops::dispatch()
    │
    ▼
op_prx / op_pra / op_prstk  (hp41-core/src/ops/print.rs)
    │  reads: stack.x / alpha_reg / full stack + lastx
    │  writes: state.print_buffer.push(formatted_line)
    ▼
return to App::call_dispatch()
    │
    ▼  (buffer drain — D-16 recommended location)
drain state.print_buffer:
    ├─ for each line: if print_log_writer.is_some() → write line+\n, flush
    └─ app.message = last line  (or "PRSTK → 6 lines" for PRSTK)
    state.print_buffer.clear()
    │
    ▼
terminal.draw() — renders app.message in status bar
```

### Recommended Project Structure (no new top-level modules)

```
hp41-core/src/ops/
├── mod.rs        # Add Op::PRX, Op::PRA, Op::PRSTK + dispatch arms
├── print.rs      # NEW: op_prx, op_pra, op_prstk
├── program.rs    # Add execute_op arms for PRX/PRA/PRSTK
└── registers.rs  # (unchanged — reference for pattern)

hp41-core/src/
└── state.rs      # Add print_buffer: Vec<String> with #[serde(default)]

hp41-cli/src/
├── main.rs       # Add print_log: Option<PathBuf> to Cli struct
├── app.rs        # Add print_log_writer field, 'P' interceptor, PrintModal arm, buffer drain
└── ui.rs         # Add PrintModal arm to pending_prompt()

hp41-cli/src/help_data.rs   # Add 3 print op entries + "=== Print ===" category header
```

### Pattern 1: New Op Module (mirrors registers.rs, trig.rs)

```rust
// Source: hp41-core/src/ops/registers.rs (verified pattern)
// hp41-core/src/ops/print.rs

use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
use crate::format::format_hpnum;

pub fn op_prx(state: &mut CalcState) -> Result<(), crate::error::HpError> {
    let line = format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode));
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_pra(state: &mut CalcState) -> Result<(), crate::error::HpError> {
    // PRA uses 24-char width, NOT format_alpha (which truncates to 12)
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    let line = format!("{:<24}", alpha);
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_prstk(state: &mut CalcState) -> Result<(), crate::error::HpError> {
    let mode = &state.display_mode;
    let lines = [
        format!("{:<7}{:>17}", "T:",     format_hpnum(&state.stack.t,     mode)),
        format!("{:<7}{:>17}", "Z:",     format_hpnum(&state.stack.z,     mode)),
        format!("{:<7}{:>17}", "Y:",     format_hpnum(&state.stack.y,     mode)),
        format!("{:<7}{:>17}", "X:",     format_hpnum(&state.stack.x,     mode)),
        format!("{:<7}{:>17}", "LASTX:", format_hpnum(&state.stack.lastx, mode)),
        {
            let alpha = state.alpha_reg.chars().take(17).collect::<String>();
            format!("{:<7}{:<17}", "ALPHA:", alpha)
        },
    ];
    for line in lines {
        state.print_buffer.push(line);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

### Pattern 2: Print Modal (mirrors StoRegister / Phase 10 'S'-prefix pattern)

```rust
// Source: hp41-cli/src/app.rs — StoRegister interceptor (verified pattern)

// In handle_key(), AFTER pending_input check, BEFORE key_to_op():
if key.code == KeyCode::Char('P') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::PrintModal);
    self.message = None;
    return;
}

// In handle_pending_input(), new arm:
Some(PendingInput::PrintModal) => {
    match key.code {
        KeyCode::Char('x') | KeyCode::Char('X') => {
            self.call_dispatch(Op::PRX);
            self.pending_input = None;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            self.call_dispatch(Op::PRA);
            self.pending_input = None;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            self.call_dispatch(Op::PRSTK);
            self.pending_input = None;
        }
        KeyCode::Esc => {
            self.pending_input = None;
        }
        _ => {
            // Silently ignore unrecognized keys — keep modal open
            self.pending_input = Some(PendingInput::PrintModal);
        }
    }
}
```

### Pattern 3: Buffer Drain After Dispatch (D-16 location — inside call_dispatch or immediately after)

The cleanest integration for D-16 is a wrapper method on `App` that calls `call_dispatch` then drains:

```rust
// Source: hp41-cli/src/app.rs — call_dispatch pattern (verified)

fn call_dispatch_and_drain(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => {
            // Drain print_buffer immediately after dispatch
            let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
            if !lines.is_empty() {
                for line in &lines {
                    if let Some(ref mut writer) = self.print_log_writer {
                        let _ = writeln!(writer, "{}", line); // best-effort, never panic
                        let _ = writer.flush();
                    }
                }
                // D-01: For PRSTK (6 lines), show summary. For PRX/PRA (1 line), show the line.
                if lines.len() > 1 {
                    self.message = Some(format!("PRSTK \u{2192} {} lines", lines.len()));
                } else {
                    self.message = Some(lines.into_iter().next().unwrap_or_default());
                    // Note: unwrap_or_default() is safe; .is_empty() already guards len==1
                }
            } else {
                self.message = None;
            }
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
}
```

Note: the planner must decide whether to introduce `call_dispatch_and_drain` as a new method, or inline the drain logic only where print ops can occur (the PrintModal arm). The latter avoids touching non-print dispatch paths; the former is cleaner but modifies all callers.

**Simpler alternative:** Drain `state.print_buffer` at the start of `call_dispatch` right after `hp41_core::ops::dispatch` succeeds, only when the buffer is non-empty. This requires adding the file writer and message-setting logic to `call_dispatch` itself.

### Pattern 4: File Logger Initialization

```rust
// Source: stdlib — OpenOptions pattern (VERIFIED: Rust docs)
// In App::new():

let print_log_writer: Option<BufWriter<File>> =
    print_log_path.and_then(|path| {
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => Some(BufWriter::new(file)),
            Err(e) => {
                // Caller sets app.message after App::new() returns
                eprintln!("Warning: failed to open print log {}: {e}", path.display());
                None
            }
        }
    });
```

Because `App::new()` currently takes no error-returning path and sets no message, the open-failure case should either be communicated via a return value or by adding an `Option<String>` startup message field pattern (same as `load_message` in `main.rs`). Planner chooses the integration point.

### Anti-Patterns to Avoid

- **`println!` in `hp41-core`:** Enforced at compile time by zero-CLI-dep workspace invariant. All output stays in `print_buffer`.
- **`unwrap()` in production code in `hp41-core`:** `#![deny(clippy::unwrap_used)]` at crate root. Use `.expect("reason")` or `?`.
- **Draining inside `op_prx/pra/prstk`:** The ops only push to buffer; draining is the CLI's responsibility.
- **Using `format_alpha` (12-char truncation) for PRA output:** PRA needs 24-char width, not the 12-char display truncation. Use `state.alpha_reg.chars().take(24)` directly.
- **Missing `execute_op` arm:** The most common Phase 3+ omission. PRX/PRA/PRSTK must appear in BOTH `dispatch()` and `execute_op()`.
- **Missing `mod print;` declaration in `ops/mod.rs`:** The new module will not be compiled until declared.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Buffered file I/O | Custom write loop | `std::io::BufWriter<std::fs::File>` | Handles partial writes, buffer flushing automatically |
| File append mode | Read + seek + write | `OpenOptions::new().create(true).append(true).open()` | OS-atomic append on most systems |
| Number formatting | Custom HP-41 formatter | `format_hpnum(&val, &display_mode)` (already in hp41-core) | Handles FIX/SCI/ENG + edge cases (overflow, FIX 0, negative) |
| Padding/alignment | Manual space padding | `format!("{:>24}", s)` / `format!("{:<24}", s)` | Rust's format spec handles right/left alignment |

---

## Common Pitfalls

### Pitfall 1: Missing execute_op Arm
**What goes wrong:** `Op::PRX` is added to `dispatch()` but not to `execute_op()` in `program.rs`. PRX works interactively but silently returns `Err(HpError::InvalidOp)` when used inside a running program.
**Why it happens:** Two separate match tables that must be kept in sync (CONTEXT D-14, STATE.md Critical Traps).
**How to avoid:** Add both match arms in the same plan task, then run `just test` to verify compilation.
**Warning signs:** Compiler does not warn about missing arms in `execute_op` (the `other =>` fallthrough catches it). Only a test calling PRX inside a program would catch this.

### Pitfall 2: print_buffer Not Cleared After Drain
**What goes wrong:** `state.print_buffer` is drained but not cleared. On the next dispatch, the buffer is empty, but if the drain does a `for line in &lines` without `.drain(..)`, the buffer remains populated. Next frame re-shows stale output.
**Why it happens:** Using `.iter()` instead of `.drain(..)`.
**How to avoid:** Use `state.print_buffer.drain(..).collect::<Vec<_>>()` (drain consumes and clears) or `state.print_buffer.clear()` explicitly after collecting.

### Pitfall 3: `'S'` Key Conflict with PrintModal
**What goes wrong:** Inside `PrintModal`, pressing `'s'` or `'S'` dispatches PRSTK correctly. But the `'S'` interceptor for `StoRegister` runs first if the `pending_input` block is not placed before the `'S'` check.
**Why it happens:** The `'S'`-interceptor and `'P'`-interceptor are in the same `handle_key()` function.
**How to avoid:** The existing code already routes `pending_input.is_some()` before all modal-opening interceptors. The `'P'` interceptor is only reached when `pending_input` is `None`, so there is no conflict. Verify the `'P'` check is placed in the same guard block as `'S'` and `'R'`.

### Pitfall 4: help_data.rs Category Count Test
**What goes wrong:** Adding a new `"=== Print ==="` category header to `HELP_DATA` causes the `test_all_thirteen_categories_present` test to fail — it asserts exactly the 13 original categories by name.
**Why it happens:** The test has a hardcoded list of the 13 v1.0 categories.
**How to avoid:** Either (a) add the three print entries to the `"=== Registers ==="` or a related category without adding a new header, or (b) add `"=== Print ==="` to the category and update the test assertion to include it (changing `13` to `14` categories).
**Recommendation:** Add a `"=== Print ==="` category header and update both the test assertion count and the category list in the test. This is the honest solution.

### Pitfall 5: format_alpha vs. 24-char PRA
**What goes wrong:** `format_alpha(&state.alpha_reg)` is used for PRA output. `format_alpha` truncates to 12 chars (HP-41 display width), producing a 12-char output instead of the required 24-char print output.
**Why it happens:** `format_alpha` is the obvious function name for formatting the ALPHA register, but it's scoped to TUI display, not print.
**How to avoid:** PRA uses `state.alpha_reg.chars().take(24).collect::<String>()` and then `format!("{:<24}", alpha)`.

### Pitfall 6: BufWriter Drop on Error Path
**What goes wrong:** If the file fails to open mid-session (e.g., disk full, permission change), writes to `BufWriter` may silently buffer without flushing. Dropping `BufWriter` without flushing discards the buffer.
**Why it happens:** `BufWriter::drop()` silently discards unflushed bytes on error.
**How to avoid:** `flush()` after every `writeln!`. Per D-04, the write is best-effort — use `let _ = writer.flush()` to ignore flush errors without panicking.

---

## Code Examples

### Adding print_buffer to CalcState

```rust
// Source: hp41-core/src/state.rs (verified — existing struct)
// Add after key_assignments field:

/// Buffer of formatted print lines from PRX/PRA/PRSTK.
/// Drained by hp41-cli after each dispatch. Never persisted across sessions.
/// #[serde(default)] preserves backward compat with v1.0 save files.
#[serde(default)]
pub print_buffer: Vec<String>,

// In CalcState::new():
print_buffer: Vec::new(),
```

### Adding Op variants and dispatch arms

```rust
// Source: hp41-core/src/ops/mod.rs (verified — existing Op enum structure)
// In Op enum, add after HmsSub:
/// PRX — print X register in current display format, right-aligned to 24 chars.
/// LiftEffect: Neutral.
PRX,
/// PRA — print ALPHA register, left-aligned to 24 chars. LiftEffect: Neutral.
PRA,
/// PRSTK — print full stack T/Z/Y/X/LASTX/ALPHA, 6 lines, 24 chars each. LiftEffect: Neutral.
PRSTK,

// In dispatch() match, add to Phase-tagged section:
Op::PRX  => print::op_prx(state),
Op::PRA  => print::op_pra(state),
Op::PRSTK => print::op_prstk(state),

// At top of dispatch(): add to use declarations:
use print::{op_prx, op_pra, op_prstk};

// At top of mod.rs: declare new module
pub mod print;
```

### Adding to execute_op in program.rs

```rust
// Source: hp41-core/src/ops/program.rs execute_op() (verified — existing pattern)
// Add before the programming-ops catch-all at the end:

Op::PRX  => super::print::op_prx(state),
Op::PRA  => super::print::op_pra(state),
Op::PRSTK => super::print::op_prstk(state),
```

### Adding print_log to Cli and App

```rust
// Source: hp41-cli/src/main.rs (verified — existing Cli struct)
// In Cli struct:
/// Append all PRX/PRA/PRSTK output to this file (created if absent, appended if exists).
#[arg(long, value_name = "FILE")]
print_log: Option<std::path::PathBuf>,

// Pass to App::new():
let mut app = App::new(initial_state, state_path, cli.print_log);

// In App struct:
/// BufWriter for --print-log, if specified. None = no file logging.
print_log_writer: Option<std::io::BufWriter<std::fs::File>>,

// In App::new() signature change:
pub fn new(state: CalcState, state_path: PathBuf, print_log: Option<PathBuf>) -> Self
```

### pending_prompt arm for PrintModal

```rust
// Source: hp41-cli/src/ui.rs pending_prompt() (verified — existing pattern)
PendingInput::PrintModal => "PRNT: _".to_string(),
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Direct `println!` from ops | `print_buffer: Vec<String>` on CalcState | Phase 11 design | Preserves zero-I/O in hp41-core |
| N/A — first print phase | `BufWriter<File>` append | Phase 11 | Standard Rust pattern; no new crates |

**No deprecated patterns apply to this phase.**

---

## Assumptions Log

> All claims in this research were verified against the codebase (`[VERIFIED: codebase inspection]`) or are direct quotes from locked CONTEXT.md decisions (`[CITED: 11-CONTEXT.md]`).

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `format_hpnum` output width for typical HP-41 values fits within 17-char right-aligned field in PRSTK lines | Code Examples | PRSTK lines could exceed 24 chars. Mitigate: `format_hpnum` returns 6–12 chars typically; right-aligning in 17 adds enough headroom. |

**Risk assessment for A1:** `format_hpnum` for SCI 9 mode (widest) returns strings like `"-1.234567890E-99"` = 16 chars. `"{:>17}"` right-aligns this to exactly 17 chars. Combined with 7-char label = 24 chars total. This is safe. [ASSUMED from `format_hpnum` implementation in format.rs — verified the implementation but not all edge cases systematically]

---

## Open Questions

1. **`App::new()` signature change for `print_log`**
   - What we know: `App::new` currently takes `(state: CalcState, state_path: PathBuf)`. Adding `print_log` as a third parameter is the simplest approach.
   - What's unclear: Whether tests that call `App::new_for_test()` or `App::new(...)` directly need updating.
   - Recommendation: `App::new_for_test()` is a private test constructor; update it to call `App::new(state, path, None)`. All test call sites in `app.rs` use `App::new_for_test()` so only one place to fix.

2. **Drain location: D-15 vs D-16**
   - What we know: D-15 drains in `run()` before `terminal.draw()`; D-16 drains inside `handle_key()` after `call_dispatch()`.
   - What's unclear: D-15 requires changing `run()` which is tested; D-16 requires either a new `call_dispatch_and_drain` method or inlining drain logic in the PrintModal arm only.
   - Recommendation: D-16 with drain inlined only in the PrintModal arm of `handle_pending_input`. This is the minimal-footprint change: only the three print ops ever need buffer drain; all other ops leave `print_buffer` empty.

3. **File open error reporting**
   - What we know: `App::new()` returns `Self`, not `Result`. Error must be communicated as a startup message.
   - What's unclear: Whether to add an `Option<String>` error field to `App` or follow the `main.rs` pattern of returning a message alongside the app.
   - Recommendation: Follow the `main.rs` pattern — `App::new()` returns `(App, Option<String>)` or `App::new()` stores the error in `self.message` after construction. The latter is simpler; the main.rs pattern using `load_message` already does this.

---

## Environment Availability

Step 2.6 SKIPPED — Phase 11 is a pure code extension with no external tool dependencies. All required capabilities (Rust stdlib BufWriter, std::fs::File, std::path::PathBuf) are in the standard library. No new binaries, services, or CLI tools are required.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + cargo llvm-cov for coverage |
| Config file | justfile (`just test`, `just ci`, `just coverage`) |
| Quick run command | `cargo test --workspace` |
| Full suite command | `just ci` (lint + test + coverage ≥80%) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PRNT-01 | PRX writes X right-aligned to 24 chars into print_buffer | unit | `cargo test -p hp41-core --test print_tests test_prx_` | ❌ Wave 0 |
| PRNT-01 | PRX respects FIX/SCI/ENG display mode | unit | `cargo test -p hp41-core --test print_tests test_prx_display_mode` | ❌ Wave 0 |
| PRNT-01 | PRX output shown in app.message after drain | unit | `cargo test -p hp41-cli` | ❌ Wave 0 |
| PRNT-02 | PRA writes alpha_reg left-aligned to 24 chars | unit | `cargo test -p hp41-core --test print_tests test_pra_` | ❌ Wave 0 |
| PRNT-03 | PRSTK writes 6 lines, each 24 chars, correct labels | unit | `cargo test -p hp41-core --test print_tests test_prstk_` | ❌ Wave 0 |
| PRNT-03 | PRSTK ALPHA line is correct when empty and non-empty | unit | `cargo test -p hp41-core --test print_tests test_prstk_alpha_` | ❌ Wave 0 |
| PRNT-04 | --print-log appends to file; file created if absent | integration | `cargo test -p hp41-cli` (temp file) | ❌ Wave 0 |
| PRNT-04 | Open failure sets app.message, no panic | unit | `cargo test -p hp41-cli` | ❌ Wave 0 |
| All | Existing v1.0 JSON save files load without error (serde(default)) | regression | `cargo test -p hp41-core --test program_tests` | ✅ (existing tests cover serde round-trip) |
| All | PRX/PRA/PRSTK inside a running program work (execute_op arms) | unit | `cargo test -p hp41-core --test print_tests test_prx_in_program` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --workspace`
- **Per wave merge:** `just ci`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-core/tests/print_tests.rs` — covers PRNT-01, PRNT-02, PRNT-03 core behavior
- [ ] CLI tests for `--print-log` file append and error handling (can live in `hp41-cli/src/app.rs` test module)

---

## Security Domain

> This phase adds no authentication, session management, or cryptography. The only security-adjacent concern is file path handling for `--print-log`.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (partial) | `--print-log` path is user-supplied; `OpenOptions` handles this safely via OS |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `--print-log` | Tampering | `OpenOptions::new().append(true)` respects OS permissions; no path sanitization needed for a local CLI tool |
| Log file grows unboundedly | Denial of Service | Out of scope for v1.1; user controls the path and can delete the file |

---

## Sources

### Primary (HIGH confidence)

- `hp41-core/src/state.rs` [VERIFIED: codebase] — `CalcState` struct; `#[serde(default)]` pattern for new fields
- `hp41-core/src/format.rs` [VERIFIED: codebase] — `format_hpnum()` signature and output format; `format_alpha()` 12-char truncation
- `hp41-core/src/ops/mod.rs` [VERIFIED: codebase] — `Op` enum, `dispatch()`, existing op patterns, `mod` declarations
- `hp41-core/src/ops/program.rs` [VERIFIED: codebase] — `execute_op()` structure, two-location update requirement
- `hp41-core/src/ops/registers.rs` [VERIFIED: codebase] — canonical new-module pattern
- `hp41-cli/src/app.rs` [VERIFIED: codebase] — `App` struct, `handle_key()`, `handle_pending_input()`, `PendingInput` enum, modal patterns
- `hp41-cli/src/main.rs` [VERIFIED: codebase] — `Cli` struct, clap 4.x pattern, `App::new()` call
- `hp41-cli/src/ui.rs` [VERIFIED: codebase] — `pending_prompt()` and `render_status()` patterns
- `hp41-cli/src/help_data.rs` [VERIFIED: codebase] — `HELP_DATA` structure, category tests
- `.planning/phases/11-print-emulation/11-CONTEXT.md` [CITED] — all locked decisions
- `Rust stdlib std::io::BufWriter` [ASSUMED — well-known stdlib API]
- `Rust stdlib std::fs::OpenOptions` [ASSUMED — well-known stdlib API]

### Secondary (MEDIUM confidence)

None required — all findings verified directly from codebase.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified in Cargo.toml and source files; zero new dependencies
- Architecture: HIGH — all integration points confirmed by reading actual source code
- Pitfalls: HIGH — derived from observed patterns in existing phase code and documented traps in STATE.md

**Research date:** 2026-05-08
**Valid until:** Until any of the core files listed in Sources are modified (format.rs, ops/mod.rs, program.rs, app.rs)
