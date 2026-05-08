---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: CLI Feature Completeness
current_phase: 9
current_plan: Wave 2 — executing 09-03
status: in_progress
last_updated: "2026-05-08T14:30:00.000Z"
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 3
  completed_plans: 0
  percent: 0
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-08)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.
**Shipped:** v1.0 CLI (2026-05-08)
**Current focus:** Phase 9 — Infrastructure & EEX Fix
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: 9 of 12 (Infrastructure & EEX Fix)
Plan: Not started
Status: Ready to execute (3 plans)
Last activity: 2026-05-08 — Phase 9 planned (3 plans in 2 waves)

Progress: [░░░░░░░░░░] 0%

---

## Performance Metrics (v1.0 Shipped Values)

| Metric | Target | Achieved |
|--------|--------|---------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms (M1) — 228× under gate |
| Key-press latency (median) | ≤ 50 ms | ~65 ns/op |
| `hp41-core` test coverage | ≥ 80% | 94.87% |
| Numerical accuracy (500-case) | ≥ 98% | 99% (495/500) |
| Panics in `hp41-core` | 0 | 0 — enforced by `#![deny(clippy::unwrap_used)]` |
| CI platforms | Win/macOS/Ubuntu | All green |

---

## Accumulated Context

### Key Decisions

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

### Critical Implementation Traps (v1.1)

- `test_flush_trailing_e_without_exponent_returns_err` must be INVERTED — it currently asserts the wrong (discarding) behavior; Phase 9 corrects this
- Every new Op variant must be added to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs`
- New CalcState fields need `#[serde(default)]` for backward compatibility with v1.0 save files
- STO arithmetic core (`op_sto_arith`) is already implemented in hp41-core — Phase 10 is TUI wiring only
- Phase 10 has no hp41-core changes; all work is in hp41-cli (modal state machine in app.rs)

### Blockers

None.

---

## Session Continuity

**Last active:** 2026-05-08
**Last action:** Phase 9 planned — 3 plans (09-01 MSRV/CI, 09-02 flush_entry_buf core, 09-03 EEX guards + display) in 2 waves
**Next action:** `/gsd-execute-phase 9`

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-08 after v1.1 roadmap creation*
