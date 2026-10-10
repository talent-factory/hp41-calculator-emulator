# Phase 65: Standalone Fidelity Fixes - Discussion Log

**Date:** 2026-06-07
**Mode:** discuss (default, interactive)

> Human-reference audit trail. NOT consumed by downstream agents — see 65-CONTEXT.md for the locked decisions.

## Area selection

**Presented (multiSelect):** MATH-01 (FACT range), DISP-01 (CLI view display), DISP-03 (AON auto-display), DISP-02 (CHS in entry).
**Selected:** all four.

Pre-discussion finding surfaced: `HpNum` is `rust_decimal`-backed (~7.92E28 ceiling), but FACT(69)≈1.7E98 — the MATH-01 success criterion is not achievable for 28..69 without changing number representation. This reframed MATH-01 from a "fix" to an architectural decision.

## MATH-01 — FACT range

**Q1: Given the Decimal ceiling, how should MATH-01 be approached?**
- Options: Partial + document divergence / **Extend HpNum range (escalate)** / Saturate-clip-to-max (advised against).
- **Selected:** Extend HpNum range — full ±9.999999999E±99.

**Q2: How should the range-extension be sequenced relative to Phase 65?**
- Options: Split into its own phase (recommended) / **Keep MATH-01 in Phase 65**.
- **Selected:** Keep in Phase 65 — phase absorbs the architectural change.

**Q3: How should the extended range be represented internally?**
- Options: Decimal mantissa + wider exponent / f64 fallback above ceiling / **Let research recommend**.
- **Selected:** Let research recommend — CONTEXT locks the constraints (≥E99, preserve 10-sig fidelity, serde back-compat), researcher picks representation + writes ADR.

## DISP-01 — CLI VIEW/AVIEW/PROMPT display

**Q: When should the CLI revert from a display override to the normal display?**
- Options: **Mirror GUI (persist until cleared)** / Hardware clear-on-keypress (breaks parity).
- **Selected:** Mirror GUI — priority `clock > stopwatch > entry_buf > prgm > display_override > alpha > X`; CLI==GUI.

## DISP-03 — AON auto-display

**Q: CLI only, or both frontends?**
- Options: **Both CLI + GUI (parity)** / CLI only (leaves a gap).
- **Selected:** Both — flag-48 read on each frontend; precedence below the VIEW override, replacing the X fallback at rest (`flag48 ? alpha : X`).

## DISP-02 — CHS during entry

**Q: CHS when the entry buffer is empty?**
- Options: **Keep: negate X (unchanged)** / Discuss further.
- **Selected:** Keep negate-X for the empty-buffer path; the in-buffer leading-`-` toggle (mantissa, no `e`) is additive, mirroring the EEX-CHS pattern at app.rs:691-707. No flush, no lift.

## Final gate

**Q: Ready for context or explore more?**
- **Selected:** Ready for context.

## Deferred ideas
- Splitting the range extension into its own phase — considered, rejected (kept in 65).
- Other FGAP items (CATALOG dump, SAVED/GETD block control) and UNC-01/02/03 verification → later phases / Phase 66.

## Claude's discretion items
- Extended `HpNum` field/enum shape (follows research + ADR).
- FACT(27..69) test-fixture strategy + recalibrating the X≤26 proptest magnitude wall.
- GUI AON reads existing `CalcStateView.flags` (no new IPC projection).
