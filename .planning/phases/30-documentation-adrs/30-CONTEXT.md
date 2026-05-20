# Phase 30: Documentation & ADRs — Context

**Gathered:** 2026-05-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 30 is the documentation-only ship that locks the v3.0 Math Pac I narrative in `docs/` and the repo-root briefing files (`README.md`, `PROJECT.md`, `CLAUDE.md`). Three concrete deliverables:

1. **`scripts/docs-matrix` extension + new matrix file.** Justfile recipes `docs-matrix` and `docs-matrix-check` invoke the existing single-input binary twice — once for `hp41cv-functions.json → hp41cv-function-matrix.md` (unchanged) and once for `hp41-math1-functions.json → hp41-math1-function-matrix.md` (new ~55-entry file). Binary signature stays 1-in/1-out (D-30.1). The new Math Pac I matrix renders an additional XROM column showing `Math 1 / 7-N` per entry (D-30.3).
2. **Three ADR write-ups + divergence-catalog expansion.** ADR-001 (Op-strategy A), ADR-002 (user-callback strict-reject), ADR-005 (separate JSON-pipeline file) ship under `docs/adr/v3.0-{001,002,005}-*.md` mirroring the long-form Context/Decision/Consequences/Alternatives-Considered template of existing ADR-003 + ADR-004 (D-30.6); rejected alternatives quote the original Phase 28 CONTEXT.md options verbatim and add OM/community citations per Pitfall 18 (D-30.7). `docs/hp41-math1-divergences.md` expands from 3.1K seed to a numbered catalog (D-30-NN IDs) covering three buckets — OM divergences, emulator extensions, behavioral policies (D-30.4) — using a 5-field entry shape: OM citation, Our behavior, OM behavior, Rationale, See (D-30.5).
3. **README v3.0 soft-claim + `v3.0 additions` block.** README gains a conservative soft-claim "Math Pac I behavioral emulation included (10 top-level programs, ~55 XEQ entry points, documented divergences)" with link to `docs/hp41-math1-function-matrix.md` (D-30.9); hard claim is deferred to Phase 32 conditional on coverage gate ≥ 95% per QUAL-01 — same gating discipline as the v2.2 "feature-complete HP-41CV" claim. `PROJECT.md` and `CLAUDE.md` gain an in-progress `v3.0 additions` block now (D-30.8) — Phase 28 and Phase 29 subsections fully populated, Phase 31 + Phase 32 subsections marked `(in progress)` with stub headers; future Phase 31/32 ship just fills its own subsection.

**In scope:**
- `scripts/docs-matrix` binary unchanged; justfile recipes `docs-matrix` and `docs-matrix-check` get a second invocation
- `docs/hp41-math1-function-matrix.md` (new ~55-entry file, generated from `docs/hp41-math1-functions.json` via `just docs-matrix`)
- XROM column in the Math Pac I matrix only (hp41cv matrix unchanged)
- `docs/adr/v3.0-001-op-strategy.md` (NEW)
- `docs/adr/v3.0-002-user-callback-policy.md` (NEW)
- `docs/adr/v3.0-005-json-pipeline.md` (NEW)
- `docs/hp41-math1-divergences.md` expansion: numbered `D-30-NN` entries in three buckets (OM Divergences / Emulator Extensions / Behavioral Policies) with 5-field structure each
- `README.md` v3.0 soft-claim line + matrix-link addition
- `PROJECT.md` `v3.0 additions` block (in-progress shape)
- `CLAUDE.md` `### v3.0 additions (Math Pac I Emulation, Phases 28–30 — 31–32 IN PROGRESS)` block, structurally parallel to the existing `### v2.2 additions` block

**Out of scope (explicit):**
- Any `hp41-core/src/`, `hp41-cli/src/`, or `hp41-gui/src/` source changes — Phase 30 is documentation-only
- `docs/hp41-math1-functions.json` authoring or modification — Phase 29 D-29.1 already shipped the file at 452 lines (~55 entries); Phase 30 consumes it read-only
- `docs/hp41cv-function-matrix.md` regeneration — the file is current; Phase 30 does NOT re-render it (the v2.2 path is unchanged)
- Any new XROM-column work in the hp41cv matrix — the column is Math-Pac-I-only (the existing v2.2 file has no XROM column and gains none)
- ADR-003 + ADR-004 — already shipped in Phase 28 (`docs/adr/v3.0-003-inv-epsilon.md` + `docs/adr/v3.0-004-intg-threshold.md`); Phase 30 does NOT modify them
- `scripts/check-free42-contamination.sh` audit script — Phase 32 / QUAL-05 (per-file header policy is documented in Phase 30's divergence/ADR text, but the CI gate script lands in Phase 32)
- Phase 31 GUI subsection content in the `v3.0 additions` block — only the stub header `(in progress)` lands in Phase 30; the body fills in at Phase 31 ship-time
- Phase 32 metrics in the `v3.0 additions` block — coverage gate / numerical accuracy / E2E smoke numbers land at Phase 32 ship-time only
- Hard claim "feature-complete Math Pac I behavioral emulation" — deferred to Phase 32 conditional on coverage gate ≥ 95% per QUAL-01

**Mandated by ROADMAP cross-cutting constraints (lines 35–45 of `.planning/ROADMAP.md`):**
- SC-4 invariant: stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` must return nothing. Phase 30 touches `docs/`, `scripts/docs-matrix/` (justfile only), `README.md`, `PROJECT.md`, `CLAUDE.md` — SC-4 trivially preserved.
- `#![deny(clippy::unwrap_used)]` continues to apply in `hp41-core` (Phase 30 does not touch `hp41-core` source).
- MSRV 1.88 unchanged. Zero new runtime or dev-dependencies in Phase 30.
- Citation provenance (Pitfall 18): every divergence-doc entry and ADR write-up MUST cite OM page-and-example, MoHPC URL, or Mike Sebastian forensic page; no uncited assertions.
- Free42 GPL contamination (Pitfall 19): ADR-002 (user-callback policy) explicitly disclaims Free42 as a source; per-file header comment policy for `hp41-core/src/ops/math1/*.rs` is documented here (the CI enforcement script lands in Phase 32 / QUAL-05).

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / STATE.md / 28-CONTEXT.md / 29-CONTEXT.md (carried forward — NOT re-decided here)

- **C-28.1 (ADR-001 lock):** Op-strategy A — one `Op` variant per Math Pac I function. ADR-001 write-up in Phase 30 quotes Phase 28's rejected Option B (`Op::XromCall(u16)` table dispatch) verbatim and cites the 4-way exhaustive-match invariant rationale.
- **C-28.2 (ADR-002 lock):** User-callback re-entrancy — strict-reject nested INTG/SOLVE/DIFEQ at op entry with `HpError::InvalidOp`. ADR-002 write-up in Phase 30 cites Math Pac I OM 1979 Hardware-Verhalten and explicitly disclaims Free42 as a source per Pitfall 19.
- **C-28.3 (ADR-005 lock):** JSON-pipeline shape — separate `docs/hp41-math1-functions.json` sibling file with identical schema plus `xrom: { module, module_id, function_id }` object per entry. ADR-005 write-up in Phase 30 cites the v2.2 D-25.16/D-25.17/D-25.18 hard-build-blocker pattern as the precedent.
- **C-28.4:** `xrom_resolve` fires LAST in the resolver chain. Documented as background context in ADR-001 but not a separate ADR.
- **D-28.3:** `XEQ "REAL"` is a derived XROM entry point — NOT in Math Pac I OM 1979. Documented as an Emulator Extension in `docs/hp41-math1-divergences.md` (D-30-NN entry).
- **D-28.5:** R/S key submits numeric input in a modal prompt. Documented as a Behavioral Policy in the divergence catalog (OM p.13 citation).
- **D-28.6:** XEQ-by-name only for hyperbolics (and all Math Pac I functions) — no dedicated key bindings. Background context in the README soft-claim narrative and in ADR-001's Consequences section.
- **D-29.1:** `docs/hp41-math1-functions.json` authored in Phase 29 (pulled forward from Phase 30 / DOC-01). Phase 30 consumes the file read-only; matrix regeneration is the only new write to this domain.

### Discussed and decided in this session (D-30.1 — D-30.9)

#### docs-matrix two-input shape

- **D-30.1: Justfile invokes the binary twice; binary signature unchanged.** `scripts/docs-matrix/src/main.rs` keeps its existing 1-in/1-out signature; the two `just` recipes `docs-matrix` and `docs-matrix-check` gain a second `cargo run` line each. Rejected the N-pairs CLI extension because the second invocation adds zero argument-parsing surface, the cold-start cost is irrelevant in a docs path, and any future v3.1 pac (Stat 1, Time, etc.) gets a third invocation without touching the binary. Rejected the hardcoded-paths option because it couples the binary to filenames — future pacs would require a recompile, breaking the build-tooling-stays-data-driven discipline.
  - **Why:** smallest blast radius. v2.2 hp41cv path stays bit-for-bit unchanged (regression-free). Future pacs are O(1) justfile edit, not a CLI redesign. Cargo-startup cost in CI is ≤ 0.5s — negligible vs. the value of leaving working code alone.

- **D-30.2: Two separate matrix files.** `docs/hp41cv-function-matrix.md` (130 entries, unchanged) + new `docs/hp41-math1-function-matrix.md` (~55 entries). README v3.0 soft-claim links specifically to `docs/hp41-math1-function-matrix.md`. Rejected the combined-matrix option because the v2.2 readers (CI gate, users, existing docs links) all expect `hp41cv-function-matrix.md` at its current path. Rejected the index-file-on-top option because it adds one extra surface to maintain for one extra navigation hop — not worth the round trip.
  - **Why:** matches the JSON-pipeline separation locked in C-28.3/ADR-005. Each matrix derives from one JSON and lives next to it. Future pacs add one JSON + one matrix without re-renderings of existing files. Zero v2.2 regression risk.

- **D-30.3: XROM column rendered in the Math Pac I matrix only.** New column `XROM` appears between `Display` and `Category` in `docs/hp41-math1-function-matrix.md`. Renders `Math 1 / 7-N` (module name / module_id-function_id) for every row. The v2.2 `hp41cv-function-matrix.md` does NOT gain an XROM column — built-in HP-41CV functions are not XROM-resolved and have no module_id. Rejected the no-column option because XROM identity is the defining feature of the Math Pac I JSON file; hiding it defeats the bidirectional parity test it asserts. Rejected the combined-Op-column option because mixing two semantic axes inside one column breaks grep tooling and complicates the matrix renderer.
  - **Why:** the matrix becomes a structural drift-catch — a JSON entry missing the C-28.3 `xrom` block renders an empty cell and is immediately visible to a human reviewer. The column is information that exists in the source-of-truth JSON; rendering it is faithful, not synthetic. Per-pac matrices have per-pac shape (with column overlap on the common fields); that's correct.

#### Divergence doc scope/structure

- **D-30.4: Three-bucket scope — OM Divergences + Emulator Extensions + Behavioral Policies.** The expanded `docs/hp41-math1-divergences.md` documents:
  1. **OM Divergences** (numerical/behavioral mismatches with OM-quoted examples): POLY multiplicity-as-cluster (Pitfall 5), INTG threshold tied to DisplayMode (ADR-004), FACT integer-only extension policy (carried from v2.2).
  2. **Emulator Extensions** (functions/behaviors we added that aren't in OM 1979): XEQ "REAL" (D-28.3 — deactivates `complex_mode`).
  3. **Behavioral Policies** (cross-cutting rules that aren't strictly divergences but are decisions worth catalog-citing): strict-reject nested INTG/SOLVE/DIFEQ per XROM-08/ADR-002, modal R/S submit per D-28.5/OM p.13.
  Rejected the OM-divergences-only narrow scope because extensions and policies still need a discoverable home for downstream readers — splitting them across CONTEXT.md and ADRs hides them. Rejected the by-program organization because cross-cutting policies would duplicate under multiple sections.
  - **Why:** one document, three clearly-labeled buckets = one place to look. Reader who hits the matrix or README and asks "what's different?" finds everything in one file with provenance. Future pacs add their own buckets the same way.

- **D-30.5: Numbered entries with 5 fixed fields.** Each entry uses ID `D-30-NN` (Phase 30 origin) and the shape:
  ```
  ### D-30-NN: <Title>
  - **OM citation**: <HP 00041-90034 page + example, or "N/A — emulator extension">
  - **Our behavior**: <one-paragraph description of what the emulator does>
  - **OM behavior**: <one-paragraph description of what OM says/shows>
  - **Rationale**: <why we made this choice>
  - **See**: <Pitfall ref, ADR link, or CONTEXT.md decision ID>
  ```
  Predictable shape supports future grep tooling and matches the citation-provenance discipline (Pitfall 18). Rejected the free-form-prose-per-program option because provenance citations get buried in narrative. Rejected the three-column comparison-tables option because the rationale field is prose-shaped and doesn't fit a table cell well.
  - **Why:** consistency across entries = scannability. Grep `^### D-30-` returns the full catalog. Each entry stands alone and can be cited externally (`docs/hp41-math1-divergences.md#d-30-01`).

#### ADR template depth and sourcing

- **D-30.6: Long-form template mirroring existing ADR-003 / ADR-004.** Each of ADR-001, ADR-002, ADR-005 carries the full structure:
  - `# ADR-NNN: Title`
  - `## Status` (Locked YYYY-MM-DD)
  - `## Context` (background + problem statement)
  - `## Decision`
  - `## Consequences` (positive / negative / neutral subsections)
  - `## Alternatives Considered` (each rejected option with why-rejected)
  - `## Footnotes / References` (OM page + MoHPC URLs + community citations)
  Target ~6–7K each per the v3.0-003/004 precedent. Rejected the shorter template because new ADRs would be inconsistent with the existing two; future archaeologists land in any ADR and find the same shape. Rejected the per-ADR-pick option because mixed depth is a confusing signal ("which ones mattered more?") and ADR-005 (JSON-pipeline) deserves the same treatment as the others — it's the lock that makes parity tests possible.
  - **Why:** consistency = readability + reduced cognitive load. The two existing ADRs set the bar; the next three meet it. Future v3.1+ pacs inherit the same template.

- **D-30.7: Rejected alternatives quote Phase 28 CONTEXT.md verbatim + add OM/community citations.** Each `## Alternatives Considered` section quotes the relevant CONTEXT.md decision block (e.g., 28-CONTEXT.md C-28.1's rejected Option B narrative) in a blockquote, then adds an `**Additional context**:` paragraph with OM page references and any MoHPC / Mike Sebastian / community-forum URLs (Pitfall 18 compliance). Rejected the rewrite-for-ADR-audience option because rewriting risks revisionism — the original deliberation is the ground truth. Rejected the link-plus-summary option because ADRs ship with the repo and should be readable offline / from any clone — relying on inter-file links makes the ADR less self-contained.
  - **Why:** faithfulness to the actual decision history + Pitfall 18 citation compliance. ADRs are historical records; the quote-then-add-citations pattern preserves provenance and lets the ADR stand alone.

#### v3.0 additions block + README claim

- **D-30.8: Add in-progress `v3.0 additions` block now in Phase 30.** Both `PROJECT.md` (in the "Settled Architecture Decisions" area or its repo-specific equivalent) and `CLAUDE.md` (immediately after the `### v2.2 additions (Test Hardening, Phase 27)` block) gain a new section:
  ```
  ### v3.0 additions (Math Pac I Emulation, Phases 28–30 — 31–32 IN PROGRESS)
  ```
  Phase 28 and Phase 29 subsections are FULLY populated (drawing from their respective CONTEXT.md decisions). Phase 30 subsection is populated based on this CONTEXT.md as Phase 30 ships. Phase 31 and Phase 32 subsections appear with stub headers and `(in progress)` markers. Future Phase 31/32 ship updates each its own subsection — the v3.0 framework block stays in place across all three remaining ships. Rejected the defer-everything-to-Phase-32 option because v2.2 precedent is incremental population (the v2.2 block grew across phases 20–27); reconstructing Phase 28/29/30 decisions at Phase 32 ship-time would be lossy. Rejected the stub-with-TBD option because Phase 28 + 29 + 30 facts are already firm — there's nothing to TBD on those subsections.
  - **Why:** matches v2.2 ship pattern (incremental population). Reader of `CLAUDE.md` sees current v3.0 state at-a-glance. Lock-in moments stay readable; future deltas land where they belong.

- **D-30.9: Conservative README soft-claim with link to matrix.** README gains under `## Features`:
  ```
  - Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry
    points, documented divergences)
  ```
  Plus a link under "See" or equivalent navigation section: `docs/hp41-math1-function-matrix.md`. The wording mirrors the v2.2 soft-claim "feature-complete HP-41CV with documented divergences" structure (CLAUDE.md → v2.2 additions → README soft-claim). Hard claim "Feature-complete Math Pac I behavioral emulation" deferred to Phase 32 conditional on coverage gate ≥ 95% per QUAL-01 — same gating discipline as v2.2. Rejected the hard-claim-now option because it skips the coverage-gate condition (consistency-with-v2.2 invariant). Rejected the minimal-one-liner option because it loses the 10/55 numbers that make the claim concrete.
  - **Why:** conservative soft-claim = matches v2.2 precedent + accurate at this point + clear upgrade path at Phase 32. The 10/55 numbers are firm facts (10 top-level programs + ~55 XEQ entry points are the locked Math Pac I inventory from Phase 28); surfacing them up front lets a reader assess fit without clicking through.

### Claude's Discretion

- **Exact placement of the `v3.0 additions` block within `PROJECT.md`:** `PROJECT.md` doesn't carry an explicit `## v2.2 additions` block today — that's a CLAUDE.md convention. Planner picks: either (a) add a parallel block to `PROJECT.md` under "Project Reference" or a new dedicated section, or (b) update `PROJECT.md`'s "Shipped" / "Current focus" lines with v3.0 phase shipping dates and leave the additions-block detail to `CLAUDE.md` only. Recommendation: pattern (b) — `PROJECT.md` is the high-level state file; CLAUDE.md is the deeper-context file. Both should be touched in Phase 30, but `PROJECT.md` gets concise milestone lines while `CLAUDE.md` carries the full `### v3.0 additions` block.
- **ADR-001/002/005 owner/date metadata:** existing ADR-003/004 carry `**Status:** Locked YYYY-MM-DD` + `**Owner:**` lines. Planner picks the lock date — likely the date Plan 28-01 research-prep landed (2026-05-16) since that's when the decisions were actually locked, NOT the Phase 30 ship date. Owner is "Plan 28-01 Task X" for each (X = the specific task number that locked the decision).
- **Divergence catalog entry order:** D-30.5 specifies the entry shape but not the order. Planner picks chronological (by which Phase 28/29 decision locked them) or thematic (group OM divergences together, then extensions, then policies). Recommendation: thematic (already enforced by the three buckets D-30.4 specifies); within each bucket, chronological by Phase 28 decision sequence (D-28.3 before D-28.5, etc.).
- **Whether `just docs-matrix-check` becomes a CI gate in Phase 30:** the v2.2 single-input `docs-matrix-check` is already a CI gate (per CLAUDE.md "v2.2 additions" → "JSON-canonical data flow"). The extended two-input form inherits the same gating. Planner picks whether to surface this explicitly in a Phase 30 CI workflow file change or whether the justfile change implicitly extends the gate (the existing CI step calls `just docs-matrix-check` which will run both invocations after D-30.1 lands). Recommendation: the latter (zero new CI workflow file changes; the existing step picks up the new behavior automatically).
- **MoHPC / Mike Sebastian forensic URLs for ADR citations:** the OM page references are firm (Phase 28 transcribed them during Plan 28-01 research-prep). The community-forum citations are nice-to-have for Pitfall 18 compliance but not all rejected options have a clean forum URL. Planner picks: cite where it strengthens the argument (e.g., MoHPC threads on `XromCall` table-dispatch patterns for ADR-001); omit where no clean source exists. Constraint: no fabricated URLs — if no clean source, the OM citation alone is sufficient.
- **XROM column header label:** D-30.3 specifies the column renders `Math 1 / 7-N`. Header label could be `XROM`, `Module / ID`, `XROM (Module/ID)`, or similar. Planner picks the cleanest label that fits within column-width budgets of the existing markdown matrix layout. Recommendation: `XROM` (matches the C-28.3 schema field name; reader who knows the term lands immediately).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.0 milestone scope, target feature areas, build sequence, key decisions ledger (ADR-001/002/003/004/005 entries already present); Phase 30 updates the "Shipped" / "Current focus" lines per D-30.8 discretion
- `.planning/REQUIREMENTS.md` — 110 v3.0 requirements; Phase 30 maps to DOC-01..07
- `.planning/ROADMAP.md` — Phase 30 section (lines 123–151) — 5 success criteria, 4 plans, notable risks/decisions; cross-cutting constraints lines 35–45
- `.planning/STATE.md` — accumulated context; Phase 30 will update via `gsd-sdk query state.record-session` at workflow end
- `CLAUDE.md` (repo root) — `### v2.2 additions` block is the structural template for the new `### v3.0 additions` block landing in Phase 30 (D-30.8); the JSON-canonical pipeline pattern (D-25.16/17/18) anchors ADR-005

### Prior phase CONTEXT (the decision archaeology Phase 30 documents)

- `.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-CONTEXT.md` — primary source for ADR-001 (C-28.1, lines 42–43), ADR-002 (C-28.2, lines 44–45), ADR-005 (C-28.3, lines 46–47), plus D-28.1..D-28.9 decisions feeding divergence catalog entries
- `.planning/phases/28-…/28-RESEARCH.md` — `Pitfall 18` (citation provenance) + `Pitfall 19` (Free42 GPL contamination) — referenced from ADR-002 and the divergence-doc policy section
- `.planning/phases/29-cli-integration/29-CONTEXT.md` — D-29.1 lock for `docs/hp41-math1-functions.json` authoring timing (Phase 30 consumes the file read-only)

### Existing v3.0 ADRs (the template Phase 30's three new ADRs mirror)

- `docs/adr/v3.0-003-inv-epsilon.md` — ADR-003 (INV-EPSILON); shipped Phase 28; structural template for Phase 30 ADRs per D-30.6
- `docs/adr/v3.0-004-intg-threshold.md` — ADR-004 (INTG convergence threshold); shipped Phase 28; structural template for Phase 30 ADRs per D-30.6

### Existing v3.0 documentation (consumed and/or expanded in Phase 30)

- `docs/hp41-math1-functions.json` — authored Phase 29 D-29.1 (452 lines, ~55 entries); Phase 30 consumes read-only as input to the matrix-regeneration path
- `docs/hp41-math1-divergences.md` — seeded Phase 28 Plan 28-07 (3.1K); Phase 30 expands per D-30.4 + D-30.5 (three buckets, numbered entries, 5-field shape)
- `docs/hp41cv-functions.json` — v2.2 baseline JSON; Phase 30 does NOT modify it but the matrix-regeneration path still runs against it (unchanged output)
- `docs/hp41cv-function-matrix.md` — v2.2 baseline matrix; Phase 30 does NOT modify it but the `just docs-matrix-check` CI gate continues to run against it

### Tooling (the build-data-driven infrastructure Phase 30 extends)

- `scripts/docs-matrix/Cargo.toml` — standalone non-workspace crate (verified empty `[workspace]` stanza); Phase 30 does NOT touch this file
- `scripts/docs-matrix/src/main.rs` — single binary, 1-in/1-out signature; Phase 30 does NOT touch this file (D-30.1 keeps the binary untouched)
- `justfile` — `docs-matrix` and `docs-matrix-check` recipes (existing single-invocation form); Phase 30 extends each recipe with a second invocation per D-30.1
- `Cargo.lock` (under `scripts/docs-matrix/`) — also unchanged

### HP Math Pac I primary source (HP-copyrighted — DO NOT redistribute)

- HP-41C/CV Math Pac Owner's Manual (HP 00041-90034, 1979) — pages referenced per ADR / divergence entry:
  - p.13: "Press R/S to continue" — D-28.5 ground truth (cited from the modal R/S behavioral-policy entry in the divergence catalog)
  - p.65 Example 3 (or wherever POLY's worked example sits — planner transcribes the exact page during the divergence-catalog write): POLY multiplicity-as-cluster citation
  - INTG convergence-threshold pages: already transcribed for ADR-004 (Phase 28); divergence-catalog entry for the threshold tie cites ADR-004
  - User-callback re-entrancy / nested-INTG behavior: cited in ADR-002

### Reference oracles (NOT sources for copying)

- Free42 — `https://thomasokken.com/free42/` and `https://github.com/thomasokken/free42`; PUBLIC GPL source. ADR-002 explicitly disclaims Free42 as a source per Pitfall 19. The contamination-guard CI script lands Phase 32 / QUAL-05.
- MoHPC (Museum of HP Calculators) — `https://www.hpmuseum.org/forum/` — community discussion threads cited in ADR-001 / ADR-002 / ADR-005 where they strengthen the argument; planner picks specific threads at write-time (D-30.7 sourcing discretion)
- Mike Sebastian forensic page — `https://www.rskey.org/~mwsebastian/miscprj/forensic.htm` — referenced where it bears on accuracy claims; not all ADRs need it

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`scripts/docs-matrix/src/main.rs`** — single-binary, single-input/output renderer is already correct for the v2.2 path. Phase 30 extends it via justfile invocation, not by touching the binary (D-30.1). The XROM column rendering (D-30.3) MUST therefore happen inside the existing renderer — most likely via a per-row conditional check (`if entry.xrom.is_some() { render column }`). Planner verifies the renderer's column-iteration shape; if the column is hardcoded, planner adds the conditional. If columns are data-driven, the JSON's optional `xrom` field surfaces automatically.
- **`docs/adr/v3.0-003-inv-epsilon.md` + `docs/adr/v3.0-004-intg-threshold.md`** — both are 6–7K long-form ADRs with the exact template Phase 30's three new ADRs must mirror (D-30.6). The `# ADR-NNN: Title` / `**Status:**` / `**Owner:**` / `**Requirement refs:**` / `**Downstream consumer:**` metadata block + `## Context` / `## Decision` / `## Consequences` / `## Alternatives Considered` section sequence is the structural template.
- **`docs/hp41-math1-divergences.md` seed entry** — Plan 28-07 left a 3.1K starter with the document-philosophy preamble + "Status: First entry added in Plan 28-07. Phase 30 / DOC-04 will expand this document with a comprehensive divergence catalog." statement. Phase 30 expands inline — preamble stays, entries get added per D-30.4 buckets + D-30.5 shape.
- **`CLAUDE.md` `### v2.2 additions (Test Hardening, Phase 27)` block** — structural template for the new `### v3.0 additions` block (D-30.8). The v2.2 block has multiple sub-sections (one per phase 20–27) + a tail note about preserved invariants — Phase 30's v3.0 block follows the same shape.
- **`README.md` v2.2 soft-claim line** — already exists at `## Features` (per CLAUDE.md / v2.2 D-25.17). Phase 30's D-30.9 entry sits parallel to it.

### Established Patterns

- **JSON-canonical pipeline + hard-build-blocker on malformed JSON** (D-25.16/17): documented in ADR-005's write-up as the rationale for the separate-file shape.
- **Justfile as task runner, never `cargo` directly** (CLAUDE.md "Never call `cargo` directly in CI or docs"): D-30.1 enforces this — the justfile change is the entire delivery surface for the matrix-regeneration extension.
- **`just docs-matrix-check` CI drift-catch pattern** (v2.2 Pitfall 8 mitigation): the recipe regenerates to a tmp file and diffs against the committed version. Phase 30's extension means BOTH matrix files are diffed; if either drifts, CI fails. Planner verifies the existing tmp-file naming doesn't collide between the two invocations (likely solved by using JSON-derived basenames).
- **OM citation + Pitfall 18 provenance discipline**: every ADR + divergence entry carries a page-and-example citation. D-30.7 enforces verbatim quotation of CONTEXT.md rejected-options + OM page citations.
- **Phase-only-doc cross-cutting touch** (CLAUDE.md "v2.2 ROADMAP cross-cutting line 205"): Phase 30 follows the v2.2 Phase 25 documentation precedent — touches `docs/`, `scripts/` (justfile only), root markdown files, but no Rust source.

### Integration Points

- **Justfile docs-matrix and docs-matrix-check recipes:** D-30.1 inserts a second `cargo run` line under each recipe. Existing line for hp41cv stays unchanged; new line for hp41-math1 appears below it (same renderer, different JSON + matrix paths).
- **`scripts/docs-matrix/src/main.rs` XROM column rendering:** D-30.3 requires the renderer to emit an XROM column when the input JSON's entries have a non-null `xrom` field. The hp41cv JSON has no `xrom` field on any entry → no XROM column rendered → hp41cv-function-matrix.md output bit-for-bit unchanged. The hp41-math1 JSON has `xrom` on every entry → XROM column rendered. Conditional emission is the single integration point; planner verifies the binary's existing column-iteration logic supports conditional column inclusion (or adds the conditional).
- **`docs/hp41-math1-function-matrix.md` (new file):** appears at the same path-shape as `docs/hp41cv-function-matrix.md`. The first `just docs-matrix` run after D-30.1 lands creates it. README v3.0 soft-claim (D-30.9) links to it.
- **`CLAUDE.md` insertion point for `### v3.0 additions` block:** immediately after the `### v2.2 additions (Test Hardening, Phase 27)` section and before the `## Tech Stack` section. The block format mirrors v2.2 sub-section-per-phase shape.
- **`PROJECT.md` insertion point:** planner's discretion (D-30.8 Claude's Discretion bullet). Likely under "Shipped" lines + a new "Current focus" update.
- **`README.md` insertion point for v3.0 soft-claim:** under `## Features`, next to the existing v2.2 soft-claim line. The link to `docs/hp41-math1-function-matrix.md` lands wherever the existing v2.2 matrix link sits (under "See" or equivalent navigation).
- **CI gate continuity:** the existing CI step that calls `just docs-matrix-check` continues to gate against drift; after D-30.1 lands, the same step runs both invocations and diffs both matrix files. No CI workflow file changes required (Claude's Discretion confirms).

</code_context>

<specifics>
## Specific Ideas

- **XROM column format:** `Math 1 / 7-N` where `7` is the locked module_id (C-28.3) and `N` is the 1-indexed `function_id` from the JSON entry's `xrom` block. The "Math 1" prefix is redundant within this single-pac matrix but anticipates future v3.1+ pacs sharing the column convention.
- **Divergence-catalog ID convention:** `D-30-NN` where NN is the entry order (01, 02, …) within the document. Planner picks the order per the Claude's Discretion bullet (thematic by bucket, chronological within bucket).
- **README soft-claim wording (locked in D-30.9):**
  ```
  - Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry
    points, documented divergences)
  ```
  Plus link `docs/hp41-math1-function-matrix.md`.
- **`### v3.0 additions` block heading (locked in D-30.8):**
  ```
  ### v3.0 additions (Math Pac I Emulation, Phases 28–30 — 31–32 IN PROGRESS)
  ```
  Phases 31 and 32 subsections within the block carry `(in progress)` suffixes until their own ships fill them in.
- **ADR-002 Free42 disclaim wording (Pitfall 19):** the `## Decision` or `## Consequences` section of ADR-002 explicitly states something like "Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979); Free42 source consulted only as sanity-check oracle, not copied." matching the per-file header policy that lands in `hp41-core/src/ops/math1/*.rs` files (CI enforcement script in Phase 32 / QUAL-05).
- **Footnote/citation style:** ADRs use markdown footnote syntax (`[^1]`) for OM page citations and inline links for community URLs — matches the v3.0-003/004 precedent.

</specifics>

<deferred>
## Deferred Ideas

- **`scripts/check-free42-contamination.sh` CI audit script + per-file header enforcement** — Phase 32 / QUAL-05. Phase 30 documents the policy in ADR-002 + divergence catalog; the CI gate ships Phase 32.
- **Free42-contamination guard in CI (`.github/workflows/ci.yml` step)** — Phase 32 / QUAL-05, lands alongside the audit script.
- **Phase 31 GUI subsection inside the `v3.0 additions` block** — populated at Phase 31 ship-time; Phase 30 only lands the stub header.
- **Phase 32 metrics inside the `v3.0 additions` block** (coverage, numerical accuracy, E2E smoke) — populated at Phase 32 ship-time.
- **Hard claim "Feature-complete Math Pac I behavioral emulation" in README and CLAUDE.md** — Phase 32 conditional on coverage gate ≥ 95% per QUAL-01. Phase 30 ships only the soft claim (D-30.9).
- **Future v3.1+ pac documentation pattern** (Stat Pac, Time Pac, Advanced Matrix, Advantage) — when the second XROM module lands, the matrix-renderer XROM column convention and the divergence-catalog three-bucket pattern serve as the template. Not part of v3.0; design lives implicitly in D-30.3 + D-30.4.
- **Index file linking all matrix files** (`docs/function-matrix-index.md`) — rejected in D-30.2 as redundant for a two-file world; could re-emerge as v3.x pacs accumulate. Capture-only, not actionable.
- **MoHPC URL + Mike Sebastian forensic citations across all three new ADRs** — Claude's Discretion: cite where they strengthen the argument, omit otherwise. No fabricated citations.
- **`docs/hp41-math1-function-matrix.md` schema documentation** — the matrix file's column convention (XROM column on Math Pac I, no XROM column on hp41cv) isn't documented anywhere yet beyond this CONTEXT. If a future agent asks "why does this matrix have an extra column?", they need to find this CONTEXT or read the JSON. If/when this becomes a friction point, capture it as a doc-debt item; for now, the CONTEXT + the JSON schema are sufficient.

</deferred>

---

*Phase: 30-documentation-adrs*
*Context gathered: 2026-05-17*
