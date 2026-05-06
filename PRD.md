# Product Requirements Document: HP-41 Calculator Emulator (Rust)

| Field             | Value                                        |
| ----------------- | -------------------------------------------- |
| **Document Type** | Standard Feature PRD                         |
| **Project Name**  | HP-41 Calculator Emulator                    |
| **Author**        | Daniel Senften                               |
| **Date**          | 2026-05-06                                   |
| **Status**        | Draft v2.0                                   |
| **Target Stack**  | Rust (cross-platform) — Cargo workspace      |

---

## 1. Executive Summary

We will build a faithful software re-creation of the **Hewlett-Packard HP-41C/CV/CX** programmable RPN calculator in Rust, targeting engineers, scientists, hobbyists, and retro-computing enthusiasts who rely on or appreciate the HP-41 workflow. The product solves the problem that original HP-41 hardware is scarce, expensive, and aging, while existing emulators are often platform-locked, visually outdated, or lack programmability fidelity.

The project is structured in two distinct phases:

- **v1.0 — CLI**: A fully functional, keyboard-driven terminal application (`hp41-cli`) backed by a UI-agnostic core library (`hp41-core`). Delivers all RPN computation, the four-level stack, alphanumeric display, ≥120 standard functions, keystroke programming, and save/load — entirely in the terminal.
- **v2.0 — GUI**: A graphical desktop application (`hp41-gui`) built on **Tauri v2**, reusing `hp41-core` unchanged. Delivers a pixel-perfect HP-41 skin, high-DPI rendering, and the authentic visual experience.

Success means recognized RPN-fidelity by HP-41 enthusiasts and a maintainable open-source Rust reference implementation of the HP-41 paradigm on modern hardware.

---

## 2. Problem Statement

### 2.1 Current State

The HP-41 series (1979–1990) remains a beloved RPN calculator among engineers and scientists for its 4-level stack, alphanumeric display, and elegant keystroke programmability. Original units are out of production. Working units on the second-hand market frequently sell for **USD 200–600+**, and capacitor/keyboard failures are common.

Existing emulators have material gaps:

- **Platform-locked**: most polished options run only on Windows or specific mobile OSes.
- **Closed-source / abandoned**: several well-known emulators have not been maintained since the early 2010s.
- **Low UI quality**: many current solutions feel dated and do not scale on high-DPI displays.
- **Imperfect fidelity**: some emulators skip USER mode, synthetic programming, or correct stack-lift behavior.

### 2.2 Problem Description

Users who want to **learn, use, or preserve** the HP-41 workflow on a modern computer cannot find a single product that is cross-platform, well-maintained, faithful to the original, and pleasant to use.

### 2.3 Impact

- **User**: lost access to a productivity tool the user has internalized over decades; learning curve for newcomers is barred by hardware unavailability.
- **Educational**: schools and training programs (e.g., surveying, navigation) cannot adopt the HP-41 paradigm without supplying physical units.
- **Cultural / preservation**: the keystroke-programming heritage risks fading from active use.

### 2.4 Evidence

- Active HP forum (`hpmuseum.org/forum`) threads requesting modern, cross-platform HP-41 emulation continue to appear in 2024–2026.
- Existing community emulator **Free42** (HP-42S, not HP-41) demonstrates strong demand: hundreds of thousands of downloads across stores.
- HP-41 ROM dumps and the SDK41 / V41 documentation make a high-fidelity recreation technically feasible.

### 2.5 Why Rust, Why Now

- **Rust** provides deterministic, GC-free execution ideal for an emulation core: no latency spikes, predictable arithmetic, safe concurrency for background program execution.
- **Cargo workspaces** enforce a clean crate boundary between core logic, CLI frontend, and GUI frontend — the architecture is embedded in the build system.
- **Tauri v2** (Rust backend + web frontend) allows a pixel-perfect HP-41 skin in HTML/CSS while keeping all emulation logic in safe Rust.
- Original HP-41 ROM behavior is well documented and legally analyzable for clean-room implementation of public algorithms (we will NOT redistribute HP-copyrighted ROM images).
- Aging original hardware increases urgency for digital preservation.

---

## 3. Objectives and Success Metrics

### 3.1 Product Objectives (SMART)

| ID  | Objective                                                                                       | Phase | Target Date  |
| --- | ----------------------------------------------------------------------------------------------- | ----- | ------------ |
| O1  | Ship CLI MVP supporting RPN, 4-level stack, alphanumeric display, and ≥120 standard functions   | v1.0  | 2026-09-05   |
| O2  | Achieve ≥98 % numerical agreement with HP-41 reference results across a 500-case test suite     | v1.0  | 2026-09-05   |
| O3  | Run on Windows, macOS, and Linux from a single codebase                                         | v1.0  | 2026-09-05   |
| O4  | Support keystroke programming (LBL/GTO/XEQ/RTN) with save/load of programs                     | v1.0  | 2026-09-05   |
| O5  | Ship GUI desktop app with HP-41 skin on all three platforms                                     | v2.0  | TBD          |

### 3.2 Business / Personal Objectives

- Establish a maintained, open-source Rust reference implementation for the HP-41 paradigm.
- Build a portfolio-quality Rust project demonstrating clean architecture, testability, and cross-platform delivery.
- Demonstrate CLI-first engineering discipline: the core must be fully usable without any graphical dependency.

### 3.3 Primary Metrics

| Metric                                  | Baseline | Target (v1.0 CLI) | Measurement                         |
| --------------------------------------- | -------- | ----------------- | ----------------------------------- |
| Numerical accuracy vs. HP-41 reference  | n/a      | ≥ 98 %            | Automated 500-case test suite       |
| Function coverage of HP-41C standard set| 0        | ≥ 120 of ~130     | Function inventory checklist        |
| Cross-platform parity (feature delta)   | n/a      | 0 features        | Manual smoke test on each OS        |
| Cold-start time (CLI)                   | n/a      | < 0.5 s           | `time hp41` on reference machine    |

### 3.4 Secondary Metrics

- GitHub stars after public release (informational).
- Community-submitted bug reports closed within 14 days (target ≥ 80 %).
- Number of user-shared programs uploaded to the bundled program library.

### 3.5 Guardrail Metrics

- Crash-free sessions ≥ 99.5 %.
- Median key-press → display update latency < 50 ms (both CLI and GUI).
- Memory footprint under typical use < 50 MB (CLI); < 150 MB (GUI with Tauri/WebView).

---

## 4. User Stories and Personas

### 4.1 Personas

**Persona 1 — "Erika, the practicing engineer" (primary)**
- Civil engineer, 52, Switzerland; learned RPN on an HP-41 in university.
- Owns a flaky HP-41CV; wants a modern, reliable replacement on her laptop.
- Goals: rapid arithmetic, custom programs for surveying tasks, no learning curve.
- Frustration: existing emulators lose her programs on update; UI scaling on 4K monitor is broken.

**Persona 2 — "Marc, the retro enthusiast"**
- Software developer, 34; never owned an HP-41 but loves vintage computing.
- Goals: explore HP-41 programming, share programs with the community.
- Frustration: cannot afford an original HP-41CX in working condition.

**Persona 3 — "Lena, the engineering student"**
- 22, mechanical engineering student.
- Goals: understand RPN as a learning exercise; complete coursework calculations.
- Frustration: classroom calculators differ; she wants a free desktop tool.

### 4.2 User Stories

| ID   | As a…           | I want to…                                    | So that…                                            | Phase |
| ---- | --------------- | --------------------------------------------- | --------------------------------------------------- | ----- |
| US1  | engineer        | enter calculations using RPN                  | I keep my decades-old workflow                      | v1.0  |
| US2  | engineer        | see a live 4-level stack (X/Y/Z/T)            | I can verify intermediate values                    | v1.0  |
| US3  | engineer        | write, save, and load keystroke programs       | I can automate my recurring computations            | v1.0  |
| US4  | user            | use my physical keyboard for input             | I do not have to mouse-click each key               | v1.0  |
| US5  | user            | toggle between FIX/SCI/ENG display modes       | numbers display in the format I need                | v1.0  |
| US6  | retro enthusiast| view the calculator with an HP-41-like skin    | I get the authentic visual experience               | v2.0  |
| US7  | student         | get a built-in quick-reference for functions   | I can learn without external manuals                | v1.0  |
| US8  | user            | run the same app on Windows, macOS, Linux      | I am not locked to one OS                           | v1.0  |

### 4.3 Acceptance Criteria (representative)

**US1 — RPN entry**
- Given the stack contains `[3, 4]`, when the user presses `+`, then X = 7 and the stack lifts correctly per HP-41 rules.
- Stack-lift enable/disable behavior matches HP-41 specification (e.g., after `ENTER↑`, stack lift is disabled).
- Negative number entry via `CHS` works at any point in number entry.

**US3 — Keystroke programming**
- A program `LBL "ADD" + RTN` can be created, saved to disk (JSON or `.raw`-like format), reloaded, and executed via `XEQ "ADD"`.
- `GTO`, `XEQ`, `RTN`, conditional tests (`X=0?`, `X<Y?`, etc.) behave identically to the HP-41 manual examples.
- Program memory size limit is enforced and surfaced to the user.

**US8 — Cross-platform**
- Application builds and runs on Windows 10+, macOS 12+, and Ubuntu 22.04+ from a single codebase with no platform-specific feature gaps in the MVP scope.

---

## 5. Functional Requirements

### 5.1 Must-Have — v1.0 CLI

| ID    | Requirement                                                                                       |
| ----- | ------------------------------------------------------------------------------------------------- |
| FR-01 | RPN input model with 4-level stack (X, Y, Z, T) and LASTX register                               |
| FR-02 | Correct stack-lift semantics (enable/disable per HP-41 rules)                                     |
| FR-03 | Alphanumeric 12-character display with annunciators (USER, PRGM, ALPHA, SHIFT, RAD/DEG/GRAD, etc.)|
| FR-04 | Core arithmetic (`+ − × ÷`, `1/x`, `√x`, `x²`, `Y^X`, `LN`, `LOG`, `e^x`, `10^x`)              |
| FR-05 | Trig (SIN/COS/TAN + inverses) with DEG/RAD/GRAD modes                                            |
| FR-06 | Number formatting modes: `FIX n`, `SCI n`, `ENG n`                                               |
| FR-07 | Numeric storage registers (R00–R99 minimum) with `STO`, `RCL`, `STO+`, `STO−`, `STO×`, `STO÷`   |
| FR-08 | ALPHA mode for entering and storing alphanumeric strings                                          |
| FR-09 | Keystroke programming: `LBL`, `GTO`, `XEQ`, `RTN`, conditional tests, `ISG`, `DSE`               |
| FR-10 | Save/load programs and full state to local disk (JSON-based serialization)                        |
| FR-11 | Physical-keyboard input mapping for all keys (terminal REPL / TUI)                               |
| FR-12 | Interactive TUI (terminal UI) showing stack, display, and annunciator state                       |
| FR-13 | Built-in function reference / help command                                                        |

### 5.2 Must-Have — v2.0 GUI

| ID    | Requirement                                                                                       |
| ----- | ------------------------------------------------------------------------------------------------- |
| FR-G1 | Desktop GUI via **Tauri v2** (Rust backend + HTML/CSS/JS frontend)                               |
| FR-G2 | On-screen calculator skin resembling the HP-41 layout (SVG/CSS assets)                           |
| FR-G3 | High-DPI / 4K-aware rendering (vector skin assets)                                               |
| FR-G4 | Mouse-click input on on-screen keys                                                               |
| FR-G5 | Full keyboard mapping retained from v1.0 core                                                     |

### 5.3 Should-Have

| ID    | Requirement                                                                              |
| ----- | ---------------------------------------------------------------------------------------- |
| FR-14 | USER mode with custom key assignments                                                    |
| FR-15 | Statistics functions (Σ+, Σ−, MEAN, SDEV, linear regression)                            |
| FR-16 | Time/date functions (HMS conversions)                                                    |
| FR-17 | Print emulation (virtual paper tape for `PRX`, `PRA`, `PRSTK`)                          |
| FR-18 | Multiple skin themes (HP-41C / CV / CX, light/dark) — v2.0 GUI only                    |
| FR-19 | Bundled sample program library (≥ 10 well-documented programs)                          |

### 5.4 Could-Have

| ID    | Requirement                                                                              |
| ----- | ---------------------------------------------------------------------------------------- |
| FR-20 | Synthetic programming support (advanced)                                                 |
| FR-21 | Module emulation: Math, Stat, Time, Advantage (legally re-implemented, no copyrighted ROM)|
| FR-22 | Import/export of `.raw` HP-41 program files                                              |
| FR-23 | Mobile build (iOS / Android) via Tauri mobile or separate port                          |

### 5.5 Won't-Have (this release)

- Bit-accurate emulation of the original Nut CPU at instruction-cycle level.
- Redistribution of HP-copyrighted ROM images (legal risk; explicitly excluded).
- HP-IL peripheral emulation.
- Wand / barcode reader emulation.
- Cloud sync of programs.

### 5.6 Edge Cases

- Division by zero must produce the HP-41 `DATA ERROR` message, not a Rust panic.
- Numeric overflow must produce `OUT OF RANGE` and clamp per HP-41 behavior (≈ 1e±499).
- Program memory full must surface a clear, HP-41-style error message.
- Unicode in user text labels must be either round-tripped or rejected with a clear message — never silently corrupted.

---

## 6. Architecture

### 6.1 Crate Workspace Structure

```
hp41-calculator/          ← Cargo workspace root
├── hp41-core/            ← Pure logic, zero UI deps
│   ├── src/stack.rs
│   ├── src/registers.rs
│   ├── src/functions/
│   ├── src/programming/
│   └── src/display.rs
├── hp41-cli/             ← v1.0 TUI frontend (ratatui / crossterm)
│   └── src/main.rs
└── hp41-gui/             ← v2.0 Tauri frontend (added in v2.0)
    ├── src-tauri/
    └── src/ (HTML/CSS/JS skin)
```

**Invariant**: `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. The core exposes a pure Rust API; frontends are thin adapters.

### 6.2 Key Crate Dependencies

| Crate          | Purpose                                    |
| -------------- | ------------------------------------------ |
| `ratatui`      | TUI layout and rendering (v1.0 CLI)        |
| `crossterm`    | Cross-platform terminal input/output       |
| `clap`         | CLI argument parsing                       |
| `serde` / `serde_json` | State serialization                |
| `tauri` (v2)   | Desktop GUI shell (v2.0 only)              |

---

## 7. Non-Functional Requirements

### 7.1 Performance

- Cold-start ≤ 0.5 s (CLI) on a baseline machine (Apple M1 / Intel i5 8th gen).
- Key-press → display update median latency ≤ 50 ms.
- Long-running programs must remain UI-responsive (Tokio async or thread-per-program).

### 7.2 Reliability

- Crash-free sessions ≥ 99.5 %.
- Auto-save calculator state every 30 s and on graceful shutdown so users do not lose programs.
- All panics eliminated from `hp41-core`; errors propagated via `Result<T, HpError>`.

### 7.3 Security & Privacy

- No telemetry by default. Any future telemetry is **opt-in** with a clear disclosure.
- Local-only data storage; no network calls in the MVP.
- File I/O confined to a user-chosen working directory; no access outside it.

### 7.4 Usability & Accessibility

- Keyboard-only operation supported in CLI (no mouse required).
- High-DPI / 4K-aware UI in v2.0 GUI (vector skin assets).
- Color contrast meets WCAG 2.1 AA for all text and annunciators (GUI).
- Localization-ready (string resource files); MVP ships English; German planned for v1.1.

### 7.5 Maintainability

- Domain logic (stack, registers, function library, program executor) lives in `hp41-core` with ≥ 80 % unit-test coverage.
- `hp41-core` has zero UI dependencies — verified by `cargo check` without any frontend features.
- Public algorithms documented inline; no closed-source binary dependencies.
- Conventional commits + GitHub Actions CI from day one.

### 7.6 Compatibility

- Rust stable toolchain (MSRV: 1.78+).
- Windows 10+, macOS 12+, Ubuntu 22.04+ (and equivalent).
- File formats versioned with forward-compatible JSON schema.

---

## 8. Out of Scope

| Item                                                | Rationale                                                | Future?       |
| --------------------------------------------------- | -------------------------------------------------------- | ------------- |
| Cycle-accurate Nut CPU simulation                   | High effort, low user value vs. behavioral emulation     | Not planned   |
| Redistribution of original HP ROM images            | Legal / copyright risk                                   | Not planned   |
| HP-IL peripherals (printer, mass storage, etc.)     | Niche; complex                                           | v2.0 candidate|
| Wand / barcode reader                               | Requires hardware; very niche                            | Not planned   |
| Mobile (iOS/Android)                                | Defer until desktop is stable                            | v2.x          |
| Cloud sync                                          | Privacy and infrastructure cost                          | Not planned   |
| Multi-user collaboration features                   | Not aligned with calculator UX                           | Not planned   |

---

## 9. Risk Assessment

| ID  | Risk                                                                | Impact | Likelihood | Mitigation                                                                                              | Owner    |
| --- | ------------------------------------------------------------------- | ------ | ---------- | ------------------------------------------------------------------------------------------------------- | -------- |
| R1  | Numerical fidelity drift vs. real HP-41 (especially trig at extremes)| High  | Medium     | Build a 500-case ground-truth test suite from the HP-41 manual and community references; CI-enforced.   | Lead Dev |
| R2  | Stack-lift edge cases differ from real hardware                     | High   | Medium     | Codify rules from HP-41 OS manual; property-based tests covering each function's stack-lift class.      | Lead Dev |
| R3  | Rust learning curve / borrow checker friction                       | Medium | Low–Medium | Developer has prior Rust exposure; time-box any complex async/lifetime issues with simpler alternatives. | Lead Dev |
| R4  | Legal exposure if HP ROM bytes are accidentally included            | High   | Low        | Clean-room policy; license audit before public release; do not import community ROM dumps.              | Lead Dev |
| R5  | Scope creep into modules / synthetic programming                    | Medium | High       | MoSCoW discipline; defer Could-Haves to v1.1.                                                           | Lead Dev |
| R6  | Performance regressions in program execution                        | Medium | Low        | Benchmark suite for representative programs; threshold checks in CI.                                    | Lead Dev |
| R7  | Tauri v2 / WebView cross-platform inconsistencies (v2.0 only)      | Medium | Medium     | Defer to v2.0; v1.0 CLI has no GUI dependency. Platform smoke test per OS at each GUI milestone.        | Lead Dev |
| R8  | Solo-developer bus factor                                           | High   | Medium     | Public Git repo from day one, README + architecture docs, conventional commits.                         | Lead Dev |

---

## 10. Timeline and Milestones

### Phase 1 — v1.0 CLI (~17 weeks, solo developer)

**Duration**: 2026-05-12 → 2026-09-05

| Phase | Weeks | Dates                   | Deliverable                                                                                    |
| ----- | ----- | ----------------------- | ---------------------------------------------------------------------------------------------- |
| M0 — Foundations         | 1–2  | 2026-05-12 → 2026-05-23 | Cargo workspace, CI (GitHub Actions), crate skeleton (`hp41-core`, `hp41-cli`)   |
| M1 — Calculator Core     | 3–6  | 2026-05-26 → 2026-06-20 | Stack, registers, FR-01–FR-08, FR-11; unit tests; `cargo test` green             |
| M2 — TUI Frontend        | 7–9  | 2026-06-23 → 2026-07-11 | FR-12, FR-13; functional TUI on all three OSes (ratatui)                         |
| M3 — Programming Engine  | 10–13| 2026-07-14 → 2026-08-08 | FR-09, FR-10; conditional/loop tests, save-load                                  |
| M4 — Hardening & Should-Haves | 14–16 | 2026-08-11 → 2026-08-29 | FR-14–FR-19, performance, full 500-case test suite                          |
| M5 — Release Prep        | 17   | 2026-09-01 → 2026-09-05 | Docs, `cargo install` / binary releases, GitHub release, sample programs          |

#### Key Milestones (v1.0)

- **2026-05-23**: Workspace compiles; `hp41-core` and `hp41-cli` crates exist with empty stubs.
- **2026-06-20**: Core calculator passes all arithmetic + trig tests; no TUI yet.
- **2026-07-11**: Public alpha — usable as a TUI RPN calculator (no programming yet).
- **2026-08-08**: Public beta — keystroke programming functional.
- **2026-09-05**: v1.0 CLI release.

### Phase 2 — v2.0 GUI (timeline TBD after v1.0 ships)

| Phase | Deliverable                                                                                  |
| ----- | -------------------------------------------------------------------------------------------- |
| G0    | Tauri v2 project skeleton; IPC bridge between `hp41-core` and JS frontend                   |
| G1    | HP-41 skin in HTML/CSS; on-screen keyboard functional                                        |
| G2    | Feature parity with CLI; all FR-G1–FR-G5 satisfied                                          |
| G3    | Should-Have GUI features (FR-18 themes, print emulation)                                     |
| G4    | Hardening, installers (MSI, .dmg, AppImage), v2.0 release                                   |

### 10.1 Dependencies

- Rust stable toolchain (1.78+) on each target OS.
- `ratatui` 0.28+, `crossterm`, `clap` 4.x, `serde_json`.
- (v2.0 only) Tauri v2 CLI + Node.js for frontend tooling.
- Access to HP-41C/CV/CX owner's manuals and the *HP-41 Synthetic Programming* reference.

### 10.2 Approvals

| Stakeholder      | Role                | Approval Required For        |
| ---------------- | ------------------- | ---------------------------- |
| Daniel Senften   | Owner / Lead Dev    | Scope, release, license      |

---

## Appendix A — Glossary

- **RPN** — Reverse Polish Notation; postfix-style calculator entry.
- **Stack lift** — HP convention controlling whether a new number pushes the X register up.
- **LASTX** — Register holding the X value before the most recent function.
- **USER mode** — HP-41 mode allowing user-defined key assignments.
- **Synthetic programming** — Use of byte sequences not normally producible from the keyboard, exposing extra functionality.
- **TUI** — Terminal User Interface; keyboard-driven text-mode UI (v1.0).
- **Tauri** — Rust-based framework for building desktop apps with a web frontend.

## Appendix B — References

- HP-41C / 41CV / 41CX Owner's Manuals (Hewlett-Packard).
- *HP-41 Synthetic Programming Made Easy* — Keith Jarett.
- HP Museum — `https://www.hpmuseum.org/`.
- Free42 (HP-42S emulator, used as engineering reference for emulator UX patterns).
- Tauri v2 documentation — `https://v2.tauri.app/`.
- ratatui documentation — `https://ratatui.rs/`.
