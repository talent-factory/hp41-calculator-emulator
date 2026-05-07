# Requirements: HP-41 Calculator Emulator

**Defined:** 2026-05-06
**Core Value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware

## v1 Requirements

### Stack Model

- [x] **CORE-01**: User has a 4-level RPN stack (X/Y/Z/T) and LASTX register that behaves identically to HP-41 hardware
- [x] **CORE-02**: All ~130 operations implement correct stack-lift semantics (Enable / Disable / Neutral) per HP-41 specification

### Math & Formatting

- [ ] **MATH-01**: User can perform core arithmetic (`+ − × ÷`, `1/x`, `√x`, `x²`, `Y^X`, `LN`, `LOG`, `e^x`, `10^x`) with HP-41-accurate 10-digit results
- [ ] **MATH-02**: User can perform trig (SIN/COS/TAN + inverses) in DEG, RAD, and GRAD modes
- [ ] **MATH-03**: User can switch number formatting between FIX n, SCI n, and ENG n (n = 0–9)

### Registers

- [ ] **REGS-01**: User has storage registers R00–R99 with STO, RCL, and STO+/−/×/÷ operations that match HP-41 hardware behavior

### ALPHA Mode

- [ ] **ALPH-01**: User can enter and store alphanumeric strings in the 24-char ALPHA register via ALPHA mode keyboard input

### Keystroke Programming

- [x] **PROG-01**: User can record keystroke programs using LBL, GTO, XEQ, RTN, conditional tests (x=0?, x<0?, x>y?, etc.), ISG, and DSE
- [x] **PROG-02**: ISG/DSE counter format (`CCCCC.FFFDD`) behaves identically to HP-41 hardware (no float arithmetic on counter fields)

### Display & TUI

- [x] **DISP-01**: User sees the HP-41 alphanumeric 12-character display with annunciators (USER, PRGM, ALPHA, SHIFT, RAD/DEG/GRAD) in the TUI
- [x] **DISP-02**: User sees a persistent TUI panel showing the 4-level stack (X/Y/Z/T), LASTX, annunciator bar, and current display at all times

### Keyboard Input

- [x] **INPUT-01**: User can operate all calculator functions via physical keyboard in the TUI without needing an external reference for basic operations

### State Persistence

- [ ] **PERS-01**: User can save and load programs and full calculator state to/from versioned JSON files on disk
- [ ] **PERS-02**: Calculator state auto-saves every 30 s and on graceful shutdown (equivalent to HP-41 continuous memory)

### User Experience

- [ ] **UX-01**: User can access a built-in function reference/help from within the TUI (`?` or `HELP` command)
- [ ] **UX-02**: User can enable USER mode with custom key assignments that are persisted in state
- [ ] **UX-03**: User can run ≥10 bundled, documented sample programs that demonstrate core programming features

### Science & Engineering Functions

- [ ] **SCI-01**: User can perform statistics operations: Σ+, Σ−, MEAN, SDEV, and linear regression using Σ registers (R01–R06)
- [ ] **SCI-02**: User can perform HMS/H conversions: →HMS, HMS→, HMS+, HMS−

### Quality & Performance

- [x] **QUAL-01**: Cold-start latency ≤ 0.5 s on Apple M1 / Intel i5 8th gen
- [x] **QUAL-02**: Key-press → display update median latency ≤ 50 ms
- [x] **QUAL-03**: Crash-free sessions ≥ 99.5%; zero panics in `hp41-core`
- [x] **QUAL-04**: `hp41-core` has ≥80% unit-test coverage and zero UI/CLI dependencies
- [x] **QUAL-05**: Single codebase runs on Windows 10+, macOS 12+, Ubuntu 22.04+
- [x] **QUAL-06**: ≥98% numerical agreement with HP-41 hardware across 500-case test suite

## v2 Requirements

### GUI Desktop App

- **GUI-01**: User can run the calculator as a pixel-perfect Tauri v2 desktop application reusing `hp41-core` unchanged
- **GUI-02**: User can select from multiple visual skin themes
- **GUI-03**: User can interact via mouse/touch on the graphical key layout

### Extended Compatibility

- **EXT-01**: User can import/export HP-41 programs in `.raw` file format (V41 compatible)
- **EXT-02**: User can load HP-41 module ROMs (Math, Stat, Time, Advantage) via pluggable module interface
- **EXT-03**: User can use synthetic programming (byte-code injection, FOCAL internals access)
- **EXT-04**: Print emulation (PRX/PRA/PRSTK) to console or text file

### Mobile

- **MOB-01**: User can run the calculator on iOS or Android

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cycle-accurate Nut CPU emulation | Enormous effort; zero user-visible benefit vs. behavioral emulation |
| HP-copyrighted ROM image redistribution | Legal risk; clean-room behavioral reimplementation is feasible and sufficient |
| HP-IL peripheral emulation (printer, disk drive) | Complex bus protocol; niche use case; v2.0+ at earliest |
| Wand/barcode reader emulation | Requires physical hardware; historically interesting only |
| Cloud sync / network calls | Privacy risk; local-first culture of HP-41 community; users can sync state file via own tools |
| Telemetry / crash reporting | Privacy expectation in retro computing community is strongly local-only |
| Animations / cosmetic transitions | HP-41 community values speed; forums explicitly call out animations as unwanted |
| iOS/Android mobile (v1.0) | Different UI paradigm; v2.0 desktop first |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Complete |
| CORE-02 | Phase 1 | Complete |
| MATH-01 | Phase 2 | Pending |
| MATH-02 | Phase 2 | Pending |
| MATH-03 | Phase 2 | Pending |
| REGS-01 | Phase 2 | Pending |
| ALPH-01 | Phase 2 | Pending |
| PROG-01 | Phase 3 | Complete |
| PROG-02 | Phase 3 | Complete |
| DISP-01 | Phase 4 | Complete |
| DISP-02 | Phase 4 | Complete |
| INPUT-01 | Phase 4 | Complete |
| PERS-01 | Phase 5 | Pending |
| PERS-02 | Phase 5 | Pending |
| UX-01 | Phase 5 | Pending |
| UX-02 | Phase 5 | Pending |
| UX-03 | Phase 5 | Pending |
| SCI-01 | Phase 6 | Validated in Phase 6 (2026-05-07) |
| SCI-02 | Phase 6 | Validated in Phase 6 (2026-05-07) |
| QUAL-01 | Phase 7 | Complete (2026-05-07) |
| QUAL-02 | Phase 7 | Complete (2026-05-07) |
| QUAL-03 | Phase 7 | Complete (2026-05-07) |
| QUAL-04 | Phase 7 | Complete (2026-05-07) |
| QUAL-05 | Phase 7 | Complete (2026-05-07) |
| QUAL-06 | Phase 7 | Complete (2026-05-07) |

**Coverage:**
- v1 requirements: 25 total
- Mapped to phases: 25
- Unmapped: 0

---
*Requirements defined: 2026-05-06*
*Last updated: 2026-05-07 after Phase 7 completion — all QUAL-* requirements satisfied*
