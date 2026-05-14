---
phase: 23-alpha-operations
reviewed: 2026-05-14T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - hp41-cli/src/prgm_display.rs
  - hp41-core/src/ops/alpha.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/registers.rs
  - hp41-core/src/state.rs
  - hp41-core/tests/phase23_arcl_asto.rs
  - hp41-core/tests/phase23_atox_xtoa_arot_posa.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
findings:
  critical: 0
  warning: 2
  info: 5
  total: 7
status: issues_found
---

# Phase 23: Code Review Report

**Reviewed:** 2026-05-14T00:00:00Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase 23 lands six HP-41CV ALPHA-register ops (`ARCL`, `ASTO`, `ATOX`, `XTOA`,
`AROT`, `POSA`) plus the foundational `text_regs: BTreeMap<u8, String>` sidecar
on `CalcState`. The submission cleanly satisfies the documented invariants the
review was asked to verify:

- **D-23.4 sidecar-clearing audit:** `op_sto` / `op_sto_arith` / `op_clreg` all
  remove (or `.clear()`) `text_regs` entries; `op_sto_arith_stack` correctly
  documents that it does NOT touch the sidecar (it targets Y/Z/T/LastX, not
  numbered regs). `op_sto_arith` orders the sidecar removal AFTER the
  `checked_*` computation, preserving atomicity on div-by-zero.
- **D-23.7 (POSA strict):** non-integer X returns `HpError::InvalidOp` via the
  `i != x` check; out-of-range integer X is rejected by the `0..=127` gate.
- **D-23.9 (AROT silently truncates):** `trunc_int()` is called without an
  equality re-check; the test `arot_silently_truncates_non_integer_x` mechanically
  pins the divergence from POSA.
- **D-23.12 (4-place landing):** All six new `Op` variants appear in exactly
  the four required places: `Op` enum + `dispatch()` in `ops/mod.rs`,
  `execute_op()` in `ops/program.rs`, `hp41-cli/src/prgm_display.rs`, and
  `hp41-gui/src-tauri/src/prgm_display.rs`. Grep confirms no orphan references.
- **D-23.14 (zero-panic):** No `.unwrap()` in production code; all 43 occurrences
  in `alpha.rs` are inside `#[cfg(test)] #[allow(clippy::unwrap_used)]` blocks.
  Bounds-checked `.expect("bounds-checked above")` in `op_arcl` is justified by
  the leading guard. No `println!` / `eprintln!` in `hp41-core`.
- **`#[serde(default)]` on `text_regs`:** present (state.rs:103); integration
  test #7 (`serde_default_loads_v21_save_file_without_text_regs_field`) confirms
  v2.0/v2.1 save files load with an empty map.
- **Multibyte safety:** `chars()` is used everywhere ALPHA strings are mutated
  (`op_arcl` text iter, `op_asto` `chars().take(6)`, `op_atox` `chars().collect`
  + `chars.remove(0)`, `op_arot` `chars()` Vec rebuild, `op_posa` `chars().position`,
  `op_xtoa` cap check via `chars().count()`). No byte-slicing of UTF-8.
- **No new HpError variants:** `InvalidOp` is reused for every Phase 23 error
  path (D-23.14 / D-23.16).

All 21 Phase 23 integration tests pass; `cargo clippy --all-targets -- -D warnings`
is clean on `hp41-core`.

Two real defects surfaced — both centred on the **interaction between
`text_regs` and `op_size`**, a touchpoint that D-23.4's CONTEXT.md audit
inventory missed. Plus five info-level items (misleading documentation
comments and minor test-coverage gaps).

## Critical Issues

_None._

## Warnings

### WR-01: `op_size` does NOT clear `text_regs` entries beyond the new SIZE — D-23.4 invariant leaks across SIZE shrink/grow cycles

**File:** `hp41-core/src/ops/program.rs:262-270`

**Issue:**
`op_size` resizes `state.regs` but leaves `state.text_regs` entirely untouched.
This breaks the spirit of D-23.4 ("every shrinking operation on the numeric
representation must keep the text shadow from drifting") and creates a
ghost-resurrection bug across SIZE cycles:

```text
1. SIZE 100;  ASTO 60 with ALPHA="GHOST"  → text_regs[60] = "GHOST", regs[60] = 0
2. SIZE 50                                → regs truncated to 50 slots;
                                            text_regs[60] is left behind
   (op_arcl(60) now correctly rejects with InvalidOp via the W-2 leading
   bounds check — good — but the orphan stays in the map)
3. SIZE 100                                → regs grown to 100 slots, regs[60] = 0
4. ARCL 60                                 → text_regs[60] takes precedence
                                              → ALPHA gets "GHOST" appended
```

After step 4 the resurrected text shadow appears to the user even though a
shrink-then-grow conceptually destroys register 60's contents on a real
HP-41. The autosave round-trip persists the orphan indefinitely.

A second concern is the same orphan leaks across `serde` save/load — a
trimmed-then-restored SIZE leaves the `text_regs` keys ≥ new-len visible in
`~/.hp41/autosave.json`, raising long-term data-bloat and threat T-23-01
surface area.

CONTEXT.md D-23.4 enumerates `op_sto` / `op_sto_arith` / `op_sto_arith_stack`
/ `op_clreg` as audit touchpoints but does NOT list `op_size`. Phase 22 owned
`op_size` but Phase 23 introduces the `text_regs` interaction, so the audit
miss belongs here.

**Fix:**
Add a retain-on-shrink step in `op_size` after the resize:

```rust
pub fn op_size(state: &mut CalcState, nnn: u16) -> Result<(), HpError> {
    if nnn > 319 {
        return Err(HpError::InvalidOp);
    }
    let target = nnn.max(1) as usize; // OQ-2: SIZE 0 → silently clamp to 1
    state.regs.resize(target, crate::num::HpNum::zero());
    // Phase 23 D-23.4: drop text_regs entries that now point past end-of-regs
    // so a shrink-then-grow cycle cannot resurrect a stale text shadow.
    state.text_regs.retain(|&k, _| (k as usize) < target);
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}
```

Add a sentinel integration test (e.g. `size_shrink_then_grow_drops_text_regs`)
that mirrors the four-step sequence above and asserts ALPHA stays empty after
the final ARCL.

---

### WR-02: `op_atox` shifts the stack unconditionally — divergence from `op_pi`'s `enter_number` discipline, contrary to the doc-comment claim

**File:** `hp41-core/src/ops/alpha.rs:166-187`

**Issue:**
The op_atox doc-comment (line 151) advertises that it "mirrors `op_pi`'s
lift-then-push ordering precedent (math.rs ~line 297)". The actual implementations
diverge in an important way:

`op_pi` (math.rs:297-305):
```rust
state.stack.lift_enabled = true;       // force lift
enter_number(state, pi_value);         // shifts ONLY if lift_enabled — currently true
apply_lift_effect(state, LiftEffect::Enable);
```

`op_atox` (alpha.rs:181-185):
```rust
apply_lift_effect(state, LiftEffect::Enable);  // sets lift_enabled = true
state.stack.t = state.stack.z.clone();         // UNCONDITIONAL shift
state.stack.z = state.stack.y.clone();
state.stack.y = state.stack.x.clone();
state.stack.x = HpNum::from(code);
```

Both currently produce the same observable behaviour for any HP-41 trajectory
that reaches `op_atox`, because `apply_lift_effect(Enable)` always makes
`lift_enabled == true` before the shift. So this is **not a correctness bug
today**. The concern is robustness against a future refactor:

1. The single source of truth for stack-lift in `hp41-core` is
   `crate::stack::enter_number()`. Bypassing it skips the one place where
   the lift discipline is centrally enforced — a regression hazard the
   project's own CLAUDE.md flags as "the most commonly mis-implemented HP-41
   feature — always check."
2. If a future change to `enter_number` (e.g. adding a hook for an x_text
   shadow channel, a soft-overflow guard, or a telemetry counter) lands,
   `op_atox` silently bypasses it. The four other call sites that direct-
   assign stack fields (`op_pi`, `op_lastx`, `op_polar_to_rect`,
   `op_rect_to_polar`) do so to express semantics that `enter_number`
   cannot (lift-then-push of a constant, LASTX overwrite, dual stack-slot
   write). ATOX's shape is a vanilla "compute value, push to X" — exactly
   what `enter_number` is for.
3. The doc-comment misleads the reader: it says "mirrors `op_pi`" but the
   structural mirror would be:
   ```rust
   state.stack.lift_enabled = true;     // force lift
   enter_number(state, HpNum::from(code));
   apply_lift_effect(state, LiftEffect::Enable);
   ```

**Fix:**
Replace the direct stack-field assignments with the `enter_number` flow used
by `op_pi`:

```rust
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
    // Mirror op_pi: force lift, push via enter_number, then declare lift effect.
    state.stack.lift_enabled = true;
    crate::stack::enter_number(state, HpNum::from(code));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

Existing tests still pass (`test_atox_pops_first_char_pushes_ascii_code_with_lift`
and `test_atox_empty_alpha_pushes_zero_with_lift` both check the post-condition
that prior X moved to Y, which the `enter_number` flow preserves).

## Info

### IN-01: `op_arot` doc-comment claim that early `apply_lift_effect(Neutral)` "settles the lift state" is misleading

**File:** `hp41-core/src/ops/alpha.rs:222-239`

**Issue:**
The doc-comment (lines 223-225) says: "apply_lift_effect(state, LiftEffect::Neutral)
is called EARLY so that the early-return on empty-ALPHA / overflow-error still
settles the lift state". But `LiftEffect::Neutral` is by definition a **no-op**
on `state.stack.lift_enabled` (`stack.rs::apply_lift_effect` falls through with
`/* intentional no-op */`). Calling it early, late, or never produces an
identical lift_enabled trajectory.

The early call is harmless. The comment is misleading and will confuse future
maintainers into thinking the placement is load-bearing.

**Fix:**
Either drop the early call (purely cosmetic — `apply_lift_effect(Neutral)` is
literally `()`) or rewrite the comment to: "The trailing `apply_lift_effect(Neutral)`
is documentary — Neutral is a no-op on `lift_enabled`. Placement is purely
stylistic." If kept, move the call back to the end of the function so it
visually parallels every other Neutral op in the file.

---

### IN-02: `op_arot` and `op_xtoa` reject `i64`-overflow X values, but the doc-comments advertise "silent truncation"

**File:** `hp41-core/src/ops/alpha.rs:236-251` (AROT) and `207-217` (XTOA)

**Issue:**
The implementation of both ops uses `n_dec.try_into::<i64>().map_err(|_| HpError::InvalidOp)?`
on the truncated X. For an X like `1e30`, `trunc_int()` returns a huge integer
Decimal that overflows i64; the op returns `Err(HpError::InvalidOp)` rather
than performing the silent-truncation or X-mod-256 calculation the doc-comment
promises.

This is a sensible practical limit — no reasonable rotation count is > 2^63 —
but the doc-comments do not call it out. AROT's doc says "silent truncation
toward zero" (D-23.9); XTOA's says "X mod 256". Neither mentions an error
path.

The 24-char ALPHA cap makes the user-observable difference trivial (rotating
HELLO by 2^65 should be the same as rotating by N mod 5 — but Phase 23 chooses
to reject). No test exercises the overflow-X branch (no
`test_arot_rejects_overflow_x` / `test_xtoa_rejects_overflow_x`).

**Fix:**
Either:
1. Extend the doc-comments to enumerate the i64-overflow rejection path
   ("X values that overflow i64 are rejected with `HpError::InvalidOp`,
   documented divergence from real HP-41 which silently wraps") — preferred.
2. Or use `rem_euclid` on the Decimal directly to honour the silent-wrap
   promise. Pricier and unlikely to matter — option 1 is the right fix.

Add one integration test per op (`arot_rejects_i64_overflow_x` /
`xtoa_rejects_i64_overflow_x`) so the divergence is mechanically pinned.

---

### IN-03: `text_regs` round-trip serialization is not directly tested — only the v1.x-without-`text_regs` deserialize path is covered

**File:** `hp41-core/tests/phase23_arcl_asto.rs:215-244`

**Issue:**
Test #7 (`serde_default_loads_v21_save_file_without_text_regs_field`) verifies
the **backward**-compat path: a JSON payload missing `text_regs` deserializes
to an empty map. There is no test for the **forward** round-trip: serialize a
populated `text_regs` to JSON, deserialize, and assert the contents survived.

If a future change broke the `Serialize` derivation (e.g. accidentally
adding `#[serde(skip_serializing)]` instead of `#[serde(default)]` to
`text_regs`), it would not be caught — the v1.x test would still pass
(no field on the wire is the v1.x scenario) but ASTO output would silently
fail to persist across an autosave/load cycle.

**Fix:**
Add a small round-trip test:

```rust
#[test]
fn text_regs_survives_serde_round_trip() {
    let mut state = CalcState::new();
    state.text_regs.insert(3, "HELLO".to_string());
    state.text_regs.insert(99, "WORLD".to_string());
    let json = serde_json::to_string(&state).unwrap();
    let restored: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.text_regs.get(&3),  Some(&"HELLO".to_string()));
    assert_eq!(restored.text_regs.get(&99), Some(&"WORLD".to_string()));
}
```

---

### IN-04: No test pins `lift_enabled == false` after POSA or `lift_enabled == true` after ATOX

**File:** `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs`

**Issue:**
D-23.16 declares POSA as `LiftEffect::Disable` (replaces X, then disables
lift so a subsequent digit overwrites the position) and ATOX as
`LiftEffect::Enable`. The lift-effect summary is verified for AROT
(`test_arot_x_preserved_neutral_lift` in `alpha.rs`) but not for the
post-POSA-Disable or post-ATOX-Enable cases. A regression that flipped
either lift effect (e.g. POSA accidentally Neutral, ATOX accidentally
Neutral) would not be detected.

This matters most for POSA: with `lift_enabled == false`, the user types
a digit and it overwrites the position-result in X (HP-41 convention for
"this is a derived value, the next number entry replaces it"). A silent
flip to Neutral / Enable would produce a stack ghost on the next digit.

**Fix:**
Add two short tests:

```rust
#[test]
fn posa_disables_lift_after_result() {
    let mut state = CalcState::new();
    state.alpha_reg = "HELLO".to_string();
    state.stack.x = HpNum::from(72);                // 'H' → pos 0
    state.stack.lift_enabled = true;
    dispatch(&mut state, Op::Posa).unwrap();
    assert!(!state.stack.lift_enabled, "POSA must disable lift (D-23.16)");
}

#[test]
fn atox_enables_lift_after_push() {
    let mut state = CalcState::new();
    state.alpha_reg = "A".to_string();
    state.stack.lift_enabled = false;
    dispatch(&mut state, Op::Atox).unwrap();
    assert!(state.stack.lift_enabled, "ATOX must enable lift (D-23.16)");
}
```

---

### IN-05: Test name typo — `arot_left_rotation_two_of_hello_produces_lloghe` should read `lloghe → llohe`

**File:** `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs:107`

**Issue:**
Test name reads `arot_left_rotation_two_of_hello_produces_lloghe`. The
assertion correctly checks for `"LLOHE"` (5 chars); the function name has
six letters (`lloghe`) — likely a transcription typo. Harmless but jarring
when grepping for test names.

**Fix:**
Rename to `arot_left_rotation_two_of_hello_produces_llohe`.

---

_Reviewed: 2026-05-14T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
