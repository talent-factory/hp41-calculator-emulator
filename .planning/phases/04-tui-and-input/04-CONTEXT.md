# Phase 4: TUI & Input - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning
**Mode:** Auto (autonomous)

<domain>
## Phase Boundary

Build the full `hp41-cli` TUI binary using ratatui 0.30 + crossterm 0.29: a persistent terminal panel showing the HP-41 4-level stack, LASTX, 12-char display, all annunciators, and a discoverable key-label reference — all driven by a synchronous `poll → update → redraw` event loop with correct panic handling.

**Deliverables:**
- `hp41-cli/Cargo.toml` updated with ratatui 0.30, crossterm 0.29, clap 4.x
- `hp41-cli/src/main.rs` — entry point: clap args, `ratatui::init()`, event loop, panic hook
- `hp41-cli/src/app.rs` — `App` struct owning `CalcState`; `update(event)` → `render(frame)` methods
- `hp41-cli/src/ui.rs` — ratatui widget layout: stack panel, display panel, annunciator bar, key-ref panel
- `hp41-cli/src/keys.rs` — physical key → `Op` mapping table; digit entry state machine
- `hp41-cli/src/prgm_display.rs` — PRGM mode step display: step number + Op name
- DISP-01, DISP-02, INPUT-01 test coverage (manual-only for full TUI, automated for key mapping logic)

</domain>

<decisions>
## Implementation Decisions

### TUI Layout
- **D-01:** Fixed single-panel layout (no dynamic resize required for v1.0). Columns: left = stack+display+annunciators, right = key-reference table. Minimum terminal size: 80×24.
- **D-02:** Stack display: X/Y/Z/T each on own line, labeled. LASTX shown below T. 12-char HP-41 display shown prominently (largest text element). Annunciator bar: `[USER] [PRGM] [ALPHA] [SHIFT] [RAD] [DEG] [GRAD]` — lit/dim based on CalcState flags.
- **D-03:** Key-reference panel shows a compact table: key → function. Discoverable without external reference. Updated dynamically when USER mode is active.

### Event Loop
- **D-04:** `event::poll(Duration::from_millis(16))` (60fps-class) → check for key events → call `app.update(key)` → `terminal.draw(|f| app.render(f))`. Never use `event::read()` — poll is required to support future 30-second auto-save timer (Phase 5) without blocking.
- **D-05:** `ratatui::init()` (not `Terminal::new()`) — installs the panic hook that restores terminal on unhandled panics. This is the SC-4 requirement.
- **D-06:** Filter `KeyEventKind::Release` immediately — Windows crossterm fires both Press and Release; only `KeyEventKind::Press` is processed. Required on Windows; harmless on macOS/Linux.

### Keyboard Mapping
- **D-07:** Key mapping in `keys.rs`: `fn key_to_op(key: KeyEvent, app: &App) -> Option<Op>`. Returns `None` for unmapped keys (silently ignored). Context-sensitive: digit keys append to `entry_buf` vs dispatch differently in PRGM mode.
- **D-08:** Core key assignments:
  - `0-9`, `.` → digit entry (append to CalcState.entry_buf)
  - `Enter` → `Op::Enter`
  - `Backspace` → `Op::Clx` (clear X, matches HP-41 ← key)
  - `+`, `-`, `*`, `/` → `Op::Add/Sub/Mul/Div`
  - `n` → `Op::Chs` (change sign)
  - `r` → `Op::Rdn` (roll down)
  - `x` → `Op::XySwap`
  - `l` → `Op::Lastx`
  - `s` → `Op::Sqrt`
  - `q` → quit
  - `p` → `Op::PrgmMode` (toggle PRGM recording)
  - `F5` → `Op::Rtn` (R/S in interactive context: run program from PC=0 or stop)
  - `F1–F4` → user-assignable (Phase 5); stub as no-op in Phase 4
  - `?` → toggle help overlay (Phase 5); stub in Phase 4
  - Full trig, log, exp: multi-key sequences or dedicated letters (see below)
- **D-09:** Multi-key sequences for less common ops: `ss` = SIN, `cc` = COS, `tt` = TAN, `ll` = LN, `gg` = LOG, `ee` = EXP, `hh` = 10^x, `ii` = 1/x, `ww` = x², `yy` = y^x. Prefix timeout: 500ms. If second key doesn't arrive, first key is treated literally or ignored.
  Alternative (simpler): shift key modifier. `Shift+s` = SIN, etc. — use this if multi-key adds complexity.
  **[auto]**: Use shift-key modifier approach: `S` (uppercase) = SIN, `C` = COS, `T` = TAN, `L` = LN, `G` = LOG, `E` = EXP (e^x), `H` = 10^x, `I` = 1/x, `W` = x², `Y` = y^x. Uppercase via Shift is standard terminal behavior.
- **D-10:** Angle mode toggle: `d` cycles DEG→RAD→GRAD (op_set_deg/rad/grad). Display mode: `f` cycles FIX→SCI→ENG with digit count 4 (default). Digit count adjustment deferred to Phase 5.

### Digit Entry
- **D-11:** Digit keys (`0-9`, `.`) call `dispatch(state, Op::PushNum(...))` only when a non-digit op is pressed (HP-41 entry-then-op model). While typing digits, append to `state.entry_buf` directly (NOT via dispatch). `flush_entry_buf` is called by dispatch automatically.
  Simpler: just append to `entry_buf` directly for digit keys; entry_buf is flushed on next op by dispatch(). This matches exactly how dispatch() already works.
- **D-12:** Display shows `entry_buf` content while non-empty (live digit preview). When empty, shows `format_hpnum(&state.stack.x, &state.display_mode)`.
- **D-13:** `EEX` key (scientific notation entry): `e` key appends "E" separator to entry_buf for SCI entry. Already supported by Decimal::from_str.

### PRGM Mode Display
- **D-14:** When `state.prgm_mode = true`, the main display area switches to program step view: `{step_num:03} {op_name}` where step_num = current `state.pc` and op_name = human-readable Op name.
- **D-15:** SST (single-step forward): `F7` key — increment state.pc (wraps at program length). BST (back-step): `F8` key — decrement state.pc. These update the PRGM display without executing.
- **D-16:** R/S (run/stop): `F5` in execute mode runs program from LBL at current pc (or prompts for label). In Phase 4, simplified: `F5` calls `run_program(state, "A")` hardcoded — full label entry deferred to Phase 5.

### Panic Handling
- **D-17:** `ratatui::init()` return value (RestoreTerminalGuard) must be held until program exit — do NOT drop it early. The guard restores terminal on drop, which fires on panic via the installed hook.
- **D-18:** All `hp41-core` errors are `Result<(), HpError>` — no panics in core. CLI boundary catches errors and displays them in the TUI status bar (one-line error message at bottom of screen), not as panics.

### Claude's Discretion
- Color scheme: minimal — use terminal defaults + bold for active annunciators, dim for inactive
- App architecture: `struct App { state: CalcState, message: Option<String> }` — simple flat struct, no state machine needed for Phase 4
- Exit: `q` or `Ctrl+C` both quit cleanly, restoring terminal

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `hp41-core/src/ops/mod.rs` — dispatch() entry point, all Op variants, flush_entry_buf
- `hp41-core/src/state.rs` — CalcState (all fields including prgm_mode, pc, program, entry_buf)
- `hp41-core/src/format.rs` — format_hpnum() for display formatting
- `hp41-core/src/ops/program.rs` — run_program() signature, is_running semantics
- `.planning/STATE.md` — key decisions (no async in core, ratatui version)
- `./CLAUDE.md` — ratatui::init() mandate, KeyEventKind::Release filter, event::poll() requirement

### Requirements
- `.planning/REQUIREMENTS.md` §DISP-01, §DISP-02, §INPUT-01
- `.planning/ROADMAP.md` §Phase 4 — success criteria (all 4 must be TRUE)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hp41-core::dispatch(state, op)` — single entry point for all ops; key handler calls this
- `hp41-core::format_hpnum(&n, &mode)` — ready to use for display rendering
- `hp41-core::format_alpha(&s)` — for ALPHA register display (12-char truncation)
- `hp41-core::run_program(state, label)` — for R/S execution
- `CalcState::new()` — clean state initialization; `App` owns one instance

### Established Patterns
- All state mutation flows through `dispatch()` — key handler must follow this
- `entry_buf` is managed by `flush_entry_buf()` inside dispatch — digit keys only need to append to `state.entry_buf` directly
- hp41-core has zero TUI dependencies — hp41-cli adds all UI-layer code

### Integration Points
- `hp41-cli/Cargo.toml` adds: `ratatui = { version = "0.30", features = ["crossterm"] }`, `crossterm = "0.29"`, `clap = { version = "4", features = ["derive"] }`
- `main.rs` calls `ratatui::init()`, runs event loop, handles `App`
- `app.rs` owns `CalcState` and translates key events → Op dispatch

</code_context>

<specifics>
## Specific Ideas

- `ratatui::init()` returns `DefaultTerminal` + installs panic hook — the return value IS the terminal handle for SC-4
- crossterm `KeyEventKind::Release` filter: `if key.kind != KeyEventKind::Press { continue; }`
- Annunciators: `state.angle_mode` → RAD/DEG/GRAD; `state.alpha_mode` → ALPHA; `state.prgm_mode` → PRGM
- USER and SHIFT annunciators: always dim in Phase 4 (USER mode is Phase 5; SHIFT is not implemented)

</specifics>

<deferred>
## Deferred Ideas

- Help overlay / searchable function reference — Phase 5 (UX-01)
- USER mode key assignments — Phase 5 (UX-02)
- F1–F4 user-assignable keys — Phase 5
- Auto-save timer inside event loop — Phase 5 (PERS-02 requires 30s auto-save)
- Label entry dialog for R/S — Phase 5
- Full digit-count adjustment (FIX 4, SCI 2, etc.) via key sequence — Phase 5
- Terminal resize handling — deferred; minimum 80×24 enforced with error message
- Mouse support — out of scope for v1.0

</deferred>

---

*Phase: 4-TUI-and-Input*
*Context gathered: 2026-05-07*
