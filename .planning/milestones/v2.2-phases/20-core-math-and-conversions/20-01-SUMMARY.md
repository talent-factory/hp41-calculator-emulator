---
phase: 20-core-math-and-conversions
plan: 01
subsystem: hp41-core
status: Complete
completed: 2026-05-14
tags: [hp41, rust, rpn, math, stack, bcd-decimal, phase-20]
requirements:
  - FN-MATH-01
  - FN-MATH-02
  - FN-MATH-03
  - FN-MATH-04
  - FN-MATH-05
  - FN-MATH-06
  - FN-MATH-07
  - FN-MATH-08
  - FN-MATH-09
  - FN-STACK-01
dependency-graph:
  requires:
    - v2.1 baseline (Phase 19 Keyboard Authenticity shipped 2026-05-13)
    - hp41-core::format (Phase 2 plan 04 `format_hpnum` + display tests)
    - hp41-core::ops::math (Phase 2 trig f64-bridge precedents)
    - hp41-core::ops::stack_ops (op_rdn template for op_r_up)
  provides:
    - 10 new Op variants reachable via dispatch + execute_op
    - HpError::OutOfRange variant (FACT pre-flight guard)
    - format::round_to_display_precision helper (single source of truth)
  affects:
    - Phase 25 (CLI keyboard wiring — `key_to_op` + `KEY_REF_TABLE` for the 10 ops)
    - Phase 26 (GUI key map — `key_map::resolve` + `KEY_DEFS` for the 10 ops)
    - Phase 27 (numerical-accuracy suite extension — Phase 20 must NOT regress)
tech-stack:
  added: []
  patterns:
    - "f64 bridge for irrational results (mirrors trig precedent)"
    - "HpNum-native path for value-mutating ops (FRC, ABS, SIGN, MOD, R↑)"
    - "Direct stack assignment for binary-out ops (P→R, R→P)"
    - "Single-source-of-truth helper extraction (round_to_display_precision)"
    - "Zero-panic policy: .expect(\"reason\") + Decimal::from_f64(...).ok_or(Overflow)"
key-files:
  created:
    - hp41-core/tests/phase20_math.rs (324 LOC, 20 tests)
  modified:
    - hp41-core/src/error.rs (added OutOfRange variant)
    - hp41-core/src/format.rs (added round_to_display_precision + 4 inline tests)
    - hp41-core/src/ops/mod.rs (10 new Op variants + 10 dispatch arms)
    - hp41-core/src/ops/math.rs (9 new pub fn op_* bodies)
    - hp41-core/src/ops/stack_ops.rs (op_r_up body)
    - hp41-core/src/ops/program.rs (10 new execute_op arms)
    - hp41-cli/src/prgm_display.rs (10 new op_display_name arms + inline test)
    - hp41-gui/src-tauri/src/prgm_display.rs (10 new op_display_name arms + inline test)
    - .planning/ROADMAP.md (SC-1 PI literal editorial correction per D-09)
decisions:
  - "D-01/D-02/D-03: extract round_to_display_precision in format.rs; RND and format_hpnum share one source of truth"
  - "D-04: FACT computed via f64 internally; final convert through Decimal::from_f64(...).map(HpNum::rounded).ok_or(Overflow)"
  - "D-05: practical magnitude cap X ≤ 27 (Decimal range ~7.9e28); X ∈ [28, 69] returns Overflow"
  - "D-06: FACT pre-flight X > 69 returns HpError::OutOfRange (new variant) — preserves SC-3 hardware-spec wording"
  - "D-07: FACT integer check via x == x.trunc_int(); negative integers also Domain"
  - "D-08: PI = 3.141592654 (parse 3.141592653589793 then HpNum::rounded → 10 sig digits)"
  - "D-09: ROADMAP SC-1 PI literal corrected from 3.1415926536 (11 digits) to 3.141592654 (10 digits)"
  - "D-10: PI LiftEffect = Enable (mirrors op_lastx — force lift_enabled=true, enter_number, LiftEffect::Enable)"
  - "D-11/D-12/D-13: P→R and R→P use f64 bridge; direct stack assignment; LASTX ← consumed X; angle mode honored via to_radians_f64 / f64_from_radians"
  - "D-14 (UPDATED 2026-05-13): MOD = Y − X · trunc(Y/X) using HpNum::trunc_int(); sign follows Y; 7 MOD -3 = 1, -7 MOD 3 = -1"
  - "D-15: FRC = x − trunc_int(x); sign-preserving (FRC(-3.7) = -0.7)"
  - "D-16: ABS via HpNum::negate for negatives"
  - "D-17: SIGN returns -1 / 0 / +1"
  - "D-18: SIGN-on-ALPHA divergence deferred to Phase 25 docs (no value-type tag on X)"
  - "D-19/D-20: op_r_up in stack_ops.rs; mirror of op_rdn with reversed assignment chain; LiftEffect::Neutral; does NOT update LASTX"
  - "D-21: single plan for all 10 Phase 20 ops"
  - "D-22: 4-place Op-variant rule enforced via Rust exhaustive matches in dispatch + execute_op + 2× prgm_display"
  - "D-23: #![deny(clippy::unwrap_used)] policy preserved; all new code uses ?-propagation or .expect(\"reason\")"
  - "D-24: SC-4 invariant preserved — only prgm_display.rs touched under hp41-gui/src-tauri/"
  - "D-25: LiftEffect summary observed — PI/P→R/R→P/RND/FRC/MOD/ABS/SIGN/FACT = Enable; R↑ = Neutral"
metrics:
  duration: 1 session (~1 hour)
  task-count: 6
  commits: 7 (6 task commits + 1 editorial correction)
  new-tests: 20 integration tests + 4 inline tests in format.rs + 1 inline test in each prgm_display.rs copy
  test-results: just ci green; cargo llvm-cov --fail-under-lines 92.5 -p hp41-core exits 0
  coverage: 92.65% lines (above 92.5% non-regression target; v2.1 baseline)
  numerical-accuracy-suite: 500/500 (non-regression preserved)
---

# Phase 20 Plan 01: Core Math & Conversions Summary

Phase 20 / Plan 20-01 ships the 10 missing HP-41CV ROM math/stack ops (PI, P→R, R→P, RND, FRC, MOD, ABS, FACT, SIGN, R↑) inside `hp41-core` with hardware-faithful semantics, plus a single `round_to_display_precision` helper that lets RND and the FIX/SCI/ENG display path share one source of truth. All 4 ROADMAP success criteria (SC-1..SC-4) now have at least one passing integration test in `hp41-core/tests/phase20_math.rs`; SC-5 is enforced at compile time by four Rust exhaustive matches (dispatch, execute_op, and both `prgm_display.rs` copies).

## Status

**Complete.** All six tasks landed, all per-task verify gates green, full pipeline (`just ci` + `cargo llvm-cov --fail-under-lines 92.5 -p hp41-core`) green, ROADMAP editorial correction filed.

## Files Touched (9 + 1 doc)

| File | Change |
| --- | --- |
| `hp41-core/src/error.rs` | Added `HpError::OutOfRange` |
| `hp41-core/src/format.rs` | Added `pub fn round_to_display_precision` + 4 inline tests |
| `hp41-core/src/ops/mod.rs` | 10 new `Op` variants + 10 dispatch arms; extended math/stack_ops imports |
| `hp41-core/src/ops/math.rs` | 9 new `pub fn op_*` bodies (PI, P→R, R→P, RND, FRC, MOD, ABS, FACT, SIGN) |
| `hp41-core/src/ops/stack_ops.rs` | New `pub fn op_r_up` body |
| `hp41-core/src/ops/program.rs` | 10 new `execute_op` arms (mirrors dispatch) |
| `hp41-cli/src/prgm_display.rs` | 10 new `op_display_name` arms + `test_display_phase20_op_labels` |
| `hp41-gui/src-tauri/src/prgm_display.rs` | 10 new `op_display_name` arms + `test_display_phase20_op_labels` |
| `hp41-core/tests/phase20_math.rs` | **New file** — 20 integration tests covering SC-1..SC-4 and per-op happy + error paths |
| `.planning/ROADMAP.md` | Editorial: SC-1 PI literal `3.1415926536` → `3.141592654` (D-09) |

## What Landed

**Wave-0 (Task 1 — `8064250`)**
- Extracted `round_to_display_precision(n: &HpNum, mode: &DisplayMode) -> HpNum` in `hp41-core/src/format.rs` per D-01/D-02. Helper covers FIX(n) (round-half-away-from-zero to n decimal places), SCI(n) (n+1 sig digits via `round_sf_with_strategy` with the `.expect("round_sf_with_strategy(<= 10) cannot fail for finite Decimal")` idiom mirroring `num.rs:18`), and ENG(n) (mantissa rounding + exponent constrained to multiples of 3, with the same carry handling as `format_eng`). Zero short-circuits to `HpNum::zero()`.
- `format_hpnum` is unchanged (Shape A) — all 15 FIX/SCI/ENG display tests and the 500-case `numerical_accuracy` suite stay green.
- Inline smoke tests: -5.65 → -5.7 (FIX 1), -5.7 → -6 (FIX 0), 9.9995 → 10 (SCI 3 carry), zero-handling across all modes.

**Wave-1 (Tasks 2–5 — `ae0c1eb`, `5fd4e5f`, `0512c5c`, `8b90b58`)**
- **Task 2 (`ae0c1eb`)**: added 10 new `Op` variants (Pi, Rup, Rnd, Frc, Abs, Sign, Fact, Mod, PolarToRect, RectToPolar), 10 dispatch arms, and `HpError::OutOfRange`. `op_*` functions ship as stubs returning `Err(HpError::InvalidOp)` so the crate compiles under `#![deny(clippy::unwrap_used)]`.
- **Task 3 (`5fd4e5f`)**: mirrored the 10 arms in `execute_op` so the new ops are reachable inside running programs. Imports extended in `program.rs`.
- **Task 4 (`0512c5c`)**: replaced stubs with real implementations.
  - `op_pi`: parses `"3.141592653589793"`, routes through `HpNum::rounded` → exact 10-sig-digit value `3.141592654` (D-08). Forces `lift_enabled = true`, calls `enter_number`, re-applies `LiftEffect::Enable` (D-10). Does NOT update LASTX (constant push, not arithmetic — same convention as `op_lastx`).
  - `op_polar_to_rect` and `op_rect_to_polar`: f64 bridge per the trig precedent. Reads use `.to_f64().expect("HpNum is always within f64 range")` (the `format.rs:161` idiom, eliminating the unreachable Overflow branch on the read side). Final-result writes use `Decimal::from_f64(...).map(HpNum::rounded).ok_or(HpError::Overflow)?` (Overflow is reachable here — NaN/Inf from f64 trig). Direct stack assignment + explicit `LiftEffect::Enable` (D-11..D-13). R→P magnitude uses `f64::hypot` for improved numerical accuracy; angle uses `xc.atan2(yc)` (HP-41 convention: Y reg = x-coord, X reg = y-coord).
  - `op_rnd`: delegates to `crate::format::round_to_display_precision` then `unary_result` (D-01..D-03).
  - `op_frc`: `x.checked_sub(&x.trunc_int())` — sign-preserving (D-15).
  - `op_abs`: `negate()` for negatives, pass-through otherwise (D-16).
  - `op_sign`: `-1` / `0` / `+1` via `is_zero` + `is_sign_negative` (D-17).
  - `op_fact`: strict 5-step check — read-as-f64 → `OutOfRange` pre-flight if `v > 69` (D-06) → integer check (D-07) → negative check (D-07) → iterative f64 product (loop bounded 1..=69) → `Decimal::from_f64(...)` catches X ≥ 28 magnitude wall (D-04/D-05).
  - `op_mod`: implements the corrected D-14 formula `Y - X * trunc(Y/X)` using `HpNum::trunc_int()`. Sign follows Y. Domain error if X = 0. LiftEffect: Enable via `binary_result`. Doc comment explicitly cites the corrected D-14 (UPDATED 2026-05-13) and includes the `7 MOD -3 = 1` and `-7 MOD 3 = -1` examples.
  - `op_r_up`: exact mirror of `op_rdn` with reversed assignment chain (`X←T; T←Z; Z←Y; Y←X`). Does NOT update LASTX (D-19). `LiftEffect::Neutral` (D-20/D-25).
- **Task 5 (`8b90b58`)**: added the 10 byte-identical `op_display_name` arms in both `hp41-cli/src/prgm_display.rs` and `hp41-gui/src-tauri/src/prgm_display.rs` (the documented SC-4 exception per CLAUDE.md). Mnemonics: `"PI"`, `"R↑"` (U+2191), `"P→R"` (U+2192), `"R→P"`, `"RND"`, `"FRC"`, `"MOD"`, `"ABS"`, `"FACT"`, `"SIGN"`. `test_display_phase20_op_labels` added in both files asserting the exact codepoint strings.

**Wave-2 (Task 6 — `fdc6b89`)**
- Created `hp41-core/tests/phase20_math.rs` with 20 integration tests (well above the 14-minimum gate). Covers SC-1..SC-4 (SC-5 is compile-time) plus per-op happy and error paths. The three deterministic MOD tests (`test_mod_seven_mod_neg_three` = +1, `test_mod_neg_seven_mod_three` = −1, `test_mod_div_by_zero` = Domain) verify the corrected D-14 semantic. f64-bridge ops use a `close_enough` helper with 1e-7 tolerance (looser than HpNum's 10-sig-digit floor).

**Editorial correction (`537704d`)**
- ROADMAP SC-1 PI literal: `3.1415926536` (11 sig digits, incorrect) → `3.141592654` (10 sig digits, HP-41 hardware value). Filed as a separate small docs commit per D-09.

## Decisions Honored

All 25 locked decisions (D-01 through D-25) are observed. The frontmatter `decisions:` list captures one-line per-decision summaries. Highlights:

- **D-14 was UPDATED 2026-05-13** (after the original CONTEXT was written): MOD now uses the trunc-toward-zero formula `Y - X * trunc(Y/X)` with sign following Y. The `op_mod` doc comment explicitly cites the corrected semantic and includes both worked examples. `REQUIREMENTS.md FN-MATH-06` was updated in parallel with the explicit `7 MOD -3 = 1` and `-7 MOD 3 = -1` examples; the three dedicated MOD tests assert these exact values.
- **D-23 (zero-panic policy)**: every new line of production code uses `?`-propagation, `.expect("HpNum is always within f64 range")` (format.rs:161 idiom for in-range-by-construction reads), or `.map(HpNum::rounded).ok_or(HpError::Overflow)` (final-result conversions where Overflow is reachable). `grep -c '\.unwrap()' hp41-core/src/ops/math.rs` returns 0.
- **D-24 (SC-4 invariant)**: `grep -rn 'fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)' hp41-gui/src-tauri/src/` returns no matches — only `op_display_name` arms touched under hp41-gui.

## Deviations from Plan

- **Tasks 2 and 3 are coupled by Rust's exhaustive-match rule.** Adding the 10 new `Op` variants makes the `execute_op` match in `program.rs` non-exhaustive. Task 2's verify (`cargo build -p hp41-core`) cannot pass without Task 3's changes also being in the working tree. Resolved by leaving `program.rs` modified-but-unstaged during Task 2's commit, then staging it for Task 3's commit. Each commit's diff is properly scoped (Task 2's commit contains the Op enum + dispatch + stubs; Task 3's commit contains the execute_op arms), and each task's verify gate runs against the working tree (which compiles at both moments). This satisfies the "atomic per-task commits with passing verify" rule without violating the per-task file scopes.

No other deviations. No auto-fixes needed (deviation Rules 1/2/3 not triggered). No checkpoints reached.

## Test Results

- `cargo test -p hp41-core --test phase20_math` — **20/20 pass**
- `cargo test -p hp41-core --test numerical_accuracy` — **500/500 pass** (non-regression preserved)
- `cargo test -p hp41-core --test format_tests` — 15/15 pass (no regression from Task 1 refactor)
- `cargo test -p hp41-core --test math_tests` — 27/27 pass
- `cargo test -p hp41-core --test stack_tests` — 18/18 pass
- `cargo test -p hp41-cli` — 136/136 pass
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` — 51/51 pass
- `just ci` — green (lint → test → coverage all pass)
- `cargo llvm-cov clean --workspace && cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` — **92.65% lines**, above the 92.5% non-regression target

## Coverage Snapshot (hp41-core, lines)

| File | Coverage |
| --- | ---: |
| `ops/math.rs` | 93.45% |
| `ops/mod.rs` | 92.66% |
| `ops/stack_ops.rs` | 100.00% |
| `format.rs` | 79.53% |
| `error.rs` | (small file; new variant covered by error-path tests) |
| **Total `hp41-core`** | **92.65%** |

The `format.rs` coverage dip is from the new `round_to_display_precision` ENG branch (which is not yet directly exercised by integration tests — its inputs to `op_rnd` use FIX in the SC-3 test, and ENG paths are exercised via `format_hpnum` only). The total non-regression gate still passes.

## Editorial Corrections Filed

- **ROADMAP SC-1 wording** (`537704d`): PI literal `3.1415926536` → `3.141592654` per D-09. Now consistent with the hardware-faithful HP-41 display and with the `op_pi` implementation.

## CLAUDE.md Additions (suggested for v2.2 wrap-up)

When v2.2 is fully shipped, the following notes belong in CLAUDE.md "v2.2 additions" block (not added now — the orchestrator owns milestone-state updates):

1. **`round_to_display_precision` is the RND single-source-of-truth.** Future RND-adjacent ops (e.g. CR-aware rounding) must consume this helper rather than re-implementing the FIX/SCI/ENG mantissa logic.
2. **MOD trunc-toward-zero convention (D-14).** Cite HP-41C Owner's Manual + Free42 source. Examples: `7 MOD -3 = 1`, `-7 MOD 3 = -1`.
3. **SIGN-on-ALPHA divergence (D-18).** Phase 20 always returns numeric; HP-41 hardware returns 0 for ALPHA-typed X. Our `CalcState` model has no type tag on X — divergence is documented but not modeled. Revisit alongside FN-ALPHA-03 in Phase 23.
4. **FACT magnitude wall (D-05).** Practical cap is X ≤ 27 (Decimal range ~7.9e28); X ∈ [28, 69] returns `HpError::Overflow`; X > 69 returns `HpError::OutOfRange` (hardware-spec pre-flight). Hardware-faithful X ≤ 69 would require extending HpNum representation — backlog item.

## Out-of-Scope (deferred to later phases)

- **SIGN-on-ALPHA typing** — Phase 23 candidate alongside FN-ALPHA-03 (`ATOX`).
- **Numerical-accuracy suite extension** for the 10 new ops — Phase 27 owns extending the 500-case suite. Phase 20 only preserves non-regression.
- **CLI keyboard wiring** for the 10 new ops (`key_to_op`, `KEY_REF_TABLE`, `help_data.rs`) — Phase 25.
- **GUI key map wiring** (`key_map::resolve`, `KEY_DEFS` in `Keyboard.tsx`) — Phase 26.
- **HpNum high-magnitude representation** (would lift FACT cap to X ≤ 69 per hardware spec) — v3.0 backlog candidate.

## Follow-ups for Phases 25 / 26

The 10 new `Op` variants are dispatchable and programmable today, but **not yet click/key-reachable** from the CLI or GUI:

- **Phase 25 (hp41-cli)**: extend `hp41-cli/src/keys.rs::key_to_op()`, add `KEY_REF_TABLE` entries, update `help_data.rs::HELP_DATA`, wire `pending_prompt()` arms if any of the new ops need a parameter modal (none currently do — they are all parameter-free).
- **Phase 26 (hp41-gui)**: extend `hp41-gui/src-tauri/src/key_map.rs::resolve()` with string IDs for the 10 ops (e.g., `"pi"`, `"p_to_r"`, `"rnd"`, `"rup"`, `"fact"`, `"sign"`, `"abs"`, `"frc"`, `"mod_op"`); add `KEY_DEFS` entries in `hp41-gui/src/Keyboard.tsx` with three-label spec (primary + optional shifted + alphaChar); remove the corresponding stub-error entries from `key_map.rs` if they exist (per the v2.1 stub-error pattern documented in CLAUDE.md §"v2.1 additions").

SC-4 and the zero-panic gate must remain green through both follow-up phases.

## Self-Check: PASSED

- All claimed files exist (`hp41-core/tests/phase20_math.rs`, the 8 modified sources, `.planning/ROADMAP.md` correction).
- All 7 commits present on `develop`: `8064250`, `ae0c1eb`, `5fd4e5f`, `0512c5c`, `8b90b58`, `fdc6b89`, `537704d`.
- `just ci` green; `cargo llvm-cov --fail-under-lines 92.5 -p hp41-core` exits 0 at 92.65%; numerical_accuracy 500/500.
- `grep -E "Op::(Pi|Rup|Rnd|Frc|Abs|Sign|Fact|Mod|PolarToRect|RectToPolar) =>"` returns 10 matches in `ops/mod.rs`, 10 in `program.rs`, 10 in `hp41-cli/src/prgm_display.rs`, 10 in `hp41-gui/src-tauri/src/prgm_display.rs` (4-place rule satisfied).
- SC-4 invariant grep under `hp41-gui/src-tauri/src/` returns no calculator-logic matches.
