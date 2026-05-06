---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
current_phase: 2
current_plan: Gap closure needed (2 gaps in format.rs)
status: gaps_found
last_updated: "2026-05-06T17:00:00.000Z"
progress:
  total_phases: 7
  completed_phases: 1
  total_plans: 11
  completed_plans: 10
  percent: 57
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.
**Target release:** 2026-09-05 (v1.0 CLI)
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

**Current phase:** 2 (Core Math) — PLANNED
**Current plan:** Ready to execute (7 plans, 4 waves)
**Status:** Ready to execute

```
Progress: [·······] 0%

Phase 1: Foundation          [ ] Not started
Phase 2: Core Math           [ ] Not started
Phase 3: Programming Engine  [ ] Not started
Phase 4: TUI & Input         [ ] Not started
Phase 5: Persistence & UX    [ ] Not started
Phase 6: Science & Engineering [ ] Not started
Phase 7: Hardening           [ ] Not started
```

---

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Cold-start latency | ≤ 0.5 s | Unmeasured |
| Key-press latency (median) | ≤ 50 ms | Unmeasured |
| `hp41-core` test coverage | ≥ 80% | 0% |
| Numerical accuracy (500-case suite) | ≥ 98% | 0 cases |
| Crash-free sessions | ≥ 99.5% | Unverified |

---

## Accumulated Context

### Key Decisions

| Decision | Rationale | Phase |
|----------|-----------|-------|
| BCD vs f64 | Must resolve before any register code; `rust_decimal` or custom BCD struct | Phase 1 |
| Stack-lift as boolean flag | `lift_enabled: bool` in `Stack`; every op declares Enable/Disable/Neutral | Phase 1 |
| `CalcState` as single source of truth | One `&mut CalcState` through all ops; no global mutable state | Phase 1 |
| Instruction enum, not dyn Trait | HP-41 instruction set is fixed/closed; enum is faster, serializable, exhaustive | Phase 3 |
| No async in hp41-core | Synchronous event loop; tokio only in hp41-cli if needed for autosave timer | All |
| ratatui 0.30 + crossterm | Only backend with Windows 10+ support; crossterm fires Key::Press + Key::Release on Windows (filter!) | Phase 4 |
| serde_json for persistence | Human-readable, shareable; users can diff/git state files | Phase 5 |

### Critical Implementation Traps

- ISG/DSE counter fields must be extracted by string-splitting at the decimal point — never with `floor()`/`fmod()` on f64
- Windows crossterm fires both `KeyEventKind::Press` and `KeyEventKind::Release` — filter to Press only or every operation executes twice
- Always use `ratatui::init()` (not `Terminal::new()`) to install the panic hook — without it, any unhandled panic leaves the terminal in raw mode
- Use `event::poll(timeout)` not `event::read()` to support the 30-second auto-save timer without blocking redraws

### Open Questions

- [ ] BCD vs f64 with 10-digit rounding: evaluate `rust_decimal` vs a custom BCD struct for Phase 1 implementation decision
- [ ] Program execution threading: short programs can run synchronously; long-running programs may need an `AtomicBool` interrupt flag and Tokio task to avoid blocking redraws (architecture note: keep this out of `hp41-core`)
- [ ] ISG/DSE format interpretation: `CCCCC.FFFDD` (5-digit current, 3-digit final, 2-digit step increment) — verify exact field widths from HP-41 Owner's Handbook before Phase 3

### Todos

- [ ] Create Cargo workspace with `hp41-core` and `hp41-cli` crates (Phase 1)
- [ ] Write license audit checklist before public release (before v1.0 tag)
- [ ] Design keyboard mapping table for 80+ HP-41 functions → PC keys (Phase 4)
- [ ] Curate 10+ bundled sample programs from public domain HP Solutions books (Phase 5)

### Blockers

None.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260506-a1g | Add .gitignore for Rust/Cargo workspace | 2026-05-06 | f603a1b | [260506-a1g-add-gitignore](.planning/quick/260506-a1g-add-gitignore/) |

---

## Session Continuity

**Last active:** 2026-05-06
**Last action:** Quick task 260506-a1g — added .gitignore for Rust/Cargo workspace
**Next action:** Run `/gsd-plan-phase 1` to plan Phase 1: Foundation

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-06 after roadmap creation*
