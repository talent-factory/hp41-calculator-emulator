# HP-41 Calculator Emulator

## What This Is

A faithful Rust-based software emulation of the HP-41C/CV/CX programmable RPN calculator, targeting engineers, scientists, hobbyists, and retro-computing enthusiasts who rely on — or want to learn — the HP-41 workflow. v1.0 delivers a fully keyboard-driven CLI (`hp41-cli`) backed by a UI-agnostic core library (`hp41-core`). v2.0 adds a Tauri-based graphical desktop app reusing `hp41-core` unchanged.

## Core Value

Faithful HP-41 RPN fidelity — the four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to the original hardware; everything else is secondary.

## Requirements

### Validated

(None yet — ship to validate)

### Active

**v1.0 CLI — Must-Have**
- [ ] FR-01: RPN input model with 4-level stack (X/Y/Z/T) and LASTX register
- [ ] FR-02: Correct stack-lift semantics (enable/disable per HP-41 rules)
- [ ] FR-03: Alphanumeric 12-char display with annunciators (USER, PRGM, ALPHA, SHIFT, RAD/DEG/GRAD)
- [ ] FR-04: Core arithmetic (`+ − × ÷`, `1/x`, `√x`, `x²`, `Y^X`, `LN`, `LOG`, `e^x`, `10^x`)
- [ ] FR-05: Trig (SIN/COS/TAN + inverses) with DEG/RAD/GRAD modes
- [ ] FR-06: Number formatting modes: FIX n, SCI n, ENG n
- [ ] FR-07: Numeric storage registers (R00–R99) with STO, RCL, STO+/−/×/÷
- [ ] FR-08: ALPHA mode for entering and storing alphanumeric strings
- [ ] FR-09: Keystroke programming — LBL, GTO, XEQ, RTN, conditional tests, ISG, DSE
- [ ] FR-10: Save/load programs and full calculator state to disk (JSON serialization)
- [ ] FR-11: Physical-keyboard input mapping (terminal REPL/TUI)
- [ ] FR-12: Interactive TUI showing stack, display, and annunciator state (ratatui)
- [ ] FR-13: Built-in function reference / help command

**v1.0 CLI — Should-Have (in scope)**
- [ ] FR-14: USER mode with custom key assignments
- [ ] FR-15: Statistics functions (Σ+, Σ−, MEAN, SDEV, linear regression)
- [ ] FR-16: Time/date functions (HMS conversions)
- [ ] FR-19: Bundled sample program library (≥10 well-documented programs)

**Non-Functional**
- [ ] NFR-1: Cold-start ≤ 0.5 s on Apple M1 / Intel i5 8th gen
- [ ] NFR-2: Key-press → display update median latency ≤ 50 ms
- [ ] NFR-3: Crash-free sessions ≥ 99.5%; all panics eliminated from `hp41-core`
- [ ] NFR-4: ≥80% unit-test coverage in `hp41-core`; zero UI dependencies in core crate
- [ ] NFR-5: Runs on Windows 10+, macOS 12+, Ubuntu 22.04+ from single codebase
- [ ] NFR-6: Auto-save state every 30 s and on graceful shutdown
- [ ] NFR-7: ≥98% numerical agreement with HP-41 reference across 500-case test suite

### Out of Scope

- v2.0 GUI (Tauri, FR-G1–G5) — deferred until v1.0 CLI ships
- FR-17 Print emulation (PRX/PRA/PRSTK) — deferred to v1.1
- FR-18 Multiple skin themes — GUI-only, v2.0
- FR-20 Synthetic programming — could-have, deferred to v1.1+
- FR-21 Module emulation (Math/Stat/Time/Advantage) — could-have, v1.1+
- FR-22 `.raw` HP-41 program file import/export — could-have, v1.1+
- FR-23 Mobile (iOS/Android) — defer until desktop stable
- Cycle-accurate Nut CPU simulation — high effort, low user value vs. behavioral emulation
- Redistribution of HP-copyrighted ROM images — legal risk, explicitly excluded forever
- HP-IL peripheral emulation — niche, complex
- Wand/barcode reader — requires hardware, very niche
- Cloud sync — privacy and infrastructure cost

## Context

The HP-41 series (1979–1990) remains beloved among engineers for its 4-level RPN stack, alphanumeric display, and keystroke programmability. Original units sell for USD 200–600+ on the second-hand market with common hardware failures. Existing emulators are platform-locked, closed-source/abandoned, low UI quality, or have fidelity gaps (USER mode, synthetic programming, stack-lift).

Free42 (HP-42S emulator) demonstrates strong demand for HP RPN emulators: hundreds of thousands of downloads. HP-41 ROM dumps and SDK41/V41 documentation make high-fidelity behavioral recreation technically feasible without ROM redistribution.

This is a solo developer project (Daniel Senften) targeting a v1.0 CLI release on 2026-09-05.

## Constraints

- **Tech stack**: Rust stable 1.78+ — deterministic, GC-free execution ideal for emulation core
- **Task runner**: `just` — all build/test/lint/run targets defined as recipes in a top-level `Justfile`; no bare `cargo` commands in CI or docs
- **Architecture**: `hp41-core` must never depend on `hp41-cli` or `hp41-gui` — enforced by Cargo workspace
- **Dependencies**: ratatui 0.28+, crossterm, clap 4.x, serde/serde_json for v1.0; Tauri v2 + Node.js for v2.0 only
- **Timeline**: v1.0 CLI release 2026-09-05 (~17 weeks from 2026-05-12)
- **Legal**: No HP-copyrighted ROM bytes anywhere in the codebase; license audit before public release
- **Privacy**: No telemetry by default; local-only data storage; no network calls in MVP
- **Compatibility**: Windows 10+, macOS 12+, Ubuntu 22.04+; file formats versioned with forward-compatible JSON schema

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Behavioral emulation, not cycle-accurate Nut CPU | High effort, low user value vs. behavioral fidelity | — Pending |
| Cargo workspace with `hp41-core` / `hp41-cli` / `hp41-gui` crates | Enforces clean separation; GUI is a thin adapter over unchanged core | — Pending |
| ratatui + crossterm for v1.0 TUI | Cross-platform, keyboard-driven, no GUI dep | — Pending |
| Tauri v2 for v2.0 GUI | Rust backend + web frontend; pixel-perfect skin in HTML/CSS; deferred until v1.0 ships | — Pending |
| GSD roadmap mirrors PRD milestones M0–M5 | Clean traceability; PRD already validated by author | — Pending |
| Should-haves FR-14/15/16/19 included in v1.0 M4 hardening phase | Author confirmed all four in scope; FR-17 print emulation deferred | — Pending |
| `just` as sole task runner | All build/test/lint/run/ci targets defined as `just` recipes; contributors and CI never call `cargo` directly | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-06 after initialization*
