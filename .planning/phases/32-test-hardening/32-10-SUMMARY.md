---
phase: 32
plan: "32-10"
subsystem: test-hardening
tags:
  - graduation
  - coverage-gate
  - readme
  - claude-md
  - project-md
  - ship
dependency_graph:
  requires:
    - 32-04
    - 32-05
    - 32-06
    - 32-07
    - 32-08
    - 32-09
  provides:
    - QUAL-01 closed (workspace coverage gate ≥ 95 % lines / ≥ 93 % regions MET)
    - README v3.0 hard-claim graduated per D-32.5
    - CLAUDE.md DEFERRED narrative replaced with MET narrative
    - PROJECT.md Active block graduated to fully-shipped v3.0
  affects:
    - QUAL-01 (workspace coverage gate)
    - ROADMAP SC-1 (per-file 90 % math1 floor)
tech_stack:
  added: []
  patterns:
    - programmatic coverage gate via `cargo llvm-cov --fail-under-lines/--fail-under-regions`
    - human-checkpoint gating of public README claims (Task 2 blocking gate)
---

# Plan 32-10 SUMMARY — README v3.0 hard-claim graduates

## Purpose

This plan is the **SHIP commit** for the post-Phase-32 gap-closure run. Plan 32-03 (the original Phase 32 ship) deferred the README v3.0 hard-claim graduation to a v3.0.1 follow-up milestone per Rule 4 (workspace coverage was 91.74 % lines / 92.14 % regions — below the 95 % / 93 % gate inherited from v2.2). The user explicitly reversed that deferral and authorized an in-Phase-32 gap-closure run. Plans 32-04..32-09 closed the gap empirically by adding ~70 risk-weighted error-branch tests; this plan ratifies the closure in the public-facing docs.

## What Shipped

### Final coverage measurement (Task 1 — programmatic gate)

`cargo llvm-cov --package hp41-core --fail-under-lines 95 --fail-under-regions 93 --summary-only` exit 0 (gate MET):

| Metric | Target | Final | Δ from baseline |
|--------|--------|-------|-----------------|
| **Lines** | ≥ 95.0 % | **95.39 %** | +3.65 pts (from 91.74 %) |
| **Regions** | ≥ 93.0 % | **94.26 %** | +2.12 pts (from 92.14 %) |

Per-file `ops/math1/*.rs` (all ≥ 90 % per ROADMAP SC-1):

| File | Baseline | Final | Δ |
|------|----------|-------|---|
| poly.rs | 76.37 % | 90.45 % | +14.08 |
| trans.rs | 81.17 % | 95.86 % | +14.69 |
| four.rs | 81.29 % | 97.66 % | +16.37 |
| solve.rs | 85.77 % | 91.93 % | +6.16 |
| difeq.rs | 85.76 % | 92.35 % | +6.59 |
| matrix.rs | 89.68 % | 94.00 % | +4.32 |
| mod.rs | 56.25 % | 90.62 % | +34.37 |
| integ.rs | 90.86 % | 92.29 % | +1.43 |
| complex.rs | — | 99.54 % | — |
| modal.rs | — | 99.74 % | — |
| hyperbolics.rs | — | 99.60 % | — |
| tri.rs | — | 97.86 % | — |
| xrom.rs | — | 100.00 % | — |

`ops/program.rs`: 86.42 % → 87.57 % (+1.15 pts via 17 new tests in `program_error_branches.rs`). 2.43 pts short of the per-file 90 % sub-target; the **workspace-level gate compensates** and the user explicitly approved graduation under this condition (see Task 2 checkpoint below).

### Task 2 — Blocking human checkpoint (APPROVED)

Asked: "Approve the README v3.0 hard-claim graduation given the workspace gate (95.39 % lines / 94.26 % regions) is MET, all math1 files ≥ 90 %, but `program.rs` at 87.57 % is 2.43 pts short of its per-file sub-target?"

User answer: **"Approve as proposed"** — graduate README to OM-cited "feature-complete" hard-claim. Primary workspace gate is met; `program.rs` deficit is acceptable (workspace average compensates).

### Task 3 — File edits

**README.md** (line 50-51):
- REMOVED: `- Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry points, documented divergences)` (soft-claim)
- ADDED: `- v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034 ([documented divergences](docs/hp41-math1-divergences.md))` (OM-cited hard-claim per D-32.5)

**CLAUDE.md** (4 edits):
- Header line 114: `### v3.0 additions (Math Pac I Emulation, Phases 28–32 — README hard-claim DEFERRED pending coverage gate)` → `### v3.0 additions (Math Pac I Emulation, Phases 28–32)`
- Header line 155: `#### Phase 32 (Test Hardening & Quality Gates, shipped 2026-05-18 — README hard-claim DEFERRED)` → `#### Phase 32 (Test Hardening & Quality Gates, shipped 2026-05-18)`
- Bullet line 161: replaced the "Coverage gate ≥ 95 % UNMET — README hard-claim graduation DEFERRED (Rule 4)" paragraph with "Coverage gate ≥ 95 % MET — README hard-claim graduated (post-Phase-32 gap-closure run, 2026-05-18)" paragraph citing the new plan IDs (32-04..32-09), the 9 new test files, and the final 95.39 % / 94.26 % measurement.

**.planning/PROJECT.md** (2 edits):
- Shipped milestones block (line 42-47): v3.0 entry graduated from `IN PROGRESS` to fully shipped; Phase 32 entry updated to 10 plans (3 original + 7 gap-closure) with final 95.39 % / 94.26 % coverage.
- Active block (line 169-171): "Coverage gate ≥ 95 % UNMET (...) DEFERRED to v3.0.1" replaced with "v3.0 fully shipped (...) graduated via post-Phase-32 gap-closure run".
- Update log line 250: replaced the DEFERRED note with a closure note referencing the 10/10 plan completion.

### Task 4 — Ship commit

Single commit via `/git-workflow:commit --with-skills` (per CLAUDE.md "Git Workflow" mandate). Subject in English per CLAUDE.md "Commit language: English only". Body embeds the `cargo llvm-cov --package hp41-core --summary-only` TOTAL row + per-math1-file rows per D-32.6 provenance discipline.

## Frozen Invariants Preserved

- **SC-4 invariant:** No `hp41-gui/src-tauri/src/` changes; no calculator/math logic duplicated in GUI source.
- **No `hp41-core/src/` source changes:** Test files only (per Phase 25 onward freeze).
- **MSRV 1.88 unchanged.**
- **`#![deny(clippy::unwrap_used)]`** continues to apply; new test files in Wave 1 carry `#![allow]` at file scope per the established pattern.
- **Save-file backward compat preserved:** no new `CalcState` fields.

## Deviations from PLAN.md

1. **`ops/program.rs` per-file sub-target unmet** (87.57 % vs. ≥ 90 % target). The plan's `<done>` criterion listed `program.rs >= 90.0 %` alongside the workspace gate, but the automated `<verify>` command (`--fail-under-lines 95 --fail-under-regions 93`) only enforces workspace totals. Surfaced to the user at the Task 2 human checkpoint; user approved graduation under this condition. Workspace gate (the dominant signal) is MET.

2. **Wave 1 plans landed 14 commits before Task 4** rather than a single bundled SHIP commit. Wave 1 ran 5 parallel `gsd-executor` agents in worktree isolation; each landed its own atomic per-task commits before this plan's Task 4 ran. The Task 4 commit therefore bundles only the 3 README/CLAUDE.md/PROJECT.md graduation edits, not the test files (those landed in their own commits per Plans 32-04..32-09).

## Plan Linkage

Closes the gap surfaced by 32-VERIFICATION.md QUAL-01 finding. Supersedes the Plan 32-03 Rule 4 architectural decision. No v3.0.1 follow-up milestone needed for this gap.
