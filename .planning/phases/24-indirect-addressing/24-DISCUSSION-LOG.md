# Phase 24: Indirect Addressing — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 24-indirect-addressing
**Areas discussed:** resolve_indirect API, Phase 22 refactor scope, FlagTest IND shape, Wave/plan structure

---

## Area A — resolve_indirect() API

| Option | Description | Selected |
|--------|-------------|----------|
| Option 1 | Helper validates pointer (regs.get + trunc + non-integer-reject + u8::try_from); returns `u8`. Caller delegates to existing `op_sto`/`op_sf`/etc. with the resolved addr — bounds + sidecar (D-23.4) + atomicity inherit gratis. ~2 lines per IND-variant. | ✓ |
| Option 2 | Two helpers (`resolve_indirect_reg` / `resolve_indirect_flag`) with built-in bounds checks. Caller indexes directly into state.regs. Violates ROADMAP "single shared resolver" principle. | |
| Option 3 | Helper returns `Decimal` (validated integer); callers convert to u8 themselves. Breaks ROADMAP-locked signature `-> Result<u8, HpError>`. | |

**User's choice:** Option 1 — Helper validates pointer, Caller delegiert (Recommended).
**Notes:** User initially asked for a more detailed explanation before deciding ("Hier brauche ich noch eine nähere Beschreibung von dir"). Once the three options were laid out with concrete code samples and the load-bearing reason (sidecar + atomicity inheritance) was made explicit, Option 1 was locked. This is **D-24.1 – D-24.4** in CONTEXT.md.

---

## Area B — Phase 22 refactor scope (Op::GtoInd / Op::XeqInd)

| Option | Description | Selected |
|--------|-------------|----------|
| Partial refactor | Extract a private `resolve_indirect_decimal(state, reg) -> Result<Decimal, HpError>` inner helper. Phase 24's `resolve_indirect()` wraps it + `u8::try_from`. Phase 22's `Op::GtoInd` / `Op::XeqInd` wrap it + `.inner().to_string()` for `find_in_program`. ONE source of pointer-validation truth, two output shapes. | ✓ |
| Leave Phase 22 untouched | `resolve_indirect()` only serves the ~15 new u8-returning IND variants. Accepts duplication of pointer logic across 2 sites. Zero risk for freshly-shipped Phase 22 code. | |
| Full refactor | GtoInd/XeqInd consume `resolve_indirect()` (u8) and stringify it. HP-41 labels go 00–99 so u8 fits semantically — but conflates "register index" semantic with "label lookup" semantic. | |

**User's choice:** Partial refactor — inner helper für Decimal, dann zwei Wrapper (Recommended).
**Notes:** Captured as **D-24.5** in CONTEXT.md. The regression-test requirement (24-01 must include sentinel coverage of Phase-22 GTO IND / XEQ IND end-to-end) is explicit.

---

## Area C — FlagTest IND variant shape

| Option | Description | Selected |
|--------|-------------|----------|
| New variant | `Op::FlagTestInd { kind: FlagTestKind, ind_reg: u8 }`. Mirrors Phase-22 `GtoInd(reg)` precedent. Backward-compat preserved (existing `Op::FlagTest { kind, flag }` unchanged). Single new variant covers all four FlagTestKind sub-cases via kind reuse. | ✓ |
| FlagOperand enum | Generalize the existing `flag` field to `FlagOperand::Direct(u8) \| Indirect(u8)`. Structurally elegant but breaks v2.0/v2.1/v2.2 save files — needs custom serde adapter. | |
| Four flat variants | `Op::FsInd(u8)` / `Op::FcInd(u8)` / `Op::FsCInd(u8)` / `Op::FcCInd(u8)`. No kind reuse. More variants, but trivial prgm_display rows. | |

**User's choice:** New variant `Op::FlagTestInd { kind, ind_reg }` (Recommended).
**Notes:** Captured as **D-24.6** in CONTEXT.md.

---

## Area D — Wave / plan structure

| Option | Description | Selected |
|--------|-------------|----------|
| Two-plan | 24-01 (Wave 1): inner `resolve_indirect_decimal` + `resolve_indirect` u8-wrapper + Phase-22 refactor (GtoInd/XeqInd onto inner helper) + regression suite. 24-02 (Wave 2): ~12 new IND-variants delegating to direct ops. Clean foundation/variants separation. | ✓ |
| Single-plan with per-variant TDD | All in 24-01: helper, refactor, variants. ~15 RED + 15 GREEN + foundation = ~30 commits in one plan. Long but atomic. | |
| Per-category split (4 plans) | 24-01 foundation, 24-02 regs IND, 24-03 flags IND, 24-04 alpha/display IND. All variant-plans touch `ops/mod.rs` + 2× `prgm_display.rs` → wave-sequential anyway, no parallelism benefit. | |

**User's choice:** Two-Plan: 24-01 Helper+Decimal-Refactor, 24-02 alle IND-Variants (Recommended).
**Notes:** Captured as **D-24.8 / D-24.9** in CONTEXT.md. Both plans touch `program.rs`; 24-02 has `depends_on: [24-01]` and forces wave-sequential.

---

## Claude's Discretion

Three implementation details where the planner has flexibility (per CONTEXT.md):

- **Inner-helper module location:** new `hp41-core/src/ops/indirect.rs` recommended; planner may inline into `ops/mod.rs` if the file would be too small.
- **`op_sto_arith_ind` arity:** single `Op::StoArithInd(u8, StoArithKind)` (mirrors `StoArith`) vs four flat variants. Default = single with kind reuse.
- **Regression-test scaffolding location:** 24-01 holds the GTO/XEQ-IND regression; 24-02 holds the new-IND-variant suite. Planner may add a small `phase24_helper.rs` if inner-helper unit tests grow large.

## Deferred Ideas

- Proptest sweep for `resolve_indirect` → Phase 27 (Test Hardening) per ROADMAP cross-cutting constraint
- CLI keyboard wiring for new IND prompts → Phase 25 (CLI Integration)
- GUI `key_map.rs` registration for new IND variants → Phase 26 (GUI Polish)
- HP-41CV function matrix entries for each IND variant → Phase 25 doc block (FN-DOC-01..04)
- Indirect-of-indirect (`STO IND IND nn`) → permanently out of scope (HP-41 hardware does not support it)
