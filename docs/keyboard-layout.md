# HP-41C/CV Keyboard Layout

> **Note:** The layout below is a schematic representation.  
> For the exact physical layout and engraving, refer to the  
> [HP-41C/CV Owner's Manual](https://www.hpmuseum.org/41ownman.htm).

## Key Layout

Each key shows: **PRIMARY** / *shifted (↑)*

```
 ┌─────────────────────────────────────────────────────────────────┐
 │  HP-41C / CV / CX                                               │
 │  ┌─────────────────────────────────────────────────────────┐   │
 │  │                    [ display ]                          │   │
 │  └─────────────────────────────────────────────────────────┘   │
 │                                                                  │
 │  [  USER  ]        [  PRGM  ]        [  ALPHA  ]               │
 │                                                                  │
 │  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐        │
 │  │  Σ+   │  │  1/x  │  │  √x   │  │  LOG  │  │  LN   │        │
 │  │ (Σ−)  │  │ (yˣ)  │  │ (x²)  │  │(10ˣ)  │  │ (eˣ)  │        │
 │  └───────┘  └───────┘  └───────┘  └───────┘  └───────┘        │
 │                                                                  │
 │  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐        │
 │  │  XEQ  │  │  STO  │  │  RCL  │  │  SST  │  │  R/S  │        │
 │  │ (GTO) │  │(→HMS) │  │ (→H)  │  │ (BST) │  │(P/R)  │        │
 │  └───────┘  └───────┘  └───────┘  └───────┘  └───────┘        │
 │                                                                  │
 │  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐        │
 │  │  x↔y  │  │  R↓   │  │  SIN  │  │  COS  │  │  TAN  │        │
 │  │(LSTx) │  │(→RAD) │  │(ASIN) │  │(ACOS) │  │(ATAN) │        │
 │  └───────┘  └───────┘  └───────┘  └───────┘  └───────┘        │
 │                                                                  │
 │  ┌─────────────────┐  ┌───────┐  ┌───────┐  ┌───────┐         │
 │  │    ENTER↑       │  │  +/−  │  │  EEX  │  │   ←   │         │
 │  │   (↑ ALPHA)     │  │(MODES)│  │(DISP) │  │(CLEAR)│         │
 │  └─────────────────┘  └───────┘  └───────┘  └───────┘         │
 │                                                                  │
 │  ┌───┐  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐            │
 │  │ON │  │   7   │  │   8   │  │   9   │  │   ÷   │            │
 │  │(↑)│  │(→DEG) │  │(→RAD) │  │(→GRD) │  │(→P)   │            │
 │  └───┘  └───────┘  └───────┘  └───────┘  └───────┘            │
 │                                                                  │
 │         ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐            │
 │         │   4   │  │   5   │  │   6   │  │   ×   │            │
 │         │(→R)   │  │ (%)   │  │(%CH)  │  │(→HMS+)│            │
 │         └───────┘  └───────┘  └───────┘  └───────┘            │
 │                                                                  │
 │         ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐            │
 │         │   1   │  │   2   │  │   3   │  │   −   │            │
 │         │(→H)   │  │(→HMS) │  │(→RAD) │  │(→DEG) │            │
 │         └───────┘  └───────┘  └───────┘  └───────┘            │
 │                                                                  │
 │         ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐            │
 │         │   0   │  │   .   │  │  R/S  │  │   +   │            │
 │         │(→DEG) │  │(→RAD) │  │       │  │       │            │
 │         └───────┘  └───────┘  └───────┘  └───────┘            │
 └─────────────────────────────────────────────────────────────────┘
```

> **Shifted functions (shown in parentheses)** are accessed by pressing  
> the **ON** key (acts as shift ↑) followed by the target key.  
> Exact shifted assignments differ between HP-41C, CV, and CX.

## Mode Keys

| Key | Function |
|-----|----------|
| `USER` | Toggle USER mode — reassigns keys to user-defined functions |
| `PRGM` | Toggle PROGRAM mode — keystrokes are recorded as program steps |
| `ALPHA` | Toggle ALPHA mode — keys enter letters instead of numbers |

## Entry Keys

| Key | Primary | Shifted |
|-----|---------|---------|
| `ENTER↑` | Push X onto stack (duplicate into Y) | Switch to ALPHA entry |
| `+/−` | Change sign of X (or complement in ALPHA) | MODES menu |
| `EEX` | Enter exponent (×10ⁿ) | DISP menu |
| `←` | Backspace / delete last digit | CLEAR submenu |

## Numeric Pad

The `0`–`9`, `.`, and `÷ × − +` keys enter digits and perform arithmetic.  
When in ALPHA mode they enter letters (A=7, B=8, C=9 …).

## Key Labeling Convention in This Emulator

The TUI represents shifted functions using the `↑` prefix notation:

```
XEQ       → primary function
↑ XEQ     → shifted function (GTO)
```

All ~130 operations accessible via the keyboard are listed in the
[Operations Reference](operations-reference.md).
