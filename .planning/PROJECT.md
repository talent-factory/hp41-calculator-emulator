# HP-41 Calculator Emulator

## Current Milestone: v3.0 Math 1 Pac Emulation

**Status:** planning (started 2026-05-16)

**Goal:** Behavioral Emulation des HP-41C **Math Pac I** (HP-Teilenummer 00041-90034, Owner's Manual 1979) als erstes XROM-Modul — 10 prompt-getriebene Workflow-Programme mit ~55 XEQ-by-Name Entry Points, nutzbar in CLI + GUI über eine neue Modal-Workflow-Schicht hinaus dem v2.x-Built-in-Pattern, ohne HP-copyrighted ROM-Image-Redistribution.

**Scope-Korrektur (2026-05-16):** Nach FEATURES-Research wurde klar, dass Math Pac I NICHT die ursprünglich angenommenen Funktionen (`M+`/`MAT*`/`INV`-as-transpose/`PROOT`/`CABS`/`CARG`/`V+`/`VDOT`) enthält — die sind im **Advanced Matrix Pac** und **Advantage Pac** (separate HP-Module). Math Pac I ist user-code in ROM (multi-step Modal-Workflows mit ALPHA-Prompts), nicht Nut-CPU-microcode (one-shot Stack-Ops). v3.0 emuliert den tatsächlichen Math Pac I; Advanced Matrix / Advantage werden separate Milestones.

**Target feature areas (Math Pac I per HP 00041-90034, 10 Top-Level Programme):**
- XROM-Modul-Framework: Slot-Management, XROM 7 (echte Math Pac ID), statisch verlinkter Resolver, vorbereitet für Stat 1 / Time / Advantage in v3.1+
- `MATRIX`: Determinante, Inverse, lineare Gleichungssysteme; Gauss-Elimination mit partieller Pivotsuche; bis 14×14; Matrix lebt ab R15
- `SOLVE`: reelle Nullstelle von f(x)=0 via modifizierter Sekanten-Iteration; ruft user-program LBL als Funktions-Callback
- `POLY`: Polynom-Wurzeln Grad 2–5 + Auswertung
- `INTG`: numerische Integration via Simpson; user-program LBL als Integrand-Callback
- `DIFEQ`: 1./2. Ordnung ODE-Solver via Runge-Kutta 4. Ordnung
- `FOUR`: Fourier-Reihen (rect + polar Koeffizienten)
- Komplex-Stack: zwei-Komplex-Zahlen-Stack (ζ/τ) überlagert auf X/Y/Z/T; arithmetik C+, C-, C×, C÷ + 13 weitere Funktionen
- Hyperbolicus: SINH, COSH, TANH, ASINH, ACOSH, ATANH (6 Ops im klassischen v2.2-Stil — einzige Familie mit one-shot UX)
- Dreiecks-Solver: SSS, ASA, SAA, SAS, SSA (5 Programme)
- `TRANS`: 2D + 3D Koordinaten-Transformationen (translate/rotate)
- Modal-Workflow-Schicht: ALPHA-Prompt-driven Mehrschritt-Flows (`ORDER=?`, `A1,1=?`, `FUNCTION NAME?`, `GUESS 1=?`) — neue Infrastruktur, parallel zur v2.2 `PendingInput`-Modal-Architektur
- User-Program-Callback: Re-entrancy in `run_loop` für INTG / SOLVE / DIFEQ; Stack-Tiefen-Cap honoriert
- CLI- + GUI-Integration: XEQ-by-Name fallback (keine dedizierten Math-Pac-Keys); JSON-canonical pipeline um `docs/hp41-math1-functions.json` erweitert
- Quality-Gates: `hp41-core` Coverage ≥ 95 %, neue Accuracy-Cases für MATRIX/POLY/INTG/SOLVE/DIFEQ/FOUR/Komplex/Hyperbolicus, GUI-E2E-Smoke erweitert um einen Math-Pac-Workflow

**Scope boundary (locked 2026-05-13, bestätigt 2026-05-16):** v3.0 ist Math 1 Pac only. Stat 1 deferred zu v3.1; Time + Advantage zu v3.2 / v3.3. HP-copyrighted ROM-Image-Redistribution bleibt permanent ausgeschlossen — wir liefern BEHAVIORAL Emulation der dokumentierten Funktionen (Owner's Manual als Verhaltens-Spec), nicht die ROM-Bytes.

**Build sequence:** core (XROM-Framework + Math-1-Ops) → cli (Tastenbelegung + Modal-Erweiterungen) → docs (Function-Matrix v3.0) → gui (key_map + KEY_DEFS + Modal-Routing) → tests (Coverage + Accuracy-Cases).

---

## Project History

**Shipped milestones:**
- v1.0 CLI (2026-05-08) — Phases 1–8, foundational RPN engine + TUI
- v1.1 CLI Feature Completeness (2026-05-09) — Phases 9–12, EEX/STO-Arith/Print/Synthetic
- v2.0 Tauri GUI (2026-05-10) — Phases 13–18, pixel-perfect HP-41C desktop app
- v2.1 Card Reader + Keyboard Authenticity (2026-05-13) — recorded as quick tasks (no Phase 19 GSD directory); 50 commits since `v2.0` tag
- v2.2 HP-41CV Feature Completeness (2026-05-15) — Phases 20–27, 26 plans; ROM-built-in set komplett (≈130 ops); coverage gate atomically auf 95 % gehoben; WebdriverIO E2E-Smoke auf Ubuntu; tag `v2.2` auf `main`
- v3.0 Math Pac I Emulation (2026-05-18) — Phases 28–32, 33 plans (26 original + 7 gap-closure); README hard-claim graduated via post-Phase-32 gap-closure run; final coverage 95.39 % lines / 94.26 % regions on `hp41-core`
  - Phase 28 XROM Framework + Math Pac I Core Ops (2026-05-16) — `hp41-core` only; 10 plans; ~40 new Op variants; XROM resolver chain fires LAST; modal-workflow state machine; user-callback re-entrancy infrastructure; 5 irreversible decisions locked (ADR-001 through ADR-005)
  - Phase 29 CLI Integration (2026-05-17) — `hp41-cli` only; 3 plans; `xeq_by_name_local_resolve` wired to `xrom_resolve`; second `OnceLock<Vec<HelpEntry>>` for Math Pac I JSON; ~40 new `op_display_name` arms; modal-prompt routing through `print_buffer`
  - Phase 30 Documentation & ADRs (2026-05-17) — `docs/` + tooling only; 3 plans; matrix-renderer two-input extension; 3 new ADRs; divergence catalog expansion; v3.0 narrative across README/PROJECT.md/CLAUDE.md
  - Phase 31 GUI Integration (2026-05-18) — `hp41-gui` only; 5 plans; CATALOG 2 XROM enumeration + Math Pac I help overlay + LCD-alternation modal prompts + R/S 3-way + Esc cascade + request_cancel channel
  - Phase 32 Test Hardening & Quality Gates (2026-05-18) — `tests/` + `scripts/` + `.github/` + `justfile` only; 10 plans (3 original + 7 gap-closure); meta-gate graduation (`math1_op_test_count` + `xrom_shadowing` actively cross-check 45 Op variants × 14 test files + 52 MATH_1.ops × 18-entry allowlist); `lint_math1_assertions.rs` Pitfall 14 + 17 discipline; `numerical_accuracy.rs` 566 → 763 cases (99.3 % pass); E2E smoke extended (`sinh(1)` + `MATRIX DET` Math Pac I workflows on Ubuntu); `scripts/check-free42-contamination.sh` D-32.7 12-symbol guard in `just ci` + `ci.yml::license-audit` parallel job (D-32.8). Gap-closure run (Plans 32-04..32-10) added ~70 error-branch tests across 9 new files, closing the coverage gate from 91.74 % → 95.39 % lines / 92.14 % → 94.26 % regions; README v3.0 line graduated to the OM-cited hard claim per D-32.5.

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

### Validated (v2.0)

- ✓ GUI-01: `hp41-gui` Tauri v2 binary added to Cargo workspace; builds and launches on macOS, Windows, Linux — v2.0 Phase 13
- ✓ GUI-02: SVG skin renders pixel-perfect HP-41C key layout (44 keys, 9+8+9+9+9 rows, ENTER double-width, authentic HP-41C colors) — v2.0 Phase 16
- ✓ GUI-03: Clickable keys in the SVG skin trigger the same `Op` dispatch as their CLI keyboard counterparts — v2.0 Phase 16
- ✓ GUI-04: HP-41 12-char dot-matrix display and annunciators render in the GUI, updating after every op — v2.0 Phase 15
- ✓ GUI-05: `hp41-core` integrated via Tauri Rust commands — no duplication of CalcState logic (SC-4 invariant verified) — v2.0 Phase 14
- ✓ GUI-06: `hp41-cli` remains fully functional and unmodified after adding `hp41-gui` to the workspace — v2.0 Phase 13
- ✓ WSPC-01/02: Workspace isolation; `just ci` stays green; nested workspace never affects root Cargo resolver — v2.0 Phase 13
- ✓ IPC-01/02: `dispatch_op`/`get_state` Tauri commands; `CalcStateView` ≤300 bytes; physical keyboard fully wired — v2.0 Phases 14+15
- ✓ SKIN-01/02/03: Pixel-perfect SVG skin with click-to-dispatch and CSS press animation — v2.0 Phase 16
- ✓ DISP-01/02: 12-char display + 5 annunciators + X/Y/Z/T/LASTX stack panel — v2.0 Phase 15
- ✓ PERS-01/02: Shared `~/.hp41/autosave.json`; 30s auto-save; scrollable print panel — v2.0 Phase 17
- ✓ PROG-01: PRGM-mode program listing with SST/BST navigation; cross-platform GUI CI — v2.0 Phase 18

### Validated (v2.1)

- ✓ CARD-01: HP 82104A Card Reader behavioral emulation — `Op::Wdta` / `Op::Rdta` / `Op::Wprgm` / `Op::Rdprgm` + `CardOpRequest` drain — v2.1 (PR #9)
- ✓ CARD-02: XEQ-by-name fallback resolves to `builtin_card_op` — works in `op_xeq`, `run_program`, `run_loop` — v2.1 (PR #9)
- ✓ CARD-03: `cards` modules mirrored in hp41-cli and hp41-gui/src-tauri (dir resolution, sanitize, drain); CLI comfort shortcuts `Ctrl+W/R/D/F`; SHA-256 round-trip tests — v2.1 (PR #9)
- ✓ SKIN-06: Authentic 5×8 keyboard layout in `hp41-gui/src/Keyboard.tsx` — 4 top-row mode buttons + 35-key main grid, ENTER 2-wide, three-label `KeyDef` (primary / shifted / alphaChar) — v2.1 (PR #10)
- ✓ INPUT-02: One-shot SHIFT prefix — frontend-only `shiftActive` state, never crosses IPC; consumes itself after dispatch; ALPHA overrides SHIFT (D-divergence) — v2.1 (PR #10)
- ✓ INPUT-03: `run_stop` Tauri command (symmetric with sst_step/bst_step); R/S key click-reachable for the first time — v2.1 (PR #10)
- ✓ UX-04: Stub-error pattern (D-5) — `pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `asn`, `catalog`, `view`, `xeq_prompt`, `gto_prompt`, `lbl_prompt` return `GuiError` surfaced as 2 s toast; never silently discarded — v2.1 (PR #10)

### Validated (v2.2 — HP-41CV Feature Completeness, shipped 2026-05-15)

**Core math / conversions (Phase 20):**
- ✓ FN-MATH-01: `PI`, `P→R`, `R→P` (polar/rect conversion respecting angle mode) — v2.2 Phase 20
- ✓ FN-MATH-02: `RND` (round X to current display setting), `FRC` (fractional part) — v2.2 Phase 20
- ✓ FN-MATH-03: `MOD` (Y mod X, sign follows Y per HP-41C/CV QRG), `ABS`, `FACT` (factorial 0–69), `SIGN` — v2.2 Phase 20
- ✓ FN-STACK-01: `R↑` (roll up — mirror of `Rdn`) — v2.2 Phase 20

**Core flags & display (Phase 21):**
- ✓ FN-FLAG-01: 56 user flags + system flags 00–55 as `flags: u64` on `CalcState` — v2.2 Phase 21
- ✓ FN-FLAG-02: `SF n`, `CF n`, `FS? n`, `FC? n`, `FS?C n`, `FC?C n` — v2.2 Phase 21
- ✓ FN-DISP-01: `VIEW nn`, `AVIEW` — v2.2 Phase 21
- ✓ FN-DISP-02: `PROMPT`, `AON` / `AOFF`, `CLD` — v2.2 Phase 21
- ✓ FN-SOUND-01: `BEEP` / `TONE n` (CLI silent stubs; GUI real tones via Web Audio) — v2.2 Phase 21

**Core program control (Phase 22):**
- ✓ FN-PROG-01: `STOP`, `PSE` — v2.2 Phase 22
- ✓ FN-PROG-02: `CLP`, `DEL nnn`, `INS` — v2.2 Phase 22
- ✓ FN-PROG-03: `GTO IND nn`, `XEQ IND nn` — v2.2 Phase 22
- ✓ FN-MEM-01: `SIZE`, `PACK`, `MEM LOST`, `CATALOG 1` — v2.2 Phase 22
- ✓ FN-USER-01: `ASN`, `CLA`, `CLST` — v2.2 Phase 22

**Core ALPHA & indirection (Phases 23, 24):**
- ✓ FN-ALPHA-01: `ARCL nn`, `ASTO nn` — v2.2 Phase 23
- ✓ FN-ALPHA-02: `ATOX`, `XTOA` — v2.2 Phase 23
- ✓ FN-ALPHA-03: `AROT n`, `POSA` — v2.2 Phase 23
- ✓ FN-IND-01: 11-variant `*Ind` family (`STO IND`, `RCL IND`, `ISG IND`, `DSE IND`, `SF IND`, `CF IND`, `FS? IND`, `FC? IND`, `STO+/-/×/÷ IND`) — v2.2 Phase 24

**CLI integration & documentation (Phase 25):**
- ✓ FN-CLI-01: Keyboard modals for prompt IDs (`sf_prompt`, `fs_prompt`, `cf_prompt`, register prompts) — v2.2 Phase 25
- ✓ FN-CLI-02: Four conditional tests on f-arith keys (`X=Y`, `X≤Y`, `X>Y`, `X=0`); remaining 8 routed through XEQ-by-name modal — v2.2 Phase 25
- ✓ FN-CLI-03: `?` help overlay JSON-derived from `docs/hp41cv-functions.json` — v2.2 Phase 25
- ✓ FN-DOC-01: `docs/hp41cv-function-matrix.md` regenerated from canonical JSON via `scripts/docs-matrix` (`just docs-matrix` / `just docs-matrix-check`) — v2.2 Phase 25

**GUI integration & polish (Phase 26):**
- ✓ FN-GUI-01: All v2.2 key IDs in `key_map.rs::resolve` + `KEY_DEFS` with three-label shift/alpha bindings — v2.2 Phase 26
- ✓ FN-GUI-02: Modal routing for previously-stubbed prompt IDs replaces `unknown key` toast — v2.2 Phase 26
- ✓ FN-GUI-03: Stub-error arm shrunk to v3.x module-pac functions only — v2.2 Phase 26
- ✓ SKIN-04: 14-segment SVG font for LCD rendering — v2.2 Phase 26
- ✓ SKIN-05: `?` keyboard-shortcut overlay (ported from CLI `help_data.rs`) — v2.2 Phase 26
- ✓ PROG-02: USER-mode keyboard-assignment display — v2.2 Phase 26
- ✓ PROG-03: `prgm_mode` binding rebound to `p` key — v2.2 Phase 26

**Test hardening (Phase 27):**
- ✓ FN-QUAL-01: `hp41-core` coverage 95.25 % (gate atomically raised 80 % → 95 %, D-27.2) — v2.2 Phase 27
- ✓ FN-QUAL-02: 566-case numerical accuracy at 99.1 % (561/566); v1.x 503-case baseline floor 498 preserved — v2.2 Phase 27
- ✓ FN-QUAL-05: WebdriverIO + tauri-driver E2E smoke on Ubuntu (`e2e-linux` job in `ci-gui.yml`) — v2.2 Phase 27
- ✓ D-27.14: Vitest CI-gated in `just gui-ci` (142/142 tests) — v2.2 Phase 27

### Active (v3.0 — Math 1 Pac Emulation)

*Stand 2026-05-18: v3.0 fully shipped (Phases 28–32, 33/33 plans). Test/CI infrastructure plus README hard-claim graduation closed via post-Phase-32 gap-closure run (Plans 32-04..32-10). Final coverage: 95.39 % lines / 94.26 % regions on `hp41-core`; all `ops/math1/*.rs` files ≥ 90 % per ROADMAP SC-1. README v3.0 line graduated to OM-cited "feature-complete per Owner's Manual 00041-90034" per D-32.5. Scope remains Math 1 Pac only (XROM-Framework + Matrix/Komplex/Polynom/Integration/Solver/Vektor). Stat 1 → v3.1.*

### Out of Scope

- v2.0 GUI advanced features (module emulation, skin themes) — deferred until core GUI is stable
- FR-18 Multiple skin themes — GUI-only, post-v2.x
- **FR-21 Module emulation (Math 1 / Stat 1 / Time / Advantage Pacs) — entire scope of v3.x (locked 2026-05-13)**
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
v2.0 Phase 15 (2026-05-10): Display & Keyboard complete — React `App.tsx` renders 12-char display, 5 annunciators, and X/Y/Z/T/LASTX stack panel; `useCallback`+`useEffect` keyboard listener with `e.repeat` guard (SC-4 fix) and modal-key silencing; `CalcStateView` extended with `y_str`/`z_str`/`t_str`/`lastx_str`/`in_eex_mode`; `eex_chs` branch wired before `key_map::resolve()`; Tailwind removed from scaffold. 5/5 SC human-verified, 13/13 Rust tests GREEN.

v2.0 Phase 16 (2026-05-10): SVG Skin complete — `Keyboard.tsx` with 44-key HP-41C SVG layout (9+8+9+9+9 rows), authentic color scheme, CSS `scale(0.92)` press animation with `transform-box: fill-box`; `handleKeyClick` dispatches to `dispatch_op`; all 23 named KEY_DEFS IDs pass `key_map::resolve()` (Wave 0 gate). 5/5 SC human-verified.

v2.0 Phase 17 (2026-05-10): Persistence & Print Output complete — `persistence.rs` in `hp41-gui` with `dirs` dep; 30s auto-save thread; startup load from `~/.hp41/autosave.json` (shared with CLI); scrollable print panel with auto-show and history accumulation. 5/5 SC human-verified; 'p' key remapped to `prx` for Phase 17 SC-5.

v2.0 Phase 18 (2026-05-10): Program Listing & CI/CD complete — `format_all_steps()` + `handle_sst`/`handle_bst` Tauri commands; `CalcStateView.program_steps`+`pc`; conditional PRGM panel in `App.tsx` with F7/F8 bindings and `activeStepRef` auto-scroll; `ci-gui.yml` 3-OS matrix CI independent from `ci.yml`. 5/5 SC verified.

*Last updated: 2026-05-10 — v2.0 Tauri GUI milestone complete (Phases 13–18); next milestone v2.1*

v2.2 HP-41CV Feature Completeness shipped 2026-05-15 (Phases 20–27, 8/8 phases, 26/26 plans) — full ROM built-in set across `hp41-core` + `hp41-cli` + `hp41-gui`; coverage gate atomically raised 80 % → 95 % (D-27.2), achieved 95.25 % lines / 93.75 % regions; 566-case numerical accuracy at 99.1 %; WebdriverIO + tauri-driver E2E smoke green on Ubuntu (`e2e-linux` CI job); Vitest now CI-gated. v1.x 503-case baseline floor 498 preserved per D-27.6. Tag `v2.2` on `main`.

*Last updated: 2026-05-18 — v3.0 Phase 31 (GUI Integration) shipped (5/5 plans, 7/7 GUI-01..07 must-haves automated-verified, 3 manual GUI smoke items deferred to UAT); CATALOG 2 XROM enumeration + Math Pac I help overlay + LCD-alternation modal prompts + R/S 3-way + Esc cascade + request_cancel channel all land in this phase. Phase 32 (Test Hardening) next.*

*Last updated: 2026-05-18 — v3.0 Phase 32 (Test Hardening & Quality Gates) fully shipped (10/10 plans: 3 original + 7 gap-closure). Original Phase 32 ship (Plans 32-01..32-03) delivered the test/CI infrastructure: meta-gate graduation, `lint_math1_assertions.rs`, `numerical_accuracy.rs` 566 → 763 cases at 99.3 % pass rate, E2E smoke extended with `sinh(1)` + `MATRIX DET` Math Pac I workflows on Ubuntu via `ci-gui.yml::e2e-linux`, Free42 GPL-contamination guard (D-32.7 12-symbol policy) in `just ci` + `ci.yml::license-audit` parallel job per D-32.8. Post-ship gap-closure run (Plans 32-04..32-10) added ~70 risk-weighted error-branch tests across 9 new `hp41-core/tests/` files plus the CR-01 + WR-01..07 cleanups, lifting coverage from 91.74 % → 95.39 % lines / 92.14 % → 94.26 % regions and graduating the README v3.0 line to the OM-cited hard claim "feature-complete per Owner's Manual 00041-90034" per D-32.5.*
