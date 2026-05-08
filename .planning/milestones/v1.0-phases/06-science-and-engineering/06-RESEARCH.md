# Phase 6: Science & Engineering - Research

**Researched:** 2026-05-07
**Domain:** Rust statistics engine + HMS/H time-angle conversion, integrated into existing hp41-core/hp41-cli Cargo workspace
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Full HP-41 statistics suite included: Σ+, Σ−, MEAN, SDEV, L.R., YHAT, CORR, CLΣSTAT. All eight ops in scope.
- **D-02:** CLΣSTAT zeros R01–R06.
- **D-03:** Σ registers are R01–R06 in existing `regs: Vec<HpNum>` — no new CalcState field. R01=Σx², R02=Σx, R03=n, R04=Σy², R05=Σy, R06=Σxy.
- **D-04:** Σ register conflict: silent overwrite. Match HP-41 hardware — R01–R06 are dual-use. No warning, no blocking.
- **D-05:** L.R. returns slope m in Y, intercept b in X. YHAT reads x from X, returns ŷ in X. CORR returns correlation coefficient r in X.
- **D-06:** Invalid HMS validation: return `HpError::InvalidInput` when seconds ≥ 60 or minutes ≥ 60.
- **D-07:** HMS+ and HMS− handle base-60 carry/borrow with rust_decimal arithmetic (string-split field extraction; no floor()/fmod()).
- **D-08:** Negative HMS values: sign applies to whole value. →HMS and HMS→ handle negative X correctly.
- **D-09:** 12 new TUI key bindings (see table below). Existing keys unchanged.

| Op | Key | Mnemonic |
|----|-----|----------|
| Σ+ | `z` | Z ≈ Σ |
| Σ− | `Z` | uppercase secondary |
| MEAN | `m` | mean |
| SDEV | `d` | deviation |
| YHAT | `y` | ŷ prediction |
| L.R. | `R` | Regression |
| CORR | `O` | cOrrelation |
| CLΣSTAT | `V` | Void/clear stats |
| HMS→ | `h` | hms→decimal |
| →HMS | `f` | Format as hms |
| HMS+ | `j` | HMS addition |
| HMS− | `J` | uppercase subtraction |

### Claude's Discretion

- HMS arithmetic internals: rust_decimal string-based H.MMSS parsing (split at decimal, extract hours/minutes/seconds as integers — same as ISG/DSE).
- MEAN stack semantics: X = x̄, Y = ȳ. SDEV: X = σx, Y = σy (sample std dev, n-1 denominator).
- Σ+ pushes n (count) into X after accumulating. Σ− pushes n into X after removing.
- Program-mode recording: all 12 new ops record normally to `program: Vec<Op>` when `prgm_mode = true`.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SCI-01 | User can perform statistics operations: Σ+, Σ−, MEAN, SDEV, and linear regression using Σ registers (R01–R06) | D-01 through D-05 are locked decisions; formulas verified in Specific Requirements section of CONTEXT.md; `regs` Vec already holds 100 registers (indices 1–6 ready) |
| SCI-02 | User can perform HMS/H conversions: →HMS, HMS→, HMS+, HMS− | D-06 through D-08 are locked decisions; `HpError::InvalidInput` must be added; string-split pattern verified from ISG/DSE implementation in program.rs |

</phase_requirements>

---

## Summary

Phase 6 adds 12 new operations across two new files (`stats.rs`, `hms.rs`) to `hp41-core/src/ops/`, wires them into the `Op` enum and `dispatch()` match arm in `mod.rs`, and exposes 12 key bindings in `hp41-cli`. The implementation is self-contained: no new CalcState fields are needed, no new dependencies are required, and all arithmetic uses the existing `rust_decimal 1.41` with `maths` + `serde-with-str` features already enabled.

The statistics engine reads/writes Σ registers at fixed indices (1–6) in the existing `state.regs: Vec<HpNum>`. The HMS conversion engine follows the identical string-split pattern already proven in ISG/DSE (`parse_counter()` in `program.rs`). The only missing piece in the current codebase is the `HpError::InvalidInput` variant, which must be added to `error.rs` before HMS validation can compile.

The most important planning pitfall is a key binding conflict: `d` (planned for SDEV) and `f` (planned for →HMS) are hardcoded in `app.rs` before `key_to_op()` is called — `d` cycles DEG/RAD/GRAD and `f` cycles FIX/SCI/ENG. Both CANNOT be routed through `key_to_op()` using those letter values without changing the app.rs intercept order. The planner must decide: reassign SDEV and →HMS to unused keys, or move `d`/`f` handling after the new op dispatch. CONTEXT.md D-09 says "existing keys unchanged," which means the simplest fix is choosing non-conflicting keys for SDEV and →HMS.

**Primary recommendation:** Implement `op_sigma_plus`, `op_sigma_minus`, stat result ops, and HMS ops as pure functions following the `unary_result()`/`binary_result()` pattern; add `HpError::InvalidInput` to `error.rs`; resolve `d`/`f` key conflicts by selecting non-conflicting keys for SDEV and →HMS.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Σ register accumulation | `hp41-core` lib | — | Pure arithmetic; no UI state involved |
| Statistics formula evaluation | `hp41-core` lib | — | Stateless math functions reading `state.regs` |
| HMS encode/decode | `hp41-core` lib | — | String-split field extraction; pure arithmetic |
| HMS validation | `hp41-core` lib | — | `HpError::InvalidInput` returned on invalid minutes/seconds |
| Op enum extension | `hp41-core/ops/mod.rs` | — | All new variants must appear in Op enum + dispatch() |
| Key bindings | `hp41-cli/keys.rs` | `app.rs` | New ops added to `key_to_op()`; `d`/`f` conflicts handled in `app.rs` |
| Help text | `hp41-cli/help_data.rs` | — | HELP_DATA static array entries for all 12 ops |
| Program display | `hp41-cli/prgm_display.rs` | — | `op_display_name()` must cover all 12 new Op variants |

---

## Standard Stack

### Core (already installed — no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rust_decimal | 1.41 [VERIFIED: Cargo.toml] | Decimal arithmetic; HpNum inner type | Already workspace dep; `maths` feature provides sqrt needed for CORR formula |
| thiserror | 2.0 [VERIFIED: Cargo.toml] | Derive macro for HpError variants | Already workspace dep; `InvalidInput` variant added with zero new boilerplate |
| serde | 1.x [VERIFIED: Cargo.toml] | Serialize/Deserialize for Op enum | Already workspace dep; new Op variants need `#[derive(Serialize, Deserialize)]` |

### No new dependencies required
All Phase 6 math (square root for CORR r formula, decimal arithmetic, string parsing) is covered by the existing `rust_decimal` with the `maths` feature already enabled in `hp41-core/Cargo.toml`. [VERIFIED: Cargo.toml]

**Installation:** No `cargo add` step needed.

---

## Architecture Patterns

### System Architecture Diagram

```
User keypress (hp41-cli)
        │
        ▼
app.rs::handle_key()
        │
        ├─► d/f intercepted before key_to_op() [CONFLICT: d=angle cycle, f=fmt cycle]
        │           (must not conflict with SDEV/→HMS bindings)
        │
        ├─► key_to_op(key, app) → Op::SigmaPlus / Op::HmsToH / ... (12 new variants)
        │
        ▼
hp41_core::ops::dispatch(state, op)
        │
        ├─► flush_entry_buf(state)  [always first]
        │
        ├─► prgm_mode gate          [record to program Vec if true]
        │
        └─► match op {
                Op::SigmaPlus  → stats::op_sigma_plus(state)
                Op::SigmaMinus → stats::op_sigma_minus(state)
                Op::Mean       → stats::op_mean(state)
                Op::Sdev       → stats::op_sdev(state)
                Op::LR         → stats::op_lr(state)
                Op::Yhat       → stats::op_yhat(state)
                Op::Corr       → stats::op_corr(state)
                Op::ClSigmaStat→ stats::op_cl_sigma_stat(state)
                Op::HmsToH     → hms::op_hms_to_h(state)
                Op::HToHms     → hms::op_h_to_hms(state)
                Op::HmsAdd     → hms::op_hms_add(state)
                Op::HmsSub     → hms::op_hms_sub(state)
            }
        │
        ▼
state.regs[1..=6]  (Σ registers, pre-existing Vec<HpNum>)
state.stack.x / .y (results written here)
```

### Recommended Project Structure
```
hp41-core/src/ops/
├── mod.rs          # Add 12 Op variants + 12 dispatch arms + 2 pub mod declarations
├── stats.rs        # NEW — all 8 statistics ops
├── hms.rs          # NEW — all 4 HMS ops
├── arithmetic.rs   # Unchanged
├── math.rs         # Unchanged
└── registers.rs    # Unchanged

hp41-core/src/
└── error.rs        # Add HpError::InvalidInput variant

hp41-core/tests/
├── stats_tests.rs  # NEW — SCI-01 test coverage
└── hms_tests.rs    # NEW — SCI-02 test coverage

hp41-cli/src/
├── keys.rs         # Append 10 new bindings to key_to_op() (not d/f — see conflict section)
├── help_data.rs    # Add "=== Science & Engineering ===" category + 12 entries
└── prgm_display.rs # Add 12 arms to op_display_name()
```

### Pattern 1: Σ Register Accumulation (stats.rs)

Σ+ reads X (and Y for two-variable stats), writes to regs[1..=6], pushes count n into X.
Uses `enter_number()` to write result, then `apply_lift_effect(Enable)` explicitly.
Does NOT use `binary_result()` — stack drop behavior differs: Y remains on stack after Σ+.
[VERIFIED: CONTEXT.md code_context section, HP-41 Owner's Handbook layout]

```rust
// Source: pattern derived from op_sto/op_rcl in registers.rs + CONTEXT.md D-03
pub fn op_sigma_plus(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();

    // Accumulate into Σ registers (1-indexed, 0-based Vec)
    // R01=Σx², R02=Σx, R03=n, R04=Σy², R05=Σy, R06=Σxy
    state.regs[1] = state.regs[1].checked_add(&x.checked_sq()?)?;   // Σx²
    state.regs[2] = state.regs[2].checked_add(&x)?;                   // Σx
    state.regs[3] = state.regs[3].checked_add(&HpNum::from(1))?;     // n++
    state.regs[4] = state.regs[4].checked_add(&y.checked_sq()?)?;   // Σy²
    state.regs[5] = state.regs[5].checked_add(&y)?;                   // Σy
    state.regs[6] = state.regs[6].checked_add(&x.checked_mul(&y)?)?; // Σxy

    // Push count n into X (HP-41 hardware behavior)
    let n = state.regs[3].clone();
    state.stack.lift_enabled = false; // Σ+ overwrites X, does not lift
    state.stack.x = n;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**Important:** Y, Z, T are NOT modified by Σ+ (unlike `binary_result()` which drops Y). LASTX is NOT updated by Σ+/Σ− (HP-41 hardware: accumulating ops do not save LASTX). [ASSUMED — verify against HP-41 Owner's Handbook page 12-3]

### Pattern 2: Statistics Result Ops (MEAN, SDEV, L.R., YHAT, CORR)

Result ops read Σ registers, compute derived values, and push results to stack.
MEAN and SDEV push two values (X and Y), requiring careful stack manipulation.
L.R. also pushes two values. CORR and YHAT push one value.

For **two-result ops** (MEAN, SDEV, L.R.): push X result last (so it ends up in stack.x).
Use `enter_number()` + `apply_lift_effect(Enable)` for each push.

```rust
// Source: pattern from op_rcl in registers.rs — enter_number + lift Enable
pub fn op_mean(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.regs[3].inner();
    if n.is_zero() {
        return Err(HpError::InvalidOp); // no data
    }
    let n_hp = state.regs[3].clone();
    let x_mean = state.regs[2].checked_div(&n_hp)?; // x̄ = Σx / n
    let y_mean = state.regs[5].checked_div(&n_hp)?; // ȳ = Σy / n

    // Push ȳ first (goes to Y), then x̄ (goes to X)
    state.stack.lift_enabled = true;
    enter_number(state, y_mean);          // Y ← ȳ
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, x_mean);          // X ← x̄ (lifted Y to Z, etc.)
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**SDEV formula:** σ = sqrt((n·Σx² − (Σx)²) / (n·(n−1))). Sample std dev (n-1 denominator). [VERIFIED: CONTEXT.md Specific Requirements]

**L.R. formula:**
- slope m = (n·Σxy − Σx·Σy) / (n·Σx² − (Σx)²)
- intercept b = ȳ − m·x̄
- Returns m in Y, b in X [VERIFIED: CONTEXT.md D-05]

**CORR formula:** r = (n·Σxy − Σx·Σy) / sqrt((n·Σx² − (Σx)²)·(n·Σy² − (Σy)²)) [VERIFIED: CONTEXT.md Specific Requirements]

**Domain guard for CORR/SDEV:** if denominator is zero (n < 2 or all x-values identical), return `HpError::Domain`. [ASSUMED]

### Pattern 3: HMS Field Extraction (hms.rs)

H.MMSS format: integer part = hours, decimal part = MMSS where MM = minutes (2 digits), SS = seconds (2 digits). Pad decimal part RIGHT to 4 chars (not 5 like ISG — HMS has only MM and SS, not FFFDD).
[VERIFIED: CONTEXT.md Claude's Discretion — "same pattern as ISG/DSE counter field extraction"]

```rust
// Source: parse_counter() in program.rs — adapted for H.MMSS format
fn parse_hms(n: &HpNum) -> Result<(i64, i64, i64, bool), HpError> {
    // Returns (hours, minutes, seconds, is_negative)
    let is_neg = n.inner().is_sign_negative();
    let abs_val = if is_neg { HpNum(n.inner().abs()) } else { n.clone() };
    let s = abs_val.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let hours: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // Right-pad fraction to 4 chars (MMSS)
    let frac_padded = format!("{:0<4}", frac_part);
    let frac_padded = if frac_padded.len() > 4 { frac_padded[..4].to_string() } else { frac_padded };
    let minutes: i64 = frac_padded[..2].parse().map_err(|_| HpError::InvalidOp)?;
    let seconds: i64 = frac_padded[2..4].parse().map_err(|_| HpError::InvalidOp)?;
    Ok((hours, minutes, seconds, is_neg))
}
```

**Validation:** after parsing, if minutes ≥ 60 or seconds ≥ 60, return `Err(HpError::InvalidInput)`. [VERIFIED: CONTEXT.md D-06]

**HMS→ (op_hms_to_h):** convert H.MMSS → decimal hours = hours + minutes/60 + seconds/3600. Use rust_decimal arithmetic; never f64. [VERIFIED: CONTEXT.md D-07]

**→HMS (op_h_to_hms):** convert decimal hours → H.MMSS.
```
total_seconds_dec = x * 3600
hours = trunc(total_seconds_dec / 3600)
remaining_seconds = total_seconds_dec - hours * 3600
minutes = trunc(remaining_seconds / 60)
seconds = remaining_seconds - minutes * 60
```
Reconstruct as: `format!("{}.{:02}{:02}", hours, minutes, seconds)` → parse as Decimal → HpNum::rounded.

**HMS+ (op_hms_add):** convert both operands to decimal hours via HMS→, add, convert result back via →HMS.
**HMS− (op_hms_sub):** same pattern, subtract instead. [VERIFIED: CONTEXT.md D-07]

**Negative HMS:** sign tracked separately; apply sign to final result. [VERIFIED: CONTEXT.md D-08]

### Pattern 4: HpError::InvalidInput Addition

`error.rs` currently has 5 variants: Overflow, DivideByZero, InvalidOp, Domain, CallDepth.
`HpError::InvalidInput` does NOT exist yet. [VERIFIED: grep confirmed no `InvalidInput` in error.rs]

Add:
```rust
#[error("invalid input")]
InvalidInput,
```

**Warning:** `tests.rs` has a test `hperror_has_four_variants` that asserts exactly 4 variants (pre-Phase 3 CallDepth addition). This test will need updating when `InvalidInput` is added. [VERIFIED: hp41-core/src/tests.rs line 14]

Also `hperror_display_messages` test hardcodes 4 messages. Both tests need adding the new variant or relaxing to "at least N variants." [VERIFIED: hp41-core/src/tests.rs]

### Pattern 5: Op Enum Extension (mod.rs)

The Op enum has a comment block for each phase. Add new variants after `Op::AlphaBackspace` in a `// ── Science & Engineering (Phase 6) ──` block.

Every new variant MUST appear in BOTH:
1. The `Op` enum definition
2. The `dispatch()` match arm in `dispatch()` 
3. The `execute_op()` match arm in `program.rs` (the interpreter's inner loop)
4. `op_display_name()` in `prgm_display.rs`

Compiler exhaustiveness checking catches 1, 2, and 3. Item 4 (prgm_display.rs) is NOT exhaustive-checked and is a known Phase 5 pitfall. [VERIFIED: prgm_display.rs is a complete match with no `_ =>` wildcard]

### Anti-Patterns to Avoid

- **Using `binary_result()` for Σ+ / Σ−:** `binary_result()` drops Y and saves LASTX. Σ+ does NOT drop Y and does NOT save LASTX. Write custom stack manipulation.
- **Using floor()/fmod() for HMS field extraction:** Use string-split at decimal point. Same prohibition as ISG/DSE. [VERIFIED: CONTEXT.md D-07, ADR-001]
- **Using f64 for HMS arithmetic:** rust_decimal throughout; f64 round-trip only for inverse trig (not applicable here).
- **Forgetting `execute_op()` in program.rs:** New Op variants added to `dispatch()` also need arms in `execute_op()` — the private interpreter loop. Missing this causes `HpError::InvalidOp` when new ops are used inside recorded programs.
- **Forgetting `op_display_name()` in prgm_display.rs:** Not compiler-enforced; missing arms cause compile error only because the existing match has no wildcard.
- **Using n instead of n-1 for SDEV:** HP-41 uses sample standard deviation (divide by n−1). [VERIFIED: CONTEXT.md Specific Requirements]
- **HMS reconstruction using format! rounding:** construct the H.MMSS string using integer minutes/seconds, not via float arithmetic; parse back to Decimal.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Decimal sqrt for CORR | Custom Newton's method | `HpNum::checked_sqrt()` (wraps `rust_decimal` MathematicalOps) | Already in num.rs; handles domain error |
| String parsing for H.MMSS | `regex` crate or custom lexer | `str.find('.')` + slice indexing | Already proven in `parse_counter()`; zero new deps |
| Stack push with lift | Custom lift logic | `enter_number(state, val)` + `apply_lift_effect(state, LiftEffect::Enable)` | Existing pattern; correct for all edge cases |
| Register reads | Direct Vec indexing without bounds | `state.regs[idx]` — Vec is always 100 elements (initialized in CalcState::new()) | Safe: R01–R06 indices 1–6 are always valid |
| Two-value result push | Complex stack juggling | `enter_number()` twice with `lift_enabled = true` before each call | Follows op_rcl pattern; produces correct X/Y ordering |

**Key insight:** Every hard problem in this phase (decimal math, string parsing, stack manipulation) has a proven solution already in the codebase. Phase 6 is assembly, not invention.

---

## Common Pitfalls

### Pitfall 1: `d` and `f` Key Binding Conflicts
**What goes wrong:** `d` (planned for SDEV) and `f` (planned for →HMS) are intercepted in `app.rs` before `key_to_op()` is called — `d` at line 291 cycles angle mode, `f` at line 302 cycles format. Any binding of these letters in `key_to_op()` is dead code.
**Why it happens:** app.rs has special-case early returns for `d` and `f` before the `key_to_op()` routing at line 337.
**How to avoid:** Use different letter keys for SDEV and →HMS. Candidates: SDEV → `D` (uppercase, currently unused), →HMS → already has `h` for HMS→ so consider `F` (uppercase `F` is unbound). OR: add SDEV and →HMS handling in app.rs before the existing `d`/`f` intercepts, which changes behavior of `d`/`f` depending on a mode flag (more complex).
**Warning signs:** Adding `KeyCode::Char('d') => Some(Op::Sdev)` to `key_to_op()` compiles but is never reached.

### Pitfall 2: Missing `execute_op()` Arms in program.rs
**What goes wrong:** New ops dispatch correctly interactively but return `HpError::InvalidOp` when used inside a recorded program.
**Why it happens:** `dispatch()` in mod.rs and `execute_op()` in program.rs are separate match expressions. Compiler only checks exhaustiveness within each match independently.
**How to avoid:** Every new Op variant added to `dispatch()` must also be added to `execute_op()` in program.rs.
**Warning signs:** Tests that run a program containing a new op fail with `InvalidOp`.

### Pitfall 3: HpError::InvalidInput Missing — compile failure
**What goes wrong:** `hms.rs` references `HpError::InvalidInput` which does not exist yet in `error.rs`.
**Why it happens:** Phase 5 and earlier added variants but `InvalidInput` was never needed until Phase 6.
**How to avoid:** Add `InvalidInput` to `error.rs` in Wave 0 (first plan) before any HMS code compiles.
**Warning signs:** `error[E0599]: no variant or associated item named 'InvalidInput'` on first compile.

### Pitfall 4: tests.rs Variant Count Assertions
**What goes wrong:** `hperror_has_four_variants` in `hp41-core/src/tests.rs` and `hperror_display_messages` assert exactly 4 error variants (or display messages). Adding `InvalidInput` breaks these.
**Why it happens:** Tests were written for Phase 1 baseline; `CallDepth` (Phase 3) was added without updating these tests. The test still says "four variants" but there are already 5.
**How to avoid:** Update both tests when adding `InvalidInput` — add it to the constructible-variants test and the display-messages test.
**Warning signs:** `test hperror_has_four_variants ... FAILED` with `assert_ne` failing on some variant.

### Pitfall 5: L.R. Stack Output Convention
**What goes wrong:** L.R. returns slope m in Y and intercept b in X (HP-41 convention, D-05). Reversing the order (m in X, b in Y) is wrong.
**Why it happens:** Natural code ordering pushes the first-computed value to X, which is m; then pushes b, landing b in X. The order must be: push m (goes to Y after next lift), then push b (goes to X).
**How to avoid:** Push slope m first, intercept b second. Final stack: X = intercept b, Y = slope m.
**Warning signs:** Round-trip test `slope/intercept` values are swapped.

### Pitfall 6: Division by Zero / Small N in Stats
**What goes wrong:** MEAN with n=0, SDEV with n<2, L.R./CORR with insufficient data cause divide-by-zero or Domain error instead of meaningful HpError.
**Why it happens:** `checked_div()` returns `HpError::DivideByZero`; sqrt of negative returns `HpError::Domain`.
**How to avoid:** Guard n=0 before MEAN (return `HpError::InvalidOp`); guard n<2 before SDEV (same); guard denominator=0 before CORR.
**Warning signs:** `unwrap()` panics or wrong error types in stat tests.

### Pitfall 7: HMS Fraction Padding Direction
**What goes wrong:** `format!("{:0>4}", frac_part)` pads LEFT (produces "0012" from "12") where RIGHT-pad is needed (produces "1200" from "12" meaning 12 minutes, 00 seconds).
**Why it happens:** Same trap as ISG/DSE. The `parse_counter()` in program.rs uses `{:0<5}` (left-align = right-pad) and the comment says "CRITICAL: left-align". HMS needs `{:0<4}`.
**How to avoid:** Always use `{:0<N}` (less-than = left-align = right-pad with zeros). Verified in `parse_counter()` source.
**Warning signs:** HMS→ of `1.3045` does not produce `1.5125`.

### Pitfall 8: Negative HMS Sign Handling
**What goes wrong:** For negative HMS (e.g., -1.3045), the fractional part `.3045` has magnitude, not the sign. Extracting minutes/seconds from the absolute value and then applying sign at the end is required.
**Why it happens:** `Decimal::to_string()` of `-1.3045` produces `"-1.3045"` — the integer part `int_part.parse::<i64>()` succeeds as -1, but minutes/seconds must be parsed from `"3045"` (not `"-3045"`).
**How to avoid:** Extract sign flag first (`n.inner().is_sign_negative()`), work with absolute value throughout, apply sign to final result.
**Warning signs:** `parse::<i64>()` fails on the `abs()` path or minutes field parses as negative.

---

## Code Examples

### Register index convention
```rust
// Source: state.rs CalcState::new() — regs: vec![HpNum::zero(); 100]
// R01–R06 are at indices 1–6 (0-indexed Vec, R00 at index 0)
// Access pattern (verified from registers.rs op_sto/op_rcl):
state.regs[1]  // R01 = Σx²
state.regs[2]  // R02 = Σx
state.regs[3]  // R03 = n (count)
state.regs[4]  // R04 = Σy²
state.regs[5]  // R05 = Σy
state.regs[6]  // R06 = Σxy
```

### HpNum checked_sqrt used in CORR
```rust
// Source: num.rs checked_sqrt()
// Returns HpError::Domain if self < 0
let denom = some_decimal_hpnum.checked_sqrt()?;
```

### Decimal arithmetic chain for CORR numerator
```rust
// Source: pattern from existing HpNum methods
let n = &state.regs[3];
let sum_xy = &state.regs[6];
let sum_x  = &state.regs[2];
let sum_y  = &state.regs[5];
let sum_x2 = &state.regs[1];
let sum_y2 = &state.regs[4];

let numerator = n.checked_mul(sum_xy)?
    .checked_sub(&sum_x.checked_mul(sum_y)?)?;
let denom_x = n.checked_mul(sum_x2)?
    .checked_sub(&sum_x.checked_sq()?)?;
let denom_y = n.checked_mul(sum_y2)?
    .checked_sub(&sum_y.checked_sq()?)?;
let denom = denom_x.checked_mul(&denom_y)?.checked_sqrt()?;
let r = numerator.checked_div(&denom)?;
```

### Round-trip HMS accuracy test (from CONTEXT.md success criterion)
```rust
// 1.3045 (1h 30m 45s) → HMS→ → should give 1.5125 (1.5125h = 1h 30m 45s)
// 1.3045 H.MMSS = 1h + 30min + 45sec = 1 + 30/60 + 45/3600 = 1.5125
#[test]
fn test_hms_to_h_round_trip() {
    let mut s = CalcState::new();
    push_dec(&mut s, "1.3045");
    dispatch(&mut s, Op::HmsToH).unwrap();
    let expected = Decimal::from_str("1.5125").unwrap();
    assert_eq!(s.stack.x.inner(), expected);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| f64 for decimal stats | rust_decimal throughout | Phase 1 ADR-001 | No binary float rounding; 10-digit exact |
| floor()/fmod() for counter fields | string-split at decimal | Phase 1 ADR-001 | Proven in ISG/DSE; same technique for HMS |
| Separate CalcState fields for Σ data | Use existing regs[1..=6] | Phase 6 D-03 | No state schema change; no migration needed |

**Deprecated/outdated:**
- None applicable to this phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Σ+ does not save LASTX (HP-41 hardware behavior) | Pattern 1 code example | If HP-41 saves LASTX on Σ+, tests will fail; check HP-41 Owner's Handbook p. 12-3 |
| A2 | SDEV Domain guard: return HpError::Domain when denominator is zero | Pattern 2 text | Could be HpError::InvalidOp instead; functional either way |
| A3 | CORR Domain guard: return HpError::Domain when denominator is zero | Pattern 2 text | Same as A2 |
| A4 | Y/Z/T are not modified by MEAN/SDEV/L.R./YHAT/CORR beyond the two pushed values | Pattern 2 code | HP-41 may clear higher stack; would affect test assertions |
| A5 | `HpNum::checked_sq()` is usable for Σx² accumulation (x * x) | Pattern 1 code | checked_sq calls checked_mul internally — verified; no domain restriction for squares |

**Note on A5:** `checked_sq()` is verified in num.rs as `self.checked_mul(self)` — no risk. Included for completeness.

---

## Open Questions (RESOLVED)

1. **SDEV and →HMS key binding conflict**
   - What we know: `d` cycles angle mode, `f` cycles format mode — both intercepted in app.rs before key_to_op()
   - What's unclear: CONTEXT.md D-09 assigns `d` to SDEV and `f` to →HMS; D-09 also says "existing keys unchanged"
   - Recommendation: Use `D` (uppercase) for SDEV and `F` (uppercase) for →HMS. Both are currently unbound. Consistent with convention: uppercase = shifted/secondary op. This is within Claude's Discretion (user delegated binding choices to Claude).
   - **RESOLVED:** Use `D` (uppercase) for SDEV and `F` (uppercase) for →HMS. Codified in Plan 03 key binding table: `KeyCode::Char('D') => Some(Op::Sdev)` and `KeyCode::Char('F') => Some(Op::HToHms)`. This is within Claude's Discretion per CONTEXT.md.

2. **LASTX behavior for Σ+/Σ−**
   - What we know: CONTEXT.md does not specify LASTX for accumulating ops
   - What's unclear: HP-41 hardware — does Σ+ save LASTX?
   - Recommendation: Do NOT save LASTX (consistent with STO behavior in registers.rs). Mark as [ASSUMED] in plan.
   - **RESOLVED (ASSUMED):** Σ+/Σ− do NOT save LASTX. Codified in Plan 02 Task 1 action comment and Plan 03 stats_tests.rs `test_sigma_plus_does_not_save_lastx` test. Assumption sourced from analogy with STO (registers.rs); HP-41 handbook not consulted. If incorrect, the behavioral test will fail and the fix is trivial (remove the LASTX guard in op_sigma_plus).

3. **Error type for insufficient data in stats ops**
   - What we know: HP-41 displays "DATA ERROR" for Σ functions with n=0
   - What's unclear: Should the emulator return HpError::InvalidOp, HpError::Domain, or HpError::InvalidInput?
   - Recommendation: Return `HpError::InvalidOp` for insufficient data (same as "nonsensical operation in context"). Reserve `HpError::InvalidInput` exclusively for HMS field-range validation (D-06 states it explicitly). Reserve `HpError::Domain` for mathematical domain errors (sqrt of negative in CORR denominator).
   - **RESOLVED:** Return `HpError::InvalidOp` for n=0 (MEAN), n<2 (SDEV), and all-identical-x (L.R./CORR denominator=0). Return `HpError::Domain` only from `checked_sqrt()` propagation when the radicand is negative. Codified in Plan 02 Tasks 1 and 2 action text and Plan 03 stats_tests.rs assertions.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies identified — all required libraries already in Cargo workspace, no new CLI tools, services, or runtimes required).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`) + proptest + insta (already installed) |
| Config file | None — standard `cargo test` |
| Quick run command | `cargo test --workspace -p hp41-core stats hms` |
| Full suite command | `just test` (runs all 337+ tests) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCI-01 | Σ+ accumulates into R01–R06 correctly | unit | `cargo test -p hp41-core sigma_plus` | ❌ Wave 0 |
| SCI-01 | Σ− removes from R01–R06 correctly | unit | `cargo test -p hp41-core sigma_minus` | ❌ Wave 0 |
| SCI-01 | MEAN returns x̄ in X, ȳ in Y | unit | `cargo test -p hp41-core mean` | ❌ Wave 0 |
| SCI-01 | SDEV uses sample std dev (n-1) | unit | `cargo test -p hp41-core sdev` | ❌ Wave 0 |
| SCI-01 | L.R. returns slope m in Y, intercept b in X | unit | `cargo test -p hp41-core lr` | ❌ Wave 0 |
| SCI-01 | YHAT returns ŷ in X | unit | `cargo test -p hp41-core yhat` | ❌ Wave 0 |
| SCI-01 | CORR returns r in X | unit | `cargo test -p hp41-core corr` | ❌ Wave 0 |
| SCI-01 | CLΣSTAT zeros R01–R06 | unit | `cargo test -p hp41-core cl_sigma_stat` | ❌ Wave 0 |
| SCI-01 | LiftEffect::Enable on result ops, Neutral on accumulating ops | unit | `cargo test -p hp41-core stats_lift` | ❌ Wave 0 |
| SCI-02 | HMS→ converts 1.3045 → 1.5125 (canonical round-trip) | unit | `cargo test -p hp41-core hms_to_h` | ❌ Wave 0 |
| SCI-02 | →HMS converts 1.5125 → 1.3045 | unit | `cargo test -p hp41-core h_to_hms` | ❌ Wave 0 |
| SCI-02 | HMS+ handles base-60 carry (e.g., 1.4500 + 0.2000 = 2.0500) | unit | `cargo test -p hp41-core hms_add` | ❌ Wave 0 |
| SCI-02 | HMS− handles base-60 borrow | unit | `cargo test -p hp41-core hms_sub` | ❌ Wave 0 |
| SCI-02 | Invalid HMS returns HpError::InvalidInput for min≥60 or sec≥60 | unit | `cargo test -p hp41-core hms_invalid` | ❌ Wave 0 |
| SCI-02 | Negative HMS handled correctly | unit | `cargo test -p hp41-core hms_negative` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-core 2>&1 | tail -3`
- **Per wave merge:** `just test`
- **Phase gate:** `just ci` (lint + test + coverage ≥ 80%) must be green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `hp41-core/tests/stats_tests.rs` — covers SCI-01 (all 8 stats ops)
- [ ] `hp41-core/tests/hms_tests.rs` — covers SCI-02 (all 4 HMS ops + validation)
- [ ] Update `hp41-core/src/tests.rs` — `hperror_has_four_variants` and `hperror_display_messages` need `InvalidInput` variant added

---

## Security Domain

The phase adds pure mathematical computation to a local desktop calculator emulator. No network access, no file I/O beyond existing persistence, no user authentication, no cryptography, no external input surface beyond keyboard.

**ASVS assessment:** Not applicable — this phase introduces no new security surface. All security properties are inherited from the existing architecture (local-only, no network, serde-only serialization already in place).

---

## Project Constraints (from CLAUDE.md)

| Directive | Impact on Phase 6 |
|-----------|-------------------|
| `hp41-core` must never depend on `hp41-cli` or `hp41-gui` | stats.rs and hms.rs go in hp41-core; no hp41-cli imports |
| All commits via `/git-workflow:commit --with-skills` | No direct `git commit` — use slash command |
| Never call `cargo` directly — use `just` | All test/build commands in tasks use `just test`, `just ci`, `just build` |
| Rust stable 1.78+ | No unstable features; existing toolchain |
| ratatui 0.30 + crossterm 0.29 | TUI unchanged in this phase; bindings added to existing key routing |
| proptest + insta for tests | Stats tests may benefit from proptest for formula coverage |
| ≥80% coverage gate on hp41-core | New code in stats.rs and hms.rs must be tested; gate already at 81.62% |
| Zero panics in hp41-core | All checked_* arithmetic; no unwrap() in stats.rs or hms.rs |

---

## Sources

### Primary (HIGH confidence)
- `hp41-core/src/ops/program.rs` — `parse_counter()` function: verified string-split pattern and padding direction for ISG/DSE counter fields
- `hp41-core/src/ops/mod.rs` — Op enum exhaustiveness rule and dispatch() pattern; all new variants must appear in both enum and match
- `hp41-core/src/ops/registers.rs` — register access pattern (op_sto, op_rcl): index arithmetic and LiftEffect handling
- `hp41-core/src/error.rs` — confirmed `InvalidInput` variant does NOT yet exist
- `hp41-cli/src/app.rs` lines 291–310 — confirmed `d` and `f` are intercepted before `key_to_op()`, creating binding conflicts
- `.planning/phases/06-science-and-engineering/06-CONTEXT.md` — all locked decisions, formulas, and register layout

### Secondary (MEDIUM confidence)
- HP-41 Owner's Handbook (referenced in CONTEXT.md Specific Requirements) — Σ register layout R01–R06, L.R./CORR/SDEV formulas cited in CONTEXT.md D-05 and Specific Requirements section

### Tertiary (LOW confidence — marked ASSUMED in research)
- LASTX behavior for Σ+/Σ− — assumed "not saved" based on analogy with STO; needs HP-41 handbook verification

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified from Cargo.toml; no new dependencies
- Architecture: HIGH — codebase read directly; patterns verified from existing implementations
- Pitfalls: HIGH — key binding conflict verified from app.rs source; parse_counter() padding direction verified from program.rs source
- Formulas: HIGH — copied verbatim from CONTEXT.md Specific Requirements (user-locked)

**Research date:** 2026-05-07
**Valid until:** 2026-07-01 (stable domain; Rust crate versions unlikely to change)
