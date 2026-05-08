# HP-41 Calculator Emulator — Project Guide

## What this is

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator.
- `hp41-core` — UI-agnostic library crate; zero CLI/UI dependencies enforced by Cargo workspace
- `hp41-cli` — TUI binary (ratatui 0.30 + crossterm 0.29)
- `hp41-gui` — Tauri v2 desktop app (v2.0, deferred)

**Core invariant:** `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. Enforced at compile time.

**Status:** v1.0 shipped (2026-05-08) — 8 phases, 45 plans, 13,399 lines Rust.

## Git Workflow

**Commits:** Always use `/git-workflow:commit --with-skills` — never commit directly via `git commit`.

**Commit language: English only.** All commit messages (subject line and body) must be written in English, regardless of any global or plugin defaults that specify another language.

## GSD Workflow

Planning artifacts live in `.planning/`. v1.0 is complete; next milestone is v1.1.

```
/gsd-progress           — check current status
/gsd-new-milestone      — start v1.1 planning
```

Phase history (v1.0): Foundation → Core Math → Programming Engine → TUI & Input → Persistence & UX → Science & Engineering → Hardening → Tech Debt Cleanup

## Settled Architecture Decisions

These decisions are final for v1.0 — do not revisit without strong justification:

- **BCD/f64:** `rust_decimal` wrapping f64 with 10-significant-digit rounding. Custom BCD was evaluated and rejected. `HpNum` in `hp41-core/src/num.rs`.
- **Stack-lift:** `lift_enabled: bool` in `Stack`. Every one of ~130 operations declares `LiftEffect::Enable / Disable / Neutral` in `ops/`.  The most commonly mis-implemented HP-41 feature — always check.
- **ISG/DSE counter:** Fields extracted by string-splitting at the decimal point — **never** `floor()`/`fmod()` on f64. See `ops/program.rs::parse_counter()`.
- **TUI:** Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook. Filter `KeyEventKind::Release` on Windows immediately or every op fires twice.
- **No async in core:** Event loop is `poll(timeout) → update → redraw`, single-threaded throughout v1.0.
- **Zero panics in `hp41-core`:** `#![deny(clippy::unwrap_used)]` is active at the crate root (`hp41-core/src/lib.rs`). All production code must use `.expect("reason")` or proper `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`.
- **EEX entry:** `flush_entry_buf()` in `ops/mod.rs` tries `Decimal::from_str()` then falls back to `Decimal::from_scientific()`. Entry_buf guards in `app.rs` block duplicate `.` and `e`, and `e` without a preceding mantissa.
- **Key bindings (Phase 8):** `'q'` → `Op::Sin`, `'g'` → `Op::Clreg`, `Delete` in ALPHA mode → `Op::AlphaClear`. `'S'` opens STO register modal (handled before `key_to_op()`). Quit is `Ctrl+C` only.
- **Coverage gate:** `just coverage` runs `cargo llvm-cov clean --workspace` first to discard stale `.profraw` data from worktree runs before measuring.

## Tech Stack (v1.0)

- Rust stable 1.78+, Cargo workspace
- **`just`** — sole task runner; all build/test/lint/run/ci targets are `just` recipes. **Never call `cargo` directly in CI or docs.**
- `rust_decimal` (HpNum BCD-accurate arithmetic)
- ratatui 0.30 + crossterm 0.29 (TUI)
- serde + serde_json (state persistence, human-readable JSON)
- proptest (property tests for stack invariants)
- cargo-llvm-cov (coverage gate: ≥80% on `hp41-core`)
- criterion (dispatch benchmarks — advisory, not CI-gated)
- clap 4.x (CLI argument parsing)

## Quality Gates (v1.0 Achieved Values)

| Gate | Target | Achieved |
|------|--------|---------|
| Cold-start | ≤ 0.5 s | 2.2 ms (M1) |
| Key latency | ≤ 50 ms median | ~65 ns/op |
| Numerical accuracy | ≥ 98% (500 cases) | 99% (495/500) |
| `hp41-core` coverage | ≥ 80% | 94.87% |
| Panics in `hp41-core` | 0 | 0 |
| CI | Win 10+, macOS 12+, Ubuntu 22.04+ | ✅ all green |

## Key Files

| File | Purpose |
|------|---------|
| `hp41-core/src/ops/mod.rs` | Op enum, `dispatch()`, `flush_entry_buf()` — central integration hub |
| `hp41-core/src/state.rs` | `CalcState` — single source of truth for all calculator state |
| `hp41-core/src/stack.rs` | `Stack`, `apply_lift_effect()` |
| `hp41-core/src/ops/program.rs` | `run_program()`, `run_loop()`, `parse_counter()` — ISG/DSE logic |
| `hp41-cli/src/app.rs` | `App`, `handle_key()`, `handle_alpha_mode_key()`, event loop |
| `hp41-cli/src/keys.rs` | `key_to_op()`, `KEY_REF_TABLE` — keyboard mapping |
| `hp41-cli/src/help_data.rs` | `HELP_DATA` — SINGLE SOURCE OF TRUTH for key descriptions in `?` overlay |
| `hp41-cli/src/persistence.rs` | `save_state()`, `load_state()` — JSON serde |
| `hp41-core/tests/numerical_accuracy.rs` | 500-case accuracy suite — must stay ≥ 490 passing |
