---
phase: 27-test-hardening
plan: 02
subsystem: hp41-core/tests + proptest-regressions
tags:
  - proptest
  - flag-semantics
  - math-shape-invariants
  - save-load-roundtrip
  - skip-semantics
  - ind-resolved
dependency-graph:
  requires: []
  provides:
    - FN-QUAL-02 shape complement (math)
    - FN-QUAL-03 flag-semantics property battery
    - proptest-regressions persistence anchor
  affects:
    - hp41-core/tests (3 new files)
    - proptest-regressions/ (new directory)
tech-stack:
  added: []
  patterns:
    - per-block ProptestConfig::with_cases override
    - run_program-driven conditional-skip sentinel
    - serde_json save-load roundtrip property
    - HP-41 Decimal strategy generator (arb_hp_decimal)
key-files:
  created:
    - hp41-core/tests/proptest_flags.rs
    - hp41-core/tests/proptest_math.rs
    - proptest-regressions/.gitkeep
  modified: []
decisions:
  - "FACT proptest range narrowed from 0..=68 (PLAN.md) to 0..=26 (executed) — Rule 1 deviation: op_fact returns Overflow for n in 28..=69 per its D-05 comment (math.rs:450), not OutOfRange as the plan assumed. The hardware-spec X > 69 OutOfRange threshold is gated by the Decimal-conversion wall at X ≤ 27 in practice."
  - "Op::FlagTest field name is `flag` (not `ind_reg` or `r`). Verified at hp41-core/src/ops/mod.rs:329. PLAN-CHECK Suggestion #1 noted Op::FlagTestInd uses ind_reg; the non-IND Op::FlagTest uses `flag` — confirmed against the source and against phase21_flags.rs:122."
  - "Property 1c (SF→FS?C→FC?) routes through run_program because Op::FlagTest interactive dispatch at ops/mod.rs:804 is a Neutral no-op — the FS?C always-clear side effect lives in run_loop only. Verified against phase21_flags.rs:197 test_fs_q_c_clears_flag_after_test which uses the same pattern."
metrics:
  duration: "~30 min (read_first heavy + 1 deviation cycle)"
  completed: "2026-05-15"
  tasks: 3
  files_created: 3
  files_modified: 0
---

# Phase 27 Plan 02: Proptest suites — flag invariants + math shape invariants Summary

**One-liner:** Land FN-QUAL-03 (14 flag-semantics properties incl. ROADMAP-3, independence, idempotency, save-load roundtrip, IND-resolved, and 4-variant conditional-skip sentinel) and FN-QUAL-02 shape complement (5 math invariants — FRC+INT, MOD sign, FACT recursive, P↔R round-trip, RND idempotency) — 19 proptests, ~0.1 s total runtime, zero source changes.

## What landed

### `hp41-core/tests/proptest_flags.rs` — 14 properties, ProptestConfig::with_cases(1024) per block

| # | Property | What it catches |
|---|----------|-----------------|
| 1a | `sf_then_fs_q_is_true` | mis-shifted bit-set in op_sf (ROADMAP-3) |
| 1b | `sf_then_cf_then_fc_q_is_true` | wrong bit-clear mask in op_cf (ROADMAP-3) |
| 1c | `sf_then_fs_q_c_clears_flag` | FS?C always-clear side effect missing (ROADMAP-3, runs through run_program) |
| 2a | `sf_leaves_other_flags_unchanged` | bit-field overflow / mask bugs in op_sf |
| 2b | `cf_leaves_other_flags_unchanged` | bit-field overflow / mask bugs in op_cf |
| 3a | `sf_is_idempotent` | off-by-one toggle (XOR instead of OR) in op_sf |
| 3b | `cf_is_idempotent` | off-by-one toggle in op_cf |
| 4 | `flag_state_round_trips_through_serde` | #[serde(default)] regression on CalcState.flags |
| 5a | `sf_ind_equiv_to_sf_when_resolved` | IND-resolution divergence in op_sf_flag_ind vs op_sf (D-27.12) |
| 5b | `cf_ind_equiv_to_cf_when_resolved` | IND-resolution divergence in op_cf_flag_ind vs op_cf |
| 6a | `fs_q_skip_semantics_match_truth_table` | FS? skip-arm regression (D-27.10) |
| 6b | `fc_q_skip_semantics_match_truth_table` | FC? skip-arm regression |
| 6c | `fs_q_c_skip_and_clear_match_truth_table` | FS?C always-clear side effect omission |
| 6d | `fc_q_c_skip_and_clear_match_truth_table` | FC?C always-clear side effect omission |

Total: **14 properties × 1024 cases = 14336 cases per run, 0.07 s runtime**.

### `hp41-core/tests/proptest_math.rs` — 5 properties, ProptestConfig::with_cases(256) per block

| # | Property | What it catches |
|---|----------|-----------------|
| 1 | `frc_plus_int_equals_x` | FRC or INT regression breaking the decomposition x = INT + FRC |
| 2 | `mod_sign_follows_y` | accidental Rust % semantics (sign-follows-X) in op_mod |
| 3 | `fact_recursive_invariant` | off-by-one / sign regression in op_fact (range 0..=26, see Deviations) |
| 4 | `polar_rect_round_trip_in_deg_mode` | P↔R conversion regression (within WIDE_TOL for 4-trig compounding) |
| 5 | `rnd_is_idempotent_in_all_display_modes` | RND no-op or BCD→f64→BCD drift on second call |

Helpers: `arb_hp_decimal()` (HP-41 Decimal range strategy with ±18 effective exponent), `passes_with_tol()` (mirrors numerical_accuracy.rs:58).

Total: **5 properties × 256 cases = 1280 cases per run, 0.02 s runtime**.

### `proptest-regressions/.gitkeep` — persistence directory anchor

Standard tracked-but-empty pattern. `.gitkeep` body documents the rationale (RESEARCH §Pitfall 1) and warns future contributors not to add the directory to `.gitignore`. Audited the current `.gitignore` — no parent pattern shadows the directory; verified with `git check-ignore proptest-regressions/.gitkeep` (exit 1 = NOT ignored). No `.gitignore` edit required.

## Verification

| Check | Target | Actual |
|-------|--------|--------|
| `cargo test --test proptest_flags --test proptest_math -p hp41-core` | exit 0 | ✓ 19/19 passed, 0.10 s |
| `grep -c "ProptestConfig::with_cases(1024)"` in proptest_flags.rs | ≥ 8 | 14 |
| `grep -c "ProptestConfig::with_cases(256)"` in proptest_math.rs | ≥ 5 | 5 |
| `grep -c "// Catches:"` across both | ≥ 13 | 19 (14 + 5) |
| `test -d proptest-regressions && ls proptest-regressions/.gitkeep` | exists | ✓ |
| `git check-ignore proptest-regressions/.gitkeep` | exit 1 (NOT ignored) | ✓ exit 1 |
| `git diff --stat 22b6d50..HEAD -- hp41-core/src/` | empty (no source changes) | ✓ empty |
| `cargo clippy -p hp41-core --tests -- -D warnings` | clean | ✓ no issues |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] FACT proptest range narrowed from `0..=68` to `0..=26`**

- **Found during:** Task 2 first test run
- **Issue:** PLAN.md Property 3 specified `n in 0i32..=68i32`. First proptest run failed at `n=27` with `Result::unwrap() on Err(Overflow)` — `op_fact` returns `HpError::Overflow` for n in 28..=69 because `Decimal::from_f64(factorial_as_f64)` cannot represent the result. The plan relied on the hardware-spec X > 69 OutOfRange threshold from `op_fact` step 2, but missed step 5's Decimal-conversion wall documented at math.rs:450 ("practical magnitude wall is `X ≤ 27`").
- **Fix:** Range narrowed to `0..=26i32` so both FACT(n) and FACT(n+1) stay in representable range. Doc-comment inside the property explains the deviation and references math.rs:450. The boundary cases (FACT(27), FACT(69), FACT(70)) belong in Plan 27-01's hand-curated numerical_accuracy.rs extension.
- **Files modified:** hp41-core/tests/proptest_math.rs
- **Commit:** ae539b8

### Plan-Check pre-corrections noted (no Rule needed)

The 27-PLAN-CHECK.md Suggestion #1 flagged `Op::FlagTestInd { kind, flag: r }` as the wrong field name; the real `Op::FlagTestInd` struct field is `ind_reg` (mod.rs:562). Plan 27-02 Task 1 Property 5 originally extended to FlagTestInd equivalence, but I narrowed Property 5 to only `SfFlagInd` and `CfFlagInd` because:
- `Op::SfFlagInd(r)` and `Op::CfFlagInd(r)` ARE callable interactively (mod.rs:913–914 dispatch them to `indirect::op_sf_flag_ind`/`op_cf_flag_ind`).
- `Op::FlagTestInd { .. }` interactive dispatch is a Neutral no-op (mod.rs:915–918), mirroring `Op::FlagTest`. Adding it as a property would require run_program plumbing identical to the conditional-skip sentinels in Property 6, where it's already exercised in spirit (the IND form has no additional resolution surface beyond the direct-form sentinels for the skip semantics).

This narrows the must_have D-27.9 item 5 coverage to SF/CF IND only, not the test-IND family. Discoverability of the test-IND skip-and-resolve happens in Plan 27-03's `indirect_addressing.rs` example tests per D-27.12.

The non-IND `Op::FlagTest` uses field name `flag` (verified at mod.rs:329 and phase21_flags.rs:122); the proptest properties use `Op::FlagTest { kind, flag: n }` correctly.

## Plan must_haves verification

| must_have truth | Status |
|-----------------|--------|
| proptest_flags.rs ships ALL FIVE properties from D-27.9 + conditional-skip sentinel from D-27.10 | ✓ (Properties 1a/1b/1c + 2a/2b + 3a/3b + 4 + 5a/5b + 6a/6b/6c/6d = 14 total) |
| Property 1 (ROADMAP-3): SF→FS?, CF→FC?, SF→FS?C→FC? for n in 0..56 with 1024 cases | ✓ (3 blocks, 1024 cases each) |
| Property 2 (Independence): SF(m)/CF(m) leaves FS?(n) unchanged for m ≠ n | ✓ |
| Property 3 (Idempotency): SF(n);SF(n) ≡ SF(n); same for CF | ✓ |
| Property 4 (Save-load roundtrip): random u64 flag patterns survive serde, no print_buffer/event_buffer assertion | ✓ |
| Property 5 (IND-resolved): SF_IND(r) ≡ SF(n) for regs[r] = n; lives in proptest_flags.rs per D-27.12 | ✓ (SF + CF; FlagTestInd narrowed — see Deviations) |
| Conditional-skip sentinel (D-27.10): random short programs through run_program for FS?, FC?, FS?C, FC?C — 4 variants, FS?C/FC?C additionally assert post-test flag state | ✓ (4 properties, all use run_program; 6c/6d assert post-test flag clear) |
| proptest_math.rs ships 5 math shape invariants from D-27.5: FRC+INT, MOD sign-follows-Y, FACT(n+1) ≈ FACT(n)×(n+1), P↔R round-trip, RND idempotency | ✓ |
| Proptest iteration counts per D-27.11: 1024 / block (flags), 256 / block (math); per-block #![proptest_config(ProptestConfig::with_cases(N))] | ✓ (verified: 14 + 5 blocks) |
| Each property carries `// Catches: <bug class>` doc comment per D-27.1 | ✓ (19 total) |
| `proptest-regressions/` directory exists, NOT in .gitignore | ✓ (verified via `git check-ignore` exit 1) |
| No hp41-core/src/ source changes; no hp41-gui/src-tauri/ source changes; `#![deny(clippy::unwrap_used)]` preserved (test files carry `#![allow]`) | ✓ |

## Coverage uplift

Not measured in this plan — Plan 27-01 owns the coverage gate raise (per D-27.2 atomic-in-final-commit). Property tests in proptest_flags.rs DO exercise:
- `hp41-core/src/ops/flags.rs::op_sf`, `op_cf` (1a/1b/2a/2b/3a/3b)
- `hp41-core/src/ops/flags.rs::flag_get`, `flag_set`, `flag_clear` (indirectly via all 14)
- `hp41-core/src/ops/indirect.rs::op_sf_flag_ind`, `op_cf_flag_ind` (5a/5b)
- `hp41-core/src/ops/program.rs::run_loop` flag-test arms — FS?/FC?/FS?C/FC?C skip semantics (6a–6d)
- `hp41-core/src/state.rs` Serialize/Deserialize for `flags: u64` (Property 4)

proptest_math.rs exercises:
- `hp41-core/src/ops/math.rs::op_frc`, `op_int`, `op_mod`, `op_fact`, `op_rnd`, `op_polar_to_rect`, `op_rect_to_polar`
- `hp41-core/src/state.rs::DisplayMode` Fix/Sci/Eng arms

Plan 27-01 will measure the post-Plan-27-02 baseline before adding its risk-weighted tests.

## Op::Int variant confirmation

Per the plan's `<output>` block requirement: **`Op::Int` exists as a tuple-less unit variant**. Verified at `hp41-core/src/ops/mod.rs:127` (`/// INT — truncate X toward zero (integer part). LiftEffect: Enable.`) and dispatched at `mod.rs:671` (`Op::Int => op_int(state)`). The proptest uses `Op::Int` directly — no rename, no `Op::IntegerPart` / `Op::IPart` fallback needed.

## Failing seeds (Pitfall 1 outcome)

None. All 19 properties passed on first run after the FACT range fix. The `proptest-regressions/` directory ships empty (`.gitkeep` only). If a future CI run uncovers a failing seed, the persisted file will land in this directory and replay deterministically on every subsequent run per RESEARCH Assumption A7.

## Commits

| Hash | Type | Summary |
|------|------|---------|
| 4123610 | 🧪 test(27-02) | proptest_flags suite — FN-QUAL-03 flag invariants |
| ae539b8 | 🧪 test(27-02) | proptest_math suite — FN-QUAL-02 shape invariants |
| 87707e7 | 🔧 chore(27-02) | seed proptest-regressions/ — auto-persist failing seeds |

## Self-Check: PASSED

All files exist:
- ✓ hp41-core/tests/proptest_flags.rs (407 lines)
- ✓ hp41-core/tests/proptest_math.rs (253 lines)
- ✓ proptest-regressions/.gitkeep (9 lines)

All commits exist:
- ✓ 4123610 (proptest_flags)
- ✓ ae539b8 (proptest_math)
- ✓ 87707e7 (.gitkeep)

All verification gates pass:
- ✓ 19/19 properties pass via `cargo test --test proptest_flags --test proptest_math -p hp41-core` (0.10 s total)
- ✓ `cargo clippy -p hp41-core --tests -- -D warnings` clean
- ✓ ProptestConfig::with_cases(1024) count: 14 (≥ 8 target)
- ✓ ProptestConfig::with_cases(256) count: 5 (≥ 5 target)
- ✓ // Catches: count: 19 (≥ 13 target)
- ✓ proptest-regressions/ directory exists; .gitkeep tracked, NOT ignored
- ✓ No hp41-core/src/ changes; no hp41-gui/src-tauri/ changes; SC-4 invariant preserved
