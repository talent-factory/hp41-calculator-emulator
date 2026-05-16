# Phase 20: Core Math & Conversions - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `20-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 20-Core-Math-and-Conversions
**Areas discussed:** RND semantics across FIX/SCI/ENG, FACT input validation strategy, PI constant exact value, P→R / R→P implementation path & plan-split

---

## RND semantics across FIX/SCI/ENG

| Option | Description | Selected |
|--------|-------------|----------|
| Extract shared `round_to_display_precision()` helper in `format.rs` | Refactor `format_hpnum()` to call it; `op_rnd()` calls it too. Single source of truth for display + value rounding. Slight refactor risk — must keep 500 numerical_accuracy cases passing. | ✓ |
| Inline rounding in `op_rnd()`, leave `format_hpnum()` untouched | New round function inside `math.rs::op_rnd()`. Zero risk to existing display tests. Two code paths to maintain. | |
| Defer SCI/ENG rounding — implement FIX-only | Round to FIX(n) regardless of current display_mode. Ships a behavioral gap. | |
| Let plan decide — capture as 'investigate during planning' | Researcher proposes the right approach in RESEARCH.md. | |

**User's choice:** Extract shared `round_to_display_precision()` helper in `format.rs`.
**Notes:** Wave-0 prep task — extract + verify all existing tests pass BEFORE any new op work lands.

---

## FACT input validation strategy

### Sub-area 1 — Decimal magnitude limit

| Option | Description | Selected |
|--------|-------------|----------|
| Compute via f64 internally; return `HpError::Overflow` for X ≥ 28 | Use f64 for the multiplication loop, convert back via `Decimal::from_f64`. f64 handles up to ~170! before infinity. Practical cap X ≤ 27 (since 28! exceeds Decimal's ~7.92e28). Documented divergence from HP-41 spec (X ≤ 69). | ✓ |
| Hard-cap at X ≥ 28 → OutOfRange; document as known limit | Pre-flight check; same divergence but earlier rejection. | |
| Implement high-magnitude HpNum extension | Refactor HpNum to carry separate exponent field. Massive scope — v3.0 architecture change. | |
| Defer FACT to a later phase; ship only the other 9 ops in Phase 20 | Drop FN-MATH-08 from Phase 20 scope. | |

**User's choice:** Compute via f64 internally; return `HpError::Overflow` for X ≥ 28.
**Notes:** Pre-existing constraint of HpNum/Decimal, not introduced by Phase 20. Documented divergence + Phase 25 CLAUDE.md note. High-magnitude HpNum option captured in Deferred Ideas.

### Sub-area 2 — Non-integer / negative validation

| Option | Description | Selected |
|--------|-------------|----------|
| Integer check via `x == x.trunc_int()`; negative → Domain; non-integer → Domain | Reuses existing `trunc_int()`. Single `HpError::Domain` for both rejections. Matches `ln(neg)`, `sqrt(neg)` precedent. | ✓ |
| Integer check via `x.inner().fract().is_zero()`; split Domain (neg) / OutOfRange (frac) | Distinguishes error types. HP-41 hardware actually shows "OUT OF RANGE" for non-integer factorial. | |
| String-split on '.' | Same approach as ISG/DSE counter parsing. Defensive but less idiomatic. | |
| Negative integers compute via gamma-like extension | Mathematically incorrect for HP-41 fidelity. | |

**User's choice:** Integer check via `x == x.trunc_int()`; both negative and non-integer return `HpError::Domain`.
**Notes:** Follows existing Domain-error precedent for undefined-math inputs.

---

## PI constant exact value

| Option | Description | Selected |
|--------|-------------|----------|
| `HpNum::rounded(Decimal::from_str("3.141592653589793"))` → X = 3.141592654 | Full f64-precision PI string, `HpNum::rounded()` clamps to 10 sig digits → 3.141592654. Matches HP-41 hardware display. SC-1 wording '3.1415926536' identified as a typo. | ✓ |
| Hardcode `Decimal::from_str("3.1415926536")` literally | Honors ROADMAP SC-1 wording exactly. Diverges from HP-41 hardware display. | |
| Use `std::f64::consts::PI` via `Decimal::from_f64(PI).map(HpNum::rounded)` | Symmetric with `to_radians_f64`. Same result as option 1 in practice. | |
| Defer to plan — researcher confirms HP-41 hardware behavior in RESEARCH.md | Phase researcher cross-references HP-41 Owner's Manual. | |

**User's choice:** `HpNum::rounded(Decimal::from_str("3.141592653589793"))` → X = 3.141592654.
**Notes:** ROADMAP SC-1 needs editorial correction from "3.1415926536" → "3.141592654" (planner flags in PLAN.md, ROADMAP corrected on commit).

---

## P→R / R→P implementation path & plan-split

| Option | Description | Selected |
|--------|-------------|----------|
| f64 bridge (mirror trig); single plan for all 10 ops | P→R/R→P use the same f64 bridge as trig (`atan2`, `sqrt`, `sin`, `cos`). One Phase 20 plan delivers all 10 ops + tests + prgm_display updates. | ✓ |
| f64 bridge; two plans — Plan A (trivial 6) + Plan B (numeric/coord 4) | Plan A: PI, ABS, SIGN, R↑, FRC, MOD. Plan B: RND, FACT, P→R, R→P. Better parallelizable. | |
| HpNum-native P→R/R→P; single plan | Mixed paths — Decimal arithmetic for sqrt, f64 only for atan2/sin/cos. | |
| Three plans — trivial / numeric edge / coord conversions | Most parallelizable but heaviest review burden. | |

**User's choice:** f64 bridge (mirror trig); single plan for all 10 ops.
**Notes:** Single-plan keeps scope cohesive and reviewable. Internal task order: Wave-0 (`round_to_display_precision()` helper) → Op variants + dispatch → per-op implementation → prgm_display sync → unit + integration tests.

---

## Claude's Discretion

- **SIGN-on-ALPHA divergence (D-18):** Not asked as a question; flagged in CONTEXT.md as a documented divergence. Our `CalcState` does not type-distinguish X (always numeric HpNum), so SIGN always returns -1/0/+1 numeric. HP-41 hardware returns 0 when X holds ALPHA data; we don't model that boundary in v2.2. Note for CLAUDE.md "v2.2 additions" in Phase 25.
- **`Op` variant naming** for the 10 new ops — planner picks. CONTEXT.md uses `Pi`, `PolarToRect`, `RectToPolar`, `Rnd`, `Frc`, `Mod`, `Abs`, `Fact`, `Sign`, `Rup` as working names following existing PascalCase brevity precedent.
- **Test placement** — inline `#[cfg(test)] mod tests` (matches existing precedent in `math.rs`) for per-op tests, plus a small integration test in `hp41-core/tests/phase20_math.rs` covering the four ROADMAP success criteria.
- **Whether `op_r_up` lives in `stack_ops.rs` or `math.rs`** — settled as `stack_ops.rs` (D-20), mirrors `op_rdn`.

## Deferred Ideas

- **High-magnitude HpNum representation** — Extending HpNum to support exponents ±99 (HP-41 hardware range). Backlog candidate for v3.0 or a dedicated arithmetic-precision milestone. Would unlock true X ≤ 69 fidelity for FACT, large exponentials, and high-magnitude numerical-accuracy cases. Massive scope — affects every op in `hp41-core` and the Decimal-based serialization format.
- **SIGN-on-ALPHA-typed-X** — Requires introducing a value-type tag on X (numeric vs alpha-data). Likely belongs alongside ARCL/ASTO work in Phase 23, where the alpha-data boundary becomes more relevant.
- **PI value tie-breaker via researcher** — If the SC-1 typo turns out to be intentional (e.g. some HP-41 variant shows 11-digit PI), the planner can request researcher verification against the Owner's Manual. Otherwise D-08 stands.
- **Numerical-accuracy suite extension for the new ops** — Owned by Phase 27 (FN-QUAL-02 in REQUIREMENTS.md). Phase 20 must not extend the suite — only ensure it doesn't regress.

---

## MOD sign-of-result semantic (added 2026-05-13 during plan-checker iteration 1)

Plan-checker iteration 1 surfaced a contradiction in the original D-14: the formula `Y - X*trunc(Y/X)` produces sign-of-**Y**, but the explanatory text said "sign inherits X". The plan inherited both halves and asked the executor to research the truth at execution time.

| Option | Description | Selected |
|--------|-------------|----------|
| Match HP-41 hardware: sign-of-Y, trunc formula | Keep `Y - X*trunc(Y/X)`. Sign follows Y. Matches HP-41C Owner's Manual + Free42 source. Update FN-MATH-06 wording + D-14 wording to match the formula. | ✓ |
| Match D-14's "sign inherits X" wording: floor formula | Change to `Y - X*floor(Y/X)`. Diverges from HP-41 hardware. | |
| Spawn researcher to verify against Free42/manual | Adds latency; user-confirmed hardware semantic is well-known. | |

**User's choice:** Match HP-41 hardware — sign-of-Y, trunc-based formula.
**Effect:** D-14 updated in-place; FN-MATH-06 updated in `.planning/REQUIREMENTS.md`. Example clarifications added: `7 MOD -3 = 1`; `-7 MOD 3 = -1`. Removes the executor-research punt from the plan.
