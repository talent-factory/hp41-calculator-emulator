---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
current_phase: 5
current_plan: Not started
status: planning
last_updated: "2026-05-07T11:41:11.096Z"
progress:
  total_phases: 7
  completed_phases: 4
  total_plans: 22
  completed_plans: 22
  percent: 100
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

**Current phase:** 5
**Current plan:** Not started
**Status:** Ready to plan

```
Progress: [███████████··] 75%

Phase 1: Foundation          [x] Complete (2026-05-06)
Phase 2: Core Math           [x] Complete (2026-05-07)
Phase 3: Programming Engine  [x] Complete (2026-05-07)
Phase 4: TUI & Input         [.] In progress (1/5 plans done)
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
| `hp41-core` test coverage | ≥ 80% | 81.62% (GATE PASSED) |
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
| PushNum in execute_op enables lift | Without LiftEffect::Enable, sequential PushNums in a program overwrite X — critical for correct stack behavior | Phase 3 |
| ISG body-before-check loop semantics | With Lbl/body/ISG/GTO structure, body runs on same pass as skipping ISG — 5 iterations with R00=1.00500 (current=1, final=5) | Phase 3 |
| ISG/DSE discard bool in interactive dispatch | op_isg/op_dse return Result<bool>; dispatch() wraps with .map(|_| ()) — skip signal only meaningful in run_loop, not interactive keypress context | Phase 3 |
| No async in hp41-core | Synchronous event loop; tokio only in hp41-cli if needed for autosave timer | All |
| ratatui 0.30 + crossterm | Only backend with Windows 10+ support; crossterm fires Key::Press + Key::Release on Windows (filter!) | Phase 4 |
| ratatui::init() returns DefaultTerminal | RestoreTerminalGuard does not exist in 0.30; ratatui::restore() must be called explicitly after run() | Phase 4 |
| draw(&self) immutable in App | Avoids borrow conflict with &mut terminal inside terminal.draw() — required by Rust borrow checker | Phase 4 |
| Digit entry appends to entry_buf directly | dispatch() auto-flushes on next non-digit op; calling dispatch per digit would push each as a separate PushNum | Phase 4 |
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

**Last active:** 2026-05-07
**Last action:** Completed plan 04-01 — ratatui 0.30 + crossterm 0.29 + clap 4.x added; App struct with poll-based event loop; module stubs compile; cargo check -p hp41-cli passes with zero errors
**Next action:** Phase 4, Plan 04-02 — ui.rs full widget layout (stack panel, display, annunciators, key reference)

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-06 after roadmap creation*
