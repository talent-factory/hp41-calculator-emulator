# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator
**Current milestone:** v1.1 (planning)

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, shipped 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- 📋 **v1.1** — STO arithmetic modals, EEX polish, FR-17 print emulation (planned)
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

### 📋 v1.1 (Planned)

- [ ] Phase 9: STO arithmetic keyboard modals + EEX exponent entry polish
- [ ] Phase 10: Print emulation (FR-17: PRX/PRA/PRSTK to console/text)

---

## Progress Table

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1. Foundation | v1.0 | 4/4 | ✅ Complete | 2026-05-06 |
| 2. Core Math | v1.0 | 7/7 | ✅ Complete | 2026-05-07 |
| 3. Programming Engine | v1.0 | 6/6 | ✅ Complete | 2026-05-07 |
| 4. TUI & Input | v1.0 | 5/5 | ✅ Complete | 2026-05-07 |
| 5. Persistence & UX | v1.0 | 11/11 | ✅ Complete | 2026-05-07 |
| 6. Science & Engineering | v1.0 | 3/3 | ✅ Complete | 2026-05-07 |
| 7. Hardening | v1.0 | 6/6 | ✅ Complete | 2026-05-07 |
| 8. Tech Debt Cleanup | v1.0 | 3/3 | ✅ Complete | 2026-05-08 |
| 9. STO + EEX Polish | v1.1 | 0/- | 📋 Planned | — |
| 10. Print Emulation | v1.1 | 0/- | 📋 Planned | — |
