# Roadmap — v3.0 Math 1 Pac Emulation

**Status:** Skeleton — wird in Workflow-Step 10 (gsd-roadmapper) mit Phasen, Goals, Requirements-Mapping und Success Criteria gefüllt.

**Milestone goal:** Behavioral Emulation des HP-41 Math 1 Pacs als erstes XROM-Modul.

**Phase numbering:** continued from v2.2 (last shipped Phase 27). v3.0 phases start at **Phase 28**.

---

## Phases (to be defined by roadmapper)

*Pending REQUIREMENTS.md approval. Roadmapper will derive 2–5 success criteria per phase from the approved requirements and ensure 100 % coverage.*

| # | Name | Goal | Requirements | Success Criteria | Build Stage |
|---|------|------|--------------|------------------|-------------|
| 28 | *(TBD)* | | | | |

---

## Cross-cutting concerns (carried from v2.x)

- **SC-4 invariant:** no calculator logic duplication in `hp41-gui` (stricter grep for `op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)`)
- **No HP-copyrighted ROM bytes** — behavioral emulation only
- **`#![deny(clippy::unwrap_used)]`** continues to apply in `hp41-core`
- **`#[serde(default)]`** on all new `CalcState` fields for v1.x–v2.x save-file backward compatibility
- **`pending_input` routing above modal interceptors** — D-07 (no silent discards)
- **CLI ↔ GUI parity** (D-25.6 invariant) — every new module function reachable in both surfaces

---

## v2.x ROADMAP archives

- v1.0: `milestones/v1.0-ROADMAP.md`
- v1.1: `milestones/v1.1-ROADMAP.md`
- v2.0: `milestones/v2.0-ROADMAP.md`
- v2.2: `milestones/v2.2-ROADMAP.md`

---

*Last updated: 2026-05-16 — skeleton; awaiting requirements approval.*
