---
phase: 32
plan: "32-03"
subsystem: tests-ci-docs
tags:
  - test-hardening
  - e2e
  - ci-gate
  - license-audit
  - QUAL-03
  - QUAL-05
  - D-32.2
  - D-32.3
  - D-32.4
  - D-32.5
  - D-32.6
  - D-32.7
  - D-32.8
  - Pitfall 19
requires:
  - 32-01 (Wave 1 — meta-gate graduation + lint + QUAL-08 explicit categories)
  - 32-02 (Wave 1 — numerical_accuracy.rs 566 → 763 cases)
provides:
  - hp41-gui/e2e/smoke.spec.js (extended from 1 to 3 it() blocks)
  - scripts/check-free42-contamination.sh (NEW)
  - Justfile license-audit recipe + extended ci chain
  - .github/workflows/ci.yml license-audit parallel job
  - CLAUDE.md Phase 32 shipped subsection (replaces in-progress stub)
  - .planning/PROJECT.md Phase 31 + 32 shipped entries
affects:
  - README.md (UNCHANGED — hard-claim graduation deferred per Rule 4)
tech-stack:
  added: []
  patterns:
    - browser.executeAsync + window.__TAURI_INTERNALS__.invoke fallback for E2E
      tests when on-screen modal click path is blocked (alphaChar/Enter
      collision in xeq_name modal)
    - Bash + grep + exit-N-on-match CI gate with disclaim-header allowlist
      (mirrors scripts/check-tauri-permissions.sh)
    - just CLI invocation parity across local-dev (just license-audit) and
      CI (taiki-e/install-action@v2 + just license-audit) per D-32.8
key-files:
  created:
    - scripts/check-free42-contamination.sh
  modified:
    - hp41-gui/e2e/smoke.spec.js
    - Justfile
    - .github/workflows/ci.yml
    - CLAUDE.md
    - .planning/PROJECT.md
decisions:
  - "click-strategy: browser.execute fallback for XEQ-by-name invocations (Open Question #1 from RESEARCH.md) — the on-screen xeq_name modal cannot type the letter 'N' because the enter key carries alphaChar='N' but App.tsx::handleClick lines 387-400 prioritize effectiveId === 'enter' as submit before alphaChar routing. Both new it() blocks use the fallback for the XEQ-by-name dispatches; the MATRIX test uses real clicks for digit + R/S submits between matrix elements (hybrid)."
  - "MATRIX input order: column-major (row varies fastest within each column) per hp41-core/src/ops/math1/matrix.rs::submit_modal lines 372-401. For [[1,2],[3,4]] the input sequence at the column-major prompts A1,1 → A2,1 → A1,2 → A2,2 is 1, 3, 2, 4. DET is invariant under transpose so the -2.0000 assertion holds regardless."
  - "Free42 contamination guard PATTERN excludes bare 'Free42' deliberately — 122 legitimate 'Free42 v3.0.5: <value>' cross-check references exist in the codebase per RESEARCH.md; the 12 D-32.7 distinctive symbols are tight enough to never match those."
  - "ci.yml license-audit job uses 'just license-audit' invocation (Option B in PATTERNS.md) for parity with local-dev path; the simpler 'bash scripts/...' direct invocation was rejected for the cosmetic-clarity reason — same command across both invocation paths reduces future debugging surface."
  - "**RULE 4 architectural decision**: README v3.0 hard-claim graduation DEFERRED. Coverage gate 91.74% lines / 92.14% regions is below the gate-conditional 95%/93% precondition mandated by D-30.9 / D-32.5. Per the Plan 32-03-05 acceptance criteria STOP clause, the README v3.0 soft-claim line stays as-is; documentation updates in CLAUDE.md + PROJECT.md reflect Phase 32 ship status WITHOUT the README hard claim. A follow-up v3.0.1 milestone will own the per-file error-branch coverage push."
metrics:
  duration: ~75 minutes
  completed: 2026-05-18
---

# Phase 32 Plan 03: E2E Math Pac I Smoke + Free42 Contamination Guard + Documentation Update Summary

Extends `hp41-gui/e2e/smoke.spec.js` with two new Math Pac I `it()` blocks (`XEQ "SINH" 1 → 1.1752` via xrom_resolve and `XEQ "MATRIX" 2x2 DET → -2.0000` via modal pipeline) using a documented `browser.execute` fallback for the XEQ-by-name submit-key gap; ships `scripts/check-free42-contamination.sh` implementing the D-32.7 12-symbol grep policy with disclaim-header allowlist; wires the guard via D-32.8 belt+suspenders into BOTH `just license-audit`/`just ci` AND `.github/workflows/ci.yml::license-audit` parallel job; updates CLAUDE.md (Phase 32 shipped subsection replaces in-progress stub) and PROJECT.md (Phase 30/31/32 graduation entries) to reflect Phase 32 ship status — but DEFERS the README v3.0 hard-claim graduation per Rule 4 because `just coverage` reports 91.74% lines / 92.14% regions, below the gate-conditional 95%/93% precondition.

## Per-Task Completion

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 32-03-01 | Reconnaissance: xeq_name submit-key + alphaChar map + MAT-02 input order | — (reconnaissance only) | DONE |
| 32-03-02 | Extend smoke.spec.js with 2 new it() blocks (sinh + MATRIX DET) | `69877cd` | DONE |
| 32-03-03 | Create scripts/check-free42-contamination.sh | `ab84485` | DONE |
| 32-03-04 | Wire license-audit into Justfile + ci.yml parallel job | `4fa66e4` + `08aa8a0` | DONE |
| 32-03-05 | Final ship commit — docs update with deferred README claim | `4c72ed0` | DONE (Rule 4 deviation) |

## Must-Have Verification

| Must-have | Status | Evidence |
|-----------|--------|----------|
| `hp41-gui/e2e/smoke.spec.js` contains 3 `it()` blocks | ✅ MET | `grep -c "^    it("` returns 3 |
| Existing `2 ENTER 3 + → 5.0000` `it()` block unchanged (D-32.4) | ✅ MET | Old `it()` block byte-identical to pre-edit state; only NEW blocks added after it |
| New sinh test references `XEQ`, `SINH`, `1.1752` | ✅ MET | 3 mentions of SINH, 1 mention of 1.1752 |
| New MATRIX test references `MATRIX`, `DET`, `-2.0000` | ✅ MET | 3 mentions of MATRIX, 1 mention of -2.0000 |
| `scripts/check-free42-contamination.sh` exists + executable | ✅ MET | `test -x` passes; bash invocation exits 0 |
| Script line 1 is `#!/usr/bin/env bash` + `set -euo pipefail` present | ✅ MET | Verified |
| PATTERN contains all 12 D-32.7 symbols | ✅ MET | `grep -c` returns 1 for the locked alternation |
| PATTERN does NOT contain bare `Free42` token | ✅ MET | Manual verification of PATTERN line shows 12 symbols only |
| Allowlist references disclaim sentence | ✅ MET | `grep -c 'Free42 source consulted only as sanity-check oracle'` returns 1 |
| All 13 math1/*.rs files carry disclaim header | ✅ MET | `grep -L` returns empty |
| `bash scripts/check-free42-contamination.sh` exits 0 | ✅ MET | "OK: no Free42 contamination detected" |
| `Justfile` has `license-audit:` recipe in `[group('ci')]` | ✅ MET | Verified with -B 1 grep |
| `ci` recipe ends with `license-audit` | ✅ MET | `ci: lint test coverage license-audit` |
| `.github/workflows/ci.yml` has `license-audit:` job | ✅ MET | Verified — sibling to lint/test/coverage/msrv, no needs: |
| Job name is `License audit (Free42 contamination)` | ✅ MET | `grep -c` returns 1 |
| `just license-audit` exits 0 against current source | ✅ MET | Verified locally |
| 13 math1/*.rs files carry disclaim header (acceptance criterion) | ✅ MET | `grep -L` returns empty |
| **README v3.0 line graduated to hard claim** | ❌ **NOT MET (Rule 4 deferred)** | Coverage gate unmet; soft-claim stays; documented in deviation section below |
| `PROJECT.md` graduated Phase 32 into Shipped block | ✅ MET | New Phase 32 entry added to v3.0 history block; Active block updated |
| `CLAUDE.md` Phase 32 stub replaced with full subsection | ✅ MET | `grep -c 'Phase 32.*in progress'` returns 0; new shipped subsection ships in `### v3.0 additions` block |
| CLAUDE.md cites D-32.5, D-32.6, D-32.7, and Pitfall 19 | ✅ MET | `grep -c` returns 4 distinct mentions |
| `wdio.conf.cjs` NOT modified | ✅ MET | `git diff --name-only` shows no entry |
| `Display14Seg.tsx` NOT modified | ✅ MET | `git diff --name-only` shows no entry |

## Files Created / Modified

### Created

- **`scripts/check-free42-contamination.sh`** (27 lines) — D-32.7 12-symbol grep with disclaim-header allowlist; bash + `set -euo pipefail`; mirrors the v2.0 Phase 31 `scripts/check-tauri-permissions.sh` shape. Verified zero matches against current `hp41-core/src/ops/math1/`.

### Modified

- **`hp41-gui/e2e/smoke.spec.js`** (86 → 245 lines, +159 lines) — Adds the `invokeBackend(command, args)` helper (uses `browser.executeAsync` to invoke `window.__TAURI_INTERNALS__.invoke`) and two new `it()` blocks: `XEQ "SINH" 1 displays 1.1752 (Math Pac I via xrom_resolve)` and `XEQ "MATRIX" 2x2 DET displays -2.0000 (Math Pac I modal pipeline)`. The existing `2 ENTER 3 + displays 5.0000` `it()` block is preserved bit-for-bit per D-32.4. Both new tests carry leading-comment blocks documenting the click-strategy decision (browser.execute fallback + real clicks where appropriate) per T-32-03 mitigation.
- **`Justfile`** (+11 lines) — Adds `license-audit` recipe in `[group('ci')]` running `bash scripts/check-free42-contamination.sh`; extends `ci` recipe from `ci: lint test coverage` to `ci: lint test coverage license-audit` per D-32.8. Doc-comment header on the new recipe + on the extended `ci` recipe references Plan 32-03 + D-32.7 + D-32.8 + Pitfall 19.
- **`.github/workflows/ci.yml`** (+18 lines) — Adds `license-audit` parallel job (sibling to lint / test / coverage / msrv; no `needs:`; `ubuntu-latest`; uses `taiki-e/install-action@v2 tool: just` for parity with local-dev). Job name `License audit (Free42 contamination)` makes the gate self-describing in the GitHub PR checks panel per D-32.8. Workflow-injection surface is zero (no `${{ github.event.* }}` interpolation; pinned action; script is checked in).
- **`CLAUDE.md`** (+11 lines, –4 lines) — Header line updated from `Phases 28–30 — 31–32 IN PROGRESS` to `Phases 28–32 — README hard-claim DEFERRED pending coverage gate`. Replaces the `Phase 31 (GUI Integration) — (in progress)` stub with `shipped 2026-05-18`. Replaces the `Phase 32 (Test Hardening & Quality Gates) — (in progress)` stub with a full Phase 32 subsection covering all 5 Wave 1+2 deliverables (meta-gate graduation, lint, accuracy extension, E2E smoke, contamination guard, deferred README graduation) with their D-32.X decision references. Frozen-invariants block at end extended from `28–30` to `28–32`; `MSRV unchanged through Phase 28–32`; notes `approx 0.5.1` ratified for `lint_math1_assertions.rs`.
- **`.planning/PROJECT.md`** (+5 lines, –1 line) — Graduates Phase 30 from `IN PROGRESS` to shipped (2026-05-17 date). Adds Phase 31 + Phase 32 shipped entries with deliverable summaries. Updates the `Active (v3.0)` block to note 26/26 plans complete, coverage gate unmet, README hard-claim deferred. Appends a new dated `Last updated: 2026-05-18` line capturing the Phase 32 ship state.

## Decisions Made

- **D-32.2 click-strategy (per-test, documented per RESEARCH.md Open Question #1):**
  - **sinh test:** `browser.execute` fallback via `window.__TAURI_INTERNALS__.invoke('dispatch_op', { keyId: 'xeq_SINH' })`. Justification: the `xeq_name` modal's submit key is `enter` (per `pending_input.ts::handleModalKey` line 339), but the `enter` key also carries `alphaChar: 'N'` (per `Keyboard.tsx` line 105). `App.tsx::handleClick` lines 387-400 prioritize `effectiveId === 'enter'` as submit BEFORE alphaChar routing — so typing 'N' inside an open xeq_name modal is impossible (the click submits the partial label). The fallback bypasses the modal and exercises only the regression-sensitive `xrom_resolve` path.
  - **MATRIX DET test:** Hybrid — `browser.execute` for `xeq_MATRIX` and `xeq_DET` (same reason as sinh, for symmetry and to avoid the modal click ceremony), but real clicks for the four matrix element entries and the R/S submits between them. Real clicks for digits + R/S exercise D-31.1 R/S 3-way routing (`submit_modal` when `modal_program_active`) and the column-major iteration order in `matrix.rs::submit_modal` lines 372-401.
- **MAT-02 input order convention (V-02 follow-up resolved):** column-major — row varies fastest, then column. For `[[1, 2], [3, 4]]` the entry sequence at the column-major prompts A1,1 → A2,1 → A1,2 → A2,2 is **1, 3, 2, 4**. DET is invariant under transpose so the `-2.0000` assertion holds either way; the column-major order matches actual MAT-02 internal storage in `mat_setup` (lines 32-39 of `math1_matrix.rs`).
- **Free42 contamination guard PATTERN scope:** bare `Free42` token deliberately EXCLUDED. RESEARCH.md HIGH-confidence note (122 legitimate `Free42 v3.0.5: <value>` cross-check references across `numerical_accuracy.rs` + `math1/*.rs`) drives this — adding bare `Free42` would force false-positive triage on every cross-check reference. The 12 D-32.7 distinctive symbols (Intel BID + decNumber + Free42 internals + GPL/AGPL markers) catch the realistic threat (copy-paste from Free42 source) without false positives.
- **D-32.8 ci.yml invocation style choice:** `just license-audit` via `taiki-e/install-action@v2` (Option B in PATTERNS.md), NOT direct `bash scripts/check-free42-contamination.sh`. Justification: same command across both local-dev and CI invocation paths reduces future-debugging surface; the cosmetic-clarity benefit outweighs the slightly slower CI warmup time (one additional action install).
- **Rule 4 architectural decision — README hard-claim graduation DEFERRED:** `just coverage` reports 91.74% lines / 92.14% regions, both below the gate-conditional 95% / 93% threshold mandated by D-30.9 / D-32.5. Per Plan 32-03-05 acceptance criteria: "If line coverage is < 95.0% OR regions < 93.0%, STOP — do not proceed with the README hard-claim graduation." The README v3.0 soft-claim line stays as-is. CLAUDE.md + PROJECT.md updates DO reflect Phase 32 ship (the test/CI infrastructure shipped green), but the README OM-cited hard claim ("v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034") waits for a follow-up v3.0.1 milestone that closes the per-file error-branch coverage gap on the 7 below-90% math1 files.

## Deviations from Plan

### Rule 4 — Architectural Decision: README v3.0 hard-claim graduation DEFERRED

**Found during:** Task 32-03-05 — pre-commit gate verification

**Issue:** `just coverage` reports 91.74% lines / 92.14% regions on the v3.0 post-Phase-32 baseline. Both metrics fall below the gate-conditional 95% / 93% thresholds mandated by D-30.9 / D-32.5 / D-32.6 for the README hard-claim graduation.

**Per-file breakdown** (math1 source files below the 90% per-file floor per ROADMAP success criterion 1):

| File | Lines % | Status |
|------|--------:|--------|
| `ops/math1/poly.rs` | 76.37 % | ✗ below 90 % |
| `ops/math1/trans.rs` | 81.17 % | ✗ below 90 % |
| `ops/math1/four.rs` | 81.29 % | ✗ below 90 % |
| `ops/math1/mod.rs` | 56.25 % | ✗ below 90 % (Wave 1 lifted 0% → 56% surgically; further gap-closure needed) |
| `ops/math1/solve.rs` | 85.77 % | ✗ below 90 % |
| `ops/math1/difeq.rs` | 85.76 % | ✗ below 90 % |
| `ops/math1/integ.rs` | 90.86 % | ✓ just over (post-Wave 1) |
| `ops/math1/matrix.rs` | 89.68 % | ✗ marginally below 90 % |
| `ops/math1/complex.rs` | 99.54 % | ✓ |
| `ops/math1/hyperbolics.rs` | 99.60 % | ✓ |
| `ops/math1/modal.rs` | 99.48 % | ✓ |
| `ops/math1/tri.rs` | 97.86 % | ✓ |
| `ops/math1/xrom.rs` | 100.00 % | ✓ |

Plus non-math1 files dragging the workspace total: `ops/program.rs` 86.42 %, `ops/math.rs` 93.79 %, `ops/stats.rs` 86.26 %.

**Root cause analysis:** Wave 1 Plan 32-01 surgically closed the `ops/math1/mod.rs` 0%→56% gap (5 risk-weighted submit_modal / cancel_modal tests) and Wave 1 Plan 32-02 added 137 numerical_accuracy.rs cases. The Plan 32-02 cases exercise existing math1 source via `dispatch()` but did not materially move per-file coverage because the gaps live in error-branch arms (POLY-07 reject path on degree-3+ Bairstow; DIFEQ overflow guards; SOLVE non-convergence; TRANS Rodrigues 3D edge cases) NOT reachable through OM-cited happy-path cases. The structural gap was correctly identified as a "Defer to Plan 32-02" item in the Wave 1 Plan 32-01 SUMMARY, but Plan 32-02's case-driven approach was insufficient to close it because the cases targeted accuracy verification (not coverage padding per D-27.3).

**Decision:** Defer the README hard-claim graduation to a follow-up v3.0.1 milestone that owns:
1. A targeted error-branch test pass on the 7 below-90 % math1 files (5-15 tests per file with `// Catches: <bug class>` rationale per D-27.1)
2. Either: (a) close the gap to ≥ 90 % per file + ≥ 95 % workspace lines, OR (b) document acceptable per-file thresholds in an ADR raising the gap to the OM-divergence catalog (e.g., POLY non-convergence branches are documented OM-divergent per Pitfall 5)
3. Then ship the README graduation in the v3.0.1 release commit per the original D-32.6 embed-coverage-output discipline

**Action taken:** The CLAUDE.md + PROJECT.md updates DO graduate Phase 32 narrative to shipped (the test/CI infrastructure shipped green; Wave 1+2 ALL delivered). The README v3.0 line UNCHANGED. The CLAUDE.md `### v3.0 additions` header line updated from `28–30 — 31–32 IN PROGRESS` to `28–32 — README hard-claim DEFERRED pending coverage gate` to make the disposition transparent to readers. PROJECT.md `Active (v3.0)` block notes "Coverage gate ≥ 95 % UNMET (91.74 % lines / 92.14 % regions) — README v3.0 hard-claim graduation DEFERRED to v3.0.1 per D-30.9 gate-conditional discipline" so the gap is auditable.

**Tracked as:** v3.0.1 follow-up milestone (not yet planned). The coverage gap is structural, not a Plan-32-03-execution defect — Plan 32-03 fully delivered its in-scope deliverables (E2E smoke, contamination guard, docs update); the README graduation precondition was a downstream gate that Wave 1+2 collectively did not close.

### Auto-fixed Issues

**1. [Rule 3 - Blocking Issue] macOS case-insensitive filesystem masked Justfile (capital J) as `justfile`**

- **Found during:** Task 32-03-04 first commit attempt
- **Issue:** I edited `justfile` (lowercase) via the Edit tool. macOS HFS+/APFS case-insensitively routed the write to the git-tracked `Justfile` (capital J), but `git add justfile` did not match the tracked path, so the first commit (`4fa66e4`) contained only the `.github/workflows/ci.yml` change and missed the Justfile recipe addition.
- **Fix:** Followed up with `git add Justfile` (correct case) and a second case-fix commit (`08aa8a0`) carrying the missing recipe. Both commits ship Plan 32-03 Task 4 deliverables together. Future tooling note: when editing files on macOS, the canonical-case form (per `git ls-files`) is what `git add` matches.
- **Files modified:** `Justfile`
- **Commit:** `08aa8a0`

## Quality Gates

- ✅ All 3 `it()` blocks in `hp41-gui/e2e/smoke.spec.js` (existing + sinh + MATRIX DET) — verified via grep.
- ✅ `bash scripts/check-free42-contamination.sh` exits 0 ("OK: no Free42 contamination detected").
- ✅ `just license-audit` exits 0 (recipe wires to the script correctly).
- ✅ Justfile `ci` recipe extended to `ci: lint test coverage license-audit` per D-32.8.
- ✅ `.github/workflows/ci.yml::license-audit` parallel job present (sibling to lint/test/coverage/msrv, no `needs:`, `ubuntu-latest`).
- ✅ All 13 `hp41-core/src/ops/math1/*.rs` files carry the verbatim disclaim header per Pitfall 19 (already in place from Phase 28; verified by `grep -L` returning empty).
- ✅ No source changes to `hp41-core/src/`, `hp41-gui/src-tauri/src/`, or `hp41-gui/src/` (SC-4 invariant preserved; `hp41-gui/src/` is outside SC-4 boundary but Phase 32 didn't touch it).
- ✅ `wdio.conf.cjs` UNCHANGED per acceptance criterion (zero git diff).
- ✅ `Display14Seg.tsx` UNCHANGED per acceptance criterion (zero git diff).
- ✅ CLAUDE.md Phase 32 stub replaced with full shipped subsection (grep returns 0 in-progress markers; 4 D-32.X / Pitfall 19 mentions).
- ✅ PROJECT.md Phase 32 graduated into the v3.0 history block.
- ⚠ Coverage gate ≥ 95 % UNMET (91.74 % lines / 92.14 % regions) — README hard-claim graduation DEFERRED per Rule 4. Documented in CLAUDE.md + PROJECT.md + this SUMMARY.
- ⚠ `just gui-e2e` was NOT run locally in this session (the worktree does not have webkit2gtk-driver / xvfb installed locally; CI runs the smoke on Ubuntu via `ci-gui.yml::e2e-linux`). The two new tests are validated against the modal-pipeline + xrom_resolve code paths via the reconnaissance in Task 32-03-01; their CI runtime validation will surface on the next push to PR.

## Known Stubs

None — all new code is functional. The E2E test fallback (`browser.execute` invoking `__TAURI_INTERNALS__.invoke`) is the documented workaround for the xeq_name modal alphaChar/Enter collision per RESEARCH.md Open Question #1; not a stub but an architectural choice.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: e2e-bypass | hp41-gui/e2e/smoke.spec.js | The `browser.execute` fallback dispatches `dispatch_op` directly through `window.__TAURI_INTERNALS__.invoke`. Mitigated by T-32-03 (limit fallback to XEQ-by-name submit + document strategy choice in leading comment). The MATRIX test mixes real-click R/S submits between matrix elements to keep the modal-pipeline + R/S routing surface exercised even though the program-name dispatches use the fallback. The frontend modal UX is still exercised by Vitest unit tests in `hp41-gui/src/`. |

## Performance

- **Duration:** ~75 minutes
- **Started:** 2026-05-18T~13:30Z
- **Completed:** 2026-05-18T~14:45Z
- **Tasks:** 5 (1 reconnaissance + 4 implementation)
- **Files modified:** 5 (`smoke.spec.js`, `check-free42-contamination.sh` NEW, `Justfile`, `ci.yml`, `CLAUDE.md`, `PROJECT.md`)

## Self-Check: PASSED

**Created files exist:**
- ✅ `scripts/check-free42-contamination.sh` — verified present + executable (`test -x` passes)

**Commits exist:**
- ✅ `69877cd` test(32-03): extend E2E smoke with Math Pac I sinh + MATRIX DET workflows
- ✅ `ab84485` feat(32-03): add Free42 GPL-contamination guard script
- ✅ `4fa66e4` chore(32-03): wire license-audit into just ci + ci.yml parallel job
- ✅ `08aa8a0` chore(32-03): wire license-audit recipe into Justfile (case-fix follow-up)
- ✅ `4c72ed0` docs(32-03): ship Phase 32 narrative; defer README hard-claim per gate

**Verification commands (all PASS):**
- `grep -c "^    it(" hp41-gui/e2e/smoke.spec.js` → 3
- `grep -c "5.0000\|1.1752\|-2.0000" hp41-gui/e2e/smoke.spec.js` → 14
- `test -x scripts/check-free42-contamination.sh` → 0
- `bash scripts/check-free42-contamination.sh` → "OK: no Free42 contamination detected" + exit 0
- `grep -E '^ci:' Justfile` → `ci: lint test coverage license-audit`
- `grep -c 'license-audit:' .github/workflows/ci.yml` → 1
- `grep -A 5 'license-audit:' .github/workflows/ci.yml | grep -c 'needs:'` → 0
- `grep -L 'Free42 source consulted only as sanity-check oracle' hp41-core/src/ops/math1/*.rs` → empty (13/13 files carry the header)
- `grep -c 'Phase 32.*in progress' CLAUDE.md` → 0 (stub replaced)
- `grep -c 'D-32.5\|D-32.6\|D-32.7\|Pitfall 19' CLAUDE.md` → 4
- `grep -c 'Phase 32' .planning/PROJECT.md` → 4

---
*Phase: 32-test-hardening*
*Completed: 2026-05-18*
