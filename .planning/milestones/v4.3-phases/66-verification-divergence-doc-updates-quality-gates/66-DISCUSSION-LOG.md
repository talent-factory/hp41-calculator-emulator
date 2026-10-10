# Phase 66 — Discussion Log

**Date:** 2026-06-10
**Mode:** discuss (default, interactive)
**For:** human reference only — downstream agents read 66-CONTEXT.md, not this file.

## Gray areas presented (multiSelect)

Presented 4; user selected 3 (skipped "Fix-vs-document per UNC" as a standalone area — folded into Area 1):
- ✅ UNC evidence standard
- ⬚ Fix-vs-document per UNC (not selected as standalone; disposition still asked once — coupled)
- ✅ Doc-sweep breadth
- ✅ Matrix + gate breadth

## Area 1 — UNC evidence standard

**Q (Evidence):** How to verify UNC-01/02/03 with no OM PDFs in repo?
- Options: Autonomous (rec) / User-in-the-loop gate / Autonomous + flag low-confidence
- **User response (freeform):** "Ich habe soeben einige Handbücher hier abgelegt … docs/manuals/HP-41CV" — dropped the actual OM PDFs into the repo.
- **Resolution:** Reframed evidence standard → verify against the **in-repo primary OM** (HP-41C_Operating_Manual.pdf + hp41c41cv-ohpg-en.pdf), autonomous, with page citations; low-confidence valve retained. → **D-01, D-02**.

**Confirm (git size):** ~640 MB of PDFs — commit vs gitignore?
- **User response (freeform):** "Ich [habe] LFS bereits entsprechend konfiguriert." Verified: `.gitattributes` has `docs/manuals/**/*.pdf filter=lfs`, OM is LFS-tracked.
- **Resolution:** Manuals committed as canonical refs; LFS keeps blobs out of regular history. → **D-04**.

**Q (Disposition):** Fix-vs-document rule when OM confirms divergence (esp. UNC-02 print-gating risk)?
- Options: Fix all OM-confirmed / Fix but UNC-02 doc-only (rec) / Defer fixes, document all three
- **User selected:** **Fix all OM-confirmed** — including UNC-02, accepting print-test rework. → **D-03**.

## Area 2 — Doc-sweep breadth

**Q (Sweep):** How exhaustive beyond the already-flipped D-40-04?
- Options: Comprehensive reconcile (rec) / Minimal (named docs only) / Comprehensive + audit ledger
- **User selected:** **Comprehensive + audit ledger** — mark FGAP-02/03/04/05/07 + UNC-01/02/03 resolved in DIVERGENCE-AUDIT.md, sweep hp41cv-divergences.md, fix README FACT note, + closure ledger (FGAP/UNC → phase → OM citation → status). → **D-05, D-06**.

## Area 3 — Matrix + gate breadth

**Q (Matrix):** What to do with the 7-scenario re-entrancy matrix (12 tests already exist)?
- Options: Verify + map (rec) / Verify + fill gaps / Re-author matrix
- **User selected:** **Verify + fill gaps** — map 7 PITFALLS scenarios → 12 tests; add only genuinely-missing ones. → **D-08**.

**Q (Gates):** Which gates must be green?
- Options: CI-defined + GUI clippy (rec) / CI-defined only / CI-defined + GUI clippy + fmt audit
- **User selected:** **CI-defined + GUI clippy** — all ROADMAP gates + ungated GUI-crate clippy (≥2 known hits = noise), NO cargo fmt on GUI crate. → **D-09, D-10**.

## Close

**Q (Done):** More gray areas or ready for context?
- **User selected:** **Ready for context.**

## Deferred ideas captured
- CATALOG 1 interactive scroll (FGAP-06), SAVED/GETD bbb.eee block control (FGAP-08) — future phases.
- GUI-crate fmt cleanup (~19 files) + GUI clippy pre-existing hits — known debt, not touched.

## Canonical refs added during discussion
- `docs/manuals/HP-41CV/HP-41C_Operating_Manual.pdf` + `hp41c41cv-ohpg-en.pdf` (and EN/DE siblings) — primary OM, LFS-tracked, surfaced by the user mid-discussion. Now the evidence standard.
