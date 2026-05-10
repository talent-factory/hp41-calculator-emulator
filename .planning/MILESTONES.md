# Milestones

## v1.0 — HP-41 Calculator Emulator CLI

**Status:** ✅ SHIPPED 2026-05-08
**Phases:** 8 (Phases 1–8)
**Plans:** 45 total, all complete
**Timeline:** 3 days (2026-05-06 → 2026-05-08)
**Source:** 13,399 lines Rust | 212 files | 68 feat+fix commits

### Delivered

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator — delivered as a keyboard-driven TUI CLI (`hp41-cli`) backed by a UI-agnostic library crate (`hp41-core`).

### Key Accomplishments

1. **4-level RPN stack with full HP-41 stack-lift semantics** — every one of ~130 operations correctly declares Enable/Disable/Neutral; `#![deny(clippy::unwrap_used)]` enforces zero panics at compile time
2. **Complete HP-41 math engine** — arithmetic, trig (DEG/RAD/GRAD), `FIX`/`SCI`/`ENG` formatting, R00–R99 registers, ALPHA mode; 10-digit `rust_decimal` accuracy with mantissa carry fix
3. **Full keystroke programming engine** — `LBL`/`GTO`/`XEQ`/`RTN`, all 12 conditional tests, `ISG`/`DSE` with CCCCC.FFFDD string-split counter (never float arithmetic)
4. **ratatui TUI** with persistent 4-level stack display, 12-char HP-41 alphanumeric display, annunciators, and complete physical keyboard mapping
5. **JSON persistence** — auto-save every 30s, exit save, `USER` mode with custom key assignments, 10 bundled sample programs
6. **Science & Engineering** — Σ+/−, MEAN, SDEV, L.R. (linear regression), HMS↔H conversions
7. **Hardened quality gates** — 2.2ms cold-start (228× under 500ms gate), 94.87% test coverage, 495/500 numerical accuracy (99%), CI green on Windows/macOS/Ubuntu
8. **Tech Debt Cleanup** — EEX scientific notation entry (`from_scientific` fallback), SIN on `'q'` key, CLREG on `'g'` key, `Delete` → AlphaClear in ALPHA mode, help overlay accuracy

### Quality at Ship

| Gate | Target | Achieved |
|------|--------|---------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms |
| Key-press latency | ≤ 50 ms | ~65 ns/op |
| hp41-core coverage | ≥ 80% | 94.87% |
| Numerical accuracy | ≥ 98% (500 cases) | 99% (495/500) |
| Panics in hp41-core | 0 | 0 |
| CI platforms | Win/macOS/Ubuntu | ✅ all green |

### Archives

- [ROADMAP.md](v1.0-ROADMAP.md)
- [REQUIREMENTS.md](v1.0-REQUIREMENTS.md)
- [Milestone Audit](v1.0-MILESTONE-AUDIT.md)

### Known Deferred Items

- EEX trailing-e-without-exponent discards number silently (documented with test)
- STO arithmetic keyboard modals (`STO+/-/×/÷`) keyboard-accessible via programs; interactive modal deferred to v1.1
- Tauri v2 GUI (hp41-gui crate) — deferred to v2.0

---

## v1.1 — HP-41 Calculator Emulator CLI Feature Completeness

**Status:** ✅ SHIPPED 2026-05-09
**Phases:** 4 (Phases 9–12)
**Plans:** 14 total, all complete

### Delivered

- **Phase 9:** MSRV 1.85, rust_decimal 1.42, EEX trailing-e hardware-faithful fix, exponent placeholder in TUI
- **Phase 10:** STO arithmetic modals (S → op → register), stack register support (Y/Z/T/LASTX), Esc cancellation
- **Phase 11:** PRX/PRA/PRSTK print emulation via `print_buffer` on CalcState, `--print-log` file output
- **Phase 12:** GETKEY, NULL, hidden registers M/N/O, 2-digit HexModal (23-entry safe subset)
- **Bugfixes found in review:** Vec::insert panic after ISG/DSE skip-at-end (CR-01); F5 overwriting last_key_code before GETKEY (F5 → 0 in keycode_to_hp41_code)

### Quality at Ship

| Gate | v1.0 | v1.1 |
|------|------|------|
| hp41-cli tests | 86 | 99 |
| hp41-core tests | 150 | 150+ |
| Synthetic tests | — | 21 |
| All requirements | 15/15 complete | ✅ |

### Archives

- [ROADMAP.md](milestones/v1.1-ROADMAP.md)
- [REQUIREMENTS.md](milestones/v1.1-REQUIREMENTS.md)

### Known Deferred Items

- SYNT-05: Full FOCAL byte-code table (~200 codes)
- SYNT-06: GETKEY interrupt-style capture (requires event loop redesign)
- PRNT-05/06: Scrollable print history, ADV/PRREG/TRACE
- STOA-04: STO arithmetic via indirect addressing
- Tauri v2 GUI (hp41-gui crate) — shipped in v2.0

---

## v2.0 — HP-41 Calculator Emulator Tauri GUI

**Status:** ✅ SHIPPED 2026-05-10
**Phases:** 6 (Phases 13–18)
**Plans:** 19 total, all complete
**Timeline:** 2 days (2026-05-09 → 2026-05-10)
**Source:** 183 files changed | 30,358 insertions

### Delivered

A pixel-perfect HP-41C desktop application built with Tauri v2 + React + TypeScript, reusing `hp41-core` unchanged alongside the existing `hp41-cli`.

### Key Accomplishments

1. **Tauri v2 workspace skeleton** — `hp41-gui` nested standalone workspace isolated from CLI Cargo graph; `just gui-dev` launches HP-41 Calculator window; `just ci` stays green; bundle identifier `ch.talent-factory.hp41`
2. **IPC Layer** — `dispatch_op`/`get_state` Tauri v2 commands; `CalcStateView` (~170 bytes, ≤300 limit); `key_map::resolve()` for 50+ named ops + 7 prefix families; Tauri v2.11 permission TOML pattern; `print_buffer` drained on every command
3. **Display & Keyboard** — React `App.tsx` with 12-char HP-41 display, 5 annunciators, X/Y/Z/T/LASTX stack panel; `useCallback`+`useEffect` keyboard listener with `busyRef` debounce; `eex_chs` branch; all hp41-cli bindings covered
4. **SVG Skin** — Pixel-perfect HP-41C 44-key SVG layout (9+8+9+9+9 rows, ENTER double-width); authentic HP-41C color scheme; CSS `scale(0.92)` press animation with `transform-box: fill-box`; Tauri window 400×700
5. **Persistence & Print Output** — Shared `~/.hp41/autosave.json` auto-save thread (30s); v1.x CLI save files load without error; scrollable print panel with auto-show, history accumulation, auto-scroll
6. **Program Listing & CI/CD** — PRGM-mode program listing panel with SST/BST navigation, F7/F8 bindings, `activeStepRef` auto-scroll; cross-platform `ci-gui.yml` (3-OS matrix, path filter, `cargo test` before build, independent from `ci.yml`)

### Archives

- [ROADMAP.md](milestones/v2.0-ROADMAP.md)
- [REQUIREMENTS.md](milestones/v2.0-REQUIREMENTS.md)

### Known Deferred Items (v2.1)

- SKIN-04: 14-segment SVG font for authentic LCD rendering
- SKIN-05: Keyboard shortcut overlay (port `?` help panel from CLI)
- PROG-02: Full keyboard assignment display in USER mode
- `prgm_mode` binding for 'p' key (currently mapped to `prx`)

---
*For current project status, see .planning/ROADMAP.md*
