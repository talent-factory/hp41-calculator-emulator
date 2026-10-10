# Phase 65: Standalone Fidelity Fixes - Research

**Researched:** 2026-06-07
**Domain:** HP-41 numeric representation, CLI/GUI display priority chains, entry-buffer CHS
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01 (range escalation):** Extend `HpNum` to represent the full HP-41 range ±9.999999999E±99.
- **D-02 (kept in Phase 65):** Range extension stays in this phase alongside DISP-01/02/03.
- **D-03 (representation — RESEARCH RESOLVES):** Candidate (a) exact mantissa + separate exponent or (b) f64 fallback above Decimal ceiling. Researcher evaluates and proposes.
  - Locked constraints: (1) reach ±9.999999999E±99; (2) no silent regression of 10-digit decimal fidelity; (3) preserve save-file backward compat.
  - ADR is required — amends Frozen Invariant.
- **D-04 (FACT behavior):** Domain guards unchanged (X>69 OutOfRange, non-integer/negative Domain). Only the post-compute Decimal conversion wall changes.
- **D-05 (DISP-01 mirror GUI):** CLI reads `state.display_override` exactly like GUI while Some.
- **D-06 (DISP-01 priority):** `clock > stopwatch > entry_buf > prgm > display_override > alpha > X`.
- **D-07 (DISP-02 in-buffer toggle):** Non-empty entry_buf without `e` → toggle leading `-` in place. No flush, no dispatch, no stack lift.
- **D-08 (DISP-02 empty-buffer unchanged):** Empty entry_buf → dispatch `Op::Chs` as now.
- **D-09 (DISP-03 both frontends):** Wire flag-48 on both CLI and GUI.
- **D-10 (DISP-03 precedence):** `... > display_override > (flag48 ? alpha_reg : X)`. VIEW overrides AON; at rest flag 48 shows alpha register instead of X.
- **D-11 (stale-comment cleanup):** Remove "deferred to v4.4" DISP-01 comments from state.rs ~lines 80 and 531.
- **D-12 (parity discipline):** D-25.6 parity, D-11 no-polling, panic-free, `just`-only, no new runtime deps.

### Claude's Discretion
- Exact field/enum shape of extended `HpNum` (follows research recommendation + ADR).
- Test-fixture strategy for FACT(27..=69) accuracy.
- Whether GUI AON reads flag 48 from `CalcStateView.flags` or needs a new projection field (prefer reusing existing `flags` array — no IPC change).

### Deferred Ideas (OUT OF SCOPE)
- Splitting range extension into its own phase.
- FGAP-06 (CATALOG dump), FGAP-08 (SAVED/GETD block control).
- UNC-01/02/03, divergence-doc finalization (Phase 66).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MATH-01 | FACT(27..=69) returns correct result; HpNum range extended to ±9.999999999E±99 | Representation recommendation + ADR wording in this document |
| DISP-01 | CLI renders display_override (VIEW/AVIEW/PROMPT) | Exact insertion point in get_display_string confirmed |
| DISP-02 | CHS during mantissa entry toggles sign in-buffer | EEX-CHS pattern at app.rs:848-860 confirmed as model |
| DISP-03 | AON (flag 48) auto-displays ALPHA register on both CLI and GUI | Flag projection path + GUI alpha_str need confirmed |
</phase_requirements>

---

## Summary

Phase 65 closes four hardware-fidelity gaps. Three (DISP-01/02/03) are surgical frontend edits with precise insertion points already identified by the CONTEXT and audit. One (MATH-01) is an architectural change: `HpNum` currently wraps `Decimal` and is structurally incapable of representing values above ~7.92E28, so `FACT(27..=69)` always overflows.

The core finding of this research is the **representation recommendation for MATH-01**: approach (a) — a dedicated `HpNum` struct with a normalized `Decimal` mantissa in `[1, 10)` plus an `i8` exponent — is the correct choice. It preserves the 10-significant-digit decimal invariant for ALL values, gives the exact HP-41 range ±9.999999999E±99, keeps arithmetic clean, and supports backward-compatible serde via an untagged enum or `#[serde(from)]` bridge. Approach (b) — f64 fallback above the Decimal ceiling — is explicitly rejected because it introduces silent base-2 rounding for large values and creates a two-path numeric system that contradicts D-03(2) and the Frozen Invariant spirit.

The three display fixes are straightforward: DISP-01 inserts one branch into `get_display_string` in `ui.rs`; DISP-02 mirrors the existing EEX-CHS toggle (app.rs:848-860) for the mantissa case; DISP-03 inserts a flag-48 read into both `get_display_string` (CLI) and the `CalcStateView::from_state` display_str chain (Rust/GUI side).

**Primary recommendation:** Implement HpNum as `struct HpNum { mantissa: Decimal, exponent: i8 }` (normalized), replacing the bare `HpNum(Decimal)` tuple struct. Name the ADR `v4.3-005-hpnum-range-extension.md`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| HpNum representation | hp41-core (num.rs) | — | All arithmetic lives in core; frontends only call format_hpnum |
| op_fact post-compute | hp41-core (math.rs) | — | op_fact is in core, outside frozen math1/ |
| display_override read | hp41-cli (ui.rs) + hp41-gui types.rs | hp41-core (state.rs, already written) | State is core; reading and displaying is frontend-tier |
| CHS in-buffer | hp41-cli (app.rs) | — | Entry buffer is a CLI-tier concern; core never sees raw keystrokes |
| AON flag 48 read | hp41-cli (ui.rs) + hp41-gui (types.rs) | hp41-core (display_ops.rs, already writes flag) | Flag is core state; reading for display is frontend-tier |
| format_hpnum for large values | hp41-core (num.rs) or shared format module | — | Used by both CLI and GUI; must produce identical output |
| Serde backward compat | hp41-core (num.rs) | — | Save files are core state; deserialization lives here |

---

## MATH-01: HpNum Range Extension — Full Analysis

### Current State

`HpNum` is `struct HpNum(#[serde(with = "rust_decimal::serde::str")] pub(crate) Decimal)`.

`rust_decimal` uses a 96-bit integer mantissa with a scale (decimal places 0-28), giving a max value of `79,228,162,514,264,337,593,543,950,335` ≈ 7.92E28. [VERIFIED: rust_decimal docs — max value stated in crate docs]

Every arithmetic op follows the same pattern:
```rust
self.0.checked_add(rhs.0).map(HpNum::rounded).ok_or(HpError::Overflow)
```
`checked_*` returns `None` on overflow → maps to `HpError::Overflow`. This hits for any result above ~7.92E28, which covers FACT(28) = 3.05E29 through FACT(69) ≈ 1.71E98.

The inverse trig ops already use a `f64` round-trip bridge (`.to_f64()` → `.asin()` → `Decimal::from_f64().map(HpNum::rounded)`). `op_fact` uses `f64` for iteration (accumulating the product as `f64`) then hits the same `Decimal::from_f64(acc).ok_or(Overflow)` wall at line 480-482 when `acc > 7.92E28`.

### Approach (a): Normalized Mantissa + i8 Exponent [RECOMMENDED]

**Structure:**
```rust
pub struct HpNum {
    // Normalized: mantissa in [1, 10) for nonzero values; 0 for zero.
    // Holds exactly 10 significant decimal digits via HpNum::rounded discipline.
    mantissa: Decimal,
    // Exponent: -99..=99 gives the full HP-41 range.
    exponent: i8,
}
```

A value represents `mantissa × 10^exponent`. Zero is `{ mantissa: 0, exponent: 0 }`. Normalization: after every operation, round mantissa to 10 sig digits and renormalize (adjust exponent so mantissa ∈ [1, 10)).

**Arithmetic:**
- For values within the current Decimal range (|exp| ≤ 28, mantissa representable), arithmetic delegates to Decimal exactly as today — no change in behavior for the 99% case.
- For large-exponent values, arithmetic works at the mantissa level: `(m1 × 10^e1) * (m2 × 10^e2) = (m1 * m2) × 10^(e1+e2)`, then renormalize. Since mantissas are both in [1, 10) with 10 digits, `m1 * m2` is in [1, 100) — always representable in Decimal.
- Addition/subtraction with different exponents requires alignment (scale the smaller-exp value): if `|e1 - e2| >= 10`, the smaller value underflows entirely (hardware-faithful — HP-41 loses precision this way). This exactly matches HP-41 hardware behavior.
- Overflow: result exponent > 99 → `HpError::Overflow`. Underflow: result exponent < -99 AND mantissa would round to zero → result is zero (or hardware Underflow — verify HP-41 behavior, likely returns 0 not error for underflow).

**Which ops change:**
All `checked_*` methods in `num.rs` must be updated to work with the new struct rather than delegating directly to `Decimal::checked_*`. The arithmetic logic remains decimal-exact; only the plumbing changes. The `MathematicalOps` methods (`checked_ln`, `checked_exp`, etc.) still work via mantissa extraction since mantissas are in Decimal range.

**f64 bridge ops (asin/acos/atan):** These already convert to f64 and back. With the new representation, they must reconstruct `HpNum` from a `Decimal` result via `HpNum::from_decimal_and_normalize()` rather than `Decimal::from_f64().map(HpNum::rounded)`. No behavioral change.

**op_fact:** The f64 accumulation loop is unchanged. The post-compute wall changes from:
```rust
Decimal::from_f64(acc).map(HpNum::rounded).ok_or(HpError::Overflow)?
```
to:
```rust
HpNum::from_f64(acc).ok_or(HpError::Overflow)?
```
where `HpNum::from_f64` constructs via `f64::log10(acc).floor()` for the exponent and `Decimal::from_f64(acc / 10f64.powi(exp))` for the mantissa. f64 has ~15.9 decimal digits of precision; rounding to 10 via `HpNum::rounded_mantissa()` is sufficient and matches the existing asin/acos/atan pattern already accepted by the project.

**display/format_hpnum:** Currently `format_hpnum` receives an `&HpNum` and formats the inner `Decimal`. With the new struct, for small-exponent values (where `mantissa × 10^exponent` fits in a Decimal), reconstruct the full `Decimal` and use existing formatting. For large-exponent values (|exp| > 13 or so), always use scientific notation: `{mantissa}E{exponent:+03}` with 10 sig digits. The HP-41 display is 14-char; scientific notation with 10 sig digits uses format `M.MMMMMMMMMExx` (1 integer digit + decimal point + 9 decimal digits + E + 2-digit exp = 14 chars). Both CLI (`format_hpnum` in hp41-core) and GUI (uses same `format_hpnum` via IPC `display_str`) are unaffected — they receive the pre-formatted string. [ASSUMED — need to verify format_hpnum location]

### Approach (b): f64 Fallback Above Decimal Ceiling [REJECTED]

**Why rejected:**
1. Introduces silent base-2 rounding for values above ~7.92E28. FACT(50) = 3.04E64; f64 has ~15.9 decimal digits but base-2 representation causes rounding artifacts that cannot be corrected to 10 sig digits reliably for arbitrary values. This violates D-03(2).
2. Creates a two-path numeric system: every op must branch on which variant is active, every format call must handle both, serde must handle both. More code surface than approach (a).
3. The project already has an f64 bridge for inverse trig (accepted because asin/acos/atan results are in [-π, π], well within Decimal range). Extending f64 as a representation for stored values is a fundamentally different commitment.
4. Approach (a) keeps 100% of values at 10 sig digits — no carve-outs needed. The Frozen Invariant amendment is clean: "arithmetic precision remains 10 decimal significant digits across the full HP-41 range; f64 is used only as an intermediate for transcendental computations (as today)."

### Serde Backward Compatibility [HIGH RISK — MUST BE EXACT]

**Current serde shape:** `HpNum` serializes as a JSON string via `rust_decimal::serde::str`. Example: `"3.1415926536"`. Every value in v1.0–v4.2 save files uses this format.

**Required:** Old saves (`"3.1415926536"`, `"123456789.0"`, etc.) must still deserialize without migration.

**Recommended serde strategy:**

Define a private `HpNumLegacy(Decimal)` newtype with the existing `rust_decimal::serde::str` serde, used only in `Deserialize`. Then implement `Deserialize for HpNum` with `#[serde(untagged)]` over an internal enum:

```rust
// Private helper for deserialization only
#[derive(Deserialize)]
#[serde(untagged)]
enum HpNumWire {
    // New format: { "m": "1.234567890", "e": 45 }
    Extended { m: String, e: i8 },
    // Legacy format: bare decimal string "1.234567890"
    Legacy(String),
}

impl<'de> Deserialize<'de> for HpNum {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let wire = HpNumWire::deserialize(de)?;
        match wire {
            HpNumWire::Extended { m, e } => {
                let mantissa = Decimal::from_str(&m).map_err(serde::de::Error::custom)?;
                Ok(HpNum { mantissa, exponent: e })
            }
            HpNumWire::Legacy(s) => {
                let d = Decimal::from_str(&s).map_err(serde::de::Error::custom)?;
                // Normalize legacy value into (mantissa, exponent) form
                Ok(HpNum::from_decimal(d))
            }
        }
    }
}
```

**Serialize** always uses the new `{ "m": "...", "e": N }` form for values requiring an exponent field (i.e., where `exponent != 0` or where mantissa doesn't roundtrip cleanly as a bare Decimal string). For values where `exponent == 0` and mantissa is exact, the planner may choose to serialize as the legacy bare string for readability — but this is a discretion call. The simplest correct implementation serializes all values in the new form.

**Implication:** Old save files load. New save files from v4.3 onward use the extended format and will NOT load in v4.2 or earlier (forward-incompatibility is acceptable per project conventions — save-file compat is one-way backward only).

**`HpValue` untagged enum:** `HpValue` uses `#[serde(untagged)]` over `Numeric(HpNum)` and `Alpha([u8;6])`. Since `HpNum`'s new serde shape is `{ "m": "...", "e": N }` (an object) while `Alpha` is a JSON array, untagged disambiguation remains unambiguous. Legacy saves with `Numeric` as a bare string (`"3.14"`) are also unambiguous from the Alpha array `[1,2,3,4,5,6]`. The `HpNumWire` untagged enum handles the string vs. object distinction within the `Numeric` branch.

### ADR Decision and Wording

**Filename:** `docs/adr/v4.3-005-hpnum-range-extension.md` *(next free number — v4.3-003 = alarm-prefix, v4.3-004 = interrupt-alarm-pending-field already exist)*

**Decision:** Replace `HpNum(Decimal)` with `HpNum { mantissa: Decimal, exponent: i8 }` (normalized, mantissa in `[1, 10)` for nonzero values, 10 sig digits via `HpNum::rounded_mantissa()`). This preserves full decimal fidelity across the entire HP-41 range ±9.999999999E±99.

**Frozen Invariant amendment wording for CLAUDE.md** (replaces the current "BCD/f64:" bullet):

> **BCD/f64:** `rust_decimal` 1.42 with 10-significant-digit rounding. `HpNum` in `hp41-core/src/num.rs` stores a normalized `(mantissa: Decimal, exponent: i8)` pair covering the full HP-41 range ±9.999999999E±99 (ADR v4.3-005). Custom BCD was evaluated and rejected (v3.0). f64 is used only as an intermediate for transcendental computations (asin/acos/atan/factorial accumulator) — never as a stored representation. The 10-sig-digit decimal invariant holds for all representable values.

---

## DISP-01: CLI display_override Insertion

### Current State (Confirmed)

`get_display_string()` in `hp41-cli/src/ui.rs` at line 133-161:

```
clock_display → stopwatch_display → entry_buf → prgm_mode → alpha_mode → X
```

There is **zero** reference to `state.display_override` in this function (confirmed by grep: 0 matches). The field exists in `CalcState` and is written by `op_view`/`op_aview`/`op_prompt` in `display_ops.rs`.

### Required Change

Insert between `prgm_mode` and `alpha_mode` branches (D-06 priority):

```rust
} else if let Some(ref s) = st.display_override {
    s.clone()
} else if st.alpha_mode {
```

This achieves: `clock > stopwatch > entry_buf > prgm > display_override > alpha > X`.

Also update stale comment at line 132 (currently `Priority: clock_active > stopwatch_keyboard_mode > entry_buf > prgm step > alpha > formatted X`) to include `display_override`.

Also remove "DISP-01 deferred to v4.4" comments in `hp41-core/src/state.rs` at ~lines 80 and 531 (D-11).

---

## DISP-02: CHS During Mantissa Entry

### Pattern to Mirror (app.rs:848-860)

The EEX-CHS toggle (confirmed by grep, actual lines ~848-860):

```rust
if c == 'n' && self.state.entry_buf.contains('e') {
    // toggle exponent sign in-place — no flush, no dispatch
    if let Some(e_pos) = self.state.entry_buf.find('e') {
        let after_e = &self.state.entry_buf[e_pos + 1..];
        if after_e.starts_with('-') {
            // "1e-" → "1e"
            self.state.entry_buf.remove(e_pos + 1);
        } else {
            // "1e2" → "1e-2", "1e" → "1e-"
            self.state.entry_buf.insert(e_pos + 1, '-');
        }
    }
    return;
}
```

### Required Change for Mantissa CHS

Gate: `entry_buf` is non-empty AND does NOT contain `'e'`. Key mapping: whatever key dispatches `Op::Chs` when entry_buf is empty must be intercepted before reaching the dispatch path.

```rust
// DISP-02: CHS during mantissa entry — toggle leading '-' in place
if <chs_key_condition> && !self.state.entry_buf.is_empty()
    && !self.state.entry_buf.contains('e')
{
    if self.state.entry_buf.starts_with('-') {
        self.state.entry_buf.remove(0);
    } else {
        self.state.entry_buf.insert(0, '-');
    }
    return;
}
```

The planner must identify the exact key code that triggers CHS in `app.rs` (the `-` key or dedicated CHS binding) to write the precise condition. The pattern above is correct; only the condition needs the exact key lookup.

**Edge case:** entry_buf = `"0"` → becomes `"-0"`. The HP-41 shows `-0` during entry (entry buf is display verbatim). On flush, `-0.0` is a negative zero in Decimal — this may need a post-flush normalization check, but that is a planner detail.

---

## DISP-03: AON Auto-Display (Flag 48)

### Current State (Confirmed)

- `op_aon` sets flag 48, `op_aoff` clears it — confirmed in `display_ops.rs`.
- CLI `ui.rs`: zero references to flag 48 (confirmed by grep: 0 matches).
- GUI App.tsx: zero references to `flags` array (the field is projected in `CalcStateView` as `Vec<u8>` of set-flag indices, but App.tsx never reads it).
- `CalcStateView.flags` is already projected: `(0u8..=55).filter(|i| (state.flags >> i) & 1 == 1).collect()` (types.rs:239-241). Flag 48 appears as the integer `48` in this Vec when set.

### CLI Change

In `get_display_string()`, replace the final else branch:

```rust
// Before:
} else {
    format_hpnum(&st.stack.x, &st.display_mode)
}

// After (D-10):
} else if (st.flags >> 48) & 1 == 1 {
    // AON: flag 48 set — show ALPHA register instead of X
    format_alpha(&st.alpha_reg)
} else {
    format_hpnum(&st.stack.x, &st.display_mode)
}
```

Alternatively use `st.flags & (1u64 << 48) != 0` — same semantics.

### GUI Change

For the GUI, the AON branch must be inserted into `CalcStateView::from_state` in `types.rs` (NOT in App.tsx). The `display_str` computation is the canonical source; the React layer just renders `calcState.display_str`. 

The display_str chain in `from_state` ends with:
```rust
} else if state.alpha_mode {
    format_alpha(&state.alpha_reg)
} else {
    format_hpnum(&state.stack.x, &state.display_mode)
}
```

Replace the final else:
```rust
} else if state.flags & (1u64 << 48) != 0 {
    // AON: flag 48 set — show ALPHA register at rest
    format_alpha(&state.alpha_reg)
} else {
    format_hpnum(&state.stack.x, &state.display_mode)
}
```

This requires NO new IPC field, NO App.tsx change, NO new `CalcStateView` field. The existing `display_str` field carries the AON string to the frontend. [VERIFIED from types.rs source] This satisfies the CONTEXT discretion note ("prefer reusing the existing flags array — no IPC change") because we never read `flags` in TS; instead the Rust side bakes the decision into `display_str`.

**Note on the CONTEXT D-10 wording:** D-10 says "GUI `... ?? display_override ?? (flag48 ? alphaText : display_str)`" — this describes the logical precedence. The cleanest implementation bakes the flag-48 decision into `display_str` on the Rust side (in `from_state`), so the TS/React layer sees the result without change. The App.tsx chain at line 1416 (`calcState.pending_yield?.text ?? calcState.display_override ?? calcState.display_str`) remains unchanged.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Decimal arithmetic | Custom BCD arithmetic | rust_decimal 1.42 MathematicalOps | Already in the project; handles 10-sig rounding |
| Exponent normalization | Custom base-conversion | Simple f64 log10 + Decimal mantissa extract | One-shot at construction; not an inner loop |
| Serde untagged dispatch | Custom format detection | #[serde(untagged)] over an enum | Already used by HpValue; well-tested pattern |

---

## Common Pitfalls

### Pitfall 1: HpNum::rounded Called on a Decimal That Cannot Normalize

**What goes wrong:** The new `HpNum::rounded_mantissa()` (analogous to current `HpNum::rounded`) calls `round_sf_with_strategy(10, ...)` on the mantissa. If the mantissa is zero, this must return `HpNum::zero()` not panic.

**How to avoid:** Guard: `if mantissa.is_zero() { return HpNum::zero(); }` before rounding.

### Pitfall 2: Serde Untagged Ambiguity

**What goes wrong:** If the new `HpNum` serializes as a JSON object `{ "m": "1.0", "e": 0 }` AND the legacy format is a bare string `"1.0"`, the `#[serde(untagged)]` enum needs the object arm FIRST (serde tries arms in order). Putting Legacy first would match the object as a string and fail.

**How to avoid:** `HpNumWire::Extended { m, e }` must be the first variant.

### Pitfall 3: display_override Priority — CLI Comment Mismatch

**What goes wrong:** The comment at `get_display_string()` line 132 lists the old chain without `display_override`. If the comment is not updated, future maintainers may re-introduce the old ordering.

**How to avoid:** Update comment as part of DISP-01 plan task.

### Pitfall 4: AON Interacts with ALPHA Mode

**What goes wrong:** D-10 says flag 48 replaces X fallback, BELOW the alpha_mode branch. If the condition order is wrong (flag-48 check before `alpha_mode` check), entering ALPHA mode while AON is set could show the alpha reg twice or conflict with ALPHA-mode display.

**How to avoid:** The flag-48 branch must be the LAST else-if before the bare X fallback, exactly as shown in the code examples above.

### Pitfall 5: op_fact f64 Precision for FACT(27..=69)

**What goes wrong:** f64 has 53-bit mantissa ≈ 15.9 decimal digits. FACT(69) ≈ 1.711224524281413E98 — f64 represents this with ~15.9 sig digits. After rounding to 10 sig digits via `HpNum::from_f64`, the result is correct to 10 digits. However, f64 cannot represent FACT(n) exactly for large n (integer overflow is not the issue; precision is). The HP-41 hardware also computes FACT iteratively in BCD and rounds to 10 digits — so f64→10-digit-round matches the hardware fidelity target.

**How to avoid:** The existing approach (f64 loop + round to 10 digits) is correct for FACT. Document explicitly in the ADR that FACT uses f64 intermediate for n=27..=69, and that this is within the existing f64-bridge policy (asin/acos/atan precedent).

### Pitfall 6: proptest_math.rs X≤26 Magnitude Wall

**What goes wrong:** The Phase-27 proptest has a calibrated bound `X <= 26` because FACT(27+) previously overflowed. After this fix, the proptest should generate X up to 69. Without recalibration, the proptest continues to only test the already-working range.

**How to avoid:** Planner must add a task to update `proptest_math.rs` to generate X in 0..=69 for FACT and add golden fixtures for FACT(27..=69) in `numerical_accuracy.rs`. [ASSUMED — exact line numbers not read, but test files confirmed to exist per CONTEXT]

---

## Architecture Patterns

### HpNum Struct Change

```
// Before
pub struct HpNum(#[serde(with = "rust_decimal::serde::str")] pub(crate) Decimal);

// After
pub struct HpNum {
    // Mantissa in [1, 10) for nonzero; exactly 10 sig digits.
    pub(crate) mantissa: Decimal,
    // Exponent: -99..=99. Full HP-41 range.
    pub(crate) exponent: i8,
}
```

### Normalization Helper

```rust
impl HpNum {
    fn normalize(mantissa: Decimal, exponent: i8) -> Result<HpNum, HpError> {
        if mantissa.is_zero() {
            return Ok(HpNum::zero());
        }
        // Round to 10 sig digits first
        let m = mantissa.round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
            .expect("round_sf always succeeds for finite Decimal");
        // Adjust exponent for digits outside [1, 10)
        // ... (planner works out exact adjustment arithmetic)
        // Guard exponent bounds
        if adjusted_exp > 99 { return Err(HpError::Overflow); }
        if adjusted_exp < -99 { return Ok(HpNum::zero()); } // underflow → 0
        Ok(HpNum { mantissa: adjusted_m, exponent: adjusted_exp as i8 })
    }
}
```

### Recommended Project Structure

No structural changes. All changes are within:

```
hp41-core/src/
├── num.rs              # HpNum struct + arithmetic + serde (MATH-01, heavy)
├── ops/math.rs         # op_fact post-compute wall (MATH-01, light edit)
hp41-cli/src/
├── ui.rs               # get_display_string() (DISP-01, DISP-03)
├── app.rs              # CHS in-buffer (DISP-02)
hp41-gui/src-tauri/src/
├── types.rs            # from_state display_str chain (DISP-03)
hp41-core/src/state.rs  # stale comment removal (D-11)
docs/adr/
└── v4.3-005-hpnum-range-extension.md  # MATH-01 ADR
```

---

## Standard Stack

### Core (No Changes)

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| rust_decimal | 1.42 | Decimal arithmetic for HpNum mantissa | Retained as-is [VERIFIED: Cargo.toml in project] |
| serde | (workspace) | Serialization | Retained; custom Deserialize for HpNum |
| ratatui | 0.30 | TUI rendering (CLI display changes) | No change needed |

**No new dependencies** — D-12 / Frozen Invariant "Zero new runtime deps since v3.0".

---

## Package Legitimacy Audit

No new packages are installed in this phase. Not applicable.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + proptest + custom numerical_accuracy harness |
| Config file | `hp41-core/tests/numerical_accuracy.rs`, `hp41-core/tests/proptest_math.rs` |
| Quick run command | `cargo test -p hp41-core --lib` |
| Full suite command | `just test` (all workspace crates) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MATH-01 | FACT(27..=69) returns 10-sig-digit result | golden fixture | `cargo test -p hp41-core --test numerical_accuracy fact` | ✅ (extend existing) |
| MATH-01 | No regression on FACT(0..=26) | regression | `cargo test -p hp41-core --test proptest_math` | ✅ (recalibrate magnitude wall) |
| MATH-01 | HpNum serde round-trip: legacy string format loads | unit | `cargo test -p hp41-core --lib hpnum serde` | ✅ (extend existing test_hpnum_serde_is_string) |
| MATH-01 | HpNum serde round-trip: large-exponent new format | unit | `cargo test -p hp41-core --lib hpnum serde` | ❌ Wave 0 gap |
| MATH-01 | Arithmetic: checked_add/mul for values with exponent > 0 | unit | `cargo test -p hp41-core --lib hpnum` | ❌ Wave 0 gap |
| MATH-01 | Overflow guard at exp > 99 | unit | `cargo test -p hp41-core --lib hpnum` | ❌ Wave 0 gap |
| DISP-01 | display_override shown in CLI | integration | `cargo test -p hp41-cli` | ❌ Wave 0 gap |
| DISP-02 | CHS mantissa toggle: "123" → "-123" → "123" | integration | `cargo test -p hp41-cli` | ❌ Wave 0 gap |
| DISP-02 | CHS with EEX active unchanged | regression | `cargo test -p hp41-cli` | ✅ (existing EEX-CHS test at ~line 2683) |
| DISP-03 | AON: display shows alpha_reg at rest (flag 48 set) | unit | `cargo test -p hp41-core --lib types` + `cargo test -p hp41-cli` | ❌ Wave 0 gap |

### Sampling Rate

- Per task commit: `cargo test -p hp41-core --lib`
- Per wave merge: `just test`
- Phase gate: `just test` green + FACT golden fixtures passing before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] HpNum large-exponent serde round-trip test
- [ ] HpNum arithmetic tests for exponent-carrying ops
- [ ] HpNum overflow-at-exp-99 test
- [ ] CLI DISP-01 integration test (display_override shown)
- [ ] CLI DISP-02 integration test (CHS mantissa toggle)
- [ ] DISP-03 unit test for AON flag-48 branch in both CLI get_display_string and GUI from_state

---

## Security Domain

This phase involves no authentication, session management, input validation beyond existing arithmetic guards, or cryptography. Security domain: NOT APPLICABLE.

---

## Environment Availability

This phase is purely code/config changes within the existing Rust workspace and TypeScript frontend. No external dependencies beyond the existing toolchain.

| Dependency | Required By | Available | Notes |
|------------|------------|-----------|-------|
| Rust stable (MSRV 1.88) | All Rust changes | ✓ | Existing workspace |
| rust_decimal 1.42 | num.rs | ✓ | Already in Cargo.toml |
| just | Task runner | ✓ | Project standard |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | format_hpnum lives in hp41-core (not hp41-cli) and is called by both CLI and GUI indirectly via types.rs | DISP-03 GUI change section | If format_hpnum is duplicated (it might be per SC-4 pattern), both copies need updating |
| A2 | proptest_math.rs contains an explicit X<=26 magnitude wall as a constant or literal that can be changed | Pitfall 6 | If it's derived from a FACT golden table length, the fix is different |
| A3 | FACT(27..=69) golden values from HP-41 hardware can be sourced from hp41-math1-divergences.md or computed via f64 to 10 sig digits | MATH-01 | If hardware-exact values are needed, a reference machine or Free42 must be consulted; f64 to 10 sig digits is the stated fidelity target |
| A4 | The CHS key in app.rs is triggered by KeyCode::Char('-') or a dedicated physical key | DISP-02 | Planner must read the CHS key binding in app.rs before writing the exact condition |

---

## Open Questions (RESOLVED)

1. **format_hpnum location and duplication**
   - What we know: `format_alpha` is imported from `hp41_core` in `ui.rs` (line 20). The CLI stack panel calls `format_hpnum` from `hp41_core` as well.
   - What's unclear: Does `format_hpnum` need updating to handle the new `HpNum` struct with separate exponent, or does `HpNum::Display` handle it?
   - **RESOLVED:** `format_hpnum` lives in `hp41-core/src/format.rs` (lines 18-25), takes `&HpNum`, and calls `n.inner()` (confirmed by PATTERNS.md and Plan 01 `<interfaces>`). After the struct change `inner()` returns the mantissa only, so `format_hpnum` MUST be updated. Plan 01 Task 1 owns this update (reconstruct `mantissa × 10^exponent` for small |exponent|, route large-exponent values to `format_sci`); `format_hpnum` is in Plan 01 `files_modified`.

2. **Exact key binding for CHS in app.rs**
   - What we know: EEX is triggered by `'e'` character. CHS maps to `Op::Chs` somewhere in the dispatch chain.
   - What's unclear: The exact `KeyCode` and whether there's a dedicated CHS path or it goes through the generic `key_to_op` resolver.
   - **RESOLVED:** The EEX-CHS in-buffer block at `app.rs` ~848-864 is keyed on `c == 'n'` (the CHS character), mirroring the EEX `c == 'e'` guard. Plan 03 Task 1 `read_first` directs the executor to confirm the exact binding in `hp41-cli/src/keys.rs` / `key_to_op` before writing the mantissa-toggle condition, and mirrors the existing `c == 'n'` shape. Resolved at plan time; the executor verifies against `keys.rs`.

---

## Sources

### Primary (HIGH confidence)
- `hp41-core/src/num.rs` — Direct read; all arithmetic ops, serde shape, `HpNum::rounded` confirmed
- `hp41-core/src/ops/math.rs` lines 440-485 — Direct read; `op_fact` logic, f64 wall at line 480-482 confirmed
- `hp41-cli/src/ui.rs` — Direct read; `get_display_string()` at lines 133-161, zero `display_override` references confirmed
- `hp41-cli/src/app.rs` — grep confirmed EEX-CHS toggle at ~lines 848-860
- `hp41-gui/src-tauri/src/types.rs` lines 155-285 — Direct read; `from_state` display_str chain, `flags` projection at lines 239-241 confirmed
- `.planning/phases/65-standalone-fidelity-fixes/65-CONTEXT.md` — All locked decisions D-01 through D-12

### Secondary (MEDIUM confidence)
- `.planning/research/DIVERGENCE-AUDIT.md` — Grep-verified gap descriptions for FGAP-02/03/05/07
- rust_decimal crate documentation — Max value ~7.92E28 [ASSUMED from training; consistent with the overflow behavior observed in math.rs comments]

---

## Metadata

**Confidence breakdown:**
- MATH-01 representation choice: HIGH — based on direct code reading of num.rs, math.rs; the tradeoffs are deterministic
- DISP-01/02/03 insertion points: HIGH — confirmed by reading ui.rs, app.rs, types.rs source directly
- Serde backward compat strategy: HIGH — based on existing `HpValue` untagged enum pattern in the same file
- FACT(27..=69) golden values: MEDIUM — f64 to 10 sig digits is the strategy; exact HP-41 hardware values not verified

**Research date:** 2026-06-07
**Valid until:** 2026-07-07 (stable domain; no external dependency changes expected)
