# Phase 20: Core Math & Conversions - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Land the 10 missing HP-41CV ROM math/stack operations in `hp41-core` with hardware-faithful semantics:
`PI`, `P→R`, `R→P`, `RND`, `FRC`, `MOD`, `ABS`, `FACT`, `SIGN`, `R↑`.

**In scope:** `hp41-core` only — new `Op` variants, dispatch arms, `execute_op` arms, both `prgm_display.rs` copies (`hp41-cli` + `hp41-gui`), unit tests, LiftEffect declarations, `format.rs` helper extraction for RND.

**Out of scope (Phase 25):** keyboard wiring in `hp41-cli/src/keys.rs`, `KEY_REF_TABLE` entries, `help_data.rs` updates, `pending_prompt()` arms.
**Out of scope (Phase 26):** `key_map.rs::resolve` entries, `KEY_DEFS` bindings in `Keyboard.tsx`.
**Out of scope (Phase 27):** extending `numerical_accuracy.rs` 500-case suite — Phase 20 lands op-local unit tests; Phase 27 owns the golden suite.

</domain>

<decisions>
## Implementation Decisions

### RND — value-rounding to display precision
- **D-01:** Extract a shared `round_to_display_precision(x: &HpNum, mode: DisplayMode) -> HpNum` helper in `hp41-core/src/format.rs`. Both `format_hpnum()` (display path) and the new `op_rnd()` (value-mutation path) call it. Single source of truth.
- **D-02:** Refactor `format_hpnum()` to call the new helper internally — must keep all existing display tests + the 500-case `numerical_accuracy.rs` suite passing. Treat the refactor as a Wave-0 prep task before any `Op::Rnd` work.
- **D-03:** Full FIX/SCI/ENG support in v2.2 — no behavioral gap. FIX(n) rounds to n decimal places; SCI(n) rounds to n+1 sig digits; ENG(n) rounds to n+1 sig digits with exponent constrained to multiples of 3. Mirror whatever `format_hpnum()` does today.

### FACT — magnitude limit and validation
- **D-04:** Compute factorial via **f64 internally**, then convert back through `Decimal::from_f64(...).map(HpNum::rounded).ok_or(HpError::Overflow)`. f64 handles up to ~170! before infinity; the `Decimal::from_f64` conversion is what enforces the practical cap.
- **D-05:** Practical magnitude cap is **X ≤ 27** (since `28! = 3.05e29` exceeds Decimal's ~7.92e28 range). For X ≥ 28 (but < 70), `Decimal::from_f64` returns None → return `HpError::Overflow`. **Documented divergence from HP-41 hardware spec of X ≤ 69** — pre-existing constraint of HpNum/Decimal, not introduced by Phase 20.
- **D-06:** For X > 69 (the hardware-spec OutOfRange threshold from SC-3), still return `HpError::OutOfRange` as a pre-flight check — preserves the SC-3 wording. In practice both X≥28 and X>69 land in error; only the error tag differs (Overflow vs OutOfRange).
- **D-07:** Integer-check via `x == x.trunc_int()` (reuses existing `HpNum::trunc_int()`). Non-integer → `HpError::Domain`. Negative integers → `HpError::Domain` (factorial undefined for negative). Same error type as `ln(neg)` / `sqrt(neg)` precedent.

### PI — exact value
- **D-08:** `Op::Pi` pushes `HpNum::rounded(Decimal::from_str("3.141592653589793").expect("PI constant must parse"))` → result is **3.141592654** (10 sig digits, matching HP-41 hardware display).
- **D-09:** **ROADMAP editorial correction needed.** SC-1 currently reads "pushes 3.1415926536 (10-digit rounded)" — that string is 11 sig digits and does not match the HP-41 hardware display. Correct text: "pushes 3.141592654 (10-digit rounded HP-41 hardware value)". Planner should flag this in the PLAN.md cross-references; the ROADMAP can be corrected when the phase commits.
- **D-10:** LiftEffect: **Enable** (PI behaves like any numeric constant push — lifts stack via the standard `enter_number` → `unary_result`-style path, except there's no input to consume; conceptually closer to `LASTX`).

### P→R / R→P — coordinate conversions
- **D-11:** **f64 bridge implementation** — mirror the existing trig precedent (`op_sin/cos/asin/acos/atan` in `math.rs:148–261`). For each op: convert Y and X to f64, compute results in f64 using `atan2`/`sqrt`/`sin`/`cos` and `f64_from_radians`/`to_radians_f64`, then `Decimal::from_f64(...).map(HpNum::rounded)` on each output **once at the end**. Avoids double-rounding for canonical angles (matches the rationale in math.rs:196–208).
- **D-12:** Stack bookkeeping — both `R→P` and `P→R` are binary-input / binary-output ops. They consume X and Y, produce new Y and X. **LASTX gets the consumed X** (standard hardware behavior). Z and T are unaffected (no upward roll). Use direct field assignment (`state.stack.y = new_y; state.stack.x = new_x;`) — neither `unary_result` nor `binary_result` fit this shape exactly. Set `state.stack.lastx = old_x` explicitly. LiftEffect: **Enable**.
- **D-13:** Angle mode honored via `to_radians_f64(angle, state.angle_mode)` for the P→R input and `f64_from_radians(rad, state.angle_mode)` for the R→P output — exactly the helpers already defined in `math.rs`.

### MOD, FRC, ABS, SIGN — straightforward unary
- **D-14:** `MOD` (binary, Y mod X): result = `Y - X * (Y/X).trunc()` where `(Y/X).trunc()` is `HpNum::trunc_int()` (truncate toward zero). **Sign follows Y** (HP-41 truncate-toward-zero convention; matches HP-41C Owner's Manual + Free42 source). Examples: `7 MOD -3 = 1` (sign of Y); `-7 MOD 3 = -1` (sign of Y). FN-MATH-06 wording corrected accordingly in REQUIREMENTS.md on 2026-05-13. LiftEffect: Enable via `binary_result`. Domain error if X = 0.
- **D-15:** `FRC` (unary, complement of `INT`): result = `x - x.trunc_int()`. Sign-preserving per FN-MATH-05 (`FRC(-3.7) = -0.7`). LiftEffect: Enable via `unary_result`.
- **D-16:** `ABS` (unary): result = `x.inner().abs()` wrapped in HpNum. LiftEffect: Enable via `unary_result`.
- **D-17:** `SIGN` (unary): -1 / 0 / +1 for negative / zero / positive X. LiftEffect: Enable via `unary_result`.
- **D-18:** **SIGN-on-ALPHA divergence** — HP-41 hardware returns 0 when "X holds ALPHA data". Our `CalcState` model does not type-distinguish X (always numeric HpNum); `alpha_reg` is a separate field. `SIGN` therefore always returns -1/0/+1 numeric. Document this as a known divergence in `CLAUDE.md` "v2.2 additions" block under Phase 25.

### R↑ — stack roll up
- **D-19:** Mirror of `op_rdn` in `hp41-core/src/ops/stack_ops.rs`. Implementation: `old_x = X; X = T; T = Z; Z = Y; Y = old_x`. **Does not update LASTX** (stack reorganization, not arithmetic — same convention as `op_rdn`). LiftEffect: **Neutral**.
- **D-20:** Add `pub fn op_r_up(state: &mut CalcState) -> Result<(), HpError>` next to `op_rdn` in `stack_ops.rs`. Op variant name: `Op::Rup` (matches existing PascalCase brevity).

### Plan structure
- **D-21:** **Single plan for all 10 ops** in Phase 20 — keeps scope cohesive and reviewable. Internal task order (Wave-0 first, then implementation):
  1. Wave-0 prep: Extract `round_to_display_precision()` helper in `format.rs` (D-01) — must compile + pass existing tests before any new ops land.
  2. New `Op` variants in `ops/mod.rs` + `dispatch()` arms.
  3. New `execute_op()` arms in `ops/program.rs`.
  4. Per-op implementation (math.rs additions + stack_ops.rs::op_r_up).
  5. `prgm_display.rs` exhaustive-match updates in BOTH copies (`hp41-cli` and `hp41-gui`).
  6. Unit tests per op (`#[cfg(test)] mod tests` inside each file) + a small integration test in `hp41-core/tests/phase20_math.rs` covering the four SCs (PI lift, R→P degree/radian, RND FIX 1/FIX 0, R↑ mirror).

### Cross-cutting (locked, not gray)
- **D-22:** Every new `Op` variant lands in 4 places: `dispatch()` + `execute_op()` + both `prgm_display.rs` copies. Compile-time exhaustive matches catch any miss.
- **D-23:** `#![deny(clippy::unwrap_used)]` applies. All new code uses `?`-propagation or `.expect("reason")`. Test modules carry `#[allow(clippy::unwrap_used)]`.
- **D-24:** SC-4 invariant preserved — no `op_*` / `flush_entry_*` / `format_hpnum` added to `hp41-gui/src-tauri/`. Only `prgm_display.rs` exhaustive-match update is allowed there.
- **D-25:** LiftEffect summary: PI=Enable, P→R=Enable, R→P=Enable, RND=Enable (value mutates X, follows unary_result), FRC=Enable, MOD=Enable (binary_result), ABS=Enable, FACT=Enable, SIGN=Enable, R↑=Neutral.

### Claude's Discretion
- Exact location of unit tests per op (inline `#[cfg(test)]` mods vs centralized) — planner decides; precedent is inline.
- Whether `op_r_up` lives in `stack_ops.rs` (D-20) or `math.rs` — settled in D-20 as `stack_ops.rs` (mirrors `op_rdn`).
- Naming for `MOD` (`Op::Mod` vs `Op::ModOp` to avoid shadowing the `mod` keyword) — planner picks; `Op::Mod` should work (it's a variant identifier, not a module name).
- Whether `Op::Pi` is dispatchable both bare and as a program step — yes by default per existing patterns (every Op variant is both interactive and programmable unless explicitly modal).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level anchors
- `.planning/PROJECT.md` — build sequence (core → cli → docs → gui → tests), shipped milestones, architectural invariants.
- `.planning/REQUIREMENTS.md` §13–25 (Core Math & Conversions FN-MATH-01..09, FN-STACK-01) — the 10 requirements this phase delivers.
- `.planning/ROADMAP.md` §35–50 (Phase 20 details, success criteria, cross-cutting constraints).
- `.planning/STATE.md` §Key Decisions — all settled v1.0/v1.1/v2.0/v2.1 architecture decisions (BCD/f64, stack-lift, LiftEffect-per-op, zero-panic policy).
- `CLAUDE.md` §"Settled Architecture Decisions / Core engine (v1.0)" — BCD/f64 rationale, LiftEffect contract, the 4-place Op-variant rule, `unwrap_used` deny.

### Code references that constrain Phase 20
- `hp41-core/src/ops/math.rs:1–285` — the file Phase 20 will extend. Lines 17–56 define `pi_over_180()`, `pi_over_200()`, `to_radians_hpnum`, `to_radians_f64`, `f64_from_radians` — **reuse**, don't reinvent. Lines 196–208 explain the f64-bridge rationale that P→R/R→P MUST follow.
- `hp41-core/src/ops/stack_ops.rs:45–57` — `op_rdn` is the exact mirror template for the new `op_r_up`. LiftEffect = Neutral, does not touch LASTX.
- `hp41-core/src/stack.rs` — `apply_lift_effect()`, `unary_result()`, `binary_result()`, `LiftEffect` enum. P→R/R→P will need direct stack assignment (D-12) since neither `*_result` helper fits the binary-out shape.
- `hp41-core/src/num.rs:213–226` — `HpNum::trunc_int()`, `negate()`, `inner()`. FRC uses `trunc_int()`; FACT uses `x == x.trunc_int()` for integer validation.
- `hp41-core/src/num.rs` (full) — `HpNum::rounded(decimal)` is the single rounding entrypoint; all new ops route their final Decimal result through it once at the end (matches trig pattern).
- `hp41-core/src/format.rs` — current home of `format_hpnum()`. D-01 extracts `round_to_display_precision()` here.
- `hp41-core/src/state.rs:42–47` — `DisplayMode` enum (Fix/Sci/Eng with u8 digit count). RND consumes this.
- `hp41-core/src/state.rs:61–62` — `angle_mode: AngleMode`. P→R/R→P consume this.
- `hp41-core/src/error.rs` — `HpError` variants. New code uses existing `Domain` (FACT non-integer/neg, MOD div-by-0), `Overflow` (FACT magnitude cap via Decimal::from_f64 failure), `OutOfRange` (FACT X>69 pre-flight per SC-3).
- `hp41-core/tests/numerical_accuracy.rs` — must remain ≥ 490/500 passing after Phase 20 lands. Phase 27 will extend the suite; Phase 20 must not regress it.

### Prior-phase decisions that flow forward
- Phase 2 (Core Math): f64 bridge pattern for trig is the precedent that locks D-11. The "single Decimal::from_f64 + HpNum::rounded at the end" rule is the most important contract for Phase 20.
- Phase 9 (Infrastructure & EEX Fix): the "no `unwrap()`, all `.expect("reason")` or `?`" pattern. Every new piece of code in Phase 20 obeys this.
- Phase 11 (Print Emulation): no I/O in `hp41-core`. None of the 10 new ops have side-channel output, so this is trivially preserved.
- Phase 18 (Program Listing & CI/CD): `prgm_display.rs` lives in TWO copies (cli + gui) and both have exhaustive matches. The 4-place Op-variant rule from CLAUDE.md §Critical Implementation Traps.

### External reference (HP-41 hardware spec)
- HP-41C/CV Owner's Manual — for SIGN-on-ALPHA semantics (D-18), FACT X=70 OutOfRange wording (D-06), MOD sign convention (D-14), R→P / P→R quadrant conventions (D-11/D-13). Researcher should cross-reference if any of these decisions need verification, but the locked-in decisions above are sufficient for planning.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`to_radians_f64(value, mode)`** (`math.rs:50`) — converts an f64 from current angle_mode to radians. Reused by P→R input-angle handling.
- **`f64_from_radians(rad, mode)`** (`math.rs:210`) — converts an f64 from radians back to current angle_mode. Reused by R→P output-angle handling.
- **`unary_result(state, result)`** (`stack.rs`) — handles LASTX-save + lift for unary ops. Used by ABS, SIGN, FRC, RND.
- **`binary_result(state, result)`** (`stack.rs`) — handles LASTX-save + Y-drop + lift for binary ops that produce a single result. Used by MOD.
- **`apply_lift_effect(state, LiftEffect::Neutral)`** (`stack.rs`) — used by R↑ (mirror of `op_rdn` precedent).
- **`HpNum::trunc_int()`** (`num.rs:224`) — integer truncation toward zero. Used by FRC (`x - x.trunc_int()`) and FACT (`x == x.trunc_int()` integer check).
- **`HpNum::rounded(decimal)`** (`num.rs`) — single rounding entrypoint. Every new op routes its final Decimal through this once at the end.
- **`Decimal::from_f64(f)`** (rust_decimal) — returns `None` on out-of-range. Used by FACT for the practical magnitude cap (D-04, D-05).
- **`op_rdn`** (`stack_ops.rs:49`) — exact template for `op_r_up`. Just reverse the assignment chain.

### Established Patterns
- **f64 bridge pattern** (math.rs:148–261, 196–208): every op that needs irrational results uses `Decimal → f64 → math → f64 → Decimal::from_f64 + HpNum::rounded` exactly once at the end. P→R/R→P/PI all follow this. RND, FRC, MOD, ABS, SIGN, R↑ are HpNum-native (no f64 needed); FACT uses f64 internally then converts back.
- **LiftEffect-per-op contract**: every op declares one of `Enable`/`Disable`/`Neutral`. Stack reorganizations (`R↑`, `Rdn`, `XY-swap`) are Neutral; value-producing ops are Enable; clearing ops (`CLX`) are Disable.
- **4-place Op-variant landing**: every new variant goes into `ops/mod.rs::Op` enum + `dispatch()` + `execute_op()` (in `ops/program.rs`) + both `prgm_display.rs` copies. Compile-time exhaustive matches enforce this — the build will fail if a place is missed.
- **`#![deny(clippy::unwrap_used)]`**: zero-panic policy at the crate root. All new code uses `?` or `.expect("reason")`. Test modules opt out via `#[allow(clippy::unwrap_used)]`.

### Integration Points
- `hp41-core/src/ops/mod.rs::Op` — add 10 new variants: `Pi`, `PolarToRect`, `RectToPolar`, `Rnd`, `Frc`, `Mod`, `Abs`, `Fact`, `Sign`, `Rup` (or `RollUp`). Planner picks final identifiers; PascalCase brevity preferred.
- `hp41-core/src/ops/mod.rs::dispatch()` — add 10 match arms (each calls the new `op_*` function).
- `hp41-core/src/ops/program.rs::execute_op()` — add 10 match arms (same shape as dispatch).
- `hp41-cli/src/prgm_display.rs::op_display_name()` — add 10 display-name arms (matches the in-program rendering — e.g. `Op::Pi => "PI"`).
- `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name()` — same 10 display-name arms (duplicated per CLAUDE.md §SC-4 note; intentional).
- `hp41-core/src/format.rs` — extract `round_to_display_precision()` helper (D-01, D-02). The existing `format_hpnum()` calls it; the new `op_rnd` calls it.

</code_context>

<specifics>
## Specific Ideas

- **PI displayed value is 3.141592654 (10 sig digits)** — not the SC-1 typo'd "3.1415926536". Planner should add a note + ROADMAP correction in PLAN.md.
- **FACT effective cap is X ≤ 27, not X ≤ 69** — pre-existing Decimal range limit. Hardware-spec preserved for the wording ("X > 69 → OutOfRange") but the practical wall is much lower. Document in the per-op doc comment + CLAUDE.md "v2.2 additions" block.
- **RND helper extraction is a Wave-0 task** — must land + pass all 500 numerical_accuracy cases BEFORE any new op work begins. Treat as a refactor commit on its own, then build forward from there.
- **SIGN-on-ALPHA divergence** — our model has no "ALPHA-typed X". Document the deviation in CLAUDE.md when Phase 25 ships docs. No special-case code in Phase 20.
- **P→R / P↔R direct-assignment** — neither `unary_result` nor `binary_result` fit the binary-out shape. Use direct field assignment + explicit `state.stack.lastx = old_x` + `apply_lift_effect(state, LiftEffect::Enable)`.

</specifics>

<deferred>
## Deferred Ideas

- **High-magnitude HpNum representation** — extending HpNum to carry a separate exponent field (allowing values up to ~9.999999999E±99) would unlock true HP-41 hardware fidelity for FACT (X ≤ 69), large exponentials, and very-large numerical-accuracy cases. **Massive scope** — affects every op in `hp41-core` and the Decimal-based serialization format. Backlog candidate for v3.0 (or its own dedicated milestone). Not blocking v2.2.
- **SIGN-on-ALPHA-typed-X** — would require introducing a value-type tag on X (numeric vs alpha-data). Probably belongs alongside ARCL/ASTO work in Phase 23, where the alpha-data boundary becomes more relevant. Note in the Phase 23 backlog when discussing FN-ALPHA-03 (`ATOX`).
- **Numerical-accuracy suite extension** — Phase 27 owns extending the 500-case suite for PI, P→R, R→P, RND, FRC, MOD, FACT. Phase 20 must NOT extend the suite (just ensure it doesn't regress). Pre-noted in Phase 27 scope.
- **PI value tie-breaker via researcher** — if planner notices the SC-1 typo is intentional (e.g. some HP-41 variant shows 11-digit PI), researcher can verify against the Owner's Manual. Otherwise the D-08 decision stands.

</deferred>

---

*Phase: 20-Core-Math-and-Conversions*
*Context gathered: 2026-05-13*
