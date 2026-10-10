# Phase 65: Standalone Fidelity Fixes - Pattern Map

**Mapped:** 2026-06-07
**Files analyzed:** 9 files (modify/create)
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-core/src/num.rs` | model | transform | `HpValue` untagged enum in same file (lines 17-24) | exact (same file) |
| `hp41-core/src/ops/math.rs` | service | CRUD | `checked_asin` in `num.rs` (f64-bridge pattern, lines 297-305) | role-match |
| `hp41-cli/src/ui.rs` | utility | request-response | `get_display_string` itself (existing chain, lines 133-161) | exact (modify in place) |
| `hp41-cli/src/app.rs` | controller | request-response | EEX-CHS block in same file (lines 848-864) | exact (same file) |
| `hp41-gui/src-tauri/src/types.rs` | model | request-response | `from_state` display_str chain in same file (lines 177-199) | exact (same file) |
| `hp41-core/src/state.rs` | model | — | comment at lines 80, 531 (stale text to remove) | trivial edit |
| `hp41-core/src/format.rs` | utility | transform | `format_hpnum` itself (lines 18-25); called by both CLI and GUI | exact (must update) |
| `docs/adr/v4.3-005-hpnum-range-extension.md` | config | — | `docs/adr/v4.3-001-single-instance-guard.md` (house ADR format) | exact (003/004 taken → 005) |
| `hp41-core/tests/numerical_accuracy.rs` + `hp41-core/tests/proptest_math.rs` | test | — | existing FACT golden fixtures (lines 2776-2856) + `fact_recursive_invariant` proptest (lines 143-171) | exact (extend) |

---

## Pattern Assignments

### `hp41-core/src/num.rs` (model, transform) — MATH-01 heavy plan

**Analog 1 — existing HpValue untagged enum serde** (`hp41-core/src/num.rs` lines 9-24):

```rust
/// Serde uses `#[serde(untagged)]`: old save files with bare Decimal
/// strings deserialize as `Numeric`; new `[u8; 6]` arrays as `Alpha`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HpValue {
    Numeric(HpNum),
    Alpha([u8; 6]),
}
```

Copy this untagged-enum pattern verbatim for the `HpNumWire` deserialization helper. The `Extended` arm (new `{ m, e }` object) must appear FIRST — serde tries arms in order and the Legacy bare-string arm must be second (Pitfall 2 from RESEARCH).

**Analog 2 — current HpNum struct and serde** (`hp41-core/src/num.rs` lines 122-133):

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HpNum(#[serde(with = "rust_decimal::serde::str")] pub(crate) Decimal);

impl HpNum {
    pub fn rounded(d: Decimal) -> Self {
        HpNum(
            d.round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
                .expect("round_sf_with_strategy(10) must succeed for valid finite Decimal"),
        )
    }

    pub fn zero() -> Self {
        HpNum(Decimal::ZERO)
    }
```

Replace the tuple struct with the new named-field struct. The `rounded` method becomes `rounded_mantissa`. The `zero()` sentinel stays. The `#[serde(with)]` attribute is replaced by custom `Serialize`/`Deserialize` impls (see serde pattern below).

**Analog 3 — existing checked_* arithmetic pattern** (`hp41-core/src/num.rs` lines 143-172):

```rust
pub fn checked_add(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
    self.0
        .checked_add(rhs.0)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

pub fn checked_mul(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
    self.0
        .checked_mul(rhs.0)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

pub fn checked_div(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
    if rhs.0.is_zero() {
        return Err(HpError::DivideByZero);
    }
    self.0
        .checked_div(rhs.0)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}
```

After the struct change, these must delegate through `HpNum::normalize(mantissa, exponent)` instead of `Decimal::checked_*`. The `Overflow` mapping at exponent > 99 stays identical.

**Analog 4 — f64-bridge pattern for construction** (`hp41-core/src/num.rs` lines 297-305):

```rust
pub fn checked_asin(&self) -> Result<HpNum, HpError> {
    let v = self.0.to_f64().ok_or(HpError::Overflow)?;
    if !(-1.0..=1.0).contains(&v) {
        return Err(HpError::Domain);
    }
    Decimal::from_f64(v.asin())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}
```

The new `HpNum::from_f64(acc)` constructor mirrors this pattern: extract the exponent via `f64::log10(acc).floor()`, extract the mantissa via `Decimal::from_f64(acc / 10f64.powi(exp))`, then call `HpNum::normalize(mantissa, exp as i8)`.

**Serde pattern to implement** (based on RESEARCH.md recommendation, using HpValue untagged enum as structural model):

```rust
// Private helper for deserialization only — not pub
#[derive(Deserialize)]
#[serde(untagged)]
enum HpNumWire {
    // New format (arm FIRST per Pitfall 2): { "m": "1.234567890", "e": 45 }
    Extended { m: String, e: i8 },
    // Legacy format (arm SECOND): bare decimal string "1.234567890"
    Legacy(String),
}

impl<'de> Deserialize<'de> for HpNum {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        match HpNumWire::deserialize(de)? {
            HpNumWire::Extended { m, e } => {
                let mantissa = Decimal::from_str(&m).map_err(serde::de::Error::custom)?;
                Ok(HpNum { mantissa, exponent: e })
            }
            HpNumWire::Legacy(s) => {
                let d = Decimal::from_str(&s).map_err(serde::de::Error::custom)?;
                Ok(HpNum::from_decimal(d))
            }
        }
    }
}
```

**Existing serde test to extend** (`hp41-core/src/num.rs` lines 372-393):

```rust
#[test]
fn test_hpnum_serde_is_string() {
    let n = HpNum::from(3i32);
    let json = serde_json::to_string(&n).unwrap();
    assert!(json.starts_with('"'), "HpNum must serialize as JSON string, got: {json}");
    let back: HpNum = serde_json::from_str(&json).unwrap();
    assert_eq!(back, n, "round-trip must be lossless");
}
```

Add parallel tests: one for the new `{ "m": "...", "e": N }` serialize round-trip for a large-exponent value, one for deserializing a legacy bare-string into the new struct.

---

### `hp41-core/src/format.rs` — `format_hpnum` update (MATH-01)

**Current signature** (`hp41-core/src/format.rs` lines 18-25):

```rust
pub fn format_hpnum(n: &HpNum, mode: &DisplayMode) -> String {
    let d = n.inner();
    match mode {
        DisplayMode::Fix(digits) => format_fix(d, *digits as usize),
        DisplayMode::Sci(digits) => format_sci(d, *digits as usize),
        DisplayMode::Eng(digits) => format_eng(d, *digits as usize),
    }
}
```

After the `HpNum` struct change, `n.inner()` returns the mantissa `Decimal` only (not the full value). The function must reconstruct the full `Decimal` when `exponent == 0` (safe — fits in Decimal), or use scientific notation directly when `|exponent| > 0`. The existing `format_sci`/`format_eng`/`format_fix` helpers operate on `Decimal` values normalized to base 1; for large-exponent values the function pre-scales and delegates to `format_sci`.

`format_alpha` (line 105-107) is unaffected — takes `&str`.

**Note on duplication (SC-4):** `format_hpnum` is defined ONCE in `hp41-core/src/format.rs` and imported by both CLI (`ui.rs` line 20: `use hp41_core::{format_alpha, format_hpnum, AngleMode}`) and GUI (`types.rs` which calls `format_hpnum` internally). It is NOT duplicated in `hp41-gui/src-tauri/src/prgm_display.rs` — that is a separate `op_display_name` duplication (the SC-4 invariant). Updating `format_hpnum` in one place updates both frontends.

---

### `hp41-core/src/ops/math.rs` (service, CRUD) — MATH-01 light edit

**Current post-compute wall** (`hp41-core/src/ops/math.rs` lines 480-484):

```rust
let result = Decimal::from_f64(acc)
    .map(HpNum::rounded)
    .ok_or(HpError::Overflow)?;
unary_result(state, result);
Ok(())
```

**Pattern to replace with** (mirrors the f64-bridge construction pattern from `checked_asin`):

```rust
let result = HpNum::from_f64(acc).ok_or(HpError::Overflow)?;
unary_result(state, result);
Ok(())
```

Domain guards above (lines 462-473) are unchanged. The `unary_result` call signature is unchanged. Only the single `Decimal::from_f64` line changes.

---

### `hp41-cli/src/ui.rs` (utility, request-response) — DISP-01 and DISP-03

**Current `get_display_string` chain** (`hp41-cli/src/ui.rs` lines 131-161):

```rust
/// Get the string to show in the HP-41 display area.
/// Priority: clock_active > stopwatch_keyboard_mode > entry_buf > prgm step > alpha > formatted X.
fn get_display_string(app: &App) -> String {
    let st = &app.state;
    if let Some(s) = get_clock_display_str(st) {
        return s;
    }
    if let Some(s) = get_stopwatch_display_str(st) {
        return s;
    }
    if !st.entry_buf.is_empty() {
        if st.entry_buf.contains('e') {
            format_entry_buf_display(&st.entry_buf)
        } else {
            st.entry_buf.clone()
        }
    } else if st.prgm_mode {
        prgm_display::format_step(st)
    } else if st.alpha_mode {
        format_alpha(&st.alpha_reg)
    } else {
        format_hpnum(&st.stack.x, &st.display_mode)
    }
}
```

**DISP-01 insertion point** (between `prgm_mode` and `alpha_mode` arms, per D-06):

```rust
    } else if st.prgm_mode {
        prgm_display::format_step(st)
    } else if let Some(ref s) = st.display_override {   // DISP-01: NEW
        s.clone()
    } else if st.alpha_mode {
        format_alpha(&st.alpha_reg)
    } else {
```

**DISP-03 insertion point** (replace the final `else` arm, per D-10 — flag-48 branch must be LAST before X fallback, AFTER alpha_mode):

```rust
    } else if st.alpha_mode {
        format_alpha(&st.alpha_reg)
    } else if st.flags & (1u64 << 48) != 0 {            // DISP-03: NEW (flag 48 = AON)
        format_alpha(&st.alpha_reg)
    } else {
        format_hpnum(&st.stack.x, &st.display_mode)
    }
```

**Comment update required:** Line 132 (`Priority: clock_active > stopwatch_keyboard_mode > entry_buf > prgm step > alpha > formatted X`) must become: `Priority: clock_active > stopwatch_keyboard_mode > entry_buf > prgm step > display_override > alpha > (flag48 ? alpha : X)`.

---

### `hp41-cli/src/app.rs` (controller, request-response) — DISP-02

**Analog — EEX-CHS in-buffer toggle** (`hp41-cli/src/app.rs` lines 848-864):

```rust
if c == 'n' && self.state.entry_buf.contains('e') {
    // CHS during EEX entry: toggle exponent sign in-place — no flush, no dispatch.
    if let Some(e_pos) = self.state.entry_buf.find('e') {
        let after_e = &self.state.entry_buf[e_pos + 1..];
        if after_e.starts_with('-') {
            // Remove the minus: "1e-2" → "1e2", "1e-" → "1e"
            self.state.entry_buf.remove(e_pos + 1);
        } else {
            // Insert minus: "1e2" → "1e-2", "1e" → "1e-"
            self.state.entry_buf.insert(e_pos + 1, '-');
        }
    }
    self.message = None;
    return;
}
```

**DISP-02 pattern to insert BEFORE the EEX-CHS block** (gate: non-empty entry_buf AND no `'e'`):

```rust
// DISP-02: CHS during mantissa entry — toggle leading '-' in place.
// Must be checked BEFORE the EEX-CHS block (entry_buf with 'e' takes the
// other branch). Key: 'n' maps to Op::Chs (see keys.rs line 117).
if c == 'n' && !self.state.entry_buf.is_empty()
    && !self.state.entry_buf.contains('e')
{
    if self.state.entry_buf.starts_with('-') {
        self.state.entry_buf.remove(0);
    } else {
        self.state.entry_buf.insert(0, '-');
    }
    self.message = None;
    return;
}
```

**Key binding confirmed:** `keys.rs` line 117 — `KeyCode::Char('n') => Some(Op::Chs)`. So the condition is `c == 'n'` (exactly as in the EEX-CHS branch). The empty-buffer path (entry_buf empty) falls through to `call_dispatch(Op::Chs)` as before — unchanged per D-08.

---

### `hp41-gui/src-tauri/src/types.rs` (model, request-response) — DISP-03 GUI side

**Current `from_state` display_str chain** (`hp41-gui/src-tauri/src/types.rs` lines 177-199):

```rust
let display_str = if let Some(s) = get_clock_display_str(state) {
    s
} else if let Some(s) = get_stopwatch_display_str(state) {
    s
} else if state.modal_program.is_some()
    && state.entry_buf.is_empty()
    && state.modal_prompt.is_some()
{
    truncate_with_continuation(
        state.modal_prompt.as_ref().expect("modal_prompt set in guard above"),
    )
} else if !state.entry_buf.is_empty() {
    state.entry_buf.clone()
} else if state.prgm_mode {
    prgm_display::format_step(state)
} else if state.alpha_mode {
    format_alpha(&state.alpha_reg)
} else {
    format_hpnum(&state.stack.x, &state.display_mode)
};
```

**DISP-03 GUI insertion** (replace the final `else` arm — per RESEARCH.md §DISP-03 confirmed approach, no IPC change, no App.tsx change):

```rust
} else if state.alpha_mode {
    format_alpha(&state.alpha_reg)
} else if state.flags & (1u64 << 48) != 0 {    // DISP-03: AON flag 48 baked into display_str
    format_alpha(&state.alpha_reg)
} else {
    format_hpnum(&state.stack.x, &state.display_mode)
};
```

**What does NOT change:** `CalcStateView` struct fields, IPC contract, App.tsx display chain (`calcState.pending_yield?.text ?? calcState.display_override ?? calcState.display_str`) — the AON string flows through the existing `display_str` field.

**Flags projection** (already present, `types.rs` lines 239-241):

```rust
let flags: Vec<u8> = (0u8..=55)
    .filter(|i| (state.flags >> i) & 1 == 1)
    .collect();
```

The flag-48 check in the display_str branch uses `state.flags` (the raw `u64` bitfield from `CalcState`), NOT the projected `Vec<u8>`. This is consistent with the CLI approach.

**Stale comment in types.rs** (line 152): `display_override is NOT written by yield paths (D-04 / DISP-01 deferred)` — update this to reflect DISP-01 is resolved in Phase 65.

---

### `hp41-core/src/state.rs` (model) — D-11 stale-comment cleanup

**Line 80** (inside `YieldState` doc comment):
```
/// `display_override` is NOT written by these yield paths (D-04 / DISP-01 deferred).
```
Change to: `/// display_override is NOT written by these yield paths (D-04); DISP-01 resolved in Phase 65.`

**Line 531** (inside `pending_yield` field doc comment):
```
/// Leaves `display_override` untouched (DISP-01 stays deferred to v4.4).
```
Change to: `/// Leaves display_override untouched (D-04); DISP-01 resolved in Phase 65.`

---

### `docs/adr/v4.3-005-hpnum-range-extension.md` (config) — NEW ADR

**ADR format analog** (`docs/adr/v4.3-001-single-instance-guard.md` lines 1-50):

```markdown
# ADR v4.3-001 — Single-Instance Guard (no duplicate tray icon)

**Status:** Accepted
**Date:** 2026-06-06
**Context:** Post-v4.2 desktop polish (...)

## Context

[narrative of the problem]

## Decision

[what was decided and why]

## Consequences

[what changes as a result]

## Alternatives considered

- **Option A** — rejected: [reason]
- **Option B** — rejected: [reason]
```

Mirror this exact heading structure. Required sections for this ADR:
- Header: `# ADR v4.3-003 — HpNum Range Extension (±9.999999999E±99)`
- `**Status:** Accepted` / `**Date:**` / `**Context:**` inline fields
- `## Context` — explain the Decimal ceiling (~7.92E28), FACT(27..=69) overflow, and the Frozen Invariant being amended
- `## Decision` — `HpNum { mantissa: Decimal, exponent: i8 }` normalized struct; arithmetic discipline; serde backward compat strategy; f64 bridge policy
- `## Frozen Invariant Amendment` — paste the replacement wording for the CLAUDE.md BCD/f64 bullet (per RESEARCH.md §ADR Decision)
- `## Consequences` — save-file forward-incompatibility note; no new runtime deps
- `## Alternatives considered` — f64 fallback (rejected) with reasons

---

### `hp41-core/tests/numerical_accuracy.rs` + `hp41-core/tests/proptest_math.rs` (test) — MATH-01 test extension

**FACT golden fixture pattern** (`hp41-core/tests/numerical_accuracy.rs` lines 2776-2820):

```rust
{
    let mut s = CalcState::new();
    push(&mut s, "20");
    dispatch(&mut s, Op::Fact).unwrap();
    case!(
        "fact",
        "FACT(20) = 2.432902008e18",
        2.432_902_008e18,
        get_x(&s),
        wide    // "wide" tolerance for f64 accumulation rounding
    );
}
```

Copy this exact block shape for each new FACT(27..=69) golden value. Use `wide` tolerance (same as FACT(20) — f64 intermediate with 10-sig-digit rounding). Representative values to add:
- `FACT(27)` ≈ 1.088886945e28
- `FACT(35)` ≈ 1.033314797e40
- `FACT(50)` ≈ 3.041409320e64
- `FACT(69)` ≈ 1.711224524e98

**proptest magnitude wall** (`hp41-core/tests/proptest_math.rs` lines 143-171):

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn fact_recursive_invariant(n in 0i32..=26i32) {   // <-- change to 0i32..=68i32
        let mut s_n = CalcState::new();
        s_n.stack.x = HpNum::from(n);
        dispatch(&mut s_n, Op::Fact).unwrap();
        let fact_n = s_n.stack.x.inner().to_f64().unwrap_or(f64::NAN);

        let mut s_n1 = CalcState::new();
        s_n1.stack.x = HpNum::from(n + 1);
        dispatch(&mut s_n1, Op::Fact).unwrap();
        let fact_n1 = s_n1.stack.x.inner().to_f64().unwrap_or(f64::NAN);

        prop_assert!(
            passes_with_tol(fact_n1, fact_n * (n + 1) as f64, 1e-8),
            "FACT({}) = {}, FACT({}) = {}, expected ≈ {}",
            n, fact_n, n + 1, fact_n1, fact_n * (n + 1) as f64
        );
    }
}
```

Change: `n in 0i32..=26i32` → `n in 0i32..=68i32`. Update the comment block above (lines 137-150) to remove the "narrowed from 0..=68 to 0..=26 because op_fact returns Overflow" note — after MATH-01, the full range is supported.

Note: `s_n.stack.x.inner()` returns the mantissa `Decimal` in the new struct. After MATH-01, for large-exponent results `inner()` returns the normalized mantissa (not the full value). The `.to_f64()` call will return the mantissa in [1, 10), not the full value. The proptest must be updated to reconstruct the full f64 value as `mantissa_f64 * 10f64.powi(exponent as i32)` for large-exponent results, OR use a helper that does a full-value `to_f64()` conversion. Planner should add `HpNum::to_f64(&self) -> Option<f64>` on the new struct returning `mantissa.to_f64()? * 10f64.powi(self.exponent as i32)`.

---

## Shared Patterns

### Flag-bit access pattern
**Source:** `hp41-core/src/ops/display_ops.rs` lines 53-54 + RESEARCH.md §DISP-03
**Apply to:** `hp41-cli/src/ui.rs` (DISP-03) and `hp41-gui/src-tauri/src/types.rs` (DISP-03)

```rust
// Both frontends use the raw u64 bitfield directly
state.flags & (1u64 << 48) != 0
// Equivalent: (state.flags >> 48) & 1 == 1
```

Use `state.flags & (1u64 << 48) != 0` consistently in both frontends — matches the `flag_set` implementation in `display_ops.rs`.

### format_alpha call pattern
**Source:** `hp41-core/src/format.rs` lines 105-107; imported as `use hp41_core::{format_alpha, ...}` in `ui.rs` line 20
**Apply to:** DISP-03 flag-48 branch in both `ui.rs` and `types.rs`

```rust
format_alpha(&st.alpha_reg)   // CLI: `st` = &app.state (CalcState)
format_alpha(&state.alpha_reg) // GUI types.rs: `state` = &CalcState
```

### HpNum::rounded / round_sf pattern
**Source:** `hp41-core/src/num.rs` lines 128-133
**Apply to:** `normalize()` helper in the new HpNum implementation

```rust
d.round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
    .expect("round_sf_with_strategy(10) must succeed for valid finite Decimal")
```

Guard before calling: `if mantissa.is_zero() { return Ok(HpNum::zero()); }` — the `expect` panics on zero (Pitfall 1 from RESEARCH.md).

### early-return in-buffer editing pattern
**Source:** `hp41-cli/src/app.rs` lines 848-864 (EEX-CHS)
**Apply to:** DISP-02 CHS mantissa toggle

Every in-buffer edit follows the same shape: check condition → mutate `self.state.entry_buf` → `self.message = None;` → `return;`. The DISP-02 block must follow this shape exactly to avoid falling through to `call_dispatch`.

---

## No Analog Found

All files have clear analogs. No files require falling back to RESEARCH.md patterns alone.

---

## Key Findings for Planner

1. **`format_hpnum` is in `hp41-core/src/format.rs`** (line 18, confirmed), called via `use hp41_core::{format_alpha, format_hpnum, AngleMode}` in `ui.rs`. Both CLI and GUI use this single definition — not duplicated per SC-4. After MATH-01, `format_hpnum` must handle the new `HpNum` struct where `n.inner()` is the mantissa (not the full value). This is an ADDITIONAL file the planner must include in the MATH-01 plan.

2. **DISP-03 GUI side goes in `types.rs`, NOT `App.tsx`** — the `display_str` field carries the AON string to the frontend; the App.tsx chain (`calcState.pending_yield?.text ?? calcState.display_override ?? calcState.display_str`) is unchanged.

3. **DISP-03 CLI side has a subtlety:** The flag-48 branch shows `format_alpha(&st.alpha_reg)` — the same data as the `alpha_mode` branch above it. The ordering `alpha_mode → flag48 → X` is correct per D-10: both show the alpha register, but `alpha_mode` is the active ALPHA entry state, while the flag-48 branch is the "at rest with AON" state.

4. **proptest `inner()` breakage:** After the struct change, `s_n.stack.x.inner()` in `proptest_math.rs` returns the `Decimal` mantissa (not the full value). Planner must add a `to_f64()` method to `HpNum` or update the proptest to use it. This is a Wave 0 gap from RESEARCH.md.

5. **CHS key is `'n'`** (confirmed: `keys.rs` line 117: `KeyCode::Char('n') => Some(Op::Chs)`). The DISP-02 condition is `c == 'n'` — identical to the EEX-CHS guard.

6. **ADR number is v4.3-003** (RESEARCH recommendation; confirmed the next available slot — existing ADRs in the v4.3 series are 001, 002, and the new alarm ADR is `v4.3-003-alarm-prefix-semantics.md`). Planner should verify the next available number.

---

## Metadata

**Analog search scope:** `hp41-core/src/`, `hp41-cli/src/`, `hp41-gui/src-tauri/src/`, `docs/adr/`, `hp41-core/tests/`
**Files read:** num.rs, format.rs, ui.rs, app.rs (targeted), types.rs (targeted), state.rs (targeted), display_ops.rs, math.rs (targeted), numerical_accuracy.rs (targeted), proptest_math.rs (targeted), v4.3-001-single-instance-guard.md
**Pattern extraction date:** 2026-06-07
