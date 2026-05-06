# HP-41 Calculator Emulator — Project Guide

## What this is

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator.
- `hp41-core` — UI-agnostic library crate; zero CLI/UI dependencies enforced by Cargo workspace
- `hp41-cli` — TUI binary (ratatui + crossterm)
- `hp41-gui` — Tauri v2 desktop app (v2.0, deferred)

**Core invariant:** `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. Enforced at compile time.

## Git Workflow

**Commits:** Always use `/git-workflow:commit --with-skills` — never commit directly via `git commit`.

## GSD Workflow

Planning artifacts live in `.planning/`. Current state: roadmap created, Phase 1 ready to start.

```
/gsd-discuss-phase 1    — gather context before planning
/gsd-plan-phase 1       — create PLAN.md for Phase 1
/gsd-execute-phase 1    — execute plans
/gsd-progress           — check status / advance workflow
```

Phases: Foundation → Core Math → Programming Engine → TUI & Input → Persistence & UX → Science & Engineering → Hardening

## Critical Architecture Decisions

- **BCD/f64:** Decide in Phase 1 before any register code. Retrofitting = full data model rewrite.
  Use `rust_decimal` or a custom BCD struct; round all trig results to 10 significant decimal digits.
- **Stack-lift:** Every one of ~130 operations must declare Enable / Disable / Neutral. The most commonly mis-implemented HP-41 feature.
- **ISG/DSE counter:** Extract fields by string-splitting at the decimal point — never with `floor()`/`fmod()` on f64.
- **TUI:** Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook. Filter `KeyEventKind::Release` on Windows immediately or every operation fires twice.
- **No async in core:** Event loop is `poll(timeout) → update → redraw`, single-threaded throughout v1.0.

## Tech Stack (v1.0)

- Rust stable 1.78+, Cargo workspace
- **`just`** — sole task runner; all build/test/lint/run/ci targets are `just` recipes. Never call `cargo` directly in CI or docs.
- ratatui 0.30 + crossterm 0.29 (TUI)
- serde + serde_json (state persistence, human-readable)
- proptest + insta (property/snapshot tests)
- cargo-llvm-cov (≥80% coverage gate on `hp41-core`)
- clap 4.x (CLI argument parsing)

## Quality Gates

- Cold-start ≤ 0.5 s (M1 / Intel i5 8th gen)
- Key latency ≤ 50 ms median
- ≥98% numerical agreement vs HP-41 hardware (500-case suite)
- ≥80% unit-test coverage in `hp41-core`
- Zero panics in `hp41-core`
- CI: Windows 10+, macOS 12+, Ubuntu 22.04+
