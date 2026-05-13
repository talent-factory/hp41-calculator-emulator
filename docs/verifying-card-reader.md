# Verifying the Card Reader

This procedure walks an operator through a complete Card Reader round-trip
on both `hp41-cli` and `hp41-gui` to confirm the feature behaves identically
across UIs and that card files are byte-stable across save → clear → load
→ save cycles.

It exercises both program cards (`WPRGM`/`RDPRGM`) and data cards
(`WDTA`/`RDTA`), all three known error paths, and the SHA-256 round-trip
invariant.

## TL;DR

| Step | CLI keys | GUI clicks | Expected |
|------|---------|-----------|----------|
| Save program | `ALPHA QUAD ALPHA XEQ WPRGM ENTER` (or `Ctrl+W` after `ALPHA QUAD ALPHA`) | `ALPHA QUAD ALPHA XEQ WPRGM ENTER` | `~/.hp41/cards/QUAD.raw` ≈ 40–50 B |
| Clear program | `PRGM CLP` | identical | listing shows `000 END` only |
| Load program | `ALPHA QUAD ALPHA XEQ RDPRGM ENTER` (or `Ctrl+R`) | identical | program lines identical to original |
| Run program | `XEQ QUAD ENTER` | identical | `X = 3.`, `R03 = 1.`, `R06 = 3.`, `R07 = 2.` |
| Round-trip hash | `sha256sum QUAD.raw` (terminal) | (terminal) | hash stable across re-saves |

## 1. Preparation

```bash
$ rm -f ~/.hp41/autosave.json
$ rm -rf ~/.hp41/cards/
$ hp41             # or: just gui-dev
```

Operator: `Ctrl+G` (CLREG) — fresh state. Program memory is empty by default
after the autosave reset above.

### 1b. Constants Setup

The demo program uses `RCL` to fetch its coefficients so that no literal
constants appear as program steps (literal constant entry records as
`Op::PushNum`, which the `.raw` codec does not encode). Preload the
constants before entering the program:

```
1     STO 00    ← a  = 1
5 CHS STO 01    ← b  = -5
6     STO 02    ← c  = 6
4     STO 04    ← constant 4
2     STO 05    ← constant 2
```

After these five steps, verify: `RCL 00` → `1.`, `RCL 01` → `-5.`,
`RCL 02` → `6.`, `RCL 04` → `4.`, `RCL 05` → `2.`

## 2. Enter and Verify the Program

Enter the following 28-step quadratic-formula solver. It computes the roots of
`x² − 5x + 6 = 0` (roots 3 and 2), and exercises alpha labels, two-digit
register ops, CHS, and a representative spread of single-byte FOCAL ops —
covering every non-trivial codec path.

| Step | Keys | Display / Notes |
|------|------|-----------------|
| 01 | `PRGM` → `LBL` → `ALPHA Q U A D ALPHA` | `01 LBL "QUAD"` |
| 02 | `RCL 01` | `02 RCL 01` ← b = -5 |
| 03 | `X²` | `03 X²` ← 25 |
| 04 | `RCL 04` | `04 RCL 04` ← 4 |
| 05 | `RCL 00` | `05 RCL 00` ← a = 1 |
| 06 | `×` | `06 ×` ← 4 |
| 07 | `RCL 02` | `07 RCL 02` ← c = 6 |
| 08 | `×` | `08 ×` ← 24 |
| 09 | `−` | `09 −` ← 25 − 24 = 1 |
| 10 | `SQRT` | `10 √x` ← √D = 1 |
| 11 | `STO 03` | `11 STO 03` ← R03 = √D = 1 |
| 12 | `RCL 01` | `12 RCL 01` ← -5 |
| 13 | `CHS` | `13 CHS` ← 5 |
| 14 | `ENTER` | `14 ENTER` ← duplicate 5 to Y |
| 15 | `RCL 03` | `15 RCL 03` ← √D = 1 |
| 16 | `+` | `16 +` ← 5 + 1 = 6 |
| 17 | `RCL 05` | `17 RCL 05` ← 2 |
| 18 | `÷` | `18 ÷` ← 3 |
| 19 | `STO 06` | `19 STO 06` ← R06 = x1 = 3 |
| 20 | `RCL 01` | `20 RCL 01` ← -5 |
| 21 | `CHS` | `21 CHS` ← 5 |
| 22 | `ENTER` | `22 ENTER` |
| 23 | `RCL 03` | `23 RCL 03` ← √D = 1 |
| 24 | `−` | `24 −` ← 5 − 1 = 4 |
| 25 | `RCL 05` | `25 RCL 05` ← 2 |
| 26 | `÷` | `26 ÷` ← 2 |
| 27 | `STO 07` | `27 STO 07` ← R07 = x2 = 2 |
| 28 | `RCL 06` | `28 RCL 06` ← x1 back in X for display |

Exit `PRGM` mode (`ENTER` is auto-appended as `END`), then run the
reference verification:

```
XEQ "QUAD" + ENTER
```

Expected end-state:

| Slot | Value |
|------|-------|
| X (display) | `3.` |
| R03 | `1.` |
| R06 | `3.` |
| R07 | `2.` |

This is the reference state against which the post-restore run is compared in
section 3.

## 3. Program Card: WPRGM → Clear → RDPRGM

```
1.  ALPHA   Q U A D   ALPHA            ; ALPHA register = "QUAD"
2.  XEQ "WPRGM" + ENTER                ; → ~/.hp41/cards/QUAD.raw exists (~40–50 B)
3.  $ sha256sum ~/.hp41/cards/QUAD.raw → hash A
4.  PRGM mode → CLP → confirm          ; listing shows only "000 END"
5.  ALPHA   Q U A D   ALPHA
6.  XEQ "RDPRGM" + ENTER               ; listing identical to original (28 lines)
7.  XEQ "QUAD" + ENTER                 ; X=3., R03=1., R06=3., R07=2.  ← behavioural identity
8.  ALPHA   Q U A D   ALPHA
9.  XEQ "WPRGM" + ENTER                ; QUAD.raw overwritten
10. $ sha256sum ~/.hp41/cards/QUAD.raw → hash B
    ASSERT hash A == hash B            ; byte-stable round-trip
```

On CLI, steps 2 and 9 can alternatively use the comfort shortcut `Ctrl+W`
immediately after setting `ALPHA "QUAD"`. Step 6 can use `Ctrl+R`.

## 4. Data Card: WDTA → Clear → RDTA

First, load two additional data values (run after the program from section 2
has completed, so R00–R07 already carry values from that run):

```
π    STO 50      ; R50 := 3.141592653...
1 CHS STO 99     ; R99 := -1   (boundary: highest valid register index)
```

This set exercises small positive integers (R00–R07), a small negative integer
(R99), and an irrational floating-point value (R50, mantissa test) across the
full `0..=99` register range.

```
1.  [data setup above: π STO 50  /  1 CHS STO 99]
2.  ALPHA   B A C K U P   ALPHA
3.  XEQ "WDTA" + ENTER                 ; ~/.hp41/cards/BACKUP.card.json exists
                                       ; format = "hp41-data-v1", registers.len() >= 100
4.  $ sha256sum ~/.hp41/cards/BACKUP.card.json  → hash C
5.  Ctrl+G (CLREG)                     ; R00 = R50 = R99 = 0
6.  ALPHA   B A C K U P   ALPHA
7.  XEQ "RDTA" + ENTER                 ; R00..R07 restored, R50 = π, R99 = -1
8.  ALPHA   B A C K U P   ALPHA
9.  XEQ "WDTA" + ENTER                 ; BACKUP.card.json overwritten
10. $ sha256sum ~/.hp41/cards/BACKUP.card.json  → hash D
    ASSERT hash C == hash D
```

On CLI, steps 3 and 9 can alternatively use `Ctrl+D`; step 7 can use `Ctrl+F`.

## 5. Error Paths

Three failure modes must be verified:

```
F1.  (ALPHA register is empty)
     XEQ "WPRGM" + ENTER               → display shows "alpha data"

F2.  ALPHA   N O P E   ALPHA
     XEQ "RDPRGM" + ENTER              → display shows "card data: io: ..."  (file missing)

F3.  $ echo 'kaputt' > ~/.hp41/cards/BAD.card.json
     ALPHA   B A D   ALPHA
     XEQ "RDTA" + ENTER                → display shows "card data: decode-json: ..."  (bad JSON / wrong tag)
```

Each error leaves the calculator state unchanged. The display returns to the
normal stack view on the next keypress.

## 6. Same Procedure in the GUI

Mirror sections 3 and 4 exactly, but ALPHA entry is done via the GUI's
physical-keyboard pass-through. In ALPHA mode, the real keyboard is
intercepted by `resolveKeyId()` in `App.tsx` and letter keys produce ALPHA
characters, so typing `Q U A D` while the ALPHA annunciator is active enters
the card name identically to the CLI flow.

The `sha256sum` steps remain terminal commands in both cases.

**Cross-UI guarantee:** hashes A and C from sections 3 and 4 must be
**identical** between CLI and GUI runs. Both UIs call the same
`hp41-core::cardreader` codec functions; the byte output is determined
entirely by the core library. Matching hashes are the empirical SC-4 proof.

## Known Limitations

- A program containing two card ops in sequence will fail the second with
  `card data: ("pending")`. The pending card op is drained between operator
  key-presses (or Tauri calls in the GUI), not inside `run_loop`. Typical
  use — one card op per program invocation — is unaffected.
- SHA-256 stability requires the `DataCard` struct's field order to remain
  unchanged across the two saves. A codec version bump invalidates the cached
  hash. The hash comparison in section 4 is valid only across code versions
  where `DataCard` has not been altered.
- The `.raw` codec does not encode `Op::PushNum` (literal constant entry in
  program mode). Programs that need constants must preload them into registers
  before running (see section 1b).

## See Also

- [Operations Reference — Card Reader (HP 82104A)](operations-reference.md#card-reader-hp-82104a)
- [Programming Guide — Saving and Loading via Card Reader](programming-guide.md#saving-and-loading-via-card-reader)
- [Architecture — Card Reader I/O](architecture.md#card-reader-io)
