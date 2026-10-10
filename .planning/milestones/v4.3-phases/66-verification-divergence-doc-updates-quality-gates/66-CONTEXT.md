# Phase 66: Verification, Divergence-Doc Updates, Quality Gates - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Close out the **v4.3 Hardware Fidelity** milestone. Three distinct, sequenced workstreams — **no new user-facing capability**:

1. **Verify UNC-01/02/03 against the primary OM** (now in-repo, LFS-tracked) and **fix any OM-confirmed divergence** this phase.
   - **UNC-01** — pressing `←` (back-arrow) when an error is displayed clears the error and returns to normal display (real HP-41). Emulator: `backspace_entry()` CLXes on empty `entry_buf` but `app.message` may persist. Verify + fix if divergent.
   - **UNC-02** — system flags 21 (printer connected) / 25 (printer enable/trace) gate `PRX`/`PRA`/`PRSTK` printing. Emulator's `print.rs` does NOT check flags 21/25 before printing. Verify + fix if divergent (accepting print-test rework).
   - **UNC-03** — SIZE reduction shows "MEMORY LOST" on the display. `op_size` truncates correctly but sets no `display_override`. Verify + fix if divergent.
2. **Comprehensive divergence-doc reconciliation** — every doc touched by v4.3 work reflects the shipped reality, plus a closure ledger.
3. **All quality gates green** + v4.3 milestone PR description updated.

**Requirements:** VERIFY-01.

**In scope:** OM-based verification of the three UNC items and fixes for any confirmed divergence; full divergence-doc sweep + audit ledger; the 7-scenario re-entrancy matrix coverage proof (verify + fill gaps); the full gate suite incl. ungated GUI-crate clippy; PR description update. The newly-added OM manuals become committed canonical refs (Git LFS).

**Out of scope:** any new `Op` variant or feature; the deferred FGAP items not closed by v4.3 (CATALOG interactive scroll FGAP-06, SAVED/GETD bbb.eee block control FGAP-08); a `cargo fmt` pass on the GUI crate (known ~19-file churn — explicitly forbidden here); version bumps (already done — 4.3.0 across all four surfaces).

**Already done before this phase (verify, don't redo):**
- `docs/hp41-time-divergences.md` §D-40-04 is **already flipped to "IMPLEMENTED … Verified in Phase 66"** (Plan 63-05). This phase confirms it and leaves the "Verified" marker truthful.
- `hp41-core/tests/phase_63_interrupting_alarms.rs` **already contains 12 passing test fns** spanning the re-entrancy scenarios. This phase maps them to the 7 PITFALLS scenarios and fills only genuine gaps.
- Phase 64 already flipped FGAP-04 / SYNT-06 (GETKEY) to implemented in `hp41cv-divergences.md`.
</domain>

<decisions>
## Implementation Decisions

### UNC verification — evidence standard & disposition

- **D-01 (OM is the evidence standard, LOCKED):** Verify UNC-01/02/03 **against the in-repo primary OM PDFs** (`docs/manuals/HP-41CV/HP-41C_Operating_Manual.pdf`, `docs/manuals/HP-41CV/hp41c41cv-ohpg-en.pdf`). This supersedes the original audit's "investigate on real hardware / Free42" resolution — the primary OM is now available and is the authoritative source for a faithful-emulation project. **Each UNC classification MUST record a primary-OM page citation.** Free42 is a tie-breaker oracle only, not a primary source.
- **D-02 (autonomous verification, LOCKED):** Verification runs **autonomously** — Claude reads the OM sections, classifies each UNC as `divergent` / `already-correct`, and proceeds without a user gate. **Low-confidence safety valve:** if the OM genuinely does not pin a behavior, mark it `needs-hardware-confirm` in the verification record and document-as-divergence rather than guess-fix; surface it in the summary instead of blocking.
- **D-03 (fix ALL OM-confirmed divergences, LOCKED):** When the OM confirms a UNC is a real divergence, **fix it in this phase** — UNC-01, UNC-02, and UNC-03 alike. The user explicitly accepted that **UNC-02 (gating `PRX`/`PRA`/`PRSTK` on flag 21/25) may require reworking existing always-print tests** to expect flag-gated no-ops; that rework is in scope. An `already-correct` UNC is documented as **verified-correct (not a divergence)** with its OM citation — closure still recorded.
- **D-04 (manuals are committed canonical refs, LOCKED):** The OM/handbook PDFs are tracked via Git LFS (`.gitattributes`: `docs/manuals/**/*.pdf filter=lfs diff=lfs merge=lfs -text`) — already confirmed in place. They are committed (not gitignored); LFS keeps the ~640 MB out of the regular git history. CONTEXT/research cites them **by path + page**.

### Divergence-doc sweep — comprehensive + audit ledger

- **D-05 (comprehensive reconciliation, LOCKED):** Sweep **every doc touched by v4.3**, not just the ROADMAP-named ones:
  - `.planning/research/DIVERGENCE-AUDIT.md` — mark **FGAP-02, FGAP-03, FGAP-04, FGAP-05, FGAP-07** and **UNC-01/02/03** resolved with the closing phase + OM citation + status.
  - `docs/hp41cv-divergences.md` — record the DISP-01/02/03 (VIEW/AVIEW/PROMPT, CHS in-buffer, AON), GETKEY, and any UNC closures as implemented (v4.3).
  - `docs/hp41-time-divergences.md` — confirm §D-40-04 is "Implemented … Verified in Phase 66" and the "Verified" claim is now actually true.
  - `README.md` — update the FACT divergence note (~line 170) to "implemented (v4.3)".
- **D-06 (closure ledger, LOCKED):** Add a **v4.3 closure ledger** — a table mapping each closed item → **FGAP/UNC id → phase that closed it → OM citation → status**. Lives in `DIVERGENCE-AUDIT.md` (or a dedicated v4.3 section). Self-documenting audit trail for the milestone.
- **D-07 (English-only, honored):** All doc prose stays English (project convention). OM page citations are factual references, not translated prose.

### Re-entrancy matrix — verify + fill gaps

- **D-08 (verify + fill gaps, LOCKED):** Build an explicit **7 PITFALLS scenarios → test-fn** mapping against the existing 12 tests in `hp41-core/tests/phase_63_interrupting_alarms.rs`. Run the suite green. **Add a test only for a PITFALLS scenario that turns out NOT to be covered** by the 12 — do not re-author or duplicate. The mapping note (test-file header comment and/or the verification record) is the artifact that satisfies Success Criteria 3 by demonstrated coverage.

### Quality gates — CI-defined + GUI clippy

- **D-09 (gate suite, LOCKED):** Phase closes only when green:
  - `just ci` (lint + 3-OS test + coverage ≥ 95% lines / ≥ 93% regions + license-audit + schema-aliases)
  - `just ci-msrv` (pinned clippy 1.88 — watch the MSRV-vs-stable lint divergence; `uninlined_format_args` etc. are `-D warnings` under 1.88)
  - `just gui-ci`
  - numerical accuracy ≥ 98% (843+ cases, `tests/numerical_accuracy.rs`)
  - zero panics in `hp41-core`
  - **PLUS** the ungated GUI-crate clippy: `cargo clippy --manifest-path hp41-gui/src-tauri/Cargo.toml` — catches the `gui-ci`-has-no-clippy blind spot. **≥ 2 known pre-existing hits (types.rs:148, commands.rs `&*p`) are accepted noise**, not regressions.
- **D-10 (NO GUI fmt pass, LOCKED):** Do **not** run `cargo fmt` on `hp41-gui/src-tauri` — it reformats ~19 pre-existing files (pre-push fmt gate checks the root workspace only). If a stray GUI edit dirties formatting, `git restore hp41-gui/` rather than committing churn.
- **D-11 (PR description, LOCKED):** Update the open v4.3 milestone PR description with the Phase 66 completion summary (per the standing "update PR per shipped phase" practice). Milestone version bump is already complete — not re-done here.

### Claude's Discretion / Planner
- Exact OM page numbers per UNC (researcher locates them in the in-repo PDFs and records citations).
- Whether UNC-02's fix touches only `print.rs` or also a shared flag-check helper; and the precise rework of any always-print tests it destabilizes.
- For UNC-03, whether "MEMORY LOST" is set via `display_override` and how long it persists (mirror existing override-dismiss semantics, D-25.6 parity).
- Where exactly the closure ledger table sits (DIVERGENCE-AUDIT.md vs a new v4.3 section) and its column shape.
- Plan sequencing: verification (gates UNC fixes) → fixes → doc sweep → matrix proof → full gate run → PR update. Doc sweep can parallelize with the matrix proof once UNC dispositions are known.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary OM source (the evidence standard — D-01)
- `docs/manuals/HP-41CV/HP-41C_Operating_Manual.pdf` (17 MB) — primary HP-41C Operating Manual. Locate: error-display / back-arrow behavior (UNC-01), printer system flags 21/25 + PRX/PRA/PRSTK gating (UNC-02), SIZE reduction → "MEMORY LOST" display (UNC-03). **Cite page numbers.**
- `docs/manuals/HP-41CV/hp41c41cv-ohpg-en.pdf` (259 MB) — full HP-41C/CV Owner's Handbook & Programming Guide (English); comprehensive cross-check for the above.
- `docs/manuals/HP-41CV/hp41-standard-en.pdf` — standard applications, secondary reference.
- (DE siblings exist: `hp41-anwenderhandbuch.pdf`, `hp41c41cv-ohpg-de.pdf`, `hp41-standard-de.pdf` — fallback only; cite the EN OM as primary.)
- LFS-tracked via `.gitattributes` (`docs/manuals/**/*.pdf filter=lfs`).

### Fidelity spec + verification ledger targets
- `.planning/research/DIVERGENCE-AUDIT.md` — **UNC-01 (line 153), UNC-02 (line 154), UNC-03 (line 155)** verbatim audit text + the FGAP table (FGAP-02/03/04/05/07). This is where the closure ledger (D-06) and FGAP/UNC resolution marks (D-05) land.
- `.planning/research/PITFALLS.md` — the 7 re-entrancy scenarios to map against the 12 existing tests (D-08).
- `.planning/research/SUMMARY.md` — fidelity-audit overview / grouping.

### UNC fix sites (verify against OM first — D-03)
- **UNC-01:** `hp41-cli/src/app.rs` — `backspace_entry()` / back-arrow path + `app.message` / status-bar rendering; the error message persistence is the suspected divergence.
- **UNC-02:** `hp41-core/src/ops/print.rs` — `PRX`/`PRA`/`PRSTK` implementations; they do not read flags 21/25. Flag storage is the 56-bit `flags: u64` in `state.rs`. Fix may destabilize always-print tests (accepted, D-03).
- **UNC-03:** `hp41-core/src/ops/program.rs` — `op_size` (~line 262, comment "hardware-faithful 'MEM LOST'"); add `display_override = Some("MEMORY LOST")` if OM-confirmed.

### Divergence docs to sweep (D-05)
- `docs/hp41-time-divergences.md` — §D-40-04 (~line 265) already "Implemented … Verified in Phase 66"; confirm the marker is now true. Footer "Last updated" line ~383.
- `docs/hp41cv-divergences.md` — record DISP/CHS/AON (Phase 65) + GETKEY (Phase 64) + any UNC closures.
- `docs/hp41-math1-divergences.md`, `docs/hp41-stat1-divergences.md`, `docs/hp41-advantage-divergences.md`, `docs/hp41-xmem-divergences.md` — scan for any v4.3-affected entries (likely none, but the comprehensive sweep checks).
- `README.md` — FACT divergence note (~line 170) → "implemented (v4.3)".

### Re-entrancy test matrix (D-08)
- `hp41-core/tests/phase_63_interrupting_alarms.rs` — **12 existing test fns** (halts/resumes, stack preservation, 4-level cap, nesting block, idle-fire, non-interrupting event path, message-alarm event path, solver/modal demotion, missing-handler, pending-clear-on-stop, v4.3 backward-compat, repeating reschedule). Map these to the 7 PITFALLS scenarios; add only the missing ones.

### Frozen invariants / discipline (must honor)
- `CLAUDE.md` — no panics (`#![deny(clippy::unwrap_used)]`), no async, `just`-only task running, D-25.6 CLI↔GUI parity (UNC-03 display + any frontend touch), save-file backward compat (no new `CalcState` fields expected here), print-emulation discipline (`println!`/`eprintln!` forbidden in core; PRX/PRA/PRSTK push to `print_buffer`).
</canonical_refs>

<code_context>
## Existing Code Insights

### Already-done assets to verify (not rebuild)
- **`docs/hp41-time-divergences.md` §D-40-04** — already flipped to "IMPLEMENTED … Verified in Phase 66" (Plan 63-05). Phase 66 makes the "Verified" claim true; does not re-flip.
- **`hp41-core/tests/phase_63_interrupting_alarms.rs`** — 12 passing tests already present; the matrix work is mapping + gap-fill, not authoring.
- **`docs/hp41cv-divergences.md`** GETKEY entry — already flipped by Phase 64 (FGAP-04 / SYNT-06).

### UNC fix integration points
- `hp41-cli/src/app.rs` — UNC-01 back-arrow / error-clear path.
- `hp41-core/src/ops/print.rs` — UNC-02 flag-21/25 gating for PRX/PRA/PRSTK.
- `hp41-core/src/ops/program.rs:262` — UNC-03 `op_size` "MEMORY LOST" display.
- `hp41-core/src/state.rs` — `flags: u64` storage (flags 21/25), `display_override: Option<String>`.

### Gate surfaces
- `just ci`, `just ci-msrv`, `just gui-ci` — defined gates.
- `cargo clippy --manifest-path hp41-gui/src-tauri/Cargo.toml` — the ungated GUI clippy added by D-09 (≥2 known noise hits).
- `tests/numerical_accuracy.rs` — ≥98% (843+ cases); UNC-02 print rework must not regress this.
</code_context>

<specifics>
## Specific Ideas

- The phase is **verification-first**: UNC dispositions (divergent vs already-correct) gate whether/what code changes; the doc sweep records the truth either way. Sequence verification before fixes before the doc ledger.
- The OM PDFs are large (LFS); the researcher should page-target rather than full-read — search the OM index/TOC for "MEMORY LOST", printer flags, and back-arrow/error-clear, then read those page ranges (Read supports PDF page ranges, ≤20 pages/request).
- UNC-02 is the one disposition with real blast radius (print-test rework) — the user chose to fix it anyway if the OM confirms; plan the test rework explicitly.
- The 12 re-entrancy tests very likely already subsume the 7 PITFALLS scenarios; expect the gap-fill to be zero or one test, but prove it with the mapping.
- MSRV clippy (1.88) is the gate most likely to surprise — style lints stable demoted to pedantic are still `-D warnings` under 1.88; check `gh pr checks` before declaring gates green.
</specifics>

<deferred>
## Deferred Ideas

- **CATALOG 1 interactive scroll (FGAP-06)** — not closed by v4.3; remains deferred (own future phase).
- **SAVED/GETD bbb.eee block control (FGAP-08)** — deferred; a consequence-of/separate-from the GETKEY work, not in this milestone.
- **GUI-crate `cargo fmt` cleanup (~19 files)** — known debt; explicitly NOT touched here (D-10). Could be its own dedicated formatting-debt phase if ever desired.
- **GUI-crate clippy pre-existing hits (types.rs:148, commands.rs `&*p`)** — accepted as noise this phase; a future cleanup could zero them.
</deferred>
