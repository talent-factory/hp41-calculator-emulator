# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator
**Current milestone:** v1.1 (in progress)

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, shipped 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- 🚧 **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix, STO modals, print emulation, synthetic programming (in progress)
- 📋 **v2.0 GUI** — Tauri desktop app (hp41-gui) reusing hp41-core unchanged (planned)

---

## Phases

<details>
<summary>✅ v1.0 CLI (Phases 1–8) — SHIPPED 2026-05-08</summary>

- [x] **Phase 1: Foundation** — Cargo workspace, CalcState, 4-level HP-41 stack with lift semantics (completed 2026-05-06)
- [x] **Phase 2: Core Math** — Arithmetic, trig, formatting, registers, ALPHA mode (completed 2026-05-07)
- [x] **Phase 3: Programming Engine** — LBL/GTO/XEQ/RTN/conditionals/ISG/DSE (completed 2026-05-07)
- [x] **Phase 4: TUI & Input** — ratatui display, annunciators, keyboard mapping (completed 2026-05-07)
- [x] **Phase 5: Persistence & UX** — JSON state, auto-save, USER mode, sample programs (completed 2026-05-07)
- [x] **Phase 6: Science & Engineering** — Statistics, HMS/H conversions (completed 2026-05-07)
- [x] **Phase 7: Hardening** — Zero panics, 94.87% coverage, 500-case accuracy, CI matrix (completed 2026-05-07)
- [x] **Phase 8: Tech Debt Cleanup** — EEX fix, SIN/'q', CLREG/'g', AlphaClear/Delete, help accuracy (completed 2026-05-08)

**Full phase details:** [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md)

</details>

### 🚧 v1.1 CLI Feature Completeness (In Progress)

**Milestone Goal:** Close remaining HP-41 behavioral gaps — correct EEX trailing-e hardware behavior, add STO arithmetic keyboard modals, print emulation (PRX/PRA/PRSTK), and a curated synthetic programming subset.

- [x] **Phase 9: Infrastructure & EEX Fix** - Bump MSRV + rust_decimal; correct EEX trailing-e-without-exponent to hardware behavior (completed 2026-05-08)
- [x] **Phase 10: STO Arithmetic Modals** - Wire the existing op_sto_arith core to a 3-step keyboard modal (S → op → register) (completed 2026-05-08)
- [x] **Phase 11: Print Emulation** - Add PRX/PRA/PRSTK ops with print_buffer on CalcState; optional file log via --print-log (completed 2026-05-08)
- [ ] **Phase 12: Synthetic Programming** - GETKEY, NULL, hidden registers M/N/O, and a hex-byte insertion modal

---

## Phase Details

### Phase 9: Infrastructure & EEX Fix
**Goal**: The project compiles on MSRV 1.85, all dependency versions are consistent, and EEX entry behaves identically to HP-41 hardware — trailing e without exponent digits commits as exponent 00; empty-buffer EEX inserts implicit mantissa 1; TUI shows exponent placeholder cursor.
**Depends on**: Phase 8 (v1.0 codebase)
**Requirements**: INFRA-01, INPUT-01, INPUT-02, INPUT-03
**Success Criteria** (what must be TRUE):
  1. Typing `1.5` then EEX then ENTER pushes `1.5` to the stack (exponent treated as 00), not a silent discard
  2. Pressing EEX on an empty entry buffer shows `1   _` in exponent entry mode, matching HP-41 hardware behavior
  3. While in partial-exponent state the TUI display shows a cursor placeholder (e.g., `1.5E_ _`) confirming exponent entry is pending
  4. `just ci` passes with Rust 1.85 toolchain; `Cargo.toml` declares `rust-version = "1.85"` and `rust_decimal` is pinned to 1.42
  5. The previously-inverted test `test_flush_trailing_e_without_exponent_returns_err` passes with corrected (hardware-faithful) assertion
**Plans**: 3 plans
  - [x] 09-01-PLAN.md — INFRA-01: Bump workspace MSRV to 1.85, rust_decimal to 1.42, add MSRV CI job
  - [x] 09-02-PLAN.md — INPUT-01 (core): flush_entry_buf trailing-e normalization + invert wrong-behavior test
  - [x] 09-03-PLAN.md — INPUT-01/02/03 (cli): EEX guards + 2-digit cap + exponent placeholder rendering

### Phase 10: STO Arithmetic Modals
**Goal**: Users can perform STO+, STO-, STO×, STO÷ to any numbered register (R00–R99) or stack register (Y/Z/T/LASTX) via a 3-step keyboard modal, with Esc cancellation at any step producing no side effects.
**Depends on**: Phase 9
**Requirements**: STOA-01, STOA-02, STOA-03
**Success Criteria** (what must be TRUE):
  1. Pressing `S`, then `+`, then `05` executes STO+ into R05 — X is added to R05 in place, no other state changes
  2. Pressing `S`, then `-`, then `Y` executes STO- into the stack Y register, leaving X unchanged
  3. Pressing Esc at any modal step (after `S`, after the arithmetic op key, or after the first register digit) cancels the operation with no state change
  4. The TUI shows the current modal step (e.g., `STO _` after `S`, `STO+ _` after the op key) so the user always knows which step they are on
**Plans**: 3 plans
  **Wave 1** *(parallel)*
  - [x] 10-01-PLAN.md — STOA-03 (core): StackReg enum, Op::StoArithStack variant, op_sto_arith_stack function, dispatch() + execute_op() arms
  - [x] 10-03-PLAN.md — STOA-01/02/03 (help): fix 4 placeholder entries in help_data.rs
  **Wave 2** *(blocked on Wave 1 Plan 10-01 completion)*
  - [x] 10-02-PLAN.md — STOA-01/02 (cli): step-2 routing in StoRegister arm, Y/Z/T/L stack-reg dispatch in StoAdd/Sub/Mul/Div arms, remove dead_code
  **Cross-cutting constraints:**
  - `Op::StoArithStack` variant from 10-01 must be visible before 10-02 can compile
  - `#![deny(clippy::unwrap_used)]` applies throughout hp41-core — all new code uses `?`-propagation
**UI hint**: yes

### Phase 11: Print Emulation
**Goal**: PRX, PRA, and PRSTK operations produce formatted print output — visible in the console and optionally appended to a file — while hp41-core remains free of any I/O dependency by buffering output through a new `print_buffer: Vec<String>` field on CalcState.
**Depends on**: Phase 10
**Requirements**: PRNT-01, PRNT-02, PRNT-03, PRNT-04
**Success Criteria** (what must be TRUE):
  1. Executing PRX writes X in current display format (FIX/SCI/ENG), right-aligned to 24 characters, to the console
  2. Executing PRA writes the ALPHA register contents, left-aligned to 24 characters, to the console
  3. Executing PRSTK writes the full stack in hardware order (T, Z, Y, X, LASTX, ALPHA), one line per register, to the console
  4. Starting `hp41-cli` with `--print-log <path>` causes all PRX/PRA/PRSTK output to be appended to the specified file in addition to the console
  5. Existing v1.0 JSON save files load without error after CalcState gains the `print_buffer` field (the field carries `#[serde(default)]`)
**Plans**: 4 plans (3 original + 1 gap closure)
  **Wave 0**
  - [x] 11-00-PLAN.md — PRNT-01/02/03/04: test scaffold (print_tests.rs with RED failing tests)
  **Wave 1** *(blocked on Wave 0)*
  - [x] 11-01-PLAN.md — PRNT-01/02/03 (core): print_buffer on CalcState, ops/print.rs module, Op::PRX/PRA/PRSTK variants, dispatch() + execute_op() arms
  **Wave 2** *(blocked on Wave 1)*
  - [x] 11-02-PLAN.md — PRNT-01/02/03/04 (cli): PrintModal keyboard modal, 'P' interceptor, call_dispatch_and_drain, print_log_writer, --print-log arg, PRNT: _ display, help entries
  **Wave 3** *(blocked on Wave 2 completion — gap closure)*
  - [x] 11-03-PLAN.md — CR-01/CR-03: drain_and_show_print_output() helper + 3 run_program call sites + serde(skip) on print_buffer
  **Cross-cutting constraints:**
  - `Op::PRX/PRA/PRSTK` variants from 11-01 must be visible before 11-02 can compile
  - `#![deny(clippy::unwrap_used)]` applies throughout hp41-core — all new core code uses `?`-propagation
  - `print_buffer.push()` is hp41-core-only; no `println!`/`eprintln!` allowed in `ops/print.rs`

### Phase 12: Synthetic Programming
**Goal**: Users can use GETKEY, NULL, and the hidden registers M/N/O inside keystroke programs, and can insert a synthetic op via a 2-digit hex byte modal that enforces a curated safe subset, expanding programmable power without unsafe byte codes.
**Depends on**: Phase 11
**Requirements**: SYNT-01, SYNT-02, SYNT-03, SYNT-04
**Success Criteria** (what must be TRUE):
  1. A program containing GETKEY pushes the HP-41 row-column key code of the last pressed key to X when the step executes
  2. A program containing NULL executes without changing any stack register, any numbered register, or the lift flag (Neutral stack-lift effect)
  3. `STO M` / `RCL M`, `STO N` / `RCL N`, `STO O` / `RCL O` work in programs identically to numbered registers and survive a JSON save/load round-trip
  4. The hex-byte insertion modal accepts a 2-digit hex code from a curated safe subset and inserts the synthetic op at the current program step; codes outside the safe subset are rejected with a visible error in the HP-41 display area
**Plans**: 3 plans
  **Wave 0**
  - [ ] 12-00-PLAN.md — SYNT-01/02/03/04: test scaffold (synthetic_tests.rs with RED failing tests)
  **Wave 1** *(blocked on Wave 0)*
  - [ ] 12-01-PLAN.md — SYNT-01/02/03/04 (core): CalcState fields (last_key_code, reg_m/n/o), 9 new Op variants, dispatch + execute_op arms, registers.rs functions, synthetic_byte_to_op, prgm_display arms
  **Wave 2** *(blocked on Wave 1)*
  - [ ] 12-02-PLAN.md — SYNT-01/03/04 (cli): keycode_to_hp41_code, last_key_code update, 'X' interceptor (PRGM-mode gated), HexModal handler with synthetic_byte_to_op validation, M/N/O branches in StoRegister/RclRegister, ui.rs HexModal arm, help_data.rs Synthetic Programming category
  **Cross-cutting constraints:**
  - All 9 new `Op` variants from 12-01 must be visible before 12-02 can compile
  - `prgm_display.rs` exhaustive match means 12-01 MUST land core variants AND display arms together
  - `#![deny(clippy::unwrap_used)]` applies throughout hp41-core — all new core code uses `?`-propagation or `.expect("reason")`
  - HexModal byte validation MUST happen before `state.program.insert()` (security invariant T-12-W2-02)
**UI hint**: yes

---

## Progress Table

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 4/4 | Complete | 2026-05-06 |
| 2. Core Math | v1.0 | 7/7 | Complete | 2026-05-07 |
| 3. Programming Engine | v1.0 | 6/6 | Complete | 2026-05-07 |
| 4. TUI & Input | v1.0 | 5/5 | Complete | 2026-05-07 |
| 5. Persistence & UX | v1.0 | 11/11 | Complete | 2026-05-07 |
| 6. Science & Engineering | v1.0 | 3/3 | Complete | 2026-05-07 |
| 7. Hardening | v1.0 | 6/6 | Complete | 2026-05-07 |
| 8. Tech Debt Cleanup | v1.0 | 3/3 | Complete | 2026-05-08 |
| 9. Infrastructure & EEX Fix | v1.1 | 3/3 | Complete | 2026-05-08 |
| 10. STO Arithmetic Modals | v1.1 | 3/3 | Complete | 2026-05-08 |
| 11. Print Emulation | v1.1 | 4/4 | Complete | 2026-05-08 |
| 12. Synthetic Programming | v1.1 | 0/3 | Not started | - |
