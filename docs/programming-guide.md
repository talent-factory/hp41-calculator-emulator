# Programming Guide

## The Stack Model

The HP-41 uses a **4-register automatic operand stack** plus a LAST X register:

```
Register  Content        Role
────────  ─────────────  ───────────────────────────────────────
T         oldest value   Fills from Z when Y is consumed
Z                        Fills from Y when X is consumed
Y                        Second operand for binary operations
X         newest value   Displayed; result of every operation
LAST X    previous X     Saved before every operation that changes X
```

Arithmetic consumes two operands (X and Y) and places the result in X. The stack **drops**: Z→Y, T→Z, and T is replicated (not lost).

```
Before:    T=4  Z=3  Y=2  X=1
After  +:  T=4  Z=4  Y=3  X=3   (1+2=3; T replicated)
```

### ENTER↑

Pressing `ENTER↑` **duplicates X into Y** and lifts the stack:

```
Before:    T=d  Z=c  Y=b  X=a
After:     T=c  Z=b  Y=a  X=a   (T is lost; X replicated into Y)
```

Use `ENTER↑` to separate two numbers being keyed in:

```
3  ENTER↑  4  ×  →  12
```

---

## Stack Lift

**Stack lift** determines whether the next digit entered pushes the stack or overwrites X.

| State | Effect of next digit entry |
|-------|---------------------------|
| Enabled  | Stack lifts: current X moves to Y before new digit goes to X |
| Disabled | New digit overwrites X (no lift) |

Operations that **Disable** lift (next entry overwrites X):
`ENTER↑`, `CLx`, `CLST`, `Σ+`, `Σ−`, `STOP`, `PSE`, all `STO` operations, `CLA`, and a few others.

Operations that **Enable** lift (next entry pushes stack):
All arithmetic, math functions, `RCL`, `LASTX`, and most operations that produce a result.

Operations that are **Neutral** (do not change lift state):
`x↔y`, `R↓`, `STO` (arithmetic variants), conditionals, flag tests.

> **Rule of thumb:** If an operation produces a new number in X, it enables lift.
> If it terminates data entry, it disables lift.

---

## Writing Programs

### Recording a Program

1. Press `PRGM` to enter program mode — the display shows the current program line.
2. Key in operations exactly as you would interactively.
3. Press `PRGM` again to exit program mode.

Every keystroke in PRGM mode is stored as a **program step**.

### Labels

Labels mark the entry points for `GTO`, `GSB`, and `XEQ`:

```
LBL A       ← named label (A–J, a–e, 00–99)
  ...
RTN
```

Call it with:

```
XEQ A       ← from the keyboard
GSB A       ← from within a program (subroutine)
```

### Subroutines

`GSB lbl` pushes the return address; `RTN` pops it. Up to 7 levels of nesting.

```
LBL "MAIN"
  GSB "SUB"
  ...
RTN

LBL "SUB"
  ...
RTN
```

### Branching

```
GTO lbl     ← unconditional jump
X>0?        ← conditional: skip next step if X > 0
GTO lbl     ← this is skipped if condition is false
```

Conditionals always skip **exactly one** step (often that step is a `GTO`).

---

## ISG / DSE Loop Counters

ISG (Increment, Skip if Greater) and DSE (Decrement, Skip if Equal or less) implement counted loops.

### Counter Format

The counter is stored as a number with the format `SSSSS.FFFII`:

```
12.00510
│       │
│  ┌────┘
│  └──► FFF = final value (005), II = increment (10 → 0.10? No: 10 = 1)
└──────► SSS = current value (12)
```

More precisely, the integer part is the current counter and the decimal part encodes the final value and step size:

```
current.FFFstep
```

The fields are read by **string-splitting at the decimal point**, never with `floor()` or `fmod()` on f64, which would introduce rounding errors.

### Example: Count 1 to 5

```
1.00500  →  STO 00      (counter from 1 to 5, step 1)

LBL 01
  < loop body >
  ISG 00
GTO 01
```

`ISG 00` increments R00, then skips `GTO 01` when the current value exceeds 5 — exiting the loop.

### DSE Example: Count Down

```
5.00100  →  STO 01      (counter from 5 to 1, step 1)

LBL 02
  < loop body >
  DSE 01
GTO 02
```

---

## Flags

Flags are single-bit boolean values. Flags 0–10 are user-controlled. Flags 11–55 are system flags.

| Range | Purpose |
|-------|---------|
| 0–10  | User-defined logic |
| 11    | Display overflow |
| 28    | Low battery |
| 29    | Print enabled |
| 42    | Continuous-on enabled |
| 55    | Ignore `STOP` during `XEQ` |

Common operations:

```
SF 00         ← set flag 0
CF 00         ← clear flag 0
FS? 00        ← skip next step if flag 0 is set
FC? 00        ← skip next step if flag 0 is clear
FS?C 00       ← test set, then clear (atomic test-and-clear)
```

---

## Alpha Register

The alpha register holds up to 24 characters and is used for labels, prompts, and output.

### Building a String

```
"RESULT: "  ← enter alpha literal (in ALPHA mode, type the string)
ARCL 00     ← append register 00's value as a string
AVIEW       ← display the alpha register
```

### Prompting for Input

```
"ENTER X:"
PROMPT      ← display and halt; user keys a value and presses R/S
STO 01      ← store what user typed
```

### Alpha ↔ Numbers

```
65  XTOA    ← 65 → "A" (appended to alpha)
ATOX        ← first char of alpha → ASCII code in X
```

---

## Indirect Addressing

Any operation that takes a register address can use `IND` to look up the address from a register:

```
STO 00        ← store X in register 00
STO IND 00    ← store X in the register whose number is in R00
GTO IND 05    ← jump to the label whose number is in R05
```

This enables dynamically-addressed arrays and dispatch tables.

---

## USER Mode

In USER mode, keys A–J (the top row) execute whatever function is stored in the corresponding user key register (via `ASIGN`). This allows personalised keyboard layouts for a specific application.

---

## Tips

- Use `CLST` at the start of a program to ensure a known stack state.
- `LASTX` recovers from a wrong operation — it holds the value of X before the last operation that changed X.
- Keep subroutines short and self-contained; they share the same stack.
- Flags 0–10 are your program's boolean variables — use them instead of dedicated registers when a simple true/false suffices.

---

## See Also

- [Operations Reference](operations-reference.md)
- [HP-41C/CV Owner's Manual, Vol. 2](https://www.hpmuseum.org/41ownman.htm) — Programming chapters
- [HP-41C/CV Advanced Functions Handbook](https://www.hpmuseum.org/41advfun.htm) — Flags, system registers, indirect addressing
