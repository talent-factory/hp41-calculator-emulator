# Phase 23: ALPHA Operations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 23-alpha-operations
**Areas discussed:** 6-char packed-text encoding, POSA needle source, AROT count & X consumption, ATOX/XTOA edge cases

---

## 6-char packed-text encoding

| Option | Description | Selected |
|--------|-------------|----------|
| Sidecar map on CalcState | `text_regs: BTreeMap<u8, String>` with `#[serde(default)]`. ASTO inserts; ARCL checks sidecar first; numeric STO/op_sto_arith clears the sidecar entry; RCL of a text register pushes 0. Save-file compatible. | ✓ |
| Register enum refactor (RegSlot) | `regs: Vec<RegSlot>` where `RegSlot = Number(HpNum) \| Text(String)`. Semantically cleanest, biggest blast radius. Save-file shape changes. | |
| Sentinel-encoded HpNum | Encode 6 ASCII chars into a Decimal pattern that can't arise from arithmetic. Brittle, tied to Decimal internals, easy to corrupt silently. | |

**User's choice:** Sidecar map on CalcState (Recommended)
**Notes:** Minimal refactor, save-file forward+backward compatible. Trade-off: RCL on a text register returns 0 — accepted because (a) RCL fidelity for multi-char POSA is structurally impossible with our HpNum model regardless, and (b) the sidecar-clearing invariant (D-23.4) keeps the two representations from drifting.

---

## POSA needle source

| Option | Description | Selected |
|--------|-------------|----------|
| X = register index into text_regs | POSA pops X, looks up `text_regs[X]`, searches ALPHA. Multi-char works. NO HP-41 precedent — invented convention. | |
| X = ASCII char code (single-char only) | POSA pops X (0..=127), searches for that single char. Faithful to HP-41CV single-char path. Multi-char deferred. | |
| Hybrid: text_regs wins, else ASCII fallback | Both paths covered, but introduces ambiguous case (X=5 could mean "reg 5" OR "ASCII 5 = ENQ"). | |
| **(Follow-up after deeper HP-41 research)** Single-char now, multi-char deferred to v3.x | After dug into HP-41 NUT format + Free42 source: real multi-char POSA relies on the 56-bit raw register copy that Decimal can't preserve. Ship single-char (genuinely faithful) and defer multi-char to v3.x typed-stack refactor. | ✓ |
| Add stack.x_text shadow channel in Phase 23 | Promote Phase 23 to include the typed-stack refactor. ~2x plan size, may need roadmap change. | |
| Single-char + explicit ADR for v3.x typed-stack design | Same as Recommended, plus write an ADR locking the future typed-stack shape now. | |

**User's choice:** First asked for more information on HP-41CV authenticity. After receiving the analysis (HP-41 NUT 56-bit raw bits, Free42 typed-stack approach, Decimal limitation), chose **"Single-char POSA now, multi-char deferred to v3.x (Recommended)"**.
**Notes:** User's goal is "HP-41CV so faithful as possible". The "X = register index" option was rejected because it's an invention with no HP-41 precedent. The single-char POSA path IS fully faithful to HP-41CV for the common case (X holds a small int). Multi-char POSA is honestly noted as structurally impossible without a typed-stack refactor — deferred to v3.x with the design template (Free42-style `stack.x_text: Option<String>`) recorded in `<deferred>`.

---

## AROT count source & X consumption

| Option | Description | Selected |
|--------|-------------|----------|
| Faithful HP-41CV: read X, trunc, modulo, X preserved | No-param Op::Arot, reads X (preserved, Neutral lift), trunc_int + rem_euclid by len. SC#4 verified. Matches Free42 + HP-41C Owner's Manual. | ✓ |
| Same logic but CONSUME X (Disable lift) | Drop X after reading, lift_enabled=false. Some sources describe AROT this way; canonical sources disagree. | |
| Reject non-integer X with InvalidOp | Stricter safety variant. HP-41 hardware actually truncates silently. | |

**User's choice:** Faithful HP-41CV (Recommended)
**Notes:** Locks in the "AROT preserves X" behavior matching real HP-41CV. Positive N rotates LEFT, negative N rotates RIGHT, both verified against SC#4 wording.

---

## ATOX/XTOA edge cases & lift

| Option | Description | Selected |
|--------|-------------|----------|
| Faithful + safe: empty ALPHA → X=0 with Enable lift; XTOA mod 256, codes >127 → '?' | ATOX always lifts. XTOA truncates X, mod 256: 0..127 → ASCII; 128..255 → '?' placeholder (HP-41 upper-ASCII glyphs Σ, λ, ⊢ not in our String model). | ✓ |
| Faithful + strict: empty ALPHA → X=0 NO lift; XTOA rejects >127 with InvalidOp | Stricter error surface; punitive on non-ASCII data. | |
| Faithful + mod-256 with Unicode passthrough | XTOA mod 256 → Latin-1 char (e.g., code=200 → 'È'). Lossless round-trip but silently mismatches HP-41 upper-ASCII glyphs. | |

**User's choice:** Faithful + safe (Recommended)
**Notes:** Documented divergence in CONTEXT.md `<deferred>`: "HP-41CV upper-ASCII char set (Σ, λ, ⊢ etc.) replaced with ? — v3.x may map to Unicode glyphs". Mid-correction during the question phase: I'd initially mis-stated that ATOX consumes X — corrected after re-reading HP-41C Owner's Manual §11 which says "deletes that character from Alpha" (one char popped from ALPHA, not from X) and pushes the ASCII code onto X with stack lift.

---

## Claude's Discretion

- Exact prgm_display strings for the 6 new variants (`"ARCL nn"`, `"ASTO nn"`, `"ATOX"`, `"XTOA"`, `"AROT"`, `"POSA"`) — planner picks final text matching HP-41 listing conventions.
- Test layout (inline `#[cfg(test)]` mods vs centralized `tests/phase23_alpha.rs` integration suite) — Phase 22 precedent: split per plan with integration suite.
- Whether to extend `hp41-core/src/ops/alpha.rs` (72 lines today) or add a new `hp41-core/src/ops/alpha_ops.rs`. Recommendation: extend if total stays under ~400 lines.
- Whether the sidecar-clearing audit (D-23.4) lives as its own Wave-0 commit before main ARCL/ASTO implementation, or is folded in. Recommendation: separate commit for git-blame clarity (Phase 22 D-22.11.1 precedent).
- Lift-then-push idiom for `op_atox`: use a `Stack` helper or direct field assignment after `apply_lift_effect`. Phase 20's `op_pi` is the canonical precedent — planner reads and mirrors.
- Round-trip property test scope (ASTO+ARCL, AROT+inverse). Planner decides scope.
- Plan structure: 1 monolithic vs 2 split (recommended) vs 3 split. CONTEXT.md D-23.18 suggests 2 plans (23-01 = sidecar + ARCL/ASTO; 23-02 = ATOX/XTOA/AROT/POSA).

## Deferred Ideas

- **Multi-char POSA via typed-stack channel** → v3.x. Design template recorded in CONTEXT.md `<deferred>` (Free42-style `stack.x_text: Option<String>` shadow channel).
- **HP-41 custom upper-ASCII char set** (Σ, λ, ⊢, etc.) → v3.x, possibly bundled with FN-POLISH-01 14-seg LCD font work. `'?'` placeholder is explicit, not permanent.
- **ASTO into hidden registers M/N/O** (synthetic ALPHA) → backlog, requires new `Op::AstoM`/`Op::AstoN`/`Op::AstoO` variants + matching sidecars.
- **Mass-clear of all `text_regs`** → backlog; no HP-41 equivalent exists. `op_clreg` already clears all per D-23.4.
- **Phase 24 IND variants of ARCL/ASTO** (FN-IND-01) → Phase 24's `resolve_indirect()` helper layers IND on top of Phase 23's direct-address forms.
</content>
</invoke>