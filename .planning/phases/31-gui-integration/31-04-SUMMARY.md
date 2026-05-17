---
phase: 31-gui-integration
plan: 04
subsystem: hp41-core + hp41-gui (frontend)
tags: [catalog, xrom, help-overlay, vitest, surgical-core-exception]
dependency_graph:
  requires: [31-02]
  provides: [Op::Catalog(2) XROM enumeration, Math Pac I help overlay section]
  affects: [hp41-core/src/ops/program.rs, hp41-gui/src/help_data.ts, hp41-gui/src/HelpOverlay.tsx]
tech_stack:
  added: []
  patterns: [Vite static JSON-import, collapsible React sections, surgical hp41-core exception]
key_files:
  created:
    - hp41-core/tests/op_catalog_xrom.rs
  modified:
    - hp41-core/src/ops/program.rs
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
    - hp41-gui/src/HelpOverlay.test.tsx
    - hp41-gui/src/App.css
    - .planning/phases/31-gui-integration/31-CONTEXT.md
decisions:
  - Op::Catalog(2) ships instant-scroll (single-pass synchronous push); PSE-step deferred to v3.1 per W1 fix
  - Two-section HelpOverlay with Math 1 Pac as own XROM-7 section, both expanded by default
  - Vite static JSON-import for hp41-math1-functions.json; no vite.config.ts change needed
metrics:
  duration: "~40 minutes"
  completed: "2026-05-17"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 6
---

# Phase 31 Plan 04: Math Pac I Help Overlay + Op::Catalog(2) XROM Enumeration

Two independent deliverables landed in this plan: (a) surgical `hp41-core` exception replacing the `Op::Catalog(2)` "NOT AVAILABLE" stub with real XROM module enumeration, and (b) `hp41-gui` frontend extension rendering Math Pac I entries in the `?`-overlay as a second top-level collapsible section.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Surgical Op::Catalog(2) extension + regression tests | 5d636fe | hp41-core/src/ops/program.rs, hp41-core/tests/op_catalog_xrom.rs |
| 2 | Parallel-load hp41-math1-functions.json in help_data.ts | 1ee327b | hp41-gui/src/help_data.ts |
| 3 | Two-section HelpOverlay.tsx + CSS + Vitest tests + context amendment | 56198cb | hp41-gui/src/HelpOverlay.tsx, HelpOverlay.test.tsx, App.css, 31-CONTEXT.md |

## Key Metrics

- **Math Pac I entries loaded from JSON:** 45 entries, 11 distinct categories
- **MATH_1.ops entries (for CAT 2 output):** 52 (including ASCII aliases for Unicode mnemonics)
- **Vitest test count:** 153 total (up from 142 before this plan; 11 new tests added)
- **Op::Catalog(2) print_buffer output (default CalcState, xrom_modules=0b0000_0001):**

```
-- CATALOG 2 --          (banner, always prepended by op_catalog)
XROM 7 MATH 1A           (module header: XROM {id} {name})
SINH                     (52 function name lines follow in MATH_1.ops order)
COSH
TANH
ASINH
ACOSH
ATANH
C+
C-
C×  (C\u{00D7})
C*  (ASCII alias)
C÷  (C\u{00F7})
C/  (ASCII alias)
REAL
MAGZ
CINV
Z↑N  Z^N  (Unicode + ASCII alias)
Z↑1/N  Z^1/N
E↑Z  E^Z
LNZ  SINZ  COSZ  TANZ
A↑Z  A^Z  LOGZ
Z↑W  Z^W
POLY  ROOTS
MATRIX  SIZE  VMAT  EDIT  DET  INV  SIMEQ  VCOL
INTG  SOLVE  SOL  DIFEQ
FOUR
SSS  ASA  SAA  SAS  SSA
TRANS  T3D
-- END --                (footer, always appended by op_catalog)
```

## Surgical hp41-core Exception Ledger

**Pattern:** analogous to v2.2 Plan 25-03 `builtin_card_op` 4→12 extension.

| Field | Value |
|-------|-------|
| File | `hp41-core/src/ops/program.rs::op_catalog` |
| Change | Split `2..=4 =>` arm into `2 =>` (XROM enumeration) + `3..=4 =>` (NOT AVAILABLE preserved) |
| API impact | None — visibility stays `pub fn op_catalog`, signature unchanged |
| Op variants added | Zero — uses existing `MATH_1` constant from Phase 28 |
| MATH_1.ops changes | None — reads read-only; Phase 28 freeze preserved |
| Import added | `use crate::ops::math1::xrom::MATH_1;` |
| SC-4 impact | None — change lives in `hp41-core/`, not `hp41-gui/src-tauri/` |
| W1 deviation | Instant-scroll (no PSE-step); matches v2.2 CAT 1 behavior exactly |

## Divergences

**D-31-04-01:** Plan 31-04 ships instant-scroll for `Op::Catalog(2)`; D-31.12 (~500ms PSE delay per line) and D-31.14 (R/S pauses/resumes; other keys cancel) are DEFERRED to v3.1 polish.

**Rationale:** v2.2 CAT 1 has no PSE-step infrastructure (verified RESEARCH Open Q2 — `op_catalog` is a single-pass synchronous loop that pushes ALL lines into `print_buffer` without any per-line yield, sleep, or state machine). Introducing PSE-step infrastructure would require a new abstraction (PSE-step iterator + per-line tick state on `CalcState` + frontend polling cooperation) that is out of Phase 31 scope (Phase 28 freeze + risk of broader hp41-core surgery beyond the surgical 1-arm extension). Instant-scroll matches v2.2 CAT 1 behavior bit-for-bit (consistency over net-new). Defer to v3.1 polish; revisit if user feedback demands hardware-faithful scroll timing.

## Context Amendment Confirmation

The `## Deferred Ideas` section in `.planning/phases/31-gui-integration/31-CONTEXT.md` now contains the following bullet (appended after "CAT 2 module-header verbosity"):

> **CAT 2 PSE-step infrastructure (D-31.12, D-31.14)** — v3.1 polish. Phase 31-04 ships instant-scroll per RESEARCH Open Q2 (verified v2.2 CAT 1 has NO PSE-step infrastructure — `op_catalog` is a single-pass synchronous loop). D-31.12's "~500ms PSE delay between lines" and D-31.14's "R/S pauses/resumes; other keys cancel" require introducing a new abstraction (PSE-step iterator + per-line tick state on `CalcState` + frontend cooperation) that is out of Phase 31 scope (Phase 28 freeze + risk of broader hp41-core surgery). Defer to v3.1 polish; revisit if user feedback demands hardware-faithful scroll timing.

## Deviations from Plan

### Planned Deviations (W1 fix — documented)

**1. [W1 fix - Scope] CAT 2 instant-scroll instead of PSE-step**
- **Rationale:** RESEARCH Open Q2 confirmed v2.2 CAT 1 has NO PSE-step infrastructure. Plan 31-04 adopted instant-scroll (single-pass synchronous push) consistent with v2.2 CAT 1.
- **Impact:** D-31.12 (~500ms PSE delay) and D-31.14 (R/S pause/resume) DEFERRED to v3.1 polish.
- **Documented in:** `<context_amendment>` in plan + `## Deferred Ideas` in 31-CONTEXT.md.

### Auto-fixed Issues

None — plan executed exactly as written for all three tasks.

## Quality Gate Results

| Gate | Command | Result |
|------|---------|--------|
| op_catalog_xrom tests | `cargo test --package hp41-core --test op_catalog_xrom` | 3/3 PASS |
| xrom_shadowing gate | `cargo test --package hp41-core --test xrom_shadowing` | 2/2 PASS |
| Vite build | `cd hp41-gui && npm run build` | PASS (255 kB bundle) |
| Vitest full suite | `cd hp41-gui && npm test` | 153/153 PASS |
| SC-4 invariant | `grep -rn "fn op_(add\|sub\|...)(" hp41-gui/src-tauri/src/` | CLEAN (no matches) |

## Self-Check: PASSED

Files verified to exist:
- hp41-core/src/ops/program.rs — contains `2 =>` arm with `MATH_1.ops` iteration
- hp41-core/tests/op_catalog_xrom.rs — 3 `#[test]` functions
- hp41-gui/src/help_data.ts — exports `helpEntriesMath1`, `helpEntriesAll`, `XromEntry`
- hp41-gui/src/HelpOverlay.tsx — contains `SECTIONS` constant + `help-overlay-section-heading` buttons
- hp41-gui/src/App.css — contains `.help-overlay-section-heading` with `font-size: 14px; font-weight: 700`
- .planning/phases/31-gui-integration/31-CONTEXT.md — CAT 2 PSE-step deferral bullet added

Commits verified:
- 5d636fe: feat(31-04): implement Op::Catalog(2) XROM enumeration + regression tests
- 1ee327b: feat(31-04): extend help_data.ts with Math Pac I JSON parallel-load
- 56198cb: feat(31-04): two-section ?-overlay for Math Pac I + CSS + tests + context amendment
