# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator v1.0 CLI
**Target release:** 2026-09-05
**Milestone:** v1.0 CLI
**Granularity:** Standard (7 phases)
**Requirements coverage:** 25/25 v1 requirements mapped

---

## Phases

- [x] **Phase 1: Foundation** - Cargo workspace, CalcState, stack with HP-41-accurate stack-lift semantics, BCD/f64 decision (completed 2026-05-06)
- [x] **Phase 2: Core Math** - Arithmetic, trig, number formatting, storage registers, ALPHA mode (completed 2026-05-07)
- [x] **Phase 3: Programming Engine** - Keystroke programming, LBL/GTO/XEQ/RTN, conditional tests, ISG/DSE (completed 2026-05-07)
- [x] **Phase 4: TUI & Input** - ratatui display panel, annunciators, physical keyboard mapping (completed 2026-05-07)
- [ ] **Phase 5: Persistence & UX** - State save/load, auto-save, built-in help, USER mode, sample programs
- [ ] **Phase 6: Science & Engineering** - Statistics functions, HMS/H conversions
- [ ] **Phase 7: Hardening** - Performance, cross-platform, test coverage, numerical accuracy suite

---

## Phase Details

### Phase 1: Foundation
**Goal**: A Cargo workspace exists with a `Justfile` covering all build/test/lint/run targets, a compiling `hp41-core` crate that models a correct 4-level HP-41 RPN stack with full stack-lift semantics, resolves the BCD vs f64 numeric representation, and returns typed errors with zero panics.
**Depends on**: Nothing
**Requirements**: CORE-01, CORE-02
**Success Criteria** (what must be TRUE):
  1. User can push values onto the 4-level stack (X/Y/Z/T) and the LASTX register captures the correct value after each operation
  2. ENTER, arithmetic result, CLX, CHS, and RCL each produce the correct stack-lift enable/disable/neutral behavior as specified in HP-41 documentation
  3. `cargo check -p hp41-core` passes with zero UI or CLI dependencies
  4. The numeric representation decision (BCD struct or `rust_decimal` wrapping f64 with 10-digit rounding) is committed to code and documented in an ADR comment in `state.rs`
  5. `just --list` shows all standard recipes (build, test, lint, run, ci) and `just ci` passes on macOS
**Plans**: 4 plans

Plans:
- [x] 01-PLAN-01.md — Cargo workspace scaffold + Justfile + cargo-llvm-cov install
- [x] 01-PLAN-02.md — HpError, HpNum, CalcState/Stack types + LiftEffect helpers + ADR comment
- [x] 01-PLAN-03.md — Op enum, dispatch, arithmetic ops (add/sub/mul/div), stack ops (enter/clx/chs/rdn/xy_swap/lastx)
- [x] 01-PLAN-04.md — CORE-01 unit tests, CORE-02 lift-effect tests, proptest suite, `just ci` gate

### Phase 2: Core Math
**Goal**: Users can perform the complete HP-41 arithmetic, trigonometric, and formatting operation set, store and recall values in R00–R99 registers, and enter alphanumeric strings in ALPHA mode — all with HP-41-accurate 10-digit results.
**Depends on**: Phase 1
**Requirements**: MATH-01, MATH-02, MATH-03, REGS-01, ALPH-01
**Success Criteria** (what must be TRUE):
  1. User can compute `+ − × ÷`, `1/x`, `√x`, `x²`, `Y^X`, `LN`, `LOG`, `e^x`, `10^x` and see a 10-digit-accurate result in the display
  2. User can press SIN/COS/TAN and their inverses after switching between DEG, RAD, and GRAD modes; a 90° input produces exactly 1 in SIN
  3. User can cycle FIX 4 → SCI 2 → ENG 3 and the same number renders in the correct notation each time
  4. User can STO a value into R00–R99, recall it with RCL, and perform STO+/−/×/÷ against a register — all matching HP-41 hardware behavior
  5. User can activate ALPHA mode, type a 24-character string, and confirm it is stored in the ALPHA register
**Plans**: 7 plans

Plans:
**Wave 1**
- [x] 02-01-PLAN.md — CalcState expansion (AngleMode/DisplayMode enums, 6 new fields) + unary_result() + Op skeleton stubs
**Wave 2** *(blocked on Wave 1)*
- [x] 02-02-PLAN.md — HpNum math methods (14 methods: recip/sqrt/sq/ln/log10/exp/exp10/powd + trig with f64 bridge)
- [x] 02-03-PLAN.md — Wave 0 test scaffolds (math_tests, trig_tests, format_tests, register_tests, alpha_tests, extend lift_tests)
**Wave 3** *(blocked on Wave 2)*
- [x] 02-04-PLAN.md — ops/math.rs with all 17 math/trig/angle ops + dispatch wiring
- [x] 02-05-PLAN.md — ops/registers.rs (STO/RCL/STO-arith/CLREG) + format.rs (FIX/SCI/ENG) + dispatch wiring
- [x] 02-06-PLAN.md — ops/alpha.rs (AlphaToggle/AlphaAppend/AlphaClear) + dispatch wiring
**Wave 4** *(blocked on Wave 3)* — human checkpoint
- [x] 02-07-PLAN.md — entry_buf flush in dispatch() + entry_buf_tests + just ci green gate

**Cross-cutting constraints:**
- All new Op variants must declare LiftEffect (Enable/Disable/Neutral) per Phase 1 convention
- All ops in hp41-core must return `Result<(), HpError>` — zero panics, zero unwrap in non-test code

### Phase 3: Programming Engine
**Goal**: Users can record, store, and execute keystroke programs with labels, branches, subroutine calls, conditional tests, and loop control — with ISG/DSE counter-field behavior identical to HP-41 hardware.
**Depends on**: Phase 2
**Requirements**: PROG-01, PROG-02
**Success Criteria** (what must be TRUE):
  1. User can enter PRGM mode, key in a program with LBL, GTO, XEQ, RTN, and RTN terminates execution returning to the caller
  2. User can run a program containing all conditional tests (x=0?, x<0?, x>y?, etc.) and observe correct skip-next-step behavior when the condition is false
  3. User can write a counting loop using ISG with counter register value `1.00500` (current=1, final=5, step=1) and observe it increment exactly 4 times before falling through
  4. User can nest XEQ calls up to 4 levels deep; a 5th nested XEQ produces a "TRY AGAIN" error without crashing
**Plans**: 6 plans

Plans:
**Wave 1** *(parallel — no dependencies)*
- [x] 03-01-PLAN.md — CalcState Phase 3 fields (program, prgm_mode, pc, call_stack, is_running)
- [x] 03-02-PLAN.md — HpError::CallDepth variant ("try again")
- [x] 03-03-PLAN.md — TestKind enum (12 variants) + Phase 3 Op variants (Lbl/Gto/Xeq/Rtn/PrgmMode/Test/Isg/Dse)
**Wave 2** *(blocked on Wave 1 — parallel with each other)*
- [x] 03-04-PLAN.md — dispatch() prgm_mode gate + flush_entry_buf() routing to program Vec
- [x] 03-05-PLAN.md — ops/program.rs (run_program, run_loop, execute_op, ISG/DSE, TestKind eval) + program_tests.rs full suite
**Wave 3** *(blocked on Wave 2)*
- [x] 03-06-PLAN.md — dispatch() Phase 3 arms wiring + lib.rs run_program export + just ci gate

### Phase 4: TUI & Input
**Goal**: Users interact with the emulator entirely via keyboard in a persistent ratatui terminal panel that shows the 4-level stack, LASTX, 12-character HP-41 display, and all annunciators at all times.
**Depends on**: Phase 3
**Requirements**: DISP-01, DISP-02, INPUT-01
**Success Criteria** (what must be TRUE):
  1. The TUI renders a persistent panel showing X/Y/Z/T, LASTX, the 12-char alphanumeric display, and annunciators (USER, PRGM, ALPHA, SHIFT, RAD/DEG/GRAD) without requiring any user action
  2. Annunciator state updates immediately when the calculator mode changes (e.g., toggling RAD vs DEG flips the annunciator in the same frame)
  3. User can perform any documented calculator operation using only the physical keyboard without consulting an external reference — discoverable key labels are visible in the TUI
  4. Any unhandled panic in hp41-core is caught at the CLI boundary and the terminal is restored to normal (not stuck in raw mode)
**Plans**: 5 plans

Plans:
**Wave 1**
- [x] 04-01-PLAN.md — Cargo.toml deps (ratatui 0.30 + crossterm 0.29 + clap 4.x) + App struct + module skeleton (compiling foundation)
**Wave 2** *(parallel — different files)*
- [x] 04-02-PLAN.md — ui.rs full widget layout: stack panel, display panel, annunciator bar, status bar, key-reference panel
- [x] 04-03-PLAN.md — keys.rs (key_to_op() + KEY_REF_TABLE) + prgm_display.rs (format_step() + op_display_name()) + unit tests
**Wave 3** *(blocked on Wave 2)*
- [x] 04-04-PLAN.md — main.rs clap args wired + manual smoke test checkpoint (human verify)
**Wave 4** *(blocked on Wave 3)*
- [x] 04-05-PLAN.md — just ci gate: full workspace tests + coverage + clippy

### Phase 5: Persistence & UX
**Goal**: Users can save and reload complete calculator state between sessions, auto-save fires every 30 seconds, an inline help system is accessible from within the TUI, USER mode works with persisted key assignments, and a bundled sample program library is ready to load.
**Depends on**: Phase 4
**Requirements**: PERS-01, PERS-02, UX-01, UX-02, UX-03
**Success Criteria** (what must be TRUE):
  1. User can save the full calculator state (stack, registers, programs, flags, USER assignments) to a named JSON file and reload it in a fresh session with all state intact
  2. If the process is killed without a manual save, at most 30 seconds of work is lost — confirmed by checking the auto-save timestamp in the state file
  3. User can press `?` or type `HELP` in the TUI and see a searchable function reference covering all HP-41 operations with their keyboard mappings
  4. User can assign a custom program label to a key in USER mode, toggle USER mode, and observe the key assignment activate; the assignment survives a save/reload cycle
  5. User can load at least 10 bundled sample programs from within the TUI and run them to produce documented outputs
**Plans**: TBD
**UI hint**: yes

### Phase 6: Science & Engineering
**Goal**: Users can perform the HP-41's built-in statistics suite (Σ registers, mean, standard deviation, linear regression) and HMS/H time-and-angle conversion functions.
**Depends on**: Phase 5
**Requirements**: SCI-01, SCI-02
**Success Criteria** (what must be TRUE):
  1. User can enter a data set with Σ+ and compute MEAN, SDEV, and linear regression coefficients that match HP-41 hardware results for the same data set
  2. User can remove an incorrect data point with Σ− and recompute statistics without re-entering the full data set
  3. User can convert 1.3045 (1h 30m 45s in HMS format) to decimal hours with HMS→ and get 1.5125, and convert back to confirm round-trip accuracy
**Plans**: TBD

### Phase 7: Hardening
**Goal**: The v1.0 CLI meets all non-functional quality requirements: cold-start under 0.5 s, key latency under 50 ms, zero panics in core, 80%+ test coverage in `hp41-core`, single-codebase cross-platform builds, and 98%+ numerical agreement with HP-41 hardware across the 500-case test suite.
**Depends on**: Phase 6
**Requirements**: QUAL-01, QUAL-02, QUAL-03, QUAL-04, QUAL-05, QUAL-06
**Success Criteria** (what must be TRUE):
  1. `cargo build --release` produces a working binary on Windows 10+, macOS 12+, and Ubuntu 22.04+ from the same codebase — verified via CI matrix
  2. Cold-start time measured with `hyperfine` on Apple M1 and Intel i5 8th gen is ≤ 0.5 s
  3. Median key-press-to-display-update latency measured over 1000 keystrokes is ≤ 50 ms
  4. `cargo test -p hp41-core` passes with zero panics and `cargo-llvm-cov` reports ≥ 80% line coverage
  5. The 500-case numerical test suite (covering arithmetic, trig, logs, ISG/DSE edge cases, and transcendental function accumulation) passes with ≥ 98% agreement vs HP-41 reference values
**Plans**: TBD

---

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 4/4 | Complete    | 2026-05-06 |
| 2. Core Math | 7/7 | Gaps Found | - |
| 3. Programming Engine | 6/6 | Complete    | 2026-05-07 |
| 4. TUI & Input | 5/5 | Complete    | 2026-05-07 |
| 5. Persistence & UX | 0/? | Not started | - |
| 6. Science & Engineering | 0/? | Not started | - |
| 7. Hardening | 0/? | Not started | - |

---

## Coverage Map

| Phase | Requirements |
|-------|-------------|
| 1. Foundation | CORE-01, CORE-02 |
| 2. Core Math | MATH-01, MATH-02, MATH-03, REGS-01, ALPH-01 |
| 3. Programming Engine | PROG-01, PROG-02 |
| 4. TUI & Input | DISP-01, DISP-02, INPUT-01 |
| 5. Persistence & UX | PERS-01, PERS-02, UX-01, UX-02, UX-03 |
| 6. Science & Engineering | SCI-01, SCI-02 |
| 7. Hardening | QUAL-01, QUAL-02, QUAL-03, QUAL-04, QUAL-05, QUAL-06 |

**Total mapped: 25/25**

---

## Key Decisions Baked In

| Decision | Rationale |
|----------|-----------|
| BCD/f64 decision in Phase 1 | Retrofitting after register code exists is a full data model rewrite (SUMMARY.md critical pitfall) |
| TUI built in Phase 4 (after programming engine) | Can't meaningfully test keystroke programming without display; but core must be complete first |
| QUAL-* deferred to Phase 7 | Quality requirements are cross-cutting; addressed in a dedicated hardening phase to avoid premature optimization |
| SCI-01/02 in Phase 6 (after Persistence) | Stats use R01–R06 Σ registers — registers and programs must be stable before adding higher-order math |
| No async in core | Event loop is `poll → update → redraw`, single-threaded; tokio never enters `hp41-core` |

---
*Roadmap created: 2026-05-06*
*Last updated: 2026-05-07 after Phase 4 planning (5 plans, 4 waves)*
