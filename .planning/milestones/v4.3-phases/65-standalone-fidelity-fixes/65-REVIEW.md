---
phase: 65-standalone-fidelity-fixes
reviewed: 2026-06-07T00:00:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - hp41-core/src/num.rs
  - hp41-core/src/format.rs
  - hp41-core/src/ops/math.rs
  - hp41-core/src/ops/indirect.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/advantage/conv.rs
  - hp41-core/src/ops/advantage/matrix_ops.rs
  - hp41-core/src/state.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/src/app.rs
  - hp41-cli/src/keys.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-core/tests/numerical_accuracy.rs
  - hp41-core/tests/proptest_math.rs
  - docs/adr/v4.3-005-hpnum-range-extension.md
findings:
  critical: 2
  warning: 5
  info: 3
  total: 10
status: issues_found
---

# Phase 65: Code Review Report

**Reviewed:** 2026-06-07
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

The dominant change (MATH-01) re-architects `HpNum` from a `Decimal` newtype into a
two-tier `{mantissa, exponent: i8}` struct reaching ±9.999E±99. The frontend fixes
(DISP-01 display_override, DISP-02 CHS in-buffer toggle, DISP-03 AON flag-48 branch on
both CLI `ui.rs` and GUI `types.rs`) are correct, well-tested, and maintain CLI↔GUI
parity (D-10/D-25.6). The serde backward-compat path (HpNumWire untagged, Extended arm
first) round-trips legacy bare-string saves correctly. No new bare `.unwrap()` exists in
non-test core code — the panic-free invariant holds.

The new large-exponent **arithmetic** path, however, has a confirmed silent-wrong-result
bug: the exponent of a `checked_mul` / `checked_div` result is computed as `i32` then
truncated with `as i8`, so any product/quotient whose true exponent exceeds 99 wraps to a
small in-range value and returns a wildly wrong number instead of `HpError::Overflow`.
This path is **never exercised by the property tests** (their generator clamps exponents
to ±18) and only thinly covered by hand-written unit tests that stay within range, so the
"3577 green" signal does not cover it. This is the highest-value finding.

## Critical Issues

### CR-01: `checked_mul` / `checked_div` exponent overflow silently wraps via `as i8`

**File:** `hp41-core/src/num.rs` (checked_mul `new_exp = e_l + e_r; from_sci(product, new_exp as i8)`; checked_div `new_exp = e_l - e_r; from_sci(quotient, new_exp as i8)`)
**Issue:**
In the large-exponent paths, `e_l` and `e_r` are each in `-99..=99` (`i32`). Their sum
(mul) or difference (div) can range to ±198, which does **not** fit in `i8` (range
-128..=127). The code computes the exponent as `i32` but then narrows it with `as i8`
before calling `from_sci`:

```rust
let new_exp = e_l + e_r;          // up to 99 + 99 = 198
HpNum::from_sci(product, new_exp as i8)   // 198 as i8 == -58  ← WRAP
```

`from_sci(_, -58)` then treats the operand as a perfectly valid in-range value and returns
a tiny number near `mantissa × 10^-58` — instead of `Err(HpError::Overflow)`. Verified
empirically: `198 as i8 == -58`. So `1e80 × 1e80` (true value 1e160, must overflow)
returns ~`1e-58`; `1e99 ÷ 1e-99` (true 1e198, must overflow) likewise yields garbage.
`checked_div` is symmetric: `99 - (-99) = 198 → -58`.

The `from_sci` guard `if exponent > 99 { Err(Overflow) }` is dead for these callers because
the wrap happens *before* `from_sci` ever sees the real magnitude. This is silent data
corruption on a core arithmetic op — the exact failure mode the project's checked_* /
Overflow discipline exists to prevent.

`checked_add_sci` is NOT affected for its aligned-sum branch (`base_exp = e_l.min(e_r)` is
in range and the carry only adds 1, handled by `from_sci`), but it shares the same latent
class — see CR note below if `to_sci` is ever fed out-of-range exponents.

**Fix:** Compute the exponent in `i32`, range-check it before narrowing, and overflow a
new `from_sci` variant (or pass `i32`):

```rust
// checked_mul large-exponent path:
let new_exp = e_l + e_r;
if new_exp > 99 { return Err(HpError::Overflow); }
if new_exp < -99 { return Ok(HpNum::zero()); }   // underflow → 0, hardware-faithful
HpNum::from_sci(product, new_exp as i8)

// checked_div large-exponent path: identical guard on `new_exp = e_l - e_r`.
```

Better: change `from_sci` to take `exponent: i32` and do the `>99 / < -99` checks there so
all call sites are protected centrally; keep the `as i8` only at the final struct store
after the range check has passed. Add a regression test for `1e80 * 1e80`,
`9e99 * 9e99`, and `1e99 / 1e-99` asserting `Err(Overflow)`.

### CR-02: Large-exponent multiply/divide/add paths are untested (false-green coverage)

**File:** `hp41-core/tests/proptest_math.rs:31-36` (generator clamps exponent to ±18); `hp41-core/src/num.rs` (checked_mul / checked_div / checked_add_sci large-exponent branches)
**Issue:**
The proptest `decimal_strategy` explicitly clamps the generated exponent to ±18:

> "The effective exponent range is clamped to ±18 because `10i64.pow(n)` would overflow
> for n > 18 … the ±18 effective range covers thoroughly."

Every generated operand therefore has `exponent == 0` after construction, so the
property-based round-trip / inverse tests **only ever hit the `exponent == 0` fast path**
of `checked_add` / `checked_mul` / `checked_div`. The new `checked_add_sci`, `to_sci`,
and the large-exponent mul/div branches receive zero property coverage. The handful of
unit tests in `num.rs` (`1e40 * 1e40 = 1e80`, `9e98 + 9e98`) all stay strictly inside the
valid range, so they cannot catch CR-01 or any exponent-boundary defect. This is why the
"3577 green / verifier 4/4" signal is misleading for the MATH-01 change: the most
behaviorally novel code in the phase is effectively unverified.

**Fix:** Add a dedicated large-exponent test module that exercises mantissa+exponent
operands directly (built via `HpNum::from_f64(1e80)` etc.), covering: (a) products and
quotients that exceed ±99 → must be `Err(Overflow)` / underflow-to-zero; (b)
`checked_add_sci` with `exp_diff` straddling the ±10 truncation boundary; (c) `to_sci`
exponent extraction at exact powers of ten (`1e30`, `1e-30`). At minimum, assert the
CR-01 cases. Optionally widen the proptest generator to emit a separate large-exponent
strategy (operands as mantissa×10^e for e in a representative ±99 spread) so inverse
identities are checked there too.

## Warnings

### WR-01: Extended serde arm deserializes without normalization or range validation

**File:** `hp41-core/src/num.rs` (Deserialize impl, `HpNumWire::Extended { m, e } => Ok(HpNum { mantissa, exponent: e })`)
**Issue:**
The Extended arm constructs `HpNum { mantissa, exponent: e }` directly from the wire
fields with no checks: the mantissa is not re-normalized to [1,10), not re-rounded to 10
sig digits, and `e` is not validated against `-99..=99` (only the i8 type bounds it to
±127). A hand-edited or corrupted save with `"e": 120` or a denormalized mantissa
(`"m": "55.3"`, `"e": 5`) loads into an `HpNum` that violates the struct's documented
invariants, which then propagates into `to_sci` / arithmetic / display and produces
inconsistent results (and `format_hpnum` will treat `exponent != 0` as a [1,10) mantissa
even when it isn't). Legitimate saves are fine because the serializer always emits
in-range normalized values — but deserialization is an untrusted boundary.

**Fix:** Route the Extended arm through the validating constructor instead of building the
struct literally:

```rust
HpNumWire::Extended { m, e } => {
    let mantissa = Decimal::from_str(&m).map_err(serde::de::Error::custom)?;
    HpNum::from_sci(mantissa, e).map_err(serde::de::Error::custom)
}
```

This re-applies the rounding/carry/range logic and rejects out-of-range exponents. Note
`from_sci` currently uses `i8` for `e`, so it cannot even represent the >99 case — another
reason to widen it (see CR-01 fix). Guard against the `e == 0` normalized case if you want
exponent-0 values to keep their full Decimal mantissa.

### WR-02: `to_sci` extracts the base-10 exponent via f64 `log10().floor()`

**File:** `hp41-core/src/num.rs` (`to_sci`: `let exp = if f >= 1.0 { f.log10().floor() as i32 } ...`)
**Issue:**
The project's Frozen Invariant mandates extracting integer/fraction structure by
string-splitting at the decimal point, "never `floor()`/`fmod()`" — precisely because f64
`log10` is imprecise at powers of ten (e.g. `log10(1000.0)` can come back as `2.9999…`,
flooring to 2 and yielding a 10×-wrong normalization). The exponent extraction here floors
an f64 `log10`. Spot checks at `1e3`, `1e15`, `1e-3` happened to land correctly on this
platform, but this is exactly the class of off-by-one the invariant forbids, and it is
load-bearing: a wrong `exp` here feeds directly into mul/div/add exponent arithmetic. The
mitigating factor (downgrade from CR) is that `to_sci`'s exponent-0 normalization branch
is only reached for operands already inside Decimal range, where the subsequent
`checked_div(scale)` partially self-corrects the mantissa — but the returned `exp` itself
can still be off by one at a boundary and corrupt the result exponent.

**Fix:** Derive the exponent from the Decimal's own scale / digit count (string form),
consistent with the ISG/DSE and date-parsing discipline, rather than f64 `log10`. E.g.
use the position of the first significant digit in `mantissa.normalize().to_string()`, or
`Decimal::scale()` + mantissa digit count. Add boundary tests at `1e-1, 1, 1e1` and exact
powers within ±28.

### WR-03: `from_f64` does not renormalize a mantissa that lands below 1.0

**File:** `hp41-core/src/num.rs` (`from_f64`: `let mantissa_f = acc / 10f64.powi(exp); ... from_sci(mantissa, exp as i8)`)
**Issue:**
`exp` comes from `abs_acc.log10().floor()` (same f64 hazard as WR-02). If f64 rounding
makes `mantissa_f` come out as e.g. `0.9999999999` (just below 1.0) instead of ~1.0 at a
power-of-ten boundary, `from_sci` will store `mantissa < 1` with the given `exp`.
`from_sci` handles the `mantissa >= 10` carry-up case but has **no** symmetric branch for
`mantissa < 1` (borrow-down), so the value is stored denormalized and is off by a factor
of 10 in subsequent large-exponent arithmetic and in `format_sci_large` (which assumes
mantissa ∈ [1,10)). Only reachable for `|acc| > ~7.92E28`, i.e. FACT(≥28) and similar, so
real but narrow.

**Fix:** After computing `mantissa_f`, clamp/renormalize before `from_sci`: if
`mantissa_f.abs() < 1.0`, multiply by 10 and decrement `exp`; if `>= 10.0`, divide and
increment (or let a hardened `from_sci` borrow-normalize). Add a FACT golden at a
power-of-ten-adjacent magnitude to exercise it.

### WR-04: `decimal_pow10_f64` / negative-exp string builders panic-by-construction on `exp == 0`-adjacent inputs

**File:** `hp41-core/src/num.rs` (`decimal_pow10_f64`, negative branch `"0.".to_string() + &"0".repeat(abs_exp - 1) + "1"`)
**Issue:**
The negative-exponent branch computes `abs_exp - 1` where `abs_exp = (-exp) as usize`.
This is only safe because the `exp == 0` and `exp > 0` cases are handled earlier — but the
function is `pub(crate)`-reachable and the invariant (`exp != 0` on entry to the negative
branch) is implicit. If a future caller passes `exp == 0` to the negative branch logic via
refactor, `abs_exp - 1` underflows `usize` and panics. The same `abs_exp - 1` pattern in
`format.rs::decimal_pow10` carries the identical implicit precondition. Not currently a
live bug, but a latent panic source in panic-free core code.

**Fix:** Make the precondition explicit with a guard (`debug_assert!(exp != 0)` plus a
`saturating_sub(1)` or early-return) so a future misuse cannot underflow.

### WR-05: `op_adv_fnrm` / `op_adv_sum` ignore the new ±9.999E±99 range (regression vs MATH-01 intent)

**File:** `hp41-core/src/ops/advantage/matrix_ops.rs` (`op_adv_fnrm` uses `Decimal::from_f64(norm_f64).map(HpNum::rounded).ok_or(HpError::Overflow)`; reduction ops accumulate via `inner()`/`checked_add`)
**Issue:**
`op_adv_fnrm` converts the final norm through `Decimal::from_f64(...).ok_or(Overflow)`,
which still hits the old ~7.92E28 Decimal wall and returns `Overflow` for norms above it —
even though `HpNum` now represents up to 9.999E99 and `HpNum::from_f64` would handle it.
This is the exact ceiling MATH-01 removed for FACT, left in place here. Similarly,
`op_adv_sum` / `op_adv_sumab` / `op_adv_rsum` accumulate with `checked_add`, which is
correct, but `op_adv_max/min/maxab/rmaxab/rnrm` compare and store via `.inner()` /
`HpNum::rounded(Decimal)` — fine for in-range matrices but they silently lose any
large-exponent element down to its Decimal `inner()` (which for `exponent != 0` is only the
[1,10) mantissa, dropping the exponent entirely → wrong comparison and wrong stored max).

**Fix:** In `op_adv_fnrm`, replace the final `Decimal::from_f64(...).map(HpNum::rounded)`
with `HpNum::from_f64(norm_f64).ok_or(HpError::Overflow)` to match the FACT post-compute
wall. In the reduction ops, compare via `HpNum::to_f64()` (full value) rather than
`.inner()` (mantissa-only for large-exponent elements), and store results via the
exponent-aware path. Add a test with a matrix element built from `HpNum::from_f64(1e40)`
to confirm MAX/MIN/FNRM don't collapse the exponent.

## Info

### IN-01: `MAX`/`MIN`/`MAXAB`/`RMAXAB`/`RNRM` compare on `.inner()` — document or assert exponent-0

**File:** `hp41-core/src/ops/advantage/matrix_ops.rs` (`op_adv_max`, `op_adv_min`, `op_adv_maxab`, `op_adv_rmaxab`, `op_adv_rnrm`)
**Issue:** These reductions compare `elem.inner()` (mantissa Decimal), which for
`exponent != 0` elements is only the normalized [1,10) mantissa — see WR-05. Even if
large-exponent matrix elements are considered out-of-scope today, the code gives no
indication that it assumes exponent-0 data.
**Fix:** Add a doc note (and ideally a `debug_assert` or to_f64-based comparison) stating
the reduction assumes Decimal-range elements, so the assumption is visible if X-MEM /
Advantage ever store large-exponent values.

### IN-02: `Display for HpNum` large-exponent format `{m}e{e}` is internal-only — confirm no UI leak

**File:** `hp41-core/src/num.rs` (`Display`: `write!(f, "{}e{}", self.mantissa, self.exponent)`)
**Issue:** For `exponent != 0`, `Display` emits e.g. `1.711e98` (lowercase `e`, no padding),
which is NOT the HP-41 display form. `format_hpnum` is the correct UI path and routes
large-exponent values to `format_sci_large`, but `op_adv_mp` (matrix print) formats
elements with `format!("R{}C{}= {}", ..., val)` using this `Display` directly. A
large-exponent matrix element would print in the internal `1.711e98` form into the print
buffer, diverging from HP-41 output.
**Fix:** Either route `op_adv_mp` element formatting through `format_hpnum`, or document
that matrix elements are always Decimal-range. Low priority given current Advantage data is
in-range.

### IN-03: Stale/contradictory doc comments in `MSIJ` and stack-drop helpers

**File:** `hp41-core/src/ops/advantage/matrix_ops.rs` (`op_adv_msij` doc block lines ~266-279)
**Issue:** The `MSIJ` doc comment contains visible deliberation residue ("Wait — per OM
convention…", "Actually per typical HP-41 pattern…") rather than a settled contract. While
not a behavioral defect (the implementation is consistent: Z=value, Y=row, X=col), the
comment undermines confidence in the spec and was not cleaned up. Also several stack-drop
sequences set `state.stack.z = state.stack.t.clone()` after already moving `t`, which is
correct but unobvious; a one-line note would help.
**Fix:** Replace the deliberation prose with the final settled semantics; this file was
touched in the MATH-01 ripple and is a cheap cleanup.

---

_Reviewed: 2026-06-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
