---
phase: 32-test-hardening
reviewed: 2026-05-18T00:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - .github/workflows/ci.yml
  - hp41-core/tests/lint_math1_assertions.rs
  - hp41-core/tests/math1_four_tri_trans.rs
  - hp41-core/tests/math1_integ.rs
  - hp41-core/tests/math1_matrix.rs
  - hp41-core/tests/math1_matrix_flow.rs
  - hp41-core/tests/math1_mod_entry_points.rs
  - hp41-core/tests/math1_op_test_count.rs
  - hp41-core/tests/math1_user_callback.rs
  - hp41-core/tests/numerical_accuracy.rs
  - hp41-core/tests/xrom_shadowing.rs
  - hp41-gui/e2e/smoke.spec.js
  - scripts/check-free42-contamination.sh
findings:
  critical: 1
  warning: 7
  info: 4
  total: 12
status: issues_found
---

# Phase 32: Code Review Report

**Reviewed:** 2026-05-18
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Phase 32 (Test Hardening) lands ~2.7k new lines in `numerical_accuracy.rs`, eight new `math1_*.rs` test files, a Bash CI helper, and a CI workflow extension. The work is broadly consistent with the project's stated discipline (file-scope `#![allow(clippy::unwrap_used)]`, OM citations, `// Catches:` comments, `LINT-EXEMPT:` annotations). Adversarial review surfaces one **BLOCKER** (15 tautological "always-pass" cases that inflate the 98% suite gate while contributing zero regression-catching value, contradicting `lint_math1_assertions.rs`'s own T-32-04 meta-test-gaming threat model) plus several **WARNING** quality issues — chiefly: the Free42 contamination guard silently passes when `MATH1_DIR` is missing/renamed, the assertion-discipline lint has documented gaps that allow real Pitfall 17 violations through, the `count_test_mentions` heuristic measures substring occurrences rather than test functions and over-counts via shorter-variant substring matches, and the E2E smoke spec relies on 9 hard-coded `browser.pause()` calls with no test-isolation between specs.

No production code is in scope this phase, so security findings are constrained to the Bash script and CI workflow YAML. No secret-leak vectors, no untrusted-input injection paths, no version-floating action pins beyond the documented `dtolnay/rust-toolchain@stable` tracker.

## Critical Issues

### CR-01: 15 tautological `r.is_ok() || r.is_err()` cases inflate the numerical-accuracy 98% gate

**File:** `hp41-core/tests/numerical_accuracy.rs:5867,5885,5903,5921,5944,5962,5980,5999,6018,6041,6060,6078,6097,6116,6266`

**Issue:** Fifteen new POLY cases in the Phase 32 / Plan 32-02 block use the predicate `if r.is_ok() || r.is_err() { 1.0 } else { 0.0 }` inside `case!()` against an expected value of `1.0`. Since Rust's `Result<T, E>` has exactly two variants (Ok and Err), the predicate is a tautology — it evaluates to `1.0` regardless of dispatcher behaviour, panics inside the dispatcher, or future regressions. The case carries zero regression-catching signal but counts toward the `passes >= ceiling(total * 0.98)` gate at line 7044. Adding 15 guaranteed passes to the denominator raises the floor while contributing nothing to the numerator's risk-weighted signal. The comment headers claim "Catches: Bairstow degree-N deflation broken" etc. — those `// Catches:` rationales are unverified by the case body.

This is exactly the meta-test-gaming threat that `hp41-core/tests/lint_math1_assertions.rs` (also landed this phase) calls out as T-32-04 at lines 19-21 and 109. The companion lint forbids `assert_eq!(decimal, decimal)` and `(a - b).abs() < EPSILON` without rationale, yet these tautologies sit in the same file and slip past — `numerical_accuracy.rs` is explicitly out-of-scope for `lint_math1_assertions.rs` (file is not `math1_*.rs`).

Representative example (line 5862-5868):
```rust
let r = dispatch(&mut s, Op::Roots);
case!(
    "poly_d3_x3_eq_1",
    "POLY: x³-1 dispatched (HP 00041-90034 p.31)",
    1.0,
    if r.is_ok() || r.is_err() { 1.0 } else { 0.0 }   // always 1.0
);
```

**Fix:** Replace each tautology with a meaningful regression sentinel. Either (a) assert a specific bound on the resolved roots (e.g., for `x³-1`, sum of real-part U= lines ≈ 1 if convergence is achieved), or (b) tighten the predicate to match a specific enumerated outcome — `if r.is_ok() || matches!(r, Err(HpError::Domain)) { 1.0 } else { 0.0 }` like line 5845 does — which would at least fail on `Err(Overflow)`, `Err(CallDepth)`, `Err(InvalidOp)`, or a panic. If neither is feasible because Bairstow's actual behaviour on these polynomials is implementation-dependent, **delete** the cases rather than recording them as passing — a deleted case can't game the 98% gate.

```rust
// Option (a): real-root sum sentinel for x³-1 (root 1 + complex pair)
let r = dispatch(&mut s, Op::Roots);
let u_vals: Vec<f64> = s.print_buffer.iter()
    .filter(|l| l.starts_with("U="))
    .map(|l| parse_u_value(l)).collect();
case!(
    "poly_d3_x3_eq_1_real",
    "POLY: x³-1 real root ≈ 1 (HP 00041-90034 p.31)",
    1.0,
    if r.is_ok() && u_vals.iter().any(|v| (v - 1.0).abs() < 0.1) { 1.0 } else { 0.0 }
);
```

## Warnings

### WR-01: `check-free42-contamination.sh` silently passes when `MATH1_DIR` does not exist

**File:** `scripts/check-free42-contamination.sh:11,20`

**Issue:** The script sets `MATH1_DIR="hp41-core/src/ops/math1"` and runs `grep -rn -E "$PATTERN" "$MATH1_DIR" | grep -v "$DISCLAIM_LINE"`. If `MATH1_DIR` is missing (refactor renamed the directory; script invoked from wrong cwd; future v3.1 module split moved files), `grep -rn` returns 2 (error), the pipeline fails, but the `if matches=$(...)` evaluates to false because the exit code is non-zero — and the script prints "OK: no Free42 contamination detected" and exits 0. Reproduced with `bash -c 'set -euo pipefail; if matches=$(grep -rn phloat /nonexistent | grep -v sanity); then echo FAIL; exit 1; fi; echo OK'` → prints `OK`.

This silently neutralises the Pitfall 19 / D-32.7 license guard whenever the directory structure changes — exactly the moment when contamination is most likely to slip in (during a refactor).

**Fix:** Add an explicit existence check at the top of the script:
```bash
set -euo pipefail
MATH1_DIR="hp41-core/src/ops/math1"
if [[ ! -d "$MATH1_DIR" ]]; then
    echo "FAIL: $MATH1_DIR does not exist — license guard cannot run." >&2
    exit 2
fi
DISCLAIM_LINE='Free42 source consulted only as sanity-check oracle'
# ... rest unchanged
```

### WR-02: `lint_math1_assertions.rs` heuristic misses multi-line `assert_eq!(decimal, decimal)` calls

**File:** `hp41-core/tests/lint_math1_assertions.rs:161-183`

**Issue:** `line_is_forbidden_assert_eq` requires `assert_eq!` AND one of `.to_f64() | HpNum | Decimal | .inner()` to appear on the SAME line. A multi-line invocation like the one already present in `hp41-core/tests/math1_poly.rs`:

```rust
assert_eq!(
    s.stack.x.inner(),
    x_before,
    "Op::PolyWorkflow must not modify stack X (LiftEffect::Neutral)"
);
```

is not detected. Neither line carries both tokens. The Pitfall 17 threat the lint advertises (`.to_f64()` cross-platform drift) is fully exposed by patterns like:

```rust
assert_eq!(
    state.stack.x.inner().to_f64().unwrap(),
    expected_f64,
);
```

The lint's `## Scope` docstring at lines 47-52 doesn't disclose this gap. T-32-04 says "the offender list is reported in full so a reviewer can spot weakening" (line 229) — but the offender wouldn't appear in the list to spot in the first place.

**Fix:** Either (a) join multi-line `assert_eq!` invocations before scanning by tracking parenthesis depth, or (b) document the limitation explicitly in the file-level `//!` so future contributors understand the lint's blind spot, or (c) widen the scan to flag any `assert_eq!` line and check the next 3 lines for the decimal tokens.

```rust
// Option (c) — minimal change:
let next_3 = lines.iter().skip(idx).take(3).copied().collect::<Vec<_>>().join("\n");
let is_decimal = next_3.contains(".to_f64()")
    || next_3.contains("HpNum")
    || next_3.contains("Decimal")
    || next_3.contains(".inner()");
```

### WR-03: `count_test_mentions` measures substring occurrences, not test functions

**File:** `hp41-core/tests/math1_op_test_count.rs:77-110`

**Issue:** The docstring at line 76-77 promises "Count how many test functions (lines containing `#[test]` followed by `fn`) in `hp41-core/tests/math1_*.rs` mention the given variant name." The implementation does NOT match this: it counts the number of non-comment lines where `trimmed.contains(variant_name)` (lines 101-106). A single test function with five mentions of `Op::Sinh` satisfies the `≥ 5` gate by itself — the gate's "≥ 5 test functions" claim is unmet.

Worse, `contains(variant_name)` is a **substring match**, so the counter for `Sol` is inflated by every line mentioning `Solve` (45 variants in xrom.rs; `Sol` is contained in `Solve`, `Sinh` is contained in `Asinh`, `Cosh` in `Acosh`, `Tanh` in `Atanh`). The gate is one-sided (`< 5` → fail), so substring inflation can only HIDE a real shortfall — a variant with only 2 genuine mentions could pass the gate by riding on a 3-mention longer variant. The "TriSaa=6, TriSas=6" minimum baseline in the docstring (line 18-22) is potentially wrong if those counts were measured by the buggy heuristic.

**Fix:** Use word-boundary matching and count function definitions, not line occurrences:
```rust
// Match Op::VariantName as a whole token (preceded by non-alnum, followed by non-alnum/_)
fn line_mentions_variant(line: &str, variant: &str) -> bool {
    let token = format!("Op::{variant}");
    if !line.contains(&token) { return false; }
    // Reject if followed by [A-Za-z0-9_] (i.e. substring of longer variant)
    line.split(&token).skip(1).any(|after| {
        after.chars().next().map_or(true, |c| !c.is_alphanumeric() && c != '_')
    })
}

// Then count test functions whose body mentions the variant:
let mut in_test_fn = false;
let mut current_fn_mentions = false;
let mut test_fn_count = 0;
for line in content.lines() {
    if line.contains("#[test]") { in_test_fn = true; current_fn_mentions = false; continue; }
    if in_test_fn && line_mentions_variant(line, variant_name) { current_fn_mentions = true; }
    if in_test_fn && line.trim() == "}" {  // approximate end-of-fn
        if current_fn_mentions { test_fn_count += 1; }
        in_test_fn = false;
    }
}
```

### WR-04: 7 cases in the Phase 32 block use a redundant `case!()` sentinel after a real `assert!`

**File:** `hp41-core/tests/numerical_accuracy.rs:4344-4346, 4361-4363, 4962-4964, 4980-4982, 5680-5682, 6180-6182, 6803-6805`

**Issue:** Pattern `case!("name", "desc", 1.0, 1.0)` is recorded after a preceding `assert!(matches!(r, Err(...)), "...")` block (e.g. lines 4329-4347 CMPLX-03). The `assert!` does the actual verification — the case! records a sentinel pass for the suite counter. The comment "// The assertion above doubles as the case proof; record a sentinel pass for the suite-counter accounting." (line 4339) is honest about this, but the sentinel still inflates the denominator of the 98% gate by 7 cases, same kind of meta-test-gaming as CR-01 but with the real check up-front.

Less severe than CR-01 because each sentinel here is paired with a real assertion that DOES fail the test if the contract breaks — so the test will surface a regression. But the 98%-gate inflation remains a quality concern.

**Fix:** Either delete the sentinels (the `assert!` already protects the contract), or record `(1.0, expected_value)` where `expected_value` is a meaningful witness from the assertion-checked branch. Preferred: remove the sentinels — the assertion is sufficient.

### WR-05: E2E smoke spec uses 9 hard-coded `browser.pause()` calls instead of waitFor predicates

**File:** `hp41-gui/e2e/smoke.spec.js:110,159,202,209,215,218,221,224,230`

**Issue:** Each click sequence is followed by `await browser.pause(150ms..500ms)`. This is the canonical flaky-E2E anti-pattern. The header comment at line 109 says "Give React a moment to round-trip through the Tauri IPC and re-render" — but the actual wait should be predicate-driven (`waitUntil(async () => (await display.getAttribute('data-text')) === '5.0000')`), not time-driven. On a slow CI runner (Ubuntu xvfb with cold cache + DET heavier than +) `250ms` may be insufficient; on a fast runner it's wasted time. Plan 32-03 noted `mochaOpts.retries: 1` per D-27.16, which is a flake mitigation — but retries can mask the underlying timing fragility.

**Fix:** Replace `browser.pause(N)` followed by `display.getAttribute('data-text')` with `display.waitUntil(async () => (await display.getAttribute('data-text')) === expected, { timeout: 5000 })`. Keeps the test fast on healthy runs and tolerant on slow ones.

### WR-06: Smoke spec tests share state across `it()` blocks — no per-test isolation

**File:** `hp41-gui/e2e/smoke.spec.js:96-243`

**Issue:** The three `it()` blocks in `describe('HP-41 GUI smoke...')` share a single Tauri app instance. After test 1 finishes with X=5, test 2 starts by clicking `1 → enter` then `xeq_SINH` — relying on the assumption that SINH operates only on X (so the residual stack from test 1 doesn't matter). Test 3 opens MATRIX workflow — which works because the modal pipeline overwrites state irrespective of pre-test residual.

This works today but is fragile. A future op added to test 2 that reads from Y (e.g. `Op::Add`) would silently consume the leftover `5` from test 1, masking a regression where the test should have failed. Additionally, on local dev runs, `~/.hp41/autosave.json` persists across `just gui-e2e` invocations and can pollute subsequent runs.

**Fix:** Add a `beforeEach` hook that resets state, e.g. via `await invokeBackend('clear_all', {})` (if such a command exists) or by deleting `~/.hp41/autosave.json` and restarting the app. At minimum, document the implicit ordering dependency at the top of each `it()` block.

### WR-07: Smoke spec `String(err && err.message ? err.message : err)` mishandles non-Error falsy values

**File:** `hp41-gui/e2e/smoke.spec.js:84`

**Issue:** The ternary `err && err.message ? err.message : err` is meant to extract `err.message` when present. But if `err` is `0`, `""`, `null`, or `false`, the ternary returns `err` itself, and `String(err)` produces `"0"`, `""`, `"null"`, `"false"` — which is then passed to the caller as `result.err`. The caller throws `new Error(...)` with the `err` value but the message becomes uninformative. Tauri rejections typically return `GuiError { message: string }` (see hp41-gui/src-tauri/src/types.rs) so this is unlikely to trigger in practice, but a backend that returns `null` (panic-in-thunk path) would produce `"invokeBackend('...') failed: null"` instead of an actionable stack.

**Fix:**
```js
.catch(err => done({
    ok: false,
    err: err && err.message ? err.message : (err === null ? 'null' : JSON.stringify(err))
}));
```

## Info

### IN-01: `tri_ssa_no_solution_case` lacks message on the length assertion

**File:** `hp41-core/tests/math1_four_tri_trans.rs:509`

**Issue:** `assert_eq!(state.print_buffer.len(), 1);` has no message. The following `assert_eq!(state.print_buffer[0], "NO SOLUTION", ...)` would panic with an index-out-of-bounds error if the buffer is empty — less informative than a `len() == 1` failure with explanation.

**Fix:** `assert_eq!(state.print_buffer.len(), 1, "Op::TriSsa NO SOLUTION must produce exactly 1 line");`

### IN-02: `make_state_with_fn` parameter `n` cast `as i32` silently truncates u32

**File:** `hp41-core/tests/math1_integ.rs:37`

**Issue:** `state.regs[0] = HpNum::from(n as i32);` — `n: u32` silently wraps to negative if `n > i32::MAX = 2_147_483_647`. The current test data uses `n ∈ {4, 8, 10, 32769, 40000}`, all well within `i32::MAX`, so this won't trigger. But test 4 (`integ_subdivision_cap_32768`) passes `32769u32` — fine. A future test with `n = 3_000_000_000` would wrap to a negative HpNum and silently break.

**Fix:** Either accept `n: i32` directly, or `HpNum::from(rust_decimal::Decimal::from(n))` (Decimal handles u32 losslessly).

### IN-03: `lint_math1_assertions.rs` Phase 1 walk does not handle `idx == 0` cleanly

**File:** `hp41-core/tests/lint_math1_assertions.rs:110-131`

**Issue:** `preceding_block_has_lint_exempt` starts with `while i > 0 { i -= 1; ... }`. If `idx == 0` (first line of file is the offender), the loop body never runs and falls through to Phase 2's comment scan at `lines[0]`. This is technically fine — Phase 2 checks `lines[0].trim_start().starts_with("//")` — but the control flow is non-obvious. A future refactor that moves the `if i == 0` check could mis-handle the edge case.

**Fix:** Add an early-return: `if idx == 0 { return false; }` at the top of the function.

### IN-04: `xrom_shadowing.rs` `BUILTIN_CARD_OP_NAMES` is a hand-curated parallel list

**File:** `hp41-core/tests/xrom_shadowing.rs:38-60`

**Issue:** The 18-entry list at lines 38-60 must be kept in sync with `hp41-core/src/ops/program.rs::builtin_card_op`. The file-level docstring at lines 29-32 acknowledges this and asks future contributors to maintain the sync. Hand-curated parallel lists drift — the v2.2 D-25.18 JSON-derived `KEY_REF_TABLE` pattern would prevent this. Not urgent (the list is short and rarely changes), but worth tracking as v3.1+ tech debt.

**Fix:** Long-term, derive the allowlist by parsing `program.rs::builtin_card_op` match arms via `include_str!` at test time, same trick as `math1_op_test_count.rs::collect_math1_variant_names`. For Phase 32, document the explicit cross-reference (commit hash where verification was done) in the file header.

---

_Reviewed: 2026-05-18_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
