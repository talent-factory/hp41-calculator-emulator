# Phase 5: Persistence & UX - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning
**Mode:** User delegated all decisions → best practices applied by Claude

<domain>
## Phase Boundary

Add durable state to the running TUI: auto-save every 30 s + on graceful shutdown, manual save/load via a fixed path (CLI-overridable), an in-TUI help overlay showing all keyboard mappings, USER mode with persisted key assignments, the two deferred keyboard input modes from Phase 4 (STO/RCL register-number entry and ALPHA mode routing), and a bundled sample program library loadable from within the TUI.

**Deliverables:**
- `hp41-core/src/state.rs` — add `user_mode: bool` + `key_assignments: HashMap<char, String>` to CalcState; `#[derive(Serialize, Deserialize)]` on CalcState and all nested types; add `Op::UserMode` variant
- `hp41-core/src/ops/` — add `Op::UserMode` dispatch; any new ops needed for USER assignment
- `hp41-cli/src/persistence.rs` — `save_state(path, &CalcState)`, `load_state(path) -> Result<CalcState>`, `default_state_path() -> PathBuf` (~/.hp41/autosave.json)
- `hp41-cli/src/app.rs` — auto-save timer (`Instant` + 30s interval), `pending_input: Option<PendingInput>` for modal entry, `show_help: bool`, `show_programs: bool`
- `hp41-cli/src/keys.rs` — STO/RCL/ALPHA modal routing, USER mode key dispatch, help/program library toggles
- `hp41-cli/src/ui.rs` — help overlay widget, program library overlay widget, modal input display in status bar
- `hp41-cli/src/help_data.rs` — static array of all operations with keyboard shortcuts (grouped by category)
- `hp41-cli/src/programs.rs` — `SampleProgram` struct + static array of ≥10 bundled programs
- PERS-01, PERS-02, UX-01, UX-02, UX-03 test coverage

</domain>

<decisions>
## Implementation Decisions

### State File Strategy
- **D-01:** Default auto-save path: `~/.hp41/autosave.json`. Directory created automatically if missing (`fs::create_dir_all`). Consistent across all platforms (use `dirs::home_dir()` from the `dirs` crate — already a transitive dependency of many CLI tools; if not present, add `dirs = "5"`).
- **D-02:** CLI override: `--state-file <path>` clap arg. When provided, load this file at startup and save to it instead of the default path. Allows multiple named saves (users pass different paths).
- **D-03:** Startup behavior: if state file exists → load it and continue session; if missing → start fresh with `CalcState::new()`. Load failures (parse error, corrupt JSON) show an error in the status bar and start fresh — never panic.
- **D-04:** Manual save: `Ctrl+S` saves to the active state file immediately, shows "Saved to ~/.hp41/autosave.json" (abbreviated) in the status bar for 2 s.
- **D-05:** Auto-save: `App` holds `last_save: Instant`. Each poll iteration checks `if last_save.elapsed() >= Duration::from_secs(30)` — saves silently (no status bar flash). Also saves on graceful exit (before `ratatui::restore()`).
- **D-06:** Serialization: `#[derive(Serialize, Deserialize)]` on `CalcState`, `Stack`, `HpNum`, `AngleMode`, `DisplayMode`. Add a top-level wrapper: `{ "version": 1, "state": { ... } }` — the version field enables future migration without a breaking change.
- **D-07:** `HpNum` serializes as its string representation (via `serde_with` or a custom `serialize`/`deserialize` pair using `Decimal::to_string` / `Decimal::from_str`) — avoids floating-point JSON precision issues.

### STO / RCL Register-Number Entry
- **D-08:** Add `pending_input: Option<PendingInput>` to `App` (NOT CalcState — it is transient UI state). Variants: `StoRegister(String)`, `RclRegister(String)`, and one variant per STO-arithmetic op (`StoAdd`, `StoSub`, `StoMul`, `StoDiv` — each wrapping a `String` accumulator).
- **D-09:** STO key → sets `pending_input = Some(PendingInput::StoRegister(String::new()))` and shows `STO [__]` in the status bar. Digit keys append to the accumulator. After exactly 2 digits, auto-dispatch `Op::Sto(n)` and clear `pending_input`. Backspace clears the accumulator (not just last digit — resets the whole pending op). `Esc` cancels.
- **D-10:** RCL and all STO-arith variants follow the same pattern. Unambiguous key bindings: `S` = STO (uppercase), `R` = RCL (uppercase). These were placeholders in Phase 4's shift-key mapping table — wire them up now.
- **D-11:** Status bar shows the pending prompt at all times while `pending_input` is `Some(...)`, overriding the normal message.

### ALPHA Mode Keyboard Routing
- **D-12:** When `state.alpha_mode == true`, `App.handle_key()` routes ALL printable `KeyCode::Char(c)` events to `dispatch(state, Op::AlphaAppend(c.to_string()))` instead of the normal key-to-op table. No `pending_input` needed — it is a global routing mode.
- **D-13:** In ALPHA mode: `Backspace` → `dispatch(state, Op::AlphaClear)` then re-append all but last char (or add `Op::AlphaBackspace` to hp41-core). `Enter` or `A` key → `dispatch(state, Op::AlphaToggle)` to exit ALPHA mode. `Esc` also exits.
- **D-14:** Annunciator "ALPHA" is already rendered dim/bright in Phase 4 ui.rs — no change needed. Status bar shows `ALPHA mode — Enter or A to exit` while active.
- **D-15:** ALPHA toggle key: `a` (lowercase, matches Phase 4's existing stub `a` → AlphaToggle). Max 24 chars; `AlphaAppend` in hp41-core silently ignores chars that would exceed the limit.

### Help Overlay
- **D-16:** `App.show_help: bool`. `?` key toggles it. When `show_help == true`, the main render method layers a help overlay widget on top of the full terminal area.
- **D-17:** Overlay is a bordered `Block` widget (title "HP-41 Function Reference") rendered in a centered 80% × 90% `Rect`. Uses ratatui's `Table` widget with three columns: Key, Operation, Description.
- **D-18:** Content: static `&[(&str, &str, &str)]` array in `help_data.rs` — (key, op_name, description) for all ops, organized by category headers (Stack, Arithmetic, Trig, Math, Registers, ALPHA, Programming, Display, Persistence, USER). ~130 entries.
- **D-19:** Navigation: Up/Down (or `j`/`k`) scrolls the table. `Esc`, `q`, or `?` closes. Scroll state tracked as `App.help_scroll: usize`.
- **D-20:** No text search filter in v1.0 — scrollable table is sufficient for 130 entries. Category headers act as landmarks.

### Sample Program Library
- **D-21:** `hp41-cli/src/programs.rs` defines `struct SampleProgram { name: &'static str, description: &'static str, ops: &'static [Op] }` and a `static SAMPLE_PROGRAMS: &[SampleProgram]` array.
- **D-22:** Access: `Ctrl+P` opens the program library overlay. Same overlay pattern as help: centered bordered Block, list of program names with one-line descriptions. Enter loads the selected program into `state.program` (overwrites existing). If `state.program` is non-empty, show a confirmation prompt: "Load [name]? Current program will be lost. [Y/n]". Esc closes without loading.
- **D-23:** 10 required programs (all public-domain HP-41 classics / trivial algorithms):
  1. **Fibonacci** — generates Fibonacci sequence; demonstrates ISG loop
  2. **Prime Test** — tests if X is prime; demonstrates ISG/DSE + conditional tests
  3. **Quadratic Solver** — solves ax²+bx+c=0; demonstrates sqrt + CHS
  4. **Factorial** — n! via ISG loop; demonstrates counter loop
  5. **GCD** — greatest common divisor (Euclidean); demonstrates DSE + XY swap
  6. **Mean + StdDev** — computes mean and std dev from N entries using registers; demonstrates STO/RCL arith
  7. **Newton Root** — finds √x by Newton's method; demonstrates convergence loop
  8. **Unit Converter: °→rad** — converts degrees to radians and back; demonstrates math ops
  9. **Stack Stats** — min/max of N stack entries; demonstrates R↓ + conditional
  10. **Countdown Timer** — counts down from X using ISG/DSE; demonstrates display + loop
- **D-24:** ~~Programs stored as Rust `const` arrays of `Op` variants~~ **AMENDED 2026-05-07:** `Op::Lbl(String)` is heap-allocated and cannot appear in `const` context (Rust language constraint). Use `static SAMPLE_PROGRAMS: std::sync::OnceLock<Vec<SampleProgram>> = OnceLock::new();` initialized via `get_or_init()` at first access. Runtime-initialized but thread-safe and effectively a compile-time-verified lazy static. User approved this amendment.

### USER Mode
- **D-25:** Add to `CalcState`: `user_mode: bool` (default false), `key_assignments: std::collections::BTreeMap<char, String>` (key char → LBL name). Use `BTreeMap` (not `HashMap`) for deterministic serde serialization.
- **D-26:** `Op::UserMode` toggle: `u` key → dispatches `Op::UserMode` → flips `state.user_mode`.
- **D-27:** Assignment UX: `Ctrl+A` triggers a two-step `pending_input` chain:
  - Step 1: `PendingInput::AssignKey` → status bar shows "Assign: press key". Any printable char c → transitions to:
  - Step 2: `PendingInput::AssignLabel(c, String::new())` → status bar shows "Assign {c} → LBL: [____]". User types label name (letters/digits). Enter confirms (`state.key_assignments.insert(c, label)`). Esc cancels.
- **D-28:** In USER mode, when `keys.rs` sees a key that has an assignment: dispatch `run_program(state, &assigned_label)` instead of the default op. F1–F4 are pre-wired: F1→user key 'a', F2→user key 'b', F3→user key 'c', F4→user key 'd' (using the HP-41 convention of LBL A–D for user-defined programs).
- **D-29:** `BTreeMap<char, String>` derives `Serialize`/`Deserialize` automatically — no custom serde needed. Survives save/reload cycle with no extra work.

### Claude's Discretion
- `dirs` crate version: use `dirs = "5"` (latest stable as of 2026). Add to `hp41-cli/Cargo.toml`.
- `Op::AlphaBackspace` vs clearing full register: add `Op::AlphaBackspace` to hp41-core that removes the last character from `alpha_reg` (using `String::pop()`). This is the correct behavioral emulation of the HP-41 `←` (backspace) key in ALPHA mode.
- Overlay z-ordering in ratatui: render the main view first, then render the overlay using the same `frame` — ratatui renders widgets in draw order so the overlay naturally appears on top.
- `Ctrl+P` for program library (not `p` alone, which is already PRGM mode toggle).
- Auto-save error handling: if the auto-save write fails (e.g., disk full), show a one-time warning in the status bar but do not terminate the session. Retry on the next 30s tick.

### Folded Todos
- **Curate 10+ bundled sample programs from public domain HP Solutions books (Phase 5):** Folded into D-23 above. 10 programs defined, implemented as `const` Op arrays in `programs.rs`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture
- `hp41-core/src/state.rs` — CalcState struct (add `user_mode`, `key_assignments` fields here; add `#[derive(Serialize, Deserialize)]`)
- `hp41-core/src/ops/mod.rs` — Op enum + dispatch(); add `Op::UserMode`, `Op::AlphaBackspace` here
- `hp41-core/src/ops/alpha.rs` — existing AlphaToggle/AlphaAppend/AlphaClear; add AlphaBackspace
- `hp41-core/src/num.rs` — HpNum newtype; needs custom serde impl
- `hp41-core/src/format.rs` — format_hpnum() — used in help_data.rs display labels
- `hp41-core/src/ops/program.rs` — run_program() signature — used for USER mode dispatch and sample program execution
- `hp41-cli/src/app.rs` — App struct; auto-save timer and pending_input go here
- `hp41-cli/src/keys.rs` — key_to_op(); ALPHA routing and pending_input routing go here
- `hp41-cli/src/ui.rs` — render(); add help overlay and program library overlay widgets

### Requirements
- `.planning/REQUIREMENTS.md` §PERS-01, §PERS-02, §UX-01, §UX-02, §UX-03
- `.planning/ROADMAP.md` §Phase 5 — success criteria (all 5 must be TRUE)

### Phase 4 Deferred Items
- `.planning/phases/04-tui-and-input/04-CONTEXT.md` §Deferred — STO/RCL, ALPHA, F1–F4, auto-save timer hook, help overlay stub, label entry dialog — all land in this phase

### Prior Decisions
- `.planning/STATE.md` — serde_json chosen for persistence; no async in core; `event::poll()` required
- `./CLAUDE.md` — ratatui::init() mandate, KeyEventKind::Release filter

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App.run()` in `hp41-cli/src/app.rs` — the `// Phase 5: auto-save timer check goes here` comment marks the exact insertion point for D-05
- `hp41_core::dispatch(state, op)` — USER mode and STO/RCL dispatch go through this unchanged
- `hp41_core::run_program(state, label)` — used directly for USER mode key execution and sample program loading
- `hp41_core::CalcState` — will add `user_mode` + `key_assignments` fields; all existing fields need `#[derive(Serialize, Deserialize)]`
- `hp41_core::ops::{AlphaToggle, AlphaAppend, AlphaClear}` — AlphaBackspace is the only new op needed
- `ui.rs` overlay pattern: follow the same `Rect` centered rendering pattern for help and program overlays

### Established Patterns
- All CalcState mutation goes through `dispatch()` — STO/RCL pending input resolves to `Op::Sto(n)` / `Op::Rcl(n)` calls, not direct state mutation
- `App` owns transient UI state (`pending_input`, `show_help`, `show_programs`) separate from `CalcState` (serialized state)
- Status bar message pattern (`App.message: Option<String>`) already established — reuse for pending input prompts and save confirmations
- Phase 4 shift-key pattern: uppercase keys = shifted ops (S=STO, R=RCL) — wire these up in D-10

### Integration Points
- `hp41-cli/Cargo.toml`: add `dirs = "5"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"` (if not already present)
- `hp41-core/Cargo.toml`: add `serde = { version = "1", features = ["derive"] }` — must remain UI-agnostic (serde is fine in core)
- `main.rs`: load state file at startup (before `App::new()` or pass loaded CalcState to `App::new(state)`), pass `--state-file` arg value down to persistence module

</code_context>

<specifics>
## Specific Ideas

- `event::poll(Duration::from_millis(16))` loop: auto-save check slot already marked with a comment in `app.rs`. Use `std::time::Instant` (no tokio needed — serde write is sync and fast enough to not miss the 16ms poll window).
- Two-digit register entry: auto-dispatch on second digit (no Enter needed). This matches HP-41 hardware behavior where STO followed by two digits is a complete operation.
- Program library programs should all start with `Op::Lbl(LblId::Alpha("A"))` so `run_program(state, "A")` works uniformly for all of them after loading.
- `BTreeMap` vs `HashMap` for key_assignments: BTreeMap gives deterministic JSON key order (alphabetical) which matters for human-readable diffs of state files.
- Help data: generate the key mapping list from the same source as `keys.rs` to avoid them drifting — consider a single `const` table in `help_data.rs` that `keys.rs` also consults, rather than two independent lists.

</specifics>

<deferred>
## Deferred Ideas

- Text search / filter in help overlay — Phase 7 quality-of-life pass or v1.1
- Named save slots (multiple named files) — out of scope; `--state-file` CLI arg covers the use case
- In-TUI "Save As" dialog (typing a filename) — deferred to v1.1; CLI arg covers it
- ALPHA mode special characters (greek letters, math symbols via shifted keys) — deferred to v1.1; basic ASCII-visible chars are sufficient for v1.0
- GTO label-entry dialog (running a specific program by label from the TUI) — Phase 4 deferred `F5` runs LBL "A" hardcoded; proper label-entry dialog deferred to v1.1 or Phase 7 polish
- Mouse support — out of scope for v1.0 (already decided in Phase 4)
- Terminal resize handling — deferred; minimum 80×24 enforced with error message (already decided)

</deferred>

---

*Phase: 5-Persistence-and-UX*
*Context gathered: 2026-05-07*
