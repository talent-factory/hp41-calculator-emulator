# Operations Reference

Complete reference for all built-in HP-41C/CV/CX operations.

**Stack-lift column:**
- `E` — Enable (next digit entry pushes stack)
- `D` — Disable (next digit entry overwrites X)
- `N` — Neutral (no change to current stack-lift state)

---

## Stack Operations

| Mnemonic | Description | Stack effect | Lift |
|----------|-------------|--------------|------|
| `ENTER↑` | Duplicate X into Y; lift Z→T | X→Y, X→X | D |
| `CLx` | Clear X register | 0→X | D |
| `R↓` (RDN) | Roll stack down: X→T, Y→X, Z→Y, T→Z | rotates | N |
| `R↑` | Roll stack up: T→X, X→Y, Y→Z, Z→T | rotates | N |
| `x↔y` | Swap X and Y | X↔Y | N |
| `LASTX` | Recall value of X before last operation | LAST X→X (stack lifts) | E |

---

## Arithmetic

| Mnemonic | Description | Stack effect | Lift |
|----------|-------------|--------------|------|
| `+` | Add | Y+X→X, stack drops | E |
| `−` | Subtract | Y−X→X, stack drops | E |
| `×` | Multiply | Y×X→X, stack drops | E |
| `÷` | Divide | Y÷X→X, stack drops | E |
| `+/−` (CHS) | Change sign of X | −X→X | N |

---

## Powers, Roots & Logarithms

| Mnemonic | Description | Stack effect | Lift |
|----------|-------------|--------------|------|
| `√x` | Square root of X | √X→X | E |
| `x²` | Square of X | X²→X | E |
| `yˣ` | Y to the power X | Yˣ→X, stack drops | E |
| `1/x` | Reciprocal of X | 1/X→X | E |
| `LOG` | Common log (base 10) | log(X)→X | E |
| `10ˣ` (ALOG) | Antilog base 10 | 10ˣ→X | E |
| `LN` | Natural logarithm | ln(X)→X | E |
| `eˣ` | Natural antilogarithm | eˣ→X | E |

---

## Trigonometry

Angle unit (DEG / RAD / GRAD) applies to all trig functions.

| Mnemonic | Description | Stack effect | Lift |
|----------|-------------|--------------|------|
| `SIN` | Sine | sin(X)→X | E |
| `COS` | Cosine | cos(X)→X | E |
| `TAN` | Tangent | tan(X)→X | E |
| `ASIN` | Arc sine | arcsin(X)→X | E |
| `ACOS` | Arc cosine | arccos(X)→X | E |
| `ATAN` | Arc tangent | arctan(X)→X | E |

---

## Hyperbolics (via X-Functions module / HP-41CX)

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `SINH` | Hyperbolic sine | E |
| `COSH` | Hyperbolic cosine | E |
| `TANH` | Hyperbolic tangent | E |
| `ASINH` | Inverse hyperbolic sine | E |
| `ACOSH` | Inverse hyperbolic cosine | E |
| `ATANH` | Inverse hyperbolic tangent | E |

---

## Number Entry & Display

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `EEX` | Begin exponent entry (×10ⁿ) | N |
| `←` | Backspace / delete last digit entered | N |
| `FIX n` | Fixed decimal notation, n decimal places (0–9) | N |
| `SCI n` | Scientific notation, n significant digits | N |
| `ENG n` | Engineering notation (exponent multiple of 3) | N |
| `ALL` | Display all available digits | N |
| `RDX,` | Set comma as decimal separator | N |
| `RDX.` | Set period as decimal separator | N |

---

## Angle Modes

| Mnemonic | Description |
|----------|-------------|
| `DEG` | Set degrees mode |
| `RAD` | Set radians mode |
| `GRAD` | Set gradians mode |

---

## Storage Registers

Registers are addressed 00–99 (physical limit depends on memory configuration).  
Indirect addressing: `IND nn` uses the integer part of register nn as the target address.

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `STO nn` | Store X in register nn | N |
| `STO+ nn` | Add X to register nn | N |
| `STO− nn` | Subtract X from register nn | N |
| `STO× nn` | Multiply register nn by X | N |
| `STO÷ nn` | Divide register nn by X | N |
| `RCL nn` | Recall register nn to X (stack lifts) | E |
| `RCL+ nn` | Add register nn to X | E |
| `RCL− nn` | Subtract register nn from X | E |
| `RCL× nn` | Multiply X by register nn | E |
| `RCL÷ nn` | Divide X by register nn | E |
| `CLRG` | Clear all data registers | D |
| `CLST` | Clear stack (X=Y=Z=T=0) | D |

---

## Loop Counters

Counter format: `SSSSS.FFFII` where SSS=start, FFF=end, II=increment (all as integer strings split at the decimal point — never extracted with `floor()`).

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `ISG nn` | Increment counter in nn; skip next step if counter > final value | N |
| `DSE nn` | Decrement counter in nn; skip next step if counter ≤ final value | N |

---

## Conditional Tests (skip next step if true)

### Stack comparisons

| Mnemonic | Condition |
|----------|-----------|
| `X=Y?` | X equals Y |
| `X≠Y?` | X not equal to Y |
| `X<Y?` | X less than Y |
| `X>Y?` | X greater than Y |
| `X≤Y?` | X less than or equal to Y |
| `X≥Y?` | X greater than or equal to Y |

### Zero comparisons

| Mnemonic | Condition |
|----------|-----------|
| `X=0?` | X equals 0 |
| `X≠0?` | X not equal to 0 |
| `X<0?` | X less than 0 |
| `X>0?` | X greater than 0 |
| `X≤0?` | X less than or equal to 0 |
| `X≥0?` | X greater than or equal to 0 |

All conditional tests: stack-lift **N**, do not modify registers.

---

## Flags

Flags 0–10 are user flags. Flags 11–55 are system flags (see Advanced Functions Handbook).

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `SF nn` | Set flag nn | N |
| `CF nn` | Clear flag nn | N |
| `FS? nn` | Skip next step if flag nn is set | N |
| `FC? nn` | Skip next step if flag nn is clear | N |
| `FS?C nn` | Skip next if flag nn set, then clear flag | N |
| `FC?C nn` | Skip next if flag nn clear, then clear flag | N |

---

## Program Control

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `LBL A`…`LBL J` | Named label (A–J, a–e) | N |
| `LBL 00`…`LBL 99` | Numbered label | N |
| `GTO lbl` | Unconditional branch to label | N |
| `GSB lbl` | Call subroutine at label (push return address) | N |
| `XEQ lbl` | Execute label (like GSB but for interactive use) | N |
| `RTN` | Return from subroutine | N |
| `END` | Mark end of program; RTN if reached during run | N |
| `STOP` | Halt execution; display X | D |
| `PSE` | Pause ~1 second, display X, then continue | D |
| `SST` | Single-step forward (manual execution) | — |
| `BST` | Single-step backward | — |
| `P/R` | Toggle PRGM mode | — |

---

## Alpha Register & String Operations

The alpha register holds up to 24 characters.

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `ALPHA` | Toggle alpha entry mode | N |
| `CLA` | Clear alpha register | N |
| `ASTO nn` | Store alpha register in register nn (6 chars/register) | N |
| `ARCL nn` | Append register nn contents to alpha register | N |
| `AVIEW` | Display alpha register; continue | N |
| `PROMPT` | Display alpha register; halt (awaits R/S) | D |
| `XTOA` | Convert X to ASCII character, append to alpha | N |
| `ATOX` | Convert first character of alpha to ASCII code in X | E |
| `ALENG` | Length of alpha register → X | E |
| `AROT` | Rotate alpha register left by X characters | N |

---

## Statistical Accumulation

The HP-41 uses registers R01–R06 for Σ-data (configurable via `ΣREG`).

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `Σ+` | Accumulate X, Y into statistics registers | D |
| `Σ−` | Remove X, Y from statistics registers | D |
| `MEAN` | Mean of X and Y data → X, Y | E |
| `SDEV` | Sample standard deviation → X, Y | E |
| `CORR` | Correlation coefficient → X | E |
| `LINF` | Linear regression: slope → X, intercept → Y | E |
| `CLΣ` | Clear statistics registers | D |
| `ΣREG` | Set first statistics register to X | N |
| `ΣREG?` | Recall first statistics register → X | E |

---

## Conversions

| Mnemonic | Description | Stack effect | Lift |
|----------|-------------|--------------|------|
| `→DEG` | Radians → degrees | X in rad → deg | E |
| `→RAD` | Degrees → radians | X in deg → rad | E |
| `→HMS` | Decimal hours → H.MMSSss format | X → H.MMSSss | E |
| `→H` | H.MMSSss → decimal hours | X → hours | E |
| `HMS+` | Add two H.MMSSss values | Y+X (HMS) → X | E |
| `HMS−` | Subtract H.MMSSss values | Y−X (HMS) → X | E |
| `→P` | Rectangular → polar magnitude; angle → Y | √(X²+Y²)→X | E |
| `→R` | Polar → rectangular X component; Y component → Y | r·cos θ → X | E |
| `%` | X percent of Y; Y preserved | Y×X/100 → X | E |
| `%CH` | Percent change from Y to X | (X−Y)/Y×100 → X | E |
| `IP` | Integer part of X | int(X) → X | E |
| `FP` | Fractional part of X | frac(X) → X | E |
| `ABS` | Absolute value of X | |X| → X | E |
| `SIGN` | Sign of X (−1, 0, +1) | sgn(X) → X | E |
| `MOD` | Y modulo X | Y mod X → X | E |
| `MAX` | Maximum of X, Y | max(X,Y) → X | E |
| `MIN` | Minimum of X, Y | min(X,Y) → X | E |
| `RND` | Round X to current display format | round(X) → X | E |
| `SQ` | x² (alias) | X² → X | E |
| `SQRT` | √x (alias) | √X → X | E |

---

## Miscellaneous

| Mnemonic | Description | Lift |
|----------|-------------|------|
| `BEEP` | Audible tone | N |
| `TONE n` | Tone n (0–9) | N |
| `SIZE nn` | Set number of data registers | N |
| `PROM` | Prompt without halting | N |
| `VW` | View register (display without alpha change) | N |
| `ADV` | Advance printer paper | N |

---

## HP-41CX: Time Module (Built-in)

| Mnemonic | Description |
|----------|-------------|
| `TIME` | Display current time → X |
| `DATE` | Display current date → X |
| `CLKMD` | Set 12/24-hour clock mode |
| `STIME` | Set time from X |
| `SDATE` | Set date from X |
| `ALMCAT` | Alarm catalog |
| `RCLALM` | Recall alarm |
| `STOALM` | Store alarm |
| `CLRALMS` | Clear all alarms |
| `XYZQ` | Extended CX-specific functions |

---

## Worked Examples

### `%CH` — Percent Change

**Question:** Sales were 200 units last quarter and 230 units this quarter. What is the percent change?

**Keystrokes (CLI):** `200` `ENTER↑` `230` `%`
**Keystrokes (GUI):** type `2 0 0`, press `Enter`, type `2 3 0`, press `%` on the physical keyboard

| Step | Keystroke | T | Z | Y | X | LAST X | Notes |
|------|-----------|---|---|---|---|--------|-------|
| 1 | `200`      | 0 | 0 | 0 | 200 | — | Enter base value |
| 2 | `ENTER↑`   | 0 | 0 | 200 | 200 | — | Lift X into Y |
| 3 | `230`      | 0 | 0 | 200 | 230 | — | Enter new value (overwrites X, lift disabled by `ENTER↑`) |
| 4 | `%CH`      | 0 | 0 | 200 | **15** | 230 | `((230−200)/200)×100 = 15`; Y preserved |
| 5 | `LASTX`    | 0 | 200 | 15 | **230** | 230 | Stack lifts (Lift = Enable); old Y/X move up; LASTX itself is unchanged |

**Result:** `15` — a 15 % increase. The base `Y = 200` is preserved on the stack (the defining HP-41 % family behaviour), so you can chain further calculations without re-entering the value. To bring the base back to X for direct inspection, use `x↔y`; to recover the original new value `230`, use `LASTX` — note that `LASTX` lifts the stack (Step 5 above), so the previous Y and X are pushed up into Z and Y rather than being discarded. The hidden `LASTX` register is itself unchanged by the `LASTX` op; only ops that modify X update it.

**Error case:** If `Y = 0`, `%CH` returns `Error: Divide by 0` and leaves the stack and `LASTX` unchanged.

**Recording in a program:** `%CH` records and plays back like any other binary op. In PRGM mode, the step displays as `%CH`; during `R/S` execution it consumes the same Y/X operands and produces the same result.

---

## See Also

- [Keyboard Layout](keyboard-layout.md)
- [Programming Guide](programming-guide.md)
- [HP-41C/CV Advanced Functions Handbook](https://www.hpmuseum.org/41advfun.htm) — complete flag/system register reference
