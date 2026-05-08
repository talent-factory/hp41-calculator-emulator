# Domain Pitfalls: HP-41 Emulator v1.1 Feature Set

**Domain:** HP-41 behavioral emulation — v1.1 additions to a hardened v1.0 Rust codebase
**Researched:** 2026-05-08
**Applies to:** STO arithmetic keyboard modals, EEX lock behavior, Print emulation, Synthetic programming

---

## Context: The v1.0 Foundation

v1.0 shipped with `#![deny(clippy::unwrap_used)]` (zero panics in core), 94.87% test coverage,
and all ~130 ops explicitly declaring `LiftEffect::Enable / Disable / Neutral`. The two
hardest behavioral features were ISG/DSE counter parsing (string-split, never `floor()`/`fmod()`)
and stack-lift semantics — both are now correct. Every new feature must slot into this
existing correctness model without loosening it.

The pitfalls below are specific to the four v1.1 features. They assume v1.0 architecture is
unchanged: `op_sto_arith()` exists in `registers.rs`, the modal `PendingInput` enum lives in
`app.rs`, and `flush_entry_buf()` in `ops/mod.rs` handles EEX.

---

## Critical Pitfalls

### Pitfall C1: STO arithmetic applied to stack registers — not just R00-R99

**What goes wrong:**
The current `op_sto_arith()` validates `reg >= 100` and returns `InvalidOp`. On the HP-41
hardware, `STO+` and `STO-` (and `×` and `÷`) accept not just data registers R00–R99 but also the
stack registers: X, Y, Z, T, and LASTX (L). For example, `STO+ Y` adds X to the Y register in
place. This is accessed on the hardware keyboard via `STO` then `+` then `.` then the stack
register letter.

An implementation that only exposes the R00–R99 path misses a documented HP-41 feature. More
dangerously, if a `u8` register number scheme is extended to encode stack registers (e.g. 100–104
for X/Y/Z/T/L), and the existing `reg >= 100 → InvalidOp` guard is not updated, valid stack
register operations silently fail.

**Why it happens:**
The v1.0 `StoArith { reg: u8, kind: StoArithKind }` encoding assumes `reg` is a data register
index. Stack registers have no natural index in 0–99. The guard `reg >= 100 → InvalidOp` was
correct for v1.0 data-register-only STO arithmetic but is wrong if the feature is extended.

**Consequences:**
Stack register arithmetic silently returns `InvalidOp` when the user tries `STO+ Z`, which is
valid HP-41 behavior. Users copying HP-41 programs that use `STO+ Z` or `STO- T` see error
messages with no explanation.

**Prevention:**
Before implementing keyboard modals, decide the register encoding. Two clean options:

1. Keep `reg: u8` for data registers (0–99) and add a separate `StoArithStack` variant in `Op`
   with an enum target `StackReg { X, Y, Z, T, L }`. Both map to `op_sto_arith_stack()` in
   `registers.rs`.
2. Use a `StoTarget` enum replacing `reg: u8` in `StoArith`: `StoTarget::DataReg(u8)` or
   `StoTarget::StackReg(StackReg)`. Cleaner but requires touching all existing `StoArith` match arms.

The keyboard modal in `app.rs` then presents a second-level menu: after `STO+`, the user
sees a prompt for register number **or** stack register letter (X/Y/Z/T/L).

Update `op_sto_arith` to remove the blanket `reg >= 100 → InvalidOp` guard if the u8 encoding
is extended to cover stack registers.

**Detection:**
Test: `STO+ Y` where X=5, Y=10 should produce Y=15 with X and stack lift unchanged. If this
returns `InvalidOp`, the guard is not updated.

**Phase:** STO arithmetic modal phase. Must be designed before keyboard modal wiring.

---

### Pitfall C2: EEX trailing-e flush produces `InvalidOp` instead of hardware lock behavior

**What goes wrong:**
On HP-41 hardware, pressing `EEX` without a preceding mantissa shows `1  00` in the display
(the calculator inserts a default mantissa of 1). The user is then in "exponent entry mode":
the next digits go into the 2-digit exponent field, and pressing an operation key before
entering exponent digits commits the partial entry as `1E+00 = 1`. The key phrase is that
the hardware **locks** you into this state — it doesn't error, it waits.

The current implementation in `flush_entry_buf()` (and the test `test_flush_trailing_e_without_exponent_returns_err`)
explicitly documents that `"1.5e"` in `entry_buf` returns `Err(HpError::InvalidOp)`. This
is partially correct: the **emulator** correctly blocks flushing a naked trailing `e`, but
it must still be possible to commit the number. The hardware behavior is:

- Pressing `EEX` with digits already in `entry_buf` (e.g. `"1.5"`) → appends `e`, display
  shows `1.5  _` (waiting for exponent digits). Pressing an op key commits `1.5E+00`.
- Pressing `EEX` with empty `entry_buf` → auto-populates `"1"` into entry_buf, then appends
  `e`. Display shows `1  _`. Pressing an op key commits `1E+00 = 1`.

**The real trap:** The current `entry_buf.contains('e') && !entry_buf.ends_with_digit()`
case means `flush_entry_buf("1.5e")` returns `Err`. This is correct for parse-time rejection
but wrong for the user experience: pressing `+` after `1.5 EEX` should commit `1.5E+00 = 1.5`,
not show an error.

**Why it happens:**
`flush_entry_buf` tries `Decimal::from_str` then `Decimal::from_scientific`. Both reject `"1.5e"`
because it lacks exponent digits. The function returns `Err` and clears `entry_buf`. The user
sees an error flash and loses their number entry.

**Consequences:**
Any user who presses `EEX` and then immediately presses an operation key (intending to enter
a number times 10^0 = the number itself) sees an `invalid operation` error instead of the
number being committed. Programs that use `EEX` followed by `0` `0` work fine; casual use of
`EEX` as a prefix followed by immediate operation is broken.

**Prevention:**
Two-part fix:

1. In `app.rs` `handle_key`, when `entry_buf` ends with `e` and a non-digit operation key is
   pressed, **normalize** `entry_buf` before calling `dispatch`. Either append `"00"` (making
   `"1.5e00"`) or strip the trailing `e` (making `"1.5"`). Both produce the same result:
   the number without a real exponent. The strip-trailing-e approach is simpler and matches
   the "EEX with no exponent typed = E+00" semantics.

2. In `flush_entry_buf`, add normalization before parse:
   ```rust
   if s.ends_with('e') || s.ends_with('E') {
       s.push_str("00"); // or strip the 'e'
   }
   ```

3. The test `test_flush_trailing_e_without_exponent_returns_err` must be updated: it currently
   asserts `Err`, which becomes wrong if normalization is added. This test will need to be
   changed to assert `Ok` with the correct value.

**The hardware "lock" aspect:**
The HP-41 display shows a `_` (cursor) in the exponent field after EEX. It does not allow
pressing the decimal point or alpha keys while in exponent mode. The current `app.rs` guards
(`'.' blocked after 'e'`) already implement this. What is missing is the graceful fallback
when no exponent digits are entered.

**Detection:**
Test sequence: push `"1.5"` into `entry_buf`, press `EEX`, then press `+`. Should commit
`1.5E+00 = 1.5` and add to Y, not show error. Also test: empty `entry_buf`, press `EEX` →
`entry_buf` should become `"1e"` (or `"1"` immediately), then press `5` → `entry_buf`
should be `"1e5"`.

**Phase:** EEX lock phase. Requires updating `flush_entry_buf`, `handle_key`, and the
existing trailing-e test.

---

### Pitfall C3: Print commands bypass `hp41-core` boundary, introducing I/O into core

**What goes wrong:**
`PRX`, `PRA`, and `PRSTK` produce output (to stdout or a file). There is a strong temptation
to implement this by adding `println!` calls or file I/O inside `hp41-core`'s `dispatch()`
or a new `op_prx()` function. This violates the core architectural invariant:
`hp41-core` must have zero I/O dependencies.

If `std::io::stdout()` or any file path appears in `hp41-core`, the crate can no longer be
reused as-is by the v2.0 GUI (Tauri), which has its own output mechanism. It also breaks
the `#![deny(clippy::unwrap_used)]` guarantee indirectly (file I/O `unwrap()`s are
the first thing developers reach for).

**Why it happens:**
Print operations feel like they belong to the compute layer — they read the stack and format
output. The formatting step is core logic; the I/O step is not. Conflating them is natural
but wrong.

**Consequences:**
- `hp41-core` gains a dependency on `std::io::Write` or `std::fs::File`.
- The v2.0 GUI cannot reuse `hp41-core` without also taking the file I/O behavior.
- Coverage measurement becomes unreliable (file I/O in tests requires temp file setup).
- `#![deny(clippy::unwrap_used)]` violations appear in the print path.

**Prevention:**
Split print into two concerns:

1. **Formatting (core):** Add `Op::PrX`, `Op::PrA`, `Op::PrStk` to the `Op` enum.
   Dispatching them in `execute_op`/`dispatch` **formats** the output string using the
   existing `format_hpnum()` from `format.rs` and pushes the string into a new
   `print_buffer: Vec<String>` field on `CalcState`. The core never writes to any I/O handle.

2. **Output (CLI):** After `call_dispatch()` in `app.rs`, drain `state.print_buffer` and
   write each line to stdout (or a file configured via CLI flag). The CLI owns all I/O.

`CalcState::print_buffer` is a `Vec<String>` that the core only appends to. The CLI drains it
after each dispatch. Tests assert the buffer contents without any I/O.

```rust
// In CalcState::new():
print_buffer: Vec::new(),

// In op_prx():
let formatted = format_hpnum(&state.stack.x, &state.display_mode);
state.print_buffer.push(formatted);
apply_lift_effect(state, LiftEffect::Neutral);
Ok(())

// In app.rs call_dispatch():
hp41_core::ops::dispatch(&mut self.state, op)?;
for line in self.state.print_buffer.drain(..) {
    println!("{}", line);  // or write to configured output
}
```

**Detection:**
`grep -r "use std::io" hp41-core/src/` — any match is a violation. Also: `cargo check` in
`hp41-core` without `hp41-cli` in the build.

**Phase:** Print emulation phase. Architecture decision must be made before any `Op::PrX` code.

---

### Pitfall C4: Synthetic programming allows arbitrary `Op` variants as direct byte injection, corrupting `is_running` and call stack

**What goes wrong:**
"Synthetic programming" in the HP-41 hardware exploits a firmware bug to inject non-standard
byte sequences. In a behavioral emulator that uses a `Vec<Op>` as program storage, the
conceptual equivalent is inserting `Op` variants that the normal keyboard recording path
would never produce. The most dangerous case: inserting `Op::PrgmMode` directly into a
running program's `Vec<Op>`.

In `run_loop()`, `Op::PrgmMode` is not handled in the `match op { ... }` arms of `execute_op()`
— it falls through to the `Err(HpError::InvalidOp)` arm that handles "Programming ops
handled by run_loop directly". If byte injection can produce a `PrgmMode` in the middle of a
running program, the program halts with `InvalidOp` and `is_running` is reset to `false`.
That alone is safe. But if byte injection can produce an unbalanced `Op::Xeq` (call without
matching RTN), the 4-deep call stack fills and `HpError::CallDepth` is returned — but
any state mutations made before that point are not rolled back.

The deeper risk: the HP-41's real dangerous bytes modify OS flags and internal registers
(the equivalent of writing to `state.is_running`, `state.prgm_mode`, or `state.pc` directly).
In a Rust `Op` enum emulator, this translates to allowing Op variants that set those fields
directly without going through the normal guards.

**Why it happens:**
Synthetic programming is often implemented as "let the user push arbitrary `Op` variants into
`state.program`." This seems safe because `Op` is a typed enum — you can't inject memory
corruption. But the behavioral effects of certain Op sequences are still dangerous:
- `Op::PrgmMode` mid-program: exits program recording state mid-run.
- `Op::Gto("MISSING")`: `find_in_program` returns `Err(HpError::InvalidOp)`, but the
  program has already been partially executed.
- Nested `Op::Xeq` chains exceeding 4 levels: `HpError::CallDepth` with partial call stack.

**Consequences:**
- `is_running` left in unexpected state if error handling is incomplete.
- Half-executed programs leave `CalcState` in a partially mutated state.
- `call_stack` not cleared on error (it is cleared by `run_program()` only at the start,
  not on error in `run_loop`).

**Prevention:**
The existing `run_program()` already sets `state.is_running = false` on all paths (the
`state.is_running = false` reset after `run_loop` is always executed). The `call_stack` is
cleared at the start of `run_program()` but not on `run_loop` error. Add:

```rust
// In run_program(), after result = run_loop():
state.is_running = false;
state.call_stack.clear(); // clear on both Ok and Err paths
result
```

For synthetic programming specifically, define a safe insertion API:

```rust
pub fn insert_synthetic_op(program: &mut Vec<Op>, pos: usize, op: Op) -> Result<(), HpError> {
    // Reject Op variants that only belong in the execution control path:
    match &op {
        Op::PrgmMode => return Err(HpError::InvalidOp),
        // Op::Lbl, Op::Gto, Op::Xeq, Op::Rtn are valid synthetic ops
        _ => {}
    }
    program.insert(pos, op);
    Ok(())
}
```

Do NOT expose raw `state.program.push(op)` to a synthetic programming UI without this guard.

**The minimal safe implementation:**
Limit synthetic programming in v1.1 to inserting/replacing `Op::Lbl`, `Op::Gto`, `Op::Xeq`,
`Op::Rtn`, and `Op::PushNum` at arbitrary positions. This covers all documented legitimate
HP-41 synthetic programming uses without enabling the OS-register class of operations.

**The risky implementation to avoid:**
Exposing a "raw byte insert" that accepts any `Op` variant including `Op::PrgmMode`,
`Op::UserMode`, and future variants that directly mutate mode flags.

**Detection:**
Test: insert `Op::PrgmMode` at position 1 of a running program; assert the call returns
`Err` and `state.is_running == false` and `state.call_stack.is_empty()`.

**Phase:** Synthetic programming phase. The safe insertion API must be designed before
any UI for byte injection is wired.

---

## Moderate Pitfalls

### Pitfall M1: STO arithmetic modal — Esc during second-digit entry leaves orphaned modal state

**What goes wrong:**
The existing `handle_reg_modal()` in `app.rs` handles `StoRegister` and `RclRegister` modals
with a 2-digit accumulator. The modal state machine is:

1. Press `S` → `PendingInput::StoRegister("")`
2. Press first digit → `PendingInput::StoRegister("0")`
3. Press second digit → dispatch `Op::StoReg(reg)`, clear pending

Adding STO arithmetic modals (`StoAdd`, `StoSub`, `StoMul`, `StoDiv`) requires a **3-step**
modal: `S` then arithmetic key (`+`/`-`/`×`/`÷`) then 2-digit register.

The trap: the existing `handle_reg_modal()` is a 2-step modal. The 3-step modal needs to
distinguish step 1 (arithmetic operator not yet selected) from step 2 (operator selected,
awaiting register digits). If `Esc` is pressed during step 2 (operator selected, no digits
yet), `pending_input` must be cleared — but if the code re-uses `handle_reg_modal` naively,
it may instead loop back to step 1 instead of cancelling entirely.

**Why it happens:**
`handle_reg_modal` is designed to be called from a `PendingInput` variant that already knows
the target operation. The 3-step STO arithmetic modal adds an intermediate state
(`StoArithOp` — operator chosen, register not yet entered). If this intermediate state is
not a named variant, the `match` in `handle_pending_input` doesn't see it, and `Esc` at the
wrong moment falls through to the `None` arm.

**Prevention:**
Add an explicit intermediate `PendingInput` variant:

```rust
pub enum PendingInput {
    // existing:
    StoRegister(String),
    RclRegister(String),
    // new:
    StoArithOp,            // waiting for +/-/×/÷
    StoAdd(String),        // operator chosen; accumulating register digits
    StoSub(String),
    StoMul(String),
    StoDiv(String),
    // ...
}
```

The `Esc` arm in `handle_pending_input` must unconditionally clear `pending_input` in every
state. Add a test: press `S`, press `+`, press `Esc` → `pending_input` is `None`.

**Detection:**
Manually: `S` `+` `Esc` — the STO arithmetic prompt must disappear and not leave a stale modal.
Test: `app.pending_input` is `None` after that sequence.

**Phase:** STO arithmetic modal phase.

---

### Pitfall M2: PRSTK print order — T is printed first, X last (top-to-bottom = top-of-stack first)

**What goes wrong:**
`PRSTK` on the HP-41 with the 82143A printer prints the stack in descending order from T to
X. The physical paper shows T at the top (printed first), then Z, then Y, then X at the
bottom. This matches how you would read a stack listing: the "bottom of the stack" (T) is
at the top of the paper, and the "display register" (X) is at the bottom near the tear line.

Naive implementations print X first (it's the "top" in a push-down stack) — this is the
opposite of HP-41 behavior.

Additionally, `PRSTK` prints the LASTX register and the ALPHA register after X. The full
output order is: T, Z, Y, X, LASTX, ALPHA (if alpha_reg is non-empty).

Format per line: the number is printed using the current display mode (FIX/SCI/ENG), using
the full 24-character print width (not the 12-character display width). The HP-41's 82143A
printer prints 24 characters per line. For emulation, pad/truncate to 24 characters.

**Why it happens:**
Confusion between "stack top" (X, the most recently entered number) and "first to print"
(T, the bottom of the 4-register stack). The natural Rust iteration order over `[x, y, z, t]`
prints X first.

**Prevention:**
```rust
fn op_prstk(state: &mut CalcState) -> Result<(), HpError> {
    let lines = vec![
        format!("{:>24}", format_hpnum(&state.stack.t, &state.display_mode)),
        format!("{:>24}", format_hpnum(&state.stack.z, &state.display_mode)),
        format!("{:>24}", format_hpnum(&state.stack.y, &state.display_mode)),
        format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode)),
        format!("{:>24}", format_hpnum(&state.stack.lastx, &state.display_mode)),
    ];
    // append ALPHA if non-empty
    state.print_buffer.extend(lines);
    if !state.alpha_reg.is_empty() {
        state.print_buffer.push(
            format!("{:>24}", &state.alpha_reg[..state.alpha_reg.len().min(24)])
        );
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Detection:**
Test: set T=1, Z=2, Y=3, X=4. Call `op_prstk`. Assert `print_buffer[0]` contains "1" and
`print_buffer[3]` contains "4".

**Phase:** Print emulation phase.

---

### Pitfall M3: PRX uses the current display mode, not a fixed format

**What goes wrong:**
`PRX` prints the X register. The trap: it prints X **in the current display mode**, not
always in a fixed 10-digit scientific format. If the user is in `FIX 2` mode, `PRX` prints
the rounded 2-decimal-place value, not the full precision value. This is correct HP-41
behavior, but emulators sometimes hardcode 10-digit scientific output for "maximum precision."

`PRA` prints the ALPHA register contents, not the X register. These are often confused.

Format width: the 82143A prints 24 characters per line. A number shorter than 24 characters
is right-aligned (the HP-41 right-aligns numbers, left-aligns alpha text).

**Prevention:**
- `op_prx`: `format_hpnum(&state.stack.x, &state.display_mode)` (uses current mode).
  Right-align to 24 chars.
- `op_pra`: Push `state.alpha_reg.clone()` (truncated to 24 chars, left-aligned) to
  `print_buffer`. Alpha text is left-aligned.
- `op_prstk`: See Pitfall M2.

Test: set `display_mode = FIX(2)`, X = 3.14159. `PRX` output should be `"                  3.14"`,
not `"             3.14159000E 00"`.

**Phase:** Print emulation phase.

---

### Pitfall M4: Synthetic programming corrupts serde round-trip of `state.program`

**What goes wrong:**
`state.program: Vec<Op>` is serialized to JSON for persistence. The `Op` enum derives
`Serialize/Deserialize`. Adding new `Op` variants for synthetic programming (e.g. a hypothetical
`Op::RawByte(u8)` for byte-level injection) changes the serde representation. Old save files
with `"RawByte"` JSON tokens fail to deserialize with the old binary.

More subtly: if synthetic programming inserts `Op::PushNum(HpNum(...))` at unusual positions
(not appended, but spliced), the JSON is still valid but the program semantics change on
load. Programs that used absolute `pc` indices embedded in comments or test fixtures become
wrong after insertion/deletion.

**Why it happens:**
The `Vec<Op>` program is a flat list — there is no line-number or address concept. Inserting
at position 5 shifts all subsequent `pc` values. If any code caches absolute `pc` offsets
(for example, the USER mode key assignments that store label names, not PCs, are safe — but
any hypothetical "bookmark" that stores a `usize` offset is not).

**Prevention:**
- Do not add `Op::RawByte(u8)` as a persistent variant. Synthetic programming should inject
  existing `Op` variants only. If truly novel behavior is needed, model it as a named `Op`
  variant with defined semantics, not as an opaque byte.
- After any insert/delete into `state.program`, reset `state.pc = 0` and clear `state.call_stack`.
  Stale `pc` values after mutation are a silent corruption source.
- Add a `#[serde(default)]` guard: any new `Op` variants should be non-breaking in
  existing JSON (use `#[serde(skip_serializing_if = "is_default")]` patterns or add the
  variant to the forward-compatibility list in `persistence.rs`).

**Detection:**
Test: record a program, save to JSON, insert a synthetic op at position 2, save again, load
the first JSON, verify `pc` resets correctly. Load the second JSON with the old binary — verify
graceful deserialization failure (not panic).

**Phase:** Synthetic programming phase.

---

### Pitfall M5: Coverage gate degrades when new ops are added without tests

**What goes wrong:**
v1.0 achieved 94.87% core coverage. Each new `Op` variant added to `Op` enum adds new
`match` arms in `dispatch()` and `execute_op()`. If those arms are not covered by tests,
the coverage denominator increases but the numerator does not, causing the ≥80% gate to
potentially fail.

The four v1.1 features add at minimum:
- `Op::PrX`, `Op::PrA`, `Op::PrStk` — 3 new dispatch arms
- Synthetic programming insertion API — new code paths
- EEX normalization — new branch in `flush_entry_buf`
- STO arithmetic modal — new `PendingInput` variants in `app.rs` (CLI coverage, not core)

**Prevention:**
For each new `Op` variant, write at minimum:
1. A unit test that dispatches it and asserts `print_buffer` or stack state.
2. A test that dispatches it **inside a running program** via `run_program()` (covers the
   `execute_op` path, which is separate from the interactive `dispatch` path).

The `execute_op()` function in `program.rs` is a separate match from `dispatch()`. Adding
a new `Op` to `dispatch()` without adding it to `execute_op()` causes a compile error
(non-exhaustive match) — but only if `execute_op` is `#[deny(unreachable_patterns)]`. Add
that lint to `program.rs` to catch missing arms at compile time.

For EEX normalization: add a test for `flush_entry_buf("1.5e")` returning `Ok` (this
replaces the existing test that asserts `Err`).

**Detection:**
Run `just coverage` after adding each new Op variant. Coverage below 94% indicates a missing
test. The CI coverage gate at ≥80% is a floor, not a target.

**Phase:** Every v1.1 phase.

---

## Minor Pitfalls

### Pitfall m1: STO arithmetic divide-by-zero — existing `checked_div` guard is correct but error message is wrong

**What goes wrong:**
`op_sto_arith` with `StoArithKind::Div` calls `checked_div`, which returns
`Err(HpError::DivideByZero)` when X is zero. This is correct. However, the error surfaces
in `app.rs` as `self.message = Some(format!("{e}"))`, which shows "divide by zero" in the
status bar. On the HP-41 hardware, `STO÷ n` with X=0 shows "DATA ERROR" not "divide by zero."

This is a minor display fidelity issue (not a behavioral correctness issue) but may
confuse users comparing emulator behavior to hardware.

**Prevention:**
Map `HpError::DivideByZero` in the `StoArith` context to a more specific message, or
unify all arithmetic errors as `"DATA ERROR"` to match HP-41 annunciator behavior. Low
priority for v1.1.

**Phase:** STO arithmetic modal phase (low priority).

---

### Pitfall m2: Print output to file — file handle not flushed on program termination

**What goes wrong:**
If print emulation writes to a `BufWriter<File>`, and the program halts via
`HpError::Overflow` (the infinite-loop guard) or `HpError::CallDepth`, the `BufWriter`
may not be flushed before the error is returned. Last few print lines are lost.

**Prevention:**
Use `std::io::LineWriter` instead of `BufWriter` for file output, so each line is flushed
immediately. Or, since `print_buffer` in `CalcState` accumulates all output before the CLI
drains it, ensure the CLI flushes after draining even on error paths:

```rust
// In app.rs call_dispatch():
let result = hp41_core::ops::dispatch(&mut self.state, op);
for line in self.state.print_buffer.drain(..) {
    if let Some(ref mut writer) = self.print_writer {
        let _ = writeln!(writer, "{}", line);
    } else {
        println!("{}", line);
    }
}
if let Some(ref mut writer) = self.print_writer {
    let _ = writer.flush();
}
result.map_err(|e| { self.message = Some(format!("{e}")); });
```

**Phase:** Print emulation phase (file output sub-feature).

---

### Pitfall m3: EEX with empty entry_buf — hardware shows "1  _" (implicit 1 mantissa)

**What goes wrong:**
On HP-41 hardware, pressing `EEX` with no digits yet entered (display shows `0.0000`) puts
`1  _` in the display — it enters exponent mode with an implicit mantissa of 1. The user
then types exponent digits. Pressing `5` produces `1  05` = `1E+05 = 100000`.

The current guard in `app.rs`:
```rust
if c == 'e' {
    if self.state.entry_buf.is_empty() || self.state.entry_buf.contains('e') {
        return; // silently ignore
    }
    self.state.entry_buf.push('e');
    ...
}
```

This blocks `EEX` when `entry_buf` is empty, preventing the "implicit 1" behavior. The
hardware does allow this — it is a legitimate entry mode.

**Consequences:**
Users who try to type `EEX 6` (meaning `1E+06`) get no response. The `EEX` key silently
does nothing. This is a usability gap, not a crash. Low severity.

**Prevention:**
Change the guard:
```rust
if c == 'e' {
    if self.state.entry_buf.contains('e') {
        return; // already have 'e' — block duplicate
    }
    if self.state.entry_buf.is_empty() {
        self.state.entry_buf.push('1'); // implicit mantissa of 1
    }
    self.state.entry_buf.push('e');
    ...
}
```

This matches HP-41 hardware: `EEX` with no mantissa inserts `1` as the implicit mantissa.

The existing test `test_eex_blocked_when_entry_buf_empty` must be updated to assert that
`entry_buf` becomes `"1e"`, not that it remains empty.

**Phase:** EEX lock phase.

---

### Pitfall m4: Synthetic programming — serde JSON program display in prgm_display.rs shows garbage for novel ops

**What goes wrong:**
`prgm_display.rs` formats `Vec<Op>` for the program listing TUI pane. If a new `Op` variant
added for synthetic programming does not have a `Display` or format arm in `prgm_display.rs`,
the program listing shows the raw Rust debug repr (`PrX` or `StoArithStack { reg: X, ... }`)
instead of the HP-41 mnemonic (`PRX` or `ST+ X`).

This is not a crash but breaks the fidelity of the program display for users reading their
synthetic programs.

**Prevention:**
For every new `Op` variant, add a corresponding arm to the format function in
`prgm_display.rs`. The mnemonic must match the HP-41 keyboard label, not the Rust variant
name. This is a compile-warning opportunity: if `prgm_display.rs` uses an exhaustive match
over `Op` variants, missing arms cause a compile error.

Check: does `prgm_display.rs` currently use `match op { ... _ => format!("{:?}", op) }`
(catch-all) or exhaustive match? A catch-all silently hides missing arms.

**Phase:** Any phase that adds new `Op` variants.

---

## Phase-Specific Warnings for v1.1

| Phase Topic | Pitfall | Mitigation |
|-------------|---------|------------|
| STO arithmetic modals | C1: stack register addressing | Decide `StoTarget` encoding before wiring modals |
| STO arithmetic modals | M1: Esc during 3-step modal | Add `StoArithOp` intermediate `PendingInput` variant |
| STO arithmetic modals | m1: divide-by-zero message | Map to `DATA ERROR` for HP-41 fidelity |
| EEX lock behavior | C2: trailing-e flush returns Err | Normalize `entry_buf` ending in `e` before parse |
| EEX lock behavior | m3: empty entry_buf EEX | Insert implicit `"1"` into entry_buf on EEX with empty buf |
| EEX lock behavior | C2: existing test must change | `test_flush_trailing_e_without_exponent_returns_err` → assert Ok |
| Print emulation | C3: I/O in hp41-core | Use `print_buffer: Vec<String>` in CalcState; drain in CLI |
| Print emulation | M2: PRSTK print order | T first, X last; include LASTX and ALPHA |
| Print emulation | M3: PRX format | Use current `display_mode`, right-align to 24 chars |
| Print emulation | m2: file handle not flushed | Flush after each drain, even on error |
| Synthetic programming | C4: dangerous Op variants | Gating `insert_synthetic_op` to block `Op::PrgmMode` |
| Synthetic programming | C4: call_stack not cleared on error | Add `state.call_stack.clear()` after `run_loop` returns Err |
| Synthetic programming | M4: serde round-trip with new Op | Reset `pc` after any program mutation; no `RawByte` variant |
| Synthetic programming | m4: prgm_display gaps | Exhaustive match in `prgm_display.rs`; no `_ =>` catch-all |
| All phases | M5: coverage gate | Two tests per new Op variant: interactive dispatch + run_program |

---

## Sources

- HP-41 STO arithmetic on stack registers: [Store and Recall on HP Calculators — Richard Nelson](http://h20331.www2.hp.com/hpsub/downloads/S04_Jul12_Store%20&%20Recall%20on%20HP%20CaLCS%20V5.pdf); [HP-41 Programming — hpmuseum.org](https://www.hpmuseum.org/prog/hp41prog.htm); [HP41 stack behavior — narkive](https://comp.sys.hp48.narkive.com/hJidHxqw/hp41-help-me-understand-the-stack-behaviour)
- EEX normalization / implicit 1 mantissa: [HP-41C Synthetic Programming — hpmuseum.org](https://www.hpmuseum.org/prog/synth41.htm) (discusses EEX CHS normalization); [HP-41C Quick Reference Guide — hpcalc.org](https://literature.hpcalc.org/community/hp41c-qrg-en.pdf)
- PRSTK print order (T–Z–Y–X–L–Alpha): [CC41 HP-41CX Emulator source — CraigBladow/cc41 on GitHub](https://github.com/CraigBladow/cc41)
- PRX format (current display mode, 24-char width): [HP 82143A printer — 24 char width, per hpmuseum](https://www.hpmuseum.org/journals/hp41/41pr.htm); [Emu41 Documentation](https://www.jeffcalc.hp41.eu/emu41/files/emu41eng.pdf)
- Synthetic programming safety: [HP-41C Synthetic Programming — hpmuseum.org](https://www.hpmuseum.org/prog/synth41.htm); [Synthetic programming — Wikipedia](https://en.wikipedia.org/wiki/Synthetic_programming_(HP-41)); [HP-41 Synthetic Quick Reference — hpcalc.org](https://literature.hpcalc.org/community/hp41-synthetic-qrg.pdf); [Forum: Synthetic on HP41CX — hp41.org](https://forum.hp41.org/viewtopic.php?f=20&t=214)
- STO arithmetic LASTX behavior: [hp33s Register Arithmetic — hp.com](http://h20331.www2.hp.com/Hpsub/downloads/33sRegister.pdf); [Stack unchanged for STO arithmetic — narkive](https://comp.sys.hp48.narkive.com/hJidHxqw/hp41-help-me-understand-the-stack-behaviour)
- hp41-core I/O boundary: Project CLAUDE.md (`hp41-core` must never depend on `hp41-cli` or `hp41-gui`); v1.0 codebase architecture
