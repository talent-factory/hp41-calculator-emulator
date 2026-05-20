---
phase: 30-documentation-adrs
plan: "03"
subsystem: docs
tags:
  - docs
  - readme
  - claude-md
  - project-md
  - v3-additions-block
dependency_graph:
  requires:
    - 30-01 (docs/hp41-math1-function-matrix.md — README link target)
    - 30-02 (docs/adr/v3.0-001/002/005-*.md — CLAUDE.md cross-reference targets)
  provides:
    - README.md v3.0 soft-claim + Documentation table row
    - .planning/PROJECT.md v3.0 milestone progress lines
    - CLAUDE.md v3.0 additions block (Phases 28/29/30 populated, 31/32 stub)
  affects:
    - All future CLAUDE.md readers (v3.0 block is the canonical reference for v3.0 decisions)
    - Public repo README (soft-claim is a public assertion about emulator capabilities)
tech_stack:
  added: []
  patterns:
    - D-30.8 v3.0-additions-block incremental population (parallel to v2.2 ship pattern)
    - D-30.9 soft-claim + deferred hard claim (same gating discipline as v2.2)
key_files:
  created: []
  modified:
    - README.md
    - .planning/PROJECT.md
    - CLAUDE.md
decisions:
  - "D-30.8: v3.0 additions block in CLAUDE.md between v2.2 Phase 27 and Tech Stack; Phase 28/29/30 fully populated; Phase 31/32 stub headers"
  - "D-30.9: README soft-claim 'Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry points, documented divergences)' under ## Features; hard claim deferred to Phase 32"
  - "D-30.8 (b): PROJECT.md concise milestone lines only; CLAUDE.md carries the full additions block"
metrics:
  duration: "~4 minutes"
  completed: "2026-05-17"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 3
  files_created: 0
requirements:
  - DOC-05
  - DOC-06
---

# Phase 30 Plan 03: README + PROJECT.md + CLAUDE.md v3.0 Narrative Summary

**One-liner:** Locked the v3.0 Math Pac I milestone narrative across three briefing files — soft-claim in README, Phase 28/29/30 milestone lines in PROJECT.md, and a fully populated v3.0 additions block in CLAUDE.md (Phases 28–30 bodies, 31–32 stub headers, frozen-invariants tail).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add v3.0 soft-claim and matrix link to README.md | cfae186 | `README.md` |
| 2 | Update PROJECT.md Shipped milestones and Current focus lines | 2de3343 | `.planning/PROJECT.md` |
| 3 | Insert v3.0 additions block into CLAUDE.md after v2.2 Phase 27 block | eacc061 | `CLAUDE.md` |

## Artifact Summary

### README.md — 3 lines added

**Exact README soft-claim bullet (verbatim D-30.9 wording):**

```
- Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry
  points, documented divergences) — see [Math Pac I Function Matrix](docs/hp41-math1-function-matrix.md)
```

Added under `## Features → **Calculator engine (`hp41-core`)**` sub-list, after the "Persistent state via JSON" bullet.

**Documentation table row added:**

```
| [Math Pac I Function Matrix](docs/hp41-math1-function-matrix.md) | Math Pac I XROM entries with module/function IDs |
```

Added in `## Documentation` table, immediately after the hp41cv-function-matrix.md row.

Hard claim "feature-complete Math Pac I" is absent — deferred to Phase 32 per D-30.9.

### .planning/PROJECT.md — 5 lines added, 1 line replaced

Shipped milestones block gained a v3.0 IN PROGRESS entry with three sub-lines:
- Phase 28 XROM Framework + Math Pac I Core Ops (2026-05-16)
- Phase 29 CLI Integration (2026-05-17)
- Phase 30 Documentation & ADRs — IN PROGRESS

Active v3.0 block italic line replaced. **PROJECT.md "Stand" date used: 2026-05-17.**

### CLAUDE.md — 50 lines added

**v3.0 additions block line range:**
- Starts at line 114 (`### v3.0 additions (Math Pac I Emulation, Phases 28–30 — 31–32 IN PROGRESS)`)
- Ends at line 163 (blank line before `## Tech Stack`)
- `## Tech Stack` header is at line 164

The block sits **between the `### v2.2 additions (Test Hardening, Phase 27)` section** (ends at line 112–113) **and the `## Tech Stack` header** (line 164).

**Subsection inventory:**

| Subsection | Status | Decision bullets |
|------------|--------|-----------------|
| Phase 28 (XROM Framework + Math Pac I Core Ops) | Fully populated | 12 bullets covering C-28.1..C-28.4 + D-28.1..D-28.9 |
| Phase 29 (CLI Integration) | Fully populated | 7 bullets covering D-29.1..D-29.6 |
| Phase 30 (Documentation & ADRs) | Fully populated | 6 bullets covering D-30.1..D-30.9 |
| Phase 31 (GUI Integration) | Stub `(in progress)` | Header only per D-30.8 |
| Phase 32 (Test Hardening & Quality Gates) | Stub `(in progress)` | Header only per D-30.8 |
| Frozen invariants tail | Present | SC-4, 4-exhaustive-match, unwrap_used, save-file compat, MSRV 1.88 |

**Hard claim confirmation:** `grep -cF "feature-complete Math Pac I" CLAUDE.md` returns **0**. The Phase 30 subsection refers to the deferred claim as "The hard 'completeness' claim is deferred to Phase 32..." to avoid the exact string.

## Verification

```
README.md:
  grep -cF "Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry" README.md = 1
  grep -cF "docs/hp41-math1-function-matrix.md" README.md = 2 (Features + Documentation table)
  grep -cF "feature-complete Math Pac I" README.md = 0
  git diff --stat README.md = 3 insertions, 0 deletions

PROJECT.md:
  grep -cF "v3.0 Math Pac I Emulation" .planning/PROJECT.md = 1
  grep -cF "Phase 28 XROM Framework + Math Pac I Core Ops (2026-05-16)" .planning/PROJECT.md = 1
  grep -cF "Phase 29 CLI Integration (2026-05-17)" .planning/PROJECT.md = 1
  grep -cF "Phase 30 Documentation & ADRs — IN PROGRESS" .planning/PROJECT.md = 1
  grep -cF "Stand 2026-05-17" .planning/PROJECT.md = 1
  git diff --stat .planning/PROJECT.md = 5 insertions, 1 deletion

CLAUDE.md:
  grep -c "### v3.0 additions ..." = 1
  grep -cE "^#### Phase 28|...|^#### Phase 32" = 5
  grep -c "(in progress)" = 2
  grep -cE "D-30\.[1-9]" = 6
  grep -cE "(C-28|D-28)\.[1-9]" = 13
  grep -cF "feature-complete Math Pac I" = 0
  grep -c "^### v2\.2 additions" = 2
  wc -l CLAUDE.md = 236 (HEAD was 186; +50 lines)

Link targets:
  docs/hp41-math1-function-matrix.md — EXISTS (Wave 1 / Plan 30-01)
  docs/adr/v3.0-001-op-strategy.md — EXISTS (Wave 1 / Plan 30-02)
  docs/adr/v3.0-002-user-callback-policy.md — EXISTS (Wave 1 / Plan 30-02)
  docs/adr/v3.0-005-json-pipeline.md — EXISTS (Wave 1 / Plan 30-02)

SC-4 invariant:
  git diff HEAD~3 HEAD -- 'hp41-core/src/' 'hp41-cli/src/' 'hp41-gui/src/' 'hp41-gui/src-tauri/src/' = 0 files
```

## Deviations from Plan

### Minor Deviation: CLAUDE.md line count estimate

**Rule applied:** None (estimation deviation — plan line-count floor was advisory, not a hard gate)

**Found during:** Task 3 post-edit verification

**Issue:** Plan's acceptance criterion `wc -l CLAUDE.md` returns ≥ 280 was based on a wrong estimate that HEAD was "~270 lines including v2.2 blocks". Actual HEAD line count: **186 lines**. The v3.0 block added 50 lines, resulting in 236 total.

**Impact:** Zero functional impact. All substantive acceptance criteria pass:
- v3.0 heading present (1 match)
- All 5 phase headers present
- 2 `(in progress)` markers
- 6 D-30.x references (≥6)
- 13 C-28/D-28 references (≥10)
- All ADR cross-references present
- Hard claim absent (0 matches)
- Both v2.2 sections preserved (2 matches)
- Block correctly positioned between v2.2 Phase 27 and Tech Stack

The block contains all decision content specified in 30-03-PLAN.md Task 3 action. The 50-line block is complete and substantive — the estimate was simply wrong about the starting file size.

## Known Stubs

None. All three files contain substantive content:
- README soft-claim is verbatim-locked D-30.9 wording
- PROJECT.md milestone lines use real ship dates from STATE.md
- CLAUDE.md v3.0 block Phase 28/29/30 subsections are fully populated; Phase 31/32 are correctly stub-only per D-30.8

## Threat Flags

None. Files modified (`README.md`, `.planning/PROJECT.md`, `CLAUDE.md`) introduce no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The README soft-claim is the only public-facing assertion — it uses the conservative D-30.9 wording and explicitly defers the hard claim to Phase 32.

## Self-Check: PASSED

| Item | Result |
|------|--------|
| `README.md` exists with soft-claim | FOUND |
| `.planning/PROJECT.md` exists with Phase 28/29/30 lines | FOUND |
| `CLAUDE.md` exists with v3.0 additions block | FOUND |
| `docs/hp41-math1-function-matrix.md` link target | FOUND |
| `docs/adr/v3.0-001-op-strategy.md` link target | FOUND |
| `docs/adr/v3.0-002-user-callback-policy.md` link target | FOUND |
| `docs/adr/v3.0-005-json-pipeline.md` link target | FOUND |
| Commit `cfae186` (Task 1 README) | FOUND |
| Commit `2de3343` (Task 2 PROJECT.md) | FOUND |
| Commit `eacc061` (Task 3 CLAUDE.md) | FOUND |
| `git diff --stat README.md CLAUDE.md .planning/PROJECT.md` (net insertions) | 58 lines added |
| No source code changes (SC-4 invariant) | CONFIRMED |
| `feature-complete Math Pac I` absent from all three files | CONFIRMED (0 matches each) |
