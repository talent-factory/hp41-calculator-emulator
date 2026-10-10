# HP-41 Calculator Emulator

[![CI](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci.yml/badge.svg)](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci.yml)
[![CI (GUI)](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci-gui.yml/badge.svg)](https://github.com/talent-factory/hp41-calculator-emulator/actions/workflows/ci-gui.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

<p align="center">
  <img src="docs/screenshots/hp41-gui-v3.x.png" alt="HP-41C/CV/CX GUI on macOS" width="320">
  <br>
  <em>HP-41C/CV/CX desktop GUI on macOS — v4.0 with all four ROM modules (Math, Stat 1, Time, Advantage), Extended Memory, theming, and onboarding</em>
</p>

A faithful, open-source behavioral emulation of the **HP-41C/CV/CX** programmable RPN calculator, written in Rust. Ships both a terminal UI (`hp41-cli`) and a pixel-perfect desktop app (`hp41-gui`, Tauri v2 + React).

Implements the full **feature-complete HP-41CV ROM built-in function set** (~130 ops) with documented divergences. See the [HP-41CV function matrix](docs/hp41cv-function-matrix.md) for status per op, keyboard reachability, and known hardware divergences.

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
| [v4.0](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v4.0) | 2026-05-28 | **Platform Maturity**: HP-41CX Extended Memory (8 X-Function ops — EMDIR/EMROOM/SAVEP/GETP/SAVED/GETD/EMREG/SAVERX; 600-register capacity; OS-builtin routing via `builtin_card_op`, no XROM bit); 4 GUI skin themes (dark/light/classic-beige/high-contrast via CSS custom properties, isolated `~/.hp41/prefs.json`); first-run onboarding wizard + searchable in-app function reference + GUI↔CLI keyboard parity (`keyboard-shortcuts.json`); `.raw` program file import/export (native dialog + CLI flags, multi-program archives); zero new runtime deps; v1.0–v3.3 save files load without migration |
| [v3.3](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v3.3) | 2026-05-26 | **Advantage Pac behavioral emulation** (HP 00041-90482): 114 XEQ entry points across XROM 22 (ADV CONV + ADV MTRX, 63 ops) + XROM 24 (ADV MATH + ADV TVM, 51 ops); base conversion & 36-bit bitwise logic; named-matrix model with MATDIM/MDET/MINV/MSYS/M*M; Laguerre polynomial root-finder (FROOT); Romberg integration (FINTG); secant root-finder (FSOLVE) with cross-nesting; RK4 ODE solver (FDIFEQ); 4-model curve fitting; 3D vector arithmetic (DOT/CROSS/UV); Newton-Raphson TVM solver; 22 scipy-derived accuracy oracles; zero new runtime deps; v1.0–v3.2 save files load without migration |
| [v3.2](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v3.2) | 2026-05-25 | **Time Pac behavioral emulation** (HP 82182A, OM 00041-90035): 35 XEQ entry points across clock, date arithmetic, stopwatch, alarm catalog; real-time clock/stopwatch backed by host system clock with `time_offset_secs` delta model; 253-entry alarm catalog with message + control alarms and repeat intervals; pure-Rust Gregorian calendar arithmetic (Fliegel-Van Flandern JDN, zero new runtime deps); live clock/stopwatch display in CLI (62 Hz) and GUI (setInterval); `?` overlay gains "Time Pac (XROM 26)" section; v1.0–v3.1 save files load without migration |
| [v3.1](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v3.1) | 2026-05-24 | **Stat 1 Pac behavioral emulation** (HP 00041-90030): 13 programs, 26 XEQ entry points covering univariate statistics, one/two-way ANOVA, ANOCOV, linear/exponential/logistic/power/polynomial/multiple regression, hypothesis tests (pooled t-test), nonparametric tests (chi-square, Spearman), normal/chi-square distribution CDF/PDF/inverse, RAND/SEED LCG extension; 3 hand-coded distribution primitives (zero new runtime deps); modal prompts for DEGREE/SEED/ν; `?` overlay gains "Stat 1 Pac (XROM 2)" section; v1.0–v3.0 save files load without migration |
| [v3.0](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v3.0) | 2026-05-20 | **Math Pac I behavioral emulation** (HP 00041-90034, 1979): 10 top-level programs, ~55 XEQ-by-Name entry points across hyperbolics, complex stack, polynomial roots (Bairstow), matrix DET/INV/SIMEQ, INTG (Simpson), SOLVE (secant), DIFEQ (RK4), triangle solvers, Fourier transform, 2D/3D coordinate transforms; modal-workflow state machine + user-callback re-entrancy; CLI ↔ GUI parity via shared `xrom_resolve`; coverage 95.39 % lines / 94.26 % regions; `?` overlay gains incremental substring search; v1.0–v2.2 save files load without migration |
| [v2.2](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v2.2) | 2026-05-16 | **Feature-complete HP-41CV ROM built-ins**: ~90 new ops across math/flags/program-control/ALPHA/indirect (Phases 20–24); f-prefix one-shot CLI + GUI parity; 14-segment SVG LCD; JSON-canonical function pipeline; `?` help overlay; USER-mode key relabel; test hardening to 95.25 % coverage + 99.1 % numerical accuracy; WebdriverIO E2E smoke on Linux CI |
| [v2.1](https://github.com/talent-factory/hp41-calculator-emulator/releases/tag/v2.1) | 2026-05-13 | Authentic HP-41C 5×8 keyboard layout, one-shot SHIFT, three-label keys (primary + orange shifted + blue ALPHA), R/S command wiring, stub-error toast pattern |
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
- v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034
  ([documented divergences](docs/hp41-math1-divergences.md)) — see [Math Pac I Function Matrix](docs/hp41-math1-function-matrix.md)
- v3.1 ships Stat 1 Pac behavioral emulation, feature-complete per Owner's Manual HP 00041-90030 (13 programs, 26 XEQ entry points,
  RAND/SEED extension, [documented divergences](docs/hp41-stat1-divergences.md)) — see [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md)
- v3.2 ships Time Pac behavioral emulation, feature-complete per Owner's Manual 00041-90035 (35 XEQ entry points, real-time clock/stopwatch/alarm backed by the host system clock,
  [documented divergences](docs/hp41-time-divergences.md)) — see [Time Pac Function Matrix](docs/hp41-time-function-matrix.md)
- v3.3 ships Advantage Pac behavioral emulation, feature-complete per Owner's Manual 00041-90482 (~117 XEQ entry points across base conversion,
  named-matrix operations, advanced math/solvers/complex/curve-fit, and time-value-of-money;
  dual XROM IDs 22 + 24; [documented divergences](docs/hp41-advantage-divergences.md)) —
  see [Advantage Pac Function Matrix](docs/hp41-advantage-function-matrix.md)
- v4.0 (Platform Maturity) adds HP-41CX Extended Memory plus desktop-platform polish:
  - **Extended Memory** — named PROGRAM + DATA file storage (HP-41CX X-Functions): 8 XEQ-by-name functions (EMDIR, EMROOM, SAVEP, GETP, SAVED, GETD, EMREG, SAVERX); 600-register capacity (fully-expanded HP-41CX); OS-builtin routing via `builtin_card_op` (no XROM bit); [documented divergences](docs/hp41-xmem-divergences.md)
  - **Theming** — 4 built-in GUI skins (dark, light, classic beige, high-contrast) via CSS custom properties, persisted in `~/.hp41/prefs.json` (isolated from calculator state)
  - **Onboarding + keyboard parity** — first-run quick-start wizard, searchable in-app function reference, and GUI physical-keyboard shortcuts at parity with the CLI (`keyboard-shortcuts.json`)
  - **`.raw` file I/O** — import/export HP-41 program files via native dialog (GUI) and CLI flags (`--import-raw`/`--export-raw`/…), including multi-program archives

**Terminal UI (`hp41-cli`)**

- ratatui 0.30 + crossterm — runs on macOS, Linux, Windows
- Persistent 4-level stack display, 12-char HP-41 alphanumeric display, all 5 annunciators
- STO arithmetic keyboard modal (`S → +/−/×/÷ → R00–R99 | Y/Z/T/L`)
- `--print-log <path>` appends PRX/PRA/PRSTK output to a file
- `?` overlay shows the full key reference

**Desktop GUI (`hp41-gui`)**

- Tauri v2 + React + TypeScript — single static window, native packaging on macOS, Windows, Linux
- Authentic HP-41C layout: 4 top-row mode keys + 5×8 main grid + orange SHIFT cap (39 keys total); three-label model (primary white + orange shifted + blue ALPHA letter)
- 14-segment SVG LCD with 49-glyph character map (digits, A–Z, punctuation), dim-off "ghost" segments for authentic LCD aesthetic
- 12-char display, 6 annunciators (incl. SHIFT one-shot), X/Y/Z/T/LASTX stack panel — all keyboard bindings from the CLI work in the GUI too
- `?` help overlay driven by the canonical `docs/hp41cv-functions.json` source — every op searchable in-app
- USER-mode per-key relabel: ASN'd custom labels render on the affected key when USER annunciator is active
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

## Installation

### Pre-built binaries (recommended for end users)

Download platform-native binaries from the [latest release page](https://github.com/talent-factory/hp41-calculator-emulator/releases/latest):

| Platform | CLI (`hp41-cli`) | GUI (`hp41-gui`) |
|----------|------------------|------------------|
| **macOS** (Apple Silicon + Intel) | `hp41-cli-vX.Y-aarch64-apple-darwin.tar.gz` | `hp41-gui_X.Y.Z_universal.dmg` |
| **Windows 10/11** | `hp41-cli-vX.Y-x86_64-pc-windows-msvc.zip` | `hp41-gui_X.Y.Z_x64-setup.exe` (installer) or `_x64-portable.exe` |
| **Linux** (x86_64) | `hp41-cli-vX.Y-x86_64-unknown-linux-gnu.tar.gz` | `hp41-gui_X.Y.Z_amd64.deb` or `.AppImage` |

macOS binaries are signed with our Apple Developer certificate and Apple-notarized. Windows binaries are unsigned — on first launch you may see a SmartScreen warning ("Windows protected your PC" → click "More info" → "Run anyway").

Binaries first ship with **v3.0.1+** (the v3.0 release is source-only); for v3.0 use the build-from-source path below.

### Build from source (development + v3.0)

```bash
# Prerequisites: Rust stable (MSRV 1.88), just
cargo install just
```

**Terminal UI (`hp41-cli`):**

```bash
just run                # build + launch the TUI
just test               # run all tests
just ci                 # full CLI gate: lint → test → coverage (≥95 % on hp41-core)
just run -- --print-log /tmp/hp41.log   # append PRX/PRA/PRSTK output to a file
```

**Desktop GUI (`hp41-gui`):**

The desktop app is built with Tauri v2 (Rust + React/Vite) and needs **Node.js 18+ and npm** in addition to the Rust toolchain, plus OS-specific system libraries (notably WebKit / `webkit2gtk` on Linux). See **[`hp41-gui/README.md`](hp41-gui/README.md)** for the full per-OS prerequisites. Run `just gui-install` before `just gui-dev` on a fresh checkout — the Tauri CLI is installed as a GUI dependency, so the dev window cannot launch until deps are present.

```bash
just gui-install        # install GUI deps (npm); also pulls in the Tauri CLI
just gui-dev            # launch the Tauri dev window
just gui-build          # release build (produces a native bundle)
just gui-ci             # GUI gate: permission check + npm ci + audit + tsc --noEmit
just gui-check          # fast Rust type-check (cargo check on src-tauri)
```

The GUI and CLI share state via `~/.hp41/autosave.json` — they auto-save every 30 s and load each other's state on launch.

## Documentation

| Document | Description |
|----------|-------------|
| [HP-41 Overview](docs/hp41-overview.md) | History, variants, RPN introduction |
| [Operations Reference](docs/operations-reference.md) | All ~130 operations by category |
| [Function Matrix](docs/hp41cv-function-matrix.md) | Per-op status, keyboard path, divergences |
| [Math Pac I Function Matrix](docs/hp41-math1-function-matrix.md) | Math Pac I XROM entries with module/function IDs |
| [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md) | Stat 1 Pac XROM entries (module 2) with function IDs and divergences |
| [Time Pac Function Matrix](docs/hp41-time-function-matrix.md) | Time Pac XROM entries (module 26) with function IDs and divergences |
| [Advantage Pac Function Matrix](docs/hp41-advantage-function-matrix.md) | Advantage Pac XROM entries (modules 22+24) with function IDs and divergences |
| [Keyboard Layout](docs/keyboard-layout.md) | Key layout and shifted functions |
| [Programming Guide](docs/programming-guide.md) | Stack model, programs, flags, loops |
| [Verifying Math Pac I](docs/verifying-math-pac-1.md) | Operator walk-through for Math Pac I (all 11 groups) |
| [Verifying Stat 1 Pac](docs/verifying-stat-pac-1.md) | Operator walk-through for Stat 1 Pac (all 7 groups) |
| [Verifying Advantage Pac](docs/verifying-advantage-pac.md) | Operator walk-through for Advantage Pac (all 7 groups) |
| [Architecture](docs/architecture.md) | Emulator internals for contributors |
| [Release Setup](docs/release-setup.md) | Maintainer guide: binary-release workflows + Apple Developer secrets |

## Documented Divergences from HP-41 Hardware

A small set of deliberate behavioral divergences from the real HP-41C/CV/CX; each is recorded as a per-row `divergences` entry in the [function matrix](docs/hp41cv-function-matrix.md):

- **PI** — 10-digit rounded value (`3.141592654`); hardware uses the same internal 10-digit precision.
- **FACT** — implemented (v4.3): FACT(27..=69) returns the correct factorial in scientific notation (10 significant digits) via `HpNum::from_f64` large-exponent form; HP-41 caps at X ≤ 69. Prior to v4.3 X in 27..=69 returned `Overflow` (now fixed, FGAP-03 / Phase 65).
- **CLP** — boundary is the next `LBL` marker; HP-41 uses `END` / `.END.` markers (not present in our flat-Vec program model).
- **PACK** — no-op; HP-41 compacts program memory (we have no gaps to compact in the flat-Vec model).
- **POSA** — single-char only; multi-char POSA is deferred to v3.x (requires typed-stack shadow channel).
- **AROT** — silently truncates non-integer N toward zero; HP-41 rejects non-integer N.
- **HP-41 upper-ASCII (codes 128–255)** in ATOX / XTOA — round-trip not preserved (HP-41 ROM glyphs are not in the UTF-8 model).
- **ALPHA mode overrides f-prefix** — by design (D-25.5 / D-26 in CLAUDE.md); pressing `f` in ALPHA mode types the literal `F` instead of arming the prefix. Hardware-faithful ALPHA-with-prefix (Σ, π, μ, …) is deferred to v3.x alongside the ALPHA-special-charset table.

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
