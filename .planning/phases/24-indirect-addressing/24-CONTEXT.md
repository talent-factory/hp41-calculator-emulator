# Phase 24: Indirect Addressing (Cross-Cutting) — Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 24 adds a **single `resolve_indirect()` family of helpers** in `hp41-core` plus **~15 new `*Ind(u8)` `Op` variants** that let every previously-direct addressable op (STO, RCL, ISG, DSE, SF, CF, FS?, FC?, FS?C, FC?C, STO+/-/×/÷, ARCL, ASTO, VIEW) accept an indirect form `<OP> IND nn` where `Rnn`'s integer part is the effective register-or-flag address.

**Mandated by ROADMAP cross-cutting constraints:**
- One shared resolver — no duplicated trunc/InvalidOp logic across ops
- Non-integer pointers reject with `HpError::InvalidOp` (no new error variants)
- All new IND variants are NEW `Op` enum members (e.g. `StoInd(u8)`, `RclInd(u8)`, `SfFlagInd(u8)`, …) and land in 4 places (D-22.21 / D-23.12)
- `#![deny(clippy::unwrap_used)]` enforced throughout the resolver path
- Largest single-phase Op-variant count in v2.2 (≈15 new variants) — plan for foundation-first then variants

**Out of scope (explicit):**
- Indirect-of-indirect (`STO IND IND nn`) — HP-41 hardware is single-level only
- IND on stack-arithmetic targets (`STO+ IND Y/Z/T/LastX`) — `op_sto_arith_stack` operates on `StackReg`, not numbered regs; the IND concept doesn't apply
- Keyboard wiring / CLI prompts for the new IND variants — deferred to Phase 25 (CLI Integration)
- GUI `key_map.rs` registration for the new IND variants — deferred to Phase 26 (GUI Polish)
- Proptest for the resolver — deferred to Phase 27 (Test Hardening)
- New programmable HP-41 ops outside the listed addressable set

</domain>

<decisions>
## Implementation Decisions

### Helper API surface (D-24.1 — D-24.4)

- **D-24.1: Two-tier resolver, ONE source of pointer-truth.** Private inner helper `resolve_indirect_decimal(state: &CalcState, reg: u8) -> Result<Decimal, HpError>` is the SINGLE place that does (a) `state.regs.get(reg as usize).ok_or(InvalidOp)`, (b) `pointer.trunc_int()`, (c) `int_part != pointer → InvalidOp`. Both downstream wrappers consume this inner helper — no duplicated trunc/validation logic anywhere in the workspace.

- **D-24.2: Public Phase-24 helper signature locked.** `pub fn resolve_indirect(state: &CalcState, reg: u8) -> Result<u8, HpError>` matches the ROADMAP-locked signature. Implementation: `let i = resolve_indirect_decimal(state, reg)?; u8::try_from(i.to_i64().ok_or(InvalidOp)?).map_err(|_| InvalidOp)`. The `to_i64` + `try_from` chain rejects pointer values that won't fit in `u8` (e.g. `R05 = 300`) with `InvalidOp` — fully covers FN-IND-02's "non-integer rejection" plus the implicit "out-of-u8-range" case.

- **D-24.3: Bounds responsibility = caller.** `resolve_indirect()` does NOT check the resolved address against `state.regs.len()` (regs ops) or `< 56` (flag ops). Those limits differ per op type — pushing them into the helper would force two helpers (`resolve_indirect_reg` / `resolve_indirect_flag`) and violate the ROADMAP "single shared resolver" principle. Instead, each `*Ind` op DELEGATES to its existing direct counterpart with the resolved address: `op_sto_ind(state, reg) -> { let addr = resolve_indirect(state, reg)?; op_sto(state, addr) }`. Existing `op_sto` / `op_sf` / `op_arcl` / etc. already enforce their own bounds via the D-22.11.1 `.get().ok_or(InvalidOp)?` pattern.

- **D-24.4: Sidecar / atomicity / lift-effect discipline erbt sich.** Because every `*Ind` op delegates to its direct counterpart after resolving the address, the existing D-23.4 sidecar-clearing (`op_sto`, `op_sto_arith`, `op_clreg` → `text_regs.remove/clear`), the D-22.x atomicity guard (`checked_*` before write), and the per-op `apply_lift_effect()` tail are reused — NOT replicated. This is the load-bearing reason for the delegation pattern in D-24.3.

### Phase 22 refactor (D-24.5)

- **D-24.5: GtoInd / XeqInd are refactored onto the inner helper.** Phase 22 shipped `Op::GtoInd(reg)` and `Op::XeqInd(reg)` in `hp41-core/src/ops/program.rs:474-486` and `:500-517` with the trunc-int + InvalidOp logic INLINE. Phase 24 replaces those inline blocks with `let i = resolve_indirect_decimal(state, *reg)?; let label_str = i.to_string(); …` — they keep their stringify-for-find_in_program path (GTO/XEQ resolve to LABELS, not register addresses) but share the inner pointer-validation helper. The pre-mutation `call_stack.len() >= 4 → CallDepth` guard in `Op::XeqInd` (D-22 atomicity) stays exactly where it is. **Regression coverage:** plan 24-01 includes a sentinel test exercising the existing Phase-22 GTO IND / XEQ IND paths through the refactored code to prove no behavior change.

### FlagTest IND variant shape (D-24.6)

- **D-24.6: New struct variant `Op::FlagTestInd { kind: FlagTestKind, ind_reg: u8 }`.** Mirrors the Phase-22 `Op::GtoInd(reg)` precedent and the existing struct shape of `Op::FlagTest { kind, flag }`. Backward-compat: v2.0/v2.1/v2.2 save files with `Op::FlagTest { kind, flag: 5 }` deserialize unchanged. Dispatch: `Op::FlagTestInd { kind, ind_reg } => { let flag = resolve_indirect(state, *ind_reg)?; op_flag_test(state, kind.clone(), flag) }`. All four `FlagTestKind` sub-cases (IsSet / IsClear / IsSetThenClear / IsClearThenClear) are covered by the single new variant via kind reuse — no `FlagOperand` enum, no breaking change to the existing `Op::FlagTest.flag` field.

### IND-variant naming pattern (D-24.7 — derived from existing precedent, not really a gray-area)

- **D-24.7: Every new variant uses `<Name>Ind(u8)` shape.** Locked by Phase 22's `Op::GtoInd(reg)` / `Op::XeqInd(reg)` precedent. Expected variant set (subject to planner-level confirmation against the dispatch table):
  - **Register ops:** `Op::StoInd(u8)`, `Op::RclInd(u8)`, `Op::StoArithInd(u8, StoArithKind)` (mirrors `Op::StoArith(u8, kind)`), `Op::IsgInd(u8)`, `Op::DseInd(u8)`
  - **Flag ops:** `Op::SfFlagInd(u8)`, `Op::CfFlagInd(u8)`, `Op::FlagTestInd { kind: FlagTestKind, ind_reg: u8 }` (per D-24.6)
  - **ALPHA-register ops:** `Op::ArclInd(u8)`, `Op::AstoInd(u8)`
  - **Display ops:** `Op::ViewInd(u8)` (delegates to `op_view`; displays VALUE of resolved register, not the pointer)
  - Total estimated: **~12 new variants** (single `Op::StoArithInd(u8, kind)` covers the four STO+/-/×/÷ IND forms via kind reuse, identical to existing `Op::StoArith`). Planner re-counts against the dispatch table.

### Wave / plan structure (D-24.8 — D-24.9)

- **D-24.8: Two plans, sequential waves.**
  - **24-01 (Wave 1) — Foundation:** `resolve_indirect_decimal` inner helper, `resolve_indirect` u8-wrapper, Phase-22 GtoInd/XeqInd refactor onto the inner helper, regression test suite proving no Phase-22 behavior change. **Files touched:** `hp41-core/src/ops/program.rs` (refactor inline blocks), new `hp41-core/src/ops/indirect.rs` (the two helpers + inline unit tests), new `hp41-core/tests/phase24_resolve_indirect.rs` (helper unit tests + GTO/XEQ IND regression).
  - **24-02 (Wave 2, depends_on: [24-01]) — Variants:** ~12 new `*Ind` Op variants. Each variant delegates to its direct counterpart via `resolve_indirect`. **Files touched:** `hp41-core/src/ops/mod.rs` (enum + dispatch), `hp41-core/src/ops/program.rs` (execute_op arms for the IND variants that are programmable), `hp41-cli/src/prgm_display.rs`, `hp41-gui/src-tauri/src/prgm_display.rs`, new `hp41-core/tests/phase24_ind_variants.rs`. The 4-place landing rule (D-22.21 / D-23.12) applies to every new variant.

- **D-24.9: File-overlap forces wave-sequential.** Both plans touch `hp41-core/src/ops/program.rs`. Both plans touch the dispatch hub `ops/mod.rs` (24-01 if the helpers get re-exported there, definitely 24-02). The wave-sequential dependency is real — `depends_on: [24-01]` in 24-02's frontmatter is mandatory.

### Cross-cutting invariants (carried forward, NOT re-decided)

- **D-22.21 / D-23.12 (4-place Op-variant landing):** every new `*Ind` variant lands in `dispatch()` (ops/mod.rs), `execute_op()` (ops/program.rs), `hp41-cli/src/prgm_display.rs`, `hp41-gui/src-tauri/src/prgm_display.rs`. Phase 24 follows.
- **D-22.11.1 (regs[] bounds via `.get().ok_or(InvalidOp)?`):** the inner helper uses it (for the pointer-register read); callers inherit their own bounds checks from their direct-form counterparts. No raw `state.regs[i]` indexing on the IND path.
- **D-23.4 (sidecar-clearing audit):** `op_sto_ind`/`op_sto_arith_ind`/`op_arcl_ind`/`op_asto_ind` delegate to direct ops that already do the sidecar work. No new sidecar-clearing call sites.
- **D-23.14 / zero-panic:** no `.unwrap()` in production code. All resolver paths use `?` or `.ok_or(InvalidOp)?`. Test modules retain `#[allow(clippy::unwrap_used)]`.
- **HpError::InvalidOp reuse:** no new `HpError` variants in Phase 24. Both non-integer pointers (FN-IND-02) and out-of-range pointers (>255 fitting u8) collapse to `InvalidOp`. Out-of-regs.len() / out-of-flag-range stay handled by the direct-op delegation.
- **`#[serde(default)]` for save-file backward-compat:** Phase 24 adds NO new `CalcState` fields — all state changes are encoded as new `Op` enum variants (program-step level, not state-snapshot level). Pre-Phase-24 save files load unchanged.

### Claude's Discretion

- **Inner-helper module location:** `hp41-core/src/ops/indirect.rs` is the recommended new file (sibling to `flags.rs`, `program.rs`, `registers.rs`). Re-exports from `ops/mod.rs`. Planner may collapse it into `ops/mod.rs` directly if the file would be too small — implementer's call once the byte count is known.
- **`op_sto_arith_ind` arity vs four-variant split:** `Op::StoArithInd(u8, StoArithKind)` mirrors the existing `Op::StoArith(u8, kind)` shape exactly. Planner may choose four flat variants (`StoAddInd` / `StoSubInd` / `StoMulInd` / `StoDivInd`) if the prgm_display formatting is cleaner that way. Default: single variant with kind reuse.
- **Regression-test scaffolding location:** 24-01 holds GTO/XEQ-IND regression (proves Phase 22 behavior unchanged through refactor). 24-02 holds the new IND-variant test suite. Planner may add a small `phase24_helper.rs` test module if the inner helper's unit tests get large; default is keep them inline in `indirect.rs` with `#[cfg(test)]`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 24 inputs (this phase's authority)
- `.planning/ROADMAP.md` §"Phase 24: Indirect Addressing (Cross-Cutting)" — goal, must-haves, cross-cutting constraints
- `.planning/REQUIREMENTS.md` §FN-IND-01, FN-IND-02 — requirement text
- `.planning/phases/24-indirect-addressing/24-CONTEXT.md` — this file (D-24.1..D-24.9)

### Phase 22 — Precedent for indirect resolution (Op::GtoInd / Op::XeqInd)
- `hp41-core/src/ops/program.rs:466-517` — current INLINE indirect-resolution pattern that Phase 24 refactors
- `.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md` §D-22.15 (XEQ-IND atomicity) and §D-22.11.1 (regs bounds-audit) — patterns to honor
- `.planning/phases/22-program-control-and-memory-ops/22-SUMMARY.md` — what shipped, naming established (`GtoInd(reg)` / `XeqInd(reg)`)

### Phase 21 — Flag-storage and FlagTest enum shape
- `hp41-core/src/ops/mod.rs:55-71` — `FlagTestKind` enum (IsSet / IsClear / IsSetThenClear / IsClearThenClear)
- `hp41-core/src/ops/mod.rs:319-327` — `Op::SfFlag(u8)`, `Op::CfFlag(u8)`, `Op::FlagTest { kind, flag }` — Phase 24's FlagTestInd struct variant mirrors this shape
- `hp41-core/src/ops/flags.rs:39-56` — `op_sf`, `op_cf` with their own `< 56` bounds checks (which Phase 24's IND-form delegates to)

### Phase 23 — ALPHA register sidecar and naming pattern
- `hp41-core/src/ops/alpha.rs` — `op_arcl`, `op_asto` direct forms (Phase 24's IND form delegates here)
- `.planning/phases/23-alpha-operations/23-CONTEXT.md` §D-23.4 — sidecar-clearing invariant that 24's `*StoInd` / `*ArclInd` inherit gratis through delegation
- `.planning/phases/23-alpha-operations/23-SUMMARY.md` — naming pattern (`Arcl(u8)` / `Asto(u8)`) — Phase 24 adds `ArclInd(u8)` / `AstoInd(u8)`

### Direct-form ops that Phase 24 delegates to (reuse, NOT duplicate)
- `hp41-core/src/ops/registers.rs` — `op_sto`, `op_rcl`, `op_sto_arith` (bounds + D-23.4 sidecar already in place)
- `hp41-core/src/ops/program.rs` — `op_isg`, `op_dse` (bounds + signed-counter logic already in place)
- `hp41-core/src/ops/flags.rs` — `op_sf`, `op_cf`, `op_flag_test` (`< 56` bounds + skip-step semantics already in place)
- `hp41-core/src/ops/alpha.rs` — `op_arcl`, `op_asto` (text_regs sidecar + 6-char ASCII pack already in place)
- `hp41-core/src/ops/display.rs` — `op_view` (display-override semantics already in place)

### Project-level invariants
- `CLAUDE.md` §"Settled Architecture Decisions" — zero-panic policy, 4-place Op landing, `#[serde(default)]` rule, English-only commits
- `.planning/PROJECT.md` — core value (HP-41 fidelity); v2.2 milestone goals

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Decimal::trunc_int()` + `HpNum` equality check pattern** (program.rs:480-483): exact precedent for the inner helper's non-integer rejection. Copy verbatim.
- **`state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?.clone()`** (program.rs:475-479): exact precedent for the bounds-safe pointer read. Copy verbatim.
- **`op_sto` / `op_sf` / `op_arcl` / `op_view` / `op_isg` / `op_dse` direct forms:** all already do their own bounds + atomicity + sidecar (where applicable). Phase 24's IND variants are 2-line shims over them.
- **`Op::StoArith(u8, StoArithKind)` struct shape** (mod.rs): exact precedent for `Op::StoArithInd(u8, StoArithKind)`.
- **`Op::FlagTest { kind, flag }` struct-variant shape** (mod.rs:324-329): exact precedent for `Op::FlagTestInd { kind, ind_reg }`.

### Established Patterns
- **4-place Op-variant landing (D-22.21 / D-23.12):** every new `*Ind` lands in dispatch(), execute_op(), and both prgm_display copies. Exhaustive `match` will fail-fast at compile time if any of the four is forgotten.
- **Delegation-over-duplication:** `op_sto_arith_stack` (registers.rs:83) is itself a non-delegating direct write; the IND family is the inverse — every `op_*_ind` delegates to its direct counterpart.
- **`#[serde(default)]` on every new CalcState field:** Phase 24 adds NO new fields. Phase-23 save files load unchanged.
- **Atomicity guard "compute-then-write":** D-22.x principle that `state.regs[idx] = checked_op?` writes only on success. The IND path inherits this gratis via delegation.

### Integration Points
- **`hp41-core/src/ops/mod.rs::dispatch()`** — new `Op::*Ind` arms route to new `op_*_ind` functions (typically 2 lines each: resolve + delegate).
- **`hp41-core/src/ops/program.rs::execute_op()`** — only the programmable IND variants need arms here (most do; `Op::FlagTestInd` is in the conditional-skip group).
- **`hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`** — bare-string display names for every new IND variant; the SC-4 mirror rule applies (every variant exists in both copies).
- **`hp41-core/src/ops/indirect.rs` (NEW)** — `resolve_indirect_decimal` (private) + `resolve_indirect` (pub) live here. The ~12 `op_*_ind` shims may live here too if file size allows; planner decides.

</code_context>

<specifics>
## Specific Ideas

- **Sentinel regression test for the Phase-22 refactor:** the test must exercise `Op::GtoInd(reg)` and `Op::XeqInd(reg)` end-to-end through `run_program` after the refactor — proves byte-for-byte equivalent behavior, not just compile-passes. Should mirror an existing Phase-22 integration test (look for `phase22_program_control.rs` GTO/XEQ-IND tests as the baseline).
- **`u8::try_from(i.inner().to_i64().ok_or(InvalidOp)?).map_err(|_| InvalidOp)` chain** is the canonical pointer-to-u8 conversion. Two cascading `InvalidOp` paths: `to_i64` for "fits in i64" and `try_from` for "fits in u8". Both collapse to a single error variant per ROADMAP.
- **`view_ind` displays the value of the resolved register, NOT the pointer.** HP-41 hardware: `VIEW IND 05` with R05=12 shows the value of R12 (e.g. "12.000" if R12=12), not the value of R05.
- **`STO IND IND nn` is rejected at the program-text level** (no `Op::StoIndInd` variant exists; the parser/keyboard never produces one). Out of scope — no special test needed.

</specifics>

<deferred>
## Deferred Ideas

- **Proptest sweep for `resolve_indirect`** — `prop_assume!(pointer.fract() != Decimal::ZERO) → resolve returns Ok(u8::try_from(int).ok().filter(|n| /* fits */))` style. Deferred to Phase 27 (Test Hardening), per ROADMAP cross-cutting constraint.
- **Keyboard wiring for IND prompts in `hp41-cli`** — new `PendingInput::StoIndPrompt`, `RclIndPrompt`, `ArclIndPrompt`, `FlagTestIndPrompt`, etc. — deferred to Phase 25 (CLI Integration), per ROADMAP.
- **GUI key_map.rs registration for IND variants** — string-ID resolver for `sto_ind`, `rcl_ind`, etc. — deferred to Phase 26 (GUI Polish), per ROADMAP.
- **HP-41CV function matrix entry for each IND variant** — documentation deliverable — deferred to Phase 25 doc-block per FN-DOC-01..04.
- **Indirect-of-indirect (`STO IND IND nn`)** — HP-41 hardware does not support it. Permanently out of scope (not deferred — won't ship).

</deferred>

---

*Phase: 24-indirect-addressing*
*Context gathered: 2026-05-14*
