# Research Summary: HP-41 Calculator Emulator

**Synthesized from:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md
**Date:** 2026-05-06

---

## Stack

- **Rust 2024 edition + Cargo workspace** enforces the `hp41-core` / `hp41-cli` boundary at the compiler level. Use ratatui 0.30 + crossterm 0.29 for the TUI; crossterm is the only backend with Windows 10+ support. Do NOT add tokio — the keyboard-driven event loop is synchronous throughout v1.0.
- **No async for v1.0.** The event loop is `poll(timeout) → update state → redraw`, all in one thread. Tokio enters only if Tauri is adopted in v2.0 (in `hp41-gui`, never in `hp41-core`).
- **serde_json for state persistence** (human-readable, shareable), proptest + insta for testing, cargo-llvm-cov for ≥80% core coverage. Avoid bincode/postcard as primary formats — users share program files as text.

---

## Features

- **The market gap is clear:** no cross-platform CLI/TUI HP-41 emulator exists. V41 is Windows-only, go41cx is Android-only, CC41 lacks a live stack panel and state persistence. The killer differentiator is a persistent ratatui stack/annunciator panel combined with cross-platform native binaries.
- **MVP sequence is fixed by dependencies:** Stack + stack-lift → arithmetic/trig/formatting → registers → ALPHA mode → keystroke programming → TUI panel → state persistence → built-in help. Deviating from this order causes rewrites.
- **Hard anti-features for v1.0:** no cycle-accurate Nut CPU, no HP ROM bytes, no `.raw` file import, no module emulation, no HP-IL peripherals, no network calls. These either carry legal risk or consume disproportionate effort with zero user-visible benefit over behavioral emulation.

---

## Architecture

- **`CalcState` as single source of truth:** one owned struct passed as `&mut CalcState` through all operations. No global mutable state, no `Arc<Mutex<>>`. This is the only ownership model that avoids lifetime gymnastics in Rust and keeps hp41-core trivially testable.
- **Elm Architecture (TEA) for the TUI:** `App → Message → update() → render()`. The `update()` function is the only place state mutates; `render()` is side-effect-free. This separates calculator logic cleanly from display logic and makes each function independently unit-testable.
- **Closed-world `Instruction` enum, not `dyn Trait`:** The HP-41 instruction set is fixed and known at compile time. Enum dispatch is 10x faster, serializable with serde, and exhaustively checked by the compiler. The 4-level return stack is a fixed array — not a `Vec` — matching the hardware constraint exactly.

---

## Pitfalls

- **BCD vs f64 (Critical):** The HP-41 uses 10-digit BCD internally. Rust `f64` diverges at digit 9–10 for transcendental functions and accumulation. Decide on a decimal arithmetic representation (`rust_decimal` or a custom BCD struct) before writing any register code — retrofitting is a full data model rewrite. Trig results must be rounded to 10 significant decimal digits after computation.
- **Stack-lift semantics and ISG/DSE format (Critical):** Stack-lift is the most commonly mis-implemented HP-41 feature. Every one of ~130 operations must declare its effect as Enable / Disable / Neutral. ISG/DSE counter fields (`CCCCC.FFFDD`) must be extracted by string-splitting at the decimal point — never with `floor()`/`fmod()` on f64, which miscomputes near power-of-10 boundaries.
- **Cross-platform TUI traps (Critical):** Crossterm on Windows fires both `KeyEventKind::Press` and `KeyEventKind::Release`; filter immediately or every operation executes twice. Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook — without it, any unhandled panic leaves the user's terminal in raw mode. Use `event::poll(timeout)` rather than `event::read()` to support the 30-second auto-save timer without blocking redraws.
