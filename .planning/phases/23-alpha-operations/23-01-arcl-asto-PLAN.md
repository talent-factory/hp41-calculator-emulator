---
phase: 23-alpha-operations
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-core/src/state.rs
  - hp41-core/src/ops/registers.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/alpha.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase23_arcl_asto.rs
autonomous: true
requirements:
  - FN-ALPHA-01
  - FN-ALPHA-02

must_haves:
  truths:
    - "ARCL appends a register's formatted value to ALPHA respecting the current FIX/SCI/ENG display mode"
    - "ASTO packs the first 6 chars of ALPHA into the text-shadow of a numbered register and zeroes the numeric slot"
    - "RCL+ARCL round-trips an ASTO'd string back into ALPHA"
    - "Every numeric STO/STO-arith/CLREG clears the matching text_regs sidecar entry so the two representations never drift"
    - "v1.x save files without a text_regs field load cleanly into v2.2 (default = empty map)"
  artifacts:
    - path: "hp41-core/src/state.rs"
      provides: "text_regs: BTreeMap<u8, String> field on CalcState with #[serde(default)], slotted next to assignments"
      contains: "pub text_regs: BTreeMap<u8, String>"
    - path: "hp41-core/src/ops/alpha.rs"
      provides: "op_arcl + op_asto + inline unit tests"
      contains: "pub fn op_arcl"
    - path: "hp41-core/src/ops/registers.rs"
      provides: "sidecar-clearing audit in op_sto / op_sto_arith / op_clreg (plus op_sto_arith_stack inspection note)"
      contains: "state.text_regs.remove(&"
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Op::Arcl(u8) and Op::Asto(u8) variants AT END of enum + dispatch() arms"
      contains: "Arcl(u8)"
    - path: "hp41-core/src/ops/program.rs"
      provides: "execute_op() arms for Arcl/Asto"
      contains: "Op::Arcl(reg)"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "op_display_name arms for Arcl/Asto"
      contains: "\"ARCL"
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "op_display_name arms for Arcl/Asto (mirror of hp41-cli copy)"
      contains: "\"ARCL"
    - path: "hp41-core/tests/phase23_arcl_asto.rs"
      provides: "Integration suite for SC#1, SC#2, sidecar-clearing invariant"
  key_links:
    - from: "Op::Arcl(u8) in ops/mod.rs"
      to: "op_arcl in ops/alpha.rs"
      via: "dispatch() arm"
      pattern: "Op::Arcl\\(reg\\) => alpha::op_arcl"
    - from: "op_arcl"
      to: "format_hpnum(r, &state.display_mode)"
      via: "fallback path when text_regs miss"
      pattern: "format_hpnum"
    - from: "op_sto / op_sto_arith / op_clreg in registers.rs"
      to: "state.text_regs.remove(&reg) / state.text_regs.clear()"
      via: "Wave-0 sidecar-clearing audit (D-23.4)"
      pattern: "text_regs\\.(remove|clear)"
---

<objective>
Land the two ALPHA-register-meets-storage-register ops `ARCL nn` and `ASTO nn`
in `hp41-core`, plus the foundational `text_regs: BTreeMap<u8, String>`
sidecar on `CalcState` that backs them. ASTO writes the packed-text shadow
of a register; ARCL reads from it (or falls back to formatting the numeric
slot via `format_hpnum`). Before either op lands, audit every numeric write
to `regs[reg]` so the two representations never drift (D-23.4).

Purpose: Phase 23 delivers FN-ALPHA-01 (ARCL) and FN-ALPHA-02 (ASTO). These
two are the only Phase 23 ops that introduce new persistent state; the
remaining 4 (ATOX/XTOA/AROT/POSA in plan 02) only touch `alpha_reg` and
`stack.x` and can land cleanly on top of this plan.

Output: One new `CalcState` field (`text_regs`), 3 audited functions in
`registers.rs`, 2 new ops (`op_arcl`, `op_asto`), 2 new Op variants landed
in all 4 places (D-23.12), and an integration test suite covering SC#1
(FIX/SCI mode dependency), SC#2 (ASTO + ARCL round-trip), and the
sidecar-clearing invariant.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@CLAUDE.md
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/23-alpha-operations/23-CONTEXT.md

<!-- Predecessor: Phase 22 introduced the BTreeMap-on-CalcState pattern -->
@.planning/phases/22-program-control-and-memory-ops/22-04-catalog-and-asn-PLAN.md

<!-- Files this plan modifies — read for current state and patterns -->
@hp41-core/src/state.rs
@hp41-core/src/ops/alpha.rs
@hp41-core/src/ops/registers.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/ops/program.rs
@hp41-cli/src/prgm_display.rs
@hp41-gui/src-tauri/src/prgm_display.rs

<!-- Canonical helpers / reuse targets — read for signatures, do NOT duplicate -->
@hp41-core/src/format.rs
@hp41-core/src/error.rs
@hp41-core/src/num.rs
@hp41-core/src/stack.rs

<interfaces>
<!-- Key signatures the executor needs in scope. Do NOT re-derive — read the source. -->

From hp41-core/src/format.rs:
- `pub fn format_hpnum(n: &HpNum, mode: &DisplayMode) -> String` — ARCL numeric formatter; already respects FIX/SCI/ENG.

From hp41-core/src/stack.rs:
- `pub enum LiftEffect { Enable, Disable, Neutral }`
- `pub fn apply_lift_effect(state: &mut CalcState, effect: LiftEffect)` — every op tail-calls this; ARCL/ASTO use `Neutral`.

From hp41-core/src/error.rs:
- `pub enum HpError { ..., InvalidOp, ... }` — out-of-range reg uses `InvalidOp`. NO new variants.

From hp41-core/src/num.rs:
- `impl HpNum { pub fn zero() -> Self; ... }` — `op_asto` zeroes the numeric slot after stashing the text.

From hp41-core/src/ops/alpha.rs (precedent for 24-char silent-discard cap):
- `pub fn op_alpha_append(state: &mut CalcState, ch: char) -> Result<(), HpError>` — `chars().count() < 24` guard.

From hp41-core/src/state.rs (slot location, next to `assignments` at line ~94):
- Phase 22 `#[serde(default)] pub assignments: BTreeMap<u8, String>` — `text_regs` slots IMMEDIATELY AFTER this, same shape, same `#[serde(default)]`.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1 (Wave-0): Sidecar-clearing audit in registers.rs</name>
  <files>hp41-core/src/ops/registers.rs</files>
  <read_first>
    - hp41-core/src/ops/registers.rs (current op_sto, op_sto_arith, op_sto_arith_stack, op_clreg signatures and bodies)
    - hp41-core/src/state.rs lines 82-94 (assignments BTreeMap slot — pattern to mirror for text_regs)
    - .planning/phases/23-alpha-operations/23-CONTEXT.md §D-23.4 (sidecar-clearing invariant — the one D-23 invariant whose absence silently corrupts ARCL output)
    - .planning/phases/22-program-control-and-memory-ops/22-03-memory-ops-PLAN.md (Phase 22 D-22.11.1 regs-bounds audit — the precedent for this Wave-0 commit pattern)
  </read_first>
  <behavior>
    - In `op_sto`: before writing `state.regs[idx] = state.stack.x.clone()`, call `state.text_regs.remove(&reg)`. (`reg: u8` is the function parameter — use `&reg`, not `&(idx as u8)`.)
    - In `op_sto_arith`: same — before `state.regs[idx] = new_val`, call `state.text_regs.remove(&reg)`.
    - In `op_sto_arith_stack`: the function takes a `StackReg` (Y/Z/T/LastX), NOT a u8 — it does NOT touch `state.regs[]`, so NO sidecar clearing is needed. Add a one-line comment confirming this audit outcome (rather than a no-op call).
    - In `op_clreg`: alongside the `state.regs = vec![HpNum::zero(); n]` reset, call `state.text_regs.clear()`.
    - The `text_regs` field does NOT yet exist (Task 2 adds it). To keep this Wave-0 commit independently meaningful (D-23.4 / D-22.11.1 precedent), this task MAY land as a single combined commit with Task 2's field addition — git-blame clarity is the only reason to split. Implementer's discretion. If combined: still write the audit changes first in the diff, then the field addition.
    - Add inline `#[cfg(test)]` tests in `registers.rs` (or extend its existing test module) covering:
      - `test_op_sto_clears_text_regs_sidecar`: pre-populate `state.text_regs.insert(5, "HELLO".into())`, then `op_sto(state, 5)` → assert `state.text_regs.get(&5) == None`.
      - `test_op_sto_arith_clears_text_regs_sidecar`: same setup, then `op_sto_arith(state, 5, StoArithKind::Add)` → assert sidecar cleared.
      - `test_op_clreg_clears_all_text_regs`: pre-populate three entries, run `op_clreg`, assert `state.text_regs.is_empty()`.
  </behavior>
  <action>
    Insert `state.text_regs.remove(&reg);` and `state.text_regs.clear();` per D-23.4 into the 3 functions named in `<behavior>`. The `op_sto_arith_stack` path gets a clarifying comment only — it does not write to `regs[]`. Reuse the existing `HpError` surface (no new variants). The `#[cfg(test)] mod tests` already has `#[allow(clippy::unwrap_used)]` precedent — keep that pattern. Test names should be prefixed with `test_phase23_` or grouped under a new `#[cfg(test)] mod phase23_sidecar_audit_tests` submodule, matching the Phase 22 D-22.11.1 commit style. Run `just test-core` after; the new tests will fail to compile until Task 2 adds the `text_regs` field — that is intentional and confirms the commits land in order.
  </action>
  <verify>
    <automated>just test-core 2>&amp;1 | tail -40</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "state.text_regs.remove(&reg)" hp41-core/src/ops/registers.rs` returns at least 2 (op_sto, op_sto_arith).
    - `grep -c "state.text_regs.clear()" hp41-core/src/ops/registers.rs` returns at least 1 (op_clreg).
    - `grep -n "op_sto_arith_stack" hp41-core/src/ops/registers.rs` — the surrounding body contains a comment of the form `// text_regs: not touched — stack registers (Y/Z/T/LastX) do not back text shadows (D-23.4)` or equivalent.
    - The 3 new `#[cfg(test)]` tests are present and (once Task 2 ships) `just test-core` runs them green.
    - No new `HpError` variant introduced (D-23 reuses `InvalidOp` only).
    - `#![deny(clippy::unwrap_used)]` still holds at crate root — no new `.unwrap()` in non-test code.
  </acceptance_criteria>
  <done>
    Four touchpoints audited per D-23.4. Tests for sidecar-clearing exist (will compile after Task 2 lands the field). `just test-core` passes when Task 1 + Task 2 are taken together. (If implementer chose to combine commits, this acceptance applies to the combined commit.)
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add text_regs field to CalcState + op_arcl + op_asto + 4-place Op landing</name>
  <files>hp41-core/src/state.rs, hp41-core/src/ops/alpha.rs, hp41-core/src/ops/mod.rs, hp41-core/src/ops/program.rs, hp41-cli/src/prgm_display.rs, hp41-gui/src-tauri/src/prgm_display.rs</files>
  <read_first>
    - hp41-core/src/state.rs lines 82-94 + lines 150-180 (assignments field declaration + CalcState::new initializer — text_regs mirrors this exactly)
    - hp41-core/src/ops/alpha.rs (full — extend in place; 71 lines now, will stay under ~250 after adding op_arcl + op_asto + their tests)
    - hp41-core/src/ops/mod.rs lines 121-140 (Op enum tail — find the end where new variants land), lines 540-580 (dispatch() arms around Op::Pi)
    - hp41-core/src/ops/program.rs lines around `Op::Catalog(n) => ...` and `Op::Asn { ... } => ...` (Phase 22 precedent for where new exhaustive-match arms slot in)
    - hp41-cli/src/prgm_display.rs lines around `Op::Cla / Op::Catalog / Op::Asn` (Phase 22 precedent for ARCL/ASTO display strings)
    - hp41-gui/src-tauri/src/prgm_display.rs lines around `Op::Cla / Op::Catalog / Op::Asn` (mirror copy — duplication is intentional per CLAUDE.md SC-4 note)
    - hp41-core/src/format.rs::format_hpnum (signature + that it respects display_mode — reuse, do NOT reimplement)
    - hp41-core/src/ops/alpha.rs::op_alpha_append (canonical 24-char cap pattern: `state.alpha_reg.chars().count() < 24`)
    - .planning/phases/23-alpha-operations/23-CONTEXT.md §D-23.1, §D-23.2, §D-23.3, §D-23.16 (text_regs field shape; ASTO semantics; ARCL lookup order; LiftEffect = Neutral for both)
    - .planning/phases/23-alpha-operations/23-CONTEXT.md §"Concrete signature sketches" (reference signatures for op_arcl + op_asto — mirror, but the executor reads the canonical source files for HpNum / Stack / format_hpnum signatures rather than copying from CONTEXT verbatim)
  </read_first>
  <behavior>
    State field (D-23.1):
    - Add `pub text_regs: BTreeMap<u8, String>` to `CalcState` immediately after the `assignments` field (line ~94). Comment block per the CONTEXT D-23.1 sketch: explains the sidecar role, the D-23.4 clearing invariant, and that `#[serde(default)]` carries v1.x compat.
    - Add `text_regs: BTreeMap::new(),` to `CalcState::new()` initializer in alphabetical-by-slot order (immediately after `assignments: BTreeMap::new(),`).

    Op variants (D-23.12, D-23.13 — land AT END of Op enum):
    - `Op::Arcl(u8)` — append register N's formatted value to ALPHA
    - `Op::Asto(u8)` — pack first 6 ALPHA chars into register N's text_regs sidecar
    - Both go in `ops/mod.rs::Op` enum tail, after the most recent Phase 22 variants.
    - Add dispatch arms in `ops/mod.rs::dispatch()` near the Phase 22 Cla/Catalog/Asn arms: `Op::Arcl(reg) => alpha::op_arcl(state, *reg)`, `Op::Asto(reg) => alpha::op_asto(state, *reg)`.
    - Add execute_op arms in `ops/program.rs::execute_op` matching the same pattern.
    - Add `op_display_name` arms in BOTH `prgm_display.rs` copies: `Op::Arcl(reg) => format!("ARCL {reg:02}")`, `Op::Asto(reg) => format!("ASTO {reg:02}")`. The `{reg:02}` width matches Phase 22 `Op::Sto(reg)` formatting.

    `op_arcl(state: &mut CalcState, reg: u8) -> Result<(), HpError>` (D-23.3, strengthened by W-2):
    - **Leading bounds check first** (defensive, mirrors op_asto / op_sto precedent): `if (reg as usize) >= state.regs.len() { return Err(HpError::InvalidOp); }`. This is a strengthening over the D-23.3 CONTEXT sketch — without it, a hand-edited autosave.json with `text_regs[200] = "X"` would let `op_arcl(state, 200)` succeed and return that bogus shadow before any numeric-range check fired. The plan supersedes the CONTEXT sketch on this point; threat T-23-01 mitigation depends on it.
    - Lookup order AFTER the bounds check: 1) if `state.text_regs.get(&reg).is_some()` → clone that string; 2) else `state.regs.get(reg as usize).expect("bounds-checked above")` and format via `crate::format::format_hpnum(r, &state.display_mode)` (the get cannot fail because of the leading check, but use `.expect` rather than `.unwrap` to comply with `#![deny(clippy::unwrap_used)]`).
    - Append the resulting text to `state.alpha_reg`, char-by-char, breaking when `chars().count() >= 24` (silent discard, mirrors `op_alpha_append` precedent).
    - LiftEffect: Neutral.

    `op_asto(state: &mut CalcState, reg: u8) -> Result<(), HpError>` (D-23.2):
    - Bounds-check first: `if (reg as usize) >= state.regs.len() { return Err(HpError::InvalidOp); }` (matches op_sto precedent).
    - Take first 6 chars of `state.alpha_reg` via `state.alpha_reg.chars().take(6).collect::<String>()` (multibyte-safe).
    - Insert into `state.text_regs.insert(reg, text)`.
    - Zero the numeric slot via `if let Some(slot) = state.regs.get_mut(reg as usize) { *slot = HpNum::zero(); }`.
    - ALPHA register is NOT modified by ASTO (only the sidecar is written).
    - LiftEffect: Neutral.

    Inline unit tests in `alpha.rs` (under the existing `#[cfg(test)] mod tests` with `#[allow(clippy::unwrap_used)]`):
    - `test_arcl_appends_numeric_register_via_format_hpnum_in_fix_mode`
    - `test_arcl_appends_numeric_register_in_sci_mode_differs_from_fix` (SC#1 verifier)
    - `test_arcl_prefers_text_regs_over_numeric_regs_when_both_set` (note: by D-23.4 this can only happen if the test bypasses the public ops — direct manipulation of `state.text_regs` is fine for the test)
    - `test_arcl_out_of_range_reg_returns_invalid_op`
    - `test_arcl_respects_24_char_alpha_cap_silent_discard`
    - `test_asto_packs_first_6_chars_into_text_regs`
    - `test_asto_zeroes_numeric_slot_after_packing` (the no-drift invariant)
    - `test_asto_out_of_range_reg_returns_invalid_op` (and verifies the sidecar was NOT inserted — atomicity)
    - `test_asto_multibyte_first_6_chars_via_chars_take_6` (e.g. "café résumé" → first 6 chars)
  </behavior>
  <action>
    Implement per D-23.1, D-23.2, D-23.3 (strengthened — see below), D-23.12, D-23.13, D-23.16 (LiftEffect::Neutral both). Extend `hp41-core/src/ops/alpha.rs` in place (current file is 71 lines; D-23 ops keep it well under the 400-line split threshold). Reuse `crate::format::format_hpnum`, `crate::num::HpNum::zero()`, `crate::ops::alpha::op_alpha_append`'s `chars().count() < 24` cap idiom, and `crate::stack::apply_lift_effect(state, LiftEffect::Neutral)`. NO new `HpError` variants per D-23.4 / D-23.14.

    **D-23.3 strengthening (W-2):** `op_arcl` MUST perform a leading `if (reg as usize) >= state.regs.len() { return Err(HpError::InvalidOp); }` BEFORE the text_regs lookup. The CONTEXT.md D-23.3 sketch consults `text_regs` first and only bounds-checks on the numeric-fallback branch; that ordering allows a tampered autosave.json with an out-of-range `text_regs` key (e.g. `text_regs[200]`) to bypass the bounds check entirely. Adding the leading check makes `op_arcl` symmetric with `op_asto` and pins threat T-23-01.

    The 4-place Op landing is mandatory (D-23.12) — every variant in `Op::` enum, `dispatch()`, `execute_op()`, both `prgm_display.rs` copies; a missed copy is a compile-time exhaustive-match failure. Display strings: `"ARCL {reg:02}"` / `"ASTO {reg:02}"` (D-23.12 planner-discretion choice).
  </action>
  <verify>
    <automated>just check &amp;&amp; just test-core 2>&amp;1 | tail -60</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "pub text_regs: BTreeMap<u8, String>" hp41-core/src/state.rs` returns 1.
    - `grep -c "text_regs: BTreeMap::new()" hp41-core/src/state.rs` returns 1 (in CalcState::new).
    - `grep -v '^//' hp41-core/src/state.rs | grep -c "#\\[serde(default)\\]"` is >= the count before this plan + 1 (one new `#[serde(default)]` for text_regs).
    - `grep -c "Arcl(u8)" hp41-core/src/ops/mod.rs` >= 2 (enum decl + dispatch arm).
    - `grep -c "Asto(u8)" hp41-core/src/ops/mod.rs` >= 2.
    - `grep -nE "Op::Arcl|Op::Asto" hp41-core/src/ops/program.rs` returns at least 2 matches (execute_op arms).
    - `grep -nE "Op::Arcl|Op::Asto" hp41-cli/src/prgm_display.rs` returns at least 2 matches.
    - `grep -nE "Op::Arcl|Op::Asto" hp41-gui/src-tauri/src/prgm_display.rs` returns at least 2 matches.
    - `grep -c "pub fn op_arcl" hp41-core/src/ops/alpha.rs` returns 1.
    - `grep -c "pub fn op_asto" hp41-core/src/ops/alpha.rs` returns 1.
    - `grep -c "format_hpnum" hp41-core/src/ops/alpha.rs` returns at least 1 (op_arcl numeric fallback).
    - `grep -c "HpNum::zero()" hp41-core/src/ops/alpha.rs` returns at least 1 (op_asto numeric zero-out).
    - **W-2 strengthening:** `op_arcl` body in `hp41-core/src/ops/alpha.rs` contains a leading bounds check — `grep -nE "reg as usize.*>=.*state.regs.len|state.regs.len.*<=.*reg as usize" hp41-core/src/ops/alpha.rs` returns at least 2 matches (one for op_arcl, one for op_asto).
    - `just check` passes (exhaustive matches compile in all 4 places).
    - `just test-core` passes including the new alpha.rs unit tests AND the Task 1 sidecar-audit tests (combined integration).
    - SC-4 stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns nothing (the GUI prgm_display.rs change only adds string-formatting arms, not calculator logic — invariant preserved per D-23.15).
    - No new `HpError` variants in `hp41-core/src/error.rs` (D-23 reuses InvalidOp only).
  </acceptance_criteria>
  <done>
    `text_regs` field exists with `#[serde(default)]`. `op_arcl` and `op_asto` are implemented per D-23.2/D-23.3, with `op_arcl` strengthened by a leading `regs.len()` bounds check (W-2). All 4 landing places carry the new variants. `just check` + `just test-core` green. Old v1.x save files load via `text_regs: BTreeMap::new()` default (verified by inline test or covered by the integration suite in Task 3).
  </done>
</task>

<task type="auto">
  <name>Task 3: Integration test suite — SC#1, SC#2, sidecar-clearing invariant, save-file backward compat</name>
  <files>hp41-core/tests/phase23_arcl_asto.rs</files>
  <read_first>
    - hp41-core/tests/numerical_accuracy.rs (top of file — for `#[allow(clippy::unwrap_used)]` test module header)
    - hp41-core/src/state.rs (CalcState::new initializer + DisplayMode + alpha_reg/regs/text_regs fields)
    - hp41-core/src/format.rs (DisplayMode enum: Fix(u8), Sci(u8), Eng(u8))
    - hp41-core/src/ops/mod.rs dispatch (so the tests can dispatch Op::Sto, Op::Rcl, Op::Cla, Op::Arcl, Op::Asto, Op::Clreg, and the FIX/SCI display-mode setters)
    - .planning/phases/23-alpha-operations/23-CONTEXT.md §"Success Criteria" and §"Specific ideas" (SC#1 = FIX/SCI/ENG mode round-trip; SC#2 = ASTO+RCL+ARCL round-trip — note SC#2 wording explicitly requires "RCL 12 and ARCL 12 reproduce GOODBY" — but D-23.5 documents RCL of a text-shadowed register pushes 0; the test's assertion must check `ARCL 12` reproduces "GOODBY", NOT that `RCL 12` reproduces it. Document this divergence in a test comment.)
  </read_first>
  <action>
    Create `hp41-core/tests/phase23_arcl_asto.rs` as a new integration test file. Header: `#![allow(clippy::unwrap_used)]` (test modules allowed to unwrap per CLAUDE.md). Tests covering:

    1. `arcl_appends_numeric_register_using_current_display_mode` (SC#1):
       - Setup: `state.alpha_reg = "HELLO"`, `state.regs[5] = HpNum::from_str("3.14")`, `state.display_mode = DisplayMode::Fix(2)`.
       - Dispatch `Op::Arcl(5)`. Assert `state.alpha_reg == "HELLO3.14"` (or whatever `format_hpnum(3.14, Fix(2))` produces — test by calling format_hpnum directly to derive the expected string at test time, not by hardcoding it).
       - Switch to `DisplayMode::Sci(3)`. Reset `state.alpha_reg = "HELLO"`. Dispatch `Op::Arcl(5)` again. Assert the appended suffix matches `format_hpnum(3.14, Sci(3))` — and crucially differs from the Fix(2) suffix.

    2. `asto_arcl_round_trip_reproduces_first_6_chars` (SC#2):
       - Setup: `state.alpha_reg = "GOODBYE"`. Dispatch `Op::Asto(12)`. Assert `state.text_regs.get(&12) == Some(&"GOODBY".to_string())`. Assert `state.regs[12] == HpNum::zero()` (no-drift invariant).
       - Clear ALPHA via `Op::Cla` (or direct `state.alpha_reg.clear()`). Dispatch `Op::Arcl(12)`. Assert `state.alpha_reg == "GOODBY"`.
       - Comment block: explain that real HP-41 RCL of a text-shadowed register copies the packed 56 bits back to X; our `HpNum = Decimal` model can't do that; D-23.5 documents the divergence — RCL of register 12 here pushes 0. SC#2 is interpreted as "ARCL round-trips the text" (not "RCL").

    3. `numeric_sto_clears_text_regs_sidecar_no_drift` (D-23.4 invariant):
       - Setup: `state.alpha_reg = "HELLO"`. Dispatch `Op::Asto(7)`. Assert `state.text_regs.get(&7).is_some()` and `state.regs[7] == HpNum::zero()`.
       - Put 3.14 in X. Dispatch `Op::Sto(7)`. Assert `state.text_regs.get(&7) == None` (sidecar CLEARED, D-23.4) AND `state.regs[7] == HpNum::from_str("3.14")` (numeric write succeeded).
       - Clear ALPHA. Dispatch `Op::Arcl(7)`. Assert it appends the formatted numeric (e.g. "3.1400" in default Fix(4)) — NOT the stale "HELLO".

    4. `clreg_clears_both_regs_and_text_regs`:
       - Setup: pre-populate `state.text_regs.insert(3, "FOO".into())`, `state.text_regs.insert(8, "BAR".into())`. Dispatch `Op::Clreg`.
       - Assert `state.text_regs.is_empty()`. Assert all `state.regs` are `HpNum::zero()`.

    5. `arcl_out_of_range_reg_returns_invalid_op_without_panic`:
       - Setup: default state (regs.len() == 100). Dispatch `Op::Arcl(200)` (200 > 100). Assert the result is `Err(HpError::InvalidOp)`. Assert ALPHA is unchanged.
       - **Comment:** After W-2's strengthening of `op_arcl`, the leading `(reg as usize) >= state.regs.len()` bounds check is the code path that raises `InvalidOp` here — `text_regs.get(&200)` is never consulted. (Pre-W-2 this test passed only because the empty `text_regs` map fell through to a separate `state.regs.get(200).ok_or(InvalidOp)?`; the new ordering rejects 200 before any sidecar lookup.)

    5b. `arcl_rejects_out_of_range_reg_even_when_text_regs_has_stale_entry` (W-2 / W-3 demonstrator):
       - Setup: default state. Pre-populate `state.text_regs.insert(200, "X".into())` (simulates a hand-edited autosave.json with an out-of-range shadow entry — T-23-01).
       - Dispatch `Op::Arcl(200)`. Assert `Err(HpError::InvalidOp)`. Assert `state.alpha_reg` is unchanged (the bogus "X" was NOT appended).
       - **Comment:** This test pins the W-2 strengthening — without the leading bounds check in `op_arcl`, this would return `Ok(())` and append "X" to ALPHA, weaponizing the tampered save file.

    6. `asto_silent_24_char_cap_on_subsequent_arcl`:
       - Setup: `state.alpha_reg = "AAAAAAAAAAAAAAAAAAAAAAA"` (23 A's). Pre-populate `state.text_regs.insert(0, "BCDEF".into())` (5 chars). Dispatch `Op::Arcl(0)`. Assert `state.alpha_reg.chars().count() == 24` and the tail is the single 'B' (only the first char fits before the 24 cap).

    7. `serde_default_loads_v21_save_file_without_text_regs_field` (D-23.13):
       - Construct a `serde_json::Value` representing a CalcState JSON WITHOUT a `text_regs` field (one of the existing fixtures from Phase 21 — `hp41-core/tests/fixtures/v20-autosave.json` — or build inline by serializing a default state, removing the `text_regs` key, and deserializing). Assert the deserialized state has `text_regs == BTreeMap::new()`.
       - If `v20-autosave.json` exists and is structured for round-trip tests, EXTEND it; otherwise build the round-trip inline so the test stays self-contained.

    Use `dispatch()` from `hp41_core::ops::dispatch` (re-export path — verify the exact module path before writing) and the `Op` enum directly. The test file is parallel to `hp41-core/tests/numerical_accuracy.rs` — same structure.
  </action>
  <verify>
    <automated>just test-core 2>&amp;1 | grep -E "phase23_arcl_asto|test result" | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - File `hp41-core/tests/phase23_arcl_asto.rs` exists with 8 `#[test]` functions covering the cases above (tests #1, #2, #3, #4, #5, #5b, #6, #7).
    - `just test-core` reports all 8 phase23_arcl_asto tests passing.
    - Test #1 explicitly compares against `format_hpnum(..., DisplayMode::Sci(3))` output (re-uses the helper rather than hardcoding the string — confirms SC#1's "respects display mode" semantic via the same source of truth).
    - Test #3 asserts both `state.text_regs.get(&7) == None` AND `state.regs[7] != HpNum::zero()` after `Op::Sto(7)` — the no-drift invariant pinned by D-23.4.
    - Test #5b pre-populates `state.text_regs.insert(200, "X".into())` and asserts `Op::Arcl(200)` returns `Err(HpError::InvalidOp)` AND `state.alpha_reg` is unchanged — pins the W-2 strengthening.
    - Test #7 deserializes a JSON object lacking `text_regs` and asserts the field defaults to `BTreeMap::new()` — confirms the `#[serde(default)]` annotation per D-23.13.
    - Coverage gate (`just coverage` not run here — Phase 27 raises the gate; verify only that the new test file does not LOWER coverage by running `just coverage 2>&amp;1 | tail -5` and comparing the reported percentage to STATE.md's 92.68% baseline. Non-regression only — no new gate enforcement.)
  </acceptance_criteria>
  <done>
    8 integration tests in `hp41-core/tests/phase23_arcl_asto.rs` all pass under `just test-core`. SC#1 + SC#2 + D-23.4 + D-23.13 + W-2 strengthening are now mechanically verified. Coverage non-regression confirmed.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| program-step → text_regs | A user-authored keystroke program may execute `ASTO nn` with arbitrary `nn` — bounds-check is the only defense; no panic budget. |
| save-file → CalcState | A hand-edited or third-party autosave.json deserializes into `text_regs: BTreeMap<u8, String>` — the BTreeMap accepts any u8 key, but ARCL/ASTO callers will fail-safe on out-of-range register access against `state.regs.len()`. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-23-01 | Tampering | hand-edited autosave.json with `text_regs: {200: "..."}` (key out of regs range) | mitigate | `op_arcl` performs a **leading `(reg as usize) >= state.regs.len()` bounds check** (W-2 strengthening of D-23.3) BEFORE the text_regs lookup; `op_asto` checks `(reg as usize) >= state.regs.len()` before insert. Out-of-range entries in the loaded map are inert (rejected by the leading `regs.len()` bounds check in `op_arcl`/`op_asto`). Integration test #5b in `phase23_arcl_asto.rs` pre-populates `text_regs[200] = "X"` and asserts `op_arcl(200)` returns `InvalidOp` with ALPHA unchanged. |
| T-23-02 | Denial-of-Service | unbounded growth of `text_regs` via repeated `ASTO nn` with varying nn from a runaway program | accept | `text_regs` is capped by `state.regs.len()` distinct keys (≤ SIZE ≤ 319 per HP-41CV spec). Each entry is ≤ 6 chars (`chars().take(6)`). Maximum total footprint < 2 KB. No DoS surface. |
| T-23-03 | Information-disclosure | text_regs serialized to autosave.json may expose user-entered ALPHA strings | accept | Autosave already serializes `state.alpha_reg`, `state.program` (Op::AlphaAppend chars), and `state.assignments` (Phase 22 names). Adding `text_regs` does not widen this surface — same locality (the user's own machine, ~/.hp41/autosave.json). No new disclosure path. |
| T-23-04 | Elevation-of-privilege | sidecar-clearing audit miss leaves stale text after numeric STO, silently changing ARCL output | mitigate | D-23.4 audit is a Wave-0 task with dedicated unit tests in registers.rs AND integration test #3 in phase23_arcl_asto.rs. `op_sto_arith_stack` is the one path that does NOT need clearing — explicit comment documents the audit outcome. |
| T-23-05 | Repudiation | n/a | accept | Single-user single-process emulator; no audit log requirement. |
</threat_model>

<verification>
- `just check` — exhaustive matches compile in all 4 landing places (Op enum, dispatch, execute_op, both prgm_display.rs).
- `just test-core` — all new unit tests in alpha.rs + registers.rs + the integration suite in tests/phase23_arcl_asto.rs pass green.
- `just ci` — full CI suite green (covers fmt, clippy with `-D clippy::unwrap_used`, test, coverage non-regression).
- SC-4 invariant grep: `grep -rnE "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns NOTHING.
- Backward-compat check: existing `~/.hp41/autosave.json` from v2.1 loads in the new build without error (manual smoke test or covered by Task 3 test #7).
</verification>

<success_criteria>
1. **SC#1 (FN-ALPHA-01)**: `ARCL 05` with R05 = 3.14 in FIX 2 appends "3.14" to ALPHA; switching to SCI 3 and re-running `ARCL 05` appends a DIFFERENT formatted string (per Task 3 test #1).
2. **SC#2 (FN-ALPHA-02)**: `ASTO 12` with ALPHA = "GOODBYE" then `ARCL 12` produces ALPHA = "GOODBY" (first 6 chars round-trip via the text_regs sidecar, per Task 3 test #2).
3. **D-23.4 invariant**: every numeric STO/STO-arith/CLREG clears the matching `text_regs` entry — verified by Task 1 + Task 3 test #3.
4. **D-23.13 save-file compat**: v1.x/v2.0/v2.1 autosave.json files without `text_regs` load cleanly with the field defaulting to `BTreeMap::new()` — verified by Task 3 test #7.
5. **D-23.12 4-place landing**: both `Op::Arcl(u8)` and `Op::Asto(u8)` appear in the Op enum, dispatch(), execute_op(), AND both prgm_display.rs copies — verified by `just check`.
</success_criteria>

<output>
After completion, create `.planning/phases/23-alpha-operations/23-01-arcl-asto-SUMMARY.md`.
</output>
</content>
