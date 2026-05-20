# Phase 32: Test Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 32-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-18
**Phase:** 32-test-hardening
**Areas discussed:** E2E smoke workflow choice, README v3.0 hard-claim wording, Free42 contamination guard strictness, Numerical-accuracy case-selection priorities

---

## E2E smoke workflow choice

### Q1: Which Math Pac I workflow lands in the E2E smoke spec?

| Option | Description | Selected |
|--------|-------------|----------|
| MATRIX 2x2 DET multi-step | Multi-step (~8 clicks). Exercises modal pipeline (ORDER=?, A1,1=?, R/S submit, modal_program transitions, LCD alternation). ~30s test runtime estimate. | |
| sinh(1) one-liner | Single XEQ-by-name path (~5 clicks). Exercises alpha-collection + xrom_resolve + display update. Faster (~10s). | |
| Both — two new test cases | Ship sinh(1) AND MATRIX DET as two separate test functions. Doubles E2E runtime budget. Exceeds literal ROADMAP "OR" wording. | ✓ |

**User's choice:** Both — two new test cases
**Notes:** Both paths exercise structurally distinct surfaces (XEQ resolution vs modal pipeline). Captured as D-32.2.

### Q2: smoke.spec.js → smoke.spec.ts conversion?

| Option | Description | Selected |
|--------|-------------|----------|
| Convert to .ts | Matches ROADMAP wording. Requires `@wdio/typescript-support` devDep + tsconfig + CI re-validation. | |
| Keep as .js | Smaller blast radius; no toolchain risk. ROADMAP wording stays a minor drift. | ✓ |

**User's choice:** Keep as .js (after I recommended this option with rationale)
**Notes:** User asked for my recommendation. I recommended keeping `.js`: zero functional benefit (smoke uses raw selectors, no IPC type imports), real toolchain risk for the e2e-linux CI job, D-27.15 AMENDED spirit-vs-letter precedent. Captured as D-32.1.

### Q3: MATRIX DET scope — verify cleanup with Esc?

| Option | Description | Selected |
|--------|-------------|----------|
| Just verify the answer | Click through to DET, assert LCD = -2.0000. Minimal. | ✓ |
| Add Esc-cancel verification | After DET, press Esc to confirm modal_program clears cleanly. ~2 extra clicks. | |

**User's choice:** Just verify the answer
**Notes:** Cancel semantics tested at Vitest unit-test layer; E2E scope bounded by D-27.13 precedent. Captured as D-32.3.

---

## README v3.0 hard-claim wording

### Q1: Does Phase 32 graduate the README v3.0 line from soft-claim to hard claim?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, hard-claim at ship | Final ship action replaces soft-claim with hard claim. Matches v2.2 precedent. Gate condition verified in same commit. | ✓ |
| Stay soft-claim | Keep current conservative wording. Future polish phases can graduate. | |
| Hard-claim only after buffer milestone | Land soft-claim; graduate in follow-up quick-task after main-stability buffer. | |

**User's choice:** Yes, hard-claim at ship
**Notes:** Aligns with Phase 30 D-30.9 deferred-to-Phase-32 plan. Captured as D-32.5.

### Q2: Which wording for the hard claim?

| Option | Description | Selected |
|--------|-------------|----------|
| Feature-complete Math Pac I behavioral emulation | Mirror of v2.2 precedent. Concise. | |
| Math Pac I behavioral emulation (10 programs, ~55 functions, documented divergences) — feature-complete | Preserves count + en-tag with feature-complete. | |
| v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034 | OM-cited; strongest grounding; longest. | ✓ |

**User's choice:** OM-cited wording
**Notes:** OM citation grounds the hard claim in the authoritative spec. Behavioral-emulation scope is implicit in "per Owner's Manual" (vs ROM-image reproduction). Captured as D-32.5.

### Q3: When in Phase 32 does the hard-claim commit land?

| Option | Description | Selected |
|--------|-------------|----------|
| Final ship commit, gate-verified | Wraps README + PROJECT.md + CLAUDE.md edits with `just coverage` output embedded. Strongest provenance link. | ✓ |
| Plan 32-03 closing commit | Land as part of E2E + Free42 guard plan. Less ceremonial. | |
| Separate doc-only commit after verification | Standalone `docs(32): graduate v3.0 hard-claim` commit after 32-01 lands. Clearest audit trail. | |

**User's choice:** Final ship commit, gate-verified
**Notes:** Strongest provenance: reviewer sees test results and README change in one diff. Mirrors v2.2 tag-landed-with-gate-raised pattern. Captured as D-32.6.

---

## Free42 contamination guard strictness

### Q1: Which guard policy?

| Option | Description | Selected |
|--------|-------------|----------|
| Option A — tight 12-symbol grep | Distinctive Free42 identifiers + copyright/license markers; allowlist disclaim header. Low FP risk; hits realistic copy-paste threat. (Recommended.) | ✓ |
| Option B — aggressive (24+ symbols + function-name patterns) | Catches more rename-attempts; higher FP rate from our own naming drift. | |
| Option C — minimal (4 unambiguous symbols) | phloat, Phloat, bid128_, Thomas Okken only. Zero FPs; misses decNumber + GPL markers. | |
| Option D — headers-only verification | Only verifies disclaim header presence. Hygiene gate, not contamination. | |

**User's choice:** Option A — tight 12-symbol grep
**Notes:** User asked for context-explanation before deciding. I expanded each option with concrete examples (Free42 cpp snippet showing `phloat`, `BID_UINT128`, `decNumberIsNaN`, copyright lines), the realistic threat model (well-intentioned contributor copy-pasting to verify behavior), and my recommendation with reasoning. User selected Option A. Captured as D-32.7.

### Q2: Where should the guard run in CI?

| Option | Description | Selected |
|--------|-------------|----------|
| Inside `just ci` (every PR) | Adds to existing `just ci` recipe. Fastest feedback. Failure blocks merge. | |
| Separate ci.yml step | Standalone "License audit" step. Self-describing in PR checks panel. | |
| Both — just ci + separate visible step | Belt + suspenders. Slight CI duplication; clearer audit trail. | ✓ |

**User's choice:** Both
**Notes:** Local-dev path catches contamination before push; CI path makes gate visible to reviewers. Mirrors `scripts/check-tauri-permissions.sh` pattern. Captured as D-32.8.

---

## Numerical-accuracy case-selection priorities

### Q1: How should the ~134 new cases distribute across Math Pac I families?

| Option | Description | Selected |
|--------|-------------|----------|
| Risk-weighted (recommended) | POLY ~25, CMPLX ~20, MAT ~18, INTG ~15, SOLVE ~15, DIFEQ ~12, HYP ~10, TRI ~8, FOUR ~6, TRANS ~3, REAL ~2. Mirrors D-27.3 pattern. | ✓ |
| Equal per program (~12 each) | 12 × 11 = 132. Simpler; misses bug-class density. | |
| Coverage-gap-driven | Run `just coverage` first; allocate to lowest-covered files. Data-driven. | |
| User picks specific must-have cases | Skip weighting question; user lists 5–10 OM examples; planner builds rest. | |

**User's choice:** Risk-weighted (recommended)
**Notes:** Reflects bug-class density from Phase 28-31 research (POLY multiplicity-as-cluster Pitfall 5, CMPLX (0,0) boundary Pitfall 6). Captured as D-32.9.

### Q2: POLY multiplicity-as-cluster — how strict is the cluster assertion?

| Option | Description | Selected |
|--------|-------------|----------|
| Cluster centroid within 1e-4 of expected root | For (x-1)^5: mean(roots).re ≈ 1.0 ± 1e-4 AND max(\|imag\|) < 1e-3. Hardware-faithful. | ✓ (Claude pick after "you decide") |
| Per-root within 1e-2 of expected | Looser; risks missing genuine drift. | |
| Bit-exact reproduction of OM example | Strictest; risks cross-platform x86/ARM drift (Pitfall 14). | |

**User's choice:** "you decide" — Claude picked centroid-within-1e-4
**Notes:** Recommended pick: hardware-faithful, OM-documented cluster spread, avoids FPU drift, stays within 1e-7 default tolerance discipline as a deliberate exception. Captured as D-32.10.

### Q3: INTG and SOLVE error paths — explicit cases?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, ~3 cases each | INTG: 3 cases hitting 2^15 subdivision cap; SOLVE: 3 cases hitting non-convergence. Asserts HpError::Domain. | ✓ |
| No, success path only | Skip in accuracy suite; rely on math1_user_callback.rs + math1_solve_paths.rs. | |

**User's choice:** Yes, ~3 cases each
**Notes:** Error branches receive little organic test exposure; 3 cases per program closes the accuracy-suite gap without bloating. Captured as D-32.11.

---

## Claude's Discretion

Areas where user said "you decide" or where the discussion left implementation flexibility (captured in CONTEXT.md `### Claude's Discretion` section):

- POLY cluster assertion strictness (centroid-within-1e-4 — Claude picked after "you decide")
- Exact distribution of the ~134 cases within each family
- Per-case `// Catches: <bug class>` doc comment wording
- `lint_math1_assertions.rs` scope (recommended: math1_*.rs only)
- `lint_math1_assertions.rs` strictness on manual-tolerance patterns (recommended: flag them too)
- Coverage-gap-closure plan if files drop below 90 %
- `scripts/check-free42-contamination.sh` exit-code semantics + `set -euo pipefail`
- Final ship commit message exact format
- PROJECT.md and CLAUDE.md update wording for the `v3.0 additions` Phase 32 subsection

## Deferred Ideas

Captured in CONTEXT.md `<deferred>` section:

- Smoke spec `.js` → `.ts` conversion (when a test legitimately needs strong typing)
- Esc-cancel verification in MATRIX DET E2E (Vitest unit-test layer handles)
- Coverage gate raise above 95 % (v3.1+ when risk-weighted tests justify)
- Cross-platform CI matrix expansion (ARM macOS / Linux aarch64 jobs — v3.x+ if drift surfaces)
- `lint_math1_assertions.rs` widening to all `tests/` (when a non-math1 gap surfaces)
- Headers-only audit as a sibling script (`scripts/check-disclaim-header.sh`)
- PROJECT.md graduation to "Shipped v3.0" entry (final-commit checklist item)
