# HP-41 Calculator Emulator

[![CI](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci.yml/badge.svg)](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci.yml)
[![CI (GUI)](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci-gui.yml/badge.svg)](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci-gui.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A faithful, open-source behavioral emulation of the **HP-41C/CV/CX** programmable RPN calculator, written in Rust. Ships both a terminal UI (`hp41-cli`) and a pixel-perfect desktop app (`hp41-gui`, Tauri v2 + React).

```
┌─────────────────────────────────────┐
│  4.0000000000   HP-41CV             │
│─────────────────────────────────────│
│  2  ENTER↑                          │
│  2  ×                               │
│  →  4                               │
└─────────────────────────────────────┘
```

## Releases

| Version | Date | Highlights |
|---------|------|------------|
| [v2.0](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v2.0) | 2026-05-10 | Tauri desktop GUI: pixel-perfect SVG skin, IPC layer, shared autosave, PRGM-mode program listing, 3-OS GUI CI |
| [v1.1](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v1.1) | 2026-05-09 | CLI feature completeness: hardware-faithful EEX, STO arithmetic modals, PRX/PRA/PRSTK print emulation, synthetic programming (GETKEY, NULL, M/N/O, HexModal) |
| [v1.0](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v1.0) | 2026-05-08 | First public release: full RPN engine, keystroke programming, ratatui TUI, JSON persistence, cross-platform CI |

## Features

**Calculator engine (`hp41-core`)**

- Full RPN stack model (X, Y, Z, T + LAST X) with correct stack-lift behaviour for every one of ~130 operations
- 100 numbered storage registers (R00–R99) plus the hidden synthetic registers M, N, O
- ALPHA register (24 chars) and string operations
- ISG/DSE loop counters with string-split semantics (no floating-point rounding errors)
- Keystroke programming: LBL / GTO / XEQ / RTN, all 12 conditional tests, ISG/DSE loops
- Hardware-faithful EEX entry (trailing-e commits as exponent 00; empty-buffer EEX inserts implicit mantissa)
- Print emulation: PRX / PRA / PRSTK push to an in-memory `print_buffer` — `hp41-core` stays I/O-free
- Synthetic programming: GETKEY, NULL, hidden registers M/N/O, 2-digit HexModal over a curated 23-entry safe subset
- Persistent state via JSON at `~/.hp41/autosave.json` — human-readable, version-stable, shared between CLI and GUI

**Terminal UI (`hp41-cli`)**

- ratatui 0.30 + crossterm — runs on macOS, Linux, Windows
- Persistent 4-level stack display, 12-char HP-41 alphanumeric display, all 5 annunciators
- STO arithmetic keyboard modal (`S → +/−/×/÷ → R00–R99 | Y/Z/T/L`)
- `--print-log <path>` appends PRX/PRA/PRSTK output to a file
- `?` overlay shows the full key reference

**Desktop GUI (`hp41-gui`)**

- Tauri v2 + React + TypeScript — single static window, native packaging on macOS, Windows, Linux
- 44-key SVG skin matching HP-41C proportions and colour scheme; CSS scale-down press animation
- 12-char display, 5 annunciators, X/Y/Z/T/LASTX stack panel — all keyboard bindings from the CLI work in the GUI too
- Scrollable PRX/PRA/PRSTK print panel
- PRGM-mode program listing with SST / BST navigation and auto-scroll
- Shared autosave with the CLI: state saved in one binary appears in the other on next launch

## Variants Emulated

This is a **behavioural** emulation — variant-specific memory limits are not enforced; the emulator always provides 100 numbered registers (R00–R99) plus the three hidden synthetic registers, regardless of which physical model the table references.

| Model   | Year | Original memory   | Notes                              |
|---------|------|-------------------|------------------------------------|
| HP-41C  | 1979 | 63 registers      | Base model                         |
| HP-41CV | 1980 | 319 registers     | "Continuously Variable" memory     |
| HP-41CX | 1983 | Extended + Time   | Built-in X-Functions & Time Module |

## Quick Start

```bash
# Prerequisites: Rust stable (MSRV 1.88), just
cargo install just
```

**Terminal UI (`hp41-cli`):**

```bash
just run                # build + launch the TUI
just test               # run all tests
just ci                 # full CLI gate: lint → test → coverage (≥80% on hp41-core)
just run -- --print-log /tmp/hp41.log   # append PRX/PRA/PRSTK output to a file
```

**Desktop GUI (`hp41-gui`):**

```bash
# Additional prerequisites: Node.js + npm; see hp41-gui/README for OS-specific
# WebKit / webkit2gtk requirements on Linux
just gui-dev            # launch the Tauri dev window
just gui-build          # release build (produces a native bundle)
just gui-ci             # GUI gate: cargo test + cargo build --release
just gui-check          # cargo check + tsc --noEmit
```

The GUI and CLI share state via `~/.hp41/autosave.json` — they auto-save every 30 s and load each other's state on launch.

## Documentation

| Document | Description |
|----------|-------------|
| [HP-41 Overview](docs/hp41-overview.md) | History, variants, RPN introduction |
| [Operations Reference](docs/operations-reference.md) | All ~130 operations by category |
| [Keyboard Layout](docs/keyboard-layout.md) | Key layout and shifted functions |
| [Programming Guide](docs/programming-guide.md) | Stack model, programs, flags, loops |
| [Architecture](docs/architecture.md) | Emulator internals for contributors |

### Official HP Manuals

- [HP-41C/CV Owner's Manual](https://www.hpmuseum.org/41ownman.htm) — Museum of HP Calculators
- [HP-41C/CV/CX Advanced Functions Handbook](https://www.hpmuseum.org/41advfun.htm)
- [HP-41CX Owner's Manual](https://www.hpmuseum.org/41cxman.htm)
- [HP-41 Programming](https://www.hpmuseum.org/prog/hp41prog.htm) — hpmuseum.org

## Project Structure

```
hp41-core/                — UI-agnostic library (calculator engine, zero CLI/UI dependencies)
hp41-cli/                 — Terminal UI binary (ratatui + crossterm)
hp41-gui/                 — Tauri v2 desktop app (nested standalone workspace)
  ├── src-tauri/          — Rust backend (IPC commands, persistence, prgm display)
  └── src/                — React + TypeScript frontend (App.tsx, Keyboard.tsx)
```

The root Cargo workspace declares `members = ["hp41-core", "hp41-cli"]`; `hp41-gui` is a **nested standalone workspace** so the `tauri` / `tauri-build` dependencies never enter the root resolver. `cargo build --workspace` from the repo root does not touch the Tauri binary.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All contributions target the `develop` branch via Pull Request.  
Direct pushes to `develop` and `main` are restricted to the maintainer.

## License

MIT — see [LICENSE](LICENSE).
