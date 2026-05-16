# Phase 27: Test Hardening — Research

**Researched:** 2026-05-15
**Domain:** Rust test hardening (coverage + proptest + integration suites) + Tauri 2.11 E2E + Vitest CI gating
**Confidence:** HIGH for coverage/proptest/IND/Vitest; **MEDIUM with one BLOCKER** for the Playwright/tauri-driver decision (see Open Question 1)

## Summary

Phase 27 is purely test + gate + CI work — no `Op` variants, no `CalcState` fields, no production source edits. The fresh `cargo llvm-cov -p hp41-core` baseline (measured 2026-05-15 during this research run) is **93.59 % lines / 91.21 % regions / 97.42 % functions**, identical to the CONTEXT.md baseline. The gap to the FN-QUAL-01 ≥ 95 % gate is **1.41 percentage points = 54 additional covered lines** out of 247 currently uncovered (3853 covered / 3853+247 = 4100 total). That gap is fully closable by exercising the **uncovered `execute_op` arms in `ops/program.rs`** (Phases 20–24 ops that have unit-test coverage via interactive `dispatch` but no `run_program` coverage) and the **fail-closed SIZE-shrink guards in `ops/stats.rs`** — both high-value bug-catching test additions, not coverage padding.

The flag-semantics proptest, IND integration suite, and Vitest CI gating are straightforward additions matching existing patterns (`proptest_stack.rs` style precedent; `phase21_flags.rs`/`phase24_ind_variants.rs` complement; one-line `justfile` edit). The numerical-accuracy extension is well-defined by D-27.5/D-27.7.

**Primary recommendation:** Confirm Open Question 1 (Playwright/tauri-driver protocol mismatch) before planning. Then proceed with 4 atomic plans: (1) coverage push + accuracy extension, (2) flag proptest + math proptest, (3) IND integration suite, (4) Playwright E2E + Vitest CI gating. Estimated 3–5 Op-free commits per plan; total Phase 27 ≈ 15–20 commits.

**Critical finding (BLOCKER for D-27.15):** `tauri-driver` 2.0.6 speaks the **WebDriver classic protocol** (Selenium/WebdriverIO clients). **Playwright does not speak WebDriver classic** — it uses CDP/native protocols only. The official Tauri E2E examples use **WebdriverIO**, not Playwright. Three viable options exist; the decision needs user confirmation. See Open Question 1.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Coverage gate enforcement | Build tooling (`justfile` + `cargo-llvm-cov`) | CI YAML | Gate is a `just coverage` recipe; `ci.yml` calls `just ci` which calls it. Existing pattern. |
| `hp41-core` unit tests | hp41-core test target | — | All new test files land in `hp41-core/tests/` — out-of-crate integration tests. |
| Flag-semantics proptests | hp41-core test target | — | `proptest_flags.rs` lives alongside `proptest_stack.rs`. |
| IND happy/sad paths | hp41-core test target | — | `indirect_addressing.rs` complements `phase24_ind_variants.rs` (existing example tests). |
| GUI E2E smoke (production binary) | hp41-gui test target | CI YAML | Depends on `just gui-build` → release binary → driver → test runner. |
| Vitest CI gating | `justfile` `gui-ci:` recipe | `ci-gui.yml` | One-line `npm test` append; CI calls `just gui-ci` unchanged. |

## Standard Stack

### Core (already present in workspace — no additions required for coverage / proptest / IND work)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cargo-llvm-cov` | latest stable | Line/region/function coverage measurement | Workspace-supported tool used by `just coverage`; `.cargo/config.toml` already wired. [VERIFIED: workspace baseline `93.59 % lines` produced by this run] |
| `proptest` | `1.11` | Property-based testing | Already declared in `hp41-core/Cargo.toml:15`. `ProptestConfig::with_cases(N)` for per-block iteration counts. [VERIFIED: `grep` in workspace + crates.io] |
| `serde_json` | (workspace dep) | Save-load roundtrip in proptest | Existing dep for persistence. |
| `rust_decimal` | `1.42` | `Decimal` strategy generators | Existing dep. |

### Supporting (new additions for the Playwright job — choice depends on Open Question 1)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@playwright/test` | `1.60.0` (latest, verified 2026-05-15 via `npm view`) | E2E test runner | If Open Question 1 resolves to **Option A** (Playwright `webkit` project + Vite dev server + IPC mocks — fast, no native binary, but does NOT exercise the real Tauri IPC layer). |
| `webdriverio` + `@wdio/cli` + `@wdio/local-runner` + `@wdio/mocha-framework` + `@wdio/spec-reporter` | `^9.19` (per Tauri 2 official example) | WebDriver client | If Open Question 1 resolves to **Option B** (WebdriverIO + tauri-driver — the official Tauri v2 pattern; tests the real production binary). |
| `tauri-driver` (cargo crate, NOT npm) | `2.0.6` | WebDriver server bridging `WebKitWebDriver` ↔ Tauri | Installed via `cargo install tauri-driver --locked`. Listens on `127.0.0.1:4444`. Both Options B and C require this. [CITED: https://v2.tauri.app/develop/tests/webdriver/, crates.io] |
| `tauri-plugin-playwright` (cargo crate) | `0.1.0` (experimental, single-author crate) | Socket bridge plugin embedded in Tauri app | If Open Question 1 resolves to **Option C** — true E2E with Playwright API on the real webview. NOT recommended for Phase 27 (early-stage, single-maintainer, adds a Tauri plugin which contradicts "no source changes to `hp41-gui/src-tauri/`"). [CITED: https://crates.io/crates/tauri-plugin-playwright/0.1.0, https://github.com/srsholmes/tauri-playwright] |

### Alternatives Considered (Playwright job)

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| WebdriverIO (Option B) | Selenium WebDriver | Both supported by `tauri-driver`. WebdriverIO is the Tauri docs' primary example and has a smaller install footprint. [CITED: https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/] |
| Playwright + Vite dev server (Option A) | Playwright + production build | The dev server path skips the IPC layer (uses `@tauri-apps/api/mocks` instead of real `invoke()`), which is the ROADMAP-stated point of E2E. CONTEXT.md D-27.15 explicitly rejects "dev server via Vite" for this reason. |

**Installation (Option B — recommended by this research):**
```bash
# system deps (already in ci-gui.yml for libwebkit2gtk-4.1-dev; ADD webkit2gtk-driver):
sudo apt-get install -y webkit2gtk-driver
# tauri-driver (cargo binary, cached via Swatinem):
cargo install tauri-driver --locked
# npm deps (in hp41-gui/):
npm install -D webdriverio @wdio/cli @wdio/local-runner @wdio/mocha-framework @wdio/spec-reporter
```

**Version verification (performed during this research):**
- `proptest`: workspace pins `1.11`; latest on crates.io is `1.11.x` (current major series). No upgrade needed. [VERIFIED: `grep proptest hp41-core/Cargo.toml`]
- `tauri-driver`: `2.0.6` latest on crates.io. Compatible with Tauri `2.11` (workspace pin). [VERIFIED: `cargo search tauri-driver`]
- `@playwright/test`: `1.60.0` latest on npm. [VERIFIED: `npm view @playwright/test version`]
- `cargo-llvm-cov`: already installed and used by existing `just coverage` recipe — no version pin needed.

## Architecture Patterns

### System Architecture Diagram

```
                         ┌─────────────────────┐
                         │ just coverage       │  ← raised --fail-under-lines 80 → 95
                         │ (cargo llvm-cov)    │
                         └─────────┬───────────┘
                                   │
                          measures │
                                   ▼
       ┌──────────────────────────────────────────────────────┐
       │ hp41-core/tests/   (16+ existing files)              │
       │  ├─ numerical_accuracy.rs  (500 → ~600 cases)        │
       │  ├─ proptest_stack.rs      (existing, untouched)     │
       │  ├─ proptest_flags.rs      (NEW — FN-QUAL-03)        │
       │  ├─ proptest_math.rs       (NEW — FN-QUAL-02 shape)  │
       │  ├─ indirect_addressing.rs (NEW — FN-QUAL-04)        │
       │  └─ phase*_*.rs            (Phase 20–24 example      │
       │                             tests — unchanged)        │
       └──────────────────────────────────────────────────────┘
                                   │
                  exercises        │
                                   ▼
       ┌──────────────────────────────────────────────────────┐
       │ hp41-core/src/   (production code — NO EDITS in P27) │
       │  ├─ ops/program.rs   ← target: execute_op arms       │
       │  ├─ ops/stats.rs     ← target: SIZE-shrink guards    │
       │  ├─ ops/mod.rs       ← target: GETKEY / synthetic    │
       │  ├─ ops/registers.rs ← already 98.34 % (low-priority)│
       │  ├─ ops/math.rs      ← target: dead-code helpers     │
       │  └─ format.rs        ← target: SCI/ENG zero modes    │
       └──────────────────────────────────────────────────────┘

       ┌──────────────────────────────────────────────────────┐
       │ hp41-gui/  (Vitest unit + new Playwright/WebdriverIO)│
       │  ├─ src/*.test.tsx         (existing, 5 files,       │
       │  │                          NOT gated on CI today)   │
       │  └─ e2e/smoke.spec.ts      (NEW — FN-QUAL-05)        │
       └─────────────┬──────────────────┬─────────────────────┘
                     │                  │
            npm test │                  │ wdio.conf.js
              (NEW   │                  │ (NEW — IF Option B)
            gate via │                  │
              just   │                  │     ┌──────────────────┐
            gui-ci)  │                  └────►│ tauri-driver     │
                     │                        │ 127.0.0.1:4444   │
                     │                        └────────┬─────────┘
                     ▼                                 │
       ┌──────────────────────────────────────────────────────┐
       │ ci-gui.yml  (3-OS matrix unchanged)                  │
       │  Linux job:                                           │
       │   1. apt: webkit2gtk-driver + libwebkit2gtk-4.1-dev  │
       │   2. cargo install tauri-driver                       │
       │   3. just gui-build  (release binary)                 │
       │   4. just gui-e2e    (NEW — runs wdio against binary)│
       │  (other OS jobs: unchanged — Playwright Linux-only)  │
       └──────────────────────────────────────────────────────┘
```

### Component Responsibilities

| File | Responsibility | Touched in Phase 27 |
|------|----------------|---------------------|
| `hp41-core/tests/numerical_accuracy.rs` | Hand-curated + per-op shape verification cases | EXTEND — append ~70–105 v2.2 cases |
| `hp41-core/tests/proptest_stack.rs` | Phase 1 stack-lift invariants | UNCHANGED (style precedent only) |
| `hp41-core/tests/proptest_flags.rs` | NEW — flag invariants + skip semantics + IND-flag property | CREATE |
| `hp41-core/tests/proptest_math.rs` | NEW — math shape invariants (FRC, MOD, RND, P→R/R→P, FACT) | CREATE |
| `hp41-core/tests/indirect_addressing.rs` | NEW — happy + non-integer sentinel per `_IND` op | CREATE |
| `hp41-gui/e2e/smoke.spec.ts` (or `wdio.conf.js`+`tests/specs/smoke.spec.ts`) | NEW — `2 ENTER 3 + → 5.0000` E2E | CREATE (depends on Open Question 1) |
| `justfile` | `coverage:` 80→95; `gui-ci:` append `npm test`; new `gui-e2e:` recipe | EDIT |
| `.github/workflows/ci-gui.yml` | Add Playwright/WDIO job (Ubuntu only) | EDIT |
| `hp41-gui/package.json` | Add E2E devDeps | EDIT |
| `CLAUDE.md` | Update Quality Gates table; add Phase 27 settled-architecture block | EDIT |

### Pattern 1: per-block `ProptestConfig` for iteration tuning

```rust
// Source: https://altsysrq.github.io/proptest-book/proptest/tutorial/config.html
// Pattern: ProptestConfig::with_cases(N) for per-test override.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    #[test]
    fn sf_followed_by_fs_q_is_true(n in 0u8..56) {
        let mut s = CalcState::new();
        dispatch(&mut s, Op::SfFlag(n)).unwrap();
        prop_assert!(hp41_core::ops::flags::flag_get(s.flags, n));
    }
}

// Separate proptest! block for math (256 cases — slower)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn rnd_is_idempotent(d in arb_hp_decimal(), digits in 0u8..=9) {
        let mut s = CalcState::new();
        s.display_mode = DisplayMode::Fix(digits);
        s.stack.x = HpNum::from(d);
        dispatch(&mut s, Op::Rnd).unwrap();
        let after_first = s.stack.x.clone();
        dispatch(&mut s, Op::Rnd).unwrap();
        prop_assert_eq!(after_first, s.stack.x);
    }
}
```

### Pattern 2: HP-41 `Decimal` strategy generator (FN-QUAL-02 shape invariants)

```rust
use proptest::prelude::*;
use rust_decimal::Decimal;

/// Generate a Decimal within HP-41's representable range:
/// mantissa = (a, b) with 0 <= a <= 9, 0 <= b < 10^10
/// exponent in -99..=99
fn arb_hp_decimal() -> impl Strategy<Value = Decimal> {
    (
        any::<bool>(),               // sign
        1u64..10_000_000_000u64,     // mantissa (10 digits)
        -99i32..=99i32,              // exponent
    )
        .prop_map(|(neg, mantissa, exp)| {
            let mut d = Decimal::from(mantissa);
            d.set_sign_negative(neg);
            // scale by 10^exp via repeated * Decimal::TEN (or pow10 helper)
            if exp >= 0 {
                d * Decimal::from(10i64.pow(exp.min(18) as u32))
            } else {
                d / Decimal::from(10i64.pow((-exp).min(18) as u32))
            }
        })
}
```

### Pattern 3: conditional-skip sentinel proptest (D-27.10)

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// Sentinel: for any flag n and any starting bool state, FS? skips next step iff flag clear.
    #[test]
    fn fs_q_skip_semantics_match_truth_table(
        n in 0u8..56,
        flag_set in any::<bool>(),
        a in 1i32..100,
        b in 1i32..100,
    ) {
        let mut s = CalcState::new();
        // arrange: set or clear the flag explicitly
        if flag_set {
            dispatch(&mut s, Op::SfFlag(n)).unwrap();
        } else {
            dispatch(&mut s, Op::CfFlag(n)).unwrap();
        }
        s.program = vec![
            Op::Lbl("T".into()),
            Op::FlagTest { kind: FlagTestKind::IsSet, flag: n },
            Op::PushNum(HpNum::from(a)),  // executed iff flag SET
            Op::PushNum(HpNum::from(b)),  // always executed
            Op::Rtn,
        ];
        run_program(&mut s, "T").unwrap();
        // when flag SET: X=b, Y=a; when flag CLEAR: X=b, Y=0 (initial)
        prop_assert_eq!(s.stack.x.inner(), Decimal::from(b));
        let expected_y = if flag_set { Decimal::from(a) } else { Decimal::ZERO };
        prop_assert_eq!(s.stack.y.inner(), expected_y);
    }
}
```

### Pattern 4: save-load roundtrip (D-27.9 item 4)

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    #[test]
    fn flag_state_round_trips_through_serde(flag_pattern: u64) {
        let mut s = CalcState::new();
        s.flags = flag_pattern;
        let json = serde_json::to_string(&s).unwrap();
        let restored: CalcState = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(restored.flags, flag_pattern);
        // print_buffer / event_buffer are #[serde(skip)] — do NOT assert on them
    }
}
```

### Pattern 5: WebdriverIO + tauri-driver E2E (Option B — recommended by this research)

```javascript
// Source: https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/
// hp41-gui/wdio.conf.js
const os = require('os');
const path = require('path');
const { spawn, spawnSync } = require('child_process');

let tauriDriver;

exports.config = {
  specs: ['./tests/specs/**/*.js'],
  maxInstances: 1,
  capabilities: [{
    maxInstances: 1,
    'tauri:options': {
      application: '../src-tauri/target/release/hp41-gui',  // production build
    },
  }],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: { ui: 'bdd', timeout: 60000 },

  // Build release binary before tests start
  onPrepare: () => spawnSync('cargo', ['build', '--release', '--manifest-path', '../src-tauri/Cargo.toml']),

  // Spawn tauri-driver
  beforeSession: () => {
    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] }
    );
  },
  afterSession: () => tauriDriver.kill(),
};
```

```javascript
// hp41-gui/tests/specs/smoke.spec.js
describe('HP-41 GUI smoke', () => {
  it('2 ENTER 3 + displays 5.0000', async () => {
    // SVG keys carry data-key-id — confirmed in hp41-gui/src/Keyboard.tsx:285,303
    await $('[data-key-id="2"]').click();
    await $('[data-key-id="enter"]').click();
    await $('[data-key-id="3"]').click();
    await $('[data-key-id="plus"]').click();
    // Display 14-seg renders via Display14Seg.tsx; LCD container needs a data-testid hook
    const display = await $('[data-testid="lcd-display"]');
    await expect(display).toHaveText('5.0000');
  });
});
```

**Note:** The `data-testid="lcd-display"` selector requires a **one-line attribute addition** to `Display14Seg.tsx`. This is a TS/JSX edit only — NOT a `hp41-gui/src-tauri/` source change — so SC-4 invariant and CONTEXT.md "no source changes to `hp41-gui/src-tauri/`" are preserved. CONTEXT.md does not prohibit `hp41-gui/src/` edits.

### Anti-Patterns to Avoid

- **Padding-only tests to hit 95 %.** D-27.3 explicitly forbids. Examples to avoid: tests that exercise `Debug` impls, `#[derive(Clone)]` paths, or `panic!()` branches gated by `if false`. The risk-weighted target list below names only paths that catch real bug classes.
- **Asserting on `print_buffer` or `event_buffer` in save-load roundtrip.** Both are `#[serde(skip)]` — restored value is always empty `Vec`. The proptest must compare only persisted fields.
- **`assert_eq!` on `Decimal` after f64 conversion in math shape proptests.** Use `<=` tolerance comparisons (HP-41 10-digit rounding compounds across ops). Pattern from existing `numerical_accuracy.rs::passes_with_tol`.
- **`proptest! { #![proptest_config = ...] }` syntax with a const.** Only `ProptestConfig::with_cases(N)` literal-call form works inside the attribute. [CITED: proptest book, configuring proptest]
- **Running Playwright on macOS/Windows runners.** ROADMAP cross-cutting and D-27.15 explicitly forbid. The `ci-gui.yml` job key must include `if: matrix.os == 'ubuntu-latest'`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WebDriver protocol implementation | Custom socket client | `tauri-driver` 2.0.6 + WebdriverIO 9.x | Official Tauri pattern; handles `WebKitWebDriver` startup, capabilities negotiation, session lifecycle. |
| HP-41 input strategy generation | Custom `Arbitrary` impls | `proptest::strategy` combinators + `prop_oneof!` | Existing `proptest_stack.rs::arb_simple_op()` precedent; shrink strategies built-in. |
| Free42 fixture extraction | Python wrapper around Free42 binary | Cite Free42 source line in test doc comment | D-27.7 explicit decision — auditable middle ground, no build dependency, no license complexity. |
| Coverage exclusion of cosmetic code | `#[cfg(not(coverage))]` annotations | Honest ceiling adjustment per D-27.3 | The risk-weighted push should reveal what's left; only annotate AFTER coverage push if the residue is genuinely cosmetic. |
| GUI coverage measurement | Build a coverage gate for hp41-gui | Measure-only snapshot, record in SUMMARY | D-27.4 explicit. |

**Key insight:** Phase 27 is a **test-discipline** phase, not a tooling-build phase. Every "should we add tool X?" question has a "no — existing pattern handles it" answer. The only new tool decision is the Playwright/WDIO/tauri-driver call (Open Question 1).

## Runtime State Inventory

Phase 27 is a test-addition phase, not a refactor — runtime state inventory does not apply. **Verified:** no rename, no string-replace, no migration. Save-file format unchanged (D-27.9 item 4 PROPTESTS the existing `#[serde(default)]` invariant, does not change it). The `justfile` `coverage:` recipe edit changes a numeric threshold but does not rename any tool, recipe, or path.

## Risk-Weighted Uncovered-Line Inventory (FRESH MEASUREMENT 2026-05-15)

This inventory was produced during this research session by running:
```bash
cargo llvm-cov clean --workspace
cargo llvm-cov -p hp41-core --text --output-path /tmp/hp41-cov.txt
```
The numbers below match the CONTEXT.md baseline (`93.59 % lines / 91.21 % regions / 97.42 % functions`).

### Priority 1: `ops/program.rs` (85.89 % regions, 89.58 % lines — single largest file, 1793 regions)

| Uncovered Lines | Function/Region | Bug class caught by adding tests |
|-----------------|----|------|
| 49–53 | `op_gto` "running branch" (`state.is_running == true → find_label → set pc`) | `op_gto` is only reached via `run_loop`'s direct match arm. The is_running guard branch in this function is dead code unless someone bypasses run_loop. **Defensible to skip** (truly unreachable in current control flow — candidate for `#[allow(dead_code)]` annotation per D-27.3 if needed). |
| 77, 80, 88 | `op_xeq` interactive-running guard + `op_rtn` empty call_stack branch | Interactive-running guards similar to GTO. Same defensible-skip case. |
| 334 | `op_catalog` defensive `_ => InvalidOp` after guarded `match n` | Defensive arm; n is already validated above. Defensible-skip OR add `Op::Catalog(99)` defensive test. |
| **647, 650, 661–665, 670, 682–683, 714, 726–737, 749–784, 806–851** | **`execute_op` arms for Phase 20–24 ops** — Rup, Pi, Rnd, Frc, Abs, Sign, Fact, Mod, PolarToRect, RectToPolar, StoArithStack, SigmaPlus/Minus, Mean, Sdev, LR, Yhat, Corr, ClSigmaStat, HmsToH, HToHms, HmsAdd/Sub, PRX/PRA/PRSTK, StoM/N/O, RclM/N/O, SyntheticByte, Wdta/Rdta/Wprgm/Rdprgm, SfFlag, CfFlag, View, AView, Aon, Aoff, Cld, Beep, Tone, Pse, Size, Cla, Clst, Pack, Catalog, Asn, Arcl, Asto, Atox, Xtoa, Arot, Posa, StoInd, RclInd, StoArithInd, SfFlagInd, CfFlagInd, ArclInd, AstoInd, ViewInd, IsgInd, DseInd | **~120 uncovered lines = bulk of the gap.** Each Phase 20–24 op has unit-test coverage via `dispatch()` (interactive path through `ops/mod.rs::dispatch`) but no `run_program` execution test. **Bug class:** any divergence between interactive and program-context execution (lift effects, side-channel writes to `print_buffer`/`event_buffer`/`display_override`, `pc` advancement). The Pitfall-3 cluster from Phase 22 (PSE display_override survival across run_loop iterations) is exactly this class. **Recommended:** one new `hp41-core/tests/program_execution_coverage.rs` that runs a short program containing each Phase 20–24 op and asserts post-state matches the interactive-dispatch baseline. ~30 tests × ~3 assertions each = ~90 covered lines, likely closes the bulk of the gap. |
| 869 | Programming-ops catch-all `Err(HpError::InvalidOp)` arm — "should never reach here" defensive | Test by constructing a `state.is_running = false` then directly calling `execute_op` with `Op::Lbl(_)` etc. **Defensible-skip** (control-flow says unreachable). |
| 950–954 | `find_label_in_state` — never called in current code (op_gto interactive branch is dead) | Same dead-path as 49–53. Defensible-skip. |

**Estimated covered lines from Priority 1 work:** ~80–100 lines.

### Priority 2: `ops/stats.rs` (84.04 % lines — lowest module)

| Uncovered Lines | Function | Bug class caught |
|-----------------|----------|------|
| 62 | `op_sigma_minus` SIZE-shrink guard: `if state.regs.len() < 7 → InvalidOp` | **Pitfall 5 regression sentinel.** Test: `SIZE 003` → `Σ−` → assert `InvalidOp`, state unchanged. |
| 93 | `op_mean` SIZE-shrink guard (same shape) | Same. Test: `SIZE 003` → `MEAN` → assert `InvalidOp`. |
| 120 | `op_sdev` SIZE-shrink guard | Same. |
| 158 | `op_lr` SIZE-shrink guard | Same. |
| 173 | `op_lr` denom-zero branch (all x values identical) | Test: 2 data points with identical x → `LR` → assert `InvalidOp`. |
| 206 | `op_yhat` SIZE-shrink guard | Same. |
| 210 | `op_yhat` n=0 guard | Test: empty Σ registers (cleared) → `YHAT` → assert `InvalidOp`. |
| 221 | `op_yhat` denom-zero | Same as 173. |
| 245 | `op_corr` SIZE-shrink guard | Same. |
| 249 | `op_corr` n=0 guard | Same as 210. |
| 279 | `op_cl_sigma_stat` SIZE-shrink guard | Same. |

**Estimated covered lines:** ~11 lines (all single-line guards). **Bug-catching value:** **highest in Phase 27** — these are explicit "fail-closed" guards that catch the SIZE-shrink-panic class of bugs (Pitfall 5 from Phase 22). One test file `tests/phase22_stats_size_shrink.rs` with ~10 tests covers all of them.

### Priority 3: `ops/mod.rs` (88.84 % regions, 91.50 % lines)

| Uncovered Lines | Function | Bug class caught |
|-----------------|----|------|
| 671, 726, 731 | Phase 21 `Op::FlagTest`/Stop/Pse interactive no-op arms (read-only result + lift apply) | Interactive dispatch of FlagTest is a Neutral no-op — assert pc unchanged, flags unchanged, lift unchanged. **High value:** documents the design invariant. |
| 740, 741, 743, 746, 749, 751, 753 | Phase 22 `Op::Pse`/`Stop`/`GtoInd`/`XeqInd` interactive paths | Interactive Pse formats display_override + pushes "PAUSE 1000"; assert both. Interactive GtoInd/XeqInd return InvalidOp; assert. |
| 811 | `Op::Prompt` interactive arm | `op_prompt` exists in `display_ops` but not run via interactive dispatch in tests. Add one test. |
| 822–839 | Phase 22 interactive `Op::Stop`, `Op::Pse`, `Op::GtoInd`/`XeqInd` arms (subset of above range) | Same as 740–753. |
| 911–912 | `Op::IsgInd` / `Op::DseInd` interactive `.map(\|_\| ())` arms (discard bool skip signal) | Phase 24 interactive IND ISG/DSE tests; the program-context counterparts are tested in `phase24_ind_variants.rs`. |

**Estimated covered lines:** ~20 lines. One new `tests/phase21_phase22_interactive_no_ops.rs` covers them.

### Priority 4: `ops/registers.rs` (98.34 % — already very high)

**No uncovered lines reported in the priority extraction.** This module is fully covered for line metric. The 7 uncovered regions are inside multi-arm match expressions where one arm wasn't taken. Defensible-skip — squeezing this last 1.66 % is exactly the cosmetic-padding D-27.3 forbids.

### Priority 5: `ops/math.rs` (91.28 % regions, 93.45 % lines)

| Uncovered Lines | Function | Bug class caught |
|-----------------|----|------|
| 25–28, 31–34 | `pi_over_180()` / `pi_over_200()` constants — called only by `to_radians_hpnum` which is `#[allow(dead_code)]` | These are pre-computed constants for a deferred HpNum-path trig implementation. **Defensible-skip** (intentionally dead per the `#[allow(dead_code)]` annotation). |
| 41–44 | `to_radians_hpnum` body (also dead_code) | Same. |
| 414 | `op_abs` positive-branch `state.stack.x.clone()` (returned when x is positive) | Existing tests exercise negative inputs only. Add one positive-input ABS test. **Trivially worth it.** |

**Estimated covered lines:** 1 line. Mostly defensible-skip — the dead `to_radians_hpnum` helper is a deliberate forward-looking artifact.

### Priority 6: `format.rs` (78.47 % lines, 79.77 % regions — second-lowest module by lines, but most paths are SCI/ENG zero-mode edges)

| Uncovered Lines | Function | Bug class caught |
|-----------------|----|------|
| 60 | `round_to_display_precision` Eng arm (calls `round_eng`) | RND ENG mode: `5.7 RND` with ENG 1 active. Existing FIX/SCI tests; add ENG. |
| 73–92 | `round_eng` body — engineering carry-handling | Existing `format_eng` tests; add `round_to_display_precision(_, &DisplayMode::Eng(3))` direct tests. |
| 148, 188–192 | `format_sci`/`format_eng` zero-mode `digits == 0` early returns | Test: `0.0 ENTER FmtSci(0)` and `0.0 ENTER FmtEng(0)` → assert display string. |
| 216–218 | `format_eng` carry threshold crossing (mantissa rounded ≥ carry_threshold → new eng_exp) | Edge case: 999.9995 in ENG(3) → 1.000E+3. |
| 247 | `decimal_pow10` `exp == 0 → Decimal::ONE` early return | Tested via callers but not directly — add one direct test for completeness. |

**Estimated covered lines:** ~15–20 lines. One new `tests/format_eng_edges.rs` with ~10 tests covers them. **Bug class:** display-mode rounding boundaries (a documented HP-41 hardware quirk class).

### Gap-closure summary (estimated additions to reach ≥ 95 %)

| Source | Estimated covered lines |
|--------|-------------------------|
| Priority 1 (program-execution coverage) | +80–100 |
| Priority 2 (stats SIZE-shrink guards) | +11 |
| Priority 3 (interactive no-op arms) | +20 |
| Priority 5 (op_abs positive branch) | +1 |
| Priority 6 (format SCI/ENG zero / ENG carry) | +15–20 |
| **Total estimated** | **+127–152 (out of 247 uncovered)** |

**Target:** 247 → ~95–120 uncovered = **95.7–97.6 % lines**. Comfortably above the 95.0 % gate. The defensible-skip lines (49–53, 77–88, 869, 950–954, math.rs dead_code) account for ~25 lines that would push the *theoretical maximum* close to 99 %; D-27.3 ceiling-fallback authorises stopping at the achievable practical maximum rather than padding to those last cosmetic lines.

## Common Pitfalls

### Pitfall 1: `proptest!` seeds nondeterministic in CI

**What goes wrong:** A property holds 999/1000 cases, fails on seed 0x4242. The local re-run with the same seed reproduces, but a fresh CI run uses a different seed and passes — masking the bug.

**Why it happens:** `proptest` uses time-based seeding unless `PROPTEST_PERSIST_FILE` is set. CI runs that fail leave a persisted seed file in `proptest-regressions/` — but only if the test runner has write access to the repo.

**How to avoid:** `proptest` writes to `proptest-regressions/<file>.txt` automatically on failure. Commit the directory: add `proptest-regressions/` to the repo (NOT `.gitignore`). Future CI runs replay persisted failing seeds before exploring new ones. [CITED: proptest book, persistence]

**Warning signs:** Flaky test that fails 1-in-1024 times. Check if `proptest-regressions/` is gitignored — if so, persistence is broken.

### Pitfall 2: `cargo llvm-cov` region coverage lags lines for match arms

**What goes wrong:** Adding tests to increase line coverage doesn't move region coverage. The 95 % gate is on lines (per CONTEXT.md), but if regions stay at ~91 %, future drift hits the line metric harder.

**Why it happens:** A single line `Op::Foo => bar(),` is one region (the match arm) but one line. If `bar()` panics on a particular input, the line is covered (the arm fired) but the region isn't (the panic-after-call branch wasn't taken). LLVM's region model counts MIR branches, not source lines.

**How to avoid:** Phase 27's gate target is **lines only** (per CONTEXT.md D-27.2). Don't chase regions. If a planner finds themselves writing low-signal tests to lift regions, stop and check the metric.

**Warning signs:** "I added 5 tests and lines went up by 0.2 % but regions didn't move" — that's normal, not a problem.

### Pitfall 3: Vitest install missing from `gui-ci` recipe

**What goes wrong:** Adding `npm test` to `just gui-ci` fails on a fresh CI runner because `vitest` isn't installed.

**Why it happens:** The existing `gui-ci` recipe (justfile line 84) already runs `npm install` before `npx tsc --noEmit`, so this is already handled. **Verified.**

**How to avoid:** No action needed. The recipe is correct; the planner just appends `cd hp41-gui && npm test` (or shorter: `npm test` if working dir is right) as line 88.

**Warning signs:** N/A — sequence is already correct.

### Pitfall 4: Playwright `webkit` mismatch with system WebKitGTK (only if Option A picked)

**What goes wrong:** Playwright's bundled `webkit` browser differs from Linux's `WebKitGTK 2.x`. A test passes against Playwright's webkit but the same selector fails against the production Tauri binary using `WebKitGTK`.

**Why it happens:** Playwright maintains a custom WebKit fork. It's closer to Tauri's runtime than Chromium, but not bit-identical. [CITED: https://takazudomodular.com/pj/zudo-tauri/docs/frontend/playwright-engine-pitfall/]

**How to avoid:** Skip Option A. If Option A is taken anyway, treat the smoke spec as "smoke at the React layer", not "smoke at the Tauri runtime" — the value proposition shifts. The planner needs to record this explicitly.

**Warning signs:** Tests pass in CI but a real Tauri build behaves differently.

### Pitfall 5: `tauri-driver` PATH on GitHub Ubuntu runner

**What goes wrong:** `cargo install tauri-driver` in CI installs to `~/.cargo/bin/tauri-driver`. The `Swatinem/rust-cache@v2` action caches `target/` but **does not cache the cargo bin directory by default**. Every CI run reinstalls (slow).

**Why it happens:** The cache action targets the workspace `target/`. Cargo-installed binaries land elsewhere.

**How to avoid:** Either (a) cache `~/.cargo/bin/` with `actions/cache@v4`, or (b) use a pre-built `tauri-driver` action (none currently exists on the marketplace). The CONTEXT.md "3–5 min runtime budget" allows for one fresh install (~30s), so option (a) is optimization-only.

**Warning signs:** Playwright job consistently takes >5 min — investigate cargo install caching.

### Pitfall 6: `tauri-driver` requires `WebKitWebDriver` binary on PATH

**What goes wrong:** `apt install libwebkit2gtk-4.1-dev` (already in `ci-gui.yml`) provides headers, NOT the WebDriver binary. The Ubuntu CI runner needs `webkit2gtk-driver` as a separate apt package.

**Why it happens:** Debian splits dev headers, runtime libs, and the WebDriver binary into separate packages.

**How to avoid:** Add `webkit2gtk-driver` to the apt-get line in `ci-gui.yml` (it's currently missing). [CITED: https://v2.tauri.app/develop/tests/webdriver/]

**Warning signs:** `tauri-driver` exits with "WebKitWebDriver not found" on first CI run.

### Pitfall 7: Coverage gate raise + tests not in same commit

**What goes wrong:** PR adds tests in commit A, raises gate in commit B. If A is reverted but B isn't, CI fails on develop.

**Why it happens:** D-27.2 mandates atomicity: same commit must add the final test batch AND raise the gate. Phase 27 plan structure must enforce this in the final wave.

**How to avoid:** Plan the gate raise as the LAST action in the LAST plan. Verifier checks "did `justfile` 80→95 change and the last test additions land in the same commit?".

**Warning signs:** Reviewer sees gate-raise commit with no test-addition diff.

### Pitfall 8: ENG / FIX / SCI 0-digit edge cases differ from non-zero

**What goes wrong:** Adding shape-invariant proptest `RND(x, FIX(n))` then idempotent fails when n=0 because `FIX 0` adds a trailing `.` to the display string but doesn't change the underlying value's rounding.

**Why it happens:** `format_fix` line 136 has special-case for `digits == 0`. `round_to_display_precision` Fix arm uses `round_dp_with_strategy(0, ...)` — no display-string concerns, but the proptest should not conflate value rounding with display formatting.

**How to avoid:** Test RND idempotency on the VALUE (via `state.stack.x.inner()`), not the display string.

**Warning signs:** Proptest fails specifically when `digits == 0`.

### Pitfall 9: SVG selector `data-key-id="plus"` vs `data-key-id="+"` confusion

**What goes wrong:** The smoke spec clicks `data-key-id="plus"` but `Keyboard.tsx:111` declares `id: 'plus'`, not `id: '+'`. Test fails: "no element matched selector".

**Why it happens:** Frontend key IDs are kebab-case strings, not glyph characters. Confirmed in research: line 111 shows `{ id: 'plus', label: '+', ... }`.

**How to avoid:** Read `Keyboard.tsx` `KEY_DEFS` before writing the smoke spec selectors. All IDs verified: `'2'`, `'enter'`, `'3'`, `'plus'` are correct.

**Warning signs:** Test fails with "selector matched 0 elements" — wrong ID.

### Pitfall 10: LCD display element missing `data-testid`

**What goes wrong:** Smoke spec selects `[data-testid="lcd-display"]` but `Display14Seg.tsx` doesn't emit that attribute.

**Why it happens:** `Display14Seg.tsx` was authored in Phase 26 without E2E in mind. CONTEXT.md doesn't pre-authorise the attribute add.

**How to avoid:** Phase 27 Plan 04 explicitly adds the one-line `data-testid` attribute to `Display14Seg.tsx`. This is a TS/JSX edit — NOT an `hp41-gui/src-tauri/` source change — so SC-4 invariant is preserved. Document the edit in the plan must_haves.

**Warning signs:** Smoke spec selector returns null for the display element.

## Code Examples

### Example 1: Math shape invariant — RND idempotency

```rust
// Source: derived from existing tests/phase20_math.rs + proptest book examples.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn rnd_is_idempotent_in_all_display_modes(
        d in arb_hp_decimal(),
        digits in 0u8..=9,
        mode in prop_oneof![
            Just(0u8),  // FIX
            Just(1u8),  // SCI
            Just(2u8),  // ENG
        ],
    ) {
        let mut s = CalcState::new();
        s.display_mode = match mode {
            0 => DisplayMode::Fix(digits),
            1 => DisplayMode::Sci(digits),
            _ => DisplayMode::Eng(digits),
        };
        s.stack.x = HpNum::from(d);
        dispatch(&mut s, Op::Rnd).unwrap();
        let after_first = s.stack.x.clone();
        dispatch(&mut s, Op::Rnd).unwrap();
        prop_assert_eq!(after_first.inner(), s.stack.x.inner());
    }
}
```

### Example 2: MOD sign-follows-Y invariant (HP-41 semantics)

```rust
// Source: HP-41 hardware semantics confirmed against
// https://www.hpmuseum.org/forum/thread-9809.html
// "7 MOD 3 = 1, 7 MOD -3 = -2, -7 MOD 3 = 2, -7 MOD -3 = -1"
// HP-41 MOD: result sign follows sign of Y (the dividend, top of stack before X)

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn mod_sign_follows_y(
        y_mag in 1i64..1_000_000,
        x_mag in 1i64..1000,
        y_neg in any::<bool>(),
        x_neg in any::<bool>(),
    ) {
        let y_val = if y_neg { -y_mag } else { y_mag };
        let x_val = if x_neg { -x_mag } else { x_mag };
        let mut s = CalcState::new();
        s.stack.y = HpNum::from(y_val);
        s.stack.x = HpNum::from(x_val);
        dispatch(&mut s, Op::Mod).unwrap();
        let result = s.stack.x.inner();
        if result.is_zero() {
            // zero result is sign-agnostic
        } else {
            prop_assert_eq!(
                result.is_sign_negative(),
                y_neg,
                "MOD({}, {}) sign should follow Y={}",
                y_val, x_val, y_val
            );
        }
    }
}
```

### Example 3: IND happy path + non-integer rejection sentinel

```rust
// Source: derived from existing phase24_ind_variants.rs::sto_ind_happy
// + ROADMAP cross-cutting D-27.12 (one-test-per-IND-op, dedicated file).

// hp41-core/tests/indirect_addressing.rs (NEW)
#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op, StoArithKind, FlagTestKind};
use hp41_core::{CalcState, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Macro: assert happy-path + non-integer-rejection for an IND op via dispatch.
/// (Skip-semantic ops -- IsgInd/DseInd/FlagTestInd -- need run_program; see below.)
macro_rules! ind_happy_and_reject {
    ($name:ident, $op:expr, $setup:expr, $assert_happy:expr) => {
        #[test]
        fn $name() {
            // Happy path
            let mut state = CalcState::new();
            state.regs[5] = HpNum::from(12i32);
            $setup(&mut state);
            dispatch(&mut state, $op).unwrap();
            $assert_happy(&state);
            // Non-integer rejection
            let mut state = CalcState::new();
            state.regs[5] = HpNum::rounded(Decimal::from_str("12.5").unwrap());
            $setup(&mut state);
            let result = dispatch(&mut state, $op);
            assert!(matches!(result, Err(HpError::InvalidOp)));
        }
    };
}

// One macro invocation per IND op — covers FN-QUAL-04 happy + reject.
ind_happy_and_reject!(
    sto_ind_complete,
    Op::StoInd(5),
    |s: &mut CalcState| s.stack.x = HpNum::from(7i32),
    |s: &CalcState| assert_eq!(s.regs[12], HpNum::from(7i32))
);

ind_happy_and_reject!(
    rcl_ind_complete,
    Op::RclInd(5),
    |s: &mut CalcState| s.regs[12] = HpNum::from(99i32),
    |s: &CalcState| assert_eq!(s.stack.x, HpNum::from(99i32))
);
// ... 15 more: StoArithInd, SfFlagInd, CfFlagInd, ArclInd, AstoInd, ViewInd, ...
// IsgInd / DseInd / FlagTestInd happy-path uses run_program (Pitfall 1 from
// phase24_ind_variants.rs) -- not the macro.
```

### Example 4: Hand-curated FACT case (HP-41-specific overflow)

```rust
// Source: HP-41C Owner's Manual: "FACT(n) where n > 69 → OUT OF RANGE error"
// Cross-checked against Free42 core_math.cc (referenced in test doc comment).

// Appended to existing tests/numerical_accuracy.rs:
case!("FACT_0_returns_1", "FACT", {
    let mut s = new_deg_state();
    push(&mut s, "0");
    dispatch(&mut s, Op::Fact).unwrap();
    get_x(&s)
}, 1.0);

case!("FACT_69_max_valid", "FACT", {
    let mut s = new_deg_state();
    push(&mut s, "69");
    dispatch(&mut s, Op::Fact).unwrap();
    get_x(&s)
}, 1.711224524281413e98);  // 69! exact value, fits in HP-41 +99 exponent

#[test]
fn fact_70_returns_out_of_range() {
    // Cross-checked against Free42 core_math.cc::docmd_fact -- returns
    // ERR_OUT_OF_RANGE for n > 69, matching HP-41C Owner's Manual p.234.
    let mut s = CalcState::new();
    let _ = dispatch(&mut s, Op::PushNum(HpNum::from(70i32)));
    let result = dispatch(&mut s, Op::Fact);
    assert!(matches!(result, Err(HpError::OutOfRange)));
}
```

### Example 5: justfile delta (D-27.2 + D-27.14)

```diff
 # Check coverage gate — ≥80% line coverage on hp41-core (clean first to avoid stale profraw data)
 coverage:
 	cargo llvm-cov clean --workspace
-	cargo llvm-cov --fail-under-lines 80 -p hp41-core
+	cargo llvm-cov --fail-under-lines 95 -p hp41-core

 # gui-ci: CI gate — TypeScript type-check, Rust tests, and release build (called from ci-gui.yml)
 gui-ci:
 	cd hp41-gui && npm install
 	cd hp41-gui && npx tsc --noEmit
 	cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml
 	cargo build --release --manifest-path hp41-gui/src-tauri/Cargo.toml
+	cd hp41-gui && npm test
+
+# gui-e2e: WebdriverIO + tauri-driver smoke (Linux only — invoked from ci-gui.yml)
+gui-e2e:
+	cd hp41-gui && npm install
+	cd hp41-gui && npm run tauri build -- --debug --no-bundle
+	cd hp41-gui && npx wdio run wdio.conf.js
```

### Example 6: ci-gui.yml delta (Playwright/WDIO job — Option B shape)

```yaml
# Add as a separate job under jobs: (NOT a step in the existing matrix build)
  e2e:
    name: GUI E2E (Ubuntu only)
    runs-on: ubuntu-latest
    needs: build  # only run if matrix build is green
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target

      - name: Cache cargo bin (tauri-driver)
        uses: actions/cache@v4
        with:
          path: ~/.cargo/bin
          key: cargo-bin-${{ runner.os }}-tauri-driver-2.0.6

      - uses: actions/setup-node@v4
        with: { node-version: 'lts/*' }
      - uses: taiki-e/install-action@v2
        with: { tool: just }

      - name: Install Linux system deps
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev \
            librsvg2-dev patchelf webkit2gtk-driver xvfb

      - name: Install tauri-driver
        run: cargo install tauri-driver --locked --version 2.0.6

      - name: Run E2E smoke under Xvfb
        run: xvfb-run -a just gui-e2e
        env:
          CI: true
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `proptest::test_runner::Config { cases: N, ..Config::default() }` literal | `ProptestConfig::with_cases(N)` builder | proptest 1.0+ | Cleaner; the literal still works but is verbose. |
| Tauri webdriver with Microsoft Edge WebDriver | `tauri-driver` 2.0.6 + `WebKitWebDriver` on Linux | Tauri 2 GA (2024) | Linux runners are standard for headless GUI tests; Edge WebDriver path remains valid for Windows. |
| Vitest mocking via `vi.mock` at top of file | Same — pattern unchanged since vitest 1.0 | — | The `App.test.tsx` precedent stays current. |
| `cargo llvm-cov` `.lcov` output for Codecov upload | Same; nothing changed | — | No tool upgrade needed. |

**Deprecated / outdated:**
- **`tauri-plugin-playwright` 0.1.0** — published Apr 2024 by a single author (`srsholmes`), no updates in 9+ months as of 2026-05. NOT recommended for Phase 27. [CITED: https://crates.io/crates/tauri-plugin-playwright/0.1.0]
- **`mocha`-only proptest test runners** — historic pattern; the workspace already uses Rust's built-in `#[test]` harness which proptest integrates with directly.

## Project Constraints (from CLAUDE.md)

- **`#![deny(clippy::unwrap_used)]` at the `hp41-core` crate root.** All new test files MUST carry `#[allow(clippy::unwrap_used)]` at the file or test mod level (existing pattern in `phase21_flags.rs`, `phase24_ind_variants.rs`, etc.).
- **No `println!` / `eprintln!` in `hp41-core`.** Test assertions on display output use `state.print_buffer` / `state.event_buffer` / `state.display_override`.
- **SC-4 invariant:** Phase 27 touches `hp41-core/tests/`, `hp41-gui/src/` (e.g. `data-testid` attribute on `Display14Seg.tsx`), `hp41-gui/e2e/` or `hp41-gui/tests/specs/` (NEW), `justfile`, `.github/workflows/ci-gui.yml`, `CLAUDE.md`. **NO source changes to `hp41-gui/src-tauri/`** — the planner must avoid any plan that touches Rust code in the GUI crate.
- **MSRV 1.88** — no MSRV change. `tauri-driver` 2.0.6 has MSRV 1.77; `proptest` 1.11 has MSRV 1.65. Both compatible.
- **Commits via `/git-workflow:commit --with-skills`** (per global memory). NEVER `git commit` directly.
- **Op variants land in 4 places** — N/A in Phase 27 (no new ops).
- **`Op` variants land before TUI code** — N/A in Phase 27.
- **Stack-lift declaration per op** — N/A in Phase 27.
- **`#[serde(default)]` on new `CalcState` fields** — N/A; Phase 27 adds no fields. D-27.9 item 4 PROPTESTS this invariant.

## Validation Architecture

> Phase 27 IS the validation work itself — this section maps Phase 27's own requirements to its test scaffolding.

### Test Framework
| Property | Value |
|----------|-------|
| Framework (Rust) | `cargo test` + `proptest 1.11` |
| Framework (GUI unit) | `vitest 4.1.6` |
| Framework (GUI E2E) | WebdriverIO 9.x + tauri-driver 2.0.6 (Option B — pending Open Question 1) |
| Config files | `hp41-gui/vite.config.ts` (vitest section); NEW `hp41-gui/wdio.conf.js` |
| Quick run command (Rust) | `cargo test -p hp41-core` (alias: `just test-core`) |
| Quick run command (Vitest) | `cd hp41-gui && npm test` |
| Quick run command (WDIO) | `just gui-e2e` |
| Full suite command | `just ci && just gui-ci && just gui-e2e` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FN-QUAL-01 | `hp41-core` line coverage ≥ 95 % | gate | `just coverage` | edited (justfile) |
| FN-QUAL-02 | Numerical accuracy ≥ 98 % on new v2.2 cases | unit + proptest | `cargo test -p hp41-core --test numerical_accuracy && cargo test -p hp41-core --test proptest_math` | partial (numerical_accuracy.rs exists, proptest_math.rs new) |
| FN-QUAL-03 | Flag-semantics proptest covers ROADMAP-3 + 4 extensions | proptest | `cargo test -p hp41-core --test proptest_flags` | new |
| FN-QUAL-04 | Every IND op: happy + non-integer rejection | unit | `cargo test -p hp41-core --test indirect_addressing` | new |
| FN-QUAL-05 | Playwright/WDIO smoke `2 ENTER 3 + → 5.0000` | E2E | `just gui-e2e` | new |

### Sampling Rate
- **Per task commit:** `just test-core` (~10s); per change to `hp41-gui/src/`: `cd hp41-gui && npm test` (~5s).
- **Per wave merge:** `just ci` (lint + test + coverage; ~2 min) + `just gui-ci` (~3 min).
- **Phase gate:** Full suite green AND `just coverage` reports ≥ 95.0 % AND Playwright/WDIO job green on Ubuntu CI before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] `hp41-core/tests/proptest_flags.rs` — covers FN-QUAL-03 (NEW)
- [ ] `hp41-core/tests/proptest_math.rs` — covers FN-QUAL-02 shape invariants (NEW)
- [ ] `hp41-core/tests/indirect_addressing.rs` — covers FN-QUAL-04 (NEW)
- [ ] `hp41-core/tests/program_execution_coverage.rs` (suggested name) — covers Priority 1 uncovered execute_op arms (NEW)
- [ ] `hp41-core/tests/phase22_stats_size_shrink.rs` (suggested name) — covers Priority 2 SIZE-shrink guards (NEW)
- [ ] `hp41-core/tests/format_eng_edges.rs` (suggested name) — covers Priority 6 SCI/ENG zero modes (NEW)
- [ ] `hp41-gui/wdio.conf.js` + `hp41-gui/tests/specs/smoke.spec.js` (or Playwright equivalent — pending Open Question 1) — covers FN-QUAL-05 (NEW)
- [ ] `proptest-regressions/` directory (Pitfall 1 mitigation) — add to repo, NOT to `.gitignore`
- [ ] `data-testid="lcd-display"` attribute on `Display14Seg.tsx` (Pitfall 10) — one-line edit
- [ ] `coverage:` recipe edit in `justfile` (D-27.2 atomic with final coverage commit)
- [ ] `gui-ci:` recipe edit appending `npm test` (D-27.14)
- [ ] `ci-gui.yml` Playwright/WDIO job (D-27.15, D-27.16)
- [ ] `CLAUDE.md` Quality Gates table update + Phase 27 settled-architecture block

## Security Domain

> `security_enforcement` not set explicitly in `.planning/config.json` → treat as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Single-user desktop calculator — no authn surface. |
| V3 Session Management | no | No sessions. |
| V4 Access Control | no | Local-only app; no remote access. |
| V5 Input Validation | yes | `serde_json` parse boundary (save-load roundtrip proptest exercises this); `Decimal::from_str` parse boundary (numerical_accuracy.rs case constructors). |
| V6 Cryptography | no | No crypto in this codebase. |
| V11 Business Logic | yes | HP-41 semantics ARE the business logic; proptests directly verify invariants. |
| V13 API & Web Service | partial | Tauri IPC is the only "API" boundary. Phase 27 E2E exercises it end-to-end (Option B). |

### Known Threat Patterns for {Rust core + Tauri 2 GUI}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious save-file triggers panic on load | Denial of Service | `#![deny(clippy::unwrap_used)]` + `#[serde(default)]` on all fields. Phase 27 save-load roundtrip proptest with random `u64` flag patterns is an empirical regression sentinel. |
| Malicious IPC payload via `dispatch_op` | Tampering | `key_map::resolve` returns `GuiError` for unknown ids (D-07 invariant). Phase 27 E2E does NOT directly test this — it's covered by Phase 26 Vitest unit tests. |
| Long ASN labels in user keymap (XSS via SVG `<text>`) | Tampering | React auto-escapes text nodes; `Keyboard.tsx` slices to 7 chars (defense-in-depth). Phase 27 Vitest CI gating will run the Phase 26 XSS test on every push. |
| `cargo install tauri-driver` MITM during CI | Tampering | Use `--locked` flag (recommended in Example 6). Cargo's checksum validation against `Cargo.lock` blocks tampered downloads. |

## Sources

### Primary (HIGH confidence)
- **Workspace files** (read during this research session):
  - `.planning/phases/27-test-hardening/27-CONTEXT.md` — 16 locked decisions
  - `.planning/REQUIREMENTS.md` §FN-QUAL-01..05 lines 103–107
  - `.planning/ROADMAP.md` Phase 27 section (lines 192–207)
  - `.planning/STATE.md` — current milestone state
  - `CLAUDE.md` — settled architecture decisions
  - `hp41-core/Cargo.toml` — verified `proptest = "1.11"`
  - `hp41-gui/src-tauri/Cargo.toml` — verified `tauri = "2.11"`
  - `hp41-gui/package.json` — verified existing devDeps
  - `hp41-gui/src/Keyboard.tsx` — verified `data-key-id` attributes on all keys (lines 285, 303)
  - `hp41-gui/src/App.test.tsx` — verified `vi.mock('@tauri-apps/api/core')` pattern
  - `hp41-gui/vite.config.ts` — verified Vitest setup
  - `justfile` — verified current `coverage:` and `gui-ci:` recipes
  - `.github/workflows/ci-gui.yml` — verified current shape (matrix build, no E2E job)
  - `hp41-core/tests/proptest_stack.rs`, `numerical_accuracy.rs`, `phase21_flags.rs`, `phase24_ind_variants.rs` — verified style precedents

- **Coverage measurement** (run during this session 2026-05-15):
  - `cargo llvm-cov clean --workspace && cargo llvm-cov -p hp41-core --text` → 93.59 % lines / 91.21 % regions / 97.42 % functions
  - Per-file uncovered-line extraction via awk pipeline on `/tmp/hp41-cov.txt`

- **Official Tauri v2 docs:**
  - https://v2.tauri.app/develop/tests/webdriver/ — `tauri-driver` setup, `WebKitWebDriver` on Linux, `webkit2gtk-driver` apt dependency
  - https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/ — full WebdriverIO config (4444 port, capabilities, spawn pattern)
  - https://v2.tauri.app/develop/tests/webdriver/ci/ — CI patterns

- **Official Playwright docs:**
  - https://playwright.dev/docs/test-retries — `retries: 1` config (D-27.16)
  - https://playwright.dev/docs/intro — install patterns

- **proptest book / docs:**
  - https://altsysrq.github.io/proptest-book/proptest/tutorial/config.html — `ProptestConfig::with_cases(N)` builder + `#[proptest_config(...)]` attribute
  - https://docs.rs/proptest/latest/proptest/test_runner/struct.Config.html — Config struct fields
  - https://altsysrq.github.io/proptest-book/proptest/persistence.html — `proptest-regressions/` directory pattern

### Secondary (MEDIUM confidence — verified with at least one official source)
- https://crates.io/crates/tauri-driver — verified version 2.0.6, MSRV check (web-fetched)
- https://www.hpmuseum.org/forum/thread-9809.html — HP-41/HP-42S MOD sign semantics (community reference, cross-referenced with calc behavior)
- https://hp41programs.yolasite.com/factorial.php — HP-41C FACT n>69 OUT OF RANGE (community reference)
- HP-41C Owner's Manual archived at https://archived.hpcalc.org/greendyk/hp41c-manual/81.html — primary HP-41 reference (D-27.7)
- https://thomasokken.com/free42/ — Free42 source-code cross-check reference (D-27.7 cite-target)

### Tertiary (LOW confidence — single web source, flagged for validation)
- https://takazudomodular.com/pj/zudo-tauri/docs/frontend/playwright-engine-pitfall/ — Playwright/Tauri WebKit mismatch (single blog post; reinforced by Playwright's own documented browser model)
- https://github.com/srsholmes/tauri-playwright — `tauri-plugin-playwright` approach (single-author crate; NOT recommended for Phase 27, just listed as Option C for completeness)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `proptest-regressions/` directory should be committed (not gitignored) to make CI seed-failure reproducible | Pitfall 1 | Low — if gitignored, flaky tests cost ~5 min per occurrence to debug; mitigation cost is one `.gitignore` edit. |
| A2 | The 80–100 newly covered lines from Priority 1 (Phase 20–24 program-execution arms) will close most of the gap; the remaining 30–50 lines from Priorities 2/3/5/6 give comfortable headroom over the 95 % gate | Gap-closure summary | Medium — if Priority 1 work yields fewer covered lines than estimated (e.g. the test pattern triggers the same arms an existing test already covers), the gap closure may fall short of 95 %. **Mitigation:** D-27.3 ceiling fallback. Confirm during plan-checker review by running `just coverage` after each plan. |
| A3 | The `data-testid="lcd-display"` attribute add to `Display14Seg.tsx` does NOT count as a `hp41-gui/src-tauri/` source change | Architecture / Pitfall 10 | Low — CONTEXT.md prohibits `hp41-gui/src-tauri/` changes, not `hp41-gui/src/`. Already verified against CONTEXT.md text. |
| A4 | WebdriverIO 9.x is compatible with `tauri-driver` 2.0.6 — the Tauri docs cite 9.19.0 in their example | Standard Stack | Medium — WDIO 9 is current per the official Tauri example fetched in this session. If a breaking change lands in WDIO 10+ before Phase 27 ships, pin to 9.x. |
| A5 | The GitHub `ubuntu-latest` runner (Ubuntu 24.04 as of 2026-05) has X server / Xvfb support out-of-the-box, OR `xvfb-run -a` wrapper is sufficient | Pattern 5 / Example 6 | Medium — Tauri docs don't explicitly state whether Xvfb is required. The example uses `xvfb-run` defensively. If unnecessary, the wrapper is harmless. |
| A6 | The 130 GUI E2E "literal ROADMAP smoke" can complete within the documented "3–5 min on Ubuntu runner" budget | CONTEXT.md D-27.15 | Low — single-spec smoke is well under that budget on reference Tauri 2 example projects. The build step (`cargo build --release`) is the dominant cost, ~2–3 min on cold cache. |
| A7 | HP-41C MOD result-sign-follows-Y semantics is implemented correctly in the current `op_mod` — the shape proptest will pass on the existing implementation, validating rather than uncovering a bug | Code Example 2 | Medium — if `op_mod` accidentally implements `f64`-style sign-follows-X (the Rust `%` operator semantics), the proptest will fail on its first negative-Y case. **Mitigation:** this IS the bug-class the proptest is supposed to catch — failure is the point. The planner should expect potential discovery here. |

**If this table is empty:** N/A — 7 assumptions documented for user / planner / discuss-phase review.

## Open Questions

### 1. **PLAYWRIGHT/TAURI-DRIVER PROTOCOL MISMATCH (BLOCKER for D-27.15)**

   **What we know:**
   - CONTEXT.md D-27.15 specifies: "Playwright launch mode — production build + `tauri-driver` (WebKitGTK on Ubuntu). … connected through Playwright's webdriver protocol."
   - `tauri-driver` 2.0.6 implements the **WebDriver classic protocol** (W3C WebDriver standard) and bridges to `WebKitWebDriver` on Linux.
   - **Playwright does NOT support the WebDriver classic protocol.** Playwright uses CDP (Chrome DevTools Protocol) for Chromium-family browsers and its own custom protocols for Firefox/WebKit. It cannot connect to a WebDriver-classic server like `tauri-driver`.
   - The official Tauri v2 E2E example at https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/ uses **WebdriverIO** (a WebDriver-classic client), not Playwright.

   **What's unclear:** Which of the three viable options the user prefers:

   - **Option A — Playwright + `webkit` project + Vite dev server + IPC mocks**
     - Pros: True Playwright (matches D-27.15 wording). Fast (no native binary). Single tool ecosystem with Phase 26 Vitest unit tests.
     - Cons: Does NOT exercise the real Tauri IPC layer. Uses `@tauri-apps/api/mocks` to stub `invoke()`. **Contradicts the stated intent of D-27.15** ("tests the actual Tauri runtime + IPC layer + React frontend, NOT a mocked Vite dev server with stubbed `invoke()` calls"). The smoke is at the React layer only, similar to App.test.tsx but rendered in a real browser.
     - Cost: ~$0 — just install `@playwright/test`. CI runtime ~30s for a single smoke.

   - **Option B — WebdriverIO + `tauri-driver` + production build (RECOMMENDED by this research)**
     - Pros: Matches the *intent* of D-27.15 (production binary, real IPC). Official Tauri pattern. Stable, well-documented.
     - Cons: Different from "Playwright" — D-27.15 wording must be revisited. WebdriverIO is a separate test ecosystem from Vitest (different syntax, different reporter).
     - Cost: ~5 npm devDeps + `cargo install tauri-driver`. CI runtime ~3–5 min.

   - **Option C — `tauri-plugin-playwright` (community crate)**
     - Pros: True Playwright API on the real Tauri webview.
     - Cons: Experimental (v0.1.0, single-author, last update >9 months). Requires adding a Tauri plugin to `hp41-gui/src-tauri/Cargo.toml` and `tauri.conf.json` — **CONTRADICTS CONTEXT.md "no source changes to `hp41-gui/src-tauri/`"**. NOT recommended.

   **Recommendation:** Take this to `/gsd-discuss-phase 27` (mini iteration) to get user confirmation. The CONTEXT.md text reads "Playwright + tauri-driver" as if compatible — that's a non-trivial misconception that needs clearing before planning locks. Recommended resolution: **Option B (WebdriverIO + tauri-driver)** with a CONTEXT.md amendment to D-27.15 changing "Playwright" → "WebdriverIO".

### 2. Coverage gate ceiling realism (D-27.3)

   **What we know:**
   - Baseline: 247 uncovered lines.
   - Risk-weighted plan covers ~127–152 lines = 95.7–97.6 % achievable.
   - The remaining ~25 "defensible-skip" lines (math.rs dead_code constants, program.rs interactive-running dead branches, find_label_in_state) are below `100 - 247/4100 = 93.97 %` cosmetic floor.

   **What's unclear:** Whether 95.0 % is actually the right gate, or whether the empirical achievable target after Phase 27's risk-weighted push (~96 %) should become the gate.

   **Recommendation:** Plan towards 95.0 %, measure after each plan, and let the verifier ratify or recommend the final gate value. D-27.3 explicitly authorises adjusting down if 95 % requires padding. Document the achieved ceiling in CLAUDE.md.

### 3. `proptest-regressions/` directory commit policy

   **What we know:**
   - Pitfall 1 recommends committing this directory.
   - The workspace `.gitignore` currently does NOT mention it (verified in research session — file's pattern unchecked).

   **What's unclear:** Whether the planner should explicitly add `!proptest-regressions/` as a `.gitignore` allow or whether the default (no entry) is sufficient.

   **Recommendation:** Add a one-line plan task to verify `.gitignore` doesn't exclude `proptest-regressions/` and commit an initial empty `proptest-regressions/.gitkeep`.

### 4. Existing `program.rs::op_gto` / `op_xeq` / `op_rtn` interactive-running dead branches

   **What we know:** Lines 49–53, 77, 80, 88 are reached only when `state.is_running == true` — but interactive `dispatch()` calls happen only with `is_running == false`. The branches exist as defensive guards.

   **What's unclear:** Whether to (a) write tests that manually set `is_running = true` and call these op functions directly (artificial), (b) annotate with `#[allow(dead_code)]` and accept the cosmetic-ceiling impact, or (c) leave as-is and document in CLAUDE.md.

   **Recommendation:** Option (c) — leave uncovered, document in the Phase 27 verification SUMMARY. The lines are truly unreachable under normal control flow; option (a) is the kind of artificial-test D-27.3 forbids.

### 5. GUI coverage measurement methodology (D-27.4 "measure-only")

   **What we know:**
   - D-27.4 allows the planner to record GUI coverage numbers as a measure-only snapshot.
   - Vitest 4 supports `--coverage` via `v8` or `istanbul` providers.

   **What's unclear:** Whether to add a new `gui-coverage` justfile recipe (one-time use) or just run Vitest with `--coverage` manually and record the numbers.

   **Recommendation:** Manual run during planning, record the numbers in Phase 27 SUMMARY for v3.x reference. No new justfile recipe — would be misleading (it's not a gate).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All test work | ✓ | 1.88 stable (MSRV) | — |
| `cargo-llvm-cov` | FN-QUAL-01 coverage gate | ✓ | (in use by `just coverage`) | — |
| `proptest` 1.11 | FN-QUAL-02/-03 properties | ✓ | 1.11 (workspace dep) | — |
| Node 22 LTS | Vitest + WDIO/Playwright | ✓ | `actions/setup-node@v4 → lts/*` already in ci-gui.yml | — |
| Vitest 4.1.6 | Vitest CI gating (D-27.14) | ✓ | 4.1.6 (devDep) | — |
| `libwebkit2gtk-4.1-dev` | Tauri Linux build (existing) | ✓ | apt-installed in ci-gui.yml | — |
| `webkit2gtk-driver` | tauri-driver Linux runtime | ✗ | — | **MUST INSTALL** — add to apt line in new E2E job (Pitfall 6) |
| `tauri-driver` 2.0.6 | Option B / C E2E | ✗ | — | **MUST INSTALL** — `cargo install tauri-driver --locked` (Pitfall 5: cache `~/.cargo/bin/`) |
| WebdriverIO 9.x | Option B E2E client | ✗ | — | **MUST INSTALL** — `npm install -D webdriverio @wdio/*` |
| `@playwright/test` 1.60.0 | Option A E2E client | ✗ | — | Pending Open Question 1 |
| `xvfb` | Possibly required for tauri-driver on Ubuntu CI | unknown | — | Defensively wrap with `xvfb-run -a` (Assumption A5) |
| Free42 binary | D-27.7 cross-check (manual only) | n/a | — | Cite source line in test doc comments (no binary needed) |

**Missing dependencies with no fallback:** None — all blockers have a documented install path.

**Missing dependencies with fallback:** Playwright vs WebdriverIO is a choice, not a fallback. See Open Question 1.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions confirmed against npm/cargo as of 2026-05-15
- Architecture: HIGH — patterns match existing precedents in the workspace
- Pitfalls: HIGH — each cross-referenced with workspace code or fetched docs
- Coverage targeting: HIGH — fresh `cargo llvm-cov` run produced the line inventory
- Playwright / tauri-driver compatibility: MEDIUM — verified Playwright does NOT speak WebDriver classic; the protocol mismatch is the source of the BLOCKER in Open Question 1
- HP-41 math quirks (Free42 cross-check): MEDIUM — manual cross-references to HP-41 manuals + community forums; Free42 source code not directly fetched in this session (D-27.7 explicitly allows citation-only)

**Research date:** 2026-05-15
**Valid until:** 2026-06-15 (30 days — Tauri 2.11 + proptest 1.11 + WebdriverIO 9 are all in stable release channels with no breaking changes imminent)

## RESEARCH COMPLETE
