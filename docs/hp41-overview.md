# HP-41 Overview

## What Is the HP-41?

The HP-41C is a programmable, alphanumeric scientific calculator introduced by Hewlett-Packard in 1979. It was the first HP calculator to feature an alphanumeric display, allowing it to show text labels, prompts, and variable names — not just numbers. The HP-41 family remained in production until 1990 and is still widely used and collected today.

Key characteristics:

- **RPN (Reverse Polish Notation)** entry — operands before operator
- **4-level automatic stack** — registers X, Y, Z, T
- **Alphanumeric display** — 12-character LCD (numbers and letters)
- **Fully programmable** — keystroke recording, conditional branching, subroutines
- **Expandable** — ROM modules, RAM modules, peripheral ports (printer, card reader, wand)

## Variants

### HP-41C (1979)

The original model. 63 program steps, 0 dedicated data registers by default (all memory is shared between program steps and data registers). Four expansion ports.

### HP-41CV (1980)

"Continuously Variable" memory. Equivalent to the HP-41C with five additional memory modules installed — 319 program steps and up to 64 data registers available simultaneously. Same four expansion ports.

### HP-41CX (1983)

Adds two built-in modules:

- **Extended Functions / Extended Memory** — additional string operations, matrix support, and an extended register file
- **Time Module** — real-time clock, alarms, date arithmetic

The CX is the most capable standalone unit without any plug-in modules.

## RPN: Reverse Polish Notation

In RPN, you enter operands first, then the operator. There is no `=` key.

| Algebraic | RPN keystrokes |
|-----------|---------------|
| 3 + 4     | `3` `ENTER↑` `4` `+` |
| (2 + 3) × 5 | `2` `ENTER↑` `3` `+` `5` `×` |
| sin(45°)  | `45` `SIN` |

The stack eliminates the need for parentheses and intermediate memory — intermediate results flow automatically through X → Y → Z → T.

## The Stack

The HP-41 uses a 4-register **automatic operand stack**:

```
┌───┬───────────┐
│ T │  top      │  ← oldest value (drops off when stack fills)
│ Z │           │
│ Y │           │
│ X │  bottom   │  ← displayed value, result of last operation
└───┴───────────┘
    LAST X        ← copy of X before last operation (for recovery)
```

### Stack Lift

Most operations **enable** stack lift: the next number entry pushes the stack up before placing the new digit in X. Certain operations (e.g. `ENTER↑`, `CLx`, `Σ+`, `STO`) **disable** stack lift for the following entry.

Understanding stack-lift is essential for predictable program behaviour — see [Programming Guide](programming-guide.md).

## Memory and Registers

| Resource        | HP-41C | HP-41CV | HP-41CX |
|-----------------|--------|---------|---------|
| Data registers  | 0–63 (shared) | 0–63 | 0–319 (extended) |
| Program steps   | shared pool | 319 max | more with XM |
| Alpha register  | 24 chars | 24 chars | 24 chars |
| Flags           | 0–55   | 0–55    | 0–55    |
| Stack depth     | 4      | 4       | 4       |

## Official Documentation

All original HP manuals are preserved at the **Museum of HP Calculators (MoHPC)**:

- [HP-41C/CV Owner's Manual (Vol. 1 & 2)](https://www.hpmuseum.org/41ownman.htm)
- [HP-41C/CV/CX Advanced Functions Handbook](https://www.hpmuseum.org/41advfun.htm)
- [HP-41CX Owner's Manual](https://www.hpmuseum.org/41cxman.htm)
- [HP-41 Solutions books (various)](https://www.hpmuseum.org/software/41soft.htm)
