# HP-41 Calculator Emulator

[![CI](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci.yml/badge.svg)](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A faithful, open-source behavioral emulation of the **HP-41C/CV/CX** programmable RPN calculator, written in Rust.

```
┌─────────────────────────────────────┐
│  4.0000000000   HP-41CV             │
│─────────────────────────────────────│
│  2  ENTER↑                          │
│  2  ×                               │
│  →  4                               │
└─────────────────────────────────────┘
```

## Features

- Full RPN stack model (X, Y, Z, T + LAST X)
- ~130 built-in operations with correct stack-lift behaviour
- 64 storage registers (expandable, matching HP-41CV/CX)
- Alpha register and string operations
- ISG/DSE loop counters with string-split semantics (no floating-point rounding errors)
- Flags 0–55 with all test/set/clear variants
- Persistent state via JSON (human-readable, version-stable)
- Terminal UI (TUI) with ratatui — runs on macOS, Linux, Windows

## Variants Emulated

| Model   | Year | Memory           | Notes                              |
|---------|------|------------------|------------------------------------|
| HP-41C  | 1979 | 63 registers     | Base model                         |
| HP-41CV | 1980 | 319 steps / 64 R | "Continuously Variable" memory     |
| HP-41CX | 1983 | Extended + Time  | Built-in X-Functions & Time Module |

## Quick Start

```bash
# Prerequisites: Rust stable 1.78+, just
cargo install just

# Build and run the TUI
just run

# Run all tests
just test

# Full CI gate (lint → test → coverage ≥80%)
just ci
```

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

## Project Structure

```
hp41-core/    — UI-agnostic library (calculator engine, zero CLI dependencies)
hp41-cli/     — Terminal UI binary (ratatui + crossterm)
hp41-gui/     — Tauri desktop app (planned, v2.0)
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All contributions target the `develop` branch via Pull Request.  
Direct pushes to `develop` and `main` are restricted to the maintainer.

## License

MIT — see [LICENSE](LICENSE).
