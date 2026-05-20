# Feature Landscape: HP-41 Math 1 Pac Emulation (v3.0)

**Domain:** Behavioral emulation of the HP-41C Math Pac I plug-in application module
**Researched:** 2026-05-16
**Scope:** v3.0 — Math Pac I (00041-90034) function library only; Stat 1 / Time / Advantage are deferred to v3.1+
**Confidence:** HIGH for the function inventory (primary source = the Owner's Manual itself); MEDIUM for implementation-pattern recommendations (algorithm choices are well-documented but emulator-specific decisions remain)

---

## Summary

The HP-41C **Math Pac I** (Hewlett-Packard part number **00041-90034**, manual dated 1979; Quick Reference Card 00041-90065) is a **user-code application module** — that is, every "function" in the pac is implemented as a standard HP-41 keystroke program stored in module ROM, not as Nut-CPU microcode. The pac contains **10 named top-level programs** with a combined ~55 user-visible entry points (XEQ-by-name labels). It is NOT the Advanced Matrix Pac (which adds element-wise `M+`, `MAT*`, `TRANS`, `IDN`, etc.) nor the Advantage Pac (which adds Romberg integration, root-solver, complex stack).

The prompt-question's assumed inventory (`MAT*`, `TRANS`, `IDN`, `RSUM`, `CSUM`, `V+`, `V-`, `VDOT`, `PROOT`, `CEXP`, `CLN`, `CSQRT`, `CY^X`) **largely misidentifies which pac is which**. Most of those names belong to the Advanced Matrix Pac or the HP-41CX/Advantage. The actual Math Pac I content is documented below from the primary source.

The pac's design philosophy is **prompt-driven workflows**: each program (e.g., `MATRIX`, `POLY`, `SOLVE`, `INTG`, `DIFEQ`, `FOUR`) is initialized by `XEQ "name"`, then prompts the user for inputs in ALPHA register text (`ORDER=?`, `DEGREE=?`, `FUNCTION NAME?`, `GUESS 1=?`, etc.). This is a **fundamentally different UX pattern** from the v2.2 ROM built-ins, which are mostly one-shot stack-acting ops. v3.0 must adopt the new pattern — multi-step modal flows driven by `print_buffer` / display strings, not stack-action.

A second structural observation: Math Pac I uses **specific numbered register blocks** (e.g., R00–R04 for polynomial coefficients in `POLY`; R00–R06 for `SOLVE`; R00–R07 for `INTG`; matrix elements placed at R(N+1) onward where N is the order in R14). Emulation must honor these register assignments because user programs can call Math Pac entry points as subroutines (Appendix B of the manual documents these subroutine entry points). Re-using register blocks differently would break programs ported from real HP-41 Math Pac users.

---

## Authoritative Source

**HP-41C Math Pac** Owner's Manual, Hewlett-Packard Company 1979, part number 00041-90034 — the entire feature catalog below derives from this manual ([archive at hpcalc.org](https://literature.hpcalc.org/items/776), [PDF](https://literature.hpcalc.org/community/hp41-pac-math-en.pdf), 17.8 MB).

Supplementary cross-references:
- HP-41C Math Pac I Quick Reference Card (00041-90065, February 1979) — confirms the QRC card list ([PDF](https://literature.hpcalc.org/community/hp41-pac-math-qrc-en.pdf)).
- Museum of HP Calculators HP-41 software library — [https://www.hpmuseum.org/software/soft41.htm](https://www.hpmuseum.org/software/soft41.htm)
- HP-41 Archive — [http://www.hp41.org/](http://www.hp41.org/)
- HPCalc community PDF index — [https://literature.hpcalc.org/](https://literature.hpcalc.org/)

---

## Pac Inventory (10 top-level programs)

The manual's Table of Contents — exact, in this order:

| # | Program (XEQ name) | Topic | SIZE | Manual page |
|---|--------------------|-------|------|-------------|
| 1 | `MATRIX` | Determinant + inverse + simultaneous equations (Gaussian elim. w/ partial pivoting) | varies (size-of-matrix-dependent) | 10 |
| 2 | `SOLVE` | Real root of f(x)=0 on an interval, modified secant iteration | 007 | 17 |
| 3 | `POLY` | Polynomial roots (degree 2–5) + evaluation | 023 | 21 |
| 4 | `INTG` | Numerical integration (discrete trapezoidal/Simpson OR explicit Simpson) | 008 | 25 |
| 5 | `DIFEQ` | 1st/2nd order ODE solver, 4th-order Runge-Kutta | 008 | 29 |
| 6 | `FOUR` | Fourier series (rect + polar coefficients) | 027 | 34 |
| 7 | `C+ C- C× C÷` + functions (complex) | Two-element complex stack, arith + 13 functions | 005 | 38 |
| 8 | `SINH/COSH/TANH/ASINH/ACOSH/ATANH` | Hyperbolic + inverse hyperbolic (6 ops) | 001 | 44 |
| 9 | Triangle Solutions (`SSS/ASA/SAA/SAS/SSA`) | 5 triangle-solver programs | 008 | 46 |
| 10 | `TRANS` | 2-D + 3-D coordinate transformations (translate/rotate) | 025 | 52 |

**Total user-visible XEQ names**: ~55 (see breakdown per category below). The user-facing surface area is larger than the program count because each top-level program exposes sub-routines reachable via `XEQ name` (the manual's Appendix B documents these).

---

## Complete Function List (per category, per primary source)

### Category 1 — Matrix Operations (`MATRIX`)

**One top-level program with 8 named entry points**, all driven from the single `MATRIX` initializer. Matrices are stored in column-major order starting at R15 (R14 holds the order N). Up to **14 × 14** is supported by the program (memory permitting: 6 × 6 fits the base HP-41C, larger needs memory modules). Algorithm: **Gaussian elimination with partial pivoting**.

| Function | Stack/register convention | Description |
|----------|--------------------------|-------------|
| `MATRIX` | Prompts: `ORDER=?` then `SET SIZE nnn` then `A1,1=?`...`AN,N=?` | Master entry — orchestrates input |
| `SIZE` | (calculator built-in, called from inside `MATRIX`) | Sets register count for matrix elements |
| `VMAT` | No stack input; sequentially displays `Ai,j=` values | View the stored matrix |
| `EDIT` | Prompts: `ROW↑COL=?` → `I` `ENTER↑` `J` → `Ai,j=?` → new value | Modify one matrix element |
| `DET` | No input; output: determinant in X-register; displays `DET=` | Compute determinant (also called as subroutine returns Det A in Y) |
| `INV` | After `DET`: sequentially output `Ci,j` column-major; displays `C1,1=`, `C2,1=`, ... | Compute inverse, output in column order |
| `SIMEQ` | Prompts `B1=?`...`BN=?` for RHS; output: `X1=` ... `XN=` | Solve system Ax=B |
| `VCOL` | No input; displays `B1`, `B2`...`BN` | View the column vector / solution |

**Out-of-band states** (manual-documented):
- `NO SOLUTION` display ⇔ singular matrix
- Flag 4: set during input/edit phase; cleared after pivoting
- Flag 5: set when `SIMEQ` has stored its column matrix; tells PVT to do a back-solve

**Inputs/outputs follow no per-op stack convention** — this is a **multi-step modal workflow**, not a stack-acting op. Implementation guidance: introduce a new `CalcState.modal_program` enum holding the active Math Pac program's step state, similar in spirit to v2.2's `PendingInput` but stateful across many key presses.

### Category 2 — Solution to f(x)=0 (`SOLVE`)

**Single program**, prompts for a user-defined function (any global label) and 0, 1, or 2 initial guesses. Algorithm: **modified secant iteration**. Uses R00–R06.

| Function | Stack/register convention | Description |
|----------|--------------------------|-------------|
| `SOLVE` | Prompts `FUNCTION NAME?` → user types ALPHA label → `GUESS 1=?` → `GUESS 2=?` → execute | Master entry; defaults: x₁=1, x₂=10 if both skipped |
| `SOL` (subroutine entry) | Guesses already in R00/R01 | Bypass prompting, for use as subroutine |

**Termination messages** (the manual documents three):
1. `NO ROOT FOUND`
2. `ROOT IS <value>` (single converged root)
3. `ROOT IS BETWEEN <v1> AND <v2>` (sign change found; bracket too tight to converge further)

**Behavior on no-root / multiple-root**:
- f(x₁)·f(x₂) < 0 and f continuous ⇒ always finds a root
- f(x₁)·f(x₂) > 0 ⇒ may fail (returns `NO ROOT FOUND`)
- Multiple roots in interval ⇒ returns one; user can narrow interval and retry

### Category 3 — Polynomial Roots/Evaluation (`POLY`)

**Single program** for degree 2–5 polynomials with real coefficients and leading coefficient 1. Coefficients stored in R00–R04. Uses R00–R22 total. Iterative method for cubic/quintic, synthetic division for reduction, closed-form quadratic.

| Function | Stack/register convention | Description |
|----------|--------------------------|-------------|
| `POLY` | Prompts `DEGREE=?` (n=2..5) → `a(n-1)=?` ... `a0=?` → `ROOTS?` | Master entry |
| `ROOTS` | (sub-entry: re-run roots after changing coefs in R00–R04) | Find roots; outputs as `ROOT=v` for real or `U=u`/`V=v`/`U=u`/`-V=-v` quartet for complex pair |
| (evaluation, no name shown) | After `ROOTS?` prompt: answer `N` → prompts `X=?` → enter x → `F<X>=v` | Evaluate polynomial at x |

**Output convention for complex root pairs**: `U=u`, `V=v`, `U=u`, `-V=-v` (i.e., u±iv printed as a 4-line block). This is unique to Math Pac I and must be reproduced exactly.

**Note**: there is **no `PROOT` function in Math Pac I** — that's the HP-41 Advantage Pac's name. Math Pac I uses `POLY` + `ROOTS`. The prompt's "PROOT" reference belongs to a later pac.

### Category 4 — Numerical Integration (`INTG`)

**Single program with two distinct modes**, sharing the `INTG` initializer. Uses R00–R07.

| Function | Stack/register convention | Description |
|----------|--------------------------|-------------|
| `INTG` | Master initializer — no input until A/B/C/D pressed | Entry |
| `A` (in discrete mode) | X = h (spacing) | Key in step h between equally-spaced x-values |
| `B` (in discrete mode) | X = f(xⱼ) | Key in jth function value; display shows j |
| `C` (in discrete mode) | No input | Compute area by trapezoidal rule (`TRAP[`) |
| `D` (in discrete mode) | No input | Compute area by Simpson's rule (`SIMP[`) — requires even n |
| `A` (in explicit mode) | Y = a (lower bound), X = b (upper bound) | Key in interval endpoints |
| `B` (in explicit mode) | X = n (number of subintervals, must be even) | Key in n, prompts `FUNCTION NAME?` |
| (function-name prompt) | ALPHA = label name | User-defined f(x) under any global label |

**Algorithm**: discrete mode uses trapezoidal OR Simpson; explicit mode uses Simpson's rule with fixed n subdivisions (NO adaptive refinement — this is HP-41 Math Pac, not Advantage's Romberg integration with convergence criterion). The user controls accuracy by choosing n.

**Error message**: `N NOT EVEN` if n is odd when `C`/`D`/`B` (explicit mode) is pressed.

### Category 5 — Differential Equations (`DIFEQ`)

**Single program**, 4th-order Runge-Kutta for 1st or 2nd order ODE. Uses R00–R07 (SIZE 008).

| Function | Stack/register convention | Description |
|----------|--------------------------|-------------|
| `DIFEQ` | Prompts `FUNCTION NAME?` → `ORDER=?` (1 or 2) → `STEP SIZE=?` → `X0=?` → `Y0=?` → (for 2nd order) `Y'0=?` → R/S for successive (x,y) pairs | Master entry |

### Category 6 — Fourier Series (`FOUR`)

**Single program**, computes Fourier coefficients from N samples per period. Up to 10 pairs (a_n, b_n). Uses R00–R26 (SIZE 027).

| Function | Stack/register convention | Description |
|----------|--------------------------|-------------|
| `FOUR` | Prompts `NO. SAMPLES=?` → `NO. FREQ=?` → `1ST COEFF=?` → `Y1=?`...`YN=?` → `RECT?` | Master entry; outputs `aₙ=v`/`bₙ=v` in rect or polar |
| `E` (USER-mode soft key) | X = t | Evaluate Fourier series at t after coefficients are computed |

### Category 7 — Complex Operations (13 functions + 4 arithmetic ops)

**The pac's largest sub-library**. Uses a two-complex-number stack — bottom register ζ (zeta, analog of X) and top register τ (tau, analog of T). A complex z = x + iy is keyed as `y ENTER↑ x` (real in X, imaginary in Y per HP-41 convention). Uses R00–R04.

#### Arithmetic (4 ops)

| Function | Input convention | Description |
|----------|------------------|-------------|
| `C+` | z1 in τ, z2 in ζ | Complex addition; result in ζ |
| `C-` | z1 in τ, z2 in ζ | Complex subtraction; result in ζ |
| `C×` | z1 in τ, z2 in ζ | Complex multiplication; result in ζ |
| `C÷` | z1 in τ, z2 in ζ | Complex division; result in ζ |

#### Functions (13 ops)

| Function | Input convention | Description |
|----------|------------------|-------------|
| `MAGZ` | z in ζ | \|z\|; returns a **real** number to X |
| `CINV` | z in ζ | 1/z |
| `Z↑N` | z in ζ, n in X | z^n |
| `Z↑1/N` | z in ζ, n in X | z^(1/n) (nth root) |
| `E↑Z` | z in ζ | exp(z) |
| `LNZ` | z in ζ | ln(z) |
| `A↑Z` | z in ζ, a (real) in X | a^z |
| `LOGZ` | z in ζ, a (real) in X | log_a(z) |
| `Z↑W` | w in τ, z in ζ | z^w (general complex power) |
| `Z↑1/W` | w in τ, z in ζ | z^(1/w) |
| `SINZ` | z in ζ | sin(z) |
| `COSZ` | z in ζ | cos(z) |
| `TANZ` | z in ζ | tan(z) |

**Note**: there is **no `CABS` / `CARG` / `CCHS` / `CCONJ` / `CPOLAR` / `CRECT` / `CSQRT`** in Math Pac I — those names appear in Advantage Pac (`COMPLEX` toggle on HP-41CX). Math Pac I provides `MAGZ` for modulus but **no separate argument function** (the user would compute argument via the underlying real-X-and-Y representation manually). The prompt's assumed naming is partially anachronistic; the actual names are above.

### Category 8 — Hyperbolics (6 ops)

| Function | Input | Description |
|----------|-------|-------------|
| `SINH` | x in X | Hyperbolic sine |
| `COSH` | x in X | Hyperbolic cosine |
| `TANH` | x in X | Hyperbolic tangent |
| `ASINH` | x in X | Inverse hyperbolic sine |
| `ACOSH` | x in X | Inverse hyperbolic cosine (x ≥ 1) |
| `ATANH` | x in X | Inverse hyperbolic tangent (\|x\| < 1) |

These are **simple one-shot stack-acting ops** — the only Math Pac category that fits the v2.2 op style. Easy implementation target.

### Category 9 — Triangle Solutions (5 ops)

| Function | Inputs (after prompts) | Description |
|----------|------------------------|-------------|
| `SSS` | three sides | Side-Side-Side; outputs angles + area |
| `ASA` | two angles + included side | Angle-Side-Angle |
| `SAA` | two angles + adjacent side | Side-Angle-Angle |
| `SAS` | two sides + included angle | Side-Angle-Side |
| `SSA` | two sides + adjacent angle | Side-Side-Angle (ambiguous case) |

Output order follows clockwise traversal of the inputs. Uses prompt flow.

### Category 10 — Coordinate Transformations (`TRANS`)

**Single program with 5 USER-mode soft-key entries** (`A`/`B`/`C`/`D`/`E`).

| Function | Input (after prompts) | Description |
|----------|----------------------|-------------|
| `TRANS` then `A` (2-D init) | x₀, y₀ (origin), θ (rotation angle) | Set up 2-D transform |
| `C` (2-D forward) | x, y | Transform to translated-rotated system |
| `E` (2-D inverse) | x', y' | Inverse transform |
| `A` (3-D init) | x₀, y₀, z₀ | Set 3-D origin |
| `B` (3-D rotation) | a, b, c (rotation vector), θ | Set 3-D rotation about an arbitrary axis |
| `C` (3-D forward) | x, y, z | Transform |
| `E` (3-D inverse) | x', y', z' | Inverse |

Uses R00–R24 (SIZE 025).

---

## Cross-Reference: Prompt-Assumed Functions That Don't Exist in Math Pac I

The downstream-consumer prompt assumed several function names that come from **other pacs** or **the HP-41CX firmware**. For roadmap accuracy:

| Prompt-assumed name | Actual home | Math Pac I has it? |
|---------------------|-------------|--------------------|
| `M+`, `M-`, `MAT*` (element-wise matrix ops) | Advanced Matrix Pac (separate module) | **NO** |
| `TRANS` (matrix transpose) | Advanced Matrix Pac | **NO** (Math Pac's `TRANS` is COORDINATE transform — homonym conflict!) |
| `IDN` (identity matrix) | Advanced Matrix Pac | **NO** |
| `RSUM`, `CSUM`, `MMOVE`, `MAT?`, `DIM?` | Advanced Matrix Pac | **NO** |
| `V+`, `V-`, `VDOT`, `VLEN`, `VANG` | Advanced Matrix Pac (vectors as 1×N matrices) | **NO** |
| `PROOT` | Advantage Pac (HP-41CX) | **NO** (Math Pac I uses `POLY`/`ROOTS`) |
| `CABS`, `CARG`, `CCHS`, `CCONJ`, `CPOLAR`, `CRECT`, `CSQRT` | Advantage Pac (`COMPLEX` mode on 41CX) | **NO** (Math Pac I provides only `MAGZ`) |
| `GAMMA`, `ERF`, `BESSEL` | Math/Stat Pac extensions or Free42 | **NO** |
| Probability distributions, `%CH`, `%T` | Stat Pac | **NO** (those are v3.1+ scope) |

**Implication for roadmap**: v3.0 (Math Pac I only) is a **smaller, more workflow-focused** scope than the prompt suggested. The "differentiator vs anti-feature" classification below reflects the *actual* Math Pac I contents, not the prompt's superset.

---

## Table Stakes (Math Pac I)

Functions users expect when they pay for the Math Pac. Missing any of these and the v3.0 milestone **fails the "feature-complete Math Pac" promise**.

| Function(s) | Why expected | Complexity | hp41-core dependency |
|-------------|--------------|------------|---------------------|
| **Hyperbolics**: `SINH`, `COSH`, `TANH`, `ASINH`, `ACOSH`, `ATANH` | Simplest, most-used Math Pac entries; pure unary ops fit v2.2 op style | **Small** | New `Op::Sinh/Cosh/Tanh/Asinh/Acosh/Atanh` variants; reuse `HpNum`/`rust_decimal` math |
| **Complex arithmetic**: `C+`, `C-`, `C×`, `C÷` | Foundation for the whole complex-functions sub-library; users need basic z₁+z₂ | **Medium** | New 2-complex-number stack abstraction (ζ and τ) layered over real X/Y/Z/T; new `ComplexStack` field on `CalcState` with `#[serde(default)]` |
| **Complex functions**: `MAGZ`, `CINV`, `E↑Z`, `LNZ`, `SINZ`, `COSZ`, `TANZ`, `Z↑N`, `Z↑1/N` | The pac's marquee functionality; users buy Math Pac specifically for complex math | **Medium** | Depends on complex-stack scaffolding above; each op is unary on ζ |
| **Polynomial roots**: `POLY` + `ROOTS` (degree 2–5) | One of the pac's headline programs; quadratic formula + cubic/quartic/quintic root-finding | **Large** | New modal program state machine; complex root pair output formatting; depends on existing register block model |
| **Polynomial evaluation** (after `POLY`) | Cheap given roots scaffolding; Horner-like evaluation | **Small** | Same state machine as above |
| **Numerical integration explicit mode**: `INTG` with user-defined f(x), Simpson's rule | The pac's marquee numerical analysis offering | **Large** | New XEQ-by-name dispatch into user programs from inside a Math Pac modal; needs `program_call_in_modal` infrastructure |
| **Numerical integration discrete mode**: trapezoidal + Simpson with even-n check | Simpler than explicit mode, no user-program callback | **Medium** | Modal state machine; cumulative samples in R00–R07 |
| **f(x)=0 solver**: `SOLVE` with user-defined f(x), modified-secant iteration | The pac's other marquee numerical offering | **Large** | XEQ-by-name dispatch from inside modal (same infrastructure as `INTG` explicit); convergence-bracket logic; three termination messages |
| **Matrix workflow**: `MATRIX` initializer + element input loop + `VMAT` + `EDIT` | Without keying-in a matrix, nothing else in the matrix sub-pac works | **Large** | Multi-step prompt flow with register-block assignment (R15 onward); `VMAT` driven by `print_buffer`; `EDIT` reuses the prompt machinery |
| **Matrix `DET` + `INV`**: Gaussian elimination with partial pivoting | Why anyone uses the matrix workflow at all | **Large** | Algorithm is well-documented; numerical accuracy concerns; output in column order to `print_buffer` |
| **Matrix `SIMEQ` + `VCOL`**: solve Ax=B | Natural extension of `INV`; same Gaussian engine | **Medium** | Reuse pivoted matrix from `INV` step (flags 4/5 protocol per manual) |

---

## Differentiators (Math Pac I)

Functions that increase fidelity but aren't strictly required for the "feature-complete" claim. Add after table-stakes ship.

| Function(s) | Value proposition | Complexity | Notes |
|-------------|-------------------|------------|-------|
| **Complex `A↑Z`, `LOGZ`, `Z↑W`, `Z↑1/W`** | Completes the complex-function set; differentiates "real Math Pac emulation" from "just the easy half" | **Medium** | Pure layering on complex stack — small per-op effort but adds up to 4 entries |
| **Triangle Solutions**: `SSS`, `ASA`, `SAA`, `SAS`, `SSA` | Distinctive Math Pac functionality; surveying / drafting users will look for these | **Medium** | 5 separate prompt flows; each has trig + Law of Cosines/Sines |
| **`DIFEQ` 1st-order Runge-Kutta** | Marquee numerical-analysis feature; requires XEQ-by-name dispatch from modal | **Large** | Algorithm is straightforward; the complication is the modal/dispatch interaction (same infrastructure as `SOLVE`/`INTG`) |
| **`DIFEQ` 2nd-order Runge-Kutta** | Natural extension of 1st-order; adds an extra prompt for y'₀ | **Small** (if 1st-order exists) | — |
| **`FOUR` Fourier coefficients** in rect + polar | Distinctive offering; well-defined algorithm (DFT formula); large register usage (R00–R26) | **Large** | Output formatting (aₙ/bₙ pairs) and rect↔polar toggle |
| **`FOUR` series evaluation at t** (USER-mode `E` key) | Completes the Fourier workflow; small additional surface | **Small** | Once coefficients exist, evaluation is cheap |
| **`TRANS` 2-D coordinate transformation** | Useful in surveying, robotics, graphics | **Medium** | 4 prompt-key entries (`A`/`C`/`E`) for init/forward/inverse |
| **`TRANS` 3-D coordinate transformation** | Less common than 2-D; needs rotation-vector + angle (Rodrigues' rotation formula) | **Large** | More complex math (axis-angle rotation), more prompt branches |
| **Matrix workflow edge cases**: `NO SOLUTION` (singular), flag 4/5 state, partial pivoting display | Fidelity to the manual; matters for users porting real programs | **Medium** | Easy to skip but breaks compatibility |
| **`COPY`-compatibility**: appearing in `CATALOG 2` listing | Real Math Pac programs are visible in CAT 2; users may try to `COPY` them | **Small** (display only, no actual COPY) | v2.2's catalog command already exists; just add entries |

---

## Anti-Features (NOT in Math Pac I — defer or exclude)

Features to **explicitly not build in v3.0**. The downstream-consumer prompt mixed several of these in by mistake — flagging them so the roadmap doesn't accidentally try to scope them.

| Anti-feature | Why avoid in v3.0 | Where it actually belongs |
|--------------|-------------------|---------------------------|
| `M+`, `M-`, `MAT*`, `INV` (element-level matrix ops) | **Wrong pac** — Advanced Matrix Pac, not Math Pac I | v3.2+ (separate pac scope) |
| `TRANS` (matrix transpose), `IDN`, `RSUM`, `CSUM`, `MMOVE` | **Wrong pac** — Advanced Matrix Pac | v3.2+ |
| `V+`, `V-`, `VDOT`, `VLEN`, `VANG` | **Wrong pac** — Advanced Matrix Pac (vectors as 1×N matrices) | v3.2+ |
| `PROOT` | **Wrong pac** — Advantage Pac (HP-41CX); Math Pac I uses `POLY`/`ROOTS` | v3.3+ (Advantage scope) |
| `CABS`/`CARG`/`CCHS`/`CCONJ`/`CPOLAR`/`CRECT`/`CSQRT`/`CEXP`/`CLN`/`CY^X` | **Wrong naming** — Advantage Pac introduces these; Math Pac I provides `MAGZ` + the Z↑/E↑Z/LNZ/SINZ family instead | v3.3+ (Advantage), or build `MAGZ` etc. and label them per Math Pac |
| `GAMMA`, `ERF`, `BESSEL` | Not in Math Pac I (or any standard HP-41 pac as named ops) | Out of scope permanently |
| Probability distributions, `%CH`, `%T` | Stat Pac | v3.1+ |
| Romberg integration (adaptive convergence) | Not in Math Pac I — it uses Simpson's with user-chosen n | v3.3 (Advantage's `∫f(x)` does Romberg) |
| Cycle-accurate execution of Math Pac user-code listings | Math Pac is user code, but emulating it as user code requires ROM-image redistribution (legal block per PROJECT.md) | Permanent — we implement BEHAVIORAL emulation only |
| Distributing the actual Math Pac `.rom` / `.mod` binary | Legal: HP-copyrighted | Permanent exclusion |
| `COPY` command for downloading Math Pac programs to user memory | Implies the ROM is loadable into memory — not behavioral | Permanent exclusion |
| Synthetic-programming access to Math Pac internals | Pac is opaque in behavioral emulation; M/N/O registers stay untouched | N/A |
| Magnetic-card I/O of Math Pac programs (`WPRGM`/`RDPRGM`) | Math Pac is a ROM module, not a card — can't be loaded/saved | N/A (existing card reader handles user programs only) |

---

## Feature Dependencies

```
v2.2 existing infrastructure (UNCHANGED — preconditions for v3.0)
  ├─> Op enum dispatch (ops/mod.rs) — needs ~55+ new Op variants
  ├─> CalcState + serde(default) for forward-compat
  ├─> print_buffer — Math Pac uses ALPHA-style display prompts; print_buffer is the right channel
  ├─> XEQ-by-name (builtin_card_op, run_program) — must extend to dispatch into Math Pac entry names
  ├─> 56 user flags + system flags — flag 4/5 already exist as concepts; Math Pac uses them for its own state
  └─> Indirect addressing (FN-IND family) — Math Pac element access uses indirect addressing internally

NEW v3.0 infrastructure (must land before any Math Pac function)
  ├─> XROM module framework (XROM number + function number; mapping to behavioral ops)
  │     └─> Catalog 2 visibility (existing v2.2 catalog extended)
  ├─> Modal program state machine
  │     ├─> Multi-step prompt flow ("ORDER=?", "A1,1=?", "FUNCTION NAME?", etc.)
  │     ├─> Register-block protocols (R14 for order, R15+ for matrix elements, R00–R04 for poly coefs, etc.)
  │     └─> Termination message handling ("NO SOLUTION", "NO ROOT FOUND", "ROOT IS BETWEEN ...")
  └─> User-function callback (for SOLVE / INTG / DIFEQ / FOUR with E key)
        └─> Calls user-defined global label, expects X on entry, returns f(x) in X

Math Pac functions (build order from leaf to root)
  Hyperbolics (6 ops, leaf)
    └─> No dependencies; one-shot unary; build first as proof-of-pattern
  Complex stack (ζ/τ + 4 arith + 13 functions)
    └─> Depends on: new ComplexStack struct on CalcState
  TRIG sub-prompt flows (5 triangle solvers)
    └─> Depends on: modal state machine
  POLY/ROOTS (degree 2–5 with complex root output)
    └─> Depends on: modal state machine + complex output formatting
  MATRIX/DET/INV/SIMEQ/VMAT/EDIT/VCOL (full workflow)
    └─> Depends on: modal state machine + register-block protocol + flag 4/5 protocol
  INTG discrete mode
    └─> Depends on: modal state machine
  INTG explicit mode + SOLVE
    └─> Depends on: user-function callback (the big infrastructure piece)
  DIFEQ
    └─> Depends on: user-function callback + 4th-order Runge-Kutta
  FOUR
    └─> Depends on: modal state machine + USER-mode E-key wiring
  TRANS 2-D + 3-D
    └─> Depends on: modal state machine + USER-mode key wiring
```

---

## Implementation Order Recommendation (Highest User-Value First)

The Math Pac unlocks value in **roughly this order** for the typical buyer:

1. **Phase A — Foundation**: XROM module framework + modal state machine + user-function callback. **No user-visible features yet** but unblocks everything else. (Cross-cutting infra; ~1 phase of work.)

2. **Phase B — Hyperbolics (6 ops)**. Smallest table-stakes win; one-shot ops; demonstrates the v3.0 pattern works end-to-end through CLI + GUI; immediate user value (anyone who wants `SINH`). (1 phase.)

3. **Phase C — Complex arithmetic + the easy 9 functions** (`C+`, `C-`, `C×`, `C÷`, `MAGZ`, `CINV`, `E↑Z`, `LNZ`, `SINZ`, `COSZ`, `TANZ`, `Z↑N`, `Z↑1/N`). Marquee feature; the largest single "wow" delivery. (1 phase.)

4. **Phase D — Polynomial roots (`POLY` + `ROOTS` for degree 2–5) + evaluation**. The second marquee feature; introduces complex root output formatting and the modal state machine in a self-contained context. (1 phase.)

5. **Phase E — Matrix workflow (full pipeline `MATRIX` → `DET` → `INV` → `SIMEQ` → `VMAT` → `EDIT` → `VCOL`)**. The pac's most complex single feature; gates the "feature-complete" claim. (1–2 phases.)

6. **Phase F — Numerical integration `INTG` (both discrete and explicit modes)**. First feature requiring user-function callback; high pedagogical value. (1 phase.)

7. **Phase G — `SOLVE` (modified secant root finder)**. Reuses INTG's callback infrastructure; second numerical-analysis flagship. (1 phase, smaller because infra exists.)

8. **Phase H — `DIFEQ` (Runge-Kutta) + remaining complex functions (`A↑Z`, `LOGZ`, `Z↑W`, `Z↑1/W`)**. (1 phase.)

9. **Phase I — Differentiators: Triangle Solutions (5 programs), `FOUR`, `TRANS` 2-D + 3-D**. Lower per-feature value but completes the pac. (1–2 phases.)

10. **Phase J — Quality hardening**: numerical accuracy suite extended with Math Pac cases (matrix `DET`/`INV` correctness vs. reference linalg, Simpson convergence vs. analytic integrals, complex-function identity tests, SOLVE root-finding on known polynomials). CLI + GUI integration (modal flows, key bindings, `xrom_help.rs` analog of `help_data.rs`). Coverage gate stays ≥95%.

**Rationale for the order:**

- **Foundation first** (A): nothing ships without the XROM/modal/callback scaffolding.
- **Hyperbolics second** (B): cheapest, validates the pattern, immediate user win.
- **Complex math third** (C): biggest "marquee" win; the pac's reputation rests on it.
- **POLY before MATRIX** (D before E): POLY is self-contained, MATRIX is the most complex single program; learning curve on modal state machine starts gentler.
- **MATRIX before INTG/SOLVE** (E before F/G): MATRIX is pure-data modal (no user-function callback); INTG/SOLVE add the callback layer on top of a known modal pattern.
- **Triangles + FOUR + TRANS last** (I): differentiators, not table stakes; can ship without and still claim "feature-complete Math Pac core".

---

## MVP Recommendation (subset for first usable v3.0 cut)

If shipping a minimal v3.0 to validate the architecture before completing everything:

1. **MVP-1**: XROM framework + Hyperbolics (6 ops) + Complex arithmetic + 5 most-used complex functions (`MAGZ`, `CINV`, `E↑Z`, `LNZ`, `SINZ`).
2. **MVP-2**: Add POLY/ROOTS for quadratic only (degree 2).
3. **MVP-3**: Add the matrix workflow for size N ≤ 4 (smaller working set; same algorithm).

This gets a usable Math Pac v3.0-alpha out the door in ~3 phases; the remaining content lands in iterative phases without changing the architecture.

---

## Complexity Estimates Per Function

| Function | Size | Risk | Notes |
|----------|------|------|-------|
| Hyperbolics (6 ops total) | Small | Low | Each is `f(X) → X` via `rust_decimal`; ~3 lines per op |
| Complex stack scaffolding | Medium | Medium | New `ComplexStack { zeta, tau }`; serde-default; touches stack-lift semantics |
| `C+`/`C-`/`C×`/`C÷` (4 ops) | Small | Low | Trivial once scaffolding exists |
| `MAGZ`, `CINV`, `E↑Z`, `LNZ` | Small | Low | Standard complex identities |
| `SINZ`/`COSZ`/`TANZ` | Medium | Medium | Numerical care for large imaginary parts (overflow in `exp(±iy)`) |
| `Z↑N`/`Z↑1/N`/`A↑Z`/`LOGZ`/`Z↑W`/`Z↑1/W` | Medium | Medium | Power/log identity chains; branch cut decisions documented in manual |
| `POLY` setup + `ROOTS` (deg 2) | Medium | Low | Quadratic formula |
| `ROOTS` (deg 3, 5 — iterative) | Large | Medium | Manual specifies iterative + synthetic division — algorithm choice matters for matching the manual's example output (root order, sign convention) |
| `ROOTS` (deg 4 — Ferrari) | Large | Medium | Cubic resolvent + two quadratics |
| `MATRIX` initializer + input loop | Medium | Medium | Modal state for N²+ inputs; register block at R15+ |
| `VMAT`/`EDIT`/`VCOL` | Small each | Low | Display loops over the register block |
| `DET` + `INV` (Gaussian elim. with partial pivoting) | Large | Medium-High | Numerical stability concerns; partial pivoting must match manual's pivot-choice algorithm for byte-exact reproducibility of the manual's worked examples |
| `SIMEQ` | Medium | Low | Reuses pivoted matrix from `INV` step |
| `SOLVE` (modified secant) | Large | Medium | Iteration limit; sign-change bracket detection; three distinct termination paths |
| `INTG` discrete (trapezoidal + Simpson) | Medium | Low | Fixed formulas; even-n check |
| `INTG` explicit (Simpson with user f(x)) | Large | High | Requires user-function callback infrastructure (first feature to need it); recursion depth = number of Simpson sub-intervals |
| `DIFEQ` (RK4, 1st + 2nd order) | Large | Medium | Standard RK4; second-order via system reduction |
| `FOUR` (DFT + rect/polar + USER-E eval) | Large | Medium | Coefficient calculation + display loop + USER-mode key wiring |
| Triangle Solutions (5 programs) | Medium each | Low | Law of Sines / Cosines; SSA ambiguous-case warning |
| `TRANS` 2-D | Medium | Low | Rotation matrix + translation |
| `TRANS` 3-D | Large | Medium | Rodrigues' rotation formula for axis-angle |

**Highest risk items** (cite for roadmap research flags):

1. **Numerical reproducibility of `DET`/`INV`** — partial pivoting order must match the manual's worked examples (the Carnahan-Luther-Wilkes reference algorithm). Floating-point ordering effects make this fragile. **Recommend a research phase BEFORE Phase E** to nail down the exact pivoting algorithm.
2. **User-function callback inside modals** (Phase F precondition) — the calculator must dispatch into user code from inside a Math Pac program, then return cleanly. Today's `run_program` is the main loop's responsibility; making it reentrant for INTG/SOLVE/DIFEQ is non-trivial. **Recommend a research phase BEFORE Phase F** to design the call-stack model.
3. **Complex stack interaction with real stack** — when a Math Pac complex op fires, what happens to T/Z/Y/X? The manual says "previous contents of τ are lost" but doesn't specify the real-stack invariant. **Recommend hardware-verification or Free42 cross-check.**
4. **`POLY` complex root pair display order** — the manual shows `U=u`/`V=v`/`U=u`/`-V=-v` — a four-line per-pair format. Exact matching of this format is a fidelity requirement.

---

## Sources

- HP-41C Math Pac Owner's Manual (00041-90034, 1979) — [PDF at hpcalc.org](https://literature.hpcalc.org/community/hp41-pac-math-en.pdf), [hpcalc index](https://literature.hpcalc.org/items/776) — **primary source**, HIGH confidence
- HP-41C Math Pac I Quick Reference Card (00041-90065, February 1979) — [PDF at hpcalc.org](https://literature.hpcalc.org/community/hp41-pac-math-qrc-en.pdf) — confirms function list, HIGH confidence
- Museum of HP Calculators HP-41 software library — [hpmuseum.org/software/soft41.htm](https://www.hpmuseum.org/software/soft41.htm) — MEDIUM confidence (community-curated)
- HP-41C XROM Numbers — [hpmuseum.org/software/xroms.htm](https://www.hpmuseum.org/software/xroms.htm) — XROM numbering convention (403 on first fetch; mirror exists)
- HP-41 Archive — [hp41.org](http://www.hp41.org/) — MEDIUM confidence
- HP-41 Matrix Operations community page — [hpmuseum.org/software/41/41matrix.htm](https://www.hpmuseum.org/software/41/41matrix.htm) — confirms 14×14 limit and Gaussian-elimination algorithm
- HP-41 community programs (hp41programs.yolasite.com) — for verifying behavior on edge cases — MEDIUM confidence
- Carnahan, Luther & Wilkes, _Applied Numerical Methods_, John Wiley & Sons 1969 — cited in the manual as the matrix algorithm reference
- Forsythe, Malcolm & Moler, _Computer Methods for Mathematical Computations_, 1972 — manual's secondary matrix reference
- Free42 source (Thomas Okken) — [thomasokken.com/free42/](https://thomasokken.com/free42/) — useful only for the HP-42S equivalents (Advantage-style functions); Math Pac I itself is **not** part of Free42's emulation surface

---

## Open Questions for Roadmap

1. **Should we adopt the Math Pac's XROM-style numbering?** The pac is XROM 7 on real hardware (`XROM 07,nn`). For programmatic dispatch (`Op::Xrom(7, 5)` → MAGZ), this is the natural model. Decision needed early in Phase A.
2. **Where does the complex stack live?** Options: (a) on `CalcState` as a separate `ComplexStack` field; (b) overlaid on T/Z/Y/X (pairs ζ=Y+iX, τ=T+iZ — matches the manual's "two-element complex stack"); (c) in dedicated registers (R02/R03 for ζ, R04/R05 for τ — matches the manual's actual R00–R04 register block). Decision: option (b) or (c) for fidelity; (a) for cleanliness. **Recommend research phase to pick.**
3. **Do Math Pac programs persist across save/restore?** Real Math Pac is ROM, always loaded. In emulation: probably yes, conditionally on "module installed" state. Tied to XROM framework design.
4. **GUI presentation**: the manual references USER-mode soft-key overlays for the pac. Do we add a `MATH` toggle to the GUI that re-labels keys when active? Likely yes for differentiator value; defer to Phase I/J integration phase.
5. **Documentation deliverable**: equivalent of `docs/hp41cv-functions.json` for the Math Pac (`docs/hp41-math1-functions.json`) — should be a separate file to keep concerns separate; same regen tooling (`scripts/docs-matrix`).

---

*End of FEATURES.md — 1 author, 1 milestone, 1 application module. Math Pac I scope is locked.*
