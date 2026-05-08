# Feature Landscape: HP-41 Calculator Emulator v1.1

**Domain:** Retro scientific/programmable calculator emulator (CLI/TUI)
**Researched:** 2026-05-08
**Scope:** v1.1 — four new features added to the complete v1.0 base
**Confidence:** MEDIUM (HP-41 hardware behavior verified from multiple community sources; some edge cases flagged LOW)

---

## Scope Note

v1.0 shipped all table-stakes HP-41 features (4-level RPN stack, arithmetic, registers,
keystroke programming, ALPHA mode, TUI, persistence, statistics). This document covers
only the four new v1.1 features. All v1.0 features are treated as given dependencies.

---

## Feature 1: STO Arithmetic Keyboard Modals

### What the hardware does (HIGH confidence)

HP-41 storage arithmetic (`STO+`, `STO-`, `STO×`, `STO÷`) computes `R[n] ← R[n] OP X`
and writes the result back into the target register. X and the rest of the stack are
completely unchanged. LASTX is not saved (this is not a stack arithmetic operation).
LiftEffect: Neutral.

**Target registers:** Any primary register R00–R99. Also stack registers X, Y, Z, T, and
LASTX (L) — on real HP-41 hardware, pressing STO+ then the decimal point then a stack
register letter works. R00–R99 is the overwhelming practical use case.

**Keyboard sequence on real hardware:** Press `STO`, then the arithmetic key (`+`, `-`,
`×`, or `÷`), then the two-digit register number. The display shows `ST+` (or `ST-` etc.)
as a prompt while waiting for the register digits. This is a two-step prefix sequence,
not a single key.

**Stack effects (confirmed from multiple HP-41 sources):**
- X register: unchanged
- Y, Z, T registers: unchanged
- LASTX: unchanged (not saved)
- R[n]: receives new value
- lift_enabled: unchanged (Neutral)

### Current v1.0 state

The core operation `op_sto_arith()` in `hp41-core/src/ops/registers.rs` is fully
implemented and correct — it performs `R[n] ← R[n] OP X` with Neutral lift, does not
touch LASTX, and guards against reg >= 100.

The TUI modal infrastructure is partially built: `PendingInput::StoAdd/StoSub/StoMul/StoDiv`
enum variants exist in `hp41-cli/src/app.rs`, and `handle_pending_input()` correctly routes
them through `handle_reg_modal()`. The `Op::StoArith` dispatch path in `dispatch()` is wired.

**What is missing:** No keyboard trigger sets `pending_input = Some(PendingInput::StoAdd(...))`.
The `'S'` key only opens `PendingInput::StoRegister` (plain STO). There is no way for the
user to initiate `STO+`, `STO-`, `STO×`, or `STO÷` from the keyboard.

### Table stakes (must implement)

| Behavior | Detail |
|----------|--------|
| Keyboard trigger for STO+ modal | `'S'` → show sub-prompt: `+`, `-`, `×`, `÷`, `Esc` |
| Keyboard trigger for STO- | Same sub-prompt flow |
| Keyboard trigger for STO× | Same sub-prompt flow |
| Keyboard trigger for STO÷ | Same sub-prompt flow |
| Display prompt while waiting | Show `ST+` / `ST-` / `ST×` / `ST÷` in TUI status |
| Register entry (two digits) | Reuse existing `handle_reg_modal()` accumulator logic |
| Esc cancels at any step | Already handled by `handle_reg_modal` |
| Help overlay entry | Add `STO+/-/×/÷` to `HELP_DATA` in `help_data.rs` |

### Differentiators (nice but not required for v1.1)

| Behavior | Detail |
|----------|--------|
| STO arithmetic to stack registers X/Y/Z/T/L | Low practical value; omit in v1.1 |
| Indirect addressing STO+ IND | Very advanced; omit in v1.1 |

### Anti-features

**Do not** change the core `op_sto_arith()` — it is already correct.
**Do not** add a separate two-key prefix sequence `S` then `+`; instead add a
sub-prompt after `S` that lets the user pick the arithmetic variant. This avoids
key conflicts with normal number entry.

### Dependencies on v1.0

- `PendingInput` enum in `hp41-cli/src/app.rs` (exists)
- `handle_reg_modal()` in `hp41-cli/src/app.rs` (exists)
- `Op::StoArith` and `StoArithKind` in `hp41-core/src/ops/mod.rs` (exists)
- `op_sto_arith()` in `hp41-core/src/ops/registers.rs` (exists, correct)
- `HELP_DATA` in `hp41-cli/src/help_data.rs` (needs new entries)

### Complexity: LOW

The core logic exists. This is a TUI key-binding wiring task, not an algorithmic task.
Estimated effort: add one new `PendingInput::StoArithOp` state, handle `+/-/×/÷` keystrokes
in it to transition to `StoAdd/StoSub/StoMul/StoDiv`, update `HELP_DATA`. Existing
`handle_reg_modal()` does the rest.

---

## Feature 2: EEX Trailing-E Without Exponent

### What the hardware does (MEDIUM confidence)

On real HP-41 hardware, if the user presses `EEX` during number entry (starting exponent
input) and then presses `ENTER` without having typed any exponent digits, the calculator
**treats the trailing `E` as exponent 00** — the number is committed as if the exponent
were zero. The display shows the mantissa followed by a space where the exponent would be
(the cursor indicating the exponent position), and ENTER completes the number.

Example: User types `3`, then `EEX`, then `ENTER` — result pushed to X is `3.0E00 = 3`.
The entry is not an error; it is treated as a valid complete number.

**Confidence note:** The exact display rendering during the `3 E__` state (what cursor
placeholder the hardware shows) could not be confirmed from available sources. The
functional behavior (ENTER commits the number with exponent 0) is consistent across
multiple emulator community sources. The HP-11C and HP-41 are confirmed to share this
behavioral pattern. MEDIUM confidence.

**A second variant:** If the user presses `EEX` without any preceding mantissa (empty
entry buffer), the HP-41 inserts an implicit mantissa of 1, resulting in `1.0E00 = 1`
if ENTER is pressed immediately. However, the v1.0 codebase already **blocks** EEX when
`entry_buf` is empty (correct behavior per `app.rs` line 287). This block is correct and
should be retained.

### Current v1.0 state

In `flush_entry_buf()` (`hp41-core/src/ops/mod.rs`), the string `"1.5e"` (trailing `e`
with no exponent digits) fails both `Decimal::from_str()` and `Decimal::from_scientific()`,
causing `flush_entry_buf()` to return `Err(HpError::InvalidOp)`. There is a test named
`test_flush_trailing_e_without_exponent_returns_err` that explicitly asserts this error
path — meaning the error is intentional but represents the v1.0 gap.

When this error occurs, `entry_buf` is cleared, so the partial number is silently dropped.
The user sees no result pushed to X. This diverges from hardware behavior.

Additionally, `app.rs` line 287 blocks `EEX` when `entry_buf.is_empty()` — this is
**correct** and should remain. The v1.1 fix is only for the "has mantissa, no exponent
digits" case.

### Table stakes (must implement)

| Behavior | Detail |
|----------|--------|
| Trailing-E completion | If `entry_buf` ends with `'e'` (no digit after it), strip the trailing `'e'` and parse the remainder as a plain decimal — equivalent to exponent 00 |
| No error on partial exponent | ENTER after `EEX` without exponent digits must push a valid number, not return `InvalidOp` |
| Update/remove failing test | `test_flush_trailing_e_without_exponent_returns_err` must become `test_flush_trailing_e_without_exponent_treats_as_zero_exponent` |
| Display state during entry | While `entry_buf` ends with `'e'`, render the display as `"<mantissa> E "` — show the partial exponent state visually (cursor in exponent position) |

### Differentiators (nice but not required for v1.1)

| Behavior | Detail |
|----------|--------|
| CHS on partial exponent (EEX then CHS then digits) | Hardware-accurate negative exponent entry; research Bug #6 from HP-41 docs mentions a translation issue with EEX+CHS sequences in card programs | Defer |
| Partial exponent backspace | Delete typed exponent digit; if zero digits remain, return to mantissa entry mode | Low priority |

### Anti-features

**Do not** change the `entry_buf.is_empty()` EEX block in `app.rs` — blocking EEX with
no mantissa is correct behavior.

**Do not** special-case this in `app.rs` — fix it in `flush_entry_buf()` in `hp41-core`
so the behavior is correct for both keyboard and programmatic use.

### Implementation approach

In `flush_entry_buf()`, before the parse attempt:

```rust
// Strip trailing 'e' with no exponent — hardware treats as exponent 00.
let s = if s.ends_with('e') || s.ends_with('E') {
    s[..s.len() - 1].to_string()
} else {
    s
};
```

Then proceed to parse the (now clean) string normally.

### Dependencies on v1.0

- `flush_entry_buf()` in `hp41-core/src/ops/mod.rs` (modify)
- `test_flush_trailing_e_without_exponent_returns_err` test (replace with passing case)
- Display rendering for `entry_buf` in `hp41-cli/src/ui.rs` or equivalent (add EEX state display)

### Complexity: LOW

Single-function change in `hp41-core`, one test update, and optional display polish.
The fix is a 3-line change. The display state rendering is slightly more work but still
contained in the TUI rendering path.

---

## Feature 3: Print Emulation (PRX, PRA, PRSTK)

### What the hardware does (HIGH confidence)

The HP-41 supported two thermal printer peripherals: the **HP 82143A** (direct plug-in)
and the **HP 82162A** (HP-IL connected). Both share the same printer ROM (XROM 29) and
support the same core print functions. The 82162A adds barcode and escape sequences.

**PRX (XROM 29,20):** Prints the X register using the current display format (FIX/SCI/ENG
setting and digit count). Output is one line of text in the format the display would show.
Stack is not modified. LASTX is not saved.

**PRA (XROM 29,8):** Prints the contents of the ALPHA register (up to 24 characters) as
a text string. Stack and ALPHA register are not modified.

**PRSTK (XROM 29,19):** Prints all stack registers plus LASTX and ALPHA. From the HP-42S
manual (which inherits from HP-41 conventions): "Print the contents of the stack registers
(x, y, z and t)". Community sources add LASTX and ALPHA to the output. The canonical
format is T-Z-Y-X printed top-to-bottom (T first), then LASTX, then ALPHA. Each register
is printed on its own line using the current display format for numeric values. Stack is
not modified.

**Output routing:** On real hardware, output goes to the thermal printer. In a software
emulator, the standard pattern (confirmed from HP-IL emulator documentation) is to route
output to `stdout` (for console viewing) and optionally to a text file (`PRINTER.TXT` or
similar). Output can be suppressed by clearing flag 26 on real hardware; in the emulator,
a simpler approach is appropriate.

**Format:** Numbers are formatted using the calculator's current display mode (FIX/SCI/ENG
with the current digit count), identical to what would appear on the 12-char HP-41 display.
No padding or justification beyond what the display format produces.

**Printer ROM (XROM 29) other functions:** The full printer ROM includes PRREG, PRREGX,
PRFLAGS, PRKEYS, PRP, PRPLOT, LIST, TRACE, NORM, MAN, ADV, DELAY, and more. Only PRX,
PRA, and PRSTK are in v1.1 scope.

### Table stakes (must implement)

| Function | Output | Stack Effect | LASTX |
|----------|--------|--------------|-------|
| `PRX` | X register formatted per current display mode, one line | Neutral | Not saved |
| `PRA` | ALPHA register contents (up to 24 chars), one line | Neutral | Not saved |
| `PRSTK` | T register (line 1), Z (line 2), Y (line 3), X (line 4), LASTX (line 5), ALPHA (line 6) | Neutral | Not saved |

**Output routing:**
- Print lines go to a dedicated in-memory `Vec<String>` in `CalcState` named `print_buf`
  (or similar) — this keeps `hp41-core` UI-agnostic (no stdout in the core library)
- `hp41-cli` drains `print_buf` each render tick and displays lines in a print panel
  in the TUI, or routes them to stdout if no TUI (e.g., during testing)
- A `--print-file <path>` CLI flag (or just append to `~/hp41-print.txt`) could be added
  as a differentiator but is not required for table stakes

**TUI rendering:** Add a scrollable print output panel to the TUI layout. Print lines
accumulate (cap at e.g. 200 lines to bound memory). The panel is visible when non-empty.

### Differentiators (nice but not required for v1.1)

| Feature | Value |
|---------|-------|
| Output to `PRINTER.TXT` file | Familiar to V41 users; easy to add |
| ADV (paper advance — blank line) | Very simple; one-liner |
| PRREG n (print register n) | Useful; low complexity |
| Flag 26 suppression | Hardware-accurate; low complexity |
| TRACE mode (auto-print each step) | Useful for debugging programs; moderate complexity |

### Anti-features

**Do not** implement the full XROM 29 printer ROM in v1.1 — PRPLOT, PRKEYS, LIST,
PRFLAGS, MAN, NORM, TRACE mode are all v1.2+ territory.

**Do not** put `println!()` inside `hp41-core` — the core must remain UI-agnostic.
All output routing must happen in `hp41-cli`.

### Architecture decision: print buffer in CalcState

Because `hp41-core` must not depend on `hp41-cli`, print output cannot go directly to
stdout from the core. The correct pattern is:

```rust
// In CalcState (hp41-core/src/state.rs):
pub print_buf: Vec<String>,
```

`op_prx()`, `op_pra()`, `op_prstk()` in `hp41-core` push formatted strings to
`state.print_buf`. `hp41-cli` renders the panel and clears the buffer.

The `print_buf` must be included in serde serialization to survive auto-save/load,
or explicitly excluded (ephemeral — cleared on startup). Ephemeral is simpler and
correct (printed output does not need to persist).

### Dependencies on v1.0

- `CalcState` in `hp41-core/src/state.rs` (add `print_buf: Vec<String>`)
- `Op` enum in `hp41-core/src/ops/mod.rs` (add `Op::PrintX`, `Op::PrintAlpha`, `Op::PrintStack`)
- `dispatch()` in `hp41-core/src/ops/mod.rs` (add match arms)
- Display formatting logic in `hp41-core` (reuse `format_display()` for number formatting)
- TUI layout in `hp41-cli/src/ui.rs` (add print panel widget)
- `HELP_DATA` in `hp41-cli/src/help_data.rs` (add PRX/PRA/PRSTK entries)
- `key_to_op()` in `hp41-cli/src/keys.rs` (add keyboard bindings)

### Complexity: MEDIUM

The hp41-core logic (push to `print_buf`) is LOW complexity. The TUI rendering (new panel
widget, scrolling, sizing) is MEDIUM. The number formatting reuse requires verifying that
the existing display format function is accessible from the new ops module.

---

## Feature 4: Synthetic Programming

### What the hardware does (HIGH confidence for definition; MEDIUM for emulator scope)

HP-41 synthetic programming is a technique that exploits a firmware bug to inject raw
FOCAL byte codes into program memory — byte sequences that cannot be entered through
normal keyboard operations but that the Nut CPU will execute. The technique was discovered
by HP-41 users (notably Keith Jarett, "HP-41 Synthetic Programming Made Easy") and
tacitly supported by HP.

**The byte grabber:** The foundational hardware technique uses a specific sequence of
keystrokes that exploits a ROM scanning bug to step partially through a multi-byte
instruction, causing the calculator to misinterpret the second byte as a new instruction's
prefix. This is a physical firmware hack — it literally cannot be replicated in a
behavioral emulator without executing actual ROM code.

**What synthetic programming enables:**
1. Access to hidden registers: STO M, STO N, STO O, RCL M, RCL N, RCL O — synthetic
   names for status registers not exposed via normal STO/RCL
2. Over 100 additional TONE variants (different durations and frequencies)
3. Additional ALPHA editing commands and special characters
4. Direct access to system flags and control registers
5. NULL instruction (no-operation byte, 0x00 — useful for alignment)
6. WROM (write to ROM/RAM at address) — only on models with writable RAM
7. GETKEY — read the last key pressed as a number (very useful for programs)
8. Synthetic key assignments (GASN — Generalized Assign): assign multi-keystroke
   sequences to a single key

**Critical emulator constraint:** Cycle-accurate Nut CPU synthetic programming is
explicitly out of scope per PROJECT.md ("Cycle-accurate Nut CPU simulation — high
effort, low user value"). The byte grabber technique itself cannot be replicated
in a behavioral emulator.

**What "synthetic programming emulation" means at the behavioral level:**
A behavioral emulator can support the *results* of synthetic programming — the
byte codes that synthetic programs produce — by accepting raw byte sequences as
program steps and mapping known synthetic byte codes to their behavioral effects.
This is how HP-41X handles it ("All synthetic programs are working without problems").

**The viable approach for v1.1:** Implement a subset of the most-used synthetic
instructions as first-class `Op` variants, accessible via a new program-entry mechanism
(e.g., a dedicated "byte code entry" mode). The byte grabber hardware exploit is
replaced by a deliberate software interface.

### HP-41 FOCAL byte code structure (MEDIUM confidence)

The HP-41 FOCAL instruction set uses variable-length 1–3 byte encodings. Key structure:

- Byte `0x00`: NULL (no-operation)
- Bytes `0x01`–`0x0F`: Short-form numeric literals (0 through 9, decimal point, CHS, EEX)
- Byte `0x10`–`0x1F`: Long-form double-byte instructions (with second byte as parameter)
- Bytes for STO: prefix `0x41`–`0x44` + register byte = STO variants
- Bytes for RCL: prefix `0x51`–`0x54` + register byte = RCL variants
- "Synthetic" instructions: byte combinations whose prefix+suffix normally can't be combined
  but are valid to execute (e.g., RCL prefix + LBL suffix byte = "RCL b")

The hidden registers M, N, O correspond to system register addresses (e.g., M = reg `0x6F`,
N = `0x70`, O = `0x71` in the hardware address space). These are accessible as STO/RCL
targets only via synthetic byte codes.

**GETKEY (confirmed):** A real FOCAL instruction (`XROM 25,17` in some ROM versions, or
a synthetic byte code in others) that reads the last key pressed as a number (row×10+col).
Extremely useful in interactive programs. This is the single most requested synthetic
instruction among emulator users.

### Table stakes for v1.1 (MEDIUM confidence — scoped conservatively)

Given that the full byte-code injection model is complex and the PROJECT.md explicitly
defers cycle-accurate CPU emulation, the v1.1 table stakes are a conservative subset:

| Feature | Priority | Rationale |
|---------|----------|-----------|
| `GETKEY` as a first-class Op | High | Most useful synthetic instruction; appears in many real HP-41 programs; pure behavioral — read last key as X value |
| `NULL` (no-op) instruction | High | Very simple; useful for program alignment; commonly appears in synthetic programs |
| Hidden register access: STO/RCL M, N, O | Medium | Enables a class of programs that use status registers for scratch space; behaviorally: 3 extra read/write registers named M/N/O |
| Program byte-code viewer/editor | Low | Advanced; lets users inspect raw byte codes of program steps; needed to round-trip `.raw` files (v1.2) |

### Differentiators (explicitly v1.2+ scope)

| Feature | Why Defer |
|---------|-----------|
| Full FOCAL byte code injection via byte grabber simulation | Complex; requires a byte-level program representation; conflicts with current `Vec<Op>` model |
| WROM / ROM write | Requires memory map model; very advanced |
| Synthetic TONE variants | Low user value without audio support |
| GASN (Generalized Assign) | Complex; v1.2 when `.raw` import is added |
| Full synthetic QRC coverage | 100+ instructions; substantial scope |

### Anti-features

**Do not** attempt to replicate the byte grabber firmware exploit — this requires
cycle-accurate ROM execution and is explicitly out of scope.

**Do not** redesign `program: Vec<Op>` to `program: Vec<u8>` for v1.1 — this is a
major architectural change that affects persistence, display, and all program logic.
GETKEY and NULL can be added as `Op` variants without this change.

**Do not** allow synthetic instructions to write to arbitrary memory addresses — this
would create security and stability issues in the emulator.

### Implementation approach for GETKEY

On real hardware, GETKEY is a function that returns the row-column code of the last key
pressed as a number in X. In the TUI emulator:

```rust
// Op variant:
Op::GetKey,

// CalcState field:
pub last_key: u8, // row*10 + col code of last key press

// In handle_key() before routing:
self.state.last_key = encode_key(key); // set before dispatch

// op_getkey():
// Push state.last_key as HpNum onto X (with lift).
```

This is a clean behavioral emulation of GETKEY — no firmware required.

### Implementation approach for NULL

```rust
Op::Null,
// dispatch: apply LiftEffect::Neutral; return Ok(())
// In PRGM mode: recorded as Op::Null, displayed as "NULL"
```

### Implementation approach for hidden registers M/N/O

Extend `CalcState` with three additional named registers:

```rust
pub reg_m: HpNum,
pub reg_n: HpNum,
pub reg_o: HpNum,
```

Add `Op::StoM`, `Op::StoN`, `Op::StoO`, `Op::RclM`, `Op::RclN`, `Op::RclO`.
These behave identically to STO/RCL but target the named registers instead of `regs[n]`.

### Dependencies on v1.0

- `CalcState` in `hp41-core/src/state.rs` (add `last_key: u8`, `reg_m/n/o`)
- `Op` enum in `hp41-core/src/ops/mod.rs` (add `GetKey`, `Null`, `StoM/N/O`, `RclM/N/O`)
- `dispatch()` in `hp41-core/src/ops/mod.rs` (add match arms)
- Key encoding in `hp41-cli/src/app.rs` (set `state.last_key` on every keypress)
- `HELP_DATA` in `hp41-cli/src/help_data.rs` (add entries)
- Persistence: serde for new state fields

### Complexity: MEDIUM (scoped to GETKEY + NULL + M/N/O registers)

GETKEY, NULL, and the three hidden registers are each individually LOW complexity but
together constitute a MEDIUM effort when accounting for persistence, TUI display, help
data, and test coverage. The full synthetic programming model (byte injection) is HIGH
complexity and deferred.

---

## Feature Dependencies

```
STO arithmetic keyboard modals
  → PendingInput::StoAdd/Sub/Mul/Div (v1.0, already in enum — WIRING ONLY)
  → Op::StoArith + op_sto_arith() (v1.0, complete)

EEX trailing-E fix
  → flush_entry_buf() (v1.0, modify)
  → entry_buf display rendering (v1.0, polish)

Print emulation (PRX/PRA/PRSTK)
  → CalcState.print_buf (new field)
  → Op::PrintX/PrintAlpha/PrintStack (new)
  → TUI print panel (new widget)
  → Number formatting function (v1.0, reuse)

Synthetic programming subset (GETKEY + NULL + M/N/O)
  → CalcState.last_key, reg_m/n/o (new fields)
  → Op::GetKey, Null, StoM, RclM, etc. (new)
  → Key encoding in app.rs (new, set on every keypress)
```

**Feature ordering for v1.1 phases:**

1. **EEX trailing-E fix** — smallest, safest, touches flush_entry_buf only. Do first.
2. **STO arithmetic modals** — wiring-only, no core changes. Second (quick win).
3. **Print emulation** — new CalcState field + new Ops + TUI widget. Third.
4. **Synthetic programming subset** — new CalcState fields + new Ops + key encoding. Fourth.

---

## MVP Recommendation

**Ship v1.1 with all four features.** They are all independent with no cross-dependencies.
Suggested phase sequencing (smallest/most confident first):

| Phase | Feature | Confidence | Effort |
|-------|---------|------------|--------|
| v1.1-Phase 1 | EEX trailing-E fix | HIGH | 0.5 day |
| v1.1-Phase 2 | STO arithmetic keyboard modals | HIGH | 1 day |
| v1.1-Phase 3 | Print emulation (PRX/PRA/PRSTK) | HIGH | 2 days |
| v1.1-Phase 4 | Synthetic programming subset (GETKEY + NULL + M/N/O) | MEDIUM | 2 days |

---

## Confidence Assessment

| Feature | Confidence | Reason |
|---------|------------|--------|
| STO arithmetic behavior | HIGH | Multiple HP-41 sources confirm; v1.0 core already correct; only keyboard wiring missing |
| STO arithmetic keyboard sequence | HIGH | "STO then arithmetic key then register" confirmed in community sources and existing modal infrastructure |
| EEX trailing-E hardware behavior | MEDIUM | Functional behavior (treat as exponent 0) consistent across sources; exact display rendering during partial entry not confirmed from primary source |
| PRX output format | HIGH | HP-42S manual p.32 confirms "Print x-register"; format uses current display mode |
| PRA output format | HIGH | "Print Alpha register" — unambiguous |
| PRSTK register order and contents | MEDIUM | T/Z/Y/X order confirmed; LASTX and ALPHA inclusion confirmed by community but not primary manual page |
| Synthetic programming table stakes | MEDIUM | GETKEY behavioral semantics are clear; NULL is trivial; M/N/O hidden registers confirmed in rskey.org source |
| Synthetic full byte codes | LOW | PDF sources not extractable; full table not retrieved in this research session |

---

## Sources

- HP-42S Owner's Manual p.32 via ManualsLib (PRX, PRA, PRSTK descriptions): https://www.manualslib.com/manual/801798/Hp-Hp-42s.html?page=32
- HP-42S Owner's Manual p.33 (STO+/STO-/STO×/STO÷ descriptions): https://www.manualslib.com/manual/801798/Hp-Hp-42s.html?page=33
- rskey.org Synthetic Programming: https://www.rskey.org/gene/calcgene/sp.htm
- HP Museum HP-41C Synthetic Programming page: https://www.hpmuseum.org/prog/synth41.htm
- Wikipedia HP-41 Synthetic Programming: https://en.wikipedia.org/wiki/Synthetic_Programming_(HP-41)
- HP41.org Synthetic Programming on HP-41CX forum thread: https://forum.hp41.org/viewtopic.php?f=20&t=214
- HP41.org 82143A vs 82162A printer differences: https://forum.hp41.org/viewtopic.php?f=5&t=492
- HP41.org HP-41 Bugs list: https://forum.hp41.org/viewtopic.php?f=14&t=494
- HP-41CX Emulator (cc41) README (PRSTK behavior): https://github.com/CraigBladow/cc41
- HP-41X emulator synthetic programming support: https://www.hrastprogrammer.com/hp41x/
- Eddie's Math Blog STO arithmetic on HP-41: https://edspi31415.blogspot.com/2025/04/rpn-with-hp-15c-dm32-stack-register.html
- HP-41C/41CV Operating Manual online: https://archived.hpcalc.org/greendyk/hp41c-manual/
- HP-41 XROM numbers (printer XROM 29): https://www.hpmuseum.org/software/xroms.htm
- v1.0 codebase: hp41-cli/src/app.rs (PendingInput, EEX guards), hp41-core/src/ops/registers.rs (op_sto_arith), hp41-core/src/ops/mod.rs (flush_entry_buf)
