# Verifying Math Pac I (HP 00041-90034)

This procedure walks an operator through the Math Pac I emulation landed in
Phase 28 (v3.0). It covers all 11 functional groups (hyperbolics, complex
stack arithmetic, complex transcendentals, polynomial root-finder, matrix
workflow, integration, solver, ODE solver, Fourier, triangle solvers,
coordinate transforms) on both `hp41-cli` and `hp41-gui`, and confirms
behavioral identity across UIs.

It exercises every directly-dispatchable Op variant (Tier 1), all
modal-state-preset paths that work today (Tier 2), one user-callback program
walk-through (Tier 3), and explicitly enumerates the modal-workflow flows that
are deferred to Phase 29 (CLI) and Phase 31 (GUI).

## TL;DR

| Group | Op count | Today's reach | Section |
|-------|----------|---------------|---------|
| Hyperbolics | 6 | All directly testable | §2 |
| Complex stack arithmetic | 5 | All directly testable | §3 |
| Complex transcendentals | 12 | All directly testable | §4 |
| Triangle solvers | 5 | All directly testable | §5 |
| POLY (ROOTS sub-entry) | 1 of 2 | ROOTS directly; POLY modal deferred | §6 |
| MATRIX kernels (DET/INV/SIMEQ) | 3 of 8 | Programmatic preset only | §7 |
| INTG / SOLVE / DIFEQ | 4 of 4 | Program-wrapper required | §8 |
| FOUR | 1 of 1 | Deferred to Phase 29/31 | §9 |
| TRANS / T3D | 0 of 2 | Deferred to Phase 29/31 | §9 |

A complete Tier 1 walk-through (sections 2 through 5) takes ≈ 20 minutes.
Tier 2 (sections 6 through 8) adds another ≈ 30 minutes.

## 1. Preparation

```bash
$ rm -f ~/.hp41/autosave.json
$ hp41             # or: just gui-dev
```

Operator: `Ctrl+G` (CLREG) — fresh state.

Two CLI conveniences used throughout:

- `f` = `f-prefix one-shot` (orange shifted key), CLI key `f`. After the next
  op fires, `f-prefix` is cleared automatically. Esc cancels.
- `XEQ` by name = CLI key `X`, then `ALPHA <name> ALPHA ENTER`.

Default angle mode is `DEG`. For sections that depend on the angle mode, the
section header notes the expected mode and the keys to switch into it.

Stack convention throughout: keystroke sequence like `2 ENTER 3 ENTER 4` means
"X=4, Y=3, Z=2, T=0 (lifted)" after the entry — standard RPN.

The "Display" column always means the X-register content after the op
completes. "FIX 4" display mode is assumed unless noted otherwise.

## 2. Hyperbolics

Six ops, all unary stack-acting, all angle-mode-INDEPENDENT (pure
mathematical functions of a real number). LiftEffect: Disable on entry
(consumes the entry buffer if active), Neutral on stack lift.

XEQ names: `SINH`, `COSH`, `TANH`, `ASINH`, `ACOSH`, `ATANH`.

| # | Setup → XEQ | Display (X) | Notes |
|---|-------------|-------------|-------|
| 2.1 | `0 → XEQ SINH ENTER` | `0.0000` | sinh(0) = 0 |
| 2.2 | `1 → XEQ SINH ENTER` | `1.1752` | sinh(1) ≈ 1.17520119 |
| 2.3 | `1 → XEQ COSH ENTER` | `1.5431` | cosh(1) ≈ 1.54308063 |
| 2.4 | `0 → XEQ COSH ENTER` | `1.0000` | cosh(0) = 1 |
| 2.5 | `1 → XEQ TANH ENTER` | `0.7616` | tanh(1) ≈ 0.76159416 |
| 2.6 | `1 → XEQ ASINH ENTER` | `0.8814` | asinh(1) ≈ 0.88137359 |
| 2.7 | `2 → XEQ ACOSH ENTER` | `1.3170` | acosh(2) ≈ 1.31695790 |
| 2.8 | `0.5 → XEQ ATANH ENTER` | `0.5493` | atanh(0.5) ≈ 0.54930614 |

### Hyperbolic domain guards

Two ops have explicit domain restrictions:

| # | Setup → XEQ | Expected | Notes |
|---|-------------|----------|-------|
| 2.9 | `0.5 → XEQ ACOSH ENTER` | `data error` | acosh undefined for x < 1 |
| 2.10 | `1 → XEQ ATANH ENTER` | `data error` | atanh undefined for &#124;x&#124; ≥ 1 |
| 2.11 | `1 CHS → XEQ ATANH ENTER` | `data error` | -1 boundary also rejected |

After a domain error, the stack is unchanged and the next keypress restores
the normal display.

## 3. Complex Stack Arithmetic

Five ops: `C+`, `C-`, `C×` (also `C*`), `C÷` (also `C/`), `REAL`.

The complex stack is an **overlay** on the standard four-level HP stack:
ζ = X + iY (re/im pair), τ = Z + iT (re/im pair). No new storage — the
same X/Y/Z/T registers are reinterpreted as a complex pair when
`complex_mode = true`.

Stack ordering for binary complex ops: ζ is the "X-equivalent" operand (top),
τ is the "Y-equivalent" operand (below). Conjugate of `C-` etc. follows
the same lift-and-replicate rules as real arithmetic.

| # | Setup → XEQ | After (X, Y, Z, T) | Notes |
|---|-------------|--------------------|-------|
| 3.1 | `1 ENTER 2 ENTER 3 ENTER 4 → XEQ C+ ENTER` | `(4, 6, 1, 1)` | (3+4i) + (1+2i) = 4+6i; T-replicate |
| 3.2 | `1 ENTER 2 ENTER 3 ENTER 4 → XEQ C- ENTER` | `(2, 2, 1, 1)` | (3+4i) - (1+2i) = 2+2i |
| 3.3 | `1 ENTER 2 ENTER 3 ENTER 4 → XEQ C× ENTER` | `(-5, 10, 1, 1)` | (3+4i)·(1+2i) = -5+10i |
| 3.4 | `1 ENTER 2 ENTER 3 ENTER 4 → XEQ C÷ ENTER` | `(2.2, -0.4, 1, 1)` | (3+4i)/(1+2i) = 2.2 - 0.4i |
| 3.5 | `… any complex result above … → XEQ REAL ENTER` | unchanged | clears `complex_mode`; stack untouched (Neutral) |

### Zero-divisor guard (Pitfall 6 fidelity gate)

`C÷` must fail BEFORE the `complex_mode` flag flips — the stack and
complex_mode flag both remain unchanged on error.

| # | Setup → XEQ | Expected | Verify |
|---|-------------|----------|--------|
| 3.6 | `XEQ REAL ENTER` (clear complex_mode) → `1 ENTER 2 ENTER 0 ENTER 0 → XEQ C÷ ENTER` | `data error` | After: `C` annunciator OFF (complex_mode preserved), X=0, Y=0, Z=2, T=1 |

## 4. Complex Transcendentals

Twelve ops on top of the complex-stack overlay from §3. Five unary, two
integer-exponent, four near-unary (trig via hyperbolic-identity
decomposition), and two binary.

Verify each result on a known reference value. Use a programmer calculator or
Wolfram Alpha for cross-checking unfamiliar values.

| # | Setup → XEQ | After X (re), Y (im) | Reference |
|---|-------------|----------------------|-----------|
| 4.1 | `3 ENTER 4 → XEQ MAGZ ENTER` | `(5, 0)` | &#124;3+4i&#124; = 5 |
| 4.2 | `3 ENTER 4 → XEQ CINV ENTER` | `(0.1200, -0.1600)` | 1/(3+4i) = 0.12 − 0.16i |
| 4.3 | `1 ENTER 1 ENTER 2 → XEQ Z↑N ENTER` (or `Z^N`) | `(0, 2)` | (1+i)² = 2i |
| 4.4 | `0 ENTER 1 ENTER 2 → XEQ Z↑1/N ENTER` | `(0.7071, 0.7071)` | √(i) = (1+i)/√2 |
| 4.5 | `0 ENTER 1 → XEQ E↑Z ENTER` | `(0.5403, 0.8415)` | e^i = cos(1)+i·sin(1) |
| 4.6 | `1 ENTER 1 → XEQ LNZ ENTER` | `(0.3466, 0.7854)` | ln(1+i) = ½ln2 + iπ/4 |
| 4.7 | `0 ENTER 1 → XEQ SINZ ENTER` | `(0, 1.1752)` | sin(i) = i·sinh(1) |
| 4.8 | `0 ENTER 1 → XEQ COSZ ENTER` | `(1.5431, 0)` | cos(i) = cosh(1) |
| 4.9 | `0 ENTER 1 → XEQ TANZ ENTER` | `(0, 0.7616)` | tan(i) = i·tanh(1) |
| 4.10 | `2 ENTER 0 ENTER 0 ENTER 1 → XEQ A↑Z ENTER` (or `A^Z`) | `(0.7692, 0.6390)` | 2^i = cos(ln2) + i·sin(ln2) |
| 4.11 | `100 ENTER 0 → XEQ LOGZ ENTER` | `(2.0000, 0)` | log10(100) = 2 |
| 4.12 | `2 ENTER 0 ENTER 0 ENTER 2 → XEQ Z↑W ENTER` (or `Z^W`) | `(-4, 0)` | 2^2i (note: W on top, Z below) |

### Complex domain guards

| # | Setup → XEQ | Expected | Notes |
|---|-------------|----------|-------|
| 4.13 | `0 ENTER 0 → XEQ CINV ENTER` | `data error` | 1/0 |
| 4.14 | `0 ENTER 0 → XEQ LNZ ENTER` | `data error` | ln(0) undefined |
| 4.15 | `0 ENTER 0 → XEQ LOGZ ENTER` | `data error` | log(0) undefined |
| 4.16 | `0 ENTER 0 ENTER 1 ENTER 0 → XEQ A↑Z ENTER` | `data error` | 0^z for non-positive Re(z) |
| 4.17 | `0 ENTER 0 ENTER 0 ENTER 1 CHS → XEQ Z↑W ENTER` | `data error` | 0^(-1) (Re(W) ≤ 0) |

All guards must fire BEFORE the `complex_mode` flag flips (verify the `C`
annunciator state if it was off before the op).

## 5. Triangle Solvers

Five ops, all angle-mode-DEPENDENT. **Set angle mode to `DEG` first**:
CLI press `D`, GUI click `f` then click `SIN` (which is the unshifted DEG
function on the SIN key per HP-41 hardware).

XEQ names: `SSS`, `ASA`, `SAA`, `SAS`, `SSA`. Each takes 3 inputs from the
stack and writes 3 outputs (the remaining sides/angles).

Standard 3-4-5 right triangle is the simplest sanity check (sides 3, 4, 5).

| # | XEQ | Stack pre (X, Y, Z) | Stack post (X, Y, Z) — what each output means | Notes |
|---|-----|---------------------|------------------------------------------------|-------|
| 5.1 | `3 ENTER 4 ENTER 5 → XEQ SSS ENTER` | a=5, b=4, c=3 | A≈36.87°, B≈53.13°, C=90° | SSS: 3 sides → 3 angles |
| 5.2 | `90 ENTER 5 ENTER 53.1301 → XEQ ASA ENTER` | A=90, c=5, B=53.13 | b=4, a=3, C=36.87° | ASA: angle-side-angle |
| 5.3 | `4 ENTER 53.1301 ENTER 90 → XEQ SAA ENTER` | a=4, A=53.13, B=90 | C=36.87°, b=5, c=3 | SAA: side-angle-angle (opposite side known) |
| 5.4 | `4 ENTER 90 ENTER 3 → XEQ SAS ENTER` | b=4, A=90, c=3 | a=5, B=53.13°, C=36.87° | SAS: side-angle-side |
| 5.5 | `5 ENTER 4 ENTER 53.1301 → XEQ SSA ENTER` | a=5, b=4, A=53.13 | B=53.13°, C=73.74°, c≈6 | SSA *unambiguous* case (a ≥ b) |

### SSA Ambiguous Case (Pitfall — TRI-05 fidelity gate)

When `a < b · sin(A)`, no triangle exists. When `a ≥ b · sin(A)` AND `a < b`,
TWO triangles exist (`B1`/`C1`/`c1` and `B2`/`C2`/`c2`). The emulator
prints both candidates and prints `ANGLE UNDEFINED` for the impossible case
per OM p.46 TRI-05.

| # | Setup → XEQ | Expected print_buffer | Notes |
|---|-------------|----------------------|-------|
| 5.6 | `3 ENTER 5 ENTER 30 → XEQ SSA ENTER` | Two solutions in print_buffer (B1/C1/c1, B2/C2/c2) | a=3, b=5, A=30°: ambiguous |
| 5.7 | `1 ENTER 5 ENTER 30 → XEQ SSA ENTER` | `ANGLE UNDEFINED` (no triangle) | a=1 < b·sin(A) = 2.5 |

CLI: the print_buffer drains to the status panel below the stack display.
GUI: print_buffer drains to the "TAPE" panel (Phase 31 makes this visible;
in Phase 28 the buffer is filled but not yet routed to a UI surface — verify
via `cargo run --bin hp41` for CLI today).

## 6. POLY (Sub-Entry ROOTS only)

`POLY` master entry opens a `DEGREE=?` modal that requires Phase 29/31 step
advance wiring (see §9). The `ROOTS` sub-entry is testable today — it reads
coefficients directly from R00..R05 and writes roots to print_buffer in the
**Pitfall 5 fidelity format**:

- Real root r: `U=<r>`
- Complex pair (u ± iv): `U=<u>` / `V=<v>` / `U=<u>` / `-V=-<v>` (4 lines)

Coefficient convention: A = R00 (highest degree), B = R01, … F = R05.
The polynomial is A·xⁿ + B·xⁿ⁻¹ + … + (constant term).

### Preparation: load coefficients

```
PRGM mode off. Then:
1     STO 00    ← A = 1   (x² coefficient)
0     STO 01    ← B = 0   (x¹ coefficient)
1 CHS STO 02    ← C = -1  (constant term)
```

Polynomial: `x² − 1 = 0`. Expected roots: +1, -1.

### Tests

| # | Setup | XEQ | Expected print_buffer (FIX 4) | Notes |
|---|-------|-----|-------------------------------|-------|
| 6.1 | as above | `XEQ ROOTS ENTER` | `U=1.0000` and `U=-1.0000` (two lines) | Two real roots — Pitfall 5 format |
| 6.2 | `1 STO 00 / 0 STO 01 / 1 STO 02` (= x² + 1) | `XEQ ROOTS ENTER` | `U=0.0000 / V=1.0000 / U=0.0000 / -V=-1.0000` (four lines) | Complex pair: ±i |
| 6.3 | `1 STO 00 / 5 CHS STO 01 / 6 STO 02` (= x² − 5x + 6) | `XEQ ROOTS ENTER` | `U=3.0000` and `U=2.0000` | Distinct real roots from the QUAD demo |

### POLY non-convergence (POLY-07)

```
1 STO 00 / 0 STO 01 / 0 STO 02 / 0 STO 03 / 0 STO 04 / 1 CHS STO 05  (x⁵ − 1)
```

| # | XEQ | Expected | Notes |
|---|-----|----------|-------|
| 6.4 | `XEQ ROOTS ENTER` | 5 roots printed (1 real, 2 complex pairs) | Bairstow deflation converges for degree-5 |
| 6.5 | construct a deliberately ill-conditioned polynomial (e.g. all coeffs 1e9) → `XEQ ROOTS ENTER` | `data error` if `\|residual\| > 1e9` | POLY-07 non-convergence guard |

## 7. MATRIX Kernels (Programmatic Preset)

The MATRIX master entry (`XEQ MATRIX`) opens a multi-step modal that is
deferred to Phase 29/31. The numerical kernels `DET`, `INV`, `SIMEQ` and the
inspection ops `SIZE`, `VMAT`, `VCOL`, `EDIT` require two scratch fields
preset before invocation:

- `state.matrix_dim = Some((rows, cols))`
- `state.matrix_active_reg = Some(base_register_index)`

These fields are normally populated by the master-entry modal. In Phase 28
there is **no operator-facing path** to set them via the CLI/GUI keyboard
alone. Manual verification today requires either:

1. **Approach A — Rust integration test only:** run `cargo test -p hp41-core math1_matrix_flow` and inspect the assertions in
   `hp41-core/tests/math1_matrix_flow.rs`. These tests preset the fields
   programmatically.
2. **Approach B — Phase 29 CLI debug shortcut (deferred):** a future CLI
   debug command `:matrix-preset <rows> <cols> <base>` will surface this for
   manual testing.
3. **Approach C — Phase 31 GUI master-modal walk-through (deferred):** click
   through the full `MATRIX` → dimensions → "enter matrix? Y/N" → EDIT flow.

| # | Test path | Today's verification |
|---|-----------|----------------------|
| 7.1 | `Op::MatDet` on a known 3×3 identity → 1.0 | `cargo test -p hp41-core --test math1_matrix_flow det_identity_3x3` |
| 7.2 | `Op::MatInv` on a 2×2 invertible matrix | `cargo test -p hp41-core --test math1_matrix_flow inv_2x2_simple` |
| 7.3 | `Op::MatSimeq` solves Ax = b for a 2×2 system | `cargo test -p hp41-core --test math1_matrix_flow simeq_2x2` |
| 7.4 | `Op::MatDet` on a near-singular matrix uses ADR-003 `INV_EPSILON = 1e-10` | `cargo test -p hp41-core --test math1_matrix_flow det_near_singular` |

Re-run any of these after touching `hp41-core/src/ops/math1/matrix.rs` to
verify behavioral non-regression.

## 8. INTG / SOLVE / DIFEQ (Program-Wrapper Required)

`Op::Integ`, `Op::Solve`, `Op::Sol`, and `Op::Difeq` are **run_loop-only ops**:
direct XEQ from interactive mode returns `data error` (`HpError::InvalidOp`)
by design — they can only execute as a step inside a running program (because
they re-enter `run_loop` to evaluate the user function repeatedly).

To verify today, write a wrapper program that:

1. Stores the user-function label in ALPHA.
2. Stores integration limits / initial guesses in scratch registers.
3. Calls Op::Integ / Op::Solve / Op::Difeq as a single program step.

### Example: INTG of f(x) = x² from 0 to 1 (expected ≈ 0.3333)

```
PRGM mode on.

Step 01: LBL "MAIN"                      ; f LBL ALPHA M A I N ALPHA
Step 02: 0 STO 00                        ; x_lo = 0
Step 03: 1 STO 01                        ; x_hi = 1
Step 04: ALPHA F ALPHA                   ; F is the function label
Step 05: INTG                            ; XEQ "INTG" inserted as program step
Step 06: RTN                             ; return to caller
Step 07: LBL "F"                         ; user function f(x)
Step 08: X^2                             ; f(x) = x²
Step 09: RTN                             ; return value in X

PRGM mode off. Verify listing: 10 lines (END auto-appended).
```

| # | Setup → action | Display (X) | Notes |
|---|----------------|-------------|-------|
| 8.1 | Press R/S to run the program from the top (or `XEQ MAIN ENTER`) | `0.3333` (FIX 4) | ∫₀¹ x² dx = 1/3 |
| 8.2 | Change Step 08 to `SIN`, re-run | `0.4597` (FIX 4) | ∫₀¹ sin(x) dx = 1 - cos(1) ≈ 0.4597 (RAD mode) |

### Strict-reject of nested user-callback (XROM-08 final form)

Write a second program where the user function itself calls INTG/SOLVE/DIFEQ.
The outer call returns `data error` BEFORE any state mutation — the
3-state guard checks `integ_state || solve_state || difeq_state` per
ADR-002 / D-28.7.

| # | Setup | Expected |
|---|-------|----------|
| 8.3 | Replace Step 08 with another `INTG` call (nested) | First-level INTG runs the outer integration setup; the nested call to INTG fires `data error` and unwinds. After: outer scratch registers preserved. |

### SOLVE example: root of f(x) = x² − 2 (expected √2 ≈ 1.4142)

```
PRGM mode on. Write:

Step 01: LBL "S"                ; LBL_S as user function
Step 02: ENTER                  ; preserve X
Step 03: ×                      ; X·X = X²
Step 04: 2 -                    ; X² − 2
Step 05: RTN

Step 06: LBL "SMAIN"            ; main entry
Step 07: ALPHA S ALPHA           ; function label
Step 08: 1 STO 00               ; guess_1
Step 09: 2 STO 01               ; guess_2
Step 10: SOL                    ; sub-entry (XEQ "SOL")
Step 11: RTN

XEQ "SMAIN" + ENTER             ; X-register = 1.4142
```

### DIFEQ example: y' = y, y(0) = 1 → y(1) ≈ e ≈ 2.7183

```
PRGM mode on.

Step 01: LBL "DMAIN"
Step 02: ALPHA F ALPHA          ; function label
Step 03: 1 STO 00               ; order = 1
Step 04: 0.1 STO 01             ; step_size h
Step 05: 0 STO 02               ; x0 = 0
Step 06: 1 STO 03               ; y0 = 1
Step 07: 10 STO 05              ; max_steps = 10 (covers x0..1)
Step 08: DIFEQ
Step 09: RTN

Step 10: LBL "F"                ; user function f(x, y) = y for the test ODE
Step 11: RDN                    ; pop x, leaves y on top
Step 12: RTN

XEQ "DMAIN" + ENTER             ; X-register ≈ 2.7183 (e ≈ 2.71828)
```

The RK4 4th-order error at h=0.1 over 10 steps is ≈ 1e-5; FIX 4 display
shows the correct value to all visible digits.

### Cancellation (Pitfall 11 — partial)

`state.cancel_requested` is plumbed from Phase 28 but the UI keypress that
SETS this flag is wired in Phase 31 (GUI). For CLI today, cancellation is
not user-reachable. The cancellation **infrastructure** (per-64-samples
lock release, clean unwinding of `integ_state`/`solve_state`/`difeq_state`)
is covered by `cargo test -p hp41-core --test math1_user_callback`.

## 9. Deferred to Phase 29 (CLI) / Phase 31 (GUI)

The following Math Pac I user-facing flows require modal-prompt **display**
and modal-input **routing** that are intentionally not in Phase 28's scope:

| Op | Behavior in Phase 28 | What's missing |
|----|----------------------|----------------|
| `XEQ POLY` | sets `state.modal_prompt = Some("DEGREE=?")` | CLI/GUI does not render `modal_prompt` to the status line; user cannot advance through DEGREE → A=? → … → F=? prompts |
| `XEQ MATRIX` | sets `modal_prompt = Some("ROWS=?")` | same — operator cannot enter dimensions, choose enter/edit, or kick off DET/INV/SIMEQ |
| `XEQ FOUR` | sets `modal_prompt = Some("NO. SAMPLES=?")` | same — operator cannot enter sample count or invoke the USER-mode E-key DFT evaluator |
| `XEQ TRANS` | sets `modal_prompt = Some("X0,Y0,θ?")` | same — 2D origin and rotation cannot be entered |
| `XEQ T3D` | sets `modal_prompt = Some("ORIGIN?")` | same — 3D origin + axis + θ cannot be entered |
| Modal R/S submit | `Op::RunStop` would advance the modal one step | not wired into CLI input handler nor GUI key dispatch |

**Workaround today:** call the sub-entry op directly with preset registers (as
in §6 ROOTS), or run the master entry inside a wrapper program that
provides parameters via scratch registers (as in §8). Phase 29/31 will close
this gap.

**Verification that the modal-opener side IS correct** — to confirm that
`XEQ POLY` (etc.) does set the modal state without crashing:

| # | Setup → XEQ | Expected | How to verify (today) |
|---|-------------|----------|-----------------------|
| 9.1 | `XEQ POLY ENTER` | No error; `state.modal_program = Some(ModalProgram::Poly(DegreePrompt))` | `cargo test -p hp41-core --test math1_poly dispatch_poly_workflow_succeeds` |
| 9.2 | `XEQ MATRIX ENTER` | No error; modal state set | `cargo test -p hp41-core --test math1_matrix dispatch_matrix_workflow_*` |
| 9.3 | `XEQ FOUR ENTER` | No error; modal state set | `cargo test -p hp41-core --test math1_four_tri_trans four_workflow_*` |
| 9.4 | `XEQ TRANS ENTER` | No error; modal state set | same test file, `trans_workflow_*` |

## 10. Error Path Summary

Three failure categories produced by Math Pac I ops; each leaves the
calculator state otherwise unchanged.

| Category | Trigger | Display | Examples in this doc |
|----------|---------|---------|---------------------|
| `data error` (Domain) | Math undefined (ACOSH(0.5), LN(0), ATANH(±1)) | `data error` | §2.9–2.11, §4.13–4.17 |
| `data error` (DivideByZero) | Reciprocal/division of zero (CINV(0), C÷ by 0+0i) | `data error` | §3.6, §4.13 |
| `data error` (InvalidOp) | Op runnable only inside `run_loop` (INTG/SOLVE/SOL/DIFEQ) invoked directly | `data error` | §8 preamble |

Cross-UI guarantee: the **same** `HpError` variant surfaces as the **same**
`data error` text in both `hp41-cli` (status line) and `hp41-gui` (toast
overlay). The text comes from `hp41-core/src/error.rs` Display impl; both UIs
read it.

## 11. Same Procedure in the GUI

Mirror sections 2 through 8 exactly. Three GUI-specific input paths to be
aware of:

**XEQ-by-name entry**: click the orange-bordered `XEQ` button (top row in
the 5×8 grid). The XEQ-by-name modal opens. Type letters via:

- The on-screen keyboard — each key's blue label (`alphaChar` below the
  primary label) types into the open XEQ modal directly (no separate ALPHA
  toggle).
- The physical keyboard — A–Z keystrokes route to the modal.

Press `ENTER` to commit the name; the resolver chain dispatches via
`xrom_resolve()` (last in chain — Pitfall 1 protected) to the Math Pac I
op.

**Angle mode switching** (§5 triangle solvers): click `f` (orange SHIFT
top-row), then click the SIN key — `f-prefix + SIN` is the conventional
DEG mnemonic on HP-41 hardware. `f + COS` selects RAD; `f + TAN` selects
GRAD. Verify via the `DEG`/`RAD`/`GRAD` annunciator that lights up.

**Print buffer visibility**: §5.6, §6.1–6.3, §8.x output goes to
`state.print_buffer`. In Phase 28, the GUI does not yet display this buffer
on a dedicated TAPE panel — verify via the CLI (`hp41` binary) for these
specific cases. Phase 31 (GUI Integration) will route the buffer to a
GUI-rendered TAPE.

**Cross-UI behavioral guarantee:** sections 2 through 5 results
(numeric stack values, annunciator states, error types) MUST be **identical**
between CLI and GUI. Both UIs dispatch through the same `hp41-core` ops; the
result is determined entirely by the core library. Mismatches are an SC-4
violation (calculator logic in `hp41-gui/src-tauri/` is forbidden) and a
release blocker.

## Known Limitations

- All 22 modal-opener flows (POLY, MATRIX, FOUR, TRANS, T3D — plus their
  multi-step prompt sequences) are not interactively reachable in Phase 28.
  Tested via Rust integration tests and via the §8 wrapper-program pattern
  for the user-callback subset (INTG/SOLVE/DIFEQ). Tracked for Phase 29
  (CLI) and Phase 31 (GUI).
- The `state.print_buffer` content (SSA dual-solution output, POLY roots,
  Triangle solver outputs in textual form) requires CLI today; GUI display
  lands in Phase 31.
- INTG cancellation (`state.cancel_requested = true`) is not wired to a
  CLI/GUI keypress in Phase 28 — only the back-end plumbing is in place.
- The `complex_atan2` helper (`hp41-core/src/ops/math1/complex.rs`) carries
  `#[allow(dead_code)]` — the complex transcendentals chose direct `f64::atan2`
  bridging instead. Behavior is unaffected; tracked for Phase 32 cleanup.
- INTG/SOLVE/DIFEQ scratch register clobber by the user function is a
  documented hardware-faithful divergence (see
  [hp41-math1-divergences.md](hp41-math1-divergences.md) Divergence 1).
  Do not `STO` to R00–R07 inside an integration / solver / ODE callback or
  the result will be silently wrong.

## See Also

- [HP-41C Math Pac I Owner's Manual citations — Divergences](hp41-math1-divergences.md)
- [ADR-003 — INV_EPSILON (matrix invert / determinant tolerance)](adr/v3.0-003-inv-epsilon.md)
- [ADR-004 — INTG convergence threshold](adr/v3.0-004-intg-threshold.md)
- [Phase 28 plans and summaries](../.planning/phases/28-xrom-framework-math-pac-i-core-ops/)
- [Phase 28 verification report](../.planning/phases/28-xrom-framework-math-pac-i-core-ops/28-VERIFICATION.md)
- [Card Reader verification — companion document](verifying-card-reader.md)
