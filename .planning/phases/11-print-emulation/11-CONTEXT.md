# Phase 11: Print Emulation - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Add PRX, PRA, and PRSTK print operations to `hp41-core` via a `print_buffer: Vec<String>` field on `CalcState`. The CLI drains the buffer after each op and writes output to the TUI status bar (message) and optionally to a file via `--print-log`. The `hp41-core` library gains zero I/O dependencies — it only buffers formatted strings.

This phase does NOT add a scrollable TUI print-history panel (deferred to PRNT-05 / v2+). It does NOT add ADV, PRREG, Flag-26/TRACE, or indirect addressing.

</domain>

<decisions>
## Implementation Decisions

### Print Output Visibility (PRNT-01/02/03)

- **D-01:** Console output = the TUI status bar (`app.message`). After each print op, the drained line is shown in `app.message`. For PRX/PRA, this shows the single formatted line. For PRSTK (6 lines), `app.message` shows `"PRSTK → 6 lines"` as a summary confirmation (full content only via `--print-log`).
- **D-02:** No layout changes to `ui.rs`. The existing `Constraint::Min(0)` status bar area is the "console" for v1.1. The scrollable TUI panel (PRNT-05) is explicitly deferred.

### File Logging (PRNT-04)

- **D-03:** File logging is opt-in via `--print-log <path>`. Without this flag, no file is written. With it, all PRX/PRA/PRSTK output is appended (not overwritten) to the specified file.
- **D-04:** File handle lifetime: open a `BufWriter<File>` in `App` when `--print-log` is specified (in `App::new()`), store as `Option<BufWriter<File>>`. Flush after each print. On open failure, set `app.message` to an error and continue without file logging (never panic).
- **D-05:** Add `print_log: Option<std::path::PathBuf>` to the `Cli` struct in `main.rs`. Pass to `App::new()`. Arg: `#[arg(long, value_name = "FILE")]`.

### Keyboard Bindings

- **D-06:** Use a `'P'`-prefix print modal, consistent with the Phase 10 STO `'S'`-prefix pattern. `'P'` (Shift+P) is intercepted in `handle_key()` before `key_to_op()`, setting `PendingInput::PrintModal`.
- **D-07:** Inside `PrintModal`, keys: `x`/`X` → PRX, `a`/`A` → PRA, `s`/`S` → PRSTK, Esc → cancel (no side effects). Other keys: silently ignored (existing modal convention). The `pending_input` check runs before the `'S'`/`'R'` interceptors, so no conflict.
- **D-08:** TUI modal display string: `"PRNT: _"` (shown in the display panel during modal, same as `"STO [__]"` pattern).
- **D-09:** Help overlay (`help_data.rs`) gains three new entries: `"P X"` → PRX, `"P A"` → PRA, `"P S"` → PRSTK.

### hp41-core Print Operations

- **D-10:** Add `print_buffer: Vec<String>` to `CalcState` with `#[serde(default)]`. Initialized to `Vec::new()` in `CalcState::new()`. This field is serialized to JSON but ignored on load (default = empty).
- **D-11:** Add `Op::PRX`, `Op::PRA`, `Op::PRSTK` to the `Op` enum. All three have `LiftEffect::Neutral` — print ops don't touch the stack.
- **D-12:** Each op is implemented in a new `hp41-core/src/ops/print.rs` module (mirrors the pattern of `registers.rs`, `trig.rs`, etc.):
  - `op_prx(state)` → formats `state.stack.x` using `format_hpnum(&state.stack.x, &state.display_mode)`, right-pads to 24 chars, pushes to `state.print_buffer`.
  - `op_pra(state)` → left-aligns `state.alpha_reg` to 24 chars, pushes to `state.print_buffer`.
  - `op_prstk(state)` → pushes 6 labeled lines (see D-13) to `state.print_buffer`.
- **D-13:** PRSTK line format: `format!("{:<7}{:>17}", label, value)` = 24 chars total. Labels: `"T:"`, `"Z:"`, `"Y:"`, `"X:"`, `"LASTX:"`, `"ALPHA:"`. T/Z/Y/X/LASTX values use `format_hpnum()` right-aligned in 17 chars. ALPHA line uses left-aligned format: `format!("{:<7}{:<17}", "ALPHA:", alpha_str)` — truncated to 17 chars if longer. When ALPHA is empty the ALPHA line is `"ALPHA:                 "`.
- **D-14:** Add Op::PRX, Op::PRA, Op::PRSTK to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs` (critical trap from STATE.md).

### CLI Buffer Drain

- **D-15:** In `App::run()`, after `handle_key()` returns, drain `state.print_buffer` before the next `terminal.draw()`. For each line: if `print_log_writer` is `Some`, write the line + `\n` to the file; set `app.message` to the last drained line (or "PRSTK → N lines" summary for PRSTK). Then clear `state.print_buffer`.
- **D-16:** Alternatively, drain inside `handle_key()` immediately after `dispatch()` returns — simpler, no change to `run()`. Either location is acceptable; planner decides based on code structure.

### Claude's Discretion

- File handle lifetime and exact drain location (D-15 vs D-16) — planner chooses the cleaner integration point.
- Error message wording for file open failure.
- Whether `format_hpnum` output for PRSTK lines needs any trimming before right-alignment.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/ROADMAP.md` — Phase 11 goal, 5 success criteria, PRNT-01/02/03/04 requirements
- `.planning/REQUIREMENTS.md` — Full acceptance criteria for PRNT-01 through PRNT-04; PRNT-05/06 deferred to v2+

### Core Implementation Files

- `hp41-core/src/state.rs` — `CalcState` struct (add `print_buffer` field with `#[serde(default)]`); `DisplayMode` enum
- `hp41-core/src/format.rs` — `format_hpnum()` (canonical formatter for PRX and PRSTK numeric lines); `format_alpha()`
- `hp41-core/src/lib.rs` — public re-exports (add `Op::PRX/PRA/PRSTK` visibility)
- `hp41-core/src/ops/mod.rs` — `Op` enum (add PRX/PRA/PRSTK variants), `dispatch()` (add arms)
- `hp41-core/src/ops/program.rs` — `execute_op()` (add Op::PRX/PRA/PRSTK arms, same pattern as other ops)
- `hp41-cli/src/main.rs` — `Cli` struct (add `--print-log` arg), `App::new()` call
- `hp41-cli/src/app.rs` — `App` struct (add file writer field), `handle_key()` (add `'P'` interceptor), `handle_pending_input()` (add `PrintModal` arm), buffer drain logic
- `hp41-cli/src/ui.rs` — pending_input display strings (add `"PRNT: _"` for `PrintModal` state)
- `hp41-cli/src/help_data.rs` — add 3 print op entries (`"P X"`, `"P A"`, `"P S"`)

### Prior Phase Context

- `.planning/phases/10-sto-arithmetic-modals/10-CONTEXT.md` — STO modal pattern (D-05 through D-09) that the Print modal mirrors exactly

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `format_hpnum(&HpNum, &DisplayMode) -> String` at `hp41-core/src/format.rs:18` — exact formatter to use for PRX and PRSTK numeric lines. Already exported via `hp41-core/src/lib.rs`.
- `format_alpha(reg: &str) -> String` at `hp41-core/src/format.rs:29` — truncates to 12 chars for display; PRA uses a wider 24-char version (left-aligned to 24 instead of 12).
- `PendingInput` enum + `handle_pending_input()` in `hp41-cli/src/app.rs` — the modal state machine. `PrintModal` follows the exact same pattern as `StoRegister` / `StoAdd` etc.
- `app.message: Option<String>` in `App` struct — existing status bar message field; used directly for print output confirmation.
- `Cli` struct in `hp41-cli/src/main.rs` with clap 4.x `#[derive(Parser)]` — add `print_log: Option<PathBuf>` with `#[arg(long)]`.

### Established Patterns

- `#![deny(clippy::unwrap_used)]` in `hp41-core` — `op_prx/pra/prstk()` must use `.expect("reason")` or `?`-propagation. Tests carry `#[allow(clippy::unwrap_used)]`.
- Every new `Op` variant in BOTH `dispatch()` AND `execute_op()` — critical trap, must not be forgotten.
- New `CalcState` fields must have `#[serde(default)]` for backward compatibility with v1.0 save files.
- Modal interceptors in `handle_key()` follow the pattern: check `pending_input` first, THEN check `'S'`/`'R'`/`Ctrl+A` interceptors.
- `LiftEffect::Neutral` for ops that don't consume or produce stack values.
- New op module in `hp41-core/src/ops/` (e.g., `print.rs`) declared in `ops/mod.rs` via `mod print;`.

### Integration Points

- `hp41-core/src/ops/mod.rs` lines 75+: `Op` enum — add `PRX`, `PRA`, `PRSTK` variants.
- `hp41-core/src/ops/mod.rs` `dispatch()`: add three `Op::PRX | Op::PRA | Op::PRSTK` arms.
- `hp41-core/src/ops/program.rs` `execute_op()`: add matching arms.
- `hp41-cli/src/app.rs` `handle_key()`: add `'P'` uppercase interceptor before `key_to_op()` call.
- `hp41-cli/src/app.rs` `handle_pending_input()`: add `PendingInput::PrintModal` arm.
- `hp41-cli/src/ui.rs` pending_input display block: add `PrintModal` → `"PRNT: _"`.

</code_context>

<specifics>
## Specific Ideas

- For PRX right-alignment: `format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode))` — simple and correct.
- For PRSTK when ALPHA is empty: `format!("{:<7}{:<17}", "ALPHA:", "")` = `"ALPHA:                 "` (17 spaces).
- File append mode: `OpenOptions::new().create(true).append(true).open(path)` wrapped in `BufWriter`.
- The `'P'` interceptor check: `KeyCode::Char('P')` with no modifier (same as `'S'` pattern); inside the modal, key comparison is case-insensitive for the single-letter dispatch (`x`/`X`, `a`/`A`, `s`/`S`).

</specifics>

<deferred>
## Deferred Ideas

- **PRNT-05: Scrollable print history panel in TUI** — explicitly deferred to v2+ in REQUIREMENTS.md. The current `app.message` approach is the v1.1 placeholder.
- **PRNT-06: ADV (paper advance), PRREG (print all registers), Flag 26 / TRACE mode** — niche printer peripheral ops, deferred to v2+.
- **Default print log path** (`~/.hp41/print.txt` always active) — considered but rejected for v1.1 in favor of explicit opt-in via `--print-log`.

</deferred>

---

*Phase: 11-Print Emulation*
*Context gathered: 2026-05-08*
