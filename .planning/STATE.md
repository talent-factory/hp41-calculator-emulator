---
gsd_state_version: 1.0
milestone: v3.0
milestone_name: — Math Pac I Emulation
status: executing
last_updated: "2026-05-17T10:43:41.622Z"
last_activity: 2026-05-17 -- Phase 29 execution started
progress:
  total_phases: 5
  completed_phases: 1
  total_plans: 13
  completed_plans: 10
  percent: 20
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
**Current focus:** Phase 29 — cli-integration
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: 29 (cli-integration) — EXECUTING
Plan: 1 of 3
Status: Executing Phase 29
Last activity: 2026-05-17 -- Phase 29 execution started
Resume from: .planning/phases/29-cli-integration/29-CONTEXT.md

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

Roadmap shipped 2026-05-16 (`.planning/ROADMAP.md`). 5 phases, 25 plans, 110 requirements:

| Phase | Name | Build stage | Plans | Requirements |
|-------|------|-------------|-------|--------------|
| 28 | XROM Framework + Math Pac I Core Ops | `hp41-core` | 10 | 90 (XROM 9 + HYP 6 + CMPLX 17 + POLY 7 + MAT 11 + INTG 8 + SOLV 8 + DIFEQ 5 + FOUR 6 + TRI 5 + TRANS 5) |
| 29 | CLI Integration | `hp41-cli` | 3 | CLI-01..05 |
| 30 | Documentation & ADRs | `docs` | 4 | DOC-01..07 |
| 31 | GUI Integration | `hp41-gui` | 5 | GUI-01..07 |
| 32 | Test Hardening | `tests` | 3 | QUAL-01..08 |

**Phase 28 carries 5 irreversible decisions** (Plan 28-01 research-prep):

1. Op-strategy A vs B — LOCKED A (ADR-001)
2. User-callback re-entrancy policy — LOCKED strict-reject nested (ADR-002)
3. INV-EPSILON value — TBD post-OM-transcription (ADR-003)
4. INTG-threshold formula — TBD post-OM-transcription (ADR-004)
5. JSON-pipeline shape — LOCKED separate file (ADR-005)

**Critical pitfalls flagged**:

- Pitfall 1 (function-name collision): mitigated by xrom_resolve firing LAST in resolver chain + `tests/xrom_shadowing.rs` CI gate
- Pitfall 2 (INTG threshold): tied to `DisplayMode`, OM-cited
- Pitfall 4 (user-callback re-entrancy): `run_loop` (NOT `run_program`) re-entry; nested INTG/SOLVE rejected per XROM-08
- Pitfall 5 (POLY clustering): multiplicity-as-cluster convention documented in divergences doc
- Pitfall 6 (complex branch cuts): `complex_atan2(0,0)→0` first arm; zero-divisor branch BEFORE division
- Pitfall 7 (INV EPSILON): MUST transcribe OM before Plan 28-06
- Pitfall 11 (GUI freeze): cancellation channel + per-64-samples lock release in Phase 31
- Pitfall 14 (cross-platform drift): relative tolerance 1e-7 in Phase 32
- Pitfall 19 (Free42 GPL contamination): per-file header + audit script in Phase 32

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
| Op-strategy A (one Op variant per Math Pac I function) | Preserves 4-exhaustive-match invariant; rejects `Op::XromCall(u16)` table dispatch | Phase 28 ADR-001 |
| User-callback re-entrancy: strict-reject nested | Matches Math Pac I Hardware-Verhalten per OM; simplest invariant | Phase 28 ADR-002 |
| JSON-pipeline: separate `hp41-math1-functions.json` | Zero migration churn on 130 existing v2.2 entries; cleaner test surfaces | Phase 28 ADR-005 |
| `xrom_resolve` fires LAST in resolver chain | Prevents Math Pac I shadowing existing built-in mnemonics (Pitfall 1) | Phase 28 |
| `run_loop` (NOT `run_program`) re-entry for INTG/SOLVE/DIFEQ | Preserves outer program clone; avoids 30 KB × 1000 samples re-clone catastrophe | Phase 28 |

### Critical Implementation Traps (carried forward — relevant for v3.0)

- **Every new Op variant must be added to 4 places:** `dispatch()` in `ops/mod.rs` + `execute_op()` in `ops/program.rs` + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. Exhaustive matches fail to compile if any is missed. Math Pac I adds ~40 variants — 4-way drift increases PR review load (Pitfall 15).
- **New CalcState fields need `#[serde(default)]`** for backward compatibility with v1.0/v1.1/v2.0/v2.1/v2.2 save files. Transient fields (`integ_state`, `solve_state`, `modal_program`, `cancel_requested`) additionally carry `#[serde(skip)]`.
- **SC-4 invariant (no core duplication in hp41-gui):** stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` — `op_display_name` is the only intentional exception. Critical when adding XROM-Modul-Dispatch — Math Pac I math logic MUST land in `hp41-core/src/ops/math1/`.
- **No `println!`/`eprintln!` in hp41-core:** route side effects via `print_buffer` (existing channel; used for Math Pac I prompts) or `event_buffer`.
- **`pending_input` routing block must remain ABOVE modal-opening interceptors** to prevent active dialogs being silently discarded.
- **D-07 (no silent discards) preserved across CLI + GUI:** v3.0 module functions surface as `GuiError`-toast or modal flow, never silent.
- **HP-copyrighted ROM-image redistribution is permanently excluded** (PROJECT.md scope-line) — v3.0 Math Pac I Emulation is BEHAVIORAL only, based on Owner's Manual 00041-90034 (1979). No HP ROM bytes in the repo, ever.
- **Free42 GPL contamination guard** (Pitfall 19): every file in `hp41-core/src/ops/math1/` carries a per-file header comment disclaiming Free42 source copying; `scripts/check-free42-contamination.sh` CI gate.

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
**Last action:** Phase 28 discuss-phase complete — `28-CONTEXT.md` + `28-DISCUSSION-LOG.md` written. 4 gray areas explored (ComplexStack location, Modal-prompt channel, Hyperbolics UX, Cancellation timing). 9 new decisions D-28.1..D-28.9 captured: overlay X/Y/Z/T complex-stack with `complex_mode: bool` (D-28.1, D-28.2); new derived `XEQ "REAL"` requirement (D-28.3); dedicated `modal_prompt: Option<String>` field overrides REQUIREMENTS XROM-09 (D-28.4); R/S key submits modal numeric input (D-28.5); hyperbolics XEQ-only — no dedicated keys (D-28.6); cancellation plumbing in Phase 28, wiring in Phase 31 (D-28.7, D-28.8); new `HpError::Canceled` variant (D-28.9).
**Next action:** `/gsd-plan-phase 28` to decompose Phase 28 into 10 plan files (28-01 framework + ADRs + research-prep for OM transcription of INV-EPSILON / INTG-threshold, 28-02 hyperbolics, 28-03 complex stack arith, 28-04 complex functions, 28-05 POLY, 28-06 MATRIX, 28-07 INTG, 28-08 SOLVE, 28-09 DIFEQ, 28-10 FOUR+triangles+TRANS).

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-16 — Phase 28 context gathered (9 new decisions); awaiting `/gsd-plan-phase 28`*
