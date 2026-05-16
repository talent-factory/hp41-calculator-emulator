# Requirements — v3.0 Math 1 Pac Emulation

**Status:** Skeleton — wird in Workflow-Step 9 (define-requirements) nach Abschluss der Research-Phase mit konkreten REQ-IDs gefüllt.

**Milestone goal:** Behavioral Emulation des HP-41 Math 1 Pacs als erstes XROM-Modul — vollständige Funktionsbibliothek (Matrix-Ops, komplexe Zahlen, Polynom-/Vektor-Ops, numerische Integration und Solver), nutzbar in CLI + GUI über das v2.x-Built-in-Pattern hinaus, ohne HP-copyrighted ROM-Image-Redistribution.

**Scope boundary:** v3.0 ist Math 1 Pac only. Stat 1 → v3.1; Time + Advantage → v3.2 / v3.3.

---

## Categories (preliminary — will be refined post-research)

### XROM Module Framework
- Slot-Management (4 module slots on HP-41C/CV)
- XROM-Nummerierung (Math 1 reserved ID — TBD via research)
- Funktions-Dispatch (Op-Variante oder neuer Op::XromCall enum)
- Module-Loading-API (statisch eingebaut für v3.0; .mod-File-Support → spätere Milestones)

### Math 1 Functions (preliminary list — Owner's Manual to confirm)
- Matrix-Ops: `M+`, `M-`, `MAT*`, `INV`, `TRANS`, `DET`, `IDN`, ...
- Komplexe Zahlen: `CADD`, `CSUB`, `CMUL`, `CDIV`, `CABS`, `CARG`, `CCHS`, `CCONJ`, ...
- Polynom-Solver: `PROOT`
- Numerische Integration: `INTEG`
- Numerische Solver: `SOLVE`
- Vektor-Operationen: `V+`, `V-`, `VDOT`
- Spezialfunktionen (TBD via research)

### CLI Integration
- Tastenbelegungen für Math-1-Funktionen (XEQ-by-name fallback + neue Modale)
- `?` help-overlay extension (JSON-Pipeline um Math 1 erweitern)

### GUI Integration
- `key_map.rs::resolve` arms für Math-1-IDs
- `KEY_DEFS` updates
- Modal-Routing analog v2.2-Pattern

### Documentation
- `docs/hp41-math1-functions.json` (analog `hp41cv-functions.json`)
- `docs/hp41-math1-function-matrix.md` (generiert via erweitertes `scripts/docs-matrix`)

### Quality Gates
- `hp41-core` Coverage ≥ 95 % (gate aus v2.2 beibehalten)
- Numerische Accuracy Suite erweitert um Math-1-Cases (Matrix-Operationen, komplexe Zahlen, Polynom-Roots)
- GUI E2E Smoke erweitert um einen Math-1-Flow

---

## Out of Scope (v3.0 — explicit exclusions)

- **HP-copyrighted ROM-image redistribution** — permanent exclusion (PROJECT.md:160). v3.0 ist Behavioral Emulation.
- **Stat 1 / Time / Advantage Pacs** — Scope von v3.1+.
- **Custom user modules** — `.mod`-File-Support deferred zu späterer Milestone.
- **HP-IL peripherals / Wand / barcode reader** — permanent exclusion.

---

## Traceability (filled by roadmapper)

| REQ-ID | Phase | Status |
|--------|-------|--------|
| *(none — to be populated)* | | |

---

*Last updated: 2026-05-16 — skeleton; awaiting research synthesis.*
