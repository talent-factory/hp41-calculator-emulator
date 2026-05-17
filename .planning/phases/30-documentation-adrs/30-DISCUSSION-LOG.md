# Phase 30: Documentation & ADRs — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `30-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-05-17
**Phase:** 30-documentation-adrs
**Areas discussed:** docs-matrix two-input shape, Divergence doc scope/structure, ADR template depth, v3.0 additions block timing

---

## docs-matrix two-input shape

### How should scripts/docs-matrix consume the second JSON file?

| Option | Description | Selected |
|--------|-------------|----------|
| Call binary twice from justfile | Existing 1-in/1-out signature unchanged; second invocation added to the docs-matrix and docs-matrix-check recipes. Smallest blast radius, zero risk to the v2.2 hp41cv path, easy to add a third JSON in v3.1. | ✓ |
| Extend CLI to N pairs | One invocation: `docs-matrix in1 out1 in2 out2 ...`. Half the cargo-startup latency in CI; adds CLI-parsing surface to the script. | |
| Hardcode both paths in main.rs | Binary loses CLI args entirely; knows about both JSONs by name. Zero argument-parsing surface; binary now coupled to filenames. | |

**User's choice:** Call binary twice from justfile (Recommended)
**Captured as:** D-30.1
**Notes:** User selected the recommended option with the preview showing the explicit two-line justfile recipe.

---

### Matrix output — one combined file or one per JSON?

| Option | Description | Selected |
|--------|-------------|----------|
| Two separate matrix files | hp41cv-function-matrix.md (130 entries, untouched) and hp41-math1-function-matrix.md (~55 entries, new). Each matrix derives from one JSON and lives next to it. | ✓ |
| One combined matrix file | Single matrix.md with sections per pac. Easier scroll for the reader at the cost of touching the v2.2 file. | |
| Two files + a top-level index | Two matrix files plus a new docs/function-matrix-index.md that links to both. Adds one extra surface to maintain. | |

**User's choice:** Two separate matrix files (Recommended)
**Captured as:** D-30.2

---

### Math Pac I matrix — render an XROM column?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, add XROM column | New column right after Display: shows "Math 1 / 7-N" (module name / module_id-function_id) for all ~55 Math Pac I rows. Drift-catch for missing xrom blocks. | ✓ |
| No XROM column | Same 7-column shape as hp41cv matrix. Pure consistency at the cost of hiding the XROM identity. | |
| Combined Op column | Embed XROM identity inside Op column ("Sinh (XROM 7-1)"). Saves one column; mixes two semantic axes. | |

**User's choice:** Yes, add XROM column (Recommended)
**Captured as:** D-30.3

---

## Divergence doc scope/structure

### What lands in the expanded hp41-math1-divergences.md?

| Option | Description | Selected |
|--------|-------------|----------|
| OM-divergences + emulator extensions + behavioral policies | Three categories: OM divergences (POLY clustering, INTG threshold, FACT integer-only), Emulator Extensions (XEQ "REAL"), Behavioral Policies (strict-reject-nested, modal R/S submit). Every entry carries OM page or ADR link. | ✓ |
| OM-divergences only | Narrowest scope: only numerical mismatches with OM-quoted examples. Extensions and policies documented elsewhere. | |
| Three sections by domain | Same content but organized by Math Pac I program (Hyperbolics / Complex / POLY / MATRIX / etc.). Cross-cutting policies would duplicate under multiple sections. | |

**User's choice:** OM-divergences + emulator extensions + behavioral policies (Recommended)
**Captured as:** D-30.4

---

### Per-entry structure for divergence catalog?

| Option | Description | Selected |
|--------|-------------|----------|
| Numbered entries with 5 fixed fields | Each entry: ID (D-30-NN), Title, OM Citation, Our Behavior, OM Behavior, Rationale, See. Predictable shape supports grep tooling; matches Pitfall 18 provenance discipline. | ✓ |
| Free-form prose per program | Few paragraphs of narrative per program. Reads better top-to-bottom; provenance citations get buried. | |
| Three-column comparison tables | Per program: a table with columns "Behavior | OM (1979) | Emulator | Status". Compact; doesn't carry rationale prose well. | |

**User's choice:** Numbered entries with 5 fixed fields (Recommended)
**Captured as:** D-30.5

---

## ADR template depth

### ADR template depth for ADR-001, ADR-002, ADR-005?

| Option | Description | Selected |
|--------|-------------|----------|
| Mirror existing ADR-003/004 long-form | Same template: full Context, Decision, Consequences, Alternatives Considered, Footnotes. ~6–7K each. Consistent reading experience. | ✓ |
| Shorter template (Context + Decision + Why) | Trim alternatives-considered narrative since CONTEXT.md carries the rejected options. ~2K each. Risks an ADR that doesn't stand alone. | |
| Per-ADR pick | ADR-001 + ADR-002 long-form (architecture-affecting); ADR-005 shorter. Mixed depth signals "this one mattered more" at the cost of inconsistency. | |

**User's choice:** Mirror existing ADR-003/004 long-form (Recommended)
**Captured as:** D-30.6

---

### Where do rejected-alternatives sections sourced from?

| Option | Description | Selected |
|--------|-------------|----------|
| Quote CONTEXT.md options verbatim + add citations | Phase 28 CONTEXT.md already lists rejected Option B for each decision; ADRs quote/lightly-edit those sections, add OM/community citations (Pitfall 18). Zero revisionism risk. | ✓ |
| Rewrite for ADR audience | Treat the ADR as standalone for a future cold reader. More work; higher drift risk. | |
| CONTEXT.md link + one-paragraph summary | Compact; ADR cannot be read offline. | |

**User's choice:** Quote CONTEXT.md options verbatim + add citations (Recommended)
**Captured as:** D-30.7

---

## v3.0 additions block timing

### When does the v3.0 additions block land in PROJECT.md / CLAUDE.md?

| Option | Description | Selected |
|--------|-------------|----------|
| Add in-progress block now in Phase 30 | Matches v2.2 precedent (incremental population). Phase 28 + 29 + 30 subsections fully populated; Phase 31 + 32 stub headers with `(in progress)` markers. | ✓ |
| Defer entire block to Phase 32 ship | Mirror v2.2 ship-time addition pattern. Cleanest semantics; most risk of decision drift between Phase 30 and 32. | |
| Stub block with TBD metrics | Add minimal block now with all five subsections present but only structural headers + TBD where v3.0 metrics belong. | |

**User's choice:** Add in-progress block now in Phase 30 (Recommended)
**Captured as:** D-30.8

---

### README v3.0 soft-claim wording — conservative or assertive?

| Option | Description | Selected |
|--------|-------------|----------|
| Conservative soft-claim | "Math Pac I behavioral emulation included (10 top-level programs, ~55 XEQ entry points, documented divergences)" + link to matrix. Hard claim deferred to Phase 32 conditional on coverage gate ≥ 95% per QUAL-01. | ✓ |
| Hard claim now | "Feature-complete Math Pac I behavioral emulation." Skips Phase 32 coverage condition. Inconsistent with v2.2 gating. | |
| Minimal one-liner + link only | "v3.0 adds Math Pac I behavioral emulation — see docs/hp41-math1-function-matrix.md." Loses the 10/55 numbers. | |

**User's choice:** Conservative soft-claim (Recommended)
**Captured as:** D-30.9

---

## Claude's Discretion

The following items were captured in `30-CONTEXT.md` under "Claude's Discretion" rather than asked of the user:

- Exact placement of the `v3.0 additions` block within `PROJECT.md` (recommendation: concise milestone lines in PROJECT.md; full block in CLAUDE.md only).
- ADR-001/002/005 owner/date metadata (recommendation: lock date = 2026-05-16, the date Plan 28-01 research-prep landed; owner = "Plan 28-01 Task X").
- Divergence catalog entry order within each bucket (recommendation: chronological by Phase 28 decision sequence).
- Whether `just docs-matrix-check` becomes a CI gate in Phase 30 (recommendation: implicitly extended via the existing CI step; no new workflow file changes).
- MoHPC / Mike Sebastian forensic URL citations across the three new ADRs (recommendation: cite where they strengthen the argument; no fabricated URLs).
- XROM column header label in the matrix file (recommendation: `XROM`).

---

## Deferred Ideas

The following items came up during the discussion (or were flagged from prior CONTEXTs) and are noted for future phases:

- `scripts/check-free42-contamination.sh` audit script + per-file header CI enforcement — Phase 32 / QUAL-05.
- Free42-contamination guard CI workflow step — Phase 32 / QUAL-05.
- Phase 31 GUI subsection content inside the `v3.0 additions` block — Phase 31 ship.
- Phase 32 metrics (coverage, numerical accuracy, E2E smoke) inside the `v3.0 additions` block — Phase 32 ship.
- Hard claim "Feature-complete Math Pac I behavioral emulation" in README and CLAUDE.md — Phase 32 conditional on coverage gate ≥ 95% per QUAL-01.
- Future v3.1+ pac documentation pattern (Stat Pac, Time Pac, etc.) — pattern serves as template when the second XROM module lands; not part of v3.0.
- Index file linking all matrix files (`docs/function-matrix-index.md`) — rejected in D-30.2 as redundant for a two-file world; may re-emerge as v3.x pacs accumulate.
- `docs/hp41-math1-function-matrix.md` schema documentation (why the matrix has an extra XROM column) — not documented anywhere beyond `30-CONTEXT.md`; may need a dedicated doc if it becomes a friction point.
