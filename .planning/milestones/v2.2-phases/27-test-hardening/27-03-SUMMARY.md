---
phase: 27-test-hardening
plan: 03
subsystem: hp41-core/tests
tags:
  - integration-tests
  - indirect-addressing
  - happy-path
  - non-integer-rejection
  - phase24-complement
  - fn-qual-04
requires: []
provides:
  - "hp41-core/tests/indirect_addressing.rs — 42 IND integration tests (17 ops × {happy, reject} + skip-semantic variations + 2 D-27.12 cross-cut anchors)"
affects:
  - "FN-QUAL-04 (REQUIREMENTS.md line 106) — primary deliverable"
  - "ROADMAP SC-4 (line 199) — `indirect_addressing.rs` file existence + content shape"
tech-stack:
  added: []
  patterns:
    - "ind_happy_and_reject! macro (Pattern A) — single source of truth for 22 plain-dispatch IND tests"
    - "run_program-driven Pattern B for 6 skip-semantic IND ops — mirrors phase24_ind_variants.rs::isg_ind_inside_run_loop precedent"
    - "Y-register disambiguation for skip vs no-skip branches (X is identical, Y is the actual signal)"
key-files:
  created:
    - "hp41-core/tests/indirect_addressing.rs (822 lines, 42 tests)"
  modified: []
decisions:
  - "Variant-shape corrections applied per PLAN-CHECK Suggestion #1: Op::StoArithInd is a TUPLE variant (u8, StoArithKind); Op::FlagTestInd struct field is `ind_reg` not `flag`. Plan template snippets that paste verbatim would have compile-failed; corrected during read_first."
  - "FS?C and FC?C flag-state assertions folded into each happy test (plan permitted alternative: 'or fold into the first test's assertion'); 3 tests per FlagTestInd kind instead of 4. Total Pattern-B: 18 tests + 2 cross-cut = 20 (vs plan's 20 target — same count, different layout)."
  - "Added 2 cross-cut tests (`sf_ind_fn_qual_04_equiv_to_sf_when_resolved_n_12`, `rcl_ind_fn_qual_04_equiv_to_rcl_when_resolved_n_12`) as D-27.12 documentary anchors — concrete pinpoints for the PROPERTY tests landing in Plan 27-02's proptest_flags.rs."
metrics:
  duration: "~25 min (read_first + write + verify + commit)"
  completed: "2026-05-15"
  test_count: 42
  line_count: 822
  passing: 42
  failing: 0
---

# Phase 27 Plan 03: IND Integration Suite Summary

**One-liner:** Ships `hp41-core/tests/indirect_addressing.rs` — 42 integration tests covering happy-path + non-integer rejection for every `_IND` op named in FN-QUAL-04 (REQUIREMENTS.md line 106), with skip-semantic ops driven through `run_program` per the Phase 24 precedent.

## Per-Op Coverage Audit

| Op | Variant shape | Tests | Pattern | Notes |
|---|---|---|---|---|
| 1. STO_IND | `Op::StoInd(u8)` | 2 | A | happy + reject |
| 2. RCL_IND | `Op::RclInd(u8)` | 2 (+1 cross-cut) | A | happy + reject + D-27.12 equiv |
| 3. ISG_IND | `Op::IsgInd(u8)` | 3 | B | under-final (no skip) + at-final (skip) + reject |
| 4. DSE_IND | `Op::DseInd(u8)` | 3 | B | above-final (no skip) + at-or-below (skip) + reject |
| 5. SF_IND | `Op::SfFlagInd(u8)` | 2 (+1 cross-cut) | A | happy + reject + D-27.12 equiv |
| 6. CF_IND | `Op::CfFlagInd(u8)` | 2 | A | happy + reject |
| 7. FS?_IND | `Op::FlagTestInd { kind: IsSet, ind_reg }` | 3 | B | flag-set (no skip) + flag-clear (skip) + reject |
| 8. FC?_IND | `Op::FlagTestInd { kind: IsClear, ind_reg }` | 3 | B | flag-clear (no skip) + flag-set (skip) + reject |
| 9. FS?C_IND | `Op::FlagTestInd { kind: IsSetThenClear, ind_reg }` | 3 | B | flag-set (no skip + clear) + flag-clear (skip + clear) + reject |
| 10. FC?C_IND | `Op::FlagTestInd { kind: IsClearThenClear, ind_reg }` | 3 | B | flag-clear (no skip + clear) + flag-set (skip + clear) + reject |
| 11. STO+_IND | `Op::StoArithInd(u8, StoArithKind::Add)` | 2 | A | happy + reject |
| 12. STO-_IND | `Op::StoArithInd(u8, StoArithKind::Sub)` | 2 | A | happy + reject; orientation Y-X verified |
| 13. STO×_IND | `Op::StoArithInd(u8, StoArithKind::Mul)` | 2 | A | happy + reject |
| 14. STO÷_IND | `Op::StoArithInd(u8, StoArithKind::Div)` | 2 | A | happy + reject; orientation Y/X verified |
| 15. ARCL_IND | `Op::ArclInd(u8)` | 2 | A | happy + reject; alpha_reg formatted-append |
| 16. ASTO_IND | `Op::AstoInd(u8)` | 2 | A | happy + reject; text_regs sidecar + numeric zero |
| 17. VIEW_IND | `Op::ViewInd(u8)` | 2 | A | happy + reject; R9 mitigation (resolved-value display) |

**Totals:** 17 ops × 2 base = 34 minimum ✓; with skip-semantic variations + cross-cut anchors = **42 tests** (matches plan's stretch target).

## D-27.1 Rationale Coverage

- **Source-level `// Catches:` comments:** 31 (verified via `grep -c "Catches:" hp41-core/tests/indirect_addressing.rs`)
- **Effective coverage after macro expansion:** every one of the 42 tests has a Catches rationale — Pattern A macro body contains 2 (one per happy/reject pair), expanded 11× = 22; Pattern B tests have explicit comments inline = 20.
- **Per-op header rationales:** every macro invocation site is prefixed by a `// Catches:` comment naming the specific bug class for that op (e.g., "STO_IND writing to the POINTER register instead of the RESOLVED register").

## D-27.12 Paradigm Split Compliance

Module doc comment explicitly cites D-27.12:

> *"The IND-resolved flag PROPERTY (D-27.9 item 5 — `SF_IND(r) ≡ SF(n)` when regs[r]=n) lives in `proptest_flags.rs` (Plan 27-02) per the D-27.12 paradigm split. This file ships example tests; that file ships properties."*

Plus 2 concrete example tests (`sf_ind_fn_qual_04_equiv_to_sf_when_resolved_n_12`, `rcl_ind_fn_qual_04_equiv_to_rcl_when_resolved_n_12`) that serve as documentary pinpoints for the PROPERTY at the canonical n=12 case. Both pass at HEAD against the current Phase 24 implementation, so they ALSO confirm the existing implementation already satisfies the equivalence — Plan 27-02's proptest will not surprise anyone.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug, plan template] `Op::StoArithInd` variant shape**

- **Found during:** Read of `hp41-core/src/ops/mod.rs:537` (read_first per plan)
- **Issue:** Plan template at Task 1 step 10 uses struct form `Op::StoArithInd { kind, n }`; canonical shape is TUPLE `Op::StoArithInd(u8, StoArithKind)` per PLAN-CHECK Suggestion #1.
- **Fix:** All 4 STO+/-/×/÷ IND test invocations use the correct tuple form `Op::StoArithInd(5, StoArithKind::Add)`. Verified at `phase24_ind_variants.rs:123`.
- **Files modified:** `hp41-core/tests/indirect_addressing.rs`
- **Commit:** `e5ff551`

**2. [Rule 1 — Bug, plan template] `Op::FlagTestInd` field name `flag` vs `ind_reg`**

- **Found during:** Read of `hp41-core/src/ops/mod.rs:560-563` (read_first per plan)
- **Issue:** Plan template at Task 2 step 4 uses `Op::FlagTestInd { kind, flag: r }`; canonical field name is `ind_reg` per PLAN-CHECK Suggestion #1.
- **Fix:** All 12 FlagTestInd invocations (4 kinds × 3 tests each) use `ind_reg: 5`. Verified at `phase24_ind_variants.rs:329-331`.
- **Files modified:** `hp41-core/tests/indirect_addressing.rs`
- **Commit:** `e5ff551`

### Plan-permitted Variations

**3. FS?C / FC?C: 3 tests per kind instead of 4**

The plan body offered both options for FS?C and FC?C: "4 tests (happy x2 + reject + post-test flag-state assertion)" OR "fold into the first test's assertion". I chose the FOLD option for cohesion — each happy test asserts BOTH the X/Y stack disambiguation AND the post-test flag state. Result: 3 tests per FlagTestInd-clear kind (matches FS?/FC? structure), 18 Pattern-B tests total instead of the 20 in one alternative ordering. Compensated by 2 cross-cut anchors to reach 42 total.

## Audit Trail

```
$ cargo test -p hp41-core --test indirect_addressing
running 42 tests
... (all 42 ok) ...
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo clippy -p hp41-core --tests -- -D warnings
... clean ...

$ git diff --stat HEAD~1 HEAD
 hp41-core/tests/indirect_addressing.rs | 822 +++++++++++++++++++++++++++++++
 1 file changed, 822 insertions(+)

$ git diff --stat HEAD~1 HEAD -- hp41-core/src/
(empty — no source changes per Phase 25 FROZEN invariant)

$ git diff --stat HEAD~1 HEAD -- hp41-gui/
(empty — no GUI changes per SC-4 invariant)
```

## Nyquist Verification Dimensions

- **Behavioral:** All 17 IND ops resolve correctly on integer pointers (regs[5]=12 ⇒ effect on regs[12] / flag 12 / display of regs[12]) and reject 12.5 with `HpError::InvalidOp`. Verified by `cargo test` — 42/42 pass.
- **Functional:** Every FN-QUAL-04 enumerated op has exactly 2+ dispatched tests (happy + reject) for Pattern-A; skip-semantic ops add additional run_program tests for both skip and execute branches (verified by per-op audit table above).
- **Regression:** A future regression in `resolve_indirect` (e.g., accidentally accepting 12.5) fires all 17 reject tests. A regression in `run_loop`'s IsgInd / DseInd / FlagTestInd arms (e.g., inverted skip truth table) fires the relevant Pattern-B test — disambiguated by the Y-register assertion.

## Coverage Uplift (Informational)

- New surface: ~50 lines of `hp41-core/src/ops/indirect.rs` and ~30 lines of `hp41-core/src/ops/program.rs::run_loop` (FlagTestInd / IsgInd / DseInd arms) are now covered by 2+ tests each.
- The actual coverage-gate raise (80→95) lives in Plan 27-01 Task 4 per D-27.2 — this plan's contribution is the IND surface specifically.
- Estimated lines-covered delta on `hp41-core` from this plan alone: ~80–120 lines (informational only; measured by `just coverage` after Plan 27-01 lands).

## Self-Check: PASSED

- Created file exists: `hp41-core/tests/indirect_addressing.rs` ✓ (822 lines, 42 tests)
- Commit exists: `e5ff551` ✓ (will be verified by SUMMARY commit below)
- All 17 FN-QUAL-04 ops covered with happy + reject ✓ (per-op audit table)
- All Pattern-B ops drive through `run_program` ✓ (24 `run_program` references in the file)
- No `hp41-core/src/` source changes ✓
- No `hp41-gui/src-tauri/` source changes ✓
- `cargo clippy --tests -D warnings` clean ✓
- `cargo test --test indirect_addressing` 42/42 pass ✓
- D-27.12 cross-reference present ✓ (module doc + 2 documentary cross-cut tests)
- Variant-shape corrections applied (PLAN-CHECK Suggestion #1) ✓
