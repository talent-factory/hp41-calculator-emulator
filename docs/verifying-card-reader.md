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
| Save program | `ALPHA QUAD ALPHA XEQ WPRGM ENTER` (or `Ctrl+W` after `ALPHA QUAD ALPHA`) | `ALPHA QUAD ALPHA XEQ WPRGM ENTER` | `~/.hp41/cards/QUAD.raw` ≈ 30–40 B |
| Clear program | `PRGM CLP` | identical | listing shows `00 END.` only |
| Load program | `ALPHA QUAD ALPHA XEQ RDPRGM ENTER` (or `Ctrl+R`) | identical | program lines identical to original |
| Run program | `XEQ QUAD ENTER` | identical | `X = 3.`, `R02 = 3.`, `R03 = 2.` |
| Round-trip hash | `sha256sum QUAD.raw` (terminal) | (terminal) | hash stable across re-saves |

## 1. Preparation

```bash
$ rm -f ~/.hp41/autosave.json
$ rm -rf ~/.hp41/cards/
$ hp41             # or: just gui-dev
```

Operator: `Ctrl+G` (CLREG) — fresh state. Program memory is empty by default
after the autosave reset above.

## 2. Enter and Verify the Program

Enter the following 22-step quadratic-formula solver. It computes the roots of
`x² − 5x + 6 = 0` (roots 3 and 2), and exercises alpha labels, two-digit
register ops, constant entry, and a representative spread of single-byte FOCAL
ops — covering every non-trivial codec path.

| Step | Keys | Display / Notes |
|------|------|-----------------|
| 01 | `PRGM` → `LBL` → `ALPHA Q U A D ALPHA` | `01 LBL "QUAD"` |
| 02 | `5` `ENTER` | `02 5.` |
| 03 | `ENTER` | `03 ENTER` |
| 04 | `×` | `04 ×` |
| 05 | `4` `ENTER` | `05 4.` |
| 06 | `1` `×` | `06 1.` / `07 ×` |
| 07 | `6` `×` | `08 6.` / `09 ×` |
| 08 | `−` | `10 −` |
| 09 | `SQRT` | `11 √x` |
| 10 | `STO 01` | `12 STO 01` |
| 11 | `5` `ENTER` | `13 5.` |
| 12 | `RCL 01` | `14 RCL 01` |
| 13 | `+` | `15 +` |
| 14 | `2` `÷` | `16 2.` / `17 ÷` |
| 15 | `STO 02` | `18 STO 02` |
| 16 | `5` `ENTER` | `19 5.` |
| 17 | `RCL 01` | `20 RCL 01` |
| 18 | `−` | `21 −` |
| 19 | `2` `÷` | `22 2.` / `23 ÷` |
| 20 | `STO 03` | `24 STO 03` |
| 21 | `RCL 02` | `25 RCL 02` |
| 22 | `RTN` (or end-of-program) | `26 END` |

Exit `PRGM` mode, then run the reference verification:

```
XEQ "QUAD" + ENTER
```

Expected end-state:

| Slot | Value |
|------|-------|
| X (display) | `3.` |
| R01 | `1.` |
| R02 | `3.` |
| R03 | `2.` |

This is the reference state against which the post-restore run is compared in
section 3.

## 3. Program Card: WPRGM → Clear → RDPRGM

```
1.  ALPHA   Q U A D   ALPHA            ; ALPHA register = "QUAD"
2.  XEQ "WPRGM" + ENTER                ; → ~/.hp41/cards/QUAD.raw exists (~30–40 B)
3.  $ sha256sum ~/.hp41/cards/QUAD.raw → hash A
4.  PRGM mode → CLP → confirm          ; listing shows only "00 END."
5.  ALPHA   Q U A D   ALPHA
6.  XEQ "RDPRGM" + ENTER               ; listing identical to original (22 lines)
7.  XEQ "QUAD" + ENTER                 ; X=3., R01=1., R02=3., R03=2.  ← behavioural identity
8.  ALPHA   Q U A D   ALPHA
9.  XEQ "WPRGM" + ENTER                ; QUAD.raw overwritten
10. $ sha256sum ~/.hp41/cards/QUAD.raw → hash B
    ASSERT hash A == hash B            ; byte-stable round-trip
```

On CLI, steps 2 and 9 can alternatively use the comfort shortcut `Ctrl+W`
immediately after setting `ALPHA "QUAD"`. Step 6 can use `Ctrl+R`.

## 4. Data Card: WDTA → Clear → RDTA

First, load two additional data values (run after the program from section 2
has completed, so R00–R03 already carry values from that run):

```
π    STO 50      ; R50 := 3.141592653...
1 CHS STO 99     ; R99 := -1   (boundary: highest valid register index)
```

This set exercises small positive integers (R00–R03), a small negative integer
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
7.  XEQ "RDTA" + ENTER                 ; R00..R03 restored, R50 = π, R99 = -1
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
     XEQ "WPRGM" + ENTER               → display shows "ALPHA DATA"

F2.  ALPHA   N O P E   ALPHA
     XEQ "RDPRGM" + ENTER              → display shows "CARD DATA"  (file missing)

F3.  $ echo 'kaputt' > ~/.hp41/cards/BAD.card.json
     ALPHA   B A D   ALPHA
     XEQ "RDTA" + ENTER                → display shows "CARD DATA"  (bad JSON / wrong tag)
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
  `CARD DATA ("pending")`. The pending card op is drained between operator
  key-presses (or Tauri calls in the GUI), not inside `run_loop`. Typical
  use — one card op per program invocation — is unaffected.
- SHA-256 stability requires the `DataCard` struct's field order to remain
  unchanged across the two saves. A codec version bump invalidates the cached
  hash. The hash comparison in section 4 is valid only across code versions
  where `DataCard` has not been altered.

## See Also

- [Operations Reference — Card Reader (HP 82104A)](operations-reference.md#card-reader-hp-82104a)
- [Programming Guide — Saving and Loading via Card Reader](programming-guide.md#saving-and-loading-via-card-reader)
- [Architecture — Card Reader I/O](architecture.md#card-reader-io)
