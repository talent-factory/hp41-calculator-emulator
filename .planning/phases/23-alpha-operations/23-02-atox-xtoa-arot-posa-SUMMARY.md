---
phase: 23-alpha-operations
plan: 02
subsystem: alpha-operations
tags: [atox, xtoa, arot, posa, alpha, fn-alpha-03, fn-alpha-04, fn-alpha-05, fn-alpha-06]

dependency-graph:
  requires:
    - 23-01 ARCL/ASTO + text_regs sidecar (merged via commit 7bb0daf — provides the 4-place landing precedent in mod.rs / program.rs / both prgm_display.rs copies)
    - Phase 20 op_pi lift-then-push idiom (canonical precedent for ATOX's lift-Enable ordering)
    - Phase 22 D-22.21 4-place Op-variant landing rule
    - Phase 2 op_alpha_append's 24-char silent-discard cap idiom (XTOA reuses)
    - HpNum::trunc_int / HpNum::inner / impl From<i32> for HpNum (num.rs reuse targets)
    - apply_lift_effect + LiftEffect (stack.rs reuse targets)
  provides:
    - "Op::Atox — first ALPHA char → ASCII code in X (lift Enable, 8-bit cap)"
    - "Op::Xtoa — X mod 256 → ASCII char appended to ALPHA (X preserved; 128..=255 → '?')"
    - "Op::Arot — rotate ALPHA by X chars via rem_euclid (X preserved, silent-trunc non-integer)"
    - "Op::Posa — single-char POSA: 0..=127 codepoint → position in X (Disable lift; -1 not-found)"
    - "Complete 6-op Phase 23 ALPHA expansion (FN-ALPHA-01..06 all shipped in hp41-core)"
  affects:
    - "Phase 24 will layer Op::PosaInd / multi-char POSA refactor (typed-stack x_text channel) on top — DEFERRED to v3.x per D-23.6"
    - "Phase 25/26 will wire ATOX/XTOA/AROT/POSA into the TUI / GUI keyboard layers (out of scope for Phase 23)"

tech-stack:
  added:
    - "No new external dependencies"
  patterns:
    - "rem_euclid (NOT %) for negative-N rotation + 256 mod (D-23.8 / D-23.11)"
    - "Direct-assign T←Z, Z←Y, Y←X, X←code after apply_lift_effect(Enable) — mirrors op_pi lift-then-push (D-23.10)"
    - "chars().position(|c| c == needle) for multibyte-safe POSA (D-23.7)"
    - "i64.try_into().map_err(|_| InvalidOp) for Decimal → integer conversion (D-23.14 zero-panic)"
    - "AROT silent-trunc (faithful HP-41CV) vs POSA strict-reject — intentional divergence pinned by tests #11 and #13"

key-files:
  created:
    - hp41-core/tests/phase23_atox_xtoa_arot_posa.rs
  modified:
    - hp41-core/src/ops/alpha.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs

decisions:
  - "Chose direct-assign of state.stack.t/z/y/x in op_atox after apply_lift_effect(Enable), rather than going through the lift_to_x / enter_number helper. Direct-assign mirrors the explicit T←Z chain in the CONTEXT.md D-23.10 sketch and matches op_pi's pattern more transparently. Documented the choice in the op_atox doc comment."
  - "Combined the 4 new Op variants + their 4-place landing + the 4 functions + 15 inline unit tests into a single feat commit (matches plan 23-01's pattern). Splitting the Op enum from the function bodies would have left the build red between commits, because exhaustive matches in both prgm_display.rs copies fail to compile until the function arms match. Per the plan's task-1 wording the inline unit tests fit naturally into the same commit."
  - "Inline alpha.rs unit tests grew to 15 (one more than the 14 the plan suggested as 'illustrative'). Extra coverage was natural — e.g., test_arot_x_preserved_neutral_lift pins the Neutral lift contract explicitly even though the SC-level forward/reverse rotation tests already imply it, because a future refactor to LiftEffect::Disable would silently regress AROT semantics without a dedicated assertion."

metrics:
  duration: "~10 min (executor agent, single wave)"
  completed: 2026-05-14T14:30Z
  commits: 2
  tasks_completed: 2
  files_modified: 5
  files_created: 1
  loc_added: 593      # 383 (Task 1 — 4 Op variants + 4 functions + 15 inline tests + 4-place landing) + 210 (Task 2 — 13 integration tests)
  loc_removed: 0
  new_tests:
    inline_alpha: 15
    integration: 13
    total: 28
---

# Phase 23 Plan 02: ATOX / XTOA / AROT / POSA — Summary

**One-liner:** Landed the final 4 of 6 Phase 23 ALPHA ops in `hp41-core` — `Op::Atox` (Enable lift, 8-bit cap), `Op::Xtoa` (Neutral, '?' for upper-ASCII), `Op::Arot` (Neutral, `rem_euclid`-based rotation, silent-truncate non-integer X), and `Op::Posa` (Disable, strict integer + ASCII-range reject, -1 not-found) — completing the FN-ALPHA-01..06 set without introducing any new persistent `CalcState` field.

## What Shipped

### `Op::Atox` — FN-ALPHA-03 (D-23.10)

Pops the first ALPHA char and pushes its Unicode codepoint into X. Multibyte first chars are 8-bit-capped: `u32::from(c).min(255)` (e.g., Σ U+03A3 = decimal 931 → 255 — documented divergence from HP-41 hardware glyphs). Empty ALPHA pushes 0. Stack lift is `LiftEffect::Enable`, applied BEFORE the direct-assign chain `T←Z, Z←Y, Y←X, X←HpNum::from(code)` — mirrors `op_pi`'s lift-then-push ordering precedent in `math.rs:297`.

### `Op::Xtoa` — FN-ALPHA-04 (D-23.11)

Appends `(X mod 256) as char` to ALPHA. Codes 0..=127 append as ASCII; 128..=255 append as `'?'` placeholder (HP-41 upper-ASCII glyphs Σ/λ/⊢ are not in our String/UTF-8 model — documented divergence). 24-char ALPHA cap silently discards the append on overflow (Phase 2 `op_alpha_append` precedent). X is **not** consumed (`LiftEffect::Neutral`). Decimal-overflow into i64 is rejected as `HpError::InvalidOp` rather than silently swallowed (D-23.14).

### `Op::Arot` — FN-ALPHA-05 (D-23.8 / D-23.9)

Rotates ALPHA by X chars via `chars().collect::<Vec<char>>()` + `rem_euclid`-based rotation index (NOT `%` — `rem_euclid(-1, 5) = 4` is required for the negative-N path: `AROT -1` of `"HELLO"` → `"OHELL"`). Positive N is left rotation (`AROT 2` of `"HELLO"` → `"LLOHE"`); negative N is right rotation. Empty ALPHA is a no-op. `|N| > len` normalises via `rem_euclid(len)`. Non-integer X is **silently truncated toward zero** (faithful HP-41CV) — STRICTER than POSA's reject path; see "Intentional Divergence" below. `apply_lift_effect(Neutral)` fires EARLY so error/empty-ALPHA paths still settle the lift state. X is **not** consumed.

### `Op::Posa` — FN-ALPHA-06, single-char path (D-23.7)

Reads X (no consume yet), then enforces two gates: `i = x.trunc_int(); if i != x { Err(InvalidOp); }` (non-integer reject — stricter than AROT) and `(0..=127).contains(&code_i64)` (ASCII range gate). On success, finds the first matching `chars()` position; replaces X with `HpNum::from(pos as i32)` (0-indexed) or `HpNum::from(-1)` if not found (SC#5 explicit wording — other HP-41 sources return haystack length; we pick -1). `LiftEffect::Disable` after replacement. Multi-char POSA is deferred to v3.x per D-23.6 (requires a typed-stack `x_text: Option<String>` shadow channel that our `HpNum = rust_decimal::Decimal` model cannot preserve).

### Intentional Divergence: AROT silent-trunc vs POSA strict-reject

| Op   | Non-integer X | Rationale |
|------|---------------|-----------|
| AROT | Silent truncate toward zero | Faithful HP-41CV (D-23.9) — `2.7` rotates by 2 chars |
| POSA | `Err(InvalidOp)` | D-23.7 — position-lookup with fractional codepoint is semantically meaningless |

Both behaviors are intentional. Removing either would be a regression. Pinned mechanically by integration tests `posa_rejects_non_integer_x` (#11) and `arot_silently_truncates_non_integer_x` (#13).

### 4-place Op-variant landing (D-23.12)

Each of the 4 new variants lands in all four required places, preserving the existing discriminant order for save-file backward compat (`Op::Atox` / `Op::Xtoa` / `Op::Arot` / `Op::Posa` slot AT TAIL after `Op::Asto` from plan 23-01):

1. `hp41-core/src/ops/mod.rs::Op` enum declaration (lines 492 / 499 / 506 / 515)
2. `hp41-core/src/ops/mod.rs::dispatch()` match
3. `hp41-core/src/ops/program.rs::execute_op()` match
4. `hp41-cli/src/prgm_display.rs::op_display_name` AND `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name` (both copies — display strings `"ATOX"` / `"XTOA"` / `"AROT"` / `"POSA"`, bare with no parameter)

## Verification

- `just build` — workspace compiles green
- `just test-core` — full hp41-core suite passes (15 new inline `alpha.rs` unit tests + 13 new `phase23_atox_xtoa_arot_posa.rs` integration tests, plus all pre-existing tests still green)
- `just lint` — `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes with zero warnings
- SC-4 invariant: `grep -rnE "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` returns nothing — no calculator/math logic leaked into the GUI crate
- HpError variant count: still 9 (no new variants — D-23.14 reuses `InvalidOp` for all four error paths)
- Production-path `.unwrap()` count in `hp41-core/src/ops/alpha.rs` outside `#[cfg(test)]`: 0 (D-23.14 zero-panic policy)
- `rem_euclid` count in `alpha.rs`: 7 (AROT uses it twice for the rotation index, XTOA once for `mod 256`, plus 4 doc-comment references)
- `pub fn op_(atox|xtoa|arot|posa)` count: 4

## Success Criteria

| SC  | Status | Evidence |
|-----|--------|----------|
| SC#3 (FN-ALPHA-03 + FN-ALPHA-04): ATOX of `ALPHA="A"` → X=65; XTOA of X=66 with empty ALPHA → `ALPHA="B"`; round-trip exact for ASCII 0..=127 | ✅ | Integration tests #1 + #2 + #3 (`atox_pops_first_char_pushes_ascii_with_lift`, `xtoa_appends_ascii_char_preserves_x`, `atox_xtoa_round_trip_preserves_ascii_0_to_127` for [32, 65, 97, 126]); inline test `test_atox_pops_first_char_pushes_ascii_code_with_lift` |
| SC#4 (FN-ALPHA-05): AROT of `"HELLO"` X=2 → `"LLOHE"`; X=-1 → `"OHELL"` | ✅ | Integration tests #5 + #6 (`arot_left_rotation_two_of_hello_produces_lloghe`, `arot_right_rotation_negative_one_of_hello_produces_ohell`); inline tests `test_arot_positive_n_left_rotation` + `test_arot_negative_n_right_rotation` |
| SC#5 (FN-ALPHA-06, single-char): POSA of `"THE QUICK BROWN FOX"` X=81 → X=4; not-found → X=-1 | ✅ | Integration tests #9 + #10 (`posa_single_char_finds_position_4_for_q_in_the_quick`, `posa_not_found_returns_minus_one`); inline tests `test_posa_finds_single_char` + `test_posa_not_found_returns_minus_one` |
| D-23.12: 4-place Op landing for all 4 new variants | ✅ | `just build` (exhaustive matches green in all 4 places) + grep counts of 4 in each of mod.rs / program.rs / both prgm_display.rs copies |
| D-23.14: zero-panic — no production `.unwrap()` calls, `try_into().map_err(InvalidOp)` for Decimal→i64, `chars()`-not-bytes for all ALPHA mutation | ✅ | Production-path grep returns 0 unwraps; `just lint` (which runs `-D warnings` with `clippy::unwrap_used` denied at the hp41-core crate root) passes clean |
| Phase 23 completion: combined with plan 23-01, all 6 FN-ALPHA-* requirements ship in hp41-core | ✅ | FN-ALPHA-01 (`Arcl`) + FN-ALPHA-02 (`Asto`) from 23-01; FN-ALPHA-03 (`Atox`) + FN-ALPHA-04 (`Xtoa`) + FN-ALPHA-05 (`Arot`) + FN-ALPHA-06 (`Posa`) from this plan |

## Commits

| Hash      | Type | Description                                                  |
|-----------|------|--------------------------------------------------------------|
| `d12347a` | feat | Op::Atox / Op::Xtoa / Op::Arot / Op::Posa + 4-place landing + functions + 15 inline tests in `alpha.rs` |
| `3349ec1` | test | 13-test integration suite in `tests/phase23_atox_xtoa_arot_posa.rs` (SC#3/SC#4/SC#5 + AROT silent-trunc vs POSA strict-reject divergence pin) |

## Deviations from Plan

None on the auto-fix side. Two implementer-discretion choices (both pre-allowed by the plan / context):

### Implementer-Discretion Choices

- **Combined Task 1 implementation + inline tests into a single feat commit.** Mirrors the plan 23-01 pattern (where Tasks 1 and 2 were also combined for the same reason — the 4-place landing requires exhaustive matches in `prgm_display.rs` to compile, so splitting the Op enum from the function bodies would leave the build red between commits). Task 2 stayed as a separate test commit for git-blame clarity, following the same shape as plan 23-01's commit `915e22c`.
- **Extended `hp41-core/src/ops/alpha.rs` rather than splitting to a new `alpha_ops.rs`.** Final `alpha.rs` size is 638 lines — comfortably under the ~400-line "soft split threshold" suggestion from D-23.18 if you only count non-test lines (the file has a large `#[cfg(test)] mod tests` block at the tail). The implementation block is around 270 lines; the test block carries the rest. Splitting would have fragmented related ALPHA-side state mutation across two files for no readability gain.
- **Direct-assign over `lift_to_x` helper in `op_atox`.** The CONTEXT.md D-23.10 sketch shows the explicit `state.stack.t = state.stack.z.clone(); state.stack.z = ...` chain after `apply_lift_effect(Enable)`. We chose this path (over the alternate `state.stack.lift_enabled = true; enter_number(state, HpNum::from(code))` path mentioned in the plan's `<interfaces>` block) because it mirrors `op_pi` more transparently and avoids a second `apply_lift_effect` call inside `enter_number`. Doc-comment in `op_atox` records the choice and rationale.

## Known Stubs

None — Phase 23 plan 02 ships complete behaviour for ATOX, XTOA, AROT, and single-char POSA. The deferred items below are explicit scope boundaries, not stubs in this plan:

- **Multi-char POSA** — deferred to v3.x per D-23.6 (requires typed-stack `x_text: Option<String>` shadow channel; structurally impossible without a Stack refactor since `HpNum = rust_decimal::Decimal` cannot preserve 56 raw bits)
- **IND variants of ATOX/XTOA/AROT/POSA** — these ops do not take a register operand, so there is no `IND nn` form to defer. (Phase 24's `resolve_indirect()` will layer ARCL IND / ASTO IND on top of the 23-01 ARCL/ASTO core.)
- **TUI keyboard wiring** — Phase 25 scope (out of scope for Phase 23 per CONTEXT.md `<domain>` boundary)
- **GUI keyboard wiring** — Phase 26 scope (out of scope for Phase 23)

## Threat Flags

None — Phase 23 plan 02 introduces no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. The new `Op` variants flow through the existing `dispatch` / `execute_op` paths; all numeric input validation uses `try_into().map_err(InvalidOp)` (covers threat T-23-06 — crafted huge integers in X); all ALPHA mutation uses `chars()` (covers threat T-23-07 — crafted multibyte ALPHA); no DoS surface (T-23-08 accept disposition holds — each op is O(n) over n ≤ 24 chars). T-23-11 (missed exhaustive-match arm) is mitigated mechanically by `just build` — the workspace fails to compile if any of the 4 new variants is missed in either `prgm_display.rs` copy.

## Self-Check: PASSED

- File `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs` — FOUND (210 LOC, 13 `#[test]` functions, all passing under `just test-core`)
- File `hp41-core/src/ops/alpha.rs` — modified (4 new `pub fn`s + 15 new inline tests; final size 638 lines) — FOUND
- File `hp41-core/src/ops/mod.rs` — modified (4 new variants at enum tail + 4 dispatch arms) — FOUND
- File `hp41-core/src/ops/program.rs` — modified (4 new `execute_op` arms) — FOUND
- File `hp41-cli/src/prgm_display.rs` — modified (4 new bare-string display arms) — FOUND
- File `hp41-gui/src-tauri/src/prgm_display.rs` — modified (4 new bare-string display arms — mirrors CLI copy per SC-4 invariant) — FOUND
- Commit `d12347a` (feat) — FOUND on `worktree-agent-a260bd8929c5b96c1`
- Commit `3349ec1` (test) — FOUND on `worktree-agent-a260bd8929c5b96c1`
- All 4 acceptance-criteria grep targets return the expected counts:
  - `grep -nE "Atox,|Xtoa,|Arot,|Posa,?$" hp41-core/src/ops/mod.rs` → 4 matches at the enum tail
  - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-core/src/ops/mod.rs` → 4 dispatch arms
  - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-core/src/ops/program.rs` → 4 execute_op arms
  - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-cli/src/prgm_display.rs` → 4 display arms
  - `grep -cE "Op::(Atox|Xtoa|Arot|Posa)" hp41-gui/src-tauri/src/prgm_display.rs` → 4 display arms
  - Production-path `.unwrap()` count in `alpha.rs` outside `#[cfg(test)]` → 0
  - `rem_euclid` count in `alpha.rs` → 7 (exceeds the ≥2 acceptance criterion)

## TDD Gate Compliance

Plan type is `execute` (not `tdd`), so the plan-level RED→GREEN→REFACTOR commit sequence does not apply. Task 1's `tdd="true"` annotation indicates task-level test-first intent; the combined feat commit (`d12347a`) lands the implementation alongside 15 inline unit tests. The plan explicitly noted (in its `<action>` block and via the plan 23-01 precedent in this phase's SUMMARY) that combining implementation + inline tests in a single commit is acceptable for `execute`-type plans, because the 4-place landing requires the Op enum + dispatch arms + both prgm_display.rs copies to compile together — splitting the test commit off (RED) would have left the build red between commits. The integration test commit (`3349ec1`) is a `test(...)` commit per the conventional commit style.
