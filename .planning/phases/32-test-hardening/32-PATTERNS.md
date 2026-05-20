# Phase 32: Test Hardening — Pattern Map

**Mapped:** 2026-05-18
**Files analyzed:** 9 new/modified (7 modified + 2 net-new)
**Analogs found:** 9 / 9 (every file has an exact or strong analog already in the codebase)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/tests/lint_math1_assertions.rs` (NEW) | test (meta-lint) | file-I/O / source-string-parsing | `hp41-core/tests/math1_op_test_count.rs` | exact (same `include_str!` + `read_dir` shape, same `tests/math1_*.rs` scope) |
| `scripts/check-free42-contamination.sh` (NEW) | CI script | request-response (grep + exit code) | `scripts/check-tauri-permissions.sh` | exact (same bash shape, same `set -euo pipefail`, same exit-N-on-match) |
| `hp41-core/tests/numerical_accuracy.rs` (MODIFIED — extend 566 → ~700 cases) | test (accuracy suite) | CRUD-into-vec of `AccuracyCase` | self (lines 3287+ Math Pac I extension precedent) | exact (continue established pattern) |
| `hp41-core/tests/math1_op_test_count.rs` (MODIFIED — delete vacuous early-return at lines 125-128) | test (meta-gate) | source-string-parsing | self (existing implementation; one-line surgical removal) | exact |
| `hp41-core/tests/math1_user_callback.rs` (OPTIONAL MODIFY — add ~2 GTO-out / recursion-cap tests) | test (regression) | request-response (run loop returns Result) | self (existing 9 active tests, e.g. `user_fn_stops_aborts_integ` at lines 351-389) | exact |
| `hp41-gui/e2e/smoke.spec.js` (MODIFIED — add 2 new `it()` blocks) | test (E2E) | request-response (WebDriver click + assert) | self (existing `2 ENTER 3 +` `it()` block lines 50-84) | exact |
| `.github/workflows/ci.yml` (MODIFIED — add `license-audit` parallel job) | config (CI) | event-driven (push / PR) | self (existing `lint` / `coverage` / `msrv` parallel jobs) | exact |
| `justfile` (MODIFIED — add `license-audit` recipe + extend `ci` recipe) | config (task runner) | request-response | self (existing `coverage` and `ci` recipes lines 60-67) | exact |
| `README.md` (FINAL-COMMIT MODIFY — graduate v3.0 soft-claim to hard-claim) | doc | static | self (existing line 50 v3.0 soft-claim under `## Features`) | exact |

**Files NOT touched but verified safe:**
- `hp41-gui/src/Display14Seg.tsx` — `data-testid="lcd-display"` confirmed present at line 228 (Phase 27 Plan 04 Task 1); the two new E2E `it()` blocks reuse this selector with zero frontend changes. SC-4 unaffected because `hp41-gui/src/` is outside the SC-4 boundary anyway.
- `hp41-gui/wdio.conf.cjs` — unchanged per D-32 (the spec extension uses the existing config).

## Pattern Assignments

### `hp41-core/tests/lint_math1_assertions.rs` (NEW — test, meta-lint over `tests/math1_*.rs`)

**Analog:** `hp41-core/tests/math1_op_test_count.rs` (152 lines, full file already read)

**Free42 disclaim header + file-doc + clippy-allow preamble** (lines 1-40):

```rust
// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 meta-test: each Math Pac I `Op` variant added in Plans 28-02..28-10
//! must have at least 5 test functions mentioning it (Pitfall 16 guard).
//!
//! **Strategy (Plan 28-01 stage — vacuous):**
//! ...
//! **Math Pac I variant detection heuristic:**
//! ...

#![allow(clippy::unwrap_used)]

use std::path::Path;
```

**The TWO important things to mirror exactly:**
1. Line 1-2 — verbatim Free42 disclaim sentence (this is also what `scripts/check-free42-contamination.sh` allowlists; the sentence MUST appear bit-for-bit).
2. Line 40 — `#![allow(clippy::unwrap_used)]` at file scope (every `hp41-core/tests/*.rs` file carries this per CLAUDE.md's "zero panics in hp41-core" invariant).

**`tests/math1_*.rs` directory scan via `CARGO_MANIFEST_DIR`** (lines 76-109 — copy this loop verbatim, change the per-line check):

```rust
fn count_test_mentions(variant_name: &str, tests_dir: &Path) -> usize {
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut total_mentions = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.starts_with("math1_") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("//") && trimmed.contains(variant_name) {
                total_mentions += 1;
            }
        }
    }
    total_mentions
}
```

**`#[test]` body shape — collect-failures + assert-empty** (lines 119-152):

```rust
#[test]
fn each_math1_op_has_at_least_5_tests() {
    let variants = collect_math1_variant_names();
    // ...
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir_buf = manifest_dir.join("tests");
    let tests_dir = tests_dir_buf.as_path();

    let mut failures: Vec<String> = Vec::new();
    for variant_name in &variants {
        let count = count_test_mentions(variant_name, tests_dir);
        if count < 5 {
            failures.push(format!(
                "Op::{variant_name}: only {count} test mention(s) in math1_*.rs (need ≥ 5)"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Pitfall 16 violation — Math Pac I variants with insufficient test coverage:\n{}",
        failures.join("\n")
    );
}
```

**Phase 32 adaptation:** the inner loop changes from "count mentions of variant name" to "scan each `tests/math1_*.rs` body for forbidden lines". Per RESEARCH §3-Open-Q2:
- Flag lines matching `assert_eq!(...\.to_f64()...)` (decimal-to-f64 equality — Pitfall 17)
- Flag lines matching the manual-tolerance pattern `(<expr> - <expr>).abs() < <num>` (forces `approx::assert_relative_eq!` everywhere — single-source-of-truth)
- The 5 known manual-tolerance offenders are `math1_matrix.rs:298`, `:323`, `:360`, `:427`, `:431` (confirmed by grep — see `Bash` results in this session). Decide whether to allowlist them (with `// LINT-EXEMPT: <reason>` markers) or refactor them in this same plan.

**`// Catches:` doc-comment per D-27.1:** every new `#[test]` carries one. Example to use:
```rust
/// Catches: Pitfall 17 — `assert_eq!(decimal, decimal)` on iterated results drifts across x86/ARM.
#[test]
fn no_decimal_assert_eq_in_math1_tests() { /* ... */ }

/// Catches: Pitfall 14 — manual `(a-b).abs() < EPSILON` undermines `max_relative = 1e-7` discipline.
#[test]
fn no_manual_tolerance_pattern_in_math1_tests() { /* ... */ }
```

---

### `scripts/check-free42-contamination.sh` (NEW — CI script)

**Analog:** `scripts/check-tauri-permissions.sh` (32 lines, full file already read)

**Full template — adapt verbatim** (lines 1-32):

```bash
#!/usr/bin/env bash
# scripts/check-tauri-permissions.sh
# CI gate: every command registered in generate_handler! must have a matching
# hp41-gui/src-tauri/permissions/<kebab-case>.toml permission file.
#
# Phase 31 Plan 31-02 (Wave 0, Task 1): authored per 31-RESEARCH.md §"Open Questions Q1"
# Threat T-31-W1-permission-coverage: without this gate, a future command added to
# generate_handler! without a TOML would silently be reachable without declared permissions.
set -euo pipefail

HANDLER_FILE="hp41-gui/src-tauri/src/lib.rs"
PERMS_DIR="hp41-gui/src-tauri/permissions"

# Extract all command names from the generate_handler! macro block.
# Pattern: looks for `commands::<name>` references (snake_case).
commands=$(grep -oE 'commands::[a-z_]+' "$HANDLER_FILE" | sed 's/commands:://' | sort -u)

missing=0
for cmd in $commands; do
    kebab=$(echo "$cmd" | sed 's/_/-/g')
    if [[ ! -f "$PERMS_DIR/$kebab.toml" ]]; then
        echo "MISSING: $PERMS_DIR/$kebab.toml  (for command: $cmd)"
        missing=$((missing + 1))
    fi
done

if [[ $missing -eq 0 ]]; then
    echo "OK: all $(echo "$commands" | wc -w | tr -d ' ') commands have permission TOMLs"
fi

exit $missing
```

**The 4 important things to mirror exactly:**
1. `#!/usr/bin/env bash` shebang (line 1).
2. `set -euo pipefail` (line 9) — same hardening clause; required by CONTEXT.md Claude's Discretion recommendation.
3. Self-documenting header comment block (lines 2-8): script path + one-line purpose + phase plan reference + threat name. Phase 32 version should reference D-32.7 (12-symbol grep policy) + D-32.8 (dual `just ci` / CI invocation) + Pitfall 19 (Free42 GPL contamination).
4. Single-purpose script — one CI invariant per file. Stay that way.

**Phase 32 adaptation per RESEARCH §"Plan 32-03" recipe** (lines 383-414 of RESEARCH.md):

```bash
#!/usr/bin/env bash
# scripts/check-free42-contamination.sh
# CI gate: hp41-core/src/ops/math1/ must contain no distinctive Free42
# identifiers (Intel BID library, decNumber, Free42 internals, copyright
# markers) outside the allowlisted disclaim header on each math1 file.
#
# Phase 32 Plan 32-03: D-32.7 (12-symbol policy) + D-32.8 (dual just-ci / ci.yml invocation).
# Pitfall 19: Free42 GPL contamination via copy-paste from the Free42 reference impl.
set -euo pipefail

MATH1_DIR="hp41-core/src/ops/math1"
DISCLAIM_LINE='Free42 source consulted only as sanity-check oracle'

# D-32.7: 12 distinctive symbols verified zero false-positives against current source.
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License'

if matches=$(grep -rn -E "$PATTERN" "$MATH1_DIR" | grep -v "$DISCLAIM_LINE"); then
    echo "FAIL: Free42 contamination detected in $MATH1_DIR:"
    echo "$matches"
    exit 1
fi

echo "OK: no Free42 contamination detected in $MATH1_DIR/"
exit 0
```

**Important gotcha (per RESEARCH HIGH confidence note):** the `Free42` bare string MUST NOT be in the symbol list — 122 legitimate `Free42 v3.0.5: …` cross-check references exist across `hp41-core/tests/numerical_accuracy.rs` (13) and `hp41-core/src/ops/math1/*.rs` (58). The 12 D-32.7 grep symbols are tighter and DO NOT match any of these. Verified.

---

### `hp41-core/tests/numerical_accuracy.rs` (MODIFIED — extend 566 → ~700 cases, in-place append)

**Analog:** self (5672-line file; the v3.0 Math Pac I extension already lives at lines 3287+)

**File-scope preamble — DO NOT TOUCH** (lines 1-26):

```rust
// Reference values are HP-41 hardware outputs, not mathematical constants.
// These approximate literals are intentional: they represent what the real
// hardware displays, not exact mathematical values.
#![allow(clippy::approx_constant)]
// HP-41 reference values use digit groupings that match the hardware display
// format (e.g., 3.141_592_653 groups at 3+3+3) rather than Rust's 3+3 convention.
#![allow(clippy::inconsistent_digit_grouping)]
#![allow(clippy::unwrap_used)]

//! 503-case numerical accuracy suite for QUAL-06.
//!
//! Reference values derived from HP-41 Owner's Handbook formulas and known
//! mathematical constants. Approach: document-derived (same as Free42, D-05).
//!
//! Tolerance: <= 1e-9 (9-digit relative accuracy threshold; ...).
//!
//! Gate: passes >= 493 (98% of 503, D-08). Failing cases printed as diagnostics.
```

**`case!` macro — already declared at lines 100-123, REUSE bit-for-bit** (the macro accepts 4 args `case!(domain, desc, expected, actual)` or 5 args with the literal `wide` token for `WIDE_TOL`):

```rust
macro_rules! case {
    ($domain:expr, $desc:expr, $expected:expr, $actual:expr) => {{
        id += 1;
        cases.push(AccuracyCase {
            id,
            domain: $domain,
            description: $desc.to_string(),
            expected: $expected,
            actual: $actual,
            tol: TOLERANCE,
        });
    }};
    ($domain:expr, $desc:expr, $expected:expr, $actual:expr, wide) => {{ /* tol: WIDE_TOL */ }};
}
```

**The Math Pac I case shape — copy this 5-line pattern verbatim** (lines 3296-3308; Op::Sinh sinh(1) example):

```rust
{
    // Source: HP 00041-90034 p.44, ex.1 — sinh(1) = 1.175201194
    // Free42 v3.0.5: 1.1752011936 — agrees with OM
    let mut s = CalcState::new();
    push(&mut s, "1");
    dispatch(&mut s, Op::Sinh).unwrap();
    case!(
        "sinh",
        "SINH(1) = 1.175201194 (HP 00041-90034 p.44)",
        1.175_201_193_6,
        get_x(&s)
    );
}
```

**The 3-line doc-comment header is the convention** (per D-27.5/D-27.7 + Plan 32-02 Claude's Discretion):
1. `// Source: HP 00041-90034 p.<n>, ex.<m> — <description>` (OM citation OR emulator-extension marker like `// Source: D-28.3 emulator extension`)
2. `// Free42 v3.0.5: <value> — agrees with OM` (optional cross-check oracle; per D-32.7 the bare string `Free42` does NOT trip the contamination guard because the guard scopes only `hp41-core/src/ops/math1/`)
3. `// Catches: <bug-class wording>` (D-27.1 / Plan 32 Claude's Discretion)

**Non-`case!` POLY cluster assertion shape — for `(x-1)^5` per D-32.10** (analog: TRANS3D pattern at lines 5570-5584):

```rust
// Existing TRANS3D pattern — exact analog for the POLY cluster assertion:
let x_rot = state.stack.x.inner().to_f64().unwrap();
let y_rot = state.stack.y.inner().to_f64().unwrap();
assert!(
    x_rot.abs() < 1e-6,
    "z-axis 90°: x' should be 0, got {x_rot}"
);
assert!(
    (y_rot - 1.0).abs() < 1e-6,
    "z-axis 90°: y' should be 1, got {y_rot}"
);
```

For Phase 32 POLY cluster (D-32.10), use the dual-assertion form per RESEARCH §"POLY cluster assertion":
```rust
{
    // Source: HP 00041-90034 p.32 — multiplicity-5 cluster
    // POLY cluster assertion per D-32.10
    // Catches: POLY multiplicity-as-cluster — per-root tolerance would risk false negatives.
    let mut s = CalcState::new();
    /* ... open POLY modal, enter degree 5, A=1 B=-5 C=10 D=-10 E=5 F=-1, run ... */
    let roots = extract_roots(&s);
    let mean_re = roots.iter().map(|(re, _)| re).sum::<f64>() / 5.0;
    let max_imag = roots.iter().map(|(_, im)| im.abs()).fold(0.0_f64, f64::max);
    assert!((mean_re - 1.0).abs() < 1e-4, "centroid drift: {}", mean_re);
    assert!(max_imag < 1e-3, "imaginary spread: {}", max_imag);
}
```

**D-27.6 baseline-floor and EXPECTED_BASELINE_FAILURES assertions** (lines 4313-4330) — Phase 32 must NOT alter these:

```rust
const EXPECTED_BASELINE_FAILURES: &[usize] = &[124, 279, 344, 438, 480];
let baseline_failures: Vec<usize> = cases
    .iter()
    .filter(|c| c.id < 504) // v1.x baseline cases have id < 504; v2.2 extension ids start at 504
    .filter(|c| !passes_with_tol(c.actual, c.expected, c.tol))
    .map(|c| c.id)
    .collect();
assert_eq!(
    baseline_failures, EXPECTED_BASELINE_FAILURES,
    "D-27.6 BASELINE DRIFT: ..."
);
assert!(
    baseline_passes >= 498,
    "D-27.6 BASELINE REGRESSION: pass count {baseline_passes}/{baseline_total} below floor 498."
);
```

**Phase 32 case IDs**: continue the existing monotonic counter (`id += 1` inside the macro). v1.x baseline = ids 1-503; v2.2 extension = ids 504-566; v3.0 Phase 28 first wave = 567-onwards; Phase 32 appends from wherever the current counter ends.

---

### `hp41-core/tests/math1_op_test_count.rs` (MODIFIED — delete 4 lines)

**Analog:** self

**The exact 4 lines to delete** (lines 125-128):

```rust
    // At Plan 28-01: variants is empty — test passes vacuously.
    // Plans 28-02..28-10 grow this list.
    if variants.is_empty() {
        // Explicitly document the vacuous-pass state.
        return;
    }
```

After deletion, line 124 (`let variants = collect_math1_variant_names();`) flows directly into line 130 (`let manifest_dir = ...`). The remaining body (failures-collection + assert) becomes non-vacuous because `MATH_1.ops` now has 52 entries since Plan 28-09 (per RESEARCH §"Per-Op Test Count Audit").

**Optional adjacent edit**: tighten the doc-comment block at lines 14-21 (`"At Plan 28-01: No Math Pac I Op variants exist yet ... passes vacuously. This is intentional and documented (Pitfall 16)."`) to reflect the Phase 32 graduation. Wording suggestion: "Plan 32-01: gate graduated to non-vacuous — all 45 Math Pac I variants meet the ≥ 5 threshold per the Per-Op Test Count Audit in 32-RESEARCH.md."

---

### `hp41-core/tests/math1_user_callback.rs` (OPTIONAL — add ~2 tests)

**Analog:** self (456 lines, 9 active tests; lines 351-389 show the `user_fn_stops_aborts_integ` shape)

**Existing test shape** (lines 351-389 — `user_fn_stops_aborts_integ`):

```rust
#[test]
fn user_fn_stops_aborts_integ() {
    // Program: LBL "H" / STOP / RTN
    let program = vec![
        Op::Lbl("H".to_string()),
        Op::Stop,
        Op::Rtn,
    ];
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = "H".to_string();
    state.regs[0] = HpNum::from(4i32); // n=4 subdivisions
    state.stack.x = HpNum::from(0i32); // a=0
    state.stack.y = HpNum::from(1i32); // b=1
    state.stack.lift_enabled = false;

    let result = op_integ_run_loop(&mut state, &program);
    assert!(result.is_ok(), "...");
    assert!(state.integ_state.is_none(), "...");
    let x_val = state.stack.x.inner().to_f64().unwrap();
    assert!((x_val - 0.5).abs() < 0.1, "...");
}
```

**Imports preamble — already in place** (lines 25-34):

```rust
#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::math1::difeq::op_difeq_run_loop;
use hp41_core::ops::math1::integ::op_integ_run_loop;
use hp41_core::ops::math1::solve::op_solve_run_loop;
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
```

**Helper functions to reuse** (lines 41-76): `make_identity_fn_state()` builds a baseline INTG-ready state; `make_nested_integ_state()` builds the nested-call regression state. New Phase 32 tests would mirror these (a `make_gto_out_of_callback_state()` and a `make_recursion_cap_state()` per RESEARCH §"math1_user_callback.rs status").

**Per RESEARCH §"math1_user_callback.rs status"**, the two missing categories:
1. `user_fn_gto_out_of_callback_handled` — user function executes `GTO "X"` where `X` is OUTSIDE the user function label. Behavior to assert: the integ loop must not silently follow the GTO out (would break integration semantics); either GTO is contained within the callback frame OR returns an error.
2. `user_fn_recursion_cap_via_user_callback_max_steps` — asserts the `USER_CALLBACK_MAX_STEPS = 100_000` budget catches a recursing user program. Look for the constant in `hp41-core/src/ops/math1/integ.rs` (or similar) before writing the test.

These are OPTIONAL per CONTEXT.md (QUAL-08 is already met with 9 tests by the 5-category rubric).

---

### `hp41-gui/e2e/smoke.spec.js` (MODIFIED — add 2 `it()` blocks inside existing `describe`)

**Analog:** self (86 lines; the `2 ENTER 3 +` `it()` block at lines 50-84 is the exact template)

**`clickKey` helper — REUSE as-is** (lines 35-47):

```javascript
async function clickKey(keyId) {
    const dispatched = await browser.execute((sel) => {
        const el = document.querySelector(sel);
        if (!el) return false;
        el.dispatchEvent(
            new MouseEvent('click', { bubbles: true, cancelable: true }),
        );
        return true;
    }, `[data-key-id="${keyId}"]`);
    if (!dispatched) {
        throw new Error(`element [data-key-id="${keyId}"] not found in DOM`);
    }
}
```

**The `it()` block pattern — exact analog** (lines 50-84):

```javascript
it('2 ENTER 3 + displays 5.0000', async () => {
    const display = await $('[data-testid="lcd-display"]');
    await display.waitForExist({ timeout: 10000 });

    await clickKey('2');
    await clickKey('enter');
    await clickKey('3');
    await clickKey('plus');

    await browser.pause(250);

    const dataText = await display.getAttribute('data-text');
    if (dataText === null) {
        throw new Error(
            "[data-testid='lcd-display'] is missing data-text — Display14Seg contract broken (see hp41-gui/src/Display14Seg.tsx)",
        );
    }
    if (dataText !== '5.0000') {
        throw new Error(
            `expected [data-testid="lcd-display"] data-text='5.0000', got '${dataText}'`,
        );
    }
});
```

**The 5 important things to mirror exactly per D-32.2 / D-32.4:**
1. **DO NOT** touch the existing `2 ENTER 3 +` `it()` block — preserve bit-for-bit (D-32.4).
2. The two new `it()` blocks go **inside the existing `describe()` block** (line 49), not in a new top-level describe (D-32.2).
3. Reuse the `clickKey` helper for every key press (no need to redefine).
4. Reuse `[data-testid="lcd-display"]` + `data-text` assertion shape (the SVG has no `<text>` content).
5. Reuse the `await browser.pause(250)` round-trip pause AND the explicit `null` / `!== expected` error branches.

**Per RESEARCH §"Research priority #3" — Wave 0 reconnaissance needed**:
- The exact XEQ-by-name submit-key id needs to be confirmed by reading `hp41-gui/src/pending_input.ts::handleModalKey` for the `'xeq_name'` case (line ~339 per grep).
- Fallback per RESEARCH if click sequence is brittle: `browser.execute(() => window.__TAURI__.invoke('dispatch_op', {keyId: 'xeq_SINH'}))` — accepted per QUAL-03 ("one Math Pac I workflow"). Document the choice in the test's leading comment.

**The two new tests' shape** (per D-32.2):
```javascript
it('XEQ "SINH" 1 displays 1.1752 (Math Pac I via xrom_resolve)', async () => { /* ... */ });
it('XEQ "MATRIX" 2 [enters values] XEQ "DET" displays -2.0000 (Math Pac I modal pipeline)', async () => { /* ... */ });
```

---

### `.github/workflows/ci.yml` (MODIFIED — add `license-audit` parallel job)

**Analog:** self (90 lines; the existing `lint` / `test` / `coverage` / `msrv` jobs are the structure to mirror)

**The simplest parallel-job shape — `lint` at lines 14-27** (no matrix, no `needs:`):

```yaml
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: just
      - run: just fmt-check
      - run: just lint
```

**Phase 32 `license-audit` job — copy `lint` shape, swap the `run` step** (per RESEARCH §"ci.yml integration" lines 452-461):

```yaml
  license-audit:
    name: License audit (Free42 contamination)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: bash scripts/check-free42-contamination.sh
```

**The 4 important things to mirror per D-32.8:**
1. **Parallel job, not a step inside `lint`** — RESEARCH §"ci.yml integration" recommends this for visible audit-trail.
2. No matrix (Ubuntu-only — the script is bash + grep; no toolchain needed).
3. No `needs:` (it can run independently in parallel with `lint` / `test` / `coverage` / `msrv`).
4. No toolchain action / rust-cache / install-action — the script needs neither Rust nor `just`. Bash + grep are available on `ubuntu-latest` by default. (If the planner decides to invoke via `just license-audit` for parity with local dev, then add the `taiki-e/install-action@v2 tool: just` step.)

**Optional shape (if planner picks `just`-style invocation for cleaner audit trail):**
```yaml
  license-audit:
    name: License audit (Free42 contamination)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/install-action@v2
        with:
          tool: just
      - run: just license-audit
```

---

### `justfile` (MODIFIED — add `license-audit` recipe + extend `ci`)

**Analog:** self (194 lines; the `coverage` recipe at lines 60-63 + `ci` composition at lines 66-67 are the templates)

**Existing `coverage` recipe + `ci` composition** (lines 56-67):

```just
# Check coverage gate — ≥95% line coverage on hp41-core (raised from 80 in Phase 27 / FN-QUAL-01, atomic per D-27.2).
# The matching CI job in ci.yml is named "Coverage (>=95%)" — keep in sync if the threshold ever changes.
[group('ci')]
coverage:
	cargo llvm-cov clean --workspace
	cargo llvm-cov --fail-under-lines 95 -p hp41-core

# Full CI gate: lint → test → coverage
[group('ci')]
ci: lint test coverage
```

**The 4 important things to mirror per D-32.8:**
1. `[group('ci')]` attribute (line 60) — every CI-gate recipe in this file carries this for `just --list` grouping.
2. Self-documenting comment block above each recipe explaining purpose + plan/phase reference. Pattern: "<Phase / D-#> rationale".
3. Recipes are TAB-indented (per the rest of the file). Justfile-strict — no spaces.
4. The `ci` recipe is a one-line composition (`ci: lint test coverage`). Phase 32 extends to `ci: lint test coverage license-audit`.

**Phase 32 additions — append at end of the `ci` group OR right after `coverage`:**

```just
# Phase 32 Plan 32-03 (D-32.7 + D-32.8 + Pitfall 19). Greps hp41-core/src/ops/math1/
# for distinctive Free42 identifiers (Intel BID, decNumber, Free42 internals + GPL/AGPL
# copyright markers) with allowlist for the legitimate per-file disclaim header.
# The matching CI job in ci.yml is named "License audit (Free42 contamination)" — keep
# script invocation in sync if the path ever changes.
[group('ci')]
license-audit:
	bash scripts/check-free42-contamination.sh

# Full CI gate: lint → test → coverage → license-audit (Phase 32 D-32.8 belt+suspenders)
[group('ci')]
ci: lint test coverage license-audit
```

**Note:** the existing `ci-msrv` recipe at line 78 stays unchanged (no license-audit — it's an MSRV-only smoke).

---

### `README.md` (FINAL-COMMIT MODIFY per D-32.5/D-32.6)

**Analog:** self (the v3.0 soft-claim is the current line 50)

**Current state — to replace** (line 50, found by grep):

```markdown
- Math Pac I behavioral emulation (10 top-level programs, ~55 XEQ entry
```

(The line wraps; the planner should read lines 50-52 in full before editing to grab the complete bullet.)

**Phase 32 target wording per D-32.5** (replace the bullet + add a follow-up line):

```markdown
- v3.0 ships Math Pac I behavioral emulation, feature-complete per
  Owner's Manual 00041-90034
  ([documented divergences](docs/hp41-math1-divergences.md))
```

**Per D-32.6, this edit lands in the FINAL ship commit** alongside:
- `PROJECT.md` `Active` line graduation to a `Shipped` v3.0 entry
- `CLAUDE.md` `### v3.0 additions` Phase 32 subsection (replacing the `(in progress)` stub from Phase 30)
- The commit body embeds the output of `just coverage` confirming the ≥ 95 % line coverage gate held

**Existing release table at line 30-37** is the structural template for any future v3.0 row that gets added.

---

## Shared Patterns

### The Free42 disclaim header (verbatim, 2 lines, EVERY hp41-core/tests/math1*.rs file)

**Source:** every `hp41-core/tests/math1_*.rs` file (13/13 confirmed) AND every `hp41-core/src/ops/math1/*.rs` file (13/13 confirmed)

**Apply to:** the new `hp41-core/tests/lint_math1_assertions.rs` file AND any conditional new math1_user_callback.rs additions.

```rust
// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
```

**Why bit-for-bit matters:** the `scripts/check-free42-contamination.sh` script greps with `grep -v 'Free42 source consulted only as sanity-check oracle'`. A typo in the disclaim line would BOTH (a) defeat the allowlist (causing false positives) AND (b) fail the implicit license-hygiene policy. This sentence is locked per ADR-002.

### File-scope clippy-allow preamble

**Source:** every `hp41-core/tests/*.rs` file (per CLAUDE.md "zero panics in `hp41-core`" invariant)

**Apply to:** `hp41-core/tests/lint_math1_assertions.rs` (new file).

```rust
#![allow(clippy::unwrap_used)]
```

For `numerical_accuracy.rs`-style files that also use approximate-constant or unusual digit grouping, the larger preamble (lines 1-8 of `numerical_accuracy.rs`) applies. For the new lint test file, the single `unwrap_used` allow is sufficient.

### `// Catches: <bug class>` doc-comment per `#[test]`

**Source:** D-27.1 / Phase 27 onward; every new test in v2.2 + v3.0 phases carries this

**Apply to:** every new `#[test]` Phase 32 lands, plus every new `case!()` body in `numerical_accuracy.rs` per D-32 Claude's Discretion (the comment goes inside the surrounding block, above the `// Source:` and `// Free42 v3.0.5:` lines).

Example (`math1_complex.rs` lines 43-51):
```rust
/// Catches: CPlus variant missing from dispatch() match (compile-time + runtime).
/// Source: HP 00041-90034 p.24.
#[test]
fn dispatch_c_plus_basic() {
    let mut s = make_state(1.0, 2.0, 3.0, 4.0);
    dispatch(&mut s, Op::CPlus).unwrap();
    assert_relative_eq!(get_x(&s), 4.0, max_relative = 1e-7);
    assert_relative_eq!(get_y(&s), 6.0, max_relative = 1e-7);
}
```

### `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` discipline

**Source:** `hp41-core/tests/math1_complex.rs:17,49,50` (active use) + Pitfall 14 (cross-platform drift on Math Pac I 6-of-10 digits)

**Apply to:** every new accuracy assertion in `tests/math1_*.rs` AND every new `numerical_accuracy.rs` case where the `case!` macro's internal `TOLERANCE = 1e-9` isn't appropriate.

```rust
use approx::assert_relative_eq;

assert_relative_eq!(get_x(&s), 4.0, max_relative = 1e-7);
```

**Important:** `approx` works on `f64` directly, NOT on `HpNum` / `Decimal`. The established conversion bridge is `state.stack.x.inner().to_f64().unwrap()` (or `get_x(state)` helper). `approx` 0.5.1 is already in `[dev-dependencies]` per Cargo.toml line 18 — DO NOT add it (RESEARCH HIGH-confidence correction of an earlier note).

### CI-gate script shape (bash + grep + exit-N-on-match)

**Source:** `scripts/check-tauri-permissions.sh:1-32` (v2.0 / Phase 31 pattern)

**Apply to:** `scripts/check-free42-contamination.sh` (new). Same `#!/usr/bin/env bash`, same `set -euo pipefail`, same self-documenting header, same single-purpose discipline.

### Justfile `[group('ci')]` recipe + `ci:` composition

**Source:** `justfile:60-67` (existing `coverage` + `ci`)

**Apply to:** new `license-audit` recipe + extended `ci: lint test coverage license-audit` composition.

## No Analog Found

None. Every Phase 32 file has an exact analog in the existing codebase (the milestone is deliberately a "re-application of the v2.2 Phase 27 playbook" per RESEARCH §"Phase Summary").

## Metadata

**Analog search scope:**
- `hp41-core/tests/` (14 math1_*.rs files + numerical_accuracy.rs + xrom_*.rs)
- `hp41-gui/e2e/` (smoke.spec.js)
- `hp41-gui/src/` (Display14Seg.tsx — verified data-testid attribute already present)
- `.github/workflows/` (ci.yml job structure)
- `scripts/` (check-tauri-permissions.sh)
- `justfile` (coverage + ci recipes)
- `README.md` (Features section bullets)

**Files scanned (read at least one targeted range):** 9
- `hp41-core/tests/math1_op_test_count.rs` (full, 152 lines)
- `scripts/check-tauri-permissions.sh` (full, 32 lines)
- `hp41-gui/e2e/smoke.spec.js` (full, 86 lines)
- `justfile` (full, 194 lines)
- `hp41-core/tests/numerical_accuracy.rs` (lines 1-130, 3280-3399, 4290-4334, 5550-5671 — non-overlapping)
- `hp41-core/tests/math1_user_callback.rs` (lines 1-80 + 340-456)
- `.github/workflows/ci.yml` (full, 90 lines)
- `hp41-core/tests/math1_complex.rs` (lines 1-60)
- `hp41-gui/src/Display14Seg.tsx` (lines 215-234)
- `hp41-core/tests/xrom_shadowing.rs` (lines 1-50)
- `README.md` (lines 1-60 + grep confirmation of line 50)

**Pattern extraction date:** 2026-05-18
