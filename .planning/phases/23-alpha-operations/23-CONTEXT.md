# Phase 23: ALPHA Operations — Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Land 6 HP-41CV ROM ops in `hp41-core` that extend ALPHA-register manipulation
beyond v1.0's append/clear primitives: `ARCL nn` (append register-N's
formatted value to ALPHA), `ASTO nn` (pack first 6 ALPHA chars into a
register), `ATOX` (first ALPHA char → ASCII code in X), `XTOA` (X mod 256 →
ASCII char appended to ALPHA), `AROT` (rotate ALPHA by N chars, N from X),
`POSA` (single-char substring search). Direct-address forms only — IND
variants are layered on by Phase 24's `resolve_indirect()` helper.

Phase 23 introduces ONE new field on `CalcState`: `text_regs: BTreeMap<u8, String>`
(the sidecar map that carries packed-text shadows of the numeric `regs`).
This sidecar enables `ASTO`/`ARCL` faithfully without a typed-stack
refactor.

**In scope:** new `Op` variants in `ops/mod.rs` (`Arcl(u8)`, `Asto(u8)`,
`Atox`, `Xtoa`, `Arot`, `Posa`), dispatch arms, `execute_op` arms, both
`prgm_display.rs` copies (`hp41-cli` + `hp41-gui`), the new `text_regs`
field on `CalcState`, `op_arcl`/`op_asto`/`op_atox`/`op_xtoa`/`op_arot`/
`op_posa` functions in a new `ops/alpha_ops.rs` (or expanded `ops/alpha.rs`
— planner picks), sidecar-clearing hooks in `op_sto`, `op_sto_arith`,
`op_sto_arith_stack`, `op_clreg`, in-module unit tests, and an integration
test per success-criterion.

**Out of scope (Phase 24):** the shared `resolve_indirect()` helper and the
IND variants of `ARCL`, `ASTO`, `STO`, `RCL`, etc. Phase 23 ships only
direct-address forms.

**Out of scope (Phase 25):** keyboard wiring in `hp41-cli/src/keys.rs`,
`KEY_REF_TABLE` entries, new `PendingInput` modal variants for
`ArclPrompt`/`AstoPrompt`, `help_data.rs` updates, `pending_prompt()`
arms.

**Out of scope (Phase 26):** `key_map.rs::resolve` entries for `arcl`/`asto`/
`atox`/`xtoa`/`arot`/`posa`, `KEY_DEFS` bindings in `Keyboard.tsx`, modal
routing for `arcl_prompt`/`asto_prompt`.

**Out of scope (Phase 27):** flag-semantics proptest extensions, the 500-case
numerical-accuracy suite (none of these ops are math-precision-sensitive,
but a packed-text round-trip property test is in-scope for Phase 23 itself).

**Out of scope (v3.x deferred — explicit divergence):** multi-char POSA
needle path. Real HP-41 multi-char POSA reads the X register's raw 56 bits
as packed ASCII. Our `HpNum = rust_decimal::Decimal` model cannot preserve
arbitrary 56 raw bits the way NUT format can, so the multi-char path is
structurally impossible without a typed-stack refactor (Free42-style
`stack.x_text: Option<String>` shadow channel). Phase 23 ships single-char
POSA only; multi-char belongs in a future v3.x phase that introduces the
typed-stack channel (see `<deferred>` for the planned design sketch).

</domain>

<decisions>
## Implementation Decisions

### Packed-text register encoding (foundational for ARCL/ASTO)

- **D-23.1 — `text_regs: BTreeMap<u8, String>` sidecar:** new field on
  `CalcState` with `#[serde(default)]`. Maps register index (0..=99) to the
  packed-text shadow of that register. Coexists with `regs: Vec<HpNum>`.
  `BTreeMap` for deterministic JSON serialization order (matches
  D-22.17 `assignments` precedent, D-25/D-29 from Phase 5).

  ```rust
  // state.rs — slot next to assignments (line ~92) for grep affinity
  /// HP-41 packed-text register shadows: ASTO writes a 6-char string here,
  /// ARCL reads from here in preference to formatting the numeric `regs[reg]`.
  /// Numeric STO/op_sto_arith/op_sto_arith_stack/op_clreg CLEAR the matching
  /// entry to keep the two representations from drifting (D-23.4).
  /// `#[serde(default)]` keeps v1.x–v2.1 save files loadable (default = empty map).
  /// Phase 23 (FN-ALPHA-01, FN-ALPHA-02).
  #[serde(default)]
  pub text_regs: BTreeMap<u8, String>,
  ```

- **D-23.2 — ASTO semantics:** `Op::Asto(u8)`. Take first 6 chars of
  `state.alpha_reg` via `chars().take(6).collect::<String>()` (multibyte-safe).
  Insert into `state.text_regs.insert(reg, text)`. Zero the numeric slot:
  `if let Some(slot) = state.regs.get_mut(reg as usize) { *slot = HpNum::zero(); }`
  (out-of-range reg returns `HpError::InvalidOp` via the `.get_mut` bounds
  check). LiftEffect: Neutral. ALPHA register is NOT modified by ASTO.

- **D-23.3 — ARCL semantics:** `Op::Arcl(u8)`. Lookup order:
  1. If `state.text_regs.get(&reg).is_some()` → append that text to
     `alpha_reg`, capping at 24 chars (silent discard for overflow,
     matches `op_alpha_append` precedent).
  2. Else → `state.regs.get(reg as usize).ok_or(InvalidOp)?` (numeric),
     format via `format_hpnum(&r, &state.display_mode)`, append (24-char cap).
  3. Out-of-range reg → `HpError::InvalidOp`.

  LiftEffect: Neutral. ARCL respects current display mode (FIX/SCI/ENG) —
  switching mode between two `ARCL 05` calls produces different appended
  text (matches SC#1).

- **D-23.4 — Sidecar-clearing invariant (every numeric write):** to prevent
  the two representations from drifting, every op that writes a numeric
  value into `regs[reg]` MUST clear `text_regs.remove(&reg)`. This is a
  Wave-0 audit task, analogous to Phase 22's D-22.11.1 regs-bounds audit.
  Touch points:
  - `hp41-core/src/ops/registers.rs::op_sto(state, r)`
  - `hp41-core/src/ops/registers.rs::op_sto_arith(state, reg, kind)`
  - `hp41-core/src/ops/registers.rs::op_sto_arith_stack` (the stack-register
    flavor doesn't touch `regs[]` — but the numbered-register flavor does;
    audit both)
  - `hp41-core/src/ops/registers.rs::op_clreg(state)` — clears all `regs`
    AND must clear all `text_regs` entries (or `state.text_regs.clear()`).

  No clearing needed in: `op_rcl` (read-only), `Op::Cla`/`Op::AlphaClear`
  (modifies ALPHA, not regs), `Op::Clst` (stack only).

- **D-23.5 — RCL on a text-shadowed register:** unchanged from current
  behavior. `op_rcl(state, r)` pushes `state.regs[r]` onto X. After ASTO,
  the numeric slot was zeroed (D-23.2), so RCL of a text register pushes
  `HpNum::zero()` onto X. Documented divergence from real HP-41 (where RCL
  copies the raw 56-bit register contents verbatim). The single-char POSA
  decision (D-23.7) means we don't need RCL fidelity for multi-char POSA
  in v2.2.

### POSA semantics (single-char path only)

- **D-23.6 — Multi-char POSA deferred:** real HP-41 POSA supports both
  single-char (X = ASCII int) AND multi-char (X holds packed text from
  prior RCL of an ASTO'd register). Our `HpNum = Decimal` cannot preserve
  the raw 56 bits required for the multi-char path. Phase 23 ships
  single-char POSA only; multi-char is deferred to a future v3.x phase
  that introduces a typed-stack channel. Documented in CLAUDE.md
  §"v2.2 ALPHA additions" and in this CONTEXT.md `<deferred>` section.

- **D-23.7 — POSA single-char implementation:** `Op::Posa`. Read X (do NOT
  pop), require integer in 0..=127, find first occurrence of that char in
  `alpha_reg`, return position as i32 (0-indexed) in X. Not-found returns
  `-1`. Out-of-range/non-integer X returns `HpError::InvalidOp`. The
  result REPLACES X (LiftEffect: Disable — drops the previous X content,
  matches "X = computed value" pattern).

  ```rust
  pub fn op_posa(state: &mut CalcState) -> Result<(), HpError> {
      let x = state.stack.x.clone();
      let i = x.trunc_int();
      if i != x { return Err(HpError::InvalidOp); }                  // non-integer
      let code_dec = i.inner();
      let code = code_dec.try_into().map_err(|_| HpError::InvalidOp)?; // i32 fit
      if !(0..=127).contains(&code) { return Err(HpError::InvalidOp); } // ASCII range
      let needle = (code as u8) as char;
      let pos = state.alpha_reg.chars()
          .position(|c| c == needle)
          .map(|p| p as i32)
          .unwrap_or(-1);
      state.stack.x = HpNum::from(pos);
      apply_lift_effect(state, LiftEffect::Disable);
      Ok(())
  }
  ```

  SC#5 reinterpreted: `ALPHA = "THE QUICK BROWN FOX"`, `X = 81` (ASCII
  `'Q'`) → `POSA` → `X = 4`. Multi-char "QUICK" search awaits v3.x.

### AROT (faithful HP-41CV: read X, trunc, modulo, preserve X)

- **D-23.8 — AROT signature & X preservation:** `Op::Arot` (no parameter —
  the ROADMAP cross-cutting constraint says "N from X register, not
  immediate operand"). Reads `state.stack.x`, truncates toward zero
  (`HpNum::trunc_int`), modulos by `alpha_reg.chars().count()` using
  `rem_euclid` (handles negative N correctly: AROT -1 of "HELLO" → "OHELL").
  X is NOT consumed — LiftEffect: Neutral (faithful HP-41CV behavior;
  matches Free42).

- **D-23.9 — AROT edge cases:**
  - Empty ALPHA: no-op, X preserved, Neutral lift.
  - |N| > len: handled by `rem_euclid` modulo (AROT 7 of "HELLO" → AROT 2).
  - Non-integer X: silently truncated toward zero (1.7 → 1, -1.7 → -1,
    0.5 → 0). This is faithful HP-41CV — we do NOT reject non-integer X
    here (in contrast to the FN-IND-02 stricter rejection in Phase 24's
    indirect resolver, where non-integer is a bug).

  ```rust
  pub fn op_arot(state: &mut CalcState) -> Result<(), HpError> {
      apply_lift_effect(state, LiftEffect::Neutral);
      let len = state.alpha_reg.chars().count();
      if len == 0 { return Ok(()); }                                 // empty ALPHA
      let n_dec = state.stack.x.trunc_int().inner();
      let n_i64: i64 = n_dec.try_into().map_err(|_| HpError::InvalidOp)?;
      let n = n_i64.rem_euclid(len as i64) as usize;                 // 0..len
      let chars: Vec<char> = state.alpha_reg.chars().collect();
      state.alpha_reg = chars[n..].iter().chain(chars[..n].iter()).collect();
      Ok(())
  }
  ```

### ATOX / XTOA (faithful: ATOX consumes one ALPHA char + lifts; XTOA preserves X)

- **D-23.10 — ATOX semantics:** `Op::Atox`. Reads first char of
  `alpha_reg`, deletes it from ALPHA, pushes its Unicode codepoint
  (capped at 255 via `.min(255)`) into X **with stack lift Enable**
  (X→Y, ASCII becomes new X). Empty ALPHA → X = 0 with Enable lift
  (the lift fires regardless — faithful to most HP-41 sources; matches
  "constant push" pattern of LIT).

  ```rust
  pub fn op_atox(state: &mut CalcState) -> Result<(), HpError> {
      let code: i32 = match state.alpha_reg.chars().next() {
          Some(c) => {
              let mut chars: Vec<char> = state.alpha_reg.chars().collect();
              chars.remove(0);
              state.alpha_reg = chars.into_iter().collect();
              u32::from(c).min(255) as i32                            // 8-bit cap
          }
          None => 0,
      };
      apply_lift_effect(state, LiftEffect::Enable);                    // BEFORE push
      state.stack.lift_to_x(HpNum::from(code));                        // T←Z, Z←Y, Y←X, X←code
      Ok(())
  }
  ```

  Note on lift-then-push ordering: existing pattern (see e.g. `op_pi` in
  Phase 20) is `apply_lift_effect` first, then assign new X. Planner
  confirms the exact Stack API used (`lift_to_x` vs direct field assign
  after lift) against the v2.2 Phase 20/21 precedent.

- **D-23.11 — XTOA semantics:** `Op::Xtoa`. Reads X (preserved), truncates
  to integer, mod 256. If result is 0..=127 → append as ASCII char.
  If 128..=255 → append `'?'` placeholder (HP-41 upper-ASCII glyphs Σ, λ,
  ⊢ are not in our String/UTF-8 model — documented divergence). 24-char
  ALPHA cap applies (silent discard, matches `op_alpha_append`).
  LiftEffect: Neutral.

  ```rust
  pub fn op_xtoa(state: &mut CalcState) -> Result<(), HpError> {
      let i_dec = state.stack.x.trunc_int().inner();
      let i_i64: i64 = i_dec.try_into().map_err(|_| HpError::InvalidOp)?;
      let code: u32 = (i_i64.rem_euclid(256)) as u32;
      let c: char = if code < 128 { code as u8 as char } else { '?' };
      if state.alpha_reg.chars().count() < 24 {
          state.alpha_reg.push(c);
      }
      apply_lift_effect(state, LiftEffect::Neutral);
      Ok(())
  }
  ```

### Cross-cutting (locked, not gray)

- **D-23.12 — 4-place Op-variant landing:** every new variant (`Arcl(u8)`,
  `Asto(u8)`, `Atox`, `Xtoa`, `Arot`, `Posa`) goes into `ops/mod.rs::Op`
  enum + `dispatch()` + `execute_op()` (in `ops/program.rs`) + BOTH
  `prgm_display.rs` copies (`hp41-cli` + `hp41-gui`). Mirrors D-22.21.
  Suggested `prgm_display` strings (planner's discretion):
  `"ARCL nn"`, `"ASTO nn"`, `"ATOX"`, `"XTOA"`, `"AROT"`, `"POSA"`.

- **D-23.13 — Save-file compat:** new `state.text_regs` field carries
  `#[serde(default)]`. New `Op` variants are additive (land at the END of
  the `Op` enum, preserving existing discriminant order). Older save files
  load with `text_regs == BTreeMap::new()`.

- **D-23.14 — Zero-panic policy:** `#![deny(clippy::unwrap_used)]` enforced.
  Particular attention to:
  - `state.alpha_reg` byte-slicing → use `chars()` only, never byte indices
    (multibyte safety) — ROADMAP-locked cross-cutting constraint.
  - `state.regs.get(reg as usize).ok_or(InvalidOp)?` for every reg read.
  - `state.regs.get_mut(reg as usize)` with `if let Some(slot) = ...`
    for every reg write (ASTO zeros via this pattern).
  - `Decimal::try_into::<i64>()` for X→i64 conversion in AROT/XTOA/POSA
    (rather than `.to_i64().unwrap_or(0)` which would silently swallow
    overflow).

- **D-23.15 — SC-4 invariant:** no `op_*` / `flush_entry_*` / `format_hpnum`
  added to `hp41-gui/src-tauri/`. Only `prgm_display.rs` exhaustive-match
  updates allowed there.

- **D-23.16 — LiftEffect summary:**
  - `Arcl(_)` / `Asto(_)`: Neutral (ALPHA-only ops, do not touch stack)
  - `Atox`: Enable (pushes ASCII code onto stack with lift)
  - `Xtoa`: Neutral (reads X but does not consume; ALPHA-side append)
  - `Arot`: Neutral (reads X but does not consume; ALPHA-side rotate)
  - `Posa`: Disable (replaces X with computed position — no lift after)

- **D-23.17 — `Op::Asto` is ALPHA-aware; `Op::Asn` and `Op::AlphaAppend`
  are distinct:** Phase 22's `Op::Asn { name, key_code }` is a USER-mode
  key-assignment (D-22.18). Phase 2's `Op::AlphaAppend(char)` appends ONE
  char to ALPHA. Phase 23's `Op::Asto(u8)` packs 6 ALPHA chars into a
  numbered register via `text_regs`. Three distinct ops, three distinct
  jobs. Planner must not consolidate.

### Plan structure (Claude's discretion — guidance)

- **D-23.18 — Suggested split (2 plans):**
  - **23-01-arcl-asto-PLAN.md** — Wave-0: sidecar-clearing audit in
    `op_sto`/`op_sto_arith`/`op_sto_arith_stack`/`op_clreg`. Wave-1:
    `text_regs` field, `Op::Arcl(u8)`, `Op::Asto(u8)`, ARCL/ASTO logic,
    round-trip property test. Covers FN-ALPHA-01, FN-ALPHA-02.
  - **23-02-atox-xtoa-arot-posa-PLAN.md** — `Op::Atox`, `Op::Xtoa`,
    `Op::Arot`, `Op::Posa` (no shared state beyond `alpha_reg` and
    `stack.x`). Covers FN-ALPHA-03, FN-ALPHA-04, FN-ALPHA-05, FN-ALPHA-06.

  Alternative: 1 monolithic plan (smaller phase than 22). Planner picks
  based on file-overlap concerns. File-overlap on `ops/mod.rs`,
  `state.rs`, both `prgm_display.rs` copies forces sequential execution
  regardless (same constraint Phase 22 hit) — Wave numbering can still
  be assigned with explicit `depends_on` chains.

### Claude's Discretion

- Exact display strings in `prgm_display.rs` for the 6 new variants
  (`"ARCL nn"`, `"ASTO nn"`, `"ATOX"`, `"XTOA"`, `"AROT"`, `"POSA"`) —
  planner picks final text; match real HP-41 listing conventions.
- Test layout (inline `#[cfg(test)]` mods in `alpha.rs` / new
  `alpha_ops.rs` vs centralized `tests/phase23_alpha.rs` integration
  suite). Phase 22 precedent: split per plan with integration suite.
- Whether to extend `hp41-core/src/ops/alpha.rs` (existing 72 lines,
  contains the v1.0 toggle/append/clear primitives) or add a new
  `hp41-core/src/ops/alpha_ops.rs` for the 6 v2.2 ops. Recommendation:
  extend `alpha.rs` if total file size stays under ~400 lines; otherwise
  split.
- Whether the sidecar-clearing audit (D-23.4) lives as its own Wave-0
  commit before the main `Arcl`/`Asto` implementation, or is folded into
  the same commit. Recommendation: separate commit for git-blame clarity
  (Phase 22 D-22.11.1 precedent).
- Exact lift-then-push idiom in `op_atox` (whether to use a `Stack`
  helper method like `lift_to_x` or to manipulate Stack fields directly
  after `apply_lift_effect`). Phase 20's `op_pi` precedent is canonical
  here — planner reads it and mirrors.
- Round-trip property test scope: ASTO + ARCL round-trip for any
  String up to 6 chars over an ASCII subset. AROT + AROT-inverse
  round-trip. Planner decides.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level anchors

- `.planning/PROJECT.md` — build sequence (core → cli → docs → gui → tests),
  shipped milestones, architectural invariants. The v2.2 scope boundary
  (locked 2026-05-13) explicitly excludes module-Pacs.
- `.planning/REQUIREMENTS.md` §48–57 (FN-ALPHA-01..06) — the 6 requirements
  this phase delivers. FN-IND-01/02 are Phase 24's scope.
- `.planning/ROADMAP.md` §101–117 (Phase 23 details, success criteria,
  cross-cutting constraints). SC#5 wording explicitly leaves the POSA
  needle-source encoding open ("or however POSA encodes the search arg")
  — D-23.6 / D-23.7 closes that gray area with single-char-only POSA.
- `.planning/STATE.md` §Key Decisions — settled v1.0–v2.1 architecture
  decisions (BCD/f64, stack-lift, LiftEffect-per-op, zero-panic policy,
  4-place Op-variant rule, SC-4 invariant).
- `.planning/phases/22-program-control-and-memory-ops/22-CONTEXT.md` —
  immediate predecessor; D-22.21 (4-place landing), D-22.22 (save-file
  compat with `#[serde(default)]`), D-22.23 (zero-panic policy),
  D-22.24 (SC-4 invariant), D-22.17 (BTreeMap precedent for `assignments`
  — Phase 23's `text_regs` mirrors this).
- `CLAUDE.md` §"Settled Architecture Decisions" — full settled-decision
  catalog, including the v1.0 ALPHA-register-as-String decision, the
  `chars()` multibyte-safety rule, the `format_hpnum`/`format_alpha`
  formatters, and the v1.1 print-buffer drain pattern.

### Code references that constrain Phase 23

**ALPHA register and existing primitives:**

- `hp41-core/src/state.rs:57–58` — `alpha_reg: String` (max 24 chars),
  `alpha_mode: bool`. Slot the new `text_regs: BTreeMap<u8, String>`
  next to `assignments` (line ~92, Phase 22) for grep affinity.
- `hp41-core/src/ops/alpha.rs:22–30` — `op_alpha_append` template for the
  24-char silent-discard cap. ARCL and XTOA reuse this pattern.
- `hp41-core/src/ops/alpha.rs:34–38` — `op_alpha_clear` (the v1.0 ALPHA
  clear primitive that Phase 22's `Op::Cla` already wraps). Phase 23 does
  NOT add a new clear variant.
- `hp41-core/src/ops/alpha.rs:44–48` — `op_alpha_backspace` precedent for
  safe `String::pop()` on a possibly-empty register.
- `hp41-core/src/format.rs:18` — `format_hpnum(&HpNum, &DisplayMode)`.
  THE display formatter ARCL uses for numeric registers (D-23.3 step 2).
  Respects FIX/SCI/ENG mode automatically (SC#1 verified through this
  reuse).
- `hp41-core/src/format.rs:103–107` — `format_alpha(reg)` — 12-char display
  truncation. NOT used by Phase 23 directly; documented for awareness
  (it's the display-side formatter, not the ALPHA mutation layer).

**Register reads/writes and the sidecar-clearing audit (D-23.4):**

- `hp41-core/src/ops/registers.rs::op_sto` — every numeric STO must
  `state.text_regs.remove(&reg)` BEFORE writing the new numeric value.
  The existing function shape (read X, write `regs[reg]`) gets one
  inserted line.
- `hp41-core/src/ops/registers.rs::op_sto_arith` — same: clear sidecar
  before in-place numeric update.
- `hp41-core/src/ops/registers.rs::op_sto_arith_stack` — verify whether
  the stack-register flavor (Y/Z/T/L) touches `regs[]`; the numbered-
  register flavor does. Audit both code paths.
- `hp41-core/src/ops/registers.rs::op_clreg` — clears all `regs` to
  `HpNum::zero()`. Must also `state.text_regs.clear()` to fully reset.
- `hp41-core/src/ops/registers.rs::op_rcl` — READ-ONLY, no clearing
  needed. After ASTO, `regs[reg] == HpNum::zero()`, so RCL pushes 0 onto
  X. Documented divergence (D-23.5).

**Stack & lift effect:**

- `hp41-core/src/stack.rs::apply_lift_effect()` — every Phase 23 op calls
  this. LiftEffect map: ARCL/ASTO = Neutral, ATOX = Enable, XTOA = Neutral,
  AROT = Neutral, POSA = Disable (D-23.16).
- `hp41-core/src/stack.rs::Stack` — `x`/`y`/`z`/`t`/`lastx`/`lift_enabled`.
  ATOX needs the lift-then-push idiom (lift X→Y first, then assign new X).
  Phase 20's `op_pi` is the canonical precedent — read it and mirror.

**Numeric primitives:**

- `hp41-core/src/num.rs:224–226` — `HpNum::trunc_int()`. Used by
  AROT (D-23.8) and XTOA (D-23.11) and POSA (D-23.7) for integer
  extraction from X. Same helper Phase 22's `Op::GtoInd`/`Op::XeqInd`
  use (D-22.15 step 1).
- `hp41-core/src/num.rs:217–219` — `HpNum::inner()`. Pulls the underlying
  `Decimal` for `.try_into::<i64>()` conversions in AROT/XTOA/POSA.
- `hp41-core/src/num.rs:229–233` — `impl From<i32> for HpNum`. Used by
  ATOX (push ASCII code) and POSA (push position result).

**Both `prgm_display.rs` copies (the 4-place rule):**

- `hp41-cli/src/prgm_display.rs::op_display_name` — must add 6 new arms
  (ARCL/ASTO with `{:02}` width on the reg param; ATOX/XTOA/AROT/POSA
  as bare strings).
- `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` — same 6
  arms. Duplication is intentional (CLAUDE.md §SC-4 note).

**Error surface:**

- `hp41-core/src/error.rs::HpError` — Phase 23 uses existing variants ONLY:
  `InvalidOp` (non-integer X, out-of-range reg, out-of-range ASCII,
  conversion failures). No new variants needed.

### Prior-phase decisions that flow forward

- **Phase 2 (ALPHA mode):** `alpha_reg: String` max 24 chars. `chars()`-not-
  bytes for multibyte safety. Silent 24-char cap on append (op_alpha_append).
  Phase 23 inherits all three rules.
- **Phase 5 (USER mode):** `BTreeMap<char, String>` for `key_assignments` —
  the original BTreeMap-for-serializable-maps precedent that D-23.1's
  `text_regs: BTreeMap<u8, String>` follows.
- **Phase 11 (Print Emulation):** `state.print_buffer` drain pattern is
  unchanged by Phase 23. None of these ops write to print_buffer.
- **Phase 12 (Synthetic Programming):** `Op::Null`, `last_key_code`, hidden
  registers M/N/O. Phase 23's `Op::Atox`/`Op::Xtoa` interact with X (not
  M/N/O) — keep distinct.
- **Phase 20 (Core Math):** `Op::Pi`'s lift-then-push idiom — canonical
  precedent for `Op::Atox`'s lift-Enable behavior. Planner reads
  `hp41-core/src/ops/math.rs::op_pi` and mirrors.
- **Phase 21 (Flags, Display Control, Sound):** the `display_override`,
  `event_buffer` `#[serde(default, skip)]` pattern — Phase 23's
  `text_regs` is `#[serde(default)]` (persistent, NOT skip — text
  registers DO get saved/loaded).
- **Phase 22 (Program Control & Memory Ops):** D-22.17 BTreeMap precedent;
  D-22.21 4-place landing; D-22.22 save-file compat; D-22.23 zero-panic;
  D-22.24 SC-4 invariant; D-22.11.1 regs-bounds audit pattern (Phase 23's
  D-23.4 sidecar-clearing audit mirrors this).

### External reference (HP-41 hardware spec)

- HP-41C Owner's Manual §11 — ATOX/XTOA semantics ("deletes that
  character from Alpha", "appends ... to the Alpha register",
  "modulo 256", "does not affect the stack").
- HP-41C Programming Quick Reference — AROT (X is the count, X
  preserved), POSA (single-char from X = canonical; multi-char from
  packed-text register = synthetic/NUT-only).
- Free42 source notes — typed-stack approach to multi-char POSA
  (`stack.x_text` shadow channel). Cited as the design template if a
  future v3.x phase implements the typed-stack refactor.
- HP-41 NUT register format — 56-bit BCD; ALPHA chars stored as 6×7-bit
  ASCII (42 bits used). NOT directly emulated by Phase 23; documented
  as the reason multi-char POSA is deferred (D-23.6).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`op_alpha_append` pattern** (`alpha.rs:22–30`) — exact template for the
  24-char silent-discard cap. ARCL and XTOA reuse this pattern. Both use
  `chars().count() < 24` (multibyte-safe).
- **`format_hpnum(&hp, &display_mode)`** (`format.rs:18`) — ARCL's numeric
  formatter. Already respects FIX/SCI/ENG and the mantissa-carry-overflow
  fallback to SCI 9. Zero new formatter logic needed.
- **`HpNum::trunc_int()`** (`num.rs:224`) — used by AROT (D-23.8), XTOA
  (D-23.11), POSA (D-23.7) for X→integer extraction. Same helper Phase 20
  `Op::Fact` and Phase 22 `Op::GtoInd`/`XeqInd` use.
- **`HpNum::inner()`** (`num.rs:217`) — pulls the `Decimal` for
  `.try_into::<i64>()` chain in AROT/XTOA/POSA.
- **`impl From<i32> for HpNum`** (`num.rs:229`) — ATOX pushes the ASCII
  code via `HpNum::from(code)`. POSA pushes the position result the same
  way.
- **`apply_lift_effect(state, LiftEffect::X)`** (`stack.rs`) — every new
  Phase 23 op calls this. LiftEffect varies per op (D-23.16).
- **`op_pi` lift-then-push idiom** (`hp41-core/src/ops/math.rs`, Phase 20)
  — canonical precedent for `Op::Atox`'s lift-Enable behavior. Planner
  reads and mirrors.

### Established Patterns

- **`chars()`-not-bytes ALPHA handling** (Phase 2): every read/write to
  `alpha_reg` must use `chars().count()`, `chars().take()`,
  `chars().next()`, or `chars().collect::<Vec<char>>()`. NEVER byte
  indices. ROADMAP-locked cross-cutting constraint. Particularly
  important for AROT (rotation), POSA (search), ATOX (first-char
  removal).
- **`BTreeMap` for serializable maps** (Phase 5, Phase 22, Phase 23):
  deterministic JSON order. `text_regs` follows this rule.
- **`#[serde(default)]` for new persistent fields** (Phase 12 reg_m/n/o,
  Phase 21 flags, Phase 22 assignments, Phase 23 text_regs). v1.x save
  files load with default values.
- **Sidecar-clearing on numeric writes** (D-23.4, new Phase 23 pattern):
  every op that writes to `regs[reg]` clears `text_regs.remove(&reg)`.
  Analogous to Phase 22's `regs.get(_).ok_or(InvalidOp)?` bounds audit
  (D-22.11.1) — a one-line addition in 4 functions.
- **`apply_lift_effect` placement**: BEFORE the stack mutation for
  lift-Enable ops (ATOX), AFTER for lift-Neutral / lift-Disable ops.
  Mirrors Phase 20 `op_pi` (Enable before push) vs Phase 22 `op_pse`
  (Neutral after writes).

### Integration Points

**ops/mod.rs::Op enum** — add 6 new variants AT THE END (preserve existing
discriminant order, D-22.22 precedent):

```rust
Arcl(u8),     // append register-N's formatted value to ALPHA
Asto(u8),     // pack first 6 ALPHA chars into register-N via text_regs sidecar
Atox,         // first ALPHA char → ASCII code in X (with Enable lift)
Xtoa,         // X mod 256 → ASCII char appended to ALPHA (X preserved)
Arot,         // rotate ALPHA by X chars (X preserved, mod len)
Posa,         // single-char POSA: X = ASCII int → position in X (lift Disable)
```

**ops/mod.rs::dispatch()** — add 6 match arms. All 6 execute fine in both
interactive and program-execution contexts (none are programming-flow
ops), so NO catch-all entries needed in `execute_op`'s programming-ops
match — direct arms only.

**state.rs::CalcState** — add ONE new field, `text_regs: BTreeMap<u8, String>`,
with `#[serde(default)]`. Slot adjacent to `assignments` (Phase 22, line ~92).

**hp41-core/src/ops/registers.rs** — sidecar-clearing audit (D-23.4):
- `op_sto(state, r)`: add `state.text_regs.remove(&r);` BEFORE the
  numeric write.
- `op_sto_arith(state, reg, kind)`: same.
- `op_sto_arith_stack`: audit; the numbered-register flavor needs it.
- `op_clreg(state)`: add `state.text_regs.clear();` alongside the regs
  reset.

**hp41-cli/src/prgm_display.rs::op_display_name** — add 6 arms.
**hp41-gui/src-tauri/src/prgm_display.rs::op_display_name** — same 6 arms.

### Concrete signature sketches (planner-consumed)

```rust
// hp41-core/src/ops/alpha.rs (extend) or new ops/alpha_ops.rs

pub fn op_arcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let text: String = if let Some(t) = state.text_regs.get(&reg) {
        t.clone()
    } else {
        let r = state.regs.get(reg as usize).ok_or(HpError::InvalidOp)?;
        crate::format::format_hpnum(r, &state.display_mode)
    };
    for c in text.chars() {
        if state.alpha_reg.chars().count() >= 24 { break; }
        state.alpha_reg.push(c);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_asto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    let text: String = state.alpha_reg.chars().take(6).collect();
    if (reg as usize) >= state.regs.len() {
        return Err(HpError::InvalidOp);                       // bounds check
    }
    state.text_regs.insert(reg, text);
    if let Some(slot) = state.regs.get_mut(reg as usize) {
        *slot = HpNum::zero();
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_atox(state: &mut CalcState) -> Result<(), HpError> {
    let code: i32 = match state.alpha_reg.chars().next() {
        Some(c) => {
            let mut chars: Vec<char> = state.alpha_reg.chars().collect();
            chars.remove(0);
            state.alpha_reg = chars.into_iter().collect();
            u32::from(c).min(255) as i32
        }
        None => 0,
    };
    apply_lift_effect(state, LiftEffect::Enable);             // BEFORE push
    // Lift-then-assign (mirrors op_pi in math.rs):
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = state.stack.x.clone();
    state.stack.x = HpNum::from(code);
    Ok(())
}

pub fn op_xtoa(state: &mut CalcState) -> Result<(), HpError> {
    let i_dec = state.stack.x.trunc_int().inner();
    let i_i64: i64 = i_dec.try_into().map_err(|_| HpError::InvalidOp)?;
    let code: u32 = (i_i64.rem_euclid(256)) as u32;
    let c: char = if code < 128 { code as u8 as char } else { '?' };
    if state.alpha_reg.chars().count() < 24 {
        state.alpha_reg.push(c);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_arot(state: &mut CalcState) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    let len = state.alpha_reg.chars().count();
    if len == 0 { return Ok(()); }
    let n_dec = state.stack.x.trunc_int().inner();
    let n_i64: i64 = n_dec.try_into().map_err(|_| HpError::InvalidOp)?;
    let n = n_i64.rem_euclid(len as i64) as usize;
    let chars: Vec<char> = state.alpha_reg.chars().collect();
    state.alpha_reg = chars[n..].iter().chain(chars[..n].iter()).collect();
    Ok(())
}

pub fn op_posa(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    let i = x.trunc_int();
    if i != x { return Err(HpError::InvalidOp); }             // non-integer
    let code_i64: i64 = i.inner().try_into().map_err(|_| HpError::InvalidOp)?;
    if !(0..=127).contains(&code_i64) {
        return Err(HpError::InvalidOp);                       // ASCII range
    }
    let needle = (code_i64 as u8) as char;
    let pos: i32 = state.alpha_reg.chars()
        .position(|c| c == needle)
        .map(|p| p as i32)
        .unwrap_or(-1);
    state.stack.x = HpNum::from(pos);
    apply_lift_effect(state, LiftEffect::Disable);            // replace X
    Ok(())
}
```

</code_context>

<specifics>
## Specific Ideas

- **The sidecar-clearing rule (D-23.4) is the single most important
  invariant of Phase 23.** Without it, the two representations
  (`regs[reg]` numeric and `text_regs[reg]` text) silently drift, and
  ARCL outputs whichever was written first. Wave-0 audit BEFORE the
  Wave-1 ARCL/ASTO implementation lands. Same Phase 22 D-22.11.1 pattern.

- **`op_clreg` clears BOTH `regs` and `text_regs`.** Anything less leaves
  ghost text shadows after a `CLREG` (Phase 8 'g' key). Test this
  explicitly in 23-01.

- **ARCL on a register that was BOTH STO'd and ASTO'd:** D-23.4 guarantees
  this can't happen — every numeric STO clears the sidecar entry, every
  ASTO zeros the numeric slot. The invariant is "at most one representation
  is non-default at any time". Test: STO 5 of 3.14 → ASTO 5 → numeric is
  now 0 AND text_regs has the ALPHA prefix; ARCL 5 appends the text, not
  "0.0000".

- **ATOX with multibyte first char** (e.g., user typed Σ which is 'Σ' in
  Unicode = codepoint 0x03A3 = 931): `.min(255)` caps it to 255. So
  Σ → X = 255 (not 0x03A3). Documented as 8-bit cap. Real HP-41 has its
  own glyph at code 156 or similar for Σ; we don't try to reverse-map.
  POSA with X = 255 won't find Σ in our String (since our chars are
  Unicode, not HP-41 custom-set). Round-trip ATOX→XTOA preserves
  ASCII 0..=127 perfectly; codes 128..=255 round-trip to '?'.

- **AROT direction**: positive N rotates LEFT (front chars move to back).
  SC#4 verifies: `AROT 2` of `"HELLO"` → `"LLOHE"` (chars 'H','E'
  moved to back). Negative N rotates RIGHT: `AROT -1` of `"HELLO"`
  → `"OHELL"` ('O' from back moved to front). The `rem_euclid` in
  D-23.8 handles negative N correctly (returns 0..len always).

- **POSA returns -1 for not-found** (SC#5 wording). Other HP-41 sources
  return the haystack length instead. We pick -1 per SC#5 explicit
  wording. Document divergence note in `op_posa` body comment.

- **No new HpError variants** — entire phase reuses `InvalidOp`. Keeps
  the error surface stable (Phase 22 D-22 precedent).

- **Phase 24's `resolve_indirect()` will refactor ARCL/ASTO to accept
  IND form.** Phase 23 ships only direct-address forms (`Op::Arcl(u8)`,
  `Op::Asto(u8)`). Phase 24 adds `Op::ArclInd(u8)`, `Op::AstoInd(u8)`
  by extracting the inline integer-from-reg pattern. Phase 23 does NOT
  need to anticipate this refactor — the IND variants are additive.

</specifics>

<deferred>
## Deferred Ideas

- **Multi-char POSA via typed-stack channel** (deferred to v3.x). Real
  HP-41 supports multi-char POSA by reading the X register's raw 56 bits
  as packed ASCII (after RCL of an ASTO'd register copies those bits to
  X). Our `HpNum = rust_decimal::Decimal` cannot preserve arbitrary 56
  raw bits. The proper fix is a Free42-style typed-stack channel:

  ```rust
  // Sketch for the future v3.x typed-stack refactor:
  pub struct Stack {
      pub x: HpNum, pub y: HpNum, pub z: HpNum, pub t: HpNum, pub lastx: HpNum,
      pub lift_enabled: bool,
      /// Text-shadow channel for ALPHA-aware ops. Set by RCL of a register
      /// whose `text_regs[reg]` is Some. Read by POSA (multi-char path).
      /// Cleared by any numeric op that writes X.
      #[serde(default, skip)]
      pub x_text: Option<String>,
  }
  ```

  This is an Op-touching refactor (RCL, every numeric op on X, ASTO from
  stack-X synthetic op) — too invasive for Phase 23. Belongs to a v3.x
  phase that also tackles other typed-stack-only ops (e.g., text
  versions of CLA/CLST, ASTO-from-X synthetic).

- **HP-41 custom upper-ASCII char set (Σ, λ, ⊢, etc.)**: real HP-41
  codes 128..=255 map to special glyphs in the LCD font. Phase 23's
  XTOA maps these to `'?'` (D-23.11). A future v3.x phase could
  introduce a custom-glyph SVG font (matches v2.2 FN-POLISH-01 14-seg
  LCD font work) and a code↔glyph table. Documented divergence in
  CLAUDE.md §"v2.2 ALPHA additions" — `'?'` is a placeholder, not a
  permanent mapping.

- **`#[serde(default)]` migration for v1.x saves that loaded as v2.2**:
  no migration code needed — `text_regs` defaults to `BTreeMap::new()`,
  which is the correct "no text shadows" state. Existing v1.x files
  with `regs[12] = 5.0` (numeric) load unchanged; running ASTO 12 then
  creates the shadow. Round-trip-safe.

- **ASTO into the M/N/O hidden registers** (Phase 12). v2.2 scope only
  supports ASTO into numbered regs 0..=99. Synthetic ASTO M/N/O would
  need new variants `Op::AstoM`/`Op::AstoN`/`Op::AstoO` and matching
  sidecars `text_reg_m`/`text_reg_n`/`text_reg_o`. Backlog candidate
  for a synthetic-ALPHA phase.

- **`Op::Clp 0` or `Op::Clp ""` to clear ALL text_regs** — Phase 22's
  `Op::Cla` clears ALPHA; nothing currently clears `text_regs` en masse
  except `op_clreg` (D-23.4). Real HP-41 has no equivalent of "clear
  all packed-text shadows" since text and number share one 56-bit
  register. Probably not needed; backlog if real use case appears.

- **Phase 24 IND variants of ARCL/ASTO** (FN-IND-01 covers
  `ARCL IND nn` and `ASTO IND nn`). The Phase 24 `resolve_indirect()`
  helper will read the pointer-register, integer-check, and produce
  the resolved u8 address. ARCL/ASTO's IND variants then call
  `resolve_indirect(state, reg)?` to get the effective register, and
  delegate to the Phase 23 `op_arcl`/`op_asto` core. Phase 23 ships
  only the direct-address forms — Phase 24 layers IND on top.

</deferred>

---

*Phase: 23-alpha-operations*
*Context gathered: 2026-05-14*
</content>
</invoke>