# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator
**Current milestone:** v2.0 Tauri GUI (in progress)

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, shipped 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix, STO modals, print emulation, synthetic programming — SHIPPED 2026-05-09
- 🚧 **v2.0 Tauri GUI** — Phases 13–18 (in progress)

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

<details>
<summary>✅ v1.1 CLI Feature Completeness (Phases 9–12) — SHIPPED 2026-05-09</summary>

- [x] **Phase 9: Infrastructure & EEX Fix** - Bump MSRV + rust_decimal; correct EEX trailing-e-without-exponent to hardware behavior (completed 2026-05-08)
- [x] **Phase 10: STO Arithmetic Modals** - Wire the existing op_sto_arith core to a 3-step keyboard modal (S → op → register) (completed 2026-05-08)
- [x] **Phase 11: Print Emulation** - Add PRX/PRA/PRSTK ops with print_buffer on CalcState; optional file log via --print-log (completed 2026-05-08)
- [x] **Phase 12: Synthetic Programming** - GETKEY, NULL, hidden registers M/N/O, and a hex-byte insertion modal (completed 2026-05-09)

</details>

### 🚧 v2.0 Tauri GUI (In Progress)

**Milestone Goal:** Ship a pixel-perfect HP-41C desktop app using Tauri v2 + React + TypeScript, reusing `hp41-core` unchanged alongside the existing CLI.

- [ ] **Phase 13: Workspace Skeleton** - Add hp41-gui as a nested Tauri v2 workspace member; just gui-dev launches an empty window; just ci stays green
- [ ] **Phase 14: IPC Layer** - Tauri commands dispatch_op/get_state; CalcStateView ~200 bytes; key_map.rs; GuiError; print_buffer drained per call
- [ ] **Phase 15: Display & Keyboard** - React display panel with 12-char output and annunciators; physical keyboard wiring with same bindings as hp41-cli
- [ ] **Phase 16: SVG Skin** - Pixel-perfect HP-41C SVG key layout (9×5, ENTER double-width, HP colors); click handlers; CSS press animation
- [ ] **Phase 17: Persistence & Print Output** - Shared ~/.hp41/autosave.json save/load; 30s auto-save; scrollable print output panel
- [ ] **Phase 18: Program Listing & CI/CD** - PRGM mode program listing with SST/BST navigation; cross-platform GUI CI job

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
  - [x] 12-00-PLAN.md — SYNT-01/02/03/04: test scaffold (synthetic_tests.rs with RED failing tests)
  **Wave 1** *(blocked on Wave 0)*
  - [x] 12-01-PLAN.md — SYNT-01/02/03/04 (core): CalcState fields (last_key_code, reg_m/n/o), 9 new Op variants, dispatch + execute_op arms, registers.rs functions, synthetic_byte_to_op, prgm_display arms
  **Wave 2** *(blocked on Wave 1)*
  - [x] 12-02-PLAN.md — SYNT-01/03/04 (cli): keycode_to_hp41_code, last_key_code update, 'X' interceptor (PRGM-mode gated), HexModal handler with synthetic_byte_to_op validation, M/N/O branches in StoRegister/RclRegister, ui.rs HexModal arm, help_data.rs Synthetic Programming category
  **Cross-cutting constraints:**
  - All 9 new `Op` variants from 12-01 must be visible before 12-02 can compile
  - `prgm_display.rs` exhaustive match means 12-01 MUST land core variants AND display arms together
  - `#![deny(clippy::unwrap_used)]` applies throughout hp41-core — all new core code uses `?`-propagation or `.expect("reason")`
  - HexModal byte validation MUST happen before `state.program.insert()` (security invariant T-12-W2-02)
**UI hint**: yes

### Phase 13: Workspace Skeleton
**Goal**: Users can launch an empty Tauri v2 window via `just gui-dev`, the hp41-gui crate is a nested workspace member that does not affect `cargo build --workspace`, and `just ci` (the CLI pipeline) remains green without modification.
**Depends on**: Phase 12
**Requirements**: WSPC-01, WSPC-02
**Success Criteria** (what must be TRUE):
  1. Running `just gui-dev` on macOS opens a blank Tauri window with the title "HP-41 Calculator" and exits cleanly on close
  2. Running `just ci` after adding hp41-gui completes with the same pass/fail outcome as before — zero regressions in hp41-cli or hp41-core
  3. Running `cargo build --workspace` from the repo root builds hp41-core and hp41-cli but does NOT attempt to build the Tauri binary (nested workspace isolation confirmed)
  4. The `tauri` and `tauri-build` crates appear only in `hp41-gui/src-tauri/Cargo.toml`, not in the root `[workspace.dependencies]`
  5. The CI matrix (Windows, macOS, Ubuntu) continues to pass the existing CLI jobs with no new failures introduced
**Plans**: TBD
**UI hint**: yes

### Phase 14: IPC Layer
**Goal**: All calculator operations reach `hp41-core` via Tauri Rust commands; the IPC response is a lean `CalcStateView` (~200 bytes) that never duplicates core logic; `print_buffer` is explicitly drained on every command; a `key_map.rs` module in hp41-gui resolves string key IDs to `Op` variants so the frontend never references Rust enums directly.
**Depends on**: Phase 13
**Requirements**: IPC-01
**Success Criteria** (what must be TRUE):
  1. Invoking `dispatch_op` with a valid key ID (e.g., `"enter"`, `"plus"`, `"sin"`) updates CalcState and returns a `CalcStateView` JSON payload of ≤300 bytes
  2. Invoking `dispatch_op` with an unknown key ID returns a serialized `GuiError` — no panic, no silent discard
  3. The `print_buffer` field is drained and its contents included in `CalcStateView.print_lines` on every command response — no print output is silently dropped
  4. `CalcState` logic (stack operations, dispatch, arithmetic) is entirely within `hp41-core`; `hp41-gui/src-tauri` contains zero duplicated calculator logic
  5. A `type AppState = Mutex<CalcState>` alias is used throughout command handlers, making incorrect state extraction a compile error rather than a runtime panic
**Plans**: TBD
**UI hint**: yes

### Phase 15: Display & Keyboard
**Goal**: Users can see the HP-41 12-character display string and all five annunciators update in the GUI after every operation, and can drive all calculator functions from the physical keyboard using the same key bindings as hp41-cli — without requiring mouse input.
**Depends on**: Phase 14
**Requirements**: DISP-01, DISP-02, IPC-02
**Success Criteria** (what must be TRUE):
  1. Pressing any key (e.g., `3`, `+`, `ENTER`) from the physical keyboard triggers the correct `dispatch_op` call and the 12-char display updates within one frame
  2. All five annunciators (USER, PRGM, ALPHA, RAD, GRAD) reflect the current CalcState and change visually when mode is toggled (e.g., RAD annunciator lights when switching to radians mode)
  3. The stack register panel shows current X/Y/Z/T/LASTX values and updates after every operation, matching the values that hp41-cli would show
  4. Physical keyboard event listeners use `useCallback` and always return a cleanup function in `useEffect`, so no duplicate IPC calls fire on a single keypress even in React StrictMode
  5. The key binding set covers all bindings present in hp41-cli's `key_to_op()` function — no key that works in the CLI is silently ignored in the GUI
**Plans**: TBD
**UI hint**: yes

### Phase 16: SVG Skin
**Goal**: Users see a pixel-perfect HP-41C calculator skin rendered as SVG — dark brown body, gold shift legends, 9×5 key grid with ENTER spanning two columns — where every key is individually clickable and shows a CSS scale-down press animation on each click.
**Depends on**: Phase 14
**Requirements**: SKIN-01, SKIN-02, SKIN-03
**Success Criteria** (what must be TRUE):
  1. The SVG skin renders all 40 HP-41C keys in the correct 9×5 grid layout with ENTER occupying a double-width position, matching HP-41C hardware proportions
  2. The color scheme matches HP-41C hardware: dark brown body, light-colored key caps for the top row, gold shift legends, white primary legends
  3. Clicking any key in the SVG invokes `dispatch_op` with the correct key ID — the result is identical to pressing the equivalent key in hp41-cli
  4. Each key click triggers a visible CSS scale-down animation (scale to ~0.92, then bounce back) that completes within 150ms without blocking further input
  5. The SVG uses a `viewBox` and scales correctly on Retina/HiDPI displays and at the fixed 400×700 window size without pixelation or layout breakage
**Plans**: TBD
**UI hint**: yes

### Phase 17: Persistence & Print Output
**Goal**: Users can close and reopen the GUI calculator without losing state — the same `~/.hp41/autosave.json` file used by hp41-cli is loaded on startup and written every 30 seconds; v1.x save files from the CLI load without error; print output from PRX/PRA/PRSTK is visible in a scrollable panel rather than silently discarded.
**Depends on**: Phase 15
**Requirements**: PERS-01, PERS-02
**Success Criteria** (what must be TRUE):
  1. After performing operations in the GUI and restarting the app, the stack and register values are restored to their state at last save — no data loss across restarts
  2. A save file created by hp41-cli v1.x loads in the GUI without a parse error or panic — the `print_buffer` field absence is handled by `#[serde(default)]`
  3. After 30 seconds of inactivity the auto-save fires silently in the background — the GUI remains responsive and no blocking occurs on the UI thread
  4. The `~/.hp41/autosave.json` path is used by both hp41-cli and hp41-gui — a state saved in the CLI is visible when the GUI starts next
  5. Executing PRX, PRA, or PRSTK causes formatted output lines to appear in the scrollable print panel; the panel retains previous output and new lines append to the bottom
**Plans**: TBD
**UI hint**: yes

### Phase 18: Program Listing & CI/CD
**Goal**: Users in PRGM mode can view the complete program listing and step through it with SST and BST in the GUI; a cross-platform CI job (macOS, Windows, Ubuntu) builds and tests hp41-gui on every push to paths that affect the GUI or core.
**Depends on**: Phase 17
**Requirements**: PROG-01
**Success Criteria** (what must be TRUE):
  1. Entering PRGM mode in the GUI displays the program listing with step numbers and mnemonic labels matching the format shown in hp41-cli
  2. Pressing the SST key (via keyboard binding or SVG click) advances the program counter by one step and highlights the current step in the listing
  3. Pressing the BST key steps backward one position in the program and the listing scrolls to keep the highlighted step visible
  4. The cross-platform CI job runs on Windows, macOS, and Ubuntu; the build completes without error on all three platforms and is triggered only on changes to `hp41-gui/**` or `hp41-core/**`
  5. `just ci` (the CLI pipeline) and the new GUI CI job are independent — a GUI build failure does not block CLI CI and vice versa
**Plans**: TBD
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
| 12. Synthetic Programming | v1.1 | 3/3 | Complete | 2026-05-09 |
| 13. Workspace Skeleton | v2.0 | 0/? | Not started | - |
| 14. IPC Layer | v2.0 | 0/? | Not started | - |
| 15. Display & Keyboard | v2.0 | 0/? | Not started | - |
| 16. SVG Skin | v2.0 | 0/? | Not started | - |
| 17. Persistence & Print Output | v2.0 | 0/? | Not started | - |
| 18. Program Listing & CI/CD | v2.0 | 0/? | Not started | - |
