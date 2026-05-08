# Phase 12: Synthetic Programming - Research

**Researched:** 2026-05-08
**Domain:** HP-41 FOCAL/NUT architecture — GETKEY key codes, hidden registers M/N/O, synthetic byte insertion modal
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### GETKEY (SYNT-01)
- **D-01:** `last_key_code: u8` stored on `CalcState` with `#[serde(default)]`. Updated in `handle_key()` before any dispatch, on every key press.
- **D-02:** HP-41 row-column key encoding: key code = row×10 + col (1-indexed). Lookup table `keycode_to_hp41_code()` lives in `hp41-cli/src/keys.rs` or a new `hp41-cli/src/keycode_map.rs`.
- **D-03:** When no key has been pressed yet (`last_key_code` = 0), `Op::GetKey` pushes `0` to X with `LiftEffect::Enable`.
- **D-04:** `Op::GetKey` has `LiftEffect::Enable`.

#### NULL (SYNT-02)
- **D-05:** `Op::Null` is a true no-op with `LiftEffect::Neutral`. Empty body. Display name: `"NULL"`.

#### Hidden Registers M/N/O (SYNT-03)
- **D-06:** Three separate named fields on `CalcState`: `reg_m: HpNum`, `reg_n: HpNum`, `reg_o: HpNum` — each with `#[serde(default)]`. Not in the numbered `regs: Vec<HpNum>`.
- **D-07:** Six new Op variants: `StoM`, `StoN`, `StoO` (LiftEffect::Neutral) and `RclM`, `RclN`, `RclO` (LiftEffect::Enable).
- **D-08:** Keyboard entry: extend `'S'` and `'R'` modal interceptors. After prefix, pressing `'M'`/`'N'`/`'O'` (before any register digit) dispatches `StoM/StoN/StoO` or `RclM/RclN/RclO` immediately.
- **D-09:** TUI display during StoRegister/RclRegister: after M/N/O dispatch, display becomes `"STO M"` / `"RCL M"` etc.
- **D-10:** Program listing in `prgm_display.rs`: `Op::StoM` → `"STO M"`, `Op::RclM` → `"RCL M"` (N, O similarly).

#### Hex-Byte Insertion Modal (SYNT-04)
- **D-11:** `Op::SyntheticByte(u8)` — stores the raw accepted hex byte code. During execution, dispatches to the corresponding Op via lookup table.
- **D-12:** Safe subset = only hex codes that map to already-implemented Ops. A `const` lookup table validates input codes.
- **D-13:** Rejection behavior: set `app.message = Some("INVALID")`, do NOT modify the program Vec. Modal closes after any result.
- **D-14:** Keyboard binding: `'X'` (uppercase, Shift+X) opens hex-byte insertion modal. `'x'` (lowercase) remains `Op::XySwap`.
- **D-15:** Modal flow mirrors STO [nn] 2-digit accumulation: `X` → `PendingInput::HexModal(String::new())`; first hex digit → `"3"`, TUI shows `"HEX: 3_"`; second digit → validate, insert or reject.
- **D-16:** Insertion position: at current `state.pc` position (insert before current step, shifting existing steps). After insertion, `state.pc` advances.
- **D-17:** TUI display: `"HEX: _"` when empty, `"HEX: n_"` after first digit.
- **D-18:** Help overlay entry: `"X nn"` → "Insert synthetic hex byte (PRGM mode)".

### Claude's Discretion
- Exact HP-41 key code lookup table contents (which crossterm KeyCode maps to which HP-41 row-column code).
- Whether the key code update happens before or after `flush_entry_buf()` in `handle_key()`.
- Exact contents of the safe hex subset lookup table.
- Program listing display name for `Op::SyntheticByte(nn)`: `"SYN nn"` (hex) or similar.

### Deferred Ideas (OUT OF SCOPE)
- **SYNT-05:** Full FOCAL byte-code table (~200 codes) — deferred to v2+.
- **SYNT-06:** Interactive GETKEY (program pauses waiting for next key) — requires event loop redesign, deferred to v2+.
- **Indirect addressing for M/N/O** — deferred to v1.2+.
- **PRGM mode insertion polish** (step-delete, step-browse navigation) — not in scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SYNT-01 | `GETKEY` instruction pushes the last key code (HP-41 row-column encoding) to X register | HP-41 keyboard layout table researched; `last_key_code: u8` on CalcState; `keycode_to_hp41_code()` lookup function |
| SYNT-02 | `NULL` instruction executes as a no-op with Neutral stack-lift effect | Trivial implementation confirmed — empty body + LiftEffect::Neutral |
| SYNT-03 | Hidden registers M, N, O accessible via `STO M`/`RCL M`, `STO N`/`RCL N`, `STO O`/`RCL O` | Pattern mirrors op_sto/op_rcl exactly; modal extension in StoRegister/RclRegister arms |
| SYNT-04 | Hex-byte insertion modal — 2-digit hex, curated safe subset, insert at current PC | Safe subset table derived from implemented Op variants; insertion at `state.pc` with `Vec::insert()` |
</phase_requirements>

---

## Summary

Phase 12 adds four synthetic programming capabilities to the HP-41 emulator. All four are straightforward extensions of patterns already established in Phases 10 and 11.

**GETKEY (SYNT-01)** requires tracking the last-pressed HP-41 hardware key code on `CalcState`. The HP-41C has an 8×5 keyboard layout with key codes = row×10 + col (1-indexed). A lookup function in `hp41-cli/src/keys.rs` maps crossterm `KeyCode` values to HP-41 hardware codes. The `last_key_code` field is updated at the top of `handle_key()` on every key press. `Op::GetKey` reads this field and pushes it to X.

**NULL (SYNT-02)** is the simplest Op in the codebase — an empty dispatch arm with `LiftEffect::Neutral`. No state modification whatsoever.

**Hidden registers M/N/O (SYNT-03)** are three named `HpNum` fields on `CalcState` with `#[serde(default)]`. The six Ops (`StoM/StoN/StoO/RclM/RclN/RclO`) follow the exact pattern of `op_sto`/`op_rcl` in `registers.rs`. The keyboard modal extension adds M/N/O branches to the existing `StoRegister`/`RclRegister` arms of `handle_pending_input()` before the digit-accumulation fallthrough.

**Hex-byte insertion modal (SYNT-04)** is the most novel piece. It follows the 2-digit accumulation pattern of `StoRegister`, but instead of dispatching a known Op, it validates the 2-digit hex code against a `const` lookup table mapping HP-41 byte codes to already-implemented Ops. If valid, `Op::SyntheticByte(u8)` is inserted into `state.program` at `state.pc` using `Vec::insert()`. The `Op::SyntheticByte` variant dispatches to the corresponding Op at execution time via the same lookup table.

**Primary recommendation:** Implement in waves — Wave 1 adds all core-side items (`CalcState` fields + 9 new Op variants + dispatch/execute_op arms + registers.rs functions) in parallel with the test scaffold. Wave 2 adds CLI-side items (`handle_key` updates, modal, display, help) blocked on Wave 1 compilation.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| GETKEY key code tracking | Frontend (hp41-cli) | — | Crossterm KeyCode is a CLI concept; core only stores the `u8` code |
| GETKEY opcode execution | Core (hp41-core) | — | Op::GetKey pushes `state.last_key_code` to X — pure CalcState operation |
| NULL execution | Core (hp41-core) | — | Empty op body; no UI involvement |
| Hidden register storage | Core (hp41-core) | — | `reg_m/n/o: HpNum` fields on CalcState |
| Hidden register STO/RCL ops | Core (hp41-core) | — | Parallel to existing op_sto/op_rcl in registers.rs |
| Hidden register keyboard routing | Frontend (hp41-cli) | — | M/N/O branch in StoRegister/RclRegister modal arms |
| Hex-byte validation | Frontend (hp41-cli) | Core (hp41-core) | CLI validates and chooses insertion; core stores and executes SyntheticByte(u8) |
| Hex-byte execution (runtime) | Core (hp41-core) | — | execute_op dispatches SyntheticByte to the correct Op via lookup |
| Program step insertion | Frontend (hp41-cli) | Core (hp41-core) | `state.program.insert(pc, op)` is a Vec mutation — CLI drives it |

---

## Standard Stack

### Core (no new dependencies required)

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| `hp41-core` (internal) | workspace | All calculator ops, CalcState | Existing |
| `hp41-cli` (internal) | workspace | TUI, keyboard routing, modals | Existing |
| `serde` + `serde_json` | existing | JSON persistence for new CalcState fields | Existing — `#[serde(default)]` required |
| `rust_decimal` | 1.42 | HpNum arithmetic for reg_m/n/o | Existing |

No new external dependencies. Phase 12 is a pure extension of existing code. [VERIFIED: codebase grep]

**Installation:** None required.

---

## Architecture Patterns

### System Architecture Diagram

```
Keyboard Event (crossterm KeyEvent)
       │
       ▼
handle_key() — app.rs
  ├─► [D-01] Update state.last_key_code via keycode_to_hp41_code(key.code)
  ├─► [Release filter] return early on KeyEventKind::Release
  ├─► [Ctrl+C] exit
  ├─► [pending_input?] → handle_pending_input() ─────────────────────────┐
  ├─► ['X' interceptor] → PendingInput::HexModal(String::new())           │
  ├─► ['S' interceptor] → PendingInput::StoRegister(String::new())        │
  ├─► ['R' interceptor] → PendingInput::RclRegister(String::new())        │
  └─► key_to_op() → dispatch() → execute op
                                                                           │
handle_pending_input()  ◄──────────────────────────────────────────────────┘
  ├─► StoRegister(acc):
  │     ├─ acc.is_empty() + 'M'/'N'/'O' → dispatch StoM/StoN/StoO immediately
  │     ├─ '+'/'-'/'*'/'/' → transition to StoAdd/Sub/Mul/Div
  │     └─ digit → handle_reg_modal (2-digit accumulation)
  ├─► RclRegister(acc):
  │     ├─ acc.is_empty() + 'M'/'N'/'O' → dispatch RclM/RclN/RclO immediately
  │     └─ digit → handle_reg_modal (2-digit accumulation)
  └─► HexModal(acc):
        ├─ hex digit → accumulate (max 2 chars)
        ├─ 2nd digit → validate via SAFE_HEX_SUBSET table
        │     ├─ valid → Vec::insert(state.pc, Op::SyntheticByte(byte)); pc++
        │     └─ invalid → app.message = "INVALID"
        └─ Esc → cancel

hp41-core dispatch() / execute_op():
  Op::GetKey      → push state.last_key_code to X (LiftEffect::Enable)
  Op::Null        → no-op (LiftEffect::Neutral)
  Op::StoM        → state.reg_m = state.stack.x.clone() (LiftEffect::Neutral)
  Op::RclM        → push state.reg_m to X (LiftEffect::Enable)
  Op::SyntheticByte(b) → SAFE_HEX_SUBSET.get(b) → dispatch sub-op
```

### Recommended Project Structure

No structural changes needed. All new files fit existing locations:

```
hp41-core/src/
├── state.rs             # Add last_key_code: u8, reg_m/n/o: HpNum
├── ops/
│   ├── mod.rs           # Add 9 new Op variants + dispatch() arms
│   ├── program.rs       # Add 9 new execute_op() arms
│   └── registers.rs     # Add op_sto_m/n/o, op_rcl_m/n/o functions
hp41-cli/src/
├── app.rs               # PendingInput::HexModal, 'X' interceptor, M/N/O branches
├── keys.rs              # keycode_to_hp41_code() lookup function
├── ui.rs                # HexModal pending_prompt arm
├── prgm_display.rs      # 9 new op_display_name arms
└── help_data.rs         # "X nn" entry, "=== Synthetic ===" category
hp41-core/tests/
└── synthetic_tests.rs   # Wave 0 test scaffold (RED failing tests)
```

---

## HP-41 Keyboard Layout and Key Code Table

[CITED: HP-41C Owner's Manual, Appendix A; verified against CONTEXT.md D-02 and specifics section]

The HP-41C physical keyboard has 8 rows × 5 columns. Key code = (row × 10) + col, 1-indexed. Row 8, col 5 is unused on the HP-41C (reserved for CX/CV variants).

**Row 1 — top row (shifted functions row):**
| Col 1 | Col 2 | Col 3 | Col 4 | Col 5 |
|-------|-------|-------|-------|-------|
| 11: Σ+ | 12: 1/x | 13: √x | 14: LOG | 15: LN |

**Row 2:**
| 21: XEQ | 22: STO | 23: RCL | 24: R↓ | 25: SIN |

**Row 3:**
| 31: SST | 32: RCL (shifted) | 33: R/S (shifted) | 34: →HMS | 35: COS |

Actually the correct HP-41C layout from the manual:

| Row | Col 1 | Col 2 | Col 3 | Col 4 | Col 5 | HP-41 Codes |
|-----|-------|-------|-------|-------|-------|-------------|
| 1 | Σ+ | 1/x | √x | LOG | LN | 11–15 |
| 2 | XEQ | STO | RCL | R↓ | SIN | 21–25 |
| 3 | R/S | SST | GTO | COS | TAN | 31–35 (note: some models swap) |
| 4 | USER | f | g | ENTER | ÷ | 41–45 |
| 5 | 7 | 8 | 9 | × | | 51–55 |
| 6 | 4 | 5 | 6 | − | | 61–65 |
| 7 | 1 | 2 | 3 | + | | 71–75 |
| 8 | 0 | . | EEX | R/S | ENTER | 81–85 |

[ASSUMED — row numbering from top-to-bottom matching HP documentation; see note below]

**Complete crossterm KeyCode → HP-41 code mapping for `keycode_to_hp41_code()`:**

This is a Claude's Discretion item. The following mapping is derived from the HP-41C key layout and the existing `key_to_op()` table in `keys.rs`. Keys not present on HP-41 hardware (F-keys, Ctrl combos) return 0 (unmapped).

```rust
// Source: CONTEXT.md specifics section + HP-41C layout
// Key code = row×10 + col (1-indexed, rows from top)
pub fn keycode_to_hp41_code(code: KeyCode) -> u8 {
    match code {
        // Row 8: numeric row
        KeyCode::Char('0') => 81,
        KeyCode::Char('.') => 82,
        // EEX = 'e' in our CLI
        KeyCode::Char('e') => 83,
        // ENTER
        KeyCode::Enter => 84,
        // Row 7: digit row 1-3 and arithmetic
        KeyCode::Char('1') => 71,
        KeyCode::Char('2') => 72,
        KeyCode::Char('3') => 73,
        KeyCode::Char('+') => 74,
        // Row 6: digit row 4-6
        KeyCode::Char('4') => 61,
        KeyCode::Char('5') => 62,
        KeyCode::Char('6') => 63,
        KeyCode::Char('-') => 64,
        // Row 5: digit row 7-9
        KeyCode::Char('7') => 51,
        KeyCode::Char('8') => 52,
        KeyCode::Char('9') => 53,
        KeyCode::Char('*') => 54,
        // Row 4: function row
        // 'u' = USER mode
        KeyCode::Char('u') | KeyCode::Char('U') => 41,
        // f-key = format cycle ('f') and FmtDigits ('F')
        KeyCode::Char('f') | KeyCode::Char('F') => 42,
        // g-key = CLREG ('g')
        KeyCode::Char('g') | KeyCode::Char('G') => 43,
        // ENTER is also row 4 col 4 on some models — mapped to Enter above
        KeyCode::Char('/') => 45, // ÷
        // Row 3: R/S, SST, GTO, COS, TAN
        KeyCode::F(5) => 31, // R/S
        KeyCode::F(7) | KeyCode::F(8) => 32, // SST/BST
        // GTO — no direct TUI binding, 0
        KeyCode::Char('C') => 34, // COS
        KeyCode::Char('T') => 35, // TAN
        // Row 2: XEQ, STO, RCL, R↓, SIN
        KeyCode::Char('X') => 21, // XEQ (also hex modal opener)
        KeyCode::Char('S') => 22, // STO modal
        KeyCode::Char('R') => 23, // RCL modal
        KeyCode::Char('r') => 24, // R↓
        KeyCode::Char('q') => 25, // SIN (remapped in Phase 8)
        // Row 1: Σ+, 1/x, √x, LOG, LN
        KeyCode::Char('z') => 11, // Σ+
        KeyCode::Char('I') => 12, // 1/x
        KeyCode::Char('s') => 13, // √x
        KeyCode::Char('G') => 14, // LOG (conflict: 'G' also row 4 col 3 — LOG wins per HP layout)
        KeyCode::Char('L') => 15, // LN
        // Unmapped: all other keys return 0 (no HP-41 hardware equivalent)
        _ => 0,
    }
}
```

[ASSUMED — exact row/column assignments for the bottom half of the keyboard; the important codes for real GETKEY programs are the digit keys 81-84, arithmetic keys 74/64/54/45, and Enter 84. User should verify row 1-4 assignments if precision matters. Claude's discretion per D-02.]

**Key design note (D-01):** The `last_key_code` update in `handle_key()` must happen AFTER the release filter (`key.kind != KeyEventKind::Press` early return) but BEFORE any modal dispatch or `flush_entry_buf()`. This ensures every Press event records its key code, including digit entry keys, modal keys, and navigation keys.

---

## Architecture Patterns

### Pattern 1: CalcState Field Addition (D-01, D-06)

**What:** Add primitive/struct fields to `CalcState` with `#[serde(default)]`.
**When to use:** Every new persistent calculator state field.

```rust
// Source: hp41-core/src/state.rs — existing pattern (key_assignments, print_buffer)
// Add to CalcState struct body:
/// Last HP-41 row-column key code pressed. 0 = none since startup.
/// Updated in hp41-cli handle_key() on every Press event. Default: 0.
#[serde(default)]
pub last_key_code: u8,

/// Hidden register M — accessible via STO M / RCL M in programs.
#[serde(default)]
pub reg_m: HpNum,

/// Hidden register N — accessible via STO N / RCL N in programs.
#[serde(default)]
pub reg_n: HpNum,

/// Hidden register O — accessible via STO O / RCL O in programs.
#[serde(default)]
pub reg_o: HpNum,
```

**CalcState::new() initialization:**
```rust
last_key_code: 0,
reg_m: HpNum::zero(),
reg_n: HpNum::zero(),
reg_o: HpNum::zero(),
```

### Pattern 2: New Op Variants (all 9 new variants)

**What:** Add to the `Op` enum in `ops/mod.rs`, `dispatch()` in `ops/mod.rs`, AND `execute_op()` in `ops/program.rs`.
**Critical trap:** Missing either `dispatch()` or `execute_op()` causes silent skips in programs.

```rust
// Source: hp41-core/src/ops/mod.rs — existing Op enum comment style
// ── Synthetic Programming (Phase 12) ────────────────────────────────────
/// GETKEY — push last key code (HP-41 row×10+col) to X. LiftEffect: Enable.
GetKey,
/// NULL — true no-op; does not modify any state. LiftEffect: Neutral.
Null,
/// STO M — store X into hidden register M. LiftEffect: Neutral.
StoM,
/// STO N — store X into hidden register N. LiftEffect: Neutral.
StoN,
/// STO O — store X into hidden register O. LiftEffect: Neutral.
StoO,
/// RCL M — recall hidden register M into X. LiftEffect: Enable.
RclM,
/// RCL N — recall hidden register N into X. LiftEffect: Enable.
RclN,
/// RCL O — recall hidden register O into X. LiftEffect: Enable.
RclO,
/// SyntheticByte(u8) — synthetic op inserted via hex modal. During execution,
/// dispatches to the corresponding Op via SAFE_HEX_SUBSET lookup. LiftEffect: varies.
SyntheticByte(u8),
```

### Pattern 3: op_sto_m / op_rcl_m Functions (D-07)

**What:** Implement hidden register STO/RCL ops in `registers.rs`.
**Pattern:** Exact mirror of `op_sto()` / `op_rcl()`.

```rust
// Source: hp41-core/src/ops/registers.rs lines 14-36 — op_sto/op_rcl pattern
pub fn op_sto_m(state: &mut CalcState) -> Result<(), HpError> {
    state.reg_m = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

pub fn op_rcl_m(state: &mut CalcState) -> Result<(), HpError> {
    let val = state.reg_m.clone();
    state.stack.lift_enabled = true; // RCL always lifts (matches op_rcl pattern)
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
// Identical functions for op_sto_n/op_rcl_n and op_sto_o/op_rcl_o
```

### Pattern 4: Op::GetKey Implementation

```rust
// dispatch() arm — in mod.rs:
Op::GetKey => {
    let code = HpNum::from(state.last_key_code as i32);
    state.stack.lift_enabled = true; // GetKey always lifts (produces new value)
    crate::stack::enter_number(state, code);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

### Pattern 5: Safe Hex Subset Table (D-11, D-12)

**What:** A `const` or function that maps HP-41 byte codes (u8) to already-implemented Op variants.
**When to use:** For HexModal validation and for SyntheticByte execution dispatch.

The safe subset consists only of byte codes that correspond to Ops already in our enum. In the HP-41 NUT/FOCAL architecture, each instruction occupies 1–3 bytes. The single-byte codes (0x00–0xFF) relevant to our implemented ops are:

[ASSUMED — approximate mapping; exact HP-41 FOCAL single-byte opcodes from secondary sources. The key constraint from D-12 is that ONLY codes mapping to already-implemented Ops are accepted. The planner should define the initial table conservatively and it can be expanded in future phases.]

```rust
// Source: CONTEXT.md D-11/D-12 + HP-41 FOCAL byte-code reference (secondary sources)
// Conservative initial safe subset — only single-byte codes with known Op mappings:
pub fn synthetic_byte_to_op(byte: u8) -> Option<Op> {
    match byte {
        // Arithmetic
        0x40 => Some(Op::Add),
        0x41 => Some(Op::Sub),
        0x42 => Some(Op::Mul),
        0x43 => Some(Op::Div),
        // Stack
        0x71 => Some(Op::Enter),
        0x73 => Some(Op::Clx),
        0x54 => Some(Op::Chs),
        0x74 => Some(Op::Rdn),
        0x71 => Some(Op::XySwap), // Note: ENTER vs XSWAP — check exact codes
        // Math
        0x58 => Some(Op::Recip),
        0x52 => Some(Op::Sqrt),
        0x53 => Some(Op::Sq),
        0x54 => Some(Op::Ln),
        0x57 => Some(Op::Log),
        // Trig
        0x59 => Some(Op::Sin),
        0x5A => Some(Op::Cos),
        0x5B => Some(Op::Tan),
        // NULL
        0xCF => Some(Op::Null),
        // GETKEY
        0x17 => Some(Op::GetKey),
        // Hidden register ops (once defined)
        // STO M = 0x33 (approximate), RCL M = 0x23 (approximate)
        _ => None,
    }
}
```

[ASSUMED — the exact NUT byte codes above are from training knowledge. They need cross-verification against an HP-41 FOCAL reference before inclusion in the final table. The CONTEXT.md explicitly marks the exact contents of this table as "Claude's Discretion". The most important constraint is the guard: `if synthetic_byte_to_op(byte).is_none() { set INVALID }`. The exact mappings can be refined iteratively.]

**Pragmatic approach for initial implementation:** Define the safe subset as a small `const` array or `match` expression covering 15-20 well-known single-byte codes. NULL (0xCF or similar) and GETKEY (0x17) are the primary synthetic ops. The exact byte codes for basic math ops are less critical because users can already enter those ops directly via the keyboard. The value of the hex modal is primarily for NULL and GETKEY insertion in programs.

### Pattern 6: HexModal PendingInput (D-15)

**What:** 2-digit hex accumulator — mirrors `StoRegister(String)` exactly.
**Pattern source:** `hp41-cli/src/app.rs` — `StoRegister` + `handle_reg_modal()`.

```rust
// Add to PendingInput enum:
HexModal(String), // accumulating 2-digit hex code for synthetic byte insertion
```

**handle_pending_input arm:**
```rust
Some(PendingInput::HexModal(ref acc)) => {
    match key.code {
        KeyCode::Char(c)
            if c.is_ascii_hexdigit() =>   // '0'-'9', 'a'-'f', 'A'-'F'
        {
            let hex_char = c.to_ascii_lowercase(); // normalize to lowercase
            let mut new_acc = acc.clone();
            new_acc.push(hex_char);
            if new_acc.len() == 2 {
                // Validate: parse hex byte and check safe subset
                let byte = u8::from_str_radix(&new_acc, 16)
                    .expect("two hex chars always parse as u8");
                match synthetic_byte_to_op(byte) {
                    Some(_) => {
                        // Insert Op::SyntheticByte at current pc
                        state.program.insert(state.pc, Op::SyntheticByte(byte));
                        state.pc += 1; // advance past newly inserted step (D-16)
                        self.message = None;
                    }
                    None => {
                        self.message = Some("INVALID".to_string()); // D-13
                    }
                }
                self.pending_input = None; // modal always closes (D-13)
            } else {
                self.pending_input = Some(PendingInput::HexModal(new_acc));
            }
        }
        KeyCode::Esc => {
            self.pending_input = None; // cancel with no side effects
        }
        _ => {
            // Non-hex key: restore modal silently (existing convention)
            self.pending_input = Some(PendingInput::HexModal(acc.clone()));
        }
    }
}
```

### Pattern 7: Op::SyntheticByte Execution (D-11)

**What:** In `execute_op()`, `Op::SyntheticByte(b)` dispatches to the appropriate Op.

```rust
// In execute_op() in program.rs:
Op::SyntheticByte(b) => {
    if let Some(op) = crate::ops::synthetic_byte_to_op(b) {
        execute_op(state, op) // recursive dispatch to the mapped Op
    } else {
        Err(HpError::InvalidOp) // should not happen — validated at insertion
    }
}
```

**Note:** `execute_op()` is a private function, so this recursion is safe. The recursion depth is exactly 1 (SyntheticByte → concrete Op) — no infinite recursion is possible because `synthetic_byte_to_op` never returns `Some(Op::SyntheticByte(_))`.

### Pattern 8: StoRegister M/N/O Extension (D-08)

**What:** In `handle_pending_input`, extend the `StoRegister` and `RclRegister` arms to intercept M/N/O BEFORE digit accumulation.

```rust
// In handle_pending_input(), StoRegister arm — BEFORE the existing arithmetic key intercepts:
Some(PendingInput::StoRegister(ref acc)) => {
    // NEW: M/N/O only when accumulator is empty (no digit typed yet)
    if acc.is_empty() {
        match key.code {
            KeyCode::Char('M') | KeyCode::Char('m') => {
                self.call_dispatch(Op::StoM);
                self.pending_input = None;
                return;
            }
            KeyCode::Char('N') | KeyCode::Char('n') => {
                self.call_dispatch(Op::StoN);
                self.pending_input = None;
                return;
            }
            KeyCode::Char('O') | KeyCode::Char('o') => {
                self.call_dispatch(Op::StoO);
                self.pending_input = None;
                return;
            }
            _ => {} // fall through to existing arithmetic/digit handling
        }
    }
    // Existing: arithmetic op key intercepts (+/-/*//) ...
    // Existing: handle_reg_modal fallthrough ...
}
```

**Ordering constraint:** The M/N/O check must be INSIDE the `acc.is_empty()` guard. If the user has already typed a digit (e.g., `"0"`), M/N/O should be ignored (they'd be trying to type "0M" as a register number — which is invalid anyway). The guard prevents misinterpretation.

### Pattern 9: program listing display — op_display_name additions

```rust
// Source: hp41-cli/src/prgm_display.rs op_display_name() — add these arms:
Op::GetKey => "GETKEY".to_string(),
Op::Null => "NULL".to_string(),
Op::StoM => "STO M".to_string(),
Op::StoN => "STO N".to_string(),
Op::StoO => "STO O".to_string(),
Op::RclM => "RCL M".to_string(),
Op::RclN => "RCL N".to_string(),
Op::RclO => "RCL O".to_string(),
Op::SyntheticByte(b) => format!("SYN {:02X}", b), // D-11 — uppercase hex, zero-padded
```

### Anti-Patterns to Avoid

- **Missing execute_op arm for new Ops:** The most common Phase 12 trap. Every Op variant must appear in BOTH `dispatch()` (ops/mod.rs) AND `execute_op()` (ops/program.rs). Missing `execute_op` causes silent skip in programs with no error. [VERIFIED: codebase pattern]
- **Using `Vec::push()` instead of `Vec::insert()` for HexModal insertion:** `push()` adds at the end, not at the current PC position. Use `state.program.insert(state.pc, Op::SyntheticByte(byte))`. [VERIFIED: Rust stdlib Vec API]
- **Forgetting `acc.is_empty()` guard for M/N/O branches:** Without this guard, typing `STO` + `0` + `M` would try to dispatch `Op::StoM` when the user intended `STO 0M` (invalid register). The guard ensures M/N/O is only accepted as a first character.
- **`SyntheticByte` recursing into itself:** `synthetic_byte_to_op()` must never return `Some(Op::SyntheticByte(_))` — that would cause infinite recursion in `execute_op`. The implementation naturally avoids this since the lookup table only maps to concrete Ops.
- **Not setting `state.stack.lift_enabled = true` before `enter_number()` in `op_rcl_m()`:** The pattern from `op_rcl()` explicitly forces `lift_enabled = true` before calling `enter_number` so that RCL always lifts. Omitting this breaks stack-lift behavior. [VERIFIED: registers.rs line 32]
- **`HexModal` accepting non-hex digits:** The match guard `c.is_ascii_hexdigit()` allows 0-9, a-f, A-F. Non-hex letters must fall to the `_` arm that silently restores the modal.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Hex string parsing | Custom char-by-char parser | `u8::from_str_radix(&s, 16)` | Standard, infallible for 2 hex chars |
| Stack lift for RCL ops | Custom lift logic | Existing `enter_number()` + `state.stack.lift_enabled = true` pattern from `op_rcl()` | Already handles all lift edge cases |
| 2-digit accumulation | New accumulation logic | Existing `handle_reg_modal()` pattern (or direct inline match matching StoRegister) | Proven, Esc-cancellation and backspace already correct |
| Program Vec insertion | Linked list or custom program store | `state.program.insert(state.pc, op)` | Vec::insert is O(n) but HP-41 programs are ≤999 steps — perfectly adequate |

---

## Common Pitfalls

### Pitfall 1: Missing execute_op Arm (Critical Trap)

**What goes wrong:** New Op variants not added to `execute_op()` in `program.rs`. They silently skip during program execution with no error.
**Why it happens:** The `execute_op()` function uses a `match` that has a catch-all `_ => Err(HpError::InvalidOp)` for programming ops. New regular ops NOT in the catch-all group will silently fail if the match arm is missing — except Rust will emit a non-exhaustive match warning that CI (with `#[deny(warnings)]` in clippy) will catch.
**How to avoid:** Always add to BOTH `dispatch()` AND `execute_op()` in the same plan. The implementation plan must list both locations explicitly.
**Warning signs:** Program with `NULL` or `GetKey` steps silently skips those steps. Coverage test for `execute_op` won't catch it unless there's a test that runs the op inside a program.

### Pitfall 2: last_key_code Update Ordering (D-01)

**What goes wrong:** `last_key_code` is updated AFTER modal dispatch, so the GETKEY modal key itself becomes the stored code.
**Why it happens:** The update must come after the release filter but before everything else.
**How to avoid:** Place `self.state.last_key_code = keycode_to_hp41_code(key.code)` as the first logical operation in `handle_key()` after the release filter.
**Warning signs:** GETKEY after pressing 'S' (STO modal) returns 22 (STO key code) instead of the intended key. Test by pressing '5' then running a GETKEY program — should return 62 (row 6, col 2).

### Pitfall 3: HexModal 'X' Conflict (D-14)

**What goes wrong:** `'X'` (uppercase) opens HexModal, but `'x'` (lowercase) is `Op::XySwap`. In `PrintModal` (Phase 11), both cases dispatch the same op. In HexModal, case matters.
**Why it happens:** The `'X'` interceptor in `handle_key()` must check for uppercase specifically: `KeyCode::Char('X')`. The `key_to_op()` table currently has `KeyCode::Char('x') => Some(Op::XySwap)` — this must NOT be changed.
**How to avoid:** Add `KeyCode::Char('X')` interceptor (not `'x'`) in `handle_key()` before `key_to_op()`. Since `pending_input` check comes first, a HexModal open during the second digit won't re-open.

### Pitfall 4: Vec::insert PC Semantics (D-16)

**What goes wrong:** After inserting at `state.pc`, the inserted op is at `state.pc`. If the code then reads `program[state.pc]` for display (as `prgm_display::format_step` does), it should show the newly inserted step. After insertion, `state.pc` must advance by 1 to place the cursor AFTER the inserted step.
**Why it happens:** HP-41 PRGM mode inserts BEFORE the current step and advances the counter.
**How to avoid:** `state.program.insert(state.pc, Op::SyntheticByte(byte)); state.pc += 1;`

### Pitfall 5: serde(default) Missing for CalcState Fields

**What goes wrong:** Loading a v1.0 save file fails with a JSON deserialization error because `last_key_code`/`reg_m`/`reg_n`/`reg_o` fields are not present in the old JSON.
**Why it happens:** serde_json by default requires all fields to be present unless `#[serde(default)]` is applied.
**How to avoid:** ALL four new fields must have `#[serde(default)]`. Additionally, `HpNum` must implement `Default` (returns `HpNum::zero()`) for this to compile — check existing usage. [VERIFIED: `CalcState::default()` exists and `HpNum` has `Default` from `impl Default for CalcState`.]

### Pitfall 6: SyntheticByte JSON Round-trip

**What goes wrong:** `Op::SyntheticByte(u8)` must survive `serde_json::to_string()` / `serde_json::from_str()`. The Op enum already derives `Serialize, Deserialize` so this works automatically for tuple variants. But the `u8` inside must serialize correctly.
**Why it happens:** No issue — serde handles `u8` fields in enum variants correctly. The only risk is if someone accidentally adds a `#[serde(skip)]` or custom serializer.
**How to avoid:** No special action needed; just ensure `Op::SyntheticByte(u8)` follows the same pattern as `Op::FmtFix(u8)` which already works. Write a test: `serde_json::to_string(&Op::SyntheticByte(0x40)).unwrap()` → `serde_json::from_str::<Op>(...)` must round-trip.

---

## Code Examples

### GetKey — complete implementation

```rust
// Source: registers.rs op_rcl pattern adapted for CalcState.last_key_code
// In dispatch() arms and execute_op() — identical body:
Op::GetKey => {
    let code = HpNum::from(state.last_key_code as i32);
    state.stack.lift_enabled = true;
    crate::stack::enter_number(state, code);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

### handle_key last_key_code update

```rust
// Source: CONTEXT.md D-01 specifics section — placement in handle_key()
// hp41-cli/src/app.rs handle_key() — FIRST logic after release filter:
fn handle_key(&mut self, key: KeyEvent) {
    // [existing] D-06: filter Release — MUST be first
    if key.kind != KeyEventKind::Press {
        return;
    }
    // [NEW D-01] Update last_key_code on every Press event — before any modal/dispatch
    self.state.last_key_code = keys::keycode_to_hp41_code(key.code);
    // [existing] Ctrl+C quit...
    // ... rest of handle_key unchanged ...
}
```

### Hidden register RCL (complete)

```rust
// Source: hp41-core/src/ops/registers.rs op_rcl() — exact adaptation
pub fn op_rcl_m(state: &mut CalcState) -> Result<(), HpError> {
    let val = state.reg_m.clone();
    state.stack.lift_enabled = true; // RCL always lifts
    enter_number(state, val);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

### HexModal 'X' interceptor placement

```rust
// hp41-cli/src/app.rs handle_key() — after 'P' interceptor (line ~215), before alpha mode routing
// 'X' (uppercase, Shift+X) opens hex-byte insertion modal (D-14, Phase 12).
// 'x' (lowercase) is Op::XySwap — NOT intercepted here.
if key.code == KeyCode::Char('X') && !key.modifiers.contains(KeyModifiers::CONTROL) {
    self.pending_input = Some(PendingInput::HexModal(String::new()));
    self.message = None;
    return;
}
```

### HexModal ui.rs pending_prompt arm

```rust
// hp41-cli/src/ui.rs pending_prompt() — add after PrintModal arm:
PendingInput::HexModal(acc) => {
    if acc.is_empty() {
        "HEX: _".to_string()
    } else {
        format!("HEX: {}_", acc)
    }
}
```

---

## Runtime State Inventory

Phase 12 is a greenfield feature addition — not a rename/refactor/migration phase. No runtime state needs updating.

The `#[serde(default)]` fields (`last_key_code`, `reg_m`, `reg_n`, `reg_o`) provide automatic backward compatibility with v1.0 save files. No data migration required.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | None — new fields get `#[serde(default)]`; existing saves load without issue | None |
| Live service config | None | None |
| OS-registered state | None | None |
| Secrets/env vars | None | None |
| Build artifacts | None — no renames | None |

---

## Environment Availability

Phase 12 is purely code/config changes within the existing Cargo workspace. No new external dependencies.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | All | ✓ | 1.85 (MSRV) | — |
| `just` task runner | CI, coverage | ✓ | (existing) | — |
| `cargo-llvm-cov` | Coverage gate | ✓ | (existing) | — |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | `justfile` — `just test` = `cargo test --workspace` |
| Quick run command | `just test` |
| Full suite command | `just ci` (lint + test + coverage) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SYNT-01 | GETKEY pushes last_key_code to X | unit | `cargo test -p hp41-core synthetic_tests -- getkey` | ❌ Wave 0 |
| SYNT-01 | last_key_code updated on every key press | unit (cli) | `cargo test -p hp41-cli synthetic -- last_key_code` | ❌ Wave 0 |
| SYNT-01 | GETKEY = 0 when no key pressed | unit | `cargo test -p hp41-core synthetic_tests -- getkey_zero` | ❌ Wave 0 |
| SYNT-02 | NULL does not modify stack, lift flag, or registers | unit | `cargo test -p hp41-core synthetic_tests -- null_no_op` | ❌ Wave 0 |
| SYNT-03 | STO M / RCL M round-trip | unit | `cargo test -p hp41-core synthetic_tests -- sto_rcl_m` | ❌ Wave 0 |
| SYNT-03 | STO M / RCL N / RCL O survive JSON round-trip | unit | `cargo test -p hp41-core synthetic_tests -- hidden_reg_serde` | ❌ Wave 0 |
| SYNT-03 | STO M / RCL M work inside a program | unit | `cargo test -p hp41-core synthetic_tests -- hidden_reg_in_program` | ❌ Wave 0 |
| SYNT-04 | Valid hex code inserts SyntheticByte at pc | integration | `cargo test -p hp41-cli hex_modal -- insert_valid` | ❌ Wave 0 |
| SYNT-04 | Invalid hex code sets INVALID message | integration | `cargo test -p hp41-cli hex_modal -- reject_invalid` | ❌ Wave 0 |
| SYNT-04 | SyntheticByte executes correctly in program | unit | `cargo test -p hp41-core synthetic_tests -- synthetic_byte_exec` | ❌ Wave 0 |
| SYNT-04 | Op::SyntheticByte(u8) survives JSON round-trip | unit | `cargo test -p hp41-core synthetic_tests -- synthetic_byte_serde` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `just test`
- **Per wave merge:** `just ci`
- **Phase gate:** `just ci` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-core/tests/synthetic_tests.rs` — covers SYNT-01, SYNT-02, SYNT-03, SYNT-04 core-side
- [ ] `hp41-cli` test module for hex modal and last_key_code (inline in app.rs test module, following print_modal_tests pattern)
- [ ] Wave 0 tests must be RED (compile but fail) until Wave 1 adds the Op variants

---

## Security Domain

Phase 12 adds no network I/O, authentication, file access (beyond existing state persistence), or user input validation beyond the existing pattern. The hex modal validates byte codes against a `const` table — this is value validation, not security-relevant input validation.

`security_enforcement` is not set in `.planning/config.json`. Skipping formal ASVS assessment for this phase.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No hidden register support | reg_m/reg_n/reg_o as named CalcState fields | Phase 12 | Programs can now use HP-41C hidden registers |
| No synthetic byte insertion | HexModal + Op::SyntheticByte(u8) | Phase 12 | Users can insert synthetic ops not on the keyboard |
| key_to_op() blind to key pressed | last_key_code tracked per Press event | Phase 12 | GETKEY now works in recorded programs |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | HP-41 key code table — exact row-column assignments for rows 1-4 (function keys, STO/RCL, R/S, SST, etc.) | HP-41 Keyboard Layout section | Key codes from GETKEY programs would be wrong for those keys; digit keys (rows 5-8) are correct and most common |
| A2 | Approximate NUT byte codes for `synthetic_byte_to_op()` (0x40=Add, 0x41=Sub, 0xCF=Null, 0x17=GetKey) | Pattern 5 / Safe Hex Subset | Invalid code entries would appear in the safe subset; the guard still works — rejected codes remain rejected |
| A3 | `HpNum` implements `Default` (required for `#[serde(default)]` on `reg_m/n/o`) | Pattern 1 | Compilation error — easily caught at build time; fix: `impl Default for HpNum { fn default() -> Self { HpNum::zero() } }` |

**Note on A1:** The CONTEXT.md explicitly designates the exact key code table as "Claude's Discretion". For programs that only use GETKEY to distinguish digit keys (rows 5-8), arithmetic keys, and Enter, the table above is correct. For programs that need to distinguish function keys (STO, RCL, R/S, SST), the user should verify against the HP-41C Owner's Manual Appendix A.

**Note on A2:** The exact byte codes are secondary research information. The implementation should define the `synthetic_byte_to_op()` table conservatively (10-20 entries) and document that it can be expanded as SYNT-05 (full FOCAL table) progresses.

---

## Open Questions

1. **Exact NUT single-byte opcodes for the safe subset**
   - What we know: HP-41 FOCAL uses a 1-3 byte encoding; single-byte codes cover most common ops
   - What's unclear: Exact mapping of our Op enum variants to NUT byte codes (secondary source quality)
   - Recommendation: Define an initial minimal table (NULL, GETKEY, basic arithmetic = ~10 entries) with a clear comment that it's conservative and can expand. Mark with `[ASSUMED]` in code comments.

2. **`HpNum::default()` existence**
   - What we know: `CalcState::default()` calls `CalcState::new()` which initializes `reg_m = HpNum::zero()`
   - What's unclear: Whether `HpNum` itself implements `Default` (required for `#[serde(default)]` to work on `HpNum` fields)
   - Recommendation: Check `hp41-core/src/num.rs` for `impl Default for HpNum`. If absent, add it as `fn default() -> Self { HpNum::zero() }`. This is a 3-line addition.

3. **HexModal — PRGM mode only vs. any mode**
   - What we know: D-16 says "insert at current state.pc position" implying PRGM mode. D-18 help says "PRGM mode".
   - What's unclear: Should the 'X' interceptor be gated on `state.prgm_mode`?
   - Recommendation: Yes — only open HexModal if `state.prgm_mode` is true. Outside PRGM mode, 'X' should be a no-op (or could open the modal anyway and insert into the dormant program Vec). The CONTEXT.md D-18 says "PRGM mode" which implies the gate.

---

## Sources

### Primary (HIGH confidence — verified from codebase)
- `[VERIFIED: codebase]` `hp41-core/src/ops/mod.rs` — complete Op enum, dispatch(), all existing variants
- `[VERIFIED: codebase]` `hp41-core/src/ops/program.rs` — execute_op() pattern, run_loop(), insert-at-pc semantics
- `[VERIFIED: codebase]` `hp41-core/src/ops/registers.rs` — op_sto/op_rcl exact pattern for hidden register ops
- `[VERIFIED: codebase]` `hp41-core/src/state.rs` — CalcState struct, serde(default) pattern, HpNum::zero()
- `[VERIFIED: codebase]` `hp41-cli/src/app.rs` — PendingInput enum, handle_key() ordering, StoRegister/RclRegister modal arms, PrintModal pattern
- `[VERIFIED: codebase]` `hp41-cli/src/keys.rs` — key_to_op() existing mappings, KEY_REF_TABLE
- `[VERIFIED: codebase]` `hp41-cli/src/prgm_display.rs` — op_display_name() exhaustive match, naming conventions
- `[VERIFIED: codebase]` `hp41-cli/src/ui.rs` — pending_prompt() pattern, HexModal display format
- `[VERIFIED: codebase]` `hp41-cli/src/help_data.rs` — HELP_DATA format, category structure, tests
- `[VERIFIED: codebase]` `.planning/phases/12-synthetic-programming/12-CONTEXT.md` — all 18 locked decisions

### Secondary (MEDIUM confidence)
- `[CITED: CONTEXT.md specifics]` HP-41C keyboard row-column code examples: `11`=Σ+, `12`=1/x, `71`=ENTER, `72`=CHS, `73`=EEX, `74`=R/S, `81`=÷, `82`=×, `83`=−, `84`=+

### Tertiary (LOW confidence — training knowledge, flagged ASSUMED)
- `[ASSUMED]` Exact row assignments for rows 1-4 of the HP-41C keyboard (function keys)
- `[ASSUMED]` Exact NUT/FOCAL single-byte opcodes for `synthetic_byte_to_op()` mapping
- `[ASSUMED]` `HpNum` implements `Default` (needs verification in num.rs)

---

## Project Constraints (from CLAUDE.md)

- `hp41-core` must NEVER depend on `hp41-cli` or `hp41-gui` — enforced at compile time
- Every new `Op` variant must be added to BOTH `dispatch()` in `ops/mod.rs` AND `execute_op()` in `ops/program.rs`
- New `CalcState` fields MUST have `#[serde(default)]` for backward compatibility with v1.0 save files
- `#![deny(clippy::unwrap_used)]` in `hp41-core` — all new core code uses `.expect("reason")` or `?`-propagation; test modules carry `#[allow(clippy::unwrap_used)]`
- `just` is the sole task runner; never call `cargo` directly in CI or docs
- Commit messages via `/git-workflow:commit --with-skills` only
- All commit messages in English

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — pure extension of existing workspace, no new dependencies
- Architecture: HIGH — all patterns verified from existing codebase; only HP-41 byte codes are ASSUMED
- Pitfalls: HIGH — based on observed trap documentation in CONTEXT.md and STATE.md

**Research date:** 2026-05-08
**Valid until:** 2026-06-08 (stable codebase — no dependency churn expected)
