---
gsd_state_version: 1.0
milestone: v3.0
milestone_name: — Math 1 Pac Emulation
status: planning
last_updated: "2026-05-16T00:00:00.000Z"
last_activity: 2026-05-16 -- Milestone v3.0 started (Math 1 Pac); research-first; phase numbering continues from 28
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-16)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.
**Shipped:** v1.0 CLI (2026-05-08)
**Shipped:** v1.1 CLI Feature Completeness (2026-05-09) — Phases 9–12 complete
**Shipped:** v2.0 Tauri GUI (2026-05-10) — Phases 13–18 complete
**Shipped:** v2.1 Card Reader + Keyboard Authenticity (2026-05-13) — recorded as quick tasks, no Phase 19 GSD directory
**Shipped:** v2.2 HP-41CV Feature Completeness (2026-05-15) — Phases 20–27 complete; 8/8 phases, 26/26 plans, 95.25 % core coverage, CI fully green
**Current focus:** v3.0 Math 1 Pac Emulation — XROM-Modul-Framework + Math-1-Funktionsbibliothek (Matrix / Komplex / Polynom / Integration / Solver / Vektor). Stat 1 → v3.1.
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements — research-first selected; 4 parallel agents (Stack / Features / Architecture / Pitfalls) to run before REQUIREMENTS.md
Last activity: 2026-05-16 — Milestone v3.0 started

---

## Performance Metrics (carried from v2.2 ship)

| Metric | Target | Last measured (v2.2) |
|--------|--------|----------------------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms (M1) — 228× under gate |
| Key-press latency (median) | ≤ 50 ms | ~65 ns/op |
| `hp41-core` test coverage | ≥ 95 % | 95.25 % lines / 93.75 % regions / 97.68 % functions |
| Numerical accuracy | ≥ 98 % (566 cases) | 99.1 % (561/566) — v1.x baseline floor 498/503 preserved |
| Panics in `hp41-core` | 0 | 0 — enforced by `#![deny(clippy::unwrap_used)]` |
| CI platforms | Win/macOS/Ubuntu | All green (`ci.yml` + `ci-gui.yml` + `e2e-linux`) |

---

## v3.0 Phase Plan

*To be filled via `/gsd-roadmapper` after REQUIREMENTS.md is approved. Phase numbering continues from Phase 28 (consistent with v1.0=1-8, v1.1=9-12, v2.0=13-18, v2.1=19, v2.2=20-27).*

---

## Accumulated Context

### Key Decisions (carried forward from v1.x–v2.x)

| Decision | Rationale | Phase |
|----------|-----------|-------|
| BCD vs f64 | `rust_decimal` with 10-digit rounding; avoid custom BCD struct | Phase 1 |
| Stack-lift as `lift_enabled: bool` | Every op declares Enable/Disable/Neutral | Phase 1 |
| `CalcState` as single source of truth | One `&mut CalcState` through all ops; no global state | Phase 1 |
| ISG/DSE string-split counter fields | Never `floor()`/`fmod()` on f64 | Phase 3 |
| `ratatui::init()` not `Terminal::new()` | Installs panic hook for terminal restore | Phase 4 |
| Digit entry via `entry_buf` | Auto-flushed on next non-digit | Phase 4 |
| `serde_json` for persistence | Human-readable, diff-able, versioned JSON | Phase 5 |
| No async in `hp41-core` | Single-threaded event loop | All |
| `#![deny(clippy::unwrap_used)]` | Compile-time zero-panic guarantee | Phase 7 |
| `print_buffer: Vec<String>` on CalcState | Keeps hp41-core I/O-free; hp41-cli drains buffer | Phase 11 |
| Bundle identifier `ch.talent-factory.hp41` (D-02) | Overrides scaffold default `com.tauri.dev` | Phase 13 |
| Tauri v2.11 app-command permissions: TOML files required | Inline app commands don't auto-generate `allow-<cmd>` permissions | Phase 14 |
| One-shot SHIFT frontend-only (`shiftActive`) | Never crosses IPC; ALPHA overrides SHIFT | v2.1 |
| f-prefix one-shot on CLI mirrors GUI (`shift_armed`) | D-25.6 CLI ↔ GUI parity invariant | Phase 25 |
| Hybrid `PendingInput` struct-variants | Collapses 34 logical ops into 2 carriers (FlagPrompt, RegisterPrompt) | Phase 25 |
| `docs/hp41cv-functions.json` as single source of truth | JSON-canonical pipeline; `scripts/docs-matrix` regenerates matrix; bidirectional parity tests | Phase 25 |
| `data-testid="lcd-display"` on `Display14Seg.tsx` | Allowed under SC-4 (hp41-gui/src/ outside boundary); enables WebdriverIO assertion | Phase 27 |
| Coverage gate atomic raise 80 % → 95 % (D-27.2) | Avoid gate-and-test split that masks regressions | Phase 27 |
| WebdriverIO + tauri-driver (not Playwright) for E2E | tauri-driver speaks WebDriver classic; Playwright is CDP/native only | Phase 27 |

### Critical Implementation Traps (carried forward — relevant for v3.0)

- **Every new Op variant must be added to 4 places:** `dispatch()` in `ops/mod.rs` + `execute_op()` in `ops/program.rs` + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. Exhaustive matches fail to compile if any is missed.
- **New CalcState fields need `#[serde(default)]`** for backward compatibility with v1.0/v1.1/v2.0/v2.1/v2.2 save files.
- **SC-4 invariant (no core duplication in hp41-gui):** stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` — `op_display_name` is the only intentional exception. Critical when adding XROM-Modul-Dispatch.
- **No `println!`/`eprintln!` in hp41-core:** route side effects via `print_buffer` or new buffer field.
- **`pending_input` routing block must remain ABOVE modal-opening interceptors** to prevent active dialogs being silently discarded.
- **D-07 (no silent discards) preserved across CLI + GUI:** v3.0 module functions must surface as `GuiError`-toast or modal flow, never silent.
- **HP-copyrighted ROM-image redistribution is permanently excluded** (PROJECT.md:160) — v3.0 Math 1 Emulation is BEHAVIORAL only, based on Owner's Manual documented behavior. No HP ROM bytes in the repo, ever.

### Blockers

None.

### Quick Tasks Completed (historical record)

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260508-y30 | CHS during EEX entry: toggle minus sign in exponent | 2026-05-08 | aa0904b | [260508-y30-eex-chs-exponent-sign-toggle](./quick/260508-y30-eex-chs-exponent-sign-toggle/) |
| 260508-06h | FIX/SCI/ENG digit-count modal via F key (0–9) | 2026-05-08 | 7ff792c | [260508-06h-fix-sci-eng-digit-input](./quick/260508-06h-fix-sci-eng-digit-input/) |
| 260513-v21a | v2.1 Card Reader: WDTA/RDTA/WPRGM/RDPRGM + XEQ-by-name + cards module + PR #9 fixes | 2026-05-13 | 72530dc…f4b3f8b | — (no GSD dir; see MILESTONES.md v2.1) |
| 260513-v21b | v2.1 Keyboard Authenticity: 5-col grid, three-label keys, one-shot SHIFT, run_stop Tauri cmd, stub-error pattern, toast overlay, PR #10 fixes | 2026-05-13 | 8cd2de4…ff56b97 | — (no GSD dir; see MILESTONES.md v2.1) |

---

## Session Continuity

**Last active:** 2026-05-16
**Last action:** Milestone v3.0 started — PROJECT.md updated with Math 1 Pac scope, v2.2 phase directories archived to `.planning/milestones/v2.2-phases/`, MILESTONES.md v2.2 section written, STATE.md reset for v3.0. Research-first selected; 4 parallel agents to run next.
**Next action:** Spawn 4 parallel `gsd-project-researcher` agents (Stack / Features / Architecture / Pitfalls), then `gsd-research-synthesizer`, then user-driven REQUIREMENTS.md scoping, then `gsd-roadmapper` for phase plan starting at Phase 28.

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-16 — Milestone v3.0 Math 1 Pac Emulation started; research-first*
