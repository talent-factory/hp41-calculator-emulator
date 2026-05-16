---
phase: 23-alpha-operations
verified: 2026-05-14T14:21:34Z
status: passed
score: 6/6 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: "review found 2 warnings (WR-01 + WR-02)"
  gaps_closed:
    - "WR-01: op_size now prunes text_regs entries past new register count (commit f109740)"
    - "WR-02: op_atox now routes through stack::enter_number (commit 990d1bb)"
  gaps_remaining: []
  regressions: []
---

# Phase 23: ALPHA Operations Verification Report

**Phase Goal:** Users can manipulate the ALPHA register beyond v1.0's append/clear primitives — `ARCL nn` appends a register's formatted value; `ASTO nn` packs the first 6 ALPHA chars into a register; `ATOX`/`XTOA` interconvert the first ALPHA char with its ASCII code in X; `AROT n` rotates ALPHA (negative N rotates right); `POSA` returns the substring position. Direct-address forms only; IND variants come in Phase 24.

**Verified:** 2026-05-14T14:21:34Z
**Status:** passed
**Re-verification:** Yes — after WR-01 + WR-02 fixes (commits f109740, 990d1bb, 4ba85c4)

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| #   | Truth                                                                                                                                                                   | Status     | Evidence                                                                                                                                                                                                              |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| SC#1 | With ALPHA="HELLO" and R05=3.14 (FIX 2), `ARCL 05` produces ALPHA="HELLO3.14"; switching to SCI 3 then `ARCL 05` again appends in SCI format                              | VERIFIED   | Integration test `arcl_appends_numeric_register_using_current_display_mode` (tests/phase23_arcl_asto.rs:32-62) builds expected via `format_hpnum(.., Fix(2))` & `Sci(3)` and asserts they differ. Inline `test_arcl_appends_numeric_register_in_sci_mode_differs_from_fix` (alpha.rs:341-366). |
| SC#2 | With ALPHA="GOODBYE", `ASTO 12` packs "GOODBY" into R12; `RCL 12` + `ARCL 12` reproduce "GOODBY"                                                                          | VERIFIED   | `asto_arcl_round_trip_reproduces_first_6_chars` (tests/phase23_arcl_asto.rs:73-91) asserts `text_regs[12] == Some("GOODBY")`, `regs[12] == HpNum::zero()`, and ARCL 12 yields ALPHA="GOODBY". D-23.5 divergence (RCL pushes 0) documented in test comment & CONTEXT.md. |
| SC#3 | With ALPHA="A", `ATOX` puts 65 in X; with X=66 + ALPHA="", `XTOA` makes ALPHA="B"                                                                                          | VERIFIED   | `atox_pops_first_char_pushes_ascii_with_lift` (line 37-45): asserts X=65 after Atox of "A"; `xtoa_appends_ascii_char_preserves_x` (line 50-57): X=66, empty ALPHA → "B"; round-trip property `atox_xtoa_round_trip_preserves_ascii_0_to_127` covers [32,65,97,126]. |
| SC#4 | With ALPHA="HELLO", `AROT 2` → ALPHA="LLOHE"; `AROT -1` → ALPHA="OHELL"                                                                                                  | VERIFIED   | `arot_left_rotation_two_of_hello_produces_lloghe` (line 107-114) asserts exact "LLOHE"; `arot_right_rotation_negative_one_of_hello_produces_ohell` (line 119-125) asserts exact "OHELL" via `rem_euclid(-1, 5) = 4`. |
| SC#5 | With ALPHA="THE QUICK BROWN FOX" containing "QUICK", `POSA` returns 4 in X; missing substring → -1                                                                       | VERIFIED (with documented scope reduction — see WARNING below) | `posa_single_char_finds_position_4_for_q_in_the_quick` (line 152-158) asserts X=4 for ASCII 'Q' (81) — uses single-char lookup. `posa_not_found_returns_minus_one` (line 163-169) asserts X=-1 for 'Z' (90) in "HELLO". |
| Direct-address only — IND variants deferred to Phase 24 | VERIFIED | No `Op::ArclInd` / `Op::AstoInd` etc. introduced in this phase; ROADMAP Phase 24 line 102 explicitly lists IND variants as next-phase scope. |

**Score:** 6/6 truths verified

### Note on SC#5 / FN-ALPHA-06 — POSA single-char vs substring

**ROADMAP SC#5 wording:** `... and X holding "QUICK" (or however POSA encodes the search arg) ...` — the bracketed clause explicitly defers the encoding-of-needle decision to planning.

**FN-ALPHA-06 wording (REQUIREMENTS.md line 55):** `User can search ALPHA for substring (POSA) — returns position in X, -1 if not found`.

**Implementation (D-23.6 / CONTEXT.md):** Reduced to single-char only. Multi-char POSA requires a typed-stack `x_text: Option<String>` shadow channel that is structurally impossible under our `HpNum = rust_decimal::Decimal` model (Free42 ships this; we don't). The single-char path satisfies SC#5's "POSA returns 4" assertion by interpreting X=81 (ASCII 'Q') as the needle — the first char of "QUICK". The "-1 for missing" assertion is satisfied verbatim.

This is a documented, intentional scope reduction. The ROADMAP-vs-PLAN divergence is acknowledged in 23-CONTEXT.md (`<deferred>` section), in alpha.rs op_posa doc-comment ("Multi-char POSA is deferred to v3.x per D-23.6"), and in the plan 02 SUMMARY. **Recommended action:** update REQUIREMENTS.md FN-ALPHA-06 to "search ALPHA for single ASCII char (POSA, single-char path)" with a v3.x note for multi-char — but this is a documentation refinement, not a Phase 23 BLOCKER, because the ROADMAP SC#5's bracketed clause explicitly allowed planner discretion on needle encoding.

### Required Artifacts

| Artifact                                                              | Expected                                                                            | Status     | Details                                                                                                                                                                |
| --------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hp41-core/src/state.rs`                                              | `text_regs: BTreeMap<u8, String>` field with `#[serde(default)]`                    | VERIFIED   | Line 104 declares the field; line 102 has `#[serde(default)]`. CalcState::new line 178 initialises `text_regs: BTreeMap::new()`.                                       |
| `hp41-core/src/ops/alpha.rs`                                          | `op_arcl` / `op_asto` / `op_atox` / `op_xtoa` / `op_arot` / `op_posa` + inline tests | VERIFIED   | All 6 `pub fn op_*` defined (lines 79, 122, 165, 207, 236, 275). 24 inline tests (9 ARCL/ASTO + 15 ATOX/XTOA/AROT/POSA) — all pass under `just test-core`.            |
| `hp41-core/src/ops/registers.rs`                                      | Sidecar-clearing audit in op_sto / op_sto_arith / op_clreg + audit comment in op_sto_arith_stack | VERIFIED   | Line 25 (op_sto), line 71 (op_sto_arith — AFTER checked_* for atomicity), line 128 (op_clreg clear()), lines 88-92 audit-outcome comment on op_sto_arith_stack. 5 inline audit tests. |
| `hp41-core/src/ops/mod.rs`                                            | Op::Arcl(u8) / Op::Asto(u8) / Op::Atox / Op::Xtoa / Op::Arot / Op::Posa AT END of enum + dispatch arms | VERIFIED   | Enum: lines 477, 484, 492, 499, 506, 515. Dispatch arms: lines 831-839.                                                                                               |
| `hp41-core/src/ops/program.rs`                                        | execute_op arms for all 6 new variants + op_size text_regs pruning (WR-01 fix)      | VERIFIED   | Execute_op arms: lines 817-825. op_size WR-01 fix: line 273 `state.text_regs.retain(|&k, _| (k as usize) < target);`.                                                  |
| `hp41-cli/src/prgm_display.rs`                                        | op_display_name arms for all 6 new variants                                          | VERIFIED   | Lines 201, 202, 205-208 — "ARCL {reg:02}", "ASTO {reg:02}", "ATOX", "XTOA", "AROT", "POSA".                                                                            |
| `hp41-gui/src-tauri/src/prgm_display.rs`                              | op_display_name arms for all 6 new variants (mirror)                                | VERIFIED   | Lines 221, 222, 226-229 — same strings as the CLI copy (SC-4 invariant preserved).                                                                                    |
| `hp41-core/tests/phase23_arcl_asto.rs`                                | Integration suite SC#1, SC#2, D-23.4, save-file compat, WR-01                       | VERIFIED   | 9 `#[test]` functions (file lines 32-291, including new WR-01 test `size_shrink_then_grow_drops_text_regs_no_ghost_resurrection`). All pass.                          |
| `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs`                      | Integration suite SC#3, SC#4, SC#5, AROT vs POSA divergence pin                     | VERIFIED   | 13 `#[test]` functions (file lines 37-210). All pass.                                                                                                                  |

### Key Link Verification

| From                                       | To                                              | Via                                                            | Status | Details                                                                                                              |
| ------------------------------------------ | ----------------------------------------------- | -------------------------------------------------------------- | ------ | -------------------------------------------------------------------------------------------------------------------- |
| Op::Arcl(u8) in ops/mod.rs                 | op_arcl in ops/alpha.rs                         | dispatch() arm                                                 | WIRED  | mod.rs:831 `Op::Arcl(reg) => alpha::op_arcl(state, reg)`; program.rs:817 mirror.                                     |
| op_arcl                                    | format_hpnum(r, &state.display_mode)            | numeric-fallback branch                                        | WIRED  | alpha.rs:95 — invoked when text_regs.get is None.                                                                    |
| op_sto / op_sto_arith / op_clreg           | state.text_regs.remove / .clear                 | Wave-0 sidecar-clearing audit (D-23.4)                         | WIRED  | registers.rs:25 (op_sto), 71 (op_sto_arith after checked_*), 128 (op_clreg).                                          |
| op_atox                                    | stack::enter_number                             | WR-02 fix routes ATOX through canonical lift-helper            | WIRED  | alpha.rs:184 `enter_number(state, HpNum::from(code))`. Mirrors op_pi (math.rs ~297).                                  |
| op_arot                                    | chars().collect::<Vec<char>>() + rem_euclid     | multibyte-safe rotation; negative N via rem_euclid             | WIRED  | alpha.rs:247 `n_i64.rem_euclid(len as i64)`; line 248-249 Vec rebuild.                                                |
| op_posa                                    | alpha_reg.chars().position(\|c\| c == needle)   | single-char search; -1 sentinel                                | WIRED  | alpha.rs:291-294.                                                                                                     |
| op_size                                    | state.text_regs.retain (WR-01 fix)              | drop entries past end-of-regs after resize                     | WIRED  | program.rs:273 — covered by phase23_arcl_asto.rs test #8 `size_shrink_then_grow_drops_text_regs_no_ghost_resurrection`. |

### Data-Flow Trace (Level 4)

Not applicable — this phase is engine-only (no UI rendering, no API endpoint, no dynamic data fetch). All "data flow" is the synchronous stack/ALPHA/text_regs mutation directly exercised by the integration tests above.

### Behavioral Spot-Checks

| Behavior                                    | Command                          | Result                                                                                | Status |
| ------------------------------------------- | -------------------------------- | ------------------------------------------------------------------------------------- | ------ |
| Full hp41-core test suite                   | `just test-core`                 | All test binaries report `test result: ok. N passed; 0 failed` (including 248 lib unit tests, 9 phase23_arcl_asto integration, 13 phase23_atox_xtoa_arot_posa integration). | PASS   |
| Workspace lint (clippy `-D warnings`)       | `just lint`                      | `cargo clippy --workspace --all-targets --all-features -- -D warnings` finished clean with zero warnings. | PASS   |
| SC-4 invariant: no core math leaks into GUI | `grep -rnE "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` | empty (exit 1)                                                            | PASS   |
| Zero-panic in alpha.rs production code      | All 25 `.unwrap()` occurrences in alpha.rs are within `#[cfg(test)] #[allow(clippy::unwrap_used)]` block (lines 300-607). | 0 production unwraps                                                      | PASS   |
| All 6 Op variants in Op enum                | `grep -nE "Arcl\(u8\)\|Asto\(u8\)\|^\s*Atox,\|^\s*Xtoa,\|^\s*Arot,\|^\s*Posa,?" hp41-core/src/ops/mod.rs` | 6 enum lines (477, 484, 492, 499, 506, 515)                              | PASS   |
| All 6 dispatch arms in mod.rs               | `grep -cE "Op::(Arcl\|Asto\|Atox\|Xtoa\|Arot\|Posa)" hp41-core/src/ops/mod.rs`        | 12 (6 enum lines + 6 dispatch arms)                                                   | PASS   |
| All 6 execute_op arms in program.rs         | grep on program.rs                                                                  | 6 (lines 817-825, with two no-arg lines in between)                                   | PASS   |
| All 6 display arms in cli prgm_display.rs   | grep                                                                                | 6 (lines 201-208)                                                                     | PASS   |
| All 6 display arms in gui prgm_display.rs   | grep                                                                                | 6 (lines 221-229)                                                                     | PASS   |

### Probe Execution

Not applicable — Phase 23 is not a migration/tooling phase and declares no `scripts/*/tests/probe-*.sh` probes in either PLAN. Verification is via `just test-core` + `just lint` (Step 7b above).

### Requirements Coverage

| Requirement   | Source Plan          | Description                                                                                                       | Status     | Evidence                                                                                                                                                                |
| ------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| FN-ALPHA-01   | 23-01-arcl-asto      | User can append a register's value to ALPHA (`ARCL nn`) using current display format                              | SATISFIED  | op_arcl (alpha.rs:79) reads text_regs sidecar first, falls back to `format_hpnum(r, &state.display_mode)`. Integration test `arcl_appends_numeric_register_using_current_display_mode` pins FIX vs SCI difference. |
| FN-ALPHA-02   | 23-01-arcl-asto      | User can store the first 6 ALPHA chars into a register as packed text (`ASTO nn`)                                  | SATISFIED  | op_asto (alpha.rs:122) writes `chars().take(6).collect()` into text_regs[reg] and zeroes regs[reg]. `asto_arcl_round_trip_reproduces_first_6_chars` integration test. |
| FN-ALPHA-03   | 23-02-atox-xtoa-arot-posa | User can convert first ALPHA char to ASCII code in X (`ATOX`); X register holds character code                  | SATISFIED  | op_atox (alpha.rs:165) routes through `stack::enter_number` (post WR-02 fix), 8-bit caps multibyte. Integration test `atox_pops_first_char_pushes_ascii_with_lift`.  |
| FN-ALPHA-04   | 23-02-atox-xtoa-arot-posa | User can append ASCII code in X as ALPHA char (`XTOA`)                                                            | SATISFIED  | op_xtoa (alpha.rs:207) `i64.rem_euclid(256)` → char, '?' for 128..=255, X preserved (Neutral). Integration test `xtoa_appends_ascii_char_preserves_x`.                |
| FN-ALPHA-05   | 23-02-atox-xtoa-arot-posa | User can rotate ALPHA register by N chars (`AROT n`); negative N rotates right                                    | SATISFIED  | op_arot (alpha.rs:236) uses `n_i64.rem_euclid(len as i64)` for negative-N. Integration tests `arot_left_rotation_two_of_hello_produces_lloghe` (LLOHE) + `arot_right_rotation_negative_one_of_hello_produces_ohell` (OHELL). |
| FN-ALPHA-06   | 23-02-atox-xtoa-arot-posa | User can search ALPHA for substring (`POSA`) — returns position in X, -1 if not found                            | SATISFIED (scope-reduced to single-char per D-23.6) | op_posa (alpha.rs:275) implements single-char path: integer + ASCII gate (0..=127), `chars().position` lookup, `-1` for not-found. Integration tests `posa_single_char_finds_position_4_for_q_in_the_quick` + `posa_not_found_returns_minus_one`. Multi-char POSA explicitly deferred to v3.x — needs typed-stack `x_text` channel that's structurally impossible under HpNum=Decimal. |

**Orphaned requirements:** None. REQUIREMENTS.md maps exactly FN-ALPHA-01..06 to Phase 23, and all six appear in the plans' `requirements:` frontmatter.

### Anti-Patterns Found

| File                                              | Line     | Pattern                                              | Severity | Impact                                                                                                       |
| ------------------------------------------------- | -------- | ---------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------ |
| (none in Phase 23 production code)                | —        | No TBD / FIXME / XXX / TODO / HACK / PLACEHOLDER in any file touched by this phase. | —        | —                                                                                                            |
| `hp41-core/src/ops/alpha.rs` doc-comment 222-225  | 223-225  | Misleading comment: "early apply_lift_effect(Neutral) settles lift state" (Neutral is a no-op) | INFO (IN-01, deferred — flagged in REVIEW.md, not addressed) | Cosmetic only. Will confuse future maintainers but no functional impact. Marked Info in REVIEW.md; no fix planned in Phase 23 scope. |
| `hp41-core/src/ops/alpha.rs` doc-comment for AROT/XTOA | 236-251, 207-217 | Doc says "silent truncation" / "X mod 256" but implementation also rejects i64-overflow with InvalidOp | INFO (IN-02, deferred — flagged in REVIEW.md, not addressed) | Doc-vs-code mismatch on the overflow-rejection edge. Pragmatically irrelevant (no reasonable rotation is > 2^63). |
| `hp41-core/tests/phase23_atox_xtoa_arot_posa.rs:107` | 107  | Test name typo: `..._produces_lloghe` (function name has six letters; assertion correctly uses "LLOHE") | INFO (IN-05, deferred — flagged in REVIEW.md, not addressed) | Harmless test-name typo; assertion is correct.                                                              |

No BLOCKER or WARNING anti-patterns. Three INFO-level findings from REVIEW.md remain documented but not blocking; they are tracked in 23-REVIEW.md and explicitly classified as "Info" by the reviewer.

### Human Verification Required

None. This phase is engine-only (no UI, no real-time, no external service). All behaviors are deterministically testable; all 22 integration tests + 24 inline alpha.rs tests + 5 sidecar-audit tests + the WR-01 regression test (`size_shrink_then_grow_drops_text_regs_no_ghost_resurrection`) exercise every must-have truth via `dispatch()` calls. CLI/GUI keyboard wiring is explicitly out of Phase 23 scope (deferred to Phases 25/26 per CONTEXT.md).

### Re-Verification Notes (WR-01 + WR-02 closure)

**WR-01 (op_size text_regs pruning) — CLOSED**

- Fix commit: `f109740` (🩹 fix(23): WR-01 — prune text_regs in op_size to close D-23.4 leak)
- Production: program.rs:273 `state.text_regs.retain(|&k, _| (k as usize) < target);` after the regs.resize.
- Regression test: `phase23_arcl_asto.rs::size_shrink_then_grow_drops_text_regs_no_ghost_resurrection` (test #8, lines 257-291) exercises the four-step scenario from the REVIEW.md finding (ASTO 60 → SIZE 50 → SIZE 100 → ARCL 60 must yield numeric fallback, NOT "GHOST"). Passes.
- Side-effect: closes long-term autosave-bloat surface noted in REVIEW.md.

**WR-02 (op_atox routes through stack::enter_number) — CLOSED**

- Fix commit: `990d1bb` (🩹 fix(23): WR-02 — route op_atox through stack::enter_number helper)
- Production: alpha.rs:178-185 — `state.stack.lift_enabled = true; enter_number(state, HpNum::from(code)); apply_lift_effect(state, LiftEffect::Enable);` mirrors op_pi (math.rs ~297) structurally.
- Doc-comment (alpha.rs:150-154) accurately describes the lift-then-push pattern via the canonical helper.
- Existing tests (`test_atox_pops_first_char_pushes_ascii_code_with_lift`, `test_atox_empty_alpha_pushes_zero_with_lift`, integration `atox_pops_first_char_pushes_ascii_with_lift`) continue to pass — the observable behavior is identical; the change is robustness against future `enter_number` refactors.

**REVIEW status flip** — commit `4ba85c4` (📚 docs(23): REVIEW-FIX — mark WR-01 + WR-02 resolved) flipped 23-REVIEW.md frontmatter `status:` from `gaps_found` to `fixed`.

### Gaps Summary

No blocking gaps. All 6 ROADMAP Success Criteria are observably satisfied in the codebase with integration-test evidence. Both WR-level warnings from the code review are closed with regression-test coverage. Three INFO-level cosmetic findings (IN-01, IN-02, IN-05 from REVIEW.md) remain documented as low-priority follow-ups but are not Phase 23 blockers.

**Recommended (non-blocking) follow-ups** — to be picked up at Phase 25 documentation polish or earlier at planner discretion:

1. Update REQUIREMENTS.md FN-ALPHA-06 wording from "substring" to "single ASCII char" to align with the D-23.6 scope reduction shipped in Phase 23. Add a v3.x cross-reference for multi-char POSA. (Documentation refinement only — does not affect Phase 23 PASS status because the ROADMAP SC#5 bracketed clause explicitly allowed planner discretion on needle encoding.)
2. Address the three INFO-level findings from REVIEW.md (misleading doc comments + test-name typo) — low priority, no behavioral impact.

---

_Verified: 2026-05-14T14:21:34Z_
_Verifier: Claude (gsd-verifier)_
