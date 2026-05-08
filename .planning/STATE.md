---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: planning
current_phase: 9
current_plan: Not started
status: Milestone v1.0 complete — ready for v1.1 planning
last_updated: "2026-05-08T10:00:00.000Z"
progress:
  total_phases: 8
  completed_phases: 8
  total_plans: 45
  completed_plans: 45
  percent: 100
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-08)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.
**Shipped:** v1.0 CLI (2026-05-08)
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

**Milestone v1.0:** Complete ✅
**Current focus:** Planning v1.1

```
Progress: [████████████████] 100% (all v1.0 phases complete)

Phase 1: Foundation          [x] Complete (2026-05-06)
Phase 2: Core Math           [x] Complete (2026-05-07)
Phase 3: Programming Engine  [x] Complete (2026-05-07)
Phase 4: TUI & Input         [x] Complete (2026-05-07)
Phase 5: Persistence & UX    [x] Complete (2026-05-07)
Phase 6: Science & Engineering [x] Complete (2026-05-07)
Phase 7: Hardening           [x] Complete (2026-05-07)
Phase 8: Tech Debt Cleanup   [x] Complete (2026-05-08)
```

---

## Performance Metrics (v1.0 Shipped Values)

| Metric | Target | Achieved |
|--------|--------|---------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms (M1) — 228× under gate |
| Key-press latency (median) | ≤ 50 ms | ~65 ns/op |
| `hp41-core` test coverage | ≥ 80% | 94.87% |
| Numerical accuracy (500-case) | ≥ 98% | 99% (495/500) |
| Panics in `hp41-core` | 0 | 0 — enforced by `#![deny(clippy::unwrap_used)]` |
| CI platforms | Win/macOS/Ubuntu | All green (run #25539003811) |

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

### Critical Implementation Traps

- ISG/DSE counter fields must be extracted by string-splitting at the decimal point — never with `floor()`/`fmod()` on f64
- Windows crossterm fires both `KeyEventKind::Press` and `KeyEventKind::Release` — filter to Press only or every operation executes twice
- Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook
- Use `event::poll(timeout)` not `event::read()` to support the 30-second auto-save timer
- `cargo llvm-cov` accumulates stale `.profraw` data in worktree runs — always `cargo llvm-cov clean --workspace` before measuring coverage

---

## Deferred Items

Items acknowledged at v1.0 milestone close (2026-05-08):

| Category | Item | Status |
|----------|------|--------|
| keyboard | STO arithmetic modals (STO+/-/×/÷) | Deferred to v1.1 |
| behavior | EEX trailing-e-without-exponent discards silently | Documented with test; v1.1 |

---

## Blockers

None.

---

## Session Continuity

**Last active:** 2026-05-08
**Last action:** v1.0 milestone complete — all phases done, REQUIREMENTS.md archived, tagged v1.0
**Next action:** `/gsd-new-milestone` to plan v1.1

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-08 after v1.0 milestone completion*
