---
phase: 30-documentation-adrs
plan: "02"
subsystem: docs
tags:
  - docs
  - adr
  - divergence-catalog
  - decision-archaeology
dependency_graph:
  requires:
    - 30-01 (docs-matrix extension — reads hp41-math1-functions.json that ADR-005 documents)
    - 28-01 (Phase 28 CONTEXT.md decisions C-28.1/C-28.2/C-28.3 quoted verbatim here)
  provides:
    - docs/hp41-math1-divergences.md (7-entry three-bucket catalog D-30-01..D-30-07)
    - docs/adr/v3.0-001-op-strategy.md
    - docs/adr/v3.0-002-user-callback-policy.md
    - docs/adr/v3.0-005-json-pipeline.md
  affects:
    - Plan 30-03 (README/CLAUDE.md v3.0 additions block links to these docs)
    - Phase 32 QUAL-05 (scripts/check-free42-contamination.sh greps ADR-002 disclaim)
tech_stack:
  added: []
  patterns:
    - D-30.5 five-field divergence entry shape (OM citation / Our behavior / OM behavior / Rationale / See)
    - D-30.6 long-form ADR template (Context / Decision / Consequences / Alternatives Considered / Footnotes)
    - D-30.7 verbatim CONTEXT.md quote discipline in Alternatives Considered
key_files:
  created:
    - docs/hp41-math1-divergences.md
    - docs/adr/v3.0-001-op-strategy.md
    - docs/adr/v3.0-002-user-callback-policy.md
    - docs/adr/v3.0-005-json-pipeline.md
  modified: []
decisions:
  - "DOC-04: Three-bucket divergence catalog shape with D-30-NN IDs and five-field entries locked in execution"
  - "DOC-07: Long-form ADR template (Context/Decision/Consequences/Alternatives/Footnotes) applied to ADR-001/002/005"
  - "Pitfall 19 compliance: Free42 disclaim sentence appears 4 times in ADR-002 (Decision + Footnotes [^4] twice)"
metrics:
  duration_minutes: 9
  completed_date: "2026-05-17"
  tasks_completed: 3
  tasks_total: 3
  files_created: 4
  files_modified: 0
---

# Phase 30 Plan 02: Divergence Catalog + ADR-001/002/005 Summary

**One-liner:** Three-bucket numbered divergence catalog (7 entries, 329 lines) + three long-form ADRs (ADR-001/002/005) documenting Math Pac I Op-strategy, user-callback policy, and JSON-pipeline shape with verbatim Phase 28 CONTEXT.md quotes and Free42 Pitfall 19 compliance.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Expand divergence catalog to three-bucket numbered format | 5b4d55f | `docs/hp41-math1-divergences.md` |
| 2 | Write ADR-001 (Op-strategy A) and ADR-005 (JSON-pipeline shape) | c392f46 | `docs/adr/v3.0-001-op-strategy.md`, `docs/adr/v3.0-005-json-pipeline.md` |
| 3 | Write ADR-002 (user-callback strict-reject) with Free42 disclaim | e05e7cf | `docs/adr/v3.0-002-user-callback-policy.md` |

## Artifact Summary

### `docs/hp41-math1-divergences.md` — 329 lines (expanded from 78-line seed)

Three-bucket numbered catalog with 7 entries carrying D-30-NN IDs (D-30-01 through D-30-07).

**Bucket 1 — OM Divergences (4 entries):**
- D-30-01: User-Program Scratch Register Clobber During INTG/SOLVE/DIFEQ
- D-30-02: POLY Complex-Root Multiplicity Rendered as Cluster
- D-30-03: INTG Convergence Threshold Tied to DisplayMode
- D-30-04: FACT Integer-Only — No GAMMA Extension

**Bucket 2 — Emulator Extensions (1 entry):**
- D-30-05: XEQ "REAL" — Deactivates `complex_mode`

**Bucket 3 — Behavioral Policies (2 entries):**
- D-30-06: Strict-Reject Nested INTG/SOLVE/DIFEQ
- D-30-07: Modal R/S Submits Numeric Input

Shape compliance: 40 five-field hits (5 fields × 7 entries = 35 minimum; 40 actual due to
multi-paragraph field bodies). Every entry carries HP 00041-90034 citation or explicit
`"N/A — emulator extension"` marker. D-30-06 See field links to ADR-002.

### `docs/adr/v3.0-001-op-strategy.md` — 211 lines / 11,287 bytes

Documents the lock of Op-strategy A (one `Op` variant per Math Pac I function).
Sections: Context → Decision → Consequences → Alternatives Considered → Footnotes.
Quotes 28-CONTEXT.md C-28.1 verbatim in Alternatives Considered per D-30.7.
Cites HP 00041-90034 (3 occurrences). Lock date: 2026-05-16.

### `docs/adr/v3.0-002-user-callback-policy.md` — 259 lines / 14,193 bytes

Documents the strict-reject nested INTG/SOLVE/DIFEQ policy.
Sections: Context → Decision → Consequences → Alternatives Considered → Footnotes.
Quotes 28-CONTEXT.md C-28.2 verbatim in Alternatives Considered per D-30.7.

**Pitfall 19 compliance:**
- Free42 mentioned 19 times (case-insensitive grep)
- Verbatim disclaim sentence "Algorithm independently re-derived from HP Math Pac I Owner's
  Manual 00041-90034 (1979); Free42 source consulted only as sanity-check oracle, not copied."
  appears 4 times: once in Context section, once in Decision section, once in Decision
  section (as standalone paragraph), once in Footnote [^4].
- Grep verification: `grep -c "consulted only as sanity-check oracle, not copied" docs/adr/v3.0-002-user-callback-policy.md` = **4**

Cites HP 00041-90034 (4 occurrences). Lock date: 2026-05-16.

### `docs/adr/v3.0-005-json-pipeline.md` — 213 lines / 11,936 bytes

Documents the separate `hp41-math1-functions.json` sibling file decision.
Sections: Context → Decision → Consequences → Alternatives Considered → Footnotes.
Quotes 28-CONTEXT.md C-28.3 verbatim in Alternatives Considered per D-30.7.
Cites D-25.16 / D-25.17 / D-25.18 v2.2 JSON-canonical pipeline precedent.
Cites HP 00041-90034 (3 occurrences). Lock date: 2026-05-16.

## Verification

### Owner's Manual citation coverage

```
grep -c "HP 00041-90034" docs/adr/v3.0-001-op-strategy.md  = 3
grep -c "HP 00041-90034" docs/adr/v3.0-002-user-callback-policy.md  = 4
grep -c "HP 00041-90034" docs/adr/v3.0-005-json-pipeline.md  = 3
Total ADR OM citations: 10 (≥3 requirement: PASSED)
```

```
grep -E "HP 00041-90034|N/A — emulator extension" docs/hp41-math1-divergences.md | wc -l = 16
(≥6 requirement: PASSED)
```

### Free42 disclaim sentence

```
grep -c "consulted only as sanity-check oracle, not copied" \
  docs/adr/v3.0-002-user-callback-policy.md = 4
(≥2 in Decision + Footnotes requirement: PASSED)
```

### Phase 28 CONTEXT.md verbatim quotes

All three ADRs carry verbatim blockquotes (`>` prefix lines) from 28-CONTEXT.md:
- ADR-001: C-28.1 (lines 42–43) — Op-strategy A vs B rejected option
- ADR-002: C-28.2 (lines 44–45) — user-callback re-entrancy strict-reject
- ADR-005: C-28.3 (lines 46–47) — JSON-pipeline separate-file shape

### Lock date compliance

All three ADRs carry `**Status:** Locked 2026-05-16` (Plan 28-01 ship date, not today's
Phase 30 ship date 2026-05-17).

### SC-4 invariant

Zero source file changes:
```
git diff --name-only HEAD~3 HEAD -- 'hp41-core/src/' 'hp41-cli/src/' 'hp41-gui/src/'
'hp41-gui/src-tauri/src/' = 0 files
```

### MoHPC / Mike Sebastian URLs

No clean dedicated URLs were found at write time for any of the three ADRs:
- ADR-001: MoHPC threads on XROM table-dispatch — no clean thread found; OM citation [^1]
  and 4-match-invariant precedent [^3] provide sufficient grounding (Footnote [^4] documents
  the "no clean source" outcome per Pitfall 18).
- ADR-002: MoHPC threads on nested-INTG behavior — no clean thread found; OM warning + strict-
  reject rationale sufficient; Footnote [^5] documents the suggested search term for future
  reference.
- ADR-005: MoHPC threads on JSON pipeline — not applicable (infrastructure decision, no
  community hardware discussion exists).

All three ADRs rely on OM page citations alone, which satisfies Pitfall 18's "OM page-and-
example OR MoHPC URL OR Mike Sebastian forensic URL" requirement.

## Deviations from Plan

None — plan executed exactly as written. All three tasks completed in sequence; all
acceptance criteria met.

## Known Stubs

None. All seven divergence entries and all three ADRs contain substantive content.
No `TODO`, `FIXME`, or placeholder text in the created files.

## Self-Check: PASSED

Files exist:
- [x] `docs/hp41-math1-divergences.md` — 329 lines ≥ 200
- [x] `docs/adr/v3.0-001-op-strategy.md` — 211 lines / 11,287 bytes
- [x] `docs/adr/v3.0-002-user-callback-policy.md` — 259 lines / 14,193 bytes
- [x] `docs/adr/v3.0-005-json-pipeline.md` — 213 lines / 11,936 bytes

Commits exist:
- [x] 5b4d55f — Task 1 divergence catalog
- [x] c392f46 — Task 2 ADR-001 + ADR-005
- [x] e05e7cf — Task 3 ADR-002
