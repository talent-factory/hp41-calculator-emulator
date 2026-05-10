# HP-41 Calculator Emulator — Project Guide

## What this is

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator.
- `hp41-core` — UI-agnostic library crate; zero CLI/UI dependencies enforced by Cargo workspace
- `hp41-cli` — TUI binary (ratatui 0.30 + crossterm 0.29)
- `hp41-gui` — Tauri v2 + React + TypeScript desktop app (nested standalone workspace)

**Core invariant:** `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. Enforced at compile time. Root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; `hp41-gui` is a nested standalone workspace.

**Status:**
- v1.0 CLI shipped 2026-05-08 — 8 phases, 45 plans
- v1.1 CLI Feature Completeness shipped 2026-05-09 — Phases 9–12, 13 plans (EEX-fix, STO arithmetic modals, print emulation, synthetic programming)
- v2.0 Tauri GUI shipped 2026-05-10 — Phases 13–18, 19 plans (pixel-perfect HP-41C desktop app)

## Git Workflow

**Commits:** Always use `/git-workflow:commit --with-skills` — never commit directly via `git commit`.

**Commit language: English only.** All commit messages (subject line and body) must be written in English, regardless of any global or plugin defaults that specify another language.

## GSD Workflow

Planning artifacts live in `.planning/`. v1.0 + v1.1 + v2.0 are shipped and archived under `.planning/milestones/`. Next milestone: v2.1 Polish.

```
/gsd-progress           — check current status
/gsd-new-milestone      — start v2.1 planning
```

**Phase history:**
- v1.0 (1–8): Foundation → Core Math → Programming Engine → TUI & Input → Persistence & UX → Science & Engineering → Hardening → Tech Debt Cleanup
- v1.1 (9–12): Infrastructure & EEX Fix → STO Arithmetic Modals → Print Emulation → Synthetic Programming
- v2.0 (13–18): Workspace Skeleton → IPC Layer → Display & Keyboard → SVG Skin → Persistence & Print Output → Program Listing & CI/CD

## Settled Architecture Decisions

These decisions are final — do not revisit without strong justification:

### Core engine (v1.0)

- **BCD/f64:** `rust_decimal` wrapping f64 with 10-significant-digit rounding. Custom BCD was evaluated and rejected. `HpNum` in `hp41-core/src/num.rs`.
- **Stack-lift:** `lift_enabled: bool` in `Stack`. Every one of ~130 operations declares `LiftEffect::Enable / Disable / Neutral` in `ops/`. The most commonly mis-implemented HP-41 feature — always check.
- **ISG/DSE counter:** Fields extracted by string-splitting at the decimal point — **never** `floor()`/`fmod()` on f64. See `ops/program.rs::parse_counter()`.
- **TUI:** Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook. Filter `KeyEventKind::Release` on Windows immediately or every op fires twice.
- **No async in core:** Event loop is `poll(timeout) → update → redraw`, single-threaded throughout. The hp41-gui spawns a separate auto-save thread but `hp41-core` itself stays single-threaded.
- **Zero panics in `hp41-core`:** `#![deny(clippy::unwrap_used)]` is active at the crate root (`hp41-core/src/lib.rs`). All production code must use `.expect("reason")` or proper `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`. Mutex locks in `hp41-gui` use `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery.
- **Key bindings (Phase 8):** `'q'` → `Op::Sin`, `'g'` → `Op::Clreg`, `Delete` in ALPHA mode → `Op::AlphaClear`. `'S'` opens STO register modal (handled before `key_to_op()`). Quit is `Ctrl+C` only.
- **Coverage gate:** `just coverage` runs `cargo llvm-cov clean --workspace` first to discard stale `.profraw` data from worktree runs before measuring.

### v1.1 additions

- **EEX trailing-e (hardware-faithful):** `flush_entry_buf()` appends `"00"` before the parse chain (`Decimal::from_str` → `Decimal::from_scientific`); empty-buffer EEX inserts implicit mantissa `"1"`; `format_entry_buf_display()` in `hp41-cli/src/ui.rs` renders the underscore placeholder cursor. **Never** discard a trailing-e number silently.
- **STO arithmetic modal:** 3-step keyboard flow `S → +/−/×/÷ → R00–R99 | Y/Z/T/L`. `StackReg` enum + `Op::StoArithStack` variant in `ops/mod.rs`; `op_sto_arith_stack()` in `registers.rs`. `pending_input` routing block must remain ABOVE modal-opening interceptors (`S`/`R`/`Ctrl+A`) so an active modal is not silently discarded.
- **Print emulation:** `print_buffer: Vec<String>` field on `CalcState` with `#[serde(skip)]` (transient, never persisted). `ops/print.rs::{PRX, PRA, PRSTK}` push lines into the buffer; `println!`/`eprintln!` are forbidden inside `hp41-core`. `hp41-cli` drains via `call_dispatch_and_drain()` (interactive) and `drain_and_show_print_output()` (programmatic `run_program` paths — wire ALL `run_program()` call sites or print output gets dropped).
- **Synthetic programming:** `last_key_code`, `reg_m`, `reg_n`, `reg_o` fields on `CalcState`, all with `#[serde(default)]` for backward-compat with v1.0 save files. `HexModal(String)` 2-digit accumulation modal; `synthetic_byte_to_op()` validates against the 23-entry safe subset **before** `state.program.insert()` (security invariant T-12-W2-02). `keycode_to_hp41_code()` in `hp41-cli/src/keys.rs` uses row×10+col encoding. F5 / R / S code paths reset `last_key_code` to 0 BEFORE GETKEY runs.
- **MSRV:** declared at `[workspace.package]` (`rust-version = "1.88"`); member crates inherit via `rust-version.workspace = true`. CI MSRV job runs in parallel — no `needs:`.

### v2.0 additions (Tauri GUI)

- **Nested workspace isolation:** Root `Cargo.toml` `members = ["hp41-core", "hp41-cli"]` — never add `hp41-gui` here. `tauri` / `tauri-build` must appear ONLY in `hp41-gui/src-tauri/Cargo.toml`, never in root `[workspace.dependencies]`.
- **Bundle identifier:** `ch.talent-factory.hp41` (overrides scaffold default `com.tauri.dev`; avoids macOS sandbox/keychain issues).
- **IPC contract:** `dispatch_op(key_id: &str)` and `get_state()` Tauri v2 commands; response is `CalcStateView` (~170 bytes, JSON budget ≤300 bytes). `key_map::resolve()` in `hp41-gui/src-tauri/src/key_map.rs` maps string IDs to `Op` variants — frontend never touches Rust enums. `print_buffer` is drained on every command response.
- **SC-4 invariant (no core duplication):** `grep -rn "fn op_\|fn flush_entry\|fn format_hpnum" hp41-gui/src-tauri/src/` MUST return nothing. If this fails, hp41-gui has duplicated `hp41-core` logic — fix immediately.
- **Tauri v2.11 permissions:** For inline app commands (not plugins), Tauri does NOT auto-generate `allow-<cmd>` permissions. Create TOML in `hp41-gui/src-tauri/permissions/<cmd-kebab>.toml` with `[[permission]] identifier + commands.allow = ["fn_name"]`, then reference the kebab-case ID in `capabilities/default.json`. Run a `cargo check` first so the permission registry is generated.
- **SVG animation:** `transform-box: fill-box` + `transform-origin: center` on `.key` is REQUIRED for SVG `scale()` to animate from each key's own center; without it, keys translate from the canvas origin instead of shrinking in place.
- **busyRef debounce:** `useRef(false)` pattern in both `App.tsx` (handleClick) and `Keyboard.tsx` (handleKeyClick) — two-layer guard against concurrent `invoke()` calls. Always pair with `pressedKey` state machine using functional setState form to avoid stale closure (Pitfall 4).
- **Persistence sharing:** `hp41-gui` reads/writes the SAME `~/.hp41/autosave.json` file as `hp41-cli`. `serde(default)` on every `CalcState` field added since v1.0 keeps v1.x save files loadable. Auto-save thread releases the `AppState` Mutex BEFORE disk I/O (commit ff39017 fix).
- **`Op` variants land before TUI code:** Every new `Op` variant must appear in BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs` AND the `prgm_display.rs` exhaustive match before any caller (`hp41-cli` or `hp41-gui`) can compile.

## Tech Stack

**Core / CLI (v1.0 + v1.1):**
- Rust stable, MSRV `1.88` (declared in `[workspace.package]`)
- **`just`** — sole task runner; all build/test/lint/run/ci targets are `just` recipes. **Never call `cargo` directly in CI or docs.** GUI recipes: `just gui-dev` / `just gui-build` / `just gui-ci` / `just gui-check`.
- `rust_decimal` 1.42 (HpNum BCD-accurate arithmetic)
- ratatui 0.30 + crossterm 0.29 (TUI)
- serde + serde_json (state persistence, human-readable JSON)
- proptest (property tests for stack invariants)
- cargo-llvm-cov (coverage gate: ≥80% on `hp41-core`)
- criterion (dispatch benchmarks — advisory, not CI-gated)
- clap 4.x (CLI argument parsing)

**GUI (v2.0):**
- Tauri v2.11 (Rust desktop runtime — nested standalone workspace in `hp41-gui/src-tauri/`)
- React 18 + TypeScript + Vite (frontend in `hp41-gui/src/`)
- `dirs` crate (resolves `~/.hp41/autosave.json` shared with hp41-cli)
- Two-layer CI: `ci.yml` (CLI, unchanged) + `ci-gui.yml` (3-OS matrix, path-filtered to `hp41-gui/**` and `hp41-core/**`, runs `cargo test` before `cargo build --release`)

## Quality Gates (maintained across v1.0 → v2.0)

| Gate | Target | v1.0 | v1.1 / v2.0 |
|------|--------|------|-------------|
| Cold-start | ≤ 0.5 s | 2.2 ms (M1) | unchanged (CLI); GUI cold-start not gated |
| Key latency | ≤ 50 ms median | ~65 ns/op | unchanged |
| Numerical accuracy | ≥ 98% (500 cases) | 99% (495/500) | unchanged |
| `hp41-core` coverage | ≥ 80% | 94.87% | 94%+ |
| Panics in `hp41-core` | 0 | 0 | 0 |
| CI | Win 10+, macOS 12+, Ubuntu 22.04+ | ✅ `ci.yml` | ✅ `ci.yml` + `ci-gui.yml` (independent) |
| MSRV | declared | — | 1.88 (CI-enforced) |

## Key Files

**Core engine:**

| File | Purpose |
|------|---------|
| `hp41-core/src/ops/mod.rs` | `Op` enum, `dispatch()`, `flush_entry_buf()` — central integration hub |
| `hp41-core/src/state.rs` | `CalcState` — single source of truth (incl. `print_buffer`, `last_key_code`, `reg_m/n/o`) |
| `hp41-core/src/stack.rs` | `Stack`, `apply_lift_effect()` |
| `hp41-core/src/ops/program.rs` | `run_program()`, `run_loop()`, `parse_counter()`, `execute_op()` — ISG/DSE logic |
| `hp41-core/src/ops/print.rs` | `op_prx()`, `op_pra()`, `op_prstk()` — buffer-only, NO `println!` |
| `hp41-core/src/ops/registers.rs` | `op_sto_arith()`, `op_sto_arith_stack()`, M/N/O hidden-register ops |
| `hp41-core/src/synthetic.rs` | `synthetic_byte_to_op()` — 23-entry safe subset validator |
| `hp41-core/tests/numerical_accuracy.rs` | 500-case accuracy suite — must stay ≥ 490 passing |

**TUI (`hp41-cli`):**

| File | Purpose |
|------|---------|
| `hp41-cli/src/app.rs` | `App`, `handle_key()`, `handle_alpha_mode_key()`, `PendingInput`, event loop, `call_dispatch_and_drain()`, `drain_and_show_print_output()` |
| `hp41-cli/src/keys.rs` | `key_to_op()`, `KEY_REF_TABLE`, `keycode_to_hp41_code()` |
| `hp41-cli/src/ui.rs` | `format_entry_buf_display()` — EEX placeholder cursor; `pending_prompt()` exhaustive match |
| `hp41-cli/src/help_data.rs` | `HELP_DATA` — SINGLE SOURCE OF TRUTH for key descriptions in `?` overlay |
| `hp41-cli/src/persistence.rs` | `save_state()`, `load_state()` — JSON serde |

**GUI (`hp41-gui`):**

| File | Purpose |
|------|---------|
| `hp41-gui/src-tauri/src/lib.rs` | `setup()`, `AppState = Mutex<CalcState>`, 30s auto-save thread, `generate_handler!` registration |
| `hp41-gui/src-tauri/src/commands.rs` | `dispatch_op`, `get_state`, `sst_step`, `bst_step` Tauri thunks + `handle_op`/`handle_get_state` helpers |
| `hp41-gui/src-tauri/src/types.rs` | `CalcStateView`, `Annunciators`, `GuiError`, `From<HpError>` |
| `hp41-gui/src-tauri/src/key_map.rs` | `resolve()` — string ID → `Op`; SC-4 invariant (no `op_*`/`flush_*`/`format_hpnum` here) |
| `hp41-gui/src-tauri/src/persistence.rs` | Shared `~/.hp41/autosave.json` (same schema as hp41-cli) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | `format_all_steps()` — always appends END so `pc == program.len()` highlights correctly |
| `hp41-gui/src-tauri/permissions/*.toml` | Tauri v2.11 inline-command permission registry |
| `hp41-gui/src/App.tsx` | React root: display, annunciators, stack panel, keyboard listener, `busyRef`, `resolveKeyId`, program panel |
| `hp41-gui/src/Keyboard.tsx` | 44-key SVG skin; `KEY_DEFS`, `handleKeyClick`, `pressedKey` state, `getKeyColor()` |
| `hp41-gui/src/App.css` | Layout, key animation, program panel styles; requires `transform-box: fill-box` on `.key` |
