# Phase 2: Core Math - Research

**Researched:** 2026-05-06
**Domain:** Rust decimal arithmetic, HP-41 operation semantics, display formatting
**Confidence:** HIGH (codebase verified) / MEDIUM-HIGH (HP-41 behavior from manuals + emulator ecosystem)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Use `rust_decimal` with `features = ["maths"]` for ln/exp/pow/trig (ADR-001 specifies this for Phase 2)
- `AngleMode` enum (DEG/RAD/GRAD) stored as field in `CalcState`; default: DEG
- Domain errors return `Err(HpError::Domain)` — already exists in error.rs
- `DisplayMode` enum: `Fix(u8)`, `Sci(u8)`, `Eng(u8)` where u8 = digit count (0–9)
- Number entry via `entry_buf: String` in `CalcState`; committed on Enter or operation
- FIX mode shows trailing zeros (e.g., FIX 4 of `1` → `1.0000`)
- SCI mode uses uppercase E-notation (`1.23456789E-10`)
- `[HpNum; 100]` fixed array for R00–R99 (0-indexed)
- `Op::StoArith { reg: u8, kind: StoArithKind }` with `StoArithKind` enum (Add/Sub/Mul/Div)
- Out-of-range register returns `Err(HpError::InvalidOp)`
- All registers initialized to zero on startup
- `alpha_reg: String` in `CalcState`, 24-character maximum enforced
- `alpha_mode: bool` field in `CalcState` (simple flag; `CalcMode` enum deferred to Phase 3)
- Number-to-ALPHA conversion (`Op::AlphaFromX`) deferred — Phase 2 covers raw char entry only

### Claude's Discretion

- Internal helper for degree↔radian conversion inside trig dispatch
- `Op::FmtFix(u8)`, `Op::FmtSci(u8)`, `Op::FmtEng(u8)` variants for mode-switching ops
- Stack-lift semantics for all new ops must declare Enable/Disable/Neutral per Phase 1 convention

### Deferred Ideas (OUT OF SCOPE)

- `Op::AlphaFromX` (append X register as string to alpha_reg) — Phase 5 UX
- `CalcMode` enum (Normal/Alpha/Prgm) — `alpha_mode: bool` suffices for Phase 2
- ALPHA annunciator display — Phase 4 TUI
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MATH-01 | Core arithmetic: `+ − × ÷`, `1/x`, `√x`, `x²`, `Y^X`, `LN`, `LOG`, `e^x`, `10^x` with HP-41-accurate 10-digit results | `rust_decimal` maths feature provides ln, log10, exp, sqrt, powd; x² = x*x via existing checked_mul; all route through HpNum::rounded(10) |
| MATH-02 | Trig (SIN/COS/TAN + inverses) in DEG, RAD, GRAD modes | rust_decimal maths has sin/cos/tan; asin/acos/atan require f64 fallback; angle conversion helper converts to radians before calling |
| MATH-03 | FIX n, SCI n, ENG n display mode switching (n = 0–9) | DisplayMode enum + format_hpnum() function; overflow from FIX to SCI is standard HP behavior |
| REGS-01 | R00–R99 STO, RCL, STO+/−/×/÷ matching HP-41 hardware | `[HpNum; 100]` in CalcState; STO/STO-arith = Neutral; RCL = Enable |
| ALPH-01 | 24-char ALPHA register via ALPHA mode keyboard input | alpha_reg: String with 24-char cap; Op::AlphaAppend(char) + Op::AlphaClear + Op::AlphaToggle |
</phase_requirements>

---

## Summary

Phase 2 adds all math functions, display formatting, storage registers, and ALPHA mode to the HP-41 emulator core. The foundation from Phase 1 (HpNum, Stack, CalcState, dispatch, LiftEffect) is solid and well-tested. Phase 2 extends each without breaking the existing interface.

The biggest technical risk is inverse trig: `rust_decimal 1.41`'s `maths` feature provides `sin`, `cos`, `tan` natively (Taylor series, decimal-native), but does **not** provide `asin`, `acos`, or `atan`. These three must be implemented via an f64 round-trip: `Decimal → f64 → libm → f64 → HpNum::rounded(10)`. At 10 significant digits this round-trip is accurate — f64 has ~15.9 decimal digits of precision, so rounding to 10 digits after the f64 inverse trig call is sufficient. This approach should pass QUAL-06 (≥98% accuracy vs hardware), but must be flagged as a Phase 7 validation target.

Display formatting (FIX/SCI/ENG) and number entry buffer (`entry_buf`) are pure Rust formatting logic — no external dependencies needed. The HP-41 display falls back from FIX to SCI when numbers are too large or too small to represent in the requested decimal places, which is a formatting decision not a data change.

**Primary recommendation:** Add `features = ["maths"]` to hp41-core/Cargo.toml, implement asin/acos/atan via f64 round-trip with HpNum::rounded(), extend CalcState with all Phase 2 fields, and add ops/math.rs + ops/registers.rs + ops/alpha.rs as new submodules.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| HpNum math methods (ln, sin, sqrt, etc.) | hp41-core library | — | All arithmetic lives in core, zero UI deps |
| Angle mode conversion (DEG/RAD/GRAD) | hp41-core library | — | Pure numeric transformation, part of CalcState |
| Display formatting (FIX/SCI/ENG) | hp41-core library | hp41-cli (rendering) | Core produces the formatted string; CLI renders it |
| Number entry buffer | hp41-core library | hp41-cli (keypress feed) | entry_buf lives in CalcState; CLI feeds keystrokes |
| Storage registers R00–R99 | hp41-core library | — | Part of CalcState, accessed only via Op dispatch |
| ALPHA register | hp41-core library | hp41-cli (display) | alpha_reg lives in CalcState; CLI shows it in display |

---

## Standard Stack

### Core Dependencies

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rust_decimal | 1.41.0 | Decimal arithmetic + maths functions | ADR-001 locked; already in workspace |
| thiserror | 2.0 | Error enum derives | Already in workspace |

**Cargo.toml change required** — hp41-core/Cargo.toml currently has:
```toml
rust_decimal = { workspace = true }
```
Must become:
```toml
rust_decimal = { workspace = true, features = ["maths"] }
```

The `maths-nopanic` sub-feature is NOT needed because the plan is to use the `checked_*` variants throughout (checked_ln, checked_exp, checked_sin, etc.) which return `Option<Decimal>` instead of panicking.

### Supporting (dev-dependencies, already present)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| proptest | 1.11 | Property-based testing | Stack-lift invariants, register round-trips |
| insta | 1.47 | Snapshot testing | Display formatting output, golden reference values |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| f64 round-trip for asin/acos/atan | Custom Taylor series in Decimal | Taylor series for arcsin/arccos converges slowly near ±1; f64+round is simpler and sufficient at 10 sig digits |
| `[HpNum; 100]` fixed array | `Vec<HpNum>` or `HashMap<u8, HpNum>` | Fixed array matches HP-41 fixed register file; zero allocation; trivially serializable; no bounds ambiguity |
| `entry_buf: String` | `entry_buf: Vec<u8>` | String is cleaner for parse(); digits are always ASCII |

### Installation

```bash
# No new packages — just add feature flag to existing dependency
# In hp41-core/Cargo.toml, change:
rust_decimal = { workspace = true }
# To:
rust_decimal = { workspace = true, features = ["maths"] }
```

**Version verification:** `rust_decimal 1.41.0` confirmed via `cargo search rust_decimal` and `cargo tree -p hp41-core`. [VERIFIED: cargo registry]

---

## Architecture Patterns

### System Architecture Diagram

```
Keypress (hp41-cli)
       │
       ▼
  entry_buf flush?  ──yes──► parse String → HpNum → Op::PushNum → dispatch()
       │ no
       ▼
  Op variant created (e.g., Op::Sin, Op::StoReg{reg:5}, Op::AlphaAppend('A'))
       │
       ▼
  dispatch(&mut CalcState, op)
       │
       ├── ops/math.rs    ─── unary_result() ──► HpNum method ──► HpNum::rounded(10)
       │                                         (maths feature or f64 round-trip)
       │
       ├── ops/registers.rs ─ read/write CalcState.regs[reg as usize]
       │
       ├── ops/alpha.rs    ─── append/clear CalcState.alpha_reg
       │
       └── ops/arithmetic.rs (Phase 1, binary_result())
              │
              └── apply_lift_effect(state, LiftEffect::Enable/Disable/Neutral)
                         │
                         └── CalcState.stack.lift_enabled updated
```

### Recommended Project Structure

```
hp41-core/src/
├── lib.rs              # Public re-exports (unchanged)
├── state.rs            # CalcState — gains regs, alpha_reg, alpha_mode,
│                       #             angle_mode, display_mode, entry_buf
├── num.rs              # HpNum — gains sin/cos/tan/asin/acos/atan/ln/log/exp/sqrt/recip/sq
├── stack.rs            # LiftEffect, binary_result, unary_result (new helper)
├── error.rs            # HpError (unchanged)
├── format.rs           # NEW: format_hpnum(n: &HpNum, mode: &DisplayMode) -> String
└── ops/
    ├── mod.rs          # Op enum + dispatch() — gains ~30 new variants
    ├── arithmetic.rs   # Phase 1 binary ops (unchanged)
    ├── stack_ops.rs    # Phase 1 stack ops (unchanged)
    ├── math.rs         # NEW: op_recip, op_sqrt, op_sq, op_ypow, op_ln, op_log,
    │                   #      op_exp, op_tenpow, op_sin, op_cos, op_tan,
    │                   #      op_asin, op_acos, op_atan, op_angle_mode
    ├── registers.rs    # NEW: op_sto, op_rcl, op_sto_arith, op_clreg
    └── alpha.rs        # NEW: op_alpha_toggle, op_alpha_append, op_alpha_clear
```

### Pattern 1: Unary Result Helper

All unary operations consume X and produce a result in X without touching Y/Z/T. A new helper `unary_result()` mirrors `binary_result()` but without stack drop:

```rust
// Source: modeled on existing binary_result() in stack.rs — [CITED: hp41-core/src/stack.rs]
pub fn unary_result(state: &mut CalcState, result: HpNum) {
    // Unary ops save X to LASTX before overwriting — HP-41 hardware behavior
    state.stack.lastx = state.stack.x.clone();
    state.stack.x = result;
    // Y, Z, T are NOT modified by unary operations
    state.stack.lift_enabled = true;  // all unary results enable lift
}
```

### Pattern 2: HpNum Math Method via rust_decimal maths feature

```rust
// Source: docs.rs/rust_decimal/latest/rust_decimal/trait.MathematicalOps.html [CITED]
use rust_decimal::MathematicalOps;

impl HpNum {
    pub fn checked_sin(&self) -> Result<HpNum, HpError> {
        self.0.checked_sin()
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    pub fn checked_ln(&self) -> Result<HpNum, HpError> {
        if self.0 <= Decimal::ZERO {
            return Err(HpError::Domain);
        }
        self.0.checked_ln()
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }
}
```

### Pattern 3: asin/acos/atan via f64 Round-Trip

`rust_decimal` maths feature does **not** provide asin, acos, or atan. [VERIFIED: docs.rs/rust_decimal MathematicalOps trait]. Use f64 round-trip:

```rust
// [ASSUMED] Approach — the accuracy trade-off is documented in the Pitfalls section
impl HpNum {
    pub fn checked_asin(&self) -> Result<HpNum, HpError> {
        let v = self.0.to_f64().ok_or(HpError::Overflow)?;
        if v < -1.0 || v > 1.0 {
            return Err(HpError::Domain);
        }
        let result = v.asin(); // f64 asin — ~15.9 digits precision
        Decimal::from_f64(result)
            .map(HpNum::rounded)   // round to 10 sig digits
            .ok_or(HpError::Overflow)
    }
    // acos, atan follow same pattern; atan has no domain restriction
}
```

### Pattern 4: Trig Dispatch with Angle Mode Conversion

```rust
// [ASSUMED] Implementation pattern — correct HP-41 behavior from manual research
fn to_radians(x: &HpNum, mode: AngleMode) -> Result<HpNum, HpError> {
    match mode {
        AngleMode::Rad  => Ok(x.clone()),
        AngleMode::Deg  => x.checked_mul(&HpNum::PI_OVER_180),
        AngleMode::Grad => x.checked_mul(&HpNum::PI_OVER_200),
    }
}

fn from_radians(x: &HpNum, mode: AngleMode) -> Result<HpNum, HpError> {
    match mode {
        AngleMode::Rad  => Ok(x.clone()),
        AngleMode::Deg  => x.checked_mul(&HpNum::DEG_PER_RAD),
        AngleMode::Grad => x.checked_mul(&HpNum::GRAD_PER_RAD),
    }
}

pub fn op_sin(state: &mut CalcState) -> Result<(), HpError> {
    let radians = to_radians(&state.stack.x, state.angle_mode)?;
    let result = radians.checked_sin()?;
    unary_result(state, result);
    Ok(())
}
```

### Pattern 5: entry_buf Flush

When any non-digit op arrives while `entry_buf` is non-empty, parse and push first:

```rust
// [ASSUMED] Standard HP entry buffer pattern
pub fn flush_entry_buf(state: &mut CalcState) -> Result<(), HpError> {
    if state.entry_buf.is_empty() {
        return Ok(());
    }
    let s = state.entry_buf.clone();
    state.entry_buf.clear();
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    let n = HpNum::rounded(d);
    enter_number(state, n);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

### Pattern 6: Storage Register Ops

```rust
// [ASSUMED] STO/RCL pattern consistent with HP-41 hardware behavior
pub fn op_sto(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 { return Err(HpError::InvalidOp); }
    state.regs[reg as usize] = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);  // STO does not affect lift
    Ok(())
}

pub fn op_rcl(state: &mut CalcState, reg: u8) -> Result<(), HpError> {
    if reg >= 100 { return Err(HpError::InvalidOp); }
    let val = state.regs[reg as usize].clone();
    enter_number(state, val);      // respects lift_enabled for the push
    apply_lift_effect(state, LiftEffect::Enable);  // RCL enables lift
    Ok(())
}

pub fn op_sto_arith(state: &mut CalcState, reg: u8, kind: StoArithKind)
    -> Result<(), HpError>
{
    if reg >= 100 { return Err(HpError::InvalidOp); }
    let new_val = match kind {
        StoArithKind::Add => state.regs[reg as usize].checked_add(&state.stack.x)?,
        StoArithKind::Sub => state.regs[reg as usize].checked_sub(&state.stack.x)?,
        StoArithKind::Mul => state.regs[reg as usize].checked_mul(&state.stack.x)?,
        StoArithKind::Div => state.regs[reg as usize].checked_div(&state.stack.x)?,
    };
    state.regs[reg as usize] = new_val;
    apply_lift_effect(state, LiftEffect::Neutral);  // STO arith never affects lift
    Ok(())
}
```

### Anti-Patterns to Avoid

- **Calling `sin()`/`ln()` directly on Decimal**: These panic on invalid input. Always use `checked_sin()`, `checked_ln()`, etc.
- **Skipping domain checks before checked_***: Even checked variants return `None` for overflow, not just domain errors. Guard Domain errors yourself (e.g., ln of negative = Domain, not just None).
- **Using f64 for register values**: All register arithmetic must flow through HpNum checked methods — no raw f64 intermediate math.
- **Implementing x² as `x.pow(2)`**: Use `state.stack.x.checked_mul(&state.stack.x)` — avoids the maths feature's power path entirely (simpler, already tested, no precision loss).
- **Angle conversion after trig call**: Convert to radians BEFORE calling sin/cos/tan. Converting the result is wrong and produces junk.
- **entry_buf containing EEX state as two separate strings**: HP-41 EEX mode appends to the same string buffer using 'E' as separator. A simple `"1.23E4"` string parses cleanly via `Decimal::from_str`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| sin/cos/tan | Taylor series in Decimal | `MathematicalOps::checked_sin/cos/tan` | Already implemented in rust_decimal maths feature with proper Maclaurin series and angle reduction |
| ln, log10, exp, 10^x, sqrt | Custom iterative | `MathematicalOps::checked_ln/log10/exp/sqrt` | Babylonian/Taylor already in rust_decimal; edge cases handled |
| x^y (Y^X) | Custom power | `MathematicalOps::checked_powd` | Handles integer and fractional exponents; uses e^(y*ln(x)) for non-whole y |
| Display number formatting | Regex/manual string building | Rust's built-in `format!("{:.nf}")` + custom ENG exponent logic | FIX and SCI are trivially expressible; only ENG exponent clamping needs custom code |
| Decimal-to-string parsing for entry_buf | Manual digit parsing | `Decimal::from_str(&entry_buf)` | Standard parse handles sign, decimal point, E-notation |

**Key insight:** rust_decimal's maths feature covers ~80% of Phase 2 math needs. Only asin/acos/atan require custom handling, and even there the implementation is a 3-line f64 bridge, not a full algorithm.

---

## HP-41 Operation Table: Phase 2 Ops

### Stack-Lift and LASTX Behavior Rules

From HP-41 hardware documentation research [CITED: archived.hpcalc.org/greendyk/hp41c-manual/]:
- **All unary math functions** (1/x, √x, x², LN, LOG, e^x, 10^x, SIN, COS, TAN, ASIN, ACOS, ATAN): Enable lift, save X to LASTX, only modify X register
- **Binary math** (Y^X): Enable lift, save X to LASTX (same as +, -, *, /)
- **STO, STO+/-/×/÷**: Neutral — never modify lift flag; do NOT save to LASTX
- **RCL**: Enable lift — pushes value onto stack; does NOT save to LASTX
- **FmtFix/FmtSci/FmtEng, AngleMode set**: Neutral — mode-only changes
- **AlphaToggle, AlphaAppend, AlphaClear**: Neutral

### Complete Phase 2 Op Table

| Op Variant | HP-41 Name | Category | Stack Effect | LASTX | Lift Effect |
|------------|------------|----------|-------------|-------|-------------|
| `Op::Recip` | 1/x | Unary math | X ← 1÷X | Saves X | Enable |
| `Op::Sqrt` | √x | Unary math | X ← √X | Saves X | Enable |
| `Op::Sq` | x² | Unary math | X ← X×X | Saves X | Enable |
| `Op::YPow` | Y^X | Binary math | X ← Y^X, stack drops | Saves X | Enable |
| `Op::Ln` | LN | Unary math | X ← ln(X) | Saves X | Enable |
| `Op::Log` | LOG | Unary math | X ← log₁₀(X) | Saves X | Enable |
| `Op::Exp` | e^x | Unary math | X ← e^X | Saves X | Enable |
| `Op::TenPow` | 10^x | Unary math | X ← 10^X | Saves X | Enable |
| `Op::Sin` | SIN | Unary math | X ← sin(X[angle_mode]) | Saves X | Enable |
| `Op::Cos` | COS | Unary math | X ← cos(X[angle_mode]) | Saves X | Enable |
| `Op::Tan` | TAN | Unary math | X ← tan(X[angle_mode]) | Saves X | Enable |
| `Op::Asin` | ASIN | Unary math | X ← asin(X) in angle_mode | Saves X | Enable |
| `Op::Acos` | ACOS | Unary math | X ← acos(X) in angle_mode | Saves X | Enable |
| `Op::Atan` | ATAN | Unary math | X ← atan(X) in angle_mode | Saves X | Enable |
| `Op::StoReg(u8)` | STO nn | Register | R[n] ← X, stack unchanged | No | Neutral |
| `Op::RclReg(u8)` | RCL nn | Register | X ← R[n] (with lift if enabled) | No | Enable |
| `Op::StoArith{reg,kind}` | STO+ / STO- / STO× / STO÷ | Register | R[n] ← R[n] OP X | No | Neutral |
| `Op::FmtFix(u8)` | FIX n | Display | Sets display_mode only | No | Neutral |
| `Op::FmtSci(u8)` | SCI n | Display | Sets display_mode only | No | Neutral |
| `Op::FmtEng(u8)` | ENG n | Display | Sets display_mode only | No | Neutral |
| `Op::SetDeg` | DEG | Angle mode | Sets angle_mode = DEG | No | Neutral |
| `Op::SetRad` | RAD | Angle mode | Sets angle_mode = RAD | No | Neutral |
| `Op::SetGrad` | GRAD | Angle mode | Sets angle_mode = GRAD | No | Neutral |
| `Op::AlphaToggle` | ALPHA | ALPHA | Flips alpha_mode bool | No | Neutral |
| `Op::AlphaAppend(char)` | (key in alpha mode) | ALPHA | alpha_reg.push(ch), max 24 | No | Neutral |
| `Op::AlphaClear` | CLRALPHA | ALPHA | alpha_reg.clear() | No | Neutral |

**Note on binary_result() vs unary_result():** Y^X uses `binary_result()` (drops Y, saves X to LASTX). All other math ops use `unary_result()` (keeps Y/Z/T, saves X to LASTX).

---

## HP-41 Display Formatting Rules

### FIX n (Fixed Decimal)

Format: `±ddddddddd.nnn...` where n digits appear after the decimal.

- n = 0 to 9 (u8 digit count in DisplayMode::Fix(u8))
- Trailing zeros ARE shown: `FIX 4` of `1` → `1.0000` [CITED: CONTEXT.md locked decision]
- Negative numbers: leading minus, e.g. `-1.2345`
- **Overflow to SCI**: When the number is too large or too small to represent in FIX format (i.e., the number would require more than 10 significant digits to show the integer part, or the number is so small that all significant digits would be hidden as trailing zeros), the HP-41 automatically falls back to SCI display. [CITED: HP-33S manual from manualslib.com + HP Museum forum research]
- The HP-41 display is 12 characters total (10 mantissa + 2 exponent); FIX output must fit within this width

**FIX overflow threshold rule** [ASSUMED — needs hardware verification]:
- If the absolute value requires an integer part with > (10 - n) digits, display in SCI instead
- If the absolute value is < 10^(-(n+10)), display in SCI instead (all significant digits would be lost)
- Example: `FIX 4`, number = `1234567890.1234` has 10-digit integer part → overflow to SCI

### SCI n (Scientific Notation)

Format: `±d.nnn...E±ee` where n digits after the decimal (total n+1 significant digits).

- HP-41 uses uppercase `E` [CITED: CONTEXT.md locked decision]
- Examples: `SCI 4` of `299792500` → `2.9979E 08` (HP-41 format: space before exponent if positive)
- n = 0 to 9; `SCI 0` → `3.E08` (one significant digit only)
- Exponent is 2 digits, zero-padded: `E 01` not `E 1`; `E-04` for negative exponents
- Range: exponent -99 to +99

### ENG n (Engineering Notation)

Format: same as SCI but exponent is always a multiple of 3; mantissa may be 1, 2, or 3 digits before the decimal.

- The n parameter = number of significant digits AFTER the first significant digit — effectively (n+1) total significant digits in the mantissa [CITED: HP Museum forum + HP-33s manual research]
- Exponent choices: ... -9, -6, -3, 0, 3, 6, 9, 12 ...
- Examples:
  - `ENG 3` of `12345.678` → `12.346E 03` (3+1=4 sig digits, exponent rounded to 3)
  - `ENG 3` of `0.001234` → `1.234E-03`
  - `ENG 3` of `1000000` → `1.000E 06`
- The mantissa is 1–3 digits before the decimal depending on which multiple-of-3 exponent is chosen
- ENG 0: one significant digit with multiple-of-3 exponent

### Display String for ALPHA Mode

When `alpha_mode = true`, the display shows `alpha_reg` contents rather than the numeric X register. This is a formatting concern — the format module should expose both:
- `format_hpnum(n: &HpNum, mode: &DisplayMode) -> String` — numeric display
- `format_alpha(reg: &str) -> String` — alpha display (direct truncation to 12 chars if needed)

### Overflow and Special Values

- HP-41 maximum displayable magnitude: `9.999999999 × 10^99` [CITED: HP Museum research]
- HP-41 minimum displayable magnitude: `1 × 10^-99`
- Numbers outside this range: `Err(HpError::Overflow)` from HpNum arithmetic (already handled)
- Zero always displays as `0.0000` in FIX 4, `0.0000E 00` in SCI 4, etc.

---

## Number Entry State Machine

The HP-41 uses a character accumulation buffer. The `entry_buf: String` in `CalcState` is the pending digit string. Rules [CITED: HP-41C manual research + HP Museum discussions]:

### States

```
[idle]          entry_buf == ""
[entering]      entry_buf != "" and !has_eex
[entering_exp]  entry_buf contains 'E' (after EEX pressed)
```

### Key Transitions

| Key | State: idle | State: entering | State: entering_exp |
|-----|-------------|-----------------|---------------------|
| Digit 0–9 | `entry_buf = "digit"` | append digit to mantissa | append digit to exponent |
| `.` (decimal) | `entry_buf = "."` | append if no `.` yet; ignore if already has `.` | ignored |
| `EEX` | `entry_buf = "1E"` | append `E` to buf (creates `"1.23E"`) | ignored |
| `CHS` | no-op (negate X via op_chs) | negate mantissa prefix | negate exponent sign |
| `BKSP/←` | no-op | remove last char; if buf empty→idle | remove last exponent digit |
| Any math op | no-op | flush_entry_buf() then execute op | same |
| ENTER | no-op | flush_entry_buf() then op_enter | same |

### Flush Rule

`flush_entry_buf()` must be called at the START of `dispatch()` when `entry_buf` is non-empty, BEFORE routing to the op handler. This ensures the number gets pushed before the op consumes it.

**Alternative location**: The CLI can flush before calling dispatch. Either works; keeping it in dispatch() makes the library self-contained and simpler for CLI code.

### Entry Buf Format Parsed by Decimal::from_str

`Decimal::from_str` accepts:
- `"1234"` — integer
- `"1.234"` — decimal
- `"1.234E5"` — scientific (uppercase or lowercase E)
- `"-1.234"` — negative

This means `entry_buf` can be passed directly to `Decimal::from_str` after CHS has been applied to the mantissa sign in the string.

---

## Storage Registers (REGS-01)

### CalcState Changes

```rust
// In state.rs
pub struct CalcState {
    pub stack: Stack,
    pub regs: [HpNum; 100],          // R00–R99, 0-indexed
    pub alpha_reg: String,            // max 24 chars
    pub alpha_mode: bool,
    pub angle_mode: AngleMode,
    pub display_mode: DisplayMode,
    pub entry_buf: String,
}
```

`[HpNum; 100]` requires `HpNum: Default`. Add `impl Default for HpNum { fn default() -> Self { HpNum::zero() } }`.

### StoArithKind Enum

```rust
// In ops/registers.rs or a shared types module
#[derive(Debug, Clone, PartialEq)]
pub enum StoArithKind { Add, Sub, Mul, Div }
```

### Register Indexing

The HP-41 registers are numbered R00–R99 from the user's perspective. These map directly to `regs[0]` through `regs[99]`. [CITED: HP-41C manual research — "primary registers R00-R62 (standard configuration)" and "R63-R99 with optional memory"]

The CONTEXT.md decision is 0-indexed internally, which matches: user types `STO 05` → `reg = 5` → `regs[5]`. No offset needed.

### CLREG (Clear All Registers)

HP-41 has a CLREG function that zeros all storage registers. Add `Op::Clreg` that sets `state.regs = [HpNum::zero(); 100]` (or equivalent array initialization). Neutral lift effect.

---

## ALPHA Mode (ALPH-01)

### CalcState Fields

```rust
pub alpha_reg: String,  // Max 24 characters — HP-41 hardware limit
pub alpha_mode: bool,   // true = keyboard sends alpha chars, not ops
```

### Op Variants

```rust
Op::AlphaToggle          // flip alpha_mode; Neutral lift
Op::AlphaAppend(char)    // push char to alpha_reg (if len < 24); Neutral lift
Op::AlphaClear           // alpha_reg.clear(); Neutral lift
```

### 24-Character Enforcement

```rust
pub fn op_alpha_append(state: &mut CalcState, ch: char) -> Result<(), HpError> {
    if state.alpha_reg.chars().count() >= 24 {
        // HP-41 silently discards excess characters — no error
        // [ASSUMED] Exact behavior: discard vs. error — hardware likely discards
    } else {
        state.alpha_reg.push(ch);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

The CONTEXT.md specifies 24-char max enforced but does not specify the overflow behavior. [ASSUMED: silent discard, consistent with HP hardware behavior on most implementations].

### ALPHA Mode and Number Entry

When `alpha_mode = true` and the user presses a digit key, the CLI should route to `Op::AlphaAppend('5')` rather than appending to `entry_buf`. This is a CLI routing decision, not a core concern.

---

## Cargo.toml Changes Required

### hp41-core/Cargo.toml

```toml
[dependencies]
rust_decimal = { workspace = true, features = ["maths"] }
thiserror = { workspace = true }
```

**The workspace Cargo.toml does NOT need to change** — the `features = ["maths"]` is crate-local. The workspace declaration `rust_decimal = "1.41"` remains as-is.

### No Other Dependency Changes

- `num-traits` is already a transitive dependency of rust_decimal and provides `ToPrimitive::to_f64()` for the asin/acos/atan f64 bridge
- No `libm` crate needed — standard Rust `f64::asin()`, `f64::acos()`, `f64::atan()` are sufficient
- No formatting library needed — standard `format!()` handles FIX and SCI; ENG needs ~20 lines of custom exponent rounding

---

## Common Pitfalls

### Pitfall 1: LASTX Not Saved for Unary Ops — The Most Common HP Emulator Bug

**What goes wrong:** Developer implements unary operations without saving to LASTX (only does it for binary ops since the existing `binary_result()` handles it). User presses `SIN`, then `LASTX` — gets wrong value.

**Why it happens:** The Phase 1 pattern `binary_result()` handles LASTX only for binary ops. New unary ops that use a simple `state.stack.x = result` miss the LASTX save.

**How to avoid:** Always use `unary_result()` for all unary math operations. This helper saves X to LASTX, sets X to result, and enables lift — making the behavior impossible to get wrong by omission.

**Warning signs:** Test `5 SIN LASTX` — result must be `5`, not the sine of 5.

### Pitfall 2: rust_decimal Panics on Domain Error Without Checked Variants

**What goes wrong:** Calling `.ln()` instead of `.checked_ln()` on a negative or zero Decimal panics at runtime, violating the core invariant "zero panics in hp41-core".

**Why it happens:** rust_decimal's non-checked math functions panic on invalid input. The documentation notes: "ln and log10 will panic on invalid input". [CITED: docs.rs/rust_decimal]

**How to avoid:** Never call the non-checked variants. Use `checked_ln()`, `checked_exp()`, `checked_sin()`, etc. exclusively. The domain guards (negative ln, asin outside ±1) must be checked before calling checked_* because checked_* returns None for BOTH domain errors AND overflow, losing the distinction.

**Warning signs:** Any direct `.sin()`, `.ln()`, `.log10()`, `.exp()` call in math.rs without `checked_` prefix.

### Pitfall 3: Angle Mode Not Applied to Inverse Trig Output

**What goes wrong:** `asin` returns a result in radians (from f64), but the HP-41 should return the result in the current angle mode. Developer forgets to convert the radians result back to degrees/grads.

**Why it happens:** Forward trig (sin/cos/tan) requires converting INPUT from angle_mode to radians. Inverse trig (asin/acos/atan) requires converting OUTPUT from radians to angle_mode. These are opposite directions.

**How to avoid:** The `from_radians()` helper must be applied to the asin/acos/atan result before placing it in X. The `to_radians()` helper must be applied to the input for sin/cos/tan.

**Warning signs:** `45 ASIN` in DEG mode should give `1.000000000` (i.e., 1.0), not `0.7853981634`. Test `1 ASIN` in DEG — must yield `90.0`.

### Pitfall 4: ENG Format Digit Count Parameter Meaning

**What goes wrong:** Implementing `ENG n` as "n digits total" when HP-41 means "n digits AFTER the first significant digit" (i.e., n+1 total significant digits, with 1–3 before the decimal depending on exponent).

**Why it happens:** SCI n means "n decimal places (total n+1 sig digits)"; ENG n means the same mathematically — but the visual layout differs because ENG can have 1, 2, or 3 digits before the decimal.

**How to avoid:** The rule for ENG n: round the number to (n+1) significant digits, then find the largest multiple-of-3 exponent that makes the mantissa ≥ 1 and < 1000. [CITED: HP Museum forum research + HP-33S manual research]

### Pitfall 5: CHS During Number Entry vs Post-Entry

**What goes wrong:** CHS is implemented only as `negate X register`. But during digit entry, CHS must negate the mantissa in `entry_buf` (prepend/remove `-`) and after EEX is pressed, CHS must negate the exponent part.

**Why it happens:** CHS is a Neutral lift operation (Phase 1) that operates on the X register. But it also modifies `entry_buf` when entry is in progress — two different code paths.

**How to avoid:** Dispatch must check `entry_buf` state first: if non-empty, CHS modifies the string; if empty, CHS negates X via the existing op_chs path.

**Warning signs:** Try: enter `3`, press `CHS` → should show `-3` without pushing. Then `EEX`, `5`, `CHS` → should show `3E-5`, not `3E5` or `-3E5`.

### Pitfall 6: STO Arith Overflow Propagation

**What goes wrong:** `STO+ 05` with X=1e99 and R05=1e99 causes overflow. The developer returns `Err(HpError::Overflow)` but the register is now in an inconsistent state (partially written or cleared).

**Why it happens:** The `checked_add` result must be stored atomically — only write to `state.regs[reg]` AFTER the checked operation succeeds.

**How to avoid:** Compute the result first, then assign: `let new_val = op?; state.regs[reg] = new_val;` — never write the register before knowing the result is valid.

### Pitfall 7: CalcState Default for [HpNum; 100]

**What goes wrong:** `[HpNum; 100]` requires `HpNum: Copy` OR `HpNum: Default + Clone` for array initialization. Without `Default`, `CalcState::new()` cannot initialize the array with `Default::default()`.

**Why it happens:** Rust's `[T; N]` array default initialization requires either a const initializer or `T: Copy + Default` (for the `[x; N]` syntax).

**How to avoid:** Add `impl Default for HpNum { fn default() -> Self { HpNum::zero() } }` and use `std::array::from_fn(|_| HpNum::default())` or `[HpNum::zero(); 100]` (requires `Copy`). Since Decimal likely implements Copy, this may work — but verify. Alternative: `std::array::from_fn(|_| HpNum::zero())` works without Copy.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| f64 for all calculator math | rust_decimal for exact decimal arithmetic | Phase 1 ADR-001 | No binary rounding artifacts; 1/3 × 3 = 1 not 0.9999... |
| Hand-roll all math | rust_decimal maths feature | Phase 2 | ~80% of Phase 2 math functions come free from the feature flag |
| Dynamic dispatch (`dyn Trait`) for ops | Enum + match (Op enum) | Phase 1 ADR | Zero allocation per op; fully serializable; exhaustive match catches missed ops |

**Deprecated/outdated:**
- `MathematicalOps::erf` / `norm_pdf` / `norm_cdf`: In rust_decimal maths but irrelevant to HP-41 Phase 2 (may become useful in Phase 6 for statistics)
- `powf(f64)`: Takes f64 exponent; use `powd(Decimal)` for HP-41 Y^X to maintain decimal precision

---

## Runtime State Inventory

Phase 2 is a greenfield extension (new fields, new ops). No rename/refactor involved.

**Nothing found in any category** — verified by examining Phase 2 scope: adds new CalcState fields and Op variants. Does not rename, migrate, or replace any existing identifiers.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | Build | ✓ | (workspace uses 1.78+) | — |
| cargo | Build/test | ✓ | part of toolchain | — |
| just | CI/task runner | ✓ | Justfile present | — |
| rust_decimal maths feature | sin/cos/tan/ln etc. | ✓ (after Cargo.toml edit) | 1.41.0 | — |

All dependencies available. No blocking gaps.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + proptest 1.11 + insta 1.47 |
| Config file | none — cargo discovers tests automatically |
| Quick run command | `cargo test -p hp41-core` |
| Full suite command | `just test` (all workspace) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MATH-01 | 1/x of 4 = 0.25; √4 = 2; LN(e) = 1; LOG(100) = 2 | unit | `cargo test -p hp41-core math_ops` | ❌ Wave 0 |
| MATH-01 | Y^X: 2^10 = 1024; 2^0.5 = 1.414213562 | unit | `cargo test -p hp41-core math_ops::test_ypow` | ❌ Wave 0 |
| MATH-01 | 10-digit accuracy: LN(2) = 0.6931471806 | unit + insta snapshot | `cargo test -p hp41-core math_ops::test_ln_accuracy` | ❌ Wave 0 |
| MATH-01 | Stack-lift: all unary ops enable lift | unit | `cargo test -p hp41-core lift_tests` | ❌ Wave 0 (extend lift_tests.rs) |
| MATH-01 | LASTX saved by all unary ops | unit | `cargo test -p hp41-core lastx_tests` | ❌ Wave 0 |
| MATH-02 | SIN(30°) = 0.5 in DEG mode | unit | `cargo test -p hp41-core trig_tests::test_sin_deg` | ❌ Wave 0 |
| MATH-02 | ASIN(0.5) = 30 in DEG mode | unit | `cargo test -p hp41-core trig_tests::test_asin_deg` | ❌ Wave 0 |
| MATH-02 | SIN(π/6) in RAD mode = 0.5 | unit | `cargo test -p hp41-core trig_tests::test_sin_rad` | ❌ Wave 0 |
| MATH-02 | TAN(100g) = TAN(90°) in GRAD mode | unit | `cargo test -p hp41-core trig_tests::test_tan_grad` | ❌ Wave 0 |
| MATH-03 | FIX 4 of 3.14159 → "3.1416" | unit | `cargo test -p hp41-core format_tests::test_fix4` | ❌ Wave 0 |
| MATH-03 | SCI 4 of 299792500 → "2.9979E 08" | unit | `cargo test -p hp41-core format_tests::test_sci4` | ❌ Wave 0 |
| MATH-03 | ENG 3 of 12345 → "12.345E 03" | unit | `cargo test -p hp41-core format_tests::test_eng3` | ❌ Wave 0 |
| MATH-03 | FIX 4 overflow: 1e15 → SCI display | unit | `cargo test -p hp41-core format_tests::test_fix_overflow` | ❌ Wave 0 |
| REGS-01 | STO 5; RCL 5 round-trips X | unit | `cargo test -p hp41-core register_tests` | ❌ Wave 0 |
| REGS-01 | STO+ 5 performs R[5] += X | unit | `cargo test -p hp41-core register_tests::test_sto_add` | ❌ Wave 0 |
| REGS-01 | STO is Neutral lift; RCL is Enable | unit | `cargo test -p hp41-core lift_tests` (extend) | ❌ Wave 0 |
| ALPH-01 | AlphaAppend builds alpha_reg | unit | `cargo test -p hp41-core alpha_tests` | ❌ Wave 0 |
| ALPH-01 | 24-char limit enforced | unit | `cargo test -p hp41-core alpha_tests::test_24_char_limit` | ❌ Wave 0 |
| ALPH-01 | AlphaClear resets alpha_reg | unit | `cargo test -p hp41-core alpha_tests::test_alpha_clear` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-core` (< 5 seconds, all unit tests)
- **Per wave merge:** `just test` (full workspace including proptest)
- **Phase gate:** `just ci` (lint + test + coverage ≥ 80%) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-core/tests/math_tests.rs` — covers MATH-01 (all unary math ops)
- [ ] `hp41-core/tests/trig_tests.rs` — covers MATH-02 (all trig ops + angle modes)
- [ ] `hp41-core/tests/format_tests.rs` — covers MATH-03 (FIX/SCI/ENG + edge cases)
- [ ] `hp41-core/tests/register_tests.rs` — covers REGS-01 (STO/RCL/STO-arith)
- [ ] `hp41-core/tests/alpha_tests.rs` — covers ALPH-01 (ALPHA mode)
- [ ] Extend `hp41-core/tests/lift_tests.rs` — add lift tests for all ~24 new op variants
- [ ] `hp41-core/src/format.rs` — new module implementing format_hpnum()

---

## Security Domain

`security_enforcement` not set in config → treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A — local calculator, no auth |
| V3 Session Management | no | N/A |
| V4 Access Control | no | N/A |
| V5 Input Validation | yes (limited) | Decimal::from_str() for entry_buf; register index bounds check (reg < 100); alpha char validation |
| V6 Cryptography | no | N/A |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Integer overflow in register index | Tampering | `if reg >= 100 { return Err(HpError::InvalidOp) }` guard before array access |
| Panic from unchecked math | Denial of Service | Use checked_* variants exclusively; no raw .sin()/.ln() calls |
| Unbounded ALPHA string growth | Tampering | Hard cap at 24 chars; `if state.alpha_reg.chars().count() >= 24 { return Ok(()) }` |
| Decimal::from_str on adversarial input | Tampering | from_str returns Err on invalid input; map to HpError::InvalidOp |

Phase 2 is a local calculator library with no network access, authentication, or external data sources. Security surface is limited to input validation.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | asin/acos/atan via f64 round-trip is accurate enough at 10 sig digits (f64 has ~15.9 digits) | Architecture Patterns, Pattern 3 | Accuracy failure at QUAL-06 boundary cases near ±1 or near 0; fallback = custom Taylor series |
| A2 | HP-41 silently discards ALPHA append when alpha_reg is already 24 chars (no error) | ALPHA Mode section | Minor UX difference; error variant is also acceptable and easily changed |
| A3 | FIX overflow threshold: integer part requires > (10 - n) digits triggers SCI fallback | Display Formatting section | Wrong threshold would display numbers differently from hardware; verify with HP-41 hardware or V41 emulator |
| A4 | entry_buf flush should happen inside dispatch(), not in CLI layer | Number Entry section | Putting it in CLI works equally well; changes only caller responsibility, not correctness |
| A5 | Decimal implements Copy (needed for `[HpNum; 100]` syntax) | Storage Registers section | If Decimal is not Copy, use `std::array::from_fn(|_| HpNum::zero())` instead |
| A6 | HP-41 ENG n parameter means (n+1) total significant digits (same as SCI n) | Display Formatting section | Low risk — confirmed by multiple sources; display output differs only in mantissa-before-decimal count |

---

## Open Questions

1. **Accuracy of rust_decimal sin/cos/tan at HP-41-boundary values**
   - What we know: rust_decimal uses Maclaurin series with early-exit tolerance ~0.0000002; HP-41 has 10 sig digit display
   - What's unclear: whether the Taylor series precision matches HP-41 at extreme angles (near π/2 for tan, very large angles requiring multiple reduction steps)
   - Recommendation: Add insta snapshot tests for SIN(89°), SIN(89.9999°), TAN(89°) against known HP-41 hardware values; if they fail at digit 10, the f64 bridge approach may need to be used for all trig, not just inverse trig

2. **FIX overflow exact threshold**
   - What we know: HP-41 switches from FIX to SCI when number cannot be displayed in fixed format
   - What's unclear: the exact condition — is it when the integer part exceeds the display width, or when significant digits would be lost? What is the exact 12-character display width constraint?
   - Recommendation: Test with V41 emulator (open source): `FIX 4`, enter `9999999999` (10-digit integer) — does it show `9999999999.0000` or switch to SCI?

3. **CHS behavior during entry_buf with EEX**
   - What we know: CHS toggles the mantissa sign when not in EEX mode, and the exponent sign when in EEX mode
   - What's unclear: After pressing `3`, `EEX`, `5`, the buf is `"3E5"`. Pressing `CHS` — does it change the mantissa to `-3E5` or the exponent to `3E-5`? The HP-41 uses EEX to indicate "cursor is on exponent".
   - Recommendation: A `has_eex: bool` sub-flag in CalcState or a separate `eex_entered: bool` is needed to track this state. The string alone is insufficient. [ASSUMED: buf = `"3E5"`, CHS → `"3E-5"` (exponent sign change) if EEX was the last mode-changing key pressed]

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: cargo tree/cargo search] — rust_decimal 1.41.0, MathematicalOps trait verified via docs.rs; no asin/acos/atan confirmed
- [CITED: docs.rs/rust_decimal/latest/rust_decimal/trait.MathematicalOps.html] — complete method list for maths feature; confirmed sin/cos/tan present; asin/acos/atan absent
- [CITED: hp41-core/src/state.rs, num.rs, stack.rs, ops/*.rs] — all existing code patterns verified by direct read
- [CITED: Justfile] — `just test` = `cargo test --workspace`; `just ci` = lint + test + coverage

### Secondary (MEDIUM confidence)
- [CITED: archived.hpcalc.org/greendyk/hp41c-manual/] — HP-41C Owner's Handbook sections on display formats, stack structure, function behavior
- [CITED: manualslib.com/manual/742866/Hp-33s.html?page=35] — HP-33S FIX/SCI behavior (same philosophy as HP-41)
- [CITED: HP Museum forum research] — stack lift behavior for unary vs binary operations; ENG format rules
- [CITED: docs.rs/rust_decimal/latest/rust_decimal/] — Feature list and maths feature description

### Tertiary (LOW confidence — flagged as ASSUMED in Assumptions Log)
- FIX overflow exact threshold (A3) — inferred from multiple HP calculator manual descriptions, not HP-41-specific
- CHS during EEX state (A5/Q3) — inferred from HP-41 keyboard model, not directly documented in accessible sources
- AlphaAppend silent discard at 24 chars (A2) — inferred from HP-41 hardware behavior patterns

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — rust_decimal 1.41 verified; features confirmed via docs.rs
- Architecture: HIGH — extends well-defined Phase 1 patterns; all patterns match existing code
- HP-41 Op Semantics: MEDIUM-HIGH — stack-lift rules from HP-41 manual + HP Museum research; inverse trig gap confirmed
- Display Formatting: MEDIUM — FIX/SCI rules well documented; ENG digit-count semantics and FIX overflow threshold are ASSUMED (A3, A6)
- Pitfalls: HIGH — derived from verified code patterns and rust_decimal documentation

**Research date:** 2026-05-06
**Valid until:** 2026-08-06 (90 days; rust_decimal stable, HP-41 behavior immutable)
