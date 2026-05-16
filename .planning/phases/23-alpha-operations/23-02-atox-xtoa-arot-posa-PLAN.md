---
phase: 23-alpha-operations
plan: 02
type: execute
wave: 2
depends_on:
  - 23-01
files_modified:
  - hp41-core/src/ops/alpha.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-cli/src/prgm_display.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-core/tests/phase23_atox_xtoa_arot_posa.rs
autonomous: true
requirements:
  - FN-ALPHA-03
  - FN-ALPHA-04
  - FN-ALPHA-05
  - FN-ALPHA-06

must_haves:
  truths:
    - "ATOX pops the first char off ALPHA and pushes its ASCII codepoint (8-bit capped) onto X with stack lift"
    - "XTOA converts X mod 256 to a character and appends it to ALPHA without consuming X"
    - "AROT rotates ALPHA by X chars (positive = left, negative = right) and preserves X"
    - "POSA single-char path finds the position of (X as ASCII char) in ALPHA, replacing X with the position or -1"
    - "All 4 new Op variants land in dispatch + execute_op + both prgm_display.rs copies (no exhaustive-match miss)"
  artifacts:
    - path: "hp41-core/src/ops/alpha.rs"
      provides: "op_atox + op_xtoa + op_arot + op_posa + inline unit tests"
      contains: "pub fn op_atox"
    - path: "hp41-core/src/ops/mod.rs"
      provides: "Op::Atox, Op::Xtoa, Op::Arot, Op::Posa variants AT END of enum + dispatch() arms"
      contains: "Atox,"
    - path: "hp41-core/src/ops/program.rs"
      provides: "execute_op() arms for Atox/Xtoa/Arot/Posa"
      contains: "Op::Atox"
    - path: "hp41-cli/src/prgm_display.rs"
      provides: "op_display_name arms for all 4 new variants"
      contains: "\"ATOX\""
    - path: "hp41-gui/src-tauri/src/prgm_display.rs"
      provides: "op_display_name arms for all 4 new variants (mirror)"
      contains: "\"ATOX\""
    - path: "hp41-core/tests/phase23_atox_xtoa_arot_posa.rs"
      provides: "Integration suite for SC#3 (ATOX/XTOA round-trip), SC#4 (AROT directions), SC#5 (POSA single-char)"
  key_links:
    - from: "Op::Atox in ops/mod.rs"
      to: "op_atox in ops/alpha.rs"
      via: "dispatch() arm"
      pattern: "Op::Atox => alpha::op_atox"
    - from: "op_atox"
      to: "apply_lift_effect(state, LiftEffect::Enable) then direct stack T←Z←Y←X←code assignment"
      via: "canonical lift-then-push idiom mirrored from op_pi"
      pattern: "LiftEffect::Enable"
    - from: "op_arot"
      to: "alpha_reg.chars().collect::<Vec<char>>() + rem_euclid rotation"
      via: "multibyte-safe rotation; negative N handled by rem_euclid"
      pattern: "rem_euclid"
    - from: "op_posa"
      to: "alpha_reg.chars().position(|c| c == needle)"
      via: "single-char search; -1 sentinel for not-found"
      pattern: "\\.position\\("
---

<objective>
Land the remaining 4 ALPHA-register ops in `hp41-core`: `ATOX`,
`XTOA`, `AROT`, `POSA` (single-char). These ops touch only `state.alpha_reg`
and `state.stack.x` — no new persistent state — so they layer cleanly on
the foundation laid by 23-01. The 4-place Op-variant landing rule
(D-23.12) is the only cross-cutting concern.

Purpose: Phase 23 delivers FN-ALPHA-03 (ATOX), FN-ALPHA-04 (XTOA),
FN-ALPHA-05 (AROT), FN-ALPHA-06 (POSA). Combined with 23-01's
ARCL/ASTO, this completes the 6-op ALPHA expansion in hp41-core.
IND variants are deferred to Phase 24; CLI/GUI wiring to Phases 25/26.

Output: 4 new `Op` variants landed in all 4 places, 4 new functions in
`alpha.rs` (or a new `alpha_ops.rs` if alpha.rs exceeds ~400 lines after
plan 01 — implementer's discretion per D-23.18), and an integration test
suite covering each SC plus the documented divergences (multi-char POSA
deferred, upper-ASCII XTOA → '?', ATOX 8-bit cap).
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

<!-- Predecessor — sets up state.text_regs and the 4-place landing pattern for ARCL/ASTO -->
@.planning/phases/23-alpha-operations/23-01-arcl-asto-PLAN.md

<!-- Files this plan modifies — read for current state and patterns -->
@hp41-core/src/ops/alpha.rs
@hp41-core/src/ops/mod.rs
@hp41-core/src/ops/program.rs
@hp41-cli/src/prgm_display.rs
@hp41-gui/src-tauri/src/prgm_display.rs

<!-- Canonical helpers / reuse targets — read for signatures -->
@hp41-core/src/ops/math.rs
@hp41-core/src/num.rs
@hp41-core/src/stack.rs
@hp41-core/src/error.rs

<interfaces>
<!-- Key signatures the executor needs in scope. Do NOT re-derive — read the source. -->

From hp41-core/src/num.rs:
- `pub fn trunc_int(&self) -> HpNum` — toward-zero integer truncation; used by AROT/XTOA/POSA.
- `pub fn inner(&self) -> Decimal` — pulls underlying Decimal for `.try_into::<i64>()` chain.
- `impl From<i32> for HpNum` — ATOX pushes ASCII code; POSA pushes position result.

From hp41-core/src/stack.rs:
- `pub enum LiftEffect { Enable, Disable, Neutral }`
- `pub fn apply_lift_effect(state: &mut CalcState, effect: LiftEffect)`
- Stack fields: `x: HpNum`, `y: HpNum`, `z: HpNum`, `t: HpNum`, `lastx: HpNum`, `lift_enabled: bool`. ATOX writes `t = z.clone(); z = y.clone(); y = x.clone(); x = HpNum::from(code)` AFTER `apply_lift_effect(state, LiftEffect::Enable)`.

From hp41-core/src/ops/math.rs::op_pi (lines ~290-305) — CANONICAL lift-then-push precedent for ATOX:
- Sets `state.stack.lift_enabled = true`, calls `enter_number(state, value)`, then `apply_lift_effect(state, LiftEffect::Enable)`.
- ATOX MAY mirror this exactly using `enter_number` (cleaner — let `enter_number` handle the T←Z←Y←X shift), OR direct-assign the four fields after `apply_lift_effect`. Either approach is faithful; the CONTEXT D-23.10 sketch shows direct-assign. Implementer chooses; document the choice in the function doc comment.

From hp41-core/src/ops/alpha.rs (after plan 01):
- `op_alpha_append` precedent for the 24-char `chars().count() < 24` cap (XTOA reuses).

LiftEffect summary (D-23.16):
- Atox: Enable (pushes a new value, shifts stack up)
- Xtoa: Neutral (reads X but does not consume; ALPHA-side append only)
- Arot: Neutral (reads X but does not consume; ALPHA-side rotate only)
- Posa: Disable (replaces X — drops the previous X content)
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add Op::Atox, Op::Xtoa, Op::Arot, Op::Posa + 4-place landing + functions in alpha.rs</name>
  <files>hp41-core/src/ops/alpha.rs, hp41-core/src/ops/mod.rs, hp41-core/src/ops/program.rs, hp41-cli/src/prgm_display.rs, hp41-gui/src-tauri/src/prgm_display.rs</files>
  <read_first>
    - hp41-core/src/ops/alpha.rs (state after 23-01: contains op_alpha_toggle/append/clear/backspace + op_arcl + op_asto)
    - hp41-core/src/ops/mod.rs lines around the new Op::Arcl(u8) and Op::Asto(u8) tail (the 4 new variants slot immediately after these per D-23.13 — preserve discriminant order)
    - hp41-core/src/ops/math.rs::op_pi (lines ~290-305) — CANONICAL lift-then-push idiom for ATOX (D-23.10 note)
    - hp41-core/src/num.rs (HpNum::trunc_int, HpNum::inner, impl From<i32> for HpNum — all three used by these 4 ops)
    - hp41-core/src/stack.rs (Stack struct field layout — ATOX direct-assigns t/z/y/x after apply_lift_effect; AROT/POSA read state.stack.x without consuming)
    - hp41-core/src/error.rs (HpError::InvalidOp — the ONLY variant any of these 4 ops returns per D-23.14; do NOT add new variants)
    - hp41-cli/src/prgm_display.rs + hp41-gui/src-tauri/src/prgm_display.rs (state after 23-01: have ARCL/ASTO arms; the 4 new bare-string arms slot immediately after)
    - .planning/phases/23-alpha-operations/23-CONTEXT.md §D-23.7 (POSA), §D-23.8 (AROT signature + rem_euclid), §D-23.9 (AROT edge cases: empty ALPHA, |N| > len, non-integer X silently truncated), §D-23.10 (ATOX lift-Enable + 8-bit cap), §D-23.11 (XTOA mod 256 + '?' for 128-255), §D-23.16 (LiftEffect summary)
    - .planning/phases/23-alpha-operations/23-CONTEXT.md §"Concrete signature sketches" lines 592-651 (reference signatures for all 4 ops — mirror, but pull the actual num.rs / stack.rs / HpError signatures from the canonical source files)
  </read_first>
  <behavior>
    Op variants (D-23.12, D-23.13 — land AT END of Op enum, AFTER Arcl/Asto from plan 01):
    - `Op::Atox` — first ALPHA char → ASCII code in X (lift Enable)
    - `Op::Xtoa` — X mod 256 → ASCII char appended to ALPHA (X preserved, lift Neutral)
    - `Op::Arot` — rotate ALPHA by X chars, X preserved, mod len via `rem_euclid` (lift Neutral)
    - `Op::Posa` — single-char POSA: X is ASCII code in 0..=127 → position in ALPHA → X (lift Disable)
    - All four are bare variants (NO parameter — AROT reads X, POSA reads X). Add dispatch arms in `ops/mod.rs::dispatch()` near the Arcl/Asto arms from plan 01.
    - Add execute_op arms in `ops/program.rs::execute_op` matching the same pattern.
    - Add `op_display_name` arms in BOTH `prgm_display.rs` copies: `Op::Atox => "ATOX".to_string()`, `Op::Xtoa => "XTOA".to_string()`, `Op::Arot => "AROT".to_string()`, `Op::Posa => "POSA".to_string()`. Bare strings, no parameter format.

    `op_atox(state: &mut CalcState) -> Result<(), HpError>` (D-23.10):
    - Determine the ASCII code: if `state.alpha_reg.chars().next()` is Some(c), build a Vec<char>, drop chars[0], rebuild `state.alpha_reg`, code = `u32::from(c).min(255) as i32`. If None (empty ALPHA), code = 0.
    - `apply_lift_effect(state, LiftEffect::Enable)` FIRST (D-23.10 ordering — mirrors op_pi).
    - Then direct-assign: `state.stack.t = state.stack.z.clone(); state.stack.z = state.stack.y.clone(); state.stack.y = state.stack.x.clone(); state.stack.x = HpNum::from(code);`
    - (Alternative: use `state.stack.lift_enabled = true; enter_number(state, HpNum::from(code))` — equivalent in effect. Doc comment must state which path was chosen, mirroring op_pi's doc.)
    - 8-bit cap is documented in the doc comment: "Multibyte first-char codepoints > 255 are capped to 255 (e.g., Σ U+03A3 → 255). HP-41 hardware codes 128-255 are not preserved as glyphs; documented divergence per D-23.10."

    `op_xtoa(state: &mut CalcState) -> Result<(), HpError>` (D-23.11):
    - Compute `i_dec = state.stack.x.trunc_int().inner()` (Decimal).
    - `i_i64: i64 = i_dec.try_into().map_err(|_| HpError::InvalidOp)?` (rejects overflow per D-23.14).
    - `code: u32 = (i_i64.rem_euclid(256)) as u32`.
    - `c: char = if code < 128 { code as u8 as char } else { '?' }` (D-23.11: HP-41 upper-ASCII glyphs are not in our String/UTF-8 model).
    - If `state.alpha_reg.chars().count() < 24` then `state.alpha_reg.push(c)` (24-char cap, silent discard per op_alpha_append precedent).
    - `apply_lift_effect(state, LiftEffect::Neutral)`. X is NOT consumed.

    `op_arot(state: &mut CalcState) -> Result<(), HpError>` (D-23.8, D-23.9):
    - `apply_lift_effect(state, LiftEffect::Neutral)` (early so error paths still settle lift state — per Phase 21/22 precedent).
    - `len = state.alpha_reg.chars().count()`. If `len == 0`, return `Ok(())` (empty ALPHA no-op, X preserved).
    - `n_dec = state.stack.x.trunc_int().inner()` (faithful HP-41CV: silently truncates non-integer X per D-23.9).
    - `n_i64: i64 = n_dec.try_into().map_err(|_| HpError::InvalidOp)?`.
    - `n = n_i64.rem_euclid(len as i64) as usize` (handles negative N: -1 of "HELLO" → rem_euclid(-1, 5) = 4 → "OHELL"). `rem_euclid` is required (NOT `%`) for the negative-N path.
    - Rebuild: `let chars: Vec<char> = state.alpha_reg.chars().collect(); state.alpha_reg = chars[n..].iter().chain(chars[..n].iter()).collect();`
    - X is NOT consumed (Neutral lift — already applied).

    `op_posa(state: &mut CalcState) -> Result<(), HpError>` (D-23.7):
    - `let x = state.stack.x.clone(); let i = x.trunc_int();`
    - `if i != x { return Err(HpError::InvalidOp); }` (non-integer X rejected per D-23.7 — STRICTER than AROT; document the divergence in the doc comment).
    - `code_i64: i64 = i.inner().try_into().map_err(|_| HpError::InvalidOp)?`.
    - `if !(0..=127).contains(&code_i64) { return Err(HpError::InvalidOp); }` (ASCII range gate).
    - `needle = (code_i64 as u8) as char`.
    - `pos: i32 = state.alpha_reg.chars().position(|c| c == needle).map(|p| p as i32).unwrap_or(-1)`.
    - `state.stack.x = HpNum::from(pos);` (REPLACES X — Disable lift).
    - `apply_lift_effect(state, LiftEffect::Disable)`.
    - Doc comment notes: SC#5 wording specifies -1 for not-found (other HP-41 sources return haystack length; we pick -1 per ROADMAP SC#5 explicit wording).

    Inline unit tests in `alpha.rs` (under existing `#[cfg(test)] mod tests`):
    - `test_atox_pops_first_char_pushes_ascii_code_with_lift` — ALPHA="ABC" → ATOX → X=65, Y=prior_X, ALPHA="BC".
    - `test_atox_empty_alpha_pushes_zero_with_lift` — ALPHA="" → ATOX → X=0, Y=prior_X, ALPHA="".
    - `test_atox_multibyte_first_char_capped_at_255` — ALPHA="Σ ..." (Σ is U+03A3, decimal 931) → ATOX → X=255.
    - `test_xtoa_appends_ascii_char_x_preserved` — X=66 (B), ALPHA="" → XTOA → ALPHA="B", X still 66.
    - `test_xtoa_upper_ascii_maps_to_question_mark` — X=200 → XTOA → ALPHA ends with '?'.
    - `test_xtoa_silent_24_char_cap` — ALPHA = 24 A's, X=66 → XTOA → ALPHA still 24 chars (no-op append).
    - `test_arot_positive_n_left_rotation` — ALPHA="HELLO", X=2 → AROT → ALPHA="LLOHE" (SC#4 forward).
    - `test_arot_negative_n_right_rotation` — ALPHA="HELLO", X=-1 → AROT → ALPHA="OHELL" (SC#4 reverse via rem_euclid).
    - `test_arot_n_greater_than_len_modulo` — ALPHA="HELLO", X=7 → AROT → ALPHA="LLOHE" (7 % 5 = 2).
    - `test_arot_empty_alpha_is_noop` — ALPHA="", X=3 → AROT → ALPHA="", X preserved.
    - `test_arot_x_preserved_neutral_lift` — pre-condition X=2.0, post-condition state.stack.x == 2.0 (clone-compare).
    - `test_posa_finds_single_char` — ALPHA="THE QUICK BROWN FOX", X=81 (ASCII 'Q') → POSA → X=4 (SC#5).
    - `test_posa_not_found_returns_minus_one` — ALPHA="HELLO", X=90 ('Z') → POSA → X=-1.
    - `test_posa_non_integer_x_returns_invalid_op` — X=2.5 → POSA → Err(InvalidOp), ALPHA unchanged, X unchanged.
    - `test_posa_out_of_range_x_returns_invalid_op` — X=200 (out of 0..=127) → POSA → Err(InvalidOp).
  </behavior>
  <action>
    Implement per D-23.7..D-23.11 + D-23.12 + D-23.16. Extend `hp41-core/src/ops/alpha.rs` in place (after 23-01 it's around ~250 lines; adding these 4 functions + tests keeps it under the 400-line split threshold from D-23.18 — but if implementer judges otherwise after 23-01 lands, create `hp41-core/src/ops/alpha_ops.rs` and put the 4 functions there). The 4-place Op landing (D-23.12) is mandatory — every variant in `Op::` enum, `dispatch()`, `execute_op()`, both `prgm_display.rs` copies. Reuse `HpNum::trunc_int()`, `HpNum::inner()`, `impl From<i32> for HpNum`, `apply_lift_effect(state, LiftEffect)`. NO new HpError variants — InvalidOp covers all four error paths (D-23.14). All ALPHA mutation must use `chars()` (never byte indices) per D-23.14 / ROADMAP cross-cutting constraint. `rem_euclid` (NOT `%`) for AROT — `%` would mis-handle negative N. ATOX direct-assign vs `enter_number`: pick one, mirror op_pi's doc-comment style, and document the choice.
  </action>
  <verify>
    <automated>just check &amp;&amp; just test-core 2>&amp;1 | tail -60</automated>
  </verify>
  <acceptance_criteria>
    - `grep -nE "Atox,|Xtoa,|Arot,|Posa,?$" hp41-core/src/ops/mod.rs` returns at least 4 matches (enum tail).
    - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-core/src/ops/mod.rs` returns at least 4 (one dispatch arm per variant).
    - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-core/src/ops/program.rs` returns at least 4 (execute_op arms).
    - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-cli/src/prgm_display.rs` returns at least 4.
    - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-gui/src-tauri/src/prgm_display.rs` returns at least 4.
    - `grep -c "pub fn op_atox" hp41-core/src/ops/alpha.rs` (or alpha_ops.rs if implementer split) returns 1; same for op_xtoa, op_arot, op_posa.
    - `grep -c "rem_euclid" hp41-core/src/ops/alpha.rs` returns at least 2 (AROT for char shift, XTOA for code mod 256).
    - `grep -v '^//' hp41-core/src/ops/alpha.rs | grep -c "\\.unwrap()"` returns 0 outside `#[cfg(test)]` modules (zero-panic per D-23.14).
    - `just check` passes (exhaustive matches compile in all 4 places).
    - `just test-core` passes including the new 15 unit tests (or however many — count is illustrative).
    - SC-4 invariant grep `grep -rnE "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns nothing (D-23.15).
    - No new HpError variants in `hp41-core/src/error.rs` (D-23 reuses InvalidOp only).
  </acceptance_criteria>
  <done>
    All 4 ops implemented per their D-23 decisions. All 4 land in dispatch + execute_op + both prgm_display.rs copies. `just check` + `just test-core` green. Inline unit tests cover SC#3/SC#4/SC#5 happy paths plus documented divergences (8-bit cap, '?' upper-ASCII, -1 not-found, non-integer rejection for POSA).
  </done>
</task>

<task type="auto">
  <name>Task 2: Integration test suite — SC#3, SC#4, SC#5, ATOX/XTOA round-trip property</name>
  <files>hp41-core/tests/phase23_atox_xtoa_arot_posa.rs</files>
  <read_first>
    - hp41-core/tests/phase23_arcl_asto.rs (from plan 01 — sibling integration suite; mirror its top-of-file structure and dispatch-helper pattern)
    - hp41-core/tests/numerical_accuracy.rs (top of file — `#[allow(clippy::unwrap_used)]` test module header)
    - hp41-core/src/ops/mod.rs (dispatch entry point + Op enum — the integration test calls `dispatch(state, &Op::Atox)` etc.)
    - .planning/phases/23-alpha-operations/23-CONTEXT.md §"Specifics" lines 678-702 (AROT direction examples; XTOA 128-255 → '?'; POSA -1 for not-found)
  </read_first>
  <action>
    Create `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs` paralleling `phase23_arcl_asto.rs`. Header: `#![allow(clippy::unwrap_used)]`. Test cases (drive via `dispatch(state, &Op::XXX)` — NOT by calling op_xxx directly, so the test covers the dispatch arm too):

    1. `atox_pops_first_char_pushes_ascii_with_lift` (SC#3 forward):
       - Setup: `state.alpha_reg = "A"`, `state.stack.x = HpNum::from(99)`.
       - Dispatch `Op::Atox`. Assert: `state.alpha_reg.is_empty()`, `state.stack.x == HpNum::from(65)`, `state.stack.y == HpNum::from(99)` (lifted).

    2. `xtoa_appends_ascii_char_preserves_x` (SC#3 reverse):
       - Setup: `state.alpha_reg = ""`, `state.stack.x = HpNum::from(66)`.
       - Dispatch `Op::Xtoa`. Assert: `state.alpha_reg == "B"`, `state.stack.x == HpNum::from(66)` (preserved).

    3. `atox_xtoa_round_trip_preserves_ascii_0_to_127` (property-style — at least 4 sample values, no proptest dep needed):
       - For code in [32, 65, 97, 126]: `state.alpha_reg = (code as u8 as char).to_string()`, dispatch Atox, assert X == HpNum::from(code), clear ALPHA, dispatch Xtoa, assert `state.alpha_reg == (code as u8 as char).to_string()`. Round-trip exact.

    4. `xtoa_upper_ascii_maps_to_question_placeholder` (D-23.11 divergence):
       - For X in [128, 200, 255]: dispatch Xtoa, assert ALPHA ends with '?'.

    5. `arot_left_rotation_two_of_hello_produces_lloghe` (SC#4 forward — exact wording uses "LLOHE"):
       - Setup: `state.alpha_reg = "HELLO"`, `state.stack.x = HpNum::from(2)`.
       - Dispatch `Op::Arot`. Assert: `state.alpha_reg == "LLOHE"`, X preserved == 2.

    6. `arot_right_rotation_negative_one_of_hello_produces_ohell` (SC#4 reverse):
       - Setup: `state.alpha_reg = "HELLO"`, `state.stack.x = HpNum::from(-1)`.
       - Dispatch `Op::Arot`. Assert: `state.alpha_reg == "OHELL"`.

    7. `arot_modulo_handles_n_greater_than_len`:
       - `"HELLO"` with X=7 → after AROT → `"LLOHE"` (same as X=2).

    8. `arot_empty_alpha_is_noop_preserves_x`:
       - `state.alpha_reg = ""`, X = HpNum::from(3). Dispatch Arot. Assert ALPHA empty, X == 3.

    9. `posa_single_char_finds_position_4_for_q_in_the_quick` (SC#5):
       - `state.alpha_reg = "THE QUICK BROWN FOX"`, `state.stack.x = HpNum::from(81)` (ASCII 'Q' = 81).
       - Dispatch `Op::Posa`. Assert `state.stack.x == HpNum::from(4)`.

    10. `posa_not_found_returns_minus_one` (SC#5 negative path):
        - `state.alpha_reg = "HELLO"`, X = HpNum::from(90) ('Z').
        - Dispatch Posa. Assert `state.stack.x == HpNum::from(-1)`.

    11. `posa_rejects_non_integer_x`:
        - X = HpNum::from_str("2.5"). Dispatch Posa. Assert Err(InvalidOp). Assert X unchanged.

    12. `posa_rejects_out_of_range_x`:
        - X = HpNum::from(200). Dispatch Posa. Assert Err(InvalidOp).

    13. `arot_silently_truncates_non_integer_x` (D-23.9 divergence from POSA's strict rejection):
        - `state.alpha_reg = "HELLO"`, X = HpNum::from_str("2.7"). Dispatch Arot. Assert ALPHA == "LLOHE" (truncated to 2). Document in test comment that AROT is faithful HP-41CV (silent trunc) while POSA is stricter (rejects) — divergence is intentional per D-23.9 / D-23.7.

    Use `hp41_core::ops::dispatch`, `hp41_core::ops::Op`, `hp41_core::state::CalcState`, `hp41_core::num::HpNum`, `hp41_core::error::HpError` (verify exact module paths via the source files before writing).
  </action>
  <verify>
    <automated>just test-core 2>&amp;1 | grep -E "phase23_atox_xtoa_arot_posa|test result" | tail -10</automated>
  </verify>
  <acceptance_criteria>
    - File `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs` exists with ≥13 `#[test]` functions covering the cases above.
    - `just test-core` reports all phase23_atox_xtoa_arot_posa tests passing.
    - Test #5 asserts the exact string `"LLOHE"` (SC#4 wording from ROADMAP).
    - Test #6 asserts the exact string `"OHELL"` (SC#4 wording).
    - Test #9 asserts `X == HpNum::from(4)` for the "Q" in "THE QUICK BROWN FOX" (SC#5 wording).
    - Test #10 asserts `X == HpNum::from(-1)` for not-found (SC#5 wording).
    - Test #11 confirms POSA rejects non-integer X (D-23.7 stricter than AROT).
    - Test #13 confirms AROT silently truncates non-integer X (D-23.9, intentional divergence).
    - Coverage non-regression: `just coverage 2>&amp;1 | tail -5` reports `hp41-core` line coverage no lower than the STATE.md baseline of 92.68%.
  </acceptance_criteria>
  <done>
    Integration suite for SC#3, SC#4, SC#5 lands in `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs`. All tests green under `just test-core`. The intentional divergence between AROT's silent truncation and POSA's strict integer rejection is mechanically pinned by tests #11 + #13. Coverage non-regression confirmed.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| program-step → ALPHA mutation | A keystroke program may execute `ATOX`/`XTOA`/`AROT`/`POSA` repeatedly. None of these allocate unboundedly or escape `hp41-core`. |
| program-step → X register | AROT/XTOA/POSA read X. POSA writes X. The integer-conversion path uses `try_into::<i64>()` so overflow is caught as `InvalidOp`, not silently swallowed (D-23.14). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-23-06 | Tampering | crafted X containing a huge integer (e.g., `Decimal::MAX`) passed to AROT/XTOA/POSA | mitigate | `Decimal::try_into::<i64>().map_err(|_| HpError::InvalidOp)` rejects overflow; subsequent `.rem_euclid(256)` / `.rem_euclid(len as i64)` cannot panic. POSA's additional `(0..=127).contains` gate narrows the surface further. |
| T-23-07 | Tampering | crafted ALPHA containing high-codepoint Unicode (e.g., a 4-byte UTF-8 emoji) | mitigate | All ALPHA mutation uses `chars()` not byte slicing (D-23.14). ATOX caps `u32::from(c).min(255)` for codepoints > 255. AROT/POSA operate on char-indices, not byte-indices — no panic possible. |
| T-23-08 | Denial-of-Service | AROT/XTOA in a tight loop with bounded ALPHA (≤24 chars) | accept | Each op is O(n) over n ≤ 24 chars. Total cost trivial. `enter_number` etc. are O(1). No DoS surface. |
| T-23-09 | Information-disclosure | ATOX leaks the first ALPHA char's codepoint into X (visible in stack readout) | accept | This is the documented behavior of the operation — not a leak. User explicitly requested it. |
| T-23-10 | Repudiation | n/a | accept | Single-user emulator. |
| T-23-11 | Elevation-of-privilege | a missed exhaustive-match arm in `prgm_display.rs` causes a runtime program-listing panic | mitigate | Compile-time exhaustive-match enforcement: `just check` will fail to compile if any of the 4 new variants is missed in either `prgm_display.rs` copy. CI runs `just ci` which includes `just check`. The 4-place landing rule (D-23.12) is mechanically enforced. |
</threat_model>

<verification>
- `just check` — exhaustive matches compile in all 4 landing places (Op enum, dispatch, execute_op, both prgm_display.rs).
- `just test-core` — all new unit tests in alpha.rs + the integration suite in tests/phase23_atox_xtoa_arot_posa.rs pass green.
- `just ci` — full CI green (covers fmt, clippy `-D clippy::unwrap_used`, test, coverage non-regression).
- SC-4 invariant: `grep -rnE "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns nothing.
- Phase 23 wrap-up: combined with 23-01, all 6 ALPHA-op variants now ship; the existing v2.1 autosave.json round-trip test from 23-01 still passes (since this plan adds no new persistent state — only `Op` enum variants land at the end, preserving discriminants per D-23.13).
</verification>

<success_criteria>
1. **SC#3 (FN-ALPHA-03 + FN-ALPHA-04)**: ATOX of `ALPHA="A"` produces X=65; XTOA of X=66 with empty ALPHA produces `ALPHA="B"`. Round-trip exact for ASCII 0..=127 (Task 2 test #3).
2. **SC#4 (FN-ALPHA-05)**: AROT of `"HELLO"` with X=2 produces `"LLOHE"`; AROT with X=-1 produces `"OHELL"` (Task 2 tests #5 + #6).
3. **SC#5 (FN-ALPHA-06, single-char path)**: POSA of `ALPHA="THE QUICK BROWN FOX"` with X=81 ('Q') produces X=4; missing char produces X=-1 (Task 2 tests #9 + #10). Multi-char POSA explicitly deferred to v3.x per D-23.6 — documented in CLAUDE.md by Phase 25.
4. **D-23.12 4-place landing**: `Op::Atox`, `Op::Xtoa`, `Op::Arot`, `Op::Posa` all appear in the Op enum, dispatch(), execute_op(), AND both prgm_display.rs copies — verified by `just check`.
5. **D-23.14 zero-panic**: no new `.unwrap()` calls outside `#[cfg(test)]` modules. All numeric conversions use `try_into().map_err(|_| InvalidOp)`. All ALPHA mutation uses `chars()`.
</success_criteria>

<output>
After completion, create `.planning/phases/23-alpha-operations/23-02-atox-xtoa-arot-posa-SUMMARY.md`.
</output>
</content>
</invoke>