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
*For current project status, see .planning/ROADMAP.md*
