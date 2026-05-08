# Architecture Patterns

**Domain:** HP-41 Calculator Emulator — v1.1 Integration Architecture
**Researched:** 2026-05-08
**Focus:** Integration points for four v1.1 features into the existing v1.0 codebase

---

## Existing Architecture (v1.0 Baseline)

### Component Map

```
hp41-core (library crate — zero UI deps)
  src/
    lib.rs             — public API re-exports; #![deny(clippy::unwrap_used)]
    state.rs           — CalcState (single source of truth for all mutable state)
    stack.rs           — Stack, apply_lift_effect(), enter_number()
    num.rs             — HpNum (rust_decimal newtype)
    format.rs          — format_hpnum(), format_alpha()
    error.rs           — HpError enum
    ops/
      mod.rs           — Op enum, dispatch(), flush_entry_buf()
      arithmetic.rs    — op_add/sub/mul/div
      math.rs          — trig, log, exp, ...
      registers.rs     — op_sto, op_rcl, op_sto_arith, op_clreg
      stack_ops.rs     — op_enter, op_clx, op_chs, ...
      alpha.rs         — op_alpha_toggle, op_alpha_append, ...
      program.rs       — run_program(), run_loop(), execute_op(), parse_counter()
      stats.rs         — sigma, mean, sdev, lr, ...
      hms.rs           — HMS conversions

hp41-cli (binary crate — depends on hp41-core)
  src/
    main.rs            — entry point
    app.rs             — App struct, handle_key(), handle_pending_input(), event loop
    keys.rs            — key_to_op(), KEY_REF_TABLE
    ui.rs              — render_ui() (ratatui)
    persistence.rs     — save_state(), load_state()
    help_data.rs       — HELP_DATA (? overlay content)
    prgm_display.rs    — program listing formatter
    programs.rs        — sample_programs()
```

### Key Invariants (must not be violated by v1.1)

1. `hp41-core` has zero UI/CLI dependencies — enforced at compile time.
2. All state lives in `CalcState`. No module-level globals.
3. `#![deny(clippy::unwrap_used)]` — production code uses `.expect("reason")` or `?`.
4. `dispatch()` calls `flush_entry_buf()` first, then checks `prgm_mode` gate.
5. `execute_op()` inside `run_loop()` does NOT call `flush_entry_buf()`.
6. `Op` derives `Serialize, Deserialize` — adding Op variants breaks existing JSON state files if not handled with `#[serde(default)]` on new CalcState fields.

### Data Flow (existing)

```
crossterm KeyEvent
  → app.handle_key()            [hp41-cli/src/app.rs]
    → entry_buf append          (digit/./e keys — no dispatch)
    → handle_pending_input()    (modal states)
    → handle_alpha_mode_key()   (alpha mode)
    → keys::key_to_op()         → hp41_core::ops::dispatch()
                                    → flush_entry_buf()
                                    → prgm_mode gate
                                    → op_xxx() implementations
```

---

## Feature 1: STO Arithmetic Keyboard Modals

### Current State

`PendingInput::StoAdd/StoSub/StoMul/StoDiv(String)` variants already exist in `app.rs` but are marked `#[allow(dead_code)]` with a comment "deferred to v1.1". The `handle_pending_input()` match arms for these four variants are already implemented and call `handle_reg_modal()` with the correct `Op::StoArith` constructor. The core dispatch (`Op::StoArith`) and the underlying `op_sto_arith()` function in `registers.rs` are fully implemented.

### What Is Missing

The trigger is missing. Currently pressing `S` unconditionally sets `PendingInput::StoRegister`. There is no way for the user to reach `PendingInput::StoAdd/Sub/Mul/Div`.

### Integration Points

**File: `hp41-cli/src/app.rs`**

The `handle_key()` function contains this block:
```rust
if key.code == KeyCode::Char('S') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::StoRegister(String::new()));
    ...
    return;
}
```

This must become a two-step modal. Step 1 waits for an arithmetic key (+/-/*//) or a digit. Step 1 needs a new `PendingInput` variant:

```rust
StoDispatch,  // waiting for: digit (StoRegister), + (StoAdd), - (StoSub), * (StoMul), / (StoDiv)
```

Step 2 is already implemented for all four arithmetic variants.

**File: `hp41-cli/src/app.rs` — `handle_pending_input()`**

Add a match arm for `PendingInput::StoDispatch`:
- Digit → set `PendingInput::StoRegister(digit_string)`
- `+` → set `PendingInput::StoAdd(String::new())`
- `-` → set `PendingInput::StoSub(String::new())`
- `*` → set `PendingInput::StoMul(String::new())`
- `/` → set `PendingInput::StoDiv(String::new())`
- `Esc` → clear

**File: `hp41-cli/src/keys.rs` — `KEY_REF_TABLE`**

Add entries documenting the two-step STO modal flow.

**File: `hp41-cli/src/help_data.rs`**

Update `HELP_DATA` to document the new STO arithmetic modal flow.

### New Data Structures

None in `CalcState`. One new `PendingInput` variant (`StoDispatch`) in `app.rs`.

### New Op Variants

None. `Op::StoArith { reg: u8, kind: StoArithKind }` is already defined.

### Files Changed

| File | Change |
|------|--------|
| `hp41-cli/src/app.rs` | Add `PendingInput::StoDispatch`; change `S` key handler to set `StoDispatch` instead of `StoRegister`; add match arm in `handle_pending_input()` |
| `hp41-cli/src/keys.rs` | Update `KEY_REF_TABLE` |
| `hp41-cli/src/help_data.rs` | Update help text |

### No hp41-core changes required.

---

## Feature 2: EEX Trailing-e-Without-Exponent Hardware Lock

### Current State

The existing behavior when a user presses a non-digit key while `entry_buf` ends with `'e'` (e.g., `"1.5e"`) is: `flush_entry_buf()` returns `Err(HpError::InvalidOp)`, the error is shown in the status bar, and `entry_buf` is cleared. This behavior is tested and documented in `mod flush_eex_tests::test_flush_trailing_e_without_exponent_returns_err`.

The real HP-41 hardware behavior is different: pressing a non-numeric key when the exponent field is pending but empty locks the entry — the calculator waits for exponent digits and does not execute the attempted operation. The EEX key has been pressed; the hardware is in "exponent entry mode" and will not commit an incomplete number.

### What Changes

**The behavior to implement:** When `entry_buf` ends with `'e'` (exponent pending), any key that would trigger a non-digit operation must be blocked. The entry buf stays in place; the status bar shows a message like "EEX: enter exponent". The lock is cleared only by a digit (extends the exponent) or Backspace (cancels the EEX and removes the `e`).

**File: `hp41-core/src/error.rs`**

Add a new error variant:
```rust
#[error("enter exponent")]
IncompleteEntry,
```

**File: `hp41-core/src/ops/mod.rs` — `flush_entry_buf()`**

Currently clears `entry_buf` and returns `Err` on a trailing `e`. The new behavior: detect trailing `e`, return `Err(HpError::IncompleteEntry)` WITHOUT clearing `entry_buf`. The buffer must survive the error so the user can continue typing exponent digits.

**File: `hp41-cli/src/app.rs` — digit-entry block in `handle_key()`**

Add a Backspace handler for "EEX backspace": when `entry_buf` ends with `'e'`, Backspace pops the `e` (returning to plain mantissa), rather than routing through to `Op::Clx`. This is the correct HP-41 behavior (Backspace undoes the EEX key).

**File: `hp41-cli/src/app.rs` — `call_dispatch()`**

When `dispatch()` returns `Err(HpError::IncompleteEntry)`, display "EEX: enter exponent" in the status bar but do NOT clear `entry_buf`. All other errors: existing behavior (show error, clear nothing).

### New Data Structures

None in `CalcState`.

### New Op Variants

None.

### Files Changed

| File | Change |
|------|--------|
| `hp41-core/src/error.rs` | Add `HpError::IncompleteEntry` |
| `hp41-core/src/ops/mod.rs` | `flush_entry_buf()`: detect trailing `e`, return `IncompleteEntry` without clearing buf |
| `hp41-cli/src/app.rs` | Backspace handling in digit-entry block (EEX backspace); `call_dispatch()` special-case for `IncompleteEntry` |

### Test Impact

`mod flush_eex_tests::test_flush_trailing_e_without_exponent_returns_err` must be updated: the asserted error changes to `HpError::IncompleteEntry`, and the assertion `state.entry_buf.is_empty()` inverts to assert that the buf is NOT cleared.

---

## Feature 3: Print Emulation (PRX / PRA / PRSTK)

### HP-41 Hardware Behavior

On the real HP-41 with the 82143A printer module:
- `PRX` prints the X register formatted per the current display mode
- `PRA` prints the ALPHA register string
- `PRSTK` prints all five registers: T, Z, Y, X, LASTX (and optionally the ALPHA register)

Flag 26 when set suppresses printer output on hardware — emulator support for this flag is a v1.2+ refinement.

### Architecture Decision: Print Sink

`hp41-core` must remain UI-agnostic. The print operations cannot write to stdout or a file directly from within the library. Two options were considered:

**Option A — Callback/trait sink:** Add `print_sink: Option<Box<dyn FnMut(String)>>` to `CalcState` or as a separate `dispatch()` parameter. Avoids storing output in state, but introduces trait objects, complicates serde, and changes the `dispatch()` signature.

**Option B — Print buffer in CalcState (recommended):** Add `print_buffer: Vec<String>` to `CalcState`. Print ops append lines to the buffer. `hp41-cli` drains the buffer after each `dispatch()` call and writes to stdout. No trait objects, no signature changes, fully serde-compatible with `#[serde(default)]`.

**Recommendation: Option B.** It avoids trait objects, is serde-compatible without special handling, keeps `dispatch()` signature unchanged, and lets tests inspect the buffer directly without stdout capture. The drain-after-dispatch pattern is a clean boundary.

### New Fields in CalcState

```rust
/// Lines accumulated by PRX/PRA/PRSTK. hp41-cli drains after each dispatch().
/// Intentionally not persisted between sessions: #[serde(default)] returns empty vec.
#[serde(default)]
pub print_buffer: Vec<String>,
```

### New Op Variants

```rust
/// PRX — print X register (formatted per display_mode). LiftEffect: Neutral.
Prx,
/// PRA — print ALPHA register string. LiftEffect: Neutral.
Pra,
/// PRSTK — print all stack registers (T/Z/Y/X/LASTX) + ALPHA. LiftEffect: Neutral.
Prstk,
```

### New Source File

`hp41-core/src/ops/print.rs` — contains `op_prx()`, `op_pra()`, `op_prstk()`. Each pushes one or more formatted lines onto `state.print_buffer` using `format_hpnum()` and `format_alpha()` from `hp41-core/src/format.rs`.

### Integration Points

**File: `hp41-core/src/state.rs`**

Add `print_buffer: Vec<String>` to `CalcState` with `#[serde(default)]`. Initialize as `Vec::new()` in `CalcState::new()`.

**File: `hp41-core/src/ops/mod.rs`**

Add `pub mod print;` declaration and use imports. Add three Op variants. Add three match arms in `dispatch()`.

**File: `hp41-core/src/ops/program.rs` — `execute_op()`**

Add match arms for `Op::Prx`, `Op::Pra`, `Op::Prstk` so print ops work inside programs. This is mandatory — see the execute_op mirror requirement below.

**File: `hp41-cli/src/app.rs` — `call_dispatch()`**

After `dispatch()` returns `Ok(())`, drain `self.state.print_buffer` and write each line to stdout (or a configured output file). Use `std::mem::take(&mut self.state.print_buffer)` to drain atomically.

**File: `hp41-cli/src/keys.rs`**

Add key bindings. Candidate uppercase letters currently unmapped:
- `KeyCode::Char('P')` (Shift+p) → `Op::Prx`  (lowercase `p` is PrgmMode)
- `KeyCode::Char('A')` (Shift+a) → `Op::Pra`  (lowercase `a` is Asin)
- `KeyCode::Char('K')` (Shift+k) → `Op::Prstk` (lowercase `k` is Atan)

### Files Changed

| File | Change |
|------|--------|
| `hp41-core/src/state.rs` | Add `print_buffer: Vec<String>` with `#[serde(default)]` |
| `hp41-core/src/ops/print.rs` | New file — `op_prx()`, `op_pra()`, `op_prstk()` |
| `hp41-core/src/ops/mod.rs` | `pub mod print;`, 3 new Op variants, 3 match arms in `dispatch()` |
| `hp41-core/src/ops/program.rs` | 3 match arms in `execute_op()` |
| `hp41-cli/src/app.rs` | Drain `print_buffer` after `call_dispatch()` |
| `hp41-cli/src/keys.rs` | 3 key bindings and KEY_REF_TABLE entries |
| `hp41-cli/src/help_data.rs` | Help text for print ops |

---

## Feature 4: Synthetic Programming

### HP-41 Hardware Context

Synthetic programming on the real HP-41 exploits the multi-byte FOCAL instruction encoding to inject byte sequences not accessible from the normal keyboard. The byte codes are raw FOCAL virtual machine opcodes (0x00–0xEF range). Users historically used "byte jumper" and "byte grabber" techniques to compose byte sequences that access hidden functions, null characters in ALPHA, and extended operations.

### Architecture Decision: Scope for v1.1

A full faithful FOCAL byte-code interpreter would require a complete FOCAL VM — this contradicts the project's settled "behavioral emulation, not cycle-accurate Nut CPU" decision. The appropriate v1.1 scope is:

**Raw byte injection into program memory as a new `Op` variant.** The byte is stored in the flat `program: Vec<Op>` as `Op::RawByte(u8)`. When the interpreter encounters `Op::RawByte`, it executes a best-effort behavioral mapping: known codes map to existing behaviors; unknown codes return `HpError::InvalidOp`. This gives users access to the synthetic programming workflow without a full FOCAL VM.

This is explicitly a behavioral emulation of synthetic programming's effects, not a bit-faithful FOCAL byte interpreter. The initial implementation covers the ~15–20 most commonly used synthetic operations.

### New Fields in CalcState

None. Raw byte codes are stored as `Op::RawByte(u8)` in the existing `program: Vec<Op>`. No new top-level fields required.

### New Op Variants

```rust
/// RawByte(b) — synthetic programming: raw FOCAL byte injection.
/// In prgm_mode recording: stored verbatim in program Vec like any op.
/// In execute_op(): dispatched to op_raw_byte() behavioral mapping.
/// Unknown byte codes return HpError::InvalidOp.
/// LiftEffect: depends on byte code semantics.
RawByte(u8),
```

### New Source File

`hp41-core/src/ops/synthetic.rs` — contains:

- `pub fn op_raw_byte(state: &mut CalcState, byte: u8) -> Result<(), HpError>` — behavioral dispatch table for known FOCAL byte codes.
- `pub fn focal_byte_name(byte: u8) -> &'static str` — returns the conventional mnemonic for a byte code (e.g., `"NULL"`, `"TONE 1"`) for use in program display formatting.

### Integration Points

**File: `hp41-core/src/ops/mod.rs`**

Add `pub mod synthetic;`, add `Op::RawByte(u8)` variant, add match arm in `dispatch()`.

**File: `hp41-core/src/ops/program.rs` — `execute_op()`**

Add match arm for `Op::RawByte(b)` calling `synthetic::op_raw_byte(state, b)`.

**File: `hp41-cli/src/app.rs`**

Add `PendingInput::SyntheticByte(String)` — a two-digit hex accumulator. Trigger: e.g., `Ctrl+B`. On two hex digits received, dispatch `Op::RawByte(parsed_byte)`. Esc cancels. This reuses the accumulator pattern already established by `handle_reg_modal()`.

**File: `hp41-cli/src/prgm_display.rs`**

Add formatting for `Op::RawByte(b)` — display as `"BYTE xx"` (hex) in the program listing, with the FOCAL mnemonic from `focal_byte_name()` appended where known.

**File: `hp41-cli/src/keys.rs`**

Add a trigger key for byte entry modal (KEY_REF_TABLE entry).

### Files Changed

| File | Change |
|------|--------|
| `hp41-core/src/ops/synthetic.rs` | New file — `op_raw_byte()`, `focal_byte_name()` |
| `hp41-core/src/ops/mod.rs` | `pub mod synthetic;`, `Op::RawByte(u8)`, match arm in `dispatch()` |
| `hp41-core/src/ops/program.rs` | Match arm in `execute_op()` |
| `hp41-cli/src/app.rs` | `PendingInput::SyntheticByte(String)`, hex accumulator modal |
| `hp41-cli/src/prgm_display.rs` | Format `Op::RawByte` in program listing |
| `hp41-cli/src/keys.rs` | Trigger key + KEY_REF_TABLE entry |
| `hp41-cli/src/help_data.rs` | Help text for byte entry |

---

## Dependency Graph and Build Order

### Feature Dependencies

```
Feature 1 (STO arithmetic modals)   — independent; all core code exists; only app.rs changes
Feature 2 (EEX hardware lock)       — independent; self-contained flush_entry_buf change
Feature 3 (Print emulation)         — independent; new Op variants, new CalcState field
Feature 4 (Synthetic programming)   — independent of 1-3, but benefits from Feature 3
                                      existing first (synthetic printer byte codes can
                                      call op_prx/op_pra rather than stub to InvalidOp)
```

No feature is a hard prerequisite for another. Soft coupling:

- Feature 4's `op_raw_byte()` behavioral map may want to call `op_prx()`/`op_pra()` for the printer byte codes (FOCAL bytes 0x00–0x0F include print functions). Building Feature 3 before Feature 4 allows direct reuse. Building Feature 4 first requires stubbing those bytes as `InvalidOp` initially.

### Recommended Build Order

**Phase 1: Feature 1 (STO arithmetic modals)**

Lowest risk, smallest surface area. No `hp41-core` changes required. The implementation is already scaffolded in `app.rs` — only `PendingInput::StoDispatch` and its handler are missing. `handle_reg_modal()` is proven by the existing STO/RCL implementation. Delivers immediately visible UX value and establishes confidence for the more complex modal pattern in Feature 4.

**Phase 2: Feature 2 (EEX hardware lock)**

Small, focused change touching two files in `hp41-core` and one in `hp41-cli`. Must be done carefully to avoid regressions in the `entry_buf` guards already present in `app.rs`. Completing this before Feature 3 keeps the `flush_entry_buf()` change isolated and the diff history clean.

**Phase 3: Feature 3 (Print emulation)**

Medium surface area. New `CalcState` field, new `Op` variants, new source file, drain logic in `app.rs`. Establishes the `#[serde(default)]` pattern for backward-compatible persistence — this is the template for any future CalcState additions. After this phase, Feature 4's `op_raw_byte()` can call `op_prx()`/`op_pra()` directly for printer byte codes.

**Phase 4: Feature 4 (Synthetic programming)**

Largest research and implementation surface. The behavioral byte-code table requires research into which FOCAL byte codes to implement at what fidelity. Build last to benefit from the established modal patterns (Feature 1), the refined `execute_op()` extension workflow (Feature 3), and the program display formatting established in Feature 3's integration work. Start with a minimal curated set (NULL character insertion, extended alpha characters, printer byte codes) and expand incrementally.

---

## Cross-Cutting Concerns

### execute_op() Mirror Requirement

`execute_op()` in `program.rs` is a MIRROR of `dispatch()` in `ops/mod.rs`. Every Op variant that works interactively must also work inside programs. Any new Op variant added to `dispatch()`'s match must also appear in `execute_op()`. Failing to do this causes a runtime `HpError::InvalidOp` when the op is encountered during program execution — no compile-time warning, because `execute_op()` ends with a catch-all arm for programming-specific ops.

For Features 3 and 4: after adding match arms to `dispatch()`, immediately add corresponding arms to `execute_op()` in the same change.

### Serde Backward Compatibility

Every new `CalcState` field must use `#[serde(default)]`. This ensures v1.0 JSON state files deserialize without error after upgrading to v1.1. Test pattern: deserialize a JSON blob that lacks the new field and verify the field takes its default value. `Vec<String>` defaults to `vec![]` via `Vec::default()`. Feature 3 is the only feature that adds a new `CalcState` field.

### #![deny(clippy::unwrap_used)] Compliance

All new `hp41-core` production code must use `.expect("reason")` or `?`-propagation. The `#[allow(clippy::unwrap_used)]` exemption applies to test modules only. New source files (`print.rs`, `synthetic.rs`) must not contain `.unwrap()` in non-test code.

### Coverage Gate

`just coverage` enforces at least 80% coverage on `hp41-core` (currently 94.87%). Each new `ops/` submodule must include unit tests sufficient to keep coverage above 80%. The print ops are simple (push formatted string to buffer) and easy to test. The synthetic byte dispatch table requires a test per implemented byte code mapping.

### Key Binding Space Available

Uppercase letters (Shift+letter) still unmapped as of v1.0: `B`, `M`, `N`, `P`, `Q`, `U`, `X`. Feature 3 needs 3 bindings; Feature 4 needs 1 trigger. `P/A/K` for print ops and `Ctrl+B` for byte entry are the recommended choices, leaving `B/M/N/Q/U/X` available for v1.2+.

---

## Architecture Patterns to Follow

### Pattern: New Op Submodule

When adding new operations (Features 3 and 4):

1. Create `hp41-core/src/ops/new_module.rs`.
2. Declare `pub mod new_module;` in `hp41-core/src/ops/mod.rs`.
3. Add Op variants to the `Op` enum.
4. Add match arms to `dispatch()`.
5. Add match arms to `execute_op()` in `program.rs`.
6. Add unit tests inside the new module (`#[allow(clippy::unwrap_used)]` on the test mod).

### Pattern: New CalcState Field

When adding persistent state (Feature 3):

1. Add field to `CalcState` struct with `#[serde(default)]`.
2. Initialize in `CalcState::new()`.
3. The `impl Default for CalcState` delegates to `new()` — no separate change needed.
4. Write a test that deserializes a JSON blob missing the field and verifies the default.

### Pattern: New PendingInput Modal

When adding a multi-key input flow (Features 1 and 4):

1. Add variant to `PendingInput` enum in `app.rs`.
2. Set the variant from the trigger key handler in `handle_key()` (before `key_to_op()` is called).
3. Add a match arm in `handle_pending_input()`.
4. For numeric accumulators (register numbers, hex bytes), reuse `handle_reg_modal()` via a closure that constructs the target Op.
5. Document the trigger key in `KEY_REF_TABLE` and `help_data.rs`.

---

## Anti-Patterns to Avoid

### Anti-Pattern: Print Directly from hp41-core

**What:** `op_prx()` calls `println!()` or writes to a file inside `hp41-core`.
**Why bad:** Breaks the zero-UI-deps invariant. Makes the library untestable without capturing stdout. Incompatible with a future GUI adapter.
**Instead:** Push formatted strings to `state.print_buffer`; drain in `hp41-cli/src/app.rs` after `dispatch()` returns.

### Anti-Pattern: Skipping the execute_op() Mirror

**What:** Adding Op variants to `dispatch()` without adding them to `execute_op()`.
**Why bad:** The op silently fails during program execution (`HpError::InvalidOp`) with no compile-time warning.
**Instead:** Always update both match statements in the same commit/plan.

### Anti-Pattern: New CalcState Field Without serde(default)

**What:** Adding a field to `CalcState` without `#[serde(default)]`.
**Why bad:** Users with v1.0 state files get a deserialization error on first v1.1 launch, losing their calculator state.
**Instead:** Every new field uses `#[serde(default)]`. Verify with a deserialization test.

### Anti-Pattern: Full FOCAL VM for Synthetic Programming

**What:** Implementing a cycle-accurate FOCAL byte interpreter for all 256 byte codes.
**Why bad:** Contradicts the settled "behavioral emulation, not cycle-accurate Nut CPU" decision. Requires HP ROM knowledge (legal risk). 500+ LOC for negligible user-visible value above a curated behavioral subset.
**Instead:** Behavioral dispatch table in `synthetic.rs` covering the ~15–20 most commonly used synthetic operations (NULL, extended ALPHA characters, printer byte codes).

### Anti-Pattern: Clearing Entry Buffer on IncompleteEntry

**What:** `flush_entry_buf()` clears `entry_buf` when returning `HpError::IncompleteEntry`.
**Why bad:** The user loses their mantissa and must re-enter it from scratch.
**Instead (Feature 2):** Return `IncompleteEntry` WITHOUT clearing `entry_buf`. The CLI displays the "EEX: enter exponent" message and keeps the entry alive. The buffer is only cleared by Backspace (removes trailing `e`) or by a valid subsequent operation.

---

## Sources

- Codebase direct read: `hp41-core/src/ops/mod.rs`, `program.rs`, `registers.rs`, `state.rs`, `error.rs`, `lib.rs` (2026-05-08)
- Codebase direct read: `hp41-cli/src/app.rs`, `keys.rs`, `ui.rs` (2026-05-08)
- HP-41 synthetic programming: [HP Museum Synthetic Programming Guide](https://www.hpmuseum.org/prog/synth41.htm), [Wikipedia: Synthetic Programming (HP-41)](https://en.wikipedia.org/wiki/Synthetic_Programming_(HP-41))
- HP-41 print commands: [forum.hp41.org — printer differences](https://forum.hp41.org/viewtopic.php?f=5&t=492)
- PROJECT.md and STATE.md (2026-05-08)
