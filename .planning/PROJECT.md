# HP-41 Calculator Emulator

## Current Milestone: v2.0 Tauri GUI

**Goal:** Ship a pixel-perfect HP-41C desktop app using Tauri v2 + React + TypeScript, reusing `hp41-core` unchanged alongside the existing CLI.

**Target features:**
- `hp41-gui` Tauri v2 binary in the existing Cargo workspace
- SVG skin: vector HP-41C key layout with clickable regions per key
- Display panel: 12-char dot-matrix display + annunciators in the GUI
- `hp41-core` integration via Tauri Rust commands (no core duplication)
- `hp41-cli` remains fully functional (both binaries ship)
- Platform targets: macOS (primary), Windows, Linux

## What This Is

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator, delivered as a keyboard-driven TUI CLI (`hp41-cli`) backed by a UI-agnostic core library (`hp41-core`). v1.0 shipped on 2026-05-08 with complete HP-41 arithmetic, keystroke programming, persistence, and cross-platform CI. v2.0 will add a Tauri-based graphical desktop app reusing `hp41-core` unchanged.

## Core Value

Faithful HP-41 RPN fidelity — the four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to the original hardware; everything else is secondary.

## Requirements

### Validated (v1.0)

- ✓ CORE-01: 4-level RPN stack (X/Y/Z/T) and LASTX register — v1.0 Phase 1
- ✓ CORE-02: Stack-lift semantics for all ~130 ops (Enable/Disable/Neutral) — v1.0 Phase 1
- ✓ MATH-01: Core arithmetic with 10-digit accuracy — v1.0 Phase 2
- ✓ MATH-02: Trig in DEG/RAD/GRAD; SIN on 'q' key — v1.0 Phases 2+8
- ✓ MATH-03: FIX/SCI/ENG formatting with mantissa carry — v1.0 Phase 2
- ✓ REGS-01: STO/RCL/CLREG; CLREG on 'g' key — v1.0 Phases 2+8
- ✓ ALPH-01: 24-char ALPHA mode; AlphaClear on Delete — v1.0 Phases 2+8
- ✓ PROG-01: Keystroke programming (LBL/GTO/XEQ/RTN/conditionals/ISG/DSE) — v1.0 Phase 3
- ✓ PROG-02: ISG/DSE CCCCC.FFFDD counter (string-split, not float) — v1.0 Phase 3
- ✓ DISP-01: 12-char HP-41 display + annunciators in TUI — v1.0 Phase 4
- ✓ DISP-02: Persistent T/Z/Y/X/LASTX panel — v1.0 Phase 4
- ✓ INPUT-01: All functions via keyboard; EEX functional — v1.0 Phases 4+8
- ✓ PERS-01: JSON save/load with full CalcState serde — v1.0 Phase 5
- ✓ PERS-02: 30s auto-save + exit save — v1.0 Phase 5
- ✓ UX-01: '?' help overlay (accurate key reference) — v1.0 Phases 5+8
- ✓ UX-02: USER mode + custom key assignments — v1.0 Phase 5
- ✓ UX-03: 10 bundled sample programs — v1.0 Phase 5
- ✓ SCI-01: Statistics (Σ+/−, MEAN, SDEV, L.R.) — v1.0 Phase 6
- ✓ SCI-02: HMS↔H conversions — v1.0 Phase 6
- ✓ QUAL-01: Cold-start 2.2ms (≤500ms) — v1.0 Phase 7
- ✓ QUAL-02: ~65 ns/op dispatch (≤50ms) — v1.0 Phase 7
- ✓ QUAL-03: Zero panics in hp41-core — v1.0 Phase 7
- ✓ QUAL-04: 94.87% test coverage (≥80%) — v1.0 Phase 7
- ✓ QUAL-05: CI green on Windows/macOS/Ubuntu — v1.0 Phase 7
- ✓ QUAL-06: 495/500 accuracy cases (99% ≥ 98%) — v1.0 Phase 7

### Validated (v1.1)

- ✓ EEX trailing-e-without-exponent behavior (HP-41 hardware lock on partial exponent) — v1.1 Phase 9 (2026-05-08)
- ✓ MSRV 1.85 enforcement + rust_decimal 1.42 — v1.1 Phase 9 (2026-05-08)
- ✓ STO arithmetic keyboard modals (STO+/-/×/÷ interactive key binding) — v1.1 Phase 10 (2026-05-08)
- ✓ FR-17: Print emulation (PRX/PRA/PRSTK) to console/text file — v1.1 Phase 11 (2026-05-08)
- ✓ FR-20: Synthetic programming (GETKEY, NULL, hidden registers M/N/O, HexModal) — v1.1 Phase 12 (2026-05-09)

### Active (v2.0)

- [ ] GUI-01: `hp41-gui` Tauri v2 binary added to Cargo workspace; builds and launches on macOS, Windows, Linux
- [ ] GUI-02: SVG skin renders pixel-perfect HP-41C key layout; all keys are visually distinct and correctly positioned
- [ ] GUI-03: Clickable keys in the SVG skin trigger the same `Op` dispatch as their CLI keyboard counterparts
- [ ] GUI-04: HP-41 12-char dot-matrix display and annunciators render in the GUI, updating after every op
- [ ] GUI-05: `hp41-core` integrated via Tauri Rust commands — no duplication of CalcState logic
- [ ] GUI-06: `hp41-cli` remains fully functional and unmodified after adding `hp41-gui` to the workspace

### Out of Scope

- v2.0 GUI advanced features (module emulation, skin themes) — deferred until core GUI is stable
- FR-18 Multiple skin themes — GUI-only, v2.0
- FR-21 Module emulation (Math/Stat/Time/Advantage) — could-have, v1.2+
- FR-22 `.raw` HP-41 program file import/export — could-have, v1.2+
- FR-23 Mobile (iOS/Android) — defer until desktop stable
- Cycle-accurate Nut CPU simulation — high effort, low user value vs. behavioral emulation
- HP-copyrighted ROM image redistribution — legal risk, excluded permanently
- HP-IL peripheral emulation — niche, complex
- Wand/barcode reader emulation — requires hardware, very niche
- Cloud sync — privacy and infrastructure cost

## Context

v1.0 shipped in 3 days (2026-05-06 → 2026-05-08) with 8 phases, 45 plans, and 13,399 lines of Rust across `hp41-core` and `hp41-cli`. The faithful stack-lift semantics and ISG/DSE counter logic (CCCCC.FFFDD string-split) were the most commonly mis-implemented HP-41 features — both are now correctly implemented and verified.

v1.1 Phase 9 (2026-05-08): MSRV formally declared at 1.85 with workspace inheritance in member crates; CI MSRV job added; EEX hardware behavior corrected — trailing-e commits as exponent 00, empty-buffer EEX inserts implicit mantissa, TUI shows placeholder cursor. 461 tests pass; 5/5 success criteria verified.

v1.1 Phase 10 (2026-05-08): STO arithmetic keyboard modal complete — S→op→register 3-step flow for R00–R99 and stack registers Y/Z/T/LASTX. `StackReg` enum + `Op::StoArithStack` + `op_sto_arith_stack()` added to hp41-core; step-2 routing and Y/Z/T/L dispatch wired in app.rs; help overlay corrected. 10/10 must-haves verified; human TUI tests approved.

v1.1 Phase 11 (2026-05-08): Print emulation complete — `print_buffer: Vec<String>` on `CalcState` keeps hp41-core I/O-free; `Op::PRX/PRA/PRSTK` in `ops/print.rs` format output into the buffer; `hp41-cli` drains via `call_dispatch_and_drain()` (interactive) and `drain_and_show_print_output()` (programmatic run_program paths); `--print-log` appends to a file. 5/5 must-haves verified; 94.00% hp41-core coverage. Gap closure plan 11-03 fixed serde(skip) on print_buffer (CR-03) and wired 3 run_program call sites (CR-01).

Key codebase facts: `hp41-core` is a UI-agnostic library with zero CLI dependencies; `hp41-cli` uses ratatui 0.30 + crossterm 0.29; all tests use `just ci` (lint + test + coverage); `#![deny(clippy::unwrap_used)]` enforces zero panics at compile time. v1.1 shipped 2026-05-09 with 4 phases (9–12), completing all planned CLI features. v2.0 adds `hp41-gui` (Tauri v2 + React + TypeScript) as a new workspace member reusing `hp41-core` unchanged.

## Constraints

- **Tech stack**: Rust stable 1.78+ — deterministic, GC-free, ideal for emulation core
- **Task runner**: `just` — sole task runner; no bare `cargo` commands in CI or docs
- **Architecture**: `hp41-core` must never depend on `hp41-cli` or `hp41-gui` — enforced at compile time
- **Dependencies**: ratatui 0.30, crossterm 0.29, clap 4.x, serde/serde_json, rust_decimal, criterion (dev)
- **Legal**: No HP-copyrighted ROM bytes; license audit before public release
- **Privacy**: No telemetry; local-only data storage; no network calls

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Behavioral emulation, not cycle-accurate Nut CPU | High effort, low user value | ✓ Good — users don't notice |
| Cargo workspace `hp41-core` / `hp41-cli` | Enforces clean separation; GUI is thin adapter | ✓ Good — `hp41-core` reused unchanged |
| `rust_decimal` for HpNum with 10-digit rounding | BCD-accurate without custom BCD struct | ✓ Good — 99% accuracy suite pass rate |
| Stack-lift as `lift_enabled: bool` in Stack | Simplest correct model for 130+ ops | ✓ Good — every op explicitly declares effect |
| ISG/DSE counter fields via string-split | Never use `floor()`/`fmod()` on f64 | ✓ Good — hardware-identical counter behavior |
| ratatui + crossterm for TUI | Cross-platform, keyboard-driven, stable | ✓ Good — CI green on all 3 platforms |
| `ratatui::init()` not `Terminal::new()` | Installs panic hook for terminal restore | ✓ Good — terminal never left in raw mode |
| `just` as sole task runner | All targets as recipes; contributors never call bare `cargo` | ✓ Good — CI compliance enforced |
| No async in hp41-core | Single-threaded event loop throughout v1.0 | ✓ Good — simpler, deterministic |
| `serde_json` for persistence | Human-readable, diff-able, forward-compatible | ✓ Good — users can inspect/backup state files |
| Digit entry appends to `entry_buf` directly | Auto-flushed on next non-digit; avoids per-digit PushNum | ✓ Good — correct HP-41 number entry behavior |
| Phase 8 tech debt closure before v1.0 tag | EEX/SIN/CLREG gaps found in audit | ✓ Good — all keyboard gaps closed before release |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each milestone** (via `/gsd-complete-milestone`):
1. Move shipped requirements from Active → Validated
2. Add new requirements to Active for next milestone
3. Update Context with current state
4. Audit Key Decisions with outcomes

---
v2.0 Phase 14 (2026-05-09): IPC Layer complete — `dispatch_op` and `get_state` Tauri v2 commands route key string IDs through `key_map.rs` to `hp41_core::ops::dispatch`; `CalcStateView` (~170 bytes, ≤300 limit) serializes state for the frontend; `print_buffer` drained on every command; Tauri v2.11 app-command permissions declared via TOML files in `src-tauri/permissions/` (auto-generation not available for inline commands). 5/5 SC verified, 9/9 unit tests GREEN.

*Last updated: 2026-05-09 — Phase 14 IPC Layer complete; Phase 15 Display & Keyboard next*
